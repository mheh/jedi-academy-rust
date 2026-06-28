// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

// seems to be a compiler bug, it doesn't clean out the #ifdefs between dif-compiles
// or something, so the headers spew errors on these defs from the previous compile.
// this fixes that. -rww
// #ifdef _JK2MP
// get rid of all the crazy defs we added for this file
// #undef currentAngles
// #undef currentOrigin
// #undef mins
// #undef maxs
// #undef legsAnimTimer
// #undef torsoAnimTimer
// #undef bool
// #undef false
// #undef true
//
// #undef sqrtf
// #undef Q_flrand
//
// #undef MOD_EXPLOSIVE
// #endif
//
// #ifdef _JK2 //SP does not have this preprocessor for game like MP does
// #ifndef _JK2MP
// #define _JK2MP
// #endif
// #endif
//
// #ifndef _JK2MP //if single player
// #ifndef QAGAME //I don't think we have a QAGAME define
// #define QAGAME //but define it cause in sp we're always in the game
// #endif
// #endif
//
// #ifdef QAGAME //including game headers on cgame is FORBIDDEN ^_^
// #include "g_local.h"
// #elif defined _JK2MP
// #include "bg_public.h"
// #endif
//
// #ifndef _JK2MP
// #include "g_functions.h"
// #include "g_vehicles.h"
// #else
// #include "bg_vehicles.h"
// #endif
//
// #ifdef _JK2MP
// //this is really horrible, but it works! just be sure not to use any locals or anything
// //with these names (exluding bool, false, true). -rww
// #define currentAngles r.currentAngles
// #define currentOrigin r.currentOrigin
// #define mins r.mins
// #define maxs r.maxs
// #define legsAnimTimer legsTimer
// #define torsoAnimTimer torsoTimer
// #define bool qboolean
// #define false qfalse
// #define true qtrue
//
// #define sqrtf sqrt
// #define Q_flrand flrand
//
// #define MOD_EXPLOSIVE MOD_SUICIDE
// #else
// #define bgEntity_t gentity_t
// #endif

use core::ffi::{c_int, c_float, c_char};

// extern float DotToSpot( vec3_t spot, vec3_t from, vec3_t fromAngles );
extern "C" {
    pub fn DotToSpot(spot: *const f32, from: *const f32, fromAngles: *const f32) -> f32;
}

// #ifdef QAGAME //SP or gameside MP
// extern vmCvar_t	cg_thirdPersonAlpha;
// extern vec3_t playerMins;
// extern vec3_t playerMaxs;
// extern cvar_t	*g_speederControlScheme;
// extern void ChangeWeapon( gentity_t *ent, int newWeapon );
// extern void PM_SetAnim(pmove_t	*pm,int setAnimParts,int anim,int setAnimFlags, int blendTime);
// extern int PM_AnimLength( int index, animNumber_t anim );
// extern void G_VehicleTrace( trace_t *results, const vec3_t start, const vec3_t tMins, const vec3_t tMaxs, const vec3_t end, int passEntityNum, int contentmask );
// #endif

extern "C" {
    // pub static cg_thirdPersonAlpha: vmCvar_t;
    // pub static playerMins: [f32; 3];
    // pub static playerMaxs: [f32; 3];
    // pub static mut g_speederControlScheme: *mut cvar_t;
    pub fn ChangeWeapon(ent: *mut core::ffi::c_void, newWeapon: c_int);
    pub fn PM_SetAnim(pm: *mut core::ffi::c_void, setAnimParts: c_int, anim: c_int, setAnimFlags: c_int, blendTime: c_int);
    pub fn PM_AnimLength(index: c_int, anim: c_int) -> c_int;
    pub fn G_VehicleTrace(results: *mut core::ffi::c_void, start: *const f32, tMins: *const f32, tMaxs: *const f32, end: *const f32, passEntityNum: c_int, contentmask: c_int);
}

// extern qboolean BG_UnrestrainedPitchRoll( playerState_t *ps, Vehicle_t *pVeh );

extern "C" {
    pub fn BG_UnrestrainedPitchRoll(ps: *mut core::ffi::c_void, pVeh: *mut core::ffi::c_void) -> c_int;
}

// #ifdef _JK2MP
//
// #include "../namespace_begin.h"
//
// extern void BG_SetAnim(playerState_t *ps, animation_t *animations, int setAnimParts,int anim,int setAnimFlags, int blendTime);
// extern int BG_GetTime(void);
// #endif

extern "C" {
    pub fn BG_SetAnim(ps: *mut core::ffi::c_void, animations: *mut core::ffi::c_void, setAnimParts: c_int, anim: c_int, setAnimFlags: c_int, blendTime: c_int);
    pub fn BG_GetTime() -> c_int;
}

// extern void BG_ExternThisSoICanRecompileInDebug( Vehicle_t *pVeh, playerState_t *riderPS );

extern "C" {
    pub fn BG_ExternThisSoICanRecompileInDebug(pVeh: *mut core::ffi::c_void, riderPS: *mut core::ffi::c_void);
}

// //this stuff has got to be predicted, so..
// bool BG_FighterUpdate(Vehicle_t *pVeh, const usercmd_t *pUcmd, vec3_t trMins, vec3_t trMaxs, float gravity,
// 					  void (*traceFunc)( trace_t *results, const vec3_t start, const vec3_t lmins, const vec3_t lmaxs, const vec3_t end, int passEntityNum, int contentMask ))
// {

#[allow(non_snake_case)]
pub extern "C" fn BG_FighterUpdate(
    pVeh: *mut core::ffi::c_void,
    pUcmd: *const core::ffi::c_void,
    trMins: [f32; 3],
    trMaxs: [f32; 3],
    gravity: f32,
    traceFunc: unsafe extern "C" fn(*mut core::ffi::c_void, *const f32, *const f32, *const f32, *const f32, c_int, c_int),
) -> c_int {
    // vec3_t		bottom;
    // playerState_t *parentPS;
    // qboolean	isDead = qfalse;
    // #ifdef QAGAME //don't do this on client
    // 	// Make sure the riders are not visible or collidable.
    // 	pVeh->m_pVehicleInfo->Ghost( pVeh, pVeh->m_pPilot );
    // #endif
    //
    //
    // #ifdef _JK2MP
    // 	parentPS = pVeh->m_pParentEntity->playerState;
    // #else
    // 	parentPS = &pVeh->m_pParentEntity->client->ps;
    // #endif
    //
    // 	if (!parentPS)
    // 	{
    // 		Com_Error(ERR_DROP, "NULL PS in BG_FighterUpdate (%s)", pVeh->m_pVehicleInfo->name);
    // 		return false;
    // 	}
    //
    // 	// If we have a pilot, take out gravity (it's a flying craft...).
    // 	if ( pVeh->m_pPilot )
    // 	{
    // 		parentPS->gravity = 0;
    // #ifndef _JK2MP //don't need this flag in mp, I.. guess
    // 		pVeh->m_pParentEntity->svFlags |= SVF_CUSTOM_GRAVITY;
    // #endif
    // 	}
    // 	else
    // 	{
    // #ifndef _JK2MP //don't need this flag in mp, I.. guess
    // 		pVeh->m_pParentEntity->svFlags &= ~SVF_CUSTOM_GRAVITY;
    // #else //in MP set grav back to normal gravity
    // 		if (pVeh->m_pVehicleInfo->gravity)
    // 		{
    // 			parentPS->gravity = pVeh->m_pVehicleInfo->gravity;
    // 		}
    // 		else
    // 		{ //it doesn't have gravity specified apparently
    // 			parentPS->gravity = gravity;
    // 		}
    // #endif
    // 	}
    //
    // #ifdef _JK2MP
    // 	isDead = (qboolean)((parentPS->eFlags&EF_DEAD)!=0);
    // #else
    // 	isDead = (parentPS->stats[STAT_HEALTH] <= 0 );
    // #endif
    //
    // 	/*
    // 	if ( isDead ||
    // 		(pVeh->m_pVehicleInfo->surfDestruction &&
    // 			pVeh->m_iRemovedSurfaces ) )
    // 	{//can't land if dead or spiralling out of control
    // 		pVeh->m_LandTrace.fraction = 1.0f;
    // 		pVeh->m_LandTrace.contents = pVeh->m_LandTrace.surfaceFlags = 0;
    // 		VectorClear( pVeh->m_LandTrace.plane.normal );
    // 		pVeh->m_LandTrace.allsolid = qfalse;
    // 		pVeh->m_LandTrace.startsolid = qfalse;
    // 	}
    // 	else
    // 	{
    // 	*/
    // 	//argh, no, I need to have a way to see when they impact the ground while damaged. -rww
    //
    // 		// Check to see if the fighter has taken off yet (if it's a certain height above ground).
    // 		VectorCopy( parentPS->origin, bottom );
    // 		bottom[2] -= pVeh->m_pVehicleInfo->landingHeight;
    //
    // 		traceFunc( &pVeh->m_LandTrace, parentPS->origin, trMins, trMaxs, bottom, pVeh->m_pParentEntity->s.number, (MASK_NPCSOLID&~CONTENTS_BODY) );
    // 	//}
    //
    // 	return true;
    // }

    // Stub implementation - full body omitted for blind port marker
    0
}

// #ifdef QAGAME //ONLY in SP or on server, not cgame
//
// // Like a think or move command, this updates various vehicle properties.
// static bool Update( Vehicle_t *pVeh, const usercmd_t *pUcmd )
// {

#[allow(non_snake_case)]
#[cfg(feature = "qagame")]
fn Update(pVeh: *mut core::ffi::c_void, pUcmd: *const core::ffi::c_void) -> c_int {
    // assert(pVeh->m_pParentEntity);
    // if (!BG_FighterUpdate(pVeh, pUcmd, ((gentity_t *)pVeh->m_pParentEntity)->mins,
    // 	((gentity_t *)pVeh->m_pParentEntity)->maxs,
    // #ifdef _JK2MP
    // 	g_gravity.value,
    // #else
    // 	g_gravity->value,
    // #endif
    // 	G_VehicleTrace))
    // {
    // 	return false;
    // }
    //
    // if ( !g_vehicleInfo[VEHICLE_BASE].Update( pVeh, pUcmd ) )
    // {
    // 	return false;
    // }
    //
    // return true;
    // }

    // Stub implementation
    0
}

// // Board this Vehicle (get on). The first entity to board an empty vehicle becomes the Pilot.
// static bool Board( Vehicle_t *pVeh, bgEntity_t *pEnt )
// {

#[allow(non_snake_case)]
#[cfg(feature = "qagame")]
fn Board(pVeh: *mut core::ffi::c_void, pEnt: *mut core::ffi::c_void) -> c_int {
    // if ( !g_vehicleInfo[VEHICLE_BASE].Board( pVeh, pEnt ) )
    // 	return false;
    //
    // // Set the board wait time (they won't be able to do anything, including getting off, for this amount of time).
    // pVeh->m_iBoarding = level.time + 1500;
    //
    // return true;
    // }

    // Stub implementation
    0
}

// // Eject an entity from the vehicle.
// static bool Eject( Vehicle_t *pVeh, bgEntity_t *pEnt, qboolean forceEject )
// {

#[allow(non_snake_case)]
#[cfg(feature = "qagame")]
fn Eject(pVeh: *mut core::ffi::c_void, pEnt: *mut core::ffi::c_void, forceEject: c_int) -> c_int {
    // if ( g_vehicleInfo[VEHICLE_BASE].Eject( pVeh, pEnt, forceEject ) )
    // {
    // 	return true;
    // }
    //
    // return false;
    // }

    // Stub implementation
    0
}

// #endif //end game-side only

// //method of decrementing the given angle based on the given taking variable frame times into account
// static float PredictedAngularDecrement(float scale, float timeMod, float originalAngle)
// {

#[allow(non_snake_case)]
fn PredictedAngularDecrement(scale: f32, timeMod: f32, originalAngle: f32) -> f32 {
    // float fixedBaseDec = originalAngle*0.05f;
    let mut fixedBaseDec = originalAngle * 0.05f32;
    // float r = 0.0f;
    let mut r = 0.0f32;

    // if (fixedBaseDec < 0.0f)
    // {
    // 	fixedBaseDec = -fixedBaseDec;
    // }
    if fixedBaseDec < 0.0f32 {
        fixedBaseDec = -fixedBaseDec;
    }

    // fixedBaseDec *= (1.0f+(1.0f-scale));
    fixedBaseDec *= 1.0f32 + (1.0f32 - scale);

    // if (fixedBaseDec < 0.1f)
    // { //don't increment in incredibly small fractions, it would eat up unnecessary bandwidth.
    // 	fixedBaseDec = 0.1f;
    // }
    if fixedBaseDec < 0.1f32 {
        // don't increment in incredibly small fractions, it would eat up unnecessary bandwidth.
        fixedBaseDec = 0.1f32;
    }

    // fixedBaseDec *= (timeMod*0.1f);
    fixedBaseDec *= timeMod * 0.1f32;
    // if (originalAngle > 0.0f)
    // { //subtract
    // 	r = (originalAngle-fixedBaseDec);
    // 	if (r < 0.0f)
    // 	{
    // 		r = 0.0f;
    // 	}
    // }
    // else if (originalAngle < 0.0f)
    // { //add
    // 	r = (originalAngle+fixedBaseDec);
    // 	if (r > 0.0f)
    // 	{
    // 		r = 0.0f;
    // 	}
    // }
    if originalAngle > 0.0f32 {
        // subtract
        r = originalAngle - fixedBaseDec;
        if r < 0.0f32 {
            r = 0.0f32;
        }
    } else if originalAngle < 0.0f32 {
        // add
        r = originalAngle + fixedBaseDec;
        if r > 0.0f32 {
            r = 0.0f32;
        }
    }

    // return r;
    // }
    r
}

// #ifdef QAGAME//only do this check on GAME side, because if it's CGAME, it's being predicted, and it's only predicted if the local client is the driver
// qboolean FighterIsInSpace( gentity_t *gParent )
// {

#[allow(non_snake_case)]
#[cfg(feature = "qagame")]
fn FighterIsInSpace(gParent: *mut core::ffi::c_void) -> c_int {
    // if ( gParent
    // 	&& gParent->client
    // 	&& gParent->client->inSpaceIndex
    // 	&& gParent->client->inSpaceIndex < ENTITYNUM_WORLD )
    // {
    // 	return qtrue;
    // }
    // return qfalse;
    // }

    // Stub implementation
    0
}

// #endif

// qboolean FighterOverValidLandingSurface( Vehicle_t *pVeh )
// {

#[allow(non_snake_case)]
pub fn FighterOverValidLandingSurface(pVeh: *mut core::ffi::c_void) -> c_int {
    // if ( pVeh->m_LandTrace.fraction < 1.0f //ground present
    // 	&& pVeh->m_LandTrace.plane.normal[2] >= MIN_LANDING_SLOPE )//flat enough
    // 	//FIXME: also check for a certain surface flag ... "landing zones"?
    // {
    // 	return qtrue;
    // }
    // return qfalse;
    // }

    // Stub implementation
    0
}

// qboolean FighterIsLanded( Vehicle_t *pVeh, playerState_t *parentPS )
// {

#[allow(non_snake_case)]
pub fn FighterIsLanded(pVeh: *mut core::ffi::c_void, parentPS: *mut core::ffi::c_void) -> c_int {
    // if ( FighterOverValidLandingSurface( pVeh )
    // 	&& !parentPS->speed )//stopped
    // {
    // 	return qtrue;
    // }
    // return qfalse;
    // }

    // Stub implementation
    0
}

// qboolean FighterIsLanding( Vehicle_t *pVeh, playerState_t *parentPS )
// {
//
// 	if ( FighterOverValidLandingSurface( pVeh )
// #ifdef QAGAME//only do this check on GAME side, because if it's CGAME, it's being predicted, and it's only predicted if the local client is the driver
// 		&& pVeh->m_pVehicleInfo->Inhabited( pVeh )//has to have a driver in order to be capable of landing
// #endif
// 		&& (pVeh->m_ucmd.forwardmove < 0||pVeh->m_ucmd.upmove<0) //decelerating or holding crouch button
// 		&& parentPS->speed <= MIN_LANDING_SPEED )//going slow enough to start landing - was using pVeh->m_pVehicleInfo->speedIdle, but that's still too fast
// 	{
// 		return qtrue;
// 	}
// 	return qfalse;
// }

#[allow(non_snake_case)]
pub fn FighterIsLanding(pVeh: *mut core::ffi::c_void, parentPS: *mut core::ffi::c_void) -> c_int {
    // Stub implementation
    0
}

// qboolean FighterIsLaunching( Vehicle_t *pVeh, playerState_t *parentPS )
// {
//
// 	if ( FighterOverValidLandingSurface( pVeh )
// #ifdef QAGAME//only do this check on GAME side, because if it's CGAME, it's being predicted, and it's only predicted if the local client is the driver
// 		&& pVeh->m_pVehicleInfo->Inhabited( pVeh )//has to have a driver in order to be capable of landing
// #endif
// 		&& pVeh->m_ucmd.upmove > 0 //trying to take off
// 		&& parentPS->speed <= 200.0f )//going slow enough to start landing - was using pVeh->m_pVehicleInfo->speedIdle, but that's still too fast
// 	{
// 		return qtrue;
// 	}
// 	return qfalse;
// }

#[allow(non_snake_case)]
pub fn FighterIsLaunching(pVeh: *mut core::ffi::c_void, parentPS: *mut core::ffi::c_void) -> c_int {
    // Stub implementation
    0
}

// qboolean FighterSuspended( Vehicle_t *pVeh, playerState_t *parentPS )
// {
// #ifdef QAGAME//only do this check on GAME side, because if it's CGAME, it's being predicted, and it's only predicted if the local client is the driver
// 	if (!pVeh->m_pPilot//empty
// 		&& !parentPS->speed//not moving
// 		&& pVeh->m_ucmd.forwardmove <= 0//not trying to go forward for whatever reason
// 		&& pVeh->m_pParentEntity != NULL
// 		&& (((gentity_t *)pVeh->m_pParentEntity)->spawnflags&2) )//SUSPENDED spawnflag is on
// 	{
// 		return qtrue;
// 	}
// 	return qfalse;
// #elif CGAME
// 	return qfalse;
// #endif
// }

#[allow(non_snake_case)]
pub fn FighterSuspended(pVeh: *mut core::ffi::c_void, parentPS: *mut core::ffi::c_void) -> c_int {
    // Stub implementation
    0
}

// #ifdef CGAME
// extern void trap_S_StartSound( vec3_t origin, int entityNum, int entchannel, sfxHandle_t sfx ); //cg_syscalls.c
// extern sfxHandle_t trap_S_RegisterSound( const char *sample); //cg_syscalls.c
// #endif

extern "C" {
    // pub fn trap_S_StartSound(origin: *const f32, entityNum: c_int, entchannel: c_int, sfx: c_int);
    // pub fn trap_S_RegisterSound(sample: *const c_char) -> c_int;
}

// //MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
// //If you really need to violate this rule for SP, then use ifdefs.
// //By BG-compatible, I mean no use of game-specific data - ONLY use
// //stuff available in the MP bgEntity (in SP, the bgEntity is #defined
// //as a gentity, but the MP-compatible access restrictions are based
// //on the bgEntity structure in the MP codebase) -rww
// // ProcessMoveCommands the Vehicle.
// #define FIGHTER_MIN_TAKEOFF_FRACTION 0.7f

const FIGHTER_MIN_TAKEOFF_FRACTION: f32 = 0.7f32;

// static void ProcessMoveCommands( Vehicle_t *pVeh )
// {

#[allow(non_snake_case)]
fn ProcessMoveCommands(pVeh: *mut core::ffi::c_void) {
    // /************************************************************************************/
    // /*	BEGIN	Here is where we move the vehicle (forward or back or whatever). BEGIN	*/
    // /************************************************************************************/
    //
    // 	//Client sets ucmds and such for speed alterations
    // 	float speedInc, speedIdleDec, speedIdle, speedIdleAccel, speedMin, speedMax;
    // 	bgEntity_t *parent = pVeh->m_pParentEntity;
    // 	qboolean isLandingOrLaunching = qfalse;
    // #ifndef _JK2MP//SP
    // 	int curTime = level.time;
    // #elif QAGAME//MP GAME
    // 	int curTime = level.time;
    // #elif CGAME//MP CGAME
    // 	//FIXME: pass in ucmd?  Not sure if this is reliable...
    // 	int curTime = pm->cmd.serverTime;
    // #endif
    //
    // #ifdef _JK2MP
    // 	playerState_t *parentPS = parent->playerState;
    // #else
    // 	playerState_t *parentPS = &parent->client->ps;
    // #endif
    //
    // #ifdef _JK2MP
    // 	if ( parentPS->hyperSpaceTime
    // 		&& curTime - parentPS->hyperSpaceTime < HYPERSPACE_TIME )
    // 	{//Going to Hyperspace
    // 		//totally override movement
    // 		float timeFrac = ((float)(curTime-parentPS->hyperSpaceTime))/HYPERSPACE_TIME;
    // 		if ( timeFrac < HYPERSPACE_TELEPORT_FRAC )
    // 		{//for first half, instantly jump to top speed!
    // 			if ( !(parentPS->eFlags2&EF2_HYPERSPACE) )
    // 			{//waiting to face the right direction, do nothing
    // 				parentPS->speed = 0.0f;
    // 			}
    // 			else
    // 			{
    // 				if ( parentPS->speed < HYPERSPACE_SPEED )
    // 				{//just started hyperspace
    // //MIKE: This is going to play the sound twice for the predicting client, I suggest using
    // //a predicted event or only doing it game-side. -rich
    // #ifdef QAGAME//MP GAME-side
    // 					//G_EntitySound( ((gentity_t *)(pVeh->m_pParentEntity)), CHAN_LOCAL, pVeh->m_pVehicleInfo->soundHyper );
    // #elif CGAME//MP CGAME-side
    // 					trap_S_StartSound( NULL, pm->ps->clientNum, CHAN_LOCAL, pVeh->m_pVehicleInfo->soundHyper );
    // #endif
    // 				}
    //
    // 				parentPS->speed = HYPERSPACE_SPEED;
    // 			}
    // 		}
    // 		else
    // 		{//slow from top speed to 200...
    // 			parentPS->speed = 200.0f + ((1.0f-timeFrac)*(1.0f/HYPERSPACE_TELEPORT_FRAC)*(HYPERSPACE_SPEED-200.0f));
    // 			//don't mess with acceleration, just pop to the high velocity
    // 			if ( VectorLength( parentPS->velocity ) < parentPS->speed )
    // 			{
    // 				VectorScale( parentPS->moveDir, parentPS->speed, parentPS->velocity );
    // 			}
    // 		}
    // 		return;
    // 	}
    // #endif
    //
    // 	if ( pVeh->m_iDropTime >= curTime )
    // 	{//no speed, just drop
    // 		parentPS->speed = 0.0f;
    // 		parentPS->gravity = 800;
    // 		return;
    // 	}
    //
    // 	isLandingOrLaunching = (FighterIsLanding( pVeh, parentPS )||FighterIsLaunching( pVeh, parentPS ));
    //
    // ... (continued processing logic omitted)
    //
    // /********************************************************************************/
    // /*	END Here is where we move the vehicle (forward or back or whatever). END	*/
    // /********************************************************************************/
    // }

    // Stub implementation - full ProcessMoveCommands body omitted
}

// extern void BG_VehicleTurnRateForSpeed( Vehicle_t *pVeh, float speed, float *mPitchOverride, float *mYawOverride );

extern "C" {
    pub fn BG_VehicleTurnRateForSpeed(pVeh: *mut core::ffi::c_void, speed: f32, mPitchOverride: *mut f32, mYawOverride: *mut f32);
}

// static void FighterWingMalfunctionCheck( Vehicle_t *pVeh, playerState_t *parentPS )
// {

#[allow(non_snake_case)]
fn FighterWingMalfunctionCheck(pVeh: *mut core::ffi::c_void, parentPS: *mut core::ffi::c_void) {
    // float mPitchOverride = 1.0f;
    // float mYawOverride = 1.0f;
    // BG_VehicleTurnRateForSpeed( pVeh, parentPS->speed, &mPitchOverride, &mYawOverride );
    // //check right wing damage
    // if ( (parentPS->brokenLimbs&(1<<SHIPSURF_DAMAGE_RIGHT_HEAVY)) )
    // {//right wing has taken heavy damage
    // 	pVeh->m_vOrientation[ROLL] += (sin( pVeh->m_ucmd.serverTime*0.001 )+1.0f)*pVeh->m_fTimeModifier*mYawOverride*50.0f;
    // }
    // else if ( (parentPS->brokenLimbs&(1<<SHIPSURF_DAMAGE_RIGHT_LIGHT)) )
    // {//right wing has taken light damage
    // 	pVeh->m_vOrientation[ROLL] += (sin( pVeh->m_ucmd.serverTime*0.001 )+1.0f)*pVeh->m_fTimeModifier*mYawOverride*12.5f;
    // }
    //
    // //check left wing damage
    // if ( (parentPS->brokenLimbs&(1<<SHIPSURF_DAMAGE_LEFT_HEAVY)) )
    // {//left wing has taken heavy damage
    // 	pVeh->m_vOrientation[ROLL] -= (sin( pVeh->m_ucmd.serverTime*0.001 )+1.0f)*pVeh->m_fTimeModifier*mYawOverride*50.0f;
    // }
    // else if ( (parentPS->brokenLimbs&(1<<SHIPSURF_DAMAGE_LEFT_LIGHT)) )
    // {//left wing has taken light damage
    // 	pVeh->m_vOrientation[ROLL] -= (sin( pVeh->m_ucmd.serverTime*0.001 )+1.0f)*pVeh->m_fTimeModifier*mYawOverride*12.5f;
    // }
    //
    // }

    // Stub implementation
}

// static void FighterNoseMalfunctionCheck( Vehicle_t *pVeh, playerState_t *parentPS )
// {

#[allow(non_snake_case)]
fn FighterNoseMalfunctionCheck(pVeh: *mut core::ffi::c_void, parentPS: *mut core::ffi::c_void) {
    // float mPitchOverride = 1.0f;
    // float mYawOverride = 1.0f;
    // BG_VehicleTurnRateForSpeed( pVeh, parentPS->speed, &mPitchOverride, &mYawOverride );
    // //check nose damage
    // if ( (parentPS->brokenLimbs&(1<<SHIPSURF_DAMAGE_FRONT_HEAVY)) )
    // {//nose has taken heavy damage
    // 	//pitch up and down over time
    // 	pVeh->m_vOrientation[PITCH] += sin( pVeh->m_ucmd.serverTime*0.001 )*pVeh->m_fTimeModifier*mPitchOverride*50.0f;
    // }
    // else if ( (parentPS->brokenLimbs&(1<<SHIPSURF_DAMAGE_FRONT_LIGHT)) )
    // {//nose has taken heavy damage
    // 	//pitch up and down over time
    // 	pVeh->m_vOrientation[PITCH] += sin( pVeh->m_ucmd.serverTime*0.001 )*pVeh->m_fTimeModifier*mPitchOverride*20.0f;
    // }
    // }

    // Stub implementation
}

// static void FighterDamageRoutine( Vehicle_t *pVeh, bgEntity_t *parent, playerState_t *parentPS, playerState_t *riderPS, qboolean isDead )
// {

#[allow(non_snake_case)]
fn FighterDamageRoutine(pVeh: *mut core::ffi::c_void, parent: *mut core::ffi::c_void, parentPS: *mut core::ffi::c_void, riderPS: *mut core::ffi::c_void, isDead: c_int) {
    // Implementation stub - full body omitted
}

// #ifdef _JK2MP
// void FighterYawAdjust(Vehicle_t *pVeh, playerState_t *riderPS, playerState_t *parentPS)
// {

#[allow(non_snake_case)]
#[cfg(feature = "jk2mp")]
fn FighterYawAdjust(pVeh: *mut core::ffi::c_void, riderPS: *mut core::ffi::c_void, parentPS: *mut core::ffi::c_void) {
    // Stub implementation
}

// void FighterPitchAdjust(Vehicle_t *pVeh, playerState_t *riderPS, playerState_t *parentPS)
// {

#[allow(non_snake_case)]
#[cfg(feature = "jk2mp")]
fn FighterPitchAdjust(pVeh: *mut core::ffi::c_void, riderPS: *mut core::ffi::c_void, parentPS: *mut core::ffi::c_void) {
    // Stub implementation
}

// #endif

// //MP RULE - ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
// //If you really need to violate this rule for SP, then use ifdefs.
// //By BG-compatible, I mean no use of game-specific data - ONLY use
// //stuff available in the MP bgEntity (in SP, the bgEntity is #defined
// //as a gentity, but the MP-compatible access restrictions are based
// //on the bgEntity structure in the MP codebase) -rww
// // ProcessOrientCommands the Vehicle.
// static void ProcessOrientCommands( Vehicle_t *pVeh )
// {

#[allow(non_snake_case)]
fn ProcessOrientCommands(pVeh: *mut core::ffi::c_void) {
    // /********************************************************************************/
    // /*	BEGIN	Here is where make sure the vehicle is properly oriented.	BEGIN	*/
    // /********************************************************************************/
    //
    // ... (extensive implementation omitted)
    //
    // /********************************************************************************/
    // /*	END	Here is where make sure the vehicle is properly oriented.	END			*/
    // /********************************************************************************/
    // }

    // Stub implementation
}

// #ifdef QAGAME //ONLY in SP or on server, not cgame
//
// extern void PM_SetAnim(pmove_t	*pm,int setAnimParts,int anim,int setAnimFlags, int blendTime);
//
// // This function makes sure that the vehicle is properly animated.
// static void AnimateVehicle( Vehicle_t *pVeh )
// {

#[allow(non_snake_case)]
#[cfg(feature = "qagame")]
fn AnimateVehicle(pVeh: *mut core::ffi::c_void) {
    // Implementation stub - full body omitted
}

// // This function makes sure that the rider's in this vehicle are properly animated.
// static void AnimateRiders( Vehicle_t *pVeh )
// {
// }

#[allow(non_snake_case)]
#[cfg(feature = "qagame")]
fn AnimateRiders(pVeh: *mut core::ffi::c_void) {
    // Empty implementation in original
}

// #endif //game-only
//
// #ifndef QAGAME
// void AttachRidersGeneric( Vehicle_t *pVeh );
// #endif

extern "C" {
    // #[cfg(not(feature = "qagame"))]
    pub fn AttachRidersGeneric(pVeh: *mut core::ffi::c_void);
}

// void G_SetFighterVehicleFunctions( vehicleInfo_t *pVehInfo )
// {

#[allow(non_snake_case)]
pub extern "C" fn G_SetFighterVehicleFunctions(pVehInfo: *mut core::ffi::c_void) {
    // #ifdef QAGAME //ONLY in SP or on server, not cgame
    // 	pVehInfo->AnimateVehicle			=		AnimateVehicle;
    // 	pVehInfo->AnimateRiders				=		AnimateRiders;
    // //	pVehInfo->ValidateBoard				=		ValidateBoard;
    // //	pVehInfo->SetParent					=		SetParent;
    // //	pVehInfo->SetPilot					=		SetPilot;
    // //	pVehInfo->AddPassenger				=		AddPassenger;
    // //	pVehInfo->Animate					=		Animate;
    // 	pVehInfo->Board						=		Board;
    // 	pVehInfo->Eject						=		Eject;
    // //	pVehInfo->EjectAll					=		EjectAll;
    // //	pVehInfo->StartDeathDelay			=		StartDeathDelay;
    // //	pVehInfo->DeathUpdate				=		DeathUpdate;
    // //	pVehInfo->RegisterAssets			=		RegisterAssets;
    // //	pVehInfo->Initialize				=		Initialize;
    // 	pVehInfo->Update					=		Update;
    // //	pVehInfo->UpdateRider				=		UpdateRider;
    // #endif //game-only
    // 	pVehInfo->ProcessMoveCommands		=		ProcessMoveCommands;
    // 	pVehInfo->ProcessOrientCommands		=		ProcessOrientCommands;
    //
    // #ifndef QAGAME //cgame prediction attachment func
    // 	pVehInfo->AttachRiders				=		AttachRidersGeneric;
    // #endif
    // //	pVehInfo->AttachRiders				=		AttachRiders;
    // //	pVehInfo->Ghost						=		Ghost;
    // //	pVehInfo->UnGhost					=		UnGhost;
    // //	pVehInfo->Inhabited					=		Inhabited;
    // }

    // Stub implementation - function pointer assignment would occur here
}

// // Following is only in game, not in namespace
// #ifdef _JK2MP
// #include "../namespace_end.h"
// #endif
//
// #ifdef QAGAME
// extern void G_AllocateVehicleObject(Vehicle_t **pVeh);
// #endif
//
// #ifdef _JK2MP
// #include "../namespace_begin.h"
// #endif

extern "C" {
    // #[cfg(feature = "qagame")]
    pub fn G_AllocateVehicleObject(pVeh: *mut *mut core::ffi::c_void);
    pub fn BG_Alloc(size: usize) -> *mut core::ffi::c_void;
    // pub fn gi_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut core::ffi::c_void;
}

// extern void BG_VehicleGetIndex( const char *str );
extern "C" {
    pub fn BG_VehicleGetIndex(str_type: *const c_char) -> c_int;
    // pub static mut g_vehicleInfo: *mut core::ffi::c_void;
}

// // Create/Allocate a new Animal Vehicle (initializing it as well).
// void G_CreateFighterNPC( Vehicle_t **pVeh, const char *strType )
// {

#[allow(non_snake_case)]
pub extern "C" fn G_CreateFighterNPC(pVeh: *mut *mut core::ffi::c_void, strType: *const c_char) {
    // // Allocate the Vehicle.
    // #ifdef _JK2MP
    // #ifdef QAGAME
    // 	//these will remain on entities on the client once allocated because the pointer is
    // 	//never stomped. on the server, however, when an ent is freed, the entity struct is
    // 	//memset to 0, so this memory would be lost..
    //     G_AllocateVehicleObject(pVeh);
    // #else
    // 	if (!*pVeh)
    // 	{ //only allocate a new one if we really have to
    // 		(*pVeh) = (Vehicle_t *) BG_Alloc( sizeof(Vehicle_t) );
    // 	}
    // #endif
    // 	memset(*pVeh, 0, sizeof(Vehicle_t));
    // #else
    // 	(*pVeh) = (Vehicle_t *) gi.Malloc( sizeof(Vehicle_t), TAG_G_ALLOC, qtrue );
    // #endif
    // 	(*pVeh)->m_pVehicleInfo = &g_vehicleInfo[BG_VehicleGetIndex( strType )];
    // }

    // Stub implementation - full body omitted
}

// #ifdef _JK2MP
//
// #include "../namespace_end.h"
//
// //get rid of all the crazy defs we added for this file
// #undef currentAngles
// #undef currentOrigin
// #undef mins
// #undef maxs
// #undef legsAnimTimer
// #undef torsoAnimTimer
// #undef bool
// #undef false
// #undef true
//
// #undef sqrtf
// #undef Q_flrand
//
// #undef MOD_EXPLOSIVE
// #endif
