//NPC_behavior.rs
/*
FIXME - MCG:
These all need to make use of the snapshots.  Write something that can look for only specific
things in a snapshot or just go through the snapshot every frame and save the info in case
we need it...
*/
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

// leave this line at the top for all NPC_xxxx.cpp files...
use crate::code::game::g_headers_h::*;
use crate::code::game::g_navigator_h::*;
use crate::code::game::Q3_Interface_h::*;
use core::ptr::{addr_of, addr_of_mut};

extern "C" {
    static mut g_AIsurrender: *mut cvar_t;
    static mut showBBoxes: qboolean;
    static mut g_crosshairEntNum: c_int;
    fn CG_Cube(mins: vec3_t, maxs: vec3_t, color: vec3_t, alpha: f32);
    fn NPC_CheckGetNewWeapon();
    fn PM_InKnockDown(ps: *mut playerState_t) -> qboolean;
    fn NPC_AimAdjust(change: c_int);
    fn G_StandardHumanoid(self_: *mut gentity_t) -> qboolean;
    // forward declaration of MakeOwnerInvis (defined elsewhere, referenced in BeamOut comments)
    fn MakeOwnerInvis(self_: *mut gentity_t);
    fn NPC_MoveDirClear(forwardmove: c_int, rightmove: c_int, reset: qboolean) -> qboolean;
    fn G_AddVoiceEvent(ent: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    fn WP_DropWeapon(dropper: *mut gentity_t, velocity: *mut f32);
    fn ChangeWeapon(ent: *mut gentity_t, newWeapon: c_int);
    fn NPC_SearchForWeapons() -> *mut gentity_t;
    fn G_CanPickUpWeapons(other: *mut gentity_t) -> qboolean;
}

static mut NPCDEBUG_BLUE: vec3_t = [0.0, 0.0, 1.0];

const APEX_HEIGHT: f32 = 200.0;
// #define PARA_WIDTH (sqrt(APEX_HEIGHT)+sqrt(APEX_HEIGHT)) -- not representable as Rust const (sqrt is not const fn)
const JUMP_SPEED: f32 = 200.0;

/*
 void NPC_BSAdvanceFight (void)

Advance towards your captureGoal and shoot anyone you can along the way.
*/
pub unsafe fn NPC_BSAdvanceFight() {
    //FIXME: IMPLEMENT
    //Head to Goal if I can

    //Make sure we're still headed where we want to capture
    if !(*NPCInfo).captureGoal.is_null() {
        //FIXME: if no captureGoal, what do we do?
        //VectorCopy( NPCInfo->captureGoal->currentOrigin, NPCInfo->tempGoal->currentOrigin );
        //NPCInfo->goalEntity = NPCInfo->tempGoal;

        NPC_SetMoveGoal(NPC, (*(*NPCInfo).captureGoal).currentOrigin, 16, qtrue);

        (*NPCInfo).goalTime = level.time + 100000;
    }

    //	NPC_BSRun();

    NPC_CheckEnemy(qtrue, qfalse);

    //FIXME: Need melee code
    if !(*NPC).enemy.is_null() {
        //See if we can shoot him
        let mut delta: vec3_t = [0.0; 3];
        let mut forward: vec3_t = [0.0; 3];
        let mut angleToEnemy: vec3_t = [0.0; 3];
        let mut hitspot: vec3_t = [0.0; 3];
        let mut muzzle: vec3_t = [0.0; 3];
        let mut diff: vec3_t = [0.0; 3];
        let mut enemy_org: vec3_t = [0.0; 3];
        let mut enemy_head: vec3_t = [0.0; 3];
        let mut distanceToEnemy: f32 = 0.0;
        let mut attack_ok: qboolean = qfalse;
        let mut dead_on: qboolean = qfalse;
        let mut attack_scale: f32 = 1.0;
        let mut aim_off: f32 = 0.0;
        let max_aim_off: f32 = 64.0;

        //Yaw to enemy
        VectorMA((*(*NPC).enemy).absmin, 0.5, (*(*NPC).enemy).maxs, enemy_org);
        CalcEntitySpot(NPC, SPOT_WEAPON, muzzle);

        VectorSubtract(enemy_org, muzzle, delta);
        vectoangles(delta, angleToEnemy);
        distanceToEnemy = VectorNormalize(delta);

        if NPC_EnemyTooFar((*NPC).enemy, distanceToEnemy * distanceToEnemy, qtrue) == 0 {
            attack_ok = qtrue;
        }

        if attack_ok != 0 {
            NPC_UpdateShootAngles(angleToEnemy, qfalse, qtrue);

            (*NPCInfo).enemyLastVisibility = enemyVisibility;
            enemyVisibility = NPC_CheckVisibility((*NPC).enemy, CHECK_FOV); //CHECK_360|//CHECK_PVS|

            if enemyVisibility == VIS_FOV {
                //He's in our FOV

                attack_ok = qtrue;
                CalcEntitySpot((*NPC).enemy, SPOT_HEAD, enemy_head);

                if attack_ok != 0 {
                    let mut tr: trace_t = core::mem::zeroed();
                    let mut traceEnt: *mut gentity_t;
                    //are we gonna hit him if we shoot at his center?
                    (gi.trace)(
                        addr_of_mut!(tr),
                        muzzle.as_ptr(),
                        core::ptr::null(),
                        core::ptr::null(),
                        enemy_org.as_ptr(),
                        (*NPC).s.number,
                        MASK_SHOT,
                    );
                    traceEnt = addr_of_mut!(g_entities[tr.entityNum as usize]);
                    if traceEnt != (*NPC).enemy
                        && (traceEnt.is_null()
                            || (*traceEnt).client.is_null()
                            || (*(*NPC).client).enemyTeam == 0
                            || (*(*NPC).client).enemyTeam
                                != (*(*traceEnt).client).playerTeam)
                    {
                        //no, so shoot for the head
                        attack_scale *= 0.75;
                        (gi.trace)(
                            addr_of_mut!(tr),
                            muzzle.as_ptr(),
                            core::ptr::null(),
                            core::ptr::null(),
                            enemy_head.as_ptr(),
                            (*NPC).s.number,
                            MASK_SHOT,
                        );
                        traceEnt = addr_of_mut!(g_entities[tr.entityNum as usize]);
                    }

                    VectorCopy(tr.endpos, hitspot);

                    if traceEnt == (*NPC).enemy
                        || (!(*traceEnt).client.is_null()
                            && (*(*NPC).client).enemyTeam != 0
                            && (*(*NPC).client).enemyTeam
                                == (*(*traceEnt).client).playerTeam)
                    {
                        dead_on = qtrue;
                    } else {
                        attack_scale *= 0.5;
                        if (*(*NPC).client).playerTeam != 0 {
                            if !traceEnt.is_null()
                                && !(*traceEnt).client.is_null()
                                && (*(*traceEnt).client).playerTeam != 0
                            {
                                if (*(*NPC).client).playerTeam
                                    == (*(*traceEnt).client).playerTeam
                                {
                                    //Don't shoot our own team
                                    attack_ok = qfalse;
                                }
                            }
                        }
                    }
                }

                if attack_ok != 0 {
                    //ok, now adjust pitch aim
                    VectorSubtract(hitspot, muzzle, delta);
                    vectoangles(delta, angleToEnemy);
                    (*(*NPC).NPC).desiredPitch = angleToEnemy[PITCH as usize];
                    NPC_UpdateShootAngles(angleToEnemy, qtrue, qfalse);

                    if dead_on == 0 {
                        //We're not going to hit him directly, try a suppressing fire
                        //see if where we're going to shoot is too far from his origin
                        AngleVectors(
                            (*NPCInfo).shootAngles,
                            forward,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                        );
                        VectorMA(muzzle, distanceToEnemy, forward, hitspot);
                        VectorSubtract(hitspot, enemy_org, diff);
                        aim_off = VectorLength(diff);
                        if aim_off > random() * max_aim_off {
                            //FIXME: use aim value to allow poor aim?
                            attack_scale *= 0.75;
                            //see if where we're going to shoot is too far from his head
                            VectorSubtract(hitspot, enemy_head, diff);
                            aim_off = VectorLength(diff);
                            if aim_off > random() * max_aim_off {
                                attack_ok = qfalse;
                            }
                        }
                        attack_scale *= (max_aim_off - aim_off + 1.0) / max_aim_off;
                    }
                }
            }
        }

        if attack_ok != 0 {
            if NPC_CheckAttack(attack_scale) != 0 {
                //check aggression to decide if we should shoot
                enemyVisibility = VIS_SHOOT;
                WeaponThink(qtrue);
            } else {
                attack_ok = qfalse;
            }
        }
        //Don't do this- only for when stationary and trying to shoot an enemy
        //		else
        //			NPC->cantHitEnemyCounter++;
    } else {
        //FIXME:
        NPC_UpdateShootAngles((*(*NPC).client).ps.viewangles, qtrue, qtrue);
    }

    if ucmd.forwardmove == 0 && ucmd.rightmove == 0 {
        //We reached our captureGoal
        if (*NPC).m_iIcarusID != IIcarusInterface::ICARUS_INVALID {
            /*NPC->taskManager*/
            Q3_TaskIDComplete(NPC, TID_BSTATE);
        }
    }
}

pub unsafe fn Disappear(self_: *mut gentity_t) {
    //	ClientDisconnect(self);
    (*self_).s.eFlags |= EF_NODRAW;
    (*self_).e_ThinkFunc = thinkF_NULL;
    (*self_).nextthink = -1;
}

pub unsafe fn BeamOut(self_: *mut gentity_t) {
    //	gentity_t *tent = G_Spawn();

    /*
    	tent->owner = self;
    	tent->think = MakeOwnerInvis;
    	tent->nextthink = level.time + 1800;
    	//G_AddEvent( ent, EV_PLAYER_TELEPORT, 0 );
    	tent = G_TempEntity( self->client->pcurrentOrigin, EV_PLAYER_TELEPORT );
    */
    //fixme: doesn't actually go away!
    (*self_).nextthink = level.time + 1500;
    (*self_).e_ThinkFunc = thinkF_Disappear;
    (*(*self_).client).playerTeam = TEAM_FREE;
    (*self_).svFlags |= SVF_BEAMING;
}

pub unsafe fn NPC_BSCinematic() {
    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(qtrue);
    }
    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON_NO_ANIM != 0 {
        if TIMER_Done(NPC, b"NoAnimFireDelay\0".as_ptr() as *const c_char) != 0 {
            TIMER_Set(
                NPC,
                b"NoAnimFireDelay\0".as_ptr() as *const c_char,
                NPC_AttackDebounceForWeapon(),
            );
            FireWeapon(NPC, (*NPCInfo).scriptFlags & SCF_ALT_FIRE);
        }
    }

    if UpdateGoal() != 0 {
        //have a goalEntity
        //move toward goal, should also face that goal
        NPC_MoveToGoal(qtrue);
    }

    if !(*NPCInfo).watchTarget.is_null() {
        //have an entity which we want to keep facing
        //NOTE: this will override any angles set by NPC_MoveToGoal
        let mut eyes: vec3_t = [0.0; 3];
        let mut viewSpot: vec3_t = [0.0; 3];
        let mut viewvec: vec3_t = [0.0; 3];
        let mut viewangles: vec3_t = [0.0; 3];

        CalcEntitySpot(NPC, SPOT_HEAD_LEAN, eyes);
        CalcEntitySpot((*NPCInfo).watchTarget, SPOT_HEAD_LEAN, viewSpot);

        VectorSubtract(viewSpot, eyes, viewvec);

        vectoangles(viewvec, viewangles);

        (*NPCInfo).lockedDesiredYaw = viewangles[YAW as usize];
        (*NPCInfo).desiredYaw = viewangles[YAW as usize];
        (*NPCInfo).lockedDesiredPitch = viewangles[PITCH as usize];
        (*NPCInfo).desiredPitch = viewangles[PITCH as usize];
    }

    NPC_UpdateAngles(qtrue, qtrue);
}

pub unsafe fn NPC_BSWait() {
    NPC_UpdateAngles(qtrue, qtrue);
}

pub unsafe fn NPC_BSInvestigate() {
    /*
    	//FIXME: maybe allow this to be set as a tempBState in a script?  Just specify the
    	//investigateGoal, investigateDebounceTime and investigateCount? (Needs a macro)
    	vec3_t		invDir, invAngles, spot;
    	gentity_t	*saveGoal;
    	//BS_INVESTIGATE would turn toward goal, maybe take a couple steps towards it,
    	//look for enemies, then turn away after your investigate counter was down-
    	//investigate counter goes up every time you set it...

    	if(level.time > NPCInfo->enemyCheckDebounceTime)
    	{
    		NPCInfo->enemyCheckDebounceTime = level.time + (NPCInfo->stats.vigilance * 1000);
    		NPC_CheckEnemy(qtrue, qfalse);
    		if(NPC->enemy)
    		{//FIXME: do anger script
    			NPCInfo->goalEntity = NPC->enemy;
    			NPCInfo->behaviorState = BS_RUN_AND_SHOOT;
    			NPCInfo->tempBehavior = BS_DEFAULT;
    			NPC_AngerSound();
    			return;
    		}
    	}

    	NPC_SetAnim( NPC, SETANIM_TORSO, TORSO_WEAPONREADY3, SETANIM_FLAG_NORMAL );

    	if(NPCInfo->stats.vigilance <= 1.0 && NPCInfo->eventOwner)
    	{
    		VectorCopy(NPCInfo->eventOwner->currentOrigin, NPCInfo->investigateGoal);
    	}

    	saveGoal = NPCInfo->goalEntity;
    	if(	level.time > NPCInfo->walkDebounceTime )
    	{
    		vec3_t	vec;

    		VectorSubtract(NPCInfo->investigateGoal, NPC->currentOrigin, vec);
    		vec[2] = 0;
    		if(VectorLength(vec) > 64)
    		{
    			if(Q_irand(0, 100) < NPCInfo->investigateCount)
    			{//take a full step
    				//NPCInfo->walkDebounceTime = level.time + 1400;
    				//actually finds length of my BOTH_WALK anim
    				NPCInfo->walkDebounceTime = PM_AnimLength( NPC->client->clientInfo.animFileIndex, BOTH_WALK1 );
    			}
    		}
    	}

    	if(	level.time < NPCInfo->walkDebounceTime )
    	{//walk toward investigateGoal

    		/*
    		NPCInfo->goalEntity = NPCInfo->tempGoal;
    		VectorCopy(NPCInfo->investigateGoal, NPCInfo->tempGoal->currentOrigin);
    		*/

    /*		NPC_SetMoveGoal( NPC, NPCInfo->investigateGoal, 16, qtrue );

    		NPC_MoveToGoal( qtrue );

    		//FIXME: walk2?
    		NPC_SetAnim(NPC,SETANIM_LEGS,BOTH_WALK1,SETANIM_FLAG_NORMAL);

    		ucmd.buttons |= BUTTON_WALKING;
    	}
    	else
    	{

    		NPC_SetAnim(NPC,SETANIM_LEGS,BOTH_STAND1,SETANIM_FLAG_NORMAL);

    		if(NPCInfo->hlookCount > 30)
    		{
    			if(Q_irand(0, 10) > 7)
    			{
    				NPCInfo->hlookCount = 0;
    			}
    		}
    		else if(NPCInfo->hlookCount < -30)
    		{
    			if(Q_irand(0, 10) > 7)
    			{
    				NPCInfo->hlookCount = 0;
    			}
    		}
    		else if(NPCInfo->hlookCount == 0)
    		{
    			NPCInfo->hlookCount = Q_irand(-1, 1);
    		}
    		else if(Q_irand(0, 10) > 7)
    		{
    			if(NPCInfo->hlookCount > 0)
    			{
    				NPCInfo->hlookCount++;
    			}
    			else//lookCount < 0
    			{
    				NPCInfo->hlookCount--;
    			}
    		}

    		if(NPCInfo->vlookCount >= 15)
    		{
    			if(Q_irand(0, 10) > 7)
    			{
    				NPCInfo->vlookCount = 0;
    			}
    		}
    		else if(NPCInfo->vlookCount <= -15)
    		{
    			if(Q_irand(0, 10) > 7)
    			{
    				NPCInfo->vlookCount = 0;
    			}
    		}
    		else if(NPCInfo->vlookCount == 0)
    		{
    			NPCInfo->vlookCount = Q_irand(-1, 1);
    		}
    		else if(Q_irand(0, 10) > 8)
    		{
    			if(NPCInfo->vlookCount > 0)
    			{
    				NPCInfo->vlookCount++;
    			}
    			else//lookCount < 0
    			{
    				NPCInfo->vlookCount--;
    			}
    		}

    		//turn toward investigateGoal
    		CalcEntitySpot( NPC, SPOT_HEAD, spot );
    		VectorSubtract(NPCInfo->investigateGoal, spot, invDir);
    		VectorNormalize(invDir);
    		vectoangles(invDir, invAngles);
    		NPCInfo->desiredYaw = AngleNormalize360(invAngles[YAW] + NPCInfo->hlookCount);
    		NPCInfo->desiredPitch = AngleNormalize360(invAngles[PITCH] + NPCInfo->hlookCount);
    	}

    	NPC_UpdateAngles(qtrue, qtrue);

    	NPCInfo->goalEntity = saveGoal;

    	if(level.time > NPCInfo->investigateDebounceTime)
    	{
    		NPCInfo->tempBehavior = BS_DEFAULT;
    	}

    	NPC_CheckSoundEvents();
    	*/
    	*/
}

pub unsafe fn NPC_CheckInvestigate(alertEventNum: c_int) -> qboolean {
    let owner: *mut gentity_t = level.alertEvents[alertEventNum as usize].owner;
    let invAdd: c_int = level.alertEvents[alertEventNum as usize].level;
    let mut soundPos: vec3_t = [0.0; 3];
    let soundRad: f32 = level.alertEvents[alertEventNum as usize].radius;
    let earshot: f32 = (*NPCInfo).stats.earshot;

    VectorCopy(level.alertEvents[alertEventNum as usize].position, soundPos);

    //NOTE: Trying to preserve previous investigation behavior
    if owner.is_null() {
        return qfalse;
    }

    if (*owner).s.eType != ET_PLAYER && owner == (*NPCInfo).goalEntity {
        return qfalse;
    }

    if (*owner).s.eFlags & EF_NODRAW != 0 {
        return qfalse;
    }

    if (*owner).flags & FL_NOTARGET != 0 {
        return qfalse;
    }

    if soundRad < earshot {
        return qfalse;
    }

    //if(!gi.inPVSIgnorePortals(ent->currentOrigin, NPC->currentOrigin))//should we be able to hear through areaportals?
    if (gi.inPVS)(soundPos.as_ptr(), (*NPC).currentOrigin.as_ptr()) == 0 {
        //can hear through doors?
        return qfalse;
    }

    if !(*owner).client.is_null()
        && (*(*owner).client).playerTeam != 0
        && (*(*NPC).client).playerTeam != 0
        && (*(*owner).client).playerTeam != (*(*NPC).client).playerTeam
    {
        if (*NPCInfo).investigateCount as f32 >= ((*NPCInfo).stats.vigilance * 200.0)
            && !owner.is_null()
        {
            //If investigateCount == 10, just take it as enemy and go
            if NPC_ValidEnemy(owner) != 0 {
                //FIXME: run angerscript
                G_SetEnemy(NPC, owner);
                (*NPCInfo).goalEntity = (*NPC).enemy;
                (*NPCInfo).goalRadius = 12.0;
                (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
                return qtrue;
            }
        } else {
            (*NPCInfo).investigateCount += invAdd;
        }
        //run awakescript
        G_ActivateBehavior(NPC, BSET_AWAKE);

        /*
        if ( Q_irand(0, 10) > 7 )
        {
        	NPC_AngerSound();
        }
        */

        //NPCInfo->hlookCount = NPCInfo->vlookCount = 0;
        (*NPCInfo).eventOwner = owner;
        VectorCopy(soundPos, (*NPCInfo).investigateGoal);
        if (*NPCInfo).investigateCount > 20 {
            (*NPCInfo).investigateDebounceTime = level.time + 10000;
        } else {
            (*NPCInfo).investigateDebounceTime =
                level.time + ((*NPCInfo).investigateCount * 500);
        }
        (*NPCInfo).tempBehavior = BS_INVESTIGATE;
        return qtrue;
    }

    return qfalse;
}

/*
void NPC_BSSleep( void )
*/
pub unsafe fn NPC_BSSleep() {
    let alertEvent: c_int = NPC_CheckAlertEvents(qtrue, qfalse, -1, qfalse, AEL_DANGER, qfalse);

    //There is an event to look at
    if alertEvent >= 0 {
        G_ActivateBehavior(NPC, BSET_AWAKE);
        return;
    }

    /*
    if ( level.time > NPCInfo->enemyCheckDebounceTime )
    {
    	if ( NPC_CheckSoundEvents() != -1 )
    	{//only 1 alert per second per 0.1 of vigilance
    		NPCInfo->enemyCheckDebounceTime = level.time + (NPCInfo->stats.vigilance * 10000);
    		G_ActivateBehavior(NPC, BSET_AWAKE);
    	}
    }
    */
}

pub unsafe fn NPC_BSFollowLeader_UpdateLeader() -> bool {
    if !(*NPC).client.is_null()
        && !(*(*NPC).client).leader.is_null() //have a leader
        && (*(*(*NPC).client).leader).s.number < MAX_CLIENTS //player
        && !(*(*(*NPC).client).leader).client.is_null() //player is a client
        && (*(*(*(*NPC).client).leader).client).pers.enterTime == 0
    //player has not finished spawning in yet
    {
        //don't do anything just yet, but don't clear the leader either
        return false;
    }

    if !(*(*NPC).client).leader.is_null()
        && (*(*(*NPC).client).leader).health <= 0
    {
        (*(*NPC).client).leader = core::ptr::null_mut();
    }

    if (*(*NPC).client).leader.is_null() {
        //ok, stand guard until we find an enemy
        if (*NPCInfo).tempBehavior == BS_HUNT_AND_KILL {
            (*NPCInfo).tempBehavior = BS_DEFAULT;
        } else {
            (*NPCInfo).tempBehavior = BS_STAND_GUARD;
            NPC_BSStandGuard();
        }
        if (*NPCInfo).behaviorState == BS_FOLLOW_LEADER {
            (*NPCInfo).behaviorState = BS_DEFAULT;
        }
        if (*NPCInfo).defaultBehavior == BS_FOLLOW_LEADER {
            (*NPCInfo).defaultBehavior = BS_DEFAULT;
        }
        return false;
    }
    return true;
}

pub unsafe fn NPC_BSFollowLeader_UpdateEnemy() {
    if (*NPC).enemy.is_null() {
        //no enemy, find one
        NPC_CheckEnemy(
            if (*NPCInfo).confusionTime < level.time { qtrue } else { qfalse },
            qfalse,
        ); //don't find new enemy if this is tempbehav
        if !(*NPC).enemy.is_null() {
            //just found one
            (*NPCInfo).enemyCheckDebounceTime = level.time + Q_irand(3000, 10000);
        } else {
            if (*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS == 0 {
                let eventID: c_int = NPC_CheckAlertEvents(qtrue, qtrue, -1, qfalse, AEL_DANGER, qfalse);
                if eventID > -1
                    && level.alertEvents[eventID as usize].level >= AEL_SUSPICIOUS
                    && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0)
                {
                    //NPCInfo->lastAlertID = level.alertEvents[eventID].ID;
                    if level.alertEvents[eventID as usize].owner.is_null()
                        || (*level.alertEvents[eventID as usize].owner).client.is_null()
                        || (*level.alertEvents[eventID as usize].owner).health <= 0
                        || (*(*level.alertEvents[eventID as usize].owner).client).playerTeam
                            != (*(*NPC).client).enemyTeam
                    {
                        //not an enemy
                    } else {
                        //FIXME: what if can't actually see enemy, don't know where he is... should we make them just become very alert and start looking for him?  Or just let combat AI handle this... (act as if you lost him)
                        G_SetEnemy(NPC, level.alertEvents[eventID as usize].owner);
                        (*NPCInfo).enemyCheckDebounceTime = level.time + Q_irand(3000, 10000);
                        (*NPCInfo).enemyLastSeenTime = level.time;
                        TIMER_Set(
                            NPC,
                            b"attackDelay\0".as_ptr() as *const c_char,
                            Q_irand(500, 1000),
                        );
                    }
                }
            }
        }
        if (*NPC).enemy.is_null() {
            if !(*(*NPC).client).leader.is_null()
                && !(*(*(*NPC).client).leader).enemy.is_null()
                && (*(*(*NPC).client).leader).enemy != NPC
                && ((!(*(*(*(*NPC).client).leader).enemy).client.is_null()
                    && (*(*(*(*NPC).client).leader).enemy).client != core::ptr::null_mut()
                    && (*(*(*(*NPC).client).leader).enemy).client as *mut _ != core::ptr::null_mut()
                    && (*(*(*(*(*NPC).client).leader).enemy).client).playerTeam
                        == (*(*NPC).client).enemyTeam)
                    || ((*(*(*(*NPC).client).leader).enemy).svFlags & SVF_NONNPC_ENEMY != 0
                        && (*(*(*(*NPC).client).leader).enemy).noDamageTeam
                            == (*(*NPC).client).enemyTeam))
                && (*(*(*(*NPC).client).leader).enemy).health > 0
            {
                G_SetEnemy(NPC, (*(*(*NPC).client).leader).enemy);
                (*NPCInfo).enemyCheckDebounceTime = level.time + Q_irand(3000, 10000);
                (*NPCInfo).enemyLastSeenTime = level.time;
            }
        }
    } else {
        if (*(*NPC).enemy).health <= 0
            || ((*(*NPC).enemy).flags & FL_NOTARGET != 0)
        {
            G_ClearEnemy(NPC);
            if (*NPCInfo).enemyCheckDebounceTime > level.time + 1000 {
                (*NPCInfo).enemyCheckDebounceTime = level.time + Q_irand(1000, 2000);
            }
        } else if (*(*NPC).client).ps.weapon != 0
            && (*NPCInfo).enemyCheckDebounceTime < level.time
        {
            NPC_CheckEnemy(
                if (*NPCInfo).confusionTime < level.time
                    || (*NPCInfo).tempBehavior != BS_FOLLOW_LEADER
                {
                    qtrue
                } else {
                    qfalse
                },
                qfalse,
            ); //don't find new enemy if this is tempbehav
        }
    }
}

pub unsafe fn NPC_BSFollowLeader_AttackEnemy() -> bool {
    if (*(*NPC).client).ps.weapon == WP_SABER {
        //|| NPCInfo->confusionTime>level.time )
        //lightsaber user or charmed enemy
        if (*NPCInfo).tempBehavior != BS_FOLLOW_LEADER {
            //not already in a temp bState
            //go after the guy
            (*NPCInfo).tempBehavior = BS_HUNT_AND_KILL;
            NPC_UpdateAngles(qtrue, qtrue);
            return true;
        }
    }

    enemyVisibility = NPC_CheckVisibility((*NPC).enemy, CHECK_FOV | CHECK_SHOOT); //CHECK_360|CHECK_PVS|
    if enemyVisibility > VIS_PVS {
        //face
        let mut enemy_org: vec3_t = [0.0; 3];
        let mut muzzle: vec3_t = [0.0; 3];
        let mut delta: vec3_t = [0.0; 3];
        let mut angleToEnemy: vec3_t = [0.0; 3];
        let mut distanceToEnemy: f32 = 0.0;

        CalcEntitySpot((*NPC).enemy, SPOT_HEAD, enemy_org);
        NPC_AimWiggle(enemy_org);

        CalcEntitySpot(NPC, SPOT_WEAPON, muzzle);

        VectorSubtract(enemy_org, muzzle, delta);
        vectoangles(delta, angleToEnemy);
        distanceToEnemy = VectorNormalize(delta);

        (*NPCInfo).desiredYaw = angleToEnemy[YAW as usize];
        (*NPCInfo).desiredPitch = angleToEnemy[PITCH as usize];
        NPC_UpdateFiringAngles(qtrue, qtrue);

        if enemyVisibility >= VIS_SHOOT {
            //shoot
            NPC_AimAdjust(2);
            if NPC_GetHFOVPercentage(
                (*(*NPC).enemy).currentOrigin,
                (*NPC).currentOrigin,
                (*(*NPC).client).ps.viewangles,
                (*NPCInfo).stats.hfov,
            ) > 0.6_f32
                && NPC_GetHFOVPercentage(
                    (*(*NPC).enemy).currentOrigin,
                    (*NPC).currentOrigin,
                    (*(*NPC).client).ps.viewangles,
                    (*NPCInfo).stats.vfov,
                ) > 0.5_f32
            {
                //actually withing our front cone
                WeaponThink(qtrue);
            }
        } else {
            NPC_AimAdjust(1);
        }

        //NPC_CheckCanAttack(1.0, qfalse);
    } else {
        NPC_AimAdjust(-1);
    }
    return false;
}

pub unsafe fn NPC_BSFollowLeader_CanAttack() -> bool {
    return !(*NPC).enemy.is_null()
        && (*(*NPC).client).ps.weapon != 0
        && ((*NPCInfo).aiFlags & NPCAI_HEAL_ROSH == 0); //Kothos twins never go after their enemy
}

pub unsafe fn NPC_BSFollowLeader_InFullBodyAttack() -> bool {
    return (*(*NPC).client).ps.legsAnim == BOTH_ATTACK1
        || (*(*NPC).client).ps.legsAnim == BOTH_ATTACK2
        || (*(*NPC).client).ps.legsAnim == BOTH_ATTACK3
        || (*(*NPC).client).ps.legsAnim == BOTH_MELEE1
        || (*(*NPC).client).ps.legsAnim == BOTH_MELEE2;
}

pub unsafe fn NPC_BSFollowLeader_LookAtLeader() {
    let mut head: vec3_t = [0.0; 3];
    let mut leaderHead: vec3_t = [0.0; 3];
    let mut delta: vec3_t = [0.0; 3];
    let mut angleToLeader: vec3_t = [0.0; 3];

    CalcEntitySpot((*(*NPC).client).leader, SPOT_HEAD, leaderHead);
    CalcEntitySpot(NPC, SPOT_HEAD, head);
    VectorSubtract(leaderHead, head, delta);
    vectoangles(delta, angleToLeader);
    VectorNormalize(delta);
    (*(*NPC).NPC).desiredYaw = angleToLeader[YAW as usize];
    (*(*NPC).NPC).desiredPitch = angleToLeader[PITCH as usize];

    NPC_UpdateAngles(qtrue, qtrue);
}

pub unsafe fn NPC_BSFollowLeader() {
    // If In A Jump, Return
    //----------------------
    if NPC_Jumping() != 0 {
        return;
    }

    // If There Is No Leader, Return
    //-------------------------------
    if !NPC_BSFollowLeader_UpdateLeader() {
        return;
    }

    // Don't Do Anything Else If In A Full Body Attack
    //-------------------------------------------------
    if NPC_BSFollowLeader_InFullBodyAttack() {
        return;
    }

    // Update The Enemy
    //------------------
    NPC_BSFollowLeader_UpdateEnemy();

    // Do Any Attacking
    //------------------
    if NPC_BSFollowLeader_CanAttack() {
        if NPC_BSFollowLeader_AttackEnemy() {
            return;
        }
    } else {
        NPC_BSFollowLeader_LookAtLeader();
    }

    let followDist: f32 = if (*NPCInfo).followDist != 0.0 {
        (*NPCInfo).followDist
    } else {
        110.0_f32
    };
    let mut moveSuccess: bool = false;

    STEER::Activate(NPC);
    {
        if !(*(*(*NPC).client).leader).client.is_null()
            && (*(*(*(*NPC).client).leader).client).ps.groundEntityNum != ENTITYNUM_NONE
        {
            // If Too Close, Back Away Some
            //------------------------------
            if STEER::Reached(NPC, (*(*NPC).client).leader, 65.0_f32) {
                STEER::Evade(NPC, (*(*NPC).client).leader);
            } else {
                // Attempt To Steer Directly To Our Goal
                //---------------------------------------
                moveSuccess = STEER::GoTo(NPC, (*(*NPC).client).leader, followDist);

                // Perhaps Not Close Enough?  Try To Use The Navigation Grid
                //-----------------------------------------------------------
                if !moveSuccess {
                    moveSuccess = NAV::GoTo(NPC, (*(*NPC).client).leader);
                    if !moveSuccess {
                        STEER::Stop(NPC);
                    }
                }
            }
        } else {
            STEER::Stop(NPC);
        }
    }
    STEER::DeActivate(NPC, addr_of_mut!(ucmd));
}

pub unsafe fn NPC_BSJump() {
    let mut dir: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut p1: vec3_t = [0.0; 3];
    let mut p2: vec3_t = [0.0; 3];
    let mut apex: vec3_t = [0.0; 3];
    let mut time: f32 = 0.0;
    let mut height: f32 = 0.0;
    let mut forward: f32 = 0.0;
    let mut z: f32 = 0.0;
    let mut xy: f32 = 0.0;
    let mut dist: f32 = 0.0;
    let mut yawError: f32 = 0.0;
    let mut apexHeight: f32 = 0.0;

    if (*NPCInfo).goalEntity.is_null() {
        //Should have task completed the navgoal
        return;
    }

    if (*NPCInfo).jumpState != JS_JUMPING && (*NPCInfo).jumpState != JS_LANDING {
        //Face navgoal
        VectorSubtract((*(*NPCInfo).goalEntity).currentOrigin, (*NPC).currentOrigin, dir);
        vectoangles(dir, angles);
        let pitch_val = AngleNormalize360(angles[PITCH as usize]);
        (*NPCInfo).desiredPitch = pitch_val;
        (*NPCInfo).lockedDesiredPitch = pitch_val;
        let yaw_val = AngleNormalize360(angles[YAW as usize]);
        (*NPCInfo).desiredYaw = yaw_val;
        (*NPCInfo).lockedDesiredYaw = yaw_val;
    }

    NPC_UpdateAngles(qtrue, qtrue);
    yawError = AngleDelta((*(*NPC).client).ps.viewangles[YAW as usize], (*NPCInfo).desiredYaw);
    //We don't really care about pitch here

    if (*NPCInfo).jumpState == JS_FACING {
        // case JS_FACING:
        if yawError < MIN_ANGLE_ERROR {
            //Facing it, Start crouching
            NPC_SetAnim(
                NPC,
                SETANIM_LEGS,
                BOTH_CROUCH1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            (*NPCInfo).jumpState = JS_CROUCHING;
        }
    } else if (*NPCInfo).jumpState == JS_CROUCHING {
        // case JS_CROUCHING:
        if (*(*NPC).client).ps.legsAnimTimer > 0 {
            //Still playing crouching anim
            return;
        }

        //Create a parabola

        if (*NPC).currentOrigin[2] > (*(*NPCInfo).goalEntity).currentOrigin[2] {
            VectorCopy((*NPC).currentOrigin, p1);
            VectorCopy((*(*NPCInfo).goalEntity).currentOrigin, p2);
        } else if (*NPC).currentOrigin[2] < (*(*NPCInfo).goalEntity).currentOrigin[2] {
            VectorCopy((*(*NPCInfo).goalEntity).currentOrigin, p1);
            VectorCopy((*NPC).currentOrigin, p2);
        } else {
            VectorCopy((*NPC).currentOrigin, p1);
            VectorCopy((*(*NPCInfo).goalEntity).currentOrigin, p2);
        }

        //z = xy*xy
        VectorSubtract(p2, p1, dir);
        dir[2] = 0.0;

        //Get xy and z diffs
        xy = VectorNormalize(dir);
        z = p1[2] - p2[2];

        apexHeight = APEX_HEIGHT / 2.0;
        /*
        //Determine most desirable apex height
        apexHeight = (APEX_HEIGHT * PARA_WIDTH/xy) + (APEX_HEIGHT * z/128);
        if ( apexHeight < APEX_HEIGHT * 0.5 )
        {
        	apexHeight = APEX_HEIGHT*0.5;
        }
        else if ( apexHeight > APEX_HEIGHT * 2 )
        {
        	apexHeight = APEX_HEIGHT*2;
        }
        */

        //FIXME: length of xy will change curve of parabola, need to account for this
        //somewhere... PARA_WIDTH

        z = (f64::sqrt((apexHeight + z) as f64) - f64::sqrt(apexHeight as f64)) as f32;

        assert!(z >= 0.0_f32);

        //		gi.Printf("apex is %4.2f percent from p1: ", (xy-z)*0.5/xy*100.0f);

        xy -= z;
        xy *= 0.5;

        assert!(xy > 0.0_f32);

        VectorMA(p1, xy, dir, apex);
        apex[2] += apexHeight;

        VectorCopy(apex, (*NPC).pos1);

        //Now we have the apex, aim for it
        height = apex[2] - (*NPC).currentOrigin[2];
        time = f64::sqrt(
            (height / (0.5_f64 * (*(*NPC).client).ps.gravity as f64)) as f64,
        ) as f32;
        if time == 0.0 {
            //			gi.Printf("ERROR no time in jump\n");
            return;
        }

        // set s.origin2 to the push velocity
        VectorSubtract(apex, (*NPC).currentOrigin, (*(*NPC).client).ps.velocity);
        (*(*NPC).client).ps.velocity[2] = 0.0;
        dist = VectorNormalize((*(*NPC).client).ps.velocity);

        forward = dist / time;
        VectorScale(
            (*(*NPC).client).ps.velocity,
            forward,
            (*(*NPC).client).ps.velocity,
        );

        (*(*NPC).client).ps.velocity[2] = time * (*(*NPC).client).ps.gravity;

        //		gi.Printf( "%s jumping %s, gravity at %4.0f percent\n", NPC->targetname, vtos(NPC->client->ps.velocity), NPC->client->ps.gravity/8.0f );

        (*NPCInfo).jumpState = JS_JUMPING;
        //FIXME: jumpsound?
    } else if (*NPCInfo).jumpState == JS_JUMPING {
        // case JS_JUMPING:

        if showBBoxes != 0 {
            VectorAdd((*NPC).mins, (*NPC).pos1, p1);
            VectorAdd((*NPC).maxs, (*NPC).pos1, p2);
            CG_Cube(p1, p2, *addr_of!(NPCDEBUG_BLUE), 0.5);
        }

        if (*NPC).s.groundEntityNum != ENTITYNUM_NONE {
            //Landed, start landing anim
            //FIXME: if the
            VectorClear((*(*NPC).client).ps.velocity);
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_LAND1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            (*NPCInfo).jumpState = JS_LANDING;
            //FIXME: landsound?
        } else if (*(*NPC).client).ps.legsAnimTimer > 0 {
            //Still playing jumping anim
            //FIXME: apply jump velocity here, a couple frames after start, not right away
            return;
        } else {
            //still in air, but done with jump anim, play inair anim
            NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_INAIR1, SETANIM_FLAG_OVERRIDE);
        }
    } else if (*NPCInfo).jumpState == JS_LANDING {
        // case JS_LANDING:
        if (*(*NPC).client).ps.legsAnimTimer > 0 {
            //Still playing landing anim
            return;
        } else {
            (*NPCInfo).jumpState = JS_WAITING;

            (*NPCInfo).goalEntity = UpdateGoal();
            // If he made it to his goal or his task is no longer pending.
            if (*NPCInfo).goalEntity.is_null()
                || Q3_TaskIDPending(NPC, TID_MOVE_NAV) == 0
            {
                NPC_ClearGoal();
                (*NPCInfo).goalTime = level.time;
                (*NPCInfo).aiFlags &= !NPCAI_MOVING;
                ucmd.forwardmove = 0;
                (*NPC).flags &= !FL_NO_KNOCKBACK;
                //Return that the goal was reached
                Q3_TaskIDComplete(NPC, TID_MOVE_NAV);
            }
        }
    } else {
        // case JS_WAITING: / default:
        (*NPCInfo).jumpState = JS_FACING;
    }
}

pub unsafe fn NPC_BSRemove() {
    NPC_UpdateAngles(qtrue, qtrue);
    if (gi.inPVS)((*NPC).currentOrigin.as_ptr(), g_entities[0].currentOrigin.as_ptr()) == 0
    //FIXME: use cg.vieworg?
    {
        G_UseTargets2(NPC, NPC, (*NPC).target3);
        (*NPC).s.eFlags |= EF_NODRAW;
        (*NPC).svFlags &= !SVF_NPC;
        (*NPC).s.eType = ET_INVISIBLE;
        (*NPC).contents = 0;
        (*NPC).health = 0;
        (*NPC).targetname = core::ptr::null_mut();

        //Disappear in half a second
        (*NPC).e_ThinkFunc = thinkF_G_FreeEntity;
        (*NPC).nextthink = level.time + FRAMETIME;
    } //FIXME: else allow for out of FOV???
}

pub unsafe fn NPC_BSSearch() {
    NPC_CheckAlertEvents(qtrue, qtrue, -1, qfalse, AEL_DANGER, qfalse);
    //FIXME: do something with these alerts...?
    //FIXME: do the Stormtrooper alert reaction?  (investigation)
    if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0)
        && (*(*NPC).client).enemyTeam != TEAM_NEUTRAL
    {
        //look for enemies
        NPC_CheckEnemy(qtrue, qfalse);
        if !(*NPC).enemy.is_null() {
            //found one
            if (*NPCInfo).tempBehavior == BS_SEARCH {
                //if tempbehavior, set tempbehavior to default
                (*NPCInfo).tempBehavior = BS_DEFAULT;
            } else {
                //if bState, change to run and shoot
                (*NPCInfo).behaviorState = BS_DEFAULT; //BS_HUNT_AND_KILL;
                //NPC_BSRunAndShoot();
            }
            return;
        }
    }

    //FIXME: what if our goalEntity is not NULL and NOT our tempGoal - they must
    //want us to do something else?  If tempBehavior, just default, else set
    //to run and shoot...?

    //FIXME: Reimplement

    if (*NPCInfo).investigateDebounceTime == 0 {
        //On our way to a tempGoal
        let mut minGoalReachedDistSquared: f32 = 32.0 * 32.0;
        let mut vec: vec3_t = [0.0; 3];

        //Keep moving toward our tempGoal
        (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;

        VectorSubtract(
            (*(*NPCInfo).tempGoal).currentOrigin,
            (*NPC).currentOrigin,
            vec,
        );
        if vec[2] < 24.0 {
            vec[2] = 0.0;
        }

        if (*(*NPCInfo).tempGoal).waypoint != WAYPOINT_NONE {
            /*
            //FIXME: can't get the radius...
            float	wpRadSq = waypoints[NPCInfo->tempGoal->waypoint].radius * waypoints[NPCInfo->tempGoal->waypoint].radius;
            if ( minGoalReachedDistSquared > wpRadSq )
            {
            	minGoalReachedDistSquared = wpRadSq;
            }
            */

            minGoalReachedDistSquared = 32.0 * 32.0; //12*12;
        }

        if VectorLengthSquared(vec) < minGoalReachedDistSquared {
            //Close enough, just got there
            (*NPC).waypoint = NAV::GetNearestNode(NPC);

            if (*NPCInfo).homeWp == WAYPOINT_NONE || (*NPC).waypoint == WAYPOINT_NONE {
                //Heading for or at an invalid waypoint, get out of this bState
                if (*NPCInfo).tempBehavior == BS_SEARCH {
                    //if tempbehavior, set tempbehavior to default
                    (*NPCInfo).tempBehavior = BS_DEFAULT;
                } else {
                    //if bState, change to stand guard
                    (*NPCInfo).behaviorState = BS_STAND_GUARD;
                    NPC_BSRunAndShoot();
                }
                return;
            }

            if (*NPC).waypoint == (*NPCInfo).homeWp {
                //Just Reached our homeWp, if this is the first time, run your lostenemyscript
                if (*NPCInfo).aiFlags & NPCAI_ENROUTE_TO_HOMEWP != 0 {
                    (*NPCInfo).aiFlags &= !NPCAI_ENROUTE_TO_HOMEWP;
                    G_ActivateBehavior(NPC, BSET_LOSTENEMY);
                }
            }

            //gi.Printf("Got there.\n");
            //gi.Printf("Looking...");
            if Q_irand(0, 1) == 0 {
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_GUARD_LOOKAROUND1,
                    SETANIM_FLAG_NORMAL,
                );
            } else {
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_GUARD_IDLE1,
                    SETANIM_FLAG_NORMAL,
                );
            }
            (*NPCInfo).investigateDebounceTime = level.time + Q_irand(3000, 10000);
        } else {
            NPC_MoveToGoal(qtrue);
        }
    } else {
        //We're there
        if (*NPCInfo).investigateDebounceTime > level.time {
            //Still waiting around for a bit
            //Turn angles every now and then to look around
            if (*(*NPCInfo).tempGoal).waypoint != WAYPOINT_NONE {
                if Q_irand(0, 30) == 0 {
                    // NAV_TODO: What if there are no neighbors?
                    let mut branchPos: vec3_t = [0.0; 3];
                    let mut lookDir: vec3_t = [0.0; 3];

                    NAV::GetNodePosition(
                        NAV::ChooseRandomNeighbor((*(*NPCInfo).tempGoal).waypoint),
                        branchPos,
                    );

                    VectorSubtract(branchPos, (*(*NPCInfo).tempGoal).currentOrigin, lookDir);
                    (*NPCInfo).desiredYaw = AngleNormalize360(
                        vectoyaw(lookDir) + Q_flrand(-45.0, 45.0),
                    );
                }
            }
            //gi.Printf(".");
        } else {
            //Just finished waiting
            (*NPC).waypoint = NAV::GetNearestNode(NPC);

            if (*NPC).waypoint == (*NPCInfo).homeWp {
                // NAV_TODO: What if there are no neighbors?

                let nextWp: c_int =
                    NAV::ChooseRandomNeighbor((*(*NPCInfo).tempGoal).waypoint);
                NAV::GetNodePosition(nextWp, (*(*NPCInfo).tempGoal).currentOrigin);
                (*(*NPCInfo).tempGoal).waypoint = nextWp;
            } else {
                //At a branch, so return home
                NAV::GetNodePosition((*NPCInfo).homeWp, (*(*NPCInfo).tempGoal).currentOrigin);
                (*(*NPCInfo).tempGoal).waypoint = (*NPCInfo).homeWp;
            }

            (*NPCInfo).investigateDebounceTime = 0;
            //Start moving toward our tempGoal
            (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;
            NPC_MoveToGoal(qtrue);
        }
    }

    NPC_UpdateAngles(qtrue, qtrue);
}

/*
-------------------------
NPC_BSSearchStart
-------------------------
*/

pub unsafe fn NPC_BSSearchStart(homeWp: c_int, bState: bState_t) {
    //FIXME: Reimplement
    (*NPCInfo).homeWp = homeWp;
    (*NPCInfo).tempBehavior = bState;
    (*NPCInfo).aiFlags |= NPCAI_ENROUTE_TO_HOMEWP;
    (*NPCInfo).investigateDebounceTime = 0;
    NAV::GetNodePosition(homeWp, (*(*NPCInfo).tempGoal).currentOrigin);
    (*(*NPCInfo).tempGoal).waypoint = homeWp;
    //gi.Printf("\nHeading for wp %d...\n", NPCInfo->homeWp);
}

/*
-------------------------
NPC_BSNoClip

  Use in extreme circumstances only
-------------------------
*/

pub unsafe fn NPC_BSNoClip() {
    if UpdateGoal() != 0 {
        let mut dir: vec3_t = [0.0; 3];
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0; 3];
        let up: vec3_t = [0.0, 0.0, 1.0];
        let mut fDot: f32;
        let mut rDot: f32;
        let mut uDot: f32;

        VectorSubtract((*(*NPCInfo).goalEntity).currentOrigin, (*NPC).currentOrigin, dir);

        vectoangles(dir, angles);
        (*NPCInfo).desiredYaw = angles[YAW as usize];

        AngleVectors((*NPC).currentAngles, forward, right, core::ptr::null_mut());

        VectorNormalize(dir);

        fDot = DotProduct(forward, dir) * 127.0;
        rDot = DotProduct(right, dir) * 127.0;
        uDot = DotProduct(up, dir) * 127.0;

        ucmd.forwardmove = (fDot as f64).floor() as _;
        ucmd.rightmove = (rDot as f64).floor() as _;
        ucmd.upmove = (uDot as f64).floor() as _;
    } else {
        //Cut velocity?
        VectorClear((*(*NPC).client).ps.velocity);
    }

    NPC_UpdateAngles(qtrue, qtrue);
}

pub unsafe fn NPC_BSWander() {
    //FIXME: don't actually go all the way to the next waypoint, just move in fits and jerks...?

    NPC_CheckAlertEvents(qtrue, qtrue, -1, qfalse, AEL_DANGER, qfalse);
    //FIXME: do something with these alerts...?
    //FIXME: do the Stormtrooper alert reaction?  (investigation)
    if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0)
        && (*(*NPC).client).enemyTeam != TEAM_NEUTRAL
    {
        //look for enemies
        NPC_CheckEnemy(qtrue, qfalse);
        if !(*NPC).enemy.is_null() {
            //found one
            if (*NPCInfo).tempBehavior == BS_WANDER {
                //if tempbehavior, set tempbehavior to default
                (*NPCInfo).tempBehavior = BS_DEFAULT;
            } else {
                //if bState, change to run and shoot
                (*NPCInfo).behaviorState = BS_DEFAULT; //BS_HUNT_AND_KILL;
                //NPC_BSRunAndShoot();
            }
            return;
        }
    }

    STEER::Activate(NPC);

    // Are We Doing A Path?
    //----------------------
    let mut HasPath: bool = NAV::HasPath(NPC);
    if HasPath {
        HasPath = NAV::UpdatePath(NPC);
        if HasPath {
            STEER::Path(NPC); // Follow The Path
            STEER::AvoidCollisions(NPC);

            if ((*NPCInfo).aiFlags & NPCAI_BLOCKED != 0)
                && (level.time - (*NPCInfo).blockedDebounceTime) > 1000
            {
                HasPath = false; // find a new one
            }
        }
    }

    if !HasPath {
        // If Debounce Time Has Expired, Choose A New Sub State
        //------------------------------------------------------
        if (*NPCInfo).investigateDebounceTime < level.time
            || (((*NPCInfo).aiFlags & NPCAI_BLOCKED != 0)
                && (level.time - (*NPCInfo).blockedDebounceTime) > 1000)
        {
            // Clear Out Flags From The Previous Substate
            //--------------------------------------------
            (*NPCInfo).aiFlags &= !NPCAI_OFF_PATH;
            (*NPCInfo).aiFlags &= !NPCAI_WALKING;

            // Pick Another Spot
            //-------------------
            let NEXTSUBSTATE: c_int = Q_irand(0, 10);

            let RandomPathNode: bool = NEXTSUBSTATE < 9; //(NEXTSUBSTATE<4);
            let PathlessWander: bool = false; //(NEXTSUBSTATE<9)

            // Random Path Node
            //------------------
            if RandomPathNode {
                // Sometimes, Walk
                //-----------------
                if Q_irand(0, 1) == 0 {
                    (*NPCInfo).aiFlags |= NPCAI_WALKING;
                }

                (*NPCInfo).investigateDebounceTime = level.time + Q_irand(3000, 10000);
                NAV::FindPath(NPC, NAV::ChooseRandomNeighbor(NAV::GetNearestNode(NPC)));
            }
            // Pathless Wandering
            //--------------------
            else if PathlessWander {
                // Sometimes, Walk
                //-----------------
                if Q_irand(0, 1) == 0 {
                    (*NPCInfo).aiFlags |= NPCAI_WALKING;
                }

                (*NPCInfo).investigateDebounceTime = level.time + Q_irand(3000, 10000);
                (*NPCInfo).aiFlags |= NPCAI_OFF_PATH;
            }
            // Just Stand Here
            //-----------------
            else {
                (*NPCInfo).investigateDebounceTime = level.time + Q_irand(2000, 10000);
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    if Q_irand(0, 1) == 0 {
                        BOTH_GUARD_LOOKAROUND1
                    } else {
                        BOTH_GUARD_IDLE1
                    },
                    SETANIM_FLAG_NORMAL,
                );
            }
        }
        // Ok, So We Don't Have A Path, And Debounce Time Is Still Active, So We Are Either Wandering Or Looking Around
        //--------------------------------------------------------------------------------------------------------------
        else {
            if (*NPCInfo).aiFlags & NPCAI_OFF_PATH != 0 {
                STEER::Wander(NPC);
                STEER::AvoidCollisions(NPC);
            } else {
                STEER::Stop(NPC);
            }
        }
    }
    STEER::DeActivate(NPC, addr_of_mut!(ucmd));

    NPC_UpdateAngles(qtrue, qtrue);
    return;
}

/*
void NPC_BSFaceLeader (void)
{
	vec3_t	head, leaderHead, delta, angleToLeader;

	if ( !NPC->client->leader )
	{//uh.... okay.
		return;
	}

	CalcEntitySpot( NPC->client->leader, SPOT_HEAD, leaderHead );
	CalcEntitySpot( NPC, SPOT_HEAD, head );
	VectorSubtract( leaderHead, head, delta );
	vectoangles( delta, angleToLeader );
	VectorNormalize( delta );
	NPC->NPC->desiredYaw = angleToLeader[YAW];
	NPC->NPC->desiredPitch = angleToLeader[PITCH];

	NPC_UpdateAngles(qtrue, qtrue);
}
*/
/*
-------------------------
NPC_BSFlee
-------------------------
*/
pub unsafe fn NPC_CanSurrender() -> qboolean {
    if !(*NPC).client.is_null() {
        match (*(*NPC).client).NPC_class {
            x if x == CLASS_ATST
                || x == CLASS_CLAW
                || x == CLASS_DESANN
                || x == CLASS_FISH
                || x == CLASS_FLIER2
                || x == CLASS_GALAK
                || x == CLASS_GLIDER
                || x == CLASS_GONK // droid
                || x == CLASS_HOWLER
                || x == CLASS_RANCOR
                || x == CLASS_SAND_CREATURE
                || x == CLASS_WAMPA
                || x == CLASS_INTERROGATOR // droid
                || x == CLASS_JAN
                || x == CLASS_JEDI
                || x == CLASS_KYLE
                || x == CLASS_LANDO
                || x == CLASS_LIZARD
                || x == CLASS_LUKE
                || x == CLASS_MARK1 // droid
                || x == CLASS_MARK2 // droid
                || x == CLASS_GALAKMECH // droid
                || x == CLASS_MINEMONSTER
                || x == CLASS_MONMOTHA
                || x == CLASS_MORGANKATARN
                || x == CLASS_MOUSE // droid
                || x == CLASS_MURJJ
                || x == CLASS_PROBE // droid
                || x == CLASS_PROTOCOL // droid
                || x == CLASS_R2D2 // droid
                || x == CLASS_R5D2 // droid
                || x == CLASS_REBORN
                || x == CLASS_REELO
                || x == CLASS_REMOTE
                || x == CLASS_SEEKER // droid
                || x == CLASS_SENTRY
                || x == CLASS_SHADOWTROOPER
                || x == CLASS_SWAMP
                || x == CLASS_TAVION
                || x == CLASS_ALORA
                || x == CLASS_TUSKEN
                || x == CLASS_BOBAFETT
                || x == CLASS_ROCKETTROOPER
                || x == CLASS_SABER_DROID
                || x == CLASS_ASSASSIN_DROID
                || x == CLASS_HAZARD_TROOPER
                || x == CLASS_PLAYER
                || x == CLASS_VEHICLE =>
            {
                return qfalse;
            }
            _ => {}
        }
        if G_StandardHumanoid(NPC) == 0 {
            return qfalse;
        }
        if (*(*NPC).client).ps.weapon == WP_SABER {
            return qfalse;
        }
    }
    if !NPCInfo.is_null() {
        if (*NPCInfo).aiFlags & NPCAI_BOSS_CHARACTER != 0 {
            return qfalse;
        }
        if (*NPCInfo).aiFlags & NPCAI_SUBBOSS_CHARACTER != 0 {
            return qfalse;
        }
        if (*NPCInfo).aiFlags & NPCAI_ROSH != 0 {
            return qfalse;
        }
        if (*NPCInfo).aiFlags & NPCAI_HEAL_ROSH != 0 {
            return qfalse;
        }
    }
    return qtrue;
}

pub unsafe fn NPC_Surrender() {
    //FIXME: say "don't shoot!" if we weren't already surrendering
    if (*(*NPC).client).ps.weaponTime != 0
        || PM_InKnockDown(addr_of_mut!((*(*NPC).client).ps)) != 0
    {
        return;
    }
    if NPC_CanSurrender() == 0 {
        return;
    }
    if (*NPC).s.weapon != WP_NONE
        && (*NPC).s.weapon != WP_MELEE
        && (*NPC).s.weapon != WP_SABER
    {
        WP_DropWeapon(NPC, core::ptr::null_mut());
    }
    if (*NPCInfo).surrenderTime < level.time - 5000 {
        //haven't surrendered for at least 6 seconds, tell them what you're doing
        //FIXME: need real dialogue EV_SURRENDER
        (*NPCInfo).blockedSpeechDebounceTime = 0; //make sure we say this
        G_AddVoiceEvent(NPC, Q_irand(EV_PUSHED1, EV_PUSHED3), 3000);
    }

    // Already Surrendering?  If So, Just Update Animations
    //------------------------------------------------------
    if (*NPCInfo).surrenderTime > level.time {
        if (*(*NPC).client).ps.torsoAnim == BOTH_COWER1_START
            && (*(*NPC).client).ps.torsoAnimTimer <= 100
        {
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_COWER1,
                SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
            );
            (*NPCInfo).surrenderTime = level.time + (*(*NPC).client).ps.torsoAnimTimer;
        }
        if (*(*NPC).client).ps.torsoAnim == BOTH_COWER1
            && (*(*NPC).client).ps.torsoAnimTimer <= 100
        {
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_COWER1_STOP,
                SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
            );
            (*NPCInfo).surrenderTime = level.time + (*(*NPC).client).ps.torsoAnimTimer;
        }
    }
    // New To The Surrender, So Start The Animation
    //----------------------------------------------
    else {
        if (*(*NPC).client).NPC_class == CLASS_JAWA
            && (*(*NPC).client).ps.weapon == WP_NONE
        {
            //an unarmed Jawa is very scared
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_COWER1,
                SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
            );
            //FIXME: stop doing this if decide to take off and run
        } else {
            // A Big Monster?  OR: Being Tracked By A Homing Rocket?  So Do The Cower Sequence
            //------------------------------------------
            if (!(*NPC).enemy.is_null()
                && !(*(*NPC).enemy).client.is_null()
                && (*(*(*NPC).enemy).client).NPC_class == CLASS_RANCOR)
                || TIMER_Done(NPC, b"rocketChasing\0".as_ptr() as *const c_char) == 0
            {
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_COWER1_START,
                    SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
                );
            }
            // Otherwise, Use The Old Surrender "Arms In Air" Animation
            //----------------------------------------------------------
            else {
                NPC_SetAnim(
                    NPC,
                    SETANIM_TORSO,
                    TORSO_SURRENDER_START,
                    SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
                );
                (*(*NPC).client).ps.torsoAnimTimer = Q_irand(3000, 8000); // Pretend the anim lasts longer
            }
        }
        (*NPCInfo).surrenderTime = level.time + (*(*NPC).client).ps.torsoAnimTimer + 1000;
    }
}

pub unsafe fn NPC_CheckSurrender() -> qboolean {
    if (*g_AIsurrender).integer == 0
        && (*(*NPC).client).NPC_class != CLASS_UGNAUGHT
        && (*(*NPC).client).NPC_class != CLASS_JAWA
    {
        //not enabled
        return qfalse;
    }
    if Q3_TaskIDPending(NPC, TID_MOVE_NAV) == 0 //not scripted to go somewhere
        && (*(*NPC).client).ps.groundEntityNum != ENTITYNUM_NONE //not in the air
        && (*(*NPC).client).ps.weaponTime == 0
        && PM_InKnockDown(addr_of_mut!((*(*NPC).client).ps)) == 0 //not firing and not on the ground
        && !(*NPC).enemy.is_null()
        && !(*(*NPC).enemy).client.is_null()
        && (*(*NPC).enemy).enemy == NPC
        && (*(*NPC).enemy).s.weapon != WP_NONE
        && ((*(*NPC).enemy).s.weapon != WP_MELEE
            || ((*(*(*NPC).enemy).client).NPC_class == CLASS_RANCOR
                || (*(*(*NPC).enemy).client).NPC_class == CLASS_WAMPA)) //enemy is using a weapon or is a Rancor or Wampa
        && (*(*NPC).enemy).health > 20
        && (*(*NPC).enemy).painDebounceTime < level.time - 3000
        && (*(*(*NPC).enemy).client).ps.forcePowerDebounce[FP_SABER_DEFENSE as usize]
            < level.time - 1000
    {
        //don't surrender if scripted to run somewhere or if we're in the air or if we're busy or if we don't have an enemy or if the enemy is not mad at me or is hurt or not a threat or busy being attacked
        //FIXME: even if not in a group, don't surrender if there are other enemies in the PVS and within a certain range?
        if (*NPC).s.weapon != WP_ROCKET_LAUNCHER
            && (*NPC).s.weapon != WP_CONCUSSION
            && (*NPC).s.weapon != WP_REPEATER
            && (*NPC).s.weapon != WP_FLECHETTE
            && (*NPC).s.weapon != WP_SABER
        {
            //jedi and heavy weapons guys never surrender
            //FIXME: rework all this logic into some orderly fashion!!!
            if (*NPC).s.weapon != WP_NONE {
                //they have a weapon so they'd have to drop it to surrender
                //don't give up unless low on health
                if (*NPC).health > 25 || (*NPC).health >= (*NPC).max_health {
                    return qfalse;
                }
                if g_crosshairEntNum == (*NPC).s.number
                    && (*NPC).painDebounceTime > level.time
                {
                    //if he just shot me, always give up
                    //fall through
                } else {
                    //don't give up unless facing enemy and he's very close
                    if InFOV(player, NPC, 60, 30) == 0 {
                        //I'm not looking at them
                        return qfalse;
                    } else if DistanceSquared((*NPC).currentOrigin, (*player).currentOrigin)
                        < 65536 /*256*256*/
                    {
                        //they're not close
                        return qfalse;
                    } else if (gi.inPVS)(
                        (*NPC).currentOrigin.as_ptr(),
                        (*player).currentOrigin.as_ptr(),
                    ) == 0
                    {
                        //they're not in the same room
                        return qfalse;
                    }
                }
            }
            if (*NPCInfo).group.is_null()
                || (!(*NPCInfo).group.is_null()
                    && (*(*NPCInfo).group).numGroup <= 1)
            {
                //I'm alone but I was in a group//FIXME: surrender anyway if just melee or no weap?
                if (*NPC).s.weapon == WP_NONE
                    //NPC has a weapon
                    || (*NPC).enemy == player
                    || ((*(*NPC).enemy).s.weapon == WP_SABER
                        && !(*(*NPC).enemy).client.is_null()
                        && (*(*(*NPC).enemy).client).ps.SaberActive())
                    || (!(*(*NPC).enemy).NPC.is_null()
                        && !(*(*(*NPC).enemy).NPC).group.is_null()
                        && (*(*(*(*NPC).enemy).NPC).group).numGroup > 2)
                {
                    //surrender only if have no weapon or fighting a player or jedi or if we are outnumbered at least 3 to 1
                    if (*NPC).enemy == player {
                        //player is the guy I'm running from
                        if g_crosshairEntNum == (*NPC).s.number {
                            //give up if player is aiming at me
                            NPC_Surrender();
                            NPC_UpdateAngles(qtrue, qtrue);
                            return qtrue;
                        } else if (*player).s.weapon == WP_SABER {
                            //player is using saber
                            if InFOV(NPC, player, 60, 30) != 0 {
                                //they're looking at me
                                if DistanceSquared(
                                    (*NPC).currentOrigin,
                                    (*player).currentOrigin,
                                ) < 16384 /*128*128*/
                                {
                                    //they're close
                                    if (gi.inPVS)(
                                        (*NPC).currentOrigin.as_ptr(),
                                        (*player).currentOrigin.as_ptr(),
                                    ) != 0
                                    {
                                        //they're in the same room
                                        NPC_Surrender();
                                        NPC_UpdateAngles(qtrue, qtrue);
                                        return qtrue;
                                    }
                                }
                            }
                        }
                    } else if !(*NPC).enemy.is_null() {
                        //???
                        //should NPC's surrender to others?
                        if InFOV(NPC, (*NPC).enemy, 30, 30) != 0 {
                            //they're looking at me
                            let maxDist: f32 = 64.0
                                + ((*NPC).maxs[0] * 1.5)
                                + ((*(*NPC).enemy).maxs[0] * 1.5);
                            let maxDist = maxDist * maxDist;
                            if DistanceSquared(
                                (*NPC).currentOrigin,
                                (*(*NPC).enemy).currentOrigin,
                            ) < maxDist
                            {
                                //they're close
                                if (gi.inPVS)(
                                    (*NPC).currentOrigin.as_ptr(),
                                    (*(*NPC).enemy).currentOrigin.as_ptr(),
                                ) != 0
                                {
                                    //they're in the same room
                                    //FIXME: should player-team NPCs not fire on surrendered NPCs?
                                    NPC_Surrender();
                                    NPC_UpdateAngles(qtrue, qtrue);
                                    return qtrue;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return qfalse;
}

pub unsafe fn NPC_JawaFleeSound() {
    if !NPC.is_null()
        && !(*NPC).client.is_null()
        && (*(*NPC).client).NPC_class == CLASS_JAWA
        && Q_irand(0, 3) == 0
        && (*NPCInfo).blockedSpeechDebounceTime < level.time
        && Q3_TaskIDPending(NPC, TID_CHAN_VOICE) == 0
    {
        //ooteenee!!!!
        //Com_Printf( "ooteenee!!!!\n" );
        G_SoundOnEnt(
            NPC,
            CHAN_VOICE,
            b"sound/chars/jawa/misc/ooh-tee-nee.wav\0".as_ptr() as *const c_char,
        );
        (*NPCInfo).blockedSpeechDebounceTime = level.time + 2000;
    }
}

pub unsafe fn NPC_BSFlee() -> qboolean {
    let mut enemyRecentlySeen: bool = false;
    let mut enemyTooCloseDist: f32 = 50.0;
    let mut reachedEscapePoint: bool = false;
    let mut hasEscapePoint: bool = false;
    let mut moveSuccess: bool = false;
    let inSurrender: bool = level.time < (*NPCInfo).surrenderTime;

    // Check For Enemies And Alert Events
    //------------------------------------
    NPC_CheckEnemy(qtrue, qfalse);
    NPC_CheckAlertEvents(qtrue, qtrue, -1, qfalse, AEL_DANGER, qfalse);
    if !(*NPC).enemy.is_null() && G_ClearLOS(NPC, (*NPC).enemy) != 0 {
        (*NPCInfo).enemyLastSeenTime = level.time;
    }
    enemyRecentlySeen =
        !(*NPC).enemy.is_null() && (level.time - (*NPCInfo).enemyLastSeenTime) < 3000;
    if enemyRecentlySeen {
        if !(*(*NPC).enemy).client.is_null()
            && (*(*(*NPC).enemy).client).NPC_class == CLASS_RANCOR
        {
            enemyTooCloseDist = 400.0;
        }
        enemyTooCloseDist += (*NPC).maxs[0] + (*(*NPC).enemy).maxs[0];
    }

    // Look For Weapons To Pick Up
    //-----------------------------
    if enemyRecentlySeen // Is There An Enemy Near?
        && (*(*NPC).client).NPC_class != CLASS_PRISONER // Prisoners can't pickup weapons
        && (*NPCInfo).rank > RANK_CIVILIAN // Neither can civilians
        && TIMER_Done(NPC, b"panic\0".as_ptr() as *const c_char) != 0 // Panic causes him to run for a bit, don't pickup weapons
        && TIMER_Done(NPC, b"CheckForWeaponToPickup\0".as_ptr() as *const c_char) != 0
        && G_CanPickUpWeapons(NPC) != 0 //Allowed To Pick Up Dropped Weapons
    {
        let foundWeap: *mut gentity_t = NPC_SearchForWeapons();

        // Ok, There Is A Weapon!  Try Going To It!
        //------------------------------------------
        if !foundWeap.is_null()
            && NAV::SafePathExists(
                (*NPC).currentOrigin,
                (*foundWeap).currentOrigin,
                (*(*NPC).enemy).currentOrigin,
                150.0,
            )
        {
            NAV::ClearPath(NPC); // Remove Any Old Path

            (*NPCInfo).goalEntity = foundWeap; // Change Our Target Goal
            (*NPCInfo).goalRadius = 30.0; // 30 good enough?

            TIMER_Set(
                NPC,
                b"CheckForWeaponToPickup\0".as_ptr() as *const c_char,
                Q_irand(10000, 50000),
            );
        }
        // Look Again Soon
        //-----------------
        else {
            TIMER_Set(
                NPC,
                b"CheckForWeaponToPickup\0".as_ptr() as *const c_char,
                Q_irand(1000, 5000),
            );
        }
    }

    // If Attempting To Get To An Entity That Is Gone, Clear The Pointer
    //-------------------------------------------------------------------
    if !(*NPCInfo).goalEntity.is_null()
        && Q3_TaskIDPending(NPC, TID_MOVE_NAV) == 0
        && !(*NPC).enemy.is_null()
        && Distance(
            (*(*NPCInfo).goalEntity).currentOrigin,
            (*(*NPC).enemy).currentOrigin,
        ) < enemyTooCloseDist
    {
        //our goal is too close to our enemy, dump it...
        (*NPCInfo).goalEntity = core::ptr::null_mut();
    }
    if !(*NPCInfo).goalEntity.is_null() && (*(*NPCInfo).goalEntity).inuse == 0 {
        (*NPCInfo).goalEntity = core::ptr::null_mut();
    }
    hasEscapePoint = !(*NPCInfo).goalEntity.is_null() && (*NPCInfo).goalRadius != 0.0;

    STEER::Activate(NPC);
    {
        // Have We Reached The Escape Point?
        //-----------------------------------
        if hasEscapePoint
            && STEER::Reached(NPC, (*NPCInfo).goalEntity, (*NPCInfo).goalRadius, false)
        {
            if Q3_TaskIDPending(NPC, TID_MOVE_NAV) != 0 {
                Q3_TaskIDComplete(NPC, TID_MOVE_NAV);
            }
            reachedEscapePoint = true;
        }

        // If Super Close To The Enemy, Run In The Other Direction
        //---------------------------------------------------------
        if enemyRecentlySeen
            && Distance((*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin)
                < enemyTooCloseDist
        {
            STEER::Evade(NPC, (*NPC).enemy);
            STEER::AvoidCollisions(NPC);
        }
        // If Already At The Escape Point, Or Surrendering, Don't Move
        //-------------------------------------------------------------
        else if reachedEscapePoint || inSurrender {
            STEER::Stop(NPC);
        } else {
            // Try To Get To The Escape Point
            //--------------------------------
            if hasEscapePoint {
                moveSuccess = STEER::GoTo(NPC, (*NPCInfo).goalEntity, true);
                if !moveSuccess {
                    moveSuccess = NAV::GoTo(NPC, (*NPCInfo).goalEntity, 0.3);
                }
            }

            // Cant Get To The Escape Point, So If There Is An Enemy
            //-------------------------------------------------------
            if !moveSuccess && enemyRecentlySeen {
                // Try To Get To The Farthest Combat Point From Him
                //--------------------------------------------------
                let Nbr: NAV::TNodeHandle =
                    NAV::ChooseFarthestNeighbor(NPC, (*(*NPC).enemy).currentOrigin, 0.25);
                if Nbr > 0 {
                    moveSuccess = STEER::GoTo(NPC, NAV::GetNodePosition(Nbr), true);
                    if !moveSuccess {
                        moveSuccess = NAV::GoTo(NPC, Nbr, 0.3);
                    }
                }
            }

            // If We Still Can't (Or Don't Need To) Move, Just Stop
            //------------------------------------------------------
            if !moveSuccess {
                STEER::Stop(NPC);
            }
        }
    }
    STEER::DeActivate(NPC, addr_of_mut!(ucmd));

    // Is There An Enemy Around?
    //---------------------------
    if enemyRecentlySeen {
        // Time To Surrender?
        //--------------------
        if TIMER_Done(NPC, b"panic\0".as_ptr() as *const c_char) != 0 {
            //done panicking, time to realize we're dogmeat, if we haven't been able to flee for a few seconds
            if (level.time - (*NPC).lastMoveTime) > 3000
                && (level.time - (*NPCInfo).surrenderTime) > 3000
            //and haven't just finished surrendering
            {
                NPC_FaceEnemy();
                NPC_Surrender();
            }
        }

        // Time To Choose A New Escape Point?
        //------------------------------------
        if (!hasEscapePoint || reachedEscapePoint)
            && TIMER_Done(NPC, b"FindNewEscapePointDebounce\0".as_ptr() as *const c_char) != 0
        {
            TIMER_Set(
                NPC,
                b"FindNewEscapePointDebounce\0".as_ptr() as *const c_char,
                2500,
            );

            let escapePoint: c_int = NPC_FindCombatPoint(
                (*NPC).currentOrigin,
                (*(*NPC).enemy).currentOrigin,
                (*NPC).currentOrigin,
                CP_COVER | CP_AVOID_ENEMY | CP_HAS_ROUTE,
                128,
            );
            if escapePoint != -1 {
                NPC_JawaFleeSound();
                NPC_SetCombatPoint(escapePoint);
                NPC_SetMoveGoal(
                    NPC,
                    level.combatPoints[escapePoint as usize].origin,
                    8,
                    qtrue,
                    escapePoint,
                );
            }
        }
    }

    // If Only Temporarly In Flee, Think About Perhaps Returning To Combat
    //---------------------------------------------------------------------
    if (*NPCInfo).tempBehavior == BS_FLEE
        && TIMER_Done(NPC, b"flee\0".as_ptr() as *const c_char) != 0
        && (*NPC).s.weapon != WP_NONE
        && (*NPC).s.weapon != WP_MELEE
    {
        (*NPCInfo).tempBehavior = BS_DEFAULT;
    }

    // Always Update Angles
    //----------------------
    NPC_UpdateAngles(qtrue, qtrue);
    if reachedEscapePoint {
        return qtrue;
    }
    return qfalse;
}

pub unsafe fn NPC_StartFlee(
    enemy: *mut gentity_t,
    dangerPoint: vec3_t,
    dangerLevel: c_int,
    fleeTimeMin: c_int,
    fleeTimeMax: c_int,
) {
    if Q3_TaskIDPending(NPC, TID_MOVE_NAV) != 0 {
        //running somewhere that a script requires us to go, don't interrupt that!
        return;
    }

    if (*NPCInfo).scriptFlags & SCF_DONT_FLEE != 0 {
        // no flee for you
        return;
    }

    //if have a fleescript, run that instead
    if G_ActivateBehavior(NPC, BSET_FLEE) != 0 {
        return;
    }

    //FIXME: play a flee sound?  Appropriate to situation?
    if !enemy.is_null() {
        NPC_JawaFleeSound();
        G_SetEnemy(NPC, enemy);
    }

    //FIXME: if don't have a weapon, find nearest one we have a route to and run for it?
    let mut cp: c_int = -1;
    if dangerLevel > AEL_DANGER
        || (*NPC).s.weapon == WP_NONE
        || ((*NPCInfo).group.is_null()
            || (*(*NPCInfo).group).numGroup <= 1)
            && (*NPC).health <= 10
    {
        //IF either great danger OR I have no weapon OR I'm alone and low on health, THEN try to find a combat point out of PVS
        cp = NPC_FindCombatPoint(
            (*NPC).currentOrigin,
            dangerPoint,
            (*NPC).currentOrigin,
            CP_COVER | CP_AVOID | CP_HAS_ROUTE | CP_NO_PVS,
            128,
        );
    }
    //FIXME: still happens too often...
    if cp == -1 {
        //okay give up on the no PVS thing
        cp = NPC_FindCombatPoint(
            (*NPC).currentOrigin,
            dangerPoint,
            (*NPC).currentOrigin,
            CP_COVER | CP_AVOID | CP_HAS_ROUTE,
            128,
        );
        if cp == -1 {
            //okay give up on the avoid
            cp = NPC_FindCombatPoint(
                (*NPC).currentOrigin,
                dangerPoint,
                (*NPC).currentOrigin,
                CP_COVER | CP_HAS_ROUTE,
                128,
            );
            if cp == -1 {
                //okay give up on the cover
                cp = NPC_FindCombatPoint(
                    (*NPC).currentOrigin,
                    dangerPoint,
                    (*NPC).currentOrigin,
                    CP_HAS_ROUTE,
                    128,
                );
            }
        }
    }

    //see if we got a valid one
    if cp != -1 {
        //found a combat point
        NPC_SetCombatPoint(cp);
        NPC_SetMoveGoal(NPC, level.combatPoints[cp as usize].origin, 8, qtrue, cp);
    } else {
        //couldn't find a place to hide
        //FIXME: re-implement the old BS_FLEE behavior of following any the waypoint edge
        //			that leads away from the danger point.
        NPC_SetMoveGoal(NPC, (*NPC).currentOrigin, 0 /*goalRadius*/, qtrue, cp);
    }

    if dangerLevel > AEL_DANGER //geat danger always makes people turn and run
        || (*NPC).s.weapon == WP_NONE //melee/unarmed guys turn and run, others keep facing you and shooting
        || (*NPC).s.weapon == WP_MELEE
        || (*NPC).s.weapon == WP_TUSKEN_STAFF
    {
        (*NPCInfo).tempBehavior = BS_FLEE; //we don't want to do this forever!
        //FIXME: only make it temp if you have a weapon?  Otherwise, permanent?
        //	NPCInfo->behaviorState = BS_FLEE;
        //	NPCInfo->tempBehavior = BS_DEFAULT;
    }

    //FIXME: localize this Timer?
    TIMER_Set(
        NPC,
        b"attackDelay\0".as_ptr() as *const c_char,
        Q_irand(500, 2500),
    );
    //FIXME: is this always applicable?
    (*NPCInfo).squadState = SQUAD_RETREAT;
    TIMER_Set(
        NPC,
        b"flee\0".as_ptr() as *const c_char,
        Q_irand(fleeTimeMin, fleeTimeMax),
    );
    TIMER_Set(
        NPC,
        b"panic\0".as_ptr() as *const c_char,
        Q_irand(1000, 4000),
    ); //how long to wait before trying to nav to a dropped weapon
    TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, 0);
}

pub unsafe fn G_StartFlee(
    self_: *mut gentity_t,
    enemy: *mut gentity_t,
    dangerPoint: vec3_t,
    dangerLevel: c_int,
    fleeTimeMin: c_int,
    fleeTimeMax: c_int,
) {
    if (*self_).NPC.is_null() {
        //player
        return;
    }
    SaveNPCGlobals();
    SetNPCGlobals(self_);

    NPC_StartFlee(enemy, dangerPoint, dangerLevel, fleeTimeMin, fleeTimeMax);

    RestoreNPCGlobals();
}

pub unsafe fn NPC_BSEmplaced() {
    //Don't do anything if we're hurt
    if (*NPC).painDebounceTime > level.time {
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(qtrue);
    }

    //If we don't have an enemy, just idle
    if NPC_CheckEnemyExt() == qfalse {
        if Q_irand(0, 30) == 0 {
            (*NPCInfo).desiredYaw = (*NPC).s.angles[1] + Q_irand(-90, 90) as f32;
        }
        if Q_irand(0, 30) == 0 {
            (*NPCInfo).desiredPitch = Q_irand(-20, 20) as f32;
        }
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    let mut enemyLOS: qboolean = qfalse;
    let mut enemyCS: qboolean = qfalse;
    let mut faceEnemy: qboolean = qfalse;
    let mut shoot: qboolean = qfalse;
    let mut impactPos: vec3_t = [0.0; 3];

    if NPC_ClearLOS((*NPC).enemy) != 0 {
        enemyLOS = qtrue;

        let hit: c_int = NPC_ShotEntity((*NPC).enemy, impactPos);
        let hitEnt: *mut gentity_t = addr_of_mut!(g_entities[hit as usize]);

        if hit == (*(*NPC).enemy).s.number
            || (!hitEnt.is_null() && (*hitEnt).takedamage != 0)
        {
            //can hit enemy or will hit glass or other minor breakable (or in emplaced gun), so shoot anyway
            enemyCS = qtrue;
            NPC_AimAdjust(2); //adjust aim better longer we have clear shot at enemy
            VectorCopy((*(*NPC).enemy).currentOrigin, (*NPCInfo).enemyLastSeenLocation);
        }
    }
    /*
    	else if ( gi.inPVS( NPC->enemy->currentOrigin, NPC->currentOrigin ) )
    	{
    		NPCInfo->enemyLastSeenTime = level.time;
    		faceEnemy = qtrue;
    		NPC_AimAdjust( -1 );//adjust aim worse longer we cannot see enemy
    	}
    */

    if enemyLOS != 0 {
        //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
        faceEnemy = qtrue;
    }
    if enemyCS != 0 {
        shoot = qtrue;
    }

    if faceEnemy != 0 {
        //face the enemy
        NPC_FaceEnemy(qtrue);
    } else {
        //we want to face in the dir we're running
        NPC_UpdateAngles(qtrue, qtrue);
    }

    if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 {
        shoot = qfalse;
    }

    if !(*NPC).enemy.is_null() && !(*(*NPC).enemy).enemy.is_null() {
        if (*(*NPC).enemy).s.weapon == WP_SABER
            && (*(*(*NPC).enemy).enemy).s.weapon == WP_SABER
        {
            //don't shoot at an enemy jedi who is fighting another jedi, for fear of injuring one or causing rogue blaster deflections (a la Obi Wan/Vader duel at end of ANH)
            shoot = qfalse;
        }
    }
    if shoot != 0 {
        //try to shoot if it's time
        if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON == 0 {
            // we've already fired, no need to do it again here
            WeaponThink(qtrue);
        }
    }
}
