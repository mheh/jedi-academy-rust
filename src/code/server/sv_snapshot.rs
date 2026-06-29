// leave this as first line for PCH reasons...
//
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use crate::code::server::server_h::{
    svs, sv, ge, client_s, clientSnapshot_t, svEntity_s, gentity_t,
    MAX_GENTITIES, MAX_RELIABLE_COMMANDS, PACKET_MASK, MAX_MAP_AREA_BYTES,
};
use crate::code::qcommon::qcommon_h::{
    msg_t, MSG_WriteByte, MSG_WriteLong, MSG_WriteString, MSG_WriteBits,
    MSG_WriteData, MSG_WriteDeltaPlayerstate,
};
use crate::code::qcommon::cm_public_h::CM_WriteAreaBits;

extern "C" {
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Netchan_Transmit(chan: *mut c_void, length: c_int, data: *const u8);
    pub fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int);
    pub fn MSG_Clear(buf: *mut msg_t);
    pub fn MSG_WriteEntity(msg: *mut msg_t, to: *mut c_void, removeNum: c_int);
    pub fn SV_GentityNum(num: c_int) -> *mut gentity_t;
    pub fn SV_SvEntityForGentity(gEnt: *mut gentity_t) -> *mut svEntity_s;
    pub fn CM_PointLeafnum(origin: *const f32) -> c_int;
    pub fn CM_LeafArea(leafnum: c_int) -> c_int;
    pub fn CM_LeafCluster(leafnum: c_int) -> c_int;
    pub fn CM_ClusterPVS(cluster: c_int) -> *const u8;
    pub fn CM_AreasConnected(area1: c_int, area2: c_int) -> c_int;
    pub fn VectorSubtract(veca: *const f32, vecb: *const f32, out: *mut f32);
    pub fn VectorNormalize(v: *mut f32) -> f32;
    pub fn VectorCopy(src: *const f32, dest: *mut f32);
    pub fn VectorMA(veca: *const f32, scale: f32, vecb: *const f32, out: *mut f32);
    pub fn AngleVectors(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32);
    pub fn DotProduct(v1: *const f32, v2: *const f32) -> f32;
    pub fn VM_Call(vm: *mut c_void, callnum: c_int, ...) -> i64;
    pub fn qsort(
        base: *mut c_void,
        nmemb: usize,
        size: usize,
        compar: Option<extern "C" fn(*const c_void, *const c_void) -> c_int>,
    );
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

//
// Constants
//
const MAX_SNAPSHOT_ENTITIES: usize = 1024;
const GENTITYNUM_BITS: c_int = 11;
const HEADER_RATE_BYTES: c_int = 48;
const EF_PERMANENT: c_int = 0x00080000;
const EF_FORCE_VISIBLE: c_int = 0x00000200;
const FP_SEE: c_int = 13;

// From server.h constants
const CS_ACTIVE: c_int = 4;
const NA_LOOPBACK: c_int = 1;
const svc_snapshot: u8 = 7;
const svc_serverCommand: u8 = 5;

// Camera related VM call numbers from vmachine.h
const CG_CAMERA_POS: c_int = 40;
const CG_CAMERA_ANG: c_int = 41;

// Structure to hold entity numbers for a snapshot
#[repr(C)]
struct snapshotEntityNumbers_t {
    numSnapshotEntities: c_int,
    snapshotEntities: [c_int; MAX_SNAPSHOT_ENTITIES],
}

// Local struct definitions to access opaque type fields
#[repr(C)]
pub struct local_netchan_t {
    pub sock: c_int,
    pub dropped: c_int,
    pub remoteAddress: [u8; 6],
    pub qport: c_int,
    pub incomingSequence: c_int,
    pub incomingAcknowledged: c_int,
    pub outgoingSequence: c_int,
    pub fragmentSequence: c_int,
    pub fragmentLength: c_int,
    pub fragmentBuffer: [u8; 17408],
}

#[repr(C)]
pub struct local_msg_t {
    pub allowoverflow: c_int,
    pub overflowed: c_int,
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

// Force sight level range table
static mut sv_sightRangeForLevel: [f32; 6] = [
    0.0,      //FORCE_LEVEL_0
    1024.0,   //FORCE_LEVEL_1
    2048.0,   //FORCE_LEVEL_2
    4096.0,   //FORCE_LEVEL_3
    4096.0,   //FORCE_LEVEL_4
    4096.0,   //FORCE_LEVEL_5
];

/*
=============
SV_EmitPacketEntities

Writes a delta update of an entityState_t list to the message.
=============
*/
unsafe fn SV_EmitPacketEntities(from: *mut clientSnapshot_t, to: *mut clientSnapshot_t, msg: *mut msg_t) {
    let mut oldent: *mut c_void;
    let mut newent: *mut c_void;
    let mut oldindex: c_int;
    let mut newindex: c_int;
    let mut oldnum: c_int;
    let mut newnum: c_int;
    let mut from_num_entities: c_int;

    // generate the delta update
    if from.is_null() {
        from_num_entities = 0;
    } else {
        from_num_entities = (*from).num_entities;
    }

    newent = core::ptr::null_mut();
    oldent = core::ptr::null_mut();
    newindex = 0;
    oldindex = 0;
    let num2Send = if (*to).num_entities >= (*svs).numSnapshotEntities {
        (*svs).numSnapshotEntities
    } else {
        (*to).num_entities
    };

    while newindex < num2Send || oldindex < from_num_entities {
        if newindex >= num2Send {
            newnum = 9999;
        } else {
            newent = ((*svs).snapshotEntities as *mut u8)
                .add(((((*to).first_entity + newindex) % (*svs).numSnapshotEntities) as usize) * core::mem::size_of::<c_int>()) as *mut c_void;
            newnum = *(newent as *mut c_int);
        }

        if oldindex >= from_num_entities {
            oldnum = 9999;
        } else {
            oldent = ((*svs).snapshotEntities as *mut u8)
                .add(((((*from).first_entity + oldindex) % (*svs).numSnapshotEntities) as usize) * core::mem::size_of::<c_int>()) as *mut c_void;
            oldnum = *(oldent as *mut c_int);
        }

        if newnum == oldnum {
            // delta update from old position
            // because the force parm is qfalse, this will not result
            // in any bytes being emited if the entity has not changed at all
            MSG_WriteEntity(msg, newent, 0);
            oldindex += 1;
            newindex += 1;
            continue;
        }

        if newnum < oldnum {
            // this is a new entity, send it from the baseline
            MSG_WriteEntity(msg, newent, 0);
            newindex += 1;
            continue;
        }

        if newnum > oldnum {
            // the old entity isn't present in the new message
            if !oldent.is_null() {
                MSG_WriteEntity(msg, core::ptr::null_mut(), *(oldent as *mut c_int));
            }
            oldindex += 1;
            continue;
        }
    }

    MSG_WriteBits(msg, (MAX_GENTITIES as c_int - 1), GENTITYNUM_BITS); // end of packetentities
}

/*
==================
SV_WriteSnapshotToClient
==================
*/
unsafe fn SV_WriteSnapshotToClient(client: *mut client_s, msg: *mut msg_t) {
    let mut frame: *mut clientSnapshot_t;
    let mut oldframe: *mut clientSnapshot_t;
    let mut lastframe: c_int;
    let mut snapFlags: c_int;

    // Cast netchan to access outgoingSequence field
    let netchan = &*((&(*client).netchan) as *const c_void as *const local_netchan_t);
    let outgoingSeq = netchan.outgoingSequence;

    // this is the snapshot we are creating
    frame = &mut (*client).frames[(outgoingSeq as usize) & PACKET_MASK];

    // try to use a previous frame as the source for delta compressing the snapshot
    if (*client).deltaMessage <= 0 || (*client).state != CS_ACTIVE {
        // client is asking for a retransmit
        oldframe = core::ptr::null_mut();
        lastframe = 0;
    } else if outgoingSeq - (*client).deltaMessage >= (32 - 3) {
        // client hasn't gotten a good message through in a long time
        Com_DPrintf(
            b"%s: Delta request from out of date packet.\n\0".as_ptr() as *const c_char,
            (*client).name.as_ptr(),
        );
        oldframe = core::ptr::null_mut();
        lastframe = 0;
    } else {
        // we have a valid snapshot to delta from
        oldframe = &mut (*client).frames[((*client).deltaMessage as usize) & PACKET_MASK];
        lastframe = outgoingSeq - (*client).deltaMessage;

        // the snapshot's entities may still have rolled off the buffer, though
        if (*oldframe).first_entity <= (*svs).nextSnapshotEntities - (*svs).numSnapshotEntities {
            Com_DPrintf(
                b"%s: Delta request from out of date entities.\n\0".as_ptr() as *const c_char,
                (*client).name.as_ptr(),
            );
            oldframe = core::ptr::null_mut();
            lastframe = 0;
        }
    }

    MSG_WriteByte(msg, svc_snapshot as c_int);

    // let the client know which reliable clientCommands we have received
    MSG_WriteLong(msg, (*client).lastClientCommand);

    // send over the current server time so the client can drift
    // its view of time to try to match
    MSG_WriteLong(msg, (*sv).time);

    // we must write a message number, because recorded demos won't have
    // the same network message sequences
    MSG_WriteLong(msg, outgoingSeq);
    MSG_WriteByte(msg, lastframe); // what we are delta'ing from
    MSG_WriteLong(msg, (*client).cmdNum); // we have executed up to here

    snapFlags = (*client).rateDelayed | ((*client).droppedCommands << 1);
    (*client).droppedCommands = 0;

    MSG_WriteByte(msg, snapFlags);

    // send over the areabits
    MSG_WriteByte(msg, (*frame).areabytes);
    MSG_WriteData(msg, (*frame).areabits.as_ptr() as *const c_void, (*frame).areabytes);

    // delta encode the playerstate
    if !oldframe.is_null() {
        MSG_WriteDeltaPlayerstate(msg, &mut (*oldframe).ps as *mut c_void, &mut (*frame).ps as *mut c_void);
    } else {
        MSG_WriteDeltaPlayerstate(msg, core::ptr::null_mut(), &mut (*frame).ps as *mut c_void);
    }

    // delta encode the entities
    SV_EmitPacketEntities(oldframe, frame, msg);
}

/*
==================
SV_UpdateServerCommandsToClient

(re)send all server commands the client hasn't acknowledged yet
==================
*/
unsafe fn SV_UpdateServerCommandsToClient(client: *mut client_s, msg: *mut msg_t) {
    let mut i: c_int;

    // write any unacknowledged serverCommands
    i = (*client).reliableAcknowledge + 1;
    while i <= (*client).reliableSequence {
        MSG_WriteByte(msg, svc_serverCommand as c_int);
        MSG_WriteLong(msg, i);
        MSG_WriteString(
            msg,
            (*client).reliableCommands[(i as usize) & (MAX_RELIABLE_COMMANDS - 1)],
        );
        i += 1;
    }
}

/*
=======================
SV_QsortEntityNumbers
=======================
*/
extern "C" fn SV_QsortEntityNumbers(a: *const c_void, b: *const c_void) -> c_int {
    let ea: *const c_int = a as *const c_int;
    let eb: *const c_int = b as *const c_int;

    unsafe {
        if *ea == *eb {
            Com_Error(5, b"SV_QsortEntityStates: duplicated entity\0".as_ptr() as *const c_char);
        }

        if *ea < *eb {
            return -1;
        }
    }

    1
}

/*
===============
SV_AddEntToSnapshot
===============
*/
unsafe fn SV_AddEntToSnapshot(svEnt: *mut svEntity_s, gEnt: *mut gentity_t, eNums: *mut snapshotEntityNumbers_t) {
    // if we have already added this entity to this snapshot, don't add again
    if (*svEnt).snapshotCounter == (*sv).snapshotCounter {
        return;
    }
    (*svEnt).snapshotCounter = (*sv).snapshotCounter;

    // if we are full, silently discard entities
    if (*eNums).numSnapshotEntities == MAX_SNAPSHOT_ENTITIES as c_int {
        return;
    }

    if ((*sv).snapshotCounter & 1) != 0 && (*eNums).numSnapshotEntities == (*svs).numSnapshotEntities - 1 {
        //we're full, and about to wrap around and stomp ents, so half the time send the first set without stomping.
        return;
    }

    // Get entity number - access s.number field from gentity_t
    // The s field contains entity number at offset 0
    let gent_num = *(gEnt as *const c_void as *const u8 as *const c_int);
    (*eNums).snapshotEntities[(*eNums).numSnapshotEntities as usize] = gent_num;
    (*eNums).numSnapshotEntities += 1;
}

/*
===============
SV_PlayerCanSeeEnt
===============
*/
unsafe fn SV_PlayerCanSeeEnt(ent: *mut gentity_t, sightLevel: c_int) -> c_int {
    //return true if this ent is in view
    //NOTE: this is similar to the func CG_PlayerCanSeeCent in cg_players
    let mut viewOrg: [f32; 3] = [0.0; 3];
    let mut viewAngles: [f32; 3] = [0.0; 3];
    let mut viewFwd: [f32; 3] = [0.0; 3];
    let mut dir2Ent: [f32; 3] = [0.0; 3];

    if ent.is_null() {
        return 0;
    }

    if VM_Call(core::ptr::null_mut(), CG_CAMERA_POS, viewOrg.as_mut_ptr() as i64) != 0 {
        if VM_Call(core::ptr::null_mut(), CG_CAMERA_ANG, viewAngles.as_mut_ptr() as i64) != 0 {
            let mut dot: f32 = 0.25;
            let range: f32 = sv_sightRangeForLevel[sightLevel as usize];

            VectorSubtract(
                (ent as *const c_void as *const u8).add(0) as *const f32,
                viewOrg.as_ptr(),
                dir2Ent.as_mut_ptr(),
            );
            let entDist: f32 = VectorNormalize(dir2Ent.as_mut_ptr());

            // Check for EF_FORCE_VISIBLE flag in entity state
            let ent_flags = *(ent as *const c_void as *const u8 as *const c_int);
            if (ent_flags & EF_FORCE_VISIBLE) != 0 {
                //no dist check on them?
            } else {
                if entDist < 128.0 {
                    //can always see them if they're really close
                    return 1;
                }

                if entDist > range {
                    //too far away to see them
                    return 0;
                }
            }

            dot += (0.99 - dot) * entDist / range; //the farther away they are, the more in front they have to be

            AngleVectors(viewAngles.as_ptr(), viewFwd.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut());
            if DotProduct(viewFwd.as_ptr(), dir2Ent.as_ptr()) < dot {
                return 0;
            }
            return 1;
        }
    }
    0
}

/*
===============
SV_AddEntitiesVisibleFromPoint
===============
*/
unsafe fn SV_AddEntitiesVisibleFromPoint(
    origin: *const f32,
    frame: *mut clientSnapshot_t,
    eNums: *mut snapshotEntityNumbers_t,
    portal: c_int,
) {
    let mut e: c_int;
    let mut i: c_int;
    let mut ent: *mut gentity_t;
    let mut svEnt: *mut svEntity_s;
    let mut l: c_int;
    let mut clientarea: c_int;
    let mut clientcluster: c_int;
    let mut leafnum: c_int;
    let mut c_fullsend: c_int;
    let mut clientpvs: *const u8;
    let mut bitvector: *const u8;
    let mut sightOn: c_int = 0;

    // during an error shutdown message we may need to transmit
    // the shutdown message after the server has shutdown, so
    // specfically check for it
    if (*sv).state == 0 {
        return;
    }

    leafnum = CM_PointLeafnum(origin);
    clientarea = CM_LeafArea(leafnum);
    clientcluster = CM_LeafCluster(leafnum);

    // calculate the visible areas
    (*frame).areabytes = CM_WriteAreaBits((*frame).areabits.as_mut_ptr(), clientarea);

    clientpvs = CM_ClusterPVS(clientcluster);

    c_fullsend = 0;

    if portal == 0 {
        //not if this if through a portal...???  James said to do this...
        // Check for FP_SEE force power active in frame->ps.forcePowersActive
        // This would require accessing frame->ps field which is opaque
        // Placeholder for force sight check
    }

    // Placeholder for entity loop - would need access to ge->num_entities
    // This is left as a structural placeholder since ge is opaque
}

/*
=============
SV_BuildClientSnapshot

Decides which entities are going to be visible to the client, and
copies off the playerstate and areabits.

This properly handles multiple recursive portals, but the render
currently doesn't.

For viewing through other player's eyes, clent can be something other than client->gentity
=============
*/
unsafe fn SV_BuildClientSnapshot(client: *mut client_s) -> *mut clientSnapshot_t {
    let mut org: [f32; 3] = [0.0; 3];
    let mut frame: *mut clientSnapshot_t;
    let mut entityNumbers: snapshotEntityNumbers_t = snapshotEntityNumbers_t {
        numSnapshotEntities: 0,
        snapshotEntities: [0; MAX_SNAPSHOT_ENTITIES],
    };
    let mut i: c_int;
    let mut ent: *mut gentity_t;
    let mut state: *mut c_void;
    let mut clent: *mut gentity_t;

    // bump the counter used to prevent double adding
    (*sv).snapshotCounter += 1;

    // Cast netchan to get outgoingSequence
    let netchan = &*((&(*client).netchan) as *const c_void as *const local_netchan_t);

    // this is the frame we are creating
    frame = &mut (*client).frames[(netchan.outgoingSequence as usize) & PACKET_MASK];

    // clear everything in this snapshot
    entityNumbers.numSnapshotEntities = 0;
    memset(
        (*frame).areabits.as_mut_ptr() as *mut c_void,
        0,
        MAX_MAP_AREA_BYTES,
    );

    clent = (*client).gentity;
    if clent.is_null() {
        return frame;
    }

    // grab the current playerState_t
    // Copy from clent->client to frame->ps
    // Since types are opaque, this is a placeholder

    // find the client's viewpoint
    //if in camera mode use camera position instead
    if VM_Call(core::ptr::null_mut(), CG_CAMERA_POS, org.as_mut_ptr() as i64) == 0 {
        //org[2] += clent->client->viewheight;
    } else {
        // VectorCopy(clent->client->origin, org);
        // org[2] += clent->client->viewheight;
        // This requires access to opaque struct fields
    }

    // add all the entities directly visible to the eye, which
    // may include portal entities that merge other viewpoints
    SV_AddEntitiesVisibleFromPoint(org.as_ptr(), frame, &mut entityNumbers, 0);

    // if there were portals visible, there may be out of order entities
    // in the list which will need to be resorted for the delta compression
    // to work correctly.  This also catches the error condition
    // of an entity being included twice.
    qsort(
        entityNumbers.snapshotEntities.as_mut_ptr() as *mut c_void,
        entityNumbers.numSnapshotEntities as usize,
        core::mem::size_of::<c_int>(),
        Some(SV_QsortEntityNumbers),
    );

    // now that all viewpoint's areabits have been OR'd together, invert
    // all of them to make it a mask vector, which is what the renderer wants
    i = 0;
    while i < (MAX_MAP_AREA_BYTES / 4) as c_int {
        let ptr = (*frame).areabits.as_mut_ptr() as *mut c_int;
        *ptr.add(i as usize) = *ptr.add(i as usize) ^ -1;
        i += 1;
    }

    // copy the entity states out
    (*frame).num_entities = 0;
    (*frame).first_entity = (*svs).nextSnapshotEntities;
    i = 0;
    while i < entityNumbers.numSnapshotEntities {
        ent = SV_GentityNum(entityNumbers.snapshotEntities[i as usize]);
        state = ((*svs).snapshotEntities as *mut u8)
            .add(((*svs).nextSnapshotEntities % (*svs).numSnapshotEntities) as usize) as *mut c_void;
        // Copy entity state
        core::ptr::copy_nonoverlapping(
            ent as *const c_void,
            state,
            core::mem::size_of::<c_int>(),
        );
        (*svs).nextSnapshotEntities += 1;
        (*frame).num_entities += 1;
        i += 1;
    }

    frame
}

/*
=======================
SV_SendMessageToClient

Called by SV_SendClientSnapshot and SV_SendClientGameState
=======================
*/
pub unsafe fn SV_SendMessageToClient(msg: *mut msg_t, client: *mut client_s) {
    let mut rateMsec: c_int;

    // record information about the message
    let msg_local = &*(msg as *const c_void as *const local_msg_t);
    let msg_cursize = msg_local.cursize;
    let netchan = &*((&(*client).netchan) as *const c_void as *const local_netchan_t);
    let outgoing_seq = netchan.outgoingSequence;

    (*client).frames[(outgoing_seq as usize) & PACKET_MASK].messageSize = msg_cursize;
    (*client).frames[(outgoing_seq as usize) & PACKET_MASK].messageSent = (*sv).time;

    // send the datagram
    Netchan_Transmit(
        &mut (*client).netchan as *mut c_void,
        msg_cursize,
        msg_local.data,
    );

    // set nextSnapshotTime based on rate and requested number of updates

    // local clients get snapshots every frame (FIXME: also treat LAN clients)
    if netchan.remoteAddress[0] as c_int == NA_LOOPBACK {
        (*client).nextSnapshotTime = (*sv).time - 1;
        return;
    }

    // normal rate / snapshotMsec calculation
    rateMsec = (msg_cursize + HEADER_RATE_BYTES) * 1000 / (*client).rate;
    if rateMsec < (*client).snapshotMsec {
        rateMsec = (*client).snapshotMsec;
        (*client).rateDelayed = 0;
    } else {
        (*client).rateDelayed = 1;
    }

    (*client).nextSnapshotTime = (*sv).time + rateMsec;

    // if we haven't gotten a message from the client in over a second, we will
    // drop to only sending one snapshot a second until they timeout
    if (*sv).time - (*client).lastPacketTime > 1000 || (*client).state != CS_ACTIVE {
        if (*client).nextSnapshotTime < (*sv).time + 1000 {
            (*client).nextSnapshotTime = (*sv).time + 1000;
        }
        return;
    }
}

/*
=======================
SV_SendClientEmptyMessage

This is just an empty message so that we can tell if
the client dropped the gamestate that went out before
=======================
*/
pub unsafe fn SV_SendClientEmptyMessage(client: *mut client_s) {
    let mut msg: msg_t = core::mem::zeroed();
    let mut buffer: [u8; 10] = [0; 10];

    MSG_Init(&mut msg as *mut msg_t, buffer.as_mut_ptr(), 10);
    SV_SendMessageToClient(&mut msg as *mut msg_t, client);
}

/*
=======================
SV_SendClientSnapshot
=======================
*/
pub unsafe fn SV_SendClientSnapshot(client: *mut client_s) {
    let mut msg_buf: [u8; 17408] = [0; 17408]; // MAX_MSGLEN
    let mut msg: msg_t = core::mem::zeroed();

    // build the snapshot
    SV_BuildClientSnapshot(client);

    // bots need to have their snapshots build, but
    // the query them directly without needing to be sent
    if !(*client).gentity.is_null() {
        // Check SVF_BOT flag - would need to access gentity s field
        // Placeholder for bot check
    }

    MSG_Init(&mut msg as *mut msg_t, msg_buf.as_mut_ptr(), 17408);
    // Set allowoverflow flag
    let msg_ptr = &mut msg as *mut msg_t as *mut local_msg_t;
    (*msg_ptr).allowoverflow = 1;

    // (re)send any reliable server commands
    SV_UpdateServerCommandsToClient(client, &mut msg as *mut msg_t);

    // send over all the relevant entityState_t
    // and the playerState_t
    SV_WriteSnapshotToClient(client, &mut msg as *mut msg_t);

    // check for overflow
    if (*msg_ptr).overflowed != 0 {
        Com_DPrintf(
            b"WARNING: msg overflowed for %s\n\0".as_ptr() as *const c_char,
            (*client).name.as_ptr(),
        );
        MSG_Clear(&mut msg as *mut msg_t);
    }

    SV_SendMessageToClient(&mut msg as *mut msg_t, client);
}

/*
=======================
SV_SendClientMessages
=======================
*/
pub unsafe fn SV_SendClientMessages() {
    let mut i: c_int;
    let mut c: *mut client_s;

    // send a message to each connected client
    i = 0;
    c = (*svs).clients;
    while i < 1 {
        if (*c).state == 0 {
            i += 1;
            c = c.add(1);
            continue; // not connected
        }

        if (*sv).time < (*c).nextSnapshotTime {
            i += 1;
            c = c.add(1);
            continue; // not time yet
        }

        if (*c).state != CS_ACTIVE {
            if (*c).state != 1 {
                // CS_ZOMBIE = 1
                SV_SendClientEmptyMessage(c);
            }
            i += 1;
            c = c.add(1);
            continue;
        }

        SV_SendClientSnapshot(c);
        i += 1;
        c = c.add(1);
    }
}
