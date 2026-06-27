//! Slice of `NPC_senses.c` — the NPC sensory layer (line-of-sight, FOV, alert
//! events). Opened bottom-up at the pure-math leaves: the FOV / in-front position
//! comparators are self-contained float math over `q_math.rs` and port cleanly
//! ahead of the rest of the file.
//!
//! The trace/LOS family (`G_ClearLineOfSight`, `InFOV`/`InFOV2`, `InVisrange`,
//! `G_ClearLOS`*) and the alert-event family (`AddSoundEvent`/`AddSightEvent`,
//! `G_CheckSoundEvents`/`G_CheckSightEvents`/`G_CheckAlertEvents`,
//! `RemoveOldestAlert`/`ClearPlayerAlertEvents`, `G_FindLocalInterestPoint`/
//! `SP_target_interest`) lean on `CalcEntitySpot` (NPC_utils.c), `EntIsGlass`
//! (NPC_combat.c), the `NPC`/`NPCInfo` think globals, the `level` alert/interest
//! globals, and `trap_Trace`/`trap_InPVS` — all now landed.
//!
//! `CanSee` (`ShotThroughGlass`, NPC_combat.c) and `NPC_CheckVisibility`
//! (`CanSee`/`CanShoot`) are now landed too. `G_CheckForDanger`/`NPC_CheckForDanger`
//! land against the real `NPC_StartFlee` (npc_behavior.rs) — this file is now COMPLETE.

#![allow(non_snake_case)] // C function names (`InFOV3`) kept verbatim
#![allow(non_upper_case_globals)] // C global `eventClearTime` kept verbatim

use core::ffi::c_int;

use crate::codemp::game::b_public_h::{
    visibility_t, SCF_DONT_FLEE, SPOT_HEAD, SPOT_HEAD_LEAN, SPOT_LEGS, SPOT_ORIGIN, VIS_360,
    VIS_FOV, VIS_NOT, VIS_PVS, VIS_SHOOT,
};
use crate::codemp::game::bg_public::MASK_OPAQUE;
use crate::codemp::game::g_local::{
    alertEvent_t, alertEventLevel_e, gentity_t, AEL_DANGER, AET_SIGHT, AET_SOUND, MAX_ALERT_EVENTS,
    MAX_INTEREST_POINTS,
};
use crate::codemp::game::g_main::{g_entities, level, Com_Printf};
use crate::codemp::game::g_public_h::{Q3_INFINITE, SVF_GLASS_BRUSH};
use crate::codemp::game::g_spawn::G_NewString;
use crate::codemp::game::g_utils::{G_FreeEntity, G_UseTargets2};
use crate::codemp::game::npc::{NPCInfo, NPC};
use crate::codemp::game::npc_behavior::NPC_StartFlee;
use crate::codemp::game::npc_combat::{CanShoot, EntIsGlass, ShotThroughGlass};
use crate::codemp::game::npc_utils::CalcEntitySpot;
use crate::codemp::game::q_math::{
    vec3_origin, AngleDelta, AngleVectors, DistanceSquared, DotProduct, VectorCompare, VectorCopy,
    VectorLengthSquared, VectorNormalize, VectorSubtract, vectoangles,
};
use crate::codemp::game::q_shared_h::{
    trace_t, vec3_t, ENTITYNUM_NONE, ENTITYNUM_WORLD, PITCH, YAW,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_OPAQUE;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `#define ALERT_CLEAR_TIME 200` (b_local.h:164). The 200 ms debounce window after
/// which a stale alert event is dropped by [`ClearPlayerAlertEvents`].
const ALERT_CLEAR_TIME: c_int = 200;

/// `#define MAX_INTEREST_DIST ( 256 * 256 )` (NPC_senses.c:864). Squared cutoff for
/// [`G_FindLocalInterestPoint`].
const MAX_INTEREST_DIST: f32 = (256 * 256) as f32;

// `CHECK_*` visibility-test flag bits (b_local.h:165-169) — the `flags` arg to
// [`NPC_CheckVisibility`].
/// `#define CHECK_PVS 1` (b_local.h:165).
const CHECK_PVS: c_int = 1;
/// `#define CHECK_360 2` (b_local.h:166).
const CHECK_360: c_int = 2;
/// `#define CHECK_FOV 4` (b_local.h:167).
const CHECK_FOV: c_int = 4;
/// `#define CHECK_SHOOT 8` (b_local.h:168).
const CHECK_SHOOT: c_int = 8;
/// `#define CHECK_VISRANGE 16` (b_local.h:169).
const CHECK_VISRANGE: c_int = 16;

/// `extern int eventClearTime;` (NPC_senses.c:5) — defined here as the file-static
/// owner; the 200 ms debouncer shared with NPC.c so corpses/missiles only add an
/// alert every `ALERT_CLEAR_TIME`.
pub static mut eventClearTime: c_int = 0;

/// `qboolean G_ClearLineOfSight( const vec3_t point1, const vec3_t point2, int ignore, int clipmask )`
/// (NPC_senses.c:11).
///
/// returns true if can see from point 1 to 2, even through glass (1 pane)- doesn't work with portals
///
/// # Safety
/// `g_entities` must be initialised; the trapped trace touches engine state.
pub unsafe fn G_ClearLineOfSight(
    point1: &vec3_t,
    point2: &vec3_t,
    ignore: c_int,
    clipmask: c_int,
) -> qboolean {
    let mut tr: trace_t;
    let hit: *mut gentity_t;

    tr = trap::Trace(point1, &vec3_origin, &vec3_origin, point2, ignore, clipmask);
    if tr.fraction == 1.0 {
        return QTRUE;
    }

    hit = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(tr.entityNum as isize);
    if EntIsGlass(hit) == QTRUE {
        let mut newpoint1: vec3_t = [0.0; 3];
        VectorCopy(&tr.endpos, &mut newpoint1);
        tr = trap::Trace(
            &newpoint1,
            &vec3_origin,
            &vec3_origin,
            point2,
            (*hit).s.number,
            clipmask,
        );

        if tr.fraction == 1.0 {
            return QTRUE;
        }
    }

    QFALSE
}

/*
CanSee
determine if NPC can see an entity

This is a straight line trace check.  This function does not look at PVS or FOV,
or take any AI related factors (for example, the NPC's reaction time) into account

FIXME do we need fat and thin version of this?
*/
/// `qboolean CanSee ( gentity_t *ent )` (NPC_senses.c:47).
///
/// Straight-line trace check from the current think NPC's head-lean eyes to `ent`'s
/// origin, head, then legs (each retried through breakable glass via
/// [`ShotThroughGlass`]). Does NOT consider PVS, FOV, or any AI/reaction-time factors.
///
/// # Safety
/// `NPC` must be set to the current think entity; `ent` must point to a valid
/// `gentity_t`; the trapped trace touches engine state.
pub unsafe fn CanSee(ent: *mut gentity_t) -> qboolean {
    let mut tr: trace_t;
    let mut eyes: vec3_t = [0.0; 3];
    let mut spot: vec3_t = [0.0; 3];

    CalcEntitySpot(NPC, SPOT_HEAD_LEAN, &mut eyes);

    CalcEntitySpot(ent, SPOT_ORIGIN, &mut spot);
    tr = trap::Trace(&eyes, &vec3_origin, &vec3_origin, &spot, (*NPC).s.number, MASK_OPAQUE);
    ShotThroughGlass(&mut tr, ent, &spot, MASK_OPAQUE);
    if tr.fraction == 1.0 {
        return QTRUE;
    }

    CalcEntitySpot(ent, SPOT_HEAD, &mut spot);
    tr = trap::Trace(&eyes, &vec3_origin, &vec3_origin, &spot, (*NPC).s.number, MASK_OPAQUE);
    ShotThroughGlass(&mut tr, ent, &spot, MASK_OPAQUE);
    if tr.fraction == 1.0 {
        return QTRUE;
    }

    CalcEntitySpot(ent, SPOT_LEGS, &mut spot);
    tr = trap::Trace(&eyes, &vec3_origin, &vec3_origin, &spot, (*NPC).s.number, MASK_OPAQUE);
    ShotThroughGlass(&mut tr, ent, &spot, MASK_OPAQUE);
    if tr.fraction == 1.0 {
        return QTRUE;
    }

    QFALSE
}

/// `qboolean InFront( vec3_t spot, vec3_t from, vec3_t fromAngles, float threshHold )`
/// (NPC_senses.c:82).
///
/// Returns `qtrue` if `spot` lies within `threshHold` (a dot-product cutoff) of the
/// direction `from` is facing, projected onto the horizontal plane (`dir[2]` and the
/// pitch are both zeroed).
pub fn InFront(spot: &vec3_t, from: &vec3_t, fromAngles: &vec3_t, threshHold: f32) -> qboolean {
    let mut dir: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let dot: f32;

    VectorSubtract(spot, from, &mut dir);
    dir[2] = 0.0;
    VectorNormalize(&mut dir);

    VectorCopy(fromAngles, &mut angles);
    angles[0] = 0.0;
    AngleVectors(&angles, Some(&mut forward), None, None);

    dot = DotProduct(&dir, &forward);

    if dot > threshHold {
        QTRUE
    } else {
        QFALSE
    }
}

/*
InFOV

IDEA: further off to side of FOV range, higher chance of failing even if technically in FOV,
	keep core of 50% to sides as always succeeding
*/

//Position compares

/// `qboolean InFOV3( vec3_t spot, vec3_t from, vec3_t fromAngles, int hFOV, int vFOV )`
/// (NPC_senses.c:109).
///
/// Pure position-vs-position FOV test: returns `qtrue` if the angle from `from` to
/// `spot` is within `vFOV` pitch and `hFOV` yaw of `fromAngles`.
pub fn InFOV3(spot: &vec3_t, from: &vec3_t, fromAngles: &vec3_t, hFOV: i32, vFOV: i32) -> qboolean {
    let mut deltaVector: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut deltaAngles: vec3_t = [0.0; 3];

    VectorSubtract(spot, from, &mut deltaVector);
    vectoangles(&deltaVector, &mut angles);

    deltaAngles[PITCH] = AngleDelta(fromAngles[PITCH], angles[PITCH]);
    deltaAngles[YAW] = AngleDelta(fromAngles[YAW], angles[YAW]);

    if deltaAngles[PITCH].abs() <= vFOV as f32 && deltaAngles[YAW].abs() <= hFOV as f32 {
        return QTRUE;
    }

    QFALSE
}

//NPC to position

/// `qboolean InFOV2( vec3_t origin, gentity_t *from, int hFOV, int vFOV )`
/// (NPC_senses.c:129).
///
/// NPC-to-position FOV test: derives `from`'s facing (client viewangles or
/// `s.angles`) and head eyespot, then defers to [`InFOV3`].
///
/// # Safety
/// `from` must point to a valid `gentity_t`.
pub unsafe fn InFOV2(origin: &vec3_t, from: *mut gentity_t, hFOV: c_int, vFOV: c_int) -> qboolean {
    let mut fromAngles: vec3_t = [0.0; 3];
    let mut eyes: vec3_t = [0.0; 3];

    if !(*from).client.is_null() {
        VectorCopy(&(*(*from).client).ps.viewangles, &mut fromAngles);
    } else {
        VectorCopy(&(*from).s.angles, &mut fromAngles);
    }

    CalcEntitySpot(from, SPOT_HEAD, &mut eyes);

    InFOV3(origin, &eyes, &fromAngles, hFOV, vFOV)
}

//Entity to entity

/// `qboolean InFOV ( gentity_t *ent, gentity_t *from, int hFOV, int vFOV )`
/// (NPC_senses.c:149).
///
/// Entity-to-entity FOV test: uses `from`'s tag_head facing if its `renderInfo.eyeAngles`
/// is set (else client viewangles / `s.angles`), then checks `ent`'s origin, head, and
/// legs spots in turn against `from`'s head-lean eyespot.
///
/// # Safety
/// `ent` and `from` must point to valid `gentity_t`.
pub unsafe fn InFOV(
    ent: *mut gentity_t,
    from: *mut gentity_t,
    hFOV: c_int,
    vFOV: c_int,
) -> qboolean {
    let mut eyes: vec3_t = [0.0; 3];
    let mut spot: vec3_t = [0.0; 3];
    let mut deltaVector: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut fromAngles: vec3_t = [0.0; 3];
    let mut deltaAngles: vec3_t = [0.0; 3];

    if !(*from).client.is_null() {
        if VectorCompare(&(*(*from).client).renderInfo.eyeAngles, &vec3_origin) == 0 {
            //Actual facing of tag_head!
            //NOTE: Stasis aliens may have a problem with this?
            VectorCopy(&(*(*from).client).renderInfo.eyeAngles, &mut fromAngles);
        } else {
            VectorCopy(&(*(*from).client).ps.viewangles, &mut fromAngles);
        }
    } else {
        VectorCopy(&(*from).s.angles, &mut fromAngles);
    }

    CalcEntitySpot(from, SPOT_HEAD_LEAN, &mut eyes);

    CalcEntitySpot(ent, SPOT_ORIGIN, &mut spot);
    VectorSubtract(&spot, &eyes, &mut deltaVector);

    vectoangles(&deltaVector, &mut angles);
    deltaAngles[PITCH] = AngleDelta(fromAngles[PITCH], angles[PITCH]);
    deltaAngles[YAW] = AngleDelta(fromAngles[YAW], angles[YAW]);
    if deltaAngles[PITCH].abs() <= vFOV as f32 && deltaAngles[YAW].abs() <= hFOV as f32 {
        return QTRUE;
    }

    CalcEntitySpot(ent, SPOT_HEAD, &mut spot);
    VectorSubtract(&spot, &eyes, &mut deltaVector);
    vectoangles(&deltaVector, &mut angles);
    deltaAngles[PITCH] = AngleDelta(fromAngles[PITCH], angles[PITCH]);
    deltaAngles[YAW] = AngleDelta(fromAngles[YAW], angles[YAW]);
    if deltaAngles[PITCH].abs() <= vFOV as f32 && deltaAngles[YAW].abs() <= hFOV as f32 {
        return QTRUE;
    }

    CalcEntitySpot(ent, SPOT_LEGS, &mut spot);
    VectorSubtract(&spot, &eyes, &mut deltaVector);
    vectoangles(&deltaVector, &mut angles);
    deltaAngles[PITCH] = AngleDelta(fromAngles[PITCH], angles[PITCH]);
    deltaAngles[YAW] = AngleDelta(fromAngles[YAW], angles[YAW]);
    if deltaAngles[PITCH].abs() <= vFOV as f32 && deltaAngles[YAW].abs() <= hFOV as f32 {
        return QTRUE;
    }

    QFALSE
}

/// `qboolean InVisrange ( gentity_t *ent )` (NPC_senses.c:210).
///
/// True if `ent` is within the NPC's `visrange` (squared distance, head-lean eyespot to
/// `ent`'s origin). The commented-out velocity/turn-rate visibility boosts are kept
/// verbatim.
///
/// # Safety
/// `NPC`/`NPCInfo` must be set to the current think entity; `ent` must be valid.
pub unsafe fn InVisrange(ent: *mut gentity_t) -> qboolean {
    //FIXME: make a calculate visibility for ents that takes into account
    //lighting, movement, turning, crouch/stand up, other anims, hide brushes, etc.
    let mut eyes: vec3_t = [0.0; 3];
    let mut spot: vec3_t = [0.0; 3];
    let mut deltaVector: vec3_t = [0.0; 3];
    let visrange: f32 = (*NPCInfo).stats.visrange * (*NPCInfo).stats.visrange;

    CalcEntitySpot(NPC, SPOT_HEAD_LEAN, &mut eyes);

    CalcEntitySpot(ent, SPOT_ORIGIN, &mut spot);
    VectorSubtract(&spot, &eyes, &mut deltaVector);

    /*if(ent->client)
    {
        float	vel, avel;
        if(ent->client->ps.velocity[0] || ent->client->ps.velocity[1] || ent->client->ps.velocity[2])
        {
            vel = VectorLength(ent->client->ps.velocity);
            if(vel > 128)
            {
                visrange += visrange * (vel/256);
            }
        }

        if(ent->avelocity[0] || ent->avelocity[1] || ent->avelocity[2])
        {//FIXME: shouldn't they need to have line of sight to you to detect this?
            avel = VectorLength(ent->avelocity);
            if(avel > 15)
            {
                visrange += visrange * (avel/60);
            }
        }
    }*/

    if VectorLengthSquared(&deltaVector) > visrange {
        return QFALSE;
    }

    QTRUE
}

/*
NPC_CheckVisibility
*/
/// `visibility_t NPC_CheckVisibility ( gentity_t *ent, int flags )` (NPC_senses.c:257).
///
/// Layered visibility test against the current think NPC, short-circuiting at the
/// coarsest failing tier: PVS → visrange → 360° LOS ([`CanSee`]) → FOV ([`InFOV`]) →
/// shootability ([`CanShoot`]), driven by the `CHECK_*` flag bits. Returns the highest
/// tier passed (`VIS_NOT` for zero flags).
///
/// # Safety
/// `NPC`/`NPCInfo` must be set to the current think entity; `ent` must point to a valid
/// `gentity_t`; the trapped trace/PVS queries touch engine state.
pub unsafe fn NPC_CheckVisibility(ent: *mut gentity_t, flags: c_int) -> visibility_t {
    // flags should never be 0
    if flags == 0 {
        return VIS_NOT;
    }

    // check PVS
    if flags & CHECK_PVS != 0 {
        if trap::InPVS(&(*ent).r.currentOrigin, &(*NPC).r.currentOrigin) == QFALSE {
            return VIS_NOT;
        }
    }
    if flags & (CHECK_360 | CHECK_FOV | CHECK_SHOOT) == 0 {
        return VIS_PVS;
    }

    // check within visrange
    if flags & CHECK_VISRANGE != 0 {
        if InVisrange(ent) == QFALSE {
            return VIS_PVS;
        }
    }

    // check 360 degree visibility
    //Meaning has to be a direct line of site
    if flags & CHECK_360 != 0 {
        if CanSee(ent) == QFALSE {
            return VIS_PVS;
        }
    }
    if flags & (CHECK_FOV | CHECK_SHOOT) == 0 {
        return VIS_360;
    }

    // check FOV
    if flags & CHECK_FOV != 0 {
        if InFOV(ent, NPC, (*NPCInfo).stats.hfov, (*NPCInfo).stats.vfov) == QFALSE {
            return VIS_360;
        }
    }

    if flags & CHECK_SHOOT == 0 {
        return VIS_FOV;
    }

    // check shootability
    if flags & CHECK_SHOOT != 0 {
        if CanShoot(ent, NPC) == QFALSE {
            return VIS_FOV;
        }
    }

    VIS_SHOOT
}

/// `float G_GetLightLevel( vec3_t pos, vec3_t fromDir )` (NPC_senses.c:388).
///
/// In the MP module this is a stub: the original cgame light-grid query is commented
/// out (no server-side lightmap), so it always reports full brightness (`255`). The
/// `pos`/`fromDir` arguments are kept for signature parity but unused.
pub fn G_GetLightLevel(_pos: &vec3_t, _fromDir: &vec3_t) -> f32 {
    /*
    vec3_t	ambient={0}, directed, lightDir;

    cgi_R_GetLighting( pos, ambient, directed, lightDir );
    lightLevel = VectorLength( ambient ) + (VectorLength( directed )*DotProduct( lightDir, fromDir ));
    */
    let lightLevel: f32;
    //rwwFIXMEFIXME: ...this is evil. We can possibly read from the server BSP data, or load the lightmap along
    //with collision data and whatnot, but is it worth it?
    lightLevel = 255.0;

    lightLevel
}

/*
-------------------------
NPC_CheckSoundEvents
-------------------------
*/
/// `static int G_CheckSoundEvents( gentity_t *self, float maxHearDist, int ignoreAlert,
/// qboolean mustHaveOwner, int minAlertLevel )` (NPC_senses.c:332).
///
/// Returns the best (highest-priority, then newest) `AET_SOUND` alert event within
/// `maxHearDist` and the event's own radius, optionally requiring an owner and a
/// minimum alert level; quiet sounds (`addLight != 0`) additionally require LOS.
///
/// # Safety
/// `level` must be initialised; `self` must point to a valid `gentity_t`.
unsafe fn G_CheckSoundEvents(
    self_: *mut gentity_t,
    mut maxHearDist: f32,
    ignoreAlert: c_int,
    mustHaveOwner: qboolean,
    minAlertLevel: c_int,
) -> c_int {
    let mut bestEvent: c_int = -1;
    let mut bestAlert: c_int = -1;
    let mut bestTime: c_int = -1;
    let mut dist: f32;
    let mut radius: f32;

    maxHearDist *= maxHearDist;

    let mut i: c_int = 0;
    while i < level.numAlertEvents {
        let ae = &level.alertEvents[i as usize];
        //are we purposely ignoring this alert?
        if i == ignoreAlert {
            i += 1;
            continue;
        }
        //We're only concerned about sounds
        if ae.r#type != AET_SOUND {
            i += 1;
            continue;
        }
        //must be at least this noticable
        if ae.level < minAlertLevel {
            i += 1;
            continue;
        }
        //must have an owner?
        if mustHaveOwner == QTRUE && ae.owner.is_null() {
            i += 1;
            continue;
        }
        //Must be within range
        dist = DistanceSquared(&ae.position, &(*self_).r.currentOrigin);

        //can't hear it
        if dist > maxHearDist {
            i += 1;
            continue;
        }

        radius = ae.radius * ae.radius;
        if dist > radius {
            i += 1;
            continue;
        }

        if ae.addLight != 0.0 {
            //a quiet sound, must have LOS to hear it
            if G_ClearLOS5(self_, &ae.position) == QFALSE {
                //no LOS, didn't hear it
                i += 1;
                continue;
            }
        }

        //See if this one takes precedence over the previous one
        if ae.level >= bestAlert //higher alert level
            || (ae.level == bestAlert && ae.timestamp >= bestTime)
        //same alert level, but this one is newer
        {
            //NOTE: equal is better because it's later in the array
            bestEvent = i;
            bestAlert = ae.level;
            bestTime = ae.timestamp;
        }
        i += 1;
    }

    bestEvent
}

/*
-------------------------
NPC_CheckSightEvents
-------------------------
*/
/// `static int G_CheckSightEvents( gentity_t *self, int hFOV, int vFOV, float maxSeeDist,
/// int ignoreAlert, qboolean mustHaveOwner, int minAlertLevel )` (NPC_senses.c:408).
///
/// Sight counterpart of [`G_CheckSoundEvents`]: best `AET_SIGHT` event within range,
/// radius, [`InFOV2`], and LOS.
///
/// # Safety
/// `level` must be initialised; `self` must point to a valid `gentity_t`.
unsafe fn G_CheckSightEvents(
    self_: *mut gentity_t,
    hFOV: c_int,
    vFOV: c_int,
    mut maxSeeDist: f32,
    ignoreAlert: c_int,
    mustHaveOwner: qboolean,
    minAlertLevel: c_int,
) -> c_int {
    let mut bestEvent: c_int = -1;
    let mut bestAlert: c_int = -1;
    let mut bestTime: c_int = -1;
    let mut dist: f32;
    let mut radius: f32;

    maxSeeDist *= maxSeeDist;
    let mut i: c_int = 0;
    while i < level.numAlertEvents {
        let ae = &level.alertEvents[i as usize];
        //are we purposely ignoring this alert?
        if i == ignoreAlert {
            i += 1;
            continue;
        }
        //We're only concerned about sounds
        if ae.r#type != AET_SIGHT {
            i += 1;
            continue;
        }
        //must be at least this noticable
        if ae.level < minAlertLevel {
            i += 1;
            continue;
        }
        //must have an owner?
        if mustHaveOwner == QTRUE && ae.owner.is_null() {
            i += 1;
            continue;
        }

        //Must be within range
        dist = DistanceSquared(&ae.position, &(*self_).r.currentOrigin);

        //can't see it
        if dist > maxSeeDist {
            i += 1;
            continue;
        }

        radius = ae.radius * ae.radius;
        if dist > radius {
            i += 1;
            continue;
        }

        //Must be visible
        if InFOV2(&ae.position, self_, hFOV, vFOV) == QFALSE {
            i += 1;
            continue;
        }

        if G_ClearLOS5(self_, &ae.position) == QFALSE {
            i += 1;
            continue;
        }

        //FIXME: possibly have the light level at this point affect the
        //			visibility/alert level of this event?  Would also
        //			need to take into account how bright the event
        //			itself is.  A lightsaber would stand out more
        //			in the dark... maybe pass in a light level that
        //			is added to the actual light level at this position?

        //See if this one takes precedence over the previous one
        if ae.level >= bestAlert //higher alert level
            || (ae.level == bestAlert && ae.timestamp >= bestTime)
        //same alert level, but this one is newer
        {
            //NOTE: equal is better because it's later in the array
            bestEvent = i;
            bestAlert = ae.level;
            bestTime = ae.timestamp;
        }
        i += 1;
    }

    bestEvent
}

/*
-------------------------
NPC_CheckAlertEvents

    NOTE: Should all NPCs create alertEvents too so they can detect each other?
-------------------------
*/
/// `int G_CheckAlertEvents( gentity_t *self, qboolean checkSight, qboolean checkSound,
/// float maxSeeDist, float maxHearDist, int ignoreAlert, qboolean mustHaveOwner,
/// int minAlertLevel )` (NPC_senses.c:478).
///
/// Returns the index of the most important alert (sight vs. sound; sound wins ties),
/// updating the chosen sight event's `light` from [`G_GetLightLevel`]. Returns `-1`
/// while the player (slot 0) is dead.
///
/// # Safety
/// `g_entities`/`level` must be initialised; `self` must point to a valid `gentity_t`.
pub unsafe fn G_CheckAlertEvents(
    self_: *mut gentity_t,
    _checkSight: qboolean,
    _checkSound: qboolean,
    maxSeeDist: f32,
    maxHearDist: f32,
    ignoreAlert: c_int,
    mustHaveOwner: qboolean,
    minAlertLevel: c_int,
) -> c_int {
    let bestSoundEvent: c_int; // = -1;
    let bestSightEvent: c_int; // = -1;
    let mut bestSoundAlert: c_int = -1;
    let mut bestSightAlert: c_int = -1;

    let player: *mut gentity_t = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    if player.is_null() || (*player).health <= 0 {
        //player is dead
        return -1;
    }

    //get sound event
    bestSoundEvent =
        G_CheckSoundEvents(self_, maxHearDist, ignoreAlert, mustHaveOwner, minAlertLevel);
    //get sound event alert level
    if bestSoundEvent >= 0 {
        bestSoundAlert = level.alertEvents[bestSoundEvent as usize].level;
    }

    //get sight event
    if !(*self_).NPC.is_null() {
        bestSightEvent = G_CheckSightEvents(
            self_,
            (*(*self_).NPC).stats.hfov,
            (*(*self_).NPC).stats.vfov,
            maxSeeDist,
            ignoreAlert,
            mustHaveOwner,
            minAlertLevel,
        );
    } else {
        bestSightEvent = G_CheckSightEvents(
            self_,
            80,
            80,
            maxSeeDist,
            ignoreAlert,
            mustHaveOwner,
            minAlertLevel,
        ); //FIXME: look at cg_view to get more accurate numbers?
    }
    //get sight event alert level
    if bestSightEvent >= 0 {
        bestSightAlert = level.alertEvents[bestSightEvent as usize].level;
    }

    //return the one that has a higher alert (or sound if equal)
    //FIXME:	This doesn't take the distance of the event into account

    if bestSightEvent >= 0 && bestSightAlert > bestSoundAlert {
        //valid best sight event, more important than the sound event
        //get the light level of the alert event for this checker
        let mut eyePoint: vec3_t = [0.0; 3];
        let mut sightDir: vec3_t = [0.0; 3];
        //get eye point
        CalcEntitySpot(self_, SPOT_HEAD_LEAN, &mut eyePoint);
        VectorSubtract(
            &level.alertEvents[bestSightEvent as usize].position,
            &eyePoint,
            &mut sightDir,
        );
        level.alertEvents[bestSightEvent as usize].light = level.alertEvents
            [bestSightEvent as usize]
            .addLight
            + G_GetLightLevel(&level.alertEvents[bestSightEvent as usize].position, &sightDir);
        //return the sight event
        return bestSightEvent;
    }
    //return the sound event
    bestSoundEvent
}

/// `int NPC_CheckAlertEvents( qboolean checkSight, qboolean checkSound, int ignoreAlert,
/// qboolean mustHaveOwner, int minAlertLevel )` (NPC_senses.c:532).
///
/// Thin wrapper over [`G_CheckAlertEvents`] for the current think NPC, sourcing the
/// see/hear distances from `NPCInfo->stats.visrange`/`earshot`.
///
/// # Safety
/// `NPC`/`NPCInfo` must be set to the current think entity.
pub unsafe fn NPC_CheckAlertEvents(
    checkSight: qboolean,
    checkSound: qboolean,
    ignoreAlert: c_int,
    mustHaveOwner: qboolean,
    minAlertLevel: c_int,
) -> c_int {
    G_CheckAlertEvents(
        NPC,
        checkSight,
        checkSound,
        (*NPCInfo).stats.visrange,
        (*NPCInfo).stats.earshot,
        ignoreAlert,
        mustHaveOwner,
        minAlertLevel,
    )
}

/// `qboolean G_CheckForDanger( gentity_t *self, int alertEvent )` (NPC_senses.c:537).
///
/// If `alertEvent` indexes an event at `AEL_DANGER` or higher that is not owned by a
/// teammate (or by `self`), make `self` flee from it (unless `SCF_DONT_FLEE` is set),
/// returning whether the danger was acted on. Non-NPC entities just report the danger.
///
/// # Safety
/// `self` (and, on the team check, the event owner) must be a valid client entity; the
/// alert array up to `level.numAlertEvents` must be valid.
pub unsafe fn G_CheckForDanger(self_: *mut gentity_t, alertEvent: c_int) -> qboolean {
    //FIXME: more bStates need to call this?
    if alertEvent == -1 {
        return QFALSE;
    }

    if level.alertEvents[alertEvent as usize].level >= AEL_DANGER {
        //run away!
        let owner = level.alertEvents[alertEvent as usize].owner;
        if owner.is_null()
            || (*owner).client.is_null()
            || (owner != self_
                && (*(*owner).client).playerTeam != (*(*self_).client).playerTeam)
        {
            if !(*self_).NPC.is_null() {
                if (*(*self_).NPC).scriptFlags & SCF_DONT_FLEE != 0 {
                    //can't flee
                    return QFALSE;
                } else {
                    NPC_StartFlee(
                        level.alertEvents[alertEvent as usize].owner,
                        &level.alertEvents[alertEvent as usize].position,
                        level.alertEvents[alertEvent as usize].level,
                        3000,
                        6000,
                    );
                    return QTRUE;
                }
            } else {
                return QTRUE;
            }
        }
    }
    QFALSE
}

/// `qboolean NPC_CheckForDanger( int alertEvent )` (NPC_senses.c:568) — one-liner that
/// forwards to [`G_CheckForDanger`] for the current think NPC.
///
/// # Safety
/// `NPC` must be set to the current think entity.
pub unsafe fn NPC_CheckForDanger(alertEvent: c_int) -> qboolean {
    //FIXME: more bStates need to call this?
    G_CheckForDanger(NPC, alertEvent)
}

/*
-------------------------
AddSoundEvent
-------------------------
*/
/// `void AddSoundEvent( gentity_t *owner, vec3_t position, float radius,
/// alertEventLevel_e alertLevel, qboolean needLOS )` (NPC_senses.c:579).
///
/// Append a sound alert event (evicting the oldest via [`RemoveOldestAlert`] when the
/// array is full). Un-owned events below `AEL_DANGER` are dropped; `needLOS` flags a
/// quiet sound that later requires LOS to hear.
///
/// # Safety
/// `level` must be initialised; `owner` may be NULL.
pub unsafe fn AddSoundEvent(
    owner: *mut gentity_t,
    position: &vec3_t,
    radius: f32,
    alertLevel: alertEventLevel_e,
    needLOS: qboolean,
) {
    //FIXME: Handle this in another manner?
    if level.numAlertEvents >= MAX_ALERT_EVENTS as c_int {
        if RemoveOldestAlert() == QFALSE {
            //how could that fail?
            return;
        }
    }

    if owner.is_null() && alertLevel < AEL_DANGER {
        //allows un-owned danger alerts
        return;
    }

    //FIXME: if owner is not a player or player ally, and there are no player allies present,
    //			perhaps we don't need to store the alert... unless we want the player to
    //			react to enemy alert events in some way?

    let idx = level.numAlertEvents as usize;
    VectorCopy(position, &mut level.alertEvents[idx].position);

    level.alertEvents[idx].radius = radius;
    level.alertEvents[idx].level = alertLevel;
    level.alertEvents[idx].r#type = AET_SOUND;
    level.alertEvents[idx].owner = owner;
    if needLOS == QTRUE {
        //a very low-level sound, when check this sound event, check for LOS
        level.alertEvents[idx].addLight = 1.0; //will force an LOS trace on this sound
    } else {
        level.alertEvents[idx].addLight = 0.0; //will force an LOS trace on this sound
    }
    level.alertEvents[idx].ID = level.curAlertID;
    level.curAlertID += 1;
    level.alertEvents[idx].timestamp = level.time;

    level.numAlertEvents += 1;
}

/*
-------------------------
AddSightEvent
-------------------------
*/
/// `void AddSightEvent( gentity_t *owner, vec3_t position, float radius,
/// alertEventLevel_e alertLevel, float addLight )` (NPC_senses.c:623).
///
/// Sight counterpart of [`AddSoundEvent`]; `addLight` is added to the actual light at
/// the point when the event is checked.
///
/// # Safety
/// `level` must be initialised; `owner` may be NULL.
pub unsafe fn AddSightEvent(
    owner: *mut gentity_t,
    position: &vec3_t,
    radius: f32,
    alertLevel: alertEventLevel_e,
    addLight: f32,
) {
    //FIXME: Handle this in another manner?
    if level.numAlertEvents >= MAX_ALERT_EVENTS as c_int {
        if RemoveOldestAlert() == QFALSE {
            //how could that fail?
            return;
        }
    }

    if owner.is_null() && alertLevel < AEL_DANGER {
        //allows un-owned danger alerts
        return;
    }

    //FIXME: if owner is not a player or player ally, and there are no player allies present,
    //			perhaps we don't need to store the alert... unless we want the player to
    //			react to enemy alert events in some way?

    let idx = level.numAlertEvents as usize;
    VectorCopy(position, &mut level.alertEvents[idx].position);

    level.alertEvents[idx].radius = radius;
    level.alertEvents[idx].level = alertLevel;
    level.alertEvents[idx].r#type = AET_SIGHT;
    level.alertEvents[idx].owner = owner;
    level.alertEvents[idx].addLight = addLight; //will get added to actual light at that point when it's checked
    level.alertEvents[idx].ID = level.curAlertID;
    level.curAlertID += 1;
    level.alertEvents[idx].timestamp = level.time;

    level.numAlertEvents += 1;
}

/*
-------------------------
ClearPlayerAlertEvents
-------------------------
*/
/// `void ClearPlayerAlertEvents( void )` (NPC_senses.c:660).
///
/// Sweep the alert-event array, deleting events older than `ALERT_CLEAR_TIME` and
/// compacting the array; bumps the 200 ms `eventClearTime` debouncer.
///
/// # Safety
/// `level` must be initialised.
pub unsafe fn ClearPlayerAlertEvents() {
    let curNumAlerts: c_int = level.numAlertEvents;
    //loop through them all (max 32)
    let mut i: c_int = 0;
    while i < curNumAlerts {
        //see if the event is old enough to delete
        if level.alertEvents[i as usize].timestamp != 0
            && level.alertEvents[i as usize].timestamp + ALERT_CLEAR_TIME < level.time
        {
            //this event has timed out
            //drop the count
            level.numAlertEvents -= 1;
            //shift the rest down
            if level.numAlertEvents > 0 {
                //still have more in the array
                if (i + 1) < MAX_ALERT_EVENTS as c_int {
                    core::ptr::copy(
                        &level.alertEvents[(i + 1) as usize] as *const alertEvent_t,
                        &mut level.alertEvents[i as usize] as *mut alertEvent_t,
                        MAX_ALERT_EVENTS - (i + 1) as usize,
                    );
                }
            } else {
                //just clear this one... or should we clear the whole array?
                core::ptr::write_bytes(&mut level.alertEvents[i as usize] as *mut alertEvent_t, 0, 1);
            }
        }
        i += 1;
    }
    //make sure this never drops below zero... if it does, something very very bad happened
    debug_assert!(level.numAlertEvents >= 0);

    if eventClearTime < level.time {
        //this is just a 200ms debouncer so things that generate constant alerts (like corpses and missiles) add an alert every 200 ms
        eventClearTime = level.time + ALERT_CLEAR_TIME;
    }
}

/// `qboolean RemoveOldestAlert( void )` (NPC_senses.c:695).
///
/// Delete the oldest alert event (by timestamp) and compact the array, returning
/// whether there is now room for a new event.
///
/// # Safety
/// `level` must be initialised.
pub unsafe fn RemoveOldestAlert() -> qboolean {
    let mut oldestEvent: c_int = -1;
    let mut oldestTime: c_int = Q3_INFINITE;
    //loop through them all (max 32)
    let mut i: c_int = 0;
    while i < level.numAlertEvents {
        //see if the event is old enough to delete
        if level.alertEvents[i as usize].timestamp < oldestTime {
            oldestEvent = i;
            oldestTime = level.alertEvents[i as usize].timestamp;
        }
        i += 1;
    }
    if oldestEvent != -1 {
        //drop the count
        level.numAlertEvents -= 1;
        //shift the rest down
        if level.numAlertEvents > 0 {
            //still have more in the array
            if (oldestEvent + 1) < MAX_ALERT_EVENTS as c_int {
                core::ptr::copy(
                    &level.alertEvents[(oldestEvent + 1) as usize] as *const alertEvent_t,
                    &mut level.alertEvents[oldestEvent as usize] as *mut alertEvent_t,
                    MAX_ALERT_EVENTS - (oldestEvent + 1) as usize,
                );
            }
        } else {
            //just clear this one... or should we clear the whole array?
            core::ptr::write_bytes(
                &mut level.alertEvents[oldestEvent as usize] as *mut alertEvent_t,
                0,
                1,
            );
        }
    }
    //make sure this never drops below zero... if it does, something very very bad happened
    debug_assert!(level.numAlertEvents >= 0);
    //return true is have room for one now
    (level.numAlertEvents < MAX_ALERT_EVENTS as c_int) as qboolean
}

/*
-------------------------
G_ClearLOS
-------------------------
*/
// Position to position
/// `qboolean G_ClearLOS( gentity_t *self, const vec3_t start, const vec3_t end )`
/// (NPC_senses.c:739).
///
/// True if `start`..`end` has a clear line of sight, seeing through up to 3 panes of
/// glass (`SVF_GLASS_BRUSH`). `self` is unused (parity).
///
/// # Safety
/// `g_entities` must be initialised; the trapped trace touches engine state.
pub unsafe fn G_ClearLOS(_self_: *mut gentity_t, start: &vec3_t, end: &vec3_t) -> qboolean {
    let mut tr: trace_t;
    let mut traceCount: c_int = 0;

    //FIXME: ENTITYNUM_NONE ok?
    tr = trap::Trace(
        start,
        &vec3_origin,
        &vec3_origin,
        end,
        ENTITYNUM_NONE,
        CONTENTS_OPAQUE, /*CONTENTS_SOLID*/ /*(CONTENTS_SOLID|CONTENTS_MONSTERCLIP)*/
    );
    while tr.fraction < 1.0 && traceCount < 3 {
        //can see through 3 panes of glass
        if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(tr.entityNum as isize);
            if !ent.is_null() && ((*ent).r.svFlags & SVF_GLASS_BRUSH) != 0 {
                //can see through glass, trace again, ignoring me
                tr = trap::Trace(
                    &tr.endpos,
                    &vec3_origin,
                    &vec3_origin,
                    end,
                    tr.entityNum as c_int,
                    MASK_OPAQUE,
                );
                traceCount += 1;
                continue;
            }
        }
        return QFALSE;
    }

    if tr.fraction == 1.0 {
        return QTRUE;
    }

    QFALSE
}

//Entity to position
/// `qboolean G_ClearLOS2( gentity_t *self, gentity_t *ent, const vec3_t end )`
/// (NPC_senses.c:767).
///
/// # Safety
/// `self`/`ent` must point to valid `gentity_t`.
pub unsafe fn G_ClearLOS2(self_: *mut gentity_t, ent: *mut gentity_t, end: &vec3_t) -> qboolean {
    let mut eyes: vec3_t = [0.0; 3];

    CalcEntitySpot(ent, SPOT_HEAD_LEAN, &mut eyes);

    G_ClearLOS(self_, &eyes, end)
}

//Position to entity
/// `qboolean G_ClearLOS3( gentity_t *self, const vec3_t start, gentity_t *ent )`
/// (NPC_senses.c:777).
///
/// # Safety
/// `self`/`ent` must point to valid `gentity_t`.
pub unsafe fn G_ClearLOS3(self_: *mut gentity_t, start: &vec3_t, ent: *mut gentity_t) -> qboolean {
    let mut spot: vec3_t = [0.0; 3];

    //Look for the chest first
    CalcEntitySpot(ent, SPOT_ORIGIN, &mut spot);

    if G_ClearLOS(self_, start, &spot) == QTRUE {
        return QTRUE;
    }

    //Look for the head next
    CalcEntitySpot(ent, SPOT_HEAD_LEAN, &mut spot);

    if G_ClearLOS(self_, start, &spot) == QTRUE {
        return QTRUE;
    }

    QFALSE
}

//NPC's eyes to entity
/// `qboolean G_ClearLOS4( gentity_t *self, gentity_t *ent )` (NPC_senses.c:797).
///
/// # Safety
/// `self`/`ent` must point to valid `gentity_t`.
pub unsafe fn G_ClearLOS4(self_: *mut gentity_t, ent: *mut gentity_t) -> qboolean {
    let mut eyes: vec3_t = [0.0; 3];

    //Calculate my position
    CalcEntitySpot(self_, SPOT_HEAD_LEAN, &mut eyes);

    G_ClearLOS3(self_, &eyes, ent)
}

//NPC's eyes to position
/// `qboolean G_ClearLOS5( gentity_t *self, const vec3_t end )` (NPC_senses.c:808).
///
/// # Safety
/// `self` must point to a valid `gentity_t`.
pub unsafe fn G_ClearLOS5(self_: *mut gentity_t, end: &vec3_t) -> qboolean {
    let mut eyes: vec3_t = [0.0; 3];

    //Calculate the my position
    CalcEntitySpot(self_, SPOT_HEAD_LEAN, &mut eyes);

    G_ClearLOS(self_, &eyes, end)
}

/*
-------------------------
NPC_GetHFOVPercentage
-------------------------
*/

/// `float NPC_GetHFOVPercentage( vec3_t spot, vec3_t from, vec3_t facing, float hFOV )`
/// (NPC_senses.c:824).
///
/// Returns how centered `spot` is within the horizontal FOV `hFOV` (as seen from
/// `from` facing `facing`): `1.0` dead-ahead, falling to `0.0` at the edge, and
/// `0.0` outside.
pub fn NPC_GetHFOVPercentage(spot: &vec3_t, from: &vec3_t, facing: &vec3_t, hFOV: f32) -> f32 {
    let mut deltaVector: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let delta: f32;

    VectorSubtract(spot, from, &mut deltaVector);

    vectoangles(&deltaVector, &mut angles);

    delta = AngleDelta(facing[YAW], angles[YAW]).abs();

    if delta > hFOV {
        return 0.0f32;
    }

    (hFOV - delta) / hFOV
}

/*
-------------------------
NPC_GetVFOVPercentage
-------------------------
*/

/// `float NPC_GetVFOVPercentage( vec3_t spot, vec3_t from, vec3_t facing, float vFOV )`
/// (NPC_senses.c:847).
///
/// Vertical counterpart of [`NPC_GetHFOVPercentage`]: how centered `spot` is within
/// the vertical FOV `vFOV`.
pub fn NPC_GetVFOVPercentage(spot: &vec3_t, from: &vec3_t, facing: &vec3_t, vFOV: f32) -> f32 {
    let mut deltaVector: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let delta: f32;

    VectorSubtract(spot, from, &mut deltaVector);

    vectoangles(&deltaVector, &mut angles);

    delta = AngleDelta(facing[PITCH], angles[PITCH]).abs();

    if delta > vFOV {
        return 0.0f32;
    }

    (vFOV - delta) / vFOV
}

/*
-------------------------
NPC_FindLocalInterestPoint
-------------------------
*/
/// `int G_FindLocalInterestPoint( gentity_t *self )` (NPC_senses.c:871).
///
/// Of the level's interest points in PVS and within `MAX_INTEREST_DIST` with clear LOS
/// (and not too close to look steeply up/down), pick the nearest and fire its `target`.
/// Returns the chosen index or `ENTITYNUM_NONE`.
///
/// # Safety
/// `level` must be initialised; `self` must point to a valid `gentity_t`.
pub unsafe fn G_FindLocalInterestPoint(self_: *mut gentity_t) -> c_int {
    let mut bestPoint: c_int = ENTITYNUM_NONE;
    let mut dist: f32;
    let mut bestDist: f32 = Q3_INFINITE as f32;
    let mut diffVec: vec3_t = [0.0; 3];
    let mut eyes: vec3_t = [0.0; 3];

    CalcEntitySpot(self_, SPOT_HEAD_LEAN, &mut eyes);
    let mut i: c_int = 0;
    while i < level.numInterestPoints {
        //Don't ignore portals?  If through a portal, need to look at portal!
        if trap::InPVS(&level.interestPoints[i as usize].origin, &eyes) == QTRUE {
            VectorSubtract(&level.interestPoints[i as usize].origin, &eyes, &mut diffVec);
            if (diffVec[0].abs() + diffVec[1].abs()) / 2.0 < 48.0
                && diffVec[2].abs() > (diffVec[0].abs() + diffVec[1].abs()) / 2.0
            {
                //Too close to look so far up or down
                i += 1;
                continue;
            }
            dist = VectorLengthSquared(&diffVec);
            //Some priority to more interesting points
            //dist -= ((int)level.interestPoints[i].lookMode * 5) * ((int)level.interestPoints[i].lookMode * 5);
            if dist < MAX_INTEREST_DIST && dist < bestDist {
                if G_ClearLineOfSight(
                    &eyes,
                    &level.interestPoints[i as usize].origin,
                    (*self_).s.number,
                    MASK_OPAQUE,
                ) == QTRUE
                {
                    bestDist = dist;
                    bestPoint = i;
                }
            }
        }
        i += 1;
    }
    if bestPoint != ENTITYNUM_NONE && !level.interestPoints[bestPoint as usize].target.is_null() {
        G_UseTargets2(self_, self_, level.interestPoints[bestPoint as usize].target);
    }
    bestPoint
}

/*QUAKED target_interest (1 0.8 0.5) (-4 -4 -4) (4 4 4)
A point that a squadmate will look at if standing still

target - thing to fire when someone looks at this thing
*/

/// `void SP_target_interest( gentity_t *self )` (NPC_senses.c:915).
///
/// Spawn for `target_interest`: register the entity's origin (and optional `target`) as
/// a level interest point, then free the entity. Refuses past `MAX_INTEREST_POINTS`.
///
/// # Safety
/// `level` must be initialised; `self` must point to a valid `gentity_t`.
pub unsafe fn SP_target_interest(self_: *mut gentity_t) {
    //FIXME: rename point_interest
    if level.numInterestPoints >= MAX_INTEREST_POINTS as c_int {
        Com_Printf(&format!(
            "ERROR:  Too many interest points, limit is {}\n",
            MAX_INTEREST_POINTS
        ));
        G_FreeEntity(self_);
        return;
    }

    let idx = level.numInterestPoints as usize;
    VectorCopy(
        &(*self_).r.currentOrigin,
        &mut level.interestPoints[idx].origin,
    );

    if !(*self_).target.is_null() && *(*self_).target != 0 {
        level.interestPoints[idx].target = G_NewString((*self_).target);
    }

    level.numInterestPoints += 1;

    G_FreeEntity(self_);
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;

    extern "C" {
        fn oracle_InFront(
            spot: *const f32,
            from: *const f32,
            fromAngles: *const f32,
            threshHold: f32,
        ) -> i32;
        fn oracle_InFOV3(
            spot: *const f32,
            from: *const f32,
            fromAngles: *const f32,
            hFOV: i32,
            vFOV: i32,
        ) -> i32;
        fn oracle_NPC_GetHFOVPercentage(
            spot: *const f32,
            from: *const f32,
            facing: *const f32,
            hFOV: f32,
        ) -> f32;
        fn oracle_NPC_GetVFOVPercentage(
            spot: *const f32,
            from: *const f32,
            facing: *const f32,
            vFOV: f32,
        ) -> f32;
    }

    // A spread of position/angle triples exercising in-front, behind, off-axis,
    // edge-of-FOV, and degenerate (zero-length) cases.
    const CASES: &[([f32; 3], [f32; 3], [f32; 3])] = &[
        ([100.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
        ([0.0, 100.0, 0.0], [0.0, 0.0, 0.0], [0.0, 90.0, 0.0]),
        ([-100.0, 0.0, 50.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
        ([50.0, 50.0, 50.0], [10.0, -20.0, 5.0], [15.0, 45.0, 0.0]),
        ([0.0, 0.0, 100.0], [0.0, 0.0, 0.0], [-90.0, 0.0, 0.0]),
        ([-30.0, 70.0, -20.0], [5.0, 5.0, 5.0], [30.0, 200.0, 10.0]),
        ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
        ([123.5, -456.25, 78.0], [-12.0, 34.0, -56.0], [12.5, -170.0, 3.0]),
    ];

    #[test]
    fn infront_matches_oracle() {
        for thresh in [-0.5f32, 0.0, 0.3, 0.7, 1.0] {
            for (spot, from, ang) in CASES {
                let rust = InFront(spot, from, ang, thresh);
                let c =
                    unsafe { oracle_InFront(spot.as_ptr(), from.as_ptr(), ang.as_ptr(), thresh) };
                assert_eq!(rust, c, "InFront {spot:?} {from:?} {ang:?} thresh={thresh}");
            }
        }
    }

    #[test]
    fn infov3_matches_oracle() {
        for (h, v) in [(0, 0), (30, 20), (90, 45), (180, 90), (45, 10)] {
            for (spot, from, ang) in CASES {
                let rust = InFOV3(spot, from, ang, h, v);
                let c = unsafe { oracle_InFOV3(spot.as_ptr(), from.as_ptr(), ang.as_ptr(), h, v) };
                assert_eq!(rust, c, "InFOV3 {spot:?} {from:?} {ang:?} h={h} v={v}");
            }
        }
    }

    #[test]
    fn hfov_percentage_matches_oracle() {
        for hfov in [1.0f32, 30.0, 45.0, 90.0, 180.0] {
            for (spot, from, facing) in CASES {
                let rust = NPC_GetHFOVPercentage(spot, from, facing, hfov);
                let c = unsafe {
                    oracle_NPC_GetHFOVPercentage(spot.as_ptr(), from.as_ptr(), facing.as_ptr(), hfov)
                };
                assert_eq!(rust.to_bits(), c.to_bits(), "HFOV {spot:?} {facing:?} hfov={hfov}");
            }
        }
    }

    #[test]
    fn vfov_percentage_matches_oracle() {
        for vfov in [1.0f32, 30.0, 45.0, 90.0, 180.0] {
            for (spot, from, facing) in CASES {
                let rust = NPC_GetVFOVPercentage(spot, from, facing, vfov);
                let c = unsafe {
                    oracle_NPC_GetVFOVPercentage(spot.as_ptr(), from.as_ptr(), facing.as_ptr(), vfov)
                };
                assert_eq!(rust.to_bits(), c.to_bits(), "VFOV {spot:?} {facing:?} vfov={vfov}");
            }
        }
    }
}
