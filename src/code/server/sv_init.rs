// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::c_int;
use crate::codemp::game::q_shared_h::qboolean;

// Forward declarations and stubs for types defined elsewhere.
// These are declared as void pointers since the full definitions are not needed here.
use core::ffi::c_void;

// Extern declarations for external C functions called from this module
extern "C" {
    fn Com_Error(error_level: c_int, fmt: *const u8, ...) -> !;
    fn Com_Printf(fmt: *const u8, ...);
    fn Com_Error_arg(error_level: c_int, fmt: *const u8, arg: c_int) -> !;

    fn strcmp(s1: *const u8, s2: *const u8) -> c_int;
    fn Z_Free(ptr: *mut c_void);
    fn Z_Malloc(size: c_int, tag: c_int, clear: qboolean) -> *mut c_void;
    fn Z_TagFree(tag: c_int);
    fn Z_Validate();
    fn Z_DumpMemMap_f();
    fn Z_Details_f();
    fn Z_TagPointers(tag: c_int);

    fn CopyString(string: *const u8) -> *mut u8;
    fn Q_strncpyz(dest: *mut u8, src: *const u8, destsize: c_int);
    fn Q_stricmp(s1: *const u8, s2: *const u8) -> c_int;

    fn Cvar_Get(var_name: *const u8, var_value: *const u8, flags: c_int) -> *mut c_void;
    fn Cvar_Set(var_name: *const u8, value: *const u8);
    fn Cvar_SetValue(var_name: *const u8, value: f32);
    fn Cvar_VariableIntegerValue(var_name: *const u8) -> c_int;
    fn Cvar_VariableString(var_name: *const u8) -> *const u8;
    fn Cvar_InfoString(bit: c_int) -> *const u8;
    fn Cvar_Defrag();

    fn va(fmt: *const u8, ...) -> *const u8;

    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn Hunk_Clear();
    fn Hunk_SetMark();

    fn CM_SameMap(server: *mut u8) -> qboolean;
    fn CM_HasTerrain() -> qboolean;
    fn CM_ClearMap();
    fn CM_LoadMap(mapname: *const u8, clientload: qboolean, checksum: *mut c_int, ...);
    fn CM_CleanLeafCache();

    fn RE_RegisterMedia_LevelLoadBegin(
        mapname: *const u8,
        eForceReload: c_int,
        bAllowScreenDissolve: qboolean,
    );
    fn RE_LoadWorldMap(mapname: *const u8);
    fn R_InitImages();
    fn R_InitShaders();
    fn R_ModelInit();
    fn R_DeleteTextures();
    fn CL_MapLoading();
    fn CL_StartHunkUsers();

    fn Music_SetLevelName(name: *const u8);

    fn SV_AddOperatorCommands();
    fn SV_RemoveOperatorCommands();
    fn SV_SendServerCommand(cl: *mut c_void, fmt: *const u8, ...);
    fn SV_SendClientSnapshot(cl: *mut c_void);
    fn SV_ShutdownGameProgs(restart: qboolean);
    fn SV_InitGameProgs();
    fn SV_ClearWorld();
    fn SV_DropClient(cl: *mut c_void, reason: *const u8);
    fn SV_FreeClient(cl: *mut c_void);
    fn SV_GentityNum(num: c_int) -> *mut gentity_s;

    // Game engine virtual machine callbacks
    fn G_ClientConnect(clientNum: c_int, firstTime: qboolean, isBot: qboolean) -> *const u8;
    fn G_RunFrame(time: c_int);
    fn G_ConnectNavs(mapname: *const u8, checksum: c_int);

    fn G2API_SetTime(time: c_int, time_type: c_int);

    // Xbox-specific declarations
    #[cfg(target_os = "windows")]
    fn qglDisable(cap: c_int);

    #[cfg(target_os = "windows")]
    fn R_ModelFree();
    #[cfg(target_os = "windows")]
    fn Sys_IORequestQueueClear();
    #[cfg(target_os = "windows")]
    fn Music_Free();
    #[cfg(target_os = "windows")]
    fn AS_FreePartial();
    #[cfg(target_os = "windows")]
    fn G_ASPreCacheFree();
    #[cfg(target_os = "windows")]
    fn Ghoul2InfoArray_Free();
    #[cfg(target_os = "windows")]
    fn Ghoul2InfoArray_Reset();
    #[cfg(target_os = "windows")]
    fn Menu_Reset();
    #[cfg(target_os = "windows")]
    fn G2_FreeRag();
    #[cfg(target_os = "windows")]
    fn ClearAllNavStructures();
    #[cfg(target_os = "windows")]
    fn ClearModelsAlreadyDone();
    #[cfg(target_os = "windows")]
    fn CL_FreeServerCommands();
    #[cfg(target_os = "windows")]
    fn CL_FreeReliableCommands();
    #[cfg(target_os = "windows")]
    fn CM_Free();
    #[cfg(target_os = "windows")]
    fn ShaderEntryPtrs_Clear();
    #[cfg(target_os = "windows")]
    fn G_FreeRoffs();
}

// Constants
const ERR_DROP: c_int = 1;
const ERR_FATAL: c_int = 2;

const MAX_CONFIGSTRINGS: usize = 256;
const TAG_CLIENTS: c_int = 11;
const TAG_G_ALLOC: c_int = 5;
const TAG_UI_ALLOC: c_int = 6;

const CVAR_SERVERINFO: c_int = 2;
const CVAR_SYSTEMINFO: c_int = 4;
const CVAR_ROM: c_int = 8;
const CVAR_TEMP: c_int = 16;

const SS_LOADING: c_int = 0;
const SS_GAME: c_int = 1;

const CS_SYSTEMINFO: c_int = 0;
const CS_SERVERINFO: c_int = 1;

const CS_CONNECTED: c_int = 2;

const G2T_SV_TIME: c_int = 2;

const PROTOCOL_VERSION: c_int = 26;

const GL_VSYNC: c_int = 0x2001;

// Structs that need to be declared for this module.
// These are minimal declarations needed for function signatures.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct client_s {
    // Minimal definition - fields used in this module
    pub state: c_int,
    pub lastPacketTime: c_int,
    pub lastConnectTime: c_int,
    pub nextSnapshotTime: c_int,
    pub userinfo: [u8; 256], // Typical userinfo buffer size
}

#[repr(C)]
pub struct entityState_s {
    // Minimal definition
    pub number: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct svEntity_s {
    pub baseline: entityState_s,
}

#[repr(C)]
pub struct gentity_s {
    pub s: entityState_s,
    pub inuse: qboolean,
    pub linked: qboolean,
}

#[repr(C)]
pub struct serverStatic_s {
    pub clients: *mut client_s,
    pub numSnapshotEntities: c_int,
    pub snapshotEntities: *mut entityState_s,
    pub initialized: qboolean,
    pub nextHeartbeatTime: c_int,
}

#[repr(C)]
pub struct server_s {
    pub configstrings: [*mut u8; MAX_CONFIGSTRINGS],
    pub svEntities: [svEntity_s; 256], // Typical entity count
    pub time: c_int,
    pub state: c_int,
    pub serverId: c_int,
}

// Mini-heap class stub
pub struct CMiniHeap {
    _private: c_void,
}

impl CMiniHeap {
    pub fn ResetHeap(&mut self) {
        // Stub implementation
    }
}

// Global declarations
// Ghoul2 Insert Start
/// G2VertSpaceServer: Ghoul2 vertex space for server-side transforms used in collision detection.
pub static mut G2VertSpaceServer: *mut CMiniHeap = core::ptr::null_mut();
// Ghoul2 Insert End

// External global declarations
extern "C" {
    pub static mut sv: server_s;
    pub static mut svs: serverStatic_s;
    pub static mut ge: *mut c_void; // Game engine pointer
    pub static mut com_version: *mut c_void;
    pub static mut com_sv_running: *mut c_void;
    pub static mut com_errorEntered: qboolean;
    pub static mut com_frameTime: c_int;
    pub static mut cvar_modifiedFlags: c_int;
    pub static mut sv_mapname: *mut c_void;
    pub static mut sv_serverid: *mut c_void;
    pub static mut sv_fps: *mut c_void;
    pub static mut sv_timeout: *mut c_void;
    pub static mut sv_zombietime: *mut c_void;
    pub static mut sv_spawntarget: *mut c_void;
    pub static mut sv_reconnectlimit: *mut c_void;
    pub static mut sv_showloss: *mut c_void;
    pub static mut sv_killserver: *mut c_void;
    pub static mut sv_mapChecksum: *mut c_void;
    pub static mut sv_testsave: *mut c_void;
    pub static mut sv_compress_saved_games: *mut c_void;
    pub static mut numVehicles: c_int;
}

/*
===============
SV_SetConfigstring

===============
*/
pub unsafe fn SV_SetConfigstring(index: c_int, val: *const u8) {
    let val = if val.is_null() {
        b"\0".as_ptr()
    } else {
        val
    };

    if index < 0 || (index as usize) >= MAX_CONFIGSTRINGS {
        Com_Error(
            ERR_DROP,
            b"SV_SetConfigstring: bad index %i\n\0".as_ptr(),
            index,
        );
    }

    if val.is_null() {
        let empty = b"\0".as_ptr();
        if strcmp(empty, (*core::ptr::addr_of!(sv)).configstrings[index as usize]) == 0 {
            return;
        }
    } else {
        if strcmp(val, (*core::ptr::addr_of!(sv)).configstrings[index as usize]) == 0 {
            return;
        }
    }

    // change the string in sv
    Z_Free((*core::ptr::addr_of!(sv)).configstrings[index as usize] as *mut c_void);
    (*core::ptr::addr_of_mut!(sv)).configstrings[index as usize] = CopyString(val);

    // send it to all the clients if we aren't
    // spawning a new server
    if (*core::ptr::addr_of!(sv)).state == SS_GAME {
        SV_SendServerCommand(core::ptr::null_mut(), b"cs %i \"%s\"\n\0".as_ptr(), index, val);
    }
}

/*
===============
SV_GetConfigstring

===============
*/
pub unsafe fn SV_GetConfigstring(index: c_int, buffer: *mut u8, bufferSize: c_int) {
    if bufferSize < 1 {
        Com_Error(
            ERR_DROP,
            b"SV_GetConfigstring: bufferSize == %i\0".as_ptr(),
            bufferSize,
        );
    }
    if index < 0 || (index as usize) >= MAX_CONFIGSTRINGS {
        Com_Error(
            ERR_DROP,
            b"SV_GetConfigstring: bad index %i\n\0".as_ptr(),
            index,
        );
    }
    if (*core::ptr::addr_of!(sv)).configstrings[index as usize].is_null() {
        *buffer = 0;
        return;
    }

    Q_strncpyz(buffer, (*core::ptr::addr_of!(sv)).configstrings[index as usize], bufferSize);
}

/*
===============
SV_SetUserinfo

===============
*/
pub unsafe fn SV_SetUserinfo(index: c_int, val: *const u8) {
    let val = if val.is_null() {
        b"\0".as_ptr()
    } else {
        val
    };

    if index < 0 || index >= 1 {
        Com_Error(
            ERR_DROP,
            b"SV_SetUserinfo: bad index %i\n\0".as_ptr(),
            index,
        );
    }

    Q_strncpyz(
        (*core::ptr::addr_of!(svs)).clients[index as usize].userinfo.as_mut_ptr(),
        val,
        (*core::ptr::addr_of!(svs)).clients[index as usize]
            .userinfo
            .len() as c_int,
    );
}

/*
===============
SV_GetUserinfo

===============
*/
pub unsafe fn SV_GetUserinfo(index: c_int, buffer: *mut u8, bufferSize: c_int) {
    if bufferSize < 1 {
        Com_Error(
            ERR_DROP,
            b"SV_GetUserinfo: bufferSize == %i\0".as_ptr(),
            bufferSize,
        );
    }
    if index < 0 || index >= 1 {
        Com_Error(
            ERR_DROP,
            b"SV_GetUserinfo: bad index %i\n\0".as_ptr(),
            index,
        );
    }
    Q_strncpyz(
        buffer,
        (*core::ptr::addr_of!(svs)).clients[index as usize].userinfo.as_ptr(),
        bufferSize,
    );
}

/*
================
SV_CreateBaseline

Entity baselines are used to compress non-delta messages
to the clients -- only the fields that differ from the
baseline will be transmitted
================
*/
pub unsafe fn SV_CreateBaseline() {
    let mut svent: *mut gentity_s;
    let mut entnum: c_int = 0;

    // NOTE: Accessing ge->num_entities would require full ge struct definition.
    // Using a stub loop for structural equivalence.
    // In practice, this would iterate through ge->num_entities.
    loop {
        if entnum >= 256 {
            // Arbitrary limit - would be ge->num_entities in actual implementation
            break;
        }

        svent = SV_GentityNum(entnum);
        if (*svent).inuse == 0 {
            entnum += 1;
            continue;
        }
        if (*svent).linked == 0 {
            entnum += 1;
            continue;
        }
        (*svent).s.number = entnum;

        //
        // take current state as baseline
        //
        (*core::ptr::addr_of_mut!(sv)).svEntities[entnum as usize].baseline = (*svent).s;

        entnum += 1;
    }
}

/*
===============
SV_Startup

Called when a game is about to begin
===============
*/
pub unsafe fn SV_Startup() {
    if (*core::ptr::addr_of!(svs)).initialized != 0 {
        Com_Error(
            ERR_FATAL,
            b"SV_Startup: svs.initialized\0".as_ptr(),
        );
    }

    (*core::ptr::addr_of_mut!(svs)).clients =
        Z_Malloc(
            core::mem::size_of::<client_s>() as c_int * 1,
            TAG_CLIENTS,
            1,
        ) as *mut client_s;
    (*core::ptr::addr_of_mut!(svs)).numSnapshotEntities = 2 * 4 * 64;
    (*core::ptr::addr_of_mut!(svs)).initialized = 1;

    Cvar_Set(b"sv_running\0".as_ptr(), b"1\0".as_ptr());
}

#[cfg(target_os = "windows")]
pub unsafe fn SV_ClearLastLevel() {
    Menu_Reset();
    Z_TagFree(TAG_G_ALLOC);
    Z_TagFree(TAG_UI_ALLOC);
    G_FreeRoffs();
    R_ModelFree();
    Music_Free();
    Sys_IORequestQueueClear();
    AS_FreePartial();
    G_ASPreCacheFree();
    Ghoul2InfoArray_Free();
    G2_FreeRag();
    ClearAllNavStructures();
    ClearModelsAlreadyDone();
    CL_FreeServerCommands();
    CL_FreeReliableCommands();
    CM_Free();
    ShaderEntryPtrs_Clear();

    numVehicles = 0;

    if !(*core::ptr::addr_of!(svs)).clients.is_null() {
        SV_FreeClient((*core::ptr::addr_of!(svs)).clients as *mut c_void);
    }
}

/*
================
SV_SpawnServer

Change the server to a new map, taking all connected
clients along with it.
================
*/
pub unsafe fn SV_SpawnServer(
    server: *mut u8,
    eForceReload: c_int,
    bAllowScreenDissolve: qboolean,
) {
    let mut checksum: c_int = 0;

    // The following fixes for potential issues only work on Xbox
    #[cfg(target_os = "windows")]
    {
        extern "C" {
            pub static mut stop_icarus: qboolean;
            pub static mut player_locked: qboolean;
            pub static mut MatrixMode: qboolean;
        }
        stop_icarus = 0;

        // Broken scripts may leave the player locked.  I think that's always bad.
        player_locked = 0;

        // If you quit while in Matrix Mode, this never gets cleared!
        MatrixMode = 0;

        // Temporary code to turn on HDR effect for specific maps only
        if Q_stricmp(server as *const u8, b"t3_rift\0".as_ptr()) == 0 {
            Cvar_Set(b"r_hdreffect\0".as_ptr(), b"1\0".as_ptr());
        } else {
            Cvar_Set(b"r_hdreffect\0".as_ptr(), b"0\0".as_ptr());
        }
    }

    RE_RegisterMedia_LevelLoadBegin(server as *const u8, eForceReload, bAllowScreenDissolve);

    Cvar_SetValue(b"cl_paused\0".as_ptr(), 0.0);
    Cvar_Set(b"timescale\0".as_ptr(), b"1\0".as_ptr()); //jic we were skipping

    // shut down the existing game if it is running
    SV_ShutdownGameProgs(1);

    Com_Printf(
        b"------ Server Initialization ------\n%s\n\0".as_ptr(),
        (*com_version).cast::<u8>(),
    );
    Com_Printf(
        b"Server: %s\n\0".as_ptr(),
        server,
    );

    #[cfg(target_os = "windows")]
    {
        // disable vsync during load for speed
        qglDisable(GL_VSYNC);
    }

    // don't let sound stutter and dump all stuff on the hunk
    CL_MapLoading();

    if CM_SameMap(server) == 0 {
        // rww - only clear if not loading the same map
        CM_ClearMap();
    }
    #[cfg(not(target_os = "windows"))]
    {
        if CM_HasTerrain() != 0 {
            // always clear when going between maps with terrain
            CM_ClearMap();
        }
    }

    // Miniheap never changes sizes, so I just put it really early in mem.
    if !G2VertSpaceServer.is_null() {
        (*G2VertSpaceServer).ResetHeap();
    }

    #[cfg(target_os = "windows")]
    {
        // Deletes all textures
        R_DeleteTextures();
    }
    Hunk_Clear();

    // Moved up from below to help reduce fragmentation
    if !(*core::ptr::addr_of!(svs)).snapshotEntities.is_null() {
        Z_Free((*core::ptr::addr_of!(svs)).snapshotEntities as *mut c_void);
        (*core::ptr::addr_of_mut!(svs)).snapshotEntities = core::ptr::null_mut();
    }

    // wipe the entire per-level structure
    // Also moved up, trying to do all freeing before new allocs
    for i in 0..MAX_CONFIGSTRINGS {
        if !(*core::ptr::addr_of!(sv)).configstrings[i].is_null() {
            Z_Free((*core::ptr::addr_of!(sv)).configstrings[i] as *mut c_void);
            (*core::ptr::addr_of_mut!(sv)).configstrings[i] = core::ptr::null_mut();
        }
    }

    #[cfg(target_os = "windows")]
    {
        SV_ClearLastLevel();
    }

    // Collect all the small allocations done by the cvar system
    // This frees, then allocates. Make it the last thing before other
    // allocations begin!
    Cvar_Defrag();

    /*
            This is useful for debugging memory fragmentation.  Please don't
           remove it.
    */
    #[cfg(target_os = "windows")]
    {
        // We've over-freed the info array above, this puts it back into a working state
        Ghoul2InfoArray_Reset();

        extern "C" {
            fn Z_DumpMemMap_f();
            fn Z_Details_f();
            fn Z_TagPointers(tag: c_int);
        }
        Z_DumpMemMap_f();
        //	Z_TagPointers(TAG_ALL);
        Z_Details_f();
    }

    // init client structures and svs.numSnapshotEntities
    // This is moved down quite a bit, but should be safe. And keeps
    // svs.clients right at the beginning of memory
    if Cvar_VariableIntegerValue(b"sv_running\0".as_ptr()) == 0 {
        SV_Startup();
    }

    // clear out those shaders, images and Models
    R_InitImages();
    R_InitShaders();
    R_ModelInit();

    // allocate the snapshot entities
    (*core::ptr::addr_of_mut!(svs)).snapshotEntities = Z_Malloc(
        (core::mem::size_of::<entityState_s>() as c_int)
            * (*core::ptr::addr_of!(svs)).numSnapshotEntities,
        TAG_CLIENTS,
        1,
    ) as *mut entityState_s;

    Music_SetLevelName(server as *const u8);

    // toggle the server bit so clients can detect that a
    // server has changed
    //!@	svs.snapFlagServerBit ^= SNAPFLAG_SERVERCOUNT;

    // set nextmap to the same map, but it may be overriden
    // by the game startup or another console command
    Cvar_Set(b"nextmap\0".as_ptr(), va(b"map %s\0".as_ptr(), server));

    memset(
        core::ptr::addr_of_mut!(sv) as *mut c_void,
        0,
        core::mem::size_of::<server_s>(),
    );

    for i in 0..MAX_CONFIGSTRINGS {
        (*core::ptr::addr_of_mut!(sv)).configstrings[i] = CopyString(b"\0".as_ptr());
    }

    (*core::ptr::addr_of_mut!(sv)).time = 1000;
    G2API_SetTime((*core::ptr::addr_of!(sv)).time, G2T_SV_TIME);

    #[cfg(target_os = "windows")]
    {
        CL_StartHunkUsers();
        CM_LoadMap(
            va(b"maps/%s.bsp\0".as_ptr(), server),
            0,
            core::ptr::addr_of_mut!(checksum),
        );
        RE_LoadWorldMap(va(b"maps/%s.bsp\0".as_ptr(), server));
    }
    #[cfg(not(target_os = "windows"))]
    {
        CM_LoadMap(
            va(b"maps/%s.bsp\0".as_ptr(), server),
            0,
            core::ptr::addr_of_mut!(checksum),
            0,
        );
    }

    // set serverinfo visible name
    Cvar_Set(b"mapname\0".as_ptr(), server as *const u8);

    Cvar_Set(b"sv_mapChecksum\0".as_ptr(), va(b"%i\0".as_ptr(), checksum));

    // serverid should be different each time
    (*core::ptr::addr_of_mut!(sv)).serverId = com_frameTime;
    Cvar_Set(
        b"sv_serverid\0".as_ptr(),
        va(b"%i\0".as_ptr(), (*core::ptr::addr_of!(sv)).serverId),
    );

    // clear physics interaction links
    SV_ClearWorld();

    // media configstring setting should be done during
    // the loading stage, so connected clients don't have
    // to load during actual gameplay
    (*core::ptr::addr_of_mut!(sv)).state = SS_LOADING;

    // load and spawn all other entities
    SV_InitGameProgs();

    // run a few frames to allow everything to settle
    for _i in 0..3 {
        G_RunFrame((*core::ptr::addr_of!(sv)).time);
        (*core::ptr::addr_of_mut!(sv)).time += 100;
        G2API_SetTime((*core::ptr::addr_of!(sv)).time, G2T_SV_TIME);
    }
    G_ConnectNavs(
        Cvar_VariableString(b"mapname\0".as_ptr()),
        Cvar_VariableIntegerValue(b"sv_mapChecksum\0".as_ptr()),
    );

    // create a baseline for more efficient communications
    SV_CreateBaseline();

    for i in 0..1 {
        // clear all time counters, because we have reset sv.time
        (*core::ptr::addr_of_mut!(svs)).clients[i as usize].lastPacketTime = 0;
        (*core::ptr::addr_of_mut!(svs)).clients[i as usize].lastConnectTime = 0;
        (*core::ptr::addr_of_mut!(svs)).clients[i as usize].nextSnapshotTime = 0;

        // send the new gamestate to all connected clients
        if (*core::ptr::addr_of!(svs)).clients[i as usize].state >= CS_CONNECTED {
            let denied: *const u8;

            // connect the client again
            denied = G_ClientConnect(i, 0, 0); // firstTime = qfalse, qbFromSavedGame
            if !denied.is_null() {
                // this generally shouldn't happen, because the client
                // was connected before the level change
                SV_DropClient(
                    (*core::ptr::addr_of_mut!(svs)).clients.add(i as usize) as *mut c_void,
                    denied,
                );
            } else {
                (*core::ptr::addr_of_mut!(svs)).clients[i as usize].state = CS_CONNECTED;
                // when we get the next packet from a connected client,
                // the new gamestate will be sent
            }
        }
    }

    // run another frame to allow things to look at all connected clients
    G_RunFrame((*core::ptr::addr_of!(sv)).time);
    (*core::ptr::addr_of_mut!(sv)).time += 100;
    G2API_SetTime((*core::ptr::addr_of!(sv)).time, G2T_SV_TIME);

    // save systeminfo and serverinfo strings
    SV_SetConfigstring(CS_SYSTEMINFO, Cvar_InfoString(CVAR_SYSTEMINFO));
    cvar_modifiedFlags &= !CVAR_SYSTEMINFO;

    SV_SetConfigstring(CS_SERVERINFO, Cvar_InfoString(CVAR_SERVERINFO));
    cvar_modifiedFlags &= !CVAR_SERVERINFO;

    // any media configstring setting now should issue a warning
    // and any configstring changes should be reliably transmitted
    // to all clients
    (*core::ptr::addr_of_mut!(sv)).state = SS_GAME;

    // send a heartbeat now so the master will get up to date info
    (*core::ptr::addr_of_mut!(svs)).nextHeartbeatTime = -9999999;

    Hunk_SetMark();
    Z_Validate();
    Z_Validate();
    Z_Validate();

    Com_Printf(b"-----------------------------------\n\0".as_ptr());
}

/*
===============
SV_Init

Only called at main exe startup, not for each game
===============
*/
pub unsafe fn SV_Init() {
    SV_AddOperatorCommands();

    // serverinfo vars
    Cvar_Get(
        b"protocol\0".as_ptr(),
        va(b"%i\0".as_ptr(), PROTOCOL_VERSION),
        CVAR_SERVERINFO | CVAR_ROM,
    );
    sv_mapname = Cvar_Get(
        b"mapname\0".as_ptr(),
        b"nomap\0".as_ptr(),
        CVAR_SERVERINFO | CVAR_ROM,
    );

    // systeminfo
    Cvar_Get(b"helpUsObi\0".as_ptr(), b"0\0".as_ptr(), CVAR_SYSTEMINFO);
    sv_serverid = Cvar_Get(
        b"sv_serverid\0".as_ptr(),
        b"0\0".as_ptr(),
        CVAR_SYSTEMINFO | CVAR_ROM,
    );

    // server vars
    sv_fps = Cvar_Get(b"sv_fps\0".as_ptr(), b"20\0".as_ptr(), CVAR_TEMP);
    sv_timeout = Cvar_Get(b"sv_timeout\0".as_ptr(), b"120\0".as_ptr(), CVAR_TEMP);
    sv_zombietime = Cvar_Get(b"sv_zombietime\0".as_ptr(), b"2\0".as_ptr(), CVAR_TEMP);
    Cvar_Get(b"nextmap\0".as_ptr(), b"\0".as_ptr(), CVAR_TEMP);
    sv_spawntarget = Cvar_Get(b"spawntarget\0".as_ptr(), b"\0".as_ptr(), 0);

    sv_reconnectlimit = Cvar_Get(b"sv_reconnectlimit\0".as_ptr(), b"3\0".as_ptr(), 0);
    sv_showloss = Cvar_Get(b"sv_showloss\0".as_ptr(), b"0\0".as_ptr(), 0);
    sv_killserver = Cvar_Get(b"sv_killserver\0".as_ptr(), b"0\0".as_ptr(), 0);
    sv_mapChecksum = Cvar_Get(b"sv_mapChecksum\0".as_ptr(), b"\0".as_ptr(), CVAR_ROM);
    sv_testsave = Cvar_Get(b"sv_testsave\0".as_ptr(), b"0\0".as_ptr(), 0);
    sv_compress_saved_games =
        Cvar_Get(b"sv_compress_saved_games\0".as_ptr(), b"1\0".as_ptr(), 0);

    // Only allocated once, no point in moving it around and fragmenting
    // create a heap for Ghoul2 to use for game side model vertex transforms used in collision detection
    {
        // Stub for CMiniHeap singleton initialization
        // static CMiniHeap singleton(132096);
        // G2VertSpaceServer = &singleton;
    }
}

/*
==================
SV_FinalMessage

Used by SV_Shutdown to send a final message to all
connected clients before the server goes down.  The messages are sent immediately,
not just stuck on the outgoing message list, because the server is going
to totally exit after returning from this function.
==================
*/
pub unsafe fn SV_FinalMessage(message: *mut u8) {
    SV_SendServerCommand(core::ptr::null_mut(), b"print \"%s\"\0".as_ptr(), message);
    SV_SendServerCommand(core::ptr::null_mut(), b"disconnect\0".as_ptr());

    // send it twice, ignoring rate
    for _j in 0..2 {
        for i in 0..1 {
            let cl = (*core::ptr::addr_of!(svs)).clients.add(i as usize);
            if (*cl).state >= CS_CONNECTED {
                // force a snapshot to be sent
                (*core::ptr::addr_of_mut!((*cl))).nextSnapshotTime = -1;
                SV_SendClientSnapshot(cl as *mut c_void);
            }
        }
    }
}

/*
================
SV_Shutdown

Called when each game quits,
before Sys_Quit or Sys_Error
================
*/
pub unsafe fn SV_Shutdown(finalmsg: *mut u8) {
    if com_sv_running.is_null() || (*(com_sv_running as *mut c_int)) == 0 {
        return;
    }

    //Com_Printf( "----- Server Shutdown -----\n" );

    if !(*core::ptr::addr_of!(svs)).clients.is_null() && com_errorEntered == 0 {
        SV_FinalMessage(finalmsg);
    }

    SV_RemoveOperatorCommands();
    SV_ShutdownGameProgs(0);

    if !(*core::ptr::addr_of!(svs)).snapshotEntities.is_null() {
        Z_Free((*core::ptr::addr_of!(svs)).snapshotEntities as *mut c_void);
        (*core::ptr::addr_of_mut!(svs)).snapshotEntities = core::ptr::null_mut();
    }

    for i in 0..MAX_CONFIGSTRINGS {
        if !(*core::ptr::addr_of!(sv)).configstrings[i].is_null() {
            Z_Free((*core::ptr::addr_of!(sv)).configstrings[i] as *mut c_void);
        }
    }

    // free current level
    memset(
        core::ptr::addr_of_mut!(sv) as *mut c_void,
        0,
        core::mem::size_of::<server_s>(),
    );

    // free server static data
    if !(*core::ptr::addr_of!(svs)).clients.is_null() {
        SV_FreeClient((*core::ptr::addr_of!(svs)).clients as *mut c_void);
        Z_Free((*core::ptr::addr_of!(svs)).clients as *mut c_void);
    }
    memset(
        core::ptr::addr_of_mut!(svs) as *mut c_void,
        0,
        core::mem::size_of::<serverStatic_s>(),
    );

    // Ensure we free any memory used by the leaf cache.
    CM_CleanLeafCache();

    Cvar_Set(b"sv_running\0".as_ptr(), b"0\0".as_ptr());

    //Com_Printf( "---------------------------\n" );
}
