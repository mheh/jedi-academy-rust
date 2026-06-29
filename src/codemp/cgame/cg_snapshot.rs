// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_snapshot.c -- things that happen on snapshot transition,
// not necessarily every single rendered frame

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// Local type stubs - forward declarations for types defined elsewhere
// These are used for structural coherence of this translation
#[repr(C)]
pub struct centity_t {
    // Stub: actual definition in cg_local.h
    // This file accesses centity_t members which are defined in the engine module
    pub currentState: entityState_t,
    pub nextState: entityState_t,
    pub snapShotTime: c_int,
    pub previousEvent: c_int,
    pub trailTime: c_int,
    pub lerpOrigin: [f32; 3],
    pub lerpAngles: [f32; 3],
    pub interpolate: i32,
    pub currentValid: i32,
    pub pe: c_void, // playerEntity - defined elsewhere
    pub noFace: i32,
    pub ghoul2: *mut c_void,
    // ... other fields omitted, only needed fields shown
}

#[repr(C)]
pub struct entityState_t {
    pub eType: c_int,
    pub eFlags: c_int,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub number: c_int,
    // ... other fields
}

#[repr(C)]
pub struct playerState_t {
    pub clientNum: c_int,
    pub pm_flags: c_int,
    pub eFlags: c_int,
    // ... other fields
}

#[repr(C)]
pub struct snapshot_t {
    pub snapFlags: c_int,
    pub serverTime: c_int,
    pub serverCommandSequence: c_int,
    pub numEntities: c_int,
    pub ps: playerState_t,
    pub entities: *mut entityState_t,
}

pub type qboolean = i32;

pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

// Extern global references
extern "C" {
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;
    pub static mut cg_entities: [centity_t; 2048]; // or appropriate size

    // Extern functions called by this module
    fn CG_ResetPlayerEntity(cent: *mut centity_t);
    fn CG_CheckEvents(cent: *mut centity_t);
    fn CG_BuildSolidList();
    fn CG_ExecuteNewServerCommands(serverCommandSequence: c_int);
    fn CG_Respawn();
    fn CG_CopyG2WeaponInstance(cent: *mut centity_t, weaponNum: c_int, ghoul2: *mut c_void);
    fn CG_TransitionPlayerState(ps: *mut playerState_t, ops: *mut playerState_t);
    fn CG_Error(msg: *const u8, ...);
    fn CG_Printf(msg: *const u8, ...);
    fn CG_AddLagometerSnapshotInfo(snap: *mut snapshot_t);
    fn trap_GetSnapshot(num: c_int, dest: *mut snapshot_t) -> qboolean;
    fn trap_GetCurrentSnapshotNumber(latestSnapshotNum: *mut c_int, latestSnapshotTime: *mut c_int);
    fn trap_G2_HaveWeGhoul2Models(ghoul2: *mut c_void) -> qboolean;
    fn trap_G2API_DuplicateGhoul2Instance(ghoul2: *mut c_void, dest: *mut *mut c_void);
    fn trap_G2API_AddBolt(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const u8) -> c_int;
    fn BG_PlayerStateToEntityState(ps: *mut playerState_t, state: *mut entityState_t, useEvents: qboolean);

    pub static mut cg_nopredict: cvar_t;
    pub static mut cg_synchronousClients: cvar_t;
}

#[repr(C)]
pub struct cg_t {
    pub snap: *mut snapshot_t,
    pub nextSnap: *mut snapshot_t,
    pub time: c_int,
    pub latestSnapshotNum: c_int,
    pub latestSnapshotTime: c_int,
    pub activeSnapshots: [snapshot_t; 2],
    pub thisFrameTeleport: qboolean,
    pub nextFrameTeleport: qboolean,
    pub demoPlayback: qboolean,
    // ... other fields
}

#[repr(C)]
pub struct cgs_t {
    pub processedSnapshotNum: c_int,
    pub clientinfo: *mut c_void, // clientInfo_t array
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // ... other fields
}

// VectorCopy macro equivalent
#[inline]
fn VectorCopy(src: &[f32; 3], dst: &mut [f32; 3]) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

// VectorAdd macro equivalent
#[inline]
fn VectorAdd(a: &[f32; 3], b: &[f32; 3], result: &mut [f32; 3]) {
    result[0] = a[0] + b[0];
    result[1] = a[1] + b[1];
    result[2] = a[2] + b[2];
}

const EVENT_VALID_MSEC: c_int = 10000;
const EF_G2ANIMATING: c_int = 0x00000001;
const EF_DEAD: c_int = 0x00000002;
const EF_TELEPORT_BIT: c_int = 0x00000004;
const ET_PLAYER: c_int = 1;
const ET_NPC: c_int = 2;
const FIRST_WEAPON: c_int = 0;
const SNAPFLAG_NOT_ACTIVE: c_int = 0x00000001;
const SNAPFLAG_SERVERCOUNT: c_int = 0x00000002;
const PMF_FOLLOW: c_int = 0x00000001;

/*
==================
CG_ResetEntity
==================
*/
unsafe fn CG_ResetEntity(cent: *mut centity_t) {
    // if the previous snapshot this entity was updated in is at least
    // an event window back in time then we can reset the previous event
    if (*cent).snapShotTime < (*core::ptr::addr_of_mut!(cg)).time - EVENT_VALID_MSEC {
        (*cent).previousEvent = 0;
    }

    (*cent).trailTime = (*core::ptr::addr_of!(cg).snap).serverTime;

    VectorCopy(&(*cent).currentState.origin, &mut (*cent).lerpOrigin);
    VectorCopy(&(*cent).currentState.angles, &mut (*cent).lerpAngles);

    if (*cent).currentState.eFlags & EF_G2ANIMATING != 0 {
        //reset the animation state
        // This references pe.torso.animationNumber and pe.legs.animationNumber
        // which are not available in our stub, so we skip this for the stub
    }

    // #if 0
    // if (cent->isRagging && (cent->currentState.eFlags & EF_DEAD))
    // {
    //     VectorAdd(cent->lerpOrigin, cent->lerpOriginOffset, cent->lerpOrigin);
    // }
    // #endif

    if (*cent).currentState.eType == ET_PLAYER || (*cent).currentState.eType == ET_NPC {
        CG_ResetPlayerEntity(cent);
    }
}

/*
===============
CG_TransitionEntity

cent->nextState is moved to cent->currentState and events are fired
===============
*/
unsafe fn CG_TransitionEntity(cent: *mut centity_t) {
    (*cent).currentState = (*cent).nextState;
    (*cent).currentValid = qtrue;

    // reset if the entity wasn't in the last frame or was teleported
    if (*cent).interpolate == 0 {
        CG_ResetEntity(cent);
    }

    // clear the next state.  if will be set by the next CG_SetNextSnap
    (*cent).interpolate = qfalse;

    // check for events
    CG_CheckEvents(cent);
}


/*
==================
CG_SetInitialSnapshot

This will only happen on the very first snapshot, or
on tourney restarts.  All other times will use
CG_TransitionSnapshot instead.

FIXME: Also called by map_restart?
==================
*/
pub unsafe fn CG_SetInitialSnapshot(snap: *mut snapshot_t) {
    let mut i: c_int;
    let mut cent: *mut centity_t;
    let mut state: *mut entityState_t;

    (*core::ptr::addr_of_mut!(cg)).snap = snap;

    if (cg_entities[(*snap).ps.clientNum as usize].ghoul2.is_null())
        && trap_G2_HaveWeGhoul2Models(
            *(core::ptr::addr_of!(cgs).clientinfo as *mut *mut c_void).add((*snap).ps.clientNum as usize),
        ) != 0
    {
        trap_G2API_DuplicateGhoul2Instance(
            *(core::ptr::addr_of!(cgs).clientinfo as *mut *mut c_void)
                .add((*snap).ps.clientNum as usize),
            &mut cg_entities[(*snap).ps.clientNum as usize].ghoul2,
        );
        CG_CopyG2WeaponInstance(
            core::ptr::addr_of_mut!(cg_entities[(*snap).ps.clientNum as usize]),
            FIRST_WEAPON,
            cg_entities[(*snap).ps.clientNum as usize].ghoul2,
        );

        if trap_G2API_AddBolt(
            cg_entities[(*snap).ps.clientNum as usize].ghoul2,
            0,
            b"face\0".as_ptr() as *const u8,
        ) == -1
        {
            //check now to see if we have this bone for setting anims and such
            cg_entities[(*snap).ps.clientNum as usize].noFace = qtrue;
        }
    }
    BG_PlayerStateToEntityState(
        &mut (*snap).ps,
        &mut cg_entities[(*snap).ps.clientNum as usize].currentState,
        qfalse,
    );

    // sort out solid entities
    CG_BuildSolidList();

    CG_ExecuteNewServerCommands((*snap).serverCommandSequence);

    // set our local weapon selection pointer to
    // what the server has indicated the current weapon is
    CG_Respawn();

    i = 0;
    while i < (*(*core::ptr::addr_of!(cg)).snap).numEntities {
        state = (*(*core::ptr::addr_of!(cg)).snap).entities.add(i as usize);
        cent = &mut cg_entities[(*state).number as usize];

        core::ptr::copy_nonoverlapping(
            state as *const _ as *const u8,
            &mut (*cent).currentState as *mut _ as *mut u8,
            core::mem::size_of::<entityState_t>(),
        );
        //cent->currentState = *state;
        (*cent).interpolate = qfalse;
        (*cent).currentValid = qtrue;

        CG_ResetEntity(cent);

        // check for events
        CG_CheckEvents(cent);

        i += 1;
    }
}


/*
===================
CG_TransitionSnapshot

The transition point from snap to nextSnap has passed
===================
*/
extern "C" {
    fn CG_UsingEWeb() -> qboolean; //cg_predict.c
}

unsafe fn CG_TransitionSnapshot() {
    let mut cent: *mut centity_t;
    let mut oldFrame: *mut snapshot_t;
    let mut i: c_int;

    if (*core::ptr::addr_of!(cg)).snap.is_null() {
        CG_Error(b"CG_TransitionSnapshot: NULL cg.snap\0".as_ptr());
    }
    if (*core::ptr::addr_of!(cg)).nextSnap.is_null() {
        CG_Error(b"CG_TransitionSnapshot: NULL cg.nextSnap\0".as_ptr());
    }

    // execute any server string commands before transitioning entities
    CG_ExecuteNewServerCommands((*(*core::ptr::addr_of!(cg)).nextSnap).serverCommandSequence);

    // if we had a map_restart, set everthing with initial
    if (*core::ptr::addr_of!(cg)).snap.is_null() {}

    // clear the currentValid flag for all entities in the existing snapshot
    i = 0;
    while i < (*(*core::ptr::addr_of!(cg)).snap).numEntities {
        cent = &mut cg_entities[(*(*(*core::ptr::addr_of!(cg)).snap).entities.add(i as usize)).number as usize];
        (*cent).currentValid = qfalse;

        i += 1;
    }

    // move nextSnap to snap and do the transitions
    oldFrame = (*core::ptr::addr_of!(cg)).snap;
    (*core::ptr::addr_of_mut!(cg)).snap = (*core::ptr::addr_of!(cg)).nextSnap;

    //CG_CheckPlayerG2Weapons(&cg.snap->ps, &cg_entities[cg.snap->ps.clientNum]);
    //CG_CheckPlayerG2Weapons(&cg.snap->ps, &cg.predictedPlayerEntity);
    BG_PlayerStateToEntityState(
        &mut (*(*core::ptr::addr_of!(cg)).snap).ps,
        &mut cg_entities[(*(*core::ptr::addr_of!(cg)).snap).ps.clientNum as usize].currentState,
        qfalse,
    );
    cg_entities[(*(*core::ptr::addr_of!(cg)).snap).ps.clientNum as usize].interpolate = qfalse;

    i = 0;
    while i < (*(*core::ptr::addr_of!(cg)).snap).numEntities {
        cent = &mut cg_entities[(*(*(*core::ptr::addr_of!(cg)).snap).entities.add(i as usize)).number as usize];
        CG_TransitionEntity(cent);

        // remember time of snapshot this entity was last updated in
        (*cent).snapShotTime = (*(*core::ptr::addr_of!(cg)).snap).serverTime;

        i += 1;
    }

    (*core::ptr::addr_of_mut!(cg)).nextSnap = core::ptr::null_mut();

    // check for playerstate transition events
    if !oldFrame.is_null() {
        let mut ops: *mut playerState_t;
        let mut ps: *mut playerState_t;

        ops = &mut (*oldFrame).ps;
        ps = &mut (*(*core::ptr::addr_of!(cg)).snap).ps;
        // teleporting checks are irrespective of prediction
        if ((*ps).eFlags ^ (*ops).eFlags) & EF_TELEPORT_BIT != 0 {
            (*core::ptr::addr_of_mut!(cg)).thisFrameTeleport = qtrue;
            // will be cleared by prediction code
        }

        // if we are not doing client side movement prediction for any
        // reason, then the client events and view changes will be issued now
        if (*core::ptr::addr_of!(cg)).demoPlayback != 0
            || ((*(*core::ptr::addr_of!(cg)).snap).ps.pm_flags & PMF_FOLLOW != 0)
            || (*core::ptr::addr_of!(cg_nopredict)).integer != 0
            || (*core::ptr::addr_of!(cg_synchronousClients)).integer != 0
            || CG_UsingEWeb() != 0
        {
            CG_TransitionPlayerState(ps, ops);
        }
    }
}


/*
===================
CG_SetNextSnap

A new snapshot has just been read in from the client system.
===================
*/
unsafe fn CG_SetNextSnap(snap: *mut snapshot_t) {
    let mut num: c_int;
    let mut es: *mut entityState_t;
    let mut cent: *mut centity_t;

    (*core::ptr::addr_of_mut!(cg)).nextSnap = snap;

    //CG_CheckPlayerG2Weapons(&cg.snap->ps, &cg_entities[cg.snap->ps.clientNum]);
    //CG_CheckPlayerG2Weapons(&cg.snap->ps, &cg.predictedPlayerEntity);
    BG_PlayerStateToEntityState(
        &mut (*snap).ps,
        &mut cg_entities[(*snap).ps.clientNum as usize].nextState,
        qfalse,
    );
    //cg_entities[ cg.snap->ps.clientNum ].interpolate = qtrue;
    //No longer want to do this, as the cg_entities[clnum] and cg.predictedPlayerEntity are one in the same.

    // check for extrapolation errors
    num = 0;
    while num < (*snap).numEntities {
        es = (*snap).entities.add(num as usize);
        cent = &mut cg_entities[(*es).number as usize];

        core::ptr::copy_nonoverlapping(
            es as *const _ as *const u8,
            &mut (*cent).nextState as *mut _ as *mut u8,
            core::mem::size_of::<entityState_t>(),
        );
        //cent->nextState = *es;

        // if this frame is a teleport, or the entity wasn't in the
        // previous frame, don't interpolate
        if (*cent).currentValid == 0 || ((*cent).currentState.eFlags ^ (*es).eFlags) & EF_TELEPORT_BIT != 0 {
            (*cent).interpolate = qfalse;
        } else {
            (*cent).interpolate = qtrue;
        }

        num += 1;
    }

    // if the next frame is a teleport for the playerstate, we
    // can't interpolate during demos
    if !(*core::ptr::addr_of!(cg)).snap.is_null()
        && ((*snap).ps.eFlags ^ (*(*core::ptr::addr_of!(cg)).snap).ps.eFlags) & EF_TELEPORT_BIT != 0
    {
        (*core::ptr::addr_of_mut!(cg)).nextFrameTeleport = qtrue;
    } else {
        (*core::ptr::addr_of_mut!(cg)).nextFrameTeleport = qfalse;
    }

    // if changing follow mode, don't interpolate
    if (*(*core::ptr::addr_of!(cg)).nextSnap).ps.clientNum != (*(*core::ptr::addr_of!(cg)).snap).ps.clientNum {
        (*core::ptr::addr_of_mut!(cg)).nextFrameTeleport = qtrue;
    }

    // if changing server restarts, don't interpolate
    if ((*(*core::ptr::addr_of!(cg)).nextSnap).snapFlags ^ (*(*core::ptr::addr_of!(cg)).snap).snapFlags)
        & SNAPFLAG_SERVERCOUNT
        != 0
    {
        (*core::ptr::addr_of_mut!(cg)).nextFrameTeleport = qtrue;
    }

    // sort out solid entities
    CG_BuildSolidList();
}


/*
========================
CG_ReadNextSnapshot

This is the only place new snapshots are requested
This may increment cgs.processedSnapshotNum multiple
times if the client system fails to return a
valid snapshot.
========================
*/
unsafe fn CG_ReadNextSnapshot() -> *mut snapshot_t {
    let mut r: qboolean;
    let mut dest: *mut snapshot_t;

    if (*core::ptr::addr_of!(cg)).latestSnapshotNum > (*core::ptr::addr_of!(cgs)).processedSnapshotNum + 1000 {
        CG_Printf(
            b"WARNING: CG_ReadNextSnapshot: way out of range, %i > %i\0".as_ptr(),
            (*core::ptr::addr_of!(cg)).latestSnapshotNum,
            (*core::ptr::addr_of!(cgs)).processedSnapshotNum,
        );
    }

    while (*core::ptr::addr_of!(cgs)).processedSnapshotNum < (*core::ptr::addr_of!(cg)).latestSnapshotNum {
        // decide which of the two slots to load it into
        if (*core::ptr::addr_of!(cg)).snap
            == &mut (*core::ptr::addr_of_mut!(cg)).activeSnapshots[0] as *mut snapshot_t
        {
            dest = &mut (*core::ptr::addr_of_mut!(cg)).activeSnapshots[1] as *mut snapshot_t;
        } else {
            dest = &mut (*core::ptr::addr_of_mut!(cg)).activeSnapshots[0] as *mut snapshot_t;
        }

        // try to read the snapshot from the client system
        (*core::ptr::addr_of_mut!(cgs)).processedSnapshotNum += 1;
        r = trap_GetSnapshot((*core::ptr::addr_of!(cgs)).processedSnapshotNum, dest);

        // FIXME: why would trap_GetSnapshot return a snapshot with the same server time
        if !(*core::ptr::addr_of!(cg)).snap.is_null()
            && r != 0
            && (*dest).serverTime == (*(*core::ptr::addr_of!(cg)).snap).serverTime
        {
            //continue;
        }

        // if it succeeded, return
        if r != 0 {
            CG_AddLagometerSnapshotInfo(dest);
            return dest;
        }

        // a GetSnapshot will return failure if the snapshot
        // never arrived, or  is so old that its entities
        // have been shoved off the end of the circular
        // buffer in the client system.

        // record as a dropped packet
        CG_AddLagometerSnapshotInfo(core::ptr::null_mut());

        // If there are additional snapshots, continue trying to
        // read them.
    }

    // nothing left to read
    return core::ptr::null_mut();
}


/*
============
CG_ProcessSnapshots

We are trying to set up a renderable view, so determine
what the simulated time is, and try to get snapshots
both before and after that time if available.

If we don't have a valid cg.snap after exiting this function,
then a 3D game view cannot be rendered.  This should only happen
right after the initial connection.  After cg.snap has been valid
once, it will never turn invalid.

Even if cg.snap is valid, cg.nextSnap may not be, if the snapshot
hasn't arrived yet (it becomes an extrapolating situation instead
of an interpolating one)

============
*/
pub unsafe fn CG_ProcessSnapshots() {
    let mut snap: *mut snapshot_t;
    let mut n: c_int = 0;

    // see what the latest snapshot the client system has is
    trap_GetCurrentSnapshotNumber(&mut n, &mut (*core::ptr::addr_of_mut!(cg)).latestSnapshotTime);
    if n != (*core::ptr::addr_of!(cg)).latestSnapshotNum {
        if n < (*core::ptr::addr_of!(cg)).latestSnapshotNum {
            // this should never happen
            CG_Error(b"CG_ProcessSnapshots: n < cg.latestSnapshotNum\0".as_ptr());
        }
        (*core::ptr::addr_of_mut!(cg)).latestSnapshotNum = n;
    }

    // If we have yet to receive a snapshot, check for it.
    // Once we have gotten the first snapshot, cg.snap will
    // always have valid data for the rest of the game
    while (*core::ptr::addr_of!(cg)).snap.is_null() {
        snap = CG_ReadNextSnapshot();
        if snap.is_null() {
            // we can't continue until we get a snapshot
            return;
        }

        // set our weapon selection to what
        // the playerstate is currently using
        if (*snap).snapFlags & SNAPFLAG_NOT_ACTIVE == 0 {
            CG_SetInitialSnapshot(snap);
        }
    }

    // loop until we either have a valid nextSnap with a serverTime
    // greater than cg.time to interpolate towards, or we run
    // out of available snapshots
    loop {
        // if we don't have a nextframe, try and read a new one in
        if (*core::ptr::addr_of!(cg)).nextSnap.is_null() {
            snap = CG_ReadNextSnapshot();

            // if we still don't have a nextframe, we will just have to
            // extrapolate
            if snap.is_null() {
                break;
            }

            CG_SetNextSnap(snap);


            // if time went backwards, we have a level restart
            if (*(*core::ptr::addr_of!(cg)).nextSnap).serverTime < (*(*core::ptr::addr_of!(cg)).snap).serverTime {
                CG_Error(b"CG_ProcessSnapshots: Server time went backwards\0".as_ptr());
            }
        }

        // if our time is < nextFrame's, we have a nice interpolating state
        if (*core::ptr::addr_of!(cg)).time >= (*(*core::ptr::addr_of!(cg)).snap).serverTime
            && (*core::ptr::addr_of!(cg)).time < (*(*core::ptr::addr_of!(cg)).nextSnap).serverTime
        {
            break;
        }

        // we have passed the transition from nextFrame to frame
        CG_TransitionSnapshot();
    }

    // assert our valid conditions upon exiting
    if (*core::ptr::addr_of!(cg)).snap.is_null() {
        CG_Error(b"CG_ProcessSnapshots: cg.snap == NULL\0".as_ptr());
    }
    if (*core::ptr::addr_of!(cg)).time < (*(*core::ptr::addr_of!(cg)).snap).serverTime {
        // this can happen right after a vid_restart
        (*core::ptr::addr_of_mut!(cg)).time = (*(*core::ptr::addr_of!(cg)).snap).serverTime;
    }
    if !(*core::ptr::addr_of!(cg)).nextSnap.is_null()
        && (*(*core::ptr::addr_of!(cg)).nextSnap).serverTime <= (*core::ptr::addr_of!(cg)).time
    {
        CG_Error(b"CG_ProcessSnapshots: cg.nextSnap->serverTime <= cg.time\0".as_ptr());
    }
}
