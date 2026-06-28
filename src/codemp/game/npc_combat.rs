//! Slice of `NPC_combat.c` — the NPC combat / enemy-targeting layer. Opened
//! bottom-up at the NPC-AI wall (deliberate user override, cycle 57): `G_SetEnemy`
//! (NPC_combat.c:349) and its in-file leaf helpers are the seam that gates
//! `WP_SaberStartMissileBlockCheck` and the broader Jedi-combat path. The strategy
//! is to land the genuinely-portable leaves first and cascade up toward `G_SetEnemy`.
//!
//! `ai_main.c` is **not** opened by this work — a callee scan confirmed none of these
//! functions reach the bot-AI core. The NPC track proper (`NPC_AI_Jedi.c`,
//! `NPC_senses.c`, `NPC.c`) is not yet ported.
//!
//! Ported here so far: `G_ClearEnemy` (NPC_combat.c:17), `G_AngerAlert` (:47),
//! `G_TeamEnemy` (:67), `G_AttackDelay` (:117), `G_ForceSaberOn` (:312),
//! `G_SetEnemy` (:349), `ChangeWeapon` (:570), `NPC_ChangeWeapon` (:843),
//! `EntIsGlass` (:1113), `G_AimSet` (:3130).

#![allow(non_snake_case)] // C function names (`G_TeamEnemy`) kept verbatim
#![allow(non_upper_case_globals)] // C global `enemyVisibility` kept verbatim

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::ai_h::SQUAD_TRANSITION;
use crate::codemp::game::b_local_h::{
    CPF_DUCK, CPF_FLEE, CPF_INVESTIGATE, CPF_SQUAD, CP_APPROACH_ENEMY, CP_AVOID, CP_AVOID_ENEMY,
    CP_CLEAR, CP_COVER, CP_DUCK, CP_FLANK, CP_FLEE, CP_HAS_ROUTE, CP_HORZ_DIST_COLL,
    CP_INVESTIGATE, CP_NEAREST, CP_NO_PVS, CP_RETREAT, CP_SQUAD,
};
use crate::codemp::game::b_public_h::{
    visibility_t, BS_DEFAULT, BS_HUNT_AND_KILL, BS_INVESTIGATE, BS_PATROL, BS_STAND_AND_SHOOT,
    NPCAI_BURST_WEAPON, RANK_CREWMAN, RANK_LT, SCF_ALT_FIRE, SCF_DONT_FIRE, SCF_NO_GROUPS,
    SPOT_CHEST, SPOT_HEAD, SPOT_ORIGIN, SPOT_WEAPON, VIS_360, VIS_FOV, VIS_SHOOT, VIS_UNKNOWN,
};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_public::{
    EF_NODRAW, ET_ITEM, EV_ANGER1, EV_ANGER3, IT_WEAPON, MASK_SHOT, STAT_WEAPONS, TEAM_BLUE,
    TEAM_FREE, TEAM_RED, TEAM_SPECTATOR, WEAPON_DROPPING, WEAPON_FIRING, WEAPON_IDLE,
    WEAPON_RAISING, WEAPON_READY,
};
use crate::codemp::game::bg_weapons::weaponData;
use crate::codemp::game::bg_weapons_h::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_PISTOL, WP_DEMP2, WP_DISRUPTOR, WP_EMPLACED_GUN,
    WP_FLECHETTE, WP_NONE, WP_REPEATER, WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON, WP_THERMAL,
    WP_TURRET,
};
use crate::codemp::game::g_combat::G_AlertTeam;
use crate::codemp::game::g_items::{Add_Ammo, CheckItemCanBePickedUpByNPC};
use crate::codemp::game::g_local::{gentity_t, FL_NOTARGET, MAX_COMBAT_POINTS};
use crate::codemp::game::g_main::{debugNPCAI, g_entities, g_spskill, level, Com_Printf};
use crate::codemp::game::g_nav::{
    NAV_ClearPathToPoint, NAV_FindClosestWaypointForPoint2, NAV_GetNearestNode, NPC_SetMoveGoal,
    NF_CLEAR_PATH, NODE_NONE, WAYPOINT_NONE,
};
use crate::codemp::game::g_public_h::{BSET_ANGER, Q3_INFINITE, SVF_GLASS_BRUSH};
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Exists, TIMER_Set};
use crate::codemp::game::g_utils::{vtos, G_CheckInSolid, G_FreeEntity, G_SetOrigin, G_Sound};
use crate::codemp::game::npc::{client, ucmd, NPCInfo, NPC};
use crate::codemp::game::npc_ai_default::NPC_LostEnemyDecideChase;
use crate::codemp::game::npc_ai_jedi::NPC_Jedi_RateNewEnemy;
use crate::codemp::game::npc_misc::{Debug_Printf, DEBUG_LEVEL_INFO};
use crate::codemp::game::npc_senses::{InVisrange, NPC_CheckVisibility};
// `G_AddVoiceEvent`'s canonical C home is NPC_sounds.c:23 → defined in `npc_sounds.rs`.
// Re-exported here since the combat-talk gate is consumed throughout this file and the
// wider NPC subsystem imports it from `npc_combat`.
pub(crate) use crate::codemp::game::npc_sounds::G_AddVoiceEvent;
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, G_ActivateBehavior, NPC_AimWiggle, NPC_CheckLookTarget, NPC_ClearLOS,
    NPC_ClearLOS3, NPC_ClearLookTarget, NPC_UpdateFiringAngles,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, vectoangles, AngleVectors, DistanceHorizontalSquared, DistanceSquared,
    DotProduct, VectorClear, VectorCopy, VectorLength, VectorLengthSquared, VectorMA,
    VectorNormalize, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{random, Q_stricmp};
use crate::codemp::game::q_shared_h::{
    trace_t, vec3_t, BUTTON_ATTACK, CHAN_AUTO, ENTITYNUM_NONE, PITCH, WORLD_SIZE, YAW,
};
use crate::codemp::game::teams_h::{
    CLASS_ATST, CLASS_GALAKMECH, CLASS_IMPERIAL, CLASS_IMPWORKER, CLASS_INTERROGATOR, CLASS_JAN,
    CLASS_JAWA, CLASS_LANDO, CLASS_MARK1, CLASS_MARK2, CLASS_MINEMONSTER, CLASS_MURJJ,
    CLASS_PRISONER, CLASS_PROBE, CLASS_REBEL, CLASS_REELO, CLASS_REMOTE, CLASS_SEEKER,
    CLASS_SENTRY, CLASS_STORMTROOPER, CLASS_SWAMPTROOPER, CLASS_TRANDOSHAN, CLASS_UGNAUGHT,
    NPCTEAM_ENEMY, NPCTEAM_FREE, NPCTEAM_NEUTRAL, NPCTEAM_PLAYER,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `#define CHECK_360 2` (b_local.h:166) — `NPC_CheckVisibility` flag bits. These b_local.h
/// macros are also defined privately in `npc_senses.rs`; kept file-local here too (the
/// header isn't ported as a shared module yet).
const CHECK_360: c_int = 2;
/// `#define CHECK_FOV 4` (b_local.h:167).
const CHECK_FOV: c_int = 4;
/// `#define CHECK_VISRANGE 16` (b_local.h:169).
const CHECK_VISRANGE: c_int = 16;

/// `visibility_t enemyVisibility;` (NPC.c:38, `extern` in b_local.h:67) — the cached
/// visibility classification of the current NPC's enemy for this think. Its true home is
/// `NPC.c`; until that file is ported it lives here, its only Stage-1 consumer
/// (`NPC_CheckPossibleEnemy`). REVISIT: re-home into `npc.rs` alongside `NPC`/`NPCInfo`/
/// `client`/`ucmd` when `NPC.c` lands.
pub static mut enemyVisibility: visibility_t = VIS_UNKNOWN;

/// `qboolean G_TeamEnemy( gentity_t *self )` (NPC_combat.c:67).
///
/// Returns `qtrue` if any living teammate (same `playerTeam`) currently has an enemy
/// that is *not* on `self`'s team — i.e. "is my team already fighting someone?". Walks
/// the whole `g_entities[1 .. level.num_entities]` array (slot 0 is the world; the C's
/// FIXME notes a teammate linked-list would be cheaper). A clientless `self`, or one on
/// `TEAM_FREE`, returns `qfalse`; an NPC flagged `SCF_NO_GROUPS` is a loner and also
/// returns `qfalse`. No oracle (reads the process-global entity array + `level`);
/// verified by review against the C.
///
/// # Safety
/// `self` must point to a valid `gentity_t`; `g_entities`/`level` must be initialised.
pub unsafe fn G_TeamEnemy(self_: *mut gentity_t) -> qboolean {
    if (*self_).client.is_null() || (*(*self_).client).playerTeam == TEAM_FREE {
        return QFALSE;
    }
    if !self_.is_null()
        && !(*self_).NPC.is_null()
        && ((*(*self_).NPC).scriptFlags & SCF_NO_GROUPS) != 0
    {
        // I'm not a team playa...
        return QFALSE;
    }

    let num_entities = (*addr_of!(level)).num_entities;
    for i in 1..num_entities {
        let ent: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize);

        if ent == self_ {
            continue;
        }
        if (*ent).health <= 0 {
            continue;
        }
        if (*ent).client.is_null() {
            continue;
        }
        if (*(*ent).client).playerTeam != (*(*self_).client).playerTeam {
            // ent is not on my team
            continue;
        }
        if !(*ent).enemy.is_null() {
            // they have an enemy
            if (*(*ent).enemy).client.is_null()
                || (*(*(*ent).enemy).client).playerTeam != (*(*self_).client).playerTeam
            {
                // the ent's enemy is either a normal ent or is a player/NPC not on my team
                return QTRUE;
            }
        }
    }

    QFALSE
}

/// `void G_AttackDelay( gentity_t *self, gentity_t *enemy )` (NPC_combat.c:117).
///
/// On acquiring `enemy`, delay the NPC's first shot (and roam move) based on how far
/// `self` is currently facing away from the enemy, then bias that delay by NPC class,
/// weapon, team, and `g_spskill` difficulty, clamping the result. Sets the
/// `"attackDelay"` and `"roamTime"` timers. Several classes/weapons (droids, sniper,
/// melee, turrets, …) `return` early with no delay. No-op unless `self` is a client NPC
/// that has an `enemy`. The commented-out class/weapon cases and the disgusting-hack
/// Imperial-holster block (both `/* */` in the C) are dropped.
///
/// # Safety
/// `self`/`self->client`/`self->NPC` and `enemy` must be valid; `g_spskill` registered.
pub unsafe fn G_AttackDelay(self_: *mut gentity_t, enemy: *mut gentity_t) {
    if !enemy.is_null() && !(*self_).client.is_null() && !(*self_).NPC.is_null() {
        // delay their attack based on how far away they're facing from enemy
        let mut fwd: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        VectorSubtract(
            &(*(*self_).client).renderInfo.eyePoint,
            &(*enemy).r.currentOrigin,
            &mut dir,
        ); // purposely backwards
        VectorNormalize(&mut dir);
        AngleVectors(
            &(*(*self_).client).renderInfo.eyeAngles,
            Some(&mut fwd),
            None,
            None,
        );

        // initial: from 1000ms delay on hard to 2000ms delay on easy
        let mut attDelay = (4 - (*addr_of!(g_spskill)).integer) * 500;
        if (*(*self_).client).playerTeam == NPCTEAM_PLAYER {
            // invert
            attDelay = 2000 - attDelay;
        }
        // add up to 4000ms delay if they're facing away
        attDelay += ((DotProduct(&fwd, &dir) + 1.0) * 2000.0).floor() as c_int;

        // Now modify the delay based on NPC_class, weapon, and team
        // NOTE: attDelay should be somewhere between 1000 to 6000 milliseconds
        match (*(*self_).client).NPC_class {
            // they give orders and hang back
            CLASS_IMPERIAL => attDelay += Q_irand(500, 1500),
            // stormtroopers shoot sooner
            CLASS_STORMTROOPER => {
                if (*(*self_).NPC).rank >= RANK_LT {
                    // officers shoot even sooner
                    attDelay -= Q_irand(500, 1500);
                } else {
                    // normal stormtroopers don't have as fast reflexes as officers
                    attDelay -= Q_irand(0, 1000);
                }
            }
            // shoot very quickly?  What about guys in water?
            CLASS_SWAMPTROOPER => attDelay -= Q_irand(1000, 2000),
            // they panic, don't fire right away
            CLASS_IMPWORKER => attDelay += Q_irand(1000, 2500),
            CLASS_TRANDOSHAN => attDelay -= Q_irand(500, 1500),
            CLASS_JAN | CLASS_LANDO | CLASS_PRISONER | CLASS_REBEL => {
                attDelay -= Q_irand(500, 1500)
            }
            CLASS_GALAKMECH | CLASS_ATST => attDelay -= Q_irand(1000, 2000),
            CLASS_REELO | CLASS_UGNAUGHT | CLASS_JAWA => return,
            CLASS_MINEMONSTER | CLASS_MURJJ => return,
            CLASS_INTERROGATOR | CLASS_PROBE | CLASS_MARK1 | CLASS_MARK2 | CLASS_SENTRY => return,
            CLASS_REMOTE | CLASS_SEEKER => return,
            _ => {}
        }

        match (*self_).s.weapon {
            WP_NONE | WP_SABER => return,
            WP_BRYAR_PISTOL => {}
            WP_BLASTER => {
                if ((*(*self_).NPC).scriptFlags & SCF_ALT_FIRE) != 0 {
                    // rapid-fire blasters
                    attDelay += Q_irand(0, 500);
                } else {
                    // regular blaster
                    attDelay -= Q_irand(0, 500);
                }
            }
            WP_BOWCASTER => attDelay += Q_irand(0, 500),
            WP_REPEATER => {
                if ((*(*self_).NPC).scriptFlags & SCF_ALT_FIRE) == 0 {
                    // rapid-fire blasters
                    attDelay += Q_irand(0, 500);
                }
            }
            WP_FLECHETTE => attDelay += Q_irand(500, 1500),
            WP_ROCKET_LAUNCHER => attDelay += Q_irand(500, 1500),
            WP_DISRUPTOR => return,  // sniper's don't delay?
            WP_THERMAL => return,    // grenade-throwing has a built-in delay
            WP_STUN_BATON => return, // Any ol' melee attack
            WP_EMPLACED_GUN => return,
            WP_TURRET => return, // turret guns
            _ => {}
        }

        if (*(*self_).client).playerTeam == NPCTEAM_PLAYER {
            // clamp it
            if attDelay > 2000 {
                attDelay = 2000;
            }
        }

        // don't shoot right away
        if attDelay > 4000 + ((2 - (*addr_of!(g_spskill)).integer) * 3000) {
            attDelay = 4000 + ((2 - (*addr_of!(g_spskill)).integer) * 3000);
        }
        TIMER_Set(self_, c"attackDelay".as_ptr(), attDelay);
        // don't move right away either
        if attDelay > 4000 {
            attDelay = 4000 - Q_irand(500, 1500);
        } else {
            attDelay -= Q_irand(500, 1500);
        }

        TIMER_Set(self_, c"roamTime".as_ptr(), attDelay);
    }
}

/// `void G_ForceSaberOn( gentity_t *ent )` (NPC_combat.c:312).
///
/// TEMP HACK (the C's word) to switch a holstered lightsaber back on: clears
/// `ps.saberHolstered` and plays each blade's ignition sound. No-ops if the saber is
/// in flight, already on, or the current weapon isn't `WP_SABER`. Called by
/// `G_SetEnemy` when an NPC with `health > 0` first acquires an enemy.
///
/// # Safety
/// `ent` and `ent->client` must be valid; `ent->client->saber[0..2]` must be initialised.
pub unsafe fn G_ForceSaberOn(ent: *mut gentity_t) {
    if (*(*ent).client).ps.saberInFlight != QFALSE {
        // alright, can't turn it on now in any case, so forget it.
        return;
    }

    if (*(*ent).client).ps.saberHolstered == 0 {
        // it's already on!
        return;
    }

    if (*(*ent).client).ps.weapon != WP_SABER {
        // This probably should never happen. But if it does we'll just return without complaining.
        return;
    }

    // Well then, turn it on.
    (*(*ent).client).ps.saberHolstered = 0;

    if (*(*ent).client).saber[0].soundOn != 0 {
        G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[0].soundOn);
    }
    if (*(*ent).client).saber[1].soundOn != 0 {
        G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[1].soundOn);
    }
}

/// `void G_AimSet( gentity_t *self, int aim )` (NPC_combat.c:3130).
///
/// Set an NPC's `currentAim` and arm the `"aimDebounce"` timer (the window before the
/// aim is allowed to drift again), scaled by `g_spskill` difficulty. No-op for a
/// non-NPC `self`. The commented-out alternate debounce timing in the C is dropped.
///
/// # Safety
/// `self` must be valid; `g_spskill` must be registered.
pub unsafe fn G_AimSet(self_: *mut gentity_t, aim: c_int) {
    if !(*self_).NPC.is_null() {
        (*(*self_).NPC).currentAim = aim;

        let debounce = 500 + (3 - (*addr_of!(g_spskill)).integer) * 100;
        TIMER_Set(
            self_,
            c"aimDebounce".as_ptr(),
            Q_irand(debounce, debounce + 1000),
        );
    }
}

/// `void G_ClearEnemy( gentity_t *self )` (NPC_combat.c:17).
///
/// Drop `self`'s current enemy: if that enemy is the one we're looking at, clear the
/// look target; if it's the NPC's `goalEntity`, clear that too; then null `self->enemy`.
/// Always runs `NPC_CheckLookTarget` first (it ticks/expires the existing look target).
/// No oracle (mutates the entity-state graph); verified by review against the C.
///
/// # Safety
/// `self` must point to a valid `gentity_t`; `self->client`/`self->NPC` may be NULL.
pub unsafe fn G_ClearEnemy(self_: *mut gentity_t) {
    NPC_CheckLookTarget(self_);

    if !(*self_).enemy.is_null() {
        if !(*self_).client.is_null()
            && (*(*self_).client).renderInfo.lookTarget == (*(*self_).enemy).s.number
        {
            NPC_ClearLookTarget(self_);
        }

        if !(*self_).NPC.is_null() && (*self_).enemy == (*(*self_).NPC).goalEntity {
            (*(*self_).NPC).goalEntity = core::ptr::null_mut();
        }
        // FIXME: set last enemy?
    }

    (*self_).enemy = core::ptr::null_mut();
}

const ANGER_ALERT_RADIUS: f32 = 512.0;
const ANGER_ALERT_SOUND_RADIUS: f32 = 256.0;

/// `void G_AngerAlert( gentity_t *self )` (NPC_combat.c:47).
///
/// "I just got angry — wake my teammates." Bails for a loner NPC (`SCF_NO_GROUPS`) or
/// while the `"interrogating"` timer is still running, then alerts the team via
/// `G_AlertTeam` (g_combat). No oracle (entity-state + timers); verified by review against the C.
///
/// # Safety
/// `self` must point to a valid `gentity_t`; `self->NPC` may be NULL.
pub unsafe fn G_AngerAlert(self_: *mut gentity_t) {
    if !self_.is_null()
        && !(*self_).NPC.is_null()
        && ((*(*self_).NPC).scriptFlags & SCF_NO_GROUPS) != 0
    {
        // I'm not a team playa...
        return;
    }
    if TIMER_Done(self_, c"interrogating".as_ptr()) == QFALSE {
        // I'm interrogating, don't wake everyone else up yet... FIXME: this may never
        // wake everyone else up, though!
        return;
    }
    // FIXME: hmm.... with all the other new alerts now, is this still neccesary or even a
    // good idea...?
    G_AlertTeam(
        self_,
        (*self_).enemy,
        ANGER_ALERT_RADIUS,
        ANGER_ALERT_SOUND_RADIUS,
    );
}

/// `void G_SetEnemy( gentity_t *self, gentity_t *enemy )` (NPC_combat.c:349).
///
/// The enemy-acquisition keystone: validates `enemy`, applies confusion/charm/notarget
/// gates, then on a *first* enemy turns the saber on, clears any stale enemy, seeds anger
/// (anger script or yell + alert), sets initial aim error and an attack delay; on a
/// *subsequent* enemy it just swaps in the new one. `NPC_Jedi_RateNewEnemy` (npc_ai_jedi)
/// and `G_AlertTeam` via `G_AngerAlert` (g_combat) are wired to their ports; only
/// `G_AddVoiceEvent` (NPC_sounds.c) is still a guarded no-op stub. The `#ifdef _DEBUG` self-assert, the
/// commented-out disguise/`ChangeWeapon` Imperial-holster hack, and the
/// `forcePushTime`/`rwwFIXME` notes are dropped/kept faithfully as in the C. No oracle
/// (deep entity-state mutation); verified by review against the C.
///
/// # Safety
/// `self` and `enemy` must point to valid `gentity_t`s; `self->client`/`self->NPC` may be
/// NULL (the early no-NPC path handles that). `g_spskill`/`level` must be initialised.
pub unsafe fn G_SetEnemy(self_: *mut gentity_t, enemy: *mut gentity_t) {
    let mut event: c_int = 0;

    // Must be valid
    if enemy.is_null() {
        return;
    }

    // Must be valid
    if (*enemy).inuse == QFALSE {
        return;
    }

    // Don't take the enemy if in notarget
    if ((*enemy).flags & FL_NOTARGET) != 0 {
        return;
    }

    if (*self_).NPC.is_null() {
        (*self_).enemy = enemy;
        return;
    }

    if (*(*self_).NPC).confusionTime > (*addr_of!(level)).time {
        // can't pick up enemies if confused
        return;
    }

    // (#ifdef _DEBUG self-assert in the C is dropped)

    //	if ( enemy->client && enemy->client->playerTeam == TEAM_DISGUISE )
    //	{//unmask the player
    //		enemy->client->playerTeam = TEAM_PLAYER;
    //	}

    if !(*self_).client.is_null()
        && !(*self_).NPC.is_null()
        && !(*enemy).client.is_null()
        && (*(*enemy).client).playerTeam == (*(*self_).client).playerTeam
    {
        // Probably a damn script!
        if (*(*self_).NPC).charmedTime > (*addr_of!(level)).time {
            // Probably a damn script!
            return;
        }
    }

    if !(*self_).NPC.is_null()
        && !(*self_).client.is_null()
        && (*(*self_).client).ps.weapon == WP_SABER
    {
        // when get new enemy, set a base aggression based on what that enemy is using,
        // how far they are, etc.
        NPC_Jedi_RateNewEnemy(self_, enemy);
    }

    // NOTE: this is not necessarily true!
    // self->NPC->enemyLastSeenTime = level.time;

    if (*self_).enemy.is_null() {
        // TEMP HACK: turn on our saber
        if (*self_).health > 0 {
            G_ForceSaberOn(self_);
        }

        // FIXME: Have to do this to prevent alert cascading
        G_ClearEnemy(self_);
        (*self_).enemy = enemy;

        // Special case- if player is being hunted by his own people, set their enemy
        // team correctly
        if (*(*self_).client).playerTeam == NPCTEAM_PLAYER && (*enemy).s.number == 0 {
            (*(*self_).client).enemyTeam = NPCTEAM_PLAYER;
        }

        // If have an anger script, run that instead of yelling
        if G_ActivateBehavior(self_, BSET_ANGER) != QFALSE {
        } else if !(*self_).client.is_null()
            && !(*enemy).client.is_null()
            && (*(*self_).client).playerTeam != (*(*enemy).client).playerTeam
        {
            // FIXME: Use anger when entire team has no enemy.
            //		 Basically, you're first one to notice enemies
            // if ( self->forcePushTime < level.time ) // not currently being pushed
            if true {
                // rwwFIXMEFIXME: Set forcePushTime
                if G_TeamEnemy(self_) == QFALSE {
                    // team did not have an enemy previously
                    event = Q_irand(EV_ANGER1, EV_ANGER3);
                }
            }

            if event != 0 {
                // yell
                G_AddVoiceEvent(self_, event, 2000);
            }
        }

        if (*self_).s.weapon == WP_BLASTER
            || (*self_).s.weapon == WP_REPEATER
            || (*self_).s.weapon == WP_THERMAL
            // rwwFIXMEFIXME: Blaster pistol useable by npcs?  (WP_BLASTER_PISTOL)
            || (*self_).s.weapon == WP_BOWCASTER
        {
            // Hmm, how about sniper and bowcaster?
            // When first get mad, aim is bad
            // Hmm, base on game difficulty, too?  Rank?
            if (*(*self_).client).playerTeam == NPCTEAM_PLAYER {
                G_AimSet(
                    self_,
                    Q_irand(
                        (*(*self_).NPC).stats.aim - (5 * (*addr_of!(g_spskill)).integer),
                        (*(*self_).NPC).stats.aim - (*addr_of!(g_spskill)).integer,
                    ),
                );
            } else {
                let mut min_err = 3;
                let mut max_err = 12;
                if (*(*self_).client).NPC_class == CLASS_IMPWORKER {
                    min_err = 15;
                    max_err = 30;
                } else if (*(*self_).client).NPC_class == CLASS_STORMTROOPER
                    && !(*self_).NPC.is_null()
                    && (*(*self_).NPC).rank <= RANK_CREWMAN
                {
                    min_err = 5;
                    max_err = 15;
                }

                G_AimSet(
                    self_,
                    Q_irand(
                        (*(*self_).NPC).stats.aim
                            - (max_err * (3 - (*addr_of!(g_spskill)).integer)),
                        (*(*self_).NPC).stats.aim
                            - (min_err * (3 - (*addr_of!(g_spskill)).integer)),
                    ),
                );
            }
        }

        // Alert anyone else in the area
        if Q_stricmp(c"desperado".as_ptr(), (*self_).NPC_type) != 0
            && Q_stricmp(c"paladin".as_ptr(), (*self_).NPC_type) != 0
        {
            // special holodeck enemies exception
            if (*(*self_).client).ps.fd.forceGripBeingGripped < (*addr_of!(level)).time as f32 {
                // gripped people can't call for help
                G_AngerAlert(self_);
            }
        }

        // Stormtroopers don't fire right away!
        G_AttackDelay(self_, enemy);

        // rwwFIXMEFIXME: Deal with this some other way.
        // (the commented-out Imperial-weapon-holster ChangeWeapon hack in the C is dropped)
        return;
    }

    // Otherwise, just picking up another enemy

    if event != 0 {
        G_AddVoiceEvent(self_, event, 2000);
    }

    // Take the enemy
    G_ClearEnemy(self_);
    (*self_).enemy = enemy;
}

/// `qboolean EntIsGlass( gentity_t *check )` (NPC_combat.c:1113).
///
/// True if `check` is breakable glass: a `"func_breakable"` with `count == 1` and
/// `health <= 100`. Used by the shot-through-glass / line-of-fire checks to let NPCs
/// shoot through breakable panes. No oracle (reads an opaque `gentity_t`); verified by
/// review against the C.
///
/// # Safety
/// `check` must point to a valid `gentity_t`; `check->classname` may be NULL.
pub unsafe fn EntIsGlass(check: *mut gentity_t) -> qboolean {
    if !(*check).classname.is_null()
        && Q_stricmp(c"func_breakable".as_ptr(), (*check).classname) == 0
        && (*check).count == 1
        && (*check).health <= 100
    {
        return QTRUE;
    }

    QFALSE
}

/// `void ChangeWeapon( gentity_t *ent, int newWeapon )` (NPC_combat.c:570).
///
/// Switch NPC `ent` to `newWeapon`: set `ps.weapon` and mirror it into `pers.cmd.weapon`,
/// reset shot/burst state, refill
/// `currentAmmo` from the player ammo array, then configure the per-weapon burst pattern
/// (`NPCAI_BURST_WEAPON` flag + `burstMin/Mean/Max/Spacing`) keyed off weapon, `SCF_ALT_FIRE`,
/// `g_spskill` difficulty, and (for the emplaced gun) the owning chair's `wait`. No-op unless
/// `ent` is a client NPC. The commented-out blaster-pistol/bot-laser/saber/tricorder/ATST
/// cases and `rwwFIXMEFIXME` notes are dropped/kept faithfully as in the C. No oracle
/// (entity-state mutation); verified by review against the C.
///
/// # Safety
/// `ent`/`ent->client`/`ent->NPC` may be NULL (the early guard handles it); `ent->parent`
/// may be NULL. `g_spskill` must be registered; `newWeapon` indexes `weaponData`/`ps.ammo`.
pub unsafe fn ChangeWeapon(ent: *mut gentity_t, newWeapon: c_int) {
    if ent.is_null() || (*ent).client.is_null() || (*ent).NPC.is_null() {
        return;
    }

    (*(*ent).client).ps.weapon = newWeapon;
    (*(*ent).client).pers.cmd.weapon = newWeapon as u8;
    (*(*ent).NPC).shotTime = 0;
    (*(*ent).NPC).burstCount = 0;
    (*(*ent).NPC).attackHold = 0;
    (*(*ent).NPC).currentAmmo =
        (*(*ent).client).ps.ammo[weaponData[newWeapon as usize].ammoIndex as usize];

    let g_sp = (*addr_of!(g_spskill)).integer;
    let scf_alt = ((*(*ent).NPC).scriptFlags & SCF_ALT_FIRE) != 0;

    match newWeapon {
        // prifle
        WP_BRYAR_PISTOL => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
            (*(*ent).NPC).burstSpacing = 1000; // attackdebounce
        }
        // rwwFIXMEFIXME: support WP_BLASTER_PISTOL and WP_BOT_LASER
        WP_SABER => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
            (*(*ent).NPC).burstSpacing = 0; // attackdebounce
        }
        WP_DISRUPTOR => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
            if scf_alt {
                match g_sp {
                    0 => (*(*ent).NPC).burstSpacing = 2500, // attackdebounce
                    1 => (*(*ent).NPC).burstSpacing = 2000, // attackdebounce
                    2 => (*(*ent).NPC).burstSpacing = 1500, // attackdebounce
                    _ => {}
                }
            } else {
                (*(*ent).NPC).burstSpacing = 1000; // attackdebounce
            }
        }
        WP_BOWCASTER => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
            if g_sp == 0 {
                (*(*ent).NPC).burstSpacing = 1000; // attack debounce
            } else if g_sp == 1 {
                (*(*ent).NPC).burstSpacing = 750; // attack debounce
            } else {
                (*(*ent).NPC).burstSpacing = 500; // attack debounce
            }
        }
        WP_REPEATER => {
            if scf_alt {
                (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
                (*(*ent).NPC).burstSpacing = 2000; // attackdebounce
            } else {
                (*(*ent).NPC).aiFlags |= NPCAI_BURST_WEAPON;
                (*(*ent).NPC).burstMin = 3;
                (*(*ent).NPC).burstMean = 6;
                (*(*ent).NPC).burstMax = 10;
                if g_sp == 0 {
                    (*(*ent).NPC).burstSpacing = 1500; // attack debounce
                } else if g_sp == 1 {
                    (*(*ent).NPC).burstSpacing = 1000; // attack debounce
                } else {
                    (*(*ent).NPC).burstSpacing = 500; // attack debounce
                }
            }
        }
        WP_DEMP2 => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
            (*(*ent).NPC).burstSpacing = 1000; // attackdebounce
        }
        WP_FLECHETTE => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
            if scf_alt {
                (*(*ent).NPC).burstSpacing = 2000; // attackdebounce
            } else {
                (*(*ent).NPC).burstSpacing = 1000; // attackdebounce
            }
        }
        WP_ROCKET_LAUNCHER => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
            if g_sp == 0 {
                (*(*ent).NPC).burstSpacing = 2500; // attack debounce
            } else if g_sp == 1 {
                (*(*ent).NPC).burstSpacing = 2000; // attack debounce
            } else {
                (*(*ent).NPC).burstSpacing = 1500; // attack debounce
            }
        }
        WP_THERMAL => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
            if g_sp == 0 {
                (*(*ent).NPC).burstSpacing = 3000; // attack debounce
            } else if g_sp == 1 {
                (*(*ent).NPC).burstSpacing = 2500; // attack debounce
            } else {
                (*(*ent).NPC).burstSpacing = 2000; // attack debounce
            }
        }
        // (commented-out WP_SABER/WP_TRICORDER burst cases in the C are dropped)
        WP_BLASTER => {
            if scf_alt {
                (*(*ent).NPC).aiFlags |= NPCAI_BURST_WEAPON;
                (*(*ent).NPC).burstMin = 3;
                (*(*ent).NPC).burstMean = 3;
                (*(*ent).NPC).burstMax = 3;
                if g_sp == 0 {
                    (*(*ent).NPC).burstSpacing = 1500; // attack debounce
                } else if g_sp == 1 {
                    (*(*ent).NPC).burstSpacing = 1000; // attack debounce
                } else {
                    (*(*ent).NPC).burstSpacing = 500; // attack debounce
                }
            } else {
                (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
                if g_sp == 0 {
                    (*(*ent).NPC).burstSpacing = 1000; // attack debounce
                } else if g_sp == 1 {
                    (*(*ent).NPC).burstSpacing = 750; // attack debounce
                } else {
                    (*(*ent).NPC).burstSpacing = 500; // attack debounce
                }
            }
        }
        WP_STUN_BATON => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
            (*(*ent).NPC).burstSpacing = 1000; // attackdebounce
        }
        // rwwFIXMEFIXME: support for atst weaps (commented-out WP_ATST_* cases dropped)
        WP_EMPLACED_GUN => {
            // FIXME: give some designer-control over this?
            if !(*ent).client.is_null() && (*(*ent).client).NPC_class == CLASS_REELO {
                (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
                (*(*ent).NPC).burstSpacing = 1000; // attack debounce
            } else {
                (*(*ent).NPC).aiFlags |= NPCAI_BURST_WEAPON;
                (*(*ent).NPC).burstMin = 2; // 3 shots, really
                (*(*ent).NPC).burstMean = 2;
                (*(*ent).NPC).burstMax = 2;

                if !(*ent).parent.is_null() {
                    // if we have an owner, it should be the chair at this point...so query
                    // the chair for its shot debounce times, etc.
                    if g_sp == 0 {
                        (*(*ent).NPC).burstSpacing = ((*(*ent).parent).wait + 400.0) as c_int; // attack debounce
                        (*(*ent).NPC).burstMin = 1;
                        (*(*ent).NPC).burstMax = 1; // two shots
                    } else if g_sp == 1 {
                        (*(*ent).NPC).burstSpacing = ((*(*ent).parent).wait + 200.0) as c_int;
                    // attack debounce
                    } else {
                        (*(*ent).NPC).burstSpacing = (*(*ent).parent).wait as c_int;
                        // attack debounce
                    }
                } else if g_sp == 0 {
                    (*(*ent).NPC).burstSpacing = 1200; // attack debounce
                    (*(*ent).NPC).burstMin = 1;
                    (*(*ent).NPC).burstMax = 1; // two shots
                } else if g_sp == 1 {
                    (*(*ent).NPC).burstSpacing = 1000; // attack debounce
                } else {
                    (*(*ent).NPC).burstSpacing = 800; // attack debounce
                }
            }
        }
        _ => {
            (*(*ent).NPC).aiFlags &= !NPCAI_BURST_WEAPON;
        }
    }
}

/// `void NPC_ChangeWeapon( int newWeapon )` (NPC_combat.c:843).
///
/// Empty in the original — its entire body is commented out behind a `rwwFIXMEFIXME`
/// ("Change the same way as players, all this stuff is just crazy"). Ported as the
/// no-op it is; the dropped block referenced the file-static `NPC` global and Ghoul2
/// weapon-model traps that are not available here. No oracle (trivial no-op).
///
/// # Safety
/// Trivially safe; takes no pointers.
pub unsafe fn NPC_ChangeWeapon(_newWeapon: c_int) {
    // rwwFIXMEFIXME: Change the same way as players, all this stuff is just crazy.
}

/// `qboolean NPC_ReserveCombatPoint( int combatPointID )` (NPC_combat.c:2922).
///
/// Marks combat point `combatPointID` as occupied, failing (`qfalse`) if the id is
/// past `level.numCombatPoints` or the point is already taken. The bounds test keeps
/// the C's `>` (not `>=`) verbatim. No oracle (reads/writes the process-global
/// `level.combatPoints` array); verified by review against the C.
///
/// # Safety
/// `level` must be initialised.
pub unsafe fn NPC_ReserveCombatPoint(combatPointID: c_int) -> qboolean {
    //Make sure it's valid
    //if combatPointID > (*addr_of!(level)).numCombatPoints {
    if combatPointID < 0 || combatPointID > (*addr_of!(level)).numCombatPoints {
        //RUST-FIX: C lacks `< 0` guard, relies on benign OOB read of combatPoints[-1] for the -1 sentinel
        return QFALSE;
    }

    //Make sure it's not already occupied
    if (*addr_of!(level)).combatPoints[combatPointID as usize].occupied != QFALSE {
        return QFALSE;
    }

    //Reserve it
    (*addr_of_mut!(level)).combatPoints[combatPointID as usize].occupied = QTRUE;

    QTRUE
}

/// `qboolean NPC_FreeCombatPoint( int combatPointID, qboolean failed )` (NPC_combat.c:2944).
///
/// Releases a previously-reserved combat point. When `failed`, first records the id in
/// `NPCInfo->lastFailedCombatPoint` (so the NPC avoids re-picking it). Fails (`qfalse`)
/// if the id is past `level.numCombatPoints` or the point was not occupied. Keeps the
/// C's `>` bound verbatim. No oracle (process-global `level` + `NPCInfo`); verified by
/// review.
///
/// # Safety
/// `level` must be initialised; when `failed`, `NPCInfo` must be non-null (the NPC core
/// sets it for the current think).
pub unsafe fn NPC_FreeCombatPoint(combatPointID: c_int, failed: qboolean) -> qboolean {
    if failed != QFALSE {
        //remember that this one failed for us
        (*(*addr_of!(NPCInfo))).lastFailedCombatPoint = combatPointID;
    }
    //Make sure it's valid
    //if combatPointID > (*addr_of!(level)).numCombatPoints {
    if combatPointID < 0 || combatPointID > (*addr_of!(level)).numCombatPoints {
        //RUST-FIX: C lacks `< 0` guard, relies on benign OOB read of combatPoints[-1] for the -1 sentinel
        return QFALSE;
    }

    //Make sure it's currently occupied
    if (*addr_of!(level)).combatPoints[combatPointID as usize].occupied == QFALSE {
        return QFALSE;
    }

    //Free it
    (*addr_of_mut!(level)).combatPoints[combatPointID as usize].occupied = QFALSE;

    QTRUE
}

/// `qboolean NPC_SetCombatPoint( int combatPointID )` (NPC_combat.c:2970).
///
/// Switches the current NPC to combat point `combatPointID`: frees the one it already
/// holds (if any), reserves the new one, then records it in `NPCInfo->combatPoint`.
/// Returns `qfalse` (leaving `NPCInfo->combatPoint` unchanged) if the reservation
/// fails. No oracle (process-global `level` + `NPCInfo`); verified by review.
///
/// # Safety
/// `NPCInfo` must be non-null and `level` initialised.
pub unsafe fn NPC_SetCombatPoint(combatPointID: c_int) -> qboolean {
    //Free a combat point if we already have one
    if (*(*addr_of!(NPCInfo))).combatPoint != -1 {
        NPC_FreeCombatPoint((*(*addr_of!(NPCInfo))).combatPoint, QFALSE);
    }

    if NPC_ReserveCombatPoint(combatPointID) == QFALSE {
        return QFALSE;
    }

    (*(*addr_of!(NPCInfo))).combatPoint = combatPointID;

    QTRUE
}

/// `int NPC_FindSquadPoint( vec3_t position )` (NPC_combat.c:2881).
///
/// Returns the index of the nearest unoccupied *squad* combat point (flagged
/// `CPF_SQUAD`) to `position`, or `-1` if none. Distance is squared (`DistanceSquared`)
/// seeded with `(float)WORLD_SIZE*(float)WORLD_SIZE` as the "infinitely far" sentinel.
/// No oracle (reads process-global `level.combatPoints`); verified by review.
///
/// # Safety
/// `position` must be a valid `vec3_t`; `level` must be initialised.
pub unsafe fn NPC_FindSquadPoint(position: &vec3_t) -> c_int {
    let mut nearest_dist: f32 = WORLD_SIZE as f32 * WORLD_SIZE as f32;
    let mut nearest_point: c_int = -1;

    let num_combat_points = (*addr_of!(level)).numCombatPoints;
    for i in 0..num_combat_points {
        let cp = (*addr_of!(level)).combatPoints[i as usize];

        //Squad points are only valid if we're looking for them
        if (cp.flags & CPF_SQUAD) == QFALSE {
            continue;
        }

        //Must be vacant
        if cp.occupied == QTRUE {
            continue;
        }

        let dist = DistanceSquared(position, &cp.origin);

        //See if this is closer than the others
        if dist < nearest_dist {
            nearest_point = i;
            nearest_dist = dist;
        }
    }

    nearest_point
}

/// `combatPt_t` (NPC_combat.c, file-local typedef) — a scored candidate combat point:
/// squared distance from the query origin plus its index into `level.combatPoints`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct combatPt_t {
    dist: f32,
    index: c_int,
}

/// `static int NPC_CollectCombatPoints( const vec3_t origin, const float radius, combatPt_t *points, const int flags )` (NPC_combat.c:2574).
///
/// Fills `points` with every unoccupied combat point within `radius` of `origin` whose
/// stored `CPF_*` flags satisfy the request `flags` (`CP_DUCK`/`CP_FLEE`/`CP_INVESTIGATE`
/// /`CP_SQUAD`/`CP_NO_PVS`), measuring squared distance horizontally
/// (`CP_HORZ_DIST_COLL`) or in 3-D. Returns the count collected (capped at
/// `MAX_COMBAT_POINTS`). The `bestDistance`/`bestPoint` tracking is computed but the C
/// returns `numPoints`, not `bestPoint` (`return numPoints;//bestPoint;`), so the best
/// index is dead — kept under `_best_point` to mirror the original. No oracle
/// (process-global `level` + `trap_InPVS`); verified by review.
///
/// # Safety
/// `origin` valid; `points` must address a buffer of at least `MAX_COMBAT_POINTS`
/// `combatPt_t`; `level` must be initialised.
unsafe fn NPC_CollectCombatPoints(
    origin: &vec3_t,
    radius: f32,
    points: *mut combatPt_t,
    flags: c_int,
) -> c_int {
    let radius_sqr = radius * radius;
    let mut best_distance: f32 = Q3_INFINITE as f32;
    let mut _best_point: c_int = 0;
    let mut num_points: c_int = 0;

    //Collect all nearest
    let num_combat_points = (*addr_of!(level)).numCombatPoints;
    for i in 0..num_combat_points {
        if num_points >= MAX_COMBAT_POINTS as c_int {
            break;
        }

        let cp = (*addr_of!(level)).combatPoints[i as usize];

        //Must be vacant
        if cp.occupied == QTRUE {
            continue;
        }

        //If we want a duck space, make sure this is one
        if (flags & CP_DUCK) != 0 && (cp.flags & CPF_DUCK) != 0 {
            continue;
        }

        //If we want a duck space, make sure this is one
        if (flags & CP_FLEE) != 0 && (cp.flags & CPF_FLEE) != 0 {
            continue;
        }

        //Make sure this is an investigate combat point
        if (flags & CP_INVESTIGATE) != 0 && (cp.flags & CPF_INVESTIGATE) != 0 {
            continue;
        }

        //Squad points are only valid if we're looking for them
        if (cp.flags & CPF_SQUAD) != 0 && (flags & CP_SQUAD) == QFALSE {
            continue;
        }

        if (flags & CP_NO_PVS) != 0 {
            //must not be within PVS of mu current origin
            if trap::InPVS(origin, &cp.origin) != QFALSE {
                continue;
            }
        }

        let distance = if (flags & CP_HORZ_DIST_COLL) != 0 {
            DistanceHorizontalSquared(origin, &cp.origin)
        } else {
            DistanceSquared(origin, &cp.origin)
        };

        if distance < radius_sqr {
            if distance < best_distance {
                best_distance = distance;
                _best_point = num_points;
            }

            (*points.offset(num_points as isize)).dist = distance;
            (*points.offset(num_points as isize)).index = i;
            num_points += 1;
        }
    }

    num_points //bestPoint;
}

/// `void CP_FindCombatPointWaypoints( void )` (NPC_combat.c:2547).
///
/// For every registered combat point, resolves and caches the nearest navigation
/// waypoint (`NAV_FindClosestWaypointForPoint2`). Under `#ifndef FINAL_BUILD` an error is
/// printed (red `^1`) for any combat point that fails to find a waypoint
/// (`WAYPOINT_NONE`). No oracle (mutates process-global `level.combatPoints`, calls a NAV
/// trap + `Com_Printf`); verified by review against the C.
///
/// # Safety
/// `level` must be initialised.
pub unsafe fn CP_FindCombatPointWaypoints() {
    let num_combat_points = (*addr_of!(level)).numCombatPoints;
    for i in 0..num_combat_points {
        let origin = (*addr_of!(level)).combatPoints[i as usize].origin;
        let wp = NAV_FindClosestWaypointForPoint2(&origin);
        (*addr_of_mut!(level)).combatPoints[i as usize].waypoint = wp;
        // #ifndef FINAL_BUILD
        if wp == WAYPOINT_NONE {
            Com_Printf(&format!(
                "^1ERROR: Combat Point at {} has no waypoint!\n",
                core::ffi::CStr::from_ptr(vtos(&origin)).to_string_lossy()
            ));
        }
        // #endif
    }
}

/// `int NPC_FindCombatPoint( const vec3_t position, const vec3_t avoidPosition, vec3_t
/// enemyPosition, const int flags, const float avoidDist, const int ignorePoint )`
/// (NPC_combat.c:2656).
///
/// Searches the registered combat points for the best one matching a bitmask of
/// `CP_*` requirements (cover / clear LOS / approach / retreat / flank / avoid / route
/// / nearest) relative to the NPC's `position`, an `avoidPosition`, and the
/// `enemyPosition`. Candidates are first gathered by `NPC_CollectCombatPoints` within a
/// collection radius (quadrupled for `CP_NO_PVS`), then each is filtered through the
/// requested predicates; with `CP_NEAREST` the lowest path-cost point wins, otherwise
/// the first surviving (closest-to-enemy) candidate is returned immediately. Returns the
/// combat-point index, or `-1` (`best`) if none qualify. The commented-out
/// `NAV_FindClosestWaypointForEnt`/`trap_Nav_GetNodePosition` alternates are kept
/// commented. No oracle (reads process-global `NPC`/`NPCInfo`/`level`, NAV traps,
/// `trap_Trace`); verified by review against the C.
///
/// # Safety
/// `position`/`avoidPosition`/`enemyPosition` must be valid `vec3_t`; `NPC`/`NPCInfo`
/// must be set and `level` initialised.
pub unsafe fn NPC_FindCombatPoint(
    position: &vec3_t,
    _avoid_position: &vec3_t,
    enemy_position: &vec3_t,
    flags: c_int,
    avoid_dist: f32,
    ignore_point: c_int,
) -> c_int {
    const MIN_AVOID_DOT: f32 = 0.75;
    const MIN_AVOID_DISTANCE: f32 = 128.0;
    const MIN_AVOID_DISTANCE_SQUARED: f32 = MIN_AVOID_DISTANCE * MIN_AVOID_DISTANCE;
    const CP_COLLECT_RADIUS: f32 = 512.0;

    let mut points: [combatPt_t; MAX_COMBAT_POINTS] = [combatPt_t::default(); MAX_COMBAT_POINTS];
    let mut best: c_int = -1;
    let mut best_cost: c_int = Q3_INFINITE as c_int;
    let mut waypoint: c_int = WAYPOINT_NONE;
    let mut coll_rad: f32 = CP_COLLECT_RADIUS;
    let mut modified_avoid_dist: f32 = avoid_dist;

    if modified_avoid_dist <= 0.0 {
        modified_avoid_dist = MIN_AVOID_DISTANCE_SQUARED;
    } else {
        modified_avoid_dist *= modified_avoid_dist;
    }

    if (flags & CP_HAS_ROUTE) != 0 || (flags & CP_NEAREST) != 0 {
        //going to be doing macro nav tests
        if (*(*addr_of!(NPC))).waypoint == WAYPOINT_NONE {
            waypoint = NAV_GetNearestNode(*addr_of!(NPC), (*(*addr_of!(NPC))).lastWaypoint);
        } else {
            waypoint = (*(*addr_of!(NPC))).waypoint;
        }
    }

    //Collect our nearest points
    if (flags & CP_NO_PVS) != 0 {
        //much larger radius since most will be dropped?
        coll_rad = CP_COLLECT_RADIUS * 4.0;
    }
    let num_points = NPC_CollectCombatPoints(enemy_position, coll_rad, points.as_mut_ptr(), flags); //position

    for j in 0..num_points {
        //const int i = (*cpi).second;
        let i = points[j as usize].index;
        let pdist = points[j as usize].dist;

        //Must not be one we want to ignore
        if i == ignore_point {
            continue;
        }

        //FIXME: able to mark certain ones as too dangerous to go to for now?  Like a tripmine/thermal/detpack is near or something?
        //If we need a cover point, check this point
        if (flags & CP_COVER) != 0
            && NPC_ClearLOS(
                &(*addr_of!(level)).combatPoints[i as usize].origin,
                enemy_position,
            ) == QTRUE
        {
            //Used to use NPC->enemy
            continue;
        }

        //Need a clear LOS to our target... and be within shot range to enemy position (FIXME: make this a separate CS_ flag? and pass in a range?)
        if (flags & CP_CLEAR) != 0 {
            if NPC_ClearLOS3(
                &(*addr_of!(level)).combatPoints[i as usize].origin,
                (*(*addr_of!(NPC))).enemy,
            ) == QFALSE
            {
                continue;
            }
            let dist = if (*(*addr_of!(NPC))).s.weapon == WP_THERMAL {
                //horizontal
                DistanceHorizontalSquared(
                    &(*addr_of!(level)).combatPoints[i as usize].origin,
                    &(*(*(*addr_of!(NPC))).enemy).r.currentOrigin,
                )
            } else {
                //actual
                DistanceSquared(
                    &(*addr_of!(level)).combatPoints[i as usize].origin,
                    &(*(*(*addr_of!(NPC))).enemy).r.currentOrigin,
                )
            };
            if dist
                > ((*(*addr_of!(NPCInfo))).stats.visrange * (*(*addr_of!(NPCInfo))).stats.visrange)
            {
                continue;
            }
        }

        //Avoid this position?
        if (flags & CP_AVOID) != 0
            && DistanceSquared(
                &(*addr_of!(level)).combatPoints[i as usize].origin,
                position,
            ) < modified_avoid_dist
        {
            //was using MIN_AVOID_DISTANCE_SQUARED, not passed in modifiedAvoidDist
            continue;
        }

        //Try to find a point closer to the enemy than where we are
        if (flags & CP_APPROACH_ENEMY) != 0 {
            if (flags & CP_HORZ_DIST_COLL) != 0 {
                if pdist > DistanceHorizontalSquared(position, enemy_position) {
                    continue;
                }
            } else if pdist > DistanceSquared(position, enemy_position) {
                continue;
            }
        }
        //Try to find a point farther from the enemy than where we are
        if (flags & CP_RETREAT) != 0 {
            if (flags & CP_HORZ_DIST_COLL) != 0 {
                if pdist < DistanceHorizontalSquared(position, enemy_position) {
                    //it's closer, don't use it
                    continue;
                }
            } else if pdist < DistanceSquared(position, enemy_position) {
                //it's closer, don't use it
                continue;
            }
        }

        //We want a point on other side of the enemy from current pos
        if (flags & CP_FLANK) != 0 {
            let mut e_dir2_me: vec3_t = [0.0; 3];
            let mut e_dir2_cp: vec3_t = [0.0; 3];

            VectorSubtract(position, enemy_position, &mut e_dir2_me);
            VectorNormalize(&mut e_dir2_me);

            VectorSubtract(
                &(*addr_of!(level)).combatPoints[i as usize].origin,
                enemy_position,
                &mut e_dir2_cp,
            );
            VectorNormalize(&mut e_dir2_cp);

            let dot = DotProduct(&e_dir2_me, &e_dir2_cp);

            //Not far enough behind enemy from current pos
            if dot >= 0.4 {
                continue;
            }
        }

        //See if we're trying to avoid our enemy
        //FIXME: this needs to check for the waypoint you'll be taking to get to that combat point
        if (flags & CP_AVOID_ENEMY) != 0 {
            let mut e_dir: vec3_t = [0.0; 3];
            let mut g_dir: vec3_t = [0.0; 3];
            let mut wp_org: vec3_t = [0.0; 3];

            VectorSubtract(position, enemy_position, &mut e_dir);
            VectorNormalize(&mut e_dir);

            /*
            NAV_FindClosestWaypointForEnt( NPC, level.combatPoints[i].waypoint );
            if ( NPC->waypoint != WAYPOINT_NONE && NPC->waypoint != level.combatPoints[i].waypoint )
            {
                trap_Nav_GetNodePosition( NPC->waypoint, wpOrg );
            }
            else
            */
            {
                VectorCopy(
                    &(*addr_of!(level)).combatPoints[i as usize].origin,
                    &mut wp_org,
                );
            }
            VectorSubtract(position, &wp_org, &mut g_dir);
            VectorNormalize(&mut g_dir);

            let dot = DotProduct(&g_dir, &e_dir);

            //Don't want to run at enemy
            if dot >= MIN_AVOID_DOT {
                continue;
            }

            //Can't be too close to the enemy
            if DistanceSquared(&wp_org, enemy_position) < modified_avoid_dist {
                continue;
            }
        }

        //Okay, now make sure it's not blocked
        let tr = trap::Trace(
            &(*addr_of!(level)).combatPoints[i as usize].origin,
            &(*(*addr_of!(NPC))).r.mins,
            &(*(*addr_of!(NPC))).r.maxs,
            &(*addr_of!(level)).combatPoints[i as usize].origin,
            (*(*addr_of!(NPC))).s.number,
            (*(*addr_of!(NPC))).clipmask,
        );
        if tr.allsolid != 0 || tr.startsolid != 0 {
            continue;
        }

        //we must have a route to the combat point
        if (flags & CP_HAS_ROUTE) != 0 {
            /*
            if ( level.combatPoints[i].waypoint == WAYPOINT_NONE )
            {
                level.combatPoints[i].waypoint = NAV_FindClosestWaypointForPoint( level.combatPoints[i].origin );
            }
            */

            if waypoint == WAYPOINT_NONE
                || (*addr_of!(level)).combatPoints[i as usize].waypoint == WAYPOINT_NONE
                || trap::Nav_GetBestNodeAltRoute2(
                    waypoint,
                    (*addr_of!(level)).combatPoints[i as usize].waypoint,
                    NODE_NONE,
                ) == WAYPOINT_NONE
            {
                //can't possibly have a route to any OR can't possibly have a route to this one OR don't have a route to this one
                if NAV_ClearPathToPoint(
                    *addr_of!(NPC),
                    &(*(*addr_of!(NPC))).r.mins,
                    &(*(*addr_of!(NPC))).r.maxs,
                    &(*addr_of!(level)).combatPoints[i as usize].origin,
                    (*(*addr_of!(NPC))).clipmask,
                    ENTITYNUM_NONE,
                ) == 0
                {
                    //don't even have a clear straight path to this one
                    continue;
                }
            }
        }

        //We want the one with the shortest path from current pos
        if (flags & CP_NEAREST) != 0
            && waypoint != WAYPOINT_NONE
            && (*addr_of!(level)).combatPoints[i as usize].waypoint != WAYPOINT_NONE
        {
            let cost = trap::Nav_GetPathCost(
                waypoint,
                (*addr_of!(level)).combatPoints[i as usize].waypoint,
            );
            if cost < best_cost {
                best_cost = cost;
                best = i;
            }
            continue;
        }

        //we want the combat point closest to the enemy
        //if ( flags & CP_CLOSEST )
        //they are sorted by this distance, so the first one to get this far is the closest
        return i;
    }

    best
}

/// `void NPC_SetPickUpGoal( gentity_t *foundWeap )` (NPC_combat.c:3046).
///
/// Sets up the NPC to navigate to a dropped weapon: copies the weapon's origin
/// (raised to ground level via its mins), routes a move goal through
/// `NPC_SetMoveGoal`, copies the weapon's waypoint onto the temp goal, and flips the
/// NPC into the default behavior with `SQUAD_TRANSITION` squad state. The
/// commented-out `NPCInfo->goalEntity = foundWeap;` is kept commented. No oracle
/// (writes process-global `NPCInfo` opaque state via `NPC_SetMoveGoal`); verified by
/// review against the C.
///
/// # Safety
/// `found_weap` must point to a valid `gentity_t`; `NPC`/`NPCInfo` must be set.
pub unsafe fn NPC_SetPickUpGoal(found_weap: *mut gentity_t) {
    let mut org: vec3_t = [0.0; 3];

    //NPCInfo->goalEntity = foundWeap;
    VectorCopy(&(*found_weap).r.currentOrigin, &mut org);
    org[2] += 24.0 - ((*found_weap).r.mins[2] * -1.0); //adjust the origin so that I am on the ground
    NPC_SetMoveGoal(
        *addr_of!(NPC),
        &org,
        ((*found_weap).r.maxs[0] * 0.75) as i32,
        QFALSE,
        -1,
        found_weap,
    );
    (*(*(*addr_of!(NPCInfo))).tempGoal).waypoint = (*found_weap).waypoint;
    (*(*addr_of!(NPCInfo))).tempBehavior = BS_DEFAULT;
    (*(*addr_of!(NPCInfo))).squadState = SQUAD_TRANSITION;
}

/// `float IdealDistance( gentity_t *self )` (NPC_combat.c:2474).
///
/// Determines the NPC's ideal distance from its enemy: a base of
/// `225 - 20*aggression`, biased per current weapon (rocket launcher and thermal
/// detonator want more standoff). The `self` parameter is unused in the C body — the
/// computation reads the file-static `NPC`/`NPCInfo` globals — so it is dropped here.
/// The commented-out `WP_TRICORDER` case is kept commented. No oracle (reads
/// process-global `NPC`/`NPCInfo`); verified by review against the C.
///
/// # Safety
/// `NPC`/`NPCInfo` must be set (the NPC core sets them for the current think).
pub unsafe fn IdealDistance() -> f32 {
    let mut ideal: f32 = 225.0 - 20.0 * (*(*addr_of!(NPCInfo))).stats.aggression as f32;
    match (*(*addr_of!(NPC))).s.weapon {
        WP_ROCKET_LAUNCHER => ideal += 200.0,

        WP_THERMAL => ideal += 50.0,

        /*	case WP_TRICORDER:
            ideal = 0;
            break;
        */
        WP_SABER | WP_BRYAR_PISTOL | WP_BLASTER => {}
        //	case WP_BLASTER_PISTOL:
        _ => {}
    }

    ideal
}

/// `qboolean HaveWeapon( int weapon )` (NPC_combat.c:1108).
///
/// True if the current NPC's `STAT_WEAPONS` bitfield has `weapon`'s bit set. The C reads
/// the file-static `client` global, which is always `NPC->client` (set together in
/// `SetNPCGlobals`), so it is accessed via `NPC->client` here. No oracle (reads the
/// process-global `NPC`); verified by review against the C.
///
/// # Safety
/// `NPC`/`NPC->client` must be set (the NPC core sets them for the current think).
pub unsafe fn HaveWeapon(weapon: c_int) -> qboolean {
    if ((*(*(*addr_of!(NPC))).client).ps.stats[STAT_WEAPONS as usize] & (1 << weapon)) != 0 {
        QTRUE
    } else {
        QFALSE
    }
}

/// `int NPC_AttackDebounceForWeapon( void )` (NPC_combat.c:1287).
///
/// How long, after firing, the NPC keeps its weapon up (does NOT control fire rate).
/// Saber returns 0; everything else returns `NPCInfo->burstSpacing`. The C reads the
/// file-static `NPC` global; the commented-out per-weapon cases (`WP_BLASTER`/
/// `WP_BRYAR_PISTOL`/`WP_BOT_LASER`/`WP_TRICORDER`) are kept commented. No oracle
/// (reads process-global `NPC`/`NPCInfo`); verified by review against the C.
///
/// # Safety
/// `NPC`/`NPC->client`/`NPCInfo` must be set for the current think.
pub unsafe fn NPC_AttackDebounceForWeapon() -> c_int {
    match (*(*(*addr_of!(NPC))).client).ps.weapon {
        /*
            case WP_BLASTER://scav rifle
                return 1000;
                break;

            case WP_BRYAR_PISTOL://prifle
                return 3000;
                break;

            case WP_SABER:
                return 100;
                break;


            case WP_TRICORDER:
                return 0;//tricorder
                break;
        */
        WP_SABER => 0,

        /*
            case WP_BOT_LASER:

            if ( g_spskill.integer == 0 )
                return 2000;

            if ( g_spskill.integer == 1 )
                return 1500;

            return 1000;
            break;
        */
        //rwwFIXMEFIXME: support
        _ => (*(*addr_of!(NPCInfo))).burstSpacing, //was 100 by default
    }
}

/// `void NPC_ApplyWeaponFireDelay( void )` (NPC_combat.c:877).
///
/// How long, if at all, the actual fire delays from the time the attack started. If we
/// just fired (`attackDebounceTime > level.time`) this is a burst-fire follow-up, so no
/// delay is added. Otherwise sets `weaponTime` per weapon. The C reads the file-static
/// `NPC`/`client` globals (`client == NPC->client`). The commented-out `WP_BOT_LASER`
/// case is kept commented; the `WP_STUN_BATON` `PM_DroidMelee` check stays the C's
/// `if (1)` rwwFIXME. No oracle (process-global `NPC`/`level` mutation); verified by
/// review against the C.
///
/// # Safety
/// `NPC`/`NPC->client` must be set; `level` must be initialised.
pub unsafe fn NPC_ApplyWeaponFireDelay() {
    if (*(*addr_of!(NPC))).attackDebounceTime > (*addr_of!(level)).time {
        //Just fired, if attacking again, must be a burst fire, so don't add delay
        //NOTE: Borg AI uses attackDebounceTime "incorrectly", so this will always return for them!
        return;
    }

    match (*(*(*addr_of!(NPC))).client).ps.weapon {
        /*
        case WP_BOT_LASER:
            NPCInfo->burstCount = 0;
            client->ps.weaponTime = 500;
            break;
        */ //rwwFIXMEFIXME: support for this
        WP_THERMAL => {
            if (*(*(*addr_of!(NPC))).client).ps.clientNum != 0 {
                //NPCs delay...
                //FIXME: player should, too, but would feel weird in 1st person, even though it
                //			would look right in 3rd person.  Really should have a wind-up anim
                //			for player as he holds down the fire button to throw, then play
                //			the actual throw when he lets go...
                (*(*(*addr_of!(NPC))).client).ps.weaponTime = 700;
            }
        }

        WP_STUN_BATON => {
            //if ( !PM_DroidMelee( client->NPC_class ) )
            if true {
                //rwwFIXMEFIXME: ...
                //FIXME: should be unique per melee anim
                (*(*(*addr_of!(NPC))).client).ps.weaponTime = 300;
            }
        }

        _ => {
            (*(*(*addr_of!(NPC))).client).ps.weaponTime = 0;
        }
    }
}

//FIXME: need a mindist for explosive weapons
/// `float NPC_MaxDistSquaredForWeapon( void )` (NPC_combat.c:1333).
///
/// The squared max engagement range for the current NPC's weapon. A nonzero
/// `NPCInfo->stats.shootDistance` overrides the per-weapon default; otherwise it is keyed
/// off `NPC->s.weapon` (disruptor doubles for alt-fire sniping; saber uses its blade
/// `lengthMax` plus a margin off `NPC->r.maxs[0]`). The commented-out
/// `WP_BLASTER_PISTOL`/`WP_SABER`/`WP_TRICORDER` cases are kept commented. No oracle
/// (reads process-global `NPC`/`NPCInfo`); verified by review against the C.
///
/// # Safety
/// `NPC`/`NPCInfo` must be set; `NPC->client` may be NULL (the saber case guards it).
pub unsafe fn NPC_MaxDistSquaredForWeapon() -> f32 {
    if (*(*addr_of!(NPCInfo))).stats.shootDistance > 0.0 {
        //overrides default weapon dist
        return (*(*addr_of!(NPCInfo))).stats.shootDistance
            * (*(*addr_of!(NPCInfo))).stats.shootDistance;
    }

    match (*(*addr_of!(NPC))).s.weapon {
        WP_BLASTER => 1024.0 * 1024.0,      //scav rifle //should be shorter?
        WP_BRYAR_PISTOL => 1024.0 * 1024.0, //prifle

        /*
            case WP_BLASTER_PISTOL://prifle
                return 1024 * 1024;
                break;
        */
        WP_DISRUPTOR => {
            //disruptor
            if ((*(*addr_of!(NPCInfo))).scriptFlags & SCF_ALT_FIRE) != 0 {
                4096.0 * 4096.0
            } else {
                1024.0 * 1024.0
            }
        }
        /*
            case WP_SABER:
                return 1024 * 1024;
                break;


            case WP_TRICORDER:
                return 0;//tricorder
                break;
        */
        WP_SABER => {
            if !(*(*addr_of!(NPC))).client.is_null()
                && (*(*(*addr_of!(NPC))).client).saber[0].blade[0].lengthMax != 0.0
            {
                //FIXME: account for whether enemy and I are heading towards each other!
                let len = (*(*(*addr_of!(NPC))).client).saber[0].blade[0].lengthMax
                    + (*(*addr_of!(NPC))).r.maxs[0] * 1.5;
                len * len
            } else {
                48.0 * 48.0
            }
        }

        _ => 1024.0 * 1024.0, //was 0
    }
}

/*
-------------------------
ValidEnemy
-------------------------
*/
/// `qboolean ValidEnemy( gentity_t *ent )` (NPC_combat.c:1399).
///
/// True if `ent` is a legitimate enemy target for the current NPC: non-null, not the NPC
/// itself, not notargeted, alive, and (for clients) not a spectator and on a team that is
/// hostile to — and not the same as — the NPC's. Clientless living ents are always valid.
/// The team mapping (`TEAM_BLUE`→player, `TEAM_RED`→enemy, else neutral) and the
/// commented-out `enemyTeam` early-out are kept faithfully. No oracle (reads process-global
/// `NPC` + opaque `gentity_t`); verified by review against the C.
///
/// # Safety
/// `ent` may be NULL (handled); `NPC`/`NPC->client` must be set for the current think.
pub unsafe fn ValidEnemy(ent: *mut gentity_t) -> qboolean {
    if ent.is_null() {
        return QFALSE;
    }

    if ent == *addr_of!(NPC) {
        return QFALSE;
    }

    //if team_free, maybe everyone is an enemy?
    //if ( !NPC->client->enemyTeam )
    //	return qfalse;

    if ((*ent).flags & FL_NOTARGET) == 0 {
        if (*ent).health > 0 {
            if (*ent).client.is_null() {
                return QTRUE;
            } else if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
                //don't go after spectators
                return QFALSE;
            } else {
                let mut ent_team = TEAM_FREE;
                if !(*ent).NPC.is_null() && !(*ent).client.is_null() {
                    ent_team = (*(*ent).client).playerTeam;
                } else if !(*ent).client.is_null() {
                    if (*(*ent).client).sess.sessionTeam == TEAM_BLUE {
                        ent_team = NPCTEAM_PLAYER;
                    } else if (*(*ent).client).sess.sessionTeam == TEAM_RED {
                        ent_team = NPCTEAM_ENEMY;
                    } else {
                        ent_team = NPCTEAM_NEUTRAL;
                    }
                }
                if ent_team == NPCTEAM_FREE
                    || (*(*(*addr_of!(NPC))).client).enemyTeam == NPCTEAM_FREE
                    || ent_team == (*(*(*addr_of!(NPC))).client).enemyTeam
                {
                    if ent_team != (*(*(*addr_of!(NPC))).client).playerTeam {
                        return QTRUE;
                    }
                }
            }
        }
    }

    QFALSE
}

/// `qboolean NPC_EnemyTooFar( gentity_t *enemy, float dist, qboolean toShoot )` (NPC_combat.c:1461).
///
/// True if `enemy` is beyond the current NPC's weapon range. When not actually trying to
/// fire (`!toShoot`) a saber NPC just has to close in, so it's never "too far". A zero
/// `dist` is computed as the squared distance between current origins; the result is then
/// compared to `NPC_MaxDistSquaredForWeapon()`. No oracle (reads process-global `NPC`);
/// verified by review against the C.
///
/// # Safety
/// `enemy` must point to a valid `gentity_t`; `NPC`/`NPC->client` must be set.
pub unsafe fn NPC_EnemyTooFar(enemy: *mut gentity_t, mut dist: f32, toShoot: qboolean) -> qboolean {
    let mut vec: vec3_t = [0.0; 3];

    if toShoot == QFALSE {
        //Not trying to actually press fire button with this check
        if (*(*(*addr_of!(NPC))).client).ps.weapon == WP_SABER {
            //Just have to get to him
            return QFALSE;
        }
    }

    if dist == 0.0 {
        VectorSubtract(
            &(*(*addr_of!(NPC))).r.currentOrigin,
            &(*enemy).r.currentOrigin,
            &mut vec,
        );
        dist = VectorLengthSquared(&vec);
    }

    if dist > NPC_MaxDistSquaredForWeapon() {
        return QTRUE;
    }

    QFALSE
}

/// `void NPC_AimAdjust( int change )` (NPC_combat.c:3097).
///
/// Drift the current NPC's `currentAim` toward `change` once the `"aimDebounce"` timer
/// elapses (re-arming it each time, scaled by `g_spskill`), clamped to
/// `[-30, stats.aim]`. If the timer doesn't yet exist it is created and the function
/// returns without adjusting. The C reads the file-static `NPC`/`NPCInfo` globals; the
/// commented-out alternate debounce timings and the `Com_Printf` debug line are kept
/// commented. No oracle (process-global `NPC`/`NPCInfo` + timers); verified by review
/// against the C.
///
/// # Safety
/// `NPC`/`NPCInfo` must be set; `g_spskill` must be registered.
pub unsafe fn NPC_AimAdjust(change: c_int) {
    if TIMER_Exists(*addr_of!(NPC), c"aimDebounce".as_ptr()) == QFALSE {
        let debounce = 500 + (3 - (*addr_of!(g_spskill)).integer) * 100;
        TIMER_Set(
            *addr_of!(NPC),
            c"aimDebounce".as_ptr(),
            Q_irand(debounce, debounce + 1000),
        );
        //int debounce = 1000+(3-g_spskill.integer)*500;
        //TIMER_Set( NPC, "aimDebounce", Q_irand( debounce, debounce+2000 ) );
        return;
    }
    if TIMER_Done(*addr_of!(NPC), c"aimDebounce".as_ptr()) != QFALSE {
        (*(*addr_of!(NPCInfo))).currentAim += change;
        if (*(*addr_of!(NPCInfo))).currentAim > (*(*addr_of!(NPCInfo))).stats.aim {
            //can never be better than max aim
            (*(*addr_of!(NPCInfo))).currentAim = (*(*addr_of!(NPCInfo))).stats.aim;
        } else if (*(*addr_of!(NPCInfo))).currentAim < -30 {
            //can never be worse than this
            (*(*addr_of!(NPCInfo))).currentAim = -30;
        }

        //Com_Printf( "%s new aim = %d\n", NPC->NPC_type, NPCInfo->currentAim );

        let debounce = 500 + (3 - (*addr_of!(g_spskill)).integer) * 100;
        TIMER_Set(
            *addr_of!(NPC),
            c"aimDebounce".as_ptr(),
            Q_irand(debounce, debounce + 1000),
        );
        //int debounce = 1000+(3-g_spskill.integer)*500;
        //TIMER_Set( NPC, "aimDebounce", Q_irand( debounce, debounce+2000 ) );
    }
}

/*QUAKED point_combat (0.7 0 0.7) (-16 -16 -24) (16 16 32) DUCK FLEE INVESTIGATE SQUAD LEAN SNIPE
NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point

DUCK - NPC will duck and fire from this point, NOT IMPLEMENTED?
FLEE - Will choose this point when running
INVESTIGATE - Will look here if a sound is heard near it
SQUAD - NOT IMPLEMENTED
LEAN - Lean-type cover, NOT IMPLEMENTED
SNIPE - Snipers look for these first, NOT IMPLEMENTED
*/
/// `void SP_point_combat( gentity_t *self )` (NPC_combat.c:2515).
///
/// Spawn function for a `point_combat` entity: registers it into the process-global
/// `level.combatPoints` table (copying its origin and `spawnflags`, marking it vacant),
/// then frees the spawn entity. Bails (freeing the entity) if the table is already full.
/// Nudges the origin up 0.125 and links so `G_CheckInSolid` can warn if it sits in solid.
/// The two `#ifndef FINAL_BUILD` diagnostics are kept active (this is a non-final build).
/// No oracle (process-global `level` mutation + spawn/link traps); verified by review
/// against the C.
///
/// # Safety
/// `self` must point to a valid spawning `gentity_t`; `level` must be initialised.
pub unsafe fn SP_point_combat(self_: *mut gentity_t) {
    if (*addr_of!(level)).numCombatPoints >= MAX_COMBAT_POINTS as c_int {
        // #ifndef FINAL_BUILD
        Com_Printf(&format!(
            "^1ERROR:  Too many combat points, limit is {}\n",
            MAX_COMBAT_POINTS
        ));
        G_FreeEntity(self_);
        return;
    }

    (*self_).s.origin[2] += 0.125;
    G_SetOrigin(self_, &(*self_).s.origin);
    trap::LinkEntity(self_);

    if G_CheckInSolid(self_, QTRUE) != QFALSE {
        // #ifndef FINAL_BUILD
        Com_Printf(&format!(
            "^1ERROR: combat point at {} in solid!\n",
            core::ffi::CStr::from_ptr(vtos(&(*self_).r.currentOrigin)).to_string_lossy()
        ));
    }

    let n = (*addr_of!(level)).numCombatPoints as usize;
    (*addr_of_mut!(level)).combatPoints[n].origin = (*self_).r.currentOrigin;

    (*addr_of_mut!(level)).combatPoints[n].flags = (*self_).spawnflags;
    (*addr_of_mut!(level)).combatPoints[n].occupied = QFALSE;

    (*addr_of_mut!(level)).numCombatPoints += 1;

    G_FreeEntity(self_);
}

/// `qboolean ShotThroughGlass( trace_t *tr, gentity_t *target, vec3_t spot, int mask )` (NPC_combat.c:1125).
///
/// If `tr` stopped on breakable glass (per [`EntIsGlass`]) that isn't the intended
/// `target`, re-trace from the glass impact point on toward `spot` (skipping the glass),
/// overwriting `*tr` with the new result and returning `qtrue` — letting NPCs see/shoot
/// through breakable panes. Otherwise returns `qfalse` and leaves `*tr` untouched. The C
/// passes `NULL` for the trace mins/maxs; the Rust `trap::Trace` wrapper takes them by
/// reference, so `&vec3_origin` (the zero vector) stands in for the point trace. No oracle
/// (`trap_Trace` + process-global `g_entities`); verified by review against the C.
///
/// # Safety
/// `tr` must point to a valid `trace_t`; `target` valid; `spot` a valid `vec3_t`;
/// `g_entities` must be initialised.
pub unsafe fn ShotThroughGlass(
    tr: *mut trace_t,
    target: *mut gentity_t,
    spot: &vec3_t,
    mask: c_int,
) -> qboolean {
    let hit: *mut gentity_t =
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset((*tr).entityNum as isize);
    if hit != target && EntIsGlass(hit) != QFALSE {
        //ok to shoot through breakable glass
        let skip = (*hit).s.number;
        let mut muzzle: vec3_t = [0.0; 3];

        VectorCopy(&(*tr).endpos, &mut muzzle);
        *tr = trap::Trace(&muzzle, &vec3_origin, &vec3_origin, spot, skip, mask);
        return QTRUE;
    }

    QFALSE
}

/*
CanShoot
determine if NPC can directly target enemy

this function does not check teams, invulnerability, notarget, etc....

Added: If can't shoot center, try head, if not, see if it's close enough to try anyway.
*/
/// `qboolean CanShoot( gentity_t *ent, gentity_t *shooter )` (NPC_combat.c:1149).
///
/// Determines whether `shooter` can directly line up a shot on `ent` (ignoring teams,
/// notarget, invulnerability, etc.). Traces from the shooter's muzzle to the target
/// origin; permits the shot through breakable glass ([`ShotThroughGlass`]); if center is
/// blocked, retries the head; failing that, succeeds anyway if the impact is within a
/// small random spread of the target. A point-blank `startsolid` against the NPC's
/// `touchedByPlayer` counts as that toucher. Blocked-by-non-client fails; a dead or
/// opposite-team blocker passes. No oracle (`trap_Trace` + process-global `g_entities`
/// + `random`); verified by review against the C.
///
/// # Safety
/// `ent`/`shooter` must point to valid `gentity_t`s; `g_entities` must be initialised.
pub unsafe fn CanShoot(ent: *mut gentity_t, shooter: *mut gentity_t) -> qboolean {
    let mut muzzle: vec3_t = [0.0; 3];
    let mut spot: vec3_t = [0.0; 3];
    let mut diff: vec3_t = [0.0; 3];

    CalcEntitySpot(shooter, SPOT_WEAPON, &mut muzzle);
    CalcEntitySpot(ent, SPOT_ORIGIN, &mut spot); //FIXME preferred target locations for some weapons (feet for R/L)

    let mut tr = trap::Trace(
        &muzzle,
        &vec3_origin,
        &vec3_origin,
        &spot,
        (*shooter).s.number,
        MASK_SHOT,
    );
    let mut trace_ent: *mut gentity_t =
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(tr.entityNum as isize);

    // point blank, baby!
    if tr.startsolid != 0
        && !(*shooter).NPC.is_null()
        && !(*(*shooter).NPC).touchedByPlayer.is_null()
    {
        trace_ent = (*(*shooter).NPC).touchedByPlayer;
    }

    if ShotThroughGlass(&mut tr, ent, &spot, MASK_SHOT) != QFALSE {
        trace_ent =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(tr.entityNum as isize);
    }

    // shot is dead on
    if trace_ent == ent {
        return QTRUE;
    }
    //MCG - Begin
    else {
        //ok, can't hit them in center, try their head
        CalcEntitySpot(ent, SPOT_HEAD, &mut spot);
        tr = trap::Trace(
            &muzzle,
            &vec3_origin,
            &vec3_origin,
            &spot,
            (*shooter).s.number,
            MASK_SHOT,
        );
        trace_ent =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(tr.entityNum as isize);
        if trace_ent == ent {
            return QTRUE;
        }
    }

    //Actually, we should just check to fire in dir we're facing and if it's close enough,
    //and we didn't hit someone on our own team, shoot
    VectorSubtract(&spot, &tr.endpos, &mut diff);
    if VectorLength(&diff) < random() * 32.0 {
        return QTRUE;
    }
    //MCG - End
    // shot would hit a non-client
    if (*trace_ent).client.is_null() {
        return QFALSE;
    }

    // shot is blocked by another player

    // he's already dead, so go ahead
    if (*trace_ent).health <= 0 {
        return QTRUE;
    }

    // don't deliberately shoot a teammate
    if !(*trace_ent).client.is_null()
        && (*(*trace_ent).client).playerTeam == (*(*shooter).client).playerTeam
    {
        return QFALSE;
    }

    // he's just in the wrong place, go ahead
    QTRUE
}

/*
-------------------------
NPC_ClearShot
-------------------------
*/
/// `qboolean NPC_ClearShot( gentity_t *ent )` (NPC_combat.c:2109).
///
/// True if the current NPC has an unobstructed line of fire from its weapon muzzle to
/// `ent`'s current origin. Blaster shots trace with a small `±2` aim-error box; everything
/// else is a point trace. A start/all-solid trace fails. The commented-out
/// `WP_BLASTER_PISTOL`/`TEAM_SCAVENGERS` notes are kept. No oracle (`trap_Trace` +
/// process-global `NPC`); verified by review against the C.
///
/// # Safety
/// `ent` may be NULL (handled); `NPC`/`NPC->client` must be set for the current think.
pub unsafe fn NPC_ClearShot(ent: *mut gentity_t) -> qboolean {
    let mut muzzle: vec3_t = [0.0; 3];

    if (*addr_of!(NPC)).is_null() || ent.is_null() {
        return QFALSE;
    }

    CalcEntitySpot(*addr_of!(NPC), SPOT_WEAPON, &mut muzzle);

    // add aim error
    // use weapon instead of specific npc types, although you could add certain npc classes if you wanted
    //	if ( NPC->client->playerTeam == TEAM_SCAVENGERS )
    let tr;
    if (*(*addr_of!(NPC))).s.weapon == WP_BLASTER
    /*|| NPC->s.weapon == WP_BLASTER_PISTOL*/
    {
        // any other guns to check for?
        let mins: vec3_t = [-2.0, -2.0, -2.0];
        let maxs: vec3_t = [2.0, 2.0, 2.0];

        tr = trap::Trace(
            &muzzle,
            &mins,
            &maxs,
            &(*ent).r.currentOrigin,
            (*(*addr_of!(NPC))).s.number,
            MASK_SHOT,
        );
    } else {
        tr = trap::Trace(
            &muzzle,
            &vec3_origin,
            &vec3_origin,
            &(*ent).r.currentOrigin,
            (*(*addr_of!(NPC))).s.number,
            MASK_SHOT,
        );
    }

    if tr.startsolid != 0 || tr.allsolid != 0 {
        return QFALSE;
    }

    if tr.entityNum as c_int == (*ent).s.number {
        return QTRUE;
    }

    QFALSE
}

/*
-------------------------
NPC_ShotEntity
-------------------------
*/
/// `int NPC_ShotEntity( gentity_t *ent, vec3_t impactPos )` (NPC_combat.c:2151).
///
/// Trace the current NPC's shot toward `ent`'s chest and return the entity number it would
/// actually hit; if `impactPos` is non-NULL, also write the impact point there. The
/// thermal detonator aims from slightly above the head (tracing 8 units forward, +24 up,
/// using `viewangles[YAW]` only). Blaster shots use a small `±2` aim-error box; otherwise
/// a point trace. The commented-out start/all-solid early-out is kept (NPCs may shoot even
/// when the muzzle would be inside the target). No oracle (`trap_Trace` + process-global
/// `NPC`); verified by review against the C.
///
/// # Safety
/// `ent` may be NULL (handled); `impactPos` may be NULL; `NPC`/`NPC->client` must be set.
pub unsafe fn NPC_ShotEntity(ent: *mut gentity_t, impactPos: *mut vec3_t) -> c_int {
    let mut muzzle: vec3_t = [0.0; 3];
    let mut targ: vec3_t = [0.0; 3];

    if (*addr_of!(NPC)).is_null() || ent.is_null() {
        return QFALSE as c_int;
    }

    if (*(*addr_of!(NPC))).s.weapon == WP_THERMAL {
        //thermal aims from slightly above head
        //FIXME: what about low-angle shots, rolling the thermal under something?
        let mut angles: vec3_t = [0.0; 3];
        let mut forward: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];

        CalcEntitySpot(*addr_of!(NPC), SPOT_HEAD, &mut muzzle);
        VectorSet(
            &mut angles,
            0.0,
            (*(*(*addr_of!(NPC))).client).ps.viewangles[1],
            0.0,
        );
        AngleVectors(&angles, Some(&mut forward), None, None);
        VectorMA(&muzzle, 8.0, &forward, &mut end);
        end[2] += 24.0;
        let tr = trap::Trace(
            &muzzle,
            &vec3_origin,
            &vec3_origin,
            &end,
            (*(*addr_of!(NPC))).s.number,
            MASK_SHOT,
        );
        VectorCopy(&tr.endpos, &mut muzzle);
    } else {
        CalcEntitySpot(*addr_of!(NPC), SPOT_WEAPON, &mut muzzle);
    }
    CalcEntitySpot(ent, SPOT_CHEST, &mut targ);

    // add aim error
    // use weapon instead of specific npc types, although you could add certain npc classes if you wanted
    //	if ( NPC->client->playerTeam == TEAM_SCAVENGERS )
    let tr;
    if (*(*addr_of!(NPC))).s.weapon == WP_BLASTER
    /*|| NPC->s.weapon == WP_BLASTER_PISTOL*/
    {
        // any other guns to check for?
        let mins: vec3_t = [-2.0, -2.0, -2.0];
        let maxs: vec3_t = [2.0, 2.0, 2.0];

        tr = trap::Trace(
            &muzzle,
            &mins,
            &maxs,
            &targ,
            (*(*addr_of!(NPC))).s.number,
            MASK_SHOT,
        );
    } else {
        tr = trap::Trace(
            &muzzle,
            &vec3_origin,
            &vec3_origin,
            &targ,
            (*(*addr_of!(NPC))).s.number,
            MASK_SHOT,
        );
    }
    //FIXME: if using a bouncing weapon like the bowcaster, should we check the reflection of the wall, too?
    if !impactPos.is_null() {
        //they want to know *where* the hit would be, too
        VectorCopy(&tr.endpos, &mut *impactPos);
    }
    /* // NPCs should be able to shoot even if the muzzle would be inside their target
        if ( tr.startsolid || tr.allsolid )
        {
            return ENTITYNUM_NONE;
        }
    */
    tr.entityNum as c_int
}

/// `qboolean NPC_EvaluateShot( int hit, qboolean glassOK )` (NPC_combat.c:2207).
///
/// Given the entity number a shot `hit` would strike, decide whether to fire anyway: yes
/// if it's the current enemy, or if the hit ent is a `SVF_GLASS_BRUSH` (glass we can shoot
/// through). Returns `qfalse` with no enemy. The `glassOK` parameter is unused in the C
/// body (the glass check is unconditional). No oracle (process-global `NPC`/`g_entities`);
/// verified by review against the C.
///
/// # Safety
/// `NPC` must be set; `hit` must be a valid `g_entities` index.
pub unsafe fn NPC_EvaluateShot(hit: c_int, _glassOK: qboolean) -> qboolean {
    if (*(*addr_of!(NPC))).enemy.is_null() {
        return QFALSE;
    }

    if hit == (*(*(*addr_of!(NPC))).enemy).s.number
        || ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(hit as isize))
            .r
            .svFlags
            & SVF_GLASS_BRUSH)
            != 0
    {
        //can hit enemy or will hit glass, so shoot anyway
        return QTRUE;
    }
    QFALSE
}

/*
NPC_CheckAttack

Simply checks aggression and returns true or false
*/
/// `qboolean NPC_CheckAttack( float scale )` (NPC_combat.c:2227).
///
/// True if the current NPC should attack now: its `aggression` (×`scale`, defaulting to
/// 1.0) must beat a random `[0,4)` roll, and its `shotTime` debounce must have elapsed.
/// No oracle (process-global `NPCInfo`/`level` + `flrand`); verified by review against
/// the C.
///
/// # Safety
/// `NPCInfo` must be set; `level` must be initialised.
pub unsafe fn NPC_CheckAttack(mut scale: f32) -> qboolean {
    if scale == 0.0 {
        scale = 1.0;
    }

    if ((*(*addr_of!(NPCInfo))).stats.aggression as f32) * scale < flrand(0.0, 4.0) {
        return QFALSE;
    }

    if (*(*addr_of!(NPCInfo))).shotTime > (*addr_of!(level)).time {
        return QFALSE;
    }

    QTRUE
}

/*
NPC_CheckDefend

Simply checks evasion and returns true or false
*/
/// `qboolean NPC_CheckDefend( float scale )` (NPC_combat.c:2249).
///
/// True if the current NPC should evade now: its `evasion` stat must beat a random
/// `[0, 4*scale)` roll (`scale` defaulting to 1.0). No oracle (process-global `NPCInfo` +
/// `random`); verified by review against the C.
///
/// # Safety
/// `NPCInfo` must be set.
pub unsafe fn NPC_CheckDefend(mut scale: f32) -> qboolean {
    if scale == 0.0 {
        scale = 1.0;
    }

    if (*(*addr_of!(NPCInfo))).stats.evasion as f32 > random() * 4.0 * scale {
        return QTRUE;
    }

    QFALSE
}

/// `qboolean NPC_CheckCanAttack (float attack_scale, qboolean stationary)`
/// (NPC_combat.c:2262).
///
/// Decides whether the NPC can fire on its enemy this frame, and (if so) drives the
/// firing through `WeaponThink`. Yaws toward the enemy head spot (with aim wiggle),
/// rejects the shot if the enemy is out of range, mid-reload, or `SCF_DONT_FIRE`, then
/// checks FOV visibility; in FOV it ducks instead of shooting if the enemy is attacking
/// it (`NPC_CheckDefend`), traces along the actual aim to find where the shot lands
/// (`ShotThroughGlass`), and scales aggression by how dead-on the aim is. Finally, if
/// `NPC_CheckAttack` passes it sets `enemyVisibility = VIS_SHOOT` and calls
/// `WeaponThink(qtrue)`. The `stationary` parameter is unused in the C body and is
/// dropped. The large commented-out alternate-trace and team-target blocks plus the
/// `if(0)` (rwwFIXMEFIXME ExplodeDeath_Wait) branch are kept verbatim. No oracle
/// (process-global `NPC`/`NPCInfo`/`client`/`ucmd`/`enemyVisibility`, `trap_Trace`);
/// verified by review against the C.
///
/// # Safety
/// `NPC`/`NPCInfo`/`client`/`ucmd` must be set and `NPC->enemy` non-null; `level`
/// initialised.
pub unsafe fn NPC_CheckCanAttack(mut attack_scale: f32, _stationary: qboolean) -> qboolean {
    let mut delta: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut angle_to_enemy: vec3_t = [0.0; 3];
    let mut hitspot: vec3_t = [0.0; 3];
    let mut muzzle: vec3_t = [0.0; 3];
    let mut diff: vec3_t = [0.0; 3];
    let mut enemy_org: vec3_t = [0.0; 3]; //, enemy_head;
    let mut attack_ok: qboolean = QFALSE;
    //	qboolean	duck_ok = qfalse;
    let mut dead_on: qboolean = QFALSE;
    let mut aim_off: f32;
    let max_aim_off: f32 = 128.0 - (16.0 * (*(*addr_of!(NPCInfo))).stats.aim as f32);

    if ((*(*(*addr_of!(NPC))).enemy).flags & FL_NOTARGET) != 0 {
        return QFALSE;
    }

    //FIXME: only check to see if should duck if that provides cover from the
    //enemy!!!
    if attack_scale == 0.0 {
        attack_scale = 1.0;
    }
    //Yaw to enemy
    CalcEntitySpot((*(*addr_of!(NPC))).enemy, SPOT_HEAD, &mut enemy_org);
    NPC_AimWiggle(&mut enemy_org);

    CalcEntitySpot(*addr_of!(NPC), SPOT_WEAPON, &mut muzzle);

    VectorSubtract(&enemy_org, &muzzle, &mut delta);
    vectoangles(&delta, &mut angle_to_enemy);
    let distance_to_enemy = VectorNormalize(&mut delta);

    (*(*(*addr_of!(NPC))).NPC).desiredYaw = angle_to_enemy[YAW];
    NPC_UpdateFiringAngles(QFALSE, QTRUE);

    if NPC_EnemyTooFar(
        (*(*addr_of!(NPC))).enemy,
        distance_to_enemy * distance_to_enemy,
        QTRUE,
    ) != QFALSE
    {
        //Too far away?  Do not attack
        return QFALSE;
    }

    if (*(*addr_of!(client))).ps.weaponTime > 0 {
        //already waiting for a shot to fire
        (*(*(*addr_of!(NPC))).NPC).desiredPitch = angle_to_enemy[PITCH];
        NPC_UpdateFiringAngles(QTRUE, QFALSE);
        return QFALSE;
    }

    if ((*(*addr_of!(NPCInfo))).scriptFlags & SCF_DONT_FIRE) != 0 {
        return QFALSE;
    }

    (*(*addr_of!(NPCInfo))).enemyLastVisibility = enemyVisibility;
    //See if they're in our FOV and we have a clear shot to them
    enemyVisibility = NPC_CheckVisibility((*(*addr_of!(NPC))).enemy, CHECK_360 | CHECK_FOV); ////CHECK_PVS|

    if enemyVisibility >= VIS_FOV {
        //He's in our FOV

        attack_ok = QTRUE;
        //CalcEntitySpot( NPC->enemy, SPOT_HEAD, enemy_head);

        //Check to duck
        if !(*(*(*addr_of!(NPC))).enemy).client.is_null()
            && (*(*(*addr_of!(NPC))).enemy).enemy == *addr_of!(NPC)
            && ((*(*(*(*addr_of!(NPC))).enemy).client).buttons & BUTTON_ATTACK) != 0
        {
            //FIXME: determine if enemy fire angles would hit me or get close
            if NPC_CheckDefend(1.0) != QFALSE {
                //FIXME: Check self-preservation?  Health?
                //duck and don't shoot
                attack_ok = QFALSE;
                (*addr_of_mut!(ucmd)).upmove = -127;
            }
        }

        if attack_ok != QFALSE {
            //are we gonna hit him
            //NEW: use actual forward facing
            AngleVectors(
                &(*(*addr_of!(client))).ps.viewangles,
                Some(&mut forward),
                None,
                None,
            );
            VectorMA(&muzzle, distance_to_enemy, &forward, &mut hitspot);
            let mut tr = trap::Trace(
                &muzzle,
                &vec3_origin,
                &vec3_origin,
                &hitspot,
                (*(*addr_of!(NPC))).s.number,
                MASK_SHOT,
            );
            ShotThroughGlass(&mut tr, (*(*addr_of!(NPC))).enemy, &hitspot, MASK_SHOT);
            /*
            //OLD: trace regardless of facing
            trap_Trace ( &tr, muzzle, NULL, NULL, enemy_org, NPC->s.number, MASK_SHOT );
            ShotThroughGlass(&tr, NPC->enemy, enemy_org, MASK_SHOT);
            */

            let trace_ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities)
                .cast::<gentity_t>())
            .offset(tr.entityNum as isize);

            /*
            if( traceEnt != NPC->enemy &&//FIXME: if someone on our team is in the way, suggest that they duck if possible
                (!traceEnt || !traceEnt->client || !NPC->client->enemyTeam || NPC->client->enemyTeam != traceEnt->client->playerTeam) )
            {//no, so shoot for somewhere between the head and torso
                //NOTE: yes, I know this looks weird, but it works
                enemy_org[0] += 0.3*Q_flrand(NPC->enemy->r.mins[0], NPC->enemy->r.maxs[0]);
                enemy_org[1] += 0.3*Q_flrand(NPC->enemy->r.mins[1], NPC->enemy->r.maxs[1]);
                enemy_org[2] -= NPC->enemy->r.maxs[2]*Q_flrand(0.0f, 1.0f);

                attack_scale *= 0.75;
                trap_Trace ( &tr, muzzle, NULL, NULL, enemy_org, NPC->s.number, MASK_SHOT );
                ShotThroughGlass(&tr, NPC->enemy, enemy_org, MASK_SHOT);
                traceEnt = &g_entities[tr.entityNum];
            }
            */

            VectorCopy(&tr.endpos, &mut hitspot);

            if trace_ent == (*(*addr_of!(NPC))).enemy
                || (!(*trace_ent).client.is_null()
                    && (*(*(*addr_of!(NPC))).client).enemyTeam != 0
                    && (*(*(*addr_of!(NPC))).client).enemyTeam == (*(*trace_ent).client).playerTeam)
            {
                dead_on = QTRUE;
            } else {
                attack_scale *= 0.5;
                if (*(*(*addr_of!(NPC))).client).playerTeam != 0
                    && !trace_ent.is_null()
                    && !(*trace_ent).client.is_null()
                    && (*(*trace_ent).client).playerTeam != 0
                    && (*(*(*addr_of!(NPC))).client).playerTeam == (*(*trace_ent).client).playerTeam
                {
                    //Don't shoot our own team
                    attack_ok = QFALSE;
                }
            }

            if attack_ok != QFALSE {
                //ok, now adjust pitch aim
                VectorSubtract(&hitspot, &muzzle, &mut delta);
                vectoangles(&delta, &mut angle_to_enemy);
                (*(*(*addr_of!(NPC))).NPC).desiredPitch = angle_to_enemy[PITCH];
                NPC_UpdateFiringAngles(QTRUE, QFALSE);

                if dead_on == QFALSE {
                    //We're not going to hit him directly, try a suppressing fire
                    //see if where we're going to shoot is too far from his origin
                    if !trace_ent.is_null()
                        && ((*trace_ent).health <= 30 || EntIsGlass(trace_ent) != QFALSE)
                    {
                        //easy to kill - go for it
                        //if(traceEnt->die == ExplodeDeath_Wait && traceEnt->splashDamage)
                        if false
                        //rwwFIXMEFIXME: ExplodeDeath_Wait?
                        {
                            //going to explode, don't shoot if close to self
                            VectorSubtract(
                                &(*(*addr_of!(NPC))).r.currentOrigin,
                                &(*trace_ent).r.currentOrigin,
                                &mut diff,
                            );
                            if VectorLengthSquared(&diff)
                                < ((*trace_ent).splashRadius * (*trace_ent).splashRadius) as f32
                            {
                                //Too close to shoot!
                                attack_ok = QFALSE;
                            } else {
                                //Hey, it might kill him, do it!
                                attack_scale *= 2.0; //
                            }
                        }
                    } else {
                        AngleVectors(
                            &(*(*addr_of!(client))).ps.viewangles,
                            Some(&mut forward),
                            None,
                            None,
                        );
                        VectorMA(&muzzle, distance_to_enemy, &forward, &mut hitspot);
                        VectorSubtract(&hitspot, &enemy_org, &mut diff);
                        aim_off = VectorLength(&diff);
                        if aim_off > random() * max_aim_off {
                            //FIXME: use aim value to allow poor aim?
                            attack_scale *= 0.75;
                            //see if where we're going to shoot is too far from his head
                            VectorSubtract(&hitspot, &enemy_org, &mut diff);
                            aim_off = VectorLength(&diff);
                            if aim_off > random() * max_aim_off {
                                attack_ok = QFALSE;
                            }
                        }
                        attack_scale *= (max_aim_off - aim_off + 1.0) / max_aim_off;
                    }
                }
            }
        }
    } else {
        //Update pitch anyway
        (*(*(*addr_of!(NPC))).NPC).desiredPitch = angle_to_enemy[PITCH];
        NPC_UpdateFiringAngles(QTRUE, QFALSE);
    }

    if attack_ok != QFALSE {
        if NPC_CheckAttack(attack_scale) != QFALSE {
            //check aggression to decide if we should shoot
            enemyVisibility = VIS_SHOOT;
            WeaponThink(QTRUE);
        } else {
            attack_ok = QFALSE;
        }
    }

    attack_ok
}

/*
-------------------------
ShootThink
-------------------------
*/
/// `void ShootThink( void )` (NPC_combat.c:924).
///
/// Decides whether the current NPC presses its fire button this think, and stamps the next
/// `shotTime`/`attackDebounceTime`. Clears `BUTTON_ATTACK`, bails on `WP_NONE`, on a weapon
/// state that isn't ready/firing/idle, or while still inside `shotTime`. Otherwise raises
/// `BUTTON_ATTACK`, refreshes `currentAmmo`, calls [`NPC_ApplyWeaponFireDelay`], and computes
/// the post-shot `delay` — burst weapons (`NPCAI_BURST_WEAPON`) seed/decrement `burstCount`
/// and only space out at the end of a burst, with a dirty `WP_EMPLACED_GUN` skill-tiered
/// override (pulling from `NPC->parent->random` when chair-mounted); non-burst weapons just
/// use `burstSpacing`. The commented-out `enemyVisibility != VIS_SHOOT` guard and the
/// `erandom` burst-count block are kept commented. No oracle (process-global
/// `ucmd`/`client`/`NPC`/`NPCInfo`/`level` reads+mutations); verified by review against the C.
///
/// # Safety
/// `NPC`/`NPCInfo`/`client` must be set; `level`/`g_spskill`/`weaponData` must be initialised.
pub unsafe fn ShootThink() {
    let mut delay: c_int;

    (*addr_of_mut!(ucmd)).buttons &= !BUTTON_ATTACK;
    /*
        if ( enemyVisibility != VIS_SHOOT)
            return;
    */

    if (*(*addr_of!(client))).ps.weapon == WP_NONE {
        return;
    }

    if (*(*addr_of!(client))).ps.weaponstate != WEAPON_READY
        && (*(*addr_of!(client))).ps.weaponstate != WEAPON_FIRING
        && (*(*addr_of!(client))).ps.weaponstate != WEAPON_IDLE
    {
        return;
    }

    if (*addr_of!(level)).time < (*(*addr_of!(NPCInfo))).shotTime {
        return;
    }

    (*addr_of_mut!(ucmd)).buttons |= BUTTON_ATTACK;

    (*(*addr_of!(NPCInfo))).currentAmmo = (*(*addr_of!(client))).ps.ammo
        [(*addr_of!(weaponData))[(*(*addr_of!(client))).ps.weapon as usize].ammoIndex as usize]; // checkme

    NPC_ApplyWeaponFireDelay();

    if ((*(*addr_of!(NPCInfo))).aiFlags & NPCAI_BURST_WEAPON) != 0 {
        if (*(*addr_of!(NPCInfo))).burstCount == 0 {
            (*(*addr_of!(NPCInfo))).burstCount = Q_irand(
                (*(*addr_of!(NPCInfo))).burstMin,
                (*(*addr_of!(NPCInfo))).burstMax,
            );
            /*
            NPCInfo->burstCount = erandom( NPCInfo->burstMean );
            if ( NPCInfo->burstCount < NPCInfo->burstMin )
            {
                NPCInfo->burstCount = NPCInfo->burstMin;
            }
            else if ( NPCInfo->burstCount > NPCInfo->burstMax )
            {
                NPCInfo->burstCount = NPCInfo->burstMax;
            }
            */
            delay = 0;
        } else {
            (*(*addr_of!(NPCInfo))).burstCount -= 1;
            if (*(*addr_of!(NPCInfo))).burstCount == 0 {
                delay = (*(*addr_of!(NPCInfo))).burstSpacing;
            } else {
                delay = 0;
            }
        }

        if delay == 0 {
            // HACK: dirty little emplaced bits, but is done because it would otherwise require some sort of new variable...
            if (*(*addr_of!(client))).ps.weapon == WP_EMPLACED_GUN {
                if !(*(*addr_of!(NPC))).parent.is_null() {
                    // try and get the debounce values from the chair if we can
                    if (*addr_of!(g_spskill)).integer == 0 {
                        delay = ((*(*(*addr_of!(NPC))).parent).random + 150.0) as c_int;
                    } else if (*addr_of!(g_spskill)).integer == 1 {
                        delay = ((*(*(*addr_of!(NPC))).parent).random + 100.0) as c_int;
                    } else {
                        delay = (*(*(*addr_of!(NPC))).parent).random as c_int;
                    }
                } else if (*addr_of!(g_spskill)).integer == 0 {
                    delay = 350;
                } else if (*addr_of!(g_spskill)).integer == 1 {
                    delay = 300;
                } else {
                    delay = 200;
                }
            }
        }
    } else {
        delay = (*(*addr_of!(NPCInfo))).burstSpacing;
    }

    (*(*addr_of!(NPCInfo))).shotTime = (*addr_of!(level)).time + delay;
    (*(*addr_of!(NPC))).attackDebounceTime =
        (*addr_of!(level)).time + NPC_AttackDebounceForWeapon();
}

/*
static void WeaponThink( qboolean inCombat )
FIXME makes this so there's a delay from event that caused us to check to actually doing it

Added: hacks for Borg
*/
/// `void WeaponThink( qboolean inCombat )` (NPC_combat.c:1035).
///
/// Per-think weapon bookkeeping for the current NPC: while raising/dropping it just holds
/// the current weapon and releases the fire button; otherwise it tops up ammo (no NPC runs
/// dry — under 10 rounds gets +100 via [`Add_Ammo`]), sets `ucmd.weapon` to the current
/// weapon, and calls [`ShootThink`] to decide firing. The big Borg/Scavenger team hacks and
/// the out-of-ammo `ChooseBestWeapon` block are kept commented exactly as in the C. The
/// `inCombat` argument is (as in the C) unused. No oracle (process-global
/// `ucmd`/`client`/`NPC`; `Add_Ammo` mutates the entity); verified by review against the C.
///
/// # Safety
/// `NPC`/`NPC->client`/`client` must be set; `weaponData` must be initialised.
pub unsafe fn WeaponThink(_inCombat: qboolean) {
    if (*(*addr_of!(client))).ps.weaponstate == WEAPON_RAISING
        || (*(*addr_of!(client))).ps.weaponstate == WEAPON_DROPPING
    {
        (*addr_of_mut!(ucmd)).weapon = (*(*addr_of!(client))).ps.weapon as u8;
        (*addr_of_mut!(ucmd)).buttons &= !BUTTON_ATTACK;
        return;
    }

    //MCG - Begin
    //For now, no-one runs out of ammo
    if (*(*(*addr_of!(NPC))).client).ps.ammo
        [(*addr_of!(weaponData))[(*(*addr_of!(client))).ps.weapon as usize].ammoIndex as usize]
        < 10
    // checkme
    //	if(NPC->client->ps.ammo[ client->ps.weapon ] < 10)
    {
        Add_Ammo(*addr_of!(NPC), (*(*addr_of!(client))).ps.weapon, 100);
    }

    /*if ( NPC->playerTeam == TEAM_BORG )
    {//HACK!!!
        if(!(NPC->client->ps.stats[STAT_WEAPONS] & ( 1 << WP_BORG_WEAPON )))
            NPC->client->ps.stats[STAT_WEAPONS] |= ( 1 << WP_BORG_WEAPON );

        if ( client->ps.weapon != WP_BORG_WEAPON )
        {
            NPC_ChangeWeapon( WP_BORG_WEAPON );
            Add_Ammo (NPC, client->ps.weapon, 10);
            NPCInfo->currentAmmo = client->ps.ammo[client->ps.weapon];
        }
    }
    else */

    /*if ( NPC->client->playerTeam == TEAM_SCAVENGERS )
    {//HACK!!!
        if(!(NPC->client->ps.stats[STAT_WEAPONS] & ( 1 << WP_BLASTER )))
            NPC->client->ps.stats[STAT_WEAPONS] |= ( 1 << WP_BLASTER );

        if ( client->ps.weapon != WP_BLASTER )

        {
            NPC_ChangeWeapon( WP_BLASTER );
            Add_Ammo (NPC, client->ps.weapon, 10);
    //			NPCInfo->currentAmmo = client->ps.ammo[client->ps.weapon];
            NPCInfo->currentAmmo = client->ps.ammo[weaponData[client->ps.weapon].ammoIndex];	// checkme
        }
    }
    else*/
    //MCG - End
    {
        // if the gun in our hands is out of ammo, we need to change
        /*if ( client->ps.ammo[client->ps.weapon] == 0 )
        {
            NPCInfo->aiFlags |= NPCAI_CHECK_WEAPON;
        }

        if ( NPCInfo->aiFlags & NPCAI_CHECK_WEAPON )
        {
            NPCInfo->aiFlags &= ~NPCAI_CHECK_WEAPON;
            bestWeapon = ChooseBestWeapon();
            if ( bestWeapon != client->ps.weapon )
            {
                NPC_ChangeWeapon( bestWeapon );
            }
        }*/
    }

    (*addr_of_mut!(ucmd)).weapon = (*(*addr_of!(client))).ps.weapon as u8;
    ShootThink();
}

/*
void NPC_CheckPossibleEnemy( gentity_t *other, visibility_t vis )

Added: hacks for scripted NPCs
*/
/// `void NPC_CheckPossibleEnemy( gentity_t *other, visibility_t vis )` (NPC_combat.c:1228).
///
/// Considers `other` as a candidate enemy for the current NPC. Bails if `other` is already
/// the enemy or is notargeted. If we already have an enemy and `other` is only in our FOV,
/// keep the current enemy unless we last saw them more than 2s ago and they're no longer in
/// FOV (recomputing `enemyVisibility` via [`NPC_CheckVisibility`] when unknown). Only takes a
/// brand-new enemy via [`G_SetEnemy`] when we have none. Then records last-seen vs last-heard
/// position/time based on whether `vis == VIS_FOV`. No oracle (process-global
/// `NPC`/`NPCInfo`/`enemyVisibility`/`level`); verified by review against the C.
///
/// # Safety
/// `other` must point to a valid `gentity_t`; `NPC`/`NPCInfo` must be set.
pub unsafe fn NPC_CheckPossibleEnemy(other: *mut gentity_t, vis: visibility_t) {
    // is he is already our enemy?
    if other == (*(*addr_of!(NPC))).enemy {
        return;
    }

    if ((*other).flags & FL_NOTARGET) != 0 {
        return;
    }

    // we already have an enemy and this guy is in our FOV, see if this guy would be better
    if !(*(*addr_of!(NPC))).enemy.is_null() && vis == VIS_FOV {
        if (*(*addr_of!(NPCInfo))).enemyLastSeenTime - (*addr_of!(level)).time < 2000 {
            return;
        }
        if *addr_of!(enemyVisibility) == VIS_UNKNOWN {
            enemyVisibility = NPC_CheckVisibility((*(*addr_of!(NPC))).enemy, CHECK_360 | CHECK_FOV);
        }
        if *addr_of!(enemyVisibility) == VIS_FOV {
            return;
        }
    }

    if (*(*addr_of!(NPC))).enemy.is_null() {
        //only take an enemy if you don't have one yet
        G_SetEnemy(*addr_of!(NPC), other);
    }

    if vis == VIS_FOV {
        (*(*addr_of!(NPCInfo))).enemyLastSeenTime = (*addr_of!(level)).time;
        VectorCopy(
            &(*other).r.currentOrigin,
            &mut (*(*addr_of!(NPCInfo))).enemyLastSeenLocation,
        );
        (*(*addr_of!(NPCInfo))).enemyLastHeardTime = 0;
        VectorClear(&mut (*(*addr_of!(NPCInfo))).enemyLastHeardLocation);
    } else {
        (*(*addr_of!(NPCInfo))).enemyLastSeenTime = 0;
        VectorClear(&mut (*(*addr_of!(NPCInfo))).enemyLastSeenLocation);
        (*(*addr_of!(NPCInfo))).enemyLastHeardTime = (*addr_of!(level)).time;
        VectorCopy(
            &(*other).r.currentOrigin,
            &mut (*(*addr_of!(NPCInfo))).enemyLastHeardLocation,
        );
    }
}

/*
-------------------------
NPC_PickEnemy

Randomly picks a living enemy from the specified team and returns it

FIXME: For now, you MUST specify an enemy team

If you specify choose closest, it will find only the closest enemy

If you specify checkVis, it will return and enemy that is visible

If you specify findPlayersFirst, it will try to find players first

You can mix and match any of those options (example: find closest visible players first)

FIXME: this should go through the snapshot and find the closest enemy
*/
/// `gentity_t *NPC_PickEnemy( gentity_t *closestTo, int enemyTeam, qboolean checkVis, qboolean findPlayersFirst, qboolean findClosest )` (NPC_combat.c:1504).
///
/// Randomly picks (or finds the closest) living enemy of the current NPC. `findPlayersFirst`
/// first probes `g_entities[0]` (the player) under the same vis/hidden/range gates as the
/// general loop, and if it yields a usable result that wins outright. Otherwise the whole
/// `g_entities[0 .. level.num_entities]` array is scanned; candidates are filtered by
/// `NPC_ValidEnemy` (ported as `ValidEnemy`) for clients or `alliedTeam == enemyTeam` for
/// non-clients, by PVS, by the patrol/investigate vis cone, by the `hiddenDist`/`hiddenDir`
/// stealth check, and by `NPC_EnemyTooFar`. When `findClosest` the nearest qualifier is
/// returned; otherwise one of the qualifying `choice[]` slots is picked at random. Formation
/// states (`BS_STAND_AND_SHOOT`/`BS_HUNT_AND_KILL`) drop the FOV check. No oracle
/// (process-global `g_entities`/`level`/`NPC`/`NPCInfo` + `rand()`); verified by review
/// against the C.
///
/// # Safety
/// `closestTo`/`NPC`/`NPC->client` must be valid; `g_entities`/`level` must be initialised.
#[allow(unused_assignments)] // faithful: C sets bestDist in the player-first block, then dead-resets it before the main loop
pub unsafe fn NPC_PickEnemy(
    closestTo: *mut gentity_t,
    enemyTeam: c_int,
    checkVis: qboolean,
    findPlayersFirst: qboolean,
    findClosest: qboolean,
) -> *mut gentity_t {
    let mut num_choices: c_int = 0;
    let mut choice: [c_int; 128] = [0; 128]; //FIXME: need a different way to determine how many choices?
    let mut newenemy: *mut gentity_t;
    let mut closestEnemy: *mut gentity_t = core::ptr::null_mut();
    let mut diff: vec3_t = [0.0; 3];
    let mut relDist: f32;
    let mut bestDist = Q3_INFINITE as f32;
    let mut visChecks = CHECK_360 | CHECK_FOV | CHECK_VISRANGE;
    let mut minVis = VIS_FOV;

    if enemyTeam == NPCTEAM_NEUTRAL {
        return core::ptr::null_mut();
    }

    if (*(*addr_of!(NPCInfo))).behaviorState == BS_STAND_AND_SHOOT
        || (*(*addr_of!(NPCInfo))).behaviorState == BS_HUNT_AND_KILL
    {
        //Formations guys don't require inFov to pick up a target
        //These other behavior states are active battle states and should not
        //use FOV.  FOV checks are for enemies who are patrolling, guarding, etc.
        visChecks &= !CHECK_FOV;
        minVis = VIS_360;
    }

    if findPlayersFirst != QFALSE {
        //try to find a player first
        newenemy = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(0);
        if !(*newenemy).client.is_null()
            && ((*newenemy).flags & FL_NOTARGET) == 0
            && ((*newenemy).s.eFlags & EF_NODRAW) == 0
        {
            if (*newenemy).health > 0 {
                if ValidEnemy(newenemy) != QFALSE
                //enemyTeam == TEAM_PLAYER || newenemy->client->playerTeam == enemyTeam || ( enemyTeam == TEAM_PLAYER )
                {
                    //FIXME:  check for range and FOV or vis?
                    if newenemy != (*(*addr_of!(NPC))).lastEnemy {
                        //Make sure we're not just going back and forth here
                        if trap::InPVS(
                            &(*newenemy).r.currentOrigin,
                            &(*(*addr_of!(NPC))).r.currentOrigin,
                        ) != QFALSE
                        {
                            let mut failed = QFALSE;

                            if (*(*addr_of!(NPCInfo))).behaviorState == BS_INVESTIGATE
                                || (*(*addr_of!(NPCInfo))).behaviorState == BS_PATROL
                            {
                                if (*(*addr_of!(NPC))).enemy.is_null() {
                                    if InVisrange(newenemy) == QFALSE {
                                        failed = QTRUE;
                                    } else if NPC_CheckVisibility(
                                        newenemy,
                                        CHECK_360 | CHECK_FOV | CHECK_VISRANGE,
                                    ) != VIS_FOV
                                    {
                                        failed = QTRUE;
                                    }
                                }
                            }

                            if failed == QFALSE {
                                VectorSubtract(
                                    &(*closestTo).r.currentOrigin,
                                    &(*newenemy).r.currentOrigin,
                                    &mut diff,
                                );
                                relDist = VectorLengthSquared(&diff);
                                if (*(*newenemy).client).hiddenDist > 0.0 {
                                    if relDist
                                        > (*(*newenemy).client).hiddenDist
                                            * (*(*newenemy).client).hiddenDist
                                    {
                                        //out of hidden range
                                        if VectorLengthSquared(&(*(*newenemy).client).hiddenDir)
                                            != 0.0
                                        {
                                            //They're only hidden from a certain direction, check
                                            VectorNormalize(&mut diff);
                                            let dot =
                                                DotProduct(&(*(*newenemy).client).hiddenDir, &diff);
                                            if dot > 0.5 {
                                                //I'm not looking in the right dir toward them to see them
                                                failed = QTRUE;
                                            } else {
                                                Debug_Printf(addr_of!(debugNPCAI), DEBUG_LEVEL_INFO, format_args!(
                                                    "{} saw {} trying to hide - hiddenDir {} targetDir {} dot {}\n",
                                                    core::ffi::CStr::from_ptr((*(*addr_of!(NPC))).targetname).to_string_lossy(),
                                                    core::ffi::CStr::from_ptr((*newenemy).targetname).to_string_lossy(),
                                                    core::ffi::CStr::from_ptr(vtos(&(*(*newenemy).client).hiddenDir)).to_string_lossy(),
                                                    core::ffi::CStr::from_ptr(vtos(&diff)).to_string_lossy(),
                                                    dot,
                                                ));
                                            }
                                        } else {
                                            failed = QTRUE;
                                        }
                                    } else {
                                        Debug_Printf(
                                            addr_of!(debugNPCAI),
                                            DEBUG_LEVEL_INFO,
                                            format_args!(
                                                "{} saw {} trying to hide - hiddenDist {}\n",
                                                core::ffi::CStr::from_ptr(
                                                    (*(*addr_of!(NPC))).targetname
                                                )
                                                .to_string_lossy(),
                                                core::ffi::CStr::from_ptr((*newenemy).targetname)
                                                    .to_string_lossy(),
                                                (*(*newenemy).client).hiddenDist,
                                            ),
                                        );
                                    }
                                }

                                if failed == QFALSE {
                                    if findClosest != QFALSE {
                                        if relDist < bestDist {
                                            if NPC_EnemyTooFar(newenemy, relDist, QFALSE) == QFALSE
                                            {
                                                if checkVis != QFALSE {
                                                    if NPC_CheckVisibility(newenemy, visChecks)
                                                        == minVis
                                                    {
                                                        bestDist = relDist;
                                                        closestEnemy = newenemy;
                                                    }
                                                } else {
                                                    bestDist = relDist;
                                                    closestEnemy = newenemy;
                                                }
                                            }
                                        }
                                    } else if NPC_EnemyTooFar(newenemy, 0.0, QFALSE) == QFALSE {
                                        if checkVis != QFALSE {
                                            if NPC_CheckVisibility(
                                                newenemy,
                                                CHECK_360 | CHECK_FOV | CHECK_VISRANGE,
                                            ) == VIS_FOV
                                            {
                                                choice[num_choices as usize] = (*newenemy).s.number;
                                                num_choices += 1;
                                            }
                                        } else {
                                            choice[num_choices as usize] = (*newenemy).s.number;
                                            num_choices += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if findClosest != QFALSE && !closestEnemy.is_null() {
        return closestEnemy;
    }

    if num_choices != 0 {
        return (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .offset(choice[(rand() % num_choices) as usize] as isize);
    }

    /*
    //FIXME: used to have an option to look *only* for the player... now...?  Still need it?
    if ( enemyTeam == TEAM_PLAYER )
    {//couldn't find the player
        return NULL;
    }
    */

    num_choices = 0;
    bestDist = Q3_INFINITE as f32;
    closestEnemy = core::ptr::null_mut();

    let mut entNum = 0;
    while entNum < (*addr_of!(level)).num_entities {
        newenemy =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(entNum as isize);

        if newenemy != *addr_of!(NPC)
            && !(*newenemy).client.is_null() /*|| newenemy->svFlags & SVF_NONNPC_ENEMY*/
            && ((*newenemy).flags & FL_NOTARGET) == 0
            && ((*newenemy).s.eFlags & EF_NODRAW) == 0
        {
            if (*newenemy).health > 0 {
                if (!(*newenemy).client.is_null() && ValidEnemy(newenemy) != QFALSE)
                    || ((*newenemy).client.is_null() && (*newenemy).alliedTeam == enemyTeam)
                {
                    //FIXME:  check for range and FOV or vis?
                    if (*(*(*addr_of!(NPC))).client).playerTeam == NPCTEAM_PLAYER
                        && enemyTeam == NPCTEAM_PLAYER
                    {
                        //player allies turning on ourselves?  How?
                        if (*newenemy).s.number != 0 {
                            //only turn on the player, not other player allies
                            entNum += 1;
                            continue;
                        }
                    }

                    if newenemy != (*(*addr_of!(NPC))).lastEnemy {
                        //Make sure we're not just going back and forth here
                        if trap::InPVS(
                            &(*newenemy).r.currentOrigin,
                            &(*(*addr_of!(NPC))).r.currentOrigin,
                        ) == QFALSE
                        {
                            entNum += 1;
                            continue;
                        }

                        if (*(*addr_of!(NPCInfo))).behaviorState == BS_INVESTIGATE
                            || (*(*addr_of!(NPCInfo))).behaviorState == BS_PATROL
                        {
                            if (*(*addr_of!(NPC))).enemy.is_null() {
                                if InVisrange(newenemy) == QFALSE {
                                    entNum += 1;
                                    continue;
                                } else if NPC_CheckVisibility(
                                    newenemy,
                                    CHECK_360 | CHECK_FOV | CHECK_VISRANGE,
                                ) != VIS_FOV
                                {
                                    entNum += 1;
                                    continue;
                                }
                            }
                        }

                        VectorSubtract(
                            &(*closestTo).r.currentOrigin,
                            &(*newenemy).r.currentOrigin,
                            &mut diff,
                        );
                        relDist = VectorLengthSquared(&diff);
                        if !(*newenemy).client.is_null() && (*(*newenemy).client).hiddenDist > 0.0 {
                            if relDist
                                > (*(*newenemy).client).hiddenDist
                                    * (*(*newenemy).client).hiddenDist
                            {
                                //out of hidden range
                                if VectorLengthSquared(&(*(*newenemy).client).hiddenDir) != 0.0 {
                                    //They're only hidden from a certain direction, check
                                    VectorNormalize(&mut diff);
                                    let dot = DotProduct(&(*(*newenemy).client).hiddenDir, &diff);
                                    if dot > 0.5 {
                                        //I'm not looking in the right dir toward them to see them
                                        entNum += 1;
                                        continue;
                                    } else {
                                        Debug_Printf(addr_of!(debugNPCAI), DEBUG_LEVEL_INFO, format_args!(
                                            "{} saw {} trying to hide - hiddenDir {} targetDir {} dot {}\n",
                                            core::ffi::CStr::from_ptr((*(*addr_of!(NPC))).targetname).to_string_lossy(),
                                            core::ffi::CStr::from_ptr((*newenemy).targetname).to_string_lossy(),
                                            core::ffi::CStr::from_ptr(vtos(&(*(*newenemy).client).hiddenDir)).to_string_lossy(),
                                            core::ffi::CStr::from_ptr(vtos(&diff)).to_string_lossy(),
                                            dot,
                                        ));
                                    }
                                } else {
                                    entNum += 1;
                                    continue;
                                }
                            } else {
                                Debug_Printf(
                                    addr_of!(debugNPCAI),
                                    DEBUG_LEVEL_INFO,
                                    format_args!(
                                        "{} saw {} trying to hide - hiddenDist {}\n",
                                        core::ffi::CStr::from_ptr((*(*addr_of!(NPC))).targetname)
                                            .to_string_lossy(),
                                        core::ffi::CStr::from_ptr((*newenemy).targetname)
                                            .to_string_lossy(),
                                        (*(*newenemy).client).hiddenDist,
                                    ),
                                );
                            }
                        }

                        if findClosest != QFALSE {
                            if relDist < bestDist {
                                if NPC_EnemyTooFar(newenemy, relDist, QFALSE) == QFALSE {
                                    if checkVis != QFALSE {
                                        //FIXME: NPCs need to be able to pick up other NPCs behind them,
                                        //but for now, commented out because it was picking up enemies it shouldn't
                                        //if ( NPC_CheckVisibility ( newenemy, CHECK_360|CHECK_VISRANGE ) >= VIS_360 )
                                        if NPC_CheckVisibility(newenemy, visChecks) == minVis {
                                            bestDist = relDist;
                                            closestEnemy = newenemy;
                                        }
                                    } else {
                                        bestDist = relDist;
                                        closestEnemy = newenemy;
                                    }
                                }
                            }
                        } else if NPC_EnemyTooFar(newenemy, 0.0, QFALSE) == QFALSE {
                            if checkVis != QFALSE {
                                //if( NPC_CheckVisibility ( newenemy, CHECK_360|CHECK_FOV|CHECK_VISRANGE ) == VIS_FOV )
                                if NPC_CheckVisibility(newenemy, CHECK_360 | CHECK_VISRANGE)
                                    >= VIS_360
                                {
                                    choice[num_choices as usize] = (*newenemy).s.number;
                                    num_choices += 1;
                                }
                            } else {
                                choice[num_choices as usize] = (*newenemy).s.number;
                                num_choices += 1;
                            }
                        }
                    }
                }
            }
        }
        entNum += 1;
    }

    if findClosest != QFALSE {
        //FIXME: you can pick up an enemy around a corner this way.
        return closestEnemy;
    }

    if num_choices == 0 {
        return core::ptr::null_mut();
    }

    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
        .offset(choice[(rand() % num_choices) as usize] as isize)
}

/*
gentity_t *NPC_PickAlly ( void )

  Simply returns closest visible ally
*/
/// `gentity_t *NPC_PickAlly( qboolean facingEachOther, float range, qboolean ignoreGroup, qboolean movingOnly )` (NPC_combat.c:1803).
///
/// Returns the closest living, in-PVS, visible ally of the current NPC within `range`. An
/// ally is a client on the NPC's `playerTeam` (or anyone, if the NPC is `NPCTEAM_ENEMY` —
/// the disguise hack). `ignoreGroup` rejects the NPC's leader and its own followers;
/// `movingOnly` requires a nonzero relative velocity; `facingEachOther` requires each to be
/// facing toward the other (the leader's `dot >= 0.5`, our own `dot <= -0.5` since `diff`
/// points from ally to us). Visibility is `NPC_CheckVisibility(.., CHECK_360|CHECK_VISRANGE)
/// >= VIS_360`. No oracle (process-global `g_entities`/`level`/`NPC`); verified by review
/// against the C.
///
/// # Safety
/// `NPC`/`NPC->client` must be set; `g_entities`/`level` must be initialised.
pub unsafe fn NPC_PickAlly(
    facingEachOther: qboolean,
    range: f32,
    ignoreGroup: qboolean,
    movingOnly: qboolean,
) -> *mut gentity_t {
    let mut ally: *mut gentity_t;
    let mut closestAlly: *mut gentity_t = core::ptr::null_mut();
    let mut diff: vec3_t = [0.0; 3];
    let mut bestDist = range;

    let mut entNum = 0;
    while entNum < (*addr_of!(level)).num_entities {
        ally = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(entNum as isize);

        if !(*ally).client.is_null() {
            if (*ally).health > 0 {
                if !(*ally).client.is_null()
                    && ((*(*ally).client).playerTeam == (*(*(*addr_of!(NPC))).client).playerTeam
                        || (*(*(*addr_of!(NPC))).client).playerTeam == NPCTEAM_ENEMY)
                {
                    //if on same team or if player is disguised as your team
                    if ignoreGroup != QFALSE {
                        if ally == (*(*(*addr_of!(NPC))).client).leader {
                            //reject
                            entNum += 1;
                            continue;
                        }
                        if !(*ally).client.is_null()
                            && !(*(*ally).client).leader.is_null()
                            && (*(*ally).client).leader == *addr_of!(NPC)
                        {
                            //reject
                            entNum += 1;
                            continue;
                        }
                    }

                    if trap::InPVS(
                        &(*ally).r.currentOrigin,
                        &(*(*addr_of!(NPC))).r.currentOrigin,
                    ) == QFALSE
                    {
                        entNum += 1;
                        continue;
                    }

                    if movingOnly != QFALSE
                        && !(*ally).client.is_null()
                        && !(*(*addr_of!(NPC))).client.is_null()
                    {
                        //They have to be moving relative to each other
                        if DistanceSquared(
                            &(*(*ally).client).ps.velocity,
                            &(*(*(*addr_of!(NPC))).client).ps.velocity,
                        ) == 0.0
                        {
                            entNum += 1;
                            continue;
                        }
                    }

                    VectorSubtract(
                        &(*(*addr_of!(NPC))).r.currentOrigin,
                        &(*ally).r.currentOrigin,
                        &mut diff,
                    );
                    let relDist = VectorNormalize(&mut diff);
                    if relDist < bestDist {
                        if facingEachOther != QFALSE {
                            let mut vf: vec3_t = [0.0; 3];

                            AngleVectors(
                                &(*(*ally).client).ps.viewangles,
                                Some(&mut vf),
                                None,
                                None,
                            );
                            VectorNormalize(&mut vf);
                            let mut dot = DotProduct(&diff, &vf);

                            if dot < 0.5 {
                                //Not facing in dir to me
                                entNum += 1;
                                continue;
                            }
                            //He's facing me, am I facing him?
                            AngleVectors(
                                &(*(*(*addr_of!(NPC))).client).ps.viewangles,
                                Some(&mut vf),
                                None,
                                None,
                            );
                            VectorNormalize(&mut vf);
                            dot = DotProduct(&diff, &vf);

                            if dot > -0.5 {
                                //I'm not facing opposite of dir to me
                                entNum += 1;
                                continue;
                            }
                            //I am facing him
                        }

                        if NPC_CheckVisibility(ally, CHECK_360 | CHECK_VISRANGE) >= VIS_360 {
                            bestDist = relDist;
                            closestAlly = ally;
                        }
                    }
                }
            }
        }
        entNum += 1;
    }

    closestAlly
}

/// `gentity_t *NPC_CheckEnemy( qboolean findNew, qboolean tooFarOk, qboolean setEnemy )`
/// (NPC_combat.c:1894).
///
/// Validates the current NPC's enemy (drops it if gone/dead/out-of-PVS) and, when allowed,
/// acquires a new one — taking a protected `defendEnt`'s enemy, else `NPC_PickEnemy` on the
/// enemy team. Reads the process-global `NPC`/`NPCInfo`/`client`; no oracle (entity-state).
/// The `if(0)` `SVF_IGNORE_ENEMIES` path and the commented-out `SVF_LOCKEDENEMY` /
/// behaviorState-switch blocks are carried over verbatim (still dead, as in the C).
///
/// # Safety
/// Dereferences the live `NPC`/`NPCInfo`/`client` globals and their enemy/`defendEnt`
/// entities; the caller must run inside an NPC think with those set.
pub unsafe fn NPC_CheckEnemy(
    findNew: qboolean,
    tooFarOk: qboolean,
    setEnemy: qboolean,
) -> *mut gentity_t {
    let mut forcefindNew: qboolean = QFALSE;
    let mut newEnemy: *mut gentity_t = core::ptr::null_mut();
    //FIXME: have a "NPCInfo->persistance" we can set to determine how long to try to shoot
    //someone we can't hit?  Rather than hard-coded 10?

    //FIXME they shouldn't recognize enemy's death instantly

    //TEMP FIX:
    //if(NPC->enemy->client)
    //{
    //	NPC->enemy->health = NPC->enemy->client->ps.stats[STAT_HEALTH];
    //}

    if !(*(*addr_of!(NPC))).enemy.is_null() {
        if (*(*(*addr_of!(NPC))).enemy).inuse == QFALSE
        //|| NPC->enemy == NPC )//wtf?  NPCs should never get mad at themselves!
        {
            if setEnemy != QFALSE {
                G_ClearEnemy(*addr_of!(NPC));
            }
        }
    }

    //if ( NPC->svFlags & SVF_IGNORE_ENEMIES )
    if false
    //rwwFIXMEFIXME: support for this flag
    {
        //We're ignoring all enemies for now
        if setEnemy != QFALSE {
            G_ClearEnemy(*addr_of!(NPC));
        }
        return core::ptr::null_mut();
    }

    //rwwFIXMEFIXME: support for this flag
    /*
    if ( NPC->svFlags & SVF_LOCKEDENEMY )
    {//keep this enemy until dead
        if ( NPC->enemy )
        {
            if ( (!NPC->NPC && !(NPC->svFlags & SVF_NONNPC_ENEMY) ) || NPC->enemy->health > 0 )
            {//Enemy never had health (a train or info_not_null, etc) or did and is now dead (NPCs, turrets, etc)
                return NULL;
            }
        }
        NPC->svFlags &= ~SVF_LOCKEDENEMY;
    }
    */

    if !(*(*addr_of!(NPC))).enemy.is_null() {
        if NPC_EnemyTooFar((*(*addr_of!(NPC))).enemy, 0.0, QFALSE) != QFALSE {
            if findNew != QFALSE {
                //See if there is a close one and take it if so, else keep this one
                forcefindNew = QTRUE;
            } else if tooFarOk == QFALSE
            //FIXME: don't need this extra bool any more
            {
                if setEnemy != QFALSE {
                    G_ClearEnemy(*addr_of!(NPC));
                }
            }
        } else if trap::InPVS(
            &(*(*addr_of!(NPC))).r.currentOrigin,
            &(*(*(*addr_of!(NPC))).enemy).r.currentOrigin,
        ) == QFALSE
        {
            //FIXME: should this be a line-of site check?
            //FIXME: a lot of things check PVS AGAIN when deciding whether
            //or not to shoot, redundant!
            //Should we lose the enemy?
            //FIXME: if lose enemy, run lostenemyscript
            if !(*(*(*addr_of!(NPC))).enemy).client.is_null()
                && (*(*(*(*addr_of!(NPC))).enemy).client).hiddenDist != 0.0
            {
                //He ducked into shadow while we weren't looking
                //Drop enemy and see if we should search for him
                NPC_LostEnemyDecideChase();
            } else {
                //If we're not chasing him, we need to lose him
                //NOTE: since we no longer have bStates, really, this logic doesn't work, so never give him up

                /*
                switch( NPCInfo->behaviorState )
                {
                case BS_HUNT_AND_KILL:
                    //Okay to lose PVS, we're chasing them
                    break;
                case BS_RUN_AND_SHOOT:
                //FIXME: only do this if !(NPCInfo->scriptFlags&SCF_CHASE_ENEMY)
                    //If he's not our goalEntity, we're running somewhere else, so lose him
                    if ( NPC->enemy != NPCInfo->goalEntity )
                    {
                        G_ClearEnemy( NPC );
                    }
                    break;
                default:
                    //We're not chasing him, so lose him as an enemy
                    G_ClearEnemy( NPC );
                    break;
                }
                */
            }
        }
    }

    if !(*(*addr_of!(NPC))).enemy.is_null() {
        if (*(*(*addr_of!(NPC))).enemy).health <= 0
            || ((*(*(*addr_of!(NPC))).enemy).flags & FL_NOTARGET) != 0
        {
            if setEnemy != QFALSE {
                G_ClearEnemy(*addr_of!(NPC));
            }
        }
    }

    //FIXME: check your defendEnt, if you have one, see if their enemy is different
    //than yours, or, if they don't have one, pick the closest enemy to THEM?
    let mut closestTo: *mut gentity_t = *addr_of!(NPC);
    if !(*(*addr_of!(NPCInfo))).defendEnt.is_null() {
        //Trying to protect someone
        if (*(*(*addr_of!(NPCInfo))).defendEnt).health > 0 {
            //Still alive, We presume we're close to them, navigation should handle this?
            if !(*(*(*addr_of!(NPCInfo))).defendEnt).enemy.is_null() {
                //They were shot or acquired an enemy
                if (*(*addr_of!(NPC))).enemy != (*(*(*addr_of!(NPCInfo))).defendEnt).enemy {
                    //They have a different enemy, take it!
                    newEnemy = (*(*(*addr_of!(NPCInfo))).defendEnt).enemy;
                    if setEnemy != QFALSE {
                        G_SetEnemy(*addr_of!(NPC), (*(*(*addr_of!(NPCInfo))).defendEnt).enemy);
                    }
                }
            } else if (*(*addr_of!(NPC))).enemy.is_null() {
                //We don't have an enemy, so find closest to defendEnt
                closestTo = (*(*addr_of!(NPCInfo))).defendEnt;
            }
        }
    }

    if (*(*addr_of!(NPC))).enemy.is_null()
        || (!(*(*addr_of!(NPC))).enemy.is_null() && (*(*(*addr_of!(NPC))).enemy).health <= 0)
        || forcefindNew != QFALSE
    {
        //FIXME: NPCs that are moving after an enemy should ignore the can't hit enemy counter- that should only be for NPCs that are standing still
        //NOTE: cantHitEnemyCounter >= 100 means we couldn't hit enemy for a full
        //	10 seconds, so give up.  This means even if we're chasing him, we would
        //	try to find another enemy after 10 seconds (assuming the cantHitEnemyCounter
        //	is allowed to increment in a chasing bState)
        let mut foundenemy: qboolean = QFALSE;

        if findNew == QFALSE {
            if setEnemy != QFALSE {
                (*(*addr_of!(NPC))).lastEnemy = (*(*addr_of!(NPC))).enemy;
                G_ClearEnemy(*addr_of!(NPC));
            }
            return core::ptr::null_mut();
        }

        //If enemy dead or unshootable, look for others on out enemy's team
        if (*(*addr_of!(client))).enemyTeam != NPCTEAM_NEUTRAL {
            //NOTE:  this only checks vis if can't hit enemy for 10 tries, which I suppose
            //			means they need to find one that in more than just PVS
            //newenemy = NPC_PickEnemy( closestTo, NPC->client->enemyTeam, (NPC->cantHitEnemyCounter > 10), qfalse, qtrue );//3rd parm was (NPC->enemyTeam == TEAM_STARFLEET)
            //For now, made it so you ALWAYS have to check VIS
            newEnemy = NPC_PickEnemy(
                closestTo,
                (*(*addr_of!(client))).enemyTeam,
                QTRUE,
                QFALSE,
                QTRUE,
            ); //3rd parm was (NPC->enemyTeam == TEAM_STARFLEET)
            if !newEnemy.is_null() {
                foundenemy = QTRUE;
                if setEnemy != QFALSE {
                    G_SetEnemy(*addr_of!(NPC), newEnemy);
                }
            }
        }

        if forcefindNew == QFALSE {
            if foundenemy == QFALSE {
                if setEnemy != QFALSE {
                    (*(*addr_of!(NPC))).lastEnemy = (*(*addr_of!(NPC))).enemy;
                    G_ClearEnemy(*addr_of!(NPC));
                }
            }

            (*(*addr_of!(NPC))).cantHitEnemyCounter = 0;
        }
        //FIXME: if we can't find any at all, go into INdependant NPC AI, pursue and kill
    }

    if !(*(*addr_of!(NPC))).enemy.is_null() && !(*(*(*addr_of!(NPC))).enemy).client.is_null() {
        if (*(*(*(*addr_of!(NPC))).enemy).client).playerTeam != 0 {
            //			assert( NPC->client->playerTeam != NPC->enemy->client->playerTeam);
            if (*(*addr_of!(client))).playerTeam
                != (*(*(*(*addr_of!(NPC))).enemy).client).playerTeam
            {
                (*(*addr_of!(client))).enemyTeam =
                    (*(*(*(*addr_of!(NPC))).enemy).client).playerTeam;
            }
        }
    }
    newEnemy
}

/// `gentity_t *NPC_SearchForWeapons( void )` (NPC_combat.c:2987).
///
/// Scans every entity for the nearest reachable dropped weapon this NPC may grab
/// (`CheckItemCanBePickedUpByNPC`), preferring one it can nav to or has a clear straight
/// path to. Reads process-global `NPC`; no oracle (entity-state + NAV/trap queries). The
/// `globals.num_entities` loop variant and the `!found->inuse` early form are kept as
/// commented C.
///
/// # Safety
/// Dereferences the live `NPC` global and the `g_entities` array.
pub unsafe fn NPC_SearchForWeapons() -> *mut gentity_t {
    let mut found: *mut gentity_t;
    let mut bestFound: *mut gentity_t = core::ptr::null_mut();
    let mut dist: f32;
    let mut bestDist: f32 = Q3_INFINITE as f32;
    let mut i: c_int = 0;
    //	for ( found = g_entities; found < &g_entities[globals.num_entities] ; found++)
    while i < (*addr_of!(level)).num_entities {
        //		if ( !found->inuse )
        //		{
        //			continue;
        //		}
        if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize)).inuse
            == QFALSE
        {
            i += 1;
            continue;
        }

        found = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(i as isize);

        //FIXME: Also look for ammo_racks that have weapons on them?
        if (*found).s.eType != ET_ITEM {
            i += 1;
            continue;
        }
        if (*(*found).item).giType != IT_WEAPON {
            i += 1;
            continue;
        }
        if ((*found).s.eFlags & EF_NODRAW) != 0 {
            i += 1;
            continue;
        }
        if CheckItemCanBePickedUpByNPC(found, *addr_of!(NPC)) != QFALSE {
            if trap::InPVS(
                &(*found).r.currentOrigin,
                &(*(*addr_of!(NPC))).r.currentOrigin,
            ) != QFALSE
            {
                dist = DistanceSquared(
                    &(*found).r.currentOrigin,
                    &(*(*addr_of!(NPC))).r.currentOrigin,
                );
                if dist < bestDist {
                    if trap::Nav_GetBestPathBetweenEnts(*addr_of!(NPC), found, NF_CLEAR_PATH) == 0
                        || trap::Nav_GetBestNodeAltRoute2(
                            (*(*addr_of!(NPC))).waypoint,
                            (*found).waypoint,
                            NODE_NONE,
                        ) == WAYPOINT_NONE
                    {
                        //can't possibly have a route to any OR can't possibly have a route to this one OR don't have a route to this one
                        if NAV_ClearPathToPoint(
                            *addr_of!(NPC),
                            &(*(*addr_of!(NPC))).r.mins,
                            &(*(*addr_of!(NPC))).r.maxs,
                            &(*found).r.currentOrigin,
                            (*(*addr_of!(NPC))).clipmask,
                            ENTITYNUM_NONE,
                        ) != QFALSE
                        {
                            //have a clear straight path to this one
                            bestDist = dist;
                            bestFound = found;
                        }
                    } else {
                        //can nav to it
                        bestDist = dist;
                        bestFound = found;
                    }
                }
            }
        }
        i += 1;
    }

    bestFound
}

/// `void NPC_CheckGetNewWeapon( void )` (NPC_combat.c:3059).
///
/// When the NPC has lost its weapon mid-combat, clears a stale weapon-pickup goal and —
/// once the `"panic"` timer is done and it has no goal — searches for a dropped weapon and
/// sets a pickup goal toward it. The commented-out nav-route guard around the pickup is kept
/// verbatim. Reads process-global `NPC`/`NPCInfo`; no oracle (entity-state).
///
/// # Safety
/// Dereferences the live `NPC`/`NPCInfo` globals.
pub unsafe fn NPC_CheckGetNewWeapon() {
    if (*(*addr_of!(NPC))).s.weapon == WP_NONE && !(*(*addr_of!(NPC))).enemy.is_null() {
        //if running away because dropped weapon...
        if !(*(*addr_of!(NPCInfo))).goalEntity.is_null()
            && (*(*addr_of!(NPCInfo))).goalEntity == (*(*addr_of!(NPCInfo))).tempGoal
            && !(*(*(*addr_of!(NPCInfo))).goalEntity).enemy.is_null()
            && (*(*(*(*addr_of!(NPCInfo))).goalEntity).enemy).inuse == QFALSE
        {
            //maybe was running at a weapon that was picked up
            (*(*addr_of!(NPCInfo))).goalEntity = core::ptr::null_mut();
        }
        if TIMER_Done(*addr_of!(NPC), c"panic".as_ptr()) != QFALSE
            && (*(*addr_of!(NPCInfo))).goalEntity.is_null()
        {
            //need a weapon, any lying around?
            let foundWeap: *mut gentity_t = NPC_SearchForWeapons();
            if !foundWeap.is_null() {
                //try to nav to it
                /*
                if ( !trap_Nav_GetBestPathBetweenEnts( NPC, foundWeap, NF_CLEAR_PATH )
                    || trap_Nav_GetBestNodeAltRoute( NPC->waypoint, foundWeap->waypoint ) == WAYPOINT_NONE )
                {//can't possibly have a route to any OR can't possibly have a route to this one OR don't have a route to this one
                    if ( !NAV_ClearPathToPoint( NPC, NPC->r.mins, NPC->r.maxs, foundWeap->r.currentOrigin, NPC->clipmask, ENTITYNUM_NONE ) )
                    {//don't even have a clear straight path to this one
                    }
                    else
                    {
                        NPC_SetPickUpGoal( foundWeap );
                    }
                }
                else
                */
                {
                    NPC_SetPickUpGoal(foundWeap);
                }
            }
        }
    }
}
