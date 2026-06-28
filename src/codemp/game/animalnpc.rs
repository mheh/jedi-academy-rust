//! Port of `AnimalNPC.c` — the animal-vehicle (Tauntaun) behaviour set: the per-type
//! vtable bodies that `G_SetAnimalVehicleFunctions` installs onto a `vehicleInfo_t`
//! (death sequence, the base-vehicle `Update` delegate, move/orient command processing,
//! and the legs/rider animation drivers).
//!
//! Landed: the full `#ifdef QAGAME` (`_JK2MP`) vtable — `DeathUpdate`, `Update`,
//! `ProcessOrientCommands`, `AnimalProcessOri` (the MP-only `ProcessOrientCommands`
//! wrapper), `AnimateVehicle`, `ProcessMoveCommands`, `AnimateRiders` — and the
//! installer `G_SetAnimalVehicleFunctions`. In the MP build the SP-only
//! `G_SoundIndexOnEnt`, Ghoul2 (`G2API_GetBoneAnimIndex`), `G_RemoveWeaponModels`, and
//! `CG_ChangeWeapon` calls are all `#ifndef _JK2MP`/`#else` and so excluded, leaving the
//! bodies portable. All are ptr-in entity-state mutators (No-oracle).
//!
//! `G_CreateAnimalNPC` — the vehicle-object constructor — is landed: its QAGAME path
//! allocates via `G_AllocateVehicleObject`, zeroes the `Vehicle_t`, and points
//! `m_pVehicleInfo` at `g_vehicleInfo[BG_VehicleGetIndex(type)]`.

#![allow(non_snake_case)] // C function names (`DeathUpdate`, `AnimateVehicle`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro/enum names kept verbatim

use core::ffi::{c_char, c_int};
use core::ptr::{addr_of, addr_of_mut, write_bytes};

use crate::codemp::game::anims::{
    BOTH_VT_ATF_G, BOTH_VT_ATL_G, BOTH_VT_ATL_S, BOTH_VT_ATL_TO_R_S, BOTH_VT_ATR_G, BOTH_VT_ATR_S,
    BOTH_VT_ATR_TO_L_S, BOTH_VT_BUCK, BOTH_VT_IDLE, BOTH_VT_IDLE1, BOTH_VT_IDLE_G, BOTH_VT_IDLE_SL,
    BOTH_VT_IDLE_SR, BOTH_VT_MOUNT_B, BOTH_VT_MOUNT_L, BOTH_VT_MOUNT_R, BOTH_VT_RUN_FWD,
    BOTH_VT_TURBO, BOTH_VT_WALK_FWD, BOTH_VT_WALK_REV,
};
use crate::codemp::game::bg_panimate::BG_AnimLength;
use crate::codemp::game::bg_pmove::PM_BGEntForNum;
use crate::codemp::game::bg_public::{
    bgEntity_t, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_HOLDLESS, SETANIM_FLAG_NORMAL,
    SETANIM_FLAG_OVERRIDE, SETANIM_FLAG_RESTART, SETANIM_LEGS,
};
use crate::codemp::game::bg_vehicleLoad::{g_vehicleInfo, BG_VehicleGetIndex};
use crate::codemp::game::bg_vehicles_h::{
    vehicleInfo_t, EWeaponPose, Vehicle_t, VEHICLE_BASE, VEH_BUCKING, VEH_CRASHING,
    VEH_SABERINLEFTHAND, WPOSE_BLASTER, WPOSE_NONE, WPOSE_SABERLEFT, WPOSE_SABERRIGHT,
};
use crate::codemp::game::bg_weapons_h::{WP_BLASTER, WP_MELEE, WP_NONE, WP_SABER};
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::level;
use crate::codemp::game::g_utils::G_AllocateVehicleObject;
use crate::codemp::game::g_vehicles::Vehicle_SetAnim;
use crate::codemp::game::q_math::{AngleNormalize180, AngleSubtract, VectorClear};
use crate::codemp::game::q_shared_h::{
    playerState_t, qboolean, usercmd_t, BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_WALKING,
    ENTITYNUM_NONE, QFALSE, YAW,
};

// Update death sequence.
pub unsafe extern "C" fn DeathUpdate(pVeh: *mut Vehicle_t) {
    if level.time >= (*pVeh).m_iDieTime {
        // If the vehicle is not empty.
        if ((*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) != QFALSE {
            ((*(*pVeh).m_pVehicleInfo).EjectAll.unwrap())(pVeh);
        } else {
            // Waste this sucker.
        }

        // Die now...
        /*		else
        {
            vec3_t	mins, maxs, bottom;
            trace_t	trace;

            if ( pVeh->m_pVehicleInfo->explodeFX )
            {
                G_PlayEffect( pVeh->m_pVehicleInfo->explodeFX, parent->currentOrigin );
                //trace down and place mark
                VectorCopy( parent->currentOrigin, bottom );
                bottom[2] -= 80;
                gi.trace( &trace, parent->currentOrigin, vec3_origin, vec3_origin, bottom, parent->s.number, CONTENTS_SOLID );
                if ( trace.fraction < 1.0f )
                {
                    VectorCopy( trace.endpos, bottom );
                    bottom[2] += 2;
                    G_PlayEffect( "ships/ship_explosion_mark", trace.endpos );
                }
            }

            parent->takedamage = qfalse;//so we don't recursively damage ourselves
            if ( pVeh->m_pVehicleInfo->explosionRadius > 0 && pVeh->m_pVehicleInfo->explosionDamage > 0 )
            {
                VectorCopy( parent->mins, mins );
                mins[2] = -4;//to keep it off the ground a *little*
                VectorCopy( parent->maxs, maxs );
                VectorCopy( parent->currentOrigin, bottom );
                bottom[2] += parent->mins[2] - 32;
                gi.trace( &trace, parent->currentOrigin, mins, maxs, bottom, parent->s.number, CONTENTS_SOLID );
                G_RadiusDamage( trace.endpos, NULL, pVeh->m_pVehicleInfo->explosionDamage, pVeh->m_pVehicleInfo->explosionRadius, NULL, MOD_EXPLOSIVE );//FIXME: extern damage and radius or base on fuel
            }

            parent->e_ThinkFunc = thinkF_G_FreeEntity;
            parent->nextthink = level.time + FRAMETIME;
        }*/
    }
}

// Like a think or move command, this updates various vehicle properties.
pub unsafe extern "C" fn Update(pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    ((*addr_of!(g_vehicleInfo))[VEHICLE_BASE as usize]
        .Update
        .unwrap())(pVeh, pUcmd)
}

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
    let parent: *mut bgEntity_t = (*pVeh).m_pParentEntity;
    let parentPS: *mut playerState_t;
    let riderPS: *mut playerState_t;

    let mut rider: *mut bgEntity_t = core::ptr::null_mut();
    if (*parent).s.owner != ENTITYNUM_NONE {
        rider = PM_BGEntForNum((*parent).s.owner); //&g_entities[parent->r.ownerNum];
    }

    // Bucking so we can't do anything.
    // #ifndef _JK2MP //bad for prediction - fixme  -- MP path: block excluded.

    if rider.is_null() {
        rider = parent;
    }

    parentPS = (*parent).playerState;
    riderPS = (*rider).playerState;

    if !rider.is_null() {
        let mut angDif =
            AngleSubtract(*(*pVeh).m_vOrientation.add(YAW), (*riderPS).viewangles[YAW]);
        if !parentPS.is_null() && (*parentPS).speed != 0.0 {
            let mut s = (*parentPS).speed;
            let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 4.0; //magic number hackery
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

    /*	speed = VectorLength( parentPS->velocity );

        // If the player is the rider...
        if ( rider->s.number < MAX_CLIENTS )
        {//FIXME: use the vehicle's turning stat in this calc
            pVeh->m_vOrientation[YAW] = riderPS->viewangles[YAW];
        }
        else
        {
            float turnSpeed = pVeh->m_pVehicleInfo->turningSpeed;
            if ( !pVeh->m_pVehicleInfo->turnWhenStopped
                && !parentPS->speed )//FIXME: or !pVeh->m_ucmd.forwardmove?
            {//can't turn when not moving
                //FIXME: or ramp up to max turnSpeed?
                turnSpeed = 0.0f;
            }
    #ifdef _JK2MP
            if (rider->s.eType == ET_NPC)
    #else
            if ( !rider || rider->NPC )
    #endif
            {//help NPCs out some
                turnSpeed *= 2.0f;
    #ifdef _JK2MP
                if (parentPS->speed > 200.0f)
    #else
                if ( parent->client->ps.speed > 200.0f )
    #endif
                {
                    turnSpeed += turnSpeed * parentPS->speed/200.0f*0.05f;
                }
            }
            turnSpeed *= pVeh->m_fTimeModifier;

            //default control scheme: strafing turns, mouselook aims
            if ( pVeh->m_ucmd.rightmove < 0 )
            {
                pVeh->m_vOrientation[YAW] += turnSpeed;
            }
            else if ( pVeh->m_ucmd.rightmove > 0 )
            {
                pVeh->m_vOrientation[YAW] -= turnSpeed;
            }

            if ( pVeh->m_pVehicleInfo->malfunctionArmorLevel && pVeh->m_iArmor <= pVeh->m_pVehicleInfo->malfunctionArmorLevel )
            {//damaged badly
            }
        }*/

    /********************************************************************************/
    /*	END	Here is where make sure the vehicle is properly oriented.	END			*/
    /********************************************************************************/
}

//temp hack til mp speeder controls are sorted -rww
pub unsafe extern "C" fn AnimalProcessOri(pVeh: *mut Vehicle_t) {
    ProcessOrientCommands(pVeh);
}

#[allow(unused_assignments)] // faithful: the VEH_BUCKING branch's `Anim = BOTH_VT_BUCK` dead store is preserved (C passes the literal to Vehicle_SetAnim, then returns)
pub unsafe extern "C" fn AnimateVehicle(pVeh: *mut Vehicle_t) {
    let mut Anim: c_int = BOTH_VT_IDLE;
    let mut iFlags: c_int = SETANIM_FLAG_NORMAL;
    let mut iBlend: c_int = 300;
    let pilot: *mut gentity_t = (*pVeh).m_pPilot as *mut gentity_t;
    let parent: *mut gentity_t = (*pVeh).m_pParentEntity as *mut gentity_t;
    let pilotPS: *mut playerState_t;
    let parentPS: *mut playerState_t;
    let fSpeedPercToMax: f32;

    pilotPS = if !pilot.is_null() {
        (*pilot).playerState
    } else {
        core::ptr::null_mut()
    };
    parentPS = (*parent).playerState;
    let _ = (pilotPS, parentPS); // pilotPS/parentPS mirror the C locals (unused on the MP path)

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

    // If they're bucking, play the animation and leave...
    if (*(*parent).client).ps.legsAnim == BOTH_VT_BUCK {
        // Done with animation? Erase the flag.
        if (*(*parent).client).ps.legsTimer <= 0 {
            (*pVeh).m_ulFlags &= !(VEH_BUCKING as core::ffi::c_ulong);
        } else {
            return;
        }
    } else if (*pVeh).m_ulFlags & (VEH_BUCKING as core::ffi::c_ulong) != 0 {
        iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
        Anim = BOTH_VT_BUCK;
        iBlend = 500;
        Vehicle_SetAnim(parent, SETANIM_LEGS, BOTH_VT_BUCK, iFlags, iBlend);
        return;
    }

    // Boarding animation.
    if (*pVeh).m_iBoarding != 0 {
        // We've just started boarding, set the amount of time it will take to finish boarding.
        if (*pVeh).m_iBoarding < 0 {
            let iAnimLen: c_int;

            // Boarding from left...
            if (*pVeh).m_iBoarding == -1 {
                Anim = BOTH_VT_MOUNT_L;
            } else if (*pVeh).m_iBoarding == -2 {
                Anim = BOTH_VT_MOUNT_R;
            } else if (*pVeh).m_iBoarding == -3 {
                Anim = BOTH_VT_MOUNT_B;
            }

            // Set the delay time (which happens to be the time it takes for the animation to complete).
            // NOTE: Here I made it so the delay is actually 70% (0.7f) of the animation time.
            iAnimLen = (BG_AnimLength((*parent).localAnimIndex, Anim) as f32 * 0.7) as c_int;
            (*pVeh).m_iBoarding = level.time + iAnimLen;

            // Set the animation, which won't be interrupted until it's completed.
            // TODO: But what if he's killed? Should the animation remain persistant???
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;

            Vehicle_SetAnim(parent, SETANIM_LEGS, Anim, iFlags, iBlend);
            if !pilot.is_null() {
                Vehicle_SetAnim(pilot, SETANIM_BOTH, Anim, iFlags, iBlend);
            }
            return;
        }
        // Otherwise we're done.
        else if (*pVeh).m_iBoarding <= level.time {
            (*pVeh).m_iBoarding = 0;
        }
    }

    // Percentage of maximum speed relative to current speed.
    //float fSpeed = VectorLength( client->ps.velocity );
    fSpeedPercToMax = (*(*parent).client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax;

    // Going in reverse...
    if fSpeedPercToMax < -0.01 {
        Anim = BOTH_VT_WALK_REV;
        iBlend = 600;
    } else {
        let Turbo: bool = fSpeedPercToMax > 0.0 && level.time < (*pVeh).m_iTurboTime;
        let Walking: bool = fSpeedPercToMax > 0.0
            && (((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 || fSpeedPercToMax <= 0.275);
        let Running: bool = fSpeedPercToMax > 0.275;

        // Remove Crashing Flag
        //----------------------
        (*pVeh).m_ulFlags &= !(VEH_CRASHING as core::ffi::c_ulong);

        if Turbo {
            // Kicked In Turbo
            iBlend = 50;
            iFlags = SETANIM_FLAG_OVERRIDE;
            Anim = BOTH_VT_TURBO;
        } else {
            // No Special Moves
            iBlend = 300;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLDLESS;
            Anim = if Walking {
                BOTH_VT_WALK_FWD
            } else if Running {
                BOTH_VT_RUN_FWD
            } else {
                BOTH_VT_IDLE1
            };
        }
    }
    Vehicle_SetAnim(parent, SETANIM_LEGS, Anim, iFlags, iBlend);
}

//MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
//If you really need to violate this rule for SP, then use ifdefs.
//By BG-compatible, I mean no use of game-specific data - ONLY use
//stuff available in the MP bgEntity (in SP, the bgEntity is #defined
//as a gentity, but the MP-compatible access restrictions are based
//on the bgEntity structure in the MP codebase) -rww
// ProcessMoveCommands the Vehicle.
#[allow(unused_assignments)] // faithful: the initial `speedMax = ...->speedMax` dead store is preserved (the curTime/m_iTurboTime if/else overwrites it before any read)
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
    let curTime: c_int;
    let parent: *mut bgEntity_t = (*pVeh).m_pParentEntity;
    let parentPS: *mut playerState_t = (*parent).playerState;

    //#elif QAGAME//MP GAME
    curTime = level.time;

    // #ifndef _JK2MP//SP bucking-stop early-out excluded (bad for prediction - fixme)

    speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;
    speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;

    speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle;
    speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
    speedMin = (*(*pVeh).m_pVehicleInfo).speedMin;
    let _ = speedIdleAccel; // faithful: C computes speedIdleAccel here but never reads it

    if !(*pVeh).m_pPilot.is_null() /*&& (pilotPS->weapon == WP_NONE || pilotPS->weapon == WP_MELEE )*/
        && ((*pVeh).m_ucmd.buttons & BUTTON_ALT_ATTACK) != 0
        && (*(*pVeh).m_pVehicleInfo).turboSpeed != 0.0
    {
        if (curTime - (*pVeh).m_iTurboTime) > (*(*pVeh).m_pVehicleInfo).turboRecharge {
            (*pVeh).m_iTurboTime = curTime + (*(*pVeh).m_pVehicleInfo).turboDuration;
            // #ifndef _JK2MP //kill me now -- the soundTurbo/G_SoundIndexOnEnt block is SP-only, excluded.
            (*parentPS).speed = (*(*pVeh).m_pVehicleInfo).turboSpeed; // Instantly Jump To Turbo Speed
        }
    }

    if curTime < (*pVeh).m_iTurboTime {
        speedMax = (*(*pVeh).m_pVehicleInfo).turboSpeed;
    } else {
        speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;
    }

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
        else if (*parentPS).speed > 0.0 {
            (*parentPS).speed -= speedIdleDec;
            if (*parentPS).speed < 0.0 {
                (*parentPS).speed = 0.0;
            }
        } else if (*parentPS).speed < 0.0 {
            (*parentPS).speed += speedIdleDec;
            if (*parentPS).speed > 0.0 {
                (*parentPS).speed = 0.0;
            }
        }
    } else {
        if (*pVeh).m_ucmd.forwardmove < 0 {
            (*pVeh).m_ucmd.forwardmove = 0;
        }
        if (*pVeh).m_ucmd.upmove < 0 {
            (*pVeh).m_ucmd.upmove = 0;
        }

        //pVeh->m_ucmd.rightmove = 0;

        /*if ( !pVeh->m_pVehicleInfo->strafePerc
            || (!g_speederControlScheme->value && !parent->s.number) )
        {//if in a strafe-capable vehicle, clear strafing unless using alternate control scheme
            pVeh->m_ucmd.rightmove = 0;
        }*/
    }

    fWalkSpeedMax = speedMax * 0.275;
    if curTime > (*pVeh).m_iTurboTime
        && ((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0
        && (*parentPS).speed > fWalkSpeedMax
    {
        (*parentPS).speed = fWalkSpeedMax;
    } else if (*parentPS).speed > speedMax {
        (*parentPS).speed = speedMax;
    } else if (*parentPS).speed < speedMin {
        (*parentPS).speed = speedMin;
    }

    /********************************************************************************/
    /*	END Here is where we move the vehicle (forward or back or whatever). END	*/
    /********************************************************************************/
}

//rwwFIXMEFIXME: This is all going to have to be predicted I think, or it will feel awful
//and lagged
// This function makes sure that the rider's in this vehicle are properly animated.
#[allow(unused_assignments)] // faithful: the MP `if (0)` reverse branch's Anim/iBlend stores are dead (the C #ifdef _JK2MP //handled in pmove in mp)
pub unsafe extern "C" fn AnimateRiders(pVeh: *mut Vehicle_t) {
    let mut Anim: c_int = BOTH_VT_IDLE;
    let mut iFlags: c_int = SETANIM_FLAG_NORMAL;
    let mut iBlend: c_int = 500;
    let pilot: *mut gentity_t = (*pVeh).m_pPilot as *mut gentity_t;
    let parent: *mut gentity_t = (*pVeh).m_pParentEntity as *mut gentity_t;
    let pilotPS: *mut playerState_t;
    let parentPS: *mut playerState_t;
    let fSpeedPercToMax: f32;

    pilotPS = (*(*pVeh).m_pPilot).playerState;
    parentPS = (*(*pVeh).m_pPilot).playerState;
    let _ = parentPS; // faithful: MP sets parentPS to the pilot's playerState; it is never read here

    // Boarding animation.
    if (*pVeh).m_iBoarding != 0 {
        return;
    }

    // Percentage of maximum speed relative to current speed.
    fSpeedPercToMax = (*(*parent).client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax;

    // Going in reverse...
    //#ifdef _JK2MP //handled in pmove in mp
    if false {
        Anim = BOTH_VT_WALK_REV;
        iBlend = 600;
    } else {
        let HasWeapon: bool = (*pilotPS).weapon != WP_NONE && (*pilotPS).weapon != WP_MELEE;
        let Attacking: bool = HasWeapon && ((*pVeh).m_ucmd.buttons & BUTTON_ATTACK) != 0;
        let mut Right: bool = (*pVeh).m_ucmd.rightmove > 0;
        let mut Left: bool = (*pVeh).m_ucmd.rightmove < 0;
        let Turbo: bool = fSpeedPercToMax > 0.0 && level.time < (*pVeh).m_iTurboTime;
        let Walking: bool = fSpeedPercToMax > 0.0
            && (((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 || fSpeedPercToMax <= 0.275);
        let Running: bool = fSpeedPercToMax > 0.275;
        let mut WeaponPose: EWeaponPose = WPOSE_NONE;

        // Remove Crashing Flag
        //----------------------
        (*pVeh).m_ulFlags &= !(VEH_CRASHING as core::ffi::c_ulong);

        // Put Away Saber When It Is Not Active
        //-------------------------------------- (#ifndef _JK2MP: SP CG_ChangeWeapon/G_RemoveWeaponModels block excluded)

        // Don't Interrupt Attack Anims
        //------------------------------
        if (*pilotPS).weaponTime > 0 {
            return;
        }

        // Compute The Weapon Pose
        //--------------------------
        if (*pilotPS).weapon == WP_BLASTER {
            WeaponPose = WPOSE_BLASTER;
        } else if (*pilotPS).weapon == WP_SABER {
            if ((*pVeh).m_ulFlags & (VEH_SABERINLEFTHAND as core::ffi::c_ulong)) != 0
                && (*pilotPS).torsoAnim == BOTH_VT_ATL_TO_R_S
            {
                (*pVeh).m_ulFlags &= !(VEH_SABERINLEFTHAND as core::ffi::c_ulong);
            }
            if ((*pVeh).m_ulFlags & (VEH_SABERINLEFTHAND as core::ffi::c_ulong)) == 0
                && (*pilotPS).torsoAnim == BOTH_VT_ATR_TO_L_S
            {
                (*pVeh).m_ulFlags |= VEH_SABERINLEFTHAND as core::ffi::c_ulong;
            }
            WeaponPose = if ((*pVeh).m_ulFlags & (VEH_SABERINLEFTHAND as core::ffi::c_ulong)) != 0 {
                WPOSE_SABERLEFT
            } else {
                WPOSE_SABERRIGHT
            };
        }

        if Attacking && WeaponPose != 0 {
            // Attack!
            iBlend = 100;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART;

            if Turbo {
                Right = true;
                Left = false;
            }

            // Auto Aiming
            //===============================================
            if !Left && !Right {
                // Allow player strafe keys to override
                // (#ifndef _JK2MP: SP enemy auto-aim block excluded)
                if (*pilotPS).weapon == WP_SABER && !Left && !Right {
                    Left = WeaponPose == WPOSE_SABERLEFT;
                    Right = !Left;
                }
            }

            if Left {
                // Attack Left
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VT_ATL_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VT_ATL_S,
                    WPOSE_SABERRIGHT => Anim = BOTH_VT_ATR_TO_L_S,
                    _ => debug_assert!(false),
                }
            } else if Right {
                // Attack Right
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VT_ATR_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VT_ATL_TO_R_S,
                    WPOSE_SABERRIGHT => Anim = BOTH_VT_ATR_S,
                    _ => debug_assert!(false),
                }
            } else {
                // Attack Ahead
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VT_ATF_G,
                    _ => debug_assert!(false),
                }
            }
        } else if Turbo {
            // Kicked In Turbo
            iBlend = 50;
            iFlags = SETANIM_FLAG_OVERRIDE;
            Anim = BOTH_VT_TURBO;
        } else {
            // No Special Moves
            iBlend = 300;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLDLESS;

            if WeaponPose == WPOSE_NONE {
                if Walking {
                    Anim = BOTH_VT_WALK_FWD;
                } else if Running {
                    Anim = BOTH_VT_RUN_FWD;
                } else {
                    Anim = BOTH_VT_IDLE1; //(Q_irand(0,1)==0)?(BOTH_VT_IDLE):(BOTH_VT_IDLE1);
                }
            } else {
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VT_IDLE_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VT_IDLE_SL,
                    WPOSE_SABERRIGHT => Anim = BOTH_VT_IDLE_SR,
                    _ => debug_assert!(false),
                }
            }
        } // No Special Moves
    }

    Vehicle_SetAnim(pilot, SETANIM_BOTH, Anim, iFlags, iBlend);
}

//on the client this function will only set up the process command funcs
pub unsafe extern "C" fn G_SetAnimalVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    //#ifdef QAGAME
    (*pVehInfo).AnimateVehicle = Some(AnimateVehicle);
    (*pVehInfo).AnimateRiders = Some(AnimateRiders);
    //	pVehInfo->ValidateBoard				=		ValidateBoard;
    //	pVehInfo->SetParent					=		SetParent;
    //	pVehInfo->SetPilot					=		SetPilot;
    //	pVehInfo->AddPassenger				=		AddPassenger;
    //	pVehInfo->Animate					=		Animate;
    //	pVehInfo->Board						=		Board;
    //	pVehInfo->Eject						=		Eject;
    //	pVehInfo->EjectAll					=		EjectAll;
    //	pVehInfo->StartDeathDelay			=		StartDeathDelay;
    (*pVehInfo).DeathUpdate = Some(DeathUpdate);
    //	pVehInfo->RegisterAssets			=		RegisterAssets;
    //	pVehInfo->Initialize				=		Initialize;
    (*pVehInfo).Update = Some(Update);
    //	pVehInfo->UpdateRider				=		UpdateRider;
    //#endif //QAGAME
    (*pVehInfo).ProcessMoveCommands = Some(ProcessMoveCommands);
    (*pVehInfo).ProcessOrientCommands = Some(ProcessOrientCommands);

    //#ifndef QAGAME //cgame prediction attachment func
    //	pVehInfo->AttachRiders				=		AttachRidersGeneric;
    //#endif
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
pub unsafe extern "C" fn G_CreateAnimalNPC(
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
