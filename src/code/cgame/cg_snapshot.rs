// cg_snapshot.c -- things that happen on snapshot transition,
// not necessarily every single frame

// this line must stay at top so the whole PCH thing works...
// #include "cg_headers.h"
// #include "cg_local.h"

use core::ffi::c_int;

// Local type stubs for structural coherence
// Full definitions should come from cg_local.h and cg_public.h

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    _private: [u8; 0],
}

#[repr(C)]
pub struct playerState_t {
    pub eFlags: c_int,
    _private: [u8; 0],
}

#[repr(C)]
pub struct playerEntity_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct snapshot_s {
    pub snapFlags: c_int,
    pub ping: c_int,
    pub serverTime: c_int,
    pub areamask: [u8; 32],
    pub cmdNum: c_int,
    pub ps: playerState_t,
    pub numEntities: c_int,
    pub entities: [entityState_t; 512], // MAX_ENTITIES_IN_SNAPSHOT
    pub numConfigstringChanges: c_int,
    pub configstringNum: c_int,
    pub numServerCommands: c_int,
    pub serverCommandSequence: c_int,
}

pub type snapshot_t = snapshot_s;

#[repr(C)]
pub struct centity_s {
    pub currentState: entityState_t,
    pub nextState: *const entityState_t,
    pub interpolate: c_int,
    pub currentValid: c_int,
    pub muzzleFlashTime: c_int,
    pub altFire: c_int,
    pub previousEvent: c_int,
    pub miscTime: c_int,
    pub pe: playerEntity_t,
    pub lerpOrigin: [f32; 3],
    pub lerpAngles: [f32; 3],
    pub renderAngles: [f32; 3],
    pub rotValue: f32,
    pub snapShotTime: c_int,
    pub gent: *mut core::ffi::c_void,
}

pub type centity_t = centity_s;

#[repr(C)]
pub struct cg_s {
    pub clientFrame: c_int,
    pub levelShot: c_int,
    pub latestSnapshotNum: c_int,
    pub latestSnapshotTime: c_int,
    pub processedSnapshotNum: c_int,
    pub snap: *mut snapshot_t,
    pub nextSnap: *mut snapshot_t,
    pub frameInterpolation: f32,
    pub thisFrameTeleport: c_int,
    pub nextFrameTeleport: c_int,
    pub frametime: c_int,
    pub time: c_int,
    pub oldTime: c_int,
    pub timelimitWarnings: c_int,
    pub renderingThirdPerson: c_int,
    pub hyperspace: c_int,
    pub predicted_player_state: playerState_t,
    pub validPPS: c_int,
    pub predictedErrorTime: c_int,
    pub predictedError: [f32; 3],
    pub stepChange: f32,
    pub stepTime: c_int,
    pub duckChange: f32,
    pub duckTime: c_int,
    pub landChange: f32,
    pub landTime: c_int,
    pub weaponSelect: c_int,
    pub saberAnimLevelPending: c_int,
    pub autoAngles: [f32; 3],
    pub autoAxis: [[f32; 3]; 3],
    pub autoAnglesFast: [f32; 3],
    pub autoAxisFast: [[f32; 3]; 3],
    pub activeSnapshots: [snapshot_t; 2],
    _private: [u8; 0],
}

pub type cg_t = cg_s;

extern "C" {
    // Global variables
    pub static mut cg: cg_t;
    pub static mut cg_entities: [centity_t; 512]; // MAX_ENTITIES proxy

    // Constants
    static ET_PLAYER: c_int;
    static EF_TELEPORT_BIT: c_int;
    static qtrue: c_int;
    static qfalse: c_int;

    // External functions
    fn CG_ResetPlayerEntity(cent: *mut centity_t);
    fn CG_CheckEvents(cent: *mut centity_t);
    fn CG_ExecuteNewServerCommands(serverCommandSequence: c_int);
    fn CG_Respawn();
    fn CG_Error(msg: *const u8);
    fn CG_TransitionPlayerState(ps: *const playerState_t, oldPs: *const playerState_t);
    fn cgi_GetSnapshot(num: c_int, snap: *mut snapshot_t) -> c_int;
    fn cgi_GetCurrentSnapshotNumber(latestNum: *mut c_int, latestTime: *mut c_int);
    fn CG_LinkCentsToGents();
    fn CG_Init_CG();
    fn CG_InitLocalEntities();
    fn CG_InitMarkPolys();
    fn memset(s: *mut core::ffi::c_void, c: c_int, n: usize) -> *mut core::ffi::c_void;

    // VectorCopy macro equivalent
    fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
}

/*
==================
CG_ResetEntity
==================
*/
#[no_mangle]
pub unsafe extern "C" fn CG_ResetEntity(cent: *mut centity_t) {
    // if an event is set, assume it is new enough to use
    // if the event had timed out, it would have been cleared
    (*cent).previousEvent = 0;

    //	(*cent).trailTime = (*cg.snap).serverTime;

    VectorCopy(
        &(*cent).currentState.origin as *const _,
        &mut (*cent).lerpOrigin as *mut _,
    );
    VectorCopy(
        &(*cent).currentState.angles as *const _,
        &mut (*cent).lerpAngles as *mut _,
    );
    if (*cent).currentState.eType == ET_PLAYER {
        CG_ResetPlayerEntity(cent);
    }
}

/*
===============
CG_TransitionEntity

cent->nextState is moved to cent->currentState and events are fired
===============
*/
#[no_mangle]
pub unsafe extern "C" fn CG_TransitionEntity(cent: *mut centity_t) {
    if !(*cent).nextState.is_null() {
        (*cent).currentState = *(*cent).nextState;
    }
    (*cent).currentValid = qtrue;

    // reset if the entity wasn't in the last frame or was teleported
    if (*cent).interpolate == 0 {
        CG_ResetEntity(cent);
    }

    // clear the next state.  if will be set by the next CG_SetNextSnap
    (*cent).interpolate = qfalse;

    if (*cent).currentState.number != 0 {
        // check for events
        CG_CheckEvents(cent);
    }
}

/*
==================
CG_SetInitialSnapshot

This will only happen on the very first snapshot, or
on tourney restarts.  All other times will use
CG_TransitionSnapshot instead.
==================
*/
#[no_mangle]
pub unsafe extern "C" fn CG_SetInitialSnapshot(snap: *mut snapshot_t) {
    let mut i: c_int = 0;
    let mut cent: *mut centity_t;
    let mut state: *mut entityState_t;

    cg.snap = snap;

    // sort out solid entities
    //CG_BuildSolidList();

    CG_ExecuteNewServerCommands((*snap).serverCommandSequence);

    // set our local weapon selection pointer to
    // what the server has indicated the current weapon is
    CG_Respawn();

    i = 0;
    while i < (*snap).numEntities {
        state = &mut (*snap).entities[i as usize] as *mut entityState_t;
        cent = &mut cg_entities[(*state).number as usize] as *mut centity_t;

        (*cent).currentState = *state;
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
#[no_mangle]
pub unsafe extern "C" fn CG_TransitionSnapshot() {
    let mut cent: *mut centity_t;
    let mut oldFrame: *mut snapshot_t;
    let mut i: c_int = 0;

    if cg.snap.is_null() {
        CG_Error(b"CG_TransitionSnapshot: NULL cg.snap\0".as_ptr());
    }
    if cg.nextSnap.is_null() {
        CG_Error(b"CG_TransitionSnapshot: NULL cg.nextSnap\0".as_ptr());
    }

    // execute any server string commands before transitioning entities
    CG_ExecuteNewServerCommands((*cg.nextSnap).serverCommandSequence);

    // clear the currentValid flag for all entities in the existing snapshot
    i = 0;
    while i < (*cg.snap).numEntities {
        cent = &mut cg_entities[(*cg.snap).entities[i as usize].number as usize] as *mut centity_t;
        (*cent).currentValid = qfalse;
        i += 1;
    }

    // move nextSnap to snap and do the transitions
    oldFrame = cg.snap;
    cg.snap = cg.nextSnap;

    // sort out solid entities
    //CG_BuildSolidList();

    i = 0;
    while i < (*cg.snap).numEntities {
        if 1 != 0 {
            //cg.snap->entities[ i ].number != 0 ) // I guess the player adds his/her events elsewhere, so doing this also gives us double events for the player!
            cent = &mut cg_entities[(*cg.snap).entities[i as usize].number as usize] as *mut centity_t;
            CG_TransitionEntity(cent);
        }
        i += 1;
    }

    cg.nextSnap = core::ptr::null_mut();

    // check for playerstate transition events
    if !oldFrame.is_null() {
        // if we are not doing client side movement prediction for any
        // reason, then the client events and view changes will be issued now
        //if ( cg_timescale.value >= 1.0f )
        {
            CG_TransitionPlayerState(&(*cg.snap).ps, &(*oldFrame).ps);
        }
    }
}

/*
===============
CG_SetEntityNextState

Determine if the entity can be interpolated between the states
present in cg.snap and cg,nextSnap
===============
*/
#[no_mangle]
pub unsafe extern "C" fn CG_SetEntityNextState(
    cent: *mut centity_t,
    state: *mut entityState_t,
) {
    (*cent).nextState = state;

    // since we can't interpolate ghoul2 stuff from one frame to another, I'm just going to copy the ghoul2 info directly into the current state now
    //	CGhoul2Info *currentModel = &state->ghoul2[1];
    //	(*cent).gent->ghoul2 = state->ghoul2;
    //	CGhoul2Info *newModel = &(*cent).gent->ghoul2[1];

    // if this frame is a teleport, or the entity wasn't in the
    // previous frame, don't interpolate
    if (*cent).currentValid == 0
        || ((*cent).currentState.eFlags ^ (*state).eFlags) & EF_TELEPORT_BIT != 0
    {
        (*cent).interpolate = qfalse;
    } else {
        (*cent).interpolate = qtrue;
    }
}

/*
===================
CG_SetNextSnap

A new snapshot has just been read in from the client system.
===================
*/
#[no_mangle]
pub unsafe extern "C" fn CG_SetNextSnap(snap: *mut snapshot_t) {
    let mut num: c_int = 0;
    let mut es: *mut entityState_t;
    let mut cent: *mut centity_t;

    cg.nextSnap = snap;

    // check for extrapolation errors
    num = 0;
    while num < (*snap).numEntities {
        es = &mut (*snap).entities[num as usize] as *mut entityState_t;
        cent = &mut cg_entities[(*es).number as usize] as *mut centity_t;
        CG_SetEntityNextState(cent, es);
        num += 1;
    }

    // if the next frame is a teleport for the playerstate,
    if !cg.snap.is_null()
        && ((*snap).ps.eFlags ^ (*cg.snap).ps.eFlags) & EF_TELEPORT_BIT != 0
    {
        cg.nextFrameTeleport = qtrue;
    } else {
        cg.nextFrameTeleport = qfalse;
    }
}

/*
========================
CG_ReadNextSnapshot

This is the only place new snapshots are requested
This may increment cg.processedSnapshotNum multiple
times if the client system fails to return a
valid snapshot.
========================
*/
#[no_mangle]
pub unsafe extern "C" fn CG_ReadNextSnapshot() -> *mut snapshot_t {
    let mut r: c_int;
    let mut dest: *mut snapshot_t;

    while cg.processedSnapshotNum < cg.latestSnapshotNum {
        // decide which of the two slots to load it into
        if cg.snap as *const _ == &cg.activeSnapshots[0] as *const _ {
            dest = &mut cg.activeSnapshots[1] as *mut snapshot_t;
        } else {
            dest = &mut cg.activeSnapshots[0] as *mut snapshot_t;
        }

        // try to read the snapshot from the client system
        cg.processedSnapshotNum += 1;
        r = cgi_GetSnapshot(cg.processedSnapshotNum, dest);

        // if it succeeded, return
        if r != 0 {
            return dest;
        }

        // a GetSnapshot will return failure if the snapshot
        // never arrived, or  is so old that its entities
        // have been shoved off the end of the circular
        // buffer in the client system.

        // record as a dropped packet
        //		CG_AddLagometerSnapshotInfo( NULL );

        // If there are additional snapshots, continue trying to
        // read them.
    }

    // nothing left to read
    core::ptr::null_mut()
}

/*
=================
CG_RestartLevel

A tournement restart will clear everything, but doesn't
require a reload of all the media
=================
*/
unsafe extern "C" fn CG_RestartLevel() {
    let mut snapshotNum: c_int = 0;
    let mut r: c_int;

    snapshotNum = cg.processedSnapshotNum;

    memset(
        cg_entities.as_mut_ptr() as *mut core::ffi::c_void,
        0,
        core::mem::size_of_val(&cg_entities),
    );
    CG_Init_CG();

    CG_LinkCentsToGents();
    CG_InitLocalEntities();
    CG_InitMarkPolys();

    // regrab the first snapshot of the restart

    cg.processedSnapshotNum = snapshotNum;
    r = cgi_GetSnapshot(
        cg.processedSnapshotNum,
        &mut cg.activeSnapshots[0] as *mut snapshot_t,
    );
    if r == 0 {
        CG_Error(b"cgi_GetSnapshot failed on restart\0".as_ptr());
    }

    CG_SetInitialSnapshot(&mut cg.activeSnapshots[0] as *mut snapshot_t);
    cg.time = (*cg.snap).serverTime;
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
#[no_mangle]
pub unsafe extern "C" fn CG_ProcessSnapshots() {
    let mut snap: *mut snapshot_t;
    let mut n: c_int = 0;
    let mut newSnapshots: c_int;

    // see what the latest snapshot the client system has is
    cgi_GetCurrentSnapshotNumber(&mut n, &mut cg.latestSnapshotTime);
    if n != cg.latestSnapshotNum {
        if n < cg.latestSnapshotNum {
            // this should never happen
            CG_Error(b"CG_ProcessSnapshots: n < cg.latestSnapshotNum\0".as_ptr());
        }
        cg.latestSnapshotNum = n;
        newSnapshots = qtrue;
    } else {
        newSnapshots = qfalse;
    }

    // If we have yet to receive a snapshot, check for it.
    // Once we have gotten the first snapshot, cg.snap will
    // always have valid data for the rest of the game
    if cg.snap.is_null() {
        snap = CG_ReadNextSnapshot();
        if snap.is_null() {
            // we can't continue until we get a snapshot
            return;
        }

        // set our weapon selection to what
        // the playerstate is currently using
        CG_SetInitialSnapshot(snap);
    }

    // loop until we either have a valid nextSnap with a serverTime
    // greater than cg.time to interpolate towards, or we run
    // out of available snapshots
    loop {
        // if we don't have a nextframe, try to read a new one in
        if cg.nextSnap.is_null() {
            snap = CG_ReadNextSnapshot();

            // if we still don't have a nextframe, we will just have to
            // extrapolate
            if snap.is_null() {
                break;
            }

            CG_SetNextSnap(snap);

            // if time went backwards, we have a level restart
            if (*cg.nextSnap).serverTime < (*cg.snap).serverTime {
                // restart the level
                CG_RestartLevel();
                continue; // we might also get a nextsnap
            }
        }

        // if our time is < nextFrame's, we have a nice interpolating state
        if cg.time < (*cg.nextSnap).serverTime {
            break;
        }

        // we have passed the transition from nextFrame to frame
        CG_TransitionSnapshot();
    }

    if (*cg.snap).serverTime > cg.time {
        cg.time = (*cg.snap).serverTime;
        #[cfg(_DEBUG)]
        {
            // Com_Printf("CG_ProcessSnapshots: cg.snap->serverTime > cg.time");
        }
    }
    if !cg.nextSnap.is_null() && (*cg.nextSnap).serverTime <= cg.time {
        cg.time = (*cg.nextSnap).serverTime - 1;
        #[cfg(_DEBUG)]
        {
            // Com_Printf("CG_ProcessSnapshots: cg.nextSnap->serverTime <= cg.time");
        }
    }
    // assert our valid conditions upon exiting
    if cg.snap.is_null() {
        CG_Error(b"CG_ProcessSnapshots: cg.snap == NULL\0".as_ptr());
    }
    if (*cg.snap).serverTime > cg.time {
        CG_Error(b"CG_ProcessSnapshots: cg.snap->serverTime > cg.time\0".as_ptr());
    }
    if !cg.nextSnap.is_null() && (*cg.nextSnap).serverTime <= cg.time {
        CG_Error(b"CG_ProcessSnapshots: cg.nextSnap->serverTime <= cg.time\0".as_ptr());
    }
}
