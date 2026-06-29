#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

/*

packet header
-------------
4	outgoing sequence.  high bit will be set if this is a fragmented message
4	acknowledge sequence
[2	qport (only for client to server)]
[2	fragment start byte]
[2	fragment length. if < FRAGMENT_SIZE, this is the last fragment]

if the sequence number is -1, the packet should be handled as an out-of-band
message instead of as part of a netcon.

All fragments will have the same sequence numbers.

The qport field is a workaround for bad address translating routers that
sometimes remap the client's source port on a packet during gameplay.

If the base part of the net address matches and the qport matches, then the
channel matches even if the IP port differs.  The IP port should be updated
to the new value before sending out any replies.

*/


const MAX_PACKETLEN: c_int = 1400;	// max size of a network packet
const MAX_LOOPDATA: c_int = 16 * 1024;

const FRAGMENT_SIZE: c_int = MAX_PACKETLEN - 100;
const PACKET_HEADER: c_int = 10;			// two ints and a short

const FRAGMENT_BIT: c_int = 1<<31;

// External type definitions from qcommon_h
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum netadrtype_t {
    NA_BAD = 0,					// an address lookup failed
    NA_LOOPBACK = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum netsrc_t {
    NS_CLIENT = 0,
    NS_SERVER = 1,
}

#[repr(C)]
pub struct netadr_t {
    pub type_: netadrtype_t,
    pub port: u16,
}

#[repr(C)]
pub struct msg_t {
    pub allowoverflow: c_int,	// if false, do a Com_Error
    pub overflowed: c_int,		// set to true if the buffer size failed (with allowoverflow set)
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,				// for bitwise reads and writes
}

#[repr(C)]
pub struct netchan_t {
    pub sock: netsrc_t,

    pub dropped: c_int,			// between last packet and previous

    pub remoteAddress: netadr_t,
    pub qport: c_int,				// qport value to write when transmitting

    // sequencing variables
    pub incomingSequence: c_int,
    pub incomingAcknowledged: c_int,

    pub outgoingSequence: c_int,

    // incoming fragment assembly buffer
    pub fragmentSequence: c_int,
    pub fragmentLength: c_int,
    pub fragmentBuffer: [u8; 17408],	// MAX_MSGLEN
}

pub type qboolean = c_int;
pub type cvar_t = c_void;

const CVAR_TEMP: c_int = 0;
const CVAR_INIT: c_int = 0;
const TAG_NEWDEL: c_int = 0;

// extern "C" functions and globals
extern "C" {
    pub static mut showpackets: *mut cvar_t;
    pub static mut showdrop: *mut cvar_t;
    pub static mut qport: *mut cvar_t;

    pub fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Z_Malloc(size: usize, tag: c_int, clear: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);

    pub fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int);
    pub fn MSG_WriteLong(buf: *mut msg_t, c: c_int);
    pub fn MSG_WriteShort(buf: *mut msg_t, c: c_int);
    pub fn MSG_WriteData(buf: *mut msg_t, data: *const c_void, length: c_int);
    pub fn MSG_BeginReading(msg: *mut msg_t);
    pub fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    pub fn MSG_ReadShort(msg: *mut msg_t) -> c_int;

    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    pub fn va(fmt: *const c_char, ...) -> *const c_char;

    pub fn NET_SendPacket(sock: netsrc_t, length: c_int, data: *const c_void, to: netadr_t);
    pub fn NET_AdrToString(a: netadr_t) -> *const c_char;
}

static netsrcString: [&[u8]; 2] = [
    b"client",
    b"server"
];

#[repr(C)]
struct loopback_t {
    loopData: [c_char; 16384],		// MAX_LOOPDATA
    get: c_int,
    send: c_int,
}

static mut loopbacks: *mut loopback_t = core::ptr::null_mut();


/*
===============
Netchan_Init

===============
*/
pub fn Netchan_Init(port: c_int) {
    unsafe {
        if loopbacks.is_null()
        {
            loopbacks = Z_Malloc(core::mem::size_of::<loopback_t>() * 2, TAG_NEWDEL, 1) as *mut loopback_t;
        }

        let port = port & 0xffff;
        showpackets = Cvar_Get(b"showpackets\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_TEMP );
        showdrop = Cvar_Get(b"showdrop\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_TEMP );
        qport = Cvar_Get(b"qport\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, port), CVAR_INIT );
    }
}

pub fn Netchan_Shutdown()
{
    unsafe {
        if !loopbacks.is_null()
        {
            Z_Free(loopbacks as *mut c_void);
            loopbacks = core::ptr::null_mut();
        }
    }
}

/*
==============
Netchan_Setup

called to open a channel to a remote system
==============
*/
pub fn Netchan_Setup(sock: netsrc_t, chan: *mut netchan_t, adr: netadr_t, qport_val: c_int) {
    unsafe {
        core::ptr::write_bytes(chan, 0, 1);

        (*chan).sock = sock;
        (*chan).remoteAddress = adr;
        (*chan).qport = qport_val;
        (*chan).incomingSequence = 0;
        (*chan).outgoingSequence = 1;
    }
}

/*
===============
Netchan_Transmit

Sends a message to a connection, fragmenting if necessary
A 0 length will still generate a packet.
================
*/
pub fn Netchan_Transmit(chan: *mut netchan_t, length: c_int, data: *const u8) {
    unsafe {
        let mut send: msg_t = core::mem::zeroed();
        let mut send_buf: [u8; 1400] = [0; 1400];
        let mut fragmentStart: c_int;
        let mut fragmentLength: c_int;

        fragmentStart = 0;		// stop warning message
        fragmentLength = 0;

        // fragment large reliable messages
        if length >= FRAGMENT_SIZE {
            fragmentStart = 0;
            loop {
                // write the packet header
                MSG_Init(&mut send, send_buf.as_mut_ptr(), send_buf.len() as c_int);

                MSG_WriteLong( &mut send, (*chan).outgoingSequence | FRAGMENT_BIT );
                MSG_WriteLong( &mut send, (*chan).incomingSequence );

                // send the qport if we are a client
                if (*chan).sock as c_int == netsrc_t::NS_CLIENT as c_int {
                    MSG_WriteShort( &mut send, *(qport as *const c_int) );
                }

                // copy the reliable message to the packet first
                fragmentLength = FRAGMENT_SIZE;
                if fragmentStart + fragmentLength > length {
                    fragmentLength = length - fragmentStart;
                }

                MSG_WriteShort( &mut send, fragmentStart );
                MSG_WriteShort( &mut send, fragmentLength );
                MSG_WriteData( &mut send, data.offset(fragmentStart as isize) as *const c_void, fragmentLength );

                // send the datagram
                NET_SendPacket( (*chan).sock, send.cursize, send_buf.as_ptr() as *const c_void, (*chan).remoteAddress );

                if !showpackets.is_null() && (*(showpackets as *mut c_int)) != 0 {
                    Com_Printf(b"%s send %4i : s=%i ack=%i fragment=%i,%i\n\0".as_ptr() as *const c_char,
                        netsrcString[ (*chan).sock as usize ].as_ptr() as *const c_char,
                        send.cursize,
                        (*chan).outgoingSequence - 1,
                        (*chan).incomingSequence,
                        fragmentStart, fragmentLength);
                }

                fragmentStart += fragmentLength;
                // this exit condition is a little tricky, because a packet
                // that is exactly the fragment length still needs to send
                // a second packet of zero length so that the other side
                // can tell there aren't more to follow
                if fragmentStart == length && fragmentLength != FRAGMENT_SIZE {
                    break;
                }
            }

            (*chan).outgoingSequence += 1;
            return;
        }

        // write the packet header
        MSG_Init(&mut send, send_buf.as_mut_ptr(), send_buf.len() as c_int);

        MSG_WriteLong( &mut send, (*chan).outgoingSequence );
        MSG_WriteLong( &mut send, (*chan).incomingSequence );
        (*chan).outgoingSequence += 1;

        // send the qport if we are a client
        if (*chan).sock as c_int == netsrc_t::NS_CLIENT as c_int {
            MSG_WriteShort( &mut send, *(qport as *const c_int) );
        }

        MSG_WriteData( &mut send, data as *const c_void, length );

        // send the datagram
        NET_SendPacket( (*chan).sock, send.cursize, send_buf.as_ptr() as *const c_void, (*chan).remoteAddress );

        if !showpackets.is_null() && (*(showpackets as *mut c_int)) != 0 {
            Com_Printf(b"%s send %4i : s=%i ack=%i\n\0".as_ptr() as *const c_char,
                netsrcString[ (*chan).sock as usize ].as_ptr() as *const c_char,
                send.cursize,
                (*chan).outgoingSequence - 1,
                (*chan).incomingSequence );
        }
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
pub fn Netchan_Process(chan: *mut netchan_t, msg: *mut msg_t) -> qboolean {
    unsafe {
        let mut sequence: c_int;
        let mut sequence_ack: c_int;
        let mut qport_val: c_int;
        let mut fragmentStart: c_int;
        let mut fragmentLength: c_int;
        let mut fragmented: qboolean;

        // get sequence numbers
        MSG_BeginReading( msg );
        sequence = MSG_ReadLong( msg );
        sequence_ack = MSG_ReadLong( msg );

        // check for fragment information
        if (sequence & FRAGMENT_BIT) != 0 {
            sequence &= !FRAGMENT_BIT;
            fragmented = 1;		// qtrue
        } else {
            fragmented = 0;		// qfalse
        }

        // read the qport if we are a server
        if (*chan).sock as c_int == netsrc_t::NS_SERVER as c_int {
            qport_val = MSG_ReadShort( msg );
        }

        // read the fragment information
        if fragmented != 0 {
            fragmentStart = MSG_ReadShort( msg );
            fragmentLength = MSG_ReadShort( msg );
        } else {
            fragmentStart = 0;		// stop warning message
            fragmentLength = 0;
        }

        if !showpackets.is_null() && (*(showpackets as *mut c_int)) != 0 {
            if fragmented != 0 {
                Com_Printf(b"%s recv %4i : s=%i ack=%i fragment=%i,%i\n\0".as_ptr() as *const c_char,
                    netsrcString[ (*chan).sock as usize ].as_ptr() as *const c_char,
                    (*msg).cursize,
                    sequence,
                    sequence_ack,
                    fragmentStart, fragmentLength );
            } else {
                Com_Printf(b"%s recv %4i : s=%i ack=%i\n\0".as_ptr() as *const c_char,
                    netsrcString[ (*chan).sock as usize ].as_ptr() as *const c_char,
                    (*msg).cursize,
                    sequence,
                    sequence_ack );
            }
        }

        //
        // discard out of order or duplicated packets
        //
        if sequence <= (*chan).incomingSequence {
            if !showdrop.is_null() && (*(showdrop as *mut c_int)) != 0
                || !showpackets.is_null() && (*(showpackets as *mut c_int)) != 0 {
                Com_Printf(b"%s:Out of order packet %i at %i\n\0".as_ptr() as *const c_char,
                    NET_AdrToString( (*chan).remoteAddress ),
                    sequence,
                    (*chan).incomingSequence );
            }
            return 0;		// qfalse
        }

        //
        // dropped packets don't keep the message from being used
        //
        (*chan).dropped = sequence - ((*chan).incomingSequence+1);
        if (*chan).dropped > 0 {
            if !showdrop.is_null() && (*(showdrop as *mut c_int)) != 0
                || !showpackets.is_null() && (*(showpackets as *mut c_int)) != 0 {
                Com_Printf(b"%s:Dropped %i packets at %i\n\0".as_ptr() as *const c_char,
                    NET_AdrToString( (*chan).remoteAddress ),
                    (*chan).dropped,
                    sequence );
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
                if !showdrop.is_null() && (*(showdrop as *mut c_int)) != 0
                    || !showpackets.is_null() && (*(showpackets as *mut c_int)) != 0 {
                    Com_Printf(b"%s:Dropped a message fragment\n\0".as_ptr() as *const c_char,
                        NET_AdrToString( (*chan).remoteAddress ) );
                }
                // we can still keep the part that we have so far,
                // so we don't need to clear chan->fragmentLength
                return 0;		// qfalse
            }

            // copy the fragment to the fragment buffer
            if fragmentLength < 0 || (*msg).readcount + fragmentLength > (*msg).cursize ||
                (*chan).fragmentLength + fragmentLength > (*chan).fragmentBuffer.len() as c_int {
                if !showdrop.is_null() && (*(showdrop as *mut c_int)) != 0
                    || !showpackets.is_null() && (*(showpackets as *mut c_int)) != 0 {
                    Com_Printf(b"%s:illegal fragment length\n\0".as_ptr() as *const c_char,
                        NET_AdrToString ((*chan).remoteAddress ) );
                }
                return 0;		// qfalse
            }

            core::ptr::copy_nonoverlapping(
                ((*msg).data).offset((*msg).readcount as isize),
                ((*chan).fragmentBuffer.as_mut_ptr() as *mut u8).offset((*chan).fragmentLength as isize),
                fragmentLength as usize );

            (*chan).fragmentLength += fragmentLength;

            // if this wasn't the last fragment, don't process anything
            if fragmentLength == FRAGMENT_SIZE {
                return 0;		// qfalse
            }

            if (*chan).fragmentLength > (*msg).maxsize {
                Com_Printf(b"%s:fragmentLength %i > msg->maxsize\n\0".as_ptr() as *const c_char,
                    NET_AdrToString ((*chan).remoteAddress ),
                    (*chan).fragmentLength );
                return 0;		// qfalse
            }

            // copy the full message over the partial fragment

            // make sure the sequence number is still there
            *((*msg).data as *mut c_int) = sequence;

            core::ptr::copy_nonoverlapping(
                (*chan).fragmentBuffer.as_ptr() as *const u8,
                ((*msg).data).offset(4),
                (*chan).fragmentLength as usize );
            (*msg).cursize = (*chan).fragmentLength + 4;
            (*chan).fragmentLength = 0;
            (*msg).readcount = 4;	// past the sequence number

            return 1;		// qtrue
        }

        //
        // the message can now be read from the current message pointer
        //
        (*chan).incomingSequence = sequence;
        (*chan).incomingAcknowledged = sequence_ack;

        return 1;		// qtrue
    }
}


//==============================================================================

/*
===================
NET_CompareBaseAdr

Compares without the port
===================
*/
pub fn NET_CompareBaseAdr(a: netadr_t, b: netadr_t) -> qboolean
{
    if a.type_ as c_int != b.type_ as c_int {
        return 0;	// qfalse
    }

    if a.type_ as c_int == netadrtype_t::NA_LOOPBACK as c_int {
        return 1;	// qtrue
    }

    unsafe {
        Com_Printf(b"NET_CompareBaseAdr: bad address type\n\0".as_ptr() as *const c_char);
    }
    0	// qfalse
}

pub fn NET_AdrToString(a: netadr_t) -> *const c_char
{
    static mut s: [c_char; 64] = [0; 64];

    unsafe {
        if a.type_ as c_int == netadrtype_t::NA_LOOPBACK as c_int {
            Com_sprintf(s.as_mut_ptr(), core::mem::size_of_val(&s), b"loopback\0".as_ptr() as *const c_char);
        }

        s.as_ptr()
    }
}


pub fn NET_CompareAdr(a: netadr_t, b: netadr_t) -> qboolean
{
    if a.type_ as c_int != b.type_ as c_int {
        return 0;	// qfalse
    }

    if a.type_ as c_int == netadrtype_t::NA_LOOPBACK as c_int {
        return 1;	// qtrue
    }

    unsafe {
        Com_Printf(b"NET_CompareAdr: bad address type\n\0".as_ptr() as *const c_char);
    }
    0	// qfalse
}


pub fn NET_IsLocalAddress(adr: netadr_t) -> qboolean {
    if adr.type_ as c_int == netadrtype_t::NA_LOOPBACK as c_int {
        return 1;	// qtrue
    }
    0	// qfalse
}



/*
=============================================================================

LOOPBACK BUFFERS FOR LOCAL PLAYER

=============================================================================
*/

pub fn NET_GetLoopPacket(sock: netsrc_t, net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean
{
    unsafe {
        let mut i: c_int;
        let mut r#loop: *mut loopback_t;

        r#loop = &mut *loopbacks.offset(sock as c_int as isize);

        //If read and write positions are the same, nothing left to read.
        if (*r#loop).get == (*r#loop).send {
            return 0;	// qfalse
        }

        //Get read position.  Wrap if too close to end.
        i = (*r#loop).get;
        if i > MAX_LOOPDATA - 4 {
            i = 0;
        }

        //Get length of packet.
        let length: c_int = *((*r#loop).loopData.as_ptr().offset(i as isize) as *const c_int);
        i += 4;

        //See if entire packet is at end of buffer or part is at the beginning.
        if i + length <= MAX_LOOPDATA {
            //Everything fits, full copy.
            core::ptr::copy_nonoverlapping(
                ((*r#loop).loopData.as_ptr() as *const u8).offset(i as isize),
                (*net_message).data,
                length as usize
            );
            (*net_message).cursize = length;
            i += length;
            (*r#loop).get = i;
        } else {
            //Doesn't all fit, partial copy
            let copyToEnd: c_int = MAX_LOOPDATA - i;
            core::ptr::copy_nonoverlapping(
                ((*r#loop).loopData.as_ptr() as *const u8).offset(i as isize),
                (*net_message).data,
                copyToEnd as usize
            );
            core::ptr::copy_nonoverlapping(
                (*r#loop).loopData.as_ptr() as *const u8,
                ((*net_message).data as *mut u8).offset(copyToEnd as isize),
                (length - copyToEnd) as usize
            );
            (*net_message).cursize = length;
            (*r#loop).get = length - copyToEnd;
        }

        core::ptr::write_bytes(net_from, 0, 1);
        (*net_from).type_ = netadrtype_t::NA_LOOPBACK;

        return 1;	// qtrue
    }
}


pub fn NET_SendLoopPacket(sock: netsrc_t, length: c_int, data: *const c_void, to: netadr_t)
{
    unsafe {
        let mut i: c_int;
        let mut r#loop: *mut loopback_t;

        r#loop = &mut *loopbacks.offset(((sock as c_int) ^ 1) as isize);

        //Make sure there is enough free space in the buffer.
        let freeSpace: c_int;
        if (*r#loop).send >= (*r#loop).get {
            freeSpace = MAX_LOOPDATA - ((*r#loop).send - (*r#loop).get);
        } else {
            freeSpace = (*r#loop).get - (*r#loop).send;
        }

        assert!(freeSpace > length);

        //Get write position.  Wrap around if too close to end.
        i = (*r#loop).send;
        if i > MAX_LOOPDATA - 4 {
            i = 0;
        }

        //Write length of packet.
        *((*r#loop).loopData.as_mut_ptr().offset(i as isize) as *mut c_int) = length;
        i += 4;

        //See if the whole packet will fit on the end of the buffer or if we
        //need to write part of it back at the beginning.
        if i + length <= MAX_LOOPDATA {
            //Everything fits, full copy.
            core::ptr::copy_nonoverlapping(
                data as *const u8,
                ((*r#loop).loopData.as_mut_ptr() as *mut u8).offset(i as isize),
                length as usize
            );
            i += length;
            (*r#loop).send = i;
        } else {
            //Doesn't all fit, partial copy
            let copyToEnd: c_int = MAX_LOOPDATA - i;
            core::ptr::copy_nonoverlapping(
                data as *const u8,
                ((*r#loop).loopData.as_mut_ptr() as *mut u8).offset(i as isize),
                copyToEnd as usize
            );
            core::ptr::copy_nonoverlapping(
                (data as *const u8).offset(copyToEnd as isize),
                (*r#loop).loopData.as_mut_ptr(),
                (length - copyToEnd) as usize
            );
            (*r#loop).send = length - copyToEnd;
        }
    }
}

//=============================================================================


pub fn NET_SendPacket(sock: netsrc_t, length: c_int, data: *const c_void, to: netadr_t) {
    unsafe {
        // sequenced packets are shown in netchan, so just show oob
        if !showpackets.is_null() && *(data as *const c_int) == -1 {
            Com_Printf(b"send packet %4i\n\0".as_ptr() as *const c_char, length);
        }

        if to.type_ as c_int == netadrtype_t::NA_LOOPBACK as c_int {
            NET_SendLoopPacket(sock, length, data, to);
            return;
        }
    }
}

/*
===============
NET_OutOfBandPrint

Sends a text message in an out-of-band datagram
================
*/
pub fn NET_OutOfBandPrint(sock: netsrc_t, adr: netadr_t, format: *const c_char, ...) {
    unsafe {
        let mut string: [c_char; 1400] = [0; 1400];

        // set the header
        string[0] = 0xff as c_char;
        string[1] = 0xff as c_char;
        string[2] = 0xff as c_char;
        string[3] = 0xff as c_char;

        // va_start/vsprintf would go here in real implementation
        // vsprintf( string.as_mut_ptr().offset(4), format, argptr );

        // send the datagram
        // NET_SendPacket( sock, strlen( string ), string, adr );
    }
}



/*
=============
NET_StringToAdr

Traps "localhost" for loopback, passes everything else to system
=============
*/
pub fn NET_StringToAdr(s: *const c_char, a: *mut netadr_t) -> qboolean {
    unsafe {
        // strcmp check for "localhost"
        // if (!strcmp (s, "localhost")) {
        //     memset (a, 0, sizeof(*a));
        //     (*a).type_ = netadrtype_t::NA_LOOPBACK;
        //     return 1;  // qtrue
        // }

        (*a).type_ = netadrtype_t::NA_BAD;
        0	// qfalse
    }
}
