//! Port of `g_exphysics.c` — the "Expensive Physics" custom origin-mover. A lightweight
//! per-frame integrator for free-flying props (thrown/dropped objects) that moves an entity
//! purely by its custom `epVelocity`/`epGravFactor` entity fields (client-side origin
//! smoothing hides the choppiness). Optionally clips against a set of ghoul2 skeleton bolts so
//! a ragdoll-ish object embeds believably instead of interpenetrating the world.
//!
//! Single-function file. No oracle: every branch is driven by engine traces
//! (`trap_Trace`/`trap_LinkEntity`/`trap_G2API_GetBoltMatrix`) and the entity's own `touch`
//! callback — same convention as [`crate::codemp::game::g_utils::G_FreeEntity`].

#![allow(non_snake_case)] // C function name (`G_RunExPhys`) kept verbatim
#![allow(non_upper_case_globals)] // C macro name kept verbatim

use core::ffi::c_int;
use core::ptr::{addr_of, null_mut};

use crate::codemp::game::bg_public::BG_GiveMeVectorFromMatrix;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::{g_entities, level};
use crate::codemp::game::g_utils::{G_FreeEntity, G_SetOrigin};
use crate::codemp::game::q_math::{
    VectorAdd, VectorClear, VectorCopy, VectorMA, VectorNormalize, VectorScale, VectorSet,
    VectorSubtract,
};
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, trace_t, vec3_t, ENTITYNUM_NONE, ORIGIN, PITCH, ROLL, YAW,
};
use crate::ffi::types::QFALSE;
use crate::trap;

const MAX_GRAVITY_PULL: f32 = 512.0;

/// `void G_RunExPhys( gentity_t *ent, float gravity, float mass, float bounce, qboolean autoKill,
/// int *g2Bolts, int numG2Bolts )` (g_exphysics.c:21). Run physics on the object (purely
/// origin-related) using the custom `epVelocity` entity-state value. Origin smoothing on the
/// client is expected to compensate for choppy movement. No oracle (engine trace/link/g2 calls
/// + the entity's `touch` callback drive every branch).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `g2Bolts` must be null or point to `numG2Bolts`
/// valid bolt indices.
pub unsafe fn G_RunExPhys(
    ent: *mut gentity_t,
    gravity: f32,
    mass: f32,
    bounce: f32,
    autoKill: i32,
    g2Bolts: *mut c_int,
    numG2Bolts: c_int,
) {
    let velScaling: f32 = 0.1;
    let mut vTotal: f32;

    debug_assert!(mass <= 1.0 && mass >= 0.01);

    if gravity != 0.0 {
        //factor it in before we do anything.
        let mut ground: vec3_t = [0.0; 3];
        VectorCopy(&(*ent).r.currentOrigin, &mut ground);
        ground[2] -= 0.1;

        let tr = trap::Trace(
            &(*ent).r.currentOrigin,
            &(*ent).r.mins,
            &(*ent).r.maxs,
            &ground,
            (*ent).s.number,
            (*ent).clipmask,
        );

        if tr.fraction == 1.0 {
            (*ent).s.groundEntityNum = ENTITYNUM_NONE;
        } else {
            (*ent).s.groundEntityNum = tr.entityNum as c_int;
        }

        if (*ent).s.groundEntityNum == ENTITYNUM_NONE {
            (*ent).epGravFactor += gravity;

            if (*ent).epGravFactor > MAX_GRAVITY_PULL {
                //cap it off if needed
                (*ent).epGravFactor = MAX_GRAVITY_PULL;
            }

            (*ent).epVelocity[2] -= (*ent).epGravFactor;
        } else {
            //if we're sitting on something then reset the gravity factor.
            (*ent).epGravFactor = 0.0;
        }
    }

    if (*ent).epVelocity[0] == 0.0 && (*ent).epVelocity[1] == 0.0 && (*ent).epVelocity[2] == 0.0 {
        //nothing to do if we have no velocity even after gravity.
        if let Some(touch) = (*ent).touch {
            //call touch if we're in something
            let mut tr = trap::Trace(
                &(*ent).r.currentOrigin,
                &(*ent).r.mins,
                &(*ent).r.maxs,
                &(*ent).r.currentOrigin,
                (*ent).s.number,
                (*ent).clipmask,
            );
            if tr.startsolid != 0 || tr.allsolid != 0 {
                touch(
                    ent,
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize),
                    &mut tr,
                );
            }
        }
        return;
    }

    //get the projected origin based on velocity.
    let mut projectedOrigin: vec3_t = [0.0; 3];
    VectorMA(
        &(*ent).r.currentOrigin,
        velScaling,
        &(*ent).epVelocity,
        &mut projectedOrigin,
    );

    let epVel = (*ent).epVelocity;
    VectorScale(&epVel, 1.0 - mass, &mut (*ent).epVelocity); //scale it down based on mass

    let mut vNorm: vec3_t = [0.0; 3];
    VectorCopy(&(*ent).epVelocity, &mut vNorm);
    vTotal = VectorNormalize(&mut vNorm);

    if vTotal < 1.0 && (*ent).s.groundEntityNum != ENTITYNUM_NONE {
        //we've pretty much stopped moving anyway, just clear it out then.
        VectorClear(&mut (*ent).epVelocity);
        (*ent).epGravFactor = 0.0;
        trap::LinkEntity(ent);
        return;
    }

    if !(*ent).ghoul2.is_null() && !g2Bolts.is_null() {
        //Have we been passed a bolt index array to clip against points on the skeleton?
        let mut tMins: vec3_t = [0.0; 3];
        let mut tMaxs: vec3_t = [0.0; 3];
        let mut trajDif: vec3_t = [0.0; 3];
        let mut gbmAngles: vec3_t = [0.0; 3];
        let mut bestCollision = trace_t::default();
        let mut collisionRootPos: vec3_t = [0.0; 3];
        let mut hasFirstCollision = false;
        let mut i: c_int = 0;

        //Maybe we could use a trap call and get the default radius for the bone specified,
        //but this will do at least for now.
        VectorSet(&mut tMins, -3.0, -3.0, -3.0);
        VectorSet(&mut tMaxs, 3.0, 3.0, 3.0);

        gbmAngles[PITCH] = 0.0;
        gbmAngles[ROLL] = 0.0;
        gbmAngles[YAW] = (*ent).s.apos.trBase[YAW];

        //Get the difference relative to the entity origin and projected origin, to add to each bolt position.
        VectorSubtract(&(*ent).r.currentOrigin, &projectedOrigin, &mut trajDif);

        while i < numG2Bolts {
            let mut matrix = mdxaBone_t::default();
            let mut boneOrg: vec3_t = [0.0; 3];
            let mut projectedBoneOrg: vec3_t = [0.0; 3];

            //Get the position of the actual bolt for this frame
            trap::G2API_GetBoltMatrix(
                (*ent).ghoul2,
                0,
                *g2Bolts.add(i as usize),
                &mut matrix,
                &gbmAngles,
                &(*ent).r.currentOrigin,
                (*addr_of!(level)).time,
                null_mut(),
                &(*ent).modelScale,
            );
            BG_GiveMeVectorFromMatrix(&matrix, ORIGIN, &mut boneOrg);

            //Now add the projected positional difference into the result
            VectorAdd(&boneOrg, &trajDif, &mut projectedBoneOrg);

            let tr = trap::Trace(
                &boneOrg,
                &tMins,
                &tMaxs,
                &projectedBoneOrg,
                (*ent).s.number,
                (*ent).clipmask,
            );

            if tr.fraction != 1.0 || tr.startsolid != 0 || tr.allsolid != 0 {
                //we've hit something
                //Store the "deepest" collision we have
                if !hasFirstCollision {
                    //don't have one yet so just use this one
                    bestCollision = tr;
                    VectorCopy(&boneOrg, &mut collisionRootPos);
                    hasFirstCollision = true;
                } else if tr.allsolid != 0 && bestCollision.allsolid == 0 {
                    //If the whole trace is solid then this one is deeper
                    bestCollision = tr;
                    VectorCopy(&boneOrg, &mut collisionRootPos);
                } else if tr.startsolid != 0
                    && bestCollision.startsolid == 0
                    && bestCollision.allsolid == 0
                {
                    //Next deepest is if it's startsolid
                    bestCollision = tr;
                    VectorCopy(&boneOrg, &mut collisionRootPos);
                } else if bestCollision.startsolid == 0
                    && bestCollision.allsolid == 0
                    && tr.fraction < bestCollision.fraction
                {
                    //and finally, if neither is startsolid/allsolid, but the new one has a smaller fraction, then it's closer to an impact point so we will use it
                    bestCollision = tr;
                    VectorCopy(&boneOrg, &mut collisionRootPos);
                }
            }

            i += 1;
        }

        if hasFirstCollision {
            //at least one bolt collided
            //We'll get the offset between the collided bolt and endpos, then trace there
            //from the origin so that our desired position becomes that point.
            VectorSubtract(&collisionRootPos, &bestCollision.endpos, &mut trajDif);

            VectorAdd(&(*ent).r.currentOrigin, &trajDif, &mut projectedOrigin);
        }
    }

    //If we didn't collide with any bolts projectedOrigin will still be the original desired
    //projected position so all is well. If we did then projectedOrigin will be modified
    //to provide us with a relative position which does not place the bolt in a solid.
    let mut tr = trap::Trace(
        &(*ent).r.currentOrigin,
        &(*ent).r.mins,
        &(*ent).r.maxs,
        &projectedOrigin,
        (*ent).s.number,
        (*ent).clipmask,
    );

    if tr.startsolid != 0 || tr.allsolid != 0 {
        //can't go anywhere from here
        // #ifdef _DEBUG Com_Printf("ExPhys object in solid (%i)\n", ent->s.number) — debug-only, omitted (retail).
        if autoKill != QFALSE as i32 {
            (*ent).think = Some(G_FreeEntity);
            (*ent).nextthink = (*addr_of!(level)).time;
        }
        return;
    }

    //Go ahead and set it to the trace endpoint regardless of what it hit
    G_SetOrigin(ent, &tr.endpos);
    trap::LinkEntity(ent);

    if tr.fraction == 1.0 {
        //Nothing was in the way.
        return;
    }

    if bounce != 0.0 {
        vTotal *= bounce; //scale it by bounce

        VectorScale(&tr.plane.normal, vTotal, &mut vNorm); //scale the trace plane normal by the bounce factor

        if vNorm[2] > 0.0 {
            (*ent).epGravFactor -= vNorm[2] * (1.0 - mass); //The lighter it is the more gravity will be reduced by bouncing vertically.
            if (*ent).epGravFactor < 0.0 {
                (*ent).epGravFactor = 0.0;
            }
        }

        //call touch first so we can check velocity upon impact if we want
        if tr.entityNum as c_int != ENTITYNUM_NONE {
            if let Some(touch) = (*ent).touch {
                //then call the touch function
                touch(
                    ent,
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize),
                    &mut tr,
                );
            }
        }

        let epVel = (*ent).epVelocity;
        VectorAdd(&epVel, &vNorm, &mut (*ent).epVelocity); //add it into the existing velocity.
    } else {
        //if no bounce, kill when it hits something.
        (*ent).epVelocity[0] = 0.0;
        (*ent).epVelocity[1] = 0.0;

        if gravity == 0.0 {
            (*ent).epVelocity[2] = 0.0;
        }
    }
}
