//! `SpeederNPC.c` — the per-type vehicle callbacks for the SPEEDER vehicle class
//! (`VH_SPEEDER`), wired into a `vehicleInfo_t` by `G_SetSpeederVehicleFunctions`.
//!
//! **Build config resolved to the retail MP game module** (`_JK2MP` defined, `QAGAME`
//! defined by the build, `CGAME`/UI undefined): the SP-only `#ifndef _JK2MP` branches
//! (the `wp_saber.h`/`g_functions.h`/`g_vehicles.h` SP includes, `g_speederControlScheme`,
//! `CG_ChangeWeapon`/`G_RemoveWeaponModels`/`G_StartMatrixEffect`, the SP `VEH_StartStrafeRam`)
//! are `#ifdef`-excluded, and the cgame-only `#ifndef QAGAME` branch of the setter
//! (`AttachRiders = AttachRidersGeneric`) is excluded because our build defines `QAGAME`.
//!
//! **Landed: the full `#ifdef QAGAME`/shared per-type vtable** — `ProcessMoveCommands`
//! (:278), `ProcessOrientCommands` (:505), the empty `AnimateVehicle` stub (:608), the
//! trivial MP `VEH_StartStrafeRam` stub (:140 `return false`), `Update` (:149),
//! `AnimateRiders` (:630), and the installer `G_SetSpeederVehicleFunctions` (:1044). In the
//! MP build the `Update` exhaust/strafe-ram block and the `G_StopEffect`/`G_SoundIndex`
//! calls live in `#ifndef _JK2MP` (SP) and are excluded; `AnimateRiders` hits the
//! `#ifdef _JK2MP if (1) return;` right after the boarding block, so the rich rider-pose
//! tail (and its SP-only `CG_ChangeWeapon`/`G_RemoveWeaponModels`/`gi.G2API_GetBoneAnimIndex`
//! sub-blocks) is dead-but-compiled — carried faithfully but unreachable. All bodies are
//! ptr-in entity-state mutators (No-oracle).
//!
//! `G_CreateSpeederNPC` (:1092) — the vehicle-object constructor — is landed: allocates
//! via `G_AllocateVehicleObject`, zeroes the `Vehicle_t`, and points `m_pVehicleInfo` at
//! `g_vehicleInfo[BG_VehicleGetIndex(type)]`.

#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

use crate::codemp::game::anims::{
    animNumber_t, BOTH_VS_AIR, BOTH_VS_AIR_G, BOTH_VS_AIR_SL, BOTH_VS_AIR_SR, BOTH_VS_ATF_G,
    BOTH_VS_ATL_G, BOTH_VS_ATL_S, BOTH_VS_ATL_TO_R_S, BOTH_VS_ATR_G, BOTH_VS_ATR_S,
    BOTH_VS_ATR_TO_L_S, BOTH_VS_IDLE, BOTH_VS_IDLE_G, BOTH_VS_IDLE_SL, BOTH_VS_IDLE_SR,
    BOTH_VS_LAND, BOTH_VS_LAND_G, BOTH_VS_LAND_SL, BOTH_VS_LAND_SR, BOTH_VS_LEANL, BOTH_VS_LEANL_G,
    BOTH_VS_LEANL_SL, BOTH_VS_LEANL_SR, BOTH_VS_LEANR, BOTH_VS_LEANR_G, BOTH_VS_LEANR_SL,
    BOTH_VS_LEANR_SR, BOTH_VS_LOOKLEFT, BOTH_VS_LOOKRIGHT, BOTH_VS_MOUNTJUMP_L, BOTH_VS_MOUNT_L,
    BOTH_VS_MOUNT_R, BOTH_VS_MOUNTTHROW_L, BOTH_VS_MOUNTTHROW_R, BOTH_VS_REV, BOTH_VS_TURBO,
};
use crate::codemp::game::bg_panimate::{bgAllAnims, BG_AnimLength, BG_SetAnim};
use crate::codemp::game::bg_pmove::{pm, BG_SabersOff};
use crate::codemp::game::bg_public::EF_JETPACK_ACTIVE;
use crate::codemp::game::bg_public::BG_GiveMeVectorFromMatrix;
use crate::codemp::game::bg_public::{
    SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_HOLDLESS, SETANIM_FLAG_NORMAL,
    SETANIM_FLAG_OVERRIDE, SETANIM_FLAG_RESTART,
};
use crate::codemp::game::bg_vehicleLoad::{g_vehicleInfo, BG_VehicleGetIndex};
use crate::codemp::game::bg_vehicles_h::{
    vehicleInfo_t, EWeaponPose, Vehicle_t, MAX_VEHICLE_EXHAUSTS, VEHICLE_BASE, VEH_CRASHING,
    VEH_FLYING, VEH_MOUNT_THROW_LEFT, VEH_MOUNT_THROW_RIGHT, VEH_SABERINLEFTHAND, VEH_SLIDEBREAKING,
    WPOSE_BLASTER, WPOSE_NONE, WPOSE_SABERLEFT, WPOSE_SABERRIGHT,
};
use crate::codemp::game::bg_weapons_h::{WP_BLASTER, WP_MELEE, WP_NONE, WP_SABER};
use crate::codemp::game::g_main::{level, BG_GetTime};
use crate::codemp::game::g_utils::{G_AllocateVehicleObject, G_PlayEffectID};
use crate::codemp::game::q_math::{AngleNormalize180, AngleSubtract};
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, playerState_t, qboolean, usercmd_t, vec3_t, BUTTON_ALT_ATTACK, BUTTON_ATTACK,
    BUTTON_USE, ENTITYNUM_NONE, MAX_CLIENTS, ORIGIN, QFALSE, QTRUE, ROLL, YAW,
};
use crate::trap;
use core::ffi::{c_char, c_int, c_ulong};
use core::ptr::{addr_of, addr_of_mut, null_mut, write_bytes};

//MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
//If you really need to violate this rule for SP, then use ifdefs.
//By BG-compatible, I mean no use of game-specific data - ONLY use
//stuff available in the MP bgEntity (in SP, the bgEntity is #defined
//as a gentity, but the MP-compatible access restrictions are based
//on the bgEntity structure in the MP codebase) -rww
// ProcessMoveCommands the Vehicle.
/// `ProcessMoveCommands` (SpeederNPC.c:278).
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
    let speedMax: f32;
    let parentPS: *mut playerState_t;
    let mut pilotPS: *mut playerState_t = null_mut();
    let curTime: i32;

    parentPS = (*(*pVeh).m_pParentEntity).playerState;
    if !(*pVeh).m_pPilot.is_null() {
        pilotPS = (*(*pVeh).m_pPilot).playerState;
    }

    // If we're flying, make us accelerate at 40% (about half) acceleration rate, and restore the pitch
    // to origin (straight) position (at 5% increments).
    if (*pVeh).m_ulFlags & VEH_FLYING as c_ulong != 0 {
        speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier * 0.4f32;
    } else if (*parentPS).m_iVehicleNum == 0 {
        //drifts to a stop
        speedInc = 0.0;
        //pVeh->m_ucmd.forwardmove = 127;
    } else {
        speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
    }
    speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;

    curTime = (*addr_of!(level)).time;

    if !(*pVeh).m_pPilot.is_null() /*&& (pilotPS->weapon == WP_NONE || pilotPS->weapon == WP_MELEE )*/ &&
        ((*pVeh).m_ucmd.buttons & BUTTON_ALT_ATTACK) != 0 && (*(*pVeh).m_pVehicleInfo).turboSpeed != 0.0
        /*||
        (parentPS && parentPS->electrifyTime > curTime && pVeh->m_pVehicleInfo->turboSpeed)*/
    //make them go!
    {
        if (!parentPS.is_null() && (*parentPS).electrifyTime > curTime)
            || (!(*(*pVeh).m_pPilot).playerState.is_null()
                && ((*(*(*pVeh).m_pPilot).playerState).weapon as i32 == WP_MELEE
                    || ((*(*(*pVeh).m_pPilot).playerState).weapon as i32 == WP_SABER
                        && BG_SabersOff((*(*pVeh).m_pPilot).playerState) != 0)))
        {
            if (curTime - (*pVeh).m_iTurboTime) > (*(*pVeh).m_pVehicleInfo).turboRecharge {
                (*pVeh).m_iTurboTime = curTime + (*(*pVeh).m_pVehicleInfo).turboDuration;
                if (*(*pVeh).m_pVehicleInfo).iTurboStartFX != 0 {
                    let mut i: usize = 0;
                    while i < MAX_VEHICLE_EXHAUSTS && (*pVeh).m_iExhaustTag[i] != -1 {
                        if !(*pVeh).m_pParentEntity.is_null()
                            && !(*(*pVeh).m_pParentEntity).ghoul2.is_null()
                            && !(*(*pVeh).m_pParentEntity).playerState.is_null()
                        {
                            //fine, I'll use a tempent for this, but only because it's played only once at the start of a turbo.
                            let mut boltOrg: vec3_t = [0.0; 3];
                            let mut boltDir: vec3_t = [0.0; 3];
                            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();

                            boltDir[0] = 0.0f32;
                            boltDir[1] = (*(*(*pVeh).m_pParentEntity).playerState).viewangles[YAW];
                            boltDir[2] = 0.0f32;

                            trap::G2API_GetBoltMatrix(
                                (*(*pVeh).m_pParentEntity).ghoul2,
                                0,
                                (*pVeh).m_iExhaustTag[i],
                                &mut boltMatrix,
                                &boltDir,
                                &(*(*(*pVeh).m_pParentEntity).playerState).origin,
                                (*addr_of!(level)).time,
                                null_mut(),
                                &(*(*pVeh).m_pParentEntity).modelScale,
                            );
                            BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut boltOrg);
                            BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut boltDir);
                            G_PlayEffectID(
                                (*(*pVeh).m_pVehicleInfo).iTurboStartFX,
                                &boltOrg,
                                &boltDir,
                            );
                        }
                        i += 1;
                    }
                }
                (*parentPS).speed = (*(*pVeh).m_pVehicleInfo).turboSpeed; // Instantly Jump To Turbo Speed
            }
        }
    }

    // Slide Breaking
    if (*pVeh).m_ulFlags & VEH_SLIDEBREAKING as c_ulong != 0 {
        if (*pVeh).m_ucmd.forwardmove >= 0 {
            (*pVeh).m_ulFlags &= !(VEH_SLIDEBREAKING as c_ulong);
        }
        (*parentPS).speed = 0.0;
    } else if curTime > (*pVeh).m_iTurboTime
        && (*pVeh).m_ulFlags & VEH_FLYING as c_ulong == 0
        && (*pVeh).m_ucmd.forwardmove < 0
        && ((*(*pVeh).m_vOrientation.add(ROLL)).abs()) > 25.0f32
    {
        (*pVeh).m_ulFlags |= VEH_SLIDEBREAKING as c_ulong;
    }

    if curTime < (*pVeh).m_iTurboTime {
        speedMax = (*(*pVeh).m_pVehicleInfo).turboSpeed;
        if !parentPS.is_null() {
            (*parentPS).eFlags |= EF_JETPACK_ACTIVE;
        }
    } else {
        speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;
        if !parentPS.is_null() {
            (*parentPS).eFlags &= !EF_JETPACK_ACTIVE;
        }
    }

    speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle;
    speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
    speedMin = (*(*pVeh).m_pVehicleInfo).speedMin;
    let _ = speedIdleAccel;

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
    } else if (*(*pVeh).m_pVehicleInfo).strafePerc == 0.0
        || (0 != 0 && (*(*pVeh).m_pParentEntity).s.number < MAX_CLIENTS as i32)
    {
        //if in a strafe-capable vehicle, clear strafing unless using alternate control scheme
        //pVeh->m_ucmd.rightmove = 0;
    }

    if (*parentPS).speed > speedMax {
        (*parentPS).speed = speedMax;
    } else if (*parentPS).speed < speedMin {
        (*parentPS).speed = speedMin;
    }

    if !parentPS.is_null() && (*parentPS).electrifyTime > curTime {
        (*parentPS).speed *= (*pVeh).m_fTimeModifier / 60.0f32;
    }

    let _ = pilotPS;

    /********************************************************************************/
    /*	END Here is where we move the vehicle (forward or back or whatever). END	*/
    /********************************************************************************/
}

//MP RULE - ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
//If you really need to violate this rule for SP, then use ifdefs.
//By BG-compatible, I mean no use of game-specific data - ONLY use
//stuff available in the MP bgEntity (in SP, the bgEntity is #defined
//as a gentity, but the MP-compatible access restrictions are based
//on the bgEntity structure in the MP codebase) -rww
//Oh, and please, use "< MAX_CLIENTS" to check for "player" and not
//"!s.number", this is a universal check that will work for both SP
//and MP. -rww
// ProcessOrientCommands the Vehicle.
/// `ProcessOrientCommands` (SpeederNPC.c:505).
pub unsafe extern "C" fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    /********************************************************************************/
    /*	BEGIN	Here is where make sure the vehicle is properly oriented.	BEGIN	*/
    /********************************************************************************/
    let riderPS: *mut playerState_t;
    let parentPS: *mut playerState_t;

    let mut angDif: f32;

    if !(*pVeh).m_pPilot.is_null() {
        riderPS = (*(*pVeh).m_pPilot).playerState;
    } else {
        riderPS = (*(*pVeh).m_pParentEntity).playerState;
    }
    parentPS = (*(*pVeh).m_pParentEntity).playerState;

    //pVeh->m_vOrientation[YAW] = 0.0f;//riderPS->viewangles[YAW];
    angDif = AngleSubtract(*(*pVeh).m_vOrientation.add(YAW), (*riderPS).viewangles[YAW]);
    if !parentPS.is_null() && (*parentPS).speed != 0.0 {
        let mut s = (*parentPS).speed;
        let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 4.0f32; //magic number hackery
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

        if (*parentPS).electrifyTime > (*pm).cmd.serverTime {
            //do some crazy stuff
            *(*pVeh).m_vOrientation.add(YAW) +=
                (((*pm).cmd.serverTime as f32 / 1000.0f32).sin() * 3.0f32) * (*pVeh).m_fTimeModifier;
        }
    }

    /********************************************************************************/
    /*	END	Here is where make sure the vehicle is properly oriented.	END			*/
    /********************************************************************************/
}

// This function makes sure that the vehicle is properly animated.
/// `AnimateVehicle` (SpeederNPC.c:608) — empty in the QAGAME build.
pub unsafe extern "C" fn AnimateVehicle(_pVeh: *mut Vehicle_t) {}

// SpeederNPC.c:139 `#else` — the MP `VEH_StartStrafeRam`. The rich SP body (`#ifndef _JK2MP`
// @:102) is excluded; the MP build compiles only this trivial stub.
/// `VEH_StartStrafeRam` (SpeederNPC.c:140, MP `#else` stub).
pub unsafe fn VEH_StartStrafeRam(
    _pVeh: *mut Vehicle_t,
    _Right: qboolean,
    _Duration: c_int,
) -> qboolean {
    QFALSE
}

// Like a think or move command, this updates various vehicle properties.
/// `Update` (SpeederNPC.c:149, `#ifdef QAGAME`).
pub unsafe extern "C" fn Update(pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    if ((*addr_of!(g_vehicleInfo))[VEHICLE_BASE as usize].Update.unwrap())(pVeh, pUcmd) == QFALSE {
        return QFALSE;
    }

    // See whether this vehicle should be exploding.
    if (*pVeh).m_iDieTime != 0 {
        ((*(*pVeh).m_pVehicleInfo).DeathUpdate.unwrap())(pVeh);
    }

    // Update move direction.
    // #ifndef _JK2MP //this makes prediction unhappy, and rightfully so. -- the SP move-direction /
    // strafe-ram / exhaust-FX (G_PlayEffect/G_StopEffect) / armor-gone (G_SoundIndex) block is
    // SP-only and excluded from the MP build.

    QTRUE
}

//rwwFIXMEFIXME: This is all going to have to be predicted I think, or it will feel awful and lagged
//would be nice to have proper prediction of animations. -rww
// This function makes sure that the rider's in this vehicle are properly animated.
/// `AnimateRiders` (SpeederNPC.c:630).
//
// MP build note: at `#ifdef _JK2MP if (1) return;` (:740) the function returns right after the
// boarding block, so the entire rider-pose tail below it is dead-but-compiled. It is carried
// faithfully (hence `#[allow(unreachable_code)]`); within it, the SP-only sub-blocks
// (`CG_ChangeWeapon`/`G_RemoveWeaponModels` @:798, `gi.G2API_GetBoneAnimIndex` @:818, the SP
// enemy auto-aim @:862) are `#ifndef _JK2MP` and excluded.
#[allow(unreachable_code)]
#[allow(unused_assignments)] // faithful: dead-tail stores into Anim/iFlags/iBlend are never read (unreachable after the MP `if (1) return`)
pub unsafe extern "C" fn AnimateRiders(pVeh: *mut Vehicle_t) {
    let mut Anim: animNumber_t = BOTH_VS_IDLE;
    let fSpeedPercToMax: f32;
    let mut iFlags: c_int = SETANIM_FLAG_NORMAL;
    let mut iBlend: c_int = 300;
    let pilotPS: *mut playerState_t;
    let parentPS: *mut playerState_t;
    let curTime: c_int;

    // Boarding animation.
    if (*pVeh).m_iBoarding != 0 {
        // We've just started moarding, set the amount of time it will take to finish moarding.
        if (*pVeh).m_iBoarding < 0 {
            let iAnimLen: c_int;

            // Boarding from left...
            if (*pVeh).m_iBoarding == -1 {
                Anim = BOTH_VS_MOUNT_L;
            } else if (*pVeh).m_iBoarding == -2 {
                Anim = BOTH_VS_MOUNT_R;
            } else if (*pVeh).m_iBoarding == -3 {
                Anim = BOTH_VS_MOUNTJUMP_L;
            } else if (*pVeh).m_iBoarding == VEH_MOUNT_THROW_LEFT {
                iBlend = 0;
                Anim = BOTH_VS_MOUNTTHROW_R;
            } else if (*pVeh).m_iBoarding == VEH_MOUNT_THROW_RIGHT {
                iBlend = 0;
                Anim = BOTH_VS_MOUNTTHROW_L;
            }

            // Set the delay time (which happens to be the time it takes for the animation to complete).
            // NOTE: Here I made it so the delay is actually 40% (0.4f) of the animation time.
            //#ifdef _JK2MP
            iAnimLen = (BG_AnimLength((*(*pVeh).m_pPilot).localAnimIndex, Anim) as f32 * 0.4) as c_int;
            (*pVeh).m_iBoarding = BG_GetTime() + iAnimLen;

            // Set the animation, which won't be interrupted until it's completed.
            // TODO: But what if he's killed? Should the animation remain persistant???
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;

            //#ifdef _JK2MP
            BG_SetAnim(
                (*(*pVeh).m_pPilot).playerState,
                (*addr_of!(bgAllAnims))[(*(*pVeh).m_pPilot).localAnimIndex as usize].anims,
                SETANIM_BOTH,
                Anim,
                iFlags,
                iBlend,
            );
        }

        // #ifndef _JK2MP: the SP m_pOldPilot / G_StartMatrixEffect / Eject / G_Throw block is excluded.

        return;
    }

    //#ifdef _JK2MP //fixme
    if true {
        return;
    }

    //#ifdef _JK2MP
    pilotPS = (*(*pVeh).m_pPilot).playerState;
    parentPS = (*(*pVeh).m_pPilot).playerState;

    //#elif QAGAME//MP GAME
    curTime = level.time;

    // Percentage of maximum speed relative to current speed.
    fSpeedPercToMax = (*parentPS).speed / (*(*pVeh).m_pVehicleInfo).speedMax;
    let _ = fSpeedPercToMax;

    // Going in reverse...
    //#ifdef _JK2MP
    if (*pVeh).m_ucmd.forwardmove < 0 && ((*pVeh).m_ulFlags & VEH_SLIDEBREAKING as c_ulong) == 0 {
        Anim = BOTH_VS_REV;
        iBlend = 500;
    } else {
        let HasWeapon: bool = (*pilotPS).weapon != WP_NONE && (*pilotPS).weapon != WP_MELEE;
        let Attacking: bool = HasWeapon && ((*pVeh).m_ucmd.buttons & BUTTON_ATTACK) != 0;
        //#ifdef _JK2MP //fixme: flying tends to spaz out a lot
        let Flying: bool = false;
        let Crashing: bool = false;
        let mut Right: bool = (*pVeh).m_ucmd.rightmove > 0;
        let mut Left: bool = (*pVeh).m_ucmd.rightmove < 0;
        let Turbo: bool = curTime < (*pVeh).m_iTurboTime;
        let mut WeaponPose: EWeaponPose = WPOSE_NONE;

        // Remove Crashing Flag
        //----------------------
        (*pVeh).m_ulFlags &= !(VEH_CRASHING as c_ulong);

        // Put Away Saber When It Is Not Active
        //-------------------------------------- (#ifndef _JK2MP: SP CG_ChangeWeapon/G_RemoveWeaponModels block excluded)

        // Don't Interrupt Attack Anims
        //------------------------------
        //#ifdef _JK2MP
        if (*pilotPS).weaponTime > 0 {
            return;
        }

        // Compute The Weapon Pose
        //--------------------------
        if (*pilotPS).weapon == WP_BLASTER {
            WeaponPose = WPOSE_BLASTER;
        } else if (*pilotPS).weapon == WP_SABER {
            if ((*pVeh).m_ulFlags & VEH_SABERINLEFTHAND as c_ulong) != 0
                && (*pilotPS).torsoAnim == BOTH_VS_ATL_TO_R_S
            {
                (*pVeh).m_ulFlags &= !(VEH_SABERINLEFTHAND as c_ulong);
            }
            if ((*pVeh).m_ulFlags & VEH_SABERINLEFTHAND as c_ulong) == 0
                && (*pilotPS).torsoAnim == BOTH_VS_ATR_TO_L_S
            {
                (*pVeh).m_ulFlags |= VEH_SABERINLEFTHAND as c_ulong;
            }
            WeaponPose = if ((*pVeh).m_ulFlags & VEH_SABERINLEFTHAND as c_ulong) != 0 {
                WPOSE_SABERLEFT
            } else {
                WPOSE_SABERRIGHT
            };
        }

        if Attacking && WeaponPose != 0 {
            // Attack!
            iBlend = 100;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART;

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
                    WPOSE_BLASTER => Anim = BOTH_VS_ATL_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VS_ATL_S,
                    WPOSE_SABERRIGHT => Anim = BOTH_VS_ATR_TO_L_S,
                    _ => debug_assert!(false),
                }
            } else if Right {
                // Attack Right
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VS_ATR_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VS_ATL_TO_R_S,
                    WPOSE_SABERRIGHT => Anim = BOTH_VS_ATR_S,
                    _ => debug_assert!(false),
                }
            } else {
                // Attack Ahead
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VS_ATF_G,
                    _ => debug_assert!(false),
                }
            }
        } else if Left && ((*pVeh).m_ucmd.buttons & BUTTON_USE) != 0 {
            // Look To The Left Behind
            iBlend = 400;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
            match WeaponPose {
                WPOSE_SABERLEFT => Anim = BOTH_VS_IDLE_SL,
                WPOSE_SABERRIGHT => Anim = BOTH_VS_IDLE_SR,
                _ => Anim = BOTH_VS_LOOKLEFT,
            }
        } else if Right && ((*pVeh).m_ucmd.buttons & BUTTON_USE) != 0 {
            // Look To The Right Behind
            iBlend = 400;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
            match WeaponPose {
                WPOSE_SABERLEFT => Anim = BOTH_VS_IDLE_SL,
                WPOSE_SABERRIGHT => Anim = BOTH_VS_IDLE_SR,
                _ => Anim = BOTH_VS_LOOKRIGHT,
            }
        } else if Turbo {
            // Kicked In Turbo
            iBlend = 50;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLDLESS;
            Anim = BOTH_VS_TURBO;
        } else if Flying {
            // Off the ground in a jump
            iBlend = 800;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;

            match WeaponPose {
                WPOSE_NONE => Anim = BOTH_VS_AIR,
                WPOSE_BLASTER => Anim = BOTH_VS_AIR_G,
                WPOSE_SABERLEFT => Anim = BOTH_VS_AIR_SL,
                WPOSE_SABERRIGHT => Anim = BOTH_VS_AIR_SR,
                _ => debug_assert!(false),
            }
        } else if Crashing {
            // Hit the ground!
            iBlend = 100;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLDLESS;

            match WeaponPose {
                WPOSE_NONE => Anim = BOTH_VS_LAND,
                WPOSE_BLASTER => Anim = BOTH_VS_LAND_G,
                WPOSE_SABERLEFT => Anim = BOTH_VS_LAND_SL,
                WPOSE_SABERRIGHT => Anim = BOTH_VS_LAND_SR,
                _ => debug_assert!(false),
            }
        } else {
            // No Special Moves
            iBlend = 300;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLDLESS;

            if *(*pVeh).m_vOrientation.add(ROLL) <= -20.0 {
                // Lean Left
                match WeaponPose {
                    WPOSE_NONE => Anim = BOTH_VS_LEANL,
                    WPOSE_BLASTER => Anim = BOTH_VS_LEANL_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VS_LEANL_SL,
                    WPOSE_SABERRIGHT => Anim = BOTH_VS_LEANL_SR,
                    _ => debug_assert!(false),
                }
            } else if *(*pVeh).m_vOrientation.add(ROLL) >= 20.0 {
                // Lean Right
                match WeaponPose {
                    WPOSE_NONE => Anim = BOTH_VS_LEANR,
                    WPOSE_BLASTER => Anim = BOTH_VS_LEANR_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VS_LEANR_SL,
                    WPOSE_SABERRIGHT => Anim = BOTH_VS_LEANR_SR,
                    _ => debug_assert!(false),
                }
            } else {
                // No Lean
                match WeaponPose {
                    WPOSE_NONE => Anim = BOTH_VS_IDLE,
                    WPOSE_BLASTER => Anim = BOTH_VS_IDLE_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VS_IDLE_SL,
                    WPOSE_SABERRIGHT => Anim = BOTH_VS_IDLE_SR,
                    _ => debug_assert!(false),
                }
            }
        } // No Special Moves
    } // Going backwards?

    //#ifdef _JK2MP
    iFlags &= !SETANIM_FLAG_OVERRIDE;
    if (*(*(*pVeh).m_pPilot).playerState).torsoAnim == Anim {
        (*(*(*pVeh).m_pPilot).playerState).torsoTimer =
            BG_AnimLength((*(*pVeh).m_pPilot).localAnimIndex, Anim);
    }
    if (*(*(*pVeh).m_pPilot).playerState).legsAnim == Anim {
        (*(*(*pVeh).m_pPilot).playerState).legsTimer =
            BG_AnimLength((*(*pVeh).m_pPilot).localAnimIndex, Anim);
    }
    BG_SetAnim(
        (*(*pVeh).m_pPilot).playerState,
        (*addr_of!(bgAllAnims))[(*(*pVeh).m_pPilot).localAnimIndex as usize].anims,
        SETANIM_BOTH,
        Anim,
        iFlags | SETANIM_FLAG_HOLD,
        iBlend,
    );
}

// SpeederNPC.c:1041 `#ifndef QAGAME extern void AttachRidersGeneric(...)` — excluded (QAGAME defined).
//on the client this function will only set up the process command funcs
/// `G_SetSpeederVehicleFunctions` (SpeederNPC.c:1044).
pub unsafe extern "C" fn G_SetSpeederVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
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
    //	pVehInfo->DeathUpdate				=		DeathUpdate;
    //	pVehInfo->RegisterAssets			=		RegisterAssets;
    //	pVehInfo->Initialize				=		Initialize;
    (*pVehInfo).Update = Some(Update);
    //	pVehInfo->UpdateRider				=		UpdateRider;
    //#endif

    //shared
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
//
// No-oracle: allocates from the module-static vehicle pool via
// `G_AllocateVehicleObject` and zeroes/initialises `Vehicle_t` through pointers.
pub unsafe extern "C" fn G_CreateSpeederNPC(pVeh: *mut *mut Vehicle_t, strType: *const c_char) {
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
