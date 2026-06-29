// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// #include "g_headers.h"

// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// #include "g_headers.h"


// #include "b_local.h"
// #include "g_nav.h"
// #include "anims.h"
// #include "g_navigator.h"
// #include "wp_saber.h"

//extern void G_AddVoiceEvent( gentity_t *self, int event, int speakDebounceTime );
extern "C" {
    pub fn WP_DeactivateSaber(self_: *mut core::ffi::c_void, clearLength: core::ffi::c_int);
    pub fn PM_AnimLength(index: core::ffi::c_int, anim: core::ffi::c_int) -> core::ffi::c_int;
    pub fn NPC_CheckPlayerTeamStealth() -> core::ffi::c_int;
    pub fn UpdateGoal() -> core::ffi::c_int;
    pub fn NPC_MoveToGoal(qTest: core::ffi::c_int) -> core::ffi::c_int;
    pub fn NPC_CheckAlertEvents(lookForPlayer: core::ffi::c_int, lookForNPC: core::ffi::c_int, ignoreTeam: core::ffi::c_int, mustBeInPVS: core::ffi::c_int, minAlertEventLevel: core::ffi::c_int) -> core::ffi::c_int;
    pub fn VectorCopy(in_: *const [core::ffi::c_float; 3], out: *mut [core::ffi::c_float; 3]);
    pub fn VectorSubtract(veca: *const [core::ffi::c_float; 3], vecb: *const [core::ffi::c_float; 3], out: *mut [core::ffi::c_float; 3]);
    pub fn vectoangles(value1: *const [core::ffi::c_float; 3], angles: *mut [core::ffi::c_float; 3]);
    pub fn NPC_UpdateAngles(ucmd: core::ffi::c_int, doMove: core::ffi::c_int);
    pub fn G_SetEnemy(self_: *mut core::ffi::c_void, enemy: *mut core::ffi::c_void);
    pub fn TIMER_Set(ent: *const core::ffi::c_void, identifier: *const core::ffi::c_char, duration: core::ffi::c_int);
    pub fn TIMER_Done(ent: *const core::ffi::c_void, identifier: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn Q_irand(a: core::ffi::c_int, b: core::ffi::c_int) -> core::ffi::c_int;
    pub fn DistanceSquared(p1: *const [core::ffi::c_float; 3], p2: *const [core::ffi::c_float; 3]) -> core::ffi::c_float;
    pub fn NPC_ClearLOS(ent: *const core::ffi::c_void) -> core::ffi::c_int;
    pub fn InFOV(origin: *const [core::ffi::c_float; 3], viewpoint: *const [core::ffi::c_float; 3], angles: *const [core::ffi::c_float; 3], fovX: core::ffi::c_int, fovY: core::ffi::c_int) -> core::ffi::c_int;
    pub fn NPC_CheckEnemyExt() -> core::ffi::c_int;
    pub fn NPC_FaceEnemy();
    pub fn NPC_SetAnim(ent: *mut core::ffi::c_void, setAnimParts: core::ffi::c_int, anim: core::ffi::c_int, flags: core::ffi::c_int);
    pub static mut NPC: *mut core::ffi::c_void;
    pub static mut NPCInfo: *mut core::ffi::c_void;
}

//extern void G_AddVoiceEvent( gentity_t *self, int event, int speakDebounceTime );

static mut enemyLOS: core::ffi::c_int = 0;
static mut enemyCS: core::ffi::c_int = 0;
static mut faceEnemy: core::ffi::c_int = 0;
static mut move_: core::ffi::c_int = 0;
static mut shoot: core::ffi::c_int = 0;
static mut enemyDist: core::ffi::c_float = 0.0;

/*
void NPC_SaberDroid_PlayConfusionSound( gentity_t *self )
{//FIXME: make this a custom sound in sound set
	if ( self->health > 0 )
	{
		G_AddVoiceEvent( self, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000 );
	}
	//reset him to be totally unaware again
	TIMER_Set( self, "enemyLastVisible", 0 );
	TIMER_Set( self, "flee", 0 );
	self->NPC->squadState = SQUAD_IDLE;
	self->NPC->tempBehavior = BS_DEFAULT;

	//self->NPC->behaviorState = BS_PATROL;
	G_ClearEnemy( self );//FIXME: or just self->enemy = NULL;?

	self->NPC->investigateCount = 0;
}
*/

/*
-------------------------
ST_Move
-------------------------
*/

unsafe fn SaberDroid_Move() -> core::ffi::c_int
{
	use core::ptr::addr_of_mut;
	// NPCInfo->combatMove = qtrue;//always move straight toward our goal
	UpdateGoal();
	// if ( !NPCInfo->goalEntity )
	// {
	//	NPCInfo->goalEntity = NPC->enemy;
	// }
	// NPCInfo->goalRadius = 30.0f;

	let moved = NPC_MoveToGoal(1);
//	navInfo_t	info;

	//Get the move info
//	NAV_GetLastMove( info );

	//FIXME: if we bump into another one of our guys and can't get around him, just stop!
	//If we hit our target, then stop and fire!
//	if ( info.flags & NIF_COLLISION )
//	{
//		if ( info.blocker == NPC->enemy )
//		{
//			SaberDroid_HoldPosition();
//		}
//	}

	//If our move failed, then reset
	/*
	if ( moved == qfalse )
	{//couldn't get to enemy
		//just hang here
		SaberDroid_HoldPosition();
	}
	*/

	return moved;
}

/*
-------------------------
NPC_BSSaberDroid_Patrol
-------------------------
*/

unsafe fn NPC_BSSaberDroid_Patrol()
{//FIXME: pick up on bodies of dead buddies?
	use core::ptr::addr_of_mut;
	// if ( NPCInfo->confusionTime < level.time )
	// {
	//Look for any enemies
	// if ( NPCInfo->scriptFlags&SCF_LOOK_FOR_ENEMIES )
	// {
	if NPC_CheckPlayerTeamStealth() != 0
	{
		//NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be automatic now
		//NPC_AngerSound();
		NPC_UpdateAngles(1, 1);
		return;
	}
	// }

	// if ( !(NPCInfo->scriptFlags&SCF_IGNORE_ALERTS) )
	// {
	//Is there danger nearby
	let alertEvent = NPC_CheckAlertEvents(1, 1, -1, 0, 1); // AEL_SUSPICIOUS
	//There is an event to look at
	if alertEvent >= 0 {//&& level.alertEvents[alertEvent].ID != NPCInfo->lastAlertID )
		//NPCInfo->lastAlertID = level.alertEvents[alertEvent].ID;
		// if ( level.alertEvents[alertEvent].level >= AEL_DISCOVERED )
		// {
		// if ( level.alertEvents[alertEvent].owner &&
		// level.alertEvents[alertEvent].owner->client &&
		// level.alertEvents[alertEvent].owner->health >= 0 &&
		// level.alertEvents[alertEvent].owner->client->playerTeam == NPC->client->enemyTeam )
		// {//an enemy
		// G_SetEnemy( NPC, level.alertEvents[alertEvent].owner );
		//NPCInfo->enemyLastSeenTime = level.time;
		TIMER_Set(addr_of_mut!(NPC) as *const _, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand(500, 2500));
		// }
		// }
		// else
		// {//FIXME: get more suspicious over time?
		//Save the position for movement (if necessary)
		// VectorCopy( level.alertEvents[alertEvent].position, NPCInfo->investigateGoal );
		// NPCInfo->investigateDebounceTime = level.time + Q_irand( 500, 1000 );
		// if ( level.alertEvents[alertEvent].level == AEL_SUSPICIOUS )
		// {//suspicious looks longer
		// NPCInfo->investigateDebounceTime += Q_irand( 500, 2500 );
		// }
		// }
	}

	// if ( NPCInfo->investigateDebounceTime > level.time )
	// {//FIXME: walk over to it, maybe?  Not if not chase enemies
	//NOTE: stops walking or doing anything else below
	let mut dir: [core::ffi::c_float; 3] = [0.0; 3];
	let mut angles: [core::ffi::c_float; 3] = [0.0; 3];

	// VectorSubtract( NPCInfo->investigateGoal, NPC->client->renderInfo.eyePoint, dir );
	// vectoangles( dir, angles );

	// float	o_yaw = NPCInfo->desiredYaw;
	// float	o_pitch = NPCInfo->desiredPitch;
	// NPCInfo->desiredYaw = angles[YAW];
	// NPCInfo->desiredPitch = angles[PITCH];

	// NPC_UpdateAngles( qtrue, qtrue );

	// NPCInfo->desiredYaw = o_yaw;
	// NPCInfo->desiredPitch = o_pitch;
	// return;
	// }
	// }

	//If we have somewhere to go, then do that
	if UpdateGoal() != 0
	{
		// ucmd.buttons |= BUTTON_WALKING;
		NPC_MoveToGoal(1);
	}
	else if !(*addr_of_mut!(NPC)).is_null()
		// && !NPC->client
		// && NPC->client->ps.weaponTime == 0
		// && TIMER_Done( NPC, "attackDelay" )
		// && TIMER_Done( NPC, "inactiveDelay" )
	{
		// if ( NPC->client->ps.SaberActive() )
		// {
		WP_DeactivateSaber(addr_of_mut!(NPC), 0);
		NPC_SetAnim(addr_of_mut!(NPC), 0, 18, 0x00000002|0x00000004); // SETANIM_BOTH=0, BOTH_TURNOFF, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
		// }
	}

	NPC_UpdateAngles(1, 1);
}

unsafe fn SaberDroid_PowerLevelForSaberAnim(self_: *mut core::ffi::c_void) -> core::ffi::c_int
{
	// switch ( self->client->ps.legsAnim )
	// {
	// case BOTH_A2_TR_BL:
	// if ( self->client->ps.torsoAnimTimer <= 200 )
	// {//end of anim
	// return FORCE_LEVEL_0;
	// }
	// else if ( PM_AnimLength( self->client->clientInfo.animFileIndex, (animNumber_t)self->client->ps.legsAnim ) - self->client->ps.torsoAnimTimer < 200 )
	// {//beginning of anim
	// return FORCE_LEVEL_0;
	// }
	// return FORCE_LEVEL_2;
	// break;
	// case BOTH_A1_BL_TR:
	// if ( self->client->ps.torsoAnimTimer <= 300 )
	// {//end of anim
	// return FORCE_LEVEL_0;
	// }
	// else if ( PM_AnimLength( self->client->clientInfo.animFileIndex, (animNumber_t)self->client->ps.legsAnim ) - self->client->ps.torsoAnimTimer < 200 )
	// {//beginning of anim
	// return FORCE_LEVEL_0;
	// }
	// return FORCE_LEVEL_1;
	// break;
	// case BOTH_A1__L__R:
	// if ( self->client->ps.torsoAnimTimer <= 250 )
	// {//end of anim
	// return FORCE_LEVEL_0;
	// }
	// else if ( PM_AnimLength( self->client->clientInfo.animFileIndex, (animNumber_t)self->client->ps.legsAnim ) - self->client->ps.torsoAnimTimer < 150 )
	// {//beginning of anim
	// return FORCE_LEVEL_0;
	// }
	// return FORCE_LEVEL_1;
	// break;
	// case BOTH_A3__L__R:
	// if ( self->client->ps.torsoAnimTimer <= 200 )
	// {//end of anim
	// return FORCE_LEVEL_0;
	// }
	// else if ( PM_AnimLength( self->client->clientInfo.animFileIndex, (animNumber_t)self->client->ps.legsAnim ) - self->client->ps.torsoAnimTimer < 300 )
	// {//beginning of anim
	// return FORCE_LEVEL_0;
	// }
	// return FORCE_LEVEL_3;
	// break;
	// }
	0 // FORCE_LEVEL_0
}

/*
-------------------------
NPC_BSSaberDroid_Attack
-------------------------
*/

unsafe fn NPC_SaberDroid_PickAttack()
{
	use core::ptr::addr_of_mut;
	let mut attackAnim = Q_irand(0, 3);
	match attackAnim
	{
		0 => { // default
			attackAnim = 134; // BOTH_A2_TR_BL
			// NPC->client->ps.saberMove = LS_A_TR2BL;
			// NPC->client->ps.saberAnimLevel = SS_MEDIUM;
		},
		1 => {
			attackAnim = 135; // BOTH_A1_BL_TR
			// NPC->client->ps.saberMove = LS_A_BL2TR;
			// NPC->client->ps.saberAnimLevel = SS_FAST;
		},
		2 => {
			attackAnim = 136; // BOTH_A1__L__R
			// NPC->client->ps.saberMove = LS_A_L2R;
			// NPC->client->ps.saberAnimLevel = SS_FAST;
		},
		3 => {
			attackAnim = 137; // BOTH_A3__L__R
			// NPC->client->ps.saberMove = LS_A_L2R;
			// NPC->client->ps.saberAnimLevel = SS_STRONG;
		},
		_ => {},
	}
	// NPC->client->ps.saberBlocking = saberMoveData[NPC->client->ps.saberMove].blocking;
	// if ( saberMoveData[NPC->client->ps.saberMove].trailLength > 0 )
	// {
	// NPC->client->ps.SaberActivateTrail( saberMoveData[NPC->client->ps.saberMove].trailLength ); // saber trail lasts for 75ms...feel free to change this if you want it longer or shorter
	// }
	// else
	// {
	// NPC->client->ps.SaberDeactivateTrail( 0 );
	// }

	NPC_SetAnim(addr_of_mut!(NPC), 0, attackAnim, 0x00000200|0x00000002); // SETANIM_BOTH, SETANIM_FLAG_HOLD|SETANIM_FLAG_OVERRIDE
	// NPC->client->ps.torsoAnim = NPC->client->ps.legsAnim;//need to do this because we have no anim split but saber code checks torsoAnim
	// NPC->client->ps.weaponTime = NPC->client->ps.torsoAnimTimer = NPC->client->ps.legsAnimTimer;
	// NPC->client->ps.weaponstate = WEAPON_FIRING;
}

unsafe fn NPC_BSSaberDroid_Attack()
{
	use core::ptr::addr_of_mut;
	//Don't do anything if we're hurt
	// if ( NPC->painDebounceTime > level.time )
	// {
	NPC_UpdateAngles(1, 1);
	// return;
	// }

	//NPC_CheckEnemy( qtrue, qfalse );
	//If we don't have an enemy, just idle
	if NPC_CheckEnemyExt() == 0 {//!NPC->enemy )//
		// NPC->enemy = NULL;
		NPC_BSSaberDroid_Patrol();//FIXME: or patrol?
		return;
	}

	if (*addr_of_mut!(NPC)).is_null() {
		//WTF?  somehow we lost our enemy?
		NPC_BSSaberDroid_Patrol();//FIXME: or patrol?
		return;
	}

	*addr_of_mut!(enemyLOS) = 0;
	*addr_of_mut!(enemyCS) = 0;
	*addr_of_mut!(move_) = 1;
	*addr_of_mut!(faceEnemy) = 0;
	*addr_of_mut!(shoot) = 0;
	// enemyDist = DistanceSquared( NPC->enemy->currentOrigin, NPC->currentOrigin );
	*addr_of_mut!(enemyDist) = 0.0;

	//can we see our target?
	// if ( NPC_ClearLOS( NPC->enemy ) )
	if NPC_ClearLOS((*addr_of_mut!(NPC))) != 0 {
		// NPCInfo->enemyLastSeenTime = level.time;
		*addr_of_mut!(enemyLOS) = 1;

		// if ( enemyDist <= 4096 && InFOV( NPC->enemy->currentOrigin, NPC->currentOrigin, NPC->client->ps.viewangles, 90, 45 ) )//within 64 & infront
		if *addr_of_mut!(enemyDist) <= 4096.0 && InFOV(addr_of!([0.0; 3]), addr_of!([0.0; 3]), addr_of!([0.0; 3]), 90, 45) != 0 {//within 64 & infront
			// VectorCopy( NPC->enemy->currentOrigin, NPCInfo->enemyLastSeenLocation );
			*addr_of_mut!(enemyCS) = 1;
		}
	}
	/*
	else if ( gi.inPVS( NPC->enemy->currentOrigin, NPC->currentOrigin ) )
	{
		NPCInfo->enemyLastSeenTime = level.time;
		faceEnemy = qtrue;
	}
	*/

	if *addr_of_mut!(enemyLOS) != 0 {
		//FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
		*addr_of_mut!(faceEnemy) = 1;
	}

	if TIMER_Done(addr_of_mut!(NPC) as *const _, b"taunting\0".as_ptr() as *const core::ffi::c_char) == 0 {
		*addr_of_mut!(move_) = 0;
	}
	else if *addr_of_mut!(enemyCS) != 0 {
		*addr_of_mut!(shoot) = 1;
		// if ( enemyDist < (NPC->maxs[0]+NPC->enemy->maxs[0]+32)*(NPC->maxs[0]+NPC->enemy->maxs[0]+32) )
		// {//close enough
		// move = qfalse;
		// }
	}//this should make him chase enemy when out of range...?

	// if ( NPC->client->ps.legsAnimTimer
	// && NPC->client->ps.legsAnim != BOTH_A3__L__R )//this one is a running attack
	// {//in the middle of a held, stationary anim, can't move
	// move = qfalse;
	// }

	if *addr_of_mut!(move_) != 0 {
		//move toward goal
		*addr_of_mut!(move_) = SaberDroid_Move();
		if *addr_of_mut!(move_) != 0 {
			//if we had to chase him, be sure to attack as soon as possible
			TIMER_Set(addr_of_mut!(NPC) as *const _, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, 0); // NPC->client->ps.weaponTime
		}
	}

	if *addr_of_mut!(faceEnemy) == 0 {
		//we want to face in the dir we're running
		if *addr_of_mut!(move_) != 0 {
			//don't run away and shoot
			// NPCInfo->desiredYaw = NPCInfo->lastPathAngles[YAW];
			// NPCInfo->desiredPitch = 0;
			*addr_of_mut!(shoot) = 0;
		}
		NPC_UpdateAngles(1, 1);
	}
	else { // if ( faceEnemy )
		//face the enemy
		NPC_FaceEnemy();
	}

	// if ( NPCInfo->scriptFlags&SCF_DONT_FIRE )
	// {
	// shoot = qfalse;
	// }

	//FIXME: need predicted blocking?
	//FIXME: don't shoot right away!
	if *addr_of_mut!(shoot) != 0 {
		//try to shoot if it's time
		if TIMER_Done(addr_of_mut!(NPC) as *const _, b"attackDelay\0".as_ptr() as *const core::ffi::c_char) != 0 {
			// if( !(NPCInfo->scriptFlags & SCF_FIRE_WEAPON) ) // we've already fired, no need to do it again here
			// {
			NPC_SaberDroid_PickAttack();
			// if ( NPCInfo->rank > RANK_CREWMAN )
			// {
			TIMER_Set(addr_of_mut!(NPC) as *const _, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand(0, 1000)); // NPC->client->ps.weaponTime+Q_irand(0, 1000)
			// }
			// else
			// {
			// TIMER_Set( NPC, "attackDelay", NPC->client->ps.weaponTime+Q_irand( 0, 1000 )+(Q_irand( 0, (3-g_spskill->integer)*2 )*500) );
			// }
			// }
		}
	}
}

unsafe fn NPC_BSSD_Default()
{
	use core::ptr::addr_of_mut;
	if (*addr_of_mut!(NPC)).is_null() {
		//don't have an enemy, look for one
		NPC_BSSaberDroid_Patrol();
	}
	else { //if ( NPC->enemy )
		//have an enemy
		// if ( !NPC->client->ps.SaberActive() )
		// {
		// NPC->client->ps.SaberActivate();
		// if ( NPC->client->ps.legsAnim == BOTH_TURNOFF
		// || NPC->client->ps.legsAnim == BOTH_STAND1 )
		// {
		NPC_SetAnim(addr_of_mut!(NPC), 0, 16, 0x00000002|0x00000004); // SETANIM_BOTH=0, BOTH_TURNON, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
		// }
		// }

		NPC_BSSaberDroid_Attack();
		TIMER_Set(addr_of_mut!(NPC) as *const _, b"inactiveDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand(2000, 4000));
	}
	// if ( !NPC->client->ps.weaponTime )
	// {
	// NPC->client->ps.saberMove = LS_READY;
	// NPC->client->ps.saberBlocking = saberMoveData[LS_READY].blocking;
	// NPC->client->ps.SaberDeactivateTrail( 0 );
	// NPC->client->ps.saberAnimLevel = SS_MEDIUM;
	// NPC->client->ps.weaponstate = WEAPON_READY;
	// }
}
