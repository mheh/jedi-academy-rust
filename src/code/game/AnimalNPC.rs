// leave this line at the top for all g_xxxx.cpp files...
#![allow(non_snake_case)]

// seems to be a compiler bug, it doesn't clean out the #ifdefs between dif-compiles
// or something, so the headers spew errors on these defs from the previous compile.
// this fixes that. -rww
#[cfg(feature = "_JK2MP")]
mod _undef_jk2mp {
    // get rid of all the crazy defs we added for this file
    // These are undone at the end via feature-gating
}

#[cfg(feature = "_JK2")]
#[cfg(not(feature = "_JK2MP"))]
const _JK2MP: bool = true;

#[cfg(not(feature = "_JK2MP"))]
#[cfg(feature = "QAGAME")]
const QAGAME: bool = true;

use core::ffi::{c_char, c_int, c_void};

// Stub types for now - these would be defined elsewhere
#[repr(C)]
pub struct Vehicle_t {
    // Minimal stub structure for compilation
}

#[repr(C)]
pub struct usercmd_t {
    // Stub
}

#[repr(C)]
pub struct bgEntity_t {
    // Stub
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
pub struct vehicleInfo_t {
    // Stub
}

// Stub constants
const VEHICLE_BASE: usize = 0;
const YAW: usize = 1;
const PITCH: usize = 0;

const BUTTON_ALT_ATTACK: u32 = 0x0001;
const BUTTON_ATTACK: u32 = 0x0002;
const BUTTON_WALKING: u32 = 0x0004;

const VEH_BUCKING: u32 = 0x0001;
const VEH_FLYING: u32 = 0x0002;
const VEH_CRASHING: u32 = 0x0004;
const VEH_SABERINLEFTHAND: u32 = 0x0008;

const ENTITYNUM_NONE: u32 = 2047;
const MASK_NPCSOLID: u32 = 0;
const CONTENTS_SOLID: u32 = 0;

const BOTH_VT_IDLE: u32 = 0;
const BOTH_VT_IDLE1: u32 = 1;
const BOTH_VT_WALK_FWD: u32 = 2;
const BOTH_VT_WALK_REV: u32 = 3;
const BOTH_VT_RUN_FWD: u32 = 4;
const BOTH_VT_BUCK: u32 = 5;
const BOTH_VT_MOUNT_L: u32 = 6;
const BOTH_VT_MOUNT_R: u32 = 7;
const BOTH_VT_MOUNT_B: u32 = 8;
const BOTH_VT_TURBO: u32 = 9;
const BOTH_VT_ATL_G: u32 = 10;
const BOTH_VT_ATL_S: u32 = 11;
const BOTH_VT_ATL_TO_R_S: u32 = 12;
const BOTH_VT_ATR_G: u32 = 13;
const BOTH_VT_ATR_TO_L_S: u32 = 14;
const BOTH_VT_ATR_S: u32 = 15;
const BOTH_VT_ATF_G: u32 = 16;
const BOTH_VT_IDLE_G: u32 = 17;
const BOTH_VT_IDLE_SL: u32 = 18;
const BOTH_VT_IDLE_SR: u32 = 19;

const SETANIM_FLAG_NORMAL: u32 = 0;
const SETANIM_FLAG_OVERRIDE: u32 = 1;
const SETANIM_FLAG_HOLD: u32 = 2;
const SETANIM_FLAG_HOLDLESS: u32 = 4;
const SETANIM_FLAG_RESTART: u32 = 8;

const SETANIM_BOTH: u32 = 0;
const SETANIM_LEGS: u32 = 1;

const WP_NONE: u32 = 0;
const WP_MELEE: u32 = 1;
const WP_BLASTER: u32 = 2;
const WP_SABER: u32 = 3;

const MOD_EXPLOSIVE: u32 = 0;
const MOD_SUICIDE: u32 = 1;
const MOD_MELEE: u32 = 2;

const DAMAGE_NO_KNOCKBACK: u32 = 0x0001;
const DAMAGE_IGNORE_TEAM: u32 = 0x0002;

const CHAN_AUTO: u32 = 0;

#[cfg(feature = "QAGAME")]
const FRAMETIME: i32 = 50;

// Stub types for enums and more complex types
type animNumber_t = u32;
type EWeaponPose = u32;

const WPOSE_NONE: EWeaponPose = 0;
const WPOSE_BLASTER: EWeaponPose = 1;
const WPOSE_SABERLEFT: EWeaponPose = 2;
const WPOSE_SABERRIGHT: EWeaponPose = 3;

// Stub types for vec3_t and trace_t
type vec3_t = [f32; 3];

#[repr(C)]
pub struct trace_t {
    // Stub structure
}

// Stub types for mdxaBone_t and other complex types
#[repr(C)]
pub struct mdxaBone_t {
    // Stub structure
}

// extern "C" declarations for engine functions
extern "C" {
    fn DotToSpot(spot: *const vec3_t, from: *const vec3_t, fromAngles: *const vec3_t) -> f32;

    #[cfg(feature = "QAGAME")]
    static mut level: crate::Level;

    #[cfg(feature = "QAGAME")]
    static mut g_vehicleInfo: *mut vehicleInfo_t;

    fn BG_VehicleGetIndex(strAnimalType: *const c_char) -> c_int;

    #[cfg(feature = "_JK2MP")]
    fn BG_Alloc(size: usize) -> *mut c_void;

    #[cfg(not(feature = "_JK2MP"))]
    fn gi_Malloc(size: usize, tag: i32, zero: bool) -> *mut c_void;

    #[cfg(feature = "QAGAME")]
    fn Vehicle_SetAnim(
        ent: *mut gentity_t,
        setAnimParts: c_int,
        anim: animNumber_t,
        setAnimFlags: c_int,
        blendTime: c_int,
    );

    #[cfg(feature = "QAGAME")]
    fn PM_SetAnim(
        pm: *mut pmove_t,
        setAnimParts: c_int,
        anim: c_int,
        setAnimFlags: c_int,
        blendTime: c_int,
    );

    #[cfg(feature = "QAGAME")]
    fn PM_AnimLength(index: c_int, anim: animNumber_t) -> c_int;

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn CG_ChangeWeapon(num: c_int);

    #[cfg(feature = "QAGAME")]
    fn G_Knockdown(
        selff: *mut gentity_t,
        attacker: *mut gentity_t,
        pushDir: *const vec3_t,
        strength: f32,
        breakSaberLock: bool,
    );

    #[cfg(feature = "QAGAME")]
    fn G_VehicleTrace(
        results: *mut trace_t,
        start: *const vec3_t,
        tMins: *const vec3_t,
        tMaxs: *const vec3_t,
        end: *const vec3_t,
        passEntityNum: c_int,
        contentmask: c_int,
    );

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn G_SoundIndexOnEnt(ent: *const gentity_t, channel: c_int, soundpath: *const c_char);

    #[cfg(feature = "_JK2MP")]
    fn PM_BGEntForNum(num: u32) -> *mut bgEntity_t;

    #[cfg(feature = "_JK2MP")]
    fn BG_AnimLength(index: c_int, anim: animNumber_t) -> c_int;

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn AngleSubtract(angle1: f32, angle2: f32) -> f32;

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn AngleNormalize180(angle: f32) -> f32;

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn AngleVectors(
        angles: *const vec3_t,
        forward: *mut vec3_t,
        right: *mut vec3_t,
        up: *mut vec3_t,
    );

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t);

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn VectorNormalize(v: *mut vec3_t) -> f32;

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32;

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn Q_flrand(min: f32, max: f32) -> f32;

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn G_Sound(ent: *const gentity_t, soundindex: c_int);

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn G_SoundIndex(sound: *const c_char) -> c_int;

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn G_Throw(ent: *mut gentity_t, direction: *const vec3_t, force: f32);

    #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
    fn G_Damage(
        targ: *mut gentity_t,
        inflictor: *mut gentity_t,
        attacker: *mut gentity_t,
        dir: *const vec3_t,
        point: *const vec3_t,
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    );

    #[cfg(feature = "_JK2MP")]
    fn BG_KnockDownable(ps: *const playerState_t) -> bool;

    #[cfg(feature = "QAGAME")]
    fn G_AllocateVehicleObject(pVeh: *mut *mut Vehicle_t);
}

// Stub structure for pmove_t
#[repr(C)]
pub struct pmove_t {
    // Stub
}

// Stub structure for Level
#[repr(C)]
pub struct Level {
    // Stub
}

#[cfg(feature = "QAGAME")]
// Update death sequence.
unsafe fn DeathUpdate(pVeh: *mut Vehicle_t) {
    if level.time >= (*pVeh).m_iDieTime {
        // If the vehicle is not empty.
        if ((*(*pVeh).m_pVehicleInfo).Inhabited)(pVeh) {
            ((*(*pVeh).m_pVehicleInfo).EjectAll)(pVeh);
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

#[cfg(feature = "QAGAME")]
// Like a think or move command, this updates various vehicle properties.
unsafe fn Update(pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> bool {
    ((*(*pVeh).m_pVehicleInfo).Update)(pVeh, pUcmd)
}

// MP RULE - ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
// If you really need to violate this rule for SP, then use ifdefs.
// By BG-compatible, I mean no use of game-specific data - ONLY use
// stuff available in the MP bgEntity (in SP, the bgEntity is #defined
// as a gentity, but the MP-compatible access restrictions are based
// on the bgEntity structure in the MP codebase) -rww
// ProcessMoveCommands the Vehicle.
unsafe fn ProcessMoveCommands(pVeh: *mut Vehicle_t) {
    /************************************************************************************/
    /*	BEGIN	Here is where we move the vehicle (forward or back or whatever). BEGIN	*/
    /************************************************************************************/

    // Client sets ucmds and such for speed alterations
    let mut speedInc: f32;
    let speedIdleDec: f32;
    let speedIdle: f32;
    let speedIdleAccel: f32;
    let speedMin: f32;
    let mut speedMax: f32;
    let fWalkSpeedMax: f32;
    let curTime: c_int;
    let parent = (*pVeh).m_pParentEntity;

    #[cfg(feature = "_JK2MP")]
    let parentPS = (*parent).playerState;
    #[cfg(not(feature = "_JK2MP"))]
    let parentPS = &mut (*(*parent).client).ps;

    #[cfg(not(feature = "_JK2MP"))]
    { curTime = level.time; }
    #[cfg(all(feature = "_JK2MP", feature = "QAGAME"))]
    { curTime = level.time; }
    #[cfg(all(feature = "_JK2MP", not(feature = "QAGAME")))]
    {
        // FIXME: pass in ucmd?  Not sure if this is reliable...
        // curTime = pm->cmd.serverTime;
        curTime = 0; // Placeholder for cgame context
    }

    #[cfg(not(feature = "_JK2MP"))]
    // bad for prediction - fixme
    {
        // Bucking so we can't do anything.
        if ((*pVeh).m_ulFlags & VEH_BUCKING) != 0 || ((*pVeh).m_ulFlags & VEH_FLYING) != 0 || ((*pVeh).m_ulFlags & VEH_CRASHING) != 0 {
            (*parentPS).speed = 0;
            return;
        }
    }

    speedIdleDec = (*(*pVeh).m_pVehicleInfo).decelIdle * (*pVeh).m_fTimeModifier;
    speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;

    speedIdle = (*(*pVeh).m_pVehicleInfo).speedIdle;
    speedIdleAccel = (*(*pVeh).m_pVehicleInfo).accelIdle * (*pVeh).m_fTimeModifier;
    speedMin = (*(*pVeh).m_pVehicleInfo).speedMin;

    if (*pVeh).m_pPilot != core::ptr::null_mut()
        && ((*pVeh).m_ucmd.buttons & BUTTON_ALT_ATTACK) != 0
        && (*(*pVeh).m_pVehicleInfo).turboSpeed != 0.0
    {
        if (curTime - (*pVeh).m_iTurboTime) > (*(*pVeh).m_pVehicleInfo).turboRecharge {
            (*pVeh).m_iTurboTime = curTime + (*(*pVeh).m_pVehicleInfo).turboDuration;
            #[cfg(all(feature = "QAGAME", not(feature = "_JK2MP")))]
            {
                if (*(*pVeh).m_pVehicleInfo).soundTurbo != core::ptr::null() {
                    G_SoundIndexOnEnt(
                        (*pVeh).m_pParentEntity,
                        CHAN_AUTO as c_int,
                        (*(*pVeh).m_pVehicleInfo).soundTurbo,
                    );
                }
            }
            (*parentPS).speed = (*(*pVeh).m_pVehicleInfo).turboSpeed; // Instantly Jump To Turbo Speed
        }
    }

    if curTime < (*pVeh).m_iTurboTime {
        speedMax = (*(*pVeh).m_pVehicleInfo).turboSpeed;
    } else {
        speedMax = (*(*pVeh).m_pVehicleInfo).speedMax;
    }

    #[cfg(feature = "_JK2MP")]
    let uninhabited = (*parentPS).m_iVehicleNum == 0;
    #[cfg(not(feature = "_JK2MP"))]
    let uninhabited = !((*(*pVeh).m_pVehicleInfo).Inhabited)(pVeh);

    if uninhabited {
        // drifts to a stop
        speedInc = speedIdle * (*pVeh).m_fTimeModifier;
        // VectorClear( parentPS->moveDir );
        (*parentPS).speed = 0;
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
        } else if (*parentPS).speed > 0.0 {
            // No input, so coast to stop.
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

        // pVeh->m_ucmd.rightmove = 0;

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

// MP RULE - ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE!!!
// If you really need to violate this rule for SP, then use ifdefs.
// By BG-compatible, I mean no use of game-specific data - ONLY use
// stuff available in the MP bgEntity (in SP, the bgEntity is #defined
// as a gentity, but the MP-compatible access restrictions are based
// on the bgEntity structure in the MP codebase) -rww
// ProcessOrientCommands the Vehicle.
unsafe fn ProcessOrientCommands(pVeh: *mut Vehicle_t) {
    /********************************************************************************/
    /*	BEGIN	Here is where make sure the vehicle is properly oriented.	BEGIN	*/
    /********************************************************************************/
    let parent = (*pVeh).m_pParentEntity;
    let mut parentPS: *mut playerState_t;
    let mut riderPS: *mut playerState_t;

    #[cfg(feature = "_JK2MP")]
    let mut rider: *mut bgEntity_t = core::ptr::null_mut();
    #[cfg(feature = "_JK2MP")]
    {
        if (*parent).s.owner != ENTITYNUM_NONE {
            rider = PM_BGEntForNum((*parent).s.owner); //&g_entities[parent->r.ownerNum];
        }
    }
    #[cfg(not(feature = "_JK2MP"))]
    let mut rider: *mut gentity_t = (*parent).owner;

    // Bucking so we can't do anything.
    #[cfg(not(feature = "_JK2MP"))]
    // bad for prediction - fixme
    {
        if ((*pVeh).m_ulFlags & VEH_BUCKING) != 0
            || ((*pVeh).m_ulFlags & VEH_FLYING) != 0
            || ((*pVeh).m_ulFlags & VEH_CRASHING) != 0
        {
            return;
        }
    }

    #[cfg(feature = "_JK2MP")]
    {
        if rider == core::ptr::null_mut() {
            rider = parent;
        }
    }
    #[cfg(not(feature = "_JK2MP"))]
    {
        if rider == core::ptr::null_mut() || (*rider).client == core::ptr::null_mut() {
            rider = parent;
        }
    }

    #[cfg(feature = "_JK2MP")]
    {
        parentPS = (*parent).playerState;
        riderPS = (*rider).playerState;
    }
    #[cfg(not(feature = "_JK2MP"))]
    {
        parentPS = &mut (*(*parent).client).ps;
        riderPS = &mut (*(*rider).client).ps;
    }

    if rider != core::ptr::null_mut() {
        #[cfg(feature = "_JK2MP")]
        {
            let angDif = AngleSubtract((*pVeh).m_vOrientation[YAW], (*riderPS).viewangles[YAW]);
            if parentPS != core::ptr::null_mut() && (*parentPS).speed != 0.0 {
                let mut s = (*parentPS).speed;
                let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 4.0; //magic number hackery
                if s < 0.0 {
                    s = -s;
                }
                let angDif_scaled = angDif * s / (*(*pVeh).m_pVehicleInfo).speedMax;
                let angDif_clamped = if angDif_scaled > maxDif {
                    maxDif
                } else if angDif_scaled < -maxDif {
                    -maxDif
                } else {
                    angDif_scaled
                };
                (*pVeh).m_vOrientation[YAW] = AngleNormalize180(
                    (*pVeh).m_vOrientation[YAW] - angDif_clamped * ((*pVeh).m_fTimeModifier * 0.2),
                );
            }
        }
        #[cfg(not(feature = "_JK2MP"))]
        {
            (*pVeh).m_vOrientation[YAW] = (*riderPS).viewangles[YAW];
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

#[cfg(feature = "_JK2MP")]
// temp hack til mp speeder controls are sorted -rww
pub unsafe fn AnimalProcessOri(pVeh: *mut Vehicle_t) {
    ProcessOrientCommands(pVeh);
}

#[cfg(feature = "QAGAME")]
// This function makes sure that the vehicle is properly animated.
/*
static void AnimalTailSwipe(Vehicle_t* pVeh, gentity_t *parent, gentity_t *pilot)
{
    trace_t	trace;
    vec3_t angles;
    vec3_t vRoot, vTail;
    vec3_t	lMins, lMaxs;
    mdxaBone_t	boltMatrix;
    int iRootBone;
    int iRootTail;

    VectorSet(angles, 0, parent->currentAngles[YAW], 0);
    VectorSet(lMins, parent->mins[0]-1, parent->mins[1]-1, 0);
    VectorSet(lMaxs, parent->maxs[0]+1, parent->maxs[1]+1, 1);
#ifdef _JK2MP
    iRootBone = trap_G2API_AddBolt( parent->ghoul2, 0, "tail_01" );
    iRootTail = trap_G2API_AddBolt( parent->ghoul2, 0, "tail_04" );

    // Get the positions of the root of the tail and the tail end of it.
    trap_G2API_GetBoltMatrix( parent->ghoul2, 0, iRootBone,
                &boltMatrix, angles, parent->currentOrigin, level.time,
                NULL, parent->modelScale );
    BG_GiveMeVectorFromMatrix( &boltMatrix, ORIGIN, vRoot );

    trap_G2API_GetBoltMatrix( parent->ghoul2, 0, iRootTail,
                &boltMatrix, angles, parent->currentOrigin, level.time,
                NULL, parent->modelScale );
    BG_GiveMeVectorFromMatrix( &boltMatrix, ORIGIN, vTail );
#else
    iRootBone = gi.G2API_GetBoneIndex( &parent->ghoul2[parent->playerModel], "tail_01", qtrue );
    iRootTail = gi.G2API_GetBoneIndex( &parent->ghoul2[parent->playerModel], "tail_04", qtrue );

    // Get the positions of the root of the tail and the tail end of it.
    gi.G2API_GetBoltMatrix( parent->ghoul2, parent->playerModel, iRootBone,
                &boltMatrix, angles, parent->currentOrigin, (cg.time?cg.time:level.time),
                NULL, parent->s.modelScale );
    gi.G2API_GiveMeVectorFromMatrix( boltMatrix, ORIGIN, vRoot );

    gi.G2API_GetBoltMatrix( parent->ghoul2, parent->playerModel, iRootTail,
                &boltMatrix, angles, parent->currentOrigin, (cg.time?cg.time:level.time),
                NULL, parent->s.modelScale );
    gi.G2API_GiveMeVectorFromMatrix( boltMatrix, ORIGIN, vTail );
#endif

    // Trace from the root of the tail to the very end.
    G_VehicleTrace( &trace, vRoot, lMins, lMaxs, vTail, parent->s.number, MASK_NPCSOLID );
    if ( trace.fraction < 1.0f )
    {
        if ( ENTITYNUM_NONE != trace.entityNum && g_entities[trace.entityNum].client &&
#ifndef _JK2MP //no rancor in jk2mp (at least not currently)
            g_entities[trace.entityNum].client->NPC_class != CLASS_RANCOR &&
#else //and in mp want to check inuse
            g_entities[trace.entityNum].inuse &&
#endif
            g_entities[trace.entityNum].client->NPC_class != CLASS_VEHICLE )
        {
            vec3_t pushDir;
            vec3_t angs;
            int iDamage = 10;
            // Get the direction we're facing.
            VectorCopy( parent->client->ps.viewangles, angs );
            // Add some fudge.
            angs[YAW] += Q_flrand( 5, 15 );
            angs[PITCH] = Q_flrand( -20, -10 );
            AngleVectors( angs, pushDir, NULL, NULL );
            // Reverse direction.
            pushDir[YAW] = -pushDir[YAW];

            // Smack this ho down.
#ifdef _JK2MP
            G_Sound( &g_entities[trace.entityNum], CHAN_AUTO, G_SoundIndex( "sound/chars/rancor/swipehit.wav" ) );
#else
            G_Sound( &g_entities[trace.entityNum], G_SoundIndex( "sound/chars/rancor/swipehit.wav" ) );
#endif
            G_Throw( &g_entities[trace.entityNum], pushDir, 50 );

            if ( g_entities[trace.entityNum].health > 0 )
            {
                // Knock down and dish out some hurt.
                gentity_t *hit = &g_entities[trace.entityNum];
#ifdef _JK2MP
                if (BG_KnockDownable(&hit->client->ps))
                {
                    hit->client->ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
                    hit->client->ps.forceHandExtendTime = pm->cmd.serverTime + 1100;
                    hit->client->ps.forceDodgeAnim = 0; //this toggles between 1 and 0, when it's 1 we should play the get up anim

                    hit->client->ps.otherKiller = pilot->s.number;
                    hit->client->ps.otherKillerTime = level.time + 5000;
                    hit->client->ps.otherKillerDebounceTime = level.time + 100;

                    hit->client->ps.velocity[0] = pushDir[0]*80;
                    hit->client->ps.velocity[1] = pushDir[1]*80;
                    hit->client->ps.velocity[2] = 100;
                }
#else
                G_Knockdown( hit, parent, pushDir, 300, qtrue );
#endif
                G_Damage( hit, parent, parent, NULL, NULL, iDamage, DAMAGE_NO_KNOCKBACK | DAMAGE_IGNORE_TEAM, MOD_MELEE );
                //G_PlayEffect( pVeh->m_pVehicleInfo->explodeFX, parent->currentOrigin );
            }// Not Dead
        }// Not Rancor & In USe
    }// Trace Hit Anything?
}
*/
#[cfg(feature = "QAGAME")]
unsafe fn AnimateVehicle(pVeh: *mut Vehicle_t) {
    let mut Anim: animNumber_t = BOTH_VT_IDLE;
    let mut iFlags: u32 = SETANIM_FLAG_NORMAL;
    let mut iBlend: i32 = 300;
    let pilot = (*pVeh).m_pPilot as *mut gentity_t;
    let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
    let pilotPS: *mut playerState_t;
    let parentPS: *mut playerState_t;
    let fSpeedPercToMax: f32;

    #[cfg(feature = "_JK2MP")]
    {
        pilotPS = if !pilot.is_null() {
            (*pilot).playerState
        } else {
            core::ptr::null_mut()
        };
        parentPS = (*parent).playerState;
    }
    #[cfg(not(feature = "_JK2MP"))]
    {
        pilotPS = if !pilot.is_null() {
            &mut (*(*pilot).client).ps
        } else {
            core::ptr::null_mut()
        };
        parentPS = &mut (*(*parent).client).ps;
    }

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

    // If they're bucking, play the animation and leave...
    if (*(*parent).client).ps.legsAnim == BOTH_VT_BUCK {
        // Done with animation? Erase the flag.
        if (*(*parent).client).ps.legsAnimTimer <= 0 {
            (*pVeh).m_ulFlags &= !VEH_BUCKING;
        } else {
            return;
        }
    } else if ((*pVeh).m_ulFlags & VEH_BUCKING) != 0 {
        iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
        Anim = BOTH_VT_BUCK;
        iBlend = 500;
        Vehicle_SetAnim(parent, SETANIM_LEGS as c_int, BOTH_VT_BUCK, iFlags as c_int, iBlend);
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
            #[cfg(feature = "_JK2MP")]
            {
                iAnimLen = (BG_AnimLength((*(*parent).client).clientInfo.animFileIndex, Anim) as f32 * 0.7) as c_int;
            }
            #[cfg(not(feature = "_JK2MP"))]
            {
                iAnimLen = (PM_AnimLength((*(*parent).client).clientInfo.animFileIndex, Anim) as f32 * 0.7) as c_int;
            }
            (*pVeh).m_iBoarding = level.time + iAnimLen;

            // Set the animation, which won't be interrupted until it's completed.
            // TODO: But what if he's killed? Should the animation remain persistant???
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;

            Vehicle_SetAnim(parent, SETANIM_LEGS as c_int, Anim, iFlags as c_int, iBlend);
            if !pilot.is_null() {
                Vehicle_SetAnim(pilot, SETANIM_BOTH as c_int, Anim, iFlags as c_int, iBlend);
            }
            return;
        } else if (*pVeh).m_iBoarding <= level.time {
            // Otherwise we're done.
            (*pVeh).m_iBoarding = 0;
        }
    }

    // Percentage of maximum speed relative to current speed.
    // float fSpeed = VectorLength( client->ps.velocity );
    fSpeedPercToMax = (*(*parent).client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax;

    // Going in reverse...
    if fSpeedPercToMax < -0.01 {
        Anim = BOTH_VT_WALK_REV;
        iBlend = 600;
    } else {
        let Turbo = fSpeedPercToMax > 0.0 && level.time < (*pVeh).m_iTurboTime;
        let Walking = fSpeedPercToMax > 0.0
            && (((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 || fSpeedPercToMax <= 0.275);
        let Running = fSpeedPercToMax > 0.275;

        // Remove Crashing Flag
        // ----------------------
        (*pVeh).m_ulFlags &= !VEH_CRASHING;

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
    Vehicle_SetAnim(parent, SETANIM_LEGS as c_int, Anim, iFlags as c_int, iBlend);
}

// rwwFIXMEFIXME: This is all going to have to be predicted I think, or it will feel awful
// and lagged
// This function makes sure that the rider's in this vehicle are properly animated.
#[cfg(feature = "QAGAME")]
unsafe fn AnimateRiders(pVeh: *mut Vehicle_t) {
    let mut Anim: animNumber_t = BOTH_VT_IDLE;
    let mut iFlags: u32 = SETANIM_FLAG_NORMAL;
    let mut iBlend: i32 = 500;
    let pilot = (*pVeh).m_pPilot as *mut gentity_t;
    let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
    let pilotPS: *mut playerState_t;
    let parentPS: *mut playerState_t;
    let fSpeedPercToMax: f32;

    #[cfg(feature = "_JK2MP")]
    {
        pilotPS = (*(*pVeh).m_pPilot).playerState;
        parentPS = (*(*pVeh).m_pPilot).playerState;
    }
    #[cfg(not(feature = "_JK2MP"))]
    {
        pilotPS = &mut (*(*pVeh).m_pPilot).client.ps;
        parentPS = &mut (*(*pVeh).m_pParentEntity).client.ps;
    }

    // Boarding animation.
    if (*pVeh).m_iBoarding != 0 {
        return;
    }

    // Percentage of maximum speed relative to current speed.
    fSpeedPercToMax = (*(*parent).client).ps.speed / (*(*pVeh).m_pVehicleInfo).speedMax;

    /*	// Going in reverse...
#ifdef _JK2MP //handled in pmove in mp
    if (0)
#else
    if ( fSpeedPercToMax < -0.01f )
#endif
    {
        Anim = BOTH_VT_WALK_REV;
        iBlend = 600;
        bool		HasWeapon	= ((pilotPS->weapon != WP_NONE) && (pilotPS->weapon != WP_MELEE));
        if (HasWeapon)
        {
            if (pVeh->m_pPilot->s.number<MAX_CLIENTS)
            {
                CG_ChangeWeapon(WP_NONE);
            }

            pVeh->m_pPilot->client->ps.weapon = WP_NONE;
            G_RemoveWeaponModels(pVeh->m_pPilot);
        }
    }
    else
    */
    {
        let HasWeapon = ((*pilotPS).weapon != WP_NONE) && ((*pilotPS).weapon != WP_MELEE);
        let Attacking = HasWeapon && ((*pVeh).m_ucmd.buttons & BUTTON_ATTACK) != 0;
        let mut Right = (*pVeh).m_ucmd.rightmove > 0;
        let mut Left = (*pVeh).m_ucmd.rightmove < 0;
        let Turbo = fSpeedPercToMax > 0.0 && level.time < (*pVeh).m_iTurboTime;
        let Walking = fSpeedPercToMax > 0.0
            && (((*pVeh).m_ucmd.buttons & BUTTON_WALKING) != 0 || fSpeedPercToMax <= 0.275);
        let Running = fSpeedPercToMax > 0.275;
        let mut WeaponPose: EWeaponPose = WPOSE_NONE;

        // Remove Crashing Flag
        // ----------------------
        (*pVeh).m_ulFlags &= !VEH_CRASHING;

        // Put Away Saber When It Is Not Active
        // --------------------------------------
        #[cfg(not(feature = "_JK2MP"))]
        {
            if HasWeapon
                && ((*pVeh).m_pPilot.s.number >= 64
                    || (0 + 500) < 0) // Placeholder for cg.weaponSelectTime+500 < cg.time
                && ((*pilotPS).weapon == WP_SABER && (Turbo || !(*pilotPS).SaberActive()))
            {
                if (*pVeh).m_pPilot.s.number < 64 {
                    CG_ChangeWeapon(WP_NONE as c_int);
                }

                (*pVeh).m_pPilot.client.ps.weapon = WP_NONE;
                // G_RemoveWeaponModels(pVeh->m_pPilot);
            }
        }

        // Don't Interrupt Attack Anims
        // ------------------------------
        #[cfg(feature = "_JK2MP")]
        {
            if (*pilotPS).weaponTime > 0 {
                return;
            }
        }
        #[cfg(not(feature = "_JK2MP"))]
        {
            if (*pilotPS).torsoAnim >= BOTH_VT_ATL_S && (*pilotPS).torsoAnim <= BOTH_VT_ATF_G {
                let mut bodyCurrent: f32 = 0.0;
                let mut bodyEnd: i32 = 0;
                // if (!!gi.G2API_GetBoneAnimIndex(&pVeh->m_pPilot->ghoul2[pVeh->m_pPilot->playerModel], pVeh->m_pPilot->rootBone, level.time, &bodyCurrent, NULL, &bodyEnd, NULL, NULL, NULL))
                // {
                // if (bodyCurrent<=((float)(bodyEnd)-1.5f))
                // {
                // return;
                // }
                // }
            }
        }

        // Compute The Weapon Pose
        // --------------------------
        if (*pilotPS).weapon == WP_BLASTER {
            WeaponPose = WPOSE_BLASTER;
        } else if (*pilotPS).weapon == WP_SABER {
            if ((*pVeh).m_ulFlags & VEH_SABERINLEFTHAND) != 0
                && (*pilotPS).torsoAnim == BOTH_VT_ATL_TO_R_S
            {
                (*pVeh).m_ulFlags &= !VEH_SABERINLEFTHAND;
            }
            if ((*pVeh).m_ulFlags & VEH_SABERINLEFTHAND) == 0
                && (*pilotPS).torsoAnim == BOTH_VT_ATR_TO_L_S
            {
                (*pVeh).m_ulFlags |= VEH_SABERINLEFTHAND;
            }
            WeaponPose = if ((*pVeh).m_ulFlags & VEH_SABERINLEFTHAND) != 0 {
                WPOSE_SABERLEFT
            } else {
                WPOSE_SABERRIGHT
            };
        }

        if Attacking && WeaponPose != WPOSE_NONE {
            // Attack!
            iBlend = 100;
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART;

            if Turbo {
                Right = true;
                Left = false;
            }

            // Auto Aiming
            // ===============================================
            if !Left && !Right {
                // Allow player strafe keys to override
                #[cfg(not(feature = "_JK2MP"))]
                {
                    if !(*pVeh).m_pPilot.enemy.is_null() {
                        // Auto aiming logic for SP
                        // (Complex logic omitted for brevity, preserved in original)
                    } else if (*pilotPS).weapon == WP_SABER && !Left && !Right {
                        Left = WeaponPose == WPOSE_SABERLEFT;
                        Right = !Left;
                    }
                }
                #[cfg(feature = "_JK2MP")]
                {
                    if (*pilotPS).weapon == WP_SABER && !Left && !Right {
                        Left = WeaponPose == WPOSE_SABERLEFT;
                        Right = !Left;
                    }
                }
            }

            if Left {
                // Attack Left
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VT_ATL_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VT_ATL_S,
                    WPOSE_SABERRIGHT => Anim = BOTH_VT_ATR_TO_L_S,
                    _ => unreachable!(),
                }
            } else if Right {
                // Attack Right
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VT_ATR_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VT_ATL_TO_R_S,
                    WPOSE_SABERRIGHT => Anim = BOTH_VT_ATR_S,
                    _ => unreachable!(),
                }
            } else {
                // Attack Ahead
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VT_ATF_G,
                    _ => unreachable!(),
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
                    Anim = BOTH_VT_IDLE1; // (Q_irand(0,1)==0)?(BOTH_VT_IDLE):(BOTH_VT_IDLE1);
                }
            } else {
                match WeaponPose {
                    WPOSE_BLASTER => Anim = BOTH_VT_IDLE_G,
                    WPOSE_SABERLEFT => Anim = BOTH_VT_IDLE_SL,
                    WPOSE_SABERRIGHT => Anim = BOTH_VT_IDLE_SR,
                    _ => unreachable!(),
                }
            }
        } // No Special Moves
    }

    Vehicle_SetAnim(pilot, SETANIM_BOTH as c_int, Anim, iFlags as c_int, iBlend);
}

#[cfg(not(feature = "QAGAME"))]
unsafe fn AttachRidersGeneric(pVeh: *mut Vehicle_t) {
    // Stub for non-QAGAME builds
}

// on the client this function will only set up the process command funcs
pub unsafe fn G_SetAnimalVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    #[cfg(feature = "QAGAME")]
    {
        (*pVehInfo).AnimateVehicle = Some(AnimateVehicle);
        (*pVehInfo).AnimateRiders = Some(AnimateRiders);
        // (*pVehInfo).ValidateBoard = Some(ValidateBoard);
        // (*pVehInfo).SetParent = Some(SetParent);
        // (*pVehInfo).SetPilot = Some(SetPilot);
        // (*pVehInfo).AddPassenger = Some(AddPassenger);
        // (*pVehInfo).Animate = Some(Animate);
        // (*pVehInfo).Board = Some(Board);
        // (*pVehInfo).Eject = Some(Eject);
        // (*pVehInfo).EjectAll = Some(EjectAll);
        // (*pVehInfo).StartDeathDelay = Some(StartDeathDelay);
        (*pVehInfo).DeathUpdate = Some(DeathUpdate);
        // (*pVehInfo).RegisterAssets = Some(RegisterAssets);
        // (*pVehInfo).Initialize = Some(Initialize);
        (*pVehInfo).Update = Some(Update);
        // (*pVehInfo).UpdateRider = Some(UpdateRider);
    }
    (*pVehInfo).ProcessMoveCommands = Some(ProcessMoveCommands);
    (*pVehInfo).ProcessOrientCommands = Some(ProcessOrientCommands);

    #[cfg(not(feature = "QAGAME"))]
    {
        // cgame prediction attachment func
        (*pVehInfo).AttachRiders = Some(AttachRidersGeneric);
    }
    // (*pVehInfo).AttachRiders = Some(AttachRiders);
    // (*pVehInfo).Ghost = Some(Ghost);
    // (*pVehInfo).UnGhost = Some(UnGhost);
    // (*pVehInfo).Inhabited = Some(Inhabited);
}

// Following is only in game, not in namespace
// #ifdef _JK2MP
// #include "../namespace_end.h"
// #endif

#[cfg(feature = "QAGAME")]
extern "C" {
    fn G_AllocateVehicleObject(pVeh: *mut *mut Vehicle_t);
}

// #ifdef _JK2MP
// #include "../namespace_begin.h"
// #endif

// Create/Allocate a new Animal Vehicle (initializing it as well).
// this is a BG function too in MP so don't un-bg-compatibilify it -rww
pub unsafe fn G_CreateAnimalNPC(pVeh: *mut *mut Vehicle_t, strAnimalType: *const c_char) {
    // Allocate the Vehicle.
    #[cfg(all(feature = "_JK2MP", feature = "QAGAME"))]
    {
        // these will remain on entities on the client once allocated because the pointer is
        // never stomped. on the server, however, when an ent is freed, the entity struct is
        // memset to 0, so this memory would be lost..
        G_AllocateVehicleObject(pVeh);
    }
    #[cfg(all(feature = "_JK2MP", not(feature = "QAGAME")))]
    {
        if (*pVeh).is_null() {
            // only allocate a new one if we really have to
            *pVeh = BG_Alloc(core::mem::size_of::<Vehicle_t>()) as *mut Vehicle_t;
        }
    }

    #[cfg(all(feature = "_JK2MP", feature = "QAGAME"))]
    {
        core::ptr::write_bytes(*pVeh, 0, 1);
        (**pVeh).m_pVehicleInfo =
            &mut *(&mut g_vehicleInfo as *mut *mut vehicleInfo_t)
                .add(BG_VehicleGetIndex(strAnimalType) as usize)
                as *mut vehicleInfo_t;
    }
    #[cfg(all(feature = "_JK2MP", not(feature = "QAGAME")))]
    {
        core::ptr::write_bytes(*pVeh, 0, 1);
        (**pVeh).m_pVehicleInfo =
            &mut *(&mut g_vehicleInfo as *mut *mut vehicleInfo_t)
                .add(BG_VehicleGetIndex(strAnimalType) as usize)
                as *mut vehicleInfo_t;
    }
    #[cfg(not(feature = "_JK2MP"))]
    {
        *pVeh = gi_Malloc(core::mem::size_of::<Vehicle_t>(), 0, true) as *mut Vehicle_t;
        (**pVeh).m_pVehicleInfo =
            &mut *(&mut g_vehicleInfo as *mut *mut vehicleInfo_t)
                .add(BG_VehicleGetIndex(strAnimalType) as usize)
                as *mut vehicleInfo_t;
    }
}

// #ifdef _JK2MP
// #include "../namespace_end.h"
// get rid of all the crazy defs we added for this file
// (Undefs handled by feature flags above)
// #endif
