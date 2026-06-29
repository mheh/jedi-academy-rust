// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_predict.c -- this file generates cg.predictedPlayerState by either
// interpolating between snapshots from the server or locally predicting
// ahead the client's movement.
// It also handles local physics interaction, like fragments bouncing off walls

use core::ffi::{c_int, c_void, c_char};

// Stub imports from cg_local.h and related headers
// These are placeholder declarations for the dependencies
// The actual implementations exist elsewhere in the codebase

use crate::codemp::cgame::cg_local_h::{
    cg_entities, cg, cgs, cg_numpermanents, cg_permanents,
    cg_showmiss, pmove_msec, pmove_fixed, cg_nopredict,
    cg_synchronousClients, cg_errorDecay, cg_showVehMiss,
};

// External function declarations
extern "C" {
    fn trap_CM_InlineModel(modelindex: c_int) -> clipHandle_t;
    fn trap_CM_TempBoxModel(mins: *const [f32; 3], maxs: *const [f32; 3]) -> clipHandle_t;
    fn trap_CM_BoxTrace(result: *mut trace_t, start: *const [f32; 3], end: *const [f32; 3],
                        mins: *const [f32; 3], maxs: *const [f32; 3], model: clipHandle_t, mask: c_int);
    fn trap_CM_TransformedBoxTrace(result: *mut trace_t, start: *const [f32; 3], end: *const [f32; 3],
                                   mins: *const [f32; 3], maxs: *const [f32; 3], model: clipHandle_t,
                                   mask: c_int, origin: *const [f32; 3], angles: *const [f32; 3]);
    fn trap_CM_PointContents(point: *const [f32; 3], model: c_int) -> c_int;
    fn trap_CM_TransformedPointContents(point: *const [f32; 3], model: clipHandle_t,
                                        origin: *const [f32; 3], angles: *const [f32; 3]) -> c_int;
    fn trap_GetCurrentCmdNumber() -> c_int;
    fn trap_GetUserCmd(cmdNumber: c_int, cmd: *mut usercmd_t);
    fn trap_Cvar_Set(var_name: *const core::ffi::c_char, value: *const core::ffi::c_char);
    fn trap_G2API_AddBolt(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const core::ffi::c_char) -> c_int;

    fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    fn VectorClear(v: *mut [f32; 3]);
    fn VectorAdd(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorScale(v: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    fn VectorLength(v: *const [f32; 3]) -> f32;
    fn VectorCompare(a: *const [f32; 3], b: *const [f32; 3]) -> c_int;

    fn BG_VehicleAdjustBBoxForOrientation(veh: *mut Vehicle_t, origin: *const [f32; 3],
                                          mins: *mut [f32; 3], maxs: *mut [f32; 3],
                                          clientNum: c_int, tracemask: c_int,
                                          localTrace: *const c_void);
    fn BG_EvaluateTrajectory(traj: *const trajectory_t, atTime: c_int, result: *mut [f32; 3]);
    fn BG_PlayerTouchesItem(ps: *mut playerState_t, s: *const entityState_t, atTime: c_int) -> c_int;
    fn BG_CanItemBeGrabbed(gametype: c_int, ent: *const entityState_t, ps: *const playerState_t) -> c_int;
    fn BG_AddPredictableEventToPlayerstate(event: c_int, eventParm: c_int, ps: *mut playerState_t);
    fn BG_TouchJumpPad(ps: *mut playerState_t, ent: *const entityState_t);
    fn PM_UpdateViewAngles(ps: *mut playerState_t, cmd: *const usercmd_t);
    fn Pmove(pmove: *mut pmove_t);

    fn LerpAngle(from: f32, to: f32, frac: f32) -> f32;
    fn AngleSubtract(angle1: f32, angle2: f32) -> f32;
    fn vectoangles(vec: *const [f32; 3], angles: *mut [f32; 3]);

    fn CG_G2TraceCollide(trace: *mut trace_t, mins: *const [f32; 3], maxs: *const [f32; 3],
                         start: *const [f32; 3], end: *const [f32; 3]);
    fn CG_Printf(msg: *const core::ffi::c_char);
    fn CG_AdjustPositionForMover(in_: *const [f32; 3], groundEntityNum: c_int,
                                 atTime: c_int, toTime: c_int, out: *mut [f32; 3]);
    fn CG_TransitionPlayerState(ps: *mut playerState_t, ops: *const playerState_t);
    fn CG_Cube(mins: *const [f32; 3], maxs: *const [f32; 3], color: *const [f32; 3], alpha: f32);

    fn Com_Error(level: c_int, msg: *const core::ffi::c_char);
    fn memset(ptr: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// Type stubs for unimplemented types
pub type clipHandle_t = c_int;

#[repr(C)]
pub struct trace_t {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: f32,
    pub endpos: [f32; 3],
    pub plane: plane_t,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub entityNum: c_int,
}

#[repr(C)]
pub struct plane_t {
    pub normal: [f32; 3],
    pub dist: f32,
}

#[repr(C)]
pub struct usercmd_t {
    pub serverTime: c_int,
    pub buttons: c_int,
    pub forwardmove: c_int,
    pub rightmove: c_int,
    pub upmove: c_int,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub solid: c_int,
    pub pos: trajectory_t,
    pub apos: trajectory_t,
    pub trDelta: [f32; 3],
    pub trBase: [f32; 3],
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub owner: c_int,
    pub modelindex: c_int,
    pub genericenemyindex: c_int,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    pub legsFlip: c_int,
    pub torsoFlip: c_int,
    pub clientNum: c_int,
    pub saberMove: c_int,
    pub forceFrame: c_int,
    pub eFlags: c_int,
    pub brokenLimbs: c_int,
    pub m_iVehicleNum: c_int,
    pub NPC_class: c_int,
    pub groundEntityNum: c_int,
}

#[repr(C)]
pub struct trajectory_t {
    pub trType: c_int,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: [f32; 3],
    pub trDelta: [f32; 3],
}

#[repr(C)]
pub struct playerState_t {
    pub commandTime: c_int,
    pub pm_type: c_int,
    pub bobCycle: c_int,
    pub origin: [f32; 3],
    pub velocity: [f32; 3],
    pub viewangles: [f32; 3],
    pub groundEntityNum: c_int,
    pub stats: [c_int; 16],
    pub persistant: [c_int; 16],
    pub ammo: [c_int; 16],
    pub weapon: c_int,
    pub clientNum: c_int,
    pub eventSequence: c_int,
    pub pm_flags: c_int,
    pub externalEvent: c_int,
    pub externalEventParm: c_int,
    pub entityEventSequence: c_int,
    pub saberLockFrame: c_int,
    pub saberHolstered: c_int,
    pub fd: forceData_t,
    pub m_iVehicleNum: c_int,
    pub jumppad_frame: c_int,
    pub jumppad_ent: c_int,
    pub pmove_framecount: c_int,
    pub slopeRecalcTime: c_int,
    pub saberLockEnemy: c_int,
    pub saberLockTime: c_int,
    pub saberInFlight: c_int,
    pub saberEntityNum: c_int,
    pub saberAnimLevel: c_int,
    pub vehOrientation: [f32; 3],
    pub vehSurfaces: c_int,
    pub vehBoarding: c_int,
}

#[repr(C)]
pub struct forceData_t {
    pub forceSide: c_int,
    pub saberAnimLevelBase: c_int,
    pub saberAnimLevel: c_int,
    pub forcePowersActive: c_int,
    pub forceMindtrickTargetIndex: c_int,
    pub forceMindtrickTargetIndex2: c_int,
    pub forceMindtrickTargetIndex3: c_int,
    pub forceMindtrickTargetIndex4: c_int,
}

#[repr(C)]
pub struct snapshot_t {
    pub serverTime: c_int,
    pub numEntities: c_int,
    pub ps: playerState_t,
    pub vps: playerState_t,
    pub entities: [entityState_t; 128],
}

#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    pub nextState: entityState_t,
    pub lerpOrigin: [f32; 3],
    pub lerpAngles: [f32; 3],
    pub currentValid: c_int,
    pub miscTime: c_int,
    pub ghoul2: *mut c_void,
    pub m_pVehicle: *mut Vehicle_t,
    pub playerState: *mut playerState_t,
    pub modelScale: [f32; 3],
    pub localAnimIndex: c_int,
}

#[repr(C)]
pub struct Vehicle_t {
    pub m_vOrientation: *mut [f32; 3],
    pub m_iRemovedSurfaces: c_int,
    pub m_ucmd: usercmd_t,
    pub m_vPrevOrientation: [f32; 3],
    pub m_iBoarding: c_int,
}

#[repr(C)]
pub struct pmove_t {
    pub ps: *mut playerState_t,
    pub cmd: usercmd_t,
    pub trace: Option<extern "C" fn(*mut trace_t, *const [f32; 3], *const [f32; 3], *const [f32; 3], *const [f32; 3], c_int, c_int)>,
    pub pointcontents: Option<extern "C" fn(*const [f32; 3], c_int) -> c_int>,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub tracemask: c_int,
    pub pmove_fixed: c_int,
    pub pmove_msec: c_int,
    pub noFootsteps: c_int,
    pub ghoul2: *mut c_void,
    pub g2Bolts_LFoot: c_int,
    pub g2Bolts_RFoot: c_int,
    pub animations: *mut c_void,
    pub gametype: c_int,
    pub debugMelee: c_int,
    pub stepSlideFix: c_int,
    pub noSpecMove: c_int,
    pub nonHumanoid: c_int,
    pub baseEnt: *mut bgEntity_t,
    pub entSize: usize,
    pub debugLevel: c_int,
    pub modelScale: [f32; 3],
}

#[repr(C)]
pub struct bgEntity_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct gitem_t {
    pub giType: c_int,
    pub giTag: c_int,
    pub quantity: c_int,
}

#[repr(C)]
pub struct clientInfo_t {
    pub saber: [saber_t; 2],
}

#[repr(C)]
pub struct saber_t {
    pub numBlades: c_int,
    pub model: [c_int; 16],
}

// Constants
const MAX_ENTITIES_IN_SNAPSHOT: usize = 256;
const MAX_GENTITIES: usize = 2048;
const MAX_CLIENTS: c_int = 64;
const ENTITYNUM_NONE: c_int = 2047;
const ENTITYNUM_WORLD: c_int = 2048;
const MAX_PS_EVENTS: c_int = 4;
const CMD_BACKUP: c_int = 64;
const DEFAULT_MINS_2: f32 = -24.0;
const DEFAULT_MAXS_2: f32 = 32.0;
const SOLID_BMODEL: c_int = -1;

// Entity types
const ET_GENERAL: c_int = 0;
const ET_PLAYER: c_int = 1;
const ET_ITEM: c_int = 2;
const ET_MISSILE: c_int = 3;
const ET_MOVER: c_int = 4;
const ET_BEAM: c_int = 5;
const ET_PORTAL: c_int = 6;
const ET_SPEAKER: c_int = 7;
const ET_PUSH_TRIGGER: c_int = 8;
const ET_TELEPORT_TRIGGER: c_int = 9;
const ET_INVISIBLE: c_int = 10;
const ET_NPC: c_int = 11;
const ET_TERRAIN: c_int = 12;

// Entity flags
const EF_NODRAW: c_int = 0x1;
const EF_ITEMPLACEHOLDER: c_int = 0x2;
const EF_DEAD: c_int = 0x4;

// Entity classes
const CLASS_VEHICLE: c_int = 13;

// Stat indices
const STAT_HEALTH: usize = 0;
const STAT_ARMOR: usize = 1;
const STAT_WEAPONS: usize = 2;
const STAT_HOLDABLE_ITEMS: usize = 3;

// Persistant stat indices
const PERS_TEAM: usize = 0;

// Content masks
const CONTENTS_BODY: c_int = 0x20000000;
const MASK_PLAYERSOLID: c_int = 0xFFFFFFFF;

// Player movement types
const PM_NORMAL: c_int = 0;
const PM_SPECTATOR: c_int = 1;
const PM_DEAD: c_int = 2;
const PM_FREEZE: c_int = 3;
const PM_INTERMISSION: c_int = 4;
const PM_JETPACK: c_int = 5;
const PM_FLOAT: c_int = 6;

// PMF flags
const PMF_FOLLOW: c_int = 1;

// Button flags
const BUTTON_TALK: c_int = 0x1000;

// Weapon constants
const WP_EMPLACED_GUN: c_int = 20;
const WP_NONE: c_int = 0;

// Game types
const GT_CTF: c_int = 4;
const GT_CTY: c_int = 5;

// Teams
const TEAM_RED: c_int = 0;
const TEAM_BLUE: c_int = 1;
const TEAM_SPECTATOR: c_int = 2;

// Error codes
const ERR_DROP: c_int = 1;

// Powerup constants
const PW_REDFLAG: c_int = 0;
const PW_BLUEFLAG: c_int = 1;
const PW_FORCE_ENLIGHTENED_LIGHT: c_int = 2;
const PW_FORCE_ENLIGHTENED_DARK: c_int = 3;

// Force sides
const FORCE_LIGHTSIDE: c_int = 0;
const FORCE_DARKSIDE: c_int = 1;

// Item types
const IT_WEAPON: c_int = 1;
const IT_ARMOR: c_int = 2;
const IT_HEALTH: c_int = 3;
const IT_POWERUP: c_int = 4;
const IT_HOLDABLE: c_int = 5;

// Dmflags
const DF_NO_FOOTSTEPS: c_int = 0x100;

// Events
const EV_ITEM_PICKUP: c_int = 1;

const DEFAULT_PMOVE: pmove_t = pmove_t {
    ps: 0 as *mut playerState_t,
    cmd: usercmd_t { serverTime: 0, buttons: 0, forwardmove: 0, rightmove: 0, upmove: 0 },
    trace: None,
    pointcontents: None,
    mins: [0.0; 3],
    maxs: [0.0; 3],
    tracemask: 0,
    pmove_fixed: 0,
    pmove_msec: 0,
    noFootsteps: 0,
    ghoul2: 0 as *mut c_void,
    g2Bolts_LFoot: 0,
    g2Bolts_RFoot: 0,
    animations: 0 as *mut c_void,
    gametype: 0,
    debugMelee: 0,
    stepSlideFix: 0,
    noSpecMove: 0,
    nonHumanoid: 0,
    baseEnt: 0 as *mut bgEntity_t,
    entSize: 0,
    debugLevel: 0,
    modelScale: [0.0; 3],
};

static mut cg_pmove: pmove_t = DEFAULT_PMOVE;

static mut cg_numSolidEntities: c_int = 0;
static mut cg_solidEntities: [*mut centity_t; MAX_ENTITIES_IN_SNAPSHOT] = [0 as *mut centity_t; MAX_ENTITIES_IN_SNAPSHOT];
static mut cg_numTriggerEntities: c_int = 0;
static mut cg_triggerEntities: [*mut centity_t; MAX_ENTITIES_IN_SNAPSHOT] = [0 as *mut centity_t; MAX_ENTITIES_IN_SNAPSHOT];

//is this client piloting this veh?
#[inline(always)]
fn CG_Piloting(vehNum: c_int) -> c_int {
    let veh: *mut centity_t;

    if vehNum == 0 {
        return 0;
    }

    unsafe {
        veh = &mut cg_entities[vehNum as usize];

        if (*veh).currentState.owner != cg.predictedPlayerState.clientNum {
            //the owner should be the current pilot
            return 0;
        }

        return 1;
    }
}

/*
====================
CG_BuildSolidList

When a new cg.snap has been set, this function builds a sublist
of the entities that are actually solid, to make for more
efficient collision detection
====================
*/
unsafe fn CG_BuildSolidList() {
    let mut i: c_int;
    let mut cent: *mut centity_t;
    let snap: *mut snapshot_t;
    let ent: *mut entityState_t;
    let mut difference: [f32; 3] = [0.0; 3];
    let mut dsquared: f32;

    cg_numSolidEntities = 0;
    cg_numTriggerEntities = 0;

    if !cg.nextSnap.is_null() && cg.nextFrameTeleport == 0 && cg.thisFrameTeleport == 0 {
        snap = cg.nextSnap;
    } else {
        snap = cg.snap;
    }

    i = 0;
    while i < (*snap).numEntities {
        cent = &mut cg_entities[(*snap).entities[i as usize].number as usize];
        ent = &mut (*cent).currentState;

        if (*ent).eType == ET_ITEM || (*ent).eType == ET_PUSH_TRIGGER || (*ent).eType == ET_TELEPORT_TRIGGER {
            cg_solidEntities[cg_numTriggerEntities as usize] = cent;
            cg_numTriggerEntities += 1;
            i += 1;
            continue;
        }

        if (*cent).nextState.solid != 0 {
            cg_solidEntities[cg_numSolidEntities as usize] = cent;
            cg_numSolidEntities += 1;
            i += 1;
            continue;
        }
        i += 1;
    }

    //rww - Horrible, terrible, awful hack.
    //We don't send your client entity from the server,
    //so it isn't added into the solid list from the snapshot,
    //and in addition, it has no solid data. So we will force
    //adding it in based on a hardcoded player bbox size.
    //This will cause issues if the player box size is ever
    //changed..
    if cg_numSolidEntities < MAX_ENTITIES_IN_SNAPSHOT as c_int {
        let playerMins: [f32; 3] = [-15.0, -15.0, DEFAULT_MINS_2];
        let playerMaxs: [f32; 3] = [15.0, 15.0, DEFAULT_MAXS_2];
        let mut i: c_int;
        let mut j: c_int;
        let mut k: c_int;

        i = playerMaxs[0] as c_int;
        if i < 1 {
            i = 1;
        }
        if i > 255 {
            i = 255;
        }

        // z is not symetric
        j = (-playerMins[2]) as c_int;
        if j < 1 {
            j = 1;
        }
        if j > 255 {
            j = 255;
        }

        // and z playerMaxs can be negative...
        k = (playerMaxs[2] + 32.0) as c_int;
        if k < 1 {
            k = 1;
        }
        if k > 255 {
            k = 255;
        }

        cg_solidEntities[cg_numSolidEntities as usize] = &mut cg_entities[cg.predictedPlayerState.clientNum as usize];
        (*cg_solidEntities[cg_numSolidEntities as usize]).currentState.solid = (k << 16) | (j << 8) | i;

        cg_numSolidEntities += 1;
    }

    dsquared = /*RMG_distancecull.value*/5000.0 + 500.0;
    dsquared *= dsquared;

    i = 0;
    while i < cg_numpermanents {
        cent = cg_permanents[i as usize];
        VectorSubtract(&(*cent).lerpOrigin, &(*snap).ps.origin, &mut difference);
        if (*cent).currentState.eType == ET_TERRAIN ||
            ((difference[0] * difference[0]) + (difference[1] * difference[1]) + (difference[2] * difference[2])) <= dsquared
        {
            (*cent).currentValid = 1;
            if (*cent).nextState.solid != 0 {
                cg_solidEntities[cg_numSolidEntities as usize] = cent;
                cg_numSolidEntities += 1;
            }
        } else {
            (*cent).currentValid = 0;
        }
        i += 1;
    }
}

#[inline(always)]
fn CG_VehicleClipCheck(ignored: *mut centity_t, trace: *mut trace_t) -> c_int {
    if trace.is_null() || (*trace).entityNum < 0 || (*trace).entityNum >= ENTITYNUM_WORLD {
        //it's alright then
        return 1;
    }

    if (*ignored).currentState.eType != ET_PLAYER && (*ignored).currentState.eType != ET_NPC {
        //can't possibly be valid then
        return 1;
    }

    if (*ignored).currentState.m_iVehicleNum != 0 {
        //see if the ignore ent is a vehicle/rider - if so, see if the ent we supposedly hit is a vehicle/rider.
        //if they belong to each other, we don't want to collide them.
        let otherguy: *mut centity_t;

        unsafe {
            otherguy = &mut cg_entities[(*trace).entityNum as usize];

            if (*otherguy).currentState.eType != ET_PLAYER && (*otherguy).currentState.eType != ET_NPC {
                //can't possibly be valid then
                return 1;
            }

            if (*otherguy).currentState.m_iVehicleNum != 0 {
                //alright, both of these are either a vehicle or a player who is on a vehicle
                let mut index: c_int;

                if (*ignored).currentState.eType == ET_PLAYER ||
                    ((*ignored).currentState.eType == ET_NPC && (*ignored).currentState.NPC_class != CLASS_VEHICLE)
                {
                    //must be a player or NPC riding a vehicle
                    index = (*ignored).currentState.m_iVehicleNum;
                } else {
                    //a vehicle
                    index = (*ignored).currentState.m_iVehicleNum - 1;
                }

                if index == (*otherguy).currentState.number {
                    //this means we're riding or being ridden by this guy, so don't collide
                    return 0;
                } else {
                    //see if I'm hitting one of my own passengers
                    if (*otherguy).currentState.eType == ET_PLAYER ||
                        ((*otherguy).currentState.eType == ET_NPC && (*otherguy).currentState.NPC_class != CLASS_VEHICLE)
                    {
                        //must be a player or NPC riding a vehicle
                        if (*otherguy).currentState.m_iVehicleNum == (*ignored).currentState.number {
                            //this means we're other guy is riding the ignored ent
                            return 0;
                        }
                    }
                }
            }
        }
    }

    return 1;
}

//rww - I'm disabling this warning for this function. It complains about oldTrace but as you can see it
//always gets set before use, and I am not wasting CPU memsetting it to shut the compiler up.
#[allow(non_snake_case)]
unsafe fn CG_ClipMoveToEntities(start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3],
                                skipNumber: c_int, mask: c_int, tr: *mut trace_t, g2Check: c_int) {
    let mut i: c_int;
    let mut x: c_int;
    let mut zd: c_int;
    let mut zu: c_int;
    let mut trace: trace_t;
    let mut oldTrace: trace_t;
    let ent: *mut entityState_t;
    let cmodel: clipHandle_t;
    let mut bmins: [f32; 3] = [0.0; 3];
    let mut bmaxs: [f32; 3] = [0.0; 3];
    let mut origin: [f32; 3] = [0.0; 3];
    let mut angles: [f32; 3] = [0.0; 3];
    let cent: *mut centity_t;
    let mut ignored: *mut centity_t = 0 as *mut centity_t;

    if skipNumber != -1 && skipNumber != ENTITYNUM_NONE {
        ignored = &mut cg_entities[skipNumber as usize];
    }

    i = 0;
    while i < cg_numSolidEntities {
        cent = cg_solidEntities[i as usize];
        let ent_ref = &mut (*cent).currentState;

        if ent_ref.number == skipNumber {
            i += 1;
            continue;
        }

        if ent_ref.number > MAX_CLIENTS &&
            (ent_ref.genericenemyindex - MAX_GENTITIES as c_int == cg.predictedPlayerState.clientNum ||
             ent_ref.genericenemyindex - MAX_GENTITIES as c_int == cg.predictedVehicleState.clientNum)
        {
            //rww - method of keeping objects from colliding in client-prediction (in case of ownership)
            i += 1;
            continue;
        }

        if ent_ref.solid == SOLID_BMODEL {
            // special value for bmodel
            cmodel = trap_CM_InlineModel(ent_ref.modelindex);
            VectorCopy(&(*cent).lerpAngles, &mut angles);
            BG_EvaluateTrajectory(&ent_ref.pos, cg.physicsTime, &mut origin);
        } else {
            // encoded bbox
            x = (ent_ref.solid & 255) as c_int;
            zd = ((ent_ref.solid >> 8) & 255) as c_int;
            zu = ((ent_ref.solid >> 16) & 255) as c_int - 32;

            bmins[0] = -x as f32;
            bmins[1] = -x as f32;
            bmaxs[0] = x as f32;
            bmaxs[1] = x as f32;
            bmins[2] = -zd as f32;
            bmaxs[2] = zu as f32;

            if ent_ref.eType == ET_NPC && ent_ref.NPC_class == CLASS_VEHICLE && !(*cent).m_pVehicle.is_null() {
                //try to dynamically adjust his bbox dynamically, if possible
                let old = (*(*cent).m_pVehicle).m_vOrientation;
                (*(*cent).m_pVehicle).m_vOrientation = &mut (*cent).lerpAngles[0];
                BG_VehicleAdjustBBoxForOrientation(
                    (*cent).m_pVehicle,
                    &(*cent).lerpOrigin,
                    &mut bmins,
                    &mut bmaxs,
                    ent_ref.number,
                    MASK_PLAYERSOLID,
                    0 as *const c_void,
                );
                (*(*cent).m_pVehicle).m_vOrientation = old;
            }

            cmodel = trap_CM_TempBoxModel(&bmins, &bmaxs);
            VectorCopy(&[0.0; 3], &mut angles);

            VectorCopy(&(*cent).lerpOrigin, &mut origin);
        }

        trap_CM_TransformedBoxTrace(&mut trace, start, end, mins, maxs, cmodel, mask, &origin, &angles);
        trace.entityNum = if trace.fraction != 1.0 { ent_ref.number } else { ENTITYNUM_NONE };

        if g2Check != 0 || (!ignored.is_null() && (*ignored).currentState.m_iVehicleNum != 0) {
            //keep these older variables around for a bit, incase we need to replace them in the Ghoul2 Collision check
            //or in the vehicle owner trace check
            oldTrace = *tr;
        }

        if trace.allsolid != 0 || trace.fraction < (*tr).fraction {
            trace.entityNum = ent_ref.number;
            *tr = trace;
        } else if trace.startsolid != 0 {
            (*tr).startsolid = 1;

            //rww 12-02-02
            (*tr).entityNum = trace.entityNum;
            trace.entityNum = ent_ref.number;
        }
        if (*tr).allsolid != 0 {
            if !ignored.is_null() && (*ignored).currentState.m_iVehicleNum != 0 {
                trace.entityNum = ent_ref.number;
                if CG_VehicleClipCheck(ignored, &mut trace) != 0 {
                    //this isn't our vehicle, we're really stuck
                    return;
                } else {
                    //it's alright, keep going
                    trace = oldTrace;
                    *tr = trace;
                }
            } else {
                return;
            }
        }

        if g2Check != 0 {
            if trace.entityNum == ent_ref.number && !(*cent).ghoul2.is_null() {
                CG_G2TraceCollide(&mut trace, mins, maxs, start, end);

                if trace.entityNum == ENTITYNUM_NONE {
                    //g2 trace failed, so put it back where it was.
                    trace = oldTrace;
                    *tr = trace;
                }
            }
        }

        if !ignored.is_null() && (*ignored).currentState.m_iVehicleNum != 0 {
            //see if this is the vehicle we hit
            let hit = &mut cg_entities[trace.entityNum as usize];
            if CG_VehicleClipCheck(ignored, &mut trace) == 0 {
                //looks like it
                trace = oldTrace;
                *tr = trace;
            } else if (*hit).currentState.eType == ET_MISSILE && (*hit).currentState.owner == (*ignored).currentState.number {
                //hack, don't hit own missiles
                trace = oldTrace;
                *tr = trace;
            }
        }
        i += 1;
    }
}

/*
================
CG_Trace
================
*/
unsafe fn CG_Trace(result: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3],
                   skipNumber: c_int, mask: c_int) {
    let mut t: trace_t;

    trap_CM_BoxTrace(&mut t, start, end, mins, maxs, 0 as clipHandle_t, mask);
    t.entityNum = if t.fraction != 1.0 { ENTITYNUM_WORLD } else { ENTITYNUM_NONE };
    // check all other solid models
    CG_ClipMoveToEntities(start, mins, maxs, end, skipNumber, mask, &mut t, 0);

    *result = t;
}

/*
================
CG_G2Trace
================
*/
unsafe fn CG_G2Trace(result: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3],
                     skipNumber: c_int, mask: c_int) {
    let mut t: trace_t;

    trap_CM_BoxTrace(&mut t, start, end, mins, maxs, 0 as clipHandle_t, mask);
    t.entityNum = if t.fraction != 1.0 { ENTITYNUM_WORLD } else { ENTITYNUM_NONE };
    // check all other solid models
    CG_ClipMoveToEntities(start, mins, maxs, end, skipNumber, mask, &mut t, 1);

    *result = t;
}

/*
================
CG_PointContents
================
*/
unsafe fn CG_PointContents(point: *const [f32; 3], passEntityNum: c_int) -> c_int {
    let mut i: c_int;
    let ent: *mut entityState_t;
    let cent: *mut centity_t;
    let cmodel: clipHandle_t;
    let mut contents: c_int;

    contents = trap_CM_PointContents(point, 0);

    i = 0;
    while i < cg_numSolidEntities {
        cent = cg_solidEntities[i as usize];

        let ent_ref = &mut (*cent).currentState;

        if ent_ref.number == passEntityNum {
            i += 1;
            continue;
        }

        if ent_ref.solid != SOLID_BMODEL {
            // special value for bmodel
            i += 1;
            continue;
        }

        cmodel = trap_CM_InlineModel(ent_ref.modelindex);
        if cmodel.is_null() as c_int == 0 {
            i += 1;
            continue;
        }

        contents |= trap_CM_TransformedPointContents(point, cmodel, &ent_ref.origin, &ent_ref.angles);
        i += 1;
    }

    return contents;
}


/*
========================
CG_InterpolatePlayerState

Generates cg.predictedPlayerState by interpolating between
cg.snap->player_state and cg.nextFrame->player_state
========================
*/
unsafe fn CG_InterpolatePlayerState(grabAngles: c_int) {
    let mut f: f32;
    let mut i: c_int;
    let out: *mut playerState_t;
    let prev: *mut snapshot_t;
    let next: *mut snapshot_t;

    out = &mut cg.predictedPlayerState;
    prev = cg.snap;
    next = cg.nextSnap;

    *out = (*prev).ps;

    // if we are still allowing local input, short circuit the view angles
    if grabAngles != 0 {
        let mut cmd: usercmd_t;
        let mut cmdNum: c_int;

        cmdNum = trap_GetCurrentCmdNumber();
        trap_GetUserCmd(cmdNum, &mut cmd);

        PM_UpdateViewAngles(out, &cmd);
    }

    // if the next frame is a teleport, we can't lerp to it
    if cg.nextFrameTeleport != 0 {
        return;
    }

    if next.is_null() || (*next).serverTime <= (*prev).serverTime {
        return;
    }

    f = (cg.time - (*prev).serverTime) as f32 / ((*next).serverTime - (*prev).serverTime) as f32;

    i = (*next).ps.bobCycle;
    if i < (*prev).ps.bobCycle {
        i += 256;  // handle wraparound
    }
    (*out).bobCycle = ((*prev).ps.bobCycle as f32 + f * (i - (*prev).ps.bobCycle as f32)) as c_int;

    i = 0;
    while i < 3 {
        (*out).origin[i as usize] = (*prev).ps.origin[i as usize] + f * ((*next).ps.origin[i as usize] - (*prev).ps.origin[i as usize]);
        if grabAngles == 0 {
            (*out).viewangles[i as usize] = LerpAngle((*prev).ps.viewangles[i as usize], (*next).ps.viewangles[i as usize], f);
        }
        (*out).velocity[i as usize] = (*prev).ps.velocity[i as usize] + f * ((*next).ps.velocity[i as usize] - (*prev).ps.velocity[i as usize]);
        i += 1;
    }

}

unsafe fn CG_InterpolateVehiclePlayerState(grabAngles: c_int) {
    let mut f: f32;
    let mut i: c_int;
    let out: *mut playerState_t;
    let prev: *mut snapshot_t;
    let next: *mut snapshot_t;

    out = &mut cg.predictedVehicleState;
    prev = cg.snap;
    next = cg.nextSnap;

    *out = (*prev).vps;

    // if we are still allowing local input, short circuit the view angles
    if grabAngles != 0 {
        let mut cmd: usercmd_t;
        let mut cmdNum: c_int;

        cmdNum = trap_GetCurrentCmdNumber();
        trap_GetUserCmd(cmdNum, &mut cmd);

        PM_UpdateViewAngles(out, &cmd);
    }

    // if the next frame is a teleport, we can't lerp to it
    if cg.nextFrameTeleport != 0 {
        return;
    }

    if next.is_null() || (*next).serverTime <= (*prev).serverTime {
        return;
    }

    f = (cg.time - (*prev).serverTime) as f32 / ((*next).serverTime - (*prev).serverTime) as f32;

    i = (*next).vps.bobCycle;
    if i < (*prev).vps.bobCycle {
        i += 256;  // handle wraparound
    }
    (*out).bobCycle = ((*prev).vps.bobCycle as f32 + f * (i - (*prev).vps.bobCycle as f32)) as c_int;

    i = 0;
    while i < 3 {
        (*out).origin[i as usize] = (*prev).vps.origin[i as usize] + f * ((*next).vps.origin[i as usize] - (*prev).vps.origin[i as usize]);
        if grabAngles == 0 {
            (*out).viewangles[i as usize] = LerpAngle((*prev).vps.viewangles[i as usize], (*next).vps.viewangles[i as usize], f);
        }
        (*out).velocity[i as usize] = (*prev).vps.velocity[i as usize] + f * ((*next).vps.velocity[i as usize] - (*prev).vps.velocity[i as usize]);
        i += 1;
    }

}

/*
===================
CG_TouchItem
===================
*/
unsafe fn CG_TouchItem(cent: *mut centity_t) {
    let item: *mut gitem_t;

    if cg_predictItems.integer == 0 {
        return;
    }
    if BG_PlayerTouchesItem(&mut cg.predictedPlayerState, &(*cent).currentState, cg.time) == 0 {
        return;
    }

    if (*cent).currentState.brokenLimbs != 0 {
        //dropped item
        return;
    }

    if ((*cent).currentState.eFlags & EF_ITEMPLACEHOLDER) != 0 {
        return;
    }

    if ((*cent).currentState.eFlags & EF_NODRAW) != 0 {
        return;
    }

    // never pick an item up twice in a prediction
    if (*cent).miscTime == cg.time {
        return;
    }

    if BG_CanItemBeGrabbed(cgs.gametype, &(*cent).currentState, &cg.predictedPlayerState) == 0 {
        return;  // can't hold it
    }

    item = &bg_itemlist[(*cent).currentState.modelindex as usize];

    //Currently there is no reliable way of knowing if the client has touched a certain item before another if they are next to each other, or rather
    //if the server has touched them in the same order. This results often in grabbing an item in the prediction and the server giving you the other
    //item. So for now prediction of armor, health, and ammo is disabled.
/*
    if (*item).giType == IT_ARMOR
    { //rww - this will be stomped next update, but it's set so that we don't try to pick up two shields in one prediction and have the server cancel one
    //	cg.predictedPlayerState.stats[STAT_ARMOR] += (*item).quantity;

        //FIXME: This can't be predicted properly at the moment
        return;
    }

    if (*item).giType == IT_HEALTH
    { //same as above, for health
    //	cg.predictedPlayerState.stats[STAT_HEALTH] += (*item).quantity;

        //FIXME: This can't be predicted properly at the moment
        return;
    }

    if (*item).giType == IT_AMMO
    { //same as above, for ammo
    //	cg.predictedPlayerState.ammo[(*item).giTag] += (*item).quantity;

        //FIXME: This can't be predicted properly at the moment
        return;
    }

    if (*item).giType == IT_HOLDABLE
    { //same as above, for holdables
    //	cg.predictedPlayerState.stats[STAT_HOLDABLE_ITEMS] |= (1 << (*item).giTag);
    }
*/
    // Special case for flags.
    // We don't predict touching our own flag
    if cgs.gametype == GT_CTF || cgs.gametype == GT_CTY {
        if cg.predictedPlayerState.persistant[PERS_TEAM] == TEAM_RED && (*item).giTag == PW_REDFLAG {
            return;
        }
        if cg.predictedPlayerState.persistant[PERS_TEAM] == TEAM_BLUE && (*item).giTag == PW_BLUEFLAG {
            return;
        }
    }

    if (*item).giType == IT_POWERUP &&
        ((*item).giTag == PW_FORCE_ENLIGHTENED_LIGHT || (*item).giTag == PW_FORCE_ENLIGHTENED_DARK)
    {
        if (*item).giTag == PW_FORCE_ENLIGHTENED_LIGHT {
            if cg.predictedPlayerState.fd.forceSide != FORCE_LIGHTSIDE {
                return;
            }
        } else {
            if cg.predictedPlayerState.fd.forceSide != FORCE_DARKSIDE {
                return;
            }
        }
    }


    // grab it
    BG_AddPredictableEventToPlayerstate(EV_ITEM_PICKUP, (*cent).currentState.number, &mut cg.predictedPlayerState);

    // remove it from the frame so it won't be drawn
    (*cent).currentState.eFlags |= EF_NODRAW;

    // don't touch it again this prediction
    (*cent).miscTime = cg.time;

    // if its a weapon, give them some predicted ammo so the autoswitch will work
    if (*item).giType == IT_WEAPON {
        cg.predictedPlayerState.stats[STAT_WEAPONS] |= 1 << (*item).giTag;
        if cg.predictedPlayerState.ammo[(*item).giTag as usize] == 0 {
            cg.predictedPlayerState.ammo[(*item).giTag as usize] = 1;
        }
    }
}


/*
=========================
CG_TouchTriggerPrediction

Predict push triggers and items
=========================
*/
unsafe fn CG_TouchTriggerPrediction() {
    let mut i: c_int;
    let mut trace: trace_t;
    let ent: *mut entityState_t;
    let cmodel: clipHandle_t;
    let cent: *mut centity_t;
    let spectator: c_int;

    // dead clients don't activate triggers
    if cg.predictedPlayerState.stats[STAT_HEALTH] <= 0 {
        return;
    }

    spectator = if cg.predictedPlayerState.pm_type == PM_SPECTATOR { 1 } else { 0 };

    if cg.predictedPlayerState.pm_type != PM_NORMAL && cg.predictedPlayerState.pm_type != PM_JETPACK && cg.predictedPlayerState.pm_type != PM_FLOAT && spectator == 0 {
        return;
    }

    i = 0;
    while i < cg_numTriggerEntities {
        cent = cg_triggerEntities[i as usize];
        let ent_ref = &mut (*cent).currentState;

        if ent_ref.eType == ET_ITEM && spectator == 0 {
            CG_TouchItem(cent);
            i += 1;
            continue;
        }

        if ent_ref.solid != SOLID_BMODEL {
            i += 1;
            continue;
        }

        cmodel = trap_CM_InlineModel(ent_ref.modelindex);
        if cmodel.is_null() as c_int == 0 {
            i += 1;
            continue;
        }

        trap_CM_BoxTrace(
            &mut trace,
            &cg.predictedPlayerState.origin,
            &cg.predictedPlayerState.origin,
            &cg_pmove.mins,
            &cg_pmove.maxs,
            cmodel,
            -1,
        );

        if trace.startsolid == 0 {
            i += 1;
            continue;
        }

        if ent_ref.eType == ET_TELEPORT_TRIGGER {
            cg.hyperspace = 1;
        } else if ent_ref.eType == ET_PUSH_TRIGGER {
            BG_TouchJumpPad(&mut cg.predictedPlayerState, ent_ref);
        }
        i += 1;
    }

    // if we didn't touch a jump pad this pmove frame
    if cg.predictedPlayerState.jumppad_frame != cg.predictedPlayerState.pmove_framecount {
        cg.predictedPlayerState.jumppad_frame = 0;
        cg.predictedPlayerState.jumppad_ent = 0;
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[cfg(test)]
fn CG_EntityStateToPlayerState(s: *mut entityState_t, ps: *mut playerState_t) {
    //currently unused vars commented out for speed.. only uncomment if you need them.
    unsafe {
        (*ps).clientNum = (*s).number;
        VectorCopy(&(*s).pos.trBase, &mut (*ps).origin);
        VectorCopy(&(*s).pos.trDelta, &mut (*ps).velocity);
        (*ps).saberLockFrame = (*s).forceFrame;
        (*ps).legsAnim = (*s).legsAnim;
        (*ps).torsoAnim = (*s).torsoAnim;
        (*ps).legsFlip = (*s).legsFlip;
        (*ps).torsoFlip = (*s).torsoFlip;
        (*ps).clientNum = (*s).clientNum;
        (*ps).saberMove = (*s).saberMove;

        /*
        VectorCopy( s->apos.trBase, ps->viewangles );

        ps->fd.forceMindtrickTargetIndex = s->trickedentindex;
        ps->fd.forceMindtrickTargetIndex2 = s->trickedentindex2;
        ps->fd.forceMindtrickTargetIndex3 = s->trickedentindex3;
        ps->fd.forceMindtrickTargetIndex4 = s->trickedentindex4;

        ps->electrifyTime = s->emplacedOwner;

        ps->speed = s->speed;

        ps->genericEnemyIndex = s->genericenemyindex;

        ps->activeForcePass = s->activeForcePass;

        ps->movementDir = s->angles2[YAW];

        ps->eFlags = s->eFlags;

        ps->saberInFlight = s->saberInFlight;
        ps->saberEntityNum = s->saberEntityNum;

        ps->fd.forcePowersActive = s->forcePowersActive;

        if (s->bolt1)
        {
            ps->duelInProgress = qtrue;
        }
        else
        {
            ps->duelInProgress = qfalse;
        }

        if (s->bolt2)
        {
            ps->dualBlade = qtrue;
        }
        else
        {
            ps->dualBlade = qfalse;
        }

        ps->emplacedIndex = s->otherEntityNum2;

        ps->saberHolstered = s->saberHolstered; //reuse bool in entitystate for players differently

        ps->genericEnemyIndex = -1; //no real option for this

        //The client has no knowledge of health levels (except for the client entity)
        if (s->eFlags & EF_DEAD)
        {
            ps->stats[STAT_HEALTH] = 0;
        }
        else
        {
            ps->stats[STAT_HEALTH] = 100;
        }

        if ( ps->externalEvent ) {
            s->event = ps->externalEvent;
            s->eventParm = ps->externalEventParm;
        } else if ( ps->entityEventSequence < ps->eventSequence ) {
            int		seq;

            if ( ps->entityEventSequence < ps->eventSequence - MAX_PS_EVENTS) {
                ps->entityEventSequence = ps->eventSequence - MAX_PS_EVENTS;
            }
            seq = ps->entityEventSequence & (MAX_PS_EVENTS-1);
            s->event = ps->events[ seq ] | ( ( ps->entityEventSequence & 3 ) << 8 );
            s->eventParm = ps->eventParms[ seq ];
            ps->entityEventSequence++;
        }

        ps->weapon = s->weapon;
        ps->groundEntityNum = s->groundEntityNum;

        for ( i = 0 ; i < MAX_POWERUPS ; i++ ) {
            if (s->powerups & (1 << i))
            {
                ps->powerups[i] = 30;
            }
            else
            {
                ps->powerups[i] = 0;
            }
        }

        ps->loopSound = s->loopSound;
        ps->generic1 = s->generic1;
        */
    }
}

// This many playerState_t structures is painfully large. And we
// don't need that many. So we just use a small pool of them.
// PC gets to keep one per entity, just in case.

#[cfg(target_os = "windows")]
struct psLinkedNode_t {
    ps: playerState_t,
    next: *mut psLinkedNode_t,
}

#[cfg(target_os = "windows")]
const CG_SEND_PS_POOL_SIZE: usize = 64;

// Placeholder initialization for ps - will be zeroed at runtime
#[cfg(target_os = "windows")]
static mut cgSendPSPool: [psLinkedNode_t; CG_SEND_PS_POOL_SIZE] = [psLinkedNode_t {
    ps: unsafe { core::mem::zeroed() },
    next: 0 as *mut psLinkedNode_t
}; CG_SEND_PS_POOL_SIZE];

#[cfg(target_os = "windows")]
static mut cgSendPSFreeList: *mut psLinkedNode_t = 0 as *mut psLinkedNode_t;

#[cfg(not(target_os = "windows"))]
static mut cgSendPSPool: [playerState_t; MAX_GENTITIES] = [unsafe { core::mem::zeroed() }; MAX_GENTITIES];

static mut cgSendPS: [*mut playerState_t; MAX_GENTITIES] = [0 as *mut playerState_t; MAX_GENTITIES];

#[cfg(target_os = "windows")]
unsafe fn AllocSendPlayerstate(entNum: c_int) {
    if !cgSendPS[entNum as usize].is_null() {
        //Com_Printf( S_COLOR_RED "ERROR: Entity %d already has a playerstate!\n", entNum );
        return;
    }

    if cgSendPSFreeList.is_null() {
        Com_Error(ERR_DROP, b"ERROR: No free playerstates! Increase CG_SEND_PS_POOL_SIZE\n\0".as_ptr() as *const c_char);
    }

    cgSendPS[entNum as usize] = &mut (*cgSendPSFreeList).ps;
    cgSendPSFreeList = (*cgSendPSFreeList).next;
}

//#define _PROFILE_ES_TO_PS

#[cfg(not(not(test)))]
static mut g_cgEStoPSTime: c_int = 0;

//Assign all the entity playerstate pointers to the corresponding one
//so that we can access playerstate stuff in bg code (and then translate
//it back to entitystate data)
unsafe fn CG_PmoveClientPointerUpdate() {
    let mut i: c_int;

    memset(&mut cgSendPSPool[0] as *mut _ as *mut c_void, 0, core::mem::size_of_val(&cgSendPSPool));

    i = 0;
    while i < MAX_GENTITIES as c_int {
        #[cfg(target_os = "windows")]
        {
            cgSendPS[i as usize] = 0 as *mut playerState_t;
        }
        #[cfg(not(target_os = "windows"))]
        {
            cgSendPS[i as usize] = &mut cgSendPSPool[i as usize];
        }

        // These will be invalid at this point on Xbox
        cg_entities[i as usize].playerState = cgSendPS[i as usize];
        i += 1;
    }

    #[cfg(target_os = "windows")]
    {
        i = 0;
        while i < CG_SEND_PS_POOL_SIZE as c_int - 1 {
            cgSendPSPool[i as usize].next = &mut cgSendPSPool[(i + 1) as usize];
            i += 1;
        }

        // Last .next is already NULL from memset above
        cgSendPSFreeList = &mut cgSendPSPool[0];
    }

    //Set up bg entity data
    cg_pmove.baseEnt = &mut cg_entities[0] as *mut _ as *mut bgEntity_t;
    cg_pmove.entSize = core::mem::size_of::<centity_t>();

    cg_pmove.ghoul2 = 0 as *mut c_void;
}

//check if local client is on an eweb
unsafe fn CG_UsingEWeb() -> c_int {
    if cg.predictedPlayerState.weapon == WP_EMPLACED_GUN && cg.predictedPlayerState.emplacedIndex != 0 &&
        cg_entities[cg.predictedPlayerState.emplacedIndex as usize].currentState.weapon == WP_NONE
    {
        return 1;
    }

    return 0;
}

/*
=================
CG_PredictPlayerState

Generates cg.predictedPlayerState for the current cg.time
cg.predictedPlayerState is guaranteed to be valid after exiting.

For demo playback, this will be an interpolation between two valid
playerState_t.

For normal gameplay, it will be the result of predicted usercmd_t on
top of the most recent playerState_t received from the server.

Each new snapshot will usually have one or more new usercmd over the last,
but we simulate all unacknowledged commands each time, not just the new ones.
This means that on an internet connection, quite a few pmoves may be issued
each frame.

OPTIMIZE: don't re-simulate unless the newly arrived snapshot playerState_t
differs from the predicted one.  Would require saving all intermediate
playerState_t during prediction.

We detect prediction errors and allow them to be decayed off over several frames
to ease the jerk.
=================
*/
// bgAllAnims is not directly called; used via pEnt->localAnimIndex
// extern "C" { fn bgAllAnims(...); }

static mut cg_vehPmove: pmove_t = DEFAULT_PMOVE;

static mut cg_vehPmoveSet: c_int = 0;

#[allow(non_snake_case)]
unsafe fn CG_PredictPlayerState() {
    let mut cmdNum: c_int;
    let mut current: c_int;
    let mut i: c_int;
    let mut oldPlayerState: playerState_t;
    let mut oldVehicleState: playerState_t;
    let mut moved: c_int;
    let mut oldestCmd: usercmd_t;
    let mut latestCmd: usercmd_t;
    let pEnt: *mut centity_t;
    let ci: *mut clientInfo_t;

    cg.hyperspace = 0; // will be set if touching a trigger_teleport

    // if this is the first frame we must guarantee
    // predictedPlayerState is valid even if there is some
    // other error condition
    if cg.validPPS == 0 {
        cg.validPPS = 1;
        cg.predictedPlayerState = (*cg.snap).ps;
        if CG_Piloting((*cg.snap).ps.m_iVehicleNum) != 0 {
            cg.predictedVehicleState = (*cg.snap).vps;
        }
    }

    // demo playback just copies the moves
    if cg.demoPlayback != 0 || ((*cg.snap).ps.pm_flags & PMF_FOLLOW) != 0 {
        CG_InterpolatePlayerState(0);
        if CG_Piloting(cg.predictedPlayerState.m_iVehicleNum) != 0 {
            CG_InterpolateVehiclePlayerState(0);
        }
        return;
    }

    // non-predicting local movement will grab the latest angles
    if cg_nopredict.integer != 0 || cg_synchronousClients.integer != 0 || CG_UsingEWeb() != 0 {
        CG_InterpolatePlayerState(1);
        if CG_Piloting(cg.predictedPlayerState.m_iVehicleNum) != 0 {
            CG_InterpolateVehiclePlayerState(1);
        }
        return;
    }

    // prepare for pmove
    cg_pmove.ps = &mut cg.predictedPlayerState;
    cg_pmove.trace = Some(CG_Trace);
    cg_pmove.pointcontents = Some(CG_PointContents);

    pEnt = &mut cg_entities[cg.predictedPlayerState.clientNum as usize];
    //rww - bgghoul2
    if cg_pmove.ghoul2 != (*pEnt).ghoul2 {
        //only update it if the g2 instance has changed
        if !cg.snap.is_null() &&
            !(*pEnt).ghoul2.is_null() &&
            ((*cg.snap).ps.pm_flags & PMF_FOLLOW) == 0 &&
            (*cg.snap).ps.persistant[PERS_TEAM] != TEAM_SPECTATOR
        {
            cg_pmove.ghoul2 = (*pEnt).ghoul2;
            cg_pmove.g2Bolts_LFoot = trap_G2API_AddBolt((*pEnt).ghoul2, 0, b"*l_leg_foot\0".as_ptr() as *const c_char);
            cg_pmove.g2Bolts_RFoot = trap_G2API_AddBolt((*pEnt).ghoul2, 0, b"*r_leg_foot\0".as_ptr() as *const c_char);
        } else {
            cg_pmove.ghoul2 = 0 as *mut c_void;
        }
    }

    ci = &mut cgs.clientinfo[cg.predictedPlayerState.clientNum as usize];

    //I'll just do this every frame in case the scale changes in realtime (don't need to update the g2 inst for that)
    VectorCopy(&(*pEnt).modelScale, &mut cg_pmove.modelScale);
    //rww end bgghoul2

    if (*cg_pmove.ps).pm_type == PM_DEAD {
        cg_pmove.tracemask = MASK_PLAYERSOLID & !CONTENTS_BODY;
    } else {
        cg_pmove.tracemask = MASK_PLAYERSOLID;
    }
    if (*cg.snap).ps.persistant[PERS_TEAM] == TEAM_SPECTATOR {
        cg_pmove.tracemask &= !CONTENTS_BODY;  // spectators can fly through bodies
    }
    cg_pmove.noFootsteps = if (cgs.dmflags & DF_NO_FOOTSTEPS) > 0 { 1 } else { 0 };

    // save the state before the pmove so we can detect transitions
    oldPlayerState = cg.predictedPlayerState;
    if CG_Piloting(cg.predictedPlayerState.m_iVehicleNum) != 0 {
        oldVehicleState = cg.predictedVehicleState;
    }

    current = trap_GetCurrentCmdNumber();

    // if we don't have the commands right after the snapshot, we
    // can't accurately predict a current position, so just freeze at
    // the last good position we had
    cmdNum = current - CMD_BACKUP + 1;
    trap_GetUserCmd(cmdNum, &mut oldestCmd);
    if oldestCmd.serverTime > (*cg.snap).ps.commandTime && oldestCmd.serverTime < cg.time {
        // special check for map_restart
        if cg_showmiss.integer != 0 {
            CG_Printf(b"exceeded PACKET_BACKUP on commands\n\0".as_ptr() as *const c_char);
        }
        return;
    }

    // get the latest command so we can know which commands are from previous map_restarts
    trap_GetUserCmd(current, &mut latestCmd);

    // get the most recent information we have, even if
    // the server time is beyond our current cg.time,
    // because predicted player positions are going to
    // be ahead of everything else anyway
    if !cg.nextSnap.is_null() && cg.nextFrameTeleport == 0 && cg.thisFrameTeleport == 0 {
        (*cg.nextSnap).ps.slopeRecalcTime = cg.predictedPlayerState.slopeRecalcTime; //this is the only value we want to maintain seperately on server/client
        cg.predictedPlayerState = (*cg.nextSnap).ps;
        if CG_Piloting((*cg.nextSnap).ps.m_iVehicleNum) != 0 {
            cg.predictedVehicleState = (*cg.nextSnap).vps;
        }
        cg.physicsTime = (*cg.nextSnap).serverTime;
    } else {
        (*cg.snap).ps.slopeRecalcTime = cg.predictedPlayerState.slopeRecalcTime; //this is the only value we want to maintain seperately on server/client
        cg.predictedPlayerState = (*cg.snap).ps;
        if CG_Piloting((*cg.snap).ps.m_iVehicleNum) != 0 {
            cg.predictedVehicleState = (*cg.snap).vps;
        }
        cg.physicsTime = (*cg.snap).serverTime;
    }

    if pmove_msec.integer < 8 {
        trap_Cvar_Set(b"pmove_msec\0".as_ptr() as *const c_char, b"8\0".as_ptr() as *const c_char);
    } else if pmove_msec.integer > 33 {
        trap_Cvar_Set(b"pmove_msec\0".as_ptr() as *const c_char, b"33\0".as_ptr() as *const c_char);
    }

    cg_pmove.pmove_fixed = pmove_fixed.integer; // | cg_pmove_fixed.integer;
    cg_pmove.pmove_msec = pmove_msec.integer;

    i = 0;
    while i < MAX_GENTITIES as c_int {
        //Written this way for optimal speed, even though it doesn't look pretty.
        //(we don't want to spend the time assigning pointers as it does take
        //a small precious fraction of time and adds up in the loop.. so says
        //the precision timer!)

        if cg_entities[i as usize].currentState.eType == ET_PLAYER || cg_entities[i as usize].currentState.eType == ET_NPC {
            // Need a new playerState_t on Xbox
            #[cfg(target_os = "windows")]
            {
                AllocSendPlayerstate(i);
            }
            VectorCopy(
                &cg_entities[i as usize].currentState.pos.trBase,
                &mut (*cgSendPS[i as usize]).origin,
            );
            VectorCopy(
                &cg_entities[i as usize].currentState.pos.trDelta,
                &mut (*cgSendPS[i as usize]).velocity,
            );
            (*cgSendPS[i as usize]).saberLockFrame = cg_entities[i as usize].currentState.forceFrame;
            (*cgSendPS[i as usize]).legsAnim = cg_entities[i as usize].currentState.legsAnim;
            (*cgSendPS[i as usize]).torsoAnim = cg_entities[i as usize].currentState.torsoAnim;
            (*cgSendPS[i as usize]).legsFlip = cg_entities[i as usize].currentState.legsFlip;
            (*cgSendPS[i as usize]).torsoFlip = cg_entities[i as usize].currentState.torsoFlip;
            (*cgSendPS[i as usize]).clientNum = cg_entities[i as usize].currentState.clientNum;
            (*cgSendPS[i as usize]).saberMove = cg_entities[i as usize].currentState.saberMove;
        }
        i += 1;
    }

    if CG_Piloting(cg.predictedPlayerState.m_iVehicleNum) != 0 {
        cg_entities[cg.predictedPlayerState.clientNum as usize].playerState = &mut cg.predictedPlayerState;
        cg_entities[cg.predictedPlayerState.m_iVehicleNum as usize].playerState = &mut cg.predictedVehicleState;

        //use the player command time, because we are running with the player cmds (this is even the case
        //on the server)
        cg.predictedVehicleState.commandTime = cg.predictedPlayerState.commandTime;
    }

    // run cmds
    moved = 0;
    cmdNum = current - CMD_BACKUP + 1;
    while cmdNum <= current {
        // get the command
        trap_GetUserCmd(cmdNum, &mut cg_pmove.cmd);

        if cg_pmove.pmove_fixed != 0 {
            PM_UpdateViewAngles(cg_pmove.ps, &cg_pmove.cmd);
        }

        // don't do anything if the time is before the snapshot player time
        if cg_pmove.cmd.serverTime <= cg.predictedPlayerState.commandTime {
            cmdNum += 1;
            continue;
        }

        // don't do anything if the command was from a previous map_restart
        if cg_pmove.cmd.serverTime > latestCmd.serverTime {
            cmdNum += 1;
            continue;
        }

        // check for a prediction error from last frame
        // on a lan, this will often be the exact value
        // from the snapshot, but on a wan we will have
        // to predict several commands to get to the point
        // we want to compare
        if CG_Piloting(oldPlayerState.m_iVehicleNum) != 0 && cg.predictedVehicleState.commandTime == oldVehicleState.commandTime {
            let mut delta: [f32; 3] = [0.0; 3];
            let mut len: f32;

            if cg.thisFrameTeleport != 0 {
                // a teleport will not cause an error decay
                VectorClear(&mut cg.predictedError);
                if cg_showVehMiss.integer != 0 {
                    CG_Printf(b"VEH PredictionTeleport\n\0".as_ptr() as *const c_char);
                }
                cg.thisFrameTeleport = 0;
            } else {
                let mut adjusted: [f32; 3] = [0.0; 3];
                CG_AdjustPositionForMover(
                    &cg.predictedVehicleState.origin,
                    cg.predictedVehicleState.groundEntityNum,
                    cg.physicsTime,
                    cg.oldTime,
                    &mut adjusted,
                );

                if cg_showVehMiss.integer != 0 {
                    if VectorCompare(&oldVehicleState.origin, &adjusted) == 0 {
                        CG_Printf(b"VEH prediction error\n\0".as_ptr() as *const c_char);
                    }
                }
                VectorSubtract(&oldVehicleState.origin, &adjusted, &mut delta);
                len = VectorLength(&delta);
                if len > 0.1 {
                    if cg_showVehMiss.integer != 0 {
                        CG_Printf(b"VEH Prediction miss: %f\n\0".as_ptr() as *const c_char);
                    }
                    if cg_errorDecay.integer != 0 {
                        let mut t: c_int;
                        let mut f: f32;

                        t = cg.time - cg.predictedErrorTime;
                        f = (cg_errorDecay.value - t as f32) / cg_errorDecay.value;
                        if f < 0.0 {
                            f = 0.0;
                        }
                        if f > 0.0 && cg_showVehMiss.integer != 0 {
                            CG_Printf(b"VEH Double prediction decay: %f\n\0".as_ptr() as *const c_char);
                        }
                        VectorScale(&cg.predictedError, f, &mut cg.predictedError);
                    } else {
                        VectorClear(&mut cg.predictedError);
                    }
                    VectorAdd(&delta, &cg.predictedError, &mut cg.predictedError);
                    cg.predictedErrorTime = cg.oldTime;
                }
                //
                if cg_showVehMiss.integer != 0 {
                    if VectorCompare(&oldVehicleState.vehOrientation, &cg.predictedVehicleState.vehOrientation) == 0 {
                        CG_Printf(b"VEH orient prediction error\n\0".as_ptr() as *const c_char);
                        CG_Printf(
                            b"VEH pitch prediction miss: %f\n\0".as_ptr() as *const c_char,
                        );
                        CG_Printf(
                            b"VEH yaw prediction miss: %f\n\0".as_ptr() as *const c_char,
                        );
                        CG_Printf(
                            b"VEH roll prediction miss: %f\n\0".as_ptr() as *const c_char,
                        );
                    }
                }
            }
        } else if oldPlayerState.m_iVehicleNum == 0 && //don't do pred err on ps while riding veh
            cg.predictedPlayerState.commandTime == oldPlayerState.commandTime
        {
            let mut delta: [f32; 3] = [0.0; 3];
            let mut len: f32;

            if cg.thisFrameTeleport != 0 {
                // a teleport will not cause an error decay
                VectorClear(&mut cg.predictedError);
                if cg_showmiss.integer != 0 {
                    CG_Printf(b"PredictionTeleport\n\0".as_ptr() as *const c_char);
                }
                cg.thisFrameTeleport = 0;
            } else {
                let mut adjusted: [f32; 3] = [0.0; 3];
                CG_AdjustPositionForMover(
                    &cg.predictedPlayerState.origin,
                    cg.predictedPlayerState.groundEntityNum,
                    cg.physicsTime,
                    cg.oldTime,
                    &mut adjusted,
                );

                if cg_showmiss.integer != 0 {
                    if VectorCompare(&oldPlayerState.origin, &adjusted) == 0 {
                        CG_Printf(b"prediction error\n\0".as_ptr() as *const c_char);
                    }
                }
                VectorSubtract(&oldPlayerState.origin, &adjusted, &mut delta);
                len = VectorLength(&delta);
                if len > 0.1 {
                    if cg_showmiss.integer != 0 {
                        CG_Printf(b"Prediction miss: %f\n\0".as_ptr() as *const c_char);
                    }
                    if cg_errorDecay.integer != 0 {
                        let mut t: c_int;
                        let mut f: f32;

                        t = cg.time - cg.predictedErrorTime;
                        f = (cg_errorDecay.value - t as f32) / cg_errorDecay.value;
                        if f < 0.0 {
                            f = 0.0;
                        }
                        if f > 0.0 && cg_showmiss.integer != 0 {
                            CG_Printf(b"Double prediction decay: %f\n\0".as_ptr() as *const c_char);
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

        if cg_pmove.pmove_fixed != 0 {
            cg_pmove.cmd.serverTime = ((cg_pmove.cmd.serverTime + pmove_msec.integer - 1) / pmove_msec.integer) * pmove_msec.integer;
        }

        cg_pmove.animations = 0 as *mut c_void; // bgAllAnims[pEnt->localAnimIndex].anims;
        cg_pmove.gametype = cgs.gametype;

        cg_pmove.debugMelee = cgs.debugMelee;
        cg_pmove.stepSlideFix = cgs.stepSlideFix;
        cg_pmove.noSpecMove = cgs.noSpecMove;

        cg_pmove.nonHumanoid = if (*pEnt).localAnimIndex > 0 { 1 } else { 0 };

        if !cg.snap.is_null() && (*cg.snap).ps.saberLockTime > cg.time {
            let blockOpp: *mut centity_t = &mut cg_entities[(*cg.snap).ps.saberLockEnemy as usize];

            if !blockOpp.is_null() {
                let mut lockDir: [f32; 3] = [0.0; 3];
                let mut lockAng: [f32; 3] = [0.0; 3];

                VectorSubtract(&(*blockOpp).lerpOrigin, &(*cg.snap).ps.origin, &mut lockDir);
                vectoangles(&lockDir, &mut lockAng);

                VectorCopy(&lockAng, &mut (*cg_pmove.ps).viewangles);
            }
        }

        //THIS is pretty much bad, but...
        (*cg_pmove.ps).fd.saberAnimLevelBase = (*cg_pmove.ps).fd.saberAnimLevel;
        if (*cg_pmove.ps).saberHolstered == 1 {
            if (*ci).saber[0].numBlades > 0 {
                (*cg_pmove.ps).fd.saberAnimLevelBase = SS_STAFF;
            } else if (*ci).saber[1].model[0] != 0 {
                (*cg_pmove.ps).fd.saberAnimLevelBase = SS_DUAL;
            }
        }

        Pmove(&mut cg_pmove);

        if CG_Piloting(cg.predictedPlayerState.m_iVehicleNum) != 0 && cg.predictedPlayerState.pm_type != PM_INTERMISSION {
            //we're riding a vehicle, let's predict it
            let veh: *mut centity_t = &mut cg_entities[cg.predictedPlayerState.m_iVehicleNum as usize];
            let mut x: c_int;
            let mut zd: c_int;
            let mut zu: c_int;

            if !(*veh).m_pVehicle.is_null() {
                //make sure pointer is set up to go to our predicted state
                (*(*veh).m_pVehicle).m_vOrientation = &mut cg.predictedVehicleState.vehOrientation[0];

                //keep this updated based on what the playerstate says
                (*(*veh).m_pVehicle).m_iRemovedSurfaces = cg.predictedVehicleState.vehSurfaces;

                trap_GetUserCmd(cmdNum, &mut (*(*veh).m_pVehicle).m_ucmd);

                if ((*(*veh).m_pVehicle).m_ucmd.buttons & BUTTON_TALK) != 0 {
                    //forced input if "chat bubble" is up
                    (*(*veh).m_pVehicle).m_ucmd.buttons = BUTTON_TALK;
                    (*(*veh).m_pVehicle).m_ucmd.forwardmove = 0;
                    (*(*veh).m_pVehicle).m_ucmd.rightmove = 0;
                    (*(*veh).m_pVehicle).m_ucmd.upmove = 0;
                }
                cg_vehPmove.ps = &mut cg.predictedVehicleState;
                cg_vehPmove.animations = 0 as *mut c_void; // bgAllAnims[veh->localAnimIndex].anims;

                memcpy(
                    &mut cg_vehPmove.cmd as *mut _ as *mut c_void,
                    &(*(*veh).m_pVehicle).m_ucmd as *const _ as *const c_void,
                    core::mem::size_of::<usercmd_t>(),
                );
                /*
                cg_vehPmove.cmd.rightmove = 0; //no vehicle can move right/left
                cg_vehPmove.cmd.upmove = 0; //no vehicle can move up/down
                */

                cg_vehPmove.gametype = cgs.gametype;
                cg_vehPmove.ghoul2 = (*veh).ghoul2;

                cg_vehPmove.nonHumanoid = if (*veh).localAnimIndex > 0 { 1 } else { 0 };

                /*
                x = (veh->currentState.solid & 255);
                zd = (veh->currentState.solid & 255);
                zu = (veh->currentState.solid & 255) - 32;

                cg_vehPmove.mins[0] = cg_vehPmove.mins[1] = -x;
                cg_vehPmove.maxs[0] = cg_vehPmove.maxs[1] = x;
                cg_vehPmove.mins[2] = -zd;
                cg_vehPmove.maxs[2] = zu;
                */
                //I think this was actually wrong.. just copy-pasted from id code. Oh well.
                x = ((*veh).currentState.solid) & 255;
                zd = (((*veh).currentState.solid >> 8) & 255) as c_int;
                zu = (((*veh).currentState.solid >> 15) & 255) as c_int;

                zu -= 32; //I don't quite get the reason for this.
                zd = -zd;

                //z/y must be symmetrical (blah)
                cg_vehPmove.mins[0] = -x as f32;
                cg_vehPmove.mins[1] = -x as f32;
                cg_vehPmove.maxs[0] = x as f32;
                cg_vehPmove.maxs[1] = x as f32;
                cg_vehPmove.mins[2] = zd as f32;
                cg_vehPmove.maxs[2] = zu as f32;

                VectorCopy(&(*veh).modelScale, &mut cg_vehPmove.modelScale);

                if cg_vehPmoveSet == 0 {
                    //do all the one-time things
                    cg_vehPmove.trace = Some(CG_Trace);
                    cg_vehPmove.pointcontents = Some(CG_PointContents);
                    cg_vehPmove.tracemask = MASK_PLAYERSOLID;
                    cg_vehPmove.debugLevel = 0;
                    cg_vehPmove.g2Bolts_LFoot = -1;
                    cg_vehPmove.g2Bolts_RFoot = -1;

                    cg_vehPmove.baseEnt = &mut cg_entities[0] as *mut _ as *mut bgEntity_t;
                    cg_vehPmove.entSize = core::mem::size_of::<centity_t>();

                    cg_vehPmoveSet = 1;
                }

                cg_vehPmove.noFootsteps = if (cgs.dmflags & DF_NO_FOOTSTEPS) > 0 { 1 } else { 0 };
                cg_vehPmove.pmove_fixed = pmove_fixed.integer;
                cg_vehPmove.pmove_msec = pmove_msec.integer;

                cg_entities[cg.predictedPlayerState.clientNum as usize].playerState = &mut cg.predictedPlayerState;
                (*veh).playerState = &mut cg.predictedVehicleState;

                //update boarding value sent from server. boarding is not predicted, but no big deal
                (*(*veh).m_pVehicle).m_iBoarding = cg.predictedVehicleState.vehBoarding;

                Pmove(&mut cg_vehPmove);
                /*
                if ( !cg_paused.integer )
                {
                    Com_Printf( "%d - PITCH change %4.2f\n", cg.time, AngleSubtract(veh->m_pVehicle->m_vOrientation[0],veh->m_pVehicle->m_vPrevOrientation[0]) );
                }
                */
                if cg_showVehBounds.integer != 0 {
                    let NPCDEBUG_RED: [f32; 3] = [1.0, 0.0, 0.0];
                    let mut absmin: [f32; 3] = [0.0; 3];
                    let mut absmax: [f32; 3] = [0.0; 3];
                    VectorAdd(&(*cg_vehPmove.ps).origin, &cg_vehPmove.mins, &mut absmin);
                    VectorAdd(&(*cg_vehPmove.ps).origin, &cg_vehPmove.maxs, &mut absmax);
                    CG_Cube(&absmin, &absmax, &NPCDEBUG_RED, 0.25);
                }
            }
        }

        moved = 1;

        // add push trigger movement effects
        CG_TouchTriggerPrediction();

        // check for predictable events that changed from previous predictions
        //CG_CheckChangedPredictableEvents(&cg.predictedPlayerState);

        cmdNum += 1;
    }

    if cg_showmiss.integer > 1 {
        CG_Printf(b"[%i : %i] \0".as_ptr() as *const c_char);
    }

    // C goto revertES equivalent: skip main logic if not moved
    let skip_main_logic = moved == 0;

    if !skip_main_logic {
        if CG_Piloting(cg.predictedPlayerState.m_iVehicleNum) != 0 {
            CG_AdjustPositionForMover(
                &cg.predictedVehicleState.origin,
                cg.predictedVehicleState.groundEntityNum,
                cg.physicsTime,
                cg.time,
                &mut cg.predictedVehicleState.origin,
            );
        } else {
            // adjust for the movement of the groundentity
            CG_AdjustPositionForMover(
                &cg.predictedPlayerState.origin,
                cg.predictedPlayerState.groundEntityNum,
                cg.physicsTime,
                cg.time,
                &mut cg.predictedPlayerState.origin,
            );
        }

        if cg_showmiss.integer != 0 {
            if cg.predictedPlayerState.eventSequence > oldPlayerState.eventSequence + MAX_PS_EVENTS {
                CG_Printf(b"WARNING: dropped event\n\0".as_ptr() as *const c_char);
            }
        }

        // fire events and other transition triggered things
        CG_TransitionPlayerState(&mut cg.predictedPlayerState, &oldPlayerState);

        if cg_showmiss.integer != 0 {
            if cg.eventSequence > cg.predictedPlayerState.eventSequence {
                CG_Printf(b"WARNING: double event\n\0".as_ptr() as *const c_char);
                cg.eventSequence = cg.predictedPlayerState.eventSequence;
            }
        }

        if cg.predictedPlayerState.m_iVehicleNum != 0 && CG_Piloting(cg.predictedPlayerState.m_iVehicleNum) == 0 {
            //a passenger on this vehicle, bolt them in
            let veh: *mut centity_t = &mut cg_entities[cg.predictedPlayerState.m_iVehicleNum as usize];
            //VectorCopy(veh->lerpAngles, cg.predictedPlayerState.viewangles);
            VectorCopy(&(*veh).lerpOrigin, &mut cg.predictedPlayerState.origin);
        }
    } else {
        if cg_showmiss.integer != 0 {
            CG_Printf(b"not moved\n\0".as_ptr() as *const c_char);
        }
    }

    // revertES label equivalent
    if CG_Piloting(cg.predictedPlayerState.m_iVehicleNum) != 0 {
        let veh: *mut centity_t = &mut cg_entities[cg.predictedPlayerState.m_iVehicleNum as usize];

        if !(*veh).m_pVehicle.is_null() {
            //switch ptr back for this ent in case we stop riding it
            (*(*veh).m_pVehicle).m_vOrientation = &mut (*cgSendPS[(*veh).currentState.number as usize]).vehOrientation[0];
        }

        cg_entities[cg.predictedPlayerState.clientNum as usize].playerState = cgSendPS[cg.predictedPlayerState.clientNum as usize];
        (*veh).playerState = cgSendPS[(*veh).currentState.number as usize];
    }

    //copy some stuff back into the entstates to help actually "predict" them if applicable
    i = 0;
    while i < MAX_GENTITIES as c_int {
        if cg_entities[i as usize].currentState.eType == ET_PLAYER || cg_entities[i as usize].currentState.eType == ET_NPC {
            cg_entities[i as usize].currentState.torsoAnim = (*cgSendPS[i as usize]).torsoAnim;
            cg_entities[i as usize].currentState.legsAnim = (*cgSendPS[i as usize]).legsAnim;
            cg_entities[i as usize].currentState.forceFrame = (*cgSendPS[i as usize]).saberLockFrame;
            cg_entities[i as usize].currentState.saberMove = (*cgSendPS[i as usize]).saberMove;
        }
        i += 1;
    }
}

// Additional constants needed
const SS_STAFF: c_int = 2;
const SS_DUAL: c_int = 1;

// Stub for bg_itemlist
extern "C" {
    static bg_itemlist: [gitem_t; 256];
}

// Stub for cg_predictItems
extern "C" {
    static cg_predictItems: vmCvar_t;
}

#[repr(C)]
pub struct vmCvar_t {
    pub integer: c_int,
    pub value: f32,
}
