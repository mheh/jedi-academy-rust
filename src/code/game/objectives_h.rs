#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// mission Objectives


// DO NOT CHANGE MAX_MISSION_OBJ. IT AFFECTS THE SAVEGAME STRUCTURE

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum objectiveNumber_t { //# Objective_e
	//=================================================
	//
	//=================================================

	LIGHTSIDE_OBJ = 0,
	HOTH2_OBJ1,
	HOTH2_OBJ2,
	HOTH2_OBJ3,
	HOTH3_OBJ1,
	HOTH3_OBJ2,
	HOTH3_OBJ3,
	T2_DPREDICAMENT_OBJ1,
	T2_DPREDICAMENT_OBJ2,
	T2_DPREDICAMENT_OBJ3,
	T2_DPREDICAMENT_OBJ4,
	T2_RANCOR_OBJ1,
	T2_RANCOR_OBJ2,
	T2_RANCOR_OBJ3,
	T2_RANCOR_OBJ4,
	T2_RANCOR_OBJ5,
	T2_RANCOR_OBJ5_2,
	T2_RANCOR_OBJ6,
	T2_WEDGE_OBJ1,
	T2_WEDGE_OBJ2,
	T2_WEDGE_OBJ3,
	T2_WEDGE_OBJ4,
	T2_WEDGE_OBJ5,
	T2_WEDGE_OBJ6,
	T2_WEDGE_OBJ7,
	T2_WEDGE_OBJ8,
	T2_WEDGE_OBJ9,
	T2_WEDGE_OBJ10,
	T2_WEDGE_OBJ11,
	T2_WEDGE_OBJ12,
	T3_RIFT_OBJ1,
	T3_RIFT_OBJ2,
	T3_RIFT_OBJ3,
	T1_DANGER_OBJ1,
	T1_DANGER_OBJ2,
	T1_DANGER_OBJ3,
	T1_DANGER_OBJ4,
	T1_DANGER_OBJ5,
	T3_BOUNTY_OBJ1,
	T3_BOUNTY_OBJ2,
	T3_BOUNTY_OBJ3,
	T3_BOUNTY_OBJ4,
	T3_BOUNTY_OBJ5,
	T3_BOUNTY_OBJ6,
	T3_BOUNTY_OBJ7,
	T3_BOUNTY_OBJ8,
	T3_BOUNTY_OBJ9,
	T2_ROGUE_OBJ1,
	T2_ROGUE_OBJ2,
	T2_TRIP_OBJ1,
	T2_TRIP_OBJ2,
	T3_BYSS_OBJ1,
	T3_BYSS_OBJ2,
	T3_BYSS_OBJ3,
	T3_HEVIL_OBJ1,
	T3_HEVIL_OBJ2,
	T3_HEVIL_OBJ3,
	T3_STAMP_OBJ1,
	T3_STAMP_OBJ2,
	T3_STAMP_OBJ3,
	T3_STAMP_OBJ4,
	TASPIR1_OBJ1,
	TASPIR1_OBJ2,
	TASPIR1_OBJ3,
	TASPIR1_OBJ4,
	TASPIR2_OBJ1,
	TASPIR2_OBJ2,
	VJUN1_OBJ1,
	VJUN1_OBJ2,
	VJUN2_OBJ1,
	VJUN3_OBJ1,
	YAVIN1_OBJ1,
	YAVIN1_OBJ2,
	YAVIN2_OBJ1,
	T1_FATAL_OBJ1,
	T1_FATAL_OBJ2,
	T1_FATAL_OBJ3,
	T1_FATAL_OBJ4,
	T1_FATAL_OBJ5,
	T1_FATAL_OBJ6,
	KOR1_OBJ1,
	KOR1_OBJ2,
	KOR2_OBJ1,
	KOR2_OBJ2,
	KOR2_OBJ3,
	KOR2_OBJ4,
	T1_RAIL_OBJ1,
	T1_RAIL_OBJ2,
	T1_RAIL_OBJ3,
	T1_SOUR_OBJ1,
	T1_SOUR_OBJ2,
	T1_SOUR_OBJ3,
	T1_SOUR_OBJ4,
	T1_SURPRISE_OBJ1,
	T1_SURPRISE_OBJ2,
	T1_SURPRISE_OBJ3,
	T1_SURPRISE_OBJ4,

	//# #eol
	MAX_OBJECTIVES,
}


#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum missionFailed_t { //# MissionFailed_e
	MISSIONFAILED_JAN=0,		//#
	MISSIONFAILED_LUKE,			//#
	MISSIONFAILED_LANDO,		//#
	MISSIONFAILED_R5D2,			//#
	MISSIONFAILED_WARDEN,		//#
	MISSIONFAILED_PRISONERS,	//#
	MISSIONFAILED_EMPLACEDGUNS,	//#
	MISSIONFAILED_LADYLUCK,		//#
	MISSIONFAILED_KYLECAPTURE,	//#
	MISSIONFAILED_TOOMANYALLIESDIED,	//#
	MISSIONFAILED_CHEWIE,		//#
	MISSIONFAILED_KYLE,			//#
	MISSIONFAILED_ROSH,			//#
	MISSIONFAILED_WEDGE,		//#
	MISSIONFAILED_TURNED,		//# Turned on your friends.

	//# #eol
	MAX_MISSIONFAILED,
}


#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum statusText_t { //# StatusText_e
	//=================================================
	//
	//=================================================
	STAT_INSUBORDINATION = 0,		//# Starfleet will not tolerate such insubordination
	STAT_YOUCAUSEDDEATHOFTEAMMATE,	//# You caused the death of a teammate.
	STAT_DIDNTPROTECTTECH,			//# You failed to protect Chell, your technician.
	STAT_DIDNTPROTECT7OF9,			//# You failed to protect 7 of 9
	STAT_NOTSTEALTHYENOUGH,			//# You weren't quite stealthy enough
	STAT_STEALTHTACTICSNECESSARY,	//# Starfleet will not tolerate such insubordination
	STAT_WATCHYOURSTEP,				//# Watch your step
	STAT_JUDGEMENTMUCHDESIRED,		//# Your judgement leaves much to be desired

	//# #eol
	MAX_STATUSTEXT,
}

extern "C" {
	pub static mut missionInfo_Updated: qboolean;
}

pub const SET_TACTICAL_OFF: c_int = 0;
pub const SET_TACTICAL_ON: c_int = 1;

pub const SET_OBJ_HIDE: c_int = 0;
pub const SET_OBJ_SHOW: c_int = 1;
pub const SET_OBJ_PENDING: c_int = 2;
pub const SET_OBJ_SUCCEEDED: c_int = 3;
pub const SET_OBJ_FAILED: c_int = 4;

pub const OBJECTIVE_HIDE: c_int = 0;
pub const OBJECTIVE_SHOW: c_int = 1;

pub const OBJECTIVE_STAT_PENDING: c_int = 0;
pub const OBJECTIVE_STAT_SUCCEEDED: c_int = 1;
pub const OBJECTIVE_STAT_FAILED: c_int = 2;

extern "C" {
	pub static mut statusTextIndex: c_int;

	pub fn OBJ_SaveObjectiveData();
	pub fn OBJ_LoadObjectiveData();
	pub fn OBJ_SetPendingObjectives(ent: *mut gentity_t);
}

// Conditional compilation block: table definitions guarded in C by #ifndef G_OBJECTIVES_CPP
// In the C header, the following are declared extern when G_OBJECTIVES_CPP is not defined,
// and defined in the .cpp file when G_OBJECTIVES_CPP is defined.
// For the Rust port of this header, we preserve the extern declarations.

extern "C" {
	pub static objectiveTable: [stringID_table_t; 0];
	pub static statusTextTable: [stringID_table_t; 0];
	pub static missionFailedTable: [stringID_table_t; 0];
}

// ============================================================================
// Type stubs for dependencies
// ============================================================================

/// Equivalent to C qboolean (typically unsigned char or int)
pub type qboolean = c_int;

/// Forward declaration for entity type; full definition elsewhere
#[repr(C)]
pub struct gentity_t {
	// Opaque structure - full definition in game/g_public.h or similar
	_opaque: [u8; 0],
}

/// String ID table entry for mapping enum values to string names
#[repr(C)]
pub struct stringID_table_t {
	pub name: *const c_char,
	pub id: *const c_void,
}
