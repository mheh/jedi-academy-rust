//! Port of `FighterNPC.c` — the flying-fighter vehicle behaviour table (X-wing-class
//! craft). Provides the per-type vtable overrides (`Board`/`Eject`/`Update`/
//! `Animate*`/`ProcessMove`/`ProcessOrient`) that `G_SetFighterVehicleFunctions`
//! installs on a `vehicleInfo_t`, layered on top of the shared `VEHICLE_BASE`
//! defaults in `g_vehicles`.
//!
//! The whole MP fighter chain is landed: `Board`/`Eject` (delegate to the
//! `VEHICLE_BASE` vtable), the pure `PredictedAngularDecrement` angle-decay helper
//! (oracle), the `Fighter{Roll,Yaw,Pitch}Adjust`/`FighterPitchClamp` orientation
//! helpers, the physics core (`BG_FighterUpdate`, `Update`, the
//! `FighterIs{Landing,Landed,Launching,InSpace}`/`FighterOverValidLandingSurface`/
//! `FighterSuspended` predicates, `Fighter{Wing,Nose}MalfunctionCheck`,
//! `FighterDamageRoutine`), the `ProcessMove`/`ProcessOrient` commands, `AnimateVehicle`,
//! the faithful-empty `AnimateRiders`, and the `G_SetFighterVehicleFunctions` installer
//! that wires them up — all No-oracle (they take/return pointers or mutate
//! `Vehicle_t`/`playerState_t` through pointers, nothing bit-exact to compare).
//! `G_CreateFighterNPC` — the vehicle-object constructor — is landed: allocates via
//! `G_AllocateVehicleObject`, zeroes the `Vehicle_t`, and points `m_pVehicleInfo` at
//! `g_vehicleInfo[BG_VehicleGetIndex(type)]`.

#![allow(non_snake_case)] // C function names (`FighterRollAdjust`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro/enum names kept verbatim

use core::ffi::{c_char, c_int, c_ulong};
use core::ptr::{addr_of, addr_of_mut, write_bytes};

use crate::codemp::game::anims::{
    BOTH_GEARS_CLOSE, BOTH_GEARS_OPEN, BOTH_WINGS_CLOSE, BOTH_WINGS_OPEN,
};
use crate::codemp::game::bg_panimate::{bgAllAnims, BG_SetAnim};
use crate::codemp::game::bg_pmove::{
    BG_UnrestrainedPitchRoll, BG_VehicleTurnRateForSpeed, PM_BGEntForNum,
};
use crate::codemp::game::bg_public::{
    bgEntity_t, EF2_HYPERSPACE, EF_DEAD, EF_JETPACK_ACTIVE, HYPERSPACE_SPEED, HYPERSPACE_TELEPORT_FRAC,
    HYPERSPACE_TIME, MASK_NPCSOLID, MOD_SUICIDE, SETANIM_BOTH, SETANIM_FLAG_NORMAL,
};
use crate::codemp::game::q_shared_h::CHAN_AUTO;
use crate::codemp::game::bg_vehicleLoad::{g_vehicleInfo, BG_VehicleGetIndex};
use crate::codemp::game::bg_vehicles_h::{
    vehicleInfo_t, Vehicle_t, MAX_VEHICLE_EXHAUSTS, MAX_STRAFE_TIME, MIN_LANDING_SLOPE,
    MIN_LANDING_SPEED, SHIPSURF_BROKEN_C, SHIPSURF_BROKEN_D, SHIPSURF_BROKEN_E, SHIPSURF_BROKEN_F,
    SHIPSURF_DAMAGE_BACK_HEAVY, SHIPSURF_DAMAGE_BACK_LIGHT, SHIPSURF_DAMAGE_FRONT_HEAVY,
    SHIPSURF_DAMAGE_FRONT_LIGHT, SHIPSURF_DAMAGE_LEFT_HEAVY, SHIPSURF_DAMAGE_LEFT_LIGHT,
    SHIPSURF_DAMAGE_RIGHT_HEAVY, SHIPSURF_DAMAGE_RIGHT_LIGHT, VEHICLE_BASE, VEH_GEARSOPEN,
    VEH_WINGSOPEN, VH_FIGHTER,
};
use crate::codemp::game::g_combat::G_DamageFromKiller;
use crate::codemp::game::g_local::{gentity_t, DAMAGE_NO_ARMOR};
use crate::codemp::game::g_main::{g_gravity, level};
use crate::codemp::game::g_utils::{G_AllocateVehicleObject, G_EntitySound};
use crate::codemp::game::g_vehicles::G_VehicleTrace;
use crate::codemp::game::q_math::{
    AngleNormalize180, AngleNormalize360, AngleSubtract, AngleVectors, DotProduct, VectorClear,
    VectorCopy, VectorMA, VectorScale, VectorLength,
};
use crate::codemp::game::q_shared_h::{
    playerState_t, qboolean, trace_t, usercmd_t, vec3_t, ENTITYNUM_NONE, ENTITYNUM_WORLD, ERR_DROP,
    MAX_CLIENTS, PITCH, QFALSE, QTRUE, ROLL, YAW,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_BODY;
use crate::codemp::game::g_main::Com_Error;

// SP-only `#define`s elided. The shared `FIGHTER_TURNING_*` magic numbers are the
// `_JK2MP` (MP) values, which is the ABI target.
const FIGHTER_TURNING_MULTIPLIER: f32 = 0.8; //was 1.6f //magic number hackery

// Board this Vehicle (get on). The first entity to board an empty vehicle becomes the Pilot.
pub unsafe extern "C" fn Board(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    if (g_vehicleInfo[VEHICLE_BASE as usize].Board.unwrap())(pVeh, pEnt) == QFALSE {
        return QFALSE;
    }

    // Set the board wait time (they won't be able to do anything, including getting off, for this amount of time).
    (*pVeh).m_iBoarding = (*addr_of!(level)).time + 1500;

    QTRUE
}

// Eject an entity from the vehicle.
pub unsafe extern "C" fn Eject(
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
    forceEject: qboolean,
) -> qboolean {
    if (g_vehicleInfo[VEHICLE_BASE as usize].Eject.unwrap())(pVeh, pEnt, forceEject) == QTRUE {
        return QTRUE;
    }

    QFALSE
}

//method of decrementing the given angle based on the given taking variable frame times into account
pub fn PredictedAngularDecrement(scale: f32, timeMod: f32, originalAngle: f32) -> f32 {
    let mut fixedBaseDec = originalAngle * 0.05;
    let mut r = 0.0;

    if fixedBaseDec < 0.0 {
        fixedBaseDec = -fixedBaseDec;
    }

    fixedBaseDec *= 1.0 + (1.0 - scale);

    if fixedBaseDec < 0.1 {
        //don't increment in incredibly small fractions, it would eat up unnecessary bandwidth.
        fixedBaseDec = 0.1;
    }

    fixedBaseDec *= timeMod * 0.1;
    if originalAngle > 0.0 {
        //subtract
        r = originalAngle - fixedBaseDec;
        if r < 0.0 {
            r = 0.0;
        }
    } else if originalAngle < 0.0 {
        //add
        r = originalAngle + fixedBaseDec;
        if r > 0.0 {
            r = 0.0;
        }
    }

    r
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    /// `PredictedAngularDecrement` is pure scalar float math (three floats in, one out),
    /// so it parity-tests bit-exact against the real C body with no marshalling. Sweep a
    /// spread of scales, time modifiers, and angles (positive, negative, zero, and the
    /// tiny-fraction clamp path) and compare raw bit patterns.
    #[test]
    fn PredictedAngularDecrement_matches_oracle_bit_exact() {
        let scales = [0.0f32, 0.5, 0.95, 1.0, 1.5, 2.0];
        let timeMods = [0.0f32, 0.1, 1.0, 2.0, 10.0, 0.05];
        let angles = [
            0.0f32, -0.0, 0.001, -0.001, 1.0, -1.0, 45.0, -45.0, 180.0, -180.0, 0.5, -0.5,
        ];
        for &scale in &scales {
            for &timeMod in &timeMods {
                for &originalAngle in &angles {
                    let rust = PredictedAngularDecrement(scale, timeMod, originalAngle);
                    let c = unsafe {
                        oracle::jka_PredictedAngularDecrement(scale, timeMod, originalAngle)
                    };
                    assert_eq!(
                        rust.to_bits(),
                        c.to_bits(),
                        "PredictedAngularDecrement({scale}, {timeMod}, {originalAngle}): \
                         rust={rust} ({:#x}) c={c} ({:#x})",
                        rust.to_bits(),
                        c.to_bits()
                    );
                }
            }
        }
    }
}

pub unsafe fn FighterRollAdjust(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
) {
    /*
        float angDif = AngleSubtract(pVeh->m_vOrientation[YAW], riderPS->viewangles[YAW]);
    */
    let mut angDif = AngleSubtract(
        (*pVeh).m_vPrevRiderViewAngles[YAW],
        (*riderPS).viewangles[YAW],
    ); //2.0f;//AngleSubtract(pVeh->m_vPrevRiderViewAngles[YAW], riderPS->viewangles[YAW]);
    /*
    if ( fabs( angDif ) < FIGHTER_TURNING_DEADZONE )
    {
        angDif = 0.0f;
    }
    else if ( angDif >= FIGHTER_TURNING_DEADZONE )
    {
        angDif -= FIGHTER_TURNING_DEADZONE;
    }
    else if ( angDif <= -FIGHTER_TURNING_DEADZONE )
    {
        angDif += FIGHTER_TURNING_DEADZONE;
    }
    */

    angDif *= 0.5;
    if angDif > 0.0 {
        angDif *= angDif;
    } else if angDif < 0.0 {
        angDif *= -angDif;
    }

    if !parentPS.is_null() && (*parentPS).speed != 0.0 {
        let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * FIGHTER_TURNING_MULTIPLIER;

        if (*(*pVeh).m_pVehicleInfo).speedDependantTurning == QTRUE {
            let mut speedFrac = 1.0;
            if (*pVeh).m_LandTrace.fraction >= 1.0
                || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE
            {
                let mut s = (*parentPS).speed;
                if s < 0.0 {
                    s = -s;
                }
                speedFrac = s / ((*(*pVeh).m_pVehicleInfo).speedMax * 0.75);
                if speedFrac < 0.25 {
                    speedFrac = 0.25;
                } else if speedFrac > 1.0 {
                    speedFrac = 1.0;
                }
            }
            angDif *= speedFrac;
        }
        if angDif > maxDif {
            angDif = maxDif;
        } else if angDif < -maxDif {
            angDif = -maxDif;
        }
        *(*pVeh).m_vOrientation.add(ROLL) = AngleNormalize180(
            *(*pVeh).m_vOrientation.add(ROLL) + angDif * ((*pVeh).m_fTimeModifier * 0.2),
        );
    }
}

pub unsafe fn FighterYawAdjust(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
) {
    let mut angDif = AngleSubtract(*(*pVeh).m_vOrientation.add(YAW), (*riderPS).viewangles[YAW]);

    if !parentPS.is_null() && (*parentPS).speed != 0.0 {
        let mut s = (*parentPS).speed;
        let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 0.8; //magic number hackery

        if s < 0.0 {
            s = -s;
        }
        angDif *= s / (*(*pVeh).m_pVehicleInfo).speedMax;
        if angDif > maxDif {
            angDif = maxDif;
        } else if angDif < -maxDif {
            angDif = -maxDif;
        }
        *(*pVeh).m_vOrientation.add(YAW) = AngleNormalize180(
            *(*pVeh).m_vOrientation.add(YAW) - angDif * ((*pVeh).m_fTimeModifier * 0.2),
        );
    }
}

pub unsafe fn FighterPitchAdjust(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
) {
    let mut angDif =
        AngleSubtract(*(*pVeh).m_vOrientation.add(PITCH), (*riderPS).viewangles[PITCH]);

    if !parentPS.is_null() && (*parentPS).speed != 0.0 {
        let mut s = (*parentPS).speed;
        let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 0.8; //magic number hackery

        if s < 0.0 {
            s = -s;
        }
        angDif *= s / (*(*pVeh).m_pVehicleInfo).speedMax;
        if angDif > maxDif {
            angDif = maxDif;
        } else if angDif < -maxDif {
            angDif = -maxDif;
        }
        *(*pVeh).m_vOrientation.add(PITCH) = AngleNormalize360(
            *(*pVeh).m_vOrientation.add(PITCH) - angDif * ((*pVeh).m_fTimeModifier * 0.2),
        );
    }
}

pub unsafe fn FighterPitchClamp(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
    curTime: i32,
) {
    if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QFALSE {
        //cap pitch reasonably
        if (*(*pVeh).m_pVehicleInfo).pitchLimit != -1.0
            && (*pVeh).m_iRemovedSurfaces == 0
            && (*parentPS).electrifyTime < curTime
        {
            if *(*pVeh).m_vOrientation.add(PITCH) > (*(*pVeh).m_pVehicleInfo).pitchLimit {
                *(*pVeh).m_vOrientation.add(PITCH) = (*(*pVeh).m_pVehicleInfo).pitchLimit;
            } else if *(*pVeh).m_vOrientation.add(PITCH) < -(*(*pVeh).m_pVehicleInfo).pitchLimit {
                *(*pVeh).m_vOrientation.add(PITCH) = -(*(*pVeh).m_pVehicleInfo).pitchLimit;
            }
        }
    }
}

//this stuff has got to be predicted, so..
// Not `extern "C"`: only called directly by the in-file `Update` (never an engine-ABI
// fn-ptr), and it takes a Rust-ABI `traceFunc` pointer, so a C ABI would be FFI-unsafe.
pub unsafe fn BG_FighterUpdate(
    pVeh: *mut Vehicle_t,
    _pUcmd: *const usercmd_t,
    trMins: *const vec3_t,
    trMaxs: *const vec3_t,
    gravity: f32,
    // C: `void (*traceFunc)(trace_t*, const vec3_t, const vec3_t, const vec3_t, const vec3_t, int, int)`.
    // The only caller passes the Rust-ABI `G_VehicleTrace`; this fn-ptr never crosses the
    // engine VM-ABI boundary, so it stays a plain Rust `unsafe fn` pointer.
    traceFunc: unsafe fn(
        results: *mut trace_t,
        start: *const vec3_t,
        lmins: *const vec3_t,
        lmaxs: *const vec3_t,
        end: *const vec3_t,
        passEntityNum: c_int,
        contentMask: c_int,
    ),
) -> qboolean {
    let mut bottom: vec3_t = [0.0; 3];
    let parentPS: *mut playerState_t;
    // #ifdef QAGAME //don't do this on client
    let mut i: c_int;

    // Make sure the riders are not visible or collidable.
    ((*(*pVeh).m_pVehicleInfo).Ghost.unwrap())(pVeh, (*pVeh).m_pPilot);
    i = 0;
    while i < (*(*pVeh).m_pVehicleInfo).maxPassengers {
        ((*(*pVeh).m_pVehicleInfo).Ghost.unwrap())(pVeh, (*pVeh).m_ppPassengers[i as usize]);
        i += 1;
    }

    parentPS = (*(*pVeh).m_pParentEntity).playerState;

    if parentPS.is_null() {
        Com_Error(ERR_DROP, "NULL PS in BG_FighterUpdate");
        #[allow(unreachable_code)]
        {
            return QFALSE;
        }
    }

    // If we have a pilot, take out gravity (it's a flying craft...).
    if !(*pVeh).m_pPilot.is_null() {
        (*parentPS).gravity = 0;
    } else {
        //in MP set grav back to normal gravity
        if (*(*pVeh).m_pVehicleInfo).gravity != 0 {
            (*parentPS).gravity = (*(*pVeh).m_pVehicleInfo).gravity;
        } else {
            //it doesn't have gravity specified apparently
            (*parentPS).gravity = gravity as c_int;
        }
    }

    // isDead computed but only used in the elided land-trace skip block below; kept faithful.
    let _isDead: qboolean =
        if (*parentPS).eFlags & EF_DEAD != 0 { QTRUE } else { QFALSE };

    /*
    if ( isDead ||
        (pVeh->m_pVehicleInfo->surfDestruction &&
            pVeh->m_iRemovedSurfaces ) )
    {//can't land if dead or spiralling out of control
        pVeh->m_LandTrace.fraction = 1.0f;
        pVeh->m_LandTrace.contents = pVeh->m_LandTrace.surfaceFlags = 0;
        VectorClear( pVeh->m_LandTrace.plane.normal );
        pVeh->m_LandTrace.allsolid = qfalse;
        pVeh->m_LandTrace.startsolid = qfalse;
    }
    else
    {
    */
    //argh, no, I need to have a way to see when they impact the ground while damaged. -rww

    // Check to see if the fighter has taken off yet (if it's a certain height above ground).
    VectorCopy(&(*parentPS).origin, &mut bottom);
    bottom[2] -= (*(*pVeh).m_pVehicleInfo).landingHeight;

    traceFunc(
        &mut (*pVeh).m_LandTrace,
        &(*parentPS).origin,
        trMins,
        trMaxs,
        &bottom,
        (*(*pVeh).m_pParentEntity).s.number,
        MASK_NPCSOLID & !CONTENTS_BODY,
    );
    //}

    QTRUE
}

// #ifdef QAGAME //ONLY in SP or on server, not cgame

// Like a think or move command, this updates various vehicle properties.
pub unsafe extern "C" fn Update(pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    debug_assert!(!(*pVeh).m_pParentEntity.is_null());
    let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
    if BG_FighterUpdate(
        pVeh,
        pUcmd,
        &(*parent).r.mins,
        &(*parent).r.maxs,
        (*addr_of!(g_gravity)).value,
        G_VehicleTrace,
    ) == QFALSE
    {
        return QFALSE;
    }

    if (g_vehicleInfo[VEHICLE_BASE as usize].Update.unwrap())(pVeh, pUcmd) == QFALSE {
        return QFALSE;
    }

    QTRUE
}

// #ifdef QAGAME//only do this check on GAME side, because if it's CGAME, it's being predicted, and it's only predicted if the local client is the driver
pub unsafe extern "C" fn FighterIsInSpace(gParent: *mut gentity_t) -> qboolean {
    if !gParent.is_null()
        && !(*gParent).client.is_null()
        && (*(*gParent).client).inSpaceIndex != 0
        && (*(*gParent).client).inSpaceIndex < ENTITYNUM_WORLD
    {
        return QTRUE;
    }
    QFALSE
}

pub unsafe extern "C" fn FighterOverValidLandingSurface(pVeh: *mut Vehicle_t) -> qboolean {
    if (*pVeh).m_LandTrace.fraction < 1.0 //ground present
        && (*pVeh).m_LandTrace.plane.normal[2] >= MIN_LANDING_SLOPE
    //flat enough
    //FIXME: also check for a certain surface flag ... "landing zones"?
    {
        return QTRUE;
    }
    QFALSE
}

pub unsafe extern "C" fn FighterIsLanded(
    pVeh: *mut Vehicle_t,
    parentPS: *mut playerState_t,
) -> qboolean {
    if FighterOverValidLandingSurface(pVeh) == QTRUE && (*parentPS).speed == 0.0
    //stopped
    {
        return QTRUE;
    }
    QFALSE
}

pub unsafe extern "C" fn FighterIsLanding(
    pVeh: *mut Vehicle_t,
    parentPS: *mut playerState_t,
) -> qboolean {
    if FighterOverValidLandingSurface(pVeh) == QTRUE
        // #ifdef QAGAME//only do this check on GAME side
        && ((*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) == QTRUE //has to have a driver in order to be capable of landing
        && ((*pVeh).m_ucmd.forwardmove < 0 || (*pVeh).m_ucmd.upmove < 0) //decelerating or holding crouch button
        && (*parentPS).speed <= MIN_LANDING_SPEED as f32
    //going slow enough to start landing - was using pVeh->m_pVehicleInfo->speedIdle, but that's still too fast
    {
        return QTRUE;
    }
    QFALSE
}

pub unsafe extern "C" fn FighterIsLaunching(
    pVeh: *mut Vehicle_t,
    parentPS: *mut playerState_t,
) -> qboolean {
    if FighterOverValidLandingSurface(pVeh) == QTRUE
        // #ifdef QAGAME//only do this check on GAME side
        && ((*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) == QTRUE //has to have a driver in order to be capable of landing
        && (*pVeh).m_ucmd.upmove > 0 //trying to take off
        && (*parentPS).speed <= 200.0
    //going slow enough to start landing - was using pVeh->m_pVehicleInfo->speedIdle, but that's still too fast
    {
        return QTRUE;
    }
    QFALSE
}

pub unsafe extern "C" fn FighterSuspended(
    pVeh: *mut Vehicle_t,
    parentPS: *mut playerState_t,
) -> qboolean {
    // #ifdef QAGAME//only do this check on GAME side
    if (*pVeh).m_pPilot.is_null() //empty
        && (*parentPS).speed == 0.0 //not moving
        && (*pVeh).m_ucmd.forwardmove <= 0 //not trying to go forward for whatever reason
        && !(*pVeh).m_pParentEntity.is_null()
        && ((*((*pVeh).m_pParentEntity as *mut gentity_t)).spawnflags & 2) != 0
    //SUSPENDED spawnflag is on
    {
        return QTRUE;
    }
    QFALSE
}

//MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
//If you really need to violate this rule for SP, then use ifdefs.
//By BG-compatible, I mean no use of game-specific data - ONLY use
//stuff available in the MP bgEntity (in SP, the bgEntity is #defined
//as a gentity, but the MP-compatible access restrictions are based
//on the bgEntity structure in the MP codebase) -rww
// ProcessMoveCommands the Vehicle.
const FIGHTER_MIN_TAKEOFF_FRACTION: f32 = 0.7;
// TODO: Remove-Xbox
pub unsafe extern "C" fn ProcessMoveCommands(pVeh: *mut Vehicle_t) {
    /************************************************************************************/
    /*	BEGIN	Here is where we move the vehicle (forward or back or whatever). BEGIN	*/
    /************************************************************************************/

    //Client sets ucmds and such for speed alterations
    let mut speedInc: f32;
    let mut speedIdleDec: f32;
    let speedIdle: f32;
    let speedIdleAccel: f32;
    let speedMin: f32;
    let mut speedMax: f32;
    let parent: *mut bgEntity_t = (*pVeh).m_pParentEntity;
    let isLandingOrLaunching: qboolean;
    // #elif QAGAME//MP GAME
    let curTime: c_int = (*addr_of!(level)).time;

    let parentPS: *mut playerState_t = (*parent).playerState;

    if (*parentPS).hyperSpaceTime != 0
        && curTime - (*parentPS).hyperSpaceTime < HYPERSPACE_TIME
    {
        //Going to Hyperspace
        //totally override movement
        let timeFrac = (curTime - (*parentPS).hyperSpaceTime) as f32 / HYPERSPACE_TIME as f32;
        if timeFrac < HYPERSPACE_TELEPORT_FRAC {
            //for first half, instantly jump to top speed!
            if (*parentPS).eFlags2 & EF2_HYPERSPACE == 0 {
                //waiting to face the right direction, do nothing
                (*parentPS).speed = 0.0;
            } else {
                if (*parentPS).speed < HYPERSPACE_SPEED {
                    //just started hyperspace
                    //MIKE: This is going to play the sound twice for the predicting client, I suggest using
                    //a predicted event or only doing it game-side. -rich
                    // #ifdef QAGAME//MP GAME-side
                    //G_EntitySound( ((gentity_t *)(pVeh->m_pParentEntity)), CHAN_LOCAL, pVeh->m_pVehicleInfo->soundHyper );
                }

                (*parentPS).speed = HYPERSPACE_SPEED;
            }
        } else {
            //slow from top speed to 200...
            (*parentPS).speed = 200.0
                + ((1.0 - timeFrac) * (1.0 / HYPERSPACE_TELEPORT_FRAC) * (HYPERSPACE_SPEED - 200.0));
            //don't mess with acceleration, just pop to the high velocity
            if VectorLength(&(*parentPS).velocity) < (*parentPS).speed {
                VectorScale(&(*parentPS).moveDir, (*parentPS).speed, &mut (*parentPS).velocity);
            }
        }
        return;
    }

    if (*pVeh).m_iDropTime >= curTime {
        //no speed, just drop
        (*parentPS).speed = 0.0;
        (*parentPS).gravity = 800;
        return;
    }

    isLandingOrLaunching = if FighterIsLanding(pVeh, parentPS) == QTRUE
        || FighterIsLaunching(pVeh, parentPS) == QTRUE
    {
        QTRUE
    } else {
        QFALSE
    };

    // If we are hitting the ground, just allow the fighter to go up and down.
    if isLandingOrLaunching == QTRUE//going slow enough to start landing
        && ((*pVeh).m_ucmd.forwardmove <= 0 || (*pVeh).m_LandTrace.fraction <= FIGHTER_MIN_TAKEOFF_FRACTION)
    //not trying to accelerate away already (or: you are trying to, but not high enough off the ground yet)
    {
        //FIXME: if start to move forward and fly over something low while still going relatively slow, you may try to land even though you don't mean to...
        //float fInvFrac = 1.0f - pVeh->m_LandTrace.fraction;

        if (*pVeh).m_ucmd.upmove > 0 {
            if (*parentPS).velocity[2] <= 0.0 && (*(*pVeh).m_pVehicleInfo).soundTakeOff != 0 {
                //taking off for the first time
                // #ifdef QAGAME//MP GAME-side
                G_EntitySound(
                    (*pVeh).m_pParentEntity as *mut gentity_t,
                    CHAN_AUTO,
                    (*(*pVeh).m_pVehicleInfo).soundTakeOff,
                );
            }
            (*parentPS).velocity[2] +=
                (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier; // * ( /*fInvFrac **/ 1.5f );
        } else if (*pVeh).m_ucmd.upmove < 0 {
            (*parentPS).velocity[2] -=
                (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier; // * ( /*fInvFrac **/ 1.8f );
        } else if (*pVeh).m_ucmd.forwardmove < 0 {
            if (*pVeh).m_LandTrace.fraction != 0.0 {
                (*parentPS).velocity[2] -=
                    (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
            }

            if (*pVeh).m_LandTrace.fraction <= FIGHTER_MIN_TAKEOFF_FRACTION {
                //pVeh->m_pParentEntity->client->ps.velocity[0] *= pVeh->m_LandTrace.fraction;
                //pVeh->m_pParentEntity->client->ps.velocity[1] *= pVeh->m_LandTrace.fraction;

                //remember to always base this stuff on the time modifier! otherwise, you create
                //framerate-dependancy issues and break prediction in MP -rww
                //parentPS->velocity[2] *= pVeh->m_LandTrace.fraction;
                //it's not an angle, but hey
                (*parentPS).velocity[2] = PredictedAngularDecrement(
                    (*pVeh).m_LandTrace.fraction,
                    (*pVeh).m_fTimeModifier * 5.0,
                    (*parentPS).velocity[2],
                );

                (*parentPS).speed = 0.0;
            }
        }

        // Make sure they don't pitch as they near the ground.
        //pVeh->m_vOrientation[PITCH] *= 0.7f;
        *(*pVeh).m_vOrientation.add(PITCH) = PredictedAngularDecrement(
            0.7,
            (*pVeh).m_fTimeModifier * 10.0,
            *(*pVeh).m_vOrientation.add(PITCH),
        );

        return;
    }

    // TODO: Remove-Xbox
    if (*pVeh).m_ucmd.upmove > 0 && (*(*pVeh).m_pVehicleInfo).turboSpeed != 0.0 {
        if (curTime - (*pVeh).m_iTurboTime) > (*(*pVeh).m_pVehicleInfo).turboRecharge {
            (*pVeh).m_iTurboTime = curTime + (*(*pVeh).m_pVehicleInfo).turboDuration;
            if (*(*pVeh).m_pVehicleInfo).iTurboStartFX != 0 {
                let mut i: c_int = 0;
                while i < MAX_VEHICLE_EXHAUSTS as c_int {
                    if (*pVeh).m_iExhaustTag[i as usize] == -1 {
                        break;
                    }
                    //TODO: MP Play Effect?
                    i += 1;
                }
            }
            //NOTE: turbo sound can't be part of effect if effect is played on every muzzle!
            if (*(*pVeh).m_pVehicleInfo).soundTurbo != 0 {
                // #elif QAGAME//MP GAME-side
                G_EntitySound(
                    (*pVeh).m_pParentEntity as *mut gentity_t,
                    CHAN_AUTO,
                    (*(*pVeh).m_pVehicleInfo).soundTurbo,
                );
            }
        }
    }
    speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
    if curTime < (*pVeh).m_iTurboTime {
        //going turbo speed
        speedMax = (*(*pVeh).m_pVehicleInfo).turboSpeed;
        //double our acceleration
        //speedInc *= 2.0f;
        //no no no! this would el breako el predictiono! we want the following... -rww
        speedInc = ((*(*pVeh).m_pVehicleInfo).acceleration * 2.0) * (*pVeh).m_fTimeModifier;
        //force us to move forward
        (*pVeh).m_ucmd.forwardmove = 127;
        //add flag to let cgame know to draw the iTurboFX effect
        (*parentPS).eFlags |= EF_JETPACK_ACTIVE;
    }
    /*
    //FIXME: if turbotime is up and we're waiting for it to recharge, should our max speed drop while we recharge?
    else if ( (curTime - pVeh->m_iTurboTime)<3000 )
    {//still waiting for the recharge
        speedMax = pVeh->m_pVehicleInfo->speedMax*0.75;
    }
    */
    else {
        //normal max speed
        speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;
        if (*parentPS).eFlags & EF_JETPACK_ACTIVE != 0 {
            //stop cgame from playing the turbo exhaust effect
            (*parentPS).eFlags &= !EF_JETPACK_ACTIVE;
        }
    }
    speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;
    speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle;
    speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
    speedMin = (*(*pVeh).m_pVehicleInfo).speedMin;

    if (*parentPS).brokenLimbs & (1 << SHIPSURF_DAMAGE_BACK_HEAVY) != 0 {
        //engine has taken heavy damage
        speedMax *= 0.8; //at 80% speed
    } else if (*parentPS).brokenLimbs & (1 << SHIPSURF_DAMAGE_BACK_LIGHT) != 0 {
        //engine has taken light damage
        speedMax *= 0.6; //at 60% speed
    }

    if (*pVeh).m_iRemovedSurfaces != 0 || (*parentPS).electrifyTime >= curTime {
        //go out of control
        (*parentPS).speed += speedInc;
        //Why set forwardmove?  PMove code doesn't use it... does it?
        (*pVeh).m_ucmd.forwardmove = 127;
    }
    // #ifdef QAGAME //well, the thing is always going to be inhabited if it's being predicted!
    else if FighterSuspended(pVeh, parentPS) == QTRUE {
        (*parentPS).speed = 0.0;
        (*pVeh).m_ucmd.forwardmove = 0;
    } else if ((*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) == QFALSE
        && (*parentPS).speed > 0.0
    {
        //pilot jumped out while we were moving forward (not landing or landed) so just keep the throttle locked
        //Why set forwardmove?  PMove code doesn't use it... does it?
        (*pVeh).m_ucmd.forwardmove = 127;
    } else if ((*parentPS).speed != 0.0
        || (*parentPS).groundEntityNum == ENTITYNUM_NONE
        || (*pVeh).m_ucmd.forwardmove != 0
        || (*pVeh).m_ucmd.upmove > 0)
        && (*pVeh).m_LandTrace.fraction >= 0.05
    {
        if (*pVeh).m_ucmd.forwardmove > 0 && speedInc != 0.0 {
            (*parentPS).speed += speedInc;
            (*pVeh).m_ucmd.forwardmove = 127;
        } else if (*pVeh).m_ucmd.forwardmove < 0 || (*pVeh).m_ucmd.upmove < 0 {
            //decelerating or braking
            if (*pVeh).m_ucmd.upmove < 0 {
                //braking (trying to land?), slow down faster
                if (*pVeh).m_ucmd.forwardmove != 0 {
                    //decelerator + brakes
                    speedInc += (*(*pVeh).m_pVehicleInfo).braking;
                    speedIdleDec += (*(*pVeh).m_pVehicleInfo).braking;
                } else {
                    //just brakes
                    speedInc = (*(*pVeh).m_pVehicleInfo).braking;
                    speedIdleDec = speedInc;
                }
            }
            if (*parentPS).speed > speedIdle {
                (*parentPS).speed -= speedInc;
            } else if (*parentPS).speed > speedMin {
                if FighterOverValidLandingSurface(pVeh) == QTRUE {
                    //there's ground below us and we're trying to slow down, slow down faster
                    (*parentPS).speed -= speedInc;
                } else {
                    (*parentPS).speed -= speedIdleDec;
                    if (*parentPS).speed < MIN_LANDING_SPEED as f32 {
                        //unless you can land, don't drop below the landing speed!!!  This way you can't come to a dead stop in mid-air
                        (*parentPS).speed = MIN_LANDING_SPEED as f32;
                    }
                }
            }
            if (*(*pVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
                (*pVeh).m_ucmd.forwardmove = 127;
            } else if speedMin >= 0.0 {
                (*pVeh).m_ucmd.forwardmove = 0;
            }
        }
        //else not accel, decel or braking
        else if (*(*pVeh).m_pVehicleInfo).throttleSticks != 0.0 {
            //we're using a throttle that sticks at current speed
            if (*parentPS).speed <= MIN_LANDING_SPEED as f32 {
                //going less than landing speed
                if FighterOverValidLandingSurface(pVeh) == QTRUE {
                    //close to ground and not going very fast
                    //slow to a stop if within landing height and not accel/decel/braking
                    if (*parentPS).speed > 0.0 {
                        //slow down
                        (*parentPS).speed -= speedIdleDec;
                    } else if (*parentPS).speed < 0.0 {
                        //going backwards, slow down
                        (*parentPS).speed += speedIdleDec;
                    }
                } else {
                    //not over a valid landing surf, but going too slow
                    //speed up to idle speed if not over a valid landing surf and not accel/decel/braking
                    if (*parentPS).speed < speedIdle {
                        (*parentPS).speed += speedIdleAccel;
                        if (*parentPS).speed > speedIdle {
                            (*parentPS).speed = speedIdle;
                        }
                    }
                }
            }
        } else {
            //then speed up or slow down to idle speed
            //accelerate to cruising speed only, otherwise, just coast
            // If they've launched, apply some constant motion.
            if ((*pVeh).m_LandTrace.fraction >= 1.0 //no ground
                    || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE)//or can't land on ground below us
                && speedIdle > 0.0
            {
                //not above ground and have an idle speed
                //float fSpeed = pVeh->m_pParentEntity->client->ps.speed;
                if (*parentPS).speed < speedIdle {
                    (*parentPS).speed += speedIdleAccel;
                    if (*parentPS).speed > speedIdle {
                        (*parentPS).speed = speedIdle;
                    }
                } else if (*parentPS).speed > 0.0 {
                    //slow down
                    (*parentPS).speed -= speedIdleDec;

                    if (*parentPS).speed < speedIdle {
                        (*parentPS).speed = speedIdle;
                    }
                }
            } else
            //either close to ground or no idle speed
            {
                //slow to a stop if no idle speed or within landing height and not accel/decel/braking
                if (*parentPS).speed > 0.0 {
                    //slow down
                    (*parentPS).speed -= speedIdleDec;
                } else if (*parentPS).speed < 0.0 {
                    //going backwards, slow down
                    (*parentPS).speed += speedIdleDec;
                }
            }
        }
    } else {
        if (*pVeh).m_ucmd.forwardmove < 0 {
            (*pVeh).m_ucmd.forwardmove = 0;
        }
        if (*pVeh).m_ucmd.upmove < 0 {
            (*pVeh).m_ucmd.upmove = 0;
        }
    }

    //This is working now, but there are some transitional jitters... Rich?
    //STRAFING==============================================================================
    if (*(*pVeh).m_pVehicleInfo).strafePerc != 0.0
        // #ifdef QAGAME//only do this check on GAME side
        && ((*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) == QTRUE //has to have a driver in order to be capable of landing
        && (*pVeh).m_iRemovedSurfaces == 0
        && (*parentPS).electrifyTime < curTime
        && (*parentPS).vehTurnaroundTime < curTime
        && ((*pVeh).m_LandTrace.fraction >= 1.0//no grounf
            || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE//can't land here
            || (*parentPS).speed > MIN_LANDING_SPEED as f32)//going too fast to land
        && (*pVeh).m_ucmd.rightmove != 0
    {
        //strafe
        let mut vAngles: vec3_t = [0.0; 3];
        let mut vRight: vec3_t = [0.0; 3];
        let mut strafeSpeed = ((*(*pVeh).m_pVehicleInfo).strafePerc * speedMax) * 5.0;
        VectorCopy(&*(*pVeh).m_vOrientation.cast::<vec3_t>(), &mut vAngles);
        vAngles[ROLL] = 0.0;
        vAngles[PITCH] = vAngles[ROLL];
        AngleVectors(&vAngles, None, Some(&mut vRight), None);

        if (*pVeh).m_ucmd.rightmove > 0 {
            //strafe right
            //FIXME: this will probably make it possible to cheat and
            //		go faster than max speed if you keep turning and strafing...
            if (*parentPS).hackingTime > -(MAX_STRAFE_TIME as c_int) {
                //can strafe right for 2 seconds
                let curStrafeSpeed = DotProduct(&(*parentPS).velocity, &vRight);
                if curStrafeSpeed > 0.0 {
                    //if > 0, already strafing right
                    strafeSpeed -= curStrafeSpeed; //so it doesn't add up
                }
                if strafeSpeed > 0.0 {
                    VectorMA(
                        &(*parentPS).velocity,
                        strafeSpeed * (*pVeh).m_fTimeModifier,
                        &vRight,
                        &mut (*parentPS).velocity,
                    );
                }
                (*parentPS).hackingTime -= (50.0 * (*pVeh).m_fTimeModifier) as c_int;
            }
        } else {
            //strafe left
            if (*parentPS).hackingTime < MAX_STRAFE_TIME as c_int {
                //can strafe left for 2 seconds
                let curStrafeSpeed = DotProduct(&(*parentPS).velocity, &vRight);
                if curStrafeSpeed < 0.0 {
                    //if < 0, already strafing left
                    strafeSpeed += curStrafeSpeed; //so it doesn't add up
                }
                if strafeSpeed > 0.0 {
                    VectorMA(
                        &(*parentPS).velocity,
                        -strafeSpeed * (*pVeh).m_fTimeModifier,
                        &vRight,
                        &mut (*parentPS).velocity,
                    );
                }
                (*parentPS).hackingTime += (50.0 * (*pVeh).m_fTimeModifier) as c_int;
            }
        }
        //strafing takes away from forward speed?  If so, strafePerc above should use speedMax
        //parentPS->speed *= (1.0f-pVeh->m_pVehicleInfo->strafePerc);
    } else
    //if ( parentPS->hackingTimef )
    {
        if (*parentPS).hackingTime > 0 {
            (*parentPS).hackingTime -= (50.0 * (*pVeh).m_fTimeModifier) as c_int;
            if (*parentPS).hackingTime < 0 {
                (*parentPS).hackingTime = 0;
            }
        } else if (*parentPS).hackingTime < 0 {
            (*parentPS).hackingTime += (50.0 * (*pVeh).m_fTimeModifier) as c_int;
            if (*parentPS).hackingTime > 0 {
                (*parentPS).hackingTime = 0;
            }
        }
    }
    //STRAFING==============================================================================

    if (*parentPS).speed > speedMax {
        (*parentPS).speed = speedMax;
    } else if (*parentPS).speed < speedMin {
        (*parentPS).speed = speedMin;
    }

    // #ifdef QAGAME//FIXME: get working in GAME and CGAME
    if *(*pVeh).m_vOrientation.add(PITCH) * 0.1 > 10.0 {
        //pitched downward, increase speed more and more based on our tilt
        if FighterIsInSpace(parent as *mut gentity_t) == QTRUE {
            //in space, do nothing with speed base on pitch...
        } else {
            //really should only do this when on a planet
            let mut mult = *(*pVeh).m_vOrientation.add(PITCH) * 0.1;
            if mult < 1.0 {
                mult = 1.0;
            }
            (*parentPS).speed = PredictedAngularDecrement(
                mult,
                (*pVeh).m_fTimeModifier * 10.0,
                (*parentPS).speed,
            );
        }
    }

    if (*pVeh).m_iRemovedSurfaces != 0 || (*parentPS).electrifyTime >= curTime {
        //going down
        if FighterIsInSpace(parent as *mut gentity_t) == QTRUE {
            //we're in a valid trigger_space brush
            //simulate randomness
            if (*parent).s.number & 3 == 0 {
                //even multiple of 3, don't do anything
                (*parentPS).gravity = 0;
            } else if (*parent).s.number & 2 == 0 {
                //even multiple of 2, go up
                (*parentPS).gravity = -500;
                (*parentPS).velocity[2] = 80.0;
            } else {
                //odd number, go down
                (*parentPS).gravity = 500;
                (*parentPS).velocity[2] = -80.0;
            }
        } else {
            //over a planet
            (*parentPS).gravity = 500;
            (*parentPS).velocity[2] = -80.0;
        }
    } else if FighterSuspended(pVeh, parentPS) == QTRUE {
        (*parentPS).gravity = 0;
    } else if ((*parentPS).speed == 0.0 || (*parentPS).speed < speedIdle)
        && (*pVeh).m_ucmd.upmove <= 0
    {
        //slowing down or stopped and not trying to take off
        if FighterIsInSpace(parent as *mut gentity_t) == QTRUE {
            //we're in space, stopping doesn't make us drift downward
            if FighterOverValidLandingSurface(pVeh) == QTRUE {
                //well, there's something below us to land on, so go ahead and lower us down to it
                (*parentPS).gravity = ((speedIdle - (*parentPS).speed) / 4.0) as c_int;
            }
        } else {
            //over a planet
            (*parentPS).gravity = ((speedIdle - (*parentPS).speed) / 4.0) as c_int;
        }
    } else {
        (*parentPS).gravity = 0;
    }

    /********************************************************************************/
    /*	END Here is where we move the vehicle (forward or back or whatever). END	*/
    /********************************************************************************/
}

pub unsafe extern "C" fn FighterWingMalfunctionCheck(
    pVeh: *mut Vehicle_t,
    parentPS: *mut playerState_t,
) {
    let mut mPitchOverride: f32 = 1.0;
    let mut mYawOverride: f32 = 1.0;
    BG_VehicleTurnRateForSpeed(pVeh, (*parentPS).speed, &mut mPitchOverride, &mut mYawOverride);
    //check right wing damage
    if (*parentPS).brokenLimbs & (1 << SHIPSURF_DAMAGE_RIGHT_HEAVY) != 0 {
        //right wing has taken heavy damage
        *(*pVeh).m_vOrientation.add(ROLL) +=
            (((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() as f32 + 1.0)
                * (*pVeh).m_fTimeModifier
                * mYawOverride
                * 50.0;
    } else if (*parentPS).brokenLimbs & (1 << SHIPSURF_DAMAGE_RIGHT_LIGHT) != 0 {
        //right wing has taken light damage
        *(*pVeh).m_vOrientation.add(ROLL) +=
            (((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() as f32 + 1.0)
                * (*pVeh).m_fTimeModifier
                * mYawOverride
                * 12.5;
    }

    //check left wing damage
    if (*parentPS).brokenLimbs & (1 << SHIPSURF_DAMAGE_LEFT_HEAVY) != 0 {
        //left wing has taken heavy damage
        *(*pVeh).m_vOrientation.add(ROLL) -=
            (((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() as f32 + 1.0)
                * (*pVeh).m_fTimeModifier
                * mYawOverride
                * 50.0;
    } else if (*parentPS).brokenLimbs & (1 << SHIPSURF_DAMAGE_LEFT_LIGHT) != 0 {
        //left wing has taken light damage
        *(*pVeh).m_vOrientation.add(ROLL) -=
            (((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() as f32 + 1.0)
                * (*pVeh).m_fTimeModifier
                * mYawOverride
                * 12.5;
    }
}

pub unsafe extern "C" fn FighterNoseMalfunctionCheck(
    pVeh: *mut Vehicle_t,
    parentPS: *mut playerState_t,
) {
    let mut mPitchOverride: f32 = 1.0;
    let mut mYawOverride: f32 = 1.0;
    BG_VehicleTurnRateForSpeed(pVeh, (*parentPS).speed, &mut mPitchOverride, &mut mYawOverride);
    //check nose damage
    if (*parentPS).brokenLimbs & (1 << SHIPSURF_DAMAGE_FRONT_HEAVY) != 0 {
        //nose has taken heavy damage
        //pitch up and down over time
        *(*pVeh).m_vOrientation.add(PITCH) +=
            ((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() as f32
                * (*pVeh).m_fTimeModifier
                * mPitchOverride
                * 50.0;
    } else if (*parentPS).brokenLimbs & (1 << SHIPSURF_DAMAGE_FRONT_LIGHT) != 0 {
        //nose has taken heavy damage
        //pitch up and down over time
        *(*pVeh).m_vOrientation.add(PITCH) +=
            ((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() as f32
                * (*pVeh).m_fTimeModifier
                * mPitchOverride
                * 20.0;
    }
}

// TODO: Port-Bug
pub unsafe extern "C" fn FighterDamageRoutine(
    pVeh: *mut Vehicle_t,
    _parent: *mut bgEntity_t,
    parentPS: *mut playerState_t,
    riderPS: *mut playerState_t,
    isDead: qboolean,
) {
    if (*pVeh).m_iRemovedSurfaces == 0 {
        //still in one piece
        if !(*pVeh).m_pParentEntity.is_null() && isDead == QTRUE {
            //death spiral
            (*pVeh).m_ucmd.upmove = 0;
            //FIXME: don't bias toward pitching down when not in space
            /*
            if ( FighterIsInSpace( pVeh->m_pParentEntity ) )
            {
            }
            else
            */
            if (*(*pVeh).m_pParentEntity).s.number % 3 != 0 {
                //NOT everyone should do this
                *(*pVeh).m_vOrientation.add(PITCH) += (*pVeh).m_fTimeModifier;
                if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QFALSE {
                    if *(*pVeh).m_vOrientation.add(PITCH) > 60.0 {
                        *(*pVeh).m_vOrientation.add(PITCH) = 60.0;
                    }
                }
            } else if (*(*pVeh).m_pParentEntity).s.number % 2 != 0 {
                *(*pVeh).m_vOrientation.add(PITCH) -= (*pVeh).m_fTimeModifier;
                if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QFALSE {
                    if *(*pVeh).m_vOrientation.add(PITCH) > -60.0 {
                        *(*pVeh).m_vOrientation.add(PITCH) = -60.0;
                    }
                }
            }
            if (*(*pVeh).m_pParentEntity).s.number % 2 != 0 {
                *(*pVeh).m_vOrientation.add(YAW) += (*pVeh).m_fTimeModifier;
                *(*pVeh).m_vOrientation.add(ROLL) += (*pVeh).m_fTimeModifier * 4.0;
            } else {
                *(*pVeh).m_vOrientation.add(YAW) -= (*pVeh).m_fTimeModifier;
                *(*pVeh).m_vOrientation.add(ROLL) -= (*pVeh).m_fTimeModifier * 4.0;
            }
        }
        return;
    }

    //if we get into here we have at least one broken piece
    (*pVeh).m_ucmd.upmove = 0;

    //if you're off the ground and not suspended, pitch down
    //FIXME: not in space!
    if (*pVeh).m_LandTrace.fraction >= 0.1 {
        if FighterSuspended(pVeh, parentPS) == QFALSE {
            //pVeh->m_ucmd.forwardmove = 0;
            //FIXME: don't bias towards pitching down when in space...
            if (*(*pVeh).m_pParentEntity).s.number % 3 == 0 {
                //NOT everyone should do this
                *(*pVeh).m_vOrientation.add(PITCH) += (*pVeh).m_fTimeModifier;
                if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QFALSE {
                    if *(*pVeh).m_vOrientation.add(PITCH) > 60.0 {
                        *(*pVeh).m_vOrientation.add(PITCH) = 60.0;
                    }
                }
            } else if (*(*pVeh).m_pParentEntity).s.number % 4 == 0 {
                *(*pVeh).m_vOrientation.add(PITCH) -= (*pVeh).m_fTimeModifier;
                if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QFALSE {
                    if *(*pVeh).m_vOrientation.add(PITCH) > -60.0 {
                        *(*pVeh).m_vOrientation.add(PITCH) = -60.0;
                    }
                }
            }
            //else: just keep going forward
        }
    }
    // #ifdef QAGAME
    if (*pVeh).m_LandTrace.fraction < 1.0 {
        //if you land at all when pieces of your ship are missing, then die
        let parent: *mut gentity_t = (*pVeh).m_pParentEntity as *mut gentity_t;
        //only have this info in MP...
        G_DamageFromKiller(
            parent,
            parent,
            core::ptr::null_mut(),
            &mut (*(*parent).client).ps.origin,
            999999,
            DAMAGE_NO_ARMOR,
            MOD_SUICIDE,
        );
    }

    if (((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C) != 0
        || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D) != 0)
        && (((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E) != 0
            || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F) != 0)
    {
        //wings on both side broken
        let mut factor: f32 = 2.0;
        if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E) != 0
            && ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F) != 0
            && ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C) != 0
            && ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D) != 0
        {
            //all wings broken
            factor *= 2.0;
        }

        if (*(*pVeh).m_pParentEntity).s.number % 2 == 0
            || (*(*pVeh).m_pParentEntity).s.number % 6 == 0
        {
            //won't yaw, so increase roll factor
            factor *= 4.0;
        }

        *(*pVeh).m_vOrientation.add(ROLL) += (*pVeh).m_fTimeModifier * factor; //do some spiralling
    } else if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C) != 0
        || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D) != 0
    {
        //left wing broken
        let mut factor: f32 = 2.0;
        if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C) != 0
            && ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D) != 0
        {
            //if both are broken..
            factor *= 2.0;
        }

        if (*(*pVeh).m_pParentEntity).s.number % 2 == 0
            || (*(*pVeh).m_pParentEntity).s.number % 6 == 0
        {
            //won't yaw, so increase roll factor
            factor *= 4.0;
        }

        *(*pVeh).m_vOrientation.add(ROLL) += factor * (*pVeh).m_fTimeModifier;
    } else if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E) != 0
        || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F) != 0
    {
        //right wing broken
        let mut factor: f32 = 2.0;
        if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E) != 0
            && ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F) != 0
        {
            //if both are broken..
            factor *= 2.0;
        }

        if (*(*pVeh).m_pParentEntity).s.number % 2 == 0
            || (*(*pVeh).m_pParentEntity).s.number % 6 == 0
        {
            //won't yaw, so increase roll factor
            factor *= 4.0;
        }

        *(*pVeh).m_vOrientation.add(ROLL) -= factor * (*pVeh).m_fTimeModifier;
    }
}

//MP RULE - ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
//If you really need to violate this rule for SP, then use ifdefs.
//By BG-compatible, I mean no use of game-specific data - ONLY use
//stuff available in the MP bgEntity (in SP, the bgEntity is #defined
//as a gentity, but the MP-compatible access restrictions are based
//on the bgEntity structure in the MP codebase) -rww
// ProcessOrientCommands the Vehicle.
#[allow(unused_assignments)] // faithful `float curRoll = 0.0f;` init (C reassigns before read)
pub unsafe extern "C" fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    /********************************************************************************/
    /*	BEGIN	Here is where make sure the vehicle is properly oriented.	BEGIN	*/
    /********************************************************************************/

    let parent: *mut bgEntity_t = (*pVeh).m_pParentEntity;
    let parentPS: *mut playerState_t;
    let riderPS: *mut playerState_t;
    let angleTimeMod: f32;
    // #ifdef QAGAME
    let groundFraction: f32 = 0.1;
    let mut curRoll: f32 = 0.0;
    let isDead: qboolean;
    let isLandingOrLanded: qboolean;
    // #elif QAGAME//MP GAME
    let curTime: c_int = (*addr_of!(level)).time;

    let mut rider: *mut bgEntity_t = core::ptr::null_mut();
    if (*parent).s.owner != ENTITYNUM_NONE {
        rider = PM_BGEntForNum((*parent).s.owner); //&g_entities[parent->r.ownerNum];
    }

    if rider.is_null() {
        rider = parent;
    }

    parentPS = (*parent).playerState;
    riderPS = (*rider).playerState;
    isDead = if (*parentPS).eFlags & EF_DEAD != 0 { QTRUE } else { QFALSE };

    if (*parentPS).hyperSpaceTime != 0
        && (curTime - (*parentPS).hyperSpaceTime) < HYPERSPACE_TIME
    {
        //Going to Hyperspace
        VectorCopy(&(*riderPS).viewangles, &mut *(*pVeh).m_vOrientation.cast::<vec3_t>());
        VectorCopy(&(*riderPS).viewangles, &mut (*parentPS).viewangles);
        return;
    }

    if (*pVeh).m_iDropTime >= curTime {
        //you can only YAW during this
        *(*pVeh).m_vOrientation.add(YAW) = (*riderPS).viewangles[YAW];
        (*parentPS).viewangles[YAW] = *(*pVeh).m_vOrientation.add(YAW);
        return;
    }

    angleTimeMod = (*pVeh).m_fTimeModifier;

    if isDead == QTRUE
        || (*parentPS).electrifyTime >= curTime
        || ((*(*pVeh).m_pVehicleInfo).surfDestruction != 0
            && (*pVeh).m_iRemovedSurfaces != 0
            && ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C) != 0
            && ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D) != 0
            && ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E) != 0
            && ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F) != 0)
    {
        //do some special stuff for when all the wings are torn off
        FighterDamageRoutine(pVeh, parent, parentPS, riderPS, isDead);
        *(*pVeh).m_vOrientation.add(ROLL) = AngleNormalize180(*(*pVeh).m_vOrientation.add(ROLL));
        return;
    }

    if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QFALSE {
        *(*pVeh).m_vOrientation.add(ROLL) = PredictedAngularDecrement(
            0.95,
            angleTimeMod * 2.0,
            *(*pVeh).m_vOrientation.add(ROLL),
        );
    }

    isLandingOrLanded = if FighterIsLanding(pVeh, parentPS) == QTRUE
        || FighterIsLanded(pVeh, parentPS) == QTRUE
    {
        QTRUE
    } else {
        QFALSE
    };

    if isLandingOrLanded == QFALSE {
        //don't do this stuff while landed.. I guess. I don't want ships spinning in place, looks silly.
        let mut m: c_int = 0;
        let mut aVelDif: f32;
        let mut dForVel: f32;

        FighterWingMalfunctionCheck(pVeh, parentPS);

        while m < 3 {
            aVelDif = (*pVeh).m_vFullAngleVelocity[m as usize];

            if aVelDif != 0.0 {
                dForVel = (aVelDif * 0.1) * (*pVeh).m_fTimeModifier;
                if dForVel > 1.0 || dForVel < -1.0 {
                    *(*pVeh).m_vOrientation.add(m as usize) += dForVel;
                    *(*pVeh).m_vOrientation.add(m as usize) =
                        AngleNormalize180(*(*pVeh).m_vOrientation.add(m as usize));
                    if m == PITCH as c_int {
                        //don't pitch downward into ground even more.
                        if *(*pVeh).m_vOrientation.add(m as usize) > 90.0
                            && (*(*pVeh).m_vOrientation.add(m as usize) - dForVel) < 90.0
                        {
                            *(*pVeh).m_vOrientation.add(m as usize) = 90.0;
                            (*pVeh).m_vFullAngleVelocity[m as usize] =
                                -(*pVeh).m_vFullAngleVelocity[m as usize];
                        }
                    }
                    (*pVeh).m_vFullAngleVelocity[m as usize] -= dForVel;
                } else {
                    (*pVeh).m_vFullAngleVelocity[m as usize] = 0.0;
                }
            }

            m += 1;
        }
    } else {
        //clear decr/incr angles once landed.
        VectorClear(&mut (*pVeh).m_vFullAngleVelocity);
    }

    curRoll = *(*pVeh).m_vOrientation.add(ROLL);

    // If we're landed, we shouldn't be able to do anything but take off.
    if isLandingOrLanded == QTRUE//going slow enough to start landing
        && (*pVeh).m_iRemovedSurfaces == 0
        && (*parentPS).electrifyTime < curTime
    //not spiraling out of control
    {
        if (*parentPS).speed > 0.0 {
            //Uh... what?  Why?
            if (*pVeh).m_LandTrace.fraction < 0.3 {
                *(*pVeh).m_vOrientation.add(PITCH) = 0.0;
            } else {
                *(*pVeh).m_vOrientation.add(PITCH) = PredictedAngularDecrement(
                    0.83,
                    angleTimeMod * 10.0,
                    *(*pVeh).m_vOrientation.add(PITCH),
                );
            }
        }
        if (*pVeh).m_LandTrace.fraction > 0.1
            || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE
        {
            //off the ground, at least (or not on a valid landing surf)
            // Dampen the turn rate based on the current height.
            FighterYawAdjust(pVeh, riderPS, parentPS);
        }
    } else if ((*pVeh).m_iRemovedSurfaces != 0 || (*parentPS).electrifyTime >= curTime)//spiralling out of control
        && ((*(*pVeh).m_pParentEntity).s.number % 2 == 0
            || (*(*pVeh).m_pParentEntity).s.number % 6 == 0)
    {
        //no yaw control
    } else if !(*pVeh).m_pPilot.is_null()
        && (*(*pVeh).m_pPilot).s.number < MAX_CLIENTS as c_int
        && (*parentPS).speed > 0.0
    //&& !( pVeh->m_ucmd.forwardmove > 0 && pVeh->m_LandTrace.fraction != 1.0f )
    {
        if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QTRUE {
            VectorCopy(&(*riderPS).viewangles, &mut *(*pVeh).m_vOrientation.cast::<vec3_t>());
            VectorCopy(&(*riderPS).viewangles, &mut (*parentPS).viewangles);
            //BG_ExternThisSoICanRecompileInDebug( pVeh, riderPS );

            curRoll = *(*pVeh).m_vOrientation.add(ROLL);

            FighterNoseMalfunctionCheck(pVeh, parentPS);

            //VectorCopy( pVeh->m_vOrientation, parentPS->viewangles );
        } else {
            /*
            float fTurnAmt[3];
            //PITCH
            fTurnAmt[PITCH] = riderPS->viewangles[PITCH] * 0.08f;
            //YAW
            fTurnAmt[YAW] = riderPS->viewangles[YAW] * 0.065f;
            fTurnAmt[YAW] *= fTurnAmt[YAW];
            // Dampen the turn rate based on the current height.
            if ( riderPS->viewangles[YAW] < 0 )
            {//must keep it negative because squaring a negative makes it positive
                fTurnAmt[YAW] = -fTurnAmt[YAW];
            }
            fTurnAmt[YAW] *= pVeh->m_LandTrace.fraction;
            //ROLL
            fTurnAmt[2] = 0.0f;
            */

            //Actal YAW
            /*
            pVeh->m_vOrientation[ROLL] = curRoll;
            FighterRollAdjust(pVeh, riderPS, parentPS);
            curRoll = pVeh->m_vOrientation[ROLL];
            */
            FighterYawAdjust(pVeh, riderPS, parentPS);

            // If we are not hitting the ground, allow the fighter to pitch up and down.
            if FighterOverValidLandingSurface(pVeh) == QFALSE
                || (*parentPS).speed > MIN_LANDING_SPEED as f32
            //if ( ( pVeh->m_LandTrace.fraction >= 1.0f || pVeh->m_ucmd.forwardmove != 0 ) && pVeh->m_LandTrace.fraction >= 0.0f )
            {
                let mut fYawDelta: f32;

                FighterPitchAdjust(pVeh, riderPS, parentPS);

                FighterNoseMalfunctionCheck(pVeh, parentPS);

                // Adjust the roll based on the turn amount and dampen it a little.
                fYawDelta = AngleSubtract(
                    *(*pVeh).m_vOrientation.add(YAW),
                    (*pVeh).m_vPrevOrientation[YAW],
                ); //pVeh->m_vOrientation[YAW] - pVeh->m_vPrevOrientation[YAW];
                if fYawDelta > 8.0 {
                    fYawDelta = 8.0;
                } else if fYawDelta < -8.0 {
                    fYawDelta = -8.0;
                }
                curRoll -= fYawDelta;
                curRoll = PredictedAngularDecrement(0.93, angleTimeMod * 2.0, curRoll);

                //cap it reasonably
                //NOTE: was hardcoded to 40.0f, now using extern data
                if (*(*pVeh).m_pVehicleInfo).rollLimit != -1.0 {
                    if curRoll > (*(*pVeh).m_pVehicleInfo).rollLimit {
                        curRoll = (*(*pVeh).m_pVehicleInfo).rollLimit;
                    } else if curRoll < -(*(*pVeh).m_pVehicleInfo).rollLimit {
                        curRoll = -(*(*pVeh).m_pVehicleInfo).rollLimit;
                    }
                }
            }
        }
    }

    // If you are directly impacting the ground, even out your pitch.
    if isLandingOrLanded == QTRUE {
        //only if capable of landing
        if isDead == QFALSE
            && (*parentPS).electrifyTime < curTime
            && ((*(*pVeh).m_pVehicleInfo).surfDestruction == 0 || (*pVeh).m_iRemovedSurfaces == 0)
        {
            //not crashing or spiralling out of control...
            if *(*pVeh).m_vOrientation.add(PITCH) > 0.0 {
                *(*pVeh).m_vOrientation.add(PITCH) = PredictedAngularDecrement(
                    0.2,
                    angleTimeMod * 10.0,
                    *(*pVeh).m_vOrientation.add(PITCH),
                );
            } else {
                *(*pVeh).m_vOrientation.add(PITCH) = PredictedAngularDecrement(
                    0.75,
                    angleTimeMod * 10.0,
                    *(*pVeh).m_vOrientation.add(PITCH),
                );
            }
        }
    }

    /*
    //NOTE: all this is redundant now since we have the FighterDamageRoutine func...
    ... (Q_irand/Q_flrand erratic-death block, fully commented out in source) ...
    */
    // If no one is in this vehicle and it's up in the sky, pitch it forward as it comes tumbling down.
    // #ifdef QAGAME //never gonna happen on client anyway
    if ((*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) == QFALSE
        && (*pVeh).m_LandTrace.fraction >= groundFraction
        && FighterIsInSpace(parent as *mut gentity_t) == QFALSE
        && FighterSuspended(pVeh, parentPS) == QFALSE
    {
        (*pVeh).m_ucmd.upmove = 0;
        //pVeh->m_ucmd.forwardmove = 0;
        *(*pVeh).m_vOrientation.add(PITCH) += (*pVeh).m_fTimeModifier;
        if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QFALSE {
            if *(*pVeh).m_vOrientation.add(PITCH) > 60.0 {
                *(*pVeh).m_vOrientation.add(PITCH) = 60.0;
            }
        }
    }

    if (*parentPS).hackingTime == 0 {
        //use that roll
        *(*pVeh).m_vOrientation.add(ROLL) = curRoll;
        //NOTE: this seems really backwards...
        if *(*pVeh).m_vOrientation.add(ROLL) != 0.0 {
            //continually adjust the yaw based on the roll..
            if ((*pVeh).m_iRemovedSurfaces != 0 || (*parentPS).electrifyTime >= curTime)//spiralling out of control
                && ((*(*pVeh).m_pParentEntity).s.number % 2 == 0
                    || (*(*pVeh).m_pParentEntity).s.number % 6 == 0)
            {
                //leave YAW alone
            } else {
                if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QFALSE {
                    *(*pVeh).m_vOrientation.add(YAW) -=
                        ((*(*pVeh).m_vOrientation.add(ROLL)) * 0.05) * (*pVeh).m_fTimeModifier;
                }
            }
        }
    } else {
        //add in strafing roll
        let strafeRoll =
            ((*parentPS).hackingTime as f32 / MAX_STRAFE_TIME) * (*(*pVeh).m_pVehicleInfo).rollLimit; //pVeh->m_pVehicleInfo->bankingSpeed*
        let strafeDif = AngleSubtract(strafeRoll, *(*pVeh).m_vOrientation.add(ROLL));
        *(*pVeh).m_vOrientation.add(ROLL) += (strafeDif * 0.1) * (*pVeh).m_fTimeModifier;
        if BG_UnrestrainedPitchRoll(riderPS, pVeh) == QFALSE {
            //cap it reasonably
            if (*(*pVeh).m_pVehicleInfo).rollLimit != -1.0
                && (*pVeh).m_iRemovedSurfaces == 0
                && (*parentPS).electrifyTime < curTime
            {
                if *(*pVeh).m_vOrientation.add(ROLL) > (*(*pVeh).m_pVehicleInfo).rollLimit {
                    *(*pVeh).m_vOrientation.add(ROLL) = (*(*pVeh).m_pVehicleInfo).rollLimit;
                } else if *(*pVeh).m_vOrientation.add(ROLL) < -(*(*pVeh).m_pVehicleInfo).rollLimit {
                    *(*pVeh).m_vOrientation.add(ROLL) = -(*(*pVeh).m_pVehicleInfo).rollLimit;
                }
            }
        }
    }

    if (*(*pVeh).m_pVehicleInfo).surfDestruction != 0 {
        FighterDamageRoutine(pVeh, parent, parentPS, riderPS, isDead);
    }
    *(*pVeh).m_vOrientation.add(ROLL) = AngleNormalize180(*(*pVeh).m_vOrientation.add(ROLL));

    /********************************************************************************/
    /*	END	Here is where make sure the vehicle is properly oriented.	END			*/
    /********************************************************************************/
}

// #ifdef QAGAME //ONLY in SP or on server, not cgame

// This function makes sure that the vehicle is properly animated.
#[allow(unused_assignments)] // faithful `qboolean isLanding/isLanded = qfalse;` inits
pub unsafe extern "C" fn AnimateVehicle(pVeh: *mut Vehicle_t) {
    let mut Anim: c_int = -1;
    let iFlags: c_int = SETANIM_FLAG_NORMAL; //iFlags = SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD only on SP gear-close path, excluded
    let iBlend: c_int = 300;
    let mut isLanding: qboolean = QFALSE;
    let mut isLanded: qboolean = QFALSE;
    let parentPS: *mut playerState_t = (*(*pVeh).m_pParentEntity).playerState;
    // #elif QAGAME//MP GAME
    let curTime: c_int = (*addr_of!(level)).time;

    if (*parentPS).hyperSpaceTime != 0
        && curTime - (*parentPS).hyperSpaceTime < HYPERSPACE_TIME
    {
        //Going to Hyperspace
        //close the wings (FIXME: makes sense on X-Wing, not Shuttle?)
        if (*pVeh).m_ulFlags & VEH_WINGSOPEN as c_ulong != 0 {
            (*pVeh).m_ulFlags &= !(VEH_WINGSOPEN as c_ulong);
            Anim = BOTH_WINGS_CLOSE;
        }
    } else {
        isLanding = FighterIsLanding(pVeh, parentPS);
        isLanded = FighterIsLanded(pVeh, parentPS);

        // if we're above launch height (way up in the air)...
        if isLanding == QFALSE && isLanded == QFALSE {
            if (*pVeh).m_ulFlags & VEH_WINGSOPEN as c_ulong == 0 {
                (*pVeh).m_ulFlags |= VEH_WINGSOPEN as c_ulong;
                (*pVeh).m_ulFlags &= !(VEH_GEARSOPEN as c_ulong);
                Anim = BOTH_WINGS_OPEN;
            }
        }
        // otherwise we're below launch height and still taking off.
        else {
            if ((*pVeh).m_ucmd.forwardmove < 0 || (*pVeh).m_ucmd.upmove < 0 || isLanded == QTRUE)
                && (*pVeh).m_LandTrace.fraction <= 0.4
                && (*pVeh).m_LandTrace.plane.normal[2] >= MIN_LANDING_SLOPE
            {
                //already landed or trying to land and close to ground
                // Open gears.
                if (*pVeh).m_ulFlags & VEH_GEARSOPEN as c_ulong == 0 {
                    if (*(*pVeh).m_pVehicleInfo).soundLand != 0 {
                        //just landed?
                        // #ifdef QAGAME//MP GAME-side
                        G_EntitySound(
                            (*pVeh).m_pParentEntity as *mut gentity_t,
                            CHAN_AUTO,
                            (*(*pVeh).m_pVehicleInfo).soundLand,
                        );
                    }
                    (*pVeh).m_ulFlags |= VEH_GEARSOPEN as c_ulong;
                    Anim = BOTH_GEARS_OPEN;
                }
            } else {
                //trying to take off and almost halfway off the ground
                // Close gears (if they're open).
                if (*pVeh).m_ulFlags & VEH_GEARSOPEN as c_ulong != 0 {
                    (*pVeh).m_ulFlags &= !(VEH_GEARSOPEN as c_ulong);
                    Anim = BOTH_GEARS_CLOSE;
                    //iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                }
                // If gears are closed, and we are below launch height, close the wings.
                else {
                    if (*pVeh).m_ulFlags & VEH_WINGSOPEN as c_ulong != 0 {
                        (*pVeh).m_ulFlags &= !(VEH_WINGSOPEN as c_ulong);
                        Anim = BOTH_WINGS_CLOSE;
                    }
                }
            }
        }
    }

    if Anim != -1 {
        BG_SetAnim(
            (*(*pVeh).m_pParentEntity).playerState,
            (*addr_of!(bgAllAnims))[(*(*pVeh).m_pParentEntity).localAnimIndex as usize].anims,
            SETANIM_BOTH,
            Anim,
            iFlags,
            iBlend,
        );
    }
}

// This function makes sure that the rider's in this vehicle are properly animated.
pub unsafe extern "C" fn AnimateRiders(_pVeh: *mut Vehicle_t) {}

// #endif //game-only

// #ifndef QAGAME
// void AttachRidersGeneric( Vehicle_t *pVeh ); — cgame-only, excluded on the server build.
// #endif

pub unsafe extern "C" fn G_SetFighterVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    // #ifdef QAGAME //ONLY in SP or on server, not cgame
    (*pVehInfo).AnimateVehicle = Some(AnimateVehicle);
    (*pVehInfo).AnimateRiders = Some(AnimateRiders);
    //	pVehInfo->ValidateBoard				=		ValidateBoard;
    //	pVehInfo->SetParent					=		SetParent;
    //	pVehInfo->SetPilot					=		SetPilot;
    //	pVehInfo->AddPassenger				=		AddPassenger;
    //	pVehInfo->Animate					=		Animate;
    (*pVehInfo).Board = Some(Board);
    (*pVehInfo).Eject = Some(Eject);
    //	pVehInfo->EjectAll					=		EjectAll;
    //	pVehInfo->StartDeathDelay			=		StartDeathDelay;
    //	pVehInfo->DeathUpdate				=		DeathUpdate;
    //	pVehInfo->RegisterAssets			=		RegisterAssets;
    //	pVehInfo->Initialize				=		Initialize;
    (*pVehInfo).Update = Some(Update);
    //	pVehInfo->UpdateRider				=		UpdateRider;
    // #endif //game-only
    (*pVehInfo).ProcessMoveCommands = Some(ProcessMoveCommands);
    (*pVehInfo).ProcessOrientCommands = Some(ProcessOrientCommands);

    // #ifndef QAGAME (cgame prediction attachment) — server build excludes this:
    //	pVehInfo->AttachRiders				=		AttachRidersGeneric;
    //	pVehInfo->AttachRiders				=		AttachRiders;
    //	pVehInfo->Ghost						=		Ghost;
    //	pVehInfo->UnGhost					=		UnGhost;
    //	pVehInfo->Inhabited					=		Inhabited;
}

// Create/Allocate a new Animal Vehicle (initializing it as well).
//
// No-oracle: allocates from the module-static vehicle pool via
// `G_AllocateVehicleObject` and zeroes/initialises `Vehicle_t` through pointers.
pub unsafe extern "C" fn G_CreateFighterNPC(pVeh: *mut *mut Vehicle_t, strType: *const c_char) {
    // Allocate the Vehicle.
    // #ifdef _JK2MP / #ifdef QAGAME (server build):
    //these will remain on entities on the client once allocated because the pointer is
    //never stomped. on the server, however, when an ent is freed, the entity struct is
    //memset to 0, so this memory would be lost..
    G_AllocateVehicleObject(pVeh);
    write_bytes(*pVeh, 0, 1); // memset(*pVeh, 0, sizeof(Vehicle_t))
    (**pVeh).m_pVehicleInfo =
        (addr_of_mut!(g_vehicleInfo) as *mut vehicleInfo_t).add(BG_VehicleGetIndex(strType) as usize);
}
