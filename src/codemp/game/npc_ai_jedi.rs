//! Slice of `NPC_AI_Jedi.c` — the Jedi/Boba-Fett NPC behavior state. Opened
//! bottom-up at the leaf seam: most of this large file funnels into the unported
//! Ghoul2 / saber / vehicle / NPC-AI core, so only the pure entity-state leaves
//! land here for now.
//!
//! Ported here so far: `G_StartMatrixEffect` (NPC_AI_Jedi.c:16),
//! `NPC_ShadowTrooper_Precache` (:103), `Boba_Precache` (:182),
//! `Boba_ChangeWeapon` (:193), `WP_ResistForcePush` (:203),
//! `NPC_Jedi_RateNewEnemy` (:914), `Jedi_ClearTimers` (:110),
//! `Jedi_PlayBlockedPushSound` (:137),
//! `Jedi_PlayDeflectSound` (:150), `NPC_Jedi_PlayConfusionSound` (:163),
//! `Boba_StopKnockdown` (:272), `Boba_FlyStart` (:345),
//! `Boba_FlyStop` (:367), `Boba_Flying` (:386), `Jedi_Cloak` (:799),
//! `Jedi_Decloak` (:818), `Jedi_CheckCloak` (:835), `Jedi_Aggression` (:863),
//! `Jedi_AggressionErosion` (:900), `Jedi_Rage` (:952), `Jedi_RageStop` (:966),
//! `Jedi_BattleTaunt` (:980), `Jedi_HoldPosition` (:1200),
//! `Jedi_ClearPathToSpot` (:1020), `NPC_MoveDirClear` (:1079),
//! `Jedi_Move` (:1219), `Jedi_Hunt` (:1253), `Jedi_Retreat` (:1300),
//! `Jedi_Advance` (:1312), `Jedi_AdjustSaberAnimLevel` (:1327),
//! `Jedi_CheckDecreaseSaberAnimLevel` (:1396),
//! `Jedi_FindEnemyInCone` (:3634), `Jedi_DebounceDirectionChanges` (:3824),
//! `Jedi_Strafe` (:1876), `Jedi_SetEnemyInfo` (:3711),
//! `Jedi_CheckFlipEvasions` (:1969),
//! `Jedi_TimersApply` (:3955), `Jedi_ReCalcParryTime` (:2253),
//! `Jedi_QuickReactions` (:2391),
//! `Jedi_SaberBusy` (:2403), `Jedi_FaceEnemy` (:3747),
//! `Jedi_CombatIdle` (:4223), `Jedi_Jump` (:4421), `Jedi_TryJump` (:4667),
//! `Jedi_Jumping` (:4815), `Jedi_CheckEnemyMovement` (:4865),
//! `Jedi_CheckJumps` (:4986), `NPC_Jedi_Pain` (:5306),
//! `Jedi_CheckDanger` (:5394), `Jedi_CheckAmbushPlayer` (:5413),
//! `Jedi_Ambush` (:5495), `Jedi_WaitingAmbush` (:5509), `Jedi_Patrol` (:5523),
//! `Jedi_CanPullBackSaber` (:5678).
//!
//! File-static ported here: `jediSpeechDebounceTime` (:94).

#![allow(non_snake_case)] // C function names (`Boba_FlyStop`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define`/enum constants kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::ptr::{addr_of, addr_of_mut, null_mut};

// Native libc `atof` — the C reads the `timescale` cvar string buffer and atof()s it
// in `WP_ResistForcePush`. Bound locally as `extern "C"`, same idiom as npc_utils.rs /
// g_ICARUScb.rs.
extern "C" {
    fn atof(s: *const c_char) -> f64;
}

use crate::codemp::game::anims::{
    animNumber_t, BOTH_A2_STABBACK1, BOTH_ARIAL_LEFT, BOTH_ARIAL_RIGHT, BOTH_ATTACK_BACK,
    BOTH_BUTTERFLY_LEFT, BOTH_BUTTERFLY_RIGHT, BOTH_CARTWHEEL_LEFT, BOTH_CARTWHEEL_RIGHT,
    BOTH_CEILING_CLING, BOTH_CEILING_DROP, BOTH_CROUCHATTACKBACK1, BOTH_DODGE_BL, BOTH_DODGE_BR,
    BOTH_DODGE_FL, BOTH_DODGE_FR, BOTH_DODGE_L, BOTH_DODGE_R, BOTH_FLIP_F, BOTH_FORCEJUMP1,
    BOTH_FORCELIGHTNING_HOLD, BOTH_JUMPFLIPSLASHDOWN1, BOTH_JUMPFLIPSTABDOWN, BOTH_RESISTPUSH,
    BOTH_WALL_FLIP_BACK1, BOTH_WALL_FLIP_LEFT, BOTH_WALL_FLIP_RIGHT, BOTH_WALL_RUN_LEFT,
    BOTH_WALL_RUN_LEFT_FLIP, BOTH_WALL_RUN_RIGHT, BOTH_WALL_RUN_RIGHT_FLIP,
};
use crate::codemp::game::b_local_h::MIN_ROCKET_DIST_SQUARED;
use crate::codemp::game::b_public_h::{
    BS_DEFAULT, BS_HUNT_AND_KILL, NPCAI_BLOCKED, NPCAI_CUSTOM_GRAVITY, RANK_CAPTAIN, RANK_CIVILIAN, RANK_COMMANDER, RANK_CREWMAN, RANK_ENSIGN,
    RANK_LT, RANK_LT_COMM, RANK_LT_JG, SCF_ALT_FIRE, SCF_CHASE_ENEMIES, SCF_DONT_FIRE,
    SCF_FIRE_WEAPON,
    SCF_LOOK_FOR_ENEMIES, SCF_NO_ACROBATICS, SPOT_HEAD,
};
use crate::codemp::game::bg_public::{
    BG_GiveMeVectorFromMatrix, EF2_FLYING, EV_CONFUSE1, EV_CONFUSE3, EV_DEFLECT1, EV_DEFLECT3,
    EV_COMBAT1, EV_COMBAT3, EV_GENERAL_SOUND, EV_GLOAT1, EV_JCHASE1, EV_JCHASE3, EV_JLOST1,
    EV_JLOST3, EV_JUMP, EV_VICTORY1, EV_VICTORY3, WEAPON_FIRING,
    EV_ANGER1, EV_ANGER3, EV_GLOAT3, EV_JDETECTED1, EV_JDETECTED3, EV_PUSHED1, EV_PUSHED3,
    EV_PUSHFAIL, EV_TAUNT1, EV_TAUNT3, ET_PLAYER, HANDEXTEND_JEDITAUNT, HANDEXTEND_NONE,
    JUMP_VELOCITY, MASK_SHOT, MOD_LAVA, PMF_DUCKED, PMF_TIME_KNOCKBACK, PW_CLOAKED, PW_DISINT_4,
    PW_PULL, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE, SETANIM_LEGS, SETANIM_TORSO,
    STEPSIZE, TEAM_NUM_TEAMS,
};
use crate::codemp::game::bg_misc::{BG_EvaluateTrajectory, BG_FindItemForAmmo};
use crate::codemp::game::bg_panimate::{
    BG_AnimLength, BG_CrouchAnim, BG_FlippingAnim, BG_InRoll, BG_SaberInAttack,
    BG_SaberInSpecialAttack, BG_SpinningSaberAnim, PM_InKnockDown, PM_SaberInKnockaway,
    PM_SaberInParry, PM_SaberInStart,
};
use crate::codemp::game::bg_pmove::{forceJumpStrength, BG_SabersOff, PM_RollingAnim};
use crate::codemp::game::bg_saber::{bg_parryDebounce, PM_SaberInBrokenParry};
use crate::codemp::game::bg_weapons_h::{
    AMMO_FORCE, WP_BLASTER, WP_BOWCASTER, WP_BRYAR_PISTOL, WP_DEMP2, WP_DET_PACK, WP_DISRUPTOR,
    WP_EMPLACED_GUN, WP_FLECHETTE, WP_NONE, WP_REPEATER, WP_ROCKET_LAUNCHER, WP_SABER,
    WP_STUN_BATON, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use crate::codemp::game::g_combat::{gPainPoint, G_Damage};
use crate::codemp::game::ai_wpnav::G_TestLine;
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::{
    gentity_t, AEL_DANGER, AEL_MINOR, DAMAGE_IGNORE_TEAM, DAMAGE_NO_ARMOR, DAMAGE_NO_KNOCKBACK,
    FL_NOTARGET, FRAMETIME,
};
use crate::codemp::game::g_main::{
    d_JediAI, d_slowmodeath, g_entities, g_gravity, g_saberRealisticCombat, g_spskill, level,
    Com_Printf,
};
use crate::codemp::game::g_public_h::{Q3_INFINITE, SVF_GLASS_BRUSH};
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Get, TIMER_Set, TIMER_Start};
use crate::codemp::game::g_nav::{navInfo_t, NAV_CheckAhead, NIF_COLLISION, NIF_MACRO_NAV};
use crate::codemp::game::g_utils::{
    G_AddEvent, G_EffectIndex, G_FreeEntity, G_PlayEffectID, G_SetOrigin, G_Sound, G_SoundIndex,
    G_SoundOnEnt, G_Spawn, GetAnglesForDirection, ShortestLineSegBewteen2LineSegs,
};
use crate::codemp::game::g_weapon::WP_SpeedOfMissileForWeapon;
use crate::codemp::game::npc::{ucmd, NPC_SetAnim, NPCInfo, NPC};
use crate::codemp::game::npc_combat::{
    G_AddVoiceEvent, G_ClearEnemy, G_SetEnemy, NPC_ChangeWeapon, NPC_CheckEnemy, NPC_ShotEntity,
    WeaponThink,
};
use crate::codemp::game::npc_ai_sniper::NPC_BSSniper_Default;
use crate::codemp::game::npc_ai_stormtrooper::NPC_BSST_Patrol;
use crate::codemp::game::npc_behavior::NPC_BSFollowLeader;
use crate::codemp::game::npc_move::{G_UcmdMoveForDir, NAV_GetLastMove, NPC_MoveToGoal};
use crate::codemp::game::npc_reactions::NPC_Pain;
use crate::codemp::game::npc_senses::{InFOV, InFront, NPC_CheckAlertEvents};
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, NPC_ClearLookTarget, NPC_ClearLOS4, NPC_FaceEnemy, NPC_FaceEntity,
    NPC_SetLookTarget, NPC_SomeoneLookingAtMe, NPC_UpdateAngles, NPC_ValidEnemy,
};
use crate::codemp::game::q_math::{
    flrand, vec3_origin, vectoangles, AngleNormalize360, AngleVectors, Distance,
    DistanceHorizontalSquared, DistanceSquared, DotProduct, VectorClear, VectorCompare, VectorCopy,
    VectorLength, VectorLengthSquared, VectorMA, VectorNormalize, VectorNormalize2, VectorScale,
    VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{Q_stricmp};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, trace_t, trajectory_t, usercmd_t, vec3_t, BLOCKED_ATK_BOUNCE, BLOCKED_LOWER_LEFT,
    BLOCKED_LOWER_RIGHT,
    BLOCKED_NONE, BLOCKED_PARRY_BROKEN, BLOCKED_TOP, BLOCKED_UPPER_LEFT, BLOCKED_UPPER_RIGHT,
    BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_FORCEGRIP, SOLID_BMODEL, TR_GRAVITY, TR_STATIONARY,
    BUTTON_FORCE_DRAIN, BUTTON_FORCE_LIGHTNING, BUTTON_WALKING, CHAN_BODY, CHAN_ITEM, CHAN_WEAPON,
    ENTITYNUM_NONE, ENTITYNUM_WORLD, FORCE_LEVEL_1, FORCE_LEVEL_2, FORCE_LEVEL_3, FORCE_LEVEL_4,
    FORCE_LEVEL_5, FP_ABSORB, FP_DRAIN, FP_GRIP, FP_HEAL, FP_LEVITATION, FP_LIGHTNING, FP_PROTECT,
    FP_PULL, FP_PUSH, FP_RAGE, FP_SABERTHROW, FP_SABER_DEFENSE,
    FP_SABER_OFFENSE, FP_SPEED, MAX_CLIENTS, MAX_GENTITIES, NEGATIVE_Y, ORIGIN, PITCH, ROLL,
    SFL_NO_CARTWHEELS, SFL_NO_WALL_FLIPS, SFL_NO_WALL_RUNS, YAW,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_BOTCLIP, CONTENTS_MONSTERCLIP, CONTENTS_SOLID,
};
use crate::codemp::game::teams_h::{
    CLASS_BOBAFETT, CLASS_DESANN, CLASS_JEDI, CLASS_LUKE, CLASS_REBORN, CLASS_SHADOWTROOPER,
    CLASS_TAVION, NPCTEAM_ENEMY, NPCTEAM_PLAYER,
};
use crate::codemp::game::w_force::{
    ForceAbsorb, ForceHeal, ForceJump, ForceLightning, ForceProtect, ForceRage, ForceSpeed,
    ForceThrow, WP_ForcePowerAvailable, WP_ForcePowerStop, WP_GetVelocityForForceJump,
};
use crate::codemp::game::w_saber::{WP_ActivateSaber, WP_DeactivateSaber, WP_MissileBlockForBlock};
use crate::codemp::game::w_saber_h::{
    evasionType_t, EVASION_CARTWHEEL, EVASION_DODGE, EVASION_DUCK, EVASION_DUCK_PARRY,
    EVASION_FJUMP, EVASION_JUMP, EVASION_JUMP_PARRY, EVASION_NONE, EVASION_OTHER, EVASION_PARRY,
    JSF_AMBUSH, SEF_BLOCKED, SEF_DEFLECTED, SEF_HITENEMY, SEF_HITOBJECT, SEF_HITWALL, SEF_INWATER,
    SEF_LOCK_WON, SEF_PARRIED, SES_LEAVING, SES_RETURNING,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `static int jediSpeechDebounceTime[TEAM_NUM_TEAMS]` (NPC_AI_Jedi.c:94) — used
/// to stop several Jedi from speaking all at once. File-scoped process-global.
#[allow(dead_code)] // first written by Jedi_BattleTaunt; other readers land later
static mut jediSpeechDebounceTime: [c_int; TEAM_NUM_TEAMS as usize] =
    [0; TEAM_NUM_TEAMS as usize];

/// `qboolean Boba_StopKnockdown( gentity_t *self, gentity_t *pusher, vec3_t pushDir,
/// qboolean forceKnockdown )` (NPC_AI_Jedi.c:272; `forceKnockdown` defaults to qfalse
/// in the C declaration). Boba-Fett-only: he can't be knocked down while flying, and
/// otherwise (2-in-3) flips or rolls with the push, (sometimes) resists it, or finally
/// falls down. Returns whether the knockdown was stopped. No oracle (entity-state +
/// timers + force jump).
///
/// # Safety
/// `self`/`pusher` must be valid; `self->client` must be valid; `pushDir` is a vec3.
pub unsafe fn Boba_StopKnockdown(
    self_: *mut gentity_t,
    pusher: *mut gentity_t,
    pushDir: &vec3_t,
    forceKnockdown: qboolean,
) -> qboolean {
    let mut pDir: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut ang: vec3_t = [0.0; 3];
    let fDot: f32;
    let rDot: f32;
    let strafeTime: c_int;

    if (*(*self_).client).NPC_class != CLASS_BOBAFETT {
        return QFALSE;
    }

    if (*(*self_).client).ps.eFlags2 & EF2_FLYING != 0
    //moveType == MT_FLYSWIM )
    {
        //can't knock me down when I'm flying
        return QTRUE;
    }

    VectorSet(&mut ang, 0.0, (*self_).r.currentAngles[YAW], 0.0);
    strafeTime = Q_irand(1000, 2000);

    AngleVectors(&ang, Some(&mut fwd), Some(&mut right), None);
    VectorNormalize2(pushDir, &mut pDir);
    fDot = DotProduct(&pDir, &fwd);
    rDot = DotProduct(&pDir, &right);

    if Q_irand(0, 2) != 0 {
        //flip or roll with it
        // C leaves `tempCmd` uninitialized on the stack; ForceJump reads only the
        // move fields, so zero-init here (faithful net effect).
        let mut tempCmd: usercmd_t = core::mem::zeroed();
        if fDot >= 0.4 {
            tempCmd.forwardmove = 127;
            TIMER_Set(self_, c"moveforward".as_ptr(), strafeTime);
        } else if fDot <= -0.4 {
            tempCmd.forwardmove = -127;
            TIMER_Set(self_, c"moveback".as_ptr(), strafeTime);
        } else if rDot > 0.0 {
            tempCmd.rightmove = 127;
            TIMER_Set(self_, c"strafeRight".as_ptr(), strafeTime);
            TIMER_Set(self_, c"strafeLeft".as_ptr(), -1);
        } else {
            tempCmd.rightmove = -127;
            TIMER_Set(self_, c"strafeLeft".as_ptr(), strafeTime);
            TIMER_Set(self_, c"strafeRight".as_ptr(), -1);
        }
        G_AddEvent(self_, EV_JUMP, 0);
        if Q_irand(0, 1) == 0 {
            //flip
            (*(*self_).client).ps.fd.forceJumpCharge = 280.0; //FIXME: calc this intelligently?
            ForceJump(self_, &mut tempCmd);
        } else {
            //roll
            TIMER_Set(self_, c"duck".as_ptr(), strafeTime);
        }
        (*self_).painDebounceTime = 0; //so we do something
    } else if Q_irand(0, 1) == 0 && forceKnockdown != QFALSE {
        //resist
        WP_ResistForcePush(self_, pusher, QTRUE);
    } else {
        //fall down
        return QFALSE;
    }

    QTRUE
}

/// `void Boba_FlyStart( gentity_t *self )` (NPC_AI_Jedi.c:345). Switches Boba to
/// jetpack flight (seeker-style AI) once the jet-recharge timer is up: zeroes gravity,
/// sets the custom-gravity AI flag + `EF2_FLYING`, arms the jetpack timer, plays the
/// take-off + loop sounds and (for NPCs) gives infinite SEEKER ammo. No oracle
/// (entity-state + timers + sound).
///
/// # Safety
/// `self`, `self->client`, and (where dereferenced) `self->NPC` must be valid.
pub unsafe fn Boba_FlyStart(self_: *mut gentity_t) {
    //switch to seeker AI for a while
    if TIMER_Done(self_, c"jetRecharge".as_ptr()) != QFALSE {
        (*(*self_).client).ps.gravity = 0;
        if !(*self_).NPC.is_null() {
            (*(*self_).NPC).aiFlags |= NPCAI_CUSTOM_GRAVITY;
        }
        (*(*self_).client).ps.eFlags2 |= EF2_FLYING; //moveType = MT_FLYSWIM;
        (*(*self_).client).jetPackTime = (*addr_of!(level)).time + Q_irand(3000, 10000);
        //take-off sound
        G_SoundOnEnt(self_, CHAN_ITEM, "sound/boba/jeton.wav");
        //jet loop sound
        (*self_).s.loopSound = G_SoundIndex("sound/boba/jethover.wav");
        if !(*self_).NPC.is_null() {
            (*self_).count = Q3_INFINITE as c_int; // SEEKER shot ammo count
        }
    }
}

/// `void Boba_FlyStop( gentity_t *self )` (NPC_AI_Jedi.c:367). Tears down the
/// jetpack flight state: restores global gravity, clears the custom-gravity AI
/// flag and the `EF2_FLYING` eFlag, zeroes the jetpack timer and the jet loop
/// sound, then (for NPCs) clears the SEEKER ammo count and arms the jet-recharge
/// and jump-chase debounce timers. No oracle (entity-state + cvar + timers).
///
/// # Safety
/// `self`, `self->client`, and (where dereferenced) `self->NPC` must be valid.
pub unsafe fn Boba_FlyStop(self_: *mut gentity_t) {
    (*(*self_).client).ps.gravity = (*addr_of!(g_gravity)).value as c_int;
    if !(*self_).NPC.is_null() {
        (*(*self_).NPC).aiFlags &= !NPCAI_CUSTOM_GRAVITY;
    }
    (*(*self_).client).ps.eFlags2 &= !EF2_FLYING;
    (*(*self_).client).jetPackTime = 0;
    //stop jet loop sound
    (*self_).s.loopSound = 0;
    if !(*self_).NPC.is_null() {
        (*self_).count = 0; // SEEKER shot ammo count
        TIMER_Set(self_, c"jetRecharge".as_ptr(), Q_irand(1000, 5000));
        TIMER_Set(self_, c"jumpChaseDebounce".as_ptr(), Q_irand(500, 2000));
    }
}

/// `qboolean Boba_Flying( gentity_t *self )` (NPC_AI_Jedi.c:386). Returns whether
/// the entity is currently flying (the `EF2_FLYING` eFlag). No oracle (pure
/// entity-state).
///
/// # Safety
/// `self` and `self->client` must be valid.
pub unsafe fn Boba_Flying(self_: *mut gentity_t) -> qboolean {
    //moveType==MT_FLYSWIM));
    if ((*(*self_).client).ps.eFlags2 & EF2_FLYING) != 0 {
        QTRUE
    } else {
        QFALSE
    }
}

/// `void Boba_FireFlameThrower( gentity_t *self )` (NPC_AI_Jedi.c:391). Traces a
/// short jet of fire forward from Boba's left-hand bolt; if it hits a takedamage
/// entity (and not the world), deals `Q_irand(20,30)` `MOD_LAVA` damage with
/// no-armor / no-knockback / ignore-team flags. No oracle (ghoul2 bolt matrix +
/// trace + damage).
///
/// # Safety
/// `self`, `self->client`, `self->ghoul2` must be valid.
pub unsafe fn Boba_FireFlameThrower(self_: *mut gentity_t) {
    let damage: c_int = Q_irand(20, 30);
    let traceEnt: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = mdxaBone_t { matrix: [[0.0; 4]; 3] };
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let traceMins: vec3_t = [-4.0, -4.0, -4.0];
    let traceMaxs: vec3_t = [4.0, 4.0, 4.0];

    trap::G2API_GetBoltMatrix(
        (*self_).ghoul2,
        0,
        (*(*self_).client).renderInfo.handLBolt,
        &mut boltMatrix,
        &(*self_).r.currentAngles,
        &(*self_).r.currentOrigin,
        (*addr_of!(level)).time,
        null_mut(),
        &(*self_).modelScale,
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut start);
    BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut dir);
    //G_PlayEffect( "boba/fthrw", start, dir );
    VectorMA(&start, 128.0, &dir, &mut end);

    let mut tr: trace_t = trap::Trace(
        &start,
        &traceMins,
        &traceMaxs,
        &end,
        (*self_).s.number,
        MASK_SHOT,
    );

    traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);
    if (tr.entityNum as c_int) < ENTITYNUM_WORLD && (*traceEnt).takedamage != QFALSE {
        G_Damage(
            traceEnt,
            self_,
            self_,
            &mut dir,
            &mut tr.endpos,
            damage,
            DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK | /*DAMAGE_NO_HIT_LOC|*/ DAMAGE_IGNORE_TEAM,
            MOD_LAVA,
        );
        //rwwFIXMEFIXME: add DAMAGE_NO_HIT_LOC?
    }
}

//extern void SP_fx_explosion_trail( gentity_t *ent );
/// `void Boba_StartFlameThrower( gentity_t *self )` (NPC_AI_Jedi.c:419). Begins a
/// 4-second flamethrower burst: holds the torso anim timer, arms the attack-delay /
/// walking / flameTime timers (for NPCs), plays the combust-fire sound, and spawns
/// the `boba/fthrw` effect at the right-hand bolt. The large commented-out
/// `SP_fx_explosion_trail` block is carried over verbatim. No oracle (timers +
/// ghoul2 bolt matrix + sound + effect).
///
/// # Safety
/// `self`/`self->client` and the `NPC` global (+ its ghoul2/client) must be valid.
pub unsafe fn Boba_StartFlameThrower(self_: *mut gentity_t) {
    let flameTime: c_int = 4000; //Q_irand( 1000, 3000 );
    let mut boltMatrix: mdxaBone_t = mdxaBone_t { matrix: [[0.0; 4]; 3] };
    let mut org: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];

    (*(*self_).client).ps.torsoTimer = flameTime; //+1000;
    if !(*self_).NPC.is_null() {
        TIMER_Set(self_, c"nextAttackDelay".as_ptr(), flameTime);
        TIMER_Set(self_, c"walking".as_ptr(), 0);
    }
    TIMER_Set(self_, c"flameTime".as_ptr(), flameTime);
    /*
    gentity_t *fire = G_Spawn();
    if ( fire != NULL )
    {
        mdxaBone_t	boltMatrix;
        vec3_t		org, dir, ang;
        gi.G2API_GetBoltMatrix( NPC->ghoul2, NPC->playerModel, NPC->handRBolt,
                &boltMatrix, NPC->r.currentAngles, NPC->r.currentOrigin, (cg.time?cg.time:level.time),
                NULL, NPC->s.modelScale );

        gi.G2API_GiveMeVectorFromMatrix( boltMatrix, ORIGIN, org );
        gi.G2API_GiveMeVectorFromMatrix( boltMatrix, NEGATIVE_Y, dir );
        vectoangles( dir, ang );

        VectorCopy( org, fire->s.origin );
        VectorCopy( ang, fire->s.angles );

        fire->targetname = "bobafire";
        SP_fx_explosion_trail( fire );
        fire->damage = 1;
        fire->radius = 10;
        fire->speed = 200;
        fire->fxID = G_EffectIndex( "boba/fthrw" );//"env/exp_fire_trail" );//"env/small_fire"
        fx_explosion_trail_link( fire );
        fx_explosion_trail_use( fire, NPC, NPC );

    }
    */
    G_SoundOnEnt(self_, CHAN_WEAPON, "sound/effects/combustfire.mp3");

    trap::G2API_GetBoltMatrix(
        (*NPC).ghoul2,
        0,
        (*(*NPC).client).renderInfo.handRBolt,
        &mut boltMatrix,
        &(*NPC).r.currentAngles,
        &(*NPC).r.currentOrigin,
        (*addr_of!(level)).time,
        null_mut(),
        &(*NPC).modelScale,
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut org);
    BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut dir);

    G_PlayEffectID(G_EffectIndex("boba/fthrw"), &org, &dir);
}

/// `void Boba_DoFlameThrower( gentity_t *self )` (NPC_AI_Jedi.c:471). Holds the
/// force-lightning torso anim while flaming; (re)starts the flame burst when both
/// the attack-delay and flameTime timers are done, then fires the flame this frame.
/// No oracle (anim + timers).
///
/// # Safety
/// `self` and its client must be valid.
pub unsafe fn Boba_DoFlameThrower(self_: *mut gentity_t) {
    NPC_SetAnim(
        self_,
        SETANIM_TORSO,
        BOTH_FORCELIGHTNING_HOLD,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
    );
    if TIMER_Done(self_, c"nextAttackDelay".as_ptr()) != QFALSE
        && TIMER_Done(self_, c"flameTime".as_ptr()) != QFALSE
    {
        Boba_StartFlameThrower(self_);
    }
    Boba_FireFlameThrower(self_);
}

/// `void Boba_FireDecide( void )` (NPC_AI_Jedi.c:481). Boba's ranged-attack
/// decision logic on the `NPC` global: maybe take off (jetpack), pick a weapon
/// based on the enemy's weapon/health, use the flamethrower up close, and otherwise
/// run the standard LOS/clear-shot checks (incl. firing on the enemy's last-seen
/// position). No oracle (NPC-global AI, traps, timers). The disruptor-sniping branch
/// at the top of the weapon switch is carried over commented as in C.
///
/// # Safety
/// The `NPC`/`NPCInfo`/`ucmd` globals and `NPC->client`/`NPC->enemy` must be valid.
// faithful: C's `faceEnemy` is write-only here (the C fn never acts on it / the dead
// `enemyCS` resets), so allow the unused-assignment + unused-variable lints.
#[allow(unused_assignments, unused_variables)]
pub unsafe fn Boba_FireDecide() {
    let mut enemyLOS: qboolean = QFALSE;
    let mut enemyCS: qboolean = QFALSE;
    let mut enemyInFOV: qboolean = QFALSE;
    //qboolean move = qtrue;
    let mut faceEnemy: qboolean = QFALSE;
    let mut shoot: qboolean = QFALSE;
    let mut hitAlly: qboolean = QFALSE;
    let mut impactPos: vec3_t = [0.0; 3];
    let enemyDist: f32;
    let dot: f32;
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut shootDir: vec3_t = [0.0; 3];

    if (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE
        && (*(*NPC).client).ps.fd.forceJumpZStart != 0.0
        && BG_FlippingAnim((*(*NPC).client).ps.legsAnim) == QFALSE
        && Q_irand(0, 10) == 0
    {
        //take off
        Boba_FlyStart(NPC);
    }

    if (*NPC).enemy.is_null() {
        return;
    }

    /*
    if ( NPC->enemy->enemy != NPC && NPC->health == NPC->client->pers.maxHealth )
    {
        NPCInfo->scriptFlags |= SCF_ALT_FIRE;
        Boba_ChangeWeapon( WP_DISRUPTOR );
    }
    else */
    if (*(*NPC).enemy).s.weapon == WP_SABER {
        (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
        Boba_ChangeWeapon(WP_ROCKET_LAUNCHER);
    } else if (*NPC).health < ((*(*NPC).client).pers.maxHealth as f32 * 0.5f32) as c_int {
        (*NPCInfo).scriptFlags |= SCF_ALT_FIRE;
        Boba_ChangeWeapon(WP_BLASTER);
        (*NPCInfo).burstMin = 3;
        (*NPCInfo).burstMean = 12;
        (*NPCInfo).burstMax = 20;
        (*NPCInfo).burstSpacing = Q_irand(300, 750); //attack debounce
    } else {
        (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
        Boba_ChangeWeapon(WP_BLASTER);
    }

    VectorClear(&mut impactPos);
    enemyDist = DistanceSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin);

    VectorSubtract(
        &(*(*NPC).enemy).r.currentOrigin,
        &(*NPC).r.currentOrigin,
        &mut enemyDir,
    );
    VectorNormalize(&mut enemyDir);
    AngleVectors(&(*(*NPC).client).ps.viewangles, Some(&mut shootDir), None, None);
    dot = DotProduct(&enemyDir, &shootDir);
    if dot > 0.5f32 || (enemyDist * (1.0f32 - dot)) < 10000.0 {
        //enemy is in front of me or they're very close and not behind me
        enemyInFOV = QTRUE;
    }

    if (enemyDist < (128.0 * 128.0) && enemyInFOV != QFALSE)
        || TIMER_Done(NPC, c"flameTime".as_ptr()) == QFALSE
    {
        //flamethrower
        Boba_DoFlameThrower(NPC);
        enemyCS = QFALSE;
        shoot = QFALSE;
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
        faceEnemy = QTRUE;
        ucmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
    } else if enemyDist < MIN_ROCKET_DIST_SQUARED {
        //128
        //enemy within 128
        if ((*(*NPC).client).ps.weapon == WP_FLECHETTE || (*(*NPC).client).ps.weapon == WP_REPEATER)
            && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0
        {
            //shooting an explosive, but enemy too close, switch to primary fire
            (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
            //FIXME: we can never go back to alt-fire this way since, after this, we don't know if we were initially supposed to use alt-fire or not...
        }
    } else if enemyDist > 65536.0 {
        //256 squared
        if (*(*NPC).client).ps.weapon == WP_DISRUPTOR {
            //sniping... should be assumed
            if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0 {
                //use primary fire
                (*NPCInfo).scriptFlags |= SCF_ALT_FIRE;
                //reset fire-timing variables
                NPC_ChangeWeapon(WP_DISRUPTOR);
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }
    }

    //can we see our target?
    if TIMER_Done(NPC, c"nextAttackDelay".as_ptr()) != QFALSE
        && TIMER_Done(NPC, c"flameTime".as_ptr()) != QFALSE
    {
        if NPC_ClearLOS4((*NPC).enemy) != QFALSE {
            (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
            enemyLOS = QTRUE;

            if (*(*NPC).client).ps.weapon == WP_NONE {
                enemyCS = QFALSE; //not true, but should stop us from firing
            } else {
                //can we shoot our target?
                if ((*(*NPC).client).ps.weapon == WP_ROCKET_LAUNCHER
                    || ((*(*NPC).client).ps.weapon == WP_FLECHETTE
                        && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0))
                    && enemyDist < MIN_ROCKET_DIST_SQUARED
                {
                    //128*128
                    enemyCS = QFALSE; //not true, but should stop us from firing
                    hitAlly = QTRUE; //us!
                                     //FIXME: if too close, run away!
                } else if enemyInFOV != QFALSE {
                    //if enemy is FOV, go ahead and check for shooting
                    let hit: c_int = NPC_ShotEntity((*NPC).enemy, &mut impactPos);
                    let hitEnt: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(hit as usize);

                    if hit == (*(*NPC).enemy).s.number
                        || (!hitEnt.is_null()
                            && !(*hitEnt).client.is_null()
                            && (*(*hitEnt).client).playerTeam == (*(*NPC).client).enemyTeam)
                        || (!hitEnt.is_null()
                            && (*hitEnt).takedamage != QFALSE
                            && (((*hitEnt).r.svFlags & SVF_GLASS_BRUSH) != 0
                                || (*hitEnt).health < 40
                                || (*NPC).s.weapon == WP_EMPLACED_GUN))
                    {
                        //can hit enemy or enemy ally or will hit glass or other minor breakable (or in emplaced gun), so shoot anyway
                        enemyCS = QTRUE;
                        //NPC_AimAdjust( 2 );//adjust aim better longer we have clear shot at enemy
                        VectorCopy(
                            &(*(*NPC).enemy).r.currentOrigin,
                            &mut (*NPCInfo).enemyLastSeenLocation,
                        );
                    } else {
                        //Hmm, have to get around this bastard
                        //NPC_AimAdjust( 1 );//adjust aim better longer we can see enemy
                        if !hitEnt.is_null()
                            && !(*hitEnt).client.is_null()
                            && (*(*hitEnt).client).playerTeam == (*(*NPC).client).playerTeam
                        {
                            //would hit an ally, don't fire!!!
                            hitAlly = QTRUE;
                        } else {
                            //Check and see where our shot *would* hit... if it's not close to the enemy (within 256?), then don't fire
                        }
                    }
                } else {
                    enemyCS = QFALSE; //not true, but should stop us from firing
                }
            }
        } else if trap::InPVS(&(*(*NPC).enemy).r.currentOrigin, &(*NPC).r.currentOrigin) != QFALSE {
            (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
            faceEnemy = QTRUE;
            //NPC_AimAdjust( -1 );//adjust aim worse longer we cannot see enemy
        }

        if (*(*NPC).client).ps.weapon == WP_NONE {
            faceEnemy = QFALSE;
            shoot = QFALSE;
        } else {
            if enemyLOS != QFALSE {
                //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
                faceEnemy = QTRUE;
            }
            if enemyCS != QFALSE {
                shoot = QTRUE;
            }
        }

        if enemyCS == QFALSE {
            //if have a clear shot, always try
            //See if we should continue to fire on their last position
            // !TIMER_Done( NPC, "stick" ) ||
            if hitAlly == QFALSE //we're not going to hit an ally
                && enemyInFOV != QFALSE //enemy is in our FOV //FIXME: or we don't have a clear LOS?
                && (*NPCInfo).enemyLastSeenTime > 0
            //we've seen the enemy
            {
                if (*addr_of!(level)).time - (*NPCInfo).enemyLastSeenTime < 10000 {
                    //we have seem the enemy in the last 10 seconds
                    if Q_irand(0, 10) == 0 {
                        //Fire on the last known position
                        let mut muzzle: vec3_t = [0.0; 3];
                        let mut dir: vec3_t = [0.0; 3];
                        let mut angles: vec3_t = [0.0; 3];
                        let mut tooClose: qboolean = QFALSE;
                        let mut tooFar: qboolean = QFALSE;
                        let mut distThreshold: f32;
                        let mut dist: f32;

                        CalcEntitySpot(NPC, SPOT_HEAD, &mut muzzle);
                        if VectorCompare(&impactPos, &vec3_origin) != 0 {
                            //never checked ShotEntity this frame, so must do a trace...
                            //vec3_t	mins = {-2,-2,-2}, maxs = {2,2,2};
                            let mut forward: vec3_t = [0.0; 3];
                            let mut end: vec3_t = [0.0; 3];
                            AngleVectors(
                                &(*(*NPC).client).ps.viewangles,
                                Some(&mut forward),
                                None,
                                None,
                            );
                            VectorMA(&muzzle, 8192.0, &forward, &mut end);
                            let tr: trace_t = trap::Trace(
                                &muzzle,
                                &vec3_origin,
                                &vec3_origin,
                                &end,
                                (*NPC).s.number,
                                MASK_SHOT,
                            );
                            VectorCopy(&tr.endpos, &mut impactPos);
                        }

                        //see if impact would be too close to me
                        distThreshold = 16384.0 /*128*128*/; //default
                        match (*NPC).s.weapon {
                            WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                            | WP_DET_PACK => {
                                distThreshold = 65536.0 /*256*256*/;
                            }
                            WP_REPEATER => {
                                if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                                    distThreshold = 65536.0 /*256*256*/;
                                }
                            }
                            _ => {}
                        }

                        dist = DistanceSquared(&impactPos, &muzzle);

                        if dist < distThreshold {
                            //impact would be too close to me
                            tooClose = QTRUE;
                        } else if (*addr_of!(level)).time - (*NPCInfo).enemyLastSeenTime > 5000
                            || (!(*NPCInfo).group.is_null()
                                && (*addr_of!(level)).time - (*(*NPCInfo).group).lastSeenEnemyTime
                                    > 5000)
                        {
                            //we've haven't seen them in the last 5 seconds
                            //see if it's too far from where he is
                            distThreshold = 65536.0 /*256*256*/; //default
                            match (*NPC).s.weapon {
                                WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                                | WP_DET_PACK => {
                                    distThreshold = 262144.0 /*512*512*/;
                                }
                                WP_REPEATER => {
                                    if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                                        distThreshold = 262144.0 /*512*512*/;
                                    }
                                }
                                _ => {}
                            }
                            dist = DistanceSquared(&impactPos, &(*NPCInfo).enemyLastSeenLocation);
                            if dist > distThreshold {
                                //impact would be too far from enemy
                                tooFar = QTRUE;
                            }
                        }

                        if tooClose == QFALSE && tooFar == QFALSE {
                            //okay too shoot at last pos
                            VectorSubtract(
                                &(*NPCInfo).enemyLastSeenLocation,
                                &muzzle,
                                &mut dir,
                            );
                            VectorNormalize(&mut dir);
                            vectoangles(&dir, &mut angles);

                            (*NPCInfo).desiredYaw = angles[YAW as usize];
                            (*NPCInfo).desiredPitch = angles[PITCH as usize];

                            shoot = QTRUE;
                            faceEnemy = QFALSE;
                        }
                    }
                }
            }
        }

        //FIXME: don't shoot right away!
        if (*(*NPC).client).ps.weaponTime > 0 {
            if (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
                if enemyLOS == QFALSE || enemyCS == QFALSE {
                    //cancel it
                    (*(*NPC).client).ps.weaponTime = 0;
                } else {
                    //delay our next attempt
                    TIMER_Set(NPC, c"nextAttackDelay".as_ptr(), Q_irand(500, 1000));
                }
            }
        } else if shoot != QFALSE {
            //try to shoot if it's time
            if TIMER_Done(NPC, c"nextAttackDelay".as_ptr()) != QFALSE {
                if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) == 0 {
                    // we've already fired, no need to do it again here
                    WeaponThink(QTRUE);
                }
                //NASTY
                if (*NPC).s.weapon == WP_ROCKET_LAUNCHER
                    && (ucmd.buttons & BUTTON_ATTACK) != 0
                    && Q_irand(0, 3) == 0
                {
                    //every now and then, shoot a homing rocket
                    ucmd.buttons &= !BUTTON_ATTACK;
                    ucmd.buttons |= BUTTON_ALT_ATTACK;
                    (*(*NPC).client).ps.weaponTime = Q_irand(500, 1500);
                }
            }
        }
    }
}

/// `void Jedi_Cloak( gentity_t *self )` (NPC_AI_Jedi.c:799). Sets the notarget
/// flag and, if not already cloaked, engages the cloak (powerup set to
/// `Q3_INFINITE`) with a cloak sound. No oracle (entity-state + sound).
///
/// # Safety
/// `self` may be NULL (guarded); `self->client` may be NULL (guarded).
pub unsafe fn Jedi_Cloak(self_: *mut gentity_t) {
    if !self_.is_null() {
        (*self_).flags |= FL_NOTARGET;
        if !(*self_).client.is_null() {
            if (*(*self_).client).ps.powerups[PW_CLOAKED as usize] == 0 {
                //cloak
                (*(*self_).client).ps.powerups[PW_CLOAKED as usize] = Q3_INFINITE;

                //FIXME: debounce attacks?
                //FIXME: temp sound
                G_Sound(
                    self_,
                    CHAN_ITEM,
                    G_SoundIndex("sound/chars/shadowtrooper/cloak.wav"),
                );
            }
        }
    }
}

/// `void Jedi_Decloak( gentity_t *self )` (NPC_AI_Jedi.c:818). Clears the notarget
/// flag and, if currently cloaked, disengages the cloak (powerup zeroed) with a
/// decloak sound. No oracle (entity-state + sound).
///
/// # Safety
/// `self` may be NULL (guarded); `self->client` may be NULL (guarded).
pub unsafe fn Jedi_Decloak(self_: *mut gentity_t) {
    if !self_.is_null() {
        (*self_).flags &= !FL_NOTARGET;
        if !(*self_).client.is_null() {
            if (*(*self_).client).ps.powerups[PW_CLOAKED as usize] != 0 {
                //Uncloak
                (*(*self_).client).ps.powerups[PW_CLOAKED as usize] = 0;

                G_Sound(
                    self_,
                    CHAN_ITEM,
                    G_SoundIndex("sound/chars/shadowtrooper/decloak.wav"),
                );
            }
        }
    }
}

/// `void Jedi_CheckCloak( void )` (NPC_AI_Jedi.c:835). For a shadowtrooper NPC,
/// decloaks when the saber is on / dead / saber-in-flight / taking pain, else
/// re-cloaks when alive, saber holstered, not in flight and not in pain. No oracle
/// (NPC-global entity-state). The commented-out EF_FORCE_GRIPPED/DRAINED gates are
/// carried over verbatim as in the C.
///
/// # Safety
/// The `NPC` global and (where reached) `NPC->client` must be valid.
pub unsafe fn Jedi_CheckCloak() {
    if !NPC.is_null()
        && !(*NPC).client.is_null()
        && (*(*NPC).client).NPC_class == CLASS_SHADOWTROOPER
    {
        if (*(*NPC).client).ps.saberHolstered == 0
            || (*NPC).health <= 0
            || (*(*NPC).client).ps.saberInFlight == QTRUE
            //	(NPC->client->ps.eFlags&EF_FORCE_GRIPPED) ||
            //	(NPC->client->ps.eFlags&EF_FORCE_DRAINED) ||
            || (*NPC).painDebounceTime > (*addr_of!(level)).time
        {
            //can't be cloaked if saber is on, or dead or saber in flight or taking pain or being gripped
            Jedi_Decloak(NPC);
        } else if (*NPC).health > 0
            && (*(*NPC).client).ps.saberInFlight != QTRUE
            //	&& !(NPC->client->ps.eFlags&EF_FORCE_GRIPPED)
            //	&& !(NPC->client->ps.eFlags&EF_FORCE_DRAINED)
            && (*NPC).painDebounceTime < (*addr_of!(level)).time
        {
            //still alive, have saber in hand, not taking pain and not being gripped
            Jedi_Cloak(NPC);
        }
    }
}

/// `static void Jedi_Aggression( gentity_t *self, int change )`
/// (NPC_AI_Jedi.c:863). Adjusts the Jedi's aggression stat by `change`, then
/// clamps it into a team/class-dependent window (good guys calmer; Desann the most
/// aggressive). No oracle (entity-state).
///
/// # Safety
/// `self`, `self->client`, and `self->NPC` must be valid.
#[allow(dead_code)] // used by Jedi_AggressionErosion/Rage/RageStop siblings as they land
unsafe fn Jedi_Aggression(self_: *mut gentity_t, change: c_int) {
    let upper_threshold: c_int;
    let lower_threshold: c_int;

    (*(*self_).NPC).stats.aggression += change;

    //FIXME: base this on initial NPC stats
    if (*(*self_).client).playerTeam == NPCTEAM_PLAYER {
        //good guys are less aggressive
        upper_threshold = 7;
        lower_threshold = 1;
    } else {
        //bad guys are more aggressive
        if (*(*self_).client).NPC_class == CLASS_DESANN {
            upper_threshold = 20;
            lower_threshold = 5;
        } else {
            upper_threshold = 10;
            lower_threshold = 3;
        }
    }

    if (*(*self_).NPC).stats.aggression > upper_threshold {
        (*(*self_).NPC).stats.aggression = upper_threshold;
    } else if (*(*self_).NPC).stats.aggression < lower_threshold {
        (*(*self_).NPC).stats.aggression = lower_threshold;
    }
    //Com_Printf( "(%d) %s agg %d change: %d\n", level.time, self->NPC_type, self->NPC->stats.aggression, change );
}

/// `static void Jedi_AggressionErosion( int amt )` (NPC_AI_Jedi.c:900). While
/// un-alerted with no enemy, periodically lowers aggression; once low enough
/// (class-dependent for Desann), turns off the saber. No oracle (NPC-global
/// entity-state + timers).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->client` must be valid.
#[allow(dead_code)] // used by Jedi behavior siblings (NPC_BSJediMaster_*) as they land
unsafe fn Jedi_AggressionErosion(amt: c_int) {
    if TIMER_Done(NPC, c"roamTime".as_ptr()) != QFALSE {
        //the longer we're not alerted and have no enemy, the more our aggression goes down
        TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(2000, 5000));
        Jedi_Aggression(NPC, amt);
    }

    if (*NPCInfo).stats.aggression < 4
        || ((*NPCInfo).stats.aggression < 6 && (*(*NPC).client).NPC_class == CLASS_DESANN)
    {
        //turn off the saber
        WP_DeactivateSaber(NPC, QFALSE);
    }
}

/// `void NPC_Jedi_RateNewEnemy( gentity_t *self, gentity_t *enemy )`
/// (NPC_AI_Jedi.c:914). Picks an initial aggression toward a freshly-acquired enemy
/// from the enemy's weapon (saber/close-blaster = charge in, far blaster = hang back)
/// and the Jedi's own health, averages it with current aggression, applies the delta
/// via [`Jedi_Aggression`] and holds off taunting for a few seconds. No oracle
/// (entity-state + timers).
///
/// # Safety
/// `self`/`enemy` must be valid; `self->NPC` must be valid.
pub unsafe fn NPC_Jedi_RateNewEnemy(self_: *mut gentity_t, enemy: *mut gentity_t) {
    let healthAggression: f32;
    let weaponAggression: f32;
    let newAggression: c_int;

    match (*enemy).s.weapon {
        WP_SABER => {
            healthAggression = (*self_).health as f32 / 200.0 * 6.0;
            weaponAggression = 7.0; //go after him
        }
        WP_BLASTER => {
            if DistanceSquared(&(*self_).r.currentOrigin, &(*enemy).r.currentOrigin) < 65536.0 {
                //256 squared
                healthAggression = (*self_).health as f32 / 200.0 * 8.0;
                weaponAggression = 8.0; //go after him
            } else {
                healthAggression = 8.0 - ((*self_).health as f32 / 200.0 * 8.0);
                weaponAggression = 2.0; //hang back for a second
            }
        }
        _ => {
            healthAggression = (*self_).health as f32 / 200.0 * 8.0;
            weaponAggression = 6.0; //approach
        }
    }
    //Average these with current aggression
    newAggression =
        ((healthAggression + weaponAggression + (*(*self_).NPC).stats.aggression as f32) / 3.0)
            .ceil() as c_int;
    //Com_Printf( "(%d) new agg %d - new enemy\n", level.time, newAggression );
    Jedi_Aggression(self_, newAggression - (*(*self_).NPC).stats.aggression);

    //don't taunt right away
    TIMER_Set(self_, c"chatter".as_ptr(), Q_irand(4000, 7000));
}

/// `static void Jedi_Rage( void )` (NPC_AI_Jedi.c:952). Spikes aggression toward
/// the max, clears the roam/chatter/movement timers, then triggers Force Rage.
/// No oracle (NPC-global entity-state + timers + force power).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals must be valid.
#[allow(dead_code)] // used by Jedi behavior siblings as they land
unsafe fn Jedi_Rage() {
    Jedi_Aggression(NPC, 10 - (*NPCInfo).stats.aggression + Q_irand(-2, 2));
    TIMER_Set(NPC, c"roamTime".as_ptr(), 0);
    TIMER_Set(NPC, c"chatter".as_ptr(), 0);
    TIMER_Set(NPC, c"walking".as_ptr(), 0);
    TIMER_Set(NPC, c"taunting".as_ptr(), 0);
    TIMER_Set(NPC, c"jumpChaseDebounce".as_ptr(), 0);
    TIMER_Set(NPC, c"movenone".as_ptr(), 0);
    TIMER_Set(NPC, c"movecenter".as_ptr(), 0);
    TIMER_Set(NPC, c"noturn".as_ptr(), 0);
    ForceRage(NPC);
}

/// `void Jedi_RageStop( gentity_t *self )` (NPC_AI_Jedi.c:966). On an NPC, calms
/// down and backs off (clears roamTime, drops aggression a little). No oracle
/// (entity-state + timers).
///
/// # Safety
/// `self` must be valid; `self->NPC` may be NULL (guarded).
pub unsafe fn Jedi_RageStop(self_: *mut gentity_t) {
    if !(*self_).NPC.is_null() {
        //calm down and back off
        TIMER_Set(self_, c"roamTime".as_ptr(), 0);
        Jedi_Aggression(self_, Q_irand(-5, 0));
    }
}

/// `static qboolean Jedi_BattleTaunt( void )` (NPC_AI_Jedi.c:980). On a chatter
/// debounce, occasionally plays a taunt voice event (only the trainer taunts when
/// a player-team Jedi faces another Jedi; reborn/enemy Jedi taunt freely), arming
/// the per-team and per-NPC speech debounces. No oracle (NPC-global entity-state +
/// timers + voice event).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->client` must be valid; `NPC->enemy` may
/// be NULL (guarded).
#[allow(dead_code)] // used by Jedi behavior siblings as they land
unsafe fn Jedi_BattleTaunt() -> qboolean {
    if TIMER_Done(NPC, c"chatter".as_ptr()) != QFALSE
        && Q_irand(0, 3) == 0
        && (*NPCInfo).blockedSpeechDebounceTime < (*addr_of!(level)).time
        && jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize] < (*addr_of!(level)).time
    {
        let mut event: c_int = -1;
        if (*(*NPC).client).playerTeam == NPCTEAM_PLAYER
            && !(*NPC).enemy.is_null()
            && !(*(*NPC).enemy).client.is_null()
            && (*(*(*NPC).enemy).client).NPC_class == CLASS_JEDI
        {
            //a jedi fighting a jedi - training
            if (*(*NPC).client).NPC_class == CLASS_JEDI && (*NPCInfo).rank == RANK_COMMANDER {
                //only trainer taunts
                event = EV_TAUNT1;
            }
        } else {
            //reborn or a jedi fighting an enemy
            event = Q_irand(EV_TAUNT1, EV_TAUNT3);
        }
        if event != -1 {
            G_AddVoiceEvent(NPC, event, 3000);
            (*NPCInfo).blockedSpeechDebounceTime = (*addr_of!(level)).time + 6000;
            jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize] =
                (*NPCInfo).blockedSpeechDebounceTime;
            TIMER_Set(NPC, c"chatter".as_ptr(), Q_irand(5000, 10000));

            if !(*NPC).enemy.is_null()
                && !(*(*NPC).enemy).NPC.is_null()
                && (*(*NPC).enemy).s.weapon == WP_SABER
                && !(*(*NPC).enemy).client.is_null()
                && (*(*(*NPC).enemy).client).NPC_class == CLASS_JEDI
            {
                //Have the enemy jedi say something in response when I'm done?
            }
            return QTRUE;
        }
    }
    QFALSE
}

/// `static void Jedi_HoldPosition( void )` (NPC_AI_Jedi.c:1200). Clears the NPC's
/// goal entity so it holds position. The commented-out squad-state and duck-timer
/// lines are carried over verbatim. No oracle (NPC-global entity-state).
///
/// # Safety
/// The `NPCInfo` global must be valid.
#[allow(dead_code)] // used by Jedi movement siblings as they land
unsafe fn Jedi_HoldPosition() {
    //NPCInfo->squadState = SQUAD_STAND_AND_SHOOT;
    (*NPCInfo).goalEntity = null_mut();

    /*
    if ( TIMER_Done( NPC, "stand" ) )
    {
        TIMER_Set( NPC, "duck", Q_irand( 2000, 4000 ) );
    }
    */
}

/// `static qboolean Jedi_ClearPathToSpot( vec3_t dest, int impactEntNum )`
/// (NPC_AI_Jedi.c:1020). Returns `qtrue` if the NPC has a clear walking path to
/// `dest`: a straight trace must reach it (or hit `impactEntNum`), and at body-size
/// intervals along the way there must be solid floor within a step/drop. No oracle
/// (NPC-global entity-state + trap_Trace).
///
/// # Safety
/// The `NPC` global must be valid.
#[allow(dead_code)] // used by Jedi movement siblings as they land
unsafe fn Jedi_ClearPathToSpot(dest: &vec3_t, impactEntNum: c_int) -> qboolean {
    let mut trace: trace_t;
    let mut mins: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let dist: f32;
    let drop: f32;
    let mut i: f32;

    //Offset the step height
    VectorSet(
        &mut mins,
        (*NPC).r.mins[0],
        (*NPC).r.mins[1],
        (*NPC).r.mins[2] + STEPSIZE as f32,
    );

    trace = trap::Trace(
        &(*NPC).r.currentOrigin,
        &mins,
        &(*NPC).r.maxs,
        dest,
        (*NPC).s.number,
        (*NPC).clipmask,
    );

    //Do a simple check
    if trace.allsolid != 0 || trace.startsolid != 0 {
        //inside solid
        return QFALSE;
    }

    if trace.fraction < 1.0 {
        //hit something
        if impactEntNum != ENTITYNUM_NONE && trace.entityNum as c_int == impactEntNum {
            //hit what we're going after
            return QTRUE;
        } else {
            return QFALSE;
        }
    }

    //otherwise, clear path in a straight line.
    //Now at intervals of my size, go along the trace and trace down STEPSIZE to make sure there is a solid floor.
    VectorSubtract(dest, &(*NPC).r.currentOrigin, &mut dir);
    dist = VectorNormalize(&mut dir);
    if dest[2] > (*NPC).r.currentOrigin[2] {
        //going up, check for steps
        drop = STEPSIZE as f32;
    } else {
        //going down or level, check for moderate drops
        drop = 64.0;
    }
    i = (*NPC).r.maxs[0] * 2.0;
    while i < dist {
        //FIXME: does this check the last spot, too?  We're assuming that should be okay since the enemy is there?
        VectorMA(&(*NPC).r.currentOrigin, i, &dir, &mut start);
        VectorCopy(&start, &mut end);
        end[2] -= drop;
        trace = trap::Trace(
            &start,
            &mins,
            &(*NPC).r.maxs,
            &end,
            (*NPC).s.number,
            (*NPC).clipmask,
        ); //NPC->r.mins?
        if trace.fraction < 1.0 || trace.allsolid != 0 || trace.startsolid != 0 {
            //good to go
            i += (*NPC).r.maxs[0] * 2.0;
            continue;
        }
        //no floor here! (or a long drop?)
        return QFALSE;
    }
    //we made it!
    QTRUE
}

/// `qboolean NPC_MoveDirClear( int forwardmove, int rightmove, qboolean reset )`
/// (NPC_AI_Jedi.c:1079). Walk/cliff check: traces a short step in the intended
/// move direction and a drop trace past it, returning `qfalse` (and, if `reset`,
/// zeroing/reversing the move command) when about to walk into a wall or off a
/// cliff. No oracle (NPC-global entity-state + trap_Trace).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->client` must be valid.
pub unsafe fn NPC_MoveDirClear(forwardmove: c_int, rightmove: c_int, reset: qboolean) -> qboolean {
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut testPos: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut trace: trace_t;
    let fwdDist: f32;
    let rtDist: f32;
    let mut bottom_max: f32 = -(STEPSIZE as f32) * 4.0 - 1.0;

    if forwardmove == 0 && rightmove == 0 {
        //not even moving
        //Com_Printf( "%d skipping walk-cliff check (not moving)\n", level.time );
        return QTRUE;
    }

    if ucmd.upmove > 0 || (*(*NPC).client).ps.fd.forceJumpCharge != 0.0 {
        //Going to jump
        //Com_Printf( "%d skipping walk-cliff check (going to jump)\n", level.time );
        return QTRUE;
    }

    if (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE {
        //in the air
        //Com_Printf( "%d skipping walk-cliff check (in air)\n", level.time );
        return QTRUE;
    }
    /*
    if ( fabs( AngleDelta( NPC->r.currentAngles[YAW], NPCInfo->desiredYaw ) ) < 5.0 )//!ucmd.angles[YAW]
    {//Not turning much, don't do this
        //NOTE: Should this not happen only if you're not turning AT ALL?
        //	You could be turning slowly but moving fast, so that would
        //	still let you walk right off a cliff...
        //NOTE: Or maybe it is a good idea to ALWAYS do this, regardless
        //	of whether ot not we're turning?  But why would we be walking
        //  straight into a wall or off	a cliff unless we really wanted to?
        return;
    }
    */

    //FIXME: to really do this right, we'd have to actually do a pmove to predict where we're
    //going to be... maybe this should be a flag and pmove handles it and sets a flag so AI knows
    //NEXT frame?  Or just incorporate current velocity, runspeed and possibly friction?
    VectorCopy(&(*NPC).r.mins, &mut mins);
    mins[2] += STEPSIZE as f32;
    angles[PITCH] = 0.0;
    angles[ROLL] = 0.0;
    angles[YAW] = (*(*NPC).client).ps.viewangles[YAW]; //Add ucmd.angles[YAW]?
    AngleVectors(&angles, Some(&mut forward), Some(&mut right), None);
    fwdDist = (forwardmove as f32) / 2.0;
    rtDist = (rightmove as f32) / 2.0;
    VectorMA(&(*NPC).r.currentOrigin, fwdDist, &forward, &mut testPos);
    let testPosCopy = testPos;
    VectorMA(&testPosCopy, rtDist, &right, &mut testPos);
    trace = trap::Trace(
        &(*NPC).r.currentOrigin,
        &mins,
        &(*NPC).r.maxs,
        &testPos,
        (*NPC).s.number,
        (*NPC).clipmask | CONTENTS_BOTCLIP,
    );
    if trace.allsolid != 0 || trace.startsolid != 0 {
        //hmm, trace started inside this brush... how do we decide if we should continue?
        //FIXME: what do we do if we start INSIDE a CONTENTS_BOTCLIP? Try the trace again without that in the clipmask?
        if reset == QTRUE {
            trace.fraction = 1.0;
        }
        VectorCopy(&testPos, &mut trace.endpos);
        //return qtrue;
    }
    if trace.fraction < 0.6 {
        //Going to bump into something very close, don't move, just turn
        if (!(*NPC).enemy.is_null() && trace.entityNum as c_int == (*(*NPC).enemy).s.number)
            || (!(*NPCInfo).goalEntity.is_null()
                && trace.entityNum as c_int == (*(*NPCInfo).goalEntity).s.number)
        {
            //okay to bump into enemy or goal
            //Com_Printf( "%d bump into enemy/goal okay\n", level.time );
            return QTRUE;
        } else if reset == QTRUE {
            //actually want to screw with the ucmd
            //Com_Printf( "%d avoiding walk into wall (entnum %d)\n", level.time, trace.entityNum );
            ucmd.forwardmove = 0;
            ucmd.rightmove = 0;
            VectorClear(&mut (*(*NPC).client).ps.moveDir);
        }
        return QFALSE;
    }

    if !(*NPCInfo).goalEntity.is_null() {
        if (*(*NPCInfo).goalEntity).r.currentOrigin[2] < (*NPC).r.currentOrigin[2] {
            //goal is below me, okay to step off at least that far plus stepheight
            bottom_max +=
                (*(*NPCInfo).goalEntity).r.currentOrigin[2] - (*NPC).r.currentOrigin[2];
        }
    }
    VectorCopy(&trace.endpos, &mut testPos);
    testPos[2] += bottom_max;

    let traceEnd = trace.endpos;
    trace = trap::Trace(
        &traceEnd,
        &mins,
        &(*NPC).r.maxs,
        &testPos,
        (*NPC).s.number,
        (*NPC).clipmask,
    );

    //FIXME:Should we try to see if we can still get to our goal using the waypoint network from this trace.endpos?
    //OR: just put NPC clip brushes on these edges (still fall through when die)

    if trace.allsolid != 0 || trace.startsolid != 0 {
        //Not going off a cliff
        //Com_Printf( "%d walk off cliff okay (droptrace in solid)\n", level.time );
        return QTRUE;
    }

    if trace.fraction < 1.0 {
        //Not going off a cliff
        //FIXME: what if plane.normal is sloped?  We'll slide off, not land... plus this doesn't account for slide-movement...
        //Com_Printf( "%d walk off cliff okay will hit entnum %d at dropdist of %4.2f\n", level.time, trace.entityNum, (trace.fraction*bottom_max) );
        return QTRUE;
    }

    //going to fall at least bottom_max, don't move, just turn... is this bad, though?  What if we want them to drop off?
    if reset == QTRUE {
        //actually want to screw with the ucmd
        //Com_Printf( "%d avoiding walk off cliff\n", level.time );
        ucmd.forwardmove = (ucmd.forwardmove as c_int * -1) as i8; //= 0;
        ucmd.rightmove = (ucmd.rightmove as c_int * -1) as i8; //= 0;
        let moveDir = (*(*NPC).client).ps.moveDir;
        VectorScale(&moveDir, -1.0, &mut (*(*NPC).client).ps.moveDir);
    }
    QFALSE
}

/// `static void Jedi_Move( gentity_t *goal, qboolean retreat )`
/// (NPC_AI_Jedi.c:1219). Sets the NPC's combat-move goal and walks toward it; on
/// `retreat`, reverses the resulting move command and move direction. Holds
/// position if it collides with the enemy or the move fails. No oracle (NPC-global
/// entity-state + NAV).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->client`/`NPC->enemy` must be valid.
#[allow(dead_code)] // used by Jedi_Retreat/Advance siblings as they land
unsafe fn Jedi_Move(goal: *mut gentity_t, retreat: qboolean) {
    let moved: qboolean;
    let mut info: navInfo_t = core::mem::zeroed();

    (*NPCInfo).combatMove = QTRUE;
    (*NPCInfo).goalEntity = goal;

    moved = NPC_MoveToGoal(QTRUE);

    //FIXME: temp retreat behavior- should really make this toward a safe spot or maybe to outflank enemy
    if retreat == QTRUE {
        //FIXME: should we trace and make sure we can go this way?  Or somehow let NPC_MoveToGoal know we want to retreat and have it handle it?
        ucmd.forwardmove = (ucmd.forwardmove as c_int * -1) as i8;
        ucmd.rightmove = (ucmd.rightmove as c_int * -1) as i8;
        let moveDir = (*(*NPC).client).ps.moveDir;
        VectorScale(&moveDir, -1.0, &mut (*(*NPC).client).ps.moveDir);
    }

    //Get the move info
    NAV_GetLastMove(&mut info);

    //If we hit our target, then stop and fire!
    if (info.flags & NIF_COLLISION) != 0 && info.blocker == (*NPC).enemy {
        Jedi_HoldPosition();
    }

    //If our move failed, then reset
    if moved == QFALSE {
        Jedi_HoldPosition();
    }
}

/// `static qboolean Jedi_Strafe( int strafeTimeMin, int strafeTimeMax, int nextStrafeTimeMin, int nextStrafeTimeMax, qboolean walking )`
/// (NPC_AI_Jedi.c:1876). Picks a (cliff/wall-clear) strafe direction at random and
/// arms the strafe/no-strafe (and optional walking) timers, unless pressing a
/// saber-lock win. Returns `qtrue` if it started a strafe. No oracle (NPC-global
/// entity-state + timers).
///
/// # Safety
/// The `NPC` global and `NPC->client` must be valid; `NPC->enemy` may be NULL.
#[allow(dead_code)] // used by Jedi combat siblings as they land
unsafe fn Jedi_Strafe(
    strafeTimeMin: c_int,
    strafeTimeMax: c_int,
    nextStrafeTimeMin: c_int,
    nextStrafeTimeMax: c_int,
    walking: qboolean,
) -> qboolean {
    if (*(*NPC).client).ps.saberEventFlags & SEF_LOCK_WON != 0
        && !(*NPC).enemy.is_null()
        && (*(*NPC).enemy).painDebounceTime > (*addr_of!(level)).time
    {
        //don't strafe if pressing the advantage of winning a saberLock
        return QFALSE;
    }
    if TIMER_Done(NPC, c"strafeLeft".as_ptr()) != QFALSE
        && TIMER_Done(NPC, c"strafeRight".as_ptr()) != QFALSE
    {
        let mut strafed: qboolean = QFALSE;
        //TODO: make left/right choice a tactical decision rather than random:
        //		try to keep own back away from walls and ledges,
        //		try to keep enemy's back to a ledge or wall
        //		Maybe try to strafe toward designer-placed "safe spots" or "goals"?
        let strafeTime: c_int = Q_irand(strafeTimeMin, strafeTimeMax);

        if Q_irand(0, 1) != 0 {
            if NPC_MoveDirClear(ucmd.forwardmove as c_int, -127, QFALSE) != QFALSE {
                TIMER_Set(NPC, c"strafeLeft".as_ptr(), strafeTime);
                strafed = QTRUE;
            } else if NPC_MoveDirClear(ucmd.forwardmove as c_int, 127, QFALSE) != QFALSE {
                TIMER_Set(NPC, c"strafeRight".as_ptr(), strafeTime);
                strafed = QTRUE;
            }
        } else {
            if NPC_MoveDirClear(ucmd.forwardmove as c_int, 127, QFALSE) != QFALSE {
                TIMER_Set(NPC, c"strafeRight".as_ptr(), strafeTime);
                strafed = QTRUE;
            } else if NPC_MoveDirClear(ucmd.forwardmove as c_int, -127, QFALSE) != QFALSE {
                TIMER_Set(NPC, c"strafeLeft".as_ptr(), strafeTime);
                strafed = QTRUE;
            }
        }

        if strafed != QFALSE {
            TIMER_Set(
                NPC,
                c"noStrafe".as_ptr(),
                strafeTime + Q_irand(nextStrafeTimeMin, nextStrafeTimeMax),
            );
            if walking != QFALSE {
                //should be a slow strafe
                TIMER_Set(NPC, c"walking".as_ptr(), strafeTime);
            }
            return QTRUE;
        }
    }
    QFALSE
}

/// `static qboolean Jedi_Hunt( void )` (NPC_AI_Jedi.c:1253). If aggressive enough,
/// approaches the enemy — either just facing it (when not allowed to chase) or
/// moving to it. Returns `qtrue` if it acted. No oracle (NPC-global entity-state +
/// NAV).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->enemy` must be valid.
#[allow(dead_code)] // used by Jedi behavior siblings as they land
unsafe fn Jedi_Hunt() -> qboolean {
    //Com_Printf( "Hunting\n" );
    //if we're at all interested in fighting, go after him
    if (*NPCInfo).stats.aggression > 1 {
        //approach enemy
        (*NPCInfo).combatMove = QTRUE;
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0 {
            NPC_UpdateAngles(QTRUE, QTRUE);
            return QTRUE;
        } else {
            if (*NPCInfo).goalEntity.is_null() {
                //hunt
                (*NPCInfo).goalEntity = (*NPC).enemy;
            }
            //Jedi_Move( NPC->enemy, qfalse );
            if NPC_MoveToGoal(QFALSE) == QTRUE {
                NPC_UpdateAngles(QTRUE, QTRUE);
                return QTRUE;
            }
        }
    }
    QFALSE
}

/// `static void Jedi_Retreat( void )` (NPC_AI_Jedi.c:1300). Backs away from the
/// enemy, unless the `noRetreat` timer says to hold. No oracle (NPC-global
/// entity-state + NAV).
///
/// # Safety
/// The `NPC` global and `NPC->enemy` must be valid.
#[allow(dead_code)] // used by Jedi behavior siblings as they land
unsafe fn Jedi_Retreat() {
    if TIMER_Done(NPC, c"noRetreat".as_ptr()) == QFALSE {
        //don't actually move
        return;
    }
    //FIXME: when retreating, we should probably see if we can retreat
    //in the direction we want.  If not...?  Evade?
    //Com_Printf( "Retreating\n" );
    Jedi_Move((*NPC).enemy, QTRUE);
}

/// `static void Jedi_Advance( void )` (NPC_AI_Jedi.c:1312). Activates the saber if
/// not already in flight, then moves toward the enemy. No oracle (NPC-global
/// entity-state + NAV).
///
/// # Safety
/// The `NPC` global and `NPC->client`/`NPC->enemy` must be valid.
#[allow(dead_code)] // used by Jedi behavior siblings as they land
unsafe fn Jedi_Advance() {
    if (*(*NPC).client).ps.saberInFlight == QFALSE {
        //NPC->client->ps.SaberActivate();
        WP_ActivateSaber(NPC);
    }
    //Com_Printf( "Advancing\n" );
    Jedi_Move((*NPC).enemy, QFALSE);

    //TIMER_Set( NPC, "roamTime", Q_irand( 2000, 4000 ) );
    //TIMER_Set( NPC, "attackDelay", Q_irand( 250, 500 ) );
    //TIMER_Set( NPC, "duck", 0 );
}

/// `static void Jedi_CombatDistance( int enemy_dist )` (NPC_AI_Jedi.c:1413). The
/// distance-management half of Jedi combat: from `enemy_dist` and a pile of
/// force/saber state it decides whether to advance, retreat, throw the saber, grip,
/// drain, heal/protect/absorb/rage, taunt, or hold position. Pure NPC AI driven by
/// timers, RNG, and force powers — no oracle. The PAS-turret branch and the
/// too-close-in-air branch are commented out in C; carried over verbatim.
///
/// # Safety
/// `NPC`/`NPC->client`/`NPCInfo` and `NPC->enemy` (+ `NPC->enemy->client` where the
/// C dereferences it) must be valid for the current combat frame.
#[allow(dead_code)] // called by Jedi_Combat as it lands
unsafe fn Jedi_CombatDistance(enemy_dist: c_int) {
    //FIXME: for many of these checks, what we really want is horizontal distance to enemy
    if (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0
        && (*(*NPC).client).ps.fd.forcePowerLevel[FP_GRIP as usize] > FORCE_LEVEL_1
    {
        //when gripping, don't move
        return;
    } else if TIMER_Done(NPC, c"gripping".as_ptr()) == QFALSE {
        //stopped gripping, clear timers just in case
        TIMER_Set(NPC, c"gripping".as_ptr(), -(*addr_of!(level)).time);
        TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(0, 1000));
    }

    if (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_DRAIN) != 0
        && (*(*NPC).client).ps.fd.forcePowerLevel[FP_DRAIN as usize] > FORCE_LEVEL_1
    {
        //when draining, don't move
        return;
    } else if TIMER_Done(NPC, c"draining".as_ptr()) == QFALSE {
        //stopped draining, clear timers just in case
        TIMER_Set(NPC, c"draining".as_ptr(), -(*addr_of!(level)).time);
        TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(0, 1000));
    }

    if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
        if TIMER_Done(NPC, c"flameTime".as_ptr()) == QFALSE {
            if enemy_dist > 50 {
                Jedi_Advance();
            } else if enemy_dist <= 0 {
                Jedi_Retreat();
            }
        } else if enemy_dist < 200 {
            Jedi_Retreat();
        } else if enemy_dist > 1024 {
            Jedi_Advance();
        }
    } else if (*(*NPC).client).ps.saberInFlight != QFALSE
        && PM_SaberInBrokenParry((*(*NPC).client).ps.saberMove) == QFALSE
        && (*(*NPC).client).ps.saberBlocked != BLOCKED_PARRY_BROKEN
    {
        //maintain distance
        if (enemy_dist as f32) < (*(*NPC).client).ps.saberEntityDist {
            Jedi_Retreat();
        } else if (enemy_dist as f32) > (*(*NPC).client).ps.saberEntityDist && enemy_dist > 100 {
            Jedi_Advance();
        }
        if (*(*NPC).client).ps.weapon == WP_SABER //using saber
            && (*(*NPC).client).ps.saberEntityState == SES_LEAVING  //not returning yet
            && (*(*NPC).client).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] > FORCE_LEVEL_1 //2nd or 3rd level lightsaber
            && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_SPEED) == 0
            && (*(*NPC).client).ps.saberEventFlags & SEF_INWATER == 0
        {
            //saber not in water
            //hold it out there
            ucmd.buttons |= BUTTON_ALT_ATTACK;
            //FIXME: time limit?
        }
    } else if TIMER_Done(NPC, c"taunting".as_ptr()) == QFALSE {
        if enemy_dist <= 64 {
            //he's getting too close
            ucmd.buttons |= BUTTON_ATTACK;
            if (*(*NPC).client).ps.saberInFlight == QFALSE {
                WP_ActivateSaber(NPC);
            }
            TIMER_Set(NPC, c"taunting".as_ptr(), -(*addr_of!(level)).time);
        }
        //else if ( NPC->client->ps.torsoAnim == BOTH_GESTURE1 && NPC->client->ps.torsoTimer < 2000 )
        else if (*(*NPC).client).ps.forceHandExtend == HANDEXTEND_JEDITAUNT
            && ((*(*NPC).client).ps.forceHandExtendTime - (*addr_of!(level)).time) < 200
        {
            //we're almost done with our special taunt
            //FIXME: this doesn't always work, for some reason
            if (*(*NPC).client).ps.saberInFlight == QFALSE {
                WP_ActivateSaber(NPC);
            }
        }
    } else if (*(*NPC).client).ps.saberEventFlags & SEF_LOCK_WON != 0 {
        //we won a saber lock, press the advantage
        if enemy_dist > 0 {
            //get closer so we can hit!
            Jedi_Advance();
        }
        if enemy_dist > 128 {
            //lost 'em
            (*(*NPC).client).ps.saberEventFlags &= !SEF_LOCK_WON;
        }
        if (*(*NPC).enemy).painDebounceTime + 2000 < (*addr_of!(level)).time {
            //the window of opportunity is gone
            (*(*NPC).client).ps.saberEventFlags &= !SEF_LOCK_WON;
        }
        //don't strafe?
        TIMER_Set(NPC, c"strafeLeft".as_ptr(), -1);
        TIMER_Set(NPC, c"strafeRight".as_ptr(), -1);
    } else if !(*(*NPC).enemy).client.is_null()
        && (*(*NPC).enemy).s.weapon == WP_SABER
        && (*(*(*NPC).enemy).client).ps.saberLockTime > (*addr_of!(level)).time
        && (*(*NPC).client).ps.saberLockTime < (*addr_of!(level)).time
    {
        //enemy is in a saberLock and we are not
        if enemy_dist < 64 {
            //FIXME: maybe just pick another enemy?
            Jedi_Retreat();
        }
    }
    /*
    else if ( NPC->enemy->s.weapon == WP_TURRET
        && !Q_stricmp( "PAS", NPC->enemy->classname )
        && NPC->enemy->s.apos.trType == TR_STATIONARY )
    {
        int	testlevel;
        if ( enemy_dist > forcePushPullRadius[FORCE_LEVEL_1] - 16 )
        {
            Jedi_Advance();
        }
        if ( NPC->client->ps.fd.forcePowerLevel[FP_PUSH] < FORCE_LEVEL_1 )
        {//
            testlevel = FORCE_LEVEL_1;
        }
        else
        {
            testlevel = NPC->client->ps.fd.forcePowerLevel[FP_PUSH];
        }
        if ( enemy_dist < forcePushPullRadius[testlevel] - 16 )
        {//close enough to push
            if ( InFront( NPC->enemy->r.currentOrigin, NPC->client->renderInfo.eyePoint, NPC->client->renderInfo.eyeAngles, 0.6f ) )
            {//knock it down
                WP_KnockdownTurret( NPC, NPC->enemy );
                //do the forcethrow call just for effect
                ForceThrow( NPC, qfalse );
            }
        }
    }
    */
    //rwwFIXMEFIXME: Give them the ability to do this again (turret needs to be fixed up to allow it)
    else if enemy_dist <= 64
        && ((*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0
            || (Q_stricmp(c"Yoda".as_ptr(), (*NPC).NPC_type) == 0 && Q_irand(0, 10) == 0))
    {
        //can't use saber and they're in striking range
        if Q_irand(0, 5) == 0
            && InFront(
                &(*(*NPC).enemy).r.currentOrigin,
                &(*NPC).r.currentOrigin,
                &(*(*NPC).client).ps.viewangles,
                0.2,
            ) != QFALSE
        {
            if ((*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0
                || ((*(*NPC).client).pers.maxHealth - (*NPC).health) as f32
                    > (*(*NPC).client).pers.maxHealth as f32 * 0.25)//lost over 1/4 of our health or not firing
                && (*(*NPC).client).ps.fd.forcePowersKnown & (1 << FP_DRAIN) != 0 //know how to drain
                && WP_ForcePowerAvailable(NPC, FP_DRAIN, 20) != QFALSE//have enough power
                && Q_irand(0, 2) == 0
            {
                //drain
                TIMER_Set(NPC, c"draining".as_ptr(), 3000);
                TIMER_Set(NPC, c"attackDelay".as_ptr(), 3000);
                Jedi_Advance();
                return;
            } else {
                ForceThrow(NPC, QFALSE);
            }
        }
        Jedi_Retreat();
    } else if enemy_dist <= 64
        && ((*(*NPC).client).pers.maxHealth - (*NPC).health) as f32
            > (*(*NPC).client).pers.maxHealth as f32 * 0.25//lost over 1/4 of our health
        && (*(*NPC).client).ps.fd.forcePowersKnown & (1 << FP_DRAIN) != 0 //know how to drain
        && WP_ForcePowerAvailable(NPC, FP_DRAIN, 20) != QFALSE//have enough power
        && Q_irand(0, 10) == 0
        && InFront(
            &(*(*NPC).enemy).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &(*(*NPC).client).ps.viewangles,
            0.2,
        ) != QFALSE
    {
        TIMER_Set(NPC, c"draining".as_ptr(), 3000);
        TIMER_Set(NPC, c"attackDelay".as_ptr(), 3000);
        Jedi_Advance();
        return;
    } else if enemy_dist <= -16 {
        //we're too damn close!
        Jedi_Retreat();
    } else if enemy_dist <= 0 {
        //we're within striking range
        //if we are attacking, see if we should stop
        if (*NPCInfo).stats.aggression < 4 {
            //back off and defend
            Jedi_Retreat();
        }
    } else if enemy_dist > 256 {
        //we're way out of range
        let mut usedForce: qboolean = QFALSE;
        if (*NPCInfo).stats.aggression < Q_irand(0, 20)
            && ((*NPC).health as f32) < (*(*NPC).client).pers.maxHealth as f32 * 0.75
            && Q_irand(0, 2) == 0
        {
            if (*(*NPC).client).ps.fd.forcePowersKnown & (1 << FP_HEAL) != 0
                && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_HEAL) == 0
                && Q_irand(0, 1) != 0
            {
                ForceHeal(NPC);
                usedForce = QTRUE;
                //FIXME: check level of heal and know not to move or attack when healing
            } else if (*(*NPC).client).ps.fd.forcePowersKnown & (1 << FP_PROTECT) != 0
                && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_PROTECT) == 0
                && Q_irand(0, 1) != 0
            {
                ForceProtect(NPC);
                usedForce = QTRUE;
            } else if (*(*NPC).client).ps.fd.forcePowersKnown & (1 << FP_ABSORB) != 0
                && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_ABSORB) == 0
                && Q_irand(0, 1) != 0
            {
                ForceAbsorb(NPC);
                usedForce = QTRUE;
            } else if (*(*NPC).client).ps.fd.forcePowersKnown & (1 << FP_RAGE) != 0
                && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_RAGE) == 0
                && Q_irand(0, 1) != 0
            {
                Jedi_Rage();
                usedForce = QTRUE;
            }
            //FIXME: what about things like mind tricks and force sight?
        }
        if enemy_dist > 384 {
            //FIXME: check for enemy facing away and/or moving away
            if Q_irand(0, 10) == 0
                && (*NPCInfo).blockedSpeechDebounceTime < (*addr_of!(level)).time
                && jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize]
                    < (*addr_of!(level)).time
            {
                if NPC_ClearLOS4((*NPC).enemy) != QFALSE {
                    G_AddVoiceEvent(NPC, Q_irand(EV_JCHASE1, EV_JCHASE3), 3000);
                }
                jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize] =
                    (*addr_of!(level)).time + 3000;
                (*NPCInfo).blockedSpeechDebounceTime = (*addr_of!(level)).time + 3000;
            }
        }
        //Unless we're totally hiding, go after him
        if (*NPCInfo).stats.aggression > 0 {
            //approach enemy
            if usedForce == QFALSE {
                Jedi_Advance();
            }
        }
    }
    /*
    else if ( enemy_dist < 96 && NPC->enemy && NPC->enemy->client && NPC->enemy->client->ps.groundEntityNum == ENTITYNUM_NONE )
    {//too close and in air, so retreat
        Jedi_Retreat();
    }
    */
    //FIXME: enemy_dist calc needs to include all blade lengths, and include distance from hand to start of blade....
    else if enemy_dist > 50 {
        //FIXME: not hardcoded- base on our reach (modelScale?) and saberLengthMax
        //we're out of striking range and we are allowed to attack
        //first, check some tactical force power decisions
        if !(*NPC).enemy.is_null()
            && !(*(*NPC).enemy).client.is_null()
            && (*(*(*NPC).enemy).client).ps.fd.forceGripBeingGripped > (*addr_of!(level)).time as f32
        {
            //They're being gripped, rush them!
            if (*(*(*NPC).enemy).client).ps.groundEntityNum != ENTITYNUM_NONE {
                //they're on the ground, so advance
                if TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE || (*NPCInfo).rank > RANK_LT {
                    //not parrying
                    if enemy_dist > 200 || (*NPCInfo).scriptFlags & SCF_DONT_FIRE == 0 {
                        //far away or allowed to use saber
                        Jedi_Advance();
                    }
                }
            }
            if (*NPCInfo).rank >= RANK_LT_JG
                && Q_irand(0, 5) == 0
                && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_SPEED) == 0
                && (*(*NPC).client).ps.saberEventFlags & SEF_INWATER == 0
            {
                //saber not in water
                //throw saber
                ucmd.buttons |= BUTTON_ALT_ATTACK;
            }
        } else if !(*NPC).enemy.is_null()
            && !(*(*NPC).enemy).client.is_null() //valid enemy
            && (*(*(*NPC).enemy).client).ps.saberInFlight != QFALSE /*NPC->enemy->client->ps.saber[0].Active()*/
            && (*(*(*NPC).enemy).client).ps.saberEntityNum != 0 //enemy throwing saber
            && (*(*NPC).client).ps.weaponTime <= 0 //I'm not busy
            && WP_ForcePowerAvailable(NPC, FP_GRIP, 0) != QFALSE //I can use the power
            && Q_irand(0, 10) == 0 //don't do it all the time, averages to 1 check a second
            && Q_irand(0, 6) < g_spskill.integer //more likely on harder diff
            && Q_irand(RANK_CIVILIAN, RANK_CAPTAIN) < (*NPCInfo).rank
        {
            //more likely against harder enemies
            //They're throwing their saber, grip them!
            //taunt
            if TIMER_Done(NPC, c"chatter".as_ptr()) != QFALSE
                && jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize]
                    < (*addr_of!(level)).time
                && (*NPCInfo).blockedSpeechDebounceTime < (*addr_of!(level)).time
            {
                G_AddVoiceEvent(NPC, Q_irand(EV_TAUNT1, EV_TAUNT3), 3000);
                jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize] =
                    (*addr_of!(level)).time + 3000;
                (*NPCInfo).blockedSpeechDebounceTime = (*addr_of!(level)).time + 3000;
                TIMER_Set(NPC, c"chatter".as_ptr(), 3000);
            }

            //grip
            TIMER_Set(NPC, c"gripping".as_ptr(), 3000);
            TIMER_Set(NPC, c"attackDelay".as_ptr(), 3000);
        } else {
            let mut chanceScale: c_int;

            if !(*NPC).enemy.is_null()
                && !(*(*NPC).enemy).client.is_null()
                && (*(*(*NPC).enemy).client).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0
            {
                //They're choking someone, probably an ally, run at them and do some sort of attack
                if (*(*(*NPC).enemy).client).ps.groundEntityNum != ENTITYNUM_NONE {
                    //they're on the ground, so advance
                    if TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE || (*NPCInfo).rank > RANK_LT
                    {
                        //not parrying
                        if enemy_dist > 200 || (*NPCInfo).scriptFlags & SCF_DONT_FIRE == 0 {
                            //far away or allowed to use saber
                            Jedi_Advance();
                        }
                    }
                }
            }
            chanceScale = 0;
            if (*(*NPC).client).NPC_class == CLASS_DESANN
                || Q_stricmp(c"Yoda".as_ptr(), (*NPC).NPC_type) == 0
            {
                chanceScale = 1;
            } else if (*NPCInfo).rank == RANK_ENSIGN {
                chanceScale = 2;
            } else if (*NPCInfo).rank >= RANK_LT_JG {
                chanceScale = 5;
            }
            if chanceScale != 0
                && (enemy_dist > Q_irand(100, 200)
                    || (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0
                    || (Q_stricmp(c"Yoda".as_ptr(), (*NPC).NPC_type) == 0 && Q_irand(0, 3) == 0))
                && enemy_dist < 500
                && (Q_irand(0, chanceScale * 10) < 5
                    || (!(*(*NPC).enemy).client.is_null()
                        && (*(*(*NPC).enemy).client).ps.weapon != WP_SABER
                        && Q_irand(0, chanceScale) == 0))
            {
                //else, randomly try some kind of attack every now and then
                if ((*NPCInfo).rank == RANK_ENSIGN || (*NPCInfo).rank > RANK_LT_JG)
                    && Q_irand(0, 1) == 0
                {
                    if WP_ForcePowerAvailable(NPC, FP_PULL, 0) != QFALSE && Q_irand(0, 2) == 0 {
                        //force pull the guy to me!
                        //FIXME: check forcePushRadius[NPC->client->ps.fd.forcePowerLevel[FP_PUSH]]
                        ForceThrow(NPC, QTRUE);
                        //maybe strafe too?
                        TIMER_Set(NPC, c"duck".as_ptr(), enemy_dist * 3);
                        if Q_irand(0, 1) != 0 {
                            ucmd.buttons |= BUTTON_ATTACK;
                        }
                    } else if WP_ForcePowerAvailable(NPC, FP_LIGHTNING, 0) != QFALSE
                        && Q_irand(0, 1) != 0
                    {
                        ForceLightning(NPC);
                        if (*(*NPC).client).ps.fd.forcePowerLevel[FP_LIGHTNING as usize]
                            > FORCE_LEVEL_1
                        {
                            (*(*NPC).client).ps.weaponTime =
                                Q_irand(1000, 3000 + (g_spskill.integer * 500));
                            TIMER_Set(
                                NPC,
                                c"holdLightning".as_ptr(),
                                (*(*NPC).client).ps.weaponTime,
                            );
                        }
                        TIMER_Set(NPC, c"attackDelay".as_ptr(), (*(*NPC).client).ps.weaponTime);
                    }
                    /*else if ( NPC->health < NPC->client->pers.maxHealth * 0.75f
                        && Q_irand( FORCE_LEVEL_0, NPC->client->ps.fd.forcePowerLevel[FP_DRAIN] ) > FORCE_LEVEL_1
                        && WP_ForcePowerAvailable( NPC, FP_DRAIN, 0 )
                        && Q_irand( 0, 1 ) )
                    {
                        ForceDrain2( NPC );
                        NPC->client->ps.weaponTime = Q_irand( 1000, 3000+(g_spskill.integer*500) );
                        TIMER_Set( NPC, "draining", NPC->client->ps.weaponTime );
                        TIMER_Set( NPC, "attackDelay", NPC->client->ps.weaponTime );
                    }*/
                    //rwwFIXMEFIXME: After new drain stuff from SP is in re-enable this.
                    else if WP_ForcePowerAvailable(NPC, FP_GRIP, 0) != QFALSE {
                        //taunt
                        if TIMER_Done(NPC, c"chatter".as_ptr()) != QFALSE
                            && jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize]
                                < (*addr_of!(level)).time
                            && (*NPCInfo).blockedSpeechDebounceTime < (*addr_of!(level)).time
                        {
                            G_AddVoiceEvent(NPC, Q_irand(EV_TAUNT1, EV_TAUNT3), 3000);
                            jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize] =
                                (*addr_of!(level)).time + 3000;
                            (*NPCInfo).blockedSpeechDebounceTime = (*addr_of!(level)).time + 3000;
                            TIMER_Set(NPC, c"chatter".as_ptr(), 3000);
                        }

                        //grip
                        TIMER_Set(NPC, c"gripping".as_ptr(), 3000);
                        TIMER_Set(NPC, c"attackDelay".as_ptr(), 3000);
                    } else if WP_ForcePowerAvailable(NPC, FP_SABERTHROW, 0) != QFALSE
                        && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_SPEED) == 0
                        && (*(*NPC).client).ps.saberEventFlags & SEF_INWATER == 0
                    {
                        //saber not in water
                        //throw saber
                        ucmd.buttons |= BUTTON_ALT_ATTACK;
                    }
                } else if (*NPCInfo).rank >= RANK_LT_JG
                    && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_SPEED) == 0
                    && (*(*NPC).client).ps.saberEventFlags & SEF_INWATER == 0
                {
                    //saber not in water
                    //throw saber
                    ucmd.buttons |= BUTTON_ALT_ATTACK;
                }
            }
            //see if we should advance now
            else if (*NPCInfo).stats.aggression > 5 {
                //approach enemy
                if TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE || (*NPCInfo).rank > RANK_LT {
                    //not parrying
                    if (*(*NPC).enemy).client.is_null()
                        || (*(*(*NPC).enemy).client).ps.groundEntityNum != ENTITYNUM_NONE
                    {
                        //they're on the ground, so advance
                        if enemy_dist > 200 || (*NPCInfo).scriptFlags & SCF_DONT_FIRE == 0 {
                            //far away or allowed to use saber
                            Jedi_Advance();
                        }
                    }
                }
            } else {
                //maintain this distance?
                //walk?
            }
            // faithful: C declares `chanceScale` mutable but the only writes are the
            // ladder above; silence the unused-write lint for the dead final read.
            let _ = &mut chanceScale;
        }
    } else {
        //we're not close enough to attack, but not far enough away to be safe
        if (*NPCInfo).stats.aggression < 4 {
            //back off and defend
            Jedi_Retreat();
        } else if (*NPCInfo).stats.aggression > 5 {
            //try to get closer
            if enemy_dist > 0 && (*NPCInfo).scriptFlags & SCF_DONT_FIRE == 0 {
                //we're allowed to use our lightsaber, get closer
                if TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE || (*NPCInfo).rank > RANK_LT {
                    //not parrying
                    if (*(*NPC).enemy).client.is_null()
                        || (*(*(*NPC).enemy).client).ps.groundEntityNum != ENTITYNUM_NONE
                    {
                        //they're on the ground, so advance
                        Jedi_Advance();
                    }
                }
            }
        } else {
            //agression is 4 or 5... somewhere in the middle
            //what do we do here?  Nothing?
            //Move forward and back?
        }
    }
    //if really really mad, rage!
    if (*NPCInfo).stats.aggression > Q_irand(5, 15)
        && ((*NPC).health as f32) < (*(*NPC).client).pers.maxHealth as f32 * 0.75
        && Q_irand(0, 2) == 0
    {
        if (*(*NPC).client).ps.fd.forcePowersKnown & (1 << FP_RAGE) != 0
            && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_RAGE) == 0
        {
            Jedi_Rage();
        }
    }
}

/// `static void Jedi_AdjustSaberAnimLevel( gentity_t *self, int newLevel )`
/// (NPC_AI_Jedi.c:1327). Picks the Jedi's saber attack level: fixed for
/// Tavion/Desann and the low enemy ranks, otherwise `newLevel` clamped to the
/// `[FORCE_LEVEL_1, FP_SABER_OFFENSE]` range. The `d_JediAI` debug print is carried
/// over. No oracle (entity-state + debug print).
///
/// # Safety
/// `self` may be NULL (guarded); `self->client`/`self->NPC` must be valid when
/// reached.
#[allow(dead_code)] // used by Jedi_CheckDecreaseSaberAnimLevel etc. as they land
unsafe fn Jedi_AdjustSaberAnimLevel(self_: *mut gentity_t, newLevel: c_int) {
    if self_.is_null() || (*self_).client.is_null() {
        return;
    }
    //FIXME: each NPC shold have a unique pattern of behavior for the order in which they
    if (*(*self_).client).NPC_class == CLASS_TAVION {
        //special attacks
        (*(*self_).client).ps.fd.saberAnimLevel = FORCE_LEVEL_5;
        return;
    } else if (*(*self_).client).NPC_class == CLASS_DESANN {
        //special attacks
        (*(*self_).client).ps.fd.saberAnimLevel = FORCE_LEVEL_4;
        return;
    }
    if (*(*self_).client).playerTeam == NPCTEAM_ENEMY {
        if (*(*self_).NPC).rank == RANK_CIVILIAN || (*(*self_).NPC).rank == RANK_LT_JG {
            //grunt and fencer always uses quick attacks
            (*(*self_).client).ps.fd.saberAnimLevel = FORCE_LEVEL_1;
            return;
        }
        if (*(*self_).NPC).rank == RANK_CREWMAN || (*(*self_).NPC).rank == RANK_ENSIGN {
            //acrobat & force-users always use medium attacks
            (*(*self_).client).ps.fd.saberAnimLevel = FORCE_LEVEL_2;
            return;
        }
        /*
        if ( self->NPC->rank == RANK_LT )
        {//boss always uses strong attacks
            self->client->ps.fd.saberAnimLevel = FORCE_LEVEL_3;
            return;
        }
        */
    }
    //use the different attacks, how often they switch and under what circumstances
    if newLevel > (*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] {
        //cap it
        (*(*self_).client).ps.fd.saberAnimLevel =
            (*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize];
    } else if newLevel < FORCE_LEVEL_1 {
        (*(*self_).client).ps.fd.saberAnimLevel = FORCE_LEVEL_1;
    } else {
        //go ahead and set it
        (*(*self_).client).ps.fd.saberAnimLevel = newLevel;
    }

    if (*addr_of!(d_JediAI)).integer != 0 {
        match (*(*self_).client).ps.fd.saberAnimLevel {
            FORCE_LEVEL_1 => {
                Com_Printf(&format!(
                    "^2{} Saber Attack Set: fast\n",
                    CStr::from_ptr((*self_).NPC_type).to_string_lossy()
                ));
            }
            FORCE_LEVEL_2 => {
                Com_Printf(&format!(
                    "^3{} Saber Attack Set: medium\n",
                    CStr::from_ptr((*self_).NPC_type).to_string_lossy()
                ));
            }
            FORCE_LEVEL_3 => {
                Com_Printf(&format!(
                    "^1{} Saber Attack Set: strong\n",
                    CStr::from_ptr((*self_).NPC_type).to_string_lossy()
                ));
            }
            _ => {}
        }
    }
}

/// `static void Jedi_CheckDecreaseSaberAnimLevel( void )` (NPC_AI_Jedi.c:1396).
/// When not attacking, occasionally randomizes the saber attack level on a
/// debounce; when attacking, just re-arms the debounce. No oracle (NPC-global
/// entity-state + timers).
///
/// # Safety
/// The `NPC` global and `NPC->client` must be valid.
#[allow(dead_code)] // used by Jedi behavior siblings as they land
unsafe fn Jedi_CheckDecreaseSaberAnimLevel() {
    if (*(*NPC).client).ps.weaponTime == 0
        && (ucmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) == 0
    {
        //not attacking
        if TIMER_Done(NPC, c"saberLevelDebounce".as_ptr()) != QFALSE && Q_irand(0, 10) == 0 {
            //Jedi_AdjustSaberAnimLevel( NPC, (NPC->client->ps.fd.saberAnimLevel-1) );//drop
            Jedi_AdjustSaberAnimLevel(NPC, Q_irand(FORCE_LEVEL_1, FORCE_LEVEL_3)); //random
            TIMER_Set(NPC, c"saberLevelDebounce".as_ptr(), Q_irand(3000, 10000));
        }
    } else {
        TIMER_Set(NPC, c"saberLevelDebounce".as_ptr(), Q_irand(1000, 5000));
    }
}

/// `evasionType_t Jedi_CheckFlipEvasions( gentity_t *self, float rightdot, float zdiff )`
/// (NPC_AI_Jedi.c:1969). Acrobatic dodges: if already wall-running, flip off the wall
/// away from the attack; otherwise (rank-gated, non-Desann) try an arial/cartwheel to
/// the side if clear, and failing that turn it into a wall-flip or wall-run off a
/// nearby wall. Returns which evasion was performed (or `EVASION_NONE`). No oracle
/// (NPC entity-state + trap_Trace + anims).
///
/// # Safety
/// `self` must be valid; `self->NPC`/`self->client` are guarded where the C guards
/// them (and dereferenced where the C dereferences them unconditionally).
#[allow(dead_code)] // sole caller Jedi_EvasionSaber is blocked on the saber-evasion AI core
pub unsafe fn Jedi_CheckFlipEvasions(
    self_: *mut gentity_t,
    rightdot: f32,
    _zdiff: f32,
) -> evasionType_t {
    if !(*self_).NPC.is_null() && (*(*self_).NPC).scriptFlags & SCF_NO_ACROBATICS != 0 {
        return EVASION_NONE;
    }
    if !(*self_).client.is_null()
        && ((*(*self_).client).ps.fd.forceRageRecoveryTime > (*addr_of!(level)).time
            || (*(*self_).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0)
    {
        //no fancy dodges when raging
        return EVASION_NONE;
    }
    //Check for:
    //ARIALS/CARTWHEELS
    //WALL-RUNS
    //WALL-FLIPS
    //FIXME: if facing a wall, do forward wall-walk-backflip
    if (*(*self_).client).ps.legsAnim == BOTH_WALL_RUN_LEFT
        || (*(*self_).client).ps.legsAnim == BOTH_WALL_RUN_RIGHT
    {
        //already running on a wall
        let mut right: vec3_t = [0.0; 3];
        let mut fwdAngles: vec3_t = [0.0; 3];
        let mut anim: c_int = -1;
        let animLength: c_int;

        VectorSet(&mut fwdAngles, 0.0, (*(*self_).client).ps.viewangles[YAW], 0.0);

        AngleVectors(&fwdAngles, None, Some(&mut right), None);

        animLength = BG_AnimLength(
            (*self_).localAnimIndex,
            (*(*self_).client).ps.legsAnim as animNumber_t,
        );
        if (*(*self_).client).ps.legsAnim == BOTH_WALL_RUN_LEFT && rightdot < 0.0 {
            //I'm running on a wall to my left and the attack is on the left
            if animLength - (*(*self_).client).ps.legsTimer > 400
                && (*(*self_).client).ps.legsTimer > 400
            {
                //not at the beginning or end of the anim
                anim = BOTH_WALL_RUN_LEFT_FLIP;
            }
        } else if (*(*self_).client).ps.legsAnim == BOTH_WALL_RUN_RIGHT && rightdot > 0.0 {
            //I'm running on a wall to my right and the attack is on the right
            if animLength - (*(*self_).client).ps.legsTimer > 400
                && (*(*self_).client).ps.legsTimer > 400
            {
                //not at the beginning or end of the anim
                anim = BOTH_WALL_RUN_RIGHT_FLIP;
            }
        }
        if anim != -1 {
            //flip off the wall!
            let parts: c_int;
            //FIXME: check the direction we will flip towards for do-not-enter/walls/drops?
            //NOTE: we presume there is still a wall there!
            if anim == BOTH_WALL_RUN_LEFT_FLIP {
                (*(*self_).client).ps.velocity[0] *= 0.5;
                (*(*self_).client).ps.velocity[1] *= 0.5;
                let vel = (*(*self_).client).ps.velocity;
                VectorMA(&vel, 150.0, &right, &mut (*(*self_).client).ps.velocity);
            } else if anim == BOTH_WALL_RUN_RIGHT_FLIP {
                (*(*self_).client).ps.velocity[0] *= 0.5;
                (*(*self_).client).ps.velocity[1] *= 0.5;
                let vel = (*(*self_).client).ps.velocity;
                VectorMA(&vel, -150.0, &right, &mut (*(*self_).client).ps.velocity);
            }
            parts = if (*(*self_).client).ps.weaponTime == 0 {
                SETANIM_BOTH
            } else {
                SETANIM_LEGS
            };
            NPC_SetAnim(
                self_,
                parts,
                anim,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            //self->client->ps.pm_flags |= (PMF_JUMPING|PMF_SLOW_MO_FALL);
            //rwwFIXMEFIXME: Add these pm flags?
            G_AddEvent(self_, EV_JUMP, 0);
            return EVASION_OTHER;
        }
    } else if (*(*self_).client).NPC_class != CLASS_DESANN //desann doesn't do these kind of frilly acrobatics
        && ((*(*self_).NPC).rank == RANK_CREWMAN || (*(*self_).NPC).rank >= RANK_LT)
        && Q_irand(0, 1) != 0
        && BG_InRoll(&mut (*(*self_).client).ps, (*(*self_).client).ps.legsAnim) == QFALSE
        && PM_InKnockDown(&mut (*(*self_).client).ps) == QFALSE
        && BG_SaberInSpecialAttack((*(*self_).client).ps.torsoAnim) == QFALSE
    {
        let mut fwd: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut traceto: vec3_t = [0.0; 3];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut fwdAngles: vec3_t = [0.0; 3];
        let mut trace: trace_t;
        let mut parts: c_int;
        let mut anim: c_int;
        let speed: f32;
        let mut checkDist: f32;
        let mut allowCartWheels: bool = true;
        let mut allowWallFlips: bool = true;

        if (*(*self_).client).ps.weapon == WP_SABER {
            if (*(*self_).client).saber[0].model[0] != 0
                && (*(*self_).client).saber[0].saberFlags & SFL_NO_CARTWHEELS != 0
            {
                allowCartWheels = false;
            } else if (*(*self_).client).saber[1].model[0] != 0
                && (*(*self_).client).saber[1].saberFlags & SFL_NO_CARTWHEELS != 0
            {
                allowCartWheels = false;
            }
            if (*(*self_).client).saber[0].model[0] != 0
                && (*(*self_).client).saber[0].saberFlags & SFL_NO_WALL_FLIPS != 0
            {
                allowWallFlips = false;
            } else if (*(*self_).client).saber[1].model[0] != 0
                && (*(*self_).client).saber[1].saberFlags & SFL_NO_WALL_FLIPS != 0
            {
                allowWallFlips = false;
            }
        }

        VectorSet(&mut mins, (*self_).r.mins[0], (*self_).r.mins[1], 0.0);
        VectorSet(&mut maxs, (*self_).r.maxs[0], (*self_).r.maxs[1], 24.0);
        VectorSet(&mut fwdAngles, 0.0, (*(*self_).client).ps.viewangles[YAW], 0.0);

        AngleVectors(&fwdAngles, Some(&mut fwd), Some(&mut right), None);

        parts = SETANIM_BOTH;

        if BG_SaberInAttack((*(*self_).client).ps.saberMove) != QFALSE
            || PM_SaberInStart((*(*self_).client).ps.saberMove) != QFALSE
        {
            parts = SETANIM_LEGS;
        }
        if rightdot >= 0.0 {
            if Q_irand(0, 1) != 0 {
                anim = BOTH_ARIAL_LEFT;
            } else {
                anim = BOTH_CARTWHEEL_LEFT;
            }
            checkDist = -128.0;
            speed = -200.0;
        } else {
            if Q_irand(0, 1) != 0 {
                anim = BOTH_ARIAL_RIGHT;
            } else {
                anim = BOTH_CARTWHEEL_RIGHT;
            }
            checkDist = 128.0;
            speed = 200.0;
        }
        //trace in the dir that we want to go
        VectorMA(&(*self_).r.currentOrigin, checkDist, &right, &mut traceto);
        trace = trap::Trace(
            &(*self_).r.currentOrigin,
            &mins,
            &maxs,
            &traceto,
            (*self_).s.number,
            CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
        );
        if trace.fraction >= 1.0 && allowCartWheels {
            //it's clear, let's do it
            //FIXME: check for drops?
            let mut fwdAngles2: vec3_t = [0.0; 3];
            let mut jumpRt: vec3_t = [0.0; 3];

            NPC_SetAnim(
                self_,
                parts,
                anim,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            (*(*self_).client).ps.weaponTime = (*(*self_).client).ps.legsTimer; //don't attack again until this anim is done
            VectorCopy(&(*(*self_).client).ps.viewangles, &mut fwdAngles2);
            fwdAngles2[PITCH as usize] = 0.0;
            fwdAngles2[ROLL as usize] = 0.0;
            //do the flip
            AngleVectors(&fwdAngles2, None, Some(&mut jumpRt), None);
            VectorScale(&jumpRt, speed, &mut (*(*self_).client).ps.velocity);
            (*(*self_).client).ps.fd.forceJumpCharge = 0.0; //so we don't play the force flip anim
            (*(*self_).client).ps.velocity[2] = 200.0;
            (*(*self_).client).ps.fd.forceJumpZStart = (*self_).r.currentOrigin[2]; //so we don't take damage if we land at same height
                                                                                    //self->client->ps.pm_flags |= PMF_JUMPING;
            if (*(*self_).client).NPC_class == CLASS_BOBAFETT {
                G_AddEvent(self_, EV_JUMP, 0);
            } else {
                G_SoundOnEnt(self_, CHAN_BODY, "sound/weapons/force/jump.wav");
            }
            //ucmd.upmove = 0;
            return EVASION_CARTWHEEL;
        } else if trace.contents & CONTENTS_BOTCLIP == 0 {
            //hit a wall, not a do-not-enter brush
            //FIXME: before we check any of these jump-type evasions, we should check for headroom, right?
            //Okay, see if we can flip *off* the wall and go the other way
            let mut idealNormal: vec3_t = [0.0; 3];
            let traceEnt: *mut gentity_t;

            VectorSubtract(&(*self_).r.currentOrigin, &traceto, &mut idealNormal);
            VectorNormalize(&mut idealNormal);
            traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(trace.entityNum as usize);
            if ((trace.entityNum as c_int) < ENTITYNUM_WORLD
                && !traceEnt.is_null()
                && (*traceEnt).s.solid != SOLID_BMODEL)
                || DotProduct(&trace.plane.normal, &idealNormal) > 0.7
            {
                //it's a ent of some sort or it's a wall roughly facing us
                let mut bestCheckDist: f32 = 0.0;
                //hmm, see if we're moving forward
                if DotProduct(&(*(*self_).client).ps.velocity, &fwd) < 200.0 {
                    //not running forward very fast
                    //check to see if it's okay to move the other way
                    if trace.fraction * checkDist <= 32.0 {
                        //wall on that side is close enough to wall-flip off of or wall-run on
                        bestCheckDist = checkDist;
                        checkDist *= -1.0;
                        VectorMA(&(*self_).r.currentOrigin, checkDist, &right, &mut traceto);
                        //trace in the dir that we want to go
                        trace = trap::Trace(
                            &(*self_).r.currentOrigin,
                            &mins,
                            &maxs,
                            &traceto,
                            (*self_).s.number,
                            CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
                        );
                        if trace.fraction >= 1.0 {
                            //it's clear, let's do it
                            if allowWallFlips {
                            //okay to do wall-flips with this saber
                            //FIXME: check for drops?
                            //turn the cartwheel into a wallflip in the other dir
                            if rightdot > 0.0 {
                                anim = BOTH_WALL_FLIP_LEFT;
                                (*(*self_).client).ps.velocity[0] = 0.0;
                                (*(*self_).client).ps.velocity[1] = 0.0;
                                let vel = (*(*self_).client).ps.velocity;
                                VectorMA(&vel, 150.0, &right, &mut (*(*self_).client).ps.velocity);
                            } else {
                                anim = BOTH_WALL_FLIP_RIGHT;
                                (*(*self_).client).ps.velocity[0] = 0.0;
                                (*(*self_).client).ps.velocity[1] = 0.0;
                                let vel = (*(*self_).client).ps.velocity;
                                VectorMA(&vel, -150.0, &right, &mut (*(*self_).client).ps.velocity);
                            }
                            (*(*self_).client).ps.velocity[2] =
                                forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.25;
                            //animate me
                            parts = if (*(*self_).client).ps.weaponTime == 0 {
                                SETANIM_BOTH
                            } else {
                                SETANIM_LEGS
                            };
                            NPC_SetAnim(
                                self_,
                                parts,
                                anim,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                            (*(*self_).client).ps.fd.forceJumpZStart = (*self_).r.currentOrigin[2]; //so we don't take damage if we land at same height
                                                                                                    //self->client->ps.pm_flags |= (PMF_JUMPING|PMF_SLOW_MO_FALL);
                            if (*(*self_).client).NPC_class == CLASS_BOBAFETT {
                                G_AddEvent(self_, EV_JUMP, 0);
                            } else {
                                G_SoundOnEnt(self_, CHAN_BODY, "sound/weapons/force/jump.wav");
                            }
                            return EVASION_OTHER;
                            }
                        } else {
                            //boxed in on both sides
                            if DotProduct(&(*(*self_).client).ps.velocity, &fwd) < 0.0 {
                                //moving backwards
                                return EVASION_NONE;
                            }
                            if trace.fraction * checkDist <= 32.0
                                && trace.fraction * checkDist < bestCheckDist
                            {
                                bestCheckDist = checkDist;
                            }
                        }
                    } else {
                        //too far from that wall to flip or run off it, check other side
                        checkDist *= -1.0;
                        VectorMA(&(*self_).r.currentOrigin, checkDist, &right, &mut traceto);
                        //trace in the dir that we want to go
                        trace = trap::Trace(
                            &(*self_).r.currentOrigin,
                            &mins,
                            &maxs,
                            &traceto,
                            (*self_).s.number,
                            CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
                        );
                        if trace.fraction * checkDist <= 32.0 {
                            //wall on this side is close enough
                            bestCheckDist = checkDist;
                        } else {
                            //neither side has a wall within 32
                            return EVASION_NONE;
                        }
                    }
                }
                //Try wall run?
                if bestCheckDist != 0.0 {
                    //one of the walls was close enough to wall-run on
                    let mut allowWallRuns: bool = true;
                    if (*(*self_).client).ps.weapon == WP_SABER {
                        if (*(*self_).client).saber[0].model[0] != 0
                            && (*(*self_).client).saber[0].saberFlags & SFL_NO_WALL_RUNS != 0
                        {
                            allowWallRuns = false;
                        } else if (*(*self_).client).saber[1].model[0] != 0
                            && (*(*self_).client).saber[1].saberFlags & SFL_NO_WALL_RUNS != 0
                        {
                            allowWallRuns = false;
                        }
                    }
                    if allowWallRuns {
                    //okay to do wallruns with this saber
                    let parts2: c_int;

                    //FIXME: check for long enough wall and a drop at the end?
                    if bestCheckDist > 0.0 {
                        //it was to the right
                        anim = BOTH_WALL_RUN_RIGHT;
                    } else {
                        //it was to the left
                        anim = BOTH_WALL_RUN_LEFT;
                    }
                    (*(*self_).client).ps.velocity[2] =
                        forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.25;
                    //animate me
                    parts2 = if (*(*self_).client).ps.weaponTime == 0 {
                        SETANIM_BOTH
                    } else {
                        SETANIM_LEGS
                    };
                    NPC_SetAnim(
                        self_,
                        parts2,
                        anim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    (*(*self_).client).ps.fd.forceJumpZStart = (*self_).r.currentOrigin[2]; //so we don't take damage if we land at same height
                                                                                            //self->client->ps.pm_flags |= (PMF_JUMPING|PMF_SLOW_MO_FALL);
                    if (*(*self_).client).NPC_class == CLASS_BOBAFETT {
                        G_AddEvent(self_, EV_JUMP, 0);
                    } else {
                        G_SoundOnEnt(self_, CHAN_BODY, "sound/weapons/force/jump.wav");
                    }
                    return EVASION_OTHER;
                    }
                }
                //else check for wall in front, do backflip off wall
            }
        }
    }
    EVASION_NONE
}

/// `int Jedi_ReCalcParryTime( gentity_t *self, evasionType_t evasionType )`
/// (NPC_AI_Jedi.c:2253). Computes how long (ms) the Jedi should hold a parry/evade
/// before re-evaluating, based on entity number (player vs NPC), the realistic-
/// combat / difficulty cvars, NPC class & rank, saber-in-flight state and the kind
/// of evasion. Pure computation (reads entity state + cvars + `bg_parryDebounce`,
/// no traps, no mutation) — oracle-tested.
///
/// # Safety
/// `self` must be valid; `self->client` and (in the NPC branch) `self->NPC` are
/// guarded before deref.
// faithful: the C sets `baseTime = 500/150` before the `switch` overwrites it (dead
// stores) — preserved verbatim.
#[allow(unused_assignments)]
pub unsafe fn Jedi_ReCalcParryTime(
    self_: *mut gentity_t,
    evasionType: evasionType_t,
) -> c_int {
    if (*self_).client.is_null() {
        return 0;
    }
    if (*self_).s.number == 0 {
        //player
        return bg_parryDebounce[(*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize]
            as usize];
    } else if !(*self_).NPC.is_null() {
        if (*addr_of!(g_saberRealisticCombat)).integer == 0
            && ((*addr_of!(g_spskill)).integer == 2
                || ((*addr_of!(g_spskill)).integer == 1
                    && (*(*self_).client).NPC_class == CLASS_TAVION))
        {
            if (*(*self_).client).NPC_class == CLASS_TAVION {
                return 0;
            } else {
                return Q_irand(0, 150);
            }
        } else {
            let mut baseTime: c_int;
            if evasionType == EVASION_DODGE {
                baseTime = (*(*self_).client).ps.torsoTimer;
            } else if evasionType == EVASION_CARTWHEEL {
                baseTime = (*(*self_).client).ps.torsoTimer;
            } else if (*(*self_).client).ps.saberInFlight != QFALSE {
                baseTime = Q_irand(1, 3) * 50;
            } else {
                if (*addr_of!(g_saberRealisticCombat)).integer != 0 {
                    baseTime = 500;

                    match (*addr_of!(g_spskill)).integer {
                        0 => {
                            baseTime = 500;
                        }
                        1 => {
                            baseTime = 300;
                        }
                        2 | _ => {
                            baseTime = 100;
                        }
                    }
                } else {
                    baseTime = 150; //500;

                    match (*addr_of!(g_spskill)).integer {
                        0 => {
                            baseTime = 200; //500;
                        }
                        1 => {
                            baseTime = 100; //300;
                        }
                        2 | _ => {
                            baseTime = 50; //100;
                        }
                    }
                }
                if (*(*self_).client).NPC_class == CLASS_TAVION {
                    //Tavion is faster
                    baseTime = (baseTime as f32 / 2.0f32).ceil() as c_int;
                } else if (*(*self_).NPC).rank >= RANK_LT_JG {
                    //fencers, bosses, shadowtroopers, luke, desann, et al use the norm
                    if Q_irand(0, 2) != 0 {
                        //medium speed parry
                        // C: `baseTime = baseTime;` — a verbatim no-op self-assign, kept
                        // as this comment so the medium-parry branch reads faithfully.
                    } else {
                        //with the occasional fast parry
                        baseTime = (baseTime as f32 / 2.0f32).ceil() as c_int;
                    }
                } else if (*(*self_).NPC).rank == RANK_CIVILIAN {
                    //grunts are slowest
                    baseTime = baseTime * Q_irand(1, 3);
                } else if (*(*self_).NPC).rank == RANK_CREWMAN {
                    //acrobats aren't so bad
                    if evasionType == EVASION_PARRY
                        || evasionType == EVASION_DUCK_PARRY
                        || evasionType == EVASION_JUMP_PARRY
                    {
                        //slower with parries
                        baseTime = baseTime * Q_irand(1, 2);
                    } else {
                        //faster with acrobatics
                        //baseTime = baseTime;
                    }
                } else {
                    //force users are kinda slow
                    baseTime = baseTime * Q_irand(1, 2);
                }
                if evasionType == EVASION_DUCK || evasionType == EVASION_DUCK_PARRY {
                    baseTime += 100;
                } else if evasionType == EVASION_JUMP || evasionType == EVASION_JUMP_PARRY {
                    baseTime += 50;
                } else if evasionType == EVASION_OTHER {
                    baseTime += 100;
                } else if evasionType == EVASION_FJUMP {
                    baseTime += 100;
                }
            }

            return baseTime;
        }
    }
    0
}

/// `qboolean Jedi_QuickReactions( gentity_t *self )` (NPC_AI_Jedi.c:2391). Returns
/// `qtrue` for Jedi who react quickly: the trainer (commander Jedi), Tavion, or a
/// high saber-defense Jedi on a sufficiently high difficulty. No oracle
/// (entity-state + cvar).
///
/// # Safety
/// `self` and `self->client` must be valid; the `NPCInfo` global must be valid.
pub unsafe fn Jedi_QuickReactions(self_: *mut gentity_t) -> qboolean {
    if ((*(*self_).client).NPC_class == CLASS_JEDI && (*NPCInfo).rank == RANK_COMMANDER)
        || (*(*self_).client).NPC_class == CLASS_TAVION
        || ((*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] > FORCE_LEVEL_1
            && (*addr_of!(g_spskill)).integer > 1)
        || ((*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] > FORCE_LEVEL_2
            && (*addr_of!(g_spskill)).integer > 0)
    {
        return QTRUE;
    }
    QFALSE
}

/// `gentity_t *Jedi_FindEnemyInCone( gentity_t *self, gentity_t *fallback, float minDot )`
/// (NPC_AI_Jedi.c:3634). Scans clients in a 2048-cube around `self`, returning the
/// nearest live enemy-team client that is in front (within `minDot`), in PVS, and
/// has a clear shot — else `fallback`. No oracle (trap_EntitiesInBox/InPVS/Trace +
/// global entities). The original's `dist = bestDist` (instead of
/// `bestDist = dist`) is preserved verbatim.
///
/// # Safety
/// `self` must be valid; `g_entities` must be initialised. `self->client` may be
/// NULL (early return).
#[allow(unused_assignments)] // faithful: the C's `dist = bestDist` dead store is preserved
pub unsafe fn Jedi_FindEnemyInCone(
    self_: *mut gentity_t,
    fallback: *mut gentity_t,
    minDot: f32,
) -> *mut gentity_t {
    let mut forward: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut dist: f32;
    let bestDist: f32 = Q3_INFINITE as f32;
    let mut enemy: *mut gentity_t = fallback;
    let mut check: *mut gentity_t;
    let mut entityList: [i32; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
    let numListedEntities: c_int;

    if (*self_).client.is_null() {
        return enemy;
    }

    AngleVectors(
        &(*(*self_).client).ps.viewangles,
        Some(&mut forward),
        None,
        None,
    );

    for e in 0..3 {
        mins[e] = (*self_).r.currentOrigin[e] - 1024.0;
        maxs[e] = (*self_).r.currentOrigin[e] + 1024.0;
    }
    numListedEntities = trap::EntitiesInBox(&mins, &maxs, &mut entityList);

    for e in 0..numListedEntities {
        check = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityList[e as usize] as usize);
        if check == self_ {
            //me
            continue;
        }
        if (*check).inuse == QFALSE {
            //freed
            continue;
        }
        if (*check).client.is_null() {
            //not a client - FIXME: what about turrets?
            continue;
        }
        if (*(*check).client).playerTeam != (*(*self_).client).enemyTeam {
            //not an enemy - FIXME: what about turrets?
            continue;
        }
        if (*check).health <= 0 {
            //dead
            continue;
        }

        if trap::InPVS(&(*check).r.currentOrigin, &(*self_).r.currentOrigin) == QFALSE {
            //can't potentially see them
            continue;
        }

        VectorSubtract(
            &(*check).r.currentOrigin,
            &(*self_).r.currentOrigin,
            &mut dir,
        );
        dist = VectorNormalize(&mut dir);

        if DotProduct(&dir, &forward) < minDot {
            //not in front
            continue;
        }

        //really should have a clear LOS to this thing...
        let tr = trap::Trace(
            &(*self_).r.currentOrigin,
            &vec3_origin,
            &vec3_origin,
            &(*check).r.currentOrigin,
            (*self_).s.number,
            MASK_SHOT,
        );
        if tr.fraction < 1.0 && tr.entityNum as c_int != (*check).s.number {
            //must have clear shot
            continue;
        }

        if dist < bestDist {
            //closer than our last best one
            dist = bestDist; // NOTE: faithful to the C — original sets dist = bestDist, not bestDist = dist
            enemy = check;
        }
    }
    enemy
}

/// `static void Jedi_SetEnemyInfo( vec3_t enemy_dest, vec3_t enemy_dir, float *enemy_dist, vec3_t enemy_movedir, float *enemy_movespeed, int prediction )`
/// (NPC_AI_Jedi.c:3711). Fills in the enemy's predicted position/direction/distance
/// and movement, accounting for a non-client enemy (use raised origin) vs a client
/// enemy (lead by `prediction` ms of velocity, subtract saber blade-0 reach from
/// the distance). No oracle (NPC-global entity-state).
///
/// # Safety
/// The `NPC` global must be valid; the out-params must be valid. Returns early if
/// there is no enemy.
#[allow(dead_code)] // used by Jedi combat siblings as they land
unsafe fn Jedi_SetEnemyInfo(
    enemy_dest: &mut vec3_t,
    enemy_dir: &mut vec3_t,
    enemy_dist: &mut f32,
    enemy_movedir: &mut vec3_t,
    enemy_movespeed: &mut f32,
    prediction: c_int,
) {
    if NPC.is_null() || (*NPC).enemy.is_null() {
        //no valid enemy
        return;
    }
    if (*(*NPC).enemy).client.is_null() {
        VectorClear(enemy_movedir);
        *enemy_movespeed = 0.0;
        VectorCopy(&(*(*NPC).enemy).r.currentOrigin, enemy_dest);
        enemy_dest[2] += (*(*NPC).enemy).r.mins[2] + 24.0; //get it's origin to a height I can work with
        VectorSubtract(enemy_dest, &(*NPC).r.currentOrigin, enemy_dir);
        //FIXME: enemy_dist calc needs to include all blade lengths, and include distance from hand to start of blade....
        *enemy_dist = VectorNormalize(enemy_dir); // - (NPC->client->ps.saberLengthMax + NPC->r.maxs[0]*1.5 + 16);
    } else {
        //see where enemy is headed
        VectorCopy(&(*(*(*NPC).enemy).client).ps.velocity, enemy_movedir);
        *enemy_movespeed = VectorNormalize(enemy_movedir);
        //figure out where he'll be, say, 3 frames from now
        VectorMA(
            &(*(*NPC).enemy).r.currentOrigin,
            *enemy_movespeed * 0.001 * prediction as f32,
            enemy_movedir,
            enemy_dest,
        );
        //figure out what dir the enemy's estimated position is from me and how far from the tip of my saber he is
        VectorSubtract(enemy_dest, &(*NPC).r.currentOrigin, enemy_dir); //NPC->client->renderInfo.muzzlePoint
        //FIXME: enemy_dist calc needs to include all blade lengths, and include distance from hand to start of blade....
        //just use the blade 0 len I guess
        *enemy_dist = VectorNormalize(enemy_dir)
            - ((*(*NPC).client).saber[0].blade[0].lengthMax + (*NPC).r.maxs[0] * 1.5 + 16.0);
        //FIXME: keep a group of enemies around me and use that info to make decisions...
        //		For instance, if there are multiple enemies, evade more, push them away
        //		and use medium attacks.  If enemies are using blasters, switch to fast.
        //		If one jedi enemy, use strong attacks.  Use grip when fighting one or
        //		two enemies, use lightning spread when fighting multiple enemies, etc.
        //		Also, when kill one, check rest of group instead of walking up to victim.
    }
}

/// `static void Jedi_DebounceDirectionChanges( void )` (NPC_AI_Jedi.c:3824).
/// Time-debounces forward/back and right/left command changes so the Jedi doesn't
/// rapidly oscillate its movement direction, re-normalizing the move command to
/// +/-127 as it cancels axes. No oracle (NPC-global entity-state + timers).
///
/// # Safety
/// The `NPC` global and `NPC->client` must be valid.
#[allow(dead_code)] // used by Jedi movement siblings as they land
unsafe fn Jedi_DebounceDirectionChanges() {
    //FIXME: check these before making fwd/back & right/left decisions?
    //Time-debounce changes in forward/back dir
    if ucmd.forwardmove > 0 {
        if TIMER_Done(NPC, c"moveback".as_ptr()) == QFALSE
            || TIMER_Done(NPC, c"movenone".as_ptr()) == QFALSE
        {
            ucmd.forwardmove = 0;
            //now we have to normalize the total movement again
            if ucmd.rightmove > 0 {
                ucmd.rightmove = 127;
            } else if ucmd.rightmove < 0 {
                ucmd.rightmove = -127;
            }
            VectorClear(&mut (*(*NPC).client).ps.moveDir);
            TIMER_Set(NPC, c"moveback".as_ptr(), -(*addr_of!(level)).time);
            if TIMER_Done(NPC, c"movenone".as_ptr()) != QFALSE {
                TIMER_Set(NPC, c"movenone".as_ptr(), Q_irand(1000, 2000));
            }
        } else if TIMER_Done(NPC, c"moveforward".as_ptr()) != QFALSE {
            //FIXME: should be if it's zero?
            TIMER_Set(NPC, c"moveforward".as_ptr(), Q_irand(500, 2000));
        }
    } else if ucmd.forwardmove < 0 {
        if TIMER_Done(NPC, c"moveforward".as_ptr()) == QFALSE
            || TIMER_Done(NPC, c"movenone".as_ptr()) == QFALSE
        {
            ucmd.forwardmove = 0;
            //now we have to normalize the total movement again
            if ucmd.rightmove > 0 {
                ucmd.rightmove = 127;
            } else if ucmd.rightmove < 0 {
                ucmd.rightmove = -127;
            }
            VectorClear(&mut (*(*NPC).client).ps.moveDir);
            TIMER_Set(NPC, c"moveforward".as_ptr(), -(*addr_of!(level)).time);
            if TIMER_Done(NPC, c"movenone".as_ptr()) != QFALSE {
                TIMER_Set(NPC, c"movenone".as_ptr(), Q_irand(1000, 2000));
            }
        } else if TIMER_Done(NPC, c"moveback".as_ptr()) != QFALSE {
            //FIXME: should be if it's zero?
            TIMER_Set(NPC, c"moveback".as_ptr(), Q_irand(250, 1000));
        }
    } else if TIMER_Done(NPC, c"moveforward".as_ptr()) == QFALSE {
        //NOTE: edge checking should stop me if this is bad... but what if it sends us colliding into the enemy?
        ucmd.forwardmove = 127;
        VectorClear(&mut (*(*NPC).client).ps.moveDir);
    } else if TIMER_Done(NPC, c"moveback".as_ptr()) == QFALSE {
        //NOTE: edge checking should stop me if this is bad...
        ucmd.forwardmove = -127;
        VectorClear(&mut (*(*NPC).client).ps.moveDir);
    }
    //Time-debounce changes in right/left dir
    if ucmd.rightmove > 0 {
        if TIMER_Done(NPC, c"moveleft".as_ptr()) == QFALSE
            || TIMER_Done(NPC, c"movecenter".as_ptr()) == QFALSE
        {
            ucmd.rightmove = 0;
            //now we have to normalize the total movement again
            if ucmd.forwardmove > 0 {
                ucmd.forwardmove = 127;
            } else if ucmd.forwardmove < 0 {
                ucmd.forwardmove = -127;
            }
            VectorClear(&mut (*(*NPC).client).ps.moveDir);
            TIMER_Set(NPC, c"moveleft".as_ptr(), -(*addr_of!(level)).time);
            if TIMER_Done(NPC, c"movecenter".as_ptr()) != QFALSE {
                TIMER_Set(NPC, c"movecenter".as_ptr(), Q_irand(1000, 2000));
            }
        } else if TIMER_Done(NPC, c"moveright".as_ptr()) != QFALSE {
            //FIXME: should be if it's zero?
            TIMER_Set(NPC, c"moveright".as_ptr(), Q_irand(250, 1500));
        }
    } else if ucmd.rightmove < 0 {
        if TIMER_Done(NPC, c"moveright".as_ptr()) == QFALSE
            || TIMER_Done(NPC, c"movecenter".as_ptr()) == QFALSE
        {
            ucmd.rightmove = 0;
            //now we have to normalize the total movement again
            if ucmd.forwardmove > 0 {
                ucmd.forwardmove = 127;
            } else if ucmd.forwardmove < 0 {
                ucmd.forwardmove = -127;
            }
            VectorClear(&mut (*(*NPC).client).ps.moveDir);
            TIMER_Set(NPC, c"moveright".as_ptr(), -(*addr_of!(level)).time);
            if TIMER_Done(NPC, c"movecenter".as_ptr()) != QFALSE {
                TIMER_Set(NPC, c"movecenter".as_ptr(), Q_irand(1000, 2000));
            }
        } else if TIMER_Done(NPC, c"moveleft".as_ptr()) != QFALSE {
            //FIXME: should be if it's zero?
            TIMER_Set(NPC, c"moveleft".as_ptr(), Q_irand(250, 1500));
        }
    } else if TIMER_Done(NPC, c"moveright".as_ptr()) == QFALSE {
        //NOTE: edge checking should stop me if this is bad...
        ucmd.rightmove = 127;
        VectorClear(&mut (*(*NPC).client).ps.moveDir);
    } else if TIMER_Done(NPC, c"moveleft".as_ptr()) == QFALSE {
        //NOTE: edge checking should stop me if this is bad...
        ucmd.rightmove = -127;
        VectorClear(&mut (*(*NPC).client).ps.moveDir);
    }
}

/// `static void Jedi_TimersApply( void )` (NPC_AI_Jedi.c:3955). Applies pending
/// strafe timers (unless they fight the desired turn), runs the direction-change
/// debounce, then sets the walking and force-power (grip/drain/lightning) command
/// buttons from their active timers. No oracle (NPC-global entity-state + timers).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->client` must be valid.
#[allow(dead_code)] // used by Jedi behavior siblings as they land
unsafe fn Jedi_TimersApply() {
    if ucmd.rightmove == 0 {
        //only if not already strafing
        //FIXME: if enemy behind me and turning to face enemy, don't strafe in that direction, too
        if TIMER_Done(NPC, c"strafeLeft".as_ptr()) == QFALSE {
            if (*NPCInfo).desiredYaw > (*(*NPC).client).ps.viewangles[YAW] + 60.0 {
                //we want to turn left, don't apply the strafing
            } else {
                //go ahead and strafe left
                ucmd.rightmove = -127;
                VectorClear(&mut (*(*NPC).client).ps.moveDir);
            }
        } else if TIMER_Done(NPC, c"strafeRight".as_ptr()) == QFALSE {
            if (*NPCInfo).desiredYaw < (*(*NPC).client).ps.viewangles[YAW] - 60.0 {
                //we want to turn right, don't apply the strafing
            } else {
                //go ahead and strafe left
                ucmd.rightmove = 127;
                VectorClear(&mut (*(*NPC).client).ps.moveDir);
            }
        }
    }

    Jedi_DebounceDirectionChanges();

    //use careful anim/slower movement if not already moving
    if ucmd.forwardmove == 0 && TIMER_Done(NPC, c"walking".as_ptr()) == QFALSE {
        ucmd.buttons |= BUTTON_WALKING;
    }

    if TIMER_Done(NPC, c"taunting".as_ptr()) == QFALSE {
        ucmd.buttons |= BUTTON_WALKING;
    }

    if TIMER_Done(NPC, c"gripping".as_ptr()) == QFALSE {
        //FIXME: what do we do if we ran out of power?  NPC's can't?
        //FIXME: don't keep turning to face enemy or we'll end up spinning around
        ucmd.buttons |= BUTTON_FORCEGRIP;
    }

    if TIMER_Done(NPC, c"draining".as_ptr()) == QFALSE {
        //FIXME: what do we do if we ran out of power?  NPC's can't?
        //FIXME: don't keep turning to face enemy or we'll end up spinning around
        ucmd.buttons |= BUTTON_FORCE_DRAIN;
    }

    if TIMER_Done(NPC, c"holdLightning".as_ptr()) == QFALSE {
        //hold down the lightning key
        ucmd.buttons |= BUTTON_FORCE_LIGHTNING;
    }
}

/// `qboolean Jedi_SaberBusy( gentity_t *self )` (NPC_AI_Jedi.c:2403). Returns
/// `qtrue` when the Jedi's saber is committed to a long action (strong attack,
/// spin/special attack, broken parry, flip or roll) and so can't parry. No oracle
/// (entity-state + BG anim predicates).
///
/// # Safety
/// `self` and `self->client` must be valid.
pub unsafe fn Jedi_SaberBusy(self_: *mut gentity_t) -> qboolean {
    if (*(*self_).client).ps.torsoTimer > 300
        && ((BG_SaberInAttack((*(*self_).client).ps.saberMove) != QFALSE
            && (*(*self_).client).ps.fd.saberAnimLevel == FORCE_LEVEL_3)
            || BG_SpinningSaberAnim((*(*self_).client).ps.torsoAnim) != QFALSE
            || BG_SaberInSpecialAttack((*(*self_).client).ps.torsoAnim) != QFALSE
            //|| PM_SaberInBounce( self->client->ps.saberMove )
            || PM_SaberInBrokenParry((*(*self_).client).ps.saberMove) != QFALSE
            //|| PM_SaberInDeflect( self->client->ps.saberMove )
            || BG_FlippingAnim((*(*self_).client).ps.torsoAnim) != QFALSE
            || PM_RollingAnim((*(*self_).client).ps.torsoAnim) != QFALSE)
    {
        //my saber is not in a parrying position
        return QTRUE;
    }
    QFALSE
}

/// `static void Jedi_FaceEnemy( qboolean doPitch )` (NPC_AI_Jedi.c:3747). Aims the
/// NPC's desired yaw/pitch at its enemy's head: holds still while gripping, leads
/// the target for Boba Fett's missiles when hurt, points *away* during back-attack
/// anims, else points at it; tilts down a little when the saber is in flight. No
/// oracle (NPC-global entity-state).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals must be valid; returns early if there is no NPC or
/// no enemy.
#[allow(dead_code)] // used by Jedi combat siblings as they land
unsafe fn Jedi_FaceEnemy(doPitch: qboolean) {
    let mut enemy_eyes: vec3_t = [0.0; 3];
    let mut eyes: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];

    if NPC.is_null() {
        return;
    }

    if (*NPC).enemy.is_null() {
        return;
    }

    if (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0
        && (*(*NPC).client).ps.fd.forcePowerLevel[FP_GRIP as usize] > FORCE_LEVEL_1
    {
        //don't update?
        (*NPCInfo).desiredPitch = (*(*NPC).client).ps.viewangles[PITCH];
        (*NPCInfo).desiredYaw = (*(*NPC).client).ps.viewangles[YAW];
        return;
    }
    CalcEntitySpot(NPC, SPOT_HEAD, &mut eyes);

    CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut enemy_eyes);

    if (*(*NPC).client).NPC_class == CLASS_BOBAFETT
        && TIMER_Done(NPC, c"flameTime".as_ptr()) != QFALSE
        && (*NPC).s.weapon != WP_NONE
        && (*NPC).s.weapon != WP_DISRUPTOR
        && ((*NPC).s.weapon != WP_ROCKET_LAUNCHER || (*NPCInfo).scriptFlags & SCF_ALT_FIRE == 0)
        && (*NPC).s.weapon != WP_THERMAL
        && (*NPC).s.weapon != WP_TRIP_MINE
        && (*NPC).s.weapon != WP_DET_PACK
        && (*NPC).s.weapon != WP_STUN_BATON
    /*&& NPC->s.weapon != WP_MELEE*/
    {
        //boba leads his enemy
        if ((*NPC).health as f32) < (*(*NPC).client).pers.maxHealth as f32 * 0.5 {
            //lead
            let missileSpeed = WP_SpeedOfMissileForWeapon(
                (*NPC).s.weapon,
                if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                    QTRUE
                } else {
                    QFALSE
                },
            );
            if missileSpeed != 0.0 {
                let mut eDist = Distance(&eyes, &enemy_eyes);
                eDist /= missileSpeed; //How many seconds it will take to get to the enemy
                let enemy_eyes_copy = enemy_eyes;
                VectorMA(
                    &enemy_eyes_copy,
                    eDist * flrand(0.95, 1.25),
                    &(*(*(*NPC).enemy).client).ps.velocity,
                    &mut enemy_eyes,
                );
            }
        }
    }

    //Find the desired angles
    if (*(*NPC).client).ps.saberInFlight == QFALSE
        && ((*(*NPC).client).ps.legsAnim == BOTH_A2_STABBACK1
            || (*(*NPC).client).ps.legsAnim == BOTH_CROUCHATTACKBACK1
            || (*(*NPC).client).ps.legsAnim == BOTH_ATTACK_BACK)
    {
        //point *away*
        GetAnglesForDirection(&enemy_eyes, &eyes, &mut angles);
    } else {
        //point towards him
        GetAnglesForDirection(&eyes, &enemy_eyes, &mut angles);
    }

    (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW]);
    /*
    if ( NPC->client->ps.saberBlocked == BLOCKED_UPPER_LEFT )
    {//temp hack- to make up for poor coverage on left side
        NPCInfo->desiredYaw += 30;
    }
    */

    if doPitch != QFALSE {
        (*NPCInfo).desiredPitch = AngleNormalize360(angles[PITCH]);
        if (*(*NPC).client).ps.saberInFlight != QFALSE {
            //tilt down a little
            (*NPCInfo).desiredPitch += 10.0;
        }
    }
    //FIXME: else desiredPitch = 0?  Or keep previous?
}

/// `static void Jedi_CombatIdle( int enemy_dist )` (NPC_AI_Jedi.c:4223). When not
/// parrying, throwing the saber, or raging, occasionally flaunts/taunts the enemy
/// based on aggression and distance — sometimes deactivating the saber and dropping
/// aggression to "taunt even more". No oracle (NPC-global entity-state + timers).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->client` must be valid.
#[allow(dead_code)] // used by Jedi behavior siblings as they land
unsafe fn Jedi_CombatIdle(enemy_dist: c_int) {
    if TIMER_Done(NPC, c"parryTime".as_ptr()) == QFALSE {
        return;
    }
    if (*(*NPC).client).ps.saberInFlight != QFALSE {
        //don't do this idle stuff if throwing saber
        return;
    }
    if (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
        || (*(*NPC).client).ps.fd.forceRageRecoveryTime > (*addr_of!(level)).time
    {
        //never taunt while raging or recovering from rage
        return;
    }
    //FIXME: make these distance numbers defines?
    if enemy_dist >= 64 {
        //FIXME: only do this if standing still?
        //based on aggression, flaunt/taunt
        let mut chance: c_int = 20;
        if (*(*NPC).client).NPC_class == CLASS_SHADOWTROOPER {
            chance = 10;
        }
        //FIXME: possibly throw local objects at enemy?
        if Q_irand(2, chance) < (*NPCInfo).stats.aggression {
            if TIMER_Done(NPC, c"chatter".as_ptr()) != QFALSE
                && (*(*NPC).client).ps.forceHandExtend == HANDEXTEND_NONE
            {
                //FIXME: add more taunt behaviors
                //FIXME: sometimes he turns it off, then turns it right back on again???
                if enemy_dist > 200
                    && (*(*NPC).client).NPC_class != CLASS_BOBAFETT
                    && (*(*NPC).client).ps.saberHolstered == 0
                    && Q_irand(0, 5) == 0
                {
                    //taunt even more, turn off the saber
                    //FIXME: don't do this if health low?
                    WP_DeactivateSaber(NPC, QFALSE);
                    //Don't attack for a bit
                    (*NPCInfo).stats.aggression = 3;
                    //FIXME: maybe start strafing?
                    //debounce this
                    if (*(*NPC).client).playerTeam != NPCTEAM_PLAYER && Q_irand(0, 1) == 0 {
                        //NPC->client->ps.taunting = level.time + 100;
                        (*(*NPC).client).ps.forceHandExtend = HANDEXTEND_JEDITAUNT;
                        (*(*NPC).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 5000;

                        TIMER_Set(NPC, c"chatter".as_ptr(), Q_irand(5000, 10000));
                        TIMER_Set(NPC, c"taunting".as_ptr(), 5500);
                    } else {
                        Jedi_BattleTaunt();
                        TIMER_Set(NPC, c"taunting".as_ptr(), Q_irand(5000, 10000));
                    }
                } else if Jedi_BattleTaunt() != QFALSE {
                    //FIXME: pick some anims
                }
            }
        }
    }
}

/// `static qboolean Jedi_Jump( vec3_t dest, int goalEntNum )` (NPC_AI_Jedi.c:4421).
/// Computes a ballistic launch velocity that lobs the NPC onto `dest`, rough-tracing
/// the gravity arc and bumping the launch speed up to 7 times when the path is
/// blocked (using the closest near-miss as a fallback). Writes the result into
/// `NPC->client->ps.velocity`. No oracle (NPC global + trap_Trace).
///
/// The C is `if ( 1 ) { ... } else { ...a more complicated jump... }`: the `else`
/// branch is dead, so only the live arc-throw branch is ported (the commented-out
/// horizontal-jump fast path above it is likewise dead and dropped).
///
/// # Safety
/// The `NPC` global and `NPC->client` must be valid; `dest` must point at a vec3.
#[allow(dead_code)] // only caller is Jedi_TryJump, whose own callers (Jedi_Combat /
                    // NPC_BSJedi_Default) are blocked on the saber-evasion AI core
unsafe fn Jedi_Jump(dest: &vec3_t, goalEntNum: c_int) -> qboolean {
    //FIXME: if land on enemy, knock him down & jump off again
    let mut shotSpeed: f32 = 300.0;
    let mut bestImpactDist: f32 = Q3_INFINITE as f32; //fireSpeed,
    let mut shotVel: vec3_t = [0.0; 3];
    let mut failCase: vec3_t = [0.0; 3];
    let mut trace: trace_t;
    let mut tr: trajectory_t = core::mem::zeroed();
    let mut blocked: qboolean;
    let timeStep: c_int = 500;
    let mut hitCount: c_int = 0;
    let maxHits: c_int = 7;
    let mut lastPos: vec3_t = [0.0; 3];
    let mut testPos: vec3_t = [0.0; 3];
    let mut bottom: vec3_t = [0.0; 3];

    while hitCount < maxHits {
        let mut targetDir: vec3_t = [0.0; 3];
        VectorSubtract(dest, &(*NPC).r.currentOrigin, &mut targetDir);
        let targetDist = VectorNormalize(&mut targetDir);

        VectorScale(&targetDir, shotSpeed, &mut shotVel);
        let mut travelTime = targetDist / shotSpeed;
        shotVel[2] += travelTime * 0.5 * (*(*NPC).client).ps.gravity as f32;

        if hitCount == 0 {
            //save the first one as the worst case scenario
            VectorCopy(&shotVel, &mut failCase);
        }

        //tracePath
        //do a rough trace of the path
        blocked = QFALSE;

        VectorCopy(&(*NPC).r.currentOrigin, &mut tr.trBase);
        VectorCopy(&shotVel, &mut tr.trDelta);
        tr.trType = TR_GRAVITY;
        tr.trTime = (*addr_of!(level)).time;
        travelTime *= 1000.0;
        VectorCopy(&(*NPC).r.currentOrigin, &mut lastPos);

        //This may be kind of wasteful, especially on long throws... use larger steps?  Divide the travelTime into a certain hard number of slices?  Trace just to apex and down?
        let mut elapsedTime = timeStep;
        while elapsedTime < (travelTime.floor() as c_int) + timeStep {
            if elapsedTime as f32 > travelTime {
                //cap it
                elapsedTime = travelTime.floor() as c_int;
            }
            BG_EvaluateTrajectory(&tr, (*addr_of!(level)).time + elapsedTime, &mut testPos);
            if testPos[2] < lastPos[2] {
                //going down, ignore botclip
                trace = trap::Trace(
                    &lastPos,
                    &(*NPC).r.mins,
                    &(*NPC).r.maxs,
                    &testPos,
                    (*NPC).s.number,
                    (*NPC).clipmask,
                );
            } else {
                //going up, check for botclip
                trace = trap::Trace(
                    &lastPos,
                    &(*NPC).r.mins,
                    &(*NPC).r.maxs,
                    &testPos,
                    (*NPC).s.number,
                    (*NPC).clipmask | CONTENTS_BOTCLIP,
                );
            }

            if trace.allsolid != 0 || trace.startsolid != 0 {
                blocked = QTRUE;
                break;
            }
            if trace.fraction < 1.0 {
                //hit something
                if trace.entityNum as c_int == goalEntNum {
                    //hit the enemy, that's perfect!
                    //Hmm, don't want to land on him, though...
                    break;
                } else {
                    if trace.contents & CONTENTS_BOTCLIP != 0 {
                        //hit a do-not-enter brush
                        blocked = QTRUE;
                        break;
                    }
                    if trace.plane.normal[2] > 0.7 && DistanceSquared(&trace.endpos, dest) < 4096.0 {
                        //hit within 64 of desired location, should be okay
                        //close enough!
                        break;
                    } else {
                        //FIXME: maybe find the extents of this brush and go above or below it on next try somehow?
                        let impactDist = DistanceSquared(&trace.endpos, dest);
                        if impactDist < bestImpactDist {
                            bestImpactDist = impactDist;
                            VectorCopy(&shotVel, &mut failCase);
                        }
                        blocked = QTRUE;
                        break;
                    }
                }
            }
            if elapsedTime == travelTime.floor() as c_int {
                //reached end, all clear
                if trace.fraction >= 1.0 {
                    //hmm, make sure we'll land on the ground...
                    //FIXME: do we care how far below ourselves or our dest we'll land?
                    VectorCopy(&trace.endpos, &mut bottom);
                    bottom[2] -= 128.0;
                    trace = trap::Trace(
                        &trace.endpos,
                        &(*NPC).r.mins,
                        &(*NPC).r.maxs,
                        &bottom,
                        (*NPC).s.number,
                        (*NPC).clipmask,
                    );
                    if trace.fraction >= 1.0 {
                        //would fall too far
                        blocked = QTRUE;
                    }
                }
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
            shotSpeed = 300.0 + ((hitCount - 2) * 100) as f32; //from 100 to 900 (skipping 300)
            if hitCount >= 2 {
                //skip 300 since that was the first value we tested
                shotSpeed += 100.0;
            }
        } else {
            //made it!
            break;
        }
    }

    if hitCount >= maxHits {
        //NOTE: worst case scenario, use the one that impacted closest to the target (or just use the first try...?)
        //NOTE: or try failcase?
        VectorCopy(&failCase, &mut (*(*NPC).client).ps.velocity);
    }
    VectorCopy(&shotVel, &mut (*(*NPC).client).ps.velocity);
    QTRUE
}

/// `static qboolean Jedi_TryJump( gentity_t *goal )` (NPC_AI_Jedi.c:4667). Decides
/// whether the NPC should leap toward `goal`: bails on no-acrobatics or an active
/// jump-chase debounce, then (when the goal is grounded and the NPC isn't knocked
/// down / rolling) either pops a small `upmove` hop, walks off for short drops, or —
/// for bigger gaps — picks a landing spot beside the enemy, fakes a force-jump via
/// [`Jedi_Jump`] with the right jump/flip anim and jump sound, and arms the chase &
/// debounce timers. No oracle (NPC/NPCInfo/ucmd globals + trap_Trace).
///
/// # Safety
/// `goal` must be valid; the `NPC`/`NPCInfo`/`ucmd` globals and `NPC->client` must be
/// valid; `goal->client` is guarded before deref.
#[allow(dead_code)] // callers (Jedi_Combat / NPC_BSJedi_Default) are blocked on the
                    // saber-evasion AI core; lands when they do
unsafe fn Jedi_TryJump(goal: *mut gentity_t) -> qboolean {
    //FIXME: never does a simple short, regular jump...
    //FIXME: I need to be on ground too!
    if (*NPCInfo).scriptFlags & SCF_NO_ACROBATICS != 0 {
        return QFALSE;
    }
    if TIMER_Done(NPC, c"jumpChaseDebounce".as_ptr()) != QFALSE {
        if (*goal).client.is_null() || (*(*goal).client).ps.groundEntityNum != ENTITYNUM_NONE {
            if PM_InKnockDown(&mut (*(*NPC).client).ps) == QFALSE
                && BG_InRoll(&mut (*(*NPC).client).ps, (*(*NPC).client).ps.legsAnim) == QFALSE
            {
                //enemy is on terra firma
                let mut goal_diff: vec3_t = [0.0; 3];
                VectorSubtract(&(*goal).r.currentOrigin, &(*NPC).r.currentOrigin, &mut goal_diff);
                let goal_z_diff = goal_diff[2];
                goal_diff[2] = 0.0;
                let goal_xy_dist = VectorNormalize(&mut goal_diff);
                if goal_xy_dist < 550.0 && goal_z_diff > -400.0
                /*was -256*/
                {
                    //for now, jedi don't take falling damage && (NPC->health > 20 || goal_z_diff > 0 ) && (NPC->health >= 100 || goal_z_diff > -128 ))//closer than @512
                    let mut debounce: qboolean = QFALSE;
                    if (*NPC).health < 150
                        && (((*NPC).health < 30 && goal_z_diff < 0.0) || goal_z_diff < -128.0)
                    {
                        //don't jump, just walk off... doesn't help with ledges, though
                        debounce = QTRUE;
                    } else if goal_z_diff < 32.0 && goal_xy_dist < 200.0 {
                        //what is their ideal jump height?
                        ucmd.upmove = 127;
                        debounce = QTRUE;
                    } else {
                        /*
                        //NO!  All Jedi can jump-navigate now...
                        if ( NPCInfo->rank != RANK_CREWMAN && NPCInfo->rank <= RANK_LT_JG )
                        {//can't do acrobatics
                            return qfalse;
                        }
                        */
                        if goal_z_diff > 0.0 || goal_xy_dist > 128.0 {
                            //Fake a force-jump
                            //Screw it, just do my own calc & throw
                            let mut dest: vec3_t = [0.0; 3];
                            VectorCopy(&(*goal).r.currentOrigin, &mut dest);
                            if goal == (*NPC).enemy {
                                let mut sideTry = 0;
                                while sideTry < 10 {
                                    //FIXME: make it so it doesn't try the same spot again?
                                    let trace: trace_t;
                                    let mut bottom: vec3_t = [0.0; 3];

                                    if Q_irand(0, 1) != 0 {
                                        dest[0] += (*(*NPC).enemy).r.maxs[0] * 1.25;
                                    } else {
                                        dest[0] += (*(*NPC).enemy).r.mins[0] * 1.25;
                                    }
                                    if Q_irand(0, 1) != 0 {
                                        dest[1] += (*(*NPC).enemy).r.maxs[1] * 1.25;
                                    } else {
                                        dest[1] += (*(*NPC).enemy).r.mins[1] * 1.25;
                                    }
                                    VectorCopy(&dest, &mut bottom);
                                    bottom[2] -= 128.0;
                                    trace = trap::Trace(
                                        &dest,
                                        &(*NPC).r.mins,
                                        &(*NPC).r.maxs,
                                        &bottom,
                                        (*goal).s.number,
                                        (*NPC).clipmask,
                                    );
                                    if trace.fraction < 1.0 {
                                        //hit floor, okay to land here
                                        break;
                                    }
                                    sideTry += 1;
                                }
                                if sideTry >= 10 {
                                    //screw it, just jump right at him?
                                    VectorCopy(&(*goal).r.currentOrigin, &mut dest);
                                }
                            }
                            if Jedi_Jump(&dest, (*goal).s.number) != QFALSE {
                                //Com_Printf( "(%d) pre-checked force jump\n", level.time );

                                //FIXME: store the dir we;re going in in case something gets in the way of the jump?
                                //? = vectoyaw( NPC->client->ps.velocity );
                                /*
                                if ( NPC->client->ps.velocity[2] < 320 )
                                {
                                    NPC->client->ps.velocity[2] = 320;
                                }
                                else
                                */
                                {
                                    //FIXME: make this a function call
                                    let jumpAnim: c_int;
                                    //FIXME: this should be more intelligent, like the normal force jump anim logic
                                    if (*(*NPC).client).NPC_class == CLASS_BOBAFETT
                                        || ((*NPCInfo).rank != RANK_CREWMAN
                                            && (*NPCInfo).rank <= RANK_LT_JG)
                                    {
                                        //can't do acrobatics
                                        jumpAnim = BOTH_FORCEJUMP1;
                                    } else {
                                        jumpAnim = BOTH_FLIP_F;
                                    }
                                    NPC_SetAnim(
                                        NPC,
                                        SETANIM_BOTH,
                                        jumpAnim,
                                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                    );
                                }

                                (*(*NPC).client).ps.fd.forceJumpZStart = (*NPC).r.currentOrigin[2];
                                //NPC->client->ps.pm_flags |= PMF_JUMPING;

                                (*(*NPC).client).ps.weaponTime = (*(*NPC).client).ps.torsoTimer;
                                (*(*NPC).client).ps.fd.forcePowersActive |= 1 << FP_LEVITATION;
                                if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                                    G_SoundOnEnt(NPC, CHAN_ITEM, "sound/boba/jeton.wav");
                                    (*(*NPC).client).jetPackTime =
                                        (*addr_of!(level)).time + Q_irand(1000, 3000);
                                } else {
                                    G_SoundOnEnt(NPC, CHAN_BODY, "sound/weapons/force/jump.wav");
                                }

                                TIMER_Set(NPC, c"forceJumpChasing".as_ptr(), Q_irand(2000, 3000));
                                debounce = QTRUE;
                            }
                        }
                    }
                    if debounce != QFALSE {
                        //Don't jump again for another 2 to 5 seconds
                        TIMER_Set(NPC, c"jumpChaseDebounce".as_ptr(), Q_irand(2000, 5000));
                        ucmd.forwardmove = 127;
                        VectorClear(&mut (*(*NPC).client).ps.moveDir);
                        TIMER_Set(NPC, c"duck".as_ptr(), -(*addr_of!(level)).time);
                        return QTRUE;
                    }
                }
            }
        }
    }
    QFALSE
}

/// `static qboolean Jedi_Jumping( gentity_t *goal )` (NPC_AI_Jedi.c:4815). While a
/// force-jump-chase is in progress and a goal exists: if landed (back on the ground)
/// it clears the chase timer; otherwise it keeps facing the goal and returns `qtrue`
/// (still in the air). No oracle (NPC global + timers + face). The large commented-out
/// block (the original mid-air push toward the goal) is dead and carried over verbatim
/// as a comment.
///
/// # Safety
/// `goal` may be NULL (guarded); the `NPC` global and `NPC->client` must be valid.
#[allow(dead_code)] // callers (Jedi_Combat / NPC_BSJedi_Default) are blocked on the
                    // saber-evasion AI core; lands when they do
unsafe fn Jedi_Jumping(goal: *mut gentity_t) -> qboolean {
    if TIMER_Done(NPC, c"forceJumpChasing".as_ptr()) == QFALSE && !goal.is_null() {
        //force-jumping at the enemy
        //		if ( !(NPC->client->ps.pm_flags & PMF_JUMPING )//forceJumpZStart )
        //			&& !(NPC->client->ps.pm_flags&PMF_TRIGGER_PUSHED))
        if (*(*NPC).client).ps.groundEntityNum != ENTITYNUM_NONE {
            //rwwFIXMEFIXME: Not sure if this is gonna work, use the PM flags ideally.
            //landed
            TIMER_Set(NPC, c"forceJumpChasing".as_ptr(), 0);
        } else {
            NPC_FaceEntity(goal, QTRUE);
            //FIXME: push me torward where I was heading
            //FIXME: if hit a ledge as soon as we jumped, we need to push toward our goal... must remember original jump dir and/or original jump dest
            /*
            vec3_t	viewangles_xy={0,0,0}, goal_dir, goal_xy_dir, forward, right;
            float	goal_dist;

            //gert horz dir to goal
            VectorSubtract( goal->r.currentOrigin, NPC->r.currentOrigin, goal_dir );
            VectorCopy( goal_dir, goal_xy_dir );
            goal_dist = VectorNormalize( goal_dir );
            goal_xy_dir[2] = 0;
            VectorNormalize( goal_xy_dir );

            //get horz facing
            viewangles_xy[1] = NPC->client->ps.viewangles[1];
            AngleVectors( viewangles_xy, forward, right, NULL );

            //get movement commands to push me toward enemy
            float fDot = DotProduct( forward, goal_dir ) * 127;
            float rDot = DotProduct( right, goal_dir ) * 127;

            ucmd.forwardmove = floor(fDot);
            ucmd.rightmove = floor(rDot);
            ucmd.upmove = 0;//don't duck
            //Cheat:
            if ( goal_dist < 128 && goal->r.currentOrigin[2] > NPC->r.currentOrigin[2] && NPC->client->ps.velocity[2] <= 0 )
            {
                NPC->client->ps.velocity[2] += 320;
            }
            */
            return QTRUE;
        }
    }
    QFALSE
}

/// `static void Jedi_CheckEnemyMovement( float enemy_dist )` (NPC_AI_Jedi.c:4865).
/// Sportsmanship: a non-boss Jedi (not Tavion/Desann/Luke/Yoda) whose enemy is
/// attacking *it* will (rank-gated) freeze in place and stop moving while the enemy
/// flips over it, flips off a wall toward it, or back-stabs from in front — sometimes
/// even sidestepping out of the way via [`G_UcmdMoveForDir`]. No oracle (NPC/ucmd
/// globals + timers).
///
/// # Safety
/// The `NPC`/`NPCInfo`/`ucmd` globals and `NPC->client` must be valid; `NPC->enemy`
/// and `NPC->enemy->client` are guarded before deref.
#[allow(dead_code)] // sole caller Jedi_Combat is blocked on the saber-evasion AI core
unsafe fn Jedi_CheckEnemyMovement(enemy_dist: f32) {
    if (*NPC).enemy.is_null() || (*(*NPC).enemy).client.is_null() {
        return;
    }

    if (*(*NPC).client).NPC_class != CLASS_TAVION
        && (*(*NPC).client).NPC_class != CLASS_DESANN
        && (*(*NPC).client).NPC_class != CLASS_LUKE
        && Q_stricmp(c"Yoda".as_ptr(), (*NPC).NPC_type) != 0
    {
        if !(*(*NPC).enemy).enemy.is_null() && (*(*NPC).enemy).enemy == NPC {
            //enemy is mad at *me*
            if (*(*(*NPC).enemy).client).ps.legsAnim == BOTH_JUMPFLIPSLASHDOWN1
                || (*(*(*NPC).enemy).client).ps.legsAnim == BOTH_JUMPFLIPSTABDOWN
            {
                //enemy is flipping over me
                if Q_irand(0, (*NPCInfo).rank) < RANK_LT {
                    //be nice and stand still for him...
                    ucmd.forwardmove = 0;
                    ucmd.rightmove = 0;
                    ucmd.upmove = 0;
                    VectorClear(&mut (*(*NPC).client).ps.moveDir);
                    (*(*NPC).client).ps.fd.forceJumpCharge = 0.0;
                    TIMER_Set(NPC, c"strafeLeft".as_ptr(), -1);
                    TIMER_Set(NPC, c"strafeRight".as_ptr(), -1);
                    TIMER_Set(NPC, c"noStrafe".as_ptr(), Q_irand(500, 1000));
                    TIMER_Set(NPC, c"movenone".as_ptr(), Q_irand(500, 1000));
                    TIMER_Set(NPC, c"movecenter".as_ptr(), Q_irand(500, 1000));
                }
            } else if (*(*(*NPC).enemy).client).ps.legsAnim == BOTH_WALL_FLIP_BACK1
                || (*(*(*NPC).enemy).client).ps.legsAnim == BOTH_WALL_FLIP_RIGHT
                || (*(*(*NPC).enemy).client).ps.legsAnim == BOTH_WALL_FLIP_LEFT
                || (*(*(*NPC).enemy).client).ps.legsAnim == BOTH_WALL_RUN_LEFT_FLIP
                || (*(*(*NPC).enemy).client).ps.legsAnim == BOTH_WALL_RUN_RIGHT_FLIP
            {
                //he's flipping off a wall
                if (*(*(*NPC).enemy).client).ps.groundEntityNum == ENTITYNUM_NONE {
                    //still in air
                    if enemy_dist < 256.0 {
                        //close
                        if Q_irand(0, (*NPCInfo).rank) < RANK_LT {
                            //be nice and stand still for him...
                            let mut enemyFwd: vec3_t = [0.0; 3];
                            let mut dest: vec3_t = [0.0; 3];
                            let mut dir: vec3_t = [0.0; 3];

                            /*
                            ucmd.forwardmove = ucmd.rightmove = ucmd.upmove = 0;
                            VectorClear( NPC->client->ps.moveDir );
                            NPC->client->ps.fd.forceJumpCharge = 0;
                            TIMER_Set( NPC, "strafeLeft", -1 );
                            TIMER_Set( NPC, "strafeRight", -1 );
                            TIMER_Set( NPC, "noStrafe", Q_irand( 500, 1000 ) );
                            TIMER_Set( NPC, "movenone", Q_irand( 500, 1000 ) );
                            TIMER_Set( NPC, "movecenter", Q_irand( 500, 1000 ) );
                            TIMER_Set( NPC, "noturn", Q_irand( 200, 500 ) );
                            */
                            //stop current movement
                            ucmd.forwardmove = 0;
                            ucmd.rightmove = 0;
                            ucmd.upmove = 0;
                            VectorClear(&mut (*(*NPC).client).ps.moveDir);
                            (*(*NPC).client).ps.fd.forceJumpCharge = 0.0;
                            TIMER_Set(NPC, c"strafeLeft".as_ptr(), -1);
                            TIMER_Set(NPC, c"strafeRight".as_ptr(), -1);
                            TIMER_Set(NPC, c"noStrafe".as_ptr(), Q_irand(500, 1000));
                            TIMER_Set(
                                NPC,
                                c"noturn".as_ptr(),
                                Q_irand(250, 500) * (3 - (*addr_of!(g_spskill)).integer),
                            );

                            VectorCopy(&(*(*(*NPC).enemy).client).ps.velocity, &mut enemyFwd);
                            VectorNormalize(&mut enemyFwd);
                            VectorMA(&(*(*NPC).enemy).r.currentOrigin, -64.0, &enemyFwd, &mut dest);
                            VectorSubtract(&dest, &(*NPC).r.currentOrigin, &mut dir);
                            if VectorNormalize(&mut dir) > 32.0 {
                                G_UcmdMoveForDir(NPC, addr_of_mut!(ucmd), &mut dir);
                            } else {
                                TIMER_Set(NPC, c"movenone".as_ptr(), Q_irand(500, 1000));
                                TIMER_Set(NPC, c"movecenter".as_ptr(), Q_irand(500, 1000));
                            }
                        }
                    }
                }
            } else if (*(*(*NPC).enemy).client).ps.legsAnim == BOTH_A2_STABBACK1 {
                //he's stabbing backwards
                if enemy_dist < 256.0 && enemy_dist > 64.0 {
                    //close
                    if InFront(
                        &(*NPC).r.currentOrigin,
                        &(*(*NPC).enemy).r.currentOrigin,
                        &(*(*NPC).enemy).r.currentAngles,
                        0.0,
                    ) == QFALSE
                    {
                        //behind him
                        if Q_irand(0, (*NPCInfo).rank) == 0 {
                            //be nice and stand still for him...
                            let mut enemyFwd: vec3_t = [0.0; 3];
                            let mut dest: vec3_t = [0.0; 3];
                            let mut dir: vec3_t = [0.0; 3];

                            //stop current movement
                            ucmd.forwardmove = 0;
                            ucmd.rightmove = 0;
                            ucmd.upmove = 0;
                            VectorClear(&mut (*(*NPC).client).ps.moveDir);
                            (*(*NPC).client).ps.fd.forceJumpCharge = 0.0;
                            TIMER_Set(NPC, c"strafeLeft".as_ptr(), -1);
                            TIMER_Set(NPC, c"strafeRight".as_ptr(), -1);
                            TIMER_Set(NPC, c"noStrafe".as_ptr(), Q_irand(500, 1000));

                            AngleVectors(
                                &(*(*NPC).enemy).r.currentAngles,
                                Some(&mut enemyFwd),
                                None,
                                None,
                            );
                            VectorMA(&(*(*NPC).enemy).r.currentOrigin, -32.0, &enemyFwd, &mut dest);
                            VectorSubtract(&dest, &(*NPC).r.currentOrigin, &mut dir);
                            if VectorNormalize(&mut dir) > 64.0 {
                                G_UcmdMoveForDir(NPC, addr_of_mut!(ucmd), &mut dir);
                            } else {
                                TIMER_Set(NPC, c"movenone".as_ptr(), Q_irand(500, 1000));
                                TIMER_Set(NPC, c"movecenter".as_ptr(), Q_irand(500, 1000));
                            }
                        }
                    }
                }
            }
        }
    }
    //FIXME: also:
    //		If enemy doing wall flip, keep running forward
    //		If enemy doing back-attack and we're behind him keep running forward toward his back, don't strafe
}

/// `static void Jedi_CheckJumps( void )` (NPC_AI_Jedi.c:4986). Predicts the arc of an
/// imminent jump (force-jump charge or a plain up-hop) and, if it would slam into a
/// do-not-enter brush or land on breakable glass / drop too far / land nowhere,
/// cancels the jump (zeroes the force-jump charge and `ucmd.upmove`). No oracle
/// (NPC/ucmd globals + trap_Trace).
///
/// The C uses `goto jump_unsafe` for the cancel paths; here those become
/// `break 'jump_unsafe` out of a labeled block, with the cleanup after it (the dead
/// `return;` after each `goto` is dropped).
///
/// # Safety
/// The `NPC`/`NPCInfo`/`ucmd` globals and `NPC->client` must be valid.
#[allow(dead_code)] // sole caller Jedi_Combat is blocked on the saber-evasion AI core
unsafe fn Jedi_CheckJumps() {
    let mut jumpVel: vec3_t = [0.0; 3];
    let mut trace: trace_t = trace_t::default();
    let mut tr: trajectory_t = core::mem::zeroed();
    let mut lastPos: vec3_t = [0.0; 3];
    let mut testPos: vec3_t = [0.0; 3];
    let mut bottom: vec3_t = [0.0; 3];
    let mut elapsedTime: c_int;

    if (*NPCInfo).scriptFlags & SCF_NO_ACROBATICS != 0 {
        (*(*NPC).client).ps.fd.forceJumpCharge = 0.0;
        ucmd.upmove = 0;
        return;
    }
    //FIXME: should probably check this before AI decides that best move is to jump?  Otherwise, they may end up just standing there and looking dumb
    //FIXME: all this work and he still jumps off ledges... *sigh*... need CONTENTS_BOTCLIP do-not-enter brushes...?
    VectorClear(&mut jumpVel);

    if (*(*NPC).client).ps.fd.forceJumpCharge != 0.0 {
        //Com_Printf( "(%d) force jump\n", level.time );
        WP_GetVelocityForForceJump(NPC, &mut jumpVel, addr_of_mut!(ucmd));
    } else if ucmd.upmove > 0 {
        //Com_Printf( "(%d) regular jump\n", level.time );
        VectorCopy(&(*(*NPC).client).ps.velocity, &mut jumpVel);
        jumpVel[2] = JUMP_VELOCITY as f32;
    } else {
        return;
    }

    //NOTE: for now, we clear ucmd.forwardmove & ucmd.rightmove while in air to avoid jumps going awry...
    if jumpVel[0] == 0.0 && jumpVel[1] == 0.0
    //FIXME: && !ucmd.forwardmove && !ucmd.rightmove?
    {
        //we assume a jump straight up is safe
        //Com_Printf( "(%d) jump straight up is safe\n", level.time );
        return;
    }
    //Now predict where this is going
    //in steps, keep evaluating the trajectory until the new z pos is <= than current z pos, trace down from there

    VectorCopy(&(*NPC).r.currentOrigin, &mut tr.trBase);
    VectorCopy(&jumpVel, &mut tr.trDelta);
    tr.trType = TR_GRAVITY;
    tr.trTime = (*addr_of!(level)).time;
    VectorCopy(&(*NPC).r.currentOrigin, &mut lastPos);

    VectorClear(&mut trace.endpos); //shut the compiler up

    'jump_unsafe: {
        //This may be kind of wasteful, especially on long throws... use larger steps?  Divide the travelTime into a certain hard number of slices?  Trace just to apex and down?
        elapsedTime = 500;
        while elapsedTime <= 4000 {
            BG_EvaluateTrajectory(&tr, (*addr_of!(level)).time + elapsedTime, &mut testPos);
            //FIXME: account for PM_AirMove if ucmd.forwardmove and/or ucmd.rightmove is non-zero...
            if testPos[2] < lastPos[2] {
                //going down, don't check for BOTCLIP
                trace = trap::Trace(
                    &lastPos,
                    &(*NPC).r.mins,
                    &(*NPC).r.maxs,
                    &testPos,
                    (*NPC).s.number,
                    (*NPC).clipmask,
                ); //FIXME: include CONTENTS_BOTCLIP?
            } else {
                //going up, check for BOTCLIP
                trace = trap::Trace(
                    &lastPos,
                    &(*NPC).r.mins,
                    &(*NPC).r.maxs,
                    &testPos,
                    (*NPC).s.number,
                    (*NPC).clipmask | CONTENTS_BOTCLIP,
                );
            }
            if trace.allsolid != 0 || trace.startsolid != 0 {
                //WTF?
                //FIXME: what do we do when we start INSIDE the CONTENTS_BOTCLIP?  Do the trace again without that clipmask?
                break 'jump_unsafe;
            }
            if trace.fraction < 1.0 {
                //hit something
                if trace.contents & CONTENTS_BOTCLIP != 0 {
                    //hit a do-not-enter brush
                    break 'jump_unsafe;
                }
                //FIXME: trace through func_glass?
                break;
            }
            VectorCopy(&testPos, &mut lastPos);

            elapsedTime += 500;
        }
        //okay, reached end of jump, now trace down from here for a floor
        VectorCopy(&trace.endpos, &mut bottom);
        if bottom[2] > (*NPC).r.currentOrigin[2] {
            //only care about dist down from current height or lower
            bottom[2] = (*NPC).r.currentOrigin[2];
        } else if (*NPC).r.currentOrigin[2] - bottom[2] > 400.0 {
            //whoa, long drop, don't do it!
            //probably no floor at end of jump, so don't jump
            break 'jump_unsafe;
        }
        bottom[2] -= 128.0;
        trace = trap::Trace(
            &trace.endpos,
            &(*NPC).r.mins,
            &(*NPC).r.maxs,
            &bottom,
            (*NPC).s.number,
            (*NPC).clipmask,
        );
        if trace.allsolid != 0 || trace.startsolid != 0 || trace.fraction < 1.0 {
            //hit ground!
            if (trace.entityNum as c_int) < ENTITYNUM_WORLD {
                //landed on an ent
                let groundEnt: *mut gentity_t =
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(trace.entityNum as usize);
                if (*groundEnt).r.svFlags & SVF_GLASS_BRUSH != 0 {
                    //don't land on breakable glass!
                    break 'jump_unsafe;
                }
            }
            //Com_Printf( "(%d) jump is safe\n", level.time );
            return;
        }
        // fall through to jump_unsafe cleanup
    }
    //jump_unsafe:
    //probably no floor at end of jump, so don't jump
    //Com_Printf( "(%d) unsafe jump cleared\n", level.time );
    (*(*NPC).client).ps.fd.forceJumpCharge = 0.0;
    ucmd.upmove = 0;
}

/// `void NPC_Jedi_Pain(gentity_t *self, gentity_t *attacker, int damage)`
/// (NPC_AI_Jedi.c:5306). Jedi pain reaction: a saber hit breaks the parry, arms a
/// rank-scaled saber-defense debounce, may shuffle the saber level and lower
/// aggression; a ranged hit raises aggression. Then stops force-grip, runs the base
/// `NPC_Pain`, yells if merely pushed, and drops off the ceiling if ambushing. The
/// two `d_JediAI` debug blocks (incl. the hit-quadrant calc) are carried over. No
/// oracle (entity-state + debug print).
///
/// # Safety
/// `self`/`attacker` must be valid; `self->client`/`self->NPC` must be valid.
pub unsafe extern "C" fn NPC_Jedi_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    let other: *mut gentity_t = attacker;
    let mut point: vec3_t = [0.0; 3];

    VectorCopy(&*addr_of!(gPainPoint), &mut point);

    //FIXME: base the actual aggression add/subtract on health?
    //FIXME: don't do this more than once per frame?
    //FIXME: when take pain, stop force gripping....?
    if (*other).s.weapon == WP_SABER {
        //back off
        TIMER_Set(self_, c"parryTime".as_ptr(), -1);
        if (*(*self_).client).NPC_class == CLASS_DESANN
            || Q_stricmp(c"Yoda".as_ptr(), (*self_).NPC_type) == 0
        {
            //less for Desann
            (*(*self_).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                (*addr_of!(level)).time + (3 - (*addr_of!(g_spskill)).integer) * 50;
        } else if (*(*self_).NPC).rank >= RANK_LT_JG {
            (*(*self_).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                (*addr_of!(level)).time + (3 - (*addr_of!(g_spskill)).integer) * 100; //300
        } else {
            (*(*self_).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                (*addr_of!(level)).time + (3 - (*addr_of!(g_spskill)).integer) * 200; //500
        }
        if Q_irand(0, 3) == 0 {
            //ouch... maybe switch up which saber power level we're using
            Jedi_AdjustSaberAnimLevel(self_, Q_irand(FORCE_LEVEL_1, FORCE_LEVEL_3));
        }
        if Q_irand(0, 1) == 0
        //damage > 20 || self->health < 40 ||
        {
            //Com_Printf( "(%d) drop agg - hit by saber\n", level.time );
            Jedi_Aggression(self_, -1);
        }
        if (*addr_of!(d_JediAI)).integer != 0 {
            Com_Printf(&format!(
                "({}) PAIN: agg {}, no parry until {}\n",
                (*addr_of!(level)).time,
                (*(*self_).NPC).stats.aggression,
                (*addr_of!(level)).time + 500
            ));
        }
        //for testing only
        // Figure out what quadrant the hit was in.
        if (*addr_of!(d_JediAI)).integer != 0 {
            let mut diff: vec3_t = [0.0; 3];
            let mut fwdangles: vec3_t = [0.0; 3];
            let mut right: vec3_t = [0.0; 3];
            let rightdot: f32;
            let zdiff: f32;

            VectorSubtract(&point, &(*(*self_).client).renderInfo.eyePoint, &mut diff);
            diff[2] = 0.0;
            fwdangles[1] = (*(*self_).client).ps.viewangles[1];
            AngleVectors(&fwdangles, None, Some(&mut right), None);
            rightdot = DotProduct(&right, &diff);
            zdiff = point[2] - (*(*self_).client).renderInfo.eyePoint[2];

            Com_Printf(&format!(
                "({}) saber hit at height {:.2}, zdiff: {:.2}, rightdot: {:.2}\n",
                (*addr_of!(level)).time,
                point[2] - (*self_).r.absmin[2],
                zdiff,
                rightdot
            ));
        }
    } else {
        //attack
        //Com_Printf( "(%d) raise agg - hit by ranged\n", level.time );
        Jedi_Aggression(self_, 1);
    }

    (*(*self_).NPC).enemyCheckDebounceTime = 0;

    WP_ForcePowerStop(self_, FP_GRIP);

    //NPC_Pain( self, inflictor, other, point, damage, mod );
    NPC_Pain(self_, attacker, damage);

    if damage == 0 && (*self_).health > 0 {
        //FIXME: better way to know I was pushed
        G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1, EV_PUSHED3), 2000);
    }

    //drop me from the ceiling if I'm on it
    if Jedi_WaitingAmbush(self_) != QFALSE {
        (*(*self_).client).noclip = QFALSE;
    }
    if (*(*self_).client).ps.legsAnim == BOTH_CEILING_CLING {
        NPC_SetAnim(
            self_,
            SETANIM_LEGS,
            BOTH_CEILING_DROP,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
    }
    if (*(*self_).client).ps.torsoAnim == BOTH_CEILING_CLING {
        NPC_SetAnim(
            self_,
            SETANIM_TORSO,
            BOTH_CEILING_DROP,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
    }
}

/// `qboolean Jedi_CheckDanger( void )` (NPC_AI_Jedi.c:5394). Checks the alert events
/// for a danger-level event with a valid enemy-team owner; if found, makes that
/// owner the enemy and arms an attack delay. Returns whether it found danger. No
/// oracle (NPC-global entity-state + alert events).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->client` must be valid; `level` must be
/// initialised.
pub unsafe fn Jedi_CheckDanger() -> qboolean {
    let alertEvent: c_int = NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_MINOR);
    if (*addr_of!(level)).alertEvents[alertEvent as usize].level >= AEL_DANGER {
        //run away!
        if (*addr_of!(level)).alertEvents[alertEvent as usize].owner.is_null()
            || (*(*addr_of!(level)).alertEvents[alertEvent as usize].owner)
                .client
                .is_null()
            || ((*addr_of!(level)).alertEvents[alertEvent as usize].owner != NPC
                && (*(*(*addr_of!(level)).alertEvents[alertEvent as usize].owner).client)
                    .playerTeam
                    != (*(*NPC).client).playerTeam)
        {
            //no owner
            return QFALSE;
        }
        G_SetEnemy(NPC, (*addr_of!(level)).alertEvents[alertEvent as usize].owner);
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
        TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
        return QTRUE;
    }
    QFALSE
}

/// `qboolean Jedi_CheckAmbushPlayer( void )` (NPC_AI_Jedi.c:5413). While lying in
/// wait, scans the players: if uncloaked and a player has their crosshair on me I
/// wake immediately, otherwise I only ambush a same-room player below me, within
/// range, in FOV and with clear LOS. On a hit, makes that player the enemy and arms
/// an attack delay. No oracle (NPC-global entity-state + senses).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->client` must be valid; `g_entities` must
/// be initialised.
pub unsafe fn Jedi_CheckAmbushPlayer() -> qboolean {
    for i in 0..MAX_CLIENTS as c_int {
        let player: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if player.is_null() || (*player).client.is_null() {
            continue;
        }

        if NPC_ValidEnemy(player) == QFALSE {
            continue;
        }

        //		if ( NPC->client->ps.powerups[PW_CLOAKED] || g_crosshairEntNum != NPC->s.number )
        if (*(*NPC).client).ps.powerups[PW_CLOAKED as usize] != 0
            || NPC_SomeoneLookingAtMe(NPC) == QFALSE
        //rwwFIXMEFIXME: Need to pay attention to who is under crosshair for each player or something.
        {
            //if I'm not cloaked and the player's crosshair is on me, I will wake up, otherwise do this stuff down here...
            if trap::InPVS(&(*player).r.currentOrigin, &(*NPC).r.currentOrigin) == QFALSE {
                //must be in same room
                continue;
            } else {
                if (*(*NPC).client).ps.powerups[PW_CLOAKED as usize] == 0 {
                    NPC_SetLookTarget(NPC, 0, 0);
                }
            }
            let zDiff: f32 = (*NPC).r.currentOrigin[2] - (*player).r.currentOrigin[2];
            if zDiff <= 0.0 || zDiff > 512.0 {
                //never ambush if they're above me or way way below me
                continue;
            }

            //If the target is this close, then wake up regardless
            let target_dist: f32 =
                DistanceHorizontalSquared(&(*player).r.currentOrigin, &(*NPC).r.currentOrigin);
            if target_dist > 4096.0 {
                //closer than 64 - always ambush
                if target_dist > 147456.0 {
                    //> 384, not close enough to ambush
                    continue;
                }
                //Check FOV first
                if (*(*NPC).client).ps.powerups[PW_CLOAKED as usize] != 0 {
                    if InFOV(player, NPC, 30, 90) == QFALSE {
                        continue;
                    }
                } else {
                    if InFOV(player, NPC, 45, 90) == QFALSE {
                        continue;
                    }
                }
            }

            if NPC_ClearLOS4(player) == QFALSE {
                continue;
            }
        }

        //Got him, return true;
        G_SetEnemy(NPC, player);
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
        TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
        return QTRUE;
    }

    //Didn't get anyone.
    QFALSE
}

/// `void Jedi_Ambush( gentity_t *self )` (NPC_AI_Jedi.c:5495). Springs an ambush:
/// un-noclips, plays the ceiling-drop anim, syncs weaponTime, activates the saber
/// (non-Boba), decloaks, and yells. No oracle (entity-state + anim/voice).
///
/// # Safety
/// `self` and `self->client` must be valid.
pub unsafe fn Jedi_Ambush(self_: *mut gentity_t) {
    (*(*self_).client).noclip = QFALSE;
    //	self->client->ps.pm_flags |= PMF_JUMPING|PMF_SLOW_MO_FALL;
    NPC_SetAnim(
        self_,
        SETANIM_BOTH,
        BOTH_CEILING_DROP,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
    );
    (*(*self_).client).ps.weaponTime = (*(*self_).client).ps.torsoTimer; //NPC->client->ps.torsoTimer; //what the?
    if (*(*self_).client).NPC_class != CLASS_BOBAFETT {
        WP_ActivateSaber(self_);
    }
    Jedi_Decloak(self_);
    G_AddVoiceEvent(self_, Q_irand(EV_ANGER1, EV_ANGER3), 1000);
}

/// `qboolean Jedi_WaitingAmbush( gentity_t *self )` (NPC_AI_Jedi.c:5509). Returns
/// `qtrue` when this Jedi is an ambusher (`JSF_AMBUSH` spawnflag) currently
/// sitting in noclip (lying in wait). No oracle (pure entity-state).
///
/// # Safety
/// `self` and `self->client` must be valid.
pub unsafe fn Jedi_WaitingAmbush(self_: *mut gentity_t) -> qboolean {
    if ((*self_).spawnflags & JSF_AMBUSH) != 0 && (*(*self_).client).noclip == QTRUE {
        return QTRUE;
    }
    QFALSE
}

/// `static void Jedi_Patrol( void )` (NPC_AI_Jedi.c:5523). Idle Jedi behavior when it
/// has no enemy: if waiting in ambush it clings to the ceiling and watches for the
/// player; otherwise (when set to look for enemies) it scans all clients for a valid
/// nearby foe (or one whose thrown saber is incoming), and either grabs it as an enemy
/// or — for the player — toys with it in escalating stages (look, face, ignite saber)
/// driven by a `watchTime` timer. Finally moves toward any goal and updates angles.
/// No oracle (NPC/ucmd globals + timers + trap_InPVS).
///
/// The C uses `goto finish` (the first-watch ignore path); here it becomes
/// `break 'finish` out of a labeled block, with the goal/angle epilogue after it.
///
/// # Safety
/// The `NPC`/`NPCInfo`/`ucmd` globals and `NPC->client` must be valid.
#[allow(dead_code)] // callers (NPC_BSJedi_Default / NPC_BSJedi_FollowLeader) are
                    // blocked on the saber-evasion AI core; lands when they do
unsafe fn Jedi_Patrol() {
    (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;

    'finish: {
        if Jedi_WaitingAmbush(NPC) != QFALSE {
            //hiding on the ceiling
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_CEILING_CLING,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
                //look for enemies
                if Jedi_CheckAmbushPlayer() != QFALSE || Jedi_CheckDanger() != QFALSE {
                    //found him!
                    Jedi_Ambush(NPC);
                    NPC_UpdateAngles(QTRUE, QTRUE);
                    return;
                }
            }
        } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
            //NPCInfo->scriptFlags & SCF_CHASE_ENEMIES )
            //look for enemies
            let mut best_enemy: *mut gentity_t = null_mut();
            let mut best_enemy_dist: f32 = Q3_INFINITE as f32;
            let mut i: c_int = 0;
            while i < ENTITYNUM_WORLD {
                let enemy: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
                let enemy_dist: f32;
                if !enemy.is_null()
                    && !(*enemy).client.is_null()
                    && NPC_ValidEnemy(enemy) != QFALSE
                    && (*(*enemy).client).playerTeam == (*(*NPC).client).enemyTeam
                {
                    if trap::InPVS(&(*NPC).r.currentOrigin, &(*enemy).r.currentOrigin) != QFALSE {
                        //we could potentially see him
                        enemy_dist =
                            DistanceSquared(&(*NPC).r.currentOrigin, &(*enemy).r.currentOrigin);
                        if (*enemy).s.eType == ET_PLAYER || enemy_dist < best_enemy_dist {
                            //if the enemy is close enough, or threw his saber, take him as the enemy
                            //FIXME: what if he throws a thermal detonator?
                            if enemy_dist < (220 * 220) as f32
                                || ((*NPCInfo).investigateCount >= 3
                                    && (*(*NPC).client).ps.saberHolstered == 0)
                            {
                                G_SetEnemy(NPC, enemy);
                                //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be auto now
                                (*NPCInfo).stats.aggression = 3;
                                break;
                            } else if (*(*enemy).client).ps.saberInFlight != QFALSE
                                && (*(*enemy).client).ps.saberHolstered == 0
                            {
                                //threw his saber, see if it's heading toward me and close enough to consider a threat
                                let saberDist: f32;
                                let mut saberDir2Me: vec3_t = [0.0; 3];
                                let mut saberMoveDir: vec3_t = [0.0; 3];
                                let saber: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                    .add((*(*enemy).client).ps.saberEntityNum as usize);
                                VectorSubtract(
                                    &(*NPC).r.currentOrigin,
                                    &(*saber).r.currentOrigin,
                                    &mut saberDir2Me,
                                );
                                saberDist = VectorNormalize(&mut saberDir2Me);
                                VectorCopy(&(*saber).s.pos.trDelta, &mut saberMoveDir);
                                VectorNormalize(&mut saberMoveDir);
                                if DotProduct(&saberMoveDir, &saberDir2Me) > 0.5 {
                                    //it's heading towards me
                                    if saberDist < 200.0 {
                                        //incoming!
                                        G_SetEnemy(NPC, enemy);
                                        //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be auto now
                                        (*NPCInfo).stats.aggression = 3;
                                        break;
                                    }
                                }
                            }
                            best_enemy_dist = enemy_dist;
                            best_enemy = enemy;
                        }
                    }
                }

                i += 1;
            }
            if (*NPC).enemy.is_null() {
                //still not mad
                if best_enemy.is_null() {
                    //Com_Printf( "(%d) drop agg - no enemy (patrol)\n", level.time );
                    Jedi_AggressionErosion(-1);
                //FIXME: what about alerts?  But not if ignore alerts
                } else {
                    //have one to consider
                    if NPC_ClearLOS4(best_enemy) != QFALSE {
                        //we have a clear (of architecture) LOS to him
                        if (*best_enemy).s.number != 0 {
                            //just attack
                            G_SetEnemy(NPC, best_enemy);
                            (*NPCInfo).stats.aggression = 3;
                        } else if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
                            //the player, toy with him
                            //get progressively more interested over time
                            if TIMER_Done(NPC, c"watchTime".as_ptr()) != QFALSE {
                                //we want to pick him up in stages
                                if TIMER_Get(NPC, c"watchTime".as_ptr()) == -1 {
                                    //this is the first time, we'll ignore him for a couple seconds
                                    TIMER_Set(NPC, c"watchTime".as_ptr(), Q_irand(3000, 5000));
                                    break 'finish;
                                } else {
                                    //okay, we've ignored him, now start to notice him
                                    if (*NPCInfo).investigateCount == 0 {
                                        G_AddVoiceEvent(
                                            NPC,
                                            Q_irand(EV_JDETECTED1, EV_JDETECTED3),
                                            3000,
                                        );
                                    }
                                    (*NPCInfo).investigateCount += 1;
                                    TIMER_Set(NPC, c"watchTime".as_ptr(), Q_irand(4000, 10000));
                                }
                            }
                            //while we're waiting, do what we need to do
                            if best_enemy_dist < (440 * 440) as f32
                                || (*NPCInfo).investigateCount >= 2
                            {
                                //stage three: keep facing him
                                NPC_FaceEntity(best_enemy, QTRUE);
                                if best_enemy_dist < (330 * 330) as f32 {
                                    //stage four: turn on the saber
                                    if (*(*NPC).client).ps.saberInFlight == QFALSE {
                                        WP_ActivateSaber(NPC);
                                    }
                                }
                            } else if best_enemy_dist < (550 * 550) as f32
                                || (*NPCInfo).investigateCount == 1
                            {
                                //stage two: stop and face him every now and then
                                if TIMER_Done(NPC, c"watchTime".as_ptr()) != QFALSE {
                                    NPC_FaceEntity(best_enemy, QTRUE);
                                }
                            } else {
                                //stage one: look at him.
                                NPC_SetLookTarget(NPC, (*best_enemy).s.number, 0);
                            }
                        }
                    } else if TIMER_Done(NPC, c"watchTime".as_ptr()) != QFALSE {
                        //haven't seen him in a bit, clear the lookTarget
                        NPC_ClearLookTarget(NPC);
                    }
                }
            }
        }
    }
    //finish:
    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons |= BUTTON_WALKING;
        //Jedi_Move( NPCInfo->goalEntity );
        NPC_MoveToGoal(QTRUE);
    }

    NPC_UpdateAngles(QTRUE, QTRUE);

    if !(*NPC).enemy.is_null() {
        //just picked one up
        (*NPCInfo).enemyCheckDebounceTime = (*addr_of!(level)).time + Q_irand(3000, 10000);
    }
}

/// `qboolean Jedi_CanPullBackSaber( gentity_t *self )` (NPC_AI_Jedi.c:5678). Whether
/// the Jedi may retract its saber: no during an un-expired broken parry, always for
/// the boss classes (Shadowtrooper/Tavion/Luke/Desann/Yoda), no while in pain, else
/// yes. No oracle (entity-state).
///
/// # Safety
/// `self`, `self->client`, and `self->NPC_type` must be valid.
pub unsafe fn Jedi_CanPullBackSaber(self_: *mut gentity_t) -> qboolean {
    if (*(*self_).client).ps.saberBlocked == BLOCKED_PARRY_BROKEN
        && TIMER_Done(self_, c"parryTime".as_ptr()) == QFALSE
    {
        return QFALSE;
    }

    if (*(*self_).client).NPC_class == CLASS_SHADOWTROOPER
        || (*(*self_).client).NPC_class == CLASS_TAVION
        || (*(*self_).client).NPC_class == CLASS_LUKE
        || (*(*self_).client).NPC_class == CLASS_DESANN
        || Q_stricmp(c"Yoda".as_ptr(), (*self_).NPC_type) == 0
    {
        return QTRUE;
    }

    if (*self_).painDebounceTime > (*addr_of!(level)).time
    /*|| self->client->ps.weaponTime > 0 */
    {
        return QFALSE;
    }

    QTRUE
}

/// `void G_StartMatrixEffect( gentity_t *ent )` (NPC_AI_Jedi.c:16). Faithfully
/// empty in the original ("perhaps write this at some point?"). No oracle.
///
/// # Safety
/// `ent` is unused; callers pass a valid `gentity_t` pointer.
pub unsafe fn G_StartMatrixEffect(_ent: *mut gentity_t) {
    //perhaps write this at some point?
}

/// `void NPC_ShadowTrooper_Precache( void )` (NPC_AI_Jedi.c:103). Registers the
/// force-ammo item and precaches the shadowtrooper cloak/decloak sounds. No oracle
/// (asset registration).
///
/// # Safety
/// Requires the item / sound index tables to be initialised.
pub unsafe fn NPC_ShadowTrooper_Precache() {
    RegisterItem(BG_FindItemForAmmo(AMMO_FORCE));
    G_SoundIndex("sound/chars/shadowtrooper/cloak.wav");
    G_SoundIndex("sound/chars/shadowtrooper/decloak.wav");
}

/// `void Boba_Precache( void )` (NPC_AI_Jedi.c:182). Precaches Boba Fett's jetpack
/// sounds and jet/flamethrower effects. No oracle (asset registration).
///
/// # Safety
/// Requires the sound/effect index tables to be initialised.
pub unsafe fn Boba_Precache() {
    G_SoundIndex("sound/boba/jeton.wav");
    G_SoundIndex("sound/boba/jethover.wav");
    G_SoundIndex("sound/effects/combustfire.mp3");
    G_EffectIndex("boba/jet");
    G_EffectIndex("boba/fthrw");
}

/// `void Boba_ChangeWeapon( int wp )` (NPC_AI_Jedi.c:193). If the `NPC` global is
/// not already wielding `wp`, swaps to it and plays the weapon-change sound. No
/// oracle (NPC-global entity-state + sound event).
///
/// # Safety
/// The `NPC` global must be valid.
pub unsafe fn Boba_ChangeWeapon(wp: c_int) {
    if (*NPC).s.weapon == wp {
        return;
    }
    NPC_ChangeWeapon(wp);
    G_AddEvent(
        NPC,
        EV_GENERAL_SOUND,
        G_SoundIndex("sound/weapons/change.wav"),
    );
}

/// `void WP_ResistForcePush( gentity_t *self, gentity_t *pusher, qboolean noPenalty )`
/// (NPC_AI_Jedi.c:203). Plays the resist-push animation (full body when on the
/// ground and not spinning/flipping/rolling/knocked-down/crouching, else torso-only),
/// then — unless `noPenalty` — pins the resister's velocity and weaponTime so they
/// can't move/attack for a beat (scaled by `timescale` when force-speed is active),
/// finally lighting the push hand effect and clearing the pull powerup. No oracle
/// (trap_Cvar + global entity-state mutation).
///
/// # Safety
/// `self`/`pusher` may be NULL; `self->client`/`pusher->client` are guarded before
/// any deref.
pub unsafe fn WP_ResistForcePush(
    self_: *mut gentity_t,
    pusher: *mut gentity_t,
    noPenalty: qboolean,
) {
    let parts: c_int;
    let mut runningResist: qboolean = QFALSE;

    if self_.is_null()
        || (*self_).health <= 0
        || (*self_).client.is_null()
        || pusher.is_null()
        || (*pusher).client.is_null()
    {
        return;
    }
    if ((*self_).s.number == 0
        || (*(*self_).client).NPC_class == CLASS_DESANN
        || Q_stricmp(c"Yoda".as_ptr(), (*self_).NPC_type) == 0
        || (*(*self_).client).NPC_class == CLASS_LUKE)
        && (VectorLengthSquared(&(*(*self_).client).ps.velocity) > 10000.0
            || (*(*self_).client).ps.fd.forcePowerLevel[FP_PUSH as usize] >= FORCE_LEVEL_3
            || (*(*self_).client).ps.fd.forcePowerLevel[FP_PULL as usize] >= FORCE_LEVEL_3)
    {
        runningResist = QTRUE;
    }
    if runningResist == QFALSE
        && (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE
        && BG_SpinningSaberAnim((*(*self_).client).ps.legsAnim) == QFALSE
        && BG_FlippingAnim((*(*self_).client).ps.legsAnim) == QFALSE
        && PM_RollingAnim((*(*self_).client).ps.legsAnim) == QFALSE
        && PM_InKnockDown(&mut (*(*self_).client).ps) == QFALSE
        && BG_CrouchAnim((*(*self_).client).ps.legsAnim) == QFALSE
    {
        //if on a surface and not in a spin or flip, play full body resist
        parts = SETANIM_BOTH;
    } else {
        //play resist just in torso
        parts = SETANIM_TORSO;
    }
    NPC_SetAnim(
        self_,
        parts,
        BOTH_RESISTPUSH,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
    );
    if noPenalty == QFALSE {
        let tFVal: f32;

        // trap_Cvar_VariableStringBuffer("timescale", buf, sizeof(buf)); tFVal = atof(buf);
        let buf = std::ffi::CString::new(trap::Cvar_VariableString("timescale")).unwrap();
        tFVal = atof(buf.as_ptr()) as f32;

        if runningResist == QFALSE {
            VectorClear(&mut (*(*self_).client).ps.velocity);
            //still stop them from attacking or moving for a bit, though
            //FIXME: maybe push just a little (like, slide)?
            (*(*self_).client).ps.weaponTime = 1000;
            if (*(*self_).client).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0 {
                (*(*self_).client).ps.weaponTime =
                    ((*(*self_).client).ps.weaponTime as f32 * tFVal).floor() as c_int;
            }
            (*(*self_).client).ps.pm_time = (*(*self_).client).ps.weaponTime;
            (*(*self_).client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
            //play the full body push effect on me
            //self->forcePushTime = level.time + 600; // let the push effect last for 600 ms
            //rwwFIXMEFIXME: Do this?
        } else {
            (*(*self_).client).ps.weaponTime = 600;
            if (*(*self_).client).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0 {
                (*(*self_).client).ps.weaponTime =
                    ((*(*self_).client).ps.weaponTime as f32 * tFVal).floor() as c_int;
            }
        }
    }
    //play my force push effect on my hand
    (*(*self_).client).ps.powerups[PW_DISINT_4 as usize] =
        (*addr_of!(level)).time + (*(*self_).client).ps.torsoTimer + 500;
    (*(*self_).client).ps.powerups[PW_PULL as usize] = 0;
    Jedi_PlayBlockedPushSound(self_);
}

/// `void Jedi_ClearTimers( gentity_t *ent )` (NPC_AI_Jedi.c:110). Zeroes every
/// per-Jedi behavior timer. No oracle (timers/entity-state).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe fn Jedi_ClearTimers(ent: *mut gentity_t) {
    TIMER_Set(ent, c"roamTime".as_ptr(), 0);
    TIMER_Set(ent, c"chatter".as_ptr(), 0);
    TIMER_Set(ent, c"strafeLeft".as_ptr(), 0);
    TIMER_Set(ent, c"strafeRight".as_ptr(), 0);
    TIMER_Set(ent, c"noStrafe".as_ptr(), 0);
    TIMER_Set(ent, c"walking".as_ptr(), 0);
    TIMER_Set(ent, c"taunting".as_ptr(), 0);
    TIMER_Set(ent, c"parryTime".as_ptr(), 0);
    TIMER_Set(ent, c"parryReCalcTime".as_ptr(), 0);
    TIMER_Set(ent, c"forceJumpChasing".as_ptr(), 0);
    TIMER_Set(ent, c"jumpChaseDebounce".as_ptr(), 0);
    TIMER_Set(ent, c"moveforward".as_ptr(), 0);
    TIMER_Set(ent, c"moveback".as_ptr(), 0);
    TIMER_Set(ent, c"movenone".as_ptr(), 0);
    TIMER_Set(ent, c"moveright".as_ptr(), 0);
    TIMER_Set(ent, c"moveleft".as_ptr(), 0);
    TIMER_Set(ent, c"movecenter".as_ptr(), 0);
    TIMER_Set(ent, c"saberLevelDebounce".as_ptr(), 0);
    TIMER_Set(ent, c"noRetreat".as_ptr(), 0);
    TIMER_Set(ent, c"holdLightning".as_ptr(), 0);
    TIMER_Set(ent, c"gripping".as_ptr(), 0);
    TIMER_Set(ent, c"draining".as_ptr(), 0);
    TIMER_Set(ent, c"noturn".as_ptr(), 0);
}

/// `void Jedi_PlayBlockedPushSound( gentity_t *self )` (NPC_AI_Jedi.c:137). Plays
/// the push-fail voice event — always for the player (entity 0), otherwise for a
/// live NPC past its blocked-speech debounce (which it then re-arms). No oracle
/// (voice event + entity-state).
///
/// # Safety
/// `self` must be valid; `self->NPC` may be NULL (the NPC branch guards it).
pub unsafe fn Jedi_PlayBlockedPushSound(self_: *mut gentity_t) {
    if (*self_).s.number == 0 {
        G_AddVoiceEvent(self_, EV_PUSHFAIL, 3000);
    } else if (*self_).health > 0
        && !(*self_).NPC.is_null()
        && (*(*self_).NPC).blockedSpeechDebounceTime < (*addr_of!(level)).time
    {
        G_AddVoiceEvent(self_, EV_PUSHFAIL, 3000);
        (*(*self_).NPC).blockedSpeechDebounceTime = (*addr_of!(level)).time + 3000;
    }
}

/// `void Jedi_PlayDeflectSound( gentity_t *self )` (NPC_AI_Jedi.c:150). Plays a
/// random deflect voice event — always for the player, otherwise for a live NPC
/// past its blocked-speech debounce (which it then re-arms). No oracle (voice
/// event + entity-state).
///
/// # Safety
/// `self` must be valid; `self->NPC` may be NULL (the NPC branch guards it).
pub unsafe fn Jedi_PlayDeflectSound(self_: *mut gentity_t) {
    if (*self_).s.number == 0 {
        G_AddVoiceEvent(self_, Q_irand(EV_DEFLECT1, EV_DEFLECT3), 3000);
    } else if (*self_).health > 0
        && !(*self_).NPC.is_null()
        && (*(*self_).NPC).blockedSpeechDebounceTime < (*addr_of!(level)).time
    {
        G_AddVoiceEvent(self_, Q_irand(EV_DEFLECT1, EV_DEFLECT3), 3000);
        (*(*self_).NPC).blockedSpeechDebounceTime = (*addr_of!(level)).time + 3000;
    }
}

/// `void NPC_Jedi_PlayConfusionSound( gentity_t *self )` (NPC_AI_Jedi.c:163). If
/// alive, plays a confusion line for Tavion/Desann, otherwise a coin-flip between
/// a taunt and a gloat line. No oracle (voice event + entity-state).
///
/// # Safety
/// `self` must be valid; `self->client` may be NULL (guarded before deref).
pub unsafe fn NPC_Jedi_PlayConfusionSound(self_: *mut gentity_t) {
    if (*self_).health > 0 {
        if !(*self_).client.is_null()
            && ((*(*self_).client).NPC_class == CLASS_TAVION
                || (*(*self_).client).NPC_class == CLASS_DESANN)
        {
            G_AddVoiceEvent(self_, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000);
        } else if Q_irand(0, 1) != 0 {
            G_AddVoiceEvent(self_, Q_irand(EV_TAUNT1, EV_TAUNT3), 2000);
        } else {
            G_AddVoiceEvent(self_, Q_irand(EV_GLOAT1, EV_GLOAT3), 2000);
        }
    }
}

/// `static void Jedi_CombatTimersUpdate( int enemy_dist )` (NPC_AI_Jedi.c:4015).
/// Periodically (roamTime) tweaks aggression based on rage state and the enemy's
/// weapon, maybe starts a strafe, and reacts to any pending saber-combat events
/// (parried / hit-enemy / blocked / deflected / hit-wall / hit-object) by adjusting
/// aggression and saber-anim level. No oracle (NPC-global AI: timers, RNG, voice
/// events).
///
/// # Safety
/// The `NPC`/`NPCInfo` globals and `NPC->client` must be valid.
#[allow(dead_code)] // called by Jedi_Combat as it lands
unsafe fn Jedi_CombatTimersUpdate(enemy_dist: c_int) {
    if TIMER_Done(NPC, c"roamTime".as_ptr()) != QFALSE {
        TIMER_Set(NPC, c"roamTime".as_ptr(), Q_irand(2000, 5000));
        //okay, now mess with agression
        if (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
            //raging
            Jedi_Aggression(NPC, Q_irand(0, 3));
        } else if (*(*NPC).client).ps.fd.forceRageRecoveryTime > (*addr_of!(level)).time {
            //recovering
            Jedi_Aggression(NPC, Q_irand(0, -2));
        }
        if !(*NPC).enemy.is_null() && !(*(*NPC).enemy).client.is_null() {
            match (*(*(*NPC).enemy).client).ps.weapon {
                WP_SABER => {
                    //If enemy has a lightsaber, always close in
                    if BG_SabersOff(addr_of_mut!((*(*(*NPC).enemy).client).ps)) != QFALSE {
                        //fool!  Standing around unarmed, charge!
                        //Com_Printf( "(%d) raise agg - enemy saber off\n", level.time );
                        Jedi_Aggression(NPC, 2);
                    } else {
                        //Com_Printf( "(%d) raise agg - enemy saber\n", level.time );
                        Jedi_Aggression(NPC, 1);
                    }
                }
                WP_BLASTER | WP_BRYAR_PISTOL | WP_DISRUPTOR | WP_BOWCASTER | WP_REPEATER
                | WP_DEMP2 | WP_FLECHETTE | WP_ROCKET_LAUNCHER => {
                    //if he has a blaster, move in when:
                    //They're not shooting at me
                    if (*(*NPC).enemy).attackDebounceTime < (*addr_of!(level)).time {
                        //does this apply to players?
                        //Com_Printf( "(%d) raise agg - enemy not shooting ranged weap\n", level.time );
                        Jedi_Aggression(NPC, 1);
                    }
                    //He's closer than a dist that gives us time to deflect
                    if enemy_dist < 256 {
                        //Com_Printf( "(%d) raise agg - enemy ranged weap- too close\n", level.time );
                        Jedi_Aggression(NPC, 1);
                    }
                }
                _ => {}
            }
        }
    }

    if TIMER_Done(NPC, c"noStrafe".as_ptr()) != QFALSE
        && TIMER_Done(NPC, c"strafeLeft".as_ptr()) != QFALSE
        && TIMER_Done(NPC, c"strafeRight".as_ptr()) != QFALSE
    {
        //FIXME: Maybe more likely to do this if aggression higher?  Or some other stat?
        if Q_irand(0, 4) == 0 {
            //start a strafe
            if Jedi_Strafe(1000, 3000, 0, 4000, QTRUE) != QFALSE {
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf("off strafe\n");
                }
            }
        } else {
            //postpone any strafing for a while
            TIMER_Set(NPC, c"noStrafe".as_ptr(), Q_irand(1000, 3000));
        }
    }

    if (*(*NPC).client).ps.saberEventFlags != 0 {
        //some kind of saber combat event is still pending
        let mut newFlags: c_int = (*(*NPC).client).ps.saberEventFlags;
        if (*(*NPC).client).ps.saberEventFlags & SEF_PARRIED != 0 {
            //parried
            TIMER_Set(NPC, c"parryTime".as_ptr(), -1);
            /*
            if ( NPCInfo->rank >= RANK_LT_JG )
            {
                NPC->client->ps.fd.forcePowerDebounce[FP_SABER_DEFENSE] = level.time + 100;
            }
            else
            {
                NPC->client->ps.fd.forcePowerDebounce[FP_SABER_DEFENSE] = level.time + 500;
            }
            */
            if !(*NPC).enemy.is_null()
                && PM_SaberInKnockaway((*(*(*NPC).enemy).client).ps.saberMove) != QFALSE
            {
                //advance!
                Jedi_Aggression(NPC, 1); //get closer
                Jedi_AdjustSaberAnimLevel(NPC, (*(*NPC).client).ps.fd.saberAnimLevel - 1);
                //use a faster attack
            } else {
                if Q_irand(0, 1) == 0 {
                    //FIXME: dependant on rank/diff?
                    //Com_Printf( "(%d) drop agg - we parried\n", level.time );
                    Jedi_Aggression(NPC, -1);
                }
                if Q_irand(0, 1) == 0 {
                    Jedi_AdjustSaberAnimLevel(NPC, (*(*NPC).client).ps.fd.saberAnimLevel - 1);
                }
            }
            if (*addr_of!(d_JediAI)).integer != 0 {
                Com_Printf(&format!(
                    "({}) PARRY: agg {}, no parry until {}\n",
                    (*addr_of!(level)).time,
                    (*NPCInfo).stats.aggression,
                    (*addr_of!(level)).time + 100,
                ));
            }
            newFlags &= !SEF_PARRIED;
        }
        if (*(*NPC).client).ps.weaponTime == 0
            && ((*(*NPC).client).ps.saberEventFlags & SEF_HITENEMY != 0)
        {
            //hit enemy
            //we hit our enemy last time we swung, drop our aggression
            if Q_irand(0, 1) == 0 {
                //FIXME: dependant on rank/diff?
                //Com_Printf( "(%d) drop agg - we hit enemy\n", level.time );
                Jedi_Aggression(NPC, -1);
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf(&format!(
                        "({}) HIT: agg {}\n",
                        (*addr_of!(level)).time,
                        (*NPCInfo).stats.aggression,
                    ));
                }
                if Q_irand(0, 3) == 0
                    && (*NPCInfo).blockedSpeechDebounceTime < (*addr_of!(level)).time
                    && jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize]
                        < (*addr_of!(level)).time
                    && (*NPC).painDebounceTime < (*addr_of!(level)).time - 1000
                {
                    G_AddVoiceEvent(NPC, Q_irand(EV_GLOAT1, EV_GLOAT3), 3000);
                    (*NPCInfo).blockedSpeechDebounceTime = (*addr_of!(level)).time + 3000;
                    jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize] =
                        (*NPCInfo).blockedSpeechDebounceTime;
                }
            }
            if Q_irand(0, 2) == 0 {
                Jedi_AdjustSaberAnimLevel(NPC, (*(*NPC).client).ps.fd.saberAnimLevel + 1);
            }
            newFlags &= !SEF_HITENEMY;
        }
        if (*(*NPC).client).ps.saberEventFlags & SEF_BLOCKED != 0 {
            //was blocked whilst attacking
            if PM_SaberInBrokenParry((*(*NPC).client).ps.saberMove) != QFALSE
                || (*(*NPC).client).ps.saberBlocked == BLOCKED_PARRY_BROKEN
            {
                //Com_Printf( "(%d) drop agg - we were knock-blocked\n", level.time );
                if (*(*NPC).client).ps.saberInFlight != QFALSE {
                    //lost our saber, too!!!
                    Jedi_Aggression(NPC, -5); //really really really should back off!!!
                } else {
                    Jedi_Aggression(NPC, -2); //really should back off!
                }
                Jedi_AdjustSaberAnimLevel(NPC, (*(*NPC).client).ps.fd.saberAnimLevel + 1);
                //use a stronger attack
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf(&format!(
                        "({}) KNOCK-BLOCKED: agg {}\n",
                        (*addr_of!(level)).time,
                        (*NPCInfo).stats.aggression,
                    ));
                }
            } else {
                if Q_irand(0, 2) == 0 {
                    //FIXME: dependant on rank/diff?
                    //Com_Printf( "(%d) drop agg - we were blocked\n", level.time );
                    Jedi_Aggression(NPC, -1);
                    if (*addr_of!(d_JediAI)).integer != 0 {
                        Com_Printf(&format!(
                            "({}) BLOCKED: agg {}\n",
                            (*addr_of!(level)).time,
                            (*NPCInfo).stats.aggression,
                        ));
                    }
                }
                if Q_irand(0, 1) == 0 {
                    Jedi_AdjustSaberAnimLevel(NPC, (*(*NPC).client).ps.fd.saberAnimLevel + 1);
                }
            }
            newFlags &= !SEF_BLOCKED;
            //FIXME: based on the type of parry the enemy is doing and my skill,
            //		choose an attack that is likely to get around the parry?
            //		right now that's generic in the saber animation code, auto-picks
            //		a next anim for me, but really should be AI-controlled.
        }
        if (*(*NPC).client).ps.saberEventFlags & SEF_DEFLECTED != 0 {
            //deflected a shot
            newFlags &= !SEF_DEFLECTED;
            if Q_irand(0, 3) == 0 {
                Jedi_AdjustSaberAnimLevel(NPC, (*(*NPC).client).ps.fd.saberAnimLevel - 1);
            }
        }
        if (*(*NPC).client).ps.saberEventFlags & SEF_HITWALL != 0 {
            //hit a wall
            newFlags &= !SEF_HITWALL;
        }
        if (*(*NPC).client).ps.saberEventFlags & SEF_HITOBJECT != 0 {
            //hit some other damagable object
            if Q_irand(0, 3) == 0 {
                Jedi_AdjustSaberAnimLevel(NPC, (*(*NPC).client).ps.fd.saberAnimLevel - 1);
            }
            newFlags &= !SEF_HITOBJECT;
        }
        (*(*NPC).client).ps.saberEventFlags = newFlags;
    }
}

/// `static qboolean Jedi_AttackDecide( int enemy_dist )` (NPC_AI_Jedi.c:4287).
/// Decides whether to launch a saber attack this frame: bails if losing a saber
/// lock, presses a won saber lock into an attack (skill-weighted chance), follows
/// a parry/knockaway with a quick attack (Tavion/fencer/jedi-trainer), and finally
/// — if close enough, allowed to fire and not parrying — calls `WeaponThink` and,
/// when attacking, maybe starts a strafe. Returns `qtrue` if it ended up attacking.
/// No oracle (NPC-global AI: RNG, timers, `ucmd`).
///
/// # Safety
/// The `NPC`/`NPCInfo`/`ucmd` globals and `NPC->client`/`NPC->enemy` must be valid.
#[allow(dead_code)] // called by Jedi_Combat as it lands
unsafe fn Jedi_AttackDecide(enemy_dist: c_int) -> qboolean {
    if !(*(*NPC).enemy).client.is_null()
        && (*(*NPC).enemy).s.weapon == WP_SABER
        && (*(*(*NPC).enemy).client).ps.saberLockTime > (*addr_of!(level)).time
        && (*(*NPC).client).ps.saberLockTime < (*addr_of!(level)).time
    {
        //enemy is in a saberLock and we are not
        return QFALSE;
    }

    if (*(*NPC).client).ps.saberEventFlags & SEF_LOCK_WON != 0 {
        //we won a saber lock, press the advantage with an attack!
        let chance: c_int;
        if (*(*NPC).client).NPC_class == CLASS_DESANN
            || (*(*NPC).client).NPC_class == CLASS_LUKE
            || Q_stricmp(c"Yoda".as_ptr(), (*NPC).NPC_type) == 0
        {
            //desann and luke
            chance = 20;
        } else if (*(*NPC).client).NPC_class == CLASS_TAVION {
            //tavion
            chance = 10;
        } else if (*(*NPC).client).NPC_class == CLASS_REBORN && (*NPCInfo).rank == RANK_LT_JG {
            //fencer
            chance = 5;
        } else {
            chance = (*NPCInfo).rank;
        }
        if Q_irand(0, 30) < chance {
            //based on skill with some randomness
            (*(*NPC).client).ps.saberEventFlags &= !SEF_LOCK_WON; //clear this now that we are using the opportunity
            TIMER_Set(NPC, c"noRetreat".as_ptr(), Q_irand(500, 2000));
            //FIXME: check enemy_dist?
            (*(*NPC).client).ps.weaponTime = 0;
            (*NPCInfo).shotTime = 0;
            (*NPC).attackDebounceTime = 0;
            //NPC->client->ps.fd.forcePowerDebounce[FP_SABER_DEFENSE] = level.time + 500;
            (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
            WeaponThink(QTRUE);
            return QTRUE;
        }
    }

    if (*(*NPC).client).NPC_class == CLASS_TAVION
        || ((*(*NPC).client).NPC_class == CLASS_REBORN && (*NPCInfo).rank == RANK_LT_JG)
        || ((*(*NPC).client).NPC_class == CLASS_JEDI && (*NPCInfo).rank == RANK_COMMANDER)
    {
        //tavion, fencers, jedi trainer are all good at following up a parry with an attack
        if (PM_SaberInParry((*(*NPC).client).ps.saberMove) != QFALSE
            || PM_SaberInKnockaway((*(*NPC).client).ps.saberMove) != QFALSE)
            && (*(*NPC).client).ps.saberBlocked != BLOCKED_PARRY_BROKEN
        {
            //try to attack straight from a parry
            (*(*NPC).client).ps.weaponTime = 0;
            (*NPCInfo).shotTime = 0;
            (*NPC).attackDebounceTime = 0;
            //NPC->client->ps.fd.forcePowerDebounce[FP_SABER_DEFENSE] = level.time + 500;
            (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
            Jedi_AdjustSaberAnimLevel(NPC, FORCE_LEVEL_1); //try to follow-up with a quick attack
            WeaponThink(QTRUE);
            return QTRUE;
        }
    }

    //try to hit them if we can
    if enemy_dist >= 64 {
        return QFALSE;
    }

    if TIMER_Done(NPC, c"parryTime".as_ptr()) == QFALSE {
        return QFALSE;
    }

    if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 {
        //not allowed to attack
        return QFALSE;
    }

    if (ucmd.buttons & BUTTON_ATTACK) == 0 && (ucmd.buttons & BUTTON_ALT_ATTACK) == 0 {
        //not already attacking
        //Try to attack
        WeaponThink(QTRUE);
    }

    //FIXME:  Maybe try to push enemy off a ledge?

    //close enough to step forward

    //FIXME: an attack debounce timer other than the phaser debounce time?
    //		or base it on aggression?

    if ucmd.buttons & BUTTON_ATTACK != 0 {
        //attacking
        /*
        if ( enemy_dist > 32 && NPCInfo->stats.aggression >= 4 )
        {//move forward if we're too far away and we're chasing him
            ucmd.forwardmove = 127;
        }
        else if ( enemy_dist < 0 )
        {//move back if we're too close
            ucmd.forwardmove = -127;
        }
        */
        //FIXME: based on the type of parry/attack the enemy is doing and my skill,
        //		choose an attack that is likely to get around the parry?
        //		right now that's generic in the saber animation code, auto-picks
        //		a next anim for me, but really should be AI-controlled.
        //FIXME: have this interact with/override above strafing code?
        if ucmd.rightmove == 0 {
            //not already strafing
            if Q_irand(0, 3) == 0 {
                //25% chance of doing this
                let mut right: vec3_t = [0.0; 3];
                let mut dir2enemy: vec3_t = [0.0; 3];

                AngleVectors(&(*NPC).r.currentAngles, None, Some(&mut right), None);
                VectorSubtract(
                    &(*(*NPC).enemy).r.currentOrigin,
                    &(*NPC).r.currentAngles,
                    &mut dir2enemy,
                );
                if DotProduct(&right, &dir2enemy) > 0.0 {
                    //he's to my right, strafe left
                    ucmd.rightmove = -127;
                    VectorClear(&mut (*(*NPC).client).ps.moveDir);
                } else {
                    //he's to my left, strafe right
                    ucmd.rightmove = 127;
                    VectorClear(&mut (*(*NPC).client).ps.moveDir);
                }
            }
        }
        return QTRUE;
    }

    QFALSE
}

/// `evasionType_t Jedi_SaberBlockGo( gentity_t *self, usercmd_t *cmd, vec3_t pHitloc,
/// vec3_t phitDir, gentity_t *incoming, float dist )` (NPC_AI_Jedi.c:2433; `dist`
/// defaults to `0.0f`). The root of the saber-block/evasion chain: from the incoming
/// hit's height and left/right quadrant relative to the eye point, picks a parry
/// (`saberBlocked` quadrant), a duck, a dodge, a jump/force-jump, or a Boba roll /
/// Tavion butterfly — returning the chosen `evasionType_t` (or `EVASION_NONE`). On a
/// real evasion it also stops taunt/grip/drain, applies the dodge anim / duck chance,
/// converts the block for thrown missiles, and bumps the saber-defense debounce by the
/// recalculated parry time. No oracle (NPC AI: RNG, timers, anims, force powers).
///
/// # Safety
/// `self`/`self->client`/`self->NPC`, `cmd`, and `pHitloc`/`phitDir` (when `incoming`
/// is NULL) must be valid; `incoming` may be NULL.
// faithful: C's `evaded` local is write-only (the function returns on `evasionType`,
// never on `evaded`), so allow the unused-assignment + unused-variable lints.
#[allow(clippy::too_many_arguments, unused_assignments, unused_variables)]
pub unsafe fn Jedi_SaberBlockGo(
    self_: *mut gentity_t,
    cmd: *mut usercmd_t,
    pHitloc: *mut vec3_t,
    phitDir: *mut vec3_t,
    incoming: *mut gentity_t,
    dist: f32,
) -> evasionType_t {
    let mut hitloc: vec3_t = [0.0; 3];
    let mut hitdir: vec3_t = [0.0; 3];
    let mut diff: vec3_t = [0.0; 3];
    let mut fwdangles: vec3_t = [0.0, 0.0, 0.0];
    let mut right: vec3_t = [0.0; 3];
    let rightdot: f32;
    let zdiff: f32;
    let mut duckChance: c_int = 0;
    let mut dodgeAnim: c_int = -1;
    let mut saberBusy: qboolean = QFALSE;
    let mut evaded: qboolean = QFALSE;
    let mut doDodge: qboolean = QFALSE;
    let mut evasionType: evasionType_t = EVASION_NONE;

    //FIXME: if we don't have our saber in hand, pick the force throw option or a jump or strafe!
    //FIXME: reborn don't block enough anymore
    if incoming.is_null() {
        VectorCopy(&*pHitloc, &mut hitloc);
        VectorCopy(&*phitDir, &mut hitdir);
        //FIXME: maybe base this on rank some?  And/or g_spskill?
        if (*(*self_).client).ps.saberInFlight != QFALSE {
            //DOH!  do non-saber evasion!
            saberBusy = QTRUE;
        } else if Jedi_QuickReactions(self_) != QFALSE {
            //jedi trainer and tavion are must faster at parrying and can do it whenever they like
            //Also, on medium, all level 3 people can parry any time and on hard, all level 2 or 3 people can parry any time
        } else {
            saberBusy = Jedi_SaberBusy(self_);
        }
    } else {
        if (*incoming).s.weapon == WP_SABER {
            //flying lightsaber, face it!
            //FIXME: for this to actually work, we'd need to call update angles too?
            //Jedi_FaceEntity( self, incoming, qtrue );
        }
        VectorCopy(&(*incoming).r.currentOrigin, &mut hitloc);
        VectorNormalize2(&(*incoming).s.pos.trDelta, &mut hitdir);
    }
    if !(*self_).client.is_null() && (*(*self_).client).NPC_class == CLASS_BOBAFETT {
        saberBusy = QTRUE;
    }

    VectorSubtract(&hitloc, &(*(*self_).client).renderInfo.eyePoint, &mut diff);
    diff[2] = 0.0;
    //VectorNormalize( diff );
    fwdangles[1] = (*(*self_).client).ps.viewangles[1];
    // Ultimately we might care if the shot was ahead or behind, but for now, just quadrant is fine.
    AngleVectors(&fwdangles, None, Some(&mut right), None);

    rightdot = DotProduct(&right, &diff); // + flrand(-0.10f,0.10f);
                                          //totalHeight = self->client->renderInfo.eyePoint[2] - self->r.absmin[2];
    zdiff = hitloc[2] - (*(*self_).client).renderInfo.eyePoint[2]; // + Q_irand(-6,6);

    //see if we can dodge if need-be
    if (dist > 16.0 && (Q_irand(0, 2) != 0 || saberBusy != QFALSE))
        || (*(*self_).client).ps.saberInFlight != QFALSE
        || BG_SabersOff(addr_of_mut!((*(*self_).client).ps)) != QFALSE
        || (*(*self_).client).NPC_class == CLASS_BOBAFETT
    {
        //either it will miss by a bit (and 25% chance) OR our saber is not in-hand OR saber is off
        if !(*self_).NPC.is_null()
            && ((*(*self_).NPC).rank == RANK_CREWMAN || (*(*self_).NPC).rank >= RANK_LT_JG)
        {
            //acrobat or fencer or above
            if (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE //on the ground
                && ((*(*self_).client).ps.pm_flags & PMF_DUCKED) == 0 && (*cmd).upmove >= 0 && TIMER_Done( self_, c"duck".as_ptr() ) != QFALSE //not ducking
                && BG_InRoll(addr_of_mut!((*(*self_).client).ps), (*(*self_).client).ps.legsAnim) == QFALSE //not rolling
                && PM_InKnockDown(addr_of_mut!((*(*self_).client).ps)) == QFALSE //not knocked down
                && ((*(*self_).client).ps.saberInFlight != QFALSE
                    || (*(*self_).client).NPC_class == CLASS_BOBAFETT
                    || (BG_SaberInAttack((*(*self_).client).ps.saberMove) == QFALSE //not attacking
                        && PM_SaberInStart((*(*self_).client).ps.saberMove) == QFALSE //not starting an attack
                        && BG_SpinningSaberAnim((*(*self_).client).ps.torsoAnim) == QFALSE //not in a saber spin
                        && BG_SaberInSpecialAttack((*(*self_).client).ps.torsoAnim) == QFALSE))
            //not in a special attack
            {
                //need to check all these because it overrides both torso and legs with the dodge
                doDodge = QTRUE;
            }
        }
    }
    // Figure out what quadrant the block was in.
    if (*addr_of!(d_JediAI)).integer != 0 {
        Com_Printf(&format!(
            "({}) evading attack from height {:.2}, zdiff: {:.2}, rightdot: {:.2}\n",
            (*addr_of!(level)).time,
            (hitloc[2] - (*self_).r.absmin[2]) as f64,
            zdiff as f64,
            rightdot as f64,
        ));
    }

    //UL = > -1//-6
    //UR = > -6//-9
    //TOP = > +6//+4
    //FIXME: take FP_SABER_DEFENSE into account here somehow?
    if zdiff >= -5.0 {
        //was 0
        if !incoming.is_null() || saberBusy == QFALSE {
            if rightdot > 12.0
                || (rightdot > 3.0 && zdiff < 5.0)
                || (incoming.is_null() && hitdir[2].abs() < 0.25f32)
            {
                //coming from right
                if doDodge != QFALSE {
                    if (*(*self_).client).NPC_class == CLASS_BOBAFETT && Q_irand(0, 2) == 0 {
                        //roll!
                        TIMER_Start(self_, c"duck".as_ptr(), Q_irand(500, 1500));
                        TIMER_Start(self_, c"strafeLeft".as_ptr(), Q_irand(500, 1500));
                        TIMER_Set(self_, c"strafeRight".as_ptr(), 0);
                        evasionType = EVASION_DUCK;
                        evaded = QTRUE;
                    } else if Q_irand(0, 1) != 0 {
                        dodgeAnim = BOTH_DODGE_FL;
                    } else {
                        dodgeAnim = BOTH_DODGE_BL;
                    }
                } else {
                    (*(*self_).client).ps.saberBlocked = BLOCKED_UPPER_RIGHT;
                    evasionType = EVASION_PARRY;
                    if (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE {
                        if zdiff > 5.0 {
                            TIMER_Start(self_, c"duck".as_ptr(), Q_irand(500, 1500));
                            evasionType = EVASION_DUCK_PARRY;
                            evaded = QTRUE;
                            if (*addr_of!(d_JediAI)).integer != 0 {
                                Com_Printf("duck ");
                            }
                        } else {
                            duckChance = 6;
                        }
                    }
                }
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf("UR block\n");
                }
            } else if rightdot < -12.0
                || (rightdot < -3.0 && zdiff < 5.0)
                || (incoming.is_null() && hitdir[2].abs() < 0.25f32)
            {
                //coming from left
                if doDodge != QFALSE {
                    if (*(*self_).client).NPC_class == CLASS_BOBAFETT && Q_irand(0, 2) == 0 {
                        //roll!
                        TIMER_Start(self_, c"duck".as_ptr(), Q_irand(500, 1500));
                        TIMER_Start(self_, c"strafeRight".as_ptr(), Q_irand(500, 1500));
                        TIMER_Set(self_, c"strafeLeft".as_ptr(), 0);
                        evasionType = EVASION_DUCK;
                        evaded = QTRUE;
                    } else if Q_irand(0, 1) != 0 {
                        dodgeAnim = BOTH_DODGE_FR;
                    } else {
                        dodgeAnim = BOTH_DODGE_BR;
                    }
                } else {
                    (*(*self_).client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
                    evasionType = EVASION_PARRY;
                    if (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE {
                        if zdiff > 5.0 {
                            TIMER_Start(self_, c"duck".as_ptr(), Q_irand(500, 1500));
                            evasionType = EVASION_DUCK_PARRY;
                            evaded = QTRUE;
                            if (*addr_of!(d_JediAI)).integer != 0 {
                                Com_Printf("duck ");
                            }
                        } else {
                            duckChance = 6;
                        }
                    }
                }
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf("UL block\n");
                }
            } else {
                (*(*self_).client).ps.saberBlocked = BLOCKED_TOP;
                evasionType = EVASION_PARRY;
                if (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE {
                    duckChance = 4;
                }
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf("TOP block\n");
                }
            }
            evaded = QTRUE;
        } else if (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE {
            //duckChance = 2;
            TIMER_Start(self_, c"duck".as_ptr(), Q_irand(500, 1500));
            evasionType = EVASION_DUCK;
            evaded = QTRUE;
            if (*addr_of!(d_JediAI)).integer != 0 {
                Com_Printf("duck ");
            }
        }
    }
    //LL = -22//= -18 to -39
    //LR = -23//= -20 to -41
    else if zdiff > -22.0 {
        //was-15 )
        if true {
            //zdiff < -10 )
            //hmm, pretty low, but not low enough to use the low block, so we need to duck
            if (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE {
                //duckChance = 2;
                TIMER_Start(self_, c"duck".as_ptr(), Q_irand(500, 1500));
                evasionType = EVASION_DUCK;
                evaded = QTRUE;
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf("duck ");
                }
            } else {
                //in air!  Ducking does no good
            }
        }
        if !incoming.is_null() || saberBusy == QFALSE {
            if rightdot > 8.0 || (rightdot > 3.0 && zdiff < -11.0) {
                //was normalized, 0.2
                if doDodge != QFALSE {
                    if (*(*self_).client).NPC_class == CLASS_BOBAFETT && Q_irand(0, 2) == 0 {
                        //roll!
                        TIMER_Start(self_, c"strafeLeft".as_ptr(), Q_irand(500, 1500));
                        TIMER_Set(self_, c"strafeRight".as_ptr(), 0);
                    } else {
                        dodgeAnim = BOTH_DODGE_L;
                    }
                } else {
                    (*(*self_).client).ps.saberBlocked = BLOCKED_UPPER_RIGHT;
                    if evasionType == EVASION_DUCK {
                        evasionType = EVASION_DUCK_PARRY;
                    } else {
                        evasionType = EVASION_PARRY;
                    }
                }
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf("mid-UR block\n");
                }
            } else if rightdot < -8.0 || (rightdot < -3.0 && zdiff < -11.0) {
                //was normalized, -0.2
                if doDodge != QFALSE {
                    if (*(*self_).client).NPC_class == CLASS_BOBAFETT && Q_irand(0, 2) == 0 {
                        //roll!
                        TIMER_Start(self_, c"strafeLeft".as_ptr(), Q_irand(500, 1500));
                        TIMER_Set(self_, c"strafeRight".as_ptr(), 0);
                    } else {
                        dodgeAnim = BOTH_DODGE_R;
                    }
                } else {
                    (*(*self_).client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
                    if evasionType == EVASION_DUCK {
                        evasionType = EVASION_DUCK_PARRY;
                    } else {
                        evasionType = EVASION_PARRY;
                    }
                }
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf("mid-UL block\n");
                }
            } else {
                (*(*self_).client).ps.saberBlocked = BLOCKED_TOP;
                if evasionType == EVASION_DUCK {
                    evasionType = EVASION_DUCK_PARRY;
                } else {
                    evasionType = EVASION_PARRY;
                }
                if (*addr_of!(d_JediAI)).integer != 0 {
                    Com_Printf("mid-TOP block\n");
                }
            }
            evaded = QTRUE;
        }
    } else if saberBusy != QFALSE || (zdiff < -36.0 && (zdiff < -44.0 || Q_irand(0, 2) == 0)) {
        //was -30 and -40//2nd one was -46
        //jump!
        if (*(*self_).client).ps.groundEntityNum == ENTITYNUM_NONE {
            //already in air, duck to pull up legs
            TIMER_Start(self_, c"duck".as_ptr(), Q_irand(500, 1500));
            evasionType = EVASION_DUCK;
            evaded = QTRUE;
            if (*addr_of!(d_JediAI)).integer != 0 {
                Com_Printf("legs up\n");
            }
            if !incoming.is_null() || saberBusy == QFALSE {
                //since the jump may be cleared if not safe, set a lower block too
                if rightdot >= 0.0 {
                    (*(*self_).client).ps.saberBlocked = BLOCKED_LOWER_RIGHT;
                    evasionType = EVASION_DUCK_PARRY;
                    if (*addr_of!(d_JediAI)).integer != 0 {
                        Com_Printf("LR block\n");
                    }
                } else {
                    (*(*self_).client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
                    evasionType = EVASION_DUCK_PARRY;
                    if (*addr_of!(d_JediAI)).integer != 0 {
                        Com_Printf("LL block\n");
                    }
                }
                evaded = QTRUE;
            }
        } else {
            //gotta jump!
            if !(*self_).NPC.is_null()
                && ((*(*self_).NPC).rank == RANK_CREWMAN || (*(*self_).NPC).rank > RANK_LT_JG)
                && (Q_irand(0, 10) == 0
                    || (Q_irand(0, 2) == 0 && ((*cmd).forwardmove != 0 || (*cmd).rightmove != 0)))
            {
                //superjump
                //FIXME: check the jump, if can't, then block
                if !(*self_).NPC.is_null()
                    && ((*(*self_).NPC).scriptFlags & SCF_NO_ACROBATICS) == 0
                    && (*(*self_).client).ps.fd.forceRageRecoveryTime < (*addr_of!(level)).time
                    && ((*(*self_).client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                    && PM_InKnockDown(addr_of_mut!((*(*self_).client).ps)) == QFALSE
                {
                    (*(*self_).client).ps.fd.forceJumpCharge = 320.0; //FIXME: calc this intelligently
                    evasionType = EVASION_FJUMP;
                    evaded = QTRUE;
                    if (*addr_of!(d_JediAI)).integer != 0 {
                        Com_Printf("force jump + ");
                    }
                }
            } else {
                //normal jump
                //FIXME: check the jump, if can't, then block
                if !(*self_).NPC.is_null()
                    && ((*(*self_).NPC).scriptFlags & SCF_NO_ACROBATICS) == 0
                    && (*(*self_).client).ps.fd.forceRageRecoveryTime < (*addr_of!(level)).time
                    && ((*(*self_).client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                {
                    if (*(*self_).client).NPC_class == CLASS_BOBAFETT && Q_irand(0, 1) == 0 {
                        //roll!
                        if rightdot > 0.0 {
                            TIMER_Start(self_, c"strafeLeft".as_ptr(), Q_irand(500, 1500));
                            TIMER_Set(self_, c"strafeRight".as_ptr(), 0);
                            TIMER_Set(self_, c"walking".as_ptr(), 0);
                        } else {
                            TIMER_Start(self_, c"strafeRight".as_ptr(), Q_irand(500, 1500));
                            TIMER_Set(self_, c"strafeLeft".as_ptr(), 0);
                            TIMER_Set(self_, c"walking".as_ptr(), 0);
                        }
                    } else if self_ == NPC {
                        (*cmd).upmove = 127;
                    } else {
                        (*(*self_).client).ps.velocity[2] = JUMP_VELOCITY as f32;
                    }
                    evasionType = EVASION_JUMP;
                    evaded = QTRUE;
                    if (*addr_of!(d_JediAI)).integer != 0 {
                        Com_Printf("jump + ");
                    }
                }
                if (*(*self_).client).NPC_class == CLASS_TAVION {
                    if incoming.is_null()
                        && (*(*self_).client).ps.groundEntityNum < ENTITYNUM_NONE
                        && Q_irand(0, 2) == 0
                    {
                        if BG_SaberInAttack((*(*self_).client).ps.saberMove) == QFALSE
                            && PM_SaberInStart((*(*self_).client).ps.saberMove) == QFALSE
                            && BG_InRoll(
                                addr_of_mut!((*(*self_).client).ps),
                                (*(*self_).client).ps.legsAnim,
                            ) == QFALSE
                            && PM_InKnockDown(addr_of_mut!((*(*self_).client).ps)) == QFALSE
                            && BG_SaberInSpecialAttack((*(*self_).client).ps.torsoAnim) == QFALSE
                        {
                            //do the butterfly!
                            let butterflyAnim: c_int = if Q_irand(0, 1) != 0 {
                                BOTH_BUTTERFLY_LEFT
                            } else {
                                BOTH_BUTTERFLY_RIGHT
                            };
                            evasionType = EVASION_CARTWHEEL;
                            NPC_SetAnim(
                                self_,
                                SETANIM_BOTH,
                                butterflyAnim,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                            (*(*self_).client).ps.velocity[2] = 225.0;
                            (*(*self_).client).ps.fd.forceJumpZStart = (*self_).r.currentOrigin[2]; //so we don't take damage if we land at same height
                                                                                                   //	self->client->ps.pm_flags |= PMF_JUMPING|PMF_SLOW_MO_FALL;
                                                                                                   //	self->client->ps.SaberActivateTrail( 300 );//FIXME: reset this when done!
                                                                                                   //Ah well. No hacking from the server for now.
                            if (*(*self_).client).NPC_class == CLASS_BOBAFETT {
                                G_AddEvent(self_, EV_JUMP, 0);
                            } else {
                                G_Sound(
                                    self_,
                                    CHAN_BODY,
                                    G_SoundIndex("sound/weapons/force/jump.wav"),
                                );
                            }
                            (*cmd).upmove = 0;
                            saberBusy = QTRUE;
                            evaded = QTRUE;
                        }
                    }
                }
            }
            evasionType = Jedi_CheckFlipEvasions(self_, rightdot, zdiff);
            if evasionType != EVASION_NONE {
                if (*addr_of!(d_slowmodeath)).integer > 5
                    && !(*self_).enemy.is_null()
                    && (*(*self_).enemy).s.number == 0
                {
                    G_StartMatrixEffect(self_);
                }
                saberBusy = QTRUE;
                evaded = QTRUE;
            } else if !incoming.is_null() || saberBusy == QFALSE {
                //since the jump may be cleared if not safe, set a lower block too
                if rightdot >= 0.0 {
                    (*(*self_).client).ps.saberBlocked = BLOCKED_LOWER_RIGHT;
                    if evasionType == EVASION_JUMP {
                        evasionType = EVASION_JUMP_PARRY;
                    } else if evasionType == EVASION_NONE {
                        evasionType = EVASION_PARRY;
                    }
                    if (*addr_of!(d_JediAI)).integer != 0 {
                        Com_Printf("LR block\n");
                    }
                } else {
                    (*(*self_).client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
                    if evasionType == EVASION_JUMP {
                        evasionType = EVASION_JUMP_PARRY;
                    } else if evasionType == EVASION_NONE {
                        evasionType = EVASION_PARRY;
                    }
                    if (*addr_of!(d_JediAI)).integer != 0 {
                        Com_Printf("LL block\n");
                    }
                }
                evaded = QTRUE;
            }
        }
    } else if !incoming.is_null() || saberBusy == QFALSE {
        if rightdot >= 0.0 {
            (*(*self_).client).ps.saberBlocked = BLOCKED_LOWER_RIGHT;
            evasionType = EVASION_PARRY;
            if (*addr_of!(d_JediAI)).integer != 0 {
                Com_Printf("LR block\n");
            }
        } else {
            (*(*self_).client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
            evasionType = EVASION_PARRY;
            if (*addr_of!(d_JediAI)).integer != 0 {
                Com_Printf("LL block\n");
            }
        }
        if !incoming.is_null() && (*incoming).s.weapon == WP_SABER {
            //thrown saber!
            if !(*self_).NPC.is_null()
                && ((*(*self_).NPC).rank == RANK_CREWMAN || (*(*self_).NPC).rank > RANK_LT_JG)
                && (Q_irand(0, 10) == 0
                    || (Q_irand(0, 2) == 0 && ((*cmd).forwardmove != 0 || (*cmd).rightmove != 0)))
            {
                //superjump
                //FIXME: check the jump, if can't, then block
                if !(*self_).NPC.is_null()
                    && ((*(*self_).NPC).scriptFlags & SCF_NO_ACROBATICS) == 0
                    && (*(*self_).client).ps.fd.forceRageRecoveryTime < (*addr_of!(level)).time
                    && ((*(*self_).client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                    && PM_InKnockDown(addr_of_mut!((*(*self_).client).ps)) == QFALSE
                {
                    (*(*self_).client).ps.fd.forceJumpCharge = 320.0; //FIXME: calc this intelligently
                    evasionType = EVASION_FJUMP;
                    if (*addr_of!(d_JediAI)).integer != 0 {
                        Com_Printf("force jump + ");
                    }
                }
            } else {
                //normal jump
                //FIXME: check the jump, if can't, then block
                if !(*self_).NPC.is_null()
                    && ((*(*self_).NPC).scriptFlags & SCF_NO_ACROBATICS) == 0
                    && (*(*self_).client).ps.fd.forceRageRecoveryTime < (*addr_of!(level)).time
                    && ((*(*self_).client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                {
                    if self_ == NPC {
                        (*cmd).upmove = 127;
                    } else {
                        (*(*self_).client).ps.velocity[2] = JUMP_VELOCITY as f32;
                    }
                    evasionType = EVASION_JUMP_PARRY;
                    if (*addr_of!(d_JediAI)).integer != 0 {
                        Com_Printf("jump + ");
                    }
                }
            }
        }
        evaded = QTRUE;
    }

    if evasionType == EVASION_NONE {
        return EVASION_NONE;
    }
    //stop taunting
    TIMER_Set(self_, c"taunting".as_ptr(), 0);
    //stop gripping
    TIMER_Set(self_, c"gripping".as_ptr(), -(*addr_of!(level)).time);
    WP_ForcePowerStop(self_, FP_GRIP);
    //stop draining
    TIMER_Set(self_, c"draining".as_ptr(), -(*addr_of!(level)).time);
    WP_ForcePowerStop(self_, FP_DRAIN);

    if dodgeAnim != -1 {
        //dodged
        evasionType = EVASION_DODGE;
        NPC_SetAnim(
            self_,
            SETANIM_BOTH,
            dodgeAnim,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        (*(*self_).client).ps.weaponTime = (*(*self_).client).ps.torsoTimer;
        //force them to stop moving in this case
        (*(*self_).client).ps.pm_time = (*(*self_).client).ps.torsoTimer;
        //FIXME: maybe make a sound?  Like a grunt?  EV_JUMP?
        (*(*self_).client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
        //dodged, not block
        if (*addr_of!(d_slowmodeath)).integer > 5
            && !(*self_).enemy.is_null()
            && (*(*self_).enemy).s.number == 0
        {
            G_StartMatrixEffect(self_);
        }
    } else {
        if duckChance != 0 {
            if Q_irand(0, duckChance) == 0 {
                TIMER_Start(self_, c"duck".as_ptr(), Q_irand(500, 1500));
                if evasionType == EVASION_PARRY {
                    evasionType = EVASION_DUCK_PARRY;
                } else {
                    evasionType = EVASION_DUCK;
                }
                /*
                if ( d_JediAI.integer )
                {
                    Com_Printf( "duck " );
                }
                */
            }
        }

        if !incoming.is_null() {
            (*(*self_).client).ps.saberBlocked =
                WP_MissileBlockForBlock((*(*self_).client).ps.saberBlocked);
        }
    }
    //if ( self->client->ps.saberBlocked != BLOCKED_NONE )
    {
        let parryReCalcTime: c_int = Jedi_ReCalcParryTime(self_, evasionType);
        if (*(*self_).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize]
            < (*addr_of!(level)).time + parryReCalcTime
        {
            (*(*self_).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                (*addr_of!(level)).time + parryReCalcTime;
        }
    }
    evasionType
}

/// `static qboolean Jedi_SaberBlock( int saberNum, int bladeNum )` (NPC_AI_Jedi.c:3091;
/// both args default to 0). Reactively parries/dodges the enemy's swinging saber: it
/// extrapolates the enemy blade tip from this/last frame's muzzle point+dir, finds the
/// closest approach between that blade and the NPC's vertical body line
/// (`ShortestLineSegBewteen2LineSegs`), bails if too far, traces the swing to an impact
/// point, then calls `Jedi_SaberBlockGo` to pick a block/dodge and sets the
/// parry/parryReCalc timers accordingly. No oracle (NPC AI: trap_Trace, RNG, timers,
/// renderInfo/saber-blade geometry).
///
/// # Safety
/// `NPC`/`NPC->client`/`NPCInfo`/`NPC->enemy` and `NPC->enemy->client` must be valid;
/// `saberNum`/`bladeNum` must index live saber/blade slots.
#[allow(dead_code)] // called by Jedi_EvasionSaber as it lands
unsafe fn Jedi_SaberBlock(saberNum: c_int, bladeNum: c_int) -> qboolean {
    let saberNum = saberNum as usize;
    let bladeNum = bladeNum as usize;
    let mut hitloc: vec3_t = [0.0; 3];
    let mut saberTipOld: vec3_t = [0.0; 3];
    let mut saberTip: vec3_t = [0.0; 3];
    let mut top: vec3_t = [0.0; 3];
    let mut bottom: vec3_t = [0.0; 3];
    let mut axisPoint: vec3_t = [0.0; 3];
    let mut saberPoint: vec3_t = [0.0; 3]; //saberBase,
    let mut dir: vec3_t = [0.0; 3];
    let mut pointDir: vec3_t = [0.0; 3];
    let mut baseDir: vec3_t = [0.0; 3];
    let mut tipDir: vec3_t = [0.0; 3];
    let mut saberHitPoint: vec3_t = [0.0; 3];
    let mut saberMins: vec3_t = [0.0; 3];
    let mut saberMaxs: vec3_t = [0.0; 3];
    let pointDist: f32;
    let baseDirPerc: f32;
    let mut dist: f32;
    #[allow(unused_assignments)]
    let mut bladeLen: f32 = 0.0;
    let tr: trace_t;
    let evasionType: evasionType_t;

    //FIXME: reborn don't block enough anymore
    /*
    //maybe do this on easy only... or only on grunt-level reborn
    if ( NPC->client->ps.weaponTime )
    {//i'm attacking right now
        return qfalse;
    }
    */

    if TIMER_Done(NPC, c"parryReCalcTime".as_ptr()) == QFALSE {
        //can't do our own re-think of which parry to use yet
        return QFALSE;
    }

    if (*(*NPC).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] > (*addr_of!(level)).time
    {
        //can't move the saber to another position yet
        return QFALSE;
    }

    /*
    if ( NPCInfo->rank < RANK_LT_JG && Q_irand( 0, (2 - g_spskill.integer) ) )
    {//lower rank reborn have a random chance of not doing it at all
        NPC->client->ps.fd.forcePowerDebounce[FP_SABER_DEFENSE] = level.time + 300;
        return qfalse;
    }
    */

    if (*(*NPC).enemy).health <= 0 || (*(*NPC).enemy).client.is_null() {
        //don't keep blocking him once he's dead (or if not a client)
        return QFALSE;
    }
    /*
    //VectorMA( NPC->enemy->client->renderInfo.muzzlePoint, NPC->enemy->client->ps.saberLength, NPC->enemy->client->renderInfo.muzzleDir, saberTip );
    //VectorMA( NPC->enemy->client->renderInfo.muzzlePointNext, NPC->enemy->client->ps.saberLength, NPC->enemy->client->renderInfo.muzzleDirNext, saberTipNext );
    VectorMA( NPC->enemy->client->renderInfo.muzzlePointOld, NPC->enemy->client->ps.saberLength, NPC->enemy->client->renderInfo.muzzleDirOld, saberTipOld );
    VectorMA( NPC->enemy->client->renderInfo.muzzlePoint, NPC->enemy->client->ps.saberLength, NPC->enemy->client->renderInfo.muzzleDir, saberTip );

    VectorSubtract( NPC->enemy->client->renderInfo.muzzlePoint, NPC->enemy->client->renderInfo.muzzlePointOld, dir );//get the dir
    VectorAdd( dir, NPC->enemy->client->renderInfo.muzzlePoint, saberBase );//extrapolate

    VectorSubtract( saberTip, saberTipOld, dir );//get the dir
    VectorAdd( dir, saberTip, saberTipOld );//extrapolate

    VectorCopy( NPC->r.currentOrigin, top );
    top[2] = NPC->r.absmax[2];
    VectorCopy( NPC->r.currentOrigin, bottom );
    bottom[2] = NPC->r.absmin[2];

    float dist = ShortestLineSegBewteen2LineSegs( saberBase, saberTipOld, bottom, top, saberPoint, axisPoint );
    if ( 0 )//dist > NPC->r.maxs[0]*4 )//was *3
    {//FIXME: sometimes he reacts when you're too far away to actually hit him
        if ( d_JediAI.integer )
        {
            Com_Printf( "enemy saber dist: %4.2f\n", dist );
        }
        TIMER_Set( NPC, "parryTime", -1 );
        return qfalse;
    }

    //get the actual point of impact
    trace_t	tr;
    trap_Trace( &tr, saberPoint, vec3_origin, vec3_origin, axisPoint, NPC->enemy->s.number, MASK_SHOT, G2_RETURNONHIT, 10 );
    if ( tr.allsolid || tr.startsolid )
    {//estimate
        VectorSubtract( saberPoint, axisPoint, dir );
        VectorNormalize( dir );
        VectorMA( axisPoint, NPC->r.maxs[0]*1.22, dir, hitloc );
    }
    else
    {
        VectorCopy( tr.endpos, hitloc );
    }
    */
    VectorSet(&mut saberMins, -4.0, -4.0, -4.0);
    VectorSet(&mut saberMaxs, 4.0, 4.0, 4.0);

    VectorMA(
        &(*(*(*NPC).enemy).client).saber[saberNum].blade[bladeNum].muzzlePointOld,
        (*(*(*NPC).enemy).client).saber[saberNum].blade[bladeNum].length,
        &(*(*(*NPC).enemy).client).saber[saberNum].blade[bladeNum].muzzleDirOld,
        &mut saberTipOld,
    );
    VectorMA(
        &(*(*(*NPC).enemy).client).saber[saberNum].blade[bladeNum].muzzlePoint,
        (*(*(*NPC).enemy).client).saber[saberNum].blade[bladeNum].length,
        &(*(*(*NPC).enemy).client).saber[saberNum].blade[bladeNum].muzzleDir,
        &mut saberTip,
    );
    //	VectorCopy(NPC->enemy->client->lastSaberBase_Always, muzzlePoint);
    //	VectorMA(muzzlePoint, GAME_SABER_LENGTH, NPC->enemy->client->lastSaberDir_Always, saberTip);
    //	VectorCopy(saberTip, saberTipOld);

    VectorCopy(&(*NPC).r.currentOrigin, &mut top);
    top[2] = (*NPC).r.absmax[2];
    VectorCopy(&(*NPC).r.currentOrigin, &mut bottom);
    bottom[2] = (*NPC).r.absmin[2];

    dist = ShortestLineSegBewteen2LineSegs(
        &(*(*(*NPC).enemy).client).renderInfo.muzzlePoint,
        &saberTip,
        &bottom,
        &top,
        &mut saberPoint,
        &mut axisPoint,
    );
    if dist > (*NPC).r.maxs[0] * 5.0 {
        //was *3
        //FIXME: sometimes he reacts when you're too far away to actually hit him
        if d_JediAI.integer != 0 {
            // S_COLOR_RED"..." — the color macro expands to the literal "^1" prefix.
            Com_Printf(&format!("^1enemy saber dist: {:4.2}\n", dist));
        }
        /*
        if ( dist < 300 //close
            && !Jedi_QuickReactions( NPC )//quick reaction people can interrupt themselves
            && (PM_SaberInStart( NPC->enemy->client->ps.saberMove ) || BG_SaberInAttack( NPC->enemy->client->ps.saberMove )) )//enemy is swinging at me
        {//he's swinging at me and close enough to be a threat, don't start an attack right now
            TIMER_Set( NPC, "parryTime", 100 );
        }
        else
        */
        {
            TIMER_Set(NPC, c"parryTime".as_ptr(), -1);
        }
        return QFALSE;
    }
    if d_JediAI.integer != 0 {
        // S_COLOR_GREEN"..." — the color macro expands to the literal "^2" prefix.
        Com_Printf(&format!("^2enemy saber dist: {:4.2}\n", dist));
    }

    VectorSubtract(
        &saberPoint,
        &(*(*(*NPC).enemy).client).renderInfo.muzzlePoint,
        &mut pointDir,
    );
    pointDist = VectorLength(&pointDir);

    bladeLen = (*(*(*NPC).enemy).client).saber[saberNum].blade[bladeNum].length;

    if bladeLen <= 0.0 {
        baseDirPerc = 0.5;
    } else {
        baseDirPerc = pointDist / bladeLen;
    }
    VectorSubtract(
        &(*(*(*NPC).enemy).client).renderInfo.muzzlePoint,
        &(*(*(*NPC).enemy).client).renderInfo.muzzlePointOld,
        &mut baseDir,
    );
    VectorSubtract(&saberTip, &saberTipOld, &mut tipDir);
    // VectorScale( baseDir, baseDirPerc, baseDir ) — in-place; copy src so the borrow
    // checker accepts the aliasing.
    let baseDirSrc = baseDir;
    VectorScale(&baseDirSrc, baseDirPerc, &mut baseDir);
    VectorMA(&baseDir, 1.0 - baseDirPerc, &tipDir, &mut dir);
    VectorMA(&saberPoint, 200.0, &dir, &mut hitloc);

    //get the actual point of impact
    tr = trap::Trace(
        &saberPoint,
        &saberMins,
        &saberMaxs,
        &hitloc,
        (*(*NPC).enemy).s.number,
        CONTENTS_BODY,
    ); //, G2_RETURNONHIT, 10 );
    if tr.allsolid != 0 || tr.startsolid != 0 || tr.fraction >= 1.0 {
        //estimate
        let mut dir2Me: vec3_t = [0.0; 3];
        VectorSubtract(&axisPoint, &saberPoint, &mut dir2Me);
        dist = VectorNormalize(&mut dir2Me);
        if DotProduct(&dir, &dir2Me) < 0.2 {
            //saber is not swinging in my direction
            /*
            if ( dist < 300 //close
                && !Jedi_QuickReactions( NPC )//quick reaction people can interrupt themselves
                && (PM_SaberInStart( NPC->enemy->client->ps.saberMove ) || BG_SaberInAttack( NPC->enemy->client->ps.saberMove )) )//enemy is swinging at me
            {//he's swinging at me and close enough to be a threat, don't start an attack right now
                TIMER_Set( NPC, "parryTime", 100 );
            }
            else
            */
            {
                TIMER_Set(NPC, c"parryTime".as_ptr(), -1);
            }
            return QFALSE;
        }
        // ShortestLineSegBewteen2LineSegs( saberPoint, hitloc, bottom, top, saberHitPoint,
        // hitloc ) — `hitloc` is both end1 input and close_pnt2 output; copy the input.
        let hitlocIn = hitloc;
        ShortestLineSegBewteen2LineSegs(
            &saberPoint,
            &hitlocIn,
            &bottom,
            &top,
            &mut saberHitPoint,
            &mut hitloc,
        );
        /*
        VectorSubtract( saberPoint, axisPoint, dir );
        VectorNormalize( dir );
        VectorMA( axisPoint, NPC->r.maxs[0]*1.22, dir, hitloc );
        */
    } else {
        VectorCopy(&tr.endpos, &mut hitloc);
    }

    if d_JediAI.integer != 0 {
        //G_DebugLine( saberPoint, hitloc, FRAMETIME, WPDEBUG_SaberColor( NPC->enemy->client->ps.saber[saberNum].blade[bladeNum].color ), qtrue );
        G_TestLine(&saberPoint, &hitloc, 0x0000ff, FRAMETIME);
    }

    //FIXME: if saber is off and/or we have force speed and want to be really cocky,
    //		and the swing misses by some amount, we can use the dodges here... :)
    evasionType = Jedi_SaberBlockGo(
        NPC,
        addr_of_mut!(ucmd),
        &mut hitloc,
        &mut dir,
        null_mut(),
        dist,
    );
    if evasionType != EVASION_DODGE {
        //we did block (not dodge)
        let parryReCalcTime: c_int;

        if (*(*NPC).client).ps.saberInFlight == QFALSE {
            //make sure saber is on
            WP_ActivateSaber(NPC);
        }

        //debounce our parry recalc time
        parryReCalcTime = Jedi_ReCalcParryTime(NPC, evasionType);
        TIMER_Set(NPC, c"parryReCalcTime".as_ptr(), Q_irand(0, parryReCalcTime));
        if d_JediAI.integer != 0 {
            Com_Printf(&format!(
                "Keep parry choice until: {}\n",
                (*addr_of!(level)).time + parryReCalcTime
            ));
        }

        //determine how long to hold this anim
        if TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE {
            if (*(*NPC).client).NPC_class == CLASS_TAVION {
                TIMER_Set(
                    NPC,
                    c"parryTime".as_ptr(),
                    Q_irand(parryReCalcTime / 2, (parryReCalcTime as f32 * 1.5) as c_int),
                );
            } else if (*NPCInfo).rank >= RANK_LT_JG {
                //fencers and higher hold a parry less
                TIMER_Set(NPC, c"parryTime".as_ptr(), parryReCalcTime);
            } else {
                //others hold it longer
                TIMER_Set(NPC, c"parryTime".as_ptr(), Q_irand(1, 2) * parryReCalcTime);
            }
        }
    } else {
        let mut dodgeTime: c_int = (*(*NPC).client).ps.torsoTimer;
        if (*NPCInfo).rank > RANK_LT_COMM && (*(*NPC).client).NPC_class != CLASS_DESANN {
            //higher-level guys can dodge faster
            dodgeTime -= 200;
        }
        TIMER_Set(NPC, c"parryReCalcTime".as_ptr(), dodgeTime);
        TIMER_Set(NPC, c"parryTime".as_ptr(), dodgeTime);
    }
    let _ = bladeLen;
    QTRUE
}

/// `static void Jedi_EvasionSaber( vec3_t enemy_movedir, float enemy_dist, vec3_t
/// enemy_dir )` (NPC_AI_Jedi.c:3328). Defends against an enemy who is using/swinging
/// a saber, throwing one, or shooting lightning: builds an evasion chance from what
/// the enemy is doing, checks whether they're coming at / facing us, then picks one of
/// push / `Jedi_SaberBlock` / strafe / force-jump as the response (the
/// `whichDefense` switch). No oracle (NPC AI: RNG, timers, force powers, anims).
///
/// # Safety
/// `NPC`/`NPC->client`/`NPCInfo`/`NPC->enemy` must be valid; `NPC->enemy->client` is
/// guarded. `enemy_movedir`/`enemy_dir` must be valid 3-vectors.
#[allow(dead_code)] // called by Jedi_Combat as it lands
unsafe fn Jedi_EvasionSaber(enemy_movedir: &vec3_t, enemy_dist: f32, enemy_dir: &vec3_t) {
    let mut dirEnemy2Me: vec3_t = [0.0; 3];
    let mut evasionChance: c_int = 30; //only step aside 30% if he's moving at me but not attacking
    let mut enemy_attacking: qboolean = QFALSE;
    let mut throwing_saber: qboolean = QFALSE;
    let mut shooting_lightning: qboolean = QFALSE;

    if (*(*NPC).enemy).client.is_null() {
        return;
    } else if !(*(*NPC).enemy).client.is_null()
        && (*(*NPC).enemy).s.weapon == WP_SABER
        && (*(*(*NPC).enemy).client).ps.saberLockTime > (*addr_of!(level)).time
    {
        //don't try to block/evade an enemy who is in a saberLock
        return;
    } else if (*(*NPC).client).ps.saberEventFlags & SEF_LOCK_WON != 0
        && (*(*NPC).enemy).painDebounceTime > (*addr_of!(level)).time
    {
        //pressing the advantage of winning a saber lock
        return;
    }

    if (*(*(*NPC).enemy).client).ps.saberInFlight != QFALSE
        && TIMER_Done(NPC, c"taunting".as_ptr()) == QFALSE
    {
        //if he's throwing his saber, stop taunting
        TIMER_Set(NPC, c"taunting".as_ptr(), -(*addr_of!(level)).time);
        if (*(*NPC).client).ps.saberInFlight == QFALSE {
            WP_ActivateSaber(NPC);
        }
    }

    if TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE {
        if (*(*NPC).client).ps.saberBlocked != BLOCKED_ATK_BOUNCE
            && (*(*NPC).client).ps.saberBlocked != BLOCKED_PARRY_BROKEN
        {
            //wasn't blocked myself
            (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
        }
    }

    if (*(*(*NPC).enemy).client).ps.weaponTime != 0
        && (*(*(*NPC).enemy).client).ps.weaponstate == WEAPON_FIRING
    {
        if (*(*NPC).client).ps.saberInFlight == QFALSE && Jedi_SaberBlock(0, 0) != QFALSE {
            return;
        }
    }

    VectorSubtract(
        &(*NPC).r.currentOrigin,
        &(*(*NPC).enemy).r.currentOrigin,
        &mut dirEnemy2Me,
    );
    VectorNormalize(&mut dirEnemy2Me);

    if (*(*(*NPC).enemy).client).ps.weaponTime != 0
        && (*(*(*NPC).enemy).client).ps.weaponstate == WEAPON_FIRING
    {
        //enemy is attacking
        enemy_attacking = QTRUE;
        evasionChance = 90;
    }

    if (*(*(*NPC).enemy).client).ps.fd.forcePowersActive & (1 << FP_LIGHTNING) != 0 {
        //enemy is shooting lightning
        enemy_attacking = QTRUE;
        shooting_lightning = QTRUE;
        evasionChance = 50;
    }

    if (*(*(*NPC).enemy).client).ps.saberInFlight != QFALSE
        && (*(*(*NPC).enemy).client).ps.saberEntityNum != ENTITYNUM_NONE
        && (*(*(*NPC).enemy).client).ps.saberEntityState != SES_RETURNING
    {
        //enemy is shooting lightning
        enemy_attacking = QTRUE;
        throwing_saber = QTRUE;
    }

    //FIXME: this needs to take skill and rank(reborn type) into account much more
    if Q_irand(0, 100) < evasionChance {
        //check to see if he's coming at me
        let facingAmt: f32;
        if VectorCompare(enemy_movedir, &vec3_origin) != QFALSE
            || shooting_lightning != QFALSE
            || throwing_saber != QFALSE
        {
            //he's not moving (or he's using a ranged attack), see if he's facing me
            let mut enemy_fwd: vec3_t = [0.0; 3];
            AngleVectors(
                &(*(*(*NPC).enemy).client).ps.viewangles,
                Some(&mut enemy_fwd),
                None,
                None,
            );
            facingAmt = DotProduct(&enemy_fwd, &dirEnemy2Me);
        } else {
            //he's moving
            facingAmt = DotProduct(enemy_movedir, &dirEnemy2Me);
        }

        if flrand(0.25, 1.0) < facingAmt {
            //coming at/facing me!
            let mut whichDefense: c_int = 0;
            if (*(*NPC).client).ps.weaponTime != 0
                || (*(*NPC).client).ps.saberInFlight != QFALSE
                || (*(*NPC).client).NPC_class == CLASS_BOBAFETT
            {
                //I'm attacking or recovering from a parry, can only try to strafe/jump right now
                if Q_irand(0, 10) < (*NPCInfo).stats.aggression {
                    return;
                }
                whichDefense = 100;
            } else {
                if shooting_lightning != QFALSE {
                    //check for lightning attack
                    //only valid defense is strafe and/or jump
                    whichDefense = 100;
                } else if throwing_saber != QFALSE {
                    //he's thrown his saber!  See if it's coming at me
                    let saberDist: f32;
                    let mut saberDir2Me: vec3_t = [0.0; 3];
                    let mut saberMoveDir: vec3_t = [0.0; 3];
                    let saber: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*(*NPC).enemy).client).ps.saberEntityNum as usize);
                    VectorSubtract(
                        &(*NPC).r.currentOrigin,
                        &(*saber).r.currentOrigin,
                        &mut saberDir2Me,
                    );
                    saberDist = VectorNormalize(&mut saberDir2Me);
                    VectorCopy(&(*saber).s.pos.trDelta, &mut saberMoveDir);
                    VectorNormalize(&mut saberMoveDir);
                    if Q_irand(0, 3) == 0 {
                        //Com_Printf( "(%d) raise agg - enemy threw saber\n", level.time );
                        Jedi_Aggression(NPC, 1);
                    }
                    if DotProduct(&saberMoveDir, &saberDir2Me) > 0.5 {
                        //it's heading towards me
                        if saberDist < 100.0 {
                            //it's close
                            whichDefense = Q_irand(3, 6);
                        } else if saberDist < 200.0 {
                            //got some time, yet, try pushing
                            whichDefense = Q_irand(0, 8);
                        }
                    }
                }
                if whichDefense != 0 {
                    //already chose one
                } else if enemy_dist > 80.0 || enemy_attacking == QFALSE {
                    //he's pretty far, or not swinging, just strafe
                    if VectorCompare(enemy_movedir, &vec3_origin) != QFALSE {
                        //if he's not moving, not swinging and far enough away, no evasion necc.
                        return;
                    }
                    if Q_irand(0, 10) < (*NPCInfo).stats.aggression {
                        return;
                    }
                    whichDefense = 100;
                } else {
                    //he's getting close and swinging at me
                    let mut fwd: vec3_t = [0.0; 3];
                    //see if I'm facing him
                    AngleVectors(
                        &(*(*NPC).client).ps.viewangles,
                        Some(&mut fwd),
                        None,
                        None,
                    );
                    if DotProduct(enemy_dir, &fwd) < 0.5 {
                        //I'm not really facing him, best option is to strafe
                        whichDefense = Q_irand(5, 16);
                    } else if enemy_dist < 56.0 {
                        //he's very close, maybe we should be more inclined to block or throw
                        whichDefense = Q_irand((*NPCInfo).stats.aggression, 12);
                    } else {
                        whichDefense = Q_irand(2, 16);
                    }
                }
            }

            if (4..=12).contains(&whichDefense) {
                //would try to block
                if (*(*NPC).client).ps.saberInFlight != QFALSE {
                    //can't, saber in not in hand, so fall back to strafe/jump
                    whichDefense = 100;
                }
            }

            match whichDefense {
                0 | 1 | 2 | 3 => {
                    //use jedi force push?
                    //FIXME: try to do this if health low or enemy back to a cliff?
                    if ((*NPCInfo).rank == RANK_ENSIGN || (*NPCInfo).rank > RANK_LT_JG)
                        && TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE
                    {
                        //FIXME: check forcePushRadius[NPC->client->ps.fd.forcePowerLevel[FP_PUSH]]
                        ForceThrow(NPC, QFALSE);
                    }
                }
                4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 => {
                    //try to parry the blow
                    //Com_Printf( "blocking\n" );
                    Jedi_SaberBlock(0, 0);
                }
                _ => {
                    //Evade!
                    //start a strafe left/right if not already
                    if Q_irand(0, 5) == 0 || Jedi_Strafe(300, 1000, 0, 1000, QFALSE) == QFALSE {
                        //certain chance they will pick an alternative evasion
                        //if couldn't strafe, try a different kind of evasion...
                        if shooting_lightning != QFALSE
                            || throwing_saber != QFALSE
                            || enemy_dist < 80.0
                        {
                            //FIXME: force-jump+forward - jump over the guy!
                            if shooting_lightning != QFALSE
                                || (Q_irand(0, 2) == 0
                                    && (*NPCInfo).stats.aggression < 4
                                    && TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE)
                            {
                                if ((*NPCInfo).rank == RANK_ENSIGN || (*NPCInfo).rank > RANK_LT_JG)
                                    && shooting_lightning == QFALSE
                                    && Q_irand(0, 2) != 0
                                {
                                    //FIXME: check forcePushRadius[NPC->client->ps.fd.forcePowerLevel[FP_PUSH]]
                                    ForceThrow(NPC, QFALSE);
                                } else if ((*NPCInfo).rank == RANK_CREWMAN
                                    || (*NPCInfo).rank > RANK_LT_JG)
                                    && (*NPCInfo).scriptFlags & SCF_NO_ACROBATICS == 0
                                    && (*(*NPC).client).ps.fd.forceRageRecoveryTime
                                        < (*addr_of!(level)).time
                                    && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_RAGE) == 0
                                    && PM_InKnockDown(&mut (*(*NPC).client).ps) == QFALSE
                                {
                                    //FIXME: make this a function call?
                                    //FIXME: check for clearance, safety of landing spot?
                                    (*(*NPC).client).ps.fd.forceJumpCharge = 480.0;
                                    //Don't jump again for another 2 to 5 seconds
                                    TIMER_Set(
                                        NPC,
                                        c"jumpChaseDebounce".as_ptr(),
                                        Q_irand(2000, 5000),
                                    );
                                    if Q_irand(0, 2) != 0 {
                                        ucmd.forwardmove = 127;
                                        VectorClear(&mut (*(*NPC).client).ps.moveDir);
                                    } else {
                                        ucmd.forwardmove = -127;
                                        VectorClear(&mut (*(*NPC).client).ps.moveDir);
                                    }
                                    //FIXME: if this jump is cleared, we can't block... so pick a random lower block?
                                    if Q_irand(0, 1) != 0 {
                                        //FIXME: make intelligent
                                        (*(*NPC).client).ps.saberBlocked = BLOCKED_LOWER_RIGHT;
                                    } else {
                                        (*(*NPC).client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
                                    }
                                }
                            } else if enemy_attacking != QFALSE {
                                Jedi_SaberBlock(0, 0);
                            }
                        }
                    } else {
                        //strafed
                        if d_JediAI.integer != 0 {
                            Com_Printf("def strafe\n");
                        }
                        if (*NPCInfo).scriptFlags & SCF_NO_ACROBATICS == 0
                            && (*(*NPC).client).ps.fd.forceRageRecoveryTime < (*addr_of!(level)).time
                            && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_RAGE) == 0
                            && ((*NPCInfo).rank == RANK_CREWMAN || (*NPCInfo).rank > RANK_LT_JG)
                            && PM_InKnockDown(&mut (*(*NPC).client).ps) == QFALSE
                            && Q_irand(0, 5) == 0
                        {
                            //FIXME: make this a function call?
                            //FIXME: check for clearance, safety of landing spot?
                            if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                                (*(*NPC).client).ps.fd.forceJumpCharge = 280.0;
                            //FIXME: calc this intelligently?
                            } else {
                                (*(*NPC).client).ps.fd.forceJumpCharge = 320.0;
                            }
                            //Don't jump again for another 2 to 5 seconds
                            TIMER_Set(NPC, c"jumpChaseDebounce".as_ptr(), Q_irand(2000, 5000));
                        }
                    }
                }
            }

            //turn off slow walking no matter what
            TIMER_Set(NPC, c"walking".as_ptr(), -(*addr_of!(level)).time);
            TIMER_Set(NPC, c"taunting".as_ptr(), -(*addr_of!(level)).time);
        }
    }
}

/// `void NPC_BSJedi_FollowLeader( void )` (NPC_AI_Jedi.c:5706). Jedi behavior-state
/// dispatcher for following a leader: if the saber was knocked out of the hand and is
/// lying on the ground, runs at it to pull it back; otherwise it handles mid-jump,
/// no-LOS jumps, and blocked-path jumps to `blockedDest` before falling back to the
/// generic `NPC_BSFollowLeader`. No oracle (NPC AI: nav, jumps, entity spawn/link).
///
/// # Safety
/// `NPC`/`NPC->client`/`NPCInfo` must be valid; `NPC->enemy`/`NPCInfo->goalEntity` are
/// guarded.
pub unsafe fn NPC_BSJedi_FollowLeader() {
    (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
    if (*NPC).enemy.is_null() {
        //Com_Printf( "(%d) drop agg - no enemy (follow)\n", level.time );
        Jedi_AggressionErosion(-1);
    }

    //did we drop our saber?  If so, go after it!
    if (*(*NPC).client).ps.saberInFlight != QFALSE {
        //saber is not in hand
        if (*(*NPC).client).ps.saberEntityNum < ENTITYNUM_NONE
            && (*(*NPC).client).ps.saberEntityNum > 0
        {
            //player is 0
            //
            if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*NPC).client).ps.saberEntityNum as usize))
                .s
                .pos
                .trType
                == TR_STATIONARY
            {
                //fell to the ground, try to pick it up...
                if Jedi_CanPullBackSaber(NPC) != QFALSE {
                    //FIXME: if it's on the ground and we just pulled it back to us, should we
                    //		stand still for a bit to make sure it gets to us...?
                    //		otherwise we could end up running away from it while it's on its
                    //		way back to us and we could lose it again.
                    (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
                    (*NPCInfo).goalEntity =
                        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*NPC).client).ps.saberEntityNum as usize);
                    ucmd.buttons |= BUTTON_ATTACK;
                    if !(*NPC).enemy.is_null() && (*(*NPC).enemy).health > 0 {
                        //get our saber back NOW!
                        if NPC_MoveToGoal(QTRUE) == QFALSE {
                            //Jedi_Move( NPCInfo->goalEntity, qfalse );
                            //can't nav to it, try jumping to it
                            NPC_FaceEntity((*NPCInfo).goalEntity, QTRUE);
                            Jedi_TryJump((*NPCInfo).goalEntity);
                        }
                        NPC_UpdateAngles(QTRUE, QTRUE);
                        return;
                    }
                }
            }
        }
    }

    if !(*NPCInfo).goalEntity.is_null() {
        let mut trace: trace_t = trace_t::default();

        if Jedi_Jumping((*NPCInfo).goalEntity) != QFALSE {
            //in mid-jump
            return;
        }

        if NAV_CheckAhead(
            NPC,
            &(*(*NPCInfo).goalEntity).r.currentOrigin,
            &mut trace,
            ((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
        ) == QFALSE
        {
            //can't get straight to him
            if NPC_ClearLOS4((*NPCInfo).goalEntity) != QFALSE
                && NPC_FaceEntity((*NPCInfo).goalEntity, QTRUE) != QFALSE
            {
                //no line of sight
                if Jedi_TryJump((*NPCInfo).goalEntity) != QFALSE {
                    //started a jump
                    return;
                }
            }
        }
        if (*NPCInfo).aiFlags & NPCAI_BLOCKED != 0 {
            //try to jump to the blockedDest
            if ((*NPCInfo).blockedDest[2] - (*NPC).r.currentOrigin[2]).abs() > 64.0 {
                let tempGoal: *mut gentity_t = G_Spawn(); //ugh, this is NOT good...?
                G_SetOrigin(tempGoal, &(*NPCInfo).blockedDest);
                trap::LinkEntity(tempGoal);
                TIMER_Set(NPC, c"jumpChaseDebounce".as_ptr(), -1);
                if Jedi_TryJump(tempGoal) != QFALSE {
                    //going to jump to the dest
                    G_FreeEntity(tempGoal);
                    return;
                }
                G_FreeEntity(tempGoal);
            }
        }
    }
    //try normal movement
    NPC_BSFollowLeader();
}

/// `static void Jedi_Combat( void )` (NPC_AI_Jedi.c:5103). The main per-frame Jedi
/// combat tick: predicts the enemy's position 300ms out, tries to jump/hunt to him if
/// the path is blocked, updates combat timers + distance management, faces the enemy,
/// runs saber evasion, applies strafe/walk timers, decides whether to attack or idle,
/// fires (Boba), and guards against strafing off ledges. No oracle (NPC AI: nav,
/// timers, RNG, force powers).
///
/// # Safety
/// `NPC`/`NPC->client`/`NPCInfo`/`NPC->enemy` must be valid for the combat frame.
#[allow(dead_code)] // called by Jedi_Attack as it lands
unsafe fn Jedi_Combat() {
    let mut enemy_dir: vec3_t = [0.0; 3];
    let mut enemy_movedir: vec3_t = [0.0; 3];
    let mut enemy_dest: vec3_t = [0.0; 3];
    let mut enemy_dist: f32 = 0.0;
    let mut enemy_movespeed: f32 = 0.0;
    #[allow(unused_assignments)]
    let mut enemy_lost: qboolean = QFALSE;

    //See where enemy will be 300 ms from now
    Jedi_SetEnemyInfo(
        &mut enemy_dest,
        &mut enemy_dir,
        &mut enemy_dist,
        &mut enemy_movedir,
        &mut enemy_movespeed,
        300,
    );

    if Jedi_Jumping((*NPC).enemy) != QFALSE {
        //I'm in the middle of a jump, so just see if I should attack
        Jedi_AttackDecide(enemy_dist as c_int);
        return;
    }

    if (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_GRIP) == 0
        || (*(*NPC).client).ps.fd.forcePowerLevel[FP_GRIP as usize] < FORCE_LEVEL_2
    {
        //not gripping
        //If we can't get straight at him
        if Jedi_ClearPathToSpot(&enemy_dest, (*(*NPC).enemy).s.number) == QFALSE {
            //hunt him down
            //Com_Printf( "No Clear Path\n" );
            if (NPC_ClearLOS4((*NPC).enemy) != QFALSE
                || (*NPCInfo).enemyLastSeenTime > (*addr_of!(level)).time - 500)
                && NPC_FaceEnemy(QTRUE) != QFALSE
            {
                //( NPCInfo->rank == RANK_CREWMAN || NPCInfo->rank > RANK_LT_JG ) &&
                //try to jump to him?
                /*
                vec3_t end;
                VectorCopy( NPC->r.currentOrigin, end );
                end[2] += 36;
                trap_Trace( &trace, NPC->r.currentOrigin, NPC->r.mins, NPC->r.maxs, end, NPC->s.number, NPC->clipmask|CONTENTS_BOTCLIP );
                if ( !trace.allsolid && !trace.startsolid && trace.fraction >= 1.0 )
                {
                    vec3_t angles, forward;
                    VectorCopy( NPC->client->ps.viewangles, angles );
                    angles[0] = 0;
                    AngleVectors( angles, forward, NULL, NULL );
                    VectorMA( end, 64, forward, end );
                    trap_Trace( &trace, NPC->r.currentOrigin, NPC->r.mins, NPC->r.maxs, end, NPC->s.number, NPC->clipmask|CONTENTS_BOTCLIP );
                    if ( !trace.allsolid && !trace.startsolid )
                    {
                        if ( trace.fraction >= 1.0 || trace.plane.normal[2] > 0 )
                        {
                            ucmd.upmove = 127;
                            ucmd.forwardmove = 127;
                            return;
                        }
                    }
                }
                */
                //FIXME: about every 1 second calc a velocity,
                //run a loop of traces with evaluate trajectory
                //for gravity with my size, see if it makes it...
                //this will also catch misacalculations that send you off ledges!
                //Com_Printf( "Considering Jump\n" );
                if Jedi_TryJump((*NPC).enemy) != QFALSE {
                    //FIXME: what about jumping to his enemyLastSeenLocation?
                    return;
                }
            }

            //Check for evasion
            if TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE {
                //finished parrying
                if (*(*NPC).client).ps.saberBlocked != BLOCKED_ATK_BOUNCE
                    && (*(*NPC).client).ps.saberBlocked != BLOCKED_PARRY_BROKEN
                {
                    //wasn't blocked myself
                    (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
                }
            }
            if Jedi_Hunt() != QFALSE && (*NPCInfo).aiFlags & NPCAI_BLOCKED == 0 {
                //FIXME: have to do this because they can ping-pong forever
                //can macro-navigate to him
                if enemy_dist < 384.0
                    && Q_irand(0, 10) == 0
                    && (*NPCInfo).blockedSpeechDebounceTime < (*addr_of!(level)).time
                    && jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize]
                        < (*addr_of!(level)).time
                    && NPC_ClearLOS4((*NPC).enemy) == QFALSE
                {
                    G_AddVoiceEvent(NPC, Q_irand(EV_JLOST1, EV_JLOST3), 3000);
                    jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize] =
                        (*addr_of!(level)).time + 3000;
                    (*NPCInfo).blockedSpeechDebounceTime = (*addr_of!(level)).time + 3000;
                }

                return;
            }
            //well, try to head for his last seen location
            /*
            else if ( Jedi_Track() )
            {
                return;
            }
            */
            else {
                //FIXME: try to find a waypoint that can see enemy, jump from there
                if (*NPCInfo).aiFlags & NPCAI_BLOCKED != 0 {
                    //try to jump to the blockedDest
                    let tempGoal: *mut gentity_t = G_Spawn(); //ugh, this is NOT good...?
                    G_SetOrigin(tempGoal, &(*NPCInfo).blockedDest);
                    trap::LinkEntity(tempGoal);
                    if Jedi_TryJump(tempGoal) != QFALSE {
                        //going to jump to the dest
                        G_FreeEntity(tempGoal);
                        return;
                    }
                    G_FreeEntity(tempGoal);
                }

                enemy_lost = QTRUE;
            }
        }
    }
    //else, we can see him or we can't track him at all

    //every few seconds, decide if we should we advance or retreat?
    Jedi_CombatTimersUpdate(enemy_dist as c_int);

    //We call this even if lost enemy to keep him moving and to update the taunting behavior
    //maintain a distance from enemy appropriate for our aggression level
    Jedi_CombatDistance(enemy_dist as c_int);

    //if ( !enemy_lost )
    {
        //Update our seen enemy position
        if (*(*NPC).enemy).client.is_null()
            || ((*(*(*NPC).enemy).client).ps.groundEntityNum != ENTITYNUM_NONE
                && (*(*NPC).client).ps.groundEntityNum != ENTITYNUM_NONE)
        {
            VectorCopy(
                &(*(*NPC).enemy).r.currentOrigin,
                &mut (*NPCInfo).enemyLastSeenLocation,
            );
        }
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
    }

    //Turn to face the enemy
    if TIMER_Done(NPC, c"noturn".as_ptr()) != QFALSE {
        Jedi_FaceEnemy(QTRUE);
    }
    NPC_UpdateAngles(QTRUE, QTRUE);

    //Check for evasion
    if TIMER_Done(NPC, c"parryTime".as_ptr()) != QFALSE {
        //finished parrying
        if (*(*NPC).client).ps.saberBlocked != BLOCKED_ATK_BOUNCE
            && (*(*NPC).client).ps.saberBlocked != BLOCKED_PARRY_BROKEN
        {
            //wasn't blocked myself
            (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
        }
    }
    if (*(*NPC).enemy).s.weapon == WP_SABER {
        Jedi_EvasionSaber(&enemy_movedir, enemy_dist, &enemy_dir);
    } else {
        //do we need to do any evasion for other kinds of enemies?
    }

    //apply strafing/walking timers, etc.
    Jedi_TimersApply();

    if (*(*NPC).client).ps.saberInFlight == QFALSE
        && ((*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_GRIP) == 0
            || (*(*NPC).client).ps.fd.forcePowerLevel[FP_GRIP as usize] < FORCE_LEVEL_2)
    {
        //not throwing saber or using force grip
        //see if we can attack
        if Jedi_AttackDecide(enemy_dist as c_int) == QFALSE {
            //we're not attacking, decide what else to do
            Jedi_CombatIdle(enemy_dist as c_int);
            //FIXME: lower aggression when actually strike offensively?  Or just when do damage?
        } else {
            //we are attacking
            //stop taunting
            TIMER_Set(NPC, c"taunting".as_ptr(), -(*addr_of!(level)).time);
        }
    } else {
    }
    if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
        Boba_FireDecide();
    }

    //Check for certain enemy special moves
    Jedi_CheckEnemyMovement(enemy_dist);
    //Make sure that we don't jump off ledges over long drops
    Jedi_CheckJumps();
    //Just make sure we don't strafe into walls or off cliffs
    if NPC_MoveDirClear(ucmd.forwardmove as c_int, ucmd.rightmove as c_int, QTRUE) == QFALSE {
        //uh-oh, we are going to fall or hit something
        let mut info: navInfo_t = core::mem::zeroed();
        //Get the move info
        NAV_GetLastMove(&mut info);
        if info.flags & NIF_MACRO_NAV == 0 {
            //micro-navigation told us to step off a ledge, try using macronav for now
            NPC_MoveToGoal(QFALSE);
        }
        //reset the timers.
        TIMER_Set(NPC, c"strafeLeft".as_ptr(), 0);
        TIMER_Set(NPC, c"strafeRight".as_ptr(), 0);
    }
    let _ = enemy_lost;
    let _ = enemy_movespeed;
}

/// `static void Jedi_Attack( void )` (NPC_AI_Jedi.c:5793). The top-level Jedi attack
/// behavior: bails during pain anims, mashes attack to win saber locks, retrieves a
/// dropped saber, gloats/heals after killing its enemy, re-validates the enemy, then
/// runs `Jedi_Combat` and post-processes the resulting ucmd (clears air movement,
/// ducks, suppresses attacks while broken-parried / not allowed / saber in water, kicks
/// in force speed). No oracle (NPC AI: RNG, timers, force powers, anims).
///
/// # Safety
/// `NPC`/`NPC->client`/`NPCInfo` must be valid; `NPC->enemy` is checked/guarded.
// faithful: C's `g_spskill` switch falls through (case 0 → 1 → 2), so each `chance`
// assignment before the last is dead — mirror it and allow the unused-assignment lint.
#[allow(unused_assignments)]
pub unsafe fn Jedi_Attack() {
    //Don't do anything if we're in a pain anim
    if (*NPC).painDebounceTime > (*addr_of!(level)).time {
        if Q_irand(0, 1) != 0 {
            Jedi_FaceEnemy(QTRUE);
        }
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    if (*(*NPC).client).ps.saberLockTime > (*addr_of!(level)).time {
        //FIXME: maybe if I'm losing I should try to force-push out of it?  Very rarely, though...
        if (*(*NPC).client).ps.fd.forcePowerLevel[FP_PUSH as usize] > FORCE_LEVEL_2
            && (*(*NPC).client).ps.saberLockTime < (*addr_of!(level)).time + 5000
            && Q_irand(0, 10) == 0
        {
            ForceThrow(NPC, QFALSE);
        }
        //based on my skill, hit attack button every other to every several frames in order to push enemy back
        else {
            let mut chance: f32;

            if (*(*NPC).client).NPC_class == CLASS_DESANN
                || Q_stricmp(c"Yoda".as_ptr(), (*NPC).NPC_type) == 0
            {
                if g_spskill.integer != 0 {
                    chance = 4.0; //he pushes *hard*
                } else {
                    chance = 3.0; //he pushes *hard*
                }
            } else if (*(*NPC).client).NPC_class == CLASS_TAVION {
                chance = 2.0 + g_spskill.value; //from 2 to 4
            } else {
                //the escalation in difficulty is nice, here, but cap it so it doesn't get *impossible* on hard
                let maxChance: f32 = RANK_LT as f32 / 2.0 + 3.0; //5?
                if g_spskill.value == 0.0 {
                    chance = (*NPCInfo).rank as f32 / 2.0;
                } else {
                    chance = (*NPCInfo).rank as f32 / 2.0 + 1.0;
                }
                if chance > maxChance {
                    chance = maxChance;
                }
            }
            //	if ( flrand( -4.0f, chance ) >= 0.0f && !(NPC->client->ps.pm_flags&PMF_ATTACK_HELD) )
            //	{
            //		ucmd.buttons |= BUTTON_ATTACK;
            //	}
            if flrand(-4.0, chance) >= 0.0 {
                ucmd.buttons |= BUTTON_ATTACK;
            }
            //rwwFIXMEFIXME: support for PMF_ATTACK_HELD
        }
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }
    //did we drop our saber?  If so, go after it!
    if (*(*NPC).client).ps.saberInFlight != QFALSE {
        //saber is not in hand
        //	if ( NPC->client->ps.saberEntityNum < ENTITYNUM_NONE && NPC->client->ps.saberEntityNum > 0 )//player is 0
        if (*(*NPC).client).ps.saberEntityNum == 0 && (*(*NPC).client).saberStoredIndex != 0 {
            //this is valid, it's 0 when our saber is gone -rww (mp-specific)
            //
            //if ( g_entities[NPC->client->ps.saberEntityNum].s.pos.trType == TR_STATIONARY )
            if true {
                //no matter
                //fell to the ground, try to pick it up
                //	if ( Jedi_CanPullBackSaber( NPC ) )
                if true {
                    //no matter
                    (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
                    (*NPCInfo).goalEntity = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*NPC).client).saberStoredIndex as usize);
                    ucmd.buttons |= BUTTON_ATTACK;
                    if !(*NPC).enemy.is_null() && (*(*NPC).enemy).health > 0 {
                        //get our saber back NOW!
                        Jedi_Move((*NPCInfo).goalEntity, QFALSE);
                        NPC_UpdateAngles(QTRUE, QTRUE);
                        if (*(*NPC).enemy).s.weapon == WP_SABER {
                            //be sure to continue evasion
                            let mut enemy_dir: vec3_t = [0.0; 3];
                            let mut enemy_movedir: vec3_t = [0.0; 3];
                            let mut enemy_dest: vec3_t = [0.0; 3];
                            let mut enemy_dist: f32 = 0.0;
                            let mut enemy_movespeed: f32 = 0.0;
                            Jedi_SetEnemyInfo(
                                &mut enemy_dest,
                                &mut enemy_dir,
                                &mut enemy_dist,
                                &mut enemy_movedir,
                                &mut enemy_movespeed,
                                300,
                            );
                            Jedi_EvasionSaber(&enemy_movedir, enemy_dist, &enemy_dir);
                        }
                        return;
                    }
                }
            }
        }
    }
    //see if our enemy was killed by us, gloat and turn off saber after cool down.
    //FIXME: don't do this if we have other enemies to fight...?
    if !(*NPC).enemy.is_null() {
        if (*(*NPC).enemy).health <= 0
            && (*(*NPC).enemy).enemy == NPC
            && (*(*NPC).client).playerTeam != NPCTEAM_PLAYER
        {
            //good guys don't gloat
            //my enemy is dead and I killed him
            (*NPCInfo).enemyCheckDebounceTime = 0; //keep looking for others

            if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                if (*NPCInfo).walkDebounceTime < (*addr_of!(level)).time
                    && (*NPCInfo).walkDebounceTime >= 0
                {
                    TIMER_Set(NPC, c"gloatTime".as_ptr(), 10000);
                    (*NPCInfo).walkDebounceTime = -1;
                }
                if TIMER_Done(NPC, c"gloatTime".as_ptr()) == QFALSE {
                    if DistanceHorizontalSquared(
                        &(*(*NPC).client).renderInfo.eyePoint,
                        &(*(*NPC).enemy).r.currentOrigin,
                    ) > 4096.0
                        && (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0
                    {
                        //64 squared
                        (*NPCInfo).goalEntity = (*NPC).enemy;
                        Jedi_Move((*NPC).enemy, QFALSE);
                        ucmd.buttons |= BUTTON_WALKING;
                    } else {
                        TIMER_Set(NPC, c"gloatTime".as_ptr(), 0);
                    }
                } else if (*NPCInfo).walkDebounceTime == -1 {
                    (*NPCInfo).walkDebounceTime = -2;
                    G_AddVoiceEvent(NPC, Q_irand(EV_VICTORY1, EV_VICTORY3), 3000);
                    jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize] =
                        (*addr_of!(level)).time + 3000;
                    (*NPCInfo).desiredPitch = 0.0;
                    (*NPCInfo).goalEntity = null_mut();
                }
                Jedi_FaceEnemy(QTRUE);
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            } else {
                if TIMER_Done(NPC, c"parryTime".as_ptr()) == QFALSE {
                    TIMER_Set(NPC, c"parryTime".as_ptr(), -1);
                    (*(*NPC).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                        (*addr_of!(level)).time + 500;
                }
                (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
                if (*(*NPC).client).ps.saberHolstered == 0
                    && (*(*NPC).client).ps.saberInFlight != QFALSE
                {
                    //saber is still on (or we're trying to pull it back), count down erosion and keep facing the enemy
                    //FIXME: need to stop this from happening over and over again when they're blocking their victim's saber
                    //FIXME: turn off saber sooner so we get cool walk anim?
                    //Com_Printf( "(%d) drop agg - enemy dead\n", level.time );
                    Jedi_AggressionErosion(-3);
                    if BG_SabersOff(&mut (*(*NPC).client).ps) != QFALSE
                        && (*(*NPC).client).ps.saberInFlight == QFALSE
                    {
                        //turned off saber (in hand), gloat
                        G_AddVoiceEvent(NPC, Q_irand(EV_VICTORY1, EV_VICTORY3), 3000);
                        jediSpeechDebounceTime[(*(*NPC).client).playerTeam as usize] =
                            (*addr_of!(level)).time + 3000;
                        (*NPCInfo).desiredPitch = 0.0;
                        (*NPCInfo).goalEntity = null_mut();
                    }
                    TIMER_Set(NPC, c"gloatTime".as_ptr(), 10000);
                }
                if (*(*NPC).client).ps.saberHolstered == 0
                    || (*(*NPC).client).ps.saberInFlight != QFALSE
                    || TIMER_Done(NPC, c"gloatTime".as_ptr()) == QFALSE
                {
                    //keep walking
                    if DistanceHorizontalSquared(
                        &(*(*NPC).client).renderInfo.eyePoint,
                        &(*(*NPC).enemy).r.currentOrigin,
                    ) > 4096.0
                        && (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0
                    {
                        //64 squared
                        (*NPCInfo).goalEntity = (*NPC).enemy;
                        Jedi_Move((*NPC).enemy, QFALSE);
                        ucmd.buttons |= BUTTON_WALKING;
                    } else {
                        //got there
                        if (*NPC).health < (*(*NPC).client).pers.maxHealth
                            && (*(*NPC).client).ps.fd.forcePowersKnown & (1 << FP_HEAL) != 0
                            && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_HEAL) == 0
                        {
                            ForceHeal(NPC);
                        }
                    }
                    Jedi_FaceEnemy(QTRUE);
                    NPC_UpdateAngles(QTRUE, QTRUE);
                    return;
                }
            }
        }
    }

    //If we don't have an enemy, just idle
    if (*(*NPC).enemy).s.weapon == WP_TURRET
        && Q_stricmp(c"PAS".as_ptr(), (*(*NPC).enemy).classname) == 0
    {
        if (*(*NPC).enemy).count <= 0 {
            //it's out of ammo
            if !(*(*NPC).enemy).activator.is_null()
                && NPC_ValidEnemy((*(*NPC).enemy).activator) != QFALSE
            {
                let turretOwner: *mut gentity_t = (*(*NPC).enemy).activator;
                G_ClearEnemy(NPC);
                G_SetEnemy(NPC, turretOwner);
            } else {
                G_ClearEnemy(NPC);
            }
        }
    }
    NPC_CheckEnemy(QTRUE, QTRUE, QTRUE);

    if (*NPC).enemy.is_null() {
        (*(*NPC).client).ps.saberBlocked = BLOCKED_NONE;
        if (*NPCInfo).tempBehavior == BS_HUNT_AND_KILL {
            //lost him, go back to what we were doing before
            (*NPCInfo).tempBehavior = BS_DEFAULT;
            NPC_UpdateAngles(QTRUE, QTRUE);
            return;
        }
        Jedi_Patrol(); //was calling Idle... why?
        return;
    }

    //always face enemy if have one
    (*NPCInfo).combatMove = QTRUE;

    //Track the player and kill them if possible
    Jedi_Combat();

    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0
        || ((*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_HEAL) != 0
            && (*(*NPC).client).ps.fd.forcePowerLevel[FP_HEAL as usize] < FORCE_LEVEL_2)
    {
        //this is really stupid, but okay...
        ucmd.forwardmove = 0;
        ucmd.rightmove = 0;
        if ucmd.upmove > 0 {
            ucmd.upmove = 0;
        }
        (*(*NPC).client).ps.fd.forceJumpCharge = 0.0;
        VectorClear(&mut (*(*NPC).client).ps.moveDir);
    }

    //NOTE: for now, we clear ucmd.forwardmove & ucmd.rightmove while in air to avoid jumps going awry...
    if (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE {
        //don't push while in air, throws off jumps!
        //FIXME: if we are in the air over a drop near a ledge, should we try to push back towards the ledge?
        ucmd.forwardmove = 0;
        ucmd.rightmove = 0;
        VectorClear(&mut (*(*NPC).client).ps.moveDir);
    }

    if TIMER_Done(NPC, c"duck".as_ptr()) == QFALSE {
        ucmd.upmove = -127;
    }

    if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
        if PM_SaberInBrokenParry((*(*NPC).client).ps.saberMove) != QFALSE
            || (*(*NPC).client).ps.saberBlocked == BLOCKED_PARRY_BROKEN
        {
            //just make sure they don't pull their saber to them if they're being blocked
            ucmd.buttons &= !BUTTON_ATTACK;
        }
    }

    if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 //not allowed to attack
        || ((*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_HEAL) != 0
            && (*(*NPC).client).ps.fd.forcePowerLevel[FP_HEAL as usize] < FORCE_LEVEL_3)
        || ((*(*NPC).client).ps.saberEventFlags & SEF_INWATER != 0
            && (*(*NPC).client).ps.saberInFlight == QFALSE)
    {
        //saber in water
        ucmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
    }

    if (*NPCInfo).scriptFlags & SCF_NO_ACROBATICS != 0 {
        ucmd.upmove = 0;
        (*(*NPC).client).ps.fd.forceJumpCharge = 0.0;
    }

    if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
        Jedi_CheckDecreaseSaberAnimLevel();
    }

    if ucmd.buttons & BUTTON_ATTACK != 0 && (*(*NPC).client).playerTeam == NPCTEAM_ENEMY {
        if Q_irand(0, (*(*NPC).client).ps.fd.saberAnimLevel) > 0
            && Q_irand(0, (*(*NPC).client).pers.maxHealth + 10) > (*NPC).health
            && Q_irand(0, 3) == 0
        {
            //the more we're hurt and the stronger the attack we're using, the more likely we are to make a anger noise when we swing
            G_AddVoiceEvent(NPC, Q_irand(EV_COMBAT1, EV_COMBAT3), 1000);
        }
    }

    if (*(*NPC).client).NPC_class != CLASS_BOBAFETT {
        if (*(*NPC).client).NPC_class == CLASS_TAVION
            || (g_spskill.integer != 0
                && ((*(*NPC).client).NPC_class == CLASS_DESANN
                    || (*NPCInfo).rank >= Q_irand(RANK_CREWMAN, RANK_CAPTAIN)))
        {
            //Tavion will kick in force speed if the player does...
            if !(*NPC).enemy.is_null()
                && (*(*NPC).enemy).s.number == 0
                && !(*(*NPC).enemy).client.is_null()
                && (*(*(*NPC).enemy).client).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0
                && (*(*NPC).client).ps.fd.forcePowersActive & (1 << FP_SPEED) == 0
            {
                let mut chance: c_int = 0;
                #[allow(clippy::match_overlapping_arm)]
                match g_spskill.integer {
                    0 => {
                        chance = 9;
                        chance = 3;
                        chance = 1;
                    }
                    1 => {
                        chance = 3;
                        chance = 1;
                    }
                    2 => {
                        chance = 1;
                    }
                    _ => {}
                }
                if Q_irand(0, chance) == 0 {
                    ForceSpeed(NPC, 0);
                }
            }
        }
    }
}

/// `void NPC_BSJedi_Default( void )` (NPC_AI_Jedi.c:6118). The top-level Jedi
/// behavior-state entry point: with no enemy it patrols (Boba uses the stormtrooper
/// patrol); with an enemy it handles ambush drop-down, the Boba sniper-switch at long
/// range, then `Jedi_Attack`, and periodically re-checks for a better/closer enemy.
/// No oracle (NPC AI: RNG, timers, force powers).
///
/// # Safety
/// `NPC`/`NPC->client`/`NPCInfo` must be valid; `NPC->enemy` is guarded.
pub unsafe fn NPC_BSJedi_Default() {
    Jedi_CheckCloak();
    if (*NPC).enemy.is_null() {
        //don't have an enemy, look for one
        if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
            NPC_BSST_Patrol();
        } else {
            Jedi_Patrol();
        }
    } else
    //if ( NPC->enemy )
    {
        //have an enemy
        if Jedi_WaitingAmbush(NPC) != QFALSE {
            //we were still waiting to drop down - must have had enemy set on me outside my AI
            Jedi_Ambush(NPC);
        }
        if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
            if (*(*NPC).enemy).enemy != NPC
                && (*NPC).health == (*(*NPC).client).pers.maxHealth
                && DistanceSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin)
                    > (800 * 800) as f32
            {
                (*NPCInfo).scriptFlags |= SCF_ALT_FIRE;
                Boba_ChangeWeapon(WP_DISRUPTOR);
                NPC_BSSniper_Default();
                return;
            }
        }
        Jedi_Attack();
        //if we have multiple-jedi combat, probably need to keep checking (at certain debounce intervals) for a better (closer, more active) enemy and switch if needbe...
        if ((ucmd.buttons == 0 && (*(*NPC).client).ps.fd.forcePowersActive == 0)
            || (!(*NPC).enemy.is_null() && (*(*NPC).enemy).health <= 0))
            && (*NPCInfo).enemyCheckDebounceTime < (*addr_of!(level)).time
        {
            //not doing anything (or walking toward a vanquished enemy - fixme: always taunt the player?), not using force powers and it's time to look again
            //FIXME: build a list of all local enemies (since we have to find best anyway) for other AI factors- like when to use group attacks, determine when to change tactics, when surrounded, when blocked by another in the enemy group, etc.  Should we build this group list or let the enemies maintain their own list and we just access it?
            let sav_enemy: *mut gentity_t = (*NPC).enemy; //FIXME: what about NPC->lastEnemy?
            let newEnemy: *mut gentity_t;

            (*NPC).enemy = null_mut();
            newEnemy = NPC_CheckEnemy(
                ((*NPCInfo).confusionTime < (*addr_of!(level)).time) as qboolean,
                QFALSE,
                QFALSE,
            );
            (*NPC).enemy = sav_enemy;
            if !newEnemy.is_null() && newEnemy != sav_enemy {
                //picked up a new enemy!
                (*NPC).lastEnemy = (*NPC).enemy;
                G_SetEnemy(NPC, newEnemy);
            }
            (*NPCInfo).enemyCheckDebounceTime = (*addr_of!(level)).time + Q_irand(1000, 3000);
        }
    }
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::codemp::game::g_local::{gclient_t, gentity_t};
    use crate::codemp::game::b_public_h::{gNPC_t, RANK_ENSIGN, RANK_LT};
    use crate::codemp::game::teams_h::{CLASS_DESANN, CLASS_REBORN};
    use crate::codemp::game::w_saber_h::{EVASION_NONE, evasionType_t};
    use core::ptr::addr_of_mut;

    extern "C" {
        // NPC_AI_Jedi.c:2253 — scalar-marshaled (see npc_ai_jedi_oracle.c).
        fn jka_jedi_recalcparrytime(
            has_client: i32,
            number: i32,
            saberDefenseLevel: i32,
            npc_present: i32,
            npc_class: i32,
            npc_rank: i32,
            torsoTimer: i32,
            saberInFlight: i32,
            evasionType: i32,
            realCombat: i32,
            spskill: i32,
        ) -> i32;
    }

    #[test]
    fn jedi_recalcparrytime_matches_oracle() {
        // Q_irand is a PC wrapper over irand (the holdrand MSVC LCG), a
        // process-global seed — take the lock and re-seed both sides per call.
        let _guard = crate::codemp::game::bg_lib::rand_lock();

        // (has_client, number, saberDefenseLevel) — number==0 is the player branch.
        let players: &[(bool, i32, i32)] = &[
            (false, 1, 0), // no client -> 0
            (true, 0, 0),  // player, defense 0
            (true, 0, 1),  // player, defense 1
            (true, 0, 2),  // player, defense 2
            (true, 0, 3),  // player, defense 3
        ];
        // NPC branch params.
        let classes = [CLASS_TAVION, CLASS_DESANN, CLASS_REBORN, CLASS_JEDI];
        let ranks = [RANK_CIVILIAN, RANK_CREWMAN, RANK_LT_JG, RANK_LT, RANK_ENSIGN];
        let evasions: [evasionType_t; 10] = [
            EVASION_NONE,
            EVASION_PARRY,
            EVASION_DUCK_PARRY,
            EVASION_JUMP_PARRY,
            EVASION_DODGE,
            EVASION_JUMP,
            EVASION_DUCK,
            EVASION_FJUMP,
            EVASION_CARTWHEEL,
            EVASION_OTHER,
        ];

        let check = |has_client: bool,
                         number: i32,
                         saberDefenseLevel: i32,
                         npc_present: bool,
                         npc_class: c_int,
                         npc_rank: c_int,
                         torso_timer: i32,
                         saber_in_flight: bool,
                         evasion: evasionType_t,
                         real_combat: i32,
                         spskill: i32| {
            let mut client: gclient_t = unsafe { core::mem::zeroed() };
            client.NPC_class = npc_class;
            client.ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] = saberDefenseLevel;
            client.ps.torsoTimer = torso_timer;
            client.ps.saberInFlight = if saber_in_flight { QTRUE } else { QFALSE };

            let mut npc: gNPC_t = unsafe { core::mem::zeroed() };
            npc.rank = npc_rank;

            let mut ent: gentity_t = unsafe { core::mem::zeroed() };
            ent.s.number = number;
            if has_client {
                ent.client = &mut client;
            }
            if npc_present {
                ent.NPC = &mut npc;
            }

            unsafe {
                (*addr_of_mut!(g_saberRealisticCombat)).integer = real_combat;
                (*addr_of_mut!(g_spskill)).integer = spskill;
            }

            // Re-seed both sides so the consumed rand() stream matches.
            let seed = (npc_class as u32)
                .wrapping_mul(31)
                .wrapping_add(npc_rank as u32)
                .wrapping_mul(31)
                .wrapping_add(evasion as u32)
                .wrapping_add((spskill as u32) << 8)
                .wrapping_add(1);

            crate::codemp::game::q_math::Rand_Init(seed as c_int);
            let rust = unsafe { Jedi_ReCalcParryTime(&mut ent, evasion) };
            unsafe { oracle_seed(seed) };
            let c = unsafe {
                jka_jedi_recalcparrytime(
                    has_client as i32,
                    number,
                    saberDefenseLevel,
                    npc_present as i32,
                    npc_class,
                    npc_rank,
                    torso_timer,
                    saber_in_flight as i32,
                    evasion,
                    real_combat,
                    spskill,
                )
            };
            assert_eq!(
                rust, c,
                "Jedi_ReCalcParryTime client={has_client} num={number} def={saberDefenseLevel} npc={npc_present} class={npc_class} rank={npc_rank} torso={torso_timer} inflight={saber_in_flight} ev={evasion} rc={real_combat} sk={spskill}"
            );
        };

        // Player / no-client branches (no Q_irand consumption).
        for &(has_client, number, def) in players {
            for &spskill in &[0i32, 1, 2, 3] {
                check(has_client, number, def, false, CLASS_JEDI, RANK_LT_JG, 250, false, EVASION_PARRY, 0, spskill);
            }
        }

        // NPC branch: full cross of class x rank x evasion x realCombat x spskill,
        // plus saber-in-flight on/off and a couple torsoTimer values.
        for &npc_class in &classes {
            for &npc_rank in &ranks {
                for &evasion in &evasions {
                    for &real_combat in &[0i32, 1] {
                        for &spskill in &[0i32, 1, 2, 3] {
                            for &saber_in_flight in &[false, true] {
                                check(
                                    true, 1, 2, true, npc_class, npc_rank, 250,
                                    saber_in_flight, evasion, real_combat, spskill,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Re-seed the oracle's holdrand LCG (Rand_Init in q_math_oracle.c) to match
    // q_math::Rand_Init, since Q_irand now forwards to irand (holdrand).
    unsafe fn oracle_seed(seed: u32) {
        crate::oracle::Rand_Init(seed as core::ffi::c_int);
    }
}

