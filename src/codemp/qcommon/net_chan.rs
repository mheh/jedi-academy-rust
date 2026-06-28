//! Mechanical port of `codemp/qcommon/net_chan.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};
use crate::codemp::game::q_shared_h::byte;

// ============================================================================
// Type definitions (from qcommon.h, q_shared.h, and local)
// ============================================================================

/// Network source type: client or server.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum netsrc_t {
    NS_CLIENT,
    NS_SERVER,
}

/// Network address type.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum netadrtype_t {
    NA_BOT,
    NA_BAD,
    NA_LOOPBACK,
    NA_BROADCAST,
    NA_IP,
    NA_IPX,
    NA_BROADCAST_IPX,
}

/// Network address structure.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct netadr_t {
    pub r#type: netadrtype_t,
    pub ip: [byte; 4],
    pub ipx: [byte; 10],
    pub port: u16,
}

/// Message structure.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct msg_t {
    pub allowoverflow: c_int,
    pub overflowed: c_int,
    pub oob: c_int,
    pub data: *mut byte,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

/// Network channel structure for packet fragmentation and sequencing.
#[repr(C)]
#[derive(Debug)]
pub struct netchan_t {
    pub sock: netsrc_t,
    pub dropped: c_int,
    pub remoteAddress: netadr_t,
    pub qport: c_int,
    pub incomingSequence: c_int,
    pub outgoingSequence: c_int,
    pub fragmentSequence: c_int,
    pub fragmentLength: c_int,
    pub fragmentBuffer: [byte; MAX_MSGLEN],
    pub unsentFragments: c_int,
    pub unsentFragmentStart: c_int,
    pub unsentLength: c_int,
    pub unsentBuffer: [byte; MAX_MSGLEN],
}

/// C string constant for netsrc_t values.
pub type cvar_t = c_void;

// ============================================================================
// Constants
// ============================================================================

#[cfg(target_os = "xbox")]
pub const MAX_PACKETLEN: usize = 1359;
#[cfg(not(target_os = "xbox"))]
pub const MAX_PACKETLEN: usize = 1400;

#[cfg(target_os = "xbox")]
pub const FRAGMENT_SIZE: usize = MAX_PACKETLEN - 55 - 10;
#[cfg(not(target_os = "xbox"))]
pub const FRAGMENT_SIZE: usize = MAX_PACKETLEN - 100;

#[cfg(not(target_os = "xbox"))]
pub const PACKET_HEADER: usize = 10;

pub const FRAGMENT_BIT: c_int = 1 << 31;

pub const MAX_MSGLEN: usize = 49152;
pub const MAX_LOOPBACK: usize = 16;

// ============================================================================
// Global cvars
// ============================================================================

pub static mut showpackets: *mut cvar_t = core::ptr::null_mut();
pub static mut showdrop: *mut cvar_t = core::ptr::null_mut();
pub static mut qport: *mut cvar_t = core::ptr::null_mut();
pub static mut net_killdroppedfragments: *mut cvar_t = core::ptr::null_mut();

// ============================================================================
// String table for netsrc_t printing
// ============================================================================

static netsrcString: [&str; 2] = [
    "client",
    "server",
];

// ============================================================================
// Loopback message buffer
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct loopmsg_t {
    pub data: [byte; MAX_PACKETLEN],
    pub datalen: c_int,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct loopback_t {
    pub msgs: [loopmsg_t; MAX_LOOPBACK],
    pub get: c_int,
    pub send: c_int,
}

pub static mut loopbacks: [loopback_t; 2] = [loopback_t {
    msgs: [loopmsg_t {
        data: [0; MAX_PACKETLEN],
        datalen: 0,
    }; MAX_LOOPBACK],
    get: 0,
    send: 0,
}; 2];

// ============================================================================
// External C functions (declared locally)
// ============================================================================

extern "C" {
    pub fn Cvar_Get(
        var_name: *const c_char,
        var_value: *const c_char,
        flags: c_int,
    ) -> *mut cvar_t;

    pub fn Com_Memset(dest: *mut c_void, c: c_int, size: usize);
    pub fn Com_Memcpy(dest: *mut c_void, src: *const c_void, size: usize);

    pub fn MSG_InitOOB(buf: *mut msg_t, data: *mut byte, length: usize);
    pub fn MSG_WriteLong(buf: *mut msg_t, c: c_int);
    pub fn MSG_WriteShort(buf: *mut msg_t, c: c_int);
    pub fn MSG_WriteData(buf: *mut msg_t, data: *const c_void, length: usize);

    pub fn MSG_BeginReadingOOB(msg: *mut msg_t);
    pub fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    pub fn MSG_ReadShort(msg: *mut msg_t) -> c_int;

    pub fn NET_SendPacket(sock: netsrc_t, length: c_int, data: *const c_void, to: netadr_t);

    pub fn Com_Printf(format: *const c_char, ...);
    pub fn Com_Error(code: c_int, format: *const c_char, ...);

    pub fn Sys_SendPacket(length: c_int, data: *const c_void, to: netadr_t);

    pub fn Com_sprintf(dest: *mut c_char, size: usize, format: *const c_char, ...);

    pub fn Huff_Compress(buf: *mut msg_t, offset: c_int);

    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, n: usize);
    pub fn Sys_StringToAdr(s: *const c_char, a: *mut netadr_t) -> c_int;

    pub fn BigShort(l: c_int) -> u16;

    pub fn va(format: *const c_char, ...) -> *const c_char;

    pub fn vsprintf(dest: *mut c_char, format: *const c_char, argptr: core::ffi::VaList) -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    pub fn atoi(s: *const c_char) -> c_int;

    // NET_OutOfBandPrint is variadic and cannot be fully implemented in Rust;
    // the C implementation must be linked in.
    pub fn NET_OutOfBandPrint(sock: netsrc_t, adr: netadr_t, format: *const c_char, ...);
}

// ============================================================================
// Functions
// ============================================================================

/*
===============
Netchan_Init

===============
*/
pub unsafe fn Netchan_Init(port: c_int) {
    let port_masked = port & 0xffff;
    showpackets = Cvar_Get(
        b"showpackets\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        0x01, // CVAR_TEMP
    );
    showdrop = Cvar_Get(
        b"showdrop\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        0x01, // CVAR_TEMP
    );
    qport = Cvar_Get(
        b"net_qport\0".as_ptr() as *const c_char,
        va(b"%i\0".as_ptr() as *const c_char, port_masked),
        0x04, // CVAR_INIT
    );
    net_killdroppedfragments = Cvar_Get(
        b"net_killdroppedfragments\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        0x01, // CVAR_TEMP
    );
}

/*
==============
Netchan_Setup

called to open a channel to a remote system
==============
*/
pub unsafe fn Netchan_Setup(
    sock: netsrc_t,
    chan: *mut netchan_t,
    adr: netadr_t,
    qport_val: c_int,
) {
    Com_Memset(chan as *mut c_void, 0, core::mem::size_of::<netchan_t>());

    (*chan).sock = sock;
    (*chan).remoteAddress = adr;
    (*chan).qport = qport_val;
    (*chan).incomingSequence = 0;
    (*chan).outgoingSequence = 1;
}

/*
=================
Netchan_TransmitNextFragment

Send one fragment of the current message
=================
*/
pub unsafe fn Netchan_TransmitNextFragment(chan: *mut netchan_t) {
    let mut send: msg_t = core::mem::zeroed();
    let mut send_buf: [byte; MAX_PACKETLEN] = [0; MAX_PACKETLEN];
    let mut fragmentLength: c_int;

    // write the packet header
    MSG_InitOOB(&mut send, send_buf.as_mut_ptr(), core::mem::size_of_val(&send_buf));

    MSG_WriteLong(&mut send, (*chan).outgoingSequence | FRAGMENT_BIT);

    // send the qport if we are a client
    if (*chan).sock == netsrc_t::NS_CLIENT {
        if !qport.is_null() {
            MSG_WriteShort(&mut send, (*(*qport)).integer);
        }
    }

    // copy the reliable message to the packet first
    fragmentLength = FRAGMENT_SIZE as c_int;
    if (*chan).unsentFragmentStart + fragmentLength > (*chan).unsentLength {
        fragmentLength = (*chan).unsentLength - (*chan).unsentFragmentStart;
    }

    MSG_WriteShort(&mut send, (*chan).unsentFragmentStart);
    MSG_WriteShort(&mut send, fragmentLength);
    MSG_WriteData(
        &mut send,
        ((*chan).unsentBuffer.as_ptr() as *const u8).add((*chan).unsentFragmentStart as usize)
            as *const c_void,
        fragmentLength as usize,
    );

    // send the datagram
    NET_SendPacket((*chan).sock, send.cursize, send.data as *const c_void, (*chan).remoteAddress);

    if !showpackets.is_null() && (*(*showpackets)).integer != 0 {
        Com_Printf(
            b"%s send %4i : s=%i fragment=%i,%i\n\0".as_ptr() as *const c_char,
            netsrcString[(*chan).sock as usize].as_ptr(),
            send.cursize,
            (*chan).outgoingSequence - 1,
            (*chan).unsentFragmentStart,
            fragmentLength,
        );
    }

    (*chan).unsentFragmentStart += fragmentLength;

    // this exit condition is a little tricky, because a packet
    // that is exactly the fragment length still needs to send
    // a second packet of zero length so that the other side
    // can tell there aren't more to follow
    if (*chan).unsentFragmentStart == (*chan).unsentLength
        && fragmentLength != FRAGMENT_SIZE as c_int
    {
        (*chan).outgoingSequence += 1;
        (*chan).unsentFragments = 0; // qfalse
    }
}

/*
===============
Netchan_Transmit

Sends a message to a connection, fragmenting if necessary
A 0 length will still generate a packet.
================
*/
pub unsafe fn Netchan_Transmit(chan: *mut netchan_t, length: c_int, data: *const byte) {
    let mut send: msg_t = core::mem::zeroed();
    let mut send_buf: [byte; MAX_PACKETLEN] = [0; MAX_PACKETLEN];

    if length > MAX_MSGLEN as c_int {
        Com_Error(1, b"Netchan_Transmit: length = %i\0".as_ptr() as *const c_char, length);
    }
    (*chan).unsentFragmentStart = 0;

    if (*chan).unsentFragments != 0 {
        Com_Printf(
            b"[ISM] Stomping Unsent Fragments %s\n\0".as_ptr() as *const c_char,
            netsrcString[(*chan).sock as usize].as_ptr(),
        );
    }

    // fragment large reliable messages
    if length >= FRAGMENT_SIZE as c_int {
        (*chan).unsentFragments = 1; // qtrue
        (*chan).unsentLength = length;
        Com_Memcpy(
            (*chan).unsentBuffer.as_mut_ptr() as *mut c_void,
            data as *const c_void,
            length as usize,
        );

        // only send the first fragment now
        Netchan_TransmitNextFragment(chan);

        return;
    }

    // write the packet header
    MSG_InitOOB(&mut send, send_buf.as_mut_ptr(), core::mem::size_of_val(&send_buf));

    MSG_WriteLong(&mut send, (*chan).outgoingSequence);
    (*chan).outgoingSequence += 1;

    // send the qport if we are a client
    if (*chan).sock == netsrc_t::NS_CLIENT {
        if !qport.is_null() {
            MSG_WriteShort(&mut send, (*(*qport)).integer);
        }
    }

    MSG_WriteData(&mut send, data as *const c_void, length as usize);

    // send the datagram
    NET_SendPacket((*chan).sock, send.cursize, send.data as *const c_void, (*chan).remoteAddress);

    if !showpackets.is_null() && (*(*showpackets)).integer != 0 {
        Com_Printf(
            b"%s send %4i : s=%i ack=%i\n\0".as_ptr() as *const c_char,
            netsrcString[(*chan).sock as usize].as_ptr(),
            send.cursize,
            (*chan).outgoingSequence - 1,
            (*chan).incomingSequence,
        );
    }
}

/*
=================
Netchan_Process

Returns qfalse if the message should not be processed due to being
out of order or a fragment.

Msg must be large enough to hold MAX_MSGLEN, because if this is the
final fragment of a multi-part message, the entire thing will be
copied out.
=================
*/
pub unsafe fn Netchan_Process(chan: *mut netchan_t, msg: *mut msg_t) -> c_int {
    let mut sequence: c_int;
    let mut qport_val: c_int;
    let mut fragmentStart: c_int;
    let mut fragmentLength: c_int;
    let mut fragmented: c_int;

    // get sequence numbers
    MSG_BeginReadingOOB(msg);
    sequence = MSG_ReadLong(msg);

    // check for fragment information
    if (sequence & FRAGMENT_BIT) != 0 {
        sequence &= !FRAGMENT_BIT;
        fragmented = 1; // qtrue
    } else {
        fragmented = 0; // qfalse
    }

    // read the qport if we are a server
    if (*chan).sock == netsrc_t::NS_SERVER {
        qport_val = MSG_ReadShort(msg);
    }

    // read the fragment information
    if fragmented != 0 {
        fragmentStart = MSG_ReadShort(msg) as u16 as c_int;
        fragmentLength = MSG_ReadShort(msg) as u16 as c_int;
    } else {
        fragmentStart = 0; // stop warning message
        fragmentLength = 0;
    }

    if !showpackets.is_null() && (*(*showpackets)).integer != 0 {
        if fragmented != 0 {
            Com_Printf(
                b"%s recv %4i : s=%i fragment=%i,%i\n\0".as_ptr() as *const c_char,
                netsrcString[(*chan).sock as usize].as_ptr(),
                (*msg).cursize,
                sequence,
                fragmentStart,
                fragmentLength,
            );
        } else {
            Com_Printf(
                b"%s recv %4i : s=%i\n\0".as_ptr() as *const c_char,
                netsrcString[(*chan).sock as usize].as_ptr(),
                (*msg).cursize,
                sequence,
            );
        }
    }

    //
    // discard out of order or duplicated packets
    //
    if sequence <= (*chan).incomingSequence {
        if (!showdrop.is_null() && (*(*showdrop)).integer != 0)
            || (!showpackets.is_null() && (*(*showpackets)).integer != 0)
        {
            Com_Printf(
                b"%s:Out of order packet %i at %i\n\0".as_ptr() as *const c_char,
                NET_AdrToString((*chan).remoteAddress),
                sequence,
                (*chan).incomingSequence,
            );
        }
        return 0; // qfalse
    }

    //
    // dropped packets don't keep the message from being used
    //
    (*chan).dropped = sequence - ((*chan).incomingSequence + 1);
    if (*chan).dropped > 0 {
        if (!showdrop.is_null() && (*(*showdrop)).integer != 0)
            || (!showpackets.is_null() && (*(*showpackets)).integer != 0)
        {
            Com_Printf(
                b"%s:Dropped %i packets at %i\n\0".as_ptr() as *const c_char,
                NET_AdrToString((*chan).remoteAddress),
                (*chan).dropped,
                sequence,
            );
        }
    }

    //
    // if this is the final framgent of a reliable message,
    // bump incoming_reliable_sequence
    //
    if fragmented != 0 {
        // make sure we
        if sequence != (*chan).fragmentSequence {
            (*chan).fragmentSequence = sequence;
            (*chan).fragmentLength = 0;
        }

        // if we missed a fragment, dump the message
        if fragmentStart != (*chan).fragmentLength {
            if (!showdrop.is_null() && (*(*showdrop)).integer != 0)
                || (!showpackets.is_null() && (*(*showpackets)).integer != 0)
            {
                Com_Printf(
                    b"%s:Dropped a message fragment\n\0".as_ptr() as *const c_char,
                    NET_AdrToString((*chan).remoteAddress),
                    sequence,
                );
            }
            // we can still keep the part that we have so far,
            // so we don't need to clear chan->fragmentLength

            //rww - not clearing this will allow us to piece together fragments belonging to other packets
            //that happen to have the same sequence (or so it seems). I am just going to clear it and force
            //the packet to be dropped.

            // hell yeah we have to dump the whole thing -gil
            // but I am scared - mw
            /*
            chan->fragmentLength = 0;
            chan->incomingSequence = sequence;
            chan->fragmentSequence = 0;
            */
            return 0; // qfalse
        }

        // copy the fragment to the fragment buffer
        if fragmentLength < 0
            || (*msg).readcount + fragmentLength > (*msg).cursize
            || (*chan).fragmentLength + fragmentLength > core::mem::size_of_val(&(*chan).fragmentBuffer) as c_int
        {
            if (!showdrop.is_null() && (*(*showdrop)).integer != 0)
                || (!showpackets.is_null() && (*(*showpackets)).integer != 0)
            {
                Com_Printf(
                    b"%s:illegal fragment length\n\0".as_ptr() as *const c_char,
                    NET_AdrToString((*chan).remoteAddress),
                );
            }
            return 0; // qfalse
        }

        Com_Memcpy(
            ((*chan).fragmentBuffer.as_mut_ptr() as *mut u8)
                .add((*chan).fragmentLength as usize) as *mut c_void,
            ((*msg).data as *const u8).add((*msg).readcount as usize) as *const c_void,
            fragmentLength as usize,
        );

        (*chan).fragmentLength += fragmentLength;

        // if this wasn't the last fragment, don't process anything
        if fragmentLength == FRAGMENT_SIZE as c_int {
            return 0; // qfalse
        }

        if (*chan).fragmentLength + 4 > (*msg).maxsize {
            Com_Printf(
                b"%s:fragmentLength %i > msg->maxsize\n\0".as_ptr() as *const c_char,
                NET_AdrToString((*chan).remoteAddress),
                (*chan).fragmentLength + 4,
            );
            return 0; // qfalse
        }

        // copy the full message over the partial fragment

        // make sure the sequence number is still there
        *((*msg).data as *mut c_int) = LittleLong(sequence);

        Com_Memcpy(
            ((*msg).data as *mut u8).add(4) as *mut c_void,
            (*chan).fragmentBuffer.as_ptr() as *const c_void,
            (*chan).fragmentLength as usize,
        );
        (*msg).cursize = (*chan).fragmentLength + 4;
        (*chan).fragmentLength = 0;
        (*msg).readcount = 4; // past the sequence number
        (*msg).bit = 32; // past the sequence number

        // but I am a wuss -mw
        // chan->incomingSequence = sequence;   // lets not accept any more with this sequence number -gil
        return 1; // qtrue
    }

    //
    // the message can now be read from the current message pointer
    //
    (*chan).incomingSequence = sequence;

    return 1; // qtrue
}

//==============================================================================

/*
===================
NET_CompareBaseAdr

Compares without the port
===================
*/
pub unsafe fn NET_CompareBaseAdr(a: netadr_t, b: netadr_t) -> c_int {
    if a.r#type as u32 != b.r#type as u32 {
        return 0; // qfalse
    }

    if a.r#type == netadrtype_t::NA_LOOPBACK {
        return 1; // qtrue
    }

    if a.r#type == netadrtype_t::NA_IP {
        if a.ip[0] == b.ip[0]
            && a.ip[1] == b.ip[1]
            && a.ip[2] == b.ip[2]
            && a.ip[3] == b.ip[3]
        {
            return 1; // qtrue
        }
        return 0; // qfalse
    }

    #[cfg(not(target_os = "xbox"))]
    {
        if a.r#type == netadrtype_t::NA_IPX {
            if memcmp(
                a.ipx.as_ptr() as *const c_void,
                b.ipx.as_ptr() as *const c_void,
                10,
            ) == 0
            {
                return 1; // qtrue
            }
            return 0; // qfalse
        }
    }

    Com_Printf(b"NET_CompareBaseAdr: bad address type\n\0".as_ptr() as *const c_char);
    return 0; // qfalse
}

pub unsafe fn NET_AdrToString(a: netadr_t) -> *const c_char {
    static mut s: [c_char; 64] = [0; 64];

    if a.r#type == netadrtype_t::NA_LOOPBACK {
        Com_sprintf(
            s.as_mut_ptr(),
            core::mem::size_of_val(&s),
            b"loopback\0".as_ptr() as *const c_char,
        );
    } else if a.r#type == netadrtype_t::NA_BOT {
        Com_sprintf(
            s.as_mut_ptr(),
            core::mem::size_of_val(&s),
            b"bot\0".as_ptr() as *const c_char,
        );
    } else if a.r#type == netadrtype_t::NA_IP {
        Com_sprintf(
            s.as_mut_ptr(),
            core::mem::size_of_val(&s),
            b"%i.%i.%i.%i:%i\0".as_ptr() as *const c_char,
            a.ip[0],
            a.ip[1],
            a.ip[2],
            a.ip[3],
            BigShort(a.port as c_int),
        );
    } else if a.r#type == netadrtype_t::NA_BAD {
        Com_sprintf(
            s.as_mut_ptr(),
            core::mem::size_of_val(&s),
            b"BAD\0".as_ptr() as *const c_char,
        );
    } else {
        Com_sprintf(
            s.as_mut_ptr(),
            core::mem::size_of_val(&s),
            b"%02x%02x%02x%02x.%02x%02x%02x%02x%02x%02x:%i\0".as_ptr() as *const c_char,
            a.ipx[0],
            a.ipx[1],
            a.ipx[2],
            a.ipx[3],
            a.ipx[4],
            a.ipx[5],
            a.ipx[6],
            a.ipx[7],
            a.ipx[8],
            a.ipx[9],
            BigShort(a.port as c_int),
        );
    }

    s.as_ptr()
}

pub unsafe fn NET_CompareAdr(a: netadr_t, b: netadr_t) -> c_int {
    if a.r#type as u32 != b.r#type as u32 {
        return 0; // qfalse
    }

    if a.r#type == netadrtype_t::NA_LOOPBACK {
        return 1; // qtrue
    }

    if a.r#type == netadrtype_t::NA_IP {
        if a.ip[0] == b.ip[0]
            && a.ip[1] == b.ip[1]
            && a.ip[2] == b.ip[2]
            && a.ip[3] == b.ip[3]
            && a.port == b.port
        {
            return 1; // qtrue
        }
        return 0; // qfalse
    }

    #[cfg(not(target_os = "xbox"))]
    {
        if a.r#type == netadrtype_t::NA_IPX {
            if memcmp(
                a.ipx.as_ptr() as *const c_void,
                b.ipx.as_ptr() as *const c_void,
                10,
            ) == 0
                && a.port == b.port
            {
                return 1; // qtrue
            }
            return 0; // qfalse
        }
    }

    Com_Printf(b"NET_CompareAdr: bad address type\n\0".as_ptr() as *const c_char);
    return 0; // qfalse
}

pub unsafe fn NET_IsLocalAddress(adr: netadr_t) -> c_int {
    if adr.r#type == netadrtype_t::NA_LOOPBACK {
        1 // qtrue
    } else {
        0 // qfalse
    }
}

/*
=============================================================================

LOOPBACK BUFFERS FOR LOCAL PLAYER

=============================================================================
*/

// there needs to be enough loopback messages to hold a complete
// gamestate of maximum size

pub unsafe fn NET_GetLoopPacket(
    sock: netsrc_t,
    net_from: *mut netadr_t,
    net_message: *mut msg_t,
) -> c_int {
    let mut i: c_int;
    let mut loop_ptr: *mut loopback_t;

    loop_ptr = &mut loopbacks[sock as usize];

    if (*loop_ptr).send - (*loop_ptr).get > MAX_LOOPBACK as c_int {
        (*loop_ptr).get = (*loop_ptr).send - MAX_LOOPBACK as c_int;
    }

    if (*loop_ptr).get >= (*loop_ptr).send {
        return 0; // qfalse
    }

    i = (*loop_ptr).get & (MAX_LOOPBACK as c_int - 1);
    (*loop_ptr).get += 1;

    Com_Memcpy(
        (*net_message).data as *mut c_void,
        (*loop_ptr).msgs[i as usize].data.as_ptr() as *const c_void,
        (*loop_ptr).msgs[i as usize].datalen as usize,
    );
    (*net_message).cursize = (*loop_ptr).msgs[i as usize].datalen;
    Com_Memset(net_from as *mut c_void, 0, core::mem::size_of::<netadr_t>());
    (*net_from).r#type = netadrtype_t::NA_LOOPBACK;
    return 1; // qtrue
}

pub unsafe fn NET_SendLoopPacket(
    sock: netsrc_t,
    length: c_int,
    data: *const c_void,
    to: netadr_t,
) {
    let mut i: c_int;
    let mut loop_ptr: *mut loopback_t;

    loop_ptr = &mut loopbacks[(sock as usize) ^ 1];

    i = (*loop_ptr).send & (MAX_LOOPBACK as c_int - 1);
    (*loop_ptr).send += 1;

    Com_Memcpy(
        (*loop_ptr).msgs[i as usize].data.as_mut_ptr() as *mut c_void,
        data,
        length as usize,
    );
    (*loop_ptr).msgs[i as usize].datalen = length;
}

//=============================================================================

pub unsafe fn NET_SendPacket(sock: netsrc_t, length: c_int, data: *const c_void, to: netadr_t) {
    // sequenced packets are shown in netchan, so just show oob
    if !showpackets.is_null()
        && (*(*showpackets)).integer != 0
        && *(data as *const c_int) == -1
    {
        Com_Printf(
            b"send packet %4i\n\0".as_ptr() as *const c_char,
            length,
        );
    }

    if to.r#type == netadrtype_t::NA_LOOPBACK {
        NET_SendLoopPacket(sock, length, data, to);
        return;
    }
    if to.r#type == netadrtype_t::NA_BOT {
        return;
    }
    if to.r#type == netadrtype_t::NA_BAD {
        return;
    }

    Sys_SendPacket(length, data, to);
}

/*
===============
NET_OutOfBandPrint

Sends a text message in an out-of-band datagram
================
*/
// NET_OutOfBandPrint is declared as extern "C" in the main extern block above.

/*
===============
NET_OutOfBandData

Sends a data message in an out-of-band datagram (only used for "connect")
================
*/
pub unsafe fn NET_OutOfBandData(sock: netsrc_t, adr: netadr_t, format: *mut byte, len: c_int) {
    let mut string: [byte; MAX_MSGLEN * 2] = [0; MAX_MSGLEN * 2];
    let mut i: c_int;
    let mut mbuf: msg_t = core::mem::zeroed();

    // set the header
    string[0] = 0xff;
    string[1] = 0xff;
    string[2] = 0xff;
    string[3] = 0xff;

    for i in 0..len {
        string[(i + 4) as usize] = *format.add(i as usize);
    }

    mbuf.data = string.as_mut_ptr();
    mbuf.cursize = len + 4;
    Huff_Compress(&mut mbuf, 12);
    // send the datagram
    NET_SendPacket(sock, mbuf.cursize, mbuf.data as *const c_void, adr);
}

/*
=============
NET_StringToAdr

Traps "localhost" for loopback, passes everything else to system
=============
*/
pub unsafe fn NET_StringToAdr(s: *const c_char, a: *mut netadr_t) -> c_int {
    let mut r: c_int;
    let mut base: [c_char; 1024] = [0; 1024]; // MAX_STRING_CHARS = 1024
    let mut port: *mut c_char = core::ptr::null_mut();

    if strcmp(s, b"localhost\0".as_ptr() as *const c_char) == 0 {
        Com_Memset(a as *mut c_void, 0, core::mem::size_of::<netadr_t>());
        (*a).r#type = netadrtype_t::NA_LOOPBACK;
        return 1; // qtrue
    }

    // look for a port number
    Q_strncpyz(
        base.as_mut_ptr(),
        s,
        core::mem::size_of_val(&base),
    );
    port = strstr(base.as_mut_ptr(), b":\0".as_ptr() as *const c_char);
    if !port.is_null() {
        *port = 0;
        port = port.add(1);
    }

    r = Sys_StringToAdr(base.as_ptr(), a);

    if r == 0 {
        (*a).r#type = netadrtype_t::NA_BAD;
        return 0; // qfalse
    }

    // inet_addr returns this if out of range
    if (*a).ip[0] == 255 && (*a).ip[1] == 255 && (*a).ip[2] == 255 && (*a).ip[3] == 255 {
        (*a).r#type = netadrtype_t::NA_BAD;
        return 0; // qfalse
    }

    if !port.is_null() {
        (*a).port = BigShort(atoi_impl(port) as c_int) as u16;
    } else {
        (*a).port = BigShort(27960) as u16; // PORT_SERVER
    }

    return 1; // qtrue
}

// ============================================================================
// Local stubs for libc functions and utilities
// ============================================================================

pub unsafe fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int {
    let s1 = s1 as *const u8;
    let s2 = s2 as *const u8;
    for i in 0..n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return (a as c_int) - (b as c_int);
        }
    }
    0
}

pub unsafe fn atoi_impl(s: *const c_char) -> c_int {
    atoi(s)
}

fn LittleLong(l: c_int) -> c_int {
    // For now, assume little-endian (most common). A proper implementation
    // would check the architecture.
    l
}
