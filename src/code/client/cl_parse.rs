// cl_parse.rs  -- parse a message received from the server

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

// ============================================================================
// External function declarations
// ============================================================================

extern "C" {
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn MSG_ReadEntity(msg: *mut msg_t, state: *mut entityState_t);
    pub fn MSG_ReadBits(msg: *mut msg_t, bits: c_int) -> c_int;
    pub fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    pub fn MSG_ReadByte(msg: *mut msg_t) -> c_int;
    pub fn MSG_ReadShort(msg: *mut msg_t) -> c_int;
    pub fn MSG_ReadData(msg: *mut msg_t, data: *mut c_void, len: c_int);
    pub fn MSG_ReadString(msg: *mut msg_t) -> *mut c_char;
    pub fn MSG_ReadDeltaPlayerstate(msg: *mut msg_t, from: *mut playerState_t, to: *mut playerState_t);
    pub fn strlen(s: *const c_char) -> usize;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn atoi(nptr: *const c_char) -> c_int;
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char;
    pub fn Info_NextPair(s: *mut *const c_char, key: *mut c_char, value: *mut c_char);
    pub fn Cvar_SetCheatState();
    pub fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn Con_Close();
    pub fn UI_UpdateConnectionString(string: *const c_char);
    pub fn CL_ClearState();
    pub fn CL_StartHunkUsers();
    pub fn Z_Free(ptr: *mut c_void);
    pub fn CopyString(str: *const c_char) -> *mut c_char;

    pub static mut cl: clientActive_t;
    pub static mut cls: clientStatic_t;
    pub static mut clc: clientConnection_t;
    pub static mut cl_shownet: *mut cvar_t;
}

// ============================================================================
// Local types and stubs needed for this file
// ============================================================================

#[repr(C)]
pub struct msg_t {
    pub readcount: c_int,
    pub cursize: c_int,
    // other fields omitted for stub
}

#[repr(C)]
pub struct playerState_t {
    // stub - actual implementation in another module
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // other fields omitted for stub
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    // other fields omitted
}

#[repr(C)]
pub struct netchan_t {
    pub outgoingSequence: c_int,
    // other fields omitted
}

#[repr(C)]
pub struct gameState_t {
    pub dataCount: c_int,
    pub stringOffsets: [c_int; 4096],
    pub stringData: [c_char; 16000],
}

#[repr(C)]
pub struct clSnapshot_t {
    pub valid: c_int,
    pub snapFlags: c_int,
    pub serverTime: c_int,
    pub messageNum: c_int,
    pub deltaNum: c_int,
    pub ping: c_int,
    pub areamask: [c_char; 32],
    pub cmdNum: c_int,
    pub ps: playerState_t,
    pub numEntities: c_int,
    pub parseEntitiesNum: c_int,
    pub serverCommandNum: c_int,
}

#[repr(C)]
pub struct clientActive_t {
    pub timeoutcount: c_int,
    pub frame: clSnapshot_t,
    pub serverTime: c_int,
    pub oldServerTime: c_int,
    pub oldFrameServerTime: c_int,
    pub serverTimeDelta: c_int,
    pub extrapolatedSnapshot: c_int,
    pub newSnapshots: c_int,
    pub gameState: gameState_t,
    pub mapname: [c_char; 64],
    pub parseEntitiesNum: c_int,
    pub mouseDx: [c_int; 2],
    pub mouseDy: [c_int; 2],
    pub mouseIndex: c_int,
    pub joystickAxis: [c_int; 6],
    pub cgameUserCmdValue: c_int,
    pub cgameSensitivity: f32,
    pub cmds: [[c_char; 256]; 64],
    pub cmdNumber: c_int,
    pub packetTime: [c_int; 32],
    pub packetCmdNumber: [c_int; 32],
    pub viewangles: [f32; 3],
    pub serverId: c_int,
    pub cinematictime: c_int,
    pub frames: [clSnapshot_t; 32],
    pub parseEntities: [entityState_t; 512],
    pub gcmdSendValue: c_int,
    pub gcmdValue: c_char,
}

#[repr(C)]
pub struct clientConnection_t {
    pub lastPacketSentTime: c_int,
    pub lastPacketTime: c_int,
    pub servername: [c_char; 256],
    pub serverAddress: [c_char; 32],
    pub connectTime: c_int,
    pub connectPacketCount: c_int,
    pub challenge: c_int,
    pub reliableSequence: c_int,
    pub reliableAcknowledge: c_int,
    pub reliableCommands: [*mut c_char; 64],
    pub serverCommandSequence: c_int,
    pub serverCommands: [*mut c_char; 64],
    pub netchan: netchan_t,
}

#[repr(C)]
pub struct clientStatic_t {
    pub state: c_int,
    pub keyCatchers: c_int,
    pub servername: [c_char; 256],
    pub rendererStarted: c_int,
    pub soundStarted: c_int,
    pub soundRegistered: c_int,
    pub uiStarted: c_int,
    pub cgameStarted: c_int,
    pub framecount: c_int,
    pub frametime: c_int,
    pub frametimeFraction: f32,
    pub realtime: c_int,
    pub realtimeFraction: f32,
    pub realFrametime: c_int,
    pub updateInfoString: [c_char; 1024],
    pub glconfig: [c_char; 128],
    pub charSetShader: c_int,
    pub whiteShader: c_int,
    pub consoleShader: c_int,
}

// Local stub constants
const MAX_INFO_KEY: usize = 64;
const MAX_INFO_VALUE: usize = 64;
const CS_SYSTEMINFO: c_int = 0;
const MAX_GAMESTATE_CHARS: c_int = 16000;
const GENTITYNUM_BITS: c_int = 11;
const MAX_CONFIGSTRINGS: c_int = 4096;
const ERR_DROP: c_int = 2;
const MAX_RELIABLE_COMMANDS: c_int = 64;
const MAX_GENTITIES: c_int = 1024;
const PACKET_BACKUP: c_int = 16;
const PACKET_MASK: c_int = PACKET_BACKUP - 1;
const MAX_PARSE_ENTITIES: c_int = 512;
const CA_LOADING: c_int = 6;

// ============================================================================
// Global data
// ============================================================================

pub static mut svc_strings: [*const c_char; 256] = [ptr::null(); 256];

// ============================================================================
// Initialization for svc_strings
// This needs to be done at startup or through static initialization
// ============================================================================

pub unsafe fn init_svc_strings() {
    svc_strings[0] = b"svc_bad\0".as_ptr() as *const c_char;
    svc_strings[1] = b"svc_nop\0".as_ptr() as *const c_char;
    svc_strings[2] = b"svc_gamestate\0".as_ptr() as *const c_char;
    svc_strings[3] = b"svc_configstring\0".as_ptr() as *const c_char;
    svc_strings[4] = b"svc_baseline\0".as_ptr() as *const c_char;
    svc_strings[5] = b"svc_serverCommand\0".as_ptr() as *const c_char;
    svc_strings[6] = b"svc_download\0".as_ptr() as *const c_char;
    svc_strings[7] = b"svc_snapshot\0".as_ptr() as *const c_char;
}

/*
=========================================================================

MESSAGE PARSING

=========================================================================
*/

/*
==================
CL_DeltaEntity

Parses deltas from the given base and adds the resulting entity
to the current frame
==================
*/
pub unsafe fn CL_DeltaEntity(msg: *mut msg_t, frame: *mut clSnapshot_t) {
    let state: *mut entityState_t;

    // save the parsed entity state into the big circular buffer so
    // it can be used as the source for a later delta
    state = core::addr_of_mut!((*cl).parseEntities[(((*cl).parseEntitiesNum & (MAX_PARSE_ENTITIES - 1)) as usize)]);

    MSG_ReadEntity(msg, state);

    if (*state).number == (MAX_GENTITIES - 1) {
        return;		// entity was delta removed
    }
    (*cl).parseEntitiesNum += 1;
    (*frame).numEntities += 1;
}

/*
==================
CL_ParsePacketEntities

==================
*/
pub unsafe fn CL_ParsePacketEntities(msg: *mut msg_t, oldframe: *mut clSnapshot_t, newframe: *mut clSnapshot_t) {
    let mut newnum: c_int;
    let mut oldstate: *mut entityState_t;
    let mut oldindex: c_int;
    let mut oldnum: c_int;

    (*newframe).parseEntitiesNum = (*cl).parseEntitiesNum;
    (*newframe).numEntities = 0;

    // delta from the entities present in oldframe
    oldindex = 0;
    oldstate = ptr::null_mut();
    if oldframe.is_null() {
        oldnum = 99999;
    } else {
        if oldindex >= (*oldframe).numEntities {
            oldnum = 99999;
        } else {
            oldstate = core::addr_of_mut!((*cl).parseEntities[((((*oldframe).parseEntitiesNum + oldindex) & ((MAX_PARSE_ENTITIES as c_int) - 1)) as usize)]);
            oldnum = (*oldstate).number;
        }
    }

    loop {
        // read the entity index number
        newnum = MSG_ReadBits(msg, GENTITYNUM_BITS);

        if newnum == (MAX_GENTITIES - 1) {
            break;
        }

        if (*msg).readcount > (*msg).cursize {
            Com_Error(ERR_DROP, b"CL_ParsePacketEntities: end of message\0".as_ptr() as *const c_char);
        }

        while oldnum < newnum {
            // one or more entities from the old packet are unchanged
            if (*cl_shownet).integer == 3 {
                Com_Printf(
                    b"%3i:  unchanged: %i\n\0".as_ptr() as *const c_char,
                    (*msg).readcount,
                    oldnum,
                );
            }
            CL_DeltaEntity(msg, newframe);

            oldindex += 1;

            if oldindex >= (*oldframe).numEntities {
                oldnum = 99999;
            } else {
                oldstate = core::addr_of_mut!((*cl).parseEntities[((((*oldframe).parseEntitiesNum + oldindex) & ((MAX_PARSE_ENTITIES as c_int) - 1)) as usize)]);
                oldnum = (*oldstate).number;
            }
        }
        if oldnum == newnum {
            // delta from previous state
            if (*cl_shownet).integer == 3 {
                Com_Printf(
                    b"%3i:  delta: %i\n\0".as_ptr() as *const c_char,
                    (*msg).readcount,
                    newnum,
                );
            }
            CL_DeltaEntity(msg, newframe);

            oldindex += 1;

            if oldindex >= (*oldframe).numEntities {
                oldnum = 99999;
            } else {
                oldstate = core::addr_of_mut!((*cl).parseEntities[((((*oldframe).parseEntitiesNum + oldindex) & ((MAX_PARSE_ENTITIES as c_int) - 1)) as usize)]);
                oldnum = (*oldstate).number;
            }
            continue;
        }

        if oldnum > newnum {
            // delta from baseline
            if (*cl_shownet).integer == 3 {
                Com_Printf(
                    b"%3i:  baseline: %i\n\0".as_ptr() as *const c_char,
                    (*msg).readcount,
                    newnum,
                );
            }
            CL_DeltaEntity(msg, newframe);
            continue;
        }

    }

    // any remaining entities in the old frame are copied over
    while oldnum != 99999 {
        // one or more entities from the old packet are unchanged
        if (*cl_shownet).integer == 3 {
            Com_Printf(
                b"%3i:  unchanged: %i\n\0".as_ptr() as *const c_char,
                (*msg).readcount,
                oldnum,
            );
        }
        CL_DeltaEntity(msg, newframe);

        oldindex += 1;

        if oldindex >= (*oldframe).numEntities {
            oldnum = 99999;
        } else {
            oldstate = core::addr_of_mut!((*cl).parseEntities[((((*oldframe).parseEntitiesNum + oldindex) & ((MAX_PARSE_ENTITIES as c_int) - 1)) as usize)]);
            oldnum = (*oldstate).number;
        }
    }
}


/*
================
CL_ParseSnapshot

If the snapshot is parsed properly, it will be copied to
cl.frame and saved in cl.frames[].  If the snapshot is invalid
for any reason, no changes to the state will be made at all.
================
*/
pub unsafe fn CL_ParseSnapshot(msg: *mut msg_t) {
    let mut len: c_int;
    let mut old: *mut clSnapshot_t;
    let mut newSnap: clSnapshot_t;
    let mut deltaNum: c_int;
    let mut oldMessageNum: c_int;
    let mut i: c_int;
    let mut packetNum: c_int;

    // get the reliable sequence acknowledge number
    (*clc).reliableAcknowledge = MSG_ReadLong(msg);

    // read in the new snapshot to a temporary buffer
    // we will only copy to cl.frame if it is valid
    memset(
        core::addr_of_mut!(newSnap) as *mut c_void,
        0,
        core::mem::size_of::<clSnapshot_t>(),
    );

    newSnap.serverCommandNum = (*clc).serverCommandSequence;
    newSnap.serverTime = MSG_ReadLong(msg);
    newSnap.messageNum = MSG_ReadLong(msg);
    deltaNum = MSG_ReadByte(msg);
    if deltaNum == 0 {
        newSnap.deltaNum = -1;
    } else {
        newSnap.deltaNum = newSnap.messageNum - deltaNum;
    }
    newSnap.cmdNum = MSG_ReadLong(msg);
    newSnap.snapFlags = MSG_ReadByte(msg);

    // If the frame is delta compressed from data that we
    // no longer have available, we must suck up the rest of
    // the frame, but not use it, then ask for a non-compressed
    // message
    if newSnap.deltaNum <= 0 {
        newSnap.valid = 1;		// uncompressed frame (qtrue)
        old = ptr::null_mut();
    } else {
        old = core::addr_of_mut!((*cl).frames[((newSnap.deltaNum & PACKET_MASK) as usize)]);
        if (*old).valid == 0 {
            // should never happen
            Com_Printf(b"Delta from invalid frame (not supposed to happen!).\n\0".as_ptr() as *const c_char);
        } else if (*old).messageNum != newSnap.deltaNum {
            // The frame that the server did the delta from
            // is too old, so we can't reconstruct it properly.
            Com_Printf(b"Delta frame too old.\n\0".as_ptr() as *const c_char);
        } else if (*cl).parseEntitiesNum - (*old).parseEntitiesNum > (MAX_PARSE_ENTITIES as c_int) {
            Com_Printf(b"Delta parseEntitiesNum too old.\n\0".as_ptr() as *const c_char);
        } else {
            newSnap.valid = 1;	// valid delta parse (qtrue)
        }
    }

    // read areamask
    len = MSG_ReadByte(msg);
    MSG_ReadData(msg, core::addr_of_mut!(newSnap.areamask[0]) as *mut c_void, len);

    // read playerinfo
    SHOWNET(msg, b"playerstate\0".as_ptr() as *const c_char);
    if !old.is_null() {
        MSG_ReadDeltaPlayerstate(msg, core::addr_of_mut!((*old).ps), core::addr_of_mut!(newSnap.ps));
    } else {
        MSG_ReadDeltaPlayerstate(msg, ptr::null_mut(), core::addr_of_mut!(newSnap.ps));
    }

    // read packet entities
    SHOWNET(msg, b"packet entities\0".as_ptr() as *const c_char);
    CL_ParsePacketEntities(msg, old, core::addr_of_mut!(newSnap));

    // if not valid, dump the entire thing now that it has
    // been properly read
    if newSnap.valid == 0 {
        return;
    }

    // clear the valid flags of any snapshots between the last
    // received and this one
    oldMessageNum = (*cl).frame.messageNum + 1;

    if (*cl).frame.messageNum - oldMessageNum >= PACKET_BACKUP {
        oldMessageNum = (*cl).frame.messageNum - (PACKET_BACKUP - 1);
    }
    while oldMessageNum < newSnap.messageNum {
        (*cl).frames[((oldMessageNum & PACKET_MASK) as usize)].valid = 0;
        oldMessageNum += 1;
    }

    // copy to the current good spot
    (*cl).frame = newSnap;

    // calculate ping time
    i = 0;
    while i < PACKET_BACKUP {
        packetNum = (((*clc).netchan.outgoingSequence - 1 - i) & PACKET_MASK) as c_int;
        if (*cl).frame.cmdNum == (*cl).packetCmdNumber[packetNum as usize] {
            (*cl).frame.ping = (*cls).realtime - (*cl).packetTime[packetNum as usize];
            break;
        }
        i += 1;
    }
    // save the frame off in the backup array for later delta comparisons
    (*cl).frames[(((*cl).frame.messageNum & PACKET_MASK) as usize)] = (*cl).frame;

    if (*cl_shownet).integer == 3 {
        Com_Printf(
            b"   frame:%i  delta:%i\n\0".as_ptr() as *const c_char,
            (*cl).frame.messageNum,
            (*cl).frame.deltaNum,
        );
    }

    // actions for valid frames
    (*cl).newSnapshots = 1;		// qtrue
}


//=====================================================================

/*
==================
CL_SystemInfoChanged

The systeminfo configstring has been changed, so parse
new information out of it.  This will happen at every
gamestate, and possibly during gameplay.
==================
*/
pub unsafe fn CL_SystemInfoChanged() {
    let mut systemInfo: *mut c_char;
    let mut s: *const c_char;
    let mut key: [c_char; MAX_INFO_KEY] = [0; MAX_INFO_KEY];
    let mut value: [c_char; MAX_INFO_VALUE] = [0; MAX_INFO_VALUE];

    systemInfo = core::addr_of_mut!((*cl).gameState.stringData[(*cl).gameState.stringOffsets[CS_SYSTEMINFO as usize] as usize]);
    (*cl).serverId = atoi(Info_ValueForKey(systemInfo, b"sv_serverid\0".as_ptr() as *const c_char));

    s = Info_ValueForKey(systemInfo, b"helpUsObi\0".as_ptr() as *const c_char);
    if atoi(s) == 0 {
        Cvar_SetCheatState();
    }

    // scan through all the variables in the systeminfo and locally set cvars to match
    s = systemInfo as *const c_char;
    loop {
        if s.is_null() {
            break;
        }
        Info_NextPair(
            core::addr_of_mut!(s),
            core::addr_of_mut!(key[0]),
            core::addr_of_mut!(value[0]),
        );
        if key[0] == 0 {
            break;
        }

        Cvar_Set(core::addr_of!(key[0]) as *const c_char, core::addr_of!(value[0]) as *const c_char);
    }
    //if ( Cvar_VariableIntegerValue("ui_iscensored") == 1 )
    //{
    //	Cvar_Set( "g_dismemberment", "0");
    //}
}

extern "C" {
    pub fn UI_UpdateConnectionString(string: *const c_char);
}

/*
==================
CL_ParseGamestate
==================
*/
pub unsafe fn CL_ParseGamestate(msg: *mut msg_t) {
    let mut i: c_int;
    let mut cmd: c_int;
    let mut s: *mut c_char;

    Con_Close();

    UI_UpdateConnectionString(b"\0".as_ptr() as *const c_char);

    // wipe local client state
    CL_ClearState();

    // a gamestate always marks a server command sequence
    (*clc).serverCommandSequence = MSG_ReadLong(msg);

    // parse all the configstrings and baselines
    (*cl).gameState.dataCount = 1;	// leave a 0 at the beginning for uninitialized configstrings
    loop {
        cmd = MSG_ReadByte(msg);

        if cmd <= 0 {
            break;
        }

        if cmd == 3 {		// svc_configstring
            let mut len: c_int;

            i = MSG_ReadShort(msg);
            if i < 0 || i >= MAX_CONFIGSTRINGS as c_int {
                Com_Error(ERR_DROP, b"configstring > MAX_CONFIGSTRINGS\0".as_ptr() as *const c_char);
            }
            s = MSG_ReadString(msg);
            len = strlen(s) as c_int;

            if len + 1 + (*cl).gameState.dataCount > MAX_GAMESTATE_CHARS {
                Com_Error(ERR_DROP, b"MAX_GAMESTATE_CHARS exceeded\0".as_ptr() as *const c_char);
            }

            // append it to the gameState string buffer
            (*cl).gameState.stringOffsets[i as usize] = (*cl).gameState.dataCount;
            memcpy(
                core::addr_of_mut!((*cl).gameState.stringData[(*cl).gameState.dataCount as usize]) as *mut c_void,
                s as *const c_void,
                (len + 1) as usize,
            );
            (*cl).gameState.dataCount += len + 1;
            if (*cl_shownet).integer == 3 {
                Com_Printf(
                    b"%3i:  CS# %d %s (%d)\n\0".as_ptr() as *const c_char,
                    (*msg).readcount,
                    i,
                    s,
                    len,
                );
            }
        } else if cmd == 4 {		// svc_baseline
            assert!(false);
        } else {
            Com_Error(ERR_DROP, b"CL_ParseGamestate: bad command byte\0".as_ptr() as *const c_char);
        }
    }

    // parse serverId and other cvars
    CL_SystemInfoChanged();

    // reinitialize the filesystem if the game directory has changed
    // #if 0
    // if ( fs_game->modified ) {
    // }
    // #endif

    // let the client game init and load data
    (*cls).state = 6;		// CA_LOADING

    CL_StartHunkUsers();

    // make sure the game starts
    Cvar_Set(b"cl_paused\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
}


//=====================================================================

pub unsafe fn CL_FreeServerCommands() {
    let mut i: c_int;

    i = 0;
    while i < MAX_RELIABLE_COMMANDS {
        if !(*clc).serverCommands[i as usize].is_null() {
            Z_Free((*clc).serverCommands[i as usize] as *mut c_void);
            (*clc).serverCommands[i as usize] = ptr::null_mut();
        }
        i += 1;
    }
}


/*
=====================
CL_ParseCommandString

Command strings are just saved off until cgame asks for them
when it transitions a snapshot
=====================
*/
pub unsafe fn CL_ParseCommandString(msg: *mut msg_t) {
    let mut s: *mut c_char;
    let mut seq: c_int;
    let mut index: c_int;

    seq = MSG_ReadLong(msg);
    s = MSG_ReadString(msg);

    // see if we have already executed stored it off
    if (*clc).serverCommandSequence >= seq {
        return;
    }
    (*clc).serverCommandSequence = seq;

    index = seq & (MAX_RELIABLE_COMMANDS - 1);
    if !(*clc).serverCommands[index as usize].is_null() {
        Z_Free((*clc).serverCommands[index as usize] as *mut c_void);
    }
    (*clc).serverCommands[index as usize] = CopyString(s);
}


/*
=====================
CL_ParseServerMessage
=====================
*/
pub unsafe fn CL_ParseServerMessage(msg: *mut msg_t) {
    let mut cmd: c_int;

    if (*cl_shownet).integer == 1 {
        Com_Printf(b"%i \0".as_ptr() as *const c_char, (*msg).cursize);
    } else if (*cl_shownet).integer >= 2 {
        Com_Printf(b"------------------\n\0".as_ptr() as *const c_char);
    }

    //
    // parse the message
    //
    loop {
        if (*msg).readcount > (*msg).cursize {
            Com_Error(ERR_DROP, b"CL_ParseServerMessage: read past end of server message\0".as_ptr() as *const c_char);
            break;
        }

        cmd = MSG_ReadByte(msg);

        if cmd == -1 {
            SHOWNET(msg, b"END OF MESSAGE\0".as_ptr() as *const c_char);
            break;
        }

        if (*cl_shownet).integer >= 2 {
            if svc_strings[cmd as usize].is_null() {
                Com_Printf(
                    b"%3i:BAD CMD %i\n\0".as_ptr() as *const c_char,
                    (*msg).readcount - 1,
                    cmd,
                );
            } else {
                SHOWNET(msg, svc_strings[cmd as usize]);
            }
        }

    // other commands
        match cmd {
            2 => {		// svc_gamestate
                CL_ParseGamestate(msg);
            }
            5 => {		// svc_serverCommand
                CL_ParseCommandString(msg);
            }
            7 => {		// svc_snapshot
                CL_ParseSnapshot(msg);
            }
            1 => {		// svc_nop
                // do nothing
            }
            _ => {
                Com_Error(ERR_DROP, b"CL_ParseServerMessage: Illegible server message\n\0".as_ptr() as *const c_char);
            }
        }
    }
}

/*
=========================================================================

SHOWNET helper macro
Expands to a conditional print based on cl_shownet

=========================================================================
*/

fn SHOWNET(msg: *mut msg_t, s: *const c_char) {
    unsafe {
        if (*cl_shownet).integer >= 2 {
            Com_Printf(b"%3i:%s\n\0".as_ptr() as *const c_char, (*msg).readcount - 1, s);
        }
    }
}
