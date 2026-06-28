//! Slice of `NPC_AI_Seeker.c` — the floating "seeker" attack droid's behavior
//! state (also the chassis Boba Fett's seekers reuse). The precache, the pain
//! handler, the height-maintenance hover, the strafe evasion, the missile-fire
//! helper, the enemy-search scan, the hunt / ranged / attack-decision chain, the
//! follow-owner behavior, and the top-level behavior-state dispatcher are all
//! ported.
//!
//! Ported here so far: `NPC_Seeker_Precache` (NPC_AI_Seeker.c:26),
//! `NPC_Seeker_Pain` (:34), `Seeker_MaintainHeight` (:49),
//! `Seeker_Strafe` (:151), `Seeker_Hunt` (:242), `Seeker_Fire` (:290),
//! `Seeker_Ranged` (:320), `Seeker_Attack` (:350), `Seeker_FindEnemy` (:383),
//! `Seeker_FollowOwner` (:439), `NPC_BSSeeker_Default` (:523).
//!
//! This file is fully ported.

#![allow(non_snake_case)] // C function names (`NPC_Seeker_Pain`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::c_char;
use core::ffi::c_int;
use core::ptr::addr_of;
use core::ptr::null_mut;

use crate::codemp::game::b_public_h::{NPCAI_CUSTOM_GRAVITY, SCF_CHASE_ENEMIES, SPOT_HEAD};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_public::{
    MASK_SHOT, MASK_SOLID, MOD_BLASTER, MOD_FALLING, MOD_TELEFRAG, MOD_UNKNOWN,
};
use crate::codemp::game::bg_weapons_h::WP_BLASTER;
use crate::codemp::game::g_combat::G_Damage;
use crate::codemp::game::g_local::{
    gentity_t, CON_DISCONNECTED, DAMAGE_DEATH_KNOCKBACK, DAMAGE_NO_PROTECTION,
};
use crate::codemp::game::g_main::{g_entities, g_spskill, level};
use crate::codemp::game::g_missile::CreateMissile;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{G_EffectIndex, G_PlayEffectID, G_Sound, G_SoundIndex};
use crate::codemp::game::npc::{
    ucmd, NPCInfo, RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals, NPC,
};
use crate::codemp::game::npc_ai_jedi::Boba_FireDecide;
use crate::codemp::game::npc_move::{NPC_GetMoveDirection, NPC_MoveToGoal};
use crate::codemp::game::npc_reactions::NPC_Pain;
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, AngleVectors, DistanceHorizontalSquared, VectorMA, VectorNormalize,
    VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{crandom, random};
use crate::codemp::game::q_shared_h::{trace_t, vec3_t, CHAN_AUTO, ENTITYNUM_NONE, MAX_GENTITIES};
use crate::codemp::game::surfaceflags_h::CONTENTS_LIGHTSABER;
use crate::codemp::game::teams_h::{CLASS_BOBAFETT, CLASS_SEEKER, NPCTEAM_NEUTRAL};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

const VELOCITY_DECAY: f32 = 0.7;

const SEEKER_STRAFE_VEL: f32 = 100.0;
const SEEKER_STRAFE_DIS: f32 = 200.0;
const SEEKER_UPWARD_PUSH: f32 = 32.0;

const SEEKER_SEEK_RADIUS: f32 = 1024.0;

const MIN_DISTANCE: f32 = 80.0;
const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

const SEEKER_FORWARD_BASE_SPEED: f32 = 10.0;
const SEEKER_FORWARD_MULTIPLIER: f32 = 2.0;

// NPC_AI_Seeker.c also defines MIN_MELEE_RANGE / MIN_MELEE_RANGE_SQR, used only
// by code not yet landed in this file.

//------------------------------------
pub unsafe fn NPC_Seeker_Precache() {
    G_SoundIndex("sound/chars/seeker/misc/fire.wav");
    G_SoundIndex("sound/chars/seeker/misc/hiss.wav");
    G_EffectIndex("env/small_explode");
}

//------------------------------------
pub unsafe extern "C" fn NPC_Seeker_Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    if (*(*self_).NPC).aiFlags & NPCAI_CUSTOM_GRAVITY == 0 {
        //void G_Damage( gentity_t *targ, gentity_t *inflictor, gentity_t *attacker, vec3_t dir, vec3_t point, int damage, int dflags, int mod, int hitLoc=HL_NONE );
        G_Damage(
            self_,
            null_mut(),
            null_mut(),
            addr_of!(vec3_origin) as *mut vec3_t,
            addr_of!(vec3_origin) as *mut vec3_t,
            999,
            0,
            MOD_FALLING,
        );
    }

    SaveNPCGlobals();
    SetNPCGlobals(self_);
    Seeker_Strafe();
    RestoreNPCGlobals();
    NPC_Pain(self_, attacker, damage);
}

//------------------------------------
pub unsafe fn Seeker_MaintainHeight() {
    let mut dif: f32;

    // Update our angles regardless
    NPC_UpdateAngles(QTRUE, QTRUE);

    // If we have an enemy, we should try to hover at or a little below enemy eye level
    if !(*NPC).enemy.is_null() {
        if TIMER_Done(NPC, c"heightChange".as_ptr()) != QFALSE {
            let mut difFactor: f32;

            TIMER_Set(NPC, c"heightChange".as_ptr(), Q_irand(1000, 3000));

            // Find the height difference
            dif = ((*(*NPC).enemy).r.currentOrigin[2]
                + flrand(
                    (*(*NPC).enemy).r.maxs[2] / 2.0,
                    (*(*NPC).enemy).r.maxs[2] + 8.0,
                ))
                - (*NPC).r.currentOrigin[2];

            difFactor = 1.0;
            if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                if TIMER_Done(NPC, c"flameTime".as_ptr()) != QFALSE {
                    difFactor = 10.0;
                }
            }

            // cap to prevent dramatic height shifts
            if dif.abs() > 2.0 * difFactor {
                if dif.abs() > 24.0 * difFactor {
                    dif = if dif < 0.0 {
                        -24.0 * difFactor
                    } else {
                        24.0 * difFactor
                    };
                }

                (*(*NPC).client).ps.velocity[2] = ((*(*NPC).client).ps.velocity[2] + dif) / 2.0;
            }
            if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                (*(*NPC).client).ps.velocity[2] *= flrand(0.85, 3.0);
            }
        }
    } else {
        let goal: *mut gentity_t;

        if !(*NPCInfo).goalEntity.is_null()
        // Is there a goal?
        {
            goal = (*NPCInfo).goalEntity;
        } else {
            goal = (*NPCInfo).lastGoalEntity;
        }
        if !goal.is_null() {
            dif = (*goal).r.currentOrigin[2] - (*NPC).r.currentOrigin[2];

            if dif.abs() > 24.0 {
                ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
            } else if (*(*NPC).client).ps.velocity[2] != 0.0 {
                (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

                if (*(*NPC).client).ps.velocity[2].abs() < 2.0 {
                    (*(*NPC).client).ps.velocity[2] = 0.0;
                }
            }
        }
    }

    // Apply friction
    if (*(*NPC).client).ps.velocity[0] != 0.0 {
        (*(*NPC).client).ps.velocity[0] *= VELOCITY_DECAY;

        if (*(*NPC).client).ps.velocity[0].abs() < 1.0 {
            (*(*NPC).client).ps.velocity[0] = 0.0;
        }
    }

    if (*(*NPC).client).ps.velocity[1] != 0.0 {
        (*(*NPC).client).ps.velocity[1] *= VELOCITY_DECAY;

        if (*(*NPC).client).ps.velocity[1].abs() < 1.0 {
            (*(*NPC).client).ps.velocity[1] = 0.0;
        }
    }
}

//------------------------------------
pub unsafe fn Seeker_Strafe() {
    let side: c_int;
    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let tr: trace_t;

    if random() > 0.7 || (*NPC).enemy.is_null() || (*(*NPC).enemy).client.is_null() {
        // Do a regular style strafe
        AngleVectors(
            &(*(*NPC).client).renderInfo.eyeAngles,
            None,
            Some(&mut right),
            None,
        );

        // Pick a random strafe direction, then check to see if doing a strafe would be
        //	reasonably valid
        side = if rand() & 1 != 0 { -1 } else { 1 };
        VectorMA(
            &(*NPC).r.currentOrigin,
            SEEKER_STRAFE_DIS * side as f32,
            &right,
            &mut end,
        );

        let tr2 = trap::Trace(
            &(*NPC).r.currentOrigin,
            &vec3_origin,
            &vec3_origin,
            &end,
            (*NPC).s.number,
            MASK_SOLID,
        );

        // Close enough
        if tr2.fraction > 0.9 {
            let mut vel = SEEKER_STRAFE_VEL;
            let mut upPush = SEEKER_UPWARD_PUSH;
            if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
                G_Sound(NPC, CHAN_AUTO, G_SoundIndex("sound/chars/seeker/misc/hiss"));
            } else {
                vel *= 3.0;
                upPush *= 4.0;
            }
            let curvel = (*(*NPC).client).ps.velocity;
            VectorMA(
                &curvel,
                vel * side as f32,
                &right,
                &mut (*(*NPC).client).ps.velocity,
            );
            // Add a slight upward push
            (*(*NPC).client).ps.velocity[2] += upPush;

            (*NPCInfo).standTime = (*addr_of!(level)).time + 1000 + (random() * 500.0) as c_int;
        }
    } else {
        let mut stDis: f32;

        // Do a strafe to try and keep on the side of their enemy
        AngleVectors(
            &(*(*(*NPC).enemy).client).renderInfo.eyeAngles,
            Some(&mut dir),
            Some(&mut right),
            None,
        );

        // Pick a random side
        side = if rand() & 1 != 0 { -1 } else { 1 };
        stDis = SEEKER_STRAFE_DIS;
        if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
            stDis *= 2.0;
        }
        VectorMA(
            &(*(*NPC).enemy).r.currentOrigin,
            stDis * side as f32,
            &right,
            &mut end,
        );

        // then add a very small bit of random in front of/behind the player action
        let end_copy = end;
        VectorMA(&end_copy, crandom() as f32 * 25.0, &dir, &mut end);

        tr = trap::Trace(
            &(*NPC).r.currentOrigin,
            &vec3_origin,
            &vec3_origin,
            &end,
            (*NPC).s.number,
            MASK_SOLID,
        );

        // Close enough
        if tr.fraction > 0.9 {
            let dis: f32;
            let mut upPush: f32;

            VectorSubtract(&tr.endpos, &(*NPC).r.currentOrigin, &mut dir);
            dir[2] *= 0.25; // do less upward change
            dis = VectorNormalize(&mut dir);

            // Try to move the desired enemy side
            let curvel = (*(*NPC).client).ps.velocity;
            VectorMA(&curvel, dis, &dir, &mut (*(*NPC).client).ps.velocity);

            upPush = SEEKER_UPWARD_PUSH;
            if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
                G_Sound(NPC, CHAN_AUTO, G_SoundIndex("sound/chars/seeker/misc/hiss"));
            } else {
                upPush *= 4.0;
            }

            // Add a slight upward push
            (*(*NPC).client).ps.velocity[2] += upPush;

            (*NPCInfo).standTime = (*addr_of!(level)).time + 2500 + (random() * 500.0) as c_int;
        }
    }
}

//------------------------------------
pub unsafe fn Seeker_Hunt(visible: qboolean, advance: qboolean) {
    let mut distance: f32 = 0.0;
    let speed: f32;
    let mut forward: vec3_t = [0.0; 3];

    NPC_FaceEnemy(QTRUE);

    // If we're not supposed to stand still, pursue the player
    if (*NPCInfo).standTime < (*addr_of!(level)).time {
        // Only strafe when we can see the player
        if visible != QFALSE {
            Seeker_Strafe();
            return;
        }
    }

    // If we don't want to advance, stop here
    if advance == QFALSE {
        return;
    }

    // Only try and navigate if the player is visible
    if visible == QFALSE {
        // Move towards our goal
        (*NPCInfo).goalEntity = (*NPC).enemy;
        (*NPCInfo).goalRadius = 24;

        // Get our direction from the navigator if we can't see our target
        if NPC_GetMoveDirection(&mut forward, &mut distance) == QFALSE {
            return;
        }
    } else {
        VectorSubtract(
            &(*(*NPC).enemy).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &mut forward,
        );
        distance = VectorNormalize(&mut forward);
    }
    let _ = distance;

    speed = SEEKER_FORWARD_BASE_SPEED
        + SEEKER_FORWARD_MULTIPLIER * (*addr_of!(g_spskill)).integer as f32;
    let curvel = (*(*NPC).client).ps.velocity;
    VectorMA(&curvel, speed, &forward, &mut (*(*NPC).client).ps.velocity);
}

//------------------------------------
pub unsafe fn Seeker_Fire() {
    let mut dir: vec3_t = [0.0; 3];
    let mut enemy_org: vec3_t = [0.0; 3];
    let mut muzzle: vec3_t = [0.0; 3];
    let missile: *mut gentity_t;

    CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut enemy_org);
    VectorSubtract(&enemy_org, &(*NPC).r.currentOrigin, &mut dir);
    VectorNormalize(&mut dir);

    // move a bit forward in the direction we shall shoot in so that the bolt doesn't poke out the other side of the seeker
    VectorMA(&(*NPC).r.currentOrigin, 15.0, &dir, &mut muzzle);

    missile = CreateMissile(&mut muzzle, &dir, 1000.0, 10000, NPC, QFALSE);

    G_PlayEffectID(
        G_EffectIndex("blaster/muzzle_flash"),
        &(*NPC).r.currentOrigin,
        &dir,
    );

    (*missile).classname = c"blaster".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BLASTER;

    (*missile).damage = 5;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_BLASTER;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    if (*NPC).r.ownerNum < ENTITYNUM_NONE {
        (*missile).r.ownerNum = (*NPC).r.ownerNum;
    }
}

//------------------------------------
pub unsafe fn Seeker_Ranged(visible: qboolean, advance: qboolean) {
    if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
        if (*NPC).count > 0 {
            if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE
            // Attack?
            {
                TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(250, 2500));
                Seeker_Fire();
                (*NPC).count -= 1;
            }
        } else {
            // out of ammo, so let it die...give it a push up so it can fall more and blow up on impact
            //		NPC->client->ps.gravity = 900;
            //		NPC->svFlags &= ~SVF_CUSTOM_GRAVITY;
            //		NPC->client->ps.velocity[2] += 16;
            G_Damage(NPC, NPC, NPC, null_mut(), null_mut(), 999, 0, MOD_UNKNOWN);
        }
    }

    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        Seeker_Hunt(visible, advance);
    }
}

//------------------------------------
pub unsafe fn Seeker_Attack() {
    let distance: f32;
    let visible: qboolean;
    let mut advance: qboolean;

    // Always keep a good height off the ground
    Seeker_MaintainHeight();

    // Rate our distance to the target, and our visibilty
    distance = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin);
    visible = NPC_ClearLOS4((*NPC).enemy);
    advance = (distance > MIN_DISTANCE_SQR) as qboolean;

    if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
        advance = (distance > (200.0 * 200.0)) as qboolean;
    }

    // If we cannot see our target, move to see it
    if visible == QFALSE {
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            Seeker_Hunt(visible, advance);
            return;
        }
    }

    Seeker_Ranged(visible, advance);
}

//------------------------------------
pub unsafe fn Seeker_FindEnemy() {
    let numFound: c_int;
    let mut dis: f32;
    let mut bestDis: f32 = SEEKER_SEEK_RADIUS * SEEKER_SEEK_RADIUS + 1.0;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut entityList: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut ent: *mut gentity_t;
    let mut best: *mut gentity_t = null_mut();
    let mut i: c_int;

    VectorSet(
        &mut maxs,
        SEEKER_SEEK_RADIUS,
        SEEKER_SEEK_RADIUS,
        SEEKER_SEEK_RADIUS,
    );
    VectorScale(&maxs, -1.0, &mut mins);

    numFound = trap::EntitiesInBox(&mins, &maxs, &mut entityList);

    i = 0;
    while i < numFound {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .offset(entityList[i as usize] as isize);

        if (*ent).s.number == (*NPC).s.number
            || (*ent).client.is_null() //&& || !ent->NPC
            || (*ent).health <= 0
            || (*ent).inuse == QFALSE
        {
            i += 1;
            continue;
        }

        if (*(*ent).client).playerTeam == (*(*NPC).client).playerTeam
            || (*(*ent).client).playerTeam == NPCTEAM_NEUTRAL
        // don't attack same team or bots
        {
            i += 1;
            continue;
        }

        // try to find the closest visible one
        if NPC_ClearLOS4(ent) == QFALSE {
            i += 1;
            continue;
        }

        dis = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*ent).r.currentOrigin);

        if dis <= bestDis {
            bestDis = dis;
            best = ent;
        }

        i += 1;
    }

    if !best.is_null() {
        // used to offset seekers around a circle so they don't occupy the same spot.  This is not a fool-proof method.
        (*NPC).random = random() * 6.3; // roughly 2pi

        (*NPC).enemy = best;
    }
}

//------------------------------------
pub unsafe fn Seeker_FollowOwner() {
    let dis: f32;
    let mut minDistSqr: f32;
    let mut pt: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut owner: *mut gentity_t =
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset((*NPC).s.owner as isize);

    Seeker_MaintainHeight();

    if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
        owner = (*NPC).enemy;
    }
    if owner.is_null() || owner == NPC || (*owner).client.is_null() {
        return;
    }
    //rwwFIXMEFIXME: Care about all clients not just 0
    dis = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*owner).r.currentOrigin);

    minDistSqr = MIN_DISTANCE_SQR;

    if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
        if TIMER_Done(NPC, c"flameTime".as_ptr()) != QFALSE {
            minDistSqr = 200.0 * 200.0;
        }
    }

    if dis < minDistSqr {
        // generally circle the player closely till we take an enemy..this is our target point
        if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
            pt[0] = (*owner).r.currentOrigin[0]
                + (((*addr_of!(level)).time as f32 * 0.001 + (*NPC).random) as f64).cos() as f32
                    * 250.0;
            pt[1] = (*owner).r.currentOrigin[1]
                + (((*addr_of!(level)).time as f32 * 0.001 + (*NPC).random) as f64).sin() as f32
                    * 250.0;
            if (*(*NPC).client).jetPackTime < (*addr_of!(level)).time {
                pt[2] = (*NPC).r.currentOrigin[2] - 64.0;
            } else {
                pt[2] = (*owner).r.currentOrigin[2] + 200.0;
            }
        } else {
            pt[0] = (*owner).r.currentOrigin[0]
                + (((*addr_of!(level)).time as f32 * 0.001 + (*NPC).random) as f64).cos() as f32
                    * 56.0;
            pt[1] = (*owner).r.currentOrigin[1]
                + (((*addr_of!(level)).time as f32 * 0.001 + (*NPC).random) as f64).sin() as f32
                    * 56.0;
            pt[2] = (*owner).r.currentOrigin[2] + 40.0;
        }

        VectorSubtract(&pt, &(*NPC).r.currentOrigin, &mut dir);
        let curvel = (*(*NPC).client).ps.velocity;
        VectorMA(&curvel, 0.8, &dir, &mut (*(*NPC).client).ps.velocity);
    } else {
        if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
            if TIMER_Done(NPC, c"seekerhiss".as_ptr()) != QFALSE {
                TIMER_Set(
                    NPC,
                    c"seekerhiss".as_ptr(),
                    1000 + (random() * 1000.0) as c_int,
                );
                G_Sound(NPC, CHAN_AUTO, G_SoundIndex("sound/chars/seeker/misc/hiss"));
            }
        }

        // Hey come back!
        (*NPCInfo).goalEntity = owner;
        (*NPCInfo).goalRadius = 32;
        NPC_MoveToGoal(QTRUE);
        (*NPC).parent = owner;
    }

    if (*NPCInfo).enemyCheckDebounceTime < (*addr_of!(level)).time {
        // check twice a second to find a new enemy
        Seeker_FindEnemy();
        (*NPCInfo).enemyCheckDebounceTime = (*addr_of!(level)).time + 500;
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

//------------------------------------
pub unsafe fn NPC_BSSeeker_Default() {
    /*
    if ( in_camera )
    {
        if ( NPC->client->NPC_class != CLASS_BOBAFETT )
        {
            // cameras make me commit suicide....
            G_Damage( NPC, NPC, NPC, NULL, NULL, 999, 0, MOD_UNKNOWN );
        }
    }
    */
    //N/A for MP.
    if (*NPC).r.ownerNum < ENTITYNUM_NONE as c_int {
        let owner: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(0);
        if (*owner).health <= 0
            || (!(*owner).client.is_null() && (*(*owner).client).pers.connected == CON_DISCONNECTED)
        {
            //owner is dead or gone
            //remove me
            G_Damage(
                NPC,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                10000,
                DAMAGE_NO_PROTECTION,
                MOD_TELEFRAG as c_int,
            );
            return;
        }
    }

    if (*NPC).random == 0.0 {
        // used to offset seekers around a circle so they don't occupy the same spot.  This is not a fool-proof method.
        (*NPC).random = random() * 6.3; // roughly 2pi
    }

    if !(*NPC).enemy.is_null() && (*(*NPC).enemy).health != 0 && (*(*NPC).enemy).inuse != QFALSE {
        if (*(*NPC).client).NPC_class != CLASS_BOBAFETT
            && ((*(*NPC).enemy).s.number == 0
                || (!(*(*NPC).enemy).client.is_null()
                    && (*(*(*NPC).enemy).client).NPC_class == CLASS_SEEKER))
        {
            //hacked to never take the player as an enemy, even if the player shoots at it
            (*NPC).enemy = null_mut();
        } else {
            Seeker_Attack();
            if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                Boba_FireDecide();
            }
            return;
        }
    }

    // In all other cases, follow the player and look for enemies to take on
    Seeker_FollowOwner();
}
