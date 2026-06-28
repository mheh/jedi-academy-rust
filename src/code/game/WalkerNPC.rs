// leave this line at the top for all g_xxxx.cpp files...
// g_headers.h equivalent declarations

// seems to be a compiler bug, it doesn't clean out the #ifdefs between dif-compiles
// or something, so the headers spew errors on these defs from the previous compile.
// this fixes that. -rww
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Forward declarations and type stubs for dependencies
// These would be defined in linked modules/headers

#[repr(C)]
pub struct Vehicle_t {
    // Stub: full definition in vehicle types
    pub m_pParentEntity: *mut bgEntity_t,
    pub m_pVehicleInfo: *mut vehicleInfo_t,
    pub m_iBoarding: c_int,
    pub m_fTimeModifier: f32,
    pub m_ucmd: usercmd_t,
    pub m_vOrientation: [f32; 3],  // YAW, PITCH, ROLL
    pub m_vPrevOrientation: [f32; 3],
    pub m_iArmor: c_int,
}

#[repr(C)]
pub struct vehicleInfo_t {
    pub decelIdle: f32,
    pub speedMax: f32,
    pub speedIdle: f32,
    pub accelIdle: f32,
    pub speedMin: f32,
    pub acceleration: f32,
    pub turningSpeed: f32,
    pub turnWhenStopped: c_int,
    pub malfunctionArmorLevel: c_int,
    pub strafePerc: f32,
    pub RegisterAssets: Option<extern "C" fn(*mut Vehicle_t)>,
    pub Board: Option<extern "C" fn(*mut Vehicle_t, *mut bgEntity_t) -> c_int>,
    pub AnimateVehicle: Option<extern "C" fn(*mut Vehicle_t)>,
    pub ProcessMoveCommands: Option<extern "C" fn(*mut Vehicle_t)>,
    pub ProcessOrientCommands: Option<extern "C" fn(*mut Vehicle_t)>,
    pub AttachRiders: Option<extern "C" fn(*mut Vehicle_t)>,
    pub Inhabited: Option<extern "C" fn(*mut Vehicle_t) -> c_int>,
}

#[repr(C)]
pub struct bgEntity_t {
    // Stub: full definition in bg_public
    pub s: entityState_t,
    pub client: *mut gclient_t,
    pub owner: *mut bgEntity_t,
    pub playerState: *mut playerState_t,
    pub NPC: c_int,
    pub health: c_int,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub owner: c_int,
    pub eType: c_int,
}

#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
}

#[repr(C)]
pub struct playerState_t {
    pub moveDir: [f32; 3],
    pub speed: f32,
    pub groundEntityNum: c_int,
    pub velocity: [f32; 3],
    pub viewangles: [f32; 3],
    pub m_iVehicleNum: c_int,
}

#[repr(C)]
pub struct usercmd_t {
    pub forwardmove: c_int,
    pub upmove: c_int,
    pub rightmove: c_int,
    pub buttons: c_int,
}

#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub client: *mut gclient_t,
    pub owner: *mut gentity_t,
    pub health: c_int,
}

#[repr(C)]
pub struct trace_t {
    // Stub: trace result
}

#[repr(C)]
pub struct animNumber_t(pub c_int);

// Constants
const ENTITYNUM_NONE: c_int = -1;
const BUTTON_WALKING: c_int = 0x00000008;
const MAX_CLIENTS: c_int = 64;
const YAW: usize = 0;
const PITCH: usize = 1;
const ROLL: usize = 2;
const SETANIM_FLAG_NORMAL: c_int = 0;
const SETANIM_FLAG_OVERRIDE: c_int = 0x04;
const SETANIM_FLAG_HOLD: c_int = 0x08;
const SETANIM_FLAG_RESTART: c_int = 0x01;
const BOTH_STAND1: animNumber_t = animNumber_t(1);
const BOTH_STAND2: animNumber_t = animNumber_t(2);
const BOTH_WALK1: animNumber_t = animNumber_t(3);
const BOTH_RUN1: animNumber_t = animNumber_t(4);
const BOTH_WALKBACK1: animNumber_t = animNumber_t(5);

extern "C" {
    pub static mut g_vehicleInfo: [vehicleInfo_t; 1];  // Stub array
    pub static mut level: level_t;

    pub fn DotToSpot(spot: *const f32, from: *const f32, fromAngles: *const f32) -> f32;
    pub fn PM_SetAnim(pm: *mut pmove_t, setAnimParts: c_int, anim: c_int, setAnimFlags: c_int, blendTime: c_int);
    pub fn PM_AnimLength(index: c_int, anim: animNumber_t) -> c_int;
    pub fn Vehicle_SetAnim(ent: *mut gentity_t, setAnimParts: c_int, anim: animNumber_t, setAnimFlags: c_int, iBlend: c_int);
    pub fn G_Knockdown(this: *mut gentity_t, attacker: *mut gentity_t, pushDir: *const f32, strength: f32, breakSaberLock: c_int);
    pub fn G_VehicleTrace(results: *mut trace_t, start: *const f32, tMins: *const f32, tMaxs: *const f32, end: *const f32, passEntityNum: c_int, contentmask: c_int);
    pub fn RegisterItem(item: *mut c_void);
    pub fn BG_FindItemForWeapon(weapon: c_int) -> *mut c_void;
    pub fn G_AllocateVehicleObject(pVeh: *mut *mut Vehicle_t);
    pub fn BG_Alloc(size: usize) -> *mut c_void;
    pub fn BG_VehicleGetIndex(strAnimalType: *const c_char) -> c_int;
    pub fn PM_BGEntForNum(num: c_int) -> *mut bgEntity_t;
    pub fn AttachRidersGeneric(pVeh: *mut Vehicle_t);
    pub fn VectorClear(v: *mut f32);
    pub fn VectorLength(v: *const f32) -> f32;
}

#[repr(C)]
pub struct level_t {
    pub time: c_int,
}

#[repr(C)]
pub struct pmove_t {
    // Stub
}

// Local constants
const WP_TURRET: c_int = 13;  // Stub weapon index
const VEHICLE_BASE: usize = 0;
const BG_VehicleNum: c_int = 0;

#[cfg(feature = "qagame")]
extern "C" {
    pub fn g_speederControlScheme() -> *mut c_void;
}

#[cfg(feature = "qagame")]
static mut RegisterAssets: Option<extern "C" fn(*mut Vehicle_t)> = None;

// RegisterAssets implementation
#[cfg(feature = "qagame")]
unsafe extern "C" fn RegisterAssets_impl(pVeh: *mut Vehicle_t) {
    // atst uses turret weapon
    #[cfg(feature = "mp")]
    {
        let item = BG_FindItemForWeapon(WP_TURRET);
        RegisterItem(item);
    }

    // call the standard RegisterAssets now
    if let Some(func) = (*g_vehicleInfo[VEHICLE_BASE].RegisterAssets) {
        func(pVeh);
    }
}

// Like a think or move command, this updates various vehicle properties.
// /*
// static bool Update( Vehicle_t *pVeh, const usercmd_t *pUcmd )
// {
// 	return g_vehicleInfo[VEHICLE_BASE].Update( pVeh, pUcmd );
// }
// */

// Board this Vehicle (get on). The first entity to board an empty vehicle becomes the Pilot.
#[cfg(feature = "qagame")]
unsafe extern "C" fn Board(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> c_int {
    if (*g_vehicleInfo[VEHICLE_BASE].Board).is_none() {
        return 0;
    }

    if (*(*g_vehicleInfo[VEHICLE_BASE].Board).unwrap())(pVeh, pEnt) == 0 {
        return 0;
    }

    // Set the board wait time (they won't be able to do anything, including getting off, for this amount of time).
    (*pVeh).m_iBoarding = level.time + 1500;

    1
}

// MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
// If you really need to violate this rule for SP, then use ifdefs.
// By BG-compatible, I mean no use of game-specific data - ONLY use
// stuff available in the MP bgEntity (in SP, the bgEntity is #defined
// as a gentity, but the MP-compatible access restrictions are based
// on the bgEntity structure in the MP codebase) -rww
// ProcessMoveCommands the Vehicle.
unsafe extern "C" fn ProcessMoveCommands(pVeh: *mut Vehicle_t) {
    /************************************************************************************/
    /*	BEGIN	Here is where we move the vehicle (forward or back or whatever). BEGIN	*/
    /************************************************************************************/

    // Client sets ucmds and such for speed alterations
    let speedIdleDec: f32;
    let speedMax: f32;
    let speedIdle: f32;
    let speedIdleAccel: f32;
    let speedMin: f32;
    let speedInc: f32;
    let fWalkSpeedMax: f32;
    let parent = (*pVeh).m_pParentEntity;

    let parentPS: *mut playerState_t;

    #[cfg(feature = "mp")]
    {
        parentPS = (*parent).playerState;
    }
    #[cfg(not(feature = "mp"))]
    {
        parentPS = &mut (*(*parent).client).ps;
    }

    speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;
    speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;

    speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle;
    speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
    speedMin = (*(*pVeh).m_pVehicleInfo).speedMin;

    #[cfg(feature = "mp")]
    let inhabited = (*parentPS).m_iVehicleNum != 0;
    #[cfg(not(feature = "mp"))]
    let inhabited = (*(*pVeh).m_pVehicleInfo).Inhabited.is_some() &&
                    (*(*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) != 0;

    if !inhabited {
        // drifts to a stop
        speedInc = speedIdle * (*pVeh).m_fTimeModifier;
        VectorClear((*parentPS).moveDir.as_mut_ptr());
        // m_ucmd.forwardmove = 127;
        (*parentPS).speed = 0.0f;
    } else {
        speedInc = (*(*pVeh).m_pVehicleInfo).acceleration * (*pVeh).m_fTimeModifier;
    }

    if (*parentPS).speed != 0.0f || (*parentPS).groundEntityNum == ENTITYNUM_NONE ||
        (*pVeh).m_ucmd.forwardmove != 0 || (*pVeh).m_ucmd.upmove > 0 {
        if (*pVeh).m_ucmd.forwardmove > 0 && speedInc != 0.0f {
            (*parentPS).speed += speedInc;
        } else if (*pVeh).m_ucmd.forwardmove < 0 {
            if (*parentPS).speed > speedIdle {
                (*parentPS).speed -= speedInc;
            } else if (*parentPS).speed > speedMin {
                (*parentPS).speed -= speedIdleDec;
            }
        }
        // No input, so coast to stop.
        else if (*parentPS).speed > 0.0f {
            (*parentPS).speed -= speedIdleDec;
            if (*parentPS).speed < 0.0f {
                (*parentPS).speed = 0.0f;
            }
        } else if (*parentPS).speed < 0.0f {
            (*parentPS).speed += speedIdleDec;
            if (*parentPS).speed > 0.0f {
                (*parentPS).speed = 0.0f;
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

        // /*if ( !pVeh->m_pVehicleInfo->strafePerc
        // 	|| (!g_speederControlScheme->value && !parent->s.number) )
        // {//if in a strafe-capable vehicle, clear strafing unless using alternate control scheme
        // 	pVeh->m_ucmd.rightmove = 0;
        // }*/
    }

    fWalkSpeedMax = speedMax * 0.275f;
    if (*pVeh).m_ucmd.buttons & BUTTON_WALKING != 0 && (*parentPS).speed > fWalkSpeedMax {
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

#[cfg(feature = "mp")]
extern "C" {
    pub fn FighterYawAdjust(pVeh: *mut Vehicle_t, riderPS: *mut playerState_t, parentPS: *mut playerState_t);  // FighterNPC.c
    pub fn FighterPitchAdjust(pVeh: *mut Vehicle_t, riderPS: *mut playerState_t, parentPS: *mut playerState_t);  // FighterNPC.c
}

// MP RULE - ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
// If you really need to violate this rule for SP, then use ifdefs.
// By BG-compatible, I mean no use of game-specific data - ONLY use
// stuff available in the MP bgEntity (in SP, the bgEntity is #defined
// as a gentity, but the MP-compatible access restrictions are based
// on the bgEntity structure in the MP codebase) -rww
// ProcessOrientCommands the Vehicle.
unsafe extern "C" fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    /********************************************************************************/
    /*	BEGIN	Here is where make sure the vehicle is properly oriented.	BEGIN	*/
    /********************************************************************************/
    let speed: f32;
    let parent = (*pVeh).m_pParentEntity;
    let parentPS: *mut playerState_t;
    let riderPS: *mut playerState_t;

    #[cfg(feature = "mp")]
    let mut rider: *mut bgEntity_t = core::ptr::null_mut();
    #[cfg(feature = "mp")]
    {
        if (*parent).s.owner != ENTITYNUM_NONE {
            rider = PM_BGEntForNum((*parent).s.owner);
        }
    }
    #[cfg(not(feature = "mp"))]
    let mut rider = (*parent).owner;

    #[cfg(feature = "mp")]
    {
        if rider.is_null() {
            rider = parent;
        }
    }
    #[cfg(not(feature = "mp"))]
    {
        if rider.is_null() || (*rider).client.is_null() {
            rider = parent;
        }
    }

    #[cfg(feature = "mp")]
    {
        parentPS = (*parent).playerState;
        riderPS = (*rider).playerState;
    }
    #[cfg(not(feature = "mp"))]
    {
        parentPS = &mut (*(*parent).client).ps;
        riderPS = &mut (*(*rider).client).ps;
    }

    speed = VectorLength((*parentPS).velocity.as_ptr());

    // If the player is the rider...
    if (*rider).s.number < MAX_CLIENTS {
        // FIXME: use the vehicle's turning stat in this calc
        #[cfg(feature = "mp")]
        {
            FighterYawAdjust(pVeh, riderPS, parentPS);
            // FighterPitchAdjust(pVeh, riderPS, parentPS);
            (*pVeh).m_vOrientation[PITCH] = (*riderPS).viewangles[PITCH];
        }
        #[cfg(not(feature = "mp"))]
        {
            (*pVeh).m_vOrientation[YAW] = (*riderPS).viewangles[YAW];
            (*pVeh).m_vOrientation[PITCH] = (*riderPS).viewangles[PITCH];
        }
    } else {
        let mut turnSpeed = (*(*pVeh).m_pVehicleInfo).turningSpeed;
        if (*(*pVeh).m_pVehicleInfo).turnWhenStopped == 0 && (*parentPS).speed == 0.0f {
            // can't turn when not moving
            // FIXME: or ramp up to max turnSpeed?
            turnSpeed = 0.0f;
        }
        #[cfg(feature = "mp")]
        {
            if (*rider).s.eType == 3 {  // ET_NPC = 3
                // help NPCs out some
                turnSpeed *= 2.0f;
                if (*parentPS).speed > 200.0f {
                    turnSpeed += turnSpeed * (*parentPS).speed / 200.0f * 0.05f;
                }
            }
        }
        #[cfg(not(feature = "mp"))]
        {
            if rider.is_null() || (*rider).NPC != 0 {
                // help NPCs out some
                turnSpeed *= 2.0f;
                if (*(*parent).client).ps.speed > 200.0f {
                    turnSpeed += turnSpeed * (*(*parent).client).ps.speed / 200.0f * 0.05f;
                }
            }
        }
        turnSpeed *= (*pVeh).m_fTimeModifier;

        // default control scheme: strafing turns, mouselook aims
        if (*pVeh).m_ucmd.rightmove < 0 {
            (*pVeh).m_vOrientation[YAW] += turnSpeed;
        } else if (*pVeh).m_ucmd.rightmove > 0 {
            (*pVeh).m_vOrientation[YAW] -= turnSpeed;
        }

        if (*(*pVeh).m_pVehicleInfo).malfunctionArmorLevel != 0 && (*pVeh).m_iArmor <= (*(*pVeh).m_pVehicleInfo).malfunctionArmorLevel {
            // damaged badly
        }
    }

    /********************************************************************************/
    /*	END	Here is where make sure the vehicle is properly oriented.	END			*/
    /********************************************************************************/
}

// back to our game-only functions
#[cfg(feature = "qagame")]
unsafe extern "C" fn AnimateVehicle(pVeh: *mut Vehicle_t) {
    let mut Anim = BOTH_STAND1;
    let mut iFlags = SETANIM_FLAG_NORMAL;
    let mut iBlend = 300;
    let parent = pVeh as *mut gentity_t;  // Cast from Vehicle_t to gentity_t
    let fSpeedPercToMax: f32;

    // We're dead (boarding is reused here so I don't have to make another variable :-).
    if (*parent).health <= 0 {
        if (*pVeh).m_iBoarding != -999 {
            // Animate the death just once!
            (*pVeh).m_iBoarding = -999;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;

            // FIXME! Why do you keep repeating over and over!!?!?!? Bastard!
            // Vehicle_SetAnim( parent, SETANIM_LEGS, BOTH_VT_DEATH1, iFlags, iBlend );
        }
        return;
    }

    // Following is redundant to g_vehicles.c
    // if ( pVeh->m_iBoarding )
    // {
    // 	//we have no boarding anim
    // 	if (pVeh->m_iBoarding < level.time)
    // 	{ //we are on now
    // 		pVeh->m_iBoarding = 0;
    // 	}
    // 	else
    // 	{
    // 		return;
    // 	}
    // }

    // Percentage of maximum speed relative to current speed.
    // float fSpeed = VectorLength( client->ps.velocity );
    fSpeedPercToMax = (*(*parent).client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax;

    // If we're moving...
    if fSpeedPercToMax > 0.0f {
        // fSpeedPercToMax >= 0.85f )
        let fYawDelta: f32;

        iBlend = 300;
        iFlags = SETANIM_FLAG_OVERRIDE;
        fYawDelta = (*pVeh).m_vPrevOrientation[YAW] - (*pVeh).m_vOrientation[YAW];

        // NOTE: Mikes suggestion for fixing the stuttering walk (left/right) is to maintain the
        // current frame between animations. I have no clue how to do this and have to work on other
        // stuff so good luck to him :-p AReis

        // If we're walking (or our speed is less than .275%)...
        if ((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 || fSpeedPercToMax < 0.275f {
            // Make them lean if we're turning.
            // /*if ( fYawDelta < -0.0001f )
            // {
            // 	Anim = BOTH_VT_WALK_FWD_L;
            // }
            // else if ( fYawDelta > 0.0001 )
            // {
            // 	Anim = BOTH_VT_WALK_FWD_R;
            // }
            // else*/
            {
                Anim = BOTH_WALK1;
            }
        }
        // otherwise we're running.
        else {
            // Make them lean if we're turning.
            // /*if ( fYawDelta < -0.0001f )
            // {
            // 	Anim = BOTH_VT_RUN_FWD_L;
            // }
            // else if ( fYawDelta > 0.0001 )
            // {
            // 	Anim = BOTH_VT_RUN_FWD_R;
            // }
            // else*/
            {
                Anim = BOTH_RUN1;
            }
        }
    } else {
        // Going in reverse...
        if fSpeedPercToMax < -0.018f {
            iFlags = SETANIM_FLAG_NORMAL;
            Anim = BOTH_WALKBACK1;
            iBlend = 500;
        } else {
            // int iChance = Q_irand( 0, 20000 );

            // Every once in a while buck or do a different idle...
            iFlags = SETANIM_FLAG_NORMAL | SETANIM_FLAG_RESTART | SETANIM_FLAG_HOLD;
            iBlend = 600;
            #[cfg(feature = "mp")]
            {
                if (*(*parent).client).ps.m_iVehicleNum != 0 {
                    // occupado
                    Anim = BOTH_STAND1;
                } else {
                    // wide open for you, baby
                    Anim = BOTH_STAND2;
                }
            }
            #[cfg(not(feature = "mp"))]
            {
                if (*(*pVeh).m_pVehicleInfo).Inhabited.is_some() &&
                   (*(*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) != 0 {
                    // occupado
                    Anim = BOTH_STAND1;
                } else {
                    // wide open for you, baby
                    Anim = BOTH_STAND2;
                }
            }
        }
    }

    Vehicle_SetAnim(parent, 0, Anim, iFlags, iBlend);  // SETANIM_LEGS = 0
}

// rwwFIXMEFIXME: This is all going to have to be predicted I think, or it will feel awful
// and lagged

#[cfg(not(feature = "qagame"))]
extern "C" {
    pub fn AttachRidersGeneric(pVeh: *mut Vehicle_t);
}

// on the client this function will only set up the process command funcs
pub unsafe extern "C" fn G_SetWalkerVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    #[cfg(feature = "qagame")]
    {
        (*pVehInfo).AnimateVehicle = Some(AnimateVehicle);
        // (*pVehInfo).AnimateRiders = Some(AnimateRiders);
        // (*pVehInfo).ValidateBoard = Some(ValidateBoard);
        // (*pVehInfo).SetParent = Some(SetParent);
        // (*pVehInfo).SetPilot = Some(SetPilot);
        // (*pVehInfo).AddPassenger = Some(AddPassenger);
        // (*pVehInfo).Animate = Some(Animate);
        (*pVehInfo).Board = Some(Board);
        // (*pVehInfo).Eject = Some(Eject);
        // (*pVehInfo).EjectAll = Some(EjectAll);
        // (*pVehInfo).StartDeathDelay = Some(StartDeathDelay);
        // (*pVehInfo).DeathUpdate = Some(DeathUpdate);
        (*pVehInfo).RegisterAssets = Some(RegisterAssets_impl);
        // (*pVehInfo).Initialize = Some(Initialize);
        // (*pVehInfo).Update = Some(Update);
        // (*pVehInfo).UpdateRider = Some(UpdateRider);
    }
    (*pVehInfo).ProcessMoveCommands = Some(ProcessMoveCommands);
    (*pVehInfo).ProcessOrientCommands = Some(ProcessOrientCommands);

    #[cfg(not(feature = "qagame"))]
    {
        (*pVehInfo).AttachRiders = Some(AttachRidersGeneric);  // cgame prediction attachment func
    }
    // (*pVehInfo).AttachRiders = Some(AttachRiders);
    // (*pVehInfo).Ghost = Some(Ghost);
    // (*pVehInfo).UnGhost = Some(UnGhost);
    // (*pVehInfo).Inhabited = Some(Inhabited);
}

// Create/Allocate a new Animal Vehicle (initializing it as well).
// this is a BG function too in MP so don't un-bg-compatibilify it -rww
pub unsafe extern "C" fn G_CreateWalkerNPC(pVeh: *mut *mut Vehicle_t, strAnimalType: *const c_char) {
    // Allocate the Vehicle.
    #[cfg(feature = "mp")]
    {
        #[cfg(feature = "qagame")]
        {
            // these will remain on entities on the client once allocated because the pointer is
            // never stomped. on the server, however, when an ent is freed, the entity struct is
            // memset to 0, so this memory would be lost..
            G_AllocateVehicleObject(pVeh);
        }
        #[cfg(not(feature = "qagame"))]
        {
            if (*pVeh).is_null() {
                // only allocate a new one if we really have to
                *pVeh = BG_Alloc(core::mem::size_of::<Vehicle_t>()) as *mut Vehicle_t;
            }
        }
        core::ptr::write_bytes(*pVeh, 0, 1);  // memset(*pVeh, 0, sizeof(Vehicle_t))
        (**pVeh).m_pVehicleInfo = &mut g_vehicleInfo[BG_VehicleGetIndex(strAnimalType) as usize];
    }
    #[cfg(not(feature = "mp"))]
    {
        // Using a stub for gi.Malloc since it's not available in BG
        // In SP: gi.Malloc( sizeof(Vehicle_t), TAG_G_ALLOC, qtrue )
        *pVeh = BG_Alloc(core::mem::size_of::<Vehicle_t>()) as *mut Vehicle_t;
        (**pVeh).m_pVehicleInfo = &mut g_vehicleInfo[BG_VehicleGetIndex(strAnimalType) as usize];
    }
}
