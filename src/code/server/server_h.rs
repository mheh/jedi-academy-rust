// server.h

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_short, c_ulong, c_void};

use crate::code::game::q_shared_h::*;
use crate::code::qcommon::qcommon_h::*;
use crate::code::game::g_public_h::*;
use crate::code::game::bg_public_h::*;

// #ifndef SERVER_H
// #define SERVER_H

//=============================================================================

//#define	PERS_SCORE				0		// !!! MUST NOT CHANGE, SERVER AND
										// GAME BOTH REFERENCE !!!
//rww - this won't do.. I need to include bg_public.h in the exe elsewhere.
//I'm including it here instead so we can have our PERS_SCORE value. And have
//it be the proper enum value.

pub const MAX_ENT_CLUSTERS: usize = 16;

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct svEntity_s {
    pub worldSector: *mut worldSector_s,
    pub nextEntityInWorldSector: *mut svEntity_s,

    pub baseline: entityState_t,    // for delta compression of initial sighting
    pub numClusters: c_int,         // if -1, use headnode instead
    pub clusternums: [c_int; MAX_ENT_CLUSTERS],
    pub lastCluster: c_int,         // if all the clusters don't fit in clusternums
    pub areanum: c_int,
    pub areanum2: c_int,
    pub snapshotCounter: c_int,     // used to prevent double adding from portal views
}

#[cfg(feature = "xbox")]
#[repr(C)]
pub struct svEntity_s {
    pub worldSector: *mut worldSector_s,
    pub nextEntityInWorldSector: *mut svEntity_s,

    pub baseline: entityState_t,        // for delta compression of initial sighting
    pub numClusters: i8,                // if -1, use headnode instead
    pub clusternums: [c_short; MAX_ENT_CLUSTERS],
    pub lastCluster: c_short,           // if all the clusters don't fit in clusternums
    pub areanum: c_short,
    pub areanum2: c_short,
    pub snapshotCounter: c_char,        // used to prevent double adding from portal views
}

pub type svEntity_t = svEntity_s;

#[repr(C)]
pub enum serverState_t {
    SS_DEAD,        // no map loaded
    SS_LOADING,     // spawning level entities
    SS_GAME,        // actively running
}

#[repr(C)]
pub struct server_t {
    pub state: serverState_t,
    pub serverId: c_int,                // changes each server start
    #[cfg(feature = "xbox")]
    pub snapshotCounter: c_char,        // incremented for each snapshot built
    #[cfg(not(feature = "xbox"))]
    pub snapshotCounter: c_int,         // incremented for each snapshot built
    pub time: c_int,                    // all entities are correct for this time		// These 2 saved out
    pub timeResidual: c_int,            // <= 1000 / sv_frame->value					//   during savegame.
    pub timeResidualFraction: f32,      // fraction of a msec accumulated
    pub nextFrameTime: c_int,           // when time > nextFrameTime, process world		// this doesn't get used anywhere! -Ste
    pub models: [*mut cmodel_s; MAX_MODELS],
    pub configstrings: [*mut c_char; MAX_CONFIGSTRINGS],
    //
    // be careful, Jake's code uses the 'svEntities' field as a marker to memset-this-far-only inside SV_InitSV()!!!!!
    //
    pub entityParsePoint: *mut c_char,  // used during game VM init

    pub mLocalSubBSPIndex: c_int,
    pub mLocalSubBSPModelOffset: c_int,
    pub mLocalSubBSPEntityParsePoint: *mut c_char,

    pub svEntities: [svEntity_t; MAX_GENTITIES],
}



#[repr(C)]
pub struct clientSnapshot_t {
    pub areabytes: c_int,
    pub areabits: [byte; MAX_MAP_AREA_BYTES],   // portalarea visibility bits
    pub ps: playerState_t,
    pub num_entities: c_int,
    pub first_entity: c_int,    // into the circular sv_packet_entities[]
                                // the entities MUST be in increasing state number
                                // order, otherwise the delta compression will fail
    pub messageSent: c_int,     // time the message was transmitted
    pub messageAcked: c_int,    // time the message was acked
    pub messageSize: c_int,     // used to rate drop packets
}

#[repr(C)]
pub enum clientState_t {
    CS_FREE,        // can be reused for a new connection
    CS_ZOMBIE,      // client has been disconnected, but don't reuse
                    // connection for a couple seconds
    CS_CONNECTED,   // has been assigned to a client_t, but no gamestate yet
    CS_PRIMED,      // gamestate has been sent, but client hasn't sent a usercmd
    CS_ACTIVE,      // client is fully in game
}


#[repr(C)]
pub struct client_s {
    pub state: clientState_t,
    pub userinfo: [c_char; MAX_INFO_STRING],    // name, etc

    pub reliableCommands: [*mut c_char; MAX_RELIABLE_COMMANDS],
    pub reliableSequence: c_int,
    pub reliableAcknowledge: c_int,

    pub gamestateMessageNum: c_int, // netchan->outgoingSequence of gamestate

    pub lastUsercmd: usercmd_t,
    pub lastMessageNum: c_int,      // for delta compression
    pub cmdNum: c_int,              // command number last executed
    pub lastClientCommand: c_int,   // reliable client message sequence
    pub gentity: *mut gentity_t,    // SV_GentityNum(clientnum)
    pub name: [c_char; MAX_NAME_LENGTH],    // extracted from userinfo, high bits masked
    pub download: *mut byte,        // file being downloaded
    pub downloadsize: c_int,        // total bytes (can't use EOF because of paks)
    pub downloadcount: c_int,       // bytes sent
    pub deltaMessage: c_int,        // frame last client usercmd message
    pub lastPacketTime: c_int,      // sv.time when packet was last received
    pub lastConnectTime: c_int,     // sv.time when connection started
    pub nextSnapshotTime: c_int,    // send another snapshot when sv.time >= nextSnapshotTime
    pub rateDelayed: qboolean,      // true if nextSnapshotTime was set based on rate instead of snapshotMsec
    pub droppedCommands: qboolean,  // true if enough pakets to pass the cl_packetdup were dropped
    pub timeoutCount: c_int,        // must timeout a few frames in a row so debugging doesn't break
    pub frames: [clientSnapshot_t; PACKET_BACKUP],  // updates can be delta'd from here
    pub ping: c_int,
    pub rate: c_int,                // bytes / second
    pub snapshotMsec: c_int,        // requests a snapshot every snapshotMsec unless rate choked
    pub netchan: netchan_t,
}

pub type client_t = client_s;

//=============================================================================


#[repr(C)]
pub struct challenge_t {
    pub adr: netadr_t,
    pub challenge: c_int,
    pub time: c_int,
}

// this structure will be cleared only when the game dll changes
#[repr(C)]
pub struct serverStatic_t {
    pub initialized: qboolean,              // sv_init has completed
    pub clients: *mut client_t,             // [sv_maxclients->integer];
    pub numSnapshotEntities: c_int,         // sv_maxclients->integer*PACKET_BACKUP*MAX_PACKET_ENTITIES
    pub nextSnapshotEntities: c_int,        // next snapshotEntities to use
    pub snapshotEntities: *mut entityState_t,   // [numSnapshotEntities]
    pub nextHeartbeatTime: c_int,
}

//=============================================================================

extern "C" {
    pub static mut svs: serverStatic_t;     // persistant server info across maps
    pub static mut sv: server_t;            // cleared each map

    pub static mut ge: *mut game_export_t;

    pub static mut sv_fps: *mut cvar_t;
    pub static mut sv_timeout: *mut cvar_t;
    pub static mut sv_zombietime: *mut cvar_t;
    pub static mut sv_reconnectlimit: *mut cvar_t;
    pub static mut sv_showloss: *mut cvar_t;
    pub static mut sv_killserver: *mut cvar_t;
    pub static mut sv_mapname: *mut cvar_t;
    pub static mut sv_spawntarget: *mut cvar_t;
    pub static mut sv_mapChecksum: *mut cvar_t;
    pub static mut sv_serverid: *mut cvar_t;
    pub static mut sv_testsave: *mut cvar_t;
    pub static mut sv_compress_saved_games: *mut cvar_t;
}

//===========================================================

//
// sv_main.c
//
extern "C" {
    pub fn SV_FinalMessage(message: *mut c_char);
    // QDECL is __cdecl; covered by extern "C"
    pub fn SV_SendServerCommand(cl: *mut client_t, fmt: *const c_char, ...);


    pub fn SV_AddOperatorCommands();
    pub fn SV_RemoveOperatorCommands();
}


//
// sv_init.c
//
extern "C" {
    pub fn SV_SetConfigstring(index: c_int, val: *const c_char);
    pub fn SV_GetConfigstring(index: c_int, buffer: *mut c_char, bufferSize: c_int);

    pub fn SV_SetUserinfo(index: c_int, val: *const c_char);
    pub fn SV_GetUserinfo(index: c_int, buffer: *mut c_char, bufferSize: c_int);

    pub fn SV_SpawnServer(server: *mut c_char, eForceReload: ForceReload_e, bAllowScreenDissolve: qboolean);
}


//
// sv_client.c
//
extern "C" {
    pub fn SV_DirectConnect(from: netadr_t);

    pub fn SV_ExecuteClientMessage(cl: *mut client_t, msg: *mut msg_t);
    pub fn SV_UserinfoChanged(cl: *mut client_t);

    pub fn SV_ClientEnterWorld(client: *mut client_t, cmd: *mut usercmd_t, eSavedGameJustLoaded: SavedGameJustLoaded_e);
    pub fn SV_DropClient(drop: *mut client_t, reason: *const c_char);

    pub fn SV_ExecuteClientCommand(cl: *mut client_t, s: *const c_char);
    pub fn SV_ClientThink(cl: *mut client_t, cmd: *mut usercmd_t);
}


//
// sv_snapshot.c
//
extern "C" {
    pub fn SV_AddServerCommand(client: *mut client_t, cmd: *const c_char);
    pub fn SV_SendMessageToClient(msg: *mut msg_t, client: *mut client_t);
    pub fn SV_SendClientMessages();
    pub fn SV_SendClientSnapshot(client: *mut client_t);
}



//
// sv_game.c
//
extern "C" {
    pub fn SV_GentityNum(num: c_int) -> *mut gentity_t;
    pub fn SV_SvEntityForGentity(gEnt: *mut gentity_t) -> *mut svEntity_t;
    pub fn SV_GEntityForSvEntity(svEnt: *mut svEntity_t) -> *mut gentity_t;
    pub fn SV_InitGameProgs();
    pub fn SV_ShutdownGameProgs(shutdownCin: qboolean);
    pub fn SV_inPVS(p1: *const vec3_t, p2: *const vec3_t) -> qboolean;
}


//============================================================
//
// high level object sorting to reduce interaction tests
//

extern "C" {
    pub fn SV_ClearWorld();
    // called after the world model has been loaded, before linking any entities

    pub fn SV_UnlinkEntity(ent: *mut gentity_t);
    // call before removing an entity, and before trying to move one,
    // so it doesn't clip against itself

    pub fn SV_LinkEntity(ent: *mut gentity_t);
    // Needs to be called any time an entity changes origin, mins, maxs,
    // or solid.  Automatically unlinks if needed.
    // sets ent->v.absmin and ent->v.absmax
    // sets ent->leafnums[] for pvs determination even if the entity
    // is not solid


    pub fn SV_ClipHandleForEntity(ent: *const gentity_t) -> clipHandle_t;


    pub fn SV_SectorList_f();


    pub fn SV_AreaEntities(mins: *const vec3_t, maxs: *const vec3_t, elist: *mut *mut gentity_t, maxcount: c_int) -> c_int;
    // fills in a table of entity pointers with entities that have bounding boxes
    // that intersect the given area.  It is possible for a non-axial bmodel
    // to be returned that doesn't actually intersect the area on an exact
    // test.
    // returns the number of pointers filled in
    // The world entity is never returned in this list.


    pub fn SV_PointContents(p: *const vec3_t, passEntityNum: c_int) -> c_int;
    // returns the CONTENTS_* value from the world and all entities at the given point.

    /*
    Ghoul2 Insert Start
    */
    pub fn SV_Trace(
        results: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        passEntityNum: c_int,
        contentmask: c_int,
        eG2TraceType: EG2_Collision,    // default G2_NOCOLLIDE in C++
        useLod: c_int,                  // default 0 in C++
    );
    /*
    Ghoul2 Insert End
    */
    // mins and maxs are relative

    // if the entire move stays in a solid volume, trace.allsolid will be set,
    // trace.startsolid will be set, and trace.fraction will be 0

    // if the starting point is in a solid, it will be allowed to move out
    // to an open area

    // passEntityNum is explicitly excluded from clipping checks (normally ENTITYNUM_NONE)
}



///////////////////////////////////////////////
//
// sv_savegame.cpp
//
extern "C" {
    pub fn SV_LoadGame_f();
    pub fn SV_LoadTransition_f();
    pub fn SV_SaveGame_f();
    pub fn SV_WipeGame_f();
    pub fn SV_TryLoadTransition(mapname: *const c_char) -> qboolean;
    pub fn SG_WriteSavegame(psPathlessBaseName: *const c_char, qbAutosave: qboolean) -> qboolean;
    pub fn SG_ReadSavegame(psPathlessBaseName: *const c_char) -> qboolean;
    pub fn SG_WipeSavegame(psPathlessBaseName: *const c_char);
    pub fn SG_Append(chid: c_ulong, data: *const c_void, length: c_int) -> qboolean;
    pub fn SG_Read(chid: c_ulong, pvAddress: *mut c_void, iLength: c_int, ppvAddressPtr: *mut *mut c_void) -> c_int;
    pub fn SG_ReadOptional(chid: c_ulong, pvAddress: *mut c_void, iLength: c_int, ppvAddressPtr: *mut *mut c_void) -> c_int;
    pub fn SG_Shutdown();
    pub fn SG_TestSave();
}
//
// note that this version number does not mean that a savegame with the same version can necessarily be loaded,
//	since anyone can change any loadsave-affecting structure somewhere in a header and change a chunk size.
// What it's used for is for things like mission pack etc if we need to distinguish "street-copy" savegames from
//	any new enhanced ones that need to ask for new chunks during loading.
//
pub const iSAVEGAME_VERSION: c_int = 1;
extern "C" {
    pub fn SG_Version() -> c_int;   // call this to know what version number a successfully-opened savegame file was
    //
    pub static mut eSavedGameJustLoaded: SavedGameJustLoaded_e;
    pub static mut qbLoadTransition: qboolean;
}
//
///////////////////////////////////////////////

// #endif	// #ifndef SERVER_H
