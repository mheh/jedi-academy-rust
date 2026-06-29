//NPC_utils.cpp

#![allow(non_snake_case)]

// leave this line at the top for all NPC_xxxx.cpp files...
// #include "g_headers.h"

use core::ffi::{c_int, c_float};

// #include "b_local.h"
// #include "Q3_Interface.h"

// === Stubs and forward declarations ===
// These types and functions are defined elsewhere in the codebase
// and are declared here to satisfy the original C dependencies

// Placeholder types (from g_headers.h, b_local.h, etc.)
#[repr(C)]
pub struct gentity_t {
    // Placeholder - full definition elsewhere
}

#[repr(C)]
pub struct gclient_t {
    // Placeholder - full definition elsewhere
}

#[repr(C)]
pub struct gNPC_t {
    // Placeholder - full definition elsewhere
}

#[repr(C)]
pub struct Vehicle_t {
    // Placeholder - full definition elsewhere
}

type qboolean = c_int;
type vec3_t = [f32; 3];
type trace_t = [u8; 1]; // Placeholder

const TEAM_NUM_TEAMS: usize = 2; // Placeholder value; actual value from headers

// TEAM constants (stubs)
const TEAM_STARFLEET: c_int = 0; // Placeholder
const TEAM_PLAYER: c_int = 0;
const TEAM_ENEMY: c_int = 1;
const TEAM_FREE: c_int = 2;

// spot_t enum (from original)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct spot_t(c_int);

const SPOT_ORIGIN: spot_t = spot_t(0);
const SPOT_CHEST: spot_t = spot_t(1);
const SPOT_HEAD: spot_t = spot_t(2);
const SPOT_HEAD_LEAN: spot_t = spot_t(3);
const SPOT_LEGS: spot_t = spot_t(4);
const SPOT_WEAPON: spot_t = spot_t(5);
const SPOT_GROUND: spot_t = spot_t(6);

// NPC class constants (stubs)
const CLASS_ATST: c_int = 0;
const CLASS_RANCOR: c_int = 1;
const CLASS_WAMPA: c_int = 2;
const CLASS_SAND_CREATURE: c_int = 3;
const CLASS_ROCKETTROOPER: c_int = 4;
const CLASS_VEHICLE: c_int = 5;
const CLASS_GALAKMECH: c_int = 6;
const CLASS_BOBAFETT: c_int = 7;

// Weapon constants (stubs)
const WP_SABER: c_int = 0;
const WP_EMPLACED_GUN: c_int = 1;
const WP_BLASTER_PISTOL: c_int = 2;
const WP_BLASTER: c_int = 3;
const WP_BOWCASTER: c_int = 4;
const WP_REPEATER: c_int = 5;
const WP_FLECHETTE: c_int = 6;
const WP_BRYAR_PISTOL: c_int = 7;
const WP_NOGHRI_STICK: c_int = 8;

// Force power constants (stubs)
const FP_SPEED: c_int = 0;

// Behavior state constants (stubs)
type bState_t = c_int;
const BS_DEFAULT: bState_t = 0;
const BS_SEARCH: bState_t = 1;
const BS_WANDER: bState_t = 2;
const BS_FOLLOW_LEADER: bState_t = 3;
const BS_FACE: bState_t = 4;

// Flag constants
const RF_LOCKEDANGLE: c_int = 0x00000001;
const SVF_IGNORE_ENEMIES: c_int = 0x00000001;
const SVF_LOCKEDENEMY: c_int = 0x00000002;
const SVF_NONNPC_ENEMY: c_int = 0x00000004;
const FL_NOTARGET: c_int = 0x00000001;

// Alert event level constants
const AEL_DISCOVERED: c_int = 2;

// Other constants
const ENTITYNUM_WORLD: c_int = 2047;
const ENTITYNUM_NONE: c_int = 2047;
const WAYPOINT_NONE: c_int = -1;
const MAX_CLIENTS: c_int = 64;
const WORLD_SIZE: f32 = 131072.0;
const MAX_RADIUS_ENTS: usize = 256;
const NEAR_DEFAULT_RADIUS: c_int = 256;

// Min angle error for updates
const MIN_ANGLE_ERROR: f32 = 0.1;

// Angle indices
const YAW: usize = 0;
const PITCH: usize = 1;
const ROLL: usize = 2;

// Render info matrix constants
const ORIGIN: c_int = 0;

// Mask constants
const MASK_PLAYERSOLID: c_int = 0;

// SCF_ flags
const SCF_DONT_FIRE: c_int = 0x00000001;

// TID_ constants
const TID_ANGLE_FACE: c_int = 0;

// Q3_INFINITE constant
const Q3_INFINITE: f32 = 999999.0;

// Stubs for external structures and types
#[repr(C)]
pub struct renderInfo_t {
    // Placeholder - full definition elsewhere
}

#[repr(C)]
pub struct playerState_t {
    // Placeholder - full definition elsewhere
}

#[repr(C)]
pub struct npcStats_t {
    // Placeholder - full definition elsewhere
}

#[repr(C)]
pub struct stringID_table_t {
    // Placeholder - full definition elsewhere
}

#[repr(C)]
pub struct alertEvent_t {
    // Placeholder - full definition elsewhere
}

#[repr(C)]
pub struct cvar_t {
    // Placeholder - full definition elsewhere
}

#[repr(C)]
pub struct mdxaBone_t {
    // Placeholder - full definition elsewhere
}

extern "C" {
    pub fn G_IsRidingVehicle(pEnt: *mut gentity_t) -> *mut Vehicle_t;

    pub fn ViewHeightFix(ent: *const gentity_t);
    pub fn AddLeanOfs(ent: *const gentity_t, point: *mut vec3_t);
    pub fn SubtractLeanOfs(ent: *const gentity_t, point: *mut vec3_t);

    pub fn VectorCompare(a: *const vec3_t, b: *const vec3_t) -> qboolean;
    pub fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    pub fn VectorMA(a: *const vec3_t, scale: f32, b: *const vec3_t, out: *mut vec3_t);
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorLengthSquared(v: *const vec3_t) -> f32;
    pub fn VectorAdd(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);

    pub fn AngleDelta(angle1: f32, angle2: f32) -> f32;
    pub fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
    pub fn AngleNormalize360(angle: f32) -> f32;

    pub fn CalcMuzzlePoint(ent: *mut gentity_t, forward: *const vec3_t, right: *const vec3_t, up: *const vec3_t, muzzlePoint: *mut vec3_t, barrel: c_int);

    pub fn Distance(p1: *const vec3_t, p2: *const vec3_t) -> f32;
    pub fn DistanceSquared(p1: *const vec3_t, p2: *const vec3_t) -> f32;

    pub fn GetAnglesForDirection(p1: *const vec3_t, p2: *const vec3_t, out: *mut vec3_t);

    pub fn InFOV(ent: *const gentity_t, observer: *const gentity_t, hfov: f32, vfov: f32) -> qboolean;
    pub fn NPC_ClearLOS(ent: *const gentity_t) -> qboolean;
    pub fn NPC_ClearShot(ent: *const gentity_t) -> qboolean;

    pub fn Q_flrand(min: f32, max: f32) -> f32;
    pub fn Q_irand(min: c_int, max: c_int) -> c_int;

    pub fn ANGLE2SHORT(angle: f32) -> c_int;
    pub fn SHORT2ANGLE(s: c_int) -> f32;

    pub fn GetIDForString(table: *const stringID_table_t, string: *const c_int) -> bState_t;
    pub fn GetStringForID(table: *const stringID_table_t, id: c_int) -> *const c_int;

    pub fn NPC_IsTrooper(ent: *mut gentity_t) -> bool;

    pub fn NPC_CheckAlertEvents(checkSight: qboolean, checkSound: qboolean, inEnt: c_int, checkType: qboolean, eventLevel: c_int) -> c_int;

    pub fn G_SetEnemy(self_: *mut gentity_t, enemy: *mut gentity_t);
    pub fn G_ClearEnemy(self_: *mut gentity_t);

    pub fn NPC_BSSearchStart(waypoint: c_int, bstate: bState_t);

    pub fn Q3_TaskIDPending(ent: *mut gentity_t, taskID: c_int) -> qboolean;
    pub fn Q3_TaskIDComplete(ent: *mut gentity_t, taskID: c_int);

    pub fn G_AddVoiceEvent(self_: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    pub fn G_CheckControlledTurretEnemy(self_: *mut gentity_t, enemy: *mut gentity_t, validate: qboolean) -> *mut gentity_t;

    pub fn Quake3Game() -> *mut IGameInterface;

    // gi (game interface) functions
    pub fn gi_trace(tr: *mut trace_t, start: *const vec3_t, mins: *const vec3_t, maxs: *const vec3_t, end: *const vec3_t, passent: c_int, contentmask: c_int);
    pub fn gi_EntitiesInBox(mins: *const vec3_t, maxs: *const vec3_t, entityList: *mut *mut gentity_t, maxcount: c_int) -> c_int;
    pub fn gi_G2API_GetBoltMatrix(ghoul2: *mut core::ffi::c_void, modelIndex: c_int, boltIndex: c_int, boltMatrix: *mut mdxaBone_t, angles: *const vec3_t, position: *const vec3_t, frameTime: c_int, entAngles: *const vec3_t, scale: f32);
    pub fn gi_G2API_GiveMeVectorFromMatrix(boltMatrix: *const mdxaBone_t, flags: c_int, vec: *mut vec3_t);
}

#[repr(C)]
pub struct IGameInterface {
    // Placeholder - full definition elsewhere
}

impl IGameInterface {
    pub unsafe fn DebugPrint(&self, _level: c_int, _fmt: *const c_int, _args: ...) {
        // Placeholder
    }
    pub unsafe fn RunScript(&self, _ent: *mut gentity_t, _script: *const c_int) {
        // Placeholder
    }
}

// Global variables
pub static mut teamNumbers: [c_int; TEAM_NUM_TEAMS] = [0; TEAM_NUM_TEAMS];
pub static mut teamStrength: [c_int; TEAM_NUM_TEAMS] = [0; TEAM_NUM_TEAMS];
pub static mut teamCounter: [c_int; TEAM_NUM_TEAMS] = [0; TEAM_NUM_TEAMS];

// Stubs for global pointers set elsewhere
#[allow(non_upper_case_globals)]
pub static mut NPC: *mut gentity_t = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut NPCInfo: *mut gNPC_t = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut client: *mut gclient_t = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut ucmd: usercmd_t = usercmd_t { angles: [0; 3] };
#[allow(non_upper_case_globals)]
pub static mut level: level_t = level_t { time: 0 };
pub static mut g_entities: [gentity_t; 1] = [gentity_t {}];
pub static mut g_timescale: *mut cvar_t = core::ptr::null_mut();
pub static mut cg: cg_t = cg_t { time: 0 };
pub static mut vec3_origin: vec3_t = [0.0; 3];
pub static mut BSTable: [stringID_table_t; 1] = [stringID_table_t {}];
pub static mut BSETTable: [stringID_table_t; 1] = [stringID_table_t {}];

#[repr(C)]
pub struct usercmd_t {
    pub angles: [c_int; 3],
}

#[repr(C)]
pub struct level_t {
    pub time: c_int,
}

#[repr(C)]
pub struct cg_t {
    pub time: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RenderFlags;

const VALID_ATTACK_CONE: f32 = 2.0; //Degrees

/*
void CalcEntitySpot ( gentity_t *ent, spot_t spot, vec3_t point )

Added: Uses shootAngles if a NPC has them

*/
pub unsafe fn CalcEntitySpot(ent: *const gentity_t, spot: spot_t, point: *mut vec3_t)
{
	let mut forward: vec3_t = [0.0; 3];
	let mut up: vec3_t = [0.0; 3];
	let mut right: vec3_t = [0.0; 3];
	let mut start: vec3_t = [0.0; 3];
	let mut end: vec3_t = [0.0; 3];
	let mut tr: trace_t = [0; 1];

	if ent.is_null()
	{
		return;
	}
	ViewHeightFix(ent);
	match spot.0
	{
	SPOT_ORIGIN.0 =>
	{
		if VectorCompare(addr_of!((*ent).currentOrigin), addr_of!(vec3_origin)) != 0
		{//brush
			VectorSubtract(addr_of!((*ent).absmax), addr_of!((*ent).absmin), point);//size
			VectorMA(addr_of!((*ent).absmin), 0.5, point, point);
		}
		else
		{
			VectorCopy(addr_of!((*ent).currentOrigin), point);
		}
	},

	SPOT_CHEST.0 | SPOT_HEAD.0 =>
	{
		if !(*ent).client.is_null() && VectorLengthSquared(addr_of!((*(*ent).client).renderInfo.eyePoint)) != 0.0 && ((*(*ent).client).ps.viewEntity <= 0 || (*(*ent).client).ps.viewEntity >= ENTITYNUM_WORLD)
		{//Actual tag_head eyespot!
			//FIXME: Stasis aliens may have a problem here...
			VectorCopy(addr_of!((*(*ent).client).renderInfo.eyePoint), point);
			if (*(*ent).client).NPC_class == CLASS_ATST
			{//adjust up some
				(*point)[2] += 28.0;//magic number :)
			}
			if !(*ent).NPC.is_null()
			{//always aim from the center of my bbox, so we don't wiggle when we lean forward or backwards
				(*point)[0] = (*ent).currentOrigin[0];
				(*point)[1] = (*ent).currentOrigin[1];
			}
			else if (*ent).s.number == 0
			{
				SubtractLeanOfs(ent, point);
			}
		}
		else
		{
			VectorCopy(addr_of!((*ent).currentOrigin), point);
			if !(*ent).client.is_null()
			{
				(*point)[2] += (*(*ent).client).ps.viewheight;
			}
		}
		if spot.0 == SPOT_CHEST.0 && !(*ent).client.is_null()
		{
			if (*(*ent).client).NPC_class != CLASS_ATST
			{//adjust up some
				(*point)[2] -= (*ent).maxs[2]*0.2;
			}
		}
	},

	SPOT_HEAD_LEAN.0 =>
	{
		if !(*ent).client.is_null() && VectorLengthSquared(addr_of!((*(*ent).client).renderInfo.eyePoint)) != 0.0 && ((*(*ent).client).ps.viewEntity <= 0 || (*(*ent).client).ps.viewEntity >= ENTITYNUM_WORLD)
		{//Actual tag_head eyespot!
			//FIXME: Stasis aliens may have a problem here...
			VectorCopy(addr_of!((*(*ent).client).renderInfo.eyePoint), point);
			if (*(*ent).client).NPC_class == CLASS_ATST
			{//adjust up some
				(*point)[2] += 28.0;//magic number :)
			}
			if !(*ent).NPC.is_null()
			{//always aim from the center of my bbox, so we don't wiggle when we lean forward or backwards
				(*point)[0] = (*ent).currentOrigin[0];
				(*point)[1] = (*ent).currentOrigin[1];
			}
			else if (*ent).s.number == 0
			{
				SubtractLeanOfs(ent, point);
			}
			//NOTE: automatically takes leaning into account!
		}
		else
		{
			VectorCopy(addr_of!((*ent).currentOrigin), point);
			if !(*ent).client.is_null()
			{
				(*point)[2] += (*(*ent).client).ps.viewheight;
			}
			//AddLeanOfs ( ent, point );
		}
	},

	//FIXME: implement...
	//case SPOT_CHEST:
		//Returns point 3/4 from tag_torso to tag_head?
		//break;

	SPOT_LEGS.0 =>
	{
		VectorCopy(addr_of!((*ent).currentOrigin), point);
		(*point)[2] += (*ent).mins[2] * 0.5;
	},

	SPOT_WEAPON.0 =>
	{
		if !(*ent).NPC.is_null() && VectorCompare(addr_of!((*(*ent).NPC).shootAngles), addr_of!(vec3_origin)) == 0 && VectorCompare(addr_of!((*(*ent).NPC).shootAngles), addr_of!((*(*ent).client).ps.viewangles)) == 0
		{
			AngleVectors(addr_of!((*(*ent).NPC).shootAngles), addr_of_mut!(forward), addr_of_mut!(right), addr_of_mut!(up));
		}
		else
		{
			AngleVectors(addr_of!((*(*ent).client).ps.viewangles), addr_of_mut!(forward), addr_of_mut!(right), addr_of_mut!(up));
		}
		CalcMuzzlePoint(ent as *mut gentity_t, addr_of!(forward), addr_of!(right), addr_of!(up), point, 0);
		//NOTE: automatically takes leaning into account!
	},

	SPOT_GROUND.0 =>
	{
		// if entity is on the ground, just use it's absmin
		if (*ent).s.groundEntityNum != -1
		{
			VectorCopy(addr_of!((*ent).currentOrigin), point);
			(*point)[2] = (*ent).absmin[2];
		}
		else
		{
			// if it is reasonably close to the ground, give the point underneath of it
			VectorCopy(addr_of!((*ent).currentOrigin), addr_of_mut!(start));
			start[2] = (*ent).absmin[2];
			VectorCopy(addr_of!(start), addr_of_mut!(end));
			end[2] -= 64.0;
			gi_trace(addr_of_mut!(tr), addr_of!(start), addr_of!((*ent).mins), addr_of!((*ent).maxs), addr_of!(end), (*ent).s.number, MASK_PLAYERSOLID);
			if (*addr_of!(tr)).fraction < 1.0
			{
				VectorCopy(addr_of!((*addr_of!(tr)).endpos), point);
			}
			else
			{
				// otherwise just use the origin
				VectorCopy(addr_of!((*ent).currentOrigin), point);
			}
		}
	},

	_ =>
	{
		VectorCopy(addr_of!((*ent).currentOrigin), point);
	},
	}
}


//===================================================================================

/*
qboolean NPC_UpdateAngles ( qboolean doPitch, qboolean doYaw )

Added: option to do just pitch or just yaw

Does not include "aim" in it's calculations

FIXME: stop compressing angles into shorts!!!!
*/
pub unsafe fn NPC_UpdateAngles(doPitch: qboolean, doYaw: qboolean) -> qboolean
{
	let mut error: f32;
	let mut decay: f32;
	let mut targetPitch: f32 = 0.0;
	let mut targetYaw: f32 = 0.0;
	let mut yawSpeed: f32;
	let mut exact: qboolean = 1; // qtrue

	// if angle changes are locked; just keep the current angles
	// aimTime isn't even set anymore... so this code was never reached, but I need a way to lock NPC's yaw, so instead of making a new SCF_ flag, just use the existing render flag... - dmv
	if NPC.is_null() || ((*NPC).enemy.is_null() && (level.time < (*NPCInfo).aimTime || (*(*NPC).client).renderInfo.renderFlags & RF_LOCKEDANGLE != 0))
	{
		if doPitch != 0
		{
			targetPitch = (*NPCInfo).lockedDesiredPitch;
		}

		if doYaw != 0
		{
			targetYaw = (*NPCInfo).lockedDesiredYaw;
		}
	}
	else
	{
		// we're changing the lockedDesired Pitch/Yaw below so it's lost it's original meaning, get rid of the lock flag
		(*(*NPC).client).renderInfo.renderFlags &= !RF_LOCKEDANGLE;

		if doPitch != 0
		{
			targetPitch = (*NPCInfo).desiredPitch;
			(*NPCInfo).lockedDesiredPitch = (*NPCInfo).desiredPitch;
		}

		if doYaw != 0
		{
			targetYaw = (*NPCInfo).desiredYaw;
			(*NPCInfo).lockedDesiredYaw = (*NPCInfo).desiredYaw;
		}
	}

	if (*NPC).s.weapon == WP_EMPLACED_GUN
	{
		// FIXME: this seems to do nothing, actually...
		yawSpeed = 20.0;
	}
	else
	{
		if (*NPC).client != core::ptr::null_mut() && (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
			&& (*NPC).enemy.is_null()
		{//just slowly lookin' around
			yawSpeed = 1.0;
		}
		else
		{
			yawSpeed = (*NPCInfo).stats.yawSpeed;
		}
	}

	if (*NPC).s.weapon == WP_SABER && (*(*NPC).client).ps.forcePowersActive&(1<<FP_SPEED) != 0
	{
		yawSpeed *= 1.0/(*g_timescale).value;
	}

	if !NPC_IsTrooper(NPC)
		&& !(*NPC).enemy.is_null()
		&& G_IsRidingVehicle(NPC).is_null()
		&& (*(*NPC).client).NPC_class != CLASS_VEHICLE
	{
		if (*NPC).s.weapon==WP_BLASTER_PISTOL ||
			(*NPC).s.weapon==WP_BLASTER ||
			(*NPC).s.weapon==WP_BOWCASTER ||
			(*NPC).s.weapon==WP_REPEATER ||
			(*NPC).s.weapon==WP_FLECHETTE ||
			(*NPC).s.weapon==WP_BRYAR_PISTOL ||
			(*NPC).s.weapon==WP_NOGHRI_STICK
		{
			yawSpeed *= 10.0;
		}
	}

	if doYaw != 0
	{
		// decay yaw error
		error = AngleDelta((*(*NPC).client).ps.viewangles[YAW], targetYaw);
		if error.abs() > MIN_ANGLE_ERROR
		{
			if error != 0.0
			{
				exact = 0; // qfalse

				decay = 60.0 + yawSpeed * 3.0;
				decay *= 50.0 / 1000.0;//msec

				if error < 0.0
				{
					error += decay;
					if error > 0.0
					{
						error = 0.0;
					}
				}
				else
				{
					error -= decay;
					if error < 0.0
					{
						error = 0.0;
					}
				}
			}
		}

		ucmd.angles[YAW] = ANGLE2SHORT(targetYaw + error) - (*client).ps.delta_angles[YAW];
	}

	//FIXME: have a pitchSpeed?
	if doPitch != 0
	{
		// decay pitch error
		error = AngleDelta((*(*NPC).client).ps.viewangles[PITCH], targetPitch);
		if error.abs() > MIN_ANGLE_ERROR
		{
			if error != 0.0
			{
				exact = 0; // qfalse

				decay = 60.0 + yawSpeed * 3.0;
				decay *= 50.0 / 1000.0;//msec

				if error < 0.0
				{
					error += decay;
					if error > 0.0
					{
						error = 0.0;
					}
				}
				else
				{
					error -= decay;
					if error < 0.0
					{
						error = 0.0;
					}
				}
			}
		}

		ucmd.angles[PITCH] = ANGLE2SHORT(targetPitch + error) - (*client).ps.delta_angles[PITCH];
	}

	ucmd.angles[ROLL] = ANGLE2SHORT((*(*NPC).client).ps.viewangles[ROLL]) - (*client).ps.delta_angles[ROLL];

	if exact != 0 && Q3_TaskIDPending(NPC, TID_ANGLE_FACE) != 0
	{
		Q3_TaskIDComplete(NPC, TID_ANGLE_FACE);
	}
	return exact;
}

pub unsafe fn NPC_AimWiggle(enemy_org: *mut vec3_t)
{
	//shoot for somewhere between the head and torso
	//NOTE: yes, I know this looks weird, but it works
	if (*NPCInfo).aimErrorDebounceTime < level.time
	{
		(*NPCInfo).aimOfs[0] = 0.3*Q_flrand((*(*NPC).enemy).mins[0], (*(*NPC).enemy).maxs[0]);
		(*NPCInfo).aimOfs[1] = 0.3*Q_flrand((*(*NPC).enemy).mins[1], (*(*NPC).enemy).maxs[1]);
		if (*(*NPC).enemy).maxs[2] > 0.0
		{
			(*NPCInfo).aimOfs[2] = (*(*NPC).enemy).maxs[2]*Q_flrand(0.0, -1.0);
		}
	}
	VectorAdd(enemy_org, addr_of!((*NPCInfo).aimOfs), enemy_org);
}

/*
qboolean NPC_UpdateFiringAngles ( qboolean doPitch, qboolean doYaw )

  Includes aim when determining angles - so they don't always hit...
  */
pub unsafe fn NPC_UpdateFiringAngles(doPitch: qboolean, doYaw: qboolean) -> qboolean
{
	let mut error: f32;
	let mut diff: f32;
	let mut decay: f32;
	let mut targetPitch: f32 = 0.0;
	let mut targetYaw: f32 = 0.0;
	let mut exact: qboolean = 1; // qtrue

	// if angle changes are locked; just keep the current angles
	if level.time < (*NPCInfo).aimTime
	{
		if doPitch != 0
		{
			targetPitch = (*NPCInfo).lockedDesiredPitch;
		}
		if doYaw != 0
		{
			targetYaw = (*NPCInfo).lockedDesiredYaw;
		}
	}
	else
	{
		if doPitch != 0
		{
			targetPitch = (*NPCInfo).desiredPitch;
		}
		if doYaw != 0
		{
			targetYaw = (*NPCInfo).desiredYaw;
		}

//		NPCInfo->aimTime = level.time + 250;
		if doPitch != 0
		{
			(*NPCInfo).lockedDesiredPitch = (*NPCInfo).desiredPitch;
		}
		if doYaw != 0
		{
			(*NPCInfo).lockedDesiredYaw = (*NPCInfo).desiredYaw;
		}
	}

	if (*NPCInfo).aimErrorDebounceTime < level.time
	{
		if Q_irand(0, 1) != 0
		{
			(*NPCInfo).lastAimErrorYaw = ((6.0 - (*NPCInfo).stats.aim as f32) as f32) * Q_flrand(-1.0, 1.0);
		}
		if Q_irand(0, 1) != 0
		{
			(*NPCInfo).lastAimErrorPitch = ((6.0 - (*NPCInfo).stats.aim as f32) as f32) * Q_flrand(-1.0, 1.0);
		}
		(*NPCInfo).aimErrorDebounceTime = level.time + Q_irand(250, 2000);
	}

	if doYaw != 0
	{
		// decay yaw diff
		diff = AngleDelta((*(*NPC).client).ps.viewangles[YAW], targetYaw);

		if diff != 0.0
		{
			exact = 0; // qfalse

			decay = 60.0 + 80.0;
			decay *= 50.0 / 1000.0;//msec
			if diff < 0.0
			{
				diff += decay;
				if diff > 0.0
				{
					diff = 0.0;
				}
			}
			else
			{
				diff -= decay;
				if diff < 0.0
				{
					diff = 0.0;
				}
			}
		}

		// add yaw error based on NPCInfo->aim value
		error = (*NPCInfo).lastAimErrorYaw;

		/*
		if(Q_irand(0, 1))
		{
			error *= -1;
		}
		*/

		ucmd.angles[YAW] = ANGLE2SHORT(targetYaw + diff + error) - (*client).ps.delta_angles[YAW];
	}

	if doPitch != 0
	{
		// decay pitch diff
		diff = AngleDelta((*(*NPC).client).ps.viewangles[PITCH], targetPitch);
		if diff != 0.0
		{
			exact = 0; // qfalse

			decay = 60.0 + 80.0;
			decay *= 50.0 / 1000.0;//msec
			if diff < 0.0
			{
				diff += decay;
				if diff > 0.0
				{
					diff = 0.0;
				}
			}
			else
			{
				diff -= decay;
				if diff < 0.0
				{
					diff = 0.0;
				}
			}
		}

		error = (*NPCInfo).lastAimErrorPitch;

		ucmd.angles[PITCH] = ANGLE2SHORT(targetPitch + diff + error) - (*client).ps.delta_angles[PITCH];
	}

	ucmd.angles[ROLL] = ANGLE2SHORT((*(*NPC).client).ps.viewangles[ROLL]) - (*client).ps.delta_angles[ROLL];

	return exact;
}
//===================================================================================

/*
static void NPC_UpdateShootAngles (vec3_t angles, qboolean doPitch, qboolean doYaw )

Does update angles on shootAngles
*/

pub unsafe fn NPC_UpdateShootAngles(angles: *const vec3_t, doPitch: qboolean, doYaw: qboolean)
{//FIXME: shoot angles either not set right or not used!
	let mut error: f32;
	let mut decay: f32;
	let mut targetPitch: f32 = 0.0;
	let mut targetYaw: f32 = 0.0;

	if doPitch != 0
	{
		targetPitch = (*angles)[PITCH];
	}
	if doYaw != 0
	{
		targetYaw = (*angles)[YAW];
	}


	if doYaw != 0
	{
		// decay yaw error
		error = AngleDelta((*NPCInfo).shootAngles[YAW], targetYaw);
		if error != 0.0
		{
			decay = 60.0 + 80.0 * (*NPCInfo).stats.aim as f32;
			decay *= 100.0 / 1000.0;//msec
			if error < 0.0
			{
				error += decay;
				if error > 0.0
				{
					error = 0.0;
				}
			}
			else
			{
				error -= decay;
				if error < 0.0
				{
					error = 0.0;
				}
			}
		}
		(*NPCInfo).shootAngles[YAW] = targetYaw + error;
	}

	if doPitch != 0
	{
		// decay pitch error
		error = AngleDelta((*NPCInfo).shootAngles[PITCH], targetPitch);
		if error != 0.0
		{
			decay = 60.0 + 80.0 * (*NPCInfo).stats.aim as f32;
			decay *= 100.0 / 1000.0;//msec
			if error < 0.0
			{
				error += decay;
				if error > 0.0
				{
					error = 0.0;
				}
			}
			else
			{
				error -= decay;
				if error < 0.0
				{
					error = 0.0;
				}
			}
		}
		(*NPCInfo).shootAngles[PITCH] = targetPitch + error;
	}
}

/*
void SetTeamNumbers (void)

Sets the number of living clients on each team

FIXME: Does not account for non-respawned players!
FIXME: Don't include medics?
*/
pub unsafe fn SetTeamNumbers()
{
	let mut found: *mut gentity_t;
	let mut i: c_int;

	for i = 0; i < TEAM_NUM_TEAMS as c_int; i += 1
	{
		teamNumbers[i as usize] = 0;
		teamStrength[i as usize] = 0;
	}

	for i = 0; i < 1; i += 1
	{
		found = addr_of_mut!(g_entities[i as usize]);

		if !(*found).client.is_null()
		{
			if (*found).health > 0 //FIXME: or if a player!
			{
				teamNumbers[(*(*found).client).playerTeam as usize] += 1;
				teamStrength[(*(*found).client).playerTeam as usize] += (*found).health;
			}
		}
	}

	for i = 0; i < TEAM_NUM_TEAMS as c_int; i += 1
	{
		//Get the average health
		if teamNumbers[i as usize] > 0
		{
			teamStrength[i as usize] = ((teamStrength[i as usize] as f32 / teamNumbers[i as usize] as f32).floor()) as c_int;
		}
	}
}

pub unsafe fn G_ActivateBehavior(self_: *mut gentity_t, bset: c_int) -> qboolean
{
	let mut bSID: bState_t = -1;
	let bs_name: *const c_int;

	if self_.is_null()
	{
		return 0; // qfalse
	}

	bs_name = (*self_).behaviorSet[bset as usize];

	if bs_name.is_null() // VALIDSTRING( bs_name )
	{
		return 0; // qfalse
	}

	if !(*self_).NPC.is_null()
	{
		bSID = GetIDForString(addr_of!(BSTable[0]), bs_name);
	}

	if bSID > -1
	{
		(*(*self_).NPC).tempBehavior = BS_DEFAULT;
		(*(*self_).NPC).behaviorState = bSID;
		if bSID == BS_SEARCH || bSID == BS_WANDER
		{
			//FIXME: Reimplement?
			if (*self_).waypoint != WAYPOINT_NONE
			{
				NPC_BSSearchStart((*self_).waypoint, bSID);
			}
			else
			{
				// Stub: NAV::GetNearestNode(self_)
				(*self_).waypoint = -1; // Placeholder
				if (*self_).waypoint != WAYPOINT_NONE
				{
					NPC_BSSearchStart((*self_).waypoint, bSID);
				}
			}
		}
	}
	else
	{
		(*Quake3Game()).DebugPrint(2, "%s attempting to run bSet %s (%s)\n" as *const c_int, (*self_).targetname as *const c_int, GetStringForID(addr_of!(BSETTable[0]), bset) as *const c_int, bs_name as *const c_int);
		(*Quake3Game()).RunScript(self_, bs_name);
	}
	return 1; // qtrue
}


/*
=============================================================================

	Extended Functions

=============================================================================
*/

/*
-------------------------
NPC_ValidEnemy
-------------------------
*/

pub unsafe fn G_ValidEnemy(self_: *mut gentity_t, enemy: *mut gentity_t) -> qboolean
{
	//Must be a valid pointer
	if enemy.is_null()
	{
		return 0; // qfalse
	}

	//Must not be me
	if enemy == self_
	{
		return 0; // qfalse
	}

	//Must not be deleted
	if (*enemy).inuse == 0 // qfalse
	{
		return 0; // qfalse
	}

	//Must be alive
	if (*enemy).health <= 0
	{
		return 0; // qfalse
	}

	//In case they're in notarget mode
	if (*enemy).flags & FL_NOTARGET != 0
	{
		return 0; // qfalse
	}

	//Must be an NPC
	if (*enemy).client.is_null()
	{
		if (*enemy).svFlags&SVF_NONNPC_ENEMY != 0
		{//still potentially valid
			if !(*self_).client.is_null()
			{
				if (*(*enemy).noDamageTeam) == (*(*self_).client).playerTeam
				{
					return 0; // qfalse
				}
				else
				{
					return 1; // qtrue
				}
			}
			else
			{
				if (*enemy).noDamageTeam == (*self_).noDamageTeam
				{
					return 0; // qfalse
				}
				else
				{
					return 1; // qtrue
				}
			}
		}
		else
		{
			return 0; // qfalse
		}
	}

	if (*(*enemy).client).playerTeam == TEAM_FREE && (*enemy).s.number < MAX_CLIENTS
	{//An evil player, everyone attacks him
		return 1; // qtrue
	}

	//Can't be on the same team
	if (*(*enemy).client).playerTeam == (*(*self_).client).playerTeam
	{
		return 0; // qfalse
	}

	//if haven't seen him in a while, give up
	//if ( NPCInfo->enemyLastSeenTime != 0 && level.time - NPCInfo->enemyLastSeenTime > 7000 )//FIXME: make a stat?
	//	return qfalse;

	if (*(*enemy).client).playerTeam == (*(*self_).client).enemyTeam //simplest case: they're on my enemy team
		|| ((*(*self_).client).enemyTeam == TEAM_FREE && (*(*enemy).client).NPC_class != (*(*self_).client).NPC_class )//I get mad at anyone and this guy isn't the same class as me
		|| ((*(*enemy).client).NPC_class == CLASS_WAMPA && !(*enemy).enemy.is_null())//a rampaging wampa
		|| ((*(*enemy).client).NPC_class == CLASS_RANCOR && !(*enemy).enemy.is_null())//a rampaging rancor
		|| ((*(*enemy).client).playerTeam == TEAM_FREE && (*(*enemy).client).enemyTeam == TEAM_FREE && !(*enemy).enemy.is_null() && !(*(*enemy).enemy).client.is_null() && ((*(*(*enemy).enemy).client).playerTeam == (*(*self_).client).playerTeam||((*(*(*enemy).enemy).client).playerTeam != TEAM_ENEMY&&(*(*self_).client).playerTeam==TEAM_PLAYER))) //enemy is a rampaging non-aligned creature who is attacking someone on our team or a non-enemy (this last condition is used only if we're a good guy - in effect, we protect the innocent)
	{
		return 1; // qtrue
	}
	//all other cases = false?
	return 0; // qfalse
}

pub unsafe fn NPC_ValidEnemy(ent: *mut gentity_t) -> qboolean
{
	return G_ValidEnemy(NPC, ent);
}

/*
-------------------------
NPC_TargetVisible
-------------------------
*/

pub unsafe fn NPC_TargetVisible(ent: *mut gentity_t) -> qboolean
{
	//Make sure we're in a valid range
	if DistanceSquared(addr_of!((*ent).currentOrigin), addr_of!((*NPC).currentOrigin)) > ((*NPCInfo).stats.visrange * (*NPCInfo).stats.visrange)
	{
		return 0; // qfalse
	}

	//Check our FOV
	if InFOV(ent, NPC, (*NPCInfo).stats.hfov, (*NPCInfo).stats.vfov) == 0 // qfalse
	{
		return 0; // qfalse
	}

	//Check for sight
	if NPC_ClearLOS(ent) == 0 // qfalse
	{
		return 0; // qfalse
	}

	return 1; // qtrue
}

/*
-------------------------
NPC_FindNearestEnemy
-------------------------
*/

pub unsafe fn NPC_FindNearestEnemy(ent: *mut gentity_t) -> c_int
{
	let mut radiusEnts: [*mut gentity_t; MAX_RADIUS_ENTS] = [core::ptr::null_mut(); MAX_RADIUS_ENTS];
	let mut nearest: *mut gentity_t;
	let mut mins: vec3_t = [0.0; 3];
	let mut maxs: vec3_t = [0.0; 3];
	let mut nearestEntID: c_int = -1;
	let mut nearestDist: f32 = WORLD_SIZE*WORLD_SIZE;
	let mut distance: f32;
	let mut numEnts: c_int;
	let mut i: c_int;

	//Setup the bbox to search in
	for i in 0..3
	{
		mins[i] = (*ent).currentOrigin[i] - (*NPCInfo).stats.visrange;
		maxs[i] = (*ent).currentOrigin[i] + (*NPCInfo).stats.visrange;
	}

	//Get a number of entities in a given space
	numEnts = gi_EntitiesInBox(addr_of!(mins), addr_of!(maxs), radiusEnts.as_mut_ptr(), MAX_RADIUS_ENTS as c_int);

	for i in 0..numEnts
	{
		nearest = G_CheckControlledTurretEnemy(ent, radiusEnts[i as usize], 1); // qtrue

		//Don't consider self
		if nearest == ent
		{
			continue;
		}

		//Must be valid
		if NPC_ValidEnemy(nearest) == 0 // qfalse
		{
			continue;
		}

		//Must be visible
		if NPC_TargetVisible(nearest) == 0 // qfalse
		{
			continue;
		}

		distance = DistanceSquared(addr_of!((*ent).currentOrigin), addr_of!((*nearest).currentOrigin));

		//Found one closer to us
		if distance < nearestDist
		{
			nearestEntID = (*nearest).s.number;
			nearestDist = distance;
		}
	}

	return nearestEntID;
}

/*
-------------------------
NPC_PickEnemyExt
-------------------------
*/

pub unsafe fn NPC_PickEnemyExt(checkAlerts: qboolean) -> *mut gentity_t
{
	//Check for Hazard Team status and remove this check
	/*
	if ( NPC->client->playerTeam != TEAM_STARFLEET )
	{
		//If we've found the player, return it
		if ( NPC_FindPlayer() )
			return &g_entities[0];
	}
	*/

	//If we've asked for the closest enemy
	let entID: c_int = NPC_FindNearestEnemy(NPC);

	//If we have a valid enemy, use it
	if entID >= 0
	{
		return addr_of_mut!(g_entities[entID as usize]);
	}

	if checkAlerts != 0
	{
		let alertEvent: c_int = NPC_CheckAlertEvents(1, 1, -1, 1, AEL_DISCOVERED); // qtrue, qtrue, qtrue

		//There is an event to look at
		if alertEvent >= 0
		{
			let event: *mut alertEvent_t = addr_of_mut!(level.alertEvents[alertEvent as usize]);

			//Don't pay attention to our own alerts
			if (*event).owner == NPC
			{
				return core::ptr::null_mut();
			}

			if (*event).level >= AEL_DISCOVERED
			{
				//If it's the player, attack him
				if (*event).owner == addr_of_mut!(g_entities[0])
				{
					return (*event).owner;
				}

				//If it's on our team, then take its enemy as well
				if !(*(*event).owner).client.is_null() && (*(*(*event).owner).client).playerTeam == (*(*NPC).client).playerTeam
				{
					return (*(*event).owner).enemy;
				}
			}
		}
	}

	return core::ptr::null_mut();
}

/*
-------------------------
NPC_FindPlayer
-------------------------
*/

pub unsafe fn NPC_FindPlayer() -> qboolean
{
	return NPC_TargetVisible(addr_of_mut!(g_entities[0]));
}

/*
-------------------------
NPC_CheckPlayerDistance
-------------------------
*/

unsafe fn NPC_CheckPlayerDistance() -> qboolean
{
	//Make sure we have an enemy
	if (*NPC).enemy.is_null()
	{
		return 0; // qfalse
	}

	//Only do this for non-players
	if (*(*NPC).enemy).s.number == 0
	{
		return 0; // qfalse
	}

	//must be set up to get mad at player
	if (*NPC).client.is_null() || (*(*NPC).client).enemyTeam != TEAM_PLAYER
	{
		return 0; // qfalse
	}

	//Must be within our FOV
	if InFOV(addr_of_mut!(g_entities[0]), NPC, (*NPCInfo).stats.hfov, (*NPCInfo).stats.vfov) == 0 // qfalse
	{
		return 0; // qfalse
	}

	let distance: f32 = DistanceSquared(addr_of!((*NPC).currentOrigin), addr_of!((*(*NPC).enemy).currentOrigin));

	if distance > DistanceSquared(addr_of!((*NPC).currentOrigin), addr_of!(g_entities[0].currentOrigin))
	{
		G_SetEnemy(NPC, addr_of_mut!(g_entities[0]));
		return 1; // qtrue
	}

	return 0; // qfalse
}

/*
-------------------------
NPC_FindEnemy
-------------------------
*/

pub unsafe fn NPC_FindEnemy(checkAlerts: qboolean) -> qboolean
{
	//We're ignoring all enemies for now
	if (*NPC).svFlags & SVF_IGNORE_ENEMIES != 0
	{
		G_ClearEnemy(NPC);
		return 0; // qfalse
	}

	//we can't pick up any enemies for now
	if (*NPCInfo).confusionTime > level.time
	{
		G_ClearEnemy(NPC);
		return 0; // qfalse
	}

	//Don't want a new enemy
	if NPC_ValidEnemy((*NPC).enemy) != 0 && (*NPC).svFlags & SVF_LOCKEDENEMY != 0
	{
		return 1; // qtrue
	}

	//See if the player is closer than our current enemy
	if (*(*NPC).client).NPC_class != CLASS_RANCOR
		&& (*(*NPC).client).NPC_class != CLASS_WAMPA
		&& (*(*NPC).client).NPC_class != CLASS_SAND_CREATURE
		&& NPC_CheckPlayerDistance() != 0
	{//rancors, wampas & sand creatures don't care if player is closer, they always go with closest
		return 1; // qtrue
	}

	//Otherwise, turn off the flag
	(*NPC).svFlags &= !SVF_LOCKEDENEMY;

	//If we've gotten here alright, then our target it still valid
	if NPC_ValidEnemy((*NPC).enemy) != 0
	{
		return 1; // qtrue
	}

	let newenemy: *mut gentity_t = NPC_PickEnemyExt(checkAlerts);

	//if we found one, take it as the enemy
	if NPC_ValidEnemy(newenemy) != 0
	{
		G_SetEnemy(NPC, newenemy);
		return 1; // qtrue
	}

	G_ClearEnemy(NPC);
	return 0; // qfalse
}

/*
-------------------------
NPC_CheckEnemyExt
-------------------------
*/

pub unsafe fn NPC_CheckEnemyExt(checkAlerts: qboolean) -> qboolean
{
	//Make sure we're ready to think again
/*
	if ( NPCInfo->enemyCheckDebounceTime > level.time )
		return qfalse;

	//Get our next think time
	NPCInfo->enemyCheckDebounceTime = level.time + NPC_GetCheckDelta();

	//Attempt to find an enemy
	return NPC_FindEnemy();
*/
	return NPC_FindEnemy(checkAlerts);
}

/*
-------------------------
NPC_FacePosition
-------------------------
*/

pub unsafe fn NPC_FacePosition(position: *mut vec3_t, doPitch: qboolean) -> qboolean
{
	let mut muzzle: vec3_t = [0.0; 3];
	let mut facing: qboolean = 1; // qtrue

	//Get the positions
	if !(*NPC).client.is_null() && ((*(*NPC).client).NPC_class == CLASS_RANCOR || (*(*NPC).client).NPC_class == CLASS_WAMPA || (*(*NPC).client).NPC_class == CLASS_SAND_CREATURE)
	{
		CalcEntitySpot(NPC, SPOT_ORIGIN, addr_of_mut!(muzzle));
		muzzle[2] += (*NPC).maxs[2] * 0.75;
	}
	else if !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == CLASS_GALAKMECH
	{
		CalcEntitySpot(NPC, SPOT_WEAPON, addr_of_mut!(muzzle));
	}
	else
	{
		CalcEntitySpot(NPC, SPOT_HEAD_LEAN, addr_of_mut!(muzzle));//SPOT_HEAD
		if !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
		{//*sigh*, look down more
			(*position)[2] -= 32.0;
		}
	}

	//Find the desired angles
	let mut angles: vec3_t = [0.0; 3];

	GetAnglesForDirection(addr_of!(muzzle), position, addr_of_mut!(angles));

	(*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW]);
	(*NPCInfo).desiredPitch = AngleNormalize360(angles[PITCH]);

	if !(*NPC).enemy.is_null() && !(*(*NPC).enemy).client.is_null() && (*(*(*NPC).enemy).client).NPC_class == CLASS_ATST
	{
		// FIXME: this is kind of dumb, but it was the easiest way to get it to look sort of ok
		(*NPCInfo).desiredYaw += Q_flrand(-5.0, 5.0) + (level.time as f32 * 0.004).sin() * 7.0;
		(*NPCInfo).desiredPitch += Q_flrand(-2.0, 2.0);
	}
	//Face that yaw
	NPC_UpdateAngles(1, 1); // qtrue, qtrue

	//Find the delta between our goal and our current facing
	let yawDelta: f32 = AngleNormalize360((*NPCInfo).desiredYaw - (SHORT2ANGLE(ucmd.angles[YAW] + (*client).ps.delta_angles[YAW])));

	//See if we are facing properly
	if yawDelta.abs() > VALID_ATTACK_CONE
	{
		facing = 0; // qfalse
	}

	if doPitch != 0
	{
		//Find the delta between our goal and our current facing
		let currentAngles: f32 = SHORT2ANGLE(ucmd.angles[PITCH] + (*client).ps.delta_angles[PITCH]);
		let pitchDelta: f32 = (*NPCInfo).desiredPitch - currentAngles;

		//See if we are facing properly
		if pitchDelta.abs() > VALID_ATTACK_CONE
		{
			facing = 0; // qfalse
		}
	}

	return facing;
}

/*
-------------------------
NPC_FaceEntity
-------------------------
*/

pub unsafe fn NPC_FaceEntity(ent: *mut gentity_t, doPitch: qboolean) -> qboolean
{
	let mut entPos: vec3_t = [0.0; 3];

	//Get the positions
	CalcEntitySpot(ent, SPOT_HEAD_LEAN, addr_of_mut!(entPos));

	return NPC_FacePosition(addr_of_mut!(entPos), doPitch);
}

/*
-------------------------
NPC_FaceEnemy
-------------------------
*/

pub unsafe fn NPC_FaceEnemy(doPitch: qboolean) -> qboolean
{
	if NPC.is_null()
	{
		return 0; // qfalse
	}

	if (*NPC).enemy.is_null()
	{
		return 0; // qfalse
	}

	return NPC_FaceEntity((*NPC).enemy, doPitch);
}

/*
-------------------------
NPC_CheckCanAttackExt
-------------------------
*/

pub unsafe fn NPC_CheckCanAttackExt() -> qboolean
{
	//We don't want them to shoot
	if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0
	{
		return 0; // qfalse
	}

	//Turn to face
	if NPC_FaceEnemy(1) == 0 // qtrue
	{
		return 0; // qfalse
	}

	//Must have a clear line of sight to the target
	if NPC_ClearShot((*NPC).enemy) == 0 // qfalse
	{
		return 0; // qfalse
	}

	return 1; // qtrue
}

/*
-------------------------
NPC_ClearLookTarget
-------------------------
*/

pub unsafe fn NPC_ClearLookTarget(self_: *mut gentity_t)
{
	if (*self_).client.is_null()
	{
		return;
	}

	(*(*self_).client).renderInfo.lookTarget = ENTITYNUM_NONE;//ENTITYNUM_WORLD;
	(*(*self_).client).renderInfo.lookTargetClearTime = 0;
}

/*
-------------------------
NPC_SetLookTarget
-------------------------
*/
pub unsafe fn NPC_SetLookTarget(self_: *mut gentity_t, entNum: c_int, clearTime: c_int)
{
	if (*self_).client.is_null()
	{
		return;
	}

	(*(*self_).client).renderInfo.lookTarget = entNum;
	(*(*self_).client).renderInfo.lookTargetClearTime = clearTime;
}

/*
-------------------------
NPC_CheckLookTarget
-------------------------
*/
pub unsafe fn NPC_CheckLookTarget(self_: *mut gentity_t) -> qboolean
{
	if !(*self_).client.is_null()
	{
		if (*(*self_).client).renderInfo.lookTarget >= 0 && (*(*self_).client).renderInfo.lookTarget < ENTITYNUM_WORLD
		{//within valid range
			if g_entities[(*(*self_).client).renderInfo.lookTarget as usize].is_null() || g_entities[(*(*self_).client).renderInfo.lookTarget as usize].inuse == 0
			{//lookTarget not inuse or not valid anymore
				NPC_ClearLookTarget(self_);
			}
			else if (*(*self_).client).renderInfo.lookTargetClearTime != 0 && (*(*self_).client).renderInfo.lookTargetClearTime < level.time
			{//Time to clear lookTarget
				NPC_ClearLookTarget(self_);
			}
			else if !g_entities[(*(*self_).client).renderInfo.lookTarget as usize].client.is_null() && !(*self_).enemy.is_null() && addr_of_mut!(g_entities[(*(*self_).client).renderInfo.lookTarget as usize]) != (*self_).enemy
			{//should always look at current enemy if engaged in battle... FIXME: this could override certain scripted lookTargets...???
				NPC_ClearLookTarget(self_);
			}
			else
			{
				return 1; // qtrue
			}
		}
	}

	return 0; // qfalse
}

/*
-------------------------
NPC_CheckCharmed
-------------------------
*/
pub unsafe fn G_CheckCharmed(self_: *mut gentity_t)
{
	if !self_.is_null()
		&& !(*self_).client.is_null()
		&& (*(*self_).client).playerTeam == TEAM_PLAYER
		&& !(*self_).NPC.is_null()
		&& (*(*self_).NPC).charmedTime != 0
		&& ((*(*self_).NPC).charmedTime < level.time || (*self_).health <= 0)
	{//we were charmed, set us back!
		//NOTE: presumptions here...
		let savTeam: c_int = (*(*self_).client).enemyTeam;
		(*(*self_).client).enemyTeam = (*(*self_).client).playerTeam;
		(*(*self_).client).playerTeam = savTeam;
		(*(*self_).client).leader = core::ptr::null_mut();
		(*(*self_).NPC).charmedTime = 0;
		if (*self_).health > 0
		{
			if (*(*self_).NPC).tempBehavior == BS_FOLLOW_LEADER
			{
				(*(*self_).NPC).tempBehavior = BS_DEFAULT;
			}
			G_ClearEnemy(self_);
			//say something to let player know you've snapped out of it
			G_AddVoiceEvent(self_, Q_irand(0, 2), 2000); // EV_CONFUSE1, EV_CONFUSE3 - stubs
		}
	}

}

pub unsafe fn G_GetBoltPosition(self_: *mut gentity_t, boltIndex: c_int, pos: *mut vec3_t, modelIndex: c_int)
{
	if self_.is_null() // || !self->ghoul2.size()
	{
		return;
	}
	let mut boltMatrix: mdxaBone_t = mdxaBone_t {};
	let mut result: vec3_t = [0.0; 3];
	let mut angles: vec3_t = [0.0, (*self_).currentAngles[YAW], 0.0];

	gi_G2API_GetBoltMatrix(core::ptr::null_mut(), modelIndex,
				boltIndex,
				addr_of_mut!(boltMatrix), addr_of!(angles), addr_of!((*self_).currentOrigin), if cg.time != 0 { cg.time } else { level.time },
				core::ptr::null(), (*self_).s.modelScale);
	if !pos.is_null()
	{
        gi_G2API_GiveMeVectorFromMatrix(addr_of!(boltMatrix), ORIGIN, addr_of_mut!(result));
		VectorCopy(addr_of!(result), pos);
	}
}

pub unsafe fn NPC_EntRangeFromBolt(targEnt: *mut gentity_t, boltIndex: c_int) -> f32
{
	let mut org: vec3_t = [0.0; 3];

	if targEnt.is_null()
	{
		return Q3_INFINITE;
	}

	G_GetBoltPosition(NPC, boltIndex, addr_of_mut!(org), 0);

	return Distance(addr_of!((*targEnt).currentOrigin), addr_of!(org));
}

pub unsafe fn NPC_EnemyRangeFromBolt(boltIndex: c_int) -> f32
{
	return NPC_EntRangeFromBolt((*NPC).enemy, boltIndex);
}

pub unsafe fn G_GetEntsNearBolt(self_: *mut gentity_t, radiusEnts: *mut *mut gentity_t, radius: f32, boltIndex: c_int, boltOrg: *mut vec3_t) -> c_int
{
	let mut mins: vec3_t = [0.0; 3];
	let mut maxs: vec3_t = [0.0; 3];
	let mut i: c_int;

	//get my handRBolt's position
	let mut org: vec3_t = [0.0; 3];

	G_GetBoltPosition(self_, boltIndex, addr_of_mut!(org), 0);

	VectorCopy(addr_of!(org), boltOrg);

	//Setup the bbox to search in
	for i in 0..3
	{
		mins[i as usize] = org[i as usize] - radius;
		maxs[i as usize] = org[i as usize] + radius;
	}

	//Get the number of entities in a given space
	return gi_EntitiesInBox(addr_of!(mins), addr_of!(maxs), radiusEnts, 128);
}

pub unsafe fn NPC_GetEntsNearBolt(radiusEnts: *mut *mut gentity_t, radius: f32, boltIndex: c_int, boltOrg: *mut vec3_t) -> c_int
{
	return G_GetEntsNearBolt(NPC, radiusEnts, radius, boltIndex, boltOrg);
}

extern "C" {
	pub fn RT_Flying(self_: *mut gentity_t) -> qboolean;
	pub fn RT_FlyStart(self_: *mut gentity_t);
	pub fn RT_FlyStop(self_: *mut gentity_t);
	pub fn Boba_Flying(self_: *mut gentity_t) -> qboolean;
	pub fn Boba_FlyStart(self_: *mut gentity_t);
	pub fn Boba_FlyStop(self_: *mut gentity_t);
}

pub unsafe fn JET_Flying(self_: *mut gentity_t) -> qboolean
{
	if self_.is_null() || (*self_).client.is_null()
	{
		return 0; // qfalse
	}
	if (*(*self_).client).NPC_class == CLASS_BOBAFETT
	{
		return Boba_Flying(self_);
	}
	else if (*(*self_).client).NPC_class == CLASS_ROCKETTROOPER
	{
		return RT_Flying(self_);
	}
	else
	{
		return 0; // qfalse
	}
}

pub unsafe fn JET_FlyStart(self_: *mut gentity_t)
{
	if self_.is_null() || (*self_).client.is_null()
	{
		return;
	}
	(*self_).lastInAirTime = level.time;
	if (*(*self_).client).NPC_class == CLASS_BOBAFETT
	{
		Boba_FlyStart(self_);
	}
	else if (*(*self_).client).NPC_class == CLASS_ROCKETTROOPER
	{
		RT_FlyStart(self_);
	}
}

pub unsafe fn JET_FlyStop(self_: *mut gentity_t)
{
	if self_.is_null() || (*self_).client.is_null()
	{
		return;
	}
	if (*(*self_).client).NPC_class == CLASS_BOBAFETT
	{
		Boba_FlyStop(self_);
	}
	else if (*(*self_).client).NPC_class == CLASS_ROCKETTROOPER
	{
		RT_FlyStop(self_);
	}
}
