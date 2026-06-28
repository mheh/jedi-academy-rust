// leave this line at the top for all g_xxxx.cpp files...
// (C header: g_headers.h)

//g_camera.cpp
// (C headers: g_local.h)
//#include "Q3_Interface.h"
//#include "anims.h"
//#include "b_local.h"
// (C header: ../cgame/cg_camera.h)
// (C headers: g_functions.h)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_char, c_void};

// ============================================================================
// Type Definitions
// ============================================================================

/// Forward reference to gentity_t from g_local.h (used as pointer).
/// Minimal stub definition with only the fields accessed by this file.
#[repr(C)]
pub struct gentity_t {
    pub targetname: [c_char; 64], // MAX_QPATH = 64
    pub speed: f32,
    pub script_targetname: *mut c_char,
    // ... other fields not used in this file
}

// ============================================================================
// External Functions and Globals
// ============================================================================

extern "C" {
    /// G_NewString - Allocate and copy a string
    pub fn G_NewString(string: *const c_char) -> *mut c_char;

    /// G_FreeEntity - Mark an entity for deletion
    pub fn G_FreeEntity(self_: *mut gentity_t);

    /// gi.Printf - Printf function from game import
    pub fn gi_Printf(fmt: *const c_char, ...) -> c_int;
}


// ============================================================================
// Functions
// ============================================================================

/*
#define MAX_CAMERA_GROUP_SUBJECTS	16
void misc_camera_focus_think (gentity_t *self)
{
	//Check to see if I should stop?
	gi.linkentity(self);
	self->nextthink = level.time + FRAMETIME;
}

void misc_camera_focus_use (gentity_t *self, gentity_t *other, gentity_t *activator)
{
	G_ActivateBehavior(self,BSET_USE);

	//First, find everyone in my cameraGroup, if I have one

	//Now find my first path_corner, if I have one

	//Start thinking

	self->e_ThinkFunc = thinkF_misc_camera_focus_think;
	misc_camera_focus_think(self);

	self->e_clThinkFunc = clThinkF_CG_MiscCameraFocusThink;	// blurgh!...
	self->s.eType = ET_THINKER;

	self->aimDebounceTime = level.time;
}
*/
/*QUAK-ED misc_camera_focus (0 0 1) (-4 -4 -4) (4 4 4) lerptostart

LERPTOSTART - With interpolate from camera's current angles to this thing's start angle instead of snapping to it, which is the default behavior

The focal point for a camera in a scene

"targetname" - Use it to get it to find it's cameraGroup and start tracking it, also get started on it's path, if any

"cameraGroup" - will find all ents in this group and pick a point in the center of that group.

"speed" angular speed modifier - 100 is normal
*/
pub unsafe fn SP_misc_camera_focus(self_: *mut gentity_t) {
    if (*self_).targetname[0] == 0 {
        gi_Printf(b"^1ERROR: misc_camera_focus with no targetname\n\0".as_ptr() as *const c_char);
        G_FreeEntity(self_);
        return;
    }

    /*
    if(self->speed > 0)
    {
        self->moveInfo.aspeed = self->speed;
    }
    else
    {
        self->moveInfo.aspeed = 100.f;
    }
    */
    (*self_).speed = 0.0;
    (*self_).script_targetname = G_NewString((*self_).targetname.as_ptr() as *const c_char);
    //	self->e_UseFunc = useF_misc_camera_focus_use;
}

/*
void misc_camera_track_think (gentity_t *self)
{
	vec3_t		vec;
	float		dist;
	//Check to see if I should stop?

	gi.linkentity(self);

	if(self->enemy)
	{//We're already heading to a path_corner
		VectorSubtract(self->currentOrigin, self->s.origin2, vec);
		dist = VectorLengthSquared(vec);
		if(dist < 256)//16 squared
		{
			G_UseTargets(self, self);
			self->target = self->enemy->target;
			self->enemy = NULL;
		}
	}

	if( !self->enemy)
	{
		if( self->target && self->target[0] )
		{//Find out next path_corner
			self->enemy = G_Find(NULL, FOFS(targetname), self->target);
			if(self->enemy)
			{
				if(self->enemy->radius < 0)
				{//Don't bother trying to maintain a radius
					self->radius = 0;
					self->moveInfo.speed = self->speed/10.0f;
				}
				else if(self->enemy->radius > 0)
				{
					self->radius = self->enemy->radius;
				}

				if(self->enemy->speed < 0)
				{//go back to our default speed
					self->moveInfo.speed = self->speed/10.0f;
				}
				else if(self->enemy->speed > 0)
				{
					self->moveInfo.speed = self->enemy->speed/10.0f;
				}
			}
		}
		else
		{//stop thinking if this is the last one
			self->e_ThinkFunc = thinkF_NULL;
			self->e_clThinkFunc = clThinkF_NULL;
			self->s.eType = ET_GENERAL;
			self->nextthink = -1;
		}
	}

	if(self->enemy)
	{//clThink will lerp this
		VectorCopy(self->enemy->currentOrigin, self->s.origin2);
	}

	self->nextthink = level.time + FRAMETIME;
}

void misc_camera_track_use (gentity_t *self, gentity_t *other, gentity_t *activator)
{

	G_ActivateBehavior(self,BSET_USE);

	//Start thinking

	self->e_ThinkFunc = thinkF_misc_camera_track_think;
	misc_camera_track_think(self);

	self->e_clThinkFunc = clThinkF_CG_MiscCameraTrackThink;
	self->s.eType = ET_THINKER;
}
*/
/*QUAK-ED misc_camera_track (0 0 1) (-4 -4 -4) (4 4 4)

The track for a camera to stay on

"targetname" - Use it to get it started on it's path

"target" - First point on it's path - if misc_camera_focus is on a path, it will pick the point on it's path closest to the above calced point
use "path_corner"s - path it should stay on- if that path_corner has a speed value, it will use this as it's speed to the next path_corner

"speed" - How quickly to move, 0 by default

"radius" - How far camera should try to stay from it's subject, default is 0 (dist doesn't matter), can pick this up from a path_corner too
*/
pub unsafe fn SP_misc_camera_track(self_: *mut gentity_t) {
    if (*self_).targetname[0] == 0 {
        gi_Printf(b"^1ERROR: misc_camera_track with no targetname\n\0".as_ptr() as *const c_char);
        G_FreeEntity(self_);
        return;
    }

    (*self_).script_targetname = G_NewString((*self_).targetname.as_ptr() as *const c_char);
    //self->moveInfo.speed = self->speed/10;

    //	self->e_UseFunc = useF_misc_camera_track_use;
}


//-------------------------------------------------
//	Bezier camera stuff
//-------------------------------------------------

pub unsafe fn cam_point_link(_ent: *mut gentity_t) {

}

pub unsafe fn cam_ctrl_point_link(_ent: *mut gentity_t) {
/*	gentity_t	*target2 = NULL;

	target2 = G_Find( NULL, FOFS(targetname), ent->target2 );

	if ( !target2 )
	{
		// Bah, you fool!  Target2 not found
		Com_Printf( "cam_point_link: target2 specified but not found: %s\n", ent->target2 );
		G_FreeEntity( ent );
		return;
	}

	// Store the control point here
	VectorCopy( target2->s.origin, ent->pos1 );

	//---------------------
	if ( ent->target )
	{
		gentity_t	*target = NULL;

		target = G_Find( NULL, FOFS(targetname), ent->target );

		if ( !target )
		{
			// Bah, you fool!  Target not found
			Com_Printf( "cam_point_link: target specified but not found: %s\n", ent->target );
			G_FreeEntity( ent );
			return;
		}

		ent->nextTrain = target;
	}
*/
}

/*QUAK-ED cam_point (0.25 0 0.5) (-2 -2 -2) (2 2 2)
Under development -- DONT USE ME!!!!!
A camera point used to construct a camera bezier path

Every cam_point MUST be targeted (target2) at one and only one control point
*/
pub unsafe fn SP_cam_point(_ent: *mut gentity_t) {
/*	if ( !ent->target2 )
	{
		// Bah, you fool!  Target2 not found so we have no idea how to make the curve
		Com_Printf( "cam_point_link: target2 was required but not found\n" );
		G_FreeEntity( ent );
		return;

	}

	// The thing we are targeting may not be spawned in yet so, wait a bit to try and link to it
	ent->e_ThinkFunc = thinkF_cam_ctrl_point_link;
	ent->nextthink = level.time + 200;

	// Save our position and link us up!
	G_SetOrigin( ent, ent->s.origin );
	gi.linkentity( ent );
*/
}
