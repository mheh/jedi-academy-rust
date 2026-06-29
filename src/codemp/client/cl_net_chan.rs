#![allow(non_snake_case)]

use core::ffi::c_int;

// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "client.h"

// ==================== Type Stubs ====================
// Types used from the original headers

pub type qboolean = c_int;
pub type byte = u8;

#[repr(C)]
pub struct msg_t {
    pub data: *mut c_int,  // will be cast to *mut u8 when used
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
    pub oob: qboolean,
    // other fields omitted
}

#[repr(C)]
pub struct netchan_t {
    // opaque structure - only pass pointers to functions
}

#[repr(C)]
pub struct client_t {
    pub serverCommands: [*const u8; 64],  // MAX_RELIABLE_COMMANDS
    pub reliableCommands: [*const u8; 64],  // MAX_RELIABLE_COMMANDS
    pub challenge: c_int,
    // other fields omitted
}

// ==================== External Functions ====================

extern "C" {
    fn MSG_ReadLong(msg: *mut msg_t) -> c_int;
    fn MSG_WriteByte(msg: *mut msg_t, c: c_int);
    fn Netchan_TransmitNextFragment(chan: *mut netchan_t);
    fn Netchan_Transmit(chan: *mut netchan_t, length: c_int, data: *const u8);
    fn Netchan_Process(chan: *mut netchan_t, msg: *mut msg_t) -> c_int;
    fn LittleLong(l: core::ffi::c_uint) -> core::ffi::c_uint;
}

// ==================== Global Variables ====================

pub static mut clc: client_t = client_t {
    serverCommands: [core::ptr::null(); 64],
    reliableCommands: [core::ptr::null(); 64],
    challenge: 0,
};

pub static mut oldsize: c_int = 0;
pub static mut newsize: c_int = 0;

// ==================== Constants ====================

const MAX_RELIABLE_COMMANDS: c_int = 64;
const CL_ENCODE_START: c_int = 12;
const CL_DECODE_START: c_int = 4;
const clc_EOF: c_int = 248;  // svc_EOF equivalent for client

// TTimo: unused, commenting out to make gcc happy
/*
==============
CL_Netchan_Encode

    // first 12 bytes of the data are always:
    long serverId;
    long messageAcknowledge;
    long reliableAcknowledge;

==============
*/
unsafe fn CL_Netchan_Encode(msg: *mut msg_t) {
    #[cfg(target_os = "xbox")]
    return;

    let mut serverId: c_int;
    let mut messageAcknowledge: c_int;
    let mut reliableAcknowledge: c_int;
    let mut i: c_int;
    let mut index: c_int;
    let mut srdc: c_int;
    let mut sbit: c_int;
    let mut soob: c_int;
    let mut key: byte;
    let mut string: *mut byte;

    if (*msg).cursize <= CL_ENCODE_START {
        return;
    }

    srdc = (*msg).readcount;
    sbit = (*msg).bit;
    soob = (*msg).oob;

    (*msg).bit = 0;
    (*msg).readcount = 0;
    (*msg).oob = 0;

    serverId = MSG_ReadLong(msg);
    messageAcknowledge = MSG_ReadLong(msg);
    reliableAcknowledge = MSG_ReadLong(msg);

    (*msg).oob = soob;
    (*msg).bit = sbit;
    (*msg).readcount = srdc;

    string = (clc.serverCommands[(reliableAcknowledge & (MAX_RELIABLE_COMMANDS-1)) as usize] as *mut byte);
    index = 0;
    //
    key = (clc.challenge ^ serverId ^ messageAcknowledge) as byte;
    i = CL_ENCODE_START;
    while i < (*msg).cursize {
        // modify the key with the last received now acknowledged server command
        if (*string.add(index as usize)) == 0 {
            index = 0;
        }
        if (/*(*string.add(index as usize)) > 127 || */	// eurofix: remove this so we can chat in european languages...	-ste
            (*string.add(index as usize)) == b'%')
        {
            key ^= ((b'.' as c_int) << (i & 1)) as byte;
        }
        else {
            key ^= (*string.add(index as usize)) << (i & 1);
        }
        index += 1;
        // encode the data with this key
        let msg_data = (*msg).data as *mut byte;
        *msg_data.add(i as usize) = (*msg_data.add(i as usize)) ^ key;
        i += 1;
    }
}

/*
==============
CL_Netchan_Decode

    // first four bytes of the data are always:
    long reliableAcknowledge;

==============
*/
unsafe fn CL_Netchan_Decode(msg: *mut msg_t) {
    #[cfg(target_os = "xbox")]
    return;

    let mut reliableAcknowledge: c_int;
    let mut i: c_int;
    let mut index: c_int;
    let mut key: byte;
    let mut string: *mut byte;
    let mut srdc: c_int;
    let mut sbit: c_int;
    let mut soob: c_int;

    srdc = (*msg).readcount;
    sbit = (*msg).bit;
    soob = (*msg).oob;

    (*msg).oob = 0;

    reliableAcknowledge = MSG_ReadLong(msg);

    (*msg).oob = soob;
    (*msg).bit = sbit;
    (*msg).readcount = srdc;

    string = (clc.reliableCommands[(reliableAcknowledge & (MAX_RELIABLE_COMMANDS-1)) as usize] as *mut byte);
    index = 0;
    // xor the client challenge with the netchan sequence number (need something that changes every message)
    key = (clc.challenge ^ LittleLong(*((*msg).data as *const core::ffi::c_uint))) as byte;
    i = (*msg).readcount + CL_DECODE_START;
    while i < (*msg).cursize {
        // modify the key with the last sent and with this message acknowledged client command
        if (*string.add(index as usize)) == 0 {
            index = 0;
        }
        if (/*(*string.add(index as usize)) > 127 || */	// eurofix: remove this so we can chat in european languages...	-ste
            (*string.add(index as usize)) == b'%')
        {
            key ^= ((b'.' as c_int) << (i & 1)) as byte;
        }
        else {
            key ^= (*string.add(index as usize)) << (i & 1);
        }
        index += 1;
        // decode the data with this key
        let msg_data = (*msg).data as *mut byte;
        *msg_data.add(i as usize) = *msg_data.add(i as usize) ^ key;
        i += 1;
    }
}

/*
=================
CL_Netchan_TransmitNextFragment
=================
*/
pub unsafe fn CL_Netchan_TransmitNextFragment(chan: *mut netchan_t) {
    Netchan_TransmitNextFragment(chan);
}

//byte chksum[65536];

/*
===============
CL_Netchan_Transmit
================
*/
pub unsafe fn CL_Netchan_Transmit(chan: *mut netchan_t, msg: *mut msg_t) {
    //	int i;
    MSG_WriteByte(msg, clc_EOF);
    //	for(i=CL_ENCODE_START;i<msg->cursize;i++) {
    //		chksum[i-CL_ENCODE_START] = msg->data[i];
    //	}

    //	Huff_Compress( msg, CL_ENCODE_START );
    CL_Netchan_Encode(msg);
    Netchan_Transmit(chan, (*msg).cursize, (*msg).data as *const byte);
}

/*
=================
CL_Netchan_Process
=================
*/
pub unsafe fn CL_Netchan_Process(chan: *mut netchan_t, msg: *mut msg_t) -> qboolean {
    let mut ret: c_int;
    //	int i;
    //	static		int newsize = 0;

    ret = Netchan_Process(chan, msg);
    if ret == 0 {
        return 0;  // qfalse
    }
    CL_Netchan_Decode(msg);
    //	Huff_Decompress( msg, CL_DECODE_START );
    //	for(i=CL_DECODE_START+msg->readcount;i<msg->cursize;i++) {
    //		if (msg->data[i] != chksum[i-(CL_DECODE_START+msg->readcount)]) {
    //			Com_Error(ERR_DROP,"bad %d v %d\n", msg->data[i], chksum[i-(CL_DECODE_START+msg->readcount)]);
    //		}
    //	}
    newsize += (*msg).cursize;
    //	Com_Printf("saved %d to %d (%d%%)\n", (oldsize>>3), newsize, 100-(newsize*100/(oldsize>>3)));
    return 1;  // qtrue
}
