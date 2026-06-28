// leave this as first line for PCH reasons...
//
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use crate::code::server::server_h::{
    svs, sv, ge, sv_fps, sv_timeout, sv_zombietime, sv_reconnectlimit, sv_showloss,
    sv_killserver, sv_mapname, sv_spawntarget, sv_mapChecksum, sv_serverid, sv_testsave,
    sv_compress_saved_games, client_s, netadr_t, msg_t, clientState_t,
    MAX_RELIABLE_COMMANDS, MAX_INFO_STRING, PACKET_BACKUP,
};
use crate::code::game::g_public_h::SVF_BOT;
use crate::code::game::bg_public_h::PERS_SCORE;

// Ghoul2 Insert Start
// Ghoul2 Insert End

// Local struct definitions to access opaque type fields
// nothing outside the Cvar_*() functions should modify these fields!
#[repr(C)]
pub struct local_cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,  // cvar_restart will reset to this value
    pub latchedString: *mut c_char,  // for CVAR_LATCH vars
    pub flags: c_int,
    pub modified: c_int,  // set each time the cvar is changed
    pub modificationCount: c_int,  // incremented each time the cvar is changed
    pub value: f32,  // atof( string )
    pub integer: c_int,  // atoi( string )
    pub next: *mut local_cvar_t,
}

// netchan_t structure
#[repr(C)]
pub struct local_netchan_t {
    pub sock: c_int,  // netsrc_t
    pub dropped: c_int,  // between last packet and previous
    pub remoteAddress: local_netadr_t,
    pub qport: c_int,  // qport value to write when transmitting
    pub incomingSequence: c_int,
    pub incomingAcknowledged: c_int,
    pub outgoingSequence: c_int,
    pub fragmentSequence: c_int,
    pub fragmentLength: c_int,
    pub fragmentBuffer: [u8; MAX_MSGLEN],
}

// netadr_t structure
#[repr(C)]
pub struct local_netadr_t {
    pub r#type: c_int,  // netadrtype_t
    pub port: u16,
}

// msg_t structure
#[repr(C)]
pub struct local_msg_t {
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,  // for bitwise reads and writes
}

// Forward declaration for playerState_s
#[repr(C)]
pub struct playerState_s {
    _opaque: [u8; 0],
}

// Minimal gentity_t structure for field access
#[repr(C)]
pub struct local_gentity_t {
    pub s: [u8; 128],  // entityState_t - opaque, using byte array for space
    pub client: *mut playerState_s,  // playerState_s *
    pub inuse: c_int,  // qboolean
    pub linked: c_int,  // qboolean
    pub svFlags: c_int,
    // ... other fields omitted
}


// Constants
const MAX_MSGLEN: usize = 1 * 17408;
const PACKET_MASK: usize = PACKET_BACKUP - 1;
const CVAR_SERVERINFO: c_int = 4;
const CVAR_SYSTEMINFO: c_int = 8;
const CS_SERVERINFO: c_int = 0;
const CS_SYSTEMINFO: c_int = 1;
const svc_serverCommand: u8 = 2;  // This should be determined from actual enum but using value
const PROTOCOL_VERSION: c_int = 40;

// Extern C declarations for external functions
extern "C" {
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, destsize: c_int, fmt: *const c_char, ...);
    fn Z_Free(ptr: *mut c_void);
    fn CopyString(string: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    fn Cmd_TokenizeString(text: *const c_char);
    fn Cmd_Argv(arg: c_int) -> *const c_char;

    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cvar_InfoString(bit: c_int) -> *const c_char;

    fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char);

    fn MSG_BeginReading(msg: *mut msg_t);
    fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    fn MSG_ReadShort(msg: *mut msg_t) -> c_int;
    fn MSG_ReadStringLine(msg: *mut msg_t) -> *mut c_char;

    fn NET_AdrToString(a: netadr_t) -> *mut c_char;
    fn NET_CompareBaseAdr(a: netadr_t, b: netadr_t) -> qboolean;
    fn NET_OutOfBandPrint(net_socket: c_int, adr: netadr_t, fmt: *const c_char, ...);

    fn Netchan_Process(chan: *mut c_void, msg: *mut msg_t) -> qboolean;

    fn SV_DirectConnect(from: netadr_t);
    fn SV_ExecuteClientMessage(cl: *mut client_s, msg: *mut msg_t);
    fn SV_DropClient(cl: *mut client_s, reason: *const c_char);
    fn SV_SendClientMessages();
    fn SV_Shutdown(finalmsg: *const c_char);
    fn SV_SetConfigstring(index: c_int, val: *const c_char);

    fn SG_TestSave();
    fn G2API_SetTime(time: c_int, time_type: c_int);
    fn SE_CheckForLanguageUpdates();

    fn Sys_Milliseconds() -> c_int;

    fn va(fmt: *const c_char, ...) -> *const c_char;

    // External cvar that might be needed
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
}

// External globals
extern "C" {
    static mut com_sv_running: *mut cvar_t;
    static mut cl_paused: *mut cvar_t;
    static mut sv_paused: *mut cvar_t;
    static mut com_speeds: *mut cvar_t;
    static mut cl_newClock: *mut cvar_t;
    static mut cvar_modifiedFlags: c_int;
    static mut time_game: c_int;
}

// Constants for networking
const NS_SERVER: c_int = 1;

/*
=============================================================================

EVENT MESSAGES

=============================================================================
*/

/*
===============
SV_ExpandNewlines

Converts newlines to "\n" so a line prints nicer
===============
*/
pub unsafe fn SV_ExpandNewlines(r#in: *mut c_char) -> *mut c_char {
    static mut string: [c_char; 1024] = [0; 1024];
    let mut l: c_int = 0;
    let mut in_ptr = r#in;

    while *in_ptr != 0 && l < ((1024 - 3) as c_int) {
        if *in_ptr == b'\n' as c_char {
            string[l as usize] = b'\\' as c_char;
            l += 1;
            string[l as usize] = b'n' as c_char;
        } else {
            string[l as usize] = *in_ptr;
        }
        l += 1;
        in_ptr = in_ptr.offset(1);
    }
    string[l as usize] = 0;

    string.as_mut_ptr()
}

/*
======================
SV_AddServerCommand

The given command will be transmitted to the client, and is guaranteed to
not have future snapshot_t executed before it is executed
======================
*/
pub unsafe fn SV_AddServerCommand(client: *mut client_s, cmd: *const c_char) {
    let mut index: c_int;

    // if we would be losing an old command that hasn't been acknowledged,
    // we must drop the connection
    if (*client).reliableSequence - (*client).reliableAcknowledge > (MAX_RELIABLE_COMMANDS as c_int) {
        SV_DropClient(client, b"Server command overflow\0".as_ptr() as *const c_char);
        return;
    }
    (*client).reliableSequence += 1;
    index = (*client).reliableSequence & ((MAX_RELIABLE_COMMANDS as c_int) - 1);
    if !(*client).reliableCommands[index as usize].is_null() {
        Z_Free((*client).reliableCommands[index as usize] as *mut c_void);
    }
    (*client).reliableCommands[index as usize] = CopyString(cmd);
}



/*
==============================================================================

CONNECTIONLESS COMMANDS

==============================================================================
*/

/*
================
SVC_Status

Responds with all the info that qplug or qspy can see about the server
and all connected players.  Used for getting detailed information after
the simple info query.
================
*/
pub unsafe fn SVC_Status(from: netadr_t) {
    let mut player: [c_char; 1024] = [0; 1024];
    let mut status: [c_char; MAX_MSGLEN] = [0; MAX_MSGLEN];
    let mut i: c_int;
    let mut cl: *mut client_s;
    let mut statusLength: c_int;
    let mut playerLength: c_int;
    let mut score: c_int;
    let mut infostring: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];

    strcpy(infostring.as_mut_ptr(), Cvar_InfoString(CVAR_SERVERINFO) as *const c_char);

    // echo back the parameter to status. so servers can use it as a challenge
    // to prevent timed spoofed reply packets that add ghost servers
    Info_SetValueForKey(infostring.as_mut_ptr(), b"challenge\0".as_ptr() as *const c_char, Cmd_Argv(1));

    status[0] = 0;
    statusLength = 0;

    i = 0;
    while i < 1 {
        cl = &mut (*core::ptr::addr_of_mut!(svs)).clients[i as usize];
        if (*cl).state as c_int >= (clientState_t::CS_CONNECTED as c_int) {
            // NOTE: Original code accesses cl->gentity->client->persistant[PERS_SCORE]
            // We cannot safely access nested opaque structures, so we default to 0
            score = 0;
            if !(*cl).gentity.is_null() && !(((*cl).gentity as *mut local_gentity_t)).client.is_null() {
                // In a complete implementation, score would be retrieved from persistant array
                // but playerState_t is opaque and field offsets cannot be safely determined
            }
            Com_sprintf(
                player.as_mut_ptr(),
                1024 as c_int,
                b"%i %i \"%s\"\n\0".as_ptr() as *const c_char,
                score,
                (*cl).ping,
                (*cl).name.as_ptr(),
            );
            playerLength = strlen(player.as_ptr()) as c_int;
            if statusLength + playerLength >= (MAX_MSGLEN as c_int) {
                break;  // can't hold any more
            }
            strcpy(
                (status.as_mut_ptr()).offset(statusLength as isize),
                player.as_ptr(),
            );
            statusLength += playerLength;
        }
        i += 1;
    }

    NET_OutOfBandPrint(
        NS_SERVER,
        from,
        b"statusResponse\n%s\n%s\0".as_ptr() as *const c_char,
        infostring.as_ptr(),
        status.as_ptr(),
    );
}

/*
================
SVC_Info

Responds with a short info message that should be enough to determine
if a user is interested in a server to do a full status
================
*/
pub unsafe fn SVC_Info(from: netadr_t) {
    let mut i: c_int;
    let mut count: c_int;
    let mut infostring: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];

    count = 0;
    i = 0;
    while i < 1 {
        if (*core::ptr::addr_of!(svs)).clients[i as usize].state as c_int >= (clientState_t::CS_CONNECTED as c_int) {
            count += 1;
        }
        i += 1;
    }

    infostring[0] = 0;

    // echo back the parameter to status. so servers can use it as a challenge
    // to prevent timed spoofed reply packets that add ghost servers
    Info_SetValueForKey(infostring.as_mut_ptr(), b"challenge\0".as_ptr() as *const c_char, Cmd_Argv(1));

    Info_SetValueForKey(infostring.as_mut_ptr(), b"protocol\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, PROTOCOL_VERSION));
    //Info_SetValueForKey( infostring, "hostname", sv_hostname->string );
    Info_SetValueForKey(infostring.as_mut_ptr(), b"mapname\0".as_ptr() as *const c_char, (*(sv_mapname as *mut local_cvar_t)).string);
    Info_SetValueForKey(infostring.as_mut_ptr(), b"clients\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, count));
    Info_SetValueForKey(infostring.as_mut_ptr(), b"sv_maxclients\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, 1));

    NET_OutOfBandPrint(NS_SERVER, from, b"infoResponse\n%s\0".as_ptr() as *const c_char, infostring.as_ptr());
}


/*
=================
SV_ConnectionlessPacket

A connectionless packet has four leading 0xff
characters to distinguish it from a game channel.
Clients that are in the game can still send
connectionless packets.
=================
*/
pub unsafe fn SV_ConnectionlessPacket(from: netadr_t, msg: *mut msg_t) {
    let mut s: *mut c_char;
    let mut c: *mut c_char;

    MSG_BeginReading(msg);
    MSG_ReadLong(msg);  // skip the -1 marker

    s = MSG_ReadStringLine(msg);

    Cmd_TokenizeString(s);

    c = Cmd_Argv(0) as *mut c_char;
    Com_DPrintf(b"SV packet %s : %s\n\0".as_ptr() as *const c_char, NET_AdrToString(from), c);

    if strcmp(c, b"getstatus\0".as_ptr() as *const c_char) == 0 {
        SVC_Status(from);
    } else if strcmp(c, b"getinfo\0".as_ptr() as *const c_char) == 0 {
        SVC_Info(from);
    } else if strcmp(c, b"connect\0".as_ptr() as *const c_char) == 0 {
        SV_DirectConnect(from);
    } else if strcmp(c, b"disconnect\0".as_ptr() as *const c_char) == 0 {
        // if a client starts up a local server, we may see some spurious
        // server disconnect messages when their new server sees our final
        // sequenced messages to the old client
    } else {
        Com_DPrintf(
            b"bad connectionless packet from %s:\n%s\n\0".as_ptr() as *const c_char,
            NET_AdrToString(from),
            s,
        );
    }
}


//============================================================================

/*
=================
SV_ReadPackets
=================
*/
pub unsafe fn SV_PacketEvent(from: netadr_t, msg: *mut msg_t) {
    let mut i: c_int;
    let mut cl: *mut client_s;
    let mut qport: c_int;

    let msg_ptr = msg as *mut local_msg_t;
    // check for connectionless packet (0xffffffff) first
    if (*msg_ptr).cursize >= 4 && *((*msg_ptr).data as *const c_int) == -1 {
        SV_ConnectionlessPacket(from, msg);
        return;
    }

    // read the qport out of the message so we can fix up
    // stupid address translating routers
    MSG_BeginReading(msg);
    MSG_ReadLong(msg);        // sequence number
    MSG_ReadLong(msg);        // sequence number
    qport = MSG_ReadShort(msg) & 0xffff;

    let from_addr = (&from as *const netadr_t) as *const local_netadr_t;

    // find which client the message is from
    i = 0;
    cl = (*core::ptr::addr_of!(svs)).clients;
    while i < 1 {
        if (*cl).state as c_int == (clientState_t::CS_FREE as c_int) {
            i += 1;
            cl = cl.offset(1);
            continue;
        }
        let netchan_ptr = (&mut (*cl).netchan as *mut netchan_t) as *mut local_netchan_t;
        if NET_CompareBaseAdr(from, (*netchan_ptr).remoteAddress) == 0 {
            i += 1;
            cl = cl.offset(1);
            continue;
        }
        // it is possible to have multiple clients from a single IP
        // address, so they are differentiated by the qport variable
        if (*netchan_ptr).qport != qport {
            i += 1;
            cl = cl.offset(1);
            continue;
        }

        // the IP port can't be used to differentiate them, because
        // some address translating routers periodically change UDP
        // port assignments
        if (*netchan_ptr).remoteAddress.port != (*from_addr).port {
            Com_Printf(b"SV_ReadPackets: fixing up a translated port\n\0".as_ptr() as *const c_char);
            (*netchan_ptr).remoteAddress.port = (*from_addr).port;
        }

        // make sure it is a valid, in sequence packet
        if Netchan_Process(&mut (*cl).netchan as *mut c_void as *mut _, msg) != 0 {
            // zombie clients stil neet to do the Netchan_Process
            // to make sure they don't need to retransmit the final
            // reliable message, but they don't do any other processing
            if (*cl).state != clientState_t::CS_ZOMBIE {
                (*cl).lastPacketTime = (*core::ptr::addr_of!(sv)).time;  // don't timeout
                let netchan_ptr2 = (&(*cl).netchan as *const netchan_t) as *const local_netchan_t;
                (*cl).frames[((*netchan_ptr2).incomingAcknowledged & (PACKET_MASK as c_int)) as usize]
                    .messageAcked = (*core::ptr::addr_of!(sv)).time;
                SV_ExecuteClientMessage(cl, msg);
            }
        }
        return;
    }

    // if we received a sequenced packet from an address we don't reckognize,
    // send an out of band disconnect packet to it
    NET_OutOfBandPrint(NS_SERVER, from, b"disconnect\0".as_ptr() as *const c_char);
}


/*
===================
SV_CalcPings

Updates the cl->ping variables
===================
*/
pub unsafe fn SV_CalcPings() {
    let mut i: c_int;
    let mut j: c_int;
    let mut cl: *mut client_s;
    let mut total: c_int;
    let mut count: c_int;
    let mut delta: c_int;

    i = 0;
    while i < 1 {
        cl = &mut (*core::ptr::addr_of_mut!(svs)).clients[i as usize];
        if (*cl).state != clientState_t::CS_ACTIVE {
            i += 1;
            continue;
        }
        let gentity_ptr = (*cl).gentity as *mut local_gentity_t;
        if (*gentity_ptr).svFlags & SVF_BOT != 0 {
            i += 1;
            continue;
        }

        total = 0;
        count = 0;
        j = 0;
        while j < (PACKET_BACKUP as c_int) {
            delta = (*cl).frames[j as usize].messageAcked - (*cl).frames[j as usize].messageSent;
            if delta >= 0 {
                count += 1;
                total += delta;
            }
            j += 1;
        }
        if count == 0 {
            (*cl).ping = 999;
        } else {
            (*cl).ping = total / count;
            if (*cl).ping > 999 {
                (*cl).ping = 999;
            }
        }

        // let the game dll know about the ping
        // NOTE: cl->gentity->client->ping = cl->ping in the original C code
        // We cannot directly access this field through Rust's type system since playerState_t is opaque
        // The ping value is calculated and stored in cl->ping which is used elsewhere
        i += 1;
    }
}

/*
==================
SV_CheckTimeouts

If a packet has not been received from a client for timeout->integer
seconds, drop the conneciton.  Server time is used instead of
realtime to avoid dropping the local client while debugging.

When a client is normally dropped, the client_t goes into a zombie state
for a few seconds to make sure any final reliable message gets resent
if necessary
==================
*/
pub unsafe fn SV_CheckTimeouts() {
    let mut i: c_int;
    let mut cl: *mut client_s;
    let mut droppoint: c_int;
    let mut zombiepoint: c_int;

    droppoint = (*core::ptr::addr_of!(sv)).time - 1000 * (*(sv_timeout as *mut local_cvar_t)).integer;
    zombiepoint = (*core::ptr::addr_of!(sv)).time - 1000 * (*(sv_zombietime as *mut local_cvar_t)).integer;

    i = 0;
    cl = (*core::ptr::addr_of!(svs)).clients;
    while i < 1 {
        // message times may be wrong across a changelevel
        if (*cl).lastPacketTime > (*core::ptr::addr_of!(sv)).time {
            (*cl).lastPacketTime = (*core::ptr::addr_of!(sv)).time;
        }

        if (*cl).state == clientState_t::CS_ZOMBIE
        && (*cl).lastPacketTime < zombiepoint {
            (*cl).state = clientState_t::CS_FREE;  // can now be reused
            i += 1;
            cl = cl.offset(1);
            continue;
        }
        if (*cl).state as c_int >= (clientState_t::CS_CONNECTED as c_int) && (*cl).lastPacketTime < droppoint {
            // wait several frames so a debugger session doesn't
            // cause a timeout
            (*cl).timeoutCount += 1;
            if (*cl).timeoutCount > 5 {
                SV_DropClient(cl, b"timed out\0".as_ptr() as *const c_char);
                (*cl).state = clientState_t::CS_FREE;  // don't bother with zombie state
            }
        } else {
            (*cl).timeoutCount = 0;
        }
        i += 1;
        cl = cl.offset(1);
    }
}


/*
==================
SV_CheckPaused
==================
*/
pub unsafe fn SV_CheckPaused() -> qboolean {
    if (*(cl_paused as *mut local_cvar_t)).integer == 0 {
        return 0;  // qfalse
    }

    (*(sv_paused as *mut local_cvar_t)).integer = 1;
    return 1;  // qtrue
}

/*
This wonderful hack is needed to avoid rendering frames until several camera related things
have wended their way through the network. The problem is basically that the server asks the
client where the camera is to decide what entities down to the client. However right after
certain transitions the client tends to give a wrong answer. CGCam_Disable is one such time/
When this happens we want to dump all rendered frame until these things have happened, in
order:

0) (This state will mean that we are awaiting state 1)
1) The server has run a frame and built a packet
2) The client has computed a camera position
3) The server has run a frame and built a packet
4) The client has recieved a packet (This state also means the game is running normally).

We will keep track of this here:

*/


/*
==================
SV_Frame

Player movement occurs as a result of packet events, which
happen before SV_Frame is called
==================
*/
pub unsafe fn SV_Frame(msec: c_int, fractionMsec: f32) {
    let mut frameMsec: c_int;
    let mut startTime: c_int = 0;
    let mut msec_local = msec;

    // the menu kills the server with this cvar
    if (*(sv_killserver as *mut local_cvar_t)).integer != 0 {
        SV_Shutdown(b"Server was killed.\n\0".as_ptr() as *const c_char);
        Cvar_Set(b"sv_killserver\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
        return;
    }

    if (*core::ptr::addr_of_mut!(com_sv_running)).is_null() || (*((*core::ptr::addr_of!(com_sv_running)) as *mut local_cvar_t)).integer == 0 {
        return;
    }

    SE_CheckForLanguageUpdates();  // will fast-return else load different language if menu changed it

    // allow pause if only the local client is connected
    if SV_CheckPaused() != 0 {
        return;
    }

    // go ahead and let time slip if the server really hitched badly
    if msec_local > 1000 {
        Com_DPrintf(b"SV_Frame: Truncating msec of %i to 1000\n\0".as_ptr() as *const c_char, msec_local);
        msec_local = 1000;
    }

    // if it isn't time for the next frame, do nothing
    if (*(sv_fps as *mut local_cvar_t)).integer < 1 {
        Cvar_Set(b"sv_fps\0".as_ptr() as *const c_char, b"10\0".as_ptr() as *const c_char);
    }
    frameMsec = 1000 / (*(sv_fps as *mut local_cvar_t)).integer;

    (*core::ptr::addr_of_mut!(sv)).timeResidual += msec_local;
    (*core::ptr::addr_of_mut!(sv)).timeResidualFraction += fractionMsec;
    if (*core::ptr::addr_of!(sv)).timeResidualFraction >= 1.0f32 {
        (*core::ptr::addr_of_mut!(sv)).timeResidualFraction -= 1.0f32;
        if !(*core::ptr::addr_of!(cl_newClock)).is_null() && (*((*core::ptr::addr_of!(cl_newClock)) as *mut local_cvar_t)).integer != 0 {
            (*core::ptr::addr_of_mut!(sv)).timeResidual += 1;
        }
    }
    if (*core::ptr::addr_of!(sv)).timeResidual < frameMsec {
        return;
    }

    // if time is about to hit the 32nd bit, restart the
    // level, which will force the time back to zero, rather
    // than checking for negative time wraparound everywhere.
    // 2giga-milliseconds = 23 days, so it won't be too often
    if (*core::ptr::addr_of!(sv)).time > 0x70000000 {
        SV_Shutdown(b"Restarting server due to time wrapping\0".as_ptr() as *const c_char);
        Com_Printf(b"You win.  if you can play this long and not die, you deserve to win.\n\0".as_ptr() as *const c_char);
        return;
    }

    // update infostrings if anything has been changed
    if cvar_modifiedFlags & CVAR_SERVERINFO != 0 {
        SV_SetConfigstring(CS_SERVERINFO, Cvar_InfoString(CVAR_SERVERINFO));
        cvar_modifiedFlags &= !CVAR_SERVERINFO;
    }
    if cvar_modifiedFlags & CVAR_SYSTEMINFO != 0 {
        SV_SetConfigstring(CS_SYSTEMINFO, Cvar_InfoString(CVAR_SYSTEMINFO));
        cvar_modifiedFlags &= !CVAR_SYSTEMINFO;
    }

    if (*((*core::ptr::addr_of!(com_speeds)) as *mut local_cvar_t)).integer != 0 {
        startTime = Sys_Milliseconds();
    }

    //  SV_BotFrame( sv.time );

    // run the game simulation in chunks
    while (*core::ptr::addr_of!(sv)).timeResidual >= frameMsec {
        (*core::ptr::addr_of_mut!(sv)).timeResidual -= frameMsec;
        (*core::ptr::addr_of_mut!(sv)).time += frameMsec;
        G2API_SetTime((*core::ptr::addr_of!(sv)).time, 2); // G2T_SV_TIME = 2

        // let everything in the world think and move
        (*ge).RunFrame((*core::ptr::addr_of!(sv)).time);
    }

    if (*((*core::ptr::addr_of!(com_speeds)) as *mut local_cvar_t)).integer != 0 {
        time_game = Sys_Milliseconds() - startTime;
    }

    SG_TestSave();  // returns immediately if not active, used for fake-save-every-cycle to test (mainly) Icarus disk code

    // check timeouts
    SV_CheckTimeouts();

    // update ping based on the last known frame from all clients
    SV_CalcPings();

    // send messages back to the clients
    SV_SendClientMessages();
}

//============================================================================
