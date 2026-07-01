//! Port of `w_saber.c` — the saber-combat subsystem. Landed incrementally: only the
//! helpers that already-ported callers reach. `UpdateClientRenderBolts` lands first as
//! a forward dependency of `g_combat.c`'s `G_GetHitLocFromSurfName` (the torso
//! hit-location branch refreshes the client's bolt points through it). `RandFloat` is the
//! file's tiny random-float helper, pulled forward as a dependency of `g_missile.c`'s
//! `G_ReflectMissile`/`G_DeflectMissile` (which jitter the bounce direction with it).

#![allow(non_snake_case)] // C function names kept verbatim
#![allow(non_upper_case_globals)] // module-static accumulators keep their C names verbatim

use core::ffi::{c_char, c_int, c_void, CStr};
use core::ptr::{addr_of, addr_of_mut, null, null_mut};

use crate::codemp::cgame::animtable::animTable;
use crate::codemp::game::ai_wpnav::G_TestLine;
use crate::codemp::game::anims::{
    animNumber_t, BOTH_A1_BL_TR, BOTH_A1_BR_TL, BOTH_A1_SPECIAL, BOTH_A1_TL_BR, BOTH_A1_TR_BL,
    BOTH_A1_T__B_, BOTH_A1__L__R, BOTH_A1__R__L, BOTH_A2_BL_TR, BOTH_A2_BR_TL, BOTH_A2_SPECIAL,
    BOTH_A2_STABBACK1, BOTH_A2_TL_BR, BOTH_A2_TR_BL, BOTH_A2_T__B_, BOTH_A2__L__R, BOTH_A2__R__L,
    BOTH_A3_BL_TR, BOTH_A3_BR_TL, BOTH_A3_SPECIAL, BOTH_A3_TL_BR, BOTH_A3_TR_BL, BOTH_A3_T__B_,
    BOTH_A3__L__R, BOTH_A3__R__L, BOTH_A4_BL_TR, BOTH_A4_BR_TL, BOTH_A4_TL_BR, BOTH_A4_TR_BL,
    BOTH_A4_T__B_, BOTH_A4__L__R, BOTH_A4__R__L, BOTH_A5_BL_TR, BOTH_A5_BR_TL, BOTH_A5_TL_BR,
    BOTH_A5_TR_BL, BOTH_A5_T__B_, BOTH_A5__L__R, BOTH_A5__R__L, BOTH_A6_BL_TR, BOTH_A6_BR_TL,
    BOTH_A6_FB, BOTH_A6_LR, BOTH_A6_SABERPROTECT, BOTH_A6_TL_BR, BOTH_A6_TR_BL, BOTH_A6_T__B_,
    BOTH_A6__L__R, BOTH_A6__R__L, BOTH_A7_BL_TR, BOTH_A7_BR_TL, BOTH_A7_HILT, BOTH_A7_KICK_B,
    BOTH_A7_KICK_BF, BOTH_A7_KICK_B_AIR, BOTH_A7_KICK_F, BOTH_A7_KICK_F_AIR, BOTH_A7_KICK_L,
    BOTH_A7_KICK_L_AIR, BOTH_A7_KICK_R, BOTH_A7_KICK_RL, BOTH_A7_KICK_R_AIR, BOTH_A7_KICK_S,
    BOTH_A7_SOULCAL, BOTH_A7_TL_BR, BOTH_A7_TR_BL, BOTH_A7_T__B_, BOTH_A7__L__R, BOTH_A7__R__L,
    BOTH_ALORA_SPIN_SLASH, BOTH_ATTACK_BACK, BOTH_BF1LOCK, BOTH_BF2LOCK, BOTH_BUTTERFLY_FL1,
    BOTH_BUTTERFLY_FR1, BOTH_BUTTERFLY_LEFT, BOTH_BUTTERFLY_RIGHT, BOTH_CCWCIRCLELOCK,
    BOTH_CROUCHATTACKBACK1, BOTH_CWCIRCLELOCK, BOTH_D1_B____, BOTH_D2_B____, BOTH_D3_B____,
    BOTH_D4_B____, BOTH_D5_B____, BOTH_D6_B____, BOTH_D7_B____, BOTH_DEADFLOP1, BOTH_FJSS_TL_BR,
    BOTH_FJSS_TR_BL, BOTH_FLIP_ATTACK7, BOTH_FORCELEAP2_T__B_, BOTH_FORCELONGLEAP_ATTACK,
    BOTH_GETUP_BROLL_B, BOTH_GETUP_BROLL_F, BOTH_GETUP_FROLL_B, BOTH_GETUP_FROLL_F, BOTH_H1_S1_BR,
    BOTH_HANG_ATTACK, BOTH_JUMPATTACK6, BOTH_JUMPATTACK7, BOTH_JUMPFLIPSLASHDOWN1,
    BOTH_JUMPFLIPSTABDOWN, BOTH_K1_S1_BL, BOTH_K1_S1_BR, BOTH_K1_S1_B_, BOTH_K1_S1_TL,
    BOTH_K1_S1_TR, BOTH_K1_S1_T_, BOTH_KYLE_GRAB, BOTH_KYLE_MISS, BOTH_KYLE_PA_1, BOTH_KYLE_PA_2,
    BOTH_LK_DL_DL_S_B_1_L, BOTH_LK_DL_DL_S_L_2, BOTH_LK_DL_DL_S_SB_1_W, BOTH_LK_DL_DL_T_L_2,
    BOTH_LK_DL_DL_T_SB_1_W, BOTH_LK_DL_ST_S_B_1_L, BOTH_LK_DL_ST_S_SB_1_W, BOTH_LK_DL_ST_T_SB_1_W,
    BOTH_LK_DL_S_S_B_1_L, BOTH_LK_DL_S_S_SB_1_W, BOTH_LK_DL_S_T_SB_1_W, BOTH_LK_ST_DL_S_B_1_L,
    BOTH_LK_ST_DL_S_SB_1_W, BOTH_LK_ST_DL_T_SB_1_W, BOTH_LK_ST_ST_S_B_1_L, BOTH_LK_ST_ST_S_L_2,
    BOTH_LK_ST_ST_S_SB_1_W, BOTH_LK_ST_ST_T_L_2, BOTH_LK_ST_ST_T_SB_1_W, BOTH_LK_ST_S_S_B_1_L,
    BOTH_LK_ST_S_S_SB_1_W, BOTH_LK_ST_S_T_SB_1_W, BOTH_LK_S_DL_S_B_1_L, BOTH_LK_S_DL_S_SB_1_W,
    BOTH_LK_S_DL_T_SB_1_W, BOTH_LK_S_ST_S_B_1_L, BOTH_LK_S_ST_S_SB_1_W, BOTH_LK_S_ST_T_SB_1_W,
    BOTH_LK_S_S_S_B_1_L, BOTH_LK_S_S_S_L_2, BOTH_LK_S_S_S_SB_1_W, BOTH_LK_S_S_T_L_2,
    BOTH_LK_S_S_T_SB_1_W, BOTH_LUNGE2_B__T_, BOTH_P1_S1_BL, BOTH_P1_S1_BR, BOTH_P1_S1_TL,
    BOTH_P1_S1_TR, BOTH_P1_S1_T_, BOTH_PLAYER_PA_1, BOTH_PLAYER_PA_2, BOTH_PULL_IMPALE_STAB,
    BOTH_PULL_IMPALE_SWING, BOTH_ROLL_STAB, BOTH_SPINATTACK6, BOTH_SPINATTACK7, BOTH_STABDOWN,
    BOTH_STABDOWN_DUAL, BOTH_STABDOWN_STAFF, BOTH_VS_ATL_S, BOTH_VS_ATR_S, BOTH_VT_ATL_S,
    BOTH_VT_ATR_S,
};
use crate::codemp::game::b_public_h::{LM_ENT, LM_INTEREST};
use crate::codemp::game::bg_lib;
use crate::codemp::game::bg_misc::{
    vectoyaw, BG_CanUseFPNow, BG_EvaluateTrajectory, BG_HasYsalamiri,
};
use crate::codemp::game::bg_panimate::BG_InExtraDefenseSaberMove;
use crate::codemp::game::bg_panimate::{
    bgAllAnims, BG_AnimLength, BG_BrokenParryForAttack, BG_BrokenParryForParry, BG_InGrappleMove,
    BG_InKnockDownOnGround, BG_InRoll, BG_InSpecialJump, BG_KickingAnim, BG_KnockawayForParry,
    BG_SaberInAttack, BG_SaberInKata, BG_SaberInReturn, BG_SaberInSpecial, BG_SaberInSpecialAttack,
    BG_SaberStartTransAnim, BG_SpinningSaberAnim, BG_StabDownAnim, BG_SuperBreakLoseAnim,
    BG_SuperBreakWinAnim, PM_InSaberAnim, PM_SaberBounceForAttack, PM_SaberDeflectionForQuad,
    PM_SaberInDeflect, PM_SaberInKnockaway, PM_SaberInParry, PM_SaberInReflect,
    PM_SaberInTransition,
};
use crate::codemp::game::bg_panimate::{
    BG_SaberInAttackPure, BG_SaberInTransitionAny, PM_InKnockDown,
};
use crate::codemp::game::bg_pmove::{
    BG_G2ATSTAngles, BG_G2PlayerAngles, BG_IK_MoveArm, BG_InKnockDown, BG_KnockDownable,
    BG_SabersOff,
};
use crate::codemp::game::bg_public::{BG_GiveMeVectorFromMatrix, ET_PLAYER};
use crate::codemp::game::bg_public::{
    BROKENLIMB_LARM, BROKENLIMB_RARM, DUELTEAM_LONE, EF2_FLYING, EF2_HELD_BY_MONSTER,
    EF_DISINTEGRATION, EF_INVULNERABLE, EF_MISSILE_STICK, EF_NODRAW, ET_BODY, ET_GENERAL,
    ET_MISSILE, ET_MOVER, ET_NPC, ET_TERRAIN, EV_SABER_BLOCK, EV_SABER_CLASHFLARE, EV_SABER_HIT,
    GT_DUEL, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_TEAM, HANDEXTEND_JEDITAUNT,
    HANDEXTEND_KNOCKDOWN, HANDEXTEND_NONE, LS_A_BACK, LS_A_BACKSTAB, LS_A_BACK_CR, LS_A_FLIP_SLASH,
    LS_A_FLIP_STAB, LS_A_JUMP_T__B_, LS_A_LUNGE, LS_D1_BL, LS_D1_BR, LS_D1_B_, LS_D1_TL, LS_D1_TR,
    LS_D1_T_, LS_D1__L, LS_D1__R, LS_K1_BL, LS_K1_BR, LS_K1_TL, LS_K1_TR, LS_K1_T_, LS_NONE,
    LS_PARRY_LL, LS_PARRY_LR, LS_PARRY_UL, LS_PARRY_UP, LS_PARRY_UR, LS_READY, LS_REFLECT_LL,
    LS_REFLECT_LR, LS_REFLECT_UL, LS_REFLECT_UP, LS_REFLECT_UR, LS_SPINATTACK, LS_SPINATTACK_DUAL,
    LS_V1_BL, MASK_PLAYERSOLID, MASK_SHOT, MASK_SOLID, MOD_MELEE, MOD_ROCKET_HOMING, MOD_SABER,
    PMF_DUCKED, PMF_FOLLOW, PMF_TIME_KNOCKBACK, Q_B, Q_BL, Q_BR, Q_L, Q_R, Q_TL, Q_TR,
    SABERLOCK_LOCK, SABERLOCK_LOSE, SABERLOCK_SIDE, SABERLOCK_SUPERBREAK, SABERLOCK_TOP,
    SABERLOCK_WIN, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE, TEAM_SPECTATOR,
    WEAPON_DROPPING, WEAPON_FIRING, WEAPON_RAISING,
};
use crate::codemp::game::bg_saber::{
    saberMoveData, BG_CheckIncrementLockAnim, PM_SaberInBounce, PM_SaberInBrokenParry,
};
use crate::codemp::game::bg_saberLoad::{
    WP_SaberBladeDoTransitionDamage, WP_SaberBladeUseSecondBladeStyle,
};
use crate::codemp::game::bg_saga::bgSiegeClasses;
use crate::codemp::game::bg_saga_h::CFL_MORESABERDMG;
use crate::codemp::game::bg_vehicles_h::{VH_FIGHTER, VH_WALKER};
use crate::codemp::game::bg_weapons_h::{WP_NONE, WP_NUM_WEAPONS, WP_SABER, WP_THERMAL};
use crate::codemp::game::g_client::{g2SaberInstance, G_UpdateClientAnims, SetClientViewAngle};
use crate::codemp::game::g_combat::{G_Damage, G_Knockdown};
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_local::FRAMETIME;
use crate::codemp::game::g_local::MAX_INTEREST_POINTS;
use crate::codemp::game::g_local::{
    DAMAGE_NO_DISMEMBER, DAMAGE_NO_KNOCKBACK, DAMAGE_SABER_KNOCKBACK1, DAMAGE_SABER_KNOCKBACK1_B2,
    DAMAGE_SABER_KNOCKBACK2, DAMAGE_SABER_KNOCKBACK2_B2, FL_BOUNCE_HALF, FL_NO_KNOCKBACK,
};
use crate::codemp::game::g_main::{
    d_saberAlwaysBoxTrace, d_saberBoxTraceSize, d_saberGhoul2Collision, d_saberInterpolate,
    d_saberKickTweak, d_saberSPStyleDamage, g_debugSaberLocks, g_duelWeaponDisable,
    g_duel_fraglimit, g_entities, g_friendlyFire, g_friendlySaber, g_g2TraceLod, g_gametype,
    g_optvehtrace, g_saberBladeFaces, g_saberDamageScale, g_saberDebugPrint, g_saberDmgDelay_Idle,
    g_saberDmgDelay_Wound, g_saberLockFactor, g_saberLocking, g_saberRealisticCombat,
    g_saberTraceSaberFirst, g_saberWallDamageScale, g_svfps, g_weaponDisable, level, Com_Printf,
};
use crate::codemp::game::g_mover::G_EntIsBreakable;
use crate::codemp::game::g_object::G_RunObject;
use crate::codemp::game::g_public_h::{
    G2TRFLAG_DOGHOULTRACE, G2TRFLAG_GETSURFINDEX, G2TRFLAG_HITCORPSES, G2TRFLAG_THICK,
    SVF_NOCLIENT, SVF_USE_CURRENT_ORIGIN,
};
use crate::codemp::game::g_team::OnSameTeam;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::G_Sound;
use crate::codemp::game::g_utils::{
    G_EntitySound, G_FreeEntity, G_InitGentity, G_ModelIndex, G_SetAnim, G_SetOrigin, G_SoundIndex,
    G_Spawn,
};
use crate::codemp::game::g_utils::{G_TempEntity, G_Throw};
use crate::codemp::game::npc_ai_jedi::{Jedi_Ambush, Jedi_SaberBlockGo, Jedi_WaitingAmbush};
use crate::codemp::game::npc_combat::G_SetEnemy;
use crate::codemp::game::npc_senses::InFront;
use crate::codemp::game::npc_utils::{G_GetBoltPosition, NPC_SetBoneAngles};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, vectoangles, AngleDelta, AngleNormalize180, AngleVectors, AnglesSubtract,
    AnglesToAxis, Distance, DistanceSquared, DotProduct, LerpAngle, VectorAdd, VectorClear,
    VectorCompare, VectorCopy, VectorInverse, VectorLength, VectorMA, VectorNormalize,
    VectorNormalize2, VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::va;
use crate::codemp::game::q_shared::Q_stricmp;
use crate::codemp::game::q_shared_h::{
    entityState_t, mdxaBone_t, saberInfo_t, saber_colors_t, trace_t, usercmd_t, vec3_t, G2Trace_t,
    BLOCKED_ATK_BOUNCE, BLOCKED_BOUNCE_MOVE, BLOCKED_LOWER_LEFT, BLOCKED_LOWER_LEFT_PROJ,
    BLOCKED_LOWER_RIGHT, BLOCKED_LOWER_RIGHT_PROJ, BLOCKED_NONE, BLOCKED_PARRY_BROKEN, BLOCKED_TOP,
    BLOCKED_TOP_PROJ, BLOCKED_UPPER_LEFT, BLOCKED_UPPER_LEFT_PROJ, BLOCKED_UPPER_RIGHT,
    BLOCKED_UPPER_RIGHT_PROJ, BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_FORCEGRIP,
    BUTTON_FORCEPOWER, BUTTON_FORCE_DRAIN, BUTTON_FORCE_LIGHTNING, BUTTON_GESTURE, CHAN_AUTO,
    CHAN_BODY, CHAN_VOICE, CHAN_WEAPON, ENTITYNUM_NONE, ENTITYNUM_WORLD, FORCE_LEVEL_0,
    FORCE_LEVEL_1, FORCE_LEVEL_2, FORCE_LEVEL_3, FORCE_LEVEL_4, FORCE_LEVEL_5, FP_DRAIN, FP_GRIP,
    FP_LIGHTNING, FP_PUSH, FP_RAGE, FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE, MAX_BLADES,
    MAX_CLIENTS, MAX_G2_COLLISIONS, MAX_SABERS, NEGATIVE_Y, ORIGIN, PITCH, ROLL, SABER_BLUE,
    SABER_GREEN, SABER_LANCE, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_TRIDENT, SABER_YELLOW,
    SFL2_ALWAYS_BLOCK, SFL2_ALWAYS_BLOCK2, SS_DUAL, SS_FAST, SS_MEDIUM, SS_STAFF, SS_STRONG,
    SS_TAVION, TR_GRAVITY, TR_INTERPOLATE, TR_LINEAR, TR_STATIONARY, YAW,
};
use crate::codemp::game::q_shared_h::{
    qhandle_t, BLK_WIDE, MAX_GENTITIES, SABER_NONE, SFL_BOUNCE_ON_WALLS,
};
use crate::codemp::game::q_shared_h::{
    SFL2_NO_CLASH_FLARE, SFL2_NO_CLASH_FLARE2, SFL2_NO_DISMEMBERMENT, SFL2_NO_IDLE_EFFECT,
    SFL_NOT_ACTIVE_BLOCKING, SFL_NOT_DISARMABLE, SFL_NOT_LOCKABLE, SFL_NOT_THROWABLE,
    SFL_RETURN_DAMAGE, SFL_SINGLE_BLADE_THROWABLE,
};
use crate::codemp::game::surfaceflags_h::{CONTENTS_BODY, CONTENTS_LIGHTSABER, CONTENTS_TRIGGER};
use crate::codemp::game::teams_h::{
    CLASS_ATST, CLASS_BOBAFETT, CLASS_GONK, CLASS_INTERROGATOR, CLASS_MARK1, CLASS_MARK2,
    CLASS_MOUSE, CLASS_PROBE, CLASS_PROTOCOL, CLASS_R2D2, CLASS_R5D2, CLASS_RANCOR, CLASS_REMOTE,
    CLASS_SEEKER, CLASS_SENTRY, CLASS_VEHICLE,
};
use crate::codemp::game::tri_coll_test::tri_tri_intersect;
use crate::codemp::game::w_force::{ForceThrow, WP_ForcePowerUsable};
use crate::codemp::game::w_saber_h::{
    EVASION_NONE, SABERMAXS_X, SABERMAXS_Y, SABERMAXS_Z, SABERMINS_X, SABERMINS_Y, SABERMINS_Z,
    SABER_REFLECT_MISSILE_CONE, SEF_BLOCKED, SEF_DEFLECTED, SEF_HITENEMY, SEF_HITOBJECT,
    SEF_HITWALL, SEF_PARRIED,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `float RandFloat(float min, float max)` (w_saber.c:41) — a uniform random float in
/// `[min, max]`. Draws from the game's own 15-bit `rand()` LCG in [`bg_lib`] (which
/// overrides libc on every target), **not** the platform RNG, so the stream is the same
/// deterministic sequence everywhere — matching [`random`](super::q_shared::random).
///
/// The PC source divides by `(float)RAND_MAX` (the Xbox source used the literal `32768.0F`,
/// now the commented-out `//for linux:` line); `RAND_MAX` for the game's bg_lib LCG is
/// `0x7fff` (`32767`, the LCG's max draw), so the result reaches exactly `max` when `rand()`
/// returns `0x7fff`. All arithmetic is single precision, exactly as the C: `rand()` promotes
/// to `float`, every other operand is a `float`, and no operation widens to `double`.
pub fn RandFloat(min: f32, max: f32) -> f32 {
    //	return ((rand() * (max - min)) / 32768.0F) + min;
    //for linux:
    ((bg_lib::rand() as f32 * (max - min)) / 32767.0f32) + min // 32767.0 == (float)RAND_MAX
}

/// `void G_DebugBoxLines(vec3_t mins, vec3_t maxs, int duration)` (w_saber.c:46) — draw the
/// twelve edges of an axis-aligned box with `G_TestLine` debug lines, top face then bottom.
///
/// Guarded by `#ifdef DEBUG_SABER_BOX` in the C, which is undefined in the retail/MP build, so
/// this is dead code there. Ported faithfully anyway, mirroring the C control flow and the
/// `0x00000ff` (blue) line colour. No oracle: it is pure `G_TestLine` (temp-entity) side
/// effects, with no computable return value.
///
/// # Safety
/// Calls `G_TestLine`, which spawns a temp-entity; valid level state required.
pub unsafe fn G_DebugBoxLines(mins: &vec3_t, maxs: &vec3_t, duration: c_int) {
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];

    let x = maxs[0] - mins[0];
    let y = maxs[1] - mins[1];

    // top of box
    VectorCopy(maxs, &mut start);
    VectorCopy(maxs, &mut end);
    start[0] -= x;
    G_TestLine(&start, &end, 0x00000ff, duration);
    end[0] = start[0];
    end[1] -= y;
    G_TestLine(&start, &end, 0x00000ff, duration);
    start[1] = end[1];
    start[0] += x;
    G_TestLine(&start, &end, 0x00000ff, duration);
    G_TestLine(&start, maxs, 0x00000ff, duration);
    // bottom of box
    VectorCopy(mins, &mut start);
    VectorCopy(mins, &mut end);
    start[0] += x;
    G_TestLine(&start, &end, 0x00000ff, duration);
    end[0] = start[0];
    end[1] += y;
    G_TestLine(&start, &end, 0x00000ff, duration);
    start[1] = end[1];
    start[0] -= x;
    G_TestLine(&start, &end, 0x00000ff, duration);
    G_TestLine(&start, mins, 0x00000ff, duration);
}

/// `qboolean G_CanBeEnemy(gentity_t *self, gentity_t *enemy)` (w_saber.c:82) — decide whether
/// `self` is allowed to harm `enemy` with the saber. Both must be in-use clients. A duel locks
/// each participant to their `duelIndex`: if either is dueling someone other than the other, they
/// cannot be enemies. Below team gametypes (`g_gametype < GT_TEAM`) anyone can hit anyone; with
/// friendly fire on, even teammates can be hit; otherwise teammates (`OnSameTeam`) are off-limits.
///
/// # Safety
/// Dereferences both `gentity_t` pointers (and their `client`s) and reads the cvar globals
/// `g_gametype`/`g_friendlyFire`; callers must pass valid entities.
pub unsafe fn G_CanBeEnemy(self_: *mut gentity_t, enemy: *mut gentity_t) -> qboolean {
    if (*self_).inuse == QFALSE
        || (*enemy).inuse == QFALSE
        || (*self_).client.is_null()
        || (*enemy).client.is_null()
    {
        return QFALSE;
    }

    if (*(*self_).client).ps.duelInProgress != QFALSE
        && (*(*self_).client).ps.duelIndex != (*enemy).s.number
    {
        //dueling but not with this person
        return QFALSE;
    }

    if (*(*enemy).client).ps.duelInProgress != QFALSE
        && (*(*enemy).client).ps.duelIndex != (*self_).s.number
    {
        //other guy dueling but not with me
        return QFALSE;
    }

    if (*addr_of!(g_gametype)).integer < GT_TEAM {
        //ok, sure
        return QTRUE;
    }

    if (*addr_of!(g_friendlyFire)).integer != 0 {
        //if ff on then can inflict damage normally on teammates
        return QTRUE;
    }

    if OnSameTeam(self_, enemy) != QFALSE {
        //ff not on, don't hurt teammates
        return QFALSE;
    }

    QTRUE
}

/// `qboolean HasSetSaberOnly(void)` (w_saber.c:9006) — `qtrue` when the server's weapon
/// configuration leaves only the saber (and `WP_NONE`) enabled. Jedi Master mode is always
/// "not saber-only" (it never restricts to saber). Otherwise the relevant weapon-disable
/// bitmask is `g_duelWeaponDisable` in (power)duel and `g_weaponDisable` elsewhere; the loop
/// returns `qfalse` the moment it finds any non-saber, non-none weapon left enabled. Reads
/// the cvar globals directly, exactly as the C.
///
/// # Safety
/// Reads the mutable cvar globals `g_gametype`, `g_duelWeaponDisable`, `g_weaponDisable`.
pub unsafe fn HasSetSaberOnly() -> qboolean {
    let mut i: c_int = 0;

    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER {
        // set to 0
        return QFALSE;
    }

    let w_disable: c_int = if (*addr_of!(g_gametype)).integer == GT_DUEL
        || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        (*addr_of!(g_duelWeaponDisable)).integer
    } else {
        (*addr_of!(g_weaponDisable)).integer
    };

    while i < WP_NUM_WEAPONS {
        if (w_disable & (1 << i)) == 0 && i != WP_SABER && i != WP_NONE {
            return QFALSE;
        }

        i += 1;
    }

    QTRUE
}

/// `void UpdateClientRenderBolts(gentity_t *self, vec3_t renderOrigin, vec3_t renderAngles)`
/// (w_saber.c:6830) — recompute the cached world positions of the client's head, hands,
/// torso, crotch and feet bolts (used for saber-collision and per-surface hit location)
/// and stamp `boltValidityTime`. Without a ghoul2 instance every point collapses to the
/// player origin.
///
/// No oracle — drives `trap_G2API_GetBoltMatrix` and reads the global `level`.
///
/// # Safety
/// `self_` must be a valid entity with a non-NULL `client`; `render_origin`/`render_angles`
/// are the render-frame transform passed straight to the ghoul2 trap.
pub unsafe fn UpdateClientRenderBolts(
    self_: *mut gentity_t,
    render_origin: &vec3_t,
    render_angles: &vec3_t,
) {
    let mut bolt_matrix: mdxaBone_t = core::mem::zeroed();
    let client = (*self_).client;

    if (*self_).ghoul2.is_null() {
        let origin = (*client).ps.origin;
        VectorCopy(&origin, &mut (*client).renderInfo.headPoint);
        VectorCopy(&origin, &mut (*client).renderInfo.handRPoint);
        VectorCopy(&origin, &mut (*client).renderInfo.handLPoint);
        VectorCopy(&origin, &mut (*client).renderInfo.torsoPoint);
        VectorCopy(&origin, &mut (*client).renderInfo.crotchPoint);
        VectorCopy(&origin, &mut (*client).renderInfo.footRPoint);
        VectorCopy(&origin, &mut (*client).renderInfo.footLPoint);
    } else {
        //head
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            (*client).renderInfo.headBolt,
            &mut bolt_matrix,
            render_angles,
            render_origin,
            (*addr_of!(level)).time,
            null_mut(),
            &(*self_).modelScale,
        );
        (*client).renderInfo.headPoint[0] = bolt_matrix.matrix[0][3];
        (*client).renderInfo.headPoint[1] = bolt_matrix.matrix[1][3];
        (*client).renderInfo.headPoint[2] = bolt_matrix.matrix[2][3];

        //right hand
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            (*client).renderInfo.handRBolt,
            &mut bolt_matrix,
            render_angles,
            render_origin,
            (*addr_of!(level)).time,
            null_mut(),
            &(*self_).modelScale,
        );
        (*client).renderInfo.handRPoint[0] = bolt_matrix.matrix[0][3];
        (*client).renderInfo.handRPoint[1] = bolt_matrix.matrix[1][3];
        (*client).renderInfo.handRPoint[2] = bolt_matrix.matrix[2][3];

        //left hand
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            (*client).renderInfo.handLBolt,
            &mut bolt_matrix,
            render_angles,
            render_origin,
            (*addr_of!(level)).time,
            null_mut(),
            &(*self_).modelScale,
        );
        (*client).renderInfo.handLPoint[0] = bolt_matrix.matrix[0][3];
        (*client).renderInfo.handLPoint[1] = bolt_matrix.matrix[1][3];
        (*client).renderInfo.handLPoint[2] = bolt_matrix.matrix[2][3];

        //chest
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            (*client).renderInfo.torsoBolt,
            &mut bolt_matrix,
            render_angles,
            render_origin,
            (*addr_of!(level)).time,
            null_mut(),
            &(*self_).modelScale,
        );
        (*client).renderInfo.torsoPoint[0] = bolt_matrix.matrix[0][3];
        (*client).renderInfo.torsoPoint[1] = bolt_matrix.matrix[1][3];
        (*client).renderInfo.torsoPoint[2] = bolt_matrix.matrix[2][3];

        //crotch
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            (*client).renderInfo.crotchBolt,
            &mut bolt_matrix,
            render_angles,
            render_origin,
            (*addr_of!(level)).time,
            null_mut(),
            &(*self_).modelScale,
        );
        (*client).renderInfo.crotchPoint[0] = bolt_matrix.matrix[0][3];
        (*client).renderInfo.crotchPoint[1] = bolt_matrix.matrix[1][3];
        (*client).renderInfo.crotchPoint[2] = bolt_matrix.matrix[2][3];

        //right foot
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            (*client).renderInfo.footRBolt,
            &mut bolt_matrix,
            render_angles,
            render_origin,
            (*addr_of!(level)).time,
            null_mut(),
            &(*self_).modelScale,
        );
        (*client).renderInfo.footRPoint[0] = bolt_matrix.matrix[0][3];
        (*client).renderInfo.footRPoint[1] = bolt_matrix.matrix[1][3];
        (*client).renderInfo.footRPoint[2] = bolt_matrix.matrix[2][3];

        //left foot
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            (*client).renderInfo.footLBolt,
            &mut bolt_matrix,
            render_angles,
            render_origin,
            (*addr_of!(level)).time,
            null_mut(),
            &(*self_).modelScale,
        );
        (*client).renderInfo.footLPoint[0] = bolt_matrix.matrix[0][3];
        (*client).renderInfo.footLPoint[1] = bolt_matrix.matrix[1][3];
        (*client).renderInfo.footLPoint[2] = bolt_matrix.matrix[2][3];
    }

    (*client).renderInfo.boltValidityTime = (*addr_of!(level)).time;
}

/// `void UpdateClientRenderinfo(gentity_t *self, vec3_t renderOrigin, vec3_t renderAngles)`
/// (w_saber.c:6893).
///
/// Cheaply refresh `self`'s client `renderInfo` once per server frame (gated on
/// `mPCalcTime < level.time`): re-add the humanoid bolts whenever the ghoul2 instance
/// changed, copy the eye angles, take the first frame of the current torso/legs anims as
/// the rough server-side frames, and recompute the muzzle point/dir and eye point. *"We're
/// just going to give rough estimates on most of this stuff, it's not like most of it
/// matters."* The `#if 0` per-frame field clears are omitted (the C compiles them out).
///
/// The `if (g_debugServerSkel.integer)` debug block recomputes the seven bolt points and
/// draws the skeleton with `G_TestLine`; `G_TestLine` lives in the not-yet-ported NAV subsystem
/// (`ai_wpnav.c`) and is not compiled into the retail server, so — exactly as every other
/// `G_TestLine` site in this file — the draws are faithful-omitted. The bolt-point
/// recomputations they fed have no other consumer, so the whole cheat-only block is
/// faithful-omitted too. No oracle (drives the `trap_G2API_*` syscalls and reads
/// entity/level state).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a non-null `client` (the C dereferences
/// `self->client` unconditionally). `renderOrigin`/`renderAngles` must be valid `vec3_t`.
pub unsafe fn UpdateClientRenderinfo(
    self_: *mut gentity_t,
    _renderOrigin: &vec3_t,
    _renderAngles: &vec3_t,
) {
    let ri = &mut (*(*self_).client).renderInfo;
    if ri.mPCalcTime < (*addr_of!(level)).time {
        //We're just going to give rough estimates on most of this stuff,
        //it's not like most of it matters.

        // #if 0'd per-frame field clears (head/torso ranges, fps mods, customRGB,
        // renderFlags, lockYaw, head/torso angles, legsYaw) faithful-omitted as in C.

        if !(*self_).ghoul2.is_null() && (*self_).ghoul2 != ri.lastG2 {
            //the g2 instance changed, so update all the bolts.
            //rwwFIXMEFIXME: Base on skeleton used? Assuming humanoid currently.
            ri.lastG2 = (*self_).ghoul2;

            if (*self_).localAnimIndex <= 1 {
                ri.headBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*head_eyes");
                ri.handRBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*r_hand");
                ri.handLBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_hand");
                ri.torsoBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "thoracic");
                ri.crotchBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "pelvis");
                ri.footRBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*r_leg_foot");
                ri.footLBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_leg_foot");
                ri.motionBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "Motion");
            } else {
                ri.headBolt = -1;
                ri.handRBolt = -1;
                ri.handLBolt = -1;
                ri.torsoBolt = -1;
                ri.crotchBolt = -1;
                ri.footRBolt = -1;
                ri.footLBolt = -1;
                ri.motionBolt = -1;
            }

            ri.lastG2 = (*self_).ghoul2;
        }

        VectorCopy(
            &(*(*self_).client).ps.viewangles,
            &mut (*(*self_).client).renderInfo.eyeAngles,
        );

        //we'll just say the legs/torso are whatever the first frame of our current anim is.
        ri.torsoFrame = (*(*addr_of!(bgAllAnims))[(*self_).localAnimIndex as usize]
            .anims
            .add((*(*self_).client).ps.torsoAnim as usize))
        .firstFrame as c_int;
        ri.legsFrame = (*(*addr_of!(bgAllAnims))[(*self_).localAnimIndex as usize]
            .anims
            .add((*(*self_).client).ps.legsAnim as usize))
        .firstFrame as c_int;
        // if (g_debugServerSkel.integer) { ...recompute bolt points, G_TestLine the
        // skeleton... } — faithful-omitted: cheat-only debug block whose only consumer is
        // G_TestLine (not-yet-ported ai_wpnav.c, not in the retail server). See doc above.

        //muzzle point calc (we are going to be cheap here)
        VectorCopy(&ri.muzzlePoint, &mut ri.muzzlePointOld);
        VectorCopy(&(*(*self_).client).ps.origin, &mut ri.muzzlePoint);
        VectorCopy(&ri.muzzleDir, &mut ri.muzzleDirOld);
        AngleVectors(
            &(*(*self_).client).ps.viewangles,
            Some(&mut ri.muzzleDir),
            None,
            None,
        );
        ri.mPCalcTime = (*addr_of!(level)).time;

        VectorCopy(&(*(*self_).client).ps.origin, &mut ri.eyePoint);
        ri.eyePoint[2] += (*(*self_).client).ps.viewheight as f32;
    }
}

/// `int VectorCompare2( const vec3_t v1, const vec3_t v2 )` (w_saber.c:4858) — an
/// epsilon-tolerant vector equality used by `G_SPSaberDamageTraceLerped` to decide whether
/// the saber blade's traced endpoints moved at all this frame (a `0.0001` slop on each axis,
/// not the bit-exact [`VectorCompare`](super::q_shared::VectorCompare)). Returns `0` the
/// moment any component differs by more than the epsilon, else `1`. All comparisons are
/// single precision, exactly as the C (`vec3_t` is `float[3]`).
pub fn VectorCompare2(v1: &vec3_t, v2: &vec3_t) -> c_int {
    if v1[0] > v2[0] + 0.0001f32
        || v1[0] < v2[0] - 0.0001f32
        || v1[1] > v2[1] + 0.0001f32
        || v1[1] < v2[1] - 0.0001f32
        || v1[2] > v2[2] + 0.0001f32
        || v1[2] < v2[2] - 0.0001f32
    {
        return 0;
    }
    1
}

/// `int WPDEBUG_SaberColor( saber_colors_t saberColor )` (w_saber.c:2677) — maps a blade
/// color to its packed `0x00BBGGRR` debug-line color (the values feed `G_TestLine` / saber
/// debug rendering). Unknown colors fall through to white (`0x00ffffff`). A pure lookup,
/// faithful to the C switch.
pub fn WPDEBUG_SaberColor(saberColor: saber_colors_t) -> c_int {
    match saberColor {
        SABER_RED => 0x000000ff,
        SABER_ORANGE => 0x000088ff,
        SABER_YELLOW => 0x0000ffff,
        SABER_GREEN => 0x0000ff00,
        SABER_BLUE => 0x00ff0000,
        SABER_PURPLE => 0x00ff00ff,
        _ => 0x00ffffff, // white
    }
}

/// `qboolean G_PrettyCloseIGuess(float a, float b, float tolerance)` (w_saber.c:7529) — a
/// symmetric float near-equality predicate (`|a - b| < tolerance`) used by the grapple/grab
/// code to compare two entities' Z origins. `QTRUE` when within tolerance, else `QFALSE`.
/// Single-precision throughout, faithful to the C.
pub fn G_PrettyCloseIGuess(a: f32, b: f32, tolerance: f32) -> qboolean {
    if (a - b) < tolerance && (a - b) > -tolerance {
        return QTRUE;
    }
    QFALSE
}

/// `void SaberBounceSound( gentity_t *self, gentity_t *other, trace_t *trace )`
/// (w_saber.c:5800) — the `touch` callback installed on a thrown saber's bounce entity. It
/// snaps the saber's visual angular base to its current angles with the pitch forced to 90
/// degrees (the blade lying flat); `other` and `trace` are unused. A `pub unsafe extern "C"`
/// fn for the `gentity_t::touch` fn-pointer ABI.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SaberBounceSound(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    VectorCopy(&(*self_).r.currentAngles, &mut (*self_).s.apos.trBase);
    (*self_).s.apos.trBase[PITCH] = 90.0;
}

/// `void DeadSaberThink(gentity_t *saberent)` (w_saber.c:5806) — think handler for the short-lived
/// "dead" saber spawned by [`MakeDeadSaber`]: once its `speed` lifetime stamp has passed it swaps its
/// `think` to [`G_FreeEntity`] to remove itself next frame; otherwise it advances its physics via
/// [`G_RunObject`].
///
/// No oracle: entity-state think over the opaque `gentity_t` driving [`G_RunObject`]/[`G_FreeEntity`],
/// per the side-effect precedent.
///
/// # Safety
/// `saberent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn DeadSaberThink(saberent: *mut gentity_t) {
    if (*saberent).speed < (*addr_of!(level)).time as f32 {
        (*saberent).think = Some(G_FreeEntity);
        (*saberent).nextthink = (*addr_of!(level)).time;
        return;
    }

    G_RunObject(saberent);
}

/// `void MakeDeadSaber(gentity_t *ent)` (w_saber.c:5818) — spawns a short-lived "dead" saber
/// entity so a dropped/destroyed saber appears to fall out of the air. Bails outright in Jedi
/// Master mode (the only saber on the level is really a world object). Otherwise spawns a
/// [`G_Spawn`] entity, copies the source saber's origin/angles, marks it a `"deadsaber"` trigger
/// missile with a gravity trajectory and a random angular spin, gives it the owner's saber ghoul2
/// model via [`WP_SaberAddG2Model`] (freeing itself if the owner has no usable saber model), seeds
/// it to remove itself after 4 seconds, inherits the real saber's launch velocity, settles its
/// position with [`saberMoveBack`], and links it into the world. Installs [`SaberBounceSound`] as
/// `touch` and the [`DeadSaberThink`] sibling as `think`.
///
/// No oracle: entity-state spawn over the opaque `gentity_t` driving [`G_Spawn`]/[`G_FreeEntity`]/
/// [`trap::LinkEntity`]/[`WP_SaberAddG2Model`], per the side-effect precedent.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `r.ownerNum`/`s.number` index `g_entities`.
pub unsafe fn MakeDeadSaber(ent: *mut gentity_t) {
    //spawn a "dead" saber entity here so it looks like the saber fell out of the air.
    //This entity will remove itself after a very short time period.
    let mut startorg: vec3_t = [0.0; 3];
    let mut startang: vec3_t = [0.0; 3];

    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER {
        //never spawn a dead saber in JM, because the only saber on the level is really a world object
        //G_Sound(ent, CHAN_AUTO, saberOffSound);
        return;
    }

    let saberent = G_Spawn();

    VectorCopy(&(*ent).r.currentOrigin, &mut startorg);
    VectorCopy(&(*ent).r.currentAngles, &mut startang);

    (*saberent).classname = c"deadsaber".as_ptr() as *mut c_char;

    (*saberent).r.svFlags = SVF_USE_CURRENT_ORIGIN as c_int;
    (*saberent).r.ownerNum = (*ent).s.number;

    (*saberent).clipmask = MASK_PLAYERSOLID;
    (*saberent).r.contents = CONTENTS_TRIGGER; //0;

    VectorSet(&mut (*saberent).r.mins, -3.0, -3.0, -1.5);
    VectorSet(&mut (*saberent).r.maxs, 3.0, 3.0, 1.5);

    (*saberent).touch = Some(SaberBounceSound);

    (*saberent).think = Some(DeadSaberThink);
    (*saberent).nextthink = (*addr_of!(level)).time;

    VectorCopy(&startorg, &mut (*saberent).s.pos.trBase);
    VectorCopy(&startang, &mut (*saberent).s.apos.trBase);

    VectorCopy(&startorg, &mut (*saberent).s.origin);
    VectorCopy(&startang, &mut (*saberent).s.angles);

    VectorCopy(&startorg, &mut (*saberent).r.currentOrigin);
    VectorCopy(&startang, &mut (*saberent).r.currentAngles);

    (*saberent).s.apos.trType = TR_GRAVITY;
    (*saberent).s.apos.trDelta[0] = Q_irand(200, 800) as f32;
    (*saberent).s.apos.trDelta[1] = Q_irand(200, 800) as f32;
    (*saberent).s.apos.trDelta[2] = Q_irand(200, 800) as f32;
    (*saberent).s.apos.trTime = (*addr_of!(level)).time - 50;

    (*saberent).s.pos.trType = TR_GRAVITY;
    (*saberent).s.pos.trTime = (*addr_of!(level)).time - 50;
    (*saberent).flags = FL_BOUNCE_HALF;
    if (*ent).r.ownerNum >= 0 && (*ent).r.ownerNum < ENTITYNUM_WORLD {
        let owner: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*ent).r.ownerNum as usize);

        if (*owner).inuse == QTRUE
            && !(*owner).client.is_null()
            && (*(*owner).client).saber[0].model[0] != 0
        {
            WP_SaberAddG2Model(
                saberent,
                (*(*owner).client).saber[0].model.as_ptr(),
                (*(*owner).client).saber[0].skin,
            );
        } else {
            //WP_SaberAddG2Model( saberent, NULL, 0 );
            //argh!!!!
            G_FreeEntity(saberent);
            return;
        }
    }

    (*saberent).s.modelGhoul2 = 1;
    (*saberent).s.g2radius = 20;

    (*saberent).s.eType = ET_MISSILE;
    (*saberent).s.weapon = WP_SABER;

    (*saberent).speed = ((*addr_of!(level)).time + 4000) as f32;

    (*saberent).bounceCount = 12;

    //fall off in the direction the real saber was headed
    VectorCopy(&(*ent).s.pos.trDelta, &mut (*saberent).s.pos.trDelta);

    saberMoveBack(saberent, QTRUE);
    (*saberent).s.pos.trType = TR_GRAVITY;

    trap::LinkEntity(saberent);
}

/// `void WP_SaberStartMissileBlockCheck( gentity_t *self, usercmd_t *ucmd )` (w_saber.c:5066) —
/// per-frame auto-defense / look-target scan: gather all entities in a 256-unit box around
/// `self`, pick the nearest valid enemy to look at (player only), and — if `self` is allowed to
/// block this frame (the long `doFullRoutine` gauntlet) — find the closest incoming missile/thrown
/// saber heading at `self` and either start an NPC saber-block / Boba evade, or a player
/// non-random block.
///
/// Faithful port mirroring the C control flow, including the `(dot1 = DotProduct(...))` /
/// `(dot2 = DotProduct(...))` assignment-in-condition idioms (the stored values are otherwise
/// unused). The disabled `if (0)` tripmine/detpack branch is omitted exactly as the C `#if 0`
/// path is dead. No oracle: pure entity-state + trap (`EntitiesInBox`/`Trace`) driven with no
/// computable return.
///
/// # Safety
/// Dereferences `self`, its `client`/`NPC`, and the global `g_entities`/`level`; issues
/// `trap_*` syscalls. Callers must pass a valid in-level entity with a client.
pub unsafe fn WP_SaberStartMissileBlockCheck(self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    let mut dist: f32;
    let mut incoming: *mut gentity_t = null_mut();
    let mut entityList: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
    let numListedEntities: c_int;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let radius: f32 = 256.0;
    let mut forward: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut missile_dir: vec3_t = [0.0; 3];
    let mut fwdangles: vec3_t = [0.0, 0.0, 0.0];
    let mut trace: trace_t;
    let mut traceTo: vec3_t = [0.0; 3];
    let mut entDir: vec3_t = [0.0; 3];
    #[allow(unused_assignments)]
    let mut dot1: f32;
    #[allow(unused_assignments)]
    let mut dot2: f32;
    let mut lookTDist: f32 = -1.0;
    let mut lookT: *mut gentity_t = null_mut();
    let mut doFullRoutine: qboolean = QTRUE;

    //keep this updated even if we don't get below
    if ((*(*self_).client).ps.eFlags2 & EF2_HELD_BY_MONSTER) == 0 {
        //lookTarget is set by and to the monster that's holding you, no other operations can change that
        (*(*self_).client).ps.hasLookTarget = QFALSE;
    }

    if (*(*self_).client).ps.weapon != WP_SABER && (*(*self_).client).NPC_class != CLASS_BOBAFETT {
        doFullRoutine = QFALSE;
    } else if (*(*self_).client).ps.saberInFlight != QFALSE {
        doFullRoutine = QFALSE;
    } else if ((*(*self_).client).ps.fd.forcePowersActive & (1 << FP_LIGHTNING)) != 0 {
        //can't block while zapping
        doFullRoutine = QFALSE;
    } else if ((*(*self_).client).ps.fd.forcePowersActive & (1 << FP_DRAIN)) != 0 {
        //can't block while draining
        doFullRoutine = QFALSE;
    } else if ((*(*self_).client).ps.fd.forcePowersActive & (1 << FP_PUSH)) != 0 {
        //can't block while shoving
        doFullRoutine = QFALSE;
    } else if ((*(*self_).client).ps.fd.forcePowersActive & (1 << FP_GRIP)) != 0 {
        //can't block while gripping (FIXME: or should it break the grip?  Pain should break the grip, I think...)
        doFullRoutine = QFALSE;
    }

    if (*(*self_).client).ps.weaponTime > 0 {
        //don't autoblock while busy with stuff
        return;
    }

    if (*(*self_).client).saber[0].saberFlags & SFL_NOT_ACTIVE_BLOCKING != 0 {
        //can't actively block with this saber type
        return;
    }

    if (*self_).health <= 0 {
        //dead don't try to block (NOTE: actual deflection happens in missile code)
        return;
    }
    if PM_InKnockDown(&mut (*(*self_).client).ps) != QFALSE {
        //can't block when knocked down
        return;
    }

    if BG_SabersOff(&mut (*(*self_).client).ps) != QFALSE
        && (*(*self_).client).NPC_class != CLASS_BOBAFETT
    {
        if (*self_).s.eType != ET_NPC {
            //player doesn't auto-activate
            doFullRoutine = QFALSE;
        }
    }

    if (*self_).s.eType == ET_PLAYER {
        //don't do this if already attacking!
        if ((*ucmd).buttons & BUTTON_ATTACK) != 0
            || BG_SaberInAttack((*(*self_).client).ps.saberMove) != QFALSE
            || BG_SaberInSpecialAttack((*(*self_).client).ps.torsoAnim) != QFALSE
            || BG_SaberInTransitionAny((*(*self_).client).ps.saberMove) != QFALSE
        {
            doFullRoutine = QFALSE;
        }
    }

    if (*(*self_).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize]
        > (*addr_of!(level)).time
    {
        //can't block while gripping (FIXME: or should it break the grip?  Pain should break the grip, I think...)
        doFullRoutine = QFALSE;
    }

    fwdangles[1] = (*(*self_).client).ps.viewangles[1];
    AngleVectors(&fwdangles, Some(&mut forward), None, None);

    for i in 0..3 {
        mins[i] = (*self_).r.currentOrigin[i] - radius;
        maxs[i] = (*self_).r.currentOrigin[i] + radius;
    }

    numListedEntities = trap::EntitiesInBox(&mins, &maxs, &mut entityList);

    let mut closestDist: f32 = radius;

    for e in 0..numListedEntities {
        let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add(entityList[e as usize] as usize);

        if ent == self_ {
            continue;
        }

        //as long as we're here I'm going to get a looktarget too, I guess. -rww
        if (*self_).s.eType == ET_PLAYER
            && !(*ent).client.is_null()
            && ((*ent).s.eType == ET_NPC || (*ent).s.eType == ET_PLAYER)
            && OnSameTeam(ent, self_) == QFALSE
            && (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR
            && ((*(*ent).client).ps.pm_flags & PMF_FOLLOW) == 0
            && ((*ent).s.eType != ET_NPC || (*ent).s.NPC_class != CLASS_VEHICLE) //don't look at vehicle NPCs
            && (*ent).health > 0
        {
            //seems like a valid enemy to look at.
            let mut vecSub: vec3_t = [0.0; 3];
            let vecLen: f32;

            VectorSubtract(
                &(*(*self_).client).ps.origin,
                &(*(*ent).client).ps.origin,
                &mut vecSub,
            );
            vecLen = VectorLength(&vecSub);

            if lookTDist == -1.0 || vecLen < lookTDist {
                let tr: trace_t;
                let mut myEyes: vec3_t = [0.0; 3];

                VectorCopy(&(*(*self_).client).ps.origin, &mut myEyes);
                myEyes[2] += (*(*self_).client).ps.viewheight as f32;

                tr = trap::Trace(
                    &myEyes,
                    &vec3_origin,
                    &vec3_origin,
                    &(*(*ent).client).ps.origin,
                    (*self_).s.number,
                    MASK_PLAYERSOLID,
                );

                if tr.fraction == 1.0 || tr.entityNum as c_int == (*ent).s.number {
                    //we have a clear line of sight to him, so it's all good.
                    lookT = ent;
                    lookTDist = vecLen;
                }
            }
        }

        if doFullRoutine == QFALSE {
            //don't care about the rest then
            continue;
        }

        if (*ent).r.ownerNum == (*self_).s.number {
            continue;
        }
        if (*ent).inuse == QFALSE {
            continue;
        }
        if (*ent).s.eType != ET_MISSILE && ((*ent).s.eFlags & EF_MISSILE_STICK) == 0 {
            //not a normal projectile
            let pOwner: *mut gentity_t;

            if (*ent).r.ownerNum < 0 || (*ent).r.ownerNum >= ENTITYNUM_WORLD {
                //not going to be a client then.
                continue;
            }

            pOwner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*ent).r.ownerNum as usize);

            if (*pOwner).inuse == QFALSE || (*pOwner).client.is_null() {
                continue; //not valid cl owner
            }

            if (*(*pOwner).client).ps.saberEntityNum == 0
                || (*(*pOwner).client).ps.saberInFlight == QFALSE
                || (*(*pOwner).client).ps.saberEntityNum != (*ent).s.number
            {
                //the saber is knocked away and/or not flying actively, or this ent is not the cl's saber ent at all
                continue;
            }

            //If we get here then it's ok to be treated as a thrown saber, I guess.
        } else if (*ent).s.pos.trType == TR_STATIONARY && (*self_).s.eType == ET_PLAYER {
            //nothing you can do with a stationary missile if you're the player
            continue;
        }

        //see if they're in front of me
        VectorSubtract(&(*ent).r.currentOrigin, &(*self_).r.currentOrigin, &mut dir);
        dist = VectorNormalize(&mut dir);
        //FIXME: handle detpacks, proximity mines and tripmines
        if (*ent).s.weapon == WP_THERMAL {
            //thermal detonator!
            if !(*self_).NPC.is_null() && dist < (*ent).splashRadius as f32 {
                if dist < (*ent).splashRadius as f32
                    && ((*ent).nextthink as i64) < (*addr_of!(level)).time as i64 + 600
                    && (*ent).count != 0
                    && (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE
                    && ((*ent).s.pos.trType == TR_STATIONARY
                        || (*ent).s.pos.trType == TR_INTERPOLATE
                        || {
                            dot1 = DotProduct(&dir, &forward);
                            dot1 < SABER_REFLECT_MISSILE_CONE
                        }
                        || WP_ForcePowerUsable(self_, FP_PUSH) == QFALSE)
                {
                    //TD is close enough to hurt me, I'm on the ground and the thing is at rest or behind me and about to blow up, or I don't have force-push so force-jump!
                    //FIXME: sometimes this might make me just jump into it...?
                    (*(*self_).client).ps.fd.forceJumpCharge = 480.0;
                } else if (*(*self_).client).NPC_class != CLASS_BOBAFETT {
                    //FIXME: check forcePushRadius[NPC->client->ps.forcePowerLevel[FP_PUSH]]
                    ForceThrow(self_, QFALSE);
                }
            }
            continue;
        } else if (*ent).splashDamage != 0 && (*ent).splashRadius != 0 {
            //exploding missile
            //FIXME: handle tripmines and detpacks somehow...
            //			maybe do a force-gesture that makes them explode?
            //			But what if we're within it's splashradius?
            if (*self_).s.eType == ET_PLAYER {
                //players don't auto-handle these at all
                continue;
            } else {
                //if ( ent->s.pos.trType == TR_STATIONARY && (ent->s.eFlags&EF_MISSILE_STICK)
                //	&& 	self->client->NPC_class != CLASS_BOBAFETT )
                if false
                /* if (0) //Maybe handle this later? */
                {
                    //a placed explosive like a tripmine or detpack
                    // (dead `if (0)` branch, faithfully omitted)
                } else if dist < (*ent).splashRadius as f32
                    && (*(*self_).client).ps.groundEntityNum != ENTITYNUM_NONE
                    && (DotProduct(&dir, &forward) < SABER_REFLECT_MISSILE_CONE
                        || WP_ForcePowerUsable(self_, FP_PUSH) == QFALSE)
                {
                    //NPCs try to evade it
                    (*(*self_).client).ps.fd.forceJumpCharge = 480.0;
                } else if (*(*self_).client).NPC_class != CLASS_BOBAFETT {
                    //else, try to force-throw it away
                    //FIXME: check forcePushRadius[NPC->client->ps.forcePowerLevel[FP_PUSH]]
                    ForceThrow(self_, QFALSE);
                }
            }
            //otherwise, can't block it, so we're screwed
            continue;
        }

        if (*ent).s.weapon != WP_SABER {
            //only block shots coming from behind
            dot1 = DotProduct(&dir, &forward);
            if dot1 < SABER_REFLECT_MISSILE_CONE {
                continue;
            }
        } else if (*self_).s.eType == ET_PLAYER {
            //player never auto-blocks thrown sabers
            continue;
        } //NPCs always try to block sabers coming from behind!

        //see if they're heading towards me
        VectorCopy(&(*ent).s.pos.trDelta, &mut missile_dir);
        VectorNormalize(&mut missile_dir);
        dot2 = DotProduct(&dir, &missile_dir);
        if dot2 > 0.0 {
            continue;
        }

        //FIXME: must have a clear trace to me, too...
        if dist < closestDist {
            VectorCopy(&(*self_).r.currentOrigin, &mut traceTo);
            traceTo[2] = (*self_).r.absmax[2] - 4.0;
            trace = trap::Trace(
                &(*ent).r.currentOrigin,
                &(*ent).r.mins,
                &(*ent).r.maxs,
                &traceTo,
                (*ent).s.number,
                (*ent).clipmask,
            );
            if trace.allsolid != 0
                || trace.startsolid != 0
                || (trace.fraction < 1.0
                    && trace.entityNum as c_int != (*self_).s.number
                    && trace.entityNum as c_int != (*(*self_).client).ps.saberEntityNum)
            {
                //okay, try one more check
                VectorNormalize2(&(*ent).s.pos.trDelta, &mut entDir);
                VectorMA(&(*ent).r.currentOrigin, radius, &entDir, &mut traceTo);
                trace = trap::Trace(
                    &(*ent).r.currentOrigin,
                    &(*ent).r.mins,
                    &(*ent).r.maxs,
                    &traceTo,
                    (*ent).s.number,
                    (*ent).clipmask,
                );
                if trace.allsolid != 0
                    || trace.startsolid != 0
                    || (trace.fraction < 1.0
                        && trace.entityNum as c_int != (*self_).s.number
                        && trace.entityNum as c_int != (*(*self_).client).ps.saberEntityNum)
                {
                    //can't hit me, ignore it
                    continue;
                }
            }
            if (*self_).s.eType == ET_NPC {
                //An NPC
                if !(*self_).NPC.is_null()
                    && (*self_).enemy.is_null()
                    && (*ent).r.ownerNum != ENTITYNUM_NONE
                {
                    let owner: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities)
                        .cast::<gentity_t>())
                    .add((*ent).r.ownerNum as usize);
                    if (*owner).health >= 0
                        && ((*owner).client.is_null()
                            || (*(*owner).client).playerTeam != (*(*self_).client).playerTeam)
                    {
                        G_SetEnemy(self_, owner);
                    }
                }
            }
            //FIXME: if NPC, predict the intersection between my current velocity/path and the missile's, see if it intersects my bounding box (+/-saberLength?), don't try to deflect unless it does?
            closestDist = dist;
            incoming = ent;
        }
    }

    if (*self_).s.eType == ET_NPC && (*self_).localAnimIndex <= 1 {
        //humanoid NPCs don't set angles based on server angles for looking, unlike other NPCs
        if !(*self_).client.is_null() && (*(*self_).client).renderInfo.lookTarget < ENTITYNUM_WORLD
        {
            lookT = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*(*self_).client).renderInfo.lookTarget as usize);
        }
    }

    if !lookT.is_null() {
        //we got a looktarget at some point so we'll assign it then.
        if ((*(*self_).client).ps.eFlags2 & EF2_HELD_BY_MONSTER) == 0 {
            //lookTarget is set by and to the monster that's holding you, no other operations can change that
            (*(*self_).client).ps.hasLookTarget = QTRUE;
            (*(*self_).client).ps.lookTarget = (*lookT).s.number;
        }
    }

    if doFullRoutine == QFALSE {
        //then we're done now
        return;
    }

    if !incoming.is_null() {
        if !(*self_).NPC.is_null()
        /*&& !G_ControlledByPlayer( self )*/
        {
            if Jedi_WaitingAmbush(self_) != QFALSE {
                Jedi_Ambush(self_);
            }
            if (*(*self_).client).NPC_class == CLASS_BOBAFETT
                && ((*(*self_).client).ps.eFlags2 & EF2_FLYING) != 0 //moveType == MT_FLYSWIM
                && (*incoming).methodOfDeath != MOD_ROCKET_HOMING
            {
                //a hovering Boba Fett, not a tracking rocket
                if Q_irand(0, 1) == 0 {
                    //strafe
                    (*(*self_).NPC).standTime = 0;
                    (*(*self_).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                        (*addr_of!(level)).time + Q_irand(1000, 2000);
                }
                if Q_irand(0, 1) == 0 {
                    //go up/down
                    TIMER_Set(self_, c"heightChange".as_ptr(), Q_irand(1000, 3000));
                    (*(*self_).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                        (*addr_of!(level)).time + Q_irand(1000, 2000);
                }
            } else if Jedi_SaberBlockGo(
                self_,
                &mut (*(*self_).NPC).last_ucmd,
                null_mut(),
                null_mut(),
                incoming,
                0.0,
            ) != EVASION_NONE
            {
                //make sure to turn on your saber if it's not on
                if (*(*self_).client).NPC_class != CLASS_BOBAFETT {
                    //self->client->ps.SaberActivate();
                    WP_ActivateSaber(self_);
                }
            }
        } else {
            //player
            let owner: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*incoming).r.ownerNum as usize);

            WP_SaberBlockNonRandom(self_, &(*incoming).r.currentOrigin, QTRUE);
            if !owner.is_null()
                && !(*owner).client.is_null()
                && ((*self_).enemy.is_null() || (*(*self_).enemy).s.weapon != WP_SABER)
            {
                //keep enemy jedi over shooters
                (*self_).enemy = owner;
                //NPC_SetLookTarget( self, owner->s.number, level.time+1000 );
                //player looktargetting done differently
            }
        }
    }
}

/// `#define MIN_SABER_SLICE_DISTANCE 50` (w_saber.c:5460) — outgoing thrown-saber slice radius.
const MIN_SABER_SLICE_DISTANCE: c_int = 50;
/// `#define MIN_SABER_SLICE_RETURN_DISTANCE 30` (w_saber.c:5462) — returning slice radius.
const MIN_SABER_SLICE_RETURN_DISTANCE: c_int = 30;
/// `#define SABER_THROWN_HIT_DAMAGE 30` (w_saber.c:5464) — base damage of a knocked-out saber.
const SABER_THROWN_HIT_DAMAGE: c_int = 30;

/// `static GAME_INLINE void saberMoveBack( gentity_t *ent, qboolean goingBack )` (w_saber.c:5736) —
/// advances a flying saber entity along its (now-`TR_LINEAR`) trajectory: it evaluates the position
/// and angle trajectories for the current frame, then — `#ifdef THROWN_SABER_COMP` (defined, so the
/// block is **live**) — when the saber is not returning and not in free-fall, performs a forward
/// compensation box-sweep ([`trap::Trace`], +32u beyond the predicted move) so a fast saber can't
/// tunnel through things between predictions. On a blocking hit (not the owner, not another
/// lightsaber) it zeroes the trajectory delta, runs [`CheckThrownSaberDamaged`] at point-blank, and
/// (unless that knocked it into `TR_GRAVITY`) calls [`thrownSaberTouch`]. Otherwise it just commits
/// the freshly-evaluated origin.
///
/// No oracle: a live entity move over the opaque `gentity_t` driving
/// [`BG_EvaluateTrajectory`]/[`trap::Trace`] and damage side effects, per the side-effect precedent.
/// The `trap_Trace` NULL-box arg is N/A here (an explicit ±24/±8 box is built); the compensation
/// trace uses that box exactly as the C.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `r.ownerNum` indexes `g_entities`.
pub unsafe fn saberMoveBack(ent: *mut gentity_t, goingBack: qboolean) {
    let mut origin: vec3_t = [0.0; 3];
    let mut oldOrg: vec3_t = [0.0; 3];

    (*ent).s.pos.trType = TR_LINEAR;

    VectorCopy(&(*ent).r.currentOrigin, &mut oldOrg);
    // get current position
    BG_EvaluateTrajectory(&(*ent).s.pos, (*addr_of!(level)).time, &mut origin);
    //Get current angles?
    BG_EvaluateTrajectory(
        &(*ent).s.apos,
        (*addr_of!(level)).time,
        &mut (*ent).r.currentAngles,
    );

    //compensation test code..
    // `#ifdef THROWN_SABER_COMP` (defined at w_saber.c:5734) — block is live.
    if goingBack == QFALSE && (*ent).s.pos.trType != TR_GRAVITY {
        //acts as a fallback in case touch code fails, keeps saber from going through things between predictions
        let originalLength: f32;
        let iCompensationLength: c_int = 32;
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut calcComp: vec3_t = [0.0; 3];
        let mut compensatedOrigin: vec3_t = [0.0; 3];
        VectorSet(&mut mins, -24.0, -24.0, -8.0);
        VectorSet(&mut maxs, 24.0, 24.0, 8.0);

        VectorSubtract(&origin, &oldOrg, &mut calcComp);
        originalLength = VectorLength(&calcComp);

        VectorNormalize(&mut calcComp);

        compensatedOrigin[0] =
            oldOrg[0] + calcComp[0] * (originalLength + iCompensationLength as f32);
        compensatedOrigin[1] =
            oldOrg[1] + calcComp[1] * (originalLength + iCompensationLength as f32);
        compensatedOrigin[2] =
            oldOrg[2] + calcComp[2] * (originalLength + iCompensationLength as f32);

        let tr = trap::Trace(
            &oldOrg,
            &mins,
            &maxs,
            &compensatedOrigin,
            (*ent).r.ownerNum,
            MASK_PLAYERSOLID,
        );

        if (tr.fraction != 1.0 || tr.startsolid != 0 || tr.allsolid != 0)
            && tr.entityNum as c_int != (*ent).r.ownerNum
            && ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .r
            .contents
                & CONTENTS_LIGHTSABER)
                == 0
        {
            VectorClear(&mut (*ent).s.pos.trDelta);

            //Unfortunately doing this would defeat the purpose of the compensation. We will have to settle for a jerk on the client.
            //VectorCopy( origin, ent->r.currentOrigin );

            //we'll skip the dist check, since we don't really care about that (we just hit it physically)
            CheckThrownSaberDamaged(
                ent,
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*ent).r.ownerNum as usize),
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize),
                256,
                0,
                QTRUE,
            );

            if (*ent).s.pos.trType == TR_GRAVITY {
                //got blocked and knocked away in the damage func
                return;
            }

            let mut tr = tr;
            tr.startsolid = 0;
            if tr.entityNum as c_int == ENTITYNUM_NONE {
                //eh, this is a filthy lie. (obviously it had to hit something or it wouldn't be in here, so we'll say it hit the world)
                tr.entityNum = ENTITYNUM_WORLD as i16;
            }
            thrownSaberTouch(
                ent,
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize),
                &mut tr,
            );
            return;
        }
    }

    VectorCopy(&origin, &mut (*ent).r.currentOrigin);
}

/// `static GAME_INLINE qboolean CheckThrownSaberDamaged(gentity_t *saberent, gentity_t *saberOwner,
/// gentity_t *ent, int dist, int returning, qboolean noDCheck)` (w_saber.c:5469) — applies the
/// damage of a thrown/returning saber to a single candidate entity `ent`. Skips while the owner is
/// in attack-wound cooldown. The client branch range/PVS/duel-checks `ent`, traces to it
/// ([`trap::Trace`], NULL box → point trace via `vec3_origin`, per the precedent), and on a clear
/// line either lets the target block it ([`WP_SaberCanBlock`] → [`WP_SaberBlockNonRandom`] + a
/// `EV_SABER_BLOCK` temp-ent, possibly knocking the saber down via [`saberCheckKnockdown_Thrown`]) or
/// deals [`G_Damage`] and spawns a `EV_SABER_HIT` temp-ent. The non-client branch damages movers/
/// animents similarly (mover hits spawn a `EV_SABER_CLASHFLARE`). Either way, unless `returning`, it
/// auto-returns the saber via [`thrownSaberTouch`] and arms the owner's 500ms attack-wound timer.
///
/// No oracle: damage/temp-ent side effects over the opaque `gentity_t` driving
/// [`trap::InPVS`]/[`trap::Trace`]/[`G_Damage`]/[`G_TempEntity`], per the side-effect precedent.
///
/// # Safety
/// All pointers must be valid `gentity_t`; `saberOwner`/`ent` clients are dereferenced under the C
/// guards that gate them.
pub unsafe fn CheckThrownSaberDamaged(
    saberent: *mut gentity_t,
    saberOwner: *mut gentity_t,
    ent: *mut gentity_t,
    dist: c_int,
    returning: c_int,
    noDCheck: qboolean,
) -> qboolean {
    let mut vecsub: vec3_t = [0.0; 3];
    let veclen: f32;
    let te: *mut gentity_t;

    if !saberOwner.is_null()
        && !(*saberOwner).client.is_null()
        && (*(*saberOwner).client).ps.saberAttackWound > (*addr_of!(level)).time
    {
        return QFALSE;
    }

    if !ent.is_null()
        && !(*ent).client.is_null()
        && (*ent).inuse == QTRUE
        && (*ent).s.number != (*saberOwner).s.number
        && (*ent).health > 0
        && (*ent).takedamage != QFALSE
        && trap::InPVS(&(*(*ent).client).ps.origin, &(*saberent).r.currentOrigin) != QFALSE
        && (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR
        && ((*(*ent).client).pers.connected != 0 || (*ent).s.eType == ET_NPC)
    {
        //hit a client
        if (*ent).inuse == QTRUE
            && !(*ent).client.is_null()
            && (*(*ent).client).ps.duelInProgress != QFALSE
            && (*(*ent).client).ps.duelIndex != (*saberOwner).s.number
        {
            return QFALSE;
        }

        if (*ent).inuse == QTRUE
            && !(*ent).client.is_null()
            && (*(*saberOwner).client).ps.duelInProgress != QFALSE
            && (*(*saberOwner).client).ps.duelIndex != (*ent).s.number
        {
            return QFALSE;
        }

        VectorSubtract(
            &(*saberent).r.currentOrigin,
            &(*(*ent).client).ps.origin,
            &mut vecsub,
        );
        veclen = VectorLength(&vecsub);

        if veclen < dist as f32 {
            //within range
            let tr = trap::Trace(
                &(*saberent).r.currentOrigin,
                &vec3_origin,
                &vec3_origin,
                &(*(*ent).client).ps.origin,
                (*saberent).s.number,
                MASK_SHOT,
            );

            if tr.fraction == 1.0 || tr.entityNum as c_int == (*ent).s.number {
                //Slice them
                if (*(*saberOwner).client).ps.isJediMaster == QFALSE
                    && WP_SaberCanBlock(ent, &tr.endpos, 0, MOD_SABER, QFALSE, 999) != QFALSE
                {
                    //they blocked it
                    WP_SaberBlockNonRandom(ent, &tr.endpos, QFALSE);

                    te = G_TempEntity(&tr.endpos, EV_SABER_BLOCK);
                    VectorCopy(&tr.endpos, &mut (*te).s.origin);
                    VectorCopy(&tr.plane.normal, &mut (*te).s.angles);
                    if (*te).s.angles[0] == 0.0
                        && (*te).s.angles[1] == 0.0
                        && (*te).s.angles[2] == 0.0
                    {
                        (*te).s.angles[1] = 1.0;
                    }
                    (*te).s.eventParm = 1;
                    (*te).s.weapon = 0; //saberNum
                    (*te).s.legsAnim = 0; //bladeNum

                    if saberCheckKnockdown_Thrown(
                        saberent,
                        saberOwner,
                        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .add(tr.entityNum as usize),
                    ) != QFALSE
                    {
                        //it was knocked out of the air
                        return QFALSE;
                    }

                    if returning == 0 {
                        //return to owner if blocked
                        thrownSaberTouch(saberent, saberent, null_mut());
                    }

                    (*(*saberOwner).client).ps.saberAttackWound = (*addr_of!(level)).time + 500;
                    return QFALSE;
                } else {
                    //a good hit
                    let mut dir: vec3_t = [0.0; 3];
                    let mut dflags: c_int = 0;

                    VectorSubtract(&tr.endpos, &(*saberent).r.currentOrigin, &mut dir);
                    VectorNormalize(&mut dir);

                    if dir[0] == 0.0 && dir[1] == 0.0 && dir[2] == 0.0 {
                        dir[1] = 1.0;
                    }

                    if (*(*saberOwner).client).saber[0].saberFlags2 & SFL2_NO_DISMEMBERMENT != 0 {
                        dflags |= DAMAGE_NO_DISMEMBER;
                    }

                    if (*(*saberOwner).client).saber[0].knockbackScale > 0.0 {
                        dflags |= DAMAGE_SABER_KNOCKBACK1;
                    }

                    if (*(*saberOwner).client).ps.isJediMaster != QFALSE {
                        //2x damage for the Jedi Master
                        let mut endpos = tr.endpos;
                        G_Damage(
                            ent,
                            saberOwner,
                            saberOwner,
                            addr_of_mut!(dir),
                            addr_of_mut!(endpos),
                            (*saberent).damage * 2,
                            dflags,
                            MOD_SABER,
                        );
                    } else {
                        let mut endpos = tr.endpos;
                        G_Damage(
                            ent,
                            saberOwner,
                            saberOwner,
                            addr_of_mut!(dir),
                            addr_of_mut!(endpos),
                            (*saberent).damage,
                            dflags,
                            MOD_SABER,
                        );
                    }

                    te = G_TempEntity(&tr.endpos, EV_SABER_HIT);
                    (*te).s.otherEntityNum = (*ent).s.number;
                    (*te).s.otherEntityNum2 = (*saberOwner).s.number;
                    (*te).s.weapon = 0; //saberNum
                    (*te).s.legsAnim = 0; //bladeNum
                    VectorCopy(&tr.endpos, &mut (*te).s.origin);
                    VectorCopy(&tr.plane.normal, &mut (*te).s.angles);
                    if (*te).s.angles[0] == 0.0
                        && (*te).s.angles[1] == 0.0
                        && (*te).s.angles[2] == 0.0
                    {
                        (*te).s.angles[1] = 1.0;
                    }

                    (*te).s.eventParm = 1;

                    if returning == 0 {
                        //return to owner if blocked
                        thrownSaberTouch(saberent, saberent, null_mut());
                    }
                }

                (*(*saberOwner).client).ps.saberAttackWound = (*addr_of!(level)).time + 500;
            }
        }
    } else if !ent.is_null()
        && (*ent).client.is_null()
        && (*ent).inuse == QTRUE
        && (*ent).takedamage != QFALSE
        && (*ent).health > 0
        && (*ent).s.number != (*saberOwner).s.number
        && (*ent).s.number != (*saberent).s.number
        && (noDCheck != QFALSE
            || trap::InPVS(&(*ent).r.currentOrigin, &(*saberent).r.currentOrigin) != QFALSE)
    {
        //hit a non-client

        let veclen: f32 = if noDCheck != QFALSE {
            0.0
        } else {
            VectorSubtract(
                &(*saberent).r.currentOrigin,
                &(*ent).r.currentOrigin,
                &mut vecsub,
            );
            VectorLength(&vecsub)
        };

        if veclen < dist as f32 {
            let mut entOrigin: vec3_t = [0.0; 3];

            if (*ent).s.eType == ET_MOVER {
                VectorSubtract(&(*ent).r.absmax, &(*ent).r.absmin, &mut entOrigin);
                let entOrigin_copy = entOrigin;
                VectorMA(&(*ent).r.absmin, 0.5, &entOrigin_copy, &mut entOrigin);
                VectorAdd(&(*ent).r.absmin, &(*ent).r.absmax, &mut entOrigin);
                let entOrigin_copy = entOrigin;
                VectorScale(&entOrigin_copy, 0.5, &mut entOrigin);
            } else {
                VectorCopy(&(*ent).r.currentOrigin, &mut entOrigin);
            }

            let tr = trap::Trace(
                &(*saberent).r.currentOrigin,
                &vec3_origin,
                &vec3_origin,
                &entOrigin,
                (*saberent).s.number,
                MASK_SHOT,
            );

            if tr.fraction == 1.0 || tr.entityNum as c_int == (*ent).s.number {
                let mut dir: vec3_t = [0.0; 3];
                let mut dflags: c_int = 0;

                VectorSubtract(&tr.endpos, &entOrigin, &mut dir);
                VectorNormalize(&mut dir);

                if (*(*saberOwner).client).saber[0].saberFlags2 & SFL2_NO_DISMEMBERMENT != 0 {
                    dflags |= DAMAGE_NO_DISMEMBER;
                }
                if (*(*saberOwner).client).saber[0].knockbackScale > 0.0 {
                    dflags |= DAMAGE_SABER_KNOCKBACK1;
                }

                let mut endpos = tr.endpos;
                if (*ent).s.eType == ET_NPC {
                    //an animent
                    G_Damage(
                        ent,
                        saberOwner,
                        saberOwner,
                        addr_of_mut!(dir),
                        addr_of_mut!(endpos),
                        40,
                        dflags,
                        MOD_SABER,
                    );
                } else {
                    G_Damage(
                        ent,
                        saberOwner,
                        saberOwner,
                        addr_of_mut!(dir),
                        addr_of_mut!(endpos),
                        5,
                        dflags,
                        MOD_SABER,
                    );
                }

                te = G_TempEntity(&tr.endpos, EV_SABER_HIT);
                (*te).s.otherEntityNum = ENTITYNUM_NONE; //don't do this for throw damage
                                                         //te->s.otherEntityNum = ent->s.number;
                (*te).s.otherEntityNum2 = (*saberOwner).s.number; //actually, do send this, though - for the overridden per-saber hit effects/sounds
                (*te).s.weapon = 0; //saberNum
                (*te).s.legsAnim = 0; //bladeNum
                VectorCopy(&tr.endpos, &mut (*te).s.origin);
                VectorCopy(&tr.plane.normal, &mut (*te).s.angles);
                if (*te).s.angles[0] == 0.0 && (*te).s.angles[1] == 0.0 && (*te).s.angles[2] == 0.0
                {
                    (*te).s.angles[1] = 1.0;
                }

                if (*ent).s.eType == ET_MOVER {
                    if !saberOwner.is_null()
                        && !(*saberOwner).client.is_null()
                        && (*(*saberOwner).client).saber[0].saberFlags2 & SFL2_NO_CLASH_FLARE != 0
                    {
                        //don't do clash flare - NOTE: assumes same is true for both sabers if using dual sabers!
                        G_FreeEntity(te); //kind of a waste, but...
                    } else {
                        //I suppose I could tie this into the saberblock event, but I'm tired of adding flags to that thing.
                        let teS = G_TempEntity(&(*te).s.origin, EV_SABER_CLASHFLARE);
                        VectorCopy(&(*te).s.origin, &mut (*teS).s.origin);

                        (*te).s.eventParm = 0;
                    }
                } else {
                    (*te).s.eventParm = 1;
                }

                if returning == 0 {
                    //return to owner if blocked
                    thrownSaberTouch(saberent, saberent, null_mut());
                }

                (*(*saberOwner).client).ps.saberAttackWound = (*addr_of!(level)).time + 500;
            }
        }
    }

    QTRUE
}

/// `static GAME_INLINE void saberCheckRadiusDamage(gentity_t *saberent, int returning)`
/// (w_saber.c:5698) — cheaply damages every entity within the thrown saber's slice radius (the
/// saber entity has no server g2 instance), by walking `g_entities[0..level.num_entities]` and
/// running [`CheckThrownSaberDamaged`] against each. Picks the smaller return radius when
/// `returning` (but not `2`); bails if the owner is missing or in attack-wound cooldown.
///
/// No oracle: a loop of damage side effects over `g_entities`, per the side-effect precedent.
///
/// # Safety
/// `saberent` must point to a valid `gentity_t` whose `r.ownerNum` indexes `g_entities`.
pub unsafe fn saberCheckRadiusDamage(saberent: *mut gentity_t, returning: c_int) {
    //we're going to cheat and damage players within the saber's radius, just for the sake of doing things more "efficiently" (and because the saber entity has no server g2 instance)
    let mut i: c_int = 0;
    let dist: c_int;
    let saberOwner: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
        .add((*saberent).r.ownerNum as usize);

    if returning != 0 && returning != 2 {
        dist = MIN_SABER_SLICE_RETURN_DISTANCE;
    } else {
        dist = MIN_SABER_SLICE_DISTANCE;
    }

    if saberOwner.is_null() || (*saberOwner).client.is_null() {
        return;
    }

    if (*(*saberOwner).client).ps.saberAttackWound > (*addr_of!(level)).time {
        return;
    }

    while i < (*addr_of!(level)).num_entities {
        let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        CheckThrownSaberDamaged(saberent, saberOwner, ent, dist, returning, QFALSE);

        i += 1;
    }
}

/// `void thrownSaberTouch (gentity_t *saberent, gentity_t *other, trace_t *trace)` (w_saber.c:6651)
/// — the `touch` callback for a thrown saber in flight. Ignores a touch on its own owner. Otherwise
/// it halts the saber's linear motion, sets the angular spin, snaps the trajectory base to the
/// current origin, and schedules [`saberBackToOwner`] for the next frame. If it touched another
/// player's saber blade it resolves to that blade's owning client, then runs
/// [`CheckThrownSaberDamaged`] at point-blank against the hit entity and clears `speed`.
///
/// No oracle: a live touch callback over the opaque `gentity_t` with damage side effects. A
/// `pub unsafe extern "C"` fn for the `gentity_t::touch` fn-pointer ABI (installed by
/// [`saberReactivate`]).
///
/// # Safety
/// `saberent` must point to a valid `gentity_t` whose `r.ownerNum` indexes `g_entities`; `other`
/// may be null. `trace` is unused.
pub unsafe extern "C" fn thrownSaberTouch(
    saberent: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    let mut hitEnt: *mut gentity_t = other;

    if !other.is_null() && (*other).s.number == (*saberent).r.ownerNum {
        return;
    }
    VectorClear(&mut (*saberent).s.pos.trDelta);
    (*saberent).s.pos.trTime = (*addr_of!(level)).time;

    (*saberent).s.apos.trType = TR_LINEAR;
    (*saberent).s.apos.trDelta[0] = 0.0;
    (*saberent).s.apos.trDelta[1] = 800.0;
    (*saberent).s.apos.trDelta[2] = 0.0;

    VectorCopy(&(*saberent).r.currentOrigin, &mut (*saberent).s.pos.trBase);

    (*saberent).think = Some(saberBackToOwner);
    (*saberent).nextthink = (*addr_of!(level)).time;

    if !other.is_null()
        && (*other).r.ownerNum < MAX_CLIENTS as c_int
        && ((*other).r.contents & CONTENTS_LIGHTSABER) != 0
        && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*other).r.ownerNum as usize))
        .client
        .is_null()
        && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*other).r.ownerNum as usize))
        .inuse
            == QTRUE
    {
        hitEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*other).r.ownerNum as usize);
    }

    //we'll skip the dist check, since we don't really care about that (we just hit it physically)
    CheckThrownSaberDamaged(
        saberent,
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*saberent).r.ownerNum as usize),
        hitEnt,
        256,
        0,
        QTRUE,
    );

    (*saberent).speed = 0.0;
}

/// `#define SABER_RETRIEVE_DELAY 3000` (w_saber.c:6081) — post-knockdown saber pickup delay (ms).
const SABER_RETRIEVE_DELAY: c_int = 3000; //3 seconds for now. This will leave you nice and open if you lose your saber.

/// `void saberBackToOwner(gentity_t *saberent)` (w_saber.c:6488) — the `think` that pulls a thrown
/// saber back to its owner. Frees/deadsabers itself if the owner is gone or spectating. If the owner
/// is dead or has no offense level, it reverts to the dormant-saber state (dead saber spawned,
/// `SVF_NOCLIENT`, model removed) and clears the owner's throw state. Otherwise it homes the saber:
/// recomputes the return direction and speed (rank 3 throwers get a faster pull), eases the speed
/// down as it nears the owner, plays the catch sound and finishes the return at ≤32u, or runs the
/// radius slice + [`saberMoveBack`] while still in flight.
///
/// No oracle: a live `think` over the opaque `gentity_t` driving [`MakeDeadSaber`]/[`saberMoveBack`]/
/// [`saberCheckRadiusDamage`]/[`G_Sound`]/[`WP_SaberRemoveG2Model`], per the side-effect precedent.
/// A `pub unsafe extern "C"` fn for the `gentity_t::think` fn-pointer ABI.
///
/// # Safety
/// `saberent` must point to a valid `gentity_t` whose `r.ownerNum` indexes `g_entities`.
pub unsafe extern "C" fn saberBackToOwner(saberent: *mut gentity_t) {
    let saberOwner: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
        .add((*saberent).r.ownerNum as usize);
    let mut dir: vec3_t = [0.0; 3];
    let ownerLen: f32;

    if (*saberent).r.ownerNum == ENTITYNUM_NONE {
        MakeDeadSaber(saberent);

        (*saberent).think = Some(G_FreeEntity);
        (*saberent).nextthink = (*addr_of!(level)).time;
        return;
    }

    if (*saberOwner).inuse != QTRUE
        || (*saberOwner).client.is_null()
        || (*(*saberOwner).client).sess.sessionTeam == TEAM_SPECTATOR
    {
        MakeDeadSaber(saberent);

        (*saberent).think = Some(G_FreeEntity);
        (*saberent).nextthink = (*addr_of!(level)).time;
        return;
    }

    if (*saberOwner).health < 1
        || (*(*saberOwner).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] == 0
    {
        //He's dead, just go back to our normal saber status
        (*saberent).touch = Some(SaberGotHit);
        (*saberent).think = Some(SaberUpdateSelf);
        (*saberent).genericValue5 = 0;
        (*saberent).nextthink = (*addr_of!(level)).time;

        if !(*saberOwner).client.is_null() && (*(*saberOwner).client).saber[0].soundOff != 0 {
            G_Sound(
                saberent,
                CHAN_AUTO,
                (*(*saberOwner).client).saber[0].soundOff,
            );
        }
        MakeDeadSaber(saberent);

        (*saberent).r.svFlags |= SVF_NOCLIENT as c_int;
        (*saberent).r.contents = CONTENTS_LIGHTSABER;
        SetSaberBoxSize(saberent);
        (*saberent).s.loopSound = 0;
        (*saberent).s.loopIsSoundset = QFALSE;
        WP_SaberRemoveG2Model(saberent);

        (*(*saberOwner).client).ps.saberInFlight = QFALSE;
        (*(*saberOwner).client).ps.saberEntityState = 0;
        (*(*saberOwner).client).ps.saberThrowDelay = (*addr_of!(level)).time + 500;
        (*(*saberOwner).client).ps.saberCanThrow = QFALSE;

        return;
    }

    //make sure this is set alright
    debug_assert!(
        (*(*saberOwner).client).ps.saberEntityNum == (*saberent).s.number
            || (*(*saberOwner).client).saberStoredIndex == (*saberent).s.number
    );
    (*(*saberOwner).client).ps.saberEntityNum = (*saberent).s.number;

    (*saberent).r.contents = CONTENTS_LIGHTSABER;

    VectorSubtract(&(*saberent).pos1, &(*saberent).r.currentOrigin, &mut dir);

    ownerLen = VectorLength(&dir);

    if (*saberent).speed < (*addr_of!(level)).time as f32 {
        let baseSpeed: f32;

        VectorNormalize(&mut dir);

        saberMoveBack(saberent, QTRUE);
        VectorCopy(&(*saberent).r.currentOrigin, &mut (*saberent).s.pos.trBase);

        if (*(*saberOwner).client).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] >= FORCE_LEVEL_3 {
            //allow players with high saber throw rank to control the return speed of the saber
            baseSpeed = 900.0;

            (*saberent).speed = (*addr_of!(level)).time as f32; // + 200;
        } else {
            baseSpeed = 700.0;
            (*saberent).speed = ((*addr_of!(level)).time + 50) as f32;
        }

        //Gradually slow down as it approaches, so it looks smoother coming into the hand.
        if ownerLen < 64.0 {
            VectorScale(&dir, baseSpeed - 200.0, &mut (*saberent).s.pos.trDelta);
        } else if ownerLen < 128.0 {
            VectorScale(&dir, baseSpeed - 150.0, &mut (*saberent).s.pos.trDelta);
        } else if ownerLen < 256.0 {
            VectorScale(&dir, baseSpeed - 100.0, &mut (*saberent).s.pos.trDelta);
        } else {
            VectorScale(&dir, baseSpeed, &mut (*saberent).s.pos.trDelta);
        }

        (*saberent).s.pos.trTime = (*addr_of!(level)).time;
    }

    /*
    if (ownerLen <= 512)
    {
        saberent->s.saberInFlight = qfalse;
        saberent->s.loopSound = saberHumSound;
        saberent->s.loopIsSoundset = qfalse;
    }
    */
    //I'm just doing this now. I don't really like the spin on the way back. And it does weird stuff with the new saber-knocked-away code.
    if (*(*saberOwner).client).ps.saberEntityNum == (*saberent).s.number {
        if (*(*saberOwner).client).saber[0].saberFlags & SFL_RETURN_DAMAGE == 0
            || (*(*saberOwner).client).ps.saberHolstered != 0
        {
            (*saberent).s.saberInFlight = QFALSE;
        }
        (*saberent).s.loopSound = (*(*saberOwner).client).saber[0].soundLoop;
        (*saberent).s.loopIsSoundset = QFALSE;

        if ownerLen <= 32.0 {
            G_Sound(
                saberent,
                CHAN_AUTO,
                G_SoundIndex("sound/weapons/saber/saber_catch.wav"),
            );

            (*(*saberOwner).client).ps.saberInFlight = QFALSE;
            (*(*saberOwner).client).ps.saberEntityState = 0;
            (*(*saberOwner).client).ps.saberCanThrow = QFALSE;
            (*(*saberOwner).client).ps.saberThrowDelay = (*addr_of!(level)).time + 300;

            (*saberent).touch = Some(SaberGotHit);

            (*saberent).think = Some(SaberUpdateSelf);
            (*saberent).genericValue5 = 0;
            (*saberent).nextthink = (*addr_of!(level)).time + 50;
            WP_SaberRemoveG2Model(saberent);

            return;
        }

        if (*saberent).s.saberInFlight == QFALSE {
            saberCheckRadiusDamage(saberent, 1);
        } else {
            saberCheckRadiusDamage(saberent, 2);
        }

        saberMoveBack(saberent, QTRUE);
    }

    (*saberent).nextthink = (*addr_of!(level)).time;
}

/// `#define MAX_LEAVE_TIME 20000` (w_saber.c:5908) — max time a knocked-down saber loiters on the
/// ground before forcibly returning to its owner.
const MAX_LEAVE_TIME: c_int = 20000;

/// `void DownedSaberThink(gentity_t *saberent)` (w_saber.c:5913) — per-frame think for a saber that
/// was knocked out of its owner's hand (installed by [`saberKnockDown`]). If the saber has no owner,
/// or the owner is gone/spectating/following, it spawns a [`MakeDeadSaber`] and frees itself. If the
/// owner has reclaimed it (or is dead, or has lost saber-offense force), it reactivates in the
/// owner's hand via [`saberReactivate`], swaps to the [`SaberGotHit`]/[`SaberUpdateSelf`]
/// touch/think pair, hides it ([`SVF_NOCLIENT`]), removes its ghoul2 model
/// ([`WP_SaberRemoveG2Model`]) if the owner is alive, and arms the throw delay. Otherwise, if the
/// owner is pressing attack past the knocked-down delay (or it's loitered past [`MAX_LEAVE_TIME`]),
/// it flies back to the owner via [`saberBackToOwner`] with the force-pull sound; failing all that it
/// just advances its physics via [`G_RunObject`].
///
/// No oracle: entity-state think over the opaque `gentity_t` driving [`MakeDeadSaber`]/
/// [`saberReactivate`]/[`G_RunObject`]/[`G_Sound`] and player-state side effects, per the side-effect
/// precedent. The `#ifdef _DEBUG` saber-index paranoia checks become [`debug_assert!`]s.
///
/// # Safety
/// `saberent` must point to a valid `gentity_t` whose `r.ownerNum` indexes `g_entities`.
pub unsafe extern "C" fn DownedSaberThink(saberent: *mut gentity_t) {
    let saberOwn: *mut gentity_t;
    let mut notDisowned: qboolean = QFALSE;
    let mut pullBack: qboolean = QFALSE;

    (*saberent).nextthink = (*addr_of!(level)).time;

    if (*saberent).r.ownerNum == ENTITYNUM_NONE as c_int {
        MakeDeadSaber(saberent);

        (*saberent).think = Some(G_FreeEntity);
        (*saberent).nextthink = (*addr_of!(level)).time;
        return;
    }

    saberOwn = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
        .add((*saberent).r.ownerNum as usize);

    if saberOwn.is_null()
        || (*saberOwn).inuse != QTRUE
        || (*saberOwn).client.is_null()
        || (*(*saberOwn).client).sess.sessionTeam == TEAM_SPECTATOR
        || ((*(*saberOwn).client).ps.pm_flags & PMF_FOLLOW) != 0
    {
        MakeDeadSaber(saberent);

        (*saberent).think = Some(G_FreeEntity);
        (*saberent).nextthink = (*addr_of!(level)).time;
        return;
    }

    if (*(*saberOwn).client).ps.saberEntityNum != 0 {
        if (*(*saberOwn).client).ps.saberEntityNum == (*saberent).s.number {
            //owner shouldn't have this set if we're thinking in here. Must've fallen off a cliff and instantly respawned or something.
            notDisowned = QTRUE;
        } else {
            //This should never happen, but just in case..
            debug_assert!(false, "ULTRA BAD THING");
            MakeDeadSaber(saberent);

            (*saberent).think = Some(G_FreeEntity);
            (*saberent).nextthink = (*addr_of!(level)).time;
            return;
        }
    }

    if notDisowned == QTRUE
        || (*saberOwn).health < 1
        || (*(*saberOwn).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] == 0
    {
        //He's dead, just go back to our normal saber status
        (*(*saberOwn).client).ps.saberEntityNum = (*(*saberOwn).client).saberStoredIndex;

        //MakeDeadSaber(saberent); //spawn a dead saber on top of where we are now. The "bodyqueue" method.
        //Actually this will get taken care of when the thrown saber func sees we're dead.

        // `#ifdef _DEBUG` (w_saber.c:5969) saber-index paranoia.
        debug_assert!(
            (*(*saberOwn).client).saberStoredIndex == (*saberent).s.number,
            "Bad saber index!!!"
        );

        saberReactivate(saberent, saberOwn);

        if (*saberOwn).health < 1 {
            (*(*saberOwn).client).ps.saberInFlight = QFALSE;
            MakeDeadSaber(saberent);
        }

        (*saberent).touch = Some(SaberGotHit);
        (*saberent).think = Some(SaberUpdateSelf);
        (*saberent).genericValue5 = 0;
        (*saberent).nextthink = (*addr_of!(level)).time;

        (*saberent).r.svFlags |= SVF_NOCLIENT as c_int;
        //saberent->r.contents = CONTENTS_LIGHTSABER;
        (*saberent).s.loopSound = 0;
        (*saberent).s.loopIsSoundset = QFALSE;

        if (*saberOwn).health > 0 {
            //only set this if he's alive. If dead we want to reflect the lack of saber on the corpse, as he died with his saber out.
            (*(*saberOwn).client).ps.saberInFlight = QFALSE;
            WP_SaberRemoveG2Model(saberent);
        }
        (*(*saberOwn).client).ps.saberEntityState = 0;
        (*(*saberOwn).client).ps.saberThrowDelay = (*addr_of!(level)).time + 500;
        (*(*saberOwn).client).ps.saberCanThrow = QFALSE;

        return;
    }

    if (*(*saberOwn).client).saberKnockedTime < (*addr_of!(level)).time
        && ((*(*saberOwn).client).pers.cmd.buttons & BUTTON_ATTACK) != 0
    {
        //He wants us back
        pullBack = QTRUE;
    } else if ((*addr_of!(level)).time - (*(*saberOwn).client).saberKnockedTime) > MAX_LEAVE_TIME {
        //Been sitting around for too long, go back no matter what he wants.
        pullBack = QTRUE;
    }

    if pullBack == QTRUE {
        //Get going back to the owner.
        (*(*saberOwn).client).ps.saberEntityNum = (*(*saberOwn).client).saberStoredIndex;

        // `#ifdef _DEBUG` (w_saber.c:6019) saber-index paranoia.
        debug_assert!(
            (*(*saberOwn).client).saberStoredIndex == (*saberent).s.number,
            "Bad saber index!!!"
        );
        saberReactivate(saberent, saberOwn);

        (*saberent).touch = Some(SaberGotHit);

        (*saberent).think = Some(saberBackToOwner);
        (*saberent).speed = 0.0;
        (*saberent).genericValue5 = 0;
        (*saberent).nextthink = (*addr_of!(level)).time;

        (*saberent).r.contents = CONTENTS_LIGHTSABER;

        G_Sound(
            saberOwn,
            CHAN_BODY,
            G_SoundIndex("sound/weapons/force/pull.wav"),
        );
        if (*(*saberOwn).client).saber[0].soundOn != 0 {
            G_Sound(saberent, CHAN_BODY, (*(*saberOwn).client).saber[0].soundOn);
        }
        if (*(*saberOwn).client).saber[1].soundOn != 0 {
            G_Sound(saberOwn, CHAN_BODY, (*(*saberOwn).client).saber[1].soundOn);
        }

        return;
    }

    G_RunObject(saberent);
    (*saberent).nextthink = (*addr_of!(level)).time;
}

/// `void saberKnockDown(gentity_t *saberent, gentity_t *saberOwner, gentity_t *other)`
/// (w_saber.c:6083) — tosses a saber out of the owner's control onto the ground: clears the owner's
/// `saberEntityNum`, arms the [`SABER_RETRIEVE_DELAY`] pickup timer, re-boxes the entity, gives it a
/// random gravity spin and the owner's ghoul2 model ([`WP_SaberAddG2Model`]), tags it an
/// `ET_MISSILE` with a negative bounce count, settles it via [`saberMoveBack`], and installs
/// [`SaberBounceSound`]/the [`DownedSaberThink`] sibling. If a *different* attacker knocked
/// it out of the air, it's flung in that attacker's facing direction. Plays the owner's saber-off
/// sounds.
///
/// No oracle: entity-state knockdown over the opaque `gentity_t` driving
/// [`WP_SaberAddG2Model`]/[`saberMoveBack`]/[`trap::LinkEntity`]/[`G_Sound`], per the side-effect
/// precedent.
///
/// # Safety
/// All three pointers must be valid `gentity_t`; `saberOwner`/`other` clients are dereferenced under
/// the C guards.
pub unsafe fn saberKnockDown(
    saberent: *mut gentity_t,
    saberOwner: *mut gentity_t,
    other: *mut gentity_t,
) {
    (*(*saberOwner).client).ps.saberEntityNum = 0; //still stored in client->saberStoredIndex
    (*(*saberOwner).client).saberKnockedTime = (*addr_of!(level)).time + SABER_RETRIEVE_DELAY;

    (*saberent).clipmask = MASK_SOLID;
    (*saberent).r.contents = CONTENTS_TRIGGER; //0;

    VectorSet(&mut (*saberent).r.mins, -3.0, -3.0, -1.5);
    VectorSet(&mut (*saberent).r.maxs, 3.0, 3.0, 1.5);

    (*saberent).s.apos.trType = TR_GRAVITY;
    (*saberent).s.apos.trDelta[0] = Q_irand(200, 800) as f32;
    (*saberent).s.apos.trDelta[1] = Q_irand(200, 800) as f32;
    (*saberent).s.apos.trDelta[2] = Q_irand(200, 800) as f32;
    (*saberent).s.apos.trTime = (*addr_of!(level)).time - 50;

    (*saberent).s.pos.trType = TR_GRAVITY;
    (*saberent).s.pos.trTime = (*addr_of!(level)).time - 50;
    (*saberent).flags |= FL_BOUNCE_HALF;

    WP_SaberAddG2Model(
        saberent,
        (*(*saberOwner).client).saber[0].model.as_ptr(),
        (*(*saberOwner).client).saber[0].skin,
    );

    (*saberent).s.modelGhoul2 = 1;
    (*saberent).s.g2radius = 20;

    (*saberent).s.eType = ET_MISSILE;
    (*saberent).s.weapon = WP_SABER;

    (*saberent).speed = ((*addr_of!(level)).time + 4000) as f32;

    (*saberent).bounceCount = -5; //8;

    saberMoveBack(saberent, QTRUE);
    (*saberent).s.pos.trType = TR_GRAVITY;

    (*saberent).s.loopSound = 0; //kill this in case it was spinning.
    (*saberent).s.loopIsSoundset = QFALSE;

    (*saberent).r.svFlags &= !(SVF_NOCLIENT as c_int); //make sure the client is getting updates on where it is and such.

    (*saberent).touch = Some(SaberBounceSound);
    (*saberent).think = Some(DownedSaberThink);
    (*saberent).nextthink = (*addr_of!(level)).time;

    if saberOwner != other {
        //if someone knocked it out of the air and it wasn't turned off, go in the direction they were facing.
        if (*other).inuse == QTRUE && !(*other).client.is_null() {
            let mut otherFwd: vec3_t = [0.0; 3];
            let deflectSpeed: f32 = 200.0;

            AngleVectors(
                &(*(*other).client).ps.viewangles,
                Some(&mut otherFwd),
                None,
                None,
            );

            (*saberent).s.pos.trDelta[0] = otherFwd[0] * deflectSpeed;
            (*saberent).s.pos.trDelta[1] = otherFwd[1] * deflectSpeed;
            (*saberent).s.pos.trDelta[2] = otherFwd[2] * deflectSpeed;
        }
    }

    trap::LinkEntity(saberent);

    if (*(*saberOwner).client).saber[0].soundOff != 0 {
        G_Sound(
            saberent,
            CHAN_BODY,
            (*(*saberOwner).client).saber[0].soundOff,
        );
    }

    if (*(*saberOwner).client).saber[1].soundOff != 0
        && (*(*saberOwner).client).saber[1].model[0] != 0
    {
        G_Sound(
            saberOwner,
            CHAN_BODY,
            (*(*saberOwner).client).saber[1].soundOff,
        );
    }
}

/// `qboolean saberKnockOutOfHand(gentity_t *saberent, gentity_t *saberOwner, vec3_t velocity)`
/// (w_saber.c:6184) — flings the owner's saber directly out of their hand in `velocity`'s direction.
/// Returns `qfalse` (no disarm) unless the saber is valid, not already gone, the owner's base pos was
/// updated within 50ms, they're not in a saber lock, and the saber is disarmable. On success it puts
/// the owner into saber-in-flight state, gives the entity its ghoul2 model, sets it up as a thrown
/// `MOD_SABER` projectile from the owner's last saber base, runs [`saberKnockDown`], then overrides
/// the knocked-away velocity.
///
/// No oracle: entity-state disarm over the opaque `gentity_t` driving
/// [`WP_SaberAddG2Model`]/[`G_SetOrigin`]/[`saberKnockDown`], per the side-effect precedent.
///
/// # Safety
/// `saberent`/`saberOwner` must be valid `gentity_t`; the owner's `client` is dereferenced under the
/// C guards.
pub unsafe fn saberKnockOutOfHand(
    saberent: *mut gentity_t,
    saberOwner: *mut gentity_t,
    velocity: &vec3_t,
) -> qboolean {
    if saberent.is_null()
        || saberOwner.is_null()
        || (*saberent).inuse != QTRUE
        || (*saberOwner).inuse != QTRUE
        || (*saberOwner).client.is_null()
    {
        return QFALSE;
    }

    if (*(*saberOwner).client).ps.saberEntityNum == 0 {
        //already gone
        return QFALSE;
    }

    if ((*addr_of!(level)).time - (*(*saberOwner).client).lastSaberStorageTime) > 50 {
        //must have a reasonably updated saber base pos
        return QFALSE;
    }

    if (*(*saberOwner).client).ps.saberLockTime > ((*addr_of!(level)).time - 100) {
        return QFALSE;
    }
    if (*(*saberOwner).client).saber[0].saberFlags & SFL_NOT_DISARMABLE != 0 {
        return QFALSE;
    }

    (*(*saberOwner).client).ps.saberInFlight = QTRUE;
    (*(*saberOwner).client).ps.saberEntityState = 1;

    (*saberent).s.saberInFlight = QFALSE; //qtrue;

    (*saberent).s.pos.trType = TR_LINEAR;
    (*saberent).s.eType = ET_GENERAL;
    (*saberent).s.eFlags = 0;

    WP_SaberAddG2Model(
        saberent,
        (*(*saberOwner).client).saber[0].model.as_ptr(),
        (*(*saberOwner).client).saber[0].skin,
    );

    (*saberent).s.modelGhoul2 = 127;

    (*saberent).parent = saberOwner;

    (*saberent).damage = SABER_THROWN_HIT_DAMAGE;
    (*saberent).methodOfDeath = MOD_SABER;
    (*saberent).splashMethodOfDeath = MOD_SABER;
    (*saberent).s.solid = 2;
    (*saberent).r.contents = CONTENTS_LIGHTSABER;

    (*saberent).genericValue5 = 0;

    VectorSet(&mut (*saberent).r.mins, -24.0, -24.0, -8.0);
    VectorSet(&mut (*saberent).r.maxs, 24.0, 24.0, 8.0);

    (*saberent).s.genericenemyindex = (*saberOwner).s.number + 1024;
    (*saberent).s.weapon = WP_SABER;

    (*saberent).genericValue5 = 0;

    G_SetOrigin(saberent, &(*(*saberOwner).client).lastSaberBase_Always); //use this as opposed to the right hand bolt,
                                                                          //because I don't want to risk reconstructing the skel again to get it here. And it isn't worth storing.
    saberKnockDown(saberent, saberOwner, saberOwner);
    VectorCopy(velocity, &mut (*saberent).s.pos.trDelta); //override the velocity on the knocked away saber.

    QTRUE
}

/// `#define SABERINVALID (...)` (w_saber.c:6158) — "sort of a silly macro I guess. But if I change
/// anything in here I'll probably want it to be everywhere." The shared guard for the
/// `saberCheckKnockdown_*` family: the saber/owner/other must all be valid in-use entities, both the
/// owner and other must be clients, the owner must currently own a saber entity, and the owner must
/// not be in a recent saber lock. Returns `true` (= "invalid, bail") when any of these fail, exactly
/// matching the C macro's short-circuit OR chain.
///
/// # Safety
/// Dereferences all three pointers and their clients; the `||` short-circuits guard the deeper
/// derefs exactly as in C.
#[inline]
unsafe fn SABERINVALID(
    saberent: *mut gentity_t,
    saberOwner: *mut gentity_t,
    other: *mut gentity_t,
) -> bool {
    saberent.is_null()
        || saberOwner.is_null()
        || other.is_null()
        || (*saberent).inuse == QFALSE
        || (*saberOwner).inuse == QFALSE
        || (*other).inuse == QFALSE
        || (*saberOwner).client.is_null()
        || (*other).client.is_null()
        || (*(*saberOwner).client).ps.saberEntityNum == 0
        || (*(*saberOwner).client).ps.saberLockTime > ((*addr_of!(level)).time - 100)
}

/// `qboolean saberCheckKnockdown_DuelLoss(gentity_t *saberent, gentity_t *saberOwner, gentity_t
/// *other)` (w_saber.c:6252) — result of losing a circle-lock duel: the loser's saber is tossed away
/// and they're put into a reflected attack anim. Derives a throw direction from the recent saber-base
/// momentum (the other player's, the owner's, or the gap between blades), puts the loser into
/// `LS_V1_BL`/`BLOCKED_BOUNCE_MOVE`, and (on a `disarmChance` roll) disarms via
/// [`saberKnockOutOfHand`]. The `SABERINVALID` macro guard is inlined.
///
/// No oracle: entity-state disarm over the opaque `gentity_t`, per the side-effect precedent.
///
/// The C `other->client->saber[1].disarmBonus;` is a no-op expression statement in the original
/// (the result is discarded); faithfully preserved as such.
///
/// # Safety
/// All three pointers must be valid `gentity_t`; clients are dereferenced under the `SABERINVALID`
/// guard.
// `totalDistance` is initialized to `1` in the C and only read inside the `validMomentum` branch
// after being reassigned; Rust flags the init as never-read, but it is kept verbatim for fidelity.
#[allow(unused_assignments)]
pub unsafe fn saberCheckKnockdown_DuelLoss(
    saberent: *mut gentity_t,
    saberOwner: *mut gentity_t,
    other: *mut gentity_t,
) -> qboolean {
    let mut dif: vec3_t = [0.0; 3];
    let mut totalDistance: f32 = 1.0;
    let distScale: f32 = 6.5;
    let mut validMomentum: qboolean = QTRUE;
    let mut disarmChance: c_int = 1;

    if SABERINVALID(saberent, saberOwner, other) {
        return QFALSE;
    }

    VectorClear(&mut dif);

    if (*(*other).client).olderIsValid == QFALSE
        || ((*addr_of!(level)).time - (*(*other).client).lastSaberStorageTime) >= 200
    {
        //see if the spots are valid
        validMomentum = QFALSE;
    }

    if validMomentum != QFALSE {
        //Get the difference
        VectorSubtract(
            &(*(*other).client).lastSaberBase_Always,
            &(*(*other).client).olderSaberBase,
            &mut dif,
        );
        totalDistance = VectorNormalize(&mut dif);

        if totalDistance == 0.0 {
            //fine, try our own
            if (*(*saberOwner).client).olderIsValid == QFALSE
                || ((*addr_of!(level)).time - (*(*saberOwner).client).lastSaberStorageTime) >= 200
            {
                validMomentum = QFALSE;
            }

            if validMomentum != QFALSE {
                VectorSubtract(
                    &(*(*saberOwner).client).lastSaberBase_Always,
                    &(*(*saberOwner).client).olderSaberBase,
                    &mut dif,
                );
                totalDistance = VectorNormalize(&mut dif);
            }
        }

        if validMomentum != QFALSE {
            if totalDistance == 0.0 {
                //try the difference between the two blades
                VectorSubtract(
                    &(*(*saberOwner).client).lastSaberBase_Always,
                    &(*(*other).client).lastSaberBase_Always,
                    &mut dif,
                );
                totalDistance = VectorNormalize(&mut dif);
            }

            if totalDistance != 0.0 {
                //if we still have no difference somehow, just let it fall to the ground when the time comes.
                if totalDistance < 20.0 {
                    totalDistance = 20.0;
                }
                let dif_copy = dif;
                VectorScale(&dif_copy, totalDistance * distScale, &mut dif);
            }
        }
    }

    (*(*saberOwner).client).ps.saberMove = LS_V1_BL; //rwwFIXMEFIXME: Ideally check which lock it was exactly and use the proper anim (same goes for the attacker)
    (*(*saberOwner).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;

    if !other.is_null() && !(*other).client.is_null() {
        disarmChance += (*(*other).client).saber[0].disarmBonus;
        if (*(*other).client).saber[1].model[0] != 0 && (*(*other).client).ps.saberHolstered == 0 {
            let _ = (*(*other).client).saber[1].disarmBonus;
        }
    }
    if Q_irand(0, disarmChance) != 0 {
        saberKnockOutOfHand(saberent, saberOwner, &dif)
    } else {
        QFALSE
    }
}

/// `qboolean saberCheckKnockdown_BrokenParry(gentity_t *saberent, gentity_t *saberOwner, gentity_t
/// *other)` (w_saber.c:6336) — tries to knock the saber out of the owner's hand when they go into a
/// broken parry (also called on reflected attacks). Compares both fighters' [`G_SaberAttackPower`]
/// (neither gets a state-based advantage here) and only disarms when the attacker is in a stronger
/// stance, on a probability roll. The throw direction comes from recent saber-base momentum; the
/// disarm itself goes through [`saberKnockOutOfHand`]. The `SABERINVALID` macro guard is inlined.
///
/// No oracle: entity-state disarm over the opaque `gentity_t`, per the side-effect precedent.
///
/// The C `other->client->saber[1].disarmBonus;` no-op expression statement is faithfully preserved.
///
/// # Safety
/// All three pointers must be valid `gentity_t`; clients are dereferenced under the `SABERINVALID`
/// guard.
pub unsafe fn saberCheckKnockdown_BrokenParry(
    saberent: *mut gentity_t,
    saberOwner: *mut gentity_t,
    other: *mut gentity_t,
) -> qboolean {
    let myAttack: c_int;
    let otherAttack: c_int;
    let mut doKnock: qboolean = QFALSE;
    let mut disarmChance: c_int = 1;

    if SABERINVALID(saberent, saberOwner, other) {
        return QFALSE;
    }

    //Neither gets an advantage based on attack state, when it comes to knocking
    //saber out of hand.
    myAttack = G_SaberAttackPower(saberOwner, QFALSE);
    otherAttack = G_SaberAttackPower(other, QFALSE);

    if (*(*other).client).olderIsValid == QFALSE
        || ((*addr_of!(level)).time - (*(*other).client).lastSaberStorageTime) >= 200
    {
        //if we don't know which way to throw the saber based on momentum between saber positions, just don't throw it
        return QFALSE;
    }

    //only knock the saber out of the hand if they're in a stronger stance I suppose. Makes strong more advantageous.
    if otherAttack > myAttack + 1 && Q_irand(1, 10) <= 7 {
        //This would be, say, strong stance against light stance.
        doKnock = QTRUE;
    } else if otherAttack > myAttack && Q_irand(1, 10) <= 3 {
        //Strong vs. medium, medium vs. light
        doKnock = QTRUE;
    }

    if doKnock != QFALSE {
        let mut dif: vec3_t = [0.0; 3];
        let mut totalDistance: f32;
        let distScale: f32 = 6.5;

        VectorSubtract(
            &(*(*other).client).lastSaberBase_Always,
            &(*(*other).client).olderSaberBase,
            &mut dif,
        );
        totalDistance = VectorNormalize(&mut dif);

        if totalDistance == 0.0 {
            //fine, try our own
            if (*(*saberOwner).client).olderIsValid == QFALSE
                || ((*addr_of!(level)).time - (*(*saberOwner).client).lastSaberStorageTime) >= 200
            {
                //if we don't know which way to throw the saber based on momentum between saber positions, just don't throw it
                return QFALSE;
            }

            VectorSubtract(
                &(*(*saberOwner).client).lastSaberBase_Always,
                &(*(*saberOwner).client).olderSaberBase,
                &mut dif,
            );
            totalDistance = VectorNormalize(&mut dif);
        }

        if totalDistance == 0.0 {
            //...forget it then.
            return QFALSE;
        }

        if totalDistance < 20.0 {
            totalDistance = 20.0;
        }
        let dif_copy = dif;
        VectorScale(&dif_copy, totalDistance * distScale, &mut dif);

        if !other.is_null() && !(*other).client.is_null() {
            disarmChance += (*(*other).client).saber[0].disarmBonus;
            if (*(*other).client).saber[1].model[0] != 0
                && (*(*other).client).ps.saberHolstered == 0
            {
                let _ = (*(*other).client).saber[1].disarmBonus;
            }
        }
        if Q_irand(0, disarmChance) != 0 {
            return saberKnockOutOfHand(saberent, saberOwner, &dif);
        }
    }

    QFALSE
}

/// `qboolean saberCheckKnockdown_Smashed(gentity_t *saberent, gentity_t *saberOwner, gentity_t
/// *other, int damage)` (w_saber.c:6423) — called when an enemy actually slashes into a thrown
/// saber. Only knocks it down if it's genuinely in flight, and then either when the attacker is in an
/// extra-defense saber move ([`BG_InExtraDefenseSaberMove`]) or the blow did >10 damage — both via
/// [`saberKnockDown`]. The `SABERINVALID` macro guard is inlined.
///
/// No oracle: entity-state knockdown over the opaque `gentity_t`, per the side-effect precedent.
///
/// # Safety
/// All three pointers must be valid `gentity_t`; clients are dereferenced under the `SABERINVALID`
/// guard.
pub unsafe fn saberCheckKnockdown_Smashed(
    saberent: *mut gentity_t,
    saberOwner: *mut gentity_t,
    other: *mut gentity_t,
    damage: c_int,
) -> qboolean {
    if SABERINVALID(saberent, saberOwner, other) {
        return QFALSE;
    }

    if (*(*saberOwner).client).ps.saberInFlight == QFALSE {
        //can only do this if the saber is already actually in flight
        return QFALSE;
    }

    if !other.is_null()
        && (*other).inuse == QTRUE
        && !(*other).client.is_null()
        && BG_InExtraDefenseSaberMove((*(*other).client).ps.saberMove) != QFALSE
    {
        //make sure the blow was strong enough
        saberKnockDown(saberent, saberOwner, other);
        return QTRUE;
    }

    if damage > 10 {
        //make sure the blow was strong enough
        saberKnockDown(saberent, saberOwner, other);
        return QTRUE;
    }

    QFALSE
}

/// `qboolean saberCheckKnockdown_Thrown(gentity_t *saberent, gentity_t *saberOwner, gentity_t
/// *other)` (w_saber.c:6455) — called upon blocking a thrown saber. If the blocker's defense level
/// exceeds the thrower's throw level (or they're equal and a 4-in-10 roll hits), the saber is tossed
/// to the ground via [`saberKnockDown`]. The `SABERINVALID` macro guard is inlined.
///
/// No oracle: entity-state knockdown over the opaque `gentity_t`, per the side-effect precedent.
///
/// # Safety
/// All three pointers must be valid `gentity_t`; clients are dereferenced under the `SABERINVALID`
/// guard.
pub unsafe fn saberCheckKnockdown_Thrown(
    saberent: *mut gentity_t,
    saberOwner: *mut gentity_t,
    other: *mut gentity_t,
) -> qboolean {
    let throwLevel: c_int;
    let defenLevel: c_int;
    let mut tossIt: qboolean = QFALSE;

    if SABERINVALID(saberent, saberOwner, other) {
        return QFALSE;
    }

    defenLevel = (*(*other).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize];
    throwLevel = (*(*saberOwner).client).ps.fd.forcePowerLevel[FP_SABERTHROW as usize];

    if defenLevel > throwLevel {
        tossIt = QTRUE;
    } else if defenLevel == throwLevel && Q_irand(1, 10) <= 4 {
        tossIt = QTRUE;
    }
    //otherwise don't

    if tossIt != QFALSE {
        saberKnockDown(saberent, saberOwner, other);
        return QTRUE;
    }

    QFALSE
}

/// `#define PROPER_THROWN_VALUE 999` (w_saber.c:284) — the sentinel stored in a flying saber
const PROPER_THROWN_VALUE: c_int = 999;

/// `void SaberUpdateSelf(gentity_t *ent)` (w_saber.c:286) — the per-frame `think` for a saber
/// entity. Frees itself if it has no owner or its owner is no longer a valid client; if the
/// owner is throwing the saber (and alive), it hands control to the missile path (marks
/// `genericValue5 = PROPER_THROWN_VALUE`) and bails. Otherwise it clears the marker and decides
/// the saber's collision state: solid ([`CONTENTS_LIGHTSABER`]) when the owner is actively
/// wielding it, or non-solid (`contents = 0`, `clipmask = 0`) when the owner isn't using the
/// saber, is spectating/dead, has it holstered, or has no attack level. It then relinks the
/// entity and reschedules itself for the next frame.
///
/// No oracle: a live saber-entity `think` that mutates `r.contents`/`clipmask`/`nextthink`,
/// frees the entity, and calls [`trap_LinkEntity`](crate::trap::LinkEntity).
///
/// The `#ifdef DEBUG_SABER_BOX` block (a `g_saberDebugBox`-gated [`G_DebugBoxLines`] call) is
/// **omitted**, matching the retail build and how the rest of the codebase drops the
/// `DEBUG_SABER_BOX`-only `g_saberDebugBox` debug paths (see `g_main.rs`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` whose `r.ownerNum` indexes `g_entities`.
pub unsafe extern "C" fn SaberUpdateSelf(ent: *mut gentity_t) {
    if (*ent).r.ownerNum == ENTITYNUM_NONE {
        (*ent).think = Some(G_FreeEntity);
        (*ent).nextthink = (*addr_of!(level)).time;
        return;
    }

    let owner =
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize);

    if (*owner).inuse != QTRUE || (*owner).client.is_null()
    /* ||
    g_entities[ent->r.ownerNum].client->sess.sessionTeam == TEAM_SPECTATOR*/
    {
        (*ent).think = Some(G_FreeEntity);
        (*ent).nextthink = (*addr_of!(level)).time;
        return;
    }

    if (*(*owner).client).ps.saberInFlight == QTRUE && (*owner).health > 0 {
        //let The Master take care of us now (we'll get treated like a missile until we return)
        (*ent).nextthink = (*addr_of!(level)).time;
        (*ent).genericValue5 = PROPER_THROWN_VALUE;
        return;
    }

    (*ent).genericValue5 = 0;

    if (*(*owner).client).ps.weapon != WP_SABER
        || ((*(*owner).client).ps.pm_flags & PMF_FOLLOW) != 0
        //RWW ADDED 7-19-03 BEGIN
        || (*(*owner).client).sess.sessionTeam == TEAM_SPECTATOR
        || (*(*owner).client).tempSpectate >= (*addr_of!(level)).time
        //RWW ADDED 7-19-03 END
        || (*owner).health < 1
        || BG_SabersOff(&mut (*(*owner).client).ps) == QTRUE
        || ((*(*owner).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] == 0
            && (*owner).s.eType != ET_NPC)
    {
        //owner is not using saber, spectating, dead, saber holstered, or has no attack level
        (*ent).r.contents = 0;
        (*ent).clipmask = 0;
    } else {
        //Standard contents (saber is active)
        // `#ifdef DEBUG_SABER_BOX` block (`g_saberDebugBox`-gated `G_DebugBoxLines` call) is
        // omitted — `DEBUG_SABER_BOX` is not defined in the retail build.
        if (*ent).r.contents != CONTENTS_LIGHTSABER {
            if ((*addr_of!(level)).time - (*(*owner).client).lastSaberStorageTime) <= 200 {
                //Only go back to solid once we're sure our owner has updated recently
                (*ent).r.contents = CONTENTS_LIGHTSABER;
                (*ent).clipmask = MASK_PLAYERSOLID | CONTENTS_LIGHTSABER;
            }
        } else {
            (*ent).r.contents = CONTENTS_LIGHTSABER;
            (*ent).clipmask = MASK_PLAYERSOLID | CONTENTS_LIGHTSABER;
        }
    }

    trap::LinkEntity(ent);

    (*ent).nextthink = (*addr_of!(level)).time;
}

/// `void SaberGotHit( gentity_t *self, gentity_t *other, trace_t *trace )` (w_saber.c:360) — the
/// `touch` callback installed on a thrown saber entity. It bails unless the saber's owner
/// (`g_entities[self->r.ownerNum]`) is a valid client; the original then does nothing else (the
/// comment notes projectile handling moved into the projectiles' own touch fns). `other` and
/// `trace` are unused. A `pub unsafe extern "C"` fn for the `gentity_t::touch` fn-pointer ABI.
///
/// The C `if (!own || !own->client)` keeps the `!own` test even though `own = &g_entities[...]`
/// can never be null; faithfully preserved here as `own.is_null()` (always false) so the source
/// reads 1:1, the `client` null-check carrying the real guard.
///
/// # Safety
/// `self_` must point to a valid `gentity_t` whose `r.ownerNum` indexes `g_entities`.
pub unsafe extern "C" fn SaberGotHit(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    let own =
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*self_).r.ownerNum as usize);

    if own.is_null() || (*own).client.is_null() {
        return;
    }

    //Do something here..? Was handling projectiles here, but instead they're now handled in their own functions.
}

/// `#define SABER_BOX_SIZE 16.0f` (w_saber.c:13) — the default half-extent for a saber
/// entity's collision box when no reliable blade-point storage is available.
const SABER_BOX_SIZE: f32 = 16.0;

/// `static GAME_INLINE void SetSaberBoxSize(gentity_t *saberent)` (w_saber.c:376) — recomputes
/// the saber entity's `r.mins`/`r.maxs` collision box from the owner's saber blade muzzle
/// points and tips. Returns early (with a zero box, a default ±[`SABER_BOX_SIZE`] box, or no
/// change) for the broken-parry/super-break-lose state, stale point storage, a missing owner, or
/// a fully holstered saber.
///
/// PC source of truth: the broken-parry/super-break-lose block now computes a per-saber/per-blade
/// `alwaysBlock`/`forceBlock` mask (from `SFL2_ALWAYS_BLOCK`/`SFL2_ALWAYS_BLOCK2` +
/// `bladeStyle2Start`) and only zeroes the box when nothing is forced on; a `dualSabers` flag and
/// per-blade holster early-outs replace the Xbox single-saber-only path.
///
/// Oracle-tested via the pass-the-read-fields precedent: the box math is extracted as a pure
/// function over the owner/blade fields it reads (the two state predicates collapsed into one
/// `inBrokenParryOrLose` flag), and the resulting `r.mins`/`r.maxs` are compared bit-exact.
/// The `#ifndef FINAL_BUILD` [`Com_Printf`] debug block (gated on `g_saberDebugPrint > 1`) is
/// present in this non-final build. The C `assert`s become `debug_assert!` (no-ops in release,
/// exactly as `assert` under `NDEBUG`).
///
/// # Safety
/// `saberent` must point to a valid, in-use `gentity_t` whose `r.ownerNum` indexes `g_entities`.
// `allow(dead_code)`: the only caller (WP_SaberInitBladeData, w_saber.c:476) is not yet ported.
#[allow(dead_code)]
unsafe fn SetSaberBoxSize(saberent: *mut gentity_t) {
    let mut owner: *mut gentity_t = null_mut();
    let mut saberOrg: vec3_t = [0.0; 3];
    let mut saberTip: vec3_t = [0.0; 3];
    let mut i: c_int;
    let mut j: c_int = 0;
    let mut k: c_int = 0;
    let mut dualSabers = QFALSE;
    let mut alwaysBlock: [[qboolean; MAX_BLADES]; MAX_SABERS] = [[QFALSE; MAX_BLADES]; MAX_SABERS];
    let mut forceBlock = QFALSE;

    debug_assert!(!saberent.is_null() && (*saberent).inuse == QTRUE);

    if (*saberent).r.ownerNum < MAX_CLIENTS as c_int && (*saberent).r.ownerNum >= 0 {
        owner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*saberent).r.ownerNum as usize);
    } else if (*saberent).r.ownerNum >= 0
        && (*saberent).r.ownerNum < ENTITYNUM_WORLD
        && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*saberent).r.ownerNum as usize))
        .s
        .eType
            == ET_NPC
    {
        owner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*saberent).r.ownerNum as usize);
    }

    if owner.is_null() || (*owner).inuse != QTRUE || (*owner).client.is_null() {
        debug_assert!(false, "Saber with no owner?");
        return;
    }

    // C: `saber[1].model && saber[1].model[0]` — `model` is a fixed array so the first half is
    // always non-null; the byte test is the real predicate.
    if (*(*owner).client).saber[1].model[0] != 0 {
        dualSabers = QTRUE;
    }

    if PM_SaberInBrokenParry((*(*owner).client).ps.saberMove) == QTRUE
        || BG_SuperBreakLoseAnim((*(*owner).client).ps.torsoAnim) == QTRUE
    {
        //let swings go right through when we're in this state
        i = 0;
        while i < MAX_SABERS as c_int {
            if i > 0 && dualSabers != QTRUE {
                //not using a second saber, set it to not blocking
                j = 0;
                while j < MAX_BLADES as c_int {
                    alwaysBlock[i as usize][j as usize] = QFALSE;
                    j += 1;
                }
            } else {
                if ((*(*owner).client).saber[i as usize].saberFlags2 & SFL2_ALWAYS_BLOCK) != 0 {
                    j = 0;
                    while j < (*(*owner).client).saber[i as usize].numBlades {
                        alwaysBlock[i as usize][j as usize] = QTRUE;
                        forceBlock = QTRUE;
                        j += 1;
                    }
                }
                if (*(*owner).client).saber[i as usize].bladeStyle2Start > 0 {
                    j = (*(*owner).client).saber[i as usize].bladeStyle2Start;
                    while j < (*(*owner).client).saber[i as usize].numBlades {
                        if ((*(*owner).client).saber[i as usize].saberFlags2 & SFL2_ALWAYS_BLOCK2)
                            != 0
                        {
                            alwaysBlock[i as usize][j as usize] = QTRUE;
                            forceBlock = QTRUE;
                        } else {
                            alwaysBlock[i as usize][j as usize] = QFALSE;
                        }
                        j += 1;
                    }
                }
            }
            i += 1;
        }
        if forceBlock != QTRUE {
            //no sabers/blades to FORCE to be on, so turn off blocking altogether
            VectorSet(&mut (*saberent).r.mins, 0.0, 0.0, 0.0);
            VectorSet(&mut (*saberent).r.maxs, 0.0, 0.0, 0.0);
            // `#ifndef FINAL_BUILD` — present in this non-final build.
            if (*addr_of!(g_saberDebugPrint)).integer > 1 {
                Com_Printf(&format!(
                    "Client {} in broken parry, saber box 0\n",
                    (*owner).s.number
                ));
            }
            return;
        }
    }

    if ((*addr_of!(level)).time - (*(*owner).client).lastSaberStorageTime) > 200
        || ((*addr_of!(level)).time
            - (*(*owner).client).saber[j as usize].blade[k as usize].storageTime)
            > 100
    {
        //it's been too long since we got a reliable point storage, so use the defaults and leave.
        VectorSet(
            &mut (*saberent).r.mins,
            -SABER_BOX_SIZE,
            -SABER_BOX_SIZE,
            -SABER_BOX_SIZE,
        );
        VectorSet(
            &mut (*saberent).r.maxs,
            SABER_BOX_SIZE,
            SABER_BOX_SIZE,
            SABER_BOX_SIZE,
        );
        return;
    }

    if dualSabers == QTRUE || (*(*owner).client).saber[0].numBlades > 1 {
        //dual sabers or multi-blade saber
        if (*(*owner).client).ps.saberHolstered > 1 {
            //entirely off
            //no blocking at all
            VectorSet(&mut (*saberent).r.mins, 0.0, 0.0, 0.0);
            VectorSet(&mut (*saberent).r.maxs, 0.0, 0.0, 0.0);
            return;
        }
    } else {
        //single saber
        if (*(*owner).client).ps.saberHolstered != 0 {
            //off
            //no blocking at all
            VectorSet(&mut (*saberent).r.mins, 0.0, 0.0, 0.0);
            VectorSet(&mut (*saberent).r.maxs, 0.0, 0.0, 0.0);
            return;
        }
    }

    //Start out at the saber origin, then go through all the blades and push out the extents
    //for each blade, then set the box relative to the origin.
    let currentOrigin = (*saberent).r.currentOrigin;
    VectorCopy(&currentOrigin, &mut (*saberent).r.mins);
    VectorCopy(&currentOrigin, &mut (*saberent).r.maxs);

    i = 0;
    while i < 3 {
        j = 0;
        while j < MAX_SABERS as c_int {
            if (*(*owner).client).saber[j as usize].model[0] == 0 {
                break;
            }
            if dualSabers == QTRUE && (*(*owner).client).ps.saberHolstered == 1 && j == 1 {
                //this mother is holstered, get outta here.
                j += 1;
                continue;
            }
            k = 0;
            while k < (*(*owner).client).saber[j as usize].numBlades {
                if k > 0 {
                    //not the first blade
                    if dualSabers != QTRUE {
                        //using a single saber
                        if (*(*owner).client).saber[j as usize].numBlades > 1 {
                            //with multiple blades
                            if (*(*owner).client).ps.saberHolstered == 1 {
                                //all blades after the first one are off
                                break;
                            }
                        }
                    }
                }
                if forceBlock == QTRUE {
                    //only do blocking with blades that are marked to block
                    if alwaysBlock[j as usize][k as usize] != QTRUE {
                        //this blade shouldn't be blocking
                        k += 1;
                        continue;
                    }
                }
                //VectorMA(owner->client->saber[j].blade[k].muzzlePoint, owner->client->saber[j].blade[k].lengthMax*0.5f, owner->client->saber[j].blade[k].muzzleDir, saberOrg);
                let blade = &(*(*owner).client).saber[j as usize].blade[k as usize];
                VectorCopy(&blade.muzzlePoint, &mut saberOrg);
                VectorMA(
                    &blade.muzzlePoint,
                    blade.lengthMax,
                    &blade.muzzleDir,
                    &mut saberTip,
                );

                if saberOrg[i as usize] < (*saberent).r.mins[i as usize] {
                    (*saberent).r.mins[i as usize] = saberOrg[i as usize];
                }
                if saberTip[i as usize] < (*saberent).r.mins[i as usize] {
                    (*saberent).r.mins[i as usize] = saberTip[i as usize];
                }

                if saberOrg[i as usize] > (*saberent).r.maxs[i as usize] {
                    (*saberent).r.maxs[i as usize] = saberOrg[i as usize];
                }
                if saberTip[i as usize] > (*saberent).r.maxs[i as usize] {
                    (*saberent).r.maxs[i as usize] = saberTip[i as usize];
                }

                //G_TestLine(saberOrg, saberTip, 0x0000ff, 50);
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }

    let currentOrigin = (*saberent).r.currentOrigin;
    let mins = (*saberent).r.mins;
    let maxs = (*saberent).r.maxs;
    VectorSubtract(&mins, &currentOrigin, &mut (*saberent).r.mins);
    VectorSubtract(&maxs, &currentOrigin, &mut (*saberent).r.maxs);
}

/// `int saberSpinSound = 0;` (w_saber.c:24) — cached `G_SoundIndex` for the thrown-saber spin
/// loop sound, set by [`WP_SaberInitBladeData`] and read where a thrown saber's `s.loopSound` is
/// assigned (w_saber.c:8258, not yet ported). A single-threaded game-module global.
pub static mut saberSpinSound: c_int = 0;

/// `void saberReactivate( gentity_t *saberent, gentity_t *saberOwner )` (w_saber.c:6053) —
/// reactivates an in-flight thrown saber entity: marks it `saberInFlight`, restores its linear
/// position/angular trajectory (the angular delta spins it about the Y axis at 800), resets it to
/// a plain `ET_GENERAL` entity with no `eFlags`, re-parents it to the owner, clears
/// `genericValue5`, recomputes the collision box via [`SetSaberBoxSize`], installs the
/// [`thrownSaberTouch`] callback, tags it as a `WP_SABER`, flags the owner's
/// `ps.saberEntityState`, and re-links it into the world. No oracle: a side-effecting setup over
/// the opaque `gentity_t` + [`trap::LinkEntity`], per the side-effect precedent.
///
/// # Safety
/// `saberent` must point to a valid `gentity_t`; `saberOwner` must point to a valid `gentity_t`
/// with a non-null `client`.
pub unsafe fn saberReactivate(saberent: *mut gentity_t, saberOwner: *mut gentity_t) {
    (*saberent).s.saberInFlight = QTRUE;

    (*saberent).s.apos.trType = TR_LINEAR;
    (*saberent).s.apos.trDelta[0] = 0.0;
    (*saberent).s.apos.trDelta[1] = 800.0;
    (*saberent).s.apos.trDelta[2] = 0.0;

    (*saberent).s.pos.trType = TR_LINEAR;
    (*saberent).s.eType = ET_GENERAL;
    (*saberent).s.eFlags = 0;

    (*saberent).parent = saberOwner;

    (*saberent).genericValue5 = 0;

    SetSaberBoxSize(saberent);

    (*saberent).touch = Some(thrownSaberTouch);

    (*saberent).s.weapon = WP_SABER;

    (*(*saberOwner).client).ps.saberEntityState = 1;

    trap::LinkEntity(saberent);
}

/// `void WP_SaberRemoveG2Model( gentity_t *saberent )` (w_saber.c:6160) — tears down the ghoul2
/// render instance attached to a saber entity (a dropped/thrown saber). If `saberent->ghoul2` is
/// non-null, frees it via [`trap::G2API_RemoveGhoul2Models`]. No oracle: a side-effecting trap
/// over the opaque `gentity_t::ghoul2` handle.
///
/// # Safety
/// `saberent` must point to a valid `gentity_t`.
pub unsafe fn WP_SaberRemoveG2Model(saberent: *mut gentity_t) {
    if !(*saberent).ghoul2.is_null() {
        trap::G2API_RemoveGhoul2Models(addr_of_mut!((*saberent).ghoul2) as *mut c_void);
    }
}

/// `void WP_SaberAddG2Model( gentity_t *saberent, const char *saberModel, qhandle_t saberSkin )`
/// (w_saber.c:6168) — gives a saber entity its ghoul2 render model. First removes any existing
/// model via [`WP_SaberRemoveG2Model`], sets `s.modelindex` from `saberModel` (or the default
/// `saber_w.glm` when none is given), then instances the model with
/// [`trap::G2API_InitGhoul2Model`]. The raw (possibly-NULL) `saberModel` is forwarded to the trap
/// exactly as the C does — the modelindex fallback does not substitute the trap's filename. No
/// oracle: a side-effecting trap over the opaque `gentity_t::ghoul2` handle.
///
/// # Safety
/// `saberent` must point to a valid `gentity_t`; `saberModel` must be NULL or a valid
/// NUL-terminated C string.
pub unsafe fn WP_SaberAddG2Model(
    saberent: *mut gentity_t,
    saberModel: *const c_char,
    saberSkin: qhandle_t,
) {
    WP_SaberRemoveG2Model(saberent);
    if !saberModel.is_null() && *saberModel != 0 {
        (*saberent).s.modelindex = G_ModelIndex(&CStr::from_ptr(saberModel).to_string_lossy());
    } else {
        (*saberent).s.modelindex = G_ModelIndex("models/weapons2/saber/saber_w.glm");
    }
    //FIXME: use customSkin?
    trap::G2API_InitGhoul2Model(
        addr_of_mut!((*saberent).ghoul2),
        saberModel,
        (*saberent).s.modelindex,
        saberSkin,
        0,
        0,
        0,
    );
}

/// `void WP_SaberInitBladeData( gentity_t *ent )` (w_saber.c:476) — creates (or reclaims) the
/// server-only "lightsaber" entity that backs a client's blade collision/box. It first sweeps
/// `g_entities[0..level.num_entities]` for stray `neverFree` lightsaber entities owned by this
/// client: the first is freed-and-reinited for reuse (its `s.modelGhoul2` zeroed first to avoid
/// issuing a needless kg2 to clients), any extras are scheduled for `G_FreeEntity`. If none was
/// reclaimed it [`G_Spawn`]s a fresh one. The saber entity is then wired up — owner, `classname`,
/// `neverFree`, the lightsaber clip/contents, box size via [`SetSaberBoxSize`], `EF_NODRAW` +
/// `SVF_NOCLIENT` (server-only), and `touch`/`think` callbacks ([`SaberGotHit`]/[`SaberUpdateSelf`])
/// — and the spin sound is cached into [`saberSpinSound`]. No oracle: a spawn/free-driven
/// side-effecting setup over the `g_entities`/`level` globals, per the side-effect precedent.
///
/// The C `if (!own)`-style redundant null tests are not present here; `checkEnt->classname &&
/// checkEnt->classname[0]` is preserved as a null-then-empty guard before [`Q_stricmp`].
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`.
pub unsafe fn WP_SaberInitBladeData(ent: *mut gentity_t) {
    let mut saberent: *mut gentity_t = null_mut();
    let mut i: c_int = 0;

    while i < (*addr_of!(level)).num_entities {
        //make sure there are no other saber entities floating around that think they belong to this client.
        let checkEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if (*checkEnt).inuse == QTRUE
            && (*checkEnt).neverFree == QTRUE
            && (*checkEnt).r.ownerNum == (*ent).s.number
            && !(*checkEnt).classname.is_null()
            && *(*checkEnt).classname != 0
            && Q_stricmp((*checkEnt).classname, c"lightsaber".as_ptr()) == 0
        {
            if !saberent.is_null() {
                //already have one
                (*checkEnt).neverFree = QFALSE;
                (*checkEnt).think = Some(G_FreeEntity);
                (*checkEnt).nextthink = (*addr_of!(level)).time;
            } else {
                //hmm.. well then, take it as my own.
                //free the bitch but don't issue a kg2 to avoid overflowing clients.
                (*checkEnt).s.modelGhoul2 = 0;
                G_FreeEntity(checkEnt);

                //now init it manually and reuse this ent slot.
                G_InitGentity(checkEnt);
                saberent = checkEnt;
            }
        }

        i += 1;
    }

    //We do not want the client to have any real knowledge of the entity whatsoever. It will only
    //ever be used on the server.
    if saberent.is_null() {
        //ok, make one then
        saberent = G_Spawn();
    }
    let saber_num = (*saberent).s.number;
    (*(*ent).client).saberStoredIndex = saber_num;
    (*(*ent).client).ps.saberEntityNum = saber_num;
    (*saberent).classname = c"lightsaber".as_ptr() as *mut c_char;

    (*saberent).neverFree = QTRUE; //the saber being removed would be a terrible thing.

    (*saberent).r.svFlags = SVF_USE_CURRENT_ORIGIN as c_int;
    (*saberent).r.ownerNum = (*ent).s.number;

    (*saberent).clipmask = MASK_PLAYERSOLID | CONTENTS_LIGHTSABER;
    (*saberent).r.contents = CONTENTS_LIGHTSABER;

    SetSaberBoxSize(saberent);

    (*saberent).mass = 10.0;

    (*saberent).s.eFlags |= EF_NODRAW;
    (*saberent).r.svFlags |= SVF_NOCLIENT as c_int;

    (*saberent).s.modelGhoul2 = 1;
    //should we happen to be removed (we belong to an NPC and he is removed) then
    //we want to attempt to remove our g2 instance on the client in case we had one.

    (*saberent).touch = Some(SaberGotHit);

    (*saberent).think = Some(SaberUpdateSelf);
    (*saberent).genericValue5 = 0;
    (*saberent).nextthink = (*addr_of!(level)).time + 50;

    saberSpinSound = G_SoundIndex("sound/weapons/saber/saberspin.wav");
}

/// `float WP_SaberBladeLength( saberInfo_t *saber )` (w_saber.c:2642) — returns the largest
/// `lengthMax` across the saber's active blades (`0..numBlades`). A pure scan over the blade
/// array; single precision throughout, faithful to the C.
///
/// # Safety
/// `saber` must point to a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn WP_SaberBladeLength(saber: *const saberInfo_t) -> f32 {
    //return largest length
    let mut len: f32 = 0.0;
    for i in 0..(*saber).numBlades {
        if (*saber).blade[i as usize].lengthMax > len {
            len = (*saber).blade[i as usize].lengthMax;
        }
    }
    len
}

/// `float WP_SaberLength( gentity_t *ent )` (w_saber.c:2656) — returns the largest blade
/// length across all of the client's sabers (via [`WP_SaberBladeLength`] over each of the
/// `MAX_SABERS` slots). Returns `0.0` for a null entity or a non-client. Faithful to the C.
///
/// # Safety
/// `ent` must be null or point to a valid `gentity_t`.
pub unsafe fn WP_SaberLength(ent: *const gentity_t) -> f32 {
    //return largest length
    if ent.is_null() || (*ent).client.is_null() {
        0.0
    } else {
        let mut best_len: f32 = 0.0;
        for i in 0..MAX_SABERS {
            let len = WP_SaberBladeLength(addr_of!((*(*ent).client).saber[i]));
            if len > best_len {
                best_len = len;
            }
        }
        best_len
    }
}

/// `void WP_DeactivateSaber( gentity_t *self, qboolean clearLength )` (w_saber.c:218) —
/// holsters the client's saber. No-op for a null entity or non-client, or if the saber is
/// already holstered. Otherwise sets `saberHolstered = 2` and plays each saber's
/// turn-off sound on `CHAN_WEAPON` (the second saber only when it has a hilt model). The
/// `clearLength` argument is unused in the original (the `SetSaberLength` call is commented
/// out — "Doesn't matter ATM"); it is kept verbatim for ABI fidelity. No oracle —
/// side-effecting through [`G_Sound`] (which allocates a temp-entity from the `g_entities`
/// global), per the established side-effect precedent.
///
/// # Safety
/// `self_` must be null or point to a valid `gentity_t`.
pub unsafe fn WP_DeactivateSaber(self_: *mut gentity_t, _clearLength: qboolean) {
    if self_.is_null() || (*self_).client.is_null() {
        return;
    }
    let client = (*self_).client;
    //keep my saber off!
    if (*client).ps.saberHolstered == 0 {
        (*client).ps.saberHolstered = 2;
        /*
        if ( clearLength )
        {
            self->client->ps.SetSaberLength( 0 );
        }
        */
        //Doens't matter ATM
        if (*client).saber[0].soundOff != 0 {
            G_Sound(self_, CHAN_WEAPON, (*client).saber[0].soundOff);
        }

        if (*client).saber[1].soundOff != 0 && (*client).saber[1].model[0] != 0 {
            G_Sound(self_, CHAN_WEAPON, (*client).saber[1].soundOff);
        }
    }
}

/// `void WP_ActivateSaber( gentity_t *self )` (w_saber.c:250) — unholsters the client's
/// saber. No-op for a null entity or non-client. If the client is an NPC mid-jedi-taunt
/// (with >200 ms of taunt remaining) the taunt is cancelled; otherwise an active
/// force-grip cripple blocks activation entirely. When the saber is holstered it is
/// switched on (`saberHolstered = 0`) and each saber's turn-on sound plays on
/// `CHAN_WEAPON`. The `self->NPC` field is only null-checked, never dereferenced. No
/// oracle — side-effecting through [`G_Sound`], per the side-effect precedent.
///
/// # Safety
/// `self_` must be null or point to a valid `gentity_t`.
pub unsafe fn WP_ActivateSaber(self_: *mut gentity_t) {
    if self_.is_null() || (*self_).client.is_null() {
        return;
    }
    let client = (*self_).client;

    if !(*self_).NPC.is_null()
        && (*client).ps.forceHandExtend == HANDEXTEND_JEDITAUNT
        && ((*client).ps.forceHandExtendTime - level.time) > 200
    {
        //if we're an NPC and in the middle of a taunt then stop it
        (*client).ps.forceHandExtend = HANDEXTEND_NONE;
        (*client).ps.forceHandExtendTime = 0;
    } else if (*client).ps.fd.forceGripCripple != 0 {
        //can't activate saber while being gripped
        return;
    }

    if (*client).ps.saberHolstered != 0 {
        (*client).ps.saberHolstered = 0;
        if (*client).saber[0].soundOn != 0 {
            G_Sound(self_, CHAN_WEAPON, (*client).saber[0].soundOn);
        }

        if (*client).saber[1].soundOn != 0 {
            G_Sound(self_, CHAN_WEAPON, (*client).saber[1].soundOn);
        }
    }
}

/// `static qboolean G_G2TraceCollide(trace_t *tr, vec3_t lastValidStart, vec3_t lastValidEnd,
/// vec3_t traceMins, vec3_t traceMaxs)` (w_saber.c:2209) — refine a normal trace that hit an
/// entity with a precise per-poly ghoul2 collision trace. Only runs when
/// `d_saberGhoul2Collision` is set and the hit entity is in use and has a ghoul2 instance; the
/// trace box half-width becomes the ghoul2 trace radius. Drives the engine's
/// `trap_G2API_CollisionDetect` against the hit model: if the first collision record matches the
/// entity, the trace endpos/normal are overwritten with the precise hit (and the client's
/// last-surface-hit is recorded) and it returns `QTRUE`; otherwise the trace is reset to "no hit"
/// and it returns `QFALSE`. No oracle — drives a trap callback and reads `g_entities`/`level`.
///
/// # Safety
/// `tr` must point to a valid `trace_t`; `g_entities`/`level` must be initialised.
#[allow(static_mut_refs)]
pub unsafe fn G_G2TraceCollide(
    tr: *mut trace_t,
    lastValidStart: &vec3_t,
    lastValidEnd: &vec3_t,
    traceMins: &vec3_t,
    traceMaxs: &vec3_t,
) -> qboolean {
    //Hit the ent with the normal trace, try the collision trace.
    let mut G2Trace: G2Trace_t;
    let g2Hit: *mut gentity_t;
    let mut angles: vec3_t = [0.0; 3];
    let mut tN: usize = 0;
    let mut fRadius: f32 = 0.0;

    if (*addr_of!(d_saberGhoul2Collision)).integer == 0 {
        return QFALSE;
    }

    if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*tr).entityNum as usize))
        .inuse
        == QFALSE
    /*||
    (g_entities[tr->entityNum].s.eFlags & EF_DEAD)*/
    {
        //don't do perpoly on corpses.
        return QFALSE;
    }

    if traceMins[0] != 0.0
        || traceMins[1] != 0.0
        || traceMins[2] != 0.0
        || traceMaxs[0] != 0.0
        || traceMaxs[1] != 0.0
        || traceMaxs[2] != 0.0
    {
        fRadius = (traceMaxs[0] - traceMins[0]) / 2.0;
    }

    G2Trace = [Default::default(); MAX_G2_COLLISIONS];

    while tN < MAX_G2_COLLISIONS {
        G2Trace[tN].mEntityNum = -1;
        tN += 1;
    }
    g2Hit = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*tr).entityNum as usize);

    if !g2Hit.is_null() && (*g2Hit).inuse != QFALSE && !(*g2Hit).ghoul2.is_null() {
        let mut g2HitOrigin: vec3_t = [0.0; 3];

        angles[ROLL as usize] = 0.0;
        angles[PITCH as usize] = 0.0;

        if !(*g2Hit).client.is_null() {
            VectorCopy(&(*(*g2Hit).client).ps.origin, &mut g2HitOrigin);
            angles[YAW as usize] = (*(*g2Hit).client).ps.viewangles[YAW as usize];
        } else {
            VectorCopy(&(*g2Hit).r.currentOrigin, &mut g2HitOrigin);
            angles[YAW as usize] = (*g2Hit).r.currentAngles[YAW as usize];
        }

        if (*addr_of!(g_optvehtrace)).integer != 0
            && (*g2Hit).s.eType == ET_NPC
            && (*g2Hit).s.NPC_class == CLASS_VEHICLE
            && !(*g2Hit).m_pVehicle.is_null()
        {
            trap::G2API_CollisionDetectCache(
                G2Trace.as_mut_ptr(),
                (*g2Hit).ghoul2,
                &angles,
                &g2HitOrigin,
                (*addr_of!(level)).time,
                (*g2Hit).s.number,
                lastValidStart,
                lastValidEnd,
                &(*g2Hit).modelScale,
                0,
                (*addr_of!(g_g2TraceLod)).integer,
                fRadius,
            );
        } else {
            trap::G2API_CollisionDetect(
                G2Trace.as_mut_ptr(),
                (*g2Hit).ghoul2,
                &angles,
                &g2HitOrigin,
                (*addr_of!(level)).time,
                (*g2Hit).s.number,
                lastValidStart,
                lastValidEnd,
                &(*g2Hit).modelScale,
                0,
                (*addr_of!(g_g2TraceLod)).integer,
                fRadius,
            );
        }

        if G2Trace[0].mEntityNum != (*g2Hit).s.number {
            (*tr).fraction = 1.0;
            (*tr).entityNum = ENTITYNUM_NONE as i16;
            (*tr).startsolid = 0;
            (*tr).allsolid = 0;
            return QFALSE;
        } else {
            //The ghoul2 trace result matches, so copy the collision position into the trace endpos and send it back.
            VectorCopy(&G2Trace[0].mCollisionPosition, &mut (*tr).endpos);
            VectorCopy(&G2Trace[0].mCollisionNormal, &mut (*tr).plane.normal);

            if !(*g2Hit).client.is_null() {
                (*(*g2Hit).client).g2LastSurfaceHit = G2Trace[0].mSurfaceIndex;
                (*(*g2Hit).client).g2LastSurfaceTime = (*addr_of!(level)).time;
            }
            return QTRUE;
        }
    }

    QFALSE
}

/// `qboolean G_SaberInBackAttack(int move)` (w_saber.c:2291) — a pure predicate that returns
/// `QTRUE` when the saber move is one of the three back-attack moves (`LS_A_BACK`,
/// `LS_A_BACK_CR`, `LS_A_BACKSTAB`). Faithful to the C switch.
pub fn G_SaberInBackAttack(r#move: c_int) -> qboolean {
    match r#move {
        LS_A_BACK | LS_A_BACK_CR | LS_A_BACKSTAB => QTRUE,
        _ => QFALSE,
    }
}

/// `qboolean SaberAttacking(gentity_t *self)` (w_saber.c:909) — decides whether the client is
/// currently in an offensive saber action. Returns `QFALSE` for any of the defensive/reactive
/// moves (parry, broken-parry, deflect, bounce, knockaway); otherwise `QTRUE` only when in an
/// attack move while firing and not blocked, or when in any saber special. Faithful to the C.
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a non-null `client`.
pub unsafe fn SaberAttacking(self_: *const gentity_t) -> qboolean {
    let ps = addr_of!((*(*self_).client).ps);
    if PM_SaberInParry((*ps).saberMove) == QTRUE {
        return QFALSE;
    }
    if PM_SaberInBrokenParry((*ps).saberMove) == QTRUE {
        return QFALSE;
    }
    if PM_SaberInDeflect((*ps).saberMove) == QTRUE {
        return QFALSE;
    }
    if PM_SaberInBounce((*ps).saberMove) == QTRUE {
        return QFALSE;
    }
    if PM_SaberInKnockaway((*ps).saberMove) == QTRUE {
        return QFALSE;
    }

    if BG_SaberInAttack((*ps).saberMove) == QTRUE
        && (*ps).weaponstate == WEAPON_FIRING
        && (*ps).saberBlocked == BLOCKED_NONE
    {
        //if we're firing and not blocking, then we're attacking.
        return QTRUE;
    }

    if BG_SaberInSpecial((*ps).saberMove) == QTRUE {
        return QTRUE;
    }

    QFALSE
}

/// `#define MAX_SABER_VICTIMS 16` (w_saber.c:3375) — capacity of the per-frame saber damage
/// accumulator (the `victim*` / `*Dmg` module statics below).
pub const MAX_SABER_VICTIMS: usize = 16;

// Module-static saber damage accumulator (w_saber.c:3376-3383). A single saber trace can hit
// several entities in one frame; rather than apply damage at each trace point, the trace path
// records every victim here and `WP_SaberApplyDamage` flushes it once at the end of the frame.
// `WP_SaberClearDamage` resets the lot, `WP_SaberDamageAdd` appends. Faithful to the C globals.
pub static mut victimEntityNum: [c_int; MAX_SABER_VICTIMS] = [0; MAX_SABER_VICTIMS];
pub static mut victimHitEffectDone: [qboolean; MAX_SABER_VICTIMS] = [QFALSE; MAX_SABER_VICTIMS];
pub static mut totalDmg: [f32; MAX_SABER_VICTIMS] = [0.0; MAX_SABER_VICTIMS];
pub static mut dmgDir: [vec3_t; MAX_SABER_VICTIMS] = [[0.0; 3]; MAX_SABER_VICTIMS];
pub static mut dmgSpot: [vec3_t; MAX_SABER_VICTIMS] = [[0.0; 3]; MAX_SABER_VICTIMS];
pub static mut dismemberDmg: [qboolean; MAX_SABER_VICTIMS] = [QFALSE; MAX_SABER_VICTIMS];
pub static mut saberKnockbackFlags: [c_int; MAX_SABER_VICTIMS] = [0; MAX_SABER_VICTIMS];
pub static mut numVictims: c_int = 0;

/// `void WP_SaberClearDamage( void )` (w_saber.c:3384) — reset the per-frame saber damage
/// accumulator: every victim slot is marked empty (`ENTITYNUM_NONE`), all damage/dir/spot and
/// the dismember/knockback flags are zeroed, and the count is cleared. Faithful to the C, which
/// zeroes the arrays via `memset`; the `victimEntityNum` slots are set to `ENTITYNUM_NONE` (not
/// 0) in an explicit loop, matching the C exactly.
///
/// # Safety
/// Mutates the module-static saber damage accumulator; call only from the single game thread.
pub unsafe fn WP_SaberClearDamage() {
    for ven in 0..MAX_SABER_VICTIMS {
        victimEntityNum[ven] = ENTITYNUM_NONE;
    }
    victimHitEffectDone = [QFALSE; MAX_SABER_VICTIMS];
    totalDmg = [0.0; MAX_SABER_VICTIMS];
    dmgDir = [[0.0; 3]; MAX_SABER_VICTIMS];
    dmgSpot = [[0.0; 3]; MAX_SABER_VICTIMS];
    dismemberDmg = [QFALSE; MAX_SABER_VICTIMS];
    saberKnockbackFlags = [0; MAX_SABER_VICTIMS];
    numVictims = 0;
}

/// `void WP_SaberDamageAdd( ... )` (w_saber.c:3400) — record a saber hit into the per-frame
/// accumulator. Skips non-damaging hits, the world and invalid ent nums. For a real hit it
/// finds the victim's existing slot (or appends a new one, bailing if the table is full),
/// sums the damage, captures the first dir/spot seen for that victim (only when still the
/// zero vector), and OR-s in the dismember flag and the `knock_back_flags` mask. Faithful to
/// the C — PC passes a precomputed `DAMAGE_SABER_KNOCKBACK*` mask (Xbox passed a bool + saber
/// index and OR-ed the flag here).
///
/// # Safety
/// Mutates the module-static saber damage accumulator; `tr_dmg_dir`/`tr_dmg_spot` must be
/// valid `vec3_t`. Call only from the single game thread.
pub unsafe fn WP_SaberDamageAdd(
    tr_victim_entity_num: c_int,
    tr_dmg_dir: *const vec3_t,
    tr_dmg_spot: *const vec3_t,
    tr_dmg: c_int,
    do_dismemberment: qboolean,
    knock_back_flags: c_int,
) {
    if tr_victim_entity_num < 0 || tr_victim_entity_num >= ENTITYNUM_WORLD {
        return;
    }

    if tr_dmg != 0 {
        //did some damage to something
        let mut cur_victim: usize = 0;
        let mut i: c_int = 0;

        while i < numVictims {
            if victimEntityNum[i as usize] == tr_victim_entity_num {
                //already hit this guy before
                cur_victim = i as usize;
                break;
            }
            i += 1;
        }
        if i == numVictims {
            //haven't hit his guy before
            if numVictims + 1 >= MAX_SABER_VICTIMS as c_int {
                //can't add another victim at this time
                return;
            }
            //add a new victim to the list
            cur_victim = numVictims as usize;
            victimEntityNum[numVictims as usize] = tr_victim_entity_num;
            numVictims += 1;
        }

        totalDmg[cur_victim] += tr_dmg as f32;
        if VectorCompare(&dmgDir[cur_victim], &vec3_origin) != 0 {
            VectorCopy(&*tr_dmg_dir, &mut dmgDir[cur_victim]);
        }
        if VectorCompare(&dmgSpot[cur_victim], &vec3_origin) != 0 {
            VectorCopy(&*tr_dmg_spot, &mut dmgSpot[cur_victim]);
        }
        if do_dismemberment != QFALSE {
            dismemberDmg[cur_victim] = QTRUE;
        }
        saberKnockbackFlags[cur_victim] |= knock_back_flags;
    }
}

/// `#define SABER_NONATTACK_DAMAGE 1` (w_saber.c:2108) — minimum total damage above which a
/// saber-on-saber (non-flesh) hit spawns a clash flare.
const SABER_NONATTACK_DAMAGE: c_int = 1;

/// `#define SABER_HITDAMAGE 35` (w_saber.c:964) — the base saber hit damage used by the
/// non-SP-style damage path (level-1 attacks and the various ramping baselines).
// `allow(dead_code)`: only consumer is CheckSaberDamage, whose caller is not yet ported.
#[allow(dead_code)]
const SABER_HITDAMAGE: c_int = 35;

/// `void WP_SaberApplyDamage( gentity_t *self )` (w_saber.c:3577) — flushes the per-frame saber
/// damage accumulator (filled by [`WP_SaberDamageAdd`]) into actual damage. For each recorded
/// victim it scales wall damage by `g_saberWallDamageScale` (clientless targets), builds the
/// dismember `dflags` plus the accumulated `saberKnockbackFlags`, and applies [`G_Damage`] with
/// [`MOD_SABER`]. The impact temp-entity / blood-spark / clash-flare effects are no longer done
/// here — PC splits them into [`WP_SaberDoHit`] (called per-blade by the caller). No oracle:
/// void + side-effecting via [`G_Damage`].
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; reads the module-static saber damage accumulator.
/// Call only from the single game thread.
pub unsafe fn WP_SaberApplyDamage(self_: *mut gentity_t) {
    if numVictims == 0 {
        return;
    }
    let mut i: c_int = 0;
    while i < numVictims {
        let victim: *mut gentity_t;
        let mut dflags: c_int = 0;

        victim = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add(victimEntityNum[i as usize] as usize);

        // nmckenzie: SABER_DAMAGE_WALLS
        if (*victim).client.is_null() {
            totalDmg[i as usize] *= g_saberWallDamageScale.value;
        }

        if dismemberDmg[i as usize] == QFALSE {
            //don't do dismemberment!
            dflags |= DAMAGE_NO_DISMEMBER;
        }
        dflags |= saberKnockbackFlags[i as usize];

        G_Damage(
            victim,
            self_,
            self_,
            addr_of_mut!(dmgDir[i as usize]),
            addr_of_mut!(dmgSpot[i as usize]),
            totalDmg[i as usize] as c_int,
            dflags,
            MOD_SABER,
        );

        i += 1;
    }
}

/// `void WP_SaberDoHit( gentity_t *self, int saberNum, int bladeNum )` (w_saber.c:3608) — PC-new:
/// spawns the saber-impact effects for every accumulated victim (split out of the Xbox
/// `WP_SaberApplyDamage`). Each victim is processed once per frame via the `victimHitEffectDone`
/// latch. Spawns an `EV_SABER_HIT` temp-entity tagged with the victim/attacker nums, the saber
/// and blade numbers, the hit origin and (negated) hit direction. Flesh hits (clients, NPCs,
/// bodies — not droids) get a blood-spark magnitude in `eventParm` (3/2/1 by damage); saber-vs-
/// saber / wall hits instead spawn an `EV_SABER_CLASHFLARE` (unless the blade's per-style
/// `SFL2_NO_CLASH_FLARE`/`SFL2_NO_CLASH_FLARE2` is set). No oracle: void + side-effecting via
/// [`G_TempEntity`].
///
/// # Safety
/// `self_` must point to a valid client `gentity_t`; reads/mutates the module-static saber
/// damage accumulator and spawns temp-entities. Call only from the single game thread.
pub unsafe fn WP_SaberDoHit(self_: *mut gentity_t, saberNum: c_int, bladeNum: c_int) {
    if numVictims == 0 {
        return;
    }
    let mut i: c_int = 0;
    while i < numVictims {
        let te: *mut gentity_t;
        let victim: *mut gentity_t;
        let mut is_droid: qboolean = QFALSE;

        if victimHitEffectDone[i as usize] != QFALSE {
            i += 1;
            continue;
        }

        victimHitEffectDone[i as usize] = QTRUE;

        victim = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add(victimEntityNum[i as usize] as usize);

        if !(*victim).client.is_null() {
            let npc_class = (*(*victim).client).NPC_class;

            if npc_class == CLASS_SEEKER
                || npc_class == CLASS_PROBE
                || npc_class == CLASS_MOUSE
                || npc_class == CLASS_REMOTE
                || npc_class == CLASS_GONK
                || npc_class == CLASS_R2D2
                || npc_class == CLASS_R5D2
                || npc_class == CLASS_PROTOCOL
                || npc_class == CLASS_MARK1
                || npc_class == CLASS_MARK2
                || npc_class == CLASS_INTERROGATOR
                || npc_class == CLASS_ATST
                || npc_class == CLASS_SENTRY
            {
                //don't make "blood" sparks for droids.
                is_droid = QTRUE;
            }
        }

        te = G_TempEntity(&dmgSpot[i as usize], EV_SABER_HIT);
        if !te.is_null() {
            (*te).s.otherEntityNum = victimEntityNum[i as usize];
            (*te).s.otherEntityNum2 = (*self_).s.number;
            (*te).s.weapon = saberNum;
            (*te).s.legsAnim = bladeNum;

            VectorCopy(&dmgSpot[i as usize], &mut (*te).s.origin);
            //VectorCopy(tr.plane.normal, te->s.angles);
            VectorScale(&dmgDir[i as usize], -1.0, &mut (*te).s.angles);

            if (*te).s.angles[0] == 0.0 && (*te).s.angles[1] == 0.0 && (*te).s.angles[2] == 0.0 {
                //don't let it play with no direction
                (*te).s.angles[1] = 1.0;
            }

            if is_droid == QFALSE
                && (!(*victim).client.is_null()
                    || (*victim).s.eType == ET_NPC
                    || (*victim).s.eType == ET_BODY)
            {
                if totalDmg[i as usize] < 5.0 {
                    (*te).s.eventParm = 3;
                } else if totalDmg[i as usize] < 20.0 {
                    (*te).s.eventParm = 2;
                } else {
                    (*te).s.eventParm = 1;
                }
            } else if WP_SaberBladeUseSecondBladeStyle(
                addr_of_mut!((*(*self_).client).saber[saberNum as usize]),
                bladeNum,
            ) == QFALSE
                && (*(*self_).client).saber[saberNum as usize].saberFlags2 & SFL2_NO_CLASH_FLARE
                    != 0
            {
                //don't do clash flare
            } else if WP_SaberBladeUseSecondBladeStyle(
                addr_of_mut!((*(*self_).client).saber[saberNum as usize]),
                bladeNum,
            ) != QFALSE
                && (*(*self_).client).saber[saberNum as usize].saberFlags2 & SFL2_NO_CLASH_FLARE2
                    != 0
            {
                //don't do clash flare
            } else {
                if totalDmg[i as usize] > SABER_NONATTACK_DAMAGE as f32 {
                    //I suppose I could tie this into the saberblock event, but I'm tired of adding flags to that thing.
                    let teS: *mut gentity_t = G_TempEntity(&(*te).s.origin, EV_SABER_CLASHFLARE);
                    VectorCopy(&(*te).s.origin, &mut (*teS).s.origin);
                }
                (*te).s.eventParm = 0;
            }
        }

        i += 1;
    }
}

/// `void WP_SaberRadiusDamage( gentity_t *ent, vec3_t point, float radius, int damage, float knockBack )`
/// (w_saber.c:3701) — area damage + knockback radiating from `point` (e.g. a saber spin attack).
/// Walks every entity in the `point ± radius` AABB ([`trap::EntitiesInBox`]): skips the not-in-use,
/// `ent` itself, and the held-by-a-monster; deals a flat 10 [`MOD_MELEE`] to clientless breakables;
/// and, for clients within `radius`, applies distance-scaled [`G_Damage`] (`ceil(damage*dist/radius)`,
/// `DAMAGE_NO_KNOCKBACK`) plus a distance-scaled [`G_Throw`] / [`G_Knockdown`] (except Rancor/AT-ST
/// and `FL_NO_KNOCKBACK` ents). No oracle — `trap_EntitiesInBox` + the `G_Damage`/`G_Throw`/
/// `G_Knockdown` chain over the global `g_entities`.
///
/// # Safety
/// `ent` may be null; if non-null with `client` set it must be a valid `gentity_t`. Touches the
/// `g_entities` global; call only from the single game thread.
pub unsafe fn WP_SaberRadiusDamage(
    ent: *mut gentity_t,
    point: &vec3_t,
    radius: f32,
    damage: c_int,
    knockBack: f32,
) {
    if ent.is_null() || (*ent).client.is_null() {
        return;
    } else if radius <= 0.0f32 || (damage <= 0 && knockBack <= 0.0) {
        return;
    } else {
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut entDir: vec3_t = [0.0; 3];
        let mut radiusEnts: [c_int; 128] = [0; 128];
        let mut radiusEnt: *mut gentity_t;
        let numEnts: c_int;
        let mut dist: f32;

        //Setup the bbox to search in
        for i in 0..3 {
            mins[i] = point[i] - radius;
            maxs[i] = point[i] + radius;
        }

        //Get the number of entities in a given space
        numEnts = trap::EntitiesInBox(&mins, &maxs, &mut radiusEnts);

        let mut i: c_int = 0;
        while i < numEnts {
            radiusEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(radiusEnts[i as usize] as usize);
            if (*radiusEnt).inuse == QFALSE {
                i += 1;
                continue;
            }

            if radiusEnt == ent {
                //Skip myself
                i += 1;
                continue;
            }

            if (*radiusEnt).client.is_null() {
                //must be a client
                if G_EntIsBreakable((*radiusEnt).s.number) != QFALSE {
                    //damage breakables within range, but not as much
                    // DEVIATION: C passes the shared `vec3_origin` global as the (in-place
                    // normalized) `dir` arg; ported as a local zero vector — VectorNormalize of
                    // the zero vector is a no-op so the result is identical.
                    let mut zero_dir: vec3_t = vec3_origin;
                    G_Damage(
                        radiusEnt,
                        ent,
                        ent,
                        addr_of_mut!(zero_dir),
                        addr_of_mut!((*radiusEnt).r.currentOrigin),
                        10,
                        0,
                        MOD_MELEE,
                    );
                }
                i += 1;
                continue;
            }

            if (*(*radiusEnt).client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
                //can't be one being held
                i += 1;
                continue;
            }

            VectorSubtract(&(*radiusEnt).r.currentOrigin, point, &mut entDir);
            dist = VectorNormalize(&mut entDir);
            if dist <= radius {
                //in range
                if damage > 0 {
                    //do damage
                    let points: c_int = ((damage as f32 * dist / radius) as f64).ceil() as c_int;
                    let mut zero_dir: vec3_t = vec3_origin;
                    G_Damage(
                        radiusEnt,
                        ent,
                        ent,
                        addr_of_mut!(zero_dir),
                        addr_of_mut!((*radiusEnt).r.currentOrigin),
                        points,
                        DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE,
                    );
                }
                if knockBack > 0.0 {
                    //do knockback
                    if !(*radiusEnt).client.is_null()
                        && (*(*radiusEnt).client).NPC_class != CLASS_RANCOR
                        && (*(*radiusEnt).client).NPC_class != CLASS_ATST
                        && (*radiusEnt).flags & FL_NO_KNOCKBACK == 0
                    {
                        //don't throw them back
                        let knockbackStr = knockBack * dist / radius;
                        entDir[2] += 0.1f32;
                        VectorNormalize(&mut entDir);
                        G_Throw(radiusEnt, &entDir, knockbackStr);
                        if (*radiusEnt).health > 0 {
                            //still alive
                            if knockbackStr > 50.0 {
                                //close enough and knockback high enough to possibly knock down
                                if dist < (radius * 0.5f32)
                                    || (*(*radiusEnt).client).ps.groundEntityNum != ENTITYNUM_NONE
                                {
                                    //within range of my fist or within ground-shaking range and not in the air
                                    G_Knockdown(radiusEnt); //, ent, entDir, 500, qtrue );
                                }
                            }
                        }
                    }
                }
            }

            i += 1;
        }
    }
}

/// `static qboolean G_KickDownable(gentity_t *ent)` (w_saber.c:7048) — gate for whether a
/// kick may knock `ent` down. When the `d_saberKickTweak` cvar is off the answer is always
/// `qtrue` (vanilla behavior). With the tweak on: a null/dead/clientless ent can't be kicked
/// down; an ent already in a knockdown (legs or torso) can't be kicked down again; and an ent
/// standing on the ground with a saber out and no weapon-time pending is immune (so it can be
/// staggered rather than floored). Faithful to the C.
///
/// # Safety
/// `ent` may be null; if non-null with `inuse`/`client` set it must be a valid `gentity_t`.
pub unsafe fn G_KickDownable(ent: *const gentity_t) -> qboolean {
    if (*addr_of!(d_saberKickTweak)).integer == 0 {
        return QTRUE;
    }

    if ent.is_null() || (*ent).inuse == QFALSE || (*ent).client.is_null() {
        return QFALSE;
    }

    if BG_InKnockDown((*(*ent).client).ps.legsAnim) != QFALSE
        || BG_InKnockDown((*(*ent).client).ps.torsoAnim) != QFALSE
    {
        return QFALSE;
    }

    if (*(*ent).client).ps.weaponTime <= 0
        && (*(*ent).client).ps.weapon == WP_SABER
        && (*(*ent).client).ps.groundEntityNum != ENTITYNUM_NONE
    {
        return QFALSE;
    }

    QTRUE
}

/// `static void G_TossTheMofo(gentity_t *ent, vec3_t tossDir, float tossStr)` (w_saber.c:7076) —
/// fling a client into the air along `tossDir` (scaled by `tossStr`), pinning the upward velocity
/// to a fixed `200`, and — if they're still alive, not already being knocked down, and both
/// [`BG_KnockDownable`] and [`G_KickDownable`] agree — drop them into a knockdown for 700ms. Does
/// nothing for non-client / not-in-use ents or for vehicle NPCs. Mutates `ent`'s playerState.
///
/// Faithful to the C; the in-place `VectorMA(velocity, …, velocity)` is spelled with a snapshot of
/// the current velocity as the read source (each output component depends only on its own input
/// component, so this is identical to the C's read-then-write).
///
/// # Safety
/// `ent` may be null; if non-null with `inuse`/`client` set it must be a valid `gentity_t`.
pub unsafe fn G_TossTheMofo(ent: *mut gentity_t, tossDir: &vec3_t, tossStr: f32) {
    if (*ent).inuse == QFALSE || (*ent).client.is_null() {
        //no good
        return;
    }

    if (*ent).s.eType == ET_NPC && (*ent).s.NPC_class == CLASS_VEHICLE {
        //no, silly
        return;
    }

    let vel = (*(*ent).client).ps.velocity;
    VectorMA(&vel, tossStr, tossDir, &mut (*(*ent).client).ps.velocity);
    (*(*ent).client).ps.velocity[2] = 200.0;
    if (*ent).health > 0
        && (*(*ent).client).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN
        && BG_KnockDownable(addr_of_mut!((*(*ent).client).ps)) != QFALSE
        && G_KickDownable(ent) != QFALSE
    {
        //if they are alive, knock them down I suppose
        (*(*ent).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
        (*(*ent).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 700;
        (*(*ent).client).ps.forceDodgeAnim = 0; //this toggles between 1 and 0, when it's 1 we should play the get up anim
                                                //ent->client->ps.quickerGetup = qtrue;
    }
}

/// `int WP_MissileBlockForBlock( int saberBlock )` (w_saber.c:8657) — maps a directional
/// saber-block result to its projectile-reflect (`*_PROJ`) counterpart. The five directional
/// blocks (`UPPER_RIGHT`/`UPPER_LEFT`/`LOWER_RIGHT`/`LOWER_LEFT`/`TOP`) translate to their
/// `*_PROJ` variants; any other value (including the already-`_PROJ` and non-directional
/// blocks) passes through unchanged. Pure switch over the `saberBlockedType_t` enum, faithful
/// to the C (the redundant `break`s after each `return` are dropped — unreachable in Rust).
pub fn WP_MissileBlockForBlock(saberBlock: c_int) -> c_int {
    match saberBlock {
        BLOCKED_UPPER_RIGHT => BLOCKED_UPPER_RIGHT_PROJ,
        BLOCKED_UPPER_LEFT => BLOCKED_UPPER_LEFT_PROJ,
        BLOCKED_LOWER_RIGHT => BLOCKED_LOWER_RIGHT_PROJ,
        BLOCKED_LOWER_LEFT => BLOCKED_LOWER_LEFT_PROJ,
        BLOCKED_TOP => BLOCKED_TOP_PROJ,
        _ => saberBlock,
    }
}

/// `int G_GetParryForBlock(int block)` (w_saber.c:1764) — maps a saber-block result to the
/// `saberMoveName_t` parry/reflect move that answers it. The five directional blocks map to
/// their `LS_PARRY_*` parry moves; the matching `*_PROJ` blocks map to their `LS_REFLECT_*`
/// reflect moves; anything else (including the non-directional blocks) yields `LS_NONE`. Pure
/// switch over the `saberBlockedType_t` enum, faithful to the C (the redundant `break`s after
/// each `return` are dropped — unreachable in Rust).
pub fn G_GetParryForBlock(block: c_int) -> c_int {
    match block {
        BLOCKED_UPPER_RIGHT => LS_PARRY_UR,
        BLOCKED_UPPER_RIGHT_PROJ => LS_REFLECT_UR,
        BLOCKED_UPPER_LEFT => LS_PARRY_UL,
        BLOCKED_UPPER_LEFT_PROJ => LS_REFLECT_UL,
        BLOCKED_LOWER_RIGHT => LS_PARRY_LR,
        BLOCKED_LOWER_RIGHT_PROJ => LS_REFLECT_LR,
        BLOCKED_LOWER_LEFT => LS_PARRY_LL,
        BLOCKED_LOWER_LEFT_PROJ => LS_REFLECT_LL,
        BLOCKED_TOP => LS_PARRY_UP,
        BLOCKED_TOP_PROJ => LS_REFLECT_UP,
        _ => LS_NONE,
    }
}

/// `void WP_SaberBlockNonRandom( gentity_t *self, vec3_t hitloc, qboolean missileBlock )`
/// (w_saber.c:8680) — deterministic version of [`WP_SaberBlock`](self): pick the
/// `ps.saberBlocked` quadrant for a hit at `hitloc`, with no random jitter (unlike `WP_SaberBlock`,
/// which adds `RandFloat`/`Q_irand` slop). Builds a horizontal direction from the client's eye
/// (origin + viewheight) to `hitloc`, projects it onto the view's `right` vector for a left/right
/// `rightdot`, and uses the vertical delta `zdiff` to split top/upper/lower. When `missileBlock`
/// is set, the chosen block is mapped to its projectile-reflect variant via
/// [`WP_MissileBlockForBlock`]. Faithful to the C, including the empty `if ( zdiff < -10 )` block
/// (a no-op the C left as a placeholder comment) and the commented-out original threshold values.
///
/// # Safety
/// Dereferences `self` and its `client`; the caller must pass a valid client entity.
pub unsafe fn WP_SaberBlockNonRandom(
    self_: *mut gentity_t,
    hitloc: &vec3_t,
    missileBlock: qboolean,
) {
    let mut diff: vec3_t = [0.0; 3];
    let mut fwdangles: vec3_t = [0.0, 0.0, 0.0];
    let mut right: vec3_t = [0.0; 3];
    let mut clEye: vec3_t = [0.0; 3];
    let rightdot: f32;
    let zdiff: f32;

    VectorCopy(&(*(*self_).client).ps.origin, &mut clEye);
    clEye[2] += (*(*self_).client).ps.viewheight as f32;

    VectorSubtract(hitloc, &clEye, &mut diff);
    diff[2] = 0.0;
    VectorNormalize(&mut diff);

    fwdangles[1] = (*(*self_).client).ps.viewangles[1];
    // Ultimately we might care if the shot was ahead or behind, but for now, just quadrant is fine.
    AngleVectors(&fwdangles, None, Some(&mut right), None);

    rightdot = DotProduct(&right, &diff);
    zdiff = hitloc[2] - clEye[2];

    if zdiff > 0.0 {
        if rightdot > 0.3 {
            (*(*self_).client).ps.saberBlocked = BLOCKED_UPPER_RIGHT;
        } else if rightdot < -0.3 {
            (*(*self_).client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
        } else {
            (*(*self_).client).ps.saberBlocked = BLOCKED_TOP;
        }
    } else if zdiff > -20.0
    //20 )
    {
        if zdiff < -10.0
        //30 )
        {
            //hmm, pretty low, but not low enough to use the low block, so we need to duck
        }
        if rightdot > 0.1 {
            (*(*self_).client).ps.saberBlocked = BLOCKED_UPPER_RIGHT;
        } else if rightdot < -0.1 {
            (*(*self_).client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
        } else {
            (*(*self_).client).ps.saberBlocked = BLOCKED_TOP;
        }
    } else if rightdot >= 0.0 {
        (*(*self_).client).ps.saberBlocked = BLOCKED_LOWER_RIGHT;
    } else {
        (*(*self_).client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
    }

    if missileBlock != QFALSE {
        (*(*self_).client).ps.saberBlocked =
            WP_MissileBlockForBlock((*(*self_).client).ps.saberBlocked);
    }
}

/// `void WP_SaberBlock( gentity_t *playerent, vec3_t hitloc, qboolean missileBlock )`
/// (w_saber.c:8753) — the random-jitter version of [`WP_SaberBlockNonRandom`]: pick the
/// `ps.saberBlocked` quadrant for a hit at `hitloc`, adding `RandFloat`/`Q_irand` slop so repeated
/// blocks vary. Builds the direction from the client's `ps.origin` (note: no viewheight offset,
/// unlike the non-random variant) to `hitloc`, projects onto the view `right` for `rightdot` (plus
/// `±0.2` jitter), and uses `zdiff` (plus `±8` jitter) to split into above / upper-three-block /
/// lower regions, choosing among `BLOCKED_*` quadrants via `Q_irand`. When `missileBlock` is set the
/// result is mapped to its projectile-reflect variant via [`WP_MissileBlockForBlock`]. Faithful to
/// the C, including the comments on quadrant coverage.
///
/// # Safety
/// Dereferences `playerent` and its `client`; the caller must pass a valid client entity.
pub unsafe fn WP_SaberBlock(playerent: *mut gentity_t, hitloc: &vec3_t, missileBlock: qboolean) {
    let mut diff: vec3_t = [0.0; 3];
    let mut fwdangles: vec3_t = [0.0, 0.0, 0.0];
    let mut right: vec3_t = [0.0; 3];
    let rightdot: f32;
    let zdiff: f32;

    VectorSubtract(hitloc, &(*(*playerent).client).ps.origin, &mut diff);
    VectorNormalize(&mut diff);

    fwdangles[1] = (*(*playerent).client).ps.viewangles[1];
    // Ultimately we might care if the shot was ahead or behind, but for now, just quadrant is fine.
    AngleVectors(&fwdangles, None, Some(&mut right), None);

    rightdot = DotProduct(&right, &diff) + RandFloat(-0.2f32, 0.2f32);
    zdiff = hitloc[2] - (*(*playerent).client).ps.origin[2] + Q_irand(-8, 8) as f32;

    // Figure out what quadrant the block was in.
    if zdiff > 24.0 {
        // Attack from above
        if Q_irand(0, 1) != 0 {
            (*(*playerent).client).ps.saberBlocked = BLOCKED_TOP;
        } else {
            (*(*playerent).client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
        }
    } else if zdiff > 13.0 {
        // The upper half has three viable blocks...
        if rightdot > 0.25 {
            // In the right quadrant...
            if Q_irand(0, 1) != 0 {
                (*(*playerent).client).ps.saberBlocked = BLOCKED_UPPER_LEFT;
            } else {
                (*(*playerent).client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
            }
        } else {
            match Q_irand(0, 3) {
                0 => {
                    (*(*playerent).client).ps.saberBlocked = BLOCKED_UPPER_RIGHT;
                }
                1 | 2 => {
                    (*(*playerent).client).ps.saberBlocked = BLOCKED_LOWER_RIGHT;
                }
                3 => {
                    (*(*playerent).client).ps.saberBlocked = BLOCKED_TOP;
                }
                _ => {}
            }
        }
    } else {
        // The lower half is a bit iffy as far as block coverage.  Pick one of the "low" ones at random.
        if Q_irand(0, 1) != 0 {
            (*(*playerent).client).ps.saberBlocked = BLOCKED_LOWER_RIGHT;
        } else {
            (*(*playerent).client).ps.saberBlocked = BLOCKED_LOWER_LEFT;
        }
    }

    if missileBlock != QFALSE {
        (*(*playerent).client).ps.saberBlocked =
            WP_MissileBlockForBlock((*(*playerent).client).ps.saberBlocked);
    }
}

/// `int WP_SaberCanBlock(gentity_t *self, vec3_t point, int dflags, int mod, qboolean projectile,
/// int attackStr)` (w_saber.c:8829) — the saber auto-block keystone: decide whether `self` blocks an
/// incoming hit at `point` and, for projectiles, kick off the block animation. Returns 1 if blocked,
/// 0 otherwise. A barrage of early-outs reject blocking when the player is mid-attack, in a
/// broken-parry/non-defensive saberMove, has the saber knocked away or in flight, sabers off, wrong
/// weapon, raising the weapon, holding +attack, attacking, not in a blocking move, still inside the
/// block cooldown (`saberBlockTime >= level.time`), or doing a force-hand gesture. `attackStr == 999`
/// is the thrown-saber sentinel (clears `attackStr`, sets `thrownSaber`). The big commented-out
/// pre-1.03 block-chance logic is carried over verbatim. `blockFactor` (the `InFront` facing
/// threshold) scales with `FP_SABER_DEFENSE` level — note level 3 keys off `d_saberGhoul2Collision`
/// — with no-defense returning 0 outright; thrown sabers and saber-vs-saber attacks each shave 0.25.
/// If still facing the hit, a projectile triggers [`WP_SaberBlockNonRandom`] and we return 1.
/// Faithful to the C; `dflags`/`mod` are unused parameters (kept for ABI parity, as in the C).
///
/// # Safety
/// Dereferences `self` and its `client`/`pers`, and reads cvar/level globals; the caller must pass a
/// valid client entity and a valid `point`.
#[allow(unused_assignments)] // faithful `float blockFactor = 0;` init — every read path reassigns it first
pub unsafe fn WP_SaberCanBlock(
    self_: *mut gentity_t,
    point: &vec3_t,
    _dflags: c_int,
    _mod: c_int,
    projectile: qboolean,
    mut attackStr: c_int,
) -> c_int {
    let mut thrownSaber: qboolean = QFALSE;
    let mut blockFactor: f32 = 0.0;

    if self_.is_null() || (*self_).client.is_null() {
        return 0;
    }

    if attackStr == 999 {
        attackStr = 0;
        thrownSaber = QTRUE;
    }

    if BG_SaberInAttack((*(*self_).client).ps.saberMove) == QTRUE {
        return 0;
    }

    if PM_InSaberAnim((*(*self_).client).ps.torsoAnim) == QTRUE
        && (*(*self_).client).ps.saberBlocked == 0
        && (*(*self_).client).ps.saberMove != LS_READY
        && (*(*self_).client).ps.saberMove != LS_NONE
    {
        if (*(*self_).client).ps.saberMove < LS_PARRY_UP
            || (*(*self_).client).ps.saberMove > LS_REFLECT_LL
        {
            return 0;
        }
    }

    if PM_SaberInBrokenParry((*(*self_).client).ps.saberMove) == QTRUE {
        return 0;
    }

    if (*(*self_).client).ps.saberEntityNum == 0 {
        //saber is knocked away
        return 0;
    }

    if BG_SabersOff(&mut (*(*self_).client).ps) == QTRUE {
        return 0;
    }

    if (*(*self_).client).ps.weapon != WP_SABER {
        return 0;
    }

    if (*(*self_).client).ps.weaponstate == WEAPON_RAISING {
        return 0;
    }

    if (*(*self_).client).ps.saberInFlight == QTRUE {
        return 0;
    }

    if (*(*self_).client).pers.cmd.buttons & BUTTON_ATTACK != 0
    /* &&
    (projectile || attackStr == FORCE_LEVEL_3)*/
    {
        //don't block when the player is trying to slash, if it's a projectile or he's doing a very strong attack
        return 0;
    }

    //Removed this for now, the new broken parry stuff should handle it. This is how
    //blocks were decided before the 1.03 patch (as you can see, it was STUPID.. for the most part)
    /*
    if (attackStr == FORCE_LEVEL_3)
    {
        if (self->client->ps.fd.forcePowerLevel[FP_SABER_DEFENSE] >= FORCE_LEVEL_3)
        {
            if (Q_irand(1, 10) < 3)
            {
                return 0;
            }
        }
        else
        {
            return 0;
        }
    }

    if (attackStr == FORCE_LEVEL_2 && Q_irand(1, 10) < 3)
    {
        if (self->client->ps.fd.forcePowerLevel[FP_SABER_DEFENSE] >= FORCE_LEVEL_3)
        {
            //do nothing for now
        }
        else if (self->client->ps.fd.forcePowerLevel[FP_SABER_DEFENSE] >= FORCE_LEVEL_2)
        {
            if (Q_irand(1, 10) < 5)
            {
                return 0;
            }
        }
        else
        {
            return 0;
        }
    }

    if (attackStr == FORCE_LEVEL_1 && !self->client->ps.fd.forcePowerLevel[FP_SABER_DEFENSE] &&
        Q_irand(1, 40) < 3)
    { //if I have no defense level at all then I might be unable to block a level 1 attack (but very rarely)
        return 0;
    }
    */

    if SaberAttacking(self_) == QTRUE {
        //attacking, can't block now
        return 0;
    }

    if (*(*self_).client).ps.saberMove != LS_READY && (*(*self_).client).ps.saberBlocking == 0 {
        return 0;
    }

    if (*(*self_).client).ps.saberBlockTime >= (*addr_of!(level)).time {
        return 0;
    }

    if (*(*self_).client).ps.forceHandExtend != HANDEXTEND_NONE {
        return 0;
    }

    if (*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] == FORCE_LEVEL_3 {
        if (*addr_of!(d_saberGhoul2Collision)).integer != 0 {
            blockFactor = 0.3f32;
        } else {
            blockFactor = 0.05f32;
        }
    } else if (*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] == FORCE_LEVEL_2 {
        blockFactor = 0.6f32;
    } else if (*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] == FORCE_LEVEL_1 {
        blockFactor = 0.9f32;
    } else {
        //for now we just don't get to autoblock with no def
        return 0;
    }

    if thrownSaber == QTRUE {
        blockFactor -= 0.25f32;
    }

    if attackStr != 0 {
        //blocking a saber, not a projectile.
        blockFactor -= 0.25f32;
    }

    if InFront(
        point,
        &(*(*self_).client).ps.origin,
        &(*(*self_).client).ps.viewangles,
        blockFactor,
    ) == QFALSE
    //orig 0.2f
    {
        return 0;
    }

    if projectile == QTRUE {
        WP_SaberBlockNonRandom(self_, point, projectile);
    }
    1
}

/// `int G_KnockawayForParry(int move)` (w_saber.c:2083) — maps a `LS_PARRY_*` parry move to the
/// `LS_K1_*` knockaway animation that pushes the saber away from center. Pure switch over the
/// `saberMoveName_t` parry enum; `LS_PARRY_UR` shares the `default` arm (the C comment notes it
/// also covers `LS_READY`), so anything not one of the four other parries yields `LS_K1_TR`.
/// Faithful to the C (the redundant `break`s after each `return` are dropped — unreachable in
/// Rust; the C has no terminal return, relying on `default` to cover all paths).
pub fn G_KnockawayForParry(r#move: c_int) -> c_int {
    match r#move {
        LS_PARRY_UP => LS_K1_T_, // push up
        LS_PARRY_UL => LS_K1_TL, // push up and to left
        LS_PARRY_LR => LS_K1_BR, // push down and to left
        LS_PARRY_LL => LS_K1_BL, // push down and to right
        _ => LS_K1_TR,           // LS_PARRY_UR / default: push up, slightly to right
    }
}

/// `int G_SaberLockAnim(int attackerSaberStyle, int defenderSaberStyle, int topOrSide,
/// int lockOrBreakOrSuperBreak, int winOrLose)` (w_saber.c:967) — pick the `BOTH_LK_*` saber-lock
/// animation for a lock/break/superbreak pose, given each combatant's saber style
/// (`saber_styles_t`), whether the lock is top or side, the lock phase
/// (`SABERLOCK_LOCK`/`BREAK`/`SUPERBREAK`), and whether this entity wins or loses.
///
/// Special case first: when both fighters use the *same* style family and we're in the LOCK phase
/// and this entity LOSEs, the anim mirrors the *defender's* stance (dual / staff / single), keyed
/// by top-vs-side. Note the "same style" test treats every single-saber style (`SS_FAST`..
/// `SS_TAVION`) as equivalent. Otherwise `baseAnim` is the loser-side break anim for the
/// attacker×defender style pair, then nudged by fixed offsets: `+5` for a top lock, `+2` for a
/// LOCK (vs. `+3` for a SUPERBREAK), and `+1` for the winner of a break/superbreak. Pure
/// `int`→`int` switch math, faithful to the C.
pub fn G_SaberLockAnim(
    attackerSaberStyle: c_int,
    defenderSaberStyle: c_int,
    topOrSide: c_int,
    lockOrBreakOrSuperBreak: c_int,
    winOrLose: c_int,
) -> c_int {
    let mut baseAnim: c_int = -1;
    if lockOrBreakOrSuperBreak == SABERLOCK_LOCK {
        // special case: if we're using the same style and locking
        if attackerSaberStyle == defenderSaberStyle
            || (attackerSaberStyle >= SS_FAST
                && attackerSaberStyle <= SS_TAVION
                && defenderSaberStyle >= SS_FAST
                && defenderSaberStyle <= SS_TAVION)
        {
            // using same style
            if winOrLose == SABERLOCK_LOSE {
                // you want the defender's stance...
                match defenderSaberStyle {
                    SS_DUAL => {
                        if topOrSide == SABERLOCK_TOP {
                            baseAnim = BOTH_LK_DL_DL_T_L_2;
                        } else {
                            baseAnim = BOTH_LK_DL_DL_S_L_2;
                        }
                    }
                    SS_STAFF => {
                        if topOrSide == SABERLOCK_TOP {
                            baseAnim = BOTH_LK_ST_ST_T_L_2;
                        } else {
                            baseAnim = BOTH_LK_ST_ST_S_L_2;
                        }
                    }
                    _ => {
                        if topOrSide == SABERLOCK_TOP {
                            baseAnim = BOTH_LK_S_S_T_L_2;
                        } else {
                            baseAnim = BOTH_LK_S_S_S_L_2;
                        }
                    }
                }
            }
        }
    }
    if baseAnim == -1 {
        match attackerSaberStyle {
            SS_DUAL => match defenderSaberStyle {
                SS_DUAL => baseAnim = BOTH_LK_DL_DL_S_B_1_L,
                SS_STAFF => baseAnim = BOTH_LK_DL_ST_S_B_1_L,
                _ => baseAnim = BOTH_LK_DL_S_S_B_1_L, // single
            },
            SS_STAFF => match defenderSaberStyle {
                SS_DUAL => baseAnim = BOTH_LK_ST_DL_S_B_1_L,
                SS_STAFF => baseAnim = BOTH_LK_ST_ST_S_B_1_L,
                _ => baseAnim = BOTH_LK_ST_S_S_B_1_L, // single
            },
            _ => match defenderSaberStyle {
                // single
                SS_DUAL => baseAnim = BOTH_LK_S_DL_S_B_1_L,
                SS_STAFF => baseAnim = BOTH_LK_S_ST_S_B_1_L,
                _ => baseAnim = BOTH_LK_S_S_S_B_1_L, // single
            },
        }
        // side lock or top lock?
        if topOrSide == SABERLOCK_TOP {
            baseAnim += 5;
        }
        // lock, break or superbreak?
        if lockOrBreakOrSuperBreak == SABERLOCK_LOCK {
            baseAnim += 2;
        } else {
            // a break or superbreak
            if lockOrBreakOrSuperBreak == SABERLOCK_SUPERBREAK {
                baseAnim += 3;
            }
            // winner or loser?
            if winOrLose == SABERLOCK_WIN {
                baseAnim += 1;
            }
        }
    }
    baseAnim
}

// sabersLockMode_t (w_saber.c:948) — which saber-lock pose to set up.
const LOCK_FIRST: c_int = 0;
const LOCK_TOP: c_int = LOCK_FIRST;
const LOCK_DIAG_TR: c_int = LOCK_TOP + 1;
const LOCK_DIAG_TL: c_int = LOCK_DIAG_TR + 1;
const LOCK_DIAG_BR: c_int = LOCK_DIAG_TL + 1;
const LOCK_DIAG_BL: c_int = LOCK_DIAG_BR + 1;
const LOCK_R: c_int = LOCK_DIAG_BL + 1;
const LOCK_L: c_int = LOCK_R + 1;
const LOCK_RANDOM: c_int = LOCK_L + 1;

const LOCK_IDEAL_DIST_TOP: f32 = 32.0; // w_saber.c:961
const LOCK_IDEAL_DIST_CIRCLE: f32 = 48.0; // w_saber.c:962
                                          // all of the new saberlocks are 46.08 from each other because Richard Lico is da MAN
const LOCK_IDEAL_DIST_JKA: f32 = 46.0; // w_saber.c:1089

/// `static qboolean WP_SabersCheckLock2( gentity_t *attacker, gentity_t *defender,
/// sabersLockMode_t lockMode )` (w_saber.c:1091) — set up a saber lock between `attacker` and
/// `defender` in the requested `lockMode` (or a random one when `LOCK_RANDOM`). Picks each
/// fighter's lock animation and its start phase: two single-saber styles use the legacy
/// `BOTH_*LOCK` butterfly/circle anims; everything else uses the JKA system via
/// [`G_SaberLockAnim`] with start phases nudged by [`BG_CheckIncrementLockAnim`]. Then drives
/// both fighters into those anims with [`G_SetAnim`], seeds the `saberLock*` playerstate fields
/// (frame from the anim's `firstFrame`+`numFrames*start`, zeroed hits, no advance, 10s lock
/// timeout, mutual enemy nums, a 1–3s weapon delay before the first push), faces them at each
/// other via [`SetClientViewAngle`], and finally pulls them to `idealDist` apart with two
/// [`trap::Trace`]/[`G_SetOrigin`]/[`trap::LinkEntity`] moves. Returns `qfalse` for an
/// unsupported `lockMode`, else `qtrue`. Faithful to the C.
///
/// # Safety
/// Dereferences both entities (and their `client`s) and reads the global animation table;
/// callers must pass valid client entities.
pub unsafe fn WP_SabersCheckLock2(
    attacker: *mut gentity_t,
    defender: *mut gentity_t,
    mut lockMode: c_int,
) -> qboolean {
    // C: `int attAnim, defAnim = 0;` / `float attStart = 0.5f, defStart = 0.5f;`
    // `float idealDist = 48.0f;` — every reaching path overwrites these before use, so they are
    // left deferred-init here (the dead-store initializers would only trip the
    // unused-assignment lint).
    let attAnim: c_int;
    let defAnim: c_int;
    let attStart: f32;
    let defStart: f32;
    let idealDist: f32;
    let mut attAngles: vec3_t = [0.0; 3];
    let mut defAngles: vec3_t = [0.0; 3];
    let mut defDir: vec3_t = [0.0; 3];
    let mut newOrg: vec3_t = [0.0; 3];
    let mut attDir: vec3_t = [0.0; 3];
    let mut diff: f32;
    let trace: trace_t;

    //MATCH ANIMS
    if lockMode == LOCK_RANDOM {
        lockMode = Q_irand(LOCK_FIRST, LOCK_RANDOM - 1);
    }
    if (*(*attacker).client).ps.fd.saberAnimLevel >= SS_FAST
        && (*(*attacker).client).ps.fd.saberAnimLevel <= SS_TAVION
        && (*(*defender).client).ps.fd.saberAnimLevel >= SS_FAST
        && (*(*defender).client).ps.fd.saberAnimLevel <= SS_TAVION
    {
        //2 single sabers?  Just do it the old way...
        match lockMode {
            LOCK_TOP => {
                attAnim = BOTH_BF2LOCK;
                defAnim = BOTH_BF1LOCK;
                attStart = 0.5;
                defStart = 0.5;
                idealDist = LOCK_IDEAL_DIST_TOP;
            }
            LOCK_DIAG_TR => {
                attAnim = BOTH_CCWCIRCLELOCK;
                defAnim = BOTH_CWCIRCLELOCK;
                attStart = 0.5;
                defStart = 0.5;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            }
            LOCK_DIAG_TL => {
                attAnim = BOTH_CWCIRCLELOCK;
                defAnim = BOTH_CCWCIRCLELOCK;
                attStart = 0.5;
                defStart = 0.5;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            }
            LOCK_DIAG_BR => {
                attAnim = BOTH_CWCIRCLELOCK;
                defAnim = BOTH_CCWCIRCLELOCK;
                attStart = 0.85;
                defStart = 0.85;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            }
            LOCK_DIAG_BL => {
                attAnim = BOTH_CCWCIRCLELOCK;
                defAnim = BOTH_CWCIRCLELOCK;
                attStart = 0.85;
                defStart = 0.85;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            }
            LOCK_R => {
                attAnim = BOTH_CCWCIRCLELOCK;
                defAnim = BOTH_CWCIRCLELOCK;
                attStart = 0.75;
                defStart = 0.75;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            }
            LOCK_L => {
                attAnim = BOTH_CWCIRCLELOCK;
                defAnim = BOTH_CCWCIRCLELOCK;
                attStart = 0.75;
                defStart = 0.75;
                idealDist = LOCK_IDEAL_DIST_CIRCLE;
            }
            _ => {
                return QFALSE;
            }
        }
    } else {
        //use the new system
        idealDist = LOCK_IDEAL_DIST_JKA; //all of the new saberlocks are 46.08 from each other because Richard Lico is da MAN
        if lockMode == LOCK_TOP {
            //top lock
            attAnim = G_SaberLockAnim(
                (*(*attacker).client).ps.fd.saberAnimLevel,
                (*(*defender).client).ps.fd.saberAnimLevel,
                SABERLOCK_TOP,
                SABERLOCK_LOCK,
                SABERLOCK_WIN,
            );
            defAnim = G_SaberLockAnim(
                (*(*defender).client).ps.fd.saberAnimLevel,
                (*(*attacker).client).ps.fd.saberAnimLevel,
                SABERLOCK_TOP,
                SABERLOCK_LOCK,
                SABERLOCK_LOSE,
            );
            attStart = 0.5;
            defStart = 0.5;
        } else {
            //side lock
            match lockMode {
                LOCK_DIAG_TR => {
                    attAnim = G_SaberLockAnim(
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    defAnim = G_SaberLockAnim(
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    attStart = 0.5;
                    defStart = 0.5;
                }
                LOCK_DIAG_TL => {
                    attAnim = G_SaberLockAnim(
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    defAnim = G_SaberLockAnim(
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    attStart = 0.5;
                    defStart = 0.5;
                }
                LOCK_DIAG_BR => {
                    attAnim = G_SaberLockAnim(
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    defAnim = G_SaberLockAnim(
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    if BG_CheckIncrementLockAnim(attAnim, SABERLOCK_WIN) != QFALSE {
                        attStart = 0.85; //move to end of anim
                    } else {
                        attStart = 0.15; //start at beginning of anim
                    }
                    if BG_CheckIncrementLockAnim(defAnim, SABERLOCK_LOSE) != QFALSE {
                        defStart = 0.85; //start at end of anim
                    } else {
                        defStart = 0.15; //start at beginning of anim
                    }
                }
                LOCK_DIAG_BL => {
                    attAnim = G_SaberLockAnim(
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    defAnim = G_SaberLockAnim(
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    if BG_CheckIncrementLockAnim(attAnim, SABERLOCK_WIN) != QFALSE {
                        attStart = 0.85; //move to end of anim
                    } else {
                        attStart = 0.15; //start at beginning of anim
                    }
                    if BG_CheckIncrementLockAnim(defAnim, SABERLOCK_LOSE) != QFALSE {
                        defStart = 0.85; //start at end of anim
                    } else {
                        defStart = 0.15; //start at beginning of anim
                    }
                }
                LOCK_R => {
                    attAnim = G_SaberLockAnim(
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    defAnim = G_SaberLockAnim(
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    if BG_CheckIncrementLockAnim(attAnim, SABERLOCK_WIN) != QFALSE {
                        attStart = 0.75; //move to end of anim
                    } else {
                        attStart = 0.25; //start at beginning of anim
                    }
                    if BG_CheckIncrementLockAnim(defAnim, SABERLOCK_LOSE) != QFALSE {
                        defStart = 0.75; //start at end of anim
                    } else {
                        defStart = 0.25; //start at beginning of anim
                    }
                }
                LOCK_L => {
                    attAnim = G_SaberLockAnim(
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_WIN,
                    );
                    defAnim = G_SaberLockAnim(
                        (*(*defender).client).ps.fd.saberAnimLevel,
                        (*(*attacker).client).ps.fd.saberAnimLevel,
                        SABERLOCK_SIDE,
                        SABERLOCK_LOCK,
                        SABERLOCK_LOSE,
                    );
                    //attacker starts with advantage
                    if BG_CheckIncrementLockAnim(attAnim, SABERLOCK_WIN) != QFALSE {
                        attStart = 0.75; //move to end of anim
                    } else {
                        attStart = 0.25; //start at beginning of anim
                    }
                    if BG_CheckIncrementLockAnim(defAnim, SABERLOCK_LOSE) != QFALSE {
                        defStart = 0.75; //start at end of anim
                    } else {
                        defStart = 0.25; //start at beginning of anim
                    }
                }
                _ => {
                    return QFALSE;
                }
            }
        }
    }

    G_SetAnim(
        attacker,
        null_mut(),
        SETANIM_BOTH,
        attAnim,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        0,
    );
    let attAnimData = (*addr_of!(bgAllAnims))[(*attacker).localAnimIndex as usize]
        .anims
        .add(attAnim as usize);
    (*(*attacker).client).ps.saberLockFrame =
        ((*attAnimData).firstFrame as f32 + ((*attAnimData).numFrames as f32 * attStart)) as c_int;

    G_SetAnim(
        defender,
        null_mut(),
        SETANIM_BOTH,
        defAnim,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        0,
    );
    let defAnimData = (*addr_of!(bgAllAnims))[(*defender).localAnimIndex as usize]
        .anims
        .add(defAnim as usize);
    (*(*defender).client).ps.saberLockFrame =
        ((*defAnimData).firstFrame as f32 + ((*defAnimData).numFrames as f32 * defStart)) as c_int;

    (*(*attacker).client).ps.saberLockHits = 0;
    (*(*defender).client).ps.saberLockHits = 0;

    (*(*attacker).client).ps.saberLockAdvance = QFALSE;
    (*(*defender).client).ps.saberLockAdvance = QFALSE;

    VectorClear(&mut (*(*attacker).client).ps.velocity);
    VectorClear(&mut (*(*defender).client).ps.velocity);
    let lockTime = (*addr_of!(level)).time + 10000;
    (*(*attacker).client).ps.saberLockTime = lockTime;
    (*(*defender).client).ps.saberLockTime = lockTime;
    (*(*attacker).client).ps.saberLockEnemy = (*defender).s.number;
    (*(*defender).client).ps.saberLockEnemy = (*attacker).s.number;
    let weaponTime = Q_irand(1000, 3000); //delay 1 to 3 seconds before pushing
    (*(*attacker).client).ps.weaponTime = weaponTime;
    (*(*defender).client).ps.weaponTime = weaponTime;

    VectorSubtract(
        &(*defender).r.currentOrigin,
        &(*attacker).r.currentOrigin,
        &mut defDir,
    );
    VectorCopy(&(*(*attacker).client).ps.viewangles, &mut attAngles);
    attAngles[YAW] = vectoyaw(&defDir);
    SetClientViewAngle(attacker, &attAngles);
    defAngles[PITCH] = attAngles[PITCH] * -1.0;
    defAngles[YAW] = AngleNormalize180(attAngles[YAW] + 180.0);
    defAngles[ROLL] = 0.0;
    SetClientViewAngle(defender, &defAngles);

    //MATCH POSITIONS
    diff = VectorNormalize(&mut defDir) - idealDist; //diff will be the total error in dist
                                                     //try to move attacker half the diff towards the defender
    VectorMA(
        &(*attacker).r.currentOrigin,
        diff * 0.5,
        &defDir,
        &mut newOrg,
    );

    trace = trap::Trace(
        &(*attacker).r.currentOrigin,
        &(*attacker).r.mins,
        &(*attacker).r.maxs,
        &newOrg,
        (*attacker).s.number,
        (*attacker).clipmask,
    );
    if trace.startsolid == 0 && trace.allsolid == 0 {
        G_SetOrigin(attacker, &trace.endpos);
        if !(*attacker).client.is_null() {
            VectorCopy(&trace.endpos, &mut (*(*attacker).client).ps.origin);
        }
        trap::LinkEntity(attacker);
    }
    //now get the defender's dist and do it for him too
    VectorSubtract(
        &(*attacker).r.currentOrigin,
        &(*defender).r.currentOrigin,
        &mut attDir,
    );
    diff = VectorNormalize(&mut attDir) - idealDist; //diff will be the total error in dist
                                                     //try to move defender all of the remaining diff towards the attacker
    VectorMA(&(*defender).r.currentOrigin, diff, &attDir, &mut newOrg);
    let trace = trap::Trace(
        &(*defender).r.currentOrigin,
        &(*defender).r.mins,
        &(*defender).r.maxs,
        &newOrg,
        (*defender).s.number,
        (*defender).clipmask,
    );
    if trace.startsolid == 0 && trace.allsolid == 0 {
        if !(*defender).client.is_null() {
            VectorCopy(&trace.endpos, &mut (*(*defender).client).ps.origin);
        }
        G_SetOrigin(defender, &trace.endpos);
        trap::LinkEntity(defender);
    }

    //DONE!
    QTRUE
}

/// `qboolean WP_SabersCheckLock( gentity_t *ent1, gentity_t *ent2 )` (w_saber.c:1335) — the
/// MP saber-lock entry point. Decides whether the two clients' current attacks should clash
/// into a saber lock and, if so, dispatches to [`WP_SabersCheckLock2`] with the appropriate
/// [`sabersLockMode_t`] pose. Runs a long gauntlet of disqualifiers first (powerduel, the
/// `g_saberLocking` cvar, both must be real lockable clients in-hand and not in flight, dueling
/// each other directly outside duel modes, within Z and 8..80 unit range, on the ground, not in
/// special-jump/roll/hand-extend/duck), then classifies `ent1`/`ent2`'s `torsoAnim` swing
/// direction (T→B, the four diagonals, L→R, R→L) — honoring a player-blocking-wide shortcut
/// (`BLK_WIDE`, weaponTime<=0, `s.number==0`) — and picks the lock pose. The `g_debugSaberLocks`
/// path forces a `LOCK_RANDOM`. No oracle — walks `gentity_t`/`gclient_t` and reads cvars/`level`.
///
/// # Safety
/// Dereferences `ent1`/`ent2` and their `client`s; callers must pass valid client entities.
pub unsafe fn WP_SabersCheckLock(ent1: *mut gentity_t, ent2: *mut gentity_t) -> qboolean {
    let mut ent1BlockingPlayer: qboolean = QFALSE;
    let mut ent2BlockingPlayer: qboolean = QFALSE;

    if (*addr_of!(g_debugSaberLocks)).integer != 0 {
        WP_SabersCheckLock2(ent1, ent2, LOCK_RANDOM);
        return QTRUE;
    }
    //for now.. it's not fair to the lone duelist.
    //we need dual saber lock animations.
    if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
        return QFALSE;
    }

    if (*addr_of!(g_saberLocking)).integer == 0 {
        return QFALSE;
    }

    if (*ent1).client.is_null() || (*ent2).client.is_null() {
        return QFALSE;
    }

    if (*ent1).s.eType == ET_NPC || (*ent2).s.eType == ET_NPC {
        //if either ents is NPC, then never let an NPC lock with someone on the same playerTeam
        if (*(*ent1).client).playerTeam == (*(*ent2).client).playerTeam {
            return QFALSE;
        }
    }

    if (*(*ent1).client).ps.saberEntityNum == 0
        || (*(*ent2).client).ps.saberEntityNum == 0
        || (*(*ent1).client).ps.saberInFlight != QFALSE
        || (*(*ent2).client).ps.saberInFlight != QFALSE
    {
        //can't get in lock if one of them has had the saber knocked out of his hand
        return QFALSE;
    }

    if (*ent1).s.eType != ET_NPC && (*ent2).s.eType != ET_NPC {
        //can always get into locks with NPCs
        if (*(*ent1).client).ps.duelInProgress == QFALSE
            || (*(*ent2).client).ps.duelInProgress == QFALSE
            || (*(*ent1).client).ps.duelIndex != (*ent2).s.number
            || (*(*ent2).client).ps.duelIndex != (*ent1).s.number
        {
            //only allow saber locking if two players are dueling with each other directly
            if (*addr_of!(g_gametype)).integer != GT_DUEL
                && (*addr_of!(g_gametype)).integer != GT_POWERDUEL
            {
                return QFALSE;
            }
        }
    }

    if ((*ent1).r.currentOrigin[2] - (*ent2).r.currentOrigin[2]).abs() > 16.0 {
        return QFALSE;
    }
    if (*(*ent1).client).ps.groundEntityNum == ENTITYNUM_NONE
        || (*(*ent2).client).ps.groundEntityNum == ENTITYNUM_NONE
    {
        return QFALSE;
    }
    let dist = DistanceSquared(&(*ent1).r.currentOrigin, &(*ent2).r.currentOrigin);
    if dist < 64.0 || dist > 6400.0 {
        //between 8 and 80 from each other
        return QFALSE;
    }

    if BG_InSpecialJump((*(*ent1).client).ps.legsAnim) == QTRUE {
        return QFALSE;
    }
    if BG_InSpecialJump((*(*ent2).client).ps.legsAnim) == QTRUE {
        return QFALSE;
    }

    if BG_InRoll(
        addr_of_mut!((*(*ent1).client).ps),
        (*(*ent1).client).ps.legsAnim,
    ) == QTRUE
    {
        return QFALSE;
    }
    if BG_InRoll(
        addr_of_mut!((*(*ent2).client).ps),
        (*(*ent2).client).ps.legsAnim,
    ) == QTRUE
    {
        return QFALSE;
    }

    if (*(*ent1).client).ps.forceHandExtend != HANDEXTEND_NONE
        || (*(*ent2).client).ps.forceHandExtend != HANDEXTEND_NONE
    {
        return QFALSE;
    }

    if ((*(*ent1).client).ps.pm_flags & PMF_DUCKED) != 0
        || ((*(*ent2).client).ps.pm_flags & PMF_DUCKED) != 0
    {
        return QFALSE;
    }

    if (*(*ent1).client).saber[0].saberFlags & SFL_NOT_LOCKABLE != 0
        || (*(*ent2).client).saber[0].saberFlags & SFL_NOT_LOCKABLE != 0
    {
        return QFALSE;
    }
    if (*(*ent1).client).saber[1].model[0] != 0
        && (*(*ent1).client).ps.saberHolstered == 0
        && (*(*ent1).client).saber[1].saberFlags & SFL_NOT_LOCKABLE != 0
    {
        return QFALSE;
    }
    if (*(*ent2).client).saber[1].model[0] != 0
        && (*(*ent2).client).ps.saberHolstered == 0
        && (*(*ent2).client).saber[1].saberFlags & SFL_NOT_LOCKABLE != 0
    {
        return QFALSE;
    }

    if InFront(
        &(*(*ent1).client).ps.origin,
        &(*(*ent2).client).ps.origin,
        &(*(*ent2).client).ps.viewangles,
        0.4,
    ) == QFALSE
    {
        return QFALSE;
    }
    if InFront(
        &(*(*ent2).client).ps.origin,
        &(*(*ent1).client).ps.origin,
        &(*(*ent1).client).ps.viewangles,
        0.4,
    ) == QFALSE
    {
        return QFALSE;
    }

    let ent1Anim = (*(*ent1).client).ps.torsoAnim;
    let ent2Anim = (*(*ent2).client).ps.torsoAnim;

    //T to B lock
    if ent1Anim == BOTH_A1_T__B_
        || ent1Anim == BOTH_A2_T__B_
        || ent1Anim == BOTH_A3_T__B_
        || ent1Anim == BOTH_A4_T__B_
        || ent1Anim == BOTH_A5_T__B_
        || ent1Anim == BOTH_A6_T__B_
        || ent1Anim == BOTH_A7_T__B_
    {
        //ent1 is attacking top-down
        return WP_SabersCheckLock2(ent1, ent2, LOCK_TOP);
    }

    if ent2Anim == BOTH_A1_T__B_
        || ent2Anim == BOTH_A2_T__B_
        || ent2Anim == BOTH_A3_T__B_
        || ent2Anim == BOTH_A4_T__B_
        || ent2Anim == BOTH_A5_T__B_
        || ent2Anim == BOTH_A6_T__B_
        || ent2Anim == BOTH_A7_T__B_
    {
        //ent2 is attacking top-down
        return WP_SabersCheckLock2(ent2, ent1, LOCK_TOP);
    }

    if (*ent1).s.number == 0
        && (*(*ent1).client).ps.saberBlocking == BLK_WIDE
        && (*(*ent1).client).ps.weaponTime <= 0
    {
        ent1BlockingPlayer = QTRUE;
    }
    if (*ent2).s.number == 0
        && (*(*ent2).client).ps.saberBlocking == BLK_WIDE
        && (*(*ent2).client).ps.weaponTime <= 0
    {
        ent2BlockingPlayer = QTRUE;
    }

    //TR to BL lock
    if ent1Anim == BOTH_A1_TR_BL
        || ent1Anim == BOTH_A2_TR_BL
        || ent1Anim == BOTH_A3_TR_BL
        || ent1Anim == BOTH_A4_TR_BL
        || ent1Anim == BOTH_A5_TR_BL
        || ent1Anim == BOTH_A6_TR_BL
        || ent1Anim == BOTH_A7_TR_BL
    {
        //ent1 is attacking diagonally
        if ent2BlockingPlayer == QTRUE {
            //player will block this anyway
            return WP_SabersCheckLock2(ent1, ent2, LOCK_DIAG_TR);
        }
        if ent2Anim == BOTH_A1_TR_BL
            || ent2Anim == BOTH_A2_TR_BL
            || ent2Anim == BOTH_A3_TR_BL
            || ent2Anim == BOTH_A4_TR_BL
            || ent2Anim == BOTH_A5_TR_BL
            || ent2Anim == BOTH_A6_TR_BL
            || ent2Anim == BOTH_A7_TR_BL
            || ent2Anim == BOTH_P1_S1_TL
        {
            //ent2 is attacking in the opposite diagonal
            return WP_SabersCheckLock2(ent1, ent2, LOCK_DIAG_TR);
        }
        if ent2Anim == BOTH_A1_BR_TL
            || ent2Anim == BOTH_A2_BR_TL
            || ent2Anim == BOTH_A3_BR_TL
            || ent2Anim == BOTH_A4_BR_TL
            || ent2Anim == BOTH_A5_BR_TL
            || ent2Anim == BOTH_A6_BR_TL
            || ent2Anim == BOTH_A7_BR_TL
            || ent2Anim == BOTH_P1_S1_BL
        {
            //ent2 is attacking in the opposite diagonal
            return WP_SabersCheckLock2(ent1, ent2, LOCK_DIAG_BL);
        }
        return QFALSE;
    }

    if ent2Anim == BOTH_A1_TR_BL
        || ent2Anim == BOTH_A2_TR_BL
        || ent2Anim == BOTH_A3_TR_BL
        || ent2Anim == BOTH_A4_TR_BL
        || ent2Anim == BOTH_A5_TR_BL
        || ent2Anim == BOTH_A6_TR_BL
        || ent2Anim == BOTH_A7_TR_BL
    {
        //ent2 is attacking diagonally
        if ent1BlockingPlayer == QTRUE {
            //player will block this anyway
            return WP_SabersCheckLock2(ent2, ent1, LOCK_DIAG_TR);
        }
        if ent1Anim == BOTH_A1_TR_BL
            || ent1Anim == BOTH_A2_TR_BL
            || ent1Anim == BOTH_A3_TR_BL
            || ent1Anim == BOTH_A4_TR_BL
            || ent1Anim == BOTH_A5_TR_BL
            || ent1Anim == BOTH_A6_TR_BL
            || ent1Anim == BOTH_A7_TR_BL
            || ent1Anim == BOTH_P1_S1_TL
        {
            //ent1 is attacking in the opposite diagonal
            return WP_SabersCheckLock2(ent2, ent1, LOCK_DIAG_TR);
        }
        if ent1Anim == BOTH_A1_BR_TL
            || ent1Anim == BOTH_A2_BR_TL
            || ent1Anim == BOTH_A3_BR_TL
            || ent1Anim == BOTH_A4_BR_TL
            || ent1Anim == BOTH_A5_BR_TL
            || ent1Anim == BOTH_A6_BR_TL
            || ent1Anim == BOTH_A7_BR_TL
            || ent1Anim == BOTH_P1_S1_BL
        {
            //ent1 is attacking in the opposite diagonal
            return WP_SabersCheckLock2(ent2, ent1, LOCK_DIAG_BL);
        }
        return QFALSE;
    }

    //TL to BR lock
    if ent1Anim == BOTH_A1_TL_BR
        || ent1Anim == BOTH_A2_TL_BR
        || ent1Anim == BOTH_A3_TL_BR
        || ent1Anim == BOTH_A4_TL_BR
        || ent1Anim == BOTH_A5_TL_BR
        || ent1Anim == BOTH_A6_TL_BR
        || ent1Anim == BOTH_A7_TL_BR
    {
        //ent1 is attacking diagonally
        if ent2BlockingPlayer == QTRUE {
            //player will block this anyway
            return WP_SabersCheckLock2(ent1, ent2, LOCK_DIAG_TL);
        }
        if ent2Anim == BOTH_A1_TL_BR
            || ent2Anim == BOTH_A2_TL_BR
            || ent2Anim == BOTH_A3_TL_BR
            || ent2Anim == BOTH_A4_TL_BR
            || ent2Anim == BOTH_A5_TL_BR
            || ent2Anim == BOTH_A6_TL_BR
            || ent2Anim == BOTH_A7_TL_BR
            || ent2Anim == BOTH_P1_S1_TR
        {
            //ent2 is attacking in the opposite diagonal
            return WP_SabersCheckLock2(ent1, ent2, LOCK_DIAG_TL);
        }
        if ent2Anim == BOTH_A1_BL_TR
            || ent2Anim == BOTH_A2_BL_TR
            || ent2Anim == BOTH_A3_BL_TR
            || ent2Anim == BOTH_A4_BL_TR
            || ent2Anim == BOTH_A5_BL_TR
            || ent2Anim == BOTH_A6_BL_TR
            || ent2Anim == BOTH_A7_BL_TR
            || ent2Anim == BOTH_P1_S1_BR
        {
            //ent2 is attacking in the opposite diagonal
            return WP_SabersCheckLock2(ent1, ent2, LOCK_DIAG_BR);
        }
        return QFALSE;
    }

    if ent2Anim == BOTH_A1_TL_BR
        || ent2Anim == BOTH_A2_TL_BR
        || ent2Anim == BOTH_A3_TL_BR
        || ent2Anim == BOTH_A4_TL_BR
        || ent2Anim == BOTH_A5_TL_BR
        || ent2Anim == BOTH_A6_TL_BR
        || ent2Anim == BOTH_A7_TL_BR
    {
        //ent2 is attacking diagonally
        if ent1BlockingPlayer == QTRUE {
            //player will block this anyway
            return WP_SabersCheckLock2(ent2, ent1, LOCK_DIAG_TL);
        }
        if ent1Anim == BOTH_A1_TL_BR
            || ent1Anim == BOTH_A2_TL_BR
            || ent1Anim == BOTH_A3_TL_BR
            || ent1Anim == BOTH_A4_TL_BR
            || ent1Anim == BOTH_A5_TL_BR
            || ent1Anim == BOTH_A6_TL_BR
            || ent1Anim == BOTH_A7_TL_BR
            || ent1Anim == BOTH_P1_S1_TR
        {
            //ent1 is attacking in the opposite diagonal
            return WP_SabersCheckLock2(ent2, ent1, LOCK_DIAG_TL);
        }
        if ent1Anim == BOTH_A1_BL_TR
            || ent1Anim == BOTH_A2_BL_TR
            || ent1Anim == BOTH_A3_BL_TR
            || ent1Anim == BOTH_A4_BL_TR
            || ent1Anim == BOTH_A5_BL_TR
            || ent1Anim == BOTH_A6_BL_TR
            || ent1Anim == BOTH_A7_BL_TR
            || ent1Anim == BOTH_P1_S1_BR
        {
            //ent1 is attacking in the opposite diagonal
            return WP_SabersCheckLock2(ent2, ent1, LOCK_DIAG_BR);
        }
        return QFALSE;
    }
    //L to R lock
    if ent1Anim == BOTH_A1__L__R
        || ent1Anim == BOTH_A2__L__R
        || ent1Anim == BOTH_A3__L__R
        || ent1Anim == BOTH_A4__L__R
        || ent1Anim == BOTH_A5__L__R
        || ent1Anim == BOTH_A6__L__R
        || ent1Anim == BOTH_A7__L__R
    {
        //ent1 is attacking l to r
        if ent2BlockingPlayer == QTRUE {
            //player will block this anyway
            return WP_SabersCheckLock2(ent1, ent2, LOCK_L);
        }
        if ent2Anim == BOTH_A1_TL_BR
            || ent2Anim == BOTH_A2_TL_BR
            || ent2Anim == BOTH_A3_TL_BR
            || ent2Anim == BOTH_A4_TL_BR
            || ent2Anim == BOTH_A5_TL_BR
            || ent2Anim == BOTH_A6_TL_BR
            || ent2Anim == BOTH_A7_TL_BR
            || ent2Anim == BOTH_P1_S1_TR
            || ent2Anim == BOTH_P1_S1_BL
        {
            //ent2 is attacking or blocking on the r
            return WP_SabersCheckLock2(ent1, ent2, LOCK_L);
        }
        return QFALSE;
    }
    if ent2Anim == BOTH_A1__L__R
        || ent2Anim == BOTH_A2__L__R
        || ent2Anim == BOTH_A3__L__R
        || ent2Anim == BOTH_A4__L__R
        || ent2Anim == BOTH_A5__L__R
        || ent2Anim == BOTH_A6__L__R
        || ent2Anim == BOTH_A7__L__R
    {
        //ent2 is attacking l to r
        if ent1BlockingPlayer == QTRUE {
            //player will block this anyway
            return WP_SabersCheckLock2(ent2, ent1, LOCK_L);
        }
        if ent1Anim == BOTH_A1_TL_BR
            || ent1Anim == BOTH_A2_TL_BR
            || ent1Anim == BOTH_A3_TL_BR
            || ent1Anim == BOTH_A4_TL_BR
            || ent1Anim == BOTH_A5_TL_BR
            || ent1Anim == BOTH_A6_TL_BR
            || ent1Anim == BOTH_A7_TL_BR
            || ent1Anim == BOTH_P1_S1_TR
            || ent1Anim == BOTH_P1_S1_BL
        {
            //ent1 is attacking or blocking on the r
            return WP_SabersCheckLock2(ent2, ent1, LOCK_L);
        }
        return QFALSE;
    }
    //R to L lock
    if ent1Anim == BOTH_A1__R__L
        || ent1Anim == BOTH_A2__R__L
        || ent1Anim == BOTH_A3__R__L
        || ent1Anim == BOTH_A4__R__L
        || ent1Anim == BOTH_A5__R__L
        || ent1Anim == BOTH_A6__R__L
        || ent1Anim == BOTH_A7__R__L
    {
        //ent1 is attacking r to l
        if ent2BlockingPlayer == QTRUE {
            //player will block this anyway
            return WP_SabersCheckLock2(ent1, ent2, LOCK_R);
        }
        if ent2Anim == BOTH_A1_TR_BL
            || ent2Anim == BOTH_A2_TR_BL
            || ent2Anim == BOTH_A3_TR_BL
            || ent2Anim == BOTH_A4_TR_BL
            || ent2Anim == BOTH_A5_TR_BL
            || ent2Anim == BOTH_A6_TR_BL
            || ent2Anim == BOTH_A7_TR_BL
            || ent2Anim == BOTH_P1_S1_TL
            || ent2Anim == BOTH_P1_S1_BR
        {
            //ent2 is attacking or blocking on the l
            return WP_SabersCheckLock2(ent1, ent2, LOCK_R);
        }
        return QFALSE;
    }
    if ent2Anim == BOTH_A1__R__L
        || ent2Anim == BOTH_A2__R__L
        || ent2Anim == BOTH_A3__R__L
        || ent2Anim == BOTH_A4__R__L
        || ent2Anim == BOTH_A5__R__L
        || ent2Anim == BOTH_A6__R__L
        || ent2Anim == BOTH_A7__R__L
    {
        //ent2 is attacking r to l
        if ent1BlockingPlayer == QTRUE {
            //player will block this anyway
            return WP_SabersCheckLock2(ent2, ent1, LOCK_R);
        }
        if ent1Anim == BOTH_A1_TR_BL
            || ent1Anim == BOTH_A2_TR_BL
            || ent1Anim == BOTH_A3_TR_BL
            || ent1Anim == BOTH_A4_TR_BL
            || ent1Anim == BOTH_A5_TR_BL
            || ent1Anim == BOTH_A6_TR_BL
            || ent1Anim == BOTH_A7_TR_BL
            || ent1Anim == BOTH_P1_S1_TL
            || ent1Anim == BOTH_P1_S1_BR
        {
            //ent1 is attacking or blocking on the l
            return WP_SabersCheckLock2(ent2, ent1, LOCK_R);
        }
        return QFALSE;
    }
    if Q_irand(0, 10) == 0 {
        return WP_SabersCheckLock2(ent1, ent2, LOCK_RANDOM);
    }
    QFALSE
}

/// `saberFace_t` (`struct saberFace_s`, w_saber.c:2309) — one triangular "face" of the
/// box-ish hull [`G_BuildSaberFaces`] builds around a blade for the detailed blade-vs-blade
/// collision check. A POD struct of three triangle vertices; no oracle (pure type). Layout is
/// load-bearing only within this module's static face buffer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct saberFace_t {
    pub v1: vec3_t,
    pub v2: vec3_t,
    pub v3: vec3_t,
}

/// `static GAME_INLINE void G_BuildSaberFaces(vec3_t base, vec3_t tip, float radius, vec3_t fwd,
/// vec3_t right, int *fNum, saberFace_t **fList)` (w_saber.c:2317) — builds the 12 triangular
/// [`saberFace_t`]s of a box-shaped hull (4 sides × 2 tris + top 2 + bottom 2) around a blade
/// running `base`→`tip` with half-extent `radius/2`, oriented by the blade's `fwd`/`right`
/// basis (`-rww`). Faces are stored into a function-local `static` buffer and handed back via
/// out-params `*fNum`/`*fList`; the buffer is a module-static here, mirroring the C single-
/// threaded function-static. The "left/right/front/back" surface selection is the C author's
/// own admitted hack, carried over verbatim. No oracle: the out-param `**fList` returns a
/// pointer into a module static (not a value), so it's exercised via [`G_SaberCollide`]'s
/// oracle path / the face-collision test rather than directly.
///
/// # Safety
/// `fNum`/`fList` must be valid writable out-params; the returned `fList` aliases the module
/// static `G_BUILDSABERFACES_FACES`, valid until the next call (single-threaded game module).
pub unsafe fn G_BuildSaberFaces(
    base: &vec3_t,
    tip: &vec3_t,
    radius: f32,
    fwd: &vec3_t,
    right: &vec3_t,
    fNum: *mut c_int,
    fList: *mut *mut saberFace_t,
) {
    static mut G_BUILDSABERFACES_FACES: [saberFace_t; 12] = [saberFace_t {
        v1: [0.0; 3],
        v2: [0.0; 3],
        v3: [0.0; 3],
    }; 12];
    let mut i: usize = 0;
    let mut invFwd: vec3_t = [0.0; 3];
    let mut invRight: vec3_t = [0.0; 3];

    VectorCopy(fwd, &mut invFwd);
    VectorInverse(&mut invFwd);
    VectorCopy(right, &mut invRight);
    VectorInverse(&mut invRight);

    let faces = &mut *addr_of_mut!(G_BUILDSABERFACES_FACES);

    while i < 8 {
        //yeah, this part is kind of a hack, but eh
        let (d1, d2): (&vec3_t, &vec3_t) = if i < 2 {
            //"left" surface
            (fwd, &invRight)
        } else if i < 4 {
            //"right" surface
            (fwd, right)
        } else if i < 6 {
            //"front" surface
            (right, fwd)
        } else {
            //"back" surface (i < 8)
            (right, &invFwd)
        };

        //first triangle for this surface
        VectorMA(base, radius / 2.0f32, d1, &mut faces[i].v1);
        let tmp = faces[i].v1;
        VectorMA(&tmp, radius / 2.0f32, d2, &mut faces[i].v1);

        VectorMA(tip, radius / 2.0f32, d1, &mut faces[i].v2);
        let tmp = faces[i].v2;
        VectorMA(&tmp, radius / 2.0f32, d2, &mut faces[i].v2);

        VectorMA(tip, -radius / 2.0f32, d1, &mut faces[i].v3);
        let tmp = faces[i].v3;
        VectorMA(&tmp, radius / 2.0f32, d2, &mut faces[i].v3);

        i += 1;

        //second triangle for this surface
        VectorMA(tip, -radius / 2.0f32, d1, &mut faces[i].v1);
        let tmp = faces[i].v1;
        VectorMA(&tmp, radius / 2.0f32, d2, &mut faces[i].v1);

        VectorMA(base, radius / 2.0f32, d1, &mut faces[i].v2);
        let tmp = faces[i].v2;
        VectorMA(&tmp, radius / 2.0f32, d2, &mut faces[i].v2);

        VectorMA(base, -radius / 2.0f32, d1, &mut faces[i].v3);
        let tmp = faces[i].v3;
        VectorMA(&tmp, radius / 2.0f32, d2, &mut faces[i].v3);

        i += 1;
    }

    //top surface
    //face 1
    VectorMA(tip, radius / 2.0f32, fwd, &mut faces[i].v1);
    let tmp = faces[i].v1;
    VectorMA(&tmp, -radius / 2.0f32, right, &mut faces[i].v1);

    VectorMA(tip, radius / 2.0f32, fwd, &mut faces[i].v2);
    let tmp = faces[i].v2;
    VectorMA(&tmp, radius / 2.0f32, right, &mut faces[i].v2);

    VectorMA(tip, -radius / 2.0f32, fwd, &mut faces[i].v3);
    let tmp = faces[i].v3;
    VectorMA(&tmp, -radius / 2.0f32, right, &mut faces[i].v3);

    i += 1;

    //face 2
    VectorMA(tip, radius / 2.0f32, fwd, &mut faces[i].v1);
    let tmp = faces[i].v1;
    VectorMA(&tmp, radius / 2.0f32, right, &mut faces[i].v1);

    VectorMA(tip, -radius / 2.0f32, fwd, &mut faces[i].v2);
    let tmp = faces[i].v2;
    VectorMA(&tmp, -radius / 2.0f32, right, &mut faces[i].v2);

    VectorMA(tip, -radius / 2.0f32, fwd, &mut faces[i].v3);
    let tmp = faces[i].v3;
    VectorMA(&tmp, radius / 2.0f32, right, &mut faces[i].v3);

    i += 1;

    //bottom surface
    //face 1
    VectorMA(base, radius / 2.0f32, fwd, &mut faces[i].v1);
    let tmp = faces[i].v1;
    VectorMA(&tmp, -radius / 2.0f32, right, &mut faces[i].v1);

    VectorMA(base, radius / 2.0f32, fwd, &mut faces[i].v2);
    let tmp = faces[i].v2;
    VectorMA(&tmp, radius / 2.0f32, right, &mut faces[i].v2);

    VectorMA(base, -radius / 2.0f32, fwd, &mut faces[i].v3);
    let tmp = faces[i].v3;
    VectorMA(&tmp, -radius / 2.0f32, right, &mut faces[i].v3);

    i += 1;

    //face 2
    VectorMA(base, radius / 2.0f32, fwd, &mut faces[i].v1);
    let tmp = faces[i].v1;
    VectorMA(&tmp, radius / 2.0f32, right, &mut faces[i].v1);

    VectorMA(base, -radius / 2.0f32, fwd, &mut faces[i].v2);
    let tmp = faces[i].v2;
    VectorMA(&tmp, -radius / 2.0f32, right, &mut faces[i].v2);

    VectorMA(base, -radius / 2.0f32, fwd, &mut faces[i].v3);
    let tmp = faces[i].v3;
    VectorMA(&tmp, radius / 2.0f32, right, &mut faces[i].v3);

    i += 1;

    //yeah.. always going to be 12 I suppose.
    *fNum = i as c_int;
    *fList = faces.as_mut_ptr();
}

/// `void G_SabCol_CalcPlaneEq(vec3_t x, vec3_t y, vec3_t z, float *planeEq)` (w_saber.c:2436)
/// — a saber-collision utility (`-rww`) that computes the plane equation `Ax + By + Cz + D = 0`
/// through the three triangle vertices `x`/`y`/`z` (a generated saber "face"), used by
/// `G_SaberFaceCollisionCheck` to decide which side of a face an attack endpoint lies on. The
/// normal (`planeEq[0..3]`) is twice the triangle's cross product, expanded by the standard
/// cofactor form, and `planeEq[3]` is `-` the scalar triple product placing the origin offset.
/// Pure float math, faithful to the C (`vec3_t` -> `&[f32; 3]`, `float *planeEq` -> `&mut [f32; 4]`).
pub fn G_SabCol_CalcPlaneEq(x: &vec3_t, y: &vec3_t, z: &vec3_t, planeEq: &mut [f32; 4]) {
    planeEq[0] = x[1] * (y[2] - z[2]) + y[1] * (z[2] - x[2]) + z[1] * (x[2] - y[2]);
    planeEq[1] = x[2] * (y[0] - z[0]) + y[2] * (z[0] - x[0]) + z[2] * (x[0] - y[0]);
    planeEq[2] = x[0] * (y[1] - z[1]) + y[0] * (z[1] - x[1]) + z[0] * (x[1] - y[1]);
    planeEq[3] = -(x[0] * (y[1] * z[2] - z[1] * y[2])
        + y[0] * (z[1] * x[2] - x[1] * z[2])
        + z[0] * (x[1] * y[2] - y[1] * x[2]));
}

/// `int G_SabCol_PointRelativeToPlane(vec3_t pos, float *side, float *planeEq)` (w_saber.c:2445)
/// — the companion saber-collision utility (`-rww`) that classifies a point against a plane
/// equation produced by [`G_SabCol_CalcPlaneEq`]: it stores the signed distance
/// `A*x + B*y + C*z + D` into `*side` and returns `1` (in front), `-1` (behind), or `0` (on the
/// plane). Pure float math, faithful to the C (`vec3_t` -> `&[f32; 3]`, the two `float *`
/// out/in params -> `&mut f32` / `&[f32; 4]`).
pub fn G_SabCol_PointRelativeToPlane(pos: &vec3_t, side: &mut f32, planeEq: &[f32; 4]) -> c_int {
    *side = planeEq[0] * pos[0] + planeEq[1] * pos[1] + planeEq[2] * pos[2] + planeEq[3];

    if *side > 0.0f32 {
        return 1;
    } else if *side < 0.0f32 {
        return -1;
    }

    0
}

/// `static GAME_INLINE qboolean G_SaberFaceCollisionCheck(int fNum, saberFace_t *fList,
/// vec3_t atkStart, vec3_t atkEnd, vec3_t atkMins, vec3_t atkMaxs, vec3_t impactPoint)`
/// (w_saber.c:2462) — the actual collision check against a blade's generated [`saberFace_t`]
/// hull (`-rww`). For each face it builds the face plane ([`G_SabCol_CalcPlaneEq`]); if the
/// attack's start/end straddle that plane, it solves for the line/plane intersection `point`,
/// then tests `point` (and the box-extruded `minPoint`/`maxPoint`) against each of the three
/// edge planes — extruded `-2` along the face normal — to confirm the hit lies inside the
/// triangle. The first face that fully contains the intersection wins: it copies `point` into
/// `impactPoint` and returns `qtrue`. A zero atkMins/atkMaxs box is widened to ±1. The C's
/// `static` locals (avoiding stack churn) become plain locals here — each is assigned before
/// use within the call, so behavior is identical. No oracle here directly; exercised via
/// [`G_SaberCollide`]'s test path.
///
/// # Safety
/// `fList` must point to `fNum` valid [`saberFace_t`]s; `atkStart`/`atkEnd`/`atkMins`/`atkMaxs`/
/// `impactPoint` must be valid `vec3_t`s (atkMins/atkMaxs may be mutated to the ±1 default).
pub unsafe fn G_SaberFaceCollisionCheck(
    fNum: c_int,
    fList: *mut saberFace_t,
    atkStart: &vec3_t,
    atkEnd: &vec3_t,
    atkMins: &mut vec3_t,
    atkMaxs: &mut vec3_t,
    impactPoint: &mut vec3_t,
) -> qboolean {
    let mut planeEq: [f32; 4] = [0.0; 4];
    let mut side: f32 = 0.0;
    let mut side2: f32 = 0.0;
    let mut dir: vec3_t = [0.0; 3];
    let mut point: vec3_t = [0.0; 3];
    let mut i: c_int = 0;

    if VectorCompare(atkMins, &vec3_origin) != 0 && VectorCompare(atkMaxs, &vec3_origin) != 0 {
        VectorSet(atkMins, -1.0f32, -1.0f32, -1.0f32);
        VectorSet(atkMaxs, 1.0f32, 1.0f32, 1.0f32);
    }

    VectorSubtract(atkEnd, atkStart, &mut dir);

    while i < fNum {
        let face = &*fList.offset(i as isize);
        G_SabCol_CalcPlaneEq(&face.v1, &face.v2, &face.v3, &mut planeEq);

        if G_SabCol_PointRelativeToPlane(atkStart, &mut side, &planeEq)
            != G_SabCol_PointRelativeToPlane(atkEnd, &mut side2, &planeEq)
        {
            //start/end points intersect with the plane
            let mut extruded: vec3_t = [0.0; 3];
            let mut minPoint: vec3_t = [0.0; 3];
            let mut maxPoint: vec3_t = [0.0; 3];
            let mut planeNormal: vec3_t = [0.0; 3];
            let mut facing: c_int;

            let planeNormal3: vec3_t = [planeEq[0], planeEq[1], planeEq[2]];
            VectorCopy(&planeNormal3, &mut planeNormal);
            side2 = planeNormal[0] * dir[0] + planeNormal[1] * dir[1] + planeNormal[2] * dir[2];

            let dist: f32 = side / side2;
            VectorMA(atkStart, -dist, &dir, &mut point);

            VectorAdd(&point, atkMins, &mut minPoint);
            VectorAdd(&point, atkMaxs, &mut maxPoint);

            //point is now the point at which we intersect on the plane.
            //see if that point is within the edges of the face.
            VectorMA(&face.v1, -2.0f32, &planeNormal, &mut extruded);
            G_SabCol_CalcPlaneEq(&face.v1, &face.v2, &extruded, &mut planeEq);
            facing = G_SabCol_PointRelativeToPlane(&point, &mut side, &planeEq);

            if facing < 0 {
                //not intersecting.. let's try with the mins/maxs and see if they interesect on the edge plane
                facing = G_SabCol_PointRelativeToPlane(&minPoint, &mut side, &planeEq);
                if facing < 0 {
                    facing = G_SabCol_PointRelativeToPlane(&maxPoint, &mut side, &planeEq);
                }
            }

            if facing >= 0 {
                //first edge is facing...
                VectorMA(&face.v2, -2.0f32, &planeNormal, &mut extruded);
                G_SabCol_CalcPlaneEq(&face.v2, &face.v3, &extruded, &mut planeEq);
                facing = G_SabCol_PointRelativeToPlane(&point, &mut side, &planeEq);

                if facing < 0 {
                    //not intersecting.. let's try with the mins/maxs and see if they interesect on the edge plane
                    facing = G_SabCol_PointRelativeToPlane(&minPoint, &mut side, &planeEq);
                    if facing < 0 {
                        facing = G_SabCol_PointRelativeToPlane(&maxPoint, &mut side, &planeEq);
                    }
                }

                if facing >= 0 {
                    //second edge is facing...
                    VectorMA(&face.v3, -2.0f32, &planeNormal, &mut extruded);
                    G_SabCol_CalcPlaneEq(&face.v3, &face.v1, &extruded, &mut planeEq);
                    facing = G_SabCol_PointRelativeToPlane(&point, &mut side, &planeEq);

                    if facing < 0 {
                        //not intersecting.. let's try with the mins/maxs and see if they interesect on the edge plane
                        facing = G_SabCol_PointRelativeToPlane(&minPoint, &mut side, &planeEq);
                        if facing < 0 {
                            facing = G_SabCol_PointRelativeToPlane(&maxPoint, &mut side, &planeEq);
                        }
                    }

                    if facing >= 0 {
                        //third edge is facing.. success
                        VectorCopy(&point, impactPoint);
                        return QTRUE;
                    }
                }
            }
        }

        i += 1;
    }

    //did not hit anything
    QFALSE
}

/// `static GAME_INLINE qboolean G_SaberCollide(gentity_t *atk, gentity_t *def, vec3_t atkStart,
/// vec3_t atkEnd, vec3_t atkMins, vec3_t atkMaxs, vec3_t impactPoint)` (w_saber.c:2563) — the
/// detailed blade-vs-blade collision test (`-rww`): does the attacker's swept attack box
/// (`atkStart`→`atkEnd`, `atkMins`/`atkMaxs`) actually intersect any of the *defender's* blades?
/// When `g_saberBladeFaces` is off this detailed check is disabled and it optimistically returns
/// `qtrue`. Both ents must be in-use clients. It walks each of the defender's sabers
/// (`saber[0..MAX_SABERS]` with a non-empty `model`) and each blade, skipping blades whose
/// `storageTime` is stale (>=200ms old). For a fresh blade it derives base/tip from
/// `muzzlePoint`+`lengthMax*muzzleDir`, recovers a `fwd`/`right` basis via
/// [`vectoangles`]/[`AngleVectors`], builds the collision hull with [`G_BuildSaberFaces`] (radius
/// `blade->radius*3`), and runs [`G_SaberFaceCollisionCheck`]; the first hit returns `qtrue`.
/// No oracle: reads deeply-nested opaque `gentity_t`/`gclient_t`/saber state plus the
/// `g_saberBladeFaces` cvar and `level.time` globals (the deep-deref/global-read precedent).
///
/// # Safety
/// `atk`/`def` must point to valid `gentity_t`s; the body only derefs their `client`s after the
/// in-use/client guard. `atkStart`/`atkEnd`/`atkMins`/`atkMaxs`/`impactPoint` must be valid
/// `vec3_t`s (atkMins/atkMaxs may be widened by [`G_SaberFaceCollisionCheck`]).
pub unsafe fn G_SaberCollide(
    atk: *mut gentity_t,
    def: *mut gentity_t,
    atkStart: &vec3_t,
    atkEnd: &vec3_t,
    atkMins: &mut vec3_t,
    atkMaxs: &mut vec3_t,
    impactPoint: &mut vec3_t,
) -> qboolean {
    let mut i: c_int;
    let mut j: c_int;

    if (*addr_of!(g_saberBladeFaces)).integer == 0 {
        //detailed check not enabled
        return QTRUE;
    }

    if (*atk).inuse != QTRUE
        || (*atk).client.is_null()
        || (*def).inuse != QTRUE
        || (*def).client.is_null()
    {
        //must have 2 clients and a valid saber entity
        return QFALSE;
    }

    i = 0;
    while i < MAX_SABERS as c_int {
        j = 0;
        if (*(*def).client).saber[i as usize].model[0] != 0 {
            //valid saber on the defender
            let mut v: vec3_t = [0.0; 3];
            let mut fwd: vec3_t = [0.0; 3];
            let mut right: vec3_t = [0.0; 3];
            let mut base: vec3_t = [0.0; 3];
            let mut tip: vec3_t = [0.0; 3];
            let mut fNum: c_int;
            let mut fList: *mut saberFace_t;

            //go through each blade on the defender's sabers
            while j < (*(*def).client).saber[i as usize].numBlades {
                let blade = &(*(*def).client).saber[i as usize].blade[j as usize];

                if ((*addr_of!(level)).time - blade.storageTime) < 200 {
                    //recently updated
                    //first get base and tip of blade
                    VectorCopy(&blade.muzzlePoint, &mut base);
                    VectorMA(&base, blade.lengthMax, &blade.muzzleDir, &mut tip);

                    //Now get relative angles between the points
                    VectorSubtract(&tip, &base, &mut v);
                    let v_copy = v;
                    vectoangles(&v_copy, &mut v);
                    AngleVectors(&v, None, Some(&mut right), Some(&mut fwd));

                    //now build collision faces for this blade
                    fNum = 0;
                    fList = null_mut();
                    G_BuildSaberFaces(
                        &base,
                        &tip,
                        blade.radius * 3.0f32,
                        &fwd,
                        &right,
                        &mut fNum,
                        &mut fList,
                    );
                    if fNum > 0 {
                        // #if 0 — debug visualization disabled in the server build (w_saber.c:2609-2625):
                        //   if (atk->s.number == 0) { ... G_TestLine(...) over the face list ... }
                        // G_TestLine is a debug-only trap not compiled into the server, so the block
                        // is reproduced disabled (not ported), matching the crate's #if 0 precedent.

                        if G_SaberFaceCollisionCheck(
                            fNum,
                            fList,
                            atkStart,
                            atkEnd,
                            atkMins,
                            atkMaxs,
                            impactPoint,
                        ) != QFALSE
                        {
                            //collided
                            return QTRUE;
                        }
                    }
                }
                j += 1;
            }
        }
        i += 1;
    }

    QFALSE
}

/// `static GAME_INLINE int G_SaberAttackPower(gentity_t *ent, qboolean attacking)`
/// (w_saber.c:120) — the saber-combat "power level" of a swing/defense for `ent`, used to decide
/// who wins a saber clash. Starts at `ps.fd.saberAnimLevel` (the dual/staff special stances are
/// pinned to "medium" = 2). An `attacking` swing is doubled-plus-one, then — if the last swing was
/// captured this frame (`lastSaberStorageTime >= level.time-50` and `olderIsValid`) — gains one
/// level per `toleranceAmt` units of swing distance (the stance-dependent tolerance is 8/16/24 for
/// strong/medium/fast, 16 otherwise). A broken arm scales the level by `0.3` (C `int *= 0.3`, i.e.
/// truncating). The level is clamped to `[1,16]`; power-duel "lone" doubles it and siege attacks
/// triple it. The `#ifndef FINAL_BUILD` debug print (gated on `g_saberDebugPrint > 1`) is present in
/// this non-final build.
///
/// # Safety
/// Dereferences `self` and its `client`; callers must pass a valid client entity (the C `assert`s
/// `ent && ent->client`).
pub unsafe fn G_SaberAttackPower(self_: *mut gentity_t, attacking: qboolean) -> c_int {
    let mut baseLevel: c_int;

    baseLevel = (*(*self_).client).ps.fd.saberAnimLevel;

    //Give "medium" strength for the two special stances.
    if baseLevel == SS_DUAL {
        baseLevel = 2;
    } else if baseLevel == SS_STAFF {
        baseLevel = 2;
    }

    if attacking != QFALSE {
        //the attacker gets a boost to help penetrate defense.
        //General boost up so the individual levels make a bigger difference.
        baseLevel *= 2;

        baseLevel += 1;

        //Get the "speed" of the swing, roughly, and add more power
        //to the attack based on it.
        if (*(*self_).client).lastSaberStorageTime >= ((*addr_of!(level)).time - 50)
            && (*(*self_).client).olderIsValid != QFALSE
        {
            let mut vSub: vec3_t = [0.0; 3];
            let mut swingDist: c_int;
            let toleranceAmt: c_int;

            //We want different "tolerance" levels for adding in the distance of the last swing
            //to the base power level depending on which stance we are using. Otherwise fast
            //would have more advantage than it should since the animations are all much faster.
            match (*(*self_).client).ps.fd.saberAnimLevel {
                SS_STRONG => {
                    toleranceAmt = 8;
                }
                SS_MEDIUM => {
                    toleranceAmt = 16;
                }
                SS_FAST => {
                    toleranceAmt = 24;
                }
                _ => {
                    //dual, staff, etc.
                    toleranceAmt = 16;
                }
            }

            VectorSubtract(
                &(*(*self_).client).lastSaberBase_Always,
                &(*(*self_).client).olderSaberBase,
                &mut vSub,
            );
            swingDist = VectorLength(&vSub) as c_int;

            while swingDist > 0 {
                //I would like to do something more clever. But I suppose this works, at least for now.
                baseLevel += 1;
                swingDist -= toleranceAmt;
            }
        }

        // `#ifndef FINAL_BUILD` — present in this non-final build.
        if (*addr_of!(g_saberDebugPrint)).integer > 1 {
            Com_Printf(&format!(
                "Client {}: ATT STR: {}\n",
                (*self_).s.number,
                baseLevel
            ));
        }
    }

    if ((*(*self_).client).ps.brokenLimbs & (1 << BROKENLIMB_RARM)) != 0
        || ((*(*self_).client).ps.brokenLimbs & (1 << BROKENLIMB_LARM)) != 0
    {
        //We're very weak when one of our arms is broken
        baseLevel = (baseLevel as f64 * 0.3) as c_int;
    }

    //Cap at reasonable values now.
    if baseLevel < 1 {
        baseLevel = 1;
    } else if baseLevel > 16 {
        baseLevel = 16;
    }

    if (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        && (*(*self_).client).sess.duelTeam == DUELTEAM_LONE
    {
        //get more power then
        return baseLevel * 2;
    } else if attacking != QFALSE && (*addr_of!(g_gametype)).integer == GT_SIEGE {
        //in siege, saber battles should be quicker and more biased toward the attacker
        return baseLevel * 3;
    }

    baseLevel
}

/// `static GAME_INLINE int G_GetAttackDamage(gentity_t *self, int minDmg, int maxDmg, float multPoint)`
/// (w_saber.c:2111) — scale a saber swing's damage by where in the attack animation the swing lands.
/// `attackAnimLength` is the current attack anim's `numFrames * fabs(frameLerp)` (read from the
/// `bgAllAnims` table at `self->localAnimIndex` / `ps.torsoAnim`), stretched by the same
/// `BG_SaberStartTransAnim` speed factor the animation itself would use. `peakPoint` is the
/// damage-peak position (`attackAnimLength` pulled back by `multPoint`), `currentPoint` is
/// `ps.torsoTimer`; `damageFactor = currentPoint/peakPoint` (folded back below 1.0 past the peak),
/// applied to `maxDmg` and clamped to `[minDmg, maxDmg]`. The C `peakDif` is dead (computed, never
/// read) but kept for fidelity.
///
/// # Safety
/// Dereferences `self` and its `client`, and reads the `bgAllAnims` global table; callers must
/// pass a valid client entity with a valid `localAnimIndex`/`torsoAnim`.
pub unsafe fn G_GetAttackDamage(
    self_: *mut gentity_t,
    minDmg: c_int,
    maxDmg: c_int,
    multPoint: f32,
) -> c_int {
    let speedDif: c_int;
    let mut totalDamage: c_int = maxDmg;
    let mut peakPoint: f32;
    let a = (*addr_of!(bgAllAnims))[(*self_).localAnimIndex as usize]
        .anims
        .add((*(*self_).client).ps.torsoAnim as usize);
    let mut attackAnimLength: f32 =
        ((*a).numFrames as f64 * ((*a).frameLerp as f32 as f64).abs()) as f32;
    let currentPoint: f32;
    let mut damageFactor: f32;
    let mut animSpeedFactor: f32 = 1.0f32;

    //Be sure to scale by the proper anim speed just as if we were going to play the animation
    BG_SaberStartTransAnim(
        (*self_).s.number,
        (*(*self_).client).ps.fd.saberAnimLevel,
        (*(*self_).client).ps.weapon,
        (*(*self_).client).ps.torsoAnim,
        &mut animSpeedFactor,
        (*(*self_).client).ps.brokenLimbs,
    );
    speedDif = (attackAnimLength - (attackAnimLength * animSpeedFactor)) as c_int;
    attackAnimLength += speedDif as f32;
    peakPoint = attackAnimLength;
    peakPoint -= attackAnimLength * multPoint;

    //we treat torsoTimer as the point in the animation (closer it is to attackAnimLength, closer it is to beginning)
    currentPoint = (*(*self_).client).ps.torsoTimer as f32;

    // C computes peakDif here but never reads it again — kept for fidelity, marked unused.
    let _peakDif: c_int = if peakPoint > currentPoint {
        (peakPoint - currentPoint) as c_int
    } else {
        (currentPoint - peakPoint) as c_int
    };

    damageFactor = currentPoint / peakPoint;
    if damageFactor > 1.0f32 {
        damageFactor = 2.0f32 - damageFactor;
    }

    totalDamage = (totalDamage as f32 * damageFactor) as c_int;
    if totalDamage < minDmg {
        totalDamage = minDmg;
    }
    if totalDamage > maxDmg {
        totalDamage = maxDmg;
    }

    //Com_Printf("%i\n", totalDamage);

    totalDamage
}

/// `static GAME_INLINE float G_GetAnimPoint(gentity_t *self)` (w_saber.c:2163) — the fraction
/// `[0..1]` of how far the client's current attack animation has progressed. Same preamble as
/// [`G_GetAttackDamage`]: `attackAnimLength` is the anim's `numFrames * fabs(frameLerp)` stretched
/// by the `BG_SaberStartTransAnim` speed factor; the returned percentage is
/// `ps.torsoTimer / attackAnimLength`.
///
/// # Safety
/// Dereferences `self` and its `client`, and reads the `bgAllAnims` global table; callers must
/// pass a valid client entity with a valid `localAnimIndex`/`torsoAnim`.
pub unsafe fn G_GetAnimPoint(self_: *mut gentity_t) -> f32 {
    let speedDif: c_int;
    let a = (*addr_of!(bgAllAnims))[(*self_).localAnimIndex as usize]
        .anims
        .add((*(*self_).client).ps.torsoAnim as usize);
    let mut attackAnimLength: f32 =
        ((*a).numFrames as f64 * ((*a).frameLerp as f32 as f64).abs()) as f32;
    let currentPoint: f32;
    let animPercentage: f32;
    let mut animSpeedFactor: f32 = 1.0f32;

    //Be sure to scale by the proper anim speed just as if we were going to play the animation
    BG_SaberStartTransAnim(
        (*self_).s.number,
        (*(*self_).client).ps.fd.saberAnimLevel,
        (*(*self_).client).ps.weapon,
        (*(*self_).client).ps.torsoAnim,
        &mut animSpeedFactor,
        (*(*self_).client).ps.brokenLimbs,
    );
    speedDif = (attackAnimLength - (attackAnimLength * animSpeedFactor)) as c_int;
    attackAnimLength += speedDif as f32;

    currentPoint = (*(*self_).client).ps.torsoTimer as f32;

    animPercentage = currentPoint / attackAnimLength;

    //Com_Printf("%f\n", animPercentage);

    animPercentage
}

/// `static GAME_INLINE qboolean G_ClientIdleInWorld(gentity_t *ent)` (w_saber.c:2185) — `qtrue`
/// when a client is standing idle in the world: never for NPCs (`ET_NPC`), otherwise only when no
/// movement key is held (`upmove`/`forwardmove`/`rightmove` all zero) and none of the action
/// buttons are pressed (gesture, force-grip, alt-attack, force-power, force-lightning, force-drain,
/// attack). Used by the saber-throw return checks to leave an unmoving bystander alone.
///
/// # Safety
/// Dereferences `ent` and (for non-NPCs) its `client`; callers must pass a valid entity.
pub unsafe fn G_ClientIdleInWorld(ent: *mut gentity_t) -> qboolean {
    if (*ent).s.eType == ET_NPC {
        return QFALSE;
    }

    if (*(*ent).client).pers.cmd.upmove == 0
        && (*(*ent).client).pers.cmd.forwardmove == 0
        && (*(*ent).client).pers.cmd.rightmove == 0
        && (*(*ent).client).pers.cmd.buttons & BUTTON_GESTURE == 0
        && (*(*ent).client).pers.cmd.buttons & BUTTON_FORCEGRIP == 0
        && (*(*ent).client).pers.cmd.buttons & BUTTON_ALT_ATTACK == 0
        && (*(*ent).client).pers.cmd.buttons & BUTTON_FORCEPOWER == 0
        && (*(*ent).client).pers.cmd.buttons & BUTTON_FORCE_LIGHTNING == 0
        && (*(*ent).client).pers.cmd.buttons & BUTTON_FORCE_DRAIN == 0
        && (*(*ent).client).pers.cmd.buttons & BUTTON_ATTACK == 0
    {
        return QTRUE;
    }

    QFALSE
}

/// `static qboolean WP_GetSaberDeflectionAngle( gentity_t *attacker, gentity_t *defender,
/// float saberHitFraction )` (w_saber.c:1811). When an attacker's saber is stopped by a
/// defender's parry, picks the attacker's bounce/deflection [`saberMove`] and sets
/// `ps.saberBlocked`. Returns `qtrue` if a deflection move was chosen, `qfalse` if it
/// bounced straight back (or bailed out). `saberHitFraction` is unused in this build.
///
/// Both entities must be valid clients with non-null `ghoul2`, and both must have stored
/// their saber position within the last 500ms (`lastSaberStorageTime`) — otherwise it
/// bails (`qfalse`). The body has two paths gated by a hardcoded local `animBasedDeflection
/// = qtrue`: the live anim-based path (compares the attack/defence [`saberMoveData`] quads,
/// mirrors the defender's left/right, and picks a bounce via [`PM_SaberBounceForAttack`] or
/// a deflection via [`PM_SaberDeflectionForQuad`]); and a dead math-based path (kept faithful
/// behind `if animBasedDeflection { … } else { … }`, never taken). `fabs`/`ceil` are taken on
/// `f32`, matching the C's `float`-domain `fabs((float)…)`/`ceil(((float)…)/2.0f)`. The
/// `#ifndef FINAL_BUILD` debug `Com_Printf`s are present (this is a non-final build, like
/// [`g_saberDebugPrint`]). No oracle: pure entity/`ps`/global-table mutation with no
/// extractable scalar return surface (it writes `saberMove`/`saberBlocked` and reads opaque
/// `ghoul2`); faithful 1:1 with the C.
pub unsafe fn WP_GetSaberDeflectionAngle(
    attacker: *mut gentity_t,
    defender: *mut gentity_t,
    _saberHitFraction: f32,
) -> qboolean {
    let animBasedDeflection: qboolean = QTRUE;

    if attacker.is_null() || (*attacker).client.is_null() || (*attacker).ghoul2.is_null() {
        return QFALSE;
    }
    if defender.is_null() || (*defender).client.is_null() || (*defender).ghoul2.is_null() {
        return QFALSE;
    }

    let lvl = &*addr_of!(level);

    if (lvl.time - (*(*attacker).client).lastSaberStorageTime) > 500 {
        // last update was too long ago, something is happening to this client to prevent his
        // saber from updating
        return QFALSE;
    }
    if (lvl.time - (*(*defender).client).lastSaberStorageTime) > 500 {
        // ditto
        return QFALSE;
    }

    let attSaberLevel = G_SaberAttackPower(attacker, SaberAttacking(attacker));
    let defSaberLevel = G_SaberAttackPower(defender, SaberAttacking(defender));

    let smd = &*addr_of!(saberMoveData);
    let table = &*addr_of!(animTable);

    if animBasedDeflection == QTRUE {
        // Hmm, let's try just basing it off the anim
        let attQuadStart = smd[(*(*attacker).client).ps.saberMove as usize].startQuad;
        let attQuadEnd = smd[(*(*attacker).client).ps.saberMove as usize].endQuad;
        let mut defQuad = smd[(*(*defender).client).ps.saberMove as usize].endQuad;
        let mut quadDiff = ((defQuad - attQuadStart) as f32).abs() as c_int;

        if (*(*defender).client).ps.saberMove == LS_READY {
            // FIXME: we should probably do SOMETHING here...
            // I have this return qfalse here in the hopes that the defender will pick a parry
            // and the attacker will hit the defender's saber again. But maybe this func call
            // should come *after* it's decided whether or not the defender is going to parry.
            return QFALSE;
        }

        // reverse the left/right of the defQuad because of the mirrored nature of facing each
        // other in combat
        match defQuad {
            Q_BR => defQuad = Q_BL,
            Q_R => defQuad = Q_L,
            Q_TR => defQuad = Q_TL,
            Q_TL => defQuad = Q_TR,
            Q_L => defQuad = Q_R,
            Q_BL => defQuad = Q_BR,
            _ => {}
        }

        if quadDiff > 4 {
            // wrap around so diff is never greater than 180 (4 * 45)
            quadDiff = 4 - (quadDiff - 4);
        }
        // have the quads, find a good anim to use
        if (quadDiff == 0 || (quadDiff == 1 && Q_irand(0, 1) != 0)) // defender pretty much stopped the attack at a 90 degree angle
            && (defSaberLevel == attSaberLevel || Q_irand(0, defSaberLevel - attSaberLevel) >= 0)
        // and the defender's style is stronger
        {
            // bounce straight back
            let attMove = (*(*attacker).client).ps.saberMove;
            (*(*attacker).client).ps.saberMove =
                PM_SaberBounceForAttack((*(*attacker).client).ps.saberMove);
            if g_saberDebugPrint.integer != 0 {
                Com_Printf(&format!(
                    "attack {} vs. parry {} bounced to {}\n",
                    core::ffi::CStr::from_ptr(table[smd[attMove as usize].animToUse as usize].name)
                        .to_string_lossy(),
                    core::ffi::CStr::from_ptr(
                        table[smd[(*(*defender).client).ps.saberMove as usize].animToUse as usize]
                            .name
                    )
                    .to_string_lossy(),
                    core::ffi::CStr::from_ptr(
                        table[smd[(*(*attacker).client).ps.saberMove as usize].animToUse as usize]
                            .name
                    )
                    .to_string_lossy(),
                ));
            }
            (*(*attacker).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
            QFALSE
        } else {
            // attack hit at an angle, figure out what angle it should bounce off att
            quadDiff = defQuad - attQuadEnd;
            // add half the diff of between the defense and attack end to the attack end
            if quadDiff > 4 {
                quadDiff = 4 - (quadDiff - 4);
            } else if quadDiff < -4 {
                quadDiff = -4 + (quadDiff + 4);
            }
            let mut newQuad = attQuadEnd + (quadDiff as f32 / 2.0f32).ceil() as c_int;
            if newQuad < Q_BR {
                // less than zero wraps around
                newQuad = Q_B + newQuad;
            }
            if newQuad == attQuadStart {
                // never come off at the same angle that we would have if the attack was not
                // interrupted
                if Q_irand(0, 1) != 0 {
                    newQuad -= 1;
                } else {
                    newQuad += 1;
                }
                if newQuad < Q_BR {
                    newQuad = Q_B;
                } else if newQuad > Q_B {
                    newQuad = Q_BR;
                }
            }
            if newQuad == defQuad {
                // bounce straight back
                let attMove = (*(*attacker).client).ps.saberMove;
                (*(*attacker).client).ps.saberMove =
                    PM_SaberBounceForAttack((*(*attacker).client).ps.saberMove);
                if g_saberDebugPrint.integer != 0 {
                    Com_Printf(&format!(
                        "attack {} vs. parry {} bounced to {}\n",
                        core::ffi::CStr::from_ptr(
                            table[smd[attMove as usize].animToUse as usize].name
                        )
                        .to_string_lossy(),
                        core::ffi::CStr::from_ptr(
                            table[smd[(*(*defender).client).ps.saberMove as usize].animToUse
                                as usize]
                                .name
                        )
                        .to_string_lossy(),
                        core::ffi::CStr::from_ptr(
                            table[smd[(*(*attacker).client).ps.saberMove as usize].animToUse
                                as usize]
                                .name
                        )
                        .to_string_lossy(),
                    ));
                }
                (*(*attacker).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                QFALSE
            }
            // else, pick a deflection
            else {
                let attMove = (*(*attacker).client).ps.saberMove;
                (*(*attacker).client).ps.saberMove = PM_SaberDeflectionForQuad(newQuad);
                if g_saberDebugPrint.integer != 0 {
                    Com_Printf(&format!(
                        "attack {} vs. parry {} deflected to {}\n",
                        core::ffi::CStr::from_ptr(
                            table[smd[attMove as usize].animToUse as usize].name
                        )
                        .to_string_lossy(),
                        core::ffi::CStr::from_ptr(
                            table[smd[(*(*defender).client).ps.saberMove as usize].animToUse
                                as usize]
                                .name
                        )
                        .to_string_lossy(),
                        core::ffi::CStr::from_ptr(
                            table[smd[(*(*attacker).client).ps.saberMove as usize].animToUse
                                as usize]
                                .name
                        )
                        .to_string_lossy(),
                    ));
                }
                (*(*attacker).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                QTRUE
            }
        }
    } else {
        // old math-based method (probably broken)
        let mut att_HitDir: vec3_t = [0.0; 3];
        let mut def_BladeDir: vec3_t = [0.0; 3];
        let mut temp: vec3_t = [0.0; 3];

        VectorCopy(&(*(*attacker).client).lastSaberBase_Always, &mut temp);

        AngleVectors(
            &(*(*attacker).client).lastSaberDir_Always,
            Some(&mut att_HitDir),
            None,
            None,
        );

        AngleVectors(
            &(*(*defender).client).lastSaberDir_Always,
            Some(&mut def_BladeDir),
            None,
            None,
        );

        // now compare
        let hitDot = DotProduct(&att_HitDir, &def_BladeDir);
        if hitDot < 0.25f32 && hitDot > -0.25f32 {
            // hit pretty much perpendicular, pop straight back
            (*(*attacker).client).ps.saberMove =
                PM_SaberBounceForAttack((*(*attacker).client).ps.saberMove);
            (*(*attacker).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
            QFALSE
        } else {
            // a deflection
            let mut att_Right: vec3_t = [0.0; 3];
            let mut att_Up: vec3_t = [0.0; 3];
            let mut att_DeflectionDir: vec3_t = [0.0; 3];

            // get the direction of the deflection
            VectorScale(&def_BladeDir, hitDot, &mut att_DeflectionDir);
            // get our bounce straight back direction
            VectorScale(&att_HitDir, -1.0f32, &mut temp);
            // add the bounce back and deflection
            let att_DeflectionDir_copy = att_DeflectionDir;
            VectorAdd(&att_DeflectionDir_copy, &temp, &mut att_DeflectionDir);
            // normalize the result to determine what direction our saber should bounce back
            // toward
            VectorNormalize(&mut att_DeflectionDir);

            // need to know the direction of the deflectoin relative to the attacker's facing
            VectorSet(
                &mut temp,
                0.0,
                (*(*attacker).client).ps.viewangles[YAW],
                0.0,
            ); // presumes no pitch!
            AngleVectors(&temp, None, Some(&mut att_Right), Some(&mut att_Up));
            let swingRDot = DotProduct(&att_Right, &att_DeflectionDir);
            let swingUDot = DotProduct(&att_Up, &att_DeflectionDir);

            if swingRDot > 0.25f32 {
                // deflect to right
                if swingUDot > 0.25f32 {
                    // deflect to top
                    (*(*attacker).client).ps.saberMove = LS_D1_TR;
                } else if swingUDot < -0.25f32 {
                    // deflect to bottom
                    (*(*attacker).client).ps.saberMove = LS_D1_BR;
                } else {
                    // deflect horizontally
                    (*(*attacker).client).ps.saberMove = LS_D1__R;
                }
            } else if swingRDot < -0.25f32 {
                // deflect to left
                if swingUDot > 0.25f32 {
                    // deflect to top
                    (*(*attacker).client).ps.saberMove = LS_D1_TL;
                } else if swingUDot < -0.25f32 {
                    // deflect to bottom
                    (*(*attacker).client).ps.saberMove = LS_D1_BL;
                } else {
                    // deflect horizontally
                    (*(*attacker).client).ps.saberMove = LS_D1__L;
                }
            } else {
                // deflect in middle
                if swingUDot > 0.25f32 {
                    // deflect to top
                    (*(*attacker).client).ps.saberMove = LS_D1_T_;
                } else if swingUDot < -0.25f32 {
                    // deflect to bottom
                    (*(*attacker).client).ps.saberMove = LS_D1_B_;
                } else {
                    // deflect horizontally?  Well, no such thing as straight back in my face,
                    // so use top
                    if swingRDot > 0.0 {
                        (*(*attacker).client).ps.saberMove = LS_D1_TR;
                    } else if swingRDot < 0.0 {
                        (*(*attacker).client).ps.saberMove = LS_D1_TL;
                    } else {
                        (*(*attacker).client).ps.saberMove = LS_D1_T_;
                    }
                }
            }

            (*(*attacker).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
            QTRUE
        }
    }
}

/// `#define SABER_EXTRAPOLATE_DIST 16.0f` (w_saber.c:2713) — how far each blade's
/// muzzle point/tip is extrapolated forward along its swing before the intersection test.
const SABER_EXTRAPOLATE_DIST: f32 = 16.0;

/// `qboolean WP_SabersIntersect( gentity_t *ent1, int ent1SaberNum, int ent1BladeNum,
/// gentity_t *ent2, qboolean checkDir )` (w_saber.c:2714) — SP-style blade-vs-blade
/// collision: for every active blade of every saber `ent2` holds, builds the swing quad
/// (extrapolated old→new muzzle base/tip, plus [`SABER_EXTRAPOLATE_DIST`]) for `ent1`'s
/// blade `ent1SaberNum`/`ent1BladeNum` and `ent2`'s blade, and runs four [`tri_tri_intersect`]
/// tests across them. When `checkDir` is set it first rejects pairs swinging in the same
/// direction (dot > 0.6) or near-parallel (|dot| > 0.9). Returns `QTRUE` on the first hit.
/// Bails to `QFALSE` for null/clientless ents or if either saber is off.
///
/// The `clc->clientNum`/`FF_Play(fffx_Laser1)` force-feedback lines (present only in the
/// console source tree, absent from the shipped MP module / OpenJK) are omitted, and the
/// `#ifdef DEBUG_SABER_BOX` `G_TestLine` debug draws are omitted (not defined in retail).
/// No oracle — walks `gentity_t`/`gclient_t`/`saberInfo_t`, infeasible to set up in C harness.
///
/// # Safety
/// Dereferences `ent1`/`ent2` and their `client`s when non-null (guarded internally).
#[allow(clippy::too_many_arguments)]
pub unsafe fn WP_SabersIntersect(
    ent1: *mut gentity_t,
    ent1SaberNum: c_int,
    ent1BladeNum: c_int,
    ent2: *mut gentity_t,
    checkDir: qboolean,
) -> qboolean {
    let mut saberBase1: vec3_t = [0.0; 3];
    let mut saberTip1: vec3_t = [0.0; 3];
    let mut saberBaseNext1: vec3_t = [0.0; 3];
    let mut saberTipNext1: vec3_t = [0.0; 3];
    let mut saberBase2: vec3_t = [0.0; 3];
    let mut saberTip2: vec3_t = [0.0; 3];
    let mut saberBaseNext2: vec3_t = [0.0; 3];
    let mut saberTipNext2: vec3_t = [0.0; 3];
    let mut ent2SaberNum: c_int;
    let mut ent2BladeNum: c_int;
    let mut dir: vec3_t = [0.0; 3];

    if ent1.is_null() || ent2.is_null() {
        return QFALSE;
    }
    if (*ent1).client.is_null() || (*ent2).client.is_null() {
        return QFALSE;
    }
    if BG_SabersOff(addr_of_mut!((*(*ent1).client).ps)) == QTRUE
        || BG_SabersOff(addr_of_mut!((*(*ent2).client).ps)) == QTRUE
    {
        return QFALSE;
    }

    ent2SaberNum = 0;
    while ent2SaberNum < MAX_SABERS as c_int {
        if (*(*ent2).client).saber[ent2SaberNum as usize].r#type != SABER_NONE {
            ent2BladeNum = 0;
            while ent2BladeNum < (*(*ent2).client).saber[ent2SaberNum as usize].numBlades {
                if (*(*ent2).client).saber[ent2SaberNum as usize].blade[ent2BladeNum as usize]
                    .lengthMax
                    > 0.0
                {
                    //valid saber and this blade is on
                    //if ( ent1->client->saberInFlight )
                    {
                        let e1b = &(*(*ent1).client).saber[ent1SaberNum as usize].blade
                            [ent1BladeNum as usize];
                        VectorCopy(&e1b.muzzlePointOld, &mut saberBase1);
                        VectorCopy(&e1b.muzzlePoint, &mut saberBaseNext1);

                        VectorSubtract(&e1b.muzzlePoint, &e1b.muzzlePointOld, &mut dir);
                        VectorNormalize(&mut dir);
                        let baseNext1Copy = saberBaseNext1;
                        VectorMA(
                            &baseNext1Copy,
                            SABER_EXTRAPOLATE_DIST,
                            &dir,
                            &mut saberBaseNext1,
                        );

                        VectorMA(
                            &saberBase1,
                            e1b.lengthMax + SABER_EXTRAPOLATE_DIST,
                            &e1b.muzzleDirOld,
                            &mut saberTip1,
                        );
                        VectorMA(
                            &saberBaseNext1,
                            e1b.lengthMax + SABER_EXTRAPOLATE_DIST,
                            &e1b.muzzleDir,
                            &mut saberTipNext1,
                        );

                        VectorSubtract(&saberTipNext1, &saberTip1, &mut dir);
                        VectorNormalize(&mut dir);
                        let tipNext1Copy = saberTipNext1;
                        VectorMA(
                            &tipNext1Copy,
                            SABER_EXTRAPOLATE_DIST,
                            &dir,
                            &mut saberTipNext1,
                        );
                    }

                    //if ( ent2->client->saberInFlight )
                    {
                        let e2b = &(*(*ent2).client).saber[ent2SaberNum as usize].blade
                            [ent2BladeNum as usize];
                        VectorCopy(&e2b.muzzlePointOld, &mut saberBase2);
                        VectorCopy(&e2b.muzzlePoint, &mut saberBaseNext2);

                        VectorSubtract(&e2b.muzzlePoint, &e2b.muzzlePointOld, &mut dir);
                        VectorNormalize(&mut dir);
                        let baseNext2Copy = saberBaseNext2;
                        VectorMA(
                            &baseNext2Copy,
                            SABER_EXTRAPOLATE_DIST,
                            &dir,
                            &mut saberBaseNext2,
                        );

                        VectorMA(
                            &saberBase2,
                            e2b.lengthMax + SABER_EXTRAPOLATE_DIST,
                            &e2b.muzzleDirOld,
                            &mut saberTip2,
                        );
                        VectorMA(
                            &saberBaseNext2,
                            e2b.lengthMax + SABER_EXTRAPOLATE_DIST,
                            &e2b.muzzleDir,
                            &mut saberTipNext2,
                        );

                        VectorSubtract(&saberTipNext2, &saberTip2, &mut dir);
                        VectorNormalize(&mut dir);
                        let tipNext2Copy = saberTipNext2;
                        VectorMA(
                            &tipNext2Copy,
                            SABER_EXTRAPOLATE_DIST,
                            &dir,
                            &mut saberTipNext2,
                        );
                    }

                    if checkDir == QTRUE {
                        //check the direction of the two swings to make sure the sabers are swinging towards each other
                        let mut saberDir1: vec3_t = [0.0; 3];
                        let mut saberDir2: vec3_t = [0.0; 3];

                        VectorSubtract(&saberTipNext1, &saberTip1, &mut saberDir1);
                        VectorSubtract(&saberTipNext2, &saberTip2, &mut saberDir2);
                        VectorNormalize(&mut saberDir1);
                        VectorNormalize(&mut saberDir2);
                        if DotProduct(&saberDir1, &saberDir2) > 0.6 {
                            //sabers moving in same dir, probably didn't actually hit
                            ent2BladeNum += 1;
                            continue;
                        }
                        //now check orientation of sabers, make sure they're not parallel or close to it
                        let dot: f32 = DotProduct(
                            &(*(*ent1).client).saber[ent1SaberNum as usize].blade
                                [ent1BladeNum as usize]
                                .muzzleDir,
                            &(*(*ent2).client).saber[ent2SaberNum as usize].blade
                                [ent2BladeNum as usize]
                                .muzzleDir,
                        );
                        if dot > 0.9 || dot < -0.9 {
                            //too parallel to really block effectively?
                            ent2BladeNum += 1;
                            continue;
                        }
                    }

                    if tri_tri_intersect(
                        &saberBase1,
                        &saberTip1,
                        &saberBaseNext1,
                        &saberBase2,
                        &saberTip2,
                        &saberBaseNext2,
                    ) != 0
                    {
                        return QTRUE;
                    }
                    if tri_tri_intersect(
                        &saberBase1,
                        &saberTip1,
                        &saberBaseNext1,
                        &saberBase2,
                        &saberTip2,
                        &saberTipNext2,
                    ) != 0
                    {
                        return QTRUE;
                    }
                    if tri_tri_intersect(
                        &saberBase1,
                        &saberTip1,
                        &saberTipNext1,
                        &saberBase2,
                        &saberTip2,
                        &saberBaseNext2,
                    ) != 0
                    {
                        return QTRUE;
                    }
                    if tri_tri_intersect(
                        &saberBase1,
                        &saberTip1,
                        &saberTipNext1,
                        &saberBase2,
                        &saberTip2,
                        &saberTipNext2,
                    ) != 0
                    {
                        return QTRUE;
                    }
                }
                ent2BladeNum += 1;
            }
        }
        ent2SaberNum += 1;
    }
    QFALSE
}

/// `static GAME_INLINE int G_PowerLevelForSaberAnim( gentity_t *ent, int saberNum,
/// qboolean mySaberHit )` (w_saber.c:2859) — map the entity's current torso animation to a
/// `FORCE_LEVEL_*` "power level" used by the saber-collision/damage code to decide how strong
/// a swing (or, when `mySaberHit`, how strong a defence) it is.
///
/// Returns [`FORCE_LEVEL_0`] for a bad/clientless entity or an out-of-range `saberNum`. Otherwise
/// it reads the current `torsoAnim`/`torsoTimer`, computes `animTimeElapsed = BG_AnimLength(...) -
/// animTimer` (how far into the anim we are), and classifies the anim: contiguous swing-anim
/// blocks (`BOTH_A1_T__B_`..`BOTH_D7_B____`) map to per-style levels (with the `SABER_LANCE`/
/// `SABER_TRIDENT` special cases), the parry/knockaway block maps to level 1, and a large
/// `switch` handles the special/lunge/spin/superbreak anims — most of which gate their level by
/// where in the anim we are (`animTimer`/`animTimeElapsed` thresholds) so only the "sweet spot"
/// of the swing does damage. `mySaberHit` short-circuits several stab anims to level 1 (defence
/// strength, not damage level). The `BG_JUMPATTACK6` `pm->ps` block from the C is dead code there
/// (the live computation uses the already-derived `animTimer`/`animTimeElapsed`); it is preserved
/// verbatim as a comment.
///
/// # Safety
/// Dereferences `ent` and `ent->client`, and calls [`BG_AnimLength`] (which reads the
/// `bgAllAnims` table); callers must pass a valid entity.
pub unsafe fn G_PowerLevelForSaberAnim(
    ent: *mut gentity_t,
    saberNum: c_int,
    mySaberHit: qboolean,
) -> c_int {
    if ent.is_null() || (*ent).client.is_null() || saberNum >= MAX_SABERS as c_int {
        FORCE_LEVEL_0
    } else {
        let anim = (*(*ent).client).ps.torsoAnim;
        let animTimer = (*(*ent).client).ps.torsoTimer;
        let animTimeElapsed =
            BG_AnimLength((*ent).localAnimIndex, anim as animNumber_t) - animTimer;
        let saber = &(*(*ent).client).saber[saberNum as usize];
        if anim >= BOTH_A1_T__B_ && anim <= BOTH_D1_B____ {
            //FIXME: these two need their own style
            if saber.r#type == SABER_LANCE {
                return FORCE_LEVEL_4;
            } else if saber.r#type == SABER_TRIDENT {
                return FORCE_LEVEL_3;
            }
            return FORCE_LEVEL_1;
        }
        if anim >= BOTH_A2_T__B_ && anim <= BOTH_D2_B____ {
            return FORCE_LEVEL_2;
        }
        if anim >= BOTH_A3_T__B_ && anim <= BOTH_D3_B____ {
            return FORCE_LEVEL_3;
        }
        if anim >= BOTH_A4_T__B_ && anim <= BOTH_D4_B____ {
            //desann
            return FORCE_LEVEL_4;
        }
        if anim >= BOTH_A5_T__B_ && anim <= BOTH_D5_B____ {
            //tavion
            return FORCE_LEVEL_2;
        }
        if anim >= BOTH_A6_T__B_ && anim <= BOTH_D6_B____ {
            //dual
            return FORCE_LEVEL_2;
        }
        if anim >= BOTH_A7_T__B_ && anim <= BOTH_D7_B____ {
            //staff
            return FORCE_LEVEL_2;
        }
        if anim >= BOTH_P1_S1_T_ && anim <= BOTH_H1_S1_BR {
            //parries, knockaways and broken parries
            return FORCE_LEVEL_1; //FIXME: saberAnimLevel?
        }
        match anim {
            BOTH_A2_STABBACK1 => {
                if mySaberHit == QTRUE {
                    //someone else hit my saber, not asking for damage level, but defense strength
                    return FORCE_LEVEL_1;
                }
                if animTimer < 450 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 400 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_ATTACK_BACK => {
                if animTimer < 500 {
                    //end of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_CROUCHATTACKBACK1 => {
                if animTimer < 800 {
                    //end of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_BUTTERFLY_LEFT | BOTH_BUTTERFLY_RIGHT | BOTH_BUTTERFLY_FL1 | BOTH_BUTTERFLY_FR1 => {
                //FIXME: break up?
                FORCE_LEVEL_3
            }
            BOTH_FJSS_TR_BL | BOTH_FJSS_TL_BR => {
                //FIXME: break up?
                FORCE_LEVEL_3
            }
            BOTH_K1_S1_T_ //# knockaway saber top
            | BOTH_K1_S1_TR //# knockaway saber top right
            | BOTH_K1_S1_TL //# knockaway saber top left
            | BOTH_K1_S1_BL //# knockaway saber bottom left
            | BOTH_K1_S1_B_ //# knockaway saber bottom
            | BOTH_K1_S1_BR => {
                //# knockaway saber bottom right
                //FIXME: break up?
                FORCE_LEVEL_3
            }
            BOTH_LUNGE2_B__T_ => {
                if mySaberHit == QTRUE {
                    //someone else hit my saber, not asking for damage level, but defense strength
                    return FORCE_LEVEL_1;
                }
                if animTimer < 400 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 150 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_FORCELEAP2_T__B_ => {
                if animTimer < 400 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 550 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_VS_ATR_S | BOTH_VS_ATL_S | BOTH_VT_ATR_S | BOTH_VT_ATL_S => {
                FORCE_LEVEL_3 //???
            }
            BOTH_JUMPFLIPSLASHDOWN1 => {
                if animTimer <= 1000 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 600 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_JUMPFLIPSTABDOWN => {
                if animTimer <= 1300 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed <= 300 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_JUMPATTACK6 => {
                /*
                if (pm->ps)
                {
                    if ( ( pm->ps->legsAnimTimer >= 1450
                            && BG_AnimLength( g_entities[ps->clientNum].client->clientInfo.animFileIndex, BOTH_JUMPATTACK6 ) - pm->ps->legsAnimTimer >= 400 )
                        ||(pm->ps->legsAnimTimer >= 400
                            && BG_AnimLength( g_entities[ps->clientNum].client->clientInfo.animFileIndex, BOTH_JUMPATTACK6 ) - pm->ps->legsAnimTimer >= 1100 ) )
                    {//pretty much sideways
                        return FORCE_LEVEL_3;
                    }
                }
                */
                if (animTimer >= 1450 && animTimeElapsed >= 400)
                    || (animTimer >= 400 && animTimeElapsed >= 1100)
                {
                    //pretty much sideways
                    return FORCE_LEVEL_3;
                }
                FORCE_LEVEL_0
            }
            BOTH_JUMPATTACK7 => {
                if animTimer <= 1200 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 200 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_SPINATTACK6 => {
                if animTimeElapsed <= 200 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_2 //FORCE_LEVEL_3;
            }
            BOTH_SPINATTACK7 => {
                if animTimer <= 500 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 500 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_2 //FORCE_LEVEL_3;
            }
            BOTH_FORCELONGLEAP_ATTACK => {
                if animTimeElapsed <= 200 {
                    //1st four frames of anim
                    return FORCE_LEVEL_3;
                }
                FORCE_LEVEL_0
            }
            /*
            case BOTH_A7_KICK_F://these kicks attack, too
            case BOTH_A7_KICK_B:
            case BOTH_A7_KICK_R:
            case BOTH_A7_KICK_L:
                //FIXME: break up
                return FORCE_LEVEL_3;
                break;
            */
            BOTH_STABDOWN => {
                if animTimer <= 900 {
                    //end of anim
                    return FORCE_LEVEL_3;
                }
                FORCE_LEVEL_0
            }
            BOTH_STABDOWN_STAFF => {
                if animTimer <= 850 {
                    //end of anim
                    return FORCE_LEVEL_3;
                }
                FORCE_LEVEL_0
            }
            BOTH_STABDOWN_DUAL => {
                if animTimer <= 900 {
                    //end of anim
                    return FORCE_LEVEL_3;
                }
                FORCE_LEVEL_0
            }
            BOTH_A6_SABERPROTECT => {
                if animTimer < 650 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 250 {
                    //start of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_A7_SOULCAL => {
                if animTimer < 650 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 600 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_A1_SPECIAL => {
                if animTimer < 600 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 200 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_A2_SPECIAL => {
                if animTimer < 300 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 200 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_A3_SPECIAL => {
                if animTimer < 700 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 200 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_FLIP_ATTACK7 => FORCE_LEVEL_3,
            BOTH_PULL_IMPALE_STAB => {
                if mySaberHit == QTRUE {
                    //someone else hit my saber, not asking for damage level, but defense strength
                    return FORCE_LEVEL_1;
                }
                if animTimer < 1000 {
                    //end of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_PULL_IMPALE_SWING => {
                if animTimer < 500 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 650 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_ALORA_SPIN_SLASH => {
                if animTimer < 900 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 250 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_A6_FB => {
                if mySaberHit == QTRUE {
                    //someone else hit my saber, not asking for damage level, but defense strength
                    return FORCE_LEVEL_1;
                }
                if animTimer < 250 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 250 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_A6_LR => {
                if mySaberHit == QTRUE {
                    //someone else hit my saber, not asking for damage level, but defense strength
                    return FORCE_LEVEL_1;
                }
                if animTimer < 250 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 250 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_3
            }
            BOTH_A7_HILT => FORCE_LEVEL_0,
            //===SABERLOCK SUPERBREAKS START===========================================================================
            BOTH_LK_S_DL_T_SB_1_W => {
                if animTimer < 700 {
                    //end of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_5
            }
            BOTH_LK_S_ST_S_SB_1_W => {
                if animTimer < 300 {
                    //end of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_5
            }
            BOTH_LK_S_DL_S_SB_1_W | BOTH_LK_S_S_S_SB_1_W => {
                if animTimer < 700 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 400 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_5
            }
            BOTH_LK_S_ST_T_SB_1_W | BOTH_LK_S_S_T_SB_1_W => {
                if animTimer < 150 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 400 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_5
            }
            BOTH_LK_DL_DL_T_SB_1_W => FORCE_LEVEL_5,
            BOTH_LK_DL_DL_S_SB_1_W | BOTH_LK_DL_ST_S_SB_1_W => {
                if animTimeElapsed < 1000 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_5
            }
            BOTH_LK_DL_ST_T_SB_1_W => {
                if animTimer < 950 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 650 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_5
            }
            BOTH_LK_DL_S_S_SB_1_W => {
                if saberNum != 0 {
                    //only right hand saber does damage in this suberbreak
                    return FORCE_LEVEL_0;
                }
                if animTimer < 900 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 450 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_5
            }
            BOTH_LK_DL_S_T_SB_1_W => {
                if saberNum != 0 {
                    //only right hand saber does damage in this suberbreak
                    return FORCE_LEVEL_0;
                }
                if animTimer < 250 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 150 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_5
            }
            BOTH_LK_ST_DL_S_SB_1_W => FORCE_LEVEL_5,
            BOTH_LK_ST_DL_T_SB_1_W => {
                //special suberbreak - doesn't kill, just kicks them backwards
                FORCE_LEVEL_0
            }
            BOTH_LK_ST_ST_S_SB_1_W | BOTH_LK_ST_S_S_SB_1_W => {
                if animTimer < 800 {
                    //end of anim
                    return FORCE_LEVEL_0;
                } else if animTimeElapsed < 350 {
                    //beginning of anim
                    return FORCE_LEVEL_0;
                }
                FORCE_LEVEL_5
            }
            BOTH_LK_ST_ST_T_SB_1_W | BOTH_LK_ST_S_T_SB_1_W => FORCE_LEVEL_5,
            //===SABERLOCK SUPERBREAKS START===========================================================================
            BOTH_HANG_ATTACK => {
                //FIME: break up
                if animTimer < 1000 {
                    //end of anim
                    FORCE_LEVEL_0
                } else if animTimeElapsed < 250 {
                    //beginning of anim
                    FORCE_LEVEL_0
                } else {
                    //sweet spot
                    FORCE_LEVEL_5
                }
            }
            BOTH_ROLL_STAB => {
                if mySaberHit == QTRUE {
                    //someone else hit my saber, not asking for damage level, but defense strength
                    return FORCE_LEVEL_1;
                }
                if animTimeElapsed > 400 {
                    //end of anim
                    FORCE_LEVEL_0
                } else {
                    FORCE_LEVEL_3
                }
            }
            _ => FORCE_LEVEL_0,
        }
    }
}

static mut saberDoClashEffect: qboolean = QFALSE;
static mut saberClashPos: vec3_t = [0.0; 3];
static mut saberClashNorm: vec3_t = [0.0; 3];
static mut saberClashEventParm: c_int = 1;

/// `void WP_SaberDoClash( gentity_t *self, int saberNum, int bladeNum )` (w_saber.c:3798) — if
/// the file-local `saberDoClashEffect` flag is set, spawn an `EV_SABER_BLOCK` temp entity at the
/// recorded clash position/normal and tag it with the clash event parm, the clashing entity
/// number, and the saber/blade numbers.
///
/// # Safety
/// Dereferences `self`; reads/writes process-global statics and spawns a temp entity. Call
/// only from the single game thread.
pub unsafe fn WP_SaberDoClash(self_: *mut gentity_t, saberNum: c_int, bladeNum: c_int) {
    if saberDoClashEffect != QFALSE {
        // FF_Play console-only — omitted (no clc server-side), per WP_SabersIntersect precedent

        let te = G_TempEntity(&*addr_of!(saberClashPos), EV_SABER_BLOCK);
        VectorCopy(&*addr_of!(saberClashPos), &mut (*te).s.origin);
        VectorCopy(&*addr_of!(saberClashNorm), &mut (*te).s.angles);
        (*te).s.eventParm = saberClashEventParm;
        (*te).s.otherEntityNum2 = (*self_).s.number;
        (*te).s.weapon = saberNum;
        (*te).s.legsAnim = bladeNum;
    }
}

/// `void WP_SaberBounceSound( gentity_t *ent, int saberNum, int bladeNum )` (w_saber.c:3812) —
/// plays the bounce/block sound when a saber blade strikes a wall (or otherwise bounces). Always
/// draws `Q_irand(1,9)` (the fallback sound index) up front to keep the RNG sequence faithful.
/// Picks, in order: the per-saber `bounceSound`/`bounce2Sound` then `blockSound`/`block2Sound`
/// (the `2`-suffixed variants when the blade uses the second blade style), falling back to one of
/// the nine generic `saberblock%d.wav` sounds. No oracle: void + side-effecting via [`G_Sound`].
///
/// # Safety
/// `ent` may be null; if non-null with `client` set it must be a valid `gentity_t`. Spawns a
/// sound temp-entity; call only from the single game thread.
pub unsafe fn WP_SaberBounceSound(ent: *mut gentity_t, saberNum: c_int, bladeNum: c_int) {
    if ent.is_null() || (*ent).client.is_null() {
        return;
    }
    let index = Q_irand(1, 9);
    let saber = addr_of_mut!((*(*ent).client).saber[saberNum as usize]);
    if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) == QFALSE && (*saber).bounceSound[0] != 0 {
        G_Sound(ent, CHAN_AUTO, (*saber).bounceSound[Q_irand(0, 2) as usize]);
    } else if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) != QFALSE
        && (*saber).bounce2Sound[0] != 0
    {
        G_Sound(
            ent,
            CHAN_AUTO,
            (*saber).bounce2Sound[Q_irand(0, 2) as usize],
        );
    } else if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) == QFALSE
        && (*saber).blockSound[0] != 0
    {
        G_Sound(ent, CHAN_AUTO, (*saber).blockSound[Q_irand(0, 2) as usize]);
    } else if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) != QFALSE
        && (*saber).block2Sound[0] != 0
    {
        G_Sound(ent, CHAN_AUTO, (*saber).block2Sound[Q_irand(0, 2) as usize]);
    } else {
        G_Sound(
            ent,
            CHAN_AUTO,
            G_SoundIndex(
                &CStr::from_ptr(va(format_args!(
                    "sound/weapons/saber/saberblock{}.wav",
                    index
                )))
                .to_string_lossy(),
            ),
        );
    }
}

// `allow(dead_code)`: written by CheckSaberDamage (for jedi AI); the readers (NPC AI) are not
// yet ported, matching the C file-statics' not-yet-ported-consumer status.
#[allow(dead_code)]
static mut saberHitWall: qboolean = QFALSE; // w_saber.c:3577
#[allow(dead_code)]
static mut saberHitSaber: qboolean = QFALSE; // w_saber.c:3578
#[allow(dead_code)]
static mut saberHitFraction: f32 = 1.0; // w_saber.c:3579

/// `static GAME_INLINE qboolean CheckSaberDamage( gentity_t *self, int rSaberNum, int rBladeNum,
/// vec3_t saberStart, vec3_t saberEnd, qboolean doInterpolate, int trMask, qboolean extrapolate )`
/// (w_saber.c:3588) — rww's MP saber damage function. This is where all the things like blocking,
/// triggering a parry, triggering a broken parry, doing actual damage, etc. are done for the
/// saber. It doesn't resemble the SP version very much, but functionality is (hopefully) about
/// the same. This is a large function. C inlines it because it gets called tons of times per
/// frame; in Rust it is a plain `unsafe fn`.
///
/// Deviations: the C function-local `static`s (`tr`/`dir`/`saberTrMins`/`saberTrMaxs`/
/// `lastValidStart`/`lastValidEnd`/`selfSaberLevel`/`otherSaberLevel`) are all written before
/// read within each call (scratch trace vars), so they are ported as plain locals — their
/// cross-call persistence is never relied upon. `ClientManager::ActiveClientNum()` (used to index
/// the saber trail) renders as `0` per the MP-client-0 convention. The `#ifdef DEBUG_SABER_BOX`
/// `G_TestLine` draw is faithful-omitted (debug-only, undefined in retail). The `goto blockStuff`
/// is rendered as a labeled block.
///
/// # Safety
/// Dereferences `self` (and its `client`), walks `g_entities`, reads cvars/`level`, performs
/// engine traces and mutates entity/player saber-combat state. Caller must pass a valid client
/// entity; trace/entity-state dependent, so no oracle test.
// `allow(dead_code)`: the callers (G_SPSaberDamageTraceLerped / WP_SaberDamageTrace) are not yet ported.
#[allow(dead_code)]
unsafe fn CheckSaberDamage(
    self_: *mut gentity_t,
    rSaberNum: c_int,
    rBladeNum: c_int,
    saberStart: &mut vec3_t,
    saberEnd: &mut vec3_t,
    doInterpolate: qboolean,
    mut trMask: c_int,
    extrapolate: qboolean,
) -> qboolean {
    let mut tr: trace_t;
    let mut dir: vec3_t = [0.0; 3];
    let mut saberTrMins: vec3_t = [0.0; 3];
    let mut saberTrMaxs: vec3_t = [0.0; 3];
    let mut lastValidStart: vec3_t = [0.0; 3];
    let mut lastValidEnd: vec3_t = [0.0; 3];
    let selfSaberLevel: c_int;
    let otherSaberLevel: c_int;
    let otherOwner: *mut gentity_t;
    let mut dmg: c_int = 0;
    let mut attackStr: c_int = 0;
    let mut saberBoxSize: f32 = (*addr_of!(d_saberBoxTraceSize)).value;
    let mut idleDamage: qboolean = QFALSE;
    let mut didHit: qboolean = QFALSE;
    let mut sabersClashed: qboolean = QFALSE;
    let mut unblockable: qboolean = QFALSE;
    let mut didDefense: qboolean = QFALSE;
    let mut didOffense: qboolean = QFALSE;
    let mut saberTraceDone: qboolean = QFALSE;
    let mut otherUnblockable: qboolean;
    let mut tryDeflectAgain: qboolean = QFALSE;

    if BG_SabersOff(&mut (*(*self_).client).ps) != QFALSE {
        return QFALSE;
    }

    selfSaberLevel = G_SaberAttackPower(self_, SaberAttacking(self_));

    //Add the standard radius into the box size
    saberBoxSize +=
        (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize].radius * 0.5;

    if (*(*self_).client).ps.weaponTime <= 0 {
        //if not doing any attacks or anything, just use point traces.
        VectorClear(&mut saberTrMins);
        VectorClear(&mut saberTrMaxs);
    } else if (*addr_of!(d_saberGhoul2Collision)).integer != 0 {
        if (*addr_of!(d_saberSPStyleDamage)).integer != 0 {
            //SP-size saber damage traces
            VectorSet(&mut saberTrMins, -2.0, -2.0, -2.0);
            VectorSet(&mut saberTrMaxs, 2.0, 2.0, 2.0);
        } else {
            VectorSet(
                &mut saberTrMins,
                -saberBoxSize * 3.0,
                -saberBoxSize * 3.0,
                -saberBoxSize * 3.0,
            );
            VectorSet(
                &mut saberTrMaxs,
                saberBoxSize * 3.0,
                saberBoxSize * 3.0,
                saberBoxSize * 3.0,
            );
        }
    } else if (*(*self_).client).ps.fd.saberAnimLevel < FORCE_LEVEL_2 {
        //box trace for fast, because it doesn't get updated so often
        VectorSet(
            &mut saberTrMins,
            -saberBoxSize,
            -saberBoxSize,
            -saberBoxSize,
        );
        VectorSet(&mut saberTrMaxs, saberBoxSize, saberBoxSize, saberBoxSize);
    } else if (*addr_of!(d_saberAlwaysBoxTrace)).integer != 0 {
        VectorSet(
            &mut saberTrMins,
            -saberBoxSize,
            -saberBoxSize,
            -saberBoxSize,
        );
        VectorSet(&mut saberTrMaxs, saberBoxSize, saberBoxSize, saberBoxSize);
    } else {
        //just trace the minimum blade radius
        saberBoxSize =
            (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize].radius * 0.4;

        VectorSet(
            &mut saberTrMins,
            -saberBoxSize,
            -saberBoxSize,
            -saberBoxSize,
        );
        VectorSet(&mut saberTrMaxs, saberBoxSize, saberBoxSize, saberBoxSize);
    }

    tr = trace_t::default();
    while saberTraceDone == QFALSE {
        if doInterpolate != QFALSE && (*addr_of!(d_saberSPStyleDamage)).integer == 0 {
            //This didn't quite work out like I hoped. But it's better than nothing. Sort of.
            let mut oldSaberStart: vec3_t = [0.0; 3];
            let mut oldSaberEnd: vec3_t = [0.0; 3];
            let mut saberDif: vec3_t = [0.0; 3];
            let mut oldSaberDif: vec3_t = [0.0; 3];
            let mut traceTests: c_int = 0;
            let mut trDif: f32 = 8.0;

            // ClientManager::ActiveClientNum() -> 0 (MP)
            if (*addr_of!(level)).time
                - (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                    .trail
                    .lastTime
                > 100
            {
                //no valid last pos, use current
                VectorCopy(saberStart, &mut oldSaberStart);
                VectorCopy(saberEnd, &mut oldSaberEnd);
            } else {
                //trace from last pos
                VectorCopy(
                    &(*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                        .trail
                        .base,
                    &mut oldSaberStart,
                );
                VectorCopy(
                    &(*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                        .trail
                        .tip,
                    &mut oldSaberEnd,
                );
            }

            VectorSubtract(saberStart, saberEnd, &mut saberDif);
            VectorSubtract(&oldSaberStart, &oldSaberEnd, &mut oldSaberDif);

            VectorNormalize(&mut saberDif);
            VectorNormalize(&mut oldSaberDif);

            saberEnd[0] = saberStart[0] - (saberDif[0] * trDif);
            saberEnd[1] = saberStart[1] - (saberDif[1] * trDif);
            saberEnd[2] = saberStart[2] - (saberDif[2] * trDif);

            oldSaberEnd[0] = oldSaberStart[0] - (oldSaberDif[0] * trDif);
            oldSaberEnd[1] = oldSaberStart[1] - (oldSaberDif[1] * trDif);
            oldSaberEnd[2] = oldSaberStart[2] - (oldSaberDif[2] * trDif);

            tr = trap::Trace(
                saberEnd,
                &saberTrMins,
                &saberTrMaxs,
                saberStart,
                (*self_).s.number,
                trMask,
            );

            VectorCopy(saberEnd, &mut lastValidStart);
            VectorCopy(saberStart, &mut lastValidEnd);
            if (tr.entityNum as c_int) < MAX_CLIENTS as c_int {
                G_G2TraceCollide(
                    &mut tr,
                    &lastValidStart,
                    &lastValidEnd,
                    &saberTrMins,
                    &saberTrMaxs,
                );
            } else if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
                let trHit: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities)
                    .cast::<gentity_t>())
                .add(tr.entityNum as usize);

                if (*trHit).inuse != QFALSE && !(*trHit).ghoul2.is_null() {
                    //hit a non-client entity with a g2 instance
                    G_G2TraceCollide(
                        &mut tr,
                        &lastValidStart,
                        &lastValidEnd,
                        &saberTrMins,
                        &saberTrMaxs,
                    );
                }
            }

            trDif += 1.0;

            while tr.fraction == 1.0 && traceTests < 4 && (tr.entityNum as c_int) >= ENTITYNUM_NONE
            {
                // ClientManager::ActiveClientNum() -> 0 (MP)
                if (*addr_of!(level)).time
                    - (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                        .trail
                        .lastTime
                    > 100
                {
                    //no valid last pos, use current
                    VectorCopy(saberStart, &mut oldSaberStart);
                    VectorCopy(saberEnd, &mut oldSaberEnd);
                } else {
                    //trace from last pos
                    VectorCopy(
                        &(*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .base,
                        &mut oldSaberStart,
                    );
                    VectorCopy(
                        &(*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .tip,
                        &mut oldSaberEnd,
                    );
                }

                VectorSubtract(saberStart, saberEnd, &mut saberDif);
                VectorSubtract(&oldSaberStart, &oldSaberEnd, &mut oldSaberDif);

                VectorNormalize(&mut saberDif);
                VectorNormalize(&mut oldSaberDif);

                saberEnd[0] = saberStart[0] - (saberDif[0] * trDif);
                saberEnd[1] = saberStart[1] - (saberDif[1] * trDif);
                saberEnd[2] = saberStart[2] - (saberDif[2] * trDif);

                oldSaberEnd[0] = oldSaberStart[0] - (oldSaberDif[0] * trDif);
                oldSaberEnd[1] = oldSaberStart[1] - (oldSaberDif[1] * trDif);
                oldSaberEnd[2] = oldSaberStart[2] - (oldSaberDif[2] * trDif);

                tr = trap::Trace(
                    saberEnd,
                    &saberTrMins,
                    &saberTrMaxs,
                    saberStart,
                    (*self_).s.number,
                    trMask,
                );

                VectorCopy(saberEnd, &mut lastValidStart);
                VectorCopy(saberStart, &mut lastValidEnd);
                if (tr.entityNum as c_int) < MAX_CLIENTS as c_int {
                    G_G2TraceCollide(
                        &mut tr,
                        &lastValidStart,
                        &lastValidEnd,
                        &saberTrMins,
                        &saberTrMaxs,
                    );
                } else if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
                    let trHit: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities)
                        .cast::<gentity_t>())
                    .add(tr.entityNum as usize);

                    if (*trHit).inuse != QFALSE && !(*trHit).ghoul2.is_null() {
                        //hit a non-client entity with a g2 instance
                        G_G2TraceCollide(
                            &mut tr,
                            &lastValidStart,
                            &lastValidEnd,
                            &saberTrMins,
                            &saberTrMaxs,
                        );
                    }
                }

                traceTests += 1;
                trDif += 8.0;
            }
        } else {
            let mut saberEndExtrapolated: vec3_t = [0.0; 3];
            if extrapolate != QFALSE {
                //extrapolate 16
                let mut diff: vec3_t = [0.0; 3];
                VectorSubtract(saberEnd, saberStart, &mut diff);
                VectorNormalize(&mut diff);
                VectorMA(
                    saberStart,
                    SABER_EXTRAPOLATE_DIST,
                    &diff,
                    &mut saberEndExtrapolated,
                );
            } else {
                VectorCopy(saberEnd, &mut saberEndExtrapolated);
            }
            tr = trap::Trace(
                saberStart,
                &saberTrMins,
                &saberTrMaxs,
                &saberEndExtrapolated,
                (*self_).s.number,
                trMask,
            );

            VectorCopy(saberStart, &mut lastValidStart);
            VectorCopy(&saberEndExtrapolated, &mut lastValidEnd);
            /*
            if ( tr.allsolid || tr.startsolid )
            {
                if ( tr.entityNum == ENTITYNUM_NONE )
                {
                    qboolean whah = qtrue;
                }
                Com_Printf( "saber trace start/all solid - ent is %d\n", tr.entityNum );
            }
            */
            if (tr.entityNum as c_int) < MAX_CLIENTS as c_int {
                G_G2TraceCollide(
                    &mut tr,
                    &lastValidStart,
                    &lastValidEnd,
                    &saberTrMins,
                    &saberTrMaxs,
                );
            } else if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
                let trHit: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities)
                    .cast::<gentity_t>())
                .add(tr.entityNum as usize);

                if (*trHit).inuse != QFALSE && !(*trHit).ghoul2.is_null() {
                    //hit a non-client entity with a g2 instance
                    G_G2TraceCollide(
                        &mut tr,
                        &lastValidStart,
                        &lastValidEnd,
                        &saberTrMins,
                        &saberTrMaxs,
                    );
                }
            }
        }

        saberTraceDone = QTRUE;
    }

    if (SaberAttacking(self_) != QFALSE
        || BG_SuperBreakWinAnim((*(*self_).client).ps.torsoAnim) != QFALSE
        || ((*addr_of!(d_saberSPStyleDamage)).integer != 0
            && (*(*self_).client).ps.saberInFlight != QFALSE
            && rSaberNum == 0)
        || (WP_SaberBladeDoTransitionDamage(
            addr_of_mut!((*(*self_).client).saber[rSaberNum as usize]),
            rBladeNum,
        ) != QFALSE
            && BG_SaberInTransitionAny((*(*self_).client).ps.saberMove) != QFALSE)
        || ((*(*self_).client).ps.m_iVehicleNum != 0 && (*(*self_).client).ps.saberMove > LS_READY))
        && (*(*self_).client).ps.saberAttackWound < (*addr_of!(level)).time
    // NB: the five `||` terms are grouped by the leading paren; `&& saberAttackWound<...` applies to the whole OR-group (C precedence)
    {
        //this animation is that of the last attack movement, and so it should do full damage
        let saberInSpecial: qboolean = BG_SaberInSpecial((*(*self_).client).ps.saberMove);
        let inBackAttack: qboolean = G_SaberInBackAttack((*(*self_).client).ps.saberMove);

        if (*addr_of!(d_saberSPStyleDamage)).integer != 0 {
            // C: `float fDmg = 0.0f;` — every path assigns before read, so the init is dead; deferred here.
            let mut fDmg: f32;
            if (*(*self_).client).ps.saberInFlight != QFALSE {
                let saberEnt: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities)
                    .cast::<gentity_t>())
                .add((*(*self_).client).ps.saberEntityNum as usize);
                if saberEnt.is_null() || (*saberEnt).s.saberInFlight == QFALSE {
                    //does less damage on the way back
                    fDmg = 1.0;
                    attackStr = FORCE_LEVEL_0;
                } else {
                    fDmg = 2.5
                        * (*(*self_).client).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] as f32;
                    attackStr = FORCE_LEVEL_1;
                }
            } else {
                attackStr = G_PowerLevelForSaberAnim(self_, rSaberNum, QFALSE);
                if (*addr_of!(g_saberRealisticCombat)).integer != 0 {
                    match attackStr {
                        FORCE_LEVEL_2 => {
                            fDmg = 5.0;
                        }
                        FORCE_LEVEL_1 | FORCE_LEVEL_0 => {
                            fDmg = 2.5;
                        }
                        // default and FORCE_LEVEL_3
                        _ => {
                            fDmg = 10.0;
                        }
                    }
                } else {
                    if (*(*self_).client).ps.torsoAnim == BOTH_SPINATTACK6 as c_int
                        || (*(*self_).client).ps.torsoAnim == BOTH_SPINATTACK7 as c_int
                    {
                        //too easy to do, lower damage
                        fDmg = 2.5;
                    } else {
                        fDmg = (2.5 * attackStr as f64) as f32; // C: `2.5 * (float)attackStr` (double mul, narrowed)
                    }
                }
            }
            if (*addr_of!(g_saberRealisticCombat)).integer > 1 {
                //always do damage, and lots of it
                if (*addr_of!(g_saberRealisticCombat)).integer > 2 {
                    //always do damage, and lots of it
                    fDmg = 25.0;
                } else if fDmg > 0.1 {
                    //only do super damage if we would have done damage according to normal rules
                    fDmg = 25.0;
                }
            }
            /*
            if ( dmg > 0.1f )
            {
                if ( (self->client->ps.forcePowersActive&(1<<FP_RAGE)) )
                {//add some damage if raged
                    dmg += self->client->ps.forcePowerLevel[FP_RAGE] * 5.0f;
                }
                else if ( self->client->ps.forceRageRecoveryTime )
                {//halve it if recovering
                    dmg *= 0.5f;
                }
            }
            */
            if (*addr_of!(g_gametype)).integer != GT_DUEL
                && (*addr_of!(g_gametype)).integer != GT_POWERDUEL
                && (*addr_of!(g_gametype)).integer != GT_SIEGE
            {
                //in faster-paced games, sabers do more damage
                fDmg *= 2.0;
            }
            if fDmg != 0.0 {
                //the longer the trace, the more damage it does
                //FIXME: in SP, we only use the part of the trace that's actually *inside* the hit ent...
                let traceLength: f32 = Distance(saberEnd, saberStart);
                if tr.fraction >= 1.0 {
                    //allsolid?
                    dmg = (fDmg * traceLength * 0.1 * 0.33).ceil() as c_int;
                } else {
                    //fractional hit, the sooner you hit in the trace, the more damage you did
                    dmg = (fDmg * traceLength * (1.0 - tr.fraction) * 0.1 * 0.33).ceil() as c_int;
                    //(1.0f-tr.fraction) isn't really accurate, but kind of simulates what we have in SP
                }
                // #ifdef DEBUG_SABER_BOX G_TestLine debug draw — faithful-omit (undefined in retail)
            }
            /*
            if ( dmg )
            {
                Com_Printf("CL %i SABER DMG: %i, anim %s, torsoTimer %i\n", self->s.number, dmg, animTable[self->client->ps.torsoAnim].name, self->client->ps.torsoTimer );
            }
            */
            if (*(*self_).client).ps.torsoAnim == BOTH_A1_SPECIAL as c_int
                || (*(*self_).client).ps.torsoAnim == BOTH_A2_SPECIAL as c_int
                || (*(*self_).client).ps.torsoAnim == BOTH_A3_SPECIAL as c_int
            {
                //parry/block/break-parry bonus for single-style kata moves
                attackStr += 1;
            }
            if BG_SuperBreakWinAnim((*(*self_).client).ps.torsoAnim) != QFALSE {
                trMask &= !CONTENTS_LIGHTSABER;
            }
        } else {
            dmg = SABER_HITDAMAGE;

            if (*(*self_).client).ps.fd.saberAnimLevel == SS_STAFF
                || (*(*self_).client).ps.fd.saberAnimLevel == SS_DUAL
            {
                if saberInSpecial != QFALSE {
                    //it will get auto-ramped based on the point in the attack, later on
                    if (*(*self_).client).ps.saberMove == LS_SPINATTACK
                        || (*(*self_).client).ps.saberMove == LS_SPINATTACK_DUAL
                    {
                        //these attacks are long and have the potential to hit a lot so they will do less damage.
                        dmg = 10;
                    } else {
                        if BG_KickingAnim((*(*self_).client).ps.legsAnim) != QFALSE
                            || BG_KickingAnim((*(*self_).client).ps.torsoAnim) != QFALSE
                        {
                            //saber shouldn't do more than min dmg during kicks
                            dmg = 2;
                        } else if BG_SaberInKata((*(*self_).client).ps.saberMove) != QFALSE {
                            //special kata move
                            if (*(*self_).client).ps.fd.saberAnimLevel == SS_DUAL {
                                //this is the nasty saber twirl, do big damage cause it makes you vulnerable
                                dmg = 90;
                            } else {
                                //staff kata
                                dmg = G_GetAttackDamage(self_, 60, 70, 0.5);
                            }
                        } else {
                            //dmg = 90;
                            //ramp from 2 to 90 by default for other specials
                            dmg = G_GetAttackDamage(self_, 2, 90, 0.5);
                        }
                    }
                } else {
                    //otherwise we'll ramp up to 70 I guess, for both dual and staff
                    dmg = G_GetAttackDamage(self_, 2, 70, 0.5);
                }
            } else if (*(*self_).client).ps.fd.saberAnimLevel == 3 {
                //new damage-ramping system
                if saberInSpecial == QFALSE && inBackAttack == QFALSE {
                    dmg = G_GetAttackDamage(self_, 2, 120, 0.5);
                } else if saberInSpecial != QFALSE
                    && ((*(*self_).client).ps.saberMove == LS_A_JUMP_T__B_)
                {
                    dmg = G_GetAttackDamage(self_, 2, 180, 0.65);
                } else if inBackAttack != QFALSE {
                    dmg = G_GetAttackDamage(self_, 2, 30, 0.5); //can hit multiple times (and almost always does), so..
                } else {
                    dmg = 100;
                }
            } else if (*(*self_).client).ps.fd.saberAnimLevel == 2 {
                if saberInSpecial != QFALSE
                    && ((*(*self_).client).ps.saberMove == LS_A_FLIP_STAB
                        || (*(*self_).client).ps.saberMove == LS_A_FLIP_SLASH)
                {
                    //a well-timed hit with this can do a full 85
                    dmg = G_GetAttackDamage(self_, 2, 80, 0.5);
                } else if inBackAttack != QFALSE {
                    dmg = G_GetAttackDamage(self_, 2, 25, 0.5);
                } else {
                    dmg = 60;
                }
            } else if (*(*self_).client).ps.fd.saberAnimLevel == 1 {
                if saberInSpecial != QFALSE && ((*(*self_).client).ps.saberMove == LS_A_LUNGE) {
                    dmg = G_GetAttackDamage(self_, 2, SABER_HITDAMAGE - 5, 0.3);
                } else if inBackAttack != QFALSE {
                    dmg = G_GetAttackDamage(self_, 2, 30, 0.5);
                } else {
                    dmg = SABER_HITDAMAGE;
                }
            }

            attackStr = (*(*self_).client).ps.fd.saberAnimLevel;
        }
    } else if (*(*self_).client).ps.saberAttackWound < (*addr_of!(level)).time
        && (*(*self_).client).ps.saberIdleWound < (*addr_of!(level)).time
    {
        //just touching, do minimal damage and only check for it every 200ms (mainly to cut down on network traffic for hit events)
        if (*(*self_).client).saber[0].saberFlags2 & SFL2_NO_IDLE_EFFECT != 0 {
            //no idle damage or effects
            return QTRUE; //true cause even though we didn't get a hit, we don't want to do those extra traces because the debounce time says not to.
        }
        trMask &= !CONTENTS_LIGHTSABER;
        if (*addr_of!(d_saberSPStyleDamage)).integer != 0 {
            if BG_SaberInReturn((*(*self_).client).ps.saberMove) != QFALSE {
                dmg = SABER_NONATTACK_DAMAGE;
            } else {
                if (*addr_of!(d_saberSPStyleDamage)).integer == 2 {
                    dmg = SABER_NONATTACK_DAMAGE;
                } else {
                    dmg = 0;
                }
            }
        } else {
            dmg = SABER_NONATTACK_DAMAGE;
        }
        idleDamage = QTRUE;
    } else {
        return QTRUE; //true cause even though we didn't get a hit, we don't want to do those extra traces because the debounce time says not to.
    }

    if BG_SaberInSpecial((*(*self_).client).ps.saberMove) != QFALSE {
        let inBackAttack: qboolean = G_SaberInBackAttack((*(*self_).client).ps.saberMove);

        unblockable = QTRUE;
        (*(*self_).client).ps.saberBlocked = 0;

        if (*addr_of!(d_saberSPStyleDamage)).integer != 0 {
        } else if inBackAttack == QFALSE {
            if (*(*self_).client).ps.saberMove == LS_A_JUMP_T__B_ {
                //do extra damage for special unblockables
                dmg += 5; //This is very tiny, because this move has a huge damage ramp
            } else if (*(*self_).client).ps.saberMove == LS_A_FLIP_STAB
                || (*(*self_).client).ps.saberMove == LS_A_FLIP_SLASH
            {
                dmg += 5; //ditto
                if dmg <= 40 || G_GetAnimPoint(self_) <= 0.4 {
                    //sort of a hack, don't want it doing big damage in the off points of the anim
                    dmg = 2;
                }
            } else if (*(*self_).client).ps.saberMove == LS_A_LUNGE {
                dmg += 2; //and ditto again
                if G_GetAnimPoint(self_) <= 0.4 {
                    //same as above
                    dmg = 2;
                }
            } else if (*(*self_).client).ps.saberMove == LS_SPINATTACK
                || (*(*self_).client).ps.saberMove == LS_SPINATTACK_DUAL
            {
                //do a constant significant amount of damage but ramp up a little to the mid-point
                dmg = G_GetAttackDamage(self_, 2, dmg + 3, 0.5);
                dmg += 10;
            } else {
                //dmg += 20;
                if BG_KickingAnim((*(*self_).client).ps.legsAnim) != QFALSE
                    || BG_KickingAnim((*(*self_).client).ps.torsoAnim) != QFALSE
                {
                    //saber shouldn't do more than min dmg during kicks
                    dmg = 2;
                } else {
                    //auto-ramp it I guess since it's a special we don't have a special case for.
                    dmg = G_GetAttackDamage(self_, 5, dmg + 5, 0.5);
                }
            }
        }
    }

    if dmg == 0 {
        if (tr.entityNum as c_int) < MAX_CLIENTS as c_int
            || ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .inuse
                != QFALSE
                && ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize))
                .r
                .contents
                    & CONTENTS_LIGHTSABER)
                    != 0)
        {
            return QTRUE;
        }
        return QFALSE;
    }

    if dmg > SABER_NONATTACK_DAMAGE {
        dmg = (dmg as f32 * (*addr_of!(g_saberDamageScale)).value) as c_int;

        //see if this specific saber has a damagescale
        if WP_SaberBladeUseSecondBladeStyle(
            addr_of_mut!((*(*self_).client).saber[rSaberNum as usize]),
            rBladeNum,
        ) == QFALSE
            && (*(*self_).client).saber[rSaberNum as usize].damageScale != 1.0
        {
            dmg = (dmg as f32 * (*(*self_).client).saber[rSaberNum as usize].damageScale).ceil()
                as c_int;
        } else if WP_SaberBladeUseSecondBladeStyle(
            addr_of_mut!((*(*self_).client).saber[rSaberNum as usize]),
            rBladeNum,
        ) != QFALSE
            && (*(*self_).client).saber[rSaberNum as usize].damageScale2 != 1.0
        {
            dmg = (dmg as f32 * (*(*self_).client).saber[rSaberNum as usize].damageScale2).ceil()
                as c_int;
        }

        if ((*(*self_).client).ps.brokenLimbs & (1 << BROKENLIMB_RARM)) != 0
            || ((*(*self_).client).ps.brokenLimbs & (1 << BROKENLIMB_LARM)) != 0
        {
            //weaken it if an arm is broken
            dmg = (dmg as f64 * 0.3) as c_int;
            if dmg <= SABER_NONATTACK_DAMAGE {
                dmg = SABER_NONATTACK_DAMAGE + 1;
            }
        }
    }
    if dmg > SABER_NONATTACK_DAMAGE && (*(*self_).client).ps.isJediMaster != QFALSE {
        //give the Jedi Master more saber attack power
        dmg *= 2;
    }

    if dmg > SABER_NONATTACK_DAMAGE
        && (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*(*self_).client).siegeClass != -1
        && (bgSiegeClasses[(*(*self_).client).siegeClass as usize].classflags
            & (1 << CFL_MORESABERDMG))
            != 0
    {
        //this class is flagged to do extra saber damage. I guess 2x will do for now.
        dmg *= 2;
    }

    if (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        && (*(*self_).client).sess.duelTeam == DUELTEAM_LONE
    {
        //always x2 when we're powerdueling alone... er, so, we apparently no longer want this?  So they say.
        if (*addr_of!(g_duel_fraglimit)).integer != 0 {
            //dmg *= 1.5 - (.4 * (float)self->client->sess.wins / (float)g_duel_fraglimit.integer);
        }
        //dmg *= 2;
    }

    // #ifndef FINAL_BUILD — retail honors the runtime cvar gate (wrapper dropped per deviation)
    if (*addr_of!(g_saberDebugPrint)).integer > 2 && dmg > 1 {
        Com_Printf(&format!("CL {} SABER DMG: {}\n", (*self_).s.number, dmg));
    }

    VectorSubtract(saberEnd, saberStart, &mut dir);
    VectorNormalize(&mut dir);

    if (tr.entityNum as c_int) == ENTITYNUM_WORLD
        || (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize))
            .s
            .eType
            == ET_TERRAIN
    {
        //register this as a wall hit for jedi AI
        (*(*self_).client).ps.saberEventFlags |= SEF_HITWALL;
        saberHitWall = QTRUE;
    }

    if saberHitWall != QFALSE
        && ((*(*self_).client).saber[rSaberNum as usize].saberFlags & SFL_BOUNCE_ON_WALLS) != 0
        && (BG_SaberInAttackPure((*(*self_).client).ps.saberMove) != QFALSE //only in a normal attack anim
            || (*(*self_).client).ps.saberMove == LS_A_JUMP_T__B_)
    //or in the strong jump-fwd-attack "death from above" move
    {
        //then bounce off
        /*
        qboolean onlyIfNotSpecial = qfalse;
        qboolean skipIt = qfalse;
        if (tr.plane.normal[2] >= 0.8f ||
            tr.plane.normal[2] <= -0.8f ||
            VectorCompare(tr.plane.normal, vec3_origin))
        {
            if ((tr.plane.normal[2] >= 0.8f || VectorCompare(tr.plane.normal, vec3_origin)) &&
                self->client->ps.viewangles[PITCH] <= 30.0f &&
                self->client->pers.cmd.upmove >= 0)
            { //don't hit the ground if we are not looking down a lot/crouched
                skipIt = qtrue;
            }
            else
            {
                onlyIfNotSpecial = qtrue;
            }
        }
        if (!skipIt && (!onlyIfNotSpecial || !BG_SaberInSpecial(self->client->ps.saberMove)))
        */
        {
            let te: *mut gentity_t;
            /*
            qboolean pre = saberDoClashEffect;

            VectorCopy( tr.endpos, saberClashPos );
            VectorCopy( tr.plane.normal, saberClashNorm );
            saberClashEventParm = 1;
            saberDoClashEffect = qtrue;
            WP_SaberDoClash( self, rSaberNum, rBladeNum );
            saberDoClashEffect = pre;
            */

            (*(*self_).client).ps.saberMove =
                BG_BrokenParryForAttack((*(*self_).client).ps.saberMove);
            (*(*self_).client).ps.saberBlocked = BLOCKED_PARRY_BROKEN;
            if (*(*self_).client).ps.torsoAnim == (*(*self_).client).ps.legsAnim {
                //set anim now on both parts
                let anim = saberMoveData[(*(*self_).client).ps.saberMove as usize].animToUse;
                G_SetAnim(
                    self_,
                    &mut (*(*self_).client).pers.cmd,
                    SETANIM_BOTH,
                    anim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            }

            //do bounce sound & force feedback
            WP_SaberBounceSound(self_, rSaberNum, rBladeNum);
            //do hit effect
            te = G_TempEntity(&tr.endpos, EV_SABER_HIT);
            (*te).s.otherEntityNum = ENTITYNUM_NONE; //we didn't hit anyone in particular
            (*te).s.otherEntityNum2 = (*self_).s.number; //send this so it knows who we are
            (*te).s.weapon = rSaberNum;
            (*te).s.legsAnim = rBladeNum;
            VectorCopy(&tr.endpos, &mut (*te).s.origin);
            VectorCopy(&tr.plane.normal, &mut (*te).s.angles);
            if (*te).s.angles[0] == 0.0 && (*te).s.angles[1] == 0.0 && (*te).s.angles[2] == 0.0 {
                //don't let it play with no direction
                (*te).s.angles[1] = 1.0;
            }
            //do radius damage/knockback, if any
            if WP_SaberBladeUseSecondBladeStyle(
                addr_of_mut!((*(*self_).client).saber[rSaberNum as usize]),
                rBladeNum,
            ) == QFALSE
            {
                WP_SaberRadiusDamage(
                    self_,
                    &tr.endpos,
                    (*(*self_).client).saber[rSaberNum as usize].splashRadius,
                    (*(*self_).client).saber[rSaberNum as usize].splashDamage,
                    (*(*self_).client).saber[rSaberNum as usize].splashKnockback,
                );
            } else {
                WP_SaberRadiusDamage(
                    self_,
                    &tr.endpos,
                    (*(*self_).client).saber[rSaberNum as usize].splashRadius2,
                    (*(*self_).client).saber[rSaberNum as usize].splashDamage2,
                    (*(*self_).client).saber[rSaberNum as usize].splashKnockback2,
                );
            }

            return QTRUE;
        }
    }

    //rww - I'm saying || tr.startsolid here, because otherwise your saber tends to skip positions and go through
    //people, and the compensation traces start in their bbox too. Which results in the saber passing through people
    //when you visually cut right through them. Which sucks.

    // The C `goto blockStuff;` (from the flesh-hit / can-block path) jumps into the tail of the
    // saber-clash branch (the `blockStuff:` label). Both that goto-path and the clash branch's
    // own fall-through reach the shared `blockStuff` tail; the flesh-damage `else` path and the
    // "neither branch" path skip it. Modeled here as a single `'blockStuff` labeled block:
    // `break 'blockStuff` == "skip the shared tail" (C fall-to-return), while falling off the end
    // of the block == reaching the `blockStuff:` label. `otherOwner`/`otherUnblockable` are set
    // before the tail runs in each reaching path.
    'blockStuff: {
        if (tr.fraction != 1.0 || tr.startsolid != 0)
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .takedamage
                != QFALSE
            && ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .health
                > 0
                || ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize))
                .s
                .eFlags
                    & EF_DISINTEGRATION)
                    == 0)
            && (tr.entityNum as c_int) != (*self_).s.number
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .inuse
                != QFALSE
        {
            //hit something that had health and takes damage
            if idleDamage != QFALSE
                && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize))
                .client
                .is_null()
                && OnSameTeam(
                    self_,
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize),
                ) != QFALSE
                && (*addr_of!(g_friendlySaber)).integer == 0
            {
                return QFALSE;
            }

            if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .client
            .is_null()
                && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize))
                .client)
                    .ps
                    .duelInProgress
                    != QFALSE
                && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize))
                .client)
                    .ps
                    .duelIndex
                    != (*self_).s.number
            {
                return QFALSE;
            }

            if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .client
            .is_null()
                && (*(*self_).client).ps.duelInProgress != QFALSE
                && (*(*self_).client).ps.duelIndex
                    != (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize))
                    .s
                    .number
            {
                return QFALSE;
            }

            if BG_StabDownAnim((*(*self_).client).ps.torsoAnim) != QFALSE
                && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize))
                .client
                .is_null()
                && BG_InKnockDownOnGround(
                    &mut (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize))
                    .client)
                        .ps,
                ) == QFALSE
            {
                //stabdowns only damage people who are actually on the ground...
                return QFALSE;
            }
            (*(*self_).client).ps.saberIdleWound =
                (*addr_of!(level)).time + (*addr_of!(g_saberDmgDelay_Idle)).integer;

            didHit = QTRUE;

            if (*addr_of!(d_saberSPStyleDamage)).integer == 0 //let's trying making blocks have to be blocked by a saber
            && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize)).client.is_null()
            && unblockable == QFALSE
            && WP_SaberCanBlock(
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize),
                &tr.endpos,
                0,
                MOD_SABER,
                QFALSE,
                attackStr,
            ) != 0
            {
                //hit a client who blocked the attack (fake: didn't actually hit their saber)
                if dmg <= SABER_NONATTACK_DAMAGE {
                    (*(*self_).client).ps.saberIdleWound =
                        (*addr_of!(level)).time + (*addr_of!(g_saberDmgDelay_Idle)).integer;
                }
                saberDoClashEffect = QTRUE;
                VectorCopy(&tr.endpos, &mut *addr_of_mut!(saberClashPos));
                VectorCopy(&tr.plane.normal, &mut *addr_of_mut!(saberClashNorm));
                saberClashEventParm = 1;

                if dmg > SABER_NONATTACK_DAMAGE {
                    let lockFactor: c_int = (*addr_of!(g_saberLockFactor)).integer;

                    if ((*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize))
                    .client)
                        .ps
                        .fd
                        .forcePowerLevel[FP_SABER_OFFENSE as usize]
                        - (*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize])
                        > 1
                        && Q_irand(1, 10) < lockFactor * 2
                    {
                        //Just got blocked by someone with a decently higher attack level, so enter into a lock (where they have the advantage due to a higher attack lev)
                        if G_ClientIdleInWorld(
                            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                .add(tr.entityNum as usize),
                        ) == QFALSE
                        {
                            if (trMask & CONTENTS_LIGHTSABER) != 0
                                && WP_SabersCheckLock(
                                    self_,
                                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                        .add(tr.entityNum as usize),
                                ) != QFALSE
                            {
                                (*(*self_).client).ps.saberBlocked = BLOCKED_NONE;
                                (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                    .add(tr.entityNum as usize))
                                .client)
                                    .ps
                                    .saberBlocked = BLOCKED_NONE;
                                return didHit;
                            }
                        }
                    } else if Q_irand(1, 20) < lockFactor {
                        if G_ClientIdleInWorld(
                            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                .add(tr.entityNum as usize),
                        ) == QFALSE
                        {
                            if (trMask & CONTENTS_LIGHTSABER) != 0
                                && WP_SabersCheckLock(
                                    self_,
                                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                        .add(tr.entityNum as usize),
                                ) != QFALSE
                            {
                                (*(*self_).client).ps.saberBlocked = BLOCKED_NONE;
                                (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                    .add(tr.entityNum as usize))
                                .client)
                                    .ps
                                    .saberBlocked = BLOCKED_NONE;
                                return didHit;
                            }
                        }
                    }
                }
                otherOwner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize);
                // goto blockStuff; — fall through to the shared blockStuff tail below
            } else {
                //damage the thing we hit
                let mut doDismemberment: qboolean = QFALSE;
                let mut knockbackFlags: c_int = 0;

                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize))
                .client
                .is_null()
                    && ((*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize))
                    .client)
                        .ps
                        .weapon
                        != WP_SABER)
                //fd.forcePowerLevel[FP_SABER_OFFENSE])
                {
                    //not a "jedi", so make them suffer more
                    if dmg > SABER_NONATTACK_DAMAGE {
                        //don't bother increasing just for idle touch damage
                        dmg = (dmg as f64 * 1.5) as c_int;
                    }
                }

                if (*addr_of!(d_saberSPStyleDamage)).integer == 0 {
                    if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize))
                    .client
                    .is_null()
                        && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .add(tr.entityNum as usize))
                        .client)
                            .ps
                            .weapon
                            == WP_SABER
                    {
                        //for jedi using the saber, half the damage (this comes with the increased default dmg debounce time)
                        if (*addr_of!(g_gametype)).integer != GT_SIEGE {
                            //unless siege..
                            if dmg > SABER_NONATTACK_DAMAGE && unblockable == QFALSE {
                                //don't reduce damage if it's only 1, or if this is an unblockable attack
                                if dmg == SABER_HITDAMAGE {
                                    //level 1 attack
                                    dmg = (dmg as f64 * 0.7) as c_int;
                                } else {
                                    dmg = (dmg as f64 * 0.5) as c_int;
                                }
                            }
                        }
                    }
                }

                if (*self_).s.eType == ET_NPC
                    && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize))
                    .client
                    .is_null()
                    && (*(*self_).client).playerTeam
                        == (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .add(tr.entityNum as usize))
                        .client)
                            .playerTeam
                {
                    //Oops. Since he's an NPC, we'll be forgiving and cut the damage down.
                    dmg = (dmg as f32 * 0.2) as c_int;
                }

                //store the damage, we'll apply it later
                if WP_SaberBladeUseSecondBladeStyle(
                    addr_of_mut!((*(*self_).client).saber[rSaberNum as usize]),
                    rBladeNum,
                ) == QFALSE
                    && (*(*self_).client).saber[rSaberNum as usize].saberFlags2
                        & SFL2_NO_DISMEMBERMENT
                        == 0
                {
                    doDismemberment = QTRUE;
                }
                if WP_SaberBladeUseSecondBladeStyle(
                    addr_of_mut!((*(*self_).client).saber[rSaberNum as usize]),
                    rBladeNum,
                ) != QFALSE
                    && (*(*self_).client).saber[rSaberNum as usize].saberFlags2
                        & SFL2_NO_DISMEMBERMENT
                        == 0
                {
                    doDismemberment = QTRUE;
                }

                if WP_SaberBladeUseSecondBladeStyle(
                    addr_of_mut!((*(*self_).client).saber[rSaberNum as usize]),
                    rBladeNum,
                ) == QFALSE
                    && (*(*self_).client).saber[rSaberNum as usize].knockbackScale > 0.0
                {
                    if rSaberNum < 1 {
                        knockbackFlags = DAMAGE_SABER_KNOCKBACK1;
                    } else {
                        knockbackFlags = DAMAGE_SABER_KNOCKBACK2;
                    }
                }

                if WP_SaberBladeUseSecondBladeStyle(
                    addr_of_mut!((*(*self_).client).saber[rSaberNum as usize]),
                    rBladeNum,
                ) != QFALSE
                    && (*(*self_).client).saber[rSaberNum as usize].knockbackScale > 0.0
                {
                    if rSaberNum < 1 {
                        knockbackFlags = DAMAGE_SABER_KNOCKBACK1_B2;
                    } else {
                        knockbackFlags = DAMAGE_SABER_KNOCKBACK2_B2;
                    }
                }

                WP_SaberDamageAdd(
                    tr.entityNum as c_int,
                    &dir,
                    &tr.endpos,
                    dmg,
                    doDismemberment,
                    knockbackFlags,
                );

                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize))
                .client
                .is_null()
                {
                    //Let jedi AI know if it hit an enemy
                    if !(*self_).enemy.is_null()
                        && (*self_).enemy
                            == (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                .add(tr.entityNum as usize)
                    {
                        (*(*self_).client).ps.saberEventFlags |= SEF_HITENEMY;
                    } else {
                        (*(*self_).client).ps.saberEventFlags |= SEF_HITOBJECT;
                    }
                }

                if (*addr_of!(d_saberSPStyleDamage)).integer != 0 {
                } else {
                    (*(*self_).client).ps.saberAttackWound = (*addr_of!(level)).time + 100;
                }

                //damage path: C falls past the whole if/else-if to `return didHit`, skipping blockStuff
                break 'blockStuff;
            }
        } else if (tr.fraction != 1.0 || tr.startsolid != 0)
            && ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .r
            .contents
                & CONTENTS_LIGHTSABER)
                != 0
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .r
            .contents
                != -1
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize))
            .inuse
                != QFALSE
        {
            //saber clash
            otherOwner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(
                (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize))
                .r
                .ownerNum as usize,
            );

            if (*otherOwner).inuse == QFALSE || (*otherOwner).client.is_null() {
                return QFALSE;
            }

            if !otherOwner.is_null()
                && !(*otherOwner).client.is_null()
                && (*(*otherOwner).client).ps.saberInFlight != QFALSE
            {
                //don't do extra collision checking vs sabers in air
            } else {
                //hit an in-hand saber, do extra collision check against it
                if (*addr_of!(d_saberSPStyleDamage)).integer != 0 {
                    //use SP-style blade-collision test
                    if WP_SabersIntersect(self_, rSaberNum, rBladeNum, otherOwner, QFALSE) == QFALSE
                    {
                        //sabers did not actually intersect
                        return QFALSE;
                    }
                } else {
                    //MP-style
                    if G_SaberCollide(
                        self_,
                        otherOwner,
                        &lastValidStart,
                        &lastValidEnd,
                        &mut saberTrMins,
                        &mut saberTrMaxs,
                        &mut tr.endpos,
                    ) == QFALSE
                    {
                        //detailed collision did not produce results...
                        return QFALSE;
                    }
                }
            }

            if OnSameTeam(self_, otherOwner) != QFALSE && (*addr_of!(g_friendlySaber)).integer == 0
            {
                return QFALSE;
            }

            if ((*self_).s.eType == ET_NPC || (*otherOwner).s.eType == ET_NPC) //just make sure one of us is an npc
            && (*(*self_).client).playerTeam == (*(*otherOwner).client).playerTeam
            && (*addr_of!(g_gametype)).integer != GT_SIEGE
            {
                //don't hit your teammate's sabers if you are an NPC. It can be rather annoying.
                return QFALSE;
            }

            if (*(*otherOwner).client).ps.duelInProgress != QFALSE
                && (*(*otherOwner).client).ps.duelIndex != (*self_).s.number
            {
                return QFALSE;
            }

            if (*(*self_).client).ps.duelInProgress != QFALSE
                && (*(*self_).client).ps.duelIndex != (*otherOwner).s.number
            {
                return QFALSE;
            }

            if (*addr_of!(g_debugSaberLocks)).integer != 0 {
                WP_SabersCheckLock2(self_, otherOwner, LOCK_RANDOM);
                return QTRUE;
            }
            didHit = QTRUE;
            (*(*self_).client).ps.saberIdleWound =
                (*addr_of!(level)).time + (*addr_of!(g_saberDmgDelay_Idle)).integer;

            if dmg <= SABER_NONATTACK_DAMAGE {
                (*(*self_).client).ps.saberIdleWound =
                    (*addr_of!(level)).time + (*addr_of!(g_saberDmgDelay_Idle)).integer;
            }

            saberDoClashEffect = QTRUE;
            VectorCopy(&tr.endpos, &mut *addr_of_mut!(saberClashPos));
            VectorCopy(&tr.plane.normal, &mut *addr_of_mut!(saberClashNorm));
            saberClashEventParm = 1;

            sabersClashed = QTRUE;
            saberHitSaber = QTRUE;
            saberHitFraction = tr.fraction;

            if saberCheckKnockdown_Smashed(
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(tr.entityNum as usize),
                otherOwner,
                self_,
                dmg,
            ) != QFALSE
            {
                //smashed it out of the air
                return QFALSE;
            }

            //is this my thrown saber?
            if (*(*self_).client).ps.saberEntityNum != 0
                && (*(*self_).client).ps.saberInFlight != QFALSE
                && rSaberNum == 0
                && saberCheckKnockdown_Smashed(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*self_).client).ps.saberEntityNum as usize),
                    self_,
                    otherOwner,
                    dmg,
                ) != QFALSE
            {
                //they smashed it out of the air
                return QFALSE;
            }
            // fall through to the shared blockStuff tail below
        } else {
            //neither a flesh hit nor a saber clash — C falls straight to `return didHit`
            break 'blockStuff;
        }

        // --- blockStuff: (w_saber.c:4433) ---
        // Reached by the saber-clash branch's fall-through, and by the flesh-hit/can-block path's
        // `goto blockStuff` (which set `otherOwner` above and skipped this branch's `else if`).
        otherUnblockable = QFALSE;

        if !otherOwner.is_null()
            && !(*otherOwner).client.is_null()
            && (*(*otherOwner).client).ps.saberInFlight != QFALSE
        {
            return QFALSE;
        }

        //this is a thrown saber, don't do any fancy saber-saber collision stuff
        if (*(*self_).client).ps.saberEntityNum != 0
            && (*(*self_).client).ps.saberInFlight != QFALSE
            && rSaberNum == 0
        {
            return QFALSE;
        }

        otherSaberLevel = G_SaberAttackPower(otherOwner, SaberAttacking(otherOwner));

        if dmg > SABER_NONATTACK_DAMAGE && unblockable == QFALSE && otherUnblockable == QFALSE {
            let lockFactor: c_int = (*addr_of!(g_saberLockFactor)).integer;

            if sabersClashed != QFALSE && Q_irand(1, 20) <= lockFactor {
                if G_ClientIdleInWorld(otherOwner) == QFALSE {
                    if WP_SabersCheckLock(self_, otherOwner) != QFALSE {
                        (*(*self_).client).ps.saberBlocked = BLOCKED_NONE;
                        (*(*otherOwner).client).ps.saberBlocked = BLOCKED_NONE;
                        return didHit;
                    }
                }
            }
        }

        if otherOwner.is_null() || (*otherOwner).client.is_null() {
            return didHit;
        }

        if BG_SaberInSpecial((*(*otherOwner).client).ps.saberMove) != QFALSE {
            otherUnblockable = QTRUE;
            (*(*otherOwner).client).ps.saberBlocked = 0;
        }

        if sabersClashed != QFALSE
            && dmg > SABER_NONATTACK_DAMAGE
            && selfSaberLevel < FORCE_LEVEL_3
            && PM_SaberInBounce((*(*otherOwner).client).ps.saberMove) == QFALSE
            && PM_SaberInParry((*(*self_).client).ps.saberMove) == QFALSE
            && PM_SaberInBrokenParry((*(*self_).client).ps.saberMove) == QFALSE
            && BG_SaberInSpecial((*(*self_).client).ps.saberMove) == QFALSE
            && PM_SaberInBounce((*(*self_).client).ps.saberMove) == QFALSE
            && PM_SaberInDeflect((*(*self_).client).ps.saberMove) == QFALSE
            && PM_SaberInReflect((*(*self_).client).ps.saberMove) == QFALSE
            && unblockable == QFALSE
        {
            //if (Q_irand(1, 10) <= 6)
            if true
            //for now, just always try a deflect. (deflect func can cause bounces too)
            {
                if WP_GetSaberDeflectionAngle(self_, otherOwner, tr.fraction) == QFALSE {
                    tryDeflectAgain = QTRUE; //Failed the deflect, try it again if we can if the guy we're smashing goes into a parry and we don't break it
                } else {
                    (*(*self_).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                    didOffense = QTRUE;
                }
            } else {
                (*(*self_).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                didOffense = QTRUE;

                // #ifndef FINAL_BUILD — runtime cvar gate honored in retail
                if (*addr_of!(g_saberDebugPrint)).integer != 0 {
                    Com_Printf(&format!(
                        "Client {} clashed into client {}'s saber, did BLOCKED_ATK_BOUNCE\n",
                        (*self_).s.number,
                        (*otherOwner).s.number
                    ));
                }
            }
        }

        if ((selfSaberLevel < FORCE_LEVEL_3
            && ((tryDeflectAgain != QFALSE && Q_irand(1, 10) <= 3)
                || (tryDeflectAgain == QFALSE && Q_irand(1, 10) <= 7)))
            || (Q_irand(1, 10) <= 1 && otherSaberLevel >= FORCE_LEVEL_3))
            && PM_SaberInBounce((*(*self_).client).ps.saberMove) == QFALSE
            && PM_SaberInBrokenParry((*(*otherOwner).client).ps.saberMove) == QFALSE
            && BG_SaberInSpecial((*(*otherOwner).client).ps.saberMove) == QFALSE
            && PM_SaberInBounce((*(*otherOwner).client).ps.saberMove) == QFALSE
            && PM_SaberInDeflect((*(*otherOwner).client).ps.saberMove) == QFALSE
            && PM_SaberInReflect((*(*otherOwner).client).ps.saberMove) == QFALSE
            && (otherSaberLevel > FORCE_LEVEL_2
                || ((*(*otherOwner).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] >= 3
                    && Q_irand(0, otherSaberLevel) != 0))
            && unblockable == QFALSE
            && otherUnblockable == QFALSE
            && dmg > SABER_NONATTACK_DAMAGE
            && didOffense == QFALSE
        //don't allow the person we're attacking to do this if we're making an unblockable attack
        {
            //knockaways can make fast-attacker go into a broken parry anim if the ent is using fast or med. In MP, we also randomly decide this for level 3 attacks.
            //Going to go ahead and let idle damage do simple knockaways. Looks sort of good that way.
            //turn the parry into a knockaway
            if (*(*self_).client).ps.saberEntityNum != 0 {
                //make sure he has his saber still
                saberCheckKnockdown_BrokenParry(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*self_).client).ps.saberEntityNum as usize),
                    self_,
                    otherOwner,
                );
            }

            if PM_SaberInParry((*(*otherOwner).client).ps.saberMove) == QFALSE {
                WP_SaberBlockNonRandom(otherOwner, &tr.endpos, QFALSE);
                (*(*otherOwner).client).ps.saberMove =
                    BG_KnockawayForParry((*(*otherOwner).client).ps.saberBlocked);
                (*(*otherOwner).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
            } else {
                (*(*otherOwner).client).ps.saberMove =
                    G_KnockawayForParry((*(*otherOwner).client).ps.saberMove); //BG_KnockawayForParry( otherOwner->client->ps.saberBlocked );
                (*(*otherOwner).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
            }

            //make them (me) go into a broken parry
            (*(*self_).client).ps.saberMove =
                BG_BrokenParryForAttack((*(*self_).client).ps.saberMove);
            (*(*self_).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;

            // #ifndef FINAL_BUILD — runtime cvar gate honored in retail
            if (*addr_of!(g_saberDebugPrint)).integer != 0 {
                Com_Printf(&format!(
                    "Client {} sent client {} into a reflected attack with a knockaway\n",
                    (*otherOwner).s.number,
                    (*self_).s.number
                ));
            }

            didDefense = QTRUE;
        } else if (selfSaberLevel > FORCE_LEVEL_2 || unblockable != QFALSE) //if we're doing a special attack, we can send them into a broken parry too (MP only)
        && ((*(*otherOwner).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize]
            < selfSaberLevel
            || ((*(*otherOwner).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize]
                == selfSaberLevel
                && (Q_irand(1, 10) as f64 >= otherSaberLevel as f64 * 1.5 || unblockable != QFALSE)))
        && PM_SaberInParry((*(*otherOwner).client).ps.saberMove) != QFALSE
        && PM_SaberInBrokenParry((*(*otherOwner).client).ps.saberMove) == QFALSE
        && PM_SaberInParry((*(*self_).client).ps.saberMove) == QFALSE
        && PM_SaberInBrokenParry((*(*self_).client).ps.saberMove) == QFALSE
        && PM_SaberInBounce((*(*self_).client).ps.saberMove) == QFALSE
        && dmg > SABER_NONATTACK_DAMAGE
        && didOffense == QFALSE
        && otherUnblockable == QFALSE
        {
            //they are in a parry, and we are slamming down on them with a move of equal or greater force than their defense, so send them into a broken parry.. unless they are already in one.
            if (*(*otherOwner).client).ps.saberEntityNum != 0 {
                //make sure he has his saber still
                saberCheckKnockdown_BrokenParry(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*(*otherOwner).client).ps.saberEntityNum as usize),
                    otherOwner,
                    self_,
                );
            }

            // #ifndef FINAL_BUILD — runtime cvar gate honored in retail
            if (*addr_of!(g_saberDebugPrint)).integer != 0 {
                Com_Printf(&format!(
                    "Client {} sent client {} into a broken parry\n",
                    (*self_).s.number,
                    (*otherOwner).s.number
                ));
            }

            (*(*otherOwner).client).ps.saberMove =
                BG_BrokenParryForParry((*(*otherOwner).client).ps.saberMove);
            (*(*otherOwner).client).ps.saberBlocked = BLOCKED_PARRY_BROKEN;

            didDefense = QTRUE;
        } else if (selfSaberLevel > FORCE_LEVEL_2) //if we're doing a special attack, we can send them into a broken parry too (MP only)
        //( otherOwner->client->ps.fd.forcePowerLevel[FP_SABER_DEFENSE] < selfSaberLevel || (otherOwner->client->ps.fd.forcePowerLevel[FP_SABER_DEFENSE] == selfSaberLevel && (Q_irand(1, 10) >= otherSaberLevel*3 || unblockable)) ) &&
        && otherSaberLevel >= FORCE_LEVEL_3
        && PM_SaberInParry((*(*otherOwner).client).ps.saberMove) != QFALSE
        && PM_SaberInBrokenParry((*(*otherOwner).client).ps.saberMove) == QFALSE
        && PM_SaberInParry((*(*self_).client).ps.saberMove) == QFALSE
        && PM_SaberInBrokenParry((*(*self_).client).ps.saberMove) == QFALSE
        && PM_SaberInBounce((*(*self_).client).ps.saberMove) == QFALSE
        && PM_SaberInDeflect((*(*self_).client).ps.saberMove) == QFALSE
        && PM_SaberInReflect((*(*self_).client).ps.saberMove) == QFALSE
        && dmg > SABER_NONATTACK_DAMAGE
        && didOffense == QFALSE
        && unblockable == QFALSE
        {
            //they are in a parry, and we are slamming down on them with a move of equal or greater force than their defense, so send them into a broken parry.. unless they are already in one.
            // #ifndef FINAL_BUILD — runtime cvar gate honored in retail
            if (*addr_of!(g_saberDebugPrint)).integer != 0 {
                Com_Printf(&format!(
                    "Client {} bounced off of client {}'s saber\n",
                    (*self_).s.number,
                    (*otherOwner).s.number
                ));
            }

            if tryDeflectAgain == QFALSE {
                if WP_GetSaberDeflectionAngle(self_, otherOwner, tr.fraction) == QFALSE {
                    tryDeflectAgain = QTRUE;
                }
            }

            didOffense = QTRUE;
        } else if SaberAttacking(otherOwner) != QFALSE
            && dmg > SABER_NONATTACK_DAMAGE
            && BG_SaberInSpecial((*(*otherOwner).client).ps.saberMove) == QFALSE
            && didOffense == QFALSE
            && otherUnblockable == QFALSE
        {
            //they were attacking and our saber hit their saber, make them bounce. But if they're in a special attack, leave them.
            if PM_SaberInBounce((*(*self_).client).ps.saberMove) == QFALSE
                && PM_SaberInBounce((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInDeflect((*(*self_).client).ps.saberMove) == QFALSE
                && PM_SaberInDeflect((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInReflect((*(*self_).client).ps.saberMove) == QFALSE
                && PM_SaberInReflect((*(*otherOwner).client).ps.saberMove) == QFALSE
            {
                let attackAdv: c_int;
                let mut defendStr: c_int = G_PowerLevelForSaberAnim(otherOwner, 0, QTRUE);
                let mut attackBonus: c_int;
                if (*(*otherOwner).client).ps.torsoAnim == BOTH_A1_SPECIAL as c_int
                    || (*(*otherOwner).client).ps.torsoAnim == BOTH_A2_SPECIAL as c_int
                    || (*(*otherOwner).client).ps.torsoAnim == BOTH_A3_SPECIAL as c_int
                {
                    //parry/block/break-parry bonus for single-style kata moves
                    defendStr += 1;
                }
                defendStr += Q_irand(0, (*(*otherOwner).client).saber[0].parryBonus);
                if (*(*otherOwner).client).saber[1].model[0] != 0
                    && (*(*otherOwner).client).ps.saberHolstered == 0
                {
                    defendStr += Q_irand(0, (*(*otherOwner).client).saber[1].parryBonus);
                }

                // #ifndef FINAL_BUILD — runtime cvar gate honored in retail
                if (*addr_of!(g_saberDebugPrint)).integer != 0 {
                    Com_Printf(&format!(
                        "Client {} and client {} bounced off of each other's sabers\n",
                        (*self_).s.number,
                        (*otherOwner).s.number
                    ));
                }

                attackBonus = Q_irand(0, (*(*self_).client).saber[0].breakParryBonus);
                if (*(*self_).client).saber[1].model[0] != 0
                    && (*(*self_).client).ps.saberHolstered == 0
                {
                    attackBonus += Q_irand(0, (*(*self_).client).saber[1].breakParryBonus);
                }
                attackAdv = (attackStr
                    + attackBonus
                    + (*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize])
                    - (defendStr
                        + (*(*otherOwner).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize]);

                if attackAdv > 1 {
                    //I won, he should knockaway
                    (*(*otherOwner).client).ps.saberMove =
                        BG_BrokenParryForAttack((*(*otherOwner).client).ps.saberMove);
                    (*(*otherOwner).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                } else if attackAdv > 0 {
                    //I won, he should bounce, I should continue
                    (*(*otherOwner).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                } else if attackAdv < 1 {
                    //I lost, I get knocked away
                    (*(*self_).client).ps.saberMove =
                        BG_BrokenParryForAttack((*(*self_).client).ps.saberMove);
                    (*(*self_).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;
                } else if attackAdv < 0 {
                    //I lost, I bounce off
                    (*(*self_).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                } else {
                    //even, both bounce off
                    (*(*self_).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                    (*(*otherOwner).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                }

                didOffense = QTRUE;
            }
        }

        if (*addr_of!(d_saberGhoul2Collision)).integer != 0
            && didDefense == QFALSE
            && dmg <= SABER_NONATTACK_DAMAGE
            && otherUnblockable == QFALSE
        //with perpoly, it looks pretty weird to have clash flares coming off the guy's face and whatnot
        {
            if PM_SaberInParry((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInBrokenParry((*(*otherOwner).client).ps.saberMove) == QFALSE
                && BG_SaberInSpecial((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInBounce((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInDeflect((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInReflect((*(*otherOwner).client).ps.saberMove) == QFALSE
            {
                WP_SaberBlockNonRandom(otherOwner, &tr.endpos, QFALSE);
                (*(*otherOwner).client).ps.saberEventFlags |= SEF_PARRIED;
            }
        } else if didDefense == QFALSE && dmg > SABER_NONATTACK_DAMAGE && otherUnblockable == QFALSE
        {
            //if not more than idle damage, don't even bother blocking.
            //block
            if PM_SaberInParry((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInBrokenParry((*(*otherOwner).client).ps.saberMove) == QFALSE
                && BG_SaberInSpecial((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInBounce((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInDeflect((*(*otherOwner).client).ps.saberMove) == QFALSE
                && PM_SaberInReflect((*(*otherOwner).client).ps.saberMove) == QFALSE
            {
                let mut crushTheParry: qboolean = QFALSE;

                if unblockable != QFALSE {
                    //It's unblockable. So send us into a broken parry immediately.
                    crushTheParry = QTRUE;
                }

                if SaberAttacking(otherOwner) == QFALSE {
                    let mut otherIdleStr: c_int = (*(*otherOwner).client).ps.fd.saberAnimLevel;
                    if otherIdleStr == SS_DUAL || otherIdleStr == SS_STAFF {
                        otherIdleStr = SS_MEDIUM;
                    }

                    WP_SaberBlockNonRandom(otherOwner, &tr.endpos, QFALSE);
                    (*(*otherOwner).client).ps.saberEventFlags |= SEF_PARRIED;
                    (*(*self_).client).ps.saberEventFlags |= SEF_BLOCKED;

                    if attackStr
                        + (*(*self_).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize]
                        > otherIdleStr
                            + (*(*otherOwner).client).ps.fd.forcePowerLevel
                                [FP_SABER_DEFENSE as usize]
                    {
                        crushTheParry = QTRUE;
                    } else {
                        tryDeflectAgain = QTRUE;
                    }
                } else if selfSaberLevel > otherSaberLevel
                    || (selfSaberLevel == otherSaberLevel && Q_irand(1, 10) <= 2)
                {
                    //they are attacking, and we managed to make them break
                    //Give them a parry, so we can later break it.
                    WP_SaberBlockNonRandom(otherOwner, &tr.endpos, QFALSE);
                    crushTheParry = QTRUE;

                    if (*(*otherOwner).client).ps.saberEntityNum != 0 {
                        //make sure he has his saber still
                        saberCheckKnockdown_BrokenParry(
                            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                .add((*(*otherOwner).client).ps.saberEntityNum as usize),
                            otherOwner,
                            self_,
                        );
                    }

                    // #ifndef FINAL_BUILD — runtime cvar gate honored in retail
                    if (*addr_of!(g_saberDebugPrint)).integer != 0 {
                        Com_Printf(&format!(
                        "Client {} forced client {} into a broken parry with a stronger attack\n",
                        (*self_).s.number,
                        (*otherOwner).s.number
                    ));
                    }
                } else {
                    //They are attacking, so are we, and obviously they have an attack level higher than or equal to ours
                    if selfSaberLevel == otherSaberLevel {
                        //equal level, try to bounce off each other's sabers
                        if didOffense == QFALSE
                            && PM_SaberInParry((*(*self_).client).ps.saberMove) == QFALSE
                            && PM_SaberInBrokenParry((*(*self_).client).ps.saberMove) == QFALSE
                            && BG_SaberInSpecial((*(*self_).client).ps.saberMove) == QFALSE
                            && PM_SaberInBounce((*(*self_).client).ps.saberMove) == QFALSE
                            && PM_SaberInDeflect((*(*self_).client).ps.saberMove) == QFALSE
                            && PM_SaberInReflect((*(*self_).client).ps.saberMove) == QFALSE
                            && unblockable == QFALSE
                        {
                            (*(*self_).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                            didOffense = QTRUE;
                        }
                        if didDefense == QFALSE
                            && PM_SaberInParry((*(*otherOwner).client).ps.saberMove) == QFALSE
                            && PM_SaberInBrokenParry((*(*otherOwner).client).ps.saberMove) == QFALSE
                            && BG_SaberInSpecial((*(*otherOwner).client).ps.saberMove) == QFALSE
                            && PM_SaberInBounce((*(*otherOwner).client).ps.saberMove) == QFALSE
                            && PM_SaberInDeflect((*(*otherOwner).client).ps.saberMove) == QFALSE
                            && PM_SaberInReflect((*(*otherOwner).client).ps.saberMove) == QFALSE
                            && unblockable == QFALSE
                        {
                            (*(*otherOwner).client).ps.saberBlocked = BLOCKED_ATK_BOUNCE;
                        }
                        // #ifndef FINAL_BUILD — runtime cvar gate honored in retail
                        if (*addr_of!(g_saberDebugPrint)).integer != 0 {
                            Com_Printf(&format!(
                                "Equal attack level bounce/deflection for clients {} and {}\n",
                                (*self_).s.number,
                                (*otherOwner).s.number
                            ));
                        }

                        (*(*self_).client).ps.saberEventFlags |= SEF_DEFLECTED;
                        (*(*otherOwner).client).ps.saberEventFlags |= SEF_DEFLECTED;
                    } else if ((*addr_of!(level)).time
                        - (*(*otherOwner).client).lastSaberStorageTime)
                        < 500
                        && unblockable == QFALSE
                    {
                        //make sure the stored saber data is updated
                        //They are higher, this means they can actually smash us into a broken parry
                        //Using reflected anims instead now
                        (*(*self_).client).ps.saberMove =
                            BG_BrokenParryForAttack((*(*self_).client).ps.saberMove);
                        (*(*self_).client).ps.saberBlocked = BLOCKED_PARRY_BROKEN;

                        if (*(*self_).client).ps.saberEntityNum != 0 {
                            //make sure he has his saber still
                            saberCheckKnockdown_BrokenParry(
                                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                    .add((*(*self_).client).ps.saberEntityNum as usize),
                                self_,
                                otherOwner,
                            );
                        }

                        // #ifndef FINAL_BUILD — runtime cvar gate honored in retail
                        if (*addr_of!(g_saberDebugPrint)).integer != 0 {
                            Com_Printf(&format!(
                            "Client {} hit client {}'s stronger attack, was forced into a broken parry\n",
                            (*self_).s.number,
                            (*otherOwner).s.number
                        ));
                        }

                        (*(*otherOwner).client).ps.saberEventFlags &= !SEF_BLOCKED;

                        didOffense = QTRUE;
                    }
                }

                if crushTheParry != QFALSE
                    && PM_SaberInParry(G_GetParryForBlock((*(*otherOwner).client).ps.saberBlocked))
                        != QFALSE
                {
                    //This means that the attack actually hit our saber, and we went to block it.
                    //But, one of the above cases says we actually can't. So we will be smashed into a broken parry instead.
                    (*(*otherOwner).client).ps.saberMove = BG_BrokenParryForParry(
                        G_GetParryForBlock((*(*otherOwner).client).ps.saberBlocked),
                    );
                    (*(*otherOwner).client).ps.saberBlocked = BLOCKED_PARRY_BROKEN;

                    (*(*otherOwner).client).ps.saberEventFlags &= !SEF_PARRIED;
                    (*(*self_).client).ps.saberEventFlags &= !SEF_BLOCKED;

                    // #ifndef FINAL_BUILD — runtime cvar gate honored in retail
                    if (*addr_of!(g_saberDebugPrint)).integer != 0 {
                        Com_Printf(&format!(
                        "Client {} broke through {}'s parry with a special or stronger attack\n",
                        (*self_).s.number,
                        (*otherOwner).s.number
                    ));
                    }
                } else if PM_SaberInParry(G_GetParryForBlock(
                    (*(*otherOwner).client).ps.saberBlocked,
                )) != QFALSE
                    && didOffense == QFALSE
                    && tryDeflectAgain != QFALSE
                {
                    //We want to try deflecting again because the other is in the parry and we haven't made any new moves
                    let preMove: c_int = (*(*otherOwner).client).ps.saberMove;

                    (*(*otherOwner).client).ps.saberMove =
                        G_GetParryForBlock((*(*otherOwner).client).ps.saberBlocked);
                    WP_GetSaberDeflectionAngle(self_, otherOwner, tr.fraction);
                    (*(*otherOwner).client).ps.saberMove = preMove;
                }
            }
        }

        (*(*self_).client).ps.saberAttackWound =
            (*addr_of!(level)).time + (*addr_of!(g_saberDmgDelay_Wound)).integer;
    } // 'blockStuff

    didHit
}

const MAX_SABER_SWING_INC: f32 = 0.33; // w_saber.c:4867

/// `void G_SPSaberDamageTraceLerped( gentity_t *self, int saberNum, int bladeNum, vec3_t baseNew,
/// vec3_t endNew, int clipmask )` (w_saber.c:4868) — the SP-style swept saber damage trace. Walks
/// the blade from its previous frame's position (`trail[...].base`/`.tip`) to its new endpoint,
/// lerping the blade angle and base in chunks so a fast swing's arc is sampled instead of being
/// flattened to a single straight trace. Each interval calls [`CheckSaberDamage`] (which sets the
/// file-statics `saberHitWall`/`saberHitSaber`/`saberHitFraction`); when a saber clash is detected
/// (`saberHitFraction < 1.0`) the remaining traces are shortened to that hit point.
///
/// Deviations: `ClientManager::ActiveClientNum()` (used to index the saber trail) renders as `0`
/// per the MP-client-0 convention, matching [`CheckSaberDamage`]. `WP_SaberDamageForTrace` is only
/// referenced inside `/* ... */` comment blocks in the C — those comments are carried verbatim and
/// nothing is called from them. The C `#define MAX_SABER_SWING_INC 0.33f` immediately preceding
/// the function is ported as a module const. All arithmetic stays single-precision to match the C.
///
/// # Safety
/// Dereferences `self` (and its `client`/saber trail) and calls [`CheckSaberDamage`], which walks
/// `g_entities`, reads cvars/`level`, performs engine traces and mutates entity/player saber-combat
/// state. Caller must pass a valid client entity; trace/entity-state dependent, so no oracle test.
// `allow(dead_code)`: callers are not yet ported.
#[allow(dead_code)]
pub unsafe fn G_SPSaberDamageTraceLerped(
    self_: *mut gentity_t,
    saberNum: c_int,
    bladeNum: c_int,
    baseNew: &mut vec3_t,
    endNew: &mut vec3_t,
    clipmask: c_int,
) {
    let mut baseOld: vec3_t = [0.0; 3];
    let mut endOld: vec3_t = [0.0; 3];
    let mut mp1: vec3_t = [0.0; 3];
    let mut mp2: vec3_t = [0.0; 3];
    let mut md1: vec3_t = [0.0; 3];
    let mut md2: vec3_t = [0.0; 3];

    if ((*addr_of!(level)).time
        - (*(*self_).client).saber[saberNum as usize].blade[bladeNum as usize]
            .trail
            .lastTime)
        > 100
    {
        //no valid last pos, use current
        VectorCopy(baseNew, &mut baseOld);
        VectorCopy(endNew, &mut endOld);
    } else {
        //trace from last pos
        VectorCopy(
            &(*(*self_).client).saber[saberNum as usize].blade[bladeNum as usize]
                .trail
                .base,
            &mut baseOld,
        );
        VectorCopy(
            &(*(*self_).client).saber[saberNum as usize].blade[bladeNum as usize]
                .trail
                .tip,
            &mut endOld,
        );
    }

    VectorCopy(&baseOld, &mut mp1);
    VectorCopy(baseNew, &mut mp2);
    VectorSubtract(&endOld, &baseOld, &mut md1);
    VectorNormalize(&mut md1);
    VectorSubtract(endNew, baseNew, &mut md2);
    VectorNormalize(&mut md2);

    saberHitWall = QFALSE;
    saberHitSaber = QFALSE;
    saberHitFraction = 1.0f32;
    if VectorCompare2(&baseOld, baseNew) != 0 && VectorCompare2(&endOld, endNew) != 0 {
        //no diff
        CheckSaberDamage(
            self_, saberNum, bladeNum, baseNew, endNew, QFALSE, clipmask, QFALSE,
        );
    } else {
        //saber moved, lerp
        let mut step: f32; //aveLength,
        let stepsize: f32 = 8.0;
        let mut ma1: vec3_t = [0.0; 3];
        let mut ma2: vec3_t = [0.0; 3];
        let mut md2ang: vec3_t = [0.0; 3];
        let mut curBase1: vec3_t = [0.0; 3];
        let mut curBase2: vec3_t = [0.0; 3];
        let mut xx: c_int;
        let mut curMD1: vec3_t = [0.0; 3]; //, mdDiff, dirDiff;
        let mut curMD2: vec3_t = [0.0; 3];
        let dirInc: f32;
        let mut curDirFrac: f32;
        let mut baseDiff: vec3_t = [0.0; 3];
        let mut bladePointOld: vec3_t = [0.0; 3];
        let mut bladePointNew: vec3_t = [0.0; 3];
        let mut extrapolate: qboolean = QTRUE;

        //do the trace at the base first
        VectorCopy(&baseOld, &mut bladePointOld);
        VectorCopy(baseNew, &mut bladePointNew);
        CheckSaberDamage(
            self_,
            saberNum,
            bladeNum,
            &mut bladePointOld,
            &mut bladePointNew,
            QFALSE,
            clipmask,
            QTRUE,
        );

        //if hit a saber, shorten rest of traces to match
        if saberHitFraction < 1.0f32 {
            //adjust muzzleDir...
            let mut ma1: vec3_t = [0.0; 3];
            let mut ma2: vec3_t = [0.0; 3];
            vectoangles(&md1, &mut ma1);
            vectoangles(&md2, &mut ma2);
            xx = 0;
            while xx < 3 {
                md2ang[xx as usize] =
                    LerpAngle(ma1[xx as usize], ma2[xx as usize], saberHitFraction);
                xx += 1;
            }
            AngleVectors(&md2ang, Some(&mut md2), None, None);
            //shorten the base pos
            VectorSubtract(&mp2, &mp1, &mut baseDiff);
            VectorMA(&mp1, saberHitFraction, &baseDiff, baseNew);
            VectorMA(
                baseNew,
                (*(*self_).client).saber[saberNum as usize].blade[bladeNum as usize].lengthMax,
                &md2,
                endNew,
            );
        }

        //If the angle diff in the blade is high, need to do it in chunks of 33 to avoid flattening of the arc
        if BG_SaberInAttack((*(*self_).client).ps.saberMove) != QFALSE
            || BG_SaberInSpecialAttack((*(*self_).client).ps.torsoAnim) != QFALSE
            || BG_SpinningSaberAnim((*(*self_).client).ps.torsoAnim) != QFALSE
            || BG_InSpecialJump((*(*self_).client).ps.torsoAnim) != QFALSE
        //|| (g_timescale->value<1.0f&&BG_SaberInTransitionAny( ent->client->ps.saberMove )) )
        {
            curDirFrac = DotProduct(&md1, &md2);
        } else {
            curDirFrac = 1.0f32;
        }
        //NOTE: if saber spun at least 180 degrees since last damage trace, this is not reliable...!
        if curDirFrac.abs() < 1.0f32 - MAX_SABER_SWING_INC {
            //the saber blade spun more than 33 degrees since the last damage trace
            curDirFrac = 1.0f32 / ((1.0f32 - curDirFrac) / MAX_SABER_SWING_INC);
            dirInc = curDirFrac;
        } else {
            curDirFrac = 1.0f32;
            dirInc = 0.0f32;
        }
        //qboolean hit_saber = qfalse;

        vectoangles(&md1, &mut ma1);
        vectoangles(&md2, &mut ma2);

        //VectorSubtract( md2, md1, mdDiff );
        VectorCopy(&md1, &mut curMD2);
        VectorCopy(&baseOld, &mut curBase2);

        loop {
            VectorCopy(&curMD2, &mut curMD1);
            VectorCopy(&curBase2, &mut curBase1);
            if curDirFrac >= 1.0f32 {
                VectorCopy(&md2, &mut curMD2);
                VectorCopy(baseNew, &mut curBase2);
            } else {
                xx = 0;
                while xx < 3 {
                    md2ang[xx as usize] = LerpAngle(ma1[xx as usize], ma2[xx as usize], curDirFrac);
                    xx += 1;
                }
                AngleVectors(&md2ang, Some(&mut curMD2), None, None);
                //VectorMA( md1, curDirFrac, mdDiff, curMD2 );
                VectorSubtract(baseNew, &baseOld, &mut baseDiff);
                VectorMA(&baseOld, curDirFrac, &baseDiff, &mut curBase2);
            }
            // Move up the blade in intervals of stepsize
            step = stepsize;
            while step
                <= (*(*self_).client).saber[saberNum as usize].blade[bladeNum as usize].lengthMax
            /*&& step < self->client->saber[saberNum].blade[bladeNum].lengthOld*/
            {
                VectorMA(&curBase1, step, &curMD1, &mut bladePointOld);
                VectorMA(&curBase2, step, &curMD2, &mut bladePointNew);

                if step + stepsize
                    >= (*(*self_).client).saber[saberNum as usize].blade[bladeNum as usize]
                        .lengthMax
                {
                    extrapolate = QFALSE;
                }
                //do the damage trace
                CheckSaberDamage(
                    self_,
                    saberNum,
                    bladeNum,
                    &mut bladePointOld,
                    &mut bladePointNew,
                    QFALSE,
                    clipmask,
                    extrapolate,
                );
                /*
                if ( WP_SaberDamageForTrace( ent->s.number, bladePointOld, bladePointNew, baseDamage, curMD2,
                    qfalse, entPowerLevel, ent->client->ps.saber[saberNum].type, qtrue,
                    saberNum, bladeNum ) )
                {
                    hit_wall = qtrue;
                }
                */

                //if hit a saber, shorten rest of traces to match
                if saberHitFraction < 1.0f32 {
                    let mut curMA1: vec3_t = [0.0; 3];
                    let mut curMA2: vec3_t = [0.0; 3];
                    //adjust muzzle endpoint
                    VectorSubtract(&mp2, &mp1, &mut baseDiff);
                    VectorMA(&mp1, saberHitFraction, &baseDiff, baseNew);
                    VectorMA(
                        baseNew,
                        (*(*self_).client).saber[saberNum as usize].blade[bladeNum as usize]
                            .lengthMax,
                        &curMD2,
                        endNew,
                    );
                    //adjust muzzleDir...
                    vectoangles(&curMD1, &mut curMA1);
                    vectoangles(&curMD2, &mut curMA2);
                    xx = 0;
                    while xx < 3 {
                        md2ang[xx as usize] =
                            LerpAngle(curMA1[xx as usize], curMA2[xx as usize], saberHitFraction);
                        xx += 1;
                    }
                    AngleVectors(&md2ang, Some(&mut curMD2), None, None);
                    saberHitSaber = QTRUE;
                }
                if saberHitWall != QFALSE {
                    break;
                }
                step += stepsize;
            }
            if saberHitWall != QFALSE || saberHitSaber != QFALSE {
                break;
            }
            if curDirFrac >= 1.0f32 {
                break;
            } else {
                curDirFrac += dirInc;
                if curDirFrac >= 1.0f32 {
                    curDirFrac = 1.0f32;
                }
            }
        }

        //do the trace at the end last
        //Special check- adjust for length of blade not being a multiple of 12
        /*
        aveLength = (ent->client->ps.saber[saberNum].blade[bladeNum].lengthOld + ent->client->ps.saber[saberNum].blade[bladeNum].length)/2;
        if ( step > aveLength )
        {//less dmg if the last interval was not stepsize
            tipDmgMod = (stepsize-(step-aveLength))/stepsize;
        }
        //NOTE: since this is the tip, we do not extrapolate the extra 16
        if ( WP_SaberDamageForTrace( ent->s.number, endOld, endNew, tipDmgMod*baseDamage, md2,
            qfalse, entPowerLevel, ent->client->ps.saber[saberNum].type, qfalse,
            saberNum, bladeNum ) )
        {
            hit_wall = qtrue;
        }
        */
    }
}

/// `#define LOOK_DEFAULT_SPEED 0.15f` (w_saber.c:549) — head-turn lerp speed used when
/// looking at a non-enemy ent (kept as a w_saber.c-local module const). The C also defines
/// `LOOK_TALKING_SPEED 0.15f` on the next line, but it is never referenced — omitted.
const LOOK_DEFAULT_SPEED: f32 = 0.15;

/// `static GAME_INLINE qboolean G_CheckLookTarget( gentity_t *ent, vec3_t lookAngles, float *lookingSpeed )`
/// (w_saber.c:552).
///
/// Compute the head-look angles for `ent` toward its current `renderInfo.lookTarget`
/// (an entity in `LM_ENT` mode, or a `level.interestPoints[]` slot in `LM_INTEREST`
/// mode), returning `qtrue` and filling `lookAngles` (relative to the eye angles) when a
/// valid target exists, `qfalse` otherwise. An NPC bolted to a vehicle instead looks
/// around randomly via the `"lookAround"` timer. `lookingSpeed` is lowered to
/// `LOOK_DEFAULT_SPEED` when looking at a non-enemy.
///
/// No oracle — reads/writes heavy gentity/gclient/renderInfo state, the global `level`,
/// `g_entities`, the timer pool and the RNG.
///
/// # Safety
/// `ent` and its `client`/`NPC` (when accessed) must be valid; `renderInfo.lookTarget`
/// must index `g_entities`/`level.interestPoints` as the modes require.
pub unsafe fn G_CheckLookTarget(
    ent: *mut gentity_t,
    lookAngles: &mut vec3_t,
    lookingSpeed: &mut f32,
) -> qboolean {
    //FIXME: also clamp the lookAngles based on the clamp + the existing difference between
    //		headAngles and torsoAngles?  But often the tag_torso is straight but the torso itself
    //		is deformed to not face straight... sigh...

    if (*ent).s.eType == ET_NPC
        && (*ent).s.m_iVehicleNum != 0
        && (*ent).s.NPC_class != CLASS_VEHICLE
    {
        //an NPC bolted to a vehicle should just look around randomly
        if TIMER_Done(ent, c"lookAround".as_ptr()) != QFALSE {
            (*(*ent).NPC).shootAngles[YAW as usize] = flrand(0.0, 360.0);
            TIMER_Set(ent, c"lookAround".as_ptr(), Q_irand(500, 3000));
        }
        VectorSet(
            lookAngles,
            0.0,
            (*(*ent).NPC).shootAngles[YAW as usize],
            0.0,
        );
        return QTRUE;
    }
    //Now calc head angle to lookTarget, if any
    if (*(*ent).client).renderInfo.lookTarget >= 0
        && (*(*ent).client).renderInfo.lookTarget < ENTITYNUM_WORLD
    {
        let mut lookDir: vec3_t = [0.0; 3];
        let mut lookOrg: vec3_t = [0.0; 3];
        let mut eyeOrg: vec3_t = [0.0; 3];
        let mut i: c_int;

        if (*(*ent).client).renderInfo.lookMode == LM_ENT {
            let lookCent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities)
                .cast::<gentity_t>())
            .add((*(*ent).client).renderInfo.lookTarget as usize);
            if !lookCent.is_null() {
                if lookCent != (*ent).enemy {
                    //We turn heads faster than headbob speed, but not as fast as if watching an enemy
                    *lookingSpeed = LOOK_DEFAULT_SPEED;
                }

                //FIXME: Ignore small deltas from current angles so we don't bob our head in synch with theirs?

                /*
                if ( ent->client->renderInfo.lookTarget == 0 && !cg.renderingThirdPerson )//!cg_thirdPerson.integer )
                {//Special case- use cg.refdef.vieworg if looking at player and not in third person view
                    VectorCopy( cg.refdef.vieworg, lookOrg );
                }
                */
 //No no no!
                if !(*lookCent).client.is_null() {
                    VectorCopy(&(*(*lookCent).client).renderInfo.eyePoint, &mut lookOrg);
                } else if (*lookCent).inuse != QFALSE
                    && VectorCompare(&(*lookCent).r.currentOrigin, &vec3_origin) == QFALSE as c_int
                {
                    VectorCopy(&(*lookCent).r.currentOrigin, &mut lookOrg);
                } else {
                    //at origin of world
                    return QFALSE;
                }
                //Look in dir of lookTarget
            }
        } else if (*(*ent).client).renderInfo.lookMode == LM_INTEREST
            && (*(*ent).client).renderInfo.lookTarget > -1
            && (*(*ent).client).renderInfo.lookTarget < MAX_INTEREST_POINTS as c_int
        {
            VectorCopy(
                &(*addr_of!(level)).interestPoints[(*(*ent).client).renderInfo.lookTarget as usize]
                    .origin,
                &mut lookOrg,
            );
        } else {
            return QFALSE;
        }

        VectorCopy(&(*(*ent).client).renderInfo.eyePoint, &mut eyeOrg);

        VectorSubtract(&lookOrg, &eyeOrg, &mut lookDir);

        vectoangles(&lookDir, lookAngles);

        i = 0;
        while i < 3 {
            lookAngles[i as usize] = AngleNormalize180(lookAngles[i as usize]);
            (*(*ent).client).renderInfo.eyeAngles[i as usize] =
                AngleNormalize180((*(*ent).client).renderInfo.eyeAngles[i as usize]);
            i += 1;
        }
        let eyeAngles = (*(*ent).client).renderInfo.eyeAngles;
        AnglesSubtract(&lookAngles.clone(), &eyeAngles, lookAngles);
        return QTRUE;
    }

    QFALSE
}

/// `static GAME_INLINE void G_G2NPCAngles(gentity_t *ent, vec3_t legs[3], vec3_t angles)`
/// (w_saber.c:642).
///
/// rww's MP "port" of the SP client-side droid/NPC head-look behavior: for the droid
/// classes (`CLASS_PROBE`/`R2D2`/`R5D2`/`ATST`) compute the body `angles`, run the
/// look-target lerp through [`G_CheckLookTarget`], smooth the head angles via
/// `renderInfo.lastHeadAngles`/`lookingDebounceTime`, and push the result to the cranium
/// bone with [`NPC_SetBoneAngles`](super::npc_utils::NPC_SetBoneAngles). Non-droid classes
/// are a no-op (the SP body is commented out).
///
/// No oracle — heavy gentity/gclient/renderInfo/ghoul2 state plus traps.
///
/// # Safety
/// `ent->client` must be valid for the droid classes; `legs`/`angles` must be writable.
pub unsafe fn G_G2NPCAngles(ent: *mut gentity_t, legs: &mut [vec3_t; 3], angles: &mut vec3_t) {
    let craniumBone = "cranium";
    let thoracicBone = "thoracic"; //only used by atst so doesn't need a case
                                   // C: `qboolean looking = qfalse;` — the init is dead (always overwritten by the
                                   // G_CheckLookTarget result before its only read), so deferred here to stay warning-clean.
    let looking: qboolean;
    let mut viewAngles: vec3_t = [0.0; 3];
    let mut lookAngles: vec3_t = [0.0; 3];

    if !(*ent).client.is_null() {
        if (*(*ent).client).NPC_class == CLASS_PROBE
            || (*(*ent).client).NPC_class == CLASS_R2D2
            || (*(*ent).client).NPC_class == CLASS_R5D2
            || (*(*ent).client).NPC_class == CLASS_ATST
        {
            let trailingLegsAngles: vec3_t = [0.0; 3];

            if (*ent).s.eType == ET_NPC
                && (*ent).s.m_iVehicleNum != 0
                && (*ent).s.NPC_class != CLASS_VEHICLE
            {
                //an NPC bolted to a vehicle should use the full angles
                VectorCopy(&(*ent).r.currentAngles, angles);
            } else {
                VectorCopy(&(*(*ent).client).ps.viewangles, angles);
                angles[PITCH as usize] = 0.0;
            }

            VectorCopy(&(*(*ent).client).ps.viewangles, &mut viewAngles);
            //			viewAngles[YAW] = viewAngles[ROLL] = 0;
            viewAngles[PITCH as usize] *= 0.5;
            VectorCopy(&viewAngles, &mut lookAngles);

            lookAngles[1] = 0.0;

            if (*(*ent).client).NPC_class == CLASS_ATST {
                //body pitch
                NPC_SetBoneAngles(ent, thoracicBone, &lookAngles);
                //BG_G2SetBoneAngles( cent, ent, ent->thoracicBone, lookAngles, BONE_ANGLES_POSTMULT,POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, cgs.model_draw);
            }

            VectorCopy(&viewAngles, &mut lookAngles);

            if !(*ent).client.is_null() && (*(*ent).client).NPC_class == CLASS_ATST {
                AnglesToAxis(&trailingLegsAngles, legs);
            } else {
                //FIXME: this needs to properly set the legs.yawing field so we don't erroneously play the turning anim, but we do play it when turning in place
            }

            {
                //look at lookTarget!
                //FIXME: snaps to side when lets go of lookTarget... ?
                let mut lookingSpeed: f32 = 0.3;
                looking = G_CheckLookTarget(ent, &mut lookAngles, &mut lookingSpeed);
                lookAngles[PITCH as usize] = 0.0;
                lookAngles[ROLL as usize] = 0.0; //droids can't pitch or roll their heads
                if looking != QFALSE {
                    //want to keep doing this lerp behavior for a full second after stopped looking (so don't snap)
                    (*(*ent).client).renderInfo.lookingDebounceTime =
                        (*addr_of!(level)).time + 1000;
                }
            }
            if (*(*ent).client).renderInfo.lookingDebounceTime > (*addr_of!(level)).time {
                //adjust for current body orientation
                let mut oldLookAngles: vec3_t = [0.0; 3];

                lookAngles[YAW as usize] -= 0.0; //ent->client->ps.viewangles[YAW];//cent->pe.torso.yawAngle;
                                                 //lookAngles[YAW] -= cent->pe.legs.yawAngle;

                //normalize
                lookAngles[YAW as usize] = AngleNormalize180(lookAngles[YAW as usize]);

                //slowly lerp to this new value
                //Remember last headAngles
                VectorCopy(
                    &(*(*ent).client).renderInfo.lastHeadAngles,
                    &mut oldLookAngles,
                );
                if VectorCompare(&oldLookAngles, &lookAngles) == QFALSE as c_int {
                    //FIXME: This clamp goes off viewAngles,
                    //but really should go off the tag_torso's axis[0] angles, no?
                    lookAngles[YAW as usize] = oldLookAngles[YAW as usize]
                        + (lookAngles[YAW as usize] - oldLookAngles[YAW as usize]) * 0.4;
                }
                //Remember current lookAngles next time
                VectorCopy(&lookAngles, &mut (*(*ent).client).renderInfo.lastHeadAngles);
            } else {
                //Remember current lookAngles next time
                VectorCopy(&lookAngles, &mut (*(*ent).client).renderInfo.lastHeadAngles);
            }
            if (*(*ent).client).NPC_class == CLASS_ATST {
                VectorCopy(&(*(*ent).client).ps.viewangles, &mut lookAngles);
                lookAngles[0] = 0.0;
                lookAngles[2] = 0.0;
                lookAngles[YAW as usize] -= trailingLegsAngles[YAW as usize];
            } else {
                lookAngles[PITCH as usize] = 0.0;
                lookAngles[ROLL as usize] = 0.0;
                lookAngles[YAW as usize] -= (*(*ent).client).ps.viewangles[YAW as usize];
            }

            NPC_SetBoneAngles(ent, craniumBone, &lookAngles);
        } else
        //if ( (ent->client->NPC_class == CLASS_GONK ) || (ent->client->NPC_class == CLASS_INTERROGATOR) || (ent->client->NPC_class == CLASS_SENTRY) )
        {
            //	VectorCopy( ent->client->ps.viewangles, angles );
            //	AnglesToAxis( angles, legs );
            //return;
        }
    }
}

/// `static GAME_INLINE void G_G2PlayerAngles(gentity_t *ent, vec3_t legs[3], vec3_t legsAngles)`
/// (w_saber.c:754).
///
/// rww's server-side mirror of cgame's `CG_G2PlayerAngles`: drives the server ghoul2
/// instance's leg/torso angles so the saber-contact traces line up with what clients see.
/// For an ET_NPC with no real client in the same PVS it early-outs (no one can see him).
/// For humanoids (`localAnimIndex <= 1`) it runs the full
/// [`BG_G2PlayerAngles`](super::bg_pmove::BG_G2PlayerAngles) torso/leg solve, then the
/// held-by-client and self IK-arm cases via [`BG_IK_MoveArm`](super::bg_pmove::BG_IK_MoveArm).
/// A `VH_WALKER` vehicle uses the AT-ST posture
/// ([`BG_G2ATSTAngles`](super::bg_pmove::BG_G2ATSTAngles)); other NPCs fall to
/// [`G_G2NPCAngles`] (with a `VH_FIGHTER` axial-angle special case).
///
/// No oracle — heavy gentity/gclient/ps/renderInfo/ghoul2 state plus traps.
///
/// # Safety
/// `ent`, `ent->client` and (on the IK paths) other clients' ghoul2 instances must be
/// valid; `legs`/`legsAngles` must be writable.
pub unsafe fn G_G2PlayerAngles(
    ent: *mut gentity_t,
    legs: &mut [vec3_t; 3],
    legsAngles: &mut vec3_t,
) {
    let mut tPitching: qboolean = QFALSE;
    let mut tYawing: qboolean = QFALSE;
    let mut lYawing: qboolean = QFALSE;
    let mut tYawAngle: f32 = (*(*ent).client).ps.viewangles[YAW as usize];
    let mut tPitchAngle: f32 = 0.0;
    let mut lYawAngle: f32 = (*(*ent).client).ps.viewangles[YAW as usize];

    let ciLegs: c_int = (*(*ent).client).ps.legsAnim;
    let ciTorso: c_int = (*(*ent).client).ps.torsoAnim;

    let mut turAngles: vec3_t = [0.0; 3];
    let mut lerpOrg: vec3_t = [0.0; 3];
    let mut lerpAng: vec3_t = [0.0; 3];

    if (*ent).s.eType == ET_NPC && !(*ent).client.is_null() {
        //sort of hacky, but it saves a pretty big load off the server
        let mut i: c_int = 0;

        //If no real clients are in the same PVS then don't do any of this stuff, no one can see him anyway!
        while i < MAX_CLIENTS as c_int {
            let clEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

            if !clEnt.is_null()
                && (*clEnt).inuse != QFALSE
                && !(*clEnt).client.is_null()
                && trap::InPVS(&(*(*clEnt).client).ps.origin, &(*(*ent).client).ps.origin) != QFALSE
            {
                //this client can see him
                break;
            }

            i += 1;
        }

        if i == MAX_CLIENTS as c_int {
            //no one can see him, just return
            return;
        }
    }

    VectorCopy(&(*(*ent).client).ps.origin, &mut lerpOrg);
    VectorCopy(&(*(*ent).client).ps.viewangles, &mut lerpAng);

    if (*ent).localAnimIndex <= 1 {
        //don't do these things on non-humanoids
        let mut lookAngles: vec3_t = [0.0; 3];
        let mut emplaced: *mut entityState_t = null_mut();

        if (*(*ent).client).ps.hasLookTarget != QFALSE {
            VectorSubtract(
                &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*ent).client).ps.lookTarget as usize))
                .r
                .currentOrigin,
                &(*(*ent).client).ps.origin,
                &mut lookAngles,
            );
            vectoangles(&lookAngles.clone(), &mut lookAngles);
            (*(*ent).client).lookTime = (*addr_of!(level)).time + 1000;
        } else {
            VectorCopy(&(*(*ent).client).ps.origin, &mut lookAngles);
        }
        lookAngles[PITCH as usize] = 0.0;

        if (*(*ent).client).ps.emplacedIndex != 0 {
            emplaced = &mut (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*(*ent).client).ps.emplacedIndex as usize))
            .s;
        }

        BG_G2PlayerAngles(
            (*ent).ghoul2,
            (*(*ent).client).renderInfo.motionBolt,
            &mut (*ent).s,
            (*addr_of!(level)).time,
            &mut lerpOrg,
            &mut lerpAng,
            legs,
            legsAngles,
            &mut tYawing,
            &mut tPitching,
            &mut lYawing,
            &mut tYawAngle,
            &mut tPitchAngle,
            &mut lYawAngle,
            FRAMETIME,
            &mut turAngles,
            &mut (*ent).modelScale,
            ciLegs,
            ciTorso,
            &mut (*(*ent).client).corrTime,
            &mut lookAngles,
            &mut (*(*ent).client).lastHeadAngles,
            (*(*ent).client).lookTime,
            emplaced,
            null_mut(),
        );

        if (*(*ent).client).ps.heldByClient != 0
            && (*(*ent).client).ps.heldByClient <= MAX_CLIENTS as c_int
        {
            //then put our arm in this client's hand
            //is index+1 because index 0 is valid.
            let heldByIndex = (*(*ent).client).ps.heldByClient - 1;
            let other =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(heldByIndex as usize);
            // C: `int lHandBolt = 0;` — the `= 0` init is dead (the `else` arm returns, so
            // the only path that reaches the read assigns it first), so deferred here.
            let lHandBolt: c_int;

            if !other.is_null()
                && (*other).inuse != QFALSE
                && !(*other).client.is_null()
                && !(*other).ghoul2.is_null()
            {
                lHandBolt = trap::G2API_AddBolt((*other).ghoul2, 0, "*l_hand");
            } else {
                //they left the game, perhaps?
                (*(*ent).client).ps.heldByClient = 0;
                return;
            }

            if lHandBolt != 0 {
                let mut boltMatrix: mdxaBone_t = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                let mut boltOrg: vec3_t = [0.0; 3];
                let mut tAngles: vec3_t = [0.0; 3];

                VectorCopy(&(*(*other).client).ps.viewangles, &mut tAngles);
                tAngles[PITCH as usize] = 0.0;
                tAngles[ROLL as usize] = 0.0;

                trap::G2API_GetBoltMatrix(
                    (*other).ghoul2,
                    0,
                    lHandBolt,
                    &mut boltMatrix,
                    &tAngles,
                    &(*(*other).client).ps.origin,
                    (*addr_of!(level)).time,
                    null_mut(),
                    &(*other).modelScale,
                );
                boltOrg[0] = boltMatrix.matrix[0][3];
                boltOrg[1] = boltMatrix.matrix[1][3];
                boltOrg[2] = boltMatrix.matrix[2][3];

                BG_IK_MoveArm(
                    (*ent).ghoul2,
                    lHandBolt,
                    (*addr_of!(level)).time,
                    &mut (*ent).s,
                    (*(*ent).client).ps.torsoAnim, /*BOTH_DEAD1*/
                    &mut boltOrg,
                    &mut (*(*ent).client).ikStatus,
                    &mut (*(*ent).client).ps.origin,
                    &mut (*(*ent).client).ps.viewangles,
                    &mut (*ent).modelScale,
                    500,
                    QFALSE,
                );
            }
        } else if (*(*ent).client).ikStatus != QFALSE {
            //make sure we aren't IKing if we don't have anyone to hold onto us.
            let mut lHandBolt: c_int = 0;

            if !ent.is_null()
                && (*ent).inuse != QFALSE
                && !(*ent).client.is_null()
                && !(*ent).ghoul2.is_null()
            {
                lHandBolt = trap::G2API_AddBolt((*ent).ghoul2, 0, "*l_hand");
            } else {
                //This shouldn't happen, but just in case it does, we'll have a failsafe.
                (*(*ent).client).ikStatus = QFALSE;
            }

            if lHandBolt != 0 {
                let mut vec3_origin_local: vec3_t = vec3_origin;
                BG_IK_MoveArm(
                    (*ent).ghoul2,
                    lHandBolt,
                    (*addr_of!(level)).time,
                    &mut (*ent).s,
                    (*(*ent).client).ps.torsoAnim, /*BOTH_DEAD1*/
                    &mut vec3_origin_local,
                    &mut (*(*ent).client).ikStatus,
                    &mut (*(*ent).client).ps.origin,
                    &mut (*(*ent).client).ps.viewangles,
                    &mut (*ent).modelScale,
                    500,
                    QTRUE,
                );
            }
        }
    } else if !(*ent).m_pVehicle.is_null()
        && (*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER
    {
        let mut lookAngles: vec3_t = [0.0; 3];

        VectorCopy(&(*(*ent).client).ps.viewangles, legsAngles);
        legsAngles[PITCH as usize] = 0.0;
        AnglesToAxis(legsAngles, legs);

        VectorCopy(&(*(*ent).client).ps.viewangles, &mut lookAngles);
        lookAngles[YAW as usize] = 0.0;
        lookAngles[ROLL as usize] = 0.0;

        BG_G2ATSTAngles((*ent).ghoul2, (*addr_of!(level)).time, &mut lookAngles);
    } else if !(*ent).NPC.is_null() {
        //an NPC not using a humanoid skeleton, do special angle stuff.
        if (*ent).s.eType == ET_NPC
            && (*ent).s.NPC_class == CLASS_VEHICLE
            && !(*ent).m_pVehicle.is_null()
            && (*(*(*ent).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER
        {
            //fighters actually want to take pitch and roll into account for the axial angles
            VectorCopy(&(*(*ent).client).ps.viewangles, legsAngles);
            AnglesToAxis(legsAngles, legs);
        } else {
            G_G2NPCAngles(ent, legs, legsAngles);
        }
    }
}

/// `#define STAFF_KICK_RANGE 16` (w_saber.c:7041) — staff-kick reach fudge, added to
/// `maxs[0]*1.5` in [`G_KickSomeMofos`]'s kick distance.
const STAFF_KICK_RANGE: c_int = 16;

/// `static gentity_t *G_KickTrace(gentity_t *ent, vec3_t kickDir, float kickDist,`
/// `vec3_t kickEnd, int kickDamage, float kickPush)` (w_saber.c:7101).
///
/// One melee kick trace: sweeps a tiny 2-unit box along `kickDir` (or straight to a passed
/// `kickEnd` bolt point, flattening Z), via the ghoul2 trace when `d_saberKickTweak` is set
/// else the plain trace. On a solid hit it debounces repeat-hits within the same anim
/// (`jediKickTime`/`jediKickIndex`), plays the hilt-slam or a random punch sound, applies
/// `MOD_MELEE` damage with no knockback, and — for a valid live/dead enemy not already
/// flying — tosses them with [`G_TossTheMofo`]. Returns the hit entity (or null).
///
/// `kickEnd` may be null (the sole caller passes null); the bolt-point branch is preserved
/// faithfully. No oracle — traces, sounds, damage and global `level`/cvars.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `kickDir` is read, `kickEnd` (if non-null) points to a
/// readable `vec3_t`.
pub unsafe fn G_KickTrace(
    ent: *mut gentity_t,
    kickDir: &vec3_t,
    kickDist: f32,
    kickEnd: *const vec3_t,
    kickDamage: c_int,
    kickPush: f32,
) -> *mut gentity_t {
    let mut traceOrg: vec3_t = [0.0; 3];
    let mut traceEnd: vec3_t = [0.0; 3];
    let mut kickMins: vec3_t = [0.0; 3];
    let mut kickMaxs: vec3_t = [0.0; 3];
    let mut hitEnt: *mut gentity_t = null_mut();
    VectorSet(&mut kickMins, -2.0, -2.0, -2.0);
    VectorSet(&mut kickMaxs, 2.0, 2.0, 2.0);
    //FIXME: variable kick height?
    if !kickEnd.is_null() && VectorCompare(&*kickEnd, &vec3_origin) == QFALSE {
        //they passed us the end point of the trace, just use that
        //this makes the trace flat
        VectorSet(
            &mut traceOrg,
            (*ent).r.currentOrigin[0],
            (*ent).r.currentOrigin[1],
            (*kickEnd)[2],
        );
        VectorCopy(&*kickEnd, &mut traceEnd);
    } else {
        //extrude
        VectorSet(
            &mut traceOrg,
            (*ent).r.currentOrigin[0],
            (*ent).r.currentOrigin[1],
            (*ent).r.currentOrigin[2] + (*ent).r.maxs[2] * 0.5,
        );
        VectorMA(&traceOrg.clone(), kickDist, kickDir, &mut traceEnd);
    }

    let mut trace: trace_t = if (*addr_of!(d_saberKickTweak)).integer != 0 {
        trap::G2Trace(
            &traceOrg,
            &kickMins,
            &kickMaxs,
            &traceEnd,
            (*ent).s.number,
            MASK_SHOT,
            G2TRFLAG_DOGHOULTRACE | G2TRFLAG_GETSURFINDEX | G2TRFLAG_THICK | G2TRFLAG_HITCORPSES,
            (*addr_of!(g_g2TraceLod)).integer,
        )
    } else {
        trap::Trace(
            &traceOrg,
            &kickMins,
            &kickMaxs,
            &traceEnd,
            (*ent).s.number,
            MASK_SHOT,
        )
    };

    //G_TestLine(traceOrg, traceEnd, 0x0000ff, 5000);
    if trace.fraction < 1.0 && trace.startsolid == 0 && trace.allsolid == 0 {
        if (*(*ent).client).jediKickTime > (*addr_of!(level)).time
            && trace.entityNum as c_int == (*(*ent).client).jediKickIndex
        {
            //we are hitting the same ent we last hit in this same anim, don't hit it again
            return null_mut();
        }
        (*(*ent).client).jediKickIndex = trace.entityNum as c_int;
        (*(*ent).client).jediKickTime = (*addr_of!(level)).time + (*(*ent).client).ps.legsTimer;

        hitEnt =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(trace.entityNum as usize);
        //FIXME: regardless of what we hit, do kick hit sound and impact effect
        //G_PlayEffect( "misc/kickHit", trace.endpos, trace.plane.normal );
        if (*(*ent).client).ps.torsoAnim == BOTH_A7_HILT {
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex("sound/movers/objects/saber_slam"),
            );
        } else {
            G_Sound(
                ent,
                CHAN_AUTO,
                G_SoundIndex(
                    &CStr::from_ptr(va(format_args!(
                        "sound/weapons/melee/punch{}",
                        Q_irand(1, 4)
                    )))
                    .to_string_lossy(),
                ),
            );
        }
        if (*hitEnt).inuse != QFALSE {
            //we hit an entity
            //FIXME: don't hit same ent more than once per kick
            if (*hitEnt).takedamage != QFALSE {
                //hurt it
                if !(*hitEnt).client.is_null() {
                    (*(*hitEnt).client).ps.otherKiller = (*ent).s.number;
                    (*(*hitEnt).client).ps.otherKillerDebounceTime =
                        (*addr_of!(level)).time + 10000;
                    (*(*hitEnt).client).ps.otherKillerTime = (*addr_of!(level)).time + 10000;
                    (*(*hitEnt).client).otherKillerMOD = MOD_MELEE;
                    (*(*hitEnt).client).otherKillerVehWeapon = 0;
                    (*(*hitEnt).client).otherKillerWeaponType = WP_NONE;
                }

                if (*addr_of!(d_saberKickTweak)).integer != 0 {
                    G_Damage(
                        hitEnt,
                        ent,
                        ent,
                        &mut kickDir.clone(),
                        &mut trace.endpos,
                        (kickDamage as f32 * 0.2) as c_int,
                        DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE,
                    );
                } else {
                    G_Damage(
                        hitEnt,
                        ent,
                        ent,
                        &mut kickDir.clone(),
                        &mut trace.endpos,
                        kickDamage,
                        DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE,
                    );
                }
            }
            if !(*hitEnt).client.is_null()
                && ((*(*hitEnt).client).ps.pm_flags & PMF_TIME_KNOCKBACK) == 0 //not already flying through air?  Intended to stop multiple hits, but...
                && G_CanBeEnemy(ent, hitEnt) != QFALSE
            {
                //FIXME: this should not always work
                if (*hitEnt).health <= 0 {
                    //we kicked a dead guy
                    //throw harder - FIXME: no matter how hard I push them, they don't go anywhere... corpses use less physics???
                    //	G_Throw( hitEnt, kickDir, kickPush*4 );
                    //see if we should play a better looking death on them
                    //	G_ThrownDeathAnimForDeathAnim( hitEnt, trace.endpos );
                    G_TossTheMofo(hitEnt, kickDir, kickPush * 4.0);
                } else {
                    /*
                    G_Throw( hitEnt, kickDir, kickPush );
                    if ( kickPush >= 75.0f && !Q_irand( 0, 2 ) )
                    {
                        G_Knockdown( hitEnt, ent, kickDir, 300, qtrue );
                    }
                    else
                    {
                        G_Knockdown( hitEnt, ent, kickDir, kickPush, qtrue );
                    }
                    */
                    if kickPush >= 75.0 && Q_irand(0, 2) == 0 {
                        G_TossTheMofo(hitEnt, kickDir, 300.0);
                    } else {
                        G_TossTheMofo(hitEnt, kickDir, kickPush);
                    }
                }
            }
        }
    }
    hitEnt
}

/// `static void G_KickSomeMofos(gentity_t *ent)` (w_saber.c:7215).
///
/// Per-frame driver for the staff/getup kicks: from the current legs anim and how far into
/// it we are (`elapsedTime`/`remainingTime` off [`BG_AnimLength`]), pick the kick direction
/// — tracing to the relevant foot/hand bolt via [`G_GetBoltPosition`] when available, else
/// guessing off the yaw — and at the right window fire a single [`G_KickTrace`]. The big
/// per-anim `switch` covers the A7 kick family (front/back/side/air/spin/butterfly) and the
/// broll/froll getups. The sole `G_KickTrace` call passes a null `kickEnd` (faithful to the
/// C, which traces along `kickDir` rather than to `kickEnd`).
///
/// No oracle — bolt positions, traces and global `level`/cvars.
///
/// # Safety
/// `ent`/`ent->client` must be valid.
pub unsafe fn G_KickSomeMofos(ent: *mut gentity_t) {
    let mut kickDir: vec3_t = [0.0; 3];
    let mut kickEnd: vec3_t = [0.0; 3];
    let mut fwdAngs: vec3_t = [0.0; 3];
    let animLength = BG_AnimLength((*ent).localAnimIndex, (*(*ent).client).ps.legsAnim) as f32;
    let elapsedTime = animLength - (*(*ent).client).ps.legsTimer as f32;
    let remainingTime = animLength - elapsedTime;
    let mut kickDist = (*ent).r.maxs[0] * 1.5 + STAFF_KICK_RANGE as f32 + 8.0; //fudge factor of 8
    let kickDamage = Q_irand(10, 15); //Q_irand( 3, 8 ); //since it can only hit a guy once now
    let mut kickPush = flrand(50.0, 100.0) as c_int;
    let mut doKick: qboolean = QFALSE;
    let ri = addr_of!((*(*ent).client).renderInfo);

    VectorSet(&mut kickDir, 0.0, 0.0, 0.0);
    VectorSet(&mut kickEnd, 0.0, 0.0, 0.0);
    VectorSet(
        &mut fwdAngs,
        0.0,
        (*(*ent).client).ps.viewangles[YAW as usize],
        0.0,
    );

    //HMM... or maybe trace from origin to footRBolt/footLBolt?  Which one?  G2 trace?  Will do hitLoc, if so...
    if (*(*ent).client).ps.torsoAnim == BOTH_A7_HILT {
        if elapsedTime >= 250.0 && remainingTime >= 250.0 {
            //front
            doKick = QTRUE;
            if (*ri).handRBolt != -1 {
                //actually trace to a bolt
                G_GetBoltPosition(ent, (*ri).handRBolt, &mut kickEnd, 0);
                VectorSubtract(&kickEnd.clone(), &(*(*ent).client).ps.origin, &mut kickDir);
                kickDir[2] = 0.0; //ah, flatten it, I guess...
                VectorNormalize(&mut kickDir);
            } else {
                //guess
                AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
            }
        }
    } else {
        match (*(*ent).client).ps.legsAnim {
            x if x == BOTH_GETUP_BROLL_B
                || x == BOTH_GETUP_BROLL_F
                || x == BOTH_GETUP_FROLL_B
                || x == BOTH_GETUP_FROLL_F =>
            {
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //front
                    doKick = QTRUE;
                    if (*ri).footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(ent, (*ri).footRBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*(*ent).client).ps.origin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                    }
                }
            }
            x if x == BOTH_A7_KICK_F_AIR
                || x == BOTH_A7_KICK_B_AIR
                || x == BOTH_A7_KICK_R_AIR
                || x == BOTH_A7_KICK_L_AIR =>
            {
                if elapsedTime >= 100.0 && remainingTime >= 250.0 {
                    //air
                    doKick = QTRUE;
                    if (*ri).footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(ent, (*ri).footRBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                    }
                }
            }
            x if x == BOTH_A7_KICK_F => {
                //FIXME: push forward?
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //front
                    doKick = QTRUE;
                    if (*ri).footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(ent, (*ri).footRBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                    }
                }
            }
            x if x == BOTH_A7_KICK_B => {
                //FIXME: push back?
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //back
                    doKick = QTRUE;
                    if (*ri).footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(ent, (*ri).footRBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                        VectorScale(&kickDir.clone(), -1.0, &mut kickDir);
                    }
                }
            }
            x if x == BOTH_A7_KICK_R => {
                //FIXME: push right?
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //right
                    doKick = QTRUE;
                    if (*ri).footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(ent, (*ri).footRBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(&fwdAngs, None, Some(&mut kickDir), None);
                    }
                }
            }
            x if x == BOTH_A7_KICK_L => {
                //FIXME: push left?
                if elapsedTime >= 250.0 && remainingTime >= 250.0 {
                    //left
                    doKick = QTRUE;
                    if (*ri).footLBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(ent, (*ri).footLBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                    } else {
                        //guess
                        AngleVectors(&fwdAngs, None, Some(&mut kickDir), None);
                        VectorScale(&kickDir.clone(), -1.0, &mut kickDir);
                    }
                }
            }
            x if x == BOTH_A7_KICK_S => {
                kickPush = flrand(75.0, 125.0) as c_int;
                if (*ri).footRBolt != -1 {
                    //actually trace to a bolt
                    if elapsedTime >= 550.0 && elapsedTime <= 1050.0 {
                        doKick = QTRUE;
                        G_GetBoltPosition(ent, (*ri).footRBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                        //NOTE: have to fudge this a little because it's not getting enough range with the anim as-is
                        VectorMA(&kickEnd.clone(), 8.0, &kickDir, &mut kickEnd);
                    }
                } else {
                    //guess
                    if elapsedTime >= 400.0 && elapsedTime < 500.0 {
                        //front
                        doKick = QTRUE;
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                    } else if elapsedTime >= 500.0 && elapsedTime < 600.0 {
                        //front-right?
                        doKick = QTRUE;
                        fwdAngs[YAW as usize] += 45.0;
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                    } else if elapsedTime >= 600.0 && elapsedTime < 700.0 {
                        //right
                        doKick = QTRUE;
                        AngleVectors(&fwdAngs, None, Some(&mut kickDir), None);
                    } else if elapsedTime >= 700.0 && elapsedTime < 800.0 {
                        //back-right?
                        doKick = QTRUE;
                        fwdAngs[YAW as usize] += 45.0;
                        AngleVectors(&fwdAngs, None, Some(&mut kickDir), None);
                    } else if elapsedTime >= 800.0 && elapsedTime < 900.0 {
                        //back
                        doKick = QTRUE;
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                        VectorScale(&kickDir.clone(), -1.0, &mut kickDir);
                    } else if elapsedTime >= 900.0 && elapsedTime < 1000.0 {
                        //back-left?
                        doKick = QTRUE;
                        fwdAngs[YAW as usize] += 45.0;
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                    } else if elapsedTime >= 1000.0 && elapsedTime < 1100.0 {
                        //left
                        doKick = QTRUE;
                        AngleVectors(&fwdAngs, None, Some(&mut kickDir), None);
                        VectorScale(&kickDir.clone(), -1.0, &mut kickDir);
                    } else if elapsedTime >= 1100.0 && elapsedTime < 1200.0 {
                        //front-left?
                        doKick = QTRUE;
                        fwdAngs[YAW as usize] += 45.0;
                        AngleVectors(&fwdAngs, None, Some(&mut kickDir), None);
                        VectorScale(&kickDir.clone(), -1.0, &mut kickDir);
                    }
                }
            }
            x if x == BOTH_A7_KICK_BF => {
                kickPush = flrand(75.0, 125.0) as c_int;
                kickDist += 20.0;
                if elapsedTime < 1500.0 {
                    //auto-aim!
                    //			overridAngles = PM_AdjustAnglesForBFKick( ent, ucmd, fwdAngs, qboolean(elapsedTime<850) )?qtrue:overridAngles;
                    //FIXME: if we haven't done the back kick yet and there's no-one there to
                    //			kick anymore, go into some anim that returns us to our base stance
                }
                if (*ri).footRBolt != -1 {
                    //actually trace to a bolt
                    if (elapsedTime >= 750.0 && elapsedTime < 850.0)
                        || (elapsedTime >= 1400.0 && elapsedTime < 1500.0)
                    {
                        //right, though either would do
                        doKick = QTRUE;
                        G_GetBoltPosition(ent, (*ri).footRBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                        //NOTE: have to fudge this a little because it's not getting enough range with the anim as-is
                        VectorMA(&kickEnd.clone(), 8.0, &kickDir, &mut kickEnd);
                    }
                } else {
                    //guess
                    if elapsedTime >= 250.0 && elapsedTime < 350.0 {
                        //front
                        doKick = QTRUE;
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                    } else if elapsedTime >= 350.0 && elapsedTime < 450.0 {
                        //back
                        doKick = QTRUE;
                        AngleVectors(&fwdAngs, Some(&mut kickDir), None, None);
                        VectorScale(&kickDir.clone(), -1.0, &mut kickDir);
                    }
                }
            }
            x if x == BOTH_A7_KICK_RL => {
                kickPush = flrand(75.0, 125.0) as c_int;
                kickDist += 10.0;

                //ok, I'm tracing constantly on these things, they NEVER hit otherwise (in MP at least)

                //FIXME: auto aim at enemies on the side of us?
                //overridAngles = PM_AdjustAnglesForRLKick( ent, ucmd, fwdAngs, qboolean(elapsedTime<850) )?qtrue:overridAngles;
                //if ( elapsedTime >= 250 && elapsedTime < 350 )
                if (*addr_of!(level)).framenum & 1 != 0 {
                    //right
                    doKick = QTRUE;
                    if (*ri).footRBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(ent, (*ri).footRBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                        //NOTE: have to fudge this a little because it's not getting enough range with the anim as-is
                        VectorMA(&kickEnd.clone(), 8.0, &kickDir, &mut kickEnd);
                    } else {
                        //guess
                        AngleVectors(&fwdAngs, None, Some(&mut kickDir), None);
                    }
                }
                //else if ( elapsedTime >= 350 && elapsedTime < 450 )
                else {
                    //left
                    doKick = QTRUE;
                    if (*ri).footLBolt != -1 {
                        //actually trace to a bolt
                        G_GetBoltPosition(ent, (*ri).footLBolt, &mut kickEnd, 0);
                        VectorSubtract(&kickEnd.clone(), &(*ent).r.currentOrigin, &mut kickDir);
                        kickDir[2] = 0.0; //ah, flatten it, I guess...
                        VectorNormalize(&mut kickDir);
                        //NOTE: have to fudge this a little because it's not getting enough range with the anim as-is
                        VectorMA(&kickEnd.clone(), 8.0, &kickDir, &mut kickEnd);
                    } else {
                        //guess
                        AngleVectors(&fwdAngs, None, Some(&mut kickDir), None);
                        VectorScale(&kickDir.clone(), -1.0, &mut kickDir);
                    }
                }
            }
            _ => {}
        }
    }

    if doKick != QFALSE {
        //		G_KickTrace( ent, kickDir, kickDist, kickEnd, kickDamage, kickPush );
        G_KickTrace(ent, &kickDir, kickDist, null(), kickDamage, kickPush as f32);
    }
}

/// `static void G_GrabSomeMofos(gentity_t *self)` (w_saber.c:7540).
///
/// The Kyle melee-grab: trace a small box from origin out to the right-hand bolt; if it hits
/// a live, enemy player/NPC at roughly the same height who isn't already in a grapple, latch
/// on. Forward move picks the grab-punch (`BOTH_KYLE_PA_1`) and back move the knee-throw
/// (`BOTH_KYLE_PA_2`); both sides get the matching torture/victim anim via [`G_SetAnim`],
/// the victim's saber is holstered off, and torso timers are synced. A failed grab plays
/// `BOTH_KYLE_MISS`.
///
/// No oracle — ghoul2 bolt matrix + trace, anims, sounds and global `level`.
///
/// # Safety
/// `self_`/`self_->client` must be valid.
pub unsafe fn G_GrabSomeMofos(self_: *mut gentity_t) {
    let ri = addr_of!((*(*self_).client).renderInfo);
    let mut boltMatrix: mdxaBone_t = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut flatAng: vec3_t = [0.0; 3];
    let mut pos: vec3_t = [0.0; 3];
    let mut grabMins: vec3_t = [0.0; 3];
    let mut grabMaxs: vec3_t = [0.0; 3];

    if (*self_).ghoul2.is_null() || (*ri).handRBolt == -1 {
        //no good
        return;
    }

    VectorSet(&mut flatAng, 0.0, (*(*self_).client).ps.viewangles[1], 0.0);
    trap::G2API_GetBoltMatrix(
        (*self_).ghoul2,
        0,
        (*ri).handRBolt,
        &mut boltMatrix,
        &flatAng,
        &(*(*self_).client).ps.origin,
        (*addr_of!(level)).time,
        null_mut(),
        &(*self_).modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut pos);

    VectorSet(&mut grabMins, -4.0, -4.0, -4.0);
    VectorSet(&mut grabMaxs, 4.0, 4.0, 4.0);

    //trace from my origin to my hand, if we hit anyone then get 'em
    let trace: trace_t = trap::G2Trace(
        &(*(*self_).client).ps.origin,
        &grabMins,
        &grabMaxs,
        &pos,
        (*self_).s.number,
        MASK_SHOT,
        G2TRFLAG_DOGHOULTRACE | G2TRFLAG_GETSURFINDEX | G2TRFLAG_THICK | G2TRFLAG_HITCORPSES,
        (*addr_of!(g_g2TraceLod)).integer,
    );

    if trace.fraction != 1.0 && (trace.entityNum as c_int) < ENTITYNUM_WORLD {
        let grabbed =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(trace.entityNum as usize);

        if (*grabbed).inuse != QFALSE
            && ((*grabbed).s.eType == ET_PLAYER || (*grabbed).s.eType == ET_NPC)
            && !(*grabbed).client.is_null()
            && (*grabbed).health > 0
            && G_CanBeEnemy(self_, grabbed) != QFALSE
            && G_PrettyCloseIGuess(
                (*(*grabbed).client).ps.origin[2],
                (*(*self_).client).ps.origin[2],
                4.0,
            ) != QFALSE
            && (BG_InGrappleMove((*(*grabbed).client).ps.torsoAnim) == 0
                || (*(*grabbed).client).ps.torsoAnim == BOTH_KYLE_GRAB)
            && (BG_InGrappleMove((*(*grabbed).client).ps.legsAnim) == 0
                || (*(*grabbed).client).ps.legsAnim == BOTH_KYLE_GRAB)
        {
            //grabbed an active player/npc
            let mut tortureAnim: c_int = -1;
            let mut correspondingAnim: c_int = -1;

            if (*(*self_).client).pers.cmd.forwardmove > 0 {
                //punch grab
                tortureAnim = BOTH_KYLE_PA_1;
                correspondingAnim = BOTH_PLAYER_PA_1;
            } else if (*(*self_).client).pers.cmd.forwardmove < 0 {
                //knee-throw
                tortureAnim = BOTH_KYLE_PA_2;
                correspondingAnim = BOTH_PLAYER_PA_2;
            }

            if tortureAnim == -1 || correspondingAnim == -1 {
                if (*(*self_).client).ps.torsoTimer < 300 && (*(*self_).client).grappleState == 0 {
                    //you failed to grab anyone, play the "failed to grab" anim
                    G_SetAnim(
                        self_,
                        &mut (*(*self_).client).pers.cmd,
                        SETANIM_BOTH,
                        BOTH_KYLE_MISS,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        0,
                    );
                    if (*(*self_).client).ps.torsoAnim == BOTH_KYLE_MISS {
                        //providing the anim set succeeded..
                        (*(*self_).client).ps.weaponTime = (*(*self_).client).ps.torsoTimer;
                    }
                }
                return;
            }

            (*(*self_).client).grappleIndex = (*grabbed).s.number;
            (*(*self_).client).grappleState = 1;

            (*(*grabbed).client).grappleIndex = (*self_).s.number;
            (*(*grabbed).client).grappleState = 20;

            //time to crack some heads
            G_SetAnim(
                self_,
                &mut (*(*self_).client).pers.cmd,
                SETANIM_BOTH,
                tortureAnim,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            if (*(*self_).client).ps.torsoAnim == tortureAnim {
                //providing the anim set succeeded..
                (*(*self_).client).ps.weaponTime = (*(*self_).client).ps.torsoTimer;
            }

            G_SetAnim(
                grabbed,
                &mut (*(*grabbed).client).pers.cmd,
                SETANIM_BOTH,
                correspondingAnim,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            if (*(*grabbed).client).ps.torsoAnim == correspondingAnim {
                //providing the anim set succeeded..
                if (*(*grabbed).client).ps.weapon == WP_SABER {
                    //turn it off
                    if (*(*grabbed).client).ps.saberHolstered == 0 {
                        (*(*grabbed).client).ps.saberHolstered = 2;
                        if (*(*grabbed).client).saber[0].soundOff != 0 {
                            G_Sound(grabbed, CHAN_AUTO, (*(*grabbed).client).saber[0].soundOff);
                        }
                        if (*(*grabbed).client).saber[1].soundOff != 0
                            && (*(*grabbed).client).saber[1].model[0] != 0
                        {
                            G_Sound(grabbed, CHAN_AUTO, (*(*grabbed).client).saber[1].soundOff);
                        }
                    }
                }
                if (*(*grabbed).client).ps.torsoTimer < (*(*self_).client).ps.torsoTimer {
                    //make sure they stay in the anim at least as long as the grabber
                    (*(*grabbed).client).ps.torsoTimer = (*(*self_).client).ps.torsoTimer;
                }
                (*(*grabbed).client).ps.weaponTime = (*(*grabbed).client).ps.torsoTimer;
            }
        }
    }

    if (*(*self_).client).ps.torsoTimer < 300 && (*(*self_).client).grappleState == 0 {
        //you failed to grab anyone, play the "failed to grab" anim
        G_SetAnim(
            self_,
            &mut (*(*self_).client).pers.cmd,
            SETANIM_BOTH,
            BOTH_KYLE_MISS,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            0,
        );
        if (*(*self_).client).ps.torsoAnim == BOTH_KYLE_MISS {
            //providing the anim set succeeded..
            (*(*self_).client).ps.weaponTime = (*(*self_).client).ps.torsoTimer;
        }
    }
}

/// `#define SABER_MAX_THROW_DISTANCE 700` (w_saber.c:6686) — per-rank cap on how far a thrown saber
/// may travel before it autoreturns.
const SABER_MAX_THROW_DISTANCE: c_int = 700;

/// `void saberFirstThrown(gentity_t *saberent)` (w_saber.c:6688) — per-frame think for an outgoing
/// thrown saber. Bails to [`MakeDeadSaber`]/[`G_FreeEntity`] if the saber has no owner or the owner
/// is gone/spectating. If the owner is dead or has lost saber-offense, it converts the saber to the
/// dropped [`SaberGotHit`]/[`SaberUpdateSelf`] state and removes its model. Otherwise it autoreturns
/// the saber (via [`thrownSaberTouch`]) when the owner released alt-attack ≥500ms after the throw,
/// when it's been out >6s, under ysalamiri/no-force ([`BG_HasYsalamiri`]/[`BG_CanUseFPNow`]), or once
/// it passes the rank-scaled [`SABER_MAX_THROW_DISTANCE`]. At saber-throw rank ≥[`FORCE_LEVEL_2`] the
/// saber re-aims toward where the owner points (a [`trap::Trace`] along the view; rank ≥
/// [`FORCE_LEVEL_3`] traces players too and updates faster). Finally it runs
/// [`saberCheckRadiusDamage`] + [`G_RunObject`].
///
/// No oracle: entity-state think over the opaque `gentity_t` driving the thrown-saber side effects,
/// per the side-effect precedent. The C forward `goto runMin` becomes a labeled block
/// (`break 'runMin`); the `trap_Trace` NULL mins/maxs are passed as `&vec3_origin` (the file's
/// established zero-box idiom).
///
/// # Safety
/// `saberent` must point to a valid `gentity_t` whose `r.ownerNum` indexes `g_entities`.
pub unsafe extern "C" fn saberFirstThrown(saberent: *mut gentity_t) {
    let mut vSub: vec3_t = [0.0; 3];
    let vLen: f32;
    let saberOwn: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
        .add((*saberent).r.ownerNum as usize);

    if (*saberent).r.ownerNum == ENTITYNUM_NONE as c_int {
        MakeDeadSaber(saberent);

        (*saberent).think = Some(G_FreeEntity);
        (*saberent).nextthink = (*addr_of!(level)).time;
        return;
    }

    if saberOwn.is_null()
        || (*saberOwn).inuse != QTRUE
        || (*saberOwn).client.is_null()
        || (*(*saberOwn).client).sess.sessionTeam == TEAM_SPECTATOR
    {
        MakeDeadSaber(saberent);

        (*saberent).think = Some(G_FreeEntity);
        (*saberent).nextthink = (*addr_of!(level)).time;
        return;
    }

    if (*saberOwn).health < 1
        || (*(*saberOwn).client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] == 0
    {
        //He's dead, just go back to our normal saber status
        (*saberent).touch = Some(SaberGotHit);
        (*saberent).think = Some(SaberUpdateSelf);
        (*saberent).genericValue5 = 0;
        (*saberent).nextthink = (*addr_of!(level)).time;

        if !(*saberOwn).client.is_null() && (*(*saberOwn).client).saber[0].soundOff != 0 {
            G_Sound(saberent, CHAN_AUTO, (*(*saberOwn).client).saber[0].soundOff);
        }
        MakeDeadSaber(saberent);

        (*saberent).r.svFlags |= SVF_NOCLIENT as c_int;
        (*saberent).r.contents = CONTENTS_LIGHTSABER;
        SetSaberBoxSize(saberent);
        (*saberent).s.loopSound = 0;
        (*saberent).s.loopIsSoundset = QFALSE;
        WP_SaberRemoveG2Model(saberent);

        (*(*saberOwn).client).ps.saberInFlight = QFALSE;
        (*(*saberOwn).client).ps.saberEntityState = 0;
        (*(*saberOwn).client).ps.saberThrowDelay = (*addr_of!(level)).time + 500;
        (*(*saberOwn).client).ps.saberCanThrow = QFALSE;

        return;
    }

    'runMin: {
        if ((*addr_of!(level)).time - (*(*saberOwn).client).ps.saberDidThrowTime) > 500 {
            if ((*(*saberOwn).client).buttons & BUTTON_ALT_ATTACK) == 0 {
                //If owner releases altattack 500ms or later after throwing saber, it autoreturns
                thrownSaberTouch(saberent, saberent, core::ptr::null_mut());
                break 'runMin;
            } else if ((*addr_of!(level)).time - (*(*saberOwn).client).ps.saberDidThrowTime) > 6000
            {
                //if it's out longer than 6 seconds, return it
                thrownSaberTouch(saberent, saberent, core::ptr::null_mut());
                break 'runMin;
            }
        }

        if BG_HasYsalamiri(
            (*addr_of!(g_gametype)).integer,
            &mut (*(*saberOwn).client).ps,
        ) != QFALSE
        {
            thrownSaberTouch(saberent, saberent, core::ptr::null_mut());
            break 'runMin;
        }

        if BG_CanUseFPNow(
            (*addr_of!(g_gametype)).integer,
            &mut (*(*saberOwn).client).ps,
            (*addr_of!(level)).time,
            FP_SABERTHROW,
        ) == QFALSE
        {
            thrownSaberTouch(saberent, saberent, core::ptr::null_mut());
            break 'runMin;
        }

        VectorSubtract(
            &(*(*saberOwn).client).ps.origin,
            &(*saberent).r.currentOrigin,
            &mut vSub,
        );
        vLen = VectorLength(&vSub);

        if vLen
            >= (SABER_MAX_THROW_DISTANCE
                * (*(*saberOwn).client).ps.fd.forcePowerLevel[FP_SABERTHROW as usize])
                as f32
        {
            thrownSaberTouch(saberent, saberent, core::ptr::null_mut());
            break 'runMin;
        }

        if (*(*saberOwn).client).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] >= FORCE_LEVEL_2
            && (*saberent).speed < (*addr_of!(level)).time as f32
        {
            //if owner is rank 3 in saber throwing, the saber goes where he points
            let mut fwd: vec3_t = [0.0; 3];
            let mut traceFrom: vec3_t = [0.0; 3];
            let mut traceTo: vec3_t = [0.0; 3];
            let mut dir: vec3_t = [0.0; 3];

            AngleVectors(
                &(*(*saberOwn).client).ps.viewangles,
                Some(&mut fwd),
                None,
                None,
            );

            VectorCopy(&(*(*saberOwn).client).ps.origin, &mut traceFrom);
            traceFrom[2] += (*(*saberOwn).client).ps.viewheight as f32;

            VectorCopy(&traceFrom, &mut traceTo);
            traceTo[0] += fwd[0] * 4096.0;
            traceTo[1] += fwd[1] * 4096.0;
            traceTo[2] += fwd[2] * 4096.0;

            saberMoveBack(saberent, QFALSE);
            VectorCopy(&(*saberent).r.currentOrigin, &mut (*saberent).s.pos.trBase);

            let tr = if (*(*saberOwn).client).ps.fd.forcePowerLevel[FP_SABERTHROW as usize]
                >= FORCE_LEVEL_3
            {
                //if highest saber throw rank, we can direct the saber toward players directly by looking at them
                trap::Trace(
                    &traceFrom,
                    &vec3_origin,
                    &vec3_origin,
                    &traceTo,
                    (*saberOwn).s.number,
                    MASK_PLAYERSOLID,
                )
            } else {
                trap::Trace(
                    &traceFrom,
                    &vec3_origin,
                    &vec3_origin,
                    &traceTo,
                    (*saberOwn).s.number,
                    MASK_SOLID,
                )
            };

            VectorSubtract(&tr.endpos, &(*saberent).r.currentOrigin, &mut dir);

            VectorNormalize(&mut dir);

            VectorScale(&dir, 500.0, &mut (*saberent).s.pos.trDelta);
            (*saberent).s.pos.trTime = (*addr_of!(level)).time;

            if (*(*saberOwn).client).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] >= FORCE_LEVEL_3
            {
                //we'll treat them to a quicker update rate if their throw rank is high enough
                (*saberent).speed = ((*addr_of!(level)).time + 100) as f32;
            } else {
                (*saberent).speed = ((*addr_of!(level)).time + 400) as f32;
            }
        }
    }

    // runMin:
    saberCheckRadiusDamage(saberent, 0);
    G_RunObject(saberent);
}

/// `void WP_SaberPositionUpdate(gentity_t *self, usercmd_t *ucmd)` (w_saber.c:7655).
///
/// rww's per-frame server-side saber maintenance: keeps the server ghoul2 client instance,
/// the saber entity position, and the blade trace endpoints as current as possible so
/// contact detection is realistic. Dispatches the kick/grab/grapple melee states first
/// ([`G_KickSomeMofos`]/[`G_GrabSomeMofos`] + the Kyle-grapple smash sequence), tries to
/// steal the client g2 instance ([`trap_G2API_OverrideServer`](crate::trap::G2API_OverrideServer)),
/// predicts a velocity-led origin, drives the leg/torso angles via [`G_G2PlayerAngles`], spawns
/// or returns the thrown saber entity, then cycles each saber/blade doing the bolt-matrix read
/// and damage traces ([`CheckSaberDamage`]/[`G_SPSaberDamageTraceLerped`]) before applying
/// accumulated damage and clash effects.
///
/// No oracle — pervasive gentity/gclient/ps/ghoul2 state plus traces, sounds, damage and
/// global `level`/cvars. The `#ifdef _DEBUG g_disableServerG2` early-out is retail-omitted.
/// The two forward `goto`s (`nextStep`/`finalUpdate`) become labeled blocks; `saberFirstThrown`
/// is installed through the `extern "C"` shim above (blocked sibling).
///
/// # Safety
/// `self_` may be null/invalid — guarded internally; `_ucmd` is unused (kept for ABI fidelity).
pub unsafe fn WP_SaberPositionUpdate(self_: *mut gentity_t, _ucmd: *mut usercmd_t) {
    let mut mySaber: *mut gentity_t = null_mut();
    let mut boltMatrix: mdxaBone_t = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut properAngles: vec3_t = [0.0; 3];
    let mut properOrigin: vec3_t = [0.0; 3];
    let mut boltAngles: vec3_t = [0.0; 3];
    let mut boltOrigin: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut legAxis: [vec3_t; 3] = [[0.0; 3]; 3];
    let mut addVel: vec3_t = [0.0; 3];
    let mut rawAngles: vec3_t = [0.0; 3];
    let mut fVSpeed: f32 = 0.0;
    let mut returnAfterUpdate: c_int = 0;
    let mut animSpeedScale: f32 = 1.0;
    let mut saberNum: c_int;
    let clientOverride: qboolean;
    let mut vehEnt: *mut gentity_t = null_mut();
    let mut rSaberNum: c_int = 0;
    // C: `int rBladeNum = 0;` — the `= 0` init is dead (every read is preceded by a
    // `rBladeNum = 0` inside the saber/blade loops), so deferred here to stay warning-clean.
    let mut rBladeNum: c_int;

    // #ifdef _DEBUG: `if (g_disableServerG2.integer) return;` — debug-build only, retail-omitted.

    if !self_.is_null() && (*self_).inuse != QFALSE && !(*self_).client.is_null() {
        if (*(*self_).client).saberCycleQueue != 0 {
            (*(*self_).client).ps.fd.saberDrawAnimLevel = (*(*self_).client).saberCycleQueue;
        } else {
            (*(*self_).client).ps.fd.saberDrawAnimLevel = (*(*self_).client).ps.fd.saberAnimLevel;
        }
    }

    if !self_.is_null()
        && (*self_).inuse != QFALSE
        && !(*self_).client.is_null()
        && (*(*self_).client).saberCycleQueue != 0
        && ((*(*self_).client).ps.weaponTime <= 0 || (*self_).health < 1)
    {
        //we cycled attack levels while we were busy, so update now that we aren't (even if that means we're dead)
        (*(*self_).client).ps.fd.saberAnimLevel = (*(*self_).client).saberCycleQueue;
        (*(*self_).client).saberCycleQueue = 0;
    }

    if self_.is_null()
        || (*self_).inuse == QFALSE
        || (*self_).client.is_null()
        || (*self_).ghoul2.is_null()
        || (*addr_of!(g2SaberInstance)).is_null()
    {
        return;
    }

    if BG_KickingAnim((*(*self_).client).ps.legsAnim) != 0 {
        //do some kick traces and stuff if we're in the appropriate anim
        G_KickSomeMofos(self_);
    } else if (*(*self_).client).ps.torsoAnim == BOTH_KYLE_GRAB {
        //try to grab someone
        G_GrabSomeMofos(self_);
    } else if (*(*self_).client).grappleState != 0 {
        let grappler = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*(*self_).client).grappleIndex as usize);

        if (*grappler).inuse == QFALSE
            || (*grappler).client.is_null()
            || (*(*grappler).client).grappleIndex != (*self_).s.number
            || BG_InGrappleMove((*(*grappler).client).ps.torsoAnim) == 0
            || BG_InGrappleMove((*(*grappler).client).ps.legsAnim) == 0
            || BG_InGrappleMove((*(*self_).client).ps.torsoAnim) == 0
            || BG_InGrappleMove((*(*self_).client).ps.legsAnim) == 0
            || (*(*self_).client).grappleState == 0
            || (*(*grappler).client).grappleState == 0
            || (*grappler).health < 1
            || (*self_).health < 1
            || G_PrettyCloseIGuess(
                (*(*self_).client).ps.origin[2],
                (*(*grappler).client).ps.origin[2],
                4.0,
            ) == QFALSE
        {
            (*(*self_).client).grappleState = 0;
            if (BG_InGrappleMove((*(*self_).client).ps.torsoAnim) != 0
                && (*(*self_).client).ps.torsoTimer > 100)
                || (BG_InGrappleMove((*(*self_).client).ps.legsAnim) != 0
                    && (*(*self_).client).ps.legsTimer > 100)
            {
                //if they're pretty far from finishing the anim then shove them into another anim
                G_SetAnim(
                    self_,
                    &mut (*(*self_).client).pers.cmd,
                    SETANIM_BOTH,
                    BOTH_KYLE_MISS,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
                if (*(*self_).client).ps.torsoAnim == BOTH_KYLE_MISS {
                    //providing the anim set succeeded..
                    (*(*self_).client).ps.weaponTime = (*(*self_).client).ps.torsoTimer;
                }
            }
        } else {
            let mut grapAng: vec3_t = [0.0; 3];

            VectorSubtract(
                &(*(*grappler).client).ps.origin,
                &(*(*self_).client).ps.origin,
                &mut grapAng,
            );

            if VectorLength(&grapAng) > 64.0 {
                //too far away, break it off
                if (BG_InGrappleMove((*(*self_).client).ps.torsoAnim) != 0
                    && (*(*self_).client).ps.torsoTimer > 100)
                    || (BG_InGrappleMove((*(*self_).client).ps.legsAnim) != 0
                        && (*(*self_).client).ps.legsTimer > 100)
                {
                    (*(*self_).client).grappleState = 0;

                    G_SetAnim(
                        self_,
                        &mut (*(*self_).client).pers.cmd,
                        SETANIM_BOTH,
                        BOTH_KYLE_MISS,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        0,
                    );
                    if (*(*self_).client).ps.torsoAnim == BOTH_KYLE_MISS {
                        //providing the anim set succeeded..
                        (*(*self_).client).ps.weaponTime = (*(*self_).client).ps.torsoTimer;
                    }
                }
            } else {
                vectoangles(&grapAng.clone(), &mut grapAng);
                SetClientViewAngle(self_, &grapAng);

                if (*(*self_).client).grappleState >= 20 {
                    //grapplee
                    //try to position myself at the correct distance from my grappler
                    let idealDist: f32;
                    let mut gFwd: vec3_t = [0.0; 3];
                    let mut idealSpot: vec3_t = [0.0; 3];
                    let trace: trace_t;

                    if (*(*grappler).client).ps.torsoAnim == BOTH_KYLE_PA_1 {
                        //grab punch
                        idealDist = 46.0;
                    } else {
                        //knee-throw
                        idealDist = 34.0;
                    }

                    AngleVectors(
                        &(*(*grappler).client).ps.viewangles,
                        Some(&mut gFwd),
                        None,
                        None,
                    );
                    VectorMA(
                        &(*(*grappler).client).ps.origin,
                        idealDist,
                        &gFwd,
                        &mut idealSpot,
                    );

                    trace = trap::Trace(
                        &(*(*self_).client).ps.origin,
                        &(*self_).r.mins,
                        &(*self_).r.maxs,
                        &idealSpot,
                        (*self_).s.number,
                        (*self_).clipmask,
                    );
                    if trace.startsolid == 0 && trace.allsolid == 0 && trace.fraction == 1.0 {
                        //go there
                        G_SetOrigin(self_, &idealSpot);
                        VectorCopy(&idealSpot, &mut (*(*self_).client).ps.origin);
                    }
                } else if (*(*self_).client).grappleState >= 1 {
                    //grappler
                    if (*(*grappler).client).ps.weapon == WP_SABER {
                        //make sure their saber is shut off
                        if (*(*grappler).client).ps.saberHolstered == 0 {
                            (*(*grappler).client).ps.saberHolstered = 2;
                            if (*(*grappler).client).saber[0].soundOff != 0 {
                                G_Sound(
                                    grappler,
                                    CHAN_AUTO,
                                    (*(*grappler).client).saber[0].soundOff,
                                );
                            }
                            if (*(*grappler).client).saber[1].soundOff != 0
                                && (*(*grappler).client).saber[1].model[0] != 0
                            {
                                G_Sound(
                                    grappler,
                                    CHAN_AUTO,
                                    (*(*grappler).client).saber[1].soundOff,
                                );
                            }
                        }
                    }

                    //check for smashy events
                    if (*(*self_).client).ps.torsoAnim == BOTH_KYLE_PA_1 {
                        //grab punch
                        if (*(*self_).client).grappleState == 1 {
                            //smack
                            if (*(*self_).client).ps.torsoTimer < 3400 {
                                let grapplerAnim = (*(*grappler).client).ps.torsoAnim;
                                let grapplerTime = (*(*grappler).client).ps.torsoTimer;

                                G_Damage(
                                    grappler,
                                    self_,
                                    self_,
                                    null_mut(),
                                    &mut (*(*self_).client).ps.origin,
                                    10,
                                    0,
                                    MOD_MELEE,
                                );

                                //it might try to put them into a pain anim or something, so override it back again
                                if (*grappler).health > 0 {
                                    (*(*grappler).client).ps.torsoAnim = grapplerAnim;
                                    (*(*grappler).client).ps.torsoTimer = grapplerTime;
                                    (*(*grappler).client).ps.legsAnim = grapplerAnim;
                                    (*(*grappler).client).ps.legsTimer = grapplerTime;
                                    (*(*grappler).client).ps.weaponTime = grapplerTime;
                                }
                                (*(*self_).client).grappleState += 1;
                            }
                        } else if (*(*self_).client).grappleState == 2 {
                            //smack!
                            if (*(*self_).client).ps.torsoTimer < 2550 {
                                let grapplerAnim = (*(*grappler).client).ps.torsoAnim;
                                let grapplerTime = (*(*grappler).client).ps.torsoTimer;

                                G_Damage(
                                    grappler,
                                    self_,
                                    self_,
                                    null_mut(),
                                    &mut (*(*self_).client).ps.origin,
                                    10,
                                    0,
                                    MOD_MELEE,
                                );

                                if (*grappler).health > 0 {
                                    (*(*grappler).client).ps.torsoAnim = grapplerAnim;
                                    (*(*grappler).client).ps.torsoTimer = grapplerTime;
                                    (*(*grappler).client).ps.legsAnim = grapplerAnim;
                                    (*(*grappler).client).ps.legsTimer = grapplerTime;
                                    (*(*grappler).client).ps.weaponTime = grapplerTime;
                                }
                                (*(*self_).client).grappleState += 1;
                            }
                        } else {
                            //SMACK!
                            if (*(*self_).client).ps.torsoTimer < 1300 {
                                let mut tossDir: vec3_t = [0.0; 3];

                                G_Damage(
                                    grappler,
                                    self_,
                                    self_,
                                    null_mut(),
                                    &mut (*(*self_).client).ps.origin,
                                    30,
                                    0,
                                    MOD_MELEE,
                                );

                                (*(*self_).client).grappleState = 0;

                                VectorSubtract(
                                    &(*(*grappler).client).ps.origin,
                                    &(*(*self_).client).ps.origin,
                                    &mut tossDir,
                                );
                                VectorNormalize(&mut tossDir);
                                VectorScale(&tossDir.clone(), 500.0, &mut tossDir);
                                tossDir[2] = 200.0;

                                VectorAdd(
                                    &(*(*grappler).client).ps.velocity.clone(),
                                    &tossDir,
                                    &mut (*(*grappler).client).ps.velocity,
                                );

                                if (*grappler).health > 0 {
                                    //if still alive knock them down
                                    (*(*grappler).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
                                    (*(*grappler).client).ps.forceHandExtendTime =
                                        (*addr_of!(level)).time + 1300;
                                }
                            }
                        }
                    } else if (*(*self_).client).ps.torsoAnim == BOTH_KYLE_PA_2 {
                        //knee throw
                        if (*(*self_).client).grappleState == 1 {
                            //knee to the face
                            if (*(*self_).client).ps.torsoTimer < 3200 {
                                let grapplerAnim = (*(*grappler).client).ps.torsoAnim;
                                let grapplerTime = (*(*grappler).client).ps.torsoTimer;

                                G_Damage(
                                    grappler,
                                    self_,
                                    self_,
                                    null_mut(),
                                    &mut (*(*self_).client).ps.origin,
                                    20,
                                    0,
                                    MOD_MELEE,
                                );

                                if (*grappler).health > 0 {
                                    (*(*grappler).client).ps.torsoAnim = grapplerAnim;
                                    (*(*grappler).client).ps.torsoTimer = grapplerTime;
                                    (*(*grappler).client).ps.legsAnim = grapplerAnim;
                                    (*(*grappler).client).ps.legsTimer = grapplerTime;
                                    (*(*grappler).client).ps.weaponTime = grapplerTime;
                                }
                                (*(*self_).client).grappleState += 1;
                            }
                        } else if (*(*self_).client).grappleState == 2 {
                            //smashed on the ground
                            if (*(*self_).client).ps.torsoTimer < 2000 {
                                //don't do damage on this one, it would look very freaky if they died
                                G_EntitySound(grappler, CHAN_VOICE, G_SoundIndex("*pain100.wav"));
                                (*(*self_).client).grappleState += 1;
                            }
                        } else {
                            //and another smash
                            if (*(*self_).client).ps.torsoTimer < 1000 {
                                G_Damage(
                                    grappler,
                                    self_,
                                    self_,
                                    null_mut(),
                                    &mut (*(*self_).client).ps.origin,
                                    30,
                                    0,
                                    MOD_MELEE,
                                );

                                if (*grappler).health > 0 {
                                    (*(*grappler).client).ps.torsoTimer = 1000;
                                    //G_SetAnim(grappler, &grappler->client->pers.cmd, SETANIM_BOTH, BOTH_GETUP3, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD, 0);
                                    (*(*grappler).client).grappleState = 0;
                                } else {
                                    //override death anim
                                    (*(*grappler).client).ps.torsoAnim = BOTH_DEADFLOP1;
                                    (*(*grappler).client).ps.legsAnim = BOTH_DEADFLOP1;
                                }

                                (*(*self_).client).grappleState = 0;
                            }
                        }
                    } else {
                        //?
                    }
                }
            }
        }
    }

    //If this is a listen server (client+server running on same machine),
    //then lets try to steal the skeleton/etc data off the client instance
    //for this entity to save us processing time.
    clientOverride = trap::G2API_OverrideServer((*self_).ghoul2);

    'finalUpdate: {
        'setup: {
            saberNum = (*(*self_).client).ps.saberEntityNum;

            if saberNum == 0 {
                saberNum = (*(*self_).client).saberStoredIndex;
            }

            if saberNum == 0 {
                returnAfterUpdate = 1;
                break 'setup; //goto nextStep;
            }

            mySaber =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(saberNum as usize);

            if (*self_).health < 1 {
                //we don't want to waste precious CPU time calculating saber positions for corpses. But we want to avoid the saber ent position lagging on spawn, so..
                //I guess it's good to keep the position updated even when contents are 0
                if !mySaber.is_null()
                    && (((*mySaber).r.contents & CONTENTS_LIGHTSABER) != 0
                        || (*mySaber).r.contents == 0)
                    && (*(*self_).client).ps.saberInFlight == QFALSE
                {
                    //Since we haven't got a bolt position, place it on top of the player origin.
                    VectorCopy(
                        &(*(*self_).client).ps.origin,
                        &mut (*mySaber).r.currentOrigin,
                    );
                }

                //I don't want to return now actually, I want to keep g2 instances for corpses up to
                //date because I'm doing better corpse hit detection/dismem (particularly for the
                //npc's)
                //return;
            }

            if BG_SuperBreakWinAnim((*(*self_).client).ps.torsoAnim) != 0 {
                (*(*self_).client).ps.weaponstate = WEAPON_FIRING;
            }
            if (*(*self_).client).ps.weapon != WP_SABER
                || (*(*self_).client).ps.weaponstate == WEAPON_RAISING
                || (*(*self_).client).ps.weaponstate == WEAPON_DROPPING
                || (*self_).health < 1
            {
                if (*(*self_).client).ps.saberInFlight == QFALSE {
                    returnAfterUpdate = 1;
                }
            }

            if (*(*self_).client).ps.saberThrowDelay < (*addr_of!(level)).time {
                if (*(*self_).client).saber[0].saberFlags & SFL_NOT_THROWABLE != 0 {
                    //cant throw it normally!
                    if (*(*self_).client).saber[0].saberFlags & SFL_SINGLE_BLADE_THROWABLE != 0 {
                        //but can throw it if only have 1 blade on
                        if (*(*self_).client).saber[0].numBlades > 1
                            && (*(*self_).client).ps.saberHolstered == 1
                        {
                            //have multiple blades and only one blade on
                            (*(*self_).client).ps.saberCanThrow = QTRUE; //qfalse;
                                                                         //huh? want to be able to throw then right?
                        } else {
                            //multiple blades on, can't throw
                            (*(*self_).client).ps.saberCanThrow = QFALSE;
                        }
                    } else {
                        //never can throw it
                        (*(*self_).client).ps.saberCanThrow = QFALSE;
                    }
                } else {
                    //can throw it!
                    (*(*self_).client).ps.saberCanThrow = QTRUE;
                }
            }
        }
        //nextStep:
        if (*(*self_).client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
            animSpeedScale = 2.0;
        }

        VectorCopy(&(*(*self_).client).ps.origin, &mut properOrigin);

        //try to predict the origin based on velocity so it's more like what the client is seeing
        VectorCopy(&(*(*self_).client).ps.velocity, &mut addVel);
        VectorNormalize(&mut addVel);

        if (*(*self_).client).ps.velocity[0] < 0.0 {
            fVSpeed += -(*(*self_).client).ps.velocity[0];
        } else {
            fVSpeed += (*(*self_).client).ps.velocity[0];
        }
        if (*(*self_).client).ps.velocity[1] < 0.0 {
            fVSpeed += -(*(*self_).client).ps.velocity[1];
        } else {
            fVSpeed += (*(*self_).client).ps.velocity[1];
        }
        if (*(*self_).client).ps.velocity[2] < 0.0 {
            fVSpeed += -(*(*self_).client).ps.velocity[2];
        } else {
            fVSpeed += (*(*self_).client).ps.velocity[2];
        }

        //fVSpeed *= 0.08;
        fVSpeed *= 1.6 / (*addr_of!(g_svfps)).value;

        //Cap it off at reasonable values so the saber box doesn't go flying ahead of us or
        //something if we get a big speed boost from something.
        if fVSpeed > 70.0 {
            fVSpeed = 70.0;
        }
        if fVSpeed < -70.0 {
            fVSpeed = -70.0;
        }

        properOrigin[0] += addVel[0] * fVSpeed;
        properOrigin[1] += addVel[1] * fVSpeed;
        properOrigin[2] += addVel[2] * fVSpeed;

        properAngles[0] = 0.0;
        if (*self_).s.number < MAX_CLIENTS as c_int && (*(*self_).client).ps.m_iVehicleNum != 0 {
            vehEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*(*self_).client).ps.m_iVehicleNum as usize);
            if (*vehEnt).inuse != QFALSE
                && !(*vehEnt).client.is_null()
                && !(*vehEnt).m_pVehicle.is_null()
            {
                properAngles[1] = *(*(*vehEnt).m_pVehicle).m_vOrientation.add(YAW as usize);
            } else {
                properAngles[1] = (*(*self_).client).ps.viewangles[YAW as usize];
                vehEnt = null_mut();
            }
        } else {
            properAngles[1] = (*(*self_).client).ps.viewangles[YAW as usize];
        }
        properAngles[2] = 0.0;

        AnglesToAxis(&properAngles, &mut legAxis);

        UpdateClientRenderinfo(self_, &mut properOrigin, &mut properAngles);

        if clientOverride == QFALSE {
            //if we get the client instance we don't need to do this
            G_G2PlayerAngles(self_, &mut legAxis, &mut properAngles);
        }

        if !vehEnt.is_null() {
            properAngles[1] = *(*(*vehEnt).m_pVehicle).m_vOrientation.add(YAW as usize);
        }

        if returnAfterUpdate != 0 && saberNum != 0 {
            //We don't even need to do GetBoltMatrix if we're only in here to keep the g2 server instance in sync
            //but keep our saber entity in sync too, just copy it over our origin.

            //I guess it's good to keep the position updated even when contents are 0
            if !mySaber.is_null()
                && (((*mySaber).r.contents & CONTENTS_LIGHTSABER) != 0
                    || (*mySaber).r.contents == 0)
                && (*(*self_).client).ps.saberInFlight == QFALSE
            {
                //Since we haven't got a bolt position, place it on top of the player origin.
                VectorCopy(
                    &(*(*self_).client).ps.origin,
                    &mut (*mySaber).r.currentOrigin,
                );
            }

            break 'finalUpdate; //goto finalUpdate;
        }

        if returnAfterUpdate != 0 {
            break 'finalUpdate; //goto finalUpdate;
        }

        //We'll get data for blade 0 first no matter what it is and stick them into
        //the constant ("_Always") values. Later we will handle going through each blade.
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            1,
            0,
            &mut boltMatrix,
            &properAngles,
            &properOrigin,
            (*addr_of!(level)).time,
            null_mut(),
            &(*self_).modelScale,
        );
        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut boltOrigin);
        BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut boltAngles);

        //immediately store these values so we don't have to recalculate this again
        if (*(*self_).client).lastSaberStorageTime != 0
            && ((*addr_of!(level)).time - (*(*self_).client).lastSaberStorageTime) < 200
        {
            //alright
            VectorCopy(
                &(*(*self_).client).lastSaberBase_Always,
                &mut (*(*self_).client).olderSaberBase,
            );
            (*(*self_).client).olderIsValid = QTRUE;
        } else {
            (*(*self_).client).olderIsValid = QFALSE;
        }

        VectorCopy(&boltOrigin, &mut (*(*self_).client).lastSaberBase_Always);
        VectorCopy(&boltAngles, &mut (*(*self_).client).lastSaberDir_Always);
        (*(*self_).client).lastSaberStorageTime = (*addr_of!(level)).time;

        VectorCopy(&boltAngles, &mut rawAngles);

        VectorMA(
            &boltOrigin,
            (*(*self_).client).saber[0].blade[0].lengthMax,
            &boltAngles,
            &mut end,
        );

        if (*(*self_).client).ps.saberEntityNum != 0 {
            //I guess it's good to keep the position updated even when contents are 0
            if !mySaber.is_null()
                && (((*mySaber).r.contents & CONTENTS_LIGHTSABER) != 0
                    || (*mySaber).r.contents == 0)
                && (*(*self_).client).ps.saberInFlight == QFALSE
            {
                //place it roughly in the middle of the saber..
                VectorMA(
                    &boltOrigin,
                    (*(*self_).client).saber[0].blade[0].lengthMax,
                    &boltAngles,
                    &mut (*mySaber).r.currentOrigin,
                );
            }
        }

        boltAngles[YAW as usize] = (*(*self_).client).ps.viewangles[YAW as usize];

        if (*(*self_).client).ps.saberInFlight != QFALSE {
            //do the thrown-saber stuff
            let saberent =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(saberNum as usize);

            if !saberent.is_null() {
                if (*(*self_).client).ps.saberEntityState == 0
                    && (*(*self_).client).ps.saberEntityNum != 0
                {
                    let mut startorg: vec3_t = [0.0; 3];
                    let mut startang: vec3_t = [0.0; 3];
                    let mut dir: vec3_t = [0.0; 3];

                    VectorCopy(&boltOrigin, &mut (*saberent).r.currentOrigin);

                    VectorCopy(&boltOrigin, &mut startorg);
                    VectorCopy(&boltAngles, &mut startang);

                    //startang[0] = 90;
                    //Instead of this we'll sort of fake it and slowly tilt it down on the client via
                    //a perframe method (which doesn't actually affect where or how the saber hits)

                    (*saberent).r.svFlags &= !SVF_NOCLIENT;
                    VectorCopy(&startorg, &mut (*saberent).s.pos.trBase);
                    VectorCopy(&startang, &mut (*saberent).s.apos.trBase);

                    VectorCopy(&startorg, &mut (*saberent).s.origin);
                    VectorCopy(&startang, &mut (*saberent).s.angles);

                    (*saberent).s.saberInFlight = QTRUE;

                    (*saberent).s.apos.trType = TR_LINEAR;
                    (*saberent).s.apos.trDelta[0] = 0.0;
                    (*saberent).s.apos.trDelta[1] = 800.0;
                    (*saberent).s.apos.trDelta[2] = 0.0;

                    (*saberent).s.pos.trType = TR_LINEAR;
                    (*saberent).s.eType = ET_GENERAL;
                    (*saberent).s.eFlags = 0;

                    WP_SaberAddG2Model(
                        saberent,
                        (*(*self_).client).saber[0].model.as_ptr(),
                        (*(*self_).client).saber[0].skin,
                    );

                    (*saberent).s.modelGhoul2 = 127;

                    (*saberent).parent = self_;

                    (*(*self_).client).ps.saberEntityState = 1;

                    //Projectile stuff:
                    AngleVectors(
                        &(*(*self_).client).ps.viewangles,
                        Some(&mut dir),
                        None,
                        None,
                    );

                    (*saberent).nextthink = (*addr_of!(level)).time + FRAMETIME;
                    (*saberent).think = Some(saberFirstThrown);

                    (*saberent).damage = SABER_THROWN_HIT_DAMAGE;
                    (*saberent).methodOfDeath = MOD_SABER;
                    (*saberent).splashMethodOfDeath = MOD_SABER;
                    (*saberent).s.solid = 2;
                    (*saberent).r.contents = CONTENTS_LIGHTSABER;

                    (*saberent).genericValue5 = 0;

                    VectorSet(
                        &mut (*saberent).r.mins,
                        SABERMINS_X,
                        SABERMINS_Y,
                        SABERMINS_Z,
                    );
                    VectorSet(
                        &mut (*saberent).r.maxs,
                        SABERMAXS_X,
                        SABERMAXS_Y,
                        SABERMAXS_Z,
                    );

                    (*saberent).s.genericenemyindex = (*self_).s.number + 1024;

                    (*saberent).touch = Some(thrownSaberTouch);

                    (*saberent).s.weapon = WP_SABER;

                    VectorScale(&dir, 400.0, &mut (*saberent).s.pos.trDelta);
                    (*saberent).s.pos.trTime = (*addr_of!(level)).time;

                    if (*(*self_).client).saber[0].spinSound != 0 {
                        (*saberent).s.loopSound = (*(*self_).client).saber[0].spinSound;
                    } else {
                        (*saberent).s.loopSound = saberSpinSound;
                    }
                    (*saberent).s.loopIsSoundset = QFALSE;

                    (*(*self_).client).ps.saberDidThrowTime = (*addr_of!(level)).time;

                    (*(*self_).client).dangerTime = (*addr_of!(level)).time;
                    (*(*self_).client).ps.eFlags &= !EF_INVULNERABLE;
                    (*(*self_).client).invulnerableTimer = 0;

                    trap::LinkEntity(saberent);
                } else if (*(*self_).client).ps.saberEntityNum != 0 {
                    //only do this stuff if your saber is active and has not been knocked out of the air.
                    VectorCopy(&boltOrigin, &mut (*saberent).pos1);
                    trap::LinkEntity(saberent);

                    if (*saberent).genericValue5 == PROPER_THROWN_VALUE {
                        //return to the owner now, this is a bad state to be in for here..
                        (*saberent).genericValue5 = 0;
                        (*saberent).think = Some(SaberUpdateSelf);
                        (*saberent).nextthink = (*addr_of!(level)).time;
                        WP_SaberRemoveG2Model(saberent);

                        (*(*self_).client).ps.saberInFlight = QFALSE;
                        (*(*self_).client).ps.saberEntityState = 0;
                        (*(*self_).client).ps.saberThrowDelay = (*addr_of!(level)).time + 500;
                        (*(*self_).client).ps.saberCanThrow = QFALSE;
                    }
                }
            }
        }

        /*
        if (self->client->ps.saberInFlight)
        { //if saber is thrown then only do the standard stuff for the left hand saber
            rSaberNum = 1;
        }
        */

        if BG_SabersOff(&mut (*(*self_).client).ps) == QFALSE {
            let saberent =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(saberNum as usize);

            if (*(*self_).client).ps.saberInFlight == QFALSE && !saberent.is_null() {
                (*saberent).r.svFlags |= SVF_NOCLIENT;
                (*saberent).r.contents = CONTENTS_LIGHTSABER;
                SetSaberBoxSize(saberent);
                (*saberent).s.loopSound = 0;
                (*saberent).s.loopIsSoundset = QFALSE;
            }

            if (*(*self_).client).ps.saberLockTime > (*addr_of!(level)).time
                && (*(*self_).client).ps.saberEntityNum != 0
            {
                if (*(*self_).client).ps.saberIdleWound < (*addr_of!(level)).time {
                    let te: *mut gentity_t;
                    let mut dir: vec3_t = [0.0; 3];
                    te = G_TempEntity(
                        &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .add(saberNum as usize))
                        .r
                        .currentOrigin,
                        EV_SABER_BLOCK,
                    );
                    VectorSet(&mut dir, 0.0, 1.0, 0.0);
                    VectorCopy(
                        &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .add(saberNum as usize))
                        .r
                        .currentOrigin,
                        &mut (*te).s.origin,
                    );
                    VectorCopy(&dir, &mut (*te).s.angles);
                    (*te).s.eventParm = 1;

                    (*(*self_).client).ps.saberIdleWound =
                        (*addr_of!(level)).time + Q_irand(400, 600);
                }

                while rSaberNum < MAX_SABERS as c_int {
                    rBladeNum = 0;
                    while rBladeNum < (*(*self_).client).saber[rSaberNum as usize].numBlades {
                        //Don't bother updating the bolt for each blade for this, it's just a very rough fallback method for during saberlocks
                        // ClientManager::ActiveClientNum() -> 0 (MP)
                        VectorCopy(
                            &boltOrigin,
                            &mut (*(*self_).client).saber[saberNum as usize].blade
                                [rBladeNum as usize]
                                .trail
                                .base,
                        );
                        VectorCopy(
                            &end,
                            &mut (*(*self_).client).saber[saberNum as usize].blade
                                [rBladeNum as usize]
                                .trail
                                .tip,
                        );
                        (*(*self_).client).saber[saberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .lastTime = (*addr_of!(level)).time;

                        rBladeNum += 1;
                    }

                    rSaberNum += 1;
                }
                (*(*self_).client).hasCurrentPosition = QTRUE;

                (*(*self_).client).ps.saberBlocked = BLOCKED_NONE;

                break 'finalUpdate; //goto finalUpdate;
            }

            //reset it in case we used it for cycling before
            // C: `rSaberNum = rBladeNum = 0;` — the rBladeNum half is dead (the `while rSaberNum`
            // loop below sets `rBladeNum = 0` before any read), so only rSaberNum is reset here.
            rSaberNum = 0;

            if (*(*self_).client).ps.saberInFlight != QFALSE {
                //if saber is thrown then only do the standard stuff for the left hand saber
                if (*(*self_).client).ps.saberEntityNum == 0 {
                    //however, if saber is not in flight but rather knocked away, our left saber is off, and thus we may do nothing.
                    rSaberNum = 1; //was 2?
                } else {
                    //thrown saber still in flight, so do damage
                    rSaberNum = 0; //was 1?
                }
            }

            WP_SaberClearDamage();
            saberDoClashEffect = QFALSE;

            //Now cycle through each saber and each blade on the saber and do damage traces.
            while rSaberNum < MAX_SABERS as c_int {
                if (*(*self_).client).saber[rSaberNum as usize].model[0] == 0 {
                    rSaberNum += 1;
                    continue;
                }

                /*
                if (rSaberNum == 0 && (self->client->ps.brokenLimbs & (1 << BROKENLIMB_RARM)))
                { //don't do saber 0 is the right arm is broken
                    rSaberNum++;
                    continue;
                }
                */
                //for now I'm keeping a broken right arm swingable, it will just look and act damaged
                //but still be useable

                if rSaberNum == 1
                    && ((*(*self_).client).ps.brokenLimbs & (1 << BROKENLIMB_LARM)) != 0
                {
                    //don't to saber 1 if the left arm is broken
                    break;
                }
                if rSaberNum > 0
                    && !(*(*self_).client).saber[1].model.is_empty()
                    && (*(*self_).client).saber[1].model[0] != 0
                    && (*(*self_).client).ps.saberHolstered == 1
                {
                    //don't to saber 2 if it's off
                    break;
                }
                rBladeNum = 0;
                while rBladeNum < (*(*self_).client).saber[rSaberNum as usize].numBlades {
                    //update muzzle data for the blade
                    VectorCopy(
                        &(*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .muzzlePoint
                            .clone(),
                        &mut (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .muzzlePointOld,
                    );
                    VectorCopy(
                        &(*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .muzzleDir
                            .clone(),
                        &mut (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .muzzleDirOld,
                    );

                    if rBladeNum > 0 //more than one blade
                        && ((*(*self_).client).saber[1].model.is_empty()
                            || (*(*self_).client).saber[1].model[0] == 0) //not using dual blades
                        && (*(*self_).client).saber[rSaberNum as usize].numBlades > 1 //using a multi-bladed saber
                        && (*(*self_).client).ps.saberHolstered == 1
                    {
                        //don't to extra blades if they're off
                        break;
                    }
                    //get the new data
                    //then update the bolt pos/dir. rBladeNum corresponds to the bolt index because blade bolts are added in order.
                    if rSaberNum == 0 && (*(*self_).client).ps.saberInFlight != QFALSE {
                        if (*(*self_).client).ps.saberEntityNum == 0 {
                            //dropped it... shouldn't get here, but...
                            //assert(0);
                            //FIXME: It's getting here a lot actually....
                            rSaberNum += 1;
                            rBladeNum = 0;
                            continue;
                        } else {
                            let saberEnt = (core::ptr::addr_of_mut!(g_entities)
                                .cast::<gentity_t>())
                            .add((*(*self_).client).ps.saberEntityNum as usize);
                            let mut saberOrg: vec3_t = [0.0; 3];
                            let mut saberAngles: vec3_t = [0.0; 3];
                            if saberEnt.is_null()
                                || (*saberEnt).inuse == QFALSE
                                || (*saberEnt).ghoul2.is_null()
                            {
                                //wtf?
                                rSaberNum += 1;
                                rBladeNum = 0;
                                continue;
                            }
                            if (*saberent).s.saberInFlight != QFALSE {
                                //spinning
                                BG_EvaluateTrajectory(
                                    &(*saberEnt).s.pos,
                                    (*addr_of!(level)).time + 50,
                                    &mut saberOrg,
                                );
                                BG_EvaluateTrajectory(
                                    &(*saberEnt).s.apos,
                                    (*addr_of!(level)).time + 50,
                                    &mut saberAngles,
                                );
                            } else {
                                //coming right back
                                let mut saberDir: vec3_t = [0.0; 3];
                                BG_EvaluateTrajectory(
                                    &(*saberEnt).s.pos,
                                    (*addr_of!(level)).time,
                                    &mut saberOrg,
                                );
                                VectorSubtract(&(*self_).r.currentOrigin, &saberOrg, &mut saberDir);
                                vectoangles(&saberDir, &mut saberAngles);
                            }
                            trap::G2API_GetBoltMatrix(
                                (*saberEnt).ghoul2,
                                0,
                                rBladeNum,
                                &mut boltMatrix,
                                &saberAngles,
                                &saberOrg,
                                (*addr_of!(level)).time,
                                null_mut(),
                                &(*self_).modelScale,
                            );
                            BG_GiveMeVectorFromMatrix(
                                &boltMatrix,
                                ORIGIN,
                                &mut (*(*self_).client).saber[rSaberNum as usize].blade
                                    [rBladeNum as usize]
                                    .muzzlePoint,
                            );
                            BG_GiveMeVectorFromMatrix(
                                &boltMatrix,
                                NEGATIVE_Y,
                                &mut (*(*self_).client).saber[rSaberNum as usize].blade
                                    [rBladeNum as usize]
                                    .muzzleDir,
                            );
                            VectorCopy(
                                &(*(*self_).client).saber[rSaberNum as usize].blade
                                    [rBladeNum as usize]
                                    .muzzlePoint
                                    .clone(),
                                &mut boltOrigin,
                            );
                            VectorMA(
                                &boltOrigin.clone(),
                                (*(*self_).client).saber[rSaberNum as usize].blade
                                    [rBladeNum as usize]
                                    .lengthMax,
                                &(*(*self_).client).saber[rSaberNum as usize].blade
                                    [rBladeNum as usize]
                                    .muzzleDir
                                    .clone(),
                                &mut end,
                            );
                        }
                    } else {
                        trap::G2API_GetBoltMatrix(
                            (*self_).ghoul2,
                            rSaberNum + 1,
                            rBladeNum,
                            &mut boltMatrix,
                            &properAngles,
                            &properOrigin,
                            (*addr_of!(level)).time,
                            null_mut(),
                            &(*self_).modelScale,
                        );
                        BG_GiveMeVectorFromMatrix(
                            &boltMatrix,
                            ORIGIN,
                            &mut (*(*self_).client).saber[rSaberNum as usize].blade
                                [rBladeNum as usize]
                                .muzzlePoint,
                        );
                        BG_GiveMeVectorFromMatrix(
                            &boltMatrix,
                            NEGATIVE_Y,
                            &mut (*(*self_).client).saber[rSaberNum as usize].blade
                                [rBladeNum as usize]
                                .muzzleDir,
                        );
                        VectorCopy(
                            &(*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                .muzzlePoint
                                .clone(),
                            &mut boltOrigin,
                        );
                        VectorMA(
                            &boltOrigin.clone(),
                            (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                .lengthMax,
                            &(*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                                .muzzleDir
                                .clone(),
                            &mut end,
                        );
                    }

                    (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                        .storageTime = (*addr_of!(level)).time;

                    if (*(*self_).client).hasCurrentPosition != QFALSE
                        && (*addr_of!(d_saberInterpolate)).integer != 0
                    {
                        if (*(*self_).client).ps.weaponTime <= 0 {
                            //rww - 07/17/02 - don't bother doing the extra stuff unless actually attacking. This is in attempt to save CPU.
                            CheckSaberDamage(
                                self_,
                                rSaberNum,
                                rBladeNum,
                                &mut boltOrigin,
                                &mut end,
                                QFALSE,
                                MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                                QFALSE,
                            );
                        } else if (*addr_of!(d_saberInterpolate)).integer == 1 {
                            let mut trMask: c_int = CONTENTS_LIGHTSABER | CONTENTS_BODY;
                            let mut sN: c_int = 0;
                            let mut gotHit: qboolean = QFALSE;
                            let mut clientUnlinked: [qboolean; MAX_CLIENTS] = [QFALSE; MAX_CLIENTS];
                            let mut skipSaberTrace: qboolean = QFALSE;

                            if (*addr_of!(g_saberTraceSaberFirst)).integer == 0 {
                                skipSaberTrace = QTRUE;
                            } else if (*addr_of!(g_saberTraceSaberFirst)).integer >= 2
                                && (*addr_of!(g_gametype)).integer != GT_DUEL
                                && (*addr_of!(g_gametype)).integer != GT_POWERDUEL
                                && (*(*self_).client).ps.duelInProgress == QFALSE
                            {
                                //if value is >= 2, and not in a duel, skip
                                skipSaberTrace = QTRUE;
                            }

                            if skipSaberTrace != QFALSE {
                                //skip the saber-contents-only trace and get right to the full trace
                                trMask = MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT;
                            } else {
                                while sN < MAX_CLIENTS as c_int {
                                    let e = (core::ptr::addr_of_mut!(g_entities)
                                        .cast::<gentity_t>())
                                    .add(sN as usize);
                                    if (*e).inuse != QFALSE
                                        && !(*e).client.is_null()
                                        && (*e).r.linked != QFALSE
                                        && (*e).health > 0
                                        && ((*e).r.contents & CONTENTS_BODY) != 0
                                    {
                                        //Take this mask off before the saber trace, because we want to hit the saber first
                                        (*e).r.contents &= !CONTENTS_BODY;
                                        clientUnlinked[sN as usize] = QTRUE;
                                    } else {
                                        clientUnlinked[sN as usize] = QFALSE;
                                    }
                                    sN += 1;
                                }
                            }

                            while gotHit == QFALSE {
                                if CheckSaberDamage(
                                    self_,
                                    rSaberNum,
                                    rBladeNum,
                                    &mut boltOrigin,
                                    &mut end,
                                    QFALSE,
                                    trMask,
                                    QFALSE,
                                ) == QFALSE
                                {
                                    if CheckSaberDamage(
                                        self_,
                                        rSaberNum,
                                        rBladeNum,
                                        &mut boltOrigin,
                                        &mut end,
                                        QTRUE,
                                        trMask,
                                        QFALSE,
                                    ) == QFALSE
                                    {
                                        let mut oldSaberStart: vec3_t = [0.0; 3];
                                        let mut oldSaberEnd: vec3_t = [0.0; 3];
                                        let mut saberAngleNow: vec3_t = [0.0; 3];
                                        let mut saberAngleBefore: vec3_t = [0.0; 3];
                                        let mut saberMidDir: vec3_t = [0.0; 3];
                                        let mut saberMidAngle: vec3_t = [0.0; 3];
                                        let mut saberMidPoint: vec3_t = [0.0; 3];
                                        let mut saberMidEnd: vec3_t = [0.0; 3];
                                        let mut saberSubBase: vec3_t = [0.0; 3];
                                        let deltaX: f32;
                                        let deltaY: f32;
                                        let deltaZ: f32;

                                        if (*addr_of!(level)).time
                                            - (*(*self_).client).saber[rSaberNum as usize].blade
                                                [rBladeNum as usize]
                                                .trail
                                                .lastTime
                                            > 100
                                        {
                                            //no valid last pos, use current
                                            VectorCopy(&boltOrigin, &mut oldSaberStart);
                                            VectorCopy(&end, &mut oldSaberEnd);
                                        } else {
                                            //trace from last pos
                                            VectorCopy(
                                                &(*(*self_).client).saber[rSaberNum as usize].blade
                                                    [rBladeNum as usize]
                                                    .trail
                                                    .base,
                                                &mut oldSaberStart,
                                            );
                                            VectorCopy(
                                                &(*(*self_).client).saber[rSaberNum as usize].blade
                                                    [rBladeNum as usize]
                                                    .trail
                                                    .tip,
                                                &mut oldSaberEnd,
                                            );
                                        }

                                        VectorSubtract(
                                            &oldSaberEnd,
                                            &oldSaberStart,
                                            &mut saberAngleBefore,
                                        );
                                        vectoangles(
                                            &saberAngleBefore.clone(),
                                            &mut saberAngleBefore,
                                        );

                                        VectorSubtract(&end, &boltOrigin, &mut saberAngleNow);
                                        vectoangles(&saberAngleNow.clone(), &mut saberAngleNow);

                                        deltaX = AngleDelta(saberAngleBefore[0], saberAngleNow[0]);
                                        deltaY = AngleDelta(saberAngleBefore[1], saberAngleNow[1]);
                                        deltaZ = AngleDelta(saberAngleBefore[2], saberAngleNow[2]);

                                        if (deltaX != 0.0 || deltaY != 0.0 || deltaZ != 0.0)
                                            && deltaX < 180.0
                                            && deltaY < 180.0
                                            && deltaZ < 180.0
                                            && (BG_SaberInAttack((*(*self_).client).ps.saberMove)
                                                != 0
                                                || PM_SaberInTransition(
                                                    (*(*self_).client).ps.saberMove,
                                                ) != 0)
                                        {
                                            //don't go beyond here if we aren't attacking/transitioning or the angle is too large.
                                            //and don't bother if the angle is the same
                                            saberMidAngle[0] = saberAngleBefore[0] + (deltaX / 2.0);
                                            saberMidAngle[1] = saberAngleBefore[1] + (deltaY / 2.0);
                                            saberMidAngle[2] = saberAngleBefore[2] + (deltaZ / 2.0);

                                            //Now that I have the angle, I'll just say the base for it is the difference between the two start
                                            //points (even though that's quite possibly completely false)
                                            VectorSubtract(
                                                &boltOrigin,
                                                &oldSaberStart,
                                                &mut saberSubBase,
                                            );
                                            saberMidPoint[0] =
                                                boltOrigin[0] + (saberSubBase[0] * 0.5);
                                            saberMidPoint[1] =
                                                boltOrigin[1] + (saberSubBase[1] * 0.5);
                                            saberMidPoint[2] =
                                                boltOrigin[2] + (saberSubBase[2] * 0.5);

                                            AngleVectors(
                                                &saberMidAngle,
                                                Some(&mut saberMidDir),
                                                None,
                                                None,
                                            );
                                            saberMidEnd[0] = saberMidPoint[0]
                                                + saberMidDir[0]
                                                    * (*(*self_).client).saber[rSaberNum as usize]
                                                        .blade
                                                        [rBladeNum as usize]
                                                        .lengthMax;
                                            saberMidEnd[1] = saberMidPoint[1]
                                                + saberMidDir[1]
                                                    * (*(*self_).client).saber[rSaberNum as usize]
                                                        .blade
                                                        [rBladeNum as usize]
                                                        .lengthMax;
                                            saberMidEnd[2] = saberMidPoint[2]
                                                + saberMidDir[2]
                                                    * (*(*self_).client).saber[rSaberNum as usize]
                                                        .blade
                                                        [rBladeNum as usize]
                                                        .lengthMax;

                                            //I'll just trace straight out and not even trace between positions to save speed.
                                            if CheckSaberDamage(
                                                self_,
                                                rSaberNum,
                                                rBladeNum,
                                                &mut saberMidPoint,
                                                &mut saberMidEnd,
                                                QFALSE,
                                                trMask,
                                                QFALSE,
                                            ) != QFALSE
                                            {
                                                gotHit = QTRUE;
                                            }
                                        }
                                    } else {
                                        gotHit = QTRUE;
                                    }
                                } else {
                                    gotHit = QTRUE;
                                }

                                if (*addr_of!(g_saberTraceSaberFirst)).integer != 0 {
                                    sN = 0;
                                    while sN < MAX_CLIENTS as c_int {
                                        if clientUnlinked[sN as usize] != QFALSE {
                                            //Make clients clip properly again.
                                            let e = (core::ptr::addr_of_mut!(g_entities)
                                                .cast::<gentity_t>())
                                            .add(sN as usize);
                                            if (*e).inuse != QFALSE && (*e).health > 0 {
                                                (*e).r.contents |= CONTENTS_BODY;
                                            }
                                        }
                                        sN += 1;
                                    }
                                }

                                if gotHit == QFALSE {
                                    if trMask
                                        != (MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT)
                                    {
                                        trMask = MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT;
                                    } else {
                                        gotHit = QTRUE; //break out of the loop
                                    }
                                }
                            }
                        } else if (*addr_of!(d_saberInterpolate)).integer != 0 {
                            //anything but 0 or 1, use the old plain method.
                            if CheckSaberDamage(
                                self_,
                                rSaberNum,
                                rBladeNum,
                                &mut boltOrigin,
                                &mut end,
                                QFALSE,
                                MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                                QFALSE,
                            ) == QFALSE
                            {
                                CheckSaberDamage(
                                    self_,
                                    rSaberNum,
                                    rBladeNum,
                                    &mut boltOrigin,
                                    &mut end,
                                    QTRUE,
                                    MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                                    QFALSE,
                                );
                            }
                        }
                    } else if (*addr_of!(d_saberSPStyleDamage)).integer != 0 {
                        G_SPSaberDamageTraceLerped(
                            self_,
                            rSaberNum,
                            rBladeNum,
                            &mut boltOrigin,
                            &mut end,
                            MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                        );
                    } else {
                        CheckSaberDamage(
                            self_,
                            rSaberNum,
                            rBladeNum,
                            &mut boltOrigin,
                            &mut end,
                            QFALSE,
                            MASK_PLAYERSOLID | CONTENTS_LIGHTSABER | MASK_SHOT,
                            QFALSE,
                        );
                    }

                    // ClientManager::ActiveClientNum() -> 0 (MP)
                    VectorCopy(
                        &boltOrigin,
                        &mut (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .base,
                    );
                    VectorCopy(
                        &end,
                        &mut (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                            .trail
                            .tip,
                    );
                    (*(*self_).client).saber[rSaberNum as usize].blade[rBladeNum as usize]
                        .trail
                        .lastTime = (*addr_of!(level)).time;
                    //VectorCopy(boltOrigin, self->client->lastSaberBase);
                    //VectorCopy(end, self->client->lastSaberTip);
                    (*(*self_).client).hasCurrentPosition = QTRUE;

                    //do hit effects
                    WP_SaberDoHit(self_, rSaberNum, rBladeNum);
                    WP_SaberDoClash(self_, rSaberNum, rBladeNum);

                    rBladeNum += 1;
                }

                rSaberNum += 1;
            }

            WP_SaberApplyDamage(self_);
            //NOTE: doing one call like this after the 2 loops above is a bit cheaper, tempentity-wise... but won't use the correct saber and blade numbers...
            //now actually go through and apply all the damage we did
            //WP_SaberDoHit( self, 0, 0 );
            //WP_SaberDoClash( self, 0, 0 );

            if !mySaber.is_null() && (*mySaber).inuse != QFALSE {
                trap::LinkEntity(mySaber);
            }

            if (*(*self_).client).ps.saberInFlight == QFALSE {
                (*(*self_).client).ps.saberEntityState = 0;
            }
        }
    }
    //finalUpdate:
    if clientOverride != QFALSE {
        //if we get the client instance we don't even need to bother setting anims and stuff
        return;
    }

    G_UpdateClientAnims(self_, animSpeedScale);
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    #[test]
    fn RandFloat_matches_oracle() {
        // RandFloat draws from the game LCG (bg_lib::rand / oracle jka_rand) — the same
        // process-global seed bg_lib's own RNG tests use; take the lock and re-seed both
        // sides in lockstep so each consumes the same rand() value, then assert bit-exact.
        let _guard = bg_lib::rand_lock();
        let ranges = [
            (0.0f32, 1.0f32),
            (-0.2, 0.2),
            (-1.0, 1.0),
            (10.0, 20.0),
            (-50.0, -10.0),
            (0.0, 0.0),
            (100.0, 0.0), // min > max: faithful to the C, no clamp
        ];
        for seed in [0u32, 1, 42, 69069, 0x8000_0000, 0xffff_ffff] {
            bg_lib::srand(seed);
            unsafe { oracle::jka_srand(seed) };
            for i in 0..50_000 {
                let (min, max) = ranges[i % ranges.len()];
                let r = RandFloat(min, max);
                let o = unsafe { oracle::jka_RandFloat(min, max) };
                assert_eq!(
                    r.to_bits(),
                    o.to_bits(),
                    "RandFloat({min},{max}) seed={seed:#x} iter={i}"
                );
            }
        }
    }

    #[test]
    fn G_SaberAttackPower_matches_oracle() {
        use crate::codemp::game::bg_public::{DUELTEAM_FREE, DUELTEAM_LONE};
        use crate::codemp::game::g_local::{gclient_t, gentity_t};
        use crate::codemp::game::g_main::level_lock;

        // This test mutates the shared `level`/`g_gametype` statics; take the crate-wide lock
        // so it serializes with the g_combat/g_items tests that read them back.
        let _g = level_lock();

        // Build a minimal zeroed client+entity, set just the fields the body reads, and
        // compare to the pass-the-read-fields oracle fed the same scalars.
        let styles = [
            0, SS_FAST, SS_MEDIUM, SS_STRONG, 4, SS_TAVION, SS_DUAL, SS_STAFF,
        ];
        // Saber-base pairs chosen so the swing distance straddles the per-stance tolerances
        // (8/16/24): zero, tiny, ~one-tolerance, several-tolerances, and an odd diagonal.
        let base_pairs: [([f32; 3], [f32; 3]); 5] = [
            ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            ([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            ([20.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            ([100.0, 50.0, -25.0], [10.0, -10.0, 5.0]),
            ([-3.5, 7.25, 0.125], [1.0, 2.0, 3.0]),
        ];
        let gametypes = [GT_DUEL, GT_POWERDUEL, GT_SIEGE, GT_JEDIMASTER];
        let duel_teams = [DUELTEAM_FREE, DUELTEAM_LONE];
        // 0=none, both arms, just R, just L.
        let broken = [
            0,
            (1 << BROKENLIMB_RARM) | (1 << BROKENLIMB_LARM),
            1 << BROKENLIMB_RARM,
            1 << BROKENLIMB_LARM,
        ];
        let level_time = 100_000;
        // storageTime relative to level.time-50: stale (skip block) and fresh (enter block).
        let storage_times = [level_time - 1000, level_time - 50, level_time];

        for &style in &styles {
            for &attacking in &[QFALSE, QTRUE] {
                for &(ref last_base, ref older_base) in &base_pairs {
                    for &gt in &gametypes {
                        for &dt in &duel_teams {
                            for &brk in &broken {
                                for &valid in &[QFALSE, QTRUE] {
                                    for &stime in &storage_times {
                                        unsafe {
                                            let mut client: gclient_t = core::mem::zeroed();
                                            client.ps.fd.saberAnimLevel = style;
                                            client.lastSaberStorageTime = stime;
                                            client.olderIsValid = valid;
                                            client.lastSaberBase_Always = *last_base;
                                            client.olderSaberBase = *older_base;
                                            client.ps.brokenLimbs = brk;
                                            client.sess.duelTeam = dt;

                                            let mut ent: gentity_t = core::mem::zeroed();
                                            ent.client = &mut client;

                                            (*core::ptr::addr_of_mut!(level)).time = level_time;
                                            g_gametype.integer = gt;

                                            let r = G_SaberAttackPower(&mut ent, attacking);
                                            let o = oracle::jka_G_SaberAttackPower(
                                                style,
                                                attacking as c_int,
                                                stime,
                                                valid as c_int,
                                                last_base.as_ptr(),
                                                older_base.as_ptr(),
                                                brk,
                                                level_time,
                                                gt,
                                                dt,
                                            );
                                            assert_eq!(
                                                r, o,
                                                "G_SaberAttackPower style={style} att={attacking:?} \
                                                 last={last_base:?} older={older_base:?} gt={gt} \
                                                 dt={dt} brk={brk} valid={valid:?} stime={stime}"
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
    }

    #[test]
    fn HasSetSaberOnly_matches_oracle() {
        use crate::codemp::game::bg_public::{GT_FFA, GT_HOLOCRON, GT_SIEGE, GT_TEAM};

        // Set the cvar globals, call the port, compare to the oracle fed the same ints.
        let gametypes = [
            GT_FFA,
            GT_HOLOCRON,
            GT_JEDIMASTER,
            GT_DUEL,
            GT_POWERDUEL,
            GT_TEAM,
            GT_SIEGE,
        ];
        // Cover: no weapons disabled, all disabled, saber-only bit patterns, and arbitrary masks.
        let masks = [
            0,
            -1,
            !(1 << WP_SABER) & !(1 << WP_NONE) & ((1 << WP_NUM_WEAPONS) - 1), // every weapon but saber/none disabled
            (1 << WP_SABER) | (1 << WP_NONE), // only saber+none disabled (others enabled)
            0x5555_5555,
            0xAAAA_AAAA_u32 as i32,
            1 << 4,
            (1 << WP_NUM_WEAPONS) - 1,
        ];
        for gt in gametypes {
            for &dwd in &masks {
                for &wd in &masks {
                    unsafe {
                        g_gametype.integer = gt;
                        g_duelWeaponDisable.integer = dwd;
                        g_weaponDisable.integer = wd;
                        let r = HasSetSaberOnly();
                        let o = oracle::jka_HasSetSaberOnly(gt, dwd, wd);
                        assert_eq!(
                            r as i32, o,
                            "HasSetSaberOnly gt={gt} dwd={dwd:#x} wd={wd:#x}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn VectorCompare2_matches_oracle() {
        // Vectors chosen to straddle the 0.0001 epsilon on each axis (inside, on the
        // boundary, and well outside), plus equal/zero cases.
        let bases: [vec3_t; 4] = [
            [0.0, 0.0, 0.0],
            [1.5, -2.25, 100.0],
            [-1000.0, 0.00005, 12345.678],
            [3.0, 3.0, 3.0],
        ];
        let deltas = [
            0.0f32, 0.00005, 0.0001, 0.00011, -0.00005, -0.0001, -0.00011, 0.5, -0.5,
        ];
        for base in bases {
            for &dx in &deltas {
                for &dy in &deltas {
                    for &dz in &deltas {
                        let other: vec3_t = [base[0] + dx, base[1] + dy, base[2] + dz];
                        let r = VectorCompare2(&base, &other);
                        let o =
                            unsafe { oracle::jka_VectorCompare2(base.as_ptr(), other.as_ptr()) };
                        assert_eq!(r, o, "VectorCompare2 base={base:?} other={other:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn WPDEBUG_SaberColor_matches_oracle() {
        // Every defined color plus a couple out-of-range values for the default branch.
        for c in -2..=8 {
            let r = WPDEBUG_SaberColor(c);
            let o = unsafe { oracle::jka_WPDEBUG_SaberColor(c) };
            assert_eq!(r, o, "WPDEBUG_SaberColor({c})");
        }
    }

    #[test]
    fn G_PrettyCloseIGuess_matches_oracle() {
        let vals = [-100.0f32, -1.0, -0.001, 0.0, 0.001, 1.0, 4.0, 100.0];
        let tols = [0.0f32, 0.001, 1.0, 4.0, 1000.0];
        for &a in &vals {
            for &b in &vals {
                for &tol in &tols {
                    let r = G_PrettyCloseIGuess(a, b, tol);
                    let o = unsafe { oracle::jka_G_PrettyCloseIGuess(a, b, tol) };
                    assert_eq!(r as i32, o, "G_PrettyCloseIGuess({a},{b},{tol})");
                }
            }
        }
    }

    #[test]
    fn WP_SaberBladeLength_matches_oracle() {
        use crate::codemp::game::q_shared_h::MAX_BLADES;
        // A range of blade-length layouts: empty, single, ascending, descending, ties,
        // negatives (faithful — no clamp), and a full MAX_BLADES array.
        let lens: [f32; MAX_BLADES] = [10.0, 32.0, 5.0, 32.0, -3.0, 0.0, 48.5, 1.0];
        for num_blades in 0..=(MAX_BLADES as c_int) {
            // Build a saberInfo_t with these blade lengths and numBlades.
            let mut saber: saberInfo_t = unsafe { core::mem::zeroed() };
            saber.numBlades = num_blades;
            for i in 0..MAX_BLADES {
                saber.blade[i].lengthMax = lens[i];
            }
            let r = unsafe { WP_SaberBladeLength(&saber) };
            let o = unsafe { oracle::jka_WP_SaberBladeLength(lens.as_ptr(), num_blades) };
            assert_eq!(
                r.to_bits(),
                o.to_bits(),
                "WP_SaberBladeLength numBlades={num_blades}"
            );
        }
    }

    #[test]
    fn G_SaberInBackAttack_matches_oracle() {
        // Cover the three back-attack moves plus neighbours and out-of-range values.
        for m in -2..=20 {
            let r = G_SaberInBackAttack(m);
            let o = unsafe { oracle::jka_G_SaberInBackAttack(m) };
            assert_eq!(r as i32, o, "G_SaberInBackAttack({m})");
        }
    }

    #[test]
    fn G_KnockawayForParry_matches_oracle() {
        // Cover the five parry moves plus neighbours and out-of-range values; everything
        // outside the four explicit parries (incl. LS_PARRY_UR) hits the default = LS_K1_TR.
        for m in 140..=170 {
            let r = G_KnockawayForParry(m);
            let o = unsafe { oracle::jka_G_KnockawayForParry(m) };
            assert_eq!(r, o, "G_KnockawayForParry({m})");
        }
        for m in [-2, -1, 0, 1, 100, 200] {
            let r = G_KnockawayForParry(m);
            let o = unsafe { oracle::jka_G_KnockawayForParry(m) };
            assert_eq!(r, o, "G_KnockawayForParry({m})");
        }
    }

    #[test]
    fn G_SaberLockAnim_matches_oracle() {
        // Exhaustively sweep all five int args across the relevant saber_styles_t range
        // (incl. out-of-range, which falls through to the "single" default arms) and every
        // SABERLOCK_* phase/side/win value, comparing the port to the oracle bit-for-bit.
        for att in 0..=8 {
            for def in 0..=8 {
                for tos in 0..=2 {
                    for phase in 0..=5 {
                        for wol in 0..=6 {
                            let r = G_SaberLockAnim(att, def, tos, phase, wol);
                            let o =
                                unsafe { oracle::jka_G_SaberLockAnim(att, def, tos, phase, wol) };
                            assert_eq!(r, o, "G_SaberLockAnim({att},{def},{tos},{phase},{wol})");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn WP_MissileBlockForBlock_matches_oracle() {
        // Every saberBlockedType_t value (-2..=15) — the five directional blocks map to
        // their *_PROJ variants, everything else passes through.
        for b in -2..=15 {
            let r = WP_MissileBlockForBlock(b);
            let o = unsafe { oracle::jka_WP_MissileBlockForBlock(b) };
            assert_eq!(r, o, "WP_MissileBlockForBlock({b})");
        }
    }

    #[test]
    fn G_GetParryForBlock_matches_oracle() {
        // Every saberBlockedType_t value (-2..=15) — the ten directional/proj blocks map to
        // their LS_PARRY_*/LS_REFLECT_* moves, everything else yields LS_NONE.
        for b in -2..=15 {
            let r = G_GetParryForBlock(b);
            let o = unsafe { oracle::jka_G_GetParryForBlock(b) };
            assert_eq!(r, o, "G_GetParryForBlock({b})");
        }
    }

    #[test]
    fn G_SabCol_CalcPlaneEq_matches_oracle() {
        // A spread of triangle vertices: degenerate (collinear/coincident), axis-aligned,
        // and arbitrary, to exercise every cofactor term bit-for-bit.
        let tris: [[vec3_t; 3]; 6] = [
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]], // collinear -> zero normal
            [[-12.5, 3.25, 100.0], [7.0, -8.0, 0.5], [0.0, 0.0, -42.0]],
            [
                [1000.0, -1000.0, 0.0],
                [0.001, 0.002, 0.003],
                [-5.5, 5.5, 5.5],
            ],
            [[3.0, 3.0, 3.0], [3.0, 3.0, 3.0], [3.0, 3.0, 3.0]], // all coincident
        ];
        for t in tris {
            let mut planeEq = [0.0f32; 4];
            let mut o_planeEq = [0.0f32; 4];
            G_SabCol_CalcPlaneEq(&t[0], &t[1], &t[2], &mut planeEq);
            unsafe {
                oracle::jka_G_SabCol_CalcPlaneEq(
                    t[0].as_ptr(),
                    t[1].as_ptr(),
                    t[2].as_ptr(),
                    o_planeEq.as_mut_ptr(),
                );
            }
            for i in 0..4 {
                assert_eq!(
                    planeEq[i].to_bits(),
                    o_planeEq[i].to_bits(),
                    "G_SabCol_CalcPlaneEq tri={t:?} planeEq[{i}]"
                );
            }
        }
    }

    #[test]
    fn G_SabCol_PointRelativeToPlane_matches_oracle() {
        // Several plane equations crossed with points on either side and exactly on the plane.
        let planes: [[f32; 4]; 4] = [
            [0.0, 0.0, 1.0, 0.0],    // z = 0 plane
            [1.0, 1.0, 1.0, -3.0],   // x+y+z = 3
            [-2.5, 0.5, 7.0, 12.34], // arbitrary
            [0.0, 0.0, 0.0, 0.0],    // degenerate -> side always 0
        ];
        let points: [vec3_t; 7] = [
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [-5.0, 2.0, 3.0],
            [0.0, 0.0, 1.0],
            [100.0, -100.0, 50.0],
            [0.001, 0.002, 0.003],
            [3.0, 0.0, 0.0],
        ];
        for plane in planes {
            for pos in points {
                let mut side = 0.0f32;
                let mut o_side = 0.0f32;
                let r = G_SabCol_PointRelativeToPlane(&pos, &mut side, &plane);
                let o = unsafe {
                    oracle::jka_G_SabCol_PointRelativeToPlane(
                        pos.as_ptr(),
                        &mut o_side,
                        plane.as_ptr(),
                    )
                };
                assert_eq!(
                    r, o,
                    "G_SabCol_PointRelativeToPlane plane={plane:?} pos={pos:?} ret"
                );
                assert_eq!(
                    side.to_bits(),
                    o_side.to_bits(),
                    "G_SabCol_PointRelativeToPlane plane={plane:?} pos={pos:?} side"
                );
            }
        }
    }

    // A spread of (base, tip, radius, fwd, right) inputs for the hull-builder / face-collision
    // oracle tests: axis-aligned blade, an off-axis blade, a tiny/zero-radius case, and a fully
    // arbitrary diagonal blade with a non-orthonormal basis (faithful — the C does no checks).
    const SABERFACE_CASES: [([f32; 3], [f32; 3], f32, [f32; 3], [f32; 3]); 4] = [
        (
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 40.0],
            6.0,
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ),
        (
            [10.0, -5.0, 2.0],
            [10.0, -5.0, 42.0],
            9.0,
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ),
        (
            [1.0, 2.0, 3.0],
            [1.5, 2.5, 5.0],
            0.0,
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
        ),
        (
            [-3.5, 7.25, 0.125],
            [12.0, -4.0, 30.0],
            13.0,
            [0.3, -0.6, 0.7],
            [0.8, 0.1, -0.2],
        ),
    ];

    #[test]
    fn G_BuildSaberFaces_matches_oracle() {
        for (base, tip, radius, fwd, right) in SABERFACE_CASES {
            let mut r_fnum: c_int = 0;
            let mut r_flist: *mut saberFace_t = core::ptr::null_mut();
            let mut o_faces = [oracle::OracleSaberFace {
                v1: [0.0; 3],
                v2: [0.0; 3],
                v3: [0.0; 3],
            }; 12];
            let (o_fnum, r_faces) = unsafe {
                G_BuildSaberFaces(&base, &tip, radius, &fwd, &right, &mut r_fnum, &mut r_flist);
                let o_fnum = oracle::jka_G_BuildSaberFaces(
                    base.as_ptr(),
                    tip.as_ptr(),
                    radius,
                    fwd.as_ptr(),
                    right.as_ptr(),
                    o_faces.as_mut_ptr(),
                );
                (
                    o_fnum,
                    core::slice::from_raw_parts(r_flist, r_fnum as usize),
                )
            };
            assert_eq!(r_fnum, o_fnum, "G_BuildSaberFaces fNum base={base:?}");
            for f in 0..(r_fnum as usize) {
                for (lbl, rv, ov) in [
                    ("v1", &r_faces[f].v1, &o_faces[f].v1),
                    ("v2", &r_faces[f].v2, &o_faces[f].v2),
                    ("v3", &r_faces[f].v3, &o_faces[f].v3),
                ] {
                    for c in 0..3 {
                        assert_eq!(
                            rv[c].to_bits(),
                            ov[c].to_bits(),
                            "G_BuildSaberFaces base={base:?} face={f} {lbl}[{c}]"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn G_SaberFaceCollisionCheck_matches_oracle() {
        // Build the hull C-side (parity already proven above), then run the collision check on
        // both sides over a spread of attack segments / boxes — hits through the hull, near
        // misses, the zero-box (widened to +-1) path, and an offset bounding box.
        let segments: [([f32; 3], [f32; 3]); 6] = [
            ([-50.0, 0.0, 20.0], [50.0, 0.0, 20.0]), // straight through, mid-blade
            ([0.0, -50.0, 5.0], [0.0, 50.0, 5.0]),   // through near base
            ([-50.0, 0.0, 200.0], [50.0, 0.0, 200.0]), // well above the tip -> miss
            ([100.0, 100.0, 100.0], [101.0, 101.0, 101.0]), // far away -> miss
            ([-20.0, -20.0, 35.0], [20.0, 20.0, 5.0]), // diagonal across the hull
            ([5.0, 5.0, -10.0], [5.0, 5.0, 60.0]),   // vertical, alongside the blade
        ];
        let boxes: [([f32; 3], [f32; 3]); 3] = [
            ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]), // zero box -> widened to +-1
            ([-2.0, -2.0, -2.0], [2.0, 2.0, 2.0]), // symmetric box
            ([-1.0, -3.0, -0.5], [4.0, 1.0, 2.0]), // asymmetric/offset box
        ];
        for (base, tip, radius, fwd, right) in SABERFACE_CASES {
            for (start, end) in segments {
                for (mins, maxs) in boxes {
                    unsafe {
                        // Rust hull -> Rust check.
                        let mut r_fnum: c_int = 0;
                        let mut r_flist: *mut saberFace_t = core::ptr::null_mut();
                        G_BuildSaberFaces(
                            &base,
                            &tip,
                            radius,
                            &fwd,
                            &right,
                            &mut r_fnum,
                            &mut r_flist,
                        );
                        let mut r_mins = mins;
                        let mut r_maxs = maxs;
                        let mut r_impact: vec3_t = [0.0; 3];
                        let r = G_SaberFaceCollisionCheck(
                            r_fnum,
                            r_flist,
                            &start,
                            &end,
                            &mut r_mins,
                            &mut r_maxs,
                            &mut r_impact,
                        );

                        // Oracle hull -> oracle check.
                        let mut o_faces = [oracle::OracleSaberFace {
                            v1: [0.0; 3],
                            v2: [0.0; 3],
                            v3: [0.0; 3],
                        }; 12];
                        let o_fnum = oracle::jka_G_BuildSaberFaces(
                            base.as_ptr(),
                            tip.as_ptr(),
                            radius,
                            fwd.as_ptr(),
                            right.as_ptr(),
                            o_faces.as_mut_ptr(),
                        );
                        let mut o_mins = mins;
                        let mut o_maxs = maxs;
                        let mut o_impact: vec3_t = [0.0; 3];
                        let o = oracle::jka_G_SaberFaceCollisionCheck(
                            o_fnum,
                            o_faces.as_mut_ptr(),
                            start.as_ptr(),
                            end.as_ptr(),
                            o_mins.as_mut_ptr(),
                            o_maxs.as_mut_ptr(),
                            o_impact.as_mut_ptr(),
                        );

                        assert_eq!(
                            r as i32, o,
                            "G_SaberFaceCollisionCheck base={base:?} start={start:?} end={end:?} box=({mins:?},{maxs:?}) ret"
                        );
                        // atkMins/atkMaxs are widened in place on the zero-box path; verify parity.
                        for c in 0..3 {
                            assert_eq!(r_mins[c].to_bits(), o_mins[c].to_bits(), "atkMins[{c}]");
                            assert_eq!(r_maxs[c].to_bits(), o_maxs[c].to_bits(), "atkMaxs[{c}]");
                        }
                        // impactPoint only meaningful on a hit.
                        if r != QFALSE {
                            for c in 0..3 {
                                assert_eq!(
                                    r_impact[c].to_bits(),
                                    o_impact[c].to_bits(),
                                    "G_SaberFaceCollisionCheck impactPoint[{c}] start={start:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn G_PowerLevelForSaberAnim_matches_oracle() {
        use crate::codemp::game::anims::MAX_TOTALANIMATIONS;
        use crate::codemp::game::bg_panimate::bgAllAnims;
        use crate::codemp::game::bg_pmove::pm_lock;
        use crate::codemp::game::bg_public::animation_t;
        use crate::codemp::game::g_local::{gclient_t, gentity_t};
        use core::ptr::{addr_of, addr_of_mut};

        // The Rust side reads ps.torsoAnim/torsoTimer, derives animTimeElapsed via the real
        // BG_AnimLength over bgAllAnims[localAnimIndex], and reads saber[saberNum].type. The
        // pass-the-read-fields oracle takes those derived scalars. We drive BG_AnimLength by
        // pointing bgAllAnims[0].anims at a backing table and setting the per-anim
        // numFrames/frameLerp so the resulting length straddles the per-anim time thresholds.
        // anims.rs and anims.h assign different absolute enum values, so each anim is a
        // (rust_const, anims.h_value) pair: the Rust constant drives torsoAnim/the backing-table
        // index, the anims.h value is fed to the oracle's matching #define-mirrored branch.
        let _g = pm_lock();

        // (rust anims.rs constant, anims.h numeric value). Covers every contiguous-range
        // boundary and every switch case (incl. each member of a shared case label).
        let anims: &[(animNumber_t, c_int)] = &[
            // contiguous range-block boundaries (style mappers)
            (BOTH_A1_T__B_, 117),
            (BOTH_D1_B____, 193),
            (BOTH_A2_T__B_, 194),
            (BOTH_D2_B____, 270),
            (BOTH_A3_T__B_, 271),
            (BOTH_D3_B____, 347),
            (BOTH_A4_T__B_, 348),
            (BOTH_D4_B____, 424),
            (BOTH_A5_T__B_, 425),
            (BOTH_D5_B____, 501),
            (BOTH_A6_T__B_, 502),
            (BOTH_D6_B____, 578),
            (BOTH_A7_T__B_, 579),
            (BOTH_D7_B____, 655),
            (BOTH_P1_S1_T_, 656),
            (BOTH_H1_S1_BR, 680),
            // switch cases
            (BOTH_A2_STABBACK1, 845),
            (BOTH_ATTACK_BACK, 846),
            (BOTH_CROUCHATTACKBACK1, 851),
            (BOTH_BUTTERFLY_LEFT, 1200),
            (BOTH_BUTTERFLY_RIGHT, 1201),
            (BOTH_BUTTERFLY_FL1, 1250),
            (BOTH_BUTTERFLY_FR1, 1249),
            (BOTH_FJSS_TR_BL, 1243),
            (BOTH_FJSS_TL_BR, 1244),
            (BOTH_K1_S1_T_, 661),
            (BOTH_K1_S1_TR, 662),
            (BOTH_K1_S1_TL, 663),
            (BOTH_K1_S1_BL, 664),
            (BOTH_K1_S1_B_, 665),
            (BOTH_K1_S1_BR, 666),
            (BOTH_LUNGE2_B__T_, 850),
            (BOTH_FORCELEAP2_T__B_, 849),
            (BOTH_VS_ATR_S, 1040),
            (BOTH_VS_ATL_S, 1039),
            (BOTH_VT_ATR_S, 1078),
            (BOTH_VT_ATL_S, 1077),
            (BOTH_JUMPFLIPSLASHDOWN1, 847),
            (BOTH_JUMPFLIPSTABDOWN, 848),
            (BOTH_JUMPATTACK6, 852),
            (BOTH_JUMPATTACK7, 853),
            (BOTH_SPINATTACK6, 854),
            (BOTH_SPINATTACK7, 855),
            (BOTH_FORCELONGLEAP_ATTACK, 861),
            (BOTH_STABDOWN, 897),
            (BOTH_STABDOWN_STAFF, 898),
            (BOTH_STABDOWN_DUAL, 899),
            (BOTH_A6_SABERPROTECT, 900),
            (BOTH_A7_SOULCAL, 901),
            (BOTH_A1_SPECIAL, 902),
            (BOTH_A2_SPECIAL, 903),
            (BOTH_A3_SPECIAL, 904),
            (BOTH_FLIP_ATTACK7, 890),
            (BOTH_PULL_IMPALE_STAB, 893),
            (BOTH_PULL_IMPALE_SWING, 894),
            (BOTH_ALORA_SPIN_SLASH, 1264),
            (BOTH_A6_FB, 1255),
            (BOTH_A6_LR, 1256),
            (BOTH_A7_HILT, 1257),
            (BOTH_LK_S_DL_T_SB_1_W, 740),
            (BOTH_LK_S_ST_S_SB_1_W, 745),
            (BOTH_LK_S_DL_S_SB_1_W, 735),
            (BOTH_LK_S_S_S_SB_1_W, 755),
            (BOTH_LK_S_ST_T_SB_1_W, 750),
            (BOTH_LK_S_S_T_SB_1_W, 760),
            (BOTH_LK_DL_DL_T_SB_1_W, 770),
            (BOTH_LK_DL_DL_S_SB_1_W, 765),
            (BOTH_LK_DL_ST_S_SB_1_W, 775),
            (BOTH_LK_DL_ST_T_SB_1_W, 780),
            (BOTH_LK_DL_S_S_SB_1_W, 785),
            (BOTH_LK_DL_S_T_SB_1_W, 790),
            (BOTH_LK_ST_DL_S_SB_1_W, 795),
            (BOTH_LK_ST_DL_T_SB_1_W, 800),
            (BOTH_LK_ST_ST_S_SB_1_W, 805),
            (BOTH_LK_ST_S_S_SB_1_W, 815),
            (BOTH_LK_ST_ST_T_SB_1_W, 810),
            (BOTH_LK_ST_S_T_SB_1_W, 820),
            (BOTH_HANG_ATTACK, 1294),
            (BOTH_ROLL_STAB, 905),
            // an anim hitting none of the above (default -> FORCE_LEVEL_0). 0 is a valid
            // index in both enums and falls in no range/case.
            (0, 0),
        ];

        // numFrames/frameLerp pairs giving a spread of BG_AnimLength values: 0, small, and
        // large enough to cross the largest threshold (1450). frameLerp negative exercises the
        // fabs in BG_AnimLength.
        let len_setup: &[(u16, i16)] = &[(0, 100), (5, 100), (10, 50), (20, 100), (30, -50)];
        // animTimer values straddling every threshold used by the body (150..1450).
        let timers = [
            -1, 0, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 800, 850, 900,
            950, 1000, 1100, 1200, 1300, 1450, 2000,
        ];
        let saber_types = [0, SABER_LANCE, SABER_TRIDENT, 1, 5];
        let saber_nums = [0, 1];

        let mut backing: Vec<animation_t> =
            vec![animation_t::default(); MAX_TOTALANIMATIONS as usize];
        unsafe {
            let saved = (*addr_of!(bgAllAnims))[0].anims;
            (*addr_of_mut!(bgAllAnims))[0].anims = backing.as_mut_ptr();

            let mut client: gclient_t = core::mem::zeroed();
            let mut ent: gentity_t = core::mem::zeroed();
            ent.client = &mut client;
            ent.localAnimIndex = 0;

            for &(r_anim, c_anim) in anims {
                for &(nf, fl) in len_setup {
                    backing[r_anim as usize].numFrames = nf;
                    backing[r_anim as usize].frameLerp = fl;
                    // The real length the Rust side will derive.
                    let anim_len = BG_AnimLength(0, r_anim);
                    for &timer in &timers {
                        for &st in &saber_types {
                            for &sn in &saber_nums {
                                for &hit in &[QFALSE, QTRUE] {
                                    client.ps.torsoAnim = r_anim;
                                    client.ps.torsoTimer = timer;
                                    client.saber[sn as usize].r#type = st;

                                    let got = G_PowerLevelForSaberAnim(&mut ent, sn, hit);
                                    let want = oracle::jka_G_PowerLevelForSaberAnim(
                                        c_anim,
                                        timer,
                                        anim_len - timer,
                                        st,
                                        sn,
                                        hit as c_int,
                                    );
                                    assert_eq!(
                                        got, want,
                                        "anim=({r_anim},{c_anim}) nf={nf} fl={fl} timer={timer} \
                                         st={st} sn={sn} hit={hit}"
                                    );
                                }
                            }
                        }
                    }
                    // reset this anim's table row so it doesn't bleed into the next anim's
                    // out-of-its-own-row state (paranoia; each iter sets its own row anyway).
                    backing[r_anim as usize].numFrames = 0;
                    backing[r_anim as usize].frameLerp = 100;
                }
            }

            // Guard paths: null client and out-of-range saberNum -> FORCE_LEVEL_0 (Rust-side).
            let mut bad: gentity_t = core::mem::zeroed();
            bad.client = null_mut();
            assert_eq!(G_PowerLevelForSaberAnim(&mut bad, 0, QFALSE), FORCE_LEVEL_0);
            assert_eq!(
                G_PowerLevelForSaberAnim(&mut ent, MAX_SABERS as c_int, QFALSE),
                FORCE_LEVEL_0
            );

            (*addr_of_mut!(bgAllAnims))[0].anims = saved;
        }
    }

    #[test]
    fn SetSaberBoxSize_matches_oracle() {
        use crate::codemp::game::g_local::{gclient_t, gentity_t};
        use crate::codemp::game::g_main::level_lock;
        use crate::codemp::game::q_shared_h::MAX_BLADES;
        use core::ffi::c_char;

        // Mutates `level` + the `g_entities` base pointer; serialize with the other tests
        // that touch those statics.
        let _g = level_lock();

        unsafe {
            // A broken-parry move and a super-break-lose anim, so the broken-parry guard fires.
            // LS_PARRY_UP is in the PM_SaberInBrokenParry... actually use a clearly non-special
            // saberMove/torsoAnim for the "normal" cases and a known broken-parry one for the
            // guard case. We feed the oracle the *same* predicate result the Rust side computes,
            // so the test stays self-consistent regardless of the exact anim numbers.
            // Broken-parry moves are LS_PARRY_* ... LS_REFLECT_* (see PM_SaberInBrokenParry).
            // A few representative saberMove / torsoAnim values to exercise both branches:
            let saber_moves: [c_int; 2] = [
                0,   /* LS_NONE: not broken */
                162, /* LS_PARRY-ish */
            ];
            let torso_anims: [c_int; 2] =
                [0, 745 /* a BOTH_LK_*_SB_*_L super-break-lose-ish */];

            let level_time = 100_000;
            // storageTime/lastSaberStorageTime relative to level.time: fresh (enter box calc),
            // stale-lastSaber (>200) and stale-blade (>100) -> default box.
            let last_times: [c_int; 3] = [level_time, level_time - 300, level_time];
            let blade_times: [c_int; 3] = [level_time, level_time, level_time - 150];

            let holstered_vals: [c_int; 3] = [0, 1, 2];
            let num_blades_sets: [[c_int; MAX_SABERS]; 3] = [[1, 1], [2, 0], [3, 2]];
            // model present flags per saber: both present, only first, neither.
            let model_present_sets: [[bool; MAX_SABERS]; 3] =
                [[true, true], [true, false], [false, false]];

            let origins: [[f32; 3]; 2] = [[0.0, 0.0, 0.0], [100.0, -50.0, 25.5]];

            for &sm in &saber_moves {
                for &ta in &torso_anims {
                    for ti in 0..last_times.len() {
                        for &hol in &holstered_vals {
                            for nb in 0..num_blades_sets.len() {
                                for mp in 0..model_present_sets.len() {
                                    for &org in &origins {
                                        let mut client: gclient_t = core::mem::zeroed();
                                        client.ps.saberMove = sm;
                                        client.ps.torsoAnim = ta;
                                        client.ps.saberHolstered = hol;
                                        client.lastSaberStorageTime = last_times[ti];

                                        // Fill saber/blade fields deterministically.
                                        let num_blades = num_blades_sets[nb];
                                        let model_present = model_present_sets[mp];
                                        for j in 0..MAX_SABERS {
                                            client.saber[j].numBlades = num_blades[j];
                                            // saber[j].model[0] != 0 iff present.
                                            client.saber[j].model[0] =
                                                if model_present[j] { b'a' as c_char } else { 0 };
                                            for k in 0..(MAX_BLADES as usize) {
                                                let f = (j * 100 + k * 10) as f32;
                                                client.saber[j].blade[k].muzzlePoint =
                                                    [f + 1.0, f + 2.0, f + 3.0];
                                                client.saber[j].blade[k].muzzleDir =
                                                    [0.5, -0.25, 1.0];
                                                client.saber[j].blade[k].lengthMax = 30.0 + f;
                                                client.saber[j].blade[k].storageTime =
                                                    blade_times[ti];
                                            }
                                        }

                                        // Two-entity table: owner at 0 (the saber owner), saber at 1.
                                        let mut ents: [gentity_t; 2] = core::mem::zeroed();
                                        ents[0].inuse = QTRUE;
                                        ents[0].client = &mut client;
                                        ents[0].s.number = 0;
                                        ents[0].s.eType = 0; // not ET_NPC; ownerNum < MAX_CLIENTS path

                                        ents[1].inuse = QTRUE;
                                        ents[1].r.ownerNum = 0;
                                        ents[1].r.currentOrigin = org;

                                        // g_entities is a static array now (no longer a
                                        // re-pointable pointer): stage the owner into slot 0
                                        // (SetSaberBoxSize looks up g_entities[ownerNum==0])
                                        // and restore it afterward. saberent stays local.
                                        let ge0 =
                                            core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
                                        let saved_ge0 = *ge0;
                                        *ge0 = ents[0];
                                        (*addr_of_mut!(level)).time = level_time;

                                        let saberent: *mut gentity_t = &mut ents[1];
                                        SetSaberBoxSize(saberent);
                                        let got_mins = (*saberent).r.mins;
                                        let got_maxs = (*saberent).r.maxs;

                                        *ge0 = saved_ge0;

                                        // Same predicate result the Rust side used.
                                        let in_bp = (PM_SaberInBrokenParry(sm) == QTRUE
                                            || BG_SuperBreakLoseAnim(ta) == QTRUE)
                                            as c_int;

                                        // Pack the oracle's flattened blade arrays.
                                        let mut mz: [[[f32; 3]; 8]; MAX_SABERS] =
                                            [[[0.0; 3]; 8]; MAX_SABERS];
                                        let mut md: [[[f32; 3]; 8]; MAX_SABERS] =
                                            [[[0.0; 3]; 8]; MAX_SABERS];
                                        let mut lm: [[f32; 8]; MAX_SABERS] = [[0.0; 8]; MAX_SABERS];
                                        let mut mpres: [c_int; MAX_SABERS] = [0; MAX_SABERS];
                                        let mut nbl: [c_int; MAX_SABERS] = [0; MAX_SABERS];
                                        // saberFlags2/bladeStyle2Start kept at 0 (forceBlock stays
                                        // false): the PC broken-parry forceBlock path leaves j/k at
                                        // out-of-range values for the later storage check (a latent
                                        // C OOB read), so it is intentionally not exercised here.
                                        let sf2: [c_int; MAX_SABERS] = [0; MAX_SABERS];
                                        let bs2: [c_int; MAX_SABERS] = [0; MAX_SABERS];
                                        for j in 0..MAX_SABERS {
                                            mpres[j] = (client.saber[j].model[0] != 0) as c_int;
                                            nbl[j] = client.saber[j].numBlades;
                                            for k in 0..8 {
                                                mz[j][k] = client.saber[j].blade[k].muzzlePoint;
                                                md[j][k] = client.saber[j].blade[k].muzzleDir;
                                                lm[j][k] = client.saber[j].blade[k].lengthMax;
                                            }
                                        }

                                        let mut o_mins: [f32; 3] = [0.0; 3];
                                        let mut o_maxs: [f32; 3] = [0.0; 3];
                                        oracle::jka_SetSaberBoxSize(
                                            o_mins.as_mut_ptr(),
                                            o_maxs.as_mut_ptr(),
                                            org.as_ptr(),
                                            in_bp,
                                            level_time,
                                            last_times[ti],
                                            blade_times[ti],
                                            hol,
                                            mpres.as_ptr(),
                                            nbl.as_ptr(),
                                            sf2.as_ptr(),
                                            bs2.as_ptr(),
                                            mz.as_ptr(),
                                            md.as_ptr(),
                                            lm.as_ptr(),
                                        );

                                        assert_eq!(
                                            got_mins.map(f32::to_bits),
                                            o_mins.map(f32::to_bits),
                                            "mins sm={sm} ta={ta} ti={ti} hol={hol} nb={nb} mp={mp} org={org:?}"
                                        );
                                        assert_eq!(
                                            got_maxs.map(f32::to_bits),
                                            o_maxs.map(f32::to_bits),
                                            "maxs sm={sm} ta={ta} ti={ti} hol={hol} nb={nb} mp={mp} org={org:?}"
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
}
