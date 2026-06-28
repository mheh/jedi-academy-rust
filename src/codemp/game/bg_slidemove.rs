//! `bg_slidemove.c` — "part of bg_pmove functionality": the classic Quake3
//! `PM_SlideMove`/`PM_StepSlideMove` collision-and-slide integrators that the
//! `PM_WalkMove`/`PM_AirMove`/`PM_WaterMove`/`PM_FlyMove` move-modes (not yet ported in
//! `bg_pmove.rs`, blocked on exactly this file) call to advance `pm->ps->origin`
//! against the world via the `pm->trace` engine callback.
//!
//! This slice lands the three movement-critical functions: [`PM_GroundSlideOkay`]
//! (the wall-run "never push up off a sloped wall" predicate), [`PM_SlideMove`] (the
//! clip-against-up-to-five-planes slide loop) and [`PM_StepSlideMove`] (the slide +
//! step-up-a-stair wrapper). All three are trace/callback-driven keystone helpers, so
//! — like the `pm`/`pml` move-fns in `bg_pmove.rs` — they get **no bit-exact oracle**;
//! they are behaviourally verified transitively once `Pmove` lands. The `static`-in-C
//! functions become `pub` (the keystone convention).
//!
//! **Precision:** f32 arithmetic throughout, matching the no-oracle move-fn convention
//! in `bg_pmove.rs` — the unsuffixed C `double` literals (`0.5`, `0.1`, `0.7`) are
//! carried as f32 deviations (the `PM_CheckJump` precedent). If a future `Pmove`
//! parity test flags a slide/step desync, these are the first place to widen to f64.
//!
//! [`PM_VehicleImpact`] (bg_slidemove.c:49) — the vehicle crash-into-ground handler — is
//! ported here (QAGAME branch only, since the crate is the server game module). Its sole
//! caller is the NPC-vehicle branch of [`PM_SlideMove`] (`clientNum >= MAX_CLIENTS`).

#![allow(non_upper_case_globals)]

use crate::codemp::game::anims::{
    BOTH_FORCELONGLEAP_ATTACK, BOTH_FORCELONGLEAP_LAND, BOTH_FORCELONGLEAP_START,
    BOTH_FORCEWALLRUNFLIP_START, BOTH_WALL_RUN_LEFT, BOTH_WALL_RUN_LEFT_STOP, BOTH_WALL_RUN_RIGHT,
    BOTH_WALL_RUN_RIGHT_STOP,
};
use crate::codemp::game::bg_local_h::{MIN_WALK_NORMAL, OVERCLIP};
use crate::codemp::game::bg_panimate::{BG_InReboundHold, BG_InReboundJump};
use crate::codemp::game::bg_pmove::{
    pm, pm_entSelf, pml, BG_KnockDownable, PM_AddEvent, PM_AddTouchEnt, PM_ClipVelocity,
};
use crate::codemp::game::bg_public::{
    bgEntity_t, EF_JETPACK_ACTIVE, ET_MISSILE, ET_MOVER, ET_NPC, ET_PLAYER, ET_TERRAIN,
    EV_PLAY_EFFECT_ID, EV_STEP_12, EV_STEP_16, EV_STEP_4, EV_STEP_8, HANDEXTEND_KNOCKDOWN,
    MOD_COLLISION, MOD_FALLING, PMF_STUCK_TO_WALL, STEPSIZE,
};
use crate::codemp::game::bg_vehicles_h::{
    Vehicle_t, MIN_LANDING_SLOPE, MIN_LANDING_SPEED, VEH_CRASHING, VH_FIGHTER, VH_SPEEDER,
    VH_WALKER,
};
use crate::codemp::game::bg_weapons_h::WP_NONE;
use crate::codemp::game::fighternpc::FighterIsLanded;
use crate::codemp::game::g_active::Client_CheckImpactBBrush;
use crate::codemp::game::g_combat::{G_Damage, G_DamageFromKiller};
use crate::codemp::game::g_local::{gentity_t, DAMAGE_NO_ARMOR};
use crate::codemp::game::g_main::{g_entities, level};
use crate::codemp::game::g_utils::G_AddEvent;
use crate::codemp::game::g_vehicles::G_FlyVehicleSurfaceDestruction;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleNormalize180, AngleVectors, AnglesSubtract, CrossProduct,
    DotProduct, VectorAdd, VectorClear, VectorCompare, VectorCopy, VectorLength, VectorMA,
    VectorNormalize, VectorNormalize2, VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::Q_stricmp;
use crate::codemp::game::q_shared_h::{
    qboolean, trace_t, vec3_t, ENTITYNUM_WORLD, MAX_CLIENTS, PITCH, QFALSE, QTRUE, ROLL,
    SOLID_BMODEL, TR_STATIONARY, YAW,
};
use crate::codemp::game::teams_h::{CLASS_ATST, CLASS_RANCOR, CLASS_VEHICLE};
use crate::codemp::game::w_saber::G_CanBeEnemy;
use core::ffi::{c_int, c_ulong};
use core::ptr::{addr_of, addr_of_mut};

const MAX_IMPACT_TURN_ANGLE: f32 = 45.0;

/// `PM_VehicleImpact(bgEntity_t *pEnt, trace_t *trace)` (bg_slidemove.c:49) — vehicle
/// crash-into-ground/-into-something handler. On a hard enough impact it bounces/turns the
/// vehicle off the surface (fighters/speeders), plays the impact effect, damages the vehicle
/// (and possibly tears off a surface via [`G_FlyVehicleSurfaceDestruction`]), and damages /
/// knocks down whatever it hit. QAGAME (server) branch only — the crate is the game module.
///
/// `pSelfVeh->m_vOrientation` is a `*mut vec_t` in this port (the SP-shared layout), so the
/// C array subscripts become `.add(idx)` and whole-vector uses cast through `*const vec3_t`.
///
/// # Safety
/// `pEnt` must be a valid `bgEntity_t*` whose `m_pVehicle`/`m_pVehicleInfo` chain is valid;
/// `pm`/`g_entities`/`level` must be live. `trace` may be NULL.
pub unsafe fn PM_VehicleImpact(pEnt: *mut bgEntity_t, trace: *mut trace_t) {
    let pmv = *addr_of!(pm);
    // See if the vehicle has crashed into the ground.
    let pSelfVeh: *mut Vehicle_t = (*pEnt).m_pVehicle;
    let mut magnitude: f32 =
        VectorLength(&(*(*pmv).ps).velocity) * (*(*pSelfVeh).m_pVehicleInfo).mass as f32 / 50.0;
    let mut forceSurfDestruction: qboolean = QFALSE;
    // #ifdef QAGAME
    let hitEnt: *mut gentity_t = if !trace.is_null() {
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*trace).entityNum as usize)
    } else {
        core::ptr::null_mut()
    };

    if hitEnt.is_null()
        || (!pSelfVeh.is_null()
            && !(*pSelfVeh).m_pPilot.is_null()
            && (*hitEnt).s.eType == ET_MISSILE
            && (*hitEnt).inuse != QFALSE
            && (*hitEnt).r.ownerNum == (*(*pSelfVeh).m_pPilot).s.number)
    {
        return;
    }

    if !pSelfVeh.is_null()      //I have a vehicle struct
        && (*pSelfVeh).m_iRemovedSurfaces != 0
    //vehicle has bits removed
    {
        //spiralling to our deaths, explode on any solid impact
        if (*hitEnt).s.NPC_class == CLASS_VEHICLE {
            //hit another vehicle, explode!
            //Give credit to whoever got me into this death spiral state
            G_DamageFromKiller(
                pEnt as *mut gentity_t,
                (*pSelfVeh).m_pParentEntity as *mut gentity_t,
                hitEnt as *mut gentity_t,
                (*(*pmv).ps).origin.as_mut_ptr() as *mut vec3_t,
                999999,
                DAMAGE_NO_ARMOR,
                MOD_COLLISION,
            );
            return;
        } else if VectorCompare(&(*trace).plane.normal, &vec3_origin) == QFALSE
            && ((*trace).entityNum as c_int == ENTITYNUM_WORLD || (*hitEnt).r.bmodel != QFALSE)
        {
            //have a valid hit plane and we hit a solid brush
            let mut moveDir: vec3_t = [0.0; 3];
            let impactDot: f32;
            VectorCopy(&(*(*pmv).ps).velocity, &mut moveDir);
            VectorNormalize(&mut moveDir);
            impactDot = DotProduct(&moveDir, &(*trace).plane.normal);
            if impactDot <= -0.7
            //hit rather head-on and hard
            {
                // Just DIE now
                //Give credit to whoever got me into this death spiral state
                G_DamageFromKiller(
                    pEnt as *mut gentity_t,
                    (*pSelfVeh).m_pParentEntity as *mut gentity_t,
                    hitEnt as *mut gentity_t,
                    (*(*pmv).ps).origin.as_mut_ptr() as *mut vec3_t,
                    999999,
                    DAMAGE_NO_ARMOR,
                    MOD_FALLING,
                );
                return;
            }
        }
    }

    if ((*trace).entityNum as c_int) < ENTITYNUM_WORLD
        && (*hitEnt).s.eType == ET_MOVER
        && (*hitEnt).s.apos.trType != TR_STATIONARY//rotating
        && ((*hitEnt).spawnflags & 16) != 0 //IMPACT
        && Q_stricmp(c"func_rotating".as_ptr(), (*hitEnt).classname) == 0
    {
        //hit a func_rotating that is supposed to destroy anything it touches!
        //guarantee the hit will happen, thereby taking off a piece of the ship
        forceSurfDestruction = QTRUE;
    } else if ((*(*pmv).ps).velocity[0].abs() + (*(*pmv).ps).velocity[1].abs()) < 100.0
        && (*(*pmv).ps).velocity[2] > -100.0
    {
        //we're landing, we're cool
        //FIXME: some sort of landing "thump", not the impactFX
        //this was annoying me -rww
        //FIXME: this shouldn't even be getting called when the vehicle is at rest!
        if !hitEnt.is_null()
            && ((*hitEnt).s.eType == ET_PLAYER || (*hitEnt).s.eType == ET_NPC)
            && (*(*pSelfVeh).m_pVehicleInfo).r#type == VH_FIGHTER
        {
            //always smack players
        } else {
            return;
        }
    }
    if !pSelfVeh.is_null()
        && ((*(*pSelfVeh).m_pVehicleInfo).r#type == VH_SPEEDER
            || (*(*pSelfVeh).m_pVehicleInfo).r#type == VH_FIGHTER)//this is kind of weird on tauntauns and atst's..
        && (magnitude >= 100.0 || forceSurfDestruction != QFALSE)
    {
        if (*(*pEnt).m_pVehicle).m_iHitDebounce < (*pmv).cmd.serverTime
            || forceSurfDestruction != QFALSE
        {
            //a bit of a hack, may conflict with getting shot, but...
            //FIXME: impact sound and effect should be gotten from g_vehicleInfo...?
            //FIXME: should pass in trace.endpos and trace.plane.normal
            let mut vehUp: vec3_t = [0.0; 3];
            // #ifdef QAGAME
            let mut noDamage: qboolean = QFALSE;

            if !trace.is_null()
                && (*pSelfVeh).m_iRemovedSurfaces == 0
                && forceSurfDestruction == QFALSE
            {
                let mut turnFromImpact: qboolean = QFALSE;
                let mut turnHitEnt: qboolean = QFALSE;
                let l: f32 = (*(*pmv).ps).speed * 0.5;
                let mut bounceDir: vec3_t = [0.0; 3];
                if ((*trace).entityNum as c_int == ENTITYNUM_WORLD
                    || (*hitEnt).s.solid == SOLID_BMODEL)//bounce off any brush
                    && VectorCompare(&(*trace).plane.normal, &vec3_origin) == QFALSE
                //have a valid plane to bounce off of
                {
                    //bounce off in the opposite direction of the impact
                    if (*(*pSelfVeh).m_pVehicleInfo).r#type == VH_SPEEDER {
                        (*(*pmv).ps).speed *= (*addr_of!(pml)).frametime;
                        VectorCopy(&(*trace).plane.normal, &mut bounceDir);
                    } else if (*trace).plane.normal[2] >= MIN_LANDING_SLOPE//flat enough to land on
                        && (*pSelfVeh).m_LandTrace.fraction < 1.0 //ground present
                        && (*(*pmv).ps).speed <= MIN_LANDING_SPEED as f32
                    {
                        //could land here, don't bounce off, in fact, return altogether!
                        return;
                    } else {
                        if (*(*pSelfVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
                            turnFromImpact = QTRUE;
                        }
                        VectorCopy(&(*trace).plane.normal, &mut bounceDir);
                    }
                } else if (*(*pSelfVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
                    //check for impact with another fighter
                    if (*hitEnt).s.NPC_class == CLASS_VEHICLE
                        && !(*hitEnt).m_pVehicle.is_null()
                        && !(*(*hitEnt).m_pVehicle).m_pVehicleInfo.is_null()
                        && (*(*(*hitEnt).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER
                    {
                        //two vehicles hit each other, turn away from the impact
                        turnFromImpact = QTRUE;
                        turnHitEnt = QTRUE;
                        VectorSubtract(
                            &(*(*pmv).ps).origin,
                            &(*hitEnt).r.currentOrigin,
                            &mut bounceDir,
                        );
                        VectorNormalize(&mut bounceDir);
                    }
                }
                if turnFromImpact != QFALSE {
                    //bounce off impact surf and turn away
                    let mut pushDir: vec3_t = [0.0; 3];
                    let mut turnAwayAngles: vec3_t = [0.0; 3];
                    let mut turnDelta: vec3_t = [0.0; 3];
                    let mut turnStrength: f32;
                    let mut moveDir: vec3_t = [0.0; 3];
                    let mut bounceDot: f32;
                    let mut turnDivider: f32;
                    //bounce
                    if turnHitEnt == QFALSE {
                        //hit wall
                        VectorScale(
                            &bounceDir,
                            (*(*pmv).ps).speed * 0.25 / (*(*pSelfVeh).m_pVehicleInfo).mass as f32,
                            &mut pushDir,
                        );
                    } else {
                        //hit another fighter
                        if !(*hitEnt).client.is_null() {
                            VectorScale(
                                &bounceDir,
                                ((*(*pmv).ps).speed + (*(*hitEnt).client).ps.speed) * 0.5,
                                &mut pushDir,
                            );
                        } else {
                            VectorScale(
                                &bounceDir,
                                ((*(*pmv).ps).speed + (*hitEnt).s.speed) * 0.5,
                                &mut pushDir,
                            );
                        }
                        let pushDirIn = pushDir;
                        VectorScale(
                            &pushDirIn,
                            l / (*(*pSelfVeh).m_pVehicleInfo).mass as f32,
                            &mut pushDir,
                        );
                        let pushDirIn = pushDir;
                        VectorScale(&pushDirIn, 0.1, &mut pushDir);
                    }
                    VectorNormalize2(&(*(*pmv).ps).velocity, &mut moveDir);
                    bounceDot = DotProduct(&moveDir, &bounceDir) * -1.0;
                    if bounceDot < 0.1 {
                        bounceDot = 0.1;
                    }
                    let pushDirIn = pushDir;
                    VectorScale(&pushDirIn, bounceDot, &mut pushDir);
                    let velIn = (*(*pmv).ps).velocity;
                    VectorAdd(&velIn, &pushDir, &mut (*(*pmv).ps).velocity);
                    //turn
                    turnDivider = (*(*pSelfVeh).m_pVehicleInfo).mass as f32 / 400.0;
                    if turnHitEnt != QFALSE {
                        //don't turn as much when hit another ship
                        turnDivider *= 4.0;
                    }
                    if turnDivider < 0.5 {
                        turnDivider = 0.5;
                    }
                    turnStrength = magnitude / 2000.0;
                    if turnStrength < 0.1 {
                        turnStrength = 0.1;
                    } else if turnStrength > 2.0 {
                        turnStrength = 2.0;
                    }
                    //get the angles we are going to turn towards
                    vectoangles(&bounceDir, &mut turnAwayAngles);
                    //get the delta from our current angles to those new angles
                    AnglesSubtract(
                        &turnAwayAngles,
                        &*((*pSelfVeh).m_vOrientation as *const vec3_t),
                        &mut turnDelta,
                    );
                    //now do pitch
                    if bounceDir[2] == 0.0 {
                        //shouldn't be any pitch
                    } else {
                        let pitchTurnStrength = turnStrength * turnDelta[PITCH];
                        let pitchTurnStrength = if pitchTurnStrength > MAX_IMPACT_TURN_ANGLE {
                            MAX_IMPACT_TURN_ANGLE
                        } else if pitchTurnStrength < -MAX_IMPACT_TURN_ANGLE {
                            -MAX_IMPACT_TURN_ANGLE
                        } else {
                            pitchTurnStrength
                        };
                        //pSelfVeh->m_vOrientation[PITCH] = AngleNormalize180(pSelfVeh->m_vOrientation[PITCH]+pitchTurnStrength/turnDivider*pSelfVeh->m_fTimeModifier);
                        (*pSelfVeh).m_vFullAngleVelocity[PITCH] = AngleNormalize180(
                            *(*pSelfVeh).m_vOrientation.add(PITCH)
                                + pitchTurnStrength / turnDivider * (*pSelfVeh).m_fTimeModifier,
                        );
                    }
                    //now do yaw
                    if bounceDir[0] == 0.0 && bounceDir[1] == 0.0 {
                        //shouldn't be any yaw
                    } else {
                        let yawTurnStrength = turnStrength * turnDelta[YAW];
                        let yawTurnStrength = if yawTurnStrength > MAX_IMPACT_TURN_ANGLE {
                            MAX_IMPACT_TURN_ANGLE
                        } else if yawTurnStrength < -MAX_IMPACT_TURN_ANGLE {
                            -MAX_IMPACT_TURN_ANGLE
                        } else {
                            yawTurnStrength
                        };
                        //pSelfVeh->m_vOrientation[ROLL] = AngleNormalize180(pSelfVeh->m_vOrientation[ROLL]-yawTurnStrength/turnDivider*pSelfVeh->m_fTimeModifier);
                        (*pSelfVeh).m_vFullAngleVelocity[ROLL] = AngleNormalize180(
                            *(*pSelfVeh).m_vOrientation.add(ROLL)
                                - yawTurnStrength / turnDivider * (*pSelfVeh).m_fTimeModifier,
                        );
                    }
                    /*
                    PM_SetPMViewAngle(pm->ps, pSelfVeh->m_vOrientation, &pSelfVeh->m_ucmd);
                    if ( pm_entVeh )
                    {//I'm a vehicle, so pm_entVeh is actually my pilot
                        bgEntity_t *pilot = pm_entVeh;
                        if ( !BG_UnrestrainedPitchRoll( pilot->playerState, pSelfVeh ) )
                        {
                            //set the rider's viewangles to the vehicle's viewangles
                            PM_SetPMViewAngle(pilot->playerState, pSelfVeh->m_vOrientation, &pSelfVeh->m_ucmd);
                        }
                    }
                    */
                    //server-side, turn the guy we hit away from us, too
                    if turnHitEnt != QFALSE//make the other guy turn and get pushed
                        && !(*hitEnt).client.is_null() //must be a valid client
                        && FighterIsLanded((*hitEnt).m_pVehicle, &mut (*(*hitEnt).client).ps) == QFALSE//but not if landed
                        && ((*hitEnt).spawnflags & 2) == 0
                    //and not if suspended
                    {
                        let l = (*(*hitEnt).client).ps.speed;
                        //now bounce *them* away and turn them
                        //flip the bounceDir
                        let bounceDirIn = bounceDir;
                        VectorScale(&bounceDirIn, -1.0, &mut bounceDir);
                        //do bounce
                        VectorScale(&bounceDir, ((*(*pmv).ps).speed + l) * 0.5, &mut pushDir);
                        let pushDirIn = pushDir;
                        VectorScale(
                            &pushDirIn,
                            l * 0.5 / (*(*(*hitEnt).m_pVehicle).m_pVehicleInfo).mass as f32,
                            &mut pushDir,
                        );
                        VectorNormalize2(&(*(*hitEnt).client).ps.velocity, &mut moveDir);
                        bounceDot = DotProduct(&moveDir, &bounceDir) * -1.0;
                        if bounceDot < 0.1 {
                            bounceDot = 0.1;
                        }
                        let pushDirIn = pushDir;
                        VectorScale(&pushDirIn, bounceDot, &mut pushDir);
                        let hitVelIn = (*(*hitEnt).client).ps.velocity;
                        VectorAdd(&hitVelIn, &pushDir, &mut (*(*hitEnt).client).ps.velocity);
                        //turn
                        turnDivider = (*(*(*hitEnt).m_pVehicle).m_pVehicleInfo).mass as f32 / 400.0;
                        if turnHitEnt != QFALSE {
                            //don't turn as much when hit another ship
                            turnDivider *= 4.0;
                        }
                        if turnDivider < 0.5 {
                            turnDivider = 0.5;
                        }
                        //get the angles we are going to turn towards
                        vectoangles(&bounceDir, &mut turnAwayAngles);
                        //get the delta from our current angles to those new angles
                        AnglesSubtract(
                            &turnAwayAngles,
                            &*((*(*hitEnt).m_pVehicle).m_vOrientation as *const vec3_t),
                            &mut turnDelta,
                        );
                        //now do pitch
                        if bounceDir[2] == 0.0 {
                            //shouldn't be any pitch
                        } else {
                            let pitchTurnStrength = turnStrength * turnDelta[PITCH];
                            let pitchTurnStrength = if pitchTurnStrength > MAX_IMPACT_TURN_ANGLE {
                                MAX_IMPACT_TURN_ANGLE
                            } else if pitchTurnStrength < -MAX_IMPACT_TURN_ANGLE {
                                -MAX_IMPACT_TURN_ANGLE
                            } else {
                                pitchTurnStrength
                            };
                            //hitEnt->m_pVehicle->m_vOrientation[PITCH] = AngleNormalize180(hitEnt->m_pVehicle->m_vOrientation[PITCH]+pitchTurnStrength/turnDivider*pSelfVeh->m_fTimeModifier);
                            (*(*hitEnt).m_pVehicle).m_vFullAngleVelocity[PITCH] = AngleNormalize180(
                                *(*(*hitEnt).m_pVehicle).m_vOrientation.add(PITCH)
                                    + pitchTurnStrength / turnDivider * (*pSelfVeh).m_fTimeModifier,
                            );
                        }
                        //now do yaw
                        if bounceDir[0] == 0.0 && bounceDir[1] == 0.0 {
                            //shouldn't be any yaw
                        } else {
                            let yawTurnStrength = turnStrength * turnDelta[YAW];
                            let yawTurnStrength = if yawTurnStrength > MAX_IMPACT_TURN_ANGLE {
                                MAX_IMPACT_TURN_ANGLE
                            } else if yawTurnStrength < -MAX_IMPACT_TURN_ANGLE {
                                -MAX_IMPACT_TURN_ANGLE
                            } else {
                                yawTurnStrength
                            };
                            //hitEnt->m_pVehicle->m_vOrientation[ROLL] = AngleNormalize180(hitEnt->m_pVehicle->m_vOrientation[ROLL]-yawTurnStrength/turnDivider*pSelfVeh->m_fTimeModifier);
                            (*(*hitEnt).m_pVehicle).m_vFullAngleVelocity[ROLL] = AngleNormalize180(
                                *(*(*hitEnt).m_pVehicle).m_vOrientation.add(ROLL)
                                    - yawTurnStrength / turnDivider * (*pSelfVeh).m_fTimeModifier,
                            );
                        }
                        //NOTE: will these angle changes stick or will they be stomped
                        //		when the vehicle goes through its own update and re-grabs
                        //		its angles from its pilot...?  Should we do a
                        //		SetClientViewAngles on the pilot?
                        /*
                        SetClientViewAngle( hitEnt, hitEnt->m_pVehicle->m_vOrientation );
                        if ( hitEnt->m_pVehicle->m_pPilot
                            && ((gentity_t *)hitEnt->m_pVehicle->m_pPilot)->client )
                        {
                            SetClientViewAngle( (gentity_t *)hitEnt->m_pVehicle->m_pPilot, hitEnt->m_pVehicle->m_vOrientation );
                        }
                        */
                    }
                }
            }

            if hitEnt.is_null() {
                return;
            }

            AngleVectors(
                &*((*pSelfVeh).m_vOrientation as *const vec3_t),
                None,
                None,
                Some(&mut vehUp),
            );
            if (*(*pSelfVeh).m_pVehicleInfo).iImpactFX != 0 {
                //G_PlayEffectID( pSelfVeh->m_pVehicleInfo->iImpactFX, pm->ps->origin, vehUp );
                //tempent use bad!
                G_AddEvent(
                    pEnt as *mut gentity_t,
                    EV_PLAY_EFFECT_ID,
                    (*(*pSelfVeh).m_pVehicleInfo).iImpactFX,
                );
            }
            (*(*pEnt).m_pVehicle).m_iHitDebounce = (*pmv).cmd.serverTime + 200;
            magnitude /= (*(*pSelfVeh).m_pVehicleInfo).toughness * 50.0;

            if !hitEnt.is_null()
                && ((*hitEnt).s.eType != ET_TERRAIN
                    || ((*hitEnt).spawnflags & 1) == 0
                    || (*(*pSelfVeh).m_pVehicleInfo).r#type == VH_FIGHTER)
            {
                //don't damage the vehicle from terrain that doesn't want to damage vehicles
                let mut killer: *mut gentity_t = core::ptr::null_mut();
                if (*(*pSelfVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
                    //increase the damage...
                    let mut mult: f32 = *(*pSelfVeh).m_vOrientation.add(PITCH) * 0.1;
                    if mult < 1.0 {
                        mult = 1.0;
                    }
                    if (*hitEnt).inuse != QFALSE && (*hitEnt).takedamage != QFALSE {
                        //if the other guy takes damage, don't hurt us a lot for ramming him
                        //unless it's a vehicle, then we get 1.5 times damage
                        if (*hitEnt).s.eType == ET_NPC
                            && (*hitEnt).s.NPC_class == CLASS_VEHICLE
                            && !(*hitEnt).m_pVehicle.is_null()
                        {
                            mult = 1.5;
                        } else {
                            mult = 0.5;
                        }
                    }

                    magnitude *= mult;
                }
                (*pSelfVeh).m_iLastImpactDmg = magnitude as c_int;
                //FIXME: what about proper death credit to the guy who shot you down?
                //FIXME: actually damage part of the ship that impacted?
                if (*hitEnt).s.eType == ET_MISSILE {
                    //missile
                    //FIX: NEVER do or take impact damage from a missile...
                    noDamage = QTRUE;
                    if ((*hitEnt).s.eFlags & EF_JETPACK_ACTIVE) != 0//vehicle missile
                        && (*hitEnt).r.ownerNum < MAX_CLIENTS as c_int
                    //valid client owner
                    {
                        //I ran into a missile and died because of the impact, give credit to the missile's owner (PROBLEM: might this ever accidently give improper credit to client 0?)
                        /*
                        if ( ((gentity_t *)hitEnt)->r.ownerNum == pEnt->s.number )
                        {//hit our own missile?  Don't damage ourselves or it... (so we don't kill ourselves!) if it hits *us*, then fine, but not here
                            noDamage = qtrue;
                        }
                        */
                        killer = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .add((*hitEnt).r.ownerNum as usize);
                    }
                }
                if noDamage == QFALSE {
                    G_Damage(
                        pEnt as *mut gentity_t,
                        hitEnt,
                        if !killer.is_null() { killer } else { hitEnt },
                        core::ptr::null_mut(),
                        (*(*pmv).ps).origin.as_mut_ptr() as *mut vec3_t,
                        (magnitude * 5.0) as c_int,
                        DAMAGE_NO_ARMOR,
                        if (*hitEnt).s.NPC_class == CLASS_VEHICLE {
                            MOD_COLLISION
                        } else {
                            MOD_FALLING
                        },
                    ); //FIXME: MOD_IMPACT
                }

                if (*(*pSelfVeh).m_pVehicleInfo).surfDestruction != 0 {
                    G_FlyVehicleSurfaceDestruction(
                        pEnt as *mut gentity_t,
                        trace,
                        magnitude as c_int,
                        forceSurfDestruction,
                    );
                }

                (*pSelfVeh).m_ulFlags |= VEH_CRASHING as c_ulong;
            }

            if !hitEnt.is_null() && (*hitEnt).inuse != QFALSE && (*hitEnt).takedamage != QFALSE {
                //damage this guy because we hit him
                let mut pmult: f32 = 1.0;
                let mut finalD: c_int;
                let attackEnt: *mut gentity_t;

                if ((*hitEnt).s.eType == ET_PLAYER && (*hitEnt).s.number < MAX_CLIENTS as c_int)
                    || ((*hitEnt).s.eType == ET_NPC && (*hitEnt).s.NPC_class != CLASS_VEHICLE)
                {
                    //probably a humanoid, or something
                    if (*(*pSelfVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
                        //player die good.. if me fighter
                        pmult = 2000.0;
                    } else {
                        pmult = 40.0;
                    }

                    if !(*hitEnt).client.is_null()
                        && BG_KnockDownable(&mut (*(*hitEnt).client).ps) != QFALSE
                        && G_CanBeEnemy(pEnt as *mut gentity_t, hitEnt) != QFALSE
                    {
                        //smash!
                        if (*(*hitEnt).client).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN {
                            (*(*hitEnt).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
                            (*(*hitEnt).client).ps.forceHandExtendTime =
                                (*pmv).cmd.serverTime + 1100;
                            (*(*hitEnt).client).ps.forceDodgeAnim = 0; //this toggles between 1 and 0, when it's 1 we should play the get up anim
                        }

                        (*(*hitEnt).client).ps.otherKiller = (*pEnt).s.number;
                        (*(*hitEnt).client).ps.otherKillerTime = (*pmv).cmd.serverTime + 5000;
                        (*(*hitEnt).client).ps.otherKillerDebounceTime =
                            (*pmv).cmd.serverTime + 100;
                        (*(*hitEnt).client).otherKillerMOD = MOD_COLLISION;
                        (*(*hitEnt).client).otherKillerVehWeapon = 0;
                        (*(*hitEnt).client).otherKillerWeaponType = WP_NONE;

                        //add my velocity into his to force him along in the correct direction from impact
                        let hitVelIn = (*(*hitEnt).client).ps.velocity;
                        VectorAdd(
                            &hitVelIn,
                            &(*(*pmv).ps).velocity,
                            &mut (*(*hitEnt).client).ps.velocity,
                        );
                        //upward thrust
                        (*(*hitEnt).client).ps.velocity[2] += 200.0;
                    }
                }

                if !(*pSelfVeh).m_pPilot.is_null() {
                    attackEnt = (*pSelfVeh).m_pPilot as *mut gentity_t;
                } else {
                    attackEnt = pEnt as *mut gentity_t;
                }

                finalD = (magnitude * pmult) as c_int;
                if finalD < 1 {
                    finalD = 1;
                }
                if noDamage == QFALSE {
                    G_Damage(
                        hitEnt,
                        attackEnt,
                        attackEnt,
                        core::ptr::null_mut(),
                        (*(*pmv).ps).origin.as_mut_ptr() as *mut vec3_t,
                        finalD,
                        0,
                        if (*hitEnt).s.NPC_class == CLASS_VEHICLE {
                            MOD_COLLISION
                        } else {
                            MOD_FALLING /*MOD_MELEE*/
                        },
                    ); //FIXME: MOD_IMPACT
                }
            }
        }
    }
}

/// `PM_GroundSlideOkay` (bg_slidemove.c:555) — `qfalse` only when rising up a wall-run /
/// long-leap / rebound-jump animation off a positive-z (upward) plane, so the slide loop
/// never pushes the player up off a sloped wall mid-wall-run. `qtrue` otherwise.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_GroundSlideOkay(zNormal: f32) -> qboolean {
    let pmv = *addr_of!(pm);

    if zNormal > 0.0 {
        if (*(*pmv).ps).velocity[2] > 0.0 {
            if (*(*pmv).ps).legsAnim == BOTH_WALL_RUN_RIGHT
                || (*(*pmv).ps).legsAnim == BOTH_WALL_RUN_LEFT
                || (*(*pmv).ps).legsAnim == BOTH_WALL_RUN_RIGHT_STOP
                || (*(*pmv).ps).legsAnim == BOTH_WALL_RUN_LEFT_STOP
                || (*(*pmv).ps).legsAnim == BOTH_FORCEWALLRUNFLIP_START
                || (*(*pmv).ps).legsAnim == BOTH_FORCELONGLEAP_START
                || (*(*pmv).ps).legsAnim == BOTH_FORCELONGLEAP_ATTACK
                || (*(*pmv).ps).legsAnim == BOTH_FORCELONGLEAP_LAND
                || BG_InReboundJump((*(*pmv).ps).legsAnim) != QFALSE
            {
                return QFALSE;
            }
        }
    }
    QTRUE
}

/*
===============
qboolean PM_ClientImpact( trace_t *trace, qboolean damageSelf )

===============
*/
/// `PM_ClientImpact` (bg_slidemove.c:585, `#ifdef QAGAME`) — slide-loop callback for a
/// non-world impact: returns `qtrue` if the hit entity is dead / no longer in the way
/// (so the loop should skip clipping against it), `qfalse` if it should still block.
///
/// The breakable-brush impact side-effect (`Client_CheckImpactBBrush`) does not affect
/// the return value (movement parity), so it is carried as a marked not-yet-ported stub.
///
/// # Safety
/// `pm`/`pm_entSelf`/`g_entities` must be valid; `trace` must point to a valid `trace_t`.
pub unsafe fn PM_ClientImpact(trace: *mut trace_t) -> qboolean {
    let pmv = *addr_of!(pm);
    //don't try to predict this
    let traceEnt: *mut gentity_t;
    let otherEntityNum: c_int = (*trace).entityNum as c_int;

    if (*addr_of!(pm_entSelf)).is_null() {
        return QFALSE;
    }

    if otherEntityNum >= ENTITYNUM_WORLD {
        return QFALSE;
    }

    traceEnt =
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(otherEntityNum as usize);

    if VectorLength(&(*(*pmv).ps).velocity) >= 100.0
        && (*(*addr_of!(pm_entSelf))).s.NPC_class != CLASS_VEHICLE
        && (*(*pmv).ps).lastOnGround + 100 < (*addr_of!(level)).time
    //&& pm->ps->groundEntityNum == ENTITYNUM_NONE )
    {
        Client_CheckImpactBBrush(
            *addr_of!(pm_entSelf) as *mut gentity_t,
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(otherEntityNum as usize),
        );
    }

    // C: `if ( !traceEnt || ... )` — `traceEnt` is `&g_entities[n]`, never null, so that
    // half of the guard can never fire; carried faithfully via the contents test alone.
    if (*traceEnt).r.contents & (*pmv).tracemask == 0 {
        //it's dead or not in my way anymore, don't clip against it
        return QTRUE;
    }

    QFALSE
}

/*
==================
PM_SlideMove

Returns qtrue if the velocity was clipped in some way
==================
*/
const MAX_CLIP_PLANES: usize = 5;

/// `PM_SlideMove` (bg_slidemove.c:629) — advance `pm->ps->origin` by `pml.frametime`
/// along `pm->ps->velocity`, clipping the velocity against up to five impact planes per
/// bump (max four bumps), so the player slides along walls/floors instead of stopping
/// dead. Returns `qtrue` if the velocity was clipped (i.e. any bump occurred).
///
/// # Safety
/// `pm`/`pml` must be valid (with a live `ps` + `trace` callback); `pm_entSelf` valid
/// where the NPC-vehicle branch reads it.
pub unsafe fn PM_SlideMove(gravity: qboolean) -> qboolean {
    let pmv = *addr_of!(pm);
    let numbumps: c_int;
    let mut dir: vec3_t = [0.0; 3];
    let mut d: f32;
    let mut numplanes: c_int;
    let mut normal: vec3_t = [0.0; 3];
    let mut planes: [vec3_t; MAX_CLIP_PLANES] = [[0.0; 3]; MAX_CLIP_PLANES];
    let mut primal_velocity: vec3_t = [0.0; 3];
    let mut clipVelocity: vec3_t = [0.0; 3];
    let mut trace: trace_t;
    let mut end: vec3_t = [0.0; 3];
    let mut time_left: f32;
    let mut into: f32;
    let mut endVelocity: vec3_t = [0.0; 3];
    let mut endClipVelocity: vec3_t = [0.0; 3];
    //qboolean	damageSelf = qtrue;

    numbumps = 4;

    VectorCopy(&(*(*pmv).ps).velocity, &mut primal_velocity);

    if gravity != QFALSE {
        VectorCopy(&(*(*pmv).ps).velocity, &mut endVelocity);
        endVelocity[2] -= (*(*pmv).ps).gravity as f32 * (*addr_of!(pml)).frametime;
        (*(*pmv).ps).velocity[2] = ((*(*pmv).ps).velocity[2] + endVelocity[2]) * 0.5;
        primal_velocity[2] = endVelocity[2];
        if (*addr_of!(pml)).groundPlane != QFALSE {
            if PM_GroundSlideOkay((*addr_of!(pml)).groundTrace.plane.normal[2]) != QFALSE {
                // slide along the ground plane
                PM_ClipVelocity(
                    (*(*pmv).ps).velocity.as_mut_ptr(),
                    (*addr_of!(pml)).groundTrace.plane.normal.as_ptr() as *mut _,
                    (*(*pmv).ps).velocity.as_mut_ptr(),
                    OVERCLIP,
                );
            }
        }
    }

    time_left = (*addr_of!(pml)).frametime;

    // never turn against the ground plane
    if (*addr_of!(pml)).groundPlane != QFALSE {
        numplanes = 1;
        VectorCopy(&(*addr_of!(pml)).groundTrace.plane.normal, &mut planes[0]);
        if PM_GroundSlideOkay(planes[0][2]) == QFALSE {
            planes[0][2] = 0.0;
            VectorNormalize(&mut planes[0]);
        }
    } else {
        numplanes = 0;
    }

    // never turn against original velocity
    VectorNormalize2(&(*(*pmv).ps).velocity, &mut planes[numplanes as usize]);
    numplanes += 1;

    let mut bumpcount: c_int = 0;
    while bumpcount < numbumps {
        // calculate position we are trying to move to
        VectorMA(
            &(*(*pmv).ps).origin,
            time_left,
            &(*(*pmv).ps).velocity,
            &mut end,
        );

        // see if we can make it there
        trace = core::mem::zeroed();
        ((*pmv).trace.unwrap())(
            &mut trace,
            (*(*pmv).ps).origin.as_ptr(),
            (*pmv).mins.as_ptr(),
            (*pmv).maxs.as_ptr(),
            end.as_ptr(),
            (*(*pmv).ps).clientNum,
            (*pmv).tracemask,
        );

        if trace.allsolid != 0 {
            // entity is completely trapped in another solid
            (*(*pmv).ps).velocity[2] = 0.0; // don't build up falling damage, but allow sideways acceleration
            return QTRUE;
        }

        if trace.fraction > 0.0 {
            // actually covered some distance
            VectorCopy(&trace.endpos, &mut (*(*pmv).ps).origin);
        }

        if trace.fraction == 1.0 {
            break; // moved the entire distance
        }

        // save entity for contact
        PM_AddTouchEnt(trace.entityNum as c_int);

        if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int {
            let pEnt: *mut bgEntity_t = *addr_of!(pm_entSelf);

            if !pEnt.is_null()
                && (*pEnt).s.eType == ET_NPC
                && (*pEnt).s.NPC_class == CLASS_VEHICLE
                && !(*pEnt).m_pVehicle.is_null()
            {
                //do vehicle impact stuff then
                PM_VehicleImpact(pEnt, &mut trace);
            }
        } else {
            // #ifdef QAGAME
            if PM_ClientImpact(&mut trace) != QFALSE {
                bumpcount += 1;
                continue;
            }
            // #endif
        }

        time_left -= time_left * trace.fraction;

        if numplanes >= MAX_CLIP_PLANES as c_int {
            // this shouldn't really happen
            VectorClear(&mut (*(*pmv).ps).velocity);
            return QTRUE;
        }

        VectorCopy(&trace.plane.normal, &mut normal);

        if PM_GroundSlideOkay(normal[2]) == QFALSE {
            //wall-running
            //never push up off a sloped wall
            normal[2] = 0.0;
            VectorNormalize(&mut normal);
        }
        //
        // if this is the same plane we hit before, nudge velocity
        // out along it, which fixes some epsilon issues with
        // non-axial planes
        //
        if (*(*pmv).ps).pm_flags & PMF_STUCK_TO_WALL == 0 {
            //no sliding if stuck to wall!
            let mut i: c_int = 0;
            while i < numplanes {
                if VectorCompare(&normal, &planes[i as usize]) != QFALSE {
                    //DotProduct( normal, planes[i] ) > 0.99 ) {
                    let v_snap = (*(*pmv).ps).velocity;
                    VectorAdd(&normal, &v_snap, &mut (*(*pmv).ps).velocity);
                    break;
                }
                i += 1;
            }
            if i < numplanes {
                bumpcount += 1;
                continue;
            }
        }
        VectorCopy(&normal, &mut planes[numplanes as usize]);
        numplanes += 1;

        //
        // modify velocity so it parallels all of the clip planes
        //

        // find a plane that it enters
        for i in 0..numplanes {
            into = DotProduct(&(*(*pmv).ps).velocity, &planes[i as usize]);
            if into >= 0.1 {
                continue; // move doesn't interact with the plane
            }

            // see how hard we are hitting things
            if -into > (*addr_of!(pml)).impactSpeed {
                (*addr_of_mut!(pml)).impactSpeed = -into;
            }

            // slide along the plane
            PM_ClipVelocity(
                (*(*pmv).ps).velocity.as_mut_ptr(),
                planes[i as usize].as_ptr() as *mut _,
                clipVelocity.as_mut_ptr(),
                OVERCLIP,
            );

            // slide along the plane
            PM_ClipVelocity(
                endVelocity.as_mut_ptr(),
                planes[i as usize].as_ptr() as *mut _,
                endClipVelocity.as_mut_ptr(),
                OVERCLIP,
            );

            // see if there is a second plane that the new move enters
            for j in 0..numplanes {
                if j == i {
                    continue;
                }
                if DotProduct(&clipVelocity, &planes[j as usize]) >= 0.1 {
                    continue; // move doesn't interact with the plane
                }

                // try clipping the move to the plane
                PM_ClipVelocity(
                    clipVelocity.as_mut_ptr(),
                    planes[j as usize].as_ptr() as *mut _,
                    clipVelocity.as_mut_ptr(),
                    OVERCLIP,
                );
                PM_ClipVelocity(
                    endClipVelocity.as_mut_ptr(),
                    planes[j as usize].as_ptr() as *mut _,
                    endClipVelocity.as_mut_ptr(),
                    OVERCLIP,
                );

                // see if it goes back into the first clip plane
                if DotProduct(&clipVelocity, &planes[i as usize]) >= 0.0 {
                    continue;
                }

                // slide the original velocity along the crease
                CrossProduct(&planes[i as usize], &planes[j as usize], &mut dir);
                VectorNormalize(&mut dir);
                d = DotProduct(&dir, &(*(*pmv).ps).velocity);
                VectorScale(&dir, d, &mut clipVelocity);

                CrossProduct(&planes[i as usize], &planes[j as usize], &mut dir);
                VectorNormalize(&mut dir);
                d = DotProduct(&dir, &endVelocity);
                VectorScale(&dir, d, &mut endClipVelocity);

                // see if there is a third plane the the new move enters
                for k in 0..numplanes {
                    if k == i || k == j {
                        continue;
                    }
                    if DotProduct(&clipVelocity, &planes[k as usize]) >= 0.1 {
                        continue; // move doesn't interact with the plane
                    }

                    // stop dead at a triple plane interaction
                    VectorClear(&mut (*(*pmv).ps).velocity);
                    return QTRUE;
                }
            }

            // if we have fixed all interactions, try another move
            VectorCopy(&clipVelocity, &mut (*(*pmv).ps).velocity);
            VectorCopy(&endClipVelocity, &mut endVelocity);
            break;
        }

        bumpcount += 1;
    }

    if gravity != QFALSE {
        VectorCopy(&endVelocity, &mut (*(*pmv).ps).velocity);
    }

    // don't change velocity if in a timer (FIXME: is this correct?)
    if (*(*pmv).ps).pm_time != 0 {
        VectorCopy(&primal_velocity, &mut (*(*pmv).ps).velocity);
    }

    if bumpcount != 0 {
        QTRUE
    } else {
        QFALSE
    }
}

/*
==================
PM_StepSlideMove

==================
*/
/// `PM_StepSlideMove` (bg_slidemove.c:856) — run [`PM_SlideMove`], and if it was blocked,
/// try the same move from a step-height higher then settle back down, so the player walks
/// up stairs. Giant NPCs (AT-ST / Rancor / walker vehicles) step higher; `stepSlideFix`
/// adds the steep-slope guards. Emits an `EV_STEP_*` event sized to the step height.
///
/// # Safety
/// `pm`/`pm_entSelf` must be valid; the vehicle pointer chain is read only under the
/// `clientNum >= MAX_CLIENTS` NPC guards, matching C.
pub unsafe fn PM_StepSlideMove(gravity: qboolean) {
    let pmv = *addr_of!(pm);
    let mut start_o: vec3_t = [0.0; 3];
    let mut start_v: vec3_t = [0.0; 3];
    let mut down_o: vec3_t = [0.0; 3];
    let mut down_v: vec3_t = [0.0; 3];
    let mut trace: trace_t;
    //	float		down_dist, up_dist;
    //	vec3_t		delta, delta2;
    let mut up: vec3_t = [0.0; 3];
    let mut down: vec3_t = [0.0; 3];
    let stepSize: f32;
    let mut isGiant: qboolean = QFALSE;
    let pEnt: *mut bgEntity_t;
    let mut skipStep: qboolean = QFALSE;

    let mut gravity = gravity;

    VectorCopy(&(*(*pmv).ps).origin, &mut start_o);
    VectorCopy(&(*(*pmv).ps).velocity, &mut start_v);

    if BG_InReboundHold((*(*pmv).ps).legsAnim) != QFALSE {
        gravity = QFALSE;
    }

    if PM_SlideMove(gravity) == QFALSE {
        return; // we got exactly where we wanted to go first try
    }

    pEnt = *addr_of!(pm_entSelf);

    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int {
        if !pEnt.is_null()
            && (*pEnt).s.NPC_class == CLASS_VEHICLE
            && !(*pEnt).m_pVehicle.is_null()
            && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).hoverHeight > 0.0
        {
            return;
        }
    }

    VectorCopy(&start_o, &mut down);
    down[2] -= STEPSIZE as f32;
    trace = core::mem::zeroed();
    ((*pmv).trace.unwrap())(
        &mut trace,
        start_o.as_ptr(),
        (*pmv).mins.as_ptr(),
        (*pmv).maxs.as_ptr(),
        down.as_ptr(),
        (*(*pmv).ps).clientNum,
        (*pmv).tracemask,
    );
    VectorSet(&mut up, 0.0, 0.0, 1.0);
    // never step up when you still have up velocity
    if (*(*pmv).ps).velocity[2] > 0.0
        && (trace.fraction == 1.0 || DotProduct(&trace.plane.normal, &up) < 0.7)
    {
        return;
    }

    VectorCopy(&(*(*pmv).ps).origin, &mut down_o);
    VectorCopy(&(*(*pmv).ps).velocity, &mut down_v);

    VectorCopy(&start_o, &mut up);

    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int {
        // apply ground friction, even if on ladder
        if (!pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_ATST)
            || ((*pEnt).s.NPC_class == CLASS_VEHICLE
                && !(*pEnt).m_pVehicle.is_null()
                && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER)
        {
            //AT-STs can step high
            up[2] += 66.0;
            isGiant = QTRUE;
        } else if !pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_RANCOR {
            //also can step up high
            up[2] += 64.0;
            isGiant = QTRUE;
        } else {
            up[2] += STEPSIZE as f32;
        }
    } else {
        up[2] += STEPSIZE as f32;
    }

    // test the player position if they were a stepheight higher
    trace = core::mem::zeroed();
    ((*pmv).trace.unwrap())(
        &mut trace,
        start_o.as_ptr(),
        (*pmv).mins.as_ptr(),
        (*pmv).maxs.as_ptr(),
        up.as_ptr(),
        (*(*pmv).ps).clientNum,
        (*pmv).tracemask,
    );
    if trace.allsolid != 0 {
        if (*pmv).debugLevel != 0 {
            // Com_Printf("%i:bend can't step\n", c_pmove);
        }
        return; // can't step up
    }

    stepSize = trace.endpos[2] - start_o[2];
    // try slidemove from this position
    VectorCopy(&trace.endpos, &mut (*(*pmv).ps).origin);
    VectorCopy(&start_v, &mut (*(*pmv).ps).velocity);

    PM_SlideMove(gravity);

    // push down the final amount
    VectorCopy(&(*(*pmv).ps).origin, &mut down);
    down[2] -= stepSize;
    trace = core::mem::zeroed();
    ((*pmv).trace.unwrap())(
        &mut trace,
        (*(*pmv).ps).origin.as_ptr(),
        (*pmv).mins.as_ptr(),
        (*pmv).maxs.as_ptr(),
        down.as_ptr(),
        (*(*pmv).ps).clientNum,
        (*pmv).tracemask,
    );

    if (*pmv).stepSlideFix != 0 {
        if (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int && trace.plane.normal[2] < MIN_WALK_NORMAL
        {
            //normal players cannot step up slopes that are too steep to walk on!
            let mut stepVec: vec3_t = [0.0; 3];
            //okay, the step up ends on a slope that it too steep to step up onto,
            //BUT:
            //If the step looks like this:
            //  (B)\__
            //        \_____(A)
            //Then it might still be okay, so we figure out the slope of the entire move
            //from (A) to (B) and if that slope is walk-upabble, then it's okay
            VectorSubtract(&trace.endpos, &down_o, &mut stepVec);
            VectorNormalize(&mut stepVec);
            if stepVec[2] > (1.0 - MIN_WALK_NORMAL) {
                skipStep = QTRUE;
            }
        }
    }

    if trace.allsolid == 0 && skipStep == QFALSE {
        //normal players cannot step up slopes that are too steep to walk on!
        if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int//NPC
            && isGiant != QFALSE
            && (trace.entityNum as c_int) < MAX_CLIENTS as c_int
            && !pEnt.is_null()
            && (*pEnt).s.NPC_class == CLASS_RANCOR
        {
            //Rancor don't step on clients
            if (*pmv).stepSlideFix != 0 {
                VectorCopy(&down_o, &mut (*(*pmv).ps).origin);
                VectorCopy(&down_v, &mut (*(*pmv).ps).velocity);
            } else {
                VectorCopy(&start_o, &mut (*(*pmv).ps).origin);
                VectorCopy(&start_v, &mut (*(*pmv).ps).velocity);
            }
        } else {
            VectorCopy(&trace.endpos, &mut (*(*pmv).ps).origin);
            if (*pmv).stepSlideFix != 0 {
                if trace.fraction < 1.0 {
                    PM_ClipVelocity(
                        (*(*pmv).ps).velocity.as_mut_ptr(),
                        trace.plane.normal.as_ptr() as *mut _,
                        (*(*pmv).ps).velocity.as_mut_ptr(),
                        OVERCLIP,
                    );
                }
            }
        }
    } else if (*pmv).stepSlideFix != 0 {
        VectorCopy(&down_o, &mut (*(*pmv).ps).origin);
        VectorCopy(&down_v, &mut (*(*pmv).ps).velocity);
    }
    if (*pmv).stepSlideFix == 0 {
        if trace.fraction < 1.0 {
            PM_ClipVelocity(
                (*(*pmv).ps).velocity.as_mut_ptr(),
                trace.plane.normal.as_ptr() as *mut _,
                (*(*pmv).ps).velocity.as_mut_ptr(),
                OVERCLIP,
            );
        }
    }

    // #if 0  -- the "trace back to the original position directly" early-out is disabled
    //   in JKA; the body always takes the #else (step-move) branch. Carried as a comment.
    {
        // use the step move
        let delta: f32;

        delta = (*(*pmv).ps).origin[2] - start_o[2];
        if delta > 2.0 {
            if delta < 7.0 {
                PM_AddEvent(EV_STEP_4);
            } else if delta < 11.0 {
                PM_AddEvent(EV_STEP_8);
            } else if delta < 15.0 {
                PM_AddEvent(EV_STEP_12);
            } else {
                PM_AddEvent(EV_STEP_16);
            }
        }
        if (*pmv).debugLevel != 0 {
            // Com_Printf("%i:stepped\n", c_pmove);
        }
    }
}
