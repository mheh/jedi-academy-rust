//! Port of `g_weapon.c` — weapon fire logic, hitscan/traces, muzzle and alt-fires.
//! Landed incrementally: only the helpers that already-ported callers reach.
//! `LogAccuracyHit` lands first as a forward dependency of `g_combat.c`'s
//! `G_RadiusDamage` (the splash-damage loop credits an accuracy hit through it).
//! The pure leaf helpers (`WP_SpeedOfMissileForWeapon`, `VectorNPos`,
//! `SnapVectorTowards`) and the small gentity/cvar helpers (`CheatsOn`,
//! `CalcMuzzlePoint`/`CalcMuzzlePointOrigin`, `W_TraceSetStart`) follow. The
//! parameter-driven missile spawners (`WP_FireTurretMissile`,
//! `WP_FireGenericBlasterMissile`, `WP_FireBlasterMissile`, `WP_FireTurboLaserMissile`,
//! `WP_FireEmplacedMissile`) land once `CreateMissile` is in. The `g_weapon.c:12-14`
//! file-statics (`forward`/`vright`/`up`/`muzzle`, `s_quadFactor`) — the per-shot view-aim
//! state `FireWeapon` writes and the player-fire helpers read — are declared incrementally
//! as their first reader lands (see DEVIATIONS). The simple player-fire weapons follow:
//! `WP_FireBryarPistol`, `WP_FireBlaster`, the bowcaster (`WP_FireBowcaster` + main/alt),
//! the repeater (`WP_FireRepeater` + main/alt), and the flechette main-fire
//! (`WP_FlechetteMainFire`; its alt path waits on the laser-trap subsystem). `FireWeapon`
//! itself lands last (its switch arms must exist first). The ghoul2 vehicle-muzzle helper
//! `WP_CalcVehMuzzle` (g_weapon.c:3539) is a self-contained leaf — no oracle (reads `level`
//! + a ghoul2 bolt-matrix trap).

#![allow(non_snake_case)] // C function names kept verbatim
#![allow(non_upper_case_globals)] // C file-static globals (`forward`, `muzzle`, …) kept verbatim

use core::ffi::{c_char, c_int, c_ulong};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::anims::BOTH_MELEE2;
use crate::codemp::game::bg_public::{
    BROKENLIMB_LARM, BROKENLIMB_RARM, EFFECT_EXPLOSION_DEMP2ALT, EFFECT_EXPLOSION_DETPACK,
    EFFECT_EXPLOSION_FLECHETTE,
    EFFECT_EXPLOSION_TRIPMINE, EFFECT_SMOKE, EFFECT_STUNHIT, EF_FIRING, ET_GENERAL, ET_MISSILE,
    ET_NPC, EV_MISSILE_MISS,
    EF_ALT_FIRING, EF_DISINTEGRATION, EF_JETPACK_ACTIVE, EF_RADAROBJECT,
    DEFAULT_MINS_2,
    EV_DISRUPTOR_HIT, EV_DISRUPTOR_MAIN_SHOT, EV_DISRUPTOR_SNIPER_MISS, EV_DISRUPTOR_SNIPER_SHOT,
    EV_SABER_BLOCK,
    GT_SIEGE, HANDEXTEND_KNOCKDOWN, HANDEXTEND_NONE,
    EV_CONC_ALT_IMPACT, MOD_CONC_ALT,
    EV_FIRE_WEAPON, EV_NOAMMO, EV_VEH_FIRE,
    MASK_SHOT, MASK_SOLID, MOD_DET_PACK_SPLASH,
    MOD_DISRUPTOR, MOD_DISRUPTOR_SNIPER,
    MOD_BLASTER, MOD_BOWCASTER, MOD_BRYAR_PISTOL, MOD_BRYAR_PISTOL_ALT, MOD_CONC, MOD_DEMP2, MOD_FLECHETTE,
    MOD_FLECHETTE_ALT_SPLASH,
    MOD_MELEE, MOD_REPEATER, MOD_REPEATER_ALT, MOD_REPEATER_ALT_SPLASH,
    MOD_ROCKET, MOD_ROCKET_HOMING, MOD_ROCKET_HOMING_SPLASH, MOD_ROCKET_SPLASH, MOD_STUN_BATON,
    MOD_THERMAL, MOD_THERMAL_SPLASH,
    MOD_TARGET_LASER, MOD_TRIP_MINE_SPLASH, MOD_UNKNOWN, MOD_VEHICLE, PMF_DUCKED, PW_CLOAKED, PW_QUAD,
    STAT_HEALTH, STAT_WEAPONS, TEAM_SPECTATOR, WEAPON_READY,
    EF_MISSILE_STICK,
};
use crate::codemp::game::bg_public::ET_MOVER;
use crate::codemp::game::bg_vehicles_h::{
    vehWeaponInfo_t, Vehicle_t, MAX_STRAFE_TIME, MAX_VEHICLE_MUZZLES, MAX_VEH_WEAPONS,
    VEH_WEAPON_BASE, VEH_WINGSOPEN, VH_ANIMAL, VH_FIGHTER, VH_FLIER, VH_SPEEDER, VH_WALKER,
};
use crate::codemp::game::bg_vehicleLoad::g_vehWeaponInfo;
use crate::codemp::game::bg_weapons::WP_MuzzlePoint;
use crate::codemp::game::bg_weapons_h::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_OLD, WP_BRYAR_PISTOL, WP_CONCUSSION, WP_DEMP2, WP_DET_PACK,
    WP_DISRUPTOR, WP_EMPLACED_GUN, WP_FLECHETTE, WP_MELEE, WP_NONE, WP_NUM_WEAPONS, WP_REPEATER,
    WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use crate::codemp::game::g_combat::{G_Damage, G_GetHitLocation, G_HeavyMelee, G_RadiusDamage};
use crate::codemp::game::g_log::G_LogWeaponFire;
use crate::codemp::game::g_local::{
    gentity_t, CON_CONNECTED, DAMAGE_DEATH_KNOCKBACK, DAMAGE_EXTRA_KNOCKBACK, DAMAGE_HALF_ABSORB,
    DAMAGE_HEAVY_WEAP_CLASS, DAMAGE_NORMAL, DAMAGE_NO_ARMOR, DAMAGE_NO_HIT_LOC,
    DAMAGE_NO_KNOCKBACK, FL_BBRUSH,
    FL_BOUNCE, FL_BOUNCE_HALF, FL_BOUNCE_SHRAPNEL, FL_NO_KNOCKBACK, FL_SHIELDED, FRAMETIME, HL_NONE,
};
use crate::codemp::game::bg_misc::{
    BG_EmplacedView, BG_EvaluateTrajectory, BG_FindItemForWeapon, BG_GiveMeVectorFromMatrix,
};
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_spawn::{G_SpawnFloat, G_SpawnInt};
use crate::codemp::game::g_main::{
    bg_fighterAltControl, d_projectileGhoul2Collision, g_cheats, g_entities, g_friendlyFire,
    g_g2TraceLod, g_gametype, g_gravity, g_quadfactor, level,
};
use crate::codemp::game::g_public_h::Q3_INFINITE;
use crate::codemp::game::g_missile::{CreateMissile, G_ExplodeMissile, G_MissileImpact};
use crate::codemp::game::g_public_h::{
    BSET_PAIN, G2TRFLAG_DOGHOULTRACE, G2TRFLAG_GETSURFINDEX, G2TRFLAG_HITCORPSES, G2TRFLAG_THICK,
    SVF_BROADCAST, SVF_GLASS_BRUSH, SVF_OWNERNOTSHARED, SVF_USE_CURRENT_ORIGIN,
};
use crate::codemp::game::g_team::OnSameTeam;
use crate::codemp::game::npc_utils::G_ActivateBehavior;
use crate::codemp::game::g_utils::{
    G_AddEvent, G_BoxInBounds, G_Find, G_FreeEntity, G_ModelIndex, G_PlayEffect, G_RadiusList,
    G_ScaleNetHealth, G_SetOrigin, G_Sound, G_SoundIndex, G_Spawn, G_TempEntity, TryHeal,
};
use crate::codemp::game::q_math::{
    vec3_origin, AngleNormalize180, AngleVectors, AnglesToAxis, CrossProduct, DirToByte, Distance, DistanceSquared,
    DotProduct,
    VectorAdd, VectorClear, VectorCopy, VectorLength, VectorMA, VectorNormalize, VectorScale,
    VectorSet, VectorSubtract, vectoangles,
};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::q_shared::{crandom, random};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, snap_vector, trace_t, trajectory_t, vec3_t, vec_t, BUTTON_USE, CHAN_AUTO, CHAN_BODY,
    CHAN_WEAPON, ENTITYNUM_NONE, ENTITYNUM_WORLD, FORCE_LEVEL_3, FP_SABER_DEFENSE, MAX_CLIENTS,
    MAX_GENTITIES, NEGATIVE_Y, NUM_FORCE_POWERS, ORIGIN, PITCH, ROLL, TR_GRAVITY, TR_STATIONARY, YAW,
};
use crate::codemp::game::w_force::Jedi_DodgeEvasion;
use crate::codemp::game::w_saber::WP_SaberCanBlock;
use crate::codemp::game::teams_h::{CLASS_GALAKMECH, CLASS_VEHICLE};
use crate::codemp::game::bg_pmove::BG_KnockDownable;
use crate::codemp::game::q_shared::Q_stricmp;
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_LIGHTSABER, CONTENTS_PLAYERCLIP, CONTENTS_SHOTCLIP, CONTENTS_SOLID,
    SURF_NOIMPACT,
};
use crate::codemp::game::g_public_h::SVF_PLAYER_USABLE;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// Bryar Pistol
const BRYAR_PISTOL_VEL: c_int = 1600;
const BRYAR_PISTOL_DAMAGE: c_int = 10;
// bryar charging gives us one more unit every 200ms--if you change this, you'll
// have to do the same in bg_pmove
const BRYAR_CHARGE_UNIT: f32 = 200.0;
const BRYAR_ALT_SIZE: f32 = 1.0;

// E11 Blaster
const BLASTER_SPREAD: f32 = 1.6; //1.2f
const BLASTER_VELOCITY: c_int = 2300;
const BLASTER_DAMAGE: c_int = 20;

// Bowcaster
const BOWCASTER_DAMAGE: c_int = 50;
const BOWCASTER_VELOCITY: c_int = 1300;
const BOWCASTER_SIZE: c_int = 2;
const BOWCASTER_ALT_SPREAD: f32 = 5.0;
const BOWCASTER_VEL_RANGE: f32 = 0.3;
// bowcaster charging gives us one more unit every 200ms--if you change this, you'll
// have to do the same in bg_pmove
const BOWCASTER_CHARGE_UNIT: f32 = 200.0;

// Repeater
const REPEATER_SPREAD: f32 = 1.4;
const REPEATER_DAMAGE: c_int = 14;
const REPEATER_VELOCITY: c_int = 1600;
const REPEATER_ALT_SIZE: c_int = 3; // half of bbox size
const REPEATER_ALT_DAMAGE: c_int = 60;
const REPEATER_ALT_SPLASH_DAMAGE: c_int = 60;
const REPEATER_ALT_SPLASH_RADIUS: c_int = 128;
const REPEATER_ALT_SPLASH_RAD_SIEGE: c_int = 80;
const REPEATER_ALT_VELOCITY: c_int = 1100;

// Flechette
const FLECHETTE_SHOTS: c_int = 5;
const FLECHETTE_SPREAD: f32 = 4.0;
const FLECHETTE_DAMAGE: c_int = 12; // 15
const FLECHETTE_VEL: c_int = 3500;
const FLECHETTE_SIZE: c_int = 1;
const FLECHETTE_ALT_DAMAGE: c_int = 60;
const FLECHETTE_ALT_SPLASH_DAM: c_int = 60;
const FLECHETTE_ALT_SPLASH_RAD: c_int = 128;

// Disruptor (g_weapon.c:31-38).
const DISRUPTOR_MAIN_DAMAGE: c_int = 30; //40
const DISRUPTOR_MAIN_DAMAGE_SIEGE: c_int = 50;
const DISRUPTOR_ALT_DAMAGE: c_int = 100; //125
const DISRUPTOR_ALT_TRACES: c_int = 3; // can go through a max of 3 damageable(sp?) entities
// distruptor charging gives us one more unit every 50ms--if you change this, you'll have to do the
// same in bg_pmove
const DISRUPTOR_CHARGE_UNIT: f32 = 50.0;

// DEMP2
const DEMP2_DAMAGE: c_int = 35;
const DEMP2_VELOCITY: c_int = 1800;
const DEMP2_SIZE: f32 = 2.0; // half of bbox size
const DEMP2_ALT_DAMAGE: c_int = 8; //12		// does 12, 36, 84 at each of the 3 charge levels. (g_weapon.c:71)
const DEMP2_CHARGE_UNIT: f32 = 700.0; // demp2 charging gives us one more unit every 700ms (g_weapon.c:72)
const DEMP2_ALT_RANGE: f32 = 4096.0; // g_weapon.c:73
const DEMP2_ALT_SPLASHRADIUS: c_int = 256; // g_weapon.c:74

// Stun Baton (g_weapon.c:115). `STUN_BATON_ALT_DAMAGE` (g_weapon.c:118) is unused by the
// fire path — omitted until a reader lands.
const STUN_BATON_DAMAGE: c_int = 20;
const STUN_BATON_RANGE: f32 = 8.0;

// Melee (g_weapon.c:121).
const MELEE_SWING1_DAMAGE: c_int = 10;
const MELEE_SWING2_DAMAGE: c_int = 12;
const MELEE_RANGE: f32 = 8.0;

const ROCKET_VELOCITY: f32 = 900.0; // g_weapon.c:90
const ROCKET_DAMAGE: c_int = 100; // g_weapon.c:91
const ROCKET_SPLASH_DAMAGE: c_int = 100; // g_weapon.c:92
const ROCKET_SPLASH_RADIUS: c_int = 160; // g_weapon.c:93
const ROCKET_SIZE: f32 = 3.0; // g_weapon.c:94
const ROCKET_ALT_THINK_TIME: c_int = 100; // g_weapon.c:95

const CONC_ALT_DAMAGE: c_int = 25; //100 (g_weapon.c:110)
const CONC_VELOCITY: f32 = 3000.0; // g_weapon.c:102
const CONC_DAMAGE: c_int = 75; //150 (g_weapon.c:103)
const CONC_SPLASH_DAMAGE: c_int = 40; //50 (g_weapon.c:107)
const CONC_SPLASH_RADIUS: c_int = 200; //300 (g_weapon.c:108)

// Laser trap / trip mine (g_weapon.c:2196-2204). Declared incrementally as their first reader
// lands. `LT_DELAY_TIME` is the short fuse used once an armed trip-wire's beam is broken.
const LT_DAMAGE: c_int = 100; // g_weapon.c:2197
const LT_SPLASH_RAD: f32 = 256.0; // g_weapon.c:2198
const LT_SPLASH_DAM: c_int = 105; // g_weapon.c:2199
const LT_SIZE: f32 = 1.5; // g_weapon.c:2201
const LT_ALT_TIME: c_int = 2000; // g_weapon.c:2202
const LT_ACTIVATION_DELAY: c_int = 1000; // g_weapon.c:2203
const LT_DELAY_TIME: c_int = 50; // g_weapon.c:2204

// File-static muzzle/aim state (g_weapon.c:12-14). `FireWeapon` writes these each shot
// (eye angles -> `forward`/`vright`/`up` via `AngleVectors`; `CalcMuzzlePoint` -> `muzzle`),
// and the per-weapon fire helpers read them. Declared incrementally as their first reader
// lands. Accessed via `addr_of!`/`addr_of_mut!` (never `&`/`&mut`) per the static-mut
// discipline (see g_main.rs). `forward` is the name the renamed `CalcMuzzlePoint`/
// `CalcMuzzlePointOrigin` param (`fwd`) sidesteps to avoid Rust's E0530.
static mut forward: vec3_t = [0.0; 3];
// `vright` (g_weapon.c:13): the per-shot right axis from `AngleVectors`. First read by the
// stun/melee fire path (offsets the muzzle 4u right). Declared incrementally per the
// file-static pattern; `FireWeapon` (not yet ported) is the writer.
static mut vright: vec3_t = [0.0; 3];
// `up` (g_weapon.c:13): the per-shot up axis from `AngleVectors`. First written by
// `WP_DropThermal` (the thermal-drop helper recomputes the aim axes from the firer's
// viewangles). Declared incrementally per the file-static pattern; `FireWeapon` (not yet ported)
// is the per-shot writer.
static mut up: vec3_t = [0.0; 3];
static mut muzzle: vec3_t = [0.0; 3];
// `s_quadFactor` (g_weapon.c:12): the quad-damage multiplier `FireWeapon` sets from
// `g_quadfactor` (or 1) at the top of each shot. In the JKA MP module it is a vestigial
// dead store — nothing else in g_weapon.c reads it (the per-weapon helpers do not scale by
// it) — but it is written faithfully so the control flow matches C 1:1. Its first (and only)
// writer is `FireWeapon`.
static mut s_quadFactor: f32 = 0.0;

/// `qboolean LogAccuracyHit( gentity_t *target, gentity_t *attacker )` (g_weapon.c:3444).
///
/// Should this hit count toward `attacker`'s accuracy stats? True only when `target`
/// takes damage, is a live client distinct from `attacker`, `attacker` is also a client,
/// and the two are not on the same team. No oracle (gentity-pointer / `level`-team logic).
///
/// # Safety
/// `target` must be a valid entity; `attacker` may be NULL (checked before deref).
pub unsafe fn LogAccuracyHit(target: *mut gentity_t, attacker: *mut gentity_t) -> qboolean {
    if (*target).takedamage == QFALSE {
        return QFALSE;
    }

    if target == attacker {
        return QFALSE;
    }

    if (*target).client.is_null() {
        return QFALSE;
    }

    if attacker.is_null() {
        return QFALSE;
    }

    if (*attacker).client.is_null() {
        return QFALSE;
    }

    if (*(*target).client).ps.stats[STAT_HEALTH as usize] <= 0 {
        return QFALSE;
    }

    if OnSameTeam(target, attacker) != QFALSE {
        return QFALSE;
    }

    QTRUE
}

/// `void touch_NULL( gentity_t *ent, gentity_t *other, trace_t *trace )` (g_weapon.c:164).
///
/// The empty touch callback. Used as a missile/projectile `touch` slot when a weapon needs
/// an entity that registers no touch behaviour (thermal detonators, alt-flechette bouncy
/// things). `extern "C"` so it is ABI-compatible with the `gentity_t::touch` function-pointer
/// slot. No oracle (empty body).
///
/// # Safety
/// All three pointers are unused; callable through the C `touch` slot.
pub unsafe extern "C" fn touch_NULL(
    _ent: *mut gentity_t,
    _other: *mut gentity_t,
    _trace: *mut trace_t,
) {
}

/// `void RocketDie(gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage,
/// int mod)` (g_weapon.c:1776) — the `die` callback installed on a fired rocket missile. When the
/// rocket takes lethal damage it clears its own `die` slot and `r.contents` (so it can't be hit
/// again), explodes via [`G_ExplodeMissile`], then schedules itself to free this frame
/// (`think = G_FreeEntity`, `nextthink = level.time`). `inflictor`/`attacker`/`damage`/`mod` are
/// unused. A `pub unsafe extern "C"` fn for the `gentity_t::die` fn-pointer ABI; `self->die = 0`
/// → `None`. No oracle (entity-state mutation).
///
/// # Safety
/// `self_` must point to a valid missile `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn RocketDie(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    _mod: c_int,
) {
    (*self_).die = None;
    (*self_).r.contents = 0;

    G_ExplodeMissile(self_);

    (*self_).think = Some(G_FreeEntity);
    (*self_).nextthink = (*addr_of!(level)).time;
}

/// `void rocketThink( gentity_t *ent )` (g_weapon.c:1645) — the `think` callback for a homing
/// (alt-fire) rocket. Each frame it: (1) self-removes when its `genericValue1` lifetime expires
/// — exploding via [`RocketDie`] if `genericValue2`, else freeing; (2) gives up tracking (and
/// arms a 10s free) if its enemy is gone/dead/cloaked; (3) otherwise steers `movedir` toward the
/// enemy's centre, capping the turn (a `dot < 0` heading does a damped ~45° `CrossProduct`-based
/// turn while halving `vel`, milder turns above that), adds decaying `random` jitter via
/// `crandom`, dives toward grounded clients within 128u, then rewrites the
/// `s.pos.trDelta`/`trBase`/`trTime` trajectory (`SnapVector`ing the delta for net bandwidth).
/// Vehicle rockets (`spawnflags&1`) use `ent->speed` and chase faster vehicle enemies. A `pub
/// unsafe extern "C"` fn for the `gentity_t::think` fn-pointer ABI. No oracle (entity-state
/// mutation + reads opaque `enemy`/`client`).
///
/// # Safety
/// `ent` must point to a valid rocket `gentity_t`; `level`/`g_entities` must be initialised.
pub unsafe extern "C" fn rocketThink(ent: *mut gentity_t) {
    let mut newdir: vec3_t = [0.0; 3];
    let mut targetdir: vec3_t = [0.0; 3];
    // C local `up={0,0,1}` (g_weapon.c rocketThink); renamed `upDir` here to dodge E0530
    // (cannot shadow the file-static `up` introduced for `WP_DropThermal`; mirrors the
    // `CalcMuzzlePoint` `forward`->`fwd` precedent).
    let upDir: vec3_t = [0.0, 0.0, 1.0];
    let mut right: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    let dot: f32;
    let dot2: f32;
    let dis: f32;
    let mut vel: f32 = if (*ent).spawnflags & 1 != 0 {
        (*ent).speed
    } else {
        ROCKET_VELOCITY
    };

    if (*ent).genericValue1 != 0 && (*ent).genericValue1 < (*addr_of!(level)).time {
        // time's up, we're done, remove us
        if (*ent).genericValue2 != 0 {
            // explode when die
            RocketDie(
                ent,
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize),
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize),
                0,
                MOD_UNKNOWN,
            );
        } else {
            // just remove when die
            G_FreeEntity(ent);
        }
        return;
    }
    if (*ent).enemy.is_null()
        || (*(*ent).enemy).client.is_null()
        || (*(*ent).enemy).health <= 0
        || (*(*(*ent).enemy).client).ps.powerups[PW_CLOAKED as usize] != 0
    {
        // no enemy or enemy not a client or enemy dead or enemy cloaked
        if (*ent).genericValue1 == 0 {
            // doesn't have its own self-kill time
            (*ent).nextthink = (*addr_of!(level)).time + 10000;
            (*ent).think = Some(G_FreeEntity);
        }
        return;
    }

    if (*ent).spawnflags & 1 != 0 {
        // vehicle rocket
        if !(*(*ent).enemy).client.is_null()
            && (*(*(*ent).enemy).client).NPC_class == CLASS_VEHICLE
        {
            // tracking another vehicle
            if (*(*(*ent).enemy).client).ps.speed + 4000.0 > vel {
                vel = (*(*(*ent).enemy).client).ps.speed + 4000.0;
            }
        }
    }

    if !(*ent).enemy.is_null() && (*(*ent).enemy).inuse != QFALSE {
        let new_dir_mult: f32 = if (*ent).angle != 0.0 {
            (*ent).angle * 2.0
        } else {
            1.0
        };
        let old_dir_mult: f32 = if (*ent).angle != 0.0 {
            (1.0 - (*ent).angle) * 2.0
        } else {
            1.0
        };

        VectorCopy(&(*(*ent).enemy).r.currentOrigin, &mut org);
        org[2] += ((*(*ent).enemy).r.mins[2] + (*(*ent).enemy).r.maxs[2]) * 0.5;

        VectorSubtract(&org, &(*ent).r.currentOrigin, &mut targetdir);
        VectorNormalize(&mut targetdir);

        // Now the rocket can't do a 180 in space, so we'll limit the turn to about 45 degrees.
        dot = DotProduct(&targetdir, &(*ent).movedir);
        if (*ent).spawnflags & 1 != 0 {
            // vehicle rocket
            if (*ent).radius > -1.0 {
                // can lose the lock if DotProduct drops below this number
                if dot < (*ent).radius {
                    // lost the lock!!!
                    // HMM... maybe can re-lock on if they come in front again?
                    /*
                    //OR: should it stop trying to lock altogether?
                    if ( ent->genericValue1 )
                    {//have a timelimit, set next think to that
                        ent->nextthink = ent->genericValue1;
                        if ( ent->genericValue2 )
                        {//explode when die
                            ent->think = G_ExplodeMissile;
                        }
                        else
                        {
                            ent->think = G_FreeEntity;
                        }
                    }
                    else
                    {
                        ent->think = NULL;
                        ent->nextthink = -1;
                    }
                    */
                    return;
                }
            }
        }

        // a dot of 1.0 means right-on-target.
        if dot < 0.0 {
            // Go in the direction opposite, start a 180.
            CrossProduct(&(*ent).movedir, &upDir, &mut right);
            dot2 = DotProduct(&targetdir, &right);

            if dot2 > 0.0 {
                // Turn 45 degrees right.
                VectorMA(&(*ent).movedir, 0.4 * new_dir_mult, &right, &mut newdir);
            } else {
                // Turn 45 degrees left.
                VectorMA(&(*ent).movedir, -0.4 * new_dir_mult, &right, &mut newdir);
            }

            // Yeah we've adjusted horizontally, but let's split the difference vertically, so we kinda try to move towards it.
            // NOTE: the C scales by `0.5` (a *double* literal, not `0.5f`) — the float sum is
            // promoted to double, multiplied, then truncated back to float on the store.
            newdir[2] = (((targetdir[2] * new_dir_mult) + ((*ent).movedir[2] * old_dir_mult))
                as f64
                * 0.5) as f32;

            // let's also slow down a lot
            vel *= 0.5;
        } else if dot < 0.70 {
            // Still a bit off, so we turn a bit softer
            VectorMA(&(*ent).movedir, 0.5 * new_dir_mult, &targetdir, &mut newdir);
        } else {
            // getting close, so turn a bit harder
            VectorMA(&(*ent).movedir, 0.9 * new_dir_mult, &targetdir, &mut newdir);
        }

        // add crazy drunkenness
        for i in 0..3 {
            // C `crandom()` is the macro `(2.0 * (random() - 0.5))` — a *double* expression, so
            // the whole product `crandom() * ent->random * 0.25f` evaluates in `double` and only
            // truncates to `float` on the `+=` store. Mirror that: compute in f64, cast on assign.
            newdir[i] =
                (newdir[i] as f64 + crandom() * (*ent).random as f64 * 0.25) as f32;
        }

        // decay the randomness
        (*ent).random *= 0.9;

        if !(*(*ent).enemy).client.is_null()
            && (*(*(*ent).enemy).client).ps.groundEntityNum != ENTITYNUM_NONE
        {
            // tracking a client who's on the ground, aim at the floor...?
            // Try to crash into the ground if we get close enough to do splash damage
            dis = Distance(&(*ent).r.currentOrigin, &org);

            if dis < 128.0 {
                // the closer we get, the more we push the rocket down, heh heh.
                newdir[2] -= (1.0 - (dis / 128.0)) * 0.6;
            }
        }

        VectorNormalize(&mut newdir);

        VectorScale(&newdir, vel * 0.5, &mut (*ent).s.pos.trDelta);
        VectorCopy(&newdir, &mut (*ent).movedir);
        trap::SnapVector(&mut (*ent).s.pos.trDelta); // save net bandwidth
        VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).s.pos.trBase);
        (*ent).s.pos.trTime = (*addr_of!(level)).time;
    }

    (*ent).nextthink = (*addr_of!(level)).time + ROCKET_ALT_THINK_TIME; // Nothing at all spectacular happened, continue.
}

/// `static void WP_FireRocket( gentity_t *ent, qboolean altFire )` (g_weapon.c:1788).
///
/// Spawn a rocket from the file-static `muzzle` along `forward`; on alt-fire the velocity is
/// halved. If the firer has a current rocket lock and it has matured (≥10 lock ticks), the
/// missile is given a homing target (an enemy client still alive and not on the same team)
/// via the [`rocketThink`] tracker. The missile is set up with a `ROCKET_SIZE` cube bbox,
/// rocket means-of-death (homing variants on alt), `MASK_SHOT` clip, and made shootable
/// out of the air (10 HP, [`RocketDie`] on death). No oracle (file-static aim + gentity
/// spawn + `level`/`g_entities`/`g_gametype` globals).
///
/// # Safety
/// `ent` must be valid; reads the file-static `muzzle`/`forward`; `level`/`g_entities` must
/// be initialised.
pub unsafe fn WP_FireRocket(ent: *mut gentity_t, altFire: qboolean) {
    let damage = ROCKET_DAMAGE;
    let mut vel = ROCKET_VELOCITY;
    // C declares `int dif = 0;` and `float rTime;` at the top, but both are only ever
    // touched inside the rocket-lock block below; scoping them there keeps the `= 0` init
    // from being flagged as a never-read store.
    let rTime: f32;

    if altFire != QFALSE {
        vel *= 0.5;
    }

    let missile = CreateMissile(
        &mut *addr_of_mut!(muzzle),
        &*addr_of!(forward),
        vel,
        10000,
        ent,
        altFire,
    );

    if !(*ent).client.is_null()
        && (*(*ent).client).ps.rocketLockIndex != ENTITYNUM_NONE
    {
        let lockTimeInterval: f32 = (if (*addr_of!(g_gametype)).integer == GT_SIEGE {
            2400.0
        } else {
            1200.0
        }) / 16.0;
        rTime = if (*(*ent).client).ps.rocketLockTime == -1.0 {
            (*(*ent).client).ps.rocketLastValidTime
        } else {
            (*(*ent).client).ps.rocketLockTime
        };

        let mut dif = (((*addr_of!(level)).time as f32 - rTime) / lockTimeInterval) as c_int;

        if dif < 0 {
            dif = 0;
        }

        //It's 10 even though it locks client-side at 8, because we want them to have a sturdy lock first, and because there's a slight difference in time between server and client
        if dif >= 10 && rTime != -1.0 {
            (*missile).enemy =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.rocketLockIndex as usize);

            if !(*missile).enemy.is_null()
                && !(*(*missile).enemy).client.is_null()
                && (*(*missile).enemy).health > 0
                && OnSameTeam(ent, (*missile).enemy) == QFALSE
            {
                //if enemy became invalid, died, or is on the same team, then don't seek it
                (*missile).angle = 0.5;
                (*missile).think = Some(rocketThink);
                (*missile).nextthink = (*addr_of!(level)).time + ROCKET_ALT_THINK_TIME;
            }
        }

        (*(*ent).client).ps.rocketLockIndex = ENTITYNUM_NONE;
        (*(*ent).client).ps.rocketLockTime = 0.0;
        (*(*ent).client).ps.rocketTargetTime = 0.0;
    }

    (*missile).classname = c"rocket_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_ROCKET_LAUNCHER;

    // Make it easier to hit things
    VectorSet(&mut (*missile).r.maxs, ROCKET_SIZE, ROCKET_SIZE, ROCKET_SIZE);
    let maxs = (*missile).r.maxs;
    VectorScale(&maxs, -1.0, &mut (*missile).r.mins);

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    if altFire != QFALSE {
        (*missile).methodOfDeath = MOD_ROCKET_HOMING;
        (*missile).splashMethodOfDeath = MOD_ROCKET_HOMING_SPLASH;
    } else {
        (*missile).methodOfDeath = MOD_ROCKET;
        (*missile).splashMethodOfDeath = MOD_ROCKET_SPLASH;
    }
    //===testing being able to shoot rockets out of the air==================================
    (*missile).health = 10;
    (*missile).takedamage = QTRUE;
    (*missile).r.contents = MASK_SHOT;
    (*missile).die = Some(RocketDie);
    //===testing being able to shoot rockets out of the air==================================

    (*missile).clipmask = MASK_SHOT;
    (*missile).splashDamage = ROCKET_SPLASH_DAMAGE;
    (*missile).splashRadius = ROCKET_SPLASH_RADIUS;

    // we don't want it to ever bounce
    (*missile).bounceCount = 0;
}

/// `float WP_SpeedOfMissileForWeapon( int wp, qboolean alt_fire )` (g_weapon.c:174).
///
/// Always 500 in shipped JKA — the per-weapon/alt-fire dispatch this once implied was
/// flattened to a constant. Both parameters are retained for source/ABI fidelity.
pub fn WP_SpeedOfMissileForWeapon(_wp: c_int, _alt_fire: qboolean) -> f32 {
    500.0
}

/// `void VectorNPos( vec3_t in, vec3_t out )` (g_weapon.c:2598).
///
/// Componentwise absolute value: `out[i] = |in[i]|`.
pub fn VectorNPos(in_: &vec3_t, out: &mut vec3_t) {
    out[0] = if in_[0] < 0.0 { -in_[0] } else { in_[0] };
    out[1] = if in_[1] < 0.0 { -in_[1] } else { in_[1] };
    out[2] = if in_[2] < 0.0 { -in_[2] } else { in_[2] };
}

/// `void SnapVectorTowards( vec3_t v, vec3_t to )` (g_weapon.c:3423).
///
/// Snap each component of `v` to an integer, biased toward `to`: C `(int)` truncation
/// when `to[i] <= v[i]`, else truncation + 1. Used so a traced impact point rounds
/// *away* from the shooter rather than back into the surface it struck.
pub fn SnapVectorTowards(v: &mut vec3_t, to: &vec3_t) {
    for i in 0..3 {
        if to[i] <= v[i] {
            v[i] = v[i] as c_int as f32;
        } else {
            v[i] = v[i] as c_int as f32 + 1.0;
        }
    }
}

/// `qboolean CheatsOn( void )` (g_weapon.c:2833).
///
/// True iff the `sv_cheats` cvar (`g_cheats`) is non-zero. No oracle — reads a global cvar.
///
/// # Safety
/// Reads the `g_cheats` global; valid once `G_InitGame` has registered the cvars.
pub unsafe fn CheatsOn() -> qboolean {
    if (*addr_of!(g_cheats)).integer == 0 {
        return QFALSE;
    }
    QTRUE
}

// ======================================================================
// DISRUPTOR
// ======================================================================

/// `static void WP_DisruptorMainFire( gentity_t *ent )` (g_weapon.c:481).
///
/// The disruptor primary (non-zoomed) hitscan. Fires a single 8192u beam from the firer's
/// eye along `forward`, looping up to 10 traces so a Jedi who dodges (or a dueller we should
/// ignore) is skipped past. A level-3+ saber-defense client can block the bolt (broadcasts
/// the shot + an `EV_SABER_BLOCK` and returns); an `FL_SHIELDED` target stops it cold.
/// Otherwise it always renders the shot beam (`EV_DISRUPTOR_MAIN_SHOT`) and, if the impact is
/// on a damageable entity, applies [`G_Damage`] (`MOD_DISRUPTOR`) + an `EV_DISRUPTOR_HIT`
/// (else an `EV_DISRUPTOR_SNIPER_MISS` mark). `static` in C — ported `pub` though the C is
/// `static` (the g_cmds.rs `G_SayTo` leaf precedent) to avoid a `dead_code` warning until its
/// caller chain `WP_FireDisruptor`→`FireWeapon` (not yet ported) lands. No oracle (gentity-state +
/// `trap::Trace`/`trap::G2Trace` + `G_Damage`/`G_TempEntity` side-effects).
///
/// # Safety
/// `ent` must be a valid client `gentity_t`; `g_entities`/`level` and the disruptor cvars
/// must be initialised.
pub unsafe fn WP_DisruptorMainFire(ent: *mut gentity_t) {
    let mut damage: c_int = DISRUPTOR_MAIN_DAMAGE;
    let mut render_impact: qboolean = QTRUE;
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut tr: trace_t;
    let mut traceEnt: *mut gentity_t;
    let mut tent: *mut gentity_t;
    let shotRange: f32 = 8192.0;
    let mut ignore: c_int;
    let mut traces: c_int;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        damage = DISRUPTOR_MAIN_DAMAGE_SIEGE;
    }

    tr = trace_t::default(); //to shut the compiler up

    VectorCopy(&(*(*ent).client).ps.origin, &mut start);
    start[2] += (*(*ent).client).ps.viewheight as f32; //By eyes

    let startCopy = start;
    VectorMA(&startCopy, shotRange, &*addr_of!(forward), &mut end);

    ignore = (*ent).s.number;
    traces = 0;
    while traces < 10 {
        //need to loop this in case we hit a Jedi who dodges the shot
        if (*addr_of!(d_projectileGhoul2Collision)).integer != 0 {
            tr = trap::G2Trace(
                &start,
                &vec3_origin,
                &vec3_origin,
                &end,
                ignore,
                MASK_SHOT,
                G2TRFLAG_DOGHOULTRACE | G2TRFLAG_GETSURFINDEX | G2TRFLAG_THICK | G2TRFLAG_HITCORPSES,
                (*addr_of!(g_g2TraceLod)).integer,
            );
        } else {
            tr = trap::Trace(&start, &vec3_origin, &vec3_origin, &end, ignore, MASK_SHOT);
        }

        traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

        if (*addr_of!(d_projectileGhoul2Collision)).integer != 0
            && (*traceEnt).inuse != QFALSE
            && !(*traceEnt).client.is_null()
        {
            //g2 collision checks -rww
            if (*traceEnt).inuse != QFALSE
                && !(*traceEnt).client.is_null()
                && !(*traceEnt).ghoul2.is_null()
            {
                //since we used G2TRFLAG_GETSURFINDEX, tr.surfaceFlags will actually contain the index of the surface on the ghoul2 model we collided with.
                (*(*traceEnt).client).g2LastSurfaceHit = tr.surfaceFlags;
                (*(*traceEnt).client).g2LastSurfaceTime = (*addr_of!(level)).time;
            }

            if !(*traceEnt).ghoul2.is_null() {
                tr.surfaceFlags = 0; //clear the surface flags after, since we actually care about them in here.
            }
        }

        if !traceEnt.is_null()
            && !(*traceEnt).client.is_null()
            && (*(*traceEnt).client).ps.duelInProgress != QFALSE
            && (*(*traceEnt).client).ps.duelIndex != (*ent).s.number
        {
            VectorCopy(&tr.endpos, &mut start);
            ignore = tr.entityNum as c_int;
            traces += 1;
            continue;
        }

        if Jedi_DodgeEvasion(
            traceEnt,
            ent,
            &mut tr,
            G_GetHitLocation(traceEnt, &tr.endpos),
        ) != QFALSE
        {
            //act like we didn't even hit him
            VectorCopy(&tr.endpos, &mut start);
            ignore = tr.entityNum as c_int;
            traces += 1;
            continue;
        } else if !traceEnt.is_null()
            && !(*traceEnt).client.is_null()
            && (*(*traceEnt).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize]
                >= FORCE_LEVEL_3
        {
            if WP_SaberCanBlock(traceEnt, &tr.endpos, 0, MOD_DISRUPTOR, QTRUE, 0) != 0 {
                //broadcast and stop the shot because it was blocked
                let te: *mut gentity_t;

                tent = G_TempEntity(&tr.endpos, EV_DISRUPTOR_MAIN_SHOT);
                VectorCopy(&*addr_of!(muzzle), &mut (*tent).s.origin2);
                (*tent).s.eventParm = (*ent).s.number;

                te = G_TempEntity(&tr.endpos, EV_SABER_BLOCK);
                VectorCopy(&tr.endpos, &mut (*te).s.origin);
                VectorCopy(&tr.plane.normal, &mut (*te).s.angles);
                if (*te).s.angles[0] == 0.0 && (*te).s.angles[1] == 0.0 && (*te).s.angles[2] == 0.0 {
                    (*te).s.angles[1] = 1.0;
                }
                (*te).s.eventParm = 0;
                (*te).s.weapon = 0; //saberNum
                (*te).s.legsAnim = 0; //bladeNum

                return;
            }
        } else if ((*traceEnt).flags & FL_SHIELDED) != 0 {
            //stopped cold
            return;
        }
        //a Jedi is not dodging this shot
        break;
    }

    if (tr.surfaceFlags & SURF_NOIMPACT) != 0 {
        render_impact = QFALSE;
    }

    // always render a shot beam, doing this the old way because I don't much feel like overriding the effect.
    tent = G_TempEntity(&tr.endpos, EV_DISRUPTOR_MAIN_SHOT);
    VectorCopy(&*addr_of!(muzzle), &mut (*tent).s.origin2);
    (*tent).s.eventParm = (*ent).s.number;

    traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

    if render_impact != QFALSE {
        if (tr.entityNum as c_int) < ENTITYNUM_WORLD && (*traceEnt).takedamage != QFALSE {
            if !(*traceEnt).client.is_null() && LogAccuracyHit(traceEnt, ent) != QFALSE {
                (*(*ent).client).accuracy_hits += 1;
            }

            G_Damage(
                traceEnt,
                ent,
                ent,
                addr_of_mut!(forward),
                &mut { tr.endpos } as *mut vec3_t,
                damage,
                DAMAGE_NORMAL,
                MOD_DISRUPTOR,
            );

            tent = G_TempEntity(&tr.endpos, EV_DISRUPTOR_HIT);
            (*tent).s.eventParm = DirToByte(&tr.plane.normal);
            if !(*traceEnt).client.is_null() {
                (*tent).s.weapon = 1;
            }
        } else {
            // Hmmm, maybe don't make any marks on things that could break
            tent = G_TempEntity(&tr.endpos, EV_DISRUPTOR_SNIPER_MISS);
            (*tent).s.eventParm = DirToByte(&tr.plane.normal);
            (*tent).s.weapon = 1;
        }
    }
}

/// `qboolean G_CanDisruptify( gentity_t *ent )` (g_weapon.c:620).
///
/// Whether `ent` may be disintegrated by a fully-charged disruptor sniper shot. Any
/// non-vehicle entity (not an `ET_NPC`/`CLASS_VEHICLE` with a `m_pVehicle`) can; among
/// vehicles only the `VH_ANIMAL` type (e.g. a tauntaun) can. No oracle — gentity-pointer
/// predicate.
///
/// # Safety
/// `ent` may be null; if non-null, its `client`, `m_pVehicle`, and (for a vehicle) that
/// vehicle's `m_pVehicleInfo` must be valid pointers.
pub unsafe fn G_CanDisruptify(ent: *mut gentity_t) -> qboolean {
    if ent.is_null()
        || (*ent).inuse == QFALSE
        || (*ent).client.is_null()
        || (*ent).s.eType != ET_NPC as c_int
        || (*ent).s.NPC_class != CLASS_VEHICLE
        || (*ent).m_pVehicle.is_null()
    {
        //not vehicle
        return QTRUE;
    }

    if (*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_ANIMAL {
        //animal is only type that can be disintigeiteigerated
        return QTRUE;
    }

    //don't do it to any other veh
    QFALSE
}

/// `void WP_DisruptorAltFire( gentity_t *ent )` (g_weapon.c:638).
///
/// The disruptor zoomed alt-fire — a charge-scaled sniper beam. Charge (`weaponChargeTime`)
/// sets both the damage bonus and how many entities the beam can punch through (1–3 traces);
/// a maxed charge marks `fullCharge`. Per trace it renders `EV_DISRUPTOR_SNIPER_SHOT`, lets a
/// dodging Jedi / level-3+ saber-defense block (returning), and on a damageable hit applies
/// [`G_Damage`] (`MOD_DISRUPTOR_SNIPER`, no-knockback); a full-charge kill of a client that
/// [`G_CanDisruptify`] passes is disintegrated (`EF_DISINTEGRATION`). The beam stops on a
/// `SURF_NOIMPACT` skybox, an `FL_SHIELDED` target, or a non-damageable non-glass/mover wall.
/// `void` (player-callable) in C — kept `pub`. No oracle (gentity-state +
/// `trap::Trace`/`trap::G2Trace` + `G_Damage`/`G_TempEntity` side-effects).
///
/// # Safety
/// `ent` must be a valid `gentity_t`; `ent->client` is checked before deref. `g_entities`/
/// `level` and the disruptor cvars must be initialised.
pub unsafe fn WP_DisruptorAltFire(ent: *mut gentity_t) {
    let mut damage: c_int;
    let skip: c_int;
    let mut render_impact: qboolean = QTRUE;
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut muzzle2: vec3_t = [0.0; 3];
    let mut tr: trace_t;
    let mut traceEnt: *mut gentity_t;
    let mut tent: *mut gentity_t;
    let shotRange: f32 = 8192.0;
    let mut i: c_int;
    let mut count: c_int;
    let mut maxCount: c_int = 60;
    let mut traces: c_int = DISRUPTOR_ALT_TRACES;
    let mut fullCharge: qboolean = QFALSE;

    damage = DISRUPTOR_ALT_DAMAGE - 30;

    VectorCopy(&*addr_of!(muzzle), &mut muzzle2); // making a backup copy

    if !(*ent).client.is_null() {
        VectorCopy(&(*(*ent).client).ps.origin, &mut start);
        start[2] += (*(*ent).client).ps.viewheight as f32; //By eyes

        count = ((*addr_of!(level)).time - (*(*ent).client).ps.weaponChargeTime)
            / DISRUPTOR_CHARGE_UNIT as c_int;
        if (*addr_of!(g_gametype)).integer == GT_SIEGE {
            //maybe a full alt-charge should be a *bit* more dangerous in Siege mode?
            //maxCount = ceil((200.0f-(float)damage)/2.0f);//cap at 200 damage total
            maxCount = 200; //the previous line ALWAYS evaluated to 135 - was that on purpose?
        }
    } else {
        VectorCopy(&(*ent).r.currentOrigin, &mut start);
        start[2] += 24.0;

        count = 100 / DISRUPTOR_CHARGE_UNIT as c_int;
    }

    count *= 2;

    if count < 1 {
        count = 1;
    } else if count >= maxCount {
        count = maxCount;
        fullCharge = QTRUE;
    }

    // more powerful charges go through more things
    if count < 10 {
        traces = 1;
    } else if count < 20 {
        traces = 2;
    }

    damage += count;

    skip = (*ent).s.number;
    let mut skip = skip;

    i = 0;
    while i < traces {
        let startCopy = start;
        VectorMA(&startCopy, shotRange, &*addr_of!(forward), &mut end);

        if (*addr_of!(d_projectileGhoul2Collision)).integer != 0 {
            tr = trap::G2Trace(
                &start,
                &vec3_origin,
                &vec3_origin,
                &end,
                skip,
                MASK_SHOT,
                G2TRFLAG_DOGHOULTRACE | G2TRFLAG_GETSURFINDEX | G2TRFLAG_THICK | G2TRFLAG_HITCORPSES,
                (*addr_of!(g_g2TraceLod)).integer,
            );
        } else {
            tr = trap::Trace(&start, &vec3_origin, &vec3_origin, &end, skip, MASK_SHOT);
        }

        traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

        if (*addr_of!(d_projectileGhoul2Collision)).integer != 0
            && (*traceEnt).inuse != QFALSE
            && !(*traceEnt).client.is_null()
        {
            //g2 collision checks -rww
            if (*traceEnt).inuse != QFALSE
                && !(*traceEnt).client.is_null()
                && !(*traceEnt).ghoul2.is_null()
            {
                //since we used G2TRFLAG_GETSURFINDEX, tr.surfaceFlags will actually contain the index of the surface on the ghoul2 model we collided with.
                (*(*traceEnt).client).g2LastSurfaceHit = tr.surfaceFlags;
                (*(*traceEnt).client).g2LastSurfaceTime = (*addr_of!(level)).time;
            }

            if !(*traceEnt).ghoul2.is_null() {
                tr.surfaceFlags = 0; //clear the surface flags after, since we actually care about them in here.
            }
        }

        if (tr.surfaceFlags & SURF_NOIMPACT) != 0 {
            render_impact = QFALSE;
        }

        if !traceEnt.is_null()
            && !(*traceEnt).client.is_null()
            && (*(*traceEnt).client).ps.duelInProgress != QFALSE
            && (*(*traceEnt).client).ps.duelIndex != (*ent).s.number
        {
            skip = tr.entityNum as c_int;
            VectorCopy(&tr.endpos, &mut start);
            i += 1;
            continue;
        }

        if Jedi_DodgeEvasion(
            traceEnt,
            ent,
            &mut tr,
            G_GetHitLocation(traceEnt, &tr.endpos),
        ) != QFALSE
        {
            skip = tr.entityNum as c_int;
            VectorCopy(&tr.endpos, &mut start);
            i += 1;
            continue;
        } else if !traceEnt.is_null()
            && !(*traceEnt).client.is_null()
            && (*(*traceEnt).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize]
                >= FORCE_LEVEL_3
        {
            if WP_SaberCanBlock(traceEnt, &tr.endpos, 0, MOD_DISRUPTOR_SNIPER, QTRUE, 0) != 0 {
                //broadcast and stop the shot because it was blocked
                let te: *mut gentity_t;

                tent = G_TempEntity(&tr.endpos, EV_DISRUPTOR_SNIPER_SHOT);
                VectorCopy(&*addr_of!(muzzle), &mut (*tent).s.origin2);
                (*tent).s.shouldtarget = fullCharge;
                (*tent).s.eventParm = (*ent).s.number;

                te = G_TempEntity(&tr.endpos, EV_SABER_BLOCK);
                VectorCopy(&tr.endpos, &mut (*te).s.origin);
                VectorCopy(&tr.plane.normal, &mut (*te).s.angles);
                if (*te).s.angles[0] == 0.0 && (*te).s.angles[1] == 0.0 && (*te).s.angles[2] == 0.0 {
                    (*te).s.angles[1] = 1.0;
                }
                (*te).s.eventParm = 0;
                (*te).s.weapon = 0; //saberNum
                (*te).s.legsAnim = 0; //bladeNum

                return;
            }
        }

        // always render a shot beam, doing this the old way because I don't much feel like overriding the effect.
        tent = G_TempEntity(&tr.endpos, EV_DISRUPTOR_SNIPER_SHOT);
        VectorCopy(&*addr_of!(muzzle), &mut (*tent).s.origin2);
        (*tent).s.shouldtarget = fullCharge;
        (*tent).s.eventParm = (*ent).s.number;

        // If the beam hits a skybox, etc. it would look foolish to add impact effects
        if render_impact != QFALSE {
            if (*traceEnt).takedamage != QFALSE && !(*traceEnt).client.is_null() {
                (*tent).s.otherEntityNum = (*traceEnt).s.number;

                // Create a simple impact type mark
                tent = G_TempEntity(&tr.endpos, EV_MISSILE_MISS);
                (*tent).s.eventParm = DirToByte(&tr.plane.normal);
                (*tent).s.eFlags |= EF_ALT_FIRING;

                if LogAccuracyHit(traceEnt, ent) != QFALSE {
                    if !(*ent).client.is_null() {
                        (*(*ent).client).accuracy_hits += 1;
                    }
                }
            } else {
                if ((*traceEnt).r.svFlags & SVF_GLASS_BRUSH) != 0
                    || (*traceEnt).takedamage != QFALSE
                    || (*traceEnt).s.eType == ET_MOVER as c_int
                {
                    if (*traceEnt).takedamage != QFALSE {
                        G_Damage(
                            traceEnt,
                            ent,
                            ent,
                            addr_of_mut!(forward),
                            &mut { tr.endpos } as *mut vec3_t,
                            damage,
                            DAMAGE_NO_KNOCKBACK,
                            MOD_DISRUPTOR_SNIPER,
                        );

                        tent = G_TempEntity(&tr.endpos, EV_DISRUPTOR_HIT);
                        (*tent).s.eventParm = DirToByte(&tr.plane.normal);
                    }
                } else {
                    // Hmmm, maybe don't make any marks on things that could break
                    tent = G_TempEntity(&tr.endpos, EV_DISRUPTOR_SNIPER_MISS);
                    (*tent).s.eventParm = DirToByte(&tr.plane.normal);
                }
                break; // and don't try any more traces
            }

            if ((*traceEnt).flags & FL_SHIELDED) != 0 {
                //stops us cold
                break;
            }

            if (*traceEnt).takedamage != QFALSE {
                let mut preAng: vec3_t = [0.0; 3];
                let preHealth: c_int = (*traceEnt).health;
                let mut preLegs: c_int = 0;
                let mut preTorso: c_int = 0;

                if !(*traceEnt).client.is_null() {
                    preLegs = (*(*traceEnt).client).ps.legsAnim;
                    preTorso = (*(*traceEnt).client).ps.torsoAnim;
                    VectorCopy(&(*(*traceEnt).client).ps.viewangles, &mut preAng);
                }

                G_Damage(
                    traceEnt,
                    ent,
                    ent,
                    addr_of_mut!(forward),
                    &mut { tr.endpos } as *mut vec3_t,
                    damage,
                    DAMAGE_NO_KNOCKBACK,
                    MOD_DISRUPTOR_SNIPER,
                );

                if !(*traceEnt).client.is_null()
                    && preHealth > 0
                    && (*traceEnt).health <= 0
                    && fullCharge != QFALSE
                    && G_CanDisruptify(traceEnt) != QFALSE
                {
                    //was killed by a fully charged sniper shot, so disintegrate
                    VectorCopy(&preAng, &mut (*(*traceEnt).client).ps.viewangles);

                    (*(*traceEnt).client).ps.eFlags |= EF_DISINTEGRATION;
                    VectorCopy(&tr.endpos, &mut (*(*traceEnt).client).ps.lastHitLoc);

                    (*(*traceEnt).client).ps.legsAnim = preLegs;
                    (*(*traceEnt).client).ps.torsoAnim = preTorso;

                    (*traceEnt).r.contents = 0;

                    VectorClear(&mut (*(*traceEnt).client).ps.velocity);
                }

                tent = G_TempEntity(&tr.endpos, EV_DISRUPTOR_HIT);
                (*tent).s.eventParm = DirToByte(&tr.plane.normal);
                if !(*traceEnt).client.is_null() {
                    (*tent).s.weapon = 1;
                }
            }
        } else
        // not rendering impact, must be a skybox or other similar thing?
        {
            break; // don't try anymore traces
        }

        // Get ready for an attempt to trace through another person
        VectorCopy(&tr.endpos, &mut *addr_of_mut!(muzzle));
        VectorCopy(&tr.endpos, &mut start);
        skip = tr.entityNum as c_int;

        i += 1;
    }
}

/// `static void WP_FireDisruptor( gentity_t *ent, qboolean altFire )` (g_weapon.c:884).
///
/// The disruptor fire dispatcher. Forces `altFire` off unless the firer is a client zoomed
/// into the disruptor (`zoomMode == 1`); an `ET_NPC` animent (no `client`) always takes the
/// alt path; otherwise alt vs. main per `altFire`. `static` in C — ported `pub` though the C
/// is `static` (the g_cmds.rs `G_SayTo` leaf precedent) to avoid a `dead_code` warning until
/// its caller `FireWeapon` (not yet ported) lands. No oracle (gentity-state dispatcher delegating
/// to the two trap/side-effect fire helpers).
///
/// # Safety
/// `ent` may be null (checked before deref); if non-null its `client` is checked before
/// deref. `g_entities`/`level` and the disruptor cvars must be initialised.
pub unsafe fn WP_FireDisruptor(ent: *mut gentity_t, mut altFire: qboolean) {
    if ent.is_null() || (*ent).client.is_null() || (*(*ent).client).ps.zoomMode != 1 {
        //do not ever let it do the alt fire when not zoomed
        altFire = QFALSE;
    }

    if !ent.is_null() && (*ent).s.eType == ET_NPC as c_int && (*ent).client.is_null() {
        //special case for animents
        WP_DisruptorAltFire(ent);
        return;
    }

    if altFire != QFALSE {
        WP_DisruptorAltFire(ent);
    } else {
        WP_DisruptorMainFire(ent);
    }
}

/// `void CalcMuzzlePoint( gentity_t *ent, vec3_t forward, vec3_t right, vec3_t up, vec3_t muzzlePoint )`
/// (g_weapon.c:3489).
///
/// Set the muzzle location relative to the firer's pivoting eye, using the per-weapon
/// offset table `WP_MuzzlePoint`. `up` is unused (kept for source/ABI fidelity). Snaps
/// the result to the integer grid via the native round-ties-even `SnapVector`
/// (`snap_vector`; see DEVIATIONS). No oracle (gentity pointer + global table).
///
/// NOTE: C's `forward` parameter is spelled `fwd` here. Rust forbids a function
/// parameter from shadowing a `static` (E0530), and the file-static `forward`
/// (g_weapon.c:13) is kept verbatim because every fire function reads it; the param
/// is the lower-traffic name, so it gives. See DEVIATIONS.
///
/// # Safety
/// `ent` and `ent->client` must be valid pointers.
pub unsafe fn CalcMuzzlePoint(
    ent: *mut gentity_t,
    fwd: &vec3_t,
    right: &vec3_t,
    _up: &vec3_t,
    muzzlePoint: &mut vec3_t,
) {
    let weapontype = (*ent).s.weapon;
    VectorCopy(&(*ent).s.pos.trBase, muzzlePoint);

    let mut muzzleOffPoint: vec3_t = [0.0; 3];
    VectorCopy(&WP_MuzzlePoint[weapontype as usize], &mut muzzleOffPoint);

    if weapontype > WP_NONE && weapontype < WP_NUM_WEAPONS {
        // Use the table to generate the muzzlepoint;
        // Crouching. Use the add-to-Z method to adjust vertically.
        let base = *muzzlePoint;
        VectorMA(&base, muzzleOffPoint[0], fwd, muzzlePoint);
        let base = *muzzlePoint;
        VectorMA(&base, muzzleOffPoint[1], right, muzzlePoint);
        muzzlePoint[2] += (*(*ent).client).ps.viewheight as f32 + muzzleOffPoint[2];
    }

    // snap to integer coordinates for more efficient network bandwidth usage
    snap_vector(muzzlePoint);
}

/// `void CalcMuzzlePointOrigin( gentity_t *ent, vec3_t origin, vec3_t forward, vec3_t right, vec3_t up, vec3_t muzzlePoint )`
/// (g_weapon.c:3519).
///
/// Simpler muzzle calc: the firer's eye point plus 14 units along `forward`, snapped to
/// the grid. `origin`/`right`/`up` are unused (kept for source/ABI fidelity). No oracle.
///
/// NOTE: C's `forward` parameter is spelled `fwd` here, to avoid shadowing the file-static
/// `forward` (g_weapon.c:13) — Rust E0530. See `CalcMuzzlePoint` and DEVIATIONS.
///
/// # Safety
/// `ent` and `ent->client` must be valid pointers.
pub unsafe fn CalcMuzzlePointOrigin(
    ent: *mut gentity_t,
    _origin: &vec3_t,
    fwd: &vec3_t,
    _right: &vec3_t,
    _up: &vec3_t,
    muzzlePoint: &mut vec3_t,
) {
    VectorCopy(&(*ent).s.pos.trBase, muzzlePoint);
    muzzlePoint[2] += (*(*ent).client).ps.viewheight as f32;
    let base = *muzzlePoint;
    VectorMA(&base, 14.0, fwd, muzzlePoint);
    // snap to integer coordinates for more efficient network bandwidth usage
    snap_vector(muzzlePoint);
}

/// `void W_TraceSetStart( gentity_t *ent, vec3_t start, vec3_t mins, vec3_t maxs )`
/// (g_weapon.c:180).
///
/// Pull `start` back to the firer's eye if it began on the far side of a wall: if the
/// shot box already sits inside the firer's bbox, leave it; otherwise trace from the eye
/// to `start` and clamp `start` to the first solid hit. No oracle (gentity + `trap_Trace`).
///
/// # Safety
/// `ent` must be valid; `ent->client` is checked before deref.
pub unsafe fn W_TraceSetStart(
    ent: *mut gentity_t,
    start: &mut vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
) {
    // make sure our start point isn't on the other side of a wall
    let mut entMins: vec3_t = [0.0; 3];
    let mut entMaxs: vec3_t = [0.0; 3];

    VectorAdd(&(*ent).r.currentOrigin, &(*ent).r.mins, &mut entMins);
    VectorAdd(&(*ent).r.currentOrigin, &(*ent).r.maxs, &mut entMaxs);

    if G_BoxInBounds(start, mins, maxs, &entMins, &entMaxs) != QFALSE {
        return;
    }

    if (*ent).client.is_null() {
        return;
    }

    let mut eyePoint: vec3_t = [0.0; 3];
    VectorCopy(&(*ent).s.pos.trBase, &mut eyePoint);
    eyePoint[2] += (*(*ent).client).ps.viewheight as f32;

    let tr = trap::Trace(
        &eyePoint,
        mins,
        maxs,
        start,
        (*ent).s.number,
        MASK_SOLID | CONTENTS_SHOTCLIP,
    );

    if tr.startsolid != 0 || tr.allsolid != 0 {
        return;
    }

    if tr.fraction < 1.0 {
        VectorCopy(&tr.endpos, start);
    }
}

/// `static void WP_TraceSetStart( gentity_t *ent, vec3_t start, vec3_t mins, vec3_t maxs )`
/// (g_weapon.c:1503).
///
/// Sibling of `W_TraceSetStart`: pull `start` back to the firer if it began on the far side
/// of a wall. Differs only in where the corrective trace originates — here from the firer's
/// player-state origin (`ent->client->ps.origin`), not the eye/`trBase`. No oracle (gentity
/// + `trap_Trace`).
///
/// # Safety
/// `ent` must be valid; `ent->client` is checked before deref.
pub unsafe fn WP_TraceSetStart(
    ent: *mut gentity_t,
    start: &mut vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
) {
    // make sure our start point isn't on the other side of a wall
    let mut entMins: vec3_t = [0.0; 3];
    let mut entMaxs: vec3_t = [0.0; 3];

    VectorAdd(&(*ent).r.currentOrigin, &(*ent).r.mins, &mut entMins);
    VectorAdd(&(*ent).r.currentOrigin, &(*ent).r.maxs, &mut entMaxs);

    if G_BoxInBounds(start, mins, maxs, &entMins, &entMaxs) != QFALSE {
        return;
    }

    if (*ent).client.is_null() {
        return;
    }

    let tr = trap::Trace(
        &(*(*ent).client).ps.origin,
        mins,
        maxs,
        start,
        (*ent).s.number,
        MASK_SOLID | CONTENTS_SHOTCLIP,
    );

    if tr.startsolid != 0 || tr.allsolid != 0 {
        return;
    }

    if tr.fraction < 1.0 {
        VectorCopy(&tr.endpos, start);
    }
}

/// `void WP_CalcVehMuzzle(gentity_t *ent, int muzzleNum)` (g_weapon.c:3539). Computes and
/// caches the world-space muzzle origin/direction for one of a vehicle's muzzles, once per
/// server frame. It reads the bolt matrix off the vehicle's ghoul2 instance at the muzzle's
/// tag (using a yaw-only viewangle for ground vehicles — animal/walker/speeder zero out
/// pitch/roll) via the no-reconstruct/no-rotate trap, then pulls the origin and forward
/// (`NEGATIVE_Y`) vectors into `m_vMuzzlePos`/`m_vMuzzleDir`. No oracle — reads the `level`
/// global and calls a ghoul2 trap.
///
/// # Safety
/// `ent` must point to a valid vehicle `gentity_t` with a non-null `client` and `m_pVehicle`;
/// `muzzleNum` must index the vehicle's muzzle arrays; `level` must be initialised.
pub unsafe fn WP_CalcVehMuzzle(ent: *mut gentity_t, muzzleNum: c_int) {
    let pVeh: *mut Vehicle_t = (*ent).m_pVehicle;
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let mut vehAngles: vec3_t = [0.0; 3];

    debug_assert!(!pVeh.is_null());

    if (*pVeh).m_iMuzzleTime[muzzleNum as usize] == (*addr_of!(level)).time {
        //already done for this frame, don't need to do it again
        return;
    }
    //Uh... how about we set this, hunh...?  :)
    (*pVeh).m_iMuzzleTime[muzzleNum as usize] = (*addr_of!(level)).time;

    VectorCopy(&(*(*ent).client).ps.viewangles, &mut vehAngles);
    if !(*pVeh).m_pVehicleInfo.is_null()
        && ((*(*pVeh).m_pVehicleInfo).r#type == VH_ANIMAL
            || (*(*pVeh).m_pVehicleInfo).r#type == VH_WALKER
            || (*(*pVeh).m_pVehicleInfo).r#type == VH_SPEEDER)
    {
        vehAngles[PITCH as usize] = 0.0;
        vehAngles[ROLL as usize] = 0.0;
    }

    crate::trap::G2API_GetBoltMatrix_NoRecNoRot(
        (*ent).ghoul2,
        0,
        (*pVeh).m_iMuzzleTag[muzzleNum as usize],
        &mut boltMatrix,
        &vehAngles,
        &(*(*ent).client).ps.origin,
        (*addr_of!(level)).time,
        core::ptr::null_mut(),
        &(*ent).modelScale,
    );
    BG_GiveMeVectorFromMatrix(
        &boltMatrix,
        ORIGIN,
        &mut (*pVeh).m_vMuzzlePos[muzzleNum as usize],
    );
    BG_GiveMeVectorFromMatrix(
        &boltMatrix,
        NEGATIVE_Y,
        &mut (*pVeh).m_vMuzzleDir[muzzleNum as usize],
    );
}

/// `#define FLECHETTE_MINE_RADIUS_CHECK 256` (g_weapon.c:83) — proximity-trigger radius for
/// the flechette alt-fire mine.
const FLECHETTE_MINE_RADIUS_CHECK: f32 = 256.0;

/// `static gentity_t *ent_list[MAX_GENTITIES]` (g_weapon.c:1156) — the file-static scratch
/// array [`G_RadiusList`] fills for [`prox_mine_think`]'s proximity scan. Declared with the
/// first reader that lands. Accessed via `addr_of_mut!` (never `&mut`) per the static-mut rule.
static mut ent_list: [*mut gentity_t; MAX_GENTITIES] = [null_mut(); MAX_GENTITIES];

/// `void prox_mine_think( gentity_t *ent )` (g_weapon.c:1464) — the per-frame `think` for a
/// laid flechette proximity mine. While its `delay` hasn't elapsed it does a small radius
/// scan ([`G_RadiusList`] within [`FLECHETTE_MINE_RADIUS_CHECK`]) and detonates if any living
/// client other than its own `activator` is inside; once the auto-explode time passes it just
/// detonates. Detonation arms [`laserTrapExplode`] for 200ms later; otherwise it re-thinks in
/// 500ms (the proximity logic is cheap enough not to need to run every frame). No oracle
/// (gentity-state / `trap`-backed radius query). A `pub unsafe extern "C"` fn for the
/// `gentity_t::think` fn-pointer ABI.
///
/// # Safety
/// `ent` must point to a valid mine `gentity_t`; `level`/`g_entities` must be initialised.
pub unsafe extern "C" fn prox_mine_think(ent: *mut gentity_t) {
    let count: c_int;
    let mut blow: qboolean = QFALSE;

    // if it isn't time to auto-explode, do a small proximity check
    if (*ent).delay > (*addr_of!(level)).time {
        count = G_RadiusList(
            &(*ent).r.currentOrigin,
            FLECHETTE_MINE_RADIUS_CHECK,
            ent,
            QTRUE,
            &mut *addr_of_mut!(ent_list),
        );

        for i in 0..count {
            let cand = (*addr_of!(ent_list))[i as usize];
            if !(*cand).client.is_null()
                && (*cand).health > 0
                && !(*ent).activator.is_null()
                && (*cand).s.number != (*(*ent).activator).s.number
            {
                blow = QTRUE;
                break;
            }
        }
    } else {
        // well, we must die now
        blow = QTRUE;
    }

    if blow != QFALSE {
        (*ent).think = Some(laserTrapExplode);
        (*ent).nextthink = (*addr_of!(level)).time + 200;
    } else {
        // we probably don't need to do this thinking logic very often...maybe this is fast enough?
        (*ent).nextthink = (*addr_of!(level)).time + 500;
    }
}

/// `void WP_ExplosiveDie( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int
/// damage, int mod )` (g_weapon.c:1537) — the `die` callback for an explosive (mine) entity:
/// it simply detonates via [`laserTrapExplode`]. `inflictor`/`attacker`/`damage`/`mod` are
/// unused. No oracle (entity-state mutation). A `pub unsafe extern "C"` fn for the
/// `gentity_t::die` fn-pointer ABI.
///
/// # Safety
/// `self_` must point to a valid mine `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn WP_ExplosiveDie(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    _mod: c_int,
) {
    laserTrapExplode(self_);
}

/// `void WP_flechette_alt_blow( gentity_t *ent )` (g_weapon.c:1543) — the `think` callback for
/// a flechette alt-fire bouncy submunition reaching the end of its fuse: it forces a unit `+x`
/// trajectory delta (so the explosion has a defined direction) and detonates via
/// [`laserTrapExplode`]. No oracle (entity-state mutation). A `pub unsafe extern "C"` fn for
/// the `gentity_t::think` fn-pointer ABI.
///
/// # Safety
/// `ent` must point to a valid flechette submunition `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn WP_flechette_alt_blow(ent: *mut gentity_t) {
    (*ent).s.pos.trDelta[0] = 1.0;
    (*ent).s.pos.trDelta[1] = 0.0;
    (*ent).s.pos.trDelta[2] = 0.0;

    laserTrapExplode(ent);
}

/// `void WP_FireTurretMissile( gentity_t *ent, vec3_t start, vec3_t dir, qboolean altFire,
/// int damage, int velocity, int mod, gentity_t *ignore )` (g_weapon.c:302).
///
/// Spawn a generic turret projectile, optionally set to pass through `ignore`. No oracle.
///
/// # Safety
/// `ent` must be valid; `ignore` may be NULL (checked). `start` is snapped in place.
#[allow(clippy::too_many_arguments)]
pub unsafe fn WP_FireTurretMissile(
    ent: *mut gentity_t,
    start: &mut vec3_t,
    dir: &vec3_t,
    altFire: qboolean,
    damage: c_int,
    velocity: c_int,
    mod_: c_int,
    ignore: *mut gentity_t,
) {
    let missile = CreateMissile(start, dir, velocity as f32, 10000, ent, altFire);

    (*missile).classname = c"generic_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_TURRET;

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = mod_;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    if !ignore.is_null() {
        (*missile).passThroughNum = (*ignore).s.number + 1;
    }

    // we don't want it to bounce forever
    (*missile).bounceCount = 8;
}

/// `void WP_FireGenericBlasterMissile( gentity_t *ent, vec3_t start, vec3_t dir,
/// qboolean altFire, int damage, int velocity, int mod )` (g_weapon.c:329).
///
/// Spawn a generic blaster bolt (the seeker drone's shot, among others). No oracle.
///
/// # Safety
/// `ent` must be valid. `start` is snapped in place.
pub unsafe fn WP_FireGenericBlasterMissile(
    ent: *mut gentity_t,
    start: &mut vec3_t,
    dir: &vec3_t,
    altFire: qboolean,
    damage: c_int,
    velocity: c_int,
    mod_: c_int,
) {
    let missile = CreateMissile(start, dir, velocity as f32, 10000, ent, altFire);

    (*missile).classname = c"generic_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = mod_;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to bounce forever
    (*missile).bounceCount = 8;
}

/// `void WP_FireBlasterMissile( gentity_t *ent, vec3_t start, vec3_t dir, qboolean altFire )`
/// (g_weapon.c:357).
///
/// Spawn the E-11 blaster bolt. Animents (`ET_NPC`) do a flat 10 damage. No oracle.
///
/// # Safety
/// `ent` must be valid. `start` is snapped in place.
pub unsafe fn WP_FireBlasterMissile(
    ent: *mut gentity_t,
    start: &mut vec3_t,
    dir: &vec3_t,
    altFire: qboolean,
) {
    let velocity = BLASTER_VELOCITY;
    let mut damage = BLASTER_DAMAGE;

    if (*ent).s.eType == ET_NPC {
        // animent
        damage = 10;
    }

    let missile = CreateMissile(start, dir, velocity as f32, 10000, ent, altFire);

    (*missile).classname = c"blaster_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BLASTER;

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_BLASTER;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to bounce forever
    (*missile).bounceCount = 8;
}

/// `void WP_FireTurboLaserMissile( gentity_t *ent, vec3_t start, vec3_t dir )`
/// (g_weapon.c:384).
///
/// Spawn a vehicle turbolaser bolt: velocity/damage/splash come off the firing entity, the
/// shot/impact effects are carried on `otherEntityNum2`/`emplacedOwner`, and it self-frees
/// after 5s via `G_FreeEntity`. No oracle.
///
/// # Safety
/// `ent` must be valid. `start` is snapped in place. `level` must be initialised.
pub unsafe fn WP_FireTurboLaserMissile(ent: *mut gentity_t, start: &mut vec3_t, dir: &vec3_t) {
    let velocity = (*ent).mass as c_int; //FIXME: externalize

    let missile = CreateMissile(start, dir, velocity as f32, 10000, ent, QFALSE);

    //use a custom shot effect
    (*missile).s.otherEntityNum2 = (*ent).genericValue14;
    //use a custom impact effect
    (*missile).s.emplacedOwner = (*ent).genericValue15;

    (*missile).classname = c"turbo_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_TURRET;

    (*missile).damage = (*ent).damage; //FIXME: externalize
    (*missile).splashDamage = (*ent).splashDamage; //FIXME: externalize
    (*missile).splashRadius = (*ent).splashRadius; //FIXME: externalize
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_TARGET_LASER; //MOD_TURBLAST; //count as a heavy weap
    (*missile).splashMethodOfDeath = MOD_TARGET_LASER; //MOD_TURBLAST;// ?SPLASH;
    (*missile).clipmask = MASK_SHOT;

    // we don't want it to bounce forever
    (*missile).bounceCount = 8;

    //set veh as cgame side owner for purpose of fx overrides
    (*missile).s.owner = (*ent).s.number;

    //don't let them last forever
    (*missile).think = Some(G_FreeEntity);
    (*missile).nextthink = (*addr_of!(level)).time + 5000; //at 20000 speed, that should be more than enough
}

/// `void WP_FireEmplacedMissile( gentity_t *ent, vec3_t start, vec3_t dir, qboolean altFire,
/// gentity_t *ignore )` (g_weapon.c:420).
///
/// Spawn an emplaced-gun blaster bolt (heavy-weapon class), optionally passing through
/// `ignore`. No oracle.
///
/// # Safety
/// `ent` must be valid; `ignore` may be NULL (checked). `start` is snapped in place.
pub unsafe fn WP_FireEmplacedMissile(
    ent: *mut gentity_t,
    start: &mut vec3_t,
    dir: &vec3_t,
    altFire: qboolean,
    ignore: *mut gentity_t,
) {
    let velocity = BLASTER_VELOCITY;
    let damage = BLASTER_DAMAGE;

    let missile = CreateMissile(start, dir, velocity as f32, 10000, ent, altFire);

    (*missile).classname = c"emplaced_gun_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_TURRET; //WP_EMPLACED_GUN;

    (*missile).activator = ignore;

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK | DAMAGE_HEAVY_WEAP_CLASS;
    (*missile).methodOfDeath = MOD_VEHICLE;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    if !ignore.is_null() {
        (*missile).passThroughNum = (*ignore).s.number + 1;
    }

    // we don't want it to bounce forever
    (*missile).bounceCount = 8;
}

/// `static void WP_FireBryarPistol( gentity_t *ent, qboolean altFire )` (g_weapon.c).
///
/// First of the player-fire helpers: spawn a bryar bolt from the file-static `muzzle`
/// along `forward` (both written by `FireWeapon`). On alt-fire, scale damage and the
/// bolt bbox by the charge level (1..5 units, one accumulated per `BRYAR_CHARGE_UNIT`
/// ms since `weaponChargeTime`); `generic1` carries the charge to the renderer. No
/// oracle (gentity spawn + `level` + file-static aim state).
///
/// # Safety
/// `ent` and `ent->client` must be valid pointers; reads the file-static `muzzle`/`forward`.
pub unsafe fn WP_FireBryarPistol(ent: *mut gentity_t, altFire: qboolean) {
    let mut damage = BRYAR_PISTOL_DAMAGE;

    let missile = CreateMissile(
        &mut *addr_of_mut!(muzzle),
        &*addr_of!(forward),
        BRYAR_PISTOL_VEL as f32,
        10000,
        ent,
        altFire,
    );

    (*missile).classname = c"bryar_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    if altFire != QFALSE {
        let mut count = (((*addr_of!(level)).time - (*(*ent).client).ps.weaponChargeTime) as f32
            / BRYAR_CHARGE_UNIT) as c_int;

        if count < 1 {
            count = 1;
        } else if count > 5 {
            count = 5;
        }

        if count > 1 {
            damage = (damage as f64 * (count as f64 * 1.7)) as c_int;
        } else {
            damage = (damage as f64 * (count as f64 * 1.5)) as c_int;
        }

        (*missile).s.generic1 = count; // The missile will then render according to the charge level.

        let boxSize = (BRYAR_ALT_SIZE as f64 * (count as f64 * 0.5)) as f32;

        VectorSet(&mut (*missile).r.maxs, boxSize, boxSize, boxSize);
        VectorSet(&mut (*missile).r.mins, -boxSize, -boxSize, -boxSize);
    }

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    if altFire != QFALSE {
        (*missile).methodOfDeath = MOD_BRYAR_PISTOL_ALT;
    } else {
        (*missile).methodOfDeath = MOD_BRYAR_PISTOL;
    }
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to bounce forever
    (*missile).bounceCount = 8;
}

/// `static void WP_FireBlaster( gentity_t *ent, qboolean altFire )` (g_weapon.c).
///
/// Convert the file-static `forward` aim to angles, add random spread on alt-fire
/// (`crandom() * BLASTER_SPREAD` per axis), rebuild a direction, and hand it plus the
/// file-static `muzzle` to `WP_FireBlasterMissile`. `crandom` is C's `double` macro, so
/// the spread is computed in `f64` and truncated to the `f32` angle (C `float +=`). No
/// oracle (file-static aim state + RNG + gentity spawn).
///
/// # Safety
/// `ent` must be valid; reads the file-static `forward`/`muzzle`.
pub unsafe fn WP_FireBlaster(ent: *mut gentity_t, altFire: qboolean) {
    let mut dir: vec3_t = [0.0; 3];
    let mut angs: vec3_t = [0.0; 3];

    vectoangles(&*addr_of!(forward), &mut angs);

    if altFire != QFALSE {
        // add some slop to the alt-fire direction
        angs[PITCH] += (crandom() * BLASTER_SPREAD as f64) as f32;
        angs[YAW] += (crandom() * BLASTER_SPREAD as f64) as f32;
    }

    AngleVectors(&angs, Some(&mut dir), None, None);

    // FIXME: if temp_org does not have clear trace to inside the bbox, don't shoot!
    WP_FireBlasterMissile(ent, &mut *addr_of_mut!(muzzle), &dir, altFire);
}

/// `static void WP_BowcasterMainFire( gentity_t *ent )` (g_weapon.c).
///
/// Charge-scaled spread shot: fan an odd number of bolts (1..5, clamped to odd) from the
/// file-static `muzzle`, each with a `crandom`-varied velocity and yaw spread fanned about
/// `forward`; damage drops as the bolt count rises. Velocity/pitch slop go through C's
/// `double` (crandom) then truncate to the `f32` field; the yaw fan is pure `f32`. No
/// oracle (file-static aim + RNG + gentity spawn loop).
///
/// # Safety
/// `ent` must be valid; `ent->client` is checked before deref. Reads `forward`/`muzzle`.
pub unsafe fn WP_BowcasterMainFire(ent: *mut gentity_t) {
    // C inits `damage = BOWCASTER_DAMAGE` then the count-based chain below overwrites it
    // on every path (a dead store in the C); deferred-init here to keep it warning-clean.
    let damage;
    let mut count;

    if (*ent).client.is_null() {
        count = 1;
    } else {
        count = ((((*addr_of!(level)).time - (*(*ent).client).ps.weaponChargeTime) as f32)
            / BOWCASTER_CHARGE_UNIT) as c_int;
    }

    if count < 1 {
        count = 1;
    } else if count > 5 {
        count = 5;
    }

    if count & 1 == 0 {
        // if we aren't odd, knock us down a level
        count -= 1;
    }

    // scale the damage down based on how many are about to be fired
    if count <= 1 {
        damage = 50;
    } else if count == 2 {
        damage = 45;
    } else if count == 3 {
        damage = 40;
    } else if count == 4 {
        damage = 35;
    } else {
        damage = 30;
    }

    for i in 0..count {
        // create a range of different velocities
        let vel =
            (BOWCASTER_VELOCITY as f64 * (crandom() * BOWCASTER_VEL_RANGE as f64 + 1.0)) as f32;

        let mut angs: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        vectoangles(&*addr_of!(forward), &mut angs);

        // add some slop to the alt-fire direction
        angs[PITCH] += (crandom() * BOWCASTER_ALT_SPREAD as f64 * 0.2) as f32;
        angs[YAW] += (i as f32 + 0.5) * BOWCASTER_ALT_SPREAD
            - count as f32 * 0.5 * BOWCASTER_ALT_SPREAD;

        AngleVectors(&angs, Some(&mut dir), None, None);

        let missile = CreateMissile(&mut *addr_of_mut!(muzzle), &dir, vel, 10000, ent, QTRUE);

        (*missile).classname = c"bowcaster_alt_proj".as_ptr() as *mut c_char;
        (*missile).s.weapon = WP_BOWCASTER;

        VectorSet(
            &mut (*missile).r.maxs,
            BOWCASTER_SIZE as f32,
            BOWCASTER_SIZE as f32,
            BOWCASTER_SIZE as f32,
        );
        let maxs = (*missile).r.maxs;
        VectorScale(&maxs, -1.0, &mut (*missile).r.mins);

        (*missile).damage = damage;
        (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
        (*missile).methodOfDeath = MOD_BOWCASTER;
        (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

        // we don't want it to bounce
        (*missile).bounceCount = 0;
    }
}

/// `static void WP_BowcasterAltFire( gentity_t *ent )` (g_weapon.c).
///
/// Single bouncing bolt fired straight along the file-static `forward` from `muzzle`.
/// No oracle (file-static aim + gentity spawn).
///
/// # Safety
/// `ent` must be valid; reads the file-static `forward`/`muzzle`.
pub unsafe fn WP_BowcasterAltFire(ent: *mut gentity_t) {
    let damage = BOWCASTER_DAMAGE;

    let missile = CreateMissile(
        &mut *addr_of_mut!(muzzle),
        &*addr_of!(forward),
        BOWCASTER_VELOCITY as f32,
        10000,
        ent,
        QFALSE,
    );

    (*missile).classname = c"bowcaster_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BOWCASTER;

    VectorSet(
        &mut (*missile).r.maxs,
        BOWCASTER_SIZE as f32,
        BOWCASTER_SIZE as f32,
        BOWCASTER_SIZE as f32,
    );
    let maxs = (*missile).r.maxs;
    VectorScale(&maxs, -1.0, &mut (*missile).r.mins);

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_BOWCASTER;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    (*missile).flags |= FL_BOUNCE;
    (*missile).bounceCount = 3;
}

/// `static void WP_FireBowcaster( gentity_t *ent, qboolean altFire )` (g_weapon.c).
///
/// Dispatch bowcaster fire to the single bouncing alt bolt or the charge-scaled main
/// spread. No oracle (thin dispatch over gentity-spawning helpers).
///
/// # Safety
/// `ent` must be valid.
pub unsafe fn WP_FireBowcaster(ent: *mut gentity_t, altFire: qboolean) {
    if altFire != QFALSE {
        WP_BowcasterAltFire(ent);
    } else {
        WP_BowcasterMainFire(ent);
    }
}

/// `static void WP_RepeaterMainFire( gentity_t *ent, vec3_t dir )` (g_weapon.c).
///
/// Spawn one repeater bolt from the file-static `muzzle` along the caller-supplied `dir`.
/// No oracle (file-static aim + gentity spawn).
///
/// # Safety
/// `ent` must be valid; reads the file-static `muzzle`.
pub unsafe fn WP_RepeaterMainFire(ent: *mut gentity_t, dir: &vec3_t) {
    let damage = REPEATER_DAMAGE;

    let missile = CreateMissile(
        &mut *addr_of_mut!(muzzle),
        dir,
        REPEATER_VELOCITY as f32,
        10000,
        ent,
        QFALSE,
    );

    (*missile).classname = c"repeater_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_REPEATER;

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_REPEATER;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to bounce forever
    (*missile).bounceCount = 8;
}

/// `static void WP_RepeaterAltFire( gentity_t *ent )` (g_weapon.c).
///
/// Lob a heavier gravity-affected grenade bolt straight along the file-static `forward`
/// from `muzzle`, given a slight upward boost and a splash radius narrowed in Siege. No
/// oracle (file-static aim + `g_gametype` + gentity spawn).
///
/// # Safety
/// `ent` must be valid; reads the file-static `forward`/`muzzle`.
pub unsafe fn WP_RepeaterAltFire(ent: *mut gentity_t) {
    let damage = REPEATER_ALT_DAMAGE;

    let missile = CreateMissile(
        &mut *addr_of_mut!(muzzle),
        &*addr_of!(forward),
        REPEATER_ALT_VELOCITY as f32,
        10000,
        ent,
        QTRUE,
    );

    (*missile).classname = c"repeater_alt_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_REPEATER;

    VectorSet(
        &mut (*missile).r.maxs,
        REPEATER_ALT_SIZE as f32,
        REPEATER_ALT_SIZE as f32,
        REPEATER_ALT_SIZE as f32,
    );
    let maxs = (*missile).r.maxs;
    VectorScale(&maxs, -1.0, &mut (*missile).r.mins);
    (*missile).s.pos.trType = TR_GRAVITY;
    (*missile).s.pos.trDelta[2] += 40.0; //give a slight boost in the upward direction
    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_REPEATER_ALT;
    (*missile).splashMethodOfDeath = MOD_REPEATER_ALT_SPLASH;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    (*missile).splashDamage = REPEATER_ALT_SPLASH_DAMAGE;
    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        // we've been having problems with this being too hyper-potent because of it's radius
        (*missile).splashRadius = REPEATER_ALT_SPLASH_RAD_SIEGE;
    } else {
        (*missile).splashRadius = REPEATER_ALT_SPLASH_RADIUS;
    }

    // we don't want it to bounce forever
    (*missile).bounceCount = 8;
}

/// `static void WP_FireRepeater( gentity_t *ent, qboolean altFire )` (g_weapon.c).
///
/// Dispatch repeater fire: the gravity grenade on alt, or a spread-jittered bolt on
/// main (the same `vectoangles` of the file-static `forward` runs first either way,
/// matching the C). No oracle (file-static aim + RNG + gentity spawn).
///
/// # Safety
/// `ent` must be valid; reads the file-static `forward`.
pub unsafe fn WP_FireRepeater(ent: *mut gentity_t, altFire: qboolean) {
    let mut dir: vec3_t = [0.0; 3];
    let mut angs: vec3_t = [0.0; 3];

    vectoangles(&*addr_of!(forward), &mut angs);

    if altFire != QFALSE {
        WP_RepeaterAltFire(ent);
    } else {
        // add some slop to the alt-fire direction
        angs[PITCH] += (crandom() * REPEATER_SPREAD as f64) as f32;
        angs[YAW] += (crandom() * REPEATER_SPREAD as f64) as f32;

        AngleVectors(&angs, Some(&mut dir), None, None);

        WP_RepeaterMainFire(ent, &dir);
    }
}

/// `static void WP_FlechetteMainFire( gentity_t *ent )` (g_weapon.c:1424).
///
/// Shotgun-style burst: fan `FLECHETTE_SHOTS` bouncing shrapnel bolts from the file-static
/// `muzzle`. The first bolt flies straight along `forward` (hits the crosshair); the rest
/// get `crandom`-jittered pitch/yaw spread. Each bolt bounces a `Q_irand(5,8)` number of
/// times. The pitch/yaw slop goes through C's `double` (crandom) before truncating to the
/// `f32` angle field. No oracle (file-static aim + RNG + gentity spawn loop).
///
/// # Safety
/// `ent` must be valid; reads the file-static `forward`/`muzzle`.
pub unsafe fn WP_FlechetteMainFire(ent: *mut gentity_t) {
    for i in 0..FLECHETTE_SHOTS {
        let mut angs: vec3_t = [0.0; 3];
        let mut fwd: vec3_t = [0.0; 3];
        vectoangles(&*addr_of!(forward), &mut angs);

        if i != 0 {
            // do nothing on the first shot, it will hit the crosshairs
            angs[PITCH] += (crandom() * FLECHETTE_SPREAD as f64) as f32;
            angs[YAW] += (crandom() * FLECHETTE_SPREAD as f64) as f32;
        }

        AngleVectors(&angs, Some(&mut fwd), None, None);

        let missile = CreateMissile(
            &mut *addr_of_mut!(muzzle),
            &fwd,
            FLECHETTE_VEL as f32,
            10000,
            ent,
            QFALSE,
        );

        (*missile).classname = c"flech_proj".as_ptr() as *mut c_char;
        (*missile).s.weapon = WP_FLECHETTE;

        VectorSet(
            &mut (*missile).r.maxs,
            FLECHETTE_SIZE as f32,
            FLECHETTE_SIZE as f32,
            FLECHETTE_SIZE as f32,
        );
        let maxs = (*missile).r.maxs;
        VectorScale(&maxs, -1.0, &mut (*missile).r.mins);

        (*missile).damage = FLECHETTE_DAMAGE;
        (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
        (*missile).methodOfDeath = MOD_FLECHETTE;
        (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

        // we don't want it to bounce forever
        (*missile).bounceCount = Q_irand(5, 8);

        (*missile).flags |= FL_BOUNCE_SHRAPNEL;
    }
}

/// `static void WP_CreateFlechetteBouncyThing( vec3_t start, vec3_t fwd, gentity_t *self )`
/// (g_weapon.c:1554).
///
/// Spawn one flechette alt-fire bouncy submunition: a gravity-bound, half-bouncing missile
/// with a 3-unit cube bbox that detonates on its fuse via [`WP_flechette_alt_blow`] (or
/// touch-NULL on contact, since alt ones explode at end-of-fuse rather than on impact). No
/// oracle (gentity spawn + entity-state). `static` in C.
///
/// # Safety
/// `self_` must be a valid firer `gentity_t`; `level` must be initialised.
pub unsafe fn WP_CreateFlechetteBouncyThing(
    start: &mut vec3_t,
    fwd: &vec3_t,
    self_: *mut gentity_t,
) {
    let missile = CreateMissile(
        start,
        fwd,
        700.0 + random() * 700.0,
        1500 + (random() * 2000.0) as c_int,
        self_,
        QTRUE,
    );

    (*missile).think = Some(WP_flechette_alt_blow);

    (*missile).activator = self_;

    (*missile).s.weapon = WP_FLECHETTE;
    (*missile).classname = c"flech_alt".as_ptr() as *mut c_char;
    (*missile).mass = 4.0;

    // How 'bout we give this thing a size...
    VectorSet(&mut (*missile).r.mins, -3.0, -3.0, -3.0);
    VectorSet(&mut (*missile).r.maxs, 3.0, 3.0, 3.0);
    (*missile).clipmask = MASK_SHOT;

    (*missile).touch = Some(touch_NULL);

    // normal ones bounce, alt ones explode on impact
    (*missile).s.pos.trType = TR_GRAVITY;

    (*missile).flags |= FL_BOUNCE_HALF;
    (*missile).s.eFlags |= EF_ALT_FIRING;

    (*missile).bounceCount = 50;

    (*missile).damage = FLECHETTE_ALT_DAMAGE;
    (*missile).dflags = 0;
    (*missile).splashDamage = FLECHETTE_ALT_SPLASH_DAM;
    (*missile).splashRadius = FLECHETTE_ALT_SPLASH_RAD;

    (*missile).r.svFlags = SVF_USE_CURRENT_ORIGIN;

    (*missile).methodOfDeath = MOD_FLECHETTE_ALT_SPLASH;
    (*missile).splashMethodOfDeath = MOD_FLECHETTE_ALT_SPLASH;

    VectorCopy(&*start, &mut (*missile).pos2);
}

/// `static void WP_FlechetteAltFire( gentity_t *self )` (g_weapon.c:1596).
///
/// Lob two flechette alt-fire bouncy submunitions from the file-static `muzzle`/`forward`
/// aim, each kicked upward and spread randomly. Uses [`WP_TraceSetStart`] to make sure the
/// start point isn't on the far side of a wall. No oracle (file-static aim + gentity spawn).
/// `static` in C.
///
/// # Safety
/// `self_` must be a valid firer `gentity_t`; reads the file-static `forward`/`muzzle`.
pub unsafe fn WP_FlechetteAltFire(self_: *mut gentity_t) {
    let mut dir: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut angs: vec3_t = [0.0; 3];

    vectoangles(&*addr_of!(forward), &mut angs);
    VectorCopy(&*addr_of!(muzzle), &mut start);

    // make sure our start point isn't on the other side of a wall
    WP_TraceSetStart(self_, &mut start, &vec3_origin, &vec3_origin);

    for _i in 0..2 {
        VectorCopy(&angs, &mut dir);

        dir[PITCH] -= random() * 4.0 + 8.0; // make it fly upwards
        dir[YAW] += crandom() as f32 * 2.0;
        AngleVectors(&dir, Some(&mut fwd), None, None);

        WP_CreateFlechetteBouncyThing(&mut start, &fwd, self_);
    }
}

/// `static void WP_FireFlechette( gentity_t *ent, qboolean altFire )` (g_weapon.c:1620).
///
/// Dispatch flechette fire: alt → [`WP_FlechetteAltFire`], main → [`WP_FlechetteMainFire`].
/// `static` in C.
///
/// # Safety
/// `ent` must be a valid firer `gentity_t`.
pub unsafe fn WP_FireFlechette(ent: *mut gentity_t, altFire: qboolean) {
    if altFire != QFALSE {
        //WP_FlechetteProxMine( ent );
        WP_FlechetteAltFire(ent);
    } else {
        WP_FlechetteMainFire(ent);
    }
}

/// `static void WP_DEMP2_MainFire( gentity_t *ent )` (g_weapon.c:1136).
///
/// Fires a single straight DEMP2 bolt from the file-static `muzzle` along `forward`. The
/// missile gets a `DEMP2_SIZE` cube bbox, `MOD_DEMP2` death, `MASK_SHOT` clip, and is
/// explicitly set to never bounce (`bounceCount = 0`). No oracle (file-static aim + gentity
/// spawn).
///
/// # Safety
/// `ent` must be valid; reads the file-static `forward`/`muzzle`.
pub unsafe fn WP_DEMP2_MainFire(ent: *mut gentity_t) {
    let damage = DEMP2_DAMAGE;

    let missile = CreateMissile(
        &mut *addr_of_mut!(muzzle),
        &*addr_of!(forward),
        DEMP2_VELOCITY as f32,
        10000,
        ent,
        QFALSE,
    );

    (*missile).classname = c"demp2_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_DEMP2;

    VectorSet(&mut (*missile).r.maxs, DEMP2_SIZE, DEMP2_SIZE, DEMP2_SIZE);
    let maxs = (*missile).r.maxs;
    VectorScale(&maxs, -1.0, &mut (*missile).r.mins);
    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_DEMP2;
    (*missile).clipmask = MASK_SHOT;

    // we don't want it to ever bounce
    (*missile).bounceCount = 0;
}

// `void Jedi_Decloak( gentity_t *self )` (NPC_misc.c) — temporarily disables a cloaked NPC's
// cloak; called in `DEMP2_AltRadiusDamage` so the alt-DEMP2 shockwave can decloak a cloaked
// player it hits. Now ported in npc_ai_jedi.rs, so imported directly.
use crate::codemp::game::npc_ai_jedi::Jedi_Decloak;

/// `void DEMP2_AltRadiusDamage( gentity_t *ent )` (g_weapon.c:1158).
///
/// The think for the alt-DEMP2 ion shockwave: each tick it grows an ellipsoidal radius
/// (`frac^3` of the 800 ms life, scaled by `count`) out to a max of `200 * count*0.6`, queries
/// `trap_EntitiesInBox` for everything inside the current box, and damages each takedamage
/// entity whose edge-distance falls in the ring between last tick's edge (`genericValue6`) and
/// the current edge. Hit clients get an electrify window (longer on speeders/walkers, skipped on
/// fighters) and a cloaked client is decloaked via the (not-yet-ported-NPC) `Jedi_Decloak`. Frees
/// itself once the shockwave finishes (`frac >= 1`) or the owner becomes invalid. No oracle
/// (entity-state think + `trap_EntitiesInBox` + `G_Damage`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `level`/`g_entities` must be initialised. A
/// `pub unsafe extern "C"` fn for the `gentity_t::think` fn-pointer ABI.
pub unsafe extern "C" fn DEMP2_AltRadiusDamage(ent: *mut gentity_t) {
    let mut frac: f32 =
        ((*addr_of!(level)).time - (*ent).genericValue5) as f32 / 800.0; // / 1600.0f; // synchronize with demp2 effect
    let mut dist: f32;
    let mut radius: f32;
    let mut fact: f32;
    let mut gent: *mut gentity_t;
    let mut iEntityList: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut entityList: [*mut gentity_t; MAX_GENTITIES] = [null_mut(); MAX_GENTITIES];
    let mut myOwner: *mut gentity_t = null_mut();
    let numListedEntities: c_int;
    let mut i: c_int;
    let mut e: c_int;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut v: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];

    if (*ent).r.ownerNum >= 0
        && (*ent).r.ownerNum < /*MAX_CLIENTS ... let npc's/shooters use it*/ MAX_GENTITIES as c_int
    {
        myOwner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize);
    }

    if myOwner.is_null() || (*myOwner).inuse == QFALSE || (*myOwner).client.is_null() {
        (*ent).think = Some(G_FreeEntity);
        (*ent).nextthink = (*addr_of!(level)).time;
        return;
    }

    frac *= frac * frac; // yes, this is completely ridiculous...but it causes the shell to grow slowly then "explode" at the end

    radius = frac * 200.0; // 200 is max radius...the model is aprox. 100 units tall...the fx draw code mults. this by 2.

    fact = (*ent).count as f32 * 0.6;

    if fact < 1.0 {
        fact = 1.0;
    }

    radius *= fact;

    for i in 0..3 {
        mins[i] = (*ent).r.currentOrigin[i] - radius;
        maxs[i] = (*ent).r.currentOrigin[i] + radius;
    }

    numListedEntities = trap::EntitiesInBox(&mins, &maxs, &mut iEntityList);

    i = 0;
    while i < numListedEntities {
        entityList[i as usize] = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(iEntityList[i as usize] as usize);
        i += 1;
    }

    e = 0;
    while e < numListedEntities {
        gent = entityList[e as usize];

        if gent.is_null() || (*gent).takedamage == QFALSE || (*gent).r.contents == 0 {
            e += 1;
            continue;
        }

        // find the distance from the edge of the bounding box
        for i in 0..3 {
            if (*ent).r.currentOrigin[i] < (*gent).r.absmin[i] {
                v[i] = (*gent).r.absmin[i] - (*ent).r.currentOrigin[i];
            } else if (*ent).r.currentOrigin[i] > (*gent).r.absmax[i] {
                v[i] = (*ent).r.currentOrigin[i] - (*gent).r.absmax[i];
            } else {
                v[i] = 0.0;
            }
        }

        // shape is an ellipsoid, so cut vertical distance in half`
        v[2] *= 0.5;

        dist = VectorLength(&v);

        if dist >= radius {
            // shockwave hasn't hit them yet
            e += 1;
            continue;
        }

        if dist + (16.0 * (*ent).count as f32) < (*ent).genericValue6 as f32 {
            // shockwave has already hit this thing...
            e += 1;
            continue;
        }

        VectorCopy(&(*gent).r.currentOrigin, &mut v);
        VectorSubtract(&v, &(*ent).r.currentOrigin, &mut dir);

        // push the center of mass higher than the origin so players get knocked into the air more
        dir[2] += 12.0;

        if gent != myOwner {
            G_Damage(
                gent,
                myOwner,
                myOwner,
                &mut dir,
                &mut (*ent).r.currentOrigin,
                (*ent).damage,
                DAMAGE_DEATH_KNOCKBACK,
                (*ent).splashMethodOfDeath,
            );
            if (*gent).takedamage != QFALSE && !(*gent).client.is_null() {
                if (*(*gent).client).ps.electrifyTime < (*addr_of!(level)).time {
                    //electrocution effect
                    if (*gent).s.eType == ET_NPC
                        && (*gent).s.NPC_class == CLASS_VEHICLE
                        && !(*gent).m_pVehicle.is_null()
                        && ((*(*(*gent).m_pVehicle).m_pVehicleInfo).r#type == VH_SPEEDER
                            || (*(*(*gent).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER)
                    {
                        //do some extra stuff to speeders/walkers
                        (*(*gent).client).ps.electrifyTime =
                            (*addr_of!(level)).time + Q_irand(3000, 4000);
                    } else if (*gent).s.NPC_class != CLASS_VEHICLE
                        || (!(*gent).m_pVehicle.is_null()
                            && (*(*(*gent).m_pVehicle).m_pVehicleInfo).r#type != VH_FIGHTER)
                    {
                        //don't do this to fighters
                        (*(*gent).client).ps.electrifyTime =
                            (*addr_of!(level)).time + Q_irand(300, 800);
                    }
                }
                if (*(*gent).client).ps.powerups[PW_CLOAKED as usize] != 0 {
                    //disable cloak temporarily
                    Jedi_Decloak(gent);
                    (*(*gent).client).cloakToggleTime =
                        (*addr_of!(level)).time + Q_irand(3000, 10000);
                }
            }
        }
        e += 1;
    }

    // store the last fraction so that next time around we can test against those things that fall between that last point and where the current shockwave edge is
    (*ent).genericValue6 = radius as c_int;

    if frac < 1.0 {
        // shock is still happening so continue letting it expand
        (*ent).nextthink = (*addr_of!(level)).time + 50;
    } else {
        //don't just leave the entity around
        (*ent).think = Some(G_FreeEntity);
        (*ent).nextthink = (*addr_of!(level)).time;
    }
}

/// `void DEMP2_AltDetonate( gentity_t *ent )` (g_weapon.c:1304).
///
/// The think the alt-DEMP2 impact missile uses on landing: pins itself at its current origin,
/// plays the combined explosion+sphere-spawn effect along its stored impact normal (`pos1`,
/// defaulted to `+Y` if zeroed so the effect has a direction), seeds the shockwave timing
/// (`genericValue5`/`genericValue6`), then becomes a plain `ET_GENERAL` entity whose think is
/// [`DEMP2_AltRadiusDamage`] firing next tick. The spawned effect entity's `s.weapon` carries
/// `count*2` so the fx scales with the charge level. No oracle (entity-state think + effect spawn).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `level` must be initialised. A `pub unsafe extern "C"`
/// fn for the `gentity_t::think` fn-pointer ABI.
pub unsafe extern "C" fn DEMP2_AltDetonate(ent: *mut gentity_t) {
    let efEnt: *mut gentity_t;

    G_SetOrigin(ent, &(*ent).r.currentOrigin);
    if (*ent).pos1[0] == 0.0 && (*ent).pos1[1] == 0.0 && (*ent).pos1[2] == 0.0 {
        //don't play effect with a 0'd out directional vector
        (*ent).pos1[1] = 1.0;
    }
    //Let's just save ourself some bandwidth and play both the effect and sphere spawn in 1 event
    efEnt = G_PlayEffect(
        EFFECT_EXPLOSION_DEMP2ALT,
        &(*ent).r.currentOrigin,
        &(*ent).pos1,
    );

    if !efEnt.is_null() {
        (*efEnt).s.weapon = (*ent).count * 2;
    }

    (*ent).genericValue5 = (*addr_of!(level)).time;
    (*ent).genericValue6 = 0;
    (*ent).nextthink = (*addr_of!(level)).time + 50;
    (*ent).think = Some(DEMP2_AltRadiusDamage);
    (*ent).s.eType = ET_GENERAL; // make us a missile no longer
}

/// `static void WP_DEMP2_AltFire( gentity_t *ent )` (g_weapon.c:1330).
///
/// The alt-DEMP2 fire: trace a `DEMP2_ALT_RANGE` ray from the file-static `muzzle` along
/// `forward`, then spawn the impact missile at the trace endpoint (instant, unlike SP's travelling
/// missile). Charge level (`count`, 1..3) scales the damage by `count*0.8` (floored at 1×); a pure
/// tap-fire (`origcount == 0`) is forced to 1 damage. The missile stores the impact normal in
/// `pos1`, is keyed to [`DEMP2_AltDetonate`] firing next tick, and is set never to bounce. No
/// oracle (file-static aim + `trap_Trace` + gentity spawn).
///
/// # Safety
/// `ent` must be valid with a non-NULL `client`; reads the file-statics `forward`/`muzzle`.
pub unsafe fn WP_DEMP2_AltFire(ent: *mut gentity_t) {
    let mut damage = DEMP2_ALT_DAMAGE;
    let mut count: c_int;
    let origcount: c_int;
    let mut fact: f32;
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];

    VectorCopy(&*addr_of!(muzzle), &mut start);

    VectorMA(&start, DEMP2_ALT_RANGE, &*addr_of!(forward), &mut end);

    count = ((*addr_of!(level)).time - (*(*ent).client).ps.weaponChargeTime) / DEMP2_CHARGE_UNIT as c_int;

    origcount = count;

    if count < 1 {
        count = 1;
    } else if count > 3 {
        count = 3;
    }

    fact = count as f32 * 0.8;
    if fact < 1.0 {
        fact = 1.0;
    }
    damage = (damage as f32 * fact) as c_int;

    if origcount == 0 {
        //this was just a tap-fire
        damage = 1;
    }

    let tr = trap::Trace(
        &start,
        &vec3_origin,
        &vec3_origin,
        &end,
        (*ent).s.number,
        MASK_SHOT,
    );

    let missile = G_Spawn();
    G_SetOrigin(missile, &tr.endpos);
    //In SP the impact actually travels as a missile based on the trace fraction, but we're
    //just going to be instant. -rww

    VectorCopy(&tr.plane.normal, &mut (*missile).pos1);

    (*missile).count = count;

    (*missile).classname = c"demp2_alt_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_DEMP2;

    (*missile).think = Some(DEMP2_AltDetonate);
    (*missile).nextthink = (*addr_of!(level)).time;

    (*missile).damage = damage;
    (*missile).splashDamage = (*missile).damage;
    (*missile).methodOfDeath = MOD_DEMP2;
    (*missile).splashMethodOfDeath = (*missile).methodOfDeath;
    (*missile).splashRadius = DEMP2_ALT_SPLASHRADIUS;

    (*missile).r.ownerNum = (*ent).s.number;

    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to ever bounce
    (*missile).bounceCount = 0;
}

/// `static void WP_FireDEMP2( gentity_t *ent, qboolean altFire )` (g_weapon.c:1400).
///
/// DEMP2 fire dispatcher: routes to [`WP_DEMP2_AltFire`] (the ion shockwave) or
/// [`WP_DEMP2_MainFire`] (the straight bolt) on `altFire`. No oracle (gentity dispatch).
///
/// # Safety
/// `ent` must be valid; forwards to the two fire paths' invariants.
pub unsafe fn WP_FireDEMP2(ent: *mut gentity_t, altFire: qboolean) {
    if altFire != QFALSE {
        WP_DEMP2_AltFire(ent);
    } else {
        WP_DEMP2_MainFire(ent);
    }
}

/// `void WP_FireStunBaton( gentity_t *ent, qboolean alt_fire )` (g_weapon.c:3241).
///
/// Box-trace forward from the eye (offset 20u forward, 4u right) over `STUN_BATON_RANGE`;
/// on a damageable hit, play the stun effect/punch sound, deal `STUN_BATON_DAMAGE`, and
/// electrify a hit player (shorter window on non-animal/non-flier vehicles). Reads the
/// file-statics `forward`/`vright`. No oracle (gentity + `trap_Trace`).
///
/// # Safety
/// `ent` must be valid; `ent->client` is NULL-checked.
pub unsafe fn WP_FireStunBaton(ent: *mut gentity_t, _alt_fire: qboolean) {
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut muzzleStun: vec3_t = [0.0; 3];

    if (*ent).client.is_null() {
        VectorCopy(&(*ent).r.currentOrigin, &mut muzzleStun);
        muzzleStun[2] += 8.0;
    } else {
        VectorCopy(&(*(*ent).client).ps.origin, &mut muzzleStun);
        muzzleStun[2] += ((*(*ent).client).ps.viewheight - 6) as f32;
    }

    let muzzleStunCopy = muzzleStun;
    VectorMA(&muzzleStunCopy, 20.0, &*addr_of!(forward), &mut muzzleStun);
    let muzzleStunCopy = muzzleStun;
    VectorMA(&muzzleStunCopy, 4.0, &*addr_of!(vright), &mut muzzleStun);

    VectorMA(&muzzleStun, STUN_BATON_RANGE, &*addr_of!(forward), &mut end);

    VectorSet(&mut maxs, 6.0, 6.0, 6.0);
    VectorScale(&maxs, -1.0, &mut mins);

    let tr = trap::Trace(
        &muzzleStun,
        &mins,
        &maxs,
        &end,
        (*ent).s.number,
        MASK_SHOT,
    );

    if tr.entityNum as c_int >= ENTITYNUM_WORLD {
        return;
    }

    let tr_ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

    if !tr_ent.is_null() && (*tr_ent).takedamage != QFALSE && !(*tr_ent).client.is_null() {
        // see if either party is involved in a duel
        if (*(*tr_ent).client).ps.duelInProgress != QFALSE
            && (*(*tr_ent).client).ps.duelIndex != (*ent).s.number
        {
            return;
        }

        if !(*ent).client.is_null()
            && (*(*ent).client).ps.duelInProgress != QFALSE
            && (*(*ent).client).ps.duelIndex != (*tr_ent).s.number
        {
            return;
        }
    }

    if !tr_ent.is_null() && (*tr_ent).takedamage != QFALSE {
        G_PlayEffect(EFFECT_STUNHIT, &tr.endpos, &tr.plane.normal);

        G_Sound(
            tr_ent,
            CHAN_WEAPON,
            G_SoundIndex(&format!("sound/weapons/melee/punch{}", Q_irand(1, 4))),
        );
        G_Damage(
            tr_ent,
            ent,
            ent,
            addr_of_mut!(forward),
            &mut { tr.endpos } as *mut vec3_t,
            STUN_BATON_DAMAGE,
            DAMAGE_NO_KNOCKBACK | DAMAGE_HALF_ABSORB,
            MOD_STUN_BATON,
        );

        if !(*tr_ent).client.is_null() {
            // if it's a player then use the shock effect
            if (*(*tr_ent).client).NPC_class == CLASS_VEHICLE {
                // not on vehicles
                if (*tr_ent).m_pVehicle.is_null()
                    || (*(*(*tr_ent).m_pVehicle).m_pVehicleInfo).r#type == VH_ANIMAL
                    || (*(*(*tr_ent).m_pVehicle).m_pVehicleInfo).r#type == VH_FLIER
                {
                    // can zap animals
                    (*(*tr_ent).client).ps.electrifyTime =
                        (*addr_of!(level)).time + Q_irand(3000, 4000);
                }
            } else {
                (*(*tr_ent).client).ps.electrifyTime = (*addr_of!(level)).time + 700;
            }
        }
    }
}

/// `void WP_FireMelee( gentity_t *ent, qboolean alt_fire )` (g_weapon.c:3322).
///
/// Box-trace forward over `MELEE_RANGE`; on any hit play a punch sound and, if the target
/// is damageable (and not the wrong side of a duel), deal swing damage — more on the second
/// (right-hook) swing, doubled for the heavy-melee class. Bails early if the swinging arm is
/// a broken limb. Reads the file-statics `forward`/`vright`. No oracle (gentity + `trap_Trace`).
///
/// # Safety
/// `ent` must be valid; the C derefs `ent->client` unconditionally for the broken-limb check,
/// so `ent->client` must be non-NULL.
pub unsafe fn WP_FireMelee(ent: *mut gentity_t, _alt_fire: qboolean) {
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut muzzlePunch: vec3_t = [0.0; 3];

    if !(*ent).client.is_null() && (*(*ent).client).ps.torsoAnim == BOTH_MELEE2 {
        // right
        if (*(*ent).client).ps.brokenLimbs & (1 << BROKENLIMB_RARM) != 0 {
            return;
        }
    } else {
        // left
        if (*(*ent).client).ps.brokenLimbs & (1 << BROKENLIMB_LARM) != 0 {
            return;
        }
    }

    if (*ent).client.is_null() {
        VectorCopy(&(*ent).r.currentOrigin, &mut muzzlePunch);
        muzzlePunch[2] += 8.0;
    } else {
        VectorCopy(&(*(*ent).client).ps.origin, &mut muzzlePunch);
        muzzlePunch[2] += ((*(*ent).client).ps.viewheight - 6) as f32;
    }

    let muzzlePunchCopy = muzzlePunch;
    VectorMA(&muzzlePunchCopy, 20.0, &*addr_of!(forward), &mut muzzlePunch);
    let muzzlePunchCopy = muzzlePunch;
    VectorMA(&muzzlePunchCopy, 4.0, &*addr_of!(vright), &mut muzzlePunch);

    VectorMA(&muzzlePunch, MELEE_RANGE, &*addr_of!(forward), &mut end);

    VectorSet(&mut maxs, 6.0, 6.0, 6.0);
    VectorScale(&maxs, -1.0, &mut mins);

    let tr = trap::Trace(
        &muzzlePunch,
        &mins,
        &maxs,
        &end,
        (*ent).s.number,
        MASK_SHOT,
    );

    if tr.entityNum as c_int != ENTITYNUM_NONE {
        // hit something
        let tr_ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

        G_Sound(
            ent,
            CHAN_AUTO,
            G_SoundIndex(&format!("sound/weapons/melee/punch{}", Q_irand(1, 4))),
        );

        if (*tr_ent).takedamage != QFALSE && !(*tr_ent).client.is_null() {
            // special duel checks
            if (*(*tr_ent).client).ps.duelInProgress != QFALSE
                && (*(*tr_ent).client).ps.duelIndex != (*ent).s.number
            {
                return;
            }

            if !(*ent).client.is_null()
                && (*(*ent).client).ps.duelInProgress != QFALSE
                && (*(*ent).client).ps.duelIndex != (*tr_ent).s.number
            {
                return;
            }
        }

        if (*tr_ent).takedamage != QFALSE {
            // damage them, do more damage if we're in the second right hook
            let mut dmg = MELEE_SWING1_DAMAGE;

            if !(*ent).client.is_null() && (*(*ent).client).ps.torsoAnim == BOTH_MELEE2 {
                // do a tad bit more damage on the second swing
                dmg = MELEE_SWING2_DAMAGE;
            }

            if G_HeavyMelee(ent) != QFALSE {
                // 2x damage for heavy melee class
                dmg *= 2;
            }

            G_Damage(
                tr_ent,
                ent,
                ent,
                addr_of_mut!(forward),
                &mut { tr.endpos } as *mut vec3_t,
                dmg,
                DAMAGE_NO_ARMOR,
                MOD_MELEE,
            );
        }
    }
}

/// `static void WP_FireConcussionAlt( gentity_t *ent )` (g_weapon.c:2929) — the concussion
/// rifle's alt-fire: a rail-gun-like instant beam. Shoves the shooter backwards, then runs up to
/// `DISRUPTOR_ALT_TRACES` traces forward, damaging each takedamage entity it passes through and
/// manually knocking down humanoid clients (distance-scaled), stopping on movers/solids. Packs the
/// final impact direction/origin into one `EV_CONC_ALT_IMPACT` temp-entity. No oracle
/// (trace/G_Damage side-effects + client playerState mutation; reads file-static `muzzle`/`forward`).
///
/// # Safety
/// `ent` and `ent->client` must be valid; reads the file-statics `muzzle`/`forward`. `level` and
/// `g_entities` must be initialised.
pub unsafe fn WP_FireConcussionAlt(ent: *mut gentity_t) {
    // a rail-gun-like beam
    let damage: c_int = CONC_ALT_DAMAGE;
    let mut skip: c_int;
    let traces: c_int = DISRUPTOR_ALT_TRACES;
    let mut render_impact: qboolean = QTRUE;
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut muzzle2: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut tr: trace_t;
    let mut traceEnt: *mut gentity_t;
    let tent: *mut gentity_t;
    let shotRange: f32 = 8192.0;
    let mut hitDodged: qboolean = QFALSE;
    let mut shot_mins: vec3_t = [0.0; 3];
    let mut shot_maxs: vec3_t = [0.0; 3];
    let mut i: c_int;

    //Shove us backwards for half a second
    let velCopy = (*(*ent).client).ps.velocity;
    VectorMA(
        &velCopy,
        -200.0,
        &*addr_of!(forward),
        &mut (*(*ent).client).ps.velocity,
    );
    (*(*ent).client).ps.groundEntityNum = ENTITYNUM_NONE;
    if (*(*ent).client).ps.pm_flags & PMF_DUCKED != 0 {
        //hunkered down
        (*(*ent).client).ps.pm_time = 100;
    } else {
        (*(*ent).client).ps.pm_time = 250;
    }
    //	ent->client->ps.pm_flags |= PMF_TIME_KNOCKBACK|PMF_TIME_NOFRICTION;
    //FIXME: only if on ground?  So no "rocket jump"?  Or: (see next FIXME)
    //FIXME: instead, set a forced ucmd backmove instead of this sliding

    VectorCopy(&*addr_of!(muzzle), &mut muzzle2); // making a backup copy

    VectorCopy(&*addr_of!(muzzle), &mut start);
    WP_TraceSetStart(ent, &mut start, &vec3_origin, &vec3_origin);

    skip = (*ent).s.number;

    //	if ( ent->client && ent->client->ps.powerups[PW_WEAPON_OVERCHARGE] > 0 && ent->client->ps.powerups[PW_WEAPON_OVERCHARGE] > cg.time )
    //	{
    //		// in overcharge mode, so doing double damage
    //		damage *= 2;
    //	}

    //Make it a little easier to hit guys at long range
    VectorSet(&mut shot_mins, -1.0, -1.0, -1.0);
    VectorSet(&mut shot_maxs, 1.0, 1.0, 1.0);

    tr = trace_t::default();

    i = 0;
    while i < traces {
        let startCopy = start;
        VectorMA(&startCopy, shotRange, &*addr_of!(forward), &mut end);

        //NOTE: if you want to be able to hit guys in emplaced guns, use "G2_COLLIDE, 10" instead of "G2_RETURNONHIT, 0"
        //alternately, if you end up hitting an emplaced_gun that has a sitter, just redo this one trace with the "G2_COLLIDE, 10" to see if we it the sitter
        //gi.trace( &tr, start, NULL, NULL, end, skip, MASK_SHOT, G2_COLLIDE, 10 );//G2_RETURNONHIT, 0 );
        if (*addr_of!(d_projectileGhoul2Collision)).integer != 0 {
            tr = trap::G2Trace(
                &start,
                &shot_mins,
                &shot_maxs,
                &end,
                skip,
                MASK_SHOT,
                G2TRFLAG_DOGHOULTRACE | G2TRFLAG_GETSURFINDEX | G2TRFLAG_HITCORPSES,
                (*addr_of!(g_g2TraceLod)).integer,
            );
        } else {
            tr = trap::Trace(&start, &shot_mins, &shot_maxs, &end, skip, MASK_SHOT);
        }

        traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

        if (*addr_of!(d_projectileGhoul2Collision)).integer != 0
            && (*traceEnt).inuse != QFALSE
            && !(*traceEnt).client.is_null()
        {
            //g2 collision checks -rww
            if (*traceEnt).inuse != QFALSE
                && !(*traceEnt).client.is_null()
                && !(*traceEnt).ghoul2.is_null()
            {
                //since we used G2TRFLAG_GETSURFINDEX, tr.surfaceFlags will actually contain the index of the surface on the ghoul2 model we collided with.
                (*(*traceEnt).client).g2LastSurfaceHit = tr.surfaceFlags;
                (*(*traceEnt).client).g2LastSurfaceTime = (*addr_of!(level)).time;
            }

            if !(*traceEnt).ghoul2.is_null() {
                tr.surfaceFlags = 0; //clear the surface flags after, since we actually care about them in here.
            }
        }
        if (tr.surfaceFlags & SURF_NOIMPACT) != 0 {
            render_impact = QFALSE;
        }

        if tr.entityNum as c_int == (*ent).s.number {
            // should never happen, but basically we don't want to consider a hit to ourselves?
            // Get ready for an attempt to trace through another person
            VectorCopy(&tr.endpos, &mut muzzle2);
            VectorCopy(&tr.endpos, &mut start);
            skip = tr.entityNum as c_int;
            // #ifdef _DEBUG
            // Com_Printf( "BAD! Concussion gun shot somehow traced back and hit the owner!\n" );
            // #endif
            i += 1;
            continue;
        }

        // always render a shot beam, doing this the old way because I don't much feel like overriding the effect.
        //NOTE: let's just draw one beam at the end
        //tent = G_TempEntity( tr.endpos, EV_CONC_ALT_SHOT );
        //tent->svFlags |= SVF_BROADCAST;

        //VectorCopy( muzzle2, tent->s.origin2 );

        if tr.fraction >= 1.0 {
            // draw the beam but don't do anything else
            break;
        }

        if (*traceEnt).s.weapon == WP_SABER
        //&& traceEnt->NPC
        {
            //FIXME: need a more reliable way to know we hit a jedi?
            hitDodged = Jedi_DodgeEvasion(traceEnt, ent, &mut tr, HL_NONE);
            //acts like we didn't even hit him
        }
        if hitDodged == QFALSE {
            if render_impact != QFALSE {
                if ((tr.entityNum as c_int) < ENTITYNUM_WORLD && (*traceEnt).takedamage != QFALSE)
                    || Q_stricmp(
                        (*traceEnt).classname,
                        c"misc_model_breakable".as_ptr(),
                    ) == 0
                    || (*traceEnt).s.eType == ET_MOVER
                {
                    let noKnockBack: qboolean;

                    // Create a simple impact type mark that doesn't last long in the world
                    //G_PlayEffectID( G_EffectIndex( "concussion/alt_hit" ), tr.endpos, tr.plane.normal );
                    //no no no

                    if !(*traceEnt).client.is_null() && LogAccuracyHit(traceEnt, ent) != QFALSE {
                        //NOTE: hitting multiple ents can still get you over 100% accuracy
                        (*(*ent).client).accuracy_hits += 1;
                    }

                    noKnockBack = ((*traceEnt).flags & FL_NO_KNOCKBACK) as qboolean; //will be set if they die, I want to know if it was on *before* they died
                    if !traceEnt.is_null()
                        && !(*traceEnt).client.is_null()
                        && (*(*traceEnt).client).NPC_class == CLASS_GALAKMECH
                    {
                        //hehe
                        G_Damage(
                            traceEnt,
                            ent,
                            ent,
                            addr_of_mut!(forward),
                            &mut tr.endpos as *mut vec3_t,
                            10,
                            DAMAGE_NO_KNOCKBACK | DAMAGE_NO_HIT_LOC,
                            MOD_CONC_ALT,
                        );
                        break;
                    }
                    G_Damage(
                        traceEnt,
                        ent,
                        ent,
                        addr_of_mut!(forward),
                        &mut tr.endpos as *mut vec3_t,
                        damage,
                        DAMAGE_NO_KNOCKBACK | DAMAGE_NO_HIT_LOC,
                        MOD_CONC_ALT,
                    );

                    //do knockback and knockdown manually
                    if !(*traceEnt).client.is_null() {
                        //only if we hit a client
                        let mut pushDir: vec3_t = [0.0; 3];
                        VectorCopy(&*addr_of!(forward), &mut pushDir);
                        if pushDir[2] < 0.2 {
                            pushDir[2] = 0.2;
                        } //hmm, re-normalize?  nah...
                          /*
                          if ( !noKnockBack )
                          {//knock-backable
                          	G_Throw( traceEnt, pushDir, 200 );
                          }
                          */
                        if (*traceEnt).health > 0 {
                            //alive
                            //if ( G_HasKnockdownAnims( traceEnt ) )
                            if noKnockBack == QFALSE
                                && (*traceEnt).localAnimIndex == 0
                                && (*(*traceEnt).client).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN
                                && BG_KnockDownable(&mut (*(*traceEnt).client).ps) != QFALSE
                            {
                                //just check for humanoids..
                                //knock-downable
                                //G_Knockdown( traceEnt, ent, pushDir, 400, qtrue );
                                let mut plPDif: vec3_t = [0.0; 3];
                                let mut pStr: f32;

                                //cap it and stuff, base the strength and whether or not we can knockdown on the distance
                                //from the shooter to the target
                                VectorSubtract(
                                    &(*(*traceEnt).client).ps.origin,
                                    &(*(*ent).client).ps.origin,
                                    &mut plPDif,
                                );
                                pStr = 500.0 - VectorLength(&plPDif);
                                if pStr < 150.0 {
                                    pStr = 150.0;
                                }
                                if pStr > 200.0 {
                                    (*(*traceEnt).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
                                    (*(*traceEnt).client).ps.forceHandExtendTime =
                                        (*addr_of!(level)).time + 1100;
                                    (*(*traceEnt).client).ps.forceDodgeAnim = 0; //this toggles between 1 and 0, when it's 1 we should play the get up anim
                                }
                                (*(*traceEnt).client).ps.otherKiller = (*ent).s.number;
                                (*(*traceEnt).client).ps.otherKillerTime =
                                    (*addr_of!(level)).time + 5000;
                                (*(*traceEnt).client).ps.otherKillerDebounceTime =
                                    (*addr_of!(level)).time + 100;
                                (*(*traceEnt).client).otherKillerMOD = MOD_UNKNOWN;
                                (*(*traceEnt).client).otherKillerVehWeapon = 0;
                                (*(*traceEnt).client).otherKillerWeaponType = WP_NONE;

                                (*(*traceEnt).client).ps.velocity[0] += pushDir[0] * pStr;
                                (*(*traceEnt).client).ps.velocity[1] += pushDir[1] * pStr;
                                (*(*traceEnt).client).ps.velocity[2] = pStr;
                            }
                        }
                    }

                    if (*traceEnt).s.eType == ET_MOVER {
                        //stop the traces on any mover
                        break;
                    }
                } else {
                    // we only make this mark on things that can't break or move
                    //	tent = G_TempEntity(tr.endpos, EV_MISSILE_MISS);
                    //	tent->s.eventParm = DirToByte(tr.plane.normal);
                    //	tent->s.eFlags |= EF_ALT_FIRING;

                    //tent->svFlags |= SVF_BROADCAST;
                    //eh? why broadcast?
                    //	VectorCopy( tr.plane.normal, tent->pos1 );

                    //mmm..no..don't do this more than once for no reason whatsoever.
                    break; // hit solid, but doesn't take damage, so stop the shot...we _could_ allow it to shoot through walls, might be cool?
                }
            } else
            // not rendering impact, must be a skybox or other similar thing?
            {
                break; // don't try anymore traces
            }
        }
        // Get ready for an attempt to trace through another person
        VectorCopy(&tr.endpos, &mut muzzle2);
        VectorCopy(&tr.endpos, &mut start);
        skip = tr.entityNum as c_int;
        hitDodged = QFALSE;

        i += 1;
    }
    //just draw one beam all the way to the end
    //	tent = G_TempEntity( tr.endpos, EV_CONC_ALT_SHOT );
    //	tent->svFlags |= SVF_BROADCAST;
    //again, why broadcast?

    //	tent = G_TempEntity(tr.endpos, EV_MISSILE_MISS);
    //	tent->s.eventParm = DirToByte(tr.plane.normal);
    //	tent->s.eFlags |= EF_ALT_FIRING;
    //	VectorCopy( muzzle, tent->s.origin2 );

    // now go along the trail and make sight events
    VectorSubtract(&tr.endpos, &*addr_of!(muzzle), &mut dir);

    //	shotDist = VectorNormalize( dir );

    //let's pack all this junk into a single tempent, and send it off.
    tent = G_TempEntity(&tr.endpos, EV_CONC_ALT_IMPACT);
    (*tent).s.eventParm = DirToByte(&tr.plane.normal);
    (*tent).s.owner = (*ent).s.number;
    VectorCopy(&dir, &mut (*tent).s.angles);
    VectorCopy(&*addr_of!(muzzle), &mut (*tent).s.origin2);
    VectorCopy(&*addr_of!(forward), &mut (*tent).s.angles2);

    // #if 0 //yuck
    //	//FIXME: if shoot *really* close to someone, the alert could be way out of their FOV
    //	for ( dist = 0; dist < shotDist; dist += 64 )
    //	{
    //		//FIXME: on a really long shot, this could make a LOT of alerts in one frame...
    //		VectorMA( muzzle, dist, dir, spot );
    //		AddSightEvent( ent, spot, 256, AEL_DISCOVERED, 50 );
    //		//FIXME: creates *way* too many effects, make it one effect somehow?
    //		G_PlayEffectID( G_EffectIndex( "concussion/alt_ring" ), spot, actualAngles );
    //	}
    //	//FIXME: spawn a temp ent that continuously spawns sight alerts here?  And 1 sound alert to draw their attention?
    //	VectorMA( start, shotDist-4, forward, spot );
    //	AddSightEvent( ent, spot, 256, AEL_DISCOVERED, 50 );
    //
    //	G_PlayEffectID( G_EffectIndex( "concussion/altmuzzle_flash" ), muzzle, forward );
    // #endif

    // _ = muzzle2 — the C keeps a `muzzle2` backup used only by the commented-out beam tempents.
    let _ = muzzle2;
}

/// `static void WP_FireConcussion( gentity_t *ent )` (g_weapon.c:3191).
///
/// The concussion rifle's primary fire: a fast rocket-like projectile. Snap the muzzle back
/// inside any wall, spawn a non-bouncing `conc_proj` with an enlarged bbox and big splash,
/// flagged for extra knockback. No oracle (gentity spawn + file-static aim state).
///
/// # Safety
/// `ent` and `ent->client` must be valid; reads the file-static `muzzle`/`forward`.
pub unsafe fn WP_FireConcussion(ent: *mut gentity_t) {
    // a fast rocket-like projectile
    let damage = CONC_DAMAGE;
    let vel = CONC_VELOCITY;

    let mut start: vec3_t = [0.0; 3];
    VectorCopy(&*addr_of!(muzzle), &mut start);
    // make sure our start point isn't on the other side of a wall
    WP_TraceSetStart(ent, &mut start, &[0.0; 3], &[0.0; 3]);

    let missile = CreateMissile(&mut start, &*addr_of!(forward), vel, 10000, ent, QFALSE);

    (*missile).classname = c"conc_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_CONCUSSION;
    (*missile).mass = 10.0;

    // Make it easier to hit things
    VectorSet(&mut (*missile).r.maxs, ROCKET_SIZE, ROCKET_SIZE, ROCKET_SIZE);
    VectorScale(&{ (*missile).r.maxs }, -1.0, &mut (*missile).r.mins);

    (*missile).damage = damage;
    (*missile).dflags = DAMAGE_EXTRA_KNOCKBACK;

    (*missile).methodOfDeath = MOD_CONC;
    (*missile).splashMethodOfDeath = MOD_CONC;

    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    (*missile).splashDamage = CONC_SPLASH_DAMAGE;
    (*missile).splashRadius = CONC_SPLASH_RADIUS;

    // we don't want it to ever bounce
    (*missile).bounceCount = 0;
}

/// `qboolean WP_LobFire( gentity_t *self, vec3_t start, vec3_t target, vec3_t mins,
/// vec3_t maxs, int clipmask, vec3_t velocity, qboolean tracePath, int ignoreEntNum,
/// int enemyNum, float minSpeed, float maxSpeed, float idealSpeed, qboolean mustHit )`
/// (g_weapon.c:2044).
///
/// Solve a ballistic lob: pick a launch velocity (under `g_gravity`) that arcs from `start`
/// to `target`. Used by the galak-mech NPC. Starting at `idealSpeed`, it optionally
/// roughly traces the parabola (sampling every 500ms with `trap_Trace`); if blocked it
/// nudges the speed (changing the arc) and retries up to 7 times, tracking the best-miss
/// `failCase`. Returns `qtrue` with a clear `velocity`, else `qfalse` with the closest miss.
/// No oracle (gentity + `trap_Trace` + globals `level`/`g_gravity`/`g_entities`).
///
/// # Safety
/// `self` must be valid; `g_entities` must be initialised (only indexed for hit ents).
pub unsafe fn WP_LobFire(
    self_: *mut gentity_t,
    start: &vec3_t,
    target: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    clipmask: c_int,
    velocity: &mut vec3_t,
    tracePath: qboolean,
    ignoreEntNum: c_int,
    enemyNum: c_int,
    minSpeed: f32,
    maxSpeed: f32,
    idealSpeed: f32,
    mustHit: qboolean,
) -> qboolean {
    //for the galak mech NPC
    let mut idealSpeed = idealSpeed;
    let mut minSpeed = minSpeed;
    let mut maxSpeed = maxSpeed;
    let speedInc: f32 = 100.0;
    let mut bestImpactDist: f32 = Q3_INFINITE as f32; //fireSpeed,
    let mut shotVel: vec3_t = [0.0; 3];
    let mut failCase: vec3_t = [0.0; 3];
    let mut tr = trajectory_t::default();
    let timeStep: c_int = 500;
    let mut hitCount: c_int = 0;
    let maxHits: c_int = 7;
    let mut lastPos: vec3_t = [0.0; 3];
    let mut testPos: vec3_t = [0.0; 3];

    if idealSpeed == 0.0 {
        idealSpeed = 300.0;
    } else if idealSpeed < speedInc {
        idealSpeed = speedInc;
    }
    let mut shotSpeed = idealSpeed;
    let skipNum = ((idealSpeed - speedInc) / speedInc) as c_int;
    if minSpeed == 0.0 {
        minSpeed = 100.0;
    }
    if maxSpeed == 0.0 {
        maxSpeed = 900.0;
    }
    let _ = (minSpeed, maxSpeed);
    while hitCount < maxHits {
        let mut targetDir: vec3_t = [0.0; 3];
        VectorSubtract(target, start, &mut targetDir);
        let targetDist = VectorNormalize(&mut targetDir);

        VectorScale(&targetDir, shotSpeed, &mut shotVel);
        let mut travelTime = targetDist / shotSpeed;
        shotVel[2] += travelTime * 0.5 * (*addr_of!(g_gravity)).value;

        if hitCount == 0 {
            //save the first (ideal) one as the failCase (fallback value)
            if mustHit == QFALSE {
                //default is fine as a return value
                VectorCopy(&shotVel, &mut failCase);
            }
        }

        if tracePath != QFALSE {
            //do a rough trace of the path
            let mut blocked = QFALSE;

            VectorCopy(start, &mut tr.trBase);
            VectorCopy(&shotVel, &mut tr.trDelta);
            tr.trType = TR_GRAVITY;
            tr.trTime = (*addr_of!(level)).time;
            travelTime *= 1000.0;
            VectorCopy(start, &mut lastPos);

            //This may be kind of wasteful, especially on long throws... use larger steps?  Divide the travelTime into a certain hard number of slices?  Trace just to apex and down?
            let mut elapsedTime = timeStep;
            while (elapsedTime as f32) < travelTime.floor() + timeStep as f32 {
                if elapsedTime as f32 > travelTime {
                    //cap it
                    elapsedTime = travelTime.floor() as c_int;
                }
                BG_EvaluateTrajectory(&tr, (*addr_of!(level)).time + elapsedTime, &mut testPos);
                let trace = trap::Trace(&lastPos, mins, maxs, &testPos, ignoreEntNum, clipmask);

                if trace.allsolid != 0 || trace.startsolid != 0 {
                    blocked = QTRUE;
                    break;
                }
                if trace.fraction < 1.0 {
                    //hit something
                    if trace.entityNum as c_int == enemyNum {
                        //hit the enemy, that's perfect!
                        break;
                    } else if trace.plane.normal[2] > 0.7
                        && DistanceSquared(&trace.endpos, target) < 4096.0
                    //hit within 64 of desired location, should be okay
                    {
                        //close enough!
                        break;
                    } else {
                        //FIXME: maybe find the extents of this brush and go above or below it on next try somehow?
                        let impactDist = DistanceSquared(&trace.endpos, target);
                        if impactDist < bestImpactDist {
                            bestImpactDist = impactDist;
                            VectorCopy(&shotVel, &mut failCase);
                        }
                        blocked = QTRUE;
                        //see if we should store this as the failCase
                        if (trace.entityNum as c_int) < ENTITYNUM_WORLD {
                            //hit an ent
                            let traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(trace.entityNum as usize);
                            if !traceEnt.is_null()
                                && (*traceEnt).takedamage != QFALSE
                                && OnSameTeam(self_, traceEnt) == QFALSE
                            {
                                //hit something breakable, so that's okay
                                //we haven't found a clear shot yet so use this as the failcase
                                VectorCopy(&shotVel, &mut failCase);
                            }
                        }
                        break;
                    }
                }
                if elapsedTime == travelTime.floor() as c_int {
                    //reached end, all clear
                    break;
                } else {
                    //all clear, try next slice
                    VectorCopy(&testPos, &mut lastPos);
                }
                elapsedTime += timeStep;
            }
            if blocked != QFALSE {
                //hit something, adjust speed (which will change arc)
                hitCount += 1;
                shotSpeed = idealSpeed + ((hitCount - skipNum) as f32 * speedInc); //from min to max (skipping ideal)
                if hitCount >= skipNum {
                    //skip ideal since that was the first value we tested
                    shotSpeed += speedInc;
                }
            } else {
                //made it!
                break;
            }
        } else {
            //no need to check the path, go with first calc
            break;
        }
    }

    if hitCount >= maxHits {
        //NOTE: worst case scenario, use the one that impacted closest to the target (or just use the first try...?)
        VectorCopy(&failCase, velocity);
        return QFALSE;
    }
    VectorCopy(&shotVel, velocity);
    QTRUE
}

/// `void emplaced_gun_pain( gentity_t *self, gentity_t *attacker, int damage )`
/// (g_weapon.c:4736) — the emplaced-gun `pain` callback. Mirrors `health` into
/// `s.health` (so the clientside sees the damage), then runs the `BSET_PAIN`
/// behaviour set unless the gun has just died (the death effect is handled cgame-side).
/// The `attacker`/`damage` params go unused here, matching the C signature.
pub unsafe extern "C" fn emplaced_gun_pain(
    self_: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
) {
    (*self_).s.health = (*self_).health;

    if (*self_).health <= 0 {
        //death effect.. for now taken care of on cgame
    } else {
        //if we have a pain behavior set then use it I guess
        G_ActivateBehavior(self_, BSET_PAIN);
    }
}

/// `void emplaced_gun_die( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int mod )`
/// (g_weapon.c:4856) — the emplaced-gun `die` callback: "set us up to flash and then
/// explode". Guards against re-entry via `genericValue4`, then arms the 3-second
/// flash-then-explode timer in `s.time` and clears the respawn delay (`genericValue5`).
/// All four trailing params (`inflictor`/`attacker`/`damage`/`mod`) are unused, matching C.
pub unsafe extern "C" fn emplaced_gun_die(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    _mod_: c_int,
) {
    //set us up to flash and then explode
    if (*self_).genericValue4 != 0 {
        return;
    }

    (*self_).genericValue4 = 1;

    (*self_).s.time = (*addr_of!(level)).time + 3000;

    (*self_).genericValue5 = 0;
}

/// `void emplaced_gun_use( gentity_t *self, gentity_t *other, trace_t *trace )`
/// (g_weapon.c:4617) — the emplaced-gun `use` callback (installed via `touch`-shaped
/// fn-pointer through [`emplaced_gun_realuse`]). Tries to mount `other` onto the gun:
/// bails if the gun is destroyed, already in use, the user isn't a client, the user's
/// last mount attempt was too recent, the user is busy (`forceHandExtend`), is standing
/// on top of the gun, is ducked, or is more than 64 units away. Then it checks the user
/// is facing roughly the way the gun points (and is positioned in front of it); if not,
/// it falls through to [`TryHeal`] instead. On a successful mount it swaps the user's
/// weapon for `WP_EMPLACED_GUN`, records the old weapon on the gun, reparents the user to
/// the gun, and stores the gun as the user's owner. No oracle (gentity / playerState
/// mutation + `TryHeal` side-effect; the geometry math via `AngleVectors`/`DotProduct`
/// is already oracled).
///
/// # Safety
/// `self_` must point to a valid emplaced-gun `gentity_t`; `other` is NULL-/client-checked
/// before use. `trace` is unused. `level` must be initialised.
pub unsafe extern "C" fn emplaced_gun_use(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    let mut fwd1: vec3_t = [0.0; 3];
    let mut fwd2: vec3_t = [0.0; 3];
    let mut dot: f32;
    let oldWeapon: c_int;
    let activator: *mut gentity_t = other;
    let zoffset: f32 = 50.0;
    let mut anglesToOwner: vec3_t = [0.0; 3];
    let mut vLen: vec3_t = [0.0; 3];
    let ownLen: f32;

    if (*self_).health <= 0 {
        //gun is destroyed
        return;
    }

    if !(*self_).activator.is_null() {
        //someone is already using me
        return;
    }

    if (*activator).client.is_null() {
        return;
    }

    if (*(*activator).client).ps.emplacedTime > (*addr_of!(level)).time as f32 {
        //last use attempt still too recent
        return;
    }

    if (*(*activator).client).ps.forceHandExtend != HANDEXTEND_NONE {
        //don't use if busy doing something else
        return;
    }

    if (*(*activator).client).ps.origin[2] > (*self_).s.origin[2] + zoffset - 8.0 {
        //can't use it from the top
        return;
    }

    if (*(*activator).client).ps.pm_flags & PMF_DUCKED != 0 {
        //must be standing
        return;
    }
    if (*(*activator).client).ps.isJediMaster != QFALSE {
        //jm can't use weapons
        return;
    }
    VectorSubtract(
        &(*self_).s.origin,
        &(*(*activator).client).ps.origin,
        &mut vLen,
    );
    ownLen = VectorLength(&vLen);

    if ownLen > 64.0 {
        //must be within 64 units of the gun to use at all
        return;
    }

    // Let's get some direction vectors for the user
    AngleVectors(
        &(*(*activator).client).ps.viewangles,
        Some(&mut fwd1),
        None,
        None,
    );

    // Get the guns direction vector
    AngleVectors(&(*self_).pos1, Some(&mut fwd2), None, None);

    dot = DotProduct(&fwd1, &fwd2);

    // Must be reasonably facing the way the gun points ( 110 degrees or so ), otherwise we don't allow to use it.
    if dot < -0.2 {
        //well, not in the right dir, try healing it instead...
        TryHeal(activator, self_);
        return;
    }

    VectorSubtract(
        &(*self_).s.origin,
        &(*(*activator).client).ps.origin,
        &mut fwd1,
    );
    VectorNormalize(&mut fwd1);

    dot = DotProduct(&fwd1, &fwd2);

    //check the positioning in relation to the gun as well
    if dot < 0.6 {
        //well, not in the right dir, try healing it instead...
        TryHeal(activator, self_);
        return;
    }

    (*self_).genericValue1 = 1;

    oldWeapon = (*activator).s.weapon;

    // swap the users weapon with the emplaced gun
    (*(*activator).client).ps.weapon = (*self_).s.weapon;
    (*(*activator).client).ps.weaponstate = WEAPON_READY;
    (*(*activator).client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_EMPLACED_GUN;

    (*(*activator).client).ps.emplacedIndex = (*self_).s.number;

    (*self_).s.emplacedOwner = (*activator).s.number;
    (*self_).s.activeForcePass = NUM_FORCE_POWERS as c_int + 1;

    // the gun will track which weapon we used to have
    (*self_).s.weapon = oldWeapon;

    //user's new owner becomes the gun ent
    (*activator).r.ownerNum = (*self_).s.number;
    (*self_).activator = activator;

    VectorSubtract(
        &(*self_).r.currentOrigin,
        &(*(*activator).client).ps.origin,
        &mut anglesToOwner,
    );
    // C aliases src==dst here; vectoangles consumes its input before writing the
    // result, so a local copy of the source reproduces the behaviour bit-for-bit.
    let anglesSrc = anglesToOwner;
    vectoangles(&anglesSrc, &mut anglesToOwner);
}

/// `void emplaced_gun_realuse( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_weapon.c:4730) — the `use` fn-pointer the engine actually invokes; it just forwards
/// to [`emplaced_gun_use`] (which has the `touch`-shaped signature) with a NULL trace. The
/// `activator` param is dropped, exactly as in C. No oracle (pure forwarder onto a
/// gentity-mutating callback).
///
/// # Safety
/// As [`emplaced_gun_use`]; `self_`/`other` must be valid `gentity_t`s.
pub unsafe extern "C" fn emplaced_gun_realuse(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    emplaced_gun_use(self_, other, null_mut());
}

// g_weapon.c:4588 / :4751 — file-local spawnflag + base-health defines for the
// emplaced gun.
const EMPLACED_CANRESPAWN: c_int = 1;
const EMPLACED_GUN_HEALTH: c_int = 800;

/// `void emplaced_gun_update(gentity_t *self)` (g_weapon.c:4754) — the emplaced gun's
/// per-frame `think`. Handles, in order: arming the respawn delay (or respawning) when
/// dead; finishing the flash-then-explode death (detpack effect + radius damage once the
/// warning timer elapses); puffing smoke while freshly dead; and managing the current
/// user — dropping them off the gun when they release USE, when they walk away (>64u),
/// when they die/leave, or otherwise forcing their weapon to stay `WP_EMPLACED_GUN`. Always
/// reschedules itself 50ms out. No oracle (gentity/playerState mutation + effect /
/// radius-damage side-effects; `Q_irand` is already oracled).
///
/// # Safety
/// `self_` must point to a valid emplaced-gun `gentity_t`; `self_->activator` (and its
/// client) is NULL-checked before use. `level` must be initialised.
pub unsafe extern "C" fn emplaced_gun_update(self_: *mut gentity_t) {
    let mut smokeOrg: vec3_t = [0.0; 3];
    let mut puffAngle: vec3_t = [0.0; 3];
    let oldWeap: c_int;
    let mut ownLen: f32 = 0.0;

    if (*self_).health < 1 && (*self_).genericValue5 == 0 {
        //we are dead, set our respawn delay if we have one
        if (*self_).spawnflags & EMPLACED_CANRESPAWN != 0 {
            (*self_).genericValue5 = (*addr_of!(level)).time + 4000 + (*self_).count;
        }
    } else if (*self_).health < 1 && (*self_).genericValue5 < (*addr_of!(level)).time {
        //we are dead, see if it's time to respawn
        (*self_).s.time = 0;
        (*self_).genericValue4 = 0;
        (*self_).genericValue3 = 0;
        (*self_).health = (EMPLACED_GUN_HEALTH as f32 * 0.4) as c_int;
        (*self_).s.health = (*self_).health;
    }

    if (*self_).genericValue4 != 0
        && (*self_).genericValue4 < 2
        && (*self_).s.time < (*addr_of!(level)).time
    {
        //we have finished our warning (red flashing) effect, it's time to finish dying
        let mut explOrg: vec3_t = [0.0; 3];

        VectorSet(&mut puffAngle, 0.0, 0.0, 1.0);

        VectorCopy(&(*self_).r.currentOrigin, &mut explOrg);
        explOrg[2] += 16.0;

        //just use the detpack explosion effect
        G_PlayEffect(EFFECT_EXPLOSION_DETPACK, &explOrg, &puffAngle);

        (*self_).genericValue3 = (*addr_of!(level)).time + Q_irand(2500, 3500);

        G_RadiusDamage(
            &(*self_).r.currentOrigin,
            self_,
            (*self_).splashDamage as f32,
            (*self_).splashRadius as f32,
            self_,
            null_mut(),
            MOD_UNKNOWN,
        );

        (*self_).s.time = -1;

        (*self_).genericValue4 = 2;
    }

    if (*self_).genericValue3 > (*addr_of!(level)).time {
        //see if we are freshly dead and should be smoking
        if (*self_).genericValue2 < (*addr_of!(level)).time {
            //is it time yet to spawn another smoke puff?
            VectorSet(&mut puffAngle, 0.0, 0.0, 1.0);
            VectorCopy(&(*self_).r.currentOrigin, &mut smokeOrg);

            smokeOrg[2] += 60.0;

            G_PlayEffect(EFFECT_SMOKE, &smokeOrg, &puffAngle);
            (*self_).genericValue2 = (*addr_of!(level)).time + Q_irand(250, 400);
        }
    }

    if !(*self_).activator.is_null()
        && !(*(*self_).activator).client.is_null()
        && (*(*self_).activator).inuse != QFALSE
    {
        //handle updating current user
        let mut vLen: vec3_t = [0.0; 3];
        VectorSubtract(
            &(*self_).s.origin,
            &(*(*(*self_).activator).client).ps.origin,
            &mut vLen,
        );
        ownLen = VectorLength(&vLen);

        if (*(*(*self_).activator).client).pers.cmd.buttons & BUTTON_USE == 0
            && (*self_).genericValue1 != 0
        {
            (*self_).genericValue1 = 0;
        }

        if (*(*(*self_).activator).client).pers.cmd.buttons & BUTTON_USE != 0
            && (*self_).genericValue1 == 0
        {
            (*(*(*self_).activator).client).ps.emplacedIndex = 0;
            (*(*(*self_).activator).client).ps.saberHolstered = 0;
            (*self_).nextthink = (*addr_of!(level)).time + 50;
            return;
        }
    }

    if (!(*self_).activator.is_null() && !(*(*self_).activator).client.is_null())
        && ((*(*self_).activator).inuse == QFALSE
            || (*(*(*self_).activator).client).ps.emplacedIndex != (*self_).s.number
            || (*self_).genericValue4 != 0
            || ownLen > 64.0)
    {
        //get the user off of me then
        (*(*(*self_).activator).client).ps.stats[STAT_WEAPONS as usize] &= !(1 << WP_EMPLACED_GUN);

        oldWeap = (*(*(*self_).activator).client).ps.weapon;
        (*(*(*self_).activator).client).ps.weapon = (*self_).s.weapon;
        (*self_).s.weapon = oldWeap;
        (*(*self_).activator).r.ownerNum = ENTITYNUM_NONE;
        (*(*(*self_).activator).client).ps.emplacedTime = ((*addr_of!(level)).time + 1000) as f32;
        (*(*(*self_).activator).client).ps.emplacedIndex = 0;
        (*(*(*self_).activator).client).ps.saberHolstered = 0;
        (*self_).activator = null_mut();

        (*self_).s.activeForcePass = 0;
    } else if !(*self_).activator.is_null() && !(*(*self_).activator).client.is_null() {
        //make sure the user is using the emplaced gun weapon
        (*(*(*self_).activator).client).ps.weapon = WP_EMPLACED_GUN;
        (*(*(*self_).activator).client).ps.weaponstate = WEAPON_READY;
    }
    (*self_).nextthink = (*addr_of!(level)).time + 50;
}

// be_aas.h `solid_t::SOLID_BBOX` — "touch on edge" bbox clip; written into
// `entityState_t::solid` by the spawn fn before `trap_LinkEntity` overwrites it.
const SOLID_BBOX: c_int = 2;

/*QUAKED emplaced_gun (0 0 1) (-30 -20 8) (30 20 60) CANRESPAWN

 count - if CANRESPAWN spawnflag, decides how long it is before gun respawns (in ms)
 constraint - number of degrees gun is constrained from base angles on each side (default 60.0)

 showhealth - set to 1 to show health bar on this entity when crosshair is over it

  teamowner - crosshair shows green for this team, red for opposite team
	0 - none
	1 - red
	2 - blue

  alliedTeam - team that can use this
	0 - any
	1 - red
	2 - blue

  teamnodmg - team that turret does not take damage from or do damage to
	0 - none
	1 - red
	2 - blue
*/
/// `void SP_emplaced_gun( gentity_t *ent )` (g_weapon.c:4870) — the `emplaced_gun` map-entity
/// spawn function. Precaches the emplaced-gun item, sets the solid bbox + bounds, drops the
/// gun to the floor with a downward trace, applies base health (halved if it can respawn),
/// wires up the `use`/`pain`/`die`/`think` callbacks to the emplaced-gun siblings, reads the
/// `count`/`constraint` spawn keys, sets the ghoul2 model + emplaced weapon id, stores the
/// base angles, and links the entity. No oracle (entity-state spawn fn: trace + item
/// precache + gentity mutation + link side-effects).
///
/// # Safety
/// `ent` must point to a freshly-spawned, zero-initialised `gentity_t` with `s.origin`/
/// `s.angles`/`spawnflags` populated by the spawner. `level` must be initialised.
pub unsafe fn SP_emplaced_gun(ent: *mut gentity_t) {
    let name = "models/map_objects/mp/turret_chair.glm";
    let mut down: vec3_t = [0.0; 3];

    //make sure our assets are precached
    RegisterItem(BG_FindItemForWeapon(WP_EMPLACED_GUN));

    (*ent).r.contents = CONTENTS_SOLID;
    (*ent).s.solid = SOLID_BBOX;

    (*ent).genericValue5 = 0;

    VectorSet(&mut (*ent).r.mins, -30.0, -20.0, 8.0);
    VectorSet(&mut (*ent).r.maxs, 30.0, 20.0, 60.0);

    VectorCopy(&(*ent).s.origin, &mut down);

    down[2] -= 1024.0;

    let tr = trap::Trace(
        &(*ent).s.origin,
        &(*ent).r.mins,
        &(*ent).r.maxs,
        &down,
        (*ent).s.number,
        MASK_SOLID,
    );

    if tr.fraction != 1.0 && tr.allsolid == 0 && tr.startsolid == 0 {
        VectorCopy(&tr.endpos, &mut (*ent).s.origin);
    }

    (*ent).spawnflags |= 4; // deadsolid

    (*ent).health = EMPLACED_GUN_HEALTH;

    if (*ent).spawnflags & EMPLACED_CANRESPAWN != 0 {
        //make it somewhat easier to kill if it can respawn
        (*ent).health = ((*ent).health as f32 * 0.4) as c_int;
    }

    (*ent).maxHealth = (*ent).health;
    G_ScaleNetHealth(ent);

    (*ent).genericValue4 = 0;

    (*ent).takedamage = QTRUE;
    (*ent).pain = Some(emplaced_gun_pain);
    (*ent).die = Some(emplaced_gun_die);

    // being caught in this thing when it blows would be really bad.
    (*ent).splashDamage = 80;
    (*ent).splashRadius = 128;

    // amount of ammo that this little poochie has
    G_SpawnInt(c"count".as_ptr(), c"600".as_ptr(), &mut (*ent).count);

    G_SpawnFloat(
        c"constraint".as_ptr(),
        c"60".as_ptr(),
        &mut (*ent).s.origin2[0],
    );

    (*ent).s.modelindex = G_ModelIndex(name);
    (*ent).s.modelGhoul2 = 1;
    (*ent).s.g2radius = 110;

    //so the cgame knows for sure that we're an emplaced weapon
    (*ent).s.weapon = WP_EMPLACED_GUN;

    // C passes &ent->s.origin directly; G_SetOrigin only reads it (copying into
    // pos.trBase / r.currentOrigin), so a local copy is behaviourally identical
    // while satisfying the borrow checker (the call mutates *ent).
    let origin = (*ent).s.origin;
    G_SetOrigin(ent, &origin);

    // store base angles for later
    VectorCopy(&(*ent).s.angles, &mut (*ent).pos1);
    VectorCopy(&(*ent).s.angles, &mut (*ent).r.currentAngles);
    VectorCopy(&(*ent).s.angles, &mut (*ent).s.apos.trBase);

    (*ent).think = Some(emplaced_gun_update);
    (*ent).nextthink = (*addr_of!(level)).time + 50;

    (*ent).r#use = Some(emplaced_gun_realuse);

    (*ent).r.svFlags |= SVF_PLAYER_USABLE;

    (*ent).s.pos.trType = TR_STATIONARY;

    (*ent).s.owner = MAX_CLIENTS as c_int + 1;
    (*ent).s.shouldtarget = QTRUE;
    //ent->s.teamowner = 0;

    trap::LinkEntity(ent);
}

/// `void charge_stick( gentity_t *self, gentity_t *other, trace_t *trace )` (g_weapon.c:2607) —
/// the `touch` callback for a thrown detpack charge as it hits something. Picks one of several
/// behaviours by what it struck:
/// - a perfectly still breakable brush (`FL_BBRUSH`, both trajectories `TR_STATIONARY`): remember
///   it as `target_ent` so it takes direct damage when the charge blows;
/// - a non-world mover hit from below (`ET_MOVER`, `plane.normal[2] > 0`): ride it via
///   `s.groundEntityNum`;
/// - a non-world non-stickable entity (a client, or anything with no weapon): randomly "bounce" off
///   the surface normal, reface, keep `charge_stick` as the touch, and bail without sticking;
/// - any other non-world entity (e.g. another projectile): explode immediately with a det-pack
///   radius blast + effect, then free next frame;
/// - otherwise (the world): stick — disarm touch, arm `DetPackBlow` after 30 s, freeze the
///   trajectory at the impact origin, orient to the (normalized) surface normal, play the stick
///   sound, spit a broadcast `EV_MISSILE_MISS` temp-entity owned by the charge, and set
///   `SVF_OWNERNOTSHARED` so the owner can shoot it. A `pub unsafe extern "C"` fn for the
/// `gentity_t::touch` fn-pointer ABI. No oracle (gentity-state mutation + radius-damage / effect /
/// sound / temp-entity side-effects; the "bounce" math is `Q_irand`-driven and already oracled).
///
/// NOTE: the z-component of the bounce nudge multiplies by `vNor[1]` (not `vNor[2]`) — this is a
/// faithful port of the original JKA bug, preserved verbatim.
///
/// # Safety
/// `self_` must point to a valid charge `gentity_t`; `trace` must be a valid `trace_t`. `other` is
/// NULL-checked before use. `level` must be initialised.
pub unsafe extern "C" fn charge_stick(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    trace: *mut trace_t,
) {
    let tent: *mut gentity_t;

    if !other.is_null()
        && (*other).flags & FL_BBRUSH != 0
        && (*other).s.pos.trType == TR_STATIONARY
        && (*other).s.apos.trType == TR_STATIONARY
    {
        //a perfectly still breakable brush, let us attach directly to it!
        (*self_).target_ent = other; //remember them when we blow up
    } else if !other.is_null()
        && (*other).s.number < ENTITYNUM_WORLD
        && (*other).s.eType == ET_MOVER
        && (*trace).plane.normal[2] > 0.0
    {
        //stick to it?
        (*self_).s.groundEntityNum = (*other).s.number;
    } else if !other.is_null()
        && (*other).s.number < ENTITYNUM_WORLD
        && (!(*other).client.is_null() || (*other).s.weapon == 0)
    {
        //hit another entity that is not stickable, "bounce" off
        let mut vNor: vec3_t = [0.0; 3];
        let mut tN: vec3_t = [0.0; 3];

        VectorCopy(&(*trace).plane.normal, &mut vNor);
        VectorNormalize(&mut vNor);
        VectorNPos(&(*self_).s.pos.trDelta, &mut tN);
        (*self_).s.pos.trDelta[0] += vNor[0] * (tN[0] * ((Q_irand(1, 10) as f32) * 0.1));
        (*self_).s.pos.trDelta[1] += vNor[1] * (tN[1] * ((Q_irand(1, 10) as f32) * 0.1));
        (*self_).s.pos.trDelta[2] += vNor[1] * (tN[2] * ((Q_irand(1, 10) as f32) * 0.1));

        vectoangles(&vNor, &mut (*self_).s.angles);
        vectoangles(&vNor, &mut (*self_).s.apos.trBase);
        (*self_).touch = Some(charge_stick);
        return;
    } else if !other.is_null() && (*other).s.number < ENTITYNUM_WORLD {
        //hit an entity that we just want to explode on (probably another projectile or something)
        let mut v: vec3_t = [0.0; 3];

        (*self_).touch = None;
        (*self_).think = None;
        (*self_).nextthink = 0;

        (*self_).takedamage = QFALSE;

        VectorClear(&mut (*self_).s.apos.trDelta);
        (*self_).s.apos.trType = TR_STATIONARY;

        G_RadiusDamage(
            &(*self_).r.currentOrigin,
            (*self_).parent,
            (*self_).splashDamage as f32,
            (*self_).splashRadius as f32,
            self_,
            self_,
            MOD_DET_PACK_SPLASH,
        );
        VectorCopy(&(*trace).plane.normal, &mut v);
        VectorCopy(&v, &mut (*self_).pos2);
        (*self_).count = -1;
        G_PlayEffect(EFFECT_EXPLOSION_DETPACK, &(*self_).r.currentOrigin, &v);

        (*self_).think = Some(G_FreeEntity);
        (*self_).nextthink = (*addr_of!(level)).time;
        return;
    }

    //if we get here I guess we hit hte world so we can stick to it

    (*self_).touch = None;
    (*self_).think = Some(DetPackBlow);
    (*self_).nextthink = (*addr_of!(level)).time + 30000;

    VectorClear(&mut (*self_).s.apos.trDelta);
    (*self_).s.apos.trType = TR_STATIONARY;

    (*self_).s.pos.trType = TR_STATIONARY;
    VectorCopy(&(*self_).r.currentOrigin, &mut (*self_).s.origin);
    VectorCopy(&(*self_).r.currentOrigin, &mut (*self_).s.pos.trBase);
    VectorClear(&mut (*self_).s.pos.trDelta);

    VectorClear(&mut (*self_).s.apos.trDelta);

    VectorNormalize(&mut (*trace).plane.normal);

    vectoangles(&(*trace).plane.normal, &mut (*self_).s.angles);
    VectorCopy(&(*self_).s.angles, &mut (*self_).r.currentAngles);
    VectorCopy(&(*self_).s.angles, &mut (*self_).s.apos.trBase);

    VectorCopy(&(*trace).plane.normal, &mut (*self_).pos2);
    (*self_).count = -1;

    G_Sound(
        self_,
        CHAN_WEAPON,
        G_SoundIndex("sound/weapons/detpack/stick.wav"),
    );

    tent = G_TempEntity(&(*self_).r.currentOrigin, EV_MISSILE_MISS);
    (*tent).s.weapon = 0;
    (*tent).parent = self_;
    (*tent).r.ownerNum = (*self_).s.number;

    //so that the owner can blow it up with projectiles
    (*self_).r.svFlags |= SVF_OWNERNOTSHARED;
}

/// `void DetPackBlow( gentity_t *self )` (g_weapon.c:2702) — the detonation `think`/explosion
/// for a stuck detpack charge. Disarms the entity (`pain`/`die` cleared, `takedamage` off), then:
/// if it was stuck to a `target_ent` it does *direct* `MOD_DET_PACK_SPLASH` damage to that target;
/// always does `G_RadiusDamage` over `splashRadius`; plays `EFFECT_EXPLOSION_DETPACK` oriented up
/// (or along `pos2` when `count == -1`, i.e. stuck to a surface); and frees itself next frame. A
/// `pub unsafe extern "C"` fn for the `gentity_t::think` fn-pointer ABI. No oracle (entity-state
/// mutation + trap/effect side-effects).
///
/// The `dir` vec `v` is left uninitialised in C at the point it is passed to `G_Damage` (used only
/// for knockback direction inside that call); we zero-initialise it for defined behaviour — it is
/// reassigned to `(0,0,1)` (or `pos2`) before the effect call regardless.
///
/// # Safety
/// `self_` must point to a valid detpack `gentity_t`; `level`/`g_entities` must be initialised.
pub unsafe extern "C" fn DetPackBlow(self_: *mut gentity_t) {
    let mut v: vec3_t = [0.0; 3];

    (*self_).pain = None;
    (*self_).die = None;
    (*self_).takedamage = QFALSE;

    if !(*self_).target_ent.is_null() {
        //we were attached to something, do *direct* damage to it!
        G_Damage(
            (*self_).target_ent,
            self_,
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*self_).r.ownerNum as usize),
            &mut v,
            addr_of_mut!((*self_).r.currentOrigin),
            (*self_).damage,
            0,
            MOD_DET_PACK_SPLASH,
        );
    }
    G_RadiusDamage(
        &(*self_).r.currentOrigin,
        (*self_).parent,
        (*self_).splashDamage as f32,
        (*self_).splashRadius as f32,
        self_,
        self_,
        MOD_DET_PACK_SPLASH,
    );
    v[0] = 0.0;
    v[1] = 0.0;
    v[2] = 1.0;

    if (*self_).count == -1 {
        VectorCopy(&(*self_).pos2, &mut v);
    }

    G_PlayEffect(EFFECT_EXPLOSION_DETPACK, &(*self_).r.currentOrigin, &v);

    (*self_).think = Some(G_FreeEntity);
    (*self_).nextthink = (*addr_of!(level)).time;
}

/// `void BlowDetpacks( gentity_t *ent )` (g_weapon.c:2812).
///
/// Detonate all of `ent`'s planted detpacks: if the firer has a charge planted, walk every
/// `"detpack"`-classname entity via [`G_Find`], and for each one parented to `ent`, snap its
/// spawn origin to its current origin, arm its `DetPackBlow` think on a short random fuse
/// (100-300 ms), and play the warning sound. Clears `hasDetPackPlanted` afterward. No oracle
/// (entity-state mutation + trap-backed `G_Sound`).
///
/// # Safety
/// `ent` must be a valid client `gentity_t` (`ent->client` non-null); `level`/`g_entities`
/// must be initialised.
pub unsafe fn BlowDetpacks(ent: *mut gentity_t) {
    let mut found: *mut gentity_t = null_mut();

    if (*(*ent).client).ps.hasDetPackPlanted != QFALSE {
        loop {
            found = G_Find(
                found,
                offset_of!(gentity_t, classname),
                c"detpack".as_ptr(),
            );
            if found.is_null() {
                break;
            }
            //loop through all ents and blow the crap out of them!
            if (*found).parent == ent {
                VectorCopy(&(*found).r.currentOrigin, &mut (*found).s.origin);
                (*found).think = Some(DetPackBlow);
                (*found).nextthink = (*addr_of!(level)).time + 100 + (random() * 200.0) as c_int;
                G_Sound(
                    found,
                    CHAN_BODY,
                    G_SoundIndex("sound/weapons/detpack/warning.wav"),
                );
            }
        }
        (*(*ent).client).ps.hasDetPackPlanted = QFALSE;
    }
}

/// `void DetPackPain( gentity_t *self, gentity_t *attacker, int damage )` (g_weapon.c:2730) —
/// the detpack charge's `pain` callback: when the stuck charge is hurt, arm it to detonate via
/// `DetPackBlow` after a short random delay (50-100 ms) and stop taking further damage. The two
/// trailing params (`attacker`/`damage`) are unused, matching C. A `pub unsafe extern "C"` fn for
/// the `gentity_t::pain` fn-pointer ABI. No oracle (entity-state mutation: sets a fn-pointer
/// `think` + `nextthink` and clears `takedamage`; the only computed value, `Q_irand(50, 100)`, is
/// RNG-driven and already oracled).
///
/// # Safety
/// `self_` must point to a valid detpack `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn DetPackPain(
    self_: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
) {
    (*self_).think = Some(DetPackBlow);
    (*self_).nextthink = (*addr_of!(level)).time + Q_irand(50, 100);
    (*self_).takedamage = QFALSE;
}

/// `void DetPackDie( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int mod )`
/// (g_weapon.c:2737) — the detpack charge's `die` callback: when the stuck charge is destroyed by
/// damage, arm it to detonate via `DetPackBlow` after a short random delay (50-100 ms) and stop
/// taking further damage. The four trailing params (`inflictor`/`attacker`/`damage`/`mod`) are
/// unused, matching C. A `pub unsafe extern "C"` fn for the `gentity_t::die` fn-pointer ABI.
/// No oracle (entity-state mutation: sets a fn-pointer `think` + `nextthink` and clears
/// `takedamage`; the only computed value, `Q_irand(50, 100)`, is RNG-driven and already oracled).
///
/// # Safety
/// `self_` must point to a valid detpack `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn DetPackDie(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    _mod_: c_int,
) {
    (*self_).think = Some(DetPackBlow);
    (*self_).nextthink = (*addr_of!(level)).time + Q_irand(50, 100);
    (*self_).takedamage = QFALSE;
}

// `void G_RunObject( gentity_t *ent )` (g_object.c) — the per-frame physics `think` shared by
// thrown objects (det packs, thermals, etc.), used for the `gentity_t::think` fn-pointer
// assignment in `drop_charge`. Now ported in g_object.rs, so imported directly.
use crate::codemp::game::g_object::G_RunObject;

/// `void drop_charge( gentity_t *self, vec3_t start, vec3_t dir )` (g_weapon.c:2744).
///
/// Spawns a det-pack projectile at `start` headed along `dir`: a `WP_DET_PACK`, `TR_GRAVITY`
/// Ghoul2 physics object (300 u/s) with a 2-unit bbox, `MOD_DET_PACK_SPLASH`, randomised tumble,
/// the already-ported `charge_stick` touch / `DetPackPain` pain / `DetPackDie` die callbacks, and
/// the (not-yet-ported-g_object) `G_RunObject` think. No oracle (entity-state construction + `rand`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `start`/`dir` must each point to a valid 3-float
/// vector; `level` must be initialised.
pub unsafe fn drop_charge(self_: *mut gentity_t, start: &vec3_t, dir: &mut vec3_t) {
    VectorNormalize(dir);

    let bolt = G_Spawn();
    (*bolt).classname = c"detpack".as_ptr() as *mut c_char;
    (*bolt).nextthink = (*addr_of!(level)).time + FRAMETIME;
    (*bolt).think = Some(G_RunObject);
    (*bolt).s.eType = ET_GENERAL;
    (*bolt).s.g2radius = 100;
    (*bolt).s.modelGhoul2 = 1;
    (*bolt).s.modelindex = G_ModelIndex("models/weapons2/detpack/det_pack_proj.glm");

    (*bolt).parent = self_;
    (*bolt).r.ownerNum = (*self_).s.number;
    (*bolt).damage = 100;
    (*bolt).splashDamage = 200;
    (*bolt).splashRadius = 200;
    (*bolt).methodOfDeath = MOD_DET_PACK_SPLASH;
    (*bolt).splashMethodOfDeath = MOD_DET_PACK_SPLASH;
    (*bolt).clipmask = MASK_SHOT;
    (*bolt).s.solid = 2;
    (*bolt).r.contents = MASK_SHOT;
    (*bolt).touch = Some(charge_stick);

    (*bolt).physicsObject = QTRUE;

    (*bolt).s.genericenemyindex = (*self_).s.number + MAX_GENTITIES as c_int;
    //rww - so client prediction knows we own this and won't hit it

    VectorSet(&mut (*bolt).r.mins, -2.0, -2.0, -2.0);
    VectorSet(&mut (*bolt).r.maxs, 2.0, 2.0, 2.0);

    (*bolt).health = 1;
    (*bolt).takedamage = QTRUE;
    (*bolt).pain = Some(DetPackPain);
    (*bolt).die = Some(DetPackDie);

    (*bolt).s.weapon = WP_DET_PACK;

    (*bolt).setTime = (*addr_of!(level)).time;

    G_SetOrigin(bolt, start);
    (*bolt).s.pos.trType = TR_GRAVITY;
    VectorCopy(start, &mut (*bolt).s.pos.trBase);
    VectorScale(dir, 300.0, &mut (*bolt).s.pos.trDelta);
    (*bolt).s.pos.trTime = (*addr_of!(level)).time;

    (*bolt).s.apos.trType = TR_GRAVITY;
    (*bolt).s.apos.trTime = (*addr_of!(level)).time;
    (*bolt).s.apos.trBase[YAW] = (rand() % 360) as f32;
    (*bolt).s.apos.trBase[PITCH] = (rand() % 360) as f32;
    (*bolt).s.apos.trBase[ROLL] = (rand() % 360) as f32;

    if rand() % 10 < 5 {
        (*bolt).s.apos.trBase[YAW] = -(*bolt).s.apos.trBase[YAW];
    }

    vectoangles(dir, &mut (*bolt).s.angles);
    let s_angles = (*bolt).s.angles;
    VectorCopy(&s_angles, &mut (*bolt).s.apos.trBase);
    VectorSet(&mut (*bolt).s.apos.trDelta, 300.0, 0.0, 0.0);
    (*bolt).s.apos.trTime = (*addr_of!(level)).time;

    trap::LinkEntity(bolt);
}

/// `void WP_DropDetPack( gentity_t *ent, qboolean alt_fire )` (g_weapon.c:2842).
///
/// The det-pack weapon's fire: first caps the owner to 10 planted packs (freeing the oldest
/// extras by `setTime`, unless cheats are on — [`CheatsOn`] — which grants unlimited). On alt-fire
/// it detonates all the owner's planted packs via [`BlowDetpacks`]; otherwise it recomputes the
/// muzzle from the firer's view (`AngleVectors` → file-statics + [`CalcMuzzlePoint`]), nudges the
/// muzzle 4u back along `forward`, drops a fresh pack via [`drop_charge`], and flags
/// `hasDetPackPlanted`. No oracle (file-static aim + gentity spawn/free).
///
/// # Safety
/// `ent`/`ent->client` are NULL-checked. Writes the file-statics `forward`/`vright`/`up`/`muzzle`;
/// `level`/`g_entities` must be initialised.
#[allow(unused_assignments)] // faithful `found = NULL;` dead reset before the inner loop reassigns it
pub unsafe fn WP_DropDetPack(ent: *mut gentity_t, alt_fire: qboolean) {
    let mut found: *mut gentity_t = null_mut();
    let mut trapcount: c_int = 0;
    let mut foundDetPacks: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    foundDetPacks[0] = ENTITYNUM_NONE;
    let trapcount_org: c_int;
    let mut lowestTimeStamp: c_int;
    let mut removeMe: c_int;
    let mut i: c_int;

    if ent.is_null() || (*ent).client.is_null() {
        return;
    }

    //limit to 10 placed at any one time
    //see how many there are now
    loop {
        found = G_Find(
            found,
            offset_of!(gentity_t, classname),
            c"detpack".as_ptr(),
        );
        if found.is_null() {
            break;
        }
        if (*found).parent != ent {
            continue;
        }
        foundDetPacks[trapcount as usize] = (*found).s.number;
        trapcount += 1;
    }
    //now remove first ones we find until there are only 9 left
    found = null_mut();
    trapcount_org = trapcount;
    lowestTimeStamp = (*addr_of!(level)).time;
    while trapcount > 9 {
        removeMe = -1;
        i = 0;
        while i < trapcount_org {
            if foundDetPacks[i as usize] == ENTITYNUM_NONE {
                i += 1;
                continue;
            }
            found = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(foundDetPacks[i as usize] as usize);
            if (*found).setTime < lowestTimeStamp {
                removeMe = i;
                lowestTimeStamp = (*found).setTime;
            }
            i += 1;
        }
        if removeMe != -1 {
            //remove it... or blow it?
            if (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(foundDetPacks[removeMe as usize] as usize).is_null() {
                break;
            } else if CheatsOn() == QFALSE {
                //Let them have unlimited if cheats are enabled
                G_FreeEntity((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(foundDetPacks[removeMe as usize] as usize));
            }
            foundDetPacks[removeMe as usize] = ENTITYNUM_NONE;
            trapcount -= 1;
        } else {
            break;
        }
    }

    if alt_fire != QFALSE {
        BlowDetpacks(ent);
    } else {
        AngleVectors(
            &(*(*ent).client).ps.viewangles,
            Some(&mut *addr_of_mut!(forward)),
            Some(&mut *addr_of_mut!(vright)),
            Some(&mut *addr_of_mut!(up)),
        );

        CalcMuzzlePoint(
            ent,
            &*addr_of!(forward),
            &*addr_of!(vright),
            &*addr_of!(up),
            &mut *addr_of_mut!(muzzle),
        );

        VectorNormalize(&mut *addr_of_mut!(forward));
        let muzzleCopy = *addr_of!(muzzle);
        VectorMA(&muzzleCopy, -4.0, &*addr_of!(forward), &mut *addr_of_mut!(muzzle));
        drop_charge(ent, &*addr_of!(muzzle), &mut *addr_of_mut!(forward));

        (*(*ent).client).ps.hasDetPackPlanted = QTRUE;
    }
}

/// `void laserTrapDelayedExplode( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int meansOfDeath )`
/// (g_weapon.c:2244) — the laser-trap / trip-mine `die` callback: when the mine is destroyed by
/// damage, remember the `attacker` as `enemy`, arm `laserTrapExplode` for next frame, and stop
/// taking further damage. When the killer is the local player (`attacker` present and
/// `s.number == 0`) the splash damage and radius are cut to a third. The `inflictor`/`damage`/
/// `meansOfDeath` params are unused, matching C. A `pub unsafe extern "C"` fn for the
/// `gentity_t::die` fn-pointer ABI. No oracle (entity-state mutation only — sets `enemy`/`think`/
/// `nextthink`/`takedamage` and conditionally scales the integer splash fields).
///
/// # Safety
/// `self_` must point to a valid laser-trap `gentity_t`; `level` must be initialised. `attacker`
/// is NULL-checked before being dereferenced.
pub unsafe extern "C" fn laserTrapDelayedExplode(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _means_of_death: c_int,
) {
    (*self_).enemy = attacker;
    (*self_).think = Some(laserTrapExplode);
    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
    (*self_).takedamage = QFALSE;
    if !attacker.is_null() && (*attacker).s.number == 0 {
        //less damage when shot by player
        (*self_).splashDamage /= 3;
        (*self_).splashRadius /= 3;
    }
}

/// `void laserTrapExplode( gentity_t *self )` (g_weapon.c:2206) — the detonation `think` for a
/// laser trap / trip mine (and the flechette alt-fire mines). Disarms (`takedamage` off), does
/// `G_RadiusDamage` over `splashRadius` against `activator` when present, fires an
/// `EV_MISSILE_MISS` event for non-flechette mines, then plays the explosion effect oriented
/// outward from the stuck surface (`s.pos.trDelta`; zeroed when `s.time == -2`) — the flechette
/// variant uses `EFFECT_EXPLOSION_FLECHETTE`, all others `EFFECT_EXPLOSION_TRIPMINE` — and frees
/// itself next frame. A `pub unsafe extern "C"` fn for the `gentity_t::think` fn-pointer ABI.
/// No oracle (entity-state mutation + radius-damage / event / effect / free side-effects).
///
/// # Safety
/// `self_` must point to a valid laser-trap `gentity_t`; `level`/`g_entities` must be initialised.
pub unsafe extern "C" fn laserTrapExplode(self_: *mut gentity_t) {
    let mut v: vec3_t = [0.0; 3];
    (*self_).takedamage = QFALSE;

    if !(*self_).activator.is_null() {
        G_RadiusDamage(
            &(*self_).r.currentOrigin,
            (*self_).activator,
            (*self_).splashDamage as f32,
            (*self_).splashRadius as f32,
            self_,
            self_,
            MOD_TRIP_MINE_SPLASH, /*MOD_LT_SPLASH*/
        );
    }

    if (*self_).s.weapon as c_int != WP_FLECHETTE {
        G_AddEvent(self_, EV_MISSILE_MISS, 0);
    }

    VectorCopy(&(*self_).s.pos.trDelta, &mut v);
    //Explode outward from the surface

    if (*self_).s.time == -2 {
        v[0] = 0.0;
        v[1] = 0.0;
        v[2] = 0.0;
    }

    if (*self_).s.weapon as c_int == WP_FLECHETTE {
        G_PlayEffect(EFFECT_EXPLOSION_FLECHETTE, &(*self_).r.currentOrigin, &v);
    } else {
        G_PlayEffect(EFFECT_EXPLOSION_TRIPMINE, &(*self_).r.currentOrigin, &v);
    }

    (*self_).think = Some(G_FreeEntity);
    (*self_).nextthink = (*addr_of!(level)).time;
}

/// `void proxMineThink( gentity_t *ent )` (g_weapon.c:2282) — the per-frame `think` for a laid
/// proximity mine (the laser-trap/det-pack proximity variant). It re-thinks every frame
/// (`nextthink = level.time`). It detonates immediately (arming [`laserTrapExplode`]) once its
/// `genericValue15` lifetime expires or its owner is gone/disconnected. Otherwise it scans every
/// client slot and detonates if any living, connected, non-spectating client (other than the
/// owner, past its `tempSpectate` grace, and either on a different team or with friendly-fire on)
/// is within half its `splashRadius`. No oracle (gentity-state / cvar-gated proximity scan). A
/// `pub unsafe extern "C"` fn for the `gentity_t::think` fn-pointer ABI.
///
/// # Safety
/// `ent` must point to a valid mine `gentity_t`; `level`/`g_entities` must be initialised.
pub unsafe extern "C" fn proxMineThink(ent: *mut gentity_t) {
    let mut i: c_int = 0;
    let mut owner: *mut gentity_t = null_mut();

    if (*ent).r.ownerNum < ENTITYNUM_WORLD {
        owner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize);
    }

    (*ent).nextthink = (*addr_of!(level)).time;

    if (*ent).genericValue15 < (*addr_of!(level)).time
        || owner.is_null()
        || (*owner).inuse == QFALSE
        || (*owner).client.is_null()
        || (*(*owner).client).pers.connected != CON_CONNECTED
    {
        //time to die!
        (*ent).think = Some(laserTrapExplode);
        return;
    }

    while i < MAX_CLIENTS as c_int {
        //eh, just check for clients, don't care about anyone else...
        let cl: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if (*cl).inuse != QFALSE
            && !(*cl).client.is_null()
            && (*(*cl).client).pers.connected == CON_CONNECTED
            && owner != cl
            && (*(*cl).client).sess.sessionTeam != TEAM_SPECTATOR
            && (*(*cl).client).tempSpectate < (*addr_of!(level)).time
            && (*cl).health > 0
        {
            if OnSameTeam(owner, cl) == QFALSE || (*addr_of!(g_friendlyFire)).integer != 0 {
                //not on the same team, or friendly fire is enabled
                let mut v: vec3_t = [0.0; 3];

                VectorSubtract(&(*ent).r.currentOrigin, &(*(*cl).client).ps.origin, &mut v);
                if VectorLength(&v) < ((*ent).splashRadius as f32 / 2.0) {
                    (*ent).think = Some(laserTrapExplode);
                    return;
                }
            }
        }
        i += 1;
    }
}

/// `void laserTrapThink( gentity_t *ent )` (g_weapon.c:2329) — the per-frame `think` for an armed
/// trip-wire laser trap. Relinks itself every think; on first run, plays the warning sound and
/// raises `EF_FIRING` to switch on the beam effect. Re-arms itself for the next frame, then traces
/// 1024u along `movedir` from its origin to find the beam's main impact point and sets
/// `s.time = -1` so clients draw the beam. If the trace hits a client or starts in solid, it
/// disarms `touch` and schedules `laserTrapExplode` after `LT_DELAY_TIME`. A `pub unsafe extern "C"`
/// fn for the `gentity_t::think` fn-pointer ABI. No oracle (gentity-state mutation + `trap_LinkEntity`
/// / `trap_Trace` / sound side-effects).
///
/// # Safety
/// `ent` must point to a valid laser-trap `gentity_t`; `level`/`g_entities` must be initialised.
pub unsafe extern "C" fn laserTrapThink(ent: *mut gentity_t) {
    let mut end: vec3_t = [0.0; 3];

    //just relink it every think
    trap::LinkEntity(ent);

    //turn on the beam effect
    if (*ent).s.eFlags & EF_FIRING == 0 {
        //arm me
        G_Sound(
            ent,
            CHAN_WEAPON,
            G_SoundIndex("sound/weapons/laser_trap/warning.wav"),
        );
        (*ent).s.eFlags |= EF_FIRING;
    }
    (*ent).think = Some(laserTrapThink);
    (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME;

    // Find the main impact point
    VectorMA(&(*ent).s.pos.trBase, 1024.0, &(*ent).movedir, &mut end);
    let tr = trap::Trace(
        &(*ent).r.currentOrigin,
        &vec3_origin,
        &vec3_origin,
        &end,
        (*ent).s.number,
        MASK_SHOT,
    );

    let traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

    (*ent).s.time = -1; //let all clients know to draw a beam from this guy

    if !(*traceEnt).client.is_null() || tr.startsolid != 0 {
        //go boom
        (*ent).touch = None;
        (*ent).nextthink = (*addr_of!(level)).time + LT_DELAY_TIME;
        (*ent).think = Some(laserTrapExplode);
    }
}

/// `void touchLaserTrap( gentity_t *ent, gentity_t *other, trace_t *trace )` (g_weapon.c:2258) —
/// the proximity-mine `touch` callback that `laserTrapStick`'s non-tripwire branch installs.
/// Hitting any non-world entity (other than the trap's own activator) arms [`laserTrapExplode`]
/// for next frame; otherwise the trap pins itself to the surface via [`laserTrapStick`]. No
/// oracle (entity-state mutation + `think`/`touch` fn-pointer assignment). A `pub unsafe
/// extern "C"` fn so it can be installed directly as a `gentity_t::touch` fn-pointer.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `other` may be NULL; `trace` must point to a valid
/// `trace_t`. `level` and `g_entities` must be initialised.
pub unsafe extern "C" fn touchLaserTrap(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    trace: *mut trace_t,
) {
    if !other.is_null() && (*other).s.number < ENTITYNUM_WORLD {
        //just explode if we hit any entity. This way we don't have things happening like tripmines floating
        //in the air after getting stuck to a moving door
        if (*ent).activator != other {
            (*ent).touch = None;
            (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME;
            (*ent).think = Some(laserTrapExplode);
            VectorCopy(&(*trace).plane.normal, &mut (*ent).s.pos.trDelta);
        }
    } else {
        (*ent).touch = None;
        if (*trace).entityNum as c_int != ENTITYNUM_NONE {
            (*ent).enemy = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*trace).entityNum as usize);
        }
        laserTrapStick(
            ent,
            (*trace).endpos.as_ptr(),
            (*trace).plane.normal.as_ptr(),
        );
    }
}

/// `void laserTrapStick( gentity_t *ent, vec3_t endpos, vec3_t normal )` (g_weapon.c:2364) —
/// called when a thrown trip-mine/laser-trap missile collides with a surface. Pins the entity to
/// the impact point, orients it to face along the surface `normal`, plays the stick sound, then
/// branches on `ent->count`: a tripwire (`count != 0`) arms after a delay and becomes shootable;
/// otherwise it becomes a proximity mine with a 30-second auto-explode and the warning beam. No
/// oracle (post-collision orient + entity-state setup; assigns `laserTrapThink`/`proxMineThink`
/// as `think` fn-pointers but does not call them). A `pub unsafe extern "C"` fn so the engine
/// missile-impact path can use it directly.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `endpos`/`normal` must each point to a valid 3-float
/// vector (the C `vec3_t` params decay to `vec_t*`). `level` must be initialised (via
/// `G_SetOrigin`/`G_Sound`).
pub unsafe extern "C" fn laserTrapStick(
    ent: *mut gentity_t,
    endpos: *const vec_t,
    normal: *const vec_t,
) {
    let endpos = &*(endpos as *const vec3_t);
    let normal = &*(normal as *const vec3_t);

    G_SetOrigin(ent, endpos);
    VectorCopy(normal, &mut (*ent).pos1);

    VectorClear(&mut (*ent).s.apos.trDelta);
    // This will orient the object to face in the direction of the normal
    VectorCopy(normal, &mut (*ent).s.pos.trDelta);
    //VectorScale( normal, -1, ent->s.pos.trDelta );
    (*ent).s.pos.trTime = (*addr_of!(level)).time;

    //This does nothing, cg_missile makes assumptions about direction of travel controlling angles
    vectoangles(normal, &mut (*ent).s.apos.trBase);
    VectorClear(&mut (*ent).s.apos.trDelta);
    (*ent).s.apos.trType = TR_STATIONARY;
    let apos_trBase = (*ent).s.apos.trBase;
    VectorCopy(&apos_trBase, &mut (*ent).s.angles);
    let s_angles = (*ent).s.angles;
    VectorCopy(&s_angles, &mut (*ent).r.currentAngles);

    G_Sound(
        ent,
        CHAN_WEAPON,
        G_SoundIndex("sound/weapons/laser_trap/stick.wav"),
    );
    if (*ent).count != 0 {
        //a tripwire
        //add draw line flag
        VectorCopy(normal, &mut (*ent).movedir);
        (*ent).think = Some(laserTrapThink);
        (*ent).nextthink = (*addr_of!(level)).time + LT_ACTIVATION_DELAY; //delay the activation
        (*ent).touch = Some(touch_NULL);
        //make it shootable
        (*ent).takedamage = QTRUE;
        (*ent).health = 5;
        (*ent).die = Some(laserTrapDelayedExplode);

        //shove the box through the wall
        VectorSet(
            &mut (*ent).r.mins,
            -LT_SIZE * 2.0,
            -LT_SIZE * 2.0,
            -LT_SIZE * 2.0,
        );
        VectorSet(&mut (*ent).r.maxs, LT_SIZE * 2.0, LT_SIZE * 2.0, LT_SIZE * 2.0);

        //so that the owner can blow it up with projectiles
        (*ent).r.svFlags |= SVF_OWNERNOTSHARED;
    } else {
        (*ent).touch = Some(touchLaserTrap);
        (*ent).think = Some(proxMineThink); //laserTrapExplode;
        (*ent).genericValue15 = (*addr_of!(level)).time + 30000; //auto-explode after 30 seconds.
        (*ent).nextthink = (*addr_of!(level)).time + LT_ALT_TIME; // How long 'til she blows

        //make it shootable
        (*ent).takedamage = QTRUE;
        (*ent).health = 5;
        (*ent).die = Some(laserTrapDelayedExplode);

        //shove the box through the wall
        VectorSet(
            &mut (*ent).r.mins,
            -LT_SIZE * 2.0,
            -LT_SIZE * 2.0,
            -LT_SIZE * 2.0,
        );
        VectorSet(&mut (*ent).r.maxs, LT_SIZE * 2.0, LT_SIZE * 2.0, LT_SIZE * 2.0);

        //so that the owner can blow it up with projectiles
        (*ent).r.svFlags |= SVF_OWNERNOTSHARED;

        if (*ent).s.eFlags & EF_FIRING == 0 {
            //arm me
            G_Sound(
                ent,
                CHAN_WEAPON,
                G_SoundIndex("sound/weapons/laser_trap/warning.wav"),
            );
            (*ent).s.eFlags |= EF_FIRING;
            (*ent).s.time = -1;
            (*ent).s.bolt2 = 1;
        }
    }
}

/// `void TrapThink( gentity_t *ent )` (g_weapon.c:2433) — the per-frame `think` a placed laser
/// trap/trip-mine runs once armed: re-arms itself 50 ms out and runs the shared thrown-object
/// physics ([`G_RunObject`]). A `pub unsafe extern "C"` fn for the `gentity_t::think` fn-pointer
/// ABI. No oracle (entity-state mutation + `G_RunObject` side-effects).
///
/// # Safety
/// `ent` must point to a valid laser-trap `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn TrapThink(ent: *mut gentity_t) {
    //laser trap think
    (*ent).nextthink = (*addr_of!(level)).time + 50;
    G_RunObject(ent);
}

/// `void CreateLaserTrap( gentity_t *laserTrap, vec3_t start, gentity_t *owner )` (g_weapon.c:2439).
///
/// Builds the laser-trap/trip-mine entity at `start` for `owner`: a `WP_TRIP_MINE`,
/// `TR_GRAVITY` Ghoul2 missile with a half-bounce, sticks-to-walls (`EF_MISSILE_STICK`)
/// `LT_SIZE` bbox, `MOD_TRIP_MINE_SPLASH` splash, and randomised tumble angles. Installs the
/// already-ported `touchLaserTrap` touch and the (not-yet-ported-`G_RunObject`) `TrapThink` think,
/// scheduling the first think 50 ms out. No oracle (entity-state construction + `rand`/effect
/// indices).
///
/// # Safety
/// `laserTrap`/`owner` must point to valid `gentity_t`s; `start` must point to a valid 3-float
/// vector; `level` must be initialised. A plain `unsafe fn` (C-internal helper, not a callback).
pub unsafe fn CreateLaserTrap(laserTrap: *mut gentity_t, start: &vec3_t, owner: *mut gentity_t) {
    //create a laser trap entity
    (*laserTrap).classname = c"laserTrap".as_ptr() as *mut c_char;
    (*laserTrap).flags |= FL_BOUNCE_HALF;
    (*laserTrap).s.eFlags |= EF_MISSILE_STICK;
    (*laserTrap).splashDamage = LT_SPLASH_DAM;
    (*laserTrap).splashRadius = LT_SPLASH_RAD as c_int;
    (*laserTrap).damage = LT_DAMAGE;
    (*laserTrap).methodOfDeath = MOD_TRIP_MINE_SPLASH;
    (*laserTrap).splashMethodOfDeath = MOD_TRIP_MINE_SPLASH;
    (*laserTrap).s.eType = ET_GENERAL;
    (*laserTrap).r.svFlags = SVF_USE_CURRENT_ORIGIN;
    (*laserTrap).s.weapon = WP_TRIP_MINE;
    (*laserTrap).s.pos.trType = TR_GRAVITY;
    (*laserTrap).r.contents = MASK_SHOT;
    (*laserTrap).parent = owner;
    (*laserTrap).activator = owner;
    (*laserTrap).r.ownerNum = (*owner).s.number;
    VectorSet(&mut (*laserTrap).r.mins, -LT_SIZE, -LT_SIZE, -LT_SIZE);
    VectorSet(&mut (*laserTrap).r.maxs, LT_SIZE, LT_SIZE, LT_SIZE);
    (*laserTrap).clipmask = MASK_SHOT;
    (*laserTrap).s.solid = 2;
    (*laserTrap).s.modelindex =
        G_ModelIndex("models/weapons2/laser_trap/laser_trap_w.glm");
    (*laserTrap).s.modelGhoul2 = 1;
    (*laserTrap).s.g2radius = 40;

    (*laserTrap).s.genericenemyindex = (*owner).s.number + MAX_GENTITIES as c_int;

    (*laserTrap).health = 1;

    (*laserTrap).s.time = 0;

    (*laserTrap).s.pos.trTime = (*addr_of!(level)).time; // move a bit on the very first frame
    VectorCopy(start, &mut (*laserTrap).s.pos.trBase);
    trap::SnapVector(&mut (*laserTrap).s.pos.trBase); // save net bandwidth

    trap::SnapVector(&mut (*laserTrap).s.pos.trDelta); // save net bandwidth
    VectorCopy(start, &mut (*laserTrap).r.currentOrigin);

    (*laserTrap).s.apos.trType = TR_GRAVITY;
    (*laserTrap).s.apos.trTime = (*addr_of!(level)).time;
    (*laserTrap).s.apos.trBase[YAW] = (rand() % 360) as f32;
    (*laserTrap).s.apos.trBase[PITCH] = (rand() % 360) as f32;
    (*laserTrap).s.apos.trBase[ROLL] = (rand() % 360) as f32;

    if rand() % 10 < 5 {
        (*laserTrap).s.apos.trBase[YAW] = -(*laserTrap).s.apos.trBase[YAW];
    }

    VectorCopy(start, &mut (*laserTrap).pos2);
    (*laserTrap).touch = Some(touchLaserTrap);
    (*laserTrap).think = Some(TrapThink);
    (*laserTrap).nextthink = (*addr_of!(level)).time + 50;
}

/// `void WP_PlaceLaserTrap( gentity_t *ent, qboolean alt_fire )` (g_weapon.c:2495).
///
/// Places a player laser trap from the file-static `muzzle` along `forward`. First caps the
/// owner to 10 simultaneous traps by freeing the oldest extras (by `setTime`), then spawns and
/// builds one via [`CreateLaserTrap`], stamps its `setTime`, marks a non-alt placement as a
/// tripwire (`count = 1`), and launches it as a `TR_GRAVITY` projectile (512 u/s alt, 256 u/s
/// main). No oracle (file-static aim + gentity spawn/link).
///
/// # Safety
/// `ent` must be valid; reads the file-statics `forward`/`muzzle`; `level`/`g_entities` must be
/// initialised.
#[allow(unused_assignments)] // faithful `found = NULL;` dead reset before the inner loop reassigns it
pub unsafe fn WP_PlaceLaserTrap(ent: *mut gentity_t, alt_fire: qboolean) {
    let mut found: *mut gentity_t = null_mut();
    let mut dir: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut trapcount: c_int = 0;
    let mut foundLaserTraps: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let trapcount_org: c_int;
    let mut lowestTimeStamp: c_int;
    let mut removeMe: c_int;
    let mut i: c_int;

    foundLaserTraps[0] = ENTITYNUM_NONE;

    VectorCopy(&*addr_of!(forward), &mut dir);
    VectorCopy(&*addr_of!(muzzle), &mut start);

    let laserTrap = G_Spawn();

    //limit to 10 placed at any one time
    //see how many there are now
    loop {
        found = G_Find(
            found,
            offset_of!(gentity_t, classname),
            c"laserTrap".as_ptr(),
        );
        if found.is_null() {
            break;
        }
        if (*found).parent != ent {
            continue;
        }
        foundLaserTraps[trapcount as usize] = (*found).s.number;
        trapcount += 1;
    }
    //now remove first ones we find until there are only 9 left
    found = null_mut();
    trapcount_org = trapcount;
    lowestTimeStamp = (*addr_of!(level)).time;
    while trapcount > 9 {
        removeMe = -1;
        i = 0;
        while i < trapcount_org {
            if foundLaserTraps[i as usize] == ENTITYNUM_NONE {
                i += 1;
                continue;
            }
            found = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(foundLaserTraps[i as usize] as usize);
            if !laserTrap.is_null() && (*found).setTime < lowestTimeStamp {
                removeMe = i;
                lowestTimeStamp = (*found).setTime;
            }
            i += 1;
        }
        if removeMe != -1 {
            //remove it... or blow it?
            if (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(foundLaserTraps[removeMe as usize] as usize).is_null() {
                break;
            } else {
                G_FreeEntity((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(foundLaserTraps[removeMe as usize] as usize));
            }
            foundLaserTraps[removeMe as usize] = ENTITYNUM_NONE;
            trapcount -= 1;
        } else {
            break;
        }
    }

    //now make the new one
    CreateLaserTrap(laserTrap, &start, ent);

    //set player-created-specific fields
    (*laserTrap).setTime = (*addr_of!(level)).time; //remember when we placed it

    if alt_fire == QFALSE {
        //tripwire
        (*laserTrap).count = 1;
    }

    //move it
    (*laserTrap).s.pos.trType = TR_GRAVITY;

    if alt_fire != QFALSE {
        VectorScale(&dir, 512.0, &mut (*laserTrap).s.pos.trDelta);
    } else {
        VectorScale(&dir, 256.0, &mut (*laserTrap).s.pos.trDelta);
    }

    trap::LinkEntity(laserTrap);
}

/// `void G_VehMuzzleFireFX( gentity_t *ent, gentity_t *broadcaster, int muzzlesFired )`
/// (g_weapon.c:3799) — "custom routine to not waste tempents horribly -rww". Broadcasts the
/// muzzle-flash FX for a firing vehicle. Bails if `ent` has no `m_pVehicle`. When no
/// `broadcaster` is supplied it spends a temp-entity (`G_TempEntity` at the firer's origin with
/// `EV_VEH_FIRE`) — otherwise it reuses the caller's `broadcaster` entity. Either way it stamps
/// the broadcaster's `s.owner` (this entity owns it) and `s.trickedentindex` (the bitfield of
/// muzzles fired this shot — fits in 16 bits since there are <= MAX_VEHICLE_MUZZLES = 12), and
/// when a `broadcaster` was given, adds the `EV_VEH_FIRE` event explicitly. No oracle
/// (gentity-pointer + temp-entity/event side-effects).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; if `broadcaster` is NULL, `ent->client` must be
/// non-NULL (the C derefs `ent->client->ps.origin`). `level` must be initialised (via
/// `G_TempEntity`).
pub unsafe fn G_VehMuzzleFireFX(
    ent: *mut gentity_t,
    broadcaster: *mut gentity_t,
    muzzlesFired: c_int,
) {
    let pVeh = (*ent).m_pVehicle;
    let b: *mut gentity_t;

    if pVeh.is_null() {
        return;
    }

    if broadcaster.is_null() {
        //oh well. We will WASTE A TEMPENT.
        b = G_TempEntity(&(*(*ent).client).ps.origin, EV_VEH_FIRE);
    } else {
        //joy
        b = broadcaster;
    }

    //this guy owns it
    (*b).s.owner = (*ent).s.number;

    //this is the bitfield of all muzzles fired this time
    //NOTE: just need MAX_VEHICLE_MUZZLES bits for this... should be cool since it's currently 12 and we're sending it in 16 bits
    (*b).s.trickedentindex = muzzlesFired;

    if !broadcaster.is_null() {
        //add the event
        G_AddEvent(b, EV_VEH_FIRE, 0);
    }
}


/// `void thermalDetonatorExplode( gentity_t *ent )` (g_weapon.c:1898) — the two-stage `think` of a
/// thrown thermal detonator. On the first call (`count == 0`) it plays the warning beep, marks
/// itself with `count = 1`, schedules a 500ms fuse in `genericValue5`, hands off to
/// `thermalThinkStandard`, and broadcasts so every client hears/sees the impending blast. On the
/// second call it snaps to its evaluated trajectory origin (+8 up), becomes a generic entity,
/// spits a broadcast `EV_MISSILE_MISS` temp-event facing straight up, marks itself
/// `freeAfterEvent`, and runs the `G_RadiusDamage` splash — crediting the owning client an
/// accuracy hit if anything was struck — then relinks. A `pub unsafe extern "C"` fn for the
/// `gentity_t::think` fn-pointer ABI. No oracle (gentity-state mutation + `G_RadiusDamage` /
/// `G_AddEvent` / `G_Sound` / `trap_LinkEntity` side-effects).
///
/// # Safety
/// `ent` must point to a valid thermal-detonator `gentity_t` whose `r.ownerNum` indexes a client;
/// `level`/`g_entities` must be initialised.
pub unsafe extern "C" fn thermalDetonatorExplode(ent: *mut gentity_t) {
    if (*ent).count == 0 {
        G_Sound(
            ent,
            CHAN_WEAPON,
            G_SoundIndex("sound/weapons/thermal/warning.wav"),
        );
        (*ent).count = 1;
        (*ent).genericValue5 = (*addr_of!(level)).time + 500;
        (*ent).think = Some(thermalThinkStandard);
        (*ent).nextthink = (*addr_of!(level)).time;
        (*ent).r.svFlags |= SVF_BROADCAST; //so everyone hears/sees the explosion?
    } else {
        let mut origin: vec3_t = [0.0; 3];
        let dir: vec3_t = [0.0, 0.0, 1.0];

        BG_EvaluateTrajectory(&(*ent).s.pos, (*addr_of!(level)).time, &mut origin);
        origin[2] += 8.0;
        trap::SnapVector(&mut origin);
        G_SetOrigin(ent, &origin);

        (*ent).s.eType = ET_GENERAL;
        G_AddEvent(ent, EV_MISSILE_MISS, DirToByte(&dir));
        (*ent).freeAfterEvent = QTRUE;

        if G_RadiusDamage(
            &(*ent).r.currentOrigin,
            (*ent).parent,
            (*ent).splashDamage as f32,
            (*ent).splashRadius as f32,
            ent,
            ent,
            (*ent).splashMethodOfDeath,
        ) != QFALSE
        {
            (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize)).client).accuracy_hits += 1;
        }

        trap::LinkEntity(ent);
    }
}

// Thermal detonator tunables (g_weapon.c:1880-1885). The alt-fire (`TD_ALT_*`) variants are
// declared incrementally as their reader lands.
const TD_DAMAGE: c_int = 70; //only do 70 on a direct impact
const TD_SPLASH_RAD: c_int = 128;
const TD_SPLASH_DAM: c_int = 90;
const TD_VELOCITY: c_int = 900;
const TD_MIN_CHARGE: f32 = 0.15;
const TD_TIME: c_int = 3000; //6000

/// `void thermalThinkStandard( gentity_t *ent )` (g_weapon.c:1934) — the per-frame `think`
/// callback for a thrown thermal detonator. Once its fuse (`genericValue5`) has elapsed, it
/// hands off to [`thermalDetonatorExplode`] for the next think (which detonates); otherwise it
/// runs the shared thrown-object physics ([`G_RunObject`]) and re-arms for the next frame. A `pub
/// unsafe extern "C"` fn for the `gentity_t::think` fn-pointer ABI. No oracle (entity-state
/// mutation + `G_RunObject`/think-pointer side-effects).
///
/// # Safety
/// `ent` must point to a valid thermal-detonator `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn thermalThinkStandard(ent: *mut gentity_t) {
    if (*ent).genericValue5 < (*addr_of!(level)).time {
        (*ent).think = Some(thermalDetonatorExplode);
        (*ent).nextthink = (*addr_of!(level)).time;
        return;
    }

    G_RunObject(ent);
    (*ent).nextthink = (*addr_of!(level)).time;
}

/// `gentity_t *WP_FireThermalDetonator( gentity_t *ent, qboolean altFire )` (g_weapon.c:1948) —
/// spawn and launch a thermal detonator from `ent`. Aims along the file-static `forward`,
/// starting at `muzzle` (pulled back through walls by `W_TraceSetStart`), spins up a
/// physics-object `gentity_t` whose per-frame `think` is `thermalThinkStandard` and whose fuse
/// (`genericValue5`) fires `TD_TIME` ms out. The throw speed scales with the client's hold/charge
/// time clamped to `[TD_MIN_CHARGE, 1.0]`, the live throw gets a +120 vertical kick, and the
/// non-alt throw bounces (`FL_BOUNCE_HALF`). Configures the missile's damage/splash, sound loop,
/// `ET_MISSILE` state and `MOD_THERMAL[_SPLASH]` means-of-death, snaps its velocity to save net
/// bandwidth, and returns the new bolt. No oracle (`G_Spawn` allocation + entity-state/trap
/// side-effects; reads the file-statics `forward`/`muzzle`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `ent->client` is NULL-checked before reading the
/// charge time. `g_entities`/`level` must be initialised so `G_Spawn` can allocate. Reads the
/// file-statics `forward`/`muzzle`.
pub unsafe fn WP_FireThermalDetonator(ent: *mut gentity_t, altFire: qboolean) -> *mut gentity_t {
    let bolt: *mut gentity_t;
    let mut dir: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut chargeAmount: f32 = 1.0; // default of full charge

    VectorCopy(&*addr_of!(forward), &mut dir);
    VectorCopy(&*addr_of!(muzzle), &mut start);

    bolt = G_Spawn();

    (*bolt).physicsObject = QTRUE;

    (*bolt).classname = c"thermal_detonator".as_ptr() as *mut c_char;
    (*bolt).think = Some(thermalThinkStandard);
    (*bolt).nextthink = (*addr_of!(level)).time;
    (*bolt).touch = Some(touch_NULL);

    // How 'bout we give this thing a size...
    VectorSet(&mut (*bolt).r.mins, -3.0, -3.0, -3.0);
    VectorSet(&mut (*bolt).r.maxs, 3.0, 3.0, 3.0);
    (*bolt).clipmask = MASK_SHOT;

    W_TraceSetStart(ent, &mut start, &(*bolt).r.mins, &(*bolt).r.maxs); //make sure our start point isn't on the other side of a wall

    if !(*ent).client.is_null() {
        chargeAmount = ((*addr_of!(level)).time - (*(*ent).client).ps.weaponChargeTime) as f32;
    }

    // get charge amount
    chargeAmount = chargeAmount / TD_VELOCITY as f32;

    if chargeAmount > 1.0 {
        chargeAmount = 1.0;
    } else if chargeAmount < TD_MIN_CHARGE {
        chargeAmount = TD_MIN_CHARGE;
    }

    // normal ones bounce, alt ones explode on impact
    (*bolt).genericValue5 = (*addr_of!(level)).time + TD_TIME; // How long 'til she blows
    (*bolt).s.pos.trType = TR_GRAVITY;
    (*bolt).parent = ent;
    (*bolt).r.ownerNum = (*ent).s.number;
    VectorScale(&dir, TD_VELOCITY as f32 * chargeAmount, &mut (*bolt).s.pos.trDelta);

    if (*ent).health >= 0 {
        (*bolt).s.pos.trDelta[2] += 120.0;
    }

    if altFire == QFALSE {
        (*bolt).flags |= FL_BOUNCE_HALF;
    }

    (*bolt).s.loopSound = G_SoundIndex("sound/weapons/thermal/thermloop.wav");
    (*bolt).s.loopIsSoundset = QFALSE;

    (*bolt).damage = TD_DAMAGE;
    (*bolt).dflags = 0;
    (*bolt).splashDamage = TD_SPLASH_DAM;
    (*bolt).splashRadius = TD_SPLASH_RAD;

    (*bolt).s.eType = ET_MISSILE;
    (*bolt).r.svFlags = SVF_USE_CURRENT_ORIGIN;
    (*bolt).s.weapon = WP_THERMAL;

    (*bolt).methodOfDeath = MOD_THERMAL;
    (*bolt).splashMethodOfDeath = MOD_THERMAL_SPLASH;

    (*bolt).s.pos.trTime = (*addr_of!(level)).time; // move a bit on the very first frame
    VectorCopy(&start, &mut (*bolt).s.pos.trBase);

    trap::SnapVector(&mut (*bolt).s.pos.trDelta); // save net bandwidth
    VectorCopy(&start, &mut (*bolt).r.currentOrigin);

    VectorCopy(&start, &mut (*bolt).pos2);

    (*bolt).bounceCount = -5;

    bolt
}

/// `gentity_t *WP_DropThermal( gentity_t *ent )` (g_weapon.c:2036) — drop a thermal detonator:
/// recompute the file-static aim axes (`forward`/`vright`/`up`) from the firer's viewangles and
/// hand off to the non-alt [`WP_FireThermalDetonator`]. No oracle (delegates to entity-state
/// helpers; writes the file-statics).
///
/// # Safety
/// `ent->client` must be non-NULL (dereferenced for `ps.viewangles`). Writes the file-statics
/// `forward`/`vright`/`up`.
pub unsafe fn WP_DropThermal(ent: *mut gentity_t) -> *mut gentity_t {
    AngleVectors(
        &(*(*ent).client).ps.viewangles,
        Some(&mut *addr_of_mut!(forward)),
        Some(&mut *addr_of_mut!(vright)),
        Some(&mut *addr_of_mut!(up)),
    );
    WP_FireThermalDetonator(ent, QFALSE)
}

/// `void FireVehicleWeapon( gentity_t *ent, qboolean alt_fire )` (g_weapon.c:4044) — fire the
/// vehicle's primary (`!alt_fire`) or alternate (`alt_fire`) weapon. Bails if the vehicle is
/// breaking apart or (for a walker) being electrocuted. Fighters can only fire with wings open.
/// Walks the muzzles that match the selected weapon, either firing all of them at once (linked
/// firing) or one at a time (round-robin via `nextMuzzle`), each via [`WP_FireVehicleWeapon`]
/// (after recomputing the muzzle via [`WP_CalcVehMuzzle`] and optionally auto-aiming via
/// [`WP_VehCheckTraceFromCamPos`] or a crosshair trace). Tracks ammo/delay (cumulative for
/// linked firing), mirrors the ammo into the vehicle NPC's first ammo slots, clears the rocket
/// lock after a homing shot, and plays the muzzle FX via [`G_VehMuzzleFireFX`]. No oracle
/// (gentity/vehicle/trap state).
///
/// # Safety
/// `ent` must point to a valid vehicle `gentity_t` with a valid `ent->client`; `level`/
/// `g_vehWeaponInfo` must be initialised.
pub unsafe fn FireVehicleWeapon(ent: *mut gentity_t, alt_fire: qboolean) {
    let pVeh: *mut Vehicle_t = (*ent).m_pVehicle;
    let mut muzzlesFired: c_int = 0;
    let mut missile: *mut gentity_t = null_mut();
    let mut vehWeapon: *mut vehWeaponInfo_t = null_mut();
    let mut clearRocketLockEntity: qboolean = QFALSE;

    if pVeh.is_null() {
        return;
    }

    if (*pVeh).m_iRemovedSurfaces != 0 {
        //can't fire when the thing is breaking apart
        return;
    }

    if (*(*pVeh).m_pVehicleInfo).r#type == VH_WALKER
        && (*(*ent).client).ps.electrifyTime > (*addr_of!(level)).time
    {
        //don't fire while being electrocuted
        return;
    }

    // TODO?: If possible (probably not enough time), it would be nice if secondary fire was actually a mode switch/toggle
    // so that, for instance, an x-wing can have 4-gun fire, or individual muzzle fire. If you wanted a different weapon, you
    // would actually have to press the 2 key or something like that (I doubt I'd get a graphic for it anyways though). -AReis

    // If this is not the alternate fire, fire a normal blaster shot...
    if !(*pVeh).m_pVehicleInfo.is_null()
        && ((*(*pVeh).m_pVehicleInfo).r#type != VH_FIGHTER
            || ((*pVeh).m_ulFlags & VEH_WINGSOPEN as c_ulong) != 0)
    {
        //fighters can only fire when wings are open
        // NOTE: Wings open also denotes that it has already launched.
        let weaponNum: usize;
        let vehWeaponIndex: c_int;
        let delay: c_int; // C: int delay = 1000; (unconditionally overwritten below)
        let aimCorrect: qboolean;
        let mut linkedFiring: qboolean = QFALSE;

        if alt_fire == QFALSE {
            weaponNum = 0;
        } else {
            weaponNum = 1;
        }

        vehWeaponIndex = (*(*pVeh).m_pVehicleInfo).weapon[weaponNum].ID;

        if (*pVeh).weaponStatus[weaponNum].ammo <= 0 {
            //no ammo for this weapon
            if !(*pVeh).m_pPilot.is_null()
                && (*((*pVeh).m_pPilot as *mut gentity_t)).s.number < MAX_CLIENTS as c_int
            {
                // let the client know he's out of ammo
                //but only if one of the vehicle muzzles is actually ready to fire this weapon
                for i in 0..MAX_VEHICLE_MUZZLES {
                    if (*(*pVeh).m_pVehicleInfo).weapMuzzle[i] != vehWeaponIndex {
                        //this muzzle doesn't match the weapon we're trying to use
                        continue;
                    }
                    if (*pVeh).m_iMuzzleTag[i] != -1
                        && (*pVeh).m_iMuzzleWait[i] < (*addr_of!(level)).time
                    {
                        //this one would have fired, send the no ammo message
                        G_AddEvent(
                            (*pVeh).m_pPilot as *mut gentity_t,
                            EV_NOAMMO,
                            weaponNum as c_int,
                        );
                        break;
                    }
                }
            }
            return;
        }

        delay = (*(*pVeh).m_pVehicleInfo).weapon[weaponNum].delay;
        aimCorrect = (*(*pVeh).m_pVehicleInfo).weapon[weaponNum].aimCorrect;
        if (*(*pVeh).m_pVehicleInfo).weapon[weaponNum].linkable == 2//always linked
            || ((*(*pVeh).m_pVehicleInfo).weapon[weaponNum].linkable == 1//optionally linkable
                 && (*pVeh).weaponStatus[weaponNum].linked != QFALSE)
        {
            //linked
            //we're linking the primary or alternate weapons, so we'll do *all* the muzzles
            linkedFiring = QTRUE;
        }

        if vehWeaponIndex <= VEH_WEAPON_BASE || vehWeaponIndex >= MAX_VEH_WEAPONS as c_int {
            //invalid vehicle weapon
            return;
        } else {
            let mut numMuzzles: c_int = 0;
            let mut numMuzzlesReady: c_int = 0;
            let mut cumulativeDelay: c_int = 0;
            let mut cumulativeAmmo: c_int = 0;
            let mut sentAmmoWarning: qboolean = QFALSE;

            vehWeapon = &mut g_vehWeaponInfo[vehWeaponIndex as usize];

            if (*(*pVeh).m_pVehicleInfo).weapon[weaponNum].linkable == 2 {
                //always linked weapons don't accumulate delay, just use specified delay
                cumulativeDelay = delay;
            }
            //find out how many we've got for this weapon
            for i in 0..MAX_VEHICLE_MUZZLES {
                if (*(*pVeh).m_pVehicleInfo).weapMuzzle[i] != vehWeaponIndex {
                    //this muzzle doesn't match the weapon we're trying to use
                    continue;
                }
                if (*pVeh).m_iMuzzleTag[i] != -1
                    && (*pVeh).m_iMuzzleWait[i] < (*addr_of!(level)).time
                {
                    numMuzzlesReady += 1;
                }
                if (*(*pVeh).m_pVehicleInfo).weapMuzzle
                    [(*pVeh).weaponStatus[weaponNum].nextMuzzle as usize]
                    != vehWeaponIndex
                {
                    //Our designated next muzzle for this weapon isn't valid for this weapon (happens when ships fire for the first time)
                    //set the next to this one
                    (*pVeh).weaponStatus[weaponNum].nextMuzzle = i as c_int;
                }
                if linkedFiring != QFALSE {
                    cumulativeAmmo += (*vehWeapon).iAmmoPerShot;
                    if (*(*pVeh).m_pVehicleInfo).weapon[weaponNum].linkable != 2 {
                        //always linked weapons don't accumulate delay, just use specified delay
                        cumulativeDelay += delay;
                    }
                }
                numMuzzles += 1;
            }

            if linkedFiring != QFALSE {
                //firing all muzzles at once
                if numMuzzlesReady != numMuzzles {
                    //can't fire all linked muzzles yet
                    return;
                } else {
                    //can fire all linked muzzles, check ammo
                    if (*pVeh).weaponStatus[weaponNum].ammo < cumulativeAmmo {
                        //can't fire, not enough ammo
                        if !(*pVeh).m_pPilot.is_null()
                            && (*((*pVeh).m_pPilot as *mut gentity_t)).s.number < MAX_CLIENTS as c_int
                        {
                            // let the client know he's out of ammo
                            G_AddEvent(
                                (*pVeh).m_pPilot as *mut gentity_t,
                                EV_NOAMMO,
                                weaponNum as c_int,
                            );
                        }
                        return;
                    }
                }
            }

            'muzzleLoop: for i in 0..MAX_VEHICLE_MUZZLES {
                if (*(*pVeh).m_pVehicleInfo).weapMuzzle[i] != vehWeaponIndex {
                    //this muzzle doesn't match the weapon we're trying to use
                    continue;
                }
                if linkedFiring == QFALSE
                    && i as c_int != (*pVeh).weaponStatus[weaponNum].nextMuzzle
                {
                    //we're only firing one muzzle and this isn't it
                    continue;
                }

                // Fire this muzzle.
                if (*pVeh).m_iMuzzleTag[i] != -1
                    && (*pVeh).m_iMuzzleWait[i] < (*addr_of!(level)).time
                {
                    let mut start: vec3_t = [0.0; 3];
                    let mut dir: vec3_t = [0.0; 3];

                    if (*pVeh).weaponStatus[weaponNum].ammo < (*vehWeapon).iAmmoPerShot {
                        //out of ammo!
                        if sentAmmoWarning == QFALSE {
                            sentAmmoWarning = QTRUE;
                            if !(*pVeh).m_pPilot.is_null()
                                && (*((*pVeh).m_pPilot as *mut gentity_t)).s.number < MAX_CLIENTS as c_int
                            {
                                // let the client know he's out of ammo
                                G_AddEvent(
                                    (*pVeh).m_pPilot as *mut gentity_t,
                                    EV_NOAMMO,
                                    weaponNum as c_int,
                                );
                            }
                        }
                    } else {
                        //have enough ammo to shoot
                        //do the firing
                        WP_CalcVehMuzzle(ent, i as c_int);
                        VectorCopy(&(*pVeh).m_vMuzzlePos[i], &mut start);
                        VectorCopy(&(*pVeh).m_vMuzzleDir[i], &mut dir);
                        if WP_VehCheckTraceFromCamPos(ent, &start, &mut dir) != QFALSE {
                            //auto-aim at whatever crosshair would be over from camera's point of view (if closer)
                        } else if aimCorrect != QFALSE {
                            //auto-aim the missile at the crosshair if there's anything there
                            let mut end: vec3_t = [0.0; 3];
                            let mut ang: vec3_t = [0.0; 3];

                            if (*(*pVeh).m_pVehicleInfo).r#type == VH_SPEEDER {
                                VectorSet(&mut ang, 0.0, *(*pVeh).m_vOrientation.add(1), 0.0);
                            } else {
                                VectorCopy(&*(*pVeh).m_vOrientation.cast::<vec3_t>(), &mut ang);
                            }
                            AngleVectors(&ang, Some(&mut dir), None, None);
                            VectorMA(&(*ent).r.currentOrigin, 32768.0, &dir, &mut end);
                            //VectorMA( ent->r.currentOrigin, 8192, dir, end );
                            let trace: trace_t = trap::Trace(
                                &(*ent).r.currentOrigin,
                                &vec3_origin,
                                &vec3_origin,
                                &end,
                                (*ent).s.number,
                                MASK_SHOT,
                            );
                            if trace.fraction < 1.0
                                && trace.allsolid == 0
                                && trace.startsolid == 0
                            {
                                VectorSubtract(&trace.endpos, &start, &mut dir);
                                VectorNormalize(&mut dir);
                            }
                        }

                        //play the weapon's muzzle effect if we have one
                        //NOTE: just need MAX_VEHICLE_MUZZLES bits for this... should be cool since it's currently 12 and we're sending it in 16 bits
                        muzzlesFired |= 1 << i;

                        missile =
                            WP_FireVehicleWeapon(ent, &mut start, &dir, vehWeapon, alt_fire, QFALSE);
                        if (*vehWeapon).fHoming != 0.0 {
                            //clear the rocket lock entity *after* all muzzles have fired
                            clearRocketLockEntity = QTRUE;
                        }
                    }

                    if linkedFiring != QFALSE {
                        //we're linking the weapon, so continue on and fire all appropriate muzzles
                        continue;
                    }
                    //else just firing one
                    //take the ammo, set the next muzzle and set the delay on it
                    if numMuzzles > 1 {
                        //more than one, look for it
                        let mut nextMuzzle = (*pVeh).weaponStatus[weaponNum].nextMuzzle;
                        loop {
                            nextMuzzle += 1;
                            if nextMuzzle >= MAX_VEHICLE_MUZZLES as c_int {
                                nextMuzzle = 0;
                            }
                            if nextMuzzle == (*pVeh).weaponStatus[weaponNum].nextMuzzle {
                                //WTF?  Wrapped without finding another valid one!
                                break;
                            }
                            if (*(*pVeh).m_pVehicleInfo).weapMuzzle[nextMuzzle as usize]
                                == vehWeaponIndex
                            {
                                //this is the next muzzle for this weapon
                                (*pVeh).weaponStatus[weaponNum].nextMuzzle = nextMuzzle;
                                break;
                            }
                        }
                    } //else, just stay on the one we just fired
                    //set the delay on the next muzzle
                    (*pVeh).m_iMuzzleWait
                        [(*pVeh).weaponStatus[weaponNum].nextMuzzle as usize] =
                        (*addr_of!(level)).time + delay;
                    //take away the ammo
                    (*pVeh).weaponStatus[weaponNum].ammo -= (*vehWeapon).iAmmoPerShot;
                    //NOTE: in order to send the vehicle's ammo info to the client, we copy the ammo into the first 2 ammo slots on the vehicle NPC's client->ps.ammo array
                    if !(*pVeh).m_pParentEntity.is_null()
                        && !(*((*pVeh).m_pParentEntity as *mut gentity_t)).client.is_null()
                    {
                        (*(*((*pVeh).m_pParentEntity as *mut gentity_t)).client).ps.ammo
                            [weaponNum] = (*pVeh).weaponStatus[weaponNum].ammo;
                    }
                    //done!
                    //we'll get in here again next frame and try the next muzzle...
                    //return;
                    break 'muzzleLoop;
                }
            }
            //we went through all the muzzles, so apply the cumulative delay and ammo cost
            if cumulativeAmmo != 0 {
                //taking ammo one shot at a time
                //take the ammo
                (*pVeh).weaponStatus[weaponNum].ammo -= cumulativeAmmo;
                //NOTE: in order to send the vehicle's ammo info to the client, we copy the ammo into the first 2 ammo slots on the vehicle NPC's client->ps.ammo array
                if !(*pVeh).m_pParentEntity.is_null()
                    && !(*((*pVeh).m_pParentEntity as *mut gentity_t)).client.is_null()
                {
                    (*(*((*pVeh).m_pParentEntity as *mut gentity_t)).client).ps.ammo[weaponNum] =
                        (*pVeh).weaponStatus[weaponNum].ammo;
                }
            }
            if cumulativeDelay != 0 {
                //we linked muzzles so we need to apply the cumulative delay now, to each of the linked muzzles
                for i in 0..MAX_VEHICLE_MUZZLES {
                    if (*(*pVeh).m_pVehicleInfo).weapMuzzle[i] != vehWeaponIndex {
                        //this muzzle doesn't match the weapon we're trying to use
                        continue;
                    }
                    //apply the cumulative delay
                    (*pVeh).m_iMuzzleWait[i] = (*addr_of!(level)).time + cumulativeDelay;
                }
            }
        }
    }

    // tryFire:
    if clearRocketLockEntity != QFALSE {
        //hmm, should probably clear that anytime any weapon fires?
        (*(*ent).client).ps.rocketLockIndex = ENTITYNUM_NONE;
        (*(*ent).client).ps.rocketLockTime = 0.0;
        (*(*ent).client).ps.rocketTargetTime = 0.0;
    }

    if !vehWeapon.is_null() && muzzlesFired > 0 {
        G_VehMuzzleFireFX(ent, missile, muzzlesFired);
    }
}

/// `void FireWeapon( gentity_t *ent, qboolean altFire )` (g_weapon.c:4350) — the per-shot
/// weapon-fire dispatcher. Sets the vestigial `s_quadFactor` (quad multiplier or 1), bumps
/// the firer's `accuracy_shots` (skipping the saber/stun-baton/melee non-tracked weapons;
/// the flechette counts as `FLECHETTE_SHOTS`), then either routes a vehicle firer to
/// [`FireVehicleWeapon`] or computes the per-shot aim axes (`forward`/`vright`/`up`) and
/// `muzzle` and switches on `ent->s.weapon` to the matching `WP_Fire*` helper. The aim setup
/// has three cases: an emplaced gun (muzzle from the gun's view-capped angles via
/// [`BG_EmplacedView`]), a blaster fired while riding a vehicle (yaw swung ±90° on
/// strafe), or the plain view-angle case. No oracle (dispatcher: file-static aim + trap/
/// gentity side-effects).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a valid `ent->client` (deref'd unconditionally,
/// as in C). `g_entities` must be initialised. Writes the file-statics
/// `forward`/`vright`/`up`/`muzzle`/`s_quadFactor`.
pub unsafe fn FireWeapon(ent: *mut gentity_t, altFire: qboolean) {
    if (*(*ent).client).ps.powerups[PW_QUAD as usize] != 0 {
        s_quadFactor = g_quadfactor.value;
    } else {
        s_quadFactor = 1.0;
    }

    // track shots taken for accuracy tracking.  Grapple is not a weapon and gauntet is just not tracked
    if (*ent).s.weapon != WP_SABER
        && (*ent).s.weapon != WP_STUN_BATON
        && (*ent).s.weapon != WP_MELEE
    {
        if (*ent).s.weapon == WP_FLECHETTE {
            (*(*ent).client).accuracy_shots += FLECHETTE_SHOTS;
        } else {
            (*(*ent).client).accuracy_shots += 1;
        }
    }

    if !ent.is_null() && !(*ent).client.is_null() && (*(*ent).client).NPC_class == CLASS_VEHICLE {
        FireVehicleWeapon(ent, altFire);
        return;
    } else {
        // set aiming directions
        if (*ent).s.weapon == WP_EMPLACED_GUN && (*(*ent).client).ps.emplacedIndex != 0 {
            //if using emplaced then base muzzle point off of gun position/angles
            let emp: *mut gentity_t =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.emplacedIndex as usize);

            if (*emp).inuse != QFALSE {
                let mut yaw: f32 = 0.0;
                let mut viewAngCap: vec3_t = [0.0; 3];
                let r#override: c_int;

                VectorCopy(&(*(*ent).client).ps.viewangles, &mut viewAngCap);
                if viewAngCap[PITCH] > 40.0 {
                    viewAngCap[PITCH] = 40.0;
                }

                r#override = BG_EmplacedView(
                    &(*(*ent).client).ps.viewangles,
                    &(*emp).s.angles,
                    &mut yaw,
                    (*emp).s.origin2[0],
                );

                if r#override != 0 {
                    viewAngCap[YAW] = yaw;
                }

                AngleVectors(
                    &viewAngCap,
                    Some(&mut *addr_of_mut!(forward)),
                    Some(&mut *addr_of_mut!(vright)),
                    Some(&mut *addr_of_mut!(up)),
                );
            } else {
                AngleVectors(
                    &(*(*ent).client).ps.viewangles,
                    Some(&mut *addr_of_mut!(forward)),
                    Some(&mut *addr_of_mut!(vright)),
                    Some(&mut *addr_of_mut!(up)),
                );
            }
        } else if (*ent).s.number < MAX_CLIENTS as c_int
            && (*(*ent).client).ps.m_iVehicleNum != 0
            && (*ent).s.weapon == WP_BLASTER
        {
            //riding a vehicle...with blaster selected
            let mut vehTurnAngles: vec3_t = [0.0; 3];
            let vehEnt: *mut gentity_t =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.m_iVehicleNum as usize);

            if (*vehEnt).inuse != QFALSE
                && !(*vehEnt).client.is_null()
                && !(*vehEnt).m_pVehicle.is_null()
            {
                VectorCopy(
                    &*((*(*vehEnt).m_pVehicle).m_vOrientation as *const vec3_t),
                    &mut vehTurnAngles,
                );
                vehTurnAngles[PITCH] = (*(*ent).client).ps.viewangles[PITCH];
            } else {
                VectorCopy(&(*(*ent).client).ps.viewangles, &mut vehTurnAngles);
            }
            if (*(*ent).client).pers.cmd.rightmove > 0 {
                //shooting to right
                vehTurnAngles[YAW] -= 90.0;
            } else if (*(*ent).client).pers.cmd.rightmove < 0 {
                //shooting to left
                vehTurnAngles[YAW] += 90.0;
            }

            AngleVectors(
                &vehTurnAngles,
                Some(&mut *addr_of_mut!(forward)),
                Some(&mut *addr_of_mut!(vright)),
                Some(&mut *addr_of_mut!(up)),
            );
        } else {
            AngleVectors(
                &(*(*ent).client).ps.viewangles,
                Some(&mut *addr_of_mut!(forward)),
                Some(&mut *addr_of_mut!(vright)),
                Some(&mut *addr_of_mut!(up)),
            );
        }

        CalcMuzzlePoint(
            ent,
            &*addr_of!(forward),
            &*addr_of!(vright),
            &*addr_of!(up),
            &mut *addr_of_mut!(muzzle),
        );

        // fire the specific weapon
        match (*ent).s.weapon {
            x if x == WP_STUN_BATON => {
                WP_FireStunBaton(ent, altFire);
            }

            x if x == WP_MELEE => {
                WP_FireMelee(ent, altFire);
            }

            x if x == WP_SABER => {}

            x if x == WP_BRYAR_PISTOL => {
                //if ( g_gametype.integer == GT_SIEGE )
                if 1 != 0 {
                    //allow alt-fire
                    WP_FireBryarPistol(ent, altFire);
                } else {
                    WP_FireBryarPistol(ent, QFALSE);
                }
            }

            x if x == WP_CONCUSSION => {
                if altFire != QFALSE {
                    WP_FireConcussionAlt(ent);
                } else {
                    WP_FireConcussion(ent);
                }
            }

            x if x == WP_BRYAR_OLD => {
                WP_FireBryarPistol(ent, altFire);
            }

            x if x == WP_BLASTER => {
                WP_FireBlaster(ent, altFire);
            }

            x if x == WP_DISRUPTOR => {
                WP_FireDisruptor(ent, altFire);
            }

            x if x == WP_BOWCASTER => {
                WP_FireBowcaster(ent, altFire);
            }

            x if x == WP_REPEATER => {
                WP_FireRepeater(ent, altFire);
            }

            x if x == WP_DEMP2 => {
                WP_FireDEMP2(ent, altFire);
            }

            x if x == WP_FLECHETTE => {
                WP_FireFlechette(ent, altFire);
            }

            x if x == WP_ROCKET_LAUNCHER => {
                WP_FireRocket(ent, altFire);
            }

            x if x == WP_THERMAL => {
                WP_FireThermalDetonator(ent, altFire);
            }

            x if x == WP_TRIP_MINE => {
                WP_PlaceLaserTrap(ent, altFire);
            }

            x if x == WP_DET_PACK => {
                WP_DropDetPack(ent, altFire);
            }

            x if x == WP_EMPLACED_GUN => {
                if !(*ent).client.is_null() && (*(*ent).client).ewebIndex != 0 {
                    //specially handled by the e-web itself
                    // break;
                } else {
                    WP_FireEmplaced(ent, altFire);
                }
            }
            _ => {
                //			assert(!"unknown weapon fire");
            }
        }
    }

    G_LogWeaponFire((*ent).s.number, (*ent).s.weapon);
}

/// `static void WP_FireEmplaced( gentity_t *ent, qboolean altFire )` (g_weapon.c:4537) — fire
/// the emplaced gun the client `ent` is mounted on. Bails unless `ent->client` and its
/// `ps.emplacedIndex` are set, and the referenced gun is in-use and alive. Builds the muzzle
/// point at the gun's origin (+46 up), then offsets 10u along the firer's view-right onto the
/// alternating cannon barrel: when `gun->genericValue10` is set it fires the *right* side
/// (+right, `side = 0`), else the *left* (-right, `side = 1`); the new `side` is stored back into
/// `genericValue10` so the next shot alternates, and an `EV_FIRE_WEAPON` event carries `side` to
/// the renderer. The shot direction comes from the file-static `forward` (via `vectoangles` →
/// `AngleVectors`), and the bolt is spawned by `WP_FireEmplacedMissile` (with `ent` as the
/// pass-through ignore). No oracle (gentity-pointer + event/missile-spawn side-effects; reads the
/// file-static `forward`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `ent->client` is NULL-checked. `g_entities` must be
/// initialised. Reads the file-static `forward`.
pub unsafe fn WP_FireEmplaced(ent: *mut gentity_t, altFire: qboolean) {
    let mut dir: vec3_t = [0.0; 3];
    let mut angs: vec3_t = [0.0; 3];
    let mut gunpoint: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let gun: *mut gentity_t;
    let side: c_int;

    if (*ent).client.is_null() {
        return;
    }

    if (*(*ent).client).ps.emplacedIndex == 0 {
        //shouldn't be using WP_EMPLACED_GUN if we aren't on an emplaced weapon
        return;
    }

    gun = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*ent).client).ps.emplacedIndex as usize);

    if (*gun).inuse == QFALSE || (*gun).health <= 0 {
        //gun was removed or killed, although we should never hit this check because we should have been forced off it already
        return;
    }

    VectorCopy(&(*gun).s.origin, &mut gunpoint);
    gunpoint[2] += 46.0;

    AngleVectors(&(*(*ent).client).ps.viewangles, None, Some(&mut right), None);

    if (*gun).genericValue10 != 0 {
        //fire out of the right cannon side
        let base = gunpoint;
        VectorMA(&base, 10.0, &right, &mut gunpoint);
        side = 0;
    } else {
        //the left
        let base = gunpoint;
        VectorMA(&base, -10.0, &right, &mut gunpoint);
        side = 1;
    }

    (*gun).genericValue10 = side;
    G_AddEvent(gun, EV_FIRE_WEAPON, side);

    vectoangles(&*addr_of!(forward), &mut angs);

    AngleVectors(&angs, Some(&mut dir), None, None);

    WP_FireEmplacedMissile(gun, &mut gunpoint, &dir, altFire, ent);
}

/// `void G_EstimateCamPos( vec3_t viewAngles, vec3_t cameraFocusLoc, float viewheight, float
/// thirdPersonRange, float thirdPersonHorzOffset, float vertOffset, float pitchOffset, int
/// ignoreEntNum, vec3_t camPos )` (g_weapon.c:3831).
///
/// Server-side estimate of where the third-person camera would sit, given the firer's view
/// angles and a focus location: pitch-offset and (unless `bg_fighterAltControl` is set) clamp
/// the view pitch, then trace from the focus to the ideal target and from there back along
/// `forward` by `thirdPersonRange`, clamping each step to the first solid hit. The resulting
/// look direction (with a forward fallback when the trace collapses it) gives camera angles,
/// and an optional horizontal offset slides the camera along its right axis. `cameraFocusLoc`
/// is an in/out param (its Z is raised by `viewheight` in place, matching the C). No oracle
/// (ends in `trap_Trace` side-effects; mutates out-params).
///
/// # Safety
/// `bg_fighterAltControl` is a process-global cvar read via `addr_of!`.
pub unsafe fn G_EstimateCamPos(
    viewAngles: &vec3_t,
    cameraFocusLoc: &mut vec3_t,
    viewheight: f32,
    thirdPersonRange: f32,
    thirdPersonHorzOffset: f32,
    vertOffset: f32,
    pitchOffset: f32,
    ignoreEntNum: c_int,
    camPos: &mut vec3_t,
) {
    let MASK_CAMERACLIP: c_int = MASK_SOLID | CONTENTS_PLAYERCLIP;
    let CAMERA_SIZE: f32 = 4.0;
    let mut cameramins: vec3_t = [0.0; 3];
    let mut cameramaxs: vec3_t = [0.0; 3];
    let mut cameraFocusAngles: vec3_t = [0.0; 3];
    let mut camerafwd: vec3_t = [0.0; 3];
    let mut cameraup: vec3_t = [0.0; 3];
    let mut cameraIdealTarget: vec3_t = [0.0; 3];
    let mut cameraCurTarget: vec3_t = [0.0; 3];
    let mut cameraIdealLoc: vec3_t = [0.0; 3];
    let mut cameraCurLoc: vec3_t = [0.0; 3];
    let mut diff: vec3_t = [0.0; 3];
    let mut camAngles: vec3_t = [0.0; 3];
    let mut viewaxis: [vec3_t; 3] = [[0.0; 3]; 3];

    VectorSet(&mut cameramins, -CAMERA_SIZE, -CAMERA_SIZE, -CAMERA_SIZE);
    VectorSet(&mut cameramaxs, CAMERA_SIZE, CAMERA_SIZE, CAMERA_SIZE);

    VectorCopy(viewAngles, &mut cameraFocusAngles);
    cameraFocusAngles[PITCH] += pitchOffset;
    if (*addr_of!(bg_fighterAltControl)).integer == 0 {
        //clamp view pitch
        cameraFocusAngles[PITCH] = AngleNormalize180(cameraFocusAngles[PITCH]);
        if cameraFocusAngles[PITCH] > 80.0 {
            cameraFocusAngles[PITCH] = 80.0;
        } else if cameraFocusAngles[PITCH] < -80.0 {
            cameraFocusAngles[PITCH] = -80.0;
        }
    }
    AngleVectors(
        &cameraFocusAngles,
        Some(&mut camerafwd),
        None,
        Some(&mut cameraup),
    );

    cameraFocusLoc[2] += viewheight;

    VectorCopy(cameraFocusLoc, &mut cameraIdealTarget);
    cameraIdealTarget[2] += vertOffset;

    //NOTE: on cgame, this uses the thirdpersontargetdamp value, we ignore that here
    VectorCopy(&cameraIdealTarget, &mut cameraCurTarget);
    let mut trace = trap::Trace(
        cameraFocusLoc,
        &cameramins,
        &cameramaxs,
        &cameraCurTarget,
        ignoreEntNum,
        MASK_CAMERACLIP,
    );
    if trace.fraction < 1.0 {
        VectorCopy(&trace.endpos, &mut cameraCurTarget);
    }

    VectorMA(
        &cameraIdealTarget,
        -(thirdPersonRange),
        &camerafwd,
        &mut cameraIdealLoc,
    );
    //NOTE: on cgame, this uses the thirdpersoncameradamp value, we ignore that here
    VectorCopy(&cameraIdealLoc, &mut cameraCurLoc);
    trace = trap::Trace(
        &cameraCurTarget,
        &cameramins,
        &cameramaxs,
        &cameraCurLoc,
        ignoreEntNum,
        MASK_CAMERACLIP,
    );
    if trace.fraction < 1.0 {
        VectorCopy(&trace.endpos, &mut cameraCurLoc);
    }

    VectorSubtract(&cameraCurTarget, &cameraCurLoc, &mut diff);
    {
        let dist = VectorNormalize(&mut diff);
        //under normal circumstances, should never be 0.00000 and so on.
        if dist == 0.0 || (diff[0] == 0.0 || diff[1] == 0.0) {
            //must be hitting something, need some value to calc angles, so use cam forward
            VectorCopy(&camerafwd, &mut diff);
        }
    }

    vectoangles(&diff, &mut camAngles);

    if thirdPersonHorzOffset != 0.0f32 {
        AnglesToAxis(&camAngles, &mut viewaxis);
        let cameraCurLocCopy = cameraCurLoc;
        VectorMA(
            &cameraCurLocCopy,
            thirdPersonHorzOffset,
            &viewaxis[1],
            &mut cameraCurLoc,
        );
    }

    VectorCopy(&cameraCurLoc, camPos);
}

/// `void WP_VehWeapSetSolidToOwner( gentity_t *self )` (g_weapon.c:3569) — the spawn-time
/// `think` for a freshly-fired vehicle-weapon missile that flags itself `SVF_OWNERNOTSHARED`
/// so the firing vehicle can collide with / shoot its own projectile. If the missile has a
/// `genericValue1` lifetime it schedules its own expiry that many ms out, choosing the expiry
/// behaviour from `genericValue2`: explode (`G_ExplodeMissile`) when set, otherwise just remove
/// itself (`G_FreeEntity`). A `pub unsafe extern "C"` fn for the `gentity_t::think` fn-pointer
/// ABI. No oracle (entity-state mutation + think/free fn-pointer assignment; makes no computable
/// function calls).
///
/// # Safety
/// `self_` must point to a valid missile `gentity_t`; `level` must be initialised.
pub unsafe extern "C" fn WP_VehWeapSetSolidToOwner(self_: *mut gentity_t) {
    (*self_).r.svFlags |= SVF_OWNERNOTSHARED;
    if (*self_).genericValue1 != 0 {
        //expire after a time
        if (*self_).genericValue2 != 0 {
            //blow up when your lifetime is up
            // `G_ExplodeMissile` is still a Rust-ABI `unsafe fn` (no caller had yet
            // installed it as a `.think` fn-pointer); its single `*mut gentity_t` arg is
            // ABI-identical to the `extern "C"` `think` slot, so transmute it into place.
            (*self_).think = Some(core::mem::transmute::<
                unsafe fn(*mut gentity_t),
                unsafe extern "C" fn(*mut gentity_t),
            >(G_ExplodeMissile)); //FIXME: custom func?
        } else {
            //just remove yourself
            (*self_).think = Some(G_FreeEntity); //FIXME: custom func?
        }
        (*self_).nextthink = (*addr_of!(level)).time + (*self_).genericValue1;
    }
}

/// `void WP_TouchVehMissile( gentity_t *ent, gentity_t *other, trace_t *trace )`
/// (g_weapon.c:3528) — the `touch` callback for a vehicle-weapon mine/missile that does damage
/// only when something touches it. Copies the incoming trace, overrides its `entityNum` to the
/// touched entity (if any), then routes through [`G_MissileImpact`] to apply the impact. A `pub
/// unsafe extern "C"` fn for the `gentity_t::touch` fn-pointer ABI. No oracle (entity-state +
/// trap-driven impact).
///
/// # Safety
/// `ent`/`trace` must be valid; `other` may be null.
pub unsafe extern "C" fn WP_TouchVehMissile(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    trace: *mut trace_t,
) {
    let mut myTrace: trace_t = *trace;
    if !other.is_null() {
        myTrace.entityNum = (*other).s.number as i16;
    }
    G_MissileImpact(ent, &mut myTrace);
}

const VEH_HOMING_MISSILE_THINK_TIME: c_int = 100; // g_weapon.c:3586

/// `gentity_t *WP_FireVehicleWeapon( gentity_t *ent, vec3_t start, vec3_t dir, vehWeaponInfo_t
/// *vehWeapon, qboolean alt_fire, qboolean isTurretWeap )` (g_weapon.c:3587) — spawns and
/// configures a vehicle-weapon projectile from `vehWeapon`'s data (damage/splash, bbox, gravity,
/// ion, health, lifetime, homing, mine-mode). Sets up the missile's `think`/`touch`/`die`
/// callbacks per the weapon's behaviour, owner, and homing-lock state, then returns it. The
/// traceline (non-projectile) branch is `//FIXME: implement` in the original, so it returns the
/// null missile. No oracle (entity-state mutation, trap/fn-pointer driven, reads the
/// `g_vehWeaponInfo` table + `g_entities`).
///
/// # Safety
/// `ent` must point to a valid vehicle `gentity_t`; `vehWeapon` may be null (handled);
/// `level`/`g_entities`/`g_vehWeaponInfo` must be initialised.
#[allow(unused_assignments)] // faithful: C's `int dif = 0;` initial store is dead (both branches reassign before read)
pub unsafe fn WP_FireVehicleWeapon(
    ent: *mut gentity_t,
    start: &mut vec3_t,
    dir: &vec3_t,
    vehWeapon: *mut vehWeaponInfo_t,
    alt_fire: qboolean,
    isTurretWeap: qboolean,
) -> *mut gentity_t {
    let mut missile: *mut gentity_t = null_mut();

    //FIXME: add some randomness...?  Inherent inaccuracy stat of weapon?  Pilot skill?
    if vehWeapon.is_null() {
        //invalid vehicle weapon
        return null_mut();
    } else if (*vehWeapon).bIsProjectile != QFALSE {
        //projectile entity
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];

        VectorSet(
            &mut maxs,
            (*vehWeapon).fWidth / 2.0f32,
            (*vehWeapon).fWidth / 2.0f32,
            (*vehWeapon).fHeight / 2.0f32,
        );
        VectorScale(&maxs, -1.0, &mut mins);

        //make sure our start point isn't on the other side of a wall
        WP_TraceSetStart(ent, start, &mins, &maxs);

        //FIXME: CUSTOM MODEL?
        //QUERY: alt_fire true or not?  Does it matter?
        missile = CreateMissile(start, dir, (*vehWeapon).fSpeed, 10000, ent, QFALSE);

        (*missile).classname = c"vehicle_proj".as_ptr() as *mut c_char;

        (*missile).s.genericenemyindex = (*ent).s.number + MAX_GENTITIES as c_int;
        (*missile).damage = (*vehWeapon).iDamage;
        (*missile).splashDamage = (*vehWeapon).iSplashDamage;
        (*missile).splashRadius = (*vehWeapon).fSplashRadius as c_int;

        //FIXME: externalize some of these properties?
        (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
        (*missile).clipmask = MASK_SHOT;
        //Maybe by checking flags...?
        if (*vehWeapon).bSaberBlockable != QFALSE {
            (*missile).clipmask |= CONTENTS_LIGHTSABER;
        }
        /*
        if ( (vehWeapon->iFlags&VWF_KNOCKBACK) )
        {
            missile->dflags &= ~DAMAGE_DEATH_KNOCKBACK;
        }
        if ( (vehWeapon->iFlags&VWF_RADAR) )
        {
            missile->s.eFlags |= EF_RADAROBJECT;
        }
        */
        // Make it easier to hit things
        VectorCopy(&mins, &mut (*missile).r.mins);
        VectorCopy(&maxs, &mut (*missile).r.maxs);
        //some slightly different stuff for things with bboxes
        if (*vehWeapon).fWidth != 0.0 || (*vehWeapon).fHeight != 0.0 {
            //we assume it's a rocket-like thing
            (*missile).s.weapon = WP_ROCKET_LAUNCHER; //does this really matter?
            (*missile).methodOfDeath = MOD_VEHICLE; //MOD_ROCKET;
            (*missile).splashMethodOfDeath = MOD_VEHICLE; //MOD_ROCKET;// ?SPLASH;

            // we don't want it to ever bounce
            (*missile).bounceCount = 0;

            (*missile).mass = 10.0;
        } else {
            //a blaster-laser-like thing
            (*missile).s.weapon = WP_BLASTER; //does this really matter?
            (*missile).methodOfDeath = MOD_VEHICLE; //count as a heavy weap
            (*missile).splashMethodOfDeath = MOD_VEHICLE; // ?SPLASH;
            // we don't want it to bounce forever
            (*missile).bounceCount = 8;
        }

        if (*vehWeapon).bHasGravity != QFALSE {
            //TESTME: is this all we need to do?
            (*missile).s.weapon = WP_THERMAL; //does this really matter?
            (*missile).s.pos.trType = TR_GRAVITY;
        }

        if (*vehWeapon).bIonWeapon != QFALSE {
            //so it disables ship shields and sends them out of control
            (*missile).s.weapon = WP_DEMP2;
        }

        if (*vehWeapon).iHealth != 0 {
            //the missile can take damage
            /*
            //don't do this - ships hit them first and have no trace.plane.normal to bounce off it at and end up in the middle of the asteroid...
            missile->health = vehWeapon->iHealth;
            missile->takedamage = qtrue;
            missile->r.contents = MASK_SHOT;
            missile->die = RocketDie;
            */
        }

        //pilot should own this projectile on server if we have a pilot
        if !(*ent).m_pVehicle.is_null() && !(*(*ent).m_pVehicle).m_pPilot.is_null() {
            //owned by vehicle pilot
            (*missile).r.ownerNum = (*(*(*ent).m_pVehicle).m_pPilot).s.number;
        } else {
            //owned by vehicle?
            (*missile).r.ownerNum = (*ent).s.number;
        }

        //set veh as cgame side owner for purpose of fx overrides
        (*missile).s.owner = (*ent).s.number;
        if alt_fire != QFALSE {
            //use the second weapon's iShotFX
            (*missile).s.eFlags |= EF_ALT_FIRING;
        }
        if isTurretWeap != QFALSE {
            //look for the turret weapon info on cgame side, not vehicle weapon info
            (*missile).s.weapon = WP_TURRET;
        }
        if (*vehWeapon).iLifeTime != 0 {
            //expire after a time
            if (*vehWeapon).bExplodeOnExpire != QFALSE {
                //blow up when your lifetime is up
                // `G_ExplodeMissile` is a Rust-ABI `unsafe fn`; its single `*mut gentity_t`
                // arg is ABI-identical to the `extern "C"` `think` slot, so transmute it into
                // place (precedent: WP_VehWeapSetSolidToOwner).
                (*missile).think = Some(core::mem::transmute::<
                    unsafe fn(*mut gentity_t),
                    unsafe extern "C" fn(*mut gentity_t),
                >(G_ExplodeMissile)); //FIXME: custom func?
            } else {
                //just remove yourself
                (*missile).think = Some(G_FreeEntity); //FIXME: custom func?
            }
            (*missile).nextthink = (*addr_of!(level)).time + (*vehWeapon).iLifeTime;
        }
        (*missile).s.otherEntityNum2 =
            vehWeapon.offset_from(addr_of_mut!(g_vehWeaponInfo[0])) as c_int;
        (*missile).s.eFlags |= EF_JETPACK_ACTIVE;
        //homing
        if (*vehWeapon).fHoming != 0.0 {
            //homing missile
            if !(*ent).client.is_null()
                && (*(*ent).client).ps.rocketLockIndex != ENTITYNUM_NONE
            {
                let mut dif: c_int = 0;
                let mut rTime: f32;
                rTime = (*(*ent).client).ps.rocketLockTime;

                if rTime == -1.0 {
                    rTime = (*(*ent).client).ps.rocketLastValidTime;
                }

                if (*vehWeapon).iLockOnTime == 0 {
                    //no minimum lock-on time
                    dif = 10; //guaranteed lock-on
                } else {
                    let lockTimeInterval = (*vehWeapon).iLockOnTime as f32 / 16.0f32;
                    dif = (((*addr_of!(level)).time as f32 - rTime) / lockTimeInterval) as c_int;
                }

                if dif < 0 {
                    dif = 0;
                }

                //It's 10 even though it locks client-side at 8, because we want them to have a sturdy lock first, and because there's a slight difference in time between server and client
                if dif >= 10 && rTime != -1.0 {
                    (*missile).enemy = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*ent).client).ps.rocketLockIndex as usize);

                    if !(*missile).enemy.is_null()
                        && !(*(*missile).enemy).client.is_null()
                        && (*(*missile).enemy).health > 0
                        && OnSameTeam(ent, (*missile).enemy) == QFALSE
                    {
                        //if enemy became invalid, died, or is on the same team, then don't seek it
                        (*missile).spawnflags |= 1; //just to let it know it should be faster...
                        (*missile).speed = (*vehWeapon).fSpeed;
                        (*missile).angle = (*vehWeapon).fHoming;
                        (*missile).radius = (*vehWeapon).fHomingFOV;
                        //crap, if we have a lifetime, need to store that somewhere else on ent and have rocketThink func check it every frame...
                        if (*vehWeapon).iLifeTime != 0 {
                            //expire after a time
                            (*missile).genericValue1 =
                                (*addr_of!(level)).time + (*vehWeapon).iLifeTime;
                            (*missile).genericValue2 = (*vehWeapon).bExplodeOnExpire;
                        }
                        //now go ahead and use the rocketThink func
                        (*missile).think = Some(rocketThink); //FIXME: custom func?
                        (*missile).nextthink =
                            (*addr_of!(level)).time + VEH_HOMING_MISSILE_THINK_TIME;
                        (*missile).s.eFlags |= EF_RADAROBJECT; //FIXME: externalize
                        if (*(*missile).enemy).s.NPC_class == CLASS_VEHICLE {
                            //let vehicle know we've locked on to them
                            (*missile).s.otherEntityNum = (*(*missile).enemy).s.number;
                        }
                    }
                }

                VectorCopy(dir, &mut (*missile).movedir);
                (*missile).random = 1.0f32; //FIXME: externalize?
            }
        }
        if (*vehWeapon).fSpeed == 0.0 {
            //a mine or something?
            if (*vehWeapon).iHealth != 0 {
                //the missile can take damage
                (*missile).health = (*vehWeapon).iHealth;
                (*missile).takedamage = QTRUE;
                (*missile).r.contents = MASK_SHOT;
                (*missile).die = Some(RocketDie);
            }
            //only do damage when someone touches us
            (*missile).s.weapon = WP_THERMAL; //does this really matter?
            G_SetOrigin(missile, start);
            (*missile).touch = Some(WP_TouchVehMissile);
            (*missile).s.eFlags |= EF_RADAROBJECT; //FIXME: externalize
            //crap, if we have a lifetime, need to store that somewhere else on ent and have rocketThink func check it every frame...
            if (*vehWeapon).iLifeTime != 0 {
                //expire after a time
                (*missile).genericValue1 = (*vehWeapon).iLifeTime;
                (*missile).genericValue2 = (*vehWeapon).bExplodeOnExpire;
            }
            //now go ahead and use the setsolidtoowner func
            (*missile).think = Some(WP_VehWeapSetSolidToOwner);
            (*missile).nextthink = (*addr_of!(level)).time + 3000;
        }
    } else {
        //traceline
        //FIXME: implement
    }

    missile
}

/// `void WP_GetVehicleCamPos( gentity_t *ent, gentity_t *pilot, vec3_t camPos )`
/// (g_weapon.c:3908) — computes the third-person camera position for a piloted vehicle by
/// gathering its `m_pVehicleInfo` camera offsets (with hacking-time and AT-ST pitch-dependant
/// adjustments) and feeding them to [`G_EstimateCamPos`]. No oracle (reads opaque
/// `m_pVehicle`/`client` state and traces via `G_EstimateCamPos`).
///
/// # Safety
/// `ent`/`pilot` must point to valid vehicle/pilot `gentity_t`s with `m_pVehicle`/`client` set.
pub unsafe fn WP_GetVehicleCamPos(ent: *mut gentity_t, pilot: *mut gentity_t, camPos: &mut vec3_t) {
    let mut thirdPersonHorzOffset = (*(*(*ent).m_pVehicle).m_pVehicleInfo).cameraHorzOffset;
    let mut thirdPersonRange = (*(*(*ent).m_pVehicle).m_pVehicleInfo).cameraRange;
    let mut pitchOffset = (*(*(*ent).m_pVehicle).m_pVehicleInfo).cameraPitchOffset;
    let mut vertOffset = (*(*(*ent).m_pVehicle).m_pVehicleInfo).cameraVertOffset;

    if (*(*ent).client).ps.hackingTime != 0 {
        thirdPersonHorzOffset +=
            ((*(*ent).client).ps.hackingTime as f32 / MAX_STRAFE_TIME) * -80.0f32;
        thirdPersonRange +=
            ((*(*ent).client).ps.hackingTime as f32 / MAX_STRAFE_TIME).abs() * 100.0f32;
    }

    if (*(*(*ent).m_pVehicle).m_pVehicleInfo).cameraPitchDependantVertOffset != QFALSE {
        if (*(*pilot).client).ps.viewangles[PITCH] > 0.0 {
            vertOffset = 130.0 + (*(*pilot).client).ps.viewangles[PITCH] * -10.0;
            if vertOffset < -170.0 {
                vertOffset = -170.0;
            }
        } else if (*(*pilot).client).ps.viewangles[PITCH] < 0.0 {
            vertOffset = 130.0 + (*(*pilot).client).ps.viewangles[PITCH] * -5.0;
            if vertOffset > 130.0 {
                vertOffset = 130.0;
            }
        } else {
            vertOffset = 30.0;
        }
        if (*(*pilot).client).ps.viewangles[PITCH] > 0.0 {
            pitchOffset = (*(*pilot).client).ps.viewangles[PITCH] * -0.75;
        } else if (*(*pilot).client).ps.viewangles[PITCH] < 0.0 {
            pitchOffset = (*(*pilot).client).ps.viewangles[PITCH] * -0.75;
        } else {
            pitchOffset = 0.0;
        }
    }

    //Control Scheme 3 Method:
    G_EstimateCamPos(
        &(*(*ent).client).ps.viewangles,
        &mut (*(*pilot).client).ps.origin,
        (*(*pilot).client).ps.viewheight as f32,
        thirdPersonRange,
        thirdPersonHorzOffset,
        vertOffset,
        pitchOffset,
        (*pilot).s.number,
        camPos,
    );
    /*
    //Control Scheme 2 Method:
    G_EstimateCamPos( ent->m_pVehicle->m_vOrientation, ent->r.currentOrigin, pilot->client->ps.viewheight, thirdPersonRange,
        thirdPersonHorzOffset, vertOffset, pitchOffset,
        pilot->s.number, camPos );
    */
}

/// `void WP_VehLeadCrosshairVeh( gentity_t *camTraceEnt, vec3_t newEnd, const vec3_t dir, const vec3_t shotStart, vec3_t shotDir )`
/// (g_weapon.c:4022) — when `g_vehAutoAimLead` is on and the crosshair-trace hit entity is a
/// vehicle, leads the target: extrapolates the hit point forward by the vehicle's closing speed
/// (only if it's moving away fast and/or far enough) and, provided the adjusted aim is within
/// ~23 degrees of the original, snaps `newEnd` to the predicted position. Either way, rewrites
/// `shotDir` to point from `shotStart` to `newEnd` (normalized). No oracle (reads the
/// `g_vehAutoAimLead` cvar + a `gentity_t`'s client state; the file's oracle harness is pure-only).
///
/// # Safety
/// `camTraceEnt` may be null (handled); when non-null its `client` chain must be valid. The
/// vec3 args must be valid; `newEnd`/`shotDir` are written.
pub unsafe fn WP_VehLeadCrosshairVeh(
    camTraceEnt: *mut gentity_t,
    newEnd: &mut vec3_t,
    dir: &vec3_t,
    shotStart: &vec3_t,
    shotDir: &mut vec3_t,
) {
    if (*addr_of!(crate::codemp::game::g_main::g_vehAutoAimLead)).integer != 0 {
        if !camTraceEnt.is_null()
            && !(*camTraceEnt).client.is_null()
            && (*(*camTraceEnt).client).NPC_class == CLASS_VEHICLE
        {
            //if the crosshair is on a vehicle, lead it
            let distAdjust: f32 = DotProduct(&(*(*camTraceEnt).client).ps.velocity, dir);
            if distAdjust > 500.0
                || DistanceSquared(&(*(*camTraceEnt).client).ps.origin, shotStart) > 7000000.0
            {
                //moving away from me at a decent speed and/or more than @2600 units away from me
                let mut predPos: vec3_t = [0.0; 3];
                let mut predShotDir: vec3_t = [0.0; 3];
                VectorMA(&*newEnd, distAdjust, dir, &mut predPos);
                VectorSubtract(&predPos, shotStart, &mut predShotDir);
                VectorNormalize(&mut predShotDir);
                let dot: f32 = DotProduct(&predShotDir, shotDir);
                if dot >= 0.75f32 {
                    //if the new aim vector is no more than 23 degrees off the original one, go ahead and adjust the aim
                    VectorCopy(&predPos, newEnd);
                }
            }
        }
    }
    VectorSubtract(&*newEnd, shotStart, shotDir);
    VectorNormalize(shotDir);
}

const MAX_XHAIR_DIST_ACCURACY: f32 = 20000.0f32; // g_weapon.c:3962

/// `qboolean WP_VehCheckTraceFromCamPos( gentity_t *ent, const vec3_t shotStart, vec3_t shotDir )`
/// (g_weapon.c:3964) — for fighter/walker vehicles piloted by a real client, simulates the
/// dynamic-crosshair trace (straight forward from the vehicle, then — for non-walkers — an extra
/// trace from the third-person camera position via [`WP_GetVehicleCamPos`]) and, when the camera
/// trace hits something closer than the main trace, rewrites `shotDir` toward that point and
/// returns `qtrue`. Otherwise returns `qfalse`. No oracle (entity-state + `trap::Trace`, reads the
/// `g_cullDistance` map cvar).
///
/// # Safety
/// `ent`/`shotStart`/`shotDir` must be valid; `ent->m_pVehicle` chain may be null (handled).
pub unsafe fn WP_VehCheckTraceFromCamPos(
    ent: *mut gentity_t,
    shotStart: &vec3_t,
    shotDir: &mut vec3_t,
) -> qboolean {
    //FIXME: only if dynamicCrosshair and dynamicCrosshairPrecision is on!
    if ent.is_null()
        || (*ent).m_pVehicle.is_null()
        || (*(*ent).m_pVehicle).m_pVehicleInfo.is_null()
        || (*(*ent).m_pVehicle).m_pPilot.is_null() //not being driven
        || (*((*(*ent).m_pVehicle).m_pPilot as *mut gentity_t)).client.is_null() //not being driven by a client...?!!!
        || ((*(*(*ent).m_pVehicle).m_pPilot).s.number >= MAX_CLIENTS as c_int)
    //being driven, but not by a real client, no need to worry about crosshair
    {
        return QFALSE;
    }
    if ((*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER
        && crate::codemp::game::g_spawn::g_cullDistance > MAX_XHAIR_DIST_ACCURACY)
        || (*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER
    {
        //FIRST: simulate the normal crosshair trace from the center of the veh straight forward
        let mut dir: vec3_t = [0.0; 3];
        let mut start: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        if (*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER {
            //for some reason, the walker always draws the crosshair out from from the first muzzle point
            AngleVectors(&(*(*ent).client).ps.viewangles, Some(&mut dir), None, None);
            VectorCopy(&(*ent).r.currentOrigin, &mut start);
            start[2] += (*(*(*ent).m_pVehicle).m_pVehicleInfo).height
                - DEFAULT_MINS_2 as f32
                - 48.0;
        } else {
            let mut ang: vec3_t = [0.0; 3];
            if (*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_SPEEDER {
                VectorSet(
                    &mut ang,
                    0.0f32,
                    *(*(*ent).m_pVehicle).m_vOrientation.add(1),
                    0.0f32,
                );
            } else {
                VectorCopy(
                    &*((*(*ent).m_pVehicle).m_vOrientation as *const vec3_t),
                    &mut ang,
                );
            }
            AngleVectors(&ang, Some(&mut dir), None, None);
            VectorCopy(&(*ent).r.currentOrigin, &mut start);
        }
        VectorMA(
            &start,
            crate::codemp::game::g_spawn::g_cullDistance,
            &dir,
            &mut end,
        );
        let trace = trap::Trace(
            &start,
            &vec3_origin,
            &vec3_origin,
            &end,
            (*ent).s.number,
            CONTENTS_SOLID | CONTENTS_BODY,
        );

        if (*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER {
            //just use the result of that one trace since walkers don't do the extra trace
            VectorSubtract(&trace.endpos, shotStart, shotDir);
            VectorNormalize(shotDir);
            return QTRUE;
        } else {
            //NOW do the trace from the camPos and compare with above trace
            //NOTE: this MUST stay up to date with the method used in CG_ScanForCrosshairEntity (where it checks the doExtraVehTraceFromViewPos bool)
            let mut viewDir2End: vec3_t = [0.0; 3];
            let mut extraEnd: vec3_t = [0.0; 3];
            let mut camPos: vec3_t = [0.0; 3];

            WP_GetVehicleCamPos(
                ent,
                (*(*ent).m_pVehicle).m_pPilot as *mut gentity_t,
                &mut camPos,
            );

            let minAutoAimDist = Distance(&(*ent).r.currentOrigin, &camPos)
                + ((*(*(*ent).m_pVehicle).m_pVehicleInfo).length / 2.0f32)
                + 200.0f32;

            VectorSubtract(&end, &camPos, &mut viewDir2End);
            VectorNormalize(&mut viewDir2End);
            VectorMA(&camPos, MAX_XHAIR_DIST_ACCURACY, &viewDir2End, &mut extraEnd);
            let extraTrace = trap::Trace(
                &camPos,
                &vec3_origin,
                &vec3_origin,
                &extraEnd,
                (*ent).s.number,
                CONTENTS_SOLID | CONTENTS_BODY,
            );
            if extraTrace.allsolid == 0
                && extraTrace.startsolid == 0
                && extraTrace.fraction < 1.0f32
                && (extraTrace.fraction * MAX_XHAIR_DIST_ACCURACY) > minAutoAimDist
                && ((extraTrace.fraction * MAX_XHAIR_DIST_ACCURACY)
                    - Distance(&(*ent).r.currentOrigin, &camPos))
                    < (trace.fraction * crate::codemp::game::g_spawn::g_cullDistance)
            {
                //this trace hit *something* that's closer than the thing the main trace hit, so use this result instead
                VectorSubtract(&extraTrace.endpos, shotStart, shotDir);
                VectorNormalize(shotDir);
                return QTRUE;
            }
        }
    }
    QFALSE
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    /// vec3 inputs spanning signs, magnitudes, and near-integer fractions — the cases
    /// where componentwise abs and truncation-based snapping can differ.
    fn vec3_samples() -> [[f32; 3]; 8] {
        [
            [0.0, 0.0, 0.0],
            [1.0, 2.0, 3.0],
            [-1.0, -2.0, -3.0],
            [1.5, -2.5, 3.5],
            [-0.5, 0.5, -0.5],
            [100.25, -100.75, 0.0],
            [-1234.5, 9999.9, -0.001],
            [0.999, -0.999, 1.0001],
        ]
    }

    #[test]
    fn WP_SpeedOfMissileForWeapon_matches_oracle() {
        for wp in 0..20 {
            for alt in [QFALSE, QTRUE] {
                let rust = WP_SpeedOfMissileForWeapon(wp, alt);
                let c = unsafe { oracle::jka_WP_SpeedOfMissileForWeapon(wp, alt) };
                assert_eq!(rust.to_bits(), c.to_bits(), "wp={wp} alt={alt}");
            }
        }
    }

    #[test]
    fn VectorNPos_matches_oracle_bit_exact() {
        for v in vec3_samples() {
            let mut rust: vec3_t = [0.0; 3];
            VectorNPos(&v, &mut rust);
            let mut c: vec3_t = [0.0; 3];
            unsafe { oracle::jka_VectorNPos(v.as_ptr(), c.as_mut_ptr()) };
            for k in 0..3 {
                assert_eq!(rust[k].to_bits(), c[k].to_bits(), "component {k} for {v:?}");
            }
        }
    }

    #[test]
    fn SnapVectorTowards_matches_oracle_bit_exact() {
        let tos = vec3_samples();
        for v in vec3_samples() {
            for to in tos {
                let mut rust = v;
                SnapVectorTowards(&mut rust, &to);
                let mut c = v;
                unsafe { oracle::jka_SnapVectorTowards(c.as_mut_ptr(), to.as_ptr()) };
                for k in 0..3 {
                    assert_eq!(
                        rust[k].to_bits(),
                        c[k].to_bits(),
                        "component {k} for v={v:?} to={to:?}"
                    );
                }
            }
        }
    }
}
