// server.h
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

// Forward declarations for types from other modules
// These are stubs to satisfy the struct definitions in this file
#[repr(C)]
pub struct worldSector_s {
    // opaque
}

#[repr(C)]
pub struct entityState_t {
    // opaque - from q_shared.h
}

#[repr(C)]
pub struct cmodel_s {
    // opaque - from qcommon.h
}

#[repr(C)]
pub struct sharedEntity_t {
    // opaque - from g_public.h
}

#[repr(C)]
pub struct playerState_t {
    // opaque - from bg_public.h
}

#[repr(C)]
pub struct usercmd_t {
    // opaque - from bg_public.h
}

#[repr(C)]
pub struct cvar_t {
    // opaque - from qcommon.h
}

#[repr(C)]
pub struct vm_t {
    // opaque - from qcommon.h
}

#[repr(C)]
pub struct netadr_t {
    // opaque - from qcommon.h
}

#[repr(C)]
pub struct netchan_t {
    // opaque - from qcommon.h
}

#[repr(C)]
pub struct msg_t {
    // opaque - from qcommon.h
}

pub type fileHandle_t = c_int;

#[repr(C)]
pub struct trace_t {
    // opaque
}

pub type clipHandle_t = c_int;

#[repr(C)]
pub struct XBPlayerInfo {
    // opaque - Xbox-specific
}

#[repr(C)]
pub enum ForceReload_e {
    // opaque - from bg_public.h
}

//=============================================================================

pub const PERS_SCORE: c_int = 0;		// !!! MUST NOT CHANGE, SERVER AND
										// GAME BOTH REFERENCE !!!

pub const MAX_ENT_CLUSTERS: usize = 16;

// PORT: C conditional compilation on struct field types (_XBOX uses c_char, non-_XBOX uses c_int)
// is not representable in Rust's type system. Using standard (non-Xbox) layout.
#[repr(C)]
pub struct svEntity_s {
    pub worldSector: *mut worldSector_s,
    pub nextEntityInWorldSector: *mut svEntity_s,

    pub baseline: entityState_t,		// for delta compression of initial sighting
    pub numClusters: c_int,		// if -1, use headnode instead
    pub clusternums: [c_int; MAX_ENT_CLUSTERS],
    pub lastCluster: c_int,		// if all the clusters don't fit in clusternums
    pub areanum: c_int,
    pub areanum2: c_int,
    pub snapshotCounter: c_int,	// used to prevent double adding from portal views
}

#[repr(C)]
pub enum serverState_t {
    SS_DEAD,			// no map loaded
    SS_LOADING,			// spawning level entities
    SS_GAME,			// actively running
}

pub const MAX_MODELS: usize = 0; // stub - from qcommon defines
pub const MAX_CONFIGSTRINGS: usize = 0; // stub - from qcommon defines
pub const MAX_GENTITIES: usize = 0; // stub - from qcommon defines

// PORT: C conditional compilation on struct field types (_XBOX uses c_char, non-_XBOX uses c_int)
// is not representable in Rust's type system. Using standard (non-Xbox) layout.
#[repr(C)]
pub struct server_t {
    pub state: serverState_t,
    pub restarting: qboolean,			// if true, send configstring changes during SS_LOADING
    pub serverId: c_int,			// changes each server start
    pub restartedServerId: c_int,	// serverId before a map_restart
    pub checksumFeed: c_int,		//
    pub snapshotCounter: c_int,	// incremented for each snapshot built
    pub timeResidual: c_int,		// <= 1000 / sv_frame->value
    pub nextFrameTime: c_int,		// when time > nextFrameTime, process world
    pub models: [*mut cmodel_s; MAX_MODELS],
    pub configstrings: [*mut c_char; MAX_CONFIGSTRINGS],
    pub svEntities: [svEntity_s; MAX_GENTITIES],

    pub entityParsePoint: *mut c_char,	// used during game VM init

    // the game virtual machine will update these on init and changes
    pub gentities: *mut sharedEntity_t,
    pub gentitySize: c_int,
    pub num_entities: c_int,		// current number, <= MAX_GENTITIES

    pub gameClients: *mut playerState_t,
    pub gameClientSize: c_int,		// will be > sizeof(playerState_t) due to game private data

    pub restartTime: c_int,

    //rwwRMG - added:
    pub mLocalSubBSPIndex: c_int,
    pub mLocalSubBSPModelOffset: c_int,
    pub mLocalSubBSPEntityParsePoint: *mut c_char,

    pub mSharedMemory: *mut c_char,
}

pub const MAX_MAP_AREA_BYTES: usize = 0; // stub - from qcommon defines
pub const MAX_PACKET_ENTITIES: usize = 0; // stub - from qcommon defines

#[repr(C)]
pub struct clientSnapshot_t {
    pub areabytes: c_int,
    pub areabits: [u8; MAX_MAP_AREA_BYTES],		// portalarea visibility bits
    pub ps: playerState_t,
    pub vps: playerState_t, //vehicle I'm riding's playerstate (if applicable) -rww
    #[cfg(feature = "onebit_combo")]
    pub pDeltaOneBit: *mut c_int,
    #[cfg(feature = "onebit_combo")]
    pub pDeltaOneBitVeh: *mut c_int,
    #[cfg(feature = "onebit_combo")]
    pub pDeltaNumBit: *mut c_int,
    #[cfg(feature = "onebit_combo")]
    pub pDeltaNumBitVeh: *mut c_int,
    pub num_entities: c_int,
    pub first_entity: c_int,		// into the circular sv_packet_entities[]
										// the entities MUST be in increasing state number
										// order, otherwise the delta compression will fail
    pub messageSent: c_int,		// time the message was transmitted
    pub messageAcked: c_int,		// time the message was acked
    pub messageSize: c_int,		// used to rate drop packets
}

#[repr(C)]
pub enum clientState_t {
    CS_FREE,		// can be reused for a new connection
    CS_ZOMBIE,		// client has been disconnected, but don't reuse
                    // connection for a couple seconds
    CS_CONNECTED,	// has been assigned to a client_t, but no gamestate yet
    CS_PRIMED,		// gamestate has been sent, but client hasn't sent a usercmd
    CS_ACTIVE,		// client is fully in game
}

pub const MAX_INFO_STRING: usize = 0; // stub - from qcommon defines
pub const MAX_RELIABLE_COMMANDS: usize = 0; // stub - from qcommon defines
pub const MAX_STRING_CHARS: usize = 0; // stub - from qcommon defines
pub const MAX_NAME_LENGTH: usize = 0; // stub - from qcommon defines
pub const MAX_QPATH: usize = 0; // stub - from qcommon defines
pub const MAX_DOWNLOAD_WINDOW: usize = 0; // stub - from qcommon defines
pub const PACKET_BACKUP: usize = 0; // stub - from qcommon defines

pub type qboolean = c_int;

#[repr(C)]
pub struct client_s {
    pub state: clientState_t,
    pub userinfo: [c_char; MAX_INFO_STRING],		// name, etc

    pub sentGamedir: qboolean, //see if he has been sent an svc_setgame

    pub reliableCommands: [[c_char; MAX_STRING_CHARS]; MAX_RELIABLE_COMMANDS],
    pub reliableSequence: c_int,		// last added reliable message, not necesarily sent or acknowledged yet
    pub reliableAcknowledge: c_int,	// last acknowledged reliable message
    pub reliableSent: c_int,			// last sent reliable message, not necesarily acknowledged yet
    pub messageAcknowledge: c_int,

    pub gamestateMessageNum: c_int,	// netchan->outgoingSequence of gamestate
    pub challenge: c_int,

    pub lastUsercmd: usercmd_t,
    pub lastMessageNum: c_int,		// for delta compression
    pub lastClientCommand: c_int,	// reliable client message sequence
    pub lastClientCommandString: [c_char; MAX_STRING_CHARS],
    pub gentity: *mut sharedEntity_t,			// SV_GentityNum(clientnum)
    pub name: [c_char; MAX_NAME_LENGTH],			// extracted from userinfo, high bits masked

    // downloading
    #[cfg(not(feature = "xbox"))]	// No downloads on Xbox
    pub downloadName: [c_char; MAX_QPATH], // if not empty string, we are downloading
    #[cfg(not(feature = "xbox"))]
    pub download: fileHandle_t,			// file being downloaded
    #[cfg(not(feature = "xbox"))]
    pub downloadSize: c_int,		// total bytes (can't use EOF because of paks)
    #[cfg(not(feature = "xbox"))]
    pub downloadCount: c_int,		// bytes sent
    #[cfg(not(feature = "xbox"))]
    pub downloadClientBlock: c_int,	// last block we sent to the client, awaiting ack
    #[cfg(not(feature = "xbox"))]
    pub downloadCurrentBlock: c_int,	// current block number
    #[cfg(not(feature = "xbox"))]
    pub downloadXmitBlock: c_int,	// last block we xmited
    #[cfg(not(feature = "xbox"))]
    pub downloadBlocks: [*mut u8; MAX_DOWNLOAD_WINDOW],	// the buffers for the download blocks
    #[cfg(not(feature = "xbox"))]
    pub downloadBlockSize: [c_int; MAX_DOWNLOAD_WINDOW],
    #[cfg(not(feature = "xbox"))]
    pub downloadEOF: qboolean,		// We have sent the EOF block
    #[cfg(not(feature = "xbox"))]
    pub downloadSendTime: c_int,	// time we last got an ack from the client

    pub deltaMessage: c_int,		// frame last client usercmd message
    pub nextReliableTime: c_int,	// svs.time when another reliable command will be allowed
    pub lastPacketTime: c_int,		// svs.time when packet was last received
    pub lastConnectTime: c_int,		// svs.time when connection started
    pub nextSnapshotTime: c_int,	// send another snapshot when svs.time >= nextSnapshotTime
    pub rateDelayed: qboolean,		// true if nextSnapshotTime was set based on rate instead of snapshotMsec
    pub timeoutCount: c_int,		// must timeout a few frames in a row so debugging doesn't break
    pub frames: [clientSnapshot_t; PACKET_BACKUP],	// updates can be delta'd from here
    pub ping: c_int,
    pub rate: c_int,				// bytes / second
    pub snapshotMsec: c_int,		// requests a snapshot every snapshotMsec unless rate choked
    pub pureAuthentic: c_int,
    pub netchan: netchan_t,

    pub lastUserInfoChange: c_int, //if > svs.time && count > x, deny change -rww
    pub lastUserInfoCount: c_int, //allow a certain number of changes within a certain time period -rww

    #[cfg(feature = "xbox")]
    pub refIndex: c_int,			// Copy of refIndex in xbOnlineInfo.xbPlayerList[]
    #[cfg(feature = "xbox")]
    pub usePrivateSlot: qboolean,		// Was this person eligble for a private slot when they joined?
}

//=============================================================================


// MAX_CHALLENGES is made large to prevent a denial
// of service attack that could cycle all of them
// out before legitimate users connected
pub const MAX_CHALLENGES: usize = 1024;

pub const AUTHORIZE_TIMEOUT: c_int = 5000;

#[repr(C)]
pub struct challenge_t {
    pub adr: netadr_t,
    pub challenge: c_int,
    pub time: c_int,				// time the last packet was sent to the autherize server
    pub pingTime: c_int,			// time the challenge response was sent to client
    pub firstTime: c_int,			// time the adr was first used, for authorize timeout checks
    pub connected: qboolean,
}


pub const MAX_MASTERS: usize = 8;				// max recipients for heartbeat packets


// this structure will be cleared only when the game dll changes
#[repr(C)]
pub struct serverStatic_t {
    pub initialized: qboolean,				// sv_init has completed

    pub time: c_int,						// will be strictly increasing across level changes

    pub snapFlagServerBit: c_int,			// ^= SNAPFLAG_SERVERCOUNT every SV_SpawnServer()

    pub clients: *mut client_s,					// [sv_maxclients->integer];
    pub numSnapshotEntities: c_int,		// sv_maxclients->integer*PACKET_BACKUP*MAX_PACKET_ENTITIES
    pub nextSnapshotEntities: c_int,		// next snapshotEntities to use
    pub snapshotEntities: *mut entityState_t,		// [numSnapshotEntities]
    pub nextHeartbeatTime: c_int,
    pub challenges: [challenge_t; MAX_CHALLENGES],	// to prevent invalid IPs from connecting
    pub redirectAddress: netadr_t,			// for rcon return messages

    pub authorizeAddress: netadr_t,			// for rcon return messages

    #[cfg(feature = "xbox")]
    pub clientRefNum: c_int,				// Index into xbonlineinfo array
}

//=============================================================================

extern "C" {
    pub static mut svs: serverStatic_t;				// persistant server info across maps
    pub static mut sv: server_t;					// cleared each map
    pub static mut gvm: *mut vm_t;				// game virtual machine
}

pub const MAX_MASTER_SERVERS: usize = 5;

extern "C" {
    pub static mut sv_fps: *mut cvar_t;
    pub static mut sv_timeout: *mut cvar_t;
    pub static mut sv_zombietime: *mut cvar_t;
    pub static mut sv_rconPassword: *mut cvar_t;
    pub static mut sv_privatePassword: *mut cvar_t;
    pub static mut sv_allowDownload: *mut cvar_t;
    pub static mut sv_maxclients: *mut cvar_t;
    pub static mut sv_privateClients: *mut cvar_t;
    pub static mut sv_hostname: *mut cvar_t;
    pub static mut sv_master: [*mut cvar_t; MAX_MASTER_SERVERS];
    pub static mut sv_reconnectlimit: *mut cvar_t;
    pub static mut sv_showghoultraces: *mut cvar_t;
    pub static mut sv_showloss: *mut cvar_t;
    pub static mut sv_padPackets: *mut cvar_t;
    pub static mut sv_killserver: *mut cvar_t;
    pub static mut sv_mapname: *mut cvar_t;
    pub static mut sv_mapChecksum: *mut cvar_t;
    pub static mut sv_serverid: *mut cvar_t;
    pub static mut sv_maxRate: *mut cvar_t;
    pub static mut sv_minPing: *mut cvar_t;
    pub static mut sv_maxPing: *mut cvar_t;
    pub static mut sv_gametype: *mut cvar_t;
    pub static mut sv_pure: *mut cvar_t;
    pub static mut sv_floodProtect: *mut cvar_t;
    pub static mut sv_needpass: *mut cvar_t;
    #[cfg(feature = "use_cd_key")]
    pub static mut sv_allowAnonymous: *mut cvar_t;
}

//===========================================================

//
// sv_main.c
//
extern "C" {
    pub fn SV_FinalMessage(message: *mut c_char);
    pub fn SV_SendServerCommand(cl: *mut client_s, fmt: *const c_char, ...);


    pub fn SV_AddOperatorCommands();
    pub fn SV_RemoveOperatorCommands();


    pub fn SV_MasterHeartbeat();
    pub fn SV_MasterShutdown();
}




//
// sv_init.c
//
extern "C" {
    pub fn SV_SetConfigstring(index: c_int, val: *const c_char);
    pub fn SV_GetConfigstring(index: c_int, buffer: *mut c_char, bufferSize: c_int);
    pub fn SV_AddConfigstring(name: *const c_char, start: c_int, max: c_int) -> c_int;

    pub fn SV_SetUserinfo(index: c_int, val: *const c_char);
    pub fn SV_GetUserinfo(index: c_int, buffer: *mut c_char, bufferSize: c_int);

    pub fn SV_ChangeMaxClients();
    pub fn SV_SpawnServer(server: *mut c_char, killBots: qboolean, eForceReload: ForceReload_e);
}



//
// sv_client.c
//
extern "C" {
    pub fn SV_GetChallenge(from: netadr_t);

    pub fn SV_DirectConnect(from: netadr_t);

    pub fn SV_AuthorizeIpPacket(from: netadr_t);

    pub fn SV_SendClientMapChange(client: *mut client_s);
    pub fn SV_ExecuteClientMessage(cl: *mut client_s, msg: *mut msg_t);
    pub fn SV_UserinfoChanged(cl: *mut client_s);

    pub fn SV_ClientEnterWorld(client: *mut client_s, cmd: *mut usercmd_t);
    pub fn SV_DropClient(drop: *mut client_s, reason: *const c_char);

    pub fn SV_ExecuteClientCommand(cl: *mut client_s, s: *const c_char, clientOK: qboolean);
    pub fn SV_ClientThink(cl: *mut client_s, cmd: *mut usercmd_t);

    pub fn SV_WriteDownloadToClient(cl: *mut client_s, msg: *mut msg_t);

    // Need to broadcast info about clients on join/leave
    #[cfg(feature = "xbox")]
    pub fn SV_SendClientNewPeer(client: *mut client_s, info: *mut XBPlayerInfo);
    #[cfg(feature = "xbox")]
    pub fn SV_SendClientRemovePeer(client: *mut client_s, index: c_int);
    #[cfg(feature = "xbox")]
    pub fn SV_SendClientXbInfo(client: *mut client_s);
}

//
// sv_ccmds.c
//
extern "C" {
    pub fn SV_Heartbeat_f();
}

//
// sv_snapshot.c
//
extern "C" {
    pub fn SV_AddServerCommand(client: *mut client_s, cmd: *const c_char);
    pub fn SV_UpdateServerCommandsToClient(client: *mut client_s, msg: *mut msg_t);
    pub fn SV_WriteFrameToClient(client: *mut client_s, msg: *mut msg_t);
    pub fn SV_SendMessageToClient(msg: *mut msg_t, client: *mut client_s);
    pub fn SV_SendClientMessages();
    pub fn SV_SendClientSnapshot(client: *mut client_s);
}

//
// sv_game.c
//
extern "C" {
    pub fn SV_NumForGentity(ent: *mut sharedEntity_t) -> c_int;
    pub fn SV_GentityNum(num: c_int) -> *mut sharedEntity_t;
    pub fn SV_GameClientNum(num: c_int) -> *mut playerState_t;
    pub fn SV_SvEntityForGentity(gEnt: *mut sharedEntity_t) -> *mut svEntity_s;
    pub fn SV_GEntityForSvEntity(svEnt: *mut svEntity_s) -> *mut sharedEntity_t;
    pub fn SV_InitGameProgs();
    pub fn SV_ShutdownGameProgs();
    pub fn SV_RestartGameProgs();
    pub fn SV_inPVS(p1: *const [f32; 3], p2: *const [f32; 3]) -> qboolean;
}

//
// sv_bot.c
//
extern "C" {
    pub fn SV_BotFrame(time: c_int);
    pub fn SV_BotAllocateClient() -> c_int;
    pub fn SV_BotFreeClient(clientNum: c_int);

    pub fn SV_BotInitCvars();
    pub fn SV_BotLibSetup() -> c_int;
    pub fn SV_BotLibShutdown() -> c_int;
    pub fn SV_BotGetSnapshotEntity(client: c_int, ent: c_int) -> c_int;
    pub fn SV_BotGetConsoleMessage(client: c_int, buf: *mut c_char, size: c_int) -> c_int;

    pub fn Bot_GetMemoryGame(size: c_int) -> *mut core::ffi::c_void;
    pub fn Bot_FreeMemoryGame(ptr: *mut core::ffi::c_void);

    pub fn BotImport_DebugPolygonCreate(color: c_int, numPoints: c_int, points: *mut [f32; 3]) -> c_int;
    pub fn BotImport_DebugPolygonDelete(id: c_int);
}

//============================================================
//
// high level object sorting to reduce interaction tests
//

extern "C" {
    pub fn SV_ClearWorld();
    // called after the world model has been loaded, before linking any entities

    pub fn SV_UnlinkEntity(ent: *mut sharedEntity_t);
    // call before removing an entity, and before trying to move one,
    // so it doesn't clip against itself

    pub fn SV_LinkEntity(ent: *mut sharedEntity_t);
    // Needs to be called any time an entity changes origin, mins, maxs,
    // or solid.  Automatically unlinks if needed.
    // sets ent->v.absmin and ent->v.absmax
    // sets ent->leafnums[] for pvs determination even if the entity
    // is not solid


    pub fn SV_ClipHandleForEntity(ent: *const sharedEntity_t) -> clipHandle_t;


    pub fn SV_SectorList_f();


    pub fn SV_AreaEntities(mins: *const [f32; 3], maxs: *const [f32; 3], entityList: *mut c_int, maxcount: c_int) -> c_int;
    // fills in a table of entity numbers with entities that have bounding boxes
    // that intersect the given area.  It is possible for a non-axial bmodel
    // to be returned that doesn't actually intersect the area on an exact
    // test.
    // returns the number of pointers filled in
    // The world entity is never returned in this list.


    pub fn SV_PointContents(p: *const [f32; 3], passEntityNum: c_int) -> c_int;
    // returns the CONTENTS_* value from the world and all entities at the given point.


    pub fn SV_Trace(results: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passEntityNum: c_int, contentmask: c_int, capsule: c_int, traceFlags: c_int, useLod: c_int);
    // mins and maxs are relative

    // if the entire move stays in a solid volume, trace.allsolid will be set,
    // trace.startsolid will be set, and trace.fraction will be 0

    // if the starting point is in a solid, it will be allowed to move out
    // to an open area

    // passEntityNum is explicitly excluded from clipping checks (normally ENTITYNUM_NONE)


    pub fn SV_ClipToEntity(trace: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], entityNum: c_int, contentmask: c_int, capsule: c_int);
    // clip to a specific entity
}

//
// sv_net_chan.c
//
extern "C" {
    pub fn SV_Netchan_Transmit(client: *mut client_s, msg: *mut msg_t);	//int length, const byte *data );
    pub fn SV_Netchan_TransmitNextFragment(chan: *mut netchan_t);
    pub fn SV_Netchan_Process(client: *mut client_s, msg: *mut msg_t) -> qboolean;
}
