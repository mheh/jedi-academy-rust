//! Slice of `NPC_utils.c` — the NPC track proper (stage 90) is not yet ported, but
//! `G_ActivateBehavior` is pulled forward here because the entity classes
//! (`g_target`/`g_trigger`/`g_mover`/`g_misc`) and ICARUS scripting both drive it:
//! a spawned entity's `behaviorSet` fires its `BSET_*` script through this hook.
//! SP maps run in MP and ship scripts, so this path is required, not stubbed.
//!
//! Ported here: `G_ActivateBehavior` (NPC_utils.c:851). The NPC-instance branch
//! (resolve `bs_name` to a `bState_t` via `BSTable` and set `self->NPC->behaviorState`)
//! is **not yet ported** with the rest of the NPC track — `gNPC_t` is still an opaque stub
//! (`b_public_h.rs`) and `BSTable`/`bState_t`/`BS_DEFAULT` are unported. No current
//! entity is an NPC, so control always reaches the ICARUS branch. See `DEVIATIONS.md`.

#![allow(non_snake_case)] // C function names (`G_ActivateBehavior`, `G_GetBoltPosition`) kept verbatim
#![allow(non_upper_case_globals)] // C global names (`teamNumbers`, `teamStrength`) kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::b_public_h::{
    spot_t, SPOT_CHEST, SPOT_GROUND, SPOT_HEAD, SPOT_HEAD_LEAN, SPOT_LEGS, SPOT_ORIGIN, SPOT_WEAPON,
};
use crate::codemp::game::b_public_h::{BS_DEFAULT, BS_FOLLOW_LEADER, SCF_DONT_FIRE};
use crate::codemp::game::bg_misc::bgToggleableSurfaces;
use crate::codemp::game::bg_public::{
    BG_GiveMeVectorFromMatrix, BG_NUM_TOGGLEABLE_SURFACES, EF2_HELD_BY_MONSTER, TEAM_NUM_TEAMS,
};
use crate::codemp::game::bg_public::{
    ET_NPC, GT_TEAM, MASK_PLAYERSOLID, TEAM_BLUE, TEAM_FREE, TEAM_RED, TEAM_SPECTATOR,
};
use crate::codemp::game::bg_public::{EV_CONFUSE1, EV_CONFUSE3, PMF_FOLLOW};
use crate::codemp::game::bg_weapons_h::{WP_EMPLACED_GUN, WP_NONE, WP_SABER};
use crate::codemp::game::g_local::AEL_DISCOVERED;
use crate::codemp::game::g_local::{gentity_t, FL_NOTARGET};
use crate::codemp::game::g_main::{g_entities, g_gametype, level, Com_Printf};
use crate::codemp::game::g_public_h::{Q3_INFINITE, TID_ANGLE_FACE};
use crate::codemp::game::g_utils::{G_BoneIndex, GetAnglesForDirection};
use crate::codemp::game::g_weapon::CalcMuzzlePoint;
use crate::codemp::game::npc::{client, ucmd, NPCInfo, NPC};
use crate::codemp::game::npc_combat::{G_AddVoiceEvent, G_ClearEnemy, G_SetEnemy, NPC_ClearShot};
use crate::codemp::game::npc_senses::{
    G_ClearLOS, G_ClearLOS2, G_ClearLOS3, G_ClearLOS4, G_ClearLOS5, InFOV, NPC_CheckAlertEvents,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, AngleDelta, AngleNormalize360, AngleVectors, Distance, DistanceSquared,
    VectorAdd, VectorCompare, VectorCopy, VectorLengthSquared, VectorMA, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{va, Q_stricmp};
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, vec3_t, ANGLE2SHORT, ENTITYNUM_NONE, ENTITYNUM_WORLD, FP_SPEED, MAX_CLIENTS,
    NEGATIVE_Y, NEGATIVE_Z, ORIGIN, PITCH, POSITIVE_X, Q3_SCRIPT_DIR, ROLL, SHORT2ANGLE,
    WORLD_SIZE, YAW,
};
use crate::codemp::game::teams_h::CLASS_ATST;
use crate::codemp::game::teams_h::{
    CLASS_GALAKMECH, CLASS_RANCOR, CLASS_WAMPA, NPCTEAM_ENEMY, NPCTEAM_FREE, NPCTEAM_NEUTRAL,
    NPCTEAM_PLAYER,
};
use crate::codemp::ghoul2::g2_h::BONE_ANGLES_POSTMULT;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// `#define MIN_ANGLE_ERROR 0.01f` (b_local.h:29).
const MIN_ANGLE_ERROR: f32 = 0.01;

// `#define VALID_ATTACK_CONE 2.0f` (NPC_utils.c:11) — degrees.
const VALID_ATTACK_CONE: f32 = 2.0;

// `#define MAX_RADIUS_ENTS 256` (NPC_utils.c:1249) — "NOTE: This can cause entities to be lost".
const MAX_RADIUS_ENTS: usize = 256;

// Native libc `atof` (the C uses it directly on the cvar string buffer in
// `NPC_UpdateAngles`). Bound locally as `extern "C"` — same idiom as g_ICARUScb.rs.
extern "C" {
    fn atof(s: *const c_char) -> f64;
}

/// `qboolean G_ActivateBehavior( gentity_t *self, int bset )` (NPC_utils.c:851).
///
/// Fire entity `self`'s behaviorSet script for slot `bset` (a `BSET_*` event:
/// spawn/use/death/…). Returns `qfalse` when the entity has no script for that slot
/// (the case for every scriptless entity) and `qtrue` when one is dispatched. The
/// entity-class `Use_*`/`Think_*` callbacks call this so SP-map scripts run in MP.
///
/// The C `if ( self->NPC ) bSID = GetIDForString( BSTable, bs_name )` and the
/// `bSID > -1` NPC-dispatch branch (`self->NPC->behaviorState = bSID`) are **not yet ported**
/// with the NPC track (stage 90): `gNPC_t` is opaque and `BSTable`/`bState_t`/
/// `BS_DEFAULT` are unported. `self->NPC` is null for every current entity, so `bSID`
/// stays `-1` and control always falls to the ICARUS branch — `trap_ICARUS_RunScript`
/// runs `scripts/<bs_name>` on the engine-side interpreter. No oracle (drives the
/// trap, reads the global entity). The `#ifndef FINAL_BUILD`/`if (0)` debug block in
/// the C else-branch is dead code and omitted.
///
/// # Safety
/// `self` may be null (checked); otherwise must point to a valid `gentity_t`, and
/// `bset` must be a valid `bSet_t` index into `behaviorSet`.
pub unsafe fn G_ActivateBehavior(self_: *mut gentity_t, bset: c_int) -> qboolean {
    if self_.is_null() {
        return QFALSE;
    }

    let bs_name = (*self_).behaviorSet[bset as usize];

    // VALIDSTRING( bs_name )  ==  ( bs_name != 0 ) && ( bs_name[0] != 0 )
    if bs_name.is_null() || *bs_name == 0 {
        return QFALSE;
    }

    // NPC behaviorState dispatch not yet ported (stage 90 — see doc above); bSID stays -1,
    // so a scripted entity always takes the C else-branch:
    //   trap_ICARUS_RunScript( self, va( "%s/%s", Q3_SCRIPT_DIR, bs_name ) );
    let path = va(format_args!(
        "{}/{}",
        Q3_SCRIPT_DIR,
        CStr::from_ptr(bs_name).to_string_lossy()
    ));
    trap::ICARUS_RunScript(self_, path as *const c_char);

    QTRUE
}

/// `void CalcEntitySpot( const gentity_t *ent, const spot_t spot, vec3_t point )`
/// (NPC_utils.c:20).
///
/// Resolve `spot` (origin / head / chest / legs / weapon-muzzle / ground) to a world
/// point on `ent`, written into `point`. *"Uses shootAngles if a NPC has them"* — the
/// `SPOT_WEAPON` branch aims along `ent->NPC->shootAngles` when set (and not equal to the
/// view angles), else along `ps.viewangles`, then defers to `CalcMuzzlePoint`. The
/// `SPOT_HEAD`/`SPOT_HEAD_LEAN`/`SPOT_CHEST` head-eyespot path reads
/// `client->renderInfo.eyePoint` (with the `CLASS_ATST` +28 magic-number nudge and the
/// NPC bbox-center XY override so leaning doesn't wiggle the aim). `SPOT_GROUND` traces
/// 64u down from the bbox bottom against `MASK_PLAYERSOLID`. The commented-out
/// `SubtractLeanOfs`/`AddLeanOfs` and the FIXME `SPOT_CHEST` tag-interpolation are carried
/// as-is (dead in the original). No oracle — dereferences `gentity_t`/`gclient_t`/`gNPC_t`
/// and drives `trap_Trace`.
///
/// # Safety
/// `ent` may be null (checked); otherwise must point to a valid `gentity_t`. `point` must
/// point to a writable `vec3_t`.
pub unsafe fn CalcEntitySpot(ent: *const gentity_t, spot: spot_t, point: *mut vec3_t) {
    let mut forward: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];

    if ent.is_null() {
        return;
    }
    let point = &mut *point;
    match spot {
        SPOT_ORIGIN => {
            if VectorCompare(&(*ent).r.currentOrigin, &vec3_origin) != 0 {
                //brush
                VectorSubtract(&(*ent).r.absmax, &(*ent).r.absmin, point); //size
                let size = *point;
                VectorMA(&(*ent).r.absmin, 0.5, &size, point);
            } else {
                VectorCopy(&(*ent).r.currentOrigin, point);
            }
        }

        SPOT_CHEST | SPOT_HEAD => {
            if !(*ent).client.is_null()
                && VectorLengthSquared(&(*(*ent).client).renderInfo.eyePoint) != 0.0
            /*&& (ent->client->ps.viewEntity <= 0 || ent->client->ps.viewEntity >= ENTITYNUM_WORLD)*/
            {
                //Actual tag_head eyespot!
                //FIXME: Stasis aliens may have a problem here...
                VectorCopy(&(*(*ent).client).renderInfo.eyePoint, point);
                if (*(*ent).client).NPC_class == CLASS_ATST {
                    //adjust up some
                    point[2] += 28.0; //magic number :)
                }
                if !(*ent).NPC.is_null() {
                    //always aim from the center of my bbox, so we don't wiggle when we lean forward or backwards
                    point[0] = (*ent).r.currentOrigin[0];
                    point[1] = (*ent).r.currentOrigin[1];
                }
                /*
                else if (ent->s.eType == ET_PLAYER )
                {
                    SubtractLeanOfs( ent, point );
                }
                */
            } else {
                VectorCopy(&(*ent).r.currentOrigin, point);
                if !(*ent).client.is_null() {
                    point[2] += (*(*ent).client).ps.viewheight as f32;
                }
            }
            if spot == SPOT_CHEST && !(*ent).client.is_null() {
                if (*(*ent).client).NPC_class != CLASS_ATST {
                    //adjust up some
                    point[2] -= (*ent).r.maxs[2] * 0.2;
                }
            }
        }

        SPOT_HEAD_LEAN => {
            if !(*ent).client.is_null()
                && VectorLengthSquared(&(*(*ent).client).renderInfo.eyePoint) != 0.0
            /*&& (ent->client->ps.viewEntity <= 0 || ent->client->ps.viewEntity >= ENTITYNUM_WORLD*/
            {
                //Actual tag_head eyespot!
                //FIXME: Stasis aliens may have a problem here...
                VectorCopy(&(*(*ent).client).renderInfo.eyePoint, point);
                if (*(*ent).client).NPC_class == CLASS_ATST {
                    //adjust up some
                    point[2] += 28.0; //magic number :)
                }
                if !(*ent).NPC.is_null() {
                    //always aim from the center of my bbox, so we don't wiggle when we lean forward or backwards
                    point[0] = (*ent).r.currentOrigin[0];
                    point[1] = (*ent).r.currentOrigin[1];
                }
                /*
                else if ( ent->s.eType == ET_PLAYER )
                {
                    SubtractLeanOfs( ent, point );
                }
                */
                //NOTE: automatically takes leaning into account!
            } else {
                VectorCopy(&(*ent).r.currentOrigin, point);
                if !(*ent).client.is_null() {
                    point[2] += (*(*ent).client).ps.viewheight as f32;
                }
                //AddLeanOfs ( ent, point );
            }
        }

        //FIXME: implement...
        //case SPOT_CHEST:
        //Returns point 3/4 from tag_torso to tag_head?
        //break;
        SPOT_LEGS => {
            VectorCopy(&(*ent).r.currentOrigin, point);
            point[2] += (*ent).r.mins[2] * 0.5;
        }

        SPOT_WEAPON => {
            if !(*ent).NPC.is_null()
                && VectorCompare(&(*(*ent).NPC).shootAngles, &vec3_origin) == 0
                && VectorCompare(&(*(*ent).NPC).shootAngles, &(*(*ent).client).ps.viewangles) == 0
            {
                AngleVectors(
                    &(*(*ent).NPC).shootAngles,
                    Some(&mut forward),
                    Some(&mut right),
                    Some(&mut up),
                );
            } else {
                AngleVectors(
                    &(*(*ent).client).ps.viewangles,
                    Some(&mut forward),
                    Some(&mut right),
                    Some(&mut up),
                );
            }
            CalcMuzzlePoint(ent as *mut gentity_t, &forward, &right, &up, point);
            //NOTE: automatically takes leaning into account!
        }

        SPOT_GROUND => {
            // if entity is on the ground, just use it's absmin
            if (*ent).s.groundEntityNum != -1 {
                VectorCopy(&(*ent).r.currentOrigin, point);
                point[2] = (*ent).r.absmin[2];
                return;
            }

            // if it is reasonably close to the ground, give the point underneath of it
            VectorCopy(&(*ent).r.currentOrigin, &mut start);
            start[2] = (*ent).r.absmin[2];
            VectorCopy(&start, &mut end);
            end[2] -= 64.0;
            let tr = trap::Trace(
                &start,
                &(*ent).r.mins,
                &(*ent).r.maxs,
                &end,
                (*ent).s.number,
                MASK_PLAYERSOLID,
            );
            if tr.fraction < 1.0 {
                VectorCopy(&tr.endpos, point);
                return;
            }

            // otherwise just use the origin
            VectorCopy(&(*ent).r.currentOrigin, point);
        }

        _ => {
            VectorCopy(&(*ent).r.currentOrigin, point);
        }
    }
}

/// `void G_GetBoltPosition( gentity_t *self, int boltIndex, vec3_t pos, int modelIndex )`
/// (NPC_utils.c:1713).
///
/// World-space origin of bolt `boltIndex` on model `modelIndex` of `self`'s ghoul2
/// instance, written into `pos`. The model is posed flat (only YAW) at `self`'s current
/// origin: clients use `ps.viewangles[YAW]` (clients *"don't actually even keep
/// r.currentAngles maintained"*), non-clients use `r.currentAngles[YAW]`. Bails if `self`
/// is gone or has no ghoul2 instance, leaving `pos` untouched. No oracle — drives the
/// `trap_G2API_GetBoltMatrix` syscall and reads entity/level state.
///
/// # Safety
/// `self_` may be null (checked); otherwise must point to a valid `gentity_t`. `pos` may
/// be null (checked, mirroring the C `if ( pos )`); otherwise must point to a `vec3_t`.
pub unsafe fn G_GetBoltPosition(
    self_: *mut gentity_t,
    boltIndex: c_int,
    pos: *mut vec3_t,
    modelIndex: c_int,
) {
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let mut result: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];

    if self_.is_null() || (*self_).inuse == QFALSE {
        return;
    }

    if !(*self_).client.is_null() {
        //clients don't actually even keep r.currentAngles maintained
        VectorSet(&mut angles, 0.0, (*(*self_).client).ps.viewangles[YAW], 0.0);
    } else {
        VectorSet(&mut angles, 0.0, (*self_).r.currentAngles[YAW], 0.0);
    }

    // C: if ( /*!self || ...haha (sorry, i'm tired)*/ !self->ghoul2 )
    if (*self_).ghoul2.is_null() {
        return;
    }

    trap::G2API_GetBoltMatrix(
        (*self_).ghoul2,
        modelIndex,
        boltIndex,
        &mut boltMatrix,
        &angles,
        &(*self_).r.currentOrigin,
        (*addr_of!(level)).time,
        null_mut(),
        &(*self_).modelScale,
    );
    if !pos.is_null() {
        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut result);
        VectorCopy(&result, &mut *pos);
    }
}

/// `void NPC_SetBoneAngles(gentity_t *ent, char *bone, vec3_t angles)` (NPC_utils.c:906).
///
/// Override one of the entity's up-to-4 server-side bone-angle slots (`s.boneIndex1/2`
/// + `s.boneAngles1/2` — slots 3/4 are commented out in the C and unused) and, if the
/// entity has a ghoul2 instance, push the override to the engine via
/// `trap_G2API_SetBoneAngles`. The C walks the slots with a `int *thebone` pointer; this
/// port mirrors that with a slot index (`thebone == None` ⇔ C's `thebone == NULL`).
///
/// No oracle — drives the `trap_G2API_SetBoneAngles` tail and mutates entity state. The
/// `#ifdef _XBOX` `byte *` variant is the non-PC build (faithful-omitted); the
/// `#ifndef FINAL_BUILD` warning is kept (FINAL_BUILD is undefined in the retail build).
pub unsafe fn NPC_SetBoneAngles(ent: *mut gentity_t, bone: &str, angles: &vec3_t) {
    // `thebone`/`boneVector` walk over the 4 slots; None == C's NULL pointer.
    let mut thebone: Option<usize> = Some(0);
    let mut firstFree: Option<usize> = None;
    let mut i: c_int = 0;
    let boneIndex: c_int = G_BoneIndex(bone);
    let flags: c_int;
    let up: c_int;
    let right: c_int;
    let forward: c_int;

    // accessor mirroring `*thebone` for the 4 slots.
    unsafe fn bone_idx(ent: *mut gentity_t, slot: usize) -> c_int {
        match slot {
            0 => (*ent).s.boneIndex1,
            1 => (*ent).s.boneIndex2,
            2 => (*ent).s.boneIndex3,
            _ => (*ent).s.boneIndex4,
        }
    }

    while let Some(slot) = thebone {
        if bone_idx(ent, slot) == 0 && firstFree.is_none() {
            //if the value is 0 then this index is clear, we can use it if we don't find the bone we want already existing.
            firstFree = Some(slot);
        } else if bone_idx(ent, slot) != 0 {
            if bone_idx(ent, slot) == boneIndex {
                //this is it
                break;
            }
        }

        match i {
            0 => {
                thebone = Some(1); // &ent->s.boneIndex2 / &ent->s.boneAngles2
            }
            1 => {
                thebone = Some(2); // &ent->s.boneIndex3 / &ent->s.boneAngles3
            }
            2 => {
                thebone = Some(3); // &ent->s.boneIndex4 / &ent->s.boneAngles4
            }
            _ => {
                thebone = None;
            }
        }

        i += 1;
    }

    if thebone.is_none() {
        //didn't find it, create it
        let firstFree = match firstFree {
            None => {
                //no free bones.. can't do a thing then.
                Com_Printf("WARNING: NPC has no free bone indexes\n");
                return;
            }
            Some(s) => s,
        };

        thebone = Some(firstFree);

        match firstFree {
            0 => (*ent).s.boneIndex1 = boneIndex,
            1 => (*ent).s.boneIndex2 = boneIndex,
            2 => (*ent).s.boneIndex3 = boneIndex,
            _ => (*ent).s.boneIndex4 = boneIndex,
        }
    }

    //If we got here then we have a vector and an index.
    let slot = thebone.unwrap();

    //Copy the angles over the vector in the entitystate, so we can use the corresponding index
    //to set the bone angles on the client.
    match slot {
        0 => VectorCopy(angles, &mut (*ent).s.boneAngles1),
        1 => VectorCopy(angles, &mut (*ent).s.boneAngles2),
        2 => VectorCopy(angles, &mut (*ent).s.boneAngles3),
        _ => VectorCopy(angles, &mut (*ent).s.boneAngles4),
    }

    //Now set the angles on our server instance if we have one.

    if (*ent).ghoul2.is_null() {
        return;
    }

    flags = BONE_ANGLES_POSTMULT;
    up = POSITIVE_X;
    right = NEGATIVE_Y;
    forward = NEGATIVE_Z;

    //first 3 bits is forward, second 3 bits is right, third 3 bits is up
    (*ent).s.boneOrient = forward | (right << 3) | (up << 6);

    trap::G2API_SetBoneAngles(
        (*ent).ghoul2,
        0,
        bone,
        angles,
        flags,
        up,
        right,
        forward,
        null_mut(),
        100,
        (*addr_of!(level)).time,
    );
}

/// `void NPC_ClearLookTarget( gentity_t *self )` (NPC_utils.c:1617).
///
/// Reset `self`'s head-look target to "none". Bails when `self` has no client, or
/// when it is `EF2_HELD_BY_MONSTER` — *"lookTarget is set by and to the monster
/// that's holding you, no other operations can change that"*. Otherwise clears
/// `renderInfo.lookTarget` to `ENTITYNUM_NONE` and zeroes `lookTargetClearTime`.
/// No oracle — pure entity-state writes.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; its `client` is checked before use.
pub unsafe fn NPC_ClearLookTarget(self_: *mut gentity_t) {
    if (*self_).client.is_null() {
        return;
    }

    if (*(*self_).client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
        //lookTarget is set by and to the monster that's holding you, no other operations can change that
        return;
    }

    (*(*self_).client).renderInfo.lookTarget = ENTITYNUM_NONE; //ENTITYNUM_WORLD;
    (*(*self_).client).renderInfo.lookTargetClearTime = 0;
}

/// `void NPC_SetLookTarget( gentity_t *self, int entNum, int clearTime )` (NPC_utils.c:1638).
///
/// Point `self`'s head-look at entity `entNum`, with `clearTime` as the deadline after
/// which `NPC_CheckLookTarget` will clear it. Same client/`EF2_HELD_BY_MONSTER` guards
/// as `NPC_ClearLookTarget`. No oracle — pure entity-state writes.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; its `client` is checked before use.
pub unsafe fn NPC_SetLookTarget(self_: *mut gentity_t, entNum: c_int, clearTime: c_int) {
    if (*self_).client.is_null() {
        return;
    }

    if (*(*self_).client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
        //lookTarget is set by and to the monster that's holding you, no other operations can change that
        return;
    }

    (*(*self_).client).renderInfo.lookTarget = entNum;
    (*(*self_).client).renderInfo.lookTargetClearTime = clearTime;
}

/// `qboolean NPC_CheckLookTarget( gentity_t *self )` (NPC_utils.c:1659).
///
/// Validate `self`'s current head-look target, clearing it (via `NPC_ClearLookTarget`)
/// when it has gone stale: the target ent is no longer `inuse`, its `lookTargetClearTime`
/// has elapsed, or `self` is in battle with a *different* enemy (*"should always look at
/// current enemy if engaged in battle... FIXME: this could override certain scripted
/// lookTargets...???"*). Returns `qtrue` only when a still-valid lookTarget remains.
/// No oracle — entity-state reads + the sibling clear.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; its `client` is checked before use.
pub unsafe fn NPC_CheckLookTarget(self_: *mut gentity_t) -> qboolean {
    if !(*self_).client.is_null() {
        if (*(*self_).client).renderInfo.lookTarget >= 0
            && (*(*self_).client).renderInfo.lookTarget < ENTITYNUM_WORLD
        {
            //within valid range
            let look_ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*(*self_).client).renderInfo.lookTarget as usize);
            if (*look_ent).inuse == QFALSE {
                //lookTarget not inuse or not valid anymore
                // (the C `&g_entities[...] == NULL` can never hold — the array is static.)
                NPC_ClearLookTarget(self_);
            } else if (*(*self_).client).renderInfo.lookTargetClearTime != 0
                && (*(*self_).client).renderInfo.lookTargetClearTime < (*addr_of!(level)).time
            {
                //Time to clear lookTarget
                NPC_ClearLookTarget(self_);
            } else if !(*look_ent).client.is_null()
                && !(*self_).enemy.is_null()
                && look_ent != (*self_).enemy
            {
                //should always look at current enemy if engaged in battle... FIXME: this could override certain scripted lookTargets...???
                NPC_ClearLookTarget(self_);
            } else {
                return QTRUE;
            }
        }
    }

    QFALSE
}

/// `void NPC_SetSurfaceOnOff(gentity_t *ent, const char *surfaceName, int surfaceFlags)`
/// (NPC_utils.c:1005).
///
/// rww — *"another method of automatically managing surface status for the client and
/// server at once"*. Resolves `surfaceName` against the `bgToggleableSurfaces` name table;
/// warns and bails if it isn't a known toggleable surface. Otherwise records the on/off
/// state in the entitystate bitfields (`s.surfacesOn`/`s.surfacesOff`) so it replicates to
/// the client, then — if `ent` has a ghoul2 instance — pushes the same change to the
/// server-side model via `trap_G2API_SetSurfaceOnOff`. `surfaceFlags` is the file-local
/// `TURN_ON` (`0x00000000`) / `TURN_OFF` (`0x00000100`) pair. No oracle — entity-state +
/// trap tail.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `surface_name` must be a valid C string.
pub unsafe fn NPC_SetSurfaceOnOff(
    ent: *mut gentity_t,
    surface_name: *const c_char,
    surface_flags: c_int,
) {
    // #define TURN_ON 0x00000000
    const TURN_ON: c_int = 0x00000000;

    let mut i: c_int = 0;
    let mut found_it: qboolean = QFALSE;

    while (i as usize) < BG_NUM_TOGGLEABLE_SURFACES
        && !(*addr_of!(bgToggleableSurfaces))[i as usize].is_null()
    {
        if Q_stricmp(surface_name, (*addr_of!(bgToggleableSurfaces))[i as usize]) == 0 {
            //got it
            found_it = QTRUE;
            break;
        }
        i += 1;
    }

    if found_it == QFALSE {
        // #ifndef FINAL_BUILD (undefined in retail — warning kept)
        Com_Printf(&format!(
            "WARNING: Tried to toggle NPC surface that isn't in toggleable surface list ({})\n",
            CStr::from_ptr(surface_name).to_string_lossy()
        ));
        return;
    }

    if surface_flags == TURN_ON {
        //Make sure the entitystate values reflect this surface as on now.
        (*ent).s.surfacesOn |= 1 << i;
        (*ent).s.surfacesOff &= !(1 << i);
    } else {
        //Otherwise make sure they're off.
        (*ent).s.surfacesOn &= !(1 << i);
        (*ent).s.surfacesOff |= 1 << i;
    }

    if (*ent).ghoul2.is_null() {
        return;
    }

    trap::G2API_SetSurfaceOnOff((*ent).ghoul2, surface_name, surface_flags);
}

// File-static team tallies (NPC_utils.c:7-8). `teamCounter` (NPC_utils.c:9) is unused
// by any ported consumer and is omitted until its user lands.
/// `int teamNumbers[TEAM_NUM_TEAMS]` (NPC_utils.c:7) — living-client count per team.
pub static mut teamNumbers: [c_int; TEAM_NUM_TEAMS as usize] = [0; TEAM_NUM_TEAMS as usize];
/// `int teamStrength[TEAM_NUM_TEAMS]` (NPC_utils.c:8) — summed/averaged health per team.
pub static mut teamStrength: [c_int; TEAM_NUM_TEAMS as usize] = [0; TEAM_NUM_TEAMS as usize];

/// `void SetTeamNumbers (void)` (NPC_utils.c:818).
///
/// *"Sets the number of living clients on each team. FIXME: Does not account for
/// non-respawned players! FIXME: Don't include medics?"* Zeroes the per-team tallies,
/// counts living clients (and sums their health) into [`teamNumbers`]/[`teamStrength`],
/// then turns `teamStrength` into the average health per team. The count loop is `for i
/// in 0..1` in the retail source — it only ever inspects entity 0. No oracle — reads the
/// global `g_entities` and writes the file-static tallies.
///
/// # Safety
/// Reads the global `g_entities` array; must be called after the entity array is set up.
pub unsafe fn SetTeamNumbers() {
    let mut i: c_int = 0;
    while i < TEAM_NUM_TEAMS {
        teamNumbers[i as usize] = 0;
        teamStrength[i as usize] = 0;
        i += 1;
    }

    i = 0;
    while i < 1 {
        let found = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !(*found).client.is_null() {
            if (*found).health > 0 {
                //FIXME: or if a player!
                let team = (*(*found).client).playerTeam as usize;
                teamNumbers[team] += 1;
                teamStrength[team] += (*found).health;
            }
        }
        i += 1;
    }

    i = 0;
    while i < TEAM_NUM_TEAMS {
        //Get the average health
        teamStrength[i as usize] = (((teamStrength[i as usize] as f32)
            / (teamNumbers[i as usize] as f32)) as f64)
            .floor() as c_int;
        i += 1;
    }
}

/// `qboolean NPC_ClearLOS( const vec3_t start, const vec3_t end )` (NPC_utils.c:1075).
///
/// One-line wrapper over `G_ClearLOS` bound to the file-global `NPC`. No oracle —
/// drives the trapped trace via `G_ClearLOS` and reads the `NPC` global.
pub unsafe fn NPC_ClearLOS(start: &vec3_t, end: &vec3_t) -> qboolean {
    G_ClearLOS(NPC, start, end)
}

/// `qboolean NPC_ClearLOS5( const vec3_t end )` (NPC_utils.c:1079).
///
/// One-line wrapper over `G_ClearLOS5` bound to the file-global `NPC`. No oracle.
pub unsafe fn NPC_ClearLOS5(end: &vec3_t) -> qboolean {
    G_ClearLOS5(NPC, end)
}

/// `qboolean NPC_ClearLOS4( gentity_t *ent )` (NPC_utils.c:1083).
///
/// One-line wrapper over `G_ClearLOS4` bound to the file-global `NPC`. No oracle.
pub unsafe fn NPC_ClearLOS4(ent: *mut gentity_t) -> qboolean {
    G_ClearLOS4(NPC, ent)
}

/// `qboolean NPC_ClearLOS3( const vec3_t start, gentity_t *ent )` (NPC_utils.c:1087).
///
/// One-line wrapper over `G_ClearLOS3` bound to the file-global `NPC`. No oracle.
pub unsafe fn NPC_ClearLOS3(start: &vec3_t, ent: *mut gentity_t) -> qboolean {
    G_ClearLOS3(NPC, start, ent)
}

/// `qboolean NPC_ClearLOS2( gentity_t *ent, const vec3_t end )` (NPC_utils.c:1091).
///
/// One-line wrapper over `G_ClearLOS2` bound to the file-global `NPC`. No oracle.
pub unsafe fn NPC_ClearLOS2(ent: *mut gentity_t, end: &vec3_t) -> qboolean {
    G_ClearLOS2(NPC, ent, end)
}

/*
-------------------------
NPC_ValidEnemy
-------------------------
*/

/// `qboolean NPC_ValidEnemy( gentity_t *ent )` (NPC_utils.c:1102).
///
/// Decide whether `ent` is a legal target for the file-global `NPC`: a live, in-use,
/// non-self, non-notarget entity that is not a teammate and falls on the NPC's enemy
/// team (or a rampaging creature / unaligned attacker per the final disjunction).
/// No oracle — pure entity/team/client-field logic over the global `NPC` and `ent`;
/// touches no `NPCInfo` (the `enemyLastSeenTime` give-up check is commented out in C).
pub unsafe fn NPC_ValidEnemy(ent: *mut gentity_t) -> qboolean {
    let mut entTeam = TEAM_FREE;
    //Must be a valid pointer
    if ent.is_null() {
        return QFALSE;
    }

    //Must not be me
    if ent == NPC {
        return QFALSE;
    }

    //Must not be deleted
    if (*ent).inuse == QFALSE {
        return QFALSE;
    }

    //Must be alive
    if (*ent).health <= 0 {
        return QFALSE;
    }

    //In case they're in notarget mode
    if (*ent).flags & FL_NOTARGET != 0 {
        return QFALSE;
    }

    //Must be an NPC
    if (*ent).client.is_null() {
        //	if ( ent->svFlags&SVF_NONNPC_ENEMY )
        if (*ent).s.eType != ET_NPC {
            //still potentially valid
            if (*ent).alliedTeam == (*(*NPC).client).playerTeam {
                return QFALSE;
            } else {
                return QTRUE;
            }
        } else {
            return QFALSE;
        }
    } else if !(*ent).client.is_null() && (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
        //don't go after spectators
        return QFALSE;
    }
    if !(*ent).NPC.is_null() && !(*ent).client.is_null() {
        entTeam = (*(*ent).client).playerTeam;
    } else if !(*ent).client.is_null() {
        if (*addr_of!(g_gametype)).integer < GT_TEAM {
            entTeam = NPCTEAM_PLAYER;
        } else {
            if (*(*ent).client).sess.sessionTeam == TEAM_BLUE {
                entTeam = NPCTEAM_PLAYER;
            } else if (*(*ent).client).sess.sessionTeam == TEAM_RED {
                entTeam = NPCTEAM_ENEMY;
            } else {
                entTeam = NPCTEAM_NEUTRAL;
            }
        }
    }
    //Can't be on the same team
    if (*(*ent).client).playerTeam == (*(*NPC).client).playerTeam {
        return QFALSE;
    }

    //if haven't seen him in a while, give up
    //if ( NPCInfo->enemyLastSeenTime != 0 && level.time - NPCInfo->enemyLastSeenTime > 7000 )//FIXME: make a stat?
    //	return qfalse;
    if entTeam == (*(*NPC).client).enemyTeam //simplest case: they're on my enemy team
        || ((*(*NPC).client).enemyTeam == NPCTEAM_FREE && (*(*ent).client).NPC_class != (*(*NPC).client).NPC_class)//I get mad at anyone and this guy isn't the same class as me
        || ((*(*ent).client).NPC_class == CLASS_WAMPA && !(*ent).enemy.is_null())//a rampaging wampa
        || ((*(*ent).client).NPC_class == CLASS_RANCOR && !(*ent).enemy.is_null())//a rampaging rancor
        || (entTeam == NPCTEAM_FREE
            && (*(*ent).client).enemyTeam == NPCTEAM_FREE
            && !(*ent).enemy.is_null()
            && !(*(*ent).enemy).client.is_null()
            && ((*(*(*ent).enemy).client).playerTeam == (*(*NPC).client).playerTeam
                || ((*(*(*ent).enemy).client).playerTeam != NPCTEAM_ENEMY
                    && (*(*NPC).client).playerTeam == NPCTEAM_PLAYER)))
    //enemy is a rampaging non-aligned creature who is attacking someone on our team or a non-enemy (this last condition is used only if we're a good guy - in effect, we protect the innocent)
    {
        return QTRUE;
    }

    QFALSE
}

/*
qboolean NPC_UpdateAngles ( qboolean doPitch, qboolean doYaw )

Added: option to do just pitch or just yaw

Does not include "aim" in it's calculations

FIXME: stop compressing angles into shorts!!!!
*/
/// `qboolean NPC_UpdateAngles( qboolean doPitch, qboolean doYaw )` (NPC_utils.c:182).
///
/// Decays the file-global `NPC`'s view-angle error toward `NPCInfo->desiredPitch/Yaw`
/// and writes the result into the global `ucmd.angles[]` (minus `client->ps.delta_angles`).
/// Returns `qtrue` when the NPC is already aiming exactly (within `MIN_ANGLE_ERROR`),
/// in which case any pending `TID_ANGLE_FACE` ICARUS task is completed. Only the live
/// `#if 1` branch is ported; the `#else` legacy block is dead and omitted. No oracle —
/// reads/writes the `NPC`/`NPCInfo`/`client`/`ucmd` globals and drives the ICARUS traps.
///
/// # Safety
/// Must be called only with the `NPC`/`NPCInfo`/`client` globals set to the NPC being
/// thought (i.e. from inside the NPC think loop).
pub unsafe fn NPC_UpdateAngles(doPitch: qboolean, doYaw: qboolean) -> qboolean {
    let mut decay: f32;
    let mut targetPitch: f32 = 0.0;
    let mut targetYaw: f32 = 0.0;
    let mut yawSpeed: f32;
    let mut exact: qboolean = QTRUE;

    // if angle changes are locked; just keep the current angles
    // aimTime isn't even set anymore... so this code was never reached, but I need a way to lock NPC's yaw, so instead of making a new SCF_ flag, just use the existing render flag... - dmv
    if (*NPC).enemy.is_null()
        && (level.time < (*NPCInfo).aimTime/*|| NPC->client->renderInfo.renderFlags & RF_LOCKEDANGLE*/)
    {
        if doPitch != QFALSE {
            targetPitch = (*NPCInfo).lockedDesiredPitch;
        }

        if doYaw != QFALSE {
            targetYaw = (*NPCInfo).lockedDesiredYaw;
        }
    } else {
        // we're changing the lockedDesired Pitch/Yaw below so it's lost it's original meaning, get rid of the lock flag
        //	NPC->client->renderInfo.renderFlags &= ~RF_LOCKEDANGLE;

        if doPitch != QFALSE {
            targetPitch = (*NPCInfo).desiredPitch;
            (*NPCInfo).lockedDesiredPitch = (*NPCInfo).desiredPitch;
        }

        if doYaw != QFALSE {
            targetYaw = (*NPCInfo).desiredYaw;
            (*NPCInfo).lockedDesiredYaw = (*NPCInfo).desiredYaw;
        }
    }

    if (*NPC).s.weapon == WP_EMPLACED_GUN {
        // FIXME: this seems to do nothing, actually...
        yawSpeed = 20.0;
    } else {
        yawSpeed = (*NPCInfo).stats.yawSpeed;
    }

    if (*NPC).s.weapon == WP_SABER
        && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0
    {
        // trap_Cvar_VariableStringBuffer("timescale", buf, sizeof(buf)); tFVal = atof(buf);
        let buf = std::ffi::CString::new(trap::Cvar_VariableString("timescale")).unwrap();
        let tFVal: f32 = atof(buf.as_ptr()) as f32;

        yawSpeed *= 1.0 / tFVal;
    }

    if doYaw != QFALSE {
        // decay yaw error
        let mut error = AngleDelta((*(*NPC).client).ps.viewangles[YAW], targetYaw);
        if error.abs() > MIN_ANGLE_ERROR {
            if error != 0.0 {
                exact = QFALSE;

                decay = 60.0 + yawSpeed * 3.0;
                decay *= 50.0 / 1000.0; //msec

                if error < 0.0 {
                    error += decay;
                    if error > 0.0 {
                        error = 0.0;
                    }
                } else {
                    error -= decay;
                    if error < 0.0 {
                        error = 0.0;
                    }
                }
            }
        }

        ucmd.angles[YAW] = ANGLE2SHORT(targetYaw + error) - (*client).ps.delta_angles[YAW];
    }

    //FIXME: have a pitchSpeed?
    if doPitch != QFALSE {
        // decay pitch error
        let mut error = AngleDelta((*(*NPC).client).ps.viewangles[PITCH], targetPitch);
        if error.abs() > MIN_ANGLE_ERROR {
            if error != 0.0 {
                exact = QFALSE;

                decay = 60.0 + yawSpeed * 3.0;
                decay *= 50.0 / 1000.0; //msec

                if error < 0.0 {
                    error += decay;
                    if error > 0.0 {
                        error = 0.0;
                    }
                } else {
                    error -= decay;
                    if error < 0.0 {
                        error = 0.0;
                    }
                }
            }
        }

        ucmd.angles[PITCH] = ANGLE2SHORT(targetPitch + error) - (*client).ps.delta_angles[PITCH];
    }

    ucmd.angles[ROLL] =
        ANGLE2SHORT((*(*NPC).client).ps.viewangles[ROLL]) - (*client).ps.delta_angles[ROLL];

    if exact != QFALSE && trap::ICARUS_TaskIDPending(NPC, TID_ANGLE_FACE) != QFALSE {
        trap::ICARUS_TaskIDComplete(NPC, TID_ANGLE_FACE);
    }
    exact
}

//rww - cheap check to see if an armed client is looking in our general direction
/// `qboolean NPC_SomeoneLookingAtMe( gentity_t *ent )` (NPC_utils.c:1048).
///
/// Cheap scan of the client slots: returns `qtrue` as soon as an in-use, armed,
/// non-spectating, non-following client both shares a PVS with `ent` and has `ent`
/// inside a ~30-degree view cone (`InFOV( ent, pEnt, 30, 30 )`). No oracle — walks the
/// `g_entities` global and drives `trap_InPVS`.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe fn NPC_SomeoneLookingAtMe(ent: *mut gentity_t) -> qboolean {
    let mut i: c_int = 0;
    let mut pEnt: *mut gentity_t;

    while i < MAX_CLIENTS as c_int {
        pEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize);

        if !pEnt.is_null()
            && (*pEnt).inuse != QFALSE
            && !(*pEnt).client.is_null()
            && (*(*pEnt).client).sess.sessionTeam != TEAM_SPECTATOR
            && (*(*pEnt).client).ps.pm_flags & PMF_FOLLOW == 0
            && (*pEnt).s.weapon != WP_NONE
        {
            if trap::InPVS(&(*ent).r.currentOrigin, &(*pEnt).r.currentOrigin) != QFALSE {
                if InFOV(ent, pEnt, 30, 30) != QFALSE {
                    //I'm in a 30 fov or so cone from this player.. that's enough I guess.
                    return QTRUE;
                }
            }
        }

        i += 1;
    }

    QFALSE
}

/*
-------------------------
NPC_CheckPlayerDistance
-------------------------
*/
/// `static qboolean NPC_CheckPlayerDistance( void )` (NPC_utils.c:1373).
///
/// `return qfalse;//MOOT in MP` — the entire distance/give-up body is `/* */`'d out in
/// the retail MP source, so this is an unconditional `qfalse`. File-private (`static`),
/// kept `pub(crate)` for the eventual NPC-track caller. No oracle — constant.
#[allow(dead_code)] // static helper; its NPC-track callers are still blocked
pub(crate) fn NPC_CheckPlayerDistance() -> qboolean {
    QFALSE //MOOT in MP
}

/*
-------------------------
NPC_CheckCharmed
-------------------------
*/
/// `void NPC_CheckCharmed( void )` (NPC_utils.c:1693).
///
/// When the file-global `NPC` was charmed onto the enemy team and its `charmedTime` has
/// elapsed, restore its original team allegiance (`playerTeam`/`enemyTeam`/`s.teamowner`
/// stashed in `genericValue1..3`), drop any leader/`BS_FOLLOW_LEADER` temp behavior, clear
/// its enemy, zero `charmedTime`, and play a random confuse voice event so the player knows
/// it snapped out. No oracle — mutates entity/NPCInfo state and drives `G_AddVoiceEvent`.
///
/// # Safety
/// Must be called only with the `NPC`/`NPCInfo` globals set to the NPC being thought.
pub unsafe fn NPC_CheckCharmed() {
    if (*NPCInfo).charmedTime != 0
        && (*NPCInfo).charmedTime < level.time
        && !(*NPC).client.is_null()
    {
        //we were charmed, set us back!
        (*(*NPC).client).playerTeam = (*NPC).genericValue1;
        (*(*NPC).client).enemyTeam = (*NPC).genericValue2;
        (*NPC).s.teamowner = (*NPC).genericValue3;

        (*(*NPC).client).leader = null_mut();
        if (*NPCInfo).tempBehavior == BS_FOLLOW_LEADER {
            (*NPCInfo).tempBehavior = BS_DEFAULT;
        }
        G_ClearEnemy(NPC);
        (*NPCInfo).charmedTime = 0;
        //say something to let player know you've snapped out of it
        G_AddVoiceEvent(NPC, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000);
    }
}

/// `float NPC_EntRangeFromBolt( gentity_t *targEnt, int boltIndex )` (NPC_utils.c:1748).
///
/// Distance from `targEnt`'s origin to the file-global `NPC`'s ghoul2 bolt `boltIndex`
/// (model 0). Returns `Q3_INFINITE` when `targEnt` is null. No oracle — drives
/// `G_GetBoltPosition` (a ghoul2 trap) over the `NPC` global.
///
/// # Safety
/// `targEnt` may be null (checked); otherwise must point to a valid `gentity_t`, and the
/// `NPC` global must be set.
pub unsafe fn NPC_EntRangeFromBolt(targEnt: *mut gentity_t, boltIndex: c_int) -> f32 {
    let mut org: vec3_t = [0.0; 3];

    if targEnt.is_null() {
        return Q3_INFINITE as f32;
    }

    G_GetBoltPosition(NPC, boltIndex, addr_of_mut!(org), 0);

    Distance(&(*targEnt).r.currentOrigin, &org)
}

/// `float NPC_EnemyRangeFromBolt( int boltIndex )` (NPC_utils.c:1762).
///
/// Convenience wrapper: distance from the file-global `NPC`'s current enemy to bolt
/// `boltIndex`. No oracle — delegates to [`NPC_EntRangeFromBolt`].
///
/// # Safety
/// The `NPC` global must be set.
pub unsafe fn NPC_EnemyRangeFromBolt(boltIndex: c_int) -> f32 {
    NPC_EntRangeFromBolt((*NPC).enemy, boltIndex)
}

/// `int NPC_GetEntsNearBolt( int *radiusEnts, float radius, int boltIndex, vec3_t boltOrg )`
/// (NPC_utils.c:1767).
///
/// Resolve the file-global `NPC`'s bolt `boltIndex` (model 0) into `boltOrg`, then fill
/// `radiusEnts` (capacity 128) with the entities whose bbox overlaps the `radius`-cube
/// around it, returning the count. No oracle — drives `G_GetBoltPosition` and
/// `trap_EntitiesInBox`.
///
/// # Safety
/// `radiusEnts` must point to a writable buffer of at least 128 `c_int`s; `boltOrg` must
/// point to a writable `vec3_t`; the `NPC` global must be set.
pub unsafe fn NPC_GetEntsNearBolt(
    radiusEnts: *mut c_int,
    radius: f32,
    boltIndex: c_int,
    boltOrg: *mut vec3_t,
) -> c_int {
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    //get my handRBolt's position
    let mut org: vec3_t = [0.0; 3];

    G_GetBoltPosition(NPC, boltIndex, addr_of_mut!(org), 0);

    let boltOrg = &mut *boltOrg;
    VectorCopy(&org, boltOrg);

    //Setup the bbox to search in
    for i in 0..3 {
        mins[i] = boltOrg[i] - radius;
        maxs[i] = boltOrg[i] + radius;
    }

    //Get the number of entities in a given space
    let list = core::slice::from_raw_parts_mut(radiusEnts, 128);
    trap::EntitiesInBox(&mins, &maxs, list)
}

/// `void NPC_AimWiggle( vec3_t enemy_org )` (NPC_utils.c:519).
///
/// Adds a slowly-changing aim offset to `enemy_org` so NPCs shoot for somewhere between
/// the head and torso (and not always dead-on). When `aimErrorDebounceTime` has elapsed it
/// rerolls `NPCInfo->aimOfs` from `NPC->enemy`'s bbox (`flrand`-scaled), then always offsets
/// `enemy_org` by it. No oracle — mutates `NPCInfo` and reads the `NPC` global.
///
/// # Safety
/// `enemy_org` must point to a writable `vec3_t`; the `NPC`/`NPCInfo` globals must be set and
/// `NPC->enemy` non-null when the reroll branch is taken.
pub unsafe fn NPC_AimWiggle(enemy_org: *mut vec3_t) {
    //shoot for somewhere between the head and torso
    //NOTE: yes, I know this looks weird, but it works
    if (*NPCInfo).aimErrorDebounceTime < level.time {
        (*NPCInfo).aimOfs[0] = 0.3 * flrand((*(*NPC).enemy).r.mins[0], (*(*NPC).enemy).r.maxs[0]);
        (*NPCInfo).aimOfs[1] = 0.3 * flrand((*(*NPC).enemy).r.mins[1], (*(*NPC).enemy).r.maxs[1]);
        if (*(*NPC).enemy).r.maxs[2] > 0.0 {
            (*NPCInfo).aimOfs[2] = (*(*NPC).enemy).r.maxs[2] * flrand(0.0, -1.0);
        }
    }
    let enemy_org = &mut *enemy_org;
    let cur = *enemy_org;
    VectorAdd(&cur, &(*NPCInfo).aimOfs, enemy_org);
}

/*
qboolean NPC_UpdateFiringAngles ( qboolean doPitch, qboolean doYaw )

  Includes aim when determining angles - so they don't always hit...
  */
/// `qboolean NPC_UpdateFiringAngles( qboolean doPitch, qboolean doYaw )` (NPC_utils.c:540).
///
/// Like [`NPC_UpdateAngles`] but folds in an aim-error term so NPCs miss according to their
/// `stats.aim`: decays the view-angle diff toward `desiredPitch/Yaw`, periodically rerolls
/// `lastAimErrorYaw/Pitch` (every `Q_irand(250,2000)`ms), and writes the summed result into
/// `ucmd.angles[]`. Only the live `#else` branch is ported (`#if 0` legacy dropped). No
/// oracle — reads/writes the `NPC`/`NPCInfo`/`client`/`ucmd` globals.
///
/// # Safety
/// Must be called only with the `NPC`/`NPCInfo`/`client` globals set to the NPC being thought.
pub unsafe fn NPC_UpdateFiringAngles(doPitch: qboolean, doYaw: qboolean) -> qboolean {
    let mut decay: f32;
    let mut targetPitch: f32 = 0.0;
    let mut targetYaw: f32 = 0.0;
    let mut exact: qboolean = QTRUE;

    // if angle changes are locked; just keep the current angles
    if level.time < (*NPCInfo).aimTime {
        if doPitch != QFALSE {
            targetPitch = (*NPCInfo).lockedDesiredPitch;
        }
        if doYaw != QFALSE {
            targetYaw = (*NPCInfo).lockedDesiredYaw;
        }
    } else {
        if doPitch != QFALSE {
            targetPitch = (*NPCInfo).desiredPitch;
        }
        if doYaw != QFALSE {
            targetYaw = (*NPCInfo).desiredYaw;
        }

        //		NPCInfo->aimTime = level.time + 250;
        if doPitch != QFALSE {
            (*NPCInfo).lockedDesiredPitch = (*NPCInfo).desiredPitch;
        }
        if doYaw != QFALSE {
            (*NPCInfo).lockedDesiredYaw = (*NPCInfo).desiredYaw;
        }
    }

    if (*NPCInfo).aimErrorDebounceTime < level.time {
        if Q_irand(0, 1) != 0 {
            (*NPCInfo).lastAimErrorYaw = ((6 - (*NPCInfo).stats.aim) as f32) * flrand(-1.0, 1.0);
        }
        if Q_irand(0, 1) != 0 {
            (*NPCInfo).lastAimErrorPitch = ((6 - (*NPCInfo).stats.aim) as f32) * flrand(-1.0, 1.0);
        }
        (*NPCInfo).aimErrorDebounceTime = level.time + Q_irand(250, 2000);
    }

    if doYaw != QFALSE {
        // decay yaw diff
        let mut diff = AngleDelta((*(*NPC).client).ps.viewangles[YAW], targetYaw);

        if diff != 0.0 {
            exact = QFALSE;

            decay = 60.0 + 80.0;
            decay *= 50.0 / 1000.0; //msec
            if diff < 0.0 {
                diff += decay;
                if diff > 0.0 {
                    diff = 0.0;
                }
            } else {
                diff -= decay;
                if diff < 0.0 {
                    diff = 0.0;
                }
            }
        }

        // add yaw error based on NPCInfo->aim value
        let error = (*NPCInfo).lastAimErrorYaw;

        /*
        if(Q_irand(0, 1))
        {
            error *= -1;
        }
        */

        ucmd.angles[YAW] = ANGLE2SHORT(targetYaw + diff + error) - (*client).ps.delta_angles[YAW];
    }

    if doPitch != QFALSE {
        // decay pitch diff
        let mut diff = AngleDelta((*(*NPC).client).ps.viewangles[PITCH], targetPitch);
        if diff != 0.0 {
            exact = QFALSE;

            decay = 60.0 + 80.0;
            decay *= 50.0 / 1000.0; //msec
            if diff < 0.0 {
                diff += decay;
                if diff > 0.0 {
                    diff = 0.0;
                }
            } else {
                diff -= decay;
                if diff < 0.0 {
                    diff = 0.0;
                }
            }
        }

        let error = (*NPCInfo).lastAimErrorPitch;

        ucmd.angles[PITCH] =
            ANGLE2SHORT(targetPitch + diff + error) - (*client).ps.delta_angles[PITCH];
    }

    ucmd.angles[ROLL] =
        ANGLE2SHORT((*(*NPC).client).ps.viewangles[ROLL]) - (*client).ps.delta_angles[ROLL];

    exact
}

/*
static void NPC_UpdateShootAngles (vec3_t angles, qboolean doPitch, qboolean doYaw )

Does update angles on shootAngles
*/
/// `void NPC_UpdateShootAngles( vec3_t angles, qboolean doPitch, qboolean doYaw )`
/// (NPC_utils.c:740).
///
/// Decays `NPCInfo->shootAngles[]` toward `angles[]` (rate scaled by `stats.aim`), updating
/// the NPC's separate shoot-aim track. *"FIXME: shoot angles either not set right or not
/// used!"* No oracle — mutates `NPCInfo`.
///
/// # Safety
/// `angles` must point to a readable `vec3_t`; the `NPCInfo` global must be set.
pub unsafe fn NPC_UpdateShootAngles(angles: *mut vec3_t, doPitch: qboolean, doYaw: qboolean) {
    let mut decay: f32;
    let mut targetPitch: f32 = 0.0;
    let mut targetYaw: f32 = 0.0;

    let angles = &*angles;
    if doPitch != QFALSE {
        targetPitch = angles[PITCH];
    }
    if doYaw != QFALSE {
        targetYaw = angles[YAW];
    }

    if doYaw != QFALSE {
        // decay yaw error
        let mut error = AngleDelta((*NPCInfo).shootAngles[YAW], targetYaw);
        if error != 0.0 {
            decay = 60.0 + 80.0 * (*NPCInfo).stats.aim as f32;
            decay *= 100.0 / 1000.0; //msec
            if error < 0.0 {
                error += decay;
                if error > 0.0 {
                    error = 0.0;
                }
            } else {
                error -= decay;
                if error < 0.0 {
                    error = 0.0;
                }
            }
        }
        (*NPCInfo).shootAngles[YAW] = targetYaw + error;
    }

    if doPitch != QFALSE {
        // decay pitch error
        let mut error = AngleDelta((*NPCInfo).shootAngles[PITCH], targetPitch);
        if error != 0.0 {
            decay = 60.0 + 80.0 * (*NPCInfo).stats.aim as f32;
            decay *= 100.0 / 1000.0; //msec
            if error < 0.0 {
                error += decay;
                if error > 0.0 {
                    error = 0.0;
                }
            } else {
                error -= decay;
                if error < 0.0 {
                    error = 0.0;
                }
            }
        }
        (*NPCInfo).shootAngles[PITCH] = targetPitch + error;
    }
}

/*
-------------------------
NPC_FacePosition
-------------------------
*/
/// `qboolean NPC_FacePosition( vec3_t position, qboolean doPitch )` (NPC_utils.c:1497).
///
/// Turn the file-global `NPC` to face world `position`: pick a muzzle spot (creature/ATST/
/// GalakMech special-cases), set `desiredYaw/Pitch` from the direction, drive
/// [`NPC_UpdateAngles`], then report whether the resulting yaw (and, if `doPitch`, pitch) is
/// within `VALID_ATTACK_CONE`. The ATST-enemy jitter (`flrand`+`sin`) is carried verbatim.
/// No oracle — reads/writes the NPC globals and `ucmd`.
///
/// # Safety
/// `position` must point to a readable `vec3_t`; the `NPC`/`NPCInfo`/`client` globals must be set.
pub unsafe fn NPC_FacePosition(position: *mut vec3_t, doPitch: qboolean) -> qboolean {
    let mut muzzle: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let yawDelta: f32;
    let mut facing: qboolean = QTRUE;

    let position = &*position;

    //Get the positions
    if !(*NPC).client.is_null()
        && ((*(*NPC).client).NPC_class == CLASS_RANCOR || (*(*NPC).client).NPC_class == CLASS_WAMPA)
    // || NPC->client->NPC_class == CLASS_SAND_CREATURE) )
    {
        CalcEntitySpot(NPC, SPOT_ORIGIN, addr_of_mut!(muzzle));
        muzzle[2] += (*NPC).r.maxs[2] * 0.75;
    } else if !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == CLASS_GALAKMECH {
        CalcEntitySpot(NPC, SPOT_WEAPON, addr_of_mut!(muzzle));
    } else {
        CalcEntitySpot(NPC, SPOT_HEAD_LEAN, addr_of_mut!(muzzle)); //SPOT_HEAD
    }

    //Find the desired angles
    GetAnglesForDirection(&muzzle, position, &mut angles);

    (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW]);
    (*NPCInfo).desiredPitch = AngleNormalize360(angles[PITCH]);

    if !(*NPC).enemy.is_null()
        && !(*(*NPC).enemy).client.is_null()
        && (*(*(*NPC).enemy).client).NPC_class == CLASS_ATST
    {
        // FIXME: this is kind of dumb, but it was the easiest way to get it to look sort of ok
        (*NPCInfo).desiredYaw +=
            flrand(-5.0, 5.0) + (((level.time as f32) * 0.004) as f64).sin() as f32 * 7.0;
        (*NPCInfo).desiredPitch += flrand(-2.0, 2.0);
    }
    //Face that yaw
    NPC_UpdateAngles(QTRUE, QTRUE);

    //Find the delta between our goal and our current facing
    yawDelta = AngleNormalize360(
        (*NPCInfo).desiredYaw - SHORT2ANGLE(ucmd.angles[YAW] + (*client).ps.delta_angles[YAW]),
    );

    //See if we are facing properly
    if yawDelta.abs() > VALID_ATTACK_CONE {
        facing = QFALSE;
    }

    if doPitch != QFALSE {
        //Find the delta between our goal and our current facing
        let currentAngles = SHORT2ANGLE(ucmd.angles[PITCH] + (*client).ps.delta_angles[PITCH]);
        let pitchDelta = (*NPCInfo).desiredPitch - currentAngles;

        //See if we are facing properly
        if pitchDelta.abs() > VALID_ATTACK_CONE {
            facing = QFALSE;
        }
    }

    facing
}

/*
-------------------------
NPC_FaceEntity
-------------------------
*/
/// `qboolean NPC_FaceEntity( gentity_t *ent, qboolean doPitch )` (NPC_utils.c:1561).
///
/// Resolve `ent`'s head spot, then face it via [`NPC_FacePosition`]. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the NPC globals must be set.
pub unsafe fn NPC_FaceEntity(ent: *mut gentity_t, doPitch: qboolean) -> qboolean {
    let mut entPos: vec3_t = [0.0; 3];

    //Get the positions
    CalcEntitySpot(ent, SPOT_HEAD_LEAN, addr_of_mut!(entPos));

    NPC_FacePosition(addr_of_mut!(entPos), doPitch)
}

/*
-------------------------
NPC_FaceEnemy
-------------------------
*/
/// `qboolean NPC_FaceEnemy( qboolean doPitch )` (NPC_utils.c:1577).
///
/// Face the file-global `NPC`'s current enemy via [`NPC_FaceEntity`]; `qfalse` if there is
/// no `NPC` or no enemy. No oracle.
///
/// # Safety
/// The `NPC` global must be set (may be null — checked).
pub unsafe fn NPC_FaceEnemy(doPitch: qboolean) -> qboolean {
    if NPC.is_null() {
        return QFALSE;
    }

    if (*NPC).enemy.is_null() {
        return QFALSE;
    }

    NPC_FaceEntity((*NPC).enemy, doPitch)
}

/*
-------------------------
NPC_CheckCanAttackExt
-------------------------
*/
/// `qboolean NPC_CheckCanAttackExt( void )` (NPC_utils.c:1594).
///
/// Decide whether the file-global `NPC` may fire: not `SCF_DONT_FIRE`, able to face the enemy
/// ([`NPC_FaceEnemy`]), and with a clear shot ([`NPC_ClearShot`]). No oracle.
///
/// # Safety
/// The `NPC`/`NPCInfo` globals must be set.
pub unsafe fn NPC_CheckCanAttackExt() -> qboolean {
    //We don't want them to shoot
    if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 {
        return QFALSE;
    }

    //Turn to face
    if NPC_FaceEnemy(QTRUE) == QFALSE {
        return QFALSE;
    }

    //Must have a clear line of sight to the target
    if NPC_ClearShot((*NPC).enemy) == QFALSE {
        return QFALSE;
    }

    QTRUE
}

/*
-------------------------
NPC_TargetVisible
-------------------------
*/
/// `qboolean NPC_TargetVisible( gentity_t *ent )` (NPC_utils.c:1201).
///
/// Is `ent` a visible target for the file-global `NPC`: within `stats.visrange`, inside the
/// NPC's `hfov`/`vfov` cone ([`InFOV`]), and with a clear line of sight ([`NPC_ClearLOS4`]).
/// No oracle — reads NPC globals and drives the LOS/FOV trace helpers.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the `NPC`/`NPCInfo` globals must be set.
pub unsafe fn NPC_TargetVisible(ent: *mut gentity_t) -> qboolean {
    //Make sure we're in a valid range
    if DistanceSquared(&(*ent).r.currentOrigin, &(*NPC).r.currentOrigin)
        > ((*NPCInfo).stats.visrange * (*NPCInfo).stats.visrange)
    {
        return QFALSE;
    }

    //Check our FOV
    if InFOV(ent, NPC, (*NPCInfo).stats.hfov, (*NPCInfo).stats.vfov) == QFALSE {
        return QFALSE;
    }

    //Check for sight
    if NPC_ClearLOS4(ent) == QFALSE {
        return QFALSE;
    }

    QTRUE
}

/*
-------------------------
NPC_FindNearestEnemy
-------------------------
*/
/// `int NPC_FindNearestEnemy( gentity_t *ent )` (NPC_utils.c:1252).
///
/// Box-query the entities within `ent`'s `stats.visrange`, then return the entity number of
/// the nearest one that is a [`NPC_ValidEnemy`] and [`NPC_TargetVisible`] (excluding `ent`
/// itself), or `-1`. No oracle — drives `trap_EntitiesInBox` and the validity/visibility
/// helpers over the `NPC` globals.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the `NPCInfo` global must be set.
pub unsafe fn NPC_FindNearestEnemy(ent: *mut gentity_t) -> c_int {
    let mut iradiusEnts: [c_int; MAX_RADIUS_ENTS] = [0; MAX_RADIUS_ENTS];
    let mut radEnt: *mut gentity_t;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut nearestEntID: c_int = -1;
    let mut nearestDist: f32 = WORLD_SIZE as f32 * WORLD_SIZE as f32;
    let mut distance: f32;
    let numEnts;
    let mut numChecks: c_int = 0;

    //Setup the bbox to search in
    for i in 0..3 {
        mins[i] = (*ent).r.currentOrigin[i] - (*NPCInfo).stats.visrange;
        maxs[i] = (*ent).r.currentOrigin[i] + (*NPCInfo).stats.visrange;
    }

    //Get a number of entities in a given space
    numEnts = trap::EntitiesInBox(&mins, &maxs, &mut iradiusEnts);

    for i in 0..numEnts {
        radEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .offset(iradiusEnts[i as usize] as isize);
        //Don't consider self
        if radEnt == ent {
            continue;
        }

        //Must be valid
        if NPC_ValidEnemy(radEnt) == QFALSE {
            continue;
        }

        numChecks += 1;
        //Must be visible
        if NPC_TargetVisible(radEnt) == QFALSE {
            continue;
        }

        distance = DistanceSquared(&(*ent).r.currentOrigin, &(*radEnt).r.currentOrigin);

        //Found one closer to us
        if distance < nearestDist {
            nearestEntID = (*radEnt).s.number;
            nearestDist = distance;
        }
    }
    let _ = numChecks;

    nearestEntID
}

/*
-------------------------
NPC_PickEnemyExt
-------------------------
*/
/// `gentity_t *NPC_PickEnemyExt( qboolean checkAlerts )` (NPC_utils.c:1308).
///
/// Pick a new enemy for the file-global `NPC`: prefer the nearest valid/visible enemy
/// ([`NPC_FindNearestEnemy`]); failing that, if `checkAlerts`, react to the highest-priority
/// `AEL_DISCOVERED`+ alert ([`NPC_CheckAlertEvents`]) — attacking the player directly,
/// inheriting a teammate's enemy, etc. The Hazard-Team `TEAM_STARFLEET` block is `/* */`'d
/// out in the source and omitted. No oracle — reads `level.alertEvents`/`g_entities` and the
/// NPC globals.
///
/// # Safety
/// The `NPC`/`NPCInfo` globals must be set.
pub unsafe fn NPC_PickEnemyExt(checkAlerts: qboolean) -> *mut gentity_t {
    //Check for Hazard Team status and remove this check
    /*
    if ( NPC->client->playerTeam != TEAM_STARFLEET )
    {
        //If we've found the player, return it
        if ( NPC_FindPlayer() )
            return &g_entities[0];
    }
    */

    //If we've asked for the closest enemy
    let entID = NPC_FindNearestEnemy(NPC);

    //If we have a valid enemy, use it
    if entID >= 0 {
        return (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(entID as isize);
    }

    if checkAlerts != QFALSE {
        let alertEvent = NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QTRUE, AEL_DISCOVERED);

        //There is an event to look at
        if alertEvent >= 0 {
            let event = addr_of_mut!(level.alertEvents[alertEvent as usize]);

            //Don't pay attention to our own alerts
            if (*event).owner == NPC {
                return null_mut();
            }

            if (*event).level >= AEL_DISCOVERED {
                //If it's the player, attack him
                if (*event).owner
                    == (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(0)
                {
                    return (*event).owner;
                }

                //If it's on our team, then take its enemy as well
                if !(*(*event).owner).client.is_null()
                    && (*(*(*event).owner).client).playerTeam == (*(*NPC).client).playerTeam
                {
                    return (*(*event).owner).enemy;
                }
            }
        }
    }

    null_mut()
}

/*
-------------------------
NPC_FindPlayer
-------------------------
*/
/// `qboolean NPC_FindPlayer( void )` (NPC_utils.c:1362).
///
/// Is the player (`g_entities[0]`) a visible target for the file-global `NPC`? Thin wrapper
/// over [`NPC_TargetVisible`]. No oracle.
///
/// # Safety
/// The `NPC`/`NPCInfo` globals must be set.
pub unsafe fn NPC_FindPlayer() -> qboolean {
    NPC_TargetVisible((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(0))
}

/*
-------------------------
NPC_FindEnemy
-------------------------
*/
/// `qboolean NPC_FindEnemy( qboolean checkAlerts )` (NPC_utils.c:1413).
///
/// Acquire/keep an enemy for the file-global `NPC`: bail while `confusionTime` is active,
/// keep the current enemy if still [`NPC_ValidEnemy`], else [`NPC_PickEnemyExt`] a new one and
/// adopt it via [`G_SetEnemy`]. The `SVF_IGNORE_ENEMIES`/`SVF_LOCKEDENEMY` flag branches are
/// `rwwFIXME` stubs (`if (0)` / commented) and carried verbatim; the two
/// [`NPC_CheckPlayerDistance`] calls are MP-moot (`qfalse`). No oracle.
///
/// # Safety
/// The `NPC`/`NPCInfo` globals must be set.
pub unsafe fn NPC_FindEnemy(checkAlerts: qboolean) -> qboolean {
    let newenemy: *mut gentity_t;

    //We're ignoring all enemies for now
    //if( NPC->svFlags & SVF_IGNORE_ENEMIES )
    if false {
        //rwwFIXMEFIXME: support for flag
        G_ClearEnemy(NPC);
        return QFALSE;
    }

    //we can't pick up any enemies for now
    if (*NPCInfo).confusionTime > level.time {
        return QFALSE;
    }

    //Don't want a new enemy
    //rwwFIXMEFIXME: support for locked enemy
    //if ( ( ValidEnemy( NPC->enemy ) ) && ( NPC->svFlags & SVF_LOCKEDENEMY ) )
    //	return qtrue;

    //See if the player is closer than our current enemy
    if NPC_CheckPlayerDistance() != QFALSE {
        return QTRUE;
    }

    //Otherwise, turn off the flag
    //	NPC->svFlags &= ~SVF_LOCKEDENEMY;
    //See if the player is closer than our current enemy
    if (*(*NPC).client).NPC_class != CLASS_RANCOR
        && (*(*NPC).client).NPC_class != CLASS_WAMPA
        //&& NPC->client->NPC_class != CLASS_SAND_CREATURE
        && NPC_CheckPlayerDistance() != QFALSE
    {
        //rancors, wampas & sand creatures don't care if player is closer, they always go with closest
        return QTRUE;
    }

    //If we've gotten here alright, then our target it still valid
    if NPC_ValidEnemy((*NPC).enemy) != QFALSE {
        return QTRUE;
    }

    newenemy = NPC_PickEnemyExt(checkAlerts);

    //if we found one, take it as the enemy
    if NPC_ValidEnemy(newenemy) != QFALSE {
        G_SetEnemy(NPC, newenemy);
        return QTRUE;
    }

    QFALSE
}

/*
-------------------------
NPC_CheckEnemyExt
-------------------------
*/
/// `qboolean NPC_CheckEnemyExt( qboolean checkAlerts )` (NPC_utils.c:1475).
///
/// Thin wrapper: the enemy-check-debounce body is `/* */`'d out in the source, so this just
/// delegates to [`NPC_FindEnemy`]. No oracle.
///
/// # Safety
/// The `NPC`/`NPCInfo` globals must be set.
pub unsafe fn NPC_CheckEnemyExt(checkAlerts: qboolean) -> qboolean {
    //Make sure we're ready to think again
    /*
        if ( NPCInfo->enemyCheckDebounceTime > level.time )
            return qfalse;

        //Get our next think time
        NPCInfo->enemyCheckDebounceTime = level.time + NPC_GetCheckDelta();

        //Attempt to find an enemy
        return NPC_FindEnemy();
    */
    NPC_FindEnemy(checkAlerts)
}
