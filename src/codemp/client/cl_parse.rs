// cl_parse.c  -- parse a message received from the server
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// Static storage - zero-initialized BSS
static mut hiddenCvarVal: [c_char; 128] = [0; 128];

// svc_strings array - preserve original order exactly
pub static SVC_STRINGS: [Option<&'static str>; 256] = {
    // Use const array initialization
    // Indices 0-9: defined strings
    // Indices 10-255: None
    const EMPTY: Option<&'static str> = None;
    let mut arr: [Option<&'static str>; 256] = [EMPTY; 256];
    arr[0] = Some("svc_bad");
    arr[1] = Some("svc_nop");
    arr[2] = Some("svc_gamestate");
    arr[3] = Some("svc_configstring");
    arr[4] = Some("svc_baseline");
    arr[5] = Some("svc_serverCommand");
    arr[6] = Some("svc_download");
    arr[7] = Some("svc_snapshot");
    arr[8] = Some("svc_setgame");
    arr[9] = Some("svc_mapchange");
    // #ifdef _XBOX
    // arr[10] = Some("svc_newpeer");
    // arr[11] = Some("svc_removepeer");
    // arr[12] = Some("svc_xbInfo");
    // #endif
    arr
};

// External types and functions - declared as stubs for structural coherence
// These would be defined in other modules of the codebase

#[repr(C)]
pub struct msg_t {
    pub readcount: c_int,
    pub cursize: c_int,
    // ... other fields omitted for stub purposes
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    // ... other fields omitted
}

#[repr(C)]
pub struct clSnapshot_t {
    pub parseEntitiesNum: c_int,
    pub numEntities: c_int,
    pub serverCommandNum: c_int,
    pub serverTime: c_int,
    pub messageNum: c_int,
    pub deltaNum: c_int,
    pub snapFlags: c_int,
    pub valid: c_int,  // qboolean
    pub areamask: [c_char; 32],
    // ... other fields omitted
}

// Local stub constants and values
const MAX_PARSE_ENTITIES: c_int = 4096;
const MAX_GENTITIES: c_int = 2048;
const GENTITYNUM_BITS: c_int = 11;
const PACKET_MASK: c_int = 0x7F;
const MAX_QPATH: c_int = 64;
const BIG_INFO_KEY: usize = 8192;
const BIG_INFO_VALUE: usize = 8192;
const MAX_GAMESTATE_CHARS: c_int = 16000;
const MAX_CONFIGSTRINGS: c_int = 2048;
const PACKET_BACKUP: c_int = 32;
const MAX_MSGLEN: c_int = 16384;
const MAX_RELIABLE_COMMANDS: c_int = 64;
const MAX_HEIGHTMAP_SIZE: c_int = 1024 * 1024;
const MAX_ONLINE_PLAYERS: c_int = 4;
const CS_SYSTEMINFO: c_int = 0;

// Local client globals - stub declarations
#[repr(C)]
pub struct client_t {
    pub parseEntities: [entityState_t; MAX_PARSE_ENTITIES as usize],
    pub parseEntitiesNum: c_int,
    pub snap: clSnapshot_t,
    pub snapshots: [clSnapshot_t; 64],
    pub gameState: GameState,
    pub serverId: c_int,
    pub newSnapshots: c_int,  // qboolean
    pub entityBaselines: [entityState_t; MAX_GENTITIES as usize],
    pub outPackets: [OutPacket; PACKET_BACKUP as usize],
}

#[repr(C)]
pub struct GameState {
    pub dataCount: c_int,
    pub stringOffsets: [c_int; MAX_CONFIGSTRINGS as usize],
    pub stringData: [c_char; MAX_GAMESTATE_CHARS as usize],
}

#[repr(C)]
pub struct OutPacket {
    pub p_serverTime: c_int,
    pub p_realtime: c_int,
}

#[repr(C)]
pub struct clientConnection_t {
    pub serverCommandSequence: c_int,
    pub reliableAcknowledge: c_int,
    pub reliableSequence: c_int,
    pub serverMessageSequence: c_int,
    pub connectPacketCount: c_int,
    pub checksumFeed: c_int,
    pub clientNum: c_int,
    pub netchan: Netchan,
    pub demoplaying: c_int,  // qboolean
    pub demowaiting: c_int,  // qboolean
    pub downloadSize: c_int,
    pub downloadBlock: c_int,
    pub downloadCount: c_int,
    pub download: *mut c_void,
    pub downloadTempName: [c_char; MAX_QPATH as usize],
    pub downloadName: [c_char; MAX_QPATH as usize],
    pub serverCommands: [[c_char; 1024]; MAX_RELIABLE_COMMANDS as usize],
    pub rmgHeightMapSize: c_int,
    pub rmgHeightMap: [c_char; MAX_HEIGHTMAP_SIZE as usize],
    pub rmgFlattenMap: [c_char; MAX_HEIGHTMAP_SIZE as usize],
    pub rmgSeed: c_int,
    pub rmgAutomapSymbolCount: c_int,
    pub rmgAutomapSymbols: [RMGSymbol; 256],
}

#[repr(C)]
pub struct RMGSymbol {
    pub mType: c_int,
    pub mSide: c_int,
    pub mOrigin: [f32; 2],
}

#[repr(C)]
pub struct Netchan {
    pub outgoingSequence: c_int,
    // ... other fields omitted
}

#[repr(C)]
pub struct playerState_t {
    pub m_iVehicleNum: c_int,
    pub commandTime: c_int,
    // ... other fields omitted
}

#[repr(C)]
pub struct z_stream {
    pub next_in: *mut c_char,
    pub avail_in: c_int,
    pub total_in: c_int,
    pub next_out: *mut c_char,
    pub avail_out: c_int,
    pub total_out: c_int,
    pub msg: *mut c_char,
    pub zalloc: *mut c_void,
    pub zfree: *mut c_void,
    pub opaque: *mut c_void,
    pub data_type: c_int,
    pub adler: c_int,
    pub reserved: c_int,
}

// External variables
extern "C" {
    static mut cl: client_t;
    static mut clc: clientConnection_t;
    static mut cls: ClientStatic;
    static cl_shownet: *mut cvar_t;
    static cgvm: *mut c_void;
    static com_sv_running: *mut cvar_t;
}

#[repr(C)]
pub struct ClientStatic {
    pub realtime: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
}

// External functions
extern "C" {
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_Memset(dest: *mut c_void, c: c_int, count: c_int);
    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: c_int);
    fn MSG_ReadBits(msg: *mut msg_t, bits: c_int) -> c_int;
    fn MSG_ReadByte(msg: *mut msg_t) -> c_int;
    fn MSG_ReadShort(msg: *mut msg_t) -> c_int;
    fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    fn MSG_ReadString(msg: *mut msg_t) -> *mut c_char;
    fn MSG_ReadBigString(msg: *mut msg_t) -> *mut c_char;
    fn MSG_ReadData(msg: *mut msg_t, data: *mut c_void, len: c_int);
    fn MSG_ReadDeltaEntity(msg: *mut msg_t, old: *const entityState_t, new: *mut entityState_t, newnum: c_int);
    fn MSG_ReadDeltaPlayerstate(msg: *mut msg_t, old: *const playerState_t, new: *mut playerState_t, ...);
    fn MSG_Bitstream(msg: *mut msg_t);
    fn MSG_CheckNETFPSFOverrides(psfOverrides: c_int);
    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char;
    fn Info_NextPair(s: *mut *const c_char, key: *mut c_char, value: *mut c_char);
    fn FS_PureServerSetLoadedPaks(s: *const c_char, t: *const c_char);
    fn FS_PureServerSetReferencedPaks(s: *const c_char, t: *const c_char);
    fn FS_ConditionalRestart(checksumFeed: c_int) -> c_int;
    fn FS_SV_FOpenFileWrite(filename: *const c_char) -> *mut c_void;
    fn FS_SV_Rename(from: *const c_char, to: *const c_char);
    fn FS_Write(buffer: *const c_void, len: c_int, f: *mut c_void) -> c_int;
    fn FS_FCloseFile(f: *mut c_void);
    fn FS_UpdateGamedir();
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cvar_SetValue(var_name: *const c_char, value: c_int);
    fn Cvar_SetCheatState();
    fn Cvar_VariableValue(var_name: *const c_char) -> c_int;
    fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
    fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: c_int);
    fn strlen(s: *const c_char) -> c_int;
    fn strstr(s: *const c_char, substr: *const c_char) -> *mut c_char;
    fn memset(s: *mut c_void, c: c_int, n: c_int) -> *mut c_void;
    fn atoi(s: *const c_char) -> c_int;
    fn Con_Close();
    fn CL_ClearState();
    fn CL_InitDownloads();
    fn CL_AddReliableCommand(cmd: *const c_char);
    fn CL_WritePacket();
    fn CL_NextDownload();
    fn VM_Call(vm: *mut c_void, call: c_int, ...) -> c_int;
    fn va(fmt: *const c_char, ...) -> *mut c_char;
    fn inflateInit(z: *mut z_stream, method: c_int) -> c_int;
    fn inflate(z: *mut z_stream) -> c_int;
    fn inflateEnd(z: *mut z_stream) -> c_int;
}

// Constants for error levels and svc commands
const ERR_DROP: c_int = 1;

// Macro-like function for SHOWNET
fn SHOWNET(msg: *mut msg_t, s: *const c_char) {
    unsafe {
        if (*cl_shownet).integer >= 2 {
            Com_Printf(b"%3i:%s\n\0".as_ptr() as *const c_char, (*msg).readcount - 1, s);
        }
    }
}

// svc command constants
const SVC_BAD: c_int = 0;
const SVC_NOP: c_int = 1;
const SVC_GAMESTATE: c_int = 2;
const SVC_CONFIGSTRING: c_int = 3;
const SVC_BASELINE: c_int = 4;
const SVC_SERVERCOMMAND: c_int = 5;
const SVC_DOWNLOAD: c_int = 6;
const SVC_SNAPSHOT: c_int = 7;
const SVC_SETGAME: c_int = 8;
const SVC_MAPCHANGE: c_int = 9;
const SVC_EOF: c_int = 10;

// For Xbox-specific handlers
#[cfg(feature = "xbox")]
const SVC_NEWPEER: c_int = 10;
#[cfg(feature = "xbox")]
const SVC_REMOVEPEER: c_int = 11;
#[cfg(feature = "xbox")]
const SVC_XBINFO: c_int = 12;

// qboolean values for compatibility
const QTRUE: c_int = 1;
const QFALSE: c_int = 0;

// CL_DeltaEntity
//
// Parses deltas from the given base and adds the resulting entity
// to the current frame
pub unsafe fn CL_DeltaEntity(msg: *mut msg_t, frame: *mut clSnapshot_t, newnum: c_int, old: *const entityState_t, unchanged: c_int) {
    // save the parsed entity state into the big circular buffer so
    // it can be used as the source for a later delta
    let state = unsafe {
        &mut cl.parseEntities[(cl.parseEntitiesNum & (MAX_PARSE_ENTITIES - 1)) as usize]
    };

    if unchanged != 0 {
        *state = *old;
    } else {
        MSG_ReadDeltaEntity(msg, old, state, newnum);
    }

    if state.number == (MAX_GENTITIES - 1) {
        return;  // entity was delta removed
    }
    cl.parseEntitiesNum += 1;
    (*frame).numEntities += 1;
}

// CL_ParsePacketEntities
pub unsafe fn CL_ParsePacketEntities(msg: *mut msg_t, oldframe: *const clSnapshot_t, newframe: *mut clSnapshot_t) {
    let mut newnum: c_int;
    let mut oldstate: *const entityState_t;
    let mut oldindex: c_int = 0;
    let mut oldnum: c_int;

    (*newframe).parseEntitiesNum = cl.parseEntitiesNum;
    (*newframe).numEntities = 0;

    // delta from the entities present in oldframe
    oldstate = core::ptr::null();
    if oldframe.is_null() {
        oldnum = 99999;
    } else {
        if oldindex >= (*oldframe).numEntities {
            oldnum = 99999;
        } else {
            oldstate = &cl.parseEntities[
                (((*oldframe).parseEntitiesNum + oldindex) & (MAX_PARSE_ENTITIES - 1)) as usize
            ];
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
                Com_Printf(b"%3i:  unchanged: %i\n\0".as_ptr() as *const c_char, (*msg).readcount, oldnum);
            }
            CL_DeltaEntity(msg, newframe, oldnum, oldstate, QTRUE);

            oldindex += 1;

            if oldindex >= (*oldframe).numEntities {
                oldnum = 99999;
            } else {
                oldstate = &cl.parseEntities[
                    (((*oldframe).parseEntitiesNum + oldindex) & (MAX_PARSE_ENTITIES - 1)) as usize
                ];
                oldnum = (*oldstate).number;
            }
        }

        if oldnum == newnum {
            // delta from previous state
            if (*cl_shownet).integer == 3 {
                Com_Printf(b"%3i:  delta: %i\n\0".as_ptr() as *const c_char, (*msg).readcount, newnum);
            }
            CL_DeltaEntity(msg, newframe, newnum, oldstate, QFALSE);

            oldindex += 1;

            if oldindex >= (*oldframe).numEntities {
                oldnum = 99999;
            } else {
                oldstate = &cl.parseEntities[
                    (((*oldframe).parseEntitiesNum + oldindex) & (MAX_PARSE_ENTITIES - 1)) as usize
                ];
                oldnum = (*oldstate).number;
            }
            continue;
        }

        if oldnum > newnum {
            // delta from baseline
            if (*cl_shownet).integer == 3 {
                Com_Printf(b"%3i:  baseline: %i\n\0".as_ptr() as *const c_char, (*msg).readcount, newnum);
            }
            CL_DeltaEntity(msg, newframe, newnum, &cl.entityBaselines[newnum as usize], QFALSE);
            continue;
        }
    }

    // any remaining entities in the old frame are copied over
    while oldnum != 99999 {
        // one or more entities from the old packet are unchanged
        if (*cl_shownet).integer == 3 {
            Com_Printf(b"%3i:  unchanged: %i\n\0".as_ptr() as *const c_char, (*msg).readcount, oldnum);
        }
        CL_DeltaEntity(msg, newframe, oldnum, oldstate, QTRUE);

        oldindex += 1;

        if oldindex >= (*oldframe).numEntities {
            oldnum = 99999;
        } else {
            oldstate = &cl.parseEntities[
                (((*oldframe).parseEntitiesNum + oldindex) & (MAX_PARSE_ENTITIES - 1)) as usize
            ];
            oldnum = (*oldstate).number;
        }
    }
}

// CL_ParseSnapshot
//
// If the snapshot is parsed properly, it will be copied to
// cl.snap and saved in cl.snapshots[].  If the snapshot is invalid
// for any reason, no changes to the state will be made at all.
pub unsafe fn CL_ParseSnapshot(msg: *mut msg_t) {
    let len: c_int;
    let old: *const clSnapshot_t;
    let mut newSnap: clSnapshot_t;
    let deltaNum: c_int;
    let oldMessageNum: c_int;
    let i: c_int;
    let mut packetNum: c_int;

    // get the reliable sequence acknowledge number
    // NOTE: now sent with all server to client messages
    // clc.reliableAcknowledge = MSG_ReadLong( msg );

    // read in the new snapshot to a temporary buffer
    // we will only copy to cl.snap if it is valid
    Com_Memset(&mut newSnap as *mut clSnapshot_t as *mut c_void, 0, core::mem::size_of::<clSnapshot_t>() as c_int);

    // we will have read any new server commands in this
    // message before we got to svc_snapshot
    newSnap.serverCommandNum = clc.serverCommandSequence;

    newSnap.serverTime = MSG_ReadLong(msg);

    newSnap.messageNum = clc.serverMessageSequence;

    deltaNum = MSG_ReadByte(msg);
    if deltaNum == 0 {
        newSnap.deltaNum = -1;
    } else {
        newSnap.deltaNum = newSnap.messageNum - deltaNum;
    }
    newSnap.snapFlags = MSG_ReadByte(msg);

    // If the frame is delta compressed from data that we
    // no longer have available, we must suck up the rest of
    // the frame, but not use it, then ask for a non-compressed
    // message
    if newSnap.deltaNum <= 0 {
        newSnap.valid = QTRUE;  // uncompressed frame
        old = core::ptr::null();
        #[cfg(not(feature = "xbox"))]
        {
            clc.demowaiting = QFALSE;  // we can start recording now
        }
    } else {
        old = &cl.snapshots[(newSnap.deltaNum & PACKET_MASK) as usize];
        if (*old).valid == 0 {
            // should never happen
            Com_Printf(b"Delta from invalid frame (not supposed to happen!).\n\0".as_ptr() as *const c_char);
        } else if (*old).messageNum != newSnap.deltaNum {
            // The frame that the server did the delta from
            // is too old, so we can't reconstruct it properly.
            Com_Printf(b"Delta frame too old.\n\0".as_ptr() as *const c_char);
        } else if cl.parseEntitiesNum - (*old).parseEntitiesNum > MAX_PARSE_ENTITIES - 128 {
            Com_DPrintf(b"Delta parseEntitiesNum too old.\n\0".as_ptr() as *const c_char);
        } else {
            newSnap.valid = QTRUE;  // valid delta parse
        }
    }

    // read areamask
    let len = MSG_ReadByte(msg);
    MSG_ReadData(msg, &mut newSnap.areamask as *mut [c_char; 32] as *mut c_void, len);

    // read playerinfo
    SHOWNET(msg, b"playerstate\0".as_ptr() as *const c_char);
    if !old.is_null() {
        MSG_ReadDeltaPlayerstate(msg, &(*old).ps, &mut newSnap.ps);
        if newSnap.ps.m_iVehicleNum != 0 {
            // this means we must have written our vehicle's ps too
            MSG_ReadDeltaPlayerstate(msg, &(*old).vps, &mut newSnap.vps, QTRUE);
        }
    } else {
        MSG_ReadDeltaPlayerstate(msg, core::ptr::null(), &mut newSnap.ps);
        if newSnap.ps.m_iVehicleNum != 0 {
            // this means we must have written our vehicle's ps too
            MSG_ReadDeltaPlayerstate(msg, core::ptr::null(), &mut newSnap.vps, QTRUE);
        }
    }

    // read packet entities
    SHOWNET(msg, b"packet entities\0".as_ptr() as *const c_char);
    CL_ParsePacketEntities(msg, old, &mut newSnap);

    // if not valid, dump the entire thing now that it has
    // been properly read
    if newSnap.valid == 0 {
        return;
    }

    // clear the valid flags of any snapshots between the last
    // received and this one, so if there was a dropped packet
    // it won't look like something valid to delta from next
    // time we wrap around in the buffer
    let mut oldMessageNum = cl.snap.messageNum + 1;

    if newSnap.messageNum - oldMessageNum >= PACKET_BACKUP {
        oldMessageNum = newSnap.messageNum - (PACKET_BACKUP - 1);
    }
    while oldMessageNum < newSnap.messageNum {
        cl.snapshots[(oldMessageNum & PACKET_MASK) as usize].valid = QFALSE;
        oldMessageNum += 1;
    }

    // copy to the current good spot
    cl.snap = newSnap;
    cl.snap.ping = 999;
    // calculate ping time
    for i in 0..PACKET_BACKUP {
        packetNum = ((clc.netchan.outgoingSequence - 1 - i) & PACKET_MASK);
        if cl.snap.ps.commandTime >= cl.outPackets[packetNum as usize].p_serverTime {
            cl.snap.ping = cls.realtime - cl.outPackets[packetNum as usize].p_realtime;
            break;
        }
    }
    // save the frame off in the backup array for later delta comparisons
    cl.snapshots[(cl.snap.messageNum & PACKET_MASK) as usize] = cl.snap;

    if (*cl_shownet).integer == 3 {
        Com_Printf(
            b"   snapshot:%i  delta:%i  ping:%i\n\0".as_ptr() as *const c_char,
            cl.snap.messageNum,
            cl.snap.deltaNum,
            cl.snap.ping
        );
    }

    cl.newSnapshots = QTRUE;
}

// CL_ParseSetGame
//
// rww - Update fs_game, this message is so we can use the ext_data
// *_overrides.txt files for mods.
pub unsafe fn CL_ParseSetGame(msg: *mut msg_t) {
    let mut newGameDir: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut i: c_int = 0;
    let mut next: c_char;

    while i < MAX_QPATH {
        next = MSG_ReadByte(msg) as c_char;

        if next as u8 != 0 {
            // if next is 0 then we have finished reading to the end of the message
            newGameDir[i as usize] = next;
        } else {
            break;
        }
        i += 1;
    }
    newGameDir[i as usize] = 0;

    Cvar_Set(b"fs_game\0".as_ptr() as *const c_char, newGameDir.as_ptr());

    // Update the search path for the mod dir
    FS_UpdateGamedir();

    // Now update the overrides manually
    #[cfg(not(feature = "xbox"))]
    {
        // No mods on Xbox
        MSG_CheckNETFPSFOverrides(QFALSE);
        MSG_CheckNETFPSFOverrides(QTRUE);
    }
}

// Global variables for server connection status
pub static mut cl_connectedToPureServer: c_int = 0;
pub static mut cl_connectedGAME: c_int = 0;
pub static mut cl_connectedCGAME: c_int = 0;
pub static mut cl_connectedUI: c_int = 0;

// CL_SystemInfoChanged
//
// The systeminfo configstring has been changed, so parse
// new information out of it.  This will happen at every
// gamestate, and possibly during gameplay.
pub unsafe fn CL_SystemInfoChanged() {
    let systemInfo: *mut c_char;
    let s: *mut c_char;
    let t: *mut c_char;
    let mut key: [c_char; BIG_INFO_KEY] = [0; BIG_INFO_KEY];
    let mut value: [c_char; BIG_INFO_VALUE] = [0; BIG_INFO_VALUE];
    let mut gameSet: c_int = QFALSE;

    systemInfo = unsafe {
        cl.gameState.stringData.as_mut_ptr()
            .add(cl.gameState.stringOffsets[CS_SYSTEMINFO as usize] as usize)
    };
    cl.serverId = atoi(Info_ValueForKey(systemInfo, b"sv_serverid\0".as_ptr() as *const c_char));

    // don't set any vars when playing a demo
    #[cfg(not(feature = "xbox"))]
    {
        // No demos on Xbox
        if clc.demoplaying != 0 {
            return;
        }
    }

    let s = Info_ValueForKey(systemInfo, b"sv_cheats\0".as_ptr() as *const c_char);
    if atoi(s) == 0 {
        Cvar_SetCheatState();
    }

    // check pure server string
    let s = Info_ValueForKey(systemInfo, b"sv_paks\0".as_ptr() as *const c_char);
    let t = Info_ValueForKey(systemInfo, b"sv_pakNames\0".as_ptr() as *const c_char);
    FS_PureServerSetLoadedPaks(s, t);

    let s = Info_ValueForKey(systemInfo, b"sv_referencedPaks\0".as_ptr() as *const c_char);
    let t = Info_ValueForKey(systemInfo, b"sv_referencedPakNames\0".as_ptr() as *const c_char);
    FS_PureServerSetReferencedPaks(s, t);

    gameSet = QFALSE;
    // scan through all the variables in the systeminfo and locally set cvars to match
    let mut s = systemInfo;
    loop {
        if s.is_null() {
            break;
        }
        Info_NextPair(&mut s, key.as_mut_ptr(), value.as_mut_ptr());
        if key[0] == 0 {
            break;
        }
        // ehw!
        if Q_stricmp(key.as_ptr(), b"fs_game\0".as_ptr() as *const c_char) == 0 {
            gameSet = QTRUE;
        }

        Cvar_Set(key.as_ptr(), value.as_ptr());
    }
    // if game folder should not be set and it is set at the client side
    if gameSet == 0 && *Cvar_VariableString(b"fs_game\0".as_ptr() as *const c_char) as u8 != 0 {
        Cvar_Set(b"fs_game\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
    }
    cl_connectedToPureServer = Cvar_VariableValue(b"sv_pure\0".as_ptr() as *const c_char);

    cl_connectedGAME = atoi(Info_ValueForKey(systemInfo, b"vm_game\0".as_ptr() as *const c_char));
    cl_connectedCGAME = atoi(Info_ValueForKey(systemInfo, b"vm_cgame\0".as_ptr() as *const c_char));
    cl_connectedUI = atoi(Info_ValueForKey(systemInfo, b"vm_ui\0".as_ptr() as *const c_char));
}

pub unsafe fn CL_ParseAutomapSymbols(msg: *mut msg_t) {
    let i: c_int;

    clc.rmgAutomapSymbolCount = MSG_ReadShort(msg);

    for i in 0..clc.rmgAutomapSymbolCount {
        clc.rmgAutomapSymbols[i as usize].mType = MSG_ReadByte(msg);
        clc.rmgAutomapSymbols[i as usize].mSide = MSG_ReadByte(msg);
        clc.rmgAutomapSymbols[i as usize].mOrigin[0] = MSG_ReadLong(msg) as f32;
        clc.rmgAutomapSymbols[i as usize].mOrigin[1] = MSG_ReadLong(msg) as f32;
    }
}

pub unsafe fn CL_ParseRMG(msg: *mut msg_t) {
    clc.rmgHeightMapSize = MSG_ReadShort(msg);
    if clc.rmgHeightMapSize == 0 {
        return;
    }

    let mut zdata: z_stream;
    let size: c_int;
    let mut heightmap1: [c_char; 15000] = [0; 15000];

    if MSG_ReadBits(msg, 1) != 0 {
        // Read the heightmap
        memset(&mut zdata as *mut z_stream as *mut c_void, 0, core::mem::size_of::<z_stream>() as c_int);
        inflateInit(&mut zdata, 4);  // Z_SYNC_FLUSH = 4

        MSG_ReadData(msg, heightmap1.as_mut_ptr() as *mut c_void, clc.rmgHeightMapSize);

        zdata.next_in = heightmap1.as_mut_ptr();
        zdata.avail_in = clc.rmgHeightMapSize;
        zdata.next_out = clc.rmgHeightMap.as_mut_ptr();
        zdata.avail_out = MAX_HEIGHTMAP_SIZE;
        inflate(&mut zdata);

        clc.rmgHeightMapSize = zdata.total_out;

        inflateEnd(&mut zdata);
    } else {
        MSG_ReadData(msg, clc.rmgHeightMap.as_mut_ptr() as *mut c_void, clc.rmgHeightMapSize);
    }

    size = MSG_ReadShort(msg);

    if MSG_ReadBits(msg, 1) != 0 {
        // Read the flatten map
        memset(&mut zdata as *mut z_stream as *mut c_void, 0, core::mem::size_of::<z_stream>() as c_int);
        inflateInit(&mut zdata, 4);  // Z_SYNC_FLUSH = 4

        MSG_ReadData(msg, heightmap1.as_mut_ptr() as *mut c_void, size);

        zdata.next_in = heightmap1.as_mut_ptr();
        zdata.avail_in = clc.rmgHeightMapSize;
        zdata.next_out = clc.rmgFlattenMap.as_mut_ptr();
        zdata.avail_out = MAX_HEIGHTMAP_SIZE;
        inflate(&mut zdata);
        inflateEnd(&mut zdata);
    } else {
        MSG_ReadData(msg, clc.rmgFlattenMap.as_mut_ptr() as *mut c_void, size);
    }

    // Read the seed
    clc.rmgSeed = MSG_ReadLong(msg);

    CL_ParseAutomapSymbols(msg);
}

// CL_ParseGamestate
pub unsafe fn CL_ParseGamestate(msg: *mut msg_t) {
    let i: c_int;
    let es: *mut entityState_t;
    let mut newnum: c_int;
    let mut nullstate: entityState_t;
    let mut cmd: c_int;
    let s: *mut c_char;

    Con_Close();

    clc.connectPacketCount = 0;

    // wipe local client state
    CL_ClearState();

    // a gamestate always marks a server command sequence
    clc.serverCommandSequence = MSG_ReadLong(msg);

    // parse all the configstrings and baselines
    cl.gameState.dataCount = 1;  // leave a 0 at the beginning for uninitialized configstrings
    loop {
        cmd = MSG_ReadByte(msg);

        if cmd == SVC_EOF {
            break;
        }

        if cmd == SVC_CONFIGSTRING {
            let len: c_int;
            let start: c_int;

            start = (*msg).readcount;

            let i = MSG_ReadShort(msg);
            if i < 0 || i >= MAX_CONFIGSTRINGS {
                Com_Error(ERR_DROP, b"configstring > MAX_CONFIGSTRINGS\0".as_ptr() as *const c_char);
            }
            let s = MSG_ReadBigString(msg);

            if (*cl_shownet).integer >= 2 {
                Com_Printf(b"%3i: %d: %s\n\0".as_ptr() as *const c_char, start, i, s);
            }

            // Commented out code block preserved from C source:
            // if (i == CS_SERVERINFO)
            // { //get the special value here
            //     char *f = strstr(s, "g_debugMelee");
            //     if (f)
            //     {
            //         while (*f && *f != '\\')
            //         { //find the \ after it
            //             f++;
            //         }
            //         if (*f == '\\')
            //         { //got it
            //             int i = 0;
            //
            //             f++;
            //             while (*f && *f != '\\' && i < 128)
            //             {
            //                 hiddenCvarVal[i] = *f;
            //                 i++;
            //                 f++;
            //             }
            //             hiddenCvarVal[i] = 0;
            //
            //             //resume here
            //             s = f;
            //         }
            //     }
            // }

            let len = strlen(s);

            if len + 1 + cl.gameState.dataCount > MAX_GAMESTATE_CHARS {
                Com_Error(ERR_DROP, b"MAX_GAMESTATE_CHARS exceeded\0".as_ptr() as *const c_char);
            }

            // append it to the gameState string buffer
            cl.gameState.stringOffsets[i as usize] = cl.gameState.dataCount;
            Com_Memcpy(
                unsafe { cl.gameState.stringData.as_mut_ptr().add(cl.gameState.dataCount as usize) as *mut c_void },
                s as *const c_void,
                len + 1,
            );
            cl.gameState.dataCount += len + 1;
        } else if cmd == SVC_BASELINE {
            newnum = MSG_ReadBits(msg, GENTITYNUM_BITS);
            if newnum < 0 || newnum >= MAX_GENTITIES {
                Com_Error(ERR_DROP, b"Baseline number out of range: %i\0".as_ptr() as *const c_char, newnum);
            }
            Com_Memset(&mut nullstate as *mut entityState_t as *mut c_void, 0, core::mem::size_of::<entityState_t>() as c_int);
            es = &mut cl.entityBaselines[newnum as usize];
            MSG_ReadDeltaEntity(msg, &nullstate, es, newnum);
        } else {
            Com_Error(ERR_DROP, b"CL_ParseGamestate: bad command byte\0".as_ptr() as *const c_char);
        }
    }

    clc.clientNum = MSG_ReadLong(msg);
    // read the checksum feed
    clc.checksumFeed = MSG_ReadLong(msg);

    CL_ParseRMG(msg);  // rwwRMG - get info for it from the server

    // parse serverId and other cvars
    CL_SystemInfoChanged();

    // reinitialize the filesystem if the game directory has changed
    if FS_ConditionalRestart(clc.checksumFeed) != 0 {
        // don't set to true because we yet have to start downloading
        // enabling this can cause double loading of a map when connecting to
        // a server which has a different game directory set
        // clc.downloadRestart = qtrue;
    }

    // This used to call CL_StartHunkUsers, but now we enter the download state before loading the
    // cgame
    CL_InitDownloads();

    // make sure the game starts
    Cvar_Set(b"cl_paused\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
}

// CL_ParseDownload
//
// A download message has been received from the server
pub unsafe fn CL_ParseDownload(msg: *mut msg_t) {
    #[cfg(feature = "xbox")]
    {
        panic!("Xbox received a download message. Unsupported!");
    }

    #[cfg(not(feature = "xbox"))]
    {
        let size: c_int;
        let mut data: [c_char; MAX_MSGLEN as usize] = [0; MAX_MSGLEN as usize];
        let mut block: c_int;

        // read the data
        block = MSG_ReadShort(msg);

        if block == 0 {
            // block zero is special, contains file size
            clc.downloadSize = MSG_ReadLong(msg);

            Cvar_SetValue(b"cl_downloadSize\0".as_ptr() as *const c_char, clc.downloadSize);

            if clc.downloadSize < 0 {
                Com_Error(ERR_DROP, MSG_ReadString(msg));
                return;
            }
        }

        size = MSG_ReadShort(msg);
        if size > 0 {
            MSG_ReadData(msg, data.as_mut_ptr() as *mut c_void, size);
        }

        if clc.downloadBlock != block {
            Com_DPrintf(b"CL_ParseDownload: Expected block %d, got %d\n\0".as_ptr() as *const c_char, clc.downloadBlock, block);
            return;
        }

        // open the file if not opened yet
        if clc.download.is_null() {
            if clc.downloadTempName[0] as u8 == 0 {
                Com_Printf(b"Server sending download, but no download was requested\n\0".as_ptr() as *const c_char);
                CL_AddReliableCommand(b"stopdl\0".as_ptr() as *const c_char);
                return;
            }

            clc.download = FS_SV_FOpenFileWrite(clc.downloadTempName.as_ptr());

            if clc.download.is_null() {
                Com_Printf(b"Could not create %s\n\0".as_ptr() as *const c_char, clc.downloadTempName.as_ptr());
                CL_AddReliableCommand(b"stopdl\0".as_ptr() as *const c_char);
                CL_NextDownload();
                return;
            }
        }

        if size != 0 {
            FS_Write(data.as_ptr() as *const c_void, size, clc.download);
        }

        let cmd = va(b"nextdl %d\0".as_ptr() as *const c_char, clc.downloadBlock);
        CL_AddReliableCommand(cmd);
        clc.downloadBlock += 1;

        clc.downloadCount += size;

        // So UI gets access to it
        Cvar_SetValue(b"cl_downloadCount\0".as_ptr() as *const c_char, clc.downloadCount);

        if size == 0 {
            // A zero length block means EOF
            if !clc.download.is_null() {
                FS_FCloseFile(clc.download);
                clc.download = core::ptr::null_mut();

                // rename the file
                FS_SV_Rename(clc.downloadTempName.as_ptr(), clc.downloadName.as_ptr());
            }
            clc.downloadTempName[0] = 0;
            clc.downloadName[0] = 0;
            Cvar_Set(b"cl_downloadName\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);

            // send intentions now
            // We need this because without it, we would hold the last nextdl and then start
            // loading right away.  If we take a while to load, the server is happily trying
            // to send us that last block over and over.
            // Write it twice to help make sure we acknowledge the download
            CL_WritePacket();
            CL_WritePacket();

            // get another file if needed
            CL_NextDownload();
        }
    }
}

pub unsafe fn CL_GetValueForHidden(s: *const c_char) -> c_int {
    // string arg here just in case I want to add more sometime and make a lookup table
    atoi(addr_of_mut!(hiddenCvarVal).cast())
}

use core::ptr::addr_of_mut;

// CL_ParseCommandString
//
// Command strings are just saved off until cgame asks for them
// when it transitions a snapshot
pub unsafe fn CL_ParseCommandString(msg: *mut msg_t) {
    let s: *mut c_char;
    let seq: c_int;
    let index: c_int;

    seq = MSG_ReadLong(msg);
    s = MSG_ReadString(msg);

    // see if we have already executed stored it off
    if clc.serverCommandSequence >= seq {
        return;
    }
    clc.serverCommandSequence = seq;

    index = seq & (MAX_RELIABLE_COMMANDS - 1);
    // Commented out code block preserved from C source:
    // if (s[0] == 'c' && s[1] == 's' && s[2] == ' ' && s[3] == '0' && s[4] == ' ')
    // { //yes.. we seem to have an incoming server info.
    //     char *f = strstr(s, "g_debugMelee");
    //     if (f)
    //     {
    //         while (*f && *f != '\\')
    //         { //find the \ after it
    //             f++;
    //         }
    //         if (*f == '\\')
    //         { //got it
    //             int i = 0;
    //
    //             f++;
    //             while (*f && *f != '\\' && i < 128)
    //             {
    //                 hiddenCvarVal[i] = *f;
    //                 i++;
    //                 f++;
    //             }
    //             hiddenCvarVal[i] = 0;
    //
    //             //don't worry about backing over beginning of string I guess,
    //             //we already know we successfully strstr'd the initial string
    //             //which exceeds this length.
    //             //MSG_ReadString appears to just return a static buffer so I
    //             //can stomp over its contents safely.
    //             f--;
    //             *f = '\"';
    //             f--;
    //             *f = ' ';
    //             f--;
    //             *f = '0';
    //             f--;
    //             *f = ' ';
    //             f--;
    //             *f = 's';
    //             f--;
    //             *f = 'c';
    //
    //             //the normal configstring gets to start here...
    //             s = f;
    //         }
    //     }
    // }
    Q_strncpyz(
        clc.serverCommands[index as usize].as_mut_ptr(),
        s,
        (core::mem::size_of::<[c_char; 1024]>()) as c_int,
    );
}

// CL_ParseServerMessage
pub unsafe fn CL_ParseServerMessage(msg: *mut msg_t) {
    let cmd: c_int;

    if (*cl_shownet).integer == 1 {
        Com_Printf(b"%i \0".as_ptr() as *const c_char, (*msg).cursize);
    } else if (*cl_shownet).integer >= 2 {
        Com_Printf(b"------------------\n\0".as_ptr() as *const c_char);
    }

    MSG_Bitstream(msg);

    // get the reliable sequence acknowledge number
    clc.reliableAcknowledge = MSG_ReadLong(msg);

    if clc.reliableAcknowledge < clc.reliableSequence - MAX_RELIABLE_COMMANDS {
        clc.reliableAcknowledge = clc.reliableSequence;
    }

    // parse the message
    loop {
        if (*msg).readcount > (*msg).cursize {
            Com_Error(ERR_DROP, b"CL_ParseServerMessage: read past end of server message\0".as_ptr() as *const c_char);
            break;
        }

        let cmd = MSG_ReadByte(msg);

        if cmd == SVC_EOF {
            SHOWNET(msg, b"END OF MESSAGE\0".as_ptr() as *const c_char);
            break;
        }

        if (*cl_shownet).integer >= 2 {
            if SVC_STRINGS[cmd as usize].is_none() {
                Com_Printf(b"%3i:BAD CMD %i\n\0".as_ptr() as *const c_char, (*msg).readcount - 1, cmd);
            } else {
                if let Some(cmd_str) = SVC_STRINGS[cmd as usize] {
                    SHOWNET(msg, cmd_str.as_ptr() as *const c_char);
                }
            }
        }

        // other commands
        match cmd {
            SVC_NOP => {},
            SVC_SERVERCOMMAND => CL_ParseCommandString(msg),
            SVC_GAMESTATE => CL_ParseGamestate(msg),
            SVC_SNAPSHOT => CL_ParseSnapshot(msg),
            SVC_SETGAME => CL_ParseSetGame(msg),
            SVC_DOWNLOAD => CL_ParseDownload(msg),
            SVC_MAPCHANGE => {
                if !cgvm.is_null() {
                    VM_Call(cgvm, 2);  // CG_MAP_CHANGE = 2
                }
            },
            #[cfg(feature = "xbox")]
            SVC_NEWPEER => {
                // jsw// new client to add to our XBonlineInfo
                // We now get the index that we should use to store this from the server
                // That ensures that our playerlist and clientinfo stay in sync!
                let index = MSG_ReadLong(msg);

                // Sanity check - server shouldn't have us overwriting an active player
                // Unless, we're the server, in which case it will be active. Doh.
                assert!((*com_sv_running).integer != 0 || false);

                // OK. Read directly into the right place
                // MSG_ReadData(msg, &xbOnlineInfo.xbPlayerList[index], sizeof(XBPlayerInfo));
                // Stub for Xbox-specific code
            },
            #[cfg(feature = "xbox")]
            SVC_REMOVEPEER => {
                // Remove a client from our xbOnlineInfo. Our ordering is the same
                // as the server, so we just get an index.
                let index = MSG_ReadLong(msg);
                // Stub for Xbox-specific code
            },
            #[cfg(feature = "xbox")]
            SVC_XBINFO => {
                // jsw//get XNADDR list from server
                // Stub for Xbox-specific code
            },
            _ => {
                Com_Error(ERR_DROP, b"CL_ParseServerMessage: Illegible server message\n\0".as_ptr() as *const c_char);
            }
        }
    }
}

extern "C" {
    fn SCR_CenterPrint(str_: *const c_char);
}
