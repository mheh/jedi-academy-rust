//NPC_senses.cpp

// leave this line at the top for all NPC_xxxx.cpp files...
// #include "g_headers.h"

use core::ffi::{c_int, c_uint};

// #include "b_local.h"
// #ifdef _DEBUG
// 	#include <float.h>
// #endif

extern "C" {
	pub static mut eventClearTime: c_int;
}

/*
qboolean G_ClearLineOfSight(const vec3_t point1, const vec3_t point2, int ignore, int clipmask)

returns true if can see from point 1 to 2, even through glass (1 pane)- doesn't work with portals
*/
pub unsafe extern "C" fn G_ClearLineOfSight(point1: *const [f32; 3], point2: *const [f32; 3], ignore: c_int, clipmask: c_int) -> c_uint
{
	let mut tr: trace_t = core::mem::zeroed();

	gi.trace ( &mut tr, point1, core::ptr::null(), core::ptr::null(), point2, ignore, clipmask );
	if tr.fraction == 1.0
	{
		return qtrue;
	}

	let hit = &mut g_entities[ tr.entityNum as usize ];
	if EntIsGlass(hit)
	{
		let mut newpoint1: [f32; 3] = [0.0; 3];
		VectorCopy(tr.endpos, &mut newpoint1);
		gi.trace (&mut tr, &newpoint1, core::ptr::null(), core::ptr::null(), point2, (*hit).s.number, clipmask );

		if tr.fraction == 1.0
		{
			return qtrue;
		}
	}

	return qfalse;
}

/*
CanSee
determine if NPC can see an entity

This is a straight line trace check.  This function does not look at PVS or FOV,
or take any AI related factors (for example, the NPC's reaction time) into account

FIXME do we need fat and thin version of this?
*/
pub unsafe extern "C" fn CanSee ( ent: *mut gentity_t ) -> c_uint
{
	let mut tr: trace_t = core::mem::zeroed();
	let mut eyes: [f32; 3] = [0.0; 3];
	let mut spot: [f32; 3] = [0.0; 3];

	CalcEntitySpot( NPC, SPOT_HEAD_LEAN, &mut eyes );

	CalcEntitySpot( ent, SPOT_ORIGIN, &mut spot );
	gi.trace ( &mut tr, &eyes, core::ptr::null(), core::ptr::null(), &spot, (*NPC).s.number, MASK_OPAQUE );
	ShotThroughGlass (&mut tr, ent, &spot, MASK_OPAQUE);
	if tr.fraction == 1.0
	{
		return qtrue;
	}

	CalcEntitySpot( ent, SPOT_HEAD, &mut spot );
	gi.trace ( &mut tr, &eyes, core::ptr::null(), core::ptr::null(), &spot, (*NPC).s.number, MASK_OPAQUE );
	ShotThroughGlass (&mut tr, ent, &spot, MASK_OPAQUE);
	if tr.fraction == 1.0
	{
		return qtrue;
	}

	CalcEntitySpot( ent, SPOT_LEGS, &mut spot );
	gi.trace ( &mut tr, &eyes, core::ptr::null(), core::ptr::null(), &spot, (*NPC).s.number, MASK_OPAQUE );
	ShotThroughGlass (&mut tr, ent, &spot, MASK_OPAQUE);
	if tr.fraction == 1.0
	{
		return qtrue;
	}

	return qfalse;
}

pub unsafe extern "C" fn InFront( spot: *const [f32; 3], from: *const [f32; 3], fromAngles: *const [f32; 3], mut threshHold: f32 ) -> c_uint
{
	let mut dir: [f32; 3] = [0.0; 3];
	let mut forward: [f32; 3] = [0.0; 3];
	let mut angles: [f32; 3] = [0.0; 3];
	let dot: f32;

	VectorSubtract( spot, from, &mut dir );
	dir[2] = 0.0;
	VectorNormalize( &mut dir );

	VectorCopy( fromAngles, &mut angles );
	angles[0] = 0.0;
	AngleVectors( &angles, &mut forward, core::ptr::null_mut(), core::ptr::null_mut() );

	dot = DotProduct( &dir, &forward );

	return if (dot > threshHold) { qtrue } else { qfalse };
}

pub unsafe extern "C" fn DotToSpot( spot: *const [f32; 3], from: *const [f32; 3], fromAngles: *const [f32; 3] ) -> f32
{
	let mut dir: [f32; 3] = [0.0; 3];
	let mut forward: [f32; 3] = [0.0; 3];
	let mut angles: [f32; 3] = [0.0; 3];
	let dot: f32;

	VectorSubtract( spot, from, &mut dir );
	dir[2] = 0.0;
	VectorNormalize( &mut dir );

	VectorCopy( fromAngles, &mut angles );
	angles[0] = 0.0;
	AngleVectors( &angles, &mut forward, core::ptr::null_mut(), core::ptr::null_mut() );

	dot = DotProduct( &dir, &forward );

	return dot;
}
/*
InFOV

IDEA: further off to side of FOV range, higher chance of failing even if technically in FOV,
	keep core of 50% to sides as always succeeding
*/

//Position compares

pub unsafe extern "C" fn InFOV_spot_angles( spot: *const [f32; 3], from: *const [f32; 3], fromAngles: *const [f32; 3], hFOV: c_int, vFOV: c_int ) -> c_uint
{
	let mut deltaVector: [f32; 3] = [0.0; 3];
	let mut angles: [f32; 3] = [0.0; 3];
	let mut deltaAngles: [f32; 3] = [0.0; 3];

	VectorSubtract ( spot, from, &mut deltaVector );
	vectoangles ( &deltaVector, &mut angles );

	deltaAngles[PITCH as usize]	= AngleDelta ( (*fromAngles)[PITCH as usize], angles[PITCH as usize] );
	deltaAngles[YAW as usize]	= AngleDelta ( (*fromAngles)[YAW as usize], angles[YAW as usize] );

	if ( (deltaAngles[PITCH as usize]).abs() <= vFOV as f32 && (deltaAngles[YAW as usize]).abs() <= hFOV as f32 )
	{
		return qtrue;
	}

	return qfalse;
}

//NPC to position

pub unsafe extern "C" fn InFOV_entity_spot( origin: *const [f32; 3], from: *mut gentity_t, hFOV: c_int, vFOV: c_int ) -> c_uint
{
	let mut fromAngles: [f32; 3] = [0.0; 3];
	let mut eyes: [f32; 3] = [0.0; 3];

	if (*from).client != core::ptr::null_mut()
	{
		VectorCopy(&(*(*from).client).ps.viewangles, &mut fromAngles);
	}
	else
	{
		VectorCopy(&(*from).s.angles, &mut fromAngles);
	}

	CalcEntitySpot( from, SPOT_HEAD, &mut eyes );

	return InFOV_spot_angles( origin, &eyes, &fromAngles, hFOV, vFOV );
}

//Entity to entity
pub unsafe extern "C" fn InFOVFromPlayerView ( ent: *mut gentity_t, hFOV: c_int, vFOV: c_int ) -> c_uint
{
	let mut eyes: [f32; 3] = [0.0; 3];
	let mut spot: [f32; 3] = [0.0; 3];
	let mut deltaVector: [f32; 3] = [0.0; 3];
	let mut angles: [f32; 3] = [0.0; 3];
	let mut fromAngles: [f32; 3] = [0.0; 3];
	let mut deltaAngles: [f32; 3] = [0.0; 3];

	if player == core::ptr::null_mut() || (*player).client == core::ptr::null_mut()
	{
		return qfalse;
	}
	if cg.time != 0
	{
		VectorCopy( &cg.refdefViewAngles, &mut fromAngles );
	}
	else
	{
		VectorCopy( &(*(*player).client).ps.viewangles, &mut fromAngles );
	}

	if cg.time != 0
	{
		VectorCopy( &cg.refdef.vieworg, &mut eyes );
	}
	else
	{
		CalcEntitySpot( player, SPOT_HEAD_LEAN, &mut eyes );
	}

	CalcEntitySpot( ent, SPOT_ORIGIN, &mut spot );
	VectorSubtract ( &spot, &eyes, &mut deltaVector);

	vectoangles ( &deltaVector, &mut angles );
	deltaAngles[PITCH as usize] = AngleDelta ( fromAngles[PITCH as usize], angles[PITCH as usize] );
	deltaAngles[YAW as usize] = AngleDelta ( fromAngles[YAW as usize], angles[YAW as usize] );
	if ( (deltaAngles[PITCH as usize]).abs() <= vFOV as f32 && (deltaAngles[YAW as usize]).abs() <= hFOV as f32 )
	{
		return qtrue;
	}

	CalcEntitySpot( ent, SPOT_HEAD, &mut spot );
	VectorSubtract ( &spot, &eyes, &mut deltaVector);
	vectoangles ( &deltaVector, &mut angles );
	deltaAngles[PITCH as usize] = AngleDelta ( fromAngles[PITCH as usize], angles[PITCH as usize] );
	deltaAngles[YAW as usize] = AngleDelta ( fromAngles[YAW as usize], angles[YAW as usize] );
	if ( (deltaAngles[PITCH as usize]).abs() <= vFOV as f32 && (deltaAngles[YAW as usize]).abs() <= hFOV as f32 )
	{
		return qtrue;
	}

	CalcEntitySpot( ent, SPOT_LEGS, &mut spot );
	VectorSubtract ( &spot, &eyes, &mut deltaVector);
	vectoangles ( &deltaVector, &mut angles );
	deltaAngles[PITCH as usize] = AngleDelta ( fromAngles[PITCH as usize], angles[PITCH as usize] );
	deltaAngles[YAW as usize] = AngleDelta ( fromAngles[YAW as usize], angles[YAW as usize] );
	if ( (deltaAngles[PITCH as usize]).abs() <= vFOV as f32 && (deltaAngles[YAW as usize]).abs() <= hFOV as f32 )
	{
		return qtrue;
	}

	return qfalse;
}

pub unsafe extern "C" fn InFOV ( ent: *mut gentity_t, from: *mut gentity_t, hFOV: c_int, vFOV: c_int ) -> c_uint
{
	let mut eyes: [f32; 3] = [0.0; 3];
	let mut spot: [f32; 3] = [0.0; 3];
	let mut deltaVector: [f32; 3] = [0.0; 3];
	let mut angles: [f32; 3] = [0.0; 3];
	let mut fromAngles: [f32; 3] = [0.0; 3];
	let mut deltaAngles: [f32; 3] = [0.0; 3];

	if (*from).client != core::ptr::null_mut()
	{
		if (*(*from).client).NPC_class != CLASS_RANCOR
			&& (*(*from).client).NPC_class != CLASS_WAMPA
			&& !VectorCompare( &(*(*from).client).renderInfo.eyeAngles, &vec3_origin )
		{//Actual facing of tag_head!
			//NOTE: Stasis aliens may have a problem with this?
			VectorCopy( &(*(*from).client).renderInfo.eyeAngles, &mut fromAngles );
		}
		else
		{
			VectorCopy( &(*(*from).client).ps.viewangles, &mut fromAngles );
		}
	}
	else
	{
		VectorCopy(&(*from).s.angles, &mut fromAngles);
	}

	CalcEntitySpot( from, SPOT_HEAD_LEAN, &mut eyes );

	CalcEntitySpot( ent, SPOT_ORIGIN, &mut spot );
	VectorSubtract ( &spot, &eyes, &mut deltaVector);

	vectoangles ( &deltaVector, &mut angles );
	deltaAngles[PITCH as usize] = AngleDelta ( fromAngles[PITCH as usize], angles[PITCH as usize] );
	deltaAngles[YAW as usize] = AngleDelta ( fromAngles[YAW as usize], angles[YAW as usize] );
	if ( (deltaAngles[PITCH as usize]).abs() <= vFOV as f32 && (deltaAngles[YAW as usize]).abs() <= hFOV as f32 )
	{
		return qtrue;
	}

	CalcEntitySpot( ent, SPOT_HEAD, &mut spot );
	VectorSubtract ( &spot, &eyes, &mut deltaVector);
	vectoangles ( &deltaVector, &mut angles );
	deltaAngles[PITCH as usize] = AngleDelta ( fromAngles[PITCH as usize], angles[PITCH as usize] );
	deltaAngles[YAW as usize] = AngleDelta ( fromAngles[YAW as usize], angles[YAW as usize] );
	if ( (deltaAngles[PITCH as usize]).abs() <= vFOV as f32 && (deltaAngles[YAW as usize]).abs() <= hFOV as f32 )
	{
		return qtrue;
	}

	CalcEntitySpot( ent, SPOT_LEGS, &mut spot );
	VectorSubtract ( &spot, &eyes, &mut deltaVector);
	vectoangles ( &deltaVector, &mut angles );
	deltaAngles[PITCH as usize] = AngleDelta ( fromAngles[PITCH as usize], angles[PITCH as usize] );
	deltaAngles[YAW as usize] = AngleDelta ( fromAngles[YAW as usize], angles[YAW as usize] );
	if ( (deltaAngles[PITCH as usize]).abs() <= vFOV as f32 && (deltaAngles[YAW as usize]).abs() <= hFOV as f32 )
	{
		return qtrue;
	}

	return qfalse;
}

pub unsafe extern "C" fn InVisrange ( ent: *mut gentity_t ) -> c_uint
{//FIXME: make a calculate visibility for ents that takes into account
	//lighting, movement, turning, crouch/stand up, other anims, hide brushes, etc.
	let mut eyes: [f32; 3] = [0.0; 3];
	let mut spot: [f32; 3] = [0.0; 3];
	let mut deltaVector: [f32; 3] = [0.0; 3];
	let visrange = ((*NPCInfo).stats.visrange*(*NPCInfo).stats.visrange);

	CalcEntitySpot( NPC, SPOT_HEAD_LEAN, &mut eyes );

	CalcEntitySpot( ent, SPOT_ORIGIN, &mut spot );
	VectorSubtract ( &spot, &eyes, &mut deltaVector);

	/*if(ent->client)
	{
		float	vel, avel;
		if(ent->client->ps.velocity[0] || ent->client->ps.velocity[1] || ent->client->ps.velocity[2])
		{
			vel = VectorLength(ent->client->ps.velocity);
			if(vel > 128)
			{
				visrange += visrange * (vel/256);
			}
		}

		if(ent->avelocity[0] || ent->avelocity[1] || ent->avelocity[2])
		{//FIXME: shouldn't they need to have line of sight to you to detect this?
			avel = VectorLength(ent->avelocity);
			if(avel > 15)
			{
				visrange += visrange * (avel/60);
			}
		}
	}*/

	if VectorLengthSquared(&deltaVector) > visrange
	{
		return qfalse;
	}

	return qtrue;
}

/*
NPC_CheckVisibility
*/

pub unsafe extern "C" fn NPC_CheckVisibility ( ent: *mut gentity_t, flags: c_int ) -> visibility_t
{
	// flags should never be 0
	if flags == 0
	{
		return VIS_NOT;
	}

	// check PVS
	if ( flags & CHECK_PVS as c_int != 0 )
	{
		if !gi.inPVS ( (*ent).currentOrigin, (*NPC).currentOrigin )
		{
			return VIS_NOT;
		}
	}
	if !( (flags & (CHECK_360|CHECK_FOV|CHECK_SHOOT) as c_int) != 0 )
	{
		return VIS_PVS;
	}

	// check within visrange
	if (flags & CHECK_VISRANGE as c_int) != 0
	{
		if !InVisrange ( ent ) != 0
		{
			return VIS_PVS;
		}
	}

	// check 360 degree visibility
	//Meaning has to be a direct line of site
	if ( flags & CHECK_360 as c_int != 0 )
	{
		if !CanSee ( ent ) != 0
		{
			return VIS_PVS;
		}
	}
	if !( (flags & (CHECK_FOV|CHECK_SHOOT) as c_int) != 0 )
	{
		return VIS_360;
	}

	// check FOV
	if ( flags & CHECK_FOV as c_int != 0 )
	{
		if !InFOV ( ent, NPC, (*NPCInfo).stats.hfov, (*NPCInfo).stats.vfov) != 0
		{
			return VIS_360;
		}
	}

	if !( (flags & CHECK_SHOOT as c_int) != 0 )
	{
		return VIS_FOV;
	}

	// check shootability
	if ( flags & CHECK_SHOOT as c_int != 0 )
	{
		if !CanShoot ( ent, NPC ) != 0
		{
			return VIS_FOV;
		}
	}

	return VIS_SHOOT;
}

/*
-------------------------
NPC_CheckSoundEvents
-------------------------
*/
static unsafe fn G_CheckSoundEvents( self_: *mut gentity_t, maxHearDist: f32, ignoreAlert: c_int, mustHaveOwner: c_uint, minAlertLevel: c_int, onGroundOnly: c_uint ) -> c_int
{
	let mut bestEvent: c_int = -1;
	let mut bestAlert: c_int = -1;
	let mut bestTime: c_int = -1;
	let dist: f32;
	let radius: f32;

	let maxHearDist_sq = maxHearDist * maxHearDist;

	for i in 0..level.numAlertEvents
	{
		//are we purposely ignoring this alert?
		if level.alertEvents[i as usize].ID == ignoreAlert as c_uint
		{
			continue;
		}
		//We're only concerned about sounds
		if level.alertEvents[i as usize].type_ != AET_SOUND as c_int
		{
			continue;
		}
		//must be at least this noticable
		if level.alertEvents[i as usize].level < minAlertLevel
		{
			continue;
		}
		//must have an owner?
		if mustHaveOwner != 0 && level.alertEvents[i as usize].owner == core::ptr::null_mut()
		{
			continue;
		}
		//must be on the ground?
		if onGroundOnly != 0 && level.alertEvents[i as usize].onGround == 0
		{
			continue;
		}

		//Must be within range
		let dist = DistanceSquared( level.alertEvents[i as usize].position, (*self_).currentOrigin );

		//can't hear it
		if dist > maxHearDist_sq
		{
			continue;
		}

		if (*self_).client != core::ptr::null_mut() && (*(*self_).client).NPC_class != CLASS_SAND_CREATURE as c_int
		{//sand creatures hear all in within their earshot, regardless of quietness and alert sound radius!
			let radius = level.alertEvents[i as usize].radius * level.alertEvents[i as usize].radius;
			if dist > radius
			{
				continue;
			}

			if level.alertEvents[i as usize].addLight != 0
			{//a quiet sound, must have LOS to hear it
				if G_ClearLOS( self_, level.alertEvents[i as usize].position ) == qfalse
				{//no LOS, didn't hear it
					continue;
				}
			}
		}

		//See if this one takes precedence over the previous one
		if level.alertEvents[i as usize].level >= bestAlert //higher alert level
			|| (level.alertEvents[i as usize].level==bestAlert&&level.alertEvents[i as usize].timestamp >= bestTime) //same alert level, but this one is newer
		{//NOTE: equal is better because it's later in the array
			bestEvent = i as c_int;
			bestAlert = level.alertEvents[i as usize].level;
			bestTime = level.alertEvents[i as usize].timestamp;
		}
	}

	return bestEvent;
}

pub unsafe extern "C" fn G_GetLightLevel( pos: *const [f32; 3], fromDir: *const [f32; 3] ) -> f32
{
	let mut ambient: [f32; 3] = [0.0; 3];
	let mut directed: [f32; 3] = [0.0; 3];
	let mut lightDir: [f32; 3] = [0.0; 3];
	let lightLevel: f32;

	cgi_R_GetLighting( pos, &mut ambient, &mut directed, &mut lightDir );

	lightLevel = VectorLength( &ambient ) + (VectorLength( &directed )*DotProduct( &lightDir, fromDir ));

	return lightLevel;
}
/*
-------------------------
NPC_CheckSightEvents
-------------------------
*/
static unsafe fn G_CheckSightEvents( self_: *mut gentity_t, hFOV: c_int, vFOV: c_int, maxSeeDist: f32, ignoreAlert: c_int, mustHaveOwner: c_uint, minAlertLevel: c_int ) -> c_int
{
	let mut bestEvent: c_int = -1;
	let mut bestAlert: c_int = -1;
	let mut bestTime: c_int = -1;
	let dist: f32;
	let radius: f32;

	let maxSeeDist_sq = maxSeeDist * maxSeeDist;
	for i in 0..level.numAlertEvents
	{
		//are we purposely ignoring this alert?
		if level.alertEvents[i as usize].ID == ignoreAlert as c_uint
		{
			continue;
		}
		//We're only concerned about sounds
		if level.alertEvents[i as usize].type_ != AET_SIGHT as c_int
		{
			continue;
		}
		//must be at least this noticable
		if level.alertEvents[i as usize].level < minAlertLevel
		{
			continue;
		}
		//must have an owner?
		if mustHaveOwner != 0 && level.alertEvents[i as usize].owner == core::ptr::null_mut()
		{
			continue;
		}

		//Must be within range
		let dist = DistanceSquared( level.alertEvents[i as usize].position, (*self_).currentOrigin );

		//can't see it
		if dist > maxSeeDist_sq
		{
			continue;
		}

		let radius = level.alertEvents[i as usize].radius * level.alertEvents[i as usize].radius;
		if dist > radius
		{
			continue;
		}

		//Must be visible
		if InFOV_entity_spot( &level.alertEvents[i as usize].position, self_, hFOV, vFOV ) == qfalse
		{
			continue;
		}

		if G_ClearLOS( self_, level.alertEvents[i as usize].position ) == qfalse
		{
			continue;
		}

		//FIXME: possibly have the light level at this point affect the
		//			visibility/alert level of this event?  Would also
		//			need to take into account how bright the event
		//			itself is.  A lightsaber would stand out more
		//			in the dark... maybe pass in a light level that
		//			is added to the actual light level at this position?

		//See if this one takes precedence over the previous one
		if level.alertEvents[i as usize].level >= bestAlert //higher alert level
			|| (level.alertEvents[i as usize].level==bestAlert&&level.alertEvents[i as usize].timestamp >= bestTime) //same alert level, but this one is newer
		{//NOTE: equal is better because it's later in the array
			bestEvent = i as c_int;
			bestAlert = level.alertEvents[i as usize].level;
			bestTime = level.alertEvents[i as usize].timestamp;
		}
	}

	return bestEvent;
}

pub unsafe extern "C" fn G_RememberAlertEvent( self_: *mut gentity_t, alertIndex: c_int ) -> c_uint
{
	if self_ == core::ptr::null_mut() || (*self_).NPC == core::ptr::null_mut()
	{//not a valid ent
		return qfalse;
	}

	if alertIndex == -1
	{//not a valid event
		return qfalse;
	}

	// Get The Event Struct
	//----------------------
	let at: *mut alertEvent_t = &mut level.alertEvents[alertIndex as usize];

	if (*at).ID == (*(*self_).NPC).lastAlertID
	{//already know this one
		return qfalse;
	}

	if (*at).owner==self_
	{//don't care about events that I made
		return qfalse;
	}

	(*(*self_).NPC).lastAlertID = (*at).ID;

	// Now, If It Is Dangerous Enough, We Want To Register This With The Pathfinding System
	//--------------------------------------------------------------------------------------
	let	IsDangerous: bool = ((*at).level >= AEL_DANGER as c_int);
	let	IsFromNPC: bool	= ((*at).owner != core::ptr::null_mut() && (*(*at).owner).client != core::ptr::null_mut());
	let	IsFromEnemy: bool = (IsFromNPC && (*(*(*at).owner).client).playerTeam!=(*(*self_).client).playerTeam);

	if (IsDangerous && (!IsFromNPC || IsFromEnemy))
	{
		NAV_RegisterDangerSense(self_, alertIndex);
	}

	return qtrue;
}
/*
-------------------------
NPC_CheckAlertEvents

    NOTE: Should all NPCs create alertEvents too so they can detect each other?
-------------------------
*/

pub unsafe extern "C" fn G_CheckAlertEvents( self_: *mut gentity_t, checkSight: c_uint, checkSound: c_uint, maxSeeDist: f32, maxHearDist: f32, ignoreAlert: c_int, mustHaveOwner: c_uint, minAlertLevel: c_int, onGroundOnly: c_uint ) -> c_int
{
	if &g_entities[0] == core::ptr::null() || g_entities[0].health <= 0
	{
		//player is dead
		return -1;
	}

	let mut bestSoundEvent: c_int = -1;
	let mut bestSightEvent: c_int = -1;
	let mut bestSoundAlert: c_int = -1;
	let mut bestSightAlert: c_int = -1;

	if checkSound != 0
	{
		//get sound event
		bestSoundEvent = G_CheckSoundEvents( self_, maxHearDist, ignoreAlert, mustHaveOwner, minAlertLevel, onGroundOnly );
		//get sound event alert level
		if bestSoundEvent >= 0
		{
			bestSoundAlert = level.alertEvents[bestSoundEvent as usize].level;
		}
	}

	if checkSight != 0
	{
		//get sight event
		if (*self_).NPC != core::ptr::null_mut()
		{
			bestSightEvent = G_CheckSightEvents( self_, (*(*self_).NPC).stats.hfov, (*(*self_).NPC).stats.vfov, maxSeeDist, ignoreAlert, mustHaveOwner, minAlertLevel );
		}
		else
		{
			bestSightEvent = G_CheckSightEvents( self_, 80, 80, maxSeeDist, ignoreAlert, mustHaveOwner, minAlertLevel );//FIXME: look at cg_view to get more accurate numbers?
		}
		//get sight event alert level
		if bestSightEvent >= 0
		{
			bestSightAlert = level.alertEvents[bestSightEvent as usize].level;
		}

		//return the one that has a higher alert (or sound if equal)
		//FIXME:	This doesn't take the distance of the event into account

		if bestSightEvent >= 0 && bestSightAlert > bestSoundAlert
		{//valid best sight event, more important than the sound event
			//get the light level of the alert event for this checker
			let mut eyePoint: [f32; 3] = [0.0; 3];
			let mut sightDir: [f32; 3] = [0.0; 3];
			//get eye point
			CalcEntitySpot( self_, SPOT_HEAD_LEAN, &mut eyePoint );
			VectorSubtract( &level.alertEvents[bestSightEvent as usize].position, &eyePoint, &mut sightDir );
			level.alertEvents[bestSightEvent as usize].light = level.alertEvents[bestSightEvent as usize].addLight + G_GetLightLevel( &level.alertEvents[bestSightEvent as usize].position, &sightDir );
			//return the sight event
			if G_RememberAlertEvent( self_, bestSightEvent ) != 0
			{
				return bestSightEvent;
			}
		}
	}
	//return the sound event
	if G_RememberAlertEvent( self_, bestSoundEvent ) != 0
	{
		return bestSoundEvent;
	}
	//no event or no new event
	return -1;
}

pub unsafe extern "C" fn NPC_CheckAlertEvents( checkSight: c_uint, checkSound: c_uint, ignoreAlert: c_int, mustHaveOwner: c_uint, minAlertLevel: c_int, onGroundOnly: c_uint ) -> c_int
{
	return G_CheckAlertEvents( NPC, checkSight, checkSound, (*NPCInfo).stats.visrange, (*NPCInfo).stats.earshot, ignoreAlert, mustHaveOwner, minAlertLevel, onGroundOnly );
}

extern "C" {
	pub fn WP_ForcePowerStop( self_: *mut gentity_t, forcePower: forcePowers_t );
}

pub unsafe extern "C" fn G_CheckForDanger( self_: *mut gentity_t, alertEvent: c_int ) -> c_uint
{//FIXME: more bStates need to call this?
	if alertEvent == -1
	{
		return qfalse;
	}

	if level.alertEvents[alertEvent as usize].level >= AEL_DANGER as c_int
	{//run away!
		if !level.alertEvents[alertEvent as usize].owner != 0 || (*level.alertEvents[alertEvent as usize].owner).client == core::ptr::null_mut() || (level.alertEvents[alertEvent as usize].owner!=self_&&(*(*level.alertEvents[alertEvent as usize].owner).client).playerTeam!=(*(*self_).client).playerTeam)
		{
			if (*self_).NPC != core::ptr::null_mut()
			{
				if ((*(*self_).NPC).scriptFlags & SCF_DONT_FLEE as c_uint) != 0
				{//can't flee
					return qfalse;
				}
				else
				{
					if level.alertEvents[alertEvent as usize].level >= AEL_DANGER_GREAT as c_int || (*self_).s.weapon == WP_NONE as c_int || (*self_).s.weapon == WP_MELEE as c_int
					{//flee for a longer period of time
						NPC_StartFlee( level.alertEvents[alertEvent as usize].owner, level.alertEvents[alertEvent as usize].position, level.alertEvents[alertEvent as usize].level, 3000, 6000 );
					}
					else if Q_irand( 0, 10 ) == 0 //FIXME: base on rank?  aggression?
					{//just normal danger and I have a weapon, so just a 25% chance of fleeing only for a few seconds
						//FIXME: used to just find a better combat point, need that functionality back
						NPC_StartFlee( level.alertEvents[alertEvent as usize].owner, level.alertEvents[alertEvent as usize].position, level.alertEvents[alertEvent as usize].level, 1000, 3000 );
					}
					else
					{//didn't flee
						TIMER_Set( NPC, "duck", 2000);	// something dangerous going on...
						return qfalse;
					}
					return qtrue;
				}
			}
			else
			{
				return qtrue;
			}
		}
	}
	return qfalse;
}
pub unsafe extern "C" fn NPC_CheckForDanger( alertEvent: c_int ) -> c_uint
{//FIXME: more bStates need to call this?
	return G_CheckForDanger( NPC, alertEvent );
}

/*
-------------------------
AddSoundEvent
-------------------------
*/
fn RemoveOldestAlert() -> c_uint;
pub unsafe extern "C" fn AddSoundEvent( owner: *mut gentity_t, position: *const [f32; 3], radius: f32, alertLevel: alertEventLevel_e, needLOS: c_uint, onGround: c_uint )
{
	//FIXME: Handle this in another manner?
	if level.numAlertEvents >= MAX_ALERT_EVENTS as c_int
	{
		if RemoveOldestAlert() == 0
		{//how could that fail?
			return;
		}
	}

	if owner == core::ptr::null_mut() && (alertLevel as c_int) < AEL_DANGER as c_int	//allows un-owned danger alerts
	{
		return;
	}

	//FIXME: why are Sand creatures suddenly crashlanding?
	if owner != core::ptr::null_mut() && (*owner).client != core::ptr::null_mut() && (*(*owner).client).NPC_class == CLASS_SAND_CREATURE as c_int
	{
		return;
	}

	//FIXME: if owner is not a player or player ally, and there are no player allies present,
	//			perhaps we don't need to store the alert... unless we want the player to
	//			react to enemy alert events in some way?

	// #ifdef _DEBUG
	// assert( !_isnan(position[0]) && !_isnan(position[1]) && !_isnan(position[2]) );
	// #endif
	VectorCopy( position, &mut level.alertEvents[ level.numAlertEvents as usize ].position );

	level.alertEvents[ level.numAlertEvents as usize ].radius	= radius;
	level.alertEvents[ level.numAlertEvents as usize ].level		= alertLevel as c_int;
	level.alertEvents[ level.numAlertEvents as usize ].type_		= AET_SOUND as c_int;
	level.alertEvents[ level.numAlertEvents as usize ].owner		= owner;
	if needLOS != 0
	{//a very low-level sound, when check this sound event, check for LOS
		level.alertEvents[ level.numAlertEvents as usize ].addLight	= 1;	//will force an LOS trace on this sound
	}
	else
	{
		level.alertEvents[ level.numAlertEvents as usize ].addLight	= 0;	//will force an LOS trace on this sound
	}
	level.alertEvents[ level.numAlertEvents as usize ].onGround = onGround as c_int;

	level.alertEvents[ level.numAlertEvents as usize ].ID		= {
		level.curAlertID += 1;
		level.curAlertID
	};
	level.alertEvents[ level.numAlertEvents as usize ].timestamp	= level.time;
	level.numAlertEvents += 1;
}

/*
-------------------------
AddSightEvent
-------------------------
*/

pub unsafe extern "C" fn AddSightEvent( owner: *mut gentity_t, position: *const [f32; 3], radius: f32, alertLevel: alertEventLevel_e, addLight: f32 )
{
	//FIXME: Handle this in another manner?
	if level.numAlertEvents >= MAX_ALERT_EVENTS as c_int
	{
		if RemoveOldestAlert() == 0
		{//how could that fail?
			return;
		}
	}

	if owner == core::ptr::null_mut() && (alertLevel as c_int) < AEL_DANGER as c_int	//allows un-owned danger alerts
	{
		return;
	}

	//FIXME: if owner is not a player or player ally, and there are no player allies present,
	//			perhaps we don't need to store the alert... unless we want the player to
	//			react to enemy alert events in some way?

	// #ifdef _DEBUG
	// assert( !_isnan(position[0]) && !_isnan(position[1]) && !_isnan(position[2]) );
	// #endif
	VectorCopy( position, &mut level.alertEvents[ level.numAlertEvents as usize ].position );

	level.alertEvents[ level.numAlertEvents as usize ].radius	= radius;
	level.alertEvents[ level.numAlertEvents as usize ].level		= alertLevel as c_int;
	level.alertEvents[ level.numAlertEvents as usize ].type_		= AET_SIGHT as c_int;
	level.alertEvents[ level.numAlertEvents as usize ].owner		= owner;
	level.alertEvents[ level.numAlertEvents as usize ].addLight	= addLight as c_int;	//will get added to actual light at that point when it's checked
	level.alertEvents[ level.numAlertEvents as usize ].ID		= {
		level.curAlertID += 1;
		level.curAlertID - 1
	};
	level.alertEvents[ level.numAlertEvents as usize ].timestamp	= level.time;

	level.numAlertEvents += 1;
}

/*
-------------------------
ClearPlayerAlertEvents
-------------------------
*/

pub unsafe extern "C" fn ClearPlayerAlertEvents( )
{
	let curNumAlerts: c_int = level.numAlertEvents;
	//loop through them all (max 32)
	let mut i = 0;
	while i < curNumAlerts
	{
		//see if the event is old enough to delete
		if level.alertEvents[i as usize].timestamp != 0 && level.alertEvents[i as usize].timestamp + ALERT_CLEAR_TIME as c_int < level.time
		{//this event has timed out
			//drop the count
			level.numAlertEvents -= 1;
			//shift the rest down
			if level.numAlertEvents > 0
			{//still have more in the array
				if (i+1) < MAX_ALERT_EVENTS as c_int
				{
					core::ptr::copy(
						&level.alertEvents[(i+1) as usize],
						&mut level.alertEvents[i as usize],
						((MAX_ALERT_EVENTS as c_int)-(i+1)) as usize
					);
				}
			}
			else
			{//just clear this one... or should we clear the whole array?
				core::ptr::write_bytes(&mut level.alertEvents[i as usize], 0, 1);
			}
		}
		else
		{
			i += 1;
		}
	}
	//make sure this never drops below zero... if it does, something very very bad happened
	// assert( level.numAlertEvents >= 0 );

	if eventClearTime < level.time
	{//this is just a 200ms debouncer so things that generate constant alerts (like corpses and missiles) add an alert every 200 ms
		eventClearTime = level.time + ALERT_CLEAR_TIME as c_int;
	}
}

pub unsafe extern "C" fn RemoveOldestAlert( ) -> c_uint
{
	let mut	oldestEvent: c_int = -1;
	let mut oldestTime: c_int = Q3_INFINITE as c_int;
	//loop through them all (max 32)
	for i in 0..level.numAlertEvents
	{
		//see if the event is old enough to delete
		if level.alertEvents[i as usize].timestamp < oldestTime
		{
			oldestEvent = i;
			oldestTime = level.alertEvents[i as usize].timestamp;
		}
	}
	if oldestEvent != -1
	{
		//drop the count
		level.numAlertEvents -= 1;
		//shift the rest down
		if level.numAlertEvents > 0
		{//still have more in the array
			if (oldestEvent+1) < MAX_ALERT_EVENTS as c_int
			{
				core::ptr::copy(
					&level.alertEvents[(oldestEvent+1) as usize],
					&mut level.alertEvents[oldestEvent as usize],
					((MAX_ALERT_EVENTS as c_int)-(oldestEvent+1)) as usize
				);
			}
		}
		else
		{//just clear this one... or should we clear the whole array?
			core::ptr::write_bytes(&mut level.alertEvents[oldestEvent as usize], 0, 1);
		}
	}
	//make sure this never drops below zero... if it does, something very very bad happened
	// assert( level.numAlertEvents >= 0 );
	//return true is have room for one now
	return if (level.numAlertEvents<MAX_ALERT_EVENTS as c_int) { qtrue } else { qfalse };
}

/*
-------------------------
G_ClearLOS
-------------------------
*/

// Position to position
pub unsafe extern "C" fn G_ClearLOS_pp( self_: *mut gentity_t, start: *const [f32; 3], end: *const [f32; 3] ) -> c_uint
{
	let mut tr: trace_t = core::mem::zeroed();
	let mut traceCount: c_int = 0;

	//FIXME: ENTITYNUM_NONE ok?
	gi.trace ( &mut tr, start, core::ptr::null(), core::ptr::null(), end, ENTITYNUM_NONE as c_int, CONTENTS_OPAQUE as c_int/*CONTENTS_SOLID*//*(CONTENTS_SOLID|CONTENTS_MONSTERCLIP)*/ );
	while tr.fraction < 1.0 && traceCount < 3
	{//can see through 3 panes of glass
		if tr.entityNum < ENTITYNUM_WORLD as c_int
		{
			if &g_entities[tr.entityNum as usize] != core::ptr::null() && (g_entities[tr.entityNum as usize].svFlags&SVF_GLASS_BRUSH as c_uint) != 0
			{//can see through glass, trace again, ignoring me
				gi.trace ( &mut tr, &tr.endpos, core::ptr::null(), core::ptr::null(), end, tr.entityNum, MASK_OPAQUE as c_int );
				traceCount += 1;
				continue;
			}
		}
		return qfalse;
	}

	if tr.fraction == 1.0
	{
		return qtrue;
	}

	return qfalse;
}

//Entity to position
pub unsafe extern "C" fn G_ClearLOS_ep( self_: *mut gentity_t, ent: *mut gentity_t, end: *const [f32; 3] ) -> c_uint
{
	let mut eyes: [f32; 3] = [0.0; 3];

	CalcEntitySpot( ent, SPOT_HEAD_LEAN, &mut eyes );

	return G_ClearLOS_pp( self_, &eyes, end );
}

//Position to entity
pub unsafe extern "C" fn G_ClearLOS_pe( self_: *mut gentity_t, start: *const [f32; 3], ent: *mut gentity_t ) -> c_uint
{
	let mut spot: [f32; 3] = [0.0; 3];

	//Look for the chest first
	CalcEntitySpot( ent, SPOT_ORIGIN, &mut spot );

	if G_ClearLOS_pp( self_, start, &spot ) != 0
	{
		return qtrue;
	}

	//Look for the head next
	CalcEntitySpot( ent, SPOT_HEAD_LEAN, &mut spot );

	if G_ClearLOS_pp( self_, start, &spot ) != 0
	{
		return qtrue;
	}

	return qfalse;
}

//NPC's eyes to entity
pub unsafe extern "C" fn G_ClearLOS_ee( self_: *mut gentity_t, ent: *mut gentity_t ) -> c_uint
{
	let mut eyes: [f32; 3] = [0.0; 3];

	//Calculate my position
	CalcEntitySpot( self_, SPOT_HEAD_LEAN, &mut eyes );

	return G_ClearLOS_ep( self_, ent, &eyes );
}

//NPC's eyes to position
pub unsafe extern "C" fn G_ClearLOS_p( self_: *mut gentity_t, end: *const [f32; 3] ) -> c_uint
{
	let mut eyes: [f32; 3] = [0.0; 3];

	//Calculate the my position
	CalcEntitySpot( self_, SPOT_HEAD_LEAN, &mut eyes );

	return G_ClearLOS_pp( self_, &eyes, end );
}

/*
-------------------------
NPC_GetFOVPercentage
-------------------------
*/

pub unsafe extern "C" fn NPC_GetHFOVPercentage( spot: *const [f32; 3], from: *const [f32; 3], facing: *const [f32; 3], hFOV: f32 ) -> f32
{
	let mut deltaVector: [f32; 3] = [0.0; 3];
	let mut angles: [f32; 3] = [0.0; 3];
	let delta: f32;

	VectorSubtract ( spot, from, &mut deltaVector );

	vectoangles ( &deltaVector, &mut angles );

	let delta = (AngleDelta ( (*facing)[YAW as usize], angles[YAW as usize] )).abs();

	if delta > hFOV
	{
		return 0.0;
	}

	return ( ( hFOV - delta ) / hFOV );
}

/*
-------------------------
NPC_GetVFOVPercentage
-------------------------
*/

pub unsafe extern "C" fn NPC_GetVFOVPercentage( spot: *const [f32; 3], from: *const [f32; 3], facing: *const [f32; 3], vFOV: f32 ) -> f32
{
	let mut deltaVector: [f32; 3] = [0.0; 3];
	let mut angles: [f32; 3] = [0.0; 3];
	let delta: f32;

	VectorSubtract ( spot, from, &mut deltaVector );

	vectoangles ( &deltaVector, &mut angles );

	let delta = (AngleDelta ( (*facing)[PITCH as usize], angles[PITCH as usize] )).abs();

	if delta > vFOV
	{
		return 0.0;
	}

	return ( ( vFOV - delta ) / vFOV );
}

const MAX_INTEREST_DIST: f32	= ( 256.0 * 256.0 );
/*
-------------------------
NPC_FindLocalInterestPoint
-------------------------
*/

pub unsafe extern "C" fn G_FindLocalInterestPoint( self_: *mut gentity_t ) -> c_int
{
	let mut bestPoint: c_int = ENTITYNUM_NONE as c_int;
	let mut bestDist: f32 = Q3_INFINITE as f32;
	let mut diffVec: [f32; 3] = [0.0; 3];
	let mut eyes: [f32; 3] = [0.0; 3];

	CalcEntitySpot( self_, SPOT_HEAD_LEAN, &mut eyes );
	for i in 0..level.numInterestPoints
	{
		//Don't ignore portals?  If through a portal, need to look at portal!
		if gi.inPVS( level.interestPoints[i as usize].origin, eyes )
		{
			VectorSubtract( &level.interestPoints[i as usize].origin, &eyes, &mut diffVec );
			if (((diffVec[0]).abs() + (diffVec[1]).abs()) / 2.0) < 48.0 &&
				(diffVec[2]).abs() > (((diffVec[0]).abs() + (diffVec[1]).abs()) / 2.0)
			{//Too close to look so far up or down
				continue;
			}
			let dist: f32 = VectorLengthSquared( &diffVec );
			//Some priority to more interesting points
			//dist -= ((int)level.interestPoints[i].lookMode * 5) * ((int)level.interestPoints[i].lookMode * 5);
			if dist < MAX_INTEREST_DIST && dist < bestDist
			{
				if G_ClearLineOfSight( &eyes, &level.interestPoints[i as usize].origin, (*self_).s.number, MASK_OPAQUE as c_int ) != 0
				{
					bestDist = dist;
					bestPoint = i as c_int;
				}
			}
		}
	}
	if bestPoint != ENTITYNUM_NONE as c_int && level.interestPoints[bestPoint as usize].target != core::ptr::null()
	{
		G_UseTargets2( self_, self_, level.interestPoints[bestPoint as usize].target );
	}
	return bestPoint;
}

/*QUAKED target_interest (1 0.8 0.5) (-4 -4 -4) (4 4 4)
A point that a squadmate will look at if standing still

target - thing to fire when someone looks at this thing
*/

pub unsafe extern "C" fn SP_target_interest( self_: *mut gentity_t )
{//FIXME: rename point_interest
	if level.numInterestPoints >= MAX_INTEREST_POINTS as c_int
	{
		gi.Printf("ERROR:  Too many interest points, limit is %d\n".as_ptr() as *const i8, MAX_INTEREST_POINTS as c_int);
		G_FreeEntity(self_);
		return;
	}

	VectorCopy(&(*self_).currentOrigin, &mut level.interestPoints[level.numInterestPoints as usize].origin);

	if (*self_).target != core::ptr::null() && (*(*self_).target).is_ascii()
	{
		level.interestPoints[level.numInterestPoints as usize].target = G_NewString( (*self_).target );
	}

	level.numInterestPoints += 1;

	G_FreeEntity(self_);
}

// ============================================================================
// LOCAL STUBS - Placeholder declarations for undefined types and functions
// ============================================================================

#[repr(C)]
pub struct trace_t {
	pub fraction: f32,
	pub endpos: [f32; 3],
	pub entityNum: c_int,
	// ... other fields as needed
}

#[repr(C)]
pub struct gentity_t {
	pub s: entityState_t,
	pub client: *mut gclient_t,
	pub currentOrigin: [f32; 3],
	pub health: c_int,
	pub NPC: *mut npcinfodata_t,
	pub svFlags: c_uint,
	pub target: *const c_char,
	pub avelocity: [f32; 3],
	// ... other fields as needed
}

#[repr(C)]
pub struct entityState_t {
	pub number: c_int,
	pub angles: [f32; 3],
	pub weapon: c_int,
	// ... other fields as needed
}

#[repr(C)]
pub struct gclient_t {
	pub ps: playerState_t,
	pub NPC_class: c_int,
	pub playerTeam: c_int,
	pub renderInfo: renderInfo_t,
	// ... other fields as needed
}

#[repr(C)]
pub struct playerState_t {
	pub viewangles: [f32; 3],
	pub velocity: [f32; 3],
	// ... other fields as needed
}

#[repr(C)]
pub struct renderInfo_t {
	pub eyeAngles: [f32; 3],
	// ... other fields as needed
}

#[repr(C)]
pub struct npcinfodata_t {
	pub stats: npcStats_t,
	pub scriptFlags: c_uint,
	pub lastAlertID: c_uint,
	// ... other fields as needed
}

#[repr(C)]
pub struct npcStats_t {
	pub visrange: f32,
	pub earshot: f32,
	pub hfov: c_int,
	pub vfov: c_int,
	// ... other fields as needed
}

#[repr(C)]
pub struct alertEvent_t {
	pub ID: c_uint,
	pub owner: *mut gentity_t,
	pub position: [f32; 3],
	pub radius: f32,
	pub level: c_int,
	pub type_: c_int,
	pub timestamp: c_int,
	pub addLight: c_int,
	pub onGround: c_int,
	pub light: f32,
	// ... other fields as needed
}

#[repr(C)]
pub struct interestPoint_t {
	pub origin: [f32; 3],
	pub target: *const c_char,
	pub lookMode: c_int,
	// ... other fields as needed
}

#[repr(C)]
pub struct level_t {
	pub numAlertEvents: c_int,
	pub alertEvents: [alertEvent_t; 32], // MAX_ALERT_EVENTS
	pub curAlertID: c_uint,
	pub time: c_int,
	pub numInterestPoints: c_int,
	pub interestPoints: [interestPoint_t; 32], // MAX_INTEREST_POINTS
	// ... other fields as needed
}

#[repr(C)]
pub struct cg_t {
	pub time: c_int,
	pub refdefViewAngles: [f32; 3],
	pub refdef: refdef_t,
	// ... other fields as needed
}

#[repr(C)]
pub struct refdef_t {
	pub vieworg: [f32; 3],
	// ... other fields as needed
}

#[repr(u32)]
pub enum visibility_t {
	VIS_NOT = 0,
	VIS_PVS = 1,
	VIS_360 = 2,
	VIS_FOV = 3,
	VIS_SHOOT = 4,
}

#[repr(C)]
pub enum alertEventLevel_e {
	AEL_DANGER = 1,
	AEL_DANGER_GREAT = 2,
	// ... other values as needed
}

#[repr(C)]
pub enum forcePowers_t {
	// Placeholder
}

const SPOT_ORIGIN: c_int = 0;
const SPOT_HEAD: c_int = 1;
const SPOT_HEAD_LEAN: c_int = 2;
const SPOT_LEGS: c_int = 3;
const PITCH: c_int = 0;
const YAW: c_int = 1;
const ENTITYNUM_NONE: c_int = 2047;
const ENTITYNUM_WORLD: c_int = 2048;
const CONTENTS_OPAQUE: c_int = 1;
const MASK_OPAQUE: c_int = 1;
const SVF_GLASS_BRUSH: c_uint = 0x400;
const CHECK_PVS: c_int = 1;
const CHECK_360: c_int = 2;
const CHECK_FOV: c_int = 4;
const CHECK_SHOOT: c_int = 8;
const CHECK_VISRANGE: c_int = 16;
const AET_SOUND: c_int = 1;
const AET_SIGHT: c_int = 2;
const MAX_ALERT_EVENTS: c_int = 32;
const MAX_INTEREST_POINTS: c_int = 32;
const ALERT_CLEAR_TIME: c_int = 200;
const Q3_INFINITE: c_int = 0x7fffffff;
const CLASS_RANCOR: c_int = 1;
const CLASS_WAMPA: c_int = 2;
const CLASS_SAND_CREATURE: c_int = 3;
const WP_NONE: c_int = 0;
const WP_MELEE: c_int = 1;
const SCF_DONT_FLEE: c_uint = 1;
const vec3_origin: [f32; 3] = [0.0, 0.0, 0.0];

extern "C" {
	pub static mut NPC: *mut gentity_t;
	pub static mut NPCInfo: *mut npcinfodata_t;
	pub static mut player: *mut gentity_t;
	pub static mut g_entities: [gentity_t; 2048];
	pub static mut level: level_t;
	pub static mut cg: cg_t;

	pub struct gameImport_t {
		pub trace: unsafe extern "C" fn(*mut trace_t, *const [f32; 3], *const (), *const (), *const [f32; 3], c_int, c_int),
		pub inPVS: unsafe extern "C" fn([f32; 3], [f32; 3]) -> c_uint,
		pub Printf: unsafe extern "C" fn(*const i8, ...),
	}
	pub static gi: gameImport_t;

	pub fn CalcEntitySpot(ent: *mut gentity_t, spot: c_int, point: *mut [f32; 3]);
	pub fn EntIsGlass(ent: *mut gentity_t) -> c_uint;
	pub fn ShotThroughGlass(tr: *mut trace_t, ent: *mut gentity_t, spot: *const [f32; 3], clipmask: c_int);
	pub fn CanShoot(ent: *mut gentity_t, by: *mut gentity_t) -> c_uint;
	pub fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
	pub fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], result: *mut [f32; 3]);
	pub fn VectorNormalize(v: *mut [f32; 3]) -> f32;
	pub fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
	pub fn DotProduct(a: *const [f32; 3], b: *const [f32; 3]) -> f32;
	pub fn vectoangles(value: *const [f32; 3], angles: *mut [f32; 3]);
	pub fn AngleDelta(angle1: f32, angle2: f32) -> f32;
	pub fn VectorLengthSquared(v: *const [f32; 3]) -> f32;
	pub fn VectorLength(v: *const [f32; 3]) -> f32;
	pub fn DistanceSquared(p1: [f32; 3], p2: [f32; 3]) -> f32;
	pub fn VectorCompare(v1: *const [f32; 3], v2: *const [f32; 3]) -> c_uint;
	pub fn cgi_R_GetLighting(pos: *const [f32; 3], ambient: *mut [f32; 3], directed: *mut [f32; 3], lightDir: *mut [f32; 3]);
	pub fn NAV_RegisterDangerSense(ent: *mut gentity_t, alertIndex: c_int);
	pub fn NPC_StartFlee(enemy: *mut gentity_t, origin: [f32; 3], alertLevel: c_int, minFlee: c_int, maxFlee: c_int);
	pub fn Q_irand(a: c_int, b: c_int) -> c_int;
	pub fn TIMER_Set(ent: *mut gentity_t, label: *const c_char, duration: c_int);
	pub fn G_UseTargets2(ent: *mut gentity_t, activator: *mut gentity_t, target: *const c_char);
	pub fn G_FreeEntity(ent: *mut gentity_t);
	pub fn G_NewString(string: *const c_char) -> *const c_char;
}
