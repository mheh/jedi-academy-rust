// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

// #include "g_local.h"
// #include "objectives.h"

use core::ffi::{c_char, c_int};

/*
=======================================================================

  SESSION DATA

Session data is the only data that stays persistant across level loads
and tournament restarts.
=======================================================================
*/

/*
================
G_WriteClientSessionData

Called on game shutdown
================
*/
pub unsafe fn G_WriteClientSessionData(client: *mut gclient_t) {
	let s: *const c_char;
	let s2: *const c_char;
	let var: *const c_char;
	let mut i: c_int;

	s = va(b"%i\0".as_ptr() as *const c_char, (*client).sess.sessionTeam);
	var = va(b"session%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.cvar_set(var, s);

	s2 = b"\0".as_ptr() as *const c_char;
	// Throw all status info into a string
	//	for (i=0;i< MAX_OBJECTIVES; i++)
	//	{
	//		s2 = va("%s %i %i",	s2, client->sess.mission_objectives[i].display,	client->sess.mission_objectives[i].status);
	//	}

	// We're saving only one objective
	s2 = va(b"%i %i\0".as_ptr() as *const c_char, (*client).sess.mission_objectives[LIGHTSIDE_OBJ as usize].display, (*client).sess.mission_objectives[LIGHTSIDE_OBJ as usize].status);

	var = va(b"sessionobj%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.cvar_set(var, s2);

	// Throw all mission stats in to a string
	s2 = va(b"%i %i %i %i %i %i %i %i %i %i %i %i\0".as_ptr() as *const c_char,
			(*client).sess.missionStats.secretsFound,
			(*client).sess.missionStats.totalSecrets,
			(*client).sess.missionStats.shotsFired,
			(*client).sess.missionStats.hits,
			(*client).sess.missionStats.enemiesSpawned,
			(*client).sess.missionStats.enemiesKilled,
			(*client).sess.missionStats.saberThrownCnt,
			(*client).sess.missionStats.saberBlocksCnt,
			(*client).sess.missionStats.legAttacksCnt,
			(*client).sess.missionStats.armAttacksCnt,
			(*client).sess.missionStats.torsoAttacksCnt,
			(*client).sess.missionStats.otherAttacksCnt
			);

	var = va(b"missionstats%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.cvar_set(var, s2);


	s2 = b"\0".as_ptr() as *const c_char;
	i = 0;
	while i < NUM_FORCE_POWERS {
		s2 = va(b"%s %i\0".as_ptr() as *const c_char, s2, (*client).sess.missionStats.forceUsed[i as usize]);
		i += 1;
	}
	var = va(b"sessionpowers%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.cvar_set(var, s2);


	s2 = b"\0".as_ptr() as *const c_char;
	i = 0;
	while i < WP_NUM_WEAPONS {
		s2 = va(b"%s %i\0".as_ptr() as *const c_char, s2, (*client).sess.missionStats.weaponUsed[i as usize]);
		i += 1;
	}
	var = va(b"sessionweapons%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.cvar_set(var, s2);


}

/*
================
G_ReadSessionData

Called on a reconnect
================
*/
pub unsafe fn G_ReadSessionData(client: *mut gclient_t) {
	let mut s: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];
	let var: *const c_char;
	let mut i: c_int;

	var = va(b"session%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.Cvar_VariableStringBuffer(var, s.as_mut_ptr(), MAX_STRING_CHARS);

	sscanf(s.as_ptr(), b"%i\0".as_ptr() as *const c_char, &mut (*client).sess.sessionTeam);

	var = va(b"sessionobj%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.Cvar_VariableStringBuffer(var, s.as_mut_ptr(), MAX_STRING_CHARS);

	var = s.as_ptr();
	//	var++;

	//	for (i=0;i< MAX_OBJECTIVES; i++)
	//	{
	//		sscanf( var, "%i %i",
	//			&client->sess.mission_objectives[i].display,
	//			&client->sess.mission_objectives[i].status);
	//			var+=4;
	//	}
	// Clear the objectives out
	i = 0;
	while i < MAX_OBJECTIVES {
		(*client).sess.mission_objectives[i as usize].display = 0;
		(*client).sess.mission_objectives[i as usize].status = OBJECTIVE_STAT_PENDING;
		i += 1;
	}

	// Now load the LIGHTSIDE objective. That's the only cross level objective.
	sscanf(var, b"%i %i\0".as_ptr() as *const c_char,
		&mut (*client).sess.mission_objectives[LIGHTSIDE_OBJ as usize].display,
		&mut (*client).sess.mission_objectives[LIGHTSIDE_OBJ as usize].status);

	var = va(b"missionstats%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.Cvar_VariableStringBuffer(var, s.as_mut_ptr(), MAX_STRING_CHARS);
	sscanf(s.as_ptr(), b"%i %i %i %i %i %i %i %i %i %i %i %i\0".as_ptr() as *const c_char,
		&mut (*client).sess.missionStats.secretsFound,
		&mut (*client).sess.missionStats.totalSecrets,
		&mut (*client).sess.missionStats.shotsFired,
		&mut (*client).sess.missionStats.hits,
		&mut (*client).sess.missionStats.enemiesSpawned,
		&mut (*client).sess.missionStats.enemiesKilled,
		&mut (*client).sess.missionStats.saberThrownCnt,
		&mut (*client).sess.missionStats.saberBlocksCnt,
		&mut (*client).sess.missionStats.legAttacksCnt,
		&mut (*client).sess.missionStats.armAttacksCnt,
		&mut (*client).sess.missionStats.torsoAttacksCnt,
		&mut (*client).sess.missionStats.otherAttacksCnt);


	var = va(b"sessionpowers%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.Cvar_VariableStringBuffer(var, s.as_mut_ptr(), MAX_STRING_CHARS);

	i = 0;
	var = strtok(s.as_mut_ptr(), b" \0".as_ptr() as *const c_char);
	while var != ::core::ptr::null() {
      /* While there are tokens in "s" */
	  (*client).sess.missionStats.forceUsed[i as usize] = atoi(var);
	  i += 1;
      /* Get next token: */
      var = strtok(::core::ptr::null_mut(), b" \0".as_ptr() as *const c_char);
	}
	assert_eq!(i, NUM_FORCE_POWERS);

	var = va(b"sessionweapons%i\0".as_ptr() as *const c_char, client.offset_from(level.clients) as c_int);
	gi.Cvar_VariableStringBuffer(var, s.as_mut_ptr(), MAX_STRING_CHARS);

	i = 0;
	var = strtok(s.as_mut_ptr(), b" \0".as_ptr() as *const c_char);
	while var != ::core::ptr::null() {
      /* While there are tokens in "s" */
	  (*client).sess.missionStats.weaponUsed[i as usize] = atoi(var);
	  i += 1;
      /* Get next token: */
      var = strtok(::core::ptr::null_mut(), b" \0".as_ptr() as *const c_char);
	}
	assert_eq!(i, WP_NUM_WEAPONS);
}


/*
================
G_InitSessionData

Called on a first-time connect
================
*/
pub unsafe fn G_InitSessionData(client: *mut gclient_t, userinfo: *mut c_char) {
	let sess: *mut clientSession_t;

	sess = &mut (*client).sess;

	(*sess).sessionTeam = TEAM_FREE;

	G_WriteClientSessionData(client);
}


/*
==================
G_InitWorldSession

==================
*/
pub unsafe fn G_InitWorldSession() {
}

/*
==================
G_WriteSessionData

==================
*/
pub unsafe fn G_WriteSessionData() {
	let mut i: c_int;

	gi.cvar_set(b"session\0".as_ptr() as *const c_char, ::core::ptr::null());

	i = 0;
	while i < level.maxclients {
		if (*level.clients.add(i as usize)).pers.connected == CON_CONNECTED {
			G_WriteClientSessionData(level.clients.add(i as usize));
		}
		i += 1;
	}
}

// === EXTERNAL DECLARATIONS (stubs for unported dependencies) ===

// Constants (from headers)
pub const MAX_STRING_CHARS: c_int = 1024;
pub const MAX_OBJECTIVES: c_int = 5;
pub const LIGHTSIDE_OBJ: c_int = 0;
pub const TEAM_FREE: c_int = 0;
pub const CON_CONNECTED: c_int = 1;
pub const OBJECTIVE_STAT_PENDING: c_int = 0;
pub const NUM_FORCE_POWERS: c_int = 16;
pub const WP_NUM_WEAPONS: c_int = 16;

// External C functions
extern "C" {
	pub fn va(fmt: *const c_char, ...) -> *const c_char;
	pub fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
	pub fn strtok(s: *mut c_char, delim: *const c_char) -> *mut c_char;
	pub fn atoi(nptr: *const c_char) -> c_int;
}

// Opaque types that we don't fully define here
#[repr(C)]
pub struct gclient_t {
	pub sess: clientSession_t,
	pub pers: clientPersistent_t,
}

#[repr(C)]
pub struct clientSession_t {
	pub sessionTeam: c_int,
	pub mission_objectives: [objective_t; 5],
	pub missionStats: missionStats_t,
}

#[repr(C)]
pub struct objective_t {
	pub display: c_int,
	pub status: c_int,
}

#[repr(C)]
pub struct missionStats_t {
	pub secretsFound: c_int,
	pub totalSecrets: c_int,
	pub shotsFired: c_int,
	pub hits: c_int,
	pub enemiesSpawned: c_int,
	pub enemiesKilled: c_int,
	pub saberThrownCnt: c_int,
	pub saberBlocksCnt: c_int,
	pub legAttacksCnt: c_int,
	pub armAttacksCnt: c_int,
	pub torsoAttacksCnt: c_int,
	pub otherAttacksCnt: c_int,
	pub forceUsed: [c_int; 16],
	pub weaponUsed: [c_int; 16],
}

#[repr(C)]
pub struct clientPersistent_t {
	pub connected: c_int,
}

#[repr(C)]
pub struct level_t {
	pub clients: *mut gclient_t,
	pub maxclients: c_int,
}

pub static mut level: level_t = level_t {
	clients: ::core::ptr::null_mut(),
	maxclients: 0,
};

#[repr(C)]
pub struct gameImport_t {
	// Stubs for game interface functions
}

pub static mut gi: gameImport_t = gameImport_t {};

// Helper function stubs for game interface methods
impl gameImport_t {
	pub fn cvar_set(&self, var_name: *const c_char, value: *const c_char) {
		// This would be implemented by the actual engine
	}

	pub fn Cvar_VariableStringBuffer(&self, var_name: *const c_char, buffer: *mut c_char, bufsize: c_int) {
		// This would be implemented by the actual engine
	}
}
