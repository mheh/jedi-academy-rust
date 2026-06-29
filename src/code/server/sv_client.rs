// sv_client.c -- server code for dealing with clients

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of_mut};
use crate::code::server::server_h::*;
use crate::code::qcommon::qcommon_h::*;
use crate::code::qcommon::net_chan::*;

extern "C" {
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Cmd_Argv(argc: c_int) -> *const c_char;
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char;
    pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char);
    pub fn NET_OutOfBandPrint(sock: netsrc_t, adr: netadr_t, fmt: *const c_char, ...);
    pub fn NET_IsLocalAddress(adr: netadr_t) -> qboolean;
    pub fn NET_CompareBaseAdr(a: netadr_t, b: netadr_t) -> qboolean;
    pub fn NET_AdrToString(a: netadr_t) -> *const c_char;
    pub fn Netchan_Setup(sock: netsrc_t, chan: *mut netchan_t, adr: netadr_t, qport_val: c_int);
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int);
    pub fn MSG_WriteByte(sb: *mut msg_t, c: c_int);
    pub fn MSG_WriteLong(buf: *mut msg_t, c: c_int);
    pub fn MSG_WriteShort(buf: *mut msg_t, c: c_int);
    pub fn MSG_WriteString(sb: *mut msg_t, s: *const c_char);
    pub fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    pub fn MSG_ReadByte(sb: *mut msg_t) -> c_int;
    pub fn MSG_ReadString(sb: *mut msg_t) -> *mut c_char;
    pub fn MSG_ReadDeltaUsercmd(msg: *mut msg_t, from: *mut c_void, to: *mut c_void);
    pub fn SV_SendMessageToClient(msg: *mut msg_t, client: *mut client_s);
    pub fn SV_GentityNum(num: c_int) -> *mut gentity_t;
    pub fn SV_SendServerCommand(cl: *mut client_s, fmt: *const c_char, ...);
    pub fn Cmd_TokenizeString(text: *const c_char);
    pub fn FS_FreeFile(buffer: *mut c_void);
}

// Local stub for strcmp since it's not explicitly declared in the codebase
// This is a faithful translation of C's strcmp behavior
extern "C" {
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
}

// Local stub for strlen
extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
}

// Local stub for atoi
extern "C" {
    pub fn atoi(s: *const c_char) -> c_int;
}

// Local stub for memset
extern "C" {
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

// Local stub for strnicmp (case-insensitive string compare)
extern "C" {
    pub fn strnicmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
}

// Local stub for Com_sprintf
extern "C" {
    pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
}

// Note: SG_WriteSavegame is already declared in server_h

// Protocol version constant
const PROTOCOL_VERSION: c_int = 40;

// Local constant for packet usercmds size (from MAX_PACKET_USERCMDS)
const LOCAL_MAX_PACKET_USERCMDS: usize = 32;

// Forward declare static functions
fn SV_Disconnect_f(cl: *mut client_s);
fn SV_UpdateUserinfo_f(cl: *mut client_s);

// ucmd_t type definition
#[repr(C)]
pub struct ucmd_t {
    pub name: *const c_char,
    pub func: Option<extern "C" fn(*mut client_s)>,
}

static ucmds: [ucmd_t; 3] = [
    ucmd_t {
        name: b"userinfo\0".as_ptr() as *const c_char,
        func: Some(SV_UpdateUserinfo_f),
    },
    ucmd_t {
        name: b"disconnect\0".as_ptr() as *const c_char,
        func: Some(SV_Disconnect_f),
    },
    ucmd_t {
        name: std::ptr::null(),
        func: None,
    },
];

const MAX_STRINGCMDS: usize = 8;

/*
==================
SV_DirectConnect

A "connect" OOB command has been received
==================
*/
pub extern "C" fn SV_DirectConnect(from: netadr_t) {
    let mut userinfo: [c_char; MAX_INFO_STRING as usize] = [0; MAX_INFO_STRING as usize];
    let mut i: c_int;
    let mut cl: *mut client_s;
    let mut newcl: *mut client_s;
    let mut temp: client_s;
    let mut ent: *mut gentity_t;
    let mut clientNum: c_int;
    let mut version: c_int;
    let mut qport: c_int;
    let mut challenge: c_int;
    let mut denied: *mut c_char;

    unsafe {
        Com_DPrintf(b"SVC_DirectConnect ()\n\0".as_ptr() as *const c_char);

        Q_strncpyz(
            userinfo.as_mut_ptr(),
            Cmd_Argv(1),
            MAX_INFO_STRING as c_int,
        );

        version = atoi(Info_ValueForKey(userinfo.as_ptr(), b"protocol\0".as_ptr() as *const c_char));
        if version != PROTOCOL_VERSION {
            NET_OutOfBandPrint(
                NS_SERVER,
                from,
                b"print\nServer uses protocol version %i.\n\0".as_ptr() as *const c_char,
                PROTOCOL_VERSION,
            );
            Com_DPrintf(
                b"    rejected connect from version %i\n\0".as_ptr() as *const c_char,
                version,
            );
            return;
        }

        qport = atoi(Info_ValueForKey(userinfo.as_ptr(), b"qport\0".as_ptr() as *const c_char));

        challenge = atoi(Info_ValueForKey(
            userinfo.as_ptr(),
            b"challenge\0".as_ptr() as *const c_char,
        ));

        // see if the challenge is valid (local clients don't need to challenge)
        if NET_IsLocalAddress(from) == 0 {
            NET_OutOfBandPrint(
                NS_SERVER,
                from,
                b"print\nNo challenge for address.\n\0".as_ptr() as *const c_char,
            );
            return;
        } else {
            // force the "ip" info key to "localhost"
            Info_SetValueForKey(
                userinfo.as_mut_ptr(),
                b"ip\0".as_ptr() as *const c_char,
                b"localhost\0".as_ptr() as *const c_char,
            );
        }

        temp = core::mem::zeroed();
        newcl = &mut temp;
        memset(newcl as *mut c_void, 0, core::mem::size_of::<client_s>());

        // if there is already a slot for this ip, reuse it
        let mut found_reconnect = false;
        i = 0;
        while i < 1 {
            cl = ((*addr_of_mut!(svs)).clients as *mut client_s).offset(i as isize);
            if (*cl).state == clientState_t::CS_FREE {
                i += 1;
                continue;
            }
            if NET_CompareBaseAdr(from, (*cl).netchan.remoteAddress) != 0
                && ((*cl).netchan.qport == qport
                    || from.port == (*cl).netchan.remoteAddress.port)
            {
                if ((*addr_of_mut!(sv)).time - (*cl).lastConnectTime)
                    < ((*(*addr_of_mut!(sv_reconnectlimit))).integer * 1000)
                {
                    Com_DPrintf(
                        b"%s:reconnect rejected : too soon\n\0".as_ptr() as *const c_char,
                        NET_AdrToString(from),
                    );
                    return;
                }
                Com_Printf(
                    b"%s:reconnect\n\0".as_ptr() as *const c_char,
                    NET_AdrToString(from),
                );
                newcl = cl;
                found_reconnect = true;
                break;
            }
            i += 1;
        }

        // if no reconnect found, find a free client slot
        if !found_reconnect {
            newcl = std::ptr::null_mut();
            i = 0;
            while i < 1 {
                cl = ((*addr_of_mut!(svs)).clients as *mut client_s).offset(i as isize);
                if (*cl).state == clientState_t::CS_FREE {
                    newcl = cl;
                    break;
                }
                i += 1;
            }

            if newcl.is_null() {
                NET_OutOfBandPrint(
                    NS_SERVER,
                    from,
                    b"print\nServer is full.\n\0".as_ptr() as *const c_char,
                );
                Com_DPrintf(b"Rejected a connection.\n\0".as_ptr() as *const c_char);
                return;
            }
        }

        // gotnewcl: (label in original C - jump target for goto)
        // build a new connection
        // accept the new client
        // this is the only place a client_t is ever initialized
        *newcl = temp;
        clientNum = (newcl as *mut c_void as usize
            - (*addr_of_mut!(svs)).clients as *mut c_void as usize)
            / core::mem::size_of::<client_s>() as usize as c_int;
        ent = SV_GentityNum(clientNum);
        (*newcl).gentity = ent;

        // save the address
        Netchan_Setup(NS_SERVER, &mut (*newcl).netchan, from, qport);

        // save the userinfo
        Q_strncpyz(
            (*newcl).userinfo.as_mut_ptr(),
            userinfo.as_ptr(),
            MAX_INFO_STRING as c_int,
        );

        // get the game a chance to reject this connection or modify the userinfo
        let ge_ref = &mut *addr_of_mut!(ge);
        denied = match (*ge_ref).ClientConnect {
            Some(func) => func(clientNum, 1 as qboolean, 0 as SavedGameJustLoaded_e),
            None => std::ptr::null_mut(),
        };
        if !denied.is_null() {
            NET_OutOfBandPrint(
                NS_SERVER,
                from,
                b"print\n%s\n\0".as_ptr() as *const c_char,
                denied,
            );
            Com_DPrintf(
                b"Game rejected a connection: %s.\n\0".as_ptr() as *const c_char,
                denied,
            );
            return;
        }

        SV_UserinfoChanged(newcl);

        // send the connect packet to the client
        NET_OutOfBandPrint(NS_SERVER, from, b"connectResponse\0".as_ptr() as *const c_char);

        (*newcl).state = clientState_t::CS_CONNECTED;
        (*newcl).nextSnapshotTime = (*addr_of_mut!(sv)).time;
        (*newcl).lastPacketTime = (*addr_of_mut!(sv)).time;
        (*newcl).lastConnectTime = (*addr_of_mut!(sv)).time;

        // when we receive the first packet from the client, we will
        // notice that it is from a different serverid and that the
        // gamestate message was not just sent, forcing a retransmit
        (*newcl).gamestateMessageNum = -1;
    }
}

/*
=====================
SV_DropClient

Called when the player is totally leaving the server, either willingly
or unwillingly.  This is NOT called if the entire server is quiting
or crashing -- SV_FinalMessage() will handle that
=====================
*/
pub extern "C" fn SV_DropClient(drop: *mut client_s, reason: *const c_char) {
    unsafe {
        if (*drop).state == clientState_t::CS_ZOMBIE {
            return; // already dropped
        }
        (*drop).state = clientState_t::CS_ZOMBIE; // become free in a few seconds

        if !(*drop).download.is_null() {
            FS_FreeFile((*drop).download as *mut c_void);
            (*drop).download = std::ptr::null_mut();
        }

        // call the prog function for removing a client
        // this will remove the body, among other things
        if let Some(func) = (*(&mut *addr_of_mut!(ge))).ClientDisconnect {
            func(
                (drop as *mut c_void as usize
                    - (*addr_of_mut!(svs)).clients as *mut c_void as usize)
                    / core::mem::size_of::<client_s>(),
            );
        }

        // tell everyone why they got dropped
        SV_SendServerCommand(
            std::ptr::null_mut(),
            b"print \"%s %s\n\"\0".as_ptr() as *const c_char,
            (*drop).name.as_ptr(),
            reason,
        );

        // add the disconnect command
        SV_SendServerCommand(drop, b"disconnect\0".as_ptr() as *const c_char);
    }
}

/*
================
SV_SendClientGameState

Sends the first message from the server to a connected client.
This will be sent on the initial connection and upon each new map load.

It will be resent if the client acknowledges a later message but has
the wrong gamestate.
================
*/
pub extern "C" fn SV_SendClientGameState(client: *mut client_s) {
    let mut start: c_int;
    let mut msg: msg_t = unsafe { core::mem::zeroed() };
    let mut msgBuffer: [u8; MAX_MSGLEN as usize] = [0; MAX_MSGLEN as usize];

    unsafe {
        Com_DPrintf(
            b"SV_SendGameState() for %s\n\0".as_ptr() as *const c_char,
            (*client).name.as_ptr(),
        );
        (*client).state = clientState_t::CS_PRIMED;

        // when we receive the first packet from the client, we will
        // notice that it is from a different serverid and that the
        // gamestate message was not just sent, forcing a retransmit
        (*client).gamestateMessageNum = (*client).netchan.outgoingSequence;

        // clear the reliable message list for this client
        (*client).reliableSequence = 0;
        (*client).reliableAcknowledge = 0;

        MSG_Init(&mut msg, msgBuffer.as_mut_ptr(), MAX_MSGLEN);

        // send the gamestate
        MSG_WriteByte(&mut msg, svc_ops_e::svc_gamestate as c_int);
        MSG_WriteLong(&mut msg, (*client).reliableSequence);

        // write the configstrings
        start = 0;
        while start < MAX_CONFIGSTRINGS as c_int {
            if *(*(*addr_of_mut!(sv)).configstrings.as_ptr().offset(start as isize)) as c_int
                != 0
            {
                MSG_WriteByte(&mut msg, svc_ops_e::svc_configstring as c_int);
                MSG_WriteShort(&mut msg, start);
                MSG_WriteString(
                    &mut msg,
                    *(*addr_of_mut!(sv))
                        .configstrings
                        .as_ptr()
                        .offset(start as isize),
                );
            }
            start += 1;
        }

        MSG_WriteByte(&mut msg, 0);

        // check for overflow
        if msg.overflowed != 0 {
            Com_Printf(
                b"WARNING: GameState overflowed for %s\n\0".as_ptr() as *const c_char,
                (*client).name.as_ptr(),
            );
        }

        // deliver this to the client
        SV_SendMessageToClient(&mut msg, client);
    }
}

/*
==================
SV_ClientEnterWorld
==================
*/
pub extern "C" fn SV_ClientEnterWorld(
    client: *mut client_s,
    cmd: *mut usercmd_t,
    eSavedGameJustLoaded: SavedGameJustLoaded_e,
) {
    let mut clientNum: c_int;
    let mut ent: *mut gentity_t;

    unsafe {
        Com_DPrintf(
            b"SV_ClientEnterWorld() from %s\n\0".as_ptr() as *const c_char,
            (*client).name.as_ptr(),
        );
        (*client).state = clientState_t::CS_ACTIVE;

        // set up the entity for the client
        clientNum = (client as *mut c_void as usize
            - (*addr_of_mut!(svs)).clients as *mut c_void as usize)
            / core::mem::size_of::<client_s>() as usize as c_int;
        ent = SV_GentityNum(clientNum);
        (*ent).s.number = clientNum;
        (*client).gentity = ent;

        // normally I check 'qbFromSavedGame' to avoid overwriting loaded client data, but this stuff I want
        //	to be reset so that client packet delta-ing bgins afresh, rather than based on your previous frame
        //	(which didn't in fact happen now if we've just loaded from a saved game...)
        //
        (*client).deltaMessage = -1;
        (*client).cmdNum = 0;
        (*client).nextSnapshotTime = (*addr_of_mut!(sv)).time; // generate a snapshot immediately

        // call the game begin function
        if let Some(func) = (*(&mut *addr_of_mut!(ge))).ClientBegin {
            func(
                (client as *mut c_void as usize
                    - (*addr_of_mut!(svs)).clients as *mut c_void as usize)
                    / core::mem::size_of::<client_s>(),
                cmd,
                eSavedGameJustLoaded,
            );
        }
    }
}

/*
============================================================

CLIENT COMMAND EXECUTION

============================================================
*/

/*
=================
SV_Disconnect_f

The client is going to disconnect, so remove the connection immediately  FIXME: move to game?
=================
*/
extern "C" fn SV_Disconnect_f(cl: *mut client_s) {
    unsafe {
        SV_DropClient(cl, b"disconnected\0".as_ptr() as *const c_char);
    }
}

/*
=================
SV_UserinfoChanged

Pull specific info from a newly changed userinfo string
into a more C friendly form.
=================
*/
pub extern "C" fn SV_UserinfoChanged(cl: *mut client_s) {
    let mut val: *mut c_char;
    let mut i: c_int;

    unsafe {
        // name for C code
        Q_strncpyz(
            (*cl).name.as_mut_ptr(),
            Info_ValueForKey((*cl).userinfo.as_ptr(), b"name\0".as_ptr() as *const c_char),
            MAX_NAME_LENGTH as c_int,
        );

        // rate command

        // if the client is on the same subnet as the server and we aren't running an
        // internet public server, assume they don't need a rate choke
        (*cl).rate = 99999; // lans should not rate limit

        // snaps command
        val = Info_ValueForKey((*cl).userinfo.as_ptr(), b"snaps\0".as_ptr() as *const c_char);
        if strlen(val) > 0 {
            i = atoi(val);
            if i < 1 {
                i = 1;
            } else if i > 30 {
                i = 30;
            }
            (*cl).snapshotMsec = 1000 / i;
        } else {
            (*cl).snapshotMsec = 50;
        }
    }
}

/*
==================
SV_UpdateUserinfo_f
==================
*/
extern "C" fn SV_UpdateUserinfo_f(cl: *mut client_s) {
    unsafe {
        Q_strncpyz(
            (*cl).userinfo.as_mut_ptr(),
            Cmd_Argv(1),
            MAX_INFO_STRING as c_int,
        );

        // call prog code to allow overrides
        if let Some(func) = (*(&mut *addr_of_mut!(ge))).ClientUserinfoChanged {
            func(
                (cl as *mut c_void as usize - (*addr_of_mut!(svs)).clients as *mut c_void as usize)
                    / core::mem::size_of::<client_s>(),
            );
        }

        SV_UserinfoChanged(cl);
    }
}

/*
==================
SV_ExecuteClientCommand
==================
*/
pub extern "C" fn SV_ExecuteClientCommand(cl: *mut client_s, s: *const c_char) {
    let mut u: *const ucmd_t;

    unsafe {
        Cmd_TokenizeString(s);

        // see if it is a server level command
        u = ucmds.as_ptr();
        while !(*u).name.is_null() {
            if strcmp(Cmd_Argv(0), (*u).name) == 0 {
                if let Some(func) = (*u).func {
                    func(cl);
                }
                return;
            }
            u = u.offset(1);
        }

        // pass unknown strings to the game
        if (*u).name.is_null() && (*addr_of_mut!(sv)).state == serverState_t::SS_GAME {
            if let Some(func) = (*(&mut *addr_of_mut!(ge))).ClientCommand {
                func(
                    (cl as *mut c_void as usize
                        - (*addr_of_mut!(svs)).clients as *mut c_void as usize)
                        / core::mem::size_of::<client_s>(),
                );
            }
        }
    }
}

/*
===============
SV_ClientCommand
===============
*/
extern "C" fn SV_ClientCommand(cl: *mut client_s, msg: *mut msg_t) {
    let mut seq: c_int;
    let mut s: *mut c_char;

    unsafe {
        seq = MSG_ReadLong(msg);
        s = MSG_ReadString(msg);

        // see if we have already executed it
        if (*cl).lastClientCommand >= seq {
            return;
        }

        Com_DPrintf(
            b"clientCommand: %s : %i : %s\n\0".as_ptr() as *const c_char,
            (*cl).name.as_ptr(),
            seq,
            s,
        );

        // drop the connection if we have somehow lost commands
        if seq > (*cl).lastClientCommand + 1 {
            Com_Printf(
                b"Client %s lost %i clientCommands\n\0".as_ptr() as *const c_char,
                (*cl).name.as_ptr(),
                seq - (*cl).lastClientCommand + 1,
            );
        }

        SV_ExecuteClientCommand(cl, s);

        (*cl).lastClientCommand = seq;
    }
}

//==================================================================================

/*
==================
SV_ClientThink
==================
*/
pub extern "C" fn SV_ClientThink(cl: *mut client_s, cmd: *mut usercmd_t) {
    unsafe {
        (*cl).lastUsercmd = *cmd;

        if (*cl).state != clientState_t::CS_ACTIVE {
            return; // may have been kicked during the last usercmd
        }

        if let Some(func) = (*(&mut *addr_of_mut!(ge))).ClientThink {
            func(
                (cl as *mut c_void as usize - (*addr_of_mut!(svs)).clients as *mut c_void as usize)
                    / core::mem::size_of::<client_s>(),
                cmd,
            );
        }
    }
}

/*
==================
SV_UserMove

The message usually contains all the movement commands
that were in the last three packets, so that the information
in dropped packets can be recovered.

On very fast clients, there may be multiple usercmd packed into
each of the backup packets.
==================
*/
extern "C" fn SV_UserMove(cl: *mut client_s, msg: *mut msg_t) {
    let mut i: c_int;
    let mut start: c_int;
    let mut cmdNum: c_int;
    let mut firstNum: c_int;
    let mut cmdCount: c_int;
    let mut nullcmd: usercmd_t = unsafe { core::mem::zeroed() };
    let mut cmds: [usercmd_t; LOCAL_MAX_PACKET_USERCMDS] = unsafe { core::mem::zeroed() };
    let mut cmd: *mut usercmd_t;
    let mut oldcmd: *mut usercmd_t;
    let mut clientTime: c_int;
    let mut serverId: c_int;

    unsafe {
        (*cl).reliableAcknowledge = MSG_ReadLong(msg);
        serverId = MSG_ReadLong(msg);
        clientTime = MSG_ReadLong(msg);
        (*cl).deltaMessage = MSG_ReadLong(msg);

        // cmdNum is the command number of the most recent included usercmd
        cmdNum = MSG_ReadLong(msg);
        cmdCount = MSG_ReadByte(msg);

        if cmdCount < 1 {
            Com_Printf(b"cmdCount < 1\n\0".as_ptr() as *const c_char);
            return;
        }

        if cmdCount > MAX_PACKET_USERCMDS {
            Com_Printf(
                b"cmdCount > MAX_PACKET_USERCMDS\n\0".as_ptr() as *const c_char
            );
            return;
        }

        memset(&mut nullcmd as *mut usercmd_t as *mut c_void, 0, core::mem::size_of::<usercmd_t>());
        oldcmd = &mut nullcmd;
        i = 0;
        while i < cmdCount {
            cmd = cmds.as_mut_ptr().offset(i as isize);
            MSG_ReadDeltaUsercmd(msg, oldcmd as *mut c_void, cmd as *mut c_void);
            oldcmd = cmd;
            i += 1;
        }

        // if this is a usercmd from a previous gamestate,
        // ignore it or retransmit the current gamestate
        if serverId != (*addr_of_mut!(sv)).serverId {
            // if we can tell that the client has dropped the last
            // gamestate we sent them, resend it
            if (*cl).netchan.incomingAcknowledged > (*cl).gamestateMessageNum {
                Com_DPrintf(
                    b"%s : dropped gamestate, resending\n\0".as_ptr() as *const c_char,
                    (*cl).name.as_ptr(),
                );
                SV_SendClientGameState(cl);
            }
            return;
        }

        // if this is the first usercmd we have received
        // this gamestate, put the client into the world
        if (*cl).state == clientState_t::CS_PRIMED {
            SV_ClientEnterWorld(cl, cmds.as_mut_ptr(), *addr_of_mut!(eSavedGameJustLoaded));
            #[cfg(not(target_os = "xbox"))]
            {
                // No auto-saving for now?
                if *(*(*addr_of_mut!(sv_mapname)).string) as i32 != b'_' as i32 {
                    let mut savename: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
                    if *addr_of_mut!(eSavedGameJustLoaded) == 0 {
                        // eNO
                        SG_WriteSavegame(b"auto\0".as_ptr() as *const c_char, 1);
                        if strnicmp(
                            (*(*addr_of_mut!(sv_mapname)).string),
                            b"academy\0".as_ptr() as *const c_char,
                            7,
                        ) != 0
                        {
                            Com_sprintf(
                                savename.as_mut_ptr(),
                                MAX_QPATH as c_int,
                                b"auto_%s\0".as_ptr() as *const c_char,
                                (*(*addr_of_mut!(sv_mapname)).string),
                            );
                            SG_WriteSavegame(savename.as_ptr(), 1); // can't use va because it's nested
                        }
                    } else if *addr_of_mut!(qbLoadTransition) != 0 {
                        // qbLoadTransition == qtrue
                        Com_sprintf(
                            savename.as_mut_ptr(),
                            MAX_QPATH as c_int,
                            b"hub/%s\0".as_ptr() as *const c_char,
                            (*(*addr_of_mut!(sv_mapname)).string),
                        );
                        SG_WriteSavegame(savename.as_ptr(), 0); // save a full one
                        SG_WriteSavegame(b"auto\0".as_ptr() as *const c_char, 0); // need a copy for auto, too
                    }
                }
            }
            *addr_of_mut!(eSavedGameJustLoaded) = 0; // eNO
            // the moves can be processed normally
        }

        if (*cl).state != clientState_t::CS_ACTIVE {
            (*cl).deltaMessage = -1;
            return;
        }

        // if there is a time gap from the last packet to this packet,
        // fill in with the first command in the packet

        // with a packetdup of 0, firstNum == cmdNum
        firstNum = cmdNum - (cmdCount - 1);
        if (*cl).cmdNum < firstNum - 1 {
            (*cl).droppedCommands = 1; // qtrue
            if (*(*addr_of_mut!(sv_showloss)).integer) != 0 {
                Com_Printf(
                    b"Lost %i usercmds from %s\n\0".as_ptr() as *const c_char,
                    firstNum - 1 - (*cl).cmdNum,
                    (*cl).name.as_ptr(),
                );
            }
            if (*cl).cmdNum < firstNum - 6 {
                (*cl).cmdNum = firstNum - 6; // don't generate too many
            }
            while (*cl).cmdNum < firstNum - 1 {
                (*cl).cmdNum += 1;
                SV_ClientThink(cl, cmds.as_mut_ptr());
            }
        }
        // skip over any usercmd_t we have already executed
        start = (*cl).cmdNum - (firstNum - 1);
        i = start;
        while i < cmdCount {
            SV_ClientThink(cl, cmds.as_mut_ptr().offset(i as isize));
            i += 1;
        }
        (*cl).cmdNum = cmdNum;
    }
}

/*
===========================================================================

USER CMD EXECUTION

===========================================================================
*/

/*
===================
SV_ExecuteClientMessage

Parse a client packet
===================
*/
pub extern "C" fn SV_ExecuteClientMessage(cl: *mut client_s, msg: *mut msg_t) {
    let mut c: c_int;

    unsafe {
        loop {
            if (*msg).readcount > (*msg).cursize {
                SV_DropClient(cl, b"had a badread\0".as_ptr() as *const c_char);
                return;
            }

            c = MSG_ReadByte(msg);
            if c == -1 {
                break;
            }

            match c {
                x if x == clc_ops_e::clc_nop as c_int => {}

                x if x == clc_ops_e::clc_move as c_int => {
                    SV_UserMove(cl, msg);
                }

                x if x == clc_ops_e::clc_clientCommand as c_int => {
                    SV_ClientCommand(cl, msg);
                    if (*cl).state == clientState_t::CS_ZOMBIE {
                        return; // disconnect command
                    }
                }

                _ => {
                    SV_DropClient(cl, b"had an unknown command char\0".as_ptr() as *const c_char);
                    return;
                }
            }
        }
    }
}

pub extern "C" fn SV_FreeClient(client: *mut client_s) {
    let mut i: c_int;

    unsafe {
        if client.is_null() {
            return;
        }

        i = 0;
        while i < MAX_RELIABLE_COMMANDS as c_int {
            if !(*client).reliableCommands[i as usize].is_null() {
                FS_FreeFile((*client).reliableCommands[i as usize] as *mut c_void);
                (*client).reliableCommands[i as usize] = std::ptr::null_mut();
                (*client).reliableSequence = 0;
            }
            i += 1;
        }
    }
}
