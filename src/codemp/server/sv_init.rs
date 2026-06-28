use core::ffi::{c_int, c_char, c_void};
use crate::codemp::qcommon::q_shared::{qtrue, qfalse, qboolean};

//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "server.h"

/*
Ghoul2 Insert Start
*/
// #if !defined(TR_LOCAL_H)
// #include "../renderer/tr_local.h"
// #endif

// #if !defined (MINIHEAP_H_INC)
// #include "../qcommon/MiniHeap.h"
// #endif

// #include "../qcommon/stringed_ingame.h"

/*
===============
SV_SetConfigstring

===============
*/

// External types and functions declared for this module
extern "C" {
    // From server.h and related
    pub static mut sv: sv_t;
    pub static mut svs: svs_t;
    pub static mut sv_maxclients: *mut cvar_t;
    pub static mut sv_gametype: *mut cvar_t;
    pub static mut sv_pure: *mut cvar_t;
    pub static mut sv_serverid: *mut cvar_t;
    pub static mut com_dedicated: *mut cvar_t;
    pub static mut com_sv_running: *mut cvar_t;
    pub static mut com_frameTime: c_int;
    pub static mut cvar_modifiedFlags: c_int;

    // From renderer
    pub static mut G2VertSpaceServer: *mut CMiniHeap;
    pub static CMiniHeap_singleton: CMiniHeap;

    // Functions from qcommon
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Memset(dst: *mut c_void, val: c_int, size: usize);
    fn Com_Milliseconds() -> c_int;
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);

    // Memory management
    fn Z_Malloc(size: usize, tag: c_int, zero: qboolean) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    // String functions
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn CopyString(str: *const c_char) -> *mut c_char;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;

    // Cvar functions
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    fn Cvar_InfoString(bits: c_int) -> *const c_char;
    fn Cvar_InfoString_Big(bits: c_int) -> *const c_char;

    // File system
    fn FS_FOpenFileRead(qpath: *const c_char, f: *mut fileHandle_t, sync: qboolean) -> c_int;
    fn FS_FCloseFile(f: fileHandle_t);
    fn FS_ClearPakReferences(flags: c_int);
    fn FS_Restart(checksumFeed: c_int);
    fn FS_LoadedPakChecksums() -> *const c_char;
    fn FS_LoadedPakNames() -> *const c_char;
    fn FS_ReferencedPakChecksums() -> *const c_char;
    fn FS_ReferencedPakNames() -> *const c_char;

    // Collision map
    fn CM_ClearMap();
    fn CM_LoadMap(mapname: *const c_char, clientload: qboolean, checksum: *mut c_int);

    // Hunk allocation
    fn Hunk_Clear();
    fn Hunk_AllocateTempMemory(size: usize) -> *mut c_void;
    fn Hunk_FreeTempMemory(buf: *mut c_void);
    fn Hunk_SetMark();

    // Server functions
    fn SV_SendServerCommand(cl: *mut client_t, fmt: *const c_char, ...);
    fn SV_GentityNum(num: c_int) -> *mut sharedEntity_t;
    fn SV_ShutdownGameProgs();
    fn SV_InitGameProgs();
    fn SV_ClearWorld();
    fn SV_DropClient(cl: *mut client_t, reason: *const c_char);
    fn SV_SendClientSnapshot(client: *mut client_t);
    fn SV_SendClientMapChange(client: *mut client_t);
    fn SV_Heartbeat_f();
    fn SV_RemoveOperatorCommands();
    fn SV_AddOperatorCommands();
    fn SV_MasterShutdown();
    fn SV_BotFrame(time: c_int);
    fn SV_BotInitCvars();
    fn SV_BotInitBotLib();

    // Client functions
    fn CL_MapLoading();
    fn CL_ShutdownAll();
    fn CL_Disconnect(showMainMenu: qboolean);

    // Renderer functions
    fn R_InitSkins();
    fn R_InitShaders(startup: qboolean);
    fn R_SVModelInit();
    fn RE_RegisterMedia_LevelLoadBegin(psMapName: *const c_char, eForceReload: ForceReload_e);

    // VM functions
    fn VM_Call(vm: *mut vm_t, callnum: c_int, ...) -> i64;
    fn VM_ExplicitArgPtr(vm: *mut vm_t, arg: i64) -> *mut c_void;

    // G2 API
    fn G2API_SetTime(time: c_int, clock: c_int);

    // Random
    fn rand() -> c_int;
    fn srand(seed: c_uint);

    // va - variadic formatting function
    fn va(fmt: *const c_char, ...) -> *mut c_char;
}

// Type stubs - these would be defined elsewhere but needed for compilation
#[repr(C)]
pub struct sv_t {
    // Stub structure for server state
    pub configstrings: [*mut c_char; 0x1000],  // MAX_CONFIGSTRINGS = 0x1000
    pub svEntities: *mut svEntity_t,
    pub num_entities: c_int,
    pub state: c_int,
    pub mLocalSubBSPIndex: c_int,
    pub checksumFeed: c_int,
    pub serverId: c_int,
    pub restartedServerId: c_int,
    pub restarting: qboolean,
}

#[repr(C)]
pub struct svs_t {
    pub initialized: qboolean,
    pub clients: *mut client_t,
    pub numSnapshotEntities: c_int,
    pub snapshotEntities: *mut entityState_s,
    pub nextSnapshotEntities: c_int,
    pub snapFlagServerBit: c_int,
    pub time: c_int,
    pub clientRefNum: c_int,  // Xbox only
}

#[repr(C)]
pub struct client_t {
    pub state: c_int,
    pub userinfo: [c_char; 1024],
    pub name: [c_char; 64],
    pub gentity: *mut sharedEntity_t,
    pub netchan: netchan_t,
    pub deltaMessage: c_int,
    pub nextSnapshotTime: c_int,
}

#[repr(C)]
pub struct netchan_t {
    pub remoteAddress: netadr_t,
    // Other fields not needed here
}

#[repr(C)]
pub struct netadr_t {
    pub r#type: c_int,
    // Other fields not needed
}

#[repr(C)]
pub struct entityState_t {
    pub svFlags: c_int,
    pub linked: qboolean,
    // Other fields not used in this file
}

#[repr(C)]
pub struct entityState_s {
    pub number: c_int,
    // Other fields not used in this file
}

#[repr(C)]
pub struct sharedEntity_t {
    pub r: entityState_t,
    pub s: entityState_s,
}

#[repr(C)]
pub struct svEntity_t {
    pub baseline: entityState_s,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    pub modified: qboolean,
    // Other fields
}

pub type fileHandle_t = c_int;

#[repr(C)]
pub struct CMiniHeap {
    // Stub
}

#[repr(C)]
pub struct vm_t {
    // Stub
}

pub type ForceReload_e = c_int;

pub type c_uint = core::ffi::c_uint;

// Constants
const MAX_CONFIGSTRINGS: c_int = 0x1000;
const MAX_CLIENTS: c_int = 64;
const MAX_STRING_CHARS: c_int = 1024;
const MAX_QPATH: usize = 256;
const PACKET_BACKUP: c_int = 32;

const TAG_CLIENTS: c_int = 0;  // Placeholder

const ERR_DROP: c_int = 0;
const ERR_FATAL: c_int = 1;

const SS_LOADING: c_int = 0;
const SS_GAME: c_int = 1;

const CS_CONNECTED: c_int = 0;
const CS_PRIMED: c_int = 1;
const CS_ACTIVE: c_int = 2;

const CS_SERVERINFO: c_int = 1;
const CS_SYSTEMINFO: c_int = 0;

const CVAR_SERVERINFO: c_int = 0x1;
const CVAR_SYSTEMINFO: c_int = 0x2;
const CVAR_LATCH: c_int = 0x4;
const CVAR_ROM: c_int = 0x8;
const CVAR_ARCHIVE: c_int = 0x10;
const CVAR_INIT: c_int = 0x20;
const CVAR_TEMP: c_int = 0x40;

const SNAPFLAG_SERVERCOUNT: c_int = 0x1;

const NA_LOOPBACK: c_int = 0;
const NA_BOT: c_int = 1;

const PROTOCOL_VERSION: c_int = 26;

const MASTER_SERVER_NAME: &str = "master.jkfiles.com";

pub fn SV_SetConfigstring(index: c_int, val: *const c_char) {
    let mut len: c_int = 0;
    let mut i: c_int = 0;
    let maxChunkSize = MAX_STRING_CHARS - 24;
    let mut client: *mut client_t = core::ptr::null_mut();

    if index < 0 || index >= MAX_CONFIGSTRINGS {
        Com_Error(ERR_DROP, c"SV_SetConfigstring: bad index %i\n".as_ptr(), index);
    }

    let mut val = val;
    if val.is_null() {
        val = c"".as_ptr();
    }

    // don't bother broadcasting an update if no change
    unsafe {
        if strcmp(val, *sv.configstrings.get_unchecked(index as usize)) == 0 {
            return;
        }

        // change the string in sv
        Z_Free(*sv.configstrings.get_unchecked(index as usize) as *mut c_void);
        *sv.configstrings.get_unchecked_mut(index as usize) = CopyString(val);

        // send it to all the clients if we aren't
        // spawning a new server
        if sv.state == SS_GAME || sv.restarting != 0 {

            // send the data to all relevent clients
            i = 0;
            client = svs.clients;
            while i < (*sv_maxclients).integer {
                if (*client).state < CS_PRIMED {
                    i += 1;
                    client = client.add(1);
                    continue;
                }
                // do not always send server info to all clients
                if index == CS_SERVERINFO && !(*client).gentity.is_null() && (((*(*client).gentity).r.svFlags & 0x1) != 0) {
                    i += 1;
                    client = client.add(1);
                    continue;
                }

                len = strlen(val) as c_int;
                if len >= maxChunkSize {
                    let mut sent: c_int = 0;
                    let mut remaining: c_int = len;
                    let mut cmd: *const c_char = c"bcs0".as_ptr();
                    let mut buf: [c_char; 1024] = [0; 1024];

                    while remaining > 0 {
                        if sent == 0 {
                            cmd = c"bcs0".as_ptr();
                        }
                        else if remaining < maxChunkSize {
                            cmd = c"bcs2".as_ptr();
                        }
                        else {
                            cmd = c"bcs1".as_ptr();
                        }
                        Q_strncpyz(buf.as_mut_ptr(), val.add(sent as usize), maxChunkSize as usize);

                        SV_SendServerCommand(client, c"%s %i \"%s\"\n".as_ptr(), cmd, index, buf.as_ptr());

                        sent += maxChunkSize - 1;
                        remaining -= maxChunkSize - 1;
                    }
                } else {
                    // standard cs, just send it
                    SV_SendServerCommand(client, c"cs %i \"%s\"\n".as_ptr(), index, val);
                }
                i += 1;
                client = client.add(1);
            }
        }
    }
}



/*
===============
SV_GetConfigstring

===============
*/
pub fn SV_GetConfigstring(index: c_int, buffer: *mut c_char, bufferSize: c_int) {
    if bufferSize < 1 {
        Com_Error(ERR_DROP, c"SV_GetConfigstring: bufferSize == %i".as_ptr(), bufferSize);
    }
    if index < 0 || index >= MAX_CONFIGSTRINGS {
        Com_Error(ERR_DROP, c"SV_GetConfigstring: bad index %i\n".as_ptr(), index);
    }
    unsafe {
        if (*sv.configstrings.get_unchecked(index as usize)).is_null() {
            *buffer = 0;
            return;
        }

        Q_strncpyz(buffer, *sv.configstrings.get_unchecked(index as usize), bufferSize as usize);
    }
}


/*
================
SV_AddConfigstring

================
*/
pub fn SV_AddConfigstring(name: *const c_char, start: c_int, max: c_int) -> c_int
{
    let mut i: c_int = 0;

    unsafe {
        if name.is_null() || *name == 0 {
            return 0;
        }

        let mut name = name;
        if *name as u8 as char == '/' || *name as u8 as char == '\\' {
            #[cfg(debug_assertions)]
            Com_DPrintf(c"WARNING: Leading slash on '%s'\n".as_ptr(), name);
            name = name.add(1);

            if *name == 0 {
                return 0;
            }
        }

        i = 1;
        while i < max {
            if *(*sv.configstrings.get_unchecked((start + i) as usize)) == 0 {
                // Didn't find it
                SV_SetConfigstring((start + i), name);
                break;
            }
            else if Q_stricmp(*sv.configstrings.get_unchecked((start + i) as usize), name) == 0 {
                return i;
            }
            i += 1;
        }
    }

    return 0;

}

/*
===============
SV_SetUserinfo

===============
*/
pub fn SV_SetUserinfo(index: c_int, val: *const c_char) {
    unsafe {
        if index < 0 || index >= (*sv_maxclients).integer {
            Com_Error(ERR_DROP, c"SV_SetUserinfo: bad index %i\n".as_ptr(), index);
        }

        let mut val = val;
        if val.is_null() {
            val = c"".as_ptr();
        }

        Q_strncpyz(
            (*svs.clients.add(index as usize)).userinfo.as_mut_ptr(),
            val,
            core::mem::size_of_val(&(*svs.clients.add(index as usize)).userinfo)
        );
        Q_strncpyz(
            (*svs.clients.add(index as usize)).name.as_mut_ptr(),
            Info_ValueForKey(val, c"name".as_ptr()),
            core::mem::size_of_val(&(*svs.clients.add(index as usize)).name)
        );
    }
}



/*
===============
SV_GetUserinfo

===============
*/
pub fn SV_GetUserinfo(index: c_int, buffer: *mut c_char, bufferSize: c_int) {
    unsafe {
        if bufferSize < 1 {
            Com_Error(ERR_DROP, c"SV_GetUserinfo: bufferSize == %i".as_ptr(), bufferSize);
        }
        if index < 0 || index >= (*sv_maxclients).integer {
            Com_Error(ERR_DROP, c"SV_GetUserinfo: bad index %i\n".as_ptr(), index);
        }
        Q_strncpyz(buffer, (*svs.clients.add(index as usize)).userinfo.as_ptr(), bufferSize as usize);
    }
}


/*
================
SV_CreateBaseline

Entity baselines are used to compress non-delta messages
to the clients -- only the fields that differ from the
baseline will be transmitted
================
*/
pub fn SV_CreateBaseline() {
    let mut svent: *mut sharedEntity_t = core::ptr::null_mut();
    let mut entnum: c_int = 0;

    unsafe {
        entnum = 1;
        while entnum < sv.num_entities {
            svent = SV_GentityNum(entnum);
            if (*svent).r.linked == 0 {
                entnum += 1;
                continue;
            }
            (*svent).s.number = entnum;

            //
            // take current state as baseline
            //
            if !sv.svEntities.is_null() {
                (*sv.svEntities.add(entnum as usize)).baseline = (*svent).s;
            }
            entnum += 1;
        }
    }
}


/*
===============
SV_BoundMaxClients

===============
*/
pub fn SV_BoundMaxClients(minimum: c_int) {
    unsafe {
        // get the current maxclients value
        Cvar_Get(c"sv_maxclients".as_ptr(), c"8".as_ptr(), 0);

        (*sv_maxclients).modified = qfalse;

        if (*sv_maxclients).integer < minimum {
            Cvar_Set(c"sv_maxclients".as_ptr(), va(c"%i".as_ptr(), minimum));
        } else if (*sv_maxclients).integer > MAX_CLIENTS {
            Cvar_Set(c"sv_maxclients".as_ptr(), va(c"%i".as_ptr(), MAX_CLIENTS));
        }
    }
}


/*
===============
SV_Startup

Called when a host starts a map when it wasn't running
one before.  Successive map or map_restart commands will
NOT cause this to be called, unless the game is exited to
the menu system first.
===============
*/
pub fn SV_Startup() {
    unsafe {
        if svs.initialized != 0 {
            Com_Error(ERR_FATAL, c"SV_Startup: svs.initialized".as_ptr());
        }
        SV_BoundMaxClients(1);

        svs.clients = Z_Malloc((core::mem::size_of::<client_t>() as c_int * (*sv_maxclients).integer) as usize, TAG_CLIENTS, qtrue) as *mut client_t;
        if (*com_dedicated).integer != 0 {
            svs.numSnapshotEntities = (*sv_maxclients).integer * PACKET_BACKUP * 64;
            Cvar_Set(c"r_ghoul2animsmooth".as_ptr(), c"0".as_ptr());
            Cvar_Set(c"r_ghoul2unsqashaftersmooth".as_ptr(), c"0".as_ptr());

        } else {
            // we don't need nearly as many when playing locally
            svs.numSnapshotEntities = (*sv_maxclients).integer * 4 * 64;
        }
        svs.initialized = qtrue;

        Cvar_Set(c"sv_running".as_ptr(), c"1".as_ptr());
    }
}

/*
Ghoul2 Insert Start
*/

pub fn SV_InitSV()
{
    unsafe {
        // clear out most of the sv struct
        Com_Memset(&mut sv as *mut _ as *mut c_void, 0, core::mem::size_of_val(&sv));
        sv.mLocalSubBSPIndex = -1;
    }
}
/*
Ghoul2 Insert End
*/

/*
==================
SV_ChangeMaxClients
==================
*/
pub fn SV_ChangeMaxClients() {
    let mut oldMaxClients: c_int = 0;
    let mut i: c_int = 0;
    let mut oldClients: *mut client_t = core::ptr::null_mut();
    let mut count: c_int = 0;

    unsafe {
        // get the highest client number in use
        count = 0;
        i = 0;
        while i < (*sv_maxclients).integer {
            if (*svs.clients.add(i as usize)).state >= CS_CONNECTED {
                if i > count {
                    count = i;
                }
            }
            i += 1;
        }
        count += 1;

        oldMaxClients = (*sv_maxclients).integer;
        // never go below the highest client number in use
        SV_BoundMaxClients(count);
        // if still the same
        if (*sv_maxclients).integer == oldMaxClients {
            return;
        }

        oldClients = Hunk_AllocateTempMemory((count as usize * core::mem::size_of::<client_t>())) as *mut client_t;
        // copy the clients to hunk memory
        i = 0;
        while i < count {
            if (*svs.clients.add(i as usize)).state >= CS_CONNECTED {
                *oldClients.add(i as usize) = *svs.clients.add(i as usize);
            }
            else {
                Com_Memset(oldClients.add(i as usize) as *mut c_void, 0, core::mem::size_of::<client_t>());
            }
            i += 1;
        }

        // free old clients arrays
        Z_Free(svs.clients as *mut c_void);

        // allocate new clients
        svs.clients = Z_Malloc(((*sv_maxclients).integer as usize * core::mem::size_of::<client_t>()), TAG_CLIENTS, qtrue) as *mut client_t;
        Com_Memset(svs.clients as *mut c_void, 0, ((*sv_maxclients).integer as usize * core::mem::size_of::<client_t>()));

        // copy the clients over
        i = 0;
        while i < count {
            if (*oldClients.add(i as usize)).state >= CS_CONNECTED {
                *svs.clients.add(i as usize) = *oldClients.add(i as usize);
            }
            i += 1;
        }

        // free the old clients on the hunk
        Hunk_FreeTempMemory(oldClients as *mut c_void);

        // allocate new snapshot entities
        if (*com_dedicated).integer != 0 {
            svs.numSnapshotEntities = (*sv_maxclients).integer * PACKET_BACKUP * 64;
        } else {
            // we don't need nearly as many when playing locally
            svs.numSnapshotEntities = (*sv_maxclients).integer * 4 * 64;
        }
    }
}

/*
================
SV_ClearServer
================
*/
pub fn SV_ClearServer() {
    let mut i: c_int = 0;

    unsafe {
        i = 0;
        while i < MAX_CONFIGSTRINGS {
            if !(*sv.configstrings.get_unchecked(i as usize)).is_null() {
                Z_Free(*sv.configstrings.get_unchecked(i as usize) as *mut c_void);
            }
            i += 1;
        }

        //	CM_ClearMap();

        /*
        Ghoul2 Insert Start
        */

        // nope, can't do this anymore.. sv contains entitystates with STL in them.
        //	memset (&sv, 0, sizeof(sv));
         	SV_InitSV();
        /*
        Ghoul2 Insert End
        */
        //	Com_Memset (&sv, 0, sizeof(sv));
    }
}

/*
================
SV_TouchCGame

  touch the cgame.vm so that a pure client can load it if it's in a seperate pk3
================
*/
pub fn SV_TouchCGame() {
    let mut f: fileHandle_t = 0;
    let mut filename: [c_char; 256] = [0; 256];

    unsafe {
        if Cvar_VariableValue(c"vm_cgame".as_ptr()) != 0.0 {
            Com_sprintf(filename.as_mut_ptr(), 256, c"vm/%s.qvm".as_ptr(), c"cgame".as_ptr());
        }
        else {
            Com_sprintf(filename.as_mut_ptr(), 256, c"cgamex86.dll".as_ptr());
        }
        FS_FOpenFileRead(filename.as_ptr(), &mut f, qfalse);
        if f != 0 {
            FS_FCloseFile(f);
        }
    }
}

pub fn SV_SendMapChange()
{
    let mut i: c_int = 0;

    unsafe {
        if !svs.clients.is_null() {
            i = 0;
            while i < (*sv_maxclients).integer {
                if (*svs.clients.add(i as usize)).state >= CS_CONNECTED {
                    if (*svs.clients.add(i as usize)).netchan.remoteAddress.r#type != NA_BOT {
                        SV_SendClientMapChange(&mut *svs.clients.add(i as usize));
                    }
                }
                i += 1;
            }
        }
    }
}

extern "C" {
    pub fn R_SVModelInit();
}


#[cfg(target_os = "windows")]
#[cfg_attr(target_os = "windows", allow(dead_code))]
//To avoid fragmentation, we want everything free by this point.
//Much of this probably violates DLL boundaries, so it's done on
//Xbox only.
extern "C" {
    fn NAV_Free();
    fn CL_ClearLastLevel();
}

#[cfg(target_os = "windows")]
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub fn SV_ClearLastLevel()
{
    unsafe {
        CL_ClearLastLevel();
        NAV_Free();

        let mut i: c_int = 0;
        while i < MAX_CONFIGSTRINGS {
            if !(*sv.configstrings.get_unchecked(i as usize)).is_null() {
                Z_Free(*sv.configstrings.get_unchecked(i as usize) as *mut c_void);
                *sv.configstrings.get_unchecked_mut(i as usize) = core::ptr::null_mut();
            }
            i += 1;
        }
    }
}


/*
================
SV_SpawnServer

Change the server to a new map, taking all connected
clients along with it.
This is NOT called for map_restart
================
*/
pub fn SV_SpawnServer(server: *mut c_char, killBots: qboolean, eForceReload: ForceReload_e) {
    let mut i: c_int = 0;
    let mut checksum: c_int = 0;
    let mut isBot: qboolean = qfalse;
    let mut systemInfo: [c_char; 16384] = [0; 16384];
    let mut p: *const c_char = core::ptr::null();

    unsafe {
        SV_SendMapChange();

        RE_RegisterMedia_LevelLoadBegin(server, eForceReload);

        // shut down the existing game if it is running
        SV_ShutdownGameProgs();

        Com_Printf(c"------ Server Initialization ------\n".as_ptr());
        Com_Printf(c"Server: %s\n".as_ptr(), server);

        /*
        Ghoul2 Insert Start
        */
         	// de allocate the snapshot entities
        if !svs.snapshotEntities.is_null() {
            // C++ delete[] would be here, but in Rust we need to handle this differently
            // For now, just mark as null since we'll reallocate
            svs.snapshotEntities = core::ptr::null_mut();
        }
        /*
        Ghoul2 Insert End
        */

        SV_SendMapChange();

        #[cfg(target_os = "windows")]
        {
            // disable vsync during load for speed
            // qglDisable(GL_VSYNC);
        }

        // if not running a dedicated server CL_MapLoading will connect the client to the server
        // also print some status stuff
        CL_MapLoading();

        #[cfg(not(target_os = "windows"))]
        {
            // make sure all the client stuff is unloaded
            CL_ShutdownAll();
        }

        CM_ClearMap();

        #[cfg(target_os = "windows")]
        {
            // extern qboolean RE_RegisterImages_LevelLoadEnd(void);
            // RE_RegisterImages_LevelLoadEnd();
            // R_DeleteTextures();
        }

        // clear the whole hunk because we're (re)loading the server
        Hunk_Clear();

        #[cfg(target_os = "windows")]
        {
            SV_ClearLastLevel();
        }

        R_InitSkins();
        R_InitShaders(qtrue);

        #[cfg(all(target_os = "windows", debug_assertions))]
        {
            // Useful for memory debugging.  Please don't delete.  Comment out if necessary.
            // extern void Z_DisplayLevelMemory(int, int, int);
            // extern void Z_Details_f(void);
            // extern void Z_TagPointers(memtag_t);
            // Z_DisplayLevelMemory(0, 0, 0);
            // Z_Details_f();
        }

        // init client structures and svs.numSnapshotEntities
        if Cvar_VariableValue(c"sv_running".as_ptr()) == 0.0 {
            SV_Startup();
        } else {
            // check for maxclients change
            if (*sv_maxclients).modified != 0 {
                SV_ChangeMaxClients();
            }
        }

        SV_SendMapChange();

        /*
        Ghoul2 Insert Start
        */
         	// clear out those shaders, images and Models as long as this
        // isnt a dedicated server.
        /*
        if ( !com_dedicated->integer )
        {
    #ifndef DEDICATED
            R_InitImages();

            R_InitShaders();

            R_ModelInit();
    #endif
        }
        else
        */
        if (*com_dedicated).integer != 0 {
            R_SVModelInit();
        }

        SV_SendMapChange();

        // clear pak references
        FS_ClearPakReferences(0);

        /*
        Ghoul2 Insert Start
        */
        // allocate the snapshot entities on the hunk
        //	svs.snapshotEntities = (struct entityState_s *)Hunk_Alloc( sizeof(entityState_t)*svs.numSnapshotEntities, h_high );
        svs.nextSnapshotEntities = 0;

        // allocate the snapshot entities
        // C++ new[] would be here, but in Rust this needs different handling
        // For now, allocate via Hunk or other means
        svs.snapshotEntities = core::ptr::null_mut();  // Placeholder - would allocate actual memory
        // we CAN afford to do this here, since we know the STL vectors in Ghoul2 are empty
        if !svs.snapshotEntities.is_null() {
            Com_Memset(svs.snapshotEntities as *mut c_void, 0, (core::mem::size_of::<entityState_s>() as c_int * svs.numSnapshotEntities) as usize);
        }

        /*
        Ghoul2 Insert End
        */

        // toggle the server bit so clients can detect that a
        // server has changed
        svs.snapFlagServerBit ^= SNAPFLAG_SERVERBIT;

        // set nextmap to the same map, but it may be overriden
        // by the game startup or another console command
        Cvar_Set(c"nextmap".as_ptr(), c"map_restart 0".as_ptr());
        //	Cvar_Set( "nextmap", va("map %s", server) );

        // wipe the entire per-level structure
        SV_ClearServer();
        i = 0;
        while i < MAX_CONFIGSTRINGS {
            *sv.configstrings.get_unchecked_mut(i as usize) = CopyString(c"".as_ptr());
            i += 1;
        }

        //rww - RAGDOLL_BEGIN
        G2API_SetTime(svs.time, 0);
        //rww - RAGDOLL_END

        // make sure we are not paused
        Cvar_Set(c"cl_paused".as_ptr(), c"0".as_ptr());

        // get a new checksum feed and restart the file system
        srand(Com_Milliseconds() as c_uint);
        sv.checksumFeed = (((rand() << 16) ^ rand()) ^ Com_Milliseconds());
        FS_Restart(sv.checksumFeed);

        #[cfg(target_os = "windows")]
        {
            // CL_StartHunkUsers();
            // CM_LoadMap( va("maps/%s.bsp", server), qfalse, &checksum );
            // RE_LoadWorldMap(va("maps/%s.bsp", server));

            // Start up voice system if it isn't running yet. (ie, if we're on syslink)
            // if( !logged_on )
            //     g_Voice.Initialize();
        }

        #[cfg(not(target_os = "windows"))]
        {
            CM_LoadMap(va(c"maps/%s.bsp".as_ptr(), server), qfalse, &mut checksum);
        }

        SV_SendMapChange();

        // set serverinfo visible name
        Cvar_Set(c"mapname".as_ptr(), server);

        Cvar_Set(c"sv_mapChecksum".as_ptr(), va(c"%i".as_ptr(), checksum));

        // serverid should be different each time
        sv.serverId = com_frameTime;
        sv.restartedServerId = sv.serverId;
        Cvar_Set(c"sv_serverid".as_ptr(), va(c"%i".as_ptr(), sv.serverId));

        // clear physics interaction links
        SV_ClearWorld();

        // media configstring setting should be done during
        // the loading stage, so connected clients don't have
        // to load during actual gameplay
        sv.state = SS_LOADING;

        // load and spawn all other entities
        SV_InitGameProgs();

        // don't allow a map_restart if game is modified
        (*sv_gametype).modified = qfalse;

        // run a few frames to allow everything to settle
        i = 0;
        while i < 3 {
            //rww - RAGDOLL_BEGIN
            G2API_SetTime(svs.time, 0);
            //rww - RAGDOLL_END
            VM_Call(gvm, GAME_RUN_FRAME, svs.time);
            SV_BotFrame(svs.time);
            svs.time += 100;
            i += 1;
        }
        //rww - RAGDOLL_BEGIN
        G2API_SetTime(svs.time, 0);
        //rww - RAGDOLL_END

        // create a baseline for more efficient communications
        SV_CreateBaseline();

        i = 0;
        while i < (*sv_maxclients).integer {
            // send the new gamestate to all connected clients
            if (*svs.clients.add(i as usize)).state >= CS_CONNECTED {
                let mut denied: *mut c_char = core::ptr::null_mut();

                if (*svs.clients.add(i as usize)).netchan.remoteAddress.r#type == NA_BOT {
                    if killBots != 0 {
                        SV_DropClient(&mut *svs.clients.add(i as usize), c"".as_ptr());
                        i += 1;
                        continue;
                    }
                    isBot = qtrue;
                }
                else {
                    isBot = qfalse;
                }

                // connect the client again
                denied = VM_ExplicitArgPtr(gvm, VM_Call(gvm, GAME_CLIENT_CONNECT, i, qfalse, isBot)) as *mut c_char;	// firstTime = qfalse
                if !denied.is_null() {
                    // this generally shouldn't happen, because the client
                    // was connected before the level change
                    SV_DropClient(&mut *svs.clients.add(i as usize), denied);
                } else {
                    if isBot == 0 {
                        // when we get the next packet from a connected client,
                        // the new gamestate will be sent
                        (*svs.clients.add(i as usize)).state = CS_CONNECTED;
                    }
                    else {
                        let mut client: *mut client_t = core::ptr::null_mut();
                        let mut ent: *mut sharedEntity_t = core::ptr::null_mut();

                        client = &mut *svs.clients.add(i as usize);
                        (*client).state = CS_ACTIVE;
                        ent = SV_GentityNum(i);
                        (*ent).s.number = i;
                        (*client).gentity = ent;

                        (*client).deltaMessage = -1;
                        (*client).nextSnapshotTime = svs.time;	// generate a snapshot immediately

                        VM_Call(gvm, GAME_CLIENT_BEGIN, i);
                    }
                }
            }
            i += 1;
        }

        // run another frame to allow things to look at all the players
        VM_Call(gvm, GAME_RUN_FRAME, svs.time);
        SV_BotFrame(svs.time);
        svs.time += 100;
        //rww - RAGDOLL_BEGIN
        G2API_SetTime(svs.time, 0);
        //rww - RAGDOLL_END

        if (*sv_pure).integer != 0 {
            // the server sends these to the clients so they will only
            // load pk3s also loaded at the server
            p = FS_LoadedPakChecksums();
            Cvar_Set(c"sv_paks".as_ptr(), p);
            if strlen(p) == 0 {
                Com_Printf(c"WARNING: sv_pure set but no PK3 files loaded\n".as_ptr());
            }
            p = FS_LoadedPakNames();
            Cvar_Set(c"sv_pakNames".as_ptr(), p);

            // if a dedicated pure server we need to touch the cgame because it could be in a
            // seperate pk3 file and the client will need to load the latest cgame.qvm
            if (*com_dedicated).integer != 0 {
                SV_TouchCGame();
            }
        }
        else {
            Cvar_Set(c"sv_paks".as_ptr(), c"".as_ptr());
            Cvar_Set(c"sv_pakNames".as_ptr(), c"".as_ptr());
        }
        // the server sends these to the clients so they can figure
        // out which pk3s should be auto-downloaded
        p = FS_ReferencedPakChecksums();
        Cvar_Set(c"sv_referencedPaks".as_ptr(), p);
        p = FS_ReferencedPakNames();
        Cvar_Set(c"sv_referencedPakNames".as_ptr(), p);

        // save systeminfo and serverinfo strings
        Q_strncpyz(systemInfo.as_mut_ptr(), Cvar_InfoString_Big(CVAR_SYSTEMINFO), core::mem::size_of_val(&systemInfo));
        cvar_modifiedFlags &= !CVAR_SYSTEMINFO;
        SV_SetConfigstring(CS_SYSTEMINFO, systemInfo.as_ptr());

        SV_SetConfigstring(CS_SERVERINFO, Cvar_InfoString(CVAR_SERVERINFO));
        cvar_modifiedFlags &= !CVAR_SERVERINFO;

        // any media configstring setting now should issue a warning
        // and any configstring changes should be reliably transmitted
        // to all clients
        sv.state = SS_GAME;

        // send a heartbeat now so the master will get up to date info
        SV_Heartbeat_f();

        Hunk_SetMark();

        /* MrE: 2000-09-13: now called in CL_DownloadsComplete
        // don't call when running dedicated
        if ( !com_dedicated->integer ) {
            // note that this is called after setting the hunk mark with Hunk_SetMark
            CL_StartHunkUsers();
        }
        */
    }
}

// Placeholder for gvm global
pub static mut gvm: *mut vm_t = core::ptr::null_mut();

// Constants for VM_Call
const GAME_RUN_FRAME: c_int = 2;
const GAME_CLIENT_CONNECT: c_int = 1;
const GAME_CLIENT_BEGIN: c_int = 3;

const SNAPFLAG_SERVERBIT: c_int = 0x1;

/*
===============
SV_Init

Only called at main exe startup, not for each game
===============
*/

pub fn SV_Init() {
    unsafe {
        SV_AddOperatorCommands();

        // serverinfo vars
        Cvar_Get(c"dmflags".as_ptr(), c"0".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"fraglimit".as_ptr(), c"20".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"timelimit".as_ptr(), c"0".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"capturelimit".as_ptr(), c"0".as_ptr(), CVAR_SERVERINFO);

        // Get these to establish them and to make sure they have a default before the menus decide to stomp them.
        Cvar_Get(c"g_maxHolocronCarry".as_ptr(), c"3".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"g_privateDuel".as_ptr(), c"1".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"g_saberLocking".as_ptr(), c"1".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"g_maxForceRank".as_ptr(), c"6".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"duel_fraglimit".as_ptr(), c"10".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"g_forceBasedTeams".as_ptr(), c"0".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"g_duelWeaponDisable".as_ptr(), c"1".as_ptr(), CVAR_SERVERINFO);

        sv_gametype = Cvar_Get(c"g_gametype".as_ptr(), c"0".as_ptr(), CVAR_SERVERINFO | CVAR_LATCH);
        sv_needpass = Cvar_Get(c"g_needpass".as_ptr(), c"0".as_ptr(), CVAR_SERVERINFO | CVAR_ROM);
        Cvar_Get(c"sv_keywords".as_ptr(), c"".as_ptr(), CVAR_SERVERINFO);
        Cvar_Get(c"protocol".as_ptr(), va(c"%i".as_ptr(), PROTOCOL_VERSION), CVAR_SERVERINFO | CVAR_ROM);
        sv_mapname = Cvar_Get(c"mapname".as_ptr(), c"nomap".as_ptr(), CVAR_SERVERINFO | CVAR_ROM);
        sv_privateClients = Cvar_Get(c"sv_privateClients".as_ptr(), c"0".as_ptr(), CVAR_SERVERINFO);
        sv_hostname = Cvar_Get(c"sv_hostname".as_ptr(), c"*Jedi*".as_ptr(), CVAR_SERVERINFO | CVAR_ARCHIVE);
        sv_maxclients = Cvar_Get(c"sv_maxclients".as_ptr(), c"8".as_ptr(), CVAR_SERVERINFO | CVAR_LATCH);
        sv_maxRate = Cvar_Get(c"sv_maxRate".as_ptr(), c"0".as_ptr(), CVAR_ARCHIVE | CVAR_SERVERINFO);
        sv_minPing = Cvar_Get(c"sv_minPing".as_ptr(), c"0".as_ptr(), CVAR_ARCHIVE | CVAR_SERVERINFO);
        sv_maxPing = Cvar_Get(c"sv_maxPing".as_ptr(), c"0".as_ptr(), CVAR_ARCHIVE | CVAR_SERVERINFO);
        sv_floodProtect = Cvar_Get(c"sv_floodProtect".as_ptr(), c"1".as_ptr(), CVAR_ARCHIVE | CVAR_SERVERINFO);
        #[cfg(feature = "USE_CD_KEY")]
        {
            sv_allowAnonymous = Cvar_Get(c"sv_allowAnonymous".as_ptr(), c"0".as_ptr(), CVAR_SERVERINFO);
        }
        // systeminfo
        Cvar_Get(c"sv_cheats".as_ptr(), c"0".as_ptr(), CVAR_SYSTEMINFO | CVAR_ROM);
        sv_serverid = Cvar_Get(c"sv_serverid".as_ptr(), c"0".as_ptr(), CVAR_SYSTEMINFO | CVAR_ROM);
        #[cfg(not(feature = "DLL_ONLY"))]
        {
            sv_pure = Cvar_Get(c"sv_pure".as_ptr(), c"1".as_ptr(), CVAR_SYSTEMINFO);
        }
        #[cfg(feature = "DLL_ONLY")]
        {
            sv_pure = Cvar_Get(c"sv_pure".as_ptr(), c"0".as_ptr(), CVAR_SYSTEMINFO | CVAR_INIT | CVAR_ROM);
        }
        Cvar_Get(c"sv_paks".as_ptr(), c"".as_ptr(), CVAR_SYSTEMINFO | CVAR_ROM);
        Cvar_Get(c"sv_pakNames".as_ptr(), c"".as_ptr(), CVAR_SYSTEMINFO | CVAR_ROM);
        Cvar_Get(c"sv_referencedPaks".as_ptr(), c"".as_ptr(), CVAR_SYSTEMINFO | CVAR_ROM);
        Cvar_Get(c"sv_referencedPakNames".as_ptr(), c"".as_ptr(), CVAR_SYSTEMINFO | CVAR_ROM);

        // server vars
        sv_rconPassword = Cvar_Get(c"rconPassword".as_ptr(), c"".as_ptr(), CVAR_TEMP);
        sv_privatePassword = Cvar_Get(c"sv_privatePassword".as_ptr(), c"".as_ptr(), CVAR_TEMP);
        sv_fps = Cvar_Get(c"sv_fps".as_ptr(), c"20".as_ptr(), CVAR_TEMP);
        sv_timeout = Cvar_Get(c"sv_timeout".as_ptr(), c"200".as_ptr(), CVAR_TEMP);
        sv_zombietime = Cvar_Get(c"sv_zombietime".as_ptr(), c"2".as_ptr(), CVAR_TEMP);
        Cvar_Get(c"nextmap".as_ptr(), c"".as_ptr(), CVAR_TEMP);

        #[cfg(not(target_os = "windows"))]
        {
            // No master or downloads on Xbox
            sv_allowDownload = Cvar_Get(c"sv_allowDownload".as_ptr(), c"0".as_ptr(), CVAR_SERVERINFO);
            sv_master[0] = Cvar_Get(c"sv_master1".as_ptr(), c"master.jkfiles.com".as_ptr(), 0);
            sv_master[1] = Cvar_Get(c"sv_master2".as_ptr(), c"".as_ptr(), CVAR_ARCHIVE);
            sv_master[2] = Cvar_Get(c"sv_master3".as_ptr(), c"".as_ptr(), CVAR_ARCHIVE);
            sv_master[3] = Cvar_Get(c"sv_master4".as_ptr(), c"".as_ptr(), CVAR_ARCHIVE);
            sv_master[4] = Cvar_Get(c"sv_master5".as_ptr(), c"".as_ptr(), CVAR_ARCHIVE);
        }
        sv_reconnectlimit = Cvar_Get(c"sv_reconnectlimit".as_ptr(), c"3".as_ptr(), 0);
        sv_showghoultraces = Cvar_Get(c"sv_showghoultraces".as_ptr(), c"0".as_ptr(), 0);
        sv_showloss = Cvar_Get(c"sv_showloss".as_ptr(), c"0".as_ptr(), 0);
        sv_padPackets = Cvar_Get(c"sv_padPackets".as_ptr(), c"0".as_ptr(), 0);
        sv_killserver = Cvar_Get(c"sv_killserver".as_ptr(), c"0".as_ptr(), 0);
        sv_mapChecksum = Cvar_Get(c"sv_mapChecksum".as_ptr(), c"".as_ptr(), CVAR_ROM);

        //	sv_debugserver = Cvar_Get ("sv_debugserver", "0", 0);

        // initialize bot cvars so they are listed and can be set before loading the botlib
        SV_BotInitCvars();

        // init the botlib here because we need the pre-compiler in the UI
        SV_BotInitBotLib();

        #[cfg(target_os = "windows")]
        {
            svs.clientRefNum = 0;
        }
        // Only allocated once, no point in moving it around and fragmenting
        // create a heap for Ghoul2 to use for game side model vertex transforms used in collision detection
        G2VertSpaceServer = &CMiniHeap_singleton;
    }
}

// Static cvars used in SV_Init
pub static mut sv_needpass: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_mapname: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_privateClients: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_hostname: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_maxRate: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_minPing: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_maxPing: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_floodProtect: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_allowAnonymous: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_rconPassword: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_privatePassword: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_fps: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_timeout: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_zombietime: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_allowDownload: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_master: [*mut cvar_t; 5] = [core::ptr::null_mut(); 5];
pub static mut sv_reconnectlimit: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_showghoultraces: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_showloss: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_padPackets: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_killserver: *mut cvar_t = core::ptr::null_mut();
pub static mut sv_mapChecksum: *mut cvar_t = core::ptr::null_mut();


/*
==================
SV_FinalMessage

Used by SV_Shutdown to send a final message to all
connected clients before the server goes down.  The messages are sent immediately,
not just stuck on the outgoing message list, because the server is going
to totally exit after returning from this function.
==================
*/
pub fn SV_FinalMessage(message: *mut c_char) {
    let mut i: c_int = 0;
    let mut j: c_int = 0;
    let mut cl: *mut client_t = core::ptr::null_mut();

    unsafe {
        // send it twice, ignoring rate
        j = 0;
        while j < 2 {
            i = 0;
            cl = svs.clients;
            while i < (*sv_maxclients).integer {
                if (*cl).state >= CS_CONNECTED {
                    // don't send a disconnect to a local client
                    if (*cl).netchan.remoteAddress.r#type != NA_LOOPBACK {
                        SV_SendServerCommand(cl, c"print \"%s\"".as_ptr(), message);
                        SV_SendServerCommand(cl, c"disconnect".as_ptr());
                    }
                    // force a snapshot to be sent
                    (*cl).nextSnapshotTime = -1;
                    SV_SendClientSnapshot(cl);
                }
                i += 1;
                cl = cl.add(1);
            }
            j += 1;
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
pub fn SV_Shutdown(finalmsg: *mut c_char)
{
    unsafe {
        if com_sv_running.is_null() || (*com_sv_running).integer == 0 {
            return;
        }

        //	Com_Printf( "----- Server Shutdown -----\n" );

        if !svs.clients.is_null() && com_errorEntered == 0 {
            SV_FinalMessage(finalmsg);
        }

        SV_RemoveOperatorCommands();
        #[cfg(not(target_os = "windows"))]
        {
            // No master on Xbox
            SV_MasterShutdown();
        }
        SV_ShutdownGameProgs();
        /*
        Ghoul2 Insert Start
        */
         	// de allocate the snapshot entities
        if !svs.snapshotEntities.is_null() {
            // C++ delete[] would be here
            svs.snapshotEntities = core::ptr::null_mut();
        }

        // free current level
        SV_ClearServer();
        CM_ClearMap();//jfm: add a clear here since it's commented out in clearServer.  This prevents crashing cmShaderTable on exit.

        // free server static data
        if !svs.clients.is_null() {
            Z_Free(svs.clients as *mut c_void);
        }
        Com_Memset(&mut svs as *mut _ as *mut c_void, 0, core::mem::size_of_val(&svs));

        Cvar_Set(c"sv_running".as_ptr(), c"0".as_ptr());
        Cvar_Set(c"ui_singlePlayerActive".as_ptr(), c"0".as_ptr());

        //	Com_Printf( "---------------------------\n" );

        #[cfg(target_os = "windows")]
        {
            // If we were advertising on Live, remove the listing. This also unregisters
            // the server's key. SysLink keys are never unregistered, so we don't do anything
            // special here for them.
            // if ( logged_on )
            //     XBL_MM_Shutdown();

            // Tear down voice now if we're on system link (Live keeps it active)
            // if( !logged_on )
            //     g_Voice.Shutdown();

            // Wipe our player list - this is important
            // memset( &xbOnlineInfo, 0, sizeof(xbOnlineInfo) );
        }

        // disconnect any local clients
        CL_Disconnect(qfalse);
    }
}

// com_errorEntered global
pub static mut com_errorEntered: c_int = 0;
