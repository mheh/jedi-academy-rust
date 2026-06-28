// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

//seems to be a compiler bug, it doesn't clean out the #ifdefs between dif-compiles
//or something, so the headers spew errors on these defs from the previous compile.
//this fixes that. -rww
#[cfg(feature = "jk2mp")]
// get rid of all the crazy defs we added for this file
// #undef currentAngles
// #undef currentOrigin
// #undef mins
// #undef maxs
// #undef legsAnimTimer
// #undef torsoAnimTimer
// #undef bool
// #undef false
// #undef true
// #undef sqrtf
// #undef Q_flrand
// #undef MOD_EXPLOSIVE

#[cfg(feature = "jk2")]
#[cfg(not(feature = "jk2mp"))]
// #define _JK2MP (from #ifdef _JK2)

#[cfg(not(feature = "jk2mp"))]
#[cfg(not(feature = "qagame"))]
// #define QAGAME (from SP single player logic)

#[cfg(feature = "qagame")]
// #include "g_local.h"
#[cfg(all(not(feature = "qagame"), feature = "jk2mp"))]
// #include "bg_public.h"

#[cfg(not(feature = "jk2mp"))]
// #include "g_functions.h"
// #include "g_vehicles.h"
// #include "..\game\wp_saber.h"
// #include "../cgame/cg_local.h"
#[cfg(feature = "jk2mp")]
// #include "bg_vehicles.h"

use core::ffi::{c_int, c_char, c_float};

// External function declarations
extern "C" {
    pub fn DotToSpot(spot: *const [c_float; 3], from: *const [c_float; 3], fromAngles: *const [c_float; 3]) -> c_float;
}

#[cfg(all(feature = "qagame"))]
extern "C" {
    pub static mut cg_thirdPersonAlpha: vmCvar_t;
    pub static mut playerMins: [c_float; 3];
    pub static mut playerMaxs: [c_float; 3];
    pub static mut g_speederControlScheme: *mut cvar_t;

    pub fn ChangeWeapon(ent: *mut gentity_t, newWeapon: c_int);
    pub fn PM_SetAnim(pm: *mut pmove_t, setAnimParts: c_int, anim: c_int, setAnimFlags: c_int, blendTime: c_int);
    pub fn PM_AnimLength(index: c_int, anim: animNumber_t) -> c_int;
}

#[cfg(feature = "jk2mp")]
extern "C" {
    pub fn BG_SetAnim(ps: *mut playerState_t, animations: *mut animation_t, setAnimParts: c_int, anim: c_int, setAnimFlags: c_int, blendTime: c_int);
    pub fn BG_GetTime() -> c_int;
}

// Stub types for foreign types (minimal local stubs for structural coherence)
#[repr(C)]
pub struct Vehicle_t {
    // Stub - actual fields would come from vehicle definitions
    pub m_ulFlags: c_int,
    pub m_pParentEntity: *mut c_int,
    pub m_pPilot: *mut c_int,
    pub m_pVehicleInfo: *mut vehicleInfo_t,
    pub m_fStrafeTime: c_float,
    pub m_iSoundDebounceTimer: c_int,
    pub m_iExhaustTag: [c_int; 4], // MAX_VEHICLE_EXHAUSTS
    pub m_iDieTime: c_int,
    pub m_vOrientation: [c_float; 3],
    pub m_ucmd: usercmd_t,
    pub m_fTimeModifier: c_float,
    pub m_iTurboTime: c_int,
    pub m_iArmor: c_int,
    pub m_iBoarding: c_int,
    pub m_pOldPilot: *mut c_int,
    pub m_vAngularVelocity: c_float,
}

#[repr(C)]
pub struct vehicleInfo_t {
    // Stub
}

#[repr(C)]
pub struct usercmd_t {
    pub forwardmove: c_int,
    pub rightmove: c_int,
    pub upmove: c_int,
    pub buttons: c_int,
}

#[repr(C)]
pub struct playerState_t {
    // Stub
}

#[repr(C)]
pub struct gentity_t {
    // Stub
}

#[repr(C)]
pub struct pmove_t {
    // Stub
}

#[repr(C)]
pub struct animation_t {
    // Stub
}

#[repr(C)]
pub struct cvar_t {
    // Stub
}

#[repr(C)]
pub struct vmCvar_t {
    // Stub
}

pub type animNumber_t = c_int;

// Constants
const STRAFERAM_DURATION: c_int = 8;
const STRAFERAM_ANGLE: c_int = 8;

const MAX_VEHICLE_EXHAUSTS: c_int = 4;

// VEH flags
const VEH_STRAFERAM: c_int = 0x1;
const VEH_FLYING: c_int = 0x2;
const VEH_ACCELERATORON: c_int = 0x4;
const VEH_ARMORLOW: c_int = 0x8;
const VEH_ARMORGONE: c_int = 0x10;
const VEH_SLIDEBREAKING: c_int = 0x20;
const VEH_OUTOFCONTROL: c_int = 0x40;
const VEH_CRASHING: c_int = 0x80;
const VEH_SABERINLEFTHAND: c_int = 0x100;

// Animation and flag constants
const SETANIM_FLAG_NORMAL: c_int = 0;
const SETANIM_FLAG_OVERRIDE: c_int = 1;
const SETANIM_FLAG_HOLD: c_int = 2;
const SETANIM_FLAG_HOLDLESS: c_int = 4;
const SETANIM_FLAG_RESTART: c_int = 8;
const SETANIM_BOTH: c_int = 0;

// Weapon poses
const WPOSE_NONE: c_int = 0;
const WPOSE_BLASTER: c_int = 1;
const WPOSE_SABERLEFT: c_int = 2;
const WPOSE_SABERRIGHT: c_int = 3;

pub type EWeaponPose = c_int;

#[cfg(not(feature = "jk2mp"))]
pub fn VEH_StartStrafeRam(pVeh: *mut Vehicle_t, Right: bool) -> bool {
    unsafe {
        if !((*pVeh).m_ulFlags & VEH_STRAFERAM) != 0 {
            // let speed = VectorLength(pVeh->m_pParentEntity->client->ps.velocity);
            // Stub: assuming speed calculation would happen here
            let speed = 0.0f32; // TODO: implement VectorLength
            if speed > 400.0f {
                // Compute Pos3
                //--------------
                // let right = [0.0f; 3];
                // AngleVectors(pVeh->m_vOrientation, 0, right, 0);
                // VectorMA(pVeh->m_pParentEntity->client->ps.velocity, (Right)?( speed):(-speed), right, pVeh->m_pParentEntity->pos3);

                (*pVeh).m_ulFlags |= VEH_STRAFERAM;
                (*pVeh).m_fStrafeTime = if Right {
                    STRAFERAM_DURATION as c_float
                } else {
                    -(STRAFERAM_DURATION as c_float)
                };

                // Stub: sound handling would go here

                return true;
            }
        }
        false
    }
}

#[cfg(feature = "jk2mp")]
pub fn VEH_StartStrafeRam(_pVeh: *mut Vehicle_t, _Right: bool, _Duration: c_int) -> bool {
    false
}

#[cfg(feature = "qagame")]
extern "C" {
    pub static mut g_vehicleInfo: [vehicleInfo_t; 64]; // Stub array
    pub static mut level: Level_t;

    pub fn G_SoundIndexOnEnt(ent: *mut gentity_t, channel: c_int, soundIndex: c_int);
    pub fn G_SoundIndex(name: *const c_char) -> c_int;
    pub fn G_PlayEffect(fx: c_int, modelIndex: c_int, boltIndex: c_int, entityNum: c_int, origin: *const [c_float; 3], duration: c_int, looping: bool);
    pub fn G_StopEffect(fx: c_int, modelIndex: c_int, boltIndex: c_int, entityNum: c_int);
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
}

#[repr(C)]
pub struct Level_t {
    // Stub
    pub time: c_int,
}

#[cfg(feature = "qagame")]
// Like a think or move command, this updates various vehicle properties.
pub fn Update(pVeh: *mut Vehicle_t, _pUcmd: *const usercmd_t) -> bool {
    unsafe {
        if !(*g_vehicleInfo[0].Update)(pVeh, _pUcmd) {
            return false;
        }

        // See whether this vehicle should be exploding.
        if (*pVeh).m_iDieTime != 0 {
            ((*(*pVeh).m_pVehicleInfo).DeathUpdate)(pVeh);
        }

        // Update move direction.
        #[cfg(not(feature = "jk2mp"))]
        {
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;

            if (*pVeh).m_ulFlags & VEH_FLYING != 0 {
                let mut vVehAngles = [0.0f32; 3];
                vVehAngles[1] = (*pVeh).m_vOrientation[1]; // YAW
                // AngleVectors( vVehAngles, parent->client->ps.moveDir, NULL, NULL );
            } else {
                let mut vVehAngles = [0.0f32; 3];
                vVehAngles[0] = (*pVeh).m_vOrientation[0]; // PITCH
                vVehAngles[1] = (*pVeh).m_vOrientation[1]; // YAW
                // AngleVectors( vVehAngles, parent->client->ps.moveDir, NULL, NULL );
            }

            // Check For A Strafe Ram
            //------------------------
            if ((*pVeh).m_ulFlags & VEH_STRAFERAM) == 0 && ((*pVeh).m_ulFlags & VEH_FLYING) == 0 {
                // Started A Strafe
                //------------------
                if (*pVeh).m_ucmd.rightmove != 0 && (*pVeh).m_fStrafeTime == 0.0f {
                    (*pVeh).m_fStrafeTime = if (*pVeh).m_ucmd.rightmove > 0 {
                        level.time as c_float
                    } else {
                        -(level.time as c_float)
                    };
                }
                // Ended A Strafe
                //----------------
                else if (*pVeh).m_ucmd.rightmove == 0 && (*pVeh).m_fStrafeTime != 0.0f {
                    // If It Was A Short Burst, Start The Strafe Ram
                    //-----------------------------------------------
                    if (level.time - ((*pVeh).m_fStrafeTime.abs() as c_int)) < 300 {
                        if !VEH_StartStrafeRam(pVeh, (*pVeh).m_fStrafeTime > 0.0f) {
                            (*pVeh).m_fStrafeTime = 0.0f;
                        }
                    }
                    // Otherwise, Clear The Timer
                    //----------------------------
                    else {
                        (*pVeh).m_fStrafeTime = 0.0f;
                    }
                }
            }

            // If Currently In A StrafeRam, Check To See If It Is Done (Timed Out)
            //---------------------------------------------------------------------
            else if (*pVeh).m_fStrafeTime == 0.0f {
                (*pVeh).m_ulFlags &= !VEH_STRAFERAM;
            }

            // Exhaust Effects Start And Stop When The Accelerator Is Pressed
            //----------------------------------------------------------------
            if (*(*pVeh).m_pVehicleInfo).iExhaustFX != 0 {
                // Start It On Each Exhaust Bolt
                //-------------------------------
                if (*pVeh).m_ucmd.forwardmove != 0 && ((*pVeh).m_ulFlags & VEH_ACCELERATORON) == 0 {
                    (*pVeh).m_ulFlags |= VEH_ACCELERATORON;
                    for i in 0..MAX_VEHICLE_EXHAUSTS {
                        if (*pVeh).m_iExhaustTag[i as usize] == -1 {
                            break;
                        }
                        // G_PlayEffect(pVeh->m_pVehicleInfo->iExhaustFX, parent->playerModel, pVeh->m_iExhaustTag[i], parent->s.number, parent->currentOrigin, 1, qtrue);
                    }
                }
                // Stop It On Each Exhaust Bolt
                //------------------------------
                else if (*pVeh).m_ucmd.forwardmove == 0 && ((*pVeh).m_ulFlags & VEH_ACCELERATORON) != 0 {
                    (*pVeh).m_ulFlags &= !VEH_ACCELERATORON;
                    for i in 0..MAX_VEHICLE_EXHAUSTS {
                        if (*pVeh).m_iExhaustTag[i as usize] == -1 {
                            break;
                        }
                        // G_StopEffect(pVeh->m_pVehicleInfo->iExhaustFX, parent->playerModel, pVeh->m_iExhaustTag[i], parent->s.number);
                    }
                }
            }

            if ((*pVeh).m_ulFlags & VEH_ARMORLOW) == 0 && ((*pVeh).m_iArmor <= (*(*pVeh).m_pVehicleInfo).armor / 3) {
                (*pVeh).m_ulFlags |= VEH_ARMORLOW;
            }

            // Armor Gone Effects (Fire)
            //---------------------------
            if (*(*pVeh).m_pVehicleInfo).iArmorGoneFX != 0 {
                if ((*pVeh).m_ulFlags & VEH_ARMORGONE) == 0 && ((*pVeh).m_iArmor <= 0) {
                    (*pVeh).m_ulFlags |= VEH_ARMORGONE;
                    // G_PlayEffect(pVeh->m_pVehicleInfo->iArmorGoneFX, parent->playerModel, parent->crotchBolt, parent->s.number, parent->currentOrigin, 1, qtrue);
                    // parent->s.loopSound = G_SoundIndex( "sound/vehicles/common/fire_lp.wav" );
                }
            }
        }

        true
    }
}

//MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
//If you really need to violate this rule for SP, then use ifdefs.
//By BG-compatible, I mean no use of game-specific data - ONLY use
//stuff available in the MP bgEntity (in SP, the bgEntity is #defined
//as a gentity, but the MP-compatible access restrictions are based
//on the bgEntity structure in the MP codebase) -rww
// ProcessMoveCommands the Vehicle.
fn ProcessMoveCommands(pVeh: *mut Vehicle_t) {
    /************************************************************************************/
    /*	BEGIN	Here is where we move the vehicle (forward or back or whatever). BEGIN	*/
    /************************************************************************************/
    //Client sets ucmds and such for speed alterations
    let mut speedInc: c_float;
    let speedIdleDec: c_float;
    let speedIdle: c_float;
    let speedIdleAccel: c_float;
    let speedMin: c_float;
    let mut speedMax: c_float;
    let mut parentPS: *mut playerState_t;
    let mut pilotPS: *mut playerState_t = std::ptr::null_mut();
    let curTime: c_int;

    unsafe {
        #[cfg(feature = "jk2mp")]
        {
            parentPS = (*(*pVeh).m_pParentEntity).playerState as *mut playerState_t;
            if !(*pVeh).m_pPilot.is_null() {
                pilotPS = (*(*pVeh).m_pPilot).playerState as *mut playerState_t;
            }
        }

        #[cfg(not(feature = "jk2mp"))]
        {
            // parentPS = &pVeh->m_pParentEntity->client->ps;
            // if (pVeh->m_pPilot)
            // {
            //     pilotPS = &pVeh->m_pPilot->client->ps;
            // }
            parentPS = std::ptr::null_mut(); // Stub
            pilotPS = std::ptr::null_mut();
        }

        // If we're flying, make us accelerate at 40% (about half) acceleration rate, and restore the pitch
        // to origin (straight) position (at 5% increments).
        if (*pVeh).m_ulFlags & VEH_FLYING != 0 {
            speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier * 0.4f;
        }
        #[cfg(feature = "jk2mp")]
        {
            if (*parentPS).m_iVehicleNum == 0 {
                //drifts to a stop
                speedInc = 0.0f;
                //pVeh->m_ucmd.forwardmove = 127;
            } else {
                speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
            }
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            if !(*(*pVeh).m_pVehicleInfo).Inhabited.is_null()
                && ((*(*pVeh).m_pVehicleInfo).Inhabited)(pVeh) == 0
            {
                //drifts to a stop
                speedInc = 0.0f;
                //pVeh->m_ucmd.forwardmove = 127;
            } else {
                speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
            }
        }

        speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;

        #[cfg(not(feature = "jk2mp"))]
        {
            curTime = level.time;
        }
        #[cfg(all(feature = "jk2mp", feature = "qagame"))]
        {
            curTime = level.time;
        }
        #[cfg(all(feature = "jk2mp", not(feature = "qagame")))]
        {
            //FIXME: pass in ucmd?  Not sure if this is reliable...
            // curTime = pm->cmd.serverTime;
            curTime = 0; // Stub
        }

        if (!(*pVeh).m_pPilot.is_null() /*&& (pilotPS->weapon == WP_NONE || pilotPS->weapon == WP_MELEE)*/
            && ((*pVeh).m_ucmd.buttons & 0x1) != 0
            && (*(*pVeh).m_pVehicleInfo).turboSpeed != 0.0f)
            #[cfg(feature = "jk2mp")]
            || (!parentPS.is_null() && 0 > curTime && (*(*pVeh).m_pVehicleInfo).turboSpeed != 0.0f) //make them go!
        {
            #[cfg(feature = "jk2mp")]
            {
                if (!parentPS.is_null() && 0 > curTime)
                    || (!(*(*pVeh).m_pPilot).playerState.is_null()
                        && (0 == 0 || (0 != 0 && 0 != 0)))
                {
                    // Inner block for MP
                }
            }

            if (curTime - (*pVeh).m_iTurboTime) > (*(*pVeh).m_pVehicleInfo).turboRecharge {
                (*pVeh).m_iTurboTime = curTime + (*(*pVeh).m_pVehicleInfo).turboDuration;
                if (*(*pVeh).m_pVehicleInfo).iTurboStartFX != 0 {
                    for i in 0..MAX_VEHICLE_EXHAUSTS {
                        if (*pVeh).m_iExhaustTag[i as usize] == -1 {
                            break;
                        }

                        #[cfg(not(feature = "jk2mp"))]
                        {
                            // Start The Turbo Fx Start
                            //--------------------------
                            // G_PlayEffect(pVeh->m_pVehicleInfo->iTurboStartFX, pVeh->m_pParentEntity->playerModel, pVeh->m_iExhaustTag[i], pVeh->m_pParentEntity->s.number, pVeh->m_pParentEntity->currentOrigin );

                            // Start The Looping Effect
                            //--------------------------
                            // if (pVeh->m_pVehicleInfo->iTurboFX)
                            // {
                            //     G_PlayEffect(pVeh->m_pVehicleInfo->iTurboFX, pVeh->m_pParentEntity->playerModel, pVeh->m_iExhaustTag[i], pVeh->m_pParentEntity->s.number, pVeh->m_pParentEntity->currentOrigin, pVeh->m_pVehicleInfo->turboDuration, qtrue);
                            // }
                        }

                        #[cfg(feature = "jk2mp")]
                        {
                            #[cfg(feature = "qagame")]
                            {
                                // if (pVeh->m_pParentEntity &&
                                //     pVeh->m_pParentEntity->ghoul2 &&
                                //     pVeh->m_pParentEntity->playerState)
                                // { //fine, I'll use a tempent for this, but only because it's played only once at the start of a turbo.
                                //     vec3_t boltOrg, boltDir;
                                //     mdxaBone_t boltMatrix;
                                //     ...
                                // }
                            }
                        }
                    }
                }

                #[cfg(not(feature = "jk2mp"))]
                {
                    if (*(*pVeh).m_pVehicleInfo).soundTurbo != 0 {
                        // G_SoundIndexOnEnt(pVeh->m_pParentEntity, CHAN_AUTO, pVeh->m_pVehicleInfo->soundTurbo);
                    }
                }

                // parentPS->speed = pVeh->m_pVehicleInfo->turboSpeed; // Instantly Jump To Turbo Speed
            }
        }

        // Slide Breaking
        if (*pVeh).m_ulFlags & VEH_SLIDEBREAKING != 0 {
            let mut should_clear = false;
            if (*pVeh).m_ucmd.forwardmove >= 0 {
                should_clear = true;
            }
            #[cfg(not(feature = "jk2mp"))]
            {
                if !should_clear && (level.time - 500) > 0 {
                    should_clear = true;
                }
            }

            if should_clear {
                (*pVeh).m_ulFlags &= !VEH_SLIDEBREAKING;
            }
            // parentPS->speed = 0;
        } else if (curTime > (*pVeh).m_iTurboTime)
            && ((*pVeh).m_ulFlags & VEH_FLYING) == 0
            && (*pVeh).m_ucmd.forwardmove < 0
            && ((*pVeh).m_vOrientation[2].abs() > 25.0f) // ROLL
        {
            (*pVeh).m_ulFlags |= VEH_SLIDEBREAKING;
        }

        if curTime < (*pVeh).m_iTurboTime {
            speedMax = (*(*pVeh).m_pVehicleInfo).turboSpeed;
            // if (parentPS)
            // {
            //     parentPS->eFlags |= EF_JETPACK_ACTIVE;
            // }
        } else {
            speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;
            // if (parentPS)
            // {
            //     parentPS->eFlags &= ~EF_JETPACK_ACTIVE;
            // }
        }

        speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle;
        speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
        speedMin = (*(*pVeh).m_pVehicleInfo).speedMin;

        // Speed handling logic (stub - actual implementation depends on playerState structure)
        // if ( parentPS->speed || parentPS->groundEntityNum == ENTITYNUM_NONE  ||
        //      pVeh->m_ucmd.forwardmove || pVeh->m_ucmd.upmove > 0 )
        // {
        //     ...speed calculations...
        // }

        // if ( parentPS->speed > speedMax )
        // {
        //     parentPS->speed = speedMax;
        // }
        // else if ( parentPS->speed < speedMin )
        // {
        //     parentPS->speed = speedMin;
        // }

        #[cfg(not(feature = "jk2mp"))]
        {
            // In SP, The AI Pilots Can Directly Control The Speed Of Their Bike In Order To
            // Match The Speed Of The Person They Are Trying To Chase
            //-------------------------------------------------------------------------------
            if !(*pVeh).m_pPilot.is_null() && ((*pVeh).m_ucmd.buttons & 0x80) != 0 {
                // parentPS->speed = pVeh->m_pPilot->client->ps.speed;
            }
        }
    }

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
#[cfg(feature = "jk2mp")]
extern "C" {
    pub fn AnimalProcessOri(pVeh: *mut Vehicle_t);
}

fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    /********************************************************************************/
    /*	BEGIN	Here is where make sure the vehicle is properly oriented.	BEGIN	*/
    /********************************************************************************/
    let mut riderPS: *mut playerState_t;
    let mut parentPS: *mut playerState_t;

    unsafe {
        #[cfg(feature = "jk2mp")]
        {
            let mut angDif: c_float;

            if !(*pVeh).m_pPilot.is_null() {
                riderPS = (*(*pVeh).m_pPilot).playerState as *mut playerState_t;
            } else {
                riderPS = (*(*pVeh).m_pParentEntity).playerState as *mut playerState_t;
            }
            parentPS = (*(*pVeh).m_pParentEntity).playerState as *mut playerState_t;

            //pVeh->m_vOrientation[YAW] = 0.0f;//riderPS->viewangles[YAW];
            // angDif = AngleSubtract(pVeh->m_vOrientation[YAW], riderPS->viewangles[YAW]);
            // if (parentPS && parentPS->speed)
            // {
            //     float s = parentPS->speed;
            //     float maxDif = pVeh->m_pVehicleInfo->turningSpeed*4.0f; //magic number hackery
            //     if (s < 0.0f)
            //     {
            //         s = -s;
            //     }
            //     angDif *= s/pVeh->m_pVehicleInfo->speedMax;
            //     if (angDif > maxDif)
            //     {
            //         angDif = maxDif;
            //     }
            //     else if (angDif < -maxDif)
            //     {
            //         angDif = -maxDif;
            //     }
            //     pVeh->m_vOrientation[YAW] = AngleNormalize180(pVeh->m_vOrientation[YAW] - angDif*(pVeh->m_fTimeModifier*0.2f));
            //
            //     if (parentPS->electrifyTime > pm->cmd.serverTime)
            //     { //do some crazy stuff
            //         pVeh->m_vOrientation[YAW] += (sin(pm->cmd.serverTime/1000.0f)*3.0f)*pVeh->m_fTimeModifier;
            //     }
            // }
        }

        #[cfg(not(feature = "jk2mp"))]
        {
            let rider = (*(*pVeh).m_pParentEntity).owner as *mut gentity_t;
            if rider.is_null() || (*rider).client.is_null() {
                riderPS = &mut (*(*pVeh).m_pParentEntity).client.unwrap().ps;
            } else {
                riderPS = &mut (*rider).client.unwrap().ps;
            }
            parentPS = &mut (*(*pVeh).m_pParentEntity).client.unwrap().ps;

            if (*pVeh).m_ulFlags & VEH_FLYING != 0 {
                (*pVeh).m_vOrientation[1] += (*pVeh).m_vAngularVelocity;
            } else if ((*pVeh).m_ulFlags & VEH_SLIDEBREAKING) != 0 ||	// No Angles Control While Out Of Control
                ((*pVeh).m_ulFlags & VEH_OUTOFCONTROL) != 0		// No Angles Control While Out Of Control
            {
                // Any ability to change orientation?
            } else if ((*pVeh).m_ulFlags & VEH_STRAFERAM) != 0			// No Angles Control While Strafe Ramming
            {
                if (*pVeh).m_fStrafeTime > 0.0f {
                    (*pVeh).m_fStrafeTime -= 1.0f;
                    (*pVeh).m_vOrientation[2] += if (*pVeh).m_fStrafeTime < (STRAFERAM_DURATION as c_float / 2.0f) {
                        -(STRAFERAM_ANGLE as c_float)
                    } else {
                        STRAFERAM_ANGLE as c_float
                    };
                } else if (*pVeh).m_fStrafeTime < 0.0f {
                    (*pVeh).m_fStrafeTime += 1.0f;
                    (*pVeh).m_vOrientation[2] += if (*pVeh).m_fStrafeTime > (-(STRAFERAM_DURATION as c_float) / 2.0f) {
                        STRAFERAM_ANGLE as c_float
                    } else {
                        -(STRAFERAM_ANGLE as c_float)
                    };
                }
            } else {
                // pVeh->m_vOrientation[YAW] = riderPS->viewangles[YAW];
            }
        }
    }

    /********************************************************************************/
    /*	END	Here is where make sure the vehicle is properly oriented.	END			*/
    /********************************************************************************/
}

#[cfg(feature = "qagame")]
pub fn AnimateVehicle(_pVeh: *mut Vehicle_t) {
    // Empty implementation in original
}

//rest of file is shared

#[cfg(not(feature = "jk2mp"))]
extern "C" {
    pub fn CG_ChangeWeapon(num: c_int);
}

#[cfg(not(feature = "jk2mp"))]
extern "C" {
    pub fn G_StartMatrixEffect(ent: *mut gentity_t, meFlags: c_int, length: c_int, timeScale: c_float, spinTime: c_int);
}

//NOTE NOTE NOTE NOTE NOTE NOTE
//I want to keep this function BG too, because it's fairly generic already, and it
//would be nice to have proper prediction of animations. -rww
// This function makes sure that the rider's in this vehicle are properly animated.
fn AnimateRiders(pVeh: *mut Vehicle_t) {
    let mut Anim: animNumber_t = 0; // BOTH_VS_IDLE
    let _fSpeedPercToMax: c_float;
    let mut iFlags: c_int = 0; // SETANIM_FLAG_NORMAL
    let mut iBlend: c_int = 300;
    let mut pilotPS: *mut playerState_t = std::ptr::null_mut();
    let mut parentPS: *mut playerState_t = std::ptr::null_mut();
    let curTime: c_int;

    unsafe {
        // Boarding animation.
        if (*pVeh).m_iBoarding != 0 {
            // We've just started moarding, set the amount of time it will take to finish moarding.
            if (*pVeh).m_iBoarding < 0 {
                let mut iAnimLen: c_int;

                // Boarding from left...
                if (*pVeh).m_iBoarding == -1 {
                    Anim = 0; // BOTH_VS_MOUNT_L
                } else if (*pVeh).m_iBoarding == -2 {
                    Anim = 0; // BOTH_VS_MOUNT_R
                } else if (*pVeh).m_iBoarding == -3 {
                    Anim = 0; // BOTH_VS_MOUNTJUMP_L
                } else if (*pVeh).m_iBoarding == 10 {
                    // VEH_MOUNT_THROW_LEFT
                    iBlend = 0;
                    Anim = 0; // BOTH_VS_MOUNTTHROW_R
                } else if (*pVeh).m_iBoarding == 11 {
                    // VEH_MOUNT_THROW_RIGHT
                    iBlend = 0;
                    Anim = 0; // BOTH_VS_MOUNTTHROW_L
                }

                // Set the delay time (which happens to be the time it takes for the animation to complete).
                // NOTE: Here I made it so the delay is actually 40% (0.4f) of the animation time.
                #[cfg(feature = "jk2mp")]
                {
                    // iAnimLen = BG_AnimLength( pVeh->m_pPilot->localAnimIndex, Anim ) * 0.4f;
                    // pVeh->m_iBoarding = BG_GetTime() + iAnimLen;
                }
                #[cfg(not(feature = "jk2mp"))]
                {
                    // iAnimLen = PM_AnimLength( pVeh->m_pPilot->client->clientInfo.animFileIndex, Anim );
                    // if (pVeh->m_iBoarding!=VEH_MOUNT_THROW_LEFT && pVeh->m_iBoarding!=VEH_MOUNT_THROW_RIGHT)
                    // {
                    //     pVeh->m_iBoarding = level.time + (iAnimLen*0.4f);
                    // }
                    // else
                    // {
                    //     pVeh->m_iBoarding = level.time + iAnimLen;
                    // }
                }

                // Set the animation, which won't be interrupted until it's completed.
                // TODO: But what if he's killed? Should the animation remain persistant???
                iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;

                #[cfg(feature = "jk2mp")]
                {
                    // BG_SetAnim(pVeh->m_pPilot->playerState, bgAllAnims[pVeh->m_pPilot->localAnimIndex].anims,
                    //     SETANIM_BOTH, Anim, iFlags, iBlend);
                }
                #[cfg(not(feature = "jk2mp"))]
                {
                    // NPC_SetAnim( pVeh->m_pPilot, SETANIM_BOTH, Anim, iFlags, iBlend );
                    // if (pVeh->m_pOldPilot)
                    // {
                    //     iAnimLen = PM_AnimLength( pVeh->m_pPilot->client->clientInfo.animFileIndex, BOTH_VS_MOUNTTHROWEE);
                    //     NPC_SetAnim( pVeh->m_pOldPilot, SETANIM_BOTH, BOTH_VS_MOUNTTHROWEE, iFlags, iBlend );
                    // }
                }
            }

            #[cfg(not(feature = "jk2mp"))]
            {
                // if (pVeh->m_pOldPilot && pVeh->m_pOldPilot->client->ps.torsoAnimTimer<=0)
                // {
                //     if (Q_irand(0, player->count)==0)
                //     {
                //         player->count++;
                //         player->lastEnemy = pVeh->m_pOldPilot;
                //         G_StartMatrixEffect(player, MEF_LOOK_AT_ENEMY|MEF_NO_RANGEVAR|MEF_NO_VERTBOB|MEF_NO_SPIN, 1000);
                //     }
                //
                //     gentity_t*	oldPilot = pVeh->m_pOldPilot;
                //     pVeh->m_pVehicleInfo->Eject(pVeh, pVeh->m_pOldPilot, qtrue);		// will set pointer to zero
                //
                //     // Kill Him
                //     //----------
                //     oldPilot->client->noRagTime = -1;	// no ragdoll for you
                //     G_Damage(oldPilot, pVeh->m_pPilot, pVeh->m_pPilot, pVeh->m_pPilot->currentAngles, pVeh->m_pPilot->currentOrigin, 1000, 0, MOD_CRUSH);
                //
                //     // Compute THe Throw Direction As Backwards From The Vehicle's Velocity
                //     //----------------------------------------------------------------------
                //     vec3_t		throwDir;
                //     VectorScale(pVeh->m_pParentEntity->client->ps.velocity, -1.0f, throwDir);
                //     VectorNormalize(throwDir);
                //     throwDir[2] += 0.3f;	// up a little
                //
                //     // Now Throw Him Out
                //     //-------------------
                //     G_Throw(oldPilot, throwDir, VectorLength(pVeh->m_pParentEntity->client->ps.velocity)/10.0f);
                //     NPC_SetAnim(oldPilot, SETANIM_BOTH, BOTH_DEATHBACKWARD1, SETANIM_FLAG_OVERRIDE, iBlend );
                // }
            }

            return;
        }

        #[cfg(feature = "jk2mp")]
        {
            if true {
                return;
            } // fixme
        }

        #[cfg(feature = "jk2mp")]
        {
            // pilotPS = pVeh->m_pPilot->playerState;
            // parentPS = pVeh->m_pPilot->playerState;
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            // pilotPS = &pVeh->m_pPilot->client->ps;
            // parentPS = &pVeh->m_pParentEntity->client->ps;
        }

        #[cfg(not(feature = "jk2mp"))]
        {
            curTime = level.time;
        }
        #[cfg(all(feature = "jk2mp", feature = "qagame"))]
        {
            curTime = level.time;
        }
        #[cfg(all(feature = "jk2mp", not(feature = "qagame")))]
        {
            //FIXME: pass in ucmd?  Not sure if this is reliable...
            // curTime = pm->cmd.serverTime;
            curTime = 0; // Stub
        }

        // Percentage of maximum speed relative to current speed.
        // fSpeedPercToMax = parentPS->speed / pVeh->m_pVehicleInfo->speedMax;

        // Rest of animation logic follows original structure...
        // Most of this is stubbed as it depends on playerState field access
    }
}

#[cfg(not(feature = "qagame"))]
extern "C" {
    pub fn AttachRidersGeneric(pVeh: *mut Vehicle_t);
}

pub fn G_SetSpeederVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    unsafe {
        #[cfg(feature = "qagame")]
        {
            // pVehInfo->AnimateVehicle = AnimateVehicle;
            // pVehInfo->AnimateRiders = AnimateRiders;
            // pVehInfo->Update = Update;
        }

        //shared
        // pVehInfo->ProcessMoveCommands = ProcessMoveCommands;
        // pVehInfo->ProcessOrientCommands = ProcessOrientCommands;

        #[cfg(not(feature = "qagame"))]
        {
            // pVehInfo->AttachRiders = AttachRidersGeneric;
        }
    }
}

// Following is only in game, not in namespace
#[cfg(feature = "jk2mp")]
// #include "../namespace_end.h"

#[cfg(feature = "qagame")]
extern "C" {
    pub fn G_AllocateVehicleObject(pVeh: *mut *mut Vehicle_t);
}

#[cfg(feature = "jk2mp")]
// #include "../namespace_begin.h"

// Create/Allocate a new Animal Vehicle (initializing it as well).
pub fn G_CreateSpeederNPC(pVeh: *mut *mut Vehicle_t, strType: *const c_char) {
    unsafe {
        #[cfg(feature = "jk2mp")]
        {
            #[cfg(feature = "qagame")]
            {
                //these will remain on entities on the client once allocated because the pointer is
                //never stomped. on the server, however, when an ent is freed, the entity struct is
                //memset to 0, so this memory would be lost..
                G_AllocateVehicleObject(pVeh);
            }
            #[cfg(not(feature = "qagame"))]
            {
                if (*pVeh).is_null() {
                    //only allocate a new one if we really have to
                    // (*pVeh) = (Vehicle_t *) BG_Alloc( sizeof(Vehicle_t) );
                }
            }
            core::ptr::write_bytes(*pVeh, 0, 1); // memset(*pVeh, 0, sizeof(Vehicle_t))
            // (*pVeh)->m_pVehicleInfo = &g_vehicleInfo[BG_VehicleGetIndex( strType )];
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            // Allocate the Vehicle.
            // (*pVeh) = (Vehicle_t *) gi.Malloc( sizeof(Vehicle_t), TAG_G_ALLOC, qtrue );
            // (*pVeh)->m_pVehicleInfo = &g_vehicleInfo[BG_VehicleGetIndex( strType )];
        }
    }
}

#[cfg(feature = "jk2mp")]
// #include "../namespace_end.h"
// get rid of all the crazy defs we added for this file
// #undef currentAngles
// #undef currentOrigin
// #undef mins
// #undef maxs
// #undef legsAnimTimer
// #undef torsoAnimTimer
// #undef bool
// #undef false
// #undef true
// #undef sqrtf
// #undef Q_flrand
// #undef MOD_EXPLOSIVE
