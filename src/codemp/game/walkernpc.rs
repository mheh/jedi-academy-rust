//! Port of `WalkerNPC.c` — the AT-ST "walker" NPC vehicle: its per-type vehicle
//! function table and the static bodies it points at. This MP codebase always has
//! `_JK2MP` and (server-side) `QAGAME` defined, so the `#ifdef _JK2MP` / `#ifdef QAGAME`
//! branches are the ones that survive; the cgame `#ifndef QAGAME` attachment and the
//! commented-out SP bodies are excluded.
//!
//! Landed bodies (all No-oracle — they take/return pointers or set fn-ptr fields, so
//! there is nothing bit-exact to compare against a C return value): `RegisterAssets`,
//! `Board`, `ProcessMoveCommands`, `WalkerYawAdjust`, `ProcessOrientCommands`,
//! `AnimateVehicle`, and the setter `G_SetWalkerVehicleFunctions`. `AttachRiders` is
//! filled with the already-ported `AttachRidersGeneric`. `G_CreateWalkerNPC` — the
//! vehicle-object constructor — allocates via `G_AllocateVehicleObject`, zeroes the
//! `Vehicle_t`, and points `m_pVehicleInfo` at `g_vehicleInfo[BG_VehicleGetIndex(type)]`.

#![allow(non_snake_case)] // C function names (`G_SetWalkerVehicleFunctions`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro/enum names kept verbatim

use core::ffi::{c_char, c_int};
use core::ptr::{addr_of_mut, write_bytes};

use crate::codemp::game::anims::{
    animNumber_t, BOTH_RUN1, BOTH_STAND1, BOTH_STAND2, BOTH_WALK1, BOTH_WALKBACK1,
};
use crate::codemp::game::bg_misc::BG_FindItemForWeapon;
use crate::codemp::game::bg_pmove::{pm, PM_BGEntForNum};
use crate::codemp::game::bg_public::{
    bgEntity_t, ET_NPC, SETANIM_FLAG_HOLD, SETANIM_FLAG_NORMAL, SETANIM_FLAG_OVERRIDE,
    SETANIM_FLAG_RESTART, SETANIM_LEGS, STAT_HEALTH,
};
use crate::codemp::game::bg_vehicleLoad::{g_vehicleInfo, BG_VehicleGetIndex};
use crate::codemp::game::bg_vehicles_h::{vehicleInfo_t, Vehicle_t, VEHICLE_BASE};
use crate::codemp::game::bg_weapons_h::WP_TURRET;
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::level;
use crate::codemp::game::g_utils::G_AllocateVehicleObject;
use crate::codemp::game::g_vehicles::Vehicle_SetAnim;
use crate::codemp::game::q_math::{AngleNormalize180, AngleSubtract, VectorClear, VectorLength};
use crate::codemp::game::q_shared_h::{
    playerState_t, qboolean, BUTTON_WALKING, ENTITYNUM_NONE, MAX_CLIENTS, PITCH, QFALSE, QTRUE, YAW,
};

// QAGAME-only static bodies (the server build always has QAGAME defined).

pub unsafe extern "C" fn RegisterAssets(pVeh: *mut Vehicle_t) {
    //atst uses turret weapon
    RegisterItem(BG_FindItemForWeapon(WP_TURRET));

    //call the standard RegisterAssets now
    (g_vehicleInfo[VEHICLE_BASE as usize].RegisterAssets.unwrap())(pVeh);
}

// Like a think or move command, this updates various vehicle properties.
/*
static bool Update( Vehicle_t *pVeh, const usercmd_t *pUcmd )
{
    return g_vehicleInfo[VEHICLE_BASE].Update( pVeh, pUcmd );
}
*/

// Board this Vehicle (get on). The first entity to board an empty vehicle becomes the Pilot.

pub unsafe extern "C" fn Board(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    if (g_vehicleInfo[VEHICLE_BASE as usize].Board.unwrap())(pVeh, pEnt) == QFALSE {
        return QFALSE;
    }

    // Set the board wait time (they won't be able to do anything, including getting off, for this amount of time).
    (*pVeh).m_iBoarding = level.time + 1500;

    QTRUE
}

//MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
//If you really need to violate this rule for SP, then use ifdefs.
//By BG-compatible, I mean no use of game-specific data - ONLY use
//stuff available in the MP bgEntity (in SP, the bgEntity is #defined
//as a gentity, but the MP-compatible access restrictions are based
//on the bgEntity structure in the MP codebase) -rww
// ProcessMoveCommands the Vehicle.

pub unsafe extern "C" fn ProcessMoveCommands(pVeh: *mut Vehicle_t) {
    /************************************************************************************/
    /*	BEGIN	Here is where we move the vehicle (forward or back or whatever). BEGIN	*/
    /************************************************************************************/

    //Client sets ucmds and such for speed alterations
    let speedInc: f32;
    let speedIdleDec: f32;
    let speedIdle: f32;
    let speedIdleAccel: f32;
    let speedMin: f32;
    let mut speedMax: f32;
    let fWalkSpeedMax: f32;
    let parent: *mut bgEntity_t = (*pVeh).m_pParentEntity;
    let parentPS: *mut playerState_t = (*parent).playerState;

    speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;
    speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;

    speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle;
    speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
    let _ = speedIdleAccel;
    speedMin = (*(*pVeh).m_pVehicleInfo).speedMin;

    if (*parentPS).m_iVehicleNum == 0 {
        //drifts to a stop
        speedInc = speedIdle * (*pVeh).m_fTimeModifier;
        VectorClear(&mut (*parentPS).moveDir);
        //m_ucmd.forwardmove = 127;
        (*parentPS).speed = 0.0;
    } else {
        speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
    }

    if (*parentPS).speed != 0.0
        || (*parentPS).groundEntityNum == ENTITYNUM_NONE
        || (*pVeh).m_ucmd.forwardmove != 0
        || (*pVeh).m_ucmd.upmove > 0
    {
        if (*pVeh).m_ucmd.forwardmove > 0 && speedInc != 0.0 {
            (*parentPS).speed += speedInc;
        } else if (*pVeh).m_ucmd.forwardmove < 0 {
            if (*parentPS).speed > speedIdle {
                (*parentPS).speed -= speedInc;
            } else if (*parentPS).speed > speedMin {
                (*parentPS).speed -= speedIdleDec;
            }
        }
        // No input, so coast to stop.
        else if (*parentPS).speed > 0.0f32 {
            (*parentPS).speed -= speedIdleDec;
            if (*parentPS).speed < 0.0f32 {
                (*parentPS).speed = 0.0f32;
            }
        } else if (*parentPS).speed < 0.0f32 {
            (*parentPS).speed += speedIdleDec;
            if (*parentPS).speed > 0.0f32 {
                (*parentPS).speed = 0.0f32;
            }
        }
    } else {
        if (*pVeh).m_ucmd.forwardmove < 0 {
            (*pVeh).m_ucmd.forwardmove = 0;
        }
        if (*pVeh).m_ucmd.upmove < 0 {
            (*pVeh).m_ucmd.upmove = 0;
        }

        (*pVeh).m_ucmd.rightmove = 0;

        /*if ( !pVeh->m_pVehicleInfo->strafePerc
            || (!g_speederControlScheme->value && !parent->s.number) )
        {//if in a strafe-capable vehicle, clear strafing unless using alternate control scheme
            pVeh->m_ucmd.rightmove = 0;
        }*/
    }

    if !parentPS.is_null() && (*parentPS).electrifyTime > (*pm).cmd.serverTime {
        speedMax *= 0.5f32;
    }

    fWalkSpeedMax = speedMax * 0.275f32;
    if (*pVeh).m_ucmd.buttons & BUTTON_WALKING != 0 && (*parentPS).speed > fWalkSpeedMax {
        (*parentPS).speed = fWalkSpeedMax;
    } else if (*parentPS).speed > speedMax {
        (*parentPS).speed = speedMax;
    } else if (*parentPS).speed < speedMin {
        (*parentPS).speed = speedMin;
    }

    if (*parentPS).stats[STAT_HEALTH as usize] <= 0 {
        //don't keep moving while you're dying!
        (*parentPS).speed = 0.0;
    }

    /********************************************************************************/
    /*	END Here is where we move the vehicle (forward or back or whatever). END	*/
    /********************************************************************************/
}

pub unsafe extern "C" fn WalkerYawAdjust(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
) {
    let mut angDif: f32 =
        AngleSubtract(*(*pVeh).m_vOrientation.add(YAW), (*riderPS).viewangles[YAW]);

    if !parentPS.is_null() && (*parentPS).speed != 0.0 {
        let mut s: f32 = (*parentPS).speed;
        let maxDif: f32 = (*(*pVeh).m_pVehicleInfo).turningSpeed * 1.5f32; //magic number hackery

        if s < 0.0f32 {
            s = -s;
        }
        angDif *= s / (*(*pVeh).m_pVehicleInfo).speedMax;
        if angDif > maxDif {
            angDif = maxDif;
        } else if angDif < -maxDif {
            angDif = -maxDif;
        }
        *(*pVeh).m_vOrientation.add(YAW) = AngleNormalize180(
            *(*pVeh).m_vOrientation.add(YAW) - angDif * ((*pVeh).m_fTimeModifier * 0.2f32),
        );
    }
}

/*
void WalkerPitchAdjust(Vehicle_t *pVeh, playerState_t *riderPS, playerState_t *parentPS)
{
    float angDif = AngleSubtract(pVeh->m_vOrientation[PITCH], riderPS->viewangles[PITCH]);

    if (parentPS && parentPS->speed)
    {
        float s = parentPS->speed;
        float maxDif = pVeh->m_pVehicleInfo->turningSpeed*0.8f; //magic number hackery

        if (s < 0.0f)
        {
            s = -s;
        }
        angDif *= s/pVeh->m_pVehicleInfo->speedMax;
        if (angDif > maxDif)
        {
            angDif = maxDif;
        }
        else if (angDif < -maxDif)
        {
            angDif = -maxDif;
        }
        pVeh->m_vOrientation[PITCH] = AngleNormalize360(pVeh->m_vOrientation[PITCH] - angDif*(pVeh->m_fTimeModifier*0.2f));
    }
}
*/

//MP RULE - ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
//If you really need to violate this rule for SP, then use ifdefs.
//By BG-compatible, I mean no use of game-specific data - ONLY use
//stuff available in the MP bgEntity (in SP, the bgEntity is #defined
//as a gentity, but the MP-compatible access restrictions are based
//on the bgEntity structure in the MP codebase) -rww
// ProcessOrientCommands the Vehicle.

pub unsafe extern "C" fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    /********************************************************************************/
    /*	BEGIN	Here is where make sure the vehicle is properly oriented.	BEGIN	*/
    /********************************************************************************/
    let speed: f32;
    let parent: *mut bgEntity_t = (*pVeh).m_pParentEntity;
    let parentPS: *mut playerState_t;
    let riderPS: *mut playerState_t;

    let mut rider: *mut bgEntity_t = core::ptr::null_mut();
    if (*parent).s.owner != ENTITYNUM_NONE {
        rider = PM_BGEntForNum((*parent).s.owner); //&g_entities[parent->r.ownerNum];
    }

    if rider.is_null() {
        rider = parent;
    }

    parentPS = (*parent).playerState;
    riderPS = (*rider).playerState;

    speed = VectorLength(&(*parentPS).velocity);
    let _ = speed;

    // If the player is the rider...
    if (*rider).s.number < MAX_CLIENTS as c_int {
        //FIXME: use the vehicle's turning stat in this calc
        WalkerYawAdjust(pVeh, riderPS, parentPS);
        //FighterPitchAdjust(pVeh, riderPS, parentPS);
        *(*pVeh).m_vOrientation.add(PITCH) = (*riderPS).viewangles[PITCH];
    } else {
        let mut turnSpeed: f32 = (*(*pVeh).m_pVehicleInfo).turningSpeed;
        if (*(*pVeh).m_pVehicleInfo).turnWhenStopped == QFALSE && (*parentPS).speed == 0.0
        //FIXME: or !pVeh->m_ucmd.forwardmove?
        {
            //can't turn when not moving
            //FIXME: or ramp up to max turnSpeed?
            turnSpeed = 0.0f32;
        }
        if (*rider).s.eType == ET_NPC {
            //help NPCs out some
            turnSpeed *= 2.0f32;
            if (*parentPS).speed > 200.0f32 {
                turnSpeed += turnSpeed * (*parentPS).speed / 200.0f32 * 0.05f32;
            }
        }
        turnSpeed *= (*pVeh).m_fTimeModifier;

        //default control scheme: strafing turns, mouselook aims
        if (*pVeh).m_ucmd.rightmove < 0 {
            *(*pVeh).m_vOrientation.add(YAW) += turnSpeed;
        } else if (*pVeh).m_ucmd.rightmove > 0 {
            *(*pVeh).m_vOrientation.add(YAW) -= turnSpeed;
        }

        if (*(*pVeh).m_pVehicleInfo).malfunctionArmorLevel != 0
            && (*pVeh).m_iArmor <= (*(*pVeh).m_pVehicleInfo).malfunctionArmorLevel
        {
            //damaged badly
        }
    }

    /********************************************************************************/
    /*	END	Here is where make sure the vehicle is properly oriented.	END			*/
    /********************************************************************************/
}

// This function makes sure that the vehicle is properly animated.
#[allow(unused_assignments)] // faithful redundant inits: `Anim`/`iFlags`/`iBlend`

pub unsafe extern "C" fn AnimateVehicle(pVeh: *mut Vehicle_t) {
    let mut Anim: animNumber_t = BOTH_STAND1;
    let mut iFlags: i32 = SETANIM_FLAG_NORMAL;
    let mut iBlend: i32 = 300;
    let parent: *mut gentity_t = (*pVeh).m_pParentEntity as *mut gentity_t;
    let fSpeedPercToMax: f32;

    // We're dead (boarding is reused here so I don't have to make another variable :-).
    if (*parent).health <= 0 {
        /*
        if ( pVeh->m_iBoarding != -999 )	// Animate the death just once!
        {
            pVeh->m_iBoarding = -999;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;

            // FIXME! Why do you keep repeating over and over!!?!?!? Bastard!
            //Vehicle_SetAnim( parent, SETANIM_LEGS, BOTH_VT_DEATH1, iFlags, iBlend );
        }
        */
        return;
    }

    // Following is redundant to g_vehicles.c
    //	if ( pVeh->m_iBoarding )
    //	{
    //		//we have no boarding anim
    //		if (pVeh->m_iBoarding < level.time)
    //		{ //we are on now
    //			pVeh->m_iBoarding = 0;
    //		}
    //		else
    //		{
    //			return;
    //		}
    //	}

    // Percentage of maximum speed relative to current speed.
    //float fSpeed = VectorLength( client->ps.velocity );
    fSpeedPercToMax = (*(*parent).client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax;

    // If we're moving...
    if fSpeedPercToMax > 0.0f32
    //fSpeedPercToMax >= 0.85f )
    {
        let _fYawDelta: f32;

        iBlend = 300;
        iFlags = SETANIM_FLAG_OVERRIDE;
        _fYawDelta = (*pVeh).m_vPrevOrientation[YAW] - *(*pVeh).m_vOrientation.add(YAW);

        // NOTE: Mikes suggestion for fixing the stuttering walk (left/right) is to maintain the
        // current frame between animations. I have no clue how to do this and have to work on other
        // stuff so good luck to him :-p AReis

        // If we're walking (or our speed is less than .275%)...
        if ((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 || fSpeedPercToMax < 0.275f32 {
            // Make them lean if we're turning.
            /*if ( fYawDelta < -0.0001f )
            {
                Anim = BOTH_VT_WALK_FWD_L;
            }
            else if ( fYawDelta > 0.0001 )
            {
                Anim = BOTH_VT_WALK_FWD_R;
            }
            else*/
            {
                Anim = BOTH_WALK1;
            }
        }
        // otherwise we're running.
        else {
            // Make them lean if we're turning.
            /*if ( fYawDelta < -0.0001f )
            {
                Anim = BOTH_VT_RUN_FWD_L;
            }
            else if ( fYawDelta > 0.0001 )
            {
                Anim = BOTH_VT_RUN_FWD_R;
            }
            else*/
            {
                Anim = BOTH_RUN1;
            }
        }
    } else {
        // Going in reverse...
        if fSpeedPercToMax < -0.018f32 {
            iFlags = SETANIM_FLAG_NORMAL;
            Anim = BOTH_WALKBACK1;
            iBlend = 500;
        } else {
            //int iChance = Q_irand( 0, 20000 );

            // Every once in a while buck or do a different idle...
            iFlags = SETANIM_FLAG_NORMAL | SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD;
            iBlend = 600;
            if (*(*parent).client).ps.m_iVehicleNum != 0 {
                //occupado
                Anim = BOTH_STAND1;
            } else {
                //wide open for you, baby
                Anim = BOTH_STAND2;
            }
        }
    }

    Vehicle_SetAnim(parent, SETANIM_LEGS, Anim, iFlags, iBlend);
}

//rwwFIXMEFIXME: This is all going to have to be predicted I think, or it will feel awful
//and lagged

//on the client this function will only set up the process command funcs

pub unsafe extern "C" fn G_SetWalkerVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    (*pVehInfo).AnimateVehicle = Some(AnimateVehicle);
    //	pVehInfo->AnimateRiders				=		AnimateRiders;
    //	pVehInfo->ValidateBoard				=		ValidateBoard;
    //	pVehInfo->SetParent					=		SetParent;
    //	pVehInfo->SetPilot					=		SetPilot;
    //	pVehInfo->AddPassenger				=		AddPassenger;
    //	pVehInfo->Animate					=		Animate;
    (*pVehInfo).Board = Some(Board);
    //	pVehInfo->Eject						=		Eject;
    //	pVehInfo->EjectAll					=		EjectAll;
    //	pVehInfo->StartDeathDelay			=		StartDeathDelay;
    //	pVehInfo->DeathUpdate				=		DeathUpdate;
    (*pVehInfo).RegisterAssets = Some(RegisterAssets);
    //	pVehInfo->Initialize				=		Initialize;
    //	pVehInfo->Update					=		Update;
    //	pVehInfo->UpdateRider				=		UpdateRider;
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
//this is a BG function too in MP so don't un-bg-compatibilify it -rww
//
// No-oracle: allocates from the module-static vehicle pool via
// `G_AllocateVehicleObject` and zeroes/initialises `Vehicle_t` through pointers.
pub unsafe extern "C" fn G_CreateWalkerNPC(
    pVeh: *mut *mut Vehicle_t,
    strAnimalType: *const c_char,
) {
    // Allocate the Vehicle.
    // #ifdef _JK2MP / #ifdef QAGAME (server build):
    //these will remain on entities on the client once allocated because the pointer is
    //never stomped. on the server, however, when an ent is freed, the entity struct is
    //memset to 0, so this memory would be lost..
    G_AllocateVehicleObject(pVeh);
    write_bytes(*pVeh, 0, 1); // memset(*pVeh, 0, sizeof(Vehicle_t))
    (**pVeh).m_pVehicleInfo = (addr_of_mut!(g_vehicleInfo) as *mut vehicleInfo_t)
        .add(BG_VehicleGetIndex(strAnimalType) as usize);
}
