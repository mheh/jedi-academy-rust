//! Port of `w_force.c` — the server-side force-power subsystem (init, activation,
//! per-power stop/cleanup, the force-combat helpers). Landed incrementally as
//! already-ported callers reach in: `WP_ForcePowerStop` lands first as the dependency
//! that the spawn-time `WP_SpawnInitForcePowers` (and `ClientSpawn`) needs to deactivate
//! each still-active power, plus the grip/lightning/drain timeout handlers call it.

#![allow(non_snake_case)] // C function names kept verbatim

use core::ffi::c_int;
use core::ptr::addr_of;

use crate::codemp::game::bg_misc::{forcePowerDarkLight, BG_CanUseFPNow, BG_HasYsalamiri};
use crate::codemp::game::bg_panimate::{
    BG_FullBodyTauntAnim, BG_InReboundHold, BG_InReboundJump, BG_SaberInSpecial,
};
use crate::codemp::game::bg_pmove::{forceJumpStrength, forcePowerNeeded, BG_InKnockDown};
use crate::codemp::game::g_local::{
    DAMAGE_NO_ARMOR, FRAMETIME, HL_ARM_LT, HL_ARM_RT, HL_BACK, HL_BACK_LT, HL_BACK_RT, HL_CHEST,
    HL_CHEST_LT, HL_CHEST_RT, HL_FOOT_LT, HL_FOOT_RT, HL_HAND_LT, HL_HAND_RT, HL_HEAD, HL_LEG_LT,
    HL_LEG_RT, HL_NONE, HL_WAIST, MOVER_POS1, MOVER_POS2, SPF_BUTTON_FPUSHABLE,
};
use crate::codemp::game::anims::{
    BOTH_DODGE_FL, BOTH_DODGE_FR, BOTH_DODGE_L, BOTH_DODGE_R, BOTH_FORCE_GETUP_B1,
    BOTH_FORCE_GETUP_B2, BOTH_FORCE_GETUP_B3, BOTH_FORCE_GETUP_B4,
    BOTH_FORCE_GETUP_B5, BOTH_FORCE_GETUP_F1, BOTH_FORCE_GETUP_F2, BOTH_GETUP1, BOTH_GETUP2,
    BOTH_GETUP3, BOTH_GETUP4, BOTH_GETUP5, BOTH_GETUP_BROLL_B, BOTH_GETUP_BROLL_F,
    BOTH_GETUP_BROLL_L, BOTH_GETUP_BROLL_R, BOTH_GETUP_FROLL_B, BOTH_GETUP_FROLL_F,
    BOTH_GETUP_FROLL_L, BOTH_GETUP_FROLL_R,
};
use crate::codemp::game::bg_public::{
    EF_DEAD, EF_INVULNERABLE, EF_MISSILE_STICK, EF_NODRAW, EF_SEEKERDRONE, EFFECT_SPARK_EXPLOSION,
    ET_ITEM, ET_MISSILE, ET_NPC, EV_FORCE_DRAINED, EV_PREDEFSOUND, EV_TEAM_POWER, GT_SIEGE,
    HANDEXTEND_CHOKE,
    HANDEXTEND_DODGE,
    HANDEXTEND_FORCE_HOLD,
    HANDEXTEND_FORCEPULL,
    HANDEXTEND_FORCEPUSH,
    HANDEXTEND_KNOCKDOWN,
    HANDEXTEND_NONE,
    HANDEXTEND_POSTTHROWN,
    HANDEXTEND_WEAPONREADY,
    GT_JEDIMASTER,
    GT_POWERDUEL,
    MASK_SOLID,
    MOD_BLASTER,
    MOD_FORCE_DARK,
    NUM_FORCE_MASTERY_LEVELS,
    PW_SPEED,
    PW_SPEEDBURST,
    PW_FORCE_BOON,
    PW_FORCE_ENLIGHTENED_DARK,
    PW_FORCE_ENLIGHTENED_LIGHT,
    PDSOUND_ABSORB,
    PDSOUND_ABSORBHIT,
    PDSOUND_FORCEGRIP,
    PDSOUND_FORCEJUMP,
    PDSOUND_PROTECT,
    PMF_FOLLOW,
    PMF_JUMP_HELD,
    WEAPON_READY,
    PMF_STUCK_TO_WALL, PM_FLOAT, PM_NORMAL, PW_BLUEFLAG, PW_CLOAKED, PW_DISINT_4, PW_PULL,
    PW_REDFLAG, SETANIM_BOTH, SETANIM_FLAG_HOLD,
    SETANIM_FLAG_OVERRIDE, STAT_HEALTH, STAT_MAX_HEALTH, TEAM_SPECTATOR, WEAPON_CHARGING,
    WEAPON_CHARGING_ALT, DUELTEAM_LONE, bgEntity_t,
};
use crate::codemp::game::bg_vehicles_h::{VH_ANIMAL, VH_SPEEDER};
use crate::codemp::game::bg_saber::BG_ForcePowerDrain;
use crate::codemp::game::bg_weapons_h::{WP_MELEE, WP_NONE, WP_SABER, WP_THERMAL};
use crate::codemp::game::teams_h::{
    CLASS_ATST, CLASS_BOBAFETT, CLASS_GALAKMECH, CLASS_GONK, CLASS_MARK1, CLASS_MARK2, CLASS_MOUSE,
    CLASS_PROBE, CLASS_PROTOCOL, CLASS_R2D2, CLASS_R5D2, CLASS_RANCOR, CLASS_REBORN, CLASS_REMOTE,
    CLASS_SEEKER, CLASS_VEHICLE, NPCTEAM_ENEMY, NPCTEAM_NEUTRAL, NPCTEAM_PLAYER,
};
use crate::codemp::game::npc_sounds::NPC_PlayConfusionSound;
use crate::codemp::game::npc_ai_jedi::NPC_Jedi_PlayConfusionSound;
use crate::codemp::game::npc_reactions::NPC_UseResponse;
use crate::codemp::game::npc_combat::G_ClearEnemy;
use crate::codemp::game::npc_senses::{AddSightEvent, AddSoundEvent};
use crate::codemp::game::bg_saga::bgSiegeClasses;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::{
    g_debugMelee, g_duel_fraglimit, g_entities, g_forceDodge, g_forceRegenTime, g_friendlyFire,
    g_gametype, g_saberRestrictForce, g_useWhileThrowing, level,
};
use crate::codemp::game::bg_saga_h::CFL_FASTFORCEREGEN;
use crate::codemp::game::g_utils::{
    G_EffectIndex, G_EntitySound, G_MuteSound, G_PlayEffect, G_PlayEffectID, G_SetAnim, G_Sound,
    G_SoundAtLoc, G_SoundIndex, G_TempEntity, GlobalUse,
};
use crate::codemp::game::npc_utils::G_ActivateBehavior;
use crate::codemp::game::g_local::AEL_SUSPICIOUS;
use crate::codemp::game::bg_public::{PM_DEAD, TEAM_BLUE, TEAM_RED};
use crate::codemp::game::g_public_h::BSET_MINDTRICK;
use crate::codemp::game::g_combat::{G_Damage, TossClientWeapon};
use crate::codemp::game::g_weapon::WP_FireGenericBlasterMissile;
use crate::codemp::game::g_missile::G_ReflectMissile;
use crate::codemp::game::g_items::Jetpack_Off;
use crate::codemp::game::g_mover::Touch_Button;
use crate::codemp::game::bg_pmove::BG_KnockDownable;
use crate::codemp::game::ai_main::{InFieldOfVision, OrgVisible};
use crate::codemp::game::bg_public::{
    GT_HOLOCRON, JUMP_VELOCITY, MASK_PLAYERSOLID, MASK_SHOT, MOD_UNKNOWN, PM_INTERMISSION,
    PM_SPECTATOR,
};
use crate::codemp::game::g_team::OnSameTeam;
use crate::codemp::game::npc_senses::InFront;
use crate::codemp::game::q_math::{
    AngleSubtract, AngleVectors, DirToByte, Distance, DotProduct, VectorAdd, VectorCompare,
    VectorCopy, VectorLength, VectorMA, VectorNormalize, VectorSubtract, vec3_origin, vectoangles,
};
use crate::codemp::game::q_shared::{Q_stricmp};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::SFL_TWO_HANDED;
use crate::codemp::game::q_shared_h::{
    forcedata_t, playerState_t, qboolean, trace_t, usercmd_t, vec3_t, BLOCKED_NONE,
    BUTTON_FORCEGRIP,
    BUTTON_FORCEPOWER, BUTTON_FORCE_DRAIN, BUTTON_FORCE_LIGHTNING, CHAN_AUTO,
    CHAN_BODY,
    CHAN_ITEM,
    CHAN_VOICE,
    CHAN_WEAPON,
    ENTITYNUM_NONE, ENTITYNUM_WORLD,
    FORCE_LEVEL_0, FORCE_LEVEL_1, FORCE_LEVEL_2, FORCE_LEVEL_3, FP_ABSORB, FP_DRAIN, FP_GRIP, FP_HEAL,
    FP_LEVITATION, FP_LIGHTNING, FP_PROTECT, FP_PULL, FP_PUSH, FP_RAGE, FP_SABERTHROW,
    FP_SABER_DEFENSE, FP_SABER_OFFENSE, FP_SEE, FP_SPEED, FP_TEAM_FORCE, FP_TEAM_HEAL, FP_TELEPATHY,
    MAX_CLIENTS, MAX_GENTITIES, M_PI, NUM_FORCE_POWERS, NUM_FORCE_POWER_LEVELS, PITCH, QFALSE,
    QTRUE, ROLL, TR_INTERPOLATE, TR_LINEAR, TR_STATIONARY, TRACK_CHANNEL_1, TRACK_CHANNEL_2,
    TRACK_CHANNEL_3,
    TRACK_CHANNEL_4, TRACK_CHANNEL_5,
    YAW,
};
use crate::codemp::game::w_saber_h::{
    FJ_BACKWARD, FJ_FORWARD, FJ_LEFT, FJ_RIGHT, FJ_UP, FORCE_JUMP_CHARGE_TIME, FORCE_LIGHTNING_RADIUS,
    FORCE_POWER_MAX, GRIP_DRAIN_AMOUNT, MAX_DRAIN_DISTANCE, MAX_GRIP_DISTANCE, MAX_TRICK_DISTANCE,
};
use crate::codemp::game::b_public_h::{SCF_NO_FORCE, SCF_NO_MIND_TRICK, SCF_NO_RESPONSE};
use crate::codemp::game::g_public_h::SVF_BOT;
// --- WP_InitForcePowers (w_force.c:152) deps ---
use core::ffi::c_char;
use crate::codemp::game::ai_main::botstates;
use crate::codemp::game::bg_misc::BG_LegalizedForcePowers;
use crate::codemp::game::bg_public::{
    EV_SET_FORCE_DISABLE, EV_SET_FREE_SABER, FORCE_MASTERY_JEDI_MASTER,
};
use crate::codemp::game::g_local::{SPECTATOR_FREE, TEAM_BEGIN};
use crate::codemp::game::g_main::{
    g_forceBasedTeams, g_forcePowerDisable, g_MaxHolocronCarry, g_maxForceRank, g_teamAutoJoin,
};
use crate::codemp::game::g_public_h::SVF_BROADCAST;
use crate::codemp::game::q_shared::{Com_sprintf, Info_ValueForKey, Q_strncpyz, Sz};
use crate::codemp::game::q_shared_h::{FORCE_DARKSIDE, FORCE_LIGHTSIDE, MAX_INFO_STRING};
use crate::codemp::game::w_saber::HasSetSaberOnly;
use crate::trap;

extern "C" {
    /// libc `int atoi( const char * )` — the retail (non-`Q3_VM`) build links the C library's
    /// `atoi` (the `g_client.rs` / `g_session.rs` precedent for `bg_lib`'s `Q3_VM`-gated copy).
    fn atoi(s: *const c_char) -> c_int;
    /// libc `char *strcpy( char *, const char * )`.
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

/// `gentity_t *G_PreDefSound(vec3_t org, int pdSound)` (w_force.c:45) — fire a predefined
/// sound at `org`. Spawns an `EV_PREDEFSOUND` temp-entity, stamps the `pdSounds_t` index into
/// `s.eventParm`, copies `org` into `s.origin`, and returns the temp entity.
///
/// No oracle — `G_TempEntity` spawns into the live entity array (entity/trap control-flow
/// precedent).
///
/// # Safety
/// The entity system must be initialised (`G_TempEntity` allocates from `g_entities`).
pub unsafe fn G_PreDefSound(org: &vec3_t, pd_sound: c_int) -> *mut gentity_t {
    let te = G_TempEntity(org, EV_PREDEFSOUND);
    (*te).s.eventParm = pd_sound;
    VectorCopy(org, &mut (*te).s.origin);

    te
}

// File-static loop-sound config-string indices (w_force.c:29-39). Each is set by
// `WP_InitForcePowers` to the registered loop sound and read by the matching activator.
// Before that init runs they stay 0 (the C initialiser), which `G_Sound` treats as a no-op
// sound index.
static mut SPEED_LOOP_SOUND: c_int = 0;
static mut RAGE_LOOP_SOUND: c_int = 0;
static mut PROTECT_LOOP_SOUND: c_int = 0;
static mut ABSORB_LOOP_SOUND: c_int = 0;
static mut SEE_LOOP_SOUND: c_int = 0;
// `int ysalamiriLoopSound = 0;` (w_force.c:39) — only written by `WP_InitForcePowers`
// (no ported reader yet); kept for fidelity with the C file-static set.
static mut YSALAMIRI_LOOP_SOUND: c_int = 0;

// `const int mindTrickTime[NUM_FORCE_POWER_LEVELS]` (w_force.c:144) — how long (ms) a
// mind-tricked/confused NPC stays charmed or confused, indexed by the tricker's FP_TELEPATHY
// power level (0/1/2/3).
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mindTrickTime: [c_int; NUM_FORCE_POWER_LEVELS] = [
    0, //none
    5000, 10000, 15000,
];

// `void Jedi_Decloak( gentity_t *self )` (NPC_misc.c) — temporarily disables a cloaked NPC's
// cloak; C declares it `extern` at the top of w_force.c. Now ported in npc_ai_jedi.rs, so
// imported directly (the prior `extern "C"` forward-decl is dropped).
use crate::codemp::game::npc_ai_jedi::Jedi_Decloak;

use crate::codemp::game::g_cmds::Cmd_ToggleSaber_f;

// `extern int g_TimeSinceLastFrame;` / `extern int g_LastFrameTime;` (w_force.c:4231-4232) —
// the per-frame delta + previous-frame timestamp. Both are g_main.c globals (g_main.c:3591-3592),
// written once per `G_RunFrame`; w_force.c only `extern`s them. Now that `G_RunFrame` has landed,
// they are homed in g_main.rs and imported here (resolving the prior file-local-mirror REVISIT).
use crate::codemp::game::g_main::{g_LastFrameTime, g_TimeSinceLastFrame};

/// `void WP_ForcePowerStop( gentity_t *self, forcePowers_t forcePower )` (w_force.c:3826) —
/// deactivate `forcePower` on `self` and run its per-power cleanup. Clears the active bit,
/// then a `switch` does the power-specific teardown: stop loop sounds (speed/see/rage/absorb/
/// protect mute their voice channel), clear heal/telepathy state, set the grip/lightning/drain
/// debounce timers and gasp the grip victim, reset hand-extend, etc. Reads the `level` clock
/// and `g_entities` (for the grip-victim entity).
///
/// No oracle — gentity/playerState mutation + sound-trap side effects (the entity/trap
/// control-flow precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`. `g_entities` must be initialised
/// (the grip case indexes it by `forceGripEntityNum`).
pub unsafe fn WP_ForcePowerStop(self_: *mut gentity_t, force_power: c_int) {
    let client = (*self_).client;
    let was_active = (*client).ps.fd.forcePowersActive;

    (*client).ps.fd.forcePowersActive &= !(1 << force_power);

    match force_power {
        FP_HEAL => {
            (*client).ps.fd.forceHealAmount = 0;
            (*client).ps.fd.forceHealTime = 0;
        }
        FP_LEVITATION => {}
        FP_SPEED => {
            if was_active & (1 << FP_SPEED) != 0 {
                G_MuteSound(
                    (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_2 - 50) as usize],
                    CHAN_VOICE,
                );
            }
        }
        FP_PUSH => {}
        FP_PULL => {}
        FP_TELEPATHY => {
            if was_active & (1 << FP_TELEPATHY) != 0 {
                G_Sound(
                    self_,
                    CHAN_AUTO,
                    G_SoundIndex("sound/weapons/force/distractstop.wav"),
                );
            }
            (*client).ps.fd.forceMindtrickTargetIndex = 0;
            (*client).ps.fd.forceMindtrickTargetIndex2 = 0;
            (*client).ps.fd.forceMindtrickTargetIndex3 = 0;
            (*client).ps.fd.forceMindtrickTargetIndex4 = 0;
        }
        FP_SEE => {
            if was_active & (1 << FP_SEE) != 0 {
                G_MuteSound(
                    (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_5 - 50) as usize],
                    CHAN_VOICE,
                );
            }
        }
        FP_GRIP => {
            (*client).ps.fd.forceGripUseTime = (*addr_of!(level)).time + 3000;
            let grip_ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*client).ps.fd.forceGripEntityNum as usize);
            if (*client).ps.fd.forcePowerLevel[FP_GRIP as usize] > FORCE_LEVEL_1
                && !(*grip_ent).client.is_null()
                && (*grip_ent).health > 0
                && (*grip_ent).inuse != 0
                // C: `(level.time - forceGripStarted) > 500` — level.time (int) minus a
                // `float` field promotes the whole expression to float, so the compare is
                // done in f32 (500 promoted to 500.0f), reproduced verbatim here.
                && ((*addr_of!(level)).time as f32 - (*(*grip_ent).client).ps.fd.forceGripStarted)
                    > 500.0
            {
                // if we had our throat crushed in for more than half a second, gasp for air when we're let go
                if was_active & (1 << FP_GRIP) != 0 {
                    G_EntitySound(grip_ent, CHAN_VOICE, G_SoundIndex("*gasp.wav"));
                }
            }

            if !(*grip_ent).client.is_null() && (*grip_ent).inuse != 0 {
                (*(*grip_ent).client).ps.forceGripChangeMovetype = PM_NORMAL;
            }

            if (*client).ps.forceHandExtend == HANDEXTEND_FORCE_HOLD {
                (*client).ps.forceHandExtendTime = 0;
            }

            (*client).ps.fd.forceGripEntityNum = ENTITYNUM_NONE;

            (*client).ps.powerups[PW_DISINT_4 as usize] = 0;
        }
        FP_LIGHTNING => {
            if (*client).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] < FORCE_LEVEL_2 {
                // don't do it again for 3 seconds, minimum... FIXME: this should be automatic once regeneration is slower (normal)
                (*client).ps.fd.forcePowerDebounce[FP_LIGHTNING as usize] =
                    (*addr_of!(level)).time + 3000;
            } else {
                (*client).ps.fd.forcePowerDebounce[FP_LIGHTNING as usize] =
                    (*addr_of!(level)).time + 1500;
            }
            if (*client).ps.forceHandExtend == HANDEXTEND_FORCE_HOLD {
                (*client).ps.forceHandExtendTime = 0; // reset hand position
            }

            (*client).ps.activeForcePass = 0;
        }
        FP_RAGE => {
            (*client).ps.fd.forceRageRecoveryTime = (*addr_of!(level)).time + 10000;
            if was_active & (1 << FP_RAGE) != 0 {
                G_MuteSound(
                    (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_3 - 50) as usize],
                    CHAN_VOICE,
                );
            }
        }
        FP_ABSORB => {
            if was_active & (1 << FP_ABSORB) != 0 {
                G_MuteSound(
                    (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_3 - 50) as usize],
                    CHAN_VOICE,
                );
            }
        }
        FP_PROTECT => {
            if was_active & (1 << FP_PROTECT) != 0 {
                G_MuteSound(
                    (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_3 - 50) as usize],
                    CHAN_VOICE,
                );
            }
        }
        FP_DRAIN => {
            // C: case FP_DRAIN falls through to `default: break;` (the missing break is
            // harmless — `default` is empty), so this arm carries the FP_DRAIN body only.
            if (*client).ps.fd.forcePowerLevel[FP_DRAIN as usize] < FORCE_LEVEL_2 {
                // don't do it again for 3 seconds, minimum...
                (*client).ps.fd.forcePowerDebounce[FP_DRAIN as usize] =
                    (*addr_of!(level)).time + 3000;
            } else {
                (*client).ps.fd.forcePowerDebounce[FP_DRAIN as usize] =
                    (*addr_of!(level)).time + 1500;
            }

            if (*client).ps.forceHandExtend == HANDEXTEND_FORCE_HOLD {
                (*client).ps.forceHandExtendTime = 0; // reset hand position
            }

            (*client).ps.activeForcePass = 0;
        }
        _ => {}
    }
}

/// `void WP_InitForcePowers( gentity_t *ent )` (w_force.c:147) — (re)initialise a client's
/// force-power loadout from its userinfo `forcepowers` string (or, for bots, from the bot's
/// personality `forceinfo`). Registers the loop-sound config-string indices, zeroes the known
/// powers, then for `GT_SIEGE` classes loads the class force levels and bails; otherwise parses
/// rank/side/per-power levels out of the string (legalising it against the server caps via
/// [`BG_LegalizedForcePowers`]), broadcasts the free-saber / force-disable `EV_SET_*` events,
/// nudges new/over-cap clients into the spectator profile menu, prunes invalid known bits, and
/// fixes up the selected power.
///
/// The `EVENT_FORCE_RANK` `#ifdef` blocks (the broadcast `EV_GIVE_NEW_RANK` temp-entities) are
/// excluded — `EVENT_FORCE_RANK` is undefined in the JKA build, so the `#else` `nfr`
/// server-command path is the live one and is what we port. The over-cap notice fires when
/// `warnClient || !sess.setForce` and the gametype is neither `GT_HOLOCRON` nor `GT_JEDIMASTER`;
/// the maxRank clamp (`>= NUM_FORCE_MASTERY_LEVELS` → re-`Cvar_Set` `g_maxForceRank`) is live.
/// The commented-out `g_forcePowerDisable`/duel/`warnClientLimit` blocks are dead in the C
/// original and carried over as comments.
///
/// No oracle — engine syscalls (`trap_GetUserinfo`/`trap_SendServerCommand`) + entity/playerState
/// mutation + temp-entity spawns (the `ClientUserinfoChanged` / w_force activator precedent).
///
/// # Safety
/// `ent` may be NULL or clientless (both are checked). `g_entities`/`level` and the client
/// system must be initialised; for bots `botstates[ent->s.number]` is read when `SVF_BOT` is set.
pub unsafe fn WP_InitForcePowers(ent: *mut gentity_t) {
    let mut i: c_int;
    let mut i_r: c_int;
    let mut maxRank: c_int = (*addr_of!(g_maxForceRank)).integer;
    let warnClient: qboolean;
    let warnClientLimit: qboolean = QFALSE;
    let mut userinfo = [0 as c_char; MAX_INFO_STRING];
    let mut forcePowers = [0 as c_char; 256];
    let mut readBuf = [0 as c_char; 256];
    let mut lastFPKnown: c_int = -1;
    let mut didEvent: qboolean = QFALSE;

    if maxRank == 0 {
        //if server has no max rank, default to max (50)
        maxRank = FORCE_MASTERY_JEDI_MASTER;
    } else if maxRank >= NUM_FORCE_MASTERY_LEVELS {
        //ack, prevent user from being dumb
        maxRank = FORCE_MASTERY_JEDI_MASTER;
        trap::Cvar_Set("g_maxForceRank", &format!("{}", maxRank));
    }

    /*
    if (g_forcePowerDisable.integer)
    {
        maxRank = FORCE_MASTERY_UNINITIATED;
    }
    */
    //rww - don't do this

    if ent.is_null() || (*ent).client.is_null() {
        return;
    }

    let client = (*ent).client;

    (*client).ps.fd.saberAnimLevel = (*client).sess.saberLevel;

    if (*client).ps.fd.saberAnimLevel < FORCE_LEVEL_1 || (*client).ps.fd.saberAnimLevel > FORCE_LEVEL_3
    {
        (*client).ps.fd.saberAnimLevel = FORCE_LEVEL_1;
    }

    if SPEED_LOOP_SOUND == 0 {
        //so that the client configstring is already modified with this when we need it
        SPEED_LOOP_SOUND = G_SoundIndex("sound/weapons/force/speedloop.wav");
    }

    if RAGE_LOOP_SOUND == 0 {
        RAGE_LOOP_SOUND = G_SoundIndex("sound/weapons/force/rageloop.wav");
    }

    if ABSORB_LOOP_SOUND == 0 {
        ABSORB_LOOP_SOUND = G_SoundIndex("sound/weapons/force/absorbloop.wav");
    }

    if PROTECT_LOOP_SOUND == 0 {
        PROTECT_LOOP_SOUND = G_SoundIndex("sound/weapons/force/protectloop.wav");
    }

    if SEE_LOOP_SOUND == 0 {
        SEE_LOOP_SOUND = G_SoundIndex("sound/weapons/force/seeloop.wav");
    }

    if YSALAMIRI_LOOP_SOUND == 0 {
        YSALAMIRI_LOOP_SOUND = G_SoundIndex("sound/player/nullifyloop.wav");
    }

    if (*ent).s.eType == ET_NPC {
        //just stop here then.
        return;
    }

    i = 0;
    while (i as usize) < NUM_FORCE_POWERS {
        (*client).ps.fd.forcePowerLevel[i as usize] = 0;
        (*client).ps.fd.forcePowersKnown &= !(1 << i);
        i += 1;
    }

    (*client).ps.fd.forcePowerSelected = -1;

    (*client).ps.fd.forceSide = 0;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*client).siegeClass != -1 {
        //Then use the powers for this class, and skip all this nonsense.
        i = 0;

        while (i as usize) < NUM_FORCE_POWERS {
            (*client).ps.fd.forcePowerLevel[i as usize] =
                (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].forcePowerLevels[i as usize];

            if (*client).ps.fd.forcePowerLevel[i as usize] == 0 {
                (*client).ps.fd.forcePowersKnown &= !(1 << i);
            } else {
                (*client).ps.fd.forcePowersKnown |= 1 << i;
            }
            i += 1;
        }

        if (*client).sess.setForce == QFALSE {
            //bring up the class selection menu
            trap::SendServerCommand(ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int, "scl");
        }
        (*client).sess.setForce = QTRUE;

        return;
    }

    if (*ent).s.eType == ET_NPC && (*ent).s.number >= MAX_CLIENTS as c_int {
        //rwwFIXMEFIXME: Temp
        strcpy(userinfo.as_mut_ptr(), c"forcepowers\\7-1-333003000313003120".as_ptr());
    } else {
        // trap_GetUserinfo( ent->s.number, userinfo, sizeof( userinfo ) );
        let info = trap::GetUserinfo((*ent).s.number);
        let bytes = info.as_bytes();
        let n = bytes.len().min(MAX_INFO_STRING - 1);
        for k in 0..n {
            userinfo[k] = bytes[k] as c_char;
        }
        userinfo[n] = 0;
    }

    Q_strncpyz(
        forcePowers.as_mut_ptr(),
        Info_ValueForKey(userinfo.as_ptr(), c"forcepowers".as_ptr()),
        forcePowers.len() as c_int,
    );

    if (*ent).r.svFlags & SVF_BOT != 0 && !(*addr_of!(botstates))[(*ent).s.number as usize].is_null()
    {
        //if it's a bot just copy the info directly from its personality
        Com_sprintf(
            forcePowers.as_mut_ptr(),
            forcePowers.len() as c_int,
            format_args!("{}", Sz((*(*addr_of!(botstates))[(*ent).s.number as usize]).forceinfo.as_ptr())),
        );
    }

    //rww - parse through the string manually and eat out all the appropriate data
    i = 0;

    if (*addr_of!(g_forceBasedTeams)).integer != 0 {
        if (*client).sess.sessionTeam == TEAM_RED {
            warnClient = (BG_LegalizedForcePowers(
                forcePowers.as_mut_ptr(),
                maxRank,
                HasSetSaberOnly(),
                FORCE_DARKSIDE,
                (*addr_of!(g_gametype)).integer,
                (*addr_of!(g_forcePowerDisable)).integer,
            ) == QFALSE) as qboolean;
        } else if (*client).sess.sessionTeam == TEAM_BLUE {
            warnClient = (BG_LegalizedForcePowers(
                forcePowers.as_mut_ptr(),
                maxRank,
                HasSetSaberOnly(),
                FORCE_LIGHTSIDE,
                (*addr_of!(g_gametype)).integer,
                (*addr_of!(g_forcePowerDisable)).integer,
            ) == QFALSE) as qboolean;
        } else {
            warnClient = (BG_LegalizedForcePowers(
                forcePowers.as_mut_ptr(),
                maxRank,
                HasSetSaberOnly(),
                0,
                (*addr_of!(g_gametype)).integer,
                (*addr_of!(g_forcePowerDisable)).integer,
            ) == QFALSE) as qboolean;
        }
    } else {
        warnClient = (BG_LegalizedForcePowers(
            forcePowers.as_mut_ptr(),
            maxRank,
            HasSetSaberOnly(),
            0,
            (*addr_of!(g_gametype)).integer,
            (*addr_of!(g_forcePowerDisable)).integer,
        ) == QFALSE) as qboolean;
    }

    i_r = 0;
    while forcePowers[i as usize] != 0 && forcePowers[i as usize] != b'-' as c_char {
        readBuf[i_r as usize] = forcePowers[i as usize];
        i_r += 1;
        i += 1;
    }
    readBuf[i_r as usize] = 0;
    //THE RANK
    (*client).ps.fd.forceRank = atoi(readBuf.as_ptr());
    i += 1;

    i_r = 0;
    while forcePowers[i as usize] != 0 && forcePowers[i as usize] != b'-' as c_char {
        readBuf[i_r as usize] = forcePowers[i as usize];
        i_r += 1;
        i += 1;
    }
    readBuf[i_r as usize] = 0;
    //THE SIDE
    (*client).ps.fd.forceSide = atoi(readBuf.as_ptr());
    i += 1;

    if (*addr_of!(g_gametype)).integer != GT_SIEGE
        && (*ent).r.svFlags & SVF_BOT != 0
        && !(*addr_of!(botstates))[(*ent).s.number as usize].is_null()
    {
        //hmm..I'm going to cheat here.
        let oldI = i;
        i_r = 0;
        while forcePowers[i as usize] != 0
            && forcePowers[i as usize] != b'\n' as c_char
            && (i_r as usize) < NUM_FORCE_POWERS
        {
            if (*client).ps.fd.forceSide == FORCE_LIGHTSIDE {
                if i_r == FP_ABSORB {
                    forcePowers[i as usize] = b'3' as c_char;
                }
                if (*(*addr_of!(botstates))[(*ent).s.number as usize]).settings.skill >= 4.0 {
                    //cheat and give them more stuff
                    if i_r == FP_HEAL {
                        forcePowers[i as usize] = b'3' as c_char;
                    } else if i_r == FP_PROTECT {
                        forcePowers[i as usize] = b'3' as c_char;
                    }
                }
            } else if (*client).ps.fd.forceSide == FORCE_DARKSIDE
                && (*(*addr_of!(botstates))[(*ent).s.number as usize]).settings.skill >= 4.0
            {
                if i_r == FP_GRIP {
                    forcePowers[i as usize] = b'3' as c_char;
                } else if i_r == FP_LIGHTNING {
                    forcePowers[i as usize] = b'3' as c_char;
                } else if i_r == FP_RAGE {
                    forcePowers[i as usize] = b'3' as c_char;
                } else if i_r == FP_DRAIN {
                    forcePowers[i as usize] = b'3' as c_char;
                }
            }

            if i_r == FP_PUSH {
                forcePowers[i as usize] = b'3' as c_char;
            } else if i_r == FP_PULL {
                forcePowers[i as usize] = b'3' as c_char;
            }

            i += 1;
            i_r += 1;
        }
        i = oldI;
    }

    i_r = 0;
    while forcePowers[i as usize] != 0
        && forcePowers[i as usize] != b'\n' as c_char
        && (i_r as usize) < NUM_FORCE_POWERS
    {
        readBuf[0] = forcePowers[i as usize];
        readBuf[1] = 0;

        (*client).ps.fd.forcePowerLevel[i_r as usize] = atoi(readBuf.as_ptr());
        if (*client).ps.fd.forcePowerLevel[i_r as usize] != 0 {
            (*client).ps.fd.forcePowersKnown |= 1 << i_r;
        } else {
            (*client).ps.fd.forcePowersKnown &= !(1 << i_r);
        }
        i += 1;
        i_r += 1;
    }
    //THE POWERS

    if (*ent).s.eType != ET_NPC {
        if HasSetSaberOnly() != QFALSE {
            let te = G_TempEntity(&vec3_origin, EV_SET_FREE_SABER);
            (*te).r.svFlags |= SVF_BROADCAST;
            (*te).s.eventParm = 1;
        } else {
            let te = G_TempEntity(&vec3_origin, EV_SET_FREE_SABER);
            (*te).r.svFlags |= SVF_BROADCAST;
            (*te).s.eventParm = 0;
        }

        if (*addr_of!(g_forcePowerDisable)).integer != 0 {
            let te = G_TempEntity(&vec3_origin, EV_SET_FORCE_DISABLE);
            (*te).r.svFlags |= SVF_BROADCAST;
            (*te).s.eventParm = 1;
        } else {
            let te = G_TempEntity(&vec3_origin, EV_SET_FORCE_DISABLE);
            (*te).r.svFlags |= SVF_BROADCAST;
            (*te).s.eventParm = 0;
        }
    }

    //rww - It seems we currently want to always do this, even if the player isn't exceeding the max
    //rank, so..
    //	if (g_gametype.integer == GT_DUEL || g_gametype.integer == GT_POWERDUEL)
    //	{ //totally messes duel up to force someone into spec mode, and besides, each "round" is
    //counted as a full restart
    //		ent->client->sess.setForce = qtrue;
    //	}

    if (*ent).s.eType == ET_NPC {
        (*client).sess.setForce = QTRUE;
    } else if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        if (*client).sess.setForce == QFALSE {
            (*client).sess.setForce = QTRUE;
            //bring up the class selection menu
            trap::SendServerCommand(ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int, "scl");
        }
    } else {
        if warnClient != QFALSE || (*client).sess.setForce == QFALSE {
            //the client's rank is too high for the server and has been autocapped, so tell them
            if (*addr_of!(g_gametype)).integer != GT_HOLOCRON
                && (*addr_of!(g_gametype)).integer != GT_JEDIMASTER
            {
                // #ifdef EVENT_FORCE_RANK (undefined) — broadcast EV_GIVE_NEW_RANK omitted
                didEvent = QTRUE;

                //				if (!(ent->r.svFlags & SVF_BOT) && g_gametype.integer != GT_DUEL && g_gametype.integer != GT_POWERDUEL && ent->s.eType != ET_NPC)
                if (*ent).r.svFlags & SVF_BOT == 0 && (*ent).s.eType != ET_NPC {
                    if (*addr_of!(g_teamAutoJoin)).integer == 0 {
                        //Make them a spectator so they can set their powerups up without being bothered.
                        (*client).sess.sessionTeam = TEAM_SPECTATOR;
                        (*client).sess.spectatorState = SPECTATOR_FREE;
                        (*client).sess.spectatorClient = 0;

                        (*client).pers.teamState.state = TEAM_BEGIN;
                        trap::SendServerCommand(
                            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                            "spc",
                        ); // Fire up the profile menu
                    }
                }

                //Event isn't very reliable, I made it a string. This way I can send it to just one
                //client also, as opposed to making a broadcast event.
                trap::SendServerCommand(
                    (*ent).s.number,
                    &format!("nfr {} {} {}", maxRank, 1, (*client).sess.sessionTeam),
                );
                //Arg1 is new max rank, arg2 is non-0 if force menu should be shown, arg3 is the current team
            }
            (*client).sess.setForce = QTRUE;
        }

        if didEvent == QFALSE {
            // #ifdef EVENT_FORCE_RANK (undefined) — broadcast EV_GIVE_NEW_RANK omitted
            trap::SendServerCommand(
                (*ent).s.number,
                &format!("nfr {} {} {}", maxRank, 0, (*client).sess.sessionTeam),
            );
        }

        if warnClientLimit != QFALSE {
            //the server has one or more force powers disabled and the client is using them in his config
            //trap_SendServerCommand(ent-g_entities, va("print \"The server has one or more force powers that you have chosen disabled.\nYou will not be able to use the disable force power(s) while playing on this server.\n\""));
        }
    }

    i = 0;
    while (i as usize) < NUM_FORCE_POWERS {
        if (*client).ps.fd.forcePowersKnown & (1 << i) != 0
            && (*client).ps.fd.forcePowerLevel[i as usize] == 0
        {
            //err..
            (*client).ps.fd.forcePowersKnown &= !(1 << i);
        } else if i != FP_LEVITATION
            && i != FP_SABER_OFFENSE
            && i != FP_SABER_DEFENSE
            && i != FP_SABERTHROW
        {
            lastFPKnown = i;
        }

        i += 1;
    }

    if (*client).ps.fd.forcePowersKnown & (*client).sess.selectedFP != 0 {
        (*client).ps.fd.forcePowerSelected = (*client).sess.selectedFP;
    }

    // C: `1 << forcePowerSelected`. `forcePowerSelected` is the -1 "none selected"
    // sentinel (set at WP_InitForcePowers entry / from sess.selectedFP), and the C relies
    // on x86's hardware shift-count masking (`-1 & 31 == 31`, a bit never set in
    // forcePowersKnown). `wrapping_shl` reproduces that masking bit-exactly; a plain `<<`
    // panics under debug overflow-checks on the negative/out-of-range shift amount.
    if (*client).ps.fd.forcePowersKnown & 1i32.wrapping_shl((*client).ps.fd.forcePowerSelected as u32)
        == 0
    {
        if lastFPKnown != -1 {
            (*client).ps.fd.forcePowerSelected = lastFPKnown;
        } else {
            (*client).ps.fd.forcePowerSelected = 0;
        }
    }

    // NOTE (bug-faithful): `i` is already NUM_FORCE_POWERS from the loop above, so this loop
    // never executes (matches the C original — forcePowerBaseLevel is left untouched here).
    while (i as usize) < NUM_FORCE_POWERS {
        (*client).ps.fd.forcePowerBaseLevel[i as usize] = (*client).ps.fd.forcePowerLevel[i as usize];
        i += 1;
    }
    (*client).ps.fd.forceUsingAdded = 0;
}

/// `void WP_SpawnInitForcePowers( gentity_t *ent )` (w_force.c:579) — reset a client's
/// force-power state at spawn. Zeroes the saber attack-chain counter, stops every still-active
/// power via [`WP_ForcePowerStop`], clears `forceDeactivateAll`, refills `forcePower`/
/// `forcePowerMax` to [`FORCE_POWER_MAX`], resets the regen/grip/heal/rage/drain/jump timers and
/// the mind-trick target indices, zeroes the holocron bits + per-power `holocronsCarried`
/// array, and in `GT_HOLOCRON` resets every `forcePowerLevel` to `FORCE_LEVEL_0` (restoring
/// the saber offense/defense floor when saber-only is set), zeroes all per-power
/// debounce/duration arrays, prunes any `forcePowersKnown` bit whose level is 0, and — in
/// `GT_SIEGE` with a chosen `siegeClass` — loads that class's `forcePowerLevels`
/// (clearing/setting each known bit accordingly).
///
/// No oracle — pure `playerState` (`ps.fd`) mutation reaching into `WP_ForcePowerStop`'s
/// gentity/sound-trap side effects (the entity/trap control-flow precedent).
///
/// # Safety
/// `ent` must be a valid entity with a non-NULL `client`. `g_entities`/`level` must be
/// initialised (the grip teardown in `WP_ForcePowerStop` indexes `g_entities`).
pub unsafe fn WP_SpawnInitForcePowers(ent: *mut gentity_t) {
    let mut i: c_int;
    let client = (*ent).client;

    (*client).ps.saberAttackChainCount = 0;

    i = 0;
    while (i as usize) < NUM_FORCE_POWERS {
        if (*client).ps.fd.forcePowersActive & (1 << i) != 0 {
            WP_ForcePowerStop(ent, i);
        }
        i += 1;
    }

    (*client).ps.fd.forceDeactivateAll = 0;

    (*client).ps.fd.forcePower = FORCE_POWER_MAX;
    (*client).ps.fd.forcePowerMax = FORCE_POWER_MAX;
    (*client).ps.fd.forcePowerRegenDebounceTime = 0;
    (*client).ps.fd.forceGripEntityNum = ENTITYNUM_NONE;
    (*client).ps.fd.forceMindtrickTargetIndex = 0;
    (*client).ps.fd.forceMindtrickTargetIndex2 = 0;
    (*client).ps.fd.forceMindtrickTargetIndex3 = 0;
    (*client).ps.fd.forceMindtrickTargetIndex4 = 0;

    (*client).ps.holocronBits = 0;

    i = 0;
    while (i as usize) < NUM_FORCE_POWERS {
        (*client).ps.holocronsCarried[i as usize] = 0.0;
        i += 1;
    }

    if (*addr_of!(g_gametype)).integer == GT_HOLOCRON {
        i = 0;
        while (i as usize) < NUM_FORCE_POWERS {
            (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_0;
            i += 1;
        }

        if HasSetSaberOnly() != QFALSE {
            if (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] < FORCE_LEVEL_1 {
                (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] = FORCE_LEVEL_1;
            }
            if (*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] < FORCE_LEVEL_1 {
                (*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] = FORCE_LEVEL_1;
            }
        }
    }

    i = 0;
    while (i as usize) < NUM_FORCE_POWERS {
        (*client).ps.fd.forcePowerDebounce[i as usize] = 0;
        (*client).ps.fd.forcePowerDuration[i as usize] = 0;
        i += 1;
    }

    (*client).ps.fd.forcePowerRegenDebounceTime = 0;
    (*client).ps.fd.forceJumpZStart = 0.0;
    (*client).ps.fd.forceJumpCharge = 0.0;
    (*client).ps.fd.forceJumpSound = 0;
    (*client).ps.fd.forceGripDamageDebounceTime = 0;
    (*client).ps.fd.forceGripBeingGripped = 0.0;
    (*client).ps.fd.forceGripCripple = 0;
    (*client).ps.fd.forceGripUseTime = 0;
    (*client).ps.fd.forceGripSoundTime = 0.0;
    (*client).ps.fd.forceGripStarted = 0.0;
    (*client).ps.fd.forceHealTime = 0;
    (*client).ps.fd.forceHealAmount = 0;
    (*client).ps.fd.forceRageRecoveryTime = 0;
    (*client).ps.fd.forceDrainEntNum = ENTITYNUM_NONE;
    (*client).ps.fd.forceDrainTime = 0.0;

    i = 0;
    while (i as usize) < NUM_FORCE_POWERS {
        if (*client).ps.fd.forcePowersKnown & (1 << i) != 0
            && (*client).ps.fd.forcePowerLevel[i as usize] == 0
        {
            // make sure all known powers are cleared if we have level 0 in them
            (*client).ps.fd.forcePowersKnown &= !(1 << i);
        }
        i += 1;
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*client).siegeClass != -1 {
        // Then use the powers for this class.
        i = 0;
        while (i as usize) < NUM_FORCE_POWERS {
            (*client).ps.fd.forcePowerLevel[i as usize] = (*addr_of!(bgSiegeClasses))
                [(*client).siegeClass as usize]
                .forcePowerLevels[i as usize];

            if (*client).ps.fd.forcePowerLevel[i as usize] == 0 {
                (*client).ps.fd.forcePowersKnown &= !(1 << i);
            } else {
                (*client).ps.fd.forcePowersKnown |= 1 << i;
            }
            i += 1;
        }
    }
}

/// `void WP_ForcePowerRegenerate( gentity_t *self, int overrideAmt )` (w_force.c:1006) —
/// called on a regular interval to regenerate force power. No-op for clientless entities.
/// Adds `overrideAmt` to `forcePower` when nonzero (a custom regen amount), otherwise adds 1,
/// then caps the result at `forcePowerMax` (default 100).
///
/// No oracle — mutates the live gentity/playerState (`ps.fd.forcePower`) through a `gentity_t`
/// pointer (the same entity/playerState-mutation precedent as the surrounding w_force leaves).
///
/// # Safety
/// `self_` must be a valid entity (its `client` is checked for NULL before use).
pub unsafe fn WP_ForcePowerRegenerate(self_: *mut gentity_t, override_amt: c_int) {
    if (*self_).client.is_null() {
        return;
    }

    let client = (*self_).client;

    if override_amt != 0 {
        //custom regen amount
        (*client).ps.fd.forcePower += override_amt;
    } else {
        //otherwise, just 1
        (*client).ps.fd.forcePower += 1;
    }

    if (*client).ps.fd.forcePower > (*client).ps.fd.forcePowerMax {
        //cap it off at the max (default 100)
        (*client).ps.fd.forcePower = (*client).ps.fd.forcePowerMax;
    }
}

/// `qboolean WP_ForcePowerAvailable( gentity_t *self, forcePowers_t forcePower, int overrideAmt )`
/// (w_force.c:781) — can `self` *afford* `forcePower` right now. The drain cost is `overrideAmt`
/// when nonzero, else `forcePowerNeeded[level][power]`. Always available if the power is already
/// active (being toggled off), if it's `FP_LEVITATION`, if the cost is zero, or — for the
/// duration-based `FP_DRAIN`/`FP_LIGHTNING` — when `forcePower >= 25`; otherwise gated on having
/// enough `forcePower`.
///
/// No oracle — reads the live gentity/playerState + the `forcePowerNeeded` static (the force
/// predicate precedent; same shape as the other w_force leaves).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`.
pub unsafe fn WP_ForcePowerAvailable(
    self_: *mut gentity_t,
    force_power: c_int,
    override_amt: c_int,
) -> qboolean {
    let client = (*self_).client;
    let drain = if override_amt != 0 {
        override_amt
    } else {
        forcePowerNeeded[(*client).ps.fd.forcePowerLevel[force_power as usize] as usize]
            [force_power as usize]
    };

    if (*client).ps.fd.forcePowersActive & (1 << force_power) != 0 {
        //we're probably going to deactivate it..
        return QTRUE;
    }
    if force_power == FP_LEVITATION {
        return QTRUE;
    }
    if drain == 0 {
        return QTRUE;
    }
    if (force_power == FP_DRAIN || force_power == FP_LIGHTNING)
        && (*client).ps.fd.forcePower >= 25
    {
        //it's ok then, drain/lightning are actually duration
        return QTRUE;
    }
    if (*client).ps.fd.forcePower < drain {
        return QFALSE;
    }
    QTRUE
}

/// `qboolean WP_ForcePowerInUse( gentity_t *self, forcePowers_t forcePower )` (w_force.c:810) —
/// is `forcePower` currently active on `self` (its bit set in `forcePowersActive`).
///
/// No oracle — reads the live gentity/playerState (force predicate precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`.
pub unsafe fn WP_ForcePowerInUse(self_: *mut gentity_t, force_power: c_int) -> qboolean {
    let client = (*self_).client;
    if (*client).ps.fd.forcePowersActive & (1 << force_power) != 0 {
        //already using this power
        return QTRUE;
    }

    QFALSE
}

/// `qboolean WP_ForcePowerUsable( gentity_t *self, forcePowers_t forcePower )` (w_force.c:820) —
/// the full "may `self` invoke `forcePower` this instant" gate. Rejects ysalamiri, the dead,
/// spectators / followers / temp-spectators, anything `BG_CanUseFPNow` forbids, unknown powers,
/// already-active powers (except levitation), a just-jumped levitation, level-0 powers; under
/// `g_debugMelee` blocks offensive powers while stuck to a wall; and under `g_saberRestrictForce`
/// (or per-saber `forceRestrictions`) blocks powers that the wielded saber(s) forbid. Falls
/// through to [`WP_ForcePowerAvailable`] for the cost check.
///
/// No oracle — reads the live gentity/playerState, the `level` clock, two cvars, and calls
/// `BG_HasYsalamiri`/`BG_CanUseFPNow` (force predicate precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`.
pub unsafe fn WP_ForcePowerUsable(self_: *mut gentity_t, force_power: c_int) -> qboolean {
    let client = (*self_).client;

    if BG_HasYsalamiri((*addr_of!(g_gametype)).integer, &mut (*client).ps) != QFALSE {
        return QFALSE;
    }

    if (*self_).health <= 0
        || (*client).ps.stats[STAT_HEALTH as usize] <= 0
        || ((*client).ps.eFlags & EF_DEAD) != 0
    {
        return QFALSE;
    }

    if (*client).ps.pm_flags & PMF_FOLLOW != 0 {
        //specs can't use powers through people
        return QFALSE;
    }
    if (*client).sess.sessionTeam == TEAM_SPECTATOR {
        return QFALSE;
    }
    if (*client).tempSpectate >= (*addr_of!(level)).time {
        return QFALSE;
    }

    if BG_CanUseFPNow(
        (*addr_of!(g_gametype)).integer,
        &mut (*client).ps,
        (*addr_of!(level)).time,
        force_power,
    ) == QFALSE
    {
        return QFALSE;
    }

    if (*client).ps.fd.forcePowersKnown & (1 << force_power) == 0 {
        //don't know this power
        return QFALSE;
    }

    if (*client).ps.fd.forcePowersActive & (1 << force_power) != 0 {
        //already using this power
        if force_power != FP_LEVITATION {
            return QFALSE;
        }
    }

    if force_power == FP_LEVITATION && (*client).fjDidJump != QFALSE {
        return QFALSE;
    }

    if (*client).ps.fd.forcePowerLevel[force_power as usize] == 0 {
        return QFALSE;
    }

    if (*addr_of!(g_debugMelee)).integer != 0
        && ((*client).ps.pm_flags & PMF_STUCK_TO_WALL) != 0
    {
        //no offensive force powers when stuck to wall
        match force_power {
            FP_GRIP | FP_LIGHTNING | FP_DRAIN | FP_SABER_OFFENSE | FP_SABER_DEFENSE
            | FP_SABERTHROW => return QFALSE,
            _ => {}
        }
    }

    if (*client).ps.saberHolstered == 0 {
        if (*client).saber[0].saberFlags & SFL_TWO_HANDED != 0
            && (*addr_of!(g_saberRestrictForce)).integer != 0
        {
            match force_power {
                FP_PUSH | FP_PULL | FP_TELEPATHY | FP_GRIP | FP_LIGHTNING | FP_DRAIN => {
                    return QFALSE
                }
                _ => {}
            }
        }

        if (*client).saber[0].saberFlags & SFL_TWO_HANDED != 0 || (*client).saber[0].model[0] != 0 {
            //this saber requires the use of two hands OR our other hand is using an active saber too
            if (*client).saber[0].forceRestrictions & (1 << force_power) != 0 {
                //this power is verboten when using this saber
                return QFALSE;
            }
        }

        if (*client).saber[0].model[0] != 0 {
            //both sabers on
            if (*addr_of!(g_saberRestrictForce)).integer != 0 {
                match force_power {
                    FP_PUSH | FP_PULL | FP_TELEPATHY | FP_GRIP | FP_LIGHTNING | FP_DRAIN => {
                        return QFALSE
                    }
                    _ => {}
                }
            }
            if (*client).saber[1].forceRestrictions & (1 << force_power) != 0 {
                //this power is verboten when using this saber
                return QFALSE;
            }
        }
    }
    WP_ForcePowerAvailable(self_, force_power, 0) // OVERRIDEFIXME
}

/// `qboolean G_IsMindTricked( forcedata_t *fd, int client )` (w_force.c:4162) — is `client`
/// (a client slot 0..63) hidden from the entity whose force-data is `fd` by an active
/// mind-trick. The 64 possible targets are packed into four 16-bit index words
/// (`forceMindtrickTargetIndex`/`..2`/`..3`/`..4`); pick the word for `client`'s 16-slot bucket,
/// subtract that bucket's base, and test the bit. NULL `fd` → false.
///
/// # Safety
/// `fd` may be NULL (the NULL-guard returns `QFALSE`); otherwise it must point to a valid
/// `forcedata_t`.
pub unsafe fn G_IsMindTricked(fd: *const forcedata_t, client: c_int) -> qboolean {
    let check_in: c_int;
    let mut sub: c_int = 0;

    if fd.is_null() {
        return QFALSE;
    }

    let trick_index1 = (*fd).forceMindtrickTargetIndex;
    let trick_index2 = (*fd).forceMindtrickTargetIndex2;
    let trick_index3 = (*fd).forceMindtrickTargetIndex3;
    let trick_index4 = (*fd).forceMindtrickTargetIndex4;

    if client > 47 {
        check_in = trick_index4;
        sub = 48;
    } else if client > 31 {
        check_in = trick_index3;
        sub = 32;
    } else if client > 15 {
        check_in = trick_index2;
        sub = 16;
    } else {
        check_in = trick_index1;
    }

    if check_in & (1 << (client - sub)) != 0 {
        return QTRUE;
    }

    QFALSE
}

/// `void HolocronUpdate(gentity_t *self)` (w_force.c:4870) — keep holocron status updated in
/// holocron (`GT_HOLOCRON`) mode. Refreshes the `g_MaxHolocronCarry` cvar mirror, then for every
/// force power: if the player carries that holocron, set its known/holocron bits and pin its level
/// to `FORCE_LEVEL_3`; otherwise clear its level, drop the holocron bit (faithful `-= (1 << i)`),
/// strip it from `forcePowersKnown` and stop it if active (except the always-kept `FP_LEVITATION`
/// and `FP_SABER_OFFENSE`), and reset its floor — levitation and saber-offense to `noHRank`
/// (floored at `FORCE_LEVEL_1`), everything else to `FORCE_LEVEL_0`. Finally, if `HasSetSaberOnly`,
/// floor saber offense/defense at `FORCE_LEVEL_1`. `noHRank` is the JKA dead-local: initialized to
/// `0`, clamped to `[FORCE_LEVEL_0, FORCE_LEVEL_3]` and never reassigned, so it is always
/// `FORCE_LEVEL_0` — its `>= FORCE_LEVEL_1` branches are unreachable; preserved verbatim for
/// fidelity.
///
/// No oracle — mutates a live `gentity_t`/`gclient_t`, refreshes a cvar via `trap_Cvar_Update`,
/// and calls `WP_ForcePowerStop` (entity/trap control-flow precedent).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a non-NULL `client`.
pub unsafe fn HolocronUpdate(self_: *mut gentity_t) {
    //keep holocron status updated in holocron mode
    let mut i: c_int = 0;
    let mut noHRank: c_int = 0;

    if noHRank < FORCE_LEVEL_0 {
        noHRank = FORCE_LEVEL_0;
    }
    if noHRank > FORCE_LEVEL_3 {
        noHRank = FORCE_LEVEL_3;
    }

    trap::Cvar_Update(&mut *core::ptr::addr_of_mut!(g_MaxHolocronCarry));

    let client = (*self_).client;
    while (i as usize) < NUM_FORCE_POWERS {
        if (*client).ps.holocronsCarried[i as usize] != 0.0 {
            //carrying it, make sure we have the power
            (*client).ps.holocronBits |= 1 << i;
            (*client).ps.fd.forcePowersKnown |= 1 << i;
            (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_3;
        } else {
            //otherwise, make sure the power is cleared from us
            (*client).ps.fd.forcePowerLevel[i as usize] = 0;
            if (*client).ps.holocronBits & (1 << i) != 0 {
                (*client).ps.holocronBits -= 1 << i;
            }

            if (*client).ps.fd.forcePowersKnown & (1 << i) != 0
                && i != FP_LEVITATION
                && i != FP_SABER_OFFENSE
            {
                (*client).ps.fd.forcePowersKnown -= 1 << i;
            }

            if (*client).ps.fd.forcePowersActive & (1 << i) != 0
                && i != FP_LEVITATION
                && i != FP_SABER_OFFENSE
            {
                WP_ForcePowerStop(self_, i);
            }

            if i == FP_LEVITATION {
                if noHRank >= FORCE_LEVEL_1 {
                    (*client).ps.fd.forcePowerLevel[i as usize] = noHRank;
                } else {
                    (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_1;
                }
            } else if i == FP_SABER_OFFENSE {
                (*client).ps.fd.forcePowersKnown |= 1 << i;

                if noHRank >= FORCE_LEVEL_1 {
                    (*client).ps.fd.forcePowerLevel[i as usize] = noHRank;
                } else {
                    (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_1;
                }
            } else {
                (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_0;
            }
        }

        i += 1;
    }

    if HasSetSaberOnly() != QFALSE {
        //if saberonly, we get these powers no matter what (still need the holocrons for level 3)
        if (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] < FORCE_LEVEL_1 {
            (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] = FORCE_LEVEL_1;
        }
        if (*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] < FORCE_LEVEL_1 {
            (*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] = FORCE_LEVEL_1;
        }
    }
}

/// `qboolean WP_HasForcePowers( const playerState_t *ps )` (w_force.c:5015) — does `ps` know
/// any force power above its baseline. A power counts if its `forcePowerLevel` exceeds
/// `void JediMasterUpdate(gentity_t *self)` (w_force.c:4958) — keep Jedi-Master status updated
/// for the JM gametype. Refreshes the `g_MaxHolocronCarry` cvar mirror, then for every force
/// power: if the client `isJediMaster`, grant it (set the known bit, pin to `FORCE_LEVEL_3`) —
/// except the team powers (`FP_TEAM_HEAL`/`FP_TEAM_FORCE`/`FP_DRAIN`/`FP_ABSORB`), which are
/// stripped (`&= ~(1 << i)`) and zeroed as useless in JM, and `FP_TELEPATHY`, which is capped at
/// `FORCE_LEVEL_2` so the JM can't hide indefinitely. Otherwise (not the Jedi Master): for every
/// power except `FP_LEVITATION`, strip it from `forcePowersKnown` (faithful `-= (1 << i)`, not
/// bit-clear), stop it if active via `WP_ForcePowerStop`, and zero its level (`FORCE_LEVEL_0`);
/// levitation is held at `FORCE_LEVEL_1`.
///
/// No oracle — mutates a live `gentity_t`/`gclient_t`, refreshes a cvar via `trap_Cvar_Update`,
/// and calls `WP_ForcePowerStop` (entity/trap control-flow precedent).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a non-NULL `client`.
pub unsafe fn JediMasterUpdate(self_: *mut gentity_t) {
    //keep jedi master status updated for JM gametype
    let mut i: c_int = 0;

    trap::Cvar_Update(&mut *core::ptr::addr_of_mut!(g_MaxHolocronCarry));

    let client = (*self_).client;
    while (i as usize) < NUM_FORCE_POWERS {
        if (*client).ps.isJediMaster != QFALSE {
            (*client).ps.fd.forcePowersKnown |= 1 << i;
            (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_3;

            if i == FP_TEAM_HEAL || i == FP_TEAM_FORCE || i == FP_DRAIN || i == FP_ABSORB {
                //team powers are useless in JM, absorb is too because no one else has powers to absorb. Drain is just
                //relatively useless in comparison, because its main intent is not to heal, but rather to cripple others
                //by draining their force at the same time. And no one needs force in JM except the JM himself.
                (*client).ps.fd.forcePowersKnown &= !(1 << i);
                (*client).ps.fd.forcePowerLevel[i as usize] = 0;
            }

            if i == FP_TELEPATHY {
                //this decision was made because level 3 mindtrick allows the JM to just hide too much, and no one else has force
                //sight to counteract it. Since the JM himself is the focus of gameplay in this mode, having him hidden for large
                //durations is indeed a bad thing.
                (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_2;
            }
        } else {
            if ((*client).ps.fd.forcePowersKnown & (1 << i)) != 0 && i != FP_LEVITATION {
                (*client).ps.fd.forcePowersKnown -= 1 << i;
            }

            if ((*client).ps.fd.forcePowersActive & (1 << i)) != 0 && i != FP_LEVITATION {
                WP_ForcePowerStop(self_, i);
            }

            if i == FP_LEVITATION {
                (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_1;
            } else {
                (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_0;
            }
        }

        i += 1;
    }
}

/// `FORCE_LEVEL_0` — except `FP_LEVITATION`, which every Jedi has at level 1, so it counts only
/// above `FORCE_LEVEL_1`. NULL `ps` → false.
///
/// # Safety
/// `ps` may be NULL (the NULL-guard returns `QFALSE`); otherwise it must point to a valid
/// `playerState_t`.
pub unsafe fn WP_HasForcePowers(ps: *const playerState_t) -> qboolean {
    let mut i: c_int;
    if !ps.is_null() {
        i = 0;
        while (i as usize) < NUM_FORCE_POWERS {
            if i == FP_LEVITATION {
                if (*ps).fd.forcePowerLevel[i as usize] > FORCE_LEVEL_1 {
                    return QTRUE;
                }
            } else if (*ps).fd.forcePowerLevel[i as usize] > FORCE_LEVEL_0 {
                return QTRUE;
            }
            i += 1;
        }
    }
    QFALSE
}

/// `void WP_AddToClientBitflags(gentity_t *ent, int entNum)` (w_force.c:1301) — mark client
/// slot `entNum` (0..63) as mind-tricked *by* `ent`: the inverse write-side of `G_IsMindTricked`.
/// The 64 possible targets are packed into four 16-bit `ent->s.trickedentindex*` words; pick the
/// word for `entNum`'s 16-slot bucket, subtract the bucket base, and set the bit. NULL `ent` → no-op.
///
/// # Safety
/// `ent` may be NULL (the NULL-guard returns early); otherwise it must point to a valid
/// `gentity_t` whose `s.trickedentindex*` fields are mutated.
pub unsafe fn WP_AddToClientBitflags(ent: *mut gentity_t, ent_num: c_int) {
    if ent.is_null() {
        return;
    }

    if ent_num > 47 {
        (*ent).s.trickedentindex4 |= 1 << (ent_num - 48);
    } else if ent_num > 31 {
        (*ent).s.trickedentindex3 |= 1 << (ent_num - 32);
    } else if ent_num > 15 {
        (*ent).s.trickedentindex2 |= 1 << (ent_num - 16);
    } else {
        (*ent).s.trickedentindex |= 1 << ent_num;
    }
}

/// `void WP_AddAsMindtricked(forcedata_t *fd, int entNum)` (w_force.c:2509) — mark client slot
/// `entNum` (0..63) as mind-tricked in `fd`: the write-side of `G_IsMindTricked`'s read. The 64
/// possible targets are packed into four 16-bit `fd->forceMindtrickTargetIndex*` words; pick the
/// word for `entNum`'s 16-slot bucket, subtract the bucket base, and set the bit. NULL `fd` → no-op.
///
/// # Safety
/// `fd` may be NULL (the NULL-guard returns early); otherwise it must point to a valid
/// `forcedata_t` whose `forceMindtrickTargetIndex*` fields are mutated.
pub unsafe fn WP_AddAsMindtricked(fd: *mut forcedata_t, ent_num: c_int) {
    if fd.is_null() {
        return;
    }

    if ent_num > 47 {
        (*fd).forceMindtrickTargetIndex4 |= 1 << (ent_num - 48);
    } else if ent_num > 31 {
        (*fd).forceMindtrickTargetIndex3 |= 1 << (ent_num - 32);
    } else if ent_num > 15 {
        (*fd).forceMindtrickTargetIndex2 |= 1 << (ent_num - 16);
    } else {
        (*fd).forceMindtrickTargetIndex |= 1 << ent_num;
    }
}

/// `static void RemoveTrickedEnt(forcedata_t *fd, int client)` (w_force.c:4206) — clear client
/// slot `client` (0..63) from `fd`'s mind-trick target set: the bitwise mirror of
/// [`WP_AddAsMindtricked`], clearing instead of setting. The 64 possible targets are packed into
/// four 16-bit `fd->forceMindtrickTargetIndex*` words; pick the word for `client`'s 16-slot
/// bucket, subtract the bucket base, and clear the bit. NULL `fd` → no-op. C-`static` (file-local,
/// not public ABI); kept `pub` per this file's convention (visibility suppresses dead_code until
/// its only caller, the not-yet-ported `WP_UpdateMindtrickEnts`, lands).
///
/// # Safety
/// `fd` may be NULL (the NULL-guard returns early); otherwise it must point to a valid
/// `forcedata_t` whose `forceMindtrickTargetIndex*` fields are mutated.
pub unsafe fn RemoveTrickedEnt(fd: *mut forcedata_t, client: c_int) {
    if fd.is_null() {
        return;
    }

    if client > 47 {
        (*fd).forceMindtrickTargetIndex4 &= !(1 << (client - 48));
    } else if client > 31 {
        (*fd).forceMindtrickTargetIndex3 &= !(1 << (client - 32));
    } else if client > 15 {
        (*fd).forceMindtrickTargetIndex2 &= !(1 << (client - 16));
    } else {
        (*fd).forceMindtrickTargetIndex &= !(1 << client);
    }
}

/// `qboolean G_InGetUpAnim(playerState_t *ps)` (w_force.c:2977) — is the player mid-getup. A
/// pure predicate: returns `qtrue` if `ps->legsAnim` *or* `ps->torsoAnim` is any of the
/// knockdown-recovery anims — the plain getups (`BOTH_GETUP1..5`), the force getups
/// (`BOTH_FORCE_GETUP_F1/F2`, `BOTH_FORCE_GETUP_B1..B5`), or the getup rolls
/// (`BOTH_GETUP_BROLL_*`, `BOTH_GETUP_FROLL_*`) — else `qfalse`. The two switches are checked
/// in order (legs first, then torso), exactly as in retail.
///
/// # Safety
/// `ps` must point to a valid `playerState_t` (the C body dereferences it unconditionally).
pub unsafe fn G_InGetUpAnim(ps: *const playerState_t) -> qboolean {
    match (*ps).legsAnim {
        x if x == BOTH_GETUP1 as c_int
            || x == BOTH_GETUP2 as c_int
            || x == BOTH_GETUP3 as c_int
            || x == BOTH_GETUP4 as c_int
            || x == BOTH_GETUP5 as c_int
            || x == BOTH_FORCE_GETUP_F1 as c_int
            || x == BOTH_FORCE_GETUP_F2 as c_int
            || x == BOTH_FORCE_GETUP_B1 as c_int
            || x == BOTH_FORCE_GETUP_B2 as c_int
            || x == BOTH_FORCE_GETUP_B3 as c_int
            || x == BOTH_FORCE_GETUP_B4 as c_int
            || x == BOTH_FORCE_GETUP_B5 as c_int
            || x == BOTH_GETUP_BROLL_B as c_int
            || x == BOTH_GETUP_BROLL_F as c_int
            || x == BOTH_GETUP_BROLL_L as c_int
            || x == BOTH_GETUP_BROLL_R as c_int
            || x == BOTH_GETUP_FROLL_B as c_int
            || x == BOTH_GETUP_FROLL_F as c_int
            || x == BOTH_GETUP_FROLL_L as c_int
            || x == BOTH_GETUP_FROLL_R as c_int =>
        {
            return QTRUE;
        }
        _ => {}
    }

    match (*ps).torsoAnim {
        x if x == BOTH_GETUP1 as c_int
            || x == BOTH_GETUP2 as c_int
            || x == BOTH_GETUP3 as c_int
            || x == BOTH_GETUP4 as c_int
            || x == BOTH_GETUP5 as c_int
            || x == BOTH_FORCE_GETUP_F1 as c_int
            || x == BOTH_FORCE_GETUP_F2 as c_int
            || x == BOTH_FORCE_GETUP_B1 as c_int
            || x == BOTH_FORCE_GETUP_B2 as c_int
            || x == BOTH_FORCE_GETUP_B3 as c_int
            || x == BOTH_FORCE_GETUP_B4 as c_int
            || x == BOTH_FORCE_GETUP_B5 as c_int
            || x == BOTH_GETUP_BROLL_B as c_int
            || x == BOTH_GETUP_BROLL_F as c_int
            || x == BOTH_GETUP_BROLL_L as c_int
            || x == BOTH_GETUP_BROLL_R as c_int
            || x == BOTH_GETUP_FROLL_B as c_int
            || x == BOTH_GETUP_FROLL_F as c_int
            || x == BOTH_GETUP_FROLL_L as c_int
            || x == BOTH_GETUP_FROLL_R as c_int =>
        {
            return QTRUE;
        }
        _ => {}
    }

    QFALSE
}

/// `qboolean G_SpecialRollGetup(gentity_t *self)` (w_force.c:5039) — pick a knockdown-getup
/// roll from the player's pending move command. Reading `self->client->pers.cmd`, a pure
/// right/left strafe (`rightmove` non-zero, no `forwardmove`) or a pure forward/back press
/// (`forwardmove` non-zero, no `rightmove`) plays the matching `BOTH_GETUP_BROLL_{R,L,F,B}`
/// anim via [`G_SetAnim`] (`SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD`) and reports a roll.
/// Otherwise a bare `upmove` plays the force-jump pre-def sound and arms a hand-extend dodge
/// (`forceDodgeAnim = 2`, `forceHandExtendTime = level.time + 500`). On any roll it plays the
/// `*jump1.wav` voice and returns `qtrue`; else `qfalse`.
///
/// The original's first three branches keep the commented-out `upmove`/`weapon` guards inert,
/// exactly as in retail — preserved as comments for fidelity.
///
/// No oracle — gentity/playerState mutation + anim/sound side effects (the w_force entity/
/// control-flow leaf precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`. The entity/sound systems must be
/// initialised ([`G_SetAnim`], [`G_PreDefSound`], [`G_EntitySound`] reach into them).
pub unsafe fn G_SpecialRollGetup(self_: *mut gentity_t) -> qboolean {
    let mut rolled = QFALSE;

    /*
    if (self->client->ps.weapon != WP_SABER &&
        self->client->ps.weapon != WP_MELEE)
    { //can't do acrobatics without saber selected
        return qfalse;
    }
    */

    let client = (*self_).client;
    let cmd = &mut (*client).pers.cmd;

    if /* !cmd.upmove && */ cmd.rightmove > 0 && cmd.forwardmove == 0 {
        G_SetAnim(
            self_,
            cmd,
            SETANIM_BOTH,
            BOTH_GETUP_BROLL_R as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            0,
        );
        rolled = QTRUE;
    } else if /* !cmd.upmove && */ cmd.rightmove < 0 && cmd.forwardmove == 0 {
        G_SetAnim(
            self_,
            cmd,
            SETANIM_BOTH,
            BOTH_GETUP_BROLL_L as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            0,
        );
        rolled = QTRUE;
    } else if /* cmd.upmove > 0 && */ cmd.rightmove == 0 && cmd.forwardmove > 0 {
        G_SetAnim(
            self_,
            cmd,
            SETANIM_BOTH,
            BOTH_GETUP_BROLL_F as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            0,
        );
        rolled = QTRUE;
    } else if /* cmd.upmove > 0 && */ cmd.rightmove == 0 && cmd.forwardmove < 0 {
        G_SetAnim(
            self_,
            cmd,
            SETANIM_BOTH,
            BOTH_GETUP_BROLL_B as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            0,
        );
        rolled = QTRUE;
    } else if cmd.upmove != 0 {
        G_PreDefSound(&(*client).ps.origin, PDSOUND_FORCEJUMP);
        (*client).ps.forceDodgeAnim = 2;
        (*client).ps.forceHandExtendTime = level.time + 500;

        //self->client->ps.velocity[2] = 300;
    }

    if rolled == QTRUE {
        G_EntitySound(self_, CHAN_VOICE, G_SoundIndex("*jump1.wav"));
    }

    rolled
}

/// `void ForceJumpCharge( gentity_t *self, usercmd_t *ucmd )` (w_force.c:2324) — the old
/// "charge"-jump handler (C-commented "I guess this is unused now"). Builds up
/// `ps.fd.forceJumpCharge` while the force-jump button is held: bails on death, on no-charge while
/// airborne, and on insufficient force power (muting the channel-1 jump-build sound). Plays the
/// `jumpbuild.wav` loop the first frame of a charge, increments the charge by
/// `forceJumpChargeInterval*50` on a 500 ms debounce, then clamps to the per-level max strength and
/// to the maximum the player can actually afford.
///
/// `forceJumpChargeInterval = forceJumpStrength[0] / (FORCE_JUMP_CHARGE_TIME/FRAMETIME)`; the
/// `FORCE_JUMP_CHARGE_TIME/FRAMETIME` term is **C integer division** (`6400/100 = 64`) done before
/// the float divide, mirrored here as a `c_int` divide cast to `f32`.
///
/// No oracle — a gentity/playerState mutation leaf with `G_MuteSound`/`G_Sound` sound-trap side
/// effects (the surrounding w_force-leaf precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`; the sound system must be initialised
/// (`G_MuteSound`/`G_Sound`/`G_SoundIndex` reach into the entity/trap layer).
pub unsafe fn ForceJumpCharge(self_: *mut gentity_t, _ucmd: *mut usercmd_t) {
    //I guess this is unused now. Was used for the "charge" jump type.
    let force_jump_charge_interval: f32 =
        forceJumpStrength[0] / (FORCE_JUMP_CHARGE_TIME / FRAMETIME) as f32;

    let client = (*self_).client;

    if (*self_).health <= 0 {
        return;
    }

    if (*client).ps.fd.forceJumpCharge == 0.0
        && (*client).ps.groundEntityNum == ENTITYNUM_NONE
    {
        return;
    }

    if (*client).ps.fd.forcePower
        < forcePowerNeeded[(*client).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
            [FP_LEVITATION as usize]
    {
        G_MuteSound(
            (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 - 50) as usize],
            CHAN_VOICE,
        );
        return;
    }

    if (*client).ps.fd.forceJumpCharge == 0.0 {
        (*client).ps.fd.forceJumpAddTime = 0;
    }

    if (*client).ps.fd.forceJumpAddTime >= level.time {
        return;
    }

    //need to play sound
    if (*client).ps.fd.forceJumpCharge == 0.0 {
        G_Sound(
            self_,
            TRACK_CHANNEL_1,
            G_SoundIndex("sound/weapons/force/jumpbuild.wav"),
        );
    }

    //Increment
    if (*client).ps.fd.forceJumpAddTime < level.time {
        (*client).ps.fd.forceJumpCharge += force_jump_charge_interval * 50.0;
        (*client).ps.fd.forceJumpAddTime = level.time + 500;
    }

    //clamp to max strength for current level
    if (*client).ps.fd.forceJumpCharge
        > forceJumpStrength[(*client).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
    {
        (*client).ps.fd.forceJumpCharge =
            forceJumpStrength[(*client).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize];
        G_MuteSound(
            (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 - 50) as usize],
            CHAN_VOICE,
        );
    }

    //clamp to max available force power
    if (*client).ps.fd.forceJumpCharge
        / force_jump_charge_interval
        / (FORCE_JUMP_CHARGE_TIME / FRAMETIME) as f32
        * forcePowerNeeded[(*client).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
            [FP_LEVITATION as usize] as f32
        > (*client).ps.fd.forcePower as f32
    {
        //can't use more than you have
        G_MuteSound(
            (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 - 50) as usize],
            CHAN_VOICE,
        );
        (*client).ps.fd.forceJumpCharge = (*client).ps.fd.forcePower as f32
            * force_jump_charge_interval
            / (FORCE_JUMP_CHARGE_TIME / FRAMETIME) as f32;
    }

    //G_Printf("%f\n", self->client->ps.fd.forceJumpCharge);
}

/// `int WP_GetVelocityForForceJump( gentity_t *self, vec3_t jumpVel, usercmd_t *ucmd )`
/// (w_force.c:2384) — compute the velocity for a force-jump from the player's view and movement
/// intent into the out-param `jumpVel`, and return the `FJ_*` direction.
///
/// Flattens the viewangles' pitch to 0 and derives `forward`/`right`, then picks a forward/right
/// push magnitude from the usercmd (±50 each on a diagonal, ±100 on a single axis, 0 if neither).
/// Mutes any prior jump sound on `TRACK_CHANNEL_1` and fires the predefined force-jump sound at
/// the player origin. Gives a minimum boost (`forceJumpCharge` floored to `JUMP_VELOCITY+400` if
/// below `JUMP_VELOCITY+40`), clamps a falling `velocity[2]` up to `-30`, then `VectorMA`s the
/// fwd/right push onto the player velocity into `jumpVel` and adds the charge to `jumpVel[2]`.
/// Returns the dominant `FJ_*` direction (only when `forceJumpCharge > 200`), else `FJ_UP`.
///
/// No oracle — `G_MuteSound` + `G_PreDefSound` (the latter spawns into the live `g_entities`
/// array) are entity/sound-trap side effects (the entity/trap control-flow precedent shared with
/// `G_PreDefSound`/`G_SpecialRollGetup`).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`, and `ucmd` a valid usercmd. The
/// entity/sound systems must be initialised (`G_MuteSound`/`G_PreDefSound` reach into them).
pub unsafe fn WP_GetVelocityForForceJump(
    self_: *mut gentity_t,
    jump_vel: &mut vec3_t,
    ucmd: *mut usercmd_t,
) -> c_int {
    let mut push_fwd: f32 = 0.0;
    let mut push_rt: f32 = 0.0;
    let mut view: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];

    let client = (*self_).client;

    VectorCopy(&(*client).ps.viewangles, &mut view);
    view[0] = 0.0;
    AngleVectors(&view, Some(&mut forward), Some(&mut right), None);
    if (*ucmd).forwardmove != 0 && (*ucmd).rightmove != 0 {
        if (*ucmd).forwardmove > 0 {
            push_fwd = 50.0;
        } else {
            push_fwd = -50.0;
        }
        if (*ucmd).rightmove > 0 {
            push_rt = 50.0;
        } else {
            push_rt = -50.0;
        }
    } else if (*ucmd).forwardmove != 0 || (*ucmd).rightmove != 0 {
        if (*ucmd).forwardmove > 0 {
            push_fwd = 100.0;
        } else if (*ucmd).forwardmove < 0 {
            push_fwd = -100.0;
        } else if (*ucmd).rightmove > 0 {
            push_rt = 100.0;
        } else if (*ucmd).rightmove < 0 {
            push_rt = -100.0;
        }
    }

    G_MuteSound(
        (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 - 50) as usize],
        CHAN_VOICE,
    );

    G_PreDefSound(&(*client).ps.origin, PDSOUND_FORCEJUMP);

    if (*client).ps.fd.forceJumpCharge < (JUMP_VELOCITY + 40) as f32 {
        //give him at least a tiny boost from just a tap
        (*client).ps.fd.forceJumpCharge = (JUMP_VELOCITY + 400) as f32;
    }

    if (*client).ps.velocity[2] < -30.0 {
        //so that we can get a good boost when force jumping in a fall
        (*client).ps.velocity[2] = -30.0;
    }

    VectorMA(&(*client).ps.velocity, push_fwd, &forward, jump_vel);
    VectorMA(&(*client).ps.velocity, push_rt, &right, jump_vel);
    jump_vel[2] += (*client).ps.fd.forceJumpCharge;
    if push_fwd > 0.0 && (*client).ps.fd.forceJumpCharge > 200.0 {
        FJ_FORWARD
    } else if push_fwd < 0.0 && (*client).ps.fd.forceJumpCharge > 200.0 {
        FJ_BACKWARD
    } else if push_rt > 0.0 && (*client).ps.fd.forceJumpCharge > 200.0 {
        FJ_RIGHT
    } else if push_rt < 0.0 && (*client).ps.fd.forceJumpCharge > 200.0 {
        FJ_LEFT
    } else {
        FJ_UP
    }
}

/// `void ForceJump( gentity_t *self, usercmd_t *ucmd )` (w_force.c:2469) — perform the force jump.
/// Bails when the `FP_LEVITATION` duration is still pending (`forcePowerDuration[FP_LEVITATION] >
/// level.time`), the power isn't usable (`!WP_ForcePowerUsable`), `self` is airborne
/// (`groundEntityNum == ENTITYNUM_NONE`), or `self` is dead (`health <= 0`). Otherwise marks
/// `fjDidJump`, computes the velocity via `WP_GetVelocityForForceJump`, records `forceJumpZStart`
/// (for soft-landing), copies the jump velocity into `ps.velocity`, fires `WP_ForcePowerStart`
/// with a charge-scaled override amount, zeroes `forceJumpCharge`, and sets `forceJumpFlip`.
///
/// The override amount mirrors the C float expression
/// `forceJumpCharge / forceJumpChargeInterval / (FORCE_JUMP_CHARGE_TIME/FRAMETIME) *
/// forcePowerNeeded[level][FP_LEVITATION]` truncated to the `int overrideAmt` param;
/// `forceJumpChargeInterval = forceJumpStrength[level] / (FORCE_JUMP_CHARGE_TIME/FRAMETIME)` where
/// the `FORCE_JUMP_CHARGE_TIME/FRAMETIME` term is **C integer division** (`6400/100 = 64`), matching
/// `ForceJumpCharge`. No oracle — mutates the live gentity/playerState and funnels into
/// `WP_ForcePowerStart`/`WP_GetVelocityForForceJump` (the surrounding-w_force-leaf precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`, and `ucmd` a valid usercmd.
pub unsafe fn ForceJump(self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    let mut jump_vel: vec3_t = [0.0; 3];

    let client = (*self_).client;

    if (*client).ps.fd.forcePowerDuration[FP_LEVITATION as usize] > (*addr_of!(level)).time {
        return;
    }
    if WP_ForcePowerUsable(self_, FP_LEVITATION) == QFALSE {
        return;
    }
    if (*self_).s.groundEntityNum == ENTITYNUM_NONE {
        return;
    }
    if (*self_).health <= 0 {
        return;
    }

    (*client).fjDidJump = QTRUE;

    let level_ = (*client).ps.fd.forcePowerLevel[FP_LEVITATION as usize];
    let force_jump_charge_interval: f32 =
        forceJumpStrength[level_ as usize] / (FORCE_JUMP_CHARGE_TIME / FRAMETIME) as f32;

    WP_GetVelocityForForceJump(self_, &mut jump_vel, ucmd);

    //FIXME: sound effect
    (*client).ps.fd.forceJumpZStart = (*client).ps.origin[2]; //remember this for when we land
    VectorCopy(&jump_vel, &mut (*client).ps.velocity);
    //wasn't allowing them to attack when jumping, but that was annoying
    //self->client->ps.weaponTime = self->client->ps.torsoAnimTimer;

    WP_ForcePowerStart(
        self_,
        FP_LEVITATION,
        ((*client).ps.fd.forceJumpCharge
            / force_jump_charge_interval
            / (FORCE_JUMP_CHARGE_TIME / FRAMETIME) as f32
            * forcePowerNeeded[level_ as usize][FP_LEVITATION as usize] as f32)
            as c_int,
    );
    //self->client->ps.fd.forcePowerDuration[FP_LEVITATION] = level.time + self->client->ps.weaponTime;
    (*client).ps.fd.forceJumpCharge = 0.0;
    (*client).ps.forceJumpFlip = QTRUE;
}

/// `int WP_AbsorbConversion(gentity_t *attacked, int atdAbsLevel, gentity_t *attacker, int atPower,
/// int atPowerLevel, int atForceSpent)` (w_force.c:947) — apply the defender's force-absorb to an
/// incoming offensive force power. Returns `-1` when the power is not absorbable
/// (only `FP_LIGHTNING`/`FP_DRAIN`/`FP_GRIP`/`FP_PUSH`/`FP_PULL`), the attacker has no absorb level,
/// or absorb is not active on `attacked`. Otherwise returns the absorb-reduced power level
/// (`atPowerLevel - atdAbsLevel`, floored at 0), refunds `(atForceSpent/3)*forcePowerLevel[FP_ABSORB]`
/// force power (floored to 1 when `atForceSpent >= 1`, capped at 100) to the defender, and — on a
/// `forcePowerSoundDebounce < level.time` window — fires `PDSOUND_ABSORBHIT` at the defender's origin
/// (stamping `s.trickedentindex = attacked->s.number`) and bumps the debounce by 400 ms.
///
/// `atForceSpent/3` is C integer division. No oracle — gentity/playerState mutation plus the
/// `G_PreDefSound` live-`g_entities` temp-entity spawn (the surrounding-w_force-leaf precedent).
///
/// # Safety
/// `attacked` must be a valid entity with a non-NULL `client`. The entity/sound systems must be
/// initialised (`G_PreDefSound` allocates from `g_entities`). `attacker` is unused (mirrors C).
pub unsafe fn WP_AbsorbConversion(
    attacked: *mut gentity_t,
    atd_abs_level: c_int,
    _attacker: *mut gentity_t,
    at_power: c_int,
    at_power_level: c_int,
    at_force_spent: c_int,
) -> c_int {
    let mut get_level: c_int;
    let mut add_tot: c_int;

    if at_power != FP_LIGHTNING
        && at_power != FP_DRAIN
        && at_power != FP_GRIP
        && at_power != FP_PUSH
        && at_power != FP_PULL
    {
        //Only these powers can be absorbed
        return -1;
    }

    if atd_abs_level == 0 {
        //looks like attacker doesn't have any absorb power
        return -1;
    }

    if (*(*attacked).client).ps.fd.forcePowersActive & (1 << FP_ABSORB) == 0 {
        //absorb is not active
        return -1;
    }

    //Subtract absorb power level from the offensive force power
    get_level = at_power_level;
    get_level -= atd_abs_level;

    if get_level < 0 {
        get_level = 0;
    }

    //let the attacker absorb an amount of force used in this attack based on his level of absorb
    add_tot = (at_force_spent / 3) * (*(*attacked).client).ps.fd.forcePowerLevel[FP_ABSORB as usize];

    if add_tot < 1 && at_force_spent >= 1 {
        add_tot = 1;
    }
    (*(*attacked).client).ps.fd.forcePower += add_tot;
    if (*(*attacked).client).ps.fd.forcePower > 100 {
        (*(*attacked).client).ps.fd.forcePower = 100;
    }

    //play sound indicating that attack was absorbed
    if (*(*attacked).client).forcePowerSoundDebounce < (*addr_of!(level)).time {
        let ab_sound = G_PreDefSound(&(*(*attacked).client).ps.origin, PDSOUND_ABSORBHIT);
        (*ab_sound).s.trickedentindex = (*attacked).s.number;

        (*(*attacked).client).forcePowerSoundDebounce = (*addr_of!(level)).time + 400;
    }

    get_level
}

/// `int ForcePowerUsableOn(gentity_t *attacker, gentity_t *other, forcePowers_t forcePower)`
/// (w_force.c:704) — the can-I-use-this-power-on-that-target gate. Returns C `int` 0/1 (not
/// `qboolean`). Returns 0 when: `other` is under an ysalamiri (`BG_HasYsalamiri`); `attacker`
/// cannot currently use `forcePower` (`!BG_CanUseFPNow`); either side is in a duel
/// (`ps.duelInProgress`); `FP_GRIP` while `other` is absorbing (firing the same
/// `PDSOUND_ABSORBHIT` debounce stamp as `WP_AbsorbConversion`) or while `other` is a
/// saber-special (`BG_SaberInSpecial`); `FP_PUSH`/`FP_PULL` while `other` is knocked down
/// (`BG_InKnockDown`); a vehicle NPC (`ET_NPC` + `CLASS_VEHICLE`) for any power but
/// `FP_LIGHTNING`; or any `ET_NPC` under `GT_SIEGE`. Otherwise returns 1.
///
/// No oracle — gentity/playerState reads plus the `FP_GRIP`-branch `G_PreDefSound` live-
/// `g_entities` temp-entity spawn (the surrounding-w_force-leaf precedent).
///
/// # Safety
/// `attacker`/`other` may be NULL (each is NULL-guarded before deref). When non-NULL, each must
/// be a valid `gentity_t`; the entity/sound systems must be initialised for the `FP_GRIP` branch.
pub unsafe fn ForcePowerUsableOn(
    attacker: *mut gentity_t,
    other: *mut gentity_t,
    force_power: c_int,
) -> c_int {
    if !other.is_null()
        && !(*other).client.is_null()
        && BG_HasYsalamiri((*addr_of!(g_gametype)).integer, &mut (*(*other).client).ps) != QFALSE
    {
        return 0;
    }

    if !attacker.is_null()
        && !(*attacker).client.is_null()
        && BG_CanUseFPNow(
            (*addr_of!(g_gametype)).integer,
            &mut (*(*attacker).client).ps,
            (*addr_of!(level)).time,
            force_power,
        ) == QFALSE
    {
        return 0;
    }

    //Dueling fighters cannot use force powers on others, with the exception of force push when locked with each other
    if !attacker.is_null()
        && !(*attacker).client.is_null()
        && (*(*attacker).client).ps.duelInProgress != 0
    {
        return 0;
    }

    if !other.is_null()
        && !(*other).client.is_null()
        && (*(*other).client).ps.duelInProgress != 0
    {
        return 0;
    }

    if force_power == FP_GRIP {
        if !other.is_null()
            && !(*other).client.is_null()
            && ((*(*other).client).ps.fd.forcePowersActive & (1 << FP_ABSORB)) != 0
        {
            //don't allow gripping to begin with if they are absorbing
            //play sound indicating that attack was absorbed
            if (*(*other).client).forcePowerSoundDebounce < (*addr_of!(level)).time {
                let ab_sound = G_PreDefSound(&(*(*other).client).ps.origin, PDSOUND_ABSORBHIT);
                (*ab_sound).s.trickedentindex = (*other).s.number;
                (*(*other).client).forcePowerSoundDebounce = (*addr_of!(level)).time + 400;
            }
            return 0;
        } else if !other.is_null()
            && !(*other).client.is_null()
            && (*(*other).client).ps.weapon == WP_SABER
            && BG_SaberInSpecial((*(*other).client).ps.saberMove) != QFALSE
        {
            //don't grip person while they are in a special or some really bad things can happen.
            return 0;
        }
    }

    if !other.is_null()
        && !(*other).client.is_null()
        && (force_power == FP_PUSH || force_power == FP_PULL)
    {
        if BG_InKnockDown((*(*other).client).ps.legsAnim) != QFALSE {
            return 0;
        }
    }

    if !other.is_null()
        && !(*other).client.is_null()
        && (*other).s.eType == ET_NPC
        && (*other).s.NPC_class == CLASS_VEHICLE
    {
        //can't use the force on vehicles.. except lightning
        if force_power == FP_LIGHTNING {
            return 1;
        } else {
            return 0;
        }
    }

    if !other.is_null()
        && !(*other).client.is_null()
        && (*other).s.eType == ET_NPC
        && (*addr_of!(g_gametype)).integer == GT_SIEGE
    {
        //can't use powers at all on npc's normally in siege...
        return 0;
    }

    1
}

/// `void GEntity_UseFunc( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (w_force.c:2902) — one-line wrapper that fires `self`'s `use` callback via `GlobalUse`.
///
/// No oracle — pure delegation to `GlobalUse` (itself a `gentity_t` `use`-callback dispatcher
/// with trap/entity side-effects, no value to compare).
///
/// # Safety
/// Pointers are forwarded unchanged to `GlobalUse`, which NULL-guards `self`; `other`/`activator`
/// may be NULL.
pub unsafe fn GEntity_UseFunc(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    GlobalUse(self_, other, activator);
}

/// `qboolean CanCounterThrow(gentity_t *self, gentity_t *thrower, qboolean pull)`
/// (w_force.c:2907) — can `self` counter an incoming force push/pull with one of their own?
/// Returns 0 (cannot counter) when: a hand gesture is in progress
/// (`forceHandExtend != HANDEXTEND_NONE`); the weapon is busy (`weaponTime > 0`); `self` is dead
/// (`health <= 0`); `self` is being disintegrated (`powerups[PW_DISINT_4] > level.time`); the
/// weapon is charging (`WEAPON_CHARGING`/`WEAPON_CHARGING_ALT`, no auto-defend while charging); in
/// `GT_SIEGE` for a `pull` from a valid client thrower, the thrower is more than 60 degrees off
/// `self`'s view yaw (can't defend a back-pull in siege); the relevant power
/// (`FP_PULL`/`FP_PUSH`) is unusable (`!WP_ForcePowerUsable`); or `self` is airborne
/// (`groundEntityNum == ENTITYNUM_NONE`). Otherwise returns 1.
///
/// Returns C `qboolean` (0/1). No oracle — a `gentity_t`/`playerState_t` read leaf gated by traps
/// and globals (`level.time`, `g_gametype`); its non-trivial deps `WP_ForcePowerUsable`,
/// `AngleSubtract`, `vectoangles`, `VectorSubtract` are themselves oracle-tested
/// (surrounding-w_force-leaf precedent).
///
/// # Safety
/// `self` must be a valid `gentity_t` with a non-NULL `client`; `thrower` may be NULL (only
/// dereferenced after a NULL + non-NULL-`client` guard).
pub unsafe fn CanCounterThrow(
    self_: *mut gentity_t,
    thrower: *mut gentity_t,
    pull: qboolean,
) -> qboolean {
    // C: `int powerUse = 0;` — the init is dead (always reassigned in the FP_PULL/FP_PUSH branch
    // below before its only read); modeled as a deferred-init binding to keep behavior identical.
    let power_use: c_int;

    if (*(*self_).client).ps.forceHandExtend != HANDEXTEND_NONE {
        return 0;
    }

    if (*(*self_).client).ps.weaponTime > 0 {
        return 0;
    }

    if (*self_).health <= 0 {
        return 0;
    }

    if (*(*self_).client).ps.powerups[PW_DISINT_4 as usize] > (*addr_of!(level)).time {
        return 0;
    }

    if (*(*self_).client).ps.weaponstate == WEAPON_CHARGING
        || (*(*self_).client).ps.weaponstate == WEAPON_CHARGING_ALT
    {
        //don't autodefend when charging a weapon
        return 0;
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && pull != QFALSE
        && !thrower.is_null()
        && !(*thrower).client.is_null()
    {
        //in siege, pull will affect people if they are not facing you, so they can't run away so much
        let mut d: vec3_t = [0.0; 3];
        let a: f32;

        VectorSubtract(
            &(*(*thrower).client).ps.origin,
            &(*(*self_).client).ps.origin,
            &mut d,
        );
        let d_copy = d;
        vectoangles(&d_copy, &mut d);

        a = AngleSubtract(d[YAW], (*(*self_).client).ps.viewangles[YAW]);

        if a > 60.0f32 || a < -60.0f32 {
            //if facing more than 60 degrees away they cannot defend
            return 0;
        }
    }

    if pull != QFALSE {
        power_use = FP_PULL;
    } else {
        power_use = FP_PUSH;
    }

    if WP_ForcePowerUsable(self_, power_use) == QFALSE {
        return 0;
    }

    if (*(*self_).client).ps.groundEntityNum == ENTITYNUM_NONE {
        //you cannot counter a push/pull if you're in the air
        return 0;
    }

    1
}

/// `void G_LetGoOfWall( gentity_t *ent )` (w_force.c:3032) — release a wall-clinging player.
/// NULL-guards `ent`/`ent->client`, clears the `PMF_STUCK_TO_WALL` pm_flag, then zeroes the
/// legs/torso anim timer whenever the respective anim is in a rebound jump or hold
/// (`BG_InReboundJump`/`BG_InReboundHold`), so a fresh anim can take over immediately.
///
/// No oracle — a `gentity_t` mutation leaf (entity/control-flow precedent; the predicate deps
/// `BG_InReboundJump`/`BG_InReboundHold` are themselves oracle-tested in `bg_panimate`).
///
/// # Safety
/// `ent` must be a valid `gentity_t` pointer or NULL; `ent->client` likewise.
pub unsafe fn G_LetGoOfWall(ent: *mut gentity_t) {
    if ent.is_null() || (*ent).client.is_null() {
        return;
    }
    let client = (*ent).client;
    (*client).ps.pm_flags &= !PMF_STUCK_TO_WALL;
    if BG_InReboundJump((*client).ps.legsAnim) != QFALSE
        || BG_InReboundHold((*client).ps.legsAnim) != QFALSE
    {
        (*client).ps.legsTimer = 0;
    }
    if BG_InReboundJump((*client).ps.torsoAnim) != QFALSE
        || BG_InReboundHold((*client).ps.torsoAnim) != QFALSE
    {
        (*client).ps.torsoTimer = 0;
    }
}

/// `void WP_ForcePowerStart( gentity_t *self, forcePowers_t forcePower, int overrideAmt )`
/// (w_force.c:1028) — activate the given force power. Stops any full-body taunt, then runs a
/// per-power `switch`: sets the bot-only `hearable`/`hearDist` hints, OR-s the power's bit into
/// `forcePowersActive`, and (for the duration-based powers) picks a `duration` keyed off the
/// power level. `FP_GRIP` also stamps a 60 s `PW_DISINT_4` powerup; `FP_LIGHTNING`/`FP_DRAIN`
/// move `overrideAmt` into `duration` and zero `overrideAmt` (they drain as damage is dealt),
/// and `FP_LIGHTNING` records `activeForcePass`. After the switch: writes
/// `forcePowerDuration[forcePower]` (`level.time + duration`, or 0), seeds the
/// `otherSoundLen`/`otherSoundTime` hearable hint, clears `forcePowerDebounce[forcePower]`, and
/// finally drains the cost via `BG_ForcePowerDrain` — `FP_SPEED` with an override drains
/// `overrideAmt*0.025`, grip/drain skip the drain entirely, everything else drains `overrideAmt`.
/// The `FP_SPEED`/`FP_TELEPATHY`/`FP_RAGE`/`FP_SEE` arms `break` out of the switch (skipping
/// activation) when the level is none of `FORCE_LEVEL_1..3` ("shouldn't get here").
///
/// No oracle — reads/mutates the live gentity/playerState + `level.time` (entity/trap
/// control-flow precedent, matching the surrounding w_force leaves).
///
/// # Safety
/// `self_` must be a valid `gentity_t` with a non-null `client`. The level/entity systems must
/// be initialised.
pub unsafe fn WP_ForcePowerStart(self_: *mut gentity_t, force_power: c_int, override_amt: c_int) {
    //activate the given force power
    let mut override_amt = override_amt;
    let mut duration: c_int = 0;
    let mut hearable: qboolean = QFALSE;
    let mut hear_dist: f32 = 0.0;

    if WP_ForcePowerAvailable(self_, force_power, override_amt) == QFALSE {
        return;
    }

    let client = (*self_).client;

    if BG_FullBodyTauntAnim((*client).ps.legsAnim) != QFALSE {
        //stop taunt
        (*client).ps.legsTimer = 0;
    }
    if BG_FullBodyTauntAnim((*client).ps.torsoAnim) != QFALSE {
        //stop taunt
        (*client).ps.torsoTimer = 0;
    }
    //hearable and hearDist are merely for the benefit of bots, and not related to if a sound is
    //actually played. If duration is set, the force power will assume to be timer-based.
    //
    // The C `switch` is modeled as a labeled block we `break` out of; the
    // "shouldn't get here" arms break *before* setting `forcePowersActive`, so we mirror that
    // by breaking out early (leaving `duration` at 0 and the power bit unset).
    'force_switch: {
        match force_power {
            FP_HEAL => {
                hearable = QTRUE;
                hear_dist = 256.0;
                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_LEVITATION => {
                hearable = QTRUE;
                hear_dist = 256.0;
                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_SPEED => {
                hearable = QTRUE;
                hear_dist = 256.0;
                if (*client).ps.fd.forcePowerLevel[FP_SPEED as usize] == FORCE_LEVEL_1 {
                    duration = 10000;
                } else if (*client).ps.fd.forcePowerLevel[FP_SPEED as usize] == FORCE_LEVEL_2 {
                    duration = 15000;
                } else if (*client).ps.fd.forcePowerLevel[FP_SPEED as usize] == FORCE_LEVEL_3 {
                    duration = 20000;
                } else {
                    break 'force_switch; //shouldn't get here
                }

                if override_amt != 0 {
                    duration = override_amt;
                }

                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_PUSH => {
                hearable = QTRUE;
                hear_dist = 256.0;
            }
            FP_PULL => {
                hearable = QTRUE;
                hear_dist = 256.0;
            }
            FP_TELEPATHY => {
                hearable = QTRUE;
                hear_dist = 256.0;
                if (*client).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_1 {
                    duration = 20000;
                } else if (*client).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_2 {
                    duration = 25000;
                } else if (*client).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_3 {
                    duration = 30000;
                } else {
                    break 'force_switch; //shouldn't get here
                }

                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_GRIP => {
                hearable = QTRUE;
                hear_dist = 256.0;
                (*client).ps.fd.forcePowersActive |= 1 << force_power;
                (*client).ps.powerups[PW_DISINT_4 as usize] = (*addr_of!(level)).time + 60000;
            }
            FP_LIGHTNING => {
                hearable = QTRUE;
                hear_dist = 512.0;
                duration = override_amt;
                override_amt = 0;
                (*client).ps.fd.forcePowersActive |= 1 << force_power;
                (*client).ps.activeForcePass =
                    (*client).ps.fd.forcePowerLevel[FP_LIGHTNING as usize];
            }
            FP_RAGE => {
                hearable = QTRUE;
                hear_dist = 256.0;
                if (*client).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_1 {
                    duration = 8000;
                } else if (*client).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_2 {
                    duration = 14000;
                } else if (*client).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_3 {
                    duration = 20000;
                } else {
                    break 'force_switch; //shouldn't get here
                }

                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_PROTECT => {
                hearable = QTRUE;
                hear_dist = 256.0;
                duration = 20000;
                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_ABSORB => {
                hearable = QTRUE;
                hear_dist = 256.0;
                duration = 20000;
                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_TEAM_HEAL => {
                hearable = QTRUE;
                hear_dist = 256.0;
                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_TEAM_FORCE => {
                hearable = QTRUE;
                hear_dist = 256.0;
                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_DRAIN => {
                hearable = QTRUE;
                hear_dist = 256.0;
                duration = override_amt;
                override_amt = 0;
                (*client).ps.fd.forcePowersActive |= 1 << force_power;
                //self->client->ps.activeForcePass = self->client->ps.fd.forcePowerLevel[FP_DRAIN];
            }
            FP_SEE => {
                hearable = QTRUE;
                hear_dist = 256.0;
                if (*client).ps.fd.forcePowerLevel[FP_SEE as usize] == FORCE_LEVEL_1 {
                    duration = 10000;
                } else if (*client).ps.fd.forcePowerLevel[FP_SEE as usize] == FORCE_LEVEL_2 {
                    duration = 20000;
                } else if (*client).ps.fd.forcePowerLevel[FP_SEE as usize] == FORCE_LEVEL_3 {
                    duration = 30000;
                } else {
                    break 'force_switch; //shouldn't get here
                }

                (*client).ps.fd.forcePowersActive |= 1 << force_power;
            }
            FP_SABER_OFFENSE => {}
            FP_SABER_DEFENSE => {}
            FP_SABERTHROW => {}
            _ => {}
        }
    }

    if duration != 0 {
        (*client).ps.fd.forcePowerDuration[force_power as usize] =
            (*addr_of!(level)).time + duration;
    } else {
        (*client).ps.fd.forcePowerDuration[force_power as usize] = 0;
    }

    if hearable != QFALSE {
        (*client).ps.otherSoundLen = hear_dist;
        (*client).ps.otherSoundTime = (*addr_of!(level)).time + 100;
    }

    (*client).ps.fd.forcePowerDebounce[force_power as usize] = 0;

    if force_power == FP_SPEED && override_amt != 0 {
        BG_ForcePowerDrain(
            &mut (*client).ps,
            force_power,
            (override_amt as f64 * 0.025) as c_int,
        );
    } else if force_power != FP_GRIP && force_power != FP_DRAIN {
        //grip and drain drain as damage is done
        BG_ForcePowerDrain(&mut (*client).ps, force_power, override_amt);
    }
}

/// `void ForceHeal( gentity_t *self )` (w_force.c:1243) — the Force Heal activator.
/// Bails when `self` is dead (`health <= 0`), the power isn't usable (`WP_ForcePowerUsable`),
/// or `self` is already at `STAT_MAX_HEALTH`. Otherwise heals an amount keyed off the heal
/// power level (+25 at `FORCE_LEVEL_3`, +10 at `FORCE_LEVEL_2`, else +5), clamps to
/// `STAT_MAX_HEALTH`, drains the power (`BG_ForcePowerDrain`), and plays the heal sound. All
/// levels are instant — the only `WP_ForcePowerStart` reference in the C is the commented-out
/// `else` branch, so this leaf never funnels through `WP_ForcePowerStart`.
///
/// No oracle — reads/writes `gentity_t`/`client`/playerState and fires `G_Sound`
/// (entity/trap control-flow precedent, matching the surrounding w_force leaves).
///
/// # Safety
/// `self_` must be a valid `gentity_t` with a non-null `client`. The entity/sound systems
/// must be initialised.
pub unsafe fn ForceHeal(self_: *mut gentity_t) {
    if (*self_).health <= 0 {
        return;
    }

    if WP_ForcePowerUsable(self_, FP_HEAL) == QFALSE {
        return;
    }

    let client = (*self_).client;

    if (*self_).health >= (*client).ps.stats[STAT_MAX_HEALTH as usize] {
        return;
    }

    if (*client).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_3 {
        (*self_).health += 25; //This was 50, but that angered the Balance God.

        if (*self_).health > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
            (*self_).health = (*client).ps.stats[STAT_MAX_HEALTH as usize];
        }
        BG_ForcePowerDrain(&mut (*client).ps, FP_HEAL, 0);
    } else if (*client).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_2 {
        (*self_).health += 10;

        if (*self_).health > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
            (*self_).health = (*client).ps.stats[STAT_MAX_HEALTH as usize];
        }
        BG_ForcePowerDrain(&mut (*client).ps, FP_HEAL, 0);
    } else {
        (*self_).health += 5;

        if (*self_).health > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
            (*self_).health = (*client).ps.stats[STAT_MAX_HEALTH as usize];
        }
        BG_ForcePowerDrain(&mut (*client).ps, FP_HEAL, 0);
    }
    /*
    else
    {
        WP_ForcePowerStart( self, FP_HEAL, 0 );
    }
    */
    //NOTE: Decided to make all levels instant.

    G_Sound(self_, CHAN_ITEM, G_SoundIndex("sound/weapons/force/heal.wav"));
}

/// `void ForceSpeed( gentity_t *self, int forceDuration )` (w_force.c:1603) — the FP_SPEED
/// activator. Bails when `self` is dead; if the power is already active and the deactivate
/// debounce has elapsed it toggles off via `WP_ForcePowerStop`. Otherwise it gates on
/// `WP_ForcePowerUsable`, refuses while holding a Siege objective item whose carrier disables
/// force powers (`genericValue15`), arms the 1500ms deactivate debounce, starts the power for
/// `forceDuration`, and plays the speed one-shot + the `speedLoopSound` loop on
/// `TRACK_CHANNEL_2`.
///
/// No oracle — gentity/playerState mutation + sound-trap side effects, indexes the live
/// `g_entities` array (the surrounding w_force activator-leaf precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`. `g_entities` must be initialised
/// (the Siege-item branch indexes it by `holdingObjectiveItem`).
pub unsafe fn ForceSpeed(self_: *mut gentity_t, force_duration: c_int) {
    if (*self_).health <= 0 {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.forceAllowDeactivateTime < (*addr_of!(level)).time
        && (*client).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0
    {
        WP_ForcePowerStop(self_, FP_SPEED);
        return;
    }

    if WP_ForcePowerUsable(self_, FP_SPEED) == QFALSE {
        return;
    }

    if (*client).holdingObjectiveItem >= MAX_CLIENTS as c_int
        && (*client).holdingObjectiveItem < ENTITYNUM_WORLD
    {
        //holding Siege item
        if (*core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*client).holdingObjectiveItem as usize)).genericValue15 != 0 {
            //disables force powers
            return;
        }
    }

    (*client).ps.forceAllowDeactivateTime = (*addr_of!(level)).time + 1500;

    WP_ForcePowerStart(self_, FP_SPEED, force_duration);
    G_Sound(self_, CHAN_BODY, G_SoundIndex("sound/weapons/force/speed.wav"));
    G_Sound(self_, TRACK_CHANNEL_2, SPEED_LOOP_SOUND);
}

/// `void ForceSeeing( gentity_t *self )` (w_force.c:1638) — the FP_SEE activator. Bails when
/// dead; toggles off when already active past the deactivate debounce; else gates on
/// `WP_ForcePowerUsable`, arms the 1500ms debounce, starts the power (duration 0 — see runs
/// indefinitely), and plays the see one-shot on `CHAN_AUTO` plus `seeLoopSound` on
/// `TRACK_CHANNEL_5`.
///
/// No oracle — gentity/playerState mutation + sound-trap side effects (w_force activator-leaf
/// precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`.
pub unsafe fn ForceSeeing(self_: *mut gentity_t) {
    if (*self_).health <= 0 {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.forceAllowDeactivateTime < (*addr_of!(level)).time
        && (*client).ps.fd.forcePowersActive & (1 << FP_SEE) != 0
    {
        WP_ForcePowerStop(self_, FP_SEE);
        return;
    }

    if WP_ForcePowerUsable(self_, FP_SEE) == QFALSE {
        return;
    }

    (*client).ps.forceAllowDeactivateTime = (*addr_of!(level)).time + 1500;

    WP_ForcePowerStart(self_, FP_SEE, 0);

    G_Sound(self_, CHAN_AUTO, G_SoundIndex("sound/weapons/force/see.wav"));
    G_Sound(self_, TRACK_CHANNEL_5, SEE_LOOP_SOUND);
}

/// `void ForceProtect( gentity_t *self )` (w_force.c:1665) — the FP_PROTECT activator. Bails
/// when dead; toggles off when already active past the deactivate debounce; else gates on
/// `WP_ForcePowerUsable`, cancels any active Rage/Absorb (mutually exclusive), arms the 1500ms
/// debounce, starts the power, fires `PDSOUND_PROTECT` at the caster's origin, and plays
/// `protectLoopSound` on `TRACK_CHANNEL_3`.
///
/// No oracle — gentity/playerState mutation + sound-trap side effects (w_force activator-leaf
/// precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`. The entity/sound systems must be
/// initialised (`G_PreDefSound` spawns a temp entity).
pub unsafe fn ForceProtect(self_: *mut gentity_t) {
    if (*self_).health <= 0 {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.forceAllowDeactivateTime < (*addr_of!(level)).time
        && (*client).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0
    {
        WP_ForcePowerStop(self_, FP_PROTECT);
        return;
    }

    if WP_ForcePowerUsable(self_, FP_PROTECT) == QFALSE {
        return;
    }

    // Make sure to turn off Force Rage and Force Absorb.
    if (*client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
        WP_ForcePowerStop(self_, FP_RAGE);
    }
    if (*client).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0 {
        WP_ForcePowerStop(self_, FP_ABSORB);
    }

    (*client).ps.forceAllowDeactivateTime = (*addr_of!(level)).time + 1500;

    WP_ForcePowerStart(self_, FP_PROTECT, 0);
    G_PreDefSound(&(*client).ps.origin, PDSOUND_PROTECT);
    G_Sound(self_, TRACK_CHANNEL_3, PROTECT_LOOP_SOUND);
}

/// `void ForceAbsorb( gentity_t *self )` (w_force.c:1701) — the FP_ABSORB activator. Bails
/// when dead; toggles off when already active past the deactivate debounce; else gates on
/// `WP_ForcePowerUsable`, cancels any active Rage/Protect (mutually exclusive), arms the 1500ms
/// debounce, starts the power, fires `PDSOUND_ABSORB` at the caster's origin, and plays
/// `absorbLoopSound` on `TRACK_CHANNEL_3`.
///
/// No oracle — gentity/playerState mutation + sound-trap side effects (w_force activator-leaf
/// precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`. The entity/sound systems must be
/// initialised (`G_PreDefSound` spawns a temp entity).
pub unsafe fn ForceAbsorb(self_: *mut gentity_t) {
    if (*self_).health <= 0 {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.forceAllowDeactivateTime < (*addr_of!(level)).time
        && (*client).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0
    {
        WP_ForcePowerStop(self_, FP_ABSORB);
        return;
    }

    if WP_ForcePowerUsable(self_, FP_ABSORB) == QFALSE {
        return;
    }

    // Make sure to turn off Force Rage and Force Protection.
    if (*client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
        WP_ForcePowerStop(self_, FP_RAGE);
    }
    if (*client).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0 {
        WP_ForcePowerStop(self_, FP_PROTECT);
    }

    (*client).ps.forceAllowDeactivateTime = (*addr_of!(level)).time + 1500;

    WP_ForcePowerStart(self_, FP_ABSORB, 0);
    G_PreDefSound(&(*client).ps.origin, PDSOUND_ABSORB);
    G_Sound(self_, TRACK_CHANNEL_3, ABSORB_LOOP_SOUND);
}

/// `void ForceRage( gentity_t *self )` (w_force.c:1737) — the FP_RAGE activator. Bails when
/// dead; toggles off when already active past the deactivate debounce; else gates on
/// `WP_ForcePowerUsable`, refuses while the rage-recovery cooldown is pending or while
/// `health < 10`, cancels any active Protect/Absorb (mutually exclusive), arms the 1500ms
/// debounce, starts the power, then plays the rage one-shot on `TRACK_CHANNEL_4` and
/// `rageLoopSound` on `TRACK_CHANNEL_3`.
///
/// No oracle — gentity/playerState mutation + sound-trap side effects (w_force activator-leaf
/// precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`.
pub unsafe fn ForceRage(self_: *mut gentity_t) {
    if (*self_).health <= 0 {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.forceAllowDeactivateTime < (*addr_of!(level)).time
        && (*client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
    {
        WP_ForcePowerStop(self_, FP_RAGE);
        return;
    }

    if WP_ForcePowerUsable(self_, FP_RAGE) == QFALSE {
        return;
    }

    if (*client).ps.fd.forceRageRecoveryTime >= (*addr_of!(level)).time {
        return;
    }

    if (*self_).health < 10 {
        return;
    }

    // Make sure to turn off Force Protection and Force Absorb.
    if (*client).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0 {
        WP_ForcePowerStop(self_, FP_PROTECT);
    }
    if (*client).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0 {
        WP_ForcePowerStop(self_, FP_ABSORB);
    }

    (*client).ps.forceAllowDeactivateTime = (*addr_of!(level)).time + 1500;

    WP_ForcePowerStart(self_, FP_RAGE, 0);

    G_Sound(self_, TRACK_CHANNEL_4, G_SoundIndex("sound/weapons/force/rage.wav"));
    G_Sound(self_, TRACK_CHANNEL_3, RAGE_LOOP_SOUND);
}

/// `void ForceLightning( gentity_t *self )` (w_force.c:1784) — the FP_LIGHTNING activator (the
/// hand-shoot half; the per-target damage lives in the not-yet-ported `ForceLightningDamage`).
/// Bails on death, on `forcePower < 25` or `!WP_ForcePowerUsable(FP_LIGHTNING)`, while the
/// `forcePowerDebounce[FP_LIGHTNING]` cool-down is pending, while a hand gesture is in progress,
/// or while the weapon is busy; else arms a 20 s force-hold hand-extend (reusing the grip anim to
/// extend the burst), plays `lightning`, and fires `WP_ForcePowerStart(FP_LIGHTNING, 500)`.
/// A clean dep-clean leaf; its dispatch caller (`WP_DoSpecificPower`) stays unported.
pub unsafe fn ForceLightning(self_: *mut gentity_t) {
    if (*self_).health <= 0 {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.fd.forcePower < 25 || WP_ForcePowerUsable(self_, FP_LIGHTNING) == QFALSE {
        return;
    }
    if (*client).ps.fd.forcePowerDebounce[FP_LIGHTNING as usize] > (*addr_of!(level)).time {
        // stops it while using it and also after using it, up to 3 second delay
        return;
    }

    if (*client).ps.forceHandExtend != HANDEXTEND_NONE {
        return;
    }

    if (*client).ps.weaponTime > 0 {
        return;
    }

    // Shoot lightning from hand
    // using grip anim now, to extend the burst time
    (*client).ps.forceHandExtend = HANDEXTEND_FORCE_HOLD;
    (*client).ps.forceHandExtendTime = (*addr_of!(level)).time + 20000;

    G_Sound(
        self_,
        CHAN_BODY,
        G_SoundIndex("sound/weapons/force/lightning"),
    );

    WP_ForcePowerStart(self_, FP_LIGHTNING, 500);
}

/// `void ForceDrain( gentity_t *self )` (w_force.c:2029) — the FP_DRAIN activator (the hand-shoot
/// half; the per-target damage/drain lives in [`ForceDrainDamage`]). The twin of
/// `ForceLightning`: bails on death, while a hand gesture is in progress, while the weapon is busy,
/// on `forcePower < 25` or `!WP_ForcePowerUsable(FP_DRAIN)`, or while the `forcePowerDebounce[FP_DRAIN]`
/// cool-down is pending; else arms a 20 s force-hold hand-extend, plays `drain.wav`, and fires
/// `WP_ForcePowerStart(FP_DRAIN, 500)`. The C's commented-out alternate hand-extend is carried inert.
/// A clean dep-clean leaf; its dispatch caller (`WP_DoSpecificPower`) stays unported.
pub unsafe fn ForceDrain(self_: *mut gentity_t) {
    if (*self_).health <= 0 {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.forceHandExtend != HANDEXTEND_NONE {
        return;
    }

    if (*client).ps.weaponTime > 0 {
        return;
    }

    if (*client).ps.fd.forcePower < 25 || WP_ForcePowerUsable(self_, FP_DRAIN) == QFALSE {
        return;
    }
    if (*client).ps.fd.forcePowerDebounce[FP_DRAIN as usize] > (*addr_of!(level)).time {
        // stops it while using it and also after using it, up to 3 second delay
        return;
    }

    //	self->client->ps.forceHandExtend = HANDEXTEND_FORCEPUSH;
    //	self->client->ps.forceHandExtendTime = level.time + 1000;
    (*client).ps.forceHandExtend = HANDEXTEND_FORCE_HOLD;
    (*client).ps.forceHandExtendTime = (*addr_of!(level)).time + 20000;

    G_Sound(self_, CHAN_BODY, G_SoundIndex("sound/weapons/force/drain.wav"));

    WP_ForcePowerStart(self_, FP_DRAIN, 500);
}

/// `void ForceDrainDamage( gentity_t *self, gentity_t *traceEnt, vec3_t dir, vec3_t impactPoint )`
/// (w_force.c:2065) — apply the per-target FP_DRAIN effect to `traceEnt`: drains its `forcePower`
/// (2/3/4 by the caster's drain level, with `WP_AbsorbConversion` knocking that down to 0/1/2 if the
/// target is absorbing), heals the caster by the drained amount (capped at max health), pushes back
/// the target's force-regen debounce, and (subject to its sound debounce) spawns one
/// `EV_FORCE_DRAINED` temp-entity at `impactPoint` carrying `DirToByte(dir)` and the target's number.
/// Clears the caster's `EF_INVULNERABLE` / invuln-timer and stamps its `dangerTime` up front.
/// Gated on `traceEnt->takedamage`, an enemy (or friendly-fire) client with force to drain, the
/// caster's `forceDrainTime` cool-down, and `ForcePowerUsableOn(FP_DRAIN)`.
///
/// The original `G_Damage`/`G_Sound`/`BG_ForcePowerDrain` calls are commented out in the C source
/// (drain does no direct HP damage and re-uses the activator's standard cost), so they are carried
/// over inert here verbatim.
///
/// No oracle — mutates the live gentity/playerState arrays (force/health fields), reads `level.time`
/// and spawns an `EV_FORCE_DRAINED` temp-entity (surrounding-w_force-leaf precedent).
pub unsafe fn ForceDrainDamage(
    self_: *mut gentity_t,
    trace_ent: *mut gentity_t,
    dir: &vec3_t,
    impact_point: &vec3_t,
) {
    let tent: *mut gentity_t;

    (*(*self_).client).dangerTime = (*addr_of!(level)).time;
    (*(*self_).client).ps.eFlags &= !EF_INVULNERABLE;
    (*(*self_).client).invulnerableTimer = 0;

    if !trace_ent.is_null() && (*trace_ent).takedamage != QFALSE {
        if !(*trace_ent).client.is_null()
            && (OnSameTeam(self_, trace_ent) == QFALSE || g_friendlyFire.integer != 0)
            && ((*(*self_).client).ps.fd.forceDrainTime as c_int) < (*addr_of!(level)).time
            && (*(*trace_ent).client).ps.fd.forcePower != 0
        {
            //an enemy or object
            if (*trace_ent).client.is_null() && (*trace_ent).s.eType == ET_NPC {
                //g2animent
                if (*trace_ent).s.genericenemyindex < (*addr_of!(level)).time {
                    (*trace_ent).s.genericenemyindex = (*addr_of!(level)).time + 2000;
                }
            }
            if ForcePowerUsableOn(self_, trace_ent, FP_DRAIN) != 0 {
                let mut mod_power_level: c_int = -1;
                let mut dmg: c_int = 0; //Q_irand( 1, 3 );
                if (*(*self_).client).ps.fd.forcePowerLevel[FP_DRAIN as usize] == FORCE_LEVEL_1 {
                    dmg = 2; //because it's one-shot
                } else if (*(*self_).client).ps.fd.forcePowerLevel[FP_DRAIN as usize]
                    == FORCE_LEVEL_2
                {
                    dmg = 3;
                } else if (*(*self_).client).ps.fd.forcePowerLevel[FP_DRAIN as usize]
                    == FORCE_LEVEL_3
                {
                    dmg = 4;
                }

                if !(*trace_ent).client.is_null() {
                    mod_power_level = WP_AbsorbConversion(
                        trace_ent,
                        (*(*trace_ent).client).ps.fd.forcePowerLevel[FP_ABSORB as usize],
                        self_,
                        FP_DRAIN,
                        (*(*self_).client).ps.fd.forcePowerLevel[FP_DRAIN as usize],
                        1,
                    );
                }

                if mod_power_level != -1 {
                    if mod_power_level == 0 {
                        dmg = 0;
                    } else if mod_power_level == 1 {
                        dmg = 1;
                    } else if mod_power_level == 2 {
                        dmg = 2;
                    }
                }
                //G_Damage( traceEnt, self, self, dir, impactPoint, dmg, 0, MOD_FORCE_DARK );

                if dmg != 0 {
                    (*(*trace_ent).client).ps.fd.forcePower -= dmg;
                }
                if (*(*trace_ent).client).ps.fd.forcePower < 0 {
                    (*(*trace_ent).client).ps.fd.forcePower = 0;
                }

                if (*(*self_).client).ps.stats[STAT_HEALTH as usize]
                    < (*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize]
                    && (*self_).health > 0
                    && (*(*self_).client).ps.stats[STAT_HEALTH as usize] > 0
                {
                    (*self_).health += dmg;
                    if (*self_).health > (*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize] {
                        (*self_).health = (*(*self_).client).ps.stats[STAT_MAX_HEALTH as usize];
                    }
                    (*(*self_).client).ps.stats[STAT_HEALTH as usize] = (*self_).health;
                }

                (*(*trace_ent).client).ps.fd.forcePowerRegenDebounceTime =
                    (*addr_of!(level)).time + 800; //don't let the client being drained get force power back right away

                //Drain the standard amount since we just drained someone else

                /*
                if (self->client->ps.fd.forcePowerLevel[FP_DRAIN] == FORCE_LEVEL_1)
                {
                    BG_ForcePowerDrain( &self->client->ps, FP_DRAIN, 0 );
                }
                else
                {
                    BG_ForcePowerDrain( &self->client->ps, FP_DRAIN, forcePowerNeeded[self->client->ps.fd.forcePowerLevel[FP_DRAIN]][FP_DRAIN]/5 );
                }

                if (self->client->ps.fd.forcePowerLevel[FP_DRAIN] == FORCE_LEVEL_1)
                {
                    self->client->ps.fd.forceDrainTime = level.time + 100;
                }
                else
                {
                    self->client->ps.fd.forceDrainTime = level.time + 20;
                }

                if ( traceEnt->client )
                {
                    if ( !Q_irand( 0, 2 ) )
                    {
                        //G_Sound( traceEnt, CHAN_BODY, G_SoundIndex( "sound/weapons/force/lightninghit.wav" ) );
                    }
                //	traceEnt->s.powerups |= ( 1 << PW_DISINT_1 );

                //	traceEnt->client->ps.powerups[PW_DISINT_1] = level.time + 500;
                }
                */

                if (*(*trace_ent).client).forcePowerSoundDebounce < (*addr_of!(level)).time {
                    tent = G_TempEntity(impact_point, EV_FORCE_DRAINED);
                    (*tent).s.eventParm = DirToByte(dir);
                    (*tent).s.owner = (*trace_ent).s.number;

                    (*(*trace_ent).client).forcePowerSoundDebounce =
                        (*addr_of!(level)).time + 400;
                }
            }
        }
    }
}

/// `int ForceShootDrain( gentity_t *self )` (w_force.c:2191) — fire the FP_DRAIN attack.
/// At power level > 2 it drains every valid enemy client within `MAX_DRAIN_DISTANCE` inside the
/// forward cone (a `trap_EntitiesInBox` broad-phase plus per-entity cone/distance/PVS/LOS
/// checks, each surviving target fed to `ForceDrainDamage`); otherwise it traces a single line
/// forward and drains whatever client it hits. Sets `activeForcePass`, spends drain power via
/// `BG_ForcePowerDrain`, stamps the regen debounce, and returns whether anyone was drained.
///
/// No oracle — drain-trace force callback: `trap_EntitiesInBox`/`trap_Trace`/`trap_InPVS` plus
/// `ForceDrainDamage` mutate the live entity array and engine collision state (entity/trap
/// control-flow precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`. `g_entities`/`level` must be
/// initialised (the arc branch indexes `g_entities`, and `ForceDrainDamage`/the regen stamp
/// read `level.time`).
pub unsafe fn ForceShootDrain(self_: *mut gentity_t) -> c_int {
    let mut tr: trace_t;
    let mut end: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut traceEnt: *mut gentity_t;
    let mut gotOneOrMore: c_int = 0;

    if (*self_).health <= 0 {
        return 0;
    }
    AngleVectors(
        &(*(*self_).client).ps.viewangles,
        Some(&mut forward),
        None,
        None,
    );
    VectorNormalize(&mut forward);

    if (*(*self_).client).ps.fd.forcePowerLevel[FP_DRAIN as usize] > FORCE_LEVEL_2 {
        //arc
        let mut center: vec3_t = [0.0; 3];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut ent_org: vec3_t = [0.0; 3];
        let mut size: vec3_t = [0.0; 3];
        let mut v: vec3_t = [0.0; 3];
        let radius: f32 = MAX_DRAIN_DISTANCE as f32;
        let mut dot: f32;
        let mut dist: f32;
        let mut entityList: [*mut gentity_t; MAX_GENTITIES] = [core::ptr::null_mut(); MAX_GENTITIES];
        let mut iEntityList: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
        let numListedEntities: c_int;
        let mut i: c_int;

        VectorCopy(&(*(*self_).client).ps.origin, &mut center);
        for i in 0..3 {
            mins[i] = center[i] - radius;
            maxs[i] = center[i] + radius;
        }
        numListedEntities = crate::trap::EntitiesInBox(&mins, &maxs, &mut iEntityList);

        i = 0;
        while i < numListedEntities {
            entityList[i as usize] = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(iEntityList[i as usize] as usize);

            i += 1;
        }

        for e in 0..numListedEntities {
            traceEnt = entityList[e as usize];

            if traceEnt.is_null() {
                continue;
            }
            if traceEnt == self_ {
                continue;
            }
            if (*traceEnt).inuse == QFALSE {
                continue;
            }
            if (*traceEnt).takedamage == QFALSE {
                continue;
            }
            if (*traceEnt).health <= 0 {
                //no torturing corpses
                continue;
            }
            if (*traceEnt).client.is_null() {
                continue;
            }
            if (*(*traceEnt).client).ps.fd.forcePower == 0 {
                continue;
            }
            if OnSameTeam(self_, traceEnt) != QFALSE && g_friendlyFire.integer == 0 {
                continue;
            }
            //this is all to see if we need to start a saber attack, if it's in flight, this doesn't matter
            // find the distance from the edge of the bounding box
            for i in 0..3 {
                if center[i] < (*traceEnt).r.absmin[i] {
                    v[i] = (*traceEnt).r.absmin[i] - center[i];
                } else if center[i] > (*traceEnt).r.absmax[i] {
                    v[i] = center[i] - (*traceEnt).r.absmax[i];
                } else {
                    v[i] = 0.0;
                }
            }

            VectorSubtract(&(*traceEnt).r.absmax, &(*traceEnt).r.absmin, &mut size);
            VectorMA(&(*traceEnt).r.absmin, 0.5, &size, &mut ent_org);

            //see if they're in front of me
            //must be within the forward cone
            VectorSubtract(&ent_org, &center, &mut dir);
            VectorNormalize(&mut dir);
            dot = DotProduct(&dir, &forward);
            if dot < 0.5 {
                continue;
            }

            //must be close enough
            dist = VectorLength(&v);
            if dist >= radius {
                continue;
            }

            //in PVS?
            if (*traceEnt).r.bmodel == QFALSE
                && crate::trap::InPVS(&ent_org, &(*(*self_).client).ps.origin) == QFALSE
            {
                //must be in PVS
                continue;
            }

            //Now check and see if we can actually hit it
            tr = crate::trap::Trace(
                &(*(*self_).client).ps.origin,
                &vec3_origin,
                &vec3_origin,
                &ent_org,
                (*self_).s.number,
                MASK_SHOT,
            );
            if tr.fraction < 1.0f32 && (tr.entityNum as c_int) != (*traceEnt).s.number {
                //must have clear LOS
                continue;
            }

            // ok, we are within the radius, add us to the incoming list
            ForceDrainDamage(self_, traceEnt, &dir, &ent_org);
            gotOneOrMore = 1;
        }
    } else {
        //trace-line
        VectorMA(&(*(*self_).client).ps.origin, 2048.0, &forward, &mut end);

        tr = crate::trap::Trace(
            &(*(*self_).client).ps.origin,
            &vec3_origin,
            &vec3_origin,
            &end,
            (*self_).s.number,
            MASK_SHOT,
        );
        if tr.entityNum as c_int == ENTITYNUM_NONE
            || tr.fraction == 1.0
            || tr.allsolid != 0
            || tr.startsolid != 0
            || (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize)).client.is_null()
            || (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize)).inuse == QFALSE
        {
            return 0;
        }

        traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);
        ForceDrainDamage(self_, traceEnt, &forward, &tr.endpos);
        gotOneOrMore = 1;
    }

    (*(*self_).client).ps.activeForcePass =
        (*(*self_).client).ps.fd.forcePowerLevel[FP_DRAIN as usize] + FORCE_LEVEL_3;

    BG_ForcePowerDrain(&mut (*(*self_).client).ps, FP_DRAIN, 5); //used to be 1, but this did, too, anger the God of Balance.

    (*(*self_).client).ps.fd.forcePowerRegenDebounceTime = (*addr_of!(level)).time + 500;

    gotOneOrMore
}

/// `void ForceTeamHeal( gentity_t *self )` (w_force.c:1326) — the FP_TEAM_HEAL activator.
/// Heals every same-team client within radius (scaled by power level) that is hurt, alive,
/// usable-on and in PVS; per-target heal amount scales down with the count (50/33/25 for
/// 1/2/3+). On the first successful target it spawns one collective `EV_TEAM_POWER`
/// temp-entity (`eventParm = 1` = heal) and drains the caster's force once; every healed
/// target is folded into that single event via the client bitflags. Bails on death,
/// `!WP_ForcePowerUsable`, or while the 2s debounce is still pending.
///
/// No oracle — reads/mutates the live gentity/playerState array, `level.time`, and calls
/// `trap_InPVS`/`G_TempEntity` (surrounding-w_force-leaf precedent).
pub unsafe fn ForceTeamHeal(self_: *mut gentity_t) {
    let mut radius: f32 = 256.0;
    let mut i: c_int = 0;
    let mut a: vec3_t = [0.0; 3];
    let mut numpl: c_int = 0;
    let mut pl: [c_int; MAX_CLIENTS] = [0; MAX_CLIENTS];
    let healthadd: c_int;
    let mut te: *mut gentity_t = core::ptr::null_mut();

    if (*self_).health <= 0 {
        return;
    }

    if WP_ForcePowerUsable(self_, FP_TEAM_HEAL) == QFALSE {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.fd.forcePowerDebounce[FP_TEAM_HEAL as usize] >= (*addr_of!(level)).time {
        return;
    }

    if (*client).ps.fd.forcePowerLevel[FP_TEAM_HEAL as usize] == FORCE_LEVEL_2 {
        radius *= 1.5;
    }
    if (*client).ps.fd.forcePowerLevel[FP_TEAM_HEAL as usize] == FORCE_LEVEL_3 {
        radius *= 2.0;
    }

    while (i as usize) < MAX_CLIENTS {
        let ent_loop = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent_loop.is_null()
            && !(*ent_loop).client.is_null()
            && self_ != ent_loop
            && OnSameTeam(self_, ent_loop) != QFALSE
            && (*(*ent_loop).client).ps.stats[STAT_HEALTH as usize]
                < (*(*ent_loop).client).ps.stats[STAT_MAX_HEALTH as usize]
            && (*(*ent_loop).client).ps.stats[STAT_HEALTH as usize] > 0
            && ForcePowerUsableOn(self_, ent_loop, FP_TEAM_HEAL) != 0
            && crate::trap::InPVS(&(*client).ps.origin, &(*(*ent_loop).client).ps.origin) != QFALSE
        {
            VectorSubtract(&(*client).ps.origin, &(*(*ent_loop).client).ps.origin, &mut a);

            if VectorLength(&a) <= radius {
                pl[numpl as usize] = i;
                numpl += 1;
            }
        }

        i += 1;
    }

    if numpl < 1 {
        return;
    }

    if numpl == 1 {
        healthadd = 50;
    } else if numpl == 2 {
        healthadd = 33;
    } else {
        healthadd = 25;
    }

    (*client).ps.fd.forcePowerDebounce[FP_TEAM_HEAL as usize] = (*addr_of!(level)).time + 2000;
    i = 0;

    while i < numpl {
        let target = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(pl[i as usize] as usize);
        if (*(*target).client).ps.stats[STAT_HEALTH as usize] > 0 && (*target).health > 0 {
            (*(*target).client).ps.stats[STAT_HEALTH as usize] += healthadd;
            if (*(*target).client).ps.stats[STAT_HEALTH as usize]
                > (*(*target).client).ps.stats[STAT_MAX_HEALTH as usize]
            {
                (*(*target).client).ps.stats[STAT_HEALTH as usize] =
                    (*(*target).client).ps.stats[STAT_MAX_HEALTH as usize];
            }

            (*target).health = (*(*target).client).ps.stats[STAT_HEALTH as usize];

            //At this point we know we got one, so add him into the collective event client bitflag
            if te.is_null() {
                te = G_TempEntity(&(*client).ps.origin, EV_TEAM_POWER);
                (*te).s.eventParm = 1; //eventParm 1 is heal, eventParm 2 is force regen

                //since we had an extra check above, do the drain now because we got at least one guy
                BG_ForcePowerDrain(
                    &mut (*client).ps,
                    FP_TEAM_HEAL,
                    forcePowerNeeded[(*client).ps.fd.forcePowerLevel[FP_TEAM_HEAL as usize] as usize]
                        [FP_TEAM_HEAL as usize],
                );
            }

            WP_AddToClientBitflags(te, pl[i as usize]);
            //Now cramming it all into one event.. doing this many g_sound events at once was a Bad Thing.
        }
        i += 1;
    }
}

/// `void ForceTeamForceReplenish( gentity_t *self )` (w_force.c:1431) — the FP_TEAM_FORCE
/// activator; the force-regen twin of [`ForceTeamHeal`]. Adds force power (50/33/25 by team
/// count) to every same-team client under 100 force, within the level-scaled radius, usable-on
/// and in PVS. Unlike heal it drains the caster *unconditionally* once a target was found
/// (mirroring the C, which drains before the apply loop), spawns one `EV_TEAM_POWER`
/// temp-entity with `eventParm = 2` (force regen), and folds each target into the event
/// bitflags. Bails on death, `!WP_ForcePowerUsable`, or while the 2s debounce is pending.
///
/// No oracle — same live-state/`trap_InPVS`/`G_TempEntity` reasoning as [`ForceTeamHeal`].
pub unsafe fn ForceTeamForceReplenish(self_: *mut gentity_t) {
    let mut radius: f32 = 256.0;
    let mut i: c_int = 0;
    let mut a: vec3_t = [0.0; 3];
    let mut numpl: c_int = 0;
    let mut pl: [c_int; MAX_CLIENTS] = [0; MAX_CLIENTS];
    let poweradd: c_int;
    let mut te: *mut gentity_t = core::ptr::null_mut();

    if (*self_).health <= 0 {
        return;
    }

    if WP_ForcePowerUsable(self_, FP_TEAM_FORCE) == QFALSE {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.fd.forcePowerDebounce[FP_TEAM_FORCE as usize] >= (*addr_of!(level)).time {
        return;
    }

    if (*client).ps.fd.forcePowerLevel[FP_TEAM_FORCE as usize] == FORCE_LEVEL_2 {
        radius *= 1.5;
    }
    if (*client).ps.fd.forcePowerLevel[FP_TEAM_FORCE as usize] == FORCE_LEVEL_3 {
        radius *= 2.0;
    }

    while (i as usize) < MAX_CLIENTS {
        let ent_loop = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent_loop.is_null()
            && !(*ent_loop).client.is_null()
            && self_ != ent_loop
            && OnSameTeam(self_, ent_loop) != QFALSE
            && (*(*ent_loop).client).ps.fd.forcePower < 100
            && ForcePowerUsableOn(self_, ent_loop, FP_TEAM_FORCE) != 0
            && crate::trap::InPVS(&(*client).ps.origin, &(*(*ent_loop).client).ps.origin) != QFALSE
        {
            VectorSubtract(&(*client).ps.origin, &(*(*ent_loop).client).ps.origin, &mut a);

            if VectorLength(&a) <= radius {
                pl[numpl as usize] = i;
                numpl += 1;
            }
        }

        i += 1;
    }

    if numpl < 1 {
        return;
    }

    if numpl == 1 {
        poweradd = 50;
    } else if numpl == 2 {
        poweradd = 33;
    } else {
        poweradd = 25;
    }
    (*client).ps.fd.forcePowerDebounce[FP_TEAM_FORCE as usize] = (*addr_of!(level)).time + 2000;

    BG_ForcePowerDrain(
        &mut (*client).ps,
        FP_TEAM_FORCE,
        forcePowerNeeded[(*client).ps.fd.forcePowerLevel[FP_TEAM_FORCE as usize] as usize]
            [FP_TEAM_FORCE as usize],
    );

    i = 0;

    while i < numpl {
        let target = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(pl[i as usize] as usize);
        (*(*target).client).ps.fd.forcePower += poweradd;
        if (*(*target).client).ps.fd.forcePower > 100 {
            (*(*target).client).ps.fd.forcePower = 100;
        }

        //At this point we know we got one, so add him into the collective event client bitflag
        if te.is_null() {
            te = G_TempEntity(&(*client).ps.origin, EV_TEAM_POWER);
            (*te).s.eventParm = 2; //eventParm 1 is heal, eventParm 2 is force regen
        }

        WP_AddToClientBitflags(te, pl[i as usize]);
        //Now cramming it all into one event.. doing this many g_sound events at once was a Bad Thing.

        i += 1;
    }
}

/// `qboolean Jedi_DodgeEvasion( gentity_t *self, gentity_t *shooter, trace_t *tr, int hitLoc )`
/// (w_force.c:5681) — the force-dodge reflex: when a shot is about to hit, see if `self` can
/// snap into a lean-dodge anim. Gated on being alive, `g_forceDodge` enabled, on the ground,
/// not mid-action (no `weaponTime`/`forceHandExtend`). With `g_forceDodge == 2` it costs/checks
/// Force Speed and rolls `Q_irand(1,7)` against the speed level; otherwise it requires level-3
/// Force Sight and always dodges. The `hitLoc` picks the directional dodge anim; on success it
/// forces the anim via `forceHandExtend`/`forceDodgeAnim`, grants a brief `PW_SPEEDBURST`, and
/// either runs `ForceSpeed`/plays the speed sound.
///
/// `shooter` and `tr` are unused in the retail code (the dodge is purely self-state-driven), kept
/// in the signature to match the C ABI.
pub unsafe fn Jedi_DodgeEvasion(
    self_: *mut gentity_t,
    _shooter: *mut gentity_t,
    _tr: *mut trace_t,
    hitLoc: c_int,
) -> qboolean {
    // Faithful to the C: `dodgeAnim` is init to -1 and a final `!= -1` guard wraps the apply
    // block, but every reaching path through the `match` either returns or assigns, so the -1
    // is never actually read. Kept verbatim to mirror the source's defensive structure.
    #[allow(unused_assignments)]
    let mut dodgeAnim: c_int = -1;

    if self_.is_null() || (*self_).client.is_null() || (*self_).health <= 0 {
        return QFALSE;
    }

    let client = (*self_).client;

    if (*addr_of!(g_forceDodge)).integer == 0 {
        return QFALSE;
    }

    if (*addr_of!(g_forceDodge)).integer != 2
        && (*client).ps.fd.forcePowersActive & (1 << FP_SEE) == 0
    {
        return QFALSE;
    }

    if (*client).ps.groundEntityNum == ENTITYNUM_NONE {
        // can't dodge in mid-air
        return QFALSE;
    }

    if (*client).ps.weaponTime > 0 || (*client).ps.forceHandExtend != HANDEXTEND_NONE {
        // in some effect that stops me from moving on my own
        return QFALSE;
    }

    if (*addr_of!(g_forceDodge)).integer == 2 {
        if (*client).ps.fd.forcePowersActive != 0 {
            // for now just don't let us dodge if we're using a force power at all
            return QFALSE;
        }
    }

    if (*addr_of!(g_forceDodge)).integer == 2 {
        if WP_ForcePowerUsable(self_, FP_SPEED) == QFALSE {
            // make sure we have it and have enough force power
            return QFALSE;
        }
    }

    if (*addr_of!(g_forceDodge)).integer == 2 {
        if Q_irand(1, 7) > (*client).ps.fd.forcePowerLevel[FP_SPEED as usize] {
            // more likely to fail on lower force speed level
            return QFALSE;
        }
    } else {
        // We now dodge all the time, but only on level 3
        if (*client).ps.fd.forcePowerLevel[FP_SEE as usize] < FORCE_LEVEL_3 {
            // more likely to fail on lower force sight level
            return QFALSE;
        }
    }

    match hitLoc {
        HL_NONE => {
            return QFALSE;
        }

        HL_FOOT_RT | HL_FOOT_LT | HL_LEG_RT | HL_LEG_LT => {
            return QFALSE;
        }

        HL_BACK_RT => {
            dodgeAnim = BOTH_DODGE_FL;
        }
        HL_CHEST_RT => {
            dodgeAnim = BOTH_DODGE_FR;
        }
        HL_BACK_LT => {
            dodgeAnim = BOTH_DODGE_FR;
        }
        HL_CHEST_LT => {
            dodgeAnim = BOTH_DODGE_FR;
        }
        HL_BACK | HL_CHEST | HL_WAIST => {
            dodgeAnim = BOTH_DODGE_FL;
        }
        HL_ARM_RT | HL_HAND_RT => {
            dodgeAnim = BOTH_DODGE_L;
        }
        HL_ARM_LT | HL_HAND_LT => {
            dodgeAnim = BOTH_DODGE_R;
        }
        HL_HEAD => {
            dodgeAnim = BOTH_DODGE_FL;
        }
        _ => {
            return QFALSE;
        }
    }

    if dodgeAnim != -1 {
        // Our own happy way of forcing an anim:
        (*client).ps.forceHandExtend = HANDEXTEND_DODGE;
        (*client).ps.forceDodgeAnim = dodgeAnim;
        (*client).ps.forceHandExtendTime = (*addr_of!(level)).time + 300;

        (*client).ps.powerups[PW_SPEEDBURST as usize] = (*addr_of!(level)).time + 100;

        if (*addr_of!(g_forceDodge)).integer == 2 {
            ForceSpeed(self_, 500);
        } else {
            G_Sound(
                self_,
                CHAN_BODY,
                G_SoundIndex("sound/weapons/force/speed.wav"),
            );
        }
        return QTRUE;
    }
    QFALSE
}

/// `void FindGenericEnemyIndex(gentity_t *self)` (w_force.c:4671) — find another client that
/// would be considered a threat (the seeker-drone / mind-trick target picker). Scans every
/// client slot for a live, hostile, in-game player that is the *closest* one both in front of
/// `self` (within an 0.8 facing dot, via [`InFront`]) and visible (LOS via [`OrgVisible`]),
/// then records its entity number in `self->client->ps.genericEnemyIndex`. Leaves the index
/// untouched when no candidate qualifies.
///
/// # Safety
/// Dereferences `self_` and its `client`, and reads `g_entities[i].client` for every slot;
/// the caller must pass a valid client entity and an initialised `g_entities` array.
pub unsafe fn FindGenericEnemyIndex(self_: *mut gentity_t) {
    //Find another client that would be considered a threat.
    let mut i: c_int = 0;
    let mut tlen: f32;
    let mut ent: *mut gentity_t;
    let mut besten: *mut gentity_t = core::ptr::null_mut();
    let mut blen: f32 = 99999999.0;
    let mut a: vec3_t = [0.0; 3];

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null()
            && !(*ent).client.is_null()
            && (*ent).s.number != (*self_).s.number
            && (*ent).health > 0
            && OnSameTeam(self_, ent) == QFALSE
            && (*(*ent).client).ps.pm_type != PM_INTERMISSION
            && (*(*ent).client).ps.pm_type != PM_SPECTATOR
        {
            VectorSubtract(&(*(*ent).client).ps.origin, &(*(*self_).client).ps.origin, &mut a);
            tlen = VectorLength(&a);

            if tlen < blen
                && InFront(
                    &(*(*ent).client).ps.origin,
                    &(*(*self_).client).ps.origin,
                    &(*(*self_).client).ps.viewangles,
                    0.8,
                ) != QFALSE
                && OrgVisible(
                    &(*(*self_).client).ps.origin,
                    &(*(*ent).client).ps.origin,
                    (*self_).s.number,
                ) != 0
            {
                blen = tlen;
                besten = ent;
            }
        }

        i += 1;
    }

    if besten.is_null() {
        return;
    }

    (*(*self_).client).ps.genericEnemyIndex = (*besten).s.number;
}

/// `void ForceLightningDamage( gentity_t *self, gentity_t *traceEnt, vec3_t dir, vec3_t impactPoint )`
/// (w_force.c:1819) — apply one tick of FP_LIGHTNING to `traceEnt`. Clears `self`'s
/// invulnerability danger state, then (for a damageable target) bumps the g2animent enemy-index,
/// hands force power to a client still in its `noLightningTime` grace, else rolls a small damage
/// figure (doubled for 2-handed melee lightning), runs it through `WP_AbsorbConversion`, deals
/// `G_Damage` (MOD_FORCE_DARK), plays a random lightning-hit sound, stamps the electrify time and
/// decloaks a cloaked victim.
///
/// No oracle — mutates the live gentity/playerState array and calls `G_Damage`/`G_Sound`/
/// `Jedi_Decloak` (entity/trap control-flow precedent, mirroring `ForceDrainDamage`).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`; `level` must be initialised.
pub unsafe fn ForceLightningDamage(
    self_: *mut gentity_t,
    trace_ent: *mut gentity_t,
    dir: *mut vec3_t,
    impact_point: *mut vec3_t,
) {
    (*(*self_).client).dangerTime = (*addr_of!(level)).time;
    (*(*self_).client).ps.eFlags &= !EF_INVULNERABLE;
    (*(*self_).client).invulnerableTimer = 0;

    if !trace_ent.is_null() && (*trace_ent).takedamage != QFALSE {
        if (*trace_ent).client.is_null() && (*trace_ent).s.eType == ET_NPC {
            //g2animent
            if (*trace_ent).s.genericenemyindex < (*addr_of!(level)).time {
                (*trace_ent).s.genericenemyindex = (*addr_of!(level)).time + 2000;
            }
        }
        if !(*trace_ent).client.is_null() {
            //an enemy or object
            if (*(*trace_ent).client).noLightningTime >= (*addr_of!(level)).time {
                //give them power and don't hurt them.
                (*(*trace_ent).client).ps.fd.forcePower += 1;
                if (*(*trace_ent).client).ps.fd.forcePower > 100 {
                    (*(*trace_ent).client).ps.fd.forcePower = 100;
                }
                return;
            }
            if ForcePowerUsableOn(self_, trace_ent, FP_LIGHTNING) != 0 {
                let mut dmg: c_int = Q_irand(1, 2); //Q_irand( 1, 3 );

                let mut mod_power_level: c_int = -1;

                if !(*trace_ent).client.is_null() {
                    mod_power_level = WP_AbsorbConversion(
                        trace_ent,
                        (*(*trace_ent).client).ps.fd.forcePowerLevel[FP_ABSORB as usize],
                        self_,
                        FP_LIGHTNING,
                        (*(*self_).client).ps.fd.forcePowerLevel[FP_LIGHTNING as usize],
                        1,
                    );
                }

                if mod_power_level != -1 {
                    if mod_power_level == 0 {
                        dmg = 0;
                        (*(*trace_ent).client).noLightningTime = (*addr_of!(level)).time + 400;
                    } else if mod_power_level == 1 {
                        dmg = 1;
                        (*(*trace_ent).client).noLightningTime = (*addr_of!(level)).time + 300;
                    } else if mod_power_level == 2 {
                        dmg = 1;
                        (*(*trace_ent).client).noLightningTime = (*addr_of!(level)).time + 100;
                    }
                }

                if (*(*self_).client).ps.weapon == WP_MELEE
                    && (*(*self_).client).ps.fd.forcePowerLevel[FP_LIGHTNING as usize]
                        > FORCE_LEVEL_2
                {
                    //2-handed lightning
                    //jackin' 'em up, Palpatine-style
                    dmg *= 2;
                }

                if dmg != 0 {
                    //rww - Shields can now absorb lightning too.
                    G_Damage(
                        trace_ent,
                        self_,
                        self_,
                        dir,
                        impact_point,
                        dmg,
                        0,
                        MOD_FORCE_DARK,
                    );
                }
                if !(*trace_ent).client.is_null() {
                    if Q_irand(0, 2) == 0 {
                        G_Sound(
                            trace_ent,
                            CHAN_BODY,
                            G_SoundIndex(&format!(
                                "sound/weapons/force/lightninghit{}",
                                Q_irand(1, 3)
                            )),
                        );
                    }

                    if (*(*trace_ent).client).ps.electrifyTime < ((*addr_of!(level)).time + 400) {
                        //only update every 400ms to reduce bandwidth usage (as it is passing a 32-bit time value)
                        (*(*trace_ent).client).ps.electrifyTime = (*addr_of!(level)).time + 800;
                    }
                    if (*(*trace_ent).client).ps.powerups[PW_CLOAKED as usize] != 0 {
                        //disable cloak temporarily
                        Jedi_Decloak(trace_ent);
                        (*(*trace_ent).client).cloakToggleTime =
                            (*addr_of!(level)).time + Q_irand(3000, 10000);
                    }
                }
            }
        }
    }
}

/// `void ForceShootLightning( gentity_t *self )` (w_force.c:1909) — fire the FP_LIGHTNING attack.
/// At power level > 2 it arcs to every valid target within `FORCE_LIGHTNING_RADIUS` inside the
/// forward cone (a `trap_EntitiesInBox` broad-phase plus per-entity cone/distance/PVS/LOS checks,
/// each surviving target fed to `ForceLightningDamage`); otherwise it traces a single line forward
/// and zaps whatever it hits.
///
/// No oracle — lightning-trace force callback: `trap_EntitiesInBox`/`trap_Trace`/`trap_InPVS` plus
/// `ForceLightningDamage` mutate the live entity array and engine collision state (entity/trap
/// control-flow precedent, mirroring `ForceShootDrain`).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`. `g_entities`/`level` must be
/// initialised (the arc branch indexes `g_entities`, and `ForceLightningDamage` reads `level.time`).
pub unsafe fn ForceShootLightning(self_: *mut gentity_t) {
    let mut tr: trace_t;
    let mut end: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut traceEnt: *mut gentity_t;

    if (*self_).health <= 0 {
        return;
    }
    AngleVectors(
        &(*(*self_).client).ps.viewangles,
        Some(&mut forward),
        None,
        None,
    );
    VectorNormalize(&mut forward);

    if (*(*self_).client).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] > FORCE_LEVEL_2 {
        //arc
        let mut center: vec3_t = [0.0; 3];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut ent_org: vec3_t = [0.0; 3];
        let mut size: vec3_t = [0.0; 3];
        let mut v: vec3_t = [0.0; 3];
        let radius: f32 = FORCE_LIGHTNING_RADIUS as f32;
        let mut dot: f32;
        let mut dist: f32;
        let mut entityList: [*mut gentity_t; MAX_GENTITIES] =
            [core::ptr::null_mut(); MAX_GENTITIES];
        let mut iEntityList: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
        let numListedEntities: c_int;
        let mut i: c_int;

        VectorCopy(&(*(*self_).client).ps.origin, &mut center);
        for i in 0..3 {
            mins[i] = center[i] - radius;
            maxs[i] = center[i] + radius;
        }
        numListedEntities = crate::trap::EntitiesInBox(&mins, &maxs, &mut iEntityList);

        i = 0;
        while i < numListedEntities {
            entityList[i as usize] =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(iEntityList[i as usize] as usize);

            i += 1;
        }

        for e in 0..numListedEntities {
            traceEnt = entityList[e as usize];

            if traceEnt.is_null() {
                continue;
            }
            if traceEnt == self_ {
                continue;
            }
            if (*traceEnt).r.ownerNum == (*self_).s.number && (*traceEnt).s.weapon != WP_THERMAL {
                //can push your own thermals
                continue;
            }
            if (*traceEnt).inuse == QFALSE {
                continue;
            }
            if (*traceEnt).takedamage == QFALSE {
                continue;
            }
            if (*traceEnt).health <= 0 {
                //no torturing corpses
                continue;
            }
            if g_friendlyFire.integer == 0 && OnSameTeam(self_, traceEnt) != QFALSE {
                continue;
            }
            //this is all to see if we need to start a saber attack, if it's in flight, this doesn't matter
            // find the distance from the edge of the bounding box
            for i in 0..3 {
                if center[i] < (*traceEnt).r.absmin[i] {
                    v[i] = (*traceEnt).r.absmin[i] - center[i];
                } else if center[i] > (*traceEnt).r.absmax[i] {
                    v[i] = center[i] - (*traceEnt).r.absmax[i];
                } else {
                    v[i] = 0.0;
                }
            }

            VectorSubtract(&(*traceEnt).r.absmax, &(*traceEnt).r.absmin, &mut size);
            VectorMA(&(*traceEnt).r.absmin, 0.5, &size, &mut ent_org);

            //see if they're in front of me
            //must be within the forward cone
            VectorSubtract(&ent_org, &center, &mut dir);
            VectorNormalize(&mut dir);
            dot = DotProduct(&dir, &forward);
            if dot < 0.5 {
                continue;
            }

            //must be close enough
            dist = VectorLength(&v);
            if dist >= radius {
                continue;
            }

            //in PVS?
            if (*traceEnt).r.bmodel == QFALSE
                && crate::trap::InPVS(&ent_org, &(*(*self_).client).ps.origin) == QFALSE
            {
                //must be in PVS
                continue;
            }

            //Now check and see if we can actually hit it
            tr = crate::trap::Trace(
                &(*(*self_).client).ps.origin,
                &vec3_origin,
                &vec3_origin,
                &ent_org,
                (*self_).s.number,
                MASK_SHOT,
            );
            if tr.fraction < 1.0f32 && (tr.entityNum as c_int) != (*traceEnt).s.number {
                //must have clear LOS
                continue;
            }

            // ok, we are within the radius, add us to the incoming list
            ForceLightningDamage(self_, traceEnt, &mut dir, &mut ent_org);
        }
    } else {
        //trace-line
        VectorMA(&(*(*self_).client).ps.origin, 2048.0, &forward, &mut end);

        tr = crate::trap::Trace(
            &(*(*self_).client).ps.origin,
            &vec3_origin,
            &vec3_origin,
            &end,
            (*self_).s.number,
            MASK_SHOT,
        );
        if tr.entityNum as c_int == ENTITYNUM_NONE
            || tr.fraction == 1.0
            || tr.allsolid != 0
            || tr.startsolid != 0
        {
            return;
        }

        traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);
        ForceLightningDamage(self_, traceEnt, &mut forward, &mut tr.endpos);
    }
}

/// `void ForceThrow( gentity_t *self, qboolean pull )` (w_force.c:3061) — the FP_PUSH/FP_PULL
/// activator: shove (or yank) everything in front of `self` away (towards). Gathers candidates
/// (a single targeted trace at level 1, else a `trap_EntitiesInBox` cone broad-phase), filters by
/// classname/team/PVS/LOS, then for each: clients get shoved with counter-throw resolution,
/// knockdowns, vehicle ejection and grip-break handling; missiles are reflected; force-usable
/// func_static/func_door movers are toggled and buttons pressed. Starts the power
/// (`WP_ForcePowerStart`), sets the hand-extend anim, and clears leftover grip state.
///
/// No oracle — gathers/mutates the live entity array via `trap_Trace`/`trap_EntitiesInBox`/
/// `trap_InPVS` and dozens of gentity/playerState writes (entity/trap control-flow precedent,
/// mirroring `ForceShootDrain`).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`; `g_entities`/`level` must be
/// initialised.
#[allow(unused_assignments)] // faithful: C dead-stores the initial `pushPower`/`powerUse` before the loop overwrites them
pub unsafe fn ForceThrow(self_: *mut gentity_t, pull: qboolean) {
    //shove things in front of you away
    let mut dist: f32;
    let mut ent: *mut gentity_t;
    let mut entityList: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
    let mut push_list: [*mut gentity_t; MAX_GENTITIES] = [core::ptr::null_mut(); MAX_GENTITIES];
    let mut numListedEntities: c_int;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut v: vec3_t = [0.0; 3];
    let mut e: c_int;
    let mut ent_count: c_int = 0;
    let radius: c_int = 1024; //since it's view-based now. //350;
    let powerLevel: c_int;
    let mut visionArc: c_int;
    let mut pushPower: c_int;
    let mut pushPowerMod: c_int;
    let mut center: vec3_t = [0.0; 3];
    let mut ent_org: vec3_t = [0.0; 3];
    let mut size: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut fwdangles: vec3_t = [0.0; 3];
    let mut dot1: f32;
    let mut tr: trace_t;
    let mut pushDir: vec3_t = [0.0; 3];
    let mut thispush_org: vec3_t = [0.0; 3];
    let mut tfrom: vec3_t = [0.0; 3];
    let mut tto: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut a: vec3_t = [0.0; 3];
    // C declares/assigns `knockback` but never reads it (dead local); kept verbatim, underscored.
    let mut _knockback: f32 = if pull != QFALSE { 0.0 } else { 200.0 };
    let mut powerUse: c_int = 0;

    visionArc = 0;

    if (*(*self_).client).ps.forceHandExtend != HANDEXTEND_NONE
        && ((*(*self_).client).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN
            || G_InGetUpAnim(&(*(*self_).client).ps) == QFALSE)
    {
        return;
    }

    if g_useWhileThrowing.integer == 0 && (*(*self_).client).ps.saberInFlight != QFALSE {
        return;
    }

    if (*(*self_).client).ps.weaponTime > 0 {
        return;
    }

    if (*self_).health <= 0 {
        return;
    }
    if (*(*self_).client).ps.powerups[PW_DISINT_4 as usize] > (*addr_of!(level)).time {
        return;
    }
    if pull != QFALSE {
        powerUse = FP_PULL;
    } else {
        powerUse = FP_PUSH;
    }

    if WP_ForcePowerUsable(self_, powerUse) == QFALSE {
        return;
    }

    if pull == QFALSE
        && (*(*self_).client).ps.saberLockTime > (*addr_of!(level)).time
        && (*(*self_).client).ps.saberLockFrame != 0
    {
        G_Sound(
            self_,
            CHAN_BODY,
            G_SoundIndex("sound/weapons/force/push.wav"),
        );
        (*(*self_).client).ps.powerups[PW_DISINT_4 as usize] = (*addr_of!(level)).time + 1500;

        (*(*self_).client).ps.saberLockHits +=
            (*(*self_).client).ps.fd.forcePowerLevel[FP_PUSH as usize] * 2;

        WP_ForcePowerStart(self_, FP_PUSH, 0);
        return;
    }

    WP_ForcePowerStart(self_, powerUse, 0);

    //make sure this plays and that you cannot press fire for about 1 second after this
    if pull != QFALSE {
        G_Sound(
            self_,
            CHAN_BODY,
            G_SoundIndex("sound/weapons/force/pull.wav"),
        );
        if (*(*self_).client).ps.forceHandExtend == HANDEXTEND_NONE {
            (*(*self_).client).ps.forceHandExtend = HANDEXTEND_FORCEPULL;
            if (*addr_of!(g_gametype)).integer == GT_SIEGE
                && (*(*self_).client).ps.weapon == WP_SABER
            {
                //hold less so can attack right after a pull
                (*(*self_).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 200;
            } else {
                (*(*self_).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 400;
            }
        }
        (*(*self_).client).ps.powerups[PW_DISINT_4 as usize] =
            (*(*self_).client).ps.forceHandExtendTime + 200;
        (*(*self_).client).ps.powerups[PW_PULL as usize] =
            (*(*self_).client).ps.powerups[PW_DISINT_4 as usize];
    } else {
        G_Sound(
            self_,
            CHAN_BODY,
            G_SoundIndex("sound/weapons/force/push.wav"),
        );
        if (*(*self_).client).ps.forceHandExtend == HANDEXTEND_NONE {
            (*(*self_).client).ps.forceHandExtend = HANDEXTEND_FORCEPUSH;
            (*(*self_).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1000;
        } else if (*(*self_).client).ps.forceHandExtend == HANDEXTEND_KNOCKDOWN
            && G_InGetUpAnim(&(*(*self_).client).ps) != QFALSE
        {
            if (*(*self_).client).ps.forceDodgeAnim > 4 {
                (*(*self_).client).ps.forceDodgeAnim -= 8;
            }
            (*(*self_).client).ps.forceDodgeAnim += 8; //special case, play push on upper torso, but keep playing current knockdown anim on legs
        }
        (*(*self_).client).ps.powerups[PW_DISINT_4 as usize] = (*addr_of!(level)).time + 1100;
        (*(*self_).client).ps.powerups[PW_PULL as usize] = 0;
    }

    VectorCopy(&(*(*self_).client).ps.viewangles, &mut fwdangles);
    AngleVectors(&fwdangles, Some(&mut forward), Some(&mut right), None);
    VectorCopy(&(*(*self_).client).ps.origin, &mut center);

    for i in 0..3 {
        mins[i] = center[i] - radius as f32;
        maxs[i] = center[i] + radius as f32;
    }

    if pull != QFALSE {
        powerLevel = (*(*self_).client).ps.fd.forcePowerLevel[FP_PULL as usize];
        pushPower = 256 * (*(*self_).client).ps.fd.forcePowerLevel[FP_PULL as usize];
    } else {
        powerLevel = (*(*self_).client).ps.fd.forcePowerLevel[FP_PUSH as usize];
        pushPower = 256 * (*(*self_).client).ps.fd.forcePowerLevel[FP_PUSH as usize];
    }

    if powerLevel == 0 {
        //Shouldn't have made it here..
        return;
    }

    if powerLevel == FORCE_LEVEL_2 {
        visionArc = 60;
    } else if powerLevel == FORCE_LEVEL_3 {
        visionArc = 180;
    }

    if powerLevel == FORCE_LEVEL_1 {
        //can only push/pull targeted things at level 1
        VectorCopy(&(*(*self_).client).ps.origin, &mut tfrom);
        tfrom[2] += (*(*self_).client).ps.viewheight as f32;
        AngleVectors(&(*(*self_).client).ps.viewangles, Some(&mut fwd), None, None);
        tto[0] = tfrom[0] + fwd[0] * radius as f32 / 2.0;
        tto[1] = tfrom[1] + fwd[1] * radius as f32 / 2.0;
        tto[2] = tfrom[2] + fwd[2] * radius as f32 / 2.0;

        tr = crate::trap::Trace(
            &tfrom,
            &vec3_origin,
            &vec3_origin,
            &tto,
            (*self_).s.number,
            MASK_PLAYERSOLID,
        );

        if tr.fraction != 1.0 && tr.entityNum as c_int != ENTITYNUM_NONE {
            let hit_ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);
            if (*hit_ent).client.is_null() && (*hit_ent).s.eType == ET_NPC {
                //g2animent
                if (*hit_ent).s.genericenemyindex < (*addr_of!(level)).time {
                    (*hit_ent).s.genericenemyindex = (*addr_of!(level)).time + 2000;
                }
            }

            numListedEntities = 0;
            entityList[numListedEntities as usize] = tr.entityNum as c_int;

            if pull != QFALSE {
                if ForcePowerUsableOn(self_, hit_ent, FP_PULL) == 0 {
                    return;
                }
            } else if ForcePowerUsableOn(self_, hit_ent, FP_PUSH) == 0 {
                return;
            }
            numListedEntities += 1;
        } else {
            //didn't get anything, so just
            return;
        }
    } else {
        numListedEntities = crate::trap::EntitiesInBox(&mins, &maxs, &mut entityList);

        e = 0;

        while e < numListedEntities {
            ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityList[e as usize] as usize);

            if (*ent).client.is_null() && (*ent).s.eType == ET_NPC {
                //g2animent
                if (*ent).s.genericenemyindex < (*addr_of!(level)).time {
                    (*ent).s.genericenemyindex = (*addr_of!(level)).time + 2000;
                }
            }

            if !ent.is_null() {
                if !(*ent).client.is_null() {
                    VectorCopy(&(*(*ent).client).ps.origin, &mut thispush_org);
                } else {
                    VectorCopy(&(*ent).s.pos.trBase, &mut thispush_org);
                }
            }

            if !ent.is_null() {
                //not in the arc, don't consider it
                VectorCopy(&(*(*self_).client).ps.origin, &mut tto);
                tto[2] += (*(*self_).client).ps.viewheight as f32;
                VectorSubtract(&thispush_org, &tto, &mut a);
                let a_copy = a;
                vectoangles(&a_copy, &mut a);

                if !(*ent).client.is_null()
                    && InFieldOfVision(&(*(*self_).client).ps.viewangles, visionArc as f32, &mut a)
                        == QFALSE
                    && ForcePowerUsableOn(self_, ent, powerUse) != 0
                {
                    //only bother with arc rules if the victim is a client
                    entityList[e as usize] = ENTITYNUM_NONE;
                } else if !(*ent).client.is_null() {
                    if pull != QFALSE {
                        if ForcePowerUsableOn(self_, ent, FP_PULL) == 0 {
                            entityList[e as usize] = ENTITYNUM_NONE;
                        }
                    } else if ForcePowerUsableOn(self_, ent, FP_PUSH) == 0 {
                        entityList[e as usize] = ENTITYNUM_NONE;
                    }
                }
            }
            e += 1;
        }
    }

    e = 0;
    while e < numListedEntities {
        if entityList[e as usize] != ENTITYNUM_NONE
            && entityList[e as usize] >= 0
            && entityList[e as usize] < MAX_GENTITIES as c_int
        {
            ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityList[e as usize] as usize);
        } else {
            ent = core::ptr::null_mut();
        }

        if ent.is_null() {
            e += 1;
            continue;
        }
        if ent == self_ {
            e += 1;
            continue;
        }
        if !(*ent).client.is_null() && OnSameTeam(ent, self_) != QFALSE {
            e += 1;
            continue;
        }
        if (*ent).inuse == QFALSE {
            e += 1;
            continue;
        }
        if (*ent).s.eType != ET_MISSILE {
            if (*ent).s.eType != ET_ITEM {
                //FIXME: need pushable objects
                if Q_stricmp(c"func_button".as_ptr(), (*ent).classname) == 0 {
                    //we might push it
                    if pull != QFALSE || (*ent).spawnflags & SPF_BUTTON_FPUSHABLE == 0 {
                        //not force-pushable, never pullable
                        e += 1;
                        continue;
                    }
                } else if (*ent).s.eFlags & EF_NODRAW != 0 {
                    e += 1;
                    continue;
                } else if (*ent).client.is_null() {
                    if Q_stricmp(c"lightsaber".as_ptr(), (*ent).classname) != 0 {
                        //not a lightsaber
                        if Q_stricmp(c"func_door".as_ptr(), (*ent).classname) != 0
                            || (*ent).spawnflags & 2/*MOVER_FORCE_ACTIVATE*/ == 0
                        {
                            //not a force-usable door
                            if Q_stricmp(c"func_static".as_ptr(), (*ent).classname) != 0
                                || ((*ent).spawnflags & 1/*F_PUSH*/ == 0
                                    && (*ent).spawnflags & 2/*F_PULL*/ == 0)
                            {
                                //not a force-usable func_static
                                if Q_stricmp(c"limb".as_ptr(), (*ent).classname) != 0 {
                                    //not a limb
                                    e += 1;
                                    continue;
                                }
                            } else if (*ent).moverState != MOVER_POS1
                                && (*ent).moverState != MOVER_POS2
                            {
                                //not at rest
                                e += 1;
                                continue;
                            }
                        }
                    }
                } else if (*(*ent).client).NPC_class == CLASS_GALAKMECH
                    || (*(*ent).client).NPC_class == CLASS_ATST
                    || (*(*ent).client).NPC_class == CLASS_RANCOR
                {
                    //can't push ATST or Galak or Rancor
                    e += 1;
                    continue;
                }
            }
        } else {
            if (*ent).s.pos.trType == TR_STATIONARY && (*ent).s.eFlags & EF_MISSILE_STICK != 0 {
                //can't force-push/pull stuck missiles (detpacks, tripmines)
                e += 1;
                continue;
            }
            if (*ent).s.pos.trType == TR_STATIONARY && (*ent).s.weapon != WP_THERMAL {
                //only thermal detonators can be pushed once stopped
                e += 1;
                continue;
            }
        }

        //this is all to see if we need to start a saber attack, if it's in flight, this doesn't matter
        // find the distance from the edge of the bounding box
        for i in 0..3 {
            if center[i] < (*ent).r.absmin[i] {
                v[i] = (*ent).r.absmin[i] - center[i];
            } else if center[i] > (*ent).r.absmax[i] {
                v[i] = center[i] - (*ent).r.absmax[i];
            } else {
                v[i] = 0.0;
            }
        }

        VectorSubtract(&(*ent).r.absmax, &(*ent).r.absmin, &mut size);
        VectorMA(&(*ent).r.absmin, 0.5, &size, &mut ent_org);

        VectorSubtract(&ent_org, &center, &mut dir);
        VectorNormalize(&mut dir);
        dot1 = DotProduct(&dir, &forward);
        if dot1 < 0.6 {
            e += 1;
            continue;
        }

        dist = VectorLength(&v);

        //Now check and see if we can actually deflect it
        //method1
        //if within a certain range, deflect it
        if dist >= radius as f32 {
            e += 1;
            continue;
        }

        //in PVS?
        if (*ent).r.bmodel == QFALSE
            && crate::trap::InPVS(&ent_org, &(*(*self_).client).ps.origin) == QFALSE
        {
            //must be in PVS
            e += 1;
            continue;
        }

        //really should have a clear LOS to this thing...
        tr = crate::trap::Trace(
            &(*(*self_).client).ps.origin,
            &vec3_origin,
            &vec3_origin,
            &ent_org,
            (*self_).s.number,
            MASK_SHOT,
        );
        if tr.fraction < 1.0f32 && (tr.entityNum as c_int) != (*ent).s.number {
            //must have clear LOS
            //try from eyes too before you give up
            let mut eyePoint: vec3_t = [0.0; 3];
            VectorCopy(&(*(*self_).client).ps.origin, &mut eyePoint);
            eyePoint[2] += (*(*self_).client).ps.viewheight as f32;
            tr = crate::trap::Trace(
                &eyePoint,
                &vec3_origin,
                &vec3_origin,
                &ent_org,
                (*self_).s.number,
                MASK_SHOT,
            );

            if tr.fraction < 1.0f32 && (tr.entityNum as c_int) != (*ent).s.number {
                e += 1;
                continue;
            }
        }

        // ok, we are within the radius, add us to the incoming list
        push_list[ent_count as usize] = ent;
        ent_count += 1;
        e += 1;
    }

    if ent_count != 0 {
        //method1:
        let mut x = 0;
        while x < ent_count {
            let mut modPowerLevel: c_int = powerLevel;

            if !(*push_list[x as usize]).client.is_null() {
                modPowerLevel = WP_AbsorbConversion(
                    push_list[x as usize],
                    (*(*push_list[x as usize]).client).ps.fd.forcePowerLevel[FP_ABSORB as usize],
                    self_,
                    powerUse,
                    powerLevel,
                    forcePowerNeeded[(*(*self_).client).ps.fd.forcePowerLevel[powerUse as usize]
                        as usize][powerUse as usize],
                );
                if modPowerLevel == -1 {
                    modPowerLevel = powerLevel;
                }
            }

            pushPower = 256 * modPowerLevel;

            if !(*push_list[x as usize]).client.is_null() {
                VectorCopy(
                    &(*(*push_list[x as usize]).client).ps.origin,
                    &mut thispush_org,
                );
            } else {
                VectorCopy(&(*push_list[x as usize]).s.origin, &mut thispush_org);
            }

            if !(*push_list[x as usize]).client.is_null() {
                //FIXME: make enemy jedi able to hunker down and resist this?
                let mut otherPushPower: c_int =
                    (*(*push_list[x as usize]).client).ps.fd.forcePowerLevel[powerUse as usize];
                let mut canPullWeapon: qboolean = QTRUE;
                let mut dirLen: f32 = 0.0;

                if g_debugMelee.integer != 0
                    && (*(*push_list[x as usize]).client).ps.pm_flags & PMF_STUCK_TO_WALL != 0
                {
                    //no resistance if stuck to wall
                    //push/pull them off the wall
                    otherPushPower = 0;
                    G_LetGoOfWall(push_list[x as usize]);
                }

                _knockback = if pull != QFALSE { 0.0 } else { 200.0 };

                pushPowerMod = pushPower;

                if (*(*push_list[x as usize]).client).pers.cmd.forwardmove != 0
                    || (*(*push_list[x as usize]).client).pers.cmd.rightmove != 0
                {
                    //if you are moving, you get one less level of defense
                    otherPushPower -= 1;

                    if otherPushPower < 0 {
                        otherPushPower = 0;
                    }
                }

                if otherPushPower != 0 && CanCounterThrow(push_list[x as usize], self_, pull) != 0 {
                    if pull != QFALSE {
                        G_Sound(
                            push_list[x as usize],
                            CHAN_BODY,
                            G_SoundIndex("sound/weapons/force/pull.wav"),
                        );
                        (*(*push_list[x as usize]).client).ps.forceHandExtend = HANDEXTEND_FORCEPULL;
                        (*(*push_list[x as usize]).client).ps.forceHandExtendTime =
                            (*addr_of!(level)).time + 400;
                    } else {
                        G_Sound(
                            push_list[x as usize],
                            CHAN_BODY,
                            G_SoundIndex("sound/weapons/force/push.wav"),
                        );
                        (*(*push_list[x as usize]).client).ps.forceHandExtend = HANDEXTEND_FORCEPUSH;
                        (*(*push_list[x as usize]).client).ps.forceHandExtendTime =
                            (*addr_of!(level)).time + 1000;
                    }
                    (*(*push_list[x as usize]).client).ps.powerups[PW_DISINT_4 as usize] =
                        (*(*push_list[x as usize]).client).ps.forceHandExtendTime + 200;

                    if pull != QFALSE {
                        (*(*push_list[x as usize]).client).ps.powerups[PW_PULL as usize] =
                            (*(*push_list[x as usize]).client).ps.powerups[PW_DISINT_4 as usize];
                    } else {
                        (*(*push_list[x as usize]).client).ps.powerups[PW_PULL as usize] = 0;
                    }

                    //Make a counter-throw effect

                    if otherPushPower >= modPowerLevel {
                        pushPowerMod = 0;
                        canPullWeapon = QFALSE;
                    } else {
                        let powerDif: c_int = modPowerLevel - otherPushPower;

                        if powerDif >= 3 {
                            pushPowerMod -= (pushPowerMod as f32 * 0.2) as c_int;
                        } else if powerDif == 2 {
                            pushPowerMod -= (pushPowerMod as f32 * 0.4) as c_int;
                        } else if powerDif == 1 {
                            pushPowerMod -= (pushPowerMod as f32 * 0.8) as c_int;
                        }

                        if pushPowerMod < 0 {
                            pushPowerMod = 0;
                        }
                    }
                }

                //shove them
                if pull != QFALSE {
                    VectorSubtract(
                        &(*(*self_).client).ps.origin,
                        &thispush_org,
                        &mut pushDir,
                    );

                    if !(*push_list[x as usize]).client.is_null()
                        && VectorLength(&pushDir) <= 256.0
                    {
                        let mut randfact: c_int = 0;

                        if modPowerLevel == FORCE_LEVEL_1 {
                            randfact = 3;
                        } else if modPowerLevel == FORCE_LEVEL_2 {
                            randfact = 7;
                        } else if modPowerLevel == FORCE_LEVEL_3 {
                            randfact = 10;
                        }

                        if OnSameTeam(self_, push_list[x as usize]) == QFALSE
                            && Q_irand(1, 10) <= randfact
                            && canPullWeapon != QFALSE
                        {
                            let mut uorg: vec3_t = [0.0; 3];
                            let mut vecnorm: vec3_t = [0.0; 3];

                            VectorCopy(&(*(*self_).client).ps.origin, &mut uorg);
                            uorg[2] += 64.0;

                            VectorSubtract(&uorg, &thispush_org, &mut vecnorm);
                            VectorNormalize(&mut vecnorm);

                            TossClientWeapon(push_list[x as usize], &vecnorm, 500.0);
                        }
                    }
                } else {
                    VectorSubtract(
                        &thispush_org,
                        &(*(*self_).client).ps.origin,
                        &mut pushDir,
                    );
                }

                if (modPowerLevel > otherPushPower
                    || (*(*push_list[x as usize]).client).ps.m_iVehicleNum != 0)
                    && !(*push_list[x as usize]).client.is_null()
                {
                    if modPowerLevel == FORCE_LEVEL_3
                        && (*(*push_list[x as usize]).client).ps.forceHandExtend
                            != HANDEXTEND_KNOCKDOWN
                    {
                        dirLen = VectorLength(&pushDir);

                        if BG_KnockDownable(&mut (*(*push_list[x as usize]).client).ps) != QFALSE
                            && dirLen <= (64 * ((modPowerLevel - otherPushPower) - 1)) as f32
                        {
                            //can only do a knockdown if fairly close
                            (*(*push_list[x as usize]).client).ps.forceHandExtend =
                                HANDEXTEND_KNOCKDOWN;
                            (*(*push_list[x as usize]).client).ps.forceHandExtendTime =
                                (*addr_of!(level)).time + 700;
                            (*(*push_list[x as usize]).client).ps.forceDodgeAnim = 0; //this toggles between 1 and 0, when it's 1 we should play the get up anim
                            (*(*push_list[x as usize]).client).ps.quickerGetup = QTRUE;
                        } else if (*push_list[x as usize]).s.number < MAX_CLIENTS as c_int
                            && (*(*push_list[x as usize]).client).ps.m_iVehicleNum != 0
                            && dirLen <= 128.0f32
                        {
                            //a player on a vehicle
                            let vehEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                .add((*(*push_list[x as usize]).client).ps.m_iVehicleNum as usize);
                            if (*vehEnt).inuse != QFALSE
                                && !(*vehEnt).client.is_null()
                                && !(*vehEnt).m_pVehicle.is_null()
                            {
                                if (*(*(*vehEnt).m_pVehicle).m_pVehicleInfo).r#type == VH_SPEEDER
                                    || (*(*(*vehEnt).m_pVehicle).m_pVehicleInfo).r#type == VH_ANIMAL
                                {
                                    //push the guy off
                                    ((*(*(*vehEnt).m_pVehicle).m_pVehicleInfo).Eject.unwrap())(
                                        (*vehEnt).m_pVehicle,
                                        push_list[x as usize] as *mut bgEntity_t,
                                        QFALSE,
                                    );
                                }
                            }
                        }
                    }
                }

                if dirLen == 0.0 {
                    dirLen = VectorLength(&pushDir);
                }

                VectorNormalize(&mut pushDir);

                if !(*push_list[x as usize]).client.is_null() {
                    //escape a force grip if we're in one
                    if (*(*self_).client).ps.fd.forceGripBeingGripped > (*addr_of!(level)).time as f32
                    {
                        //force the enemy to stop gripping me if I managed to push him
                        if (*(*push_list[x as usize]).client).ps.fd.forceGripEntityNum
                            == (*self_).s.number
                        {
                            if modPowerLevel
                                >= (*(*push_list[x as usize]).client).ps.fd.forcePowerLevel
                                    [FP_GRIP as usize]
                            {
                                //only break the grip if our push/pull level is >= their grip level
                                WP_ForcePowerStop(push_list[x as usize], FP_GRIP);
                                (*(*self_).client).ps.fd.forceGripBeingGripped = 0.0;
                                (*(*push_list[x as usize]).client).ps.fd.forceGripUseTime =
                                    (*addr_of!(level)).time + 1000; //since we just broke out of it..
                            }
                        }
                    }

                    (*(*push_list[x as usize]).client).ps.otherKiller = (*self_).s.number;
                    (*(*push_list[x as usize]).client).ps.otherKillerTime =
                        (*addr_of!(level)).time + 5000;
                    (*(*push_list[x as usize]).client).ps.otherKillerDebounceTime =
                        (*addr_of!(level)).time + 100;
                    (*(*push_list[x as usize]).client).otherKillerMOD = MOD_UNKNOWN;
                    (*(*push_list[x as usize]).client).otherKillerVehWeapon = 0;
                    (*(*push_list[x as usize]).client).otherKillerWeaponType = WP_NONE;

                    pushPowerMod -= (dirLen * 0.7) as c_int;
                    if pushPowerMod < 16 {
                        pushPowerMod = 16;
                    }

                    //fullbody push effect
                    (*(*push_list[x as usize]).client).pushEffectTime =
                        (*addr_of!(level)).time + 600;

                    (*(*push_list[x as usize]).client).ps.velocity[0] =
                        pushDir[0] * pushPowerMod as f32;
                    (*(*push_list[x as usize]).client).ps.velocity[1] =
                        pushDir[1] * pushPowerMod as f32;

                    if (*(*push_list[x as usize]).client).ps.velocity[2] as c_int == 0 {
                        //if not going anywhere vertically, boost them up a bit
                        (*(*push_list[x as usize]).client).ps.velocity[2] =
                            pushDir[2] * pushPowerMod as f32;

                        if (*(*push_list[x as usize]).client).ps.velocity[2] < 128.0 {
                            (*(*push_list[x as usize]).client).ps.velocity[2] = 128.0;
                        }
                    } else {
                        (*(*push_list[x as usize]).client).ps.velocity[2] =
                            pushDir[2] * pushPowerMod as f32;
                    }
                }
            } else if (*push_list[x as usize]).s.eType == ET_MISSILE
                && (*push_list[x as usize]).s.pos.trType != TR_STATIONARY
                && ((*push_list[x as usize]).s.pos.trType != TR_INTERPOLATE
                    || (*push_list[x as usize]).s.weapon != WP_THERMAL)
            {
                //rolling and stationary thermal detonators are dealt with below
                if pull != QFALSE {
                    //deflect rather than reflect?
                } else {
                    G_ReflectMissile(self_, push_list[x as usize], &forward);
                }
            } else if Q_stricmp(c"func_static".as_ptr(), (*push_list[x as usize]).classname) == 0 {
                //force-usable func_static
                if pull == QFALSE && (*push_list[x as usize]).spawnflags & 1/*F_PUSH*/ != 0 {
                    GEntity_UseFunc(push_list[x as usize], self_, self_);
                } else if pull != QFALSE && (*push_list[x as usize]).spawnflags & 2/*F_PULL*/ != 0 {
                    GEntity_UseFunc(push_list[x as usize], self_, self_);
                }
            } else if Q_stricmp(c"func_door".as_ptr(), (*push_list[x as usize]).classname) == 0
                && (*push_list[x as usize]).spawnflags & 2 != 0
            {
                //push/pull the door
                let mut pos1: vec3_t = [0.0; 3];
                let mut pos2: vec3_t = [0.0; 3];
                let mut trFrom: vec3_t = [0.0; 3];

                VectorCopy(&(*(*self_).client).ps.origin, &mut trFrom);
                trFrom[2] += (*(*self_).client).ps.viewheight as f32;

                AngleVectors(&(*(*self_).client).ps.viewangles, Some(&mut forward), None, None);
                VectorNormalize(&mut forward);
                VectorMA(&trFrom, radius as f32, &forward, &mut end);
                tr = crate::trap::Trace(
                    &trFrom,
                    &vec3_origin,
                    &vec3_origin,
                    &end,
                    (*self_).s.number,
                    MASK_SHOT,
                );
                if tr.entityNum as c_int != (*push_list[x as usize]).s.number
                    || tr.fraction == 1.0
                    || tr.allsolid != 0
                    || tr.startsolid != 0
                {
                    //must be pointing right at it
                    x += 1;
                    continue;
                }

                if VectorCompare(&vec3_origin, &(*push_list[x as usize]).s.origin) != 0 {
                    //does not have an origin brush, so pos1 & pos2 are relative to world origin, need to calc center
                    VectorSubtract(
                        &(*push_list[x as usize]).r.absmax,
                        &(*push_list[x as usize]).r.absmin,
                        &mut size,
                    );
                    VectorMA(
                        &(*push_list[x as usize]).r.absmin,
                        0.5,
                        &size,
                        &mut center,
                    );
                    if (*push_list[x as usize]).spawnflags & 1 != 0
                        && (*push_list[x as usize]).moverState == MOVER_POS1
                    {
                        //if at pos1 and started open, make sure we get the center where it *started* because we're going to add back in the relative values pos1 and pos2
                        let center_copy = center;
                        VectorSubtract(&center_copy, &(*push_list[x as usize]).pos1, &mut center);
                    } else if (*push_list[x as usize]).spawnflags & 1 == 0
                        && (*push_list[x as usize]).moverState == MOVER_POS2
                    {
                        //if at pos2, make sure we get the center where it *started* because we're going to add back in the relative values pos1 and pos2
                        let center_copy = center;
                        VectorSubtract(&center_copy, &(*push_list[x as usize]).pos2, &mut center);
                    }
                    VectorAdd(&center, &(*push_list[x as usize]).pos1, &mut pos1);
                    VectorAdd(&center, &(*push_list[x as usize]).pos2, &mut pos2);
                } else {
                    //actually has an origin, pos1 and pos2 are absolute
                    VectorCopy(&(*push_list[x as usize]).r.currentOrigin, &mut center);
                    VectorCopy(&(*push_list[x as usize]).pos1, &mut pos1);
                    VectorCopy(&(*push_list[x as usize]).pos2, &mut pos2);
                }

                if Distance(&pos1, &trFrom) < Distance(&pos2, &trFrom) {
                    //pos1 is closer
                    if (*push_list[x as usize]).moverState == MOVER_POS1 {
                        //at the closest pos
                        if pull != QFALSE {
                            //trying to pull, but already at closest point, so screw it
                            x += 1;
                            continue;
                        }
                    } else if (*push_list[x as usize]).moverState == MOVER_POS2 {
                        //at farthest pos
                        if pull == QFALSE {
                            //trying to push, but already at farthest point, so screw it
                            x += 1;
                            continue;
                        }
                    }
                } else {
                    //pos2 is closer
                    if (*push_list[x as usize]).moverState == MOVER_POS1 {
                        //at the farthest pos
                        if pull == QFALSE {
                            //trying to push, but already at farthest point, so screw it
                            x += 1;
                            continue;
                        }
                    } else if (*push_list[x as usize]).moverState == MOVER_POS2 {
                        //at closest pos
                        if pull != QFALSE {
                            //trying to pull, but already at closest point, so screw it
                            x += 1;
                            continue;
                        }
                    }
                }
                GEntity_UseFunc(push_list[x as usize], self_, self_);
            } else if Q_stricmp(c"func_button".as_ptr(), (*push_list[x as usize]).classname) == 0 {
                //pretend you pushed it
                Touch_Button(push_list[x as usize], self_, core::ptr::null_mut());
                x += 1;
                continue;
            }
            x += 1;
        }
    }

    //attempt to break any leftover grips
    //if we're still in a current grip that wasn't broken by the push, it will still remain
    (*(*self_).client).dangerTime = (*addr_of!(level)).time;
    (*(*self_).client).ps.eFlags &= !EF_INVULNERABLE;
    (*(*self_).client).invulnerableTimer = 0;

    if (*(*self_).client).ps.fd.forceGripBeingGripped > (*addr_of!(level)).time as f32 {
        (*(*self_).client).ps.fd.forceGripBeingGripped = 0.0;
    }
}

/// `void DoGripAction(gentity_t *self, forcePowers_t forcePower)` (w_force.c:3952) — one tick of
/// FP_GRIP on the entity `self` is currently gripping. Validates the grip target (existence, LOS,
/// distance, facing, absorb-converted grip level) and stops the power if any check fails; else
/// applies level-scaled effects: per-second choke damage, jetpack-off, float-movetype, lift
/// velocity, and a delayed big crack with a choke anim at levels 2/3.
///
/// No oracle — mutates the live gentity/playerState graph and calls `G_Damage`/`trap_Trace`/
/// `WP_ForcePowerStop`/`Jetpack_Off` (entity/trap control-flow precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`; `g_entities`/`level` must be
/// initialised (the grip target is indexed out of `g_entities`).
#[allow(unused_assignments)] // faithful: C dead-initialises `gripLevel = 0` before the absorb-conversion overwrites it
pub unsafe fn DoGripAction(self_: *mut gentity_t, force_power: c_int) {
    let gripEnt: *mut gentity_t;
    let mut gripLevel: c_int = 0;
    let tr: trace_t;
    let mut a: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut fwd_o: vec3_t = [0.0; 3];
    let mut start_o: vec3_t = [0.0; 3];
    let mut nvel: vec3_t = [0.0; 3];

    (*(*self_).client).dangerTime = (*addr_of!(level)).time;
    (*(*self_).client).ps.eFlags &= !EF_INVULNERABLE;
    (*(*self_).client).invulnerableTimer = 0;

    gripEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
        .add((*(*self_).client).ps.fd.forceGripEntityNum as usize);

    if gripEnt.is_null()
        || (*gripEnt).client.is_null()
        || (*gripEnt).inuse == QFALSE
        || (*gripEnt).health < 1
        || ForcePowerUsableOn(self_, gripEnt, FP_GRIP) == 0
    {
        WP_ForcePowerStop(self_, force_power);
        (*(*self_).client).ps.fd.forceGripEntityNum = ENTITYNUM_NONE;

        if !gripEnt.is_null() && !(*gripEnt).client.is_null() && (*gripEnt).inuse != QFALSE {
            (*(*gripEnt).client).ps.forceGripChangeMovetype = PM_NORMAL;
        }
        return;
    }

    VectorSubtract(
        &(*(*gripEnt).client).ps.origin,
        &(*(*self_).client).ps.origin,
        &mut a,
    );

    tr = crate::trap::Trace(
        &(*(*self_).client).ps.origin,
        &vec3_origin,
        &vec3_origin,
        &(*(*gripEnt).client).ps.origin,
        (*self_).s.number,
        MASK_PLAYERSOLID,
    );

    gripLevel = WP_AbsorbConversion(
        gripEnt,
        (*(*gripEnt).client).ps.fd.forcePowerLevel[FP_ABSORB as usize],
        self_,
        FP_GRIP,
        (*(*self_).client).ps.fd.forcePowerLevel[FP_GRIP as usize],
        forcePowerNeeded[(*(*self_).client).ps.fd.forcePowerLevel[FP_GRIP as usize] as usize]
            [FP_GRIP as usize],
    );

    if gripLevel == -1 {
        gripLevel = (*(*self_).client).ps.fd.forcePowerLevel[FP_GRIP as usize];
    }

    if gripLevel == 0 {
        WP_ForcePowerStop(self_, force_power);
        return;
    }

    if VectorLength(&a) > MAX_GRIP_DISTANCE as f32 {
        WP_ForcePowerStop(self_, force_power);
        return;
    }

    if InFront(
        &(*(*gripEnt).client).ps.origin,
        &(*(*self_).client).ps.origin,
        &(*(*self_).client).ps.viewangles,
        0.9f32,
    ) == QFALSE
        && gripLevel < FORCE_LEVEL_3
    {
        WP_ForcePowerStop(self_, force_power);
        return;
    }

    if tr.fraction != 1.0f32 && (tr.entityNum as c_int) != (*gripEnt).s.number
    /*&&
    gripLevel < FORCE_LEVEL_3*/
    {
        WP_ForcePowerStop(self_, force_power);
        return;
    }

    if (*(*self_).client).ps.fd.forcePowerDebounce[FP_GRIP as usize] < (*addr_of!(level)).time {
        //2 damage per second while choking, resulting in 10 damage total (not including The Squeeze<tm>)
        (*(*self_).client).ps.fd.forcePowerDebounce[FP_GRIP as usize] =
            (*addr_of!(level)).time + 1000;
        G_Damage(
            gripEnt,
            self_,
            self_,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            2,
            DAMAGE_NO_ARMOR,
            MOD_FORCE_DARK,
        );
    }

    Jetpack_Off(gripEnt); //make sure the guy being gripped has his jetpack off.

    if gripLevel == FORCE_LEVEL_1 {
        (*(*gripEnt).client).ps.fd.forceGripBeingGripped = ((*addr_of!(level)).time + 1000) as f32;

        if ((*addr_of!(level)).time as f32 - (*(*gripEnt).client).ps.fd.forceGripStarted) > 5000.0 {
            WP_ForcePowerStop(self_, force_power);
        }
        return;
    }

    if gripLevel == FORCE_LEVEL_2 {
        (*(*gripEnt).client).ps.fd.forceGripBeingGripped = ((*addr_of!(level)).time + 1000) as f32;

        if (*(*gripEnt).client).ps.forceGripMoveInterval < (*addr_of!(level)).time {
            (*(*gripEnt).client).ps.velocity[2] = 30.0;

            (*(*gripEnt).client).ps.forceGripMoveInterval = (*addr_of!(level)).time + 300; //only update velocity every 300ms, so as to avoid heavy bandwidth usage
        }

        (*(*gripEnt).client).ps.otherKiller = (*self_).s.number;
        (*(*gripEnt).client).ps.otherKillerTime = (*addr_of!(level)).time + 5000;
        (*(*gripEnt).client).ps.otherKillerDebounceTime = (*addr_of!(level)).time + 100;
        (*(*gripEnt).client).otherKillerMOD = MOD_UNKNOWN;
        (*(*gripEnt).client).otherKillerVehWeapon = 0;
        (*(*gripEnt).client).otherKillerWeaponType = WP_NONE;

        (*(*gripEnt).client).ps.forceGripChangeMovetype = PM_FLOAT;

        if ((*addr_of!(level)).time as f32 - (*(*gripEnt).client).ps.fd.forceGripStarted) > 3000.0
            && (*(*self_).client).ps.fd.forceGripDamageDebounceTime == 0
        {
            //if we managed to lift him into the air for 2 seconds, give him a crack
            (*(*self_).client).ps.fd.forceGripDamageDebounceTime = 1;
            G_Damage(
                gripEnt,
                self_,
                self_,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                20,
                DAMAGE_NO_ARMOR,
                MOD_FORCE_DARK,
            );

            //Must play custom sounds on the actual entity. Don't use G_Sound (it creates a temp entity for the sound)
            G_EntitySound(
                gripEnt,
                CHAN_VOICE,
                G_SoundIndex(&format!("*choke{}.wav", Q_irand(1, 3))),
            );

            (*(*gripEnt).client).ps.forceHandExtend = HANDEXTEND_CHOKE;
            (*(*gripEnt).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 2000;

            if (*(*gripEnt).client).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0 {
                //choking, so don't let him keep gripping himself
                WP_ForcePowerStop(gripEnt, FP_GRIP);
            }
        } else if ((*addr_of!(level)).time as f32 - (*(*gripEnt).client).ps.fd.forceGripStarted) > 4000.0 {
            WP_ForcePowerStop(self_, force_power);
        }
        return;
    }

    if gripLevel == FORCE_LEVEL_3 {
        (*(*gripEnt).client).ps.fd.forceGripBeingGripped = ((*addr_of!(level)).time + 1000) as f32;

        (*(*gripEnt).client).ps.otherKiller = (*self_).s.number;
        (*(*gripEnt).client).ps.otherKillerTime = (*addr_of!(level)).time + 5000;
        (*(*gripEnt).client).ps.otherKillerDebounceTime = (*addr_of!(level)).time + 100;
        (*(*gripEnt).client).otherKillerMOD = MOD_UNKNOWN;
        (*(*gripEnt).client).otherKillerVehWeapon = 0;
        (*(*gripEnt).client).otherKillerWeaponType = WP_NONE;

        (*(*gripEnt).client).ps.forceGripChangeMovetype = PM_FLOAT;

        if (*(*gripEnt).client).ps.forceGripMoveInterval < (*addr_of!(level)).time {
            let nvLen: f32;

            VectorCopy(&(*(*gripEnt).client).ps.origin, &mut start_o);
            AngleVectors(&(*(*self_).client).ps.viewangles, Some(&mut fwd), None, None);
            fwd_o[0] = (*(*self_).client).ps.origin[0] + fwd[0] * 128.0;
            fwd_o[1] = (*(*self_).client).ps.origin[1] + fwd[1] * 128.0;
            fwd_o[2] = (*(*self_).client).ps.origin[2] + fwd[2] * 128.0;
            fwd_o[2] += 16.0;
            VectorSubtract(&fwd_o, &start_o, &mut nvel);

            nvLen = VectorLength(&nvel);

            if nvLen < 16.0 {
                //within x units of desired spot
                VectorNormalize(&mut nvel);
                (*(*gripEnt).client).ps.velocity[0] = nvel[0] * 8.0;
                (*(*gripEnt).client).ps.velocity[1] = nvel[1] * 8.0;
                (*(*gripEnt).client).ps.velocity[2] = nvel[2] * 8.0;
            } else if nvLen < 64.0 {
                VectorNormalize(&mut nvel);
                (*(*gripEnt).client).ps.velocity[0] = nvel[0] * 128.0;
                (*(*gripEnt).client).ps.velocity[1] = nvel[1] * 128.0;
                (*(*gripEnt).client).ps.velocity[2] = nvel[2] * 128.0;
            } else if nvLen < 128.0 {
                VectorNormalize(&mut nvel);
                (*(*gripEnt).client).ps.velocity[0] = nvel[0] * 256.0;
                (*(*gripEnt).client).ps.velocity[1] = nvel[1] * 256.0;
                (*(*gripEnt).client).ps.velocity[2] = nvel[2] * 256.0;
            } else if nvLen < 200.0 {
                VectorNormalize(&mut nvel);
                (*(*gripEnt).client).ps.velocity[0] = nvel[0] * 512.0;
                (*(*gripEnt).client).ps.velocity[1] = nvel[1] * 512.0;
                (*(*gripEnt).client).ps.velocity[2] = nvel[2] * 512.0;
            } else {
                VectorNormalize(&mut nvel);
                (*(*gripEnt).client).ps.velocity[0] = nvel[0] * 700.0;
                (*(*gripEnt).client).ps.velocity[1] = nvel[1] * 700.0;
                (*(*gripEnt).client).ps.velocity[2] = nvel[2] * 700.0;
            }

            (*(*gripEnt).client).ps.forceGripMoveInterval = (*addr_of!(level)).time + 300; //only update velocity every 300ms, so as to avoid heavy bandwidth usage
        }

        if ((*addr_of!(level)).time as f32 - (*(*gripEnt).client).ps.fd.forceGripStarted) > 3000.0
            && (*(*self_).client).ps.fd.forceGripDamageDebounceTime == 0
        {
            //if we managed to lift him into the air for 2 seconds, give him a crack
            (*(*self_).client).ps.fd.forceGripDamageDebounceTime = 1;
            G_Damage(
                gripEnt,
                self_,
                self_,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                40,
                DAMAGE_NO_ARMOR,
                MOD_FORCE_DARK,
            );

            //Must play custom sounds on the actual entity. Don't use G_Sound (it creates a temp entity for the sound)
            G_EntitySound(
                gripEnt,
                CHAN_VOICE,
                G_SoundIndex(&format!("*choke{}.wav", Q_irand(1, 3))),
            );

            (*(*gripEnt).client).ps.forceHandExtend = HANDEXTEND_CHOKE;
            (*(*gripEnt).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 2000;

            if (*(*gripEnt).client).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0 {
                //choking, so don't let him keep gripping himself
                WP_ForcePowerStop(gripEnt, FP_GRIP);
            }
        } else if ((*addr_of!(level)).time as f32 - (*(*gripEnt).client).ps.fd.forceGripStarted) > 4000.0 {
            WP_ForcePowerStop(self_, force_power);
        }
        return;
    }
}

/// `static void WP_UpdateMindtrickEnts(gentity_t *self)` (w_force.c:4234) — prune `self`'s
/// FP_TELEPATHY (mind-trick) victim list. Walks every client slot: drops a victim that died/left/
/// has force-sight active, drops one the tricker just fired at while it had LOS (within
/// `g_TimeSinceLastFrame*4` of the danger time), or drops one inside a ysalamiri field. If the
/// list ends up empty (or the caster is carrying a flag) it stops the power.
///
/// No oracle — mutates the live gentity/playerState graph and calls `trap_InPVS`/`OrgVisible`/
/// `WP_ForcePowerStop` (entity/trap control-flow precedent).
///
/// DEVIATION: ported `pub` though C is `static` (the `WP_DisruptorMainFire`/`G_SayTo` precedent)
/// to avoid a dead_code warning until its caller `WP_ForcePowersUpdate` lands.
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`; `g_entities`/`level` must be
/// initialised.
pub unsafe fn WP_UpdateMindtrickEnts(self_: *mut gentity_t) {
    let mut i: c_int = 0;

    while i < MAX_CLIENTS as c_int {
        if G_IsMindTricked(&(*(*self_).client).ps.fd, i) != QFALSE {
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

            if ent.is_null()
                || (*ent).client.is_null()
                || (*ent).inuse == QFALSE
                || (*ent).health < 1
                || ((*(*ent).client).ps.fd.forcePowersActive & (1 << FP_SEE)) != 0
            {
                RemoveTrickedEnt(&mut (*(*self_).client).ps.fd, i);
            } else if ((*addr_of!(level)).time - (*(*self_).client).dangerTime)
                < g_TimeSinceLastFrame * 4
            {
                //Untrick this entity if the tricker (self) fires while in his fov
                if crate::trap::InPVS(
                    &(*(*ent).client).ps.origin,
                    &(*(*self_).client).ps.origin,
                ) != QFALSE
                    && OrgVisible(
                        &(*(*ent).client).ps.origin,
                        &(*(*self_).client).ps.origin,
                        (*ent).s.number,
                    ) != 0
                {
                    RemoveTrickedEnt(&mut (*(*self_).client).ps.fd, i);
                }
            } else if BG_HasYsalamiri((*addr_of!(g_gametype)).integer, &mut (*(*ent).client).ps)
                != QFALSE
            {
                RemoveTrickedEnt(&mut (*(*self_).client).ps.fd, i);
            }
        }

        i += 1;
    }

    if (*(*self_).client).ps.fd.forceMindtrickTargetIndex == 0
        && (*(*self_).client).ps.fd.forceMindtrickTargetIndex2 == 0
        && (*(*self_).client).ps.fd.forceMindtrickTargetIndex3 == 0
        && (*(*self_).client).ps.fd.forceMindtrickTargetIndex4 == 0
    {
        //everyone who we had tricked is no longer tricked, so stop the power
        WP_ForcePowerStop(self_, FP_TELEPATHY);
    } else if (*(*self_).client).ps.powerups[PW_REDFLAG as usize] != 0
        || (*(*self_).client).ps.powerups[PW_BLUEFLAG as usize] != 0
    {
        WP_ForcePowerStop(self_, FP_TELEPATHY);
    }
}

/// `void SeekerDroneUpdate(gentity_t *self)` (w_force.c:4709) — per-frame think for the
/// seeker-drone holdable orbiting a player. Spark-explodes and clears the drone when the owner
/// dies or the lifetime expires; otherwise it validates/re-acquires a target
/// (`FindGenericEnemyIndex`), computes the drone's time-based orbit offset and, when its fire
/// timer is up and it has clear LOS, fires a generic blaster missile at the target.
///
/// No oracle — drives `G_PlayEffect`/`trap_Trace`/`WP_FireGenericBlasterMissile`/sound traps and
/// mutates the live playerState (entity/trap control-flow precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`; `g_entities`/`level` must be
/// initialised.
#[allow(unused_assignments)] // faithful: C dead-initialises `prefig = 0` before the lifetime-expiry branch overwrites it
pub unsafe fn SeekerDroneUpdate(self_: *mut gentity_t) {
    let mut org: vec3_t = [0.0; 3];
    let mut elevated: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut a: vec3_t = [0.0; 3];
    let mut endir: vec3_t = [0.0; 3];
    let en: *mut gentity_t;
    let angle: f32;
    let mut prefig: f32 = 0.0;
    let tr: trace_t;

    if (*(*self_).client).ps.eFlags & EF_SEEKERDRONE == 0 {
        (*(*self_).client).ps.genericEnemyIndex = -1;
        return;
    }

    if (*self_).health < 1 {
        VectorCopy(&(*(*self_).client).ps.origin, &mut elevated);
        elevated[2] += 40.0;

        let angle = (((*addr_of!(level)).time / 12) & 255) as f32 * (M_PI * 2.0) / 255.0; //magical numbers make magic happen
        dir[0] = (angle as f64).cos() as f32 * 20.0;
        dir[1] = (angle as f64).sin() as f32 * 20.0;
        dir[2] = (angle as f64).cos() as f32 * 5.0;
        VectorAdd(&elevated, &dir, &mut org);

        a[ROLL] = 0.0;
        a[YAW] = 0.0;
        a[PITCH] = 1.0;

        G_PlayEffect(EFFECT_SPARK_EXPLOSION, &org, &a);

        (*(*self_).client).ps.eFlags -= EF_SEEKERDRONE;
        (*(*self_).client).ps.genericEnemyIndex = -1;

        return;
    }

    if (*(*self_).client).ps.droneExistTime >= (*addr_of!(level)).time as f32
        && (*(*self_).client).ps.droneExistTime < ((*addr_of!(level)).time + 5000) as f32
    {
        (*(*self_).client).ps.genericEnemyIndex =
            (1024.0 + (*(*self_).client).ps.droneExistTime) as c_int;
        if (*(*self_).client).ps.droneFireTime < (*addr_of!(level)).time as f32 {
            G_Sound(
                self_,
                CHAN_BODY,
                G_SoundIndex("sound/weapons/laser_trap/warning.wav"),
            );
            (*(*self_).client).ps.droneFireTime = ((*addr_of!(level)).time + 100) as f32;
        }
        return;
    } else if (*(*self_).client).ps.droneExistTime < (*addr_of!(level)).time as f32 {
        VectorCopy(&(*(*self_).client).ps.origin, &mut elevated);
        elevated[2] += 40.0;

        prefig = ((*(*self_).client).ps.droneExistTime - (*addr_of!(level)).time as f32) / 80.0;

        if prefig > 55.0 {
            prefig = 55.0;
        } else if prefig < 1.0 {
            prefig = 1.0;
        }

        elevated[2] -= 55.0 - prefig;

        let angle = (((*addr_of!(level)).time / 12) & 255) as f32 * (M_PI * 2.0) / 255.0; //magical numbers make magic happen
        dir[0] = (angle as f64).cos() as f32 * 20.0;
        dir[1] = (angle as f64).sin() as f32 * 20.0;
        dir[2] = (angle as f64).cos() as f32 * 5.0;
        VectorAdd(&elevated, &dir, &mut org);

        a[ROLL] = 0.0;
        a[YAW] = 0.0;
        a[PITCH] = 1.0;

        G_PlayEffect(EFFECT_SPARK_EXPLOSION, &org, &a);

        (*(*self_).client).ps.eFlags -= EF_SEEKERDRONE;
        (*(*self_).client).ps.genericEnemyIndex = -1;

        return;
    }

    if (*(*self_).client).ps.genericEnemyIndex == -1 {
        (*(*self_).client).ps.genericEnemyIndex = ENTITYNUM_NONE;
    }

    if (*(*self_).client).ps.genericEnemyIndex != ENTITYNUM_NONE
        && (*(*self_).client).ps.genericEnemyIndex != -1
    {
        let en = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*self_).client).ps.genericEnemyIndex as usize);

        if en.is_null() || (*en).client.is_null() {
            (*(*self_).client).ps.genericEnemyIndex = ENTITYNUM_NONE;
        } else if (*en).s.number == (*self_).s.number {
            (*(*self_).client).ps.genericEnemyIndex = ENTITYNUM_NONE;
        } else if (*en).health < 1 {
            (*(*self_).client).ps.genericEnemyIndex = ENTITYNUM_NONE;
        } else if OnSameTeam(self_, en) != QFALSE {
            (*(*self_).client).ps.genericEnemyIndex = ENTITYNUM_NONE;
        } else if InFront(
            &(*(*en).client).ps.origin,
            &(*(*self_).client).ps.origin,
            &(*(*self_).client).ps.viewangles,
            0.8f32,
        ) == QFALSE
        {
            (*(*self_).client).ps.genericEnemyIndex = ENTITYNUM_NONE;
        } else if OrgVisible(
            &(*(*self_).client).ps.origin,
            &(*(*en).client).ps.origin,
            (*self_).s.number,
        ) == 0
        {
            (*(*self_).client).ps.genericEnemyIndex = ENTITYNUM_NONE;
        }
    }

    if (*(*self_).client).ps.genericEnemyIndex == ENTITYNUM_NONE
        || (*(*self_).client).ps.genericEnemyIndex == -1
    {
        FindGenericEnemyIndex(self_);
    }

    if (*(*self_).client).ps.genericEnemyIndex != ENTITYNUM_NONE
        && (*(*self_).client).ps.genericEnemyIndex != -1
    {
        en = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*self_).client).ps.genericEnemyIndex as usize);

        VectorCopy(&(*(*self_).client).ps.origin, &mut elevated);
        elevated[2] += 40.0;

        angle = (((*addr_of!(level)).time / 12) & 255) as f32 * (M_PI * 2.0) / 255.0; //magical numbers make magic happen
        dir[0] = (angle as f64).cos() as f32 * 20.0;
        dir[1] = (angle as f64).sin() as f32 * 20.0;
        dir[2] = (angle as f64).cos() as f32 * 5.0;
        VectorAdd(&elevated, &dir, &mut org);

        //org is now where the thing should be client-side because it uses the same time-based offset
        if (*(*self_).client).ps.droneFireTime < (*addr_of!(level)).time as f32 {
            tr = crate::trap::Trace(
                &org,
                &vec3_origin,
                &vec3_origin,
                &(*(*en).client).ps.origin,
                -1,
                MASK_SOLID,
            );

            if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
                VectorSubtract(&(*(*en).client).ps.origin, &org, &mut endir);
                VectorNormalize(&mut endir);

                WP_FireGenericBlasterMissile(self_, &mut org, &endir, QFALSE, 15, 2000, MOD_BLASTER);
                G_SoundAtLoc(
                    &org,
                    CHAN_WEAPON,
                    G_SoundIndex("sound/weapons/bryar/fire.wav"),
                );

                (*(*self_).client).ps.droneFireTime =
                    ((*addr_of!(level)).time + Q_irand(400, 700)) as f32;
            }
        }
    }
}

/// `void ForceGrip( gentity_t *self )` (w_force.c:1530) — the FP_GRIP activator. Bails when
/// dead, mid hand-extend, mid weapon-time, still in the grip-use debounce, or the power isn't
/// usable. Traces forward `MAX_GRIP_DISTANCE`; if it hits a non-crippled, not-already-gripped,
/// grip-usable, non-friendly client, latches the victim into `forceGripEntityNum`, ejects a
/// player riding a speeder/animal vehicle, and arms a 5s `HANDEXTEND_FORCE_HOLD`. Else clears
/// `forceGripEntityNum` to `ENTITYNUM_NONE`.
///
/// No oracle — `trap_Trace` + live-entity-array mutation + sound/vehicle side effects (the
/// w_force activator-leaf precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`.
unsafe fn ForceGrip(self_: *mut gentity_t) {
    if (*self_).health <= 0 {
        return;
    }

    if (*(*self_).client).ps.forceHandExtend != HANDEXTEND_NONE {
        return;
    }

    if (*(*self_).client).ps.weaponTime > 0 {
        return;
    }

    if (*(*self_).client).ps.fd.forceGripUseTime > (*addr_of!(level)).time {
        return;
    }

    if WP_ForcePowerUsable(self_, FP_GRIP) == QFALSE {
        return;
    }

    let mut tfrom: vec3_t = [0.0; 3];
    let mut tto: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];

    VectorCopy(&(*(*self_).client).ps.origin, &mut tfrom);
    tfrom[2] += (*(*self_).client).ps.viewheight as f32;
    AngleVectors(&(*(*self_).client).ps.viewangles, Some(&mut fwd), None, None);
    tto[0] = tfrom[0] + fwd[0] * MAX_GRIP_DISTANCE as f32;
    tto[1] = tfrom[1] + fwd[1] * MAX_GRIP_DISTANCE as f32;
    tto[2] = tfrom[2] + fwd[2] * MAX_GRIP_DISTANCE as f32;

    let tr = crate::trap::Trace(
        &tfrom,
        &vec3_origin,
        &vec3_origin,
        &tto,
        (*self_).s.number,
        MASK_PLAYERSOLID,
    );

    let traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

    if tr.fraction != 1.0
        && (tr.entityNum as c_int) != ENTITYNUM_NONE
        && !(*traceEnt).client.is_null()
        && (*(*traceEnt).client).ps.fd.forceGripCripple == 0
        && (*(*traceEnt).client).ps.fd.forceGripBeingGripped < (*addr_of!(level)).time as f32
        && ForcePowerUsableOn(self_, traceEnt, FP_GRIP) != QFALSE
        && (g_friendlyFire.integer != 0 || OnSameTeam(self_, traceEnt) == QFALSE)
    {
        //don't grip someone who's still crippled
        if (*traceEnt).s.number < MAX_CLIENTS as c_int
            && (*(*traceEnt).client).ps.m_iVehicleNum != 0
        {
            //a player on a vehicle
            let vehEnt =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*traceEnt).client).ps.m_iVehicleNum as usize);
            if (*vehEnt).inuse != QFALSE
                && !(*vehEnt).client.is_null()
                && !(*vehEnt).m_pVehicle.is_null()
            {
                if (*(*(*vehEnt).m_pVehicle).m_pVehicleInfo).r#type == VH_SPEEDER
                    || (*(*(*vehEnt).m_pVehicle).m_pVehicleInfo).r#type == VH_ANIMAL
                {
                    //push the guy off
                    ((*(*(*vehEnt).m_pVehicle).m_pVehicleInfo).Eject.unwrap())(
                        (*vehEnt).m_pVehicle,
                        traceEnt as *mut bgEntity_t,
                        QFALSE,
                    );
                }
            }
        }
        (*(*self_).client).ps.fd.forceGripEntityNum = tr.entityNum as c_int;
        (*(*traceEnt).client).ps.fd.forceGripStarted = (*addr_of!(level)).time as f32;
        (*(*self_).client).ps.fd.forceGripDamageDebounceTime = 0;

        (*(*self_).client).ps.forceHandExtend = HANDEXTEND_FORCE_HOLD;
        (*(*self_).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 5000;
    } else {
        (*(*self_).client).ps.fd.forceGripEntityNum = ENTITYNUM_NONE;
    }
}

/// `qboolean ForceTelepathyCheckDirectNPCTarget( gentity_t *self, trace_t *tr, qboolean
/// *tookPower )` (w_force.c:2534) — the direct-on-NPC path of FP_TELEPATHY. Traces forward
/// half `MAX_TRICK_DISTANCE`; if it hits an organic, non-droid, non-player NPC, either
/// activates a mind-trick script, charms an enemy onto the caster's team (mind-trick 3),
/// confuses a lower-rank enemy, or has an ally respond — then plays the touch effect. If it
/// missed an NPC but hit world geometry (mind-trick >1), spawns a distraction sight/sound
/// event. Writes the trace result into `*tr` for the caller; sets `*tookPower` when it
/// consumed force power on a distraction. Returns `qtrue` if it acted directly on an NPC.
///
/// No oracle — `trap_Trace` + live-entity-array / NPC-state mutation + effect/sound side
/// effects (the w_force activator-leaf precedent).
///
/// # Safety
/// `self_` must be a valid client entity; `tr`/`tookPower` must be valid out-pointers.
unsafe fn ForceTelepathyCheckDirectNPCTarget(
    self_: *mut gentity_t,
    tr: *mut trace_t,
    tookPower: *mut qboolean,
) -> qboolean {
    let mut targetLive: qboolean = QFALSE;
    let mut tfrom: vec3_t = [0.0; 3];
    let mut tto: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let radius: f32 = MAX_TRICK_DISTANCE as f32;

    //Check for a direct usage on NPCs first
    VectorCopy(&(*(*self_).client).ps.origin, &mut tfrom);
    tfrom[2] += (*(*self_).client).ps.viewheight as f32;
    AngleVectors(&(*(*self_).client).ps.viewangles, Some(&mut fwd), None, None);
    tto[0] = tfrom[0] + fwd[0] * radius / 2.0;
    tto[1] = tfrom[1] + fwd[1] * radius / 2.0;
    tto[2] = tfrom[2] + fwd[2] * radius / 2.0;

    *tr = crate::trap::Trace(
        &tfrom,
        &vec3_origin,
        &vec3_origin,
        &tto,
        (*self_).s.number,
        MASK_PLAYERSOLID,
    );

    if (*tr).entityNum as c_int == ENTITYNUM_NONE
        || (*tr).fraction == 1.0
        || (*tr).allsolid != 0
        || (*tr).startsolid != 0
    {
        return QFALSE;
    }

    let traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*tr).entityNum as usize);

    if !(*traceEnt).NPC.is_null() && (*(*traceEnt).NPC).scriptFlags & SCF_NO_FORCE != 0 {
        return QFALSE;
    }

    if !traceEnt.is_null() && !(*traceEnt).client.is_null() {
        match (*(*traceEnt).client).NPC_class {
            CLASS_GALAKMECH //cant grip him, he's in armor
            | CLASS_ATST //much too big to grip!
            //no droids either
            | CLASS_PROBE
            | CLASS_GONK
            | CLASS_R2D2
            | CLASS_R5D2
            | CLASS_MARK1
            | CLASS_MARK2
            | CLASS_MOUSE
            | CLASS_SEEKER
            | CLASS_REMOTE
            | CLASS_PROTOCOL
            | CLASS_BOBAFETT
            | CLASS_RANCOR => {}
            _ => {
                targetLive = QTRUE;
            }
        }
    }

    if (*traceEnt).s.number < MAX_CLIENTS as c_int {
        //a regular client
        return QFALSE;
    }

    if targetLive != QFALSE && !(*traceEnt).NPC.is_null() {
        //hit an organic non-player
        let mut eyeDir: vec3_t = [0.0; 3];
        if G_ActivateBehavior(traceEnt, BSET_MINDTRICK) != QFALSE {
            //activated a script on him
            //FIXME: do the visual sparkles effect on their heads, still?
            WP_ForcePowerStart(self_, FP_TELEPATHY, 0);
        } else if ((*self_).NPC.is_null() == false
            && (*(*traceEnt).client).playerTeam != (*(*self_).client).playerTeam)
            || ((*self_).NPC.is_null()
                && (*(*traceEnt).client).playerTeam
                    != (*(*self_).client).sess.sessionTeam)
        {
            //an enemy
            let override_amt: c_int = 0;
            if (*(*traceEnt).NPC).scriptFlags & SCF_NO_MIND_TRICK != 0 {
            } else if (*traceEnt).s.weapon != WP_SABER
                && (*(*traceEnt).client).NPC_class != CLASS_REBORN
            {
                //haha!  Jedi aren't easily confused!
                if (*(*self_).client).ps.fd.forcePowerLevel[FP_TELEPATHY as usize]
                    > FORCE_LEVEL_2
                {
                    //turn them to our side
                    //if mind trick 3 and aiming at an enemy need more force power
                    if (*traceEnt).s.weapon != WP_NONE {
                        //don't charm people who aren't capable of fighting... like ugnaughts and droids
                        let newPlayerTeam: c_int;
                        let newEnemyTeam: c_int;

                        if !(*traceEnt).enemy.is_null() {
                            G_ClearEnemy(traceEnt);
                        }
                        if !(*traceEnt).NPC.is_null() {
                            //traceEnt->NPC->tempBehavior = BS_FOLLOW_LEADER;
                            (*(*traceEnt).client).leader = self_;
                        }
                        //FIXME: maybe pick an enemy right here?
                        if !(*self_).NPC.is_null() {
                            //NPC
                            newPlayerTeam = (*(*self_).client).playerTeam;
                            newEnemyTeam = (*(*self_).client).enemyTeam;
                        } else {
                            //client/bot
                            if (*(*self_).client).sess.sessionTeam == TEAM_BLUE {
                                //rebel
                                newPlayerTeam = NPCTEAM_PLAYER;
                                newEnemyTeam = NPCTEAM_ENEMY;
                            } else if (*(*self_).client).sess.sessionTeam == TEAM_RED {
                                //imperial
                                newPlayerTeam = NPCTEAM_ENEMY;
                                newEnemyTeam = NPCTEAM_PLAYER;
                            } else {
                                //neutral - wan't attack anyone
                                newPlayerTeam = NPCTEAM_NEUTRAL;
                                newEnemyTeam = NPCTEAM_NEUTRAL;
                            }
                        }
                        //store these for retrieval later
                        (*traceEnt).genericValue1 = (*(*traceEnt).client).playerTeam;
                        (*traceEnt).genericValue2 = (*(*traceEnt).client).enemyTeam;
                        (*traceEnt).genericValue3 = (*traceEnt).s.teamowner;
                        //set the new values
                        (*(*traceEnt).client).playerTeam = newPlayerTeam;
                        (*(*traceEnt).client).enemyTeam = newEnemyTeam;
                        (*traceEnt).s.teamowner = newPlayerTeam;
                        //FIXME: need a *charmed* timer on this...?  Or do TEAM_PLAYERS assume that "confusion" means they should switch to team_enemy when done?
                        (*(*traceEnt).NPC).charmedTime = (*addr_of!(level)).time
                            + mindTrickTime[(*(*self_).client).ps.fd.forcePowerLevel
                                [FP_TELEPATHY as usize]
                                as usize];
                    }
                } else {
                    //just confuse them
                    //somehow confuse them?  Set don't fire to true for a while?  Drop their aggression?  Maybe just take their enemy away and don't let them pick one up for a while unless shot?
                    (*(*traceEnt).NPC).confusionTime = (*addr_of!(level)).time
                        + mindTrickTime[(*(*self_).client).ps.fd.forcePowerLevel
                            [FP_TELEPATHY as usize]
                            as usize]; //confused for about 10 seconds
                    NPC_PlayConfusionSound(traceEnt);
                    if !(*traceEnt).enemy.is_null() {
                        G_ClearEnemy(traceEnt);
                    }
                }
            } else {
                NPC_Jedi_PlayConfusionSound(traceEnt);
            }
            WP_ForcePowerStart(self_, FP_TELEPATHY, override_amt);
        } else if (*(*traceEnt).client).playerTeam == (*(*self_).client).playerTeam {
            //an ally
            //maybe just have him look at you?  Respond?  Take your enemy?
            if (*(*traceEnt).client).ps.pm_type < PM_DEAD
                && (*traceEnt).NPC.is_null() == false
                && (*(*traceEnt).NPC).scriptFlags & SCF_NO_RESPONSE == 0
            {
                NPC_UseResponse(traceEnt, self_, QFALSE);
                WP_ForcePowerStart(self_, FP_TELEPATHY, 1);
            }
        } //NOTE: no effect on TEAM_NEUTRAL?
        AngleVectors(
            &(*(*traceEnt).client).renderInfo.eyeAngles,
            Some(&mut eyeDir),
            None,
            None,
        );
        VectorNormalize(&mut eyeDir);
        G_PlayEffectID(
            G_EffectIndex("force/force_touch"),
            &(*(*traceEnt).client).renderInfo.eyePoint,
            &eyeDir,
        );

        //make sure this plays and that you cannot press fire for about 1 second after this
        //FIXME: BOTH_FORCEMINDTRICK or BOTH_FORCEDISTRACT
        //NPC_SetAnim( self, SETANIM_TORSO, BOTH_MINDTRICK1, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_RESTART|SETANIM_FLAG_HOLD );
        //FIXME: build-up or delay this until in proper part of anim
        // mindTrickDone = qtrue; (vestigial in C — never read)
    } else {
        if (*(*self_).client).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] > FORCE_LEVEL_1
            && (*tr).fraction * 2048.0 > 64.0
        {
            //don't create a diversion less than 64 from you of if at power level 1
            //use distraction anim instead
            G_PlayEffectID(
                G_EffectIndex("force/force_touch"),
                &(*tr).endpos,
                &(*tr).plane.normal,
            );
            //FIXME: these events don't seem to always be picked up...?
            AddSoundEvent(self_, &(*tr).endpos, 512.0, AEL_SUSPICIOUS, QTRUE); //, qtrue );
            AddSightEvent(self_, &(*tr).endpos, 512.0, AEL_SUSPICIOUS, 50.0);
            WP_ForcePowerStart(self_, FP_TELEPATHY, 0);
            *tookPower = QTRUE;
        }
        //NPC_SetAnim( self, SETANIM_TORSO, BOTH_MINDTRICK2, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_RESTART|SETANIM_FLAG_HOLD );
    }
    //self->client->ps.saberMove = self->client->ps.saberBounceMove = LS_READY;//don't finish whatever saber anim you may have been in
    (*(*self_).client).ps.saberBlocked = BLOCKED_NONE;
    (*(*self_).client).ps.weaponTime = 1000;
    /*
    if ( self->client->ps.fd.forcePowersActive&(1<<FP_SPEED) )
    {
        self->client->ps.weaponTime = floor( self->client->ps.weaponTime * g_timescale->value );
    }
    */
    QTRUE
}

/// `void ForceTelepathy(gentity_t *self)` (w_force.c:2730) — the FP_TELEPATHY (mind-trick)
/// activator. Bails when dead, mid hand-extend, mid weapon-time, carrying a flag, or when the
/// power isn't usable; toggles off when already active past the deactivate debounce. First
/// tries the direct-on-NPC path (`ForceTelepathyCheckDirectNPCTarget`). Otherwise, at level 1
/// mind-tricks the single client the forward trace hit; at levels 2/3 boxes everyone in a
/// `visionArc` (180°/360°) within `radius` and mind-tricks each visible, force-usable enemy
/// client.
///
/// No oracle — `trap_Trace`/`trap_EntitiesInBox` + live-entity-array / playerState mutation +
/// sound side effects (the w_force activator-leaf precedent).
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`.
unsafe fn ForceTelepathy(self_: *mut gentity_t) {
    let mut tr: trace_t = trace_t::default();
    let mut tto: vec3_t = [0.0; 3];
    let mut thispush_org: vec3_t = [0.0; 3];
    let mut a: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut fwdangles: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut center: vec3_t = [0.0; 3];
    let mut visionArc: f32 = 0.0;
    let mut radius: f32 = MAX_TRICK_DISTANCE as f32;
    let mut tookPower: qboolean = QFALSE;

    if (*self_).health <= 0 {
        return;
    }

    if (*(*self_).client).ps.forceHandExtend != HANDEXTEND_NONE {
        return;
    }

    if (*(*self_).client).ps.weaponTime > 0 {
        return;
    }

    if (*(*self_).client).ps.powerups[PW_REDFLAG as usize] != 0
        || (*(*self_).client).ps.powerups[PW_BLUEFLAG as usize] != 0
    {
        //can't mindtrick while carrying the flag
        return;
    }

    if (*(*self_).client).ps.forceAllowDeactivateTime < (*addr_of!(level)).time
        && (*(*self_).client).ps.fd.forcePowersActive & (1 << FP_TELEPATHY) != 0
    {
        WP_ForcePowerStop(self_, FP_TELEPATHY);
        return;
    }

    if WP_ForcePowerUsable(self_, FP_TELEPATHY) == QFALSE {
        return;
    }

    if ForceTelepathyCheckDirectNPCTarget(self_, &mut tr, &mut tookPower) != QFALSE {
        //hit an NPC directly
        (*(*self_).client).ps.forceAllowDeactivateTime = (*addr_of!(level)).time + 1500;
        G_Sound(
            self_,
            CHAN_AUTO,
            G_SoundIndex("sound/weapons/force/distract.wav"),
        );
        (*(*self_).client).ps.forceHandExtend = HANDEXTEND_FORCEPUSH;
        (*(*self_).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1000;
        return;
    }

    if (*(*self_).client).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_2 {
        visionArc = 180.0;
    } else if (*(*self_).client).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_3 {
        visionArc = 360.0;
        radius = MAX_TRICK_DISTANCE as f32 * 2.0;
    }

    VectorCopy(&(*(*self_).client).ps.viewangles, &mut fwdangles);
    AngleVectors(&fwdangles, Some(&mut forward), Some(&mut right), None);
    VectorCopy(&(*(*self_).client).ps.origin, &mut center);

    for i in 0..3 {
        mins[i] = center[i] - radius;
        maxs[i] = center[i] + radius;
    }

    if (*(*self_).client).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_1 {
        if tr.fraction != 1.0
            && (tr.entityNum as c_int) != ENTITYNUM_NONE
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize)).inuse != QFALSE
            && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize)).client.is_null()
            && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize)).client)
                .pers
                .connected
                != 0
            && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize)).client)
                .sess
                .sessionTeam
                != TEAM_SPECTATOR
        {
            WP_AddAsMindtricked(&mut (*(*self_).client).ps.fd, tr.entityNum as c_int);
            if tookPower == QFALSE {
                WP_ForcePowerStart(self_, FP_TELEPATHY, 0);
            }

            G_Sound(
                self_,
                CHAN_AUTO,
                G_SoundIndex("sound/weapons/force/distract.wav"),
            );

            (*(*self_).client).ps.forceHandExtend = HANDEXTEND_FORCEPUSH;
            (*(*self_).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1000;
        } else {
        }
    } else {
        //level 2 & 3
        let mut ent: *mut gentity_t;
        let mut entityList: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
        let numListedEntities: c_int;
        let mut e: c_int = 0;
        let mut gotatleastone: qboolean = QFALSE;

        numListedEntities = crate::trap::EntitiesInBox(&mins, &maxs, &mut entityList);

        while e < numListedEntities {
            ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityList[e as usize] as usize);

            if !ent.is_null() {
                //not in the arc, don't consider it
                if !(*ent).client.is_null() {
                    VectorCopy(&(*(*ent).client).ps.origin, &mut thispush_org);
                } else {
                    VectorCopy(&(*ent).s.pos.trBase, &mut thispush_org);
                }
                VectorCopy(&(*(*self_).client).ps.origin, &mut tto);
                tto[2] += (*(*self_).client).ps.viewheight as f32;
                VectorSubtract(&thispush_org, &tto, &mut a);
                let a_copy = a;
                vectoangles(&a_copy, &mut a);

                if (*ent).client.is_null() {
                    entityList[e as usize] = ENTITYNUM_NONE;
                } else if InFieldOfVision(&(*(*self_).client).ps.viewangles, visionArc, &mut a)
                    == 0
                {
                    //only bother with arc rules if the victim is a client
                    entityList[e as usize] = ENTITYNUM_NONE;
                } else if ForcePowerUsableOn(self_, ent, FP_TELEPATHY) == QFALSE {
                    entityList[e as usize] = ENTITYNUM_NONE;
                } else if OnSameTeam(self_, ent) != QFALSE {
                    entityList[e as usize] = ENTITYNUM_NONE;
                }
            }
            ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityList[e as usize] as usize);
            if !ent.is_null() && ent != self_ && !(*ent).client.is_null() {
                gotatleastone = QTRUE;
                WP_AddAsMindtricked(&mut (*(*self_).client).ps.fd, (*ent).s.number);
            }
            e += 1;
        }

        if gotatleastone != QFALSE {
            (*(*self_).client).ps.forceAllowDeactivateTime = (*addr_of!(level)).time + 1500;

            if tookPower == QFALSE {
                WP_ForcePowerStart(self_, FP_TELEPATHY, 0);
            }

            G_Sound(
                self_,
                CHAN_AUTO,
                G_SoundIndex("sound/weapons/force/distract.wav"),
            );

            (*(*self_).client).ps.forceHandExtend = HANDEXTEND_FORCEPUSH;
            (*(*self_).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1000;
        }
    }
}

/// `static void WP_ForcePowerRun( gentity_t *self, forcePowers_t forcePower, usercmd_t *cmd )`
/// (w_force.c:4280) — runs one per-frame tick of a single *currently-active* force power,
/// dispatching on `forcePower`. Each case applies the ongoing effect (heal increments,
/// rage health drain, grip/lightning/drain channelling, mind-trick updates) and calls
/// `WP_ForcePowerStop` when the power's run conditions are no longer met.
///
/// No oracle — entity-state/playerState mutation dispatch that fires the per-power handlers
/// (`ForceShootDrain`/`ForceShootLightning`/`DoGripAction`/`WP_UpdateMindtrickEnts`), which
/// themselves touch the live entity array / sound traps (the entity/trap control-flow
/// precedent shared with `WP_DoSpecificPower`).
///
/// # Safety
/// `self_` must be a valid client entity (`self_->client` non-null), `cmd` must be a valid
/// `usercmd_t`, and `g_entities` initialised (the Siege-item branch indexes it by
/// `holdingObjectiveItem`).
unsafe fn WP_ForcePowerRun(self_: *mut gentity_t, force_power: c_int, cmd: *mut usercmd_t) {
    // C: `extern usercmd_t ucmd;` — declared at the top of the function but never referenced
    // (a dead declaration). Intentionally omitted.
    let client = (*self_).client;

    match force_power {
        FP_HEAL => {
            if (*client).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_1
                && ((*client).ps.velocity[0] != 0.0
                    || (*client).ps.velocity[1] != 0.0
                    || (*client).ps.velocity[2] != 0.0)
            {
                WP_ForcePowerStop(self_, force_power);
                return;
            }

            if (*self_).health < 1 || (*client).ps.stats[STAT_HEALTH as usize] < 1 {
                WP_ForcePowerStop(self_, force_power);
                return;
            }

            if (*client).ps.fd.forceHealTime > (*addr_of!(level)).time {
                return;
            }
            if (*self_).health > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
                //rww - we might start out over max_health and we don't want force heal taking us down to 100 or whatever max_health is
                WP_ForcePowerStop(self_, force_power);
                return;
            }
            (*client).ps.fd.forceHealTime = (*addr_of!(level)).time + 1000;
            (*self_).health += 1;
            (*client).ps.fd.forceHealAmount += 1;

            if (*self_).health > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
                // Past max health
                (*self_).health = (*client).ps.stats[STAT_MAX_HEALTH as usize];
                WP_ForcePowerStop(self_, force_power);
            }

            if ((*client).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_1
                && (*client).ps.fd.forceHealAmount >= 25)
                || ((*client).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_2
                    && (*client).ps.fd.forceHealAmount >= 33)
            {
                WP_ForcePowerStop(self_, force_power);
            }
        }
        FP_SPEED => {
            //This is handled in PM_WalkMove and PM_StepSlideMove
            if (*client).holdingObjectiveItem >= MAX_CLIENTS as c_int
                && (*client).holdingObjectiveItem < ENTITYNUM_WORLD
            {
                if (*core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*client).holdingObjectiveItem as usize)).genericValue15 != 0 {
                    //disables force powers
                    WP_ForcePowerStop(self_, force_power);
                }
            }
            /*
            if ( self->client->ps.powerups[PW_REDFLAG]
                || self->client->ps.powerups[PW_BLUEFLAG]
                || self->client->ps.powerups[PW_NEUTRALFLAG] )
            {//no force speed when carrying flag
                WP_ForcePowerStop( self, forcePower );
            }
            */
        }
        FP_GRIP => {
            if (*client).ps.forceHandExtend != HANDEXTEND_FORCE_HOLD {
                WP_ForcePowerStop(self_, FP_GRIP);
                return;
            }

            if (*client).ps.fd.forcePowerDebounce[FP_PULL as usize] < (*addr_of!(level)).time {
                //This is sort of not ideal. Using the debounce value reserved for pull for this because pull doesn't need it.
                BG_ForcePowerDrain(&mut (*client).ps, force_power, 1);
                (*client).ps.fd.forcePowerDebounce[FP_PULL as usize] = (*addr_of!(level)).time + 100;
            }

            if (*client).ps.fd.forcePower < 1 {
                WP_ForcePowerStop(self_, FP_GRIP);
                return;
            }

            DoGripAction(self_, force_power);
        }
        FP_LEVITATION => {
            if (*client).ps.groundEntityNum != ENTITYNUM_NONE
                && (*client).ps.fd.forceJumpZStart == 0.0
            {
                //done with jump
                WP_ForcePowerStop(self_, force_power);
            }
        }
        FP_RAGE => {
            if (*self_).health < 1 {
                WP_ForcePowerStop(self_, force_power);
                return;
            }
            if (*client).ps.forceRageDrainTime < (*addr_of!(level)).time {
                let mut add_time: c_int = 400;

                (*self_).health -= 2;

                if (*client).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_1 {
                    add_time = 150;
                } else if (*client).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_2 {
                    add_time = 300;
                } else if (*client).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_3 {
                    add_time = 450;
                }
                (*client).ps.forceRageDrainTime = (*addr_of!(level)).time + add_time;
            }

            if (*self_).health < 1 {
                (*self_).health = 1;
                WP_ForcePowerStop(self_, force_power);
            }

            (*client).ps.stats[STAT_HEALTH as usize] = (*self_).health;
        }
        FP_DRAIN => {
            if (*client).ps.forceHandExtend != HANDEXTEND_FORCE_HOLD {
                WP_ForcePowerStop(self_, force_power);
                return;
            }

            if (*client).ps.fd.forcePowerLevel[FP_DRAIN as usize] > FORCE_LEVEL_1 {
                //higher than level 1
                if ((*cmd).buttons & BUTTON_FORCE_DRAIN as c_int) != 0
                    || (((*cmd).buttons & BUTTON_FORCEPOWER as c_int) != 0
                        && (*client).ps.fd.forcePowerSelected == FP_DRAIN)
                {
                    //holding it keeps it going
                    (*client).ps.fd.forcePowerDuration[FP_DRAIN as usize] =
                        (*addr_of!(level)).time + 500;
                }
            }
            // OVERRIDEFIXME
            if WP_ForcePowerAvailable(self_, force_power, 0) == QFALSE
                || (*client).ps.fd.forcePowerDuration[FP_DRAIN as usize] < (*addr_of!(level)).time
                || (*client).ps.fd.forcePower < 25
            {
                WP_ForcePowerStop(self_, force_power);
            } else {
                ForceShootDrain(self_);
            }
        }
        FP_LIGHTNING => {
            if (*client).ps.forceHandExtend != HANDEXTEND_FORCE_HOLD {
                //Animation for hand extend doesn't end with hand out, so we have to limit lightning intervals by animation intervals (once hand starts to go in in animation, lightning should stop)
                WP_ForcePowerStop(self_, force_power);
                return;
            }

            if (*client).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] > FORCE_LEVEL_1 {
                //higher than level 1
                if ((*cmd).buttons & BUTTON_FORCE_LIGHTNING as c_int) != 0
                    || (((*cmd).buttons & BUTTON_FORCEPOWER as c_int) != 0
                        && (*client).ps.fd.forcePowerSelected == FP_LIGHTNING)
                {
                    //holding it keeps it going
                    (*client).ps.fd.forcePowerDuration[FP_LIGHTNING as usize] =
                        (*addr_of!(level)).time + 500;
                }
            }
            // OVERRIDEFIXME
            if WP_ForcePowerAvailable(self_, force_power, 0) == QFALSE
                || (*client).ps.fd.forcePowerDuration[FP_LIGHTNING as usize]
                    < (*addr_of!(level)).time
                || (*client).ps.fd.forcePower < 25
            {
                WP_ForcePowerStop(self_, force_power);
            } else {
                ForceShootLightning(self_);
                BG_ForcePowerDrain(&mut (*client).ps, force_power, 0);
            }
        }
        FP_TELEPATHY => {
            if (*client).holdingObjectiveItem >= MAX_CLIENTS as c_int
                && (*client).holdingObjectiveItem < ENTITYNUM_WORLD
                && (*core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*client).holdingObjectiveItem as usize)).genericValue15 != 0
            {
                //if force hindered can't mindtrick whilst carrying a siege item
                WP_ForcePowerStop(self_, FP_TELEPATHY);
            } else {
                WP_UpdateMindtrickEnts(self_);
            }
        }
        FP_SABER_OFFENSE => {}
        FP_SABER_DEFENSE => {}
        FP_SABERTHROW => {}
        FP_PROTECT => {
            if (*client).ps.fd.forcePowerDebounce[force_power as usize] < (*addr_of!(level)).time {
                BG_ForcePowerDrain(&mut (*client).ps, force_power, 1);
                if (*client).ps.fd.forcePower < 1 {
                    WP_ForcePowerStop(self_, force_power);
                }

                (*client).ps.fd.forcePowerDebounce[force_power as usize] =
                    (*addr_of!(level)).time + 300;
            }
        }
        FP_ABSORB => {
            if (*client).ps.fd.forcePowerDebounce[force_power as usize] < (*addr_of!(level)).time {
                BG_ForcePowerDrain(&mut (*client).ps, force_power, 1);
                if (*client).ps.fd.forcePower < 1 {
                    WP_ForcePowerStop(self_, force_power);
                }

                (*client).ps.fd.forcePowerDebounce[force_power as usize] =
                    (*addr_of!(level)).time + 600;
            }
        }
        _ => {}
    }
}

/// `int WP_DoSpecificPower( gentity_t *self, usercmd_t *ucmd, forcePowers_t forcepower )`
/// (w_force.c:4506) — activate a single force power for `self`, dispatching on `forcepower`.
/// Returns `1` if the power "succeeded" (held powers), `0` for non-hold powers (which always
/// report 0) or if the power could not be afforded / a hold-release is pending.
///
/// No oracle — entity-state force dispatch (mutates `self->client->ps.fd` and fires the
/// per-power handlers, which themselves touch the live entity array / traps).
///
/// # Safety
/// `self` must be a valid client entity (`self->client` non-null) and `ucmd` valid.
pub unsafe fn WP_DoSpecificPower(
    self_: *mut gentity_t,
    ucmd: *mut usercmd_t,
    forcepower: c_int,
) -> c_int {
    let mut power_succeeded: c_int;

    power_succeeded = 1;

    // OVERRIDEFIXME
    if WP_ForcePowerAvailable(self_, forcepower, 0) == QFALSE {
        return 0;
    }

    let client = (*self_).client;

    match forcepower {
        FP_HEAL => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceHeal(self_);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_LEVITATION => {
            //if leave the ground by some other means, cancel the force jump so we don't suddenly jump when we land.

            if (*client).ps.groundEntityNum == ENTITYNUM_NONE {
                (*client).ps.fd.forceJumpCharge = 0.0;
                G_MuteSound(
                    (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 - 50) as usize],
                    CHAN_VOICE,
                );
            //This only happens if the groundEntityNum == ENTITYNUM_NONE when the button is actually released
            } else {
                //still on ground, so jump
                ForceJump(self_, ucmd);
            }
        }
        FP_SPEED => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceSpeed(self_, 0);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_GRIP => {
            if (*client).ps.fd.forceGripEntityNum == ENTITYNUM_NONE {
                ForceGrip(self_);
            }

            if (*client).ps.fd.forceGripEntityNum != ENTITYNUM_NONE {
                if (*client).ps.fd.forcePowersActive & (1 << FP_GRIP) == 0 {
                    WP_ForcePowerStart(self_, FP_GRIP, 0);
                    BG_ForcePowerDrain(&mut (*client).ps, FP_GRIP, GRIP_DRAIN_AMOUNT);
                }
            } else {
                power_succeeded = 0;
            }
        }
        FP_LIGHTNING => {
            ForceLightning(self_);
        }
        FP_PUSH => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 && (*self_).r.svFlags & SVF_BOT == 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceThrow(self_, QFALSE);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_PULL => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceThrow(self_, QTRUE);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_TELEPATHY => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceTelepathy(self_);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_RAGE => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceRage(self_);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_PROTECT => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceProtect(self_);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_ABSORB => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceAbsorb(self_);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_TEAM_HEAL => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceTeamHeal(self_);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_TEAM_FORCE => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceTeamForceReplenish(self_);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_DRAIN => {
            ForceDrain(self_);
        }
        FP_SEE => {
            power_succeeded = 0; //always 0 for nonhold powers
            if (*client).ps.fd.forceButtonNeedRelease != 0 {
                //need to release before we can use nonhold powers again
            } else {
                ForceSeeing(self_);
                (*client).ps.fd.forceButtonNeedRelease = 1;
            }
        }
        FP_SABER_OFFENSE => {}
        FP_SABER_DEFENSE => {}
        FP_SABERTHROW => {}
        _ => {}
    }

    power_succeeded
}

/// `void WP_ForcePowersUpdate( gentity_t *self, usercmd_t *ucmd )` (w_force.c:5096) — the
/// per-frame force-power tick for one client: deactivate powers that should stop, apply
/// enlightenment / ysalamiri overrides, service the per-button powers (jump/grip/lightning/
/// drain + the generic selected power), run each active power, and regenerate force when idle.
///
/// No oracle — entity-state update (touches the live entity array, fires per-power handlers,
/// reads cvars/level). KEYSTONE.
///
/// # Safety
/// `self` may be null (guarded). When non-null, `self->client` and `ucmd` must be valid.
#[allow(unused_assignments)] // faithful: C dead-resets `i = 0` between blocks before each loop reassigns it
pub unsafe fn WP_ForcePowersUpdate(self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    let mut using_force: qboolean = QFALSE;
    let mut i: c_int;
    let mut prepower: c_int = 0;
    //see if any force powers are running
    if self_.is_null() {
        return;
    }

    if (*self_).client.is_null() {
        return;
    }

    let client = (*self_).client;

    if (*client).ps.pm_flags & PMF_FOLLOW != 0 {
        //not a "real" game client, it's a spectator following someone
        return;
    }
    if (*client).sess.sessionTeam == TEAM_SPECTATOR {
        return;
    }

    /*
    if (self->client->ps.fd.saberAnimLevel > self->client->ps.fd.forcePowerLevel[FP_SABER_OFFENSE])
    {
        self->client->ps.fd.saberAnimLevel = self->client->ps.fd.forcePowerLevel[FP_SABER_OFFENSE];
    }
    else if (!self->client->ps.fd.saberAnimLevel)
    {
        self->client->ps.fd.saberAnimLevel = FORCE_LEVEL_1;
    }
    */
    //The stance in relation to power level is no longer applicable with the crazy new akimbo/staff stances.
    if (*client).ps.fd.saberAnimLevel == 0 {
        (*client).ps.fd.saberAnimLevel = FORCE_LEVEL_1;
    }

    if (*addr_of!(g_gametype)).integer != GT_SIEGE {
        if (*client).ps.fd.forcePowersKnown & (1 << FP_LEVITATION) == 0 {
            (*client).ps.fd.forcePowersKnown |= 1 << FP_LEVITATION;
        }

        if (*client).ps.fd.forcePowerLevel[FP_LEVITATION as usize] < FORCE_LEVEL_1 {
            (*client).ps.fd.forcePowerLevel[FP_LEVITATION as usize] = FORCE_LEVEL_1;
        }
    }

    if (*client).ps.fd.forcePowerSelected < 0 {
        //bad
        (*client).ps.fd.forcePowerSelected = 0;
    }

    if ((*client).sess.selectedFP != (*client).ps.fd.forcePowerSelected as c_int
        || (*client).sess.saberLevel != (*client).ps.fd.saberAnimLevel)
        && (*self_).r.svFlags & SVF_BOT == 0
    {
        if (*client).sess.updateUITime < (*addr_of!(level)).time {
            //a bit hackish, but we don't want the client to flood with userinfo updates if they rapidly cycle
            //through their force powers or saber attack levels

            (*client).sess.selectedFP = (*client).ps.fd.forcePowerSelected as c_int;
            (*client).sess.saberLevel = (*client).ps.fd.saberAnimLevel;
        }
    }

    if g_LastFrameTime == 0 {
        g_LastFrameTime = (*addr_of!(level)).time;
    }

    if (*client).ps.forceHandExtend == HANDEXTEND_KNOCKDOWN {
        (*client).ps.zoomFov = 0.0;
        (*client).ps.zoomMode = 0;
        (*client).ps.zoomLocked = QFALSE;
        (*client).ps.zoomTime = 0;
    }

    if (*client).ps.forceHandExtend == HANDEXTEND_KNOCKDOWN
        && (*client).ps.forceHandExtendTime >= (*addr_of!(level)).time
    {
        (*client).ps.saberMove = 0;
        (*client).ps.saberBlocking = 0;
        (*client).ps.saberBlocked = 0;
        (*client).ps.weaponTime = 0;
        (*client).ps.weaponstate = WEAPON_READY;
    } else if (*client).ps.forceHandExtend != HANDEXTEND_NONE
        && (*client).ps.forceHandExtendTime < (*addr_of!(level)).time
    {
        if (*client).ps.forceHandExtend == HANDEXTEND_KNOCKDOWN && (*client).ps.forceDodgeAnim == 0 {
            if (*self_).health < 1 || (*client).ps.eFlags & EF_DEAD != 0 {
                (*client).ps.forceHandExtend = HANDEXTEND_NONE;
            } else if G_SpecialRollGetup(self_) != QFALSE {
                (*client).ps.forceHandExtend = HANDEXTEND_NONE;
            } else {
                //hmm.. ok.. no more getting up on your own, you've gotta push something, unless..
                if ((*addr_of!(level)).time - (*client).ps.forceHandExtendTime) > 4000 {
                    //4 seconds elapsed, I guess they're too dumb to push something to get up!
                    if (*client).pers.cmd.upmove != 0
                        && (*client).ps.fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_1
                    {
                        //force getup
                        G_PreDefSound(&(*client).ps.origin, PDSOUND_FORCEJUMP);
                        (*client).ps.forceDodgeAnim = 2;
                        (*client).ps.forceHandExtendTime = (*addr_of!(level)).time + 500;

                    //self->client->ps.velocity[2] = 400;
                    } else if (*client).ps.quickerGetup != QFALSE {
                        G_EntitySound(self_, CHAN_VOICE, G_SoundIndex("*jump1.wav"));
                        (*client).ps.forceDodgeAnim = 3;
                        (*client).ps.forceHandExtendTime = (*addr_of!(level)).time + 500;
                        (*client).ps.velocity[2] = 300.0;
                    } else {
                        (*client).ps.forceDodgeAnim = 1;
                        (*client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1000;
                    }
                }
            }
            (*client).ps.quickerGetup = QFALSE;
        } else if (*client).ps.forceHandExtend == HANDEXTEND_POSTTHROWN {
            if (*self_).health < 1 || (*client).ps.eFlags & EF_DEAD != 0 {
                (*client).ps.forceHandExtend = HANDEXTEND_NONE;
            } else if (*client).ps.groundEntityNum != ENTITYNUM_NONE
                && (*client).ps.forceDodgeAnim == 0
            {
                (*client).ps.forceDodgeAnim = 1;
                (*client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1000;
                G_EntitySound(self_, CHAN_VOICE, G_SoundIndex("*jump1.wav"));
                (*client).ps.velocity[2] = 100.0;
            } else if (*client).ps.forceDodgeAnim == 0 {
                (*client).ps.forceHandExtendTime = (*addr_of!(level)).time + 100;
            } else {
                (*client).ps.forceHandExtend = HANDEXTEND_WEAPONREADY;
            }
        } else {
            (*client).ps.forceHandExtend = HANDEXTEND_WEAPONREADY;
        }
    }

    if (*addr_of!(g_gametype)).integer == GT_HOLOCRON {
        HolocronUpdate(self_);
    }
    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER {
        JediMasterUpdate(self_);
    }

    SeekerDroneUpdate(self_);

    if (*client).ps.powerups[PW_FORCE_BOON as usize] != 0 {
        prepower = (*client).ps.fd.forcePower;
    }

    'powersetcheck: {
        if BG_HasYsalamiri((*addr_of!(g_gametype)).integer, &mut (*client).ps) != QFALSE
            || (*client).ps.fd.forceDeactivateAll != 0
            || (*client).tempSpectate >= (*addr_of!(level)).time
        {
            //has ysalamiri.. or we want to forcefully stop all his active powers
            i = 0;

            while i < NUM_FORCE_POWERS as c_int {
                if (*client).ps.fd.forcePowersActive & (1 << i) != 0 && i != FP_LEVITATION {
                    WP_ForcePowerStop(self_, i);
                }

                i += 1;
            }

            if (*client).tempSpectate >= (*addr_of!(level)).time {
                (*client).ps.fd.forcePower = 100;
                (*client).ps.fd.forceRageRecoveryTime = 0;
            }

            (*client).ps.fd.forceDeactivateAll = 0;

            if (*client).ps.fd.forceJumpCharge != 0.0 {
                G_MuteSound(
                    (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 - 50) as usize],
                    CHAN_VOICE,
                );
                (*client).ps.fd.forceJumpCharge = 0.0;
            }
        } else {
            //otherwise just do a check through them all to see if they need to be stopped for any reason.
            i = 0;

            while i < NUM_FORCE_POWERS as c_int {
                if (*client).ps.fd.forcePowersActive & (1 << i) != 0
                    && i != FP_LEVITATION
                    && BG_CanUseFPNow(
                        (*addr_of!(g_gametype)).integer,
                        &mut (*client).ps,
                        (*addr_of!(level)).time,
                        i,
                    ) == QFALSE
                {
                    WP_ForcePowerStop(self_, i);
                }

                i += 1;
            }
        }

        i = 0;

        if (*client).ps.powerups[PW_FORCE_ENLIGHTENED_LIGHT as usize] != 0
            || (*client).ps.powerups[PW_FORCE_ENLIGHTENED_DARK as usize] != 0
        {
            //enlightenment
            if (*client).ps.fd.forceUsingAdded == 0 {
                i = 0;

                while i < NUM_FORCE_POWERS as c_int {
                    (*client).ps.fd.forcePowerBaseLevel[i as usize] =
                        (*client).ps.fd.forcePowerLevel[i as usize];

                    if forcePowerDarkLight[i as usize] == 0
                        || (*client).ps.fd.forceSide == forcePowerDarkLight[i as usize]
                    {
                        (*client).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_3;
                        (*client).ps.fd.forcePowersKnown |= 1 << i;
                    }

                    i += 1;
                }

                (*client).ps.fd.forceUsingAdded = 1;
            }
        } else if (*client).ps.fd.forceUsingAdded != 0 {
            //we don't have enlightenment but we're still using enlightened powers, so clear them back to how they should be.
            i = 0;

            while i < NUM_FORCE_POWERS as c_int {
                (*client).ps.fd.forcePowerLevel[i as usize] =
                    (*client).ps.fd.forcePowerBaseLevel[i as usize];
                if (*client).ps.fd.forcePowerLevel[i as usize] == 0 {
                    if (*client).ps.fd.forcePowersActive & (1 << i) != 0 {
                        WP_ForcePowerStop(self_, i);
                    }
                    (*client).ps.fd.forcePowersKnown &= !(1 << i);
                }

                i += 1;
            }

            (*client).ps.fd.forceUsingAdded = 0;
        }

        i = 0;

        if (*client).ps.fd.forcePowersActive & (1 << FP_TELEPATHY) == 0 {
            //clear the mindtrick index values
            (*client).ps.fd.forceMindtrickTargetIndex = 0;
            (*client).ps.fd.forceMindtrickTargetIndex2 = 0;
            (*client).ps.fd.forceMindtrickTargetIndex3 = 0;
            (*client).ps.fd.forceMindtrickTargetIndex4 = 0;
        }

        if (*self_).health < 1 {
            (*client).ps.fd.forceGripBeingGripped = 0.0;
        }

        if (*client).ps.fd.forceGripBeingGripped > (*addr_of!(level)).time as f32 {
            (*client).ps.fd.forceGripCripple = 1;

            //keep the saber off during this period
            if (*client).ps.weapon == WP_SABER && (*client).ps.saberHolstered == 0 {
                Cmd_ToggleSaber_f(self_);
            }
        } else {
            (*client).ps.fd.forceGripCripple = 0;
        }

        if (*client).ps.fd.forceJumpSound != 0 {
            G_PreDefSound(&(*client).ps.origin, PDSOUND_FORCEJUMP);
            (*client).ps.fd.forceJumpSound = 0;
        }

        if (*client).ps.fd.forceGripCripple != 0 {
            if (*client).ps.fd.forceGripSoundTime < (*addr_of!(level)).time as f32 {
                G_PreDefSound(&(*client).ps.origin, PDSOUND_FORCEGRIP);
                (*client).ps.fd.forceGripSoundTime = ((*addr_of!(level)).time + 1000) as f32;
            }
        }

        if (*client).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0 {
            (*client).ps.powerups[PW_SPEED as usize] = (*addr_of!(level)).time + 100;
        }

        if (*self_).health <= 0 {
            //if dead, deactivate any active force powers
            i = 0;
            while i < NUM_FORCE_POWERS as c_int {
                if (*client).ps.fd.forcePowerDuration[i as usize] != 0
                    || (*client).ps.fd.forcePowersActive & (1 << i) != 0
                {
                    WP_ForcePowerStop(self_, i);
                    (*client).ps.fd.forcePowerDuration[i as usize] = 0;
                }
                i += 1;
            }
            break 'powersetcheck;
        }

        if (*client).ps.groundEntityNum != ENTITYNUM_NONE {
            (*client).fjDidJump = QFALSE;
        }

        if (*client).ps.fd.forceJumpCharge != 0.0
            && (*client).ps.groundEntityNum == ENTITYNUM_NONE
            && (*client).fjDidJump != QFALSE
        {
            //this was for the "charge" jump method... I guess
            if (*ucmd).upmove < 10
                && ((*ucmd).buttons & BUTTON_FORCEPOWER == 0
                    || (*client).ps.fd.forcePowerSelected != FP_LEVITATION)
            {
                G_MuteSound(
                    (*client).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 - 50) as usize],
                    CHAN_VOICE,
                );
                (*client).ps.fd.forceJumpCharge = 0.0;
            }
        }
        // #ifndef METROID_JUMP
        else if (*ucmd).upmove > 10
            && (*client).ps.pm_flags & PMF_JUMP_HELD != 0
            && (*client).ps.groundTime != 0
            && ((*addr_of!(level)).time - (*client).ps.groundTime) > 150
            && BG_HasYsalamiri((*addr_of!(g_gametype)).integer, &mut (*client).ps) == QFALSE
            && BG_CanUseFPNow(
                (*addr_of!(g_gametype)).integer,
                &mut (*client).ps,
                (*addr_of!(level)).time,
                FP_LEVITATION,
            ) != QFALSE
        {
            //just charging up
            ForceJumpCharge(self_, ucmd);
            using_force = QTRUE;
        } else if (*ucmd).upmove < 10
            && (*client).ps.groundEntityNum == ENTITYNUM_NONE
            && (*client).ps.fd.forceJumpCharge != 0.0
        {
            (*client).ps.pm_flags &= !PMF_JUMP_HELD;
        }
        // #endif

        if (*client).ps.pm_flags & PMF_JUMP_HELD == 0 && (*client).ps.fd.forceJumpCharge != 0.0 {
            if (*ucmd).buttons & BUTTON_FORCEPOWER == 0
                || (*client).ps.fd.forcePowerSelected != FP_LEVITATION
            {
                if WP_DoSpecificPower(self_, ucmd, FP_LEVITATION) != 0 {
                    using_force = QTRUE;
                }
            }
        }

        if (*ucmd).buttons & BUTTON_FORCEGRIP != 0 {
            //grip is one of the powers with its own button.. if it's held, call the specific grip power function.
            if WP_DoSpecificPower(self_, ucmd, FP_GRIP) != 0 {
                using_force = QTRUE;
            } else {
                //don't let recharge even if the grip misses if the player still has the button down
                using_force = QTRUE;
            }
        } else {
            //see if we're using it generically.. if not, stop.
            if (*client).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0 {
                if (*ucmd).buttons & BUTTON_FORCEPOWER == 0
                    || (*client).ps.fd.forcePowerSelected != FP_GRIP
                {
                    WP_ForcePowerStop(self_, FP_GRIP);
                }
            }
        }

        if (*ucmd).buttons & BUTTON_FORCE_LIGHTNING != 0 {
            //lightning
            WP_DoSpecificPower(self_, ucmd, FP_LIGHTNING);
            using_force = QTRUE;
        } else {
            //see if we're using it generically.. if not, stop.
            if (*client).ps.fd.forcePowersActive & (1 << FP_LIGHTNING) != 0 {
                if (*ucmd).buttons & BUTTON_FORCEPOWER == 0
                    || (*client).ps.fd.forcePowerSelected != FP_LIGHTNING
                {
                    WP_ForcePowerStop(self_, FP_LIGHTNING);
                }
            }
        }

        if (*ucmd).buttons & BUTTON_FORCE_DRAIN != 0 {
            //drain
            WP_DoSpecificPower(self_, ucmd, FP_DRAIN);
            using_force = QTRUE;
        } else {
            //see if we're using it generically.. if not, stop.
            if (*client).ps.fd.forcePowersActive & (1 << FP_DRAIN) != 0 {
                if (*ucmd).buttons & BUTTON_FORCEPOWER == 0
                    || (*client).ps.fd.forcePowerSelected != FP_DRAIN
                {
                    WP_ForcePowerStop(self_, FP_DRAIN);
                }
            }
        }

        if (*ucmd).buttons & BUTTON_FORCEPOWER != 0
            && BG_CanUseFPNow(
                (*addr_of!(g_gametype)).integer,
                &mut (*client).ps,
                (*addr_of!(level)).time,
                (*client).ps.fd.forcePowerSelected,
            ) != QFALSE
        {
            if (*client).ps.fd.forcePowerSelected == FP_LEVITATION {
                ForceJumpCharge(self_, ucmd);
                using_force = QTRUE;
            } else if WP_DoSpecificPower(self_, ucmd, (*client).ps.fd.forcePowerSelected) != 0 {
                using_force = QTRUE;
            } else if (*client).ps.fd.forcePowerSelected == FP_GRIP {
                using_force = QTRUE;
            }
        } else {
            (*client).ps.fd.forceButtonNeedRelease = 0;
        }

        i = 0;
        while i < NUM_FORCE_POWERS as c_int {
            if (*client).ps.fd.forcePowerDuration[i as usize] != 0 {
                if (*client).ps.fd.forcePowerDuration[i as usize] < (*addr_of!(level)).time {
                    if (*client).ps.fd.forcePowersActive & (1 << i) != 0 {
                        //turn it off
                        WP_ForcePowerStop(self_, i);
                    }
                    (*client).ps.fd.forcePowerDuration[i as usize] = 0;
                }
            }
            if (*client).ps.fd.forcePowersActive & (1 << i) != 0 {
                using_force = QTRUE;
                WP_ForcePowerRun(self_, i, ucmd);
            }
            i += 1;
        }
        if (*client).ps.saberInFlight != 0 && (*client).ps.saberEntityNum != 0 {
            //don't regen force power while throwing saber
            if (*client).ps.saberEntityNum < ENTITYNUM_NONE && (*client).ps.saberEntityNum > 0 {
                //player is 0
                //
                let saber_ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*client).ps.saberEntityNum as usize);
                if !saber_ent.is_null() && (*saber_ent).s.pos.trType == TR_LINEAR {
                    //fell to the ground and we're trying to pull it back
                    using_force = QTRUE;
                }
            }
        }
        if (*client).ps.fd.forcePowersActive == 0
            || (*client).ps.fd.forcePowersActive == (1 << FP_DRAIN)
        {
            //when not using the force, regenerate at 1 point per half second
            if (*client).ps.saberInFlight == 0
                && (*client).ps.fd.forcePowerRegenDebounceTime < (*addr_of!(level)).time
                && ((*client).ps.weapon != WP_SABER
                    || BG_SaberInSpecial((*client).ps.saberMove) == QFALSE)
            {
                if (*addr_of!(g_gametype)).integer != GT_HOLOCRON
                    || (*addr_of!(g_MaxHolocronCarry)).value != 0.0
                {
                    //if (!g_trueJedi.integer || self->client->ps.weapon == WP_SABER)
                    //let non-jedi force regen since we're doing a more strict jedi/non-jedi thing... this gives dark jedi something to drain
                    {
                        if (*client).ps.powerups[PW_FORCE_BOON as usize] != 0 {
                            WP_ForcePowerRegenerate(self_, 6);
                        } else if (*client).ps.isJediMaster != QFALSE
                            && (*addr_of!(g_gametype)).integer == GT_JEDIMASTER
                        {
                            WP_ForcePowerRegenerate(self_, 4); //jedi master regenerates 4 times as fast
                        } else {
                            WP_ForcePowerRegenerate(self_, 0);
                        }
                    }
                    /*
                    else if (g_trueJedi.integer && self->client->ps.weapon != WP_SABER)
                    {
                        self->client->ps.fd.forcePower = 0;
                    }
                    */
                } else {
                    //regenerate based on the number of holocrons carried
                    let mut holoregen: c_int = 0;
                    let mut holo: c_int = 0;
                    while (holo as usize) < NUM_FORCE_POWERS {
                        if (*client).ps.holocronsCarried[holo as usize] != 0.0 {
                            holoregen += 1;
                        }
                        holo += 1;
                    }

                    WP_ForcePowerRegenerate(self_, holoregen);
                }

                if (*addr_of!(g_gametype)).integer == GT_SIEGE {
                    if (*client).holdingObjectiveItem != 0
                        && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .add((*client).holdingObjectiveItem as usize))
                        .inuse
                            != QFALSE
                        && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .add((*client).holdingObjectiveItem as usize))
                        .genericValue15
                            != 0
                    {
                        //1 point per 7 seconds.. super slow
                        (*client).ps.fd.forcePowerRegenDebounceTime = (*addr_of!(level)).time + 7000;
                    } else if (*client).siegeClass != -1
                        && (*addr_of!(bgSiegeClasses))[(*client).siegeClass as usize].classflags
                            & (1 << CFL_FASTFORCEREGEN)
                            != 0
                    {
                        //if this is siege and our player class has the fast force regen ability, then recharge with 1/5th the usual delay
                        (*client).ps.fd.forcePowerRegenDebounceTime = (*addr_of!(level)).time
                            + ((*addr_of!(g_forceRegenTime)).integer as f32 * 0.2) as c_int;
                    } else {
                        (*client).ps.fd.forcePowerRegenDebounceTime =
                            (*addr_of!(level)).time + (*addr_of!(g_forceRegenTime)).integer;
                    }
                } else {
                    if (*addr_of!(g_gametype)).integer == GT_POWERDUEL
                        && (*client).sess.duelTeam == DUELTEAM_LONE
                    {
                        if (*addr_of!(g_duel_fraglimit)).integer != 0 {
                            (*client).ps.fd.forcePowerRegenDebounceTime = (*addr_of!(level)).time
                                + ((*addr_of!(g_forceRegenTime)).integer as f32
                                    * (0.6
                                        + (0.3 * (*client).sess.wins as f32
                                            / (*addr_of!(g_duel_fraglimit)).integer as f32)))
                                    as c_int;
                        } else {
                            (*client).ps.fd.forcePowerRegenDebounceTime = (*addr_of!(level)).time
                                + ((*addr_of!(g_forceRegenTime)).integer as f32 * 0.7) as c_int;
                        }
                    } else {
                        (*client).ps.fd.forcePowerRegenDebounceTime =
                            (*addr_of!(level)).time + (*addr_of!(g_forceRegenTime)).integer;
                    }
                }
            }
        }
    } // powersetcheck:

    if prepower != 0 && (*client).ps.fd.forcePower < prepower {
        let mut dif: c_int = (prepower - (*client).ps.fd.forcePower) / 2;
        if dif < 1 {
            dif = 1;
        }

        (*client).ps.fd.forcePower = prepower - dif;
    }

    // `using_force` is computed faithfully but unused after the powersetcheck block in this port
    // (the C function only consumes the per-branch assignments through fall-through control flow).
    let _ = using_force;
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    #[test]
    fn WP_HasForcePowers_matches_oracle() {
        // NULL ps must return false.
        unsafe {
            assert_eq!(
                WP_HasForcePowers(core::ptr::null()) as c_int,
                oracle::jka_WP_HasForcePowers(0, core::ptr::null())
            );
        }

        // Exercise: empty, every single power raised in isolation (covers the FP_LEVITATION
        // special case at index 1), and a mix of levels.
        let mut cases: Vec<[c_int; NUM_FORCE_POWERS]> = Vec::new();
        cases.push([0; NUM_FORCE_POWERS]);
        for slot in 0..NUM_FORCE_POWERS {
            for lvl in 0..=3 {
                let mut a = [0; NUM_FORCE_POWERS];
                a[slot] = lvl;
                cases.push(a);
            }
        }
        // a couple of full mixes
        let mut mix1 = [0; NUM_FORCE_POWERS];
        for (k, v) in mix1.iter_mut().enumerate() {
            *v = (k % 4) as c_int;
        }
        cases.push(mix1);
        cases.push([1; NUM_FORCE_POWERS]); // all at level 1 (only levitation excluded)

        for levels in &cases {
            // SAFETY: playerState_t is plain repr(C) data; zeroed is a valid all-fields-0 state.
            let mut ps: playerState_t = unsafe { core::mem::zeroed() };
            ps.fd.forcePowerLevel = *levels;
            unsafe {
                let r = WP_HasForcePowers(&ps) as c_int;
                let o = oracle::jka_WP_HasForcePowers(1, levels.as_ptr());
                assert_eq!(r, o, "WP_HasForcePowers levels={levels:?}");
            }
        }
    }

    #[test]
    fn G_IsMindTricked_matches_oracle() {
        // NULL fd must return false (for any client).
        unsafe {
            assert_eq!(
                G_IsMindTricked(core::ptr::null(), 5) as c_int,
                oracle::jka_G_IsMindTricked(0, 0, 0, 0, 0, 5)
            );
        }

        let index_sets: [[c_int; 4]; 5] = [
            [0, 0, 0, 0],
            [-1, -1, -1, -1],
            [0x5555, 0xAAAA, 0x0001, 0x8000],
            [1 << 3, 1 << 7, 1 << 0, 1 << 15],
            [0xFFFF, 0, 0xFFFF, 0],
        ];
        for set in &index_sets {
            // SAFETY: forcedata_t is plain repr(C) data; zeroed is a valid all-fields-0 state.
            let mut fd: forcedata_t = unsafe { core::mem::zeroed() };
            fd.forceMindtrickTargetIndex = set[0];
            fd.forceMindtrickTargetIndex2 = set[1];
            fd.forceMindtrickTargetIndex3 = set[2];
            fd.forceMindtrickTargetIndex4 = set[3];
            for client in 0..64 {
                unsafe {
                    let r = G_IsMindTricked(&fd, client) as c_int;
                    let o = oracle::jka_G_IsMindTricked(1, set[0], set[1], set[2], set[3], client);
                    assert_eq!(r, o, "G_IsMindTricked set={set:?} client={client}");
                }
            }
        }
    }

    #[test]
    fn WP_AddToClientBitflags_matches_oracle() {
        // NULL ent must be a no-op (the oracle's has_ent=0 path leaves its scalars untouched).
        unsafe {
            let (mut o1, mut o2, mut o3, mut o4) = (0, 0, 0, 0);
            oracle::jka_WP_AddToClientBitflags(0, &mut o1, &mut o2, &mut o3, &mut o4, 5);
            assert_eq!((o1, o2, o3, o4), (0, 0, 0, 0));
            WP_AddToClientBitflags(core::ptr::null_mut(), 5);
        }

        // Pre-load seeds covering the four buckets, then add every client slot 0..63 and compare
        // the four resulting trickedentindex* words against the oracle.
        let seeds: [[c_int; 4]; 4] = [
            [0, 0, 0, 0],
            [0x0001, 0x8000, 0x00FF, 0xFF00],
            [-1, -1, -1, -1],
            [0x5555, 0xAAAA, 0x1234, 0x4321],
        ];
        for seed in &seeds {
            for ent_num in 0..64 {
                // SAFETY: gentity_t is plain repr(C) data; zeroed is a valid all-fields-0 state.
                let mut ent: gentity_t = unsafe { core::mem::zeroed() };
                ent.s.trickedentindex = seed[0];
                ent.s.trickedentindex2 = seed[1];
                ent.s.trickedentindex3 = seed[2];
                ent.s.trickedentindex4 = seed[3];

                let (mut o1, mut o2, mut o3, mut o4) = (seed[0], seed[1], seed[2], seed[3]);
                unsafe {
                    WP_AddToClientBitflags(&mut ent, ent_num);
                    oracle::jka_WP_AddToClientBitflags(
                        1, &mut o1, &mut o2, &mut o3, &mut o4, ent_num,
                    );
                }
                assert_eq!(
                    (
                        ent.s.trickedentindex,
                        ent.s.trickedentindex2,
                        ent.s.trickedentindex3,
                        ent.s.trickedentindex4,
                    ),
                    (o1, o2, o3, o4),
                    "WP_AddToClientBitflags seed={seed:?} ent_num={ent_num}"
                );
            }
        }
    }

    #[test]
    fn WP_AddAsMindtricked_matches_oracle() {
        // NULL fd must be a no-op (the oracle's has_fd=0 path leaves its scalars untouched).
        unsafe {
            let (mut o1, mut o2, mut o3, mut o4) = (0, 0, 0, 0);
            oracle::jka_WP_AddAsMindtricked(0, &mut o1, &mut o2, &mut o3, &mut o4, 5);
            assert_eq!((o1, o2, o3, o4), (0, 0, 0, 0));
            WP_AddAsMindtricked(core::ptr::null_mut(), 5);
        }

        // Pre-load seeds covering the four buckets, then add every client slot 0..63 and compare
        // the four resulting forceMindtrickTargetIndex* words against the oracle.
        let seeds: [[c_int; 4]; 4] = [
            [0, 0, 0, 0],
            [0x0001, 0x8000, 0x00FF, 0xFF00],
            [-1, -1, -1, -1],
            [0x5555, 0xAAAA, 0x1234, 0x4321],
        ];
        for seed in &seeds {
            for ent_num in 0..64 {
                // SAFETY: forcedata_t is plain repr(C) data; zeroed is a valid all-fields-0 state.
                let mut fd: forcedata_t = unsafe { core::mem::zeroed() };
                fd.forceMindtrickTargetIndex = seed[0];
                fd.forceMindtrickTargetIndex2 = seed[1];
                fd.forceMindtrickTargetIndex3 = seed[2];
                fd.forceMindtrickTargetIndex4 = seed[3];

                let (mut o1, mut o2, mut o3, mut o4) = (seed[0], seed[1], seed[2], seed[3]);
                unsafe {
                    WP_AddAsMindtricked(&mut fd, ent_num);
                    oracle::jka_WP_AddAsMindtricked(1, &mut o1, &mut o2, &mut o3, &mut o4, ent_num);
                }
                assert_eq!(
                    (
                        fd.forceMindtrickTargetIndex,
                        fd.forceMindtrickTargetIndex2,
                        fd.forceMindtrickTargetIndex3,
                        fd.forceMindtrickTargetIndex4,
                    ),
                    (o1, o2, o3, o4),
                    "WP_AddAsMindtricked seed={seed:?} ent_num={ent_num}"
                );
            }
        }
    }

    #[test]
    fn RemoveTrickedEnt_matches_oracle() {
        // NULL fd must be a no-op (the oracle's has_fd=0 path leaves its scalars untouched).
        unsafe {
            let (mut o1, mut o2, mut o3, mut o4) = (0, 0, 0, 0);
            oracle::jka_RemoveTrickedEnt(0, &mut o1, &mut o2, &mut o3, &mut o4, 5);
            assert_eq!((o1, o2, o3, o4), (0, 0, 0, 0));
            RemoveTrickedEnt(core::ptr::null_mut(), 5);
        }

        // Pre-load seeds (including all-bits-set so every clear is observable), then remove every
        // client slot 0..63 and compare the four resulting forceMindtrickTargetIndex* words
        // against the oracle.
        let seeds: [[c_int; 4]; 4] = [
            [-1, -1, -1, -1],
            [0x0001, 0x8000, 0x00FF, 0xFF00],
            [0, 0, 0, 0],
            [0x5555, 0xAAAA, 0x1234, 0x4321],
        ];
        for seed in &seeds {
            for client in 0..64 {
                // SAFETY: forcedata_t is plain repr(C) data; zeroed is a valid all-fields-0 state.
                let mut fd: forcedata_t = unsafe { core::mem::zeroed() };
                fd.forceMindtrickTargetIndex = seed[0];
                fd.forceMindtrickTargetIndex2 = seed[1];
                fd.forceMindtrickTargetIndex3 = seed[2];
                fd.forceMindtrickTargetIndex4 = seed[3];

                let (mut o1, mut o2, mut o3, mut o4) = (seed[0], seed[1], seed[2], seed[3]);
                unsafe {
                    RemoveTrickedEnt(&mut fd, client);
                    oracle::jka_RemoveTrickedEnt(1, &mut o1, &mut o2, &mut o3, &mut o4, client);
                }
                assert_eq!(
                    (
                        fd.forceMindtrickTargetIndex,
                        fd.forceMindtrickTargetIndex2,
                        fd.forceMindtrickTargetIndex3,
                        fd.forceMindtrickTargetIndex4,
                    ),
                    (o1, o2, o3, o4),
                    "RemoveTrickedEnt seed={seed:?} client={client}"
                );
            }
        }
    }

    #[test]
    fn G_InGetUpAnim_matches_oracle() {
        // The full getup-anim set the predicate matches, plus representative non-members
        // (including the gaps 1229/1230/1238 between the named runs, and out-of-range values).
        let getup: [c_int; 20] = [
            BOTH_GETUP1 as c_int,
            BOTH_GETUP2 as c_int,
            BOTH_GETUP3 as c_int,
            BOTH_GETUP4 as c_int,
            BOTH_GETUP5 as c_int,
            BOTH_FORCE_GETUP_F1 as c_int,
            BOTH_FORCE_GETUP_F2 as c_int,
            BOTH_FORCE_GETUP_B1 as c_int,
            BOTH_FORCE_GETUP_B2 as c_int,
            BOTH_FORCE_GETUP_B3 as c_int,
            BOTH_FORCE_GETUP_B4 as c_int,
            BOTH_FORCE_GETUP_B5 as c_int,
            BOTH_GETUP_BROLL_B as c_int,
            BOTH_GETUP_BROLL_F as c_int,
            BOTH_GETUP_BROLL_L as c_int,
            BOTH_GETUP_BROLL_R as c_int,
            BOTH_GETUP_FROLL_B as c_int,
            BOTH_GETUP_FROLL_F as c_int,
            BOTH_GETUP_FROLL_L as c_int,
            BOTH_GETUP_FROLL_R as c_int,
        ];
        let non_members: [c_int; 7] = [0, -1, 1223, 1229, 1230, 1238, 1247];

        // Build the candidate anim pool: every member + every non-member.
        let mut pool: Vec<c_int> = getup.to_vec();
        pool.extend_from_slice(&non_members);

        // Exercise every (legs, torso) pair so both switches and their interaction are covered.
        for &legs in &pool {
            for &torso in &pool {
                // SAFETY: playerState_t is plain repr(C) data; zeroed is a valid all-fields-0 state.
                let mut ps: playerState_t = unsafe { core::mem::zeroed() };
                ps.legsAnim = legs;
                ps.torsoAnim = torso;
                unsafe {
                    let r = G_InGetUpAnim(&ps) as c_int;
                    let o = oracle::jka_G_InGetUpAnim(legs, torso);
                    assert_eq!(r, o, "G_InGetUpAnim legs={legs} torso={torso}");
                }
            }
        }
    }
}
