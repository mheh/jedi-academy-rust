//! Port of `g_vehicles.c` — the server-side vehicle subsystem (flight/ride physics,
//! surface impact damage, board/eject, turrets). Landed incrementally: only the leaves
//! whose full dep-set is already ported. The bulk of the file — the Update/Board/Eject
//! loop, turret aiming, surface destruction — is not yet ported, blocked on the vehicle
//! update loop, `PM_*`, the vehicle anim engine, and `trap_Trace`/`NPC_SetSurfaceOnOff`.
//!
//! Landed so far: the pure `G_ShipSurfaceForSurfName` surface-name predicate (oracle)
//! plus the two impact-damage-flag setters `G_SetVehDamageFlags` /
//! `G_VehicleSetDamageLocFlags` (No-oracle — they mutate `gentity`/`Vehicle_t` state).

#![allow(non_snake_case)] // C function names (`G_ShipSurfaceForSurfName`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro/enum names kept verbatim

use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::c_ulong;
use core::ptr::{addr_of, null_mut, write_bytes};

use crate::codemp::game::anims::{
    animNumber_t, BOTH_JUMP1, BOTH_ROLL_B, BOTH_ROLL_F, BOTH_ROLL_L, BOTH_ROLL_R, BOTH_STAND2,
    BOTH_VS_DISMOUNT_L, BOTH_VS_DISMOUNT_R, BOTH_VS_IDLE,
};
use crate::codemp::game::b_public_h::{BS_CINEMATIC, NPCAI_CUSTOM_GRAVITY};
use crate::codemp::game::bg_misc::vectoyaw;
use crate::codemp::game::bg_panimate::{
    bgAllAnims, BG_AnimLength, BG_SetAnim, BG_SetLegsAnimTimer, BG_SetTorsoAnimTimer,
};
use crate::codemp::game::bg_pmove::{BG_UnrestrainedPitchRoll, PM_BGEntForNum};
use crate::codemp::game::bg_public::{
    bgEntity_t, BG_GiveMeVectorFromMatrix, DEFAULT_MAXS_2, DEFAULT_MINS_2, EF_NODRAW, EV_JUMP,
    EV_ROLL, JUMP_VELOCITY, MOD_SUICIDE, MOD_UNKNOWN, MOD_VEH_EXPLOSION, SETANIM_BOTH,
    SETANIM_FLAG_HOLD, SETANIM_FLAG_HOLDLESS, SETANIM_FLAG_NORMAL, SETANIM_FLAG_OVERRIDE,
    STAT_ARMOR, STAT_HEALTH, STAT_MAX_HEALTH, STAT_WEAPONS, WEAPON_READY,
};
use crate::codemp::game::bg_vehicleLoad::AttachRidersGeneric;
use crate::codemp::game::bg_vehicles_h::{
    vehicleInfo_t, Vehicle_t, MAX_VEHICLE_EXHAUSTS, MAX_VEHICLE_MUZZLES, MAX_VEHICLE_TURRETS,
    MAX_VEHICLE_WEAPONS, SHIPSURF_BACK, SHIPSURF_BROKEN_A, SHIPSURF_BROKEN_B, SHIPSURF_BROKEN_C,
    SHIPSURF_BROKEN_D, SHIPSURF_BROKEN_E, SHIPSURF_BROKEN_F, SHIPSURF_BROKEN_G,
    SHIPSURF_DAMAGE_FRONT_HEAVY, SHIPSURF_DAMAGE_FRONT_LIGHT, SHIPSURF_FRONT, SHIPSURF_LEFT,
    SHIPSURF_RIGHT, VEH_BUCKING, VEH_EJECT_BOTTOM, VEH_EJECT_FRONT, VEH_EJECT_LEFT, VEH_EJECT_REAR,
    VEH_EJECT_RIGHT, VEH_EJECT_TOP, VEH_FLYING, VEH_GEARSOPEN, VEH_MOUNT_THROW_LEFT,
    VEH_MOUNT_THROW_RIGHT, VEH_WEAPON_BASE, VH_ANIMAL, VH_FIGHTER, VH_SPEEDER, VH_WALKER,
};
use crate::codemp::game::bg_weapons_h::{WP_BLASTER, WP_NONE};
use crate::codemp::game::g_client::SetClientViewAngle;
use crate::codemp::game::g_combat::{G_Damage, G_DamageFromKiller, G_RadiusDamage};
use crate::codemp::game::g_local::{
    gentity_t, CON_CONNECTED, DAMAGE_NO_ARMOR, DAMAGE_NO_HIT_LOC, DAMAGE_NO_PROTECTION,
    DAMAGE_NO_SELF_PROTECTION, FL_UNDYING, FL_VEH_BOARDING, FRAMETIME,
};
use crate::codemp::game::g_main::{g_entities, g_gravity, level};
use crate::codemp::game::g_public_h::{SVF_NOCLIENT, TID_CHAN_VOICE};
use crate::codemp::game::g_utils::{
    G_AddEvent, G_EffectIndex, G_EntitySound, G_FreeEntity, G_MuteSound, G_PlayEffectID,
    G_SetAngles, G_SetOrigin, G_Sound, G_SoundIndex,
};
use crate::codemp::game::g_vehicleTurret::VEH_TurretThink;
use crate::codemp::game::npc::NPC_SetAnim;
use crate::codemp::game::npc_spawn::NPC_Spawn_Do;
use crate::codemp::game::npc_utils::NPC_SetSurfaceOnOff;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleSubtract, AngleVectors, DotProduct, VectorAdd, VectorClear,
    VectorCopy, VectorLength, VectorMA, VectorNormalize, VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::Q_strncmp;
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, qboolean, trace_t, usercmd_t, vec3_t, BUTTON_TALK, BUTTON_USE, BUTTON_USE_HOLDABLE,
    CHAN_AUTO, CHAN_VOICE, ENTITYNUM_NONE, MAX_CLIENTS, NEGATIVE_Y, ORIGIN, PITCH, QFALSE, QTRUE,
    ROLL, YAW,
};
use crate::codemp::game::surfaceflags_h::{CONTENTS_BODY, CONTENTS_SOLID};
use crate::codemp::game::teams_h::CLASS_VEHICLE;
use crate::trap;

/// `int G_ShipSurfaceForSurfName( const char *surfaceName )` (g_vehicles.c:2650) —
/// maps a model surface name to its `SHIPSURF_*` impact-damage region, or -1.
///
/// # Safety
/// `surfaceName` must be null or a NUL-terminated C string.
pub unsafe fn G_ShipSurfaceForSurfName(surfaceName: *const c_char) -> c_int {
    if surfaceName.is_null() {
        return -1;
    }
    if Q_strncmp(c"nose".as_ptr(), surfaceName, 4) == 0
        || Q_strncmp(c"f_gear".as_ptr(), surfaceName, 6) == 0
        || Q_strncmp(c"glass".as_ptr(), surfaceName, 5) == 0
    {
        return SHIPSURF_FRONT;
    }
    if Q_strncmp(c"body".as_ptr(), surfaceName, 4) == 0 {
        return SHIPSURF_BACK;
    }
    if Q_strncmp(c"r_wing1".as_ptr(), surfaceName, 7) == 0
        || Q_strncmp(c"r_wing2".as_ptr(), surfaceName, 7) == 0
        || Q_strncmp(c"r_gear".as_ptr(), surfaceName, 6) == 0
    {
        return SHIPSURF_RIGHT;
    }
    if Q_strncmp(c"l_wing1".as_ptr(), surfaceName, 7) == 0
        || Q_strncmp(c"l_wing2".as_ptr(), surfaceName, 7) == 0
        || Q_strncmp(c"l_gear".as_ptr(), surfaceName, 6) == 0
    {
        return SHIPSURF_LEFT;
    }
    -1
}

/// `void G_SetVehDamageFlags( gentity_t *veh, int shipSurf, int damageLevel )`
/// (g_vehicles.c:2681) — sets the cgame-visible `brokenLimbs` damage bits for a ship
/// surface at the given `damageLevel` (0 none, 1 light, 2 heavy, 3 destroyed), and on
/// the rear surface mirrors that to the carried droid unit.
///
/// # Safety
/// `veh` must be a valid `gentity_t*` whose `client` is non-null.
pub unsafe fn G_SetVehDamageFlags(veh: *mut gentity_t, shipSurf: c_int, damageLevel: c_int) {
    let mut dmgFlag: c_int;
    match damageLevel {
        3 => {
            //destroyed
            //add both flags so cgame side knows this surf is GONE
            //add heavy
            dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
            (*(*veh).client).ps.brokenLimbs |= 1 << dmgFlag;
            //add light
            dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
            (*(*veh).client).ps.brokenLimbs |= 1 << dmgFlag;
            //copy down
            (*veh).s.brokenLimbs = (*(*veh).client).ps.brokenLimbs;
            //check droid
            if shipSurf == SHIPSURF_BACK {
                //destroy the droid if we have one
                if !(*veh).m_pVehicle.is_null() && !(*(*veh).m_pVehicle).m_pDroidUnit.is_null() {
                    //we have one
                    let droidEnt = (*(*veh).m_pVehicle).m_pDroidUnit as *mut gentity_t;
                    if !droidEnt.is_null()
                        && (((*droidEnt).flags & FL_UNDYING) != 0 || (*droidEnt).health > 0)
                    {
                        //boom
                        //make it vulnerable
                        (*droidEnt).flags &= !FL_UNDYING;
                        //blow it up
                        G_Damage(
                            droidEnt,
                            (*veh).enemy,
                            (*veh).enemy,
                            null_mut(),
                            null_mut(),
                            99999,
                            0,
                            MOD_UNKNOWN,
                        );
                    }
                }
            }
        }
        2 => {
            //heavy only
            dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
            (*(*veh).client).ps.brokenLimbs |= 1 << dmgFlag;
            //remove light
            dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
            (*(*veh).client).ps.brokenLimbs &= !(1 << dmgFlag);
            //copy down
            (*veh).s.brokenLimbs = (*(*veh).client).ps.brokenLimbs;
            //check droid
            if shipSurf == SHIPSURF_BACK {
                //make the droid vulnerable if we have one
                if !(*veh).m_pVehicle.is_null() && !(*(*veh).m_pVehicle).m_pDroidUnit.is_null() {
                    //we have one
                    let droidEnt = (*(*veh).m_pVehicle).m_pDroidUnit as *mut gentity_t;
                    if !droidEnt.is_null() && ((*droidEnt).flags & FL_UNDYING) != 0 {
                        //make it vulnerab;e
                        (*droidEnt).flags &= !FL_UNDYING;
                    }
                }
            }
        }
        1 => {
            //light only
            //add light
            dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
            (*(*veh).client).ps.brokenLimbs |= 1 << dmgFlag;
            //remove heavy (shouldn't have to do this, but...
            dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
            (*(*veh).client).ps.brokenLimbs &= !(1 << dmgFlag);
            //copy down
            (*veh).s.brokenLimbs = (*(*veh).client).ps.brokenLimbs;
        }
        _ => {
            //case 0://no damage
            //default:
            //remove heavy
            dmgFlag = SHIPSURF_DAMAGE_FRONT_HEAVY + (shipSurf - SHIPSURF_FRONT);
            (*(*veh).client).ps.brokenLimbs &= !(1 << dmgFlag);
            //remove light
            dmgFlag = SHIPSURF_DAMAGE_FRONT_LIGHT + (shipSurf - SHIPSURF_FRONT);
            (*(*veh).client).ps.brokenLimbs &= !(1 << dmgFlag);
            //copy down
            (*veh).s.brokenLimbs = (*(*veh).client).ps.brokenLimbs;
        }
    }
}

/// `void G_VehicleSetDamageLocFlags( gentity_t *veh, int impactDir, int deathPoint )`
/// (g_vehicles.c:2761) — given accumulated `locationDamage[impactDir]`, classifies the
/// surface as destroyed/heavy/light against the vehicle's per-direction health and
/// malfunction-armour thresholds, then sets the corresponding damage flags.
///
/// Note the original shadows the `deathPoint` parameter with an inner local of the same
/// name (the parameter is unused); the port keeps the C's inner-local semantics verbatim
/// and marks the parameter `_deathPoint`.
///
/// # Safety
/// `veh` must be a valid `gentity_t*`; when `client` is non-null its
/// `m_pVehicle`/`m_pVehicleInfo` chain must be valid.
pub unsafe fn G_VehicleSetDamageLocFlags(
    veh: *mut gentity_t,
    impactDir: c_int,
    _deathPoint: c_int,
) {
    if (*veh).client.is_null() {
        return;
    } else {
        let deathPoint: c_int;
        let heavyDamagePoint: c_int;
        let lightDamagePoint: c_int;
        match impactDir {
            SHIPSURF_FRONT => {
                deathPoint = (*(*(*veh).m_pVehicle).m_pVehicleInfo).health_front;
            }
            SHIPSURF_BACK => {
                deathPoint = (*(*(*veh).m_pVehicle).m_pVehicleInfo).health_back;
            }
            SHIPSURF_RIGHT => {
                deathPoint = (*(*(*veh).m_pVehicle).m_pVehicleInfo).health_right;
            }
            SHIPSURF_LEFT => {
                deathPoint = (*(*(*veh).m_pVehicle).m_pVehicleInfo).health_left;
            }
            _ => {
                return;
            }
        }
        if !(*veh).m_pVehicle.is_null()
            && !(*(*veh).m_pVehicle).m_pVehicleInfo.is_null()
            && (*(*(*veh).m_pVehicle).m_pVehicleInfo).malfunctionArmorLevel != 0
            && (*(*(*veh).m_pVehicle).m_pVehicleInfo).armor != 0
        {
            let mut perc = (*(*(*veh).m_pVehicle).m_pVehicleInfo).malfunctionArmorLevel as f32
                / (*(*(*veh).m_pVehicle).m_pVehicleInfo).armor as f32;
            if perc > 0.99f32 {
                perc = 0.99f32;
            }
            lightDamagePoint = (deathPoint as f64 * perc as f64 * 0.25f32 as f64).ceil() as c_int;
            heavyDamagePoint = (deathPoint as f64 * perc as f64).ceil() as c_int;
        } else {
            heavyDamagePoint = (deathPoint as f64 * 0.66f32 as f64).ceil() as c_int;
            lightDamagePoint = (deathPoint as f64 * 0.14f32 as f64).ceil() as c_int;
        }

        if (*veh).locationDamage[impactDir as usize] >= deathPoint {
            //destroyed
            G_SetVehDamageFlags(veh, impactDir, 3);
        } else if (*veh).locationDamage[impactDir as usize] <= lightDamagePoint {
            //light only
            G_SetVehDamageFlags(veh, impactDir, 1);
        } else if (*veh).locationDamage[impactDir as usize] <= heavyDamagePoint {
            //heavy only
            G_SetVehDamageFlags(veh, impactDir, 2);
        }
    }
}

/// `void Vehicle_SetAnim( gentity_t *ent, int setAnimParts, int anim, int setAnimFlags, int iBlend )`
/// (g_vehicles.c) — sets the vehicle entity's animation through `BG_SetAnim` using its
/// local anim set, then mirrors the resulting `legsAnim` into the entity state.
///
/// # Safety
/// `ent` must be a valid `gentity_t*` whose `client` is non-null and whose
/// `localAnimIndex` indexes a loaded `bgAllAnims` set.
pub unsafe fn Vehicle_SetAnim(
    ent: *mut gentity_t,
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
    iBlend: c_int,
) {
    debug_assert!(!(*ent).client.is_null());
    BG_SetAnim(
        &mut (*(*ent).client).ps,
        (*addr_of!(bgAllAnims))[(*ent).localAnimIndex as usize].anims,
        setAnimParts,
        anim,
        setAnimFlags,
        iBlend,
    );
    (*ent).s.legsAnim = (*(*ent).client).ps.legsAnim;
}

/// `bool Initialize( Vehicle_t *pVeh )` (g_vehicles.c:1335) — resets a vehicle to its
/// spawned state: armor/shield/health from its `vehicleInfo`, max ammo on every weapon and
/// turret, custom gravity, passenger slots cleared, all tag/muzzle indices reset, default
/// blaster weapon, and the landed (idle) animation. MP (`_JK2MP`/`QAGAME`) path taken.
///
/// `m_vOrientation` is a `*mut vec_t` in this port (the SP-shared layout), so the C array
/// writes `VectorClear`/`[YAW]` become pointer writes. The `iMuzzle[i]` index sharing the
/// turret loop index `i` is a faithful copy of the C (a known quirk).
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` whose `m_pParentEntity` is a valid `gentity_t*` with
/// a non-null `client`/`NPC` and whose `m_pVehicleInfo`/`m_vOrientation` are valid.
pub unsafe extern "C" fn Initialize(pVeh: *mut Vehicle_t) -> qboolean {
    let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
    // C declares `int i = 0;` here; the initial 0 is immediately overwritten by the first
    // `for ( i = 0; ... )` below, so the Rust binding is initialised at that first use.
    let mut i: c_int;

    if parent.is_null() || (*parent).client.is_null() {
        return QFALSE;
    }

    (*(*parent).client).ps.m_iVehicleNum = 0;
    (*parent).s.m_iVehicleNum = 0;
    {
        (*pVeh).m_iArmor = (*(*pVeh).m_pVehicleInfo).armor;
        // parent->client->pers.maxHealth = parent->client->ps.stats[STAT_MAX_HEALTH] =
        //   parent->NPC->stats.health = parent->health =
        //   parent->client->ps.stats[STAT_HEALTH] = pVeh->m_iArmor;
        let v = (*pVeh).m_iArmor;
        (*(*parent).client).ps.stats[STAT_HEALTH as usize] = v;
        (*parent).health = v;
        (*(*parent).NPC).stats.health = v;
        (*(*parent).client).ps.stats[STAT_MAX_HEALTH as usize] = v;
        (*(*parent).client).pers.maxHealth = v;
        (*pVeh).m_iShields = (*(*pVeh).m_pVehicleInfo).shields;
        G_VehUpdateShields(parent);
        (*(*parent).client).ps.stats[STAT_ARMOR as usize] = (*pVeh).m_iShields;
    }
    (*parent).mass = (*(*pVeh).m_pVehicleInfo).mass as f32;
    //initialize the ammo to max
    i = 0;
    while (i as usize) < MAX_VEHICLE_WEAPONS {
        (*pVeh).weaponStatus[i as usize].ammo =
            (*(*pVeh).m_pVehicleInfo).weapon[i as usize].ammoMax;
        (*(*parent).client).ps.ammo[i as usize] = (*pVeh).weaponStatus[i as usize].ammo;
        i += 1;
    }
    i = 0;
    while (i as usize) < MAX_VEHICLE_TURRETS {
        (*pVeh).turretStatus[i as usize].nextMuzzle =
            (*(*pVeh).m_pVehicleInfo).turret[i as usize].iMuzzle[i as usize] - 1;
        (*pVeh).turretStatus[i as usize].ammo =
            (*(*pVeh).m_pVehicleInfo).turret[i as usize].iAmmoMax;
        (*(*parent).client).ps.ammo[MAX_VEHICLE_WEAPONS + i as usize] =
            (*pVeh).turretStatus[i as usize].ammo;
        if (*(*pVeh).m_pVehicleInfo).turret[i as usize].bAI != QFALSE {
            //they're going to be finding enemies, init this to NONE
            (*pVeh).turretStatus[i as usize].enemyEntNum = ENTITYNUM_NONE;
        }
        i += 1;
    }
    //begin stopped...?
    (*(*parent).client).ps.speed = 0.0;

    VectorClear(&mut *((*pVeh).m_vOrientation as *mut vec3_t));
    *(*pVeh).m_vOrientation.add(YAW) = (*parent).s.angles[YAW];

    if (*(*pVeh).m_pVehicleInfo).gravity != 0
        && (*(*pVeh).m_pVehicleInfo).gravity as f32 != (*addr_of!(g_gravity)).value
    {
        //not normal gravity
        if !(*parent).NPC.is_null() {
            (*(*parent).NPC).aiFlags |= NPCAI_CUSTOM_GRAVITY;
        }
        (*(*parent).client).ps.gravity = (*(*pVeh).m_pVehicleInfo).gravity;
    }

    if (*(*pVeh).m_pVehicleInfo).maxPassengers > 0 {
        // Allocate an array of entity pointers.
        // (MP uses a static pointer array; the SP-only G_Alloc is #ifdef'd out.)
        let mut i: c_int = 0;
        while i < (*(*pVeh).m_pVehicleInfo).maxPassengers {
            (*pVeh).m_ppPassengers[i as usize] = null_mut();
            i += 1;
        }
    }

    (*pVeh).m_iNumPassengers = 0;
    /*
    if ( pVeh->m_iVehicleTypeID == VH_FIGHTER )
    {
        pVeh->m_ulFlags = VEH_GEARSOPEN;
    }
    else
    */
    //why?! -rww
    {
        (*pVeh).m_ulFlags = 0;
    }
    (*pVeh).m_fTimeModifier = 1.0f32;
    (*pVeh).m_iBoarding = 0;
    (*pVeh).m_bWasBoarding = QFALSE;
    (*pVeh).m_pOldPilot = null_mut();
    VectorClear(&mut (*pVeh).m_vBoardingVelocity);
    (*pVeh).m_pPilot = null_mut();
    write_bytes(&mut (*pVeh).m_ucmd, 0, 1);
    (*pVeh).m_iDieTime = 0;
    (*pVeh).m_EjectDir = VEH_EJECT_LEFT;

    //pVeh->m_iDriverTag = -1;
    //pVeh->m_iLeftExhaustTag = -1;
    //pVeh->m_iRightExhaustTag = -1;
    //pVeh->m_iGun1Tag = -1;
    //pVeh->m_iGun1Bone = -1;
    //pVeh->m_iLeftWingBone = -1;
    //pVeh->m_iRightWingBone = -1;
    write_bytes(
        (*pVeh).m_iExhaustTag.as_mut_ptr() as *mut u8,
        0xff,
        core::mem::size_of::<c_int>() * MAX_VEHICLE_EXHAUSTS,
    );
    write_bytes(
        (*pVeh).m_iMuzzleTag.as_mut_ptr() as *mut u8,
        0xff,
        core::mem::size_of::<c_int>() * MAX_VEHICLE_MUZZLES,
    );
    // FIXME! Use external values read from the vehicle data file!
    (*pVeh).m_iDroidUnitTag = -1;

    //initialize to blaster, just since it's a basic weapon and there's no lightsaber crap...?
    (*(*parent).client).ps.weapon = WP_BLASTER;
    (*(*parent).client).ps.weaponstate = WEAPON_READY;
    (*(*parent).client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_BLASTER;

    //Initialize to landed (wings closed, gears down) animation
    {
        let iFlags: c_int = SETANIM_FLAG_NORMAL;
        let iBlend: c_int = 300;
        (*pVeh).m_ulFlags |= VEH_GEARSOPEN as c_ulong;
        BG_SetAnim(
            (*(*pVeh).m_pParentEntity).playerState,
            (*addr_of!(bgAllAnims))[(*(*pVeh).m_pParentEntity).localAnimIndex as usize].anims,
            SETANIM_BOTH,
            BOTH_VS_IDLE,
            iFlags,
            iBlend,
        );
    }

    QTRUE
}

/// `void SetParent( Vehicle_t *pVeh, bgEntity_t *pParentEntity )` (g_vehicles.c,
/// `GAME_INLINE`) — sets the vehicle's parent entity back-pointer.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*`.
pub unsafe extern "C" fn SetParent(pVeh: *mut Vehicle_t, pParentEntity: *mut bgEntity_t) {
    (*pVeh).m_pParentEntity = pParentEntity;
}

// Add a pilot to the vehicle.
/// `void SetPilot( Vehicle_t *pVeh, bgEntity_t *pPilot )` (g_vehicles.c, `GAME_INLINE`) —
/// sets the vehicle's pilot back-pointer.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*`.
pub unsafe extern "C" fn SetPilot(pVeh: *mut Vehicle_t, pPilot: *mut bgEntity_t) {
    (*pVeh).m_pPilot = pPilot;
}

// Add a passenger to the vehicle (false if we're full).
/// `bool AddPassenger( Vehicle_t *pVeh )` (g_vehicles.c, `GAME_INLINE`) — stub: always
/// returns `false` (this base vehicle never accepts passengers).
///
/// # Safety
/// Trivial; the argument is unused.
pub unsafe extern "C" fn AddPassenger(_pVeh: *mut Vehicle_t) -> qboolean {
    QFALSE
}

// Whether this vehicle is currently inhabited (by anyone) or not.
/// `bool Inhabited( Vehicle_t *pVeh )` (g_vehicles.c, `GAME_INLINE`) — true if the vehicle
/// has a pilot or any passengers.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*`.
pub unsafe extern "C" fn Inhabited(pVeh: *mut Vehicle_t) -> qboolean {
    (!(*pVeh).m_pPilot.is_null() || (*pVeh).m_iNumPassengers != 0) as qboolean
}

/// `void Ghost( Vehicle_t *pVeh, bgEntity_t *pEnt )` (g_vehicles.c) — hides the entity from
/// clients and disables its collision (used while a rider is mid-board/eject).
///
/// # Safety
/// `pEnt` may be null; when non-null it must be a valid `gentity_t*` and its `client` (if
/// non-null) must be valid.
pub unsafe extern "C" fn Ghost(_pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    if pEnt.is_null() {
        return;
    }

    let ent = pEnt as *mut gentity_t;

    // This was introduced to prevent one extra entity from being sent to the clients
    (*ent).r.svFlags |= SVF_NOCLIENT;

    (*ent).s.eFlags |= EF_NODRAW;
    if !(*ent).client.is_null() {
        (*(*ent).client).ps.eFlags |= EF_NODRAW;
    }
    (*ent).r.contents = 0;
}

/// `void UnGhost( Vehicle_t *pVeh, bgEntity_t *pEnt )` (g_vehicles.c) — reverses `Ghost`:
/// re-shows the entity and restores body collision.
///
/// # Safety
/// `pEnt` may be null; when non-null it must be a valid `gentity_t*` and its `client` (if
/// non-null) must be valid.
pub unsafe extern "C" fn UnGhost(_pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) {
    if pEnt.is_null() {
        return;
    }

    let ent = pEnt as *mut gentity_t;

    // make sure the client is sent again
    (*ent).r.svFlags &= !SVF_NOCLIENT;

    (*ent).s.eFlags &= !EF_NODRAW;
    if !(*ent).client.is_null() {
        (*(*ent).client).ps.eFlags &= !EF_NODRAW;
    }
    (*ent).r.contents = CONTENTS_BODY;
}

/// `void G_EjectDroidUnit( Vehicle_t *pVeh, qboolean kill )` (g_vehicles.c) — detaches the
/// carried droid (astromech) from the vehicle, clearing its vehicle/owner links and undying
/// flag, and optionally killing it.
///
/// The `QAGAME` server-side block is taken (this is the MP game module).
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` whose `m_pDroidUnit` is a valid `gentity_t*`.
pub unsafe fn G_EjectDroidUnit(pVeh: *mut Vehicle_t, kill: qboolean) {
    (*(*pVeh).m_pDroidUnit).s.m_iVehicleNum = ENTITYNUM_NONE;
    (*(*pVeh).m_pDroidUnit).s.owner = ENTITYNUM_NONE;
    //	pVeh->m_pDroidUnit->s.otherEntityNum2 = ENTITYNUM_NONE;
    {
        let droidEnt = (*pVeh).m_pDroidUnit as *mut gentity_t;
        (*droidEnt).flags &= !FL_UNDYING;
        (*droidEnt).r.ownerNum = ENTITYNUM_NONE;
        if !(*droidEnt).client.is_null() {
            (*(*droidEnt).client).ps.m_iVehicleNum = ENTITYNUM_NONE;
        }
        if kill != QFALSE {
            //Kill them, too
            //FIXME: proper origin, MOD and attacker (for credit/death message)?  Get from vehicle?
            G_MuteSound((*droidEnt).s.number, CHAN_VOICE);
            G_Damage(
                droidEnt,
                null_mut(),
                null_mut(),
                null_mut(),
                &mut (*droidEnt).s.origin,
                10000,
                0,
                MOD_SUICIDE,
            ); //FIXME: proper MOD?  Get from vehicle?
        }
    }
    (*pVeh).m_pDroidUnit = null_mut();
}

/// `bool ValidateBoard( Vehicle_t *pVeh, bgEntity_t *pEnt )` (g_vehicles.c) — decides
/// whether `pEnt` may board `pVeh`, and if so records the board direction in `m_iBoarding`.
/// Rejects dying vehicles; for an occupied vehicle, applies the per-type steal rules
/// (fighter: passenger seat only; walker: must be on the hatch; speeder: only on a
/// throw-mount); otherwise computes left/right/jump board from the entity's bearing.
///
/// `currentAngles`/`currentOrigin` resolve to `r.currentAngles`/`r.currentOrigin` per the
/// MP `#define`s at the top of `g_vehicles.c`.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` with valid `m_pParentEntity`/`m_pVehicleInfo`; `pEnt`
/// must be a valid `bgEntity_t*` (`gentity_t*`).
pub unsafe extern "C" fn ValidateBoard(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    // Determine where the entity is entering the vehicle from (left, right, or back).
    let mut vVehToEnt: vec3_t = [0.0; 3];
    let mut vVehDir: vec3_t = [0.0; 3];
    let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
    let ent = pEnt as *mut gentity_t;
    let mut vVehAngles: vec3_t = [0.0; 3];
    let fDot: f32;

    if (*pVeh).m_iDieTime > 0 {
        return QFALSE;
    }

    if !(*pVeh).m_pPilot.is_null() {
        //already have a driver!
        if (*(*pVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
            //I know, I know, this should by in the fighters's validateboard()
            //can never steal a fighter from it's pilot
            if (*pVeh).m_iNumPassengers < (*(*pVeh).m_pVehicleInfo).maxPassengers {
                return QTRUE;
            } else {
                return QFALSE;
            }
        } else if (*(*pVeh).m_pVehicleInfo).r#type == VH_WALKER {
            //I know, I know, this should by in the walker's validateboard()
            if (*ent).client.is_null() || (*(*ent).client).ps.groundEntityNum != (*parent).s.number
            {
                //can only steal an occupied AT-ST if you're on top (by the hatch)
                return QFALSE;
            }
        } else if (*(*pVeh).m_pVehicleInfo).r#type == VH_SPEEDER {
            //you can only steal the bike from the driver if you landed on the driver or bike
            return ((*pVeh).m_iBoarding == VEH_MOUNT_THROW_LEFT
                || (*pVeh).m_iBoarding == VEH_MOUNT_THROW_RIGHT) as qboolean;
        }
    }
    // Yes, you shouldn't have put this here (you 'should' have made an 'overriden' ValidateBoard func), but in this
    // instance it's more than adequate (which is why I do it too :-). Making a whole other function for this is silly.
    else if (*(*pVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
        // If you're a fighter, you allow everyone to enter you from all directions.
        return QTRUE;
    }

    // Clear out all orientation axis except for the yaw.
    VectorSet(&mut vVehAngles, 0.0, (*parent).r.currentAngles[YAW], 0.0);

    // Vector from Entity to Vehicle.
    VectorSubtract(
        &(*ent).r.currentOrigin,
        &(*parent).r.currentOrigin,
        &mut vVehToEnt,
    );
    vVehToEnt[2] = 0.0;
    VectorNormalize(&mut vVehToEnt);

    // Get the right vector.
    AngleVectors(&vVehAngles, None, Some(&mut vVehDir), None);
    VectorNormalize(&mut vVehDir);

    // Find the angle between the vehicle right vector and the vehicle to entity vector.
    fDot = DotProduct(&vVehToEnt, &vVehDir);

    // If the entity is within a certain angle to the left of the vehicle...
    if fDot >= 0.5f32 {
        // Right board.
        (*pVeh).m_iBoarding = -2;
    } else if fDot <= -0.5f32 {
        // Left board.
        (*pVeh).m_iBoarding = -1;
    }
    // Maybe they're trying to board from the back...
    else {
        // The forward vector of the vehicle.
        //	AngleVectors( vVehAngles, vVehDir, NULL, NULL );
        //	VectorNormalize( vVehDir );

        // Find the angle between the vehicle forward and the vehicle to entity vector.
        //	fDot = DotProduct( vVehToEnt, vVehDir );

        // If the entity is within a certain angle behind the vehicle...
        //if ( fDot <= -0.85f )
        {
            // Jump board.
            (*pVeh).m_iBoarding = -3;
        }
    }

    // If for some reason we couldn't board, leave...
    if (*pVeh).m_iBoarding > -1 {
        return QFALSE;
    }

    QTRUE
}

/// `static void StartDeathDelay( Vehicle_t *pVeh, int iDelayTimeOverride )` (g_vehicles.c)
/// — schedules the vehicle's explosion time and, if flammable, starts the fire loop sound.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` whose `m_pParentEntity`/`m_pVehicleInfo` are valid;
/// when flammable, `m_pParentEntity->client` must be non-null.
pub unsafe extern "C" fn StartDeathDelay(pVeh: *mut Vehicle_t, iDelayTimeOverride: c_int) {
    let parent = (*pVeh).m_pParentEntity as *mut gentity_t;

    if iDelayTimeOverride != 0 {
        (*pVeh).m_iDieTime = (*addr_of!(level)).time + iDelayTimeOverride;
    } else {
        (*pVeh).m_iDieTime = (*addr_of!(level)).time + (*(*pVeh).m_pVehicleInfo).explosionDelay;
    }

    if (*(*pVeh).m_pVehicleInfo).flammable != QFALSE {
        (*parent).s.loopSound = G_SoundIndex("sound/vehicles/common/fire_lp.wav");
        (*(*parent).client).ps.loopSound = (*parent).s.loopSound;
    }
}

// void FighterStorePilotViewAngles( Vehicle_t *pVeh, bgEntity_t *parent ) (g_vehicles.c:597)
// is wrapped in #ifdef VEH_CONTROL_SCHEME_4, which is never defined in the PC tree, so the
// whole function is excluded from the PC MP build. Omitted for Xbox->PC fidelity (Xbox had
// no such guard). Its only caller (Board) is likewise gated out.

/// `void G_VehUpdateShields( gentity_t *targ )` (g_vehicles.c) — recomputes the vehicle's
/// shield bar (`activeForcePass`, 0..10) from its current/max shield points, skipping
/// non-vehicles and shield-less vehicles.
///
/// # Safety
/// `targ` may be null; when non-null its `client`/`m_pVehicle`/`m_pVehicleInfo` chain (if
/// non-null) must be valid.
pub unsafe fn G_VehUpdateShields(targ: *mut gentity_t) {
    if targ.is_null()
        || (*targ).client.is_null()
        || (*targ).m_pVehicle.is_null()
        || (*(*targ).m_pVehicle).m_pVehicleInfo.is_null()
    {
        return;
    }
    if (*(*(*targ).m_pVehicle).m_pVehicleInfo).shields <= 0 {
        //doesn't have shields, so don't have to send it
        return;
    }
    let frac = ((*(*targ).m_pVehicle).m_iShields as f32
        / (*(*(*targ).m_pVehicle).m_pVehicleInfo).shields as f32)
        * 10.0f32;
    (*(*targ).client).ps.activeForcePass = (frac as f64).floor() as c_int;
}

/// `Vehicle_t *G_IsRidingVehicle( gentity_t *pEnt )` (g_vehicles.c:100) — if `pEnt` is a
/// non-vehicle client currently assigned a vehicle index, returns that vehicle's
/// `Vehicle_t*`; otherwise NULL.
///
/// # Safety
/// `pEnt` may be null; when non-null its `client` (if non-null) must be valid, and
/// `g_entities` must point at a live entity array indexed by `s.m_iVehicleNum`.
pub unsafe fn G_IsRidingVehicle(pEnt: *mut gentity_t) -> *mut Vehicle_t {
    let ent = pEnt;

    if !ent.is_null()
        && !(*ent).client.is_null()
        && (*(*ent).client).NPC_class != CLASS_VEHICLE
        && (*ent).s.m_iVehicleNum != 0
    //ent->client && ( ent->client->ps.eFlags & EF_IN_VEHICLE ) && ent->owner )
    {
        return (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*ent).s.m_iVehicleNum as usize))
        .m_pVehicle;
    }
    null_mut()
}

/// `float G_CanJumpToEnemyVeh( Vehicle_t *pVeh, const usercmd_t *pUcmd )` (g_vehicles.c:113)
/// — stub: always returns `0.0f`.
///
/// # Safety
/// Trivial; arguments are unused.
pub unsafe fn G_CanJumpToEnemyVeh(_pVeh: *mut Vehicle_t, _pUcmd: *const usercmd_t) -> f32 {
    0.0f32
}

/// `void G_VehicleTrace( trace_t *results, const vec3_t start, const vec3_t tMins,
/// const vec3_t tMaxs, const vec3_t end, int passEntityNum, int contentmask )`
/// (g_vehicles.c:95) — thin pass-through to `trap_Trace`, writing the trace result into
/// `*results`. The idiomatic [`trap::Trace`] wrapper returns the `trace_t` by value; this
/// mirrors the C signature (caller-provided `results` out-pointer) by copying that value in.
///
/// # Safety
/// `results` must be a valid, writable `*mut trace_t`; `start`/`tMins`/`tMaxs`/`end` must be
/// valid `*const vec3_t` (C's `vec3_t` parameters decay to `float*`).
pub unsafe fn G_VehicleTrace(
    results: *mut trace_t,
    start: *const vec3_t,
    tMins: *const vec3_t,
    tMaxs: *const vec3_t,
    end: *const vec3_t,
    passEntityNum: c_int,
    contentmask: c_int,
) {
    *results = trap::Trace(&*start, &*tMins, &*tMaxs, &*end, passEntityNum, contentmask);
}

// Attachs an entity to the vehicle it's riding (it's owner).
/// `void G_AttachToVehicle( gentity_t *pEnt, usercmd_t **ucmd )` (g_vehicles.c:165) — places a
/// rider entity at the vehicle's `*driver` tag: takes the vehicle's `waypoint`, looks up the
/// driver bolt on the vehicle's ghoul2 model, extracts the tag's origin into the rider's
/// `ps.origin`, sets the entity origin there and relinks it. No oracle (entity/G2-trap state).
///
/// # Safety
/// `pEnt`/`ucmd` follow the C contract (early-returns on either null). `pEnt->client` must be
/// valid; `g_entities` indexed by `r.ownerNum` must be live. `m_vOrientation` decays to
/// `float*` as the C angles arg.
pub unsafe fn G_AttachToVehicle(pEnt: *mut gentity_t, ucmd: *mut *mut usercmd_t) {
    let vehEnt: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let ent: *mut gentity_t;
    let crotchBolt: c_int;

    if pEnt.is_null() || ucmd.is_null() {
        return;
    }

    ent = pEnt;

    vehEnt =
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize);
    (*ent).waypoint = (*vehEnt).waypoint; // take the veh's waypoint as your own

    if (*vehEnt).m_pVehicle.is_null() {
        return;
    }

    crotchBolt = trap::G2API_AddBolt((*vehEnt).ghoul2, 0, "*driver");

    // Get the driver tag.
    trap::G2API_GetBoltMatrix(
        (*vehEnt).ghoul2,
        0,
        crotchBolt,
        &mut boltMatrix,
        &*(*(*vehEnt).m_pVehicle).m_vOrientation.cast::<vec3_t>(),
        &(*vehEnt).r.currentOrigin,
        (*addr_of!(level)).time,
        core::ptr::null_mut(),
        &(*vehEnt).modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut (*(*ent).client).ps.origin);
    G_SetOrigin(ent, &(*(*ent).client).ps.origin.clone());
    trap::LinkEntity(ent);
}

// void G_KnockOffVehicle( gentity_t *pRider, gentity_t *self, qboolean bPull ) (g_vehicles.c:292)
// is wrapped in #ifndef _JK2MP, so the whole function is excluded from the MP build (where
// _JK2MP is defined). It is SP-only code (its sibling G_DriveATST/G_DrivableATSTDie block, also
// #ifndef _JK2MP //"don't want this in mp at least for now", is likewise gated out). Omitted for
// MP-module fidelity — not part of the codemp/game target.

// Animate the vehicle and it's riders.
/// `void Animate( Vehicle_t *pVeh )` (g_vehicles.c:194) — dispatches to the per-vehicle-type
/// `AnimateRiders`/`AnimateVehicle` fn-pointers in `m_pVehicleInfo`. Those targets live in the
/// still-unported vehicle-type drivers and are wired up outside this file; this is the faithful
/// fn-pointer dispatch only — it pulls in no driver logic.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` whose `m_pVehicleInfo` is valid; `AnimateVehicle` must be
/// a live (non-null) fn-pointer (as the C unconditionally calls it).
pub unsafe extern "C" fn Animate(pVeh: *mut Vehicle_t) {
    // Validate a pilot rider.
    if !(*pVeh).m_pPilot.is_null() {
        if let Some(AnimateRiders) = (*(*pVeh).m_pVehicleInfo).AnimateRiders {
            AnimateRiders(pVeh);
        }
    }

    ((*(*pVeh).m_pVehicleInfo).AnimateVehicle.unwrap())(pVeh);
}

// Register all the assets used by this vehicle.
/// `void RegisterAssets( Vehicle_t *pVeh )` (g_vehicles.c:1328) — empty function body
/// (the original registers nothing here).
///
/// # Safety
/// Trivial; the argument is unused.
pub unsafe extern "C" fn RegisterAssets(_pVeh: *mut Vehicle_t) {}

/// `void G_VehicleDamageBoxSizing( Vehicle_t *pVeh )` (g_vehicles.c:2505) — once all four
/// wings have been blasted off, traces a tight new bounding box for the wingless hull and,
/// if clear, shrinks the parent entity's `mins`/`maxs` to it (letting a stripped ship fly
/// down narrow halls); if the new box is obstructed, suicides the vehicle instead.
///
/// `parent->maxs`/`parent->mins` resolve to `r.maxs`/`r.mins` per the MP `#define`s at the
/// top of `g_vehicles.c`. `parent->ghoul2` is read only as a NULL-check (no Ghoul2 trap).
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` whose `m_pParentEntity` is a valid `gentity_t*`; when
/// its `client`/`m_pVehicle` are non-null they must be valid.
pub unsafe fn G_VehicleDamageBoxSizing(pVeh: *mut Vehicle_t) {
    let mut fwd: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut nose: vec3_t = [0.0; 3]; //maxs
    let mut back: vec3_t = [0.0; 3]; //mins
    let trace: trace_t;
    let fDist: f32 = 256.0f32; //estimated distance to nose from origin
    let bDist: f32 = 256.0f32; //estimated distance to back from origin
    let wDist: f32 = 32.0f32; //width on each side from origin
    let hDist: f32 = 32.0f32; //height on each side from origin
    let parent = (*pVeh).m_pParentEntity as *mut gentity_t;

    if (*parent).ghoul2.is_null() || (*parent).m_pVehicle.is_null() || (*parent).client.is_null() {
        //shouldn't have gotten in here then
        return;
    }

    //for now, let's only do anything if all wings are stripped off.
    //this is because I want to be able to tear my wings off and fling
    //myself down narrow hallways to my death. Because it's fun! -rww
    if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C) == 0
        || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D) == 0
        || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E) == 0
        || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F) == 0
    {
        return;
    }

    //get directions based on orientation
    AngleVectors(
        &*((*pVeh).m_vOrientation as *const vec3_t),
        Some(&mut fwd),
        Some(&mut right),
        Some(&mut up),
    );

    //get the nose and back positions (relative to 0, they're gonna be mins/maxs)
    VectorMA(&vec3_origin, fDist, &fwd, &mut nose);
    VectorMA(&vec3_origin, -bDist, &fwd, &mut back);

    //move the nose and back to opposite right/left, they will end up as our relative mins and maxs
    VectorMA(&nose.clone(), wDist, &right, &mut nose);
    VectorMA(&nose.clone(), -wDist, &right, &mut back);

    //use the same concept for up/down now
    VectorMA(&nose.clone(), hDist, &up, &mut nose);
    VectorMA(&nose.clone(), -hDist, &up, &mut back);

    //and now, let's trace and see if our new mins/maxs are safe..
    trace = trap::Trace(
        &(*(*parent).client).ps.origin,
        &back,
        &nose,
        &(*(*parent).client).ps.origin,
        (*parent).s.number,
        (*parent).clipmask,
    );
    if trace.allsolid == 0 && trace.startsolid == 0 && trace.fraction == 1.0f32 {
        //all clear!
        VectorCopy(&nose, &mut (*parent).r.maxs);
        VectorCopy(&back, &mut (*parent).r.mins);
    } else {
        //oh well, DIE!
        //FIXME: does this give proper credit to the enemy who shot you down?
        G_Damage(
            parent,
            parent,
            parent,
            null_mut(),
            &mut (*(*parent).client).ps.origin,
            9999,
            DAMAGE_NO_PROTECTION,
            MOD_SUICIDE,
        );
    }
}

//get one of 4 possible impact locations based on the trace direction
/// `int G_FlyVehicleImpactDir( gentity_t *veh, trace_t *trace )` (g_vehicles.c:2563) —
/// classifies a flight-vehicle collision into one of the four `SHIPSURF_*` regions: first
/// (if the nose is clear) traces each not-yet-removed wing forward for a wing impact, then
/// falls back to the trace plane-normal yaw relative to the pilot's view to pick
/// front/right/left/back.
///
/// # Safety
/// `veh` must be a valid `gentity_t*`; `trace` may be null. When non-null, `veh->client` and
/// `veh->m_pVehicle` must be valid.
pub unsafe fn G_FlyVehicleImpactDir(veh: *mut gentity_t, trace: *mut trace_t) -> c_int {
    let impactAngle: f32;
    let relativeAngle: f32;
    let mut localTrace: trace_t;
    let mut testMins: vec3_t = [0.0; 3];
    let mut testMaxs: vec3_t = [0.0; 3];
    let mut rWing: vec3_t = [0.0; 3];
    let mut lWing: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut fPos: vec3_t = [0.0; 3];
    let pVeh = (*veh).m_pVehicle;
    let mut noseClear: qboolean = QFALSE;

    if trace.is_null() || pVeh.is_null() || (*veh).client.is_null() {
        return -1;
    }

    AngleVectors(
        &(*(*veh).client).ps.viewangles,
        Some(&mut fwd),
        Some(&mut right),
        None,
    );
    VectorSet(&mut testMins, -24.0f32, -24.0f32, -24.0f32);
    VectorSet(&mut testMaxs, 24.0f32, 24.0f32, 24.0f32);

    //do a trace to determine if the nose is clear
    VectorMA(&(*(*veh).client).ps.origin, 256.0f32, &fwd, &mut fPos);
    localTrace = trap::Trace(
        &(*(*veh).client).ps.origin,
        &testMins,
        &testMaxs,
        &fPos,
        (*veh).s.number,
        (*veh).clipmask,
    );
    if localTrace.startsolid == 0 && localTrace.allsolid == 0 && localTrace.fraction == 1.0f32 {
        //otherwise I guess it's not clear..
        noseClear = QTRUE;
    }

    if noseClear != QFALSE {
        //if nose is clear check for tearing the wings off
        //sadly, the trace endpos given always matches the vehicle origin, so we
        //can't get a real impact direction. First we'll trace forward and see if the wings are colliding
        //with anything, and if not, we'll fall back to checking the trace plane normal.
        VectorMA(&(*(*veh).client).ps.origin, 128.0f32, &right, &mut rWing);
        VectorMA(&(*(*veh).client).ps.origin, -128.0f32, &right, &mut lWing);

        //test the right wing - unless it's already removed
        if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E) == 0
            || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F) == 0
        {
            VectorMA(&rWing, 256.0f32, &fwd, &mut fPos);
            localTrace = trap::Trace(
                &rWing,
                &testMins,
                &testMaxs,
                &fPos,
                (*veh).s.number,
                (*veh).clipmask,
            );
            if localTrace.startsolid != 0
                || localTrace.allsolid != 0
                || localTrace.fraction != 1.0f32
            {
                //impact
                return SHIPSURF_RIGHT;
            }
        }

        //test the left wing - unless it's already removed
        if ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C) == 0
            || ((*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D) == 0
        {
            VectorMA(&lWing, 256.0f32, &fwd, &mut fPos);
            localTrace = trap::Trace(
                &lWing,
                &testMins,
                &testMaxs,
                &fPos,
                (*veh).s.number,
                (*veh).clipmask,
            );
            if localTrace.startsolid != 0
                || localTrace.allsolid != 0
                || localTrace.fraction != 1.0f32
            {
                //impact
                return SHIPSURF_LEFT;
            }
        }
    }

    //try to use the trace plane normal
    impactAngle = vectoyaw(&(*trace).plane.normal);
    relativeAngle = AngleSubtract(impactAngle, (*(*veh).client).ps.viewangles[YAW]);

    if relativeAngle > 130.0f32 || relativeAngle < -130.0f32 {
        //consider this front
        return SHIPSURF_FRONT;
    } else if relativeAngle > 0.0f32 {
        return SHIPSURF_RIGHT;
    } else if relativeAngle < 0.0f32 {
        return SHIPSURF_LEFT;
    }

    SHIPSURF_BACK
}

//try to break surfaces off the ship on impact
// (C also `#define TURN_ON 0x00000000` here; unused in the ported fns, so omitted to
// avoid a dead-const warning — `NPC_SetSurfaceOnOff` only ever receives `TURN_OFF`.)
const TURN_OFF: c_int = 0x00000100;

/// `qboolean G_FlyVehicleDestroySurface( gentity_t *veh, int surface )` (g_vehicles.c:2822) —
/// tears the named model surfaces off the ship for the given `SHIPSURF_*` impact region
/// (via `NPC_SetSurfaceOnOff`), ORs the matching `SHIPSURF_BROKEN_*` bits into
/// `m_iRemovedSurfaces`, plays the pilot's death scream on the first break, then deals
/// non-self explosive radius damage and starts the electrocution shader.
///
/// # Safety
/// `veh` must be a valid `gentity_t*` whose `client`/`m_pVehicle` are valid.
pub unsafe fn G_FlyVehicleDestroySurface(veh: *mut gentity_t, surface: c_int) -> qboolean {
    let mut surfName: [*const c_char; 4] = [null_mut(); 4]; //up to 4 surfs at once
    let mut numSurfs: c_int = 0;
    let mut smashedBits: c_int = 0;

    if surface == -1 {
        //not valid?
        return QFALSE;
    }

    match surface {
        SHIPSURF_FRONT => {
            //break the nose off
            surfName[0] = c"nose".as_ptr();

            smashedBits = SHIPSURF_BROKEN_G;

            numSurfs = 1;
        }
        SHIPSURF_BACK => {
            //break both the bottom wings off for a backward impact I guess
            surfName[0] = c"r_wing2".as_ptr();
            surfName[1] = c"l_wing2".as_ptr();

            //get rid of the landing gear
            surfName[2] = c"r_gear".as_ptr();
            surfName[3] = c"l_gear".as_ptr();

            smashedBits =
                SHIPSURF_BROKEN_A | SHIPSURF_BROKEN_B | SHIPSURF_BROKEN_D | SHIPSURF_BROKEN_F;

            numSurfs = 4;
        }
        SHIPSURF_RIGHT => {
            //break both right wings off
            surfName[0] = c"r_wing1".as_ptr();
            surfName[1] = c"r_wing2".as_ptr();

            //get rid of the landing gear
            surfName[2] = c"r_gear".as_ptr();

            smashedBits = SHIPSURF_BROKEN_B | SHIPSURF_BROKEN_E | SHIPSURF_BROKEN_F;

            numSurfs = 3;
        }
        SHIPSURF_LEFT => {
            //break both left wings off
            surfName[0] = c"l_wing1".as_ptr();
            surfName[1] = c"l_wing2".as_ptr();

            //get rid of the landing gear
            surfName[2] = c"l_gear".as_ptr();

            smashedBits = SHIPSURF_BROKEN_A | SHIPSURF_BROKEN_C | SHIPSURF_BROKEN_D;

            numSurfs = 3;
        }
        _ => {}
    }

    if numSurfs < 1 {
        //didn't get any valid surfs..
        return QFALSE;
    }

    while numSurfs > 0 {
        //use my silly system of automatically managing surf status on both client and server
        numSurfs -= 1;
        NPC_SetSurfaceOnOff(veh, surfName[numSurfs as usize], TURN_OFF);
    }

    if (*(*veh).m_pVehicle).m_iRemovedSurfaces == 0 {
        //first time something got blown off
        if !(*(*veh).m_pVehicle).m_pPilot.is_null() {
            //make the pilot scream to his death
            G_EntitySound(
                (*(*veh).m_pVehicle).m_pPilot as *mut gentity_t,
                CHAN_VOICE,
                G_SoundIndex("*falling1.wav"),
            );
        }
    }
    //so we can check what's broken
    (*(*veh).m_pVehicle).m_iRemovedSurfaces |= smashedBits;

    //do some explosive damage, but don't damage this ship with it
    G_RadiusDamage(
        &(*(*veh).client).ps.origin,
        veh,
        100 as f32,
        500 as f32,
        veh,
        null_mut(),
        MOD_VEH_EXPLOSION,
    );

    //when spiraling to your death, do the electical shader
    (*(*veh).client).ps.electrifyTime = (*addr_of!(level)).time + 10000;

    QTRUE
}

/// `void G_FlyVehicleSurfaceDestruction( gentity_t *veh, trace_t *trace, int magnitude,
/// qboolean force )` (g_vehicles.c:2910) — on a flight-vehicle impact, accumulates location
/// damage for the impacted `SHIPSURF_*` region and, once past that region's death health (or
/// when `force`d), tears the surface off and sets the damage-loc flags. Can break off a second
/// distinct surface in the same impact (the C `goto anotherImpact`, modelled as a loop here).
///
/// `veh->ghoul2` is read only as a NULL-check (no Ghoul2 trap).
///
/// # Safety
/// `veh` must be a valid `gentity_t*` whose `client`/`m_pVehicle`/`m_pVehicleInfo` chain is
/// valid; `trace` may be null (forwarded to `G_FlyVehicleImpactDir`).
pub unsafe fn G_FlyVehicleSurfaceDestruction(
    veh: *mut gentity_t,
    trace: *mut trace_t,
    magnitude: c_int,
    force: qboolean,
) {
    let mut impactDir: c_int;
    let mut secondImpact: c_int;
    let mut deathPoint: c_int = -1;
    let mut alreadyRebroken: qboolean = QFALSE;

    if (*veh).ghoul2.is_null() || (*veh).m_pVehicle.is_null() {
        //no g2 instance.. or no vehicle instance
        return;
    }

    impactDir = G_FlyVehicleImpactDir(veh, trace);

    // C label `anotherImpact:` — re-entered via `goto` to break off a second surface.
    loop {
        if impactDir == -1 {
            //not valid?
            return;
        }

        (*veh).locationDamage[impactDir as usize] += magnitude * 7;

        match impactDir {
            SHIPSURF_FRONT => {
                deathPoint = (*(*(*veh).m_pVehicle).m_pVehicleInfo).health_front;
            }
            SHIPSURF_BACK => {
                deathPoint = (*(*(*veh).m_pVehicle).m_pVehicleInfo).health_back;
            }
            SHIPSURF_RIGHT => {
                deathPoint = (*(*(*veh).m_pVehicle).m_pVehicleInfo).health_right;
            }
            SHIPSURF_LEFT => {
                deathPoint = (*(*(*veh).m_pVehicle).m_pVehicleInfo).health_left;
            }
            _ => {}
        }

        if deathPoint != -1 {
            //got a valid health value
            if force != QFALSE && (*veh).locationDamage[impactDir as usize] < deathPoint {
                //force that surf to be destroyed
                (*veh).locationDamage[impactDir as usize] = deathPoint;
            }
            if (*veh).locationDamage[impactDir as usize] >= deathPoint {
                //do it
                if G_FlyVehicleDestroySurface(veh, impactDir) != QFALSE {
                    //actually took off a surface
                    G_VehicleSetDamageLocFlags(veh, impactDir, deathPoint);
                }
            } else {
                G_VehicleSetDamageLocFlags(veh, impactDir, deathPoint);
            }
        }

        if alreadyRebroken == QFALSE {
            secondImpact = G_FlyVehicleImpactDir(veh, trace);
            if impactDir != secondImpact {
                //can break off another piece in this same impact.. but only break off up to 2 at once
                alreadyRebroken = QTRUE;
                impactDir = secondImpact;
                continue; // goto anotherImpact;
            }
        }
        break;
    }
}

/// `void G_VehicleSpawn( gentity_t *self )` (g_vehicles.c:119) — turns a map vehicle
/// spawner into a live vehicle NPC: links the spawner, ensures a default `count`, spawns the
/// NPC via `NPC_Spawn_Do` (which removes `self`), restores the saved yaw, forces cinematic
/// behavior for non-animal vehicles, and (spawnflag 1) arms the no-pilot self-destruct timer
/// with default damage/speed fallbacks.
///
/// `self->currentOrigin` resolves to `r.currentOrigin` per the MP `#define`s.
///
/// `extern "C"` so it can be installed as a `gentity_t::think` fn-ptr
/// (`Option<unsafe extern "C" fn(*mut gentity_t)>`) by the NPC_spawn.c vehicle spawners
/// (`SP_NPC_Vehicle`/`NPC_VehicleSpawnUse`), matching the C `self->think = G_VehicleSpawn`.
///
/// # Safety
/// `self` must be a valid `gentity_t*`; on a successful spawn the returned `vehEnt` must have
/// a valid `NPC`/`m_pVehicle`/`m_pVehicleInfo` chain.
pub unsafe extern "C" fn G_VehicleSpawn(self_: *mut gentity_t) {
    let yaw: f32;
    let vehEnt: *mut gentity_t;

    VectorCopy(&(*self_).r.currentOrigin, &mut (*self_).s.origin);

    trap::LinkEntity(self_);

    if (*self_).count == 0 {
        (*self_).count = 1;
    }

    //save this because self gets removed in next func
    yaw = (*self_).s.angles[YAW];

    vehEnt = NPC_Spawn_Do(self_);

    if vehEnt.is_null() {
        return; //return NULL;
    }

    (*vehEnt).s.angles[YAW] = yaw;
    if (*(*(*vehEnt).m_pVehicle).m_pVehicleInfo).r#type != VH_ANIMAL {
        (*(*vehEnt).NPC).behaviorState = BS_CINEMATIC;
    }

    if (*vehEnt).spawnflags & 1 != 0 {
        //die without pilot
        if (*vehEnt).damage == 0 {
            //default 10 sec
            (*vehEnt).damage = 10000;
        }
        if (*vehEnt).speed == 0.0 {
            //default 512 units
            (*vehEnt).speed = 512.0f32;
        }
        (*(*vehEnt).m_pVehicle).m_iPilotTime = (*addr_of!(level)).time + (*vehEnt).damage;
    }
    //return vehEnt;
}

/// `bool Board( Vehicle_t *pVeh, bgEntity_t *pEnt )` (g_vehicles.c:342) — puts `pEnt` on the
/// vehicle: validates boardability, then makes a player (`s.number < MAX_CLIENTS`) the pilot
/// (or a passenger if a pilot exists), wires the bidirectional vehicle/owner indices, restores
/// suspended docks, hides the rider if configured, plays the start sound, caches fighter view
/// angles, and snaps the rider's view to the vehicle yaw.
///
/// The MP (`_JK2MP`/`QAGAME`) path is taken throughout; the SP-only cgame bits
/// (`cvar_set`/`CG_CenterPrint`/`CG_ChangeWeapon`/`G_RemoveWeaponModels`/`G_SoundIndexOnEnt`)
/// are `#ifndef _JK2MP` and so excluded. Sibling calls go through the now-installed
/// `m_pVehicleInfo` vtable (`ValidateBoard`/`SetPilot`/`Ghost`). `bool`/`true`/`false` are
/// `#define`d to `qboolean`/`qtrue`/`qfalse`.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` with valid `m_pParentEntity`/`m_pVehicleInfo` and an
/// installed shared vtable; `pEnt` may be null. Boarding clients must have a non-null `client`.
pub unsafe extern "C" fn Board(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean {
    let mut vPlayerDir: vec3_t = [0.0; 3];
    let ent = pEnt as *mut gentity_t;
    let parent = (*pVeh).m_pParentEntity as *mut gentity_t;

    // If it's not a valid entity, OR if the vehicle is blowing up (it's dead), OR it's not
    // empty, OR we're already being boarded, OR the person trying to get on us is already
    // in a vehicle (that was a fun bug :-), leave!
    if ent.is_null()
        || (*parent).health <= 0
        /*|| !( parent->client->ps.eFlags & EF_EMPTY_VEHICLE )*/
        || ((*pVeh).m_iBoarding > 0)
        || ((*(*ent).client).ps.m_iVehicleNum != 0)
    {
        return QFALSE;
    }

    // Bucking so we can't do anything (NOTE: Should probably be a better name since fighters don't buck...).
    if (*pVeh).m_ulFlags & VEH_BUCKING as c_ulong != 0 {
        return QFALSE;
    }

    // Validate the entity's ability to board this vehicle.
    if ((*(*pVeh).m_pVehicleInfo).ValidateBoard.unwrap())(pVeh, pEnt) == QFALSE {
        return QFALSE;
    }

    // FIXME FIXME!!! Ask Mike monday where ent->client->ps.eFlags might be getting changed!!! It is always 0 (when it should
    // be 1024) so a person riding a vehicle is able to ride another vehicle!!!!!!!!

    // Tell everybody their status.
    // ALWAYS let the player be the pilot.
    if (*ent).s.number < MAX_CLIENTS as c_int {
        (*pVeh).m_pOldPilot = (*pVeh).m_pPilot;

        if (*pVeh).m_pPilot.is_null() {
            //become the pilot, if there isn't one now
            ((*(*pVeh).m_pVehicleInfo).SetPilot.unwrap())(pVeh, ent as *mut bgEntity_t);
        }
        // If we're not yet full...
        else if (*pVeh).m_iNumPassengers < (*(*pVeh).m_pVehicleInfo).maxPassengers {
            let mut i: c_int;
            // Find an empty slot and put that passenger here.
            i = 0;
            while i < (*(*pVeh).m_pVehicleInfo).maxPassengers {
                if (*pVeh).m_ppPassengers[i as usize].is_null() {
                    (*pVeh).m_ppPassengers[i as usize] = ent as *mut bgEntity_t;
                    //Server just needs to tell client which passengernum he is
                    if !(*ent).client.is_null() {
                        (*(*ent).client).ps.generic1 = i + 1;
                    }
                    break;
                }
                i += 1;
            }
            (*pVeh).m_iNumPassengers += 1;
        }
        // We're full, sorry...
        else {
            return QFALSE;
        }
        (*ent).s.m_iVehicleNum = (*parent).s.number;
        if !(*ent).client.is_null() {
            (*(*ent).client).ps.m_iVehicleNum = (*ent).s.m_iVehicleNum;
        }
        if (*pVeh).m_pPilot == ent as *mut bgEntity_t {
            (*parent).r.ownerNum = (*ent).s.number;
            (*parent).s.owner = (*parent).r.ownerNum; //for prediction
        }

        {
            let gParent = parent;
            if (*gParent).spawnflags & 2 != 0 {
                //was being suspended
                (*gParent).spawnflags &= !2; //SUSPENDED - clear this spawnflag, no longer docked, okay to free-fall if not in space
                                             //gParent->client->ps.eFlags &= ~EF_RADAROBJECT;
                G_Sound(
                    gParent,
                    CHAN_AUTO,
                    G_SoundIndex("sound/vehicles/common/release.wav"),
                );
                if (*gParent).fly_sound_debounce_time != 0 {
                    //we should drop like a rock for a few seconds
                    (*pVeh).m_iDropTime =
                        (*addr_of!(level)).time + (*gParent).fly_sound_debounce_time;
                }
            }
        }

        //FIXME: rider needs to look in vehicle's direction when he gets in
        // Clear these since they're used to turn the vehicle now.
        /*SetClientViewAngle( ent, pVeh->m_vOrientation );
        memset( &parent->client->usercmd, 0, sizeof( usercmd_t ) );
        memset( &pVeh->m_ucmd, 0, sizeof( usercmd_t ) );
        VectorClear( parent->client->ps.viewangles );
        VectorClear( parent->client->ps.delta_angles );*/

        // Set the looping sound only when there is a pilot (when the vehicle is "on").
        if (*(*pVeh).m_pVehicleInfo).soundLoop != 0 {
            (*(*parent).client).ps.loopSound = (*(*pVeh).m_pVehicleInfo).soundLoop;
            (*parent).s.loopSound = (*(*parent).client).ps.loopSound;
        }
    } else {
        // If there's no pilot, try to drive this vehicle.
        if (*pVeh).m_pPilot.is_null() {
            ((*(*pVeh).m_pVehicleInfo).SetPilot.unwrap())(pVeh, ent as *mut bgEntity_t);
            // TODO: Set pilot should do all this stuff....
            (*parent).r.ownerNum = (*ent).s.number;
            (*parent).s.owner = (*parent).r.ownerNum; //for prediction

            // Set the looping sound only when there is a pilot (when the vehicle is "on").
            if (*(*pVeh).m_pVehicleInfo).soundLoop != 0 {
                (*(*parent).client).ps.loopSound = (*(*pVeh).m_pVehicleInfo).soundLoop;
                (*parent).s.loopSound = (*(*parent).client).ps.loopSound;
            }

            (*(*parent).client).ps.speed = 0.0;
            write_bytes(&mut (*pVeh).m_ucmd, 0, 1);
        }
        // If we're not yet full...
        else if (*pVeh).m_iNumPassengers < (*(*pVeh).m_pVehicleInfo).maxPassengers {
            let mut i: c_int;
            // Find an empty slot and put that passenger here.
            i = 0;
            while i < (*(*pVeh).m_pVehicleInfo).maxPassengers {
                if (*pVeh).m_ppPassengers[i as usize].is_null() {
                    (*pVeh).m_ppPassengers[i as usize] = ent as *mut bgEntity_t;
                    //Server just needs to tell client which passengernum he is
                    if !(*ent).client.is_null() {
                        (*(*ent).client).ps.generic1 = i + 1;
                    }
                    break;
                }
                i += 1;
            }
            (*pVeh).m_iNumPassengers += 1;
        }
        // We're full, sorry...
        else {
            return QFALSE;
        }
    }

    // Make sure the entity knows it's in a vehicle.
    (*(*ent).client).ps.m_iVehicleNum = (*parent).s.number;
    (*ent).r.ownerNum = (*parent).s.number;
    (*ent).s.owner = (*ent).r.ownerNum; //for prediction
    if (*pVeh).m_pPilot == ent as *mut bgEntity_t {
        (*(*parent).client).ps.m_iVehicleNum = (*ent).s.number + 1; //always gonna be under MAX_CLIENTS so no worries about 1 byte overflow
    }

    //memset( &ent->client->usercmd, 0, sizeof( usercmd_t ) );

    //FIXME: no saber or weapons if numHands = 2, should switch to speeder weapon, no attack anim on player
    if (*(*pVeh).m_pVehicleInfo).numHands == 2 {
        //switch to vehicle weapon
        // (SP-only `#ifndef _JK2MP`: CG_ChangeWeapon/ps.weapon=WP_NONE/G_RemoveWeaponModels —
        //  excluded in the MP build.)
    }

    if (*(*pVeh).m_pVehicleInfo).hideRider != QFALSE {
        //hide the rider
        ((*(*pVeh).m_pVehicleInfo).Ghost.unwrap())(pVeh, ent as *mut bgEntity_t);
    }

    // Play the start sounds
    if (*(*pVeh).m_pVehicleInfo).soundOn != 0 {
        G_Sound(parent, CHAN_AUTO, (*(*pVeh).m_pVehicleInfo).soundOn);
    }

    // (PC: the VH_FIGHTER FighterStorePilotViewAngles "clear their angles" block
    // is under #ifdef VEH_CONTROL_SCHEME_4, never defined in the PC tree — excluded.)
    VectorCopy(&*((*pVeh).m_vOrientation as *const vec3_t), &mut vPlayerDir);
    vPlayerDir[ROLL] = 0.0;
    SetClientViewAngle(ent, &vPlayerDir);

    QTRUE
}

/// `bool VEH_TryEject( Vehicle_t *pVeh, gentity_t *parent, gentity_t *ent, int ejectDir,
/// vec3_t vExitPos )` (g_vehicles.c:583) — computes a safe exit position for `ent` leaving the
/// vehicle in `ejectDir` (offset out by the combined diagonal half-extents of vehicle+entity,
/// with a walker bias), traces from the entity's origin to that spot, and reports whether the
/// eject is viable — writing the (possibly clipped) landing spot into `vExitPos`.
///
/// `currentAngles`/`currentOrigin`/`maxs` resolve to `r.currentAngles`/`r.currentOrigin`/
/// `r.maxs` per the MP `#define`s. The `_JK2MP` paths are taken: client mins/maxs are the
/// hardcoded PMove box, and the post-trace "stuck on body" guard is commented out in MP, so a
/// non-clear trace (`fraction < 1`) always fails. `bool`/`true`/`false` are `#define`d to
/// `qboolean`/`qtrue`/`qfalse` in this file.
///
/// # Safety
/// `pVeh`/`parent`/`ent` must be valid; `vExitPos` must be a valid, writable `*mut vec3_t`.
pub unsafe fn VEH_TryEject(
    pVeh: *mut Vehicle_t,
    parent: *mut gentity_t,
    ent: *mut gentity_t,
    ejectDir: c_int,
    vExitPos: *mut vec3_t,
) -> qboolean {
    let mut fBias: f32;
    let fVehDiag: f32;
    let fEntDiag: f32;
    let oldOwner: c_int;
    let mut vEntMins: vec3_t = [0.0; 3];
    let mut vEntMaxs: vec3_t = [0.0; 3];
    // C leaves vVehLeaveDir uninitialised; only some switch arms write it (the
    // VEH_EJECT_BOTTOM arm leaves it untouched). Zero-init here — a benign deviation
    // (the degenerate bottom-eject would otherwise normalise stack garbage).
    let mut vVehLeaveDir: vec3_t = [0.0; 3];
    let mut vVehAngles: vec3_t = [0.0; 3];
    let mut m_ExitTrace: trace_t = trace_t::default();

    // Make sure that the entity is not 'stuck' inside the vehicle (since their bboxes will now intersect).
    // This makes the entity leave the vehicle from the right side.
    VectorSet(&mut vVehAngles, 0.0, (*parent).r.currentAngles[YAW], 0.0);
    match ejectDir {
        // Left.
        VEH_EJECT_LEFT => {
            AngleVectors(&vVehAngles, None, Some(&mut vVehLeaveDir), None);
            vVehLeaveDir[0] = -vVehLeaveDir[0];
            vVehLeaveDir[1] = -vVehLeaveDir[1];
            vVehLeaveDir[2] = -vVehLeaveDir[2];
        }
        // Right.
        VEH_EJECT_RIGHT => {
            AngleVectors(&vVehAngles, None, Some(&mut vVehLeaveDir), None);
        }
        // Front.
        VEH_EJECT_FRONT => {
            AngleVectors(&vVehAngles, Some(&mut vVehLeaveDir), None, None);
        }
        // Rear.
        VEH_EJECT_REAR => {
            AngleVectors(&vVehAngles, Some(&mut vVehLeaveDir), None, None);
            vVehLeaveDir[0] = -vVehLeaveDir[0];
            vVehLeaveDir[1] = -vVehLeaveDir[1];
            vVehLeaveDir[2] = -vVehLeaveDir[2];
        }
        // Top.
        VEH_EJECT_TOP => {
            AngleVectors(&vVehAngles, None, None, Some(&mut vVehLeaveDir));
        }
        // Bottom?.
        VEH_EJECT_BOTTOM => {}
        _ => {}
    }
    VectorNormalize(&mut vVehLeaveDir);
    //NOTE: not sure why following line was needed - MCG
    //pVeh->m_EjectDir = VEH_EJECT_LEFT;

    // Since (as of this time) the collidable geometry of the entity is just an axis
    // aligned box, we need to get the diagonal length of it in case we come out on that side.
    // Diagonal Length == squareroot( squared( Sidex / 2 ) + squared( Sidey / 2 ) );

    // TODO: DO diagonal for entity.

    fBias = 1.0f32;
    if (*(*pVeh).m_pVehicleInfo).r#type == VH_WALKER {
        //hacktastic!
        fBias += 0.2f32;
    }
    VectorCopy(&(*ent).r.currentOrigin, &mut *vExitPos);
    fVehDiag = (((*parent).r.maxs[0] * (*parent).r.maxs[0])
        + ((*parent).r.maxs[1] * (*parent).r.maxs[1]))
        .sqrt();
    VectorCopy(&(*ent).r.maxs, &mut vEntMaxs);
    if (*ent).s.number < MAX_CLIENTS as c_int {
        //for some reason, in MP, player client mins and maxs are never stored permanently, just set to these hardcoded numbers in PMove
        vEntMaxs[0] = 15.0;
        vEntMaxs[1] = 15.0;
    }
    fEntDiag = ((vEntMaxs[0] * vEntMaxs[0]) + (vEntMaxs[1] * vEntMaxs[1])).sqrt();
    vVehLeaveDir[0] *= (fVehDiag + fEntDiag) * fBias; // x
    vVehLeaveDir[1] *= (fVehDiag + fEntDiag) * fBias; // y
    vVehLeaveDir[2] *= (fVehDiag + fEntDiag) * fBias;
    VectorAdd(&(*vExitPos).clone(), &vVehLeaveDir, &mut *vExitPos);

    //we actually could end up *not* getting off if the trace fails...
    // Check to see if this new position is a valid place for our entity to go.
    VectorSet(&mut vEntMins, -15.0f32, -15.0f32, DEFAULT_MINS_2 as f32);
    VectorSet(&mut vEntMaxs, 15.0f32, 15.0f32, DEFAULT_MAXS_2 as f32);
    oldOwner = (*ent).r.ownerNum;
    (*ent).r.ownerNum = ENTITYNUM_NONE;
    G_VehicleTrace(
        &mut m_ExitTrace as *mut trace_t,
        &(*ent).r.currentOrigin,
        &vEntMins,
        &vEntMaxs,
        &*vExitPos,
        (*ent).s.number,
        (*ent).clipmask,
    );
    (*ent).r.ownerNum = oldOwner;

    if m_ExitTrace.allsolid != 0 //in solid
        || m_ExitTrace.startsolid != 0
    {
        return QFALSE;
    }
    // If the trace hit something, we can't go there!
    if m_ExitTrace.fraction < 1.0f32 {
        //not totally clear
        //		if ( (parent->clipmask&ent->r.contents) )//vehicle could actually get stuck on body
        {
            //the trace hit the vehicle, don't let them get out, just in case
            return QFALSE;
        }
        // C then has `//otherwise, use the trace.endpos\n VectorCopy( m_ExitTrace.endpos,
        // vExitPos );` — but in MP the guard above is unconditional, so that line is dead code
        // (unreachable). Omitted here to avoid an `unreachable_code` warning.
    }
    QTRUE
}

/// `bool Eject( Vehicle_t *pVeh, bgEntity_t *pEnt, qboolean forceEject )` (g_vehicles.c:728) —
/// removes `pEnt` from the vehicle: handles the droid unit, tainted/disconnected and dead
/// riders, finds a clear exit position by cycling eject directions through `VEH_TryEject`,
/// repositions the rider, clears the pilot/passenger slot (promoting the first passenger to
/// pilot if needed), un-hides the rider, silences the vehicle when empty, resets the rider's
/// view/anim timers, and arms the re-board cooldown.
///
/// The MP (`_JK2MP`/`QAGAME`) paths are taken; the SP-only cgame bits (`CG_ChangeWeapon`/
/// `G_RemoveWeaponModels`/`cg.overrides`/the trailing `SetClientViewAngle`) are `#ifndef
/// _JK2MP` and excluded. Sibling calls go through the installed vtable
/// (`SetPilot`/`UnGhost`). The C `goto getItOutOfMe` (tainted-rider fast path) is modelled
/// with a `goto_get_it_out` flag guarding the validation/exit-position section.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` with valid `m_pParentEntity`/`m_pVehicleInfo` and an
/// installed vtable; `pEnt` may be null. Live riders must have a non-null `client`.
pub unsafe extern "C" fn Eject(
    pVeh: *mut Vehicle_t,
    pEnt: *mut bgEntity_t,
    forceEject: qboolean,
) -> qboolean {
    let parent: *mut gentity_t;
    let mut vExitPos: vec3_t = [0.0; 3];
    let ent = pEnt as *mut gentity_t;
    let firstEjectDir: c_int;

    let mut taintedRider: qboolean = QFALSE;
    let mut deadRider: qboolean = QFALSE;
    let mut goto_get_it_out = false;

    if pEnt == (*pVeh).m_pDroidUnit {
        G_EjectDroidUnit(pVeh, QFALSE);
        return QTRUE;
    }

    if !ent.is_null() {
        if (*ent).inuse == QFALSE
            || (*ent).client.is_null()
            || (*(*ent).client).pers.connected != CON_CONNECTED
        {
            taintedRider = QTRUE;
            parent = (*pVeh).m_pParentEntity as *mut gentity_t;
            goto_get_it_out = true;
        } else {
            if (*ent).health < 1 {
                deadRider = QTRUE;
            }
            parent = (*pVeh).m_pParentEntity as *mut gentity_t; // set below at the C `parent = ...` once past validation
        }
    } else {
        parent = (*pVeh).m_pParentEntity as *mut gentity_t;
    }

    if !goto_get_it_out {
        // Validate.
        if ent.is_null() {
            return QFALSE;
        }
        if forceEject == QFALSE
            && !((*pVeh).m_iBoarding == 0
                || (*pVeh).m_iBoarding == -999
                || ((*pVeh).m_iBoarding < -3 && (*pVeh).m_iBoarding >= -9))
        {
            //I don't care, if he's dead get him off even if he died while boarding
            deadRider = QTRUE;
            (*pVeh).m_iBoarding = 0;
            (*pVeh).m_bWasBoarding = QFALSE;
        }

        // (SP-only `#ifndef _JK2MP`: CG_ChangeWeapon/ps.weapon=WP_NONE/G_RemoveWeaponModels —
        //  excluded in the MP build.)

        // parent already resolved above (C does `parent = pVeh->m_pParentEntity` here).

        //Try ejecting in every direction
        if (*pVeh).m_EjectDir < VEH_EJECT_LEFT {
            (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
        } else if (*pVeh).m_EjectDir > VEH_EJECT_BOTTOM {
            (*pVeh).m_EjectDir = VEH_EJECT_BOTTOM;
        }
        firstEjectDir = (*pVeh).m_EjectDir;
        while VEH_TryEject(pVeh, parent, ent, (*pVeh).m_EjectDir, &mut vExitPos) == QFALSE {
            (*pVeh).m_EjectDir += 1;
            if (*pVeh).m_EjectDir > VEH_EJECT_BOTTOM {
                (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
            }
            if (*pVeh).m_EjectDir == firstEjectDir {
                //they all failed
                if deadRider == QFALSE {
                    //if he's dead.. just shove him in solid, who cares.
                    return QFALSE;
                }
                if forceEject != QFALSE {
                    //we want to always get out, just eject him here
                    VectorCopy(&(*ent).r.currentOrigin, &mut vExitPos);
                    break;
                } else {
                    //can't eject
                    return QFALSE;
                }
            }
        }

        // Move them to the exit position.
        G_SetOrigin(ent, &vExitPos);
        VectorCopy(&(*ent).r.currentOrigin, &mut (*(*ent).client).ps.origin);
        trap::LinkEntity(ent);

        // If it's the player, stop overrides.
        if (*ent).s.number < MAX_CLIENTS as c_int {
            // (SP-only: cg.overrides.active = 0;)
        }
    }

    // getItOutOfMe: (C label — tainted riders jump straight here)

    // If he's the pilot...
    if (*pVeh).m_pPilot == ent as *mut bgEntity_t {
        let mut j: c_int = 0;

        (*pVeh).m_pPilot = null_mut();
        (*parent).r.ownerNum = ENTITYNUM_NONE;
        (*parent).s.owner = (*parent).r.ownerNum; //for prediction

        //keep these current angles
        //SetClientViewAngle( parent, pVeh->m_vOrientation );
        write_bytes(&mut (*(*parent).client).pers.cmd, 0, 1);
        write_bytes(&mut (*pVeh).m_ucmd, 0, 1);

        //if there are some passengers, promote the first passenger to pilot
        while j < (*pVeh).m_iNumPassengers {
            if !(*pVeh).m_ppPassengers[j as usize].is_null() {
                let mut k: c_int = 1;
                ((*(*pVeh).m_pVehicleInfo).SetPilot.unwrap())(
                    pVeh,
                    (*pVeh).m_ppPassengers[j as usize],
                );
                (*parent).r.ownerNum = (*((*pVeh).m_ppPassengers[j as usize] as *mut gentity_t))
                    .s
                    .number;
                (*parent).s.owner = (*parent).r.ownerNum; //for prediction
                (*(*parent).client).ps.m_iVehicleNum = (*((*pVeh).m_ppPassengers[j as usize]
                    as *mut gentity_t))
                    .s
                    .number
                    + 1;

                //rearrange the passenger slots now..
                //Server just needs to tell client he's not a passenger anymore
                if !(*((*pVeh).m_ppPassengers[j as usize] as *mut gentity_t))
                    .client
                    .is_null()
                {
                    (*(*((*pVeh).m_ppPassengers[j as usize] as *mut gentity_t)).client)
                        .ps
                        .generic1 = 0;
                }
                (*pVeh).m_ppPassengers[j as usize] = null_mut();
                while k < (*pVeh).m_iNumPassengers {
                    if (*pVeh).m_ppPassengers[(k - 1) as usize].is_null() {
                        //move down
                        (*pVeh).m_ppPassengers[(k - 1) as usize] =
                            (*pVeh).m_ppPassengers[k as usize];
                        (*pVeh).m_ppPassengers[k as usize] = null_mut();
                        //Server just needs to tell client which passenger he is
                        if !(*((*pVeh).m_ppPassengers[(k - 1) as usize] as *mut gentity_t))
                            .client
                            .is_null()
                        {
                            (*(*((*pVeh).m_ppPassengers[(k - 1) as usize] as *mut gentity_t))
                                .client)
                                .ps
                                .generic1 = k;
                        }
                    }
                    k += 1;
                }
                (*pVeh).m_iNumPassengers -= 1;

                break;
            }
            j += 1;
        }
    } else if ent == (*pVeh).m_pOldPilot as *mut gentity_t {
        (*pVeh).m_pOldPilot = null_mut();
    } else {
        let mut i: c_int = 0;
        // Look for this guy in the passenger list.
        while i < (*(*pVeh).m_pVehicleInfo).maxPassengers {
            // If we found him...
            if (*pVeh).m_ppPassengers[i as usize] as *mut gentity_t == ent {
                //Server just needs to tell client he's not a passenger anymore
                if !(*((*pVeh).m_ppPassengers[i as usize] as *mut gentity_t))
                    .client
                    .is_null()
                {
                    (*(*((*pVeh).m_ppPassengers[i as usize] as *mut gentity_t)).client)
                        .ps
                        .generic1 = 0;
                }
                (*pVeh).m_ppPassengers[i as usize] = null_mut();
                (*pVeh).m_iNumPassengers -= 1;
                break;
            }
            i += 1;
        }

        // Didn't find him, can't eject because they aren't in the vehicle (hopefully)!
        if i == (*(*pVeh).m_pVehicleInfo).maxPassengers {
            return QFALSE;
        }
    }

    //I hate adding these!
    if taintedRider == QFALSE {
        if (*(*pVeh).m_pVehicleInfo).hideRider != QFALSE {
            ((*(*pVeh).m_pVehicleInfo).UnGhost.unwrap())(pVeh, ent as *mut bgEntity_t);
        }
    }

    // If the vehicle now has no pilot...
    if (*pVeh).m_pPilot.is_null() {
        (*(*parent).client).ps.loopSound = 0;
        (*parent).s.loopSound = 0;
        // Completely empty vehicle...?
        if (*pVeh).m_iNumPassengers == 0 {
            (*(*parent).client).ps.m_iVehicleNum = 0;
        }
    }

    if taintedRider != QFALSE {
        //you can go now
        (*pVeh).m_iBoarding = (*addr_of!(level)).time + 1000;
        return QTRUE;
    }

    // Client not in a vehicle.
    (*(*ent).client).ps.m_iVehicleNum = 0;
    (*ent).r.ownerNum = ENTITYNUM_NONE;
    (*ent).s.owner = (*ent).r.ownerNum; //for prediction

    (*(*ent).client).ps.viewangles[PITCH] = 0.0f32;
    (*(*ent).client).ps.viewangles[ROLL] = 0.0f32;
    (*(*ent).client).ps.viewangles[YAW] = *(*pVeh).m_vOrientation.add(YAW);
    SetClientViewAngle(ent, &(*(*ent).client).ps.viewangles.clone());

    if (*(*ent).client).solidHack != 0 {
        (*(*ent).client).solidHack = 0;
        (*ent).r.contents = CONTENTS_BODY;
    }
    (*ent).s.m_iVehicleNum = 0;

    // Jump out.
    /*	if ( ent->client->ps.velocity[2] < JUMP_VELOCITY )
    {
        ent->client->ps.velocity[2] = JUMP_VELOCITY;
    }
    else
    {
        ent->client->ps.velocity[2] += JUMP_VELOCITY;
    }*/

    // Make sure entity is facing the direction it got off at.
    // (SP-only `#ifndef _JK2MP`: VectorCopy m_vOrientation + SetClientViewAngle — excluded.)

    //if was using vehicle weapon, remove it and switch to normal weapon when hop out...
    if (*(*ent).client).ps.weapon == WP_NONE {
        //FIXME: check against this vehicle's gun from the g_vehicleInfo table
        //remove the vehicle's weapon from me
        //ent->client->ps.stats[STAT_WEAPONS] &= ~( 1 << WP_EMPLACED_GUN );
        //ent->client->ps.ammo[weaponData[WP_EMPLACED_GUN].ammoIndex] = 0;//maybe store this ammo on the vehicle before clearing it?
        //switch back to a normal weapon we're carrying

        //FIXME: store the weapon we were using when we got on and restore that when hop off
    } else {
        //FIXME: if they have their saber out:
        //if dualSabers, add the second saber into the left hand
        //saber[0] has more than one blade, turn them all on
        //NOTE: this is because you're only allowed to use your first saber's first blade on a vehicle
    }

    /*	if ( !ent->s.number && ent->client->ps.weapon != WP_SABER
        && cg_gunAutoFirst.value )
    {
        gi.cvar_set( "cg_thirdperson", "0" );
    }*/
    BG_SetLegsAnimTimer(&mut (*(*ent).client).ps, 0);
    BG_SetTorsoAnimTimer(&mut (*(*ent).client).ps, 0);

    // Set how long until this vehicle can be boarded again.
    (*pVeh).m_iBoarding = (*addr_of!(level)).time + 1000;

    QTRUE
}

/// `bool EjectAll( Vehicle_t *pVeh )` (g_vehicles.c:1086) — force-ejects every occupant
/// (pilot, old pilot, all passengers) via the vtable `Eject`, killing each on the way out if
/// `killRiderOnDeath` is set, then detaches the droid unit. Sets the eject direction to TOP
/// and clears any in-progress boarding first.
///
/// MP/`QAGAME` paths taken. `bool`/`true`/`false` are `#define`d to `qboolean`/`qtrue`/`qfalse`.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` with valid `m_pVehicleInfo` and an installed vtable;
/// any occupant pointers must be valid `gentity_t*`.
pub unsafe extern "C" fn EjectAll(pVeh: *mut Vehicle_t) -> qboolean {
    // TODO: Setup a default escape for ever vehicle type.

    (*pVeh).m_EjectDir = VEH_EJECT_TOP;
    // Make sure no other boarding calls exist. We MUST exit.
    (*pVeh).m_iBoarding = 0;
    (*pVeh).m_bWasBoarding = QFALSE;

    // Throw them off.
    if !(*pVeh).m_pPilot.is_null() {
        let pilot = (*pVeh).m_pPilot as *mut gentity_t;
        ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, (*pVeh).m_pPilot, QTRUE);
        if (*(*pVeh).m_pVehicleInfo).killRiderOnDeath != QFALSE && !pilot.is_null() {
            //Kill them, too
            //FIXME: proper origin, MOD and attacker (for credit/death message)?  Get from vehicle?
            G_MuteSound((*pilot).s.number, CHAN_VOICE);
            G_Damage(
                pilot,
                null_mut(),
                null_mut(),
                null_mut(),
                &mut (*pilot).s.origin,
                10000,
                0,
                MOD_SUICIDE,
            );
        }
    }
    if !(*pVeh).m_pOldPilot.is_null() {
        let pilot = (*pVeh).m_pOldPilot as *mut gentity_t;
        ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, (*pVeh).m_pOldPilot, QTRUE);
        if (*(*pVeh).m_pVehicleInfo).killRiderOnDeath != QFALSE && !pilot.is_null() {
            //Kill them, too
            //FIXME: proper origin, MOD and attacker (for credit/death message)?  Get from vehicle?
            G_MuteSound((*pilot).s.number, CHAN_VOICE);
            G_Damage(
                pilot,
                null_mut(),
                null_mut(),
                null_mut(),
                &mut (*pilot).s.origin,
                10000,
                0,
                MOD_SUICIDE,
            );
        }
    }
    if (*pVeh).m_iNumPassengers != 0 {
        let mut i: c_int = 0;

        while i < (*(*pVeh).m_pVehicleInfo).maxPassengers {
            if !(*pVeh).m_ppPassengers[i as usize].is_null() {
                let rider = (*pVeh).m_ppPassengers[i as usize] as *mut gentity_t;
                ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(
                    pVeh,
                    (*pVeh).m_ppPassengers[i as usize],
                    QTRUE,
                );
                if (*(*pVeh).m_pVehicleInfo).killRiderOnDeath != QFALSE && !rider.is_null() {
                    //Kill them, too
                    //FIXME: proper origin, MOD and attacker (for credit/death message)?  Get from vehicle?
                    G_MuteSound((*rider).s.number, CHAN_VOICE);
                    G_Damage(
                        rider,
                        null_mut(),
                        null_mut(),
                        null_mut(),
                        &mut (*rider).s.origin,
                        10000,
                        0,
                        MOD_SUICIDE,
                    ); //FIXME: proper MOD?  Get from vehicle?
                }
            }
            i += 1;
        }
        (*pVeh).m_iNumPassengers = 0;
    }

    if !(*pVeh).m_pDroidUnit.is_null() {
        G_EjectDroidUnit(pVeh, (*(*pVeh).m_pVehicleInfo).killRiderOnDeath);
    }

    QTRUE
}

/// `static void DeathUpdate( Vehicle_t *pVeh )` (g_vehicles.c:1194) — once the death-delay
/// timer expires, ejects (and if still occupied, explosively kills) everyone aboard, then —
/// if fully empty — plays the explode effect, drops an explosion mark via a ground trace,
/// deals radius damage, and queues the parent entity for removal next frame.
///
/// MP/`QAGAME` paths taken; the SP-only blocks (`noRagTime`, `G_StopEffect` loops, the
/// string-form `G_PlayEffect`, `Q_irand`/`AddSoundEvent`/`AddSightEvent`) are excluded. The
/// in-file `#define MOD_EXPLOSIVE MOD_SUICIDE` is applied (so `MOD_SUICIDE` is used here).
/// `Inhabited`/`EjectAll` are called through the installed vtable.
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` with valid `m_pParentEntity`/`m_pVehicleInfo` and an
/// installed vtable; occupant pointers must be valid `gentity_t*`.
pub unsafe extern "C" fn DeathUpdate(pVeh: *mut Vehicle_t) {
    let parent = (*pVeh).m_pParentEntity as *mut gentity_t;

    if (*addr_of!(level)).time >= (*pVeh).m_iDieTime {
        // If the vehicle is not empty.
        if ((*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) != QFALSE {
            ((*(*pVeh).m_pVehicleInfo).EjectAll.unwrap())(pVeh);
            if ((*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) != QFALSE {
                //if we've still got people in us, just kill the bastards
                if !(*pVeh).m_pPilot.is_null() {
                    //FIXME: does this give proper credit to the enemy who shot you down?
                    G_Damage(
                        (*pVeh).m_pPilot as *mut gentity_t,
                        (*pVeh).m_pParentEntity as *mut gentity_t,
                        (*pVeh).m_pParentEntity as *mut gentity_t,
                        null_mut(),
                        &mut (*(*(*pVeh).m_pParentEntity).playerState).origin,
                        999,
                        DAMAGE_NO_PROTECTION,
                        MOD_SUICIDE, // #define MOD_EXPLOSIVE MOD_SUICIDE
                    );
                }
                if (*pVeh).m_iNumPassengers != 0 {
                    let mut i: c_int = 0;

                    while i < (*(*pVeh).m_pVehicleInfo).maxPassengers {
                        if !(*pVeh).m_ppPassengers[i as usize].is_null() {
                            //FIXME: does this give proper credit to the enemy who shot you down?
                            G_Damage(
                                (*pVeh).m_ppPassengers[i as usize] as *mut gentity_t,
                                (*pVeh).m_pParentEntity as *mut gentity_t,
                                (*pVeh).m_pParentEntity as *mut gentity_t,
                                null_mut(),
                                &mut (*(*(*pVeh).m_pParentEntity).playerState).origin,
                                999,
                                DAMAGE_NO_PROTECTION,
                                MOD_SUICIDE, // #define MOD_EXPLOSIVE MOD_SUICIDE
                            );
                        }
                        i += 1;
                    }
                }
            }
        }

        if ((*(*pVeh).m_pVehicleInfo).Inhabited.unwrap())(pVeh) == QFALSE {
            //explode now as long as we managed to kick everyone out
            let mut lMins: vec3_t = [0.0; 3];
            let mut lMaxs: vec3_t = [0.0; 3];
            let mut bottom: vec3_t = [0.0; 3];
            let mut trace: trace_t = trace_t::default();

            if (*(*pVeh).m_pVehicleInfo).iExplodeFX != 0 {
                let mut fxAng: vec3_t = [0.0; 3];

                VectorSet(&mut fxAng, -90.0f32, 0.0f32, 0.0f32);
                G_PlayEffectID(
                    (*(*pVeh).m_pVehicleInfo).iExplodeFX,
                    &(*parent).r.currentOrigin,
                    &fxAng,
                );
                //trace down and place mark
                VectorCopy(&(*parent).r.currentOrigin, &mut bottom);
                bottom[2] -= 80.0;
                G_VehicleTrace(
                    &mut trace as *mut trace_t,
                    &(*parent).r.currentOrigin,
                    &vec3_origin,
                    &vec3_origin,
                    &bottom,
                    (*parent).s.number,
                    CONTENTS_SOLID,
                );
                if trace.fraction < 1.0f32 {
                    VectorCopy(&trace.endpos, &mut bottom);
                    bottom[2] += 2.0;
                    VectorSet(&mut fxAng, -90.0f32, 0.0f32, 0.0f32);
                    G_PlayEffectID(
                        G_EffectIndex("ships/ship_explosion_mark"),
                        &trace.endpos,
                        &fxAng,
                    );
                }
            }

            (*parent).takedamage = QFALSE; //so we don't recursively damage ourselves
            if (*(*pVeh).m_pVehicleInfo).explosionRadius > 0.0
                && (*(*pVeh).m_pVehicleInfo).explosionDamage > 0
            {
                VectorCopy(&(*parent).r.mins, &mut lMins);
                lMins[2] = -4.0; //to keep it off the ground a *little*
                VectorCopy(&(*parent).r.maxs, &mut lMaxs);
                VectorCopy(&(*parent).r.currentOrigin, &mut bottom);
                bottom[2] += (*parent).r.mins[2] - 32.0;
                G_VehicleTrace(
                    &mut trace as *mut trace_t,
                    &(*parent).r.currentOrigin,
                    &lMins,
                    &lMaxs,
                    &bottom,
                    (*parent).s.number,
                    CONTENTS_SOLID,
                );
                G_RadiusDamage(
                    &trace.endpos,
                    parent,
                    (*(*pVeh).m_pVehicleInfo).explosionDamage as f32,
                    (*(*pVeh).m_pVehicleInfo).explosionRadius,
                    null_mut(),
                    null_mut(),
                    MOD_VEH_EXPLOSION,
                ); //FIXME: extern damage and radius or base on fuel
            }

            (*parent).think = Some(G_FreeEntity);
            (*parent).nextthink = (*addr_of!(level)).time + FRAMETIME;
        }
    }
}

// `#ifdef _JK2MP //we want access to this one clientside, but it's the only
// //generic vehicle function we care about over there
// extern void AttachRidersGeneric( Vehicle_t *pVeh );` — ported as
// [`crate::codemp::game::bg_vehicleLoad::AttachRidersGeneric`].

// Attachs all the riders of this vehicle to their appropriate tag (*driver, *pass1, *pass2, whatever...).
/// `static void AttachRiders( Vehicle_t *pVeh )` (g_vehicles.c:2318) — re-positions every rider
/// (pilot, old pilot, passengers, droid) at its bolt-tag on the vehicle model each frame. The MP
/// `#ifdef _JK2MP` path: defer the bolt math to [`AttachRidersGeneric`] for the pilots, then for
/// each passenger and the droid recompute the `*driver`/droid-tag matrix off the parent's
/// yaw-only viewangles and snap the rider's origin (and, for the droid, view-angles + stand
/// anim) there, relinking. The SP `#else` G2-call path is excluded (MP build only). No oracle
/// (entity/G2-trap state).
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*`; its `m_pParentEntity`/riders, where non-null, must be
/// valid `gentity_t*` with valid `client`/`ghoul2` as the C `assert`s require.
pub unsafe extern "C" fn AttachRiders(pVeh: *mut Vehicle_t) {
    let mut i: c_int = 0;

    AttachRidersGeneric(pVeh);

    if !(*pVeh).m_pPilot.is_null() {
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let pilot = (*pVeh).m_pPilot as *mut gentity_t;
        (*pilot).waypoint = (*parent).waypoint; // take the veh's waypoint as your own

        //assuming we updated him relative to the bolt in AttachRidersGeneric
        G_SetOrigin(pilot, &(*(*pilot).client).ps.origin.clone());
        trap::LinkEntity(pilot);
    }

    if !(*pVeh).m_pOldPilot.is_null() {
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let oldpilot = (*pVeh).m_pOldPilot as *mut gentity_t;
        (*oldpilot).waypoint = (*parent).waypoint; // take the veh's waypoint as your own

        //assuming we updated him relative to the bolt in AttachRidersGeneric
        G_SetOrigin(oldpilot, &(*(*oldpilot).client).ps.origin.clone());
        trap::LinkEntity(oldpilot);
    }

    //attach passengers
    while i < (*pVeh).m_iNumPassengers {
        if !(*pVeh).m_ppPassengers[i as usize].is_null() {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut yawOnlyAngles: vec3_t = [0.0; 3];
            let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
            let pilot = (*pVeh).m_ppPassengers[i as usize] as *mut gentity_t;
            let crotchBolt: c_int;

            debug_assert!(!(*parent).ghoul2.is_null());
            crotchBolt = trap::G2API_AddBolt((*parent).ghoul2, 0, "*driver");
            debug_assert!(!(*parent).client.is_null());
            debug_assert!(!(*pilot).client.is_null());

            VectorSet(
                &mut yawOnlyAngles,
                0.0,
                (*(*parent).client).ps.viewangles[YAW],
                0.0,
            );

            // Get the driver tag.
            trap::G2API_GetBoltMatrix(
                (*parent).ghoul2,
                0,
                crotchBolt,
                &mut boltMatrix,
                &yawOnlyAngles,
                &(*(*parent).client).ps.origin,
                (*addr_of!(level)).time,
                core::ptr::null_mut(),
                &(*parent).modelScale,
            );
            BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut (*(*pilot).client).ps.origin);

            G_SetOrigin(pilot, &(*(*pilot).client).ps.origin.clone());
            trap::LinkEntity(pilot);
        }
        i += 1;
    }

    //attach droid
    if !(*pVeh).m_pDroidUnit.is_null() && (*pVeh).m_iDroidUnitTag != -1 {
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut yawOnlyAngles: vec3_t = [0.0; 3];
        let mut fwd: vec3_t = [0.0; 3];
        let parent = (*pVeh).m_pParentEntity as *mut gentity_t;
        let droid = (*pVeh).m_pDroidUnit as *mut gentity_t;

        debug_assert!(!(*parent).ghoul2.is_null());
        debug_assert!(!(*parent).client.is_null());
        //assert(droid->client);

        if !(*droid).client.is_null() {
            VectorSet(
                &mut yawOnlyAngles,
                0.0,
                (*(*parent).client).ps.viewangles[YAW],
                0.0,
            );

            // Get the droid tag.
            trap::G2API_GetBoltMatrix(
                (*parent).ghoul2,
                0,
                (*pVeh).m_iDroidUnitTag,
                &mut boltMatrix,
                &yawOnlyAngles,
                &(*parent).r.currentOrigin,
                (*addr_of!(level)).time,
                core::ptr::null_mut(),
                &(*parent).modelScale,
            );
            BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut (*(*droid).client).ps.origin);
            BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut fwd);
            vectoangles(&fwd, &mut (*(*droid).client).ps.viewangles);

            G_SetOrigin(droid, &(*(*droid).client).ps.origin.clone());
            G_SetAngles(droid, &(*(*droid).client).ps.viewangles.clone());
            SetClientViewAngle(droid, &(*(*droid).client).ps.viewangles.clone());
            trap::LinkEntity(droid);

            if !(*droid).NPC.is_null() {
                NPC_SetAnim(
                    droid,
                    SETANIM_BOTH,
                    BOTH_STAND2,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                (*(*droid).client).ps.legsTimer = 500;
                (*(*droid).client).ps.torsoTimer = 500;
            }
        }
    }
}

/// `bool Update( Vehicle_t *pVeh, const usercmd_t *pUmcd )` (g_vehicles.c:1472) —
/// the per-frame base vehicle-update driver: recharges weapon/turret/shield ammo, handles the
/// dieing/dead transitions, the disconnect-while-boarding pilot death checks, rider eject of
/// dead riders, weapon link/unlink toggling, the turret think loop, then runs the
/// orient/move-command processors, shifting-sound debounce, move-direction setup and surface
/// destruction damage.
///
/// MP/`_JK2MP`+`QAGAME` build: the C uses `goto maintainSelfDuringBoarding;` to skip the whole
/// middle section (parent revalidation, rider death eject, ucmd copy, weapon-link loop, turret
/// think) when boarding isn't finished, then falls through to the shared tail. There is no
/// `unsafe` goto here — a `skip_middle` flag guards that middle block; the label block + tail
/// run unconditionally afterwards (both on the skip and the normal fall-through).
///
/// SP / `#ifndef _JK2MP` / CGAME branches are omitted: the boarding-not-done `return false`
/// (we take the goto/skip side), the "always knock guys around" knockback block
/// (`G_VehicleTrace`/`G_Throw`/`G_Knockdown`), the non-connected SP death-check variants, the
/// SP line-of-sight spawnflags block, the `parent->enemy = pilot->enemy` tail, and the
/// `G_SoundIndexOnEnt` shift-sound playback (MP keeps only the `// TODO: MP Shift Sound
/// Playback` placeholder — no sound emitted). no-oracle (per-frame vehicle update — mutates
/// entity/playerState/Vehicle_t state and calls the vtable processors).
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` with valid `m_pParentEntity` (whose `playerState` and
/// `client` are non-null), `m_pVehicleInfo` (with an installed vtable), and `m_vOrientation`;
/// `pUcmd` must be a valid `usercmd_t*`.
pub unsafe extern "C" fn Update(pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean {
    let mut parent: *mut gentity_t = (*pVeh).m_pParentEntity as *mut gentity_t;
    //static float fMod = 1000.0f / 60.0f;
    let mut vVehAngles: vec3_t = [0.0; 3];
    let mut i: usize;
    let prevSpeed: c_int;
    let nextSpeed: c_int;
    let curTime: c_int;
    let halfMaxSpeed: c_int;
    let parentPS: *mut crate::codemp::game::q_shared_h::playerState_t;
    let mut linkHeld: qboolean = QFALSE;

    parentPS = (*(*pVeh).m_pParentEntity).playerState;

    curTime = (*addr_of!(level)).time;

    //increment the ammo for all rechargeable weapons
    i = 0;
    while i < MAX_VEHICLE_WEAPONS as usize {
        if (*(*pVeh).m_pVehicleInfo).weapon[i].ID > VEH_WEAPON_BASE //have a weapon in this slot
            && (*(*pVeh).m_pVehicleInfo).weapon[i].ammoRechargeMS != 0 //its ammo is rechargable
            && (*pVeh).weaponStatus[i].ammo < (*(*pVeh).m_pVehicleInfo).weapon[i].ammoMax //its ammo is below max
            && (*pUcmd).serverTime - (*pVeh).weaponStatus[i].lastAmmoInc
                >= (*(*pVeh).m_pVehicleInfo).weapon[i].ammoRechargeMS
        //enough time has passed
        {
            //add 1 to the ammo
            (*pVeh).weaponStatus[i].lastAmmoInc = (*pUcmd).serverTime;
            (*pVeh).weaponStatus[i].ammo += 1;
            //NOTE: in order to send the vehicle's ammo info to the client, we copy the ammo into the first 2 ammo slots on the vehicle NPC's client->ps.ammo array
            if !parent.is_null() && !(*parent).client.is_null() {
                (*(*parent).client).ps.ammo[i] = (*pVeh).weaponStatus[i].ammo;
            }
        }
        i += 1;
    }
    i = 0;
    while i < MAX_VEHICLE_TURRETS as usize {
        if (*(*pVeh).m_pVehicleInfo).turret[i].iWeapon > VEH_WEAPON_BASE //have a weapon in this slot
            && (*(*pVeh).m_pVehicleInfo).turret[i].iAmmoRechargeMS != 0 //its ammo is rechargable
            && (*pVeh).turretStatus[i].ammo < (*(*pVeh).m_pVehicleInfo).turret[i].iAmmoMax //its ammo is below max
            && (*pUcmd).serverTime - (*pVeh).turretStatus[i].lastAmmoInc
                >= (*(*pVeh).m_pVehicleInfo).turret[i].iAmmoRechargeMS
        //enough time has passed
        {
            //add 1 to the ammo
            (*pVeh).turretStatus[i].lastAmmoInc = (*pUcmd).serverTime;
            (*pVeh).turretStatus[i].ammo += 1;
            //NOTE: in order to send the vehicle's ammo info to the client, we copy the ammo into the first 2 ammo slots on the vehicle NPC's client->ps.ammo array
            if !parent.is_null() && !(*parent).client.is_null() {
                (*(*parent).client).ps.ammo[MAX_VEHICLE_WEAPONS as usize + i] =
                    (*pVeh).turretStatus[i].ammo;
            }
        }
        i += 1;
    }

    //increment shields for rechargable shields
    if (*(*pVeh).m_pVehicleInfo).shieldRechargeMS != 0
        && (*parentPS).stats[STAT_ARMOR as usize] > 0 //still have some shields left
        && (*parentPS).stats[STAT_ARMOR as usize] < (*(*pVeh).m_pVehicleInfo).shields //its below max
        && (*pUcmd).serverTime - (*pVeh).lastShieldInc >= (*(*pVeh).m_pVehicleInfo).shieldRechargeMS
    //enough time has passed
    {
        (*parentPS).stats[STAT_ARMOR as usize] += 1;
        if (*parentPS).stats[STAT_ARMOR as usize] > (*(*pVeh).m_pVehicleInfo).shields {
            (*parentPS).stats[STAT_ARMOR as usize] = (*(*pVeh).m_pVehicleInfo).shields;
        }
        (*pVeh).m_iShields = (*parentPS).stats[STAT_ARMOR as usize];
        G_VehUpdateShields(parent);
    }

    //sometimes this gets out of whack, probably init'ing
    if !parent.is_null() && (*parent).r.ownerNum != (*parent).s.owner {
        (*parent).s.owner = (*parent).r.ownerNum;
    }

    //keep the PS value in sync. set it up here in case we return below at some point.
    if (*pVeh).m_iBoarding != 0 {
        (*(*parent).client).ps.vehBoarding = QTRUE;
    } else {
        (*(*parent).client).ps.vehBoarding = QFALSE;
    }

    // See whether this vehicle should be dieing or dead.
    if (*pVeh).m_iDieTime != 0 {
        //NOTE!!!: This HAS to be consistent with cgame!!!
        // Keep track of the old orientation.
        let ori: vec3_t = *((*pVeh).m_vOrientation as *const vec3_t);
        VectorCopy(&ori, &mut (*pVeh).m_vPrevOrientation);

        // Process the orient commands.
        ((*(*pVeh).m_pVehicleInfo).ProcessOrientCommands.unwrap())(pVeh);
        // Need to copy orientation to our entity's viewangles so that it renders at the proper angle and currentAngles is correct.
        SetClientViewAngle(parent, &*((*pVeh).m_vOrientation as *const vec3_t));
        if !(*pVeh).m_pPilot.is_null() {
            SetClientViewAngle(
                (*pVeh).m_pPilot as *mut gentity_t,
                &*((*pVeh).m_vOrientation as *const vec3_t),
            );
        }
        /*
        for ( i = 0; i < pVeh->m_pVehicleInfo->maxPassengers; i++ )
        {
            if ( pVeh->m_ppPassengers[i] )
            {
                SetClientViewAngle( (gentity_t *)pVeh->m_ppPassengers[i], pVeh->m_vOrientation );
            }
        }
        */

        // Process the move commands.
        ((*(*pVeh).m_pVehicleInfo).ProcessMoveCommands.unwrap())(pVeh);

        // Setup the move direction.
        if (*(*pVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
            AngleVectors(
                &*((*pVeh).m_vOrientation as *const vec3_t),
                Some(&mut (*(*parent).client).ps.moveDir),
                None,
                None,
            );
        } else {
            VectorSet(&mut vVehAngles, 0.0, *(*pVeh).m_vOrientation.add(YAW), 0.0);
            AngleVectors(
                &vVehAngles,
                Some(&mut (*(*parent).client).ps.moveDir),
                None,
                None,
            );
        }
        ((*(*pVeh).m_pVehicleInfo).DeathUpdate.unwrap())(pVeh);
        return QFALSE;
    }
    // Vehicle dead!
    else if (*parent).health <= 0 {
        // Instant kill.
        if (*(*pVeh).m_pVehicleInfo).r#type == VH_FIGHTER && (*pVeh).m_iLastImpactDmg > 500 {
            //explode instantly in inferno-y death
            ((*(*pVeh).m_pVehicleInfo).StartDeathDelay.unwrap())(
                pVeh, -1, /* -1 causes instant death */
            );
        } else {
            ((*(*pVeh).m_pVehicleInfo).StartDeathDelay.unwrap())(pVeh, 0);
        }
        ((*(*pVeh).m_pVehicleInfo).DeathUpdate.unwrap())(pVeh);
        return QFALSE;
    }

    //special check in case someone disconnects/dies while boarding
    if (*parent).spawnflags & 1 != 0 {
        if !(*pVeh).m_pPilot.is_null() || (*pVeh).m_bHasHadPilot == QFALSE {
            if !(*pVeh).m_pPilot.is_null() && (*pVeh).m_bHasHadPilot == QFALSE {
                (*pVeh).m_bHasHadPilot = QTRUE;
                (*pVeh).m_iPilotLastIndex = (*(*pVeh).m_pPilot).s.number;
            }
            (*pVeh).m_iPilotTime = (*addr_of!(level)).time + (*parent).damage;
        } else if (*pVeh).m_iPilotTime != 0 {
            //die
            let oldPilot: *mut gentity_t = core::ptr::addr_of_mut!(g_entities)
                .cast::<gentity_t>()
                .add((*pVeh).m_iPilotLastIndex as usize);

            if (*oldPilot).inuse == QFALSE
                || (*oldPilot).client.is_null()
                || (*(*oldPilot).client).pers.connected != CON_CONNECTED
            {
                //no longer in the game?
                G_Damage(
                    parent,
                    parent,
                    parent,
                    null_mut(),
                    &mut (*(*parent).client).ps.origin,
                    99999,
                    DAMAGE_NO_PROTECTION,
                    MOD_SUICIDE,
                );
            } else {
                let mut v: vec3_t = [0.0; 3];
                VectorSubtract(
                    &(*(*parent).client).ps.origin,
                    &(*(*oldPilot).client).ps.origin,
                    &mut v,
                );

                if VectorLength(&v) < (*parent).speed {
                    //they are still within the minimum distance to their vehicle
                    (*pVeh).m_iPilotTime = (*addr_of!(level)).time + (*parent).damage;
                } else if (*pVeh).m_iPilotTime < (*addr_of!(level)).time {
                    //dying time
                    G_Damage(
                        parent,
                        parent,
                        parent,
                        null_mut(),
                        &mut (*(*parent).client).ps.origin,
                        99999,
                        DAMAGE_NO_PROTECTION,
                        MOD_SUICIDE,
                    );
                }
            }
        }
    }

    //special check in case someone disconnects/dies while boarding
    if (*pVeh).m_iBoarding != 0 {
        let pilotEnt: *mut gentity_t = (*pVeh).m_pPilot as *mut gentity_t;
        if !pilotEnt.is_null()
            && ((*pilotEnt).inuse == QFALSE
                || (*pilotEnt).client.is_null()
                || (*pilotEnt).health <= 0
                || (*(*pilotEnt).client).pers.connected != CON_CONNECTED)
        {
            ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, (*pVeh).m_pPilot, QTRUE);
            return QFALSE;
        }
    }

    let mut skip_middle: qboolean = QFALSE;
    // If we're not done mounting, can't do anything.
    if (*pVeh).m_iBoarding != 0 {
        if (*pVeh).m_bWasBoarding == QFALSE {
            VectorCopy(&(*parentPS).velocity, &mut (*pVeh).m_vBoardingVelocity);
            (*pVeh).m_bWasBoarding = QTRUE;
        }

        // See if we're done boarding.
        if (*pVeh).m_iBoarding > -1 && (*pVeh).m_iBoarding <= (*addr_of!(level)).time {
            (*pVeh).m_bWasBoarding = QFALSE;
            (*pVeh).m_iBoarding = 0;
        } else {
            //MP: goto maintainSelfDuringBoarding;
            skip_middle = QTRUE;
        }
    }

    if skip_middle == QFALSE {
        parent = (*pVeh).m_pParentEntity as *mut gentity_t;

        // Validate vehicle.
        if parent.is_null() || (*parent).client.is_null() || (*parent).health <= 0 {
            return QFALSE;
        }

        // See if any of the riders are dead and if so kick em off.
        if !(*pVeh).m_pPilot.is_null() {
            let pilotEnt: *mut gentity_t = (*pVeh).m_pPilot as *mut gentity_t;

            if (*pilotEnt).inuse == QFALSE
                || (*pilotEnt).client.is_null()
                || (*pilotEnt).health <= 0
                || (*(*pilotEnt).client).pers.connected != CON_CONNECTED
            {
                ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, (*pVeh).m_pPilot, QTRUE);
            }
        }
        // If we're not empty...
        if (*pVeh).m_iNumPassengers > 0 {
            // See if any of these suckers are dead.
            i = 0;
            while i < (*(*pVeh).m_pVehicleInfo).maxPassengers as usize {
                let psngr: *mut gentity_t = (*pVeh).m_ppPassengers[i] as *mut gentity_t;

                if !psngr.is_null()
                    && ((*psngr).inuse == QFALSE
                        || (*psngr).client.is_null()
                        || (*psngr).health <= 0
                        || (*(*psngr).client).pers.connected != CON_CONNECTED)
                {
                    ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(
                        pVeh,
                        (*pVeh).m_ppPassengers[i],
                        QTRUE,
                    );
                    (*pVeh).m_iNumPassengers -= 1;
                }
                i += 1;
            }
        }

        // Copy over the commands for local storage.
        (*(*parent).client).pers.cmd = (*pVeh).m_ucmd;
        (*pVeh).m_ucmd.buttons &= !BUTTON_TALK; //|BUTTON_GESTURE); //don't want some of these buttons

        /*
        // Update time modifier.
        pVeh->m_fTimeModifier = pVeh->m_ucmd.serverTime - parent->client->ps.commandTime;
        //sanity check
        if ( pVeh->m_fTimeModifier < 1 )
        {
            pVeh->m_fTimeModifier = 1;
        }
        else if ( pVeh->m_fTimeModifier > 200 )
        {
            pVeh->m_fTimeModifier = 200;
        }
        //normalize to 1.0f at 20fps
        pVeh->m_fTimeModifier = pVeh->m_fTimeModifier / fMod;
        */

        //check for weapon linking/unlinking command
        i = 0;
        while i < MAX_VEHICLE_WEAPONS as usize {
            //HMM... can't get a seperate command for each weapon, so do them all...?
            if (*(*pVeh).m_pVehicleInfo).weapon[i].linkable == 2 {
                //always linked
                //FIXME: just set this once, on Initialize...?
                if (*pVeh).weaponStatus[i].linked == QFALSE {
                    (*pVeh).weaponStatus[i].linked = QTRUE;
                }
            } else if (*pVeh).m_ucmd.buttons & BUTTON_USE_HOLDABLE != 0 {
                //pilot pressed the "weapon link" toggle button
                let _pilotPS: *mut crate::codemp::game::q_shared_h::playerState_t;
                let mut rider: *mut bgEntity_t = null_mut();
                if (*parent).s.owner != ENTITYNUM_NONE {
                    rider = PM_BGEntForNum((*parent).s.owner); //&g_entities[parent->r.ownerNum];
                }
                _pilotPS = (*rider).playerState;
                if (*pVeh).linkWeaponToggleHeld == QFALSE {
                    //so we don't hold it down and toggle it back and forth
                    //okay to toggle
                    if (*(*pVeh).m_pVehicleInfo).weapon[i].linkable == 1 {
                        //link-toggleable
                        (*pVeh).weaponStatus[i].linked = if (*pVeh).weaponStatus[i].linked != QFALSE
                        {
                            QFALSE
                        } else {
                            QTRUE
                        };
                    }
                }
                linkHeld = QTRUE;
            }
            i += 1;
        }
        if linkHeld != QFALSE {
            //so we don't hold it down and toggle it back and forth
            (*pVeh).linkWeaponToggleHeld = QTRUE;
        } else {
            //so we don't hold it down and toggle it back and forth
            (*pVeh).linkWeaponToggleHeld = QFALSE;
        }
        //now pass it over the network so cgame knows about it
        //NOTE: SP can just cheat and check directly
        (*parentPS).vehWeaponsLinked = QFALSE;
        i = 0;
        while i < MAX_VEHICLE_WEAPONS as usize {
            //HMM... can't get a seperate command for each weapon, so do them all...?
            if (*pVeh).weaponStatus[i].linked != QFALSE {
                (*parentPS).vehWeaponsLinked = QTRUE;
            }
            i += 1;
        }

        i = 0;
        while i < MAX_VEHICLE_TURRETS as usize {
            //HMM... can't get a seperate command for each weapon, so do them all...?
            VEH_TurretThink(pVeh, parent, i as c_int);
            i += 1;
        }
    }

    // maintainSelfDuringBoarding:
    if !(*pVeh).m_pPilot.is_null()
        && !(*(*pVeh).m_pPilot).playerState.is_null()
        && (*pVeh).m_iBoarding != 0
    {
        let ori: vec3_t = *((*pVeh).m_vOrientation as *const vec3_t);
        VectorCopy(&ori, &mut (*(*(*pVeh).m_pPilot).playerState).viewangles);
        (*pVeh).m_ucmd.buttons = 0;
        (*pVeh).m_ucmd.forwardmove = 0;
        (*pVeh).m_ucmd.rightmove = 0;
        (*pVeh).m_ucmd.upmove = 0;
    }

    // Keep track of the old orientation.
    let ori: vec3_t = *((*pVeh).m_vOrientation as *const vec3_t);
    VectorCopy(&ori, &mut (*pVeh).m_vPrevOrientation);

    // Process the orient commands.
    ((*(*pVeh).m_pVehicleInfo).ProcessOrientCommands.unwrap())(pVeh);
    // Need to copy orientation to our entity's viewangles so that it renders at the proper angle and currentAngles is correct.
    SetClientViewAngle(parent, &*((*pVeh).m_vOrientation as *const vec3_t));
    if !(*pVeh).m_pPilot.is_null() {
        if BG_UnrestrainedPitchRoll((*(*pVeh).m_pPilot).playerState, pVeh) == QFALSE {
            let mut newVAngle: vec3_t = [0.0; 3];
            newVAngle[PITCH] = (*(*(*pVeh).m_pPilot).playerState).viewangles[PITCH];
            newVAngle[YAW] = (*(*(*pVeh).m_pPilot).playerState).viewangles[YAW];
            newVAngle[ROLL] = *(*pVeh).m_vOrientation.add(ROLL);
            SetClientViewAngle((*pVeh).m_pPilot as *mut gentity_t, &newVAngle);
        }
    }
    /*
    for ( i = 0; i < pVeh->m_pVehicleInfo->maxPassengers; i++ )
    {
        if ( pVeh->m_ppPassengers[i] )
        {
            SetClientViewAngle( (gentity_t *)pVeh->m_ppPassengers[i], pVeh->m_vOrientation );
        }
    }
    */

    // Process the move commands.
    prevSpeed = (*parentPS).speed as c_int;
    ((*(*pVeh).m_pVehicleInfo).ProcessMoveCommands.unwrap())(pVeh);
    nextSpeed = (*parentPS).speed as c_int;
    halfMaxSpeed = ((*(*pVeh).m_pVehicleInfo).speedMax * 0.5f32) as c_int;

    // Shifting Sounds
    //=====================================================================
    if (*pVeh).m_iTurboTime < curTime
        && (*pVeh).m_iSoundDebounceTimer < curTime
        && ((nextSpeed > prevSpeed && nextSpeed > halfMaxSpeed && prevSpeed < halfMaxSpeed)
            || (nextSpeed > halfMaxSpeed && Q_irand(0, 1000) == 0))
    {
        let mut shiftSound: c_int = Q_irand(1, 4);
        match shiftSound {
            1 => shiftSound = (*(*pVeh).m_pVehicleInfo).soundShift1,
            2 => shiftSound = (*(*pVeh).m_pVehicleInfo).soundShift2,
            3 => shiftSound = (*(*pVeh).m_pVehicleInfo).soundShift3,
            4 => shiftSound = (*(*pVeh).m_pVehicleInfo).soundShift4,
            _ => {}
        }
        if shiftSound != 0 {
            (*pVeh).m_iSoundDebounceTimer = curTime + Q_irand(1000, 4000);
            // TODO: MP Shift Sound Playback
        }
    }
    //=====================================================================

    // Setup the move direction.
    if (*(*pVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
        AngleVectors(
            &*((*pVeh).m_vOrientation as *const vec3_t),
            Some(&mut (*(*parent).client).ps.moveDir),
            None,
            None,
        );
    } else {
        VectorSet(&mut vVehAngles, 0.0, *(*pVeh).m_vOrientation.add(YAW), 0.0);
        AngleVectors(
            &vVehAngles,
            Some(&mut (*(*parent).client).ps.moveDir),
            None,
            None,
        );
    }

    if (*(*pVeh).m_pVehicleInfo).surfDestruction != 0 {
        if (*pVeh).m_iRemovedSurfaces != 0 {
            let dmg: f32;
            G_VehicleDamageBoxSizing(pVeh);

            //damage him constantly if any chunks are currently taken off

            // 3 seconds max on death.
            dmg = (*(*parent).client).ps.stats[STAT_MAX_HEALTH as usize] as f32
                * (*pVeh).m_fTimeModifier
                / 180.0f32;
            //FIXME: aside from bypassing shields, maybe set m_iShields to 0, too... ?
            G_DamageFromKiller(
                parent,
                parent,
                parent,
                &mut (*(*parent).client).ps.origin,
                dmg as c_int,
                DAMAGE_NO_SELF_PROTECTION
                    | DAMAGE_NO_HIT_LOC
                    | DAMAGE_NO_PROTECTION
                    | DAMAGE_NO_ARMOR,
                MOD_SUICIDE,
            );
        }

        //make sure playerstate value stays in sync
        (*(*parent).client).ps.vehSurfaces = (*pVeh).m_iRemovedSurfaces;
    }

    //keep the PS value in sync
    if (*pVeh).m_iBoarding != 0 {
        (*(*parent).client).ps.vehBoarding = QTRUE;
    } else {
        (*(*parent).client).ps.vehBoarding = QFALSE;
    }

    QTRUE
}

/// `bool UpdateRider( Vehicle_t *pVeh, bgEntity_t *pRider, usercmd_t *pUmcd )`
/// (g_vehicles.c:2058) — per-frame rider-side update: copies the pilot's rocket-lock state down
/// to passengers, then handles every rider-initiated dismount (USE-key exit, strafe roll-off,
/// jump-off, roll-off), playing the matching animation and ejecting through the vtable `Eject`.
///
/// MP/`_JK2MP` build: takes the `trap::ICARUS_TaskIDPending` / `ps.fd.forceJumpZStart` /
/// `flags & FL_VEH_BOARDING` / `BG_AnimLength` branches. The SP branches are omitted —
/// `Q3_TaskIDPending`, `PM_AnimLength`, `ps.eFlags & EF_VEH_BOARDING`, the
/// `ps.pm_flags |= (PMF_JUMPING|PMF_JUMP_HELD)` set, and the whole `G_CanJumpToEnemyVeh`
/// jump-to-enemy-vehicle block (G_Throw/StartDeathDelay/Board/G_IsRidingVehicle). MP uses
/// `ps.torsoTimer` where the SP-leaning original wrote `ps.torsoAnimTimer` (matching the OpenJK
/// MP fix — `torsoAnimTimer` does not exist in the MP playerState). no-oracle (entity-state +
/// anim/event + trap chain).
///
/// # Safety
/// `pVeh` must be a valid `Vehicle_t*` with valid `m_pParentEntity`/`m_pVehicleInfo` and an
/// installed vtable; `pRider`/`pUcmd` must be valid pointers (live riders have a non-null
/// `client`).
pub unsafe extern "C" fn UpdateRider(
    pVeh: *mut Vehicle_t,
    pRider: *mut bgEntity_t,
    pUcmd: *mut usercmd_t,
) -> qboolean {
    let parent: *mut gentity_t;
    let rider: *mut gentity_t;

    if (*pVeh).m_iBoarding != 0 && (*pVeh).m_iDieTime == 0 {
        return QTRUE;
    }

    parent = (*pVeh).m_pParentEntity as *mut gentity_t;
    rider = pRider as *mut gentity_t;
    //MG FIXME !! Single player needs update!
    if !rider.is_null()
        && !(*rider).client.is_null()
        && !parent.is_null()
        && !(*parent).client.is_null()
    {
        //so they know who we're locking onto with our rockets, if anyone
        (*(*rider).client).ps.rocketLockIndex = (*(*parent).client).ps.rocketLockIndex;
        (*(*rider).client).ps.rocketLockTime = (*(*parent).client).ps.rocketLockTime;
        (*(*rider).client).ps.rocketTargetTime = (*(*parent).client).ps.rocketTargetTime;
    }
    // Regular exit.
    if (*pUcmd).buttons & BUTTON_USE != 0 && (*(*pVeh).m_pVehicleInfo).r#type != VH_SPEEDER {
        if (*(*pVeh).m_pVehicleInfo).r#type == VH_WALKER {
            //just get the fuck out
            (*pVeh).m_EjectDir = VEH_EJECT_REAR;
            if ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, pRider, QFALSE) != QFALSE {
                return QFALSE;
            }
        } else if (*pVeh).m_ulFlags & VEH_FLYING as c_ulong == 0 {
            // If going too fast, roll off.
            if (*(*parent).client).ps.speed <= 600.0 && (*pUcmd).rightmove != 0 {
                if ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, pRider, QFALSE) != QFALSE {
                    let Anim: animNumber_t;
                    let iFlags: c_int =
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS;
                    let iBlend: c_int = 300;
                    if (*pUcmd).rightmove > 0 {
                        Anim = BOTH_ROLL_R;
                        (*pVeh).m_EjectDir = VEH_EJECT_RIGHT;
                    } else {
                        Anim = BOTH_ROLL_L;
                        (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
                    }
                    VectorScale(
                        &(*(*parent).client).ps.velocity,
                        0.25,
                        &mut (*(*rider).client).ps.velocity,
                    );
                    Vehicle_SetAnim(rider, SETANIM_BOTH, Anim, iFlags, iBlend);
                    //PM_SetAnim(pm,SETANIM_BOTH,anim,SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD|SETANIM_FLAG_HOLDLESS);
                    (*(*rider).client).ps.weaponTime = (*(*rider).client).ps.torsoTimer - 200; //just to make sure it's cleared when roll is done
                    G_AddEvent(rider, EV_ROLL, 0);
                    return QFALSE;
                }
            } else {
                // FIXME: Check trace to see if we should start playing the animation.
                let Anim: animNumber_t;
                let iFlags: c_int = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                let iBlend: c_int = 500;
                if (*pUcmd).rightmove > 0 {
                    Anim = BOTH_VS_DISMOUNT_R;
                    (*pVeh).m_EjectDir = VEH_EJECT_RIGHT;
                } else {
                    Anim = BOTH_VS_DISMOUNT_L;
                    (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
                }

                if (*pVeh).m_iBoarding <= 1 {
                    // NOTE: I know I shouldn't reuse pVeh->m_iBoarding so many times for so many different
                    // purposes, but it's not used anywhere else right here so why waste memory???
                    let iAnimLen: c_int = BG_AnimLength((*rider).localAnimIndex, Anim);
                    (*pVeh).m_iBoarding = (*addr_of!(level)).time + iAnimLen;
                    // Weird huh? Well I wanted to reuse flags and this should never be set in an
                    // entity, so what the heck.
                    (*rider).flags |= FL_VEH_BOARDING;

                    // Make sure they can't fire when leaving.
                    (*(*rider).client).ps.weaponTime = iAnimLen;
                }

                VectorScale(
                    &(*(*parent).client).ps.velocity,
                    0.25,
                    &mut (*(*rider).client).ps.velocity,
                );

                Vehicle_SetAnim(rider, SETANIM_BOTH, Anim, iFlags, iBlend);
            }
        }
        // Flying, so just fall off.
        else {
            (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
            if ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, pRider, QFALSE) != QFALSE {
                return QFALSE;
            }
        }
    }

    // Getting off animation complete (if we had one going)?
    if (*pVeh).m_iBoarding < (*addr_of!(level)).time && (*rider).flags & FL_VEH_BOARDING != 0 {
        (*rider).flags &= !FL_VEH_BOARDING;
        // Eject this guy now.
        if ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, pRider, QFALSE) != QFALSE {
            return QFALSE;
        }
    }

    if (*(*pVeh).m_pVehicleInfo).r#type != VH_FIGHTER
        && (*(*pVeh).m_pVehicleInfo).r#type != VH_WALKER
    {
        // Jump off.
        if (*pUcmd).upmove > 0 {
            // NOT IN MULTI PLAYER!
            //===================================================================
            // (SP-only `#ifndef _JK2MP` G_CanJumpToEnemyVeh jump-to-enemy-vehicle block —
            //  Eject TOP / StartDeathDelay / G_Throw / Board onto enemy vehicle — omitted in
            //  the MP build, so this falls straight through to the force-jump-off code.)
            //===================================================================
            if ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, pRider, QFALSE) != QFALSE {
                // Allow them to force jump off.
                VectorScale(
                    &(*(*parent).client).ps.velocity,
                    0.5,
                    &mut (*(*rider).client).ps.velocity,
                );
                (*(*rider).client).ps.velocity[2] += JUMP_VELOCITY as f32;
                (*(*rider).client).ps.fd.forceJumpZStart = (*(*rider).client).ps.origin[2];

                if trap::ICARUS_TaskIDPending(rider, TID_CHAN_VOICE) == QFALSE {
                    G_AddEvent(rider, EV_JUMP, 0);
                }
                Vehicle_SetAnim(
                    rider,
                    SETANIM_BOTH,
                    BOTH_JUMP1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    300,
                );
                return QFALSE;
            }
        }

        // Roll off.
        if (*pUcmd).upmove < 0 {
            let mut Anim: animNumber_t = BOTH_ROLL_B;
            (*pVeh).m_EjectDir = VEH_EJECT_REAR;
            if (*pUcmd).rightmove > 0 {
                Anim = BOTH_ROLL_R;
                (*pVeh).m_EjectDir = VEH_EJECT_RIGHT;
            } else if (*pUcmd).rightmove < 0 {
                Anim = BOTH_ROLL_L;
                (*pVeh).m_EjectDir = VEH_EJECT_LEFT;
            } else if (*pUcmd).forwardmove < 0 {
                Anim = BOTH_ROLL_B;
                (*pVeh).m_EjectDir = VEH_EJECT_REAR;
            } else if (*pUcmd).forwardmove > 0 {
                Anim = BOTH_ROLL_F;
                (*pVeh).m_EjectDir = VEH_EJECT_FRONT;
            }

            if ((*(*pVeh).m_pVehicleInfo).Eject.unwrap())(pVeh, pRider, QFALSE) != QFALSE {
                if (*pVeh).m_ulFlags & VEH_FLYING as c_ulong == 0 {
                    VectorScale(
                        &(*(*parent).client).ps.velocity,
                        0.25,
                        &mut (*(*rider).client).ps.velocity,
                    );
                    Vehicle_SetAnim(
                        rider,
                        SETANIM_BOTH,
                        Anim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS,
                        300,
                    );
                    //PM_SetAnim(pm,SETANIM_BOTH,anim,SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD|SETANIM_FLAG_HOLDLESS);
                    (*(*rider).client).ps.weaponTime = (*(*rider).client).ps.torsoTimer - 200; //just to make sure it's cleared when roll is done
                    G_AddEvent(rider, EV_ROLL, 0);
                }
                return QFALSE;
            }
        }
    }

    QTRUE
}

/// `void G_SetSharedVehicleFunctions( vehicleInfo_t *pVehInfo )` (g_vehicles.c:3010) —
/// installs the base (type-shared) vehicle driver vtable into `pVehInfo`. Each
/// `Option<unsafe extern "C" fn ...>` field is wired to the matching sibling in this file.
///
/// `AnimateVehicle`/`AnimateRiders`/`ProcessMoveCommands`/`ProcessOrientCommands` stay
/// commented-out exactly as in the C (per-type drivers fill those in). `Update`/`UpdateRider`
/// both point at their real Rust definitions in this file.
///
/// # Safety
/// `pVehInfo` must be a valid, writable `*mut vehicleInfo_t`.
pub unsafe fn G_SetSharedVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    //	pVehInfo->AnimateVehicle				=		AnimateVehicle;
    //	pVehInfo->AnimateRiders					=		AnimateRiders;
    (*pVehInfo).ValidateBoard = Some(ValidateBoard);
    (*pVehInfo).SetParent = Some(SetParent);
    (*pVehInfo).SetPilot = Some(SetPilot);
    (*pVehInfo).AddPassenger = Some(AddPassenger);
    (*pVehInfo).Animate = Some(Animate);
    (*pVehInfo).Board = Some(Board);
    (*pVehInfo).Eject = Some(Eject);
    (*pVehInfo).EjectAll = Some(EjectAll);
    (*pVehInfo).StartDeathDelay = Some(StartDeathDelay);
    (*pVehInfo).DeathUpdate = Some(DeathUpdate);
    (*pVehInfo).RegisterAssets = Some(RegisterAssets);
    (*pVehInfo).Initialize = Some(Initialize);
    (*pVehInfo).Update = Some(Update);
    (*pVehInfo).UpdateRider = Some(UpdateRider);
    //	pVehInfo->ProcessMoveCommands			=		ProcessMoveCommands;
    //	pVehInfo->ProcessOrientCommands			=		ProcessOrientCommands;
    (*pVehInfo).AttachRiders = Some(AttachRiders);
    (*pVehInfo).Ghost = Some(Ghost);
    (*pVehInfo).UnGhost = Some(UnGhost);
    (*pVehInfo).Inhabited = Some(Inhabited);
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    #[test]
    fn G_ShipSurfaceForSurfName_matches_oracle() {
        // Every recognised surface name plus near-miss prefixes, empty, and unrelated.
        let names: &[&core::ffi::CStr] = &[
            c"nose",
            c"f_gear",
            c"glass",
            c"body",
            c"r_wing1",
            c"r_wing2",
            c"r_gear",
            c"l_wing1",
            c"l_wing2",
            c"l_gear",
            c"nos",
            c"nosey",
            c"glas",
            c"bod",
            c"bodywork",
            c"r_win",
            c"r_wing",
            c"l_gearx",
            c"",
            c"random",
            c"GLASS",
            c"Body",
        ];
        for &n in names {
            let r = unsafe { G_ShipSurfaceForSurfName(n.as_ptr()) };
            let o = unsafe { oracle::G_ShipSurfaceForSurfName(n.as_ptr()) };
            assert_eq!(r, o, "G_ShipSurfaceForSurfName({n:?})");
        }
        // NULL pointer path.
        let nul: *const core::ffi::c_char = core::ptr::null();
        assert_eq!(unsafe { G_ShipSurfaceForSurfName(nul) }, unsafe {
            oracle::G_ShipSurfaceForSurfName(nul)
        });
    }
}
