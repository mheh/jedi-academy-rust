#![allow(non_snake_case)]

use core::ffi::c_int;

pub type byte = u8;
pub type qboolean = c_int;

// External C types - stub declarations for this file
// These types are actually defined in server.h but we declare them as opaque here
#[repr(C)]
pub struct netchan_t {
    pub outgoingSequence: c_int,
    _private: [u8; 0],
}

#[repr(C)]
pub struct msg_t {
    pub data: *mut byte,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
    pub oob: qboolean,
    _private: [u8; 0],
}

#[repr(C)]
pub struct client_t {
    pub netchan: netchan_t,
    pub challenge: c_int,
    pub lastClientCommandString: *mut byte,
    pub reliableCommands: [*mut byte; 64],  // MAX_RELIABLE_COMMANDS
    _private: [u8; 0],
}

// External functions from other modules
extern "C" {
    fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    fn MSG_WriteByte(msg: *mut msg_t, byte: byte);
    fn Netchan_TransmitNextFragment(chan: *mut netchan_t);
    fn Netchan_Transmit(chan: *mut netchan_t, length: c_int, data: *const byte);
    fn Netchan_Process(chan: *mut netchan_t, msg: *mut msg_t) -> qboolean;
}

// Constants
const SV_ENCODE_START: usize = 4;
const SV_DECODE_START: usize = 12;
const MAX_RELIABLE_COMMANDS: usize = 64;
const svc_EOF: byte = 0;

// TTimo: unused, commenting out to make gcc happy
#[allow(dead_code)]
unsafe fn SV_Netchan_Encode(client: *mut client_t, msg: *mut msg_t) {
    #[cfg(target_os = "xbox")]
    return;

    let mut reliableAcknowledge: c_int = 0;
    let mut i: c_int = 0;
    let mut index: c_int = 0;
    let mut key: byte = 0;
    let mut string: *mut byte = core::ptr::null_mut();
    let srdc: c_int;
    let sbit: c_int;
    let soob: qboolean;

    if (*msg).cursize < SV_ENCODE_START as c_int {
        return;
    }

    srdc = (*msg).readcount;
    sbit = (*msg).bit;
    soob = (*msg).oob;

    (*msg).bit = 0;
    (*msg).readcount = 0;
    (*msg).oob = 0;

    reliableAcknowledge = MSG_ReadLong(msg);

    (*msg).oob = soob;
    (*msg).bit = sbit;
    (*msg).readcount = srdc;

    string = (*client).lastClientCommandString;
    index = 0;
    // xor the client challenge with the netchan sequence number
    key = ((*client).challenge as u8) ^ ((*client).netchan.outgoingSequence as u8);
    i = SV_ENCODE_START as c_int;
    while i < (*msg).cursize {
        // modify the key with the last received and with this message acknowledged client command
        if *string.offset(index as isize) == 0 {
            index = 0;
        }
        if /*string[index] > 127 ||*/	// eurofix: remove this so we can chat in european languages...	-ste
           *string.offset(index as isize) == b'%' as u8
        {
            key ^= (b'.' as u8) << (i as u32 & 1);
        } else {
            key ^= *string.offset(index as isize) << (i as u32 & 1);
        }
        index += 1;
        // encode the data with this key
        *(*msg).data.offset(i as isize) ^= key;
        i += 1;
    }
}

/*
==============
SV_Netchan_Encode

    // first four bytes of the data are always:
    long reliableAcknowledge;

==============
*/

/*
==============
SV_Netchan_Decode

    // first 12 bytes of the data are always:
    long serverId;
    long messageAcknowledge;
    long reliableAcknowledge;

==============
*/
#[allow(dead_code)]
unsafe fn SV_Netchan_Decode(client: *mut client_t, msg: *mut msg_t) {
    #[cfg(target_os = "xbox")]
    return;

    let mut serverId: c_int = 0;
    let mut messageAcknowledge: c_int = 0;
    let mut reliableAcknowledge: c_int = 0;
    let mut i: c_int = 0;
    let mut index: c_int = 0;
    let srdc: c_int;
    let sbit: c_int;
    let soob: qboolean;
    let mut key: byte = 0;
    let mut string: *mut byte = core::ptr::null_mut();

    srdc = (*msg).readcount;
    sbit = (*msg).bit;
    soob = (*msg).oob;

    (*msg).oob = 0;

    serverId = MSG_ReadLong(msg);
    messageAcknowledge = MSG_ReadLong(msg);
    reliableAcknowledge = MSG_ReadLong(msg);

    (*msg).oob = soob;
    (*msg).bit = sbit;
    (*msg).readcount = srdc;

    string = (*client).reliableCommands[(reliableAcknowledge as usize) & (MAX_RELIABLE_COMMANDS - 1)];
    index = 0;
    //
    key = ((*client).challenge as u8) ^ (serverId as u8) ^ (messageAcknowledge as u8);
    i = (*msg).readcount + SV_DECODE_START as c_int;
    while i < (*msg).cursize {
        // modify the key with the last sent and acknowledged server command
        if *string.offset(index as isize) == 0 {
            index = 0;
        }
        if /*string[index] > 127 || */	// eurofix: remove this so we can chat in european languages...	-ste
           *string.offset(index as isize) == b'%' as u8
        {
            key ^= (b'.' as u8) << (i as u32 & 1);
        } else {
            key ^= *string.offset(index as isize) << (i as u32 & 1);
        }
        index += 1;
        // decode the data with this key
        *(*msg).data.offset(i as isize) ^= key;
        i += 1;
    }
}

/*
=================
SV_Netchan_TransmitNextFragment
=================
*/
pub unsafe fn SV_Netchan_TransmitNextFragment(chan: *mut netchan_t) {
    Netchan_TransmitNextFragment(chan);
}

/*
===============
SV_Netchan_Transmit
================
*/

//extern byte chksum[65536];
pub unsafe fn SV_Netchan_Transmit(client: *mut client_t, msg: *mut msg_t) {	//int length, const byte *data ) {
    //	int i;
    MSG_WriteByte(msg, svc_EOF);
    //	for(i=SV_ENCODE_START;i<msg->cursize;i++) {
    //		chksum[i-SV_ENCODE_START] = msg->data[i];
    //	}
    //	Huff_Compress( msg, SV_ENCODE_START );
    SV_Netchan_Encode(client, msg);
    Netchan_Transmit(&mut (*client).netchan, (*msg).cursize, (*msg).data);
}

/*
=================
Netchan_SV_Process
=================
*/
pub unsafe fn SV_Netchan_Process(client: *mut client_t, msg: *mut msg_t) -> qboolean {
    let mut ret: qboolean = 0;
    //	int i;
    ret = Netchan_Process(&mut (*client).netchan, msg);
    if ret == 0 {
        return 0;  // qfalse
    }
    SV_Netchan_Decode(client, msg);
    //	Huff_Decompress( msg, SV_DECODE_START );
    //	for(i=SV_DECODE_START+msg->readcount;i<msg->cursize;i++) {
    //		if (msg->data[i] != chksum[i-(SV_DECODE_START+msg->readcount)]) {
    //			Com_Error(ERR_DROP,"bad\n");
    //		}
    //	}
    return 1;  // qtrue
}
