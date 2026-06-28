// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// (C header: g_headers.h)

// b_local.h
// g_nav.h
// anims.h
// g_navigator.h

use core::ffi::{c_int, c_void};

// Stub type declarations for external game types
#[repr(C)]
pub struct gentity_t {
    _stub: [u8; 0],
}

#[repr(C)]
pub struct vec3_t {
    _stub: [u8; 0],
}

// External function declarations
extern "C" {
    pub fn CG_DrawAlert(origin: *const c_void, rating: f32);
    pub fn G_AddVoiceEvent(elf: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    pub fn NPC_TempLookTarget(
        elf: *mut gentity_t,
        lookEntNum: c_int,
        minLookTime: c_int,
        maxLookTime: c_int,
    );
    pub fn G_ExpandPointToBBox(
        point: *const c_void,
        mins: *const c_void,
        maxs: *const c_void,
        ignore: c_int,
        clipmask: c_int,
    ) -> c_int;
    pub fn NPC_AimAdjust(change: c_int);
    pub fn FlyingCreature(ent: *mut gentity_t) -> c_int;
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

extern "C" {
    pub fn NPC_CheckPlayerTeamStealth() -> c_int;
}

// Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_UNDERFIRE: c_int = 1;
const LSTATE_INVESTIGATE: c_int = 2;

// Static variables
static mut enemyLOS: c_int = 0;
static mut enemyCS: c_int = 0;
static mut faceEnemy: c_int = 0;
static mut move_val: c_int = 0;
static mut shoot: c_int = 0;
static mut enemyDist: f32 = 0.0;

// Stub statics and extern declarations for missing game functions
extern "C" {
    pub static mut NPC: *mut c_void;
    pub static mut NPCInfo: *mut c_void;
    pub static mut level: c_void;
    pub static mut g_entities: [gentity_t; 1]; // Stub array
    pub static mut ucmd: c_void;

    pub fn TIMER_Set(ent: *mut gentity_t, name: *const c_void, duration: c_int);
    pub fn TIMER_Done(ent: *const gentity_t, name: *const c_void) -> c_int;
    pub fn G_ClearEnemy(ent: *mut gentity_t);
    pub fn NPC_Pain(
        elf: *mut gentity_t,
        inflictor: *mut gentity_t,
        other: *mut gentity_t,
        point: *const c_void,
        damage: c_int,
        mod_: c_int,
    );
    pub fn NPC_FreeCombatPoint(cp: c_int, alreadyFree: c_int);
    pub fn NPC_MoveToGoal(allowVJump: c_int) -> c_int;
    pub fn NPC_FindCombatPoint(
        origin1: *const c_void,
        mins: *const c_void,
        maxs: *const c_void,
        cpFlags: c_int,
        radius: c_int,
    ) -> c_int;
    pub fn NPC_SetCombatPoint(cp: c_int);
    pub fn NPC_SetMoveGoal(
        ent: *mut gentity_t,
        goal: *const c_void,
        radius: c_int,
        block: c_int,
        cp: c_int,
    );
    pub fn NPC_CheckPlayerTeamStealth() -> c_int;
    pub fn NPC_UpdateAngles(doPitch: c_int, doYaw: c_int);
    pub fn NPC_CheckAlertEvents(
        check: c_int,
        checkOther: c_int,
        ignoreAlert: c_int,
        mustHaveOwner: c_int,
        minAlert: c_int,
    ) -> c_int;
    pub fn NPC_CheckForDanger(alertEvent: c_int) -> c_int;
    pub fn G_SetEnemy(ent: *mut gentity_t, enemy: *mut gentity_t);
    pub fn UpdateGoal() -> c_int;
    pub fn VectorCopy(in_: *const c_void, out: *mut c_void);
    pub fn VectorSubtract(veca: *const c_void, vecb: *const c_void, out: *mut c_void);
    pub fn vectoangles(value: *const c_void, angles: *mut c_void);
    pub fn NPC_CheckEnemyExt() -> c_int;
    pub fn NPC_ClearLOS(ent: *const gentity_t) -> c_int;
    pub fn InFOV(
        origin: *const c_void,
        from: *const c_void,
        angles: *const c_void,
        fov1: c_int,
        fov2: c_int,
    ) -> c_int;
    pub fn NPC_ShotEntity(ent: *const gentity_t) -> c_int;
    pub fn DistanceSquared(p1: *const c_void, p2: *const c_void) -> f32;
    pub fn DistanceHorizontalSquared(p1: *const c_void, p2: *const c_void) -> f32;
    pub fn NPC_ChangeWeapon(wp: c_int);
    pub fn NPC_FaceEnemy();
    pub fn VectorCompare(v1: *const c_void, v2: *const c_void) -> c_int;
    pub fn WeaponThink(force: c_int);
    pub fn NPC_BSGrenadier_Patrol();
    pub fn NPC_ReachedGoal();
}

// Weapon constants (stubs - copied from pattern)
const WP_MELEE: c_int = 0;
const WP_THERMAL: c_int = 1;
const WP_SABER: c_int = 2;

// Event constants (stubs)
const EV_CONFUSE1: c_int = 1;
const EV_CONFUSE3: c_int = 3;
const EV_PUSHED1: c_int = 4;
const EV_PUSHED3: c_int = 6;

// Squad state constants (stubs)
const SQUAD_IDLE: c_int = 0;
const SQUAD_STAND_AND_SHOOT: c_int = 1;
const SQUAD_RETREAT: c_int = 2;
const SQUAD_TRANSITION: c_int = 3;
const SQUAD_SCOUT: c_int = 4;
const SQUAD_COVER: c_int = 5;

// Alert event level constants (stubs)
const AEL_SUSPICIOUS: c_int = 1;
const AEL_DISCOVERED: c_int = 2;
const AEL_DANGER: c_int = 3;

// Script flag constants (stubs)
const SCF_LOOK_FOR_ENEMIES: c_int = 1;
const SCF_IGNORE_ALERTS: c_int = 2;
const SCF_CHASE_ENEMIES: c_int = 4;
const SCF_DONT_FIRE: c_int = 8;
const SCF_FIRE_WEAPON: c_int = 16;
const SCF_USE_CP_NEAREST: c_int = 32;

// Combat point constants (stubs)
const CP_CLEAR: c_int = 1;
const CP_HAS_ROUTE: c_int = 2;
const CP_FLANK: c_int = 4;
const CP_APPROACH_ENEMY: c_int = 8;
const CP_CLOSEST: c_int = 16;
const CP_NEAREST: c_int = 32;
const CP_HORZ_DIST_COLL: c_int = 64;

// Stat constants (stubs)
const STAT_WEAPONS: c_int = 1;

// Button constants (stubs)
const BUTTON_WALKING: c_int = 1;

// SVF constants (stubs)
const SVF_GLASS_BRUSH: c_int = 1;

// Angle constants (stubs)
const YAW: c_int = 0;
const PITCH: c_int = 1;

#[allow(non_snake_case)]
pub fn Grenadier_ClearTimers(ent: *mut gentity_t) {
    unsafe {
        TIMER_Set(ent, "chatter\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "duck\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "stand\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "shuffleTime\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "sleepTime\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "enemyLastVisible\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "roamTime\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "hideTime\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "attackDelay\0" as *const u8 as *const c_void, 0); //FIXME: Slant for difficulty levels
        TIMER_Set(ent, "stick\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "scoutTime\0" as *const u8 as *const c_void, 0);
        TIMER_Set(ent, "flee\0" as *const u8 as *const c_void, 0);
    }
}

#[allow(non_snake_case)]
pub fn NPC_Grenadier_PlayConfusionSound(self_: *mut gentity_t) {
    //FIXME: make this a custom sound in sound set
    unsafe {
        let health = (*self_).health;
        if health > 0 {
            G_AddVoiceEvent(self_, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000);
        }
        //reset him to be totally unaware again
        TIMER_Set(self_, "enemyLastVisible\0" as *const u8 as *const c_void, 0);
        TIMER_Set(self_, "flee\0" as *const u8 as *const c_void, 0);
        // (*NPCInfo).squadState = SQUAD_IDLE;
        // (*NPCInfo).tempBehavior = BS_DEFAULT;

        //(*NPCInfo).behaviorState = BS_PATROL;
        G_ClearEnemy(self_); //FIXME: or just (*self_).enemy = NULL;?

        // (*NPCInfo).investigateCount = 0;
    }
}

/*
-------------------------
NPC_ST_Pain
-------------------------
*/

#[allow(non_snake_case)]
pub fn NPC_Grenadier_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const c_void,
    damage: c_int,
    mod_: c_int,
) {
    unsafe {
        // (*(*self_).NPC).localState = LSTATE_UNDERFIRE;

        TIMER_Set(self_, "duck\0" as *const u8 as *const c_void, -1);
        TIMER_Set(self_, "stand\0" as *const u8 as *const c_void, 2000);

        NPC_Pain(self_, inflictor, other, point, damage, mod_);

        let npc_health = (*self_).health;
        if damage == 0 && npc_health > 0 {
            //FIXME: better way to know I was pushed
            G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1, EV_PUSHED3), 2000);
        }
    }
}

/*
-------------------------
ST_HoldPosition
-------------------------
*/

#[allow(non_snake_case)]
static mut _Grenadier_HoldPosition_guard: () = ();

#[allow(non_snake_case)]
fn Grenadier_HoldPosition() {
    unsafe {
        // NPC_FreeCombatPoint( (*NPCInfo).combatPoint, qtrue );
        // (*NPCInfo).goalEntity = NULL;

        /*if ( TIMER_Done( NPC, "stand" ) )
        {//FIXME: what if can't shoot from this pos?
            TIMER_Set( NPC, "duck", Q_irand( 2000, 4000 ) );
        }
        */
    }
}

/*
-------------------------
ST_Move
-------------------------
*/

#[allow(non_snake_case)]
fn Grenadier_Move() -> c_int {
    unsafe {
        // (*NPCInfo).combatMove = qtrue; //always move straight toward our goal

        let moved: c_int = NPC_MoveToGoal(1); //true
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
        if moved == 0 {
            //couldn't get to enemy
            // if ( ((*NPCInfo).scriptFlags&SCF_CHASE_ENEMIES) && (*NPC).client->ps.weapon == WP_THERMAL && (*NPCInfo).goalEntity && (*NPCInfo).goalEntity == (*NPC).enemy )
            // {//we were running after enemy
            // 	//Try to find a combat point that can hit the enemy
            // 	let mut cpFlags: c_int = (CP_CLEAR|CP_HAS_ROUTE);
            // 	if ( (*NPCInfo).scriptFlags&SCF_USE_CP_NEAREST )
            // 	{
            // 		cpFlags &= !(CP_FLANK|CP_APPROACH_ENEMY|CP_CLOSEST);
            // 		cpFlags |= CP_NEAREST;
            // 	}
            // 	let cp: c_int = NPC_FindCombatPoint( (*NPC).currentOrigin, (*NPC).currentOrigin, (*NPC).currentOrigin, cpFlags, 32 );
            // 	if ( cp == -1 && !((*NPCInfo).scriptFlags&SCF_USE_CP_NEAREST) )
            // 	{//okay, try one by the enemy
            // 		cp = NPC_FindCombatPoint( (*NPC).currentOrigin, (*NPC).currentOrigin, (*(*NPC).enemy).currentOrigin, CP_CLEAR|CP_HAS_ROUTE|CP_HORZ_DIST_COLL, 32 );
            // 	}
            // 	//NOTE: there may be a perfectly valid one, just not one within CP_COLLECT_RADIUS of either me or him...
            // 	if ( cp != -1 )
            // 	{//found a combat point that has a clear shot to enemy
            // 		NPC_SetCombatPoint( cp );
            // 		NPC_SetMoveGoal( NPC, level.combatPoints[cp].origin, 8, qtrue, cp );
            // 		return moved;
            // 	}
            // }
            //just hang here
            Grenadier_HoldPosition();
        }

        return moved;
    }
}

/*
-------------------------
NPC_BSGrenadier_Patrol
-------------------------
*/

#[allow(non_snake_case)]
pub fn NPC_BSGrenadier_Patrol() {
    //FIXME: pick up on bodies of dead buddies?
    unsafe {
        // if ( (*NPCInfo).confusionTime < (*level).time )
        // {
        //Look for any enemies
        // if ( (*NPCInfo).scriptFlags&SCF_LOOK_FOR_ENEMIES )
        // {
        if NPC_CheckPlayerTeamStealth() != 0 {
            //(*NPCInfo).behaviorState = BS_HUNT_AND_KILL;//should be automatic now
            //NPC_AngerSound();
            NPC_UpdateAngles(1, 1);
            return;
        }
        // 	}

        // if ( !((*NPCInfo).scriptFlags&SCF_IGNORE_ALERTS) )
        // {
        //Is there danger nearby
        let alertEvent: c_int = NPC_CheckAlertEvents(1, 1, -1, 0, AEL_SUSPICIOUS);
        if NPC_CheckForDanger(alertEvent) != 0 {
            NPC_UpdateAngles(1, 1);
            return;
        } else {
            //check for other alert events
            //There is an event to look at
            if alertEvent >= 0 {
                //&& (*level).alertEvents[alertEvent].ID != (*NPCInfo).lastAlertID )
                // (*NPCInfo).lastAlertID = (*level).alertEvents[alertEvent].ID;
                // if ( (*level).alertEvents[alertEvent].level == AEL_DISCOVERED )
                // {
                // 	if ( (*level).alertEvents[alertEvent].owner &&
                // 		(*level).alertEvents[alertEvent].owner->client &&
                // 		(*level).alertEvents[alertEvent].owner->health >= 0 &&
                // 		(*level).alertEvents[alertEvent].owner->client->playerTeam == (*(*NPC).client).enemyTeam )
                // 	{//an enemy
                // 		G_SetEnemy( NPC, (*level).alertEvents[alertEvent].owner );
                // 		//(*NPCInfo).enemyLastSeenTime = (*level).time;
                // 		TIMER_Set( NPC, "attackDelay", Q_irand( 500, 2500 ) );
                // 	}
                // }
                // else
                // {//FIXME: get more suspicious over time?
                // 	//Save the position for movement (if necessary)
                // 	VectorCopy( (*level).alertEvents[alertEvent].position, (*NPCInfo).investigateGoal );
                // 	(*NPCInfo).investigateDebounceTime = (*level).time + Q_irand( 500, 1000 );
                // 	if ( (*level).alertEvents[alertEvent].level == AEL_SUSPICIOUS )
                // 	{//suspicious looks longer
                // 		(*NPCInfo).investigateDebounceTime += Q_irand( 500, 2500 );
                // 	}
                // }
            }
        }

        // if ( (*NPCInfo).investigateDebounceTime > (*level).time )
        // {//FIXME: walk over to it, maybe?  Not if not chase enemies
        // 	//NOTE: stops walking or doing anything else below
        // 	let mut dir: [f32; 3] = [0.0; 3];
        // 	let mut angles: [f32; 3] = [0.0; 3];
        // 	let o_yaw: f32;
        // 	let o_pitch: f32;

        // 	VectorSubtract( (*NPCInfo).investigateGoal, (*(*NPC).client).renderInfo.eyePoint, &mut dir );
        // 	vectoangles( &dir, &mut angles );

        // 	o_yaw = (*NPCInfo).desiredYaw;
        // 	o_pitch = (*NPCInfo).desiredPitch;
        // 	(*NPCInfo).desiredYaw = angles[YAW];
        // 	(*NPCInfo).desiredPitch = angles[PITCH];

        // 	NPC_UpdateAngles( qtrue, qtrue );

        // 	(*NPCInfo).desiredYaw = o_yaw;
        // 	(*NPCInfo).desiredPitch = o_pitch;
        // 	return;
        // }
        // }
        // }

        //If we have somewhere to go, then do that
        if UpdateGoal() != 0 {
            // ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(1); //true
        }

        NPC_UpdateAngles(1, 1);
    }
}

/*
-------------------------
NPC_BSGrenadier_Idle
-------------------------
*/
/*
fn NPC_BSGrenadier_Idle( )
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

#[allow(non_snake_case)]
fn Grenadier_CheckMoveState() {
    unsafe {
        //See if we're a scout
        // if ( !((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) )//behaviorState == BS_STAND_AND_SHOOT )
        // {
        // 	if ( (*NPCInfo).goalEntity == (*NPC).enemy )
        // 	{
        // 		move = qfalse;
        // 		return;
        // 	}
        // }
        //See if we're running away
        // else if ( (*NPCInfo).squadState == SQUAD_RETREAT )
        // {
        // 	if ( TIMER_Done( NPC, "flee" ) != 0 )
        // 	{
        // 		(*NPCInfo).squadState = SQUAD_IDLE;
        // 	}
        // 	else
        // 	{
        // 		faceEnemy = qfalse;
        // 	}
        // }
        /*
        else if ( (*NPCInfo).squadState == SQUAD_IDLE )
        {
            if ( !(*NPCInfo).goalEntity )
            {
                move = qfalse;
                return;
            }
            //Should keep moving toward player when we're out of range... right?
        }
        */

        //See if we're moving towards a goal, not the enemy
        // if ( ( (*NPCInfo).goalEntity != (*NPC).enemy ) && ( (*NPCInfo).goalEntity != NULL ) )
        // {
        // 	//Did we make it?
        // 	if ( STEER::Reached(NPC, (*NPCInfo).goalEntity, 16, !!FlyingCreature(NPC)) ||
        // 		( (*NPCInfo).squadState == SQUAD_SCOUT && enemyLOS && enemyDist <= 10000 ) )
        // 	{
        // 		let newSquadState: c_int = SQUAD_STAND_AND_SHOOT;
        // 		//we got where we wanted to go, set timers based on why we were running
        // 		match (*NPCInfo).squadState
        // 		{
        // 		case SQUAD_RETREAT://was running away
        // 			TIMER_Set( NPC, "duck", ((*NPC).max_health - (*NPC).health) * 100 );
        // 			TIMER_Set( NPC, "hideTime", Q_irand( 3000, 7000 ) );
        // 			newSquadState = SQUAD_COVER;
        // 			break;
        // 		case SQUAD_TRANSITION://was heading for a combat point
        // 			TIMER_Set( NPC, "hideTime", Q_irand( 2000, 4000 ) );
        // 			break;
        // 		case SQUAD_SCOUT://was running after player
        // 			break;
        // 		default:
        // 			break;
        // 		}
        // 		NPC_ReachedGoal();
        // 		//don't attack right away
        // 		TIMER_Set( NPC, "attackDelay", Q_irand( 250, 500 ) );	//FIXME: Slant for difficulty levels
        // 		//don't do something else just yet
        // 		TIMER_Set( NPC, "roamTime", Q_irand( 1000, 4000 ) );
        // 		//stop fleeing
        // 		if ( (*NPCInfo).squadState == SQUAD_RETREAT )
        // 		{
        // 			TIMER_Set( NPC, "flee", -(*level).time );
        // 			(*NPCInfo).squadState = SQUAD_IDLE;
        // 		}
        // 		return;
        // 	}

        // 	//keep going, hold of roamTimer until we get there
        // 	TIMER_Set( NPC, "roamTime", Q_irand( 4000, 8000 ) );
        // }

        // if ( !(*NPCInfo).goalEntity )
        // {
        // 	if ( (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES )
        // 	{
        // 		(*NPCInfo).goalEntity = (*NPC).enemy;
        // 		(*NPCInfo).goalRadius = ((*NPC).maxs[0]*1.5);
        // 	}
        // }
    }
}

/*
-------------------------
ST_CheckFireState
-------------------------
*/

#[allow(non_snake_case)]
fn Grenadier_CheckFireState() {
    unsafe {
        if enemyCS != 0 {
            //if have a clear shot, always try
            return;
        }

        // if ( (*NPCInfo).squadState == SQUAD_RETREAT || (*NPCInfo).squadState == SQUAD_TRANSITION || (*NPCInfo).squadState == SQUAD_SCOUT )
        // {//runners never try to fire at the last pos
        // 	return;
        // }

        // if ( !VectorCompare( (*(*NPC).client).ps.velocity, vec3_origin ) )
        // {//if moving at all, don't do this
        // 	return;
        // }

        //continue to fire on their last position
        /*
        if ( !Q_irand( 0, 1 ) && (*NPCInfo).enemyLastSeenTime && (*level).time - (*NPCInfo).enemyLastSeenTime < 4000 )
        {
            //Fire on the last known position
            let mut muzzle: [f32; 3] = [0.0; 3];
            let mut dir: [f32; 3] = [0.0; 3];
            let mut angles: [f32; 3] = [0.0; 3];

            CalcEntitySpot( NPC, SPOT_WEAPON, &mut muzzle );
            VectorSubtract( (*NPCInfo).enemyLastSeenLocation, &muzzle, &mut dir );

            VectorNormalize( &mut dir );

            vectoangles( &dir, &mut angles );

            (*NPCInfo).desiredYaw		= angles[YAW];
            (*NPCInfo).desiredPitch	= angles[PITCH];
            //FIXME: they always throw toward enemy, so this will be very odd...
            shoot = qtrue;
            faceEnemy = qfalse;

            return;
        }
        */
    }
}

#[allow(non_snake_case)]
pub fn Grenadier_EvaluateShot(hit: c_int) -> c_int {
    unsafe {
        // if ( !(*NPC).enemy )
        // {
        // 	return qfalse;
        // }

        // if ( hit == (*(*NPC).enemy).s.number || (&g_entities[hit] != NULL && ((*&g_entities[hit]).svFlags&SVF_GLASS_BRUSH)) )
        // {//can hit enemy or will hit glass, so shoot anyway
        // 	return qtrue;
        // }
        return 0; //qfalse
    }
}

/*
-------------------------
NPC_BSGrenadier_Attack
-------------------------
*/

#[allow(non_snake_case)]
pub fn NPC_BSGrenadier_Attack() {
    unsafe {
        //Don't do anything if we're hurt
        // if ( (*NPC).painDebounceTime > (*level).time )
        // {
        // 	NPC_UpdateAngles( qtrue, qtrue );
        // 	return;
        // }

        //NPC_CheckEnemy( qtrue, qfalse );
        //If we don't have an enemy, just idle
        if NPC_CheckEnemyExt() == 0 {
            //!(*NPC).enemy )//
            NPC_BSGrenadier_Patrol(); //FIXME: or patrol?
            return;
        }

        // if ( TIMER_Done( NPC, "flee" ) != 0 && NPC_CheckForDanger( NPC_CheckAlertEvents( qtrue, qtrue, -1, qfalse, AEL_DANGER ) ) != 0 )
        // {//going to run
        // 	NPC_UpdateAngles( qtrue, qtrue );
        // 	return;
        // }

        // if ( !(*NPC).enemy )
        // {//WTF?  somehow we lost our enemy?
        // 	NPC_BSGrenadier_Patrol();//FIXME: or patrol?
        // 	return;
        // }

        enemyLOS = 0;
        enemyCS = 0;
        move_val = 1; //true
        faceEnemy = 0; //qfalse
        shoot = 0; //qfalse
        // enemyDist = DistanceSquared( (*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin );

        //See if we should switch to melee attack
        // if ( enemyDist < 16384 && (!(*(*NPC).enemy).client||(*(*(*NPC).enemy).client).ps.weapon != WP_SABER||(!(*(*(*NPC).enemy).client).ps.SaberActive())) )//128
        // {//enemy is close and not using saber
        // 	if ( (*(*NPC).client).ps.weapon == WP_THERMAL )
        // 	{//grenadier
        // 		let mut trace: trace_t = Default::default();
        // 		gi.trace ( &mut trace, (*NPC).currentOrigin, (*(*NPC).enemy).mins, (*(*NPC).enemy).maxs, (*(*NPC).enemy).currentOrigin, (*NPC).s.number, (*(*NPC).enemy).clipmask );
        // 		if ( !(*trace).allsolid && !(*trace).startsolid && ((*trace).fraction == 1.0 || (*trace).entityNum == (*(*NPC).enemy).s.number ) )
        // 		{//I can get right to him
        // 			//reset fire-timing variables
        // 			NPC_ChangeWeapon( WP_MELEE );
        // 			if ( !((*NPCInfo).scriptFlags&SCF_CHASE_ENEMIES) )//(*NPCInfo).behaviorState == BS_STAND_AND_SHOOT )
        // 			{//FIXME: should we be overriding scriptFlags?
        // 				(*NPCInfo).scriptFlags |= SCF_CHASE_ENEMIES;//(*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
        // 			}
        // 		}
        // 	}
        // }
        // else if ( enemyDist > 65536 || ((*(*NPC).enemy).client && (*(*(*NPC).enemy).client).ps.weapon == WP_SABER && (*(*(*NPC).enemy).client).ps.SaberActive()) )//256
        // {//enemy is far or using saber
        // 	if ( (*(*NPC).client).ps.weapon == WP_MELEE && ((*(*NPC).client).ps.stats[STAT_WEAPONS]&(1<<WP_THERMAL)) )
        // 	{//fisticuffs, make switch to thermal if have it
        // 		//reset fire-timing variables
        // 		NPC_ChangeWeapon( WP_THERMAL );
        // 	}
        // }

        //can we see our target?
        // if ( NPC_ClearLOS( (*NPC).enemy ) != 0 )
        // {
        // 	(*NPCInfo).enemyLastSeenTime = (*level).time;
        // 	enemyLOS = qtrue;

        // 	if ( (*(*NPC).client).ps.weapon == WP_MELEE )
        // 	{
        // 		if ( enemyDist <= 4096 && InFOV( (*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin, (*(*NPC).client).ps.viewangles, 90, 45 ) != 0 )//within 64 & infront
        // 		{
        // 			VectorCopy( (*(*NPC).enemy).currentOrigin, (*NPCInfo).enemyLastSeenLocation );
        // 			enemyCS = qtrue;
        // 		}
        // 	}
        // 	else if ( InFOV( (*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin, (*(*NPC).client).ps.viewangles, 45, 90 ) != 0 )
        // 	{//in front of me
        // 		//can we shoot our target?
        // 		//FIXME: how accurate/necessary is this check?
        // 		let hit: c_int = NPC_ShotEntity( (*NPC).enemy );
        // 		let hitEnt: *mut gentity_t = &mut g_entities[hit as usize];
        // 		if ( hit == (*(*NPC).enemy).s.number
        // 			|| ( hitEnt != NULL && (*hitEnt).client != NULL && (*(*hitEnt).client).playerTeam == (*(*NPC).client).enemyTeam ) )
        // 		{
        // 			VectorCopy( (*(*NPC).enemy).currentOrigin, (*NPCInfo).enemyLastSeenLocation );
        // 			let enemyHorzDist: f32 = DistanceHorizontalSquared( (*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin );
        // 			if ( enemyHorzDist < 1048576 )
        // 			{//within 1024
        // 				enemyCS = qtrue;
        // 				NPC_AimAdjust( 2 );//adjust aim better longer we have clear shot at enemy
        // 			}
        // 			else
        // 			{
        // 				NPC_AimAdjust( 1 );//adjust aim better longer we can see enemy
        // 			}
        // 		}
        // 	}
        // }
        // else
        // {
        // 	NPC_AimAdjust( -1 );//adjust aim worse longer we cannot see enemy
        // }
        /*
        else if ( gi.inPVS( (*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin ) )
        {
            (*NPCInfo).enemyLastSeenTime = (*level).time;
            faceEnemy = qtrue;
        }
        */

        if enemyLOS != 0 {
            //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
            faceEnemy = 1; //true
        }

        if enemyCS != 0 {
            shoot = 1; //true
            // if ( (*(*NPC).client).ps.weapon == WP_THERMAL )
            // {//don't chase and throw
            // 	move = qfalse;
            // }
            // else if ( (*(*NPC).client).ps.weapon == WP_MELEE && enemyDist < ((*NPC).maxs[0]+(*(*NPC).enemy).maxs[0]+16)*((*NPC).maxs[0]+(*(*NPC).enemy).maxs[0]+16) )
            // {//close enough
            // 	move = qfalse;
            // }
        } //this should make him chase enemy when out of range...?

        //Check for movement to take care of
        Grenadier_CheckMoveState();

        //See if we should override shooting decision with any special considerations
        Grenadier_CheckFireState();

        if move_val != 0 {
            //move toward goal
            // if ( (*NPCInfo).goalEntity != NULL )//&& ( (*NPCInfo).goalEntity != (*NPC).enemy || enemyDist > 10000 ) )//100 squared
            // {
            // 	move = Grenadier_Move();
            // }
            // else
            // {
            // 	move = qfalse;
            // }
        }

        if move_val == 0 {
            // if ( !TIMER_Done( NPC, "duck" ) )
            // {
            // 	ucmd.upmove = -127;
            // }
            //FIXME: what about leaning?
        } else {
            //stop ducking!
            TIMER_Set(
                NPC as *mut gentity_t,
                "duck\0" as *const u8 as *const c_void,
                -1,
            );
        }

        if faceEnemy == 0 {
            //we want to face in the dir we're running
            if move_val != 0 {
                //don't run away and shoot
                // (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[YAW];
                // (*NPCInfo).desiredPitch = 0;
                shoot = 0; //qfalse
            }
            NPC_UpdateAngles(1, 1);
        } else {
            // if ( faceEnemy )
            //face the enemy
            NPC_FaceEnemy();
        }

        // if ( (*NPCInfo).scriptFlags&SCF_DONT_FIRE )
        // {
        // 	shoot = qfalse;
        // }

        //FIXME: don't shoot right away!
        if shoot != 0 {
            //try to shoot if it's time
            // if ( TIMER_Done( NPC, "attackDelay" ) != 0 )
            // {
            // 	if( !((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) ) // we've already fired, no need to do it again here
            // 	{
            // 		WeaponThink( qtrue );
            // 		TIMER_Set( NPC, "attackDelay", (*NPCInfo).shotTime-(*level).time );
            // 	}

            // }
        }
    }
}

#[allow(non_snake_case)]
pub fn NPC_BSGrenadier_Default() {
    unsafe {
        // if( (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON )
        // {
        // 	WeaponThink( qtrue );
        // }

        // if( !(*NPC).enemy )
        // {//don't have an enemy, look for one
        // 	NPC_BSGrenadier_Patrol();
        // }
        // else//if ( (*NPC).enemy )
        // {//have an enemy
        // 	NPC_BSGrenadier_Attack();
        // }
    }
}

// Stub for Q_irand - will need real implementation
#[inline]
fn Q_irand(min: c_int, max: c_int) -> c_int {
    // Placeholder implementation
    min
}
