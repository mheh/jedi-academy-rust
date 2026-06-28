// leave this line at the top for all g_xxxx.cpp files...
// (C header: g_headers.h)

//g_objectives.cpp
//reads in ext_data\objectives.dat to objectives[]

// (C headers: g_local.h, g_items.h)

use core::ffi::{c_int, c_void};

// Constants from objectives.h
const MAX_OBJECTIVES: c_int = 100; // MAX_MISSION_OBJ

const OBJECTIVE_STAT_PENDING: c_int = 0;
const OBJECTIVE_STAT_FAILED: c_int = 2;

// Type declarations matching C structs from oracle/code/game/g_shared.h

#[repr(C)]
pub struct objectives_s {
    pub display: c_int,   // qboolean
    pub status: c_int,    // int
}
pub type objectives_t = objectives_s;

#[repr(C)]
pub struct clientSession_s {
    pub missionObjectivesShown: c_int,
    pub sessionTeam: c_int,
    pub mission_objectives: [objectives_t; 100], // MAX_MISSION_OBJ = 100
    // missionStats_t missionStats; (opaque, not accessed by this module)
}
pub type clientSession_t = clientSession_s;

// Opaque type stubs for types defined in other headers.
// Using byte arrays with sizes matching the oracle structures.
#[repr(C)]
pub struct playerState_s {
    pub _data: [u8; 1552], // playerState_t from q_shared.h
}
pub type playerState_t = playerState_s;

#[repr(C)]
pub struct clientPersistant_s {
    pub _data: [u8; 156], // clientPersistant_t from g_shared.h (estimated from oracle/code)
}
pub type clientPersistant_t = clientPersistant_s;

#[repr(C)]
pub struct gclient_s {
    pub ps: playerState_t,
    pub pers: clientPersistant_t,
    pub sess: clientSession_t,
}
pub type gclient_t = gclient_s;

#[repr(C)]
pub struct entityState_s {
    pub _data: [u8; 532], // entityState_t from q_shared.h
}
pub type entityState_t = entityState_s;

#[repr(C)]
pub struct gentity_s {
    pub s: entityState_t,
    pub client: *mut gclient_t,   // NULL if not a player
}
pub type gentity_t = gentity_s;

// External function declarations (from engine/game interface)
extern "C" {
    // gi.AppendToSaveGame and gi.ReadFromSaveGame
    // The 'OBJT' tag in C would be a 4-byte literal: 0x544a424f
    pub fn AppendToSaveGame(tag: c_int, data: *mut c_void, size: usize);
    pub fn ReadFromSaveGame(tag: c_int, data: *mut c_void, size: usize);

    // Global level array from g_local.h: gclient_t *clients; // [maxclients]
    pub static mut level_clients: *mut gclient_t;
}

// Global variable from objectives.h
pub static mut missionInfo_Updated: c_int = 0;

/*
============
OBJ_SetPendingObjectives
============
*/
#[allow(non_snake_case)]
pub unsafe fn OBJ_SetPendingObjectives(ent: *mut gentity_t) {
    let mut i: c_int = 0;

    while i < MAX_OBJECTIVES {
        if ((*(*ent).client).sess.mission_objectives[i as usize].status == OBJECTIVE_STAT_PENDING)
            && ((*(*ent).client).sess.mission_objectives[i as usize].display != 0)
        {
            (*(*ent).client).sess.mission_objectives[i as usize].status = OBJECTIVE_STAT_FAILED;
        }
        i += 1;
    }
}

/*
============
OBJ_SaveMissionObjectives
============
*/
#[allow(non_snake_case)]
pub unsafe fn OBJ_SaveMissionObjectives(client: *mut gclient_t) {
    // 'OBJT' in C is a 4-byte character literal: ('O' << 24) | ('B' << 16) | ('J' << 8) | 'T'
    let tag: c_int = (('O' as c_int) << 24) | (('B' as c_int) << 16) | (('J' as c_int) << 8) | ('T' as c_int);
    AppendToSaveGame(
        tag,
        core::ptr::addr_of_mut!((*client).sess.mission_objectives) as *mut c_void,
        core::mem::size_of_val(&(*client).sess.mission_objectives),
    );
}

/*
============
OBJ_SaveObjectiveData
============
*/
#[allow(non_snake_case)]
pub unsafe fn OBJ_SaveObjectiveData() {
    let client: *mut gclient_t;

    client = level_clients;

    OBJ_SaveMissionObjectives(client);
}

/*
============
OBJ_LoadMissionObjectives
============
*/
#[allow(non_snake_case)]
pub unsafe fn OBJ_LoadMissionObjectives(client: *mut gclient_t) {
    // 'OBJT' in C is a 4-byte character literal
    let tag: c_int = (('O' as c_int) << 24) | (('B' as c_int) << 16) | (('J' as c_int) << 8) | ('T' as c_int);
    ReadFromSaveGame(
        tag,
        core::ptr::addr_of_mut!((*client).sess.mission_objectives) as *mut c_void,
        core::mem::size_of_val(&(*client).sess.mission_objectives),
    );
}

/*
============
OBJ_LoadObjectiveData
============
*/
#[allow(non_snake_case)]
pub unsafe fn OBJ_LoadObjectiveData() {
    let client: *mut gclient_t;

    client = level_clients;

    OBJ_LoadMissionObjectives(client);
}
