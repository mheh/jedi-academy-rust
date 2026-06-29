// cg_predict.c -- this file generates cg.predicted_player_state by either
// interpolating between snapshots from the server or locally predicting
// ahead the client's movement

//#include "cg_local.h"

use core::ffi::c_int;

// Dependencies from other modules - local stubs for now
// extern crate cg_media; // from cg_media.h
// extern crate g_vehicles; // from ..\game\g_vehicles.h

static mut cg_pmove: pmove_t = pmove_t {
    gent: core::ptr::null_mut(),
    ps: core::ptr::null_mut(),
    trace: None,
    pointcontents: None,
    tracemask: 0,
    noFootsteps: 0,
    cmd: usercmd_t {
        angles: [0; 3],
        buttons: 0,
        weapon: 0,
        forwardmove: 0,
        rightmove: 0,
        upmove: 0,
    },
};

static mut cg_numSolidEntities: c_int = 0;
static mut cg_solidEntities: [*mut centity_t; MAX_ENTITIES_IN_SNAPSHOT] = [core::ptr::null_mut(); MAX_ENTITIES_IN_SNAPSHOT];

/*
====================
CG_BuildSolidList

When a new cg.snap has been set, this function builds a sublist
of the entities that are actually solid, to make for more
efficient collision detection
====================
*/
pub fn CG_BuildSolidList() {
    let mut i: c_int;
    let mut cent: *mut centity_t;
    let mut difference: vec3_t = [0.0; 3];
    let mut dsquared: f32;

    unsafe {
        cg_numSolidEntities = 0;

        if cg.snap.is_null() {
            return;
        }

        for i in 0..(*cg.snap).numEntities {
            if (*cg.snap).entities[i as usize].number < ENTITYNUM_WORLD {
                cent = &mut cg_entities[(*cg.snap).entities[i as usize].number as usize];

                if !(*cent).gent.is_null() && (*(*cent).gent).s.solid != 0 {
                    cg_solidEntities[cg_numSolidEntities as usize] = cent;
                    cg_numSolidEntities += 1;
                }
            }
        }

        dsquared = 5000.0 + 500.0;
        dsquared *= dsquared;

        for i in 0..cg_numpermanents {
            cent = cg_permanents[i as usize];
            VectorSubtract((*cent).lerpOrigin, (*cg.snap).ps.origin, &mut difference);
            if (*cent).currentState.eType == ET_TERRAIN
                || ((difference[0] * difference[0])
                    + (difference[1] * difference[1])
                    + (difference[2] * difference[2]))
                    <= dsquared
            {
                (*cent).currentValid = qtrue;
                if !(*cent).nextState.is_null() && (*(*cent).nextState).solid != 0 {
                    cg_solidEntities[cg_numSolidEntities as usize] = cent;
                    cg_numSolidEntities += 1;
                }
            } else {
                (*cent).currentValid = qfalse;
            }
        }
    }
}

/*
====================
CG_ClipMoveToEntities

====================
*/
pub fn CG_ClipMoveToEntities(
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    skipNumber: c_int,
    mask: c_int,
    tr: *mut trace_t,
) {
    let mut i: c_int;
    let mut x: c_int;
    let mut zd: c_int;
    let mut zu: c_int;
    let mut trace: trace_t;
    let mut ent: *mut entityState_t;
    let mut cmodel: clipHandle_t;
    let mut bmins: vec3_t = [0.0; 3];
    let mut bmaxs: vec3_t = [0.0; 3];
    let mut origin: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut cent: *mut centity_t;

    unsafe {
        for i in 0..cg_numSolidEntities {
            cent = cg_solidEntities[i as usize];
            ent = &mut (*cent).currentState;

            if (*ent).number == skipNumber {
                continue;
            }

            if (*ent).eType == ET_PUSH_TRIGGER {
                continue;
            }
            if (*ent).eType == ET_TELEPORT_TRIGGER {
                continue;
            }

            if (*ent).solid == SOLID_BMODEL {
                // special value for bmodel
                cmodel = cgi_CM_InlineModel((*ent).modelindex);
                VectorCopy((*cent).lerpAngles, &mut angles);

                //Hmm... this would cause traces against brush movers to snap at 20fps (as with the third person camera)...
                //Let's use the lerpOrigin for now and see if it breaks anything...
                //EvaluateTrajectory( &cent->currentState.pos, cg.snap->serverTime, origin );
                VectorCopy((*cent).lerpOrigin, &mut origin);
            } else {
                // encoded bbox
                x = ((*ent).solid & 255) as c_int;
                zd = (((*ent).solid >> 8) & 255) as c_int;
                zu = ((((*ent).solid >> 16) & 255) as c_int) - 32;

                bmins[0] = -x as f32;
                bmins[1] = -x as f32;
                bmaxs[0] = x as f32;
                bmaxs[1] = x as f32;
                bmins[2] = -zd as f32;
                bmaxs[2] = zu as f32;

                cmodel = cgi_CM_TempBoxModel(&bmins, &bmaxs); //,  cent->gent->contents );
                VectorCopy(vec3_origin, &mut angles);
                VectorCopy((*cent).lerpOrigin, &mut origin);
            }

            cgi_CM_TransformedBoxTrace(&mut trace, start, end, mins, maxs, cmodel, mask, &origin, &angles);

            if trace.allsolid != 0 || trace.fraction < (*tr).fraction {
                trace.entityNum = (*ent).number;
                *tr = trace;
            } else if trace.startsolid != 0 {
                (*tr).startsolid = qtrue;
            }
            if (*tr).allsolid != 0 {
                return;
            }
        }
    }
}

/*
================
CG_Trace
================
*/
pub fn CG_Trace(
    result: *mut trace_t,
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    skipNumber: c_int,
    mask: c_int,
    eG2TraceType: EG2_Collision, /*=G2_NOCOLLIDE*/
    useLod: c_int,                /*=0*/
) {
    let mut t: trace_t = unsafe { core::mem::zeroed() };

    unsafe {
        cgi_CM_BoxTrace(&mut t, start, end, mins, maxs, 0, mask);
        t.entityNum = if t.fraction != 1.0 {
            ENTITYNUM_WORLD
        } else {
            ENTITYNUM_NONE
        };
        // check all other solid models
        CG_ClipMoveToEntities(start, mins, maxs, end, skipNumber, mask, &mut t);

        *result = t;
    }
}

/*
================
CG_PointContents
================
*/

const USE_SV_PNT_CONTENTS: c_int = 1;

#[cfg(USE_SV_PNT_CONTENTS = "1")]
pub fn CG_PointContents(point: &vec3_t, passEntityNum: c_int) -> c_int {
    unsafe { gi.pointcontents(point, passEntityNum) }
}

#[cfg(not(USE_SV_PNT_CONTENTS = "1"))]
pub fn CG_PointContents(point: &vec3_t, passEntityNum: c_int) -> c_int {
    let mut i: c_int;
    let mut ent: *mut entityState_t;
    let mut cent: *mut centity_t;
    let mut cmodel: clipHandle_t;
    let mut contents: c_int;

    unsafe {
        contents = cgi_CM_PointContents(point, 0);

        for i in 0..cg_numSolidEntities {
            cent = cg_solidEntities[i as usize];

            ent = &mut (*cent).currentState;

            if (*ent).number == passEntityNum {
                continue;
            }

            if (*ent).solid != SOLID_BMODEL {
                // special value for bmodel
                continue;
            }

            cmodel = cgi_CM_InlineModel((*ent).modelindex);
            if cmodel.is_null() {
                continue;
            }

            contents |= cgi_CM_TransformedPointContents(point, cmodel, (*ent).origin, (*ent).angles);
        }

        contents
    }
}

pub fn CG_SetClientViewAngles(angles: &vec3_t, overrideViewEnt: qboolean) {
    unsafe {
        if (*cg.snap).ps.viewEntity <= 0
            || (*cg.snap).ps.viewEntity >= ENTITYNUM_WORLD
            || overrideViewEnt != 0
        {
            //don't clamp angles when looking through a viewEntity
            for i in 0..3 {
                cg.predicted_player_state.viewangles[i] = angles[i];
                cg.predicted_player_state.delta_angles[i] = 0;
                (*cg.snap).ps.viewangles[i] = angles[i];
                (*cg.snap).ps.delta_angles[i] = 0;
                g_entities[0].client.cast::<*mut client_t>().as_mut().unwrap().pers.cmd_angles[i] =
                    ANGLE2SHORT(angles[i]);
            }
            cgi_SetUserCmdAngles(angles[PITCH], angles[YAW], angles[ROLL]);
        }
    }
}

extern "C" {
    pub fn PM_AdjustAnglesToGripper(gent: *mut gentity_t, cmd: *mut usercmd_t) -> qboolean;
    pub fn PM_AdjustAnglesForSpinningFlip(ent: *mut gentity_t, ucmd: *mut usercmd_t, anglesOnly: qboolean) -> qboolean;
    pub fn G_CheckClampUcmd(ent: *mut gentity_t, ucmd: *mut usercmd_t) -> qboolean;
    pub fn G_IsRidingVehicle(ent: *mut gentity_t) -> *mut Vehicle_t;
}

pub fn CG_CheckModifyUCmd(cmd: *mut usercmd_t, viewangles: *mut vec3_t) -> qboolean {
    let mut overridAngles: qboolean = qfalse;

    unsafe {
        if (*cg.snap).ps.viewEntity > 0 && (*cg.snap).ps.viewEntity < ENTITYNUM_WORLD {
            //controlling something else
            core::ptr::write_bytes(cmd as *mut u8, 0, core::mem::size_of::<usercmd_t>());
            /*
            //to keep pointing in same dir, need to set cmd.angles
            cmd->angles[PITCH] = ANGLE2SHORT( cg.snap->ps.viewangles[PITCH] ) - cg.snap->ps.delta_angles[PITCH];
            cmd->angles[YAW] = ANGLE2SHORT( cg.snap->ps.viewangles[YAW] ) - cg.snap->ps.delta_angles[YAW];
            cmd->angles[ROLL] = 0;
            */
            VectorCopy(g_entities[0].pos4, &mut *viewangles);
            overridAngles = qtrue;
            //CG_SetClientViewAngles( g_entities[cg.snap->ps.viewEntity].client->ps.viewangles, qtrue );
        } else if !G_IsRidingVehicle(&mut g_entities[0]).is_null() {
            overridAngles = qtrue;
            /*
            int vehIndex = g_entities[0].owner->client->ps.vehicleIndex;
            if ( vehIndex != VEHICLE_NONE
                && (vehicleData[vehIndex].type == VH_FIGHTER || (vehicleData[vehIndex].type == VH_SPEEDER )) )
            {//in vehicle flight mode
                float speed = VectorLength( cg.snap->ps.velocity );
                if ( !speed || cg.snap->ps.groundEntityNum != ENTITYNUM_NONE )
                {
                    cmd->rightmove = 0;
                    cmd->angles[PITCH] = 0;
                    cmd->angles[YAW] = ANGLE2SHORT( cg.snap->ps.viewangles[YAW] ) - cg.snap->ps.delta_angles[YAW];
                    CG_SetClientViewAngles( cg.snap->ps.viewangles, qfalse );
                }
            }
            */
        }

        if !g_entities[0].is_null() as *const gentity_t as *const core::ffi::c_void != core::ptr::null()
            && !g_entities[0].client.is_null()
        {
            if PM_AdjustAnglesToGripper(&mut g_entities[0], cmd) == qfalse {
                if PM_AdjustAnglesForSpinningFlip(&mut g_entities[0], cmd, qtrue) != qfalse {
                    CG_SetClientViewAngles(
                        (*(*g_entities[0].client).ps).viewangles,
                        qfalse,
                    );
                    if !viewangles.is_null() {
                        VectorCopy((*(*g_entities[0].client).ps).viewangles, &mut *viewangles);
                        overridAngles = qtrue;
                    }
                }
            } else {
                CG_SetClientViewAngles((*(*g_entities[0].client).ps).viewangles, qfalse);
                if !viewangles.is_null() {
                    VectorCopy((*(*g_entities[0].client).ps).viewangles, &mut *viewangles);
                    overridAngles = qtrue;
                }
            }
            if G_CheckClampUcmd(&mut g_entities[0], cmd) != qfalse {
                CG_SetClientViewAngles((*(*g_entities[0].client).ps).viewangles, qfalse);
                if !viewangles.is_null() {
                    VectorCopy((*(*g_entities[0].client).ps).viewangles, &mut *viewangles);
                    overridAngles = qtrue;
                }
            }
        }
        overridAngles
    }
}

pub fn CG_OnMovingPlat(ps: *mut playerState_t) -> qboolean {
    unsafe {
        if (*ps).groundEntityNum != ENTITYNUM_NONE {
            let es: *mut entityState_t = &mut cg_entities[(*ps).groundEntityNum as usize].currentState;
            if (*es).eType == ET_MOVER {
                //on a mover
                if (*es).pos.trType != TR_STATIONARY {
                    if (*es).pos.trType != TR_LINEAR_STOP && (*es).pos.trType != TR_NONLINEAR_STOP {
                        //a constant mover
                        if !VectorCompare(vec3_origin, (*es).pos.trDelta) {
                            //is moving
                            return qtrue;
                        }
                    } else {
                        //a linear-stop mover
                        if (*es).pos.trTime + (*es).pos.trDuration > cg.time {
                            //still moving
                            return qtrue;
                        }
                    }
                }
            }
        }
        qfalse
    }
}

/*
========================
CG_InterpolatePlayerState

Generates cg.predicted_player_state by interpolating between
cg.snap->player_state and cg.nextFrame->player_state
========================
*/
pub fn CG_InterpolatePlayerState(grabAngles: qboolean) {
    let mut f: f32;
    let mut i: c_int;
    let mut out: *mut playerState_t;
    let mut prev: *mut snapshot_t;
    let mut next: *mut snapshot_t;
    let mut skip: qboolean = qfalse;
    let mut oldOrg: vec3_t = [0.0; 3];

    unsafe {
        out = &mut cg.predicted_player_state;
        prev = cg.snap;
        next = cg.nextSnap;

        VectorCopy((*out).origin, &mut oldOrg);
        *out = (*cg.snap).ps;

        // if we are still allowing local input, short circuit the view angles
        if grabAngles != 0 {
            let mut cmd: usercmd_t = core::mem::zeroed();
            let mut cmdNum: c_int;

            cmdNum = cgi_GetCurrentCmdNumber();
            cgi_GetUserCmd(cmdNum, &mut cmd);

            skip = CG_CheckModifyUCmd(&mut cmd, &mut (*out).viewangles);

            if skip == qfalse {
                //NULL so that it doesn't execute a block of code that must be run from game
                PM_UpdateViewAngles(out, &mut cmd, core::ptr::null_mut());
            }
        }

        // if the next frame is a teleport, we can't lerp to it
        if cg.nextFrameTeleport != 0 {
            return;
        }

        if !next.is_null() && (*next).serverTime > (*prev).serverTime {
            f = (cg.time - (*prev).serverTime) as f32 / ((*next).serverTime - (*prev).serverTime) as f32;

            i = (*next).ps.bobCycle;
            if i < (*prev).ps.bobCycle {
                i += 256; // handle wraparound
            }
            (*out).bobCycle = (*prev).ps.bobCycle as f32 + f * (i - (*prev).ps.bobCycle as c_int) as f32;

            for i in 0..3 {
                (*out).origin[i] =
                    (*prev).ps.origin[i] + f * ((*next).ps.origin[i] - (*prev).ps.origin[i]);
                if grabAngles == qfalse {
                    (*out).viewangles[i] = LerpAngle(
                        (*prev).ps.viewangles[i],
                        (*next).ps.viewangles[i],
                        f,
                    );
                }
                (*out).velocity[i] =
                    (*prev).ps.velocity[i] + f * ((*next).ps.velocity[i] - (*prev).ps.velocity[i]);
            }
        }

        let mut onPlat: bool = false;
        let mut pent: *mut centity_t = core::ptr::null_mut();
        if (*out).groundEntityNum > 0 {
            pent = &mut cg_entities[(*out).groundEntityNum as usize];
            if (*pent).currentState.eType == ET_MOVER {
                onPlat = true;
            }
        }

        if cg.validPPS != 0
            && cg_smoothPlayerPos.value > 0.0
            && cg_smoothPlayerPos.value < 1.0
            && !onPlat
        {
            // 0 = no smoothing, 1 = no movement
            for i in 0..3 {
                (*out).origin[i] = cg_smoothPlayerPos.value * (oldOrg[i] - (*out).origin[i]) + (*out).origin[i];
            }
        } else if onPlat && cg_smoothPlayerPlat.value > 0.0 && cg_smoothPlayerPlat.value < 1.0 {
            //		if (cg.frametime<150)
            //		{
            assert!(!pent.is_null());
            let mut p1: vec3_t = [0.0; 3];
            let mut p2: vec3_t = [0.0; 3];
            let mut vel: vec3_t = [0.0; 3];
            let mut lerpTime: f32;

            EvaluateTrajectory(&(*pent).currentState.pos, (*cg.snap).serverTime, &mut p1);
            if !cg.nextSnap.is_null()
                && (*cg.nextSnap).serverTime > (*cg.snap).serverTime
                && !(*pent).nextState.is_null()
            {
                EvaluateTrajectory(&(*(*pent).nextState).pos, (*cg.nextSnap).serverTime, &mut p2);
                lerpTime = ((*cg.nextSnap).serverTime - (*cg.snap).serverTime) as f32;
            } else {
                EvaluateTrajectory(&(*pent).currentState.pos, (*cg.snap).serverTime + 50, &mut p2);
                lerpTime = 50.0;
            }

            let mut accel: f32 = cg_smoothPlayerPlatAccel.value * cg.frametime as f32 / lerpTime;

            if accel > 20.0 {
                accel = 20.0;
            }

            for i in 0..3 {
                vel[i] = accel * (p2[i] - p1[i]);
            }

            VectorAdd((*out).origin, vel, &mut (*out).origin);

            if cg.validPPS != 0 && cg_smoothPlayerPlat.value > 0.0 && cg_smoothPlayerPlat.value < 1.0 {
                // 0 = no smoothing, 1 = no movement
                for i in 0..3 {
                    (*out).origin[i] = cg_smoothPlayerPlat.value * (oldOrg[i] - (*out).origin[i]) + (*out).origin[i];
                }
            }
            //		}
        }
    }
}

/*
===================
CG_TouchItem
===================
*/
pub fn CG_TouchItem(cent: *mut centity_t) {
    let mut item: *mut gitem_t;

    unsafe {
        // never pick an item up twice in a prediction
        if (*cent).miscTime == cg.time {
            return;
        }

        if BG_PlayerTouchesItem(&cg.predicted_player_state, &(*cent).currentState, cg.time) == qfalse {
            return;
        }

        if BG_CanItemBeGrabbed(&(*cent).currentState, &cg.predicted_player_state) == qfalse {
            return; // can't hold it
        }

        item = &mut bg_itemlist[(*cent).currentState.modelindex as usize];

        // grab it
        AddEventToPlayerstate(
            EV_ITEM_PICKUP,
            (*cent).currentState.modelindex,
            &mut cg.predicted_player_state,
        );

        // remove it from the frame so it won't be drawn
        (*cent).currentState.eFlags |= EF_NODRAW;

        // don't touch it again this prediction
        (*cent).miscTime = cg.time;

        // if its a weapon, give them some predicted ammo so the autoswitch will work
        if (*item).giType == IT_WEAPON {
            let ammotype: c_int = weaponData[(*item).giTag as usize].ammoIndex;
            cg.predicted_player_state.stats[STAT_WEAPONS as usize] |= 1 << (*item).giTag;
            if cg.predicted_player_state.ammo[ammotype as usize] == 0 {
                cg.predicted_player_state.ammo[ammotype as usize] = 1;
            }
        }
    }
}

/*
=========================
CG_TouchTriggerPrediction

Predict push triggers and items
Only called for the last command
=========================
*/
pub fn CG_TouchTriggerPrediction() {
    let mut i: c_int;
    let mut trace: trace_t = unsafe { core::mem::zeroed() };
    let mut ent: *mut entityState_t;
    let mut cmodel: clipHandle_t;
    let mut cent: *mut centity_t;
    let mut spectator: qboolean;

    unsafe {
        // dead clients don't activate triggers
        if cg.predicted_player_state.stats[STAT_HEALTH as usize] <= 0 {
            return;
        }

        spectator = if cg.predicted_player_state.pm_type == PM_SPECTATOR {
            qtrue
        } else {
            qfalse
        };

        if cg.predicted_player_state.pm_type != PM_NORMAL && spectator == qfalse {
            return;
        }

        for i in 0..(*cg.snap).numEntities {
            cent = &mut cg_entities[(*cg.snap).entities[i as usize].number as usize];
            ent = &mut (*cent).currentState;

            if (*ent).eType == ET_ITEM && spectator == qfalse {
                CG_TouchItem(cent);
                continue;
            }

            if (*ent).eType != ET_PUSH_TRIGGER && (*ent).eType != ET_TELEPORT_TRIGGER {
                continue;
            }

            if (*ent).solid != SOLID_BMODEL {
                continue;
            }

            cmodel = cgi_CM_InlineModel((*ent).modelindex);
            if cmodel.is_null() {
                continue;
            }

            cgi_CM_BoxTrace(
                &mut trace,
                cg.predicted_player_state.origin,
                cg.predicted_player_state.origin,
                cg_pmove.mins,
                cg_pmove.maxs,
                cmodel,
                -1,
            );

            if trace.startsolid == 0 {
                continue;
            }

            if (*ent).eType == ET_TELEPORT_TRIGGER {
                cg.hyperspace = qtrue;
            } else {
                // we hit this push trigger
                if spectator != qfalse {
                    continue;
                }

                VectorCopy((*ent).origin2, &mut cg.predicted_player_state.velocity);
            }
        }
    }
}

/*
=================
CG_PredictPlayerState

Generates cg.predicted_player_state for the current cg.time
cg.predicted_player_state is guaranteed to be valid after exiting.

For normal gameplay, it will be the result of predicted usercmd_t on
top of the most recent playerState_t received from the server.

Each new refdef will usually have exactly one new usercmd over the last,
but we have to simulate all unacknowledged commands since the last snapshot
received.  This means that on an internet connection, quite a few
pmoves may be issued each frame.

OPTIMIZE: don't re-simulate unless the newly arrived snapshot playerState_t
differs from the predicted one.

We detect prediction errors and allow them to be decayed off over several frames
to ease the jerk.
=================
*/

extern "C" {
    pub static mut player_locked: qboolean;
}

pub fn CG_PredictPlayerState() {
    let mut cmdNum: c_int;
    let mut current: c_int;
    let mut oldPlayerState: playerState_t;

    unsafe {
        cg.hyperspace = qfalse; // will be set if touching a trigger_teleport

        // if this is the first frame we must guarantee
        // predicted_player_state is valid even if there is some
        // other error condition
        if cg.validPPS == 0 {
            cg.validPPS = qtrue;
            cg.predicted_player_state = (*cg.snap).ps;
        }

        if true {
            //cg_timescale.value >= 1.0f )
            // demo playback just copies the moves
            /*
            if ( (cg.snap->ps.pm_flags & PMF_FOLLOW) ) {
                CG_InterpolatePlayerState( qfalse );
                return;
            }
            */

            // non-predicting local movement will grab the latest angles
            CG_InterpolatePlayerState(qtrue);
            return;
        }

        // prepare for pmove
        //FIXME: is this bad???
        cg_pmove.gent = core::ptr::null_mut();
        cg_pmove.ps = &mut cg.predicted_player_state;
        cg_pmove.trace = Some(CG_Trace);
        cg_pmove.pointcontents = Some(CG_PointContents);
        cg_pmove.tracemask = MASK_PLAYERSOLID;
        cg_pmove.noFootsteps = 0; //( cgs.dmflags & DF_NO_FOOTSTEPS ) > 0;

        // save the state before the pmove so we can detect transitions
        oldPlayerState = cg.predicted_player_state;

        // if we are too far out of date, just freeze
        cmdNum = (*cg.snap).cmdNum;
        current = cgi_GetCurrentCmdNumber();

        if current - cmdNum >= CMD_BACKUP {
            return;
        }

        // get the most recent information we have
        cg.predicted_player_state = (*cg.snap).ps;

        // we should always be predicting at least one frame
        if cmdNum >= current {
            return;
        }

        // run cmds
        loop {
            // check for a prediction error from last frame
            // on a lan, this will often be the exact value
            // from the snapshot, but on a wan we will have
            // to predict several commands to get to the point
            // we want to compare
            if cmdNum == current - 1 {
                let mut delta: vec3_t = [0.0; 3];
                let mut len: f32;

                if cg.thisFrameTeleport != 0 {
                    // a teleport will not cause an error decay
                    VectorClear(&mut cg.predictedError);
                    cg.thisFrameTeleport = qfalse;
                } else {
                    let mut adjusted: vec3_t = [0.0; 3];
                    CG_AdjustPositionForMover(
                        cg.predicted_player_state.origin,
                        cg.predicted_player_state.groundEntityNum,
                        cg.oldTime,
                        &mut adjusted,
                    );

                    VectorSubtract(oldPlayerState.origin, adjusted, &mut delta);
                    len = VectorLength(delta);
                    if len > 0.1 {
                        if cg_errorDecay.integer != 0 {
                            let mut t: c_int;
                            let mut f: f32;

                            t = cg.time - cg.predictedErrorTime;
                            f = (cg_errorDecay.value - t as f32) / cg_errorDecay.value;
                            if f < 0.0 {
                                f = 0.0;
                            }
                            VectorScale(&cg.predictedError, f, &mut cg.predictedError);
                        } else {
                            VectorClear(&mut cg.predictedError);
                        }
                        VectorAdd(&delta, &cg.predictedError, &mut cg.predictedError);
                        cg.predictedErrorTime = cg.oldTime;
                    }
                }
            }

            // if the command can't be gotten because it is
            // too far out of date, the frame is invalid
            // this should never happen, because we check ranges at
            // the top of the function
            cmdNum += 1;
            if cgi_GetUserCmd(cmdNum, &mut cg_pmove.cmd) == qfalse {
                break;
            }

            let ent: *mut gentity_t = &mut g_entities[0]; //cheating and dirty, I know, but this is a SP game so prediction can cheat
            if player_locked != 0
                || (!ent.is_null()
                    && (*ent).s.number == 0
                    && (*ent).aimDebounceTime > level.time)
                || (!ent.is_null()
                    && !(*ent).client.is_null()
                    && (*(*ent).client).ps.pm_time != 0
                    && ((*(*ent).client).ps.pm_flags & PMF_TIME_KNOCKBACK) != 0)
                || (!ent.is_null() && (*ent).forcePushTime > level.time)
            {
                //lock out player control unless dead
                //VectorClear( cg_pmove.cmd.angles );
                cg_pmove.cmd.forwardmove = 0;
                cg_pmove.cmd.rightmove = 0;
                cg_pmove.cmd.buttons = 0;
                cg_pmove.cmd.upmove = 0;
            }
            CG_CheckModifyUCmd(&mut cg_pmove.cmd, core::ptr::null_mut());
            //FIXME: prediction on clients in timescale results in jerky positional translation
            Pmove(&mut cg_pmove);

            // add push trigger movement effects
            CG_TouchTriggerPrediction();

            if !(cmdNum < current) {
                break;
            }
        }

        // adjust for the movement of the groundentity
        CG_AdjustPositionForMover(
            cg.predicted_player_state.origin,
            cg.predicted_player_state.groundEntityNum,
            cg.time,
            &mut cg.predicted_player_state.origin,
        );

        // fire events and other transition triggered things
        CG_TransitionPlayerState(&mut cg.predicted_player_state, &oldPlayerState);
    }
}

// ============================================================================
// LOCAL STUBS AND EXTERNAL DECLARATIONS
// ============================================================================

// Stub types and externs - these would normally come from other compiled modules
// For faithful porting, we declare them as extern or create minimal stubs

pub type qboolean = c_int;
pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

pub type vec3_t = [f32; 3];
pub const vec3_origin: vec3_t = [0.0, 0.0, 0.0];

pub type clipHandle_t = *mut core::ffi::c_void;

pub const MAX_ENTITIES_IN_SNAPSHOT: usize = 256;
pub const ENTITYNUM_WORLD: c_int = (1 << 10);
pub const ENTITYNUM_NONE: c_int = ENTITYNUM_WORLD - 1;
pub const ENTITYNUM_MAX_NOT_NETWORKED: c_int = ENTITYNUM_WORLD;

pub const ET_GENERAL: c_int = 0;
pub const ET_PLAYER: c_int = 1;
pub const ET_ITEM: c_int = 2;
pub const ET_MISSLE: c_int = 3;
pub const ET_SPECIAL: c_int = 4;
pub const ET_SPEAKER: c_int = 5;
pub const ET_PUSH_TRIGGER: c_int = 6;
pub const ET_TELEPORT_TRIGGER: c_int = 7;
pub const ET_INVISIBLE: c_int = 8;
pub const ET_TERRAIN: c_int = 9;
pub const ET_MOVER: c_int = 10;
pub const ET_BEAM: c_int = 11;
pub const ET_PORTAL: c_int = 12;
pub const ET_EVENTS: c_int = 13;

pub const SOLID_NOT: c_int = 0;
pub const SOLID_TRIGGER: c_int = 1;
pub const SOLID_BBOX: c_int = 2;
pub const SOLID_BSP: c_int = 3;
pub const SOLID_BMODEL: c_int = 4;

pub const TR_STATIONARY: c_int = 0;
pub const TR_LINEAR: c_int = 1;
pub const TR_LINEAR_STOP: c_int = 2;
pub const TR_SINE: c_int = 3;
pub const TR_GRAVITY: c_int = 4;
pub const TR_NONLINEAR_STOP: c_int = 5;

pub const MASK_ALL: c_int = -1;
pub const MASK_PLAYERSOLID: c_int = 0;

pub const PM_NORMAL: c_int = 0;
pub const PM_SPECTATOR: c_int = 1;

pub const EV_ITEM_PICKUP: c_int = 0;

pub const EF_NODRAW: c_int = 1;

pub const IT_WEAPON: c_int = 3;

pub const STAT_HEALTH: usize = 0;
pub const STAT_WEAPONS: usize = 1;

pub const PITCH: usize = 0;
pub const YAW: usize = 1;
pub const ROLL: usize = 2;

pub const CMD_BACKUP: c_int = 64;

pub const PMF_TIME_KNOCKBACK: c_int = 1;
pub const PMF_FOLLOW: c_int = 2;

pub type EG2_Collision = c_int;
pub const G2_NOCOLLIDE: EG2_Collision = 0;

#[repr(C)]
pub struct pmove_t {
    pub gent: *mut gentity_t,
    pub ps: *mut playerState_t,
    pub trace: Option<unsafe extern "C" fn(*mut trace_t, &vec3_t, &vec3_t, &vec3_t, &vec3_t, c_int, c_int, EG2_Collision, c_int)>,
    pub pointcontents: Option<unsafe extern "C" fn(&vec3_t, c_int) -> c_int>,
    pub tracemask: c_int,
    pub noFootsteps: c_int,
    pub cmd: usercmd_t,
}

#[repr(C)]
pub struct usercmd_t {
    pub angles: [c_int; 3],
    pub buttons: c_int,
    pub weapon: c_int,
    pub forwardmove: c_int,
    pub rightmove: c_int,
    pub upmove: c_int,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub pos: trajectory_t,
    pub solid: c_int,
    pub origin: vec3_t,
    pub origin2: vec3_t,
    pub angles: vec3_t,
    pub modelindex: c_int,
    pub eFlags: c_int,
    pub otherEntityNum: c_int,
    // ... other fields
}

#[repr(C)]
pub struct trajectory_t {
    pub trType: c_int,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: vec3_t,
    pub trDelta: vec3_t,
}

#[repr(C)]
pub struct centity_t {
    pub s: entityState_t,
    pub lerpOrigin: vec3_t,
    pub lerpAngles: vec3_t,
    pub currentState: entityState_t,
    pub nextState: *mut entityState_t,
    pub currentValid: qboolean,
    pub miscTime: c_int,
    pub gent: *mut gentity_t,
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: qboolean,
    pub startsolid: qboolean,
    pub fraction: f32,
    pub endpos: vec3_t,
    pub plane: cplane_t,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub entityNum: c_int,
    pub lateralFraction: f32,
}

#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub type_: c_int,
    pub signbits: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub commandTime: c_int,
    pub pm_type: c_int,
    pub pm_flags: c_int,
    pub pm_time: c_int,
    pub origin: vec3_t,
    pub velocity: vec3_t,
    pub weaponTime: c_int,
    pub weaponChargeTime: c_int,
    pub gravity: c_int,
    pub speed: c_int,
    pub delta_angles: vec3_t,
    pub groundEntityNum: c_int,
    pub legsTimer: c_int,
    pub legsAnim: c_int,
    pub torsoTimer: c_int,
    pub torsoAnim: c_int,
    pub movementDir: c_int,
    pub grapplePoint: vec3_t,
    pub eFlags: c_int,
    pub eventSequence: c_int,
    pub events: [c_int; 2],
    pub eventParms: [c_int; 2],
    pub externalEvent: c_int,
    pub externalEventParm: c_int,
    pub clientNum: c_int,
    pub weapon: c_int,
    pub weaponstate: c_int,
    pub viewangles: vec3_t,
    pub viewheight: c_int,
    pub bobCycle: f32,
    pub stats: [c_int; 32],
    pub ammo: [c_int; 16],
    pub generic1: c_int,
    pub loopSound: c_int,
    pub jumppad_ent: c_int,
    pub fallingToDeath: c_int,
    pub useEvent: c_int,
    pub vehicleIndex: c_int,
    pub vehicleAngles: vec3_t,
}

#[repr(C)]
pub struct snapshot_t {
    pub snapFlags: c_int,
    pub serverTime: c_int,
    pub ps: playerState_t,
    pub numEntities: c_int,
    pub entities: [entityState_t; 256],
    pub cmdNum: c_int,
}

#[repr(C)]
pub struct gitem_t {
    pub classname: *mut c_char,
    pub pickup_sound: *mut c_char,
    pub world_model: *mut c_char,
    pub view_model: *mut c_char,
    pub icon: *mut c_char,
    pub pickup_name: *mut c_char,
    pub quantity: c_int,
    pub giType: c_int,
    pub giTag: c_int,
    pub precaches: *mut c_char,
    pub sounds: *mut c_char,
}

#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub client: *mut client_t,
    pub gent: *mut gentity_t,
    pub pos4: vec3_t,
    pub aimDebounceTime: c_int,
    pub forcePushTime: c_int,
    pub owner: *mut gentity_t,
}

#[repr(C)]
pub struct client_t {
    pub ps: playerState_t,
    pub pers: clientPersistant_t,
}

#[repr(C)]
pub struct clientPersistant_t {
    pub cmd_angles: [c_int; 3],
    // ... other fields
}

#[repr(C)]
pub struct Vehicle_t {
    pub dummy: c_int,
}

pub struct cvar_t {
    pub value: f32,
    pub integer: c_int,
}

// Global pointers and arrays - declared as extern stubs
extern "C" {
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;
    pub static mut cg_entities: [centity_t; 512];
    pub static mut g_entities: [gentity_t; 512];
    pub static mut cg_permanents: [*mut centity_t; 256];
    pub static mut cg_numpermanents: c_int;
    pub static mut bg_itemlist: [gitem_t; 256];
    pub static mut weaponData: [weaponData_t; 32];
    pub static mut level: level_t;
    pub static mut gi: game_import_t;
    pub static mut cg_smoothPlayerPos: cvar_t;
    pub static mut cg_smoothPlayerPlat: cvar_t;
    pub static mut cg_smoothPlayerPlatAccel: cvar_t;
    pub static mut cg_errorDecay: cvar_t;

    pub fn cgi_CM_BoxTrace(tr: *mut trace_t, start: &vec3_t, end: &vec3_t, mins: &vec3_t, maxs: &vec3_t, model: clipHandle_t, mask: c_int);
    pub fn cgi_CM_TransformedBoxTrace(tr: *mut trace_t, start: &vec3_t, end: &vec3_t, mins: &vec3_t, maxs: &vec3_t, model: clipHandle_t, mask: c_int, origin: &vec3_t, angles: &vec3_t);
    pub fn cgi_CM_InlineModel(index: c_int) -> clipHandle_t;
    pub fn cgi_CM_TempBoxModel(mins: &vec3_t, maxs: &vec3_t) -> clipHandle_t;
    pub fn cgi_CM_PointContents(point: &vec3_t, model: clipHandle_t) -> c_int;
    pub fn cgi_CM_TransformedPointContents(point: &vec3_t, model: clipHandle_t, origin: &vec3_t, angles: &vec3_t) -> c_int;
    pub fn cgi_GetCurrentCmdNumber() -> c_int;
    pub fn cgi_GetUserCmd(cmd: c_int, cmd_out: *mut usercmd_t) -> qboolean;
    pub fn cgi_SetUserCmdAngles(pitch: f32, yaw: f32, roll: f32);
    pub fn VectorSubtract(a: &vec3_t, b: &vec3_t, c: &mut vec3_t);
    pub fn VectorCopy(a: &vec3_t, b: &mut vec3_t);
    pub fn VectorAdd(a: &vec3_t, b: &vec3_t, c: &mut vec3_t);
    pub fn VectorScale(a: &vec3_t, b: f32, c: &mut vec3_t);
    pub fn VectorClear(a: &mut vec3_t);
    pub fn VectorCompare(a: &vec3_t, b: &vec3_t) -> qboolean;
    pub fn VectorLength(a: &vec3_t) -> f32;
    pub fn LerpAngle(from: f32, to: f32, frac: f32) -> f32;
    pub fn ANGLE2SHORT(x: f32) -> c_int;
    pub fn EvaluateTrajectory(tr: *const trajectory_t, atTime: c_int, result: &mut vec3_t);
    pub fn PM_UpdateViewAngles(ps: *mut playerState_t, cmd: *mut usercmd_t, ent: *mut gentity_t);
    pub fn BG_PlayerTouchesItem(ps: *const playerState_t, ent: *const entityState_t, time: c_int) -> qboolean;
    pub fn BG_CanItemBeGrabbed(ent: *const entityState_t, ps: *const playerState_t) -> qboolean;
    pub fn AddEventToPlayerstate(event: c_int, eventParm: c_int, ps: *mut playerState_t);
    pub fn CG_AdjustPositionForMover(in_: &vec3_t, groundent: c_int, atTime: c_int, out: &mut vec3_t);
    pub fn CG_TransitionPlayerState(ps: *mut playerState_t, ops: *mut playerState_t);
    pub fn Pmove(pmove: *mut pmove_t);
}

#[repr(C)]
pub struct cg_t {
    pub snap: *mut snapshot_t,
    pub nextSnap: *mut snapshot_t,
    pub predicted_player_state: playerState_t,
    pub time: c_int,
    pub oldTime: c_int,
    pub frametime: c_int,
    pub hyperspace: qboolean,
    pub nextFrameTeleport: qboolean,
    pub thisFrameTeleport: qboolean,
    pub validPPS: qboolean,
    pub predictedError: vec3_t,
    pub predictedErrorTime: c_int,
}

#[repr(C)]
pub struct cgs_t {
    pub dummy: c_int,
}

#[repr(C)]
pub struct weaponData_t {
    pub ammoIndex: c_int,
}

#[repr(C)]
pub struct level_t {
    pub time: c_int,
}

#[repr(C)]
pub struct game_import_t {
    pub pointcontents: unsafe extern "C" fn(&vec3_t, c_int) -> c_int,
}

use core::ffi::c_char;
