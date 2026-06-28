//! Port of `g_combat.c` — the damage/death core: `G_Damage` and its supporting
//! armor, hit-location, dismemberment, and death-animation helpers.
//!
//! Landed incrementally, leaf-helpers first. The not-yet-ported-subsystem callees that
//! `G_Damage` reaches (vehicle shields/surfaces, bot-AI notification, CTF carrier
//! checks, force-wall release, jetpack shutdown, NPC arm-breakage) keep their exact
//! guarded call-sites but route to local stubs so the control flow matches the C
//! 1:1 until those subsystems land. `targ->die`/`targ->pain` are dispatched through
//! the entity function-pointer slots, so the death/pain handlers themselves are
//! ported elsewhere.

#![allow(non_snake_case)] // C function names (`G_Damage`, `CheckArmor`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro names kept verbatim

use core::ffi::{c_char, c_int};
use core::ptr::{addr_of, addr_of_mut, null, null_mut};

use crate::codemp::game::bg_g2_utils::BG_GetRootSurfNameWithVariant;
use crate::codemp::game::bg_panimate::BG_InDeathAnim;
use crate::codemp::game::bg_pmove::BG_KnockDownable;
use crate::codemp::game::bg_public::{
    gitem_t, HANDEXTEND_KNOCKDOWN,
    BG_GiveMeVectorFromMatrix, ARMOR_PROTECTION, ARMOR_REDUCTION_FACTOR, BROKENLIMB_LARM,
    BROKENLIMB_RARM, EF_DEAD, EF_DISINTEGRATION, EF_INVULNERABLE, ET_GENERAL, ET_INVISIBLE,
    ET_MISSILE, ET_MOVER, ET_NPC, ET_PLAYER,
    EV_DESTROY_WEAPON_MODEL, EV_GHOUL2_MARK, EV_GIB_PLAYER, EV_NOAMMO, EV_POWERUP_BATTLESUIT,
    EV_SABER_HIT, EV_SCOREPLUM, EV_SHIELD_HIT,
    IT_WEAPON, PW_NUM_POWERUPS, STAT_WEAPONS, WEAPON_DROPPING,
    GIB_HEALTH, GT_CTF, GT_CTY, GT_DUEL,
    GT_POWERDUEL,
    GT_SIEGE, GT_TEAM, G2_MODELPART_HEAD, G2_MODELPART_LARM, G2_MODELPART_LLEG, G2_MODELPART_RARM,
    G2_MODELPART_RHAND, G2_MODELPART_RLEG, G2_MODELPART_WAIST, G2_MODEL_PART, MASK_SOLID,
    MOD_BRYAR_PISTOL, MOD_BRYAR_PISTOL_ALT, MOD_CONC, MOD_CONC_ALT, MOD_CRUSH, MOD_DEMP2,
    MOD_DEMP2_ALT, MOD_DET_PACK_SPLASH, MOD_FALLING, MOD_FLECHETTE_ALT_SPLASH, MOD_LAVA, MOD_MELEE,
    MOD_REPEATER_ALT, MOD_REPEATER_ALT_SPLASH, MOD_ROCKET, MOD_ROCKET_HOMING,
    MOD_ROCKET_HOMING_SPLASH, MOD_ROCKET_SPLASH, MOD_SABER, MOD_SLIME, MOD_SUICIDE, MOD_TARGET_LASER,
    MOD_TELEFRAG, MOD_THERMAL,
    MOD_THERMAL_SPLASH, MOD_TIMED_MINE_SPLASH, MOD_TRIGGER_HURT, MOD_TRIP_MINE_SPLASH, MOD_TURBLAST,
    MOD_WATER,
    MOD_UNKNOWN, MOD_VEHICLE, PDSOUND_PROTECTHIT, PERS_ATTACKEE_ARMOR, PERS_ATTACKER, PERS_HITS,
    PERS_SCORE, PERS_TEAM,
    PMF_STUCK_TO_WALL, PMF_TIME_KNOCKBACK, PW_BATTLESUIT, PW_FORCE_BOON, STAT_ARMOR, STAT_DEAD_YAW,
    STAT_HEALTH,
    STAT_MAX_HEALTH, TEAM_SPECTATOR, WEAPON_CHARGING, WEAPON_CHARGING_ALT, WEAPON_READY,
    DUELTEAM_DOUBLE, DUELTEAM_LONE, EF2_SHIP_DEATH, EV_DEATH1, EV_OBITUARY, MOD_BLASTER,
    MOD_MAX, MOD_STUN_BATON, MOD_TEAM_CHANGE, PERS_EXCELLENT_COUNT, PERS_GAUNTLET_FRAG_COUNT,
    PERS_KILLED, PERS_PLAYEREVENTS, PLAYEREVENT_GAUNTLETREWARD, PM_DEAD, PM_NORMAL,
    PW_BLUEFLAG, PW_NEUTRALFLAG, PW_REDFLAG, SETANIM_BOTH, SETANIM_FLAG_HOLD,
    SETANIM_FLAG_OVERRIDE, SETANIM_FLAG_RESTART, TEAM_BLUE, TEAM_FREE, TEAM_RED,
};
use crate::codemp::game::bg_misc::{
    bg_itemlist, vectoyaw, BG_FindItemForPowerup, BG_FindItemForWeapon, BG_GetItemIndexByTag,
};
use crate::codemp::game::bg_public::bgEntity_t;
use crate::codemp::game::bg_weapons::weaponData;
use crate::codemp::game::g_client::{ClientUserinfoChanged, G_BreakArm, G_UpdateClientAnims};
use crate::codemp::game::g_cmds::Cmd_Score_f;
use crate::codemp::game::bg_saga::bgSiegeClasses;
use crate::codemp::game::bg_saga_h::{CFL_HEAVYMELEE, CFL_STRONGAGAINSTPHYSICAL};
use crate::codemp::game::bg_vehicles_h::{
    SHIPSURF_BACK, SHIPSURF_FRONT, SHIPSURF_LEFT, SHIPSURF_RIGHT, VH_ANIMAL, VH_FIGHTER, VH_SPEEDER,
    VH_WALKER,
};
use crate::codemp::game::bg_weapons_h::{
    WP_BRYAR_PISTOL, WP_DET_PACK, WP_EMPLACED_GUN, WP_NONE, WP_NUM_WEAPONS, WP_ROCKET_LAUNCHER,
    WP_SABER, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use crate::codemp::game::g_log::{
    G_LogWeaponDamage, G_LogWeaponDeath, G_LogWeaponFrag, G_LogWeaponKill,
};
use crate::codemp::game::g_local::{
    gclient_t, gentity_t, CON_CONNECTED, DAMAGE_HALF_ABSORB, DAMAGE_HALF_ARMOR_REDUCTION, DAMAGE_NO_ARMOR,
    DAMAGE_NO_DISMEMBER, DAMAGE_NO_HIT_LOC, DAMAGE_NO_KNOCKBACK, DAMAGE_NO_PROTECTION,
    DAMAGE_NO_SELF_PROTECTION, DAMAGE_RADIUS, DAMAGE_SABER_KNOCKBACK1, DAMAGE_SABER_KNOCKBACK2,
    FL_BBRUSH, FL_DMG_BY_HEAVY_WEAP_ONLY, FL_DMG_BY_SABER_ONLY, FL_GODMODE, FL_NO_KNOCKBACK,
    FL_SHIELDED, FL_UNDYING, FRAMETIME, HL_ARM_LT, HL_ARM_RT, HL_BACK, HL_BACK_LT, HL_BACK_RT,
    HL_CHEST, HL_CHEST_LT, HL_CHEST_RT, HL_FOOT_LT, HL_FOOT_RT, HL_GENERIC1, HL_GENERIC2,
    HL_GENERIC3, HL_GENERIC4, HL_GENERIC5, HL_GENERIC6, HL_HAND_LT, HL_HAND_RT, HL_HEAD,
    HL_LEG_LT, HL_LEG_RT, HL_MAX, HL_NONE, HL_WAIST, MOVER_POS1, CARNAGE_REWARD_TIME,
    REWARD_SPRITE_TIME,
};
use crate::codemp::game::g_items::{Drop_Item, Jetpack_Off, LaunchItem};
use crate::codemp::game::g_main::{
    d_projectileGhoul2Collision, d_saberGhoul2Collision, g_armBreakage, g_austrian, g_debugDamage,
    g_debugMelee, g_dismember, g_entities, g_ff_objectives, g_friendlyFire, g_gametype, g_gravity,
    g_knockback,
    g_dontFrickinCheck, g_locationBasedDamage, g_saberDmgVelocityScale, g_slowmoDuelEnd,
    g_trueJedi, level, CalculateRanks, CheckExitRules, gDoSlowMoDuel, gSlowMoDuelTime, g_endPDuel, G_LogPrintf,
    G_Printf,
};
use crate::codemp::game::q_math::{
    vec3_origin, AngleVectors, DirToByte, DistanceSquared, DotProduct, VectorAdd, VectorClear,
    VectorCompare, VectorCopy, VectorLength, VectorLengthSquared, VectorMA, VectorNormalize,
    VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{Com_sprintf, Q_stricmp, Q_strncmp, Q_strncpyz, Sz};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::g_exphysics::G_RunExPhys;
use crate::codemp::game::g_public_h::{
    BSET_DEATH, BSET_FFDEATH, BSET_VICTORY, Q3_INFINITE, SVF_BROADCAST, SVF_GLASS_BRUSH,
    SVF_SINGLECLIENT, SVF_USE_CURRENT_ORIGIN,
};
use crate::codemp::game::g_timer::{TIMER_Clear2, TIMER_Set};
use crate::codemp::game::g_team::{
    OnSameTeam, Team_CheckHurtCarrier, Team_FragBonuses, Team_ReturnFlag,
};
use crate::codemp::game::g_utils::{
    G_AddEvent, G_EffectIndex, G_FreeEntity, G_MuteSound, G_PlayEffectID, G_ScaleNetHealth,
    G_SetAnim, G_SetOrigin, G_Sound, G_SoundIndex, G_Spawn, G_TempEntity, G_UseTargets,
    G_UseTargets2, GlobalUse,
};
use crate::codemp::game::g_weapon::{BlowDetpacks, LogAccuracyHit};
use crate::codemp::game::npc_utils::{G_ActivateBehavior, NPC_ClearLOS2};
use crate::codemp::game::npc_combat::{G_SetEnemy, NPC_FreeCombatPoint};
use crate::codemp::game::npc_ai_utils::{AI_DeleteSelfFromGroup, AI_GroupMemberKilled};
use crate::codemp::game::npc_ai_jedi::{Boba_FlyStop, Jedi_WaitingAmbush};
use crate::codemp::game::npc_ai_rancor::Rancor_DropVictim;
use crate::codemp::game::npc_senses::InFOV;
use crate::codemp::game::b_public_h::{
    SCF_FFDEATH, SCF_IGNORE_ALERTS, SCF_LOOK_FOR_ENEMIES, SCF_NO_GROUPS,
};
use crate::codemp::cgame::animtable::animTable;
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, playerState_t, trace_t, vec3_t, CHAN_AUTO, CHAN_WEAPON, ENTITYNUM_NONE, ENTITYNUM_WORLD, FORCE_LEVEL_1,
    FORCE_LEVEL_2, FORCE_LEVEL_3, FP_PROTECT, FP_RAGE, MAX_CLIENTS, MAX_GENTITIES, MAX_QPATH,
    NEGATIVE_Y, ORIGIN, PITCH, ROLL, TR_LINEAR_STOP, TR_NONLINEAR_STOP, TR_STATIONARY, YAW,
};
use crate::codemp::game::anims::{
    BOTH_CHOKE3, BOTH_DEAD1, BOTH_DEAD10, BOTH_DEAD11, BOTH_DEAD12, BOTH_DEAD13, BOTH_DEAD14,
    BOTH_DEAD15, BOTH_DEAD16, BOTH_DEAD17, BOTH_DEAD18, BOTH_DEAD19, BOTH_DEAD2, BOTH_DEAD3,
    BOTH_DEAD4, BOTH_DEAD5, BOTH_DEAD6, BOTH_DEAD7, BOTH_DEAD8, BOTH_DEAD9, BOTH_DEADBACKWARD1,
    BOTH_DEADBACKWARD2, BOTH_DEADFLOP1, BOTH_DEADFORWARD1, BOTH_DEADFORWARD2, BOTH_DEATH1,
    BOTH_DEATH10, BOTH_DEATH11, BOTH_DEATH12, BOTH_DEATH13, BOTH_DEATH14, BOTH_DEATH15,
    BOTH_DEATH16, BOTH_DEATH17, BOTH_DEATH18, BOTH_DEATH19, BOTH_DEATH1IDLE, BOTH_DEATH2,
    BOTH_DEATH25, BOTH_DEATH3, BOTH_DEATH4, BOTH_DEATH5, BOTH_DEATH6, BOTH_DEATH7, BOTH_DEATH8,
    BOTH_DEATH9, BOTH_DEATHBACKWARD1, BOTH_DEATHBACKWARD2, BOTH_DEATHFORWARD1, BOTH_DEATHFORWARD2,
    BOTH_DEATHFORWARD3, BOTH_DEATH_CROUCHED, BOTH_DEATH_FALLING_DN, BOTH_DEATH_FALLING_UP,
    BOTH_DEATH_FLIP, BOTH_DEATH_LYING_DN, BOTH_DEATH_LYING_UP, BOTH_DEATH_ROLL, BOTH_DEATH_SPIN_180,
    BOTH_FALLDEAD1LAND, BOTH_FALLDEATH1, BOTH_FALLDEATH1INAIR, BOTH_FALLDEATH1LAND,
    BOTH_FORCE_GETUP_B1, BOTH_FORCE_GETUP_B2, BOTH_FORCE_GETUP_B3, BOTH_FORCE_GETUP_B4,
    BOTH_FORCE_GETUP_B5, BOTH_FORCE_GETUP_B6, BOTH_FORCE_GETUP_F1, BOTH_FORCE_GETUP_F2, BOTH_GETUP1,
    BOTH_GETUP2, BOTH_GETUP3, BOTH_GETUP4, BOTH_GETUP5, BOTH_GETUP_CROUCH_B1, BOTH_GETUP_CROUCH_F1,
    BOTH_KNOCKDOWN1, BOTH_KNOCKDOWN2, BOTH_KNOCKDOWN3, BOTH_KNOCKDOWN4, BOTH_KNOCKDOWN5,
    BOTH_LYINGDEAD1, BOTH_LYINGDEATH1, BOTH_RIGHTHANDCHOPPEDOFF, BOTH_STUMBLEDEAD1,
    BOTH_STUMBLEDEATH1,
};
use crate::codemp::game::bg_panimate::{
    bgAllAnims, bgHumanoidAnimations, BG_FlippingAnim, BG_HasAnimation, BG_InRoll, BG_PickAnim,
};
use crate::codemp::game::surfaceflags_h::{CONTENTS_CORPSE, CONTENTS_NODROP, CONTENTS_TRIGGER};
use crate::codemp::game::teams_h::{
    CLASS_ATST, CLASS_GALAKMECH, CLASS_GONK, CLASS_INTERROGATOR, CLASS_MARK1, CLASS_MARK2,
    CLASS_MOUSE, CLASS_PROBE,
    CLASS_PROTOCOL, CLASS_R2D2, CLASS_R5D2, CLASS_RANCOR, CLASS_REMOTE, CLASS_SEEKER, CLASS_SENTRY,
    CLASS_VEHICLE,
};
use crate::codemp::game::w_force::{G_LetGoOfWall, G_PreDefSound};
use crate::codemp::game::ai_main::BotDamageNotification;
use crate::codemp::game::g_vehicles::{
    G_FlyVehicleDestroySurface, G_ShipSurfaceForSurfName, G_VehUpdateShields,
    G_VehicleSetDamageLocFlags,
};
use crate::codemp::game::w_saber::UpdateClientRenderBolts;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `qboolean G_HeavyMelee( gentity_t *attacker )` (g_combat.c:32) — true only in
/// Siege when the attacker's siege class carries the `CFL_HEAVYMELEE` flag. Used by
/// the melee damage path to grant the heavy-melee knockdown bonus.
///
/// No oracle — reads the global `g_gametype` cvar and `bgSiegeClasses` table.
///
/// # Safety
/// `attacker` may be NULL (checked); when non-NULL its `client` (also checked) and
/// `siegeClass` must index a valid `bgSiegeClasses` slot, as in the C original.
pub unsafe fn G_HeavyMelee(attacker: *mut gentity_t) -> qboolean {
    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && !attacker.is_null()
        && !(*attacker).client.is_null()
        && (*(*attacker).client).siegeClass != -1
        && (bgSiegeClasses[(*(*attacker).client).siegeClass as usize].classflags
            & (1 << CFL_HEAVYMELEE))
            != 0
    {
        return QTRUE;
    }
    QFALSE
}

/// `int G_GetHitLocation(gentity_t *target, vec3_t ppoint)` (g_combat.c:45) —
/// classifies an impact `ppoint` into a hit location (`HL_*`) by projecting the
/// point onto the target's local up/forward/right axes and bucketing the three dot
/// products into a 5×5×5 grid. Returns `HL_NONE` when `ppoint` is NULL or the origin.
///
/// No oracle harness wires it (it takes a `gentity_t`), but the float math is
/// oracle-checked via [`oracle_tests`] against the extracted C. The threshold tests
/// promote the `f32` dot products to `f64` to match C's `float`→`double` comparison
/// promotion exactly.
///
/// # Safety
/// `target` must be a valid entity (its `client` is NULL-checked, `r` read);
/// `ppoint` may be NULL (checked) and otherwise must point to a `vec3_t`.
pub unsafe fn G_GetHitLocation(target: *mut gentity_t, ppoint: *const vec3_t) -> c_int {
    let mut point: vec3_t = [0.0; 3];
    let mut point_dir: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    // C leaves `tangles` uninitialized when the target has no client; every caller
    // passes a client, so we zero-init rather than reproduce that UB (DEVIATION).
    let mut tangles: vec3_t = [0.0; 3];
    let mut tcenter: vec3_t = [0.0; 3];

    // Get target forward, right and up.
    if !(*target).client.is_null() {
        // Ignore player's pitch and roll.
        VectorSet(&mut tangles, 0.0, (*target).r.currentAngles[YAW], 0.0);
    }

    AngleVectors(
        &tangles,
        Some(&mut forward),
        Some(&mut right),
        Some(&mut up),
    );

    // Get center of target.
    VectorAdd(&(*target).r.absmin, &(*target).r.absmax, &mut tcenter);
    let tcenter_in = tcenter;
    VectorScale(&tcenter_in, 0.5, &mut tcenter);

    // Get radius width of target. (Dead in C too — the cylinder-projection block that
    // consumed it is commented out — but kept for faithfulness.)
    let _tradius = ((*target).r.maxs[0].abs()
        + (*target).r.maxs[1].abs()
        + (*target).r.mins[0].abs()
        + (*target).r.mins[1].abs())
        / 4.0;

    // Get impact point.
    if !ppoint.is_null() && VectorCompare(&*ppoint, &vec3_origin) == 0 {
        VectorCopy(&*ppoint, &mut point);
    } else {
        return HL_NONE;
    }

    VectorSubtract(&point, &tcenter, &mut point_dir);
    VectorNormalize(&mut point_dir);

    // Get bottom to top (vertical) position index
    let udot = DotProduct(&up, &point_dir);
    let vertical = if (udot as f64) > 0.800 {
        4
    } else if (udot as f64) > 0.400 {
        3
    } else if (udot as f64) > -0.333 {
        2
    } else if (udot as f64) > -0.666 {
        1
    } else {
        0
    };

    // Get back to front (forward) position index.
    let fdot = DotProduct(&forward, &point_dir);
    let forward_idx = if (fdot as f64) > 0.666 {
        4
    } else if (fdot as f64) > 0.333 {
        3
    } else if (fdot as f64) > -0.333 {
        2
    } else if (fdot as f64) > -0.666 {
        1
    } else {
        0
    };

    // Get left to right (lateral) position index.
    let rdot = DotProduct(&right, &point_dir);
    let lateral = if (rdot as f64) > 0.666 {
        4
    } else if (rdot as f64) > 0.333 {
        3
    } else if (rdot as f64) > -0.333 {
        2
    } else if (rdot as f64) > -0.666 {
        1
    } else {
        0
    };

    let hit_loc: c_int = vertical * 25 + forward_idx * 5 + lateral;

    if hit_loc <= 10 {
        // Feet.
        if (rdot as f64) > 0.0 {
            return HL_FOOT_RT;
        } else {
            return HL_FOOT_LT;
        }
    } else if hit_loc <= 50 {
        // Legs.
        if (rdot as f64) > 0.0 {
            return HL_LEG_RT;
        } else {
            return HL_LEG_LT;
        }
    } else if hit_loc == 56 || hit_loc == 60 || hit_loc == 61 || hit_loc == 65 || hit_loc == 66
        || hit_loc == 70
    {
        // Hands.
        if (rdot as f64) > 0.0 {
            return HL_HAND_RT;
        } else {
            return HL_HAND_LT;
        }
    } else if hit_loc == 83 || hit_loc == 87 || hit_loc == 88 || hit_loc == 92 || hit_loc == 93
        || hit_loc == 97
    {
        // Arms.
        if (rdot as f64) > 0.0 {
            return HL_ARM_RT;
        } else {
            return HL_ARM_LT;
        }
    } else if (107..=109).contains(&hit_loc)
        || (112..=114).contains(&hit_loc)
        || (117..=119).contains(&hit_loc)
    {
        // Head.
        return HL_HEAD;
    } else {
        if (udot as f64) < 0.3 {
            return HL_WAIST;
        } else if (fdot as f64) < 0.0 {
            if (rdot as f64) > 0.4 {
                return HL_BACK_RT;
            } else if (rdot as f64) < -0.4 {
                return HL_BACK_LT;
            } else if (fdot as f64) < 0.0 {
                return HL_BACK;
            }
        } else {
            if (rdot as f64) > 0.3 {
                return HL_CHEST_RT;
            } else if (rdot as f64) < -0.3 {
                return HL_CHEST_LT;
            } else if (fdot as f64) < 0.0 {
                return HL_CHEST;
            }
        }
    }
    HL_NONE
}

/// `int CheckArmor (gentity_t *ent, int damage, int dflags)` (g_combat.c:2896) —
/// drains the client's shield (`STAT_ARMOR`) by the protectable portion of `damage`
/// and returns the amount the shields absorbed. `DAMAGE_HALF_ABSORB` halves the
/// absorbed share (`ARMOR_PROTECTION`); `DAMAGE_HALF_ARMOR_REDUCTION` whittles armor
/// at half rate (`ARMOR_REDUCTION_FACTOR`). Disabled (returns 0) for armorless
/// damage, for non-clients, and for ion-cannon-electrified vehicle shields.
///
/// No oracle — operates on the `gclient_t` shield stat and reads the global `level`,
/// like its g_utils game-state siblings. The two FP steps mirror C's `double`
/// promotion exactly (`0.50` constants), so the `ceil`/cast results are bit-identical.
///
/// # Safety
/// `ent` must be a valid entity; its `client` (NULL-checked) and, on the vehicle
/// branch, `m_pVehicle` are dereferenced exactly as in the C original.
pub unsafe fn CheckArmor(ent: *mut gentity_t, damage: c_int, dflags: c_int) -> c_int {
    if damage == 0 {
        return 0;
    }

    let client = (*ent).client;

    if client.is_null() {
        return 0;
    }

    if dflags & DAMAGE_NO_ARMOR != 0 {
        return 0;
    }

    if (*client).NPC_class == CLASS_VEHICLE
        && !(*ent).m_pVehicle.is_null()
        && (*(*ent).client).ps.electrifyTime > (*addr_of!(level)).time
    {
        // ion-cannon has disabled this ship's shields, take damage on hull!
        return 0;
    }
    // armor
    let count = (*client).ps.stats[STAT_ARMOR as usize];

    let mut save = if dflags & DAMAGE_HALF_ABSORB != 0 {
        // Half the damage gets absorbed by the shields, rather than 100%
        (damage as f64 * ARMOR_PROTECTION as f64).ceil() as c_int
    } else {
        // All the damage gets absorbed by the shields.
        damage
    };

    // save is the most damage that the armor is elibigle to protect, of course, but it's limited by the total armor.
    if save >= count {
        save = count;
    }

    if save == 0 {
        return 0;
    }

    if dflags & DAMAGE_HALF_ARMOR_REDUCTION != 0 {
        // Armor isn't whittled so easily by sniper shots.
        (*client).ps.stats[STAT_ARMOR as usize] -=
            (save as f64 * ARMOR_REDUCTION_FACTOR as f64) as c_int;
    } else {
        (*client).ps.stats[STAT_ARMOR as usize] -= save;
    }

    save
}

/// `void G_ApplyKnockback( gentity_t *targ, vec3_t newDir, float knockback )`
/// (g_combat.c:2951) — push `targ` along `newDir` by `knockback`, scaled by the
/// `g_knockback` cvar and inversely by the target's mass (`physicsBounce`, else 200).
/// With positive gravity the horizontal component is scaled `0.8` and the vertical
/// `1.5` (a small upward "pop"); otherwise the push is uniform. Clients take it on
/// `ps.velocity`; non-stationary non-client movers take it on the position trajectory
/// (`s.pos.trDelta`, re-basing `trBase`/`trTime` to the current origin/time). For a
/// client with no pending `pm_time`, a 50–200 ms knockback timer is armed so the move
/// can't be cancelled immediately.
///
/// Oracle-checked bit-exact via [`oracle_tests`] against the extracted C: the gravity
/// branch's `*0.8`/`*1.5` scaling is computed through `double` exactly as the C
/// `VectorScale` macro / statement promote it.
///
/// # Safety
/// `targ` must be a valid `*mut gentity_t`; `new_dir` a valid `*const vec3_t`.
pub unsafe fn G_ApplyKnockback(targ: *mut gentity_t, new_dir: *const vec3_t, knockback: f32) {
    let mut kvel: vec3_t = [0.0; 3];
    let mass: f32;

    if (*targ).physicsBounce > 0.0 {
        // overide the mass
        mass = (*targ).physicsBounce;
    } else {
        mass = 200.0;
    }

    let kn = (*addr_of!(g_knockback)).value;
    let nd = &*new_dir;

    if (*addr_of!(g_gravity)).value > 0.0 {
        // Matches the C `VectorScale` macro: the f32 product `kn * knockback / mass`
        // is promoted through `double` by the `* 0.8` literal before the per-component
        // multiply, then narrowed back to f32 on store.
        let scale = (kn * knockback / mass) as f64 * 0.8;
        kvel[0] = (nd[0] as f64 * scale) as f32;
        kvel[1] = (nd[1] as f64 * scale) as f32;
        kvel[2] = ((nd[2] * kn * knockback / mass) as f64 * 1.5) as f32;
    } else {
        VectorScale(nd, kn * knockback / mass, &mut kvel);
    }

    if !(*targ).client.is_null() {
        let v_cur = (*(*targ).client).ps.velocity;
        VectorAdd(&v_cur, &kvel, &mut (*(*targ).client).ps.velocity);
    } else if (*targ).s.pos.trType != TR_STATIONARY
        && (*targ).s.pos.trType != TR_LINEAR_STOP
        && (*targ).s.pos.trType != TR_NONLINEAR_STOP
    {
        let d_cur = (*targ).s.pos.trDelta;
        VectorAdd(&d_cur, &kvel, &mut (*targ).s.pos.trDelta);
        let cur_origin = (*targ).r.currentOrigin;
        VectorCopy(&cur_origin, &mut (*targ).s.pos.trBase);
        (*targ).s.pos.trTime = (*addr_of!(level)).time;
    }

    // set the timer so that the other client can't cancel
    // out the movement immediately
    if !(*targ).client.is_null() && (*(*targ).client).ps.pm_time == 0 {
        let mut t: c_int = (knockback * 2.0) as c_int;
        if t < 50 {
            t = 50;
        }
        if t > 200 {
            t = 200;
        }
        (*(*targ).client).ps.pm_time = t;
        (*(*targ).client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
    }
}

/// `void CheckAlmostCapture( gentity_t *self, gentity_t *attacker )` (g_combat.c:805) —
/// faithful no-op: the entire C body is wrapped in `#if 0` (a disabled CTF "almost scored"
/// HOLYSHIT taunt), so it compiles to nothing. The signature is kept for parity; both
/// params are unused exactly as in the compiled C. No oracle (no behavior to test).
///
/// # Safety
/// Trivially safe — does nothing. `unsafe` is kept for signature parity with its callers.
pub unsafe fn CheckAlmostCapture(_self_: *mut gentity_t, _attacker: *mut gentity_t) {
    // C body is entirely `#if 0` — disabled CTF near-capture taunt.
}

/// `qboolean G_InKnockDown( playerState_t *ps )` (g_combat.c:852) — true while the
/// player is in any knockdown or get-up animation (`legsAnim`). Used by the special
/// death-anim selector and the knockdown logic to know the player is already floored.
///
/// Oracle-tested via [`oracle_tests`]: the C body reads only `ps->legsAnim`, so the
/// `jka_` wrapper takes a bare `int`. Note the two crouch get-ups
/// (`BOTH_GETUP_CROUCH_F1`/`_B1`) are deliberately *not* in the original switch, so
/// they fall through to `qfalse`.
///
/// # Safety
/// `ps` must be a valid `playerState_t` (only `legsAnim` is read).
pub unsafe fn G_InKnockDown(ps: *const playerState_t) -> qboolean {
    match (*ps).legsAnim {
        BOTH_KNOCKDOWN1 | BOTH_KNOCKDOWN2 | BOTH_KNOCKDOWN3 | BOTH_KNOCKDOWN4
        | BOTH_KNOCKDOWN5 => QTRUE,
        BOTH_GETUP1 | BOTH_GETUP2 | BOTH_GETUP3 | BOTH_GETUP4 | BOTH_GETUP5
        | BOTH_FORCE_GETUP_F1 | BOTH_FORCE_GETUP_F2 | BOTH_FORCE_GETUP_B1
        | BOTH_FORCE_GETUP_B2 | BOTH_FORCE_GETUP_B3 | BOTH_FORCE_GETUP_B4
        | BOTH_FORCE_GETUP_B5 => QTRUE,
        _ => QFALSE,
    }
}

/// `void G_Knockdown( gentity_t *victim )` (g_combat.c:4328) — force a client into the
/// 1100 ms knockdown hand-extend state, unless [`BG_KnockDownable`] vetoes it (riding a
/// vehicle or manning an emplaced gun). Clears any pending dodge anim and the
/// quicker-getup flag so the full knockdown plays.
///
/// Oracle-tested via [`oracle::jka_G_Knockdown`]: the body touches only
/// `client->ps` (the two `BG_KnockDownable` gates plus the four written fields) and
/// `level.time`, so the wrapper marshals just those in and out.
///
/// # Safety
/// `victim` may be NULL or have a NULL `client` (both checked); otherwise it must point
/// at a valid `gentity_t` whose `client` is a valid `gclient_t`.
pub unsafe fn G_Knockdown(victim: *mut gentity_t) {
    if !victim.is_null()
        && !(*victim).client.is_null()
        && BG_KnockDownable(&mut (*(*victim).client).ps) != QFALSE
    {
        (*(*victim).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
        (*(*victim).client).ps.forceDodgeAnim = 0;
        (*(*victim).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1100;
        (*(*victim).client).ps.quickerGetup = QFALSE;
    }
}

/// `gentity_t *G_GetJediMaster(void)` (g_combat.c:1744) — return the current Jedi Master
/// client by scanning client slots for the first connected entity whose
/// `ps.isJediMaster` flag is set. Faithful 1:1 with retail PC. No oracle — pure scan over
/// the global `g_entities` table with opaque `gentity_t`/`gclient_t`.
///
/// # Safety
/// Reads the global `g_entities` base pointer; must be called after `G_InitGame` has
/// allocated it (as all in-game callers are).
// TODO: Port-Bug
pub unsafe fn G_GetJediMaster() -> *mut gentity_t {
    let mut i: c_int = 0;
    while i < MAX_CLIENTS as c_int {
        let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if (*ent).inuse != 0
            && !(*ent).client.is_null()
            && (*(*ent).client).ps.isJediMaster != 0
        {
            return ent;
        }
        i += 1;
    }
    null_mut()
}

/// `qboolean G_ThereIsAMaster(void)` (g_combat.c:4432) — is any client the Jedi Master?
/// Scans all client slots for an entity whose `ps.isJediMaster` flag is set (note: unlike
/// [`G_GetJediMaster`], retail PC does *not* test `inuse` here). Faithful 1:1 with retail
/// PC. No oracle — pure scan over the global `g_entities` table.
///
/// # Safety
/// Reads the global `g_entities` base pointer; see [`G_GetJediMaster`].
pub unsafe fn G_ThereIsAMaster() -> qboolean {
    let mut i: c_int = 0;
    while i < MAX_CLIENTS as c_int {
        let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if !(*ent).client.is_null() && (*(*ent).client).ps.isJediMaster != 0 {
            return QTRUE;
        }
        i += 1;
    }
    QFALSE
}

/// `void G_AddPowerDuelScore(int team, int score)` (g_combat.c:2015) — award `score` to
/// every *living* combatant on the given power-duel `team`. Scans all client slots and, for
/// each connected, non-loser, alive, non-spectating client whose `sess.duelTeam` matches,
/// adds `score` to `sess.wins` and re-broadcasts that client's userinfo (so the updated win
/// count propagates) via [`ClientUserinfoChanged`]. Faithful 1:1 with the C.
///
/// No oracle — the body is pure entity-state mutation over the global `g_entities` table
/// plus a `trap_*`-backed `ClientUserinfoChanged` broadcast, neither of which the oracle
/// harness can stand up (opaque `gclient_t`/`gentity_t`, engine traps).
///
/// # Safety
/// Reads the global `g_entities` base pointer; must be called after `G_InitGame` has
/// allocated it (as all in-game callers are).
pub unsafe fn G_AddPowerDuelScore(team: c_int, score: c_int) {
    let mut i: c_int = 0;
    while i < MAX_CLIENTS as c_int {
        let check = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if (*check).inuse != 0
            && !(*check).client.is_null()
            && (*(*check).client).pers.connected == CON_CONNECTED
            && (*(*check).client).iAmALoser == 0
            && (*(*check).client).ps.stats[STAT_HEALTH as usize] > 0
            && (*(*check).client).sess.sessionTeam != TEAM_SPECTATOR
            && (*(*check).client).sess.duelTeam == team
        {
            // found a living client on the specified team
            (*(*check).client).sess.wins += score;
            ClientUserinfoChanged((*check).s.number);
        }
        i += 1;
    }
}

/// `void G_AddPowerDuelLoserScore(int team, int score)` (g_combat.c:2036) — the loser-side
/// counterpart to [`G_AddPowerDuelScore`]: award `score` losses to every *defeated* combatant
/// on the given power-duel `team`. Scans all client slots and, for each connected client on
/// the matching `sess.duelTeam` that has either been flagged a loser (`iAmALoser`) or died
/// without spectating (`ps.stats[STAT_HEALTH] <= 0 && sess.sessionTeam != TEAM_SPECTATOR`),
/// adds `score` to `sess.losses` and re-broadcasts that client's userinfo (so the updated
/// loss count propagates) via [`ClientUserinfoChanged`]. Faithful 1:1 with the C (the C's
/// "found a living client" comment is a copy-paste artifact from the win variant — kept).
///
/// No oracle — the body is pure entity-state mutation over the global `g_entities` table
/// plus a `trap_*`-backed `ClientUserinfoChanged` broadcast, neither of which the oracle
/// harness can stand up (opaque `gclient_t`/`gentity_t`, engine traps).
///
/// # Safety
/// Reads the global `g_entities` base pointer; must be called after `G_InitGame` has
/// allocated it (as all in-game callers are).
pub unsafe fn G_AddPowerDuelLoserScore(team: c_int, score: c_int) {
    let mut i: c_int = 0;
    while i < MAX_CLIENTS as c_int {
        let check = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if (*check).inuse != 0
            && !(*check).client.is_null()
            && (*(*check).client).pers.connected == CON_CONNECTED
            && ((*(*check).client).iAmALoser != 0
                || ((*(*check).client).ps.stats[STAT_HEALTH as usize] <= 0
                    && (*(*check).client).sess.sessionTeam != TEAM_SPECTATOR))
            && (*(*check).client).sess.duelTeam == team
        {
            // found a living client on the specified team
            (*(*check).client).sess.losses += score;
            ClientUserinfoChanged((*check).s.number);
        }
        i += 1;
    }
}

/// `void G_BroadcastObit( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int killer, int meansOfDeath, int wasInVehicle, qboolean wasJediMaster )` (g_combat.c:2059) — builds an `EV_OBITUARY` temp entity and flags it `SVF_BROADCAST` so the death/obituary event reaches every client.
///
/// No oracle — spawns a broadcast temp entity via [`G_TempEntity`] and mutates live game state.
///
/// # Safety
/// `self_` must be a valid entity; `inflictor`/`attacker` may be NULL (checked).
pub unsafe fn G_BroadcastObit(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    killer: c_int,
    meansOfDeath: c_int,
    wasInVehicle: c_int,
    wasJediMaster: qboolean,
) {
    // broadcast the death event to everyone
    if (*self_).s.eType != ET_NPC && g_noPDuelCheck == QFALSE {
        let ent: *mut gentity_t = G_TempEntity(&(*self_).r.currentOrigin, EV_OBITUARY);
        (*ent).s.eventParm = meansOfDeath;
        (*ent).s.otherEntityNum = (*self_).s.number;
        if !attacker.is_null() {
            (*ent).s.otherEntityNum2 = (*attacker).s.number;
        } else {
            //???
            (*ent).s.otherEntityNum2 = killer;
        }
        if !inflictor.is_null()
            && Q_stricmp(c"vehicle_proj".as_ptr(), (*inflictor).classname as *const c_char) == 0
        {
            //a vehicle missile
            (*ent).s.eventParm = MOD_VEHICLE;
            //store index into g_vehWeaponInfo
            (*ent).s.weapon = (*inflictor).s.otherEntityNum2 + 1;
            //store generic rocket or blaster type of missile
            (*ent).s.generic1 = (*inflictor).s.weapon;
        }
        if wasInVehicle != 0 && (*self_).s.number < MAX_CLIENTS as c_int {
            //target is in a vehicle, store the entnum
            (*ent).s.lookTarget = wasInVehicle;
        }
        if !attacker.is_null() {
            if (*attacker).s.m_iVehicleNum != 0 && (*attacker).s.number < MAX_CLIENTS as c_int {
                //target is in a vehicle, store the entnum
                (*ent).s.brokenLimbs = (*attacker).s.m_iVehicleNum;
            } else if (*ent).s.lookTarget != 0
                && Q_stricmp(c"func_rotating".as_ptr(), (*attacker).classname as *const c_char) == 0
            {
                //my vehicle was killed by a func_rotating, probably an asteroid, so...
                (*ent).s.saberInFlight = QTRUE;
            }
        }
        (*ent).r.svFlags = SVF_BROADCAST; // send to everyone
        (*ent).s.isJediMaster = wasJediMaster;
    }
}

/// `static int G_CheckSpecialDeathAnim( gentity_t *self, vec3_t point, int damage, int mod,
/// int hitLoc )` (g_combat.c:881) — pick a context-sensitive death animation when the victim
/// is mid-roll, mid-flip, or in a knockdown/get-up animation, so the death blends naturally
/// out of the current pose. Returns the chosen `BOTH_*` anim, or `-1` to fall through to the
/// generic hit-location selector in [`G_PickDeathAnim`].
///
/// Faithful 1:1 with the C: `point`/`damage`/`mod`/`hitLoc` are unused (the choice is driven
/// entirely by `client->ps` — `legsAnim`, `legsTimer`, `viewangles`, `velocity` — plus the
/// knockdown anim's total length). `animLength` is `numFrames * fabs(frameLerp)` from the
/// loaded anim tables (`bgAllAnims[localAnimIndex].anims[legsAnim]` /
/// `bgHumanoidAnimations[legsAnim]`), matching the `as c_int` truncation used elsewhere.
///
/// Oracle-tested via [`oracle::jka_G_CheckSpecialDeathAnim`]: the verbatim body runs on a
/// reduced playerState (`legsAnim`/`legsTimer`/`viewangles`/`velocity`) with the
/// `BG_InRoll`/`BG_FlippingAnim` gates and the `numFrames`/`frameLerp` pair marshalled in, so
/// the test needs no anim-table or struct-layout match.
///
/// # Safety
/// `self_` must be valid with a non-NULL `client` (the C is only reached through
/// [`G_PickDeathAnim`]'s `self->client` guard); `localAnimIndex`/`legsAnim` must index the
/// loaded anim tables, as in the original.
pub unsafe fn G_CheckSpecialDeathAnim(
    self_: *mut gentity_t,
    _point: *const vec3_t,
    _damage: c_int,
    _mod_: c_int,
    _hit_loc: c_int,
) -> c_int {
    let mut death_anim: c_int = -1;

    let ps = addr_of!((*(*self_).client).ps);

    if BG_InRoll(addr_of_mut!((*(*self_).client).ps), (*ps).legsAnim) != QFALSE {
        death_anim = BOTH_DEATH_ROLL; //# Death anim from a roll
    } else if BG_FlippingAnim((*ps).legsAnim) != QFALSE {
        death_anim = BOTH_DEATH_FLIP; //# Death anim from a flip
    } else if G_InKnockDown(ps) != QFALSE {
        //since these happen a lot, let's handle them case by case
        let anim = *(*addr_of!(bgAllAnims))[(*self_).localAnimIndex as usize]
            .anims
            .add((*ps).legsAnim as usize);
        let hanim = (*addr_of!(bgHumanoidAnimations))[(*ps).legsAnim as usize];
        let anim_length: c_int =
            (anim.numFrames as f64 * (hanim.frameLerp as f32 as f64).abs()) as c_int;
        let legs_timer = (*ps).legsTimer;
        match (*ps).legsAnim {
            BOTH_KNOCKDOWN1 => {
                if anim_length - legs_timer > 100 {
                    //on our way down
                    if legs_timer > 600 {
                        //still partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_KNOCKDOWN2 => {
                if anim_length - legs_timer > 700 {
                    //on our way down
                    if legs_timer > 600 {
                        //still partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_KNOCKDOWN3 => {
                if anim_length - legs_timer > 100 {
                    //on our way down
                    if legs_timer > 1300 {
                        //still partially up
                        death_anim = BOTH_DEATH_FALLING_DN;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_DN;
                    }
                }
            }
            BOTH_KNOCKDOWN4 => {
                if anim_length - legs_timer > 300 {
                    //on our way down
                    if legs_timer > 350 {
                        //still partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                } else {
                    //crouch death
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                }
            }
            BOTH_KNOCKDOWN5 => {
                if legs_timer < 750 {
                    //flat
                    death_anim = BOTH_DEATH_LYING_DN;
                }
            }
            BOTH_GETUP1 => {
                if legs_timer < 350 {
                    //standing up
                } else if legs_timer < 800 {
                    //crouching
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                } else {
                    //lying down
                    if anim_length - legs_timer > 450 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_GETUP2 => {
                if legs_timer < 150 {
                    //standing up
                } else if legs_timer < 850 {
                    //crouching
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                } else {
                    //lying down
                    if anim_length - legs_timer > 500 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_GETUP3 => {
                if legs_timer < 250 {
                    //standing up
                } else if legs_timer < 600 {
                    //crouching
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                } else {
                    //lying down
                    if anim_length - legs_timer > 150 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_DN;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_DN;
                    }
                }
            }
            BOTH_GETUP4 => {
                if legs_timer < 250 {
                    //standing up
                } else if legs_timer < 600 {
                    //crouching
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                } else {
                    //lying down
                    if anim_length - legs_timer > 850 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_DN;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_GETUP5 => {
                if legs_timer > 850 {
                    //lying down
                    if anim_length - legs_timer > 1500 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_DN;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_DN;
                    }
                }
            }
            BOTH_GETUP_CROUCH_B1 => {
                if legs_timer < 800 {
                    //crouching
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                } else {
                    //lying down
                    if anim_length - legs_timer > 400 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_GETUP_CROUCH_F1 => {
                if legs_timer < 800 {
                    //crouching
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                } else {
                    //lying down
                    if anim_length - legs_timer > 150 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_DN;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_DN;
                    }
                }
            }
            BOTH_FORCE_GETUP_B1 => {
                if legs_timer < 325 {
                    //standing up
                } else if legs_timer < 725 {
                    //spinning up
                    death_anim = BOTH_DEATH_SPIN_180; //# Death anim when facing backwards
                } else if legs_timer < 900 {
                    //crouching
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                } else {
                    //lying down
                    if anim_length - legs_timer > 50 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_FORCE_GETUP_B2 => {
                if legs_timer < 575 {
                    //standing up
                } else if legs_timer < 875 {
                    //spinning up
                    death_anim = BOTH_DEATH_SPIN_180; //# Death anim when facing backwards
                } else if legs_timer < 900 {
                    //crouching
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                } else {
                    //lying down
                    //partially up
                    death_anim = BOTH_DEATH_FALLING_UP;
                }
            }
            BOTH_FORCE_GETUP_B3 => {
                if legs_timer < 150 {
                    //standing up
                } else if legs_timer < 775 {
                    //flipping
                    death_anim = BOTH_DEATHBACKWARD2; //backflip
                } else {
                    //lying down
                    //partially up
                    death_anim = BOTH_DEATH_FALLING_UP;
                }
            }
            BOTH_FORCE_GETUP_B4 => {
                if legs_timer < 325 {
                    //standing up
                } else {
                    //lying down
                    if anim_length - legs_timer > 150 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_FORCE_GETUP_B5 => {
                if legs_timer < 550 {
                    //standing up
                } else if legs_timer < 1025 {
                    //kicking up
                    death_anim = BOTH_DEATHBACKWARD2; //backflip
                } else {
                    //lying down
                    if anim_length - legs_timer > 50 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_FORCE_GETUP_B6 => {
                if legs_timer < 225 {
                    //standing up
                } else if legs_timer < 425 {
                    //crouching up
                    let mut fwd: vec3_t = [0.0; 3];
                    AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                    let thrown = DotProduct(&fwd, &(*ps).velocity);

                    if thrown < -150.0 {
                        death_anim = BOTH_DEATHBACKWARD1; //# Death anim when crouched and thrown back
                    } else {
                        death_anim = BOTH_DEATH_CROUCHED; //# Death anim when crouched
                    }
                } else if legs_timer < 825 {
                    //flipping up
                    death_anim = BOTH_DEATHFORWARD3; //backflip
                } else {
                    //lying down
                    if anim_length - legs_timer > 225 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_UP;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_UP;
                    }
                }
            }
            BOTH_FORCE_GETUP_F1 => {
                if legs_timer < 275 {
                    //standing up
                } else if legs_timer < 750 {
                    //flipping
                    death_anim = BOTH_DEATH14;
                } else {
                    //lying down
                    if anim_length - legs_timer > 100 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_DN;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_DN;
                    }
                }
            }
            BOTH_FORCE_GETUP_F2 => {
                if legs_timer < 1200 {
                    //standing
                } else {
                    //lying down
                    if anim_length - legs_timer > 225 {
                        //partially up
                        death_anim = BOTH_DEATH_FALLING_DN;
                    } else {
                        //down
                        death_anim = BOTH_DEATH_LYING_DN;
                    }
                }
            }
            _ => {}
        }
    }

    death_anim
}

/// `int G_PickDeathAnim( gentity_t *self, vec3_t point, int damage, int mod, int hitLoc )`
/// (g_combat.c:1373) — choose the death animation for a killed entity. Special-cases the
/// space-choke and avoid-dismember overrides, blends out of a roll/flip/knockdown via
/// [`G_CheckSpecialDeathAnim`], else picks by hit location with `damage`/`max_health`
/// buckets and `Q_irand` variety, then validates the choice against the loaded anim set
/// (falling back to a random `BOTH_DEATH1..BOTH_DEATH25` via [`BG_PickAnim`]). Also handles
/// the non-client g2animent (`ET_NPC`) path, which reads `s.legsAnim`/`s.pos.trDelta`.
///
/// Faithful 1:1: the dead-"flop" `switch` collapses to a single arm — every listed
/// already-dead/dying `legAnim` set `deathAnim = -2` through C fall-through (the
/// `PM_FinishedCurrentLegsAnim` flop blocks are commented out upstream), so the Rust match
/// lists them all in one `-2` arm. The `max_health` fraction tests use `f64` (the `0.25`
/// etc. are C `double` literals); `!VectorLengthSquared(objVelocity)` is `== 0.0`.
///
/// Oracle-tested via [`oracle::jka_G_PickDeathAnim`]: the verbatim hit-location selector
/// runs with the resolved `hitLoc`, `objVelocity`, `max_health`, and the
/// `G_CheckSpecialDeathAnim` result marshalled in. The Rust test seeds both the Rust and
/// oracle holdrand LCGs identically (so the in-lockstep `Q_irand` draws agree) and stocks the
/// anim table full so the validation tail never reaches `BG_PickAnim`; the trivial early
/// returns (non-client/non-NPC → 0, space-choke, avoid-dismember) are asserted directly.
///
/// # Safety
/// `self_` may be NULL or have a NULL `client` (both checked — NULL non-NPC returns 0);
/// otherwise valid as in the C. `point` must be a valid `vec3_t` when `hit_loc == HL_NONE`
/// (passed through to [`G_GetHitLocation`]).
pub unsafe fn G_PickDeathAnim(
    self_: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    mod_: c_int,
    mut hit_loc: c_int,
) -> c_int {
    //FIXME: play dead flop anims on body if in an appropriate _DEAD anim when this func is called
    let mut death_anim: c_int = -1;
    let max_health: c_int;
    let leg_anim: c_int;
    let mut obj_velocity: vec3_t = [0.0; 3];

    if self_.is_null() || (*self_).client.is_null() {
        if self_.is_null() || (*self_).s.eType != ET_NPC {
            //g2animent
            return 0;
        }
    }

    if !(*self_).client.is_null() {
        max_health = (*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize];

        if (*(*self_).client).inSpaceIndex != 0
            && (*(*self_).client).inSpaceIndex != ENTITYNUM_NONE
        {
            return BOTH_CHOKE3;
        }
    } else {
        max_health = 60;
    }

    if !(*self_).client.is_null() {
        VectorCopy(&(*(*self_).client).ps.velocity, &mut obj_velocity);
    } else {
        VectorCopy(&(*self_).s.pos.trDelta, &mut obj_velocity);
    }

    if hit_loc == HL_NONE {
        hit_loc = G_GetHitLocation(self_, point); //self->hitLoc
    }

    if !(*self_).client.is_null() {
        leg_anim = (*(*self_).client).ps.legsAnim;
    } else {
        leg_anim = (*self_).s.legsAnim;
    }

    if *addr_of!(gGAvoidDismember) != 0 {
        return BOTH_RIGHTHANDCHOPPEDOFF;
    }

    //dead flops
    // The C `switch` chains these cases via fall-through (the commented-out
    // PM_FinishedCurrentLegsAnim flop blocks aside), every one ending at `deathAnim = -2`.
    match leg_anim {
        BOTH_DEATH1 | BOTH_DEAD1 | BOTH_DEATH2 | BOTH_DEAD2 | BOTH_DEATH8 | BOTH_DEAD8
        | BOTH_DEATH13 | BOTH_DEAD13 | BOTH_DEATH14 | BOTH_DEAD14 | BOTH_DEATH16 | BOTH_DEAD16
        | BOTH_DEADBACKWARD1 | BOTH_DEADBACKWARD2 | BOTH_DEATH10 | BOTH_DEAD10 | BOTH_DEATH15
        | BOTH_DEAD15 | BOTH_DEADFORWARD1 | BOTH_DEADFORWARD2 | BOTH_DEADFLOP1 | BOTH_DEAD3
        | BOTH_DEAD4 | BOTH_DEAD5 | BOTH_DEAD6 | BOTH_DEAD7 | BOTH_DEAD9 | BOTH_DEAD11
        | BOTH_DEAD12 | BOTH_DEAD17 | BOTH_DEAD18 | BOTH_DEAD19 | BOTH_LYINGDEAD1
        | BOTH_STUMBLEDEAD1 | BOTH_FALLDEAD1LAND | BOTH_DEATH3 | BOTH_DEATH4 | BOTH_DEATH5
        | BOTH_DEATH6 | BOTH_DEATH7 | BOTH_DEATH9 | BOTH_DEATH11 | BOTH_DEATH12 | BOTH_DEATH17
        | BOTH_DEATH18 | BOTH_DEATH19 | BOTH_DEATHFORWARD1 | BOTH_DEATHFORWARD2 | BOTH_DEATH1IDLE
        | BOTH_LYINGDEATH1 | BOTH_STUMBLEDEATH1 | BOTH_FALLDEATH1 | BOTH_FALLDEATH1INAIR
        | BOTH_FALLDEATH1LAND => {
            death_anim = -2;
        }
        _ => {}
    }

    if death_anim == -1 {
        if !(*self_).client.is_null() {
            death_anim = G_CheckSpecialDeathAnim(self_, point, damage, mod_, hit_loc);
        }

        if death_anim == -1 {
            //death anims
            match hit_loc {
                HL_FOOT_RT | HL_FOOT_LT => {
                    if mod_ == MOD_SABER && Q_irand(0, 2) == 0 {
                        return BOTH_DEATH10; //chest: back flip
                    } else if Q_irand(0, 2) == 0 {
                        death_anim = BOTH_DEATH4; //back: forward
                    } else if Q_irand(0, 1) == 0 {
                        death_anim = BOTH_DEATH5; //same as 4
                    } else {
                        death_anim = BOTH_DEATH15; //back: forward
                    }
                }
                HL_LEG_RT => {
                    if Q_irand(0, 2) == 0 {
                        death_anim = BOTH_DEATH4; //back: forward
                    } else if Q_irand(0, 1) == 0 {
                        death_anim = BOTH_DEATH5; //same as 4
                    } else {
                        death_anim = BOTH_DEATH15; //back: forward
                    }
                }
                HL_LEG_LT => {
                    if Q_irand(0, 2) == 0 {
                        death_anim = BOTH_DEATH4; //back: forward
                    } else if Q_irand(0, 1) == 0 {
                        death_anim = BOTH_DEATH5; //same as 4
                    } else {
                        death_anim = BOTH_DEATH15; //back: forward
                    }
                }
                HL_BACK => {
                    if VectorLengthSquared(&obj_velocity) == 0.0 {
                        death_anim = BOTH_DEATH17; //head/back: croak
                    } else if Q_irand(0, 2) == 0 {
                        death_anim = BOTH_DEATH4; //back: forward
                    } else if Q_irand(0, 1) == 0 {
                        death_anim = BOTH_DEATH5; //same as 4
                    } else {
                        death_anim = BOTH_DEATH15; //back: forward
                    }
                }
                HL_CHEST_RT | HL_ARM_RT | HL_HAND_RT | HL_BACK_RT => {
                    if (damage as f64) <= max_health as f64 * 0.25 {
                        death_anim = BOTH_DEATH9; //chest right: snap, fall forward
                    } else if (damage as f64) <= max_health as f64 * 0.5 {
                        death_anim = BOTH_DEATH3; //chest right: back
                    } else if (damage as f64) <= max_health as f64 * 0.75 {
                        death_anim = BOTH_DEATH6; //chest right: spin
                    } else {
                        //TEMP HACK: play spinny deaths less often
                        if Q_irand(0, 1) != 0 {
                            death_anim = BOTH_DEATH8; //chest right: spin high
                        } else {
                            match Q_irand(0, 2) {
                                1 => {
                                    death_anim = BOTH_DEATH3; //chest right: back
                                }
                                2 => {
                                    death_anim = BOTH_DEATH6; //chest right: spin
                                }
                                _ => {
                                    death_anim = BOTH_DEATH9; //chest right: snap, fall forward
                                }
                            }
                        }
                    }
                }
                HL_CHEST_LT | HL_ARM_LT | HL_HAND_LT | HL_BACK_LT => {
                    if (damage as f64) <= max_health as f64 * 0.25 {
                        death_anim = BOTH_DEATH11; //chest left: snap, fall forward
                    } else if (damage as f64) <= max_health as f64 * 0.5 {
                        death_anim = BOTH_DEATH7; //chest left: back
                    } else if (damage as f64) <= max_health as f64 * 0.75 {
                        death_anim = BOTH_DEATH12; //chest left: spin
                    } else {
                        //TEMP HACK: play spinny deaths less often
                        if Q_irand(0, 1) != 0 {
                            death_anim = BOTH_DEATH14; //chest left: spin high
                        } else {
                            match Q_irand(0, 2) {
                                1 => {
                                    death_anim = BOTH_DEATH7; //chest left: back
                                }
                                2 => {
                                    death_anim = BOTH_DEATH12; //chest left: spin
                                }
                                _ => {
                                    death_anim = BOTH_DEATH11; //chest left: snap, fall forward
                                }
                            }
                        }
                    }
                }
                HL_CHEST | HL_WAIST => {
                    if (damage as f64) <= max_health as f64 * 0.25
                        || VectorLengthSquared(&obj_velocity) == 0.0
                    {
                        if Q_irand(0, 1) == 0 {
                            death_anim = BOTH_DEATH18; //gut: fall right
                        } else {
                            death_anim = BOTH_DEATH19; //gut: fall left
                        }
                    } else if (damage as f64) <= max_health as f64 * 0.5 {
                        death_anim = BOTH_DEATH2; //chest: backward short
                    } else if (damage as f64) <= max_health as f64 * 0.75 {
                        if Q_irand(0, 1) == 0 {
                            death_anim = BOTH_DEATH1; //chest: backward med
                        } else {
                            death_anim = BOTH_DEATH16; //same as 1
                        }
                    } else {
                        death_anim = BOTH_DEATH10; //chest: back flip
                    }
                }
                HL_HEAD => {
                    if (damage as f64) <= max_health as f64 * 0.5 {
                        death_anim = BOTH_DEATH17; //head/back: croak
                    } else {
                        death_anim = BOTH_DEATH13; //head: stumble, fall back
                    }
                }
                _ => {}
            }
        }
    }

    // Validate.....
    if death_anim == -1 || BG_HasAnimation((*self_).localAnimIndex, death_anim) == QFALSE {
        // I guess we'll take what we can get.....
        death_anim = BG_PickAnim((*self_).localAnimIndex, BOTH_DEATH1, BOTH_DEATH25);
    }

    death_anim
}

/// `qboolean G_GetHitLocFromSurfName( gentity_t *ent, const char *surfName, int *hitLoc,
/// vec3_t point, vec3_t dir, vec3_t bladeDir, int mod )` (g_combat.c:3632) — map an
/// impacted ghoul2 surface name to a hit location (`*hitLoc`) and decide whether the hit
/// dismembers. Humanoids resolve limb surfaces by bolt-position proximity; the ATST (and
/// the disabled-in-the-original MARK1/MARK2/GALAKMECH droids) use hard-coded surface maps.
/// Dismemberment requires `g_dismember == 100` and, when a direction is supplied, the
/// impact and blade directions to be roughly perpendicular to the limb's cap bolt.
///
/// No oracle — drives ghoul2 bolt traps and reads the global `level`/`g_dismember`.
///
/// # Safety
/// `ent` must be valid with a non-NULL `client`; `surf_name` a NUL-terminated string;
/// `hit_loc` a valid `*mut c_int`; `point` non-NULL (checked); `dir`/`blade_dir` may be
/// NULL (checked).
pub unsafe fn G_GetHitLocFromSurfName(
    ent: *mut gentity_t,
    surf_name: *const c_char,
    hit_loc: *mut c_int,
    point: *const vec3_t,
    dir: *const vec3_t,
    blade_dir: *const vec3_t,
    mod_: c_int,
) -> qboolean {
    let mut dismember: qboolean = QFALSE;
    let actual_time: c_int;
    let mut knee_l_bolt: c_int = -1;
    let mut knee_r_bolt: c_int = -1;
    let mut hand_r_bolt: c_int = -1;
    let mut hand_l_bolt: c_int = -1;
    let mut foot_r_bolt: c_int = -1;
    let mut foot_l_bolt: c_int = -1;

    *hit_loc = HL_NONE;

    if surf_name.is_null() || *surf_name == 0 {
        return QFALSE;
    }

    if (*ent).client.is_null() {
        return QFALSE;
    }

    if point.is_null() {
        return QFALSE;
    }

    let client = (*ent).client;

    if !client.is_null()
        && ((*client).NPC_class == CLASS_R2D2
            || (*client).NPC_class == CLASS_R2D2
            || (*client).NPC_class == CLASS_GONK
            || (*client).NPC_class == CLASS_MOUSE
            || (*client).NPC_class == CLASS_SENTRY
            || (*client).NPC_class == CLASS_INTERROGATOR
            || (*client).NPC_class == CLASS_SENTRY
            || (*client).NPC_class == CLASS_PROBE)
    {
        // we don't care about per-surface hit-locations or dismemberment for these guys
        return QFALSE;
    }

    if (*ent).localAnimIndex <= 1 {
        // humanoid
        hand_l_bolt = trap::G2API_AddBolt((*ent).ghoul2, 0, "*l_hand");
        hand_r_bolt = trap::G2API_AddBolt((*ent).ghoul2, 0, "*r_hand");
        knee_l_bolt = trap::G2API_AddBolt((*ent).ghoul2, 0, "*hips_l_knee");
        knee_r_bolt = trap::G2API_AddBolt((*ent).ghoul2, 0, "*hips_r_knee");
        foot_l_bolt = trap::G2API_AddBolt((*ent).ghoul2, 0, "*l_leg_foot");
        foot_r_bolt = trap::G2API_AddBolt((*ent).ghoul2, 0, "*r_leg_foot");
    }

    if !(*ent).client.is_null() && (*client).NPC_class == CLASS_ATST {
        // FIXME: almost impossible to hit these... perhaps radius damage to these parts?
        if Q_stricmp(c"head_light_blaster_cann".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_ARM_LT;
        } else if Q_stricmp(c"head_concussion_charger".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_ARM_RT;
        }
        return QFALSE;
    } else if !(*ent).client.is_null() && (*client).NPC_class == CLASS_MARK1 {
        if Q_stricmp(c"l_arm".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_ARM_LT;
        } else if Q_stricmp(c"r_arm".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_ARM_RT;
        } else if Q_stricmp(c"torso_front".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_CHEST;
        } else if Q_stricmp(c"torso_tube1".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC1;
        } else if Q_stricmp(c"torso_tube2".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC2;
        } else if Q_stricmp(c"torso_tube3".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC3;
        } else if Q_stricmp(c"torso_tube4".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC4;
        } else if Q_stricmp(c"torso_tube5".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC5;
        } else if Q_stricmp(c"torso_tube6".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC6;
        }
        return QFALSE;
    } else if !(*ent).client.is_null() && (*client).NPC_class == CLASS_MARK2 {
        if Q_stricmp(c"torso_canister1".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC1;
        } else if Q_stricmp(c"torso_canister2".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC2;
        } else if Q_stricmp(c"torso_canister3".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC3;
        }
        return QFALSE;
    } else if !(*ent).client.is_null() && (*client).NPC_class == CLASS_GALAKMECH {
        if Q_stricmp(c"torso_antenna".as_ptr(), surf_name) == 0
            || Q_stricmp(c"torso_antenna_base".as_ptr(), surf_name) == 0
        {
            *hit_loc = HL_GENERIC1;
        } else if Q_stricmp(c"torso_shield".as_ptr(), surf_name) == 0 {
            *hit_loc = HL_GENERIC2;
        } else {
            *hit_loc = HL_CHEST;
        }
        return QFALSE;
    }

    // FIXME: check the hitLoc and hitDir against the cap tag for the place where the
    // split will be — only allow dismember when the hit dir is roughly perpendicular.
    actual_time = (*addr_of!(level)).time;
    if Q_strncmp(c"hips".as_ptr(), surf_name, 4) == 0 {
        // FIXME: test properly for legs
        *hit_loc = HL_WAIST;
        if !(*ent).client.is_null() && !(*ent).ghoul2.is_null() {
            let mut bolt_matrix: mdxaBone_t = core::mem::zeroed();
            let mut tag_org: vec3_t = [0.0; 3];
            let mut angles: vec3_t = [0.0; 3];

            VectorSet(&mut angles, 0.0, (*ent).r.currentAngles[YAW], 0.0);
            if knee_l_bolt >= 0 {
                trap::G2API_GetBoltMatrix(
                    (*ent).ghoul2,
                    0,
                    knee_l_bolt,
                    &mut bolt_matrix,
                    &angles,
                    &(*ent).r.currentOrigin,
                    actual_time,
                    null_mut(),
                    &(*ent).modelScale,
                );
                BG_GiveMeVectorFromMatrix(&bolt_matrix, ORIGIN, &mut tag_org);
                if DistanceSquared(&*point, &tag_org) < 100.0 {
                    // actually hit the knee
                    *hit_loc = HL_LEG_LT;
                }
            }
            if *hit_loc == HL_WAIST && knee_r_bolt >= 0 {
                trap::G2API_GetBoltMatrix(
                    (*ent).ghoul2,
                    0,
                    knee_r_bolt,
                    &mut bolt_matrix,
                    &angles,
                    &(*ent).r.currentOrigin,
                    actual_time,
                    null_mut(),
                    &(*ent).modelScale,
                );
                BG_GiveMeVectorFromMatrix(&bolt_matrix, ORIGIN, &mut tag_org);
                if DistanceSquared(&*point, &tag_org) < 100.0 {
                    // actually hit the knee
                    *hit_loc = HL_LEG_RT;
                }
            }
        }
    } else if Q_strncmp(c"torso".as_ptr(), surf_name, 5) == 0 {
        if (*ent).client.is_null() {
            *hit_loc = HL_CHEST;
        } else {
            let mut t_fwd: vec3_t = [0.0; 3];
            let mut t_rt: vec3_t = [0.0; 3];
            let mut t_up: vec3_t = [0.0; 3];
            let mut dir_to_impact: vec3_t = [0.0; 3];
            AngleVectors(
                &(*client).renderInfo.torsoAngles,
                Some(&mut t_fwd),
                Some(&mut t_rt),
                Some(&mut t_up),
            );

            if (*client).renderInfo.boltValidityTime != (*addr_of!(level)).time {
                let mut render_ang: vec3_t = [0.0; 3];
                render_ang[0] = 0.0;
                render_ang[1] = (*client).ps.viewangles[YAW];
                render_ang[2] = 0.0;
                UpdateClientRenderBolts(ent, &(*client).ps.origin, &render_ang);
            }

            VectorSubtract(&*point, &(*client).renderInfo.torsoPoint, &mut dir_to_impact);
            let front_side = DotProduct(&t_fwd, &dir_to_impact);
            let right_side = DotProduct(&t_rt, &dir_to_impact);
            let up_side = DotProduct(&t_up, &dir_to_impact);
            if up_side < -10.0 {
                // hit at waist
                *hit_loc = HL_WAIST;
            } else {
                // hit on upper torso
                if right_side > 4.0 {
                    *hit_loc = HL_ARM_RT;
                } else if right_side < -4.0 {
                    *hit_loc = HL_ARM_LT;
                } else if right_side > 2.0 {
                    if front_side > 0.0 {
                        *hit_loc = HL_CHEST_RT;
                    } else {
                        *hit_loc = HL_BACK_RT;
                    }
                } else if right_side < -2.0 {
                    if front_side > 0.0 {
                        *hit_loc = HL_CHEST_LT;
                    } else {
                        *hit_loc = HL_BACK_LT;
                    }
                } else if up_side > -3.0 && mod_ == MOD_SABER {
                    *hit_loc = HL_HEAD;
                } else if front_side > 0.0 {
                    *hit_loc = HL_CHEST;
                } else {
                    *hit_loc = HL_BACK;
                }
            }
        }
    } else if Q_strncmp(c"head".as_ptr(), surf_name, 4) == 0 {
        *hit_loc = HL_HEAD;
    } else if Q_strncmp(c"r_arm".as_ptr(), surf_name, 5) == 0 {
        *hit_loc = HL_ARM_RT;
        if !(*ent).client.is_null() && !(*ent).ghoul2.is_null() && hand_r_bolt >= 0 {
            let mut bolt_matrix: mdxaBone_t = core::mem::zeroed();
            let mut tag_org: vec3_t = [0.0; 3];
            let mut angles: vec3_t = [0.0; 3];
            VectorSet(&mut angles, 0.0, (*ent).r.currentAngles[YAW], 0.0);
            trap::G2API_GetBoltMatrix(
                (*ent).ghoul2,
                0,
                hand_r_bolt,
                &mut bolt_matrix,
                &angles,
                &(*ent).r.currentOrigin,
                actual_time,
                null_mut(),
                &(*ent).modelScale,
            );
            BG_GiveMeVectorFromMatrix(&bolt_matrix, ORIGIN, &mut tag_org);
            if DistanceSquared(&*point, &tag_org) < 256.0 {
                // actually hit the hand
                *hit_loc = HL_HAND_RT;
            }
        }
    } else if Q_strncmp(c"l_arm".as_ptr(), surf_name, 5) == 0 {
        *hit_loc = HL_ARM_LT;
        if !(*ent).client.is_null() && !(*ent).ghoul2.is_null() && hand_l_bolt >= 0 {
            let mut bolt_matrix: mdxaBone_t = core::mem::zeroed();
            let mut tag_org: vec3_t = [0.0; 3];
            let mut angles: vec3_t = [0.0; 3];
            VectorSet(&mut angles, 0.0, (*ent).r.currentAngles[YAW], 0.0);
            trap::G2API_GetBoltMatrix(
                (*ent).ghoul2,
                0,
                hand_l_bolt,
                &mut bolt_matrix,
                &angles,
                &(*ent).r.currentOrigin,
                actual_time,
                null_mut(),
                &(*ent).modelScale,
            );
            BG_GiveMeVectorFromMatrix(&bolt_matrix, ORIGIN, &mut tag_org);
            if DistanceSquared(&*point, &tag_org) < 256.0 {
                // actually hit the hand
                *hit_loc = HL_HAND_LT;
            }
        }
    } else if Q_strncmp(c"r_leg".as_ptr(), surf_name, 5) == 0 {
        *hit_loc = HL_LEG_RT;
        if !(*ent).client.is_null() && !(*ent).ghoul2.is_null() && foot_r_bolt >= 0 {
            let mut bolt_matrix: mdxaBone_t = core::mem::zeroed();
            let mut tag_org: vec3_t = [0.0; 3];
            let mut angles: vec3_t = [0.0; 3];
            VectorSet(&mut angles, 0.0, (*ent).r.currentAngles[YAW], 0.0);
            trap::G2API_GetBoltMatrix(
                (*ent).ghoul2,
                0,
                foot_r_bolt,
                &mut bolt_matrix,
                &angles,
                &(*ent).r.currentOrigin,
                actual_time,
                null_mut(),
                &(*ent).modelScale,
            );
            BG_GiveMeVectorFromMatrix(&bolt_matrix, ORIGIN, &mut tag_org);
            if DistanceSquared(&*point, &tag_org) < 100.0 {
                // actually hit the foot
                *hit_loc = HL_FOOT_RT;
            }
        }
    } else if Q_strncmp(c"l_leg".as_ptr(), surf_name, 5) == 0 {
        *hit_loc = HL_LEG_LT;
        if !(*ent).client.is_null() && !(*ent).ghoul2.is_null() && foot_l_bolt >= 0 {
            let mut bolt_matrix: mdxaBone_t = core::mem::zeroed();
            let mut tag_org: vec3_t = [0.0; 3];
            let mut angles: vec3_t = [0.0; 3];
            VectorSet(&mut angles, 0.0, (*ent).r.currentAngles[YAW], 0.0);
            trap::G2API_GetBoltMatrix(
                (*ent).ghoul2,
                0,
                foot_l_bolt,
                &mut bolt_matrix,
                &angles,
                &(*ent).r.currentOrigin,
                actual_time,
                null_mut(),
                &(*ent).modelScale,
            );
            BG_GiveMeVectorFromMatrix(&bolt_matrix, ORIGIN, &mut tag_org);
            if DistanceSquared(&*point, &tag_org) < 100.0 {
                // actually hit the foot
                *hit_loc = HL_FOOT_LT;
            }
        }
    } else if Q_strncmp(c"r_hand".as_ptr(), surf_name, 6) == 0
        || Q_strncmp(c"w_".as_ptr(), surf_name, 2) == 0
    {
        // right hand or weapon
        *hit_loc = HL_HAND_RT;
    } else if Q_strncmp(c"l_hand".as_ptr(), surf_name, 6) == 0 {
        *hit_loc = HL_HAND_LT;
    }

    // if ( g_dismemberment->integer >= 11381138 || !ent->client->dismembered )
    if (*addr_of!(g_dismember)).integer == 100 {
        // full probability...
        if !(*ent).client.is_null() && (*client).NPC_class == CLASS_PROTOCOL {
            dismember = QTRUE;
        } else if !dir.is_null()
            && ((*dir)[0] != 0.0 || (*dir)[1] != 0.0 || (*dir)[2] != 0.0)
            && !blade_dir.is_null()
            && ((*blade_dir)[0] != 0.0 || (*blade_dir)[1] != 0.0 || (*blade_dir)[2] != 0.0)
        {
            // we care about direction (presumably for dismemberment)
            // either we don't care about probabilties or the probability let us continue
            #[allow(clippy::if_same_then_else)]
            if true {
                // Fix me?
                let mut tag_name: Option<&str> = None;
                let mut aoa: f32 = 0.5;
                // dir must be roughly perpendicular to the hitLoc's cap bolt
                match *hit_loc {
                    HL_LEG_RT => tag_name = Some("*hips_cap_r_leg"),
                    HL_LEG_LT => tag_name = Some("*hips_cap_l_leg"),
                    HL_WAIST => {
                        tag_name = Some("*hips_cap_torso");
                        aoa = 0.25;
                    }
                    HL_CHEST_RT | HL_ARM_RT | HL_BACK_LT => tag_name = Some("*torso_cap_r_arm"),
                    HL_CHEST_LT | HL_ARM_LT | HL_BACK_RT => tag_name = Some("*torso_cap_l_arm"),
                    HL_HAND_RT => tag_name = Some("*r_arm_cap_r_hand"),
                    HL_HAND_LT => tag_name = Some("*l_arm_cap_l_hand"),
                    HL_HEAD => {
                        tag_name = Some("*torso_cap_head");
                        aoa = 0.25;
                    }
                    // HL_CHEST | HL_BACK | HL_FOOT_RT | HL_FOOT_LT and default:
                    // no dismemberment possible with these, so no checks needed
                    _ => {}
                }
                if let Some(tn) = tag_name {
                    let tag_bolt = trap::G2API_AddBolt((*ent).ghoul2, 0, tn);
                    if tag_bolt != -1 {
                        let mut bolt_matrix: mdxaBone_t = core::mem::zeroed();
                        let mut tag_org: vec3_t = [0.0; 3];
                        let mut tag_dir: vec3_t = [0.0; 3];
                        let mut angles: vec3_t = [0.0; 3];

                        VectorSet(&mut angles, 0.0, (*ent).r.currentAngles[YAW], 0.0);
                        trap::G2API_GetBoltMatrix(
                            (*ent).ghoul2,
                            0,
                            tag_bolt,
                            &mut bolt_matrix,
                            &angles,
                            &(*ent).r.currentOrigin,
                            actual_time,
                            null_mut(),
                            &(*ent).modelScale,
                        );
                        BG_GiveMeVectorFromMatrix(&bolt_matrix, ORIGIN, &mut tag_org);
                        BG_GiveMeVectorFromMatrix(&bolt_matrix, NEGATIVE_Y, &mut tag_dir);
                        if DistanceSquared(&*point, &tag_org) < 256.0 {
                            // hit close
                            let mut dot = DotProduct(&*dir, &tag_dir);
                            if dot < aoa && dot > -aoa {
                                // hit roughly perpendicular
                                dot = DotProduct(&*blade_dir, &tag_dir);
                                if dot < aoa && dot > -aoa {
                                    // blade was roughly perpendicular
                                    dismember = QTRUE;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // hmm, no direction supplied.
            dismember = QTRUE;
        }
    }
    dismember
}

/// `void G_LocationBasedDamageModifier(gentity_t *ent, vec3_t point, int mod, int dflags,
/// int *damage)` (g_combat.c:4227) — scale `*damage` by where on the body it landed.
/// Skipped entirely when `g_locationBasedDamage` is off, for `DAMAGE_NO_HIT_LOC`, and for
/// idle (≤1) saber damage. The hit location comes from the last ghoul2-collision surface
/// (when saber/projectile ghoul2 collision is active) via [`G_GetHitLocFromSurfName`],
/// otherwise from the geometric [`G_GetHitLocation`]. Feet ×0.5, legs ×0.7, arms ×0.85,
/// hands ×0.6, head ×1.3; torso/back/waist unchanged.
///
/// No oracle — reads global cvars/`level` and drives a ghoul2 trap. The multipliers use
/// `f64` to match C's `double` literal arithmetic before the truncating `int` store.
///
/// # Safety
/// `ent` must be valid; `point` may be NULL (checked) else a `vec3_t`; `damage` a valid
/// `*mut c_int`.
pub unsafe fn G_LocationBasedDamageModifier(
    ent: *mut gentity_t,
    point: *const vec3_t,
    mod_: c_int,
    dflags: c_int,
    damage: *mut c_int,
) {
    let mut hit_loc: c_int = -1;

    if (*addr_of!(g_locationBasedDamage)).integer == 0 {
        // then leave it alone
        return;
    }

    if dflags & DAMAGE_NO_HIT_LOC != 0 {
        // then leave it alone
        return;
    }

    if mod_ == MOD_SABER && *damage <= 1 {
        // don't bother for idle damage
        return;
    }

    if point.is_null() {
        return;
    }

    if !(*ent).client.is_null() && (*(*ent).client).NPC_class == CLASS_VEHICLE {
        // no location-based damage on vehicles
        return;
    }

    if ((*addr_of!(d_saberGhoul2Collision)).integer != 0
        && !(*ent).client.is_null()
        && (*(*ent).client).g2LastSurfaceTime == (*addr_of!(level)).time
        && mod_ == MOD_SABER)
        || ((*addr_of!(d_projectileGhoul2Collision)).integer != 0
            && !(*ent).client.is_null()
            && (*(*ent).client).g2LastSurfaceTime == (*addr_of!(level)).time)
    {
        // using ghoul2 collision? Then the last-hit surface data is current.
        let mut hit_surface: [c_char; MAX_QPATH] = [0; MAX_QPATH];

        trap::G2API_GetSurfaceName(
            (*ent).ghoul2,
            (*(*ent).client).g2LastSurfaceHit,
            0,
            hit_surface.as_mut_ptr(),
        );

        if hit_surface[0] != 0 {
            G_GetHitLocFromSurfName(
                ent,
                hit_surface.as_ptr(),
                &mut hit_loc,
                point,
                &vec3_origin as *const vec3_t,
                &vec3_origin as *const vec3_t,
                MOD_UNKNOWN,
            );
        }
    }

    if hit_loc == -1 {
        hit_loc = G_GetHitLocation(ent, point);
    }

    match hit_loc {
        HL_FOOT_RT | HL_FOOT_LT => {
            *damage = (*damage as f64 * 0.5) as c_int;
        }
        HL_LEG_RT | HL_LEG_LT => {
            *damage = (*damage as f64 * 0.7) as c_int;
        }
        HL_WAIST | HL_BACK_RT | HL_BACK_LT | HL_BACK | HL_CHEST_RT | HL_CHEST_LT | HL_CHEST => {
            // normal damage
        }
        HL_ARM_RT | HL_ARM_LT => {
            *damage = (*damage as f64 * 0.85) as c_int;
        }
        HL_HAND_RT | HL_HAND_LT => {
            *damage = (*damage as f64 * 0.6) as c_int;
        }
        HL_HEAD => {
            *damage = (*damage as f64 * 1.3) as c_int;
        }
        _ => {
            // do nothing then
        }
    }
}

/// `void G_GetDismemberLoc(gentity_t *self, vec3_t boltPoint, int limbType)`
/// (g_combat.c:3069) — approximate the world position of a severed limb *without*
/// consulting server-side ghoul2, by offsetting the entity origin along its up/right
/// axes per `limbType`. Used for non-client (NPC) dismemberment, where the precise
/// ghoul2-bolt path ([`G_GetDismemberBolt`]) isn't taken.
///
/// Pure vector math (`AngleVectors` + per-part offsets); oracle-checked bit-exact via
/// [`tests`] against the extracted C across an angle/origin sweep for every limb type.
///
/// # Safety
/// `self_` must be a valid entity (only `r.currentAngles`/`r.currentOrigin` are read);
/// `bolt_point` a valid `*mut vec3_t` (written).
pub unsafe fn G_GetDismemberLoc(self_: *mut gentity_t, bolt_point: *mut vec3_t, limb_type: c_int) {
    // Just get the general area without using server-side ghoul2
    let mut fwd: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    AngleVectors(
        &(*self_).r.currentAngles,
        Some(&mut fwd),
        Some(&mut right),
        Some(&mut up),
    );

    VectorCopy(&(*self_).r.currentOrigin, &mut *bolt_point);

    let bp = &mut *bolt_point;
    match limb_type {
        G2_MODELPART_HEAD => {
            bp[0] += up[0] * 24.0;
            bp[1] += up[1] * 24.0;
            bp[2] += up[2] * 24.0;
        }
        G2_MODELPART_WAIST => {
            bp[0] += up[0] * 4.0;
            bp[1] += up[1] * 4.0;
            bp[2] += up[2] * 4.0;
        }
        G2_MODELPART_LARM => {
            bp[0] += up[0] * 18.0;
            bp[1] += up[1] * 18.0;
            bp[2] += up[2] * 18.0;

            bp[0] -= right[0] * 10.0;
            bp[1] -= right[1] * 10.0;
            bp[2] -= right[2] * 10.0;
        }
        G2_MODELPART_RARM => {
            bp[0] += up[0] * 18.0;
            bp[1] += up[1] * 18.0;
            bp[2] += up[2] * 18.0;

            bp[0] += right[0] * 10.0;
            bp[1] += right[1] * 10.0;
            bp[2] += right[2] * 10.0;
        }
        G2_MODELPART_RHAND => {
            bp[0] += up[0] * 8.0;
            bp[1] += up[1] * 8.0;
            bp[2] += up[2] * 8.0;

            bp[0] += right[0] * 10.0;
            bp[1] += right[1] * 10.0;
            bp[2] += right[2] * 10.0;
        }
        G2_MODELPART_LLEG => {
            bp[0] -= up[0] * 4.0;
            bp[1] -= up[1] * 4.0;
            bp[2] -= up[2] * 4.0;

            bp[0] -= right[0] * 10.0;
            bp[1] -= right[1] * 10.0;
            bp[2] -= right[2] * 10.0;
        }
        G2_MODELPART_RLEG => {
            bp[0] -= up[0] * 4.0;
            bp[1] -= up[1] * 4.0;
            bp[2] -= up[2] * 4.0;

            bp[0] += right[0] * 10.0;
            bp[1] += right[1] * 10.0;
            bp[2] += right[2] * 10.0;
        }
        _ => {}
    }
}

/// `void G_GetDismemberBolt(gentity_t *self, vec3_t boltPoint, int limbType)`
/// (g_combat.c:3141) — the precise client-dismemberment limb position: add a ghoul2
/// bolt on the limb's rotate bone, predict the render origin from velocity, fetch the
/// bolt matrix, and return its translation in `boltPoint`. For a severed right hand it
/// also spits saber-hit sparks (`EV_SABER_HIT`) over the wrist stub.
///
/// No oracle — drives the `G2API_AddBolt`/`G2API_GetBoltMatrix` traps, spawns a temp
/// entity, and reads the global `level`, like its ghoul2-driven siblings. The velocity
/// scale promotes through `f64` to match C's `float *= 0.08` (double) rounding.
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client` (its `ps`/`ghoul2`/
/// `modelScale` are read); `bolt_point` a valid `*mut vec3_t` (written).
pub unsafe fn G_GetDismemberBolt(self_: *mut gentity_t, bolt_point: *mut vec3_t, limb_type: c_int) {
    // C dead-initializes `useBolt` from self->genericValue5; it is overwritten by the
    // AddBolt result before any read, so we bind it directly.
    let mut proper_origin: vec3_t = [0.0; 3];
    let mut proper_angles: vec3_t = [0.0; 3];
    let mut add_vel: vec3_t = [0.0; 3];
    let mut bolt_matrix: mdxaBone_t = core::mem::zeroed();
    let mut f_v_speed: f32 = 0.0;

    let rotate_bone: &str = match limb_type {
        G2_MODELPART_HEAD => "cranium",
        G2_MODELPART_WAIST => {
            if (*self_).localAnimIndex <= 1 {
                // humanoid
                "thoracic"
            } else {
                "pelvis"
            }
        }
        G2_MODELPART_LARM => "lradius",
        G2_MODELPART_RARM => "rradius",
        G2_MODELPART_RHAND => "rhand",
        G2_MODELPART_LLEG => "ltibia",
        G2_MODELPART_RLEG => "rtibia",
        _ => "rtibia",
    };

    let use_bolt = trap::G2API_AddBolt((*self_).ghoul2, 0, rotate_bone);

    let client = (*self_).client;
    VectorCopy(&(*client).ps.origin, &mut proper_origin);
    VectorCopy(&(*client).ps.viewangles, &mut proper_angles);

    // try to predict the origin based on velocity so it's more like what the client is seeing
    VectorCopy(&(*client).ps.velocity, &mut add_vel);
    VectorNormalize(&mut add_vel);

    if (*client).ps.velocity[0] < 0.0 {
        f_v_speed += -(*client).ps.velocity[0];
    } else {
        f_v_speed += (*client).ps.velocity[0];
    }
    if (*client).ps.velocity[1] < 0.0 {
        f_v_speed += -(*client).ps.velocity[1];
    } else {
        f_v_speed += (*client).ps.velocity[1];
    }
    if (*client).ps.velocity[2] < 0.0 {
        f_v_speed += -(*client).ps.velocity[2];
    } else {
        f_v_speed += (*client).ps.velocity[2];
    }

    f_v_speed = (f_v_speed as f64 * 0.08) as f32;

    proper_origin[0] += add_vel[0] * f_v_speed;
    proper_origin[1] += add_vel[1] * f_v_speed;
    proper_origin[2] += add_vel[2] * f_v_speed;

    proper_angles[0] = 0.0;
    proper_angles[1] = (*client).ps.viewangles[YAW];
    proper_angles[2] = 0.0;

    trap::G2API_GetBoltMatrix(
        (*self_).ghoul2,
        0,
        use_bolt,
        &mut bolt_matrix,
        &proper_angles,
        &proper_origin,
        (*addr_of!(level)).time,
        null_mut(),
        &(*self_).modelScale,
    );

    let bp = &mut *bolt_point;
    bp[0] = bolt_matrix.matrix[0][3];
    bp[1] = bolt_matrix.matrix[1][3];
    bp[2] = bolt_matrix.matrix[2][3];

    trap::G2API_GetBoltMatrix(
        (*self_).ghoul2,
        1,
        0,
        &mut bolt_matrix,
        &proper_angles,
        &proper_origin,
        (*addr_of!(level)).time,
        null_mut(),
        &(*self_).modelScale,
    );

    if !(*self_).client.is_null() && limb_type == G2_MODELPART_RHAND {
        // Make some saber hit sparks over the severed wrist area
        let mut bolt_angles: vec3_t = [0.0; 3];

        bolt_angles[0] = -bolt_matrix.matrix[0][1];
        bolt_angles[1] = -bolt_matrix.matrix[1][1];
        bolt_angles[2] = -bolt_matrix.matrix[2][1];

        let te = G_TempEntity(&*bolt_point, EV_SABER_HIT);
        (*te).s.otherEntityNum = (*self_).s.number;
        (*te).s.otherEntityNum2 = ENTITYNUM_NONE;
        (*te).s.weapon = 0; // saberNum
        (*te).s.legsAnim = 0; // bladeNum

        VectorCopy(&*bolt_point, &mut (*te).s.origin);
        VectorCopy(&bolt_angles, &mut (*te).s.angles);

        if (*te).s.angles[0] == 0.0 && (*te).s.angles[1] == 0.0 && (*te).s.angles[2] == 0.0 {
            // don't let it play with no direction
            (*te).s.angles[1] = 1.0;
        }

        (*te).s.eventParm = 16; // lots of sparks
    }
}

/// `int gPainHitLoc` (g_combat.c:4367) — module global caching the last pain hit
/// location (an `HL_*` value). Written by `G_Damage`'s pain path; read by
/// [`G_GetHitQuad`] as its (overwritten-on-every-path, so effectively dead) default.
pub static mut gPainHitLoc: c_int = -1;

/// `char *hitLocName[HL_MAX]` (g_combat.c:3042) — human-readable hit-location names,
/// indexed by `HL_*`. Only used for the Austrian-rules duel dismemberment log line.
/// The `"front right shouler"` entry preserves the original's typo verbatim.
static hitLocName: [&str; HL_MAX as usize] = [
    "none",                // HL_NONE = 0
    "right foot",          // HL_FOOT_RT
    "left foot",           // HL_FOOT_LT
    "right leg",           // HL_LEG_RT
    "left leg",            // HL_LEG_LT
    "waist",               // HL_WAIST
    "back right shoulder", // HL_BACK_RT
    "back left shoulder",  // HL_BACK_LT
    "back",                // HL_BACK
    "front right shouler", // HL_CHEST_RT
    "front left shoulder", // HL_CHEST_LT
    "chest",               // HL_CHEST
    "right arm",           // HL_ARM_RT
    "left arm",            // HL_ARM_LT
    "right hand",          // HL_HAND_RT
    "left hand",           // HL_HAND_LT
    "head",                // HL_HEAD
    "generic1",            // HL_GENERIC1
    "generic2",            // HL_GENERIC2
    "generic3",            // HL_GENERIC3
    "generic4",            // HL_GENERIC4
    "generic5",            // HL_GENERIC5
    "generic6",            // HL_GENERIC6
];

/// `int gGAvoidDismember` (g_combat.c:3628) — dismemberment override flag read by
/// [`G_CheckForDismemberment`]: `1` suppresses it entirely, `2` forces it (a guaranteed
/// right-hand sever, bypassing the randomness/damage gates), `0` is normal probability.
pub static mut gGAvoidDismember: c_int = 0;

/// `void LimbTouch( gentity_t *self, gentity_t *other, trace_t *trace )` (g_combat.c:3262)
/// — the touch handler wired onto a severed limb entity. Empty in the original (limbs
/// don't react to contact); kept so the `limb->touch` slot in [`G_Dismember`] matches.
///
/// # Safety
/// Called by the engine through the entity `touch` fn-pointer; matches that ABI.
pub unsafe extern "C" fn LimbTouch(
    _self_: *mut gentity_t,
    _other: *mut gentity_t,
    _trace: *mut trace_t,
) {
}

/// `void LimbThink( gentity_t *ent )` (g_combat.c:3266) — the per-frame think handler
/// for a severed limb: pick a per-part mass/bounce, free the limb once its lifespan
/// (`speed`) elapses, otherwise step it through the extended-physics integrator
/// ([`G_RunExPhys`]) at a fixed 50 ms cadence (so it behaves the same regardless of
/// `sv_fps`).
///
/// No oracle — drives `G_RunExPhys`/`G_FreeEntity` and reads the global `level`.
///
/// # Safety
/// Called by the engine through the entity `think` fn-pointer; `ent` must be a valid
/// limb entity.
pub unsafe extern "C" fn LimbThink(ent: *mut gentity_t) {
    let gravity: f32 = 3.0;
    let mut mass: f32 = 0.09;
    let mut bounce: f32 = 1.3;

    match (*ent).s.modelGhoul2 {
        G2_MODELPART_HEAD => {
            mass = 0.08;
            bounce = 1.4;
        }
        G2_MODELPART_WAIST => {
            mass = 0.1;
            bounce = 1.2;
        }
        // LARM / RARM / RHAND / LLEG / RLEG and default: keep the defaults.
        _ => {}
    }

    // `speed` is a float; C promotes level.time (int) to float for the compare.
    if (*ent).speed < (*addr_of!(level)).time as f32 {
        (*ent).think = Some(G_FreeEntity);
        (*ent).nextthink = (*addr_of!(level)).time;
        return;
    }

    if (*ent).genericValue5 <= (*addr_of!(level)).time {
        // this will be every frame by standard, but we want to compensate in case sv_fps is not 20.
        G_RunExPhys(ent, gravity, mass, bounce, QTRUE, null_mut(), 0);
        (*ent).genericValue5 = (*addr_of!(level)).time + 50;
    }

    (*ent).nextthink = (*addr_of!(level)).time;
}

/// `void G_Dismember( gentity_t *ent, gentity_t *enemy, vec3_t point, int limbType,
/// float limbRollBase, float limbPitchBase, int deathAnim, qboolean postDeath )`
/// (g_combat.c:3311) — spawn the free-flying `playerlimb` entity for a severed body
/// part: resolve the limb/stub surface names, bail if the limb is already off, then
/// build a trigger-contents ghoul2 limb at `point`, seed its extended-physics velocity
/// from the victim's motion (and, for a saber kill, the attacker's recent blade sweep),
/// drop the surfs on NPC ghoul2 instances, and link it.
///
/// `limbRollBase`/`limbPitchBase`/`deathAnim`/`postDeath` are vestigial — the C body
/// never reads them — so they carry leading underscores here.
///
/// No oracle — spawns an entity, drives the ghoul2/link traps, and reads the global
/// `level`, like its entity-spawning siblings.
///
/// # Safety
/// `ent` must be a valid entity (its `ghoul2`/`client`/`r`/`s` are read); `enemy` may be
/// NULL (checked); `point` a valid `*const vec3_t`.
#[allow(clippy::too_many_arguments)]
pub unsafe fn G_Dismember(
    ent: *mut gentity_t,
    enemy: *mut gentity_t,
    point: *const vec3_t,
    limb_type: c_int,
    _limb_roll_base: f32,
    _limb_pitch_base: f32,
    _death_anim: c_int,
    _post_death: qboolean,
) {
    let mut new_point: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut vel: vec3_t = [0.0; 3];
    let mut limb_name: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut stub_name: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut stub_cap_name: [c_char; MAX_QPATH] = [0; MAX_QPATH];

    if limb_type == G2_MODELPART_HEAD {
        Q_strncpyz(limb_name.as_mut_ptr(), c"head".as_ptr(), MAX_QPATH as c_int);
        Q_strncpyz(
            stub_cap_name.as_mut_ptr(),
            c"torso_cap_head".as_ptr(),
            MAX_QPATH as c_int,
        );
    } else if limb_type == G2_MODELPART_WAIST {
        Q_strncpyz(limb_name.as_mut_ptr(), c"torso".as_ptr(), MAX_QPATH as c_int);
        Q_strncpyz(
            stub_cap_name.as_mut_ptr(),
            c"hips_cap_torso".as_ptr(),
            MAX_QPATH as c_int,
        );
    } else if limb_type == G2_MODELPART_LARM {
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"l_arm".as_ptr(),
            limb_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"torso".as_ptr(),
            stub_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        Com_sprintf(
            stub_cap_name.as_mut_ptr(),
            MAX_QPATH as c_int,
            format_args!("{}_cap_l_arm", Sz(stub_name.as_ptr())),
        );
    } else if limb_type == G2_MODELPART_RARM {
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"r_arm".as_ptr(),
            limb_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"torso".as_ptr(),
            stub_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        Com_sprintf(
            stub_cap_name.as_mut_ptr(),
            MAX_QPATH as c_int,
            format_args!("{}_cap_r_arm", Sz(stub_name.as_ptr())),
        );
    } else if limb_type == G2_MODELPART_RHAND {
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"r_hand".as_ptr(),
            limb_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"r_arm".as_ptr(),
            stub_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        Com_sprintf(
            stub_cap_name.as_mut_ptr(),
            MAX_QPATH as c_int,
            format_args!("{}_cap_r_hand", Sz(stub_name.as_ptr())),
        );
    } else if limb_type == G2_MODELPART_LLEG {
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"l_leg".as_ptr(),
            limb_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"hips".as_ptr(),
            stub_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        Com_sprintf(
            stub_cap_name.as_mut_ptr(),
            MAX_QPATH as c_int,
            format_args!("{}_cap_l_leg", Sz(stub_name.as_ptr())),
        );
    } else if limb_type == G2_MODELPART_RLEG {
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"r_leg".as_ptr(),
            limb_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"hips".as_ptr(),
            stub_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        Com_sprintf(
            stub_cap_name.as_mut_ptr(),
            MAX_QPATH as c_int,
            format_args!("{}_cap_r_leg", Sz(stub_name.as_ptr())),
        );
    } else {
        // umm... just default to the right leg, I guess (same as on client)
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"r_leg".as_ptr(),
            limb_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        BG_GetRootSurfNameWithVariant(
            (*ent).ghoul2,
            c"hips".as_ptr(),
            stub_name.as_mut_ptr(),
            MAX_QPATH as c_int,
        );
        Com_sprintf(
            stub_cap_name.as_mut_ptr(),
            MAX_QPATH as c_int,
            format_args!("{}_cap_r_leg", Sz(stub_name.as_ptr())),
        );
    }

    if !(*ent).ghoul2.is_null()
        && trap::G2API_GetSurfaceRenderStatus((*ent).ghoul2, 0, limb_name.as_ptr()) != 0
    {
        // is it already off? If so there's no reason to be doing it again, so get out of here.
        return;
    }

    VectorCopy(&*point, &mut new_point);
    let limb = G_Spawn();
    (*limb).classname = c"playerlimb".as_ptr() as *mut c_char;

    G_SetOrigin(limb, &new_point);
    VectorCopy(&new_point, &mut (*limb).s.pos.trBase);
    (*limb).think = Some(LimbThink);
    (*limb).touch = Some(LimbTouch);
    (*limb).speed = ((*addr_of!(level)).time + Q_irand(8000, 16000)) as f32;
    (*limb).nextthink = (*addr_of!(level)).time + FRAMETIME;

    (*limb).r.svFlags = SVF_USE_CURRENT_ORIGIN;
    (*limb).clipmask = MASK_SOLID;
    (*limb).r.contents = CONTENTS_TRIGGER;
    (*limb).physicsObject = QTRUE;
    VectorSet(&mut (*limb).r.mins, -6.0, -6.0, -3.0);
    VectorSet(&mut (*limb).r.maxs, 6.0, 6.0, 6.0);

    (*limb).s.g2radius = 200;

    (*limb).s.eType = ET_GENERAL;
    (*limb).s.weapon = G2_MODEL_PART;
    (*limb).s.modelGhoul2 = limb_type;
    (*limb).s.modelindex = (*ent).s.number;
    if (*ent).client.is_null() {
        (*limb).s.modelindex = -1;
        (*limb).s.otherEntityNum2 = (*ent).s.number;
    }

    VectorClear(&mut (*limb).s.apos.trDelta);

    if !(*ent).client.is_null() {
        VectorCopy(
            &(*(*ent).client).ps.viewangles,
            &mut (*limb).r.currentAngles,
        );
        VectorCopy(&(*(*ent).client).ps.viewangles, &mut (*limb).s.apos.trBase);
    } else {
        VectorCopy(&(*ent).r.currentAngles, &mut (*limb).r.currentAngles);
        VectorCopy(&(*ent).r.currentAngles, &mut (*limb).s.apos.trBase);
    }

    // Set up the ExPhys values for the entity.
    (*limb).epGravFactor = 0.0;
    VectorClear(&mut (*limb).epVelocity);
    VectorSubtract(&*point, &(*ent).r.currentOrigin, &mut dir);
    VectorNormalize(&mut dir);
    if !(*ent).client.is_null() {
        VectorCopy(&(*(*ent).client).ps.velocity, &mut vel);
    } else {
        VectorCopy(&(*ent).s.pos.trDelta, &mut vel);
    }
    VectorMA(&vel, 80.0, &dir, &mut (*limb).epVelocity);

    // add some vertical velocity
    if limb_type == G2_MODELPART_HEAD || limb_type == G2_MODELPART_WAIST {
        (*limb).epVelocity[2] += 10.0;
    }

    if !enemy.is_null()
        && !(*enemy).client.is_null()
        && !ent.is_null()
        && ent != enemy
        && (*ent).s.number != (*enemy).s.number
        && (*(*enemy).client).ps.weapon == WP_SABER
        && (*(*enemy).client).olderIsValid != QFALSE
        && ((*addr_of!(level)).time - (*(*enemy).client).lastSaberStorageTime) < 200
    {
        // The enemy has valid saber positions between this and last frame. Use them to factor in direction of the limb.
        let mut dif: vec3_t = [0.0; 3];
        let dist_scale: f32 = 1.2;

        // scale down the initial velocity first, which is based on the speed of the limb owner.
        // ExPhys object velocity operates on a slightly different scale than Q3-based physics velocity.
        let ev = (*limb).epVelocity;
        VectorScale(&ev, 0.4, &mut (*limb).epVelocity);

        VectorSubtract(
            &(*(*enemy).client).lastSaberBase_Always,
            &(*(*enemy).client).olderSaberBase,
            &mut dif,
        );
        let total_distance = VectorNormalize(&mut dif);

        let dif_in = dif;
        VectorScale(&dif_in, total_distance * dist_scale, &mut dif);
        let ev = (*limb).epVelocity;
        VectorAdd(&ev, &dif, &mut (*limb).epVelocity);

        if !(*ent).client.is_null()
            && ((*(*ent).client).ps.torsoTimer > 0
                || BG_InDeathAnim((*(*ent).client).ps.torsoAnim) == QFALSE)
        {
            // if he's done with his death anim we don't actually want the limbs going far
            let mut pre_vel: vec3_t = [0.0; 3];

            VectorCopy(&(*limb).epVelocity, &mut pre_vel);
            pre_vel[2] = 0.0;
            let total_distance = VectorNormalize(&mut pre_vel);

            if total_distance < 40.0 {
                let m_amt: f32 = 40.0; // 60.0/totalDistance;

                (*limb).epVelocity[0] = pre_vel[0] * m_amt;
                (*limb).epVelocity[1] = pre_vel[1] * m_amt;
            }
        } else if !(*ent).client.is_null() {
            let ev = (*limb).epVelocity;
            VectorScale(&ev, 0.3, &mut (*limb).epVelocity);
        }
    }

    if (*ent).s.eType == ET_NPC && !(*ent).ghoul2.is_null() {
        // if it's an npc remove these surfs on the server too. For players we don't even care
        // cause there's no further dismemberment after death. (The C `&& limbName && stubCapName`
        // is always true — they are fixed-size buffers.)
        trap::G2API_SetSurfaceOnOff((*ent).ghoul2, limb_name.as_ptr(), 0x00000100);
        trap::G2API_SetSurfaceOnOff((*ent).ghoul2, stub_cap_name.as_ptr(), 0);
    }

    (*limb).s.customRGBA[0] = (*ent).s.customRGBA[0];
    (*limb).s.customRGBA[1] = (*ent).s.customRGBA[1];
    (*limb).s.customRGBA[2] = (*ent).s.customRGBA[2];
    (*limb).s.customRGBA[3] = (*ent).s.customRGBA[3];

    trap::LinkEntity(limb);
}

/// `int G_GetHitQuad( gentity_t *self, vec3_t hitloc )` (g_combat.c:3546) — coarse
/// "which quadrant did the hit land in" classifier used as the dismemberment fallback
/// `void DismembermentTest( gentity_t *self )` (g_combat.c:3497) — debug helper that
/// dismembers every limb in turn (`G2_MODELPART_HEAD`..`G2_MODELPART_RLEG`), driving
/// each through [`G_GetDismemberBolt`] + [`G_Dismember`]. No oracle: pure dispatch over
/// the two ported impure helpers.
///
/// # Safety
/// `self_` must be a valid entity suitable for [`G_GetDismemberBolt`]/[`G_Dismember`].
pub unsafe fn DismembermentTest(self_: *mut gentity_t) {
    let mut sect = G2_MODELPART_HEAD;
    let mut bolt_point: vec3_t = [0.0; 3];

    while sect <= G2_MODELPART_RLEG {
        G_GetDismemberBolt(self_, &mut bolt_point, sect);
        G_Dismember(self_, self_, &bolt_point, sect, 90.0, 0.0, BOTH_DEATH1, QFALSE);
        sect += 1;
    }
}

/// `void DismembermentByNum( gentity_t *self, int num )` (g_combat.c:3510) — debug
/// helper that maps a 0..6 index to a `G2_MODELPART_*` and dismembers that one limb.
/// No oracle: pure dispatch over the two ported impure helpers. Faithful to the C
/// `switch` (out-of-range `num` falls through to the default `G2_MODELPART_HEAD` seed).
///
/// # Safety
/// `self_` must be a valid entity suitable for [`G_GetDismemberBolt`]/[`G_Dismember`].
pub unsafe fn DismembermentByNum(self_: *mut gentity_t, num: c_int) {
    let mut sect = G2_MODELPART_HEAD;
    let mut bolt_point: vec3_t = [0.0; 3];

    match num {
        0 => sect = G2_MODELPART_HEAD,
        1 => sect = G2_MODELPART_WAIST,
        2 => sect = G2_MODELPART_LARM,
        3 => sect = G2_MODELPART_RARM,
        4 => sect = G2_MODELPART_RHAND,
        5 => sect = G2_MODELPART_LLEG,
        6 => sect = G2_MODELPART_RLEG,
        _ => {}
    }

    G_GetDismemberBolt(self_, &mut bolt_point, sect);
    G_Dismember(self_, self_, &bolt_point, sect, 90.0, 0.0, BOTH_DEATH1, QFALSE);
}

/// when the precise per-surface hit location is unknown. Projects the eye→impact
/// direction onto the view's right axis and buckets by the vertical delta into a
/// `G2_MODELPART_*` part (arms/head up high, legs down low).
///
/// No oracle — reads `client->ps`/`s` entity fields like its `G_GetHitLocFromSurfName`
/// sibling. The decimal-threshold comparisons promote `rightdot` (`f32`) through `f64`
/// to match C's `float`→`double` compare, exactly as the oracle-proven
/// [`G_GetHitLocation`] does; the `±0`/`±20` integer thresholds stay `f32` compares.
/// `self->client` being non-NULL selects the eye/yaw source. The `gPainHitLoc` default
/// is overwritten on every code path (dead in the original too).
///
/// # Safety
/// `self_` must be a valid entity (its `client`, when non-NULL, plus `s` are read);
/// `hitloc` a valid `*const vec3_t`.
#[allow(unused_assignments)] // faithful: C dead-inits hitLoc from gPainHitLoc
pub unsafe fn G_GetHitQuad(self_: *mut gentity_t, hitloc: *const vec3_t) -> c_int {
    let mut diff: vec3_t = [0.0; 3];
    let mut fwdangles: vec3_t = [0.0, 0.0, 0.0];
    let mut right: vec3_t = [0.0; 3];
    let mut cl_eye: vec3_t = [0.0; 3];
    let mut hit_loc: c_int = *addr_of!(gPainHitLoc);

    if !(*self_).client.is_null() {
        VectorCopy(&(*(*self_).client).ps.origin, &mut cl_eye);
        cl_eye[2] += (*(*self_).client).ps.viewheight as f32;
    } else {
        VectorCopy(&(*self_).s.pos.trBase, &mut cl_eye);
        cl_eye[2] += 16.0;
    }

    VectorSubtract(&*hitloc, &cl_eye, &mut diff);
    diff[2] = 0.0;
    VectorNormalize(&mut diff);

    if !(*self_).client.is_null() {
        fwdangles[1] = (*(*self_).client).ps.viewangles[1];
    } else {
        fwdangles[1] = (*self_).s.apos.trBase[1];
    }
    // Ultimately we might care if the shot was ahead or behind, but for now, just quadrant is fine.
    AngleVectors(&fwdangles, None, Some(&mut right), None);

    let rightdot = DotProduct(&right, &diff);
    let zdiff = (*hitloc)[2] - cl_eye[2];

    if zdiff > 0.0 {
        if (rightdot as f64) > 0.3 {
            hit_loc = G2_MODELPART_RARM;
        } else if (rightdot as f64) < -0.3 {
            hit_loc = G2_MODELPART_LARM;
        } else {
            hit_loc = G2_MODELPART_HEAD;
        }
    } else if zdiff > -20.0 {
        if (rightdot as f64) > 0.1 {
            hit_loc = G2_MODELPART_RARM;
        } else if (rightdot as f64) < -0.1 {
            hit_loc = G2_MODELPART_LARM;
        } else {
            hit_loc = G2_MODELPART_HEAD;
        }
    } else if rightdot >= 0.0 {
        hit_loc = G2_MODELPART_RLEG;
    } else {
        hit_loc = G2_MODELPART_LLEG;
    }

    hit_loc
}

/// `void G_CheckForDismemberment(gentity_t *ent, gentity_t *enemy, vec3_t point,
/// int damage, int deathAnim, qboolean postDeath)` (g_combat.c:4099) — the last direct
/// `G_Damage` dependency: decide whether a hit dismembers `ent` and, if so, which limb,
/// then fire [`G_Dismember`]. Gated on humanoid-ness (only the protocol droid among
/// non-humanoids dismembers), `g_dismember`, the `gGAvoidDismember` override, and a
/// random/damage roll. The hit location comes from the last ghoul2-collision surface
/// (when active) else the geometric [`G_GetHitLocation`], mapped to a `G2_MODELPART_*`
/// with [`G_GetHitQuad`] as the fallback.
///
/// No oracle — drives the dismemberment entity-spawn chain and reads global cvars/
/// `level`, like the rest of the dismember family.
///
/// # Safety
/// `ent` must be a valid entity; when `localAnimIndex > 1` its `NPC`/`client` are read
/// as in the C original (no extra NULL guards). `enemy` may be NULL; `point` valid.
pub unsafe fn G_CheckForDismemberment(
    ent: *mut gentity_t,
    enemy: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    death_anim: c_int,
    post_death: qboolean,
) {
    let mut hit_loc: c_int = -1;
    let mut bolt_point: vec3_t = [0.0; 3];
    let dismember: c_int = (*addr_of!(g_dismember)).integer;

    if (*ent).localAnimIndex > 1 {
        if (*ent).NPC.is_null() {
            return;
        }

        if (*(*ent).client).NPC_class != CLASS_PROTOCOL {
            // this is the only non-humanoid allowed to do dismemberment.
            return;
        }
    }

    if dismember == 0 {
        return;
    }

    if *addr_of!(gGAvoidDismember) == 1 {
        return;
    }

    if *addr_of!(gGAvoidDismember) != 2 {
        // this means do the dismemberment regardless of randomness and damage
        if Q_irand(0, 100) > dismember {
            return;
        }

        if damage < 5 {
            return;
        }
    }

    if *addr_of!(gGAvoidDismember) == 2 {
        hit_loc = HL_HAND_RT;
    } else {
        if (*addr_of!(d_saberGhoul2Collision)).integer != 0
            && !(*ent).client.is_null()
            && (*(*ent).client).g2LastSurfaceTime == (*addr_of!(level)).time
        {
            let mut hit_surface: [c_char; MAX_QPATH] = [0; MAX_QPATH];

            trap::G2API_GetSurfaceName(
                (*ent).ghoul2,
                (*(*ent).client).g2LastSurfaceHit,
                0,
                hit_surface.as_mut_ptr(),
            );

            if hit_surface[0] != 0 {
                G_GetHitLocFromSurfName(
                    ent,
                    hit_surface.as_ptr(),
                    &mut hit_loc,
                    point,
                    &vec3_origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    MOD_UNKNOWN,
                );
            }
        }

        if hit_loc == -1 {
            hit_loc = G_GetHitLocation(ent, point);
        }
    }

    // C inits hitLocUse = -1 then assigns it in every switch case (including default);
    // expressed here as the match value so the dead -1 init drops away.
    let hit_loc_use: c_int = match hit_loc {
        HL_FOOT_RT | HL_LEG_RT => G2_MODELPART_RLEG,
        HL_FOOT_LT | HL_LEG_LT => G2_MODELPART_LLEG,
        HL_WAIST => G2_MODELPART_WAIST,
        HL_ARM_RT => G2_MODELPART_RARM,
        HL_HAND_RT => G2_MODELPART_RHAND,
        HL_ARM_LT | HL_HAND_LT => G2_MODELPART_LARM,
        HL_HEAD => G2_MODELPART_HEAD,
        _ => G_GetHitQuad(ent, point),
    };

    if hit_loc_use == -1 {
        return;
    }

    if !(*ent).client.is_null() {
        G_GetDismemberBolt(ent, &mut bolt_point, hit_loc_use);
        if (*addr_of!(g_austrian)).integer != 0
            && ((*addr_of!(g_gametype)).integer == GT_DUEL
                || (*addr_of!(g_gametype)).integer == GT_POWERDUEL)
        {
            G_LogPrintf(&format!(
                "Duel Dismemberment: {} dismembered at {}\n",
                Sz((*(*ent).client).pers.netname.as_ptr()),
                hitLocName[hit_loc as usize]
            ));
        }
    } else {
        G_GetDismemberLoc(ent, &mut bolt_point, hit_loc_use);
    }
    G_Dismember(
        ent,
        enemy,
        &bolt_point,
        hit_loc_use,
        90.0,
        0.0,
        death_anim,
        post_death,
    );
}

// The vehicle-damage / bot / predef-sound callees reached by `G_Damage`
// (`G_VehUpdateShields`, `G_ShipSurfaceForSurfName`, `G_FlyVehicleDestroySurface`,
// `G_VehicleSetDamageLocFlags`, `BotDamageNotification`, `G_PreDefSound`, `G_LetGoOfWall`)
// are all ported in their home files and imported above.



/// `int gPainMOD` (g_combat.c:4366) — module global caching the `MOD_*` of the last
/// pain event; written by [`G_Damage`]'s pain path right before dispatching
/// `targ->pain`, read by the (not-yet-ported) pain handlers. Joins [`gPainHitLoc`] above.
pub static mut gPainMOD: c_int = 0;

/// `vec3_t gPainPoint` (g_combat.c:4368) — module global caching the world point of
/// the last pain event; written alongside [`gPainMOD`] before the `targ->pain`
/// dispatch and read by the (not-yet-ported) pain handlers.
pub static mut gPainPoint: vec3_t = [0.0; 3];

/// `qboolean gSiegeRoundBegun` (g_saga.c:36) — Siege round-active flag, owned by the
/// not-yet-ported Siege subsystem (g_saga.c). [`G_Damage`] only reads it (the `extern` at
/// g_combat.c:4364); defined here as its temporary canonical home until `g_saga.rs`
/// lands and takes ownership. Never set true yet (no Siege code runs), so the
/// `GT_SIEGE` early-out it guards is currently inert.
pub static mut gSiegeRoundBegun: qboolean = QFALSE;

/// `void G_ApplyVehicleOtherKiller( gentity_t *targ, gentity_t *inflictor, gentity_t *attacker, int mod, qboolean vehicleDying )` (g_combat.c:4463) — records the last attacker as the victim's "other killer" (so a later suicide/eject still credits the kill), and when the vehicle itself is dying propagates that credit down to its pilot and passengers.
///
/// No oracle — mutates live client/playerState game state and reads the global `level`/`g_entities`.
///
/// # Safety
/// `targ` may be NULL (checked); `inflictor`/`attacker` may be NULL (checked).
pub unsafe fn G_ApplyVehicleOtherKiller(
    targ: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    mod_: c_int,
    vehicleDying: qboolean,
) {
    if !targ.is_null() && !(*targ).client.is_null() && !attacker.is_null() {
        if (*(*targ).client).ps.otherKillerDebounceTime > (*addr_of!(level)).time {
            //wait a minute, I already have a last damager
            if (*targ).health < 0
                || (!(*targ).m_pVehicle.is_null() && (*(*targ).m_pVehicle).m_iRemovedSurfaces != 0)
            {
                //already dying?  don't let credit transfer to anyone else
                return;
            }
            //otherwise, still alive, so, fine, use this damager...
        }

        (*(*targ).client).ps.otherKiller = (*attacker).s.number;
        (*(*targ).client).ps.otherKillerTime = (*addr_of!(level)).time + 25000;
        (*(*targ).client).ps.otherKillerDebounceTime = (*addr_of!(level)).time + 25000;
        (*(*targ).client).otherKillerMOD = mod_;
        if !inflictor.is_null()
            && Q_stricmp(c"vehicle_proj".as_ptr(), (*inflictor).classname as *const c_char) == 0
        {
            (*(*targ).client).otherKillerVehWeapon = (*inflictor).s.otherEntityNum2 + 1;
            (*(*targ).client).otherKillerWeaponType = (*inflictor).s.weapon;
        } else {
            (*(*targ).client).otherKillerVehWeapon = 0;
            (*(*targ).client).otherKillerWeaponType = WP_NONE;
        }
        if vehicleDying != QFALSE {
            //propogate otherkiller down to pilot and passengers so that proper credit is given if they suicide or eject...
            if !(*targ).m_pVehicle.is_null() {
                if !(*(*targ).m_pVehicle).m_pPilot.is_null() {
                    let pilot = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*(*targ).m_pVehicle).m_pPilot).s.number as usize);
                    if !(*pilot).client.is_null() {
                        (*(*pilot).client).ps.otherKiller = (*(*targ).client).ps.otherKiller;
                        (*(*pilot).client).ps.otherKillerTime = (*(*targ).client).ps.otherKillerTime;
                        (*(*pilot).client).ps.otherKillerDebounceTime =
                            (*(*targ).client).ps.otherKillerDebounceTime;
                        (*(*pilot).client).otherKillerMOD = (*(*targ).client).otherKillerMOD;
                        (*(*pilot).client).otherKillerVehWeapon =
                            (*(*targ).client).otherKillerVehWeapon;
                        (*(*pilot).client).otherKillerWeaponType =
                            (*(*targ).client).otherKillerWeaponType;
                    }
                }
                let mut passNum: c_int = 0;
                while passNum < (*(*targ).m_pVehicle).m_iNumPassengers {
                    let pass = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(
                        (*(*(*targ).m_pVehicle).m_ppPassengers[passNum as usize]).s.number as usize,
                    );
                    if !(*pass).client.is_null() {
                        (*(*pass).client).ps.otherKiller = (*(*targ).client).ps.otherKiller;
                        (*(*pass).client).ps.otherKillerTime = (*(*targ).client).ps.otherKillerTime;
                        (*(*pass).client).ps.otherKillerDebounceTime =
                            (*(*targ).client).ps.otherKillerDebounceTime;
                        (*(*pass).client).otherKillerMOD = (*(*targ).client).otherKillerMOD;
                        (*(*pass).client).otherKillerVehWeapon =
                            (*(*targ).client).otherKillerVehWeapon;
                        (*(*pass).client).otherKillerWeaponType =
                            (*(*targ).client).otherKillerWeaponType;
                    }
                    passNum += 1;
                }
            }
        }
    }
}

/// `qboolean G_CheckVehicleNPCTeamDamage( gentity_t *ent )` (g_combat.c:4528) — covers
/// both vehicles and NPCs riding vehicles (droids): true only for a non-client `ET_NPC`
/// that is not itself a `CLASS_VEHICLE` but is sitting in one (`s.m_iVehicleNum` set).
///
/// No oracle — reads only entity game state.
///
/// # Safety
/// `ent` may be NULL (checked); otherwise must be a valid entity.
pub unsafe fn G_CheckVehicleNPCTeamDamage(ent: *mut gentity_t) -> qboolean {
    //NOTE: this covers both the vehicle and NPCs riding vehicles (droids)
    if ent.is_null() || (*ent).s.number < MAX_CLIENTS as c_int || (*ent).s.eType != ET_NPC {
        //not valid or a real client or not an NPC
        return QFALSE;
    }

    if (*ent).s.NPC_class != CLASS_VEHICLE {
        //regualar NPC
        if (*ent).s.m_iVehicleNum != 0 {
            //an NPC in a vehicle, check for team damage
            return QTRUE;
        }
    }
    QFALSE
}

/// `void G_Damage( gentity_t *targ, gentity_t *inflictor, gentity_t *attacker,
/// vec3_t dir, vec3_t point, int damage, int dflags, int mod )` (g_combat.c:4370) —
/// the damage core. Applies armor/shield absorption, knockback, friendly-fire and
/// protection gates, force-protect/rage scaling and location damage, then drives the
/// target's `die`/`pain` function-pointer slots. Ported as a faithful monolith; its
/// not-yet-ported-subsystem callees route to the guarded stubs above.
///
/// No oracle — drives entity `die`/`pain` fn-pointers, the `trap_G2API_*` engine
/// surface queries, and the full `level`/`g_entities`/cvar globals, none of which the
/// off-engine harness can reproduce. Verified by faithful 1:1 translation + build.
///
/// # Safety
/// `targ` must be a valid entity (deref'd unconditionally from the `takedamage`
/// gate on, as in C); `inflictor`/`attacker` may be NULL (defaulted to the world
/// entity); `dir`/`point` may be NULL (each checked before use).
pub unsafe fn G_Damage(
    targ: *mut gentity_t,
    mut inflictor: *mut gentity_t,
    mut attacker: *mut gentity_t,
    dir: *mut vec3_t,
    point: *mut vec3_t,
    mut damage: c_int,
    mut dflags: c_int,
    mod_: c_int,
) {
    let client: *mut gclient_t;
    let mut take: c_int;
    // C declares `save` and sets it to 0 here, but never reads it — dead, omitted.
    let asave: c_int;
    let mut knockback: c_int;
    // C inits `subamt` to 0, but every read is preceded by an assignment — init is dead.
    let mut subamt: c_int;
    let mut famt: f32 = 0.0;
    let mut hamt: f32 = 0.0;
    let mut shield_absorbed: f32 = 0.0;

    if !targ.is_null() && (*targ).damageRedirect != 0 {
        G_Damage(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*targ).damageRedirectTo as usize),
            inflictor,
            attacker,
            dir,
            point,
            damage,
            dflags,
            mod_,
        );
        return;
    }

    if mod_ == MOD_DEMP2 && !targ.is_null() && (*targ).inuse != QFALSE && !(*targ).client.is_null() {
        if (*(*targ).client).ps.electrifyTime < (*addr_of!(level)).time {
            //electrocution effect
            if (*targ).s.eType == ET_NPC
                && (*targ).s.NPC_class == CLASS_VEHICLE
                && !(*targ).m_pVehicle.is_null()
                && ((*(*(*targ).m_pVehicle).m_pVehicleInfo).r#type == VH_SPEEDER
                    || (*(*(*targ).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER)
            {
                //do some extra stuff to speeders/walkers
                (*(*targ).client).ps.electrifyTime = (*addr_of!(level)).time + Q_irand(3000, 4000);
            } else if (*targ).s.NPC_class != CLASS_VEHICLE
                || (!(*targ).m_pVehicle.is_null()
                    && (*(*(*targ).m_pVehicle).m_pVehicleInfo).r#type != VH_FIGHTER)
            {
                //don't do this to fighters
                (*(*targ).client).ps.electrifyTime = (*addr_of!(level)).time + Q_irand(300, 800);
            }
        }
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE && *addr_of!(gSiegeRoundBegun) == QFALSE {
        //nothing can be damaged til the round starts.
        return;
    }

    if (*targ).takedamage == QFALSE {
        return;
    }

    if (*targ).flags & FL_SHIELDED != 0 && mod_ != MOD_SABER && (*targ).client.is_null() {
        //magnetically protected, this thing can only be damaged by lightsabers
        return;
    }

    if (*targ).flags & FL_DMG_BY_SABER_ONLY != 0 && mod_ != MOD_SABER {
        //saber-only damage
        return;
    }

    if !(*targ).client.is_null() {
        //don't take damage when in a walker, or fighter
        //unless the walker/fighter is dead!!! -rww
        if (*(*targ).client).ps.clientNum < MAX_CLIENTS as c_int && (*(*targ).client).ps.m_iVehicleNum != 0 {
            let veh = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*targ).client).ps.m_iVehicleNum as usize);
            if !(*veh).m_pVehicle.is_null() && (*veh).health > 0 {
                if (*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER
                    || (*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER
                {
                    if dflags & DAMAGE_NO_PROTECTION == 0 {
                        return;
                    }
                }
            }
        }
    }

    if (*targ).flags & FL_DMG_BY_HEAVY_WEAP_ONLY != 0 {
        //only take damage from explosives and such
        if mod_ != MOD_REPEATER_ALT
            && mod_ != MOD_ROCKET
            && mod_ != MOD_FLECHETTE_ALT_SPLASH
            && mod_ != MOD_ROCKET_HOMING
            && mod_ != MOD_THERMAL
            && mod_ != MOD_THERMAL_SPLASH
            && mod_ != MOD_TRIP_MINE_SPLASH
            && mod_ != MOD_TIMED_MINE_SPLASH
            && mod_ != MOD_DET_PACK_SPLASH
            && mod_ != MOD_VEHICLE
            && mod_ != MOD_CONC
            && mod_ != MOD_CONC_ALT
            && mod_ != MOD_SABER
            && mod_ != MOD_TURBLAST
            && mod_ != MOD_SUICIDE
            && mod_ != MOD_FALLING
            && mod_ != MOD_CRUSH
            && mod_ != MOD_TELEFRAG
            && mod_ != MOD_TRIGGER_HURT
        {
            if mod_ != MOD_MELEE || G_HeavyMelee(attacker) == QFALSE {
                //let classes with heavy melee ability damage heavy wpn dmg doors with fists
                return;
            }
        }
    }

    if (*targ).flags & FL_BBRUSH != 0 {
        if mod_ == MOD_DEMP2
            || mod_ == MOD_DEMP2_ALT
            || mod_ == MOD_BRYAR_PISTOL
            || mod_ == MOD_BRYAR_PISTOL_ALT
            || mod_ == MOD_MELEE
        {
            //these don't damage bbrushes.. ever
            if mod_ != MOD_MELEE || G_HeavyMelee(attacker) == QFALSE {
                //let classes with heavy melee ability damage breakable brushes with fists
                return;
            }
        }
    }

    if !targ.is_null() && !(*targ).client.is_null() && (*(*targ).client).ps.duelInProgress != 0 {
        if !attacker.is_null()
            && !(*attacker).client.is_null()
            && (*attacker).s.number != (*(*targ).client).ps.duelIndex
        {
            return;
        } else if !attacker.is_null() && !(*attacker).client.is_null() && mod_ != MOD_SABER {
            return;
        }
    }
    if !attacker.is_null()
        && !(*attacker).client.is_null()
        && (*(*attacker).client).ps.duelInProgress != 0
    {
        if !targ.is_null()
            && !(*targ).client.is_null()
            && (*targ).s.number != (*(*attacker).client).ps.duelIndex
        {
            return;
        } else if !targ.is_null() && !(*targ).client.is_null() && mod_ != MOD_SABER {
            return;
        }
    }

    if !targ.is_null()
        && !(*targ).client.is_null()
        && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
    {
        damage = (damage as f64 * 0.5) as c_int;
    }

    // the intermission has allready been qualified for, so don't
    // allow any extra scoring
    if (*addr_of!(level)).intermissionQueued != 0 {
        return;
    }
    if inflictor.is_null() {
        inflictor = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(ENTITYNUM_WORLD as usize);
    }
    if attacker.is_null() {
        attacker = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(ENTITYNUM_WORLD as usize);
    }

    // shootable doors / buttons don't actually have any health

    //if genericValue4 == 1 then it's glass or a breakable and those do have health
    if (*targ).s.eType == ET_MOVER && (*targ).genericValue4 != 1 {
        if (*targ).r#use.is_some() && (*targ).moverState == MOVER_POS1 {
            GlobalUse(targ, inflictor, attacker);
        }
        return;
    }
    // reduce damage by the attacker's handicap value
    // unless they are rocket jumping
    if !(*attacker).client.is_null() && attacker != targ && (*attacker).s.eType == ET_PLAYER {
        let max = (*(*attacker).client).ps.stats[STAT_MAX_HEALTH as usize];
        damage = damage * max / 100;
    }

    if dflags & DAMAGE_NO_HIT_LOC == 0 {
        //see if we should modify it by damage location
        if (*targ).inuse != QFALSE
            && (!(*targ).client.is_null() || (*targ).s.eType == ET_NPC)
            && (*attacker).inuse != QFALSE
            && (!(*attacker).client.is_null() || (*attacker).s.eType == ET_NPC)
        {
            //check for location based damage stuff.
            G_LocationBasedDamageModifier(targ, point, mod_, dflags, &mut damage);
        }
    }

    if !(*targ).client.is_null()
        && (*(*targ).client).NPC_class == CLASS_RANCOR
        && (attacker.is_null()
            || (*attacker).client.is_null()
            || (*(*attacker).client).NPC_class != CLASS_RANCOR)
    {
        // I guess always do 10 points of damage...feel free to tweak as needed
        if damage < 10 {
            //ignore piddly little damage
            damage = 0;
        } else if damage >= 10 {
            damage = 10;
        }
    }

    client = (*targ).client;

    if !client.is_null() {
        if (*client).noclip != QFALSE {
            return;
        }
    }

    if dir.is_null() {
        dflags |= DAMAGE_NO_KNOCKBACK;
    } else {
        VectorNormalize(&mut *dir);
    }

    knockback = damage;
    if knockback > 200 {
        knockback = 200;
    }
    if (*targ).flags & FL_NO_KNOCKBACK != 0 {
        knockback = 0;
    }
    if dflags & DAMAGE_NO_KNOCKBACK != 0 {
        knockback = 0;
    }

    // figure momentum add, even if the damage won't be taken
    if knockback != 0 && !(*targ).client.is_null() {
        let mut kvel: vec3_t = [0.0; 3];
        let mass: f32 = 200.0;

        if mod_ == MOD_SABER {
            let mut saber_knockback_scale: f32 = (*addr_of!(g_saberDmgVelocityScale)).value;
            if dflags & DAMAGE_SABER_KNOCKBACK1 != 0 || dflags & DAMAGE_SABER_KNOCKBACK2 != 0 {
                //saber does knockback, scale it by the right number
                if !attacker.is_null() && !(*attacker).client.is_null() {
                    if dflags & DAMAGE_SABER_KNOCKBACK1 != 0 && dflags & DAMAGE_SABER_KNOCKBACK2 != 0
                    {
                        //hit with both
                        saber_knockback_scale *= ((*(*attacker).client).saber[0].knockbackScale
                            + (*(*attacker).client).saber[1].knockbackScale)
                            * 0.5;
                    } else if dflags & DAMAGE_SABER_KNOCKBACK1 != 0 {
                        //hit with first only
                        saber_knockback_scale *= (*(*attacker).client).saber[0].knockbackScale;
                    } else {
                        //second only
                        saber_knockback_scale *= (*(*attacker).client).saber[1].knockbackScale;
                    }
                }
            }
            VectorScale(
                &*dir,
                ((*addr_of!(g_knockback)).value * knockback as f32 / mass) * saber_knockback_scale,
                &mut kvel,
            );
        } else {
            VectorScale(
                &*dir,
                (*addr_of!(g_knockback)).value * knockback as f32 / mass,
                &mut kvel,
            );
        }
        let v_cur = (*(*targ).client).ps.velocity;
        VectorAdd(&v_cur, &kvel, &mut (*(*targ).client).ps.velocity);

        if !attacker.is_null() && !(*attacker).client.is_null() && attacker != targ {
            let mut dur: f32 = 5000.0;
            let mut dur2: f32 = 100.0;
            if !(*targ).client.is_null()
                && (*targ).s.eType == ET_NPC
                && (*targ).s.NPC_class == CLASS_VEHICLE
            {
                dur = 25000.0;
                dur2 = 25000.0;
            }
            (*(*targ).client).ps.otherKiller = (*attacker).s.number;
            (*(*targ).client).ps.otherKillerTime = ((*addr_of!(level)).time as f32 + dur) as c_int;
            (*(*targ).client).ps.otherKillerDebounceTime =
                ((*addr_of!(level)).time as f32 + dur2) as c_int;
        }
        // set the timer so that the other client can't cancel
        // out the movement immediately
        if (*(*targ).client).ps.pm_time == 0
            && ((*addr_of!(g_saberDmgVelocityScale)).integer != 0
                || mod_ != MOD_SABER
                || dflags & DAMAGE_SABER_KNOCKBACK1 != 0
                || dflags & DAMAGE_SABER_KNOCKBACK2 != 0)
        {
            let mut t: c_int;

            t = knockback * 2;
            if t < 50 {
                t = 50;
            }
            if t > 200 {
                t = 200;
            }
            (*(*targ).client).ps.pm_time = t;
            (*(*targ).client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
        }
    } else if !(*targ).client.is_null()
        && (*targ).s.eType == ET_NPC
        && (*targ).s.NPC_class == CLASS_VEHICLE
        && attacker != targ
    {
        (*(*targ).client).ps.otherKiller = (*attacker).s.number;
        (*(*targ).client).ps.otherKillerTime = (*addr_of!(level)).time + 25000;
        (*(*targ).client).ps.otherKillerDebounceTime = (*addr_of!(level)).time + 25000;
    }

    if ((*addr_of!(g_trueJedi)).integer != 0 || (*addr_of!(g_gametype)).integer == GT_SIEGE)
        && !client.is_null()
    {
        //less explosive damage for jedi, more saber damage for non-jedi
        if (*client).ps.trueJedi != QFALSE
            || ((*addr_of!(g_gametype)).integer == GT_SIEGE && (*client).ps.weapon == WP_SABER)
        {
            //if the target is a trueJedi, reduce splash and explosive damage to 1/2
            match mod_ {
                MOD_REPEATER_ALT | MOD_REPEATER_ALT_SPLASH | MOD_DEMP2_ALT
                | MOD_FLECHETTE_ALT_SPLASH | MOD_ROCKET | MOD_ROCKET_SPLASH | MOD_ROCKET_HOMING
                | MOD_ROCKET_HOMING_SPLASH | MOD_THERMAL | MOD_THERMAL_SPLASH
                | MOD_TRIP_MINE_SPLASH | MOD_TIMED_MINE_SPLASH | MOD_DET_PACK_SPLASH => {
                    damage = (damage as f64 * 0.75) as c_int;
                }
                _ => {}
            }
        } else if ((*client).ps.trueNonJedi != QFALSE
            || ((*addr_of!(g_gametype)).integer == GT_SIEGE && (*client).ps.weapon != WP_SABER))
            && mod_ == MOD_SABER
        {
            //if the target is a trueNonJedi, take more saber damage... combined with the 1.5 in the w_saber stuff, this is 6 times damage!
            if damage < 100 {
                damage *= 4;
                if damage > 100 {
                    damage = 100;
                }
            }
        }
    }

    if !(*attacker).client.is_null()
        && !(*targ).client.is_null()
        && (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*(*targ).client).siegeClass != -1
        && (bgSiegeClasses[(*(*targ).client).siegeClass as usize].classflags
            & (1 << CFL_STRONGAGAINSTPHYSICAL))
            != 0
    {
        //this class is flagged to take less damage from physical attacks.
        //For now I'm just decreasing against any client-based attack, this can be changed later I guess.
        damage = (damage as f64 * 0.5) as c_int;
    }

    // check for completely getting out of the damage
    if dflags & DAMAGE_NO_PROTECTION == 0 {
        // if TF_NO_FRIENDLY_FIRE is set, don't do damage to the target
        // if the attacker was on the same team
        if targ != attacker {
            if OnSameTeam(targ, attacker) != QFALSE {
                if (*addr_of!(g_friendlyFire)).integer == 0 {
                    return;
                }
            } else if !attacker.is_null()
                && (*attacker).inuse != QFALSE
                && (*attacker).client.is_null()
                && !(*attacker).activator.is_null()
                && targ != (*attacker).activator
                && (*(*attacker).activator).inuse != QFALSE
                && !(*(*attacker).activator).client.is_null()
            {
                //emplaced guns don't hurt teammates of user
                if OnSameTeam(targ, (*attacker).activator) != QFALSE {
                    if (*addr_of!(g_friendlyFire)).integer == 0 {
                        return;
                    }
                }
            } else if (*targ).inuse != QFALSE
                && !(*targ).client.is_null()
                && (*addr_of!(g_gametype)).integer >= GT_TEAM
                && (*attacker).s.number >= MAX_CLIENTS as c_int
                && (*attacker).alliedTeam != 0
                && (*(*targ).client).sess.sessionTeam == (*attacker).alliedTeam
                && (*addr_of!(g_friendlyFire)).integer == 0
            {
                //things allied with my team should't hurt me.. I guess
                return;
            }
        }
        /*
                if (g_gametype.integer == GT_JEDIMASTER && !g_friendlyFire.integer &&
                    targ && targ->client && attacker && attacker->client &&
                    targ != attacker && !targ->client->ps.isJediMaster && !attacker->client->ps.isJediMaster &&
                    G_ThereIsAMaster())
                {
                    return;
                }
        */
        if (*targ).s.number >= MAX_CLIENTS as c_int
            && !(*targ).client.is_null()
            && (*targ).s.shouldtarget != QFALSE
            && (*targ).s.teamowner != 0
            && !attacker.is_null()
            && (*attacker).inuse != QFALSE
            && !(*attacker).client.is_null()
            && (*targ).s.owner >= 0
            && (*targ).s.owner < MAX_CLIENTS as c_int
        {
            let targown = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*targ).s.owner as usize);

            if !targown.is_null()
                && (*targown).inuse != QFALSE
                && !(*targown).client.is_null()
                && OnSameTeam(targown, attacker) != QFALSE
            {
                if (*addr_of!(g_friendlyFire)).integer == 0 {
                    return;
                }
            }
        }

        // check for godmode
        if (*targ).flags & FL_GODMODE != 0 && (*targ).s.eType != ET_NPC {
            return;
        }

        if !targ.is_null()
            && !(*targ).client.is_null()
            && (*(*targ).client).ps.eFlags & EF_INVULNERABLE != 0
            && !attacker.is_null()
            && !(*attacker).client.is_null()
            && targ != attacker
        {
            if (*(*targ).client).invulnerableTimer <= (*addr_of!(level)).time {
                (*(*targ).client).ps.eFlags &= !EF_INVULNERABLE;
            } else {
                return;
            }
        }
    }

    //check for teamnodmg
    //NOTE: non-client objects hitting clients (and clients hitting clients) purposely doesn't obey this teamnodmg (for emplaced guns)
    if !attacker.is_null() && (*targ).client.is_null() {
        //attacker hit a non-client
        if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*addr_of!(g_ff_objectives)).integer == 0
        {
            //in siege mode (and...?)
            if (*targ).teamnodmg != 0 {
                //targ shouldn't take damage from a certain team
                if !(*attacker).client.is_null() {
                    //a client hit a non-client object
                    if (*targ).teamnodmg == (*(*attacker).client).sess.sessionTeam {
                        return;
                    }
                } else if (*attacker).teamnodmg != 0 {
                    //a non-client hit a non-client object
                    //FIXME: maybe check alliedTeam instead?
                    if (*targ).teamnodmg == (*attacker).teamnodmg {
                        if !(*attacker).activator.is_null()
                            && (*(*attacker).activator).inuse != QFALSE
                            && (*(*attacker).activator).s.number < MAX_CLIENTS as c_int
                            && !(*(*attacker).activator).client.is_null()
                            && (*(*(*attacker).activator).client).sess.sessionTeam
                                != (*targ).teamnodmg
                        {
                            //uh, let them damage it I guess.
                        } else {
                            return;
                        }
                    }
                }
            }
        }
    }

    // battlesuit protects from all radius damage (but takes knockback)
    // and protects 50% against all damage
    if !client.is_null() && (*client).ps.powerups[PW_BATTLESUIT as usize] != 0 {
        G_AddEvent(targ, EV_POWERUP_BATTLESUIT, 0);
        if dflags & DAMAGE_RADIUS != 0 || mod_ == MOD_FALLING {
            return;
        }
        damage = (damage as f64 * 0.5) as c_int;
    }

    // add to the attacker's hit counter (if the target isn't a general entity like a prox mine)
    if !(*attacker).client.is_null()
        && targ != attacker
        && (*targ).health > 0
        && (*targ).s.eType != ET_MISSILE
        && (*targ).s.eType != ET_GENERAL
        && !client.is_null()
    {
        if OnSameTeam(targ, attacker) != QFALSE {
            (*(*attacker).client).ps.persistant[PERS_HITS as usize] -= 1;
        } else {
            (*(*attacker).client).ps.persistant[PERS_HITS as usize] += 1;
        }
        (*(*attacker).client).ps.persistant[PERS_ATTACKEE_ARMOR as usize] =
            ((*targ).health << 8) | (*client).ps.stats[STAT_ARMOR as usize];
    }

    // always give half damage if hurting self... but not in siege.  Heavy weapons need a counter.
    // calculated after knockback, so rocket jumping works
    if targ == attacker && dflags & DAMAGE_NO_SELF_PROTECTION == 0 {
        if (*addr_of!(g_gametype)).integer == GT_SIEGE {
            damage = (damage as f64 * 1.5) as c_int;
        } else {
            damage = (damage as f64 * 0.5) as c_int;
        }
    }

    if damage < 1 {
        damage = 1;
    }
    take = damage;

    // save some from armor
    asave = CheckArmor(targ, take, dflags);

    if asave != 0 {
        shield_absorbed = asave as f32;
    }

    take -= asave;
    if !(*targ).client.is_null() {
        //update vehicle shields and armor, check for explode
        if (*(*targ).client).NPC_class == CLASS_VEHICLE && !(*targ).m_pVehicle.is_null() {
            //FIXME: should be in its own function in g_vehicles.c now, too big to be here
            let mut surface: c_int = -1;
            if !attacker.is_null() {
                //so we know the last guy who shot at us
                (*targ).enemy = attacker;
            }

            if (*(*(*targ).m_pVehicle).m_pVehicleInfo).r#type == VH_ANIMAL {
                //((CVehicleNPC *)targ->NPC)->m_ulFlags |= CVehicleNPC::VEH_BUCKING;
            }

            (*(*targ).m_pVehicle).m_iShields = (*(*targ).client).ps.stats[STAT_ARMOR as usize];
            G_VehUpdateShields(targ);
            (*(*targ).m_pVehicle).m_iArmor -= take;
            if (*(*targ).m_pVehicle).m_iArmor <= 0 {
                (*targ).s.eFlags |= EF_DEAD;
                (*(*targ).client).ps.eFlags |= EF_DEAD;
                (*(*targ).m_pVehicle).m_iArmor = 0;
            }
            if (*(*(*targ).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER {
                //get the last surf that was hit
                if !(*targ).client.is_null()
                    && (*(*targ).client).g2LastSurfaceTime == (*addr_of!(level)).time
                {
                    let mut hit_surface: [c_char; MAX_QPATH] = [0; MAX_QPATH];

                    trap::G2API_GetSurfaceName(
                        (*targ).ghoul2,
                        (*(*targ).client).g2LastSurfaceHit,
                        0,
                        hit_surface.as_mut_ptr(),
                    );

                    if hit_surface[0] != 0 {
                        surface = G_ShipSurfaceForSurfName(hit_surface.as_ptr());

                        if take != 0 && surface > 0 {
                            //hit a certain part of the ship
                            let mut death_point: c_int = 0;

                            (*targ).locationDamage[surface as usize] += take;

                            match surface {
                                SHIPSURF_FRONT => {
                                    death_point = (*(*(*targ).m_pVehicle).m_pVehicleInfo).health_front;
                                }
                                SHIPSURF_BACK => {
                                    death_point = (*(*(*targ).m_pVehicle).m_pVehicleInfo).health_back;
                                }
                                SHIPSURF_RIGHT => {
                                    death_point = (*(*(*targ).m_pVehicle).m_pVehicleInfo).health_right;
                                }
                                SHIPSURF_LEFT => {
                                    death_point = (*(*(*targ).m_pVehicle).m_pVehicleInfo).health_left;
                                }
                                _ => {}
                            }

                            //presume 0 means it wasn't set and so it should never die.
                            if death_point != 0 {
                                if (*targ).locationDamage[surface as usize] >= death_point {
                                    //this area of the ship is now dead
                                    if G_FlyVehicleDestroySurface(targ, surface) != QFALSE {
                                        //actually took off a surface
                                        G_VehicleSetDamageLocFlags(targ, surface, death_point);
                                    }
                                } else {
                                    G_VehicleSetDamageLocFlags(targ, surface, death_point);
                                }
                            }
                        }
                    }
                }
            }
            if (*(*(*targ).m_pVehicle).m_pVehicleInfo).r#type != VH_ANIMAL {
                /*
                if ( targ->m_pVehicle->m_iArmor <= 0 )
                {//vehicle all out of armor
                    Vehicle_t *pVeh = targ->m_pVehicle;
                    if ( pVeh->m_iDieTime == 0 )
                    {//just start the flaming effect and explosion delay, if it's not going already...
                        pVeh->m_pVehicleInfo->StartDeathDelay( pVeh, 0 );
                    }
                }
                else*/
                if !attacker.is_null()
                    //&& attacker->client
                    && targ != attacker
                    && !point.is_null()
                    && VectorCompare(&(*(*targ).client).ps.origin, &*point) == 0
                    && (*(*targ).m_pVehicle).m_LandTrace.fraction >= 1.0
                {
                    //just took a hit, knock us around
                    let mut v_up: vec3_t = [0.0; 3];
                    let mut impact_dir: vec3_t = [0.0; 3];
                    let mut impact_strength: f32 = (damage as f32 / 200.0) * 10.0;
                    let mut dot: f32;
                    if impact_strength > 10.0 {
                        impact_strength = 10.0;
                    }
                    //pitch or roll us based on where we were hit
                    AngleVectors(
                        &*((*(*targ).m_pVehicle).m_vOrientation as *const vec3_t),
                        None,
                        None,
                        Some(&mut v_up),
                    );
                    VectorSubtract(&*point, &(*targ).r.currentOrigin, &mut impact_dir);
                    VectorNormalize(&mut impact_dir);
                    if surface <= 0 {
                        //no surf guess where we were hit, then
                        let mut v_fwd: vec3_t = [0.0; 3];
                        let mut v_right: vec3_t = [0.0; 3];
                        AngleVectors(
                            &*((*(*targ).m_pVehicle).m_vOrientation as *const vec3_t),
                            Some(&mut v_fwd),
                            Some(&mut v_right),
                            Some(&mut v_up),
                        );
                        dot = DotProduct(&v_right, &impact_dir);
                        if dot > 0.4 {
                            surface = SHIPSURF_RIGHT;
                        } else if dot < -0.4 {
                            surface = SHIPSURF_LEFT;
                        } else {
                            dot = DotProduct(&v_fwd, &impact_dir);
                            if dot > 0.0 {
                                surface = SHIPSURF_FRONT;
                            } else {
                                surface = SHIPSURF_BACK;
                            }
                        }
                    }
                    match surface {
                        SHIPSURF_FRONT => {
                            dot = DotProduct(&v_up, &impact_dir);
                            if dot > 0.0 {
                                *(*(*targ).m_pVehicle).m_vOrientation.add(PITCH) += impact_strength;
                            } else {
                                *(*(*targ).m_pVehicle).m_vOrientation.add(PITCH) -= impact_strength;
                            }
                        }
                        SHIPSURF_BACK => {
                            dot = DotProduct(&v_up, &impact_dir);
                            if dot > 0.0 {
                                *(*(*targ).m_pVehicle).m_vOrientation.add(PITCH) -= impact_strength;
                            } else {
                                *(*(*targ).m_pVehicle).m_vOrientation.add(PITCH) += impact_strength;
                            }
                        }
                        SHIPSURF_RIGHT => {
                            dot = DotProduct(&v_up, &impact_dir);
                            if dot > 0.0 {
                                *(*(*targ).m_pVehicle).m_vOrientation.add(ROLL) -= impact_strength;
                            } else {
                                *(*(*targ).m_pVehicle).m_vOrientation.add(ROLL) += impact_strength;
                            }
                        }
                        SHIPSURF_LEFT => {
                            dot = DotProduct(&v_up, &impact_dir);
                            if dot > 0.0 {
                                *(*(*targ).m_pVehicle).m_vOrientation.add(ROLL) += impact_strength;
                            } else {
                                *(*(*targ).m_pVehicle).m_vOrientation.add(ROLL) -= impact_strength;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    if mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT {
        //FIXME: screw with non-animal vehicles, too?
        if !client.is_null() {
            if (*client).NPC_class == CLASS_VEHICLE
                && !(*targ).m_pVehicle.is_null()
                && !(*(*targ).m_pVehicle).m_pVehicleInfo.is_null()
                && (*(*(*targ).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER
            {
                //all damage goes into the disruption of shields and systems
                take = 0;
            } else {
                if (*client).jetPackOn != QFALSE {
                    //disable jetpack temporarily
                    Jetpack_Off(targ);
                    (*client).jetPackToggleTime = (*addr_of!(level)).time + Q_irand(3000, 10000);
                }

                if (*client).NPC_class == CLASS_PROTOCOL
                    || (*client).NPC_class == CLASS_SEEKER
                    || (*client).NPC_class == CLASS_R2D2
                    || (*client).NPC_class == CLASS_R5D2
                    || (*client).NPC_class == CLASS_MOUSE
                    || (*client).NPC_class == CLASS_GONK
                {
                    // DEMP2 does more damage to these guys.
                    take *= 2;
                } else if (*client).NPC_class == CLASS_PROBE
                    || (*client).NPC_class == CLASS_INTERROGATOR
                    || (*client).NPC_class == CLASS_MARK1
                    || (*client).NPC_class == CLASS_MARK2
                    || (*client).NPC_class == CLASS_SENTRY
                    || (*client).NPC_class == CLASS_ATST
                {
                    // DEMP2 does way more damage to these guys.
                    take *= 5;
                } else if take > 0 {
                    take /= 3;
                    if take < 1 {
                        take = 1;
                    }
                }
            }
        }
    }

    // `#ifndef FINAL_BUILD` — present in the shipping (non-FINAL_BUILD) MP module.
    if (*addr_of!(g_debugDamage)).integer != 0 {
        G_Printf(&format!(
            "{}: client:{} health:{} damage:{} armor:{}\n",
            (*addr_of!(level)).time,
            (*targ).s.number,
            (*targ).health,
            take,
            asave
        ));
    }

    // add to the damage inflicted on a player this frame
    // the total will be turned into screen blends and view angle kicks
    // at the end of the frame
    if !client.is_null() {
        if !attacker.is_null() {
            (*client).ps.persistant[PERS_ATTACKER as usize] = (*attacker).s.number;
        } else {
            (*client).ps.persistant[PERS_ATTACKER as usize] = ENTITYNUM_WORLD;
        }
        (*client).damage_armor += asave;
        (*client).damage_blood += take;
        (*client).damage_knockback += knockback;
        if !dir.is_null() {
            VectorCopy(&*dir, &mut (*client).damage_from);
            (*client).damage_fromWorld = QFALSE;
        } else {
            VectorCopy(&(*targ).r.currentOrigin, &mut (*client).damage_from);
            (*client).damage_fromWorld = QTRUE;
        }

        if !attacker.is_null() && !(*attacker).client.is_null() {
            BotDamageNotification(client, attacker);
        } else if !inflictor.is_null() && !(*inflictor).client.is_null() {
            BotDamageNotification(client, inflictor);
        }
    }

    // See if it's the player hurting the emeny flag carrier
    if (*addr_of!(g_gametype)).integer == GT_CTF || (*addr_of!(g_gametype)).integer == GT_CTY {
        Team_CheckHurtCarrier(targ, attacker);
    }

    if !(*targ).client.is_null() {
        // set the last client who damaged the target
        (*(*targ).client).lasthurt_client = (*attacker).s.number;
        (*(*targ).client).lasthurt_mod = mod_;
    }

    if take != 0
        && !(*targ).client.is_null()
        && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0
    {
        if (*(*targ).client).ps.fd.forcePower != 0 {
            let mut maxtake: c_int = take;

            //G_Sound(targ, CHAN_AUTO, protectHitSound);
            if (*(*targ).client).forcePowerSoundDebounce < (*addr_of!(level)).time {
                G_PreDefSound(&(*(*targ).client).ps.origin, PDSOUND_PROTECTHIT);
                (*(*targ).client).forcePowerSoundDebounce = (*addr_of!(level)).time + 400;
            }

            if (*(*targ).client).ps.fd.forcePowerLevel[FP_PROTECT as usize] == FORCE_LEVEL_1 {
                famt = 1.0;
                hamt = 0.40;

                if maxtake > 100 {
                    maxtake = 100;
                }
            } else if (*(*targ).client).ps.fd.forcePowerLevel[FP_PROTECT as usize] == FORCE_LEVEL_2 {
                famt = 0.5;
                hamt = 0.60;

                if maxtake > 200 {
                    maxtake = 200;
                }
            } else if (*(*targ).client).ps.fd.forcePowerLevel[FP_PROTECT as usize] == FORCE_LEVEL_3 {
                famt = 0.25;
                hamt = 0.80;

                if maxtake > 400 {
                    maxtake = 400;
                }
            }

            if (*(*targ).client).ps.powerups[PW_FORCE_BOON as usize] == 0 {
                (*(*targ).client).ps.fd.forcePower =
                    ((*(*targ).client).ps.fd.forcePower as f32 - maxtake as f32 * famt) as c_int;
            } else {
                (*(*targ).client).ps.fd.forcePower = ((*(*targ).client).ps.fd.forcePower as f32
                    - (maxtake as f32 * famt) / 2.0)
                    as c_int;
            }
            subamt = (maxtake as f32 * hamt + (take - maxtake) as f32) as c_int;
            if (*(*targ).client).ps.fd.forcePower < 0 {
                subamt += (*(*targ).client).ps.fd.forcePower;
                (*(*targ).client).ps.fd.forcePower = 0;
            }
            if subamt != 0 {
                take -= subamt;

                if take < 0 {
                    take = 0;
                }
            }
        }
    }

    if shield_absorbed != 0.0 {
        /*
        if ( targ->client->NPC_class == CLASS_VEHICLE )
        {
            targ->client->ps.electrifyTime = level.time + Q_irand( 500, 1000 );
        }
        else
        */
        {
            // Send off an event to show a shield shell on the player, pointing in the right direction.
            //evEnt = G_TempEntity(vec3_origin, EV_SHIELD_HIT);
            //rww - er.. what the? This isn't broadcast, why is it being set on vec3_origin?!
            let ev_ent = G_TempEntity(&(*targ).r.currentOrigin, EV_SHIELD_HIT);
            (*ev_ent).s.otherEntityNum = (*targ).s.number;
            (*ev_ent).s.eventParm = DirToByte(&*dir);
            (*ev_ent).s.time2 = shield_absorbed as c_int;
            /*
                    shieldAbsorbed *= 20;

                    if (shieldAbsorbed > 1500)
                    {
                        shieldAbsorbed = 1500;
                    }
                    if (shieldAbsorbed < 200)
                    {
                        shieldAbsorbed = 200;
                    }

                    if (targ->client->ps.powerups[PW_SHIELDHIT] < (level.time + shieldAbsorbed))
                    {
                        targ->client->ps.powerups[PW_SHIELDHIT] = level.time + shieldAbsorbed;
                    }
                    //flicker for as many ms as damage was absorbed (*20)
                    //therefore 10 damage causes 1/5 of a seond of flickering, whereas
                    //a full 100 causes 2 seconds (but is reduced to 1.5 seconds due to the max)

            */
        }
    }

    // do the damage
    if take != 0 {
        if !(*targ).client.is_null()
            && (*targ).s.number < MAX_CLIENTS as c_int
            && (mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT)
        {
            //uh.. shock them or something. what the hell, I don't know.
            if (*(*targ).client).ps.weaponTime <= 0 {
                //yeah, we were supposed to be beta a week ago, I don't feel like
                //breaking the game so I'm gonna be safe and only do this only
                //if your weapon is not busy
                (*(*targ).client).ps.weaponTime = 2000;
                (*(*targ).client).ps.electrifyTime = (*addr_of!(level)).time + 2000;
                if (*(*targ).client).ps.weaponstate == WEAPON_CHARGING
                    || (*(*targ).client).ps.weaponstate == WEAPON_CHARGING_ALT
                {
                    (*(*targ).client).ps.weaponstate = WEAPON_READY;
                }
            }
        }

        if !(*targ).client.is_null()
            && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
            && (!(*inflictor).client.is_null() || !(*attacker).client.is_null())
        {
            take /= (*(*targ).client).ps.fd.forcePowerLevel[FP_RAGE as usize] + 1;
        }
        (*targ).health -= take;

        if (*targ).flags & FL_UNDYING != 0 {
            //take damage down to 1, but never die
            if (*targ).health < 1 {
                (*targ).health = 1;
            }
        }

        if !(*targ).client.is_null() {
            (*(*targ).client).ps.stats[STAT_HEALTH as usize] = (*targ).health;
        }

        if !(*targ).client.is_null()
            && (*(*targ).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
            && (!(*inflictor).client.is_null() || !(*attacker).client.is_null())
        {
            if (*targ).health <= 0 {
                (*targ).health = 1;
            }
            if (*(*targ).client).ps.stats[STAT_HEALTH as usize] <= 0 {
                (*(*targ).client).ps.stats[STAT_HEALTH as usize] = 1;
            }
        }

        //We want to go ahead and set gPainHitLoc regardless of if we have a pain func,
        //so we can adjust the location damage too.
        if !(*targ).client.is_null()
            && !(*targ).ghoul2.is_null()
            && (*(*targ).client).g2LastSurfaceTime == (*addr_of!(level)).time
        {
            //We updated the hit surface this frame, so it's valid.
            let mut hit_surface: [c_char; MAX_QPATH] = [0; MAX_QPATH];

            trap::G2API_GetSurfaceName(
                (*targ).ghoul2,
                (*(*targ).client).g2LastSurfaceHit,
                0,
                hit_surface.as_mut_ptr(),
            );

            if hit_surface[0] != 0 {
                G_GetHitLocFromSurfName(
                    targ,
                    hit_surface.as_ptr(),
                    addr_of_mut!(gPainHitLoc),
                    point,
                    dir,
                    &vec3_origin as *const vec3_t,
                    mod_,
                );
            } else {
                *addr_of_mut!(gPainHitLoc) = -1;
            }

            if *addr_of!(gPainHitLoc) < HL_MAX
                && *addr_of!(gPainHitLoc) >= 0
                && (*targ).locationDamage[*addr_of!(gPainHitLoc) as usize] < Q3_INFINITE
                && ((*targ).s.eType == ET_PLAYER || (*targ).s.NPC_class != CLASS_VEHICLE)
            {
                (*targ).locationDamage[*addr_of!(gPainHitLoc) as usize] += take;

                if (*addr_of!(g_armBreakage)).integer != 0
                    && (*(*targ).client).ps.brokenLimbs == 0
                    && (*(*targ).client).ps.stats[STAT_HEALTH as usize] > 0
                    && (*targ).health > 0
                    && (*targ).s.eFlags & EF_DEAD == 0
                {
                    //check for breakage
                    if (*targ).locationDamage[HL_ARM_RT as usize]
                        + (*targ).locationDamage[HL_HAND_RT as usize]
                        >= 80
                    {
                        G_BreakArm(targ, BROKENLIMB_RARM);
                    } else if (*targ).locationDamage[HL_ARM_LT as usize]
                        + (*targ).locationDamage[HL_HAND_LT as usize]
                        >= 80
                    {
                        G_BreakArm(targ, BROKENLIMB_LARM);
                    }
                }
            }
        } else {
            *addr_of_mut!(gPainHitLoc) = -1;
        }

        if (*targ).maxHealth != 0 {
            //if this is non-zero this guy should be updated his s.health to send to the client
            G_ScaleNetHealth(targ);
        }

        if (*targ).health <= 0 {
            if !client.is_null() {
                (*targ).flags |= FL_NO_KNOCKBACK;

                if !point.is_null() {
                    VectorCopy(&*point, &mut (*targ).pos1);
                } else {
                    VectorCopy(&(*(*targ).client).ps.origin, &mut (*targ).pos1);
                }
            } else if (*targ).s.eType == ET_NPC {
                //g2animent
                VectorCopy(&*point, &mut (*targ).pos1);
            }

            if (*targ).health < -999 {
                (*targ).health = -999;
            }

            // If we are a breaking glass brush, store the damage point so we can do cool things with it.
            if (*targ).r.svFlags & SVF_GLASS_BRUSH != 0 {
                VectorCopy(&*point, &mut (*targ).pos1);
                if !dir.is_null() {
                    VectorCopy(&*dir, &mut (*targ).pos2);
                } else {
                    VectorClear(&mut (*targ).pos2);
                }
            }

            if (*targ).s.eType == ET_NPC
                && !(*targ).client.is_null()
                && (*targ).s.eFlags & EF_DEAD != 0
            {
                //an NPC that's already dead. Maybe we can cut some more limbs off!
                if (mod_ == MOD_SABER || (mod_ == MOD_MELEE && G_HeavyMelee(attacker) != QFALSE))
                    //saber or heavy melee (claws)
                    && take > 2
                    && dflags & DAMAGE_NO_DISMEMBER == 0
                {
                    G_CheckForDismemberment(
                        targ,
                        attacker,
                        &(*targ).pos1,
                        take,
                        (*(*targ).client).ps.torsoAnim,
                        QTRUE,
                    );
                }
            }

            (*targ).enemy = attacker;
            if let Some(die) = (*targ).die {
                die(targ, inflictor, attacker, take, mod_);
            }
            G_ActivateBehavior(targ, BSET_DEATH);
            return;
        } else {
            if (*addr_of!(g_debugMelee)).integer != 0 {
                //getting hurt makes you let go of the wall
                if !(*targ).client.is_null()
                    && (*(*targ).client).ps.pm_flags & PMF_STUCK_TO_WALL != 0
                {
                    G_LetGoOfWall(targ);
                }
            }
            if let Some(pain) = (*targ).pain {
                if (*targ).s.eType != ET_NPC || mod_ != MOD_SABER || take > 1 {
                    //don't even notify NPCs of pain if it's just idle saber damage
                    *addr_of_mut!(gPainMOD) = mod_;
                    if !point.is_null() {
                        VectorCopy(&*point, &mut *addr_of_mut!(gPainPoint));
                    } else {
                        VectorCopy(&(*targ).r.currentOrigin, &mut *addr_of_mut!(gPainPoint));
                    }
                    pain(targ, attacker, take);
                }
            }
        }

        G_LogWeaponDamage((*attacker).s.number, mod_, take);
    }
}

/// `G_DamageFromKiller` (g_combat.c:5717) — apply `G_Damage` to `pEnt` while crediting the
/// kill to whoever last put `pVehEnt`'s pilot into a fatal state (the `otherKiller*` fields on
/// `pVehEnt->client`), rather than to the immediate `attacker`. Used by the vehicle
/// death-spiral impact paths in `PM_VehicleImpact`. If the credited killer used a vehicle
/// weapon, a temporary "vehicle_proj" inflictor entity is spawned (and freed afterward) so the
/// obituary attributes the correct weapon. If the killer is itself a piloted vehicle, credit
/// passes through to its pilot.
///
/// # Safety
/// `pEnt`/`pVehEnt`/`attacker` may be NULL (guarded); `g_entities`/`level` must be live.
pub unsafe fn G_DamageFromKiller(
    pEnt: *mut gentity_t,
    pVehEnt: *mut gentity_t,
    attacker: *mut gentity_t,
    org: *mut vec3_t,
    damage: c_int,
    dflags: c_int,
    mut mod_: c_int,
) {
    let mut killer: *mut gentity_t = attacker;
    let mut inflictor: *mut gentity_t = attacker;
    let mut tempInflictor: qboolean = QFALSE;
    if pEnt.is_null() || pVehEnt.is_null() || (*pVehEnt).client.is_null() {
        return;
    }
    if (*(*pVehEnt).client).ps.otherKiller < ENTITYNUM_WORLD
        && (*(*pVehEnt).client).ps.otherKillerTime > (*addr_of!(level)).time
    {
        let potentialKiller: *mut gentity_t =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*pVehEnt).client).ps.otherKiller as usize);

        if (*potentialKiller).inuse != QFALSE
        //&& potentialKiller->client)
        {
            //he's valid I guess
            killer = potentialKiller;
            mod_ = (*(*pVehEnt).client).otherKillerMOD;
            inflictor = killer;
            if (*(*pVehEnt).client).otherKillerVehWeapon > 0 {
                inflictor = G_Spawn();
                if !inflictor.is_null() {
                    //fake up the inflictor
                    tempInflictor = QTRUE;
                    (*inflictor).classname = c"vehicle_proj".as_ptr() as *mut c_char;
                    (*inflictor).s.otherEntityNum2 =
                        (*(*pVehEnt).client).otherKillerVehWeapon - 1;
                    (*inflictor).s.weapon = (*(*pVehEnt).client).otherKillerWeaponType;
                }
            }
        }
    }
    //FIXME: damage hitEnt, some, too?  Our explosion should hurt them some, but...
    if !killer.is_null()
        && (*killer).s.eType == ET_NPC
        && (*killer).s.NPC_class == CLASS_VEHICLE
        && !(*killer).m_pVehicle.is_null()
        && !(*(*killer).m_pVehicle).m_pPilot.is_null()
    {
        killer = (*(*killer).m_pVehicle).m_pPilot as *mut gentity_t;
    }
    G_Damage(pEnt, inflictor, killer, null_mut(), org, damage, dflags, mod_);
    if tempInflictor != QFALSE {
        G_FreeEntity(inflictor);
    }
}

/// `qboolean CanDamage( gentity_t *targ, vec3_t origin )` (g_combat.c:5476).
///
/// Returns true if the inflictor at `origin` can directly damage `targ` — used for
/// explosions and melee attacks. Traces to the midpoint of `targ`'s bounding box (bmodels
/// often have their origin at 0,0,0), then to four ±15 offsets in the X/Y plane; any
/// unobstructed trace (`fraction == 1.0`) or one that hits `targ` itself passes. No oracle
/// (calls `trap_Trace`).
pub unsafe fn CanDamage(targ: *mut gentity_t, origin: &vec3_t) -> qboolean {
    let mut dest: vec3_t = [0.0; 3];
    let mut midpoint: vec3_t = [0.0; 3];

    // use the midpoint of the bounds instead of the origin, because
    // bmodels may have their origin is 0,0,0
    VectorAdd(&(*targ).r.absmin, &(*targ).r.absmax, &mut midpoint);
    let midpoint_in = midpoint;
    VectorScale(&midpoint_in, 0.5, &mut midpoint);

    VectorCopy(&midpoint, &mut dest);
    let tr = trap::Trace(origin, &vec3_origin, &vec3_origin, &dest, ENTITYNUM_NONE, MASK_SOLID);
    if tr.fraction == 1.0 || tr.entityNum as c_int == (*targ).s.number {
        return QTRUE;
    }

    // this should probably check in the plane of projection,
    // rather than in world coordinate, and also include Z
    VectorCopy(&midpoint, &mut dest);
    dest[0] += 15.0;
    dest[1] += 15.0;
    let tr = trap::Trace(origin, &vec3_origin, &vec3_origin, &dest, ENTITYNUM_NONE, MASK_SOLID);
    if tr.fraction == 1.0 {
        return QTRUE;
    }

    VectorCopy(&midpoint, &mut dest);
    dest[0] += 15.0;
    dest[1] -= 15.0;
    let tr = trap::Trace(origin, &vec3_origin, &vec3_origin, &dest, ENTITYNUM_NONE, MASK_SOLID);
    if tr.fraction == 1.0 {
        return QTRUE;
    }

    VectorCopy(&midpoint, &mut dest);
    dest[0] -= 15.0;
    dest[1] += 15.0;
    let tr = trap::Trace(origin, &vec3_origin, &vec3_origin, &dest, ENTITYNUM_NONE, MASK_SOLID);
    if tr.fraction == 1.0 {
        return QTRUE;
    }

    VectorCopy(&midpoint, &mut dest);
    dest[0] -= 15.0;
    dest[1] -= 15.0;
    let tr = trap::Trace(origin, &vec3_origin, &vec3_origin, &dest, ENTITYNUM_NONE, MASK_SOLID);
    if tr.fraction == 1.0 {
        return QTRUE;
    }

    QFALSE
}

/// `qboolean G_RadiusDamage( vec3_t origin, gentity_t *attacker, float damage, float radius,
/// gentity_t *ignore, gentity_t *missile, int mod )` (g_combat.c:5531).
///
/// Splash damage: gather every entity whose bbox overlaps the `radius` box around `origin`,
/// scale `damage` linearly by edge distance, line-of-sight gate each through [`CanDamage`],
/// and apply it via [`G_Damage`] with `DAMAGE_RADIUS` knockback (credited to the vehicle
/// pilot when the attacker is a piloted vehicle). Returns true if any live client was hit.
/// No oracle (`trap_EntitiesInBox` + the `G_Damage` chain + global `g_entities`).
///
/// # Safety
/// `attacker`/`ignore`/`missile` may be NULL; entity pointers are deref'd only behind the
/// same guards as the C.
pub unsafe fn G_RadiusDamage(
    origin: &vec3_t,
    attacker: *mut gentity_t,
    damage: f32,
    mut radius: f32,
    ignore: *mut gentity_t,
    missile: *mut gentity_t,
    mod_: c_int,
) -> qboolean {
    let mut points: f32;
    let mut dist: f32;
    let mut ent: *mut gentity_t;
    let mut entity_list = [0i32; MAX_GENTITIES];
    let num_listed_entities: c_int;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut v: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut hit_client = QFALSE;
    // C declares `qboolean roastPeople = qfalse;` mutable, but its only assignment lives in
    // the commented-out projectile-mark block below, so it stays false — declared immutable.
    let roast_people = QFALSE;

    /*
    if (missile && !missile->client && missile->s.weapon > WP_NONE &&
        missile->s.weapon < WP_NUM_WEAPONS && missile->r.ownerNum >= 0 &&
        (missile->r.ownerNum < MAX_CLIENTS || g_entities[missile->r.ownerNum].s.eType == ET_NPC))
    { //sounds like it's a valid weapon projectile.. is it a valid explosive to create marks from?
        switch(missile->s.weapon)
        {
        case WP_FLECHETTE: //flechette issuing this will be alt-fire
        case WP_ROCKET_LAUNCHER:
        case WP_THERMAL:
        case WP_TRIP_MINE:
        case WP_DET_PACK:
            roastPeople = qtrue; //Then create explosive marks
            break;
        default:
            break;
        }
    }
    */
    //oh well.. maybe sometime? I am trying to cut down on tempent use.

    if radius < 1.0 {
        radius = 1.0;
    }

    for i in 0..3 {
        mins[i] = origin[i] - radius;
        maxs[i] = origin[i] + radius;
    }

    num_listed_entities = trap::EntitiesInBox(&mins, &maxs, &mut entity_list);

    for e in 0..num_listed_entities {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entity_list[e as usize] as usize);

        if ent == ignore {
            continue;
        }
        if (*ent).takedamage == QFALSE {
            continue;
        }

        // find the distance from the edge of the bounding box
        for i in 0..3 {
            if origin[i] < (*ent).r.absmin[i] {
                v[i] = (*ent).r.absmin[i] - origin[i];
            } else if origin[i] > (*ent).r.absmax[i] {
                v[i] = origin[i] - (*ent).r.absmax[i];
            } else {
                v[i] = 0.0;
            }
        }

        dist = VectorLength(&v);
        if dist >= radius {
            continue;
        }

        // C: `points = damage * ( 1.0 - dist / radius )` — the `1.0` is a double, so the
        // subtraction and product evaluate in double, then truncate into the float `points`.
        points = (damage as f64 * (1.0 - (dist / radius) as f64)) as f32;

        if CanDamage(ent, origin) != QFALSE {
            if LogAccuracyHit(ent, attacker) != QFALSE {
                hit_client = QTRUE;
            }
            VectorSubtract(&(*ent).r.currentOrigin, origin, &mut dir);
            // push the center of mass higher than the origin so players
            // get knocked into the air more
            dir[2] += 24.0;
            if !attacker.is_null()
                && (*attacker).inuse != QFALSE
                && !(*attacker).client.is_null()
                && (*attacker).s.eType == ET_NPC
                && (*attacker).s.NPC_class == CLASS_VEHICLE
                && !(*attacker).m_pVehicle.is_null()
                && !(*(*attacker).m_pVehicle).m_pPilot.is_null()
            {
                // say my pilot did it.
                G_Damage(
                    ent,
                    missile,
                    (*(*attacker).m_pVehicle).m_pPilot as *mut gentity_t,
                    &mut dir,
                    origin as *const vec3_t as *mut vec3_t,
                    points as c_int,
                    DAMAGE_RADIUS,
                    mod_,
                );
            } else {
                G_Damage(
                    ent,
                    missile,
                    attacker,
                    &mut dir,
                    origin as *const vec3_t as *mut vec3_t,
                    points as c_int,
                    DAMAGE_RADIUS,
                    mod_,
                );
            }

            if !ent.is_null()
                && !(*ent).client.is_null()
                && roast_people != QFALSE
                && !missile.is_null()
                && VectorCompare(&(*ent).r.currentOrigin, &(*missile).r.currentOrigin) == 0
            {
                // the thing calling this function can create burn marks on people, so create an event to do so
                let ev_ent = G_TempEntity(&(*ent).r.currentOrigin, EV_GHOUL2_MARK);

                (*ev_ent).s.otherEntityNum = (*ent).s.number; // the entity the mark should be placed on
                (*ev_ent).s.weapon = WP_ROCKET_LAUNCHER; // always say it's rocket so we make the right mark

                // Try to place the decal by going from the missile location to the location of the person that was hit
                VectorCopy(&(*missile).r.currentOrigin, &mut (*ev_ent).s.origin);
                VectorCopy(&(*ent).r.currentOrigin, &mut (*ev_ent).s.origin2);

                // it's hacky, but we want to move it up so it's more likely to hit
                // the torso.
                if (*missile).r.currentOrigin[2] < (*ent).r.currentOrigin[2] {
                    // move it up less so the decal is placed lower on the model then
                    (*ev_ent).s.origin2[2] += 8.0;
                } else {
                    (*ev_ent).s.origin2[2] += 24.0;
                }

                // Special col check
                (*ev_ent).s.eventParm = 1;
            }
        }
    }

    hit_client
}

/// `int RaySphereIntersections( vec3_t origin, float radius, vec3_t point, vec3_t dir, vec3_t intersections[2] )`
/// (g_combat.c:3005).
///
/// Pure ray/sphere intersection: normalizes `dir` in place, solves
/// `|origin - (point + t*dir)| = radius` for `t`, writes up to two intersection points and
/// returns the hit count (0/1/2). Only `intersections[0]` is touched when there is a single
/// root, neither when there is none — matching the C, which leaves the rest untouched.
///
/// C's `sqrt` is the libm `double` routine, so the `f32` discriminant promotes to `f64`, the
/// root is taken there and rounds back to `f32`; the `f64` round-trip keeps it bit-exact.
pub fn RaySphereIntersections(
    origin: &vec3_t,
    radius: f32,
    point: &vec3_t,
    dir: &mut vec3_t,
    intersections: &mut [vec3_t; 2],
) -> c_int {
    // normalize dir so a = 1
    VectorNormalize(dir);
    let b: f32 = 2.0
        * (dir[0] * (point[0] - origin[0])
            + dir[1] * (point[1] - origin[1])
            + dir[2] * (point[2] - origin[2]));
    let c: f32 = (point[0] - origin[0]) * (point[0] - origin[0])
        + (point[1] - origin[1]) * (point[1] - origin[1])
        + (point[2] - origin[2]) * (point[2] - origin[2])
        - radius * radius;

    let d: f32 = b * b - 4.0 * c;
    if d > 0.0 {
        let t = ((-(b as f64) + (d as f64).sqrt()) / 2.0) as f32;
        VectorMA(point, t, dir, &mut intersections[0]);
        let t = ((-(b as f64) - (d as f64).sqrt()) / 2.0) as f32;
        VectorMA(point, t, dir, &mut intersections[1]);
        2
    } else if d == 0.0 {
        let t = (-(b as f64) / 2.0) as f32;
        VectorMA(point, t, dir, &mut intersections[0]);
        1
    } else {
        0
    }
}

/// `void ScorePlum( gentity_t *ent, vec3_t origin, int score )` (g_combat.c:416).
///
/// Spawns a single-client `EV_SCOREPLUM` temp event at `origin` showing `score` to `ent`'s
/// own client. No oracle (`G_TempEntity` + the global entity pool/`level`).
///
/// # Safety
/// `ent` must be a valid entity; `G_TempEntity` may return an over-budget slot exactly as in
/// the C.
pub unsafe fn ScorePlum(ent: *mut gentity_t, origin: &vec3_t, score: c_int) {
    let plum = G_TempEntity(origin, EV_SCOREPLUM as c_int);
    // only send this temp entity to a single client
    (*plum).r.svFlags |= SVF_SINGLECLIENT;
    (*plum).r.singleClient = (*ent).s.number;
    //
    (*plum).s.otherEntityNum = (*ent).s.number;
    (*plum).s.time = score;
}

/// `qboolean g_dontPenalizeTeam;` (g_cmds.c) — module global, declared `extern` in
/// g_combat.c right above [`AddScore`]. When set, [`AddScore`] skips adding the score to
/// the team total (so a player whose score change should not affect the team — e.g. an
/// admin/cheat adjustment — is not double-counted). Owned here as a file-static until its
/// real home (g_cmds.c) lands.
pub static mut g_dontPenalizeTeam: qboolean = QFALSE;

/// `void AddScore( gentity_t *ent, vec3_t origin, int score )` (g_combat.c:436).
///
/// Adds score to both the client and his team. No-ops for non-clients and during pre-match
/// warmup; in team games adds to the team total unless [`g_dontPenalizeTeam`] is set, then
/// recomputes the rankings via [`CalculateRanks`].
///
/// No oracle (void state-mutator: bumps `PERS_SCORE`/`teamScores` then calls
/// `CalculateRanks`, which walks the global client/level state).
///
/// # Safety
/// `ent` must be a valid entity; `origin` is currently unused (the score-plum call is
/// commented out in the original, mirrored here).
pub unsafe fn AddScore(ent: *mut gentity_t, _origin: &vec3_t, score: c_int) {
    /*
    if (g_gametype.integer == GT_SIEGE)
    { //no scoring in this gametype at all.
        return;
    }
    */

    if (*ent).client.is_null() {
        return;
    }
    // no scoring during pre-match warmup
    if (*addr_of!(level)).warmupTime != 0 {
        return;
    }
    // show score plum
    //ScorePlum(ent, origin, score);
    //
    (*(*ent).client).ps.persistant[PERS_SCORE as usize] += score;
    if (*addr_of!(g_gametype)).integer == GT_TEAM && *addr_of!(g_dontPenalizeTeam) == QFALSE {
        (*addr_of_mut!(level)).teamScores
            [(*(*ent).client).ps.persistant[PERS_TEAM as usize] as usize] += score;
    }
    CalculateRanks();
}

/// `void TossClientWeapon(gentity_t *self, vec3_t direction, float speed)`
/// (g_combat.c:467).
///
/// rww - Toss the weapon away from the player in the specified direction.
///
/// No oracle: entity-state side-effecting — derefs `self->client->ps`, spawns a
/// launched item via `LaunchItem` and posts `EV_NOAMMO` through `G_AddEvent`.
///
/// # Safety
/// `self_` must be a valid client entity (`self_->client` non-NULL).
pub unsafe fn TossClientWeapon(self_: *mut gentity_t, direction: &vec3_t, speed: f32) {
    let mut vel: vec3_t = [0.0; 3];
    let item: *mut gitem_t;
    let launched: *mut gentity_t;
    let weapon = (*self_).s.weapon;
    let ammoSub: c_int;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        //no dropping weaps
        return;
    }

    if weapon <= WP_BRYAR_PISTOL {
        //can't have this
        return;
    }

    if weapon == WP_EMPLACED_GUN || weapon == WP_TURRET {
        return;
    }

    // find the item type for this weapon
    item = BG_FindItemForWeapon(weapon);

    ammoSub = (*(*self_).client).ps.ammo[weaponData[weapon as usize].ammoIndex as usize]
        - bg_itemlist[BG_GetItemIndexByTag(weapon, IT_WEAPON) as usize].quantity;

    if ammoSub < 0 {
        let mut ammoQuan = (*item).quantity;
        ammoQuan -= -ammoSub;

        if ammoQuan <= 0 {
            //no ammo
            return;
        }
    }

    vel[0] = direction[0] * speed;
    vel[1] = direction[1] * speed;
    vel[2] = direction[2] * speed;

    launched = LaunchItem(item, &(*(*self_).client).ps.origin, &vel);

    (*launched).s.generic1 = (*self_).s.number;
    (*launched).s.powerups = level.time + 1500;

    (*launched).count = bg_itemlist[BG_GetItemIndexByTag(weapon, IT_WEAPON) as usize].quantity;

    (*(*self_).client).ps.ammo[weaponData[weapon as usize].ammoIndex as usize] -=
        bg_itemlist[BG_GetItemIndexByTag(weapon, IT_WEAPON) as usize].quantity;

    if (*(*self_).client).ps.ammo[weaponData[weapon as usize].ammoIndex as usize] < 0 {
        (*launched).count -=
            -(*(*self_).client).ps.ammo[weaponData[weapon as usize].ammoIndex as usize];
        (*(*self_).client).ps.ammo[weaponData[weapon as usize].ammoIndex as usize] = 0;
    }

    if ((*(*self_).client).ps.ammo[weaponData[weapon as usize].ammoIndex as usize] < 1
        && weapon != WP_DET_PACK)
        || (weapon != WP_THERMAL && weapon != WP_DET_PACK && weapon != WP_TRIP_MINE)
    {
        let mut i = 0;
        let mut weap = -1;

        (*(*self_).client).ps.stats[STAT_WEAPONS as usize] &= !(1 << weapon);

        while i < WP_NUM_WEAPONS {
            if ((*(*self_).client).ps.stats[STAT_WEAPONS as usize] & (1 << i)) != 0 && i != WP_NONE {
                //this one's good
                weap = i;
                break;
            }
            i += 1;
        }

        if weap != -1 {
            (*self_).s.weapon = weap;
            (*(*self_).client).ps.weapon = weap;
        } else {
            (*self_).s.weapon = 0;
            (*(*self_).client).ps.weapon = 0;
        }

        G_AddEvent(self_, EV_NOAMMO, weapon);
    }
}

/// `void TossClientItems( gentity_t *self )` (g_combat.c:567).
///
/// Toss the weapon and powerups for the killed player.
///
/// No oracle: entity-state side-effecting — broadcasts `EV_DESTROY_WEAPON_MODEL`
/// via `G_TempEntity` and drops the weapon + powerups through `Drop_Item`.
///
/// # Safety
/// `self_` must be a valid client entity (`self_->client` non-NULL).
pub unsafe fn TossClientItems(self_: *mut gentity_t) {
    let mut item: *mut gitem_t;
    let mut weapon: c_int;
    let mut angle: f32;
    let mut i: c_int;
    let mut drop: *mut gentity_t;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        //just don't drop anything then
        return;
    }

    // drop the weapon if not a gauntlet or machinegun
    weapon = (*self_).s.weapon;

    // make a special check to see if they are changing to a new
    // weapon that isn't the mg or gauntlet.  Without this, a client
    // can pick up a weapon, be killed, and not drop the weapon because
    // their weapon change hasn't completed yet and they are still holding the MG.
    if weapon == WP_BRYAR_PISTOL {
        if (*(*self_).client).ps.weaponstate == WEAPON_DROPPING {
            weapon = (*(*self_).client).pers.cmd.weapon as c_int;
        }
        if ((*(*self_).client).ps.stats[STAT_WEAPONS as usize] & (1 << weapon)) == 0 {
            weapon = WP_NONE;
        }
    }

    (*self_).s.bolt2 = weapon;

    if weapon > WP_BRYAR_PISTOL
        && weapon != WP_EMPLACED_GUN
        && weapon != WP_TURRET
        && (*(*self_).client).ps.ammo[weaponData[weapon as usize].ammoIndex as usize] != 0
    {
        // find the item type for this weapon
        item = BG_FindItemForWeapon(weapon);

        // tell all clients to remove the weapon model on this guy until he respawns
        let te = G_TempEntity(&vec3_origin, EV_DESTROY_WEAPON_MODEL);
        (*te).r.svFlags |= SVF_BROADCAST;
        (*te).s.eventParm = (*self_).s.number;

        // spawn the item
        Drop_Item(self_, item, 0.0);
    }

    // drop all the powerups if not in teamplay
    if (*addr_of!(g_gametype)).integer != GT_TEAM && (*addr_of!(g_gametype)).integer != GT_SIEGE {
        angle = 45.0;
        i = 1;
        while i < PW_NUM_POWERUPS {
            if (*(*self_).client).ps.powerups[i as usize] > level.time {
                item = BG_FindItemForPowerup(i);
                if item.is_null() {
                    i += 1;
                    continue;
                }
                drop = Drop_Item(self_, item, angle);
                // decide how many seconds it has left
                (*drop).count = ((*(*self_).client).ps.powerups[i as usize] - level.time) / 1000;
                if (*drop).count < 1 {
                    (*drop).count = 1;
                }
                angle += 45.0;
            }
            i += 1;
        }
    }
}

/// `void LookAtKiller( gentity_t *self, gentity_t *inflictor, gentity_t *attacker )`
/// (g_combat.c:642).
///
/// Points the dead client's view (`STAT_DEAD_YAW`) toward the attacker, else the inflictor,
/// else leaves it at the current yaw. The local `angles` is computed exactly as in the C
/// even though it is never consumed (vestigial). No oracle (writes only `client->ps.stats`,
/// driven entirely by `vectoyaw`, which is already parity-tested).
///
/// # Safety
/// `self_` must be a valid client entity; `inflictor`/`attacker` may be NULL.
pub unsafe fn LookAtKiller(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
) {
    let mut dir: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];

    if !attacker.is_null() && attacker != self_ {
        VectorSubtract(&(*attacker).s.pos.trBase, &(*self_).s.pos.trBase, &mut dir);
    } else if !inflictor.is_null() && inflictor != self_ {
        VectorSubtract(&(*inflictor).s.pos.trBase, &(*self_).s.pos.trBase, &mut dir);
    } else {
        (*(*self_).client).ps.stats[STAT_DEAD_YAW as usize] =
            (*self_).s.angles[YAW as usize] as c_int;
        return;
    }

    (*(*self_).client).ps.stats[STAT_DEAD_YAW as usize] = vectoyaw(&dir) as c_int;

    angles[YAW as usize] = vectoyaw(&dir);
    angles[PITCH as usize] = 0.0;
    angles[ROLL as usize] = 0.0;
    let _ = angles;
}

/// `void GibEntity( gentity_t *self, int killer )` (g_combat.c:667).
///
/// Fires the `EV_GIB_PLAYER` event for `killer` and turns the entity into a non-damageable
/// invisible non-solid. No oracle (`G_AddEvent` + global pool).
///
/// # Safety
/// `self` must be a valid entity.
pub unsafe fn GibEntity(self_: *mut gentity_t, killer: c_int) {
    G_AddEvent(self_, EV_GIB_PLAYER as c_int, killer);
    (*self_).takedamage = QFALSE;
    (*self_).s.eType = ET_INVISIBLE;
    (*self_).r.contents = 0;
}

/// `void BodyRid( gentity_t *ent )` (g_combat.c:674).
///
/// Corpse `think`: unlink the entity and drop its physics flag. No oracle (`trap_UnlinkEntity`).
///
/// # Safety
/// `ent` must be a valid entity; matches the engine `think` fn-pointer ABI.
pub unsafe extern "C" fn BodyRid(ent: *mut gentity_t) {
    trap::UnlinkEntity(ent);
    (*ent).physicsObject = QFALSE;
}

/// DEVIATION: `g_noPDuelCheck` is a `g_main.c` global (`g_main.c:1740`) that `player_die`
/// reads/writes but which has not yet been ported into `g_main.rs`. To keep this port
/// in-file, it lives here as a file-local mirror; when `g_main.rs` ports it, this should be
/// removed and re-imported from `g_main`. `g_dontFrickinCheck`/`gDoSlowMoDuel`/`g_endPDuel`/
/// `gSlowMoDuelTime` already exist in `g_main.rs` and are imported, not mirrored.
///
/// `qboolean g_noPDuelCheck = qfalse;` (g_main.c:1740) — suppresses the power-duel /
/// obituary checks while a forced power-duel state-change is in progress.
static mut g_noPDuelCheck: qboolean = QFALSE;

/// `static int i;` inside `player_die` (g_combat.c:2735) — the globally-cycled death-event
/// index (`EV_DEATH1 + i`, wrapped `% 3`). A C function-local `static` persists across
/// calls; in Rust it is hoisted to a file-static so the same state is shared between
/// `player_die` invocations.
static mut PLAYER_DIE_DEATH_ANIM_I: c_int = 0;

/// `char *modNames[MOD_MAX]` (g_combat.c:754) — obituary string table, indexed by
/// `meansOfDeath`. Used by `player_die` to log the kill. `c"..."` literals give NUL-terminated
/// `*const c_char` mirroring the C `char*[]`.
pub static mut modNames: [*const c_char; MOD_MAX as usize] = [
    c"MOD_UNKNOWN".as_ptr(),
    c"MOD_STUN_BATON".as_ptr(),
    c"MOD_MELEE".as_ptr(),
    c"MOD_SABER".as_ptr(),
    c"MOD_BRYAR_PISTOL".as_ptr(),
    c"MOD_BRYAR_PISTOL_ALT".as_ptr(),
    c"MOD_BLASTER".as_ptr(),
    c"MOD_TURBLAST".as_ptr(),
    c"MOD_DISRUPTOR".as_ptr(),
    c"MOD_DISRUPTOR_SPLASH".as_ptr(),
    c"MOD_DISRUPTOR_SNIPER".as_ptr(),
    c"MOD_BOWCASTER".as_ptr(),
    c"MOD_REPEATER".as_ptr(),
    c"MOD_REPEATER_ALT".as_ptr(),
    c"MOD_REPEATER_ALT_SPLASH".as_ptr(),
    c"MOD_DEMP2".as_ptr(),
    c"MOD_DEMP2_ALT".as_ptr(),
    c"MOD_FLECHETTE".as_ptr(),
    c"MOD_FLECHETTE_ALT_SPLASH".as_ptr(),
    c"MOD_ROCKET".as_ptr(),
    c"MOD_ROCKET_SPLASH".as_ptr(),
    c"MOD_ROCKET_HOMING".as_ptr(),
    c"MOD_ROCKET_HOMING_SPLASH".as_ptr(),
    c"MOD_THERMAL".as_ptr(),
    c"MOD_THERMAL_SPLASH".as_ptr(),
    c"MOD_TRIP_MINE_SPLASH".as_ptr(),
    c"MOD_TIMED_MINE_SPLASH".as_ptr(),
    c"MOD_DET_PACK_SPLASH".as_ptr(),
    c"MOD_VEHICLE".as_ptr(),
    c"MOD_CONC".as_ptr(),
    c"MOD_CONC_ALT".as_ptr(),
    c"MOD_FORCE_DARK".as_ptr(),
    c"MOD_SENTRY".as_ptr(),
    c"MOD_WATER".as_ptr(),
    c"MOD_SLIME".as_ptr(),
    c"MOD_LAVA".as_ptr(),
    c"MOD_CRUSH".as_ptr(),
    c"MOD_TELEFRAG".as_ptr(),
    c"MOD_FALLING".as_ptr(),
    c"MOD_SUICIDE".as_ptr(),
    c"MOD_TARGET_LASER".as_ptr(),
    c"MOD_TRIGGER_HURT".as_ptr(),
    // C declares `modNames[MOD_MAX]` and supplies only these 42 positional string
    // initializers (the list is unchanged Xbox->PC, ending at the "MOD_TRIGGER_HURT"
    // literal). On retail PC MOD_MAX is 45 (the new MOD_COLLISION/MOD_VEH_EXPLOSION
    // shifted the later enumerators), so the trailing 3 slots are zero-filled to NULL.
    // Faithful quirk: the 42 string literals fill indices 0..41 by position, so after
    // the enum insert they no longer line up with their names past MOD_FALLING (e.g.
    // index 39 holds "MOD_SUICIDE" while MOD_COLLISION == 39) — reproduced verbatim,
    // matching PC g_combat.c.
    null(),
    null(),
    null(),
];

/*
==================
player_die
==================
*/
/// `void player_die( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage,
/// int meansOfDeath )` (g_combat.c:2073) — the player/NPC death `die` callback: ejects
/// vehicle riders, clears combat/saber state, tallies the kill (scores, team bonuses,
/// reward sounds, power-duel win conditions), logs the obituary, broadcasts `EV_OBITUARY`,
/// tosses items, plays the death anim + checks dismemberment, fires death scripts, and
/// re-points `self->die` at `body_die`.
///
/// No oracle — entirely `gentity_t`/`gclient_t`/`level` global mutation plus `trap_*`
/// side effects and a `die` fn-pointer assignment; nothing computable is returned.
///
/// The C JediMaster / `ICARUS_FreeEnt` / `G_LogWeapon*` blocks are commented out in the
/// original and carried over as comments. The `EF_LOCKED_TO_WEAPON`/`RunEmplacedWeapon`
/// and `if (0) Boba_FlyStop` blocks are likewise dead in the C and preserved as comments.
///
/// # Safety
/// Engine `die` fn-pointer callback. `self` must be a valid entity with a non-NULL
/// `client`; `inflictor`/`attacker` may be NULL (checked). `g_entities`/`level` must be
/// initialised.
// TODO: Remove-Xbox
pub unsafe extern "C" fn player_die(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    mut attacker: *mut gentity_t,
    damage: c_int,
    meansOfDeath: c_int,
) {
    let ent: *mut gentity_t;
    let anim: c_int;
    let contents: c_int;
    let mut killer: c_int;
    let mut i: c_int;
    let mut killerName: *const c_char;
    let obit: *const c_char;
    let wasJediMaster: qboolean = QFALSE;
    let sPMType: c_int;

    if (*(*self_).client).ps.pm_type == PM_DEAD {
        return;
    }

    if (*addr_of!(level)).intermissiontime != 0 {
        return;
    }

    //check player stuff
    g_dontFrickinCheck = QFALSE;

    if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
        //don't want to wait til later in the frame if this is the case
        CheckExitRules();

        if (*addr_of!(level)).intermissiontime != 0 {
            return;
        }
    }

    if (*self_).s.eType == ET_NPC
        && (*self_).s.NPC_class == CLASS_VEHICLE
        && !(*self_).m_pVehicle.is_null()
        && (*(*(*self_).m_pVehicle).m_pVehicleInfo).explosionDelay == 0
        && (!(*(*self_).m_pVehicle).m_pPilot.is_null()
            || (*(*self_).m_pVehicle).m_iNumPassengers > 0
            || !(*(*self_).m_pVehicle).m_pDroidUnit.is_null())
    {
        //kill everyone on board in the name of the attacker... if the vehicle has no death delay
        let mut murderer: *mut gentity_t = null_mut();
        let mut killEnt: *mut gentity_t;
        // C: `int i = 0;` — the initial 0 is dead (only the passenger loop below assigns
        // and reads `i`), so the binding is left for that loop to initialise.
        let mut i: c_int;

        if (*(*self_).client).ps.otherKillerTime >= (*addr_of!(level)).time {
            //use the last attacker
            murderer = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*self_).client).ps.otherKiller as usize);
            if (*murderer).inuse == QFALSE || (*murderer).client.is_null() {
                murderer = null_mut();
            } else if (*murderer).s.number >= MAX_CLIENTS as c_int
                && (*murderer).s.eType == ET_NPC
                && (*murderer).s.NPC_class == CLASS_VEHICLE
                && !(*murderer).m_pVehicle.is_null()
                && !(*(*murderer).m_pVehicle).m_pPilot.is_null()
            {
                let murderPilot = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*(*murderer).m_pVehicle).m_pPilot).s.number as usize);
                if (*murderPilot).inuse != QFALSE && !(*murderPilot).client.is_null() {
                    //give the pilot of the offending vehicle credit for the kill
                    murderer = murderPilot;
                }
            }
        } else if !attacker.is_null() && (*attacker).inuse != QFALSE && !(*attacker).client.is_null()
        {
            if (*attacker).s.number >= MAX_CLIENTS as c_int
                && (*attacker).s.eType == ET_NPC
                && (*attacker).s.NPC_class == CLASS_VEHICLE
                && !(*attacker).m_pVehicle.is_null()
                && !(*(*attacker).m_pVehicle).m_pPilot.is_null()
            {
                //set vehicles pilot's killer as murderer
                murderer = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*(*attacker).m_pVehicle).m_pPilot).s.number as usize);
                if (*murderer).inuse != QFALSE
                    && !(*murderer).client.is_null()
                    && (*(*murderer).client).ps.otherKillerTime >= (*addr_of!(level)).time
                {
                    murderer = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*murderer).client).ps.otherKiller as usize);
                    if (*murderer).inuse == QFALSE || (*murderer).client.is_null() {
                        murderer = null_mut();
                    }
                } else {
                    murderer = null_mut();
                }
            } else {
                murderer = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*attacker).s.number as usize);
            }
        } else if !(*(*self_).m_pVehicle).m_pPilot.is_null() {
            murderer = (*(*self_).m_pVehicle).m_pPilot as *mut gentity_t;
            if (*murderer).inuse == QFALSE || (*murderer).client.is_null() {
                murderer = null_mut();
            }
        }

        //no valid murderer.. just use self I guess
        if murderer.is_null() {
            murderer = self_;
        }

        if (*(*(*self_).m_pVehicle).m_pVehicleInfo).hideRider != QFALSE {
            //pilot is *inside* me, so kill him, too
            killEnt = (*(*self_).m_pVehicle).m_pPilot as *mut gentity_t;
            if !killEnt.is_null() && (*killEnt).inuse != QFALSE && !(*killEnt).client.is_null() {
                G_Damage(
                    killEnt,
                    murderer,
                    murderer,
                    null_mut(),
                    addr_of_mut!((*(*killEnt).client).ps.origin),
                    99999,
                    DAMAGE_NO_PROTECTION,
                    MOD_BLASTER,
                );
            }
            if !(*(*self_).m_pVehicle).m_pVehicleInfo.is_null() {
                //FIXME: this wile got stuck in an endless loop, that's BAD!!  This method SUCKS (not initting "i", not incrementing it or using it directly, all sorts of badness), so I'm rewriting it
                //while (i < self->m_pVehicle->m_iNumPassengers)
                let numPass = (*(*self_).m_pVehicle).m_iNumPassengers;
                i = 0;
                while i < numPass && (*(*self_).m_pVehicle).m_iNumPassengers != 0 {
                    //go through and eject the last passenger
                    killEnt = (*(*self_).m_pVehicle).m_ppPassengers
                        [((*(*self_).m_pVehicle).m_iNumPassengers - 1) as usize]
                        as *mut gentity_t;
                    if !killEnt.is_null() {
                        ((*(*(*self_).m_pVehicle).m_pVehicleInfo).Eject.unwrap())(
                            (*self_).m_pVehicle,
                            killEnt as *mut bgEntity_t,
                            QTRUE,
                        );
                        if (*killEnt).inuse != QFALSE && !(*killEnt).client.is_null() {
                            G_Damage(
                                killEnt,
                                murderer,
                                murderer,
                                null_mut(),
                                addr_of_mut!((*(*killEnt).client).ps.origin),
                                99999,
                                DAMAGE_NO_PROTECTION,
                                MOD_BLASTER,
                            );
                        }
                    }
                    i += 1;
                }
            }
        }
        killEnt = (*(*self_).m_pVehicle).m_pDroidUnit as *mut gentity_t;
        if !killEnt.is_null() && (*killEnt).inuse != QFALSE && !(*killEnt).client.is_null() {
            (*killEnt).flags &= !FL_UNDYING;
            G_Damage(
                killEnt,
                murderer,
                murderer,
                null_mut(),
                addr_of_mut!((*(*killEnt).client).ps.origin),
                99999,
                DAMAGE_NO_PROTECTION,
                MOD_BLASTER,
            );
        }
    }

    (*(*self_).client).ps.emplacedIndex = 0;

    G_BreakArm(self_, 0); //unbreak anything we have broken
    (*(*self_).client).ps.saberEntityNum = (*(*self_).client).saberStoredIndex; //in case we died while our saber was knocked away.

    (*(*self_).client).bodyGrabIndex = ENTITYNUM_NONE;
    (*(*self_).client).bodyGrabTime = 0;

    if (*(*self_).client).holdingObjectiveItem > 0 {
        //carrying a siege objective item - make sure it updates and removes itself from us now in case this is an instant death-respawn situation
        let objectiveItem =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*self_).client).holdingObjectiveItem as usize);

        if (*objectiveItem).inuse != QFALSE && (*objectiveItem).think.is_some() {
            ((*objectiveItem).think.unwrap())(objectiveItem);
        }
    }

    if ((*(*self_).client).inSpaceIndex != 0
        && (*(*self_).client).inSpaceIndex != ENTITYNUM_NONE)
        || ((*(*self_).client).ps.eFlags2 & EF2_SHIP_DEATH) != 0
    {
        (*(*self_).client).noCorpse = QTRUE;
    }

    if (*(*self_).client).NPC_class != CLASS_VEHICLE && (*(*self_).client).ps.m_iVehicleNum != 0 {
        //I'm riding a vehicle
        //tell it I'm getting off
        let veh = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*self_).client).ps.m_iVehicleNum as usize);

        if (*veh).inuse != QFALSE && !(*veh).client.is_null() && !(*veh).m_pVehicle.is_null() {
            ((*(*(*veh).m_pVehicle).m_pVehicleInfo).Eject.unwrap())(
                (*veh).m_pVehicle,
                self_ as *mut bgEntity_t,
                QTRUE,
            );

            if (*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER {
                //go into "die in ship" mode with flag
                (*(*self_).client).ps.eFlags2 |= EF2_SHIP_DEATH;

                //put me over where my vehicle exploded
                G_SetOrigin(self_, &(*(*veh).client).ps.origin);
                VectorCopy(&(*(*veh).client).ps.origin, &mut (*(*self_).client).ps.origin);
            }
        }
        //droids throw heads if they haven't yet
        match (*(*self_).client).NPC_class {
            x if x == CLASS_R2D2 => {
                if trap::G2API_GetSurfaceRenderStatus(
                    (*self_).ghoul2,
                    0,
                    c"head".as_ptr(),
                ) == 0
                {
                    let mut up: vec3_t = [0.0; 3];
                    AngleVectors(&(*self_).r.currentAngles, None, None, Some(&mut up));
                    G_PlayEffectID(
                        G_EffectIndex("chunks/r2d2head_veh"),
                        &(*self_).r.currentOrigin,
                        &up,
                    );
                }
            }
            x if x == CLASS_R5D2 => {
                if trap::G2API_GetSurfaceRenderStatus(
                    (*self_).ghoul2,
                    0,
                    c"head".as_ptr(),
                ) == 0
                {
                    let mut up: vec3_t = [0.0; 3];
                    AngleVectors(&(*self_).r.currentAngles, None, None, Some(&mut up));
                    G_PlayEffectID(
                        G_EffectIndex("chunks/r5d2head_veh"),
                        &(*self_).r.currentOrigin,
                        &up,
                    );
                }
            }
            _ => {}
        }
    }

    if !(*self_).NPC.is_null() {
        if !(*self_).client.is_null() && Jedi_WaitingAmbush(self_) != QFALSE {
            //ambushing trooper
            (*(*self_).client).noclip = QFALSE;
        }
        NPC_FreeCombatPoint((*(*self_).NPC).combatPoint, QFALSE);
        if !(*(*self_).NPC).group.is_null() {
            //lastInGroup = (self->NPC->group->numGroup < 2);
            AI_GroupMemberKilled(self_);
            AI_DeleteSelfFromGroup(self_);
        }

        if !(*(*self_).NPC).tempGoal.is_null() {
            G_FreeEntity((*(*self_).NPC).tempGoal);
            (*(*self_).NPC).tempGoal = null_mut();
        }
        /*
        if ( self->s.eFlags & EF_LOCKED_TO_WEAPON )
        {
            // dumb, just get the NPC out of the chair
    extern void RunEmplacedWeapon( gentity_t *ent, usercmd_t **ucmd );

            usercmd_t cmd, *ad_cmd;

            memset( &cmd, 0, sizeof( usercmd_t ));

            //gentity_t *old = self->owner;

            if ( self->owner )
            {
                self->owner->s.frame = self->owner->startFrame = self->owner->endFrame = 0;
                self->owner->svFlags &= ~SVF_ANIMATING;
            }

            cmd.buttons |= BUTTON_USE;
            ad_cmd = &cmd;
            RunEmplacedWeapon( self, &ad_cmd );
            //self->owner = old;
        }
        */
        //if ( self->client->NPC_class == CLASS_BOBAFETT && self->client->moveType == MT_FLYSWIM )
        if false {
            Boba_FlyStop(self_);
        }
        if (*self_).s.NPC_class == CLASS_RANCOR {
            Rancor_DropVictim(self_);
        }
    }
    if !attacker.is_null()
        && !(*attacker).NPC.is_null()
        && !(*(*attacker).NPC).group.is_null()
        && (*(*(*attacker).NPC).group).enemy == self_
    {
        (*(*(*attacker).NPC).group).enemy = null_mut();
    }

    //Cheap method until/if I decide to put fancier stuff in (e.g. sabers falling out of hand and slowly
    //holstering on death like sp)
    if (*(*self_).client).ps.weapon == WP_SABER
        && (*(*self_).client).ps.saberHolstered == 0
        && (*(*self_).client).ps.saberEntityNum != 0
    {
        if (*(*self_).client).ps.saberInFlight == QFALSE
            && (*(*self_).client).saber[0].soundOff != 0
        {
            G_Sound(self_, CHAN_AUTO, (*(*self_).client).saber[0].soundOff);
        }
        if (*(*self_).client).saber[1].soundOff != 0
            && (*(*self_).client).saber[1].model[0] != 0
        {
            G_Sound(self_, CHAN_AUTO, (*(*self_).client).saber[1].soundOff);
        }
    }

    //Use any target we had
    G_UseTargets(self_, self_);

    if (*addr_of!(g_slowmoDuelEnd)).integer != 0
        && ((*addr_of!(g_gametype)).integer == GT_DUEL
            || (*addr_of!(g_gametype)).integer == GT_POWERDUEL)
        && !attacker.is_null()
        && (*attacker).inuse != QFALSE
        && !(*attacker).client.is_null()
    {
        if gDoSlowMoDuel == QFALSE {
            gDoSlowMoDuel = QTRUE;
            gSlowMoDuelTime = (*addr_of!(level)).time;
        }
    }
    /*
    else if (self->NPC && attacker && attacker->client && attacker->s.number < MAX_CLIENTS && !gDoSlowMoDuel)
    {
        gDoSlowMoDuel = qtrue;
        gSlowMoDuelTime = level.time;
    }
    */

    //Make sure the jetpack is turned off.
    Jetpack_Off(self_);

    (*(*self_).client).ps.heldByClient = 0;
    (*(*self_).client).beingThrown = 0;
    (*(*self_).client).doingThrow = 0;

    if !inflictor.is_null()
        && !(*inflictor).activator.is_null()
        && (*inflictor).client.is_null()
        && (*attacker).client.is_null()
        && !(*(*inflictor).activator).client.is_null()
        && (*(*inflictor).activator).inuse != QFALSE
        && (*inflictor).s.weapon == WP_TURRET
    {
        attacker = (*inflictor).activator;
    }
    /*
    if (self->client && self->client->ps.isJediMaster)
    {
        wasJediMaster = qtrue;
    }
    */
    //if he was charging or anything else, kill the sound
    G_MuteSound((*self_).s.number, CHAN_WEAPON);

    BlowDetpacks(self_); //blow detpacks if they're planted

    (*(*self_).client).ps.fd.forceDeactivateAll = 1;

    if (self_ == attacker || (*attacker).client.is_null())
        && (meansOfDeath == MOD_CRUSH
            || meansOfDeath == MOD_FALLING
            || meansOfDeath == MOD_TRIGGER_HURT
            || meansOfDeath == MOD_UNKNOWN)
        && (*(*self_).client).ps.otherKillerTime > (*addr_of!(level)).time
    {
        attacker = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*self_).client).ps.otherKiller as usize);
    }

    // check for an almost capture
    CheckAlmostCapture(self_, attacker);

    (*(*self_).client).ps.pm_type = PM_DEAD;
    (*(*self_).client).ps.pm_flags &= !PMF_STUCK_TO_WALL;

    if !attacker.is_null() {
        killer = (*attacker).s.number;
        if !(*attacker).client.is_null() {
            killerName = (*(*attacker).client).pers.netname.as_ptr();
        } else {
            killerName = c"<non-client>".as_ptr();
        }
    } else {
        killer = ENTITYNUM_WORLD;
        killerName = c"<world>".as_ptr();
    }

    if killer < 0 || killer >= MAX_CLIENTS as c_int {
        killer = ENTITYNUM_WORLD;
        killerName = c"<world>".as_ptr();
    }

    if meansOfDeath < 0 || meansOfDeath as usize >= (*addr_of!(modNames)).len() {
        obit = c"<bad obituary>".as_ptr();
    } else {
        obit = (*addr_of!(modNames))[meansOfDeath as usize];
    }

    G_LogPrintf(&format!(
        "Kill: {} {} {}: {} killed {} by {}\n",
        killer,
        (*self_).s.number,
        meansOfDeath,
        Sz(killerName),
        Sz((*(*self_).client).pers.netname.as_ptr()),
        Sz(obit)
    ));

    if (*addr_of!(g_austrian)).integer != 0
        && (*addr_of!(g_gametype)).integer == GT_DUEL
        && (*addr_of!(level)).numPlayingClients >= 2
    {
        let r0 = (*(*addr_of!(level)).clients
            .add((*addr_of!(level)).sortedClients[0] as usize))
        .respawnTime;
        let r1 = (*(*addr_of!(level)).clients
            .add((*addr_of!(level)).sortedClients[1] as usize))
        .respawnTime;
        let spawnTime = if r0 > r1 { r0 } else { r1 };
        G_LogPrintf("Duel Kill Details:\n");
        G_LogPrintf(&format!("Kill Time: {}\n", (*addr_of!(level)).time - spawnTime));
        G_LogPrintf(&format!(
            "victim: {}, hits on enemy {}\n",
            Sz((*(*self_).client).pers.netname.as_ptr()),
            (*(*self_).client).ps.persistant[PERS_HITS as usize]
        ));
        if !attacker.is_null() && !(*attacker).client.is_null() {
            G_LogPrintf(&format!(
                "killer: {}, hits on enemy {}, health: {}\n",
                Sz((*(*attacker).client).pers.netname.as_ptr()),
                (*(*attacker).client).ps.persistant[PERS_HITS as usize],
                (*attacker).health
            ));
            //also - if MOD_SABER, list the animation and saber style
            if meansOfDeath == MOD_SABER {
                G_LogPrintf(&format!(
                    "killer saber style: {}, killer saber anim {}\n",
                    (*(*attacker).client).ps.fd.saberAnimLevel,
                    Sz((*addr_of!(animTable))
                        [(*(*attacker).client).ps.torsoAnim as usize]
                        .name)
                ));
            }
        }
    }

    G_LogWeaponKill(killer, meansOfDeath);
    G_LogWeaponDeath((*self_).s.number, (*self_).s.weapon);
    if !attacker.is_null() && !(*attacker).client.is_null() && (*attacker).inuse != QFALSE {
        G_LogWeaponFrag(killer, (*self_).s.number);
    }

    // broadcast the death event to everyone
    if (*self_).s.eType != ET_NPC && g_noPDuelCheck == QFALSE {
        ent = G_TempEntity(&(*self_).r.currentOrigin, EV_OBITUARY);
        (*ent).s.eventParm = meansOfDeath;
        (*ent).s.otherEntityNum = (*self_).s.number;
        (*ent).s.otherEntityNum2 = killer;
        (*ent).r.svFlags = SVF_BROADCAST; // send to everyone
                                          //		ent->s.isJediMaster = wasJediMaster;
    }

    (*self_).enemy = attacker;

    (*(*self_).client).ps.persistant[PERS_KILLED as usize] += 1;

    if self_ == attacker {
        (*(*self_).client).ps.fd.suicides += 1;
    }

    if !attacker.is_null() && !(*attacker).client.is_null() {
        (*(*attacker).client).lastkilled_client = (*self_).s.number;

        G_CheckVictoryScript(attacker);

        if attacker == self_ || OnSameTeam(self_, attacker) != QFALSE {
            if (*addr_of!(g_gametype)).integer == GT_DUEL {
                //in duel, if you kill yourself, the person you are dueling against gets a kill for it
                let mut otherClNum: c_int = -1;
                if (*addr_of!(level)).sortedClients[0] == (*self_).s.number {
                    otherClNum = (*addr_of!(level)).sortedClients[1];
                } else if (*addr_of!(level)).sortedClients[1] == (*self_).s.number {
                    otherClNum = (*addr_of!(level)).sortedClients[0];
                }

                if otherClNum >= 0
                    && otherClNum < MAX_CLIENTS as c_int
                    && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(otherClNum as usize)).inuse != QFALSE
                    && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(otherClNum as usize)).client.is_null()
                    && otherClNum != (*attacker).s.number
                {
                    AddScore(
                        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(otherClNum as usize),
                        &(*self_).r.currentOrigin,
                        1,
                    );
                } else {
                    AddScore(attacker, &(*self_).r.currentOrigin, -1);
                }
            } else {
                AddScore(attacker, &(*self_).r.currentOrigin, -1);
            }
        /*
            if (g_gametype.integer == GT_JEDIMASTER)
            {
                if (self->client && self->client->ps.isJediMaster)
                { //killed ourself so return the saber to the original position
                  //(to avoid people jumping off ledges and making the saber
                  //unreachable for 60 seconds)
                    ThrowSaberToAttacker(self, NULL);
                    self->client->ps.isJediMaster = qfalse;
                }
            }
        */
        } else {
            /*
            if (g_gametype.integer == GT_JEDIMASTER)
            {
                if ((attacker->client && attacker->client->ps.isJediMaster) ||
                    (self->client && self->client->ps.isJediMaster))
                {
                    AddScore( attacker, self->r.currentOrigin, 1 );

                    if (self->client && self->client->ps.isJediMaster)
                    {
                        ThrowSaberToAttacker(self, attacker);
                        self->client->ps.isJediMaster = qfalse;
                    }
                }
                else
                {
                    gentity_t *jmEnt = G_GetJediMaster();

                    if (jmEnt && jmEnt->client)
                    {
                        AddScore( jmEnt, self->r.currentOrigin, 1 );
                    }
                }
            }
            else
            */
            {
                AddScore(attacker, &(*self_).r.currentOrigin, 1);
            }

            if meansOfDeath == MOD_STUN_BATON {
                // play humiliation on player
                (*(*attacker).client).ps.persistant[PERS_GAUNTLET_FRAG_COUNT as usize] += 1;

                (*(*attacker).client).rewardTime = (*addr_of!(level)).time + REWARD_SPRITE_TIME;

                // also play humiliation on target
                (*(*self_).client).ps.persistant[PERS_PLAYEREVENTS as usize] ^=
                    PLAYEREVENT_GAUNTLETREWARD;
            }

            // check for two kills in a short amount of time
            // if this is close enough to the last kill, give a reward sound
            if (*addr_of!(level)).time - (*(*attacker).client).lastKillTime < CARNAGE_REWARD_TIME {
                // play excellent on player
                (*(*attacker).client).ps.persistant[PERS_EXCELLENT_COUNT as usize] += 1;

                (*(*attacker).client).rewardTime = (*addr_of!(level)).time + REWARD_SPRITE_TIME;
            }
            (*(*attacker).client).lastKillTime = (*addr_of!(level)).time;
        }
    } else {
        /*
        if (self->client && self->client->ps.isJediMaster)
        { //killed ourself so return the saber to the original position
          //(to avoid people jumping off ledges and making the saber
          //unreachable for 60 seconds)
            ThrowSaberToAttacker(self, NULL);
            self->client->ps.isJediMaster = qfalse;
        }
        */
        if (*addr_of!(g_gametype)).integer == GT_DUEL {
            //in duel, if you kill yourself, the person you are dueling against gets a kill for it
            let mut otherClNum: c_int = -1;
            if (*addr_of!(level)).sortedClients[0] == (*self_).s.number {
                otherClNum = (*addr_of!(level)).sortedClients[1];
            } else if (*addr_of!(level)).sortedClients[1] == (*self_).s.number {
                otherClNum = (*addr_of!(level)).sortedClients[0];
            }

            if otherClNum >= 0
                && otherClNum < MAX_CLIENTS as c_int
                && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(otherClNum as usize)).inuse != QFALSE
                && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(otherClNum as usize)).client.is_null()
                && otherClNum != (*self_).s.number
            {
                AddScore(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(otherClNum as usize),
                    &(*self_).r.currentOrigin,
                    1,
                );
            } else {
                AddScore(self_, &(*self_).r.currentOrigin, -1);
            }
        } else {
            AddScore(self_, &(*self_).r.currentOrigin, -1);
        }
    }

    // Add team bonuses
    Team_FragBonuses(self_, inflictor, attacker);

    // if I committed suicide, the flag does not fall, it returns.
    if meansOfDeath == MOD_SUICIDE {
        if (*(*self_).client).ps.powerups[PW_NEUTRALFLAG as usize] != 0 {
            // only happens in One Flag CTF
            Team_ReturnFlag(TEAM_FREE);
            (*(*self_).client).ps.powerups[PW_NEUTRALFLAG as usize] = 0;
        } else if (*(*self_).client).ps.powerups[PW_REDFLAG as usize] != 0 {
            // only happens in standard CTF
            Team_ReturnFlag(TEAM_RED);
            (*(*self_).client).ps.powerups[PW_REDFLAG as usize] = 0;
        } else if (*(*self_).client).ps.powerups[PW_BLUEFLAG as usize] != 0 {
            // only happens in standard CTF
            Team_ReturnFlag(TEAM_BLUE);
            (*(*self_).client).ps.powerups[PW_BLUEFLAG as usize] = 0;
        }
    }

    // if client is in a nodrop area, don't drop anything (but return CTF flags!)
    contents = trap::PointContents(&(*self_).r.currentOrigin, -1);
    if (contents & CONTENTS_NODROP) == 0 && (*(*self_).client).ps.fallingToDeath == 0 {
        if (*self_).s.eType != ET_NPC {
            TossClientItems(self_);
        }
    } else if (*(*self_).client).ps.powerups[PW_NEUTRALFLAG as usize] != 0 {
        // only happens in One Flag CTF
        Team_ReturnFlag(TEAM_FREE);
    } else if (*(*self_).client).ps.powerups[PW_REDFLAG as usize] != 0 {
        // only happens in standard CTF
        Team_ReturnFlag(TEAM_RED);
    } else if (*(*self_).client).ps.powerups[PW_BLUEFLAG as usize] != 0 {
        // only happens in standard CTF
        Team_ReturnFlag(TEAM_BLUE);
    }

    if MOD_TEAM_CHANGE == meansOfDeath {
        // Give them back a point since they didn't really die.
        AddScore(self_, &(*self_).r.currentOrigin, 1);
    } else {
        Cmd_Score_f(self_); // show scores
    }

    // send updated scores to any clients that are following this one,
    // or they would get stale scoreboards
    i = 0;
    while i < (*addr_of!(level)).maxclients {
        let client: *mut gclient_t = (*addr_of!(level)).clients.add(i as usize);
        if (*client).pers.connected != CON_CONNECTED {
            i += 1;
            continue;
        }
        if (*client).sess.sessionTeam != TEAM_SPECTATOR {
            i += 1;
            continue;
        }
        if (*client).sess.spectatorClient == (*self_).s.number {
            Cmd_Score_f((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize));
        }
        i += 1;
    }

    (*self_).takedamage = QTRUE; // can still be gibbed

    (*self_).s.weapon = WP_NONE;
    (*self_).s.powerups = 0;
    if (*self_).s.eType != ET_NPC {
        //handled differently for NPCs
        (*self_).r.contents = CONTENTS_CORPSE;
    }
    (*(*self_).client).ps.zoomMode = 0; // Turn off zooming when we die

    //rww - 07/19/02 - I removed this because it isn't working and it's ugly (for people on the outside)
    /*
    self->s.angles[0] = 0;
    self->s.angles[2] = 0;
    LookAtKiller (self, inflictor, attacker);

    VectorCopy( self->s.angles, self->client->ps.viewangles );
    */

    (*self_).s.loopSound = 0;
    (*self_).s.loopIsSoundset = QFALSE;

    if (*self_).s.eType != ET_NPC {
        //handled differently for NPCs
        (*self_).r.maxs[2] = -8.0;
    }

    // don't allow respawn until the death anim is done
    // g_forcerespawn may force spawning at some later time
    (*(*self_).client).respawnTime = (*addr_of!(level)).time + 1700;

    // remove powerups
    (*(*self_).client).ps.powerups = [0; crate::codemp::game::q_shared_h::MAX_POWERUPS];

    // NOTENOTE No gib deaths right now, this is star wars.
    /*
    // never gib in a nodrop
    if ( (self->health <= GIB_HEALTH && !(contents & CONTENTS_NODROP) && g_blood.integer) || meansOfDeath == MOD_SUICIDE)
    {
        // gib death
        GibEntity( self, killer );
    }
    else
    */
    {
        // normal death

        // (static int i;) — globally cycled death-anim index; mirrored by the file-static
        // `PLAYER_DIE_DEATH_ANIM_I` so it persists across calls like the C `static`.

        anim = G_PickDeathAnim(self_, &(*self_).pos1, damage, meansOfDeath, HL_NONE);

        if anim >= 1 {
            //Some droids don't have death anims
            // for the no-blood option, we need to prevent the health
            // from going to gib level
            if (*self_).health <= GIB_HEALTH {
                (*self_).health = GIB_HEALTH + 1;
            }

            (*(*self_).client).respawnTime = (*addr_of!(level)).time + 1000; //((self->client->animations[anim].numFrames*40)/(50.0f / self->client->animations[anim].frameLerp))+300;

            sPMType = (*(*self_).client).ps.pm_type;
            (*(*self_).client).ps.pm_type = PM_NORMAL; //don't want pm type interfering with our setanim calls.

            if (*self_).inuse != QFALSE {
                //not disconnecting
                G_SetAnim(
                    self_,
                    null_mut(),
                    SETANIM_BOTH,
                    anim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
                    0,
                );
            }

            (*(*self_).client).ps.pm_type = sPMType;

            if meansOfDeath == MOD_SABER
                || (meansOfDeath == MOD_MELEE && G_HeavyMelee(attacker) != QFALSE)
            {
                //saber or heavy melee (claws)
                //update the anim on the actual skeleton (so bolt point will reflect the correct position) and then check for dismem
                G_UpdateClientAnims(self_, 1.0);
                G_CheckForDismemberment(self_, attacker, &(*self_).pos1, damage, anim, QFALSE);
            }
        } else if !(*self_).NPC.is_null()
            && !(*self_).client.is_null()
            && (*(*self_).client).NPC_class != CLASS_MARK1
            && (*(*self_).client).NPC_class != CLASS_VEHICLE
        {
            //in this case if we're an NPC it's my guess that we want to get removed straight away.
            (*self_).think = Some(G_FreeEntity);
            (*self_).nextthink = (*addr_of!(level)).time;
        }

        //self->client->ps.legsAnim = anim;
        //self->client->ps.torsoAnim = anim;
        //		self->client->ps.pm_flags |= PMF_UPDATE_ANIM;		// Make sure the pmove sets up the GHOUL2 anims.

        //rww - do this on respawn, not death
        //CopyToBodyQue (self);

        //G_AddEvent( self, EV_DEATH1 + i, killer );
        if wasJediMaster != QFALSE {
            G_AddEvent(self_, EV_DEATH1 + PLAYER_DIE_DEATH_ANIM_I, 1);
        } else {
            G_AddEvent(self_, EV_DEATH1 + PLAYER_DIE_DEATH_ANIM_I, 0);
        }

        if self_ != attacker {
            //don't make NPCs want to murder you on respawn for killing yourself!
            G_DeathAlert(self_, attacker);
        }

        // the body can still be gibbed
        if (*self_).NPC.is_null() {
            //don't remove NPCs like this!
            (*self_).die = Some(body_die);
        }

        //It won't gib, it will disintegrate (because this is Star Wars).
        (*self_).takedamage = QTRUE;

        // globally cycle through the different death animations
        PLAYER_DIE_DEATH_ANIM_I = (PLAYER_DIE_DEATH_ANIM_I + 1) % 3;
    }

    if !(*self_).NPC.is_null() {
        //If an NPC, make sure we start running our scripts again- this gets set to infinite while we fall to our deaths
        (*(*self_).NPC).nextBStateThink = (*addr_of!(level)).time;
    }

    if G_ActivateBehavior(self_, BSET_DEATH) != QFALSE {
        //deathScript = qtrue;
    }

    if !(*self_).NPC.is_null() && ((*(*self_).NPC).scriptFlags & SCF_FFDEATH) != 0 {
        if G_ActivateBehavior(self_, BSET_FFDEATH) != QFALSE {
            //FIXME: should running this preclude running the normal deathscript?
            //deathScript = qtrue;
        }
        G_UseTargets2(self_, self_, (*self_).target4 as *const c_char);
    }

    /*
    if ( !deathScript && !(self->svFlags&SVF_KILLED_SELF) )
    {
        //Should no longer run scripts
        //WARNING!!! DO NOT DO THIS WHILE RUNNING A SCRIPT, ICARUS WILL CRASH!!!
        //FIXME: shouldn't ICARUS handle this internally?
        ICARUS_FreeEnt(self);
    }
    */
    //rwwFIXMEFIXME: Do this too?

    // Free up any timers we may have on us.
    TIMER_Clear2(self_);

    trap::LinkEntity(self_);

    if !(*self_).NPC.is_null() {
        (*(*self_).NPC).timeOfDeath = (*addr_of!(level)).time; //this will change - used for debouncing post-death events
    }

    // Start any necessary death fx for this entity
    DeathFX(self_);

    if (*addr_of!(g_gametype)).integer == GT_POWERDUEL && g_noPDuelCheck == QFALSE {
        //powerduel checks
        if (*(*self_).client).sess.duelTeam == DUELTEAM_LONE {
            //automatically means a win as there is only one
            G_AddPowerDuelScore(DUELTEAM_DOUBLE, 1);
            G_AddPowerDuelLoserScore(DUELTEAM_LONE, 1);
            g_endPDuel = QTRUE;
        } else if (*(*self_).client).sess.duelTeam == DUELTEAM_DOUBLE {
            let mut i: c_int = 0;
            let mut check: *mut gentity_t;
            let mut heLives: qboolean = QFALSE;

            while i < MAX_CLIENTS as c_int {
                check = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
                if (*check).inuse != QFALSE
                    && !(*check).client.is_null()
                    && (*check).s.number != (*self_).s.number
                    && (*(*check).client).pers.connected == CON_CONNECTED
                    && (*(*check).client).iAmALoser == QFALSE
                    && (*(*check).client).ps.stats[STAT_HEALTH as usize] > 0
                    && (*(*check).client).sess.sessionTeam != TEAM_SPECTATOR
                    && (*(*check).client).sess.duelTeam == DUELTEAM_DOUBLE
                {
                    //still an active living paired duelist so it's not over yet.
                    heLives = QTRUE;
                    break;
                }
                i += 1;
            }

            if heLives == QFALSE {
                //they're all dead, give the lone duelist the win.
                G_AddPowerDuelScore(DUELTEAM_LONE, 1);
                G_AddPowerDuelLoserScore(DUELTEAM_DOUBLE, 1);
                g_endPDuel = QTRUE;
            }
        }
    }
}

/// `void body_die( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage,
/// int meansOfDeath )` (g_combat.c:685).
///
/// Corpse `die` handler. NPCs that have finished their death anim and died to a
/// non-standard-weapon means are freed; otherwise (player corpses) the disintegration effect
/// is applied unless already disintegrating or freshly respawned. No gibbing ("this is star
/// wars"). No oracle (global `level`, `die`/`think` fn-pointers, entity pool).
///
/// # Safety
/// `self_` must be a valid entity; `inflictor`/`attacker` may be NULL (unused here). Matches
/// the engine `die` fn-pointer ABI.
pub unsafe extern "C" fn body_die(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    means_of_death: c_int,
) {
    // NOTENOTE No gibbing right now, this is star wars.
    let mut do_disint: qboolean = QFALSE;

    if (*self_).s.eType == ET_NPC {
        // well, just rem it then, so long as it's done with its death anim and it's not a
        // standard weapon.
        if !(*self_).client.is_null()
            && (*(*self_).client).ps.torsoTimer <= 0
            && (means_of_death == MOD_UNKNOWN
                || means_of_death == MOD_WATER
                || means_of_death == MOD_SLIME
                || means_of_death == MOD_LAVA
                || means_of_death == MOD_CRUSH
                || means_of_death == MOD_TELEFRAG
                || means_of_death == MOD_FALLING
                || means_of_death == MOD_SUICIDE
                || means_of_death == MOD_TARGET_LASER
                || means_of_death == MOD_TRIGGER_HURT)
        {
            (*self_).think = Some(G_FreeEntity);
            (*self_).nextthink = (*addr_of!(level)).time;
        }
        return;
    }

    if (*self_).health < (GIB_HEALTH + 1) {
        (*self_).health = GIB_HEALTH + 1;

        if !(*self_).client.is_null()
            && ((*addr_of!(level)).time - (*(*self_).client).respawnTime) < 2000
        {
            do_disint = QFALSE;
        } else {
            do_disint = QTRUE;
        }
    }

    if !(*self_).client.is_null() && ((*(*self_).client).ps.eFlags & EF_DISINTEGRATION) != 0 {
        return;
    } else if ((*self_).s.eFlags & EF_DISINTEGRATION) != 0 {
        return;
    }

    if do_disint == QTRUE {
        if !(*self_).client.is_null() {
            (*(*self_).client).ps.eFlags |= EF_DISINTEGRATION;
            let origin = (*(*self_).client).ps.origin;
            VectorCopy(&origin, &mut (*(*self_).client).ps.lastHitLoc);
        } else {
            (*self_).s.eFlags |= EF_DISINTEGRATION;
            let current_origin = (*self_).r.currentOrigin;
            VectorCopy(&current_origin, &mut (*self_).s.origin2);

            // since it's the corpse entity, tell it to "remove" itself
            (*self_).think = Some(BodyRid);
            (*self_).nextthink = (*addr_of!(level)).time + 1000;
        }
    }
}

/// `void ObjectDie(gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int meansOfDeath)`
/// (g_combat.c:21) — the generic `die` callback for scripted/breakable map objects:
/// fire the object's targets (if any), then free it. `inflictor`/`damage`/`meansOfDeath`
/// are unused (the standard `die` ABI shape).
///
/// No oracle (`G_UseTargets` walks the entity pool + `G_FreeEntity` mutates globals).
///
/// # Safety
/// Engine `die` fn-pointer callback. `self`/`attacker` must be valid entities; the
/// other params follow the `die` ABI.
pub unsafe extern "C" fn ObjectDie(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _means_of_death: c_int,
) {
    if !(*self_).target.is_null() {
        G_UseTargets(self_, attacker);
    }

    //remove my script_targetname
    G_FreeEntity(self_);
}

/// `void ExplodeDeath( gentity_t *self )` (g_combat.c:368) — the exploder entity's
/// death `think`: stop taking damage (kills chain-reaction runaway loops), silence its
/// loop sound, re-base its trajectory at the current origin, then deal splash damage
/// (when `splashDamage`/`splashRadius` are set, crediting `parent` if present) and
/// finally `ObjectDie`. The client-side `CG_SurfaceExplosion`/`G_PlayEffect`/`G_Sound`
/// blocks are commented out in the C original and stay dropped.
///
/// No oracle (`G_RadiusDamage`/`ObjectDie` mutate entity globals).
///
/// # Safety
/// Engine `think` fn-pointer callback. `self` must be a valid entity; `parent` may be NULL.
pub unsafe extern "C" fn ExplodeDeath(self_: *mut gentity_t) {
    let mut forward: vec3_t = [0.0; 3];

    (*self_).takedamage = QFALSE; //stop chain reaction runaway loops

    (*self_).s.loopSound = 0;
    (*self_).s.loopIsSoundset = QFALSE;

    VectorCopy(&(*self_).r.currentOrigin, &mut (*self_).s.pos.trBase);

    //	tent = G_TempEntity( self->s.origin, EV_FX_EXPLOSION );
    AngleVectors(&(*self_).s.angles, Some(&mut forward), None, None);

    /*
        if ( self->fxID > 0 )
        {
            G_PlayEffect( self->fxID, self->r.currentOrigin, forward );
        }
        else
        */

    {
        //		CG_SurfaceExplosion( self->r.currentOrigin, forward, 20.0f, 12.0f, ((self->spawnflags&4)==qfalse) );	//FIXME: This needs to be consistent to all exploders!
        //		G_Sound(self, self->sounds );
    }

    if (*self_).splashDamage > 0 && (*self_).splashRadius > 0 {
        let mut attacker = self_;
        if !(*self_).parent.is_null() {
            attacker = (*self_).parent;
        }
        G_RadiusDamage(
            &(*self_).r.currentOrigin,
            attacker,
            (*self_).splashDamage as f32,
            (*self_).splashRadius as f32,
            attacker,
            std::ptr::null_mut(),
            MOD_UNKNOWN,
        );
    }

    ObjectDie(self_, self_, self_, 20, 0);
}

/*
-------------------------
G_AlertTeam
-------------------------
*/
/// `void G_AlertTeam( gentity_t *victim, gentity_t *attacker, float radius, float soundDist )`
/// (g_combat.c:1769) — when an NPC is hurt/killed, wakes its same-team NPC neighbours within
/// `radius` and sets `attacker` as their enemy (if they're not already mad at someone, alive,
/// flagged to look for enemies, not ignoring alerts, group-capable, and within sound range or
/// FOV+LOS).
///
/// No oracle — pure entity-state graph (`gentity_t`/`gNPC_t`/`gclient_t`) plus `trap_*`
/// (`trap_EntitiesInBox`/`trap_InPVS`) and the `G_SetEnemy` side effect; nothing computable
/// is returned.
///
/// # Safety
/// `victim` must be a valid `gentity_t`; `attacker` may be NULL (checked, along with its
/// `client`). `g_entities` must be initialised. Dereferences each in-box ent's
/// `client`/`NPC` (both checked) graph.
pub unsafe fn G_AlertTeam(
    victim: *mut gentity_t,
    attacker: *mut gentity_t,
    radius: f32,
    soundDist: f32,
) {
    let mut radiusEnts: [c_int; 128] = [0; 128];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let numEnts: c_int;
    let sndDistSq: f32 = soundDist * soundDist;

    if attacker.is_null() || (*attacker).client.is_null() {
        return;
    }

    //Setup the bbox to search in
    for i in 0..3 {
        mins[i] = (*victim).r.currentOrigin[i] - radius;
        maxs[i] = (*victim).r.currentOrigin[i] + radius;
    }

    //Get the number of entities in a given space
    numEnts = trap::EntitiesInBox(&mins, &maxs, &mut radiusEnts);

    //Cull this list
    for i in 0..numEnts as usize {
        let check = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(radiusEnts[i] as usize);

        //Validate clients
        if (*check).client.is_null() {
            continue;
        }

        //only want NPCs
        if (*check).NPC.is_null() {
            continue;
        }

        //Don't bother if they're ignoring enemies
        //		if ( check->svFlags & SVF_IGNORE_ENEMIES )
        //			continue;

        //This NPC specifically flagged to ignore alerts
        if (*(*check).NPC).scriptFlags & SCF_IGNORE_ALERTS != 0 {
            continue;
        }

        //This NPC specifically flagged to ignore alerts
        if (*(*check).NPC).scriptFlags & SCF_LOOK_FOR_ENEMIES == 0 {
            continue;
        }

        //this ent does not participate in group AI
        if (*(*check).NPC).scriptFlags & SCF_NO_GROUPS != 0 {
            continue;
        }

        //Skip the requested avoid check if present
        if check == victim {
            continue;
        }

        //Skip the attacker
        if check == attacker {
            continue;
        }

        //Must be on the same team
        if (*(*check).client).playerTeam != (*(*victim).client).playerTeam {
            continue;
        }

        //Must be alive
        if (*check).health <= 0 {
            continue;
        }

        if (*check).enemy.is_null() {
            //only do this if they're not already mad at someone
            let distSq = DistanceSquared(&(*check).r.currentOrigin, &(*victim).r.currentOrigin);
            if distSq > 16384.0 /*128 squared*/
                && trap::InPVS(&(*victim).r.currentOrigin, &(*check).r.currentOrigin) == QFALSE
            {
                //not even potentially visible/hearable
                continue;
            }
            //NOTE: this allows sound alerts to still go through doors/PVS if the teammate is within 128 of the victim...
            if soundDist <= 0.0 || distSq > sndDistSq {
                //out of sound range
                if InFOV(
                    victim,
                    check,
                    (*(*check).NPC).stats.hfov,
                    (*(*check).NPC).stats.vfov,
                ) == QFALSE
                    || NPC_ClearLOS2(check, &(*victim).r.currentOrigin) == QFALSE
                {
                    //out of FOV or no LOS
                    continue;
                }
            }

            //FIXME: This can have a nasty cascading effect if setup wrong...
            G_SetEnemy(check, attacker);
        }
    }
}

/*
-------------------------
G_DeathAlert
-------------------------
*/

const DEATH_ALERT_RADIUS: f32 = 512.0;
const DEATH_ALERT_SOUND_RADIUS: f32 = 512.0;

/// `void G_DeathAlert( gentity_t *victim, gentity_t *attacker )` (g_combat.c:1868) — on an
/// NPC death, alerts its same-team neighbours via `G_AlertTeam` at the death radii.
///
/// No oracle — thin wrapper that only calls `G_AlertTeam` (pure entity-state/trap side
/// effects).
///
/// # Safety
/// See `G_AlertTeam`.
pub unsafe fn G_DeathAlert(victim: *mut gentity_t, attacker: *mut gentity_t) {
    //FIXME: with all the other alert stuff, do we really need this?
    G_AlertTeam(
        victim,
        attacker,
        DEATH_ALERT_RADIUS,
        DEATH_ALERT_SOUND_RADIUS,
    );
}

/// `void DeathFX( gentity_t *ent )` (g_combat.c:1882) — plays the per-NPC-class
/// death explosion effect(s) and sound when a droid/creature NPC dies. Dispatches on
/// `ent->client->NPC_class`; player/non-NPC deaths hit the `default` no-op. A direct
/// `player_die` dependency.
///
/// No oracle — pure trap side effects (`G_PlayEffectID`/`G_Sound` + effect/sound index
/// registration); nothing computable is returned.
///
/// # Safety
/// `ent` may be NULL (checked); when non-NULL its `client` (also checked) is read for
/// `NPC_class` and `r.currentOrigin`/`r.currentAngles` are read.
pub unsafe fn DeathFX(ent: *mut gentity_t) {
    let mut effect_pos: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut default_dir: vec3_t = [0.0; 3];

    if ent.is_null() || (*ent).client.is_null() {
        return;
    }

    VectorSet(&mut default_dir, 0.0, 0.0, 1.0);

    // team no longer indicates species/race.  NPC_class should be used to identify certain npc types
    match (*(*ent).client).NPC_class {
        x if x == CLASS_MOUSE => {
            VectorCopy(&(*ent).r.currentOrigin, &mut effect_pos);
            effect_pos[2] -= 20.0;
            G_PlayEffectID(G_EffectIndex("env/small_explode"), &effect_pos, &default_dir);
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/mouse/misc/death1"),
            );
        }

        x if x == CLASS_PROBE => {
            VectorCopy(&(*ent).r.currentOrigin, &mut effect_pos);
            effect_pos[2] += 50.0;
            G_PlayEffectID(
                G_EffectIndex("explosions/probeexplosion1"),
                &effect_pos,
                &default_dir,
            );
        }

        x if x == CLASS_ATST => {
            AngleVectors(&(*ent).r.currentAngles, None, Some(&mut right), None);
            VectorMA(&(*ent).r.currentOrigin, 20.0, &right, &mut effect_pos);
            effect_pos[2] += 180.0;
            G_PlayEffectID(
                G_EffectIndex("explosions/droidexplosion1"),
                &effect_pos,
                &default_dir,
            );
            let effect_pos_in = effect_pos;
            VectorMA(&effect_pos_in, -40.0, &right, &mut effect_pos);
            G_PlayEffectID(
                G_EffectIndex("explosions/droidexplosion1"),
                &effect_pos,
                &default_dir,
            );
        }

        x if x == CLASS_SEEKER || x == CLASS_REMOTE => {
            G_PlayEffectID(
                G_EffectIndex("env/small_explode"),
                &(*ent).r.currentOrigin,
                &default_dir,
            );
        }

        x if x == CLASS_GONK => {
            VectorCopy(&(*ent).r.currentOrigin, &mut effect_pos);
            effect_pos[2] -= 5.0;
            //		statusTextIndex = Q_irand( IGT_RESISTANCEISFUTILE, IGT_NAMEIS8OF12 );
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex(&format!(
                    "sound/chars/gonk/misc/death{}.wav",
                    Q_irand(1, 3)
                )),
            );
            G_PlayEffectID(G_EffectIndex("env/med_explode"), &effect_pos, &default_dir);
        }

        // should list all remaining droids here, hope I didn't miss any
        x if x == CLASS_R2D2 => {
            VectorCopy(&(*ent).r.currentOrigin, &mut effect_pos);
            effect_pos[2] -= 10.0;
            G_PlayEffectID(G_EffectIndex("env/med_explode"), &effect_pos, &default_dir);
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/mark2/misc/mark2_explo"),
            );
        }

        x if x == CLASS_PROTOCOL || x == CLASS_R5D2 => {
            VectorCopy(&(*ent).r.currentOrigin, &mut effect_pos);
            effect_pos[2] -= 10.0;
            G_PlayEffectID(G_EffectIndex("env/med_explode"), &effect_pos, &default_dir);
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/mark2/misc/mark2_explo"),
            );
        }

        x if x == CLASS_MARK2 => {
            VectorCopy(&(*ent).r.currentOrigin, &mut effect_pos);
            effect_pos[2] -= 15.0;
            G_PlayEffectID(
                G_EffectIndex("explosions/droidexplosion1"),
                &effect_pos,
                &default_dir,
            );
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/mark2/misc/mark2_explo"),
            );
        }

        x if x == CLASS_INTERROGATOR => {
            VectorCopy(&(*ent).r.currentOrigin, &mut effect_pos);
            effect_pos[2] -= 15.0;
            G_PlayEffectID(
                G_EffectIndex("explosions/droidexplosion1"),
                &effect_pos,
                &default_dir,
            );
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/interrogator/misc/int_droid_explo"),
            );
        }

        x if x == CLASS_MARK1 => {
            AngleVectors(&(*ent).r.currentAngles, None, Some(&mut right), None);
            VectorMA(&(*ent).r.currentOrigin, 10.0, &right, &mut effect_pos);
            effect_pos[2] -= 15.0;
            G_PlayEffectID(
                G_EffectIndex("explosions/droidexplosion1"),
                &effect_pos,
                &default_dir,
            );
            let effect_pos_in = effect_pos;
            VectorMA(&effect_pos_in, -20.0, &right, &mut effect_pos);
            G_PlayEffectID(
                G_EffectIndex("explosions/droidexplosion1"),
                &effect_pos,
                &default_dir,
            );
            let effect_pos_in = effect_pos;
            VectorMA(&effect_pos_in, -20.0, &right, &mut effect_pos);
            G_PlayEffectID(
                G_EffectIndex("explosions/droidexplosion1"),
                &effect_pos,
                &default_dir,
            );
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/mark1/misc/mark1_explo"),
            );
        }

        x if x == CLASS_SENTRY => {
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/sentry/misc/sentry_explo"),
            );
            VectorCopy(&(*ent).r.currentOrigin, &mut effect_pos);
            G_PlayEffectID(G_EffectIndex("env/med_explode"), &effect_pos, &default_dir);
        }

        _ => {}
    }
}

/// No oracle — pure entity/NPC state mutation: it activates the victory behavior
/// set, mutates `NPC->blockedSpeechDebounceTime`/`greetingDebounceTime`, sets
/// `self->wait`, and drives `TIMER_Set`/`Q_irand`. No computable return value to
/// parity-test. (The two `G_AddVoiceEvent` calls are commented out in the C
/// original, so they are carried over as comments here too.)
pub unsafe fn G_CheckVictoryScript(self_: *mut gentity_t) {
    if G_ActivateBehavior(self_, BSET_VICTORY) == QFALSE {
        if !(*self_).NPC.is_null() && (*self_).s.weapon == WP_SABER {
            //Jedi taunt from within their AI
            (*(*self_).NPC).blockedSpeechDebounceTime = 0; //get them ready to taunt
            return;
        }
        if !(*self_).client.is_null() && (*(*self_).client).NPC_class == CLASS_GALAKMECH {
            (*self_).wait = 1.0;
            TIMER_Set(self_, c"gloatTime".as_ptr(), Q_irand(5000, 8000));
            (*(*self_).NPC).blockedSpeechDebounceTime = 0; //get him ready to taunt
            return;
        }
        //FIXME: any way to not say this *right away*?  Wait for victim's death anim/scream to finish?
        if !(*self_).NPC.is_null()
            && !(*(*self_).NPC).group.is_null()
            && !(*(*(*self_).NPC).group).commander.is_null()
            && !(*(*(*(*self_).NPC).group).commander).NPC.is_null()
            && (*(*(*(*(*self_).NPC).group).commander).NPC).rank > (*(*self_).NPC).rank
            && Q_irand(0, 2) == 0
        {
            //sometimes have the group commander speak instead
            (*(*(*(*(*self_).NPC).group).commander).NPC).greetingDebounceTime =
                (*addr_of!(level)).time + Q_irand(2000, 5000);
            //G_AddVoiceEvent( self->NPC->group->commander, Q_irand(EV_VICTORY1, EV_VICTORY3), 2000 );
        } else if !(*self_).NPC.is_null() {
            (*(*self_).NPC).greetingDebounceTime = (*addr_of!(level)).time + Q_irand(2000, 5000);
            //G_AddVoiceEvent( self, Q_irand(EV_VICTORY1, EV_VICTORY3), 2000 );
        }
    }
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle;

    /// Serializes the tests that mutate the process-global `level` (and the `g_gravity`/
    /// `g_knockback` cvars) so they don't race each other under the parallel test runner.
    /// Re-exports the crate-wide shared lock (g_main) so it serializes cross-module, not just
    /// within this module.
    use crate::codemp::game::g_main::level_lock;

    /// `G_GetHitLocation` over a sweep of yaws and impact points around a humanoid-sized
    /// box (including the origin → `HL_NONE`), checked bit-exact against the extracted C.
    /// Every caller passes a client, so the client slot is set non-null (only its
    /// presence is read). Covers all five vertical/forward/lateral buckets and the
    /// foot/leg/hand/arm/head/waist/back/chest classifications.
    #[test]
    fn g_gethitlocation_matches_oracle() {
        let absmin: vec3_t = [-16.0, -16.0, -24.0];
        let absmax: vec3_t = [16.0, 16.0, 32.0];
        let mins: vec3_t = [-16.0, -16.0, -24.0];
        let maxs: vec3_t = [16.0, 16.0, 32.0];

        let yaws = [0.0f32, 45.0, 90.0, 135.0, 180.0, 270.0, -33.0];
        let coords = [-40.0f32, -20.0, -8.0, 0.0, 8.0, 20.0, 40.0];

        for &yaw in &yaws {
            for &px in &coords {
                for &py in &coords {
                    for &pz in &coords {
                        let ppoint: vec3_t = [px, py, pz];

                        let mut ent: gentity_t = unsafe { core::mem::zeroed() };
                        ent.r.absmin = absmin;
                        ent.r.absmax = absmax;
                        ent.r.mins = mins;
                        ent.r.maxs = maxs;
                        ent.r.currentAngles = [0.0, yaw, 0.0];
                        ent.client = 8 as *mut _; // non-null; only presence is checked
                        let got =
                            unsafe { G_GetHitLocation(&mut ent, &ppoint as *const vec3_t) };

                        let want = unsafe {
                            oracle::jka_G_GetHitLocation(
                                1,
                                yaw,
                                absmin.as_ptr(),
                                absmax.as_ptr(),
                                mins.as_ptr(),
                                maxs.as_ptr(),
                                ppoint.as_ptr(),
                            )
                        };
                        assert_eq!(got, want, "yaw={yaw} ppoint={ppoint:?}");
                    }
                }
            }
        }
    }

    /// `G_GetDismemberLoc` over a sweep of pitch/yaw/roll angles, origins, and every
    /// limb type (plus a default), checked bit-exact against the extracted C. The
    /// function reads only `r.currentAngles`/`r.currentOrigin`, so the marshalling
    /// wrapper carries just those.
    #[test]
    fn g_getdismemberloc_matches_oracle() {
        let angle_sets: [vec3_t; 6] = [
            [0.0, 0.0, 0.0],
            [0.0, 90.0, 0.0],
            [15.0, 45.0, -30.0],
            [0.0, 180.0, 0.0],
            [-20.0, 270.0, 10.0],
            [33.0, -75.0, 5.0],
        ];
        let origins: [vec3_t; 3] = [
            [0.0, 0.0, 0.0],
            [100.0, -50.0, 24.0],
            [-37.5, 12.25, -8.0],
        ];
        let limb_types = [
            G2_MODELPART_HEAD,
            G2_MODELPART_WAIST,
            G2_MODELPART_LARM,
            G2_MODELPART_RARM,
            G2_MODELPART_RHAND,
            G2_MODELPART_LLEG,
            G2_MODELPART_RLEG,
            99, // default arm
        ];

        for angles in &angle_sets {
            for origin in &origins {
                for &limb in &limb_types {
                    let mut ent: gentity_t = unsafe { core::mem::zeroed() };
                    ent.r.currentAngles = *angles;
                    ent.r.currentOrigin = *origin;

                    let mut got: vec3_t = [0.0; 3];
                    unsafe { G_GetDismemberLoc(&mut ent, &mut got, limb) };

                    let mut want: vec3_t = [0.0; 3];
                    unsafe {
                        oracle::jka_G_GetDismemberLoc(
                            angles.as_ptr(),
                            origin.as_ptr(),
                            limb,
                            want.as_mut_ptr(),
                        );
                    }
                    assert_eq!(
                        got, want,
                        "angles={angles:?} origin={origin:?} limb={limb}"
                    );
                }
            }
        }
    }

    /// `G_ApplyKnockback` over a sweep of gravity sign, `g_knockback`, mass
    /// (`physicsBounce`), push direction, knockback magnitude, client-vs-mover,
    /// trajectory type, and pending `pm_time` — checked bit-exact against the extracted
    /// C. Covers the gravity `*0.8`/`*1.5` double-promoted scaling, the
    /// stationary-mover early-out, and the 50–200 ms timer clamp. Mutates the process
    /// cvar/`level` globals it reads; safe because every other oracle test is pure.
    #[test]
    fn g_applyknockback_matches_oracle() {
        use crate::codemp::game::q_shared_h::{TR_GRAVITY, TR_LINEAR};
        let _g = level_lock();

        let dirs: [vec3_t; 4] = [
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.577_350_26, 0.577_350_26, 0.577_350_26],
            [-0.3, 0.8, -0.5],
        ];
        let gravities = [800.0f32, 0.0, -1.0];
        let knock_cvars = [1000.0f32, 500.0];
        let bounces = [0.0f32, 100.0, 250.0];
        let knockbacks = [0.0f32, 10.0, 24.7, 100.0, 250.0];
        let tr_types = [TR_STATIONARY, TR_LINEAR_STOP, TR_NONLINEAR_STOP, TR_LINEAR, TR_GRAVITY];
        let pm_times = [0i32, 100];
        let level_time = 12_345i32;

        let in_vel: vec3_t = [5.0, -3.0, 2.0];
        let in_trdelta: vec3_t = [-1.0, 4.0, 0.5];
        let in_trbase: vec3_t = [10.0, 20.0, 30.0];
        let in_origin: vec3_t = [11.0, 22.0, 33.0];
        let in_trtime = 999i32;
        let in_pmflags = 0i32;

        unsafe {
            (*addr_of_mut!(level)).time = level_time;
        }

        for &grav in &gravities {
            for &kn in &knock_cvars {
                unsafe {
                    (*addr_of_mut!(g_gravity)).value = grav;
                    (*addr_of_mut!(g_knockback)).value = kn;
                }
                for &bounce in &bounces {
                    for dir in &dirs {
                        for &knockback in &knockbacks {
                            for &tr in &tr_types {
                                for &pm_time in &pm_times {
                                    for &has_client in &[false, true] {
                                        let mut ent: gentity_t = unsafe { core::mem::zeroed() };
                                        let mut client: gclient_t = unsafe { core::mem::zeroed() };
                                        ent.physicsBounce = bounce;
                                        ent.s.pos.trType = tr;
                                        ent.s.pos.trDelta = in_trdelta;
                                        ent.s.pos.trBase = in_trbase;
                                        ent.s.pos.trTime = in_trtime;
                                        ent.r.currentOrigin = in_origin;
                                        if has_client {
                                            client.ps.velocity = in_vel;
                                            client.ps.pm_time = pm_time;
                                            client.ps.pm_flags = in_pmflags;
                                            ent.client = &mut client;
                                        }

                                        unsafe {
                                            G_ApplyKnockback(&mut ent, dir as *const vec3_t, knockback);
                                        }

                                        // Gather the Rust-mutated fields; null-client cases
                                        // leave velocity / pm untouched (= inputs).
                                        let got_vel = if has_client {
                                            client.ps.velocity
                                        } else {
                                            in_vel
                                        };
                                        let (got_pmt, got_pmf) = if has_client {
                                            (client.ps.pm_time, client.ps.pm_flags)
                                        } else {
                                            (pm_time, in_pmflags)
                                        };
                                        let got = (
                                            got_vel,
                                            ent.s.pos.trDelta,
                                            ent.s.pos.trBase,
                                            ent.s.pos.trTime,
                                            got_pmt,
                                            got_pmf,
                                        );

                                        let mut w_vel: vec3_t = [0.0; 3];
                                        let mut w_trd: vec3_t = [0.0; 3];
                                        let mut w_trb: vec3_t = [0.0; 3];
                                        let mut w_trt: c_int = 0;
                                        let mut w_pmt: c_int = 0;
                                        let mut w_pmf: c_int = 0;
                                        unsafe {
                                            oracle::jka_G_ApplyKnockback(
                                                bounce,
                                                has_client as c_int,
                                                tr,
                                                dir.as_ptr(),
                                                knockback,
                                                grav,
                                                kn,
                                                level_time,
                                                in_vel.as_ptr(),
                                                in_trdelta.as_ptr(),
                                                in_trbase.as_ptr(),
                                                in_origin.as_ptr(),
                                                in_trtime,
                                                pm_time,
                                                in_pmflags,
                                                w_vel.as_mut_ptr(),
                                                w_trd.as_mut_ptr(),
                                                w_trb.as_mut_ptr(),
                                                &mut w_trt,
                                                &mut w_pmt,
                                                &mut w_pmf,
                                            );
                                        }
                                        let want = (w_vel, w_trd, w_trb, w_trt, w_pmt, w_pmf);

                                        assert_eq!(
                                            got, want,
                                            "grav={grav} kn={kn} bounce={bounce} dir={dir:?} \
                                             knockback={knockback} tr={tr} pm_time={pm_time} \
                                             has_client={has_client}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// `RaySphereIntersections` over rays that miss (0 roots), graze (tangent, 1 root) and
    /// pierce (2 roots) a sphere, plus non-axis-aligned and non-unit input directions to
    /// exercise the in-place `VectorNormalize`. Checked bit-exact against the extracted C:
    /// the returned count, the normalized `dir`, and each intersection slot the call
    /// actually writes (the untouched slots are left as-is in both, so they aren't compared).
    #[test]
    fn raysphereintersections_matches_oracle() {
        struct Case {
            origin: vec3_t,
            radius: f32,
            point: vec3_t,
            dir: vec3_t,
        }
        let cases = [
            // pierce, axis-aligned: 2 roots
            Case { origin: [0.0, 0.0, 0.0], radius: 10.0, point: [-50.0, 0.0, 0.0], dir: [1.0, 0.0, 0.0] },
            // tangent: d == 0 exactly (all integer-valued operands)
            Case { origin: [0.0, 0.0, 0.0], radius: 10.0, point: [-50.0, 10.0, 0.0], dir: [1.0, 0.0, 0.0] },
            // miss: d < 0, 0 roots
            Case { origin: [0.0, 0.0, 0.0], radius: 10.0, point: [-50.0, 20.0, 0.0], dir: [1.0, 0.0, 0.0] },
            // non-axis, non-unit dir (exercises normalize), offset center, pierce
            Case { origin: [10.0, 5.0, -3.0], radius: 25.0, point: [-40.0, -30.0, 12.0], dir: [3.0, 2.0, -1.0] },
            // diagonal ray at far center
            Case { origin: [100.0, 100.0, 100.0], radius: 50.0, point: [0.0, 0.0, 0.0], dir: [1.0, 1.0, 1.0] },
            // ray origin inside the sphere (one root each side → still 2)
            Case { origin: [0.0, 0.0, 0.0], radius: 30.0, point: [5.0, -2.0, 1.0], dir: [0.3, 0.9, -0.2] },
        ];

        for (i, c) in cases.iter().enumerate() {
            let mut dir = c.dir;
            let mut inter: [vec3_t; 2] = [[f32::NAN; 3]; 2];
            let n = RaySphereIntersections(&c.origin, c.radius, &c.point, &mut dir, &mut inter);

            let mut w_dir: vec3_t = [0.0; 3];
            let mut w0: vec3_t = [0.0; 3];
            let mut w1: vec3_t = [0.0; 3];
            let want_n = unsafe {
                oracle::jka_RaySphereIntersections(
                    c.origin.as_ptr(),
                    c.radius,
                    c.point.as_ptr(),
                    c.dir.as_ptr(),
                    w_dir.as_mut_ptr(),
                    w0.as_mut_ptr(),
                    w1.as_mut_ptr(),
                )
            };
            assert_eq!(n, want_n, "case {i}: count");
            assert_eq!(dir, w_dir, "case {i}: normalized dir");
            if n >= 1 {
                assert_eq!(inter[0], w0, "case {i}: intersections[0]");
            }
            if n >= 2 {
                assert_eq!(inter[1], w1, "case {i}: intersections[1]");
            }
        }
    }

    /// `G_InKnockDown` over every knockdown/get-up anim that returns `qtrue`, the two
    /// crouch get-ups that deliberately fall through to `qfalse`, and assorted unrelated
    /// anims — each compared bit-exact against the extracted C switch.
    #[test]
    fn g_inknockdown_matches_oracle() {
        use crate::codemp::game::anims::{
            BOTH_FORCE_GETUP_B6, BOTH_GETUP_CROUCH_B1, BOTH_GETUP_CROUCH_F1,
        };
        // The full set that must return qtrue (matches the C case list).
        let knocked = [
            BOTH_KNOCKDOWN1, BOTH_KNOCKDOWN2, BOTH_KNOCKDOWN3, BOTH_KNOCKDOWN4, BOTH_KNOCKDOWN5,
            BOTH_GETUP1, BOTH_GETUP2, BOTH_GETUP3, BOTH_GETUP4, BOTH_GETUP5,
            BOTH_FORCE_GETUP_F1, BOTH_FORCE_GETUP_F2, BOTH_FORCE_GETUP_B1, BOTH_FORCE_GETUP_B2,
            BOTH_FORCE_GETUP_B3, BOTH_FORCE_GETUP_B4, BOTH_FORCE_GETUP_B5,
        ];
        // Anims that must NOT count as a knockdown — including the two crouch get-ups
        // (1229/1230) that fall in the numeric gap but are absent from the switch.
        let not_knocked = [
            -1, 0, 1, BOTH_GETUP_CROUCH_F1, BOTH_GETUP_CROUCH_B1, BOTH_FORCE_GETUP_B6, 1218, 1238,
            9999,
        ];

        for &legs in knocked.iter().chain(not_knocked.iter()) {
            let mut ps: playerState_t = unsafe { core::mem::zeroed() };
            ps.legsAnim = legs;
            let got = unsafe { G_InKnockDown(&ps) };
            let want = unsafe { oracle::jka_G_InKnockDown(legs) };
            assert_eq!(got as c_int, want, "legsAnim={legs}");
        }
    }

    /// `G_Knockdown` over the `BG_KnockDownable` gate matrix (`m_iVehicleNum` /
    /// `emplacedIndex`) and a sweep of `level.time` (incl. negative), checked bit-exact
    /// against the extracted C. The four written fields are seeded with sentinels so the
    /// not-knockdownable branch (fields left untouched) is observable; the gated branch
    /// must produce `HANDEXTEND_KNOCKDOWN` / 0 / `time+1100` / `QFALSE`. Mutates the
    /// `level` global it reads; safe because the other oracle tests don't depend on it.
    #[test]
    fn g_knockdown_matches_oracle() {
        let _g = level_lock();
        let veh_nums = [0i32, 1, 7];
        let emplaced = [0i32, 1, 3];
        let times = [-5000i32, 0, 12_345];
        // Sentinel seeds for the four mutable fields, so an untouched field is visible.
        let (seed_fhe, seed_fda, seed_fhet, seed_qg) = (99i32, 88i32, 77i32, 1i32);

        for &veh in &veh_nums {
            for &emp in &emplaced {
                for &t in &times {
                    unsafe {
                        (*addr_of_mut!(level)).time = t;
                    }
                    let mut ent: gentity_t = unsafe { core::mem::zeroed() };
                    let mut client: gclient_t = unsafe { core::mem::zeroed() };
                    client.ps.m_iVehicleNum = veh;
                    client.ps.emplacedIndex = emp;
                    client.ps.forceHandExtend = seed_fhe;
                    client.ps.forceDodgeAnim = seed_fda;
                    client.ps.forceHandExtendTime = seed_fhet;
                    client.ps.quickerGetup = seed_qg;
                    ent.client = &mut client;

                    unsafe {
                        G_Knockdown(&mut ent);
                    }

                    let (mut o_fhe, mut o_fda, mut o_fhet, mut o_qg) =
                        (seed_fhe, seed_fda, seed_fhet, seed_qg);
                    unsafe {
                        oracle::jka_G_Knockdown(
                            veh, emp, t, &mut o_fhe, &mut o_fda, &mut o_fhet, &mut o_qg,
                        );
                    }

                    let tag = format!("veh={veh} emp={emp} t={t}");
                    assert_eq!(client.ps.forceHandExtend, o_fhe, "{tag} forceHandExtend");
                    assert_eq!(client.ps.forceDodgeAnim, o_fda, "{tag} forceDodgeAnim");
                    assert_eq!(
                        client.ps.forceHandExtendTime, o_fhet,
                        "{tag} forceHandExtendTime"
                    );
                    assert_eq!(client.ps.quickerGetup, o_qg, "{tag} quickerGetup");
                }
            }
        }

        // The NULL-victim and NULL-client guards must be no-ops, not crashes.
        unsafe {
            G_Knockdown(core::ptr::null_mut());
            let mut bare: gentity_t = core::mem::zeroed();
            bare.client = core::ptr::null_mut();
            G_Knockdown(&mut bare);
        }
    }

    /// `G_CheckSpecialDeathAnim` over the full knockdown/get-up switch, both gate
    /// branches (roll/flip), and the generic no-op fall-through, checked bit-exact
    /// against the extracted C. `legsTimer` sweeps every threshold the switch tests;
    /// `numFrames`/`frameLerp` vary `animLength`; the `velocity` set flips the
    /// crouched-vs-thrown-back (`thrown < -150`) sub-branch (viewangles 0 → forward
    /// `[1,0,0]`, so `thrown == velocity.x`). The `BG_InRoll`/`BG_FlippingAnim` gates are
    /// computed by the real functions and passed to the oracle, so the comparison is of
    /// the selector logic given identical gate results (the gates are tested in
    /// bg_panimate). Holds [`pm_lock`](crate::codemp::game::bg_pmove::pm_lock) since it
    /// seeds the shared `bgAllAnims`/`bgHumanoidAnimations` globals.
    #[test]
    fn g_checkspecialdeathanim_matches_oracle() {
        use crate::codemp::game::anims::{BOTH_FLIP_F, BOTH_ROLL_F, MAX_TOTALANIMATIONS};
        use crate::codemp::game::bg_panimate::{bgAllAnims, bgHumanoidAnimations};
        use crate::codemp::game::bg_pmove::pm_lock;
        use crate::codemp::game::bg_public::animation_t;

        let _g = pm_lock();

        // localAnimIndex slot we point at a controllable backing animation array.
        const IDX: c_int = 1;
        let mut backing: Vec<animation_t> =
            vec![animation_t::default(); MAX_TOTALANIMATIONS as usize];

        let anims = [
            BOTH_KNOCKDOWN1, BOTH_KNOCKDOWN2, BOTH_KNOCKDOWN3, BOTH_KNOCKDOWN4, BOTH_KNOCKDOWN5,
            BOTH_GETUP1, BOTH_GETUP2, BOTH_GETUP3, BOTH_GETUP4, BOTH_GETUP5, BOTH_GETUP_CROUCH_B1,
            BOTH_GETUP_CROUCH_F1, BOTH_FORCE_GETUP_B1, BOTH_FORCE_GETUP_B2, BOTH_FORCE_GETUP_B3,
            BOTH_FORCE_GETUP_B4, BOTH_FORCE_GETUP_B5, BOTH_FORCE_GETUP_B6, BOTH_FORCE_GETUP_F1,
            BOTH_FORCE_GETUP_F2, BOTH_ROLL_F, BOTH_FLIP_F, BOTH_DEATH14,
        ];
        // Every threshold the switch compares legsTimer against (plus 0 and a large value).
        let timers = [
            0, 100, 149, 150, 225, 250, 275, 325, 350, 425, 550, 575, 600, 700, 725, 750, 775, 800,
            825, 850, 875, 900, 1025, 1200, 1300, 1500,
        ];
        let frames: [(u16, i16); 4] = [(0, 0), (20, 100), (50, -80), (100, 50)];
        let viewangles: vec3_t = [0.0, 0.0, 0.0];
        let velocities: [vec3_t; 3] = [[200.0, 0.0, 0.0], [-200.0, 0.0, 0.0], [0.0, 0.0, 0.0]];

        unsafe {
            let saved = (*addr_of!(bgAllAnims))[IDX as usize].anims;
            (*addr_of_mut!(bgAllAnims))[IDX as usize].anims = backing.as_mut_ptr();

            for &la in &anims {
                for &(nf, fl) in &frames {
                    backing[la as usize].numFrames = nf;
                    (*addr_of_mut!(bgHumanoidAnimations))[la as usize].frameLerp = fl;

                    for &lt in &timers {
                        for &vel in &velocities {
                            let mut client: gclient_t = core::mem::zeroed();
                            client.ps.legsAnim = la;
                            client.ps.legsTimer = lt;
                            client.ps.viewangles = viewangles;
                            client.ps.velocity = vel;

                            let mut ent: gentity_t = core::mem::zeroed();
                            ent.client = &mut client;
                            ent.localAnimIndex = IDX;

                            let in_roll = BG_InRoll(&mut client.ps, la);
                            let flipping = BG_FlippingAnim(la);

                            let got = G_CheckSpecialDeathAnim(
                                &mut ent,
                                core::ptr::null(),
                                0,
                                0,
                                HL_NONE,
                            );
                            let want = crate::oracle::jka_G_CheckSpecialDeathAnim(
                                la,
                                lt,
                                viewangles.as_ptr(),
                                vel.as_ptr(),
                                in_roll,
                                flipping,
                                nf,
                                fl,
                            );
                            assert_eq!(
                                got, want,
                                "legsAnim={la} legsTimer={lt} numFrames={nf} frameLerp={fl} vel={vel:?}"
                            );
                        }
                    }

                    backing[la as usize].numFrames = 0;
                    (*addr_of_mut!(bgHumanoidAnimations))[la as usize].frameLerp = 0;
                }
            }

            (*addr_of_mut!(bgAllAnims))[IDX as usize].anims = saved;
        }
    }

    /// `G_PickDeathAnim`'s hit-location selector, checked bit-exact against the extracted
    /// C. The `legsAnim` is a neutral (non-roll/flip/knockdown, non-dead-flop) value so
    /// `G_CheckSpecialDeathAnim` returns -1 and the hitLoc switch runs; the anim table is
    /// stocked full (every `numFrames = 1`) so `BG_HasAnimation` passes and the validation
    /// tail never reaches the RNG-driven `BG_PickAnim`. Rust's `Rand_Init` and the oracle's
    /// `Rand_Init` are seeded identically before each pair so the in-lockstep `Q_irand`
    /// (a PC wrapper over `irand`'s holdrand LCG) draws agree. Sweeps every `HL_*`, the
    /// saber-vs-other `mod`, the damage/max_health buckets, the zero-vs-nonzero velocity
    /// split, and a set of seeds. Holds
    /// [`pm_lock`](crate::codemp::game::bg_pmove::pm_lock) (anim-table globals). The trivial
    /// early returns are asserted directly.
    #[test]
    fn g_pickdeathanim_matches_oracle() {
        use crate::codemp::game::anims::MAX_TOTALANIMATIONS;
        use crate::codemp::game::bg_panimate::{bgAllAnims, bgNumAllAnims};
        use crate::codemp::game::bg_pmove::pm_lock;
        use crate::codemp::game::bg_public::animation_t;
        use crate::codemp::game::q_math::Rand_Init;
        use crate::oracle::{jka_G_PickDeathAnim, Rand_Init as oracle_Rand_Init};

        let _g = pm_lock();

        const IDX: c_int = 1;
        // Not a dead-flop / knockdown / roll / flip anim, so G_CheckSpecialDeathAnim → -1
        // and the dead-flop switch leaves deathAnim == -1 → the hitLoc switch runs.
        const NEUTRAL_LEGANIM: c_int = 200;

        // Every anim valid so BG_HasAnimation passes and BG_PickAnim is never reached.
        let mut backing: Vec<animation_t> = vec![
            animation_t { numFrames: 1, ..Default::default() };
            MAX_TOTALANIMATIONS as usize
        ];

        let hitlocs = [
            HL_FOOT_RT, HL_FOOT_LT, HL_LEG_RT, HL_LEG_LT, HL_BACK, HL_CHEST_RT, HL_ARM_RT,
            HL_HAND_RT, HL_BACK_RT, HL_CHEST_LT, HL_ARM_LT, HL_HAND_LT, HL_BACK_LT, HL_CHEST,
            HL_WAIST, HL_HEAD,
        ];
        let mods = [MOD_SABER, MOD_MELEE]; // MOD_MELEE ≠ MOD_SABER → foot saber sub-branch off
        let max_health: c_int = 100;
        let damages = [10, 25, 26, 50, 51, 75, 76, 200];
        let velocities: [vec3_t; 2] = [[0.0, 0.0, 0.0], [120.0, 0.0, 0.0]];
        let seeds: [u32; 6] = [1, 2, 3, 7, 12345, 99999];

        unsafe {
            let saved_anims = (*addr_of!(bgAllAnims))[IDX as usize].anims;
            let saved_num = *addr_of!(bgNumAllAnims);
            (*addr_of_mut!(bgAllAnims))[IDX as usize].anims = backing.as_mut_ptr();
            if *addr_of!(bgNumAllAnims) < IDX {
                *addr_of_mut!(bgNumAllAnims) = IDX;
            }

            for &hl in &hitlocs {
                for &md in &mods {
                    for &dmg in &damages {
                        for &vel in &velocities {
                            for &seed in &seeds {
                                let mut client: gclient_t = core::mem::zeroed();
                                client.ps.legsAnim = NEUTRAL_LEGANIM;
                                client.ps.stats[STAT_MAX_HEALTH as usize] = max_health;
                                client.ps.velocity = vel;

                                let mut ent: gentity_t = core::mem::zeroed();
                                ent.client = &mut client;
                                ent.localAnimIndex = IDX;

                                Rand_Init(seed as c_int);
                                let got =
                                    G_PickDeathAnim(&mut ent, core::ptr::null(), dmg, md, hl);
                                oracle_Rand_Init(seed as c_int);
                                let want = jka_G_PickDeathAnim(
                                    hl,
                                    dmg,
                                    md,
                                    max_health,
                                    vel.as_ptr(),
                                    -1,
                                );
                                assert_eq!(
                                    got, want,
                                    "hitLoc={hl} mod={md} dmg={dmg} vel={vel:?} seed={seed}"
                                );
                            }
                        }
                    }
                }
            }

            (*addr_of_mut!(bgAllAnims))[IDX as usize].anims = saved_anims;
            *addr_of_mut!(bgNumAllAnims) = saved_num;
        }

        // Deterministic early returns (no RNG, no shared mutable globals touched).
        unsafe {
            // NULL self → 0.
            assert_eq!(
                G_PickDeathAnim(core::ptr::null_mut(), core::ptr::null(), 10, MOD_SABER, HL_HEAD),
                0
            );
            // Non-client, non-NPC entity → 0.
            let mut ent: gentity_t = core::mem::zeroed();
            ent.client = core::ptr::null_mut();
            ent.s.eType = ET_GENERAL;
            assert_eq!(
                G_PickDeathAnim(&mut ent, core::ptr::null(), 10, MOD_SABER, HL_HEAD),
                0
            );
            // inSpaceIndex set → BOTH_CHOKE3.
            let mut client: gclient_t = core::mem::zeroed();
            client.inSpaceIndex = 5;
            let mut ent2: gentity_t = core::mem::zeroed();
            ent2.client = &mut client;
            assert_eq!(
                G_PickDeathAnim(&mut ent2, core::ptr::null(), 10, MOD_SABER, HL_HEAD),
                BOTH_CHOKE3
            );
        }
    }
}
