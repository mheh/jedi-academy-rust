//! Port of `g_mover.c` — movers (doors, plats, buttons, trains), breakables and the
//! brush-pushing physics core. Landed incrementally: only the functions whose full
//! dep-set is already ported. This module carries the three pure rotation-matrix helpers,
//! the **entity-pushing core** — `G_TestEntityPosition`, `G_TryPushingEntity`, `G_MoverPush`,
//! `G_MoverTeam` and `G_RunMover` (the per-frame mover entry point) — driven by the file-scope
//! `pushed`/`pushed_p` back-out stack, and the **binary-mover state machine** (the door/plat/
//! button two-position core): the door-sound pair, `CalcTeamDoorCenter`, `SetMoverState`,
//! `MatchTeam`/`Think_MatchTeam`, `ReturnToPos1`, `Reached_BinaryMover`, `Use_BinaryMover[_Go]`
//! and `LockDoors`/`UnLockDoors`, plus the spawn-time `InitMover`/`InitMoverTrData` setup. On top
//! of that: `Blocked_Door`, the door-state predicates (`G_EntIsDoor`/`G_FindDoorTrigger`/
//! `G_EntIsUnlockedDoor`), the plat/button touch + trigger-spawn helpers (`Touch_Plat`/
//! `Touch_PlatCenterTrigger`/`SpawnPlatTrigger`/`Touch_Button`), the train `Think_BeginMoving` and
//! the `SP_func_*` spawners including `SP_func_door`. The auto door-trigger chain
//! (`Touch_DoorTriggerSpectator`/`Touch_DoorTrigger`/`Think_SpawnNewDoorTrigger`) is ported; its
//! spectator nudge drops JKA's `CM_PointLeafnum`/`CM_LeafCluster` "out of the world" guard (engine
//! collision-model internals outside the `trap_*` ABI) for OpenJK parity — see `DEVIATIONS.md`.

#![allow(non_snake_case)] // C function names (`G_CreateRotationMatrix`, …) kept verbatim
#![allow(non_camel_case_types)] // C type name (`pushed_t`) kept verbatim
#![allow(non_upper_case_globals)] // C global names (`pushed`, `pushed_p`) kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::bg_misc::BG_EvaluateTrajectory;
use crate::codemp::game::bg_public::{
    DEFAULT_MAXS_2, DEFAULT_MINS_2, ET_BODY, ET_MOVER, ET_NPC, ET_PLAYER, EF2_HYPERSPACE,
    EF_MISSILE_STICK, EF_NODRAW, EF_PERMANENT, EF_RADAROBJECT, EF_SHADER_ANIM, EV_BMODEL_SOUND,
    EV_DEBRIS, EV_GENERAL_SOUND, EV_GLASS_SHATTER, EV_MISC_MODEL_EXP, EV_PLAYDOORSOUND, GT_SIEGE,
    MASK_SOLID, MOD_CRUSH, MOD_UNKNOWN, STAT_HEALTH, TEAM_SPECTATOR,
};
use crate::codemp::game::g_combat::{G_Damage, G_RadiusDamage};
use crate::codemp::game::g_misc::TeleportPlayer;
use crate::codemp::game::g_local::{
    gentity_t, moverState_t, DAMAGE_NO_KNOCKBACK, FL_BBRUSH, FL_DMG_BY_HEAVY_WEAP_ONLY,
    FL_DMG_BY_SABER_ONLY, FL_INACTIVE, FL_TEAMSLAVE, FRAMETIME, MOVER_1TO2, MOVER_2TO1, MOVER_POS1,
    MOVER_POS2,
};
use crate::codemp::game::g_main::{
    g_entities, g_gametype, g_gravity, level, Com_Printf, G_Error, G_Printf, G_RunThink,
};
use crate::codemp::game::g_public_h::{
    BSET_PAIN, BSET_USE, SVF_BROADCAST, SVF_GLASS_BRUSH, SVF_NOCLIENT, SVF_PLAYER_USABLE,
    SVF_USE_CURRENT_ORIGIN,
};
use crate::codemp::game::g_spawn::{G_SpawnFloat, G_SpawnInt, G_SpawnString, G_SpawnVector};
use crate::codemp::game::g_utils::{
    G_AddEvent, G_EffectIndex, G_Find, G_FreeEntity, G_ModelIndex, G_PlayEffectID, G_ScaleNetHealth,
    G_SetAngles, G_SetMovedir, G_SetOrigin, G_SoundIndex, G_SoundSetIndex, G_Spawn, G_TempEntity,
    G_UseTargets, G_UseTargets2, GlobalUse, vtos,
};
use crate::codemp::game::npc_utils::G_ActivateBehavior;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AddPointToBounds, AngleVectors, Distance, DotProduct,
    RadiusFromBounds, VectorAdd, VectorClear, VectorCompare, VectorCopy, VectorInverse,
    VectorLength, VectorMA, VectorNormalize, VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{random, Q_stricmp};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{
    material_t, trace_t, vec3_t, ANGLE2SHORT, ENTITYNUM_NONE, ENTITYNUM_WORLD, MAT_CRATE1,
    MAT_CRATE2, MAT_DRK_STONE, MAT_ELECTRICAL, MAT_ELEC_METAL, MAT_GLASS, MAT_GLASS_METAL,
    MAT_GRATE1, MAT_GREY_STONE, MAT_LT_STONE, MAT_METAL, MAT_METAL2, MAT_METAL3, MAT_ROPE,
    MAT_SNOWY_ROCK, MAT_WHITE_METAL, MAX_CLIENTS, MAX_GENTITIES, M_PI, RAD2DEG, TR_LINEAR,
    TR_LINEAR_STOP, TR_NONLINEAR_STOP, TR_SINE, TR_STATIONARY, YAW,
};
use crate::codemp::game::surfaceflags_h::{CONTENTS_BODY, CONTENTS_SOLID, CONTENTS_TRIGGER};
use crate::codemp::game::teams_h::CLASS_VEHICLE;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

extern "C" {
    /// `int atoi( const char *string )` — the retail (non-`Q3_VM`) build links the C library's
    /// `atoi`; the `Q3_VM` shim in [`crate::codemp::game::bg_lib`] is gated behind the `vm` feature.
    fn atoi(s: *const c_char) -> c_int;
}

/// `pushed_t` (g_mover.c:18) — one saved entity position on the mover-push back-out stack: the
/// entity plus its pre-push origin/angles, and (for clients) the saved delta-yaw.
#[repr(C)]
#[derive(Clone, Copy)]
struct pushed_t {
    ent: *mut gentity_t,
    origin: vec3_t,
    angles: vec3_t,
    deltayaw: f32,
}

/// `pushed_t pushed[MAX_GENTITIES], *pushed_p;` (g_mover.c:24) — the file-scope stack of saved
/// positions for the entities being shoved by the mover team currently moving, and the moving
/// stack top. [`G_MoverTeam`] resets `pushed_p` to the base before each team move,
/// [`G_TryPushingEntity`] pushes a saved position, and the back-out loop in [`G_MoverPush`]
/// pops them to undo a blocked move. Reached only through `addr_of_mut!` — never a `&`/`&mut`
/// to a `static mut`.
static mut pushed: [pushed_t; MAX_GENTITIES] = [pushed_t {
    ent: null_mut(),
    origin: [0.0; 3],
    angles: [0.0; 3],
    deltayaw: 0.0,
}; MAX_GENTITIES];
static mut pushed_p: *mut pushed_t = null_mut();

/// `int BMS_START = 0; int BMS_MID = 1; int BMS_END = 2;` (g_mover.c:35) — the three door
/// sound-set slots (start / mid-loop / end). In C these are `extern int` globals (g_local.h:1203)
/// initialised once and never reassigned; ported as constants since nothing mutates them.
pub const BMS_START: c_int = 0;
pub const BMS_MID: c_int = 1;
pub const BMS_END: c_int = 2;

/// Mover spawnflag bits (`#define`s at g_mover.c:25). Only the subset the binary-mover state
/// machine, `InitMover`, and the touch/door-state helpers use is defined here; the rest
/// (`MOVER_START_ON`, `MOVER_GOODIE`) land with the `SP_func_*` spawners that read them.
const MOVER_FORCE_ACTIVATE: c_int = 2;
const MOVER_CRUSHER: c_int = 4;
const MOVER_TOGGLE: c_int = 8;
const MOVER_LOCKED: c_int = 16;
const MOVER_PLAYER_USE: c_int = 64;
const MOVER_INACTIVE: c_int = 128;

/// `void G_PlayDoorLoopSound( gentity_t *ent )` (g_mover.c:45). If the mover has a sound set,
/// latch its index and start the looping mid-motion sound (`BMS_MID`) as a soundset loop. No-op
/// when the entity has no `soundSet`. No oracle (entity-state mutation + `G_SoundSetIndex`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe fn G_PlayDoorLoopSound(ent: *mut gentity_t) {
    if (*ent).soundSet.is_null() || *(*ent).soundSet == 0 {
        return;
    }

    (*ent).s.soundSetIndex = G_SoundSetIndex(&CStr::from_ptr((*ent).soundSet).to_string_lossy());
    (*ent).s.loopIsSoundset = QTRUE;
    (*ent).s.loopSound = BMS_MID;
}

/// `void G_PlayDoorSound( gentity_t *ent, int type )` (g_mover.c:68). If the mover has a sound
/// set, latch its index and fire an `EV_PLAYDOORSOUND` event with `type` (`BMS_START`/`BMS_END`)
/// so the client plays the one-shot. No-op when the entity has no `soundSet`. No oracle
/// (entity-state mutation + `G_AddEvent`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe fn G_PlayDoorSound(ent: *mut gentity_t, type_: c_int) {
    if (*ent).soundSet.is_null() || *(*ent).soundSet == 0 {
        return;
    }

    (*ent).s.soundSetIndex = G_SoundSetIndex(&CStr::from_ptr((*ent).soundSet).to_string_lossy());

    G_AddEvent(ent, EV_PLAYDOORSOUND, type_);
}

/// `gentity_t *G_TestEntityPosition( gentity_t *ent )` (g_mover.c:86). Trace `ent`'s bounding
/// box against the world at its current spot; return the solid entity it is stuck inside (via
/// the trace's `entityNum`) or null if the spot is clear. Clients trace from `ps.origin` with a
/// floor-clamped `maxs[2]`; everything else traces from `s.pos.trBase`. No oracle (engine
/// `trap_Trace` drives the result).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` and `g_entities` must be initialised.
pub unsafe fn G_TestEntityPosition(ent: *mut gentity_t) -> *mut gentity_t {
    let mask: c_int = if (*ent).clipmask != 0 {
        (*ent).clipmask
    } else {
        MASK_SOLID
    };

    let tr = if !(*ent).client.is_null() {
        let mut vMax: vec3_t = [0.0; 3];
        VectorCopy(&(*ent).r.maxs, &mut vMax);
        if vMax[2] < 1.0 {
            vMax[2] = 1.0;
        }
        trap::Trace(
            &(*(*ent).client).ps.origin,
            &(*ent).r.mins,
            &vMax,
            &(*(*ent).client).ps.origin,
            (*ent).s.number,
            mask,
        )
    } else {
        trap::Trace(
            &(*ent).s.pos.trBase,
            &(*ent).r.mins,
            &(*ent).r.maxs,
            &(*ent).s.pos.trBase,
            (*ent).s.number,
            mask,
        )
    };

    if tr.startsolid != 0 {
        return (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);
    }

    null_mut()
}

/// `void G_CreateRotationMatrix(vec3_t angles, vec3_t matrix[3])` (g_mover.c:118). Builds
/// the rotation basis for `angles` via [`AngleVectors`], then inverts the right vector
/// (row 1) — matching the C convention used by [`G_TryPushingEntity`] to rotate pushed
/// entities into/out of the mover's frame. Oracle-tested bit-exact via
/// [`crate::oracle::jka_G_CreateRotationMatrix`].
pub fn G_CreateRotationMatrix(angles: &vec3_t, matrix: &mut [vec3_t; 3]) {
    {
        let [m0, m1, m2] = &mut *matrix;
        AngleVectors(angles, Some(m0), Some(m1), Some(m2));
    }
    VectorInverse(&mut matrix[1]);
}

/// `void G_TransposeMatrix(vec3_t matrix[3], vec3_t transpose[3])` (g_mover.c:128).
/// Writes the transpose of `matrix` into `transpose`. Oracle-tested bit-exact via
/// [`crate::oracle::jka_G_TransposeMatrix`].
pub fn G_TransposeMatrix(matrix: &[vec3_t; 3], transpose: &mut [vec3_t; 3]) {
    for i in 0..3 {
        for j in 0..3 {
            transpose[i][j] = matrix[j][i];
        }
    }
}

/// `void G_RotatePoint(vec3_t point, vec3_t matrix[3])` (g_mover.c:142). Rotates `point`
/// in place by `matrix` (each output component is the dot of a basis row with the
/// original point). Oracle-tested bit-exact via [`crate::oracle::jka_G_RotatePoint`].
pub fn G_RotatePoint(point: &mut vec3_t, matrix: &[vec3_t; 3]) {
    let mut tvec: vec3_t = [0.0; 3];

    VectorCopy(point, &mut tvec);
    point[0] = DotProduct(&matrix[0], &tvec);
    point[1] = DotProduct(&matrix[1], &tvec);
    point[2] = DotProduct(&matrix[2], &tvec);
}

/// `qboolean G_TryPushingEntity( gentity_t *check, gentity_t *pusher, vec3_t move, vec3_t amove )`
/// (g_mover.c:158). Try to shove `check` by the pusher's linear `move` and angular `amove`,
/// rotating a rider into/out of the mover's frame via the matrix helpers. Saves the old
/// position on the `pushed` stack first; returns `qfalse` if the move is blocked and `check`
/// cannot even be left where it was. No oracle (entity array + `trap_Trace`/`trap_LinkEntity`
/// + `G_Damage` control flow).
///
/// # Safety
/// `check`/`pusher` must point to valid `gentity_t`s and the `pushed`/`pushed_p` stack must be
/// active (set up by [`G_MoverTeam`]).
pub unsafe fn G_TryPushingEntity(
    check: *mut gentity_t,
    pusher: *mut gentity_t,
    move_: &vec3_t,
    amove: &vec3_t,
) -> qboolean {
    let mut matrix: [vec3_t; 3] = [[0.0; 3]; 3];
    let mut transpose: [vec3_t; 3] = [[0.0; 3]; 3];
    let mut org: vec3_t = [0.0; 3];
    let mut org2: vec3_t = [0.0; 3];
    let mut move2: vec3_t = [0.0; 3];

    // The EF_MOVER_STOP early-out was serverside-only and never set — omitted (see C comment).
    if (*pusher).s.apos.trType != TR_STATIONARY // rotating
        && ((*pusher).spawnflags & 16) != 0 // IMPACT
        && Q_stricmp(c"func_rotating".as_ptr(), (*pusher).classname) == 0
    {
        // just blow the fuck out of them
        G_Damage(
            check,
            pusher,
            pusher,
            null_mut(),
            null_mut(),
            (*pusher).damage,
            DAMAGE_NO_KNOCKBACK,
            MOD_CRUSH,
        );
        return QTRUE;
    }

    // save off the old position
    if pushed_p > (addr_of_mut!(pushed) as *mut pushed_t).add(MAX_GENTITIES) {
        G_Error("pushed_p > &pushed[MAX_GENTITIES]");
    }
    (*pushed_p).ent = check;
    VectorCopy(&(*check).s.pos.trBase, &mut (*pushed_p).origin);
    VectorCopy(&(*check).s.apos.trBase, &mut (*pushed_p).angles);
    if !(*check).client.is_null() {
        (*pushed_p).deltayaw = (*(*check).client).ps.delta_angles[YAW] as f32;
        VectorCopy(&(*(*check).client).ps.origin, &mut (*pushed_p).origin);
    }
    pushed_p = pushed_p.add(1);

    // try moving the contacted entity
    // figure movement due to the pusher's amove
    G_CreateRotationMatrix(amove, &mut transpose);
    G_TransposeMatrix(&transpose, &mut matrix);
    if !(*check).client.is_null() {
        VectorSubtract(&(*(*check).client).ps.origin, &(*pusher).r.currentOrigin, &mut org);
    } else {
        VectorSubtract(&(*check).s.pos.trBase, &(*pusher).r.currentOrigin, &mut org);
    }
    VectorCopy(&org, &mut org2);
    G_RotatePoint(&mut org2, &matrix);
    VectorSubtract(&org2, &org, &mut move2);
    // add movement
    let base = (*check).s.pos.trBase;
    VectorAdd(&base, move_, &mut (*check).s.pos.trBase);
    let base = (*check).s.pos.trBase;
    VectorAdd(&base, &move2, &mut (*check).s.pos.trBase);
    if !(*check).client.is_null() {
        let corg = (*(*check).client).ps.origin;
        VectorAdd(&corg, move_, &mut (*(*check).client).ps.origin);
        let corg = (*(*check).client).ps.origin;
        VectorAdd(&corg, &move2, &mut (*(*check).client).ps.origin);
        // make sure the client's view rotates when on a rotating mover
        (*(*check).client).ps.delta_angles[YAW] += ANGLE2SHORT(amove[YAW]);
    }

    // may have pushed them off an edge
    if (*check).s.groundEntityNum != (*pusher).s.number {
        (*check).s.groundEntityNum = ENTITYNUM_NONE; //-1;
    }

    let mut block = G_TestEntityPosition(check);
    if block.is_null() {
        // pushed ok
        if !(*check).client.is_null() {
            VectorCopy(&(*(*check).client).ps.origin, &mut (*check).r.currentOrigin);
        } else {
            VectorCopy(&(*check).s.pos.trBase, &mut (*check).r.currentOrigin);
        }
        trap::LinkEntity(check);
        return QTRUE;
    }

    if (*check).takedamage != QFALSE
        && (*check).client.is_null()
        && (*check).s.weapon != 0
        && (*check).r.ownerNum < MAX_CLIENTS as c_int
        && (*check).health < 500
    {
        if (*check).health > 0 {
            G_Damage(
                check,
                pusher,
                pusher,
                &vec3_origin as *const vec3_t as *mut vec3_t,
                addr_of_mut!((*check).r.currentOrigin),
                999,
                0,
                MOD_UNKNOWN,
            );
        }
        return QFALSE;
    }
    // if it is ok to leave in the old position, do it
    // this is only relevent for riding entities, not pushed
    // Sliding trapdoors can cause this.
    VectorCopy(&(*pushed_p.sub(1)).origin, &mut (*check).s.pos.trBase);
    if !(*check).client.is_null() {
        VectorCopy(&(*pushed_p.sub(1)).origin, &mut (*(*check).client).ps.origin);
    }
    VectorCopy(&(*pushed_p.sub(1)).angles, &mut (*check).s.apos.trBase);
    block = G_TestEntityPosition(check);
    if block.is_null() {
        (*check).s.groundEntityNum = -1;
        pushed_p = pushed_p.sub(1);
        return QTRUE;
    }

    // blocked
    QFALSE
}

/// `qboolean G_MoverPush( gentity_t *pusher, vec3_t move, vec3_t amove, gentity_t **obstacle )`
/// (g_mover.c:273). Move `pusher` to its destination and shove every solid item/player/NPC
/// caught in the swept bounds, crushing or backing out as needed. Returns `qfalse` with
/// `*obstacle` set to the blocking entity if a push could not be resolved; objects already
/// moved are rolled back so riders don't keep sliding. No oracle (entity-box query + per-entity
/// `G_TryPushingEntity`/`G_Damage` control flow).
///
/// # Safety
/// `pusher` must point to a valid `gentity_t`, `obstacle` to a writable `*mut gentity_t`, and
/// `g_entities` must be initialised.
pub unsafe fn G_MoverPush(
    pusher: *mut gentity_t,
    move_: &vec3_t,
    amove: &vec3_t,
    obstacle: *mut *mut gentity_t,
) -> qboolean {
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut entityList: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut totalMins: vec3_t = [0.0; 3];
    let mut totalMaxs: vec3_t = [0.0; 3];

    *obstacle = null_mut();

    // mins/maxs are the bounds at the destination
    // totalMins / totalMaxs are the bounds for the entire move
    if (*pusher).r.currentAngles[0] != 0.0
        || (*pusher).r.currentAngles[1] != 0.0
        || (*pusher).r.currentAngles[2] != 0.0
        || amove[0] != 0.0
        || amove[1] != 0.0
        || amove[2] != 0.0
    {
        let radius = RadiusFromBounds(&(*pusher).r.mins, &(*pusher).r.maxs);
        for i in 0..3 {
            mins[i] = (*pusher).r.currentOrigin[i] + move_[i] - radius;
            maxs[i] = (*pusher).r.currentOrigin[i] + move_[i] + radius;
            totalMins[i] = mins[i] - move_[i];
            totalMaxs[i] = maxs[i] - move_[i];
        }
    } else {
        for i in 0..3 {
            mins[i] = (*pusher).r.absmin[i] + move_[i];
            maxs[i] = (*pusher).r.absmax[i] + move_[i];
        }

        VectorCopy(&(*pusher).r.absmin, &mut totalMins);
        VectorCopy(&(*pusher).r.absmax, &mut totalMaxs);
        for i in 0..3 {
            if move_[i] > 0.0 {
                totalMaxs[i] += move_[i];
            } else {
                totalMins[i] += move_[i];
            }
        }
    }

    // unlink the pusher so we don't get it in the entityList
    trap::UnlinkEntity(pusher);

    let listedEntities = trap::EntitiesInBox(&totalMins, &totalMaxs, &mut entityList);

    // move the pusher to it's final position
    let porg = (*pusher).r.currentOrigin;
    VectorAdd(&porg, move_, &mut (*pusher).r.currentOrigin);
    let pang = (*pusher).r.currentAngles;
    VectorAdd(&pang, amove, &mut (*pusher).r.currentAngles);
    trap::LinkEntity(pusher);

    // see if any solid entities are inside the final position
    for e in 0..listedEntities {
        let check = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityList[e as usize] as usize);

        // only push items and players
        if /*check->s.eType != ET_ITEM &&*/ (*check).s.eType != ET_PLAYER
            && (*check).s.eType != ET_NPC
            && (*check).physicsObject == QFALSE
        {
            continue;
        }

        // if the entity is standing on the pusher, it will definitely be moved
        if (*check).s.groundEntityNum != (*pusher).s.number {
            // see if the ent needs to be tested
            if (*check).r.absmin[0] >= maxs[0]
                || (*check).r.absmin[1] >= maxs[1]
                || (*check).r.absmin[2] >= maxs[2]
                || (*check).r.absmax[0] <= mins[0]
                || (*check).r.absmax[1] <= mins[1]
                || (*check).r.absmax[2] <= mins[2]
            {
                continue;
            }
            // see if the ent's bbox is inside the pusher's final position
            // this does allow a fast moving object to pass through a thin entity...
            if G_TestEntityPosition(check).is_null() {
                continue;
            }
        }

        // the entity needs to be pushed
        if G_TryPushingEntity(check, pusher, move_, amove) != QFALSE {
            continue;
        }

        if (*pusher).damage != 0 && !(*check).client.is_null() && ((*pusher).spawnflags & 32) != 0 {
            G_Damage(check, pusher, pusher, null_mut(), null_mut(), (*pusher).damage, 0, MOD_CRUSH);
            continue;
        }

        if (*check).s.eType == ET_BODY
            || ((*check).s.eType == ET_PLAYER && (*check).health < 1)
        {
            // whatever, just crush it
            G_Damage(check, pusher, pusher, null_mut(), null_mut(), 999, 0, MOD_CRUSH);
            continue;
        }

        // the move was blocked an entity

        // bobbing entities are instant-kill and never get blocked
        if (*pusher).s.pos.trType == TR_SINE || (*pusher).s.apos.trType == TR_SINE {
            G_Damage(check, pusher, pusher, null_mut(), null_mut(), 99999, 0, MOD_CRUSH);
            continue;
        }

        // save off the obstacle so we can call the block function (crush, etc)
        *obstacle = check;

        // move back any entities we already moved
        // go backwards, so if the same entity was pushed
        // twice, it goes back to the original position
        let base = addr_of_mut!(pushed) as *mut pushed_t;
        let mut p = pushed_p.wrapping_sub(1);
        while p >= base {
            VectorCopy(&(*p).origin, &mut (*(*p).ent).s.pos.trBase);
            VectorCopy(&(*p).angles, &mut (*(*p).ent).s.apos.trBase);
            if !(*(*p).ent).client.is_null() {
                (*(*(*p).ent).client).ps.delta_angles[YAW] = (*p).deltayaw as c_int;
                VectorCopy(&(*p).origin, &mut (*(*(*p).ent).client).ps.origin);
            }
            trap::LinkEntity((*p).ent);
            p = p.wrapping_sub(1);
        }
        return QFALSE;
    }

    QTRUE
}

/// `void G_MoverTeam( gentity_t *ent )` (g_mover.c:406). Move a whole mover team atomically:
/// evaluate each part's trajectory, push everything via [`G_MoverPush`], and if any part is
/// blocked roll the entire team back and call the captain's `blocked` callback; otherwise fire
/// each part's `reached` callback when it arrives. No oracle (team-chain walk + trajectory eval
/// + `G_MoverPush` control flow).
///
/// # Safety
/// `ent` must point to a valid mover-team captain `gentity_t`; `level` must be initialised.
pub unsafe fn G_MoverTeam(ent: *mut gentity_t) {
    let mut move_: vec3_t = [0.0; 3];
    let mut amove: vec3_t = [0.0; 3];
    let mut origin: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut obstacle: *mut gentity_t = null_mut();

    // make sure all team slaves can move before commiting
    // any moves or calling any think functions
    // if the move is blocked, all moved objects will be backed out
    pushed_p = addr_of_mut!(pushed) as *mut pushed_t;
    let mut part = ent;
    while !part.is_null() {
        // get current position
        BG_EvaluateTrajectory(&(*part).s.pos, (*addr_of!(level)).time, &mut origin);
        BG_EvaluateTrajectory(&(*part).s.apos, (*addr_of!(level)).time, &mut angles);
        VectorSubtract(&origin, &(*part).r.currentOrigin, &mut move_);
        VectorSubtract(&angles, &(*part).r.currentAngles, &mut amove);
        if VectorCompare(&move_, &vec3_origin) == 0 || VectorCompare(&amove, &vec3_origin) == 0 {
            // actually moved
            if G_MoverPush(part, &move_, &amove, &mut obstacle) == QFALSE {
                break; // move was blocked
            }
        }
        part = (*part).teamchain;
    }

    if !part.is_null() {
        // go back to the previous position
        part = ent;
        while !part.is_null() {
            (*part).s.pos.trTime += (*addr_of!(level)).time - (*addr_of!(level)).previousTime;
            (*part).s.apos.trTime += (*addr_of!(level)).time - (*addr_of!(level)).previousTime;
            BG_EvaluateTrajectory(&(*part).s.pos, (*addr_of!(level)).time, &mut (*part).r.currentOrigin);
            BG_EvaluateTrajectory(&(*part).s.apos, (*addr_of!(level)).time, &mut (*part).r.currentAngles);
            trap::LinkEntity(part);
            part = (*part).teamchain;
        }

        // if the pusher has a "blocked" function, call it
        if let Some(blocked) = (*ent).blocked {
            blocked(ent, obstacle);
        }
        return;
    }

    // the move succeeded
    part = ent;
    while !part.is_null() {
        // call the reached function if time is at or past end point
        if (*part).s.pos.trType == TR_LINEAR_STOP || (*part).s.pos.trType == TR_NONLINEAR_STOP {
            if (*addr_of!(level)).time >= (*part).s.pos.trTime + (*part).s.pos.trDuration {
                if let Some(reached) = (*part).reached {
                    reached(part);
                }
            }
        }
        part = (*part).teamchain;
    }
}

/// `void G_RunMover( gentity_t *ent )` (g_mover.c:469). Per-frame entry point for a mover: team
/// slaves are skipped (the captain drives them); an active captain advances its team via
/// [`G_MoverTeam`]; then its think runs via [`G_RunThink`]. No oracle (entity-state dispatch).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe fn G_RunMover(ent: *mut gentity_t) {
    // if not a team captain, don't do anything, because
    // the captain will handle everything
    if (*ent).flags & FL_TEAMSLAVE != 0 {
        return;
    }

    // if stationary at one of the positions, don't move anything
    if (*ent).s.pos.trType != TR_STATIONARY || (*ent).s.apos.trType != TR_STATIONARY {
        G_MoverTeam(ent);
    }

    // check think function
    G_RunThink(ent);
}

// ===========================================================================
// GENERAL MOVERS
//
// Doors, plats, and buttons are all binary (two position) movers.
// Pos1 is "at rest", pos2 is "activated".
// ===========================================================================

/// `void CalcTeamDoorCenter( gentity_t *ent, vec3_t center )` (g_mover.c:501). Walk the mover
/// team and return the running midpoint of every part's bbox center in `center`. No oracle
/// (team-chain walk over entity bounds).
///
/// # Safety
/// `ent` must point to a valid mover `gentity_t`; its `teamchain` must be a valid list.
pub unsafe fn CalcTeamDoorCenter(ent: *mut gentity_t, center: &mut vec3_t) {
    let mut slavecenter: vec3_t = [0.0; 3];

    // Start with our center
    VectorAdd(&(*ent).r.mins, &(*ent).r.maxs, center);
    let c = *center;
    VectorScale(&c, 0.5, center);
    let mut slave = (*ent).teamchain;
    while !slave.is_null() {
        // Find slave's center
        VectorAdd(&(*slave).r.mins, &(*slave).r.maxs, &mut slavecenter);
        let sc = slavecenter;
        VectorScale(&sc, 0.5, &mut slavecenter);
        // Add that to our own, find middle
        let c = *center;
        VectorAdd(&c, &slavecenter, center);
        let c = *center;
        VectorScale(&c, 0.5, center);
        slave = (*slave).teamchain;
    }
}

/// `void SetMoverState( gentity_t *ent, moverState_t moverState, int time )` (g_mover.c:525).
/// Latch the mover's state and rebuild its position trajectory: `POS1`/`POS2` are stationary at
/// the endpoints, `1TO2`/`2TO1` move linearly (or non-linearly for non-`alt_fire` movers) over
/// `trDuration`. Re-evaluates the trajectory at `level.time` and relinks. No oracle
/// (entity-state mutation + `BG_EvaluateTrajectory`/`trap_LinkEntity`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `level` must be initialised.
pub unsafe fn SetMoverState(ent: *mut gentity_t, moverState: moverState_t, time: c_int) {
    let mut delta: vec3_t = [0.0; 3];

    (*ent).moverState = moverState;

    (*ent).s.pos.trTime = time;

    if (*ent).s.pos.trDuration <= 0 {
        // Don't allow divide by zero!
        (*ent).s.pos.trDuration = 1;
    }

    match moverState {
        MOVER_POS1 => {
            VectorCopy(&(*ent).pos1, &mut (*ent).s.pos.trBase);
            (*ent).s.pos.trType = TR_STATIONARY;
        }
        MOVER_POS2 => {
            VectorCopy(&(*ent).pos2, &mut (*ent).s.pos.trBase);
            (*ent).s.pos.trType = TR_STATIONARY;
        }
        MOVER_1TO2 => {
            VectorCopy(&(*ent).pos1, &mut (*ent).s.pos.trBase);
            VectorSubtract(&(*ent).pos2, &(*ent).pos1, &mut delta);
            let f = 1000.0 / (*ent).s.pos.trDuration as f32;
            VectorScale(&delta, f, &mut (*ent).s.pos.trDelta);
            if (*ent).alt_fire != QFALSE {
                (*ent).s.pos.trType = TR_LINEAR_STOP;
            } else {
                (*ent).s.pos.trType = TR_NONLINEAR_STOP;
            }
        }
        MOVER_2TO1 => {
            VectorCopy(&(*ent).pos2, &mut (*ent).s.pos.trBase);
            VectorSubtract(&(*ent).pos1, &(*ent).pos2, &mut delta);
            let f = 1000.0 / (*ent).s.pos.trDuration as f32;
            VectorScale(&delta, f, &mut (*ent).s.pos.trDelta);
            if (*ent).alt_fire != QFALSE {
                (*ent).s.pos.trType = TR_LINEAR_STOP;
            } else {
                (*ent).s.pos.trType = TR_NONLINEAR_STOP;
            }
        }
        _ => {}
    }
    BG_EvaluateTrajectory(&(*ent).s.pos, (*addr_of!(level)).time, &mut (*ent).r.currentOrigin);
    trap::LinkEntity(ent);
}

/// `void MatchTeam( gentity_t *teamLeader, int moverState, int time )` (g_mover.c:590). Drive
/// every part of a mover team to the same `moverState` at the same `time` so they move in lock
/// step. No oracle (team-chain walk over [`SetMoverState`]).
///
/// # Safety
/// `teamLeader` must point to a valid mover-team captain `gentity_t`.
pub unsafe fn MatchTeam(teamLeader: *mut gentity_t, moverState: c_int, time: c_int) {
    let mut slave = teamLeader;
    while !slave.is_null() {
        SetMoverState(slave, moverState as moverState_t, time);
        slave = (*slave).teamchain;
    }
}

/// `void ReturnToPos1( gentity_t *ent )` (g_mover.c:604). Think function that sends an opened
/// mover back toward pos1: clear the think, drive the team to `MOVER_2TO1`, and (re)start the
/// door loop + start sounds. No oracle (entity-state mutation + sound helpers).
///
/// # Safety
/// `ent` must point to a valid mover `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn ReturnToPos1(ent: *mut gentity_t) {
    (*ent).think = None;
    (*ent).nextthink = 0;
    (*ent).s.time = (*addr_of!(level)).time;

    MatchTeam(ent, MOVER_2TO1, (*addr_of!(level)).time);

    // starting sound
    G_PlayDoorLoopSound(ent);
    G_PlayDoorSound(ent, BMS_START); //??
}

/// `void Reached_BinaryMover( gentity_t *ent )` (g_mover.c:621). `reached` callback fired when a
/// binary mover hits an endpoint: stop the loop sound, latch the resting state, play the end
/// sound, and either schedule the return-to-pos1 (or stay open for a `wait < 0` / toggle door),
/// then fire the open/close targets and toggle areaportals on close. No oracle (entity-state +
/// target/areaportal control flow).
///
/// # Safety
/// `ent` must point to a valid mover `gentity_t`; `level`/`g_entities` must be initialised.
pub unsafe extern "C" fn Reached_BinaryMover(ent: *mut gentity_t) {
    // stop the looping sound
    (*ent).s.loopSound = 0;
    (*ent).s.loopIsSoundset = QFALSE;

    if (*ent).moverState == MOVER_1TO2 {
        // reached open
        let mut doorcenter: vec3_t = [0.0; 3];

        // reached pos2
        SetMoverState(ent, MOVER_POS2, (*addr_of!(level)).time);

        CalcTeamDoorCenter(ent, &mut doorcenter);

        // play sound
        G_PlayDoorSound(ent, BMS_END);

        if (*ent).wait < 0.0 {
            // Done for good
            (*ent).think = None;
            (*ent).nextthink = 0;
            (*ent).r#use = None;
        } else {
            // return to pos1 after a delay
            (*ent).think = Some(ReturnToPos1);
            if (*ent).spawnflags & 8 != 0 {
                // Toggle, keep think, wait for next use?
                (*ent).nextthink = -1;
            } else {
                (*ent).nextthink = ((*addr_of!(level)).time as f32 + (*ent).wait) as c_int;
            }
        }

        // fire targets
        if (*ent).activator.is_null() {
            (*ent).activator = ent;
        }
        G_UseTargets2(ent, (*ent).activator, (*ent).opentarget);
    } else if (*ent).moverState == MOVER_2TO1 {
        // closed
        let mut doorcenter: vec3_t = [0.0; 3];

        // reached pos1
        SetMoverState(ent, MOVER_POS1, (*addr_of!(level)).time);

        CalcTeamDoorCenter(ent, &mut doorcenter);

        // play sound
        G_PlayDoorSound(ent, BMS_END);

        // close areaportals
        if (*ent).teammaster == ent || (*ent).teammaster.is_null() {
            trap::AdjustAreaPortalState(ent, QFALSE);
        }
        G_UseTargets2(ent, (*ent).activator, (*ent).closetarget);
    } else {
        G_Error("Reached_BinaryMover: bad moverState");
    }
}

/// `void Use_BinaryMover_Go( gentity_t *ent )` (g_mover.c:701). The mover's actual move trigger
/// (run directly, or as a think after `ent->delay`): from pos1 it starts opening (loop+start
/// sounds, open areaportal, fire targets); from pos2 it schedules the close; mid-move it reverses
/// direction, computing the partial-travel time (with the non-linear `acos` correction) so the
/// reversal is seamless. No oracle (entity-state + trajectory math + target control flow).
///
/// # Safety
/// `ent` must point to a valid mover `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn Use_BinaryMover_Go(ent: *mut gentity_t) {
    let activator = (*ent).activator;

    (*ent).activator = activator;

    if (*ent).moverState == MOVER_POS1 {
        let mut doorcenter: vec3_t = [0.0; 3];

        // start moving 50 msec later, becase if this was player
        // triggered, level.time hasn't been advanced yet
        MatchTeam(ent, MOVER_1TO2, (*addr_of!(level)).time + 50);

        CalcTeamDoorCenter(ent, &mut doorcenter);

        // starting sound
        G_PlayDoorLoopSound(ent);
        G_PlayDoorSound(ent, BMS_START);
        (*ent).s.time = (*addr_of!(level)).time;

        // open areaportal
        if (*ent).teammaster == ent || (*ent).teammaster.is_null() {
            trap::AdjustAreaPortalState(ent, QTRUE);
        }
        G_UseTargets(ent, (*ent).activator);
        return;
    }

    // if all the way up, just delay before coming down
    if (*ent).moverState == MOVER_POS2 {
        //have to do this because the delay sets our think to Use_BinaryMover_Go
        (*ent).think = Some(ReturnToPos1);
        if (*ent).spawnflags & 8 != 0 {
            // TOGGLE doors don't use wait!
            (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME;
        } else {
            (*ent).nextthink = ((*addr_of!(level)).time as f32 + (*ent).wait) as c_int;
        }
        G_UseTargets2(ent, (*ent).activator, (*ent).target2);
        return;
    }

    // only partway down before reversing
    if (*ent).moverState == MOVER_2TO1 {
        let total;
        let mut partial;
        if (*ent).s.pos.trType == TR_NONLINEAR_STOP {
            let mut curDelta: vec3_t = [0.0; 3];
            total = (*ent).s.pos.trDuration - 50;
            VectorSubtract(&(*ent).r.currentOrigin, &(*ent).pos1, &mut curDelta);
            let mut fPartial = VectorLength(&curDelta) / VectorLength(&(*ent).s.pos.trDelta);
            let fp = fPartial;
            VectorScale(&(*ent).s.pos.trDelta, fp, &mut curDelta);
            fPartial /= (*ent).s.pos.trDuration as f32;
            fPartial /= 0.001f32;
            fPartial = (fPartial as f64).acos() as f32;
            fPartial = RAD2DEG(fPartial);
            fPartial = (90.0 - fPartial) / 90.0 * (*ent).s.pos.trDuration as f32;
            partial = total - (fPartial as f64).floor() as c_int;
        } else {
            total = (*ent).s.pos.trDuration;
            partial = (*addr_of!(level)).time - (*ent).s.pos.trTime;
        }

        if partial > total {
            partial = total;
        }
        (*ent).s.pos.trTime = (*addr_of!(level)).time - (total - partial); //ent->s.time;

        MatchTeam(ent, MOVER_1TO2, (*ent).s.pos.trTime);

        G_PlayDoorSound(ent, BMS_START);

        return;
    }

    // only partway up before reversing
    if (*ent).moverState == MOVER_1TO2 {
        let total;
        let mut partial;
        if (*ent).s.pos.trType == TR_NONLINEAR_STOP {
            let mut curDelta: vec3_t = [0.0; 3];
            total = (*ent).s.pos.trDuration - 50;
            VectorSubtract(&(*ent).r.currentOrigin, &(*ent).pos2, &mut curDelta);
            let mut fPartial = VectorLength(&curDelta) / VectorLength(&(*ent).s.pos.trDelta);
            let fp = fPartial;
            VectorScale(&(*ent).s.pos.trDelta, fp, &mut curDelta);
            fPartial /= (*ent).s.pos.trDuration as f32;
            fPartial /= 0.001f32;
            fPartial = (fPartial as f64).acos() as f32;
            fPartial = RAD2DEG(fPartial);
            fPartial = (90.0 - fPartial) / 90.0 * (*ent).s.pos.trDuration as f32;
            partial = total - (fPartial as f64).floor() as c_int;
        } else {
            total = (*ent).s.pos.trDuration;
            partial = (*addr_of!(level)).time - (*ent).s.pos.trTime;
        }
        if partial > total {
            partial = total;
        }

        (*ent).s.pos.trTime = (*addr_of!(level)).time - (total - partial); //ent->s.time;
        MatchTeam(ent, MOVER_2TO1, (*ent).s.pos.trTime);

        G_PlayDoorSound(ent, BMS_START);
    }
}

/// `void UnLockDoors( gentity_t *const ent )` (g_mover.c:824). Unlock a door and every team
/// slave: clear `MOVER_LOCKED`, advance to the second anim frame, and (for non-toggle doors)
/// drop the targetname so it can never be re-locked. No oracle (team-chain entity mutation).
///
/// # Safety
/// `ent` must point to a valid door `gentity_t`; its `teamchain` must be a valid list.
pub unsafe fn UnLockDoors(ent: *mut gentity_t) {
    //noise?
    //go through and unlock the door and all the slaves
    let mut slave = ent;
    loop {
        // want to allow locked toggle doors, so keep the targetname
        if (*slave).spawnflags & MOVER_TOGGLE == 0 {
            (*slave).targetname = null_mut(); //not usable ever again
        }
        (*slave).spawnflags &= !MOVER_LOCKED;
        (*slave).s.frame = 1; //second stage of anim
        slave = (*slave).teamchain;
        if slave.is_null() {
            break;
        }
    }
}

/// `void LockDoors( gentity_t *const ent )` (g_mover.c:840). Lock a door and every team slave:
/// set `MOVER_LOCKED` and reset to the first anim frame. No oracle (team-chain entity mutation).
///
/// # Safety
/// `ent` must point to a valid door `gentity_t`; its `teamchain` must be a valid list.
pub unsafe fn LockDoors(ent: *mut gentity_t) {
    //noise?
    //go through and lock the door and all the slaves
    let mut slave = ent;
    loop {
        (*slave).spawnflags |= MOVER_LOCKED;
        (*slave).s.frame = 0; //first stage of anim
        slave = (*slave).teamchain;
        if slave.is_null() {
            break;
        }
    }
}

/// `void Use_BinaryMover( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_mover.c:855). The mover's `use` callback: routes to the team master, bails on inactive
/// movers, unlocks a locked door (and stops), runs the `BSET_USE` script, then either fires
/// `Use_BinaryMover_Go` immediately or schedules it after `ent->delay`. No oracle (entity-state +
/// callback dispatch).
///
/// # Safety
/// `ent` must point to a valid mover `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn Use_BinaryMover(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*ent).r#use.is_none() {
        //I cannot be used anymore, must be a door with a wait of -1 that's opened.
        return;
    }

    // only the master should be used
    if (*ent).flags & FL_TEAMSLAVE != 0 {
        Use_BinaryMover((*ent).teammaster, other, activator);
        return;
    }

    if (*ent).flags & FL_INACTIVE != 0 {
        return;
    }

    if (*ent).spawnflags & MOVER_LOCKED != 0 {
        //a locked door, unlock it
        UnLockDoors(ent);
        return;
    }

    G_ActivateBehavior(ent, BSET_USE);

    (*ent).enemy = other;
    (*ent).activator = activator;
    if (*ent).delay != 0 {
        (*ent).think = Some(Use_BinaryMover_Go);
        (*ent).nextthink = (*addr_of!(level)).time + (*ent).delay;
    } else {
        Use_BinaryMover_Go(ent);
    }
}

/// `void InitMoverTrData( gentity_t *ent )` (g_mover.c:888). Build the mover's resting position
/// trajectory from `pos1`/`pos2`/`speed`: stationary at pos1, with `trDelta` along the move and
/// `trDuration` the time to cover it at `speed` (defaulting speed to 100). No oracle (entity-state
/// mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with `pos1`/`pos2` set.
pub unsafe fn InitMoverTrData(ent: *mut gentity_t) {
    let mut move_: vec3_t = [0.0; 3];

    (*ent).s.pos.trType = TR_STATIONARY;
    VectorCopy(&(*ent).pos1, &mut (*ent).s.pos.trBase);

    // calculate time to reach second position from speed
    VectorSubtract(&(*ent).pos2, &(*ent).pos1, &mut move_);
    let distance = VectorLength(&move_);
    if (*ent).speed == 0.0 {
        (*ent).speed = 100.0;
    }
    VectorScale(&move_, (*ent).speed, &mut (*ent).s.pos.trDelta);
    (*ent).s.pos.trDuration = (distance * 1000.0 / (*ent).speed) as c_int;
    if (*ent).s.pos.trDuration <= 0 {
        (*ent).s.pos.trDuration = 1;
    }
}

/// `void InitMover( gentity_t *ent )` (g_mover.c:919). Spawn-time setup for a binary mover:
/// resolve the `model2` draw model (ignoring unsupported `.glm`), pack the optional `light`/
/// `color` into `constantLight`, wire the `use`/`reached` callbacks, set `MOVER_POS1`/`ET_MOVER`
/// and the inactive/player-usable svFlags, seat it at pos1, link, and build its trajectory via
/// [`InitMoverTrData`]. No oracle (spawn-key reads + entity-state mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the spawn-var system must be active for the current
/// entity (`InitMover` reads `light`/`color` keys).
pub unsafe fn InitMover(ent: *mut gentity_t) {
    let mut light: f32 = 0.0;
    let mut color: vec3_t = [0.0; 3];

    // if the "model2" key is set, use a seperate model
    // for drawing, but clip against the brushes
    if !(*ent).model2.is_null() {
        if CStr::from_ptr((*ent).model2).to_string_lossy().contains(".glm") {
            //for now, not supported in MP.
            (*ent).s.modelindex2 = 0;
        } else {
            (*ent).s.modelindex2 = G_ModelIndex(&CStr::from_ptr((*ent).model2).to_string_lossy());
        }
    }

    // if the "color" or "light" keys are set, setup constantLight
    let lightSet = G_SpawnFloat(c"light".as_ptr(), c"100".as_ptr(), &mut light);
    let colorSet = G_SpawnVector(c"color".as_ptr(), c"1 1 1".as_ptr(), color.as_mut_ptr());
    if lightSet != QFALSE || colorSet != QFALSE {
        let mut r = (color[0] * 255.0) as c_int;
        if r > 255 {
            r = 255;
        }
        let mut g = (color[1] * 255.0) as c_int;
        if g > 255 {
            g = 255;
        }
        let mut b = (color[2] * 255.0) as c_int;
        if b > 255 {
            b = 255;
        }
        let mut i = (light / 4.0) as c_int;
        if i > 255 {
            i = 255;
        }
        (*ent).s.constantLight = r | (g << 8) | (b << 16) | (i << 24);
    }

    (*ent).r#use = Some(Use_BinaryMover);
    (*ent).reached = Some(Reached_BinaryMover);

    (*ent).moverState = MOVER_POS1;
    (*ent).r.svFlags = SVF_USE_CURRENT_ORIGIN;
    if (*ent).spawnflags & MOVER_INACTIVE != 0 {
        // Make it inactive
        (*ent).flags |= FL_INACTIVE;
    }
    if (*ent).spawnflags & MOVER_PLAYER_USE != 0 {
        //Can be used by the player's BUTTON_USE
        (*ent).r.svFlags |= SVF_PLAYER_USABLE;
    }
    (*ent).s.eType = ET_MOVER;
    VectorCopy(&(*ent).pos1, &mut (*ent).r.currentOrigin);
    trap::LinkEntity(ent);

    InitMoverTrData(ent);
}

/// `void Think_MatchTeam( gentity_t *ent )` (g_mover.c:1209). Think function that re-syncs a
/// mover team to the captain's current state at `level.time`. No oracle (wraps [`MatchTeam`]).
///
/// # Safety
/// `ent` must point to a valid mover-team captain `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn Think_MatchTeam(ent: *mut gentity_t) {
    MatchTeam(ent, (*ent).moverState, (*addr_of!(level)).time);
}

/// `void Blocked_Door( gentity_t *ent, gentity_t *other )` (g_mover.c:1008). A mover's `blocked`
/// callback: crush the obstruction for `ent->damage`, and unless this is a `MOVER_CRUSHER` (which
/// never reverses), bounce the door back the other way via [`Use_BinaryMover`]. No oracle
/// (entity-state + callback dispatch).
///
/// # Safety
/// `ent`/`other` must point to valid `gentity_t`s.
pub unsafe extern "C" fn Blocked_Door(ent: *mut gentity_t, other: *mut gentity_t) {
    if (*ent).damage != 0 {
        G_Damage(
            other,
            ent,
            ent,
            null_mut(),
            null_mut(),
            (*ent).damage,
            0,
            MOD_CRUSH,
        );
    }
    if (*ent).spawnflags & MOVER_CRUSHER != 0 {
        return; // crushers don't reverse
    }

    // reverse direction
    Use_BinaryMover(ent, ent, other);
}

/// `static void Touch_DoorTriggerSpectator( gentity_t *ent, gentity_t *other, trace_t *trace )`
/// (g_mover.c:1027). A spectator standing in a closed door's trigger is nudged through to the far
/// side: pick the nearer face along the door's thin axis (`ent->count`), build an origin just
/// outside that face, and if a player-sized box-trace there comes back clear, teleport the
/// spectator to it facing away from the door. No oracle (trap trace + teleport).
///
/// **DEVIATION (OpenJK parity):** JKA additionally gates the teleport on
/// `CM_LeafCluster(CM_PointLeafnum(origin)) != -1` ("don't teleport out of the world"), but
/// `CM_PointLeafnum`/`CM_LeafCluster` are engine collision-model internals (`qcommon/cm_*`) — they
/// are **not** part of the game module's `trap_*` syscall ABI, and the runtime engine (OpenJK)
/// exposes no such syscall and itself drops the guard. We follow OpenJK and omit it; the four
/// trace conditions remain. See `crate/DEVIATIONS.md`.
///
/// # Safety
/// `ent`/`other` must point to valid `gentity_t`s; `ent` is the door's auto-trigger with `count`
/// set to the thin axis.
pub unsafe fn Touch_DoorTriggerSpectator(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    let axis = (*ent).count;
    let mut dir: vec3_t = [0.0; 3];
    let mut origin: vec3_t = [0.0; 3];
    VectorClear(&mut dir);
    if ((*other).s.origin[axis as usize] - (*ent).r.absmax[axis as usize]).abs()
        < ((*other).s.origin[axis as usize] - (*ent).r.absmin[axis as usize]).abs()
    {
        origin[axis as usize] = (*ent).r.absmin[axis as usize] - 10.0;
        dir[axis as usize] = -1.0;
    } else {
        origin[axis as usize] = (*ent).r.absmax[axis as usize] + 10.0;
        dir[axis as usize] = 1.0;
    }
    for i in 0..3 {
        if i == axis as usize {
            continue;
        }
        origin[i] = ((*ent).r.absmin[i] + (*ent).r.absmax[i]) * 0.5;
    }

    let mut angles: vec3_t = [0.0; 3];
    vectoangles(&dir, &mut angles);

    let mut p_mins: vec3_t = [0.0; 3];
    let mut p_maxs: vec3_t = [0.0; 3];
    VectorSet(&mut p_mins, -15.0, -15.0, DEFAULT_MINS_2 as f32);
    VectorSet(&mut p_maxs, 15.0, 15.0, DEFAULT_MAXS_2 as f32);
    let tr = trap::Trace(
        &origin,
        &p_mins,
        &p_maxs,
        &origin,
        (*other).s.number,
        (*other).clipmask,
    );
    if tr.startsolid == 0
        && tr.allsolid == 0
        && tr.fraction == 1.0
        && tr.entityNum as c_int == ENTITYNUM_NONE
    // JKA also requires CM_LeafCluster(CM_PointLeafnum(origin)) != -1 here; dropped for OpenJK parity (see fn doc)
    {
        TeleportPlayer(other, &origin, &angles);
    }
}

/// `void Touch_DoorTrigger( gentity_t *ent, gentity_t *other, trace_t *trace )` (g_mover.c:1070).
/// The door's auto-trigger `touch`: spectators get shoved through closed doors via
/// [`Touch_DoorTriggerSpectator`]; vehicles (and players riding them) can't open doors unless the
/// door is `vehopen` (`genericValue14`); otherwise, if the door isn't inactive and isn't a locked
/// door this toucher may not open, kick the parent mover open via [`Use_BinaryMover`] (temporarily
/// clearing `MOVER_LOCKED` for an allowed team so the whole team isn't unlocked). No oracle
/// (callback dispatch + entity-state).
///
/// # Safety
/// `ent`/`other` must point to valid `gentity_t`s; `ent->parent` is the door mover.
pub unsafe extern "C" fn Touch_DoorTrigger(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    trace: *mut trace_t,
) {
    let mut relock_ent: *mut gentity_t = null_mut();

    if !(*other).client.is_null() && (*(*other).client).sess.sessionTeam == TEAM_SPECTATOR {
        // if the door is not open and not opening
        if (*(*ent).parent).moverState != MOVER_1TO2 && (*(*ent).parent).moverState != MOVER_POS2 {
            Touch_DoorTriggerSpectator(ent, other, trace);
        }
        return;
    }

    if (*ent).genericValue14 == 0
        && ((*ent).parent.is_null() || (*(*ent).parent).genericValue14 == 0)
    {
        if !(*other).client.is_null()
            && (*other).s.number >= MAX_CLIENTS as c_int
            && (*other).s.eType == ET_NPC as c_int
            && (*other).s.NPC_class == CLASS_VEHICLE
        {
            // doors don't open for vehicles
            return;
        }

        if !(*other).client.is_null()
            && (*other).s.number < MAX_CLIENTS as c_int
            && (*(*other).client).ps.m_iVehicleNum != 0
        {
            // can't open a door while on a vehicle
            return;
        }
    }

    if (*ent).flags & FL_INACTIVE != 0 {
        return;
    }

    if (*(*ent).parent).spawnflags & MOVER_LOCKED != 0 {
        // don't even try to use the door if it's locked
        if (*(*ent).parent).alliedTeam == 0 // we don't have a "teamallow" team
            || (*other).client.is_null() // we do have a "teamallow" team, but this isn't a client
            || (*(*other).client).sess.sessionTeam != (*(*ent).parent).alliedTeam
        // it is a client, but it's not on the right team
        {
            return;
        } else {
            // temporarily unlock us while we call Use_BinaryMover (so it doesn't unlock all the doors in this team)
            if (*(*ent).parent).flags & FL_TEAMSLAVE != 0 {
                relock_ent = (*(*ent).parent).teammaster;
            } else {
                relock_ent = (*ent).parent;
            }
            if !relock_ent.is_null() {
                (*relock_ent).spawnflags &= !MOVER_LOCKED;
            }
        }
    }

    if (*(*ent).parent).moverState != MOVER_1TO2 {
        // Door is not already opening
        // If door is closed, opening or open, check this
        Use_BinaryMover((*ent).parent, ent, other);
    }
    if !relock_ent.is_null() {
        // re-lock us
        (*relock_ent).spawnflags |= MOVER_LOCKED;
    }
}

/// `void Think_SpawnNewDoorTrigger( gentity_t *ent )` (g_mover.c:1160). After every part of a door
/// team has spawned, enclose the whole team in one auto-trigger: union the team's bounds, expand
/// the thinnest axis ±120u, spawn a `CONTENTS_TRIGGER` `trigger_door` parented to the door with
/// [`Touch_DoorTrigger`] as its touch (its `count` remembering the thin axis), then sync the team
/// state via [`MatchTeam`]. Also re-marks the slaves shootable if the master takes damage. No
/// oracle ([`G_Spawn`] + trap link).
///
/// # Safety
/// `ent` must point to a valid door master `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn Think_SpawnNewDoorTrigger(ent: *mut gentity_t) {
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    // set all of the slaves as shootable
    if (*ent).takedamage != QFALSE {
        let mut other = ent;
        while !other.is_null() {
            (*other).takedamage = QTRUE;
            other = (*other).teamchain;
        }
    }

    // find the bounds of everything on the team
    VectorCopy(&(*ent).r.absmin, &mut mins);
    VectorCopy(&(*ent).r.absmax, &mut maxs);

    let mut other = (*ent).teamchain;
    while !other.is_null() {
        AddPointToBounds(&(*other).r.absmin, &mut mins, &mut maxs);
        AddPointToBounds(&(*other).r.absmax, &mut mins, &mut maxs);
        other = (*other).teamchain;
    }

    // find the thinnest axis, which will be the one we expand
    let mut best = 0usize;
    for i in 1..3 {
        if maxs[i] - mins[i] < maxs[best] - mins[best] {
            best = i;
        }
    }
    maxs[best] += 120.0;
    mins[best] -= 120.0;

    // create a trigger with this size
    let trigger = G_Spawn();
    VectorCopy(&mins, &mut (*trigger).r.mins);
    VectorCopy(&maxs, &mut (*trigger).r.maxs);
    (*trigger).parent = ent;
    (*trigger).r.contents = CONTENTS_TRIGGER;
    (*trigger).touch = Some(Touch_DoorTrigger);
    trap::LinkEntity(trigger);
    (*trigger).classname = c"trigger_door".as_ptr() as *mut c_char;
    // remember the thinnest axis
    (*trigger).count = best as c_int;

    MatchTeam(ent, (*ent).moverState, (*addr_of!(level)).time);
}

/// `qboolean G_EntIsDoor( int entityNum )` (g_mover.c:1214). True iff the slot is a real entity
/// (not the world/none) whose classname is `func_door`. No oracle (entity-table lookup +
/// classname compare).
///
/// # Safety
/// `g_entities` must be initialised.
pub unsafe fn G_EntIsDoor(entityNum: c_int) -> qboolean {
    if entityNum < 0 || entityNum >= ENTITYNUM_WORLD {
        return QFALSE;
    }

    let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityNum as usize);
    if !ent.is_null() && Q_stricmp(c"func_door".as_ptr(), (*ent).classname) == 0 {
        // blocked by a door
        return QTRUE;
    }
    QFALSE
}

/// `gentity_t *G_FindDoorTrigger( gentity_t *ent )` (g_mover.c:1231). Find the trigger that owns a
/// door: walk up to the team master, then prefer a `CONTENTS_TRIGGER` ent targeting the door by
/// `target`/`target2`, falling back to the auto-spawned `trigger_door` whose `parent` is the door.
/// Returns null if none. No oracle ([`G_Find`] table walk + classname/parent compares).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `g_entities`/`level` must be initialised.
pub unsafe fn G_FindDoorTrigger(ent: *mut gentity_t) -> *mut gentity_t {
    let mut owner: *mut gentity_t = null_mut();
    let mut door = ent;
    if (*door).flags & FL_TEAMSLAVE != 0 {
        // not the master door, get the master door
        while !(*door).teammaster.is_null() && (*door).flags & FL_TEAMSLAVE != 0 {
            door = (*door).teammaster;
        }
    }
    if !(*door).targetname.is_null() {
        // find out what is targeting it
        // FIXME: if ent->targetname, check what kind of trigger/ent is targetting it?  If a normal trigger (active, etc), then it's okay?
        loop {
            owner = G_Find(owner, offset_of!(gentity_t, target), (*door).targetname);
            if owner.is_null() {
                break;
            }
            if (*owner).r.contents & CONTENTS_TRIGGER != 0 {
                return owner;
            }
        }
        owner = null_mut();
        loop {
            owner = G_Find(owner, offset_of!(gentity_t, target2), (*door).targetname);
            if owner.is_null() {
                break;
            }
            if (*owner).r.contents & CONTENTS_TRIGGER != 0 {
                return owner;
            }
        }
    }

    owner = null_mut();
    loop {
        owner = G_Find(owner, offset_of!(gentity_t, classname), c"trigger_door".as_ptr());
        if owner.is_null() {
            break;
        }
        if (*owner).parent == door {
            return owner;
        }
    }

    null_mut()
}

/// `qboolean G_EntIsUnlockedDoor( int entityNum )` (g_mover.c:1274). True iff the slot is a door
/// that a player can currently open just by walking up: an active `trigger_multiple` targets it
/// (or its auto-trigger is active) and it isn't shoot-to-open, player-use-only, force-activate, or
/// locked. No oracle (door predicate over [`G_Find`]/[`G_FindDoorTrigger`]).
///
/// # Safety
/// `g_entities`/`level` must be initialised.
pub unsafe fn G_EntIsUnlockedDoor(entityNum: c_int) -> qboolean {
    if entityNum < 0 || entityNum >= ENTITYNUM_WORLD {
        return QFALSE;
    }

    if G_EntIsDoor(entityNum) != QFALSE {
        let mut ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityNum as usize);
        let mut owner: *mut gentity_t;
        if (*ent).flags & FL_TEAMSLAVE != 0 {
            // not the master door, get the master door
            while !(*ent).teammaster.is_null() && (*ent).flags & FL_TEAMSLAVE != 0 {
                ent = (*ent).teammaster;
            }
        }
        if !(*ent).targetname.is_null() {
            // find out what is targetting it
            owner = null_mut();
            // FIXME: if ent->targetname, check what kind of trigger/ent is targetting it?  If a normal trigger (active, etc), then it's okay?
            loop {
                owner = G_Find(owner, offset_of!(gentity_t, target), (*ent).targetname);
                if owner.is_null() {
                    break;
                }
                if Q_stricmp(c"trigger_multiple".as_ptr(), (*owner).classname) == 0
                // FIXME: other triggers okay too?
                {
                    if (*owner).flags & FL_INACTIVE == 0 {
                        return QTRUE;
                    }
                }
            }
            owner = null_mut();
            loop {
                owner = G_Find(owner, offset_of!(gentity_t, target2), (*ent).targetname);
                if owner.is_null() {
                    break;
                }
                if Q_stricmp(c"trigger_multiple".as_ptr(), (*owner).classname) == 0
                // FIXME: other triggers okay too?
                {
                    if (*owner).flags & FL_INACTIVE == 0 {
                        return QTRUE;
                    }
                }
            }
            return QFALSE;
        } else {
            // check the door's auto-created trigger instead
            owner = G_FindDoorTrigger(ent);
            if !owner.is_null() && (*owner).flags & FL_INACTIVE != 0 {
                // owning auto-created trigger is inactive
                return QFALSE;
            }
        }
        if (*ent).flags & FL_INACTIVE == 0 // assumes that the reactivate trigger isn't right next to the door!
            && (*ent).health == 0
            && (*ent).spawnflags & MOVER_PLAYER_USE == 0
            && (*ent).spawnflags & MOVER_FORCE_ACTIVATE == 0
            && (*ent).spawnflags & MOVER_LOCKED == 0
        // FIXME: what about MOVER_GOODIE?
        {
            return QTRUE;
        }
    }
    QFALSE
}

/// `void SP_func_door( gentity_t *ent )` (g_mover.c:1372). Spawn a `func_door`: a binary mover
/// whose closed rest is its spawn origin (`pos1`) and whose open position (`pos2`) is `movedir`
/// projected across the brush extent minus `lip`; `start_open` (spawnflag 1) swaps the two.
/// Reads `vehopen`/`lip`/`dmg`/`teamallow`/`health` keys, installs [`Blocked_Door`], runs
/// [`InitMover`], and — for the team master — schedules either [`Think_MatchTeam`] (targeted /
/// shoot / player-use / force-activate doors) or [`Think_SpawnNewDoorTrigger`] (auto-trigger
/// doors, including locked ones) one frame out. Locked doors flag `EF_SHADER_ANIM`. No oracle
/// (spawn-key reads + trap brush-model setup).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning; `level` must be initialised.
pub unsafe fn SP_func_door(ent: *mut gentity_t) {
    let mut abs_movedir: vec3_t = [0.0; 3];
    let mut size: vec3_t = [0.0; 3];
    let mut lip: f32 = 0.0;

    G_SpawnInt(
        c"vehopen".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue14,
    );

    (*ent).blocked = Some(Blocked_Door);

    // default speed of 400
    if (*ent).speed == 0.0 {
        (*ent).speed = 400.0;
    }

    // default wait of 2 seconds
    if (*ent).wait == 0.0 {
        (*ent).wait = 2.0;
    }
    (*ent).wait *= 1000.0;

    (*ent).delay *= 1000;

    // default lip of 8 units
    G_SpawnFloat(c"lip".as_ptr(), c"8".as_ptr(), &mut lip);

    // default damage of 2 points
    G_SpawnInt(c"dmg".as_ptr(), c"2".as_ptr(), &mut (*ent).damage);
    if (*ent).damage < 0 {
        (*ent).damage = 0;
    }

    G_SpawnInt(c"teamallow".as_ptr(), c"0".as_ptr(), &mut (*ent).alliedTeam);

    // first position at start
    VectorCopy(&(*ent).s.origin, &mut (*ent).pos1);

    // calculate second position
    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());
    G_SetMovedir(&mut (*ent).s.angles, &mut (*ent).movedir);
    abs_movedir[0] = (*ent).movedir[0].abs();
    abs_movedir[1] = (*ent).movedir[1].abs();
    abs_movedir[2] = (*ent).movedir[2].abs();
    VectorSubtract(&(*ent).r.maxs, &(*ent).r.mins, &mut size);
    let distance = DotProduct(&abs_movedir, &size) - lip;
    let pos1 = (*ent).pos1;
    VectorMA(&pos1, distance, &(*ent).movedir, &mut (*ent).pos2);

    // if "start_open", reverse position 1 and 2
    if (*ent).spawnflags & 1 != 0 {
        let mut temp: vec3_t = [0.0; 3];
        VectorCopy(&(*ent).pos2, &mut temp);
        VectorCopy(&(*ent).s.origin, &mut (*ent).pos2);
        VectorCopy(&temp, &mut (*ent).pos1);
    }

    if (*ent).spawnflags & MOVER_LOCKED != 0 {
        // a locked door, set up as locked until used directly
        (*ent).s.eFlags |= EF_SHADER_ANIM; // use frame-controlled shader anim
        (*ent).s.frame = 0; // first stage of anim
    }
    InitMover(ent);

    (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME;

    if (*ent).flags & FL_TEAMSLAVE == 0 {
        let mut health: c_int = 0;

        G_SpawnInt(c"health".as_ptr(), c"0".as_ptr(), &mut health);

        if health != 0 {
            (*ent).takedamage = QTRUE;
        }

        if (*ent).spawnflags & MOVER_LOCKED == 0
            && (!(*ent).targetname.is_null()
                || health != 0
                || (*ent).spawnflags & MOVER_PLAYER_USE != 0
                || (*ent).spawnflags & MOVER_FORCE_ACTIVATE != 0)
        {
            // non touch/shoot doors
            (*ent).think = Some(Think_MatchTeam);

            if (*ent).spawnflags & MOVER_FORCE_ACTIVATE != 0 {
                // so we know it's push/pullable on the client
                (*ent).s.bolt1 = 1;
            }
        } else {
            // locked doors still spawn a trigger
            (*ent).think = Some(Think_SpawnNewDoorTrigger);
        }
    }
}

/// `void Touch_Plat( gentity_t *ent, gentity_t *other, trace_t *trace )` (g_mover.c:1481). Plat
/// `touch`: a living player standing on a raised plat (`MOVER_POS2`) delays its return-to-bottom
/// by one second. Ignores non-clients and the dead. No oracle (entity-state mutation).
///
/// # Safety
/// `ent`/`other` must point to valid `gentity_t`s; `level` must be initialised.
pub unsafe extern "C" fn Touch_Plat(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if (*other).client.is_null() || (*(*other).client).ps.stats[STAT_HEALTH as usize] <= 0 {
        return;
    }

    // delay return-to-pos1 by one second
    if (*ent).moverState == MOVER_POS2 {
        (*ent).nextthink = (*addr_of!(level)).time + 1000;
    }
}

/// `void Touch_PlatCenterTrigger( gentity_t *ent, gentity_t *other, trace_t *trace )`
/// (g_mover.c:1499). The plat's center trigger: when a client steps in and the plat is resting at
/// the bottom (`MOVER_POS1`), start it going up via [`Use_BinaryMover`]. No oracle (callback
/// dispatch).
///
/// # Safety
/// `ent`/`other` must point to valid `gentity_t`s; `ent->parent` must be the plat.
pub unsafe extern "C" fn Touch_PlatCenterTrigger(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if (*other).client.is_null() {
        return;
    }

    if (*(*ent).parent).moverState == MOVER_POS1 {
        Use_BinaryMover((*ent).parent, ent, other);
    }
}

/// `void SpawnPlatTrigger( gentity_t *ent )` (g_mover.c:1519). Spawn the plat's center trigger: a
/// thin `CONTENTS_TRIGGER` box just above the plat's bottom position (inset 33u on x/y, expanding
/// to a 1u sliver if the plat is too small), parented to the plat with [`Touch_PlatCenterTrigger`]
/// as its touch. No oracle ([`G_Spawn`] + trap link).
///
/// # Safety
/// `ent` must point to a valid plat `gentity_t` with `pos1`/bounds set.
pub unsafe fn SpawnPlatTrigger(ent: *mut gentity_t) {
    let mut tmin: vec3_t = [0.0; 3];
    let mut tmax: vec3_t = [0.0; 3];

    // the middle trigger will be a thin trigger just
    // above the starting position
    let trigger = G_Spawn();
    (*trigger).touch = Some(Touch_PlatCenterTrigger);
    (*trigger).r.contents = CONTENTS_TRIGGER;
    (*trigger).parent = ent;

    tmin[0] = (*ent).pos1[0] + (*ent).r.mins[0] + 33.0;
    tmin[1] = (*ent).pos1[1] + (*ent).r.mins[1] + 33.0;
    tmin[2] = (*ent).pos1[2] + (*ent).r.mins[2];

    tmax[0] = (*ent).pos1[0] + (*ent).r.maxs[0] - 33.0;
    tmax[1] = (*ent).pos1[1] + (*ent).r.maxs[1] - 33.0;
    tmax[2] = (*ent).pos1[2] + (*ent).r.maxs[2] + 8.0;

    if tmax[0] <= tmin[0] {
        tmin[0] = (*ent).pos1[0] + ((*ent).r.mins[0] + (*ent).r.maxs[0]) * 0.5;
        tmax[0] = tmin[0] + 1.0;
    }
    if tmax[1] <= tmin[1] {
        tmin[1] = (*ent).pos1[1] + ((*ent).r.mins[1] + (*ent).r.maxs[1]) * 0.5;
        tmax[1] = tmin[1] + 1.0;
    }

    VectorCopy(&tmin, &mut (*trigger).r.mins);
    VectorCopy(&tmax, &mut (*trigger).r.maxs);

    trap::LinkEntity(trigger);
}

/// `void Touch_Button( gentity_t *ent, gentity_t *other, trace_t *trace )` (g_mover.c:1625). Button
/// `touch`: a client touching a button at rest (`MOVER_POS1`) presses it via [`Use_BinaryMover`].
/// No oracle (callback dispatch).
///
/// # Safety
/// `ent`/`other` must point to valid `gentity_t`s.
pub unsafe extern "C" fn Touch_Button(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if (*other).client.is_null() {
        return;
    }

    if (*ent).moverState == MOVER_POS1 {
        Use_BinaryMover(ent, other, other);
    }
}

/// `void Think_BeginMoving( gentity_t *ent )` (g_mover.c:1719). Train think: the wait at a corner
/// is over, so kick the start sound, (re)start the door loop, and stamp the trajectory to move
/// linearly from now. No oracle (sound helpers + trajectory stamp).
///
/// # Safety
/// `ent` must point to a valid train `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn Think_BeginMoving(ent: *mut gentity_t) {
    G_PlayDoorSound(ent, BMS_START);
    G_PlayDoorLoopSound(ent);
    (*ent).s.pos.trTime = (*addr_of!(level)).time;
    (*ent).s.pos.trType = TR_LINEAR_STOP;
}

/// `void SP_func_plat( gentity_t *ent )` (g_mover.c:1568). Spawn a `func_plat`: a binary mover
/// whose rest position (`pos1`) is `height` below its top (`pos2` = spawn origin); `height`
/// defaults to the brush model's z-extent minus `lip`. Installs [`Touch_Plat`] (keeps the plat
/// down while a live player stands on it), [`Blocked_Door`], parents the plat to itself so it
/// reads as a door, and auto-spawns the center trigger via [`SpawnPlatTrigger`] unless it has a
/// `targetname`. No oracle (spawn-key reads + trap brush-model setup).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning.
pub unsafe fn SP_func_plat(ent: *mut gentity_t) {
    let mut height: f32 = 0.0;
    let mut lip: f32 = 0.0;

    VectorClear(&mut (*ent).s.angles);

    G_SpawnFloat(c"speed".as_ptr(), c"200".as_ptr(), &mut (*ent).speed);
    G_SpawnInt(c"dmg".as_ptr(), c"2".as_ptr(), &mut (*ent).damage);
    G_SpawnFloat(c"wait".as_ptr(), c"1".as_ptr(), &mut (*ent).wait);
    G_SpawnFloat(c"lip".as_ptr(), c"8".as_ptr(), &mut lip);

    (*ent).wait = 1000.0;

    // create second position
    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());

    if G_SpawnFloat(c"height".as_ptr(), c"0".as_ptr(), &mut height) == QFALSE {
        height = ((*ent).r.maxs[2] - (*ent).r.mins[2]) - lip;
    }

    // pos1 is the rest (bottom) position, pos2 is the top
    VectorCopy(&(*ent).s.origin, &mut (*ent).pos2);
    VectorCopy(&(*ent).pos2, &mut (*ent).pos1);
    (*ent).pos1[2] -= height;

    InitMover(ent);

    // touch function keeps the plat from returning while
    // a live player is standing on it
    (*ent).touch = Some(Touch_Plat);

    (*ent).blocked = Some(Blocked_Door);

    (*ent).parent = ent; // so it can be treated as a door

    // spawn the trigger if one hasn't been custom made
    if (*ent).targetname.is_null() {
        SpawnPlatTrigger(ent);
    }
}

/// `void SP_func_button( gentity_t *ent )` (g_mover.c:1652). Spawn a `func_button`: a binary mover
/// that slides `lip`-shy of its brush size along its `angle` (`pos1` = origin, `pos2` =
/// `pos1 + distance·movedir`). Speed defaults to 40, wait to 1s. A button with `health` is
/// shootable (`takedamage`); otherwise it installs [`Touch_Button`]. No oracle (spawn-key reads +
/// trap brush-model setup).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning.
pub unsafe fn SP_func_button(ent: *mut gentity_t) {
    let mut abs_movedir: vec3_t = [0.0; 3];
    let mut size: vec3_t = [0.0; 3];
    let mut lip: f32 = 0.0;

    if (*ent).speed == 0.0 {
        (*ent).speed = 40.0;
    }

    if (*ent).wait == 0.0 {
        (*ent).wait = 1.0;
    }
    (*ent).wait *= 1000.0;

    // first position
    VectorCopy(&(*ent).s.origin, &mut (*ent).pos1);

    // calculate second position
    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());

    G_SpawnFloat(c"lip".as_ptr(), c"4".as_ptr(), &mut lip);

    G_SetMovedir(&mut (*ent).s.angles, &mut (*ent).movedir);
    abs_movedir[0] = (*ent).movedir[0].abs();
    abs_movedir[1] = (*ent).movedir[1].abs();
    abs_movedir[2] = (*ent).movedir[2].abs();
    VectorSubtract(&(*ent).r.maxs, &(*ent).r.mins, &mut size);
    let distance =
        abs_movedir[0] * size[0] + abs_movedir[1] * size[1] + abs_movedir[2] * size[2] - lip;
    VectorMA(&(*ent).pos1, distance, &(*ent).movedir, &mut (*ent).pos2);

    if (*ent).health != 0 {
        // shootable button
        (*ent).takedamage = QTRUE;
    } else {
        // touchable button
        (*ent).touch = Some(Touch_Button);
    }

    InitMover(ent);
}

/// `void func_rotating_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_mover.c:2030). `func_rotating` `use`: toggle the spin. If currently spinning (`TR_LINEAR`),
/// stop it (`TR_STATIONARY`), kill the loop sound, and fire the one-shot stop sound (`BMS_END`);
/// otherwise fire the start sound (`BMS_START`), latch the mid-loop sound, and start spinning.
/// Sound work is skipped when the entity has no `soundSet`. No oracle (callback dispatch +
/// entity-state mutation). Note: its spawner `SP_func_rotating` is still blocked on
/// `SP_func_breakable` (breakable track); this `use` callback lands independently.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn func_rotating_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    if (*self_).s.apos.trType == TR_LINEAR {
        (*self_).s.apos.trType = TR_STATIONARY;
        // stop the sound if it stops moving
        (*self_).s.loopSound = 0;
        (*self_).s.loopIsSoundset = QFALSE;
        // play stop sound too?
        if !(*self_).soundSet.is_null() && *(*self_).soundSet != 0 {
            (*self_).s.soundSetIndex =
                G_SoundSetIndex(&CStr::from_ptr((*self_).soundSet).to_string_lossy());
            G_AddEvent(self_, EV_BMODEL_SOUND, BMS_END);
        }
    } else {
        if !(*self_).soundSet.is_null() && *(*self_).soundSet != 0 {
            (*self_).s.soundSetIndex =
                G_SoundSetIndex(&CStr::from_ptr((*self_).soundSet).to_string_lossy());
            G_AddEvent(self_, EV_BMODEL_SOUND, BMS_START);
            (*self_).s.loopSound = BMS_MID;
            (*self_).s.loopIsSoundset = QTRUE;
        }
        (*self_).s.apos.trType = TR_LINEAR;
    }
}

/// `void SP_func_static( gentity_t *ent )` (g_mover.c:1948). Spawn a `func_static`: a brush model
/// that just sits there (conditional walls / draw-only models). Seats it at its origin as a
/// zero-travel mover (`pos1` == `pos2`), installs [`func_static_use`], and applies the spawnflag
/// dressing — `2048` and the `hyperspace` key force `SVF_BROADCAST` (huge area-portal-touching
/// brushes), `SWITCH_SHADER` (4) enables the frame-driven shader anim, push/pull flags (1|2) set
/// `bolt1` so the client knows it is Force-movable. Clamps `model2scale` to 0..1023 and, inside a
/// BSP instance, marks it `EF_PERMANENT`. No oracle (spawn-key reads + entity-state mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning.
pub unsafe fn SP_func_static(ent: *mut gentity_t) {
    let mut test: c_int = 0;
    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());

    VectorCopy(&(*ent).s.origin, &mut (*ent).pos1);
    VectorCopy(&(*ent).s.origin, &mut (*ent).pos2);

    InitMover(ent);

    (*ent).r#use = Some(func_static_use);
    (*ent).reached = None;

    G_SetOrigin(ent, &(*ent).s.origin);
    G_SetAngles(ent, &(*ent).s.angles);

    if (*ent).spawnflags & 2048 != 0 {
        // yes this is very very evil, but for now (pre-alpha) it's a solution
        // I need to rotate something that is huge and it's touching too many area portals...
        (*ent).r.svFlags |= SVF_BROADCAST;
    }

    if (*ent).spawnflags & 4 != 0
    /*SWITCH_SHADER*/
    {
        (*ent).s.eFlags |= EF_SHADER_ANIM; // use frame-controlled shader anim
        (*ent).s.frame = 0; // first stage of anim
    }

    if (*ent).spawnflags & 1 != 0 || (*ent).spawnflags & 2 != 0 {
        // so we know it's push/pullable on the client
        (*ent).s.bolt1 = 1;
    }

    G_SpawnInt(
        c"model2scale".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).s.iModelScale,
    );
    if (*ent).s.iModelScale < 0 {
        //NOTE: -1 scale is x -100% (so -3 is 300%)
        (*ent).s.legsFlip = QTRUE; //treat it as a scalar
        (*ent).s.iModelScale = -(*ent).s.iModelScale;
    } else if (*ent).s.iModelScale > 1023 {
        (*ent).s.iModelScale = 1023;
    }

    G_SpawnInt(c"hyperspace".as_ptr(), c"0".as_ptr(), &mut test);
    if test != 0 {
        (*ent).r.svFlags |= SVF_BROADCAST;
        (*ent).s.eFlags2 |= EF2_HYPERSPACE;
    }

    trap::LinkEntity(ent);

    if (*addr_of!(level)).mBSPInstanceDepth != 0 {
        // this means that this guy will never be updated, moved, changed, etc.
        (*ent).s.eFlags = EF_PERMANENT;
    }
}

/// `void func_static_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_mover.c:2011). `func_static` `use`: run its `BSET_USE` behavior, toggle the shader anim
/// frame (if `SWITCH_SHADER`), and fire its targets. No oracle (callback dispatch).
///
/// # Safety
/// `self_`/`activator` must point to valid `gentity_t`s.
pub unsafe extern "C" fn func_static_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);

    if (*self_).spawnflags & 4 != 0
    /*SWITCH_SHADER*/
    {
        (*self_).s.frame = if (*self_).s.frame != 0 { 0 } else { 1 }; // toggle frame
    }
    G_UseTargets(self_, activator);
}

/// `void SP_func_bobbing( gentity_t *ent )` (g_mover.c:2219). Spawn a `func_bobbing`: a brush
/// model that slides sinusoidally (`TR_SINE`) about its origin. `speed` (default 4s) is the bob
/// period, `height` (default 32) the amplitude, `phase` the 0..1 cycle offset; the axis is X
/// (spawnflag 1), Y (2), or Z (default). No oracle (spawn-key reads + trajectory stamp).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning.
pub unsafe fn SP_func_bobbing(ent: *mut gentity_t) {
    let mut height: f32 = 0.0;
    let mut phase: f32 = 0.0;

    G_SpawnFloat(c"speed".as_ptr(), c"4".as_ptr(), &mut (*ent).speed);
    G_SpawnFloat(c"height".as_ptr(), c"32".as_ptr(), &mut height);
    G_SpawnInt(c"dmg".as_ptr(), c"2".as_ptr(), &mut (*ent).damage);
    G_SpawnFloat(c"phase".as_ptr(), c"0".as_ptr(), &mut phase);

    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());
    InitMover(ent);

    VectorCopy(&(*ent).s.origin, &mut (*ent).s.pos.trBase);
    VectorCopy(&(*ent).s.origin, &mut (*ent).r.currentOrigin);

    (*ent).s.pos.trDuration = ((*ent).speed * 1000.0) as c_int;
    (*ent).s.pos.trTime = ((*ent).s.pos.trDuration as f32 * phase) as c_int;
    (*ent).s.pos.trType = TR_SINE;

    // set the axis of bobbing
    if (*ent).spawnflags & 1 != 0 {
        (*ent).s.pos.trDelta[0] = height;
    } else if (*ent).spawnflags & 2 != 0 {
        (*ent).s.pos.trDelta[1] = height;
    } else {
        (*ent).s.pos.trDelta[2] = height;
    }
}

/// `void SP_func_pendulum( gentity_t *ent )` (g_mover.c:2268). Spawn a `func_pendulum`: a brush
/// model that swings sinusoidally (`TR_SINE` on angles) about its origin brush. The swing
/// frequency is the physical pendulum constant `1/(2π)·√(g/3L)` where `L` is the brush's lower
/// z-extent (clamped to ≥8) and `g` is `g_gravity`; `speed` (default 30°) is the swing amplitude
/// and `phase` the 0..1 cycle offset. The frequency math is computed in double like the C (libm
/// `sqrt`, double `M_PI`). No oracle (spawn-key reads + trajectory stamp).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning.
pub unsafe fn SP_func_pendulum(ent: *mut gentity_t) {
    let mut phase: f32 = 0.0;
    let mut speed: f32 = 0.0;

    G_SpawnFloat(c"speed".as_ptr(), c"30".as_ptr(), &mut speed);
    G_SpawnInt(c"dmg".as_ptr(), c"2".as_ptr(), &mut (*ent).damage);
    G_SpawnFloat(c"phase".as_ptr(), c"0".as_ptr(), &mut phase);

    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());

    // find pendulum length
    let mut length: f32 = (*ent).r.mins[2].abs();
    if length < 8.0 {
        length = 8.0;
    }

    let freq: f32 =
        (1.0_f64 / (M_PI as f64 * 2.0) * (((*addr_of!(g_gravity)).value / (3.0 * length)) as f64).sqrt()) as f32;

    (*ent).s.pos.trDuration = (1000.0 / freq) as c_int;

    InitMover(ent);

    VectorCopy(&(*ent).s.origin, &mut (*ent).s.pos.trBase);
    VectorCopy(&(*ent).s.origin, &mut (*ent).r.currentOrigin);

    VectorCopy(&(*ent).s.angles, &mut (*ent).s.apos.trBase);

    (*ent).s.apos.trDuration = (1000.0 / freq) as c_int;
    (*ent).s.apos.trTime = ((*ent).s.apos.trDuration as f32 * phase) as c_int;
    (*ent).s.apos.trType = TR_SINE;
    (*ent).s.apos.trDelta[2] = speed;
}

/// `void func_wait_return_solid( gentity_t *self )` (g_mover.c:2983). `func_usable` think/helper:
/// once a frame, try to make the (toggled-off) usable solid again. It blocks on `CONTENTS_BODY`
/// to test the space; if the `WAIT` flag (16) is clear or nothing is standing in the way
/// ([`G_TestEntityPosition`] is NULL), it rebuilds the brush mover, clears `SVF_NOCLIENT`/
/// `EF_NODRAW`, reinstalls [`func_usable_use`], and fires `target2`. Otherwise it reschedules
/// itself next frame. No oracle (callback dispatch + entity-state mutation).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn func_wait_return_solid(self_: *mut gentity_t) {
    // once a frame, see if it's clear.
    (*self_).clipmask = CONTENTS_BODY;
    if (*self_).spawnflags & 16 == 0 || G_TestEntityPosition(self_).is_null() {
        trap::SetBrushModel(self_, &CStr::from_ptr((*self_).model).to_string_lossy());
        InitMover(self_);
        VectorCopy(&(*self_).s.origin, &mut (*self_).s.pos.trBase);
        VectorCopy(&(*self_).s.origin, &mut (*self_).r.currentOrigin);
        (*self_).r.svFlags &= !SVF_NOCLIENT;
        (*self_).s.eFlags &= !EF_NODRAW;
        (*self_).r#use = Some(func_usable_use);
        (*self_).clipmask = 0;
        if !(*self_).target2.is_null() && *(*self_).target2 != 0 {
            G_UseTargets2(self_, (*self_).activator, (*self_).target2);
        }
        //FIXME: Animations?
    } else {
        (*self_).clipmask = 0;
        (*self_).think = Some(func_wait_return_solid);
        (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
    }
}

/// `void func_usable_think( gentity_t *self )` (g_mover.c:3015). `func_usable` think for the
/// `ALWAYS_ON` (8) wait-delay: re-arm the entity by restoring `SVF_PLAYER_USABLE`, reinstalling
/// [`func_usable_use`], and clearing the think. No oracle (callback dispatch).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn func_usable_think(self_: *mut gentity_t) {
    if (*self_).spawnflags & 8 != 0 {
        (*self_).r.svFlags |= SVF_PLAYER_USABLE; // Replace the usable flag
        (*self_).r#use = Some(func_usable_use);
        (*self_).think = None;
    }
}

/// `qboolean G_EntIsRemovableUsable( int entNum )` (g_mover.c:3025). True iff the slot is a
/// `func_usable` that can be toggled away: not just a shader-animator (`EF_SHADER_ANIM`), not
/// `ALWAYS_ON` (spawnflag 8), and targetable. No oracle (entity predicate).
///
/// # Safety
/// `g_entities` must be initialised and `entNum` a valid slot index.
pub unsafe fn G_EntIsRemovableUsable(entNum: c_int) -> qboolean {
    let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entNum as usize);
    if !(*ent).classname.is_null() && Q_stricmp(c"func_usable".as_ptr(), (*ent).classname) == 0 {
        if (*ent).s.eFlags & EF_SHADER_ANIM == 0
            && (*ent).spawnflags & 8 == 0
            && !(*ent).targetname.is_null()
        {
            // not just a shader-animator and not ALWAYS_ON, so it must be removable somehow
            return QTRUE;
        }
    }
    QFALSE
}

/// `void func_usable_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_mover.c:3038). `func_usable` `use`: toggle on and off. A shader-anim usable advances its
/// frame (wrapping at `genericValue5`) and fires targets; an `ALWAYS_ON` (8) usable strips its
/// own usability, fires targets, and optionally re-arms after `wait`; otherwise it flips solid:
/// when off it becomes solid again via [`func_wait_return_solid`], when on it goes non-solid
/// (`SVF_NOCLIENT`/`EF_NODRAW`) and fires targets. No oracle (callback dispatch + entity-state
/// mutation).
///
/// # Safety
/// `self_`/`activator` must point to valid `gentity_t`s; `level` must be initialised.
pub unsafe extern "C" fn func_usable_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);
    if (*self_).s.eFlags & EF_SHADER_ANIM != 0 {
        // animate shader when used
        (*self_).s.frame += 1; // inc frame
        if (*self_).s.frame > (*self_).genericValue5 {
            // wrap around
            (*self_).s.frame = 0;
        }
        if !(*self_).target.is_null() && *(*self_).target != 0 {
            G_UseTargets(self_, activator);
        }
    } else if (*self_).spawnflags & 8 != 0 {
        // ALWAYS_ON
        // Remove the ability to use the entity directly
        (*self_).r.svFlags &= !SVF_PLAYER_USABLE;
        // also remove ability to call any use func at all!
        (*self_).r#use = None;

        if !(*self_).target.is_null() && *(*self_).target != 0 {
            G_UseTargets(self_, activator);
        }

        if (*self_).wait != 0.0 {
            (*self_).think = Some(func_usable_think);
            (*self_).nextthink = ((*addr_of!(level)).time as f32 + (*self_).wait * 1000.0) as c_int;
        }

        return;
    } else if (*self_).count == 0 {
        // become solid again
        (*self_).count = 1;
        func_wait_return_solid(self_);
    } else {
        (*self_).s.solid = 0;
        (*self_).r.contents = 0;
        (*self_).clipmask = 0;
        (*self_).r.svFlags |= SVF_NOCLIENT;
        (*self_).s.eFlags |= EF_NODRAW;
        (*self_).count = 0;

        if !(*self_).target.is_null() && *(*self_).target != 0 {
            G_UseTargets(self_, activator);
        }
        (*self_).think = None;
        (*self_).nextthink = -1;
    }
}

/// `void func_usable_pain( gentity_t *self, gentity_t *attacker, int damage )` (g_mover.c:3096).
/// `func_usable` `pain`: a shootable usable that is hit just triggers itself ([`GlobalUse`]). No
/// oracle (callback dispatch).
///
/// # Safety
/// `self_`/`attacker` must point to valid `gentity_t`s.
pub unsafe extern "C" fn func_usable_pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
) {
    GlobalUse(self_, attacker, attacker);
}

/// `void func_usable_die( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage,
/// int mod )` (g_mover.c:3101). `func_usable` `die`: a destroyed usable stops taking damage and
/// triggers itself ([`GlobalUse`]). No oracle (callback dispatch).
///
/// # Safety
/// `self_`/`inflictor`/`attacker` must point to valid `gentity_t`s.
pub unsafe extern "C" fn func_usable_die(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _mod_: c_int,
) {
    (*self_).takedamage = QFALSE;
    GlobalUse(self_, inflictor, attacker);
}

/// `void SP_func_usable( gentity_t *self )` (g_mover.c:3128). Spawn a `func_usable`: a toggleable
/// brush model (conditional wall / shader-animator). Seats it at its origin as a zero-travel
/// mover, reads `endframe` into `genericValue5`, resolves the optional `model2` (skipping
/// unsupported `.glm`), and installs [`func_usable_use`]. `STARTOFF` (1) spawns it non-solid and
/// hidden; a `health` value makes it shootable ([`func_usable_die`]/[`func_usable_pain`]); a
/// positive `endframe` switches it into frame-driven shader-anim mode. No oracle (spawn-key reads
/// + entity-state mutation).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` during entity spawning.
pub unsafe fn SP_func_usable(self_: *mut gentity_t) {
    trap::SetBrushModel(self_, &CStr::from_ptr((*self_).model).to_string_lossy());
    InitMover(self_);
    VectorCopy(&(*self_).s.origin, &mut (*self_).s.pos.trBase);
    VectorCopy(&(*self_).s.origin, &mut (*self_).r.currentOrigin);
    VectorCopy(&(*self_).s.origin, &mut (*self_).pos1);

    G_SpawnInt(
        c"endframe".as_ptr(),
        c"0".as_ptr(),
        &mut (*self_).genericValue5,
    );

    if !(*self_).model2.is_null() && *(*self_).model2 != 0 {
        if CStr::from_ptr((*self_).model2)
            .to_string_lossy()
            .contains(".glm")
        {
            //for now, not supported in MP.
            (*self_).s.modelindex2 = 0;
        } else {
            (*self_).s.modelindex2 = G_ModelIndex(&CStr::from_ptr((*self_).model2).to_string_lossy());
        }
    }

    (*self_).count = 1;
    if (*self_).spawnflags & 1 != 0 {
        (*self_).s.solid = 0;
        (*self_).r.contents = 0;
        (*self_).clipmask = 0;
        (*self_).r.svFlags |= SVF_NOCLIENT;
        (*self_).s.eFlags |= EF_NODRAW;
        (*self_).count = 0;
    }

    (*self_).r#use = Some(func_usable_use);

    if (*self_).health != 0 {
        (*self_).takedamage = QTRUE;
        (*self_).die = Some(func_usable_die);
        (*self_).pain = Some(func_usable_pain);
    }

    if (*self_).genericValue5 > 0 {
        (*self_).s.frame = 0;
        (*self_).s.eFlags |= EF_SHADER_ANIM;
        (*self_).s.time = (*self_).genericValue5 + 1;
    }

    trap::LinkEntity(self_);
}

/// `void SP_path_corner( gentity_t *self )` (g_mover.c:1865). Spawn a `path_corner`: a waypoint
/// for `func_train`s. It carries no brush or model — it just needs a `targetname` so trains can
/// chain to it; a path_corner without one is useless, so it warns and frees itself. Path corners
/// are never linked into the world. No oracle (spawn-key check + free).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` during entity spawning.
pub unsafe fn SP_path_corner(self_: *mut gentity_t) {
    if (*self_).targetname.is_null() {
        G_Printf(&format!(
            "path_corner with no targetname at {}\n",
            CStr::from_ptr(vtos(&(*self_).s.origin)).to_string_lossy()
        ));
        G_FreeEntity(self_);
        return;
    }
    // path corners don't need to be linked in
}

// TRAIN spawnflags (g_mover.c:1708).
const TRAIN_START_ON: c_int = 1;
#[allow(dead_code)] // (g_mover.c:1709) verbatim; train TOGGLE handling not yet ported
const TRAIN_TOGGLE: c_int = 2;
const TRAIN_BLOCK_STOPS: c_int = 4;

/// `void Reached_Train( gentity_t *ent )` (g_mover.c:1731). A `func_train` reached its current
/// path_corner — advance to the next leg. Stops if there is no next corner; otherwise fires the
/// reached corner's targets, retargets `pos1`/`pos2` to the next segment, picks the corner's
/// `speed` (falling back to the train's, clamped to `>= 1`), computes the leg duration and starts
/// the mover ([`SetMoverState`] `MOVER_1TO2`) with the end door-sound. A `wait` on the corner
/// holds the train stationary for that long via [`Think_BeginMoving`]; otherwise the move loop
/// sound plays. No oracle (entity-graph walk + trajectory/state mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn Reached_Train(ent: *mut gentity_t) {
    // copy the apropriate values
    let next = (*ent).nextTrain;
    if next.is_null() || (*next).nextTrain.is_null() {
        return; // just stop
    }

    // fire all other targets
    G_UseTargets(next, null_mut());

    // set the new trajectory
    (*ent).nextTrain = (*next).nextTrain;
    VectorCopy(&(*next).s.origin, &mut (*ent).pos1);
    VectorCopy(&(*(*next).nextTrain).s.origin, &mut (*ent).pos2);

    // if the path_corner has a speed, use that
    let mut speed = if (*next).speed != 0.0 {
        (*next).speed
    } else {
        // otherwise use the train's speed
        (*ent).speed
    };
    if speed < 1.0 {
        speed = 1.0;
    }

    // calculate duration
    let mut move_: vec3_t = [0.0; 3];
    VectorSubtract(&(*ent).pos2, &(*ent).pos1, &mut move_);
    let length = VectorLength(&move_);

    (*ent).s.pos.trDuration = (length * 1000.0 / speed) as c_int;

    // start it going
    SetMoverState(ent, MOVER_1TO2, (*addr_of!(level)).time);

    G_PlayDoorSound(ent, BMS_END);

    // if there is a "wait" value on the target, don't start moving yet
    if (*next).wait != 0.0 {
        (*ent).s.loopSound = 0;
        (*ent).s.loopIsSoundset = QFALSE;
        (*ent).nextthink = ((*addr_of!(level)).time as f32 + (*next).wait * 1000.0) as c_int;
        (*ent).think = Some(Think_BeginMoving);
        (*ent).s.pos.trType = TR_STATIONARY;
    } else {
        G_PlayDoorLoopSound(ent);
    }
}

/// `void Think_SetupTrainTargets( gentity_t *ent )` (g_mover.c:1794). Link all the path corners
/// together: resolve `ent->target` to the first corner, then walk each corner's `target` to the
/// next `path_corner` (skipping non-corner targets that also get fired), stitching the
/// `nextTrain` chain. Warns and bails if the first target is unfound. Then either kicks the train
/// into motion ([`Reached_Train`]) when it has no `targetname` or the `START_ON` flag is set, or
/// parks it at its spawn origin ([`G_SetOrigin`]). No oracle (entity-graph walk + state mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the entity table must be populated.
pub unsafe extern "C" fn Think_SetupTrainTargets(ent: *mut gentity_t) {
    (*ent).nextTrain = G_Find(null_mut(), offset_of!(gentity_t, targetname), (*ent).target);
    if (*ent).nextTrain.is_null() {
        Com_Printf(&format!(
            "func_train at {} with an unfound target\n",
            CStr::from_ptr(vtos(&(*ent).r.absmin)).to_string_lossy()
        ));
        //Free me?`
        return;
    }

    //FIXME: this can go into an infinite loop if last path_corner doesn't link to first
    //path_corner, like so:
    // t1---->t2---->t3
    //         ^      |
    //          \_____|
    let mut start: *mut gentity_t = null_mut();
    let mut path = (*ent).nextTrain;
    while path != start {
        if start.is_null() {
            start = path;
        }

        if (*path).target.is_null() {
            //			gi.Printf( "Train corner at %s without a target\n",
            //				vtos(path->s.origin) );
            //end of path
            break;
        }

        // find a path_corner among the targets
        // there may also be other targets that get fired when the corner
        // is reached
        let mut next: *mut gentity_t = null_mut();
        loop {
            next = G_Find(next, offset_of!(gentity_t, targetname), (*path).target);
            if next.is_null() {
                //				gi.Printf( "Train corner at %s without a target path_corner\n",
                //					vtos(path->s.origin) );
                //end of path
                break;
            }
            if CStr::from_ptr((*next).classname) == c"path_corner" {
                break;
            }
        }

        if !next.is_null() {
            (*path).nextTrain = next;
        } else {
            break;
        }

        path = next;
    }

    if (*ent).targetname.is_null() || (*ent).spawnflags & TRAIN_START_ON != 0
    /*start on*/
    {
        // start the train moving from the first corner
        Reached_Train(ent);
    } else {
        G_SetOrigin(ent, &(*ent).s.origin);
    }
}

/// `void SP_func_train( gentity_t *self )` (g_mover.c:1889). Spawn a `func_train`: a mover that
/// rides between `path_corner` waypoints. Clears its angles, applies the crush damage default (`2`,
/// or `0` under `BLOCK_STOPS`), defaults `speed` to `100`, and requires a `target` (warns + frees
/// otherwise). Sets the brush model, runs [`InitMover`], installs [`Reached_Train`] as the
/// endpoint callback, and defers [`Think_SetupTrainTargets`] one frame so the corners have spawned.
/// No oracle (spawn-key reads + entity-state mutation).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` during entity spawning.
pub unsafe fn SP_func_train(self_: *mut gentity_t) {
    VectorClear(&mut (*self_).s.angles);

    if (*self_).spawnflags & TRAIN_BLOCK_STOPS != 0 {
        (*self_).damage = 0;
    } else if (*self_).damage == 0 {
        (*self_).damage = 2;
    }

    if (*self_).speed == 0.0 {
        (*self_).speed = 100.0;
    }

    if (*self_).target.is_null() {
        G_Printf(&format!(
            "func_train without a target at {}\n",
            CStr::from_ptr(vtos(&(*self_).r.absmin)).to_string_lossy()
        ));
        G_FreeEntity(self_);
        return;
    }

    trap::SetBrushModel(self_, &CStr::from_ptr((*self_).model).to_string_lossy());
    InitMover(self_);

    (*self_).reached = Some(Reached_Train);

    // start trains on the second frame, to make sure their targets have had
    // a chance to spawn
    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
    (*self_).think = Some(Think_SetupTrainTargets);
}

/// `void use_wall( gentity_t *ent, gentity_t *other, gentity_t *activator )` (g_mover.c:3203).
/// `func_wall` `use`: toggle the wall in or out of existence. When absent (no `CONTENTS_SOLID`)
/// it reappears — visible, solid, and (unless `START_OFF`, flag 1) closing its area portal;
/// otherwise it vanishes — non-solid, `SVF_NOCLIENT`/`EF_NODRAW`, opening its portal. No oracle
/// (callback dispatch + entity-state mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn use_wall(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    G_ActivateBehavior(ent, BSET_USE);

    // Not there so make it there
    if (*ent).r.contents & CONTENTS_SOLID == 0 {
        (*ent).r.svFlags &= !SVF_NOCLIENT;
        (*ent).s.eFlags &= !EF_NODRAW;
        (*ent).r.contents = CONTENTS_SOLID;
        if (*ent).spawnflags & 1 == 0 {
            // START_OFF doesn't effect area portals
            trap::AdjustAreaPortalState(ent, QFALSE);
        }
    }
    // Make it go away
    else {
        (*ent).r.contents = 0;
        (*ent).r.svFlags |= SVF_NOCLIENT;
        (*ent).s.eFlags |= EF_NODRAW;
        if (*ent).spawnflags & 1 == 0 {
            // START_OFF doesn't effect area portals
            trap::AdjustAreaPortalState(ent, QTRUE);
        }
    }
}

/// `#define FUNC_WALL_OFF 1` (g_mover.c:3231) — the `START_OFF` spawnflag for `func_wall`.
const FUNC_WALL_OFF: c_int = 1;

/// `void SP_func_wall( gentity_t *ent )` (g_mover.c:3244). Spawn a `func_wall`: a brush model that
/// sits there until used (conditional wall). Seats it at its origin as a zero-travel mover and
/// installs [`use_wall`]; `START_OFF` (`FUNC_WALL_OFF`) spawns it non-solid and hidden. No oracle
/// (spawn-key reads + entity-state mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning.
pub unsafe fn SP_func_wall(ent: *mut gentity_t) {
    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());

    VectorCopy(&(*ent).s.origin, &mut (*ent).pos1);
    VectorCopy(&(*ent).s.origin, &mut (*ent).pos2);

    InitMover(ent);
    VectorCopy(&(*ent).s.origin, &mut (*ent).s.pos.trBase);
    VectorCopy(&(*ent).s.origin, &mut (*ent).r.currentOrigin);

    // it must be START_OFF
    if (*ent).spawnflags & FUNC_WALL_OFF != 0 {
        (*ent).r.contents = 0;
        (*ent).r.svFlags |= SVF_NOCLIENT;
        (*ent).s.eFlags |= EF_NODRAW;
    }

    (*ent).r#use = Some(use_wall);

    trap::LinkEntity(ent);
}

/// `void SP_func_rotating (gentity_t *ent)` (g_mover.c:2123). Spawn a `func_rotating`: a brush that
/// spins continuously. If it has `health` it is also breakable (delegates to [`SP_func_breakable`]
/// with spawnflags temporarily cleared); otherwise it's set up as a plain mover. Either way it then
/// reads `model2scale`, picks its rotation axis from `spinangles` or the axis spawnflags, sets a
/// `TR_LINEAR` angular trajectory, defaults its crush `damage`, and optionally registers as a radar
/// object. No oracle (spawn-key reads + entity-state mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning; the spawn-var system must be
/// active.
pub unsafe fn SP_func_rotating(ent: *mut gentity_t) {
    let mut spinangles: vec3_t = [0.0; 3];
    if (*ent).health != 0 {
        let sav_spawnflags = (*ent).spawnflags;
        (*ent).spawnflags = 0;
        SP_func_breakable(ent);
        (*ent).spawnflags = sav_spawnflags;
    } else {
        trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());
        InitMover(ent);

        VectorCopy(&(*ent).s.origin, &mut (*ent).s.pos.trBase);
        VectorCopy(&(*ent).s.pos.trBase, &mut (*ent).r.currentOrigin);
        VectorCopy(&(*ent).s.apos.trBase, &mut (*ent).r.currentAngles);

        trap::LinkEntity(ent);
    }

    G_SpawnInt(
        c"model2scale".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).s.iModelScale,
    );
    if (*ent).s.iModelScale < 0 {
        //NOTE: -1 scale is x -100% (so -3 is 300%)
        (*ent).s.legsFlip = QTRUE; //treat it as a scalar
        (*ent).s.iModelScale = -(*ent).s.iModelScale;
    } else if (*ent).s.iModelScale > 1023 {
        (*ent).s.iModelScale = 1023;
    }

    if G_SpawnVector(
        c"spinangles".as_ptr(),
        c"0 0 0".as_ptr(),
        spinangles.as_mut_ptr(),
    ) != QFALSE
    {
        (*ent).speed = VectorLength(&spinangles);
        // set the axis of rotation
        VectorCopy(&spinangles, &mut (*ent).s.apos.trDelta);
    } else {
        if (*ent).speed == 0.0 {
            (*ent).speed = 100.0;
        }
        // set the axis of rotation
        if (*ent).spawnflags & 4 != 0 {
            (*ent).s.apos.trDelta[2] = (*ent).speed;
        } else if (*ent).spawnflags & 8 != 0 {
            (*ent).s.apos.trDelta[0] = (*ent).speed;
        } else {
            (*ent).s.apos.trDelta[1] = (*ent).speed;
        }
    }
    (*ent).s.apos.trType = TR_LINEAR;

    if (*ent).damage == 0 {
        if (*ent).spawnflags & 16 != 0 {
            //IMPACT
            (*ent).damage = 10000;
        } else {
            (*ent).damage = 2;
        }
    }
    if (*ent).spawnflags & 2 != 0 {
        //RADAR
        //show up on Radar at close range and play impact sound when close...?  Range based on my size
        (*ent).s.speed = Distance(&(*ent).r.absmin, &(*ent).r.absmax) * 0.5;
        (*ent).s.eFlags |= EF_RADAROBJECT;
    }
}

/* ===========================================================================
   BREAKABLE BRUSH / GLASS  (g_mover.c:2306+)
=========================================================================== */

/// `void G_MiscModelExplosion( vec3_t mins, vec3_t maxs, int size, material_t chunkType )`
/// (g_mover.c:2351). Spawn an `EV_MISC_MODEL_EXP` temp-entity at the centre of the given bounds,
/// stashing the bounds (`maxs`→`origin2`, `mins`→`angles2`), the explosion `size` in `s.time`, and
/// the chunk material in `s.eventParm` for the client effect to read. No oracle (spawns a temp
/// entity via the `g_entities` global).
///
/// # Safety
/// `g_entities`/`level` must be initialised so [`G_TempEntity`] can allocate.
pub unsafe fn G_MiscModelExplosion(mins: &vec3_t, maxs: &vec3_t, size: c_int, chunkType: material_t) {
    let mut mid: vec3_t = [0.0; 3];

    VectorAdd(mins, maxs, &mut mid);
    let mid_sum = mid;
    VectorScale(&mid_sum, 0.5, &mut mid);

    let te = G_TempEntity(&mid, EV_MISC_MODEL_EXP);

    VectorCopy(maxs, &mut (*te).s.origin2);
    VectorCopy(mins, &mut (*te).s.angles2);
    (*te).s.time = size;
    (*te).s.eventParm = chunkType;
}

/// `void G_Chunks( int owner, vec3_t origin, const vec3_t normal, const vec3_t mins,
/// const vec3_t maxs, float speed, int numChunks, material_t chunkType, int customChunk,
/// float baseScale )` (g_mover.c:2367). Spawn an `EV_DEBRIS` temp-entity, cramming every chunk
/// parameter (owner, origin, spray normal, bounds, speed, count, material, custom model, base
/// scale) into the event entity's state for the client-side debris spawner to unpack. No oracle
/// (spawns a temp entity via the `g_entities` global).
///
/// # Safety
/// `g_entities`/`level` must be initialised so [`G_TempEntity`] can allocate.
#[allow(clippy::too_many_arguments)]
pub unsafe fn G_Chunks(
    owner: c_int,
    origin: &vec3_t,
    normal: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    speed: f32,
    numChunks: c_int,
    chunkType: material_t,
    customChunk: c_int,
    baseScale: f32,
) {
    let te = G_TempEntity(origin, EV_DEBRIS);

    //Now it's time to cram everything horribly into the entitystate of an event entity.
    (*te).s.owner = owner;
    VectorCopy(origin, &mut (*te).s.origin);
    VectorCopy(normal, &mut (*te).s.angles);
    VectorCopy(maxs, &mut (*te).s.origin2);
    VectorCopy(mins, &mut (*te).s.angles2);
    (*te).s.speed = speed;
    (*te).s.eventParm = numChunks;
    (*te).s.trickedentindex = chunkType;
    (*te).s.modelindex = customChunk;
    (*te).s.apos.trBase[0] = baseScale;
}

/// `void funcBBrushDieGo (gentity_t *self)` (g_mover.c:2386). The breakable brush's actual
/// destruction: detonate any missiles stuck to it, go non-solid, fire its targets, then spew an
/// explosion effect, an optional custom effect, optional splash damage + sound, and a volume of
/// debris chunks scaled from its bounds. Finally seals its area portal and schedules itself to be
/// freed next frame. No oracle (uses `random`, spawns temp entities, dispatches damage callbacks).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `g_entities`/`level` must be initialised.
pub unsafe extern "C" fn funcBBrushDieGo(self_: *mut gentity_t) {
    let mut org: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let attacker = (*self_).enemy;
    let mut scale: f32;
    let mut numChunks: c_int;
    let mut size: c_int = 0;
    let chunkType: material_t = (*self_).material;

    // if a missile is stuck to us, blow it up so we don't look dumb
    for i in 0..MAX_GENTITIES {
        let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if (*ent).s.groundEntityNum == (*self_).s.number
            && (*ent).s.eFlags & EF_MISSILE_STICK != 0
        {
            G_Damage(ent, self_, self_, null_mut(), null_mut(), 99999, 0, MOD_CRUSH); //?? MOD?
        }
    }

    //So chunks don't get stuck inside me
    (*self_).s.solid = 0;
    (*self_).r.contents = 0;
    (*self_).clipmask = 0;
    trap::LinkEntity(self_);

    VectorSet(&mut up, 0.0, 0.0, 1.0);

    if !(*self_).target.is_null() && !attacker.is_null() {
        G_UseTargets(self_, attacker);
    }

    VectorSubtract(&(*self_).r.absmax, &(*self_).r.absmin, &mut org); // size

    numChunks = (random() * 6.0 + 18.0) as c_int;

    // This formula really has no logical basis other than the fact that it seemed to be the closest to yielding the results that I wanted.
    // Volume is length * width * height...then break that volume down based on how many chunks we have
    let prod = org[0] * org[1] * org[2];
    scale = ((prod as f64).sqrt().sqrt() * 1.75_f64) as f32;

    if scale > 48.0 {
        size = 2;
    } else if scale > 24.0 {
        size = 1;
    }

    scale /= numChunks as f32;

    if (*self_).radius > 0.0 {
        // designer wants to scale number of chunks, helpful because the above scale code is far from perfect
        //	I do this after the scale calculation because it seems that the chunk size generally seems to be very close, it's just the number of chunks is a bit weak
        numChunks = (numChunks as f32 * (*self_).radius) as c_int;
    }

    let org_ma = org;
    VectorMA(&(*self_).r.absmin, 0.5, &org_ma, &mut org);
    VectorAdd(&(*self_).r.absmin, &(*self_).r.absmax, &mut org);
    let org_half = org;
    VectorScale(&org_half, 0.5, &mut org);

    if !attacker.is_null() && !(*attacker).client.is_null() {
        VectorSubtract(&org, &(*attacker).r.currentOrigin, &mut dir);
        VectorNormalize(&mut dir);
    } else {
        VectorCopy(&up, &mut dir);
    }

    if (*self_).spawnflags & 2048 == 0 {
        // NO_EXPLOSION
        // we are allowed to explode
        G_MiscModelExplosion(&(*self_).r.absmin, &(*self_).r.absmax, size, chunkType);
    }

    if (*self_).genericValue15 != 0 {
        //a custom effect to play
        let mut ang: vec3_t = [0.0; 3];
        VectorSet(&mut ang, 0.0, 1.0, 0.0);
        G_PlayEffectID((*self_).genericValue15, &org, &ang);
    }

    if (*self_).splashDamage > 0 && (*self_).splashRadius > 0 {
        //explode
        G_RadiusDamage(
            &org,
            self_,
            (*self_).splashDamage as f32,
            (*self_).splashRadius as f32,
            self_,
            null_mut(),
            MOD_UNKNOWN,
        );

        let te = G_TempEntity(&org, EV_GENERAL_SOUND);
        (*te).s.eventParm = G_SoundIndex("sound/weapons/explosions/cargoexplode.wav");
    }

    //FIXME: base numChunks off size?
    G_Chunks(
        (*self_).s.number,
        &org,
        &dir,
        &(*self_).r.absmin,
        &(*self_).r.absmax,
        300.0,
        numChunks,
        chunkType,
        0,
        scale * (*self_).mass,
    );

    trap::AdjustAreaPortalState(self_, QTRUE);
    (*self_).think = Some(G_FreeEntity);
    (*self_).nextthink = (*addr_of!(level)).time + 50;
    //G_FreeEntity( self );
}

/// `void funcBBrushDie (gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage,
/// int mod)` (g_mover.c:2488). The breakable brush's `die`: stop taking damage (so a chain
/// reaction can't run away), remember the attacker, then either schedule [`funcBBrushDieGo`] after
/// the entity's `delay` (seconds) or detonate immediately. No oracle (callback dispatch +
/// entity-state mutation).
///
/// # Safety
/// `self_`/`attacker` must point to valid `gentity_t`s; `level` must be initialised.
pub unsafe extern "C" fn funcBBrushDie(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _mod_: c_int,
) {
    (*self_).takedamage = QFALSE; //stop chain reaction runaway loops

    (*self_).enemy = attacker;

    if (*self_).delay != 0 {
        (*self_).think = Some(funcBBrushDieGo);
        (*self_).nextthink =
            (*addr_of!(level)).time + ((*self_).delay as f32 * 1000.0).floor() as c_int;
        return;
    }

    funcBBrushDieGo(self_);
}

/// `void funcBBrushUse (gentity_t *self, gentity_t *other, gentity_t *activator)` (g_mover.c:2504).
/// The breakable brush's `use`: fire its `BSET_USE` behaviour set, then — if `USE_NOT_BREAK` (64)
/// is set — merely fire its targets, otherwise destroy it via [`funcBBrushDie`]. No oracle
/// (callback dispatch).
///
/// # Safety
/// `self_`/`other`/`activator` must point to valid `gentity_t`s.
pub unsafe extern "C" fn funcBBrushUse(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);
    if (*self_).spawnflags & 64 != 0 {
        //Using it doesn't break it, makes it use it's targets
        if !(*self_).target.is_null() && *(*self_).target != 0 {
            G_UseTargets(self_, activator);
        }
    } else {
        funcBBrushDie(self_, other, activator, (*self_).health, MOD_UNKNOWN);
    }
}

/// `void funcBBrushPain(gentity_t *self, gentity_t *attacker, int damage)` (g_mover.c:2520). The
/// breakable brush's `pain`: debounced by `painDebounceTime`, it fires its `paintarget` (using the
/// stored activator, or the attacker if it's a client), runs its `BSET_PAIN` behaviour set, and —
/// for the stone materials — sprays a few debris chunks. A `wait` of -1 makes pain fire only once.
/// No oracle (callback dispatch + `Q_irand`).
///
/// # Safety
/// `self_`/`attacker` must point to valid `gentity_t`s; `level` must be initialised.
pub unsafe extern "C" fn funcBBrushPain(self_: *mut gentity_t, attacker: *mut gentity_t, _damage: c_int) {
    if (*self_).painDebounceTime > (*addr_of!(level)).time {
        return;
    }

    if !(*self_).paintarget.is_null() && *(*self_).paintarget != 0 {
        if (*self_).activator.is_null() {
            if !attacker.is_null() && (*attacker).inuse != QFALSE && !(*attacker).client.is_null() {
                G_UseTargets2(self_, attacker, (*self_).paintarget);
            }
        } else {
            G_UseTargets2(self_, (*self_).activator, (*self_).paintarget);
        }
    }

    G_ActivateBehavior(self_, BSET_PAIN);

    if (*self_).material == MAT_DRK_STONE
        || (*self_).material == MAT_LT_STONE
        || (*self_).material == MAT_GREY_STONE
        || (*self_).material == MAT_SNOWY_ROCK
    {
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let scale: f32;
        let mut numChunks: c_int;
        VectorSubtract(&(*self_).r.absmax, &(*self_).r.absmin, &mut org); // size
        // This formula really has no logical basis other than the fact that it seemed to be the closest to yielding the results that I wanted.
        // Volume is length * width * height...then break that volume down based on how many chunks we have
        scale = VectorLength(&org) / 100.0;
        let org_ma = org;
        VectorMA(&(*self_).r.absmin, 0.5, &org_ma, &mut org);
        VectorAdd(&(*self_).r.absmin, &(*self_).r.absmax, &mut org);
        let org_half = org;
        VectorScale(&org_half, 0.5, &mut org);
        if !attacker.is_null() && !(*attacker).client.is_null() {
            VectorSubtract(&(*attacker).r.currentOrigin, &org, &mut dir);
            VectorNormalize(&mut dir);
        } else {
            VectorSet(&mut dir, 0.0, 0.0, 1.0);
        }
        numChunks = Q_irand(1, 3);
        if (*self_).radius > 0.0 {
            // designer wants to scale number of chunks, helpful because the above scale code is far from perfect
            //	I do this after the scale calculation because it seems that the chunk size generally seems to be very close, it's just the number of chunks is a bit weak
            numChunks = (numChunks as f32 * (*self_).radius).ceil() as c_int;
        }
        G_Chunks(
            (*self_).s.number,
            &org,
            &dir,
            &(*self_).r.absmin,
            &(*self_).r.absmax,
            300.0,
            numChunks,
            (*self_).material,
            0,
            scale * (*self_).mass,
        );
    }

    if (*self_).wait == -1.0 {
        (*self_).pain = None;
        return;
    }

    (*self_).painDebounceTime = ((*addr_of!(level)).time as f32 + (*self_).wait) as c_int;
}

/// `void funcBBrushTouch( gentity_t *ent, gentity_t *other, trace_t *trace )` (g_mover.c:2651). The
/// breakable brush's `touch`: empty in the retail game (kept as a verbatim no-op so the `touch`
/// hook is installed). No oracle (empty).
///
/// # Safety
/// Arguments are unused; the function is inert.
pub unsafe extern "C" fn funcBBrushTouch(
    _ent: *mut gentity_t,
    _other: *mut gentity_t,
    _trace: *mut trace_t,
) {
}

/// `static void CacheChunkEffects( material_t material )` (g_mover.c:2311). Precache the debris
/// effect(s) for a breakable's material so they're registered before the brush can break. No
/// oracle (registers effect indices via the engine).
///
/// # Safety
/// Must be called during spawn so [`G_EffectIndex`] can register effects.
unsafe fn CacheChunkEffects(material: material_t) {
    match material {
        MAT_GLASS => {
            G_EffectIndex("chunks/glassbreak");
        }
        MAT_GLASS_METAL => {
            G_EffectIndex("chunks/glassbreak");
            G_EffectIndex("chunks/metalexplode");
        }
        MAT_ELECTRICAL | MAT_ELEC_METAL => {
            G_EffectIndex("chunks/sparkexplode");
        }
        MAT_METAL | MAT_METAL2 | MAT_METAL3 | MAT_CRATE1 | MAT_CRATE2 => {
            G_EffectIndex("chunks/metalexplode");
        }
        MAT_GRATE1 => {
            G_EffectIndex("chunks/grateexplode");
        }
        MAT_DRK_STONE | MAT_LT_STONE | MAT_GREY_STONE | MAT_WHITE_METAL | MAT_SNOWY_ROCK => {
            G_EffectIndex("chunks/rockbreaklg");
            G_EffectIndex("chunks/rockbreakmed");
        }
        MAT_ROPE => {
            G_EffectIndex("chunks/ropebreak");
            //		G_SoundIndex(); // FIXME: give it a sound
        }
        _ => {}
    }
}

/// `static void InitBBrush ( gentity_t *ent )` (g_mover.c:2587). Spawn-time setup shared by all
/// breakable brushes: seat it at its origin, set its brush model, install [`funcBBrushDie`], flag
/// it `FL_BBRUSH`, resolve the optional `model2`, pack `light`/`color` into `constantLight`, honour
/// the player-usable (128) spawnflag, and link it as a stationary `ET_MOVER`. No oracle (spawn-key
/// reads + entity-state mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning.
unsafe fn InitBBrush(ent: *mut gentity_t) {
    let mut light: f32 = 0.0;
    let mut color: vec3_t = [0.0; 3];

    VectorCopy(&(*ent).s.origin, &mut (*ent).pos1);

    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());

    (*ent).die = Some(funcBBrushDie);

    (*ent).flags |= FL_BBRUSH;

    //This doesn't have to be an svFlag, can just be a flag.
    //And it might not be needed anyway.
    //ent->r.svFlags |= SVF_BBRUSH;

    // if the "model2" key is set, use a seperate model
    // for drawing, but clip against the brushes
    if !(*ent).model2.is_null() && *(*ent).model2 != 0 {
        (*ent).s.modelindex2 = G_ModelIndex(&CStr::from_ptr((*ent).model2).to_string_lossy());
    }

    // if the "color" or "light" keys are set, setup constantLight
    let lightSet = G_SpawnFloat(c"light".as_ptr(), c"100".as_ptr(), &mut light);
    let colorSet = G_SpawnVector(c"color".as_ptr(), c"1 1 1".as_ptr(), color.as_mut_ptr());
    if lightSet != QFALSE || colorSet != QFALSE {
        let mut r = (color[0] * 255.0) as c_int;
        if r > 255 {
            r = 255;
        }
        let mut g = (color[1] * 255.0) as c_int;
        if g > 255 {
            g = 255;
        }
        let mut b = (color[2] * 255.0) as c_int;
        if b > 255 {
            b = 255;
        }
        let mut i = (light / 4.0) as c_int;
        if i > 255 {
            i = 255;
        }
        (*ent).s.constantLight = r | (g << 8) | (b << 16) | (i << 24);
    }

    if (*ent).spawnflags & 128 != 0 {
        //Can be used by the player's BUTTON_USE
        (*ent).r.svFlags |= SVF_PLAYER_USABLE;
    }

    (*ent).s.eType = ET_MOVER;
    trap::LinkEntity(ent);

    (*ent).s.pos.trType = TR_STATIONARY;
    VectorCopy(&(*ent).pos1, &mut (*ent).s.pos.trBase);
}

/// `void SP_func_breakable( gentity_t *self )` (g_mover.c:2719). Spawn a `func_breakable`: a brush
/// that can be shot or used to destruction. Reads the death effect (`playfx`), default health,
/// optional HUD health bar (`showhealth`), saber-only/heavy-weapon-only damage flags, the chunk
/// material/radius/mass and splash damage, precaches its effects, installs the
/// use/pain/touch/`die` callbacks, resolves the Siege `teamnodmg`, then runs [`InitBBrush`]. No
/// oracle (spawn-key reads + entity-state mutation).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` during entity spawning; the spawn-var system must be
/// active.
pub unsafe fn SP_func_breakable(self_: *mut gentity_t) {
    let mut t: c_int = 0;
    let mut s: *mut c_char = null_mut();

    G_SpawnString(c"playfx".as_ptr(), c"".as_ptr(), &mut s);

    if !s.is_null() && *s != 0 {
        //should we play a special death effect?
        (*self_).genericValue15 = G_EffectIndex(&CStr::from_ptr(s).to_string_lossy());
    } else {
        (*self_).genericValue15 = 0;
    }

    if (*self_).spawnflags & 1 == 0 {
        if (*self_).health == 0 {
            (*self_).health = 10;
        }
    }

    G_SpawnInt(c"showhealth".as_ptr(), c"0".as_ptr(), &mut t);

    if t != 0 {
        //a non-0 maxhealth value will mean we want to show the health on the hud
        (*self_).maxHealth = (*self_).health;
        G_ScaleNetHealth(self_);
    }

    //NOTE: g_spawn.c does this automatically now
    //G_SpawnInt( "teamowner", "0", &t );
    //self->s.teamowner = t;

    if (*self_).spawnflags & 16 != 0 {
        // saber only
        (*self_).flags |= FL_DMG_BY_SABER_ONLY;
    } else if (*self_).spawnflags & 32 != 0 {
        // heavy weap
        (*self_).flags |= FL_DMG_BY_HEAVY_WEAP_ONLY;
    }

    if (*self_).health != 0 {
        (*self_).takedamage = QTRUE;
    }

    G_SoundIndex("sound/weapons/explosions/cargoexplode.wav"); //precaching
    G_SpawnFloat(c"radius".as_ptr(), c"1".as_ptr(), &mut (*self_).radius); // used to scale chunk code if desired by a designer
    G_SpawnInt(c"material".as_ptr(), c"0".as_ptr(), &mut (*self_).material);

    G_SpawnInt(
        c"splashDamage".as_ptr(),
        c"0".as_ptr(),
        &mut (*self_).splashDamage,
    );
    G_SpawnInt(
        c"splashRadius".as_ptr(),
        c"0".as_ptr(),
        &mut (*self_).splashRadius,
    );

    CacheChunkEffects((*self_).material);

    (*self_).r#use = Some(funcBBrushUse);

    //if ( self->paintarget )
    {
        (*self_).pain = Some(funcBBrushPain);
    }

    (*self_).touch = Some(funcBBrushTouch);

    /*
    if ( self->team && self->team[0] )
    {
        self->alliedTeam = TranslateTeamName( self->team );
        if(self->alliedTeam == TEAM_FREE)
        {
            G_Error("team name %s not recognized\n", self->team);
        }
    }
    */
    if !(*self_).team.is_null()
        && *(*self_).team != 0
        && (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*self_).teamnodmg == 0
    {
        (*self_).teamnodmg = atoi((*self_).team);
    }
    (*self_).team = null_mut();
    if (*self_).model.is_null() {
        G_Error("func_breakable with NULL model\n");
    }
    InitBBrush(self_);

    if (*self_).radius == 0.0 {
        //numchunks multiplier
        (*self_).radius = 1.0;
    }
    if (*self_).mass == 0.0 {
        //chunksize multiplier
        (*self_).mass = 1.0;
    }
    (*self_).genericValue4 = 1; //so damage sys knows it's a bbrush
}

/// `qboolean G_EntIsBreakable( int entityNum )` (g_mover.c:2819). Classify whether an entity is a
/// breakable: glass brushes (`SVF_GLASS_BRUSH`) and the `func_breakable`/`misc_model_breakable`/
/// `misc_maglock` classnames qualify. The world entity and out-of-range indices never do. No
/// oracle (reads the `g_entities` global + classname strings).
///
/// # Safety
/// `g_entities` must be initialised; the indexed entity's `classname` must be valid or null.
pub unsafe fn G_EntIsBreakable(entityNum: c_int) -> qboolean {
    if entityNum < 0 || entityNum >= ENTITYNUM_WORLD {
        return QFALSE;
    }

    let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityNum as usize);
    if (*ent).r.svFlags & SVF_GLASS_BRUSH != 0 {
        return QTRUE;
    }
    if Q_stricmp(c"func_breakable".as_ptr(), (*ent).classname) == 0 {
        return QTRUE;
    }

    if Q_stricmp(c"misc_model_breakable".as_ptr(), (*ent).classname) == 0 {
        return QTRUE;
    }
    if Q_stricmp(c"misc_maglock".as_ptr(), (*ent).classname) == 0 {
        return QTRUE;
    }

    QFALSE
}

/* ===========================================================================
   GLASS  (g_mover.c:2857+)
=========================================================================== */

/// `void GlassDie(gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int mod)`
/// (g_mover.c:2863). Breakable glass `die`: guarded against re-triggering, it fires its targets and
/// spawns an `EV_GLASS_SHATTER` temp-entity carrying the impact point (`pos1`), break direction
/// (`pos2`), shard count and shatter time, then frees itself. No oracle (callback dispatch + temp
/// entity).
///
/// # Safety
/// `self_`/`attacker` must point to valid `gentity_t`s; `g_entities`/`level` must be initialised.
pub unsafe extern "C" fn GlassDie(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _mod_: c_int,
) {
    let mut dif: vec3_t = [0.0; 3];

    if (*self_).genericValue5 != 0 {
        //was already destroyed, do not retrigger it
        return;
    }

    (*self_).genericValue5 = 1;

    dif[0] = ((*self_).r.absmax[0] + (*self_).r.absmin[0]) / 2.0;
    dif[1] = ((*self_).r.absmax[1] + (*self_).r.absmin[1]) / 2.0;
    dif[2] = ((*self_).r.absmax[2] + (*self_).r.absmin[2]) / 2.0;

    G_UseTargets(self_, attacker);

    (*self_).splashRadius = 40; // ?? some random number, maybe it's ok?

    let te = G_TempEntity(&dif, EV_GLASS_SHATTER);
    (*te).s.genericenemyindex = (*self_).s.number;
    VectorCopy(&(*self_).pos1, &mut (*te).s.origin);
    VectorCopy(&(*self_).pos2, &mut (*te).s.angles);
    (*te).s.trickedentindex = (*self_).splashRadius;
    (*te).s.pos.trTime = (*self_).genericValue3;

    G_FreeEntity(self_);
}

/// `void GlassDie_Old(gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage,
/// int mod)` (g_mover.c:2893). The pre-shatter-FX glass `die`: fires targets and spawns an
/// `EV_GLASS_SHATTER` temp-entity carrying the brush bounds (`r.maxs`/`r.mins`) rather than an
/// impact point, then frees itself. Retained verbatim though the retail game wires up [`GlassDie`].
/// No oracle (callback dispatch + temp entity).
///
/// # Safety
/// `self_`/`attacker` must point to valid `gentity_t`s; `g_entities`/`level` must be initialised.
pub unsafe extern "C" fn GlassDie_Old(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _mod_: c_int,
) {
    let mut dif: vec3_t = [0.0; 3];

    dif[0] = ((*self_).r.absmax[0] + (*self_).r.absmin[0]) / 2.0;
    dif[1] = ((*self_).r.absmax[1] + (*self_).r.absmin[1]) / 2.0;
    dif[2] = ((*self_).r.absmax[2] + (*self_).r.absmin[2]) / 2.0;

    G_UseTargets(self_, attacker);

    let te = G_TempEntity(&dif, EV_GLASS_SHATTER);
    (*te).s.genericenemyindex = (*self_).s.number;
    VectorCopy(&(*self_).r.maxs, &mut (*te).s.origin);
    VectorCopy(&(*self_).r.mins, &mut (*te).s.angles);

    G_FreeEntity(self_);
}

/// `void GlassPain(gentity_t *self, gentity_t *attacker, int damage)` (g_mover.c:2912). Breakable
/// glass `pain`: a verbatim no-op in the retail game (kept so the `pain` hook is installed). No
/// oracle (empty).
///
/// # Safety
/// Arguments are unused; the function is inert.
pub unsafe extern "C" fn GlassPain(
    _self_: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
) {
    //G_Printf("Mr. Glass says: PLZ NO IT HURTS\n");
    //Make "cracking" sound?
}

/// `void GlassUse(gentity_t *self, gentity_t *other, gentity_t *activator)` (g_mover.c:2918).
/// Breakable glass `use`: with no real impactor to blame, it synthesises a break point/direction
/// from the centres of `self` and `other` (scaling the direction to 390), then calls [`GlassDie`].
/// No oracle (callback dispatch).
///
/// # Safety
/// `self_`/`other`/`activator` must point to valid `gentity_t`s.
pub unsafe extern "C" fn GlassUse(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    let mut temp1: vec3_t = [0.0; 3];
    let mut temp2: vec3_t = [0.0; 3];

    //no direct object to blame for the break, so fill the values with whatever
    VectorAdd(&(*self_).r.mins, &(*self_).r.maxs, &mut temp1);
    let t1 = temp1;
    VectorScale(&t1, 0.5, &mut temp1);

    VectorAdd(&(*other).r.mins, &(*other).r.maxs, &mut temp2);
    let t2 = temp2;
    VectorScale(&t2, 0.5, &mut temp2);

    VectorSubtract(&temp1, &temp2, &mut (*self_).pos2);
    VectorCopy(&temp1, &mut (*self_).pos1);

    VectorNormalize(&mut (*self_).pos2);
    let p2 = (*self_).pos2;
    VectorScale(&p2, 390.0, &mut (*self_).pos2);

    GlassDie(self_, other, activator, 100, MOD_UNKNOWN);
}

/// `void SP_func_glass( gentity_t *ent )` (g_mover.c:2945). Spawn a `func_glass`: breakable glass.
/// Sets its brush model, runs [`InitMover`], flags it `SVF_GLASS_BRUSH`, seats it at its origin,
/// defaults health to 1, reads `maxshards`, makes it a `MOVER_POS1` mover, sets `takedamage` unless
/// the `INACTIVE` (1) spawnflag is set, and installs the [`GlassDie`]/[`GlassUse`]/[`GlassPain`]
/// callbacks. No oracle (spawn-key reads + entity-state mutation).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` during entity spawning; the spawn-var system must be
/// active.
pub unsafe fn SP_func_glass(ent: *mut gentity_t) {
    trap::SetBrushModel(ent, &CStr::from_ptr((*ent).model).to_string_lossy());
    InitMover(ent);

    (*ent).r.svFlags = SVF_GLASS_BRUSH;

    VectorCopy(&(*ent).s.origin, &mut (*ent).s.pos.trBase);
    VectorCopy(&(*ent).s.origin, &mut (*ent).r.currentOrigin);
    if (*ent).health == 0 {
        (*ent).health = 1;
    }

    G_SpawnInt(
        c"maxshards".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue3,
    );

    (*ent).genericValue1 = 0;

    (*ent).genericValue4 = 1;

    (*ent).moverState = MOVER_POS1;

    if (*ent).spawnflags & 1 != 0 {
        (*ent).takedamage = QFALSE;
    } else {
        (*ent).takedamage = QTRUE;
    }

    (*ent).die = Some(GlassDie);
    (*ent).r#use = Some(GlassUse);
    (*ent).pain = Some(GlassPain);
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle::{jka_G_CreateRotationMatrix, jka_G_RotatePoint, jka_G_TransposeMatrix};

    /// Spread of angle triples — zero, the cardinal yaws, negatives, and large
    /// out-of-range magnitudes that exercise the trig path. `G_CreateRotationMatrix`
    /// checked bit-exact (all nine matrix floats) against the extracted C.
    #[test]
    fn g_createrotationmatrix_matches_oracle() {
        let cases: &[vec3_t] = &[
            [0.0, 0.0, 0.0],
            [0.0, 90.0, 0.0],
            [45.0, 45.0, 45.0],
            [-30.0, 200.0, -170.0],
            [725.25, -725.25, 12.5],
            [89.9, 359.9, -0.1],
        ];

        for (i, angles) in cases.iter().enumerate() {
            let mut matrix: [vec3_t; 3] = [[0.0; 3]; 3];
            G_CreateRotationMatrix(angles, &mut matrix);

            let mut want = [0f32; 9];
            unsafe { jka_G_CreateRotationMatrix(angles.as_ptr(), want.as_mut_ptr()) };

            let got: [f32; 9] = [
                matrix[0][0], matrix[0][1], matrix[0][2],
                matrix[1][0], matrix[1][1], matrix[1][2],
                matrix[2][0], matrix[2][1], matrix[2][2],
            ];
            assert_eq!(got, want, "case {i}: angles {angles:?}");
        }
    }

    /// Transpose of an asymmetric basis — checked bit-exact (all nine floats) against C.
    #[test]
    fn g_transposematrix_matches_oracle() {
        let cases: &[[vec3_t; 3]] = &[
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            [[-1.5, 0.0, 2.25], [100.0, -0.5, 3.0], [-7.0, 8.5, -9.0]],
        ];

        for (i, matrix) in cases.iter().enumerate() {
            let mut transpose: [vec3_t; 3] = [[0.0; 3]; 3];
            G_TransposeMatrix(matrix, &mut transpose);

            let flat_in: [f32; 9] = [
                matrix[0][0], matrix[0][1], matrix[0][2],
                matrix[1][0], matrix[1][1], matrix[1][2],
                matrix[2][0], matrix[2][1], matrix[2][2],
            ];
            let mut want = [0f32; 9];
            unsafe { jka_G_TransposeMatrix(flat_in.as_ptr(), want.as_mut_ptr()) };

            let got: [f32; 9] = [
                transpose[0][0], transpose[0][1], transpose[0][2],
                transpose[1][0], transpose[1][1], transpose[1][2],
                transpose[2][0], transpose[2][1], transpose[2][2],
            ];
            assert_eq!(got, want, "case {i}");
        }
    }

    /// Rotate a spread of points through an asymmetric matrix — checked bit-exact
    /// (all three output components) against C.
    #[test]
    fn g_rotatepoint_matches_oracle() {
        let matrix: [vec3_t; 3] = [[0.5, -1.0, 2.0], [3.0, 0.25, -0.5], [-1.0, 4.0, 1.5]];
        let points: &[vec3_t] = &[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [-2.5, 3.0, -7.25],
            [100.0, -0.5, 0.0],
        ];

        for (i, point) in points.iter().enumerate() {
            let mut p = *point;
            G_RotatePoint(&mut p, &matrix);

            let flat_m: [f32; 9] = [
                matrix[0][0], matrix[0][1], matrix[0][2],
                matrix[1][0], matrix[1][1], matrix[1][2],
                matrix[2][0], matrix[2][1], matrix[2][2],
            ];
            let mut want = [0f32; 3];
            unsafe { jka_G_RotatePoint(point.as_ptr(), flat_m.as_ptr(), want.as_mut_ptr()) };

            assert_eq!(p, want, "case {i}: point {point:?}");
        }
    }
}
