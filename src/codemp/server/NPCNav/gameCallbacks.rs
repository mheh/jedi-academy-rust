//rww - callbacks the navigation system needs to make to the game code.

use core::ffi::{c_int, c_void};

// Type stubs from ../../game/q_shared.h and ../../game/g_public.h
// Only the fields used in this file are included

#[repr(C)]
pub struct entityState_s {
	pub number: c_int,
	// Additional fields not used in this file
}

#[repr(C)]
pub struct sharedEntity_s {
	pub s: entityState_s,
	// Additional fields not used in this file
}

pub type sharedEntity_t = sharedEntity_s;
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

// Game VM command constants (from g_public.h or server.h)
// STUB: actual values should come from the game headers
pub const GAME_NAV_CLEARPATHTOPOINT: c_int = 1;
pub const GAME_NAV_CLEARLOS: c_int = 2;
pub const GAME_NAV_CLEARPATHBETWEENPOINTS: c_int = 3;
pub const GAME_NAV_CHECKNODEFAILEDFORENT: c_int = 4;
pub const GAME_NAV_ENTISUNLOCKEDDOOR: c_int = 5;
pub const GAME_NAV_ENTISDOOR: c_int = 6;
pub const GAME_NAV_ENTISBREAKABLE: c_int = 7;
pub const GAME_NAV_ENTISREMOVABLEUSABLE: c_int = 8;
pub const GAME_NAV_FINDCOMBATPOINTWAYPOINTS: c_int = 9;

// External engine globals and functions
extern "C" {
	pub static mut gvm: *mut c_void;
	pub fn VM_Call(vm: *mut c_void, cmd: c_int, ...) -> c_int;
}

pub fn GNavCallback_NAV_ClearPathToPoint(
	self_: *mut sharedEntity_t,
	pmins: vec3_t,
	pmaxs: vec3_t,
	point: vec3_t,
	clipmask: c_int,
	okToHitEntNum: c_int,
) -> qboolean {
	unsafe {
		VM_Call(
			gvm,
			GAME_NAV_CLEARPATHTOPOINT,
			(*self_).s.number,
			&pmins[0] as *const f32,
			&pmaxs[0] as *const f32,
			&point[0] as *const f32,
			clipmask,
			okToHitEntNum,
		) as qboolean
	}
}

pub fn GNavCallback_NPC_ClearLOS(
	ent: *mut sharedEntity_t,
	end: vec3_t,
) -> qboolean {
	unsafe {
		VM_Call(gvm, GAME_NAV_CLEARLOS, (*ent).s.number, &end[0] as *const f32) as qboolean
	}
}

pub fn GNavCallback_NAVNEW_ClearPathBetweenPoints(
	start: vec3_t,
	end: vec3_t,
	mins: vec3_t,
	maxs: vec3_t,
	ignore: c_int,
	clipmask: c_int,
) -> c_int {
	unsafe {
		VM_Call(
			gvm,
			GAME_NAV_CLEARPATHBETWEENPOINTS,
			&start[0] as *const f32,
			&end[0] as *const f32,
			&mins[0] as *const f32,
			&maxs[0] as *const f32,
			ignore,
			clipmask,
		)
	}
}

pub fn GNavCallback_NAV_CheckNodeFailedForEnt(
	ent: *mut sharedEntity_t,
	nodeNum: c_int,
) -> qboolean {
	unsafe {
		VM_Call(
			gvm,
			GAME_NAV_CHECKNODEFAILEDFORENT,
			(*ent).s.number,
			nodeNum,
		) as qboolean
	}
}

pub fn GNavCallback_G_EntIsUnlockedDoor(entityNum: c_int) -> qboolean {
	unsafe {
		VM_Call(gvm, GAME_NAV_ENTISUNLOCKEDDOOR, entityNum) as qboolean
	}
}

pub fn GNavCallback_G_EntIsDoor(entityNum: c_int) -> qboolean {
	unsafe {
		VM_Call(gvm, GAME_NAV_ENTISDOOR, entityNum) as qboolean
	}
}

pub fn GNavCallback_G_EntIsBreakable(entityNum: c_int) -> qboolean {
	unsafe {
		VM_Call(gvm, GAME_NAV_ENTISBREAKABLE, entityNum) as qboolean
	}
}

pub fn GNavCallback_G_EntIsRemovableUsable(entNum: c_int) -> qboolean {
	unsafe {
		VM_Call(gvm, GAME_NAV_ENTISREMOVABLEUSABLE, entNum) as qboolean
	}
}

pub fn GNavCallback_CP_FindCombatPointWaypoints() {
	unsafe {
		VM_Call(gvm, GAME_NAV_FINDCOMBATPOINTWAYPOINTS);
	}
}
