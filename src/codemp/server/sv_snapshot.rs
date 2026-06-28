#![allow(non_snake_case)]

use core::ffi::{c_int, c_void, c_char};

// Type stubs for external types
#[repr(C)]
pub struct clientSnapshot_t {
    pub num_entities: c_int,
    pub first_entity: c_int,
    pub areabytes: c_int,
    pub areabits: [u8; 16],  // MAX_MAP_AREA_BYTES
    pub ps: playerState_t,
    pub vps: playerState_t,
    pub messageSize: c_int,
    pub messageSent: c_int,
    pub messageAcked: c_int,
    #[cfg(feature = "_ONEBIT_COMBO")]
    pub pDeltaOneBit: *const c_int,
    #[cfg(feature = "_ONEBIT_COMBO")]
    pub pDeltaNumBit: *const c_int,
    #[cfg(feature = "_ONEBIT_COMBO")]
    pub pDeltaOneBitVeh: *const c_int,
    #[cfg(feature = "_ONEBIT_COMBO")]
    pub pDeltaNumBitVeh: *const c_int,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eFlags: c_int,
    pub isPortalEnt: c_int,
    pub origin: [f32; 3],
    pub origin2: [f32; 3],
    pub generic1: c_int,
    // additional fields would go here
}

#[repr(C)]
pub struct playerState_t {
    pub clientNum: c_int,
    pub m_iVehicleNum: c_int,
    pub origin: [f32; 3],
    pub viewheight: f32,
    pub deltaOneBits: c_int,
    pub deltaNumBits: c_int,
    // additional fields would go here
}

#[repr(C)]
pub struct sharedEntity_t {
    pub s: entityState_t,
    pub r: sharedEntityRecord_t,
    pub playerState: *const c_void,
    // additional fields would go here
}

#[repr(C)]
pub struct sharedEntityRecord_t {
    pub linked: bool,
    pub svFlags: c_int,
    pub broadcastClients: [u32; 2],  // MAX_CLIENTS/32 rounded up
    pub singleClient: c_int,
    pub areanum: c_int,
    pub areanum2: c_int,
    pub numClusters: c_int,
    pub clusternums: [c_int; 16],  // MAX_ENTITY_CLUSTERS
    pub lastCluster: c_int,
    pub absmin: [f32; 3],
    pub absmax: [f32; 3],
}

#[repr(C)]
pub struct svEntity_t {
    pub snapshotCounter: c_int,
    pub baseline: entityState_t,
    pub areanum: c_int,
    pub areanum2: c_int,
    pub numClusters: c_int,
    pub clusternums: [c_int; 16],  // MAX_ENTITY_CLUSTERS
    pub lastCluster: c_int,
}

#[repr(C)]
pub struct msg_t {
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
    pub oob: c_int,
    pub allowoverflow: c_int,
    pub overflowed: c_int,
}

#[repr(C)]
pub struct client_t {
    pub state: c_int,
    pub gentity: *mut sharedEntity_t,
    pub name: [c_char; 32],
    pub netchan: netchan_t,
    pub frames: *mut clientSnapshot_t,
    pub deltaMessage: c_int,
    pub reliableAcknowledge: c_int,
    pub reliableSequence: c_int,
    pub reliableSent: c_int,
    pub reliableCommands: *mut [c_char; 1024],  // MAX_RELIABLE_COMMANDS
    pub rate: c_int,
    pub nextSnapshotTime: c_int,
    pub snapshotMsec: c_int,
    pub rateDelayed: c_int,
    pub sentGamedir: c_int,
    pub lastClientCommand: c_int,
    pub downloadName: [c_char; 256],
}

#[repr(C)]
pub struct netchan_t {
    pub outgoingSequence: c_int,
    pub unsentFragments: c_int,
    pub unsentLength: c_int,
    pub unsentFragmentStart: c_int,
    pub remoteAddress: netadr_t,
}

#[repr(C)]
pub struct netadr_t {
    pub typ: c_int,
    // additional fields would go here
}

#[repr(C)]
pub struct serverStatic_t {
    pub clients: *mut client_t,
    pub time: c_int,
    pub snapFlagServerBit: c_int,
    pub nextSnapshotEntities: c_int,
    pub numSnapshotEntities: c_int,
    pub snapshotEntities: *mut entityState_t,
}

#[repr(C)]
pub struct server_t {
    pub state: c_int,
    pub snapshotCounter: c_int,
    pub num_entities: c_int,
    pub svEntities: *mut svEntity_t,
}

#[repr(C)]
pub struct cvar_t {
    pub string: *mut c_char,
    pub integer: c_int,
    // additional fields would go here
}

// Constants
const PACKET_MASK: c_int = 0x1F;  // depends on PACKET_BACKUP
const CS_ACTIVE: c_int = 0;
const CS_ZOMBIE: c_int = 1;
const MAX_GENTITIES: c_int = 4096;
const GENTITYNUM_BITS: c_int = 12;
const SNAPFLAG_RATE_DELAYED: c_int = 1;
const SNAPFLAG_NOT_ACTIVE: c_int = 2;
const EF_PERMANENT: c_int = 0x0200;
const SVF_NOCLIENT: c_int = 0x0001;
const SVF_SINGLECLIENT: c_int = 0x0002;
const SVF_NOTSINGLECLIENT: c_int = 0x0004;
const SVF_BROADCAST: c_int = 0x0020;
const SVF_PORTAL: c_int = 0x0040;
const SVF_BOT: c_int = 0x0800;
const MAX_MAP_AREA_BYTES: c_int = 16;
const NA_LOOPBACK: c_int = 1;
const MAX_MSGLEN: c_int = 16384;
const MAX_SNAPSHOT_ENTITIES: c_int = 1024;
const HEADER_RATE_BYTES: c_int = 48;
const MAX_RELIABLE_COMMANDS: c_int = 64;
const PACKET_BACKUP: c_int = 32;
const svc_snapshot: u8 = 4;
const svc_serverCommand: u8 = 5;
const svc_nop: u8 = 1;
const svc_setgame: u8 = 10;
const ERR_DROP: c_int = 0;
const ERR_FATAL: c_int = 1;

// External function declarations
extern "C" {
    pub static mut svs: serverStatic_t;
    pub static mut sv: server_t;
    pub static mut com_RMG: *mut cvar_t;
    pub static mut sv_padPackets: *mut cvar_t;
    pub static mut sv_maxRate: *mut cvar_t;
    pub static mut sv_maxclients: *mut cvar_t;
    pub static mut fs_gamedirvar: *mut cvar_t;

    pub fn MSG_WriteDeltaEntity(msg: *mut msg_t, from: *const entityState_t, to: *const entityState_t, force: c_int);
    pub fn MSG_WriteBits(msg: *mut msg_t, value: c_int, bits: c_int);
    pub fn MSG_WriteByte(msg: *mut msg_t, c: u8);
    pub fn MSG_WriteLong(msg: *mut msg_t, l: c_int);
    pub fn MSG_WriteString(msg: *mut msg_t, s: *const c_char);
    pub fn MSG_WriteData(msg: *mut msg_t, data: *const c_void, len: c_int);
    #[cfg(feature = "_ONEBIT_COMBO")]
    pub fn MSG_WriteDeltaPlayerstate(msg: *mut msg_t, from: *const playerState_t, to: *const playerState_t,
                                      pDeltaOneBit: *const c_int, pDeltaNumBit: *const c_int, force: c_int);
    #[cfg(not(feature = "_ONEBIT_COMBO"))]
    pub fn MSG_WriteDeltaPlayerstate(msg: *mut msg_t, from: *const playerState_t, to: *const playerState_t, force: c_int);

    pub fn MSG_Clear(msg: *mut msg_t);
    pub fn MSG_Init(msg: *mut msg_t, data: *mut u8, length: c_int);

    pub fn CM_PointLeafnum(p: *const f32) -> c_int;
    pub fn CM_LeafArea(leafnum: c_int) -> c_int;
    pub fn CM_LeafCluster(leafnum: c_int) -> c_int;
    pub fn CM_WriteAreaBits(buffer: *mut u8, area: c_int) -> c_int;
    pub fn CM_ClusterPVS(cluster: c_int) -> *const u8;
    pub fn CM_AreasConnected(area1: c_int, area2: c_int) -> c_int;

    pub fn SV_GentityNum(num: c_int) -> *mut sharedEntity_t;
    pub fn SV_SvEntityForGentity(gent: *mut sharedEntity_t) -> *mut svEntity_t;
    pub fn SV_GameClientNum(clientNum: c_int) -> *mut playerState_t;
    pub fn SV_Netchan_TransmitNextFragment(netchan: *mut netchan_t);
    pub fn SV_Netchan_Transmit(client: *mut client_t, msg: *mut msg_t);
    pub fn SV_WriteDownloadToClient(client: *mut client_t, msg: *mut msg_t);

    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Memset(dst: *mut c_void, c: c_int, count: usize);

    pub fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn Sys_IsLANAddress(adr: netadr_t) -> c_int;
    pub fn VM_ArgPtr(p: c_int) -> *mut c_void;

    pub fn VectorAdd(veca: *const f32, vecb: *const f32, out: *mut f32);
    pub fn VectorScale(in_v: *const f32, scale: f32, out: *mut f32);
    pub fn VectorSubtract(veca: *const f32, vecb: *const f32, out: *mut f32);
    pub fn VectorCopy(in_v: *const f32, out: *mut f32);
    pub fn VectorLength(v: *const f32) -> f32;
    pub fn VectorLengthSquared(v: *const f32) -> f32;

    pub fn qsort(base: *mut c_void, nmemb: usize, size: usize,
                 compar: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int);
}

// Local global variable
pub static mut g_svCullDist: f32 = -1.0;

/*
=============================================================================

Delta encode a client frame onto the network channel

A normal server packet will look like:

4	sequence number (high bit set if an oversize fragment)
<optional reliable commands>
1	svc_snapshot
4	last client reliable command
4	serverTime
1	lastframe for delta compression
1	snapFlags
1	areaBytes
<areabytes>
<playerstate>
<packetentities>

=============================================================================
*/

/*
=============
SV_EmitPacketEntities

Writes a delta update of an entityState_t list to the message.
=============
*/
unsafe fn SV_EmitPacketEntities(from: *const clientSnapshot_t, to: *const clientSnapshot_t, msg: *mut msg_t) {
    let mut oldent: *const entityState_t = core::ptr::null();
    let mut newent: *const entityState_t = core::ptr::null();
    let mut oldindex: c_int = 0;
    let mut newindex: c_int = 0;
    let mut oldnum: c_int;
    let mut newnum: c_int;
    let from_num_entities: c_int;

    // generate the delta update
    if from.is_null() {
        from_num_entities = 0;
    } else {
        from_num_entities = (*from).num_entities;
    }

    newent = core::ptr::null();
    oldent = core::ptr::null();
    newindex = 0;
    oldindex = 0;
    while newindex < (*to).num_entities || oldindex < from_num_entities {
        if newindex >= (*to).num_entities {
            newnum = 9999;
        } else {
            newent = &(*svs.snapshotEntities.add(((*to).first_entity + newindex) as usize % svs.numSnapshotEntities as usize));
            newnum = (*newent).number;
        }

        if oldindex >= from_num_entities {
            oldnum = 9999;
        } else {
            oldent = &(*svs.snapshotEntities.add(((*from).first_entity + oldindex) as usize % svs.numSnapshotEntities as usize));
            oldnum = (*oldent).number;
        }

        if newnum == oldnum {
            // delta update from old position
            // because the force parm is qfalse, this will not result
            // in any bytes being emited if the entity has not changed at all
            MSG_WriteDeltaEntity(msg, oldent, newent, 0);
            oldindex += 1;
            newindex += 1;
            continue;
        }

        if newnum < oldnum {
            // this is a new entity, send it from the baseline
            MSG_WriteDeltaEntity(msg, &(*sv.svEntities.add(newnum as usize)).baseline, newent, 1);
            newindex += 1;
            continue;
        }

        if newnum > oldnum {
            // the old entity isn't present in the new message
            MSG_WriteDeltaEntity(msg, oldent, core::ptr::null(), 1);
            oldindex += 1;
            continue;
        }
    }

    MSG_WriteBits(msg, (MAX_GENTITIES - 1) as c_int, GENTITYNUM_BITS as c_int);	// end of packetentities
}



/*
==================
SV_WriteSnapshotToClient
==================
*/
unsafe fn SV_WriteSnapshotToClient(client: *mut client_t, msg: *mut msg_t) {
    let mut frame: *mut clientSnapshot_t;
    let mut oldframe: *mut clientSnapshot_t;
    let mut lastframe: c_int;
    let mut i: c_int;
    let mut snapFlags: c_int;

    // this is the snapshot we are creating
    frame = (*client).frames.add(((*client).netchan.outgoingSequence & PACKET_MASK) as usize) as *mut clientSnapshot_t;

    // try to use a previous frame as the source for delta compressing the snapshot
    if (*client).deltaMessage <= 0 || (*client).state != CS_ACTIVE {
        // client is asking for a retransmit
        oldframe = core::ptr::null_mut();
        lastframe = 0;
    } else if (*client).netchan.outgoingSequence - (*client).deltaMessage >= (PACKET_BACKUP - 3) {
        // client hasn't gotten a good message through in a long time
        Com_DPrintf(b"%s: Delta request from out of date packet.\n\0".as_ptr() as *const c_char, (*client).name.as_ptr());
        oldframe = core::ptr::null_mut();
        lastframe = 0;
    } else {
        // we have a valid snapshot to delta from
        oldframe = (*client).frames.add(((*client).deltaMessage & PACKET_MASK) as usize) as *mut clientSnapshot_t;
        lastframe = (*client).netchan.outgoingSequence - (*client).deltaMessage;

        // the snapshot's entities may still have rolled off the buffer, though
        if (*oldframe).first_entity <= svs.nextSnapshotEntities - svs.numSnapshotEntities {
            Com_DPrintf(b"%s: Delta request from out of date entities.\n\0".as_ptr() as *const c_char, (*client).name.as_ptr());
            oldframe = core::ptr::null_mut();
            lastframe = 0;
        }
    }

    MSG_WriteByte(msg, svc_snapshot);

    // NOTE, MRE: now sent at the start of every message from server to client
    // let the client know which reliable clientCommands we have received
    //MSG_WriteLong( msg, client->lastClientCommand );

    // send over the current server time so the client can drift
    // its view of time to try to match
    MSG_WriteLong(msg, svs.time);

    // what we are delta'ing from
    MSG_WriteByte(msg, lastframe as u8);

    snapFlags = svs.snapFlagServerBit;
    if (*client).rateDelayed != 0 {
        snapFlags |= SNAPFLAG_RATE_DELAYED;
    }
    if (*client).state != CS_ACTIVE {
        snapFlags |= SNAPFLAG_NOT_ACTIVE;
    }

    MSG_WriteByte(msg, snapFlags as u8);

    // send over the areabits
    MSG_WriteByte(msg, (*frame).areabytes as u8);
    MSG_WriteData(msg, (*frame).areabits.as_ptr() as *const c_void, (*frame).areabytes);

    // delta encode the playerstate
    if !oldframe.is_null() {
        #[cfg(feature = "_ONEBIT_COMBO")]
        {
            MSG_WriteDeltaPlayerstate(msg, &(*oldframe).ps, &(*frame).ps, (*frame).pDeltaOneBit, (*frame).pDeltaNumBit, 0);
        }
        #[cfg(not(feature = "_ONEBIT_COMBO"))]
        {
            MSG_WriteDeltaPlayerstate(msg, &(*oldframe).ps, &(*frame).ps, 0);
        }
        if (*frame).ps.m_iVehicleNum != 0 {
            // then write the vehicle's playerstate too
            if (*oldframe).ps.m_iVehicleNum == 0 {
                // if last frame didn't have vehicle, then the old vps isn't gonna delta
                // properly (because our vps on the client could be anything)
                #[cfg(feature = "_ONEBIT_COMBO")]
                {
                    MSG_WriteDeltaPlayerstate(msg, core::ptr::null(), &(*frame).vps, core::ptr::null(), core::ptr::null(), 1);
                }
                #[cfg(not(feature = "_ONEBIT_COMBO"))]
                {
                    MSG_WriteDeltaPlayerstate(msg, core::ptr::null(), &(*frame).vps, 1);
                }
            }
            else {
                #[cfg(feature = "_ONEBIT_COMBO")]
                {
                    MSG_WriteDeltaPlayerstate(msg, &(*oldframe).vps, &(*frame).vps, (*frame).pDeltaOneBitVeh, (*frame).pDeltaNumBitVeh, 1);
                }
                #[cfg(not(feature = "_ONEBIT_COMBO"))]
                {
                    MSG_WriteDeltaPlayerstate(msg, &(*oldframe).vps, &(*frame).vps, 1);
                }
            }
        }
    } else {
        #[cfg(feature = "_ONEBIT_COMBO")]
        {
            MSG_WriteDeltaPlayerstate(msg, core::ptr::null(), &(*frame).ps, core::ptr::null(), core::ptr::null(), 0);
        }
        #[cfg(not(feature = "_ONEBIT_COMBO"))]
        {
            MSG_WriteDeltaPlayerstate(msg, core::ptr::null(), &(*frame).ps, 0);
        }
        if (*frame).ps.m_iVehicleNum != 0 {
            // then write the vehicle's playerstate too
            #[cfg(feature = "_ONEBIT_COMBO")]
            {
                MSG_WriteDeltaPlayerstate(msg, core::ptr::null(), &(*frame).vps, core::ptr::null(), core::ptr::null(), 1);
            }
            #[cfg(not(feature = "_ONEBIT_COMBO"))]
            {
                MSG_WriteDeltaPlayerstate(msg, core::ptr::null(), &(*frame).vps, 1);
            }
        }
    }

    // delta encode the entities
    SV_EmitPacketEntities(oldframe, frame, msg);

    // padding for rate debugging
    if (*sv_padPackets).integer != 0 {
        i = 0;
        while i < (*sv_padPackets).integer {
            MSG_WriteByte(msg, svc_nop);
            i += 1;
        }
    }
}


/*
==================
SV_UpdateServerCommandsToClient

(re)send all server commands the client hasn't acknowledged yet
==================
*/
pub unsafe fn SV_UpdateServerCommandsToClient(client: *mut client_t, msg: *mut msg_t) {
    let mut i: c_int;

    // write any unacknowledged serverCommands
    i = (*client).reliableAcknowledge + 1;
    while i <= (*client).reliableSequence {
        MSG_WriteByte(msg, svc_serverCommand);
        MSG_WriteLong(msg, i);
        MSG_WriteString(msg, (*(*client).reliableCommands.add((i & (MAX_RELIABLE_COMMANDS - 1)) as usize)).as_ptr());
        i += 1;
    }
    (*client).reliableSent = (*client).reliableSequence;
}

/*
=============================================================================

Build a client snapshot structure

=============================================================================
*/

#[repr(C)]
struct snapshotEntityNumbers_t {
    numSnapshotEntities: c_int,
    snapshotEntities: [c_int; MAX_SNAPSHOT_ENTITIES as usize],
}

/*
=======================
SV_QsortEntityNumbers
=======================
*/
unsafe extern "C" fn SV_QsortEntityNumbers(a: *const c_void, b: *const c_void) -> c_int {
    let ea: *const c_int = a as *const c_int;
    let eb: *const c_int = b as *const c_int;

    if *ea == *eb {
        Com_Error(ERR_DROP, b"SV_QsortEntityStates: duplicated entity\0".as_ptr() as *const c_char);
    }

    if *ea < *eb {
        return -1;
    }

    return 1;
}


/*
===============
SV_AddEntToSnapshot
===============
*/
unsafe fn SV_AddEntToSnapshot(svEnt: *mut svEntity_t, gEnt: *mut sharedEntity_t, eNums: *mut snapshotEntityNumbers_t) {
    // if we have already added this entity to this snapshot, don't add again
    if (*svEnt).snapshotCounter == sv.snapshotCounter {
        return;
    }
    (*svEnt).snapshotCounter = sv.snapshotCounter;

    // if we are full, silently discard entities
    if (*eNums).numSnapshotEntities == MAX_SNAPSHOT_ENTITIES {
        return;
    }

    (*eNums).snapshotEntities[(*eNums).numSnapshotEntities as usize] = (*gEnt).s.number;
    (*eNums).numSnapshotEntities += 1;
}

/*
===============
SV_AddEntitiesVisibleFromPoint
===============
*/
unsafe fn SV_AddEntitiesVisibleFromPoint(origin: *const f32, frame: *mut clientSnapshot_t,
                                    eNums: *mut snapshotEntityNumbers_t, portal: c_int) {
    let mut e: c_int;
    let mut i: c_int;
    let mut ent: *mut sharedEntity_t;
    let mut svEnt: *mut svEntity_t;
    let mut l: c_int;
    let mut clientarea: c_int;
    let mut clientcluster: c_int;
    let mut leafnum: c_int;
    let mut c_fullsend: c_int;
    let mut clientpvs: *const u8;
    let mut bitvector: *const u8;
    let mut difference: [f32; 3] = [0.0; 3];
    let mut length: f32;
    let mut radius: f32;

    // during an error shutdown message we may need to transmit
    // the shutdown message after the server has shutdown, so
    // specfically check for it
    if sv.state == 0 {
        return;
    }

    leafnum = CM_PointLeafnum(origin);
    clientarea = CM_LeafArea(leafnum);
    clientcluster = CM_LeafCluster(leafnum);

    // calculate the visible areas
    (*frame).areabytes = CM_WriteAreaBits((*frame).areabits.as_mut_ptr(), clientarea);

    clientpvs = CM_ClusterPVS(clientcluster);

    c_fullsend = 0;

    e = 0;
    while e < sv.num_entities {
        ent = SV_GentityNum(e);

        // never send entities that aren't linked in
        if (*ent).r.linked == false {
            e += 1;
            continue;
        }

        if ((*ent).s.eFlags & EF_PERMANENT) != 0 {
            // he's permanent, so don't send him down!
            e += 1;
            continue;
        }

        if (*ent).s.number != e {
            Com_DPrintf(b"FIXING ENT->S.NUMBER!!!\n\0".as_ptr() as *const c_char);
            (*ent).s.number = e;
        }

        // entities can be flagged to explicitly not be sent to the client
        if ((*ent).r.svFlags & SVF_NOCLIENT) != 0 {
            e += 1;
            continue;
        }

        // entities can be flagged to be sent to only one client
        if ((*ent).r.svFlags & SVF_SINGLECLIENT) != 0 {
            if (*ent).r.singleClient != (*frame).ps.clientNum {
                e += 1;
                continue;
            }
        }
        // entities can be flagged to be sent to everyone but one client
        if ((*ent).r.svFlags & SVF_NOTSINGLECLIENT) != 0 {
            if (*ent).r.singleClient == (*frame).ps.clientNum {
                e += 1;
                continue;
            }
        }

        svEnt = SV_SvEntityForGentity(ent);

        // don't double add an entity through portals
        if (*svEnt).snapshotCounter == sv.snapshotCounter {
            e += 1;
            continue;
        }

        // broadcast entities are always sent, and so is the main player so we don't see noclip weirdness
        if ((*ent).r.svFlags & SVF_BROADCAST) != 0 || (e == (*frame).ps.clientNum) || (((*ent).r.broadcastClients[((*frame).ps.clientNum / 32) as usize] & (1 << ((*frame).ps.clientNum % 32))) != 0)
        {
            SV_AddEntToSnapshot(svEnt, ent, eNums);
            e += 1;
            continue;
        }

        if (*ent).r.isPortalEnt != false {
            // rww - portal entities are always sent as well
            SV_AddEntToSnapshot(svEnt, ent, eNums);
            e += 1;
            continue;
        }

        if !com_RMG.is_null() && (*com_RMG).integer != 0 {
            VectorAdd((*ent).r.absmax.as_ptr(), (*ent).r.absmin.as_ptr(), difference.as_mut_ptr());
            VectorScale(difference.as_ptr(), 0.5, difference.as_mut_ptr());
            VectorSubtract(origin, difference.as_ptr(), difference.as_mut_ptr());
            length = VectorLength(difference.as_ptr());

            // calculate the diameter
            VectorSubtract((*ent).r.absmax.as_ptr(), (*ent).r.absmin.as_ptr(), difference.as_mut_ptr());
            radius = VectorLength(difference.as_ptr());
            if length - radius < 5000.0 {
                // more of a diameter check
                SV_AddEntToSnapshot(svEnt, ent, eNums);
            }
        }
        else {
            // ignore if not touching a PV leaf
            // check area
            if (CM_AreasConnected(clientarea, (*svEnt).areanum) == 0) {
                // doors can legally straddle two areas, so
                // we may need to check another one
                if (CM_AreasConnected(clientarea, (*svEnt).areanum2) == 0) {
                    e += 1;
                    continue;		// blocked by a door
                }
            }

            bitvector = clientpvs;

            // check individual leafs
            if (*svEnt).numClusters == 0 {
                e += 1;
                continue;
            }
            l = 0;
            #[cfg(feature = "_XBOX")]
            {
                if !bitvector.is_null() {
                    i = 0;
                    while i < (*svEnt).numClusters {
                        l = (*svEnt).clusternums[i as usize];
                        if (*bitvector.add((l >> 3) as usize) & (1 << (l & 7))) != 0 {
                            break;
                        }
                        i += 1;
                    }
                }
            }
            #[cfg(not(feature = "_XBOX"))]
            {
                i = 0;
                while i < (*svEnt).numClusters {
                    l = (*svEnt).clusternums[i as usize];
                    if (*bitvector.add((l >> 3) as usize) & (1 << (l & 7))) != 0 {
                        break;
                    }
                    i += 1;
                }
            }

            // if we haven't found it to be visible,
            // check overflow clusters that coudln't be stored
            #[cfg(feature = "_XBOX")]
            {
                if !bitvector.is_null() && i == (*svEnt).numClusters {
                    if (*svEnt).lastCluster != 0 {
                        let mut l_check = l;
                        while l_check <= (*svEnt).lastCluster {
                            if (*bitvector.add((l_check >> 3) as usize) & (1 << (l_check & 7))) != 0 {
                                break;
                            }
                            l_check += 1;
                        }
                        if l_check == (*svEnt).lastCluster {
                            e += 1;
                            continue;	// not visible
                        }
                    } else {
                        e += 1;
                        continue;
                    }
                }
            }
            #[cfg(not(feature = "_XBOX"))]
            {
                if i == (*svEnt).numClusters {
                    if (*svEnt).lastCluster != 0 {
                        let mut l_check = l;
                        while l_check <= (*svEnt).lastCluster {
                            if (*bitvector.add((l_check >> 3) as usize) & (1 << (l_check & 7))) != 0 {
                                break;
                            }
                            l_check += 1;
                        }
                        if l_check == (*svEnt).lastCluster {
                            e += 1;
                            continue;	// not visible
                        }
                    } else {
                        e += 1;
                        continue;
                    }
                }
            }

            if g_svCullDist != -1.0 {
                // do a distance cull check
                VectorAdd((*ent).r.absmax.as_ptr(), (*ent).r.absmin.as_ptr(), difference.as_mut_ptr());
                VectorScale(difference.as_ptr(), 0.5, difference.as_mut_ptr());
                VectorSubtract(origin, difference.as_ptr(), difference.as_mut_ptr());
                length = VectorLength(difference.as_ptr());

                // calculate the diameter
                VectorSubtract((*ent).r.absmax.as_ptr(), (*ent).r.absmin.as_ptr(), difference.as_mut_ptr());
                radius = VectorLength(difference.as_ptr());
                if length - radius >= g_svCullDist {
                    // then don't add it
                    e += 1;
                    continue;
                }
            }

            // add it
            SV_AddEntToSnapshot(svEnt, ent, eNums);

            // if its a portal entity, add everything visible from its camera position
            if ((*ent).r.svFlags & SVF_PORTAL) != 0 {
                if (*ent).s.generic1 != 0 {
                    let mut dir: [f32; 3] = [0.0; 3];
                    VectorSubtract((*ent).s.origin2.as_ptr(), origin, dir.as_mut_ptr());
                    if VectorLengthSquared(dir.as_ptr()) > (((*ent).s.generic1 as f32) * ((*ent).s.generic1 as f32)) {
                        e += 1;
                        continue;
                    }
                }
                SV_AddEntitiesVisibleFromPoint((*ent).s.origin2.as_ptr(), frame, eNums, 1);
                #[cfg(feature = "_XBOX")]
                {
                    // Must get clientpvs again since above call destroyed it.
                    clientpvs = CM_ClusterPVS(clientcluster);
                }
            }
        }
        e += 1;
    }
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
unsafe fn SV_BuildClientSnapshot(client: *mut client_t) {
    let mut org: [f32; 3] = [0.0; 3];
    let mut frame: *mut clientSnapshot_t;
    let mut entityNumbers: snapshotEntityNumbers_t;
    let mut i: c_int;
    let mut ent: *mut sharedEntity_t;
    let mut state: *mut entityState_t;
    let mut svEnt: *mut svEntity_t;
    let mut clent: *mut sharedEntity_t;
    let mut ps: *mut playerState_t;

    // bump the counter used to prevent double adding
    sv.snapshotCounter += 1;

    // this is the frame we are creating
    frame = (*client).frames.add(((*client).netchan.outgoingSequence & PACKET_MASK) as usize) as *mut clientSnapshot_t;

    // clear everything in this snapshot
    entityNumbers.numSnapshotEntities = 0;
    Com_Memset((*frame).areabits.as_mut_ptr() as *mut c_void, 0, core::mem::size_of_val(&(*frame).areabits));

    (*frame).num_entities = 0;

    clent = (*client).gentity;
    if clent.is_null() || (*client).state == CS_ZOMBIE {
        return;
    }

    // grab the current playerState_t
    ps = SV_GameClientNum(client as *const _ as *const c_int as c_int);
    (*frame).ps = *ps;
    #[cfg(feature = "_ONEBIT_COMBO")]
    {
        (*frame).pDeltaOneBit = &(*ps).deltaOneBits;
        (*frame).pDeltaNumBit = &(*ps).deltaNumBits;
    }

    if (*ps).m_iVehicleNum != 0 {
        // get the vehicle's playerstate too then
        let veh: *mut sharedEntity_t = SV_GentityNum((*ps).m_iVehicleNum);

        if !veh.is_null() && !(*veh).playerState.is_null() {
            // Now VMA it and we've got ourselves a playerState
            let vps: *mut playerState_t = VM_ArgPtr((*veh).playerState as c_int) as *mut playerState_t;

            (*frame).vps = *vps;
            #[cfg(feature = "_ONEBIT_COMBO")]
            {
                (*frame).pDeltaOneBitVeh = &(*vps).deltaOneBits;
                (*frame).pDeltaNumBitVeh = &(*vps).deltaNumBits;
            }
        }
    }

    let clientNum: c_int;
    // never send client's own entity, because it can
    // be regenerated from the playerstate
    clientNum = (*frame).ps.clientNum;
    if clientNum < 0 || clientNum >= MAX_GENTITIES {
        Com_Error(ERR_DROP, b"SV_SvEntityForGentity: bad gEnt\0".as_ptr() as *const c_char);
    }
    svEnt = &mut (*sv.svEntities.add(clientNum as usize));
    (*svEnt).snapshotCounter = sv.snapshotCounter;


    // find the client's viewpoint
    VectorCopy((*ps).origin.as_ptr(), org.as_mut_ptr());
    org[2] += (*ps).viewheight;

    // add all the entities directly visible to the eye, which
    // may include portal entities that merge other viewpoints
    SV_AddEntitiesVisibleFromPoint(org.as_ptr(), frame, &mut entityNumbers, 0);

    // if there were portals visible, there may be out of order entities
    // in the list which will need to be resorted for the delta compression
    // to work correctly.  This also catches the error condition
    // of an entity being included twice.
    qsort(entityNumbers.snapshotEntities.as_mut_ptr() as *mut c_void, entityNumbers.numSnapshotEntities as usize,
        core::mem::size_of::<c_int>(), SV_QsortEntityNumbers);

    // now that all viewpoint's areabits have been OR'd together, invert
    // all of them to make it a mask vector, which is what the renderer wants
    i = 0;
    while i < MAX_MAP_AREA_BYTES / 4 {
        let ptr: *mut c_int = (*frame).areabits.as_mut_ptr() as *mut c_int;
        *ptr.add(i as usize) = *ptr.add(i as usize) ^ -1;
        i += 1;
    }

    // copy the entity states out
    (*frame).num_entities = 0;
    (*frame).first_entity = svs.nextSnapshotEntities;
    i = 0;
    while i < entityNumbers.numSnapshotEntities {
        ent = SV_GentityNum(entityNumbers.snapshotEntities[i as usize]);
        state = &mut (*svs.snapshotEntities.add((svs.nextSnapshotEntities % svs.numSnapshotEntities) as usize));
        *state = (*ent).s;
        svs.nextSnapshotEntities += 1;
        // this should never hit, map should always be restarted first in SV_Frame
        if svs.nextSnapshotEntities >= 0x7FFFFFFE {
            Com_Error(ERR_FATAL, b"svs.nextSnapshotEntities wrapped\0".as_ptr() as *const c_char);
        }
        (*frame).num_entities += 1;
        i += 1;
    }
}


/*
====================
SV_RateMsec

Return the number of msec a given size message is supposed
to take to clear, based on the current rate
====================
*/
fn SV_RateMsec(client: *const client_t, mut messageSize: c_int) -> c_int {
    let mut rate: c_int;
    let mut rateMsec: c_int;

    // individual messages will never be larger than fragment size
    if messageSize > 1500 {
        messageSize = 1500;
    }
    unsafe {
        rate = (*client).rate;
        if !sv_maxRate.is_null() && (*sv_maxRate).integer != 0 {
            if (*sv_maxRate).integer < 1000 {
                Cvar_Set(b"sv_MaxRate\0".as_ptr() as *const c_char, b"1000\0".as_ptr() as *const c_char);
            }
            if (*sv_maxRate).integer < rate {
                rate = (*sv_maxRate).integer;
            }
        }
    }
    rateMsec = (messageSize + HEADER_RATE_BYTES) * 1000 / rate;

    return rateMsec;
}

/*
=======================
SV_SendMessageToClient

Called by SV_SendClientSnapshot and SV_SendClientGameState
=======================
*/
pub unsafe fn SV_SendMessageToClient(msg: *mut msg_t, client: *mut client_t) {
    let mut rateMsec: c_int;

    // MW - my attempt to fix illegible server message errors caused by
    // packet fragmentation of initial snapshot.
    while (*client).state != 0 && (*client).netchan.unsentFragments != 0 {
        // send additional message fragments if the last message
        // was too large to send at once
        Com_Printf(b"[ISM]SV_SendClientGameState() [1] for %s, writing out old fragments\n\0".as_ptr() as *const c_char, (*client).name.as_ptr());
        SV_Netchan_TransmitNextFragment(&mut (*client).netchan);
    }

    // record information about the message
    (*(*client).frames.add(((*client).netchan.outgoingSequence & PACKET_MASK) as usize)).messageSize = (*msg).cursize;
    (*(*client).frames.add(((*client).netchan.outgoingSequence & PACKET_MASK) as usize)).messageSent = svs.time;
    (*(*client).frames.add(((*client).netchan.outgoingSequence & PACKET_MASK) as usize)).messageAcked = -1;

    // send the datagram
    SV_Netchan_Transmit(client, msg);	//msg->cursize, msg->data );

    // set nextSnapshotTime based on rate and requested number of updates

    // local clients get snapshots every frame
    if (*client).netchan.remoteAddress.typ == NA_LOOPBACK || Sys_IsLANAddress((*client).netchan.remoteAddress) != 0 {
        (*client).nextSnapshotTime = svs.time - 1;
        return;
    }

    // normal rate / snapshotMsec calculation
    rateMsec = SV_RateMsec(client, (*msg).cursize);

    if rateMsec < (*client).snapshotMsec {
        // never send more packets than this, no matter what the rate is at
        rateMsec = (*client).snapshotMsec;
        (*client).rateDelayed = 0;
    } else {
        (*client).rateDelayed = 1;
    }

    (*client).nextSnapshotTime = svs.time + rateMsec;

    // don't pile up empty snapshots while connecting
    if (*client).state != CS_ACTIVE {
        // a gigantic connection message may have already put the nextSnapshotTime
        // more than a second away, so don't shorten it
        // do shorten if client is downloading
        #[cfg(feature = "_XBOX")]
        {
            // No downloads on Xbox
            if (*client).nextSnapshotTime < svs.time + 1000 {
                (*client).nextSnapshotTime = svs.time + 1000;
            }
        }
        #[cfg(not(feature = "_XBOX"))]
        {
            if (*client).downloadName[0] == 0 && (*client).nextSnapshotTime < svs.time + 1000 {
                (*client).nextSnapshotTime = svs.time + 1000;
            }
        }
    }
}


/*
=======================
SV_SendClientSnapshot

Also called by SV_FinalMessage

=======================
*/
pub unsafe fn SV_SendClientSnapshot(client: *mut client_t) {
    let mut msg_buf: [u8; MAX_MSGLEN as usize] = [0; MAX_MSGLEN as usize];
    let mut msg: msg_t = core::mem::zeroed();

    if (*client).sentGamedir == 0 {
        // rww - if this is the case then make sure there is an svc_setgame sent before this snap
        let mut i: c_int = 0;

        MSG_Init(&mut msg, msg_buf.as_mut_ptr(), core::mem::size_of_val(&msg_buf) as c_int);

        // have to include this for each message.
        MSG_WriteLong(&mut msg, (*client).lastClientCommand);

        MSG_WriteByte(&mut msg, svc_setgame);

        while (*fs_gamedirvar).string.add(i as usize) as *const u8 != &0u8 {
            MSG_WriteByte(&mut msg, *(*fs_gamedirvar).string.add(i as usize) as u8);
            i += 1;
        }
        MSG_WriteByte(&mut msg, 0);

        // MW - my attempt to fix illegible server message errors caused by
        // packet fragmentation of initial snapshot.
        // rww - reusing this code here
        while (*client).state != 0 && (*client).netchan.unsentFragments != 0 {
            // send additional message fragments if the last message
            // was too large to send at once
            Com_Printf(b"[ISM]SV_SendClientGameState() [1] for %s, writing out old fragments\n\0".as_ptr() as *const c_char, (*client).name.as_ptr());
            SV_Netchan_TransmitNextFragment(&mut (*client).netchan);
        }

        // record information about the message
        (*(*client).frames.add(((*client).netchan.outgoingSequence & PACKET_MASK) as usize)).messageSize = msg.cursize;
        (*(*client).frames.add(((*client).netchan.outgoingSequence & PACKET_MASK) as usize)).messageSent = svs.time;
        (*(*client).frames.add(((*client).netchan.outgoingSequence & PACKET_MASK) as usize)).messageAcked = -1;

        // send the datagram
        SV_Netchan_Transmit(client, &mut msg);	//msg->cursize, msg->data );

        (*client).sentGamedir = 1;
    }

    // build the snapshot
    SV_BuildClientSnapshot(client);

    // bots need to have their snapshots build, but
    // the query them directly without needing to be sent
    if !(*client).gentity.is_null() && ((*(*client).gentity).r.svFlags & SVF_BOT) != 0 {
        return;
    }

    MSG_Init(&mut msg, msg_buf.as_mut_ptr(), core::mem::size_of_val(&msg_buf) as c_int);
    msg.allowoverflow = 1;

    // NOTE, MRE: all server->client messages now acknowledge
    // let the client know which reliable clientCommands we have received
    MSG_WriteLong(&mut msg, (*client).lastClientCommand);

    // (re)send any reliable server commands
    SV_UpdateServerCommandsToClient(client, &mut msg);

    // send over all the relevant entityState_t
    // and the playerState_t
    SV_WriteSnapshotToClient(client, &mut msg);

    // Add any download data if the client is downloading
    #[cfg(not(feature = "_XBOX"))]
    {
        // No downloads on Xbox
        SV_WriteDownloadToClient(client, &mut msg);
    }

    // check for overflow
    if msg.overflowed != 0 {
        Com_Printf(b"WARNING: msg overflowed for %s\n\0".as_ptr() as *const c_char, (*client).name.as_ptr());
        MSG_Clear(&mut msg);
    }

    SV_SendMessageToClient(&mut msg, client);
}


/*
=======================
SV_SendClientMessages
=======================
*/
pub fn SV_SendClientMessages() {
    let mut i: c_int;
    let mut c: *mut client_t;

    unsafe {
        // send a message to each connected client
        i = 0;
        c = svs.clients;
        while i < (*sv_maxclients).integer {
            if (*c).state == 0 {
                i += 1;
                c = c.add(1);
                continue;		// not connected
            }

            if svs.time < (*c).nextSnapshotTime {
                i += 1;
                c = c.add(1);
                continue;		// not time yet
            }

            // send additional message fragments if the last message
            // was too large to send at once
            if (*c).netchan.unsentFragments != 0 {
                (*c).nextSnapshotTime = svs.time +
                    SV_RateMsec(c, (*c).netchan.unsentLength - (*c).netchan.unsentFragmentStart);
                SV_Netchan_TransmitNextFragment(&mut (*c).netchan);
                i += 1;
                c = c.add(1);
                continue;
            }

            // generate and send a new message
            SV_SendClientSnapshot(c);
            i += 1;
            c = c.add(1);
        }
    }
}
