//! Port of `g_vehicleTurret.c` — vehicle auto/passenger turret aiming + firing.
//!
//! The AI/passenger turret logic for vehicles (fighters, walkers, etc.): finding
//! enemies in radius, slewing the turret bones toward a target within its clamp
//! limits, and firing the turret's vehicle weapon when aim is good.
//!
//! Ported here: `VEH_TurretCheckFire` (g_vehicleTurret.c:12),
//! `VEH_TurretAnglesToEnemy` (:61), `VEH_TurretAim` (:89),
//! `VEH_TurretFindEnemies` (:193, C `static`), `VEH_TurretObeyPassengerControl` (:304),
//! `VEH_TurretThink` (:324).

#![allow(non_snake_case)] // C function names kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use crate::codemp::game::bg_public::{GT_TEAM, MASK_SHOT, TEAM_SPECTATOR};
use crate::codemp::game::bg_vehicleLoad::g_vehWeaponInfo;
use crate::codemp::game::bg_vehicles_h::{turretStats_t, vehWeaponInfo_t, Vehicle_t};
use crate::codemp::game::bg_weapons_h::WP_TURRET;
use crate::codemp::game::g_local::{gentity_t, FL_BBRUSH, FL_NOTARGET};
use crate::codemp::game::g_main::{g_entities, g_gametype, level};
use crate::codemp::game::g_team::OnSameTeam;
use crate::codemp::game::g_utils::G_RadiusList;
use crate::codemp::game::g_weapon::{G_VehMuzzleFireFX, WP_CalcVehMuzzle, WP_FireVehicleWeapon};
use crate::codemp::game::npc_utils::NPC_SetBoneAngles;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleNormalize180, AnglesSubtract, VectorClear, VectorCopy,
    VectorLengthSquared, VectorMA, VectorNormalize, VectorSubtract,
};
use crate::codemp::game::q_shared::{Q_stricmp, Q_strncmp};
use crate::codemp::game::q_shared_h::{
    vec3_t, BUTTON_ALT_ATTACK, BUTTON_ATTACK, ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_GENTITIES,
    PITCH, YAW,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

use core::ffi::{c_int, CStr};
use core::ptr::{addr_of, null_mut};

/// `void VEH_TurretCheckFire( Vehicle_t *pVeh, gentity_t *parent, turretStats_t *turretStats,
/// vehWeaponInfo_t *vehWeapon, int turretNum, int curMuzzle )` (g_vehicleTurret.c:12).
///
/// Fires the turret's current muzzle if the muzzle is valid, off cooldown, and has ammo:
/// recalcs the muzzle, fires the vehicle weapon, plays the muzzle FX, decrements ammo,
/// toggles to the turret's next muzzle and arms its fire delay. No oracle (mutates
/// `Vehicle_t`/`gentity_t` state and calls weapon/FX traps); verified by review against the C.
///
/// # Safety
/// `pVeh`, `parent`, `turretStats`, `vehWeapon` must point to valid objects; `level` must be
/// initialised; `turretNum`/`curMuzzle` must be valid turret/muzzle indices.
pub unsafe fn VEH_TurretCheckFire(
    pVeh: *mut Vehicle_t,
    parent: *mut gentity_t,
    //gentity_t *turretEnemy,
    turretStats: *mut turretStats_t,
    vehWeapon: *mut vehWeaponInfo_t,
    turretNum: c_int,
    curMuzzle: c_int,
) {
    // if it's time to fire and we have an enemy, then gun 'em down!  pushDebounce time controls next fire time
    if (*pVeh).m_iMuzzleTag[curMuzzle as usize] == -1 {
        //invalid muzzle?
        return;
    }

    if (*pVeh).m_iMuzzleWait[curMuzzle as usize] >= (*addr_of!(level)).time {
        //can't fire yet
        return;
    }

    if (*pVeh).turretStatus[turretNum as usize].ammo < (*vehWeapon).iAmmoPerShot {
        //no ammo, can't fire
        return;
    }

    //if ( turretEnemy )
    {
        //FIXME: check to see if I'm aiming generally where I want to
        let muzzlesFired: c_int = 1 << curMuzzle;
        let missile: *mut gentity_t;
        WP_CalcVehMuzzle(parent, curMuzzle);

        //FIXME: some variation in fire dir
        let dir = (*pVeh).m_vMuzzleDir[curMuzzle as usize];
        missile = WP_FireVehicleWeapon(
            parent,
            &mut (*pVeh).m_vMuzzlePos[curMuzzle as usize],
            &dir,
            vehWeapon,
            (turretNum != 0) as qboolean,
            QTRUE,
        );

        //play the weapon's muzzle effect if we have one
        G_VehMuzzleFireFX(parent, missile, muzzlesFired);

        //take the ammo away
        (*pVeh).turretStatus[turretNum as usize].ammo -= (*vehWeapon).iAmmoPerShot;
        //toggle to the next muzzle on this turret, if there is one
        let nextMuzzle: c_int = if (curMuzzle + 1)
            == (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].iMuzzle[0]
        {
            (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].iMuzzle[1]
        } else {
            (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].iMuzzle[0]
        };
        if nextMuzzle != 0 {
            //a valid muzzle to toggle to
            (*pVeh).turretStatus[turretNum as usize].nextMuzzle = nextMuzzle - 1;
            //-1 because you type muzzles 1-10 in the .veh file
        }
        //add delay to the next muzzle so it doesn't fire right away on the next frame
        (*pVeh).m_iMuzzleWait[(*pVeh).turretStatus[turretNum as usize].nextMuzzle as usize] =
            (*addr_of!(level)).time + (*turretStats).iDelay;
    }
}

/// `void VEH_TurretAnglesToEnemy( Vehicle_t *pVeh, int curMuzzle, float fSpeed,
/// gentity_t *turretEnemy, qboolean bAILead, vec3_t desiredAngles )` (g_vehicleTurret.c:61).
///
/// Computes the absolute world angles from the turret muzzle to `turretEnemy`, optionally
/// leading the target by its velocity (scaled by distance / projectile speed) when `bAILead`
/// is set, and writes them into `desiredAngles`. No oracle (reads through `gentity_t`/`Vehicle_t`
/// pointers); verified by review against the C.
///
/// # Safety
/// `pVeh` and `turretEnemy` must point to valid objects; `curMuzzle` must be a valid muzzle
/// index; `desiredAngles` must be writable.
pub unsafe fn VEH_TurretAnglesToEnemy(
    pVeh: *mut Vehicle_t,
    curMuzzle: c_int,
    fSpeed: f32,
    turretEnemy: *mut gentity_t,
    bAILead: qboolean,
    desiredAngles: &mut vec3_t,
) {
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    VectorCopy(&(*turretEnemy).r.currentOrigin, &mut org);
    if bAILead != QFALSE {
        //we want to lead them a bit
        let mut diff: vec3_t = [0.0; 3];
        let mut velocity: vec3_t = [0.0; 3];
        VectorSubtract(&org, &(*pVeh).m_vMuzzlePos[curMuzzle as usize], &mut diff);
        let dist = VectorNormalize(&mut diff);
        if !(*turretEnemy).client.is_null() {
            VectorCopy(&(*(*turretEnemy).client).ps.velocity, &mut velocity);
        } else {
            VectorCopy(&(*turretEnemy).s.pos.trDelta, &mut velocity);
        }
        VectorMA(&org.clone(), dist / fSpeed, &velocity, &mut org);
    }

    //FIXME: this isn't quite right, it's aiming from the muzzle, not the center of the turret...
    VectorSubtract(&org, &(*pVeh).m_vMuzzlePos[curMuzzle as usize], &mut enemyDir);
    //Get the desired absolute, world angles to our target
    vectoangles(&enemyDir, desiredAngles);
}

/// `qboolean VEH_TurretAim( Vehicle_t *pVeh, gentity_t *parent, gentity_t *turretEnemy,
/// turretStats_t *turretStats, vehWeaponInfo_t *vehWeapon, int turretNum, int curMuzzle,
/// vec3_t desiredAngles )` (g_vehicleTurret.c:89).
///
/// Slews the turret toward `desiredAngles` (or toward `turretEnemy` if one is given): computes
/// the relative desired angles, clamps them to the turret's yaw/pitch limits, caps the per-frame
/// turn by `fTurnSpeed`, and drives the yaw/pitch bones via `NPC_SetBoneAngles`. Returns whether
/// the turret is aimed within its clamps (so the caller may fire). No oracle (mutates ghoul2 bone
/// state + reads through pointers); verified by review against the C.
///
/// Faithful simplification: the C declares `float aimCorrect = qfalse;` and `return aimCorrect;`
/// against a `qboolean` return — the float only ever holds 0.0/1.0, so it is ported as a
/// `qboolean` directly.
///
/// # Safety
/// `pVeh`, `parent`, `turretStats`, `vehWeapon` must point to valid objects (`turretEnemy` may be
/// NULL); `curMuzzle`/`turretNum` valid; `desiredAngles` writable. `pVeh->m_vOrientation` must be
/// a valid `vec3_t` pointer.
pub unsafe fn VEH_TurretAim(
    pVeh: *mut Vehicle_t,
    parent: *mut gentity_t,
    turretEnemy: *mut gentity_t,
    turretStats: *mut turretStats_t,
    vehWeapon: *mut vehWeaponInfo_t,
    turretNum: c_int,
    curMuzzle: c_int,
    desiredAngles: &mut vec3_t,
) -> qboolean {
    let mut curAngles: vec3_t = [0.0; 3];
    let mut addAngles: vec3_t = [0.0; 3];
    let mut newAngles: vec3_t = [0.0; 3];
    let mut yawAngles: vec3_t;
    let mut pitchAngles: vec3_t;
    let mut aimCorrect: qboolean = QFALSE;

    WP_CalcVehMuzzle(parent, curMuzzle);
    //get the current absolute angles of the turret right now
    vectoangles(&(*pVeh).m_vMuzzleDir[curMuzzle as usize], &mut curAngles);
    //subtract out the vehicle's angles to get the relative alignment
    AnglesSubtract(
        &curAngles.clone(),
        &*((*pVeh).m_vOrientation as *const vec3_t),
        &mut curAngles,
    );

    if !turretEnemy.is_null() {
        aimCorrect = QTRUE;
        // ...then we'll calculate what new aim adjustments we should attempt to make this frame
        // Aim at enemy
        VEH_TurretAnglesToEnemy(
            pVeh,
            curMuzzle,
            (*vehWeapon).fSpeed,
            turretEnemy,
            (*turretStats).bAILead,
            desiredAngles,
        );
    }
    //subtract out the vehicle's angles to get the relative desired alignment
    AnglesSubtract(
        &desiredAngles.clone(),
        &*((*pVeh).m_vOrientation as *const vec3_t),
        desiredAngles,
    );
    //Now clamp the desired relative angles
    //clamp yaw
    desiredAngles[YAW] = AngleNormalize180(desiredAngles[YAW]);
    if (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampLeft != 0.0
        && desiredAngles[YAW] > (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampLeft
    {
        aimCorrect = QFALSE;
        desiredAngles[YAW] = (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampLeft;
    }
    if (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampRight != 0.0
        && desiredAngles[YAW] < (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampRight
    {
        aimCorrect = QFALSE;
        desiredAngles[YAW] = (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampRight;
    }
    //clamp pitch
    desiredAngles[PITCH] = AngleNormalize180(desiredAngles[PITCH]);
    if (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampDown != 0.0
        && desiredAngles[PITCH]
            > (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampDown
    {
        aimCorrect = QFALSE;
        desiredAngles[PITCH] =
            (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampDown;
    }
    if (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampUp != 0.0
        && desiredAngles[PITCH] < (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampUp
    {
        aimCorrect = QFALSE;
        desiredAngles[PITCH] = (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampUp;
    }
    //Now get the offset we want from our current relative angles
    AnglesSubtract(desiredAngles, &curAngles, &mut addAngles);
    //Now cap the addAngles for our fTurnSpeed
    if addAngles[PITCH] > (*turretStats).fTurnSpeed {
        //aimCorrect = qfalse;//???
        addAngles[PITCH] = (*turretStats).fTurnSpeed;
    } else if addAngles[PITCH] < -(*turretStats).fTurnSpeed {
        //aimCorrect = qfalse;//???
        addAngles[PITCH] = -(*turretStats).fTurnSpeed;
    }
    if addAngles[YAW] > (*turretStats).fTurnSpeed {
        //aimCorrect = qfalse;//???
        addAngles[YAW] = (*turretStats).fTurnSpeed;
    } else if addAngles[YAW] < -(*turretStats).fTurnSpeed {
        //aimCorrect = qfalse;//???
        addAngles[YAW] = -(*turretStats).fTurnSpeed;
    }
    //Now add the additional angles back in to our current relative angles
    //FIXME: add some AI aim error randomness...?
    newAngles[PITCH] = AngleNormalize180(curAngles[PITCH] + addAngles[PITCH]);
    newAngles[YAW] = AngleNormalize180(curAngles[YAW] + addAngles[YAW]);
    //Now set the bone angles to the new angles
    //set yaw
    if !(*turretStats).yawBone.is_null() {
        yawAngles = [0.0; 3];
        VectorClear(&mut yawAngles);
        yawAngles[(*turretStats).yawAxis as usize] = newAngles[YAW];
        NPC_SetBoneAngles(
            parent,
            CStr::from_ptr((*turretStats).yawBone).to_str().unwrap_or(""),
            &yawAngles,
        );
    }
    //set pitch
    if !(*turretStats).pitchBone.is_null() {
        pitchAngles = [0.0; 3];
        VectorClear(&mut pitchAngles);
        pitchAngles[(*turretStats).pitchAxis as usize] = newAngles[PITCH];
        NPC_SetBoneAngles(
            parent,
            CStr::from_ptr((*turretStats).pitchBone)
                .to_str()
                .unwrap_or(""),
            &pitchAngles,
        );
    }
    //force muzzle to recalc next check
    (*pVeh).m_iMuzzleTime[curMuzzle as usize] = 0;

    aimCorrect
}

/// `static qboolean VEH_TurretFindEnemies( Vehicle_t *pVeh, gentity_t *parent,
/// turretStats_t *turretStats, int turretNum, int curMuzzle )` (g_vehicleTurret.c:193).
///
/// Scans `fAIRange` around the turret muzzle for a valid, visible enemy: skips friendlies,
/// spectators, the pilot/passengers, non-clients that aren't breakable brushes or misc_turrets,
/// and anything without a clear PVS/trace shot, then records the nearest (clients preferred) into
/// `turretStatus[turretNum].enemyEntNum`. C `static` — file-local. No oracle (radius/PVS/trace
/// traps + entity-state reads); verified by review against the C.
///
/// # Safety
/// `pVeh`, `parent`, `turretStats` must point to valid objects; `g_entities` populated;
/// `turretNum`/`curMuzzle` valid.
unsafe fn VEH_TurretFindEnemies(
    pVeh: *mut Vehicle_t,
    parent: *mut gentity_t,
    turretStats: *mut turretStats_t,
    turretNum: c_int,
    curMuzzle: c_int,
) -> qboolean {
    let mut found: qboolean = QFALSE;
    let mut bestDist: f32 = (*turretStats).fAIRange * (*turretStats).fAIRange;
    let mut org2: vec3_t = [0.0; 3];
    let mut foundClient: qboolean = QFALSE;
    let mut entity_list: [*mut gentity_t; MAX_GENTITIES] = [null_mut(); MAX_GENTITIES];
    let mut bestTarget: *mut gentity_t = null_mut();

    WP_CalcVehMuzzle(parent, curMuzzle);
    VectorCopy(&(*pVeh).m_vMuzzlePos[curMuzzle as usize], &mut org2);

    let count = G_RadiusList(&org2, (*turretStats).fAIRange, parent, QTRUE, &mut entity_list);

    for i in 0..count {
        let target: *mut gentity_t = entity_list[i as usize];

        if target == parent
            || (*target).takedamage == QFALSE
            || (*target).health <= 0
            || ((*target).flags & FL_NOTARGET) != 0
        {
            continue;
        }
        if (*target).client.is_null() {
            // only attack clients
            if ((*target).flags & FL_BBRUSH) == 0 //not a breakable brush
                || (*target).takedamage == QFALSE //is a bbrush, but invincible
                || (!(*target).NPC_targetname.is_null()
                    && !(*parent).targetname.is_null()
                    && Q_stricmp((*target).NPC_targetname, (*parent).targetname) != 0)
            //not in invicible bbrush, but can only be broken by an NPC that is not me
            {
                if (*target).s.weapon == WP_TURRET
                    && !(*target).classname.is_null()
                    && Q_strncmp(c"misc_turret".as_ptr(), (*target).classname, 11) == 0
                {
                    //these guys we want to shoot at
                } else {
                    continue;
                }
            }
            //else: we will shoot at bbrushes!
        } else if (*(*target).client).sess.sessionTeam == TEAM_SPECTATOR {
            continue;
        }
        if target == ((*pVeh).m_pPilot as *mut gentity_t)
            || (*target).r.ownerNum == (*parent).s.number
        {
            //don't get angry at my pilot or passengers?
            continue;
        }
        if !(*parent).client.is_null() && (*(*parent).client).sess.sessionTeam != 0 {
            if !(*target).client.is_null() {
                if (*(*target).client).sess.sessionTeam == (*(*parent).client).sess.sessionTeam {
                    // A bot/client/NPC we don't want to shoot
                    continue;
                }
            } else if (*target).teamnodmg == (*(*parent).client).sess.sessionTeam {
                //some other entity that's allied with us
                continue;
            }
        }
        if trap::InPVS(&org2, &(*target).r.currentOrigin) == QFALSE {
            continue;
        }

        let mut org: vec3_t = [0.0; 3];
        VectorCopy(&(*target).r.currentOrigin, &mut org);

        let tr = trap::Trace(
            &org2,
            &vec3_origin,
            &vec3_origin,
            &org,
            (*parent).s.number,
            MASK_SHOT,
        );

        if tr.entityNum as c_int == (*target).s.number
            || (tr.allsolid == 0 && tr.startsolid == 0 && tr.fraction == 1.0)
        {
            // Only acquire if have a clear shot, Is it in range and closer than our best?
            let mut enemyDir: vec3_t = [0.0; 3];
            VectorSubtract(&(*target).r.currentOrigin, &org2, &mut enemyDir);
            let enemyDist = VectorLengthSquared(&enemyDir);

            if enemyDist < bestDist || (!(*target).client.is_null() && foundClient == QFALSE) {
                // all things equal, keep current
                bestTarget = target;
                bestDist = enemyDist;
                found = QTRUE;
                if !(*target).client.is_null() {
                    //prefer clients over non-clients
                    foundClient = QTRUE;
                }
            }
        }
    }

    if found != QFALSE {
        (*pVeh).turretStatus[turretNum as usize].enemyEntNum = (*bestTarget).s.number;
    }

    found
}

/// `void VEH_TurretObeyPassengerControl( Vehicle_t *pVeh, gentity_t *parent, int turretNum )`
/// (g_vehicleTurret.c:304).
///
/// When a living passenger client controls this turret, aims it at that passenger's view angles
/// and fires if they are holding an attack button. No oracle (entity-state reads + aim/fire trap
/// chain); verified by review against the C.
///
/// # Safety
/// `pVeh`, `parent` must point to valid objects; `turretNum` valid; `g_vehWeaponInfo` initialised.
pub unsafe fn VEH_TurretObeyPassengerControl(
    pVeh: *mut Vehicle_t,
    parent: *mut gentity_t,
    turretNum: c_int,
) {
    let turretStats: *mut turretStats_t =
        &mut (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize];
    let passenger: *mut gentity_t =
        (*pVeh).m_ppPassengers[((*turretStats).passengerNum - 1) as usize] as *mut gentity_t;

    if !passenger.is_null() && !(*passenger).client.is_null() && (*passenger).health > 0 {
        //a valid, living passenger client
        let vehWeapon: *mut vehWeaponInfo_t = &mut g_vehWeaponInfo[(*turretStats).iWeapon as usize];
        let curMuzzle: c_int = (*pVeh).turretStatus[turretNum as usize].nextMuzzle;
        let mut aimAngles: vec3_t = [0.0; 3];
        VectorCopy(&(*(*passenger).client).ps.viewangles, &mut aimAngles);

        VEH_TurretAim(
            pVeh,
            parent,
            null_mut(),
            turretStats,
            vehWeapon,
            turretNum,
            curMuzzle,
            &mut aimAngles,
        );
        if ((*(*passenger).client).pers.cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) != 0 {
            //he's pressing an attack button, so fire!
            VEH_TurretCheckFire(pVeh, parent, turretStats, vehWeapon, turretNum, curMuzzle);
        }
    }
}

/// `void VEH_TurretThink( Vehicle_t *pVeh, gentity_t *parent, int turretNum )`
/// (g_vehicleTurret.c:324).
///
/// Per-frame driver for one turret: hands off to passenger control if a passenger mans it,
/// otherwise (for AI turrets) validates/drops the held enemy, finds a new one when the hold
/// time expires, range/PVS/trace-checks it, and on a good line aims and fires. No oracle (full
/// AI turret update — entity-state mutation + radius/PVS/trace traps); verified by review
/// against the C.
///
/// # Safety
/// `pVeh`, `parent` must point to valid objects; `g_entities`/`level`/`g_vehWeaponInfo`
/// initialised; `turretNum` a valid turret index.
pub unsafe fn VEH_TurretThink(pVeh: *mut Vehicle_t, parent: *mut gentity_t, turretNum: c_int) {
    let mut doAim: qboolean = QFALSE;
    let turretStats: *mut turretStats_t =
        &mut (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize];
    let mut turretEnemy: *mut gentity_t = null_mut();

    if turretStats.is_null() || (*turretStats).iAmmoMax == 0 {
        //not a valid turret
        return;
    }

    if (*turretStats).passengerNum != 0 && (*pVeh).m_iNumPassengers >= (*turretStats).passengerNum {
        //the passenger that has control of this turret is on the ship
        VEH_TurretObeyPassengerControl(pVeh, parent, turretNum);
        return;
    } else if (*turretStats).bAI == QFALSE {
        //try AI
        //this turret does not think on its own.
        return;
    }
    //okay, so it has AI, but still don't think if there's no pilot!
    if (*pVeh).m_pPilot.is_null() {
        return;
    }

    let vehWeapon: *mut vehWeaponInfo_t = &mut g_vehWeaponInfo[(*turretStats).iWeapon as usize];
    let rangeSq: f32 = (*turretStats).fAIRange * (*turretStats).fAIRange;
    let curMuzzle: c_int = (*pVeh).turretStatus[turretNum as usize].nextMuzzle;

    if (*pVeh).turretStatus[turretNum as usize].enemyEntNum < ENTITYNUM_WORLD {
        turretEnemy = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*pVeh).turretStatus[turretNum as usize].enemyEntNum as usize);
        if (*turretEnemy).health < 0
            || (*turretEnemy).inuse == QFALSE
            || turretEnemy == ((*pVeh).m_pPilot as *mut gentity_t) //enemy became my pilot///?
            || turretEnemy == parent
            || (*turretEnemy).r.ownerNum == (*parent).s.number // a passenger?
            || (!(*turretEnemy).client.is_null()
                && (*(*turretEnemy).client).sess.sessionTeam == TEAM_SPECTATOR)
        {
            //don't keep going after spectators, pilot, self, dead people, etc.
            turretEnemy = null_mut();
            (*pVeh).turretStatus[turretNum as usize].enemyEntNum = ENTITYNUM_NONE;
        }
    }

    if (*pVeh).turretStatus[turretNum as usize].enemyHoldTime < (*addr_of!(level)).time {
        if VEH_TurretFindEnemies(pVeh, parent, turretStats, turretNum, curMuzzle) != QFALSE {
            turretEnemy = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*pVeh).turretStatus[turretNum as usize].enemyEntNum as usize);
            doAim = QTRUE;
        } else if !(*parent).enemy.is_null() && (*(*parent).enemy).s.number < ENTITYNUM_WORLD {
            if (*addr_of!(g_gametype)).integer < GT_TEAM
                || OnSameTeam((*parent).enemy, parent) == QFALSE
            {
                //either not in a team game or the enemy isn't on the same team
                turretEnemy = (*parent).enemy;
                doAim = QTRUE;
            }
        }
        if !turretEnemy.is_null() {
            //found one
            if !(*turretEnemy).client.is_null() {
                //hold on to clients for a min of 3 seconds
                (*pVeh).turretStatus[turretNum as usize].enemyHoldTime =
                    (*addr_of!(level)).time + 3000;
            } else {
                //hold less
                (*pVeh).turretStatus[turretNum as usize].enemyHoldTime =
                    (*addr_of!(level)).time + 500;
            }
        }
    }
    if !turretEnemy.is_null() {
        if (*turretEnemy).health > 0 {
            // enemy is alive
            WP_CalcVehMuzzle(parent, curMuzzle);
            let mut enemyDir: vec3_t = [0.0; 3];
            VectorSubtract(
                &(*turretEnemy).r.currentOrigin,
                &(*pVeh).m_vMuzzlePos[curMuzzle as usize],
                &mut enemyDir,
            );
            let enemyDist = VectorLengthSquared(&enemyDir);

            if enemyDist < rangeSq {
                // was in valid radius
                if trap::InPVS(
                    &(*pVeh).m_vMuzzlePos[curMuzzle as usize],
                    &(*turretEnemy).r.currentOrigin,
                ) != QFALSE
                {
                    // Every now and again, check to see if we can even trace to the enemy
                    let mut start: vec3_t = [0.0; 3];
                    let mut end: vec3_t = [0.0; 3];
                    VectorCopy(&(*pVeh).m_vMuzzlePos[curMuzzle as usize], &mut start);

                    VectorCopy(&(*turretEnemy).r.currentOrigin, &mut end);
                    let tr = trap::Trace(
                        &start,
                        &vec3_origin,
                        &vec3_origin,
                        &end,
                        (*parent).s.number,
                        MASK_SHOT,
                    );

                    if tr.entityNum as c_int == (*turretEnemy).s.number
                        || (tr.allsolid == 0 && tr.startsolid == 0)
                    {
                        doAim = QTRUE; // Can see our enemy
                    }
                }
            }
        }
    }

    if doAim != QFALSE {
        let mut aimAngles: vec3_t = [0.0; 3];
        if VEH_TurretAim(
            pVeh,
            parent,
            turretEnemy,
            turretStats,
            vehWeapon,
            turretNum,
            curMuzzle,
            &mut aimAngles,
        ) != QFALSE
        {
            VEH_TurretCheckFire(
                pVeh, parent, /*turretEnemy,*/ turretStats, vehWeapon, turretNum, curMuzzle,
            );
        }
    }
}
