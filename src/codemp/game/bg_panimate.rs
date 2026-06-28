//! `bg_panimate.rs` (←bg_panimate.c) — player-animation utilities.
//!
//! This unit ports the **"Animation utility functions (sequence checking)"**
//! block (bg_panimate.c lines 32-1631): the stateless `BG_*`/`PM_*` predicates
//! that classify an animation index (`animNumber_t`) or a saber move
//! (`saberMoveName_t`) by switching/range-checking over the enum value. The file
//! comment calls these "totally stateless and stupid" — they touch no globals and
//! no `pmove_t`/`playerState_t` state, so they port cleanly and are each verified
//! by an exhaustive integer input-sweep against the verbatim C
//! (`oracle/bg_panimate_oracle.c`).
//!
//! `BG_BrokenParryForAttack` / `PM_SaberBounceForAttack` index the
//! `saberMoveData[]` global (defined in `bg_saber.c`); they landed once that
//! table was ported and are verified by the same input-sweep (over the valid
//! `[0, LS_MOVE_MAX)` index range — both languages read the array, so the sweep
//! can't run off the ends).
//!
//! `BG_InRoll` / `PM_InKnockDown` / `PM_InRollComplete` / `PM_CanRollFromSoulCal`
//! take a `playerState_t *` and switch on `ps->legsAnim` / the `anim` argument,
//! reading `ps->legsTimer`. `playerState_t` is ported, so these landed; verified
//! by feeding a small set of crafted `playerState_t` values (the oracle TU
//! transcribes the struct) rather than an int sweep.
//!
//! The **animation-SETTER cluster** (bg_panimate.c 1633-2965) is also ported here:
//! `BG_FlipPart`, the file-scope animation-set globals (`bgAllAnims`/`bgNumAllAnims`/
//! `bgHumanoidAnimations`/`bgAllEvents`/…), `BG_InitAnimsets`/`BG_ClearAnimsets`/
//! `BG_AnimsetAlloc`/`BG_AnimsetFree`, the legs/torso anim starters
//! (`BG_StartLegsAnim`/`PM_ContinueLegsAnim`/`PM_ForceLegsAnim`/`BG_StartTorsoAnim`/
//! `PM_StartTorsoAnim`) and timer setters, `BG_SaberStartTransAnim`,
//! `BG_SetAnimFinal`/`PM_SetAnimFinal`, `BG_HasAnimation`/`BG_PickAnim`, and the
//! `BG_SetAnim`/`PM_SetAnim` keystone. **No ghoul2 / `bg_strap.h` / `mdxaBone_t`
//! dependency** — that surface lives in the animation-FILE parsers, not the setters.
//! The float timer math + control flow is C-oracle-verified bit-exact
//! (`oracle/bg_panimate_setters_oracle.c`); the thin `pm`-forwarders are verified
//! transitively through the same oracle, and the gate predicates / animset-state
//! inits behaviorally (their sub-calls are each independently oracle-verified).
//! **`BG_InitAnimsets` retires a not-yet-ported `G_InitGame` prologue call.**
//!
//! The animation-FILE parser `BG_ParseAnimationFile` (bg_panimate.c:2282) is also ported
//! (engine-trap file I/O, no oracle). Its two neighbours are **absent from this build**:
//! `SpewDebugStuffToFile` is `#ifdef CONVENIENT_ANIMATION_FILE_DEBUG_THING` (never defined)
//! and the `BG_ParseAnimationEvtFile` family is `#ifndef QAGAME` (cgame-only) — this crate
//! is the QAGAME server module. With that, **`bg_panimate.c` is fully ported** for the
//! server build.
//!
//! Faithful to `refs/raven-jediacademy/codemp/game/bg_panimate.c`; all original section
//! banners and per-case comments are carried. The dead `break;` after each
//! `return` in the C switches carries no behavior and is dropped (a `match` arm
//! has no fall-through).

#![allow(non_snake_case, non_upper_case_globals)]

use crate::codemp::game::anims::*;
use crate::codemp::game::bg_misc::BG_Alloc;
use crate::codemp::game::bg_pmove::{pm, PM_RunningAnim, PM_WalkingAnim};
use crate::codemp::game::bg_public::*;
use crate::codemp::game::bg_saber::{saberMoveData, BG_MySaber};
use crate::codemp::game::bg_weapons_h::WP_SABER;
use crate::codemp::cgame::animtable::animTable;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::{g_entities, Com_Error, Com_Printf};
use crate::codemp::game::q_shared::{COM_Parse, GetIDForString, Q_stricmp};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{
    playerState_t, qboolean, saberBlockedType_t, saberInfo_t, stringID_table_t, BLOCKED_LOWER_LEFT,
    BLOCKED_LOWER_RIGHT, BLOCKED_TOP, BLOCKED_UPPER_LEFT, ERR_DROP, FORCE_LEVEL_1, FORCE_LEVEL_3,
    FP_RAGE, FP_SPEED, FS_READ, MAX_CLIENTS, QFALSE, QTRUE,
};
use crate::trap;
use core::ffi::{c_char, c_int, CStr};
use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut};

/*
==============================================================================
BEGIN: Animation utility functions (sequence checking)
==============================================================================
*/
//Called regardless of pm validity:

// VVFIXME - Most of these functions are totally stateless and stupid. Don't
// need multiple copies of this, but it's much easier (and less likely to
// break in the future) if I keep separate namespace versions now.

pub fn BG_SaberStanceAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_STAND1            //not really a saberstance anim, actually... "saber off" stance
        | BOTH_STAND2          //single-saber, medium style
        | BOTH_SABERFAST_STANCE //single-saber, fast style
        | BOTH_SABERSLOW_STANCE //single-saber, strong style
        | BOTH_SABERSTAFF_STANCE //saber staff style
        | BOTH_SABERDUAL_STANCE => QTRUE, //dual saber style
        _ => QFALSE,
    }
}

pub fn BG_CrouchAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_SIT1                 //# Normal chair sit.
        | BOTH_SIT2               //# Lotus position.
        | BOTH_SIT3               //# Sitting in tired position: elbows on knees
        | BOTH_CROUCH1            //# Transition from standing to crouch
        | BOTH_CROUCH1IDLE        //# Crouching idle
        | BOTH_CROUCH1WALK        //# Walking while crouched
        | BOTH_CROUCH1WALKBACK    //# Walking while crouched
        | BOTH_CROUCH2TOSTAND1    //# going from crouch2 to stand1
        | BOTH_CROUCH3            //# Desann crouching down to Kyle (cin 9)
        | BOTH_KNEES1             //# Tavion on her knees
        | BOTH_CROUCHATTACKBACK1  //FIXME: not if in middle of anim?
        | BOTH_ROLL_STAB => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InSpecialJump(anim: c_int) -> qboolean {
    match anim {
        BOTH_WALL_RUN_RIGHT
        | BOTH_WALL_RUN_RIGHT_STOP
        | BOTH_WALL_RUN_RIGHT_FLIP
        | BOTH_WALL_RUN_LEFT
        | BOTH_WALL_RUN_LEFT_STOP
        | BOTH_WALL_RUN_LEFT_FLIP
        | BOTH_WALL_FLIP_RIGHT
        | BOTH_WALL_FLIP_LEFT
        | BOTH_FLIP_BACK1
        | BOTH_FLIP_BACK2
        | BOTH_FLIP_BACK3
        | BOTH_WALL_FLIP_BACK1
        | BOTH_BUTTERFLY_LEFT
        | BOTH_BUTTERFLY_RIGHT
        | BOTH_BUTTERFLY_FL1
        | BOTH_BUTTERFLY_FR1
        | BOTH_FJSS_TR_BL
        | BOTH_FJSS_TL_BR
        | BOTH_FORCELEAP2_T__B_
        | BOTH_JUMPFLIPSLASHDOWN1 //#
        | BOTH_JUMPFLIPSTABDOWN   //#
        | BOTH_JUMPATTACK6
        | BOTH_JUMPATTACK7
        | BOTH_ARIAL_LEFT
        | BOTH_ARIAL_RIGHT
        | BOTH_ARIAL_F1
        | BOTH_CARTWHEEL_LEFT
        | BOTH_CARTWHEEL_RIGHT
        | BOTH_FORCELONGLEAP_START
        | BOTH_FORCELONGLEAP_ATTACK
        | BOTH_FORCEWALLRUNFLIP_START
        | BOTH_FORCEWALLRUNFLIP_END
        | BOTH_FORCEWALLRUNFLIP_ALT
        | BOTH_FLIP_ATTACK7
        | BOTH_FLIP_HOLD7
        | BOTH_FLIP_LAND
        | BOTH_A7_SOULCAL => return QTRUE,
        _ => {}
    }
    if BG_InReboundJump(anim) != QFALSE {
        return QTRUE;
    }
    if BG_InReboundHold(anim) != QFALSE {
        return QTRUE;
    }
    if BG_InReboundRelease(anim) != QFALSE {
        return QTRUE;
    }
    if BG_InBackFlip(anim) != QFALSE {
        return QTRUE;
    }
    QFALSE
}

pub fn BG_InSaberStandAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_SABERFAST_STANCE
        | BOTH_STAND2
        | BOTH_SABERSLOW_STANCE
        | BOTH_SABERDUAL_STANCE
        | BOTH_SABERSTAFF_STANCE => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InReboundJump(anim: c_int) -> qboolean {
    match anim {
        BOTH_FORCEWALLREBOUND_FORWARD
        | BOTH_FORCEWALLREBOUND_LEFT
        | BOTH_FORCEWALLREBOUND_BACK
        | BOTH_FORCEWALLREBOUND_RIGHT => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InReboundHold(anim: c_int) -> qboolean {
    match anim {
        BOTH_FORCEWALLHOLD_FORWARD
        | BOTH_FORCEWALLHOLD_LEFT
        | BOTH_FORCEWALLHOLD_BACK
        | BOTH_FORCEWALLHOLD_RIGHT => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InReboundRelease(anim: c_int) -> qboolean {
    match anim {
        BOTH_FORCEWALLRELEASE_FORWARD
        | BOTH_FORCEWALLRELEASE_LEFT
        | BOTH_FORCEWALLRELEASE_BACK
        | BOTH_FORCEWALLRELEASE_RIGHT => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InBackFlip(anim: c_int) -> qboolean {
    match anim {
        BOTH_FLIP_BACK1 | BOTH_FLIP_BACK2 | BOTH_FLIP_BACK3 => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_DirectFlippingAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_FLIP_F   //# Flip forward
        | BOTH_FLIP_B //# Flip backwards
        | BOTH_FLIP_L //# Flip left
        | BOTH_FLIP_R => QTRUE, //# Flip right
        _ => QFALSE,
    }
}

// `qboolean BG_SaberInAttackPure( int move )` (bg_panimate.c:219) — the bare
// attack-quadrant range check, without BG_SaberInAttack's extra special-move cases.
pub fn BG_SaberInAttackPure(r#move: c_int) -> qboolean {
    if r#move >= LS_A_TL2BR && r#move <= LS_A_T2B {
        return QTRUE;
    }
    QFALSE
}

pub fn BG_SaberInAttack(r#move: c_int) -> qboolean {
    if r#move >= LS_A_TL2BR && r#move <= LS_A_T2B {
        return QTRUE;
    }
    match r#move {
        LS_A_BACK
        | LS_A_BACK_CR
        | LS_A_BACKSTAB
        | LS_ROLL_STAB
        | LS_A_LUNGE
        | LS_A_JUMP_T__B_
        | LS_A_FLIP_STAB
        | LS_A_FLIP_SLASH
        | LS_JUMPATTACK_DUAL
        | LS_JUMPATTACK_ARIAL_LEFT
        | LS_JUMPATTACK_ARIAL_RIGHT
        | LS_JUMPATTACK_CART_LEFT
        | LS_JUMPATTACK_CART_RIGHT
        | LS_JUMPATTACK_STAFF_LEFT
        | LS_JUMPATTACK_STAFF_RIGHT
        | LS_BUTTERFLY_LEFT
        | LS_BUTTERFLY_RIGHT
        | LS_A_BACKFLIP_ATK
        | LS_SPINATTACK_DUAL
        | LS_SPINATTACK
        | LS_LEAP_ATTACK
        | LS_SWOOP_ATTACK_RIGHT
        | LS_SWOOP_ATTACK_LEFT
        | LS_TAUNTAUN_ATTACK_RIGHT
        | LS_TAUNTAUN_ATTACK_LEFT
        | LS_KICK_F
        | LS_KICK_B
        | LS_KICK_R
        | LS_KICK_L
        | LS_KICK_S
        | LS_KICK_BF
        | LS_KICK_RL
        | LS_KICK_F_AIR
        | LS_KICK_B_AIR
        | LS_KICK_R_AIR
        | LS_KICK_L_AIR
        | LS_STABDOWN
        | LS_STABDOWN_STAFF
        | LS_STABDOWN_DUAL
        | LS_DUAL_SPIN_PROTECT
        | LS_STAFF_SOULCAL
        | LS_A1_SPECIAL
        | LS_A2_SPECIAL
        | LS_A3_SPECIAL
        | LS_UPSIDE_DOWN_ATTACK
        | LS_PULL_ATTACK_STAB
        | LS_PULL_ATTACK_SWING
        | LS_SPINATTACK_ALORA
        | LS_DUAL_FB
        | LS_DUAL_LR
        | LS_HILT_BASH => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_SaberInKata(saberMove: c_int) -> qboolean {
    match saberMove {
        LS_A1_SPECIAL
        | LS_A2_SPECIAL
        | LS_A3_SPECIAL
        | LS_DUAL_SPIN_PROTECT
        | LS_STAFF_SOULCAL => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InKataAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_A6_SABERPROTECT
        | BOTH_A7_SOULCAL
        | BOTH_A1_SPECIAL
        | BOTH_A2_SPECIAL
        | BOTH_A3_SPECIAL => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_SaberInSpecial(r#move: c_int) -> qboolean {
    match r#move {
        LS_A_BACK
        | LS_A_BACK_CR
        | LS_A_BACKSTAB
        | LS_ROLL_STAB
        | LS_A_LUNGE
        | LS_A_JUMP_T__B_
        | LS_A_FLIP_STAB
        | LS_A_FLIP_SLASH
        | LS_JUMPATTACK_DUAL
        | LS_JUMPATTACK_ARIAL_LEFT
        | LS_JUMPATTACK_ARIAL_RIGHT
        | LS_JUMPATTACK_CART_LEFT
        | LS_JUMPATTACK_CART_RIGHT
        | LS_JUMPATTACK_STAFF_LEFT
        | LS_JUMPATTACK_STAFF_RIGHT
        | LS_BUTTERFLY_LEFT
        | LS_BUTTERFLY_RIGHT
        | LS_A_BACKFLIP_ATK
        | LS_SPINATTACK_DUAL
        | LS_SPINATTACK
        | LS_LEAP_ATTACK
        | LS_SWOOP_ATTACK_RIGHT
        | LS_SWOOP_ATTACK_LEFT
        | LS_TAUNTAUN_ATTACK_RIGHT
        | LS_TAUNTAUN_ATTACK_LEFT
        | LS_KICK_F
        | LS_KICK_B
        | LS_KICK_R
        | LS_KICK_L
        | LS_KICK_S
        | LS_KICK_BF
        | LS_KICK_RL
        | LS_KICK_F_AIR
        | LS_KICK_B_AIR
        | LS_KICK_R_AIR
        | LS_KICK_L_AIR
        | LS_STABDOWN
        | LS_STABDOWN_STAFF
        | LS_STABDOWN_DUAL
        | LS_DUAL_SPIN_PROTECT
        | LS_STAFF_SOULCAL
        | LS_A1_SPECIAL
        | LS_A2_SPECIAL
        | LS_A3_SPECIAL
        | LS_UPSIDE_DOWN_ATTACK
        | LS_PULL_ATTACK_STAB
        | LS_PULL_ATTACK_SWING
        | LS_SPINATTACK_ALORA
        | LS_DUAL_FB
        | LS_DUAL_LR
        | LS_HILT_BASH => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_KickMove(r#move: c_int) -> qboolean {
    match r#move {
        LS_KICK_F
        | LS_KICK_B
        | LS_KICK_R
        | LS_KICK_L
        | LS_KICK_S
        | LS_KICK_BF
        | LS_KICK_RL
        | LS_KICK_F_AIR
        | LS_KICK_B_AIR
        | LS_KICK_R_AIR
        | LS_KICK_L_AIR
        | LS_HILT_BASH => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_SaberInIdle(r#move: c_int) -> qboolean {
    match r#move {
        LS_NONE | LS_READY | LS_DRAW | LS_PUTAWAY => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InExtraDefenseSaberMove(r#move: c_int) -> qboolean {
    match r#move {
        LS_SPINATTACK_DUAL
        | LS_SPINATTACK
        | LS_DUAL_SPIN_PROTECT
        | LS_STAFF_SOULCAL
        | LS_A1_SPECIAL
        | LS_A2_SPECIAL
        | LS_A3_SPECIAL
        | LS_JUMPATTACK_DUAL => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_FlippingAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_FLIP_F   //# Flip forward
        | BOTH_FLIP_B //# Flip backwards
        | BOTH_FLIP_L //# Flip left
        | BOTH_FLIP_R //# Flip right
        | BOTH_WALL_RUN_RIGHT_FLIP
        | BOTH_WALL_RUN_LEFT_FLIP
        | BOTH_WALL_FLIP_RIGHT
        | BOTH_WALL_FLIP_LEFT
        | BOTH_FLIP_BACK1
        | BOTH_FLIP_BACK2
        | BOTH_FLIP_BACK3
        | BOTH_WALL_FLIP_BACK1
        //Not really flips, but...
        | BOTH_WALL_RUN_RIGHT
        | BOTH_WALL_RUN_LEFT
        | BOTH_WALL_RUN_RIGHT_STOP
        | BOTH_WALL_RUN_LEFT_STOP
        | BOTH_BUTTERFLY_LEFT
        | BOTH_BUTTERFLY_RIGHT
        | BOTH_BUTTERFLY_FL1
        | BOTH_BUTTERFLY_FR1
        //
        | BOTH_ARIAL_LEFT
        | BOTH_ARIAL_RIGHT
        | BOTH_ARIAL_F1
        | BOTH_CARTWHEEL_LEFT
        | BOTH_CARTWHEEL_RIGHT
        | BOTH_JUMPFLIPSLASHDOWN1
        | BOTH_JUMPFLIPSTABDOWN
        | BOTH_JUMPATTACK6
        | BOTH_JUMPATTACK7
        //JKA
        | BOTH_FORCEWALLRUNFLIP_END
        | BOTH_FORCEWALLRUNFLIP_ALT
        | BOTH_FLIP_ATTACK7
        | BOTH_A7_SOULCAL => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_SpinningSaberAnim(anim: c_int) -> qboolean {
    match anim {
        //level 1 - FIXME: level 1 will have *no* spins
        BOTH_T1_BR_BL
        | BOTH_T1__R__L
        | BOTH_T1__R_BL
        | BOTH_T1_TR_BL
        | BOTH_T1_BR_TL
        | BOTH_T1_BR__L
        | BOTH_T1_TL_BR
        | BOTH_T1__L_BR
        | BOTH_T1__L__R
        | BOTH_T1_BL_BR
        | BOTH_T1_BL__R
        | BOTH_T1_BL_TR
        //level 2
        | BOTH_T2_BR__L
        | BOTH_T2_BR_BL
        | BOTH_T2__R_BL
        | BOTH_T2__L_BR
        | BOTH_T2_BL_BR
        | BOTH_T2_BL__R
        //level 3
        | BOTH_T3_BR__L
        | BOTH_T3_BR_BL
        | BOTH_T3__R_BL
        | BOTH_T3__L_BR
        | BOTH_T3_BL_BR
        | BOTH_T3_BL__R
        //level 4
        | BOTH_T4_BR__L
        | BOTH_T4_BR_BL
        | BOTH_T4__R_BL
        | BOTH_T4__L_BR
        | BOTH_T4_BL_BR
        | BOTH_T4_BL__R
        //level 5
        | BOTH_T5_BR_BL
        | BOTH_T5__R__L
        | BOTH_T5__R_BL
        | BOTH_T5_TR_BL
        | BOTH_T5_BR_TL
        | BOTH_T5_BR__L
        | BOTH_T5_TL_BR
        | BOTH_T5__L_BR
        | BOTH_T5__L__R
        | BOTH_T5_BL_BR
        | BOTH_T5_BL__R
        | BOTH_T5_BL_TR
        //level 6
        | BOTH_T6_BR_TL
        | BOTH_T6__R_TL
        | BOTH_T6__R__L
        | BOTH_T6__R_BL
        | BOTH_T6_TR_TL
        | BOTH_T6_TR__L
        | BOTH_T6_TR_BL
        | BOTH_T6_T__TL
        | BOTH_T6_T__BL
        | BOTH_T6_TL_BR
        | BOTH_T6__L_BR
        | BOTH_T6__L__R
        | BOTH_T6_TL__R
        | BOTH_T6_TL_TR
        | BOTH_T6__L_TR
        | BOTH_T6__L_T_
        | BOTH_T6_BL_T_
        | BOTH_T6_BR__L
        | BOTH_T6_BR_BL
        | BOTH_T6_BL_BR
        | BOTH_T6_BL__R
        | BOTH_T6_BL_TR
        //level 7
        | BOTH_T7_BR_TL
        | BOTH_T7_BR__L
        | BOTH_T7_BR_BL
        | BOTH_T7__R__L
        | BOTH_T7__R_BL
        | BOTH_T7_TR__L
        | BOTH_T7_T___R
        | BOTH_T7_TL_BR
        | BOTH_T7__L_BR
        | BOTH_T7__L__R
        | BOTH_T7_BL_BR
        | BOTH_T7_BL__R
        | BOTH_T7_BL_TR
        | BOTH_T7_TL_TR
        | BOTH_T7_T__BR
        | BOTH_T7__L_TR
        | BOTH_V7_BL_S7
        //special
        //case BOTH_A2_STABBACK1:
        | BOTH_ATTACK_BACK
        | BOTH_CROUCHATTACKBACK1
        | BOTH_BUTTERFLY_LEFT
        | BOTH_BUTTERFLY_RIGHT
        | BOTH_FJSS_TR_BL
        | BOTH_FJSS_TL_BR
        | BOTH_JUMPFLIPSLASHDOWN1
        | BOTH_JUMPFLIPSTABDOWN => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_SaberInSpecialAttack(anim: c_int) -> qboolean {
    match anim {
        BOTH_A2_STABBACK1
        | BOTH_ATTACK_BACK
        | BOTH_CROUCHATTACKBACK1
        | BOTH_ROLL_STAB
        | BOTH_BUTTERFLY_LEFT
        | BOTH_BUTTERFLY_RIGHT
        | BOTH_BUTTERFLY_FL1
        | BOTH_BUTTERFLY_FR1
        | BOTH_FJSS_TR_BL
        | BOTH_FJSS_TL_BR
        | BOTH_LUNGE2_B__T_
        | BOTH_FORCELEAP2_T__B_
        | BOTH_JUMPFLIPSLASHDOWN1 //#
        | BOTH_JUMPFLIPSTABDOWN   //#
        | BOTH_JUMPATTACK6
        | BOTH_JUMPATTACK7
        | BOTH_SPINATTACK6
        | BOTH_SPINATTACK7
        | BOTH_FORCELONGLEAP_ATTACK
        | BOTH_VS_ATR_S
        | BOTH_VS_ATL_S
        | BOTH_VT_ATR_S
        | BOTH_VT_ATL_S
        | BOTH_A7_KICK_F
        | BOTH_A7_KICK_B
        | BOTH_A7_KICK_R
        | BOTH_A7_KICK_L
        | BOTH_A7_KICK_S
        | BOTH_A7_KICK_BF
        | BOTH_A7_KICK_RL
        | BOTH_A7_KICK_F_AIR
        | BOTH_A7_KICK_B_AIR
        | BOTH_A7_KICK_R_AIR
        | BOTH_A7_KICK_L_AIR
        | BOTH_STABDOWN
        | BOTH_STABDOWN_STAFF
        | BOTH_STABDOWN_DUAL
        | BOTH_A6_SABERPROTECT
        | BOTH_A7_SOULCAL
        | BOTH_A1_SPECIAL
        | BOTH_A2_SPECIAL
        | BOTH_A3_SPECIAL
        | BOTH_FLIP_ATTACK7
        | BOTH_PULL_IMPALE_STAB
        | BOTH_PULL_IMPALE_SWING
        | BOTH_ALORA_SPIN_SLASH
        | BOTH_A6_FB
        | BOTH_A6_LR
        | BOTH_A7_HILT => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_KickingAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_A7_KICK_F
        | BOTH_A7_KICK_B
        | BOTH_A7_KICK_R
        | BOTH_A7_KICK_L
        | BOTH_A7_KICK_S
        | BOTH_A7_KICK_BF
        | BOTH_A7_KICK_RL
        | BOTH_A7_KICK_F_AIR
        | BOTH_A7_KICK_B_AIR
        | BOTH_A7_KICK_R_AIR
        | BOTH_A7_KICK_L_AIR
        | BOTH_A7_HILT
        //NOT kicks, but do kick traces anyway
        | BOTH_GETUP_BROLL_B
        | BOTH_GETUP_BROLL_F
        | BOTH_GETUP_FROLL_B
        | BOTH_GETUP_FROLL_F => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InGrappleMove(anim: c_int) -> c_int {
    match anim {
        BOTH_KYLE_GRAB | BOTH_KYLE_MISS => return 1, //grabbing at someone
        BOTH_KYLE_PA_1 | BOTH_KYLE_PA_2 => return 2, //beating the shit out of someone
        BOTH_PLAYER_PA_1 | BOTH_PLAYER_PA_2 | BOTH_PLAYER_PA_FLY => return 3, //getting the shit beaten out of you
        _ => {}
    }
    0
}

pub fn BG_BrokenParryForAttack(r#move: c_int) -> c_int {
    //Our attack was knocked away by a knockaway parry
    //FIXME: need actual anims for this
    //FIXME: need to know which side of the saber was hit!  For now, we presume the saber gets knocked away from the center
    let start_quad = unsafe { (*addr_of!(saberMoveData))[r#move as usize].startQuad };
    match start_quad {
        Q_B => LS_V1_B_,
        Q_BR => LS_V1_BR,
        Q_R => LS_V1__R,
        Q_TR => LS_V1_TR,
        Q_T => LS_V1_T_,
        Q_TL => LS_V1_TL,
        Q_L => LS_V1__L,
        Q_BL => LS_V1_BL,
        _ => LS_NONE,
    }
}

pub fn BG_BrokenParryForParry(r#move: c_int) -> c_int {
    //FIXME: need actual anims for this
    //FIXME: need to know which side of the saber was hit!  For now, we presume the saber gets knocked away from the center
    match r#move {
        LS_PARRY_UP => {
            //Hmm... since we don't know what dir the hit came from, randomly pick knock down or knock back
            if Q_irand(0, 1) != 0 {
                LS_H1_B_
            } else {
                LS_H1_T_
            }
        }
        LS_PARRY_UR => LS_H1_TR,
        LS_PARRY_UL => LS_H1_TL,
        LS_PARRY_LR => LS_H1_BR,
        LS_PARRY_LL => LS_H1_BL,
        LS_READY => LS_H1_B_, //???
        _ => LS_NONE,
    }
}

pub fn BG_KnockawayForParry(r#move: c_int) -> c_int {
    //FIXME: need actual anims for this
    //FIXME: need to know which side of the saber was hit!  For now, we presume the saber gets knocked away from the center
    match r#move as saberBlockedType_t {
        BLOCKED_TOP => LS_K1_T_,         //LS_PARRY_UP: push up
        BLOCKED_UPPER_LEFT => LS_K1_TL,  //LS_PARRY_UL: push up and to left
        BLOCKED_LOWER_RIGHT => LS_K1_BR, //LS_PARRY_LR: push down and to left
        BLOCKED_LOWER_LEFT => LS_K1_BL,  //LS_PARRY_LL: push down and to right
        // case BLOCKED_UPPER_RIGHT (LS_PARRY_UR) + default (LS_READY): push up, slightly to right
        _ => LS_K1_TR,
    }
    //return LS_NONE;
}

pub unsafe fn BG_InRoll(ps: *mut playerState_t, anim: c_int) -> qboolean {
    match anim {
        BOTH_GETUP_BROLL_B | BOTH_GETUP_BROLL_F | BOTH_GETUP_BROLL_L | BOTH_GETUP_BROLL_R
        | BOTH_GETUP_FROLL_B | BOTH_GETUP_FROLL_F | BOTH_GETUP_FROLL_L | BOTH_GETUP_FROLL_R
        | BOTH_ROLL_F | BOTH_ROLL_B | BOTH_ROLL_R | BOTH_ROLL_L => {
            if (*ps).legsTimer > 0 {
                return QTRUE;
            }
        }
        _ => {}
    }
    QFALSE
}

pub fn BG_InSpecialDeathAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_DEATH_ROLL        //# Death anim from a roll
        | BOTH_DEATH_FLIP      //# Death anim from a flip
        | BOTH_DEATH_SPIN_90_R //# Death anim when facing 90 degrees right
        | BOTH_DEATH_SPIN_90_L //# Death anim when facing 90 degrees left
        | BOTH_DEATH_SPIN_180  //# Death anim when facing backwards
        | BOTH_DEATH_LYING_UP  //# Death anim when lying on back
        | BOTH_DEATH_LYING_DN  //# Death anim when lying on front
        | BOTH_DEATH_FALLING_DN //# Death anim when falling on face
        | BOTH_DEATH_FALLING_UP //# Death anim when falling on back
        | BOTH_DEATH_CROUCHED => QTRUE, //# Death anim when crouched
        _ => QFALSE,
    }
}

pub fn BG_InDeathAnim(anim: c_int) -> qboolean {
    //Purposely does not cover stumbledeath and falldeath...
    match anim {
        BOTH_DEATH1      //# First Death anim
        | BOTH_DEATH2    //# Second Death anim
        | BOTH_DEATH3    //# Third Death anim
        | BOTH_DEATH4    //# Fourth Death anim
        | BOTH_DEATH5    //# Fifth Death anim
        | BOTH_DEATH6    //# Sixth Death anim
        | BOTH_DEATH7    //# Seventh Death anim
        | BOTH_DEATH8    //#
        | BOTH_DEATH9    //#
        | BOTH_DEATH10   //#
        | BOTH_DEATH11   //#
        | BOTH_DEATH12   //#
        | BOTH_DEATH13   //#
        | BOTH_DEATH14   //#
        | BOTH_DEATH14_UNGRIP //# Desann's end death (cin #35)
        | BOTH_DEATH14_SITUP  //# Tavion sitting up after having been thrown (cin #23)
        | BOTH_DEATH15   //#
        | BOTH_DEATH16   //#
        | BOTH_DEATH17   //#
        | BOTH_DEATH18   //#
        | BOTH_DEATH19   //#
        | BOTH_DEATH20   //#
        | BOTH_DEATH21   //#
        | BOTH_DEATH22   //#
        | BOTH_DEATH23   //#
        | BOTH_DEATH24   //#
        | BOTH_DEATH25   //#
        | BOTH_DEATHFORWARD1  //# First Death in which they get thrown forward
        | BOTH_DEATHFORWARD2  //# Second Death in which they get thrown forward
        | BOTH_DEATHFORWARD3  //# Tavion's falling in cin# 23
        | BOTH_DEATHBACKWARD1 //# First Death in which they get thrown backward
        | BOTH_DEATHBACKWARD2 //# Second Death in which they get thrown backward
        | BOTH_DEATH1IDLE     //# Idle while close to death
        | BOTH_LYINGDEATH1    //# Death to play when killed lying down
        | BOTH_STUMBLEDEATH1  //# Stumble forward and fall face first death
        | BOTH_FALLDEATH1     //# Fall forward off a high cliff and splat death - start
        | BOTH_FALLDEATH1INAIR //# Fall forward off a high cliff and splat death - loop
        | BOTH_FALLDEATH1LAND  //# Fall forward off a high cliff and splat death - hit bottom
        //# #sep case BOTH_ DEAD POSES # Should be last frame of corresponding previous anims
        | BOTH_DEAD1     //# First Death finished pose
        | BOTH_DEAD2     //# Second Death finished pose
        | BOTH_DEAD3     //# Third Death finished pose
        | BOTH_DEAD4     //# Fourth Death finished pose
        | BOTH_DEAD5     //# Fifth Death finished pose
        | BOTH_DEAD6     //# Sixth Death finished pose
        | BOTH_DEAD7     //# Seventh Death finished pose
        | BOTH_DEAD8     //#
        | BOTH_DEAD9     //#
        | BOTH_DEAD10    //#
        | BOTH_DEAD11    //#
        | BOTH_DEAD12    //#
        | BOTH_DEAD13    //#
        | BOTH_DEAD14    //#
        | BOTH_DEAD15    //#
        | BOTH_DEAD16    //#
        | BOTH_DEAD17    //#
        | BOTH_DEAD18    //#
        | BOTH_DEAD19    //#
        | BOTH_DEAD20    //#
        | BOTH_DEAD21    //#
        | BOTH_DEAD22    //#
        | BOTH_DEAD23    //#
        | BOTH_DEAD24    //#
        | BOTH_DEAD25    //#
        | BOTH_DEADFORWARD1  //# First thrown forward death finished pose
        | BOTH_DEADFORWARD2  //# Second thrown forward death finished pose
        | BOTH_DEADBACKWARD1 //# First thrown backward death finished pose
        | BOTH_DEADBACKWARD2 //# Second thrown backward death finished pose
        | BOTH_LYINGDEAD1    //# Killed lying down death finished pose
        | BOTH_STUMBLEDEAD1  //# Stumble forward death finished pose
        | BOTH_FALLDEAD1LAND //# Fall forward and splat death finished pose
        //# #sep case BOTH_ DEAD TWITCH/FLOP # React to being shot from death poses
        | BOTH_DEADFLOP1 //# React to being shot from First Death finished pose
        | BOTH_DEADFLOP2 //# React to being shot from Second Death finished pose
        | BOTH_DISMEMBER_HEAD1  //#
        | BOTH_DISMEMBER_TORSO1 //#
        | BOTH_DISMEMBER_LLEG   //#
        | BOTH_DISMEMBER_RLEG   //#
        | BOTH_DISMEMBER_RARM   //#
        | BOTH_DISMEMBER_LARM => QTRUE, //#
        _ => BG_InSpecialDeathAnim(anim),
    }
}

pub fn BG_InKnockDownOnly(anim: c_int) -> qboolean {
    match anim {
        BOTH_KNOCKDOWN1
        | BOTH_KNOCKDOWN2
        | BOTH_KNOCKDOWN3
        | BOTH_KNOCKDOWN4
        | BOTH_KNOCKDOWN5 => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InSaberLockOld(anim: c_int) -> qboolean {
    match anim {
        BOTH_BF2LOCK | BOTH_BF1LOCK | BOTH_CWCIRCLELOCK | BOTH_CCWCIRCLELOCK => QTRUE,
        _ => QFALSE,
    }
}

pub fn BG_InSaberLock(anim: c_int) -> qboolean {
    match anim {
        BOTH_LK_S_DL_S_L_1    //lock if I'm using single vs. a dual
        | BOTH_LK_S_DL_T_L_1  //lock if I'm using single vs. a dual
        | BOTH_LK_S_ST_S_L_1  //lock if I'm using single vs. a staff
        | BOTH_LK_S_ST_T_L_1  //lock if I'm using single vs. a staff
        | BOTH_LK_S_S_S_L_1   //lock if I'm using single vs. a single and I initiated
        | BOTH_LK_S_S_T_L_1   //lock if I'm using single vs. a single and I initiated
        | BOTH_LK_DL_DL_S_L_1 //lock if I'm using dual vs. dual and I initiated
        | BOTH_LK_DL_DL_T_L_1 //lock if I'm using dual vs. dual and I initiated
        | BOTH_LK_DL_ST_S_L_1 //lock if I'm using dual vs. a staff
        | BOTH_LK_DL_ST_T_L_1 //lock if I'm using dual vs. a staff
        | BOTH_LK_DL_S_S_L_1  //lock if I'm using dual vs. a single
        | BOTH_LK_DL_S_T_L_1  //lock if I'm using dual vs. a single
        | BOTH_LK_ST_DL_S_L_1 //lock if I'm using staff vs. dual
        | BOTH_LK_ST_DL_T_L_1 //lock if I'm using staff vs. dual
        | BOTH_LK_ST_ST_S_L_1 //lock if I'm using staff vs. a staff and I initiated
        | BOTH_LK_ST_ST_T_L_1 //lock if I'm using staff vs. a staff and I initiated
        | BOTH_LK_ST_S_S_L_1  //lock if I'm using staff vs. a single
        | BOTH_LK_ST_S_T_L_1  //lock if I'm using staff vs. a single
        | BOTH_LK_S_S_S_L_2
        | BOTH_LK_S_S_T_L_2
        | BOTH_LK_DL_DL_S_L_2
        | BOTH_LK_DL_DL_T_L_2
        | BOTH_LK_ST_ST_S_L_2
        | BOTH_LK_ST_ST_T_L_2 => QTRUE,
        _ => BG_InSaberLockOld(anim),
    }
    //return qfalse;
}

//Called only where pm is valid (not all require pm, but some do):
pub fn PM_InCartwheel(anim: c_int) -> qboolean {
    match anim {
        BOTH_ARIAL_LEFT
        | BOTH_ARIAL_RIGHT
        | BOTH_ARIAL_F1
        | BOTH_CARTWHEEL_LEFT
        | BOTH_CARTWHEEL_RIGHT => QTRUE,
        _ => QFALSE,
    }
}

pub unsafe fn BG_InKnockDownOnGround(ps: *mut playerState_t) -> qboolean {
    match (*ps).legsAnim {
        BOTH_KNOCKDOWN1 | BOTH_KNOCKDOWN2 | BOTH_KNOCKDOWN3 | BOTH_KNOCKDOWN4
        | BOTH_KNOCKDOWN5 | BOTH_RELEASED => {
            //if ( PM_AnimLength( g_entities[ps->clientNum].client->clientInfo.animFileIndex, (animNumber_t)ps->legsAnim ) - ps->legsAnimTimer > 300 )
            {
                //at end of fall down anim
                return QTRUE;
            }
        }
        BOTH_GETUP1 | BOTH_GETUP2 | BOTH_GETUP3 | BOTH_GETUP4 | BOTH_GETUP5
        | BOTH_GETUP_CROUCH_F1 | BOTH_GETUP_CROUCH_B1 | BOTH_FORCE_GETUP_F1
        | BOTH_FORCE_GETUP_F2 | BOTH_FORCE_GETUP_B1 | BOTH_FORCE_GETUP_B2
        | BOTH_FORCE_GETUP_B3 | BOTH_FORCE_GETUP_B4 | BOTH_FORCE_GETUP_B5
        | BOTH_FORCE_GETUP_B6 => {
            if BG_AnimLength(0, (*ps).legsAnim as animNumber_t) - (*ps).legsTimer < 500 {
                //at beginning of getup anim
                return QTRUE;
            }
        }
        BOTH_GETUP_BROLL_B | BOTH_GETUP_BROLL_F | BOTH_GETUP_BROLL_L | BOTH_GETUP_BROLL_R
        | BOTH_GETUP_FROLL_B | BOTH_GETUP_FROLL_F | BOTH_GETUP_FROLL_L | BOTH_GETUP_FROLL_R => {
            if BG_AnimLength(0, (*ps).legsAnim as animNumber_t) - (*ps).legsTimer < 500 {
                //at beginning of getup anim
                return QTRUE;
            }
        }
        BOTH_LK_DL_ST_T_SB_1_L => {
            if (*ps).legsTimer < 1000 {
                return QTRUE;
            }
        }
        BOTH_PLAYER_PA_3_FLY => {
            if (*ps).legsTimer < 300 {
                return QTRUE;
            }
        }
        _ => {}
    }
    QFALSE
}

pub fn BG_StabDownAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_STABDOWN | BOTH_STABDOWN_STAFF | BOTH_STABDOWN_DUAL => QTRUE,
        _ => QFALSE,
    }
}

pub fn PM_SaberBounceForAttack(r#move: c_int) -> c_int {
    let start_quad = unsafe { (*addr_of!(saberMoveData))[r#move as usize].startQuad };
    match start_quad {
        Q_B | Q_BR => LS_B1_BR,
        Q_R => LS_B1__R,
        Q_TR => LS_B1_TR,
        Q_T => LS_B1_T_,
        Q_TL => LS_B1_TL,
        Q_L => LS_B1__L,
        Q_BL => LS_B1_BL,
        _ => LS_NONE,
    }
}

pub fn PM_SaberDeflectionForQuad(quad: c_int) -> c_int {
    match quad {
        Q_B => LS_D1_B_,
        Q_BR => LS_D1_BR,
        Q_R => LS_D1__R,
        Q_TR => LS_D1_TR,
        Q_T => LS_D1_T_,
        Q_TL => LS_D1_TL,
        Q_L => LS_D1__L,
        Q_BL => LS_D1_BL,
        _ => LS_NONE,
    }
}

pub fn PM_SaberInDeflect(r#move: c_int) -> qboolean {
    if r#move >= LS_D1_BR && r#move <= LS_D1_B_ {
        return QTRUE;
    }
    QFALSE
}

pub fn PM_SaberInParry(r#move: c_int) -> qboolean {
    if r#move >= LS_PARRY_UP && r#move <= LS_PARRY_LL {
        return QTRUE;
    }
    QFALSE
}

pub fn PM_SaberInKnockaway(r#move: c_int) -> qboolean {
    if r#move >= LS_K1_T_ && r#move <= LS_K1_BL {
        return QTRUE;
    }
    QFALSE
}

pub fn PM_SaberInReflect(r#move: c_int) -> qboolean {
    if r#move >= LS_REFLECT_UP && r#move <= LS_REFLECT_LL {
        return QTRUE;
    }
    QFALSE
}

pub fn PM_SaberInStart(r#move: c_int) -> qboolean {
    if r#move >= LS_S_TL2BR && r#move <= LS_S_T2B {
        return QTRUE;
    }
    QFALSE
}

pub fn PM_SaberInReturn(r#move: c_int) -> qboolean {
    if r#move >= LS_R_TL2BR && r#move <= LS_R_T2B {
        return QTRUE;
    }
    QFALSE
}

pub fn BG_SaberInReturn(r#move: c_int) -> qboolean {
    PM_SaberInReturn(r#move)
}

pub fn PM_InSaberAnim(anim: c_int) -> qboolean {
    if anim >= BOTH_A1_T__B_ && anim <= BOTH_H1_S1_BR {
        return QTRUE;
    }
    QFALSE
}

pub unsafe fn PM_InKnockDown(ps: *mut playerState_t) -> qboolean {
    match (*ps).legsAnim {
        BOTH_KNOCKDOWN1 | BOTH_KNOCKDOWN2 | BOTH_KNOCKDOWN3 | BOTH_KNOCKDOWN4 | BOTH_KNOCKDOWN5 => {
            return QTRUE;
        }
        BOTH_GETUP1 | BOTH_GETUP2 | BOTH_GETUP3 | BOTH_GETUP4 | BOTH_GETUP5
        | BOTH_FORCE_GETUP_F1 | BOTH_FORCE_GETUP_F2 | BOTH_FORCE_GETUP_B1 | BOTH_FORCE_GETUP_B2
        | BOTH_FORCE_GETUP_B3 | BOTH_FORCE_GETUP_B4 | BOTH_FORCE_GETUP_B5 | BOTH_GETUP_BROLL_B
        | BOTH_GETUP_BROLL_F | BOTH_GETUP_BROLL_L | BOTH_GETUP_BROLL_R | BOTH_GETUP_FROLL_B
        | BOTH_GETUP_FROLL_F | BOTH_GETUP_FROLL_L | BOTH_GETUP_FROLL_R => {
            if (*ps).legsTimer != 0 {
                return QTRUE;
            }
        }
        _ => {}
    }
    QFALSE
}

pub fn PM_PainAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_PAIN1     //# First take pain anim
        | BOTH_PAIN2   //# Second take pain anim
        | BOTH_PAIN3   //# Third take pain anim
        | BOTH_PAIN4   //# Fourth take pain anim
        | BOTH_PAIN5   //# Fifth take pain anim - from behind
        | BOTH_PAIN6   //# Sixth take pain anim - from behind
        | BOTH_PAIN7   //# Seventh take pain anim - from behind
        | BOTH_PAIN8   //# Eigth take pain anim - from behind
        | BOTH_PAIN9   //#
        | BOTH_PAIN10  //#
        | BOTH_PAIN11  //#
        | BOTH_PAIN12  //#
        | BOTH_PAIN13  //#
        | BOTH_PAIN14  //#
        | BOTH_PAIN15  //#
        | BOTH_PAIN16  //#
        | BOTH_PAIN17  //#
        | BOTH_PAIN18 => QTRUE, //#
        _ => QFALSE,
    }
}

pub fn PM_JumpingAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_JUMP1        //# Jump - wind-up and leave ground
        | BOTH_INAIR1     //# In air loop (from jump)
        | BOTH_LAND1      //# Landing (from in air loop)
        | BOTH_LAND2      //# Landing Hard (from a great height)
        | BOTH_JUMPBACK1  //# Jump backwards - wind-up and leave ground
        | BOTH_INAIRBACK1 //# In air loop (from jump back)
        | BOTH_LANDBACK1  //# Landing backwards(from in air loop)
        | BOTH_JUMPLEFT1  //# Jump left - wind-up and leave ground
        | BOTH_INAIRLEFT1 //# In air loop (from jump left)
        | BOTH_LANDLEFT1  //# Landing left(from in air loop)
        | BOTH_JUMPRIGHT1 //# Jump right - wind-up and leave ground
        | BOTH_INAIRRIGHT1 //# In air loop (from jump right)
        | BOTH_LANDRIGHT1 //# Landing right(from in air loop)
        | BOTH_FORCEJUMP1 //# Jump - wind-up and leave ground
        | BOTH_FORCEINAIR1 //# In air loop (from jump)
        | BOTH_FORCELAND1 //# Landing (from in air loop)
        | BOTH_FORCEJUMPBACK1 //# Jump backwards - wind-up and leave ground
        | BOTH_FORCEINAIRBACK1 //# In air loop (from jump back)
        | BOTH_FORCELANDBACK1 //# Landing backwards(from in air loop)
        | BOTH_FORCEJUMPLEFT1 //# Jump left - wind-up and leave ground
        | BOTH_FORCEINAIRLEFT1 //# In air loop (from jump left)
        | BOTH_FORCELANDLEFT1 //# Landing left(from in air loop)
        | BOTH_FORCEJUMPRIGHT1 //# Jump right - wind-up and leave ground
        | BOTH_FORCEINAIRRIGHT1 //# In air loop (from jump right)
        | BOTH_FORCELANDRIGHT1 => QTRUE, //# Landing right(from in air loop)
        _ => QFALSE,
    }
}

pub fn PM_LandingAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_LAND1       //# Landing (from in air loop)
        | BOTH_LAND2     //# Landing Hard (from a great height)
        | BOTH_LANDBACK1 //# Landing backwards(from in air loop)
        | BOTH_LANDLEFT1 //# Landing left(from in air loop)
        | BOTH_LANDRIGHT1 //# Landing right(from in air loop)
        | BOTH_FORCELAND1 //# Landing (from in air loop)
        | BOTH_FORCELANDBACK1 //# Landing backwards(from in air loop)
        | BOTH_FORCELANDLEFT1 //# Landing left(from in air loop)
        | BOTH_FORCELANDRIGHT1 => QTRUE, //# Landing right(from in air loop)
        _ => QFALSE,
    }
}

pub fn PM_SpinningAnim(anim: c_int) -> qboolean {
    /*
    switch ( anim )
    {
    //FIXME: list any other spinning anims
    default:
        break;
    }
    */
    BG_SpinningSaberAnim(anim)
}

pub fn PM_InOnGroundAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_DEAD1
        | BOTH_DEAD2
        | BOTH_DEAD3
        | BOTH_DEAD4
        | BOTH_DEAD5
        | BOTH_DEADFORWARD1
        | BOTH_DEADBACKWARD1
        | BOTH_DEADFORWARD2
        | BOTH_DEADBACKWARD2
        | BOTH_LYINGDEATH1
        | BOTH_LYINGDEAD1
        | BOTH_SLEEP1        //# laying on back-rknee up-rhand on torso
        | BOTH_KNOCKDOWN1    //#
        | BOTH_KNOCKDOWN2    //#
        | BOTH_KNOCKDOWN3    //#
        | BOTH_KNOCKDOWN4    //#
        | BOTH_KNOCKDOWN5    //#
        | BOTH_GETUP1
        | BOTH_GETUP2
        | BOTH_GETUP3
        | BOTH_GETUP4
        | BOTH_GETUP5
        | BOTH_GETUP_CROUCH_F1
        | BOTH_GETUP_CROUCH_B1
        | BOTH_FORCE_GETUP_F1
        | BOTH_FORCE_GETUP_F2
        | BOTH_FORCE_GETUP_B1
        | BOTH_FORCE_GETUP_B2
        | BOTH_FORCE_GETUP_B3
        | BOTH_FORCE_GETUP_B4
        | BOTH_FORCE_GETUP_B5
        | BOTH_FORCE_GETUP_B6
        | BOTH_GETUP_BROLL_B
        | BOTH_GETUP_BROLL_F
        | BOTH_GETUP_BROLL_L
        | BOTH_GETUP_BROLL_R
        | BOTH_GETUP_FROLL_B
        | BOTH_GETUP_FROLL_F
        | BOTH_GETUP_FROLL_L
        | BOTH_GETUP_FROLL_R => QTRUE,
        _ => QFALSE,
    }
}

pub unsafe fn PM_InRollComplete(ps: *mut playerState_t, anim: c_int) -> qboolean {
    match anim {
        BOTH_ROLL_F | BOTH_ROLL_B | BOTH_ROLL_R | BOTH_ROLL_L => {
            if (*ps).legsTimer < 1 {
                return QTRUE;
            }
        }
        _ => {}
    }
    QFALSE
}

pub unsafe fn PM_CanRollFromSoulCal(ps: *mut playerState_t) -> qboolean {
    if (*ps).legsAnim == BOTH_A7_SOULCAL && (*ps).legsTimer < 700 && (*ps).legsTimer > 250 {
        return QTRUE;
    }
    QFALSE
}

pub fn BG_SuperBreakLoseAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_LK_S_DL_S_SB_1_L    //super break I lost
        | BOTH_LK_S_DL_T_SB_1_L  //super break I lost
        | BOTH_LK_S_ST_S_SB_1_L  //super break I lost
        | BOTH_LK_S_ST_T_SB_1_L  //super break I lost
        | BOTH_LK_S_S_S_SB_1_L   //super break I lost
        | BOTH_LK_S_S_T_SB_1_L   //super break I lost
        | BOTH_LK_DL_DL_S_SB_1_L //super break I lost
        | BOTH_LK_DL_DL_T_SB_1_L //super break I lost
        | BOTH_LK_DL_ST_S_SB_1_L //super break I lost
        | BOTH_LK_DL_ST_T_SB_1_L //super break I lost
        | BOTH_LK_DL_S_S_SB_1_L  //super break I lost
        | BOTH_LK_DL_S_T_SB_1_L  //super break I lost
        | BOTH_LK_ST_DL_S_SB_1_L //super break I lost
        | BOTH_LK_ST_DL_T_SB_1_L //super break I lost
        | BOTH_LK_ST_ST_S_SB_1_L //super break I lost
        | BOTH_LK_ST_ST_T_SB_1_L //super break I lost
        | BOTH_LK_ST_S_S_SB_1_L  //super break I lost
        | BOTH_LK_ST_S_T_SB_1_L => QTRUE, //super break I lost
        _ => QFALSE,
    }
}

pub fn BG_SuperBreakWinAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_LK_S_DL_S_SB_1_W    //super break I won
        | BOTH_LK_S_DL_T_SB_1_W  //super break I won
        | BOTH_LK_S_ST_S_SB_1_W  //super break I won
        | BOTH_LK_S_ST_T_SB_1_W  //super break I won
        | BOTH_LK_S_S_S_SB_1_W   //super break I won
        | BOTH_LK_S_S_T_SB_1_W   //super break I won
        | BOTH_LK_DL_DL_S_SB_1_W //super break I won
        | BOTH_LK_DL_DL_T_SB_1_W //super break I won
        | BOTH_LK_DL_ST_S_SB_1_W //super break I won
        | BOTH_LK_DL_ST_T_SB_1_W //super break I won
        | BOTH_LK_DL_S_S_SB_1_W  //super break I won
        | BOTH_LK_DL_S_T_SB_1_W  //super break I won
        | BOTH_LK_ST_DL_S_SB_1_W //super break I won
        | BOTH_LK_ST_DL_T_SB_1_W //super break I won
        | BOTH_LK_ST_ST_S_SB_1_W //super break I won
        | BOTH_LK_ST_ST_T_SB_1_W //super break I won
        | BOTH_LK_ST_S_S_SB_1_W  //super break I won
        | BOTH_LK_ST_S_T_SB_1_W => QTRUE, //super break I won
        _ => QFALSE,
    }
}

pub fn BG_SaberLockBreakAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_BF1BREAK
        | BOTH_BF2BREAK
        | BOTH_CWCIRCLEBREAK
        | BOTH_CCWCIRCLEBREAK
        | BOTH_LK_S_DL_S_B_1_L   //normal break I lost
        | BOTH_LK_S_DL_S_B_1_W   //normal break I won
        | BOTH_LK_S_DL_T_B_1_L   //normal break I lost
        | BOTH_LK_S_DL_T_B_1_W   //normal break I won
        | BOTH_LK_S_ST_S_B_1_L   //normal break I lost
        | BOTH_LK_S_ST_S_B_1_W   //normal break I won
        | BOTH_LK_S_ST_T_B_1_L   //normal break I lost
        | BOTH_LK_S_ST_T_B_1_W   //normal break I won
        | BOTH_LK_S_S_S_B_1_L    //normal break I lost
        | BOTH_LK_S_S_S_B_1_W    //normal break I won
        | BOTH_LK_S_S_T_B_1_L    //normal break I lost
        | BOTH_LK_S_S_T_B_1_W    //normal break I won
        | BOTH_LK_DL_DL_S_B_1_L  //normal break I lost
        | BOTH_LK_DL_DL_S_B_1_W  //normal break I won
        | BOTH_LK_DL_DL_T_B_1_L  //normal break I lost
        | BOTH_LK_DL_DL_T_B_1_W  //normal break I won
        | BOTH_LK_DL_ST_S_B_1_L  //normal break I lost
        | BOTH_LK_DL_ST_S_B_1_W  //normal break I won
        | BOTH_LK_DL_ST_T_B_1_L  //normal break I lost
        | BOTH_LK_DL_ST_T_B_1_W  //normal break I won
        | BOTH_LK_DL_S_S_B_1_L   //normal break I lost
        | BOTH_LK_DL_S_S_B_1_W   //normal break I won
        | BOTH_LK_DL_S_T_B_1_L   //normal break I lost
        | BOTH_LK_DL_S_T_B_1_W   //normal break I won
        | BOTH_LK_ST_DL_S_B_1_L  //normal break I lost
        | BOTH_LK_ST_DL_S_B_1_W  //normal break I won
        | BOTH_LK_ST_DL_T_B_1_L  //normal break I lost
        | BOTH_LK_ST_DL_T_B_1_W  //normal break I won
        | BOTH_LK_ST_ST_S_B_1_L  //normal break I lost
        | BOTH_LK_ST_ST_S_B_1_W  //normal break I won
        | BOTH_LK_ST_ST_T_B_1_L  //normal break I lost
        | BOTH_LK_ST_ST_T_B_1_W  //normal break I won
        | BOTH_LK_ST_S_S_B_1_L   //normal break I lost
        | BOTH_LK_ST_S_S_B_1_W   //normal break I won
        | BOTH_LK_ST_S_T_B_1_L   //normal break I lost
        | BOTH_LK_ST_S_T_B_1_W => QTRUE, //normal break I won
        _ => (BG_SuperBreakLoseAnim(anim) != QFALSE || BG_SuperBreakWinAnim(anim) != QFALSE) as qboolean,
    }
}

pub fn BG_FullBodyTauntAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_GESTURE1
        | BOTH_DUAL_TAUNT
        | BOTH_STAFF_TAUNT
        | BOTH_BOW
        | BOTH_MEDITATE
        | BOTH_SHOWOFF_FAST
        | BOTH_SHOWOFF_MEDIUM
        | BOTH_SHOWOFF_STRONG
        | BOTH_SHOWOFF_DUAL
        | BOTH_SHOWOFF_STAFF
        | BOTH_VICTORY_FAST
        | BOTH_VICTORY_MEDIUM
        | BOTH_VICTORY_STRONG
        | BOTH_VICTORY_DUAL
        | BOTH_VICTORY_STAFF => QTRUE,
        _ => QFALSE,
    }
}

/*
=============
BG_AnimLength

Get the "length" of an anim given the local anim index (which skeleton)
and anim number. Obviously does not take things like the length of the
anim while force speeding (as an example) and whatnot into account.
=============
*/
pub unsafe fn BG_AnimLength(index: c_int, anim: animNumber_t) -> c_int {
    if anim >= MAX_ANIMATIONS {
        return -1;
    }
    let a = (*addr_of!(bgAllAnims))[index as usize]
        .anims
        .add(anim as usize);
    // C: numFrames (unsigned short) * fabs((float)frameLerp) returned as int.
    // Replicate the short->float->double promotion at the fabs and the
    // truncate-toward-zero on the int return.
    ((*a).numFrames as f64 * ((*a).frameLerp as f32 as f64).abs()) as c_int
}

//just use whatever pm->animations is
pub unsafe fn PM_AnimLength(_index: c_int, anim: animNumber_t) -> c_int {
    let pmv = *addr_of!(pm);
    if anim >= MAX_ANIMATIONS || (*pmv).animations.is_null() {
        return -1;
    }
    if anim < 0 {
        Com_Error(ERR_DROP, &format!("ERROR: anim {} < 0\n", anim));
    }
    let a = (*pmv).animations.add(anim as usize);
    ((*a).numFrames as f64 * ((*a).frameLerp as f32 as f64).abs()) as c_int
}

pub unsafe fn PM_DebugLegsAnim(anim: c_int) {
    let pmv = *addr_of!(pm);
    let oldAnim = (*(*pmv).ps).legsAnim;
    let newAnim = anim;

    if oldAnim < MAX_TOTALANIMATIONS
        && oldAnim >= BOTH_DEATH1
        && newAnim < MAX_TOTALANIMATIONS
        && newAnim >= BOTH_DEATH1
    {
        // C passes the `stringID_table_t` struct to `%s`; its first member is the
        // `name` char*, so this prints the animation's stringized name.
        let table = &*addr_of!(animTable);
        Com_Printf(&format!(
            "OLD: {}\n",
            CStr::from_ptr(table[oldAnim as usize].name).to_string_lossy()
        ));
        Com_Printf(&format!(
            "NEW: {}\n",
            CStr::from_ptr(table[newAnim as usize].name).to_string_lossy()
        ));
    }
}

pub fn PM_SaberInTransition(r#move: c_int) -> qboolean {
    if r#move >= LS_T1_BR__R && r#move <= LS_T1_BL__L {
        return QTRUE;
    }
    QFALSE
}

pub fn BG_SaberInTransitionAny(r#move: c_int) -> qboolean {
    if PM_SaberInStart(r#move) != QFALSE {
        return QTRUE;
    } else if PM_SaberInTransition(r#move) != QFALSE {
        return QTRUE;
    } else if PM_SaberInReturn(r#move) != QFALSE {
        return QTRUE;
    }
    QFALSE
}

/*
==============================================================================
END: Animation utility functions (sequence checking)
==============================================================================
*/

// ===========================================================================
// Animation-set globals + state-changing setters (bg_panimate.c 1633-2965).
//
// Everything above is stateless; the functions below MUTATE animation state on
// a `playerState_t` (or via the `pm` keystone) and read the animation-set
// globals declared here. The animation-file parser `BG_ParseAnimationFile`
// (bg_panimate.c:2282) closes out the file at the bottom of this section; its
// `#ifndef QAGAME` / `#ifdef CONVENIENT_..._DEBUG_THING` neighbours are not in the
// server build (see the module header).
// ===========================================================================

pub unsafe fn BG_FlipPart(ps: *mut playerState_t, part: c_int) {
    if part == SETANIM_TORSO {
        if (*ps).torsoFlip != QFALSE {
            (*ps).torsoFlip = QFALSE;
        } else {
            (*ps).torsoFlip = QTRUE;
        }
    } else if part == SETANIM_LEGS {
        if (*ps).legsFlip != QFALSE {
            (*ps).legsFlip = QFALSE;
        } else {
            (*ps).legsFlip = QTRUE;
        }
    }
}

// `#ifdef Q3_VM char BGPAFtext[60000];` -- parser scratch buffer, only allocated
// in the bytecode-VM build (== the `vm` cargo feature). Used by `BG_ParseAnimationFile`
// in that build; the native build uses a stack-local buffer instead.
#[cfg(feature = "vm")]
pub static mut BGPAFtext: [core::ffi::c_char; 60000] = [0; 60000];

pub static mut BGPAFtextLoaded: qboolean = QFALSE;

// humanoid animations are the only ones that are statically allocated.
pub static mut bgHumanoidAnimations: [animation_t; MAX_TOTALANIMATIONS as usize] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

pub static mut bgAllAnims: [bgLoadedAnim_t; MAX_ANIM_FILES] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
// start off at 2, because 0 will always be assigned to humanoid, and 1 will
// always be rockettrooper
pub static mut bgNumAllAnims: c_int = 2;

pub static mut bgAllEvents: [bgLoadedEvents_t; MAX_ANIM_FILES] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
pub static mut bgNumAnimEvents: c_int = 1;
// PC relocated this file-scope counter next to BG_ParseAnimationEvtFile (still
// unported); BG_InitAnimsets no longer resets it. Its only live uses are in the
// not-yet-ported event-parser, so it reads as dead until that lands.
#[allow(dead_code)]
static mut bg_animParseIncluding: c_int = 0;

// TODO: Port-Bug
//ALWAYS call on game/cgame init
pub unsafe fn BG_InitAnimsets() {
    core::ptr::write_bytes(addr_of_mut!(bgAllAnims), 0, 1);
    BGPAFtextLoaded = QFALSE; // VVFIXME - The PC doesn't seem to need this, but why?
}

//ALWAYS call on game/cgame shutdown
pub unsafe fn BG_ClearAnimsets() {
    // The body is entirely commented out in the C (a `strap_TrueFree` loop over
    // bgAllAnims[i].anims); kept as a no-op to preserve the call site.
}

pub unsafe fn BG_AnimsetAlloc() -> *mut animation_t {
    debug_assert!((*addr_of!(bgNumAllAnims) as usize) < MAX_ANIM_FILES);
    let idx = *addr_of!(bgNumAllAnims) as usize;
    (*addr_of_mut!(bgAllAnims))[idx].anims =
        BG_Alloc((size_of::<animation_t>() * MAX_TOTALANIMATIONS as usize) as c_int)
            as *mut animation_t;

    (*addr_of!(bgAllAnims))[idx].anims
}

pub unsafe fn BG_AnimsetFree(_animset: *mut animation_t) {
    // The body is entirely commented out in the C (a `strap_TrueFree` + `_DEBUG`
    // assert); kept as a no-op to preserve the call site.
}

// `SpewDebugStuffToFile` (bg_panimate.c:1668) and `BG_ParseAnimationEvtFile` /
// `ParseAnimationEvtBlock` / `CheckAnimFrameForEventType` + the
// `animEventTypeTable`/`footstepTypeTable` (bg_panimate.c:1755-2271) are absent from
// this build: the former is `#ifdef CONVENIENT_ANIMATION_FILE_DEBUG_THING` (a
// never-defined debug macro), the latter are `#ifndef QAGAME` ("none of this is
// actually needed serverside") and this crate compiles as the QAGAME server module.
// Only `BG_ParseAnimationFile` (unguarded) belongs in the server build; it follows.
//
// ============================================================================
// EXCLUDED FROM THE QAGAME SERVER BUILD — commented-out 1:1 translations
// ----------------------------------------------------------------------------
// The four functions below are NEVER compiled into this crate (the JKA QAGAME
// server module). They are reproduced here, fully commented out, purely for
// 1:1-file-completeness / self-documentation against
// `refs/raven-jediacademy/codemp/game/bg_panimate.c`. They MUST stay commented:
// adding them live would pull in cgame-only traps (`trap_S_RegisterSound`,
// `trap_FX_RegisterEffect`) and the `animevent_t` event-table surface that the
// server build deliberately omits.
//
//   * `SpewDebugStuffToFile`         (bg_panimate.c:1668) — guarded
//     `#ifdef CONVENIENT_ANIMATION_FILE_DEBUG_THING` (a debug macro that is
//     never `#define`d; the `#define` on line 1665 is itself commented out).
//   * `CheckAnimFrameForEventType`   (bg_panimate.c:1779)
//   * `ParseAnimationEvtBlock`       (bg_panimate.c:1797)
//   * `BG_ParseAnimationEvtFile`     (bg_panimate.c:2117)
//     — all three live inside the file-spanning `#ifndef QAGAME` block
//     (bg_panimate.c:1755 "none of this is actually needed serverside …" →
//     `#endif` at bg_panimate.c:2271), alongside the `animEventTypeTable` /
//     `footstepTypeTable` string tables (transcribed as comments below).
// ============================================================================
//
// ---- #ifdef CONVENIENT_ANIMATION_FILE_DEBUG_THING (never defined) ----------
// SpewDebugStuffToFile (bg_panimate.c:1668)
//   void SpewDebugStuffToFile();
//
// pub unsafe fn SpewDebugStuffToFile() {
//     let mut f: fileHandle_t = 0;
//     let mut i: c_int = 0;
//
//     trap::FS_FOpenFile(c"file_of_debug_stuff_MP.txt".as_ptr(), addr_of_mut!(f), FS_WRITE);
//
//     if f == 0 {
//         return;
//     }
//
//     BGPAFtext[0] = 0;
//
//     while i < MAX_ANIMATIONS {
//         strcat(
//             addr_of_mut!(BGPAFtext) as *mut c_char,
//             va(c"%i %i\n".as_ptr(), i, bgHumanoidAnimations[i as usize].frameLerp),
//         );
//         i += 1;
//     }
//
//     trap::FS_Write(
//         addr_of!(BGPAFtext) as *const c_void,
//         strlen(addr_of!(BGPAFtext) as *const c_char) as c_int,
//         f,
//     );
//     trap::FS_FCloseFile(f);
// }
//
// ---- #ifndef QAGAME (cgame-only; bg_panimate.c:1755-2271) -------------------
// String tables consumed by ParseAnimationEvtBlock (bg_panimate.c:1757, 1769):
//
// static mut animEventTypeTable: [stringID_table_t; AEV_NUM_AEV as usize + 1] = [
//     ENUM2STRING(AEV_SOUND),     //# animID AEV_SOUND framenum soundpath randomlow randomhi chancetoplay
//     ENUM2STRING(AEV_FOOTSTEP),  //# animID AEV_FOOTSTEP framenum footstepType
//     ENUM2STRING(AEV_EFFECT),    //# animID AEV_EFFECT framenum effectpath boltName
//     ENUM2STRING(AEV_FIRE),      //# animID AEV_FIRE framenum altfire chancetofire
//     ENUM2STRING(AEV_MOVE),      //# animID AEV_MOVE framenum forwardpush rightpush uppush
//     ENUM2STRING(AEV_SOUNDCHAN), //# animID AEV_SOUNDCHAN framenum CHANNEL soundpath randomlow randomhi chancetoplay
//     // must be terminated
//     stringID_table_t { name: ptr::null(), id: -1 },
// ];
//
// static mut footstepTypeTable: [stringID_table_t; NUM_FOOTSTEP_TYPES as usize + 1] = [
//     ENUM2STRING(FOOTSTEP_R),
//     ENUM2STRING(FOOTSTEP_L),
//     ENUM2STRING(FOOTSTEP_HEAVY_R),
//     ENUM2STRING(FOOTSTEP_HEAVY_L),
//     // must be terminated
//     stringID_table_t { name: ptr::null(), id: -1 },
// ];
//
// CheckAnimFrameForEventType (bg_panimate.c:1779)
//   int CheckAnimFrameForEventType( animevent_t *animEvents, int keyFrame, animEventType_t eventType );
//
// pub unsafe fn CheckAnimFrameForEventType(
//     animEvents: *mut animevent_t,
//     keyFrame: c_int,
//     eventType: animEventType_t,
// ) -> c_int {
//     for i in 0..MAX_ANIM_EVENTS {
//         if (*animEvents.add(i as usize)).keyFrame == keyFrame {
//             // there is an animevent on this frame already
//             if (*animEvents.add(i as usize)).eventType == eventType {
//                 // and it is of the same type
//                 return i;
//             }
//         }
//     }
//     // nope
//     -1
// }
//
// ParseAnimationEvtBlock (bg_panimate.c:1797)
//   void ParseAnimationEvtBlock( const char *aeb_filename, animevent_t *animEvents,
//                                animation_t *animations, int *i, const char **text_p );
//   (NB: the `i` param is unused in the C body — only `lastAnimEvent` advances the index.)
//
// pub unsafe fn ParseAnimationEvtBlock(
//     aeb_filename: *const c_char,
//     animEvents: *mut animevent_t,
//     animations: *mut animation_t,
//     _i: *mut c_int,
//     text_p: *mut *const c_char,
// ) {
//     let mut token: *const c_char;
//     let (mut num, mut n, mut animNum, mut keyFrame, mut lowestVal, mut highestVal);
//     let mut curAnimEvent: c_int;
//     let mut lastAnimEvent: c_int = 0;
//     let mut eventType: animEventType_t;
//     let mut stringData: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
//
//     // get past starting bracket
//     loop {
//         token = COM_Parse(text_p);
//         if Q_stricmp(token, c"{".as_ptr()) == 0 {
//             break;
//         }
//     }
//
//     // NOTE: instead of a blind increment, increase the index
//     //          this way if we have an event on an anim that already
//     //          has an event of that type, it stomps it
//
//     // read information for each frame
//     loop {
//         if lastAnimEvent >= MAX_ANIM_EVENTS {
//             Com_Error(
//                 ERR_DROP,
//                 c"ParseAnimationEvtBlock: number events in animEvent file %s > MAX_ANIM_EVENTS(%i)".as_ptr(),
//                 aeb_filename,
//                 MAX_ANIM_EVENTS,
//             );
//             return;
//         }
//         // Get base frame of sequence
//         token = COM_Parse(text_p);
//         if token.is_null() || *token == 0 {
//             break;
//         }
//
//         if Q_stricmp(token, c"}".as_ptr()) == 0 {
//             // At end of block
//             break;
//         }
//
//         // Compare to same table as animations used so we don't have to use actual
//         // numbers for animation first frames, just need offsets.
//         animNum = GetIDForString(addr_of!(animTable) as *mut stringID_table_t, token);
//         if animNum == -1 {
//             // Unrecognized ANIM ENUM name; skip to end of line and keep going
//             // #ifndef FINAL_BUILD
//             Com_Printf(
//                 c"%cWARNING: Unknown token %s in animEvent file %s\n".as_ptr(),
//                 /* S_COLOR_YELLOW */ token, aeb_filename,
//             );
//             // #endif
//             while *token != 0 {
//                 token = COM_ParseExt(text_p, QFALSE); // returns "" at EOL
//             }
//             continue;
//         }
//
//         if (*animations.add(animNum as usize)).numFrames == 0 {
//             // we don't use this anim
//             // #ifndef FINAL_BUILD
//             Com_Printf(
//                 c"%c%s animevents.cfg: anim %s not used by this model\n".as_ptr(),
//                 /* S_COLOR_YELLOW */ aeb_filename, token,
//             );
//             // #endif
//             SkipRestOfLine(text_p);
//             continue;
//         }
//
//         token = COM_Parse(text_p);
//         eventType = GetIDForString(addr_of!(animEventTypeTable) as *mut stringID_table_t, token) as animEventType_t;
//         if eventType == AEV_NONE || eventType == -1 {
//             // Unrecognized ANIM EVENT TYPE; skip this line
//             continue;
//         }
//
//         // set our start frame
//         keyFrame = (*animations.add(animNum as usize)).firstFrame;
//         // Get offset to frame within sequence
//         token = COM_Parse(text_p);
//         if token.is_null() {
//             break;
//         }
//         keyFrame += atoi(token);
//
//         // see if this frame already has an event of this type on it; if so, overwrite
//         curAnimEvent = CheckAnimFrameForEventType(animEvents, keyFrame, eventType);
//         if curAnimEvent == -1 {
//             // this anim frame doesn't already have an event of this type on it
//             curAnimEvent = lastAnimEvent;
//         }
//
//         // now plug the data into the chosen event index
//         (*animEvents.add(curAnimEvent as usize)).eventType = eventType;
//         (*animEvents.add(curAnimEvent as usize)).keyFrame = keyFrame;
//
//         // now read out the proper data based on the type
//         match (*animEvents.add(curAnimEvent as usize)).eventType {
//             AEV_SOUNDCHAN => {
//                 //# animID AEV_SOUNDCHAN framenum CHANNEL soundpath randomlow randomhi chancetoplay
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break;
//                 }
//                 if stricmp(token, c"CHAN_VOICE_ATTEN".as_ptr()) == 0 {
//                     (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDCHANNEL as usize] = CHAN_VOICE_ATTEN;
//                 } else if stricmp(token, c"CHAN_VOICE_GLOBAL".as_ptr()) == 0 {
//                     (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDCHANNEL as usize] = CHAN_VOICE_GLOBAL;
//                 } else if stricmp(token, c"CHAN_ANNOUNCER".as_ptr()) == 0 {
//                     (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDCHANNEL as usize] = CHAN_ANNOUNCER;
//                 } else if stricmp(token, c"CHAN_BODY".as_ptr()) == 0 {
//                     (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDCHANNEL as usize] = CHAN_BODY;
//                 } else if stricmp(token, c"CHAN_WEAPON".as_ptr()) == 0 {
//                     (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDCHANNEL as usize] = CHAN_WEAPON;
//                 } else if stricmp(token, c"CHAN_VOICE".as_ptr()) == 0 {
//                     (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDCHANNEL as usize] = CHAN_VOICE;
//                 } else {
//                     (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDCHANNEL as usize] = CHAN_AUTO;
//                 }
//                 // fall through to normal sound  --- C `case` fallthrough into AEV_SOUND ---
//                 // (Rust has no fallthrough; the live port would factor the AEV_SOUND body
//                 //  into a shared closure invoked by both arms.)
//                 // [AEV_SOUND body follows]
//                 // ... see AEV_SOUND ...
//             }
//             AEV_SOUND => {
//                 //# animID AEV_SOUND framenum soundpath randomlow randomhi chancetoplay
//                 // get soundstring
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break;
//                 }
//                 strcpy(stringData.as_mut_ptr(), token);
//                 // get lowest value
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 lowestVal = atoi(token);
//                 // get highest value
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 highestVal = atoi(token);
//                 // Now precache all the sounds
//                 if lowestVal != 0 && highestVal != 0 {
//                     if (highestVal - lowestVal) >= MAX_RANDOM_ANIM_SOUNDS {
//                         highestVal = lowestVal + (MAX_RANDOM_ANIM_SOUNDS - 1);
//                     }
//                     n = lowestVal;
//                     num = AED_SOUNDINDEX_START;
//                     while n <= highestVal && num <= AED_SOUNDINDEX_END {
//                         if stringData[0] == b'*' as c_char {
//                             (*animEvents.add(curAnimEvent as usize)).eventData[num as usize] = 0;
//                         } else {
//                             (*animEvents.add(curAnimEvent as usize)).eventData[num as usize] =
//                                 trap_S_RegisterSound(va(stringData.as_ptr(), n)) as c_int;
//                         }
//                         n += 1;
//                         num += 1;
//                     }
//                     (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUND_NUMRANDOMSNDS as usize] = num - 1;
//                 } else {
//                     if stringData[0] == b'*' as c_char {
//                         (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDINDEX_START as usize] = 0;
//                     } else {
//                         (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDINDEX_START as usize] =
//                             trap_S_RegisterSound(stringData.as_ptr()) as c_int;
//                     }
//                     // #ifndef FINAL_BUILD
//                     if (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUNDINDEX_START as usize] == 0
//                         && stringData[0] != b'*' as c_char
//                     {
//                         // couldn't register it - file not found
//                         Com_Printf(
//                             c"%cParseAnimationSndBlock: sound %s does not exist (animevents.cfg %s)!\n".as_ptr(),
//                             /* S_COLOR_RED */ stringData.as_ptr(), aeb_filename,
//                         );
//                     }
//                     // #endif
//                     (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUND_NUMRANDOMSNDS as usize] = 0;
//                 }
//                 // get probability
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_SOUND_PROBABILITY as usize] = atoi(token);
//             }
//             AEV_FOOTSTEP => {
//                 //# animID AEV_FOOTSTEP framenum footstepType
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break;
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_FOOTSTEP_TYPE as usize] =
//                     GetIDForString(addr_of!(footstepTypeTable) as *mut stringID_table_t, token);
//                 // get probability
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_FOOTSTEP_PROBABILITY as usize] = atoi(token);
//             }
//             AEV_EFFECT => {
//                 //# animID AEV_EFFECT framenum effectpath boltName
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break;
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_EFFECTINDEX as usize] =
//                     trap_FX_RegisterEffect(token);
//                 // get bolt index
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break;
//                 }
//                 if Q_stricmp(c"none".as_ptr(), token) != 0 && Q_stricmp(c"NULL".as_ptr(), token) != 0 {
//                     // actually are specifying a bolt to use
//                     if (*animEvents.add(curAnimEvent as usize)).stringData.is_null() {
//                         (*animEvents.add(curAnimEvent as usize)).stringData = BG_Alloc(2048) as *mut c_char;
//                     }
//                     strcpy((*animEvents.add(curAnimEvent as usize)).stringData, token);
//                 }
//                 // NOTE: this string is later used to add a bolt and store the index.
//                 // get probability
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_EFFECT_PROBABILITY as usize] = atoi(token);
//             }
//             AEV_FIRE => {
//                 //# animID AEV_FIRE framenum altfire chancetofire
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_FIRE_ALT as usize] = atoi(token);
//                 // get probability
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_FIRE_PROBABILITY as usize] = atoi(token);
//             }
//             AEV_MOVE => {
//                 //# animID AEV_MOVE framenum forwardpush rightpush uppush
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_MOVE_FWD as usize] = atoi(token);
//                 // get right push
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_MOVE_RT as usize] = atoi(token);
//                 // get upwards push
//                 token = COM_Parse(text_p);
//                 if token.is_null() {
//                     break; // WARNING! BAD TABLE!
//                 }
//                 (*animEvents.add(curAnimEvent as usize)).eventData[AED_MOVE_UP as usize] = atoi(token);
//             }
//             _ => {
//                 // unknown?
//                 SkipRestOfLine(text_p);
//                 continue;
//             }
//         }
//
//         if curAnimEvent == lastAnimEvent {
//             lastAnimEvent += 1;
//         }
//     }
// }
//
// BG_ParseAnimationEvtFile (bg_panimate.c:2117)
//   int BG_ParseAnimationEvtFile( const char *as_filename, int animFileIndex, int eventFileIndex );
//   Read models/players/<model>/animevents.cfg (presence not required).
//
// pub unsafe fn BG_ParseAnimationEvtFile(
//     as_filename: *const c_char,
//     animFileIndex: c_int,
//     eventFileIndex: c_int,
// ) -> c_int {
//     let mut text_p: *const c_char;
//     let mut len: c_int;
//     let mut token: *const c_char;
//     let mut text: [c_char; 80000] = [0; 80000];
//     let mut sfilename: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
//     let mut f: fileHandle_t = 0;
//     let (mut i, mut j, mut upper_i, mut lower_i);
//     let mut usedIndex: c_int = -1;
//     let legsAnimEvents: *mut animevent_t;
//     let torsoAnimEvents: *mut animevent_t;
//     let animations: *mut animation_t;
//     let forcedIndex: c_int;
//
//     debug_assert!(animFileIndex < MAX_ANIM_FILES as c_int);
//     debug_assert!(eventFileIndex < MAX_ANIM_FILES as c_int);
//
//     if eventFileIndex == -1 {
//         forcedIndex = 0;
//     } else {
//         forcedIndex = eventFileIndex;
//     }
//
//     if bg_animParseIncluding <= 0 {
//         // if we should be parsing an included file, skip this part
//         if bgAllEvents[forcedIndex as usize].eventsParsed != QFALSE {
//             // already cached this one
//             return forcedIndex;
//         }
//     }
//
//     legsAnimEvents = bgAllEvents[forcedIndex as usize].legsAnimEvents.as_mut_ptr();
//     torsoAnimEvents = bgAllEvents[forcedIndex as usize].torsoAnimEvents.as_mut_ptr();
//     animations = bgAllAnims[animFileIndex as usize].anims;
//
//     if bg_animParseIncluding <= 0 {
//         // Go through and see if this filename is already in the table.
//         i = 0;
//         while i < bgNumAnimEvents && forcedIndex != 0 {
//             if Q_stricmp(as_filename, bgAllEvents[i as usize].filename.as_ptr()) == 0 {
//                 // looks like we have it already.
//                 return i;
//             }
//             i += 1;
//         }
//     }
//
//     // Load and parse animevents.cfg file
//     Com_sprintf(
//         sfilename.as_mut_ptr(),
//         size_of_val(&sfilename) as c_int,
//         c"%sanimevents.cfg".as_ptr(),
//         as_filename,
//     );
//
//     if bg_animParseIncluding <= 0 {
//         // should already be done if we're including; initialize anim event array
//         for ii in 0..MAX_ANIM_EVENTS {
//             // Type of event
//             (*torsoAnimEvents.add(ii as usize)).eventType = AEV_NONE;
//             (*legsAnimEvents.add(ii as usize)).eventType = AEV_NONE;
//             // Frame to play event on
//             (*torsoAnimEvents.add(ii as usize)).keyFrame = -1;
//             (*legsAnimEvents.add(ii as usize)).keyFrame = -1;
//             // one temporary string storage slot
//             (*torsoAnimEvents.add(ii as usize)).stringData = ptr::null_mut();
//             (*legsAnimEvents.add(ii as usize)).stringData = ptr::null_mut();
//             // Unique IDs (soundIndex / effect index / footstep type, etc.)
//             for jj in 0..AED_ARRAY_SIZE {
//                 (*torsoAnimEvents.add(ii as usize)).eventData[jj as usize] = -1;
//                 (*legsAnimEvents.add(ii as usize)).eventData[jj as usize] = -1;
//             }
//             let _ = j; // (C uses `j` for the inner loop)
//         }
//     }
//
//     // load the file
//     len = trap::FS_FOpenFile(sfilename.as_ptr(), addr_of_mut!(f), FS_READ);
//     if len <= 0 {
//         // no file
//         // goto fin;
//     } else if len as usize >= size_of_val(&text) - 1 {
//         trap::FS_FCloseFile(f);
//         // #ifndef FINAL_BUILD
//         Com_Error(ERR_DROP, c"File %s too long\n".as_ptr(), sfilename.as_ptr());
//         // #else  Com_Printf( "File %s too long\n", sfilename ); #endif
//         // goto fin;
//     } else {
//         trap::FS_Read(text.as_mut_ptr() as *mut c_void, len, f);
//         text[len as usize] = 0;
//         trap::FS_FCloseFile(f);
//
//         // parse the text
//         text_p = text.as_ptr();
//         upper_i = 0;
//         lower_i = 0;
//
//         // read information for batches of sounds (UPPER or LOWER)
//         loop {
//             // Get base frame of sequence
//             token = COM_Parse(addr_of_mut!(text_p));
//             if token.is_null() || *token == 0 {
//                 break;
//             }
//
//             if Q_stricmp(token, c"include".as_ptr()) == 0 {
//                 // grab from another animevents.cfg
//                 let include_filename = COM_Parse(addr_of_mut!(text_p));
//                 if !include_filename.is_null() {
//                     let mut fullIPath: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
//                     strcpy(fullIPath.as_mut_ptr(), va(c"models/players/%s/".as_ptr(), include_filename));
//                     bg_animParseIncluding += 1;
//                     BG_ParseAnimationEvtFile(fullIPath.as_ptr(), animFileIndex, forcedIndex);
//                     bg_animParseIncluding -= 1;
//                 }
//             }
//
//             if Q_stricmp(token, c"UPPEREVENTS".as_ptr()) == 0 {
//                 // A batch of upper sounds
//                 ParseAnimationEvtBlock(as_filename, torsoAnimEvents, animations, addr_of_mut!(upper_i), addr_of_mut!(text_p));
//             } else if Q_stricmp(token, c"LOWEREVENTS".as_ptr()) == 0 {
//                 // A batch of lower sounds
//                 ParseAnimationEvtBlock(as_filename, legsAnimEvents, animations, addr_of_mut!(lower_i), addr_of_mut!(text_p));
//             }
//         }
//
//         usedIndex = forcedIndex;
//     }
//     // fin:
//     // Mark this anim set so we know we tried to load the sounds (don't care if it failed)
//     if bg_animParseIncluding <= 0 {
//         bgAllEvents[forcedIndex as usize].eventsParsed = QTRUE;
//         strcpy(bgAllEvents[forcedIndex as usize].filename.as_mut_ptr(), as_filename);
//         if forcedIndex != 0 {
//             bgNumAnimEvents += 1;
//         }
//     }
//
//     usedIndex
// }
// ============================================================================
// END excluded-from-QAGAME translations
// ============================================================================

// libc `atoi`/`atof`/`strcpy`/`strstr` for the native (non-`Q3_VM`) build:
// `BG_ParseAnimationFile` reads frame counts / fps from text tokens and copies the
// filename into the `bgAllAnims` cache. Declared `extern "C"` (the bg_vehicleLoad /
// bg_misc precedent) rather than imported from `bg_lib` (whose copies are `Q3_VM`-only).
extern "C" {
    fn atoi(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
}

/// `BG_ParseAnimationFile( const char *filename, animation_t *animset, qboolean isHumanoid )`
/// (bg_panimate.c:2282) — read a `models/players/<model>/animation.cfg` and fill `animset`
/// with one [`animation_t`] row per recognized [`animTable`] token (firstFrame, numFrames,
/// loopFrames, and fps→frameLerp via `floor`/`ceil` of `1000/fps`). Caches the loaded set
/// into the `bgAllAnims` table and returns its index (0 = humanoid, 1 = rockettrooper, else
/// the next free slot), or -1 on failure.
///
/// Build-faithful to the retail QAGAME / non-`FINAL_BUILD` / native (non-`Q3_VM`) module:
/// - the `#ifndef Q3_VM char BGPAFtext[60000]` scratch buffer is a stack local here; under the
///   `vm` feature (== `Q3_VM`) it is the module-global [`BGPAFtext`] (declared only there).
/// - the `#ifdef _DEBUG` unknown-token warn-and-skip block (carried as a comment below) is
///   omitted — `_DEBUG` is undefined in this build, so an unrecognized token simply falls
///   through to the next `COM_Parse` (each numeric field then re-misses `animTable` and is
///   skipped likewise, so the net effect is identical).
/// - the `#ifdef CONVENIENT_ANIMATION_FILE_DEBUG_THING SpewDebugStuffToFile()` call is omitted
///   (that macro is never defined).
///
/// The C pointer param `animset` is reassigned internally (pass-by-value), so it is `mut`
/// here; the function's outward effect is through the `bgAllAnims`/`bgNumAllAnims`/
/// `BGPAFtextLoaded` globals and the bytes written into the chosen `animset` block.
///
/// No oracle test — the body is engine-trap file I/O (`trap_FS_*`) the off-engine oracle
/// harness cannot satisfy (cf. `BG_VehWeaponLoadParms` / `BG_ModelCache`).
pub unsafe fn BG_ParseAnimationFile(
    filename: *const c_char,
    mut animset: *mut animation_t,
    isHumanoid: qboolean,
) -> c_int {
    let usedIndex: c_int;
    let mut nextIndex: c_int = *addr_of!(bgNumAllAnims);
    let mut dynAlloc: qboolean = QFALSE;

    // `#ifndef Q3_VM char BGPAFtext[60000];` — stack scratch in the native build.
    #[cfg(not(feature = "vm"))]
    let mut BGPAFtext_storage = [0 as c_char; 60000];
    #[cfg(not(feature = "vm"))]
    let bgpaf_text: *mut c_char = BGPAFtext_storage.as_mut_ptr();
    // `#ifdef Q3_VM` — the module-global buffer (declared only under the `vm` feature;
    // a local `let` cannot shadow the static, hence the distinct name).
    #[cfg(feature = "vm")]
    let bgpaf_text: *mut c_char = addr_of_mut!(BGPAFtext) as *mut c_char;

    if isHumanoid == QFALSE {
        let mut i = 0;
        while i < *addr_of!(bgNumAllAnims) {
            // see if it's been loaded already
            if Q_stricmp((*addr_of!(bgAllAnims))[i as usize].filename.as_ptr(), filename) == 0 {
                // C does `animset = bgAllAnims[i].anims;` here, but that write to the
                // pass-by-value pointer is dead — it returns `i` on the next line.
                return i; // alright, we already have it.
            }
            i += 1;
        }

        // Looks like it has not yet been loaded. Allocate space for the anim set if we
        // need to, and continue along.
        if animset.is_null() {
            if !strstr(filename, c"players/_humanoid/".as_ptr()).is_null() {
                // then use the static humanoid set.
                animset = addr_of_mut!(bgHumanoidAnimations) as *mut animation_t;
                nextIndex = 0;
            } else if !strstr(filename, c"players/rockettrooper/".as_ptr()).is_null() {
                // rockettrooper always index 1
                nextIndex = 1;
                animset = BG_AnimsetAlloc();
                dynAlloc = QTRUE; // so we know to free this memory if we return early. Don't want leaks.

                if animset.is_null() {
                    debug_assert!(false, "Anim set alloc failed!");
                    return -1;
                }
            } else {
                animset = BG_AnimsetAlloc();
                dynAlloc = QTRUE; // so we know to free this memory if we return early. Don't want leaks.

                if animset.is_null() {
                    debug_assert!(false, "Anim set alloc failed!");
                    return -1;
                }
            }
        }
    }
    // #ifdef _DEBUG else { assert(animset); } -- _DEBUG undefined in this build.

    // load the file
    if *addr_of!(BGPAFtextLoaded) == QFALSE || isHumanoid == QFALSE {
        // rww - We are always using the same animation config now. So only load it once.
        let fname = CStr::from_ptr(filename).to_string_lossy();
        let (len, f) = trap::FS_FOpenFile(&fname, FS_READ);
        if len <= 0 || len >= (60000 - 1) {
            if dynAlloc != QFALSE {
                BG_AnimsetFree(animset);
            }
            if len > 0 {
                Com_Error(
                    ERR_DROP,
                    &format!("{fname} exceeds the allowed game-side animation buffer!"),
                );
            }
            return -1;
        }

        let buf = core::slice::from_raw_parts_mut(bgpaf_text as *mut u8, len as usize);
        trap::FS_Read(buf, f);
        *bgpaf_text.add(len as usize) = 0;
        trap::FS_FCloseFile(f);
    } else {
        if dynAlloc != QFALSE {
            debug_assert!(false, "Should not have allocated dynamically for humanoid");
            BG_AnimsetFree(animset);
        }
        return 0; // humanoid index
    }

    // parse the text
    let mut text_p: *const c_char = bgpaf_text;

    //FIXME: have some way of playing anims backwards... negative numFrames?

    // initialize anim array so that from 0 to MAX_ANIMATIONS, set default values 0 0 -1 100
    let mut i = 0;
    while i < MAX_ANIMATIONS {
        (*animset.add(i as usize)).firstFrame = 0;
        (*animset.add(i as usize)).numFrames = 0;
        (*animset.add(i as usize)).loopFrames = -1;
        (*animset.add(i as usize)).frameLerp = 100;
        i += 1;
    }

    // read information for each frame
    loop {
        let mut token = COM_Parse(addr_of_mut!(text_p));

        if token.is_null() || *token == 0 {
            break;
        }

        let animNum = GetIDForString(addr_of!(animTable) as *const stringID_table_t, token);
        if animNum == -1 {
            // #ifdef _DEBUG (undefined here):
            //   Com_Printf(S_COLOR_RED"WARNING: Unknown token %s in %s\n", token, filename);
            //   while (token[0]) token = COM_ParseExt(&text_p, qfalse); // consume to EOL
            continue;
        }

        token = COM_Parse(addr_of_mut!(text_p));
        if token.is_null() {
            break;
        }
        (*animset.add(animNum as usize)).firstFrame = atoi(token) as u16;

        token = COM_Parse(addr_of_mut!(text_p));
        if token.is_null() {
            break;
        }
        (*animset.add(animNum as usize)).numFrames = atoi(token) as u16;

        token = COM_Parse(addr_of_mut!(text_p));
        if token.is_null() {
            break;
        }
        (*animset.add(animNum as usize)).loopFrames = atoi(token) as i8;

        token = COM_Parse(addr_of_mut!(text_p));
        if token.is_null() {
            break;
        }
        let mut fps = atof(token) as f32;
        if fps == 0.0 {
            fps = 1.0; // Don't allow divide by zero error
        }
        if fps < 0.0 {
            // backwards
            (*animset.add(animNum as usize)).frameLerp = (1000.0f32 / fps).floor() as i16;
        } else {
            (*animset.add(animNum as usize)).frameLerp = (1000.0f32 / fps).ceil() as i16;
        }
    }
    /*
    #ifdef _DEBUG
        // Check the array, and print the ones that have nothing in them. (commented out in C)
    #endif
    */
    // #ifdef CONVENIENT_ANIMATION_FILE_DEBUG_THING SpewDebugStuffToFile(); -- macro never defined.

    let _wasLoaded = *addr_of!(BGPAFtextLoaded);

    if isHumanoid != QFALSE {
        (*addr_of_mut!(bgAllAnims))[0].anims = animset;
        strcpy((*addr_of_mut!(bgAllAnims))[0].filename.as_mut_ptr(), filename);
        BGPAFtextLoaded = QTRUE;

        usedIndex = 0;
    } else {
        (*addr_of_mut!(bgAllAnims))[nextIndex as usize].anims = animset;
        strcpy(
            (*addr_of_mut!(bgAllAnims))[nextIndex as usize].filename.as_mut_ptr(),
            filename,
        );

        if nextIndex > 1 {
            // don't bother increasing the number if this ended up as a humanoid/rockettrooper load.
            usedIndex = *addr_of!(bgNumAllAnims);
            bgNumAllAnims += 1;
        } else {
            BGPAFtextLoaded = QTRUE;
            usedIndex = nextIndex;
        }
    }

    /*
    if (!wasLoaded && BGPAFtextLoaded) { ... auto-load rockettrooper ... } // commented out in C
    */

    usedIndex
}

/// `BG_StartLegsAnim` is `static` in C; reached only through the `PM_*` callers
/// and `BG_SetAnimFinal` in this file.
unsafe fn BG_StartLegsAnim(ps: *mut playerState_t, anim: c_int) {
    if (*ps).pm_type >= PM_DEAD {
        debug_assert!(BG_InDeathAnim(anim) == QFALSE);
        //please let me know if this assert fires on you (ideally before you close/ignore it) -rww

        //vehicles are allowed to do this.. IF it's a vehicle death anim
        if (*ps).clientNum < MAX_CLIENTS as c_int || anim != BOTH_VT_DEATH1 {
            return;
        }
    }
    if (*ps).legsTimer > 0 {
        return; // a high priority animation is running
    }

    if (*ps).legsAnim == anim {
        BG_FlipPart(ps, SETANIM_LEGS);
    }
    // #ifdef QAGAME
    else if {
        let gents = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
        (*gents.add((*ps).clientNum as usize)).s.legsAnim == anim
    } {
        //toggled anim to one anim then back to the one we were at previously in
        //one frame, indicating that anim should be restarted.
        BG_FlipPart(ps, SETANIM_LEGS);
    }
    // #endif
    (*ps).legsAnim = anim;

    /*
    if ( pm->debugLevel ) {
        Com_Printf("%d:  StartLegsAnim %d, on client#%d\n", pm->cmd.serverTime, anim, pm->ps->clientNum);
    }
    */
}

pub unsafe fn PM_ContinueLegsAnim(anim: c_int) {
    let pmv = *addr_of!(pm);
    if (*(*pmv).ps).legsAnim == anim {
        return;
    }
    if (*(*pmv).ps).legsTimer > 0 {
        return; // a high priority animation is running
    }

    BG_StartLegsAnim((*pmv).ps, anim);
}

pub unsafe fn PM_ForceLegsAnim(anim: c_int) {
    let pmv = *addr_of!(pm);
    if BG_InSpecialJump((*(*pmv).ps).legsAnim) != QFALSE
        && (*(*pmv).ps).legsTimer > 0
        && BG_InSpecialJump(anim) == QFALSE
    {
        return;
    }

    if BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) != QFALSE
        && (*(*pmv).ps).legsTimer > 0
        && BG_InRoll((*pmv).ps, anim) == QFALSE
    {
        return;
    }

    (*(*pmv).ps).legsTimer = 0;
    BG_StartLegsAnim((*pmv).ps, anim);
}

/*
===================
TORSO Animations
Override animations for upper body
===================
*/
unsafe fn BG_StartTorsoAnim(ps: *mut playerState_t, anim: c_int) {
    if (*ps).pm_type >= PM_DEAD {
        debug_assert!(BG_InDeathAnim(anim) == QFALSE);
        //please let me know if this assert fires on you (ideally before you close/ignore it) -rww
        return;
    }

    if (*ps).torsoAnim == anim {
        BG_FlipPart(ps, SETANIM_TORSO);
    }
    // #ifdef QAGAME
    else if {
        let gents = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
        (*gents.add((*ps).clientNum as usize)).s.torsoAnim == anim
    } {
        //toggled anim to one anim then back to the one we were at previously in
        //one frame, indicating that anim should be restarted.
        BG_FlipPart(ps, SETANIM_TORSO);
    }
    // #endif
    (*ps).torsoAnim = anim;
}

pub unsafe fn PM_StartTorsoAnim(anim: c_int) {
    let pmv = *addr_of!(pm);
    BG_StartTorsoAnim((*pmv).ps, anim);
}

/*
-------------------------
PM_SetLegsAnimTimer
-------------------------
*/
pub unsafe fn BG_SetLegsAnimTimer(ps: *mut playerState_t, time: c_int) {
    (*ps).legsTimer = time;

    if (*ps).legsTimer < 0 && time != -1 {
        //Cap timer to 0 if was counting down, but let it be -1 if that was intentional.  NOTENOTE Yeah this seems dumb, but it mirrors SP.
        (*ps).legsTimer = 0;
    }
}

pub unsafe fn PM_SetLegsAnimTimer(time: c_int) {
    let pmv = *addr_of!(pm);
    BG_SetLegsAnimTimer((*pmv).ps, time);
}

/*
-------------------------
PM_SetTorsoAnimTimer
-------------------------
*/
pub unsafe fn BG_SetTorsoAnimTimer(ps: *mut playerState_t, time: c_int) {
    (*ps).torsoTimer = time;

    if (*ps).torsoTimer < 0 && time != -1 {
        //Cap timer to 0 if was counting down, but let it be -1 if that was intentional.  NOTENOTE Yeah this seems dumb, but it mirrors SP.
        (*ps).torsoTimer = 0;
    }
}

pub unsafe fn PM_SetTorsoAnimTimer(time: c_int) {
    let pmv = *addr_of!(pm);
    BG_SetTorsoAnimTimer((*pmv).ps, time);
}

pub unsafe fn BG_SaberStartTransAnim(
    clientNum: c_int,
    saberAnimLevel: c_int,
    weapon: c_int,
    anim: c_int,
    animSpeed: *mut f32,
    broken: c_int,
) {
    if anim >= BOTH_A1_T__B_ && anim <= BOTH_ROLL_STAB {
        if weapon == WP_SABER {
            let mut saber: *mut saberInfo_t = BG_MySaber(clientNum, 0);
            if !saber.is_null() && (*saber).animSpeedScale != 1.0f32 {
                *animSpeed *= (*saber).animSpeedScale;
            }
            saber = BG_MySaber(clientNum, 1);
            if !saber.is_null() && (*saber).animSpeedScale != 1.0f32 {
                *animSpeed *= (*saber).animSpeedScale;
            }
        }
    }

    if ((anim) >= BOTH_T1_BR__R && (anim) <= BOTH_T1_BL_TL)
        || ((anim) >= BOTH_T2_BR__R && (anim) <= BOTH_T2_BL_TL)
        || ((anim) >= BOTH_T3_BR__R && (anim) <= BOTH_T3_BL_TL)
    {
        if saberAnimLevel == FORCE_LEVEL_1 {
            *animSpeed *= 1.5;
        } else if saberAnimLevel == FORCE_LEVEL_3 {
            *animSpeed *= 0.75;
        }

        if broken & (1 << BROKENLIMB_RARM) != 0 {
            *animSpeed *= 0.5;
        } else if broken & (1 << BROKENLIMB_LARM) != 0 {
            *animSpeed *= 0.65;
        }
    } else if broken != 0 && PM_InSaberAnim(anim) != QFALSE {
        if broken & (1 << BROKENLIMB_RARM) != 0 {
            *animSpeed *= 0.5;
        } else if broken & (1 << BROKENLIMB_LARM) != 0 {
            *animSpeed *= 0.65;
        }
    }
}

/*
-------------------------
PM_SetAnimFinal
-------------------------
*/
// (The C forward-declares `PM_RunningAnim`/`PM_WalkingAnim` here; they live in
// bg_pmove.rs and are imported above.)

// `blendTime` is immediately overwritten to 0 in the C (the comment notes
// "Setting blendTime here breaks actual blending"), so the parameter is unused.
unsafe fn BG_SetAnimFinal(
    ps: *mut playerState_t,
    animations: *mut animation_t,
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
    _blendTime: c_int, // default blendTime=350
) {
    let mut editAnimSpeed: f32 = 1.0;

    if animations.is_null() {
        return;
    }

    debug_assert!(anim > -1);
    let a = animations.add(anim as usize);
    let firstFrame = (*a).firstFrame;
    let numFrames = (*a).numFrames;
    let frameLerp = (*a).frameLerp;
    debug_assert!(firstFrame > 0 || numFrames > 0);

    //NOTE: Setting blendTime here breaks actual blending..
    // blendTime = 0;

    BG_SaberStartTransAnim(
        (*ps).clientNum,
        (*ps).fd.saberAnimLevel,
        (*ps).weapon,
        anim,
        &mut editAnimSpeed,
        (*ps).brokenLimbs,
    );

    // Set torso anim
    if setAnimParts & SETANIM_TORSO != 0 {
        // The C uses `goto setAnimLegs` to skip the rest of the torso block; a
        // labelled block whose `break` jumps to the start of the legs block.
        'setAnimLegs: {
            // Don't reset if it's already running the anim
            if (setAnimFlags & SETANIM_FLAG_RESTART) == 0 && (*ps).torsoAnim == anim {
                break 'setAnimLegs;
            }
            // or if a more important anim is running
            if (setAnimFlags & SETANIM_FLAG_OVERRIDE) == 0
                && ((*ps).torsoTimer > 0 || (*ps).torsoTimer == -1)
            {
                break 'setAnimLegs;
            }

            BG_StartTorsoAnim(ps, anim);

            if setAnimFlags & SETANIM_FLAG_HOLD != 0 {
                if setAnimFlags & SETANIM_FLAG_HOLDLESS != 0 {
                    // Make sure to only wait in full 1/20 sec server frame intervals.
                    let dur: c_int;
                    let speedDif: c_int;

                    // dur = (animations[anim].numFrames-1) * fabs((float)(animations[anim].frameLerp));
                    let dur0 = (numFrames as c_int - 1) as f64 * (frameLerp as f32 as f64).abs();
                    let dur0 = dur0 as c_int;
                    speedDif = (dur0 as f32 - (dur0 as f32 * editAnimSpeed)) as c_int;
                    dur = dur0 + speedDif;
                    if dur > 1 {
                        (*ps).torsoTimer = dur - 1;
                    } else {
                        (*ps).torsoTimer = (frameLerp as f32 as f64).abs() as c_int;
                    }
                } else {
                    (*ps).torsoTimer =
                        (numFrames as f64 * (frameLerp as f32 as f64).abs()) as c_int;
                }

                if (*ps).fd.forcePowersActive & (1 << FP_RAGE) != 0 {
                    (*ps).torsoTimer = ((*ps).torsoTimer as f64 / 1.7) as c_int;
                }
            }
        }
    }

    // setAnimLegs:
    // Set legs anim
    if setAnimParts & SETANIM_LEGS != 0 {
        // The C `goto setAnimDone` skips the rest of the legs block.
        'setAnimDone: {
            // Don't reset if it's already running the anim
            if (setAnimFlags & SETANIM_FLAG_RESTART) == 0 && (*ps).legsAnim == anim {
                break 'setAnimDone;
            }
            // or if a more important anim is running
            if (setAnimFlags & SETANIM_FLAG_OVERRIDE) == 0
                && ((*ps).legsTimer > 0 || (*ps).legsTimer == -1)
            {
                break 'setAnimDone;
            }

            BG_StartLegsAnim(ps, anim);

            if setAnimFlags & SETANIM_FLAG_HOLD != 0 {
                if setAnimFlags & SETANIM_FLAG_HOLDLESS != 0 {
                    // Make sure to only wait in full 1/20 sec server frame intervals.
                    let dur: c_int;
                    let speedDif: c_int;

                    let dur0 = (numFrames as c_int - 1) as f64 * (frameLerp as f32 as f64).abs();
                    let dur0 = dur0 as c_int;
                    speedDif = (dur0 as f32 - (dur0 as f32 * editAnimSpeed)) as c_int;
                    dur = dur0 + speedDif;
                    if dur > 1 {
                        (*ps).legsTimer = dur - 1;
                    } else {
                        (*ps).legsTimer = (frameLerp as f32 as f64).abs() as c_int;
                    }
                } else {
                    (*ps).legsTimer =
                        (numFrames as f64 * (frameLerp as f32 as f64).abs()) as c_int;
                }

                if PM_RunningAnim(anim) != QFALSE || PM_WalkingAnim(anim) != QFALSE
                //these guys are ok, they don't actually reference pm
                {
                    if (*ps).fd.forcePowersActive & (1 << FP_RAGE) != 0 {
                        (*ps).legsTimer = ((*ps).legsTimer as f64 / 1.3) as c_int;
                    } else if (*ps).fd.forcePowersActive & (1 << FP_SPEED) != 0 {
                        (*ps).legsTimer = ((*ps).legsTimer as f64 / 1.7) as c_int;
                    }
                }
            }
        }
    }

    // setAnimDone:
}

pub unsafe fn PM_SetAnimFinal(
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
    blendTime: c_int,
) {
    let pmv = *addr_of!(pm);
    BG_SetAnimFinal((*pmv).ps, (*pmv).animations, setAnimParts, anim, setAnimFlags, blendTime);
}

pub unsafe fn BG_HasAnimation(animIndex: c_int, animation: c_int) -> qboolean {
    //must be a valid anim number
    if animation < 0 || animation >= MAX_ANIMATIONS {
        return QFALSE;
    }

    //Must have a file index entry
    if animIndex < 0 || animIndex > *addr_of!(bgNumAllAnims) {
        return QFALSE;
    }

    let animations = (*addr_of!(bgAllAnims))[animIndex as usize].anims;

    //No frames, no anim
    if (*animations.add(animation as usize)).numFrames == 0 {
        return QFALSE;
    }

    //Has the sequence
    QTRUE
}

pub unsafe fn BG_PickAnim(animIndex: c_int, minAnim: c_int, maxAnim: c_int) -> c_int {
    let mut anim: c_int;
    let mut count = 0;

    loop {
        anim = Q_irand(minAnim, maxAnim);
        count += 1;
        if !(BG_HasAnimation(animIndex, anim) == QFALSE && count < 1000) {
            break;
        }
    }

    if count == 1000 {
        //guess we just don't have a death anim then.
        return -1;
    }

    anim
}

//I want to be able to use this on a playerstate even when we are not the focus
//of a pmove too so I have ported it to true BGishness.
//Please do not reference pm in this function or any functions that it calls,
//or I will cry. -rww
pub unsafe fn BG_SetAnim(
    ps: *mut playerState_t,
    mut animations: *mut animation_t,
    setAnimParts: c_int,
    mut anim: c_int,
    setAnimFlags: c_int,
    blendTime: c_int,
) {
    if animations.is_null() {
        animations = (*addr_of!(bgAllAnims))[0].anims;
    }

    if (*animations.add(anim as usize)).firstFrame == 0
        && (*animations.add(anim as usize)).numFrames == 0
    {
        if anim == BOTH_RUNBACK1 || anim == BOTH_WALKBACK1 || anim == BOTH_RUN1 {
            //hack for droids
            anim = BOTH_WALK2;
        }

        if (*animations.add(anim as usize)).firstFrame == 0
            && (*animations.add(anim as usize)).numFrames == 0
        {
            //still? Just return then I guess.
            return;
        }
    }

    /*
    if (BG_InSpecialJump(anim))
    {
        setAnimFlags |= SETANIM_FLAG_RESTART;
    }
    */
    //Don't know why I put this here originally but it's messing stuff up now and it isn't needed.

    //	if (BG_InRoll(ps, ps->legsAnim))
    //	{ //never interrupt a roll
    //		return;
    //	}

    if setAnimFlags & SETANIM_FLAG_OVERRIDE != 0 {
        if setAnimParts & SETANIM_TORSO != 0 {
            if (setAnimFlags & SETANIM_FLAG_RESTART) != 0 || (*ps).torsoAnim != anim {
                BG_SetTorsoAnimTimer(ps, 0);
            }
        }
        if setAnimParts & SETANIM_LEGS != 0 {
            if (setAnimFlags & SETANIM_FLAG_RESTART) != 0 || (*ps).legsAnim != anim {
                BG_SetLegsAnimTimer(ps, 0);
            }
        }
    }

    BG_SetAnimFinal(ps, animations, setAnimParts, anim, setAnimFlags, blendTime);
}

pub unsafe fn PM_SetAnim(setAnimParts: c_int, anim: c_int, setAnimFlags: c_int, blendTime: c_int) {
    let pmv = *addr_of!(pm);
    BG_SetAnim((*pmv).ps, (*pmv).animations, setAnimParts, anim, setAnimFlags, blendTime);
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;

    // The full input domain: well below 0 (exercises the `_`/`default` arm with
    // negatives) up past MAX_TOTALANIMATIONS (1544) — covers every animNumber_t
    // and every saberMoveName_t (LS_MOVE_MAX = 162), saberQuadrant_t and
    // saberBlockedType_t value, plus the out-of-range tails on both sides.
    const LO: c_int = -8;
    const HI: c_int = 2100;

    // Drives every stateless predicate that depends only on its integer argument
    // through the Rust port and the verbatim-C oracle, asserting bit-identical
    // output across the whole input domain. This transitively verifies the
    // BOTH_*/LS_*/Q_*/BLOCKED_* constant values baked into each case label.
    // BG_BrokenParryForParry is excluded here (its LS_PARRY_UP branch is random)
    // and covered separately below.
    #[test]
    fn all_stateless_predicates_match_oracle() {
        for i in LO..=HI {
            macro_rules! chk {
                ($f:ident) => {
                    assert_eq!(
                        super::$f(i),
                        unsafe { crate::oracle::$f(i) },
                        "{} input {}",
                        stringify!($f),
                        i
                    );
                };
            }
            chk!(BG_SaberStanceAnim);
            chk!(BG_CrouchAnim);
            chk!(BG_InSpecialJump);
            chk!(BG_InSaberStandAnim);
            chk!(BG_InReboundJump);
            chk!(BG_InReboundHold);
            chk!(BG_InReboundRelease);
            chk!(BG_InBackFlip);
            chk!(BG_DirectFlippingAnim);
            chk!(BG_SaberInAttackPure);
            chk!(BG_SaberInAttack);
            chk!(BG_SaberInKata);
            chk!(BG_InKataAnim);
            chk!(BG_SaberInSpecial);
            chk!(BG_KickMove);
            chk!(BG_SaberInIdle);
            chk!(BG_InExtraDefenseSaberMove);
            chk!(BG_FlippingAnim);
            chk!(BG_SpinningSaberAnim);
            chk!(BG_SaberInSpecialAttack);
            chk!(BG_KickingAnim);
            chk!(BG_InGrappleMove);
            chk!(BG_KnockawayForParry);
            chk!(BG_InSpecialDeathAnim);
            chk!(BG_InDeathAnim);
            chk!(BG_InKnockDownOnly);
            chk!(BG_InSaberLockOld);
            chk!(BG_InSaberLock);
            chk!(PM_InCartwheel);
            chk!(BG_StabDownAnim);
            chk!(PM_SaberDeflectionForQuad);
            chk!(PM_SaberInDeflect);
            chk!(PM_SaberInParry);
            chk!(PM_SaberInKnockaway);
            chk!(PM_SaberInReflect);
            chk!(PM_SaberInStart);
            chk!(PM_SaberInReturn);
            chk!(BG_SaberInReturn);
            chk!(PM_InSaberAnim);
            chk!(PM_PainAnim);
            chk!(PM_JumpingAnim);
            chk!(PM_LandingAnim);
            chk!(PM_SpinningAnim);
            chk!(PM_InOnGroundAnim);
            chk!(BG_SuperBreakLoseAnim);
            chk!(BG_SuperBreakWinAnim);
            chk!(BG_SaberLockBreakAnim);
            chk!(BG_FullBodyTauntAnim);
            chk!(PM_SaberInTransition);
            chk!(BG_SaberInTransitionAny);
        }
    }

    // BG_BrokenParryForParry's LS_PARRY_UP case calls Q_irand(0,1) — the Rust
    // port uses q_shared's LCG, the oracle resolves to q_shared_oracle.c's copy,
    // and the two RNG states are independent, so equality can't hold there. The
    // sweep compares every OTHER input against the oracle; LS_PARRY_UP is checked
    // for set-membership in the two possible outcomes the C source can return.
    #[test]
    fn bg_brokenparryforparry_matches_oracle() {
        // LS_PARRY_UP drives Q_irand → irand's shared `holdrand` LCG; the result is
        // checked by set-membership (below) so the exact draw is irrelevant. Hold
        // bg_lib's rand lock for test hygiene against the parallel runner.
        let _guard = crate::codemp::game::bg_lib::rand_lock();
        for i in LO..=HI {
            if i == LS_PARRY_UP {
                let r = BG_BrokenParryForParry(i);
                assert!(
                    r == LS_H1_B_ || r == LS_H1_T_,
                    "LS_PARRY_UP must yield LS_H1_B_ or LS_H1_T_, got {r}"
                );
                continue;
            }
            assert_eq!(
                BG_BrokenParryForParry(i),
                unsafe { crate::oracle::BG_BrokenParryForParry(i) },
                "input {i}"
            );
        }
    }

    // BG_BrokenParryForAttack / PM_SaberBounceForAttack switch on
    // saberMoveData[move].startQuad, so they index the table — unlike the other
    // predicates they can't be swept past the array ends (both languages would
    // read out of bounds). Sweep every valid saber-move index instead; this still
    // exercises all eight startQuad values present in the table (Q_B..Q_BL → every
    // LS_V1_*/LS_B1_* return) and the LS_NONE default for any unmapped quadrant.
    #[test]
    fn saber_move_indexed_predicates_match_oracle() {
        for i in 0..LS_MOVE_MAX {
            assert_eq!(
                BG_BrokenParryForAttack(i),
                unsafe { crate::oracle::BG_BrokenParryForAttack(i) },
                "BG_BrokenParryForAttack input {i}"
            );
            assert_eq!(
                PM_SaberBounceForAttack(i),
                unsafe { crate::oracle::PM_SaberBounceForAttack(i) },
                "PM_SaberBounceForAttack input {i}"
            );
        }
    }

    // The four playerState_t* predicates read only ps->legsAnim and ps->legsTimer.
    // Build a zeroed playerState_t (pointer-free POD), set the two fields, and
    // compare against the verbatim-C bodies (driven through int-marshalling
    // wrappers over a minimal struct in the oracle TU). The legsTimer set hits
    // every threshold the four functions compare against (>0, !=0, <1, <700, >250);
    // the anim / legsAnim axis sweeps the full domain to exercise each case label.
    #[test]
    fn playerstate_predicates_match_oracle() {
        let timers = [-1, 0, 1, 249, 250, 251, 699, 700, 701, 999, 1000, 1001];
        let mut ps: playerState_t = unsafe { core::mem::zeroed() };
        for &t in &timers {
            for a in LO..=HI {
                ps.legsTimer = t;
                ps.legsAnim = a;
                unsafe {
                    // BG_InRoll: switches on the anim argument, reads legsTimer.
                    assert_eq!(
                        BG_InRoll(&mut ps, a),
                        crate::oracle::jka_BG_InRoll(t, a, a),
                        "BG_InRoll legsTimer={t} anim={a}"
                    );
                    // PM_InKnockDown: switches on ps->legsAnim, reads legsTimer.
                    assert_eq!(
                        PM_InKnockDown(&mut ps),
                        crate::oracle::jka_PM_InKnockDown(t, a),
                        "PM_InKnockDown legsTimer={t} legsAnim={a}"
                    );
                    // PM_InRollComplete: switches on the anim argument, reads legsTimer.
                    assert_eq!(
                        PM_InRollComplete(&mut ps, a),
                        crate::oracle::jka_PM_InRollComplete(t, a, a),
                        "PM_InRollComplete legsTimer={t} anim={a}"
                    );
                    // PM_CanRollFromSoulCal: reads ps->legsAnim and legsTimer.
                    assert_eq!(
                        PM_CanRollFromSoulCal(&mut ps),
                        crate::oracle::jka_PM_CanRollFromSoulCal(t, a),
                        "PM_CanRollFromSoulCal legsTimer={t} legsAnim={a}"
                    );
                }
            }
        }
    }
}

// Parity tests for the animation-SETTER cluster (BG_SetAnim / BG_SetAnimFinal /
// BG_SaberStartTransAnim and their transitive call graph: BG_FlipPart, the
// legs/torso starters + timer setters, PM_InSaberAnim / PM_RunningAnim /
// PM_WalkingAnim). Driven against the verbatim-C oracle in
// oracle/bg_panimate_setters_oracle.c. The `pm`/`g_entities` globals are
// serialized via pm_lock(); the float timer math is asserted bit-exact.
#[cfg(all(test, feature = "oracle"))]
mod setter_tests {
    use super::*;
    use crate::codemp::game::bg_pmove::pm_lock;
    use crate::codemp::game::bg_public::pmove_t;
    use crate::codemp::game::bg_weapons_h::WP_NONE;
    use crate::codemp::game::g_local::gentity_t;
    use core::mem::MaybeUninit;
    use core::ptr::null_mut;

    // Which entry point a scenario drives. The `Pm*` variants route through the
    // `pm` keystone (PM_SetAnim / PM_SetAnimFinal forward to the BG_* core with
    // pm->ps / pm->animations), so matching the oracle BG_* result transitively
    // verifies the forwarder + the pm dereference.
    #[derive(Clone, Copy)]
    enum Mode {
        Final,
        Set,
        PmFinal,
        PmSet,
    }

    // Build Rust state from the 22-int `input`, run the chosen entry point, return
    // the 6 mutated playerState_t fields. `anims_ptr` and the `g_entities` global
    // must already be set up; touched slots are reset before return so the buffers
    // can be reused.
    unsafe fn rust_run(anims_ptr: *mut animation_t, input: &[c_int; 22], mode: Mode) -> [c_int; 6] {
        let gptr = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
        let cn = input[1] as usize;
        (*gptr.add(cn)).s.legsAnim = input[15];
        (*gptr.add(cn)).s.torsoAnim = input[16];

        let ai = input[11] as usize;
        *anims_ptr.add(ai) = animation_t {
            firstFrame: input[12] as u16,
            numFrames: input[13] as u16,
            frameLerp: input[14] as i16,
            loopFrames: 0,
        };
        if input[19] >= 0 {
            *anims_ptr.add(input[19] as usize) = animation_t {
                firstFrame: input[20] as u16,
                numFrames: input[21] as u16,
                frameLerp: 0,
                loopFrames: 0,
            };
        }

        let mut ps: playerState_t = MaybeUninit::zeroed().assume_init();
        ps.pm_type = input[0];
        ps.clientNum = input[1];
        ps.legsTimer = input[2];
        ps.torsoTimer = input[3];
        ps.legsAnim = input[4];
        ps.torsoAnim = input[5];
        ps.legsFlip = input[6];
        ps.torsoFlip = input[7];
        ps.brokenLimbs = input[8];
        ps.fd.saberAnimLevel = input[9];
        ps.fd.forcePowersActive = input[10];

        let (parts, anim, flags) = (input[17], input[11], input[18]);
        match mode {
            Mode::Final => BG_SetAnimFinal(&mut ps, anims_ptr, parts, anim, flags, 350),
            Mode::Set => BG_SetAnim(&mut ps, anims_ptr, parts, anim, flags, 350),
            Mode::PmFinal | Mode::PmSet => {
                let mut pmv: pmove_t = MaybeUninit::zeroed().assume_init();
                pmv.ps = &mut ps;
                pmv.animations = anims_ptr;
                *addr_of_mut!(pm) = &mut pmv;
                match mode {
                    Mode::PmFinal => PM_SetAnimFinal(parts, anim, flags, 350),
                    _ => PM_SetAnim(parts, anim, flags, 350),
                }
                *addr_of_mut!(pm) = null_mut();
            }
        }

        // reset touched slots for the next scenario
        (*gptr.add(cn)).s.legsAnim = 0;
        (*gptr.add(cn)).s.torsoAnim = 0;
        *anims_ptr.add(ai) = animation_t::default();
        if input[19] >= 0 {
            *anims_ptr.add(input[19] as usize) = animation_t::default();
        }

        [
            ps.legsTimer,
            ps.torsoTimer,
            ps.legsAnim,
            ps.torsoAnim,
            ps.legsFlip,
            ps.torsoFlip,
        ]
    }

    // The oracle has no `pm`; PmFinal/PmSet compare against the same BG_* the
    // forwarder ultimately calls.
    unsafe fn oracle_run(input: &[c_int; 22], mode: Mode) -> [c_int; 6] {
        let mut out = [0i32; 6];
        match mode {
            Mode::Set | Mode::PmSet => crate::oracle::jka_BG_SetAnim(input.as_ptr(), out.as_mut_ptr()),
            Mode::Final | Mode::PmFinal => {
                crate::oracle::jka_BG_SetAnimFinal(input.as_ptr(), out.as_mut_ptr())
            }
        }
        out
    }

    // (pm_type, clientNum, legsTimer, torsoTimer, legsAnim, torsoAnim, legsFlip,
    //  torsoFlip, brokenLimbs, saberAnimLevel, forcePowersActive)
    const BASES: &[[c_int; 11]] = &[
        [0, 3, 0, 0, 0, 0, 0, 0, 0, 1, 0],
        [0, 3, 5, 5, 0, 0, 0, 0, 0, 3, 1 << FP_RAGE],
        [0, 5, -1, -1, 999, 888, 1, 1, 1 << BROKENLIMB_RARM, 0, 1 << FP_SPEED],
        [5, 3, 0, 0, 0, 0, 0, 0, 1 << BROKENLIMB_LARM, 1, 0], // PM_DEAD, client < MAX
        [5, 40, 0, 0, 0, 0, 0, 0, 0, 1, 0],                   // PM_DEAD, client >= MAX
        [0, 7, 5, 0, 777, 666, 0, 1, 0, 1, 1 << FP_RAGE],
    ];

    // anim indices spanning the branch space: saber-transition, death, run, walk,
    // runback (droid source), and a plain anim.
    const ANIMS: &[c_int] = &[
        BOTH_T1_BR__R, // saber transition -> editAnimSpeed != 1
        BOTH_VT_DEATH1,
        BOTH_RUN1,
        BOTH_WALK2,
        BOTH_RUNBACK1,
        400,
    ];

    // (firstFrame, numFrames, frameLerp) -- all keep firstFrame>0 || numFrames>0
    // so BG_SetAnimFinal's debug_assert holds.
    const FRAMES: &[(u16, u16, i16)] = &[
        (1, 10, 50),
        (5, 20, -33),
        (1, 1, 100),
        (1, 200, 7),
        (1, 0, -100),
    ];

    const FLAGS: &[c_int] = &[
        0,
        SETANIM_FLAG_OVERRIDE,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_HOLDLESS,
        SETANIM_FLAG_HOLD,
        SETANIM_FLAG_RESTART | SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        SETANIM_FLAG_RESTART,
    ];

    const PARTS: &[c_int] = &[SETANIM_TORSO, SETANIM_LEGS, SETANIM_BOTH];

    #[test]
    fn bg_set_anim_final_matches_oracle() {
        let _g = pm_lock();
        let mut gents: Vec<gentity_t> = (0..64)
            .map(|_| unsafe { MaybeUninit::zeroed().assume_init() })
            .collect();
        let mut anims: Vec<animation_t> = vec![animation_t::default(); 2048];
        unsafe {
            core::ptr::copy_nonoverlapping(gents.as_mut_ptr(), core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), gents.len());
            let anims_ptr = anims.as_mut_ptr();

            for base in BASES {
                for &anim in ANIMS {
                    // A death anim under PM_DEAD would trip BG_Start*Anim's
                    // debug_assert (mirrors the C assert; release-only path).
                    if base[0] >= PM_DEAD && BG_InDeathAnim(anim) != QFALSE {
                        continue;
                    }
                    for &(ff, nf, fl) in FRAMES {
                        for &parts in PARTS {
                            for &flags in FLAGS {
                                // 3 variants: normal, current-anim-matches, ent-matches.
                                for variant in 0..3 {
                                    let mut input: [c_int; 22] = [
                                        base[0], base[1], base[2], base[3], base[4], base[5],
                                        base[6], base[7], base[8], base[9], base[10], anim,
                                        ff as c_int, nf as c_int, fl as c_int, 0, 0, parts, flags,
                                        -1, 0, 0,
                                    ];
                                    match variant {
                                        1 => {
                                            input[4] = anim; // legsAnim == anim
                                            input[5] = anim; // torsoAnim == anim
                                        }
                                        2 => {
                                            input[15] = anim; // g_entities legsAnim == anim
                                            input[16] = anim; // g_entities torsoAnim == anim
                                        }
                                        _ => {}
                                    }

                                    let got = rust_run(anims_ptr, &input, Mode::Final);
                                    let want = oracle_run(&input, Mode::Final);
                                    assert_eq!(got, want, "BG_SetAnimFinal input {:?}", input);
                                }
                            }
                        }
                    }
                }
            }

            core::ptr::write_bytes(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), 0, gents.len());
        }
    }

    #[test]
    fn bg_set_anim_matches_oracle() {
        let _g = pm_lock();
        let mut gents: Vec<gentity_t> = (0..64)
            .map(|_| unsafe { MaybeUninit::zeroed().assume_init() })
            .collect();
        let mut anims: Vec<animation_t> = vec![animation_t::default(); 2048];
        // SetAnim adds a (0,0) frame entry to exercise the droid / early-return path.
        let frames_set: &[(u16, u16, i16)] = &[(1, 10, 50), (1, 0, -100), (0, 0, 0)];
        unsafe {
            core::ptr::copy_nonoverlapping(gents.as_mut_ptr(), core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), gents.len());
            let anims_ptr = anims.as_mut_ptr();

            for base in BASES {
                for &anim in ANIMS {
                    if base[0] >= PM_DEAD && BG_InDeathAnim(anim) != QFALSE {
                        continue;
                    }
                    for &(ff, nf, fl) in frames_set {
                        for &parts in PARTS {
                            for &flags in FLAGS {
                                for variant in 0..3 {
                                    let mut input: [c_int; 22] = [
                                        base[0], base[1], base[2], base[3], base[4], base[5],
                                        base[6], base[7], base[8], base[9], base[10], anim,
                                        ff as c_int, nf as c_int, fl as c_int, 0, 0, parts, flags,
                                        -1, 0, 0,
                                    ];
                                    match variant {
                                        1 => {
                                            input[4] = anim;
                                            input[5] = anim;
                                        }
                                        2 => {
                                            input[15] = anim;
                                            input[16] = anim;
                                        }
                                        _ => {}
                                    }

                                    let got = rust_run(anims_ptr, &input, Mode::Set);
                                    let want = oracle_run(&input, Mode::Set);
                                    assert_eq!(got, want, "BG_SetAnim input {:?}", input);
                                }
                            }
                        }
                    }
                }
            }

            core::ptr::write_bytes(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), 0, gents.len());
        }
    }

    // BG_SetAnim's droid hack: anim with empty frames that IS RUNBACK1/WALKBACK1/
    // RUN1 redirects to BOTH_WALK2; here WALK2 carries valid frames so the redirect
    // proceeds into BG_SetAnimFinal (rather than early-returning).
    #[test]
    fn bg_set_anim_droid_redirect_matches_oracle() {
        let _g = pm_lock();
        let mut gents: Vec<gentity_t> = (0..64)
            .map(|_| unsafe { MaybeUninit::zeroed().assume_init() })
            .collect();
        let mut anims: Vec<animation_t> = vec![animation_t::default(); 2048];
        unsafe {
            core::ptr::copy_nonoverlapping(gents.as_mut_ptr(), core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), gents.len());
            let anims_ptr = anims.as_mut_ptr();

            for &src in &[BOTH_RUNBACK1, BOTH_WALKBACK1, BOTH_RUN1] {
                for &flags in FLAGS {
                    for &parts in PARTS {
                        let input: [c_int; 22] = [
                            0, 3, 0, 0, 0, 0, 0, 0, 0, 1, 1 << FP_RAGE, src, 0, 0, 0, 0, 0, parts,
                            flags, BOTH_WALK2, 1, 30,
                        ];
                        let got = rust_run(anims_ptr, &input, Mode::Set);
                        let want = oracle_run(&input, Mode::Set);
                        assert_eq!(got, want, "BG_SetAnim droid input {:?}", input);
                    }
                }
            }

            core::ptr::write_bytes(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), 0, gents.len());
        }
    }

    // BG_SaberStartTransAnim: full anim-domain sweep crossing every T1/T2/T3 and
    // saber-anim branch boundary, against saber levels / broken-limb masks / start
    // speeds. The float result is asserted bit-exact. Driven with weapon=WP_NONE so
    // the PC `weapon==WP_SABER` per-saber `animSpeedScale` scale path is skipped on
    // both sides (it reads live saber-subsystem state via BG_MySaber, exercised by
    // bg_saberLoad's own parity tests); this keeps the sweep on the level/broken math.
    #[test]
    fn bg_saber_start_trans_anim_matches_oracle() {
        for anim in 0..=1543i32 {
            for &level in &[0, FORCE_LEVEL_1, 2, FORCE_LEVEL_3] {
                for &broken in &[0, 1 << BROKENLIMB_RARM, 1 << BROKENLIMB_LARM, 6] {
                    for &speed in &[1.0f32, 2.5, 0.3] {
                        let mut s = speed;
                        unsafe { BG_SaberStartTransAnim(0, level, WP_NONE, anim, &mut s, broken) };
                        let want = unsafe {
                            crate::oracle::jka_BG_SaberStartTransAnim(0, level, WP_NONE, anim, speed, broken)
                        };
                        assert_eq!(
                            s.to_bits(),
                            want.to_bits(),
                            "BG_SaberStartTransAnim level={level} anim={anim} broken={broken} speed={speed}"
                        );
                    }
                }
            }
        }
    }

    // PM_SetAnim / PM_SetAnimFinal forward to the BG_* core through the `pm`
    // keystone; a sample of scenarios routed through them must match the oracle
    // BG_* exactly (verifies the pm->ps/pm->animations dereference + the forward).
    #[test]
    fn pm_setanim_forwarders_match_oracle() {
        let _g = pm_lock();
        let mut gents: Vec<gentity_t> = (0..64)
            .map(|_| unsafe { MaybeUninit::zeroed().assume_init() })
            .collect();
        let mut anims: Vec<animation_t> = vec![animation_t::default(); 2048];
        unsafe {
            core::ptr::copy_nonoverlapping(gents.as_mut_ptr(), core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), gents.len());
            let anims_ptr = anims.as_mut_ptr();

            for base in BASES {
                for &anim in ANIMS {
                    if base[0] >= PM_DEAD && BG_InDeathAnim(anim) != QFALSE {
                        continue;
                    }
                    for &(ff, nf, fl) in FRAMES {
                        for &flags in FLAGS {
                            let input: [c_int; 22] = [
                                base[0], base[1], base[2], base[3], base[4], base[5], base[6],
                                base[7], base[8], base[9], base[10], anim, ff as c_int, nf as c_int,
                                fl as c_int, 0, 0, SETANIM_BOTH, flags, -1, 0, 0,
                            ];
                            for &(m, om) in
                                &[(Mode::PmFinal, Mode::Final), (Mode::PmSet, Mode::Set)]
                            {
                                let got = rust_run(anims_ptr, &input, m);
                                let want = oracle_run(&input, om);
                                assert_eq!(got, want, "PM forwarder input {:?}", input);
                            }
                        }
                    }
                }
            }

            core::ptr::write_bytes(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), 0, gents.len());
        }
    }

    // PM_ContinueLegsAnim / PM_ForceLegsAnim gate BG_StartLegsAnim (itself verified
    // transitively above) using BG_InSpecialJump / BG_InRoll (both C-oracle-verified
    // in bg_panimate_oracle.c). Here we assert the gate behavior directly: when a
    // gate passes, ps->legsAnim becomes `anim`; when it blocks, legsAnim is unchanged.
    #[test]
    fn pm_continue_force_legs_anim_gates() {
        let _g = pm_lock();
        let mut gents: Vec<gentity_t> = (0..4)
            .map(|_| unsafe { MaybeUninit::zeroed().assume_init() })
            .collect();
        unsafe {
            core::ptr::copy_nonoverlapping(gents.as_mut_ptr(), core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), gents.len());

            let run = |pm_type: c_int, legs_anim: c_int, legs_timer: c_int, new_anim: c_int, force: bool| -> (c_int, c_int) {
                let mut ps: playerState_t = MaybeUninit::zeroed().assume_init();
                ps.pm_type = pm_type;
                ps.clientNum = 1;
                ps.legsAnim = legs_anim;
                ps.legsTimer = legs_timer;
                let mut pmv: pmove_t = MaybeUninit::zeroed().assume_init();
                pmv.ps = &mut ps;
                *addr_of_mut!(pm) = &mut pmv;
                if force {
                    PM_ForceLegsAnim(new_anim);
                } else {
                    PM_ContinueLegsAnim(new_anim);
                }
                *addr_of_mut!(pm) = null_mut();
                (ps.legsAnim, ps.legsTimer)
            };

            // PM_ContinueLegsAnim: already on `anim` -> no change.
            assert_eq!(run(0, BOTH_RUN1, 0, BOTH_RUN1, false), (BOTH_RUN1, 0));
            // high-priority timer running -> blocked, legsAnim unchanged.
            assert_eq!(run(0, BOTH_WALK1, 500, BOTH_RUN1, false).0, BOTH_WALK1);
            // free (timer 0, different anim) -> starts the new anim.
            assert_eq!(run(0, BOTH_WALK1, 0, BOTH_RUN1, false).0, BOTH_RUN1);

            // PM_ForceLegsAnim: zeroes the timer then starts (overrides a running anim).
            assert_eq!(run(0, BOTH_WALK1, 500, BOTH_RUN1, true), (BOTH_RUN1, 0));
            // in a special-jump with timer left, not switching to another special
            // jump -> blocked (BG_InSpecialJump gate).
            assert_eq!(
                run(0, BOTH_ARIAL_LEFT, 500, BOTH_RUN1, true).0,
                BOTH_ARIAL_LEFT
            );
            // in a roll (timer>0), not rolling target -> blocked (BG_InRoll gate).
            assert_eq!(run(0, BOTH_ROLL_F, 500, BOTH_RUN1, true).0, BOTH_ROLL_F);

            core::ptr::write_bytes(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), 0, gents.len());
        }
    }

    // BG_HasAnimation: range checks + a single bgAllAnims[idx].anims[anim].numFrames
    // read. Set up the real bgAllAnims global (indices 0..=bgNumAllAnims) and sweep
    // animIndex / animation / numFrames against the verbatim-C oracle.
    #[test]
    fn bg_has_animation_matches_oracle() {
        let _g = pm_lock();
        const NUM_ALL: c_int = 2;
        // backing animation arrays for bgAllAnims[0..=NUM_ALL]
        let mut backing: Vec<Vec<animation_t>> = (0..=NUM_ALL)
            .map(|_| vec![animation_t::default(); MAX_TOTALANIMATIONS as usize])
            .collect();
        unsafe {
            let saved_num = *addr_of!(bgNumAllAnims);
            *addr_of_mut!(bgNumAllAnims) = NUM_ALL;
            for i in 0..=NUM_ALL as usize {
                (*addr_of_mut!(bgAllAnims))[i].anims = backing[i].as_mut_ptr();
            }

            for &animation in &[-1, 0, 5, 50, MAX_ANIMATIONS - 1, MAX_ANIMATIONS, 1600] {
                for &num_frames in &[0i32, 1, 30] {
                    // write the slot used by valid (animIndex,animation) pairs
                    if (0..MAX_TOTALANIMATIONS).contains(&animation) {
                        for i in 0..=NUM_ALL as usize {
                            backing[i][animation as usize].numFrames = num_frames as u16;
                        }
                    }
                    for &anim_index in &[-1, 0, 1, 2, 3] {
                        let got = BG_HasAnimation(anim_index, animation);
                        let want = crate::oracle::jka_BG_HasAnimation(
                            anim_index, animation, NUM_ALL, num_frames,
                        );
                        assert_eq!(
                            got, want,
                            "BG_HasAnimation animIndex={anim_index} animation={animation} numFrames={num_frames}"
                        );
                    }
                    // reset the slot
                    if (0..MAX_TOTALANIMATIONS).contains(&animation) {
                        for i in 0..=NUM_ALL as usize {
                            backing[i][animation as usize].numFrames = 0;
                        }
                    }
                }
            }

            // restore globals (null the anims pointers -- backing is about to drop)
            for i in 0..=NUM_ALL as usize {
                (*addr_of_mut!(bgAllAnims))[i].anims = core::ptr::null_mut();
            }
            *addr_of_mut!(bgNumAllAnims) = saved_num;
        }
    }

    // BG_InitAnimsets (PC) only zeroes bgAllAnims + clears BGPAFtextLoaded; the
    // bgNumAllAnims/bgNumAnimEvents counters are left at their init defaults (2/1)
    // — PC relocated those resets next to the unported event parser. BG_AnimsetAlloc
    // fills bgNumAllAnims's slot pointer from the bg_pool. No C oracle -- these are
    // trivial state inits over the BG_Alloc bump allocator (the g_mem precedent).
    #[test]
    fn bg_initanimsets_and_alloc_behavioral() {
        let _g = pm_lock();
        unsafe {
            BG_InitAnimsets();
            assert_eq!(*addr_of!(bgNumAllAnims), 2);
            assert_eq!(*addr_of!(bgNumAnimEvents), 1);
            assert_eq!(*addr_of!(BGPAFtextLoaded), QFALSE);
            // bgAllAnims zeroed -> slot 0 anims pointer is null
            assert!((*addr_of!(bgAllAnims))[0].anims.is_null());

            // BG_AnimsetAlloc fills the bgNumAllAnims (== 2) slot with a fresh,
            // non-null block and returns the same pointer.
            let p = BG_AnimsetAlloc();
            assert!(!p.is_null());
            assert_eq!((*addr_of!(bgAllAnims))[2].anims, p);

            // leave the globals clean for other tests
            BG_InitAnimsets();
        }
    }

    // BG_PickAnim loops Q_irand over [minAnim,maxAnim] until BG_HasAnimation passes
    // (or 1000 tries -> -1). No C oracle (shared-LCG RNG state diverges, like
    // BG_BrokenParryForParry); assert the contract instead. Holds rand_lock() since
    // it drives the Q_irand LCG.
    #[test]
    fn bg_pick_anim_behavioral() {
        let _g = pm_lock();
        let _r = crate::codemp::game::bg_lib::rand_lock();
        const NUM_ALL: c_int = 2;
        let mut backing: Vec<animation_t> = vec![animation_t::default(); MAX_TOTALANIMATIONS as usize];
        unsafe {
            let saved_num = *addr_of!(bgNumAllAnims);
            *addr_of_mut!(bgNumAllAnims) = NUM_ALL;
            (*addr_of_mut!(bgAllAnims))[1].anims = backing.as_mut_ptr();

            // Exactly one valid anim in [40,45]: index 42 has frames.
            backing[42].numFrames = 10;
            for _ in 0..50 {
                assert_eq!(BG_PickAnim(1, 40, 45), 42);
            }

            // No valid anim in the range -> -1 after 1000 tries.
            backing[42].numFrames = 0;
            assert_eq!(BG_PickAnim(1, 40, 45), -1);

            (*addr_of_mut!(bgAllAnims))[1].anims = core::ptr::null_mut();
            *addr_of_mut!(bgNumAllAnims) = saved_num;
        }
    }

    // BG_AnimLength: anim >= MAX_ANIMATIONS guard, else
    // numFrames * fabs((float)frameLerp) -> int over bgAllAnims[0].anims[anim].
    // Sweep anim (incl. the >= MAX_ANIMATIONS guard) x numFrames x frameLerp
    // (incl. i16 extremes) against the verbatim-C oracle. anim is kept >= 0 (C
    // has no negative guard here -> a negative index would be OOB in both).
    #[test]
    fn bg_anim_length_matches_oracle() {
        let _g = pm_lock();
        let mut backing: Vec<animation_t> =
            vec![animation_t::default(); MAX_TOTALANIMATIONS as usize];
        unsafe {
            let saved = (*addr_of!(bgAllAnims))[0].anims;
            (*addr_of_mut!(bgAllAnims))[0].anims = backing.as_mut_ptr();
            for &anim in &[0, 1, 42, 400, MAX_ANIMATIONS - 1, MAX_ANIMATIONS, 1600] {
                for &nf in &[0i32, 1, 10, 200, 65535] {
                    for &fl in &[0i32, 1, -1, 7, -33, 100, -100, 32767, -32768] {
                        if (0..MAX_TOTALANIMATIONS).contains(&anim) {
                            backing[anim as usize].numFrames = nf as u16;
                            backing[anim as usize].frameLerp = fl as i16;
                        }
                        let got = BG_AnimLength(0, anim);
                        let want = crate::oracle::jka_BG_AnimLength(anim, nf, fl);
                        assert_eq!(got, want, "BG_AnimLength anim={anim} nf={nf} fl={fl}");
                    }
                }
            }
            (*addr_of_mut!(bgAllAnims))[0].anims = saved;
        }
    }

    // PM_AnimLength: same arithmetic over pm->animations[anim] (shared oracle),
    // plus the extra `!pm->animations` guard asserted behaviorally. anim kept
    // >= 0 so the `anim < 0 -> Com_Error` (diverging, aborting) path is not hit.
    #[test]
    fn pm_anim_length_matches_oracle() {
        let _g = pm_lock();
        let mut backing: Vec<animation_t> =
            vec![animation_t::default(); MAX_TOTALANIMATIONS as usize];
        unsafe {
            let mut pmv: pmove_t = MaybeUninit::zeroed().assume_init();
            pmv.animations = backing.as_mut_ptr();
            *addr_of_mut!(pm) = &mut pmv;
            for &anim in &[0, 1, 42, 400, MAX_ANIMATIONS - 1, MAX_ANIMATIONS, 1600] {
                for &nf in &[0i32, 1, 10, 200, 65535] {
                    for &fl in &[0i32, 1, -1, 7, -33, 100, -100, 32767, -32768] {
                        if (0..MAX_TOTALANIMATIONS).contains(&anim) {
                            backing[anim as usize].numFrames = nf as u16;
                            backing[anim as usize].frameLerp = fl as i16;
                        }
                        let got = PM_AnimLength(0, anim);
                        let want = crate::oracle::jka_BG_AnimLength(anim, nf, fl);
                        assert_eq!(got, want, "PM_AnimLength anim={anim} nf={nf} fl={fl}");
                    }
                }
            }
            // !pm->animations -> -1 before any read (write through the pm global
            // alias so the subsequent raw-pointer read is observable).
            (*(*addr_of_mut!(pm))).animations = null_mut();
            assert_eq!(PM_AnimLength(0, 42), -1);
            *addr_of_mut!(pm) = null_mut();
        }
    }
}
