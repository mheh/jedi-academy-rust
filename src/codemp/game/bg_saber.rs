//! `bg_saber.c` — the shared (both-games) saber-combat module. **Now essentially
//! complete for the server build.** The file's pure **data tables** landed first
//! (data-layer-first per the 1.03 roadmap): the saber-move FSM table
//! [`saberMoveData`] (the `LS_*` move descriptors), the two
//! `[Q_NUM_QUADS][Q_NUM_QUADS]` quadrant matrices [`transitionMove`] /
//! [`saberMoveTransitionAngle`], and the [`bg_parryDebounce`] per-defense-level
//! array. Then the **self-contained move helpers** — the ones that touch only those
//! tables or their plain `int`/`usercmd_t` arguments ([`PM_AttackMoveForQuad`],
//! [`PM_SaberMoveQuadrantForMovement`], [`PM_SaberInBounce`],
//! [`PM_SaberAttackChainAngle`], [`PM_SaberInBrokenParry`], [`PM_BrokenParryForParry`]).
//! Then the full `pm`/`pml`-driven saber physics / saber-lock / collision FSM —
//! including the helpers once listed here as not yet ported ([`PM_irand_timesync`],
//! [`PM_SaberKataDone`], [`PM_SaberAnimTransitionAnim`], [`PM_CheckStabDown`],
//! [`PM_SomeoneInFront`], …), all landed — including the `#ifdef QAGAME`
//! `NPC_SetAnim( &g_entities[...] )` game-side calls in the lock-win/-loss path.
//!
//! `saberMoveData[]` carries a `char *name`, so (like `vehicleFields[]`) the array
//! is `!Sync` and mirrors the C mutable global as `static mut`; the two int matrices
//! and `bg_parryDebounce` are pointer-free read-only lookups, so they are immutable
//! `pub static` (the `bg_weapons.rs` precedent). Rows are built positionally through
//! the `const fn smd()` (the `wd()`/`vf()` convention), with every C row comment
//! carried verbatim.
//!
//! Oracle: the real `bg_saber.c` cannot be `#include`d (it drags in the clang-hostile
//! tree), so `oracle/bg_saber_oracle.c` `#include`s the AUTHENTIC `anims.h` for the
//! `BOTH_*` values, transcribes the `LS_*`/`Q_*`/`BLK_*` enums + the `AFLAG_*`/struct,
//! then copies all four tables VERBATIM from `bg_saber.c`; the tests compare every
//! element (names via `CStr`), so any transcription slip in the Rust port is caught.

#![allow(non_upper_case_globals, non_snake_case)]

use crate::codemp::game::anims::*;
use crate::codemp::game::bg_local_h::MIN_WALK_NORMAL;
use crate::codemp::game::bg_misc::{
    BG_AddPredictableEventToPlayerstate, BG_CanUseFPNow, BG_HasYsalamiri,
};
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::g_entities;
use crate::codemp::game::npc::NPC_SetAnim;
use crate::codemp::game::bg_panimate::{
    BG_FlippingAnim, BG_InKataAnim, BG_InRoll, BG_InSaberLock, BG_InSaberLockOld, BG_InSaberStandAnim,
    BG_InSpecialJump, BG_KickMove, BG_KickingAnim, BG_SaberInAttack, BG_SaberInIdle, BG_SaberInKata,
    BG_SaberInSpecial, BG_SaberInSpecialAttack, BG_SaberInTransitionAny, BG_SpinningSaberAnim,
    BG_SuperBreakLoseAnim, BG_SuperBreakWinAnim, PM_InKnockDown, PM_JumpingAnim,
    PM_SaberBounceForAttack, PM_SaberInKnockaway, PM_SaberInParry, PM_SaberInReflect,
    PM_SaberInReturn, PM_SaberInStart, PM_SaberInTransition, PM_SetAnim,
};
use crate::codemp::game::bg_pmove::{
    forcePowerNeeded, pm, pml, BG_InKnockDown, BG_InSlopeAnim, BG_KnockDownable, BG_SabersOff,
    PM_AddEvent, PM_BGEntForNum, PM_BeginWeaponChange, PM_FinishWeaponChange, PM_GetSaberStance,
    PM_RunningAnim, PM_SetForceJumpZStart, PM_SwimmingAnim, PM_WalkingAnim,
};
use crate::codemp::game::bg_public::*;
use crate::codemp::game::bg_weapons::weaponData;
use crate::codemp::game::q_math::{
    AngleVectors, DistanceSquared, Q_random, VectorCopy, VectorLength, VectorMA, VectorNormalize,
    VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared_h::{
    playerState_t, qboolean, trace_t, usercmd_t, vec3_t, BLK_NO, BLK_TIGHT, BLK_WIDE,
    BLOCKED_ATK_BOUNCE, BLOCKED_BOUNCE_MOVE, BLOCKED_LOWER_LEFT, BLOCKED_LOWER_LEFT_PROJ,
    BLOCKED_LOWER_RIGHT, BLOCKED_LOWER_RIGHT_PROJ, BLOCKED_NONE, BLOCKED_PARRY_BROKEN, BLOCKED_TOP,
    BLOCKED_TOP_PROJ, BLOCKED_UPPER_LEFT, BLOCKED_UPPER_LEFT_PROJ, BLOCKED_UPPER_RIGHT,
    BLOCKED_UPPER_RIGHT_PROJ, BUTTON_ALT_ATTACK, BUTTON_ATTACK, ENTITYNUM_NONE, ENTITYNUM_WORLD,
    FORCE_LEVEL_1, FORCE_LEVEL_2, FORCE_LEVEL_3, FP_GRIP, FP_LEVITATION, FP_SABERTHROW,
    FP_SABER_DEFENSE, FP_SABER_OFFENSE, MAX_CLIENTS, NUM_FORCE_POWER_LEVELS, PITCH, QFALSE, QTRUE,
    ROLL, SFL_NO_MIRROR_ATTACKS, SFL_NO_ROLL_STAB, SFL_NO_STABDOWN, SS_DESANN, SS_DUAL, SS_FAST,
    SS_MEDIUM, SS_STAFF, SS_STRONG, SS_TAVION, YAW, saberInfo_t,
};
use crate::codemp::game::bg_weapons_h::WP_SABER;
use crate::codemp::game::w_saber_h::{
    SABERMAXS_X, SABERMAXS_Y, SABERMAXS_Z, SABERMINS_X, SABERMINS_Y, SABERMINS_Z,
    SABER_MIN_THROW_DIST, SEF_LOCK_WON,
};
use core::ffi::{c_char, c_int, c_short, c_uint, CStr};
use core::ptr::addr_of;

// Per-move animation-flag presets (bg_saber.c) — combinations of the SETANIM_FLAG_*
// bits, used by the `animSetFlags` column below.
const AFLAG_IDLE: c_uint = SETANIM_FLAG_NORMAL as c_uint;
const AFLAG_ACTIVE: c_uint =
    (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS) as c_uint;
// AFLAG_WAIT is defined by bg_saber.c but referenced by no row in this table (a C
// `#define` warns on neither side); kept for fidelity.
#[allow(dead_code)]
const AFLAG_WAIT: c_uint = (SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS) as c_uint;
const AFLAG_FINISH: c_uint = SETANIM_FLAG_HOLD as c_uint;

/// Builds one [`saberMoveData_t`] row positionally, mirroring the C aggregate
/// initializer (field order: name, animToUse, startQuad, endQuad, animSetFlags,
/// blendTime, blocking, chain_idle, chain_attack, trailLength). The `char *name`
/// points at a `'static` C-string literal (`c"..."`); the table never writes
/// through it, so the `*const`→`*mut` cast is sound (matches the C `char *` field).
#[allow(clippy::too_many_arguments)]
const fn smd(
    name: &'static CStr,
    animToUse: c_int,
    startQuad: c_int,
    endQuad: c_int,
    animSetFlags: c_uint,
    blendTime: c_int,
    blocking: c_int,
    chain_idle: saberMoveName_t,
    chain_attack: saberMoveName_t,
    trailLength: qboolean,
) -> saberMoveData_t {
    saberMoveData_t {
        name: name.as_ptr() as *mut c_char,
        animToUse,
        startQuad,
        endQuad,
        animSetFlags,
        blendTime,
        blocking,
        chain_idle,
        chain_attack,
        trailLength,
    }
}

//FIXME: add the alternate anims for each style?
/// `saberMoveData[LS_MOVE_MAX]` (bg_saber.c) — the saber-move finite-state table:
/// per `LS_*` move, its animation, start/end quadrants, anim flags, blend time,
/// block arc, and the idle/attack moves it chains into. (C note: `NB:randomized`.)
pub static mut saberMoveData: [saberMoveData_t; LS_MOVE_MAX as usize] = [
    // name			anim(do all styles?)startQ	endQ	setanimflag		blend,	blocking	chain_idle		chain_attack	trailLen
    smd(c"None", BOTH_STAND1, Q_R, Q_R, AFLAG_IDLE, 350, BLK_NO, LS_NONE, LS_NONE, 0), // LS_NONE		= 0,

    // General movements with saber
    smd(c"Ready", BOTH_STAND2, Q_R, Q_R, AFLAG_IDLE, 350, BLK_WIDE, LS_READY, LS_S_R2L, 0), // LS_READY,
    smd(c"Draw", BOTH_STAND1TO2, Q_R, Q_R, AFLAG_FINISH, 350, BLK_NO, LS_READY, LS_S_R2L, 0), // LS_DRAW,
    smd(c"Putaway", BOTH_STAND2TO1, Q_R, Q_R, AFLAG_FINISH, 350, BLK_NO, LS_READY, LS_S_R2L, 0), // LS_PUTAWAY,

    // Attacks
    //UL2LR
    smd(c"TL2BR Att", BOTH_A1_TL_BR, Q_TL, Q_BR, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_R_TL2BR, LS_R_TL2BR, 200), // LS_A_TL2BR
    //SLASH LEFT
    smd(c"L2R Att", BOTH_A1__L__R, Q_L, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_R_L2R, LS_R_L2R, 200), // LS_A_L2R
    //LL2UR
    smd(c"BL2TR Att", BOTH_A1_BL_TR, Q_BL, Q_TR, AFLAG_ACTIVE, 50, BLK_TIGHT, LS_R_BL2TR, LS_R_BL2TR, 200), // LS_A_BL2TR
    //LR2UL
    smd(c"BR2TL Att", BOTH_A1_BR_TL, Q_BR, Q_TL, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_R_BR2TL, LS_R_BR2TL, 200), // LS_A_BR2TL
    //SLASH RIGHT
    smd(c"R2L Att", BOTH_A1__R__L, Q_R, Q_L, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_R_R2L, LS_R_R2L, 200), // LS_A_R2L
    //UR2LL
    smd(c"TR2BL Att", BOTH_A1_TR_BL, Q_TR, Q_BL, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_R_TR2BL, LS_R_TR2BL, 200), // LS_A_TR2BL
    //SLASH DOWN
    smd(c"T2B Att", BOTH_A1_T__B_, Q_T, Q_B, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_R_T2B, LS_R_T2B, 200), // LS_A_T2B
    //special attacks
    smd(c"Back Stab", BOTH_A2_STABBACK1, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_A_BACKSTAB
    smd(c"Back Att", BOTH_ATTACK_BACK, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_A_BACK
    smd(c"CR Back Att", BOTH_CROUCHATTACKBACK1, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_A_BACK_CR
    smd(c"RollStab", BOTH_ROLL_STAB, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_ROLL_STAB
    smd(c"Lunge Att", BOTH_LUNGE2_B__T_, Q_B, Q_T, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_A_LUNGE
    smd(c"Jump Att", BOTH_FORCELEAP2_T__B_, Q_T, Q_B, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_A_JUMP_T__B_
    smd(c"Flip Stab", BOTH_JUMPFLIPSTABDOWN, Q_R, Q_T, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1_T___R, 200), // LS_A_FLIP_STAB
    smd(c"Flip Slash", BOTH_JUMPFLIPSLASHDOWN1, Q_L, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1__R_T_, 200), // LS_A_FLIP_SLASH
    smd(c"DualJump Atk", BOTH_JUMPATTACK6, Q_R, Q_BL, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1_BL_TR, 200), // LS_JUMPATTACK_DUAL

    smd(c"DualJumpAtkL_A", BOTH_ARIAL_LEFT, Q_R, Q_TL, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_A_TL2BR, 200), // LS_JUMPATTACK_ARIAL_LEFT
    smd(c"DualJumpAtkR_A", BOTH_ARIAL_RIGHT, Q_R, Q_TR, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_A_TR2BL, 200), // LS_JUMPATTACK_ARIAL_RIGHT

    smd(c"DualJumpAtkL_A", BOTH_CARTWHEEL_LEFT, Q_R, Q_TL, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1_TL_BR, 200), // LS_JUMPATTACK_CART_LEFT
    smd(c"DualJumpAtkR_A", BOTH_CARTWHEEL_RIGHT, Q_R, Q_TR, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1_TR_BL, 200), // LS_JUMPATTACK_CART_RIGHT

    smd(c"DualJumpAtkLStaff", BOTH_BUTTERFLY_FL1, Q_R, Q_L, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1__L__R, 200), // LS_JUMPATTACK_STAFF_LEFT
    smd(c"DualJumpAtkRStaff", BOTH_BUTTERFLY_FR1, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1__R__L, 200), // LS_JUMPATTACK_STAFF_RIGHT

    smd(c"ButterflyLeft", BOTH_BUTTERFLY_LEFT, Q_R, Q_L, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1__L__R, 200), // LS_BUTTERFLY_LEFT
    smd(c"ButterflyRight", BOTH_BUTTERFLY_RIGHT, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1__R__L, 200), // LS_BUTTERFLY_RIGHT

    smd(c"BkFlip Atk", BOTH_JUMPATTACK7, Q_B, Q_T, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_T1_T___R, 200), // LS_A_BACKFLIP_ATK
    smd(c"DualSpinAtk", BOTH_SPINATTACK6, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_SPINATTACK_DUAL
    smd(c"StfSpinAtk", BOTH_SPINATTACK7, Q_L, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_SPINATTACK
    smd(c"LngLeapAtk", BOTH_FORCELONGLEAP_ATTACK, Q_R, Q_L, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_LEAP_ATTACK
    smd(c"SwoopAtkR", BOTH_VS_ATR_S, Q_R, Q_T, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_SWOOP_ATTACK_RIGHT
    smd(c"SwoopAtkL", BOTH_VS_ATL_S, Q_L, Q_T, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_SWOOP_ATTACK_LEFT
    smd(c"TauntaunAtkR", BOTH_VT_ATR_S, Q_R, Q_T, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_TAUNTAUN_ATTACK_RIGHT
    smd(c"TauntaunAtkL", BOTH_VT_ATL_S, Q_L, Q_T, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_TAUNTAUN_ATTACK_LEFT
    smd(c"StfKickFwd", BOTH_A7_KICK_F, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_KICK_F
    smd(c"StfKickBack", BOTH_A7_KICK_B, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_KICK_B
    smd(c"StfKickRight", BOTH_A7_KICK_R, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_KICK_R
    smd(c"StfKickLeft", BOTH_A7_KICK_L, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_KICK_L
    smd(c"StfKickSpin", BOTH_A7_KICK_S, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_S_R2L, 200), // LS_KICK_S
    smd(c"StfKickBkFwd", BOTH_A7_KICK_BF, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_S_R2L, 200), // LS_KICK_BF
    smd(c"StfKickSplit", BOTH_A7_KICK_RL, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_S_R2L, 200), // LS_KICK_RL
    smd(c"StfKickFwdAir", BOTH_A7_KICK_F_AIR, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_KICK_F_AIR
    smd(c"StfKickBackAir", BOTH_A7_KICK_B_AIR, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_KICK_B_AIR
    smd(c"StfKickRightAir", BOTH_A7_KICK_R_AIR, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_KICK_R_AIR
    smd(c"StfKickLeftAir", BOTH_A7_KICK_L_AIR, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_KICK_L_AIR
    smd(c"StabDown", BOTH_STABDOWN, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_STABDOWN
    smd(c"StabDownStf", BOTH_STABDOWN_STAFF, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_STABDOWN_STAFF
    smd(c"StabDownDual", BOTH_STABDOWN_DUAL, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_S_R2L, 200), // LS_STABDOWN_DUAL
    smd(c"dualspinprot", BOTH_A6_SABERPROTECT, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 500), // LS_DUAL_SPIN_PROTECT
    smd(c"StfSoulCal", BOTH_A7_SOULCAL, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 500), // LS_STAFF_SOULCAL
    smd(c"specialfast", BOTH_A1_SPECIAL, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 2000), // LS_A1_SPECIAL
    smd(c"specialmed", BOTH_A2_SPECIAL, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 2000), // LS_A2_SPECIAL
    smd(c"specialstr", BOTH_A3_SPECIAL, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 2000), // LS_A3_SPECIAL
    smd(c"upsidedwnatk", BOTH_FLIP_ATTACK7, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_UPSIDE_DOWN_ATTACK
    smd(c"pullatkstab", BOTH_PULL_IMPALE_STAB, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_PULL_ATTACK_STAB
    smd(c"pullatkswing", BOTH_PULL_IMPALE_SWING, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_PULL_ATTACK_SWING
    smd(c"AloraSpinAtk", BOTH_ALORA_SPIN_SLASH, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_SPINATTACK_ALORA
    smd(c"Dual FB Atk", BOTH_A6_FB, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_DUAL_FB
    smd(c"Dual LR Atk", BOTH_A6_LR, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_DUAL_LR
    smd(c"StfHiltBash", BOTH_A7_HILT, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_HILT_BASH

    //starts
    smd(c"TL2BR St", BOTH_S1_S1_TL, Q_R, Q_TL, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_A_TL2BR, LS_A_TL2BR, 200), // LS_S_TL2BR
    smd(c"L2R St", BOTH_S1_S1__L, Q_R, Q_L, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_A_L2R, LS_A_L2R, 200), // LS_S_L2R
    smd(c"BL2TR St", BOTH_S1_S1_BL, Q_R, Q_BL, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_A_BL2TR, LS_A_BL2TR, 200), // LS_S_BL2TR
    smd(c"BR2TL St", BOTH_S1_S1_BR, Q_R, Q_BR, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_A_BR2TL, LS_A_BR2TL, 200), // LS_S_BR2TL
    smd(c"R2L St", BOTH_S1_S1__R, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_A_R2L, LS_A_R2L, 200), // LS_S_R2L
    smd(c"TR2BL St", BOTH_S1_S1_TR, Q_R, Q_TR, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_A_TR2BL, LS_A_TR2BL, 200), // LS_S_TR2BL
    smd(c"T2B St", BOTH_S1_S1_T_, Q_R, Q_T, AFLAG_ACTIVE, 100, BLK_TIGHT, LS_A_T2B, LS_A_T2B, 200), // LS_S_T2B

    //returns
    smd(c"TL2BR Ret", BOTH_R1_BR_S1, Q_BR, Q_R, AFLAG_FINISH, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_R_TL2BR
    smd(c"L2R Ret", BOTH_R1__R_S1, Q_R, Q_R, AFLAG_FINISH, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_R_L2R
    smd(c"BL2TR Ret", BOTH_R1_TR_S1, Q_TR, Q_R, AFLAG_FINISH, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_R_BL2TR
    smd(c"BR2TL Ret", BOTH_R1_TL_S1, Q_TL, Q_R, AFLAG_FINISH, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_R_BR2TL
    smd(c"R2L Ret", BOTH_R1__L_S1, Q_L, Q_R, AFLAG_FINISH, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_R_R2L
    smd(c"TR2BL Ret", BOTH_R1_BL_S1, Q_BL, Q_R, AFLAG_FINISH, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_R_TR2BL
    smd(c"T2B Ret", BOTH_R1_B__S1, Q_B, Q_R, AFLAG_FINISH, 100, BLK_TIGHT, LS_READY, LS_READY, 200), // LS_R_T2B

    //Transitions
    smd(c"BR2R Trans", BOTH_T1_BR__R, Q_BR, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_R_L2R, LS_A_R2L, 150), //# Fast arc bottom right to right
    smd(c"BR2TR Trans", BOTH_T1_BR_TR, Q_BR, Q_TR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_TR2BL, 150), //# Fast arc bottom right to top right		(use: BOTH_T1_TR_BR)
    smd(c"BR2T Trans", BOTH_T1_BR_T_, Q_BR, Q_T, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_T2B, 150), //# Fast arc bottom right to top			(use: BOTH_T1_T__BR)
    smd(c"BR2TL Trans", BOTH_T1_BR_TL, Q_BR, Q_TL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BR2TL, LS_A_TL2BR, 150), //# Fast weak spin bottom right to top left
    smd(c"BR2L Trans", BOTH_T1_BR__L, Q_BR, Q_L, AFLAG_ACTIVE, 100, BLK_NO, LS_R_R2L, LS_A_L2R, 150), //# Fast weak spin bottom right to left
    smd(c"BR2BL Trans", BOTH_T1_BR_BL, Q_BR, Q_BL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TR2BL, LS_A_BL2TR, 150), //# Fast weak spin bottom right to bottom left
    smd(c"R2BR Trans", BOTH_T1__R_BR, Q_R, Q_BR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TL2BR, LS_A_BR2TL, 150), //# Fast arc right to bottom right			(use: BOTH_T1_BR__R)
    smd(c"R2TR Trans", BOTH_T1__R_TR, Q_R, Q_TR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_TR2BL, 150), //# Fast arc right to top right
    smd(c"R2T Trans", BOTH_T1__R_T_, Q_R, Q_T, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_T2B, 150), //# Fast ar right to top				(use: BOTH_T1_T___R)
    smd(c"R2TL Trans", BOTH_T1__R_TL, Q_R, Q_TL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BR2TL, LS_A_TL2BR, 150), //# Fast arc right to top left
    smd(c"R2L Trans", BOTH_T1__R__L, Q_R, Q_L, AFLAG_ACTIVE, 100, BLK_NO, LS_R_R2L, LS_A_L2R, 150), //# Fast weak spin right to left
    smd(c"R2BL Trans", BOTH_T1__R_BL, Q_R, Q_BL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TR2BL, LS_A_BL2TR, 150), //# Fast weak spin right to bottom left
    smd(c"TR2BR Trans", BOTH_T1_TR_BR, Q_TR, Q_BR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TL2BR, LS_A_BR2TL, 150), //# Fast arc top right to bottom right
    smd(c"TR2R Trans", BOTH_T1_TR__R, Q_TR, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_R_L2R, LS_A_R2L, 150), //# Fast arc top right to right			(use: BOTH_T1__R_TR)
    smd(c"TR2T Trans", BOTH_T1_TR_T_, Q_TR, Q_T, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_T2B, 150), //# Fast arc top right to top				(use: BOTH_T1_T__TR)
    smd(c"TR2TL Trans", BOTH_T1_TR_TL, Q_TR, Q_TL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BR2TL, LS_A_TL2BR, 150), //# Fast arc top right to top left
    smd(c"TR2L Trans", BOTH_T1_TR__L, Q_TR, Q_L, AFLAG_ACTIVE, 100, BLK_NO, LS_R_R2L, LS_A_L2R, 150), //# Fast arc top right to left
    smd(c"TR2BL Trans", BOTH_T1_TR_BL, Q_TR, Q_BL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TR2BL, LS_A_BL2TR, 150), //# Fast weak spin top right to bottom left
    smd(c"T2BR Trans", BOTH_T1_T__BR, Q_T, Q_BR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TL2BR, LS_A_BR2TL, 150), //# Fast arc top to bottom right
    smd(c"T2R Trans", BOTH_T1_T___R, Q_T, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_R_L2R, LS_A_R2L, 150), //# Fast arc top to right
    smd(c"T2TR Trans", BOTH_T1_T__TR, Q_T, Q_TR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_TR2BL, 150), //# Fast arc top to top right
    smd(c"T2TL Trans", BOTH_T1_T__TL, Q_T, Q_TL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BR2TL, LS_A_TL2BR, 150), //# Fast arc top to top left
    smd(c"T2L Trans", BOTH_T1_T___L, Q_T, Q_L, AFLAG_ACTIVE, 100, BLK_NO, LS_R_R2L, LS_A_L2R, 150), //# Fast arc top to left
    smd(c"T2BL Trans", BOTH_T1_T__BL, Q_T, Q_BL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TR2BL, LS_A_BL2TR, 150), //# Fast arc top to bottom left
    smd(c"TL2BR Trans", BOTH_T1_TL_BR, Q_TL, Q_BR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TL2BR, LS_A_BR2TL, 150), //# Fast weak spin top left to bottom right
    smd(c"TL2R Trans", BOTH_T1_TL__R, Q_TL, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_R_L2R, LS_A_R2L, 150), //# Fast arc top left to right			(use: BOTH_T1__R_TL)
    smd(c"TL2TR Trans", BOTH_T1_TL_TR, Q_TL, Q_TR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_TR2BL, 150), //# Fast arc top left to top right			(use: BOTH_T1_TR_TL)
    smd(c"TL2T Trans", BOTH_T1_TL_T_, Q_TL, Q_T, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_T2B, 150), //# Fast arc top left to top				(use: BOTH_T1_T__TL)
    smd(c"TL2L Trans", BOTH_T1_TL__L, Q_TL, Q_L, AFLAG_ACTIVE, 100, BLK_NO, LS_R_R2L, LS_A_L2R, 150), //# Fast arc top left to left				(use: BOTH_T1__L_TL)
    smd(c"TL2BL Trans", BOTH_T1_TL_BL, Q_TL, Q_BL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TR2BL, LS_A_BL2TR, 150), //# Fast arc top left to bottom left
    smd(c"L2BR Trans", BOTH_T1__L_BR, Q_L, Q_BR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TL2BR, LS_A_BR2TL, 150), //# Fast weak spin left to bottom right
    smd(c"L2R Trans", BOTH_T1__L__R, Q_L, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_R_L2R, LS_A_R2L, 150), //# Fast weak spin left to right
    smd(c"L2TR Trans", BOTH_T1__L_TR, Q_L, Q_TR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_TR2BL, 150), //# Fast arc left to top right			(use: BOTH_T1_TR__L)
    smd(c"L2T Trans", BOTH_T1__L_T_, Q_L, Q_T, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_T2B, 150), //# Fast arc left to top				(use: BOTH_T1_T___L)
    smd(c"L2TL Trans", BOTH_T1__L_TL, Q_L, Q_TL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BR2TL, LS_A_TL2BR, 150), //# Fast arc left to top left
    smd(c"L2BL Trans", BOTH_T1__L_BL, Q_L, Q_BL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TR2BL, LS_A_BL2TR, 150), //# Fast arc left to bottom left			(use: BOTH_T1_BL__L)
    smd(c"BL2BR Trans", BOTH_T1_BL_BR, Q_BL, Q_BR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TL2BR, LS_A_BR2TL, 150), //# Fast weak spin bottom left to bottom right
    smd(c"BL2R Trans", BOTH_T1_BL__R, Q_BL, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_R_L2R, LS_A_R2L, 150), //# Fast weak spin bottom left to right
    smd(c"BL2TR Trans", BOTH_T1_BL_TR, Q_BL, Q_TR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_TR2BL, 150), //# Fast weak spin bottom left to top right
    smd(c"BL2T Trans", BOTH_T1_BL_T_, Q_BL, Q_T, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_A_T2B, 150), //# Fast arc bottom left to top			(use: BOTH_T1_T__BL)
    smd(c"BL2TL Trans", BOTH_T1_BL_TL, Q_BL, Q_TL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BR2TL, LS_A_TL2BR, 150), //# Fast arc bottom left to top left		(use: BOTH_T1_TL_BL)
    smd(c"BL2L Trans", BOTH_T1_BL__L, Q_BL, Q_L, AFLAG_ACTIVE, 100, BLK_NO, LS_R_R2L, LS_A_L2R, 150), //# Fast arc bottom left to left

    //Bounces
    smd(c"Bounce BR", BOTH_B1_BR___, Q_BR, Q_BR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TL2BR, LS_T1_BR_TR, 150),
    smd(c"Bounce R", BOTH_B1__R___, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_R_L2R, LS_T1__R__L, 150),
    smd(c"Bounce TR", BOTH_B1_TR___, Q_TR, Q_TR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_T1_TR_TL, 150),
    smd(c"Bounce T", BOTH_B1_T____, Q_T, Q_T, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_T1_T__BL, 150),
    smd(c"Bounce TL", BOTH_B1_TL___, Q_TL, Q_TL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BR2TL, LS_T1_TL_TR, 150),
    smd(c"Bounce L", BOTH_B1__L___, Q_L, Q_L, AFLAG_ACTIVE, 100, BLK_NO, LS_R_R2L, LS_T1__L__R, 150),
    smd(c"Bounce BL", BOTH_B1_BL___, Q_BL, Q_BL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TR2BL, LS_T1_BL_TR, 150),

    //Deflected attacks (like bounces, but slide off enemy saber, not straight back)
    smd(c"Deflect BR", BOTH_D1_BR___, Q_BR, Q_BR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TL2BR, LS_T1_BR_TR, 150),
    smd(c"Deflect R", BOTH_D1__R___, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_R_L2R, LS_T1__R__L, 150),
    smd(c"Deflect TR", BOTH_D1_TR___, Q_TR, Q_TR, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_T1_TR_TL, 150),
    smd(c"Deflect T", BOTH_B1_T____, Q_T, Q_T, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_T1_T__BL, 150),
    smd(c"Deflect TL", BOTH_D1_TL___, Q_TL, Q_TL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BR2TL, LS_T1_TL_TR, 150),
    smd(c"Deflect L", BOTH_D1__L___, Q_L, Q_L, AFLAG_ACTIVE, 100, BLK_NO, LS_R_R2L, LS_T1__L__R, 150),
    smd(c"Deflect BL", BOTH_D1_BL___, Q_BL, Q_BL, AFLAG_ACTIVE, 100, BLK_NO, LS_R_TR2BL, LS_T1_BL_TR, 150),
    smd(c"Deflect B", BOTH_D1_B____, Q_B, Q_B, AFLAG_ACTIVE, 100, BLK_NO, LS_R_BL2TR, LS_T1_T__BL, 150),

    //Reflected attacks
    smd(c"Reflected BR", BOTH_V1_BR_S1, Q_BR, Q_BR, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_READY, 150), //	LS_V1_BR
    smd(c"Reflected R", BOTH_V1__R_S1, Q_R, Q_R, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_READY, 150), //	LS_V1__R
    smd(c"Reflected TR", BOTH_V1_TR_S1, Q_TR, Q_TR, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_READY, 150), //	LS_V1_TR
    smd(c"Reflected T", BOTH_V1_T__S1, Q_T, Q_T, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_READY, 150), //	LS_V1_T_
    smd(c"Reflected TL", BOTH_V1_TL_S1, Q_TL, Q_TL, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_READY, 150), //	LS_V1_TL
    smd(c"Reflected L", BOTH_V1__L_S1, Q_L, Q_L, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_READY, 150), //	LS_V1__L
    smd(c"Reflected BL", BOTH_V1_BL_S1, Q_BL, Q_BL, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_READY, 150), //	LS_V1_BL
    smd(c"Reflected B", BOTH_V1_B__S1, Q_B, Q_B, AFLAG_ACTIVE, 100, BLK_NO, LS_READY, LS_READY, 150), //	LS_V1_B_

    // Broken parries
    smd(c"BParry Top", BOTH_H1_S1_T_, Q_T, Q_B, AFLAG_ACTIVE, 50, BLK_NO, LS_READY, LS_READY, 150), // LS_PARRY_UP,
    smd(c"BParry UR", BOTH_H1_S1_TR, Q_TR, Q_BL, AFLAG_ACTIVE, 50, BLK_NO, LS_READY, LS_READY, 150), // LS_PARRY_UR,
    smd(c"BParry UL", BOTH_H1_S1_TL, Q_TL, Q_BR, AFLAG_ACTIVE, 50, BLK_NO, LS_READY, LS_READY, 150), // LS_PARRY_UL,
    smd(c"BParry LR", BOTH_H1_S1_BL, Q_BL, Q_TR, AFLAG_ACTIVE, 50, BLK_NO, LS_READY, LS_READY, 150), // LS_PARRY_LR,
    smd(c"BParry Bot", BOTH_H1_S1_B_, Q_B, Q_T, AFLAG_ACTIVE, 50, BLK_NO, LS_READY, LS_READY, 150), // LS_PARRY_LL
    smd(c"BParry LL", BOTH_H1_S1_BR, Q_BR, Q_TL, AFLAG_ACTIVE, 50, BLK_NO, LS_READY, LS_READY, 150), // LS_PARRY_LL

    // Knockaways
    smd(c"Knock Top", BOTH_K1_S1_T_, Q_R, Q_T, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_BL2TR, LS_T1_T__BR, 150), // LS_PARRY_UP,
    smd(c"Knock UR", BOTH_K1_S1_TR, Q_R, Q_TR, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_BL2TR, LS_T1_TR__R, 150), // LS_PARRY_UR,
    smd(c"Knock UL", BOTH_K1_S1_TL, Q_R, Q_TL, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_BR2TL, LS_T1_TL__L, 150), // LS_PARRY_UL,
    smd(c"Knock LR", BOTH_K1_S1_BL, Q_R, Q_BL, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_TL2BR, LS_T1_BL_TL, 150), // LS_PARRY_LR,
    smd(c"Knock LL", BOTH_K1_S1_BR, Q_R, Q_BR, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_TR2BL, LS_T1_BR_TR, 150), // LS_PARRY_LL

    // Parry
    smd(c"Parry Top", BOTH_P1_S1_T_, Q_R, Q_T, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_BL2TR, LS_A_T2B, 150), // LS_PARRY_UP,
    smd(c"Parry UR", BOTH_P1_S1_TR, Q_R, Q_TL, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_BL2TR, LS_A_TR2BL, 150), // LS_PARRY_UR,
    smd(c"Parry UL", BOTH_P1_S1_TL, Q_R, Q_TR, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_BR2TL, LS_A_TL2BR, 150), // LS_PARRY_UL,
    smd(c"Parry LR", BOTH_P1_S1_BL, Q_R, Q_BR, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_TL2BR, LS_A_BR2TL, 150), // LS_PARRY_LR,
    smd(c"Parry LL", BOTH_P1_S1_BR, Q_R, Q_BL, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_TR2BL, LS_A_BL2TR, 150), // LS_PARRY_LL

    // Reflecting a missile
    smd(c"Reflect Top", BOTH_P1_S1_T_, Q_R, Q_T, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_BL2TR, LS_A_T2B, 300), // LS_PARRY_UP,
    smd(c"Reflect UR", BOTH_P1_S1_TL, Q_R, Q_TR, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_BR2TL, LS_A_TL2BR, 300), // LS_PARRY_UR,
    smd(c"Reflect UL", BOTH_P1_S1_TR, Q_R, Q_TL, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_BL2TR, LS_A_TR2BL, 300), // LS_PARRY_UL,
    smd(c"Reflect LR", BOTH_P1_S1_BR, Q_R, Q_BL, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_TR2BL, LS_A_BL2TR, 300), // LS_PARRY_LR
    smd(c"Reflect LL", BOTH_P1_S1_BL, Q_R, Q_BR, AFLAG_ACTIVE, 50, BLK_WIDE, LS_R_TL2BR, LS_A_BR2TL, 300), // LS_PARRY_LL,
];

/// `transitionMove[Q_NUM_QUADS][Q_NUM_QUADS]` (bg_saber.c) — for a [from-quad][to-quad]
/// pair, the `LS_T1_*` transition move that bridges them (`LS_NONE` where no transition
/// exists). Row-major, matching the C flat initializer.
#[rustfmt::skip]
pub static transitionMove: [[c_int; Q_NUM_QUADS as usize]; Q_NUM_QUADS as usize] = [
    [
        LS_NONE, //Can't transition to same pos!
        LS_T1_BR__R, //40
        LS_T1_BR_TR,
        LS_T1_BR_T_,
        LS_T1_BR_TL,
        LS_T1_BR__L,
        LS_T1_BR_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1__R_BR, //46
        LS_NONE, //Can't transition to same pos!
        LS_T1__R_TR,
        LS_T1__R_T_,
        LS_T1__R_TL,
        LS_T1__R__L,
        LS_T1__R_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_TR_BR, //52
        LS_T1_TR__R,
        LS_NONE, //Can't transition to same pos!
        LS_T1_TR_T_,
        LS_T1_TR_TL,
        LS_T1_TR__L,
        LS_T1_TR_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_T__BR, //58
        LS_T1_T___R,
        LS_T1_T__TR,
        LS_NONE, //Can't transition to same pos!
        LS_T1_T__TL,
        LS_T1_T___L,
        LS_T1_T__BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_TL_BR, //64
        LS_T1_TL__R,
        LS_T1_TL_TR,
        LS_T1_TL_T_,
        LS_NONE, //Can't transition to same pos!
        LS_T1_TL__L,
        LS_T1_TL_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1__L_BR, //70
        LS_T1__L__R,
        LS_T1__L_TR,
        LS_T1__L_T_,
        LS_T1__L_TL,
        LS_NONE, //Can't transition to same pos!
        LS_T1__L_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_BL_BR, //76
        LS_T1_BL__R,
        LS_T1_BL_TR,
        LS_T1_BL_T_,
        LS_T1_BL_TL,
        LS_T1_BL__L,
        LS_NONE, //Can't transition to same pos!
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
    [
        LS_T1_BL_BR, //NOTE: there are no transitions from bottom, so re-use the bottom right transitions
        LS_T1_BR__R,
        LS_T1_BR_TR,
        LS_T1_BR_T_,
        LS_T1_BR_TL,
        LS_T1_BR__L,
        LS_T1_BR_BL,
        LS_NONE, //No transitions to bottom, and no anims start there, so shouldn't need any
    ],
];

/// `saberMoveTransitionAngle[Q_NUM_QUADS][Q_NUM_QUADS]` (bg_saber.c) — for a
/// [from-quad][to-quad] pair, the angle (degrees) swept by the transition. Row-major.
#[rustfmt::skip]
pub static saberMoveTransitionAngle: [[c_int; Q_NUM_QUADS as usize]; Q_NUM_QUADS as usize] = [
    [
        0, //Q_BR,Q_BR,
        45, //Q_BR,Q_R,
        90, //Q_BR,Q_TR,
        135, //Q_BR,Q_T,
        180, //Q_BR,Q_TL,
        215, //Q_BR,Q_L,
        270, //Q_BR,Q_BL,
        45, //Q_BR,Q_B,
    ],
    [
        45, //Q_R,Q_BR,
        0, //Q_R,Q_R,
        45, //Q_R,Q_TR,
        90, //Q_R,Q_T,
        135, //Q_R,Q_TL,
        180, //Q_R,Q_L,
        215, //Q_R,Q_BL,
        90, //Q_R,Q_B,
    ],
    [
        90, //Q_TR,Q_BR,
        45, //Q_TR,Q_R,
        0, //Q_TR,Q_TR,
        45, //Q_TR,Q_T,
        90, //Q_TR,Q_TL,
        135, //Q_TR,Q_L,
        180, //Q_TR,Q_BL,
        135, //Q_TR,Q_B,
    ],
    [
        135, //Q_T,Q_BR,
        90, //Q_T,Q_R,
        45, //Q_T,Q_TR,
        0, //Q_T,Q_T,
        45, //Q_T,Q_TL,
        90, //Q_T,Q_L,
        135, //Q_T,Q_BL,
        180, //Q_T,Q_B,
    ],
    [
        180, //Q_TL,Q_BR,
        135, //Q_TL,Q_R,
        90, //Q_TL,Q_TR,
        45, //Q_TL,Q_T,
        0, //Q_TL,Q_TL,
        45, //Q_TL,Q_L,
        90, //Q_TL,Q_BL,
        135, //Q_TL,Q_B,
    ],
    [
        215, //Q_L,Q_BR,
        180, //Q_L,Q_R,
        135, //Q_L,Q_TR,
        90, //Q_L,Q_T,
        45, //Q_L,Q_TL,
        0, //Q_L,Q_L,
        45, //Q_L,Q_BL,
        90, //Q_L,Q_B,
    ],
    [
        270, //Q_BL,Q_BR,
        215, //Q_BL,Q_R,
        180, //Q_BL,Q_TR,
        135, //Q_BL,Q_T,
        90, //Q_BL,Q_TL,
        45, //Q_BL,Q_L,
        0, //Q_BL,Q_BL,
        45, //Q_BL,Q_B,
    ],
    [
        45, //Q_B,Q_BR,
        90, //Q_B,Q_R,
        135, //Q_B,Q_TR,
        180, //Q_B,Q_T,
        135, //Q_B,Q_TL,
        90, //Q_B,Q_L,
        45, //Q_B,Q_BL,
        0, //Q_B,Q_B,
    ],
];

/// `bg_parryDebounce[NUM_FORCE_POWER_LEVELS]` (bg_saber.c) — minimum ms between
/// parries, indexed by force-defense level (level 0 = no defense).
pub static bg_parryDebounce: [c_int; NUM_FORCE_POWER_LEVELS] = [
    500, //if don't even have defense, can't use defense!
    300,
    150,
    50,
];

/// `PM_irand_timesync` (bg_saber.c:11) — the file's first function: a deterministic
/// pseudo-random integer in `[val1, val2]`, seeded off (and mutating) the current
/// command's `serverTime` so server and client agree. Faithful: the whole expression
/// evaluates in `f32` (`Q_random` returns float; the int operands promote) then
/// truncates toward zero on the `as c_int`, matching the C `int i = ...` assignment.
/// `Q_random` writes back the advanced seed into `pm->cmd.serverTime` — preserved via
/// the `&mut` borrow. No oracle (drives the saber anim machine over the verified
/// [`Q_random`]).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_irand_timesync(val1: c_int, val2: c_int) -> c_int {
    let pmv = *addr_of!(pm);

    let mut i: c_int = ((val1 - 1) as f32
        + Q_random(&mut (*pmv).cmd.serverTime) * (val2 - val1) as f32
        + 1.0) as c_int;
    if i < val1 {
        i = val1;
    }
    if i > val2 {
        i = val2;
    }

    i
}

/// `BG_ForcePowerDrain` (bg_saber.c:28) — the file's first function: deduct the cost
/// of using `forcePower` from the player's force pool. `overrideAmt` of 0 means "use
/// the table cost" ([`forcePowerNeeded`], now ported in `bg_pmove.rs` — this is the
/// blocker the bg_pmove data-layer slice just lifted). `FP_LEVITATION` is a special
/// case: the drain scales with upward velocity (`velocity[2]`) and is divided down by
/// jump rank. Faithful raw-ptr `unsafe fn` (the C sig takes `playerState_t *`);
/// `forcePowers_t` → `c_int` (the anon-enum convention). C `!drain` → `drain == 0`.
pub unsafe fn BG_ForcePowerDrain(ps: *mut playerState_t, forcePower: c_int, overrideAmt: c_int) {
    //take away the power
    let mut drain = overrideAmt;

    /*
    if (ps->powerups[PW_FORCE_BOON])
    {
        return;
    }
    */
    //No longer grant infinite force with boon.

    if drain == 0 {
        drain = forcePowerNeeded[(*ps).fd.forcePowerLevel[forcePower as usize] as usize]
            [forcePower as usize];
    }
    if drain == 0 {
        return;
    }

    if forcePower == FP_LEVITATION {
        //special case
        let mut jumpDrain = 0;

        if (*ps).velocity[2] > 250.0 {
            jumpDrain = 20;
        } else if (*ps).velocity[2] > 200.0 {
            jumpDrain = 16;
        } else if (*ps).velocity[2] > 150.0 {
            jumpDrain = 12;
        } else if (*ps).velocity[2] > 100.0 {
            jumpDrain = 8;
        } else if (*ps).velocity[2] > 50.0 {
            jumpDrain = 6;
        } else if (*ps).velocity[2] > 0.0 {
            jumpDrain = 4;
        }

        if jumpDrain != 0 {
            if (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] != 0 {
                //don't divide by 0!
                jumpDrain /= (*ps).fd.forcePowerLevel[FP_LEVITATION as usize];
            }
        }

        (*ps).fd.forcePower -= jumpDrain;
        if (*ps).fd.forcePower < 0 {
            (*ps).fd.forcePower = 0;
        }

        return;
    }

    (*ps).fd.forcePower -= drain;
    if (*ps).fd.forcePower < 0 {
        (*ps).fd.forcePower = 0;
    }
}

/// `BG_EnoughForcePowerForMove` (bg_saber.c:103) — gate a force-costing saber move on
/// the player's current `forcePower`: if below `cost`, post an `EV_NOAMMO` and refuse.
/// Reads the `pm` keystone global. No oracle (pm-state gate over the verified
/// [`PM_AddEvent`]).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn BG_EnoughForcePowerForMove(cost: c_int) -> qboolean {
    let pmv = *addr_of!(pm);

    if (*(*pmv).ps).fd.forcePower < cost {
        PM_AddEvent(EV_NOAMMO);
        return QFALSE;
    }

    QTRUE
}

// ===================================================================
//  Self-contained move helpers (bg_saber.c). Each touches only the data
//  tables above or its plain int/usercmd_t arguments, so it ports ahead of
//  the `pm`/`pml`/`forcePowerNeeded` globals the rest of the file needs.
//  Source order is preserved; the `pm`/`pml`-reading helpers interleaved with
//  these (`PM_SaberKataDone`, `PM_SaberAnimTransitionAnim`, `PM_CheckStabDown`,
//  ...) have since landed too — see further down this file.
// ===================================================================

/// `PM_AttackMoveForQuad` (bg_saber.c) — the basic attack move that swings from
/// the given quadrant.
pub fn PM_AttackMoveForQuad(quad: c_int) -> saberMoveName_t {
    match quad {
        Q_B | Q_BR => LS_A_BR2TL,
        Q_R => LS_A_R2L,
        Q_TR => LS_A_TR2BL,
        Q_T => LS_A_T2B,
        Q_TL => LS_A_TL2BR,
        Q_L => LS_A_L2R,
        Q_BL => LS_A_BL2TR,
        _ => LS_NONE,
    }
}

/// `PM_SaberMoveQuadrantForMovement` (bg_saber.c) — maps the movement keys
/// (`ucmd->rightmove`/`forwardmove`) to the attack quadrant they trigger.
/// Faithful raw-ptr `unsafe fn` (the C sig takes `usercmd_t *`).
pub unsafe fn PM_SaberMoveQuadrantForMovement(ucmd: *mut usercmd_t) -> c_int {
    if (*ucmd).rightmove > 0 {
        //moving right
        if (*ucmd).forwardmove > 0 {
            //forward right = TL2BR slash
            Q_TL
        } else if (*ucmd).forwardmove < 0 {
            //backward right = BL2TR uppercut
            Q_BL
        } else {
            //just right is a left slice
            Q_L
        }
    } else if (*ucmd).rightmove < 0 {
        //moving left
        if (*ucmd).forwardmove > 0 {
            //forward left = TR2BL slash
            Q_TR
        } else if (*ucmd).forwardmove < 0 {
            //backward left = BR2TL uppercut
            Q_BR
        } else {
            //just left is a right slice
            Q_R
        }
    } else {
        //not moving left or right
        if (*ucmd).forwardmove > 0 {
            //forward= T2B slash
            Q_T
        } else if (*ucmd).forwardmove < 0 {
            //backward= T2B slash	//or B2T uppercut?
            Q_T
        } else {
            //Not moving at all
            Q_R
        }
    }
}

//===================================================================
/// `PM_SaberInBounce` (bg_saber.c) — is `move` one of the bounce/deflect moves?
pub fn PM_SaberInBounce(r#move: c_int) -> qboolean {
    if r#move >= LS_B1_BR && r#move <= LS_B1_BL {
        return QTRUE;
    }
    if r#move >= LS_D1_BR && r#move <= LS_D1_BL {
        return QTRUE;
    }
    QFALSE
}

/// `PM_SaberAttackChainAngle` (bg_saber.c) — the transition angle (degrees) from
/// `move1`'s end quadrant to `move2`'s start quadrant; `-1` if either is invalid.
pub fn PM_SaberAttackChainAngle(move1: c_int, move2: c_int) -> c_int {
    if move1 == -1 || move2 == -1 {
        return -1;
    }
    unsafe {
        let smd = &*core::ptr::addr_of!(saberMoveData);
        saberMoveTransitionAngle[smd[move1 as usize].endQuad as usize]
            [smd[move2 as usize].startQuad as usize]
    }
}

/// `PM_SetAnimFrame` (bg_saber.c) — record the current saber-lock frame on the
/// player state. `torso`/`legs` are part of the C signature but the body uses
/// only `gent`/`frame` (we can't actually query the live anim frame), hence the
/// fn-scoped `#[allow(unused_variables)]`. Faithful raw-ptr `unsafe fn`.
#[allow(unused_variables)]
pub unsafe fn PM_SetAnimFrame(
    gent: *mut playerState_t,
    frame: c_int,
    torso: qboolean,
    legs: qboolean,
) {
    (*gent).saberLockFrame = frame;
}

/// `BG_CheckIncrementLockAnim` (bg_saber.c) — given a saber-lock anim and which
/// side this is for (`SABERLOCK_WIN`/`SABERLOCK_LOSE`), should the lock position
/// advance?
//RULE: if you are the first style in the lock anim, you advance from LOSING position to WINNING position
//		if you are the second style in the lock anim, you advance from WINNING position to LOSING position
pub fn BG_CheckIncrementLockAnim(anim: c_int, winOrLose: c_int) -> qboolean {
    let mut increment = QFALSE; //???
    match anim {
        //increment to win:
        BOTH_LK_DL_DL_S_L_1 //lock if I'm using dual vs. dual and I initiated
        | BOTH_LK_DL_DL_S_L_2 //lock if I'm using dual vs. dual and other initiated
        | BOTH_LK_DL_DL_T_L_1 //lock if I'm using dual vs. dual and I initiated
        | BOTH_LK_DL_DL_T_L_2 //lock if I'm using dual vs. dual and other initiated
        | BOTH_LK_DL_S_S_L_1 //lock if I'm using dual vs. a single
        | BOTH_LK_DL_S_T_L_1 //lock if I'm using dual vs. a single
        | BOTH_LK_DL_ST_S_L_1 //lock if I'm using dual vs. a staff
        | BOTH_LK_DL_ST_T_L_1 //lock if I'm using dual vs. a staff
        | BOTH_LK_S_S_S_L_1 //lock if I'm using single vs. a single and I initiated
        | BOTH_LK_S_S_T_L_2 //lock if I'm using single vs. a single and other initiated
        | BOTH_LK_ST_S_S_L_1 //lock if I'm using staff vs. a single
        | BOTH_LK_ST_S_T_L_1 //lock if I'm using staff vs. a single
        | BOTH_LK_ST_ST_T_L_1 //lock if I'm using staff vs. a staff and I initiated
        | BOTH_LK_ST_ST_T_L_2 => {
            //lock if I'm using staff vs. a staff and other initiated
            if winOrLose == SABERLOCK_WIN {
                increment = QTRUE;
            } else {
                increment = QFALSE;
            }
        }

        //decrement to win:
        BOTH_LK_S_DL_S_L_1 //lock if I'm using single vs. a dual
        | BOTH_LK_S_DL_T_L_1 //lock if I'm using single vs. a dual
        | BOTH_LK_S_S_S_L_2 //lock if I'm using single vs. a single and other intitiated
        | BOTH_LK_S_S_T_L_1 //lock if I'm using single vs. a single and I initiated
        | BOTH_LK_S_ST_S_L_1 //lock if I'm using single vs. a staff
        | BOTH_LK_S_ST_T_L_1 //lock if I'm using single vs. a staff
        | BOTH_LK_ST_DL_S_L_1 //lock if I'm using staff vs. dual
        | BOTH_LK_ST_DL_T_L_1 //lock if I'm using staff vs. dual
        | BOTH_LK_ST_ST_S_L_1 //lock if I'm using staff vs. a staff and I initiated
        | BOTH_LK_ST_ST_S_L_2 => {
            //lock if I'm using staff vs. a staff and other initiated
            if winOrLose == SABERLOCK_WIN {
                increment = QFALSE;
            } else {
                increment = QTRUE;
            }
        }
        _ => {}
    }
    increment
}

/// `PM_SaberInBrokenParry` (bg_saber.c) — is `move` a reflected-attack (`LS_V1_*`)
/// or broken-parry (`LS_H1_*`) move?
pub fn PM_SaberInBrokenParry(r#move: c_int) -> qboolean {
    if r#move >= LS_V1_BR && r#move <= LS_V1_B_ {
        return QTRUE;
    }
    if r#move >= LS_H1_T_ && r#move <= LS_H1_BL {
        return QTRUE;
    }
    QFALSE
}

/// `PM_BrokenParryForParry` (bg_saber.c) — the broken-parry (`LS_H1_*`) move that
/// a given parry move breaks into.
pub fn PM_BrokenParryForParry(r#move: c_int) -> c_int {
    match r#move {
        LS_PARRY_UP => LS_H1_T_,
        LS_PARRY_UR => LS_H1_TR,
        LS_PARRY_UL => LS_H1_TL,
        LS_PARRY_LR => LS_H1_BL,
        LS_PARRY_LL => LS_H1_BR,
        LS_READY => LS_H1_B_,
        _ => LS_NONE,
    }
}

// `PM_GroundDistance` / `PM_WalkableGroundDistance` (bg_saber.c:1788/1804) —
// pulled in out of source order (the data-table/self-contained-helper slice
// above stops at ~722) because they are `extern`-declared and consumed by the
// `bg_pmove.c` `pm` keystone (PM_CheckJump and the saber jump-attack code). Both
// drop a 4096-unit trace straight down through `pm->trace` and return the
// distance to whatever it hit; the walkable variant returns the full 4096 if the
// surface is too steep to stand on. Trace/callback-driven, so no bit-exact oracle
// (the PM_CorrectAllSolid/PM_GroundTrace precedent) — faithful transcription.

/// `float PM_GroundDistance( void )` (bg_saber.c:1788).
pub unsafe fn PM_GroundDistance() -> f32 {
    let pmv = *addr_of!(pm);
    let mut tr: trace_t = core::mem::zeroed();
    let mut down: vec3_t = [0.0; 3];

    VectorCopy(&(*(*pmv).ps).origin, &mut down);

    down[2] -= 4096.0;

    ((*pmv).trace.unwrap())(
        &mut tr,
        (*(*pmv).ps).origin.as_ptr(),
        (*pmv).mins.as_ptr(),
        (*pmv).maxs.as_ptr(),
        down.as_ptr(),
        (*(*pmv).ps).clientNum,
        MASK_SOLID,
    );

    VectorSubtract(&(*(*pmv).ps).origin, &tr.endpos, &mut down);

    VectorLength(&down)
}

/// `float PM_WalkableGroundDistance( void )` (bg_saber.c:1804).
pub unsafe fn PM_WalkableGroundDistance() -> f32 {
    let pmv = *addr_of!(pm);
    let mut tr: trace_t = core::mem::zeroed();
    let mut down: vec3_t = [0.0; 3];

    VectorCopy(&(*(*pmv).ps).origin, &mut down);

    down[2] -= 4096.0;

    ((*pmv).trace.unwrap())(
        &mut tr,
        (*(*pmv).ps).origin.as_ptr(),
        (*pmv).mins.as_ptr(),
        (*pmv).maxs.as_ptr(),
        down.as_ptr(),
        (*(*pmv).ps).clientNum,
        MASK_SOLID,
    );

    if tr.plane.normal[2] < MIN_WALK_NORMAL {
        //can't stand on this plane
        return 4096.0;
    }

    VectorSubtract(&(*(*pmv).ps).origin, &tr.endpos, &mut down);

    VectorLength(&down)
}

// ===================================================================
//  Saber-attack state machine — `PM_WeaponLightsaber`'s dependency tree
//  (bg_saber.c). These `pm`/`pml`-reading helpers drive saber attacks,
//  kicks, katas and saber-locks. Ported leaves-first (dependency order
//  rather than strict C source order — the file's existing helper-section
//  grouping precedent); each carries its bg_saber.c line in its doc.
// ===================================================================

/// `PM_InSecondaryStyle` (bg_saber.c:2006) — is the player attacking from their saber
/// style's *secondary* stance (a staff/dual wielder whose active `saberAnimLevel` has
/// diverged from its `saberAnimLevelBase`)? Pure `pm`-state predicate. No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_InSecondaryStyle() -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    if (*ps).fd.saberAnimLevelBase == SS_STAFF || (*ps).fd.saberAnimLevelBase == SS_DUAL {
        if (*ps).fd.saberAnimLevel != (*ps).fd.saberAnimLevelBase {
            return QTRUE;
        }
    }
    QFALSE
}

/// `PM_SaberMoveOkayForKata` (bg_saber.c:2408) — may a kata special be initiated from
/// the current saber move? Only from `LS_READY` or a start-transition. No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberMoveOkayForKata() -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    if (*ps).saberMove == LS_READY || PM_SaberInStart((*ps).saberMove) != QFALSE {
        QTRUE
    } else {
        QFALSE
    }
}

/// `PM_CheckAltKickAttack` (bg_saber.c:2449) — should an alt-attack press launch a staff
/// kick? Requires the alt-attack button, not mid-flip (or the flip nearly done), the
/// `SS_STAFF` style, and the saber not holstered. No oracle. (The C's `PMF_ALT_ATTACK_HELD`
/// and `saber[0].throwable` guards are already `//`-commented in the original.)
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_CheckAltKickAttack() -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0
        //&& (!(pm->ps->pm_flags&PMF_ALT_ATTACK_HELD)||PM_SaberInReturn(pm->ps->saberMove))
        && (BG_FlippingAnim((*ps).legsAnim) == QFALSE || (*ps).legsTimer <= 250)
        && (*ps).fd.saberAnimLevel == SS_STAFF/*||!pm->ps->saber[0].throwable*/
        && (*ps).saberHolstered == 0
    {
        return QTRUE;
    }
    QFALSE
}

/// `PM_CanDoDualDoubleAttacks` (bg_saber.c:1826) — guard for the dual-saber left/right
/// and front/back double attacks: disallowed while already mid special-attack anim
/// (torso or legs). C `static`; ported `pub` per this file's helper convention. No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_CanDoDualDoubleAttacks() -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    if (*ps).weapon == WP_SABER {
        let mut saber: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 0);
        if !saber.is_null() && (*saber).saberFlags & SFL_NO_MIRROR_ATTACKS != 0 {
            return QFALSE;
        }
        saber = BG_MySaber((*ps).clientNum, 1);
        if !saber.is_null() && (*saber).saberFlags & SFL_NO_MIRROR_ATTACKS != 0 {
            return QFALSE;
        }
    }
    if BG_SaberInSpecialAttack((*ps).torsoAnim) != QFALSE
        || BG_SaberInSpecialAttack((*ps).legsAnim) != QFALSE
    {
        return QFALSE;
    }
    QTRUE
}

/// `PM_CheckEnemyPresence` (bg_saber.c:1836) — is anyone in the given cardinal `dir`
/// within `radius`? A single box trace in that direction (the SP code walks a bbox ent
/// list, but that's not cheap in predicted code). C `static`; ported `pub` per this
/// file's helper convention. No oracle (trace-driven).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_CheckEnemyPresence(dir: c_int, radius: f32) -> qboolean {
    //anyone in this dir?
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut angles: vec3_t = [0.0; 3];
    let mut checkDir: vec3_t = [0.0; 3];
    let mut tTo: vec3_t = [0.0; 3];
    let mut tMins: vec3_t = [0.0; 3];
    let mut tMaxs: vec3_t = [0.0; 3];
    let mut tr: trace_t = core::mem::zeroed();
    let tSize: f32 = 12.0;
    //sp uses a bbox ent list check, but.. that's not so easy/fast to
    //do in predicted code. So I'll just do a single box trace in the proper direction,
    //and take whatever is first hit.

    VectorSet(&mut tMins, -tSize, -tSize, -tSize);
    VectorSet(&mut tMaxs, tSize, tSize, tSize);

    VectorCopy(&(*ps).viewangles, &mut angles);
    angles[PITCH] = 0.0;

    match dir {
        DIR_RIGHT => {
            AngleVectors(&angles, None, Some(&mut checkDir), None);
        }
        DIR_LEFT => {
            AngleVectors(&angles, None, Some(&mut checkDir), None);
            let tmp = checkDir;
            VectorScale(&tmp, -1.0, &mut checkDir);
        }
        DIR_FRONT => {
            AngleVectors(&angles, Some(&mut checkDir), None, None);
        }
        DIR_BACK => {
            AngleVectors(&angles, Some(&mut checkDir), None, None);
            let tmp = checkDir;
            VectorScale(&tmp, -1.0, &mut checkDir);
        }
        _ => {}
    }

    VectorMA(&(*ps).origin, radius, &checkDir, &mut tTo);
    ((*pmv).trace.unwrap())(
        &mut tr,
        (*ps).origin.as_ptr(),
        tMins.as_ptr(),
        tMaxs.as_ptr(),
        tTo.as_ptr(),
        (*ps).clientNum,
        MASK_PLAYERSOLID,
    );

    if tr.fraction != 1.0 && (tr.entityNum as c_int) < ENTITYNUM_WORLD {
        //let's see who we hit
        let bgEnt = PM_BGEntForNum(tr.entityNum as c_int);

        if !bgEnt.is_null()
            && ((*bgEnt).s.eType == ET_PLAYER || (*bgEnt).s.eType == ET_NPC)
        {
            //this guy can be considered an "enemy"... if he is on the same team, oh well.
            return QTRUE;
        }
    }

    //no one in the trace
    QFALSE
}

/// `BACK_STAB_DISTANCE` (bg_saber.c:1606) — reach of the backstab rear box trace.
const BACK_STAB_DISTANCE: f32 = 128.0;

/// `PM_CanBackstab` (bg_saber.c:1608) — is there a player/NPC close behind us (within
/// [`BACK_STAB_DISTANCE`])? A box trace straight back along the eye-flat forward vector.
/// No oracle (trace-driven).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_CanBackstab() -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut tr: trace_t = core::mem::zeroed();
    let mut flatAng: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut back: vec3_t = [0.0; 3];
    let trmins: vec3_t = [-15.0, -15.0, -8.0];
    let trmaxs: vec3_t = [15.0, 15.0, 8.0];

    VectorCopy(&(*ps).viewangles, &mut flatAng);
    flatAng[PITCH] = 0.0;

    AngleVectors(&flatAng, Some(&mut fwd), None, None);

    back[0] = (*ps).origin[0] - fwd[0] * BACK_STAB_DISTANCE;
    back[1] = (*ps).origin[1] - fwd[1] * BACK_STAB_DISTANCE;
    back[2] = (*ps).origin[2] - fwd[2] * BACK_STAB_DISTANCE;

    ((*pmv).trace.unwrap())(
        &mut tr,
        (*ps).origin.as_ptr(),
        trmins.as_ptr(),
        trmaxs.as_ptr(),
        back.as_ptr(),
        (*ps).clientNum,
        MASK_PLAYERSOLID,
    );

    if tr.fraction != 1.0 && tr.entityNum >= 0 && (tr.entityNum as c_int) < ENTITYNUM_NONE {
        let bgEnt = PM_BGEntForNum(tr.entityNum as c_int);

        if !bgEnt.is_null() && ((*bgEnt).s.eType == ET_PLAYER || (*bgEnt).s.eType == ET_NPC) {
            return QTRUE;
        }
    }

    QFALSE
}

/// `PM_CheckStabDown` (bg_saber.c:584) — if a knocked-down player/NPC is right in front
/// (a 164-unit box trace ahead), return the style-appropriate downward stab move
/// (`LS_STABDOWN`/`_STAFF`/`_DUAL`), else `LS_NONE`. First bails to `LS_NONE` if either
/// of the client's sabers ([`BG_MySaber`]) carries `SFL_NO_STABDOWN`. Players have their
/// vertical velocity and jump intent zeroed first. No oracle (trace-driven).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_CheckStabDown() -> saberMoveName_t {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut faceFwd: vec3_t = [0.0; 3];
    let mut facingAngles: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut ent: *mut bgEntity_t = core::ptr::null_mut();
    let mut tr: trace_t = core::mem::zeroed();
    //yeah, vm's may complain, but.. who cares!
    let trmins: vec3_t = [-15.0, -15.0, -15.0];
    let trmaxs: vec3_t = [15.0, 15.0, 15.0];

    let saber1: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 0);
    let saber2: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 1);
    if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_STABDOWN != 0 {
        return LS_NONE;
    }
    if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_STABDOWN != 0 {
        return LS_NONE;
    }

    if (*ps).groundEntityNum == ENTITYNUM_NONE {
        //sorry must be on ground!
        return LS_NONE;
    }
    if (*ps).clientNum < MAX_CLIENTS as c_int {
        //player
        (*ps).velocity[2] = 0.0;
        (*pmv).cmd.upmove = 0;
    }

    VectorSet(&mut facingAngles, 0.0, (*ps).viewangles[YAW], 0.0);
    AngleVectors(&facingAngles, Some(&mut faceFwd), None, None);

    //FIXME: need to only move forward until we bump into our target...?
    VectorMA(&(*ps).origin, 164.0, &faceFwd, &mut fwd);

    ((*pmv).trace.unwrap())(
        &mut tr,
        (*ps).origin.as_ptr(),
        trmins.as_ptr(),
        trmaxs.as_ptr(),
        fwd.as_ptr(),
        (*ps).clientNum,
        MASK_PLAYERSOLID,
    );

    if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
        ent = PM_BGEntForNum(tr.entityNum as c_int);
    }

    if !ent.is_null()
        && ((*ent).s.eType == ET_PLAYER || (*ent).s.eType == ET_NPC)
        && BG_InKnockDown((*ent).s.legsAnim) != QFALSE
    {
        //guy is on the ground below me, do a top-down attack
        if (*ps).fd.saberAnimLevel == SS_DUAL {
            return LS_STABDOWN_DUAL;
        } else if (*ps).fd.saberAnimLevel == SS_STAFF {
            return LS_STABDOWN_STAFF;
        } else {
            return LS_STABDOWN;
        }
    }
    LS_NONE
}

/// `PM_SaberKataDone` (bg_saber.c:780) — has the current attack kata reached its chain
/// limit, forcing a return to ready before the next swing? Staff/dual/Desann/Tavion chain
/// infinitely; `FORCE_LEVEL_3` decides by chain count and the [`PM_SaberAttackChainAngle`]
/// momentum between `curmove`→`newmove`; fast/medium use a per-level `chainTolerance`
/// against a [`PM_irand_timesync`] roll. No oracle (drives the saber FSM over the verified
/// random + chain-angle helpers).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberKataDone(curmove: c_int, newmove: c_int) -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    if (*ps).m_iVehicleNum != 0 {
        //never continue kata on vehicle
        if (*ps).saberAttackChainCount > 0 {
            return QTRUE;
        }
    }

    if (*ps).fd.saberAnimLevel == SS_DESANN || (*ps).fd.saberAnimLevel == SS_TAVION {
        //desann and tavion can link up as many attacks as they want
        return QFALSE;
    }

    if (*ps).fd.saberAnimLevel == SS_STAFF {
        //TEMP: for now, let staff attacks infinitely chain
        QFALSE
    } else if (*ps).fd.saberAnimLevel == SS_DUAL {
        //TEMP: for now, let staff attacks infinitely chain
        QFALSE
    } else if (*ps).fd.saberAnimLevel == FORCE_LEVEL_3 {
        if curmove == LS_NONE || newmove == LS_NONE {
            if (*ps).fd.saberAnimLevel >= FORCE_LEVEL_3
                && (*ps).saberAttackChainCount > PM_irand_timesync(0, 1)
            {
                return QTRUE;
            }
        } else if (*ps).saberAttackChainCount > PM_irand_timesync(2, 3) {
            return QTRUE;
        } else if (*ps).saberAttackChainCount > 0 {
            let chainAngle: c_int = PM_SaberAttackChainAngle(curmove, newmove);
            if chainAngle < 135 || chainAngle > 215 {
                //if trying to chain to a move that doesn't continue the momentum
                return QTRUE;
            } else if chainAngle == 180 {
                //continues the momentum perfectly, allow it to chain 66% of the time
                if (*ps).saberAttackChainCount > 1 {
                    return QTRUE;
                }
            } else {
                //would continue the movement somewhat, 50% chance of continuing
                if (*ps).saberAttackChainCount > 2 {
                    return QTRUE;
                }
            }
        }
        QFALSE
    } else {
        //Perhaps have chainAngle influence fast and medium chains as well? For now, just do level 3.
        if newmove == LS_A_TL2BR
            || newmove == LS_A_L2R
            || newmove == LS_A_BL2TR
            || newmove == LS_A_BR2TL
            || newmove == LS_A_R2L
            || newmove == LS_A_TR2BL
        {
            //lower chaining tolerance for spinning saber anims
            let chainTolerance: c_int = if (*ps).fd.saberAnimLevel == FORCE_LEVEL_1 {
                5
            } else {
                3
            };

            if (*ps).saberAttackChainCount >= chainTolerance
                && PM_irand_timesync(1, (*ps).saberAttackChainCount) > chainTolerance
            {
                return QTRUE;
            }
        }
        if (*ps).fd.saberAnimLevel == FORCE_LEVEL_2
            && (*ps).saberAttackChainCount > PM_irand_timesync(2, 5)
        {
            return QTRUE;
        }
        QFALSE
    }
}

/// `PM_SaberFlipOverAttackMove` (bg_saber.c:1640) — launch the medium-style flip-over
/// downward attack: fling forward+up, post a jump, and return the flip-slash move. No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberFlipOverAttackMove() -> saberMoveName_t {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut fwdAngles: vec3_t = [0.0; 3];
    let mut jumpFwd: vec3_t = [0.0; 3];
    //	float zDiff = 0;
    //	playerState_t *psData;
    //	bgEntity_t *bgEnt;

    let saber1: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 0);
    let saber2: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 1);
    //see if we have an overridden (or cancelled) lunge move
    if !saber1.is_null() && (*saber1).jumpAtkFwdMove != LS_INVALID {
        if (*saber1).jumpAtkFwdMove != LS_NONE {
            return (*saber1).jumpAtkFwdMove as saberMoveName_t;
        }
    }
    if !saber2.is_null() && (*saber2).jumpAtkFwdMove != LS_INVALID {
        if (*saber2).jumpAtkFwdMove != LS_NONE {
            return (*saber2).jumpAtkFwdMove as saberMoveName_t;
        }
    }
    //no overrides, cancelled?
    if !saber1.is_null() && (*saber1).jumpAtkFwdMove == LS_NONE {
        return LS_A_T2B; //LS_NONE;
    }
    if !saber2.is_null() && (*saber2).jumpAtkFwdMove == LS_NONE {
        return LS_A_T2B; //LS_NONE;
    }

    VectorCopy(&(*ps).viewangles, &mut fwdAngles);
    fwdAngles[PITCH] = 0.0;
    fwdAngles[ROLL] = 0.0;
    AngleVectors(&fwdAngles, Some(&mut jumpFwd), None, None);
    VectorScale(&jumpFwd, 150.0, &mut (*ps).velocity); //was 50
    (*ps).velocity[2] = 400.0;

    /*
    bgEnt = PM_BGEntForNum(tr->entityNum);

    if (!bgEnt)
    {
        return LS_A_FLIP_STAB;
    }

    psData = bgEnt->playerState;

    //go higher for enemies higher than you, lower for those lower than you
    if (psData)
    {
        zDiff = psData->origin[2] - pm->ps->origin[2];
    }
    else
    {
        zDiff = 0;
    }
    pm->ps->velocity[2] += (zDiff)*1.5f;

    //clamp to decent-looking values
    if ( zDiff <= 0 && pm->ps->velocity[2] < 200 )
    {//if we're on same level, don't let me jump so low, I clip into the ground
        pm->ps->velocity[2] = 200;
    }
    else if ( pm->ps->velocity[2] < 100 )
    {
        pm->ps->velocity[2] = 100;
    }
    else if ( pm->ps->velocity[2] > 400 )
    {
        pm->ps->velocity[2] = 400;
    }
    */

    PM_SetForceJumpZStart((*ps).origin[2]); //so we don't take damage if we land at same height

    PM_AddEvent(EV_JUMP);
    (*ps).fd.forceJumpSound = 1;
    (*pmv).cmd.upmove = 0;

    /*
    if ( PM_irand_timesync( 0, 1 ) )
    {
        return LS_A_FLIP_STAB;
    }
    else
    */
    {
        LS_A_FLIP_SLASH
    }
}

/// `PM_SaberBackflipAttackMove` (bg_saber.c:1707) — launch the backflip attack: jump
/// straight up hard and return `LS_A_BACKFLIP_ATK`. No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberBackflipAttackMove() -> c_int {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let saber1: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 0);
    let saber2: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 1);
    //see if we have an overridden (or cancelled) lunge move
    if !saber1.is_null() && (*saber1).jumpAtkBackMove != LS_INVALID {
        if (*saber1).jumpAtkBackMove != LS_NONE {
            return (*saber1).jumpAtkBackMove as saberMoveName_t;
        }
    }
    if !saber2.is_null() && (*saber2).jumpAtkBackMove != LS_INVALID {
        if (*saber2).jumpAtkBackMove != LS_NONE {
            return (*saber2).jumpAtkBackMove as saberMoveName_t;
        }
    }
    //no overrides, cancelled?
    if !saber1.is_null() && (*saber1).jumpAtkBackMove == LS_NONE {
        return LS_A_T2B; //LS_NONE;
    }
    if !saber2.is_null() && (*saber2).jumpAtkBackMove == LS_NONE {
        return LS_A_T2B; //LS_NONE;
    }
    //just do it
    (*pmv).cmd.upmove = 127;
    (*ps).velocity[2] = 500.0;
    LS_A_BACKFLIP_ATK
}

/// `PM_SaberDualJumpAttackMove` (bg_saber.c:1714) — begin the dual-saber jump attack;
/// suppresses the jump for this frame (the anim delays it). No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberDualJumpAttackMove() -> c_int {
    //FIXME: to make this move easier to execute, should be allowed to do it
    //		after you've already started your jump... but jump is delayed in
    //		this anim, so how do we undo the jump?
    let pmv = *addr_of!(pm);
    (*pmv).cmd.upmove = 0; //no jump just yet
    LS_JUMPATTACK_DUAL
}

/// `FLIPHACK_DISTANCE` (bg_saber.c:1723) — reach of the front box trace in
/// [`PM_SomeoneInFront`].
const FLIPHACK_DISTANCE: f32 = 200.0;

/// `PM_SomeoneInFront` (bg_saber.c:1725) — is there a player/NPC ahead within
/// [`FLIPHACK_DISTANCE`]? A box trace straight forward along the eye-flat forward vector,
/// writing the result into `*tr`. The MP-simplified counterpart of the SP version. No
/// oracle (trace-driven).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`; `tr` must point to a writable `trace_t`.
pub unsafe fn PM_SomeoneInFront(tr: *mut trace_t) -> qboolean {
    //Also a very simplified version of the sp counterpart
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut flatAng: vec3_t = [0.0; 3];
    let mut fwd: vec3_t = [0.0; 3];
    let mut back: vec3_t = [0.0; 3];
    let trmins: vec3_t = [-15.0, -15.0, -8.0];
    let trmaxs: vec3_t = [15.0, 15.0, 8.0];

    VectorCopy(&(*ps).viewangles, &mut flatAng);
    flatAng[PITCH] = 0.0;

    AngleVectors(&flatAng, Some(&mut fwd), None, None);

    back[0] = (*ps).origin[0] + fwd[0] * FLIPHACK_DISTANCE;
    back[1] = (*ps).origin[1] + fwd[1] * FLIPHACK_DISTANCE;
    back[2] = (*ps).origin[2] + fwd[2] * FLIPHACK_DISTANCE;

    ((*pmv).trace.unwrap())(
        tr,
        (*ps).origin.as_ptr(),
        trmins.as_ptr(),
        trmaxs.as_ptr(),
        back.as_ptr(),
        (*ps).clientNum,
        MASK_PLAYERSOLID,
    );

    if (*tr).fraction != 1.0 && (*tr).entityNum >= 0 && ((*tr).entityNum as c_int) < ENTITYNUM_NONE
    {
        let bgEnt = PM_BGEntForNum((*tr).entityNum as c_int);

        if !bgEnt.is_null() && ((*bgEnt).s.eType == ET_PLAYER || (*bgEnt).s.eType == ET_NPC) {
            return QTRUE;
        }
    }

    QFALSE
}

/// `PM_SaberLungeAttackMove` (bg_saber.c:1756) — launch the lunge: a flat forward burst
/// plus a jump event, returning `LS_A_LUNGE`. No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
// TODO: Remove-Xbox
pub unsafe fn PM_SaberLungeAttackMove() -> saberMoveName_t {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut fwdAngles: vec3_t = [0.0; 3];
    let mut jumpFwd: vec3_t = [0.0; 3];

    VectorCopy(&(*ps).viewangles, &mut fwdAngles);
    fwdAngles[PITCH] = 0.0;
    fwdAngles[ROLL] = 0.0;
    //do the lunge
    AngleVectors(&fwdAngles, Some(&mut jumpFwd), None, None);
    VectorScale(&jumpFwd, 150.0, &mut (*ps).velocity);
    PM_AddEvent(EV_JUMP);

    LS_A_LUNGE
}

/// `PM_SaberJumpAttackMove` (bg_saber.c:1770) — launch the strong-style death-from-above
/// jump attack: forward+up burst, jump event, return `LS_A_JUMP_T__B_`. No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberJumpAttackMove() -> saberMoveName_t {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut fwdAngles: vec3_t = [0.0; 3];
    let mut jumpFwd: vec3_t = [0.0; 3];

    let saber1: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 0);
    let saber2: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 1);
    //see if we have an overridden (or cancelled) lunge move
    if !saber1.is_null() && (*saber1).jumpAtkFwdMove != LS_INVALID {
        if (*saber1).jumpAtkFwdMove != LS_NONE {
            return (*saber1).jumpAtkFwdMove as saberMoveName_t;
        }
    }
    if !saber2.is_null() && (*saber2).jumpAtkFwdMove != LS_INVALID {
        if (*saber2).jumpAtkFwdMove != LS_NONE {
            return (*saber2).jumpAtkFwdMove as saberMoveName_t;
        }
    }
    //no overrides, cancelled?
    if !saber1.is_null() && (*saber1).jumpAtkFwdMove == LS_NONE {
        return LS_A_T2B; //LS_NONE;
    }
    if !saber2.is_null() && (*saber2).jumpAtkFwdMove == LS_NONE {
        return LS_A_T2B; //LS_NONE;
    }

    VectorCopy(&(*ps).viewangles, &mut fwdAngles);
    fwdAngles[PITCH] = 0.0;
    fwdAngles[ROLL] = 0.0;
    AngleVectors(&fwdAngles, Some(&mut jumpFwd), None, None);
    VectorScale(&jumpFwd, 300.0, &mut (*ps).velocity);
    (*ps).velocity[2] = 280.0;
    PM_SetForceJumpZStart((*ps).origin[2]); //so we don't take damage if we land at same height

    PM_AddEvent(EV_JUMP);
    (*ps).fd.forceJumpSound = 1;
    (*pmv).cmd.upmove = 0;

    LS_A_JUMP_T__B_
}

/// `PM_SaberJumpAttackMove2` (bg_saber.c:1891) — pick the jump+forward+attack move,
/// honouring per-saber `jumpAtkFwdMove` overrides ([`BG_MySaber`]): a non-`LS_NONE`
/// override on either saber wins; a cancelled (`LS_NONE`) override forces `LS_A_T2B`;
/// otherwise dual style delegates to [`PM_SaberDualJumpAttackMove`] and staff style
/// returns `LS_JUMPATTACK_STAFF_RIGHT`. The oracle covers the pure `(saber0, saber1,
/// saberAnimLevel)` decision via an int-marshalling wrapper (`BG_MySaber`'s live
/// `g_entities` resolution is modelled by per-saber present/`jumpAtkFwdMove` pairs).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`; `pm->ps->clientNum` a valid `g_entities` index.
pub unsafe fn PM_SaberJumpAttackMove2() -> saberMoveName_t {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let saber1: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 0);
    let saber2: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 1);
    //see if we have an overridden (or cancelled) lunge move
    if !saber1.is_null() && (*saber1).jumpAtkFwdMove != LS_INVALID {
        if (*saber1).jumpAtkFwdMove != LS_NONE {
            return (*saber1).jumpAtkFwdMove as saberMoveName_t;
        }
    }
    if !saber2.is_null() && (*saber2).jumpAtkFwdMove != LS_INVALID {
        if (*saber2).jumpAtkFwdMove != LS_NONE {
            return (*saber2).jumpAtkFwdMove as saberMoveName_t;
        }
    }
    //no overrides, cancelled?
    if !saber1.is_null() && (*saber1).jumpAtkFwdMove == LS_NONE {
        return LS_A_T2B; //LS_NONE;
    }
    if !saber2.is_null() && (*saber2).jumpAtkFwdMove == LS_NONE {
        return LS_A_T2B; //LS_NONE;
    }
    //just do it
    if (*ps).fd.saberAnimLevel == SS_DUAL {
        PM_SaberDualJumpAttackMove()
    } else {
        //rwwFIXMEFIXME I don't like randomness for this sort of thing, gives people reason to
        //complain combat is unpredictable. Maybe do something more clever to determine
        //if we should do a left or right?
        /*
        if (PM_irand_timesync(0, 1))
        {
            newmove = LS_JUMPATTACK_STAFF_LEFT;
        }
        else
        */
        {
            LS_JUMPATTACK_STAFF_RIGHT
        }
    }
}

/// `PM_SaberLockWinAnim` (bg_saber.c:879) — for the saber-lock *winner* (us), pick and
/// play the break/win animation appropriate to our current lock anim (the old BF/circle
/// lock system; new-system locks fall through to the caller). Returns the chosen anim, or
/// `-1` if none. No oracle (anim FSM over the verified [`PM_SetAnim`]).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberLockWinAnim(victory: qboolean, superBreak: qboolean) -> c_int {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut winAnim: c_int = -1;
    match (*ps).torsoAnim {
        /*
            default:
        #ifndef FINAL_BUILD
            Com_Printf( S_COLOR_RED"ERROR-PM_SaberLockBreak: %s not in saberlock anim, anim = (%d)%s\n", pm->gent->NPC_type, pm->ps->torsoAnim, animTable[pm->ps->torsoAnim].name );
        #endif
        */
        BOTH_BF2LOCK => {
            if superBreak != QFALSE {
                winAnim = BOTH_LK_S_S_T_SB_1_W;
            } else if victory == QFALSE {
                winAnim = BOTH_BF1BREAK;
            } else {
                (*ps).saberMove = LS_A_T2B;
                winAnim = BOTH_A3_T__B_;
            }
        }
        BOTH_BF1LOCK => {
            if superBreak != QFALSE {
                winAnim = BOTH_LK_S_S_T_SB_1_W;
            } else if victory == QFALSE {
                winAnim = BOTH_KNOCKDOWN4;
            } else {
                (*ps).saberMove = LS_K1_T_;
                winAnim = BOTH_K1_S1_T_;
            }
        }
        BOTH_CWCIRCLELOCK => {
            if superBreak != QFALSE {
                winAnim = BOTH_LK_S_S_S_SB_1_W;
            } else if victory == QFALSE {
                (*ps).saberMove = LS_V1_BL; //pm->ps->saberBounceMove =
                (*ps).saberBlocked = BLOCKED_PARRY_BROKEN;
                winAnim = BOTH_V1_BL_S1;
            } else {
                winAnim = BOTH_CWCIRCLEBREAK;
            }
        }
        BOTH_CCWCIRCLELOCK => {
            if superBreak != QFALSE {
                winAnim = BOTH_LK_S_S_S_SB_1_W;
            } else if victory == QFALSE {
                (*ps).saberMove = LS_V1_BR; //pm->ps->saberBounceMove =
                (*ps).saberBlocked = BLOCKED_PARRY_BROKEN;
                winAnim = BOTH_V1_BR_S1;
            } else {
                winAnim = BOTH_CCWCIRCLEBREAK;
            }
        }
        _ => {
            //must be using new system:
        }
    }
    if winAnim != -1 {
        PM_SetAnim(
            SETANIM_BOTH,
            winAnim,
            (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
            0,
        );
        (*ps).weaponTime = (*ps).torsoTimer;
        (*ps).saberBlocked = BLOCKED_NONE;
        (*ps).weaponstate = WEAPON_FIRING;
        /*
        if ( superBreak
            && winAnim != BOTH_LK_ST_DL_T_SB_1_W )
        {//going to attack with saber, do a saber trail
            pm->ps->SaberActivateTrail( 200 );
        }
        */
    }
    winAnim
}

/// `PM_SaberLockLoseAnim` (bg_saber.c:986) — for the saber-lock *loser* (`genemy`), pick
/// and apply the break/lose animation for their current lock anim. We are the `QAGAME`
/// build, so the `#ifdef QAGAME` branch is taken: its `weaponTime` write and the
/// `NPC_SetAnim(&g_entities[...])` call (recovering the entity from `genemy->clientNum`).
/// Returns the chosen anim, or `-1`. No oracle.
///
/// # Safety
/// `genemy` must point to a valid `playerState_t`.
pub unsafe fn PM_SaberLockLoseAnim(
    genemy: *mut playerState_t,
    victory: qboolean,
    superBreak: qboolean,
) -> c_int {
    let mut loseAnim: c_int = -1;
    match (*genemy).torsoAnim {
        /*
            default:
        #ifndef FINAL_BUILD
            Com_Printf( S_COLOR_RED"ERROR-PM_SaberLockBreak: %s not in saberlock anim, anim = (%d)%s\n", genemy->NPC_type, genemy->client->ps.torsoAnim, animTable[genemy->client->ps.torsoAnim].name );
        #endif
        */
        BOTH_BF2LOCK => {
            if superBreak != QFALSE {
                loseAnim = BOTH_LK_S_S_T_SB_1_L;
            } else if victory == QFALSE {
                loseAnim = BOTH_BF1BREAK;
            } else if victory == QFALSE {
                //no-one won
                (*genemy).saberMove = LS_K1_T_;
                loseAnim = BOTH_K1_S1_T_;
            } else {
                //FIXME: this anim needs to transition back to ready when done
                loseAnim = BOTH_BF1BREAK;
            }
        }
        BOTH_BF1LOCK => {
            if superBreak != QFALSE {
                loseAnim = BOTH_LK_S_S_T_SB_1_L;
            } else if victory == QFALSE {
                loseAnim = BOTH_KNOCKDOWN4;
            } else if victory == QFALSE {
                //no-one won
                (*genemy).saberMove = LS_A_T2B;
                loseAnim = BOTH_A3_T__B_;
            } else {
                loseAnim = BOTH_KNOCKDOWN4;
            }
        }
        BOTH_CWCIRCLELOCK => {
            if superBreak != QFALSE {
                loseAnim = BOTH_LK_S_S_S_SB_1_L;
            } else if victory == QFALSE {
                (*genemy).saberMove = LS_V1_BL; //genemy->saberBounceMove =
                (*genemy).saberBlocked = BLOCKED_PARRY_BROKEN;
                loseAnim = BOTH_V1_BL_S1;
            } else if victory == QFALSE {
                //no-one won
                loseAnim = BOTH_CCWCIRCLEBREAK;
            } else {
                (*genemy).saberMove = LS_V1_BL; //genemy->saberBounceMove =
                (*genemy).saberBlocked = BLOCKED_PARRY_BROKEN;
                loseAnim = BOTH_V1_BL_S1;
                /*
                genemy->client->ps.saberMove = genemy->client->ps.saberBounceMove = LS_H1_BR;
                genemy->client->ps.saberBlocked = BLOCKED_PARRY_BROKEN;
                loseAnim = BOTH_H1_S1_BL;
                */
            }
        }
        BOTH_CCWCIRCLELOCK => {
            if superBreak != QFALSE {
                loseAnim = BOTH_LK_S_S_S_SB_1_L;
            } else if victory == QFALSE {
                (*genemy).saberMove = LS_V1_BR; //genemy->saberBounceMove =
                (*genemy).saberBlocked = BLOCKED_PARRY_BROKEN;
                loseAnim = BOTH_V1_BR_S1;
            } else if victory == QFALSE {
                //no-one won
                loseAnim = BOTH_CWCIRCLEBREAK;
            } else {
                (*genemy).saberMove = LS_V1_BR; //genemy->saberBounceMove =
                (*genemy).saberBlocked = BLOCKED_PARRY_BROKEN;
                loseAnim = BOTH_V1_BR_S1;
                /*
                genemy->client->ps.saberMove = genemy->client->ps.saberBounceMove = LS_H1_BL;
                genemy->client->ps.saberBlocked = BLOCKED_PARRY_BROKEN;
                loseAnim = BOTH_H1_S1_BR;
                */
            }
        }
        _ => {}
    }
    if loseAnim != -1 {
        NPC_SetAnim(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*genemy).clientNum as usize),
            SETANIM_BOTH,
            loseAnim,
            (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
        );
        (*genemy).weaponTime = (*genemy).torsoTimer; // + 250;
        (*genemy).saberBlocked = BLOCKED_NONE;
        (*genemy).weaponstate = WEAPON_READY;
    }
    loseAnim
}

/// `PM_SaberLockResultAnim` (bg_saber.c:1114) — for the new-system saber locks, map a
/// `duelist`'s lock anim to its win/lose/superbreak result anim and apply it. As the
/// `QAGAME` build the `#ifdef QAGAME` branches are taken: when the duelist is us we drive
/// [`PM_SetAnim`]; for the *other* duelist we drive `NPC_SetAnim(&g_entities[...])`
/// (recovering the entity from `duelist->clientNum`), and the `if ( 1 )` QAGAME guards run
/// unconditionally. Returns the result anim, or `-1`. No oracle.
///
/// # Safety
/// `duelist` must point to a valid `playerState_t`; `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberLockResultAnim(
    duelist: *mut playerState_t,
    superBreak: qboolean,
    won: qboolean,
) -> c_int {
    let pmv = *addr_of!(pm);

    let mut baseAnim: c_int = (*duelist).torsoAnim;
    match baseAnim {
        BOTH_LK_S_S_S_L_2 => baseAnim = BOTH_LK_S_S_S_L_1, //lock if I'm using single vs. a single and other intitiated
        BOTH_LK_S_S_T_L_2 => baseAnim = BOTH_LK_S_S_T_L_1, //lock if I'm using single vs. a single and other initiated
        BOTH_LK_DL_DL_S_L_2 => baseAnim = BOTH_LK_DL_DL_S_L_1, //lock if I'm using dual vs. dual and other initiated
        BOTH_LK_DL_DL_T_L_2 => baseAnim = BOTH_LK_DL_DL_T_L_1, //lock if I'm using dual vs. dual and other initiated
        BOTH_LK_ST_ST_S_L_2 => baseAnim = BOTH_LK_ST_ST_S_L_1, //lock if I'm using staff vs. a staff and other initiated
        BOTH_LK_ST_ST_T_L_2 => baseAnim = BOTH_LK_ST_ST_T_L_1, //lock if I'm using staff vs. a staff and other initiated
        _ => {}
    }
    //what kind of break?
    if superBreak == QFALSE {
        baseAnim -= 2;
    } else if superBreak != QFALSE {
        baseAnim += 1;
    } else {
        //WTF?  Not a valid result
        return -1;
    }
    //win or lose?
    if won != QFALSE {
        baseAnim += 1;
    }

    //play the anim and hold it
    // #ifdef QAGAME — server-side: set it on the other guy, too
    if (*duelist).clientNum == (*(*pmv).ps).clientNum {
        //me
        PM_SetAnim(
            SETANIM_BOTH,
            baseAnim,
            (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
            0,
        );
    } else {
        //other guy
        NPC_SetAnim(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*duelist).clientNum as usize),
            SETANIM_BOTH,
            baseAnim,
            (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
        );
    }

    if superBreak != QFALSE && won == QFALSE {
        //if you lose a superbreak, you're defenseless
        /*
        //Taken care of in SetSaberBoxSize()
        //make saberent not block
        gentity_t *saberent = &g_entities[duelist->client->ps.saberEntityNum];
        if ( saberent )
        {
            VectorClear(saberent->mins);
            VectorClear(saberent->maxs);
            G_SetOrigin(saberent, duelist->currentOrigin);
        }
        */
        // #ifdef QAGAME: if ( 1 )
        {
            //set sabermove to none
            (*duelist).saberMove = LS_NONE;
            //Hold the anim a little longer than it is
            (*duelist).torsoTimer += 250;
        }
    }

    // #ifdef QAGAME: if ( 1 )
    {
        //no attacking during this anim
        (*duelist).weaponTime = (*duelist).torsoTimer;
        (*duelist).saberBlocked = BLOCKED_NONE;
        /*
        if ( superBreak
            && won
            && baseAnim != BOTH_LK_ST_DL_T_SB_1_W )
        {//going to attack with saber, do a saber trail
            duelist->client->ps.SaberActivateTrail( 200 );
        }
        */
    }
    baseAnim
}

/// `PM_SaberAnimTransitionAnim` (bg_saber.c:425) — pick the *transition* move that bridges
/// `curmove`→`newmove`: starts/returns to/from ready, kata-end returns ([`PM_SaberKataDone`]),
/// or a quadrant-to-quadrant link via [`transitionMove`]/[`saberMoveData`]. Falls back to
/// `newmove` if no transition applies. No oracle (move FSM over verified tables/helpers).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberAnimTransitionAnim(curmove: c_int, newmove: c_int) -> c_int {
    let mut retmove: c_int = newmove;
    if curmove == LS_READY {
        //just standing there
        match newmove {
            LS_A_TL2BR | LS_A_L2R | LS_A_BL2TR | LS_A_BR2TL | LS_A_R2L | LS_A_TR2BL | LS_A_T2B => {
                //transition is the start
                retmove = LS_S_TL2BR + (newmove - LS_A_TL2BR);
            }
            _ => {}
        }
    } else {
        match newmove {
            //transitioning to ready pose
            LS_READY => {
                match curmove {
                    //transitioning from an attack
                    LS_A_TL2BR | LS_A_L2R | LS_A_BL2TR | LS_A_BR2TL | LS_A_R2L | LS_A_TR2BL
                    | LS_A_T2B => {
                        //transition is the return
                        retmove = LS_R_TL2BR + (newmove - LS_A_TL2BR);
                    }
                    _ => {}
                }
            }
            //transitioning to an attack
            LS_A_TL2BR | LS_A_L2R | LS_A_BL2TR | LS_A_BR2TL | LS_A_R2L | LS_A_TR2BL | LS_A_T2B => {
                if newmove == curmove {
                    //going into an attack
                    if PM_SaberKataDone(curmove, newmove) != QFALSE {
                        //done with this kata, must return to ready before attack again
                        retmove = LS_R_TL2BR + (newmove - LS_A_TL2BR);
                    } else {
                        //okay to chain to another attack
                        retmove = transitionMove[saberMoveData[curmove as usize].endQuad as usize]
                            [saberMoveData[newmove as usize].startQuad as usize];
                    }
                } else if saberMoveData[curmove as usize].endQuad
                    == saberMoveData[newmove as usize].startQuad
                {
                    //new move starts from same quadrant
                    retmove = newmove;
                } else {
                    match curmove {
                        //transitioning from an attack
                        LS_A_TL2BR | LS_A_L2R | LS_A_BL2TR | LS_A_BR2TL | LS_A_R2L | LS_A_TR2BL
                        | LS_A_T2B | LS_D1_BR | LS_D1__R | LS_D1_TR | LS_D1_T_ | LS_D1_TL
                        | LS_D1__L | LS_D1_BL | LS_D1_B_ => {
                            retmove = transitionMove
                                [saberMoveData[curmove as usize].endQuad as usize]
                                [saberMoveData[newmove as usize].startQuad as usize];
                        }
                        //transitioning from a return
                        LS_R_TL2BR | LS_R_L2R | LS_R_BL2TR | LS_R_BR2TL | LS_R_R2L | LS_R_TR2BL
                        | LS_R_T2B
                        //transitioning from a bounce
                        /*
                        case LS_BOUNCE_UL2LL: case LS_BOUNCE_LL2UL: case LS_BOUNCE_L2LL:
                        case LS_BOUNCE_L2UL: case LS_BOUNCE_UR2LR: case LS_BOUNCE_LR2UR:
                        case LS_BOUNCE_R2LR: case LS_BOUNCE_R2UR: case LS_BOUNCE_TOP:
                        case LS_OVER_UR2UL: case LS_OVER_UL2UR: case LS_BOUNCE_UR:
                        case LS_BOUNCE_UL: case LS_BOUNCE_LR: case LS_BOUNCE_LL:
                        */
                        //transitioning from a parry/reflection/knockaway/broken parry
                        | LS_PARRY_UP | LS_PARRY_UR | LS_PARRY_UL | LS_PARRY_LR | LS_PARRY_LL
                        | LS_REFLECT_UP | LS_REFLECT_UR | LS_REFLECT_UL | LS_REFLECT_LR
                        | LS_REFLECT_LL | LS_K1_T_ | LS_K1_TR | LS_K1_TL | LS_K1_BR | LS_K1_BL
                        | LS_V1_BR | LS_V1__R | LS_V1_TR | LS_V1_T_ | LS_V1_TL | LS_V1__L
                        | LS_V1_BL | LS_V1_B_ | LS_H1_T_ | LS_H1_TR | LS_H1_TL | LS_H1_BR
                        | LS_H1_BL => {
                            retmove = transitionMove
                                [saberMoveData[curmove as usize].endQuad as usize]
                                [saberMoveData[newmove as usize].startQuad as usize];
                        }
                        //NB: transitioning from transitions is fine
                        _ => {}
                    }
                }
            }
            //transitioning to any other anim is not supported
            _ => {}
        }
    }

    if retmove == LS_NONE {
        return newmove;
    }

    retmove
}

/// Saber alt-attack force costs (bg_saber.c:1890) — full kata / left-right / front-back.
const SABER_ALT_ATTACK_POWER: c_int = 50; //75?
const SABER_ALT_ATTACK_POWER_LR: c_int = 10; //30?
const SABER_ALT_ATTACK_POWER_FB: c_int = 25; //30/50?

/// `PM_CanDoKata` (bg_saber.c:2421) — may the player launch a kata special this frame?
/// Requires the primary style (not secondary), saber in hand and in a kata-ready move,
/// grounded, both attack buttons held, no movement input, and enough force
/// ([`BG_EnoughForcePowerForMove`]). No oracle (pm-state gate).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_CanDoKata() -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    if PM_InSecondaryStyle() != QFALSE {
        return QFALSE;
    }
    if (*ps).saberInFlight == QFALSE //not throwing saber
        && PM_SaberMoveOkayForKata() != QFALSE
        && BG_SaberInKata((*ps).saberMove) == QFALSE
        && BG_InKataAnim((*ps).legsAnim) == QFALSE
        && BG_InKataAnim((*ps).torsoAnim) == QFALSE
        /*
        && pm->ps->saberAnimLevel >= SS_FAST//fast, med or strong style
        && pm->ps->saberAnimLevel <= SS_STRONG//FIXME: Tavion, too?
        */
        && (*ps).groundEntityNum != ENTITYNUM_NONE //not in the air
        && (*pmv).cmd.buttons & BUTTON_ATTACK != 0 //pressing attack
        && (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 //pressing alt attack
        && (*pmv).cmd.forwardmove == 0 //not moving f/b
        && (*pmv).cmd.rightmove == 0 //not moving r/l
        && (*pmv).cmd.upmove <= 0 //not jumping...?
        && BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER) != QFALSE
    //have enough power
    {
        //FIXME: check rage, etc...
        return QTRUE;
    }
    QFALSE
}

/// `PM_SaberPowerCheck` (bg_saber.c:2469) — does the player have enough force power for a
/// saber-throw action? While the saber is already in flight, just compare against the
/// [`forcePowerNeeded`] table (no `EV_NOAMMO` spam); otherwise gate through
/// [`BG_EnoughForcePowerForMove`]. No oracle (pm-state gate).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_SaberPowerCheck() -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    if (*ps).saberInFlight != QFALSE {
        //so we don't keep doing stupid force out thing while guiding saber.
        if (*ps).fd.forcePower
            > forcePowerNeeded[(*ps).fd.forcePowerLevel[FP_SABERTHROW as usize] as usize]
                [FP_SABERTHROW as usize]
        {
            return QTRUE;
        }
    } else {
        return BG_EnoughForcePowerForMove(
            forcePowerNeeded[(*ps).fd.forcePowerLevel[FP_SABERTHROW as usize] as usize]
                [FP_SABERTHROW as usize],
        );
    }

    QFALSE
}

/// `PM_CanDoRollStab` (bg_saber.c:2802) — vetoes the end-of-roll stab (`LS_ROLL_STAB`)
/// when wielding [`WP_SABER`] and either of the client's sabers ([`BG_MySaber`]) carries
/// `SFL_NO_ROLL_STAB`; otherwise (or with any other weapon) the move is allowed. The
/// oracle covers the pure `(weapon, saber0, saber1)` decision via an int-marshalling
/// wrapper (`BG_MySaber`'s live `g_entities` resolution is modelled by per-saber
/// present/flags pairs in the harness).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`; `pm->ps->clientNum` a valid `g_entities` index.
pub unsafe fn PM_CanDoRollStab() -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    if (*ps).weapon == WP_SABER {
        let mut saber: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 0);
        if !saber.is_null() && (*saber).saberFlags & SFL_NO_ROLL_STAB != 0 {
            return QFALSE;
        }
        saber = BG_MySaber((*ps).clientNum, 1);
        if !saber.is_null() && (*saber).saberFlags & SFL_NO_ROLL_STAB != 0 {
            return QFALSE;
        }
    }
    QTRUE
}

/// `PM_CheckPullAttack` (bg_saber.c:1895) — the force-pull impale special. The **entire**
/// body is `#if 0`-disabled in MP ("disabling these for MP, they aren't useful"), so this
/// faithfully reduces to `return LS_NONE`. The disabled block (bg_saber.c:1897-2002) checked
/// ready/return stance + single-saber style + `PW_PULL`/`PW_DISINT_4` powerups + enough force
/// for `SABER_ALT_ATTACK_POWER_FB`, then would have driven the target via game-side
/// `NPC_SetAnim`/`G_Sound`/`level.time` (all unportable in predicted bg code anyway). No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_CheckPullAttack() -> saberMoveName_t {
    // #if 0 — whole pull-attack body disabled for MP (bg_saber.c:1897-2002); see doc above.
    LS_NONE
}

/// `PM_KickMoveForConditions` (bg_saber.c:2332) — choose the kick move from the player's
/// movement input: right/left → side kick, forward/back → front/back kick (consuming the
/// `cmd` move so it isn't double-applied); the multi-enemy "fancy kick" selection is behind
/// a C `if (0)` and kept disabled. Returns the `LS_KICK_*` move, or `-1`. No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_KickMoveForConditions() -> c_int {
    let pmv = *addr_of!(pm);

    let mut kickMove: c_int = -1;

    //FIXME: only if FP_SABER_OFFENSE >= 3
    if (*pmv).cmd.rightmove != 0 {
        //kick to side
        if (*pmv).cmd.rightmove > 0 {
            //kick right
            kickMove = LS_KICK_R;
        } else {
            //kick left
            kickMove = LS_KICK_L;
        }
        (*pmv).cmd.rightmove = 0;
    } else if (*pmv).cmd.forwardmove != 0 {
        //kick front/back
        if (*pmv).cmd.forwardmove > 0 {
            //kick fwd
            /*
            if (pm->ps->groundEntityNum != ENTITYNUM_NONE &&
                PM_CheckEnemyPresence( DIR_FRONT, 64.0f ))
            {
                kickMove = LS_HILT_BASH;
            }
            else
            */
            {
                kickMove = LS_KICK_F;
            }
        } else {
            //kick back
            kickMove = LS_KICK_B;
        }
        (*pmv).cmd.forwardmove = 0;
    } else {
        //if (pm->cmd.buttons & BUTTON_ATTACK)
        //if (pm->ps->pm_flags & PMF_JUMP_HELD)
        if false {
            //ok, let's try some fancy kicks
            //qboolean is actually of type int anyway, but just for safeness.
            let front: c_int = PM_CheckEnemyPresence(DIR_FRONT, 100.0) as c_int;
            let back: c_int = PM_CheckEnemyPresence(DIR_BACK, 100.0) as c_int;
            let right: c_int = PM_CheckEnemyPresence(DIR_RIGHT, 100.0) as c_int;
            let left: c_int = PM_CheckEnemyPresence(DIR_LEFT, 100.0) as c_int;
            let numEnemy: c_int = front + back + right + left;

            if numEnemy >= 3 || ((right == 0 || left == 0) && numEnemy >= 2) {
                //> 2 enemies near, or, >= 2 enemies near and they are not to the right and left.
                kickMove = LS_KICK_S;
            } else if right != 0 && left != 0 {
                //enemies on both sides
                kickMove = LS_KICK_RL;
            } else {
                //oh well, just do a forward kick
                kickMove = LS_KICK_F;
            }

            (*pmv).cmd.upmove = 0;
        }
    }

    kickMove
}

/// `PM_SaberLockBreak` (bg_saber.c:1220) — resolve a saber lock between us (`pm->ps`) and
/// `genemy`: select win/lose result anims (single-vs-single via [`PM_SaberLockWinAnim`]/
/// [`PM_SaberLockLoseAnim`], else the new-system [`PM_SaberLockResultAnim`]); on a `victory`
/// with over-power but no superbreak, knock the loser down ([`BG_KnockDownable`]) and flag
/// the duel loss; on a draw, shove both apart. Then clears all lock state and posts jumps.
/// No oracle (state machine over verified anim/knockdown/random helpers).
///
/// In C `winAnim`/`loseAnim`/`singleVsSingle` are written but never read (the
/// `PM_SaberLock*Anim` calls matter for their *side effects* on the playerstates); their
/// return values are discarded here, keeping behaviour identical.
///
/// # Safety
/// `genemy` must point to a valid `playerState_t`; `pm` must point to a valid `pmove_t`.
// TODO: Remove-Xbox
pub unsafe fn PM_SaberLockBreak(genemy: *mut playerState_t, victory: qboolean, strength: c_int) {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut noKnockdown: qboolean = QFALSE;
    let superBreak: qboolean = if strength + (*ps).saberLockHits > 0 {
        QTRUE
    } else {
        QFALSE
    }; //Q_irand(2,4));

    let winAnim: c_int = PM_SaberLockWinAnim(victory, superBreak);
    if winAnim != -1 {
        //a single vs. single break
        // loseAnim = PM_SaberLockLoseAnim(...) — return discarded (write-only in C)
        PM_SaberLockLoseAnim(genemy, victory, superBreak);
    } else {
        //must be a saberlock that's not between single and single...
        // singleVsSingle = qfalse; (write-only in C)
        // winAnim = PM_SaberLockResultAnim( pm->ps, superBreak, qtrue ) — return discarded
        PM_SaberLockResultAnim(ps, superBreak, QTRUE);
        (*ps).weaponstate = WEAPON_FIRING;
        // loseAnim = PM_SaberLockResultAnim( genemy, superBreak, qfalse ) — return discarded
        PM_SaberLockResultAnim(genemy, superBreak, QFALSE);
        (*genemy).weaponstate = WEAPON_READY;
    }

    if victory != QFALSE {
        //someone lost the lock, so punish them by knocking them down
        if (*ps).saberLockHits != 0 && superBreak == QFALSE {
            //there was some over-power in the win, but not enough to superbreak
            let mut oppDir: vec3_t = [0.0; 3];

            let strength: c_int = 8;

            VectorSubtract(&(*genemy).origin, &(*ps).origin, &mut oppDir);
            VectorNormalize(&mut oppDir);

            if noKnockdown != QFALSE {
                if (*genemy).saberEntityNum == 0 {
                    //if he has already lost his saber then just knock him down
                    noKnockdown = QFALSE;
                }
            }

            if noKnockdown == QFALSE && BG_KnockDownable(genemy) != QFALSE {
                (*genemy).forceHandExtend = HANDEXTEND_KNOCKDOWN;
                (*genemy).forceHandExtendTime = (*pmv).cmd.serverTime + 1100;
                (*genemy).forceDodgeAnim = 0; //this toggles between 1 and 0, when it's 1 we should play the get up anim

                (*genemy).otherKiller = (*ps).clientNum;
                (*genemy).otherKillerTime = (*pmv).cmd.serverTime + 5000;
                (*genemy).otherKillerDebounceTime = (*pmv).cmd.serverTime + 100;

                (*genemy).velocity[0] = oppDir[0] * (strength * 40) as f32;
                (*genemy).velocity[1] = oppDir[1] * (strength * 40) as f32;
                (*genemy).velocity[2] = 100.0;
            }

            (*pmv).checkDuelLoss = (*genemy).clientNum + 1;

            (*ps).saberEventFlags |= SEF_LOCK_WON;
        }
    } else {
        //If no one lost, then shove each player away from the other
        let mut oppDir: vec3_t = [0.0; 3];

        let strength: c_int = 4;

        VectorSubtract(&(*genemy).origin, &(*ps).origin, &mut oppDir);
        VectorNormalize(&mut oppDir);
        (*genemy).velocity[0] = oppDir[0] * (strength * 40) as f32;
        (*genemy).velocity[1] = oppDir[1] * (strength * 40) as f32;
        (*genemy).velocity[2] = 150.0;

        VectorSubtract(&(*ps).origin, &(*genemy).origin, &mut oppDir);
        VectorNormalize(&mut oppDir);
        (*ps).velocity[0] = oppDir[0] * (strength * 40) as f32;
        (*ps).velocity[1] = oppDir[1] * (strength * 40) as f32;
        (*ps).velocity[2] = 150.0;

        (*genemy).forceHandExtend = HANDEXTEND_WEAPONREADY;
    }

    (*ps).weaponTime = 0;
    (*genemy).weaponTime = 0;

    (*genemy).saberLockTime = 0;
    (*ps).saberLockTime = (*genemy).saberLockTime;
    (*genemy).saberLockFrame = 0;
    (*ps).saberLockFrame = (*genemy).saberLockFrame;
    (*genemy).saberLockEnemy = 0;
    (*ps).saberLockEnemy = (*genemy).saberLockEnemy;

    (*ps).forceHandExtend = HANDEXTEND_WEAPONREADY;

    PM_AddEvent(EV_JUMP);
    if victory == QFALSE {
        //no-one won
        BG_AddPredictableEventToPlayerstate(EV_JUMP, 0, genemy);
    } else if PM_irand_timesync(0, 1) != 0 {
        BG_AddPredictableEventToPlayerstate(EV_JUMP, PM_irand_timesync(0, 75), genemy);
    }
}

/// `PM_SaberLocked` (bg_saber.c:1384) — advance an active saber lock one tick: if both
/// duelists are still locked and in range, and we're "advancing" (holding attack), shove
/// our lock anim frame toward winning by our `FP_SABER_OFFENSE` strength (the loser's frame
/// mirrors it); reaching the anim's end wins → [`PM_SaberLockBreak`] with victory. Anything
/// that broke the lock (out of range, no longer locked) ends it without a winner. No oracle
/// (anim-frame state machine over verified lock predicates + [`PM_irand_timesync`]).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a valid `animations` table.
pub unsafe fn PM_SaberLocked() {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    // C: `int remaining = 0;` — the 0 init is never observed (remaining is assigned in every
    // non-returning advance path before it's read), so it's elided to stay warning-free.
    let remaining: c_int;
    let eGenemy = PM_BGEntForNum((*ps).saberLockEnemy);

    if eGenemy.is_null() {
        return;
    }

    let genemy: *mut playerState_t = (*eGenemy).playerState;

    if genemy.is_null() {
        return;
    }
    /*if ( ( (pm->ps->torsoAnim) == BOTH_BF2LOCK || ... ) && ( ... ) ) */ //yeah..
    if (*ps).saberLockFrame != 0
        && (*genemy).saberLockFrame != 0
        && BG_InSaberLock((*ps).torsoAnim) != QFALSE
        && BG_InSaberLock((*genemy).torsoAnim) != QFALSE
    {
        let dist: f32;

        (*ps).torsoTimer = 0;
        (*ps).weaponTime = 0;
        (*genemy).torsoTimer = 0;
        (*genemy).weaponTime = 0;

        dist = DistanceSquared(&(*ps).origin, &(*genemy).origin);
        if dist < 64.0 || dist > 6400.0 {
            //between 8 and 80 from each other
            PM_SaberLockBreak(genemy, QFALSE, 0);
            return;
        }
        /*
        //NOTE: time-out is handled around where PM_SaberLocked is called
        if ( pm->ps->saberLockTime <= pm->cmd.serverTime + 500 )
        {//lock just ended
            PM_SaberLockBreak( genemy, qfalse, 0 );
            return;
        }
        */
        if (*ps).saberLockAdvance != QFALSE {
            //holding attack
            let mut anim: *mut animation_t;
            let curFrame: c_int;

            (*ps).saberLockAdvance = QFALSE;

            anim = (*pmv).animations.add((*ps).torsoAnim as usize);

            let currentFrame: f32 = (*ps).saberLockFrame as f32;

            let strength: c_int = (*ps).fd.forcePowerLevel[FP_SABER_OFFENSE as usize] + 1;

            //advance/decrement my frame number
            if BG_InSaberLockOld((*ps).torsoAnim) != QFALSE {
                //old locks
                if (*ps).torsoAnim == BOTH_CCWCIRCLELOCK || (*ps).torsoAnim == BOTH_BF2LOCK {
                    curFrame = ((currentFrame as f64).floor() - strength as f64) as c_int;
                    //drop my frame one
                    if curFrame <= (*anim).firstFrame as c_int {
                        //I won!  Break out
                        PM_SaberLockBreak(genemy, QTRUE, strength);
                        return;
                    } else {
                        PM_SetAnimFrame(ps, curFrame, QTRUE, QTRUE);
                        remaining = curFrame - (*anim).firstFrame as c_int;
                    }
                } else {
                    curFrame = ((currentFrame as f64).ceil() + strength as f64) as c_int;
                    //advance my frame one
                    if curFrame >= (*anim).firstFrame as c_int + (*anim).numFrames as c_int {
                        //I won!  Break out
                        PM_SaberLockBreak(genemy, QTRUE, strength);
                        return;
                    } else {
                        PM_SetAnimFrame(ps, curFrame, QTRUE, QTRUE);
                        remaining =
                            (*anim).firstFrame as c_int + (*anim).numFrames as c_int - curFrame;
                    }
                }
            } else {
                //new locks
                if BG_CheckIncrementLockAnim((*ps).torsoAnim, SABERLOCK_WIN) != QFALSE {
                    curFrame = ((currentFrame as f64).ceil() + strength as f64) as c_int;
                    //advance my frame one
                    if curFrame >= (*anim).firstFrame as c_int + (*anim).numFrames as c_int {
                        //I won!  Break out
                        PM_SaberLockBreak(genemy, QTRUE, strength);
                        return;
                    } else {
                        PM_SetAnimFrame(ps, curFrame, QTRUE, QTRUE);
                        remaining =
                            (*anim).firstFrame as c_int + (*anim).numFrames as c_int - curFrame;
                    }
                } else {
                    curFrame = ((currentFrame as f64).floor() - strength as f64) as c_int;
                    //drop my frame one
                    if curFrame <= (*anim).firstFrame as c_int {
                        //I won!  Break out
                        PM_SaberLockBreak(genemy, QTRUE, strength);
                        return;
                    } else {
                        PM_SetAnimFrame(ps, curFrame, QTRUE, QTRUE);
                        remaining = curFrame - (*anim).firstFrame as c_int;
                    }
                }
            }
            if PM_irand_timesync(0, 2) == 0 {
                PM_AddEvent(EV_JUMP);
            }
            //advance/decrement enemy frame number
            anim = (*pmv).animations.add((*genemy).torsoAnim as usize);

            if BG_InSaberLockOld((*genemy).torsoAnim) != QFALSE {
                if (*genemy).torsoAnim == BOTH_CWCIRCLELOCK || (*genemy).torsoAnim == BOTH_BF1LOCK {
                    if PM_irand_timesync(0, 2) == 0 {
                        BG_AddPredictableEventToPlayerstate(
                            EV_PAIN,
                            ((80.0f32 / 100.0f32 * 100.0f32) as f64).floor() as c_int,
                            genemy,
                        );
                    }
                    PM_SetAnimFrame(genemy, (*anim).firstFrame as c_int + remaining, QTRUE, QTRUE);
                } else {
                    PM_SetAnimFrame(
                        genemy,
                        (*anim).firstFrame as c_int + (*anim).numFrames as c_int - remaining,
                        QTRUE,
                        QTRUE,
                    );
                }
            } else {
                //new locks
                if BG_CheckIncrementLockAnim((*genemy).torsoAnim, SABERLOCK_LOSE) != QFALSE {
                    if PM_irand_timesync(0, 2) == 0 {
                        BG_AddPredictableEventToPlayerstate(
                            EV_PAIN,
                            ((80.0f32 / 100.0f32 * 100.0f32) as f64).floor() as c_int,
                            genemy,
                        );
                    }
                    PM_SetAnimFrame(
                        genemy,
                        (*anim).firstFrame as c_int + (*anim).numFrames as c_int - remaining,
                        QTRUE,
                        QTRUE,
                    );
                } else {
                    PM_SetAnimFrame(genemy, (*anim).firstFrame as c_int + remaining, QTRUE, QTRUE);
                }
            }
        }
    } else {
        //something broke us out of it
        PM_SaberLockBreak(genemy, QFALSE, 0);
    }
}

/// `PM_SaberAttackForMovement` (bg_saber.c:2019) — given the current move and the player's
/// movement/attack input, choose the next saber attack move: side cartwheels/arials,
/// dual/staff jump attacks, medium flip / strong DFA / weak lunge, backflip, backstab,
/// down-stab, bounce chains, plus the dual-saber left-right / front-back double attacks when
/// enemies flank. Drains force for the specials. No oracle (input→move FSM over verified
/// helpers/tables). C `if (1)`/`if (0)` constant guards are preserved literally.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
// TODO: Remove-Xbox
pub unsafe fn PM_SaberAttackForMovement(curmove: saberMoveName_t) -> saberMoveName_t {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut newmove: saberMoveName_t = LS_NONE;
    let noSpecials: qboolean = PM_InSecondaryStyle();

    if (*pmv).cmd.rightmove > 0 {
        //moving right
        if noSpecials == QFALSE
            && (*ps).velocity[2] > 20.0 //pm->ps->groundEntityNum != ENTITYNUM_NONE//on ground
            && (*pmv).cmd.buttons & BUTTON_ATTACK != 0 //hitting attack
            && PM_GroundDistance() < 70.0 //not too high above ground
            && ((*pmv).cmd.upmove > 0 || (*ps).pm_flags & PMF_JUMP_HELD != 0) //focus-holding player
            && BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_LR) != QFALSE
        //have enough power
        {
            //cartwheel right
            let mut right: vec3_t = [0.0; 3];
            let mut fwdAngles: vec3_t = [0.0; 3];

            VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW], 0.0);

            BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_LR);

            AngleVectors(&fwdAngles, None, Some(&mut right), None);
            (*ps).velocity[1] = 0.0;
            (*ps).velocity[0] = (*ps).velocity[1];
            let vtmp = (*ps).velocity;
            VectorMA(&vtmp, 190.0, &right, &mut (*ps).velocity);
            if (*ps).fd.saberAnimLevel == SS_STAFF {
                newmove = LS_BUTTERFLY_RIGHT;
                (*ps).velocity[2] = 350.0;
            } else {
                //PM_SetJumped( JUMP_VELOCITY, qtrue );
                PM_AddEvent(EV_JUMP);
                (*ps).velocity[2] = 300.0;

                //if ( !Q_irand( 0, 1 ) )
                //if (PM_GroundDistance() >= 25.0f)
                if true {
                    newmove = LS_JUMPATTACK_ARIAL_RIGHT;
                } else {
                    newmove = LS_JUMPATTACK_CART_RIGHT;
                }
            }
        } else if (*pmv).cmd.forwardmove > 0 {
            //forward right = TL2BR slash
            newmove = LS_A_TL2BR;
        } else if (*pmv).cmd.forwardmove < 0 {
            //backward right = BL2TR uppercut
            newmove = LS_A_BL2TR;
        } else {
            //just right is a left slice
            newmove = LS_A_L2R;
        }
    } else if (*pmv).cmd.rightmove < 0 {
        //moving left
        if noSpecials == QFALSE
            && (*ps).velocity[2] > 20.0 //pm->ps->groundEntityNum != ENTITYNUM_NONE//on ground
            && (*pmv).cmd.buttons & BUTTON_ATTACK != 0 //hitting attack
            && PM_GroundDistance() < 70.0 //not too high above ground
            && ((*pmv).cmd.upmove > 0 || (*ps).pm_flags & PMF_JUMP_HELD != 0) //focus-holding player
            && BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_LR) != QFALSE
        //have enough power
        {
            //cartwheel left
            let mut right: vec3_t = [0.0; 3];
            let mut fwdAngles: vec3_t = [0.0; 3];

            VectorSet(&mut fwdAngles, 0.0, (*ps).viewangles[YAW], 0.0);

            BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_LR);

            AngleVectors(&fwdAngles, None, Some(&mut right), None);
            (*ps).velocity[1] = 0.0;
            (*ps).velocity[0] = (*ps).velocity[1];
            let vtmp = (*ps).velocity;
            VectorMA(&vtmp, -190.0, &right, &mut (*ps).velocity);
            if (*ps).fd.saberAnimLevel == SS_STAFF {
                newmove = LS_BUTTERFLY_LEFT;
                (*ps).velocity[2] = 250.0;
            } else {
                //PM_SetJumped( JUMP_VELOCITY, qtrue );
                PM_AddEvent(EV_JUMP);
                (*ps).velocity[2] = 350.0;

                //if ( !Q_irand( 0, 1 ) )
                //if (PM_GroundDistance() >= 25.0f)
                if true {
                    newmove = LS_JUMPATTACK_ARIAL_LEFT;
                } else {
                    newmove = LS_JUMPATTACK_CART_LEFT;
                }
            }
        } else if (*pmv).cmd.forwardmove > 0 {
            //forward left = TR2BL slash
            newmove = LS_A_TR2BL;
        } else if (*pmv).cmd.forwardmove < 0 {
            //backward left = BR2TL uppercut
            newmove = LS_A_BR2TL;
        } else {
            //just left is a right slice
            newmove = LS_A_R2L;
        }
    } else {
        //not moving left or right
        if (*pmv).cmd.forwardmove > 0 {
            //forward= T2B slash
            if noSpecials == QFALSE
                && ((*ps).fd.saberAnimLevel == SS_DUAL || (*ps).fd.saberAnimLevel == SS_STAFF)
                && (*ps).fd.forceRageRecoveryTime < (*pmv).cmd.serverTime
                //pm->ps->fd.forcePowerLevel[FP_LEVITATION] > FORCE_LEVEL_1 &&
                && ((*ps).groundEntityNum != ENTITYNUM_NONE || PM_GroundDistance() <= 40.0)
                && (*ps).velocity[2] >= 0.0
                && ((*pmv).cmd.upmove > 0 || (*ps).pm_flags & PMF_JUMP_HELD != 0)
                && BG_SaberInTransitionAny((*ps).saberMove) == QFALSE
                && BG_SaberInAttack((*ps).saberMove) == QFALSE
                && (*ps).weaponTime <= 0
                && (*ps).forceHandExtend == HANDEXTEND_NONE
                && (*pmv).cmd.buttons & BUTTON_ATTACK != 0
                && BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != QFALSE
            {
                //DUAL/STAFF JUMP ATTACK
                BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                if (*ps).fd.saberAnimLevel == SS_DUAL {
                    newmove = PM_SaberDualJumpAttackMove();
                } else {
                    //rwwFIXMEFIXME I don't like randomness for this sort of thing...
                    /*
                    if (PM_irand_timesync(0, 1))
                    {
                        newmove = LS_JUMPATTACK_STAFF_LEFT;
                    }
                    else
                    */
                    {
                        newmove = LS_JUMPATTACK_STAFF_RIGHT;
                    }
                }
            } else if noSpecials == QFALSE
                && (*ps).fd.saberAnimLevel == SS_MEDIUM
                && (*ps).velocity[2] > 100.0
                && PM_GroundDistance() < 32.0
                && BG_InSpecialJump((*ps).legsAnim) == QFALSE
                && BG_SaberInSpecialAttack((*ps).torsoAnim) == QFALSE
                && BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != QFALSE
            {
                //FLIP AND DOWNWARD ATTACK
                //trace_t tr;
                //if (PM_SomeoneInFront(&tr))
                {
                    newmove = PM_SaberFlipOverAttackMove();
                    BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                }
            } else if noSpecials == QFALSE
                && (*ps).fd.saberAnimLevel == SS_STRONG
                && (*ps).velocity[2] > 100.0
                && PM_GroundDistance() < 32.0
                && BG_InSpecialJump((*ps).legsAnim) == QFALSE
                && BG_SaberInSpecialAttack((*ps).torsoAnim) == QFALSE
                && BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != QFALSE
            {
                //DFA
                //trace_t tr;
                //if (PM_SomeoneInFront(&tr))
                {
                    newmove = PM_SaberJumpAttackMove();
                    BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                }
            } else if ((*ps).fd.saberAnimLevel == SS_FAST
                || (*ps).fd.saberAnimLevel == SS_DUAL
                || (*ps).fd.saberAnimLevel == SS_STAFF)
                && (*ps).groundEntityNum != ENTITYNUM_NONE
                && (*ps).pm_flags & PMF_DUCKED != 0
                && (*ps).weaponTime <= 0
                && BG_SaberInSpecialAttack((*ps).torsoAnim) == QFALSE
                && BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != QFALSE
            {
                //LUNGE (weak)
                BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                if (*ps).fd.saberAnimLevel == FORCE_LEVEL_1 {
                    newmove = PM_SaberLungeAttackMove();
                } else if noSpecials == QFALSE && (*ps).fd.saberAnimLevel == SS_STAFF {
                    newmove = LS_SPINATTACK;
                } else if noSpecials == QFALSE {
                    newmove = LS_SPINATTACK_DUAL;
                }
            } else if noSpecials == QFALSE {
                let stabDownMove: saberMoveName_t = PM_CheckStabDown();
                if stabDownMove != LS_NONE
                    && BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != QFALSE
                {
                    newmove = stabDownMove;
                    BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                } else {
                    newmove = LS_A_T2B;
                }
            }
        } else if (*pmv).cmd.forwardmove < 0 {
            //backward= T2B slash//B2T uppercut?
            if noSpecials == QFALSE
                && (*ps).fd.saberAnimLevel == SS_STAFF
                && (*ps).fd.forceRageRecoveryTime < (*pmv).cmd.serverTime
                && (*ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_1
                && ((*ps).groundEntityNum != ENTITYNUM_NONE || PM_GroundDistance() <= 40.0)
                && (*ps).velocity[2] >= 0.0
                && ((*pmv).cmd.upmove > 0 || (*ps).pm_flags & PMF_JUMP_HELD != 0)
                && BG_SaberInTransitionAny((*ps).saberMove) == QFALSE
                && BG_SaberInAttack((*ps).saberMove) == QFALSE
                && (*ps).weaponTime <= 0
                && (*ps).forceHandExtend == HANDEXTEND_NONE
                && (*pmv).cmd.buttons & BUTTON_ATTACK != 0
            {
                //BACKFLIP ATTACK
                newmove = PM_SaberBackflipAttackMove();
            } else if PM_CanBackstab() != QFALSE
                && BG_SaberInSpecialAttack((*ps).torsoAnim) == QFALSE
            {
                //BACKSTAB (attack varies by level)
                if (*ps).fd.saberAnimLevel >= FORCE_LEVEL_2 && (*ps).fd.saberAnimLevel != SS_STAFF {
                    //medium and higher attacks
                    if (*ps).pm_flags & PMF_DUCKED != 0 || (*pmv).cmd.upmove < 0 {
                        newmove = LS_A_BACK_CR;
                    } else {
                        newmove = LS_A_BACK;
                    }
                } else {
                    //weak attack
                    newmove = LS_A_BACKSTAB;
                }
            } else {
                newmove = LS_A_T2B;
            }
        } else if PM_SaberInBounce(curmove) != QFALSE {
            //bounces should go to their default attack if you don't specify a direction but are attacking
            newmove = saberMoveData[curmove as usize].chain_attack;

            if PM_SaberKataDone(curmove, newmove) != QFALSE {
                newmove = saberMoveData[curmove as usize].chain_idle;
            } else {
                newmove = saberMoveData[curmove as usize].chain_attack;
            }
        } else if curmove == LS_READY {
            //Not moving at all, shouldn't have gotten here...?
            //for now, just pick a random attack
            //newmove = Q_irand( LS_A_TL2BR, LS_A_T2B );
            //rww - If we don't seed with a "common" value, the client and server will get mismatched
            //prediction values.
            newmove = LS_A_T2B; //decided we don't like random attacks when idle, use an overhead instead.
        }
    }

    if (*ps).fd.saberAnimLevel == SS_DUAL {
        if (newmove == LS_A_R2L
            || newmove == LS_S_R2L
            || newmove == LS_A_L2R
            || newmove == LS_S_L2R)
            && PM_CanDoDualDoubleAttacks() != QFALSE
            && PM_CheckEnemyPresence(DIR_RIGHT, 100.0) != QFALSE
            && PM_CheckEnemyPresence(DIR_LEFT, 100.0) != QFALSE
        {
            //enemy both on left and right
            newmove = LS_DUAL_LR;
            //probably already moved, but...
            (*pmv).cmd.rightmove = 0;
        } else if (newmove == LS_A_T2B
            || newmove == LS_S_T2B
            || newmove == LS_A_BACK
            || newmove == LS_A_BACK_CR)
            && PM_CanDoDualDoubleAttacks() != QFALSE
            && PM_CheckEnemyPresence(DIR_FRONT, 100.0) != QFALSE
            && PM_CheckEnemyPresence(DIR_BACK, 100.0) != QFALSE
        {
            //enemy both in front and back
            newmove = LS_DUAL_FB;
            //probably already moved, but...
            (*pmv).cmd.forwardmove = 0;
        }
    }

    newmove
}

/// `PM_WeaponLightsaber` (bg_saber.c:2501, ~915 LOC) — the lightsaber weapon state
/// machine, run from `PM_Weapon` when wielding `WP_SABER`. In order it handles: the
/// knockdown/roll-stab early-out; saber-lock advance/break; kick & super-break anim
/// holds; the sabers-off ready/unholster path (which `goto`s the weapon-change checks);
/// dropped-saber recovery; saber-throw and staff-kick on alt-attack; in-flight saber
/// guiding; the dead-player early-out; the `weaponTime` countdown plus the `saberBlocked`
/// parry/reflect/bounce reaction table; then the weapon-change / kata / weapon-raising
/// machinery and finally the attack-move lookup that cross-indexes the current move with
/// the movement command to pick the next `LS_*` swing.
///
/// The C `goto weapChecks` (taken by the sabers-off branch, with `checkOnlyWeap=qtrue`)
/// is modelled by a `goto_weap_checks` flag guarding the skipped middle region (the
/// throw/guide/block code); control reaches the shared weapon-change tail either way.
/// `delayed_fire` is always `qfalse` here (kept verbatim for fidelity, so the big
/// `if(!delayed_fire)` block always runs). The dead `int amount` assignment is carried
/// as `let _amount`. `G_DrainPowerForSpecialMove`/`PM_CheckDualSpinProtect` are left as
/// comments — both are already commented out in the JKA source. No oracle (drives the
/// anim/weapon state over already-verified `BG_*`/`PM_Saber*` predicates, [`saberMoveData`],
/// [`transitionMove`], [`weaponData`] and [`forcePowerNeeded`]).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_WeaponLightsaber() {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let delayed_fire: qboolean = QFALSE;
    let mut anim: c_int = -1;
    let curmove: c_int;
    let mut newmove: c_int = LS_NONE;

    let mut checkOnlyWeap: qboolean = QFALSE;

    if PM_InKnockDown(ps) != QFALSE || BG_InRoll(ps, (*ps).legsAnim) != QFALSE {
        //in knockdown
        // make weapon function
        if (*ps).weaponTime > 0 {
            (*ps).weaponTime -= (*addr_of!(pml)).msec;
            if (*ps).weaponTime <= 0 {
                (*ps).weaponTime = 0;
            }
        }
        if (*ps).legsAnim == BOTH_ROLL_F && (*ps).legsTimer <= 250 {
            if ((*pmv).cmd.buttons & BUTTON_ATTACK) != 0 {
                if BG_EnoughForcePowerForMove(SABER_ALT_ATTACK_POWER_FB) != QFALSE
                    && (*ps).saberInFlight == QFALSE
                {
                    //make sure the saber is on for this move!
                    if (*ps).saberHolstered == 2 {
                        //all the way off
                        (*ps).saberHolstered = 0;
                        PM_AddEvent(EV_SABER_UNHOLSTER);
                    }
                    PM_SetSaberMove(LS_ROLL_STAB as c_short);
                    BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER_FB);
                }
            }
        }
        return;
    }

    if (*ps).saberLockTime > (*pmv).cmd.serverTime {
        (*ps).saberMove = LS_NONE;
        PM_SaberLocked();
        return;
    } else if (*ps).saberLockFrame != 0 {
        if (*ps).saberLockEnemy < ENTITYNUM_NONE && (*ps).saberLockEnemy >= 0 {
            let bgEnt = PM_BGEntForNum((*ps).saberLockEnemy);
            if !bgEnt.is_null() {
                let en = (*bgEnt).playerState;
                if !en.is_null() {
                    PM_SaberLockBreak(en, QFALSE, 0);
                    return;
                }
            }
        }

        if (*ps).saberLockFrame != 0 {
            (*ps).torsoTimer = 0;
            PM_SetAnim(SETANIM_TORSO, BOTH_STAND1, SETANIM_FLAG_OVERRIDE, 100);
            (*ps).saberLockFrame = 0;
        }
    }

    if BG_KickingAnim((*ps).legsAnim) != QFALSE || BG_KickingAnim((*ps).torsoAnim) != QFALSE {
        if (*ps).legsTimer > 0 {
            //you're kicking, no interruptions
            return;
        }
        //done?  be immeditately ready to do an attack
        (*ps).saberMove = LS_READY;
        (*ps).weaponTime = 0;
    }

    if BG_SuperBreakLoseAnim((*ps).torsoAnim) != QFALSE
        || BG_SuperBreakWinAnim((*ps).torsoAnim) != QFALSE
    {
        if (*ps).torsoTimer > 0 {
            //never interrupt these
            return;
        }
    }

    let mut goto_weap_checks = false;

    if BG_SabersOff(ps) != QFALSE {
        if (*ps).saberMove != LS_READY {
            PM_SetSaberMove(LS_READY as c_short);
        }

        if (*ps).legsAnim != (*ps).torsoAnim
            && BG_InSlopeAnim((*ps).legsAnim) == QFALSE
            && (*ps).torsoTimer <= 0
        {
            PM_SetAnim(SETANIM_TORSO, (*ps).legsAnim, SETANIM_FLAG_OVERRIDE, 100);
        } else if BG_InSlopeAnim((*ps).legsAnim) != QFALSE && (*ps).torsoTimer <= 0 {
            PM_SetAnim(SETANIM_TORSO, PM_GetSaberStance(), SETANIM_FLAG_OVERRIDE, 100);
        }

        if (*ps).weaponTime < 1
            && (((*pmv).cmd.buttons & BUTTON_ALT_ATTACK) != 0
                || ((*pmv).cmd.buttons & BUTTON_ATTACK) != 0)
        {
            if (*ps).duelTime < (*pmv).cmd.serverTime {
                if (*ps).m_iVehicleNum == 0 {
                    //don't let em unholster the saber by attacking while on vehicle
                    (*ps).saberHolstered = 0;
                    PM_AddEvent(EV_SABER_UNHOLSTER);
                } else {
                    (*pmv).cmd.buttons &= !BUTTON_ALT_ATTACK;
                    (*pmv).cmd.buttons &= !BUTTON_ATTACK;
                }
            }
        }

        if (*ps).weaponTime > 0 {
            (*ps).weaponTime -= (*addr_of!(pml)).msec;
        }

        checkOnlyWeap = QTRUE;
        goto_weap_checks = true;
    }

    if !goto_weap_checks {
        if (*ps).saberEntityNum == 0 && (*ps).saberInFlight != QFALSE {
            //this means our saber has been knocked away
            //Old method, don't want to do this now because we want to finish up reflected attacks
            //and things if our saber is pried out of our hands from one.
            if (*ps).fd.saberAnimLevel == SS_DUAL {
                if (*ps).saberHolstered > 1 {
                    (*ps).saberHolstered = 1;
                }
            } else {
                (*pmv).cmd.buttons &= !BUTTON_ATTACK;
            }
            (*pmv).cmd.buttons &= !BUTTON_ALT_ATTACK;
        }

        if ((*pmv).cmd.buttons & BUTTON_ALT_ATTACK) != 0 {
            //might as well just check for a saber throw right here
            if (*ps).fd.saberAnimLevel == SS_STAFF {
                //kick instead of doing a throw
                //if in a saber attack return anim, can interrupt it with a kick
                if (*ps).weaponTime > 0
                    && PM_SaberInReturn((*ps).saberMove) != QFALSE
                    && (*ps).saberBlocked == BLOCKED_NONE
                    && ((*pmv).cmd.buttons & BUTTON_ATTACK) == 0
                {
                    if ((*pmv).cmd.forwardmove != 0 || (*pmv).cmd.rightmove != 0)
                        && PM_CheckAltKickAttack() != QFALSE
                    {
                        //allow them to do the kick now!
                        let kickMove = PM_KickMoveForConditions();
                        if kickMove != -1 {
                            (*ps).weaponTime = 0;
                            PM_SetSaberMove(kickMove as c_short);
                            return;
                        }
                    }
                }
            } else if (*ps).weaponTime < 1
                && (*ps).saberCanThrow != QFALSE
                && BG_HasYsalamiri((*pmv).gametype, ps) == QFALSE
                && BG_CanUseFPNow((*pmv).gametype, ps, (*pmv).cmd.serverTime, FP_SABERTHROW) != QFALSE
                && (*ps).fd.forcePowerLevel[FP_SABERTHROW as usize] > 0
                && PM_SaberPowerCheck() != QFALSE
            {
                let mut sabTr: trace_t = core::mem::zeroed();
                let mut fwd: vec3_t = [0.0; 3];
                let mut minFwd: vec3_t = [0.0; 3];
                let mut sabMins: vec3_t = [0.0; 3];
                let mut sabMaxs: vec3_t = [0.0; 3];

                VectorSet(&mut sabMins, SABERMINS_X, SABERMINS_Y, SABERMINS_Z);
                VectorSet(&mut sabMaxs, SABERMAXS_X, SABERMAXS_Y, SABERMAXS_Z);

                AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
                VectorMA(&(*ps).origin, SABER_MIN_THROW_DIST, &fwd, &mut minFwd);

                ((*pmv).trace.unwrap())(
                    &mut sabTr,
                    (*ps).origin.as_ptr(),
                    sabMins.as_ptr(),
                    sabMaxs.as_ptr(),
                    minFwd.as_ptr(),
                    (*ps).clientNum,
                    MASK_PLAYERSOLID,
                );

                if sabTr.allsolid != 0 || sabTr.startsolid != 0 || sabTr.fraction < 1.0 {
                    //not enough room to throw
                } else {
                    //throw it
                    //This will get set to false again once the saber makes it back to its owner game-side
                    if (*ps).saberInFlight == QFALSE {
                        (*ps).fd.forcePower -= forcePowerNeeded
                            [(*ps).fd.forcePowerLevel[FP_SABERTHROW as usize] as usize]
                            [FP_SABERTHROW as usize];
                    }

                    (*ps).saberInFlight = QTRUE;
                }
            }
        }

        if (*ps).saberInFlight != QFALSE && (*ps).saberEntityNum != 0 {
            //guiding saber
            if (*ps).fd.saberAnimLevel != SS_DUAL //not using 2 sabers
                || (*ps).saberHolstered != 0 //left one off
                || (((*pmv).cmd.buttons & BUTTON_ATTACK) == 0 //not trying to start an attack AND...
                    && ((*ps).torsoAnim == BOTH_SABERDUAL_STANCE //not already attacking
                        || (*ps).torsoAnim == BOTH_SABERPULL
                        || (*ps).torsoAnim == BOTH_STAND1
                        || PM_RunningAnim((*ps).torsoAnim) != QFALSE
                        || PM_WalkingAnim((*ps).torsoAnim) != QFALSE
                        || PM_JumpingAnim((*ps).torsoAnim) != QFALSE
                        || PM_SwimmingAnim((*ps).torsoAnim) != QFALSE))
            {
                PM_SetAnim(
                    SETANIM_TORSO,
                    BOTH_SABERPULL,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    100,
                );
                (*ps).torsoTimer = 1;
                return;
            }
        }

        // check for dead player
        if (*ps).stats[STAT_HEALTH as usize] <= 0 {
            return;
        }

        // make weapon function
        if (*ps).weaponTime > 0 {
            //check for special pull move while busy
            let pullmove: saberMoveName_t = PM_CheckPullAttack();
            if pullmove != LS_NONE {
                (*ps).weaponTime = 0;
                (*ps).torsoTimer = 0;
                (*ps).legsTimer = 0;
                (*ps).forceHandExtend = HANDEXTEND_NONE;
                (*ps).weaponstate = WEAPON_READY;
                PM_SetSaberMove(pullmove as c_short);
                return;
            }

            (*ps).weaponTime -= (*addr_of!(pml)).msec;
        } else {
            (*ps).weaponstate = WEAPON_READY;
        }

        // Now we react to a block action by the player's lightsaber.
        if (*ps).saberBlocked != 0 {
            if (*ps).saberBlocked >= BLOCKED_UPPER_RIGHT
                && (*ps).saberBlocked < BLOCKED_UPPER_RIGHT_PROJ
            {
                //hold the parry for a bit
                (*ps).weaponTime = bg_parryDebounce
                    [(*ps).fd.forcePowerLevel[FP_SABER_DEFENSE as usize] as usize]
                    + 200;
            }
            match (*ps).saberBlocked {
                BLOCKED_BOUNCE_MOVE => {
                    //act as a bounceMove and reset the saberMove instead of using a seperate value for it
                    (*ps).torsoTimer = 0;
                    PM_SetSaberMove((*ps).saberMove as c_short);
                    (*ps).weaponTime = (*ps).torsoTimer;
                    (*ps).saberBlocked = 0;
                }
                BLOCKED_PARRY_BROKEN => {
                    //whatever parry we were is in now broken, play the appropriate knocked-away anim
                    let nextMove: c_int;

                    if PM_SaberInBrokenParry((*ps).saberMove) != QFALSE {
                        //already have one...?
                        nextMove = (*ps).saberMove;
                    } else {
                        nextMove = PM_BrokenParryForParry((*ps).saberMove);
                    }
                    if nextMove != LS_NONE {
                        PM_SetSaberMove(nextMove as c_short);
                        (*ps).weaponTime = (*ps).torsoTimer;
                    } else {
                        //Maybe in a knockaway?
                    }
                }
                BLOCKED_ATK_BOUNCE => {
                    // If there is absolutely no blocked move in the chart, don't even mess with the animation.
                    // OR if we are already in a block or parry.
                    if (*ps).saberMove >= LS_T1_BR__R {
                        //an actual bounce?  Other bounces before this are actually transitions?
                        (*ps).saberBlocked = BLOCKED_NONE;
                    } else {
                        let bounceMove: c_int;

                        if PM_SaberInBounce((*ps).saberMove) != QFALSE
                            || BG_SaberInAttack((*ps).saberMove) == QFALSE
                        {
                            if ((*pmv).cmd.buttons & BUTTON_ATTACK) != 0 {
                                //transition to a new attack
                                let mut newQuad = PM_SaberMoveQuadrantForMovement(&mut (*pmv).cmd);
                                while newQuad == saberMoveData[(*ps).saberMove as usize].startQuad {
                                    //player is still in same attack quad, don't repeat that attack
                                    //because it looks bad
                                    newQuad = PM_irand_timesync(Q_BR, Q_BL);
                                } //else player is switching up anyway, take the new attack dir
                                bounceMove = transitionMove
                                    [saberMoveData[(*ps).saberMove as usize].startQuad as usize]
                                    [newQuad as usize];
                            } else {
                                //return to ready
                                if saberMoveData[(*ps).saberMove as usize].startQuad == Q_T {
                                    bounceMove = LS_R_BL2TR;
                                } else if saberMoveData[(*ps).saberMove as usize].startQuad < Q_T {
                                    bounceMove = LS_R_TL2BR
                                        + saberMoveData[(*ps).saberMove as usize].startQuad
                                        - Q_BR;
                                } else {
                                    bounceMove = LS_R_BR2TL
                                        + saberMoveData[(*ps).saberMove as usize].startQuad
                                        - Q_TL;
                                }
                            }
                        } else {
                            //start the bounce
                            bounceMove = PM_SaberBounceForAttack((*ps).saberMove);
                        }

                        PM_SetSaberMove(bounceMove as c_short);

                        (*ps).weaponTime = (*ps).torsoTimer;
                    }
                }
                BLOCKED_UPPER_RIGHT => PM_SetSaberMove(LS_PARRY_UR as c_short),
                BLOCKED_UPPER_RIGHT_PROJ => PM_SetSaberMove(LS_REFLECT_UR as c_short),
                BLOCKED_UPPER_LEFT => PM_SetSaberMove(LS_PARRY_UL as c_short),
                BLOCKED_UPPER_LEFT_PROJ => PM_SetSaberMove(LS_REFLECT_UL as c_short),
                BLOCKED_LOWER_RIGHT => PM_SetSaberMove(LS_PARRY_LR as c_short),
                BLOCKED_LOWER_RIGHT_PROJ => PM_SetSaberMove(LS_REFLECT_LR as c_short),
                BLOCKED_LOWER_LEFT => PM_SetSaberMove(LS_PARRY_LL as c_short),
                BLOCKED_LOWER_LEFT_PROJ => PM_SetSaberMove(LS_REFLECT_LL as c_short),
                BLOCKED_TOP => PM_SetSaberMove(LS_PARRY_UP as c_short),
                BLOCKED_TOP_PROJ => PM_SetSaberMove(LS_REFLECT_UP as c_short),
                _ => {
                    (*ps).saberBlocked = BLOCKED_NONE;
                }
            }
            if (*ps).saberBlocked >= BLOCKED_UPPER_RIGHT
                && (*ps).saberBlocked < BLOCKED_UPPER_RIGHT_PROJ
            {
                //hold the parry for a bit
                if (*ps).torsoTimer < (*ps).weaponTime {
                    (*ps).torsoTimer = (*ps).weaponTime;
                }
            }

            //clear block
            (*ps).saberBlocked = 0;

            // Charging is like a lead-up before attacking again.  This is an appropriate use,
            // or we can create a new weaponstate for blocking
            (*ps).weaponstate = WEAPON_READY;

            // Done with block, so stop these active weapon branches.
            return;
        }
    }

    // weapChecks:
    if (*ps).saberEntityNum != 0 {
        //only check if we have our saber with us
        // check for weapon change
        // can't change if weapon is firing, but can change again if lowering or raising
        if (*ps).weaponTime <= 0 && (*ps).torsoTimer <= 0 {
            if (*ps).weapon != (*pmv).cmd.weapon as c_int {
                PM_BeginWeaponChange((*pmv).cmd.weapon as c_int);
            }
        }
    }

    if PM_CanDoKata() != QFALSE {
        //FIXME: make sure to turn on saber(s)!
        match (*ps).fd.saberAnimLevel {
            SS_FAST | SS_TAVION => PM_SetSaberMove(LS_A1_SPECIAL as c_short),
            SS_MEDIUM => PM_SetSaberMove(LS_A2_SPECIAL as c_short),
            SS_STRONG | SS_DESANN => PM_SetSaberMove(LS_A3_SPECIAL as c_short),
            SS_DUAL => PM_SetSaberMove(LS_DUAL_SPIN_PROTECT as c_short), //PM_CheckDualSpinProtect() (commented out in JKA)
            SS_STAFF => PM_SetSaberMove(LS_STAFF_SOULCAL as c_short),
            _ => {}
        }
        (*ps).weaponstate = WEAPON_FIRING;
        //G_DrainPowerForSpecialMove( pm->gent, FP_SABER_OFFENSE, SABER_ALT_ATTACK_POWER ); (commented out in JKA)
        BG_ForcePowerDrain(ps, FP_GRIP, SABER_ALT_ATTACK_POWER);
        return;
    }

    if (*ps).weaponTime > 0 {
        return;
    }

    // *********************************************************
    // WEAPON_DROPPING
    // *********************************************************

    // change weapon if time
    if (*ps).weaponstate == WEAPON_DROPPING {
        PM_FinishWeaponChange();
        return;
    }

    // *********************************************************
    // WEAPON_RAISING
    // *********************************************************

    if (*ps).weaponstate == WEAPON_RAISING {
        //Just selected the weapon
        (*ps).weaponstate = WEAPON_IDLE;
        if (*ps).legsAnim == BOTH_WALK1 {
            PM_SetAnim(SETANIM_TORSO, BOTH_WALK1, SETANIM_FLAG_NORMAL, 100);
        } else if (*ps).legsAnim == BOTH_RUN1 {
            PM_SetAnim(SETANIM_TORSO, BOTH_RUN1, SETANIM_FLAG_NORMAL, 100);
        } else if (*ps).legsAnim == BOTH_RUN2 {
            PM_SetAnim(SETANIM_TORSO, BOTH_RUN2, SETANIM_FLAG_NORMAL, 100);
        } else if (*ps).legsAnim == BOTH_RUN_STAFF {
            PM_SetAnim(SETANIM_TORSO, BOTH_RUN_STAFF, SETANIM_FLAG_NORMAL, 100);
        } else if (*ps).legsAnim == BOTH_RUN_DUAL {
            PM_SetAnim(SETANIM_TORSO, BOTH_RUN_DUAL, SETANIM_FLAG_NORMAL, 100);
        } else if (*ps).legsAnim == BOTH_WALK1 {
            PM_SetAnim(SETANIM_TORSO, BOTH_WALK1, SETANIM_FLAG_NORMAL, 100);
        } else if (*ps).legsAnim == BOTH_WALK2 {
            PM_SetAnim(SETANIM_TORSO, BOTH_WALK2, SETANIM_FLAG_NORMAL, 100);
        } else if (*ps).legsAnim == BOTH_WALK_STAFF {
            PM_SetAnim(SETANIM_TORSO, BOTH_WALK_STAFF, SETANIM_FLAG_NORMAL, 100);
        } else if (*ps).legsAnim == BOTH_WALK_DUAL {
            PM_SetAnim(SETANIM_TORSO, BOTH_WALK_DUAL, SETANIM_FLAG_NORMAL, 100);
        } else {
            PM_SetAnim(SETANIM_TORSO, PM_GetSaberStance(), SETANIM_FLAG_NORMAL, 100);
        }

        if (*ps).weaponstate == WEAPON_RAISING {
            return;
        }
    }

    if checkOnlyWeap != QFALSE {
        return;
    }

    // *********************************************************
    // Check for WEAPON ATTACK
    // *********************************************************
    if (*ps).fd.saberAnimLevel == SS_STAFF && ((*pmv).cmd.buttons & BUTTON_ALT_ATTACK) != 0 {
        //ok, try a kick I guess.
        let mut kickMove: c_int = -1;

        if BG_KickingAnim((*ps).torsoAnim) == QFALSE
            && BG_KickingAnim((*ps).legsAnim) == QFALSE
            && BG_InRoll(ps, (*ps).legsAnim) == QFALSE
            && (*ps).saberMove == LS_READY //not already in a kick
            && ((*ps).pm_flags & PMF_DUCKED) == 0 //not ducked
            && (*pmv).cmd.upmove >= 0
        //not trying to duck
        {
            //player kicks
            kickMove = PM_KickMoveForConditions();
        }

        if kickMove != -1 {
            if (*ps).groundEntityNum == ENTITYNUM_NONE {
                //if in air, convert kick to an in-air kick
                //let's only allow air kicks if a certain distance from the ground
                //it's silly to be able to do them right as you land.
                //also looks wrong to transition from a non-complete flip anim...
                let gDist = PM_GroundDistance();
                if (BG_FlippingAnim((*ps).legsAnim) == QFALSE || (*ps).legsTimer <= 0)
                    && gDist > 64.0 //strict minimum
                    && gDist > (-(*ps).velocity[2]) - 64.0
                //make sure we are high to ground relative to downward velocity as well
                {
                    match kickMove {
                        LS_KICK_F => kickMove = LS_KICK_F_AIR,
                        LS_KICK_B => kickMove = LS_KICK_B_AIR,
                        LS_KICK_R => kickMove = LS_KICK_R_AIR,
                        LS_KICK_L => kickMove = LS_KICK_L_AIR,
                        _ => kickMove = -1, //oh well, can't do any other kick move while in-air
                    }
                } else {
                    //leave it as a normal kick unless we're too high up
                    if gDist > 128.0 || (*ps).velocity[2] >= 0.0 {
                        //off ground, but too close to ground
                        kickMove = -1;
                    }
                }
            }

            if kickMove != -1 {
                PM_SetSaberMove(kickMove as c_short);
                return;
            }
        }
    }

    //this is never a valid regular saber attack button
    (*pmv).cmd.buttons &= !BUTTON_ALT_ATTACK;

    if delayed_fire == QFALSE {
        // Start with the current move, and cross index it with the current control states.
        if (*ps).saberMove > LS_NONE && (*ps).saberMove < LS_MOVE_MAX {
            curmove = (*ps).saberMove;
        } else {
            curmove = LS_READY;
        }

        if curmove == LS_A_JUMP_T__B_ || (*ps).torsoAnim == BOTH_FORCELEAP2_T__B_ {
            //must transition back to ready from this anim
            newmove = LS_R_T2B;
        }
        // check for fire
        else if ((*pmv).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) == 0 {
            //not attacking
            (*ps).weaponTime = 0;

            if (*ps).weaponTime > 0 {
                //Still firing
                (*ps).weaponstate = WEAPON_FIRING;
            } else if (*ps).weaponstate != WEAPON_READY {
                (*ps).weaponstate = WEAPON_IDLE;
            }
            //Check for finishing an anim if necc.
            if curmove >= LS_S_TL2BR && curmove <= LS_S_T2B {
                //started a swing, must continue from here
                newmove = LS_A_TL2BR + (curmove - LS_S_TL2BR);
            } else if curmove >= LS_A_TL2BR && curmove <= LS_A_T2B {
                //finished an attack, must continue from here
                newmove = LS_R_TL2BR + (curmove - LS_A_TL2BR);
            } else if PM_SaberInTransition(curmove) != QFALSE {
                //in a transition, must play sequential attack
                newmove = saberMoveData[curmove as usize].chain_attack;
            } else if PM_SaberInBounce(curmove) != QFALSE {
                //in a bounce
                newmove = saberMoveData[curmove as usize].chain_idle; //oops, not attacking, so don't chain
            } else {
                //FIXME: what about returning from a parry?
                PM_SetSaberMove(LS_READY as c_short);
                return;
            }
        }

        // ***************************************************
        // Pressing attack, so we must look up the proper attack move.

        if (*ps).weaponTime > 0 {
            // Last attack is not yet complete.
            (*ps).weaponstate = WEAPON_FIRING;
            return;
        } else {
            let mut both: qboolean = QFALSE;
            if (*ps).torsoAnim == BOTH_FORCELONGLEAP_ATTACK
                || (*ps).torsoAnim == BOTH_FORCELONGLEAP_LAND
            {
                //can't attack in these anims
                return;
            } else if (*ps).torsoAnim == BOTH_FORCELONGLEAP_START {
                //only 1 attack you can do from this anim
                if (*ps).torsoTimer >= 200 {
                    //hit it early enough to do the attack
                    PM_SetSaberMove(LS_LEAP_ATTACK as c_short);
                }
                return;
            }
            if curmove >= LS_PARRY_UP && curmove <= LS_REFLECT_LL {
                //from a parry or reflection, can go directly into an attack
                match saberMoveData[curmove as usize].endQuad {
                    Q_T => newmove = LS_A_T2B,
                    Q_TR => newmove = LS_A_TR2BL,
                    Q_TL => newmove = LS_A_TL2BR,
                    Q_BR => newmove = LS_A_BR2TL,
                    Q_BL => newmove = LS_A_BL2TR,
                    //shouldn't be a parry that ends at L, R or B
                    _ => {}
                }
            }

            if newmove != LS_NONE {
                //have a valid, final LS_ move picked, so skip findingt he transition move and just get the anim
                anim = saberMoveData[newmove as usize].animToUse;
            }

            //FIXME: diagonal dirs use the figure-eight attacks from ready pose?
            if anim == -1 {
                //FIXME: take FP_SABER_OFFENSE into account here somehow?
                if PM_SaberInTransition(curmove) != QFALSE {
                    //in a transition, must play sequential attack
                    newmove = saberMoveData[curmove as usize].chain_attack;
                } else if curmove >= LS_S_TL2BR && curmove <= LS_S_T2B {
                    //started a swing, must continue from here
                    newmove = LS_A_TL2BR + (curmove - LS_S_TL2BR);
                } else if PM_SaberInBrokenParry(curmove) != QFALSE {
                    //broken parries must always return to ready
                    newmove = LS_READY;
                } else {
                    //get attack move from movement command
                    newmove = PM_SaberAttackForMovement(curmove);
                    if (PM_SaberInBounce(curmove) != QFALSE
                        || PM_SaberInBrokenParry(curmove) != QFALSE)
                        && saberMoveData[newmove as usize].startQuad
                            == saberMoveData[curmove as usize].endQuad
                    {
                        //this attack would be a repeat of the last (which was blocked), so don't
                        //actually use it, use the default chain attack for this bounce
                        newmove = saberMoveData[curmove as usize].chain_attack;
                    }

                    if PM_SaberKataDone(curmove, newmove) != QFALSE {
                        //cannot chain this time
                        newmove = saberMoveData[curmove as usize].chain_idle;
                    }
                }
                if newmove != LS_NONE {
                    //Now get the proper transition move
                    newmove = PM_SaberAnimTransitionAnim(curmove, newmove);
                    anim = saberMoveData[newmove as usize].animToUse;
                }
            }

            if anim == -1 {
                //not side-stepping, pick neutral anim
                // Add randomness for prototype?
                newmove = saberMoveData[curmove as usize].chain_attack;

                anim = saberMoveData[newmove as usize].animToUse;

                if (*pmv).cmd.forwardmove == 0
                    && (*pmv).cmd.rightmove == 0
                    && (*pmv).cmd.upmove >= 0
                    && (*ps).groundEntityNum != ENTITYNUM_NONE
                {
                    //not moving at all, so set the anim on entire body
                    both = QTRUE;
                }
            }

            if anim == -1 {
                match (*ps).legsAnim {
                    BOTH_WALK1 | BOTH_WALK2 | BOTH_WALK_STAFF | BOTH_WALK_DUAL | BOTH_WALKBACK1
                    | BOTH_WALKBACK2 | BOTH_WALKBACK_STAFF | BOTH_WALKBACK_DUAL | BOTH_RUN1
                    | BOTH_RUN2 | BOTH_RUN_STAFF | BOTH_RUN_DUAL | BOTH_RUNBACK1 | BOTH_RUNBACK2
                    | BOTH_RUNBACK_STAFF => {
                        anim = (*ps).legsAnim;
                    }
                    _ => {
                        anim = PM_GetSaberStance();
                    }
                }

                newmove = LS_READY;
            }

            PM_SetSaberMove(newmove as c_short);

            if both != QFALSE && (*ps).torsoAnim == anim {
                PM_SetAnim(
                    SETANIM_LEGS,
                    anim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    100,
                );
            }

            //don't fire again until anim is done
            (*ps).weaponTime = (*ps).torsoTimer;
        }
    }

    // *********************************************************
    // WEAPON_FIRING
    // *********************************************************

    (*ps).weaponstate = WEAPON_FIRING;

    // C computes `amount = weaponData[pm->ps->weapon].energyPerShot;` here but never
    // reads it -- a dead assignment carried from the original (bound to `_` to keep it
    // warning-free while preserving the lookup).
    let _amount: c_int = weaponData[(*ps).weapon as usize].energyPerShot;

    let mut addTime: c_int = (*ps).weaponTime;

    (*ps).saberAttackSequence = (*ps).torsoAnim;
    if addTime == 0 {
        addTime = weaponData[(*ps).weapon as usize].fireTime;
    }
    (*ps).weaponTime = addTime;
}

/// `PM_SetSaberMove` (bg_saber.c:3416) — the saber-move FSM driver. Given a new
/// `LS_*` move it: selects the base animation from [`saberMoveData`], applies the
/// draw/putaway and staff/dual/force-level anim shifts, decides whether the move
/// plays on torso / legs / both, pushes it through [`PM_SetAnim`], then (if the
/// torso anim actually changed) commits `saberMove`/`saberBlocking`/`torsoAnim`,
/// fires the swing-start `EV_SABER_ATTACK`, and rolls broken-limb pain.
///
/// `short newMove`→`c_short`: C promotes it to `int` in every use, captured once as
/// `nm` (the promotion is the only place the `short` width is observable — at the
/// call boundary the caller's `int` is already truncated to `short`). `unsigned int
/// setflags`→`c_uint` (cast to `c_int` only at the `PM_SetAnim` boundary, as C does
/// implicitly). No oracle (drives the anim machine over the already-verified
/// `BG_Saber*`/`PM_Saber*` predicates, [`saberMoveData`], [`PM_GetSaberStance`] and
/// [`PM_SetAnim`]).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_SetSaberMove(newMove: c_short) {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;
    let nm: c_int = newMove as c_int;

    let mut setflags: c_uint = saberMoveData[nm as usize].animSetFlags;
    let mut anim: c_int = saberMoveData[nm as usize].animToUse;
    let mut parts: c_int = SETANIM_TORSO;

    if nm == LS_READY || nm == LS_A_FLIP_STAB || nm == LS_A_FLIP_SLASH {
        //finished with a kata (or in a special move) reset attack counter
        (*ps).saberAttackChainCount = 0;
    } else if BG_SaberInAttack(nm) != QFALSE {
        //continuing with a kata, increment attack counter
        (*ps).saberAttackChainCount += 1;
    }

    if (*ps).saberAttackChainCount > 16 {
        //for the sake of being able to send the value over the net within a reasonable bit count
        (*ps).saberAttackChainCount = 16;
    }

    if nm == LS_DRAW {
        if (*ps).fd.saberAnimLevel == SS_STAFF {
            anim = BOTH_S1_S7;
        } else if (*ps).fd.saberAnimLevel == SS_DUAL {
            anim = BOTH_S1_S6;
        }
    } else if nm == LS_PUTAWAY {
        if (*ps).fd.saberAnimLevel == SS_STAFF {
            anim = BOTH_S7_S1;
        } else if (*ps).fd.saberAnimLevel == SS_DUAL {
            anim = BOTH_S6_S1;
        }
    } else if (*ps).fd.saberAnimLevel == SS_STAFF && nm >= LS_S_TL2BR && nm < LS_REFLECT_LL {
        //staff has an entirely new set of anims, besides special attacks
        //FIXME: include ready and draw/putaway?
        //FIXME: get hand-made bounces and deflections?
        if nm >= LS_V1_BR && nm <= LS_REFLECT_LL {
            //there aren't 1-7, just 1, 6 and 7, so just set it
            anim = BOTH_P7_S7_T_ + (anim - BOTH_P1_S1_T_); //shift it up to the proper set
        } else {
            //add the appropriate animLevel
            anim += ((*ps).fd.saberAnimLevel - FORCE_LEVEL_1) * SABER_ANIM_GROUP_SIZE;
        }
    } else if (*ps).fd.saberAnimLevel == SS_DUAL && nm >= LS_S_TL2BR && nm < LS_REFLECT_LL {
        //akimbo has an entirely new set of anims, besides special attacks
        //FIXME: include ready and draw/putaway?
        //FIXME: get hand-made bounces and deflections?
        if nm >= LS_V1_BR && nm <= LS_REFLECT_LL {
            //there aren't 1-7, just 1, 6 and 7, so just set it
            anim = BOTH_P6_S6_T_ + (anim - BOTH_P1_S1_T_); //shift it up to the proper set
        } else {
            //add the appropriate animLevel
            anim += ((*ps).fd.saberAnimLevel - FORCE_LEVEL_1) * SABER_ANIM_GROUP_SIZE;
        }
    }
    /*
    else if ( newMove == LS_DRAW && pm->ps->SaberStaff() )
    {//hold saber out front as we turn it on
        //FIXME: need a real "draw" anim for this (and put-away)
        anim = BOTH_SABERSTAFF_STANCE;
    }
    */
    else if (*ps).fd.saberAnimLevel > FORCE_LEVEL_1
        && BG_SaberInIdle(nm) == QFALSE
        && PM_SaberInParry(nm) == QFALSE
        && PM_SaberInKnockaway(nm) == QFALSE
        && PM_SaberInBrokenParry(nm) == QFALSE
        && PM_SaberInReflect(nm) == QFALSE
        && BG_SaberInSpecial(nm) == QFALSE
    {
        //readies, parries and reflections have only 1 level
        anim += ((*ps).fd.saberAnimLevel - FORCE_LEVEL_1) * SABER_ANIM_GROUP_SIZE;
    }

    // If the move does the same animation as the last one, we need to force a restart...
    if saberMoveData[(*ps).saberMove as usize].animToUse == anim && nm > LS_PUTAWAY {
        setflags |= SETANIM_FLAG_RESTART as c_uint;
    }

    //saber torso anims should always be highest priority (4/12/02 - for special anims only)
    if (*ps).m_iVehicleNum == 0 {
        //if not riding a vehicle
        if BG_SaberInSpecial(nm) != QFALSE {
            setflags |= SETANIM_FLAG_OVERRIDE as c_uint;
        }
        /*
        if ( newMove == LS_A_LUNGE
            || newMove == LS_A_JUMP_T__B_
            || newMove == LS_A_BACKSTAB
            || newMove == LS_A_BACK
            || newMove == LS_A_BACK_CR
            || newMove == LS_A_FLIP_STAB
            || newMove == LS_A_FLIP_SLASH
            || newMove == LS_JUMPATTACK_DUAL
            || newMove == LS_A_BACKFLIP_ATK)
        {
            setflags |= SETANIM_FLAG_OVERRIDE;
        }
        */
    }
    if BG_InSaberStandAnim(anim) != QFALSE || anim == BOTH_STAND1 {
        anim = (*ps).legsAnim;

        if (anim >= BOTH_STAND1 && anim <= BOTH_STAND4TOATTACK2)
            || (anim >= TORSO_DROPWEAP1 && anim <= TORSO_WEAPONIDLE10)
        {
            //If standing then use the special saber stand anim
            anim = PM_GetSaberStance();
        }

        if (*ps).pm_flags & PMF_DUCKED != 0 {
            //Playing torso walk anims while crouched makes you look like a monkey
            anim = PM_GetSaberStance();
        }

        if anim == BOTH_WALKBACK1 || anim == BOTH_WALKBACK2 || anim == BOTH_WALK1 {
            //normal stance when walking backward so saber doesn't look like it's cutting through leg
            anim = PM_GetSaberStance();
        }

        if BG_InSlopeAnim(anim) != QFALSE {
            anim = PM_GetSaberStance();
        }

        parts = SETANIM_TORSO;
    }

    if (*ps).m_iVehicleNum == 0 {
        //if not riding a vehicle
        if nm == LS_JUMPATTACK_ARIAL_RIGHT || nm == LS_JUMPATTACK_ARIAL_LEFT {
            //force only on legs
            parts = SETANIM_LEGS;
        } else if nm == LS_A_LUNGE
            || nm == LS_A_JUMP_T__B_
            || nm == LS_A_BACKSTAB
            || nm == LS_A_BACK
            || nm == LS_A_BACK_CR
            || nm == LS_ROLL_STAB
            || nm == LS_A_FLIP_STAB
            || nm == LS_A_FLIP_SLASH
            || nm == LS_JUMPATTACK_DUAL
            || nm == LS_JUMPATTACK_ARIAL_LEFT
            || nm == LS_JUMPATTACK_ARIAL_RIGHT
            || nm == LS_JUMPATTACK_CART_LEFT
            || nm == LS_JUMPATTACK_CART_RIGHT
            || nm == LS_JUMPATTACK_STAFF_LEFT
            || nm == LS_JUMPATTACK_STAFF_RIGHT
            || nm == LS_A_BACKFLIP_ATK
            || nm == LS_STABDOWN
            || nm == LS_STABDOWN_STAFF
            || nm == LS_STABDOWN_DUAL
            || nm == LS_DUAL_SPIN_PROTECT
            || nm == LS_STAFF_SOULCAL
            || nm == LS_A1_SPECIAL
            || nm == LS_A2_SPECIAL
            || nm == LS_A3_SPECIAL
            || nm == LS_UPSIDE_DOWN_ATTACK
            || nm == LS_PULL_ATTACK_STAB
            || nm == LS_PULL_ATTACK_SWING
            || BG_KickMove(nm) != QFALSE
        {
            parts = SETANIM_BOTH;
        } else if BG_SpinningSaberAnim(anim) != QFALSE {
            //spins must be played on entire body
            parts = SETANIM_BOTH;
        } else if (*pmv).cmd.forwardmove == 0
            && (*pmv).cmd.rightmove == 0
            && (*pmv).cmd.upmove == 0
        {
            //not trying to run, duck or jump
            if BG_FlippingAnim((*ps).legsAnim) == QFALSE
                && BG_InRoll(ps, (*ps).legsAnim) == QFALSE
                && PM_InKnockDown(ps) == QFALSE
                && PM_JumpingAnim((*ps).legsAnim) == QFALSE
                && BG_InSpecialJump((*ps).legsAnim) == QFALSE
                && anim != PM_GetSaberStance()
                && (*ps).groundEntityNum != ENTITYNUM_NONE
                && (*ps).pm_flags & PMF_DUCKED == 0
            {
                parts = SETANIM_BOTH;
            } else if (*ps).pm_flags & PMF_DUCKED == 0
                && (nm == LS_SPINATTACK_DUAL || nm == LS_SPINATTACK)
            {
                parts = SETANIM_BOTH;
            }
        }

        PM_SetAnim(parts, anim, setflags as c_int, saberMoveData[nm as usize].blendTime);
        if parts != SETANIM_LEGS
            && ((*ps).legsAnim == BOTH_ARIAL_LEFT || (*ps).legsAnim == BOTH_ARIAL_RIGHT)
        {
            if (*ps).legsTimer > (*ps).torsoTimer {
                (*ps).legsTimer = (*ps).torsoTimer;
            }
        }
    }

    if (*ps).torsoAnim == anim {
        //successfully changed anims
        //special check for *starting* a saber swing
        //playing at attack
        if BG_SaberInAttack(nm) != QFALSE || BG_SaberInSpecialAttack(anim) != QFALSE {
            if (*ps).saberMove != nm {
                //wasn't playing that attack before
                if nm != LS_KICK_F
                    && nm != LS_KICK_B
                    && nm != LS_KICK_R
                    && nm != LS_KICK_L
                    && nm != LS_KICK_F_AIR
                    && nm != LS_KICK_B_AIR
                    && nm != LS_KICK_R_AIR
                    && nm != LS_KICK_L_AIR
                {
                    PM_AddEvent(EV_SABER_ATTACK);
                }

                if (*ps).brokenLimbs != 0 {
                    //randomly make pain sounds with a broken arm because we are suffering.
                    let mut iFactor: c_int = -1;

                    if (*ps).brokenLimbs & (1 << BROKENLIMB_RARM) != 0 {
                        //You're using it more. So it hurts more.
                        iFactor = 5;
                    } else if (*ps).brokenLimbs & (1 << BROKENLIMB_LARM) != 0 {
                        iFactor = 10;
                    }

                    if iFactor != -1 {
                        if PM_irand_timesync(0, iFactor) == 0 {
                            BG_AddPredictableEventToPlayerstate(
                                EV_PAIN,
                                PM_irand_timesync(1, 100),
                                ps,
                            );
                        }
                    }
                }
            }
        }

        if BG_SaberInSpecial(nm) != QFALSE && (*ps).weaponTime < (*ps).torsoTimer {
            //rww 01-02-03 - I think this will solve the issue of special attacks being interruptable, hopefully without side effects
            (*ps).weaponTime = (*ps).torsoTimer;
        }

        (*ps).saberMove = nm;
        (*ps).saberBlocking = saberMoveData[nm as usize].blocking;

        (*ps).torsoAnim = anim;

        if (*ps).weaponTime <= 0 {
            (*ps).saberBlocked = BLOCKED_NONE;
        }
    }
}

/// `BG_MySaber` (bg_saber.c:4100) — returns a pointer to the requested `saberNum`
/// for the given client, or null if the entity isn't in use / has no client struct,
/// or that saber slot carries no hilt model. This is the `QAGAME` (server game module)
/// build; the `CGAME` `clientInfo`/`cg_entities` branch is client-only and excluded.
/// No oracle (pointer return over the live `g_entities` array).
///
/// The C tests `!saber.model || !saber.model[0]`; `model` is a `char[MAX_QPATH]`
/// array whose address is never null, so that first test is vacuous and only the
/// `model[0]` empty-string check matters (matching the crate's established idiom).
///
/// # Safety
/// `clientNum` must be a valid index into `g_entities`; `saberNum` in `0..MAX_SABERS`.
pub unsafe fn BG_MySaber(clientNum: c_int, saberNum: c_int) -> *mut saberInfo_t {
    //returns a pointer to the requested saberNum
    let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(clientNum as usize);
    if (*ent).inuse != QFALSE && !(*ent).client.is_null() {
        if (*(*ent).client).saber[saberNum as usize].model[0] == 0 {
            //don't have saber anymore!
            return core::ptr::null_mut();
        }
        return &mut (*(*ent).client).saber[saberNum as usize];
    }
    core::ptr::null_mut()
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::{
        jka_bgsab_parryDebounce_ptr, jka_bgsab_saberMoveData_ptr,
        jka_bgsab_saberMoveTransitionAngle_ptr, jka_bgsab_transitionMove_ptr,
    };
    use core::ptr::addr_of;

    /// Element-wise parity of all four `bg_saber.c` data tables against the authentic
    /// C (verbatim copies in the oracle TU). `saberMoveData_t` carries raw pointers
    /// (no `PartialEq`), so each row is compared field-by-field; the `name` strings
    /// live at different addresses, so they are compared as `CStr` contents.
    #[test]
    fn saber_data_tables_match_c() {
        unsafe {
            // saberMoveData[] — field-by-field over all LS_MOVE_MAX rows.
            let n = LS_MOVE_MAX as usize;
            let rust = &*addr_of!(saberMoveData);
            let c = core::slice::from_raw_parts(jka_bgsab_saberMoveData_ptr(), n);
            for i in 0..n {
                let (r, cc) = (&rust[i], &c[i]);
                assert_eq!(CStr::from_ptr(r.name), CStr::from_ptr(cc.name), "saberMoveData[{i}].name");
                assert_eq!(r.animToUse, cc.animToUse, "saberMoveData[{i}].animToUse");
                assert_eq!(r.startQuad, cc.startQuad, "saberMoveData[{i}].startQuad");
                assert_eq!(r.endQuad, cc.endQuad, "saberMoveData[{i}].endQuad");
                assert_eq!(r.animSetFlags, cc.animSetFlags, "saberMoveData[{i}].animSetFlags");
                assert_eq!(r.blendTime, cc.blendTime, "saberMoveData[{i}].blendTime");
                assert_eq!(r.blocking, cc.blocking, "saberMoveData[{i}].blocking");
                assert_eq!(r.chain_idle, cc.chain_idle, "saberMoveData[{i}].chain_idle");
                assert_eq!(r.chain_attack, cc.chain_attack, "saberMoveData[{i}].chain_attack");
                assert_eq!(r.trailLength, cc.trailLength, "saberMoveData[{i}].trailLength");
            }

            // transitionMove[8][8] and saberMoveTransitionAngle[8][8] — row-major flat.
            let q = Q_NUM_QUADS as usize;
            let c_tm = core::slice::from_raw_parts(jka_bgsab_transitionMove_ptr(), q * q);
            let c_ta = core::slice::from_raw_parts(jka_bgsab_saberMoveTransitionAngle_ptr(), q * q);
            for i in 0..q {
                for j in 0..q {
                    assert_eq!(transitionMove[i][j], c_tm[i * q + j], "transitionMove[{i}][{j}]");
                    assert_eq!(
                        saberMoveTransitionAngle[i][j],
                        c_ta[i * q + j],
                        "saberMoveTransitionAngle[{i}][{j}]"
                    );
                }
            }

            // bg_parryDebounce[NUM_FORCE_POWER_LEVELS].
            let c_pd = core::slice::from_raw_parts(jka_bgsab_parryDebounce_ptr(), NUM_FORCE_POWER_LEVELS);
            assert_eq!(&bg_parryDebounce[..], c_pd, "bg_parryDebounce");
        }
    }

    /// Parity of the six self-contained move helpers against verbatim C bodies in
    /// `oracle/bg_saber_oracle.c`. The four stateless switch/range predicates sweep a
    /// wide integer domain (they index no array, so any int is safe — the sweep
    /// transitively verifies every case-label constant); `PM_SaberAttackChainAngle`
    /// indexes the move tables, so it is swept only over `{-1} ∪ [0, LS_MOVE_MAX)`; and
    /// `PM_SaberMoveQuadrantForMovement` is driven through every sign of its two move
    /// axes (the full `usercmd_t` layout is verified in `q_shared_h_oracle.c`).
    #[test]
    fn saber_self_contained_helpers_match_c() {
        use crate::oracle::{
            jka_PM_SaberMoveQuadrantForMovement, PM_AttackMoveForQuad as c_AttackMoveForQuad,
            PM_BrokenParryForParry as c_BrokenParryForParry,
            PM_SaberAttackChainAngle as c_SaberAttackChainAngle,
            PM_SaberInBounce as c_SaberInBounce, PM_SaberInBrokenParry as c_SaberInBrokenParry,
        };
        unsafe {
            // Stateless switch/range predicates over a domain covering every LS_*/Q_*.
            for x in -10..=2200 {
                assert_eq!(PM_AttackMoveForQuad(x), c_AttackMoveForQuad(x), "PM_AttackMoveForQuad({x})");
                assert_eq!(PM_SaberInBounce(x), c_SaberInBounce(x), "PM_SaberInBounce({x})");
                assert_eq!(
                    PM_SaberInBrokenParry(x),
                    c_SaberInBrokenParry(x),
                    "PM_SaberInBrokenParry({x})"
                );
                assert_eq!(
                    PM_BrokenParryForParry(x),
                    c_BrokenParryForParry(x),
                    "PM_BrokenParryForParry({x})"
                );
            }

            // PM_SaberAttackChainAngle indexes saberMoveData[]/saberMoveTransitionAngle[],
            // so only {-1} ∪ [0, LS_MOVE_MAX) is in-bounds (the -1 guard runs first).
            for m1 in -1..LS_MOVE_MAX {
                for m2 in -1..LS_MOVE_MAX {
                    assert_eq!(
                        PM_SaberAttackChainAngle(m1, m2),
                        c_SaberAttackChainAngle(m1, m2),
                        "PM_SaberAttackChainAngle({m1}, {m2})"
                    );
                }
            }

            // PM_SaberMoveQuadrantForMovement: every sign combination of the move axes.
            for f in -2i8..=2 {
                for r in -2i8..=2 {
                    let mut cmd: usercmd_t = core::mem::zeroed();
                    cmd.forwardmove = f;
                    cmd.rightmove = r;
                    assert_eq!(
                        PM_SaberMoveQuadrantForMovement(&mut cmd),
                        jka_PM_SaberMoveQuadrantForMovement(f as c_int, r as c_int),
                        "PM_SaberMoveQuadrantForMovement(f={f}, r={r})"
                    );
                }
            }
        }
    }

    /// Parity of the two self-contained saber-lock helpers against verbatim C
    /// bodies in `oracle/bg_saber_oracle.c`. `BG_CheckIncrementLockAnim` indexes
    /// no array, so its `anim` arg sweeps a wide domain (transitively verifying
    /// every `BOTH_LK_*` case label) crossed with every `winOrLose` value around
    /// the `SABERLOCK_*` range. `PM_SetAnimFrame` writes through a real (zeroed)
    /// `playerState_t`; the C side is reached via an int-marshalling wrapper over
    /// a minimal struct, so the comparison is on the resulting field value only.
    #[test]
    fn saber_lock_helpers_match_c() {
        use crate::oracle::{jka_PM_SetAnimFrame, BG_CheckIncrementLockAnim as c_CheckIncrementLockAnim};
        unsafe {
            for anim in -10..=2200 {
                for wol in -1..=7 {
                    assert_eq!(
                        BG_CheckIncrementLockAnim(anim, wol),
                        c_CheckIncrementLockAnim(anim, wol),
                        "BG_CheckIncrementLockAnim({anim}, {wol})"
                    );
                }
            }

            // PM_SetAnimFrame: the field takes the passed frame regardless of the
            // (ignored) torso/legs flags.
            for frame in [-5, 0, 1, 42, 12345] {
                let mut ps: playerState_t = core::mem::zeroed();
                ps.saberLockFrame = -999; // sentinel, must be overwritten
                PM_SetAnimFrame(&mut ps, frame, QTRUE, QFALSE);
                assert_eq!(ps.saberLockFrame, jka_PM_SetAnimFrame(frame), "PM_SetAnimFrame({frame})");
            }
        }
    }

    /// Parity of `BG_ForcePowerDrain` against the verbatim C body in
    /// `oracle/bg_saber_oracle.c`. The C side runs the body over a minimal
    /// `playerState_t` (reusing the already-verified `forcePowerNeeded` table) and
    /// returns the resulting `forcePower`; the Rust side runs over a real zeroed
    /// `playerState_t` set up in the same field order. The sweep covers the table-cost
    /// (`overrideAmt==0`) and override paths, the `FP_LEVITATION` special case across
    /// every velocity threshold and the div-by-zero (`rank 0`) guard, and the
    /// clamp-to-zero. The rank index stays in `[0, NUM_FORCE_POWER_LEVELS)` and
    /// `forcePower` in `[0, NUM_FORCE_POWERS)` so the table indexing is in-bounds.
    #[test]
    fn bg_force_power_drain_matches_c() {
        use crate::oracle::jka_BG_ForcePowerDrain;
        unsafe {
            // FP_HEAL..FP_TELEPATHY (0..6) — includes FP_LEVITATION (1).
            for force_power in 0..6i32 {
                for override_amt in [0, 1, 5, 50, 999] {
                    for level_power in 0..NUM_FORCE_POWER_LEVELS as i32 {
                        for level_lev in 0..NUM_FORCE_POWER_LEVELS as i32 {
                            for &vel_z in &[
                                -10.0f32, 0.0, 1.0, 50.0, 51.0, 100.0, 150.0, 200.0, 250.0, 300.0,
                            ] {
                                for start_fp in [0, 5, 16, 100] {
                                    let mut ps: playerState_t = core::mem::zeroed();
                                    ps.velocity[2] = vel_z;
                                    ps.fd.forcePower = start_fp;
                                    ps.fd.forcePowerLevel[force_power as usize] = level_power;
                                    ps.fd.forcePowerLevel[FP_LEVITATION as usize] = level_lev;
                                    BG_ForcePowerDrain(&mut ps, force_power, override_amt);
                                    assert_eq!(
                                        ps.fd.forcePower,
                                        jka_BG_ForcePowerDrain(
                                            force_power, override_amt, level_power, level_lev,
                                            vel_z, start_fp,
                                        ),
                                        "BG_ForcePowerDrain(fp={force_power}, ovr={override_amt}, lvlP={level_power}, lvlL={level_lev}, vz={vel_z}, start={start_fp})"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Parity of `PM_CanDoRollStab` against the verbatim C decision in
    /// `oracle/bg_saber_oracle.c`. The live function is pm-/`g_entities`-state-gated:
    /// it reads `pm->ps->weapon`/`clientNum` and resolves each saber via `BG_MySaber`
    /// over `g_entities`. The Rust side is exercised through the REAL function by
    /// staging a minimal `pmove_t`/`playerState_t` plus a single live client whose two
    /// saber slots' hilt-model presence + `saberFlags` are set per case; the C side
    /// receives the same `(weapon, present, flags)` decision inputs via an
    /// int-marshalling wrapper. The sweep crosses `WP_SABER` vs. a non-saber weapon
    /// with every {absent, present/no-flag, present/`SFL_NO_ROLL_STAB`,
    /// present/unrelated-flag} state for each of the two saber slots.
    #[test]
    fn pm_can_do_roll_stab_matches_c() {
        use crate::codemp::game::bg_weapons_h::WP_NONE;
        use crate::codemp::game::g_local::gclient_t;
        use crate::oracle::jka_PM_CanDoRollStab;
        unsafe {
            const CLIENT: c_int = 1;
            // (model-present, saberFlags) cases per saber slot.
            let cases: [(bool, c_int); 4] = [
                (false, 0),
                (true, 0),
                (true, SFL_NO_ROLL_STAB),
                (true, SFL_NO_STABDOWN), // an unrelated flag must NOT veto
            ];
            for weapon in [WP_SABER, WP_NONE] {
                for &(p0, f0) in &cases {
                    for &(p1, f1) in &cases {
                        let mut ps: playerState_t = core::mem::zeroed();
                        ps.weapon = weapon;
                        ps.clientNum = CLIENT;
                        let mut pmv: pmove_t = core::mem::zeroed();
                        pmv.ps = &mut ps;

                        let mut client: gclient_t = core::mem::zeroed();
                        client.saber[0].model[0] = if p0 { b'a' as c_char } else { 0 };
                        client.saber[0].saberFlags = f0;
                        client.saber[1].model[0] = if p1 { b'a' as c_char } else { 0 };
                        client.saber[1].saberFlags = f1;

                        let ent = core::ptr::addr_of_mut!(g_entities)
                            .cast::<gentity_t>()
                            .add(CLIENT as usize);
                        (*ent).inuse = QTRUE;
                        (*ent).client = &mut client;

                        pm = &mut pmv;
                        let got = PM_CanDoRollStab();
                        pm = core::ptr::null_mut();
                        (*ent).inuse = QFALSE;
                        (*ent).client = core::ptr::null_mut();

                        assert_eq!(
                            got,
                            jka_PM_CanDoRollStab(weapon, p0 as c_int, f0, p1 as c_int, f1),
                            "PM_CanDoRollStab(weapon={weapon}, s0=({p0},{f0}), s1=({p1},{f1}))"
                        );
                    }
                }
            }
        }
    }

    /// Parity of `PM_SaberJumpAttackMove2` against the verbatim C decision in
    /// `oracle/bg_saber_oracle.c`. The live function is pm-/`g_entities`-state-gated: it
    /// reads `pm->ps->clientNum`/`fd.saberAnimLevel` and resolves each saber via
    /// [`BG_MySaber`] over `g_entities`. The Rust side is exercised through the REAL
    /// function by staging a minimal `pmove_t`/`playerState_t` plus a single live client
    /// whose two saber slots' hilt-model presence + `jumpAtkFwdMove` are set per case;
    /// the C side receives the same `(present, move, saberAnimLevel)` decision inputs via
    /// an int-marshalling wrapper. The sweep crosses each saber's {absent, no-override
    /// (`LS_INVALID`), cancelled (`LS_NONE`), explicit override} state with both
    /// `SS_DUAL` (which delegates to [`PM_SaberDualJumpAttackMove`]) and a non-dual style.
    #[test]
    fn pm_saber_jump_attack_move2_matches_c() {
        use crate::codemp::game::g_local::gclient_t;
        use crate::oracle::jka_PM_SaberJumpAttackMove2;
        unsafe {
            const CLIENT: c_int = 1;
            // (model-present, jumpAtkFwdMove) cases per saber slot.
            let cases: [(bool, c_int); 4] = [
                (false, LS_INVALID), // absent (move ignored)
                (true, LS_INVALID),  // no override
                (true, LS_NONE),     // cancelled
                (true, LS_A_LUNGE),  // an explicit override move
            ];
            for anim_level in [SS_DUAL, SS_STAFF] {
                for &(p0, m0) in &cases {
                    for &(p1, m1) in &cases {
                        let mut ps: playerState_t = core::mem::zeroed();
                        ps.clientNum = CLIENT;
                        ps.fd.saberAnimLevel = anim_level;
                        let mut pmv: pmove_t = core::mem::zeroed();
                        pmv.ps = &mut ps;

                        let mut client: gclient_t = core::mem::zeroed();
                        client.saber[0].model[0] = if p0 { b'a' as c_char } else { 0 };
                        client.saber[0].jumpAtkFwdMove = m0;
                        client.saber[1].model[0] = if p1 { b'a' as c_char } else { 0 };
                        client.saber[1].jumpAtkFwdMove = m1;

                        let ent = core::ptr::addr_of_mut!(g_entities)
                            .cast::<gentity_t>()
                            .add(CLIENT as usize);
                        (*ent).inuse = QTRUE;
                        (*ent).client = &mut client;

                        pm = &mut pmv;
                        let got = PM_SaberJumpAttackMove2();
                        pm = core::ptr::null_mut();
                        (*ent).inuse = QFALSE;
                        (*ent).client = core::ptr::null_mut();

                        assert_eq!(
                            got,
                            jka_PM_SaberJumpAttackMove2(
                                p0 as c_int,
                                m0,
                                p1 as c_int,
                                m1,
                                anim_level
                            ),
                            "PM_SaberJumpAttackMove2(s0=({p0},{m0}), s1=({p1},{m1}), anim={anim_level})"
                        );
                    }
                }
            }
        }
    }
}
