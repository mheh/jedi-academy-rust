//! `bg_pmove.c` — "both games player movement code: takes a playerstate and a
//! usercmd as input and returns a modified playerstate." This is the bit-exact
//! ~10,880-LOC movement monster; it is ported **data-layer-first** per the 1.03
//! roadmap (the `bg_saber.c` / `bg_vehicleLoad.c` precedent). The first slice ported
//! the file's pointer-free file-scope globals and the five numeric tuning tables —
//! the data every later logic slice consumes.
//!
//! The **`pm`/`pml` keystone** then begins here: the context globals (`pm`, `pml`,
//! `pm_entSelf`, `pm_entVeh`) that the whole `PM_*` family reads, followed by the
//! `PM_AddEvent`/`PM_AddEventWithParm`/`PM_AddTouchEnt` event helpers. The remaining
//! `PM_*`/`Pmove*` movement logic lands in later source-ordered slices (not yet
//! ported, kept as comments below).
//!
//! Mutability mirrors C use: the 13 `pm_*` movement-parameter floats and the five
//! force tables are never written at runtime, so they are immutable `pub static`
//! lookups (the `bg_weapons.rs` precedent). The three genuine runtime-state globals
//! — `c_pmove` (the frame counter, `c_pmove++` in `PmoveSingle`), `gPMDoSlowFall`
//! and `pm_cancelOutZoom` (set during movement/weapon code) — are `static mut`,
//! matching their non-const C definitions (all zero-initialized).
//!
//! Landing the force tables here **retires three not-yet-ported `extern` declarations**
//! carried as source-order comments in [`bg_local_h`](crate::codemp::game::bg_local_h)
//! and [`w_saber_h`](crate::codemp::game::w_saber_h): `forcePowerNeeded`,
//! `forceJumpHeight`, `forceJumpStrength`. (The fourth in those lists,
//! `forcePushPullRadius`, is NOT defined in `bg_pmove.c` — it lives in another TU
//! (`w_force.c`), so its not-yet-ported-extern comment stays.) It also unblocks the
//! `forcePowerNeeded[]` consumers in `bg_saber.c` (`BG_ForcePowerDrain`) and the
//! not-yet-ported force-cost predicates in `bg_panimate.c`.
//!
//! Oracle: the real `bg_pmove.c` cannot be `#include`d (its quoted `#include`s drag
//! in the clang-hostile reference tree). So `oracle/bg_pmove_oracle.c` transcribes
//! the three sizing constants (`NUM_FORCE_POWERS`, `NUM_FORCE_POWER_LEVELS`,
//! `JUMP_VELOCITY`) and the table/param DATA VERBATIM from `bg_pmove.c`, exposing
//! pointers; the test below compares every element, catching any single-value typo
//! on either side.

#![allow(non_upper_case_globals)]

use crate::codemp::game::anims::{
    BOTH_FORCE_GETUP_B1, BOTH_FORCE_GETUP_B2, BOTH_FORCE_GETUP_B3, BOTH_FORCE_GETUP_B4,
    BOTH_FORCE_GETUP_B5, BOTH_FORCE_GETUP_F1, BOTH_FORCE_GETUP_F2, BOTH_GETUP1, BOTH_GETUP2,
    BOTH_GETUP3, BOTH_GETUP4, BOTH_GETUP5, BOTH_GETUP_BROLL_B, BOTH_GETUP_BROLL_F,
    BOTH_GETUP_BROLL_L, BOTH_GETUP_BROLL_R, BOTH_GETUP_FROLL_B, BOTH_GETUP_FROLL_F,
    BOTH_GETUP_FROLL_L, BOTH_GETUP_FROLL_R, BOTH_KNOCKDOWN1, BOTH_KNOCKDOWN2, BOTH_KNOCKDOWN3,
    BOTH_KNOCKDOWN4, BOTH_KNOCKDOWN5,
    BOTH_JUMPATTACK6, BOTH_ROLL_B, BOTH_ROLL_F, BOTH_ROLL_L, BOTH_ROLL_R, BOTH_RUN1, BOTH_RUN1START,
    BOTH_RUN1STOP, BOTH_RUN2, BOTH_RUNBACK1, BOTH_RUNBACK2, BOTH_RUNBACK_DUAL, BOTH_RUNBACK_STAFF,
    BOTH_RUNSTRAFE_LEFT1, BOTH_RUNSTRAFE_RIGHT1, BOTH_RUN_DUAL, BOTH_RUN_STAFF, BOTH_SABERDUAL_STANCE,
    BOTH_SABERFAST_STANCE, BOTH_SABERSLOW_STANCE, BOTH_SABERSTAFF_STANCE, BOTH_STAND1, BOTH_STAND2,
    BOTH_STAND3, BOTH_STAND5, BOTH_CROUCH1, BOTH_STRAFE_LEFT1, BOTH_STRAFE_RIGHT1,
    TORSO_WEAPONREADY1, TORSO_WEAPONREADY2, TORSO_WEAPONREADY3, TORSO_WEAPONREADY10,
    BOTH_SWIMBACKWARD, BOTH_SWIMFORWARD, BOTH_SWIM_IDLE1, BOTH_WALK1, BOTH_WALK2, BOTH_WALK5,
    BOTH_WALK6, BOTH_WALK7, BOTH_WALKBACK1, BOTH_WALKBACK2, BOTH_WALKBACK_DUAL, BOTH_WALKBACK_STAFF,
    BOTH_WALK_DUAL, BOTH_WALK_STAFF, BOTH_WALL_RUN_LEFT, BOTH_WALL_RUN_RIGHT, LEGS_LEFTUP1,
    LEGS_LEFTUP2, LEGS_LEFTUP3, LEGS_LEFTUP4, LEGS_LEFTUP5, LEGS_RIGHTUP1, LEGS_RIGHTUP2,
    LEGS_RIGHTUP3, LEGS_RIGHTUP4, LEGS_RIGHTUP5, LEGS_S1_LUP1, LEGS_S1_LUP2, LEGS_S1_LUP3,
    LEGS_S1_LUP4, LEGS_S1_LUP5, LEGS_S1_RUP1, LEGS_S1_RUP2, LEGS_S1_RUP3, LEGS_S1_RUP4,
    LEGS_S1_RUP5, LEGS_S3_LUP1, LEGS_S3_LUP2, LEGS_S3_LUP3, LEGS_S3_LUP4, LEGS_S3_LUP5,
    LEGS_S3_RUP1, LEGS_S3_RUP2, LEGS_S3_RUP3, LEGS_S3_RUP4, LEGS_S3_RUP5, LEGS_S4_LUP1,
    LEGS_S4_LUP2, LEGS_S4_LUP3, LEGS_S4_LUP4, LEGS_S4_LUP5, LEGS_S4_RUP1, LEGS_S4_RUP2,
    LEGS_S4_RUP3, LEGS_S4_RUP4, LEGS_S4_RUP5, LEGS_S5_LUP1, LEGS_S5_LUP2, LEGS_S5_LUP3,
    LEGS_S5_LUP4, LEGS_S5_LUP5, LEGS_S5_RUP1, LEGS_S5_RUP2, LEGS_S5_RUP3, LEGS_S5_RUP4,
    LEGS_S5_RUP5,
};
use crate::codemp::game::bg_local_h::{pml_t, MIN_WALK_NORMAL, OVERCLIP};
use crate::codemp::game::bg_slidemove::{PM_GroundSlideOkay, PM_SlideMove, PM_StepSlideMove};
use crate::codemp::game::bg_misc::{
    vectoyaw, BG_AddPredictableEventToPlayerstate, BG_CanUseFPNow, BG_GetItemIndexByTag,
    BG_HasYsalamiri,
};
use crate::codemp::game::bg_panimate::{
    BG_SaberInAttack, BG_SaberInSpecialAttack, BG_SpinningSaberAnim, PM_CanRollFromSoulCal,
    PM_SaberInStart,
};
use crate::codemp::game::g_main::{Com_Error, Com_Printf};
use crate::codemp::game::g_utils::{G_AddEvent, G_PlayEffect, G_PlayEffectID};
use crate::codemp::game::g_active::G_CheapWeaponFire;
use crate::codemp::game::bg_public::{
    EFFECT_ACID_SPLASH, EFFECT_LAVA_SPLASH, EFFECT_WATER_SPLASH, EV_PLAY_EFFECT_ID,
};
use crate::codemp::game::bg_public::{
    bgEntity_t, pmove_t, CROUCH_VIEWHEIGHT, DEAD_VIEWHEIGHT, DEFAULT_VIEWHEIGHT, EF2_FLYING,
    EV_TAUNT, GT_SIEGE, JUMP_VELOCITY, LS_NONE,
    MASK_PLAYERSOLID, MASK_SOLID, MASK_WATER, MAXTOUCH, MINS_Z, PMF_ALL_TIMES, PMF_BACKWARDS_RUN,
    PMF_DUCKED,
    PMF_FIX_MINS, PMF_STUCK_TO_WALL, PMF_TIME_KNOCKBACK, PMF_TIME_WATERJUMP, PM_FLOAT, PM_NORMAL,
    PM_SPECTATOR, STAT_WEAPONS, STEPSIZE,
};
use crate::codemp::game::bg_vehicles_h::{
    Vehicle_t, MIN_LANDING_SLOPE, VH_ANIMAL, VH_FIGHTER, VH_SPEEDER, VH_WALKER,
};
use crate::codemp::game::bg_weapons_h::{WP_BLASTER, WP_MELEE, WP_NUM_WEAPONS, WP_SABER};
use crate::codemp::game::q_math::{
    vec3_origin, AngleDelta, AngleMod, AngleNormalize180, AngleNormalize360, AngleSubtract,
    AnglesSubtract, AngleVectors,
    DotProduct, Q_fabs,
    VectorAdd, VectorClear, VectorCompare, VectorCopy, VectorLength, VectorLengthSquared, VectorMA,
    VectorNormalize, VectorScale, VectorSet, VectorSubtract, vectoangles,
};
use crate::codemp::game::q_shared_h::{
    entityState_t, mdxaBone_t, playerState_t, qboolean, trace_t, usercmd_t, vec3_t, vec_t,
    ANGLE2SHORT,
    BUTTON_ALT_ATTACK,
    BUTTON_ATTACK, BUTTON_GESTURE, ENTITYNUM_NONE, ENTITYNUM_WORLD, FORCE_LEVEL_0, FORCE_LEVEL_1, FORCE_LEVEL_2,
    FORCE_LEVEL_3, FP_LEVITATION, MAX_CLIENTS, MAX_GENTITIES, NUM_FORCE_POWERS,
    NUM_FORCE_POWER_LEVELS, PITCH, QFALSE, QTRUE, ROLL, SFL_NO_FLIPS, SFL_NO_ROLLS,
    SFL_NO_WALL_FLIPS, SFL_NO_WALL_GRAB, SFL_NO_WALL_RUNS, SHORT2ANGLE, SOLID_BMODEL,
    SS_DUAL, SS_FAST, SS_STAFF, SS_STRONG, SS_TAVION, YAW,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_LAVA, CONTENTS_SLIME, CONTENTS_SOLID, CONTENTS_WATER, MATERIAL_DIRT,
    MATERIAL_GRAVEL, MATERIAL_MASK, MATERIAL_MUD, MATERIAL_SAND, MATERIAL_SNOW, SURF_NOSTEPS,
    SURF_SLICK,
};
use crate::codemp::game::teams_h::CLASS_VEHICLE;
// Ground-trace / crash-land slice (bg_pmove.c:3583-4129) extra deps:
use crate::codemp::game::anims::{
    BOTH_A7_KICK_B_AIR, BOTH_A7_KICK_F_AIR, BOTH_A7_KICK_L_AIR, BOTH_A7_KICK_R_AIR, BOTH_A7_SOULCAL,
    BOTH_CHOKE3, BOTH_FORCEJUMP1, BOTH_FORCEJUMPBACK1, BOTH_FORCEJUMPLEFT1, BOTH_FORCEJUMPRIGHT1,
    BOTH_FORCELAND1, BOTH_FORCELANDBACK1, BOTH_FORCELANDLEFT1, BOTH_FORCELANDRIGHT1, BOTH_GUNSIT1,
    BOTH_INAIR1, BOTH_JUMP1, BOTH_JUMPBACK1, BOTH_JUMPLEFT1, BOTH_JUMPRIGHT1, BOTH_LAND1,
    BOTH_LANDBACK1, BOTH_LANDLEFT1, BOTH_LANDRIGHT1, TORSO_WEAPONREADY4,
};
// PM_CheckJump (bg_pmove.c:1766) extra anim deps:
use crate::codemp::game::anims::{
    BOTH_BUTTERFLY_LEFT, BOTH_FLIP_B, BOTH_FLIP_BACK1, BOTH_FLIP_BACK2, BOTH_FLIP_BACK3,
    BOTH_FLIP_F, BOTH_FLIP_L, BOTH_FLIP_R, BOTH_FORCEINAIR1, BOTH_FORCEINAIRBACK1,
    BOTH_FORCEINAIRLEFT1, BOTH_FORCEINAIRRIGHT1, BOTH_FORCEWALLREBOUND_BACK,
    BOTH_FORCEWALLREBOUND_FORWARD, BOTH_FORCEWALLREBOUND_LEFT, BOTH_FORCEWALLREBOUND_RIGHT,
    BOTH_FORCEWALLRUNFLIP_END, BOTH_FORCEWALLRUNFLIP_START, BOTH_WALL_FLIP_BACK1,
    BOTH_WALL_FLIP_LEFT, BOTH_WALL_FLIP_RIGHT, BOTH_WALL_RUN_LEFT_FLIP, BOTH_WALL_RUN_RIGHT_FLIP,
};
use crate::codemp::game::bg_local_h::TIMER_LAND;
// Wall-run/wall-jump angle-adjuster (bg_pmove.c:1334-1736) extra anim deps:
use crate::codemp::game::anims::{
    BOTH_FORCEWALLHOLD_BACK, BOTH_FORCEWALLHOLD_FORWARD, BOTH_FORCEWALLHOLD_LEFT,
    BOTH_FORCEWALLHOLD_RIGHT, BOTH_FORCEWALLRELEASE_FORWARD, BOTH_WALL_RUN_LEFT_STOP,
    BOTH_WALL_RUN_RIGHT_STOP,
};
use crate::codemp::game::bg_misc::{WeaponReadyAnim, WeaponReadyLegsAnim};
use crate::codemp::game::bg_panimate::{
    BG_AnimLength, BG_InBackFlip, BG_InDeathAnim, BG_InReboundHold, BG_InReboundJump, BG_InRoll,
    BG_InSpecialJump, BG_SaberInSpecial, PM_AnimLength, PM_ForceLegsAnim, PM_InKnockDown,
    PM_InOnGroundAnim,
    PM_InRollComplete, PM_SetAnim, PM_StartTorsoAnim,
};
use crate::codemp::game::bg_public::{
    EV_FALL, EV_FOOTSTEP, EV_JUMP, EV_ROLL, ET_NPC, EV_SABER_ATTACK, GT_DUEL, GT_POWERDUEL, GT_TEAM,
    HANDEXTEND_KNOCKDOWN, HANDEXTEND_NONE, HANDEXTEND_POSTTHROWN, HANDEXTEND_PRETHROWN, LS_R_TL2BR,
    PMF_BACKWARDS_JUMP, PMF_JUMP_HELD, PMF_RESPAWNED, PMF_ROLLING, PMF_TIME_LAND, PM_JETPACK,
    SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_NORMAL, SETANIM_FLAG_OVERRIDE,
    SETANIM_FLAG_RESTART, SETANIM_LEGS,
};
use crate::codemp::game::bg_saber::{BG_ForcePowerDrain, PM_WalkableGroundDistance, PM_irand_timesync};
// vehicle weapon anims (PM_VehicleWeaponAnimate)
use crate::codemp::game::anims::{
    BOTH_ATTACK3, BOTH_VS_AIR_G, BOTH_VS_ATF_G, BOTH_VS_ATL_G, BOTH_VS_ATL_S, BOTH_VS_ATR_G,
    BOTH_VS_ATR_S, BOTH_VS_IDLE, BOTH_VS_IDLE_G, BOTH_VS_IDLE_SL, BOTH_VS_IDLE_SR, BOTH_VS_LAND_G,
    BOTH_VS_LAND_SL, BOTH_VS_LAND_SR, BOTH_VS_REV, BOTH_VT_ATF_G, BOTH_VT_ATL_G, BOTH_VT_ATL_S,
    BOTH_VT_ATR_G, BOTH_VT_ATR_S, BOTH_VT_IDLE, BOTH_VT_IDLE_G, BOTH_VT_IDLE_S, BOTH_VT_RUN_FWD,
    BOTH_VT_TURBO, BOTH_VT_WALK_REV,
};
use crate::codemp::game::bg_weapons_h::{WP_DISRUPTOR, WP_EMPLACED_GUN};
use crate::codemp::game::g_cmds::TryGrapple;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::{bg_fighterAltControl, g_entities, g_gametype};
use crate::codemp::game::surfaceflags_h::SURF_NODAMAGE;
// PmoveSingle / Pmove keystone (bg_pmove.c:9837 / 10830) extra deps:
use crate::codemp::game::anims::{
    BOTH_A2_SPECIAL, BOTH_A3_SPECIAL,
    BOTH_A2_STABBACK1, BOTH_ATTACK_BACK, BOTH_CROUCHATTACKBACK1, BOTH_FORCELEAP2_T__B_,
    BOTH_FORCEWALLRUNFLIP_ALT, BOTH_INAIRBACK1, BOTH_INAIRLEFT1, BOTH_INAIRRIGHT1,
    BOTH_JUMPFLIPSLASHDOWN1, BOTH_JUMPFLIPSTABDOWN, BOTH_MEDITATE, BOTH_MEDITATE_END,
    BOTH_ROLL_STAB,
};
use crate::codemp::game::bg_public::{
    EF_ALT_FIRING, EF_DISINTEGRATION, EF_FIRING, EF_JETPACK_FLAMING, EF_NODRAW, EF_TALK,
    EV_WEAPON_CHARGE, HANDEXTEND_TAUNT, HYPERSPACE_TIME, LS_A_BACK, LS_A_BACKSTAB,
    LS_A_BACK_CR, LS_A_FLIP_SLASH, LS_A_FLIP_STAB, LS_A_JUMP_T__B_, LS_A_LUNGE, LS_DUAL_FB,
    LS_DUAL_LR, LS_PULL_ATTACK_STAB, LS_PULL_ATTACK_SWING, LS_STABDOWN,
    LS_STABDOWN_DUAL, LS_STABDOWN_STAFF, LS_STAFF_SOULCAL, PM_DEAD, PM_FREEZE, PM_INTERMISSION, PM_NOCLIP,
    PM_SPINTERMISSION, STAT_HEALTH, STAT_HOLDABLE_ITEMS, WEAPON_CHARGING_ALT,
    WEAPON_READY,
};
use crate::codemp::game::bg_panimate::{
    BG_FullBodyTauntAnim, BG_InGrappleMove, BG_InKataAnim, BG_KickMove, BG_KickingAnim,
    BG_SaberInKata, BG_SaberLockBreakAnim, PM_ContinueLegsAnim, PM_SaberInTransition,
};
// PM_CmdForSaberMoves (bg_pmove.c:9232) extra deps:
use crate::codemp::game::anims::{
    BOTH_BUTTERFLY_FL1, BOTH_BUTTERFLY_FR1, BOTH_BUTTERFLY_RIGHT, BOTH_JUMPATTACK7,
};
use crate::codemp::game::bg_public::{
    LS_A_BACKFLIP_ATK, LS_BUTTERFLY_LEFT, LS_BUTTERFLY_RIGHT, LS_JUMPATTACK_DUAL,
    LS_JUMPATTACK_STAFF_LEFT, LS_JUMPATTACK_STAFF_RIGHT, LS_SPINATTACK, LS_SPINATTACK_DUAL,
};
// PM_VehicleViewAngles / PM_VehForcedTurning / PM_VehFaceHyperspacePoint (bg_pmove.c:9402-9655)
// extra deps:
use crate::codemp::game::bg_public::{DEFAULT_MINS_2, EF2_HYPERSPACE, HYPERSPACE_TELEPORT_FRAC};
use crate::codemp::game::bg_vehicles_h::{MAX_VEHICLE_TURRETS, VH_FLIER};
use crate::codemp::game::q_math::AnglesToAxis;
use crate::codemp::game::q_math::Distance;
use crate::codemp::game::g_weapon::WP_GetVehicleCamPos;
use crate::codemp::game::bg_saber::PM_GroundDistance;
// PM_WaterEvents (bg_pmove.c:5518) extra deps (the EFFECT_*_SPLASH constants arrive with
// the not-yet-ported G_PlayEffect calls):
use crate::codemp::game::bg_public::{
    EV_DISRUPTOR_ZOOMSOUND, EV_USE, EV_WATER_CLEAR, EV_WATER_LEAVE, EV_WATER_TOUCH, EV_WATER_UNDER,
    HANDEXTEND_DODGE,
};
// PM_CrashLandEffect (bg_pmove.c:3536) material->effect map (the not-yet-ported G_PlayEffect arg):
use crate::codemp::game::bg_public::{
    EFFECT_LANDING_DIRT, EFFECT_LANDING_GRAVEL, EFFECT_LANDING_MUD, EFFECT_LANDING_SAND,
    EFFECT_LANDING_SNOW,
};
use crate::codemp::game::q_shared_h::{FP_GRIP, FP_RAGE, FP_SPEED};
use crate::codemp::game::bg_vehicles_h::{VEH_CRASHING, VEH_FLYING};
use crate::codemp::game::bg_weapons::weaponData;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{
    BUTTON_FORCEGRIP, BUTTON_FORCEPOWER, BUTTON_FORCE_DRAIN, BUTTON_FORCE_LIGHTNING, BUTTON_TALK,
    BUTTON_USE, BUTTON_USE_HOLDABLE, BUTTON_WALKING, PS_PMOVEFRAMECOUNTBITS,
};
use crate::codemp::game::teams_h::{CLASS_RANCOR, CLASS_WAMPA};
// PM_Footsteps (bg_pmove.c:5027) pull-ins.
use crate::codemp::game::anims::{
    BOTH_BUTTON_HOLD, BOTH_BUTTON_RELEASE, BOTH_CROUCH1IDLE, BOTH_CROUCH1WALK, BOTH_CROUCH1WALKBACK,
    BOTH_STAND4,
};
use crate::codemp::game::bg_public::{EF2_ALERTED, EF2_USE_ALT_ANIM, EV_FOOTSPLASH, EV_SWIM};
use crate::codemp::game::anims::{BOTH_STAND1TO2, BOTH_STAND2TO1};
use crate::codemp::game::bg_panimate::{PM_InSaberAnim, PM_LandingAnim, PM_PainAnim};
use crate::codemp::ghoul2::g2_h::BONE_ANGLES_POSTMULT;
use crate::codemp::game::q_shared_h::{ERR_DROP, NEGATIVE_Y, NEGATIVE_Z, POSITIVE_X};
// BG_IK_MoveArm (ghoul2 IK arm) deps:
use crate::codemp::game::bg_panimate::bgHumanoidAnimations;
use crate::codemp::game::q_shared_h::{
    sharedIKMoveParams_t, sharedRagDollUpdateParams_t, sharedSetBoneIKStateParams_t, IKS_DYNAMIC,
    IKS_NONE,
};
use crate::trap;
use core::ffi::{c_int, c_short, c_ulong, c_void};
// PM_BeginWeaponChange / PM_FinishWeaponChange / PM_CanSetWeaponAnims (bg_pmove.c:5604/5637/6155).
use crate::codemp::game::anims::{TORSO_DROPWEAP1, TORSO_RAISEWEAP1};
use crate::codemp::game::bg_public::{
    EV_CHANGE_WEAPON, LS_DRAW, SETANIM_TORSO, WEAPON_DROPPING, WEAPON_RAISING,
};
use crate::codemp::game::bg_saber::PM_SetSaberMove;
use crate::codemp::game::bg_weapons_h::WP_NONE;
// PM_RocketLock / PM_DoChargedWeapons (bg_pmove.c:5660/5757).
use crate::codemp::game::bg_public::{ET_PLAYER, EV_WEAPON_CHARGE_ALT, PW_CLOAKED, WEAPON_CHARGING};
use crate::codemp::game::bg_vehicleLoad::g_vehWeaponInfo;
use crate::codemp::game::bg_weapons::WP_MuzzlePoint;
use crate::codemp::game::bg_weapons_h::{
    WP_BOWCASTER, WP_BRYAR_OLD, WP_BRYAR_PISTOL, WP_CONCUSSION, WP_DEMP2, WP_ROCKET_LAUNCHER,
    WP_THERMAL,
};
use core::ptr::{addr_of, addr_of_mut, null_mut};
// PM_Weapon (bg_pmove.c:6427) pull-ins. (Several BOTH_* getup/knockdown/stand anims are
// already imported above for earlier slices.)
use crate::codemp::game::anims::{
    BOTH_A3_TL_BR, BOTH_ATTACK4, BOTH_B1_BL___, BOTH_D3_TL___, BOTH_ENGAGETAUNT,
    BOTH_FORCE_2HANDEDLIGHTNING_HOLD, BOTH_FORCEGRIP_HOLD, BOTH_FORCELIGHTNING_HOLD, BOTH_FORCEPULL,
    BOTH_FORCEPUSH, BOTH_GESTURE1, BOTH_KNEES1, BOTH_MELEE1, BOTH_MELEE2, BOTH_SABERPULL,
    BOTH_STAND6, TORSO_WEAPONIDLE3,
};
use crate::codemp::game::bg_misc::WeaponAttackAnim;
use crate::codemp::game::bg_panimate::BG_FlippingAnim;
use crate::codemp::game::bg_public::{
    EV_ALT_FIRE, EV_FIRE_WEAPON, EV_NOAMMO, HANDEXTEND_CHOKE, HANDEXTEND_DRAGGING,
    HANDEXTEND_DUELCHALLENGE, HANDEXTEND_FORCEPULL, HANDEXTEND_FORCEPUSH, HANDEXTEND_FORCE_HOLD,
    HANDEXTEND_JEDITAUNT, HANDEXTEND_POSTTHROW, HANDEXTEND_PRETHROW, HANDEXTEND_SABERPULL,
    HANDEXTEND_WEAPONREADY, LS_HILT_BASH, LS_KICK_B, LS_KICK_B_AIR, LS_KICK_F, LS_KICK_F_AIR,
    LS_KICK_L, LS_KICK_L_AIR, LS_KICK_R, LS_KICK_R_AIR, LS_READY, PERS_TEAM, PMF_USE_ITEM_HELD,
    IT_HOLDABLE, STAT_HOLDABLE_ITEM, TEAM_SPECTATOR, WEAPON_FIRING,
};
use crate::codemp::game::bg_saber::{saberMoveData, PM_KickMoveForConditions, PM_WeaponLightsaber};
use crate::codemp::game::bg_saber::BG_MySaber;
use crate::codemp::game::q_shared_h::saberInfo_t;
use crate::codemp::game::bg_weapons_h::{WP_DET_PACK, WP_STUN_BATON, WP_TRIP_MINE};
use crate::codemp::game::q_shared_h::{FP_DRAIN, FP_LIGHTNING};
// PM_ItemUsable (bg_pmove.c:6025) + the holdable-item consumption block (:6844-6881) pull-ins.
use crate::codemp::game::bg_misc::{bg_itemlist, BG_CycleInven, BG_IsItemSelectable};
use crate::codemp::game::bg_public::{
    EF_DEAD, EF_SEEKERDRONE, EV_ITEMUSEFAIL, EV_USE_ITEM0, HI_AMMODISP, HI_BINOCULARS, HI_CLOAK,
    HI_EWEB, HI_HEALTHDISP, HI_JETPACK, HI_MEDPAC, HI_MEDPAC_BIG, HI_SEEKER, HI_SENTRY_GUN,
    HI_SHIELD, MASK_SHOT, STAT_MAX_HEALTH,
};
use crate::codemp::game::q_shared_h::{
    SEEKER_ALREADYDEPLOYED, SENTRY_ALREADYPLACED, SENTRY_NOROOM, SHIELD_NOROOM,
};

/// `MAX_WEAPON_CHARGE_TIME` (bg_pmove.c:16) — cap on how long a non-disruptor/rocket/thermal
/// weapon may sit in a `WEAPON_CHARGING(_ALT)` state before the fire button is force-released.
const MAX_WEAPON_CHARGE_TIME: c_int = 5000;

/// `pmove_t *pm` (bg_pmove.c:29) — the active per-frame movement context, set at the
/// top of `PmoveSingle` and read by the whole `PM_*` family. A `static mut` raw
/// pointer (NULL until a `Pmove` is in flight), accessed via `addr_of!`.
pub static mut pm: *mut pmove_t = null_mut();

/// `pml_t pml` (bg_pmove.c:30) — the per-`PmoveSingle` local movement scratch state
/// (frametime, basis vectors, ground trace, previous origin/velocity). Zero-init like
/// its C file-scope definition; `pml_t` is pointer-free (ported in `bg_local_h.rs`).
pub static mut pml: pml_t = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

/// `bgEntity_t *pm_entSelf` (bg_pmove.c:32) — the entity being moved this frame.
pub static mut pm_entSelf: *mut bgEntity_t = null_mut();
/// `bgEntity_t *pm_entVeh` (bg_pmove.c:33) — the vehicle the mover is riding, if any.
pub static mut pm_entVeh: *mut bgEntity_t = null_mut();

/// `gPMDoSlowFall` (bg_pmove.c) — set each `PmoveSingle` from `PM_DoSlowFall()`.
pub static mut gPMDoSlowFall: qboolean = QFALSE;

/// `pm_cancelOutZoom` (bg_pmove.c) — toggled by the weapon-zoom code.
pub static mut pm_cancelOutZoom: qboolean = QFALSE;

// movement parameters
/// `pm_stopspeed` (bg_pmove.c).
pub static pm_stopspeed: f32 = 100.0;
/// `pm_duckScale` (bg_pmove.c).
pub static pm_duckScale: f32 = 0.50;
/// `pm_swimScale` (bg_pmove.c).
pub static pm_swimScale: f32 = 0.50;
/// `pm_wadeScale` (bg_pmove.c).
pub static pm_wadeScale: f32 = 0.70;

/// `pm_vehicleaccelerate` (bg_pmove.c).
pub static pm_vehicleaccelerate: f32 = 36.0;
/// `pm_accelerate` (bg_pmove.c).
pub static pm_accelerate: f32 = 10.0;
/// `pm_airaccelerate` (bg_pmove.c).
pub static pm_airaccelerate: f32 = 1.0;
/// `pm_wateraccelerate` (bg_pmove.c).
pub static pm_wateraccelerate: f32 = 4.0;
/// `pm_flyaccelerate` (bg_pmove.c).
pub static pm_flyaccelerate: f32 = 8.0;

/// `pm_friction` (bg_pmove.c).
pub static pm_friction: f32 = 6.0;
/// `pm_waterfriction` (bg_pmove.c).
pub static pm_waterfriction: f32 = 1.0;
/// `pm_flightfriction` (bg_pmove.c).
pub static pm_flightfriction: f32 = 3.0;
/// `pm_spectatorfriction` (bg_pmove.c).
pub static pm_spectatorfriction: f32 = 5.0;

/// `c_pmove` (bg_pmove.c) — `PmoveSingle` frame counter (`c_pmove++`).
pub static mut c_pmove: c_int = 0;

/// `forceSpeedLevels[4]` (bg_pmove.c) — FP_SPEED rate multiplier per rank.
pub static forceSpeedLevels: [f32; 4] = [
    1.0, //rank 0?
    1.25, 1.5, 1.75,
];

/// `forcePowerNeeded[NUM_FORCE_POWER_LEVELS][NUM_FORCE_POWERS]` (bg_pmove.c) — the
/// per-(rank, power) force-point cost matrix; rows are FORCE_LEVEL_0..3, columns the
/// `FP_*` powers in enum order. (999 == effectively unusable at that rank.)
#[rustfmt::skip]
pub static forcePowerNeeded: [[c_int; NUM_FORCE_POWERS]; NUM_FORCE_POWER_LEVELS] = [
    [ //nothing should be usable at rank 0..
        999,//FP_HEAL,//instant
        999,//FP_LEVITATION,//hold/duration
        999,//FP_SPEED,//duration
        999,//FP_PUSH,//hold/duration
        999,//FP_PULL,//hold/duration
        999,//FP_TELEPATHY,//instant
        999,//FP_GRIP,//hold/duration
        999,//FP_LIGHTNING,//hold/duration
        999,//FP_RAGE,//duration
        999,//FP_PROTECT,//duration
        999,//FP_ABSORB,//duration
        999,//FP_TEAM_HEAL,//instant
        999,//FP_TEAM_FORCE,//instant
        999,//FP_DRAIN,//hold/duration
        999,//FP_SEE,//duration
        999,//FP_SABER_OFFENSE,
        999,//FP_SABER_DEFENSE,
        999,//FP_SABERTHROW,
        //NUM_FORCE_POWERS
    ],
    [
        65,//FP_HEAL,//instant //was 25, but that was way too little
        10,//FP_LEVITATION,//hold/duration
        50,//FP_SPEED,//duration
        20,//FP_PUSH,//hold/duration
        20,//FP_PULL,//hold/duration
        20,//FP_TELEPATHY,//instant
        30,//FP_GRIP,//hold/duration
        1,//FP_LIGHTNING,//hold/duration
        50,//FP_RAGE,//duration
        50,//FP_PROTECT,//duration
        50,//FP_ABSORB,//duration
        50,//FP_TEAM_HEAL,//instant
        50,//FP_TEAM_FORCE,//instant
        20,//FP_DRAIN,//hold/duration
        20,//FP_SEE,//duration
        0,//FP_SABER_OFFENSE,
        2,//FP_SABER_DEFENSE,
        20,//FP_SABERTHROW,
        //NUM_FORCE_POWERS
    ],
    [
        60,//FP_HEAL,//instant
        10,//FP_LEVITATION,//hold/duration
        50,//FP_SPEED,//duration
        20,//FP_PUSH,//hold/duration
        20,//FP_PULL,//hold/duration
        20,//FP_TELEPATHY,//instant
        30,//FP_GRIP,//hold/duration
        1,//FP_LIGHTNING,//hold/duration
        50,//FP_RAGE,//duration
        25,//FP_PROTECT,//duration
        25,//FP_ABSORB,//duration
        33,//FP_TEAM_HEAL,//instant
        33,//FP_TEAM_FORCE,//instant
        20,//FP_DRAIN,//hold/duration
        20,//FP_SEE,//duration
        0,//FP_SABER_OFFENSE,
        1,//FP_SABER_DEFENSE,
        20,//FP_SABERTHROW,
        //NUM_FORCE_POWERS
    ],
    [
        50,//FP_HEAL,//instant //You get 5 points of health.. for 50 force points!
        10,//FP_LEVITATION,//hold/duration
        50,//FP_SPEED,//duration
        20,//FP_PUSH,//hold/duration
        20,//FP_PULL,//hold/duration
        20,//FP_TELEPATHY,//instant
        60,//FP_GRIP,//hold/duration
        1,//FP_LIGHTNING,//hold/duration
        50,//FP_RAGE,//duration
        10,//FP_PROTECT,//duration
        10,//FP_ABSORB,//duration
        25,//FP_TEAM_HEAL,//instant
        25,//FP_TEAM_FORCE,//instant
        20,//FP_DRAIN,//hold/duration
        20,//FP_SEE,//duration
        0,//FP_SABER_OFFENSE,
        0,//FP_SABER_DEFENSE,
        20,//FP_SABERTHROW,
        //NUM_FORCE_POWERS
    ],
];

/// `forceJumpHeight[NUM_FORCE_POWER_LEVELS]` (bg_pmove.c).
pub static forceJumpHeight: [f32; NUM_FORCE_POWER_LEVELS] = [
    32.0,  //normal jump (+stepheight+crouchdiff = 66)
    96.0,  //(+stepheight+crouchdiff = 130)
    192.0, //(+stepheight+crouchdiff = 226)
    384.0, //(+stepheight+crouchdiff = 418)
];

/// `forceJumpStrength[NUM_FORCE_POWER_LEVELS]` (bg_pmove.c). The rank-0 entry is the
/// `JUMP_VELOCITY` define (bg_public.h), so it tracks that constant exactly.
pub static forceJumpStrength: [f32; NUM_FORCE_POWER_LEVELS] = [
    JUMP_VELOCITY as f32, //normal jump
    420.0, 590.0, 840.0,
];

// --- NOT YET PORTED: bg_pmove.c lines 56-170 (PM_INLINE/file-scope helpers above
//     PM_BGEntForNum) — none stand on their own yet.

/// `PM_BGEntForNum` (bg_pmove.c:171) — "Get a pointer to the bgEntity by the index."
/// Returns the entity at `num` in the engine's entity array, computed as
/// `(byte *)pm->baseEnt + pm->entSize*num`. The `entSize` stride lets one routine
/// index either the game's `gentity_t` array or cgame's `centity_t` array. Returns
/// NULL (with a debug assert, matching the C `assert(!"...")`) when the `pm` context
/// or its base/stride are unset.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` for the duration of the call.
pub unsafe fn PM_BGEntForNum(num: c_int) -> *mut bgEntity_t {
    let pmv = *addr_of!(pm);

    if pmv.is_null() {
        debug_assert!(
            false,
            "You cannot call PM_BGEntForNum outside of pm functions!"
        );
        return null_mut();
    }

    if (*pmv).baseEnt.is_null() {
        debug_assert!(false, "Base entity address not set");
        return null_mut();
    }

    if (*pmv).entSize == 0 {
        debug_assert!(false, "sizeof(ent) is 0, impossible (not set?)");
        return null_mut();
    }

    debug_assert!(num >= 0 && (num as usize) < MAX_GENTITIES);

    ((*pmv).baseEnt as *mut u8).add(((*pmv).entSize * num) as usize) as *mut bgEntity_t
}

/// `BG_SabersOff` (bg_pmove.c:200) — true when the player's saber(s) are fully
/// holstered. For dual/staff styles, "off" needs `saberHolstered >= 2` (both blades
/// stowed), so a half-holstered (`< 2`) dual/staff is still considered on.
///
/// # Safety
/// `ps` must point to a valid `playerState_t`.
pub unsafe fn BG_SabersOff(ps: *mut playerState_t) -> qboolean {
    if (*ps).saberHolstered == 0 {
        return QFALSE;
    }
    if (*ps).fd.saberAnimLevelBase == SS_DUAL || (*ps).fd.saberAnimLevelBase == SS_STAFF {
        if (*ps).saberHolstered < 2 {
            return QFALSE;
        }
    }
    QTRUE
}

/// `BG_KnockDownable` (bg_pmove.c:217) — whether the player can be knocked down right
/// now. Riding a vehicle or manning an emplaced gun / e-web makes them immune.
///
/// # Safety
/// `ps` may be NULL (handled); otherwise it must point to a valid `playerState_t`.
pub unsafe fn BG_KnockDownable(ps: *mut playerState_t) -> qboolean {
    if ps.is_null() {
        // just for safety
        return QFALSE;
    }

    if (*ps).m_iVehicleNum != 0 {
        // riding a vehicle, don't knock me down
        return QFALSE;
    }

    if (*ps).emplacedIndex != 0 {
        // using emplaced gun or eweb, can't be knocked down
        return QFALSE;
    }

    // ok, I guess?
    QTRUE
}

// I should probably just do a global inline sometime.
// (C: `#define PM_INLINE ID_INLINE` (or none under __LCC__); Rust has no inline hint
// to carry here.)

/// `PM_IsRocketTrooper` (bg_pmove.c:246) — "hacky assumption check, assume any client
/// non-humanoid is a rocket trooper." The actual test is `#if 0`'d out in JKA, so this
/// always returns `qfalse`; the disabled logic is carried verbatim as a comment.
pub fn PM_IsRocketTrooper() -> qboolean {
    /*
    if (pm->ps->clientNum < MAX_CLIENTS &&
        pm->gametype == GT_SIEGE &&
        pm->nonHumanoid)
    {
        return qtrue;
    }
    */

    QFALSE
}

/// `PM_GetSaberStance` (bg_pmove.c:260) — the idle-stance animation for the active
/// saber style / force rank. Falls back to `BOTH_STAND1` when the saber was lost
/// (`saberEntityNum == 0`) or is holstered.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a valid `ps`.
pub unsafe fn PM_GetSaberStance() -> c_int {
    let pmv = *addr_of!(pm);

    // C: `int anim = BOTH_STAND2;` — the switch reassigns on every path, so folded into
    // the trailing match expression below (no dead initializer).
    let anim: c_int;
    let saber1: *mut saberInfo_t = BG_MySaber((*(*pmv).ps).clientNum, 0);
    let saber2: *mut saberInfo_t = BG_MySaber((*(*pmv).ps).clientNum, 1);

    if (*(*pmv).ps).saberEntityNum == 0 {
        //lost it
        return BOTH_STAND1;
    }

    if BG_SabersOff((*pmv).ps) != QFALSE {
        return BOTH_STAND1;
    }

    if !saber1.is_null() && (*saber1).readyAnim != -1 {
        return (*saber1).readyAnim;
    }

    if !saber2.is_null() && (*saber2).readyAnim != -1 {
        return (*saber2).readyAnim;
    }

    if !saber1.is_null() && !saber2.is_null() && (*(*pmv).ps).saberHolstered == 0 {
        //dual sabers, both on
        return BOTH_SABERDUAL_STANCE;
    }

    match (*(*pmv).ps).fd.saberAnimLevel {
        SS_DUAL => {
            anim = BOTH_SABERDUAL_STANCE;
        }
        SS_STAFF => {
            anim = BOTH_SABERSTAFF_STANCE;
        }
        SS_FAST | SS_TAVION => {
            anim = BOTH_SABERFAST_STANCE;
        }
        SS_STRONG => {
            anim = BOTH_SABERSLOW_STANCE;
        }
        // SS_NONE | SS_MEDIUM | SS_DESANN | default
        _ => {
            anim = BOTH_STAND2;
        }
    }
    anim
}

/// `PM_DoSlowFall` (bg_pmove.c:299) — true during the first half-second (`legsTimer >
/// 500`) of a left/right wall-run, so the player drifts down the wall slowly.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a valid `ps`.
pub unsafe fn PM_DoSlowFall() -> qboolean {
    let pmv = *addr_of!(pm);
    if ((*(*pmv).ps).legsAnim == BOTH_WALL_RUN_RIGHT
        || (*(*pmv).ps).legsAnim == BOTH_WALL_RUN_LEFT)
        && (*(*pmv).ps).legsTimer > 500
    {
        return QTRUE;
    }

    QFALSE
}

// begin vehicle functions crudely ported from sp -rww
/*
====================================================================
void pitch_roll_for_slope (edict_t *forwhom, vec3_t *slope, vec3_t storeAngles )

MG

This will adjust the pitch and roll of a monster to match
a given slope - if a non-'0 0 0' slope is passed, it will
use that value, otherwise it will use the ground underneath
the monster.  If it doesn't find a surface, it does nothinh\g
and returns.
====================================================================
*/
/// `PM_pitch_roll_for_slope` (bg_pmove.c:324). With `pass_slope` null/zero, traces 300
/// units straight down from the entity to read the ground-plane normal; otherwise uses
/// the passed slope. The pitch/roll is written to `storeAngles` when non-null, else
/// applied to `pm->ps->viewangles` (and the bbox mins / origin nudged up).
///
/// No oracle: calls the `pm->trace` engine callback and walks the vehicle pointer chain
/// (verified by inspection; f32 arithmetic per the project precision convention).
///
/// # Safety
/// `pm` (with valid `ps`), `forwhom` and its vehicle (if `CLASS_VEHICLE`) must be valid;
/// `pass_slope` / `storeAngles` are optional `vec3_t` pointers (may be null).
pub unsafe fn PM_pitch_roll_for_slope(
    forwhom: *mut bgEntity_t,
    pass_slope: *mut vec_t,
    storeAngles: *mut vec_t,
) {
    let pmv = *addr_of!(pm);
    let mut slope: vec3_t = [0.0; 3];
    let mut nvf: vec3_t = [0.0; 3];
    let mut ovf: vec3_t = [0.0; 3];
    let mut ovr: vec3_t = [0.0; 3];
    let mut startspot: vec3_t = [0.0; 3];
    let mut endspot: vec3_t = [0.0; 3];
    let mut new_angles: vec3_t = [0.0, 0.0, 0.0];
    let pitch: f32;
    let r#mod: f32;
    let dot: f32;

    // if we don't have a slope, get one
    if pass_slope.is_null() || VectorCompare(&vec3_origin, &*(pass_slope as *const vec3_t)) != 0 {
        let mut trace: trace_t = core::mem::zeroed();

        VectorCopy(&(*(*pmv).ps).origin, &mut startspot);
        startspot[2] += (*pmv).mins[2] + 4.0;
        VectorCopy(&startspot, &mut endspot);
        endspot[2] -= 300.0;
        ((*pmv).trace.unwrap())(
            &mut trace,
            (*(*pmv).ps).origin.as_ptr(),
            vec3_origin.as_ptr(),
            vec3_origin.as_ptr(),
            endspot.as_ptr(),
            (*forwhom).s.number,
            MASK_SOLID,
        );
        //		if(trace_fraction>0.05&&forwhom.movetype==MOVETYPE_STEP)
        //			forwhom.flags(-)FL_ONGROUND;

        if trace.fraction >= 1.0 {
            return;
        }

        // C: `if( !( &trace.plane ) ) return;` — the address of a struct field is never
        // null, so this guard can never fire; carried faithfully as a no-op comment.

        if VectorCompare(&vec3_origin, &trace.plane.normal) != 0 {
            return;
        }

        VectorCopy(&trace.plane.normal, &mut slope);
    } else {
        VectorCopy(&*(pass_slope as *const vec3_t), &mut slope);
    }

    if (*forwhom).s.NPC_class == CLASS_VEHICLE {
        // special code for vehicles
        let pVeh: *mut Vehicle_t = (*forwhom).m_pVehicle;
        let mut tempAngles: vec3_t = [0.0; 3];

        tempAngles[PITCH as usize] = 0.0;
        tempAngles[ROLL as usize] = 0.0;
        tempAngles[YAW as usize] = *(*pVeh).m_vOrientation.add(YAW as usize);
        AngleVectors(&tempAngles, Some(&mut ovf), Some(&mut ovr), None);
    } else {
        AngleVectors(&(*(*pmv).ps).viewangles, Some(&mut ovf), Some(&mut ovr), None);
    }

    vectoangles(&slope, &mut new_angles);
    pitch = new_angles[PITCH as usize] + 90.0;
    new_angles[ROLL as usize] = 0.0;
    new_angles[PITCH as usize] = 0.0;

    AngleVectors(&new_angles, Some(&mut nvf), None, None);

    r#mod = if DotProduct(&nvf, &ovr) < 0.0 { -1.0 } else { 1.0 };

    dot = DotProduct(&nvf, &ovf);

    if !storeAngles.is_null() {
        *storeAngles.add(PITCH as usize) = dot * pitch;
        *storeAngles.add(ROLL as usize) = (1.0 - Q_fabs(dot)) * pitch * r#mod;
    } else
    /* if ( forwhom->client ) */
    {
        let oldmins2: f32;

        (*(*pmv).ps).viewangles[PITCH as usize] = dot * pitch;
        (*(*pmv).ps).viewangles[ROLL as usize] = (1.0 - Q_fabs(dot)) * pitch * r#mod;
        oldmins2 = (*pmv).mins[2];
        (*pmv).mins[2] = -24.0 + 12.0 * (*(*pmv).ps).viewangles[PITCH as usize].abs() / 180.0;
        // FIXME: if it gets bigger, move up
        if oldmins2 > (*pmv).mins[2] {
            // our mins is now lower, need to move up
            // FIXME: trace?
            (*(*pmv).ps).origin[2] += oldmins2 - (*pmv).mins[2];
            //forwhom->currentOrigin[2] = forwhom->client->ps.origin[2];
            //gi.linkentity( forwhom );
        }
    }
    /*
    else
    {
        forwhom->currentAngles[PITCH] = dot * pitch;
        forwhom->currentAngles[ROLL] = ((1-Q_fabs(dot)) * pitch * mod);
    }
    */
}

// pm_flying classes (bg_pmove.c:421-424).
const FLY_NONE: c_int = 0;
const FLY_NORMAL: c_int = 1;
const FLY_VEHICLE: c_int = 2;
const FLY_HOVER: c_int = 3;
/// `static int pm_flying` (bg_pmove.c:424) — the current flight mode, set by
/// [`PM_SetSpecialMoveValues`] and read by the move dispatch.
static mut pm_flying: c_int = FLY_NONE;

/// `PM_SetSpecialMoveValues` (bg_pmove.c:425) — classify the mover's flight mode
/// (`pm_flying`): real players never fly; an NPC flies normally if `EF2_FLYING`, else a
/// `CLASS_VEHICLE` flies as a fighter (`VH_FIGHTER`) or hovers (`hoverHeight > 0`).
///
/// # Safety
/// `pm` (with valid `ps`) must be valid; `pm_entSelf`, if non-null, must be valid.
pub unsafe fn PM_SetSpecialMoveValues() {
    let pmv = *addr_of!(pm);
    let pEnt: *mut bgEntity_t;

    if (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int {
        // we know that real players aren't vehs
        *addr_of_mut!(pm_flying) = FLY_NONE;
        return;
    }

    // default until we decide otherwise
    *addr_of_mut!(pm_flying) = FLY_NONE;

    pEnt = *addr_of!(pm_entSelf);

    if !pEnt.is_null() {
        if ((*(*pmv).ps).eFlags2 & EF2_FLYING) != 0 {
            // pm->gent->client->moveType == MT_FLYSWIM
            *addr_of_mut!(pm_flying) = FLY_NORMAL;
        } else if (*pEnt).s.NPC_class == CLASS_VEHICLE {
            if (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER {
                *addr_of_mut!(pm_flying) = FLY_VEHICLE;
            } else if (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).hoverHeight > 0.0 {
                *addr_of_mut!(pm_flying) = FLY_HOVER;
            }
        }
    }
}

/// `PM_SetVehicleAngles` (bg_pmove.c:460, `static` in C) — bank/pitch the ridden
/// vehicle toward the target angles derived from water level / ground slope / view,
/// then step `m_vOrientation` toward them by `vehicleBankingSpeed`. Early-outs unless
/// the mover is a banking `CLASS_VEHICLE`.
///
/// No oracle: deep vehicle pointer chain + the `pm->trace` callback via
/// `PM_pitch_roll_for_slope` (verified by inspection; f32 arithmetic, `sin` via f64 per
/// the `AngleVectors` convention).
///
/// # Safety
/// `pm`/`pml`/`pm_entSelf` and the vehicle chain must be valid; `normal` is an optional
/// ground-normal `vec3_t` pointer (may be null).
pub unsafe fn PM_SetVehicleAngles(normal: *mut vec_t) {
    let pmv = *addr_of!(pm);
    let pEnt: *mut bgEntity_t = *addr_of!(pm_entSelf);
    let pVeh: *mut Vehicle_t;
    let mut vAngles: vec3_t = [0.0; 3];
    let mut vehicleBankingSpeed: f32;
    let pitchBias: f32;
    let mut i: c_int;

    if pEnt.is_null() || (*pEnt).s.NPC_class != CLASS_VEHICLE {
        return;
    }

    pVeh = (*pEnt).m_pVehicle;

    //float	curVehicleBankingSpeed;
    let vinfo = (*pVeh).m_pVehicleInfo;
    vehicleBankingSpeed = ((*vinfo).bankingSpeed * 32.0) * (*addr_of!(pml)).frametime; //0.25f

    if vehicleBankingSpeed <= 0.0
        || ((*vinfo).pitchLimit == 0.0 && (*vinfo).rollLimit == 0.0)
    {
        // don't bother, this vehicle doesn't bank
        return;
    }
    //FIXME: do 3 traces to define a plane and use that... smoothes it out some, too...
    //pitch_roll_for_slope( pm->gent, normal, vAngles );
    //FIXME: maybe have some pitch control in water and/or air?

    if (*vinfo).r#type == VH_FIGHTER {
        pitchBias = 0.0;
    } else {
        //FIXME: gravity does not matter in SPACE!!!
        //center of gravity affects pitch in air/water (FIXME: what about roll?)
        pitchBias = 90.0 * (*vinfo).centerOfGravity[0]; //if centerOfGravity is all the way back (-1.0f), vehicle pitches up 90 degrees when in air
    }

    VectorClear(&mut vAngles);
    if (*pmv).waterlevel > 0 {
        // in water
        // view pitch has some influence when in water
        // FIXME: take center of gravity into account?
        vAngles[PITCH as usize] +=
            ((*(*pmv).ps).viewangles[PITCH as usize] - vAngles[PITCH as usize]) * 0.75
                + (pitchBias * 0.5);
    } else if !normal.is_null() {
        // have a valid surface below me
        PM_pitch_roll_for_slope(pEnt, normal, vAngles.as_mut_ptr());
        if ((*addr_of!(pml)).groundTrace.contents & (CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA))
            != 0
        {
            // on water
            // view pitch has some influence when on a fluid surface
            // FIXME: take center of gravity into account
            vAngles[PITCH as usize] +=
                ((*(*pmv).ps).viewangles[PITCH as usize] - vAngles[PITCH as usize]) * 0.5
                    + (pitchBias * 0.5);
        }
    } else {
        // in air, let pitch match view...?
        // FIXME: take center of gravity into account
        vAngles[PITCH as usize] = (*(*pmv).ps).viewangles[PITCH as usize] * 0.5 + pitchBias;
        // don't bank so fast when in the air
        vehicleBankingSpeed *= 0.125 * (*addr_of!(pml)).frametime;
    }
    //NOTE: if angles are flat and we're moving through air (not on ground),
    //		then pitch/bank?
    if (*vinfo).rollLimit > 0.0 {
        // roll when banking
        let mut velocity: vec3_t = [0.0; 3];
        let mut speed: f32;
        VectorCopy(&(*(*pmv).ps).velocity, &mut velocity);
        velocity[2] = 0.0;
        speed = VectorNormalize(&mut velocity);
        if speed > 32.0 || speed < -32.0 {
            let mut rt: vec3_t = [0.0; 3];
            let mut tempVAngles: vec3_t = [0.0; 3];
            let side: f32;
            let dp: f32;

            // Magic number fun!  Speed is used for banking, so modulate the speed by a sine wave
            //FIXME: this banks too early
            speed *= ((150.0 + (*addr_of!(pml)).frametime) as f64 * 0.003).sin() as f32;

            // Clamp to prevent harsh rolling
            if speed > 60.0 {
                speed = 60.0;
            }

            VectorCopy(&*((*pVeh).m_vOrientation as *const vec3_t), &mut tempVAngles);
            tempVAngles[ROLL as usize] = 0.0;
            AngleVectors(&tempVAngles, None, Some(&mut rt), None);
            dp = DotProduct(&velocity, &rt);
            side = speed * dp;
            vAngles[ROLL as usize] -= side;
        }
    }

    //cap
    if (*vinfo).pitchLimit != -1.0 {
        if vAngles[PITCH as usize] > (*vinfo).pitchLimit {
            vAngles[PITCH as usize] = (*vinfo).pitchLimit;
        } else if vAngles[PITCH as usize] < -(*vinfo).pitchLimit {
            vAngles[PITCH as usize] = -(*vinfo).pitchLimit;
        }
    }

    if vAngles[ROLL as usize] > (*vinfo).rollLimit {
        vAngles[ROLL as usize] = (*vinfo).rollLimit;
    } else if vAngles[ROLL as usize] < -(*vinfo).rollLimit {
        vAngles[ROLL as usize] = -(*vinfo).rollLimit;
    }

    //do it
    i = 0;
    while i < 3 {
        if i == YAW as c_int {
            // yawing done elsewhere
            i += 1;
            continue;
        }
        //bank faster the higher the difference is
        /*
        else if ( i == PITCH )
        {
            curVehicleBankingSpeed = vehicleBankingSpeed*fabs(AngleNormalize180(AngleSubtract( vAngles[PITCH], pVeh->m_vOrientation[PITCH] )))/(g_vehicleInfo[pm->ps->vehicleIndex].pitchLimit/2.0f);
        }
        else if ( i == ROLL )
        {
            curVehicleBankingSpeed = vehicleBankingSpeed*fabs(AngleNormalize180(AngleSubtract( vAngles[ROLL], pVeh->m_vOrientation[ROLL] )))/(g_vehicleInfo[pm->ps->vehicleIndex].rollLimit/2.0f);
        }

        if ( curVehicleBankingSpeed )
        */
        {
            let orient = (*pVeh).m_vOrientation.add(i as usize);
            if *orient >= vAngles[i as usize] + vehicleBankingSpeed {
                *orient -= vehicleBankingSpeed;
            } else if *orient <= vAngles[i as usize] - vehicleBankingSpeed {
                *orient += vehicleBankingSpeed;
            } else {
                *orient = vAngles[i as usize];
            }
        }
        i += 1;
    }
}

// #ifndef QAGAME extern vmCvar_t cg_paused; #endif -- cgame-only, excluded in the
// QAGAME (game module) build; carried as a comment.

/// `BG_ExternThisSoICanRecompileInDebug` (bg_pmove.c:619) — the entire body is `#if 0`'d
/// out in JKA (a debug-recompile stub), so this is a no-op. The disabled rider-banking
/// logic is carried verbatim below.
///
/// # Safety
/// Trivially safe (empty body); the raw pointers are unused.
pub unsafe fn BG_ExternThisSoICanRecompileInDebug(_pVeh: *mut Vehicle_t, _riderPS: *mut playerState_t) {
    /*
    float pitchSubtract, pitchDelta, yawDelta;
    //Com_Printf( S_COLOR_RED"PITCH: %4.2f, YAW: %4.2f, ROLL: %4.2f\n", riderPS->viewangles[0],riderPS->viewangles[1],riderPS->viewangles[2]);
    yawDelta = AngleSubtract(riderPS->viewangles[YAW],pVeh->m_vPrevRiderViewAngles[YAW]);
    #ifndef QAGAME
        if ( !cg_paused.integer )
        {
            //Com_Printf( "%d - yawDelta %4.2f\n", pm->cmd.serverTime, yawDelta );
        }
    #endif
    yawDelta *= (4.0f*pVeh->m_fTimeModifier);
    pVeh->m_vOrientation[ROLL] -= yawDelta;

    pitchDelta = AngleSubtract(riderPS->viewangles[PITCH],pVeh->m_vPrevRiderViewAngles[PITCH]);
    pitchDelta *= (2.0f*pVeh->m_fTimeModifier);
    pitchSubtract = pitchDelta * (fabs(pVeh->m_vOrientation[ROLL])/90.0f);
    pVeh->m_vOrientation[PITCH] += pitchDelta-pitchSubtract;
    if ( pVeh->m_vOrientation[ROLL] > 0 )
    {
        pVeh->m_vOrientation[YAW] += pitchSubtract;
    }
    else
    {
        pVeh->m_vOrientation[YAW] -= pitchSubtract;
    }
    pVeh->m_vOrientation[PITCH] = AngleNormalize180( pVeh->m_vOrientation[PITCH] );
    pVeh->m_vOrientation[YAW] = AngleNormalize360( pVeh->m_vOrientation[YAW] );
    pVeh->m_vOrientation[ROLL] = AngleNormalize180( pVeh->m_vOrientation[ROLL] );

    VectorCopy( riderPS->viewangles, pVeh->m_vPrevRiderViewAngles );
    */
}

/// `BG_VehicleTurnRateForSpeed` (bg_pmove.c:654) — fill `mPitchOverride`/`mYawOverride`
/// with the vehicle's mouse pitch/yaw scaled by a speed fraction (for vehicles with
/// `speedDependantTurning`, while airborne / on a steep enough surface).
///
/// No oracle: reads the vehicle-info pointer chain + writes through out-pointers
/// (verified by inspection).
///
/// # Safety
/// `pVeh` may be null (handled); otherwise it and its `m_pVehicleInfo` must be valid,
/// and `mPitchOverride`/`mYawOverride` valid `float` out-pointers.
pub unsafe fn BG_VehicleTurnRateForSpeed(
    pVeh: *mut Vehicle_t,
    speed: f32,
    mPitchOverride: *mut f32,
    mYawOverride: *mut f32,
) {
    if !pVeh.is_null() && !(*pVeh).m_pVehicleInfo.is_null() {
        let vinfo = (*pVeh).m_pVehicleInfo;
        let mut speedFrac: f32 = 1.0;
        if (*vinfo).speedDependantTurning != QFALSE {
            if (*pVeh).m_LandTrace.fraction >= 1.0
                || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE
            {
                speedFrac = speed / ((*vinfo).speedMax * 0.75);
                if speedFrac < 0.25 {
                    speedFrac = 0.25;
                } else if speedFrac > 1.0 {
                    speedFrac = 1.0;
                }
            }
        }
        if (*vinfo).mousePitch != 0.0 {
            *mPitchOverride = (*vinfo).mousePitch * speedFrac;
        }
        if (*vinfo).mouseYaw != 0.0 {
            *mYawOverride = (*vinfo).mouseYaw * speedFrac;
        }
    }
}

/// `PM_HoverTrace` (bg_pmove.c:697) — the hover-vehicle ground/water trace. In water,
/// applies bouyancy lift (with an `iWakeFX` surface splash when breaking the surface at
/// speed); otherwise traces `hoverHeight` straight down — a steep side-slope shoves us
/// off it (`-300*d` downward), a walkable surface within reach gets a `hoverStrength`
/// push up (plus a splash when hovering over a fluid). Finally banks the vehicle to the
/// ground normal and clears `VEH_FLYING`, or — airborne — sets `VEH_FLYING` and decays
/// `m_vAngularVelocity` toward zero by `frametime`. Closes with [`PM_GroundTraceMissed`].
///
/// No oracle: deep `Vehicle_t` pointer chain + the `pm->trace` engine callback + `Q_irand`
/// (verified by inspection). f32 arithmetic, with the C double-`fabs` velocity-magnitude
/// compares carried in f64 (the line-1954 precedent). The two `#ifdef QAGAME` `iWakeFX`
/// dispatches (`G_AddEvent` / `G_PlayEffectID`) are wired to their g_utils ports (this module
/// IS the QAGAME build); the gating `Q_irand` runs so the shared RNG advances bit-for-bit.
///
/// # Safety
/// `pm`/`pml`/`pm_entSelf` and the vehicle chain (when `CLASS_VEHICLE`) must be valid.
pub unsafe fn PM_HoverTrace() {
    let pmv = *addr_of!(pm);
    let pVeh: *mut Vehicle_t;
    let hoverHeight: f32;
    let mut point: vec3_t = [0.0; 3];
    let mut vAng: vec3_t = [0.0; 3];
    let mut fxAxis: [vec3_t; 3] = [[0.0; 3]; 3];
    let relativeWaterLevel: f32;

    let pEnt: *mut bgEntity_t = *addr_of!(pm_entSelf);
    if pEnt.is_null() || (*pEnt).s.NPC_class != CLASS_VEHICLE {
        return;
    }

    pVeh = (*pEnt).m_pVehicle;
    hoverHeight = (*(*pVeh).m_pVehicleInfo).hoverHeight;
    let trace: *mut trace_t = addr_of_mut!((*addr_of_mut!(pml)).groundTrace);

    (*addr_of_mut!(pml)).groundPlane = QFALSE;

    //relativeWaterLevel = (pm->ps->waterheight - (pm->ps->origin[2]+pm->mins[2]));
    relativeWaterLevel = (*pmv).waterlevel as f32; //I.. guess this works
    if (*pmv).waterlevel != 0 && relativeWaterLevel >= 0.0 {
        //in water
        if (*(*pVeh).m_pVehicleInfo).bouyancy <= 0.0 {
            //sink like a rock
        } else {
            //rise up
            //1.0f should make you float half-in, half-out of water
            let floatHeight: f32 = ((*(*pVeh).m_pVehicleInfo).bouyancy
                * (((*pmv).maxs[2] - (*pmv).mins[2]) * 0.5))
                - (hoverHeight * 0.5);
            if relativeWaterLevel > floatHeight {
                //too low, should rise up
                (*(*pmv).ps).velocity[2] +=
                    (relativeWaterLevel - floatHeight) * (*pVeh).m_fTimeModifier;
            }
        }
        //if ( pm->ps->waterheight < pm->ps->origin[2]+pm->maxs[2] )
        if (*pmv).waterlevel <= 1 {
            //part of us is sticking out of water
            if ((*(*pmv).ps).velocity[0] as f64).abs() + ((*(*pmv).ps).velocity[1] as f64).abs()
                > 100.0
            {
                //moving at a decent speed
                if Q_irand((*addr_of!(pml)).frametime as c_int, 100) >= 50 {
                    //splash
                    // `wakeOrg`/`fxAxis` are built only to feed the not-yet-ported `iWakeFX`
                    // effect call below; `wakeOrg` is in fact dead even in retail (the
                    // active QAGAME `G_AddEvent` path ignores it — only the commented-out
                    // `G_PlayEffectID` ever read it), so its stores never get read here.
                    let mut wakeOrg: vec3_t = [0.0; 3];
                    // distinct &mut into each axis slot (the borrow checker rejects three
                    // overlapping `&mut fxAxis[i]`); fxAxis[0]=up, [1]=right, [2]=forward.
                    let [up, right, forward] = &mut fxAxis;

                    vAng[PITCH as usize] = 0.0;
                    vAng[ROLL as usize] = 0.0;
                    vAng[YAW as usize] = *(*pVeh).m_vOrientation.add(YAW as usize);
                    AngleVectors(&vAng, Some(forward), Some(right), Some(up));
                    VectorCopy(&(*(*pmv).ps).origin, &mut wakeOrg);
                    //wakeOrg[2] = pm->ps->waterheight;
                    // the wakeOrg[2] stores are dead (see note above) — allow the lint.
                    #[allow(unused_assignments)]
                    if (*pmv).waterlevel >= 2 {
                        wakeOrg[2] = (*(*pmv).ps).origin[2] + 16.0;
                    } else {
                        wakeOrg[2] = (*(*pmv).ps).origin[2];
                    }
                    // #ifdef QAGAME //yeah, this is kind of crappy and makes no use of prediction whatsoever
                    if (*(*pVeh).m_pVehicleInfo).iWakeFX != 0 {
                        //G_PlayEffectID( pVeh->m_pVehicleInfo->iWakeFX, wakeOrg, fxAxis[0] );
                        //tempent use bad!
                        G_AddEvent(
                            pEnt as *mut gentity_t,
                            EV_PLAY_EFFECT_ID,
                            (*(*pVeh).m_pVehicleInfo).iWakeFX,
                        );
                    }
                    // #endif
                }
            }
        }
    } else {
        let mut traceContents: c_int;
        // C: `float minNormal = (float)MIN_WALK_NORMAL;` then immediately overwritten — dead init.
        let minNormal: f32 = (*(*pVeh).m_pVehicleInfo).maxSlope;

        point[0] = (*(*pmv).ps).origin[0];
        point[1] = (*(*pmv).ps).origin[1];
        point[2] = (*(*pmv).ps).origin[2] - hoverHeight;

        //FIXME: check for water, too?  If over water, go slower and make wave effect
        //		If *in* water, go really slow and use bouyancy stat to determine how far below surface to float

        //NOTE: if bouyancy is 2.0f or higher, you float over water like it's solid ground.
        //		if it's 1.0f, you sink halfway into water.  If it's 0, you sink...
        traceContents = (*pmv).tracemask;
        if (*(*pVeh).m_pVehicleInfo).bouyancy >= 2.0 {
            //sit on water
            traceContents |= CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA;
        }
        ((*pmv).trace.unwrap())(
            trace,
            (*(*pmv).ps).origin.as_ptr(),
            (*pmv).mins.as_ptr(),
            (*pmv).maxs.as_ptr(),
            point.as_ptr(),
            (*(*pmv).ps).clientNum,
            traceContents,
        );
        if (*trace).plane.normal[0] > 0.5
            || (*trace).plane.normal[0] < -0.5
            || (*trace).plane.normal[1] > 0.5
            || (*trace).plane.normal[1] < -0.5
        {
            //steep slanted hill, don't go up it.
            let mut d: f32 = (*trace).plane.normal[0].abs();
            let e: f32 = (*trace).plane.normal[1].abs();
            if e > d {
                d = e;
            }
            (*(*pmv).ps).velocity[2] = -300.0 * d;
        } else if (*trace).plane.normal[2] >= minNormal {
            //not a steep slope, so push us up
            if (*trace).fraction < 1.0 {
                //push up off ground
                let hoverForce: f32 = (*(*pVeh).m_pVehicleInfo).hoverStrength;
                if (*trace).fraction > 0.5 {
                    (*(*pmv).ps).velocity[2] +=
                        (1.0 - (*trace).fraction) * hoverForce * (*pVeh).m_fTimeModifier;
                } else {
                    (*(*pmv).ps).velocity[2] += (0.5 - ((*trace).fraction * (*trace).fraction))
                        * hoverForce
                        * 2.0
                        * (*pVeh).m_fTimeModifier;
                }
                if ((*trace).contents & (CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA)) != 0 {
                    //hovering on water, make a spash if moving
                    if ((*(*pmv).ps).velocity[0] as f64).abs()
                        + ((*(*pmv).ps).velocity[1] as f64).abs()
                        > 100.0
                    {
                        //moving at a decent speed
                        if Q_irand((*addr_of!(pml)).frametime as c_int, 100) >= 50 {
                            //splash
                            // fxAxis feeds only the not-yet-ported `iWakeFX` call; distinct &mut
                            // per axis (fxAxis[0]=up, [1]=right, [2]=forward).
                            let [up, right, forward] = &mut fxAxis;
                            vAng[PITCH as usize] = 0.0;
                            vAng[ROLL as usize] = 0.0;
                            vAng[YAW as usize] = *(*pVeh).m_vOrientation.add(YAW as usize);
                            AngleVectors(&vAng, Some(forward), Some(right), Some(up));
                            // #ifdef QAGAME
                            if (*(*pVeh).m_pVehicleInfo).iWakeFX != 0 {
                                G_PlayEffectID(
                                    (*(*pVeh).m_pVehicleInfo).iWakeFX,
                                    &(*trace).endpos,
                                    &fxAxis[0],
                                );
                            }
                            // #endif
                        }
                    }
                }
                (*addr_of_mut!(pml)).groundPlane = QTRUE;
            }
        }
    }
    if (*addr_of!(pml)).groundPlane != QFALSE {
        PM_SetVehicleAngles((*addr_of_mut!(pml)).groundTrace.plane.normal.as_mut_ptr());
        // We're on the ground.
        (*pVeh).m_ulFlags &= !(VEH_FLYING as c_ulong);

        (*pVeh).m_vAngularVelocity = 0.0;
    } else {
        PM_SetVehicleAngles(null_mut());
        // We're flying in the air.
        (*pVeh).m_ulFlags |= VEH_FLYING as c_ulong;
        //groundTrace

        if (*pVeh).m_vAngularVelocity == 0.0 {
            (*pVeh).m_vAngularVelocity = *(*pVeh).m_vOrientation.add(YAW as usize)
                - (*pVeh).m_vPrevOrientation[YAW as usize];
            if (*pVeh).m_vAngularVelocity < -15.0 {
                (*pVeh).m_vAngularVelocity = -15.0;
            }
            if (*pVeh).m_vAngularVelocity > 15.0 {
                (*pVeh).m_vAngularVelocity = 15.0;
            }
        }
        //pVeh->m_vAngularVelocity *= 0.95f;		// Angular Velocity Decays Over Time
        if (*pVeh).m_vAngularVelocity > 0.0 {
            (*pVeh).m_vAngularVelocity -= (*addr_of!(pml)).frametime;
            if (*pVeh).m_vAngularVelocity < 0.0 {
                (*pVeh).m_vAngularVelocity = 0.0;
            }
        } else if (*pVeh).m_vAngularVelocity < 0.0 {
            (*pVeh).m_vAngularVelocity += (*addr_of!(pml)).frametime;
            if (*pVeh).m_vAngularVelocity > 0.0 {
                (*pVeh).m_vAngularVelocity = 0.0;
            }
        }
    }
    PM_GroundTraceMissed();
}
//end vehicle functions crudely ported from sp -rww

/*
===============
PM_AddEvent
===============
*/
/// `PM_AddEvent` (bg_pmove.c:888) — queue a predictable pmove event on `pm->ps`.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` whose `ps` points to a valid `playerState_t`.
pub unsafe fn PM_AddEvent(newEvent: c_int) {
    // `pmv` is the file-scope `pm` context pointer, read via `addr_of!` (a `let`
    // cannot be named `pm` — it would shadow the static). This is the convention the
    // rest of the keystone follows for reading the `pm`/`pml` globals.
    let pmv = *addr_of!(pm);
    BG_AddPredictableEventToPlayerstate(newEvent, 0, (*pmv).ps);
}

/// `PM_AddEventWithParm` (bg_pmove.c:892) — like [`PM_AddEvent`] but with an event parm.
///
/// # Safety
/// See [`PM_AddEvent`].
pub unsafe fn PM_AddEventWithParm(newEvent: c_int, parm: c_int) {
    let pmv = *addr_of!(pm);
    BG_AddPredictableEventToPlayerstate(newEvent, parm, (*pmv).ps);
}

/*
===============
PM_AddTouchEnt
===============
*/
/// `PM_AddTouchEnt` (bg_pmove.c:902) — add `entityNum` to `pm->touchents` (skipping
/// the world entity, a full list, and duplicates already present).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_AddTouchEnt(entityNum: c_int) {
    let pmv = *addr_of!(pm); // the `pm` context pointer (see PM_AddEvent)

    if entityNum == ENTITYNUM_WORLD {
        return;
    }
    if (*pmv).numtouch == MAXTOUCH as c_int {
        return;
    }

    // see if it is already added
    let mut i = 0;
    while i < (*pmv).numtouch {
        if (*pmv).touchents[i as usize] == entityNum {
            return;
        }
        i += 1;
    }

    // add it
    (*pmv).touchents[(*pmv).numtouch as usize] = entityNum;
    (*pmv).numtouch += 1;
}

/*
==================
PM_ClipVelocity

Slide off of the impacting surface
==================
*/
/// `PM_ClipVelocity` (bg_pmove.c:932) — reflect `in` off the surface `normal` into
/// `out`, scaled by `overbounce`. Reads the `pm` global for the stuck-to-wall and
/// step-slide-fix special cases.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`; `in`/`normal`/`out` to valid `vec3_t`s.
pub unsafe fn PM_ClipVelocity(
    r#in: *mut vec_t,
    normal: *mut vec_t,
    out: *mut vec_t,
    overbounce: f32,
) {
    let pmv = *addr_of!(pm);
    let mut backoff: f32;
    let mut change: f32;
    let oldInZ: f32;

    if (*(*pmv).ps).pm_flags & PMF_STUCK_TO_WALL != 0 {
        //no sliding!
        VectorCopy(&*(r#in as *const vec3_t), &mut *(out as *mut vec3_t));
        return;
    }
    oldInZ = *r#in.add(2);

    backoff = DotProduct(&*(r#in as *const vec3_t), &*(normal as *const vec3_t));

    if backoff < 0.0 {
        backoff *= overbounce;
    } else {
        backoff /= overbounce;
    }

    for i in 0..3 {
        change = *normal.add(i) * backoff;
        *out.add(i) = *r#in.add(i) - change;
    }
    if (*pmv).stepSlideFix != 0 {
        if (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int//normal player
			&& (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE//on the ground
			&& *normal.add(2) < MIN_WALK_NORMAL
        //sliding against a steep slope
        {
            //if walking on the ground, don't slide up slopes that are too steep to walk on
            *out.add(2) = oldInZ;
        }
    }
}

/*
==================
PM_Friction

Handles both ground friction and water friction
==================
*/
/// `PM_Friction` (bg_pmove.c:976) — apply ground/water/vehicle/spectator friction to
/// `pm->ps->velocity`.
///
/// # Safety
/// `pm`/`pm_entSelf` must be valid; the vehicle pointer chain is dereferenced only
/// under the `CLASS_VEHICLE` guard, matching C.
pub unsafe fn PM_Friction() {
    let pmv = *addr_of!(pm);
    let mut vec: vec3_t = [0.0; 3];
    let vel: *mut f32;
    let speed: f32;
    let mut newspeed: f32;
    let mut control: f32;
    let mut drop: f32;
    let mut pEnt: *mut bgEntity_t = null_mut();

    vel = (*(*pmv).ps).velocity.as_mut_ptr();

    VectorCopy(&*(vel as *const vec3_t), &mut vec);
    if (*addr_of!(pml)).walking != 0 {
        vec[2] = 0.0; // ignore slope movement
    }

    speed = VectorLength(&vec);
    if speed < 1.0 {
        *vel.add(0) = 0.0;
        *vel.add(1) = 0.0; // allow sinking underwater
        if (*(*pmv).ps).pm_type == PM_SPECTATOR {
            *vel.add(2) = 0.0;
        }
        // FIXME: still have z friction underwater?
        return;
    }

    drop = 0.0;

    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int {
        pEnt = *addr_of!(pm_entSelf);
    }

    // apply ground friction, even if on ladder
    if *addr_of!(pm_flying) != FLY_VEHICLE
        && !pEnt.is_null()
        && (*pEnt).s.NPC_class == CLASS_VEHICLE
        && !(*pEnt).m_pVehicle.is_null()
        && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).r#type != VH_ANIMAL
        && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).r#type != VH_WALKER
        && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).friction != 0.0
    {
        let friction: f32 = (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).friction;
        if (*(*pmv).ps).pm_flags & PMF_TIME_KNOCKBACK == 0
        /*&& !(pm->ps->pm_flags & PMF_TIME_NOFRICTION)*/
        {
            control = if speed < pm_stopspeed { pm_stopspeed } else { speed };
            drop += control * friction * (*addr_of!(pml)).frametime;
            /*
            if ( Flying == FLY_HOVER )
            {
                if ( pm->cmd.rightmove )
                {//if turning, increase friction
                    control *= 2.0f;
                }
                if ( pm->ps->groundEntityNum < ENTITYNUM_NONE )
                {//on the ground
                    drop += control*friction*pml.frametime;
                }
                else if ( pml.groundPlane )
                {//on a slope
                    drop += control*friction*2.0f*pml.frametime;
                }
                else
                {//in air
                    drop += control*2.0f*friction*pml.frametime;
                }
            }
            */
        }
    } else if *addr_of!(pm_flying) != FLY_NORMAL && *addr_of!(pm_flying) != FLY_VEHICLE {
        // apply ground friction
        if (*pmv).waterlevel <= 1 {
            if (*addr_of!(pml)).walking != 0
                && (*addr_of!(pml)).groundTrace.surfaceFlags & SURF_SLICK == 0
            {
                // if getting knocked back, no friction
                if (*(*pmv).ps).pm_flags & PMF_TIME_KNOCKBACK == 0 {
                    control = if speed < pm_stopspeed { pm_stopspeed } else { speed };
                    drop += control * pm_friction * (*addr_of!(pml)).frametime;
                }
            }
        }
    }

    if *addr_of!(pm_flying) == FLY_VEHICLE {
        if (*(*pmv).ps).pm_flags & PMF_TIME_KNOCKBACK == 0 {
            control = speed; // < pm_stopspeed ? pm_stopspeed : speed;
            drop += control * pm_friction * (*addr_of!(pml)).frametime;
        }
    }

    // apply water friction even if just wading
    if (*pmv).waterlevel != 0 {
        drop += speed * pm_waterfriction * (*pmv).waterlevel as f32 * (*addr_of!(pml)).frametime;
    }
    // If on a client then there is no friction
    else if (*(*pmv).ps).groundEntityNum < MAX_CLIENTS as c_int {
        drop = 0.0;
    }

    if (*(*pmv).ps).pm_type == PM_SPECTATOR || (*(*pmv).ps).pm_type == PM_FLOAT {
        if (*(*pmv).ps).pm_type == PM_FLOAT {
            //almost no friction while floating
            // C promotes through `double` here (the `0.1` literal), so replicate that
            // chain in f64 to stay bit-exact against the oracle.
            drop = (drop as f64 + speed as f64 * 0.1 * (*addr_of!(pml)).frametime as f64) as f32;
        } else {
            drop += speed * pm_spectatorfriction * (*addr_of!(pml)).frametime;
        }
    }

    // scale the velocity
    newspeed = speed - drop;
    if newspeed < 0.0 {
        newspeed = 0.0;
    }
    newspeed /= speed;

    *vel.add(0) = *vel.add(0) * newspeed;
    *vel.add(1) = *vel.add(1) * newspeed;
    *vel.add(2) = *vel.add(2) * newspeed;
}

/*
==============
PM_Accelerate

Handles user intended acceleration
==============
*/
/// `PM_Accelerate` (bg_pmove.c:1111) — accelerate `pm->ps->velocity` toward `wishdir`
/// up to `wishspeed`. The siege branch uses the push-based method for clients.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`.
pub unsafe fn PM_Accelerate(wishdir: *mut vec_t, wishspeed: f32, accel: f32) {
    let pmv = *addr_of!(pm);

    if (*pmv).gametype != GT_SIEGE
        || (*(*pmv).ps).m_iVehicleNum != 0
        || (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
        || (*(*pmv).ps).pm_type != PM_NORMAL
    {
        //standard method, allows "bunnyhopping" and whatnot
        let addspeed: f32;
        let mut accelspeed: f32;
        let currentspeed: f32;

        currentspeed = DotProduct(&(*(*pmv).ps).velocity, &*(wishdir as *const vec3_t));
        addspeed = wishspeed - currentspeed;
        if addspeed <= 0.0 && (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int {
            return;
        }

        if addspeed < 0.0 {
            accelspeed = (-accel) * (*addr_of!(pml)).frametime * wishspeed;
            if accelspeed < addspeed {
                accelspeed = addspeed;
            }
        } else {
            accelspeed = accel * (*addr_of!(pml)).frametime * wishspeed;
            if accelspeed > addspeed {
                accelspeed = addspeed;
            }
        }

        for i in 0..3 {
            (*(*pmv).ps).velocity[i] += accelspeed * *wishdir.add(i);
        }
    } else {
        //use the proper way for siege
        let mut wishVelocity: vec3_t = [0.0; 3];
        let mut pushDir: vec3_t = [0.0; 3];
        let pushLen: f32;
        let mut canPush: f32;

        VectorScale(&*(wishdir as *const vec3_t), wishspeed, &mut wishVelocity);
        VectorSubtract(&wishVelocity, &(*(*pmv).ps).velocity, &mut pushDir);
        pushLen = VectorNormalize(&mut pushDir);

        canPush = accel * (*addr_of!(pml)).frametime * wishspeed;
        if canPush > pushLen {
            canPush = pushLen;
        }

        let mut out: vec3_t = [0.0; 3];
        VectorMA(&(*(*pmv).ps).velocity, canPush, &pushDir, &mut out);
        (*(*pmv).ps).velocity = out;
    }
}

/*
============
PM_CmdScale

Returns the scale factor to apply to cmd movements
This allows the clients to use axial -127 to 127 values for all directions
without getting a sqrt(2) distortion in speed.
============
*/
/// `PM_CmdScale` (bg_pmove.c:1177) — scale factor mapping the axial -127..127 command
/// movements to `pm->ps->speed` without sqrt(2) diagonal distortion. Upmove is
/// deliberately excluded (`umove = 0`).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`; `cmd` to a valid `usercmd_t`.
pub unsafe fn PM_CmdScale(cmd: *mut usercmd_t) -> f32 {
    let pmv = *addr_of!(pm);
    let mut max: c_int;
    let total: f32;
    let scale: f32;
    let umove: c_int = 0; //cmd->upmove;
                          //don't factor upmove into scaling speed

    max = ((*cmd).forwardmove as c_int).abs();
    if ((*cmd).rightmove as c_int).abs() > max {
        max = ((*cmd).rightmove as c_int).abs();
    }
    if umove.abs() > max {
        max = umove.abs();
    }
    if max == 0 {
        return 0.0;
    }

    let sum: c_int = ((*cmd).forwardmove as c_int * (*cmd).forwardmove as c_int)
        + ((*cmd).rightmove as c_int * (*cmd).rightmove as c_int)
        + (umove * umove);
    // C: `(float)(int sum)` then `sqrt` promotes to double, result truncated to float.
    total = ((sum as f32) as f64).sqrt() as f32;
    // C divides through `double` (the `127.0` literal); replicate to stay bit-exact.
    let a: f32 = (*(*pmv).ps).speed * max as f32;
    scale = (a as f64 / (127.0_f64 * total as f64)) as f32;

    scale
}

/*
================
PM_SetMovementDir

Determine the rotation of the legs reletive
to the facing dir
================
*/
/// `PM_SetMovementDir` (bg_pmove.c:1211) — classify the legs' rotation (0..7) relative
/// to the facing direction from the command's forward/right move. With no active input,
/// nudge a pure-sideways dir to its nearest diagonal so a stop isn't too crooked.
/// `static` in C → `pub` here (keystone callers + the oracle).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_SetMovementDir() {
    let pmv = *addr_of!(pm);
    if (*pmv).cmd.forwardmove != 0 || (*pmv).cmd.rightmove != 0 {
        if (*pmv).cmd.rightmove == 0 && (*pmv).cmd.forwardmove > 0 {
            (*(*pmv).ps).movementDir = 0;
        } else if (*pmv).cmd.rightmove < 0 && (*pmv).cmd.forwardmove > 0 {
            (*(*pmv).ps).movementDir = 1;
        } else if (*pmv).cmd.rightmove < 0 && (*pmv).cmd.forwardmove == 0 {
            (*(*pmv).ps).movementDir = 2;
        } else if (*pmv).cmd.rightmove < 0 && (*pmv).cmd.forwardmove < 0 {
            (*(*pmv).ps).movementDir = 3;
        } else if (*pmv).cmd.rightmove == 0 && (*pmv).cmd.forwardmove < 0 {
            (*(*pmv).ps).movementDir = 4;
        } else if (*pmv).cmd.rightmove > 0 && (*pmv).cmd.forwardmove < 0 {
            (*(*pmv).ps).movementDir = 5;
        } else if (*pmv).cmd.rightmove > 0 && (*pmv).cmd.forwardmove == 0 {
            (*(*pmv).ps).movementDir = 6;
        } else if (*pmv).cmd.rightmove > 0 && (*pmv).cmd.forwardmove > 0 {
            (*(*pmv).ps).movementDir = 7;
        }
    } else {
        // if they aren't actively going directly sideways,
        // change the animation to the diagonal so they
        // don't stop too crooked
        if (*(*pmv).ps).movementDir == 2 {
            (*(*pmv).ps).movementDir = 1;
        } else if (*(*pmv).ps).movementDir == 6 {
            (*(*pmv).ps).movementDir = 7;
        }
    }
}

/// `PM_ForceJumpingUp` (bg_pmove.c:1244) — are we still rising under a held force jump?
/// A chain of disqualifiers (let-go-of-jump, special/saber anims, ysalamiri, can't-use-FP),
/// then `qtrue` only if airborne, jump still held, level-1+ levitation, and rising. No
/// oracle: pure predicate over `pm`/`pm->ps` whose every sub-call is independently
/// oracle-verified (the keystone-helper convention).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_ForceJumpingUp() -> qboolean {
    let pmv = *addr_of!(pm);
    if ((*(*pmv).ps).fd.forcePowersActive & (1 << FP_LEVITATION)) == 0
        && (*(*pmv).ps).fd.forceJumpCharge != 0.0
    {
        //already jumped and let go
        return QFALSE;
    }

    if BG_InSpecialJump((*(*pmv).ps).legsAnim) != QFALSE {
        return QFALSE;
    }

    if BG_SaberInSpecial((*(*pmv).ps).saberMove) != QFALSE {
        return QFALSE;
    }

    if BG_SaberInSpecialAttack((*(*pmv).ps).legsAnim) != QFALSE {
        return QFALSE;
    }

    if BG_HasYsalamiri((*pmv).gametype, (*pmv).ps) != QFALSE {
        return QFALSE;
    }

    if BG_CanUseFPNow((*pmv).gametype, (*pmv).ps, (*pmv).cmd.serverTime, FP_LEVITATION) == QFALSE {
        return QFALSE;
    }

    if (*(*pmv).ps).groundEntityNum == ENTITYNUM_NONE && //in air
        ((*(*pmv).ps).pm_flags & PMF_JUMP_HELD) != 0 && //jumped
        (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_0 && //force-jump capable
        (*(*pmv).ps).velocity[2] > 0.0
    //going up
    {
        return QTRUE;
    }
    QFALSE
}

/// `PM_JumpForDir` (bg_pmove.c:1286) — pick the directional jump-start legs anim from the
/// command move axes, maintaining the `PMF_BACKWARDS_JUMP` flag, and play it (unless dead).
/// No oracle: trivial branch select feeding the oracle-verified `PM_SetAnim`.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_JumpForDir() {
    let pmv = *addr_of!(pm);
    let anim;
    if (*pmv).cmd.forwardmove > 0 {
        anim = BOTH_JUMP1;
        (*(*pmv).ps).pm_flags &= !PMF_BACKWARDS_JUMP;
    } else if (*pmv).cmd.forwardmove < 0 {
        anim = BOTH_JUMPBACK1;
        (*(*pmv).ps).pm_flags |= PMF_BACKWARDS_JUMP;
    } else if (*pmv).cmd.rightmove > 0 {
        anim = BOTH_JUMPRIGHT1;
        (*(*pmv).ps).pm_flags &= !PMF_BACKWARDS_JUMP;
    } else if (*pmv).cmd.rightmove < 0 {
        anim = BOTH_JUMPLEFT1;
        (*(*pmv).ps).pm_flags &= !PMF_BACKWARDS_JUMP;
    } else {
        anim = BOTH_JUMP1;
        (*(*pmv).ps).pm_flags &= !PMF_BACKWARDS_JUMP;
    }
    if BG_InDeathAnim((*(*pmv).ps).legsAnim) == QFALSE {
        PM_SetAnim(SETANIM_LEGS, anim, SETANIM_FLAG_OVERRIDE, 100);
    }
}

/// `PM_SetPMViewAngle` (bg_pmove.c:1320) — store `angle` as the player's viewangles and
/// back out the matching `delta_angles` (the engine adds delta_angles to the command
/// angles to recover the view direction). Takes its `ps`/`ucmd` explicitly (not the
/// `pm` global). `ANGLE2SHORT` truncates float→int, hence the oracle.
///
/// # Safety
/// `ps`/`ucmd` must point to valid structs; `angle` to three floats.
pub unsafe fn PM_SetPMViewAngle(ps: *mut playerState_t, angle: *mut vec_t, ucmd: *mut usercmd_t) {
    for i in 0..3 {
        // set the delta angle
        let cmdAngle: c_int = ANGLE2SHORT(*angle.add(i));
        (*ps).delta_angles[i] = cmdAngle - (*ucmd).angles[i];
    }
    VectorCopy(&*(angle as *const vec3_t), &mut (*ps).viewangles);
}

/// `PM_AdjustAngleForWallRun` (bg_pmove.c:1334) — while wall-running left/right, trace to
/// the wall and, if it's still there, pin the view perpendicular to it, drive the command
/// sideways, and (when `doMove`) shove velocity along the wall; otherwise play the stop
/// anim. `static` in C → `pub` here (the `PmoveSingle` caller). No oracle: trace/anim-
/// callback-driven, like the move-mode cluster — verified transitively through `PmoveSingle`.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`; `ps`/`ucmd` to valid structs.
pub unsafe fn PM_AdjustAngleForWallRun(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
    doMove: qboolean,
) -> qboolean {
    let pmv = *addr_of!(pm);
    if ((*ps).legsAnim == BOTH_WALL_RUN_RIGHT || (*ps).legsAnim == BOTH_WALL_RUN_LEFT)
        && (*ps).legsTimer > 500
    {
        // wall-running and not at end of anim
        // stick to wall, if there is one
        let mut fwd: vec3_t = [0.0; 3];
        let mut rt: vec3_t = [0.0; 3];
        let mut traceTo: vec3_t = [0.0; 3];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut fwdAngles: vec3_t = [0.0; 3];
        let mut trace: trace_t = core::mem::zeroed();
        let dist: f32;
        let yawAdjust: f32;

        VectorSet(&mut mins, -15.0, -15.0, 0.0);
        VectorSet(&mut maxs, 15.0, 15.0, 24.0);
        VectorSet(&mut fwdAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

        AngleVectors(&fwdAngles, Some(&mut fwd), Some(&mut rt), None);
        if (*ps).legsAnim == BOTH_WALL_RUN_RIGHT {
            dist = 128.0;
            yawAdjust = -90.0;
        } else {
            dist = -128.0;
            yawAdjust = 90.0;
        }
        VectorMA(&(*ps).origin, dist, &rt, &mut traceTo);

        ((*pmv).trace.unwrap())(
            &mut trace,
            (*ps).origin.as_ptr(),
            mins.as_ptr(),
            maxs.as_ptr(),
            traceTo.as_ptr(),
            (*ps).clientNum,
            MASK_PLAYERSOLID,
        );

        if trace.fraction < 1.0 && (trace.plane.normal[2] >= 0.0 && trace.plane.normal[2] <= 0.4) {
            //&& ent->client->ps.groundEntityNum == ENTITYNUM_NONE )
            let mut trace2: trace_t = core::mem::zeroed();
            let mut traceTo2: vec3_t = [0.0; 3];
            let mut wallRunFwd: vec3_t = [0.0; 3];
            let mut wallRunAngles: vec3_t = [0.0; 3];

            VectorClear(&mut wallRunAngles);
            wallRunAngles[YAW] = vectoyaw(&trace.plane.normal) + yawAdjust;
            AngleVectors(&wallRunAngles, Some(&mut wallRunFwd), None, None);

            VectorMA(&(*(*pmv).ps).origin, 32.0, &wallRunFwd, &mut traceTo2);
            ((*pmv).trace.unwrap())(
                &mut trace2,
                (*(*pmv).ps).origin.as_ptr(),
                mins.as_ptr(),
                maxs.as_ptr(),
                traceTo2.as_ptr(),
                (*(*pmv).ps).clientNum,
                MASK_PLAYERSOLID,
            );
            if trace2.fraction < 1.0 && DotProduct(&trace2.plane.normal, &wallRunFwd) <= -0.999 {
                // wall we can't run on in front of us
                trace.fraction = 1.0; // just a way to get it to kick us off the wall below
            }
        }

        if trace.fraction < 1.0
            && (trace.plane.normal[2] >= 0.0 && trace.plane.normal[2] <= 0.4/*MAX_WALL_RUN_Z_NORMAL*/)
        {
            // still a wall there
            if (*ps).legsAnim == BOTH_WALL_RUN_RIGHT {
                (*ucmd).rightmove = 127;
            } else {
                (*ucmd).rightmove = -127;
            }
            if (*ucmd).upmove < 0 {
                (*ucmd).upmove = 0;
            }
            // make me face perpendicular to the wall
            (*ps).viewangles[YAW] = vectoyaw(&trace.plane.normal) + yawAdjust;

            PM_SetPMViewAngle(ps, (*ps).viewangles.as_mut_ptr(), ucmd);

            (*ucmd).angles[YAW] = ANGLE2SHORT((*ps).viewangles[YAW]) - (*ps).delta_angles[YAW];
            if doMove == QTRUE {
                // push me forward
                let zVel: f32 = (*ps).velocity[2];
                if (*ps).legsTimer > 500 {
                    // not at end of anim yet
                    let mut speed: f32 = 175.0;
                    if (*ucmd).forwardmove < 0 {
                        // slower
                        speed = 100.0;
                    } else if (*ucmd).forwardmove > 0 {
                        speed = 250.0; // running speed
                    }
                    VectorScale(&fwd, speed, &mut (*ps).velocity);
                }
                (*ps).velocity[2] = zVel; // preserve z velocity
                                          // pull me toward the wall, too
                VectorMA(&(*ps).velocity, dist, &rt, &mut (*ps).velocity);
            }
            (*ucmd).forwardmove = 0;
            return QTRUE;
        } else if doMove == QTRUE {
            // stop it
            if (*ps).legsAnim == BOTH_WALL_RUN_RIGHT {
                PM_SetAnim(
                    SETANIM_BOTH,
                    BOTH_WALL_RUN_RIGHT_STOP,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            } else if (*ps).legsAnim == BOTH_WALL_RUN_LEFT {
                PM_SetAnim(
                    SETANIM_BOTH,
                    BOTH_WALL_RUN_LEFT_STOP,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            }
        }
    }

    QFALSE
}

/// `PM_AdjustAnglesForWallRunUpFlipAlt` (bg_pmove.c:1442) — aim the command view at the
/// current viewangles via the already-verified `PM_SetPMViewAngle`; always returns
/// `qtrue`. (The two commented-out per-axis ANGLE2SHORT lines are carried from C.)
/// No oracle: pure delegation, no arithmetic of its own.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`; `ucmd` to a valid `usercmd_t`.
pub unsafe fn PM_AdjustAnglesForWallRunUpFlipAlt(ucmd: *mut usercmd_t) -> qboolean {
    let pmv = *addr_of!(pm);
    //	ucmd->angles[PITCH] = ANGLE2SHORT( pm->ps->viewangles[PITCH] ) - pm->ps->delta_angles[PITCH];
    //	ucmd->angles[YAW] = ANGLE2SHORT( pm->ps->viewangles[YAW] ) - pm->ps->delta_angles[YAW];
    PM_SetPMViewAngle((*pmv).ps, (*(*pmv).ps).viewangles.as_mut_ptr(), ucmd);
    QTRUE
}

/// `PM_AdjustAngleForWallRunUp` (bg_pmove.c:1450) — while running up a wall
/// (`BOTH_FORCEWALLRUNFLIP_START`), trace forward: if there's a standable floor at the
/// top do the alt-flip, otherwise keep climbing (face the wall, push up) until a ceiling
/// or the anim end kicks us off. `static` in C → `pub` here (the `PmoveSingle` caller).
/// No oracle (trace/anim-callback-driven).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`; `ps`/`ucmd` to valid structs.
pub unsafe fn PM_AdjustAngleForWallRunUp(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
    doMove: qboolean,
) -> qboolean {
    let pmv = *addr_of!(pm);
    if (*ps).legsAnim == BOTH_FORCEWALLRUNFLIP_START {
        // wall-running up
        // stick to wall, if there is one
        let mut fwd: vec3_t = [0.0; 3];
        let mut traceTo: vec3_t = [0.0; 3];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut fwdAngles: vec3_t = [0.0; 3];
        let mut trace: trace_t = core::mem::zeroed();
        let dist: f32 = 128.0;

        VectorSet(&mut mins, -15.0, -15.0, 0.0);
        VectorSet(&mut maxs, 15.0, 15.0, 24.0);
        VectorSet(&mut fwdAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

        AngleVectors(&fwdAngles, Some(&mut fwd), None, None);
        VectorMA(&(*ps).origin, dist, &fwd, &mut traceTo);
        ((*pmv).trace.unwrap())(
            &mut trace,
            (*ps).origin.as_ptr(),
            mins.as_ptr(),
            maxs.as_ptr(),
            traceTo.as_ptr(),
            (*ps).clientNum,
            MASK_PLAYERSOLID,
        );
        if trace.fraction > 0.5 {
            // hmm, some room, see if there's a floor right here
            let mut trace2: trace_t = core::mem::zeroed();
            let mut top: vec3_t = [0.0; 3];
            let mut bottom: vec3_t = [0.0; 3];

            VectorCopy(&trace.endpos, &mut top);
            top[2] += ((*pmv).mins[2] * -1.0) + 4.0;
            VectorCopy(&top, &mut bottom);
            bottom[2] -= 64.0;
            ((*pmv).trace.unwrap())(
                &mut trace2,
                top.as_ptr(),
                (*pmv).mins.as_ptr(),
                (*pmv).maxs.as_ptr(),
                bottom.as_ptr(),
                (*ps).clientNum,
                MASK_PLAYERSOLID,
            );
            if trace2.allsolid == 0
                && trace2.startsolid == 0
                && trace2.fraction < 1.0
                && trace2.plane.normal[2] > 0.7
            {
                // slope we can stand on
                // cool, do the alt-flip and land on whetever it is we just scaled up
                VectorScale(&fwd, 100.0, &mut (*(*pmv).ps).velocity);
                (*(*pmv).ps).velocity[2] += 400.0;
                PM_SetAnim(
                    SETANIM_BOTH,
                    BOTH_FORCEWALLRUNFLIP_ALT,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
                (*(*pmv).ps).pm_flags |= PMF_JUMP_HELD;
                //ent->client->ps.pm_flags |= PMF_JUMPING|PMF_SLOW_MO_FALL;
                //ent->client->ps.forcePowersActive |= (1<<FP_LEVITATION);
                //G_AddEvent( ent, EV_JUMP, 0 );
                PM_AddEvent(EV_JUMP);
                (*ucmd).upmove = 0;
                return QFALSE;
            }
        }

        if //ucmd->upmove <= 0 &&
            (*ps).legsTimer > 0
            && (*ucmd).forwardmove > 0
            && trace.fraction < 1.0
            && (trace.plane.normal[2] >= 0.0 && trace.plane.normal[2] <= 0.4/*MAX_WALL_RUN_Z_NORMAL*/)
        {
            // still a vertical wall there
            // make sure there's not a ceiling above us!
            let mut trace2: trace_t = core::mem::zeroed();
            VectorCopy(&(*ps).origin, &mut traceTo);
            traceTo[2] += 64.0;
            ((*pmv).trace.unwrap())(
                &mut trace2,
                (*ps).origin.as_ptr(),
                mins.as_ptr(),
                maxs.as_ptr(),
                traceTo.as_ptr(),
                (*ps).clientNum,
                MASK_PLAYERSOLID,
            );
            if trace2.fraction < 1.0 {
                // will hit a ceiling, so force jump-off right now
                // NOTE: hits any entity or clip brush in the way, too, not just architecture!
            } else {
                // all clear, keep going
                // FIXME: don't pull around 90 turns
                // FIXME: simulate stepping up steps here, somehow?
                (*ucmd).forwardmove = 127;
                if (*ucmd).upmove < 0 {
                    (*ucmd).upmove = 0;
                }
                // make me face the wall
                (*ps).viewangles[YAW] = vectoyaw(&trace.plane.normal) + 180.0;
                PM_SetPMViewAngle(ps, (*ps).viewangles.as_mut_ptr(), ucmd);
                /*
                if ( ent->client->ps.viewEntity <= 0 || ent->client->ps.viewEntity >= ENTITYNUM_WORLD )
                {//don't clamp angles when looking through a viewEntity
                    SetClientViewAngle( ent, ent->client->ps.viewangles );
                }
                */
                (*ucmd).angles[YAW] = ANGLE2SHORT((*ps).viewangles[YAW]) - (*ps).delta_angles[YAW];
                //if ( ent->s.number || !player_locked )
                if true
                //aslkfhsakf
                {
                    if doMove == QTRUE {
                        // pull me toward the wall
                        VectorScale(&trace.plane.normal, -dist * trace.fraction, &mut (*ps).velocity);
                        // push me up
                        if (*ps).legsTimer > 200 {
                            // not at end of anim yet
                            let speed: f32 = 300.0;
                            /*
                            if ( ucmd->forwardmove < 0 )
                            {//slower
                                speed = 100;
                            }
                            else if ( ucmd->forwardmove > 0 )
                            {
                                speed = 250;//running speed
                            }
                            */
                            (*ps).velocity[2] = speed; // preserve z velocity
                        }
                    }
                }
                (*ucmd).forwardmove = 0;
                return QTRUE;
            }
        }
        // failed!
        if doMove == QTRUE {
            // stop it
            VectorScale(&fwd, -300.0, &mut (*ps).velocity);
            (*ps).velocity[2] += 200.0;
            //NPC_SetAnim( ent, SETANIM_BOTH, BOTH_FORCEWALLRUNFLIP_END, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
            //why?!?#?#@!%$R@$KR#F:Hdl;asfm
            PM_SetAnim(
                SETANIM_BOTH,
                BOTH_FORCEWALLRUNFLIP_END,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            (*ps).pm_flags |= PMF_JUMP_HELD;
            //ent->client->ps.pm_flags |= PMF_JUMPING|PMF_SLOW_MO_FALL;

            //FIXME do I need this in mp?
            //ent->client->ps.forcePowersActive |= (1<<FP_LEVITATION);
            PM_AddEvent(EV_JUMP);
            (*ucmd).upmove = 0;
            //return qtrue;
        }
    }
    QFALSE
}

/// `BG_ForceWallJumpStrength` (bg_pmove.c:1580) — the wall-jump launch strength,
/// `forceJumpStrength[FORCE_LEVEL_3] / 2.5`. `static` in C → `pub` here (the oracle +
/// the not-yet-ported `PM_AdjustAngleForWallJump` caller).
pub fn BG_ForceWallJumpStrength() -> f32 {
    forceJumpStrength[FORCE_LEVEL_3 as usize] / 2.5
}

// #define JUMP_OFF_WALL_SPEED 200.0f
const JUMP_OFF_WALL_SPEED: f32 = 200.0;

/// `PM_AdjustAngleForWallJump` (bg_pmove.c:1585) — while hugging a wall (rebound/hold
/// anims, or `PMF_STUCK_TO_WALL`), trace to the wall in the anim's direction: if it's
/// still there, align to it and stick; otherwise (and if already stuck) launch off it
/// with a force-jump. `pm->debugMelee` enables the "hold until release" skill. `static`
/// in C → `pub` here (the `PmoveSingle` caller). No oracle (trace/anim-callback-driven).
///
/// # Safety
/// `pm` must point to a valid `pmove_t`; `ps`/`ucmd` to valid structs.
pub unsafe fn PM_AdjustAngleForWallJump(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
    doMove: qboolean,
) -> qboolean {
    let pmv = *addr_of!(pm);
    if ((BG_InReboundJump((*ps).legsAnim) != QFALSE || BG_InReboundHold((*ps).legsAnim) != QFALSE)
        && (BG_InReboundJump((*ps).torsoAnim) != QFALSE
            || BG_InReboundHold((*ps).torsoAnim) != QFALSE))
        || ((*(*pmv).ps).pm_flags & PMF_STUCK_TO_WALL) != 0
    {
        // hugging wall, getting ready to jump off
        // stick to wall, if there is one
        let mut checkDir: vec3_t = [0.0; 3];
        let mut traceTo: vec3_t = [0.0; 3];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut fwdAngles: vec3_t = [0.0; 3];
        let mut trace: trace_t = core::mem::zeroed();
        let dist: f32 = 128.0;
        let yawAdjust: f32;

        VectorSet(&mut mins, (*pmv).mins[0], (*pmv).mins[1], 0.0);
        VectorSet(&mut maxs, (*pmv).maxs[0], (*pmv).maxs[1], 24.0);
        VectorSet(&mut fwdAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

        match (*ps).legsAnim {
            x if x == BOTH_FORCEWALLREBOUND_RIGHT || x == BOTH_FORCEWALLHOLD_RIGHT => {
                AngleVectors(&fwdAngles, None, Some(&mut checkDir), None);
                yawAdjust = -90.0;
            }
            x if x == BOTH_FORCEWALLREBOUND_LEFT || x == BOTH_FORCEWALLHOLD_LEFT => {
                AngleVectors(&fwdAngles, None, Some(&mut checkDir), None);
                // VectorScale( checkDir, -1, checkDir ) — in-place (vec3_t is Copy).
                let checkDirIn = checkDir;
                VectorScale(&checkDirIn, -1.0, &mut checkDir);
                yawAdjust = 90.0;
            }
            x if x == BOTH_FORCEWALLREBOUND_FORWARD || x == BOTH_FORCEWALLHOLD_FORWARD => {
                AngleVectors(&fwdAngles, Some(&mut checkDir), None, None);
                yawAdjust = 180.0;
            }
            x if x == BOTH_FORCEWALLREBOUND_BACK || x == BOTH_FORCEWALLHOLD_BACK => {
                AngleVectors(&fwdAngles, Some(&mut checkDir), None, None);
                // VectorScale( checkDir, -1, checkDir ) — in-place (vec3_t is Copy).
                let checkDirIn = checkDir;
                VectorScale(&checkDirIn, -1.0, &mut checkDir);
                yawAdjust = 0.0;
            }
            _ => {
                // WTF???
                (*(*pmv).ps).pm_flags &= !PMF_STUCK_TO_WALL;
                return QFALSE;
            }
        }
        if (*pmv).debugMelee != 0 {
            // uber-skillz
            if (*ucmd).upmove > 0 {
                // hold on until you let go manually
                if BG_InReboundHold((*ps).legsAnim) != QFALSE {
                    // keep holding
                    if (*ps).legsTimer < 150 {
                        (*ps).legsTimer = 150;
                    }
                } else {
                    // if got to hold part of anim, play hold anim
                    if (*ps).legsTimer <= 300 {
                        (*ps).saberHolstered = 2;
                        PM_SetAnim(
                            SETANIM_BOTH,
                            BOTH_FORCEWALLRELEASE_FORWARD
                                + ((*ps).legsAnim - BOTH_FORCEWALLHOLD_FORWARD),
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            0,
                        );
                        (*ps).legsTimer = 150;
                        (*ps).torsoTimer = 150;
                    }
                }
            }
        }
        VectorMA(&(*ps).origin, dist, &checkDir, &mut traceTo);
        ((*pmv).trace.unwrap())(
            &mut trace,
            (*ps).origin.as_ptr(),
            mins.as_ptr(),
            maxs.as_ptr(),
            traceTo.as_ptr(),
            (*ps).clientNum,
            MASK_PLAYERSOLID,
        );
        if //ucmd->upmove <= 0 &&
            (*ps).legsTimer > 100
            && trace.fraction < 1.0
            && (trace.plane.normal[2] as f64).abs() <= 0.2f32 as f64/*MAX_WALL_GRAB_SLOPE*/
        {
            // still a vertical wall there
            // FIXME: don't pull around 90 turns
            /*
            if ( ent->s.number || !player_locked )
            {
                ucmd->forwardmove = 127;
            }
            */
            if (*ucmd).upmove < 0 {
                (*ucmd).upmove = 0;
            }
            // align me to the wall
            (*ps).viewangles[YAW] = vectoyaw(&trace.plane.normal) + yawAdjust;
            PM_SetPMViewAngle(ps, (*ps).viewangles.as_mut_ptr(), ucmd);
            /*
            if ( ent->client->ps.viewEntity <= 0 || ent->client->ps.viewEntity >= ENTITYNUM_WORLD )
            {//don't clamp angles when looking through a viewEntity
                SetClientViewAngle( ent, ent->client->ps.viewangles );
            }
            */
            (*ucmd).angles[YAW] = ANGLE2SHORT((*ps).viewangles[YAW]) - (*ps).delta_angles[YAW];
            //if ( ent->s.number || !player_locked )
            if true {
                if doMove == QTRUE {
                    // pull me toward the wall
                    VectorScale(&trace.plane.normal, -128.0, &mut (*ps).velocity);
                }
            }
            (*ucmd).upmove = 0;
            (*ps).pm_flags |= PMF_STUCK_TO_WALL;
            return QTRUE;
        } else if doMove == QTRUE && ((*ps).pm_flags & PMF_STUCK_TO_WALL) != 0 {
            // jump off
            // push off of it!
            (*ps).pm_flags &= !PMF_STUCK_TO_WALL;
            (*ps).velocity[0] = 0.0;
            (*ps).velocity[1] = 0.0;
            VectorScale(&checkDir, -JUMP_OFF_WALL_SPEED, &mut (*ps).velocity);
            (*ps).velocity[2] = BG_ForceWallJumpStrength();
            (*ps).pm_flags |= PMF_JUMP_HELD; //PMF_JUMPING|PMF_JUMP_HELD;
                                             //G_SoundOnEnt( ent, CHAN_BODY, "sound/weapons/force/jump.wav" );
            (*ps).fd.forceJumpSound = 1; // this is a stupid thing, i should fix it.
                                         //ent->client->ps.forcePowersActive |= (1<<FP_LEVITATION);
            if (*ps).origin[2] < (*ps).fd.forceJumpZStart {
                (*ps).fd.forceJumpZStart = (*ps).origin[2];
            }
            //FIXME do I need this?

            BG_ForcePowerDrain(ps, FP_LEVITATION, 10);
            // no control for half a second
            (*ps).pm_flags |= PMF_TIME_KNOCKBACK;
            (*ps).pm_time = 500;
            (*ucmd).forwardmove = 0;
            (*ucmd).rightmove = 0;
            (*ucmd).upmove = 127;

            if BG_InReboundHold((*ps).legsAnim) != QFALSE {
                // if was in hold pose, release now
                PM_SetAnim(
                    SETANIM_BOTH,
                    BOTH_FORCEWALLRELEASE_FORWARD + ((*ps).legsAnim - BOTH_FORCEWALLHOLD_FORWARD),
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            } else {
                //PM_JumpForDir();
                PM_SetAnim(
                    SETANIM_LEGS,
                    BOTH_FORCEJUMP1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
                    0,
                );
            }

            //return qtrue;
        }
    }
    (*ps).pm_flags &= !PMF_STUCK_TO_WALL;
    QFALSE
}

/// `PM_SetForceJumpZStart` (bg_pmove.c:1737) — record the Z height at which a force jump
/// began (used to scale landing damage). A zero value is nudged to `-0.1` so it stays
/// distinguishable from "not jumping". The `-= 0.1` promotes through `f64` (the `0.1`
/// double literal), replicated here to stay bit-exact.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_SetForceJumpZStart(value: f32) {
    let pmv = *addr_of!(pm);
    (*(*pmv).ps).fd.forceJumpZStart = value;
    if (*(*pmv).ps).fd.forceJumpZStart == 0.0 {
        (*(*pmv).ps).fd.forceJumpZStart =
            ((*(*pmv).ps).fd.forceJumpZStart as f64 - 0.1) as f32;
    }
}

/// `forceJumpHeightMax[NUM_FORCE_POWER_LEVELS]` (bg_pmove.c:1746) — kept here with
/// its sibling force tables though it sits mid-file (the intervening functions are
/// not yet ported above). Values are the jump heights plus step+crouch adjustments.
pub static forceJumpHeightMax: [f32; NUM_FORCE_POWER_LEVELS] = [
    66.0,  //normal jump (32+stepheight(18)+crouchdiff(24) = 74)
    130.0, //(96+stepheight(18)+crouchdiff(24) = 138)
    226.0, //(192+stepheight(18)+crouchdiff(24) = 234)
    418.0, //(384+stepheight(18)+crouchdiff(24) = 426)
];

/// `PM_GrabWallForJump` (bg_pmove.c:1754) — latch the player onto a wall for a rebound
/// jump: play the (caller-supplied) grab anim, make the grab sound, set `PMF_STUCK_TO_WALL`.
/// No oracle: delegates to the oracle-verified `PM_SetAnim`/`PM_AddEvent`.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_GrabWallForJump(anim: c_int) {
    //NOTE!!! assumes an appropriate anim is being passed in!!!
    let pmv = *addr_of!(pm);
    PM_SetAnim(
        SETANIM_BOTH,
        anim,
        SETANIM_FLAG_RESTART | SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        0,
    );
    PM_AddEvent(EV_JUMP); //make sound for grab
    (*(*pmv).ps).pm_flags |= PMF_STUCK_TO_WALL;
}

/*
=============
PM_CheckJump
=============
*/
/// `PM_CheckJump` (bg_pmove.c:1766) — the master jump gate. Rejects when jumping is
/// disallowed (vehicle NPC, knockdown/throw hand-extends, jetpack, just-respawned,
/// in-knockdown/roll), drains force power for an in-progress force jump, then handles
/// the `METROID_JUMP` held-in-air force-jump scaling, the wall-run / wall-flip /
/// wall-rebound special jumps, and finally the plain jump (sets `JUMP_VELOCITY`, clears
/// the ground, plays the directional jump anim). Returns `qtrue` if a jump was started.
///
/// `static` in C → `pub` here. Trace/callback-driven (the whole `pm` keystone) → no
/// bit-exact oracle; faithful transcription, behavioural. The two large commented-out
/// blocks (the SP saber jump-attack hacks and the run-up-wall-flip-backwards variant)
/// are carried verbatim as comments. C `int`→`float` promotions are preserved (the
/// `* 1.5` double in the dot-comparison, the `JUMP_VELOCITY` int → f32 conversions).
///
/// # Safety
/// `pm` (with a live `ps`) and `pm_entSelf` must be valid.
pub unsafe fn PM_CheckJump() -> qboolean {
    let pmv = *addr_of!(pm);

    let mut allowFlips: qboolean = QTRUE;

    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int {
        let pEnt: *mut bgEntity_t = *addr_of!(pm_entSelf);

        if (*pEnt).s.eType == ET_NPC && (*pEnt).s.NPC_class == CLASS_VEHICLE {
            //no!
            return QFALSE;
        }
    }

    if (*(*pmv).ps).forceHandExtend == HANDEXTEND_KNOCKDOWN
        || (*(*pmv).ps).forceHandExtend == HANDEXTEND_PRETHROWN
        || (*(*pmv).ps).forceHandExtend == HANDEXTEND_POSTTHROWN
    {
        return QFALSE;
    }

    if (*(*pmv).ps).pm_type == PM_JETPACK {
        //there's no actual jumping while we jetpack
        return QFALSE;
    }

    //Don't allow jump until all buttons are up
    if ((*(*pmv).ps).pm_flags & PMF_RESPAWNED) != 0 {
        return QFALSE;
    }

    if PM_InKnockDown((*pmv).ps) != QFALSE || BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) != QFALSE
    {
        //in knockdown
        return QFALSE;
    }

    if (*(*pmv).ps).weapon == WP_SABER {
        let saber1: *mut saberInfo_t = BG_MySaber((*(*pmv).ps).clientNum, 0);
        let saber2: *mut saberInfo_t = BG_MySaber((*(*pmv).ps).clientNum, 1);
        if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_FLIPS != 0 {
            allowFlips = QFALSE;
        }
        if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_FLIPS != 0 {
            allowFlips = QFALSE;
        }
    }

    if (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE
        || (*(*pmv).ps).origin[2] < (*(*pmv).ps).fd.forceJumpZStart
    {
        (*(*pmv).ps).fd.forcePowersActive &= !(1 << FP_LEVITATION);
    }

    if ((*(*pmv).ps).fd.forcePowersActive & (1 << FP_LEVITATION)) != 0 {
        //Force jump is already active.. continue draining power appropriately until we land.
        if (*(*pmv).ps).fd.forcePowerDebounce[FP_LEVITATION as usize] < (*pmv).cmd.serverTime {
            if (*pmv).gametype == GT_DUEL || (*pmv).gametype == GT_POWERDUEL {
                //jump takes less power
                BG_ForcePowerDrain((*pmv).ps, FP_LEVITATION, 1);
            } else {
                BG_ForcePowerDrain((*pmv).ps, FP_LEVITATION, 5);
            }
            if (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] >= FORCE_LEVEL_2 {
                (*(*pmv).ps).fd.forcePowerDebounce[FP_LEVITATION as usize] =
                    (*pmv).cmd.serverTime + 300;
            } else {
                (*(*pmv).ps).fd.forcePowerDebounce[FP_LEVITATION as usize] =
                    (*pmv).cmd.serverTime + 200;
            }
        }
    }

    if (*(*pmv).ps).forceJumpFlip != QFALSE {
        //Forced jump anim
        let mut anim = BOTH_FORCEINAIR1;
        let mut parts = SETANIM_BOTH;
        if allowFlips != 0 {
            if (*pmv).cmd.forwardmove > 0 {
                anim = BOTH_FLIP_F;
            } else if (*pmv).cmd.forwardmove < 0 {
                anim = BOTH_FLIP_B;
            } else if (*pmv).cmd.rightmove > 0 {
                anim = BOTH_FLIP_R;
            } else if (*pmv).cmd.rightmove < 0 {
                anim = BOTH_FLIP_L;
            }
        } else {
            if (*pmv).cmd.forwardmove > 0 {
                anim = BOTH_FORCEINAIR1;
            } else if (*pmv).cmd.forwardmove < 0 {
                anim = BOTH_FORCEINAIRBACK1;
            } else if (*pmv).cmd.rightmove > 0 {
                anim = BOTH_FORCEINAIRRIGHT1;
            } else if (*pmv).cmd.rightmove < 0 {
                anim = BOTH_FORCEINAIRLEFT1;
            }
        }
        if (*(*pmv).ps).weaponTime != 0 {
            //FIXME: really only care if we're in a saber attack anim...
            parts = SETANIM_LEGS;
        }

        PM_SetAnim(parts, anim, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD, 150);
        (*(*pmv).ps).forceJumpFlip = QFALSE;
        return QTRUE;
    }
    // #if METROID_JUMP (defined 1)
    if (*pmv).waterlevel < 2 {
        if (*(*pmv).ps).gravity > 0 {
            //can't do this in zero-G
            if PM_ForceJumpingUp() != QFALSE {
                //holding jump in air
                let curHeight = (*(*pmv).ps).origin[2] - (*(*pmv).ps).fd.forceJumpZStart;
                let lvl = (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] as usize;
                //check for max force jump level and cap off & cut z vel
                if (curHeight <= forceJumpHeight[0] //still below minimum jump height
                    || ((*(*pmv).ps).fd.forcePower != 0 && (*pmv).cmd.upmove >= 10))//still have force power available and still trying to jump up
                    && curHeight < forceJumpHeight[lvl]
                    && (*(*pmv).ps).fd.forceJumpZStart != 0.0
                //still below maximum jump height
                {
                    //can still go up
                    if curHeight > forceJumpHeight[0] {
                        //passed normal jump height  *2?
                        if ((*(*pmv).ps).fd.forcePowersActive & (1 << FP_LEVITATION)) == 0 {
                            //haven't started forcejump yet
                            //start force jump
                            (*(*pmv).ps).fd.forcePowersActive |= 1 << FP_LEVITATION;
                            (*(*pmv).ps).fd.forceJumpSound = 1;
                            //play flip
                            if ((*pmv).cmd.forwardmove != 0 || (*pmv).cmd.rightmove != 0) && //pushing in a dir
                                (*(*pmv).ps).legsAnim != BOTH_FLIP_F && //not already flipping
                                (*(*pmv).ps).legsAnim != BOTH_FLIP_B &&
                                (*(*pmv).ps).legsAnim != BOTH_FLIP_R &&
                                (*(*pmv).ps).legsAnim != BOTH_FLIP_L
                                && allowFlips != 0
                            {
                                let mut anim = BOTH_FORCEINAIR1;
                                let mut parts = SETANIM_BOTH;

                                if (*pmv).cmd.forwardmove > 0 {
                                    anim = BOTH_FLIP_F;
                                } else if (*pmv).cmd.forwardmove < 0 {
                                    anim = BOTH_FLIP_B;
                                } else if (*pmv).cmd.rightmove > 0 {
                                    anim = BOTH_FLIP_R;
                                } else if (*pmv).cmd.rightmove < 0 {
                                    anim = BOTH_FLIP_L;
                                }
                                if (*(*pmv).ps).weaponTime != 0 {
                                    parts = SETANIM_LEGS;
                                }

                                PM_SetAnim(
                                    parts,
                                    anim,
                                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                    150,
                                );
                            } else if (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize]
                                > FORCE_LEVEL_1
                            {
                                let mut facingFwd: vec3_t = [0.0; 3];
                                let mut facingRight: vec3_t = [0.0; 3];
                                let mut facingAngles: vec3_t = [0.0; 3];
                                let mut anim: c_int = -1;
                                let dotR: f32;
                                let dotF: f32;

                                VectorSet(&mut facingAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

                                AngleVectors(
                                    &facingAngles,
                                    Some(&mut facingFwd),
                                    Some(&mut facingRight),
                                    None,
                                );
                                dotR = DotProduct(&facingRight, &(*(*pmv).ps).velocity);
                                dotF = DotProduct(&facingFwd, &(*(*pmv).ps).velocity);

                                if dotR.abs() > dotF.abs() * 1.5 {
                                    if dotR > 150.0 {
                                        anim = BOTH_FORCEJUMPRIGHT1;
                                    } else if dotR < -150.0 {
                                        anim = BOTH_FORCEJUMPLEFT1;
                                    }
                                } else if dotF > 150.0 {
                                    anim = BOTH_FORCEJUMP1;
                                } else if dotF < -150.0 {
                                    anim = BOTH_FORCEJUMPBACK1;
                                }
                                if anim != -1 {
                                    let mut parts = SETANIM_BOTH;
                                    if (*(*pmv).ps).weaponTime != 0 {
                                        //FIXME: really only care if we're in a saber attack anim...
                                        parts = SETANIM_LEGS;
                                    }

                                    PM_SetAnim(
                                        parts,
                                        anim,
                                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                        150,
                                    );
                                }
                            }
                        } else {
                            //jump is already active (the anim has started)
                            if (*(*pmv).ps).legsTimer < 1 {
                                //not in the middle of a legsAnim
                                let anim = (*(*pmv).ps).legsAnim;
                                let mut newAnim: c_int = -1;
                                if anim == BOTH_FORCEJUMP1 {
                                    newAnim = BOTH_FORCELAND1; //BOTH_FORCEINAIR1;
                                } else if anim == BOTH_FORCEJUMPBACK1 {
                                    newAnim = BOTH_FORCELANDBACK1; //BOTH_FORCEINAIRBACK1;
                                } else if anim == BOTH_FORCEJUMPLEFT1 {
                                    newAnim = BOTH_FORCELANDLEFT1; //BOTH_FORCEINAIRLEFT1;
                                } else if anim == BOTH_FORCEJUMPRIGHT1 {
                                    newAnim = BOTH_FORCELANDRIGHT1; //BOTH_FORCEINAIRRIGHT1;
                                }
                                if newAnim != -1 {
                                    let mut parts = SETANIM_BOTH;
                                    if (*(*pmv).ps).weaponTime != 0 {
                                        parts = SETANIM_LEGS;
                                    }

                                    PM_SetAnim(
                                        parts,
                                        newAnim,
                                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                        150,
                                    );
                                }
                            }
                        }
                    }

                    //need to scale this down, start with height velocity (based on max force jump height) and scale down to regular jump vel
                    (*(*pmv).ps).velocity[2] =
                        (forceJumpHeight[lvl] - curHeight) / forceJumpHeight[lvl]
                            * forceJumpStrength[lvl]; //JUMP_VELOCITY;
                    (*(*pmv).ps).velocity[2] /= 10.0;
                    (*(*pmv).ps).velocity[2] += JUMP_VELOCITY as f32;
                    (*(*pmv).ps).pm_flags |= PMF_JUMP_HELD;
                } else if curHeight > forceJumpHeight[0]
                    && curHeight < forceJumpHeight[lvl] - forceJumpHeight[0]
                {
                    //still have some headroom, don't totally stop it
                    if (*(*pmv).ps).velocity[2] > JUMP_VELOCITY as f32 {
                        (*(*pmv).ps).velocity[2] = JUMP_VELOCITY as f32;
                    }
                } else {
                    //pm->ps->velocity[2] = 0;
                    //rww - changed for the sake of balance in multiplayer

                    if (*(*pmv).ps).velocity[2] > JUMP_VELOCITY as f32 {
                        (*(*pmv).ps).velocity[2] = JUMP_VELOCITY as f32;
                    }
                }
                (*pmv).cmd.upmove = 0;
                return QFALSE;
            }
        }
    }
    // #endif

    //Not jumping
    if (*pmv).cmd.upmove < 10 && (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE {
        return QFALSE;
    }

    // must wait for jump to be released
    if ((*(*pmv).ps).pm_flags & PMF_JUMP_HELD) != 0 {
        // clear upmove so cmdscale doesn't lower running speed
        (*pmv).cmd.upmove = 0;
        return QFALSE;
    }

    if (*(*pmv).ps).gravity <= 0 {
        //in low grav, you push in the dir you're facing as long as there is something behind you to shove off of
        let mut forward: vec3_t = [0.0; 3];
        let mut back: vec3_t = [0.0; 3];
        let mut trace: trace_t = core::mem::zeroed();

        AngleVectors(&(*(*pmv).ps).viewangles, Some(&mut forward), None, None);
        VectorMA(&(*(*pmv).ps).origin, -8.0, &forward, &mut back);
        ((*pmv).trace.unwrap())(
            &mut trace,
            (*(*pmv).ps).origin.as_ptr(),
            (*pmv).mins.as_ptr(),
            (*pmv).maxs.as_ptr(),
            back.as_ptr(),
            (*(*pmv).ps).clientNum,
            (*pmv).tracemask,
        );

        if trace.fraction <= 1.0 {
            let v_snap = (*(*pmv).ps).velocity;
            VectorMA(
                &v_snap,
                (JUMP_VELOCITY * 2) as f32,
                &forward,
                &mut (*(*pmv).ps).velocity,
            );
            PM_SetAnim(
                SETANIM_LEGS,
                BOTH_FORCEJUMP1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
                150,
            );
        } //else no surf close enough to push off of
        (*pmv).cmd.upmove = 0;
    } else if (*pmv).cmd.upmove > 0
        && (*pmv).waterlevel < 2
        && (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_0
        && ((*(*pmv).ps).pm_flags & PMF_JUMP_HELD) == 0
        && ((*(*pmv).ps).weapon == WP_SABER || (*(*pmv).ps).weapon == WP_MELEE)
        && PM_IsRocketTrooper() == QFALSE
        && BG_HasYsalamiri((*pmv).gametype, (*pmv).ps) == QFALSE
        && BG_CanUseFPNow((*pmv).gametype, (*pmv).ps, (*pmv).cmd.serverTime, FP_LEVITATION)
            != QFALSE
    {
        let mut allowWallRuns: qboolean = QTRUE;
        let mut allowWallFlips: qboolean = QTRUE;
        let mut allowFlips: qboolean = QTRUE;
        let mut allowWallGrabs: qboolean = QTRUE;
        if (*(*pmv).ps).weapon == WP_SABER {
            let saber1: *mut saberInfo_t = BG_MySaber((*(*pmv).ps).clientNum, 0);
            let saber2: *mut saberInfo_t = BG_MySaber((*(*pmv).ps).clientNum, 1);
            if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_WALL_RUNS != 0 {
                allowWallRuns = QFALSE;
            }
            if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_WALL_RUNS != 0 {
                allowWallRuns = QFALSE;
            }
            if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_WALL_FLIPS != 0 {
                allowWallFlips = QFALSE;
            }
            if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_WALL_FLIPS != 0 {
                allowWallFlips = QFALSE;
            }
            if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_FLIPS != 0 {
                allowFlips = QFALSE;
            }
            if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_FLIPS != 0 {
                allowFlips = QFALSE;
            }
            if !saber1.is_null() && (*saber1).saberFlags & SFL_NO_WALL_GRAB != 0 {
                allowWallGrabs = QFALSE;
            }
            if !saber2.is_null() && (*saber2).saberFlags & SFL_NO_WALL_GRAB != 0 {
                allowWallGrabs = QFALSE;
            }
        }

        if (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE {
            //on the ground
            //check for left-wall and right-wall special jumps
            let mut anim: c_int = -1;
            let mut vertPush: f32 = 0.0;
            if (*pmv).cmd.rightmove > 0
                && (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_1
            {
                //strafing right
                if (*pmv).cmd.forwardmove > 0 {
                    //wall-run
                    if allowWallRuns != 0 {
                        vertPush = forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.0;
                        anim = BOTH_WALL_RUN_RIGHT;
                    }
                } else if (*pmv).cmd.forwardmove == 0 {
                    //wall-flip
                    if allowWallFlips != 0 {
                        vertPush = forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.25;
                        anim = BOTH_WALL_FLIP_RIGHT;
                    }
                }
            } else if (*pmv).cmd.rightmove < 0
                && (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_1
            {
                //strafing left
                if (*pmv).cmd.forwardmove > 0 {
                    //wall-run
                    if allowWallRuns != 0 {
                        vertPush = forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.0;
                        anim = BOTH_WALL_RUN_LEFT;
                    }
                } else if (*pmv).cmd.forwardmove == 0 {
                    //wall-flip
                    if allowWallFlips != 0 {
                        vertPush = forceJumpStrength[FORCE_LEVEL_2 as usize] / 2.25;
                        anim = BOTH_WALL_FLIP_LEFT;
                    }
                }
            } else if (*pmv).cmd.forwardmove < 0 && ((*pmv).cmd.buttons & BUTTON_ATTACK) == 0 {
                //backflip
                if allowFlips != 0 {
                    vertPush = JUMP_VELOCITY as f32;
                    anim = BOTH_FLIP_BACK1; //BG_PickAnim( BOTH_FLIP_BACK1, BOTH_FLIP_BACK3 );
                }
            }

            vertPush += 128.0; //give them an extra shove

            if anim != -1 {
                let mut fwd: vec3_t = [0.0; 3];
                let mut right: vec3_t = [0.0; 3];
                let mut traceto: vec3_t = [0.0; 3];
                let mut mins: vec3_t = [0.0; 3];
                let mut maxs: vec3_t = [0.0; 3];
                let mut fwdAngles: vec3_t = [0.0; 3];
                let mut idealNormal: vec3_t = [0.0; 3];
                let mut wallNormal: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();
                let mut doTrace = QFALSE;
                let contents: c_int = MASK_SOLID; //MASK_PLAYERSOLID;

                VectorSet(&mut mins, (*pmv).mins[0], (*pmv).mins[1], 0.0);
                VectorSet(&mut maxs, (*pmv).maxs[0], (*pmv).maxs[1], 24.0);
                VectorSet(&mut fwdAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

                //memset(&trace, 0, sizeof(trace)); //to shut the compiler up (already zeroed)

                AngleVectors(&fwdAngles, Some(&mut fwd), Some(&mut right), None);

                //trace-check for a wall, if necc.
                if anim == BOTH_WALL_FLIP_LEFT || anim == BOTH_WALL_RUN_LEFT {
                    //NOTE: BOTH_WALL_FLIP_LEFT purposely falls through to BOTH_WALL_RUN_LEFT!
                    doTrace = QTRUE;
                    VectorMA(&(*(*pmv).ps).origin, -16.0, &right, &mut traceto);
                } else if anim == BOTH_WALL_FLIP_RIGHT || anim == BOTH_WALL_RUN_RIGHT {
                    //NOTE: BOTH_WALL_FLIP_RIGHT purposely falls through to BOTH_WALL_RUN_RIGHT!
                    doTrace = QTRUE;
                    VectorMA(&(*(*pmv).ps).origin, 16.0, &right, &mut traceto);
                } else if anim == BOTH_WALL_FLIP_BACK1 {
                    doTrace = QTRUE;
                    VectorMA(&(*(*pmv).ps).origin, 16.0, &fwd, &mut traceto);
                }

                if doTrace != QFALSE {
                    ((*pmv).trace.unwrap())(
                        &mut trace,
                        (*(*pmv).ps).origin.as_ptr(),
                        mins.as_ptr(),
                        maxs.as_ptr(),
                        traceto.as_ptr(),
                        (*(*pmv).ps).clientNum,
                        contents,
                    );
                    VectorCopy(&trace.plane.normal, &mut wallNormal);
                    VectorNormalize(&mut wallNormal);
                    VectorSubtract(&(*(*pmv).ps).origin, &traceto, &mut idealNormal);
                    VectorNormalize(&mut idealNormal);
                }

                if doTrace == QFALSE
                    || (trace.fraction < 1.0
                        && ((trace.entityNum as c_int) < MAX_CLIENTS as c_int
                            || DotProduct(&wallNormal, &idealNormal) > 0.7))
                {
                    //there is a wall there.. or hit a client
                    if (anim != BOTH_WALL_RUN_LEFT
                        && anim != BOTH_WALL_RUN_RIGHT
                        && anim != BOTH_FORCEWALLRUNFLIP_START)
                        || (wallNormal[2] >= 0.0 && wallNormal[2] <= 0.4
                        /*MAX_WALL_RUN_Z_NORMAL*/)
                    {
                        //wall-runs can only run on perfectly flat walls, sorry.
                        let mut parts;
                        //move me to side
                        if anim == BOTH_WALL_FLIP_LEFT {
                            (*(*pmv).ps).velocity[1] = 0.0;
                            (*(*pmv).ps).velocity[0] = 0.0;
                            let v_snap = (*(*pmv).ps).velocity;
                            VectorMA(&v_snap, 150.0, &right, &mut (*(*pmv).ps).velocity);
                        } else if anim == BOTH_WALL_FLIP_RIGHT {
                            (*(*pmv).ps).velocity[1] = 0.0;
                            (*(*pmv).ps).velocity[0] = 0.0;
                            let v_snap = (*(*pmv).ps).velocity;
                            VectorMA(&v_snap, -150.0, &right, &mut (*(*pmv).ps).velocity);
                        } else if anim == BOTH_FLIP_BACK1
                            || anim == BOTH_FLIP_BACK2
                            || anim == BOTH_FLIP_BACK3
                            || anim == BOTH_WALL_FLIP_BACK1
                        {
                            (*(*pmv).ps).velocity[1] = 0.0;
                            (*(*pmv).ps).velocity[0] = 0.0;
                            let v_snap = (*(*pmv).ps).velocity;
                            VectorMA(&v_snap, -150.0, &fwd, &mut (*(*pmv).ps).velocity);
                        }

                        /*
                        if ( doTrace && anim != BOTH_WALL_RUN_LEFT && anim != BOTH_WALL_RUN_RIGHT )
                        {
                            if (trace.entityNum < MAX_CLIENTS)
                            {
                                pm->ps->forceKickFlip = trace.entityNum+1; //let the server know that this person gets kicked by this client
                            }
                        }
                        */

                        //up
                        if vertPush != 0.0 {
                            (*(*pmv).ps).velocity[2] = vertPush;
                            (*(*pmv).ps).fd.forcePowersActive |= 1 << FP_LEVITATION;
                        }
                        //animate me
                        parts = SETANIM_LEGS;
                        if anim == BOTH_BUTTERFLY_LEFT {
                            parts = SETANIM_BOTH;
                            (*pmv).cmd.buttons &= !BUTTON_ATTACK;
                            (*(*pmv).ps).saberMove = LS_NONE;
                        } else if (*(*pmv).ps).weaponTime == 0 {
                            parts = SETANIM_BOTH;
                        }
                        PM_SetAnim(parts, anim, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD, 0);
                        if anim == BOTH_BUTTERFLY_LEFT {
                            (*(*pmv).ps).weaponTime = (*(*pmv).ps).torsoTimer;
                        }
                        PM_SetForceJumpZStart((*(*pmv).ps).origin[2]); //so we don't take damage if we land at same height
                        (*(*pmv).ps).pm_flags |= PMF_JUMP_HELD;
                        (*pmv).cmd.upmove = 0;
                        (*(*pmv).ps).fd.forceJumpSound = 1;
                    }
                }
            }
        } else {
            //in the air
            let legsAnim = (*(*pmv).ps).legsAnim;

            if legsAnim == BOTH_WALL_RUN_LEFT || legsAnim == BOTH_WALL_RUN_RIGHT {
                //running on a wall
                let mut right: vec3_t = [0.0; 3];
                let mut traceto: vec3_t = [0.0; 3];
                let mut mins: vec3_t = [0.0; 3];
                let mut maxs: vec3_t = [0.0; 3];
                let mut fwdAngles: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();
                let mut anim: c_int = -1;

                VectorSet(&mut mins, (*pmv).mins[0], (*pmv).mins[0], 0.0);
                VectorSet(&mut maxs, (*pmv).maxs[0], (*pmv).maxs[0], 24.0);
                VectorSet(&mut fwdAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

                AngleVectors(&fwdAngles, None, Some(&mut right), None);

                if legsAnim == BOTH_WALL_RUN_LEFT {
                    if (*(*pmv).ps).legsTimer > 400 {
                        //not at the end of the anim
                        let animLen = PM_AnimLength(0, BOTH_WALL_RUN_LEFT) as f32;
                        if ((*(*pmv).ps).legsTimer as f32) < animLen - 400.0 {
                            //not at start of anim
                            VectorMA(&(*(*pmv).ps).origin, -16.0, &right, &mut traceto);
                            anim = BOTH_WALL_RUN_LEFT_FLIP;
                        }
                    }
                } else if legsAnim == BOTH_WALL_RUN_RIGHT {
                    if (*(*pmv).ps).legsTimer > 400 {
                        //not at the end of the anim
                        let animLen = PM_AnimLength(0, BOTH_WALL_RUN_RIGHT) as f32;
                        if ((*(*pmv).ps).legsTimer as f32) < animLen - 400.0 {
                            //not at start of anim
                            VectorMA(&(*(*pmv).ps).origin, 16.0, &right, &mut traceto);
                            anim = BOTH_WALL_RUN_RIGHT_FLIP;
                        }
                    }
                }
                if anim != -1 {
                    ((*pmv).trace.unwrap())(
                        &mut trace,
                        (*(*pmv).ps).origin.as_ptr(),
                        mins.as_ptr(),
                        maxs.as_ptr(),
                        traceto.as_ptr(),
                        (*(*pmv).ps).clientNum,
                        CONTENTS_SOLID | CONTENTS_BODY,
                    );
                    if trace.fraction < 1.0 {
                        //flip off wall
                        let mut parts;

                        if anim == BOTH_WALL_RUN_LEFT_FLIP {
                            (*(*pmv).ps).velocity[0] *= 0.5;
                            (*(*pmv).ps).velocity[1] *= 0.5;
                            let v_snap = (*(*pmv).ps).velocity;
                            VectorMA(&v_snap, 150.0, &right, &mut (*(*pmv).ps).velocity);
                        } else if anim == BOTH_WALL_RUN_RIGHT_FLIP {
                            (*(*pmv).ps).velocity[0] *= 0.5;
                            (*(*pmv).ps).velocity[1] *= 0.5;
                            let v_snap = (*(*pmv).ps).velocity;
                            VectorMA(&v_snap, -150.0, &right, &mut (*(*pmv).ps).velocity);
                        }
                        parts = SETANIM_LEGS;
                        if (*(*pmv).ps).weaponTime == 0 {
                            parts = SETANIM_BOTH;
                        }
                        PM_SetAnim(parts, anim, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD, 0);
                        (*pmv).cmd.upmove = 0;
                    }
                }
                if (*pmv).cmd.upmove != 0 {
                    //jump failed, so don't try to do normal jump code, just return
                    return QFALSE;
                }
            }
            //NEW JKA
            else if (*(*pmv).ps).legsAnim == BOTH_FORCEWALLRUNFLIP_START {
                let mut fwd: vec3_t = [0.0; 3];
                let mut traceto: vec3_t = [0.0; 3];
                let mut mins: vec3_t = [0.0; 3];
                let mut maxs: vec3_t = [0.0; 3];
                let mut fwdAngles: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();
                let mut anim: c_int = -1;
                let animLen: f32;

                VectorSet(&mut mins, (*pmv).mins[0], (*pmv).mins[0], 0.0);
                VectorSet(&mut maxs, (*pmv).maxs[0], (*pmv).maxs[0], 24.0);
                //hmm, did you mean [1] and [1]?
                VectorSet(&mut fwdAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);
                AngleVectors(&fwdAngles, Some(&mut fwd), None, None);

                debug_assert!(!(*addr_of!(pm_entSelf)).is_null()); //null pm_entSelf would be a Bad Thing<tm>
                animLen = BG_AnimLength(
                    (*(*addr_of!(pm_entSelf))).localAnimIndex,
                    BOTH_FORCEWALLRUNFLIP_START,
                ) as f32;
                if ((*(*pmv).ps).legsTimer as f32) < animLen - 400.0 {
                    //not at start of anim
                    VectorMA(&(*(*pmv).ps).origin, 16.0, &fwd, &mut traceto);
                    anim = BOTH_FORCEWALLRUNFLIP_END;
                }
                if anim != -1 {
                    ((*pmv).trace.unwrap())(
                        &mut trace,
                        (*(*pmv).ps).origin.as_ptr(),
                        mins.as_ptr(),
                        maxs.as_ptr(),
                        traceto.as_ptr(),
                        (*(*pmv).ps).clientNum,
                        CONTENTS_SOLID | CONTENTS_BODY,
                    );
                    if trace.fraction < 1.0 {
                        //flip off wall
                        let parts;

                        (*(*pmv).ps).velocity[0] *= 0.5;
                        (*(*pmv).ps).velocity[1] *= 0.5;
                        let v_snap = (*(*pmv).ps).velocity;
                        VectorMA(&v_snap, -300.0, &fwd, &mut (*(*pmv).ps).velocity);
                        (*(*pmv).ps).velocity[2] += 200.0;
                        if (*(*pmv).ps).weaponTime == 0 {
                            //not attacking, set anim on both
                            parts = SETANIM_BOTH;
                        } else {
                            parts = SETANIM_LEGS;
                        }
                        PM_SetAnim(parts, anim, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD, 0);
                        //FIXME: do damage to traceEnt, like above?
                        //pm->ps->pm_flags |= PMF_JUMPING|PMF_SLOW_MO_FALL;
                        //ha ha, so silly with your silly jumpy fally flags.
                        (*pmv).cmd.upmove = 0;
                        PM_AddEvent(EV_JUMP);
                    }
                }
                if (*pmv).cmd.upmove != 0 {
                    //jump failed, so don't try to do normal jump code, just return
                    return QFALSE;
                }
            }
            /*
            else if ( pm->cmd.forwardmove > 0 //pushing forward
                && pm->ps->fd.forcePowerLevel[FP_LEVITATION] > FORCE_LEVEL_1
                && pm->ps->velocity[2] > 200
                && PM_GroundDistance() <= 80 //unfortunately we do not have a happy ground timer like SP (this would use up more bandwidth if we wanted prediction workign right), so we'll just use the actual ground distance.
                && !BG_InSpecialJump(pm->ps->legsAnim))
            {//run up wall, flip backwards
                vec3_t fwd, traceto, mins, maxs, fwdAngles;
                trace_t	trace;
                vec3_t	idealNormal;

                VectorSet(mins, pm->mins[0],pm->mins[1],pm->mins[2]);
                VectorSet(maxs, pm->maxs[0],pm->maxs[1],pm->maxs[2]);
                VectorSet(fwdAngles, 0, pm->ps->viewangles[YAW], 0);

                AngleVectors( fwdAngles, fwd, NULL, NULL );
                VectorMA( pm->ps->origin, 32, fwd, traceto );

                pm->trace( &trace, pm->ps->origin, mins, maxs, traceto, pm->ps->clientNum, MASK_PLAYERSOLID );//FIXME: clip brushes too?
                VectorSubtract( pm->ps->origin, traceto, idealNormal );
                VectorNormalize( idealNormal );

                if ( trace.fraction < 1.0f )
                {//there is a wall there
                    int parts = SETANIM_LEGS;

                    pm->ps->velocity[0] = pm->ps->velocity[1] = 0;
                    VectorMA( pm->ps->velocity, -150, fwd, pm->ps->velocity );
                    pm->ps->velocity[2] += 128;

                    if ( !pm->ps->weaponTime )
                    {
                        parts = SETANIM_BOTH;
                    }
                    PM_SetAnim( parts, BOTH_WALL_FLIP_BACK1, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD, 0 );

                    pm->ps->legsTimer -= 600; //I force this anim to play to the end to prevent landing on your head and suddenly flipping over.
                                              //It is a bit too long at the end though, so I'll just shorten it.

                    PM_SetForceJumpZStart(pm->ps->origin[2]);//so we don't take damage if we land at same height
                    pm->cmd.upmove = 0;
                    pm->ps->fd.forceJumpSound = 1;
                    BG_ForcePowerDrain( pm->ps, FP_LEVITATION, 5 );

                    if (trace.entityNum < MAX_CLIENTS)
                    {
                        pm->ps->forceKickFlip = trace.entityNum+1; //let the server know that this person gets kicked by this client
                    }
                }
            }
            */
            else if (*pmv).cmd.forwardmove > 0 //pushing forward
                && (*(*pmv).ps).fd.forceRageRecoveryTime < (*pmv).cmd.serverTime	//not in a force Rage recovery period
                && (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_1
                && PM_WalkableGroundDistance() <= 80.0 //unfortunately we do not have a happy ground timer like SP (this would use up more bandwidth if we wanted prediction workign right), so we'll just use the actual ground distance.
                && ((*(*pmv).ps).legsAnim == BOTH_JUMP1 || (*(*pmv).ps).legsAnim == BOTH_INAIR1)
            //not in a flip or spin or anything
            {
                //run up wall, flip backwards
                if allowWallRuns != 0 {
                //FIXME: have to be moving... make sure it's opposite the wall... or at least forward?
                let mut wallWalkAnim = BOTH_WALL_FLIP_BACK1;
                let mut parts = SETANIM_LEGS;
                let contents: c_int = MASK_SOLID; //MASK_PLAYERSOLID;//CONTENTS_SOLID;
                                                  //qboolean kick = qtrue;
                if (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_2 {
                    wallWalkAnim = BOTH_FORCEWALLRUNFLIP_START;
                    parts = SETANIM_BOTH;
                //kick = qfalse;
                } else if (*(*pmv).ps).weaponTime == 0 {
                    parts = SETANIM_BOTH;
                }
                //if ( PM_HasAnimation( pm->gent, wallWalkAnim ) )
                if true
                //sure, we have it! Because I SAID SO.
                {
                    let mut fwd: vec3_t = [0.0; 3];
                    let mut traceto: vec3_t = [0.0; 3];
                    let mut mins: vec3_t = [0.0; 3];
                    let mut maxs: vec3_t = [0.0; 3];
                    let mut fwdAngles: vec3_t = [0.0; 3];
                    let mut trace: trace_t = core::mem::zeroed();
                    let mut idealNormal: vec3_t = [0.0; 3];
                    let traceEnt: *mut bgEntity_t;

                    VectorSet(&mut mins, (*pmv).mins[0], (*pmv).mins[1], 0.0);
                    VectorSet(&mut maxs, (*pmv).maxs[0], (*pmv).maxs[1], 24.0);
                    VectorSet(&mut fwdAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

                    AngleVectors(&fwdAngles, Some(&mut fwd), None, None);
                    VectorMA(&(*(*pmv).ps).origin, 32.0, &fwd, &mut traceto);

                    ((*pmv).trace.unwrap())(
                        &mut trace,
                        (*(*pmv).ps).origin.as_ptr(),
                        mins.as_ptr(),
                        maxs.as_ptr(),
                        traceto.as_ptr(),
                        (*(*pmv).ps).clientNum,
                        contents,
                    ); //FIXME: clip brushes too?
                    VectorSubtract(&(*(*pmv).ps).origin, &traceto, &mut idealNormal);
                    VectorNormalize(&mut idealNormal);
                    traceEnt = PM_BGEntForNum(trace.entityNum as c_int);

                    if trace.fraction < 1.0
                        && (((trace.entityNum as c_int) < ENTITYNUM_WORLD
                            && !traceEnt.is_null()
                            && (*traceEnt).s.solid != SOLID_BMODEL)
                            || DotProduct(&trace.plane.normal, &idealNormal) > 0.7)
                    {
                        //there is a wall there
                        (*(*pmv).ps).velocity[1] = 0.0;
                        (*(*pmv).ps).velocity[0] = 0.0;
                        if wallWalkAnim == BOTH_FORCEWALLRUNFLIP_START {
                            (*(*pmv).ps).velocity[2] = forceJumpStrength[FORCE_LEVEL_3 as usize] / 2.0;
                        } else {
                            let v_snap = (*(*pmv).ps).velocity;
                            VectorMA(&v_snap, -150.0, &fwd, &mut (*(*pmv).ps).velocity);
                            (*(*pmv).ps).velocity[2] += 150.0;
                        }
                        //animate me
                        PM_SetAnim(parts, wallWalkAnim, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD, 0);
                        //						pm->ps->pm_flags |= PMF_JUMPING|PMF_SLOW_MO_FALL;
                        //again with the flags!
                        //G_SoundOnEnt( pm->gent, CHAN_BODY, "sound/weapons/force/jump.wav" );
                        //yucky!
                        PM_SetForceJumpZStart((*(*pmv).ps).origin[2]); //so we don't take damage if we land at same height
                        (*pmv).cmd.upmove = 0;
                        (*(*pmv).ps).fd.forceJumpSound = 1;
                        BG_ForcePowerDrain((*pmv).ps, FP_LEVITATION, 5);

                        //kick if jumping off an ent
                        /*
                        if ( kick && traceEnt && (traceEnt->s.eType == ET_PLAYER || traceEnt->s.eType == ET_NPC) )
                        { //kick that thang!
                            pm->ps->forceKickFlip = traceEnt->s.number+1;
                        }
                        */
                        (*pmv).cmd.rightmove = 0;
                        (*pmv).cmd.forwardmove = 0;
                    }
                }
                }
            } else if (BG_InSpecialJump(legsAnim) == QFALSE//not in a special jump anim
                    ||BG_InReboundJump(legsAnim) != QFALSE//we're already in a rebound
                    ||BG_InBackFlip(legsAnim) != QFALSE)//a backflip (needed so you can jump off a wall behind you)
                //&& pm->ps->velocity[2] <= 0
                && (*(*pmv).ps).velocity[2] > -1200.0 //not falling down very fast
                && ((*(*pmv).ps).pm_flags & PMF_JUMP_HELD) == 0//have to have released jump since last press
                && ((*pmv).cmd.forwardmove != 0 || (*pmv).cmd.rightmove != 0)//pushing in a direction
                //&& pm->ps->forceRageRecoveryTime < pm->cmd.serverTime	//not in a force Rage recovery period
                && (*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] > FORCE_LEVEL_2//level 3 jump or better
                //&& WP_ForcePowerAvailable( pm->gent, FP_LEVITATION, 10 )//have enough force power to do another one
                && BG_CanUseFPNow((*pmv).gametype, (*pmv).ps, (*pmv).cmd.serverTime, FP_LEVITATION) != QFALSE
                && ((*(*pmv).ps).origin[2]-(*(*pmv).ps).fd.forceJumpZStart) < (forceJumpHeightMax[FORCE_LEVEL_3 as usize]-(BG_ForceWallJumpStrength()/2.0))
            //can fit at least one more wall jump in (yes, using "magic numbers"... for now)
            //&& (pm->ps->legsAnim == BOTH_JUMP1 || pm->ps->legsAnim == BOTH_INAIR1 ) )//not in a flip or spin or anything
            {
                //see if we're pushing at a wall and jump off it if so
                if allowWallGrabs != 0 {
                //FIXME: make sure we have enough force power
                //FIXME: check  to see if we can go any higher
                //FIXME: limit to a certain number of these in a row?
                //FIXME: maybe don't require a ucmd direction, just check all 4?
                //FIXME: should stick to the wall for a second, then push off...
                let mut checkDir: vec3_t = [0.0; 3];
                let mut traceto: vec3_t = [0.0; 3];
                let mut mins: vec3_t = [0.0; 3];
                let mut maxs: vec3_t = [0.0; 3];
                let mut fwdAngles: vec3_t = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();
                let mut idealNormal: vec3_t = [0.0; 3];
                let mut anim: c_int = -1;

                VectorSet(&mut mins, (*pmv).mins[0], (*pmv).mins[1], 0.0);
                VectorSet(&mut maxs, (*pmv).maxs[0], (*pmv).maxs[1], 24.0);
                VectorSet(&mut fwdAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

                if (*pmv).cmd.rightmove != 0 {
                    if (*pmv).cmd.rightmove > 0 {
                        anim = BOTH_FORCEWALLREBOUND_RIGHT;
                        AngleVectors(&fwdAngles, None, Some(&mut checkDir), None);
                    } else if (*pmv).cmd.rightmove < 0 {
                        anim = BOTH_FORCEWALLREBOUND_LEFT;
                        AngleVectors(&fwdAngles, None, Some(&mut checkDir), None);
                        let c_snap = checkDir;
                        VectorScale(&c_snap, -1.0, &mut checkDir);
                    }
                } else if (*pmv).cmd.forwardmove > 0 {
                    anim = BOTH_FORCEWALLREBOUND_FORWARD;
                    AngleVectors(&fwdAngles, Some(&mut checkDir), None, None);
                } else if (*pmv).cmd.forwardmove < 0 {
                    anim = BOTH_FORCEWALLREBOUND_BACK;
                    AngleVectors(&fwdAngles, Some(&mut checkDir), None, None);
                    let c_snap = checkDir;
                    VectorScale(&c_snap, -1.0, &mut checkDir);
                }
                if anim != -1 {
                    //trace in the dir we're pushing in and see if there's a vertical wall there
                    let traceEnt: *mut bgEntity_t;

                    VectorMA(&(*(*pmv).ps).origin, 8.0, &checkDir, &mut traceto);
                    ((*pmv).trace.unwrap())(
                        &mut trace,
                        (*(*pmv).ps).origin.as_ptr(),
                        mins.as_ptr(),
                        maxs.as_ptr(),
                        traceto.as_ptr(),
                        (*(*pmv).ps).clientNum,
                        CONTENTS_SOLID,
                    ); //FIXME: clip brushes too?
                    VectorSubtract(&(*(*pmv).ps).origin, &traceto, &mut idealNormal);
                    VectorNormalize(&mut idealNormal);
                    traceEnt = PM_BGEntForNum(trace.entityNum as c_int);
                    if trace.fraction < 1.0
                        && trace.plane.normal[2].abs() <= 0.2/*MAX_WALL_GRAB_SLOPE*/
                        && (((trace.entityNum as c_int) < ENTITYNUM_WORLD
                            && !traceEnt.is_null()
                            && (*traceEnt).s.solid != SOLID_BMODEL)
                            || DotProduct(&trace.plane.normal, &idealNormal) > 0.7)
                    {
                        //there is a wall there
                        let dot = DotProduct(&(*(*pmv).ps).velocity, &trace.plane.normal);
                        if dot < 1.0 {
                            //can't be heading *away* from the wall!
                            //grab it!
                            PM_GrabWallForJump(anim);
                        }
                    }
                }
                }
            } else {
                //FIXME: if in a butterfly, kick people away?
            }
            //END NEW JKA
        }
    }

    /*
    if ( pm->cmd.upmove > 0
        && (pm->ps->weapon == WP_SABER || pm->ps->weapon == WP_MELEE)
        && !PM_IsRocketTrooper()
        && (pm->ps->weaponTime > 0||pm->cmd.buttons&BUTTON_ATTACK) )
    {//okay, we just jumped and we're in an attack
        if ( !BG_InRoll( pm->ps, pm->ps->legsAnim )
            && !PM_InKnockDown( pm->ps )
            && !BG_InDeathAnim(pm->ps->legsAnim)
            && !BG_FlippingAnim( pm->ps->legsAnim )
            && !PM_SpinningAnim( pm->ps->legsAnim )
            && !BG_SaberInSpecialAttack( pm->ps->torsoAnim )
            && ( BG_SaberInAttack( pm->ps->saberMove ) ) )
        {//not in an anim we shouldn't interrupt
            //see if it's not too late to start a special jump-attack
            float animLength = PM_AnimLength( 0, (animNumber_t)pm->ps->torsoAnim );
            if ( animLength - pm->ps->torsoTimer < 500 )
            {//just started the saberMove
                //check for special-case jump attacks
                if ( pm->ps->fd.saberAnimLevel == FORCE_LEVEL_2 )
                {//using medium attacks
                    if (PM_GroundDistance() < 32 &&
                        !BG_InSpecialJump(pm->ps->legsAnim))
                    { //FLIP AND DOWNWARD ATTACK
                        //trace_t tr;

                        //if (PM_SomeoneInFront(&tr))
                        {
                            PM_SetSaberMove(PM_SaberFlipOverAttackMove());
                            pml.groundPlane = qfalse;
                            pml.walking = qfalse;
                            pm->ps->pm_flags |= PMF_JUMP_HELD;
                            pm->ps->groundEntityNum = ENTITYNUM_NONE;
                            VectorClear(pml.groundTrace.plane.normal);

                            pm->ps->weaponTime = pm->ps->torsoTimer;
                        }
                    }
                }
                else if ( pm->ps->fd.saberAnimLevel == FORCE_LEVEL_3 )
                {//using strong attacks
                    if ( pm->cmd.forwardmove > 0 && //going forward
                        (pm->cmd.buttons & BUTTON_ATTACK) && //must be holding attack still
                        PM_GroundDistance() < 32 &&
                        !BG_InSpecialJump(pm->ps->legsAnim))
                    {//strong attack: jump-hack
                        PM_SetSaberMove( PM_SaberJumpAttackMove() );
                        pml.groundPlane = qfalse;
                        pml.walking = qfalse;
                        pm->ps->pm_flags |= PMF_JUMP_HELD;
                        pm->ps->groundEntityNum = ENTITYNUM_NONE;
                        VectorClear(pml.groundTrace.plane.normal);

                        pm->ps->weaponTime = pm->ps->torsoTimer;
                    }
                }
            }
        }
    }
    */
    if (*(*pmv).ps).groundEntityNum == ENTITYNUM_NONE {
        return QFALSE;
    }
    if (*pmv).cmd.upmove > 0 {
        //no special jumps
        (*(*pmv).ps).velocity[2] = JUMP_VELOCITY as f32;
        PM_SetForceJumpZStart((*(*pmv).ps).origin[2]); //so we don't take damage if we land at same height
        (*(*pmv).ps).pm_flags |= PMF_JUMP_HELD;
    }

    //Jumping
    (*addr_of_mut!(pml)).groundPlane = QFALSE;
    (*addr_of_mut!(pml)).walking = QFALSE;
    (*(*pmv).ps).pm_flags |= PMF_JUMP_HELD;
    (*(*pmv).ps).groundEntityNum = ENTITYNUM_NONE;
    PM_SetForceJumpZStart((*(*pmv).ps).origin[2]);

    PM_AddEvent(EV_JUMP);

    //Set the animations
    if (*(*pmv).ps).gravity > 0 && BG_InSpecialJump((*(*pmv).ps).legsAnim) == QFALSE {
        PM_JumpForDir();
    }

    QTRUE
}

/*
===================
PM_CheckWaterJump
===================
*/
/// `PM_CheckWaterJump` (bg_pmove.c:2648) — at a water/wall boundary (waterlevel 2, solid
/// directly ahead, clear just above it), launch the player up and out of the water and
/// arm the `PMF_TIME_WATERJUMP` timer. `static`→`pub`.
///
/// No oracle: driven by the `pm->pointcontents` engine callback (the `PM_SetWaterLevel`
/// precedent).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `pointcontents` callback.
pub unsafe fn PM_CheckWaterJump() -> qboolean {
    let pmv = *addr_of!(pm);
    let mut spot: vec3_t = [0.0; 3];
    let mut cont: c_int;
    let mut flatforward: vec3_t = [0.0; 3];

    if (*(*pmv).ps).pm_time != 0 {
        return QFALSE;
    }

    // check for water jump
    if (*pmv).waterlevel != 2 {
        return QFALSE;
    }

    flatforward[0] = (*addr_of!(pml)).forward[0];
    flatforward[1] = (*addr_of!(pml)).forward[1];
    flatforward[2] = 0.0;
    VectorNormalize(&mut flatforward);

    VectorMA(&(*(*pmv).ps).origin, 30.0, &flatforward, &mut spot);
    spot[2] += 4.0;
    cont = ((*pmv).pointcontents.unwrap())(spot.as_ptr(), (*(*pmv).ps).clientNum);
    if cont & CONTENTS_SOLID == 0 {
        return QFALSE;
    }

    spot[2] += 16.0;
    cont = ((*pmv).pointcontents.unwrap())(spot.as_ptr(), (*(*pmv).ps).clientNum);
    if cont != 0 {
        return QFALSE;
    }

    // jump out of water
    {
        // VectorScale(pml.forward, 200, ps->velocity) — distinct src/dst.
        let mut scaled: vec3_t = [0.0; 3];
        VectorScale(&(*addr_of!(pml)).forward, 200.0, &mut scaled);
        (*(*pmv).ps).velocity = scaled;
    }
    (*(*pmv).ps).velocity[2] = 350.0;

    (*(*pmv).ps).pm_flags |= PMF_TIME_WATERJUMP;
    (*(*pmv).ps).pm_time = 2000;

    QTRUE
}

/*
===================
PM_WaterJumpMove

Flowing water
===================
*/
/// `PM_WaterJumpMove` (bg_pmove.c:2700) — a water-jump has no control but still falls:
/// step-slide forward, apply gravity, and cancel the timed water-jump flags the moment
/// downward velocity resumes. `static`→`pub`.
///
/// No oracle: a trace/callback-driven move-fn (composes the already-landed
/// `PM_StepSlideMove`); behaviourally verified transitively once `Pmove` lands.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps` + `trace` callback.
pub unsafe fn PM_WaterJumpMove() {
    let pmv = *addr_of!(pm);
    // waterjump has no control, but falls

    PM_StepSlideMove(QTRUE);

    (*(*pmv).ps).velocity[2] -= (*(*pmv).ps).gravity as f32 * (*addr_of!(pml)).frametime;
    if (*(*pmv).ps).velocity[2] < 0.0 {
        // cancel as soon as we are falling down again
        (*(*pmv).ps).pm_flags &= !PMF_ALL_TIMES;
        (*(*pmv).ps).pm_time = 0;
    }
}

/*
===================
PM_WaterMove

===================
*/
/// `PM_WaterMove` (bg_pmove.c:2719) — underwater locomotion: divert to the water-jump
/// move when climbing out, otherwise apply friction, build a wish-velocity (sinking when
/// idle), clamp to the swim speed, accelerate, slide along the ground plane to climb
/// slopes, and integrate via `PM_SlideMove`. `static`→`pub`.
///
/// No oracle: a trace/callback-driven move-fn (composes the already-verified
/// `PM_Friction`/`PM_CmdScale`/`PM_Accelerate`/`PM_ClipVelocity` + the landed
/// `PM_SlideMove`); behaviourally verified transitively once `Pmove` lands.
///
/// # Safety
/// `pm`/`pml` must be valid (live `ps` + `trace`).
pub unsafe fn PM_WaterMove() {
    let pmv = *addr_of!(pm);
    let mut wishvel: vec3_t = [0.0; 3];
    let mut wishspeed: f32;
    let mut wishdir: vec3_t = [0.0; 3];
    let scale: f32;
    let vel: f32;

    if PM_CheckWaterJump() != QFALSE {
        PM_WaterJumpMove();
        return;
    }
    // #if 0 — disabled "jump = head for surface" upmove boost (carried verbatim, never built):
    //   if ( pm->cmd.upmove >= 10 ) {
    //       if (pm->ps->velocity[2] > -300) {
    //           if ( pm->watertype == CONTENTS_WATER )      pm->ps->velocity[2] = 100;
    //           else if (pm->watertype == CONTENTS_SLIME)   pm->ps->velocity[2] = 80;
    //           else                                        pm->ps->velocity[2] = 50;
    //       }
    //   }
    // #endif
    PM_Friction();

    scale = PM_CmdScale(addr_of_mut!((*pmv).cmd));
    //
    // user intentions
    //
    if scale == 0.0 {
        wishvel[0] = 0.0;
        wishvel[1] = 0.0;
        wishvel[2] = -60.0; // sink towards bottom
    } else {
        for i in 0..3 {
            wishvel[i] = scale * (*addr_of!(pml)).forward[i] * (*pmv).cmd.forwardmove as f32
                + scale * (*addr_of!(pml)).right[i] * (*pmv).cmd.rightmove as f32;
        }

        wishvel[2] += scale * (*pmv).cmd.upmove as f32;
    }

    VectorCopy(&wishvel, &mut wishdir);
    wishspeed = VectorNormalize(&mut wishdir);

    if wishspeed > (*(*pmv).ps).speed * pm_swimScale {
        wishspeed = (*(*pmv).ps).speed * pm_swimScale;
    }

    PM_Accelerate(wishdir.as_mut_ptr(), wishspeed, pm_wateraccelerate);

    // make sure we can go up slopes easily under water
    if (*addr_of!(pml)).groundPlane != QFALSE
        && DotProduct(
            &(*(*pmv).ps).velocity,
            &(*addr_of!(pml)).groundTrace.plane.normal,
        ) < 0.0
    {
        vel = VectorLength(&(*(*pmv).ps).velocity);
        // slide along the ground plane (in==out velocity: PM_ClipVelocity reads each
        // in[i] before writing out[i], so the same pointer is safe).
        PM_ClipVelocity(
            (*(*pmv).ps).velocity.as_mut_ptr(),
            (*addr_of!(pml)).groundTrace.plane.normal.as_ptr() as *mut _,
            (*(*pmv).ps).velocity.as_mut_ptr(),
            OVERCLIP,
        );

        VectorNormalize(&mut (*(*pmv).ps).velocity);
        // VectorScale(velocity, vel, velocity) — distinct src/dst to avoid aliasing.
        let mut scaled: vec3_t = [0.0; 3];
        VectorScale(&(*(*pmv).ps).velocity, vel, &mut scaled);
        (*(*pmv).ps).velocity = scaled;
    }

    PM_SlideMove(QFALSE);
}

/*
===================
PM_FlyVehicleMove

===================
*/
/// `PM_FlyVehicleMove` (bg_pmove.c:2791) — flight move for a ridden/NPC flying vehicle:
/// apply friction (preserving downward velocity while falling, killing it on the ground),
/// derive the wish-velocity from the pre-computed `moveDir` (player input is intentionally
/// not used here — see the C note), normalise, accelerate hard, and step-slide with
/// gravity. `static`→`pub`.
///
/// No oracle: a trace/callback-driven vehicle move-fn; behaviourally verified transitively
/// once `Pmove` lands. `fmove`/`smove` stay `0.0` (the UCmd-driven wish-dir branch is dead
/// because of that, but carried faithfully).
///
/// # Safety
/// `pm`/`pml` must be valid (live `ps` + `trace`).
// The negative-speed `wishspeed *= -1` is immediately overwritten by the final
// `VectorNormalize`, dead in the C too — faithful redundancy, hence the allow.
#[allow(unused_assignments)]
pub unsafe fn PM_FlyVehicleMove() {
    let pmv = *addr_of!(pm);
    let mut wishvel: vec3_t = [0.0; 3];
    let mut wishspeed: f32;
    let mut wishdir: vec3_t = [0.0; 3];
    let scale: f32;
    let zVel: f32;
    let fmove: f32 = 0.0;
    let smove: f32 = 0.0;

    // We don't use these here because we pre-calculate the movedir in the vehicle update anyways, and if
    // you leave this, you get strange motion during boarding (the player can move the vehicle).
    //fmove = pm->cmd.forwardmove;
    //smove = pm->cmd.rightmove;

    // normal slowdown
    if (*(*pmv).ps).gravity != 0
        && (*(*pmv).ps).velocity[2] < 0.0
        && (*(*pmv).ps).groundEntityNum == ENTITYNUM_NONE
    {
        //falling
        zVel = (*(*pmv).ps).velocity[2];
        PM_Friction();
        (*(*pmv).ps).velocity[2] = zVel;
    } else {
        PM_Friction();
        if (*(*pmv).ps).velocity[2] < 0.0 && (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE {
            (*(*pmv).ps).velocity[2] = 0.0; // ignore slope movement
        }
    }

    scale = PM_CmdScale(addr_of_mut!((*pmv).cmd));

    // Get The WishVel And WishSpeed
    //-------------------------------
    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int {
        //NPC

        // If The UCmds Were Set, But Never Converted Into A MoveDir, Then Make The WishDir From UCmds
        //--------------------------------------------------------------------------------------------
        if (fmove != 0.0 || smove != 0.0)
            && VectorCompare(&(*(*pmv).ps).moveDir, &vec3_origin) != 0
        {
            //gi.Printf("Generating MoveDir\n");
            for i in 0..3 {
                wishvel[i] = (*addr_of!(pml)).forward[i] * fmove + (*addr_of!(pml)).right[i] * smove;
            }

            VectorCopy(&wishvel, &mut wishdir);
            wishspeed = VectorNormalize(&mut wishdir);
            wishspeed *= scale;
        }
        // Otherwise, Use The Move Dir
        //-----------------------------
        else {
            wishspeed = (*(*pmv).ps).speed;
            VectorScale(&(*(*pmv).ps).moveDir, (*(*pmv).ps).speed, &mut wishvel);
            VectorCopy(&(*(*pmv).ps).moveDir, &mut wishdir);
        }
    } else {
        for i in 0..3 {
            wishvel[i] = (*addr_of!(pml)).forward[i] * fmove + (*addr_of!(pml)).right[i] * smove;
        }
        // when going up or down slopes the wish velocity should Not be zero
        //	wishvel[2] = 0;

        VectorCopy(&wishvel, &mut wishdir);
        wishspeed = VectorNormalize(&mut wishdir);
        wishspeed *= scale;
    }

    // Handle negative speed.
    if wishspeed < 0.0 {
        wishspeed *= -1.0;
        // VectorScale(wishvel, -1, wishvel) — in==out, snapshot to avoid aliasing.
        let mut neg: vec3_t = [0.0; 3];
        VectorScale(&wishvel, -1.0, &mut neg);
        wishvel = neg;
        VectorScale(&wishdir, -1.0, &mut neg);
        wishdir = neg;
    }

    VectorCopy(&wishvel, &mut wishdir);
    wishspeed = VectorNormalize(&mut wishdir);

    PM_Accelerate(wishdir.as_mut_ptr(), wishspeed, 100.0);

    PM_StepSlideMove(QTRUE);
}

/*
===================
PM_FlyMove

Only with the flight powerup
===================
*/
/// `PM_FlyMove` (bg_pmove.c:2888) — flight-powerup / spectator free flight: friction,
/// `PM_CmdScale` (with the spectator alt-attack turbo boost), a wish-velocity from the
/// `pml` basis + up-move, accelerate, then step-slide without gravity. `static`→`pub`.
///
/// No oracle: a trace/callback-driven move-fn; behaviourally verified transitively once
/// `Pmove` lands.
///
/// # Safety
/// `pm`/`pml` must be valid (live `ps` + `trace`).
pub unsafe fn PM_FlyMove() {
    let pmv = *addr_of!(pm);
    let mut wishvel: vec3_t = [0.0; 3];
    let wishspeed: f32;
    let mut wishdir: vec3_t = [0.0; 3];
    let mut scale: f32;

    // normal slowdown
    PM_Friction();

    scale = PM_CmdScale(addr_of_mut!((*pmv).cmd));

    if (*(*pmv).ps).pm_type == PM_SPECTATOR && (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
        //turbo boost
        scale *= 10.0;
    }

    //
    // user intentions
    //
    if scale == 0.0 {
        wishvel[0] = 0.0;
        wishvel[1] = 0.0;
        wishvel[2] = (*(*pmv).ps).speed * ((*pmv).cmd.upmove as f32 / 127.0);
    } else {
        for i in 0..3 {
            wishvel[i] = scale * (*addr_of!(pml)).forward[i] * (*pmv).cmd.forwardmove as f32
                + scale * (*addr_of!(pml)).right[i] * (*pmv).cmd.rightmove as f32;
        }

        wishvel[2] += scale * (*pmv).cmd.upmove as f32;
    }

    VectorCopy(&wishvel, &mut wishdir);
    wishspeed = VectorNormalize(&mut wishdir);

    PM_Accelerate(wishdir.as_mut_ptr(), wishspeed, pm_flyaccelerate);

    PM_StepSlideMove(QFALSE);
}

/*
===================
PM_AirMove

===================
*/
/// `PM_AirMove` (bg_pmove.c:2935) — airborne locomotion: maybe continue a force-jump,
/// apply friction, project the move basis flat, build a (vehicle-aware) wish-velocity,
/// accelerate with the air-accel (or vehicle traction), clip against a steep ground plane,
/// then step-slide (with gravity unless stuck to a wall). `static`→`pub`.
///
/// No oracle: a trace/callback-driven move-fn (composes `PM_CheckJump`/`PM_Friction`/
/// `PM_CmdScale`/`PM_SetMovementDir`/`PM_Accelerate`/`PM_GroundSlideOkay` + the landed
/// `PM_StepSlideMove`); behaviourally verified transitively once `Pmove` lands. The
/// hovercraft-control and reduced-strafe `#if 0` block is carried as a comment.
///
/// # Safety
/// `pm`/`pml`/`pm_entSelf` and any vehicle pointer chain must be valid (live `ps` + `trace`).
// The hover-branch `wishspeed = ps->speed` is overwritten by the unconditional final
// `VectorNormalize`, dead in the C too — faithful redundancy, hence the allow.
#[allow(unused_assignments)]
pub unsafe fn PM_AirMove() {
    let pmv = *addr_of!(pm);
    let mut wishvel: vec3_t = [0.0; 3];
    let fmove: f32;
    let smove: f32;
    let mut wishdir: vec3_t = [0.0; 3];
    let mut wishspeed: f32;
    let mut scale: f32;
    let mut accelerate: f32;
    let mut cmd: usercmd_t;
    let mut pVeh: *mut Vehicle_t = null_mut();

    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int {
        let pEnt: *mut bgEntity_t = *addr_of!(pm_entSelf);

        if !pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_VEHICLE {
            pVeh = (*pEnt).m_pVehicle;
        }
    }

    if (*(*pmv).ps).pm_type != PM_SPECTATOR {
        // #if METROID_JUMP (defined 1) — the #else `forceJumpZStart && forceJumpFlip`
        // guard is dead and not carried.
        PM_CheckJump();
    }
    PM_Friction();

    fmove = (*pmv).cmd.forwardmove as f32;
    smove = (*pmv).cmd.rightmove as f32;

    cmd = (*pmv).cmd;
    scale = PM_CmdScale(addr_of_mut!(cmd));

    // set the movementDir so clients can rotate the legs for strafing
    PM_SetMovementDir();

    // project moves down to flat plane
    (*addr_of_mut!(pml)).forward[2] = 0.0;
    (*addr_of_mut!(pml)).right[2] = 0.0;
    VectorNormalize(&mut (*addr_of_mut!(pml)).forward);
    VectorNormalize(&mut (*addr_of_mut!(pml)).right);

    if !pVeh.is_null() && (*(*pVeh).m_pVehicleInfo).hoverHeight > 0.0 {
        //in a hovering vehicle, have air control
        // C source wraps the body in a redundant `if ( 1 )`; the `#if 0` else-branch
        // (slope-aware hovercraft / strafe control) is dead and carried only as a comment.
        wishspeed = (*(*pmv).ps).speed;
        VectorScale(&(*(*pmv).ps).moveDir, (*(*pmv).ps).speed, &mut wishvel);
        VectorCopy(&(*(*pmv).ps).moveDir, &mut wishdir);
        scale = 1.0;
    } else if *addr_of!(gPMDoSlowFall) != QFALSE {
        //no air-control
        VectorClear(&mut wishvel);
    } else if (*(*pmv).ps).pm_type == PM_JETPACK {
        //reduced air control while not jetting
        for i in 0..2 {
            wishvel[i] = (*addr_of!(pml)).forward[i] * fmove + (*addr_of!(pml)).right[i] * smove;
        }
        wishvel[2] = 0.0;

        if (*pmv).cmd.upmove <= 0 {
            // VectorScale(wishvel, 0.8, wishvel) — in==out, snapshot.
            let mut scaled: vec3_t = [0.0; 3];
            VectorScale(&wishvel, 0.8, &mut scaled);
            wishvel = scaled;
        } else {
            //if we are jetting then we have more control than usual
            let mut scaled: vec3_t = [0.0; 3];
            VectorScale(&wishvel, 2.0, &mut scaled);
            wishvel = scaled;
        }
    } else {
        for i in 0..2 {
            wishvel[i] = (*addr_of!(pml)).forward[i] * fmove + (*addr_of!(pml)).right[i] * smove;
        }
        wishvel[2] = 0.0;
    }

    VectorCopy(&wishvel, &mut wishdir);
    wishspeed = VectorNormalize(&mut wishdir);
    wishspeed *= scale;

    accelerate = pm_airaccelerate;
    if !pVeh.is_null() && (*(*pVeh).m_pVehicleInfo).r#type == VH_SPEEDER {
        //speeders have more control in air
        //in mid-air
        accelerate = (*(*pVeh).m_pVehicleInfo).traction;
        if (*addr_of!(pml)).groundPlane != QFALSE {
            //on a slope of some kind, shouldn't have much control and should slide a lot
            accelerate *= 0.5;
        }
    }
    // not on ground, so little effect on velocity
    PM_Accelerate(wishdir.as_mut_ptr(), wishspeed, accelerate);

    // we may have a ground plane that is very steep, even
    // though we don't have a groundentity
    // slide along the steep plane
    if (*addr_of!(pml)).groundPlane != QFALSE {
        if (*(*pmv).ps).pm_flags & PMF_STUCK_TO_WALL == 0 {
            //don't slide when stuck to a wall
            if PM_GroundSlideOkay((*addr_of!(pml)).groundTrace.plane.normal[2]) != QFALSE {
                PM_ClipVelocity(
                    (*(*pmv).ps).velocity.as_mut_ptr(),
                    (*addr_of!(pml)).groundTrace.plane.normal.as_ptr() as *mut _,
                    (*(*pmv).ps).velocity.as_mut_ptr(),
                    OVERCLIP,
                );
            }
        }
    }

    if (*(*pmv).ps).pm_flags & PMF_STUCK_TO_WALL != 0 {
        //no grav when stuck to wall
        PM_StepSlideMove(QFALSE);
    } else {
        PM_StepSlideMove(QTRUE);
    }
}

/*
===================
PM_WalkMove

===================
*/
/// `PM_WalkMove` (bg_pmove.c:3172) — grounded locomotion: divert to swim/air on
/// water-surface/jump, apply friction, project the move basis onto the ground plane,
/// build a (vehicle-aware) wish-velocity, clamp for ducking/rolling/wading, accelerate,
/// preserve speed along slopes, and step-slide. `static`→`pub`.
///
/// No oracle: a trace/callback-driven move-fn (composes the verified
/// friction/cmd-scale/accelerate/clip helpers + the landed slide integrators and
/// `PM_WaterMove`/`PM_AirMove`); behaviourally verified transitively once `Pmove` lands.
///
/// **Precision:** the wading `waterScale` block is computed in `f64` (matching the C
/// `pm->waterlevel / 3.0` and `1.0 - (1.0 - pm_swimScale) * waterScale` double promotions,
/// per the `PM_NoclipMove` `pm_friction*1.5` precedent), then narrowed to `f32`.
///
/// # Safety
/// `pm`/`pml`/`pm_entSelf` and any vehicle pointer chain must be valid (live `ps` + `trace`).
pub unsafe fn PM_WalkMove() {
    let pmv = *addr_of!(pm);
    let mut wishvel: vec3_t = [0.0; 3];
    let fmove: f32;
    let smove: f32;
    let mut wishdir: vec3_t = [0.0; 3];
    let mut wishspeed: f32 = 0.0;
    let scale: f32;
    let mut cmd: usercmd_t;
    let accelerate: f32;
    let vel: f32;
    let mut npcMovement: qboolean = QFALSE;

    if (*pmv).waterlevel > 2
        && DotProduct(
            &(*addr_of!(pml)).forward,
            &(*addr_of!(pml)).groundTrace.plane.normal,
        ) > 0.0
    {
        // begin swimming
        PM_WaterMove();
        return;
    }

    if (*(*pmv).ps).pm_type != PM_SPECTATOR {
        if PM_CheckJump() != QFALSE {
            // jumped away
            if (*pmv).waterlevel > 1 {
                PM_WaterMove();
            } else {
                PM_AirMove();
            }
            return;
        }
    }

    PM_Friction();

    fmove = (*pmv).cmd.forwardmove as f32;
    smove = (*pmv).cmd.rightmove as f32;

    cmd = (*pmv).cmd;
    scale = PM_CmdScale(addr_of_mut!(cmd));

    // set the movementDir so clients can rotate the legs for strafing
    PM_SetMovementDir();

    // project moves down to flat plane
    (*addr_of_mut!(pml)).forward[2] = 0.0;
    (*addr_of_mut!(pml)).right[2] = 0.0;

    // project the forward and right directions onto the ground plane
    // (in==out forward/right: PM_ClipVelocity reads each in[i] before writing out[i]).
    PM_ClipVelocity(
        (*addr_of_mut!(pml)).forward.as_mut_ptr(),
        (*addr_of!(pml)).groundTrace.plane.normal.as_ptr() as *mut _,
        (*addr_of_mut!(pml)).forward.as_mut_ptr(),
        OVERCLIP,
    );
    PM_ClipVelocity(
        (*addr_of_mut!(pml)).right.as_mut_ptr(),
        (*addr_of!(pml)).groundTrace.plane.normal.as_ptr() as *mut _,
        (*addr_of_mut!(pml)).right.as_mut_ptr(),
        OVERCLIP,
    );
    //
    VectorNormalize(&mut (*addr_of_mut!(pml)).forward);
    VectorNormalize(&mut (*addr_of_mut!(pml)).right);

    // Get The WishVel And WishSpeed
    //-------------------------------
    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
        && VectorCompare(&(*(*pmv).ps).moveDir, &vec3_origin) == 0
    {
        //NPC
        let pEnt: *mut bgEntity_t = *addr_of!(pm_entSelf);

        if !pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_VEHICLE {
            // If The UCmds Were Set, But Never Converted Into A MoveDir, Then Make The WishDir From UCmds
            //--------------------------------------------------------------------------------------------
            if (fmove != 0.0 || smove != 0.0)
                && VectorCompare(&(*(*pmv).ps).moveDir, &vec3_origin) != 0
            {
                //gi.Printf("Generating MoveDir\n");
                for i in 0..3 {
                    wishvel[i] =
                        (*addr_of!(pml)).forward[i] * fmove + (*addr_of!(pml)).right[i] * smove;
                }

                VectorCopy(&wishvel, &mut wishdir);
                wishspeed = VectorNormalize(&mut wishdir);
                wishspeed *= scale;
            }
            // Otherwise, Use The Move Dir
            //-----------------------------
            else {
                //wishspeed = pm->ps->speed;
                VectorScale(&(*(*pmv).ps).moveDir, (*(*pmv).ps).speed, &mut wishvel);
                VectorCopy(&wishvel, &mut wishdir);
                wishspeed = VectorNormalize(&mut wishdir);
            }

            npcMovement = QTRUE;
        }
    }

    if npcMovement == QFALSE {
        for i in 0..3 {
            wishvel[i] = (*addr_of!(pml)).forward[i] * fmove + (*addr_of!(pml)).right[i] * smove;
        }
        // when going up or down slopes the wish velocity should Not be zero

        VectorCopy(&wishvel, &mut wishdir);
        wishspeed = VectorNormalize(&mut wishdir);
        wishspeed *= scale;
    }

    // clamp the speed lower if ducking
    if (*(*pmv).ps).pm_flags & PMF_DUCKED != 0 {
        if wishspeed > (*(*pmv).ps).speed * pm_duckScale {
            wishspeed = (*(*pmv).ps).speed * pm_duckScale;
        }
    } else if (*(*pmv).ps).pm_flags & PMF_ROLLING != 0
        && BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) == QFALSE
        && PM_InRollComplete((*pmv).ps, (*(*pmv).ps).legsAnim) == QFALSE
    {
        if wishspeed > (*(*pmv).ps).speed * pm_duckScale {
            wishspeed = (*(*pmv).ps).speed * pm_duckScale;
        }
    }

    // clamp the speed lower if wading or walking on the bottom
    if (*pmv).waterlevel != 0 {
        // C computes in f64: `waterlevel / 3.0` and `1.0 - (1.0 - pm_swimScale)*waterScale`.
        let mut waterScale: f32;

        waterScale = ((*pmv).waterlevel as f64 / 3.0) as f32;
        waterScale = (1.0_f64 - (1.0_f64 - pm_swimScale as f64) * waterScale as f64) as f32;
        if wishspeed > (*(*pmv).ps).speed * waterScale {
            wishspeed = (*(*pmv).ps).speed * waterScale;
        }
    }

    // when a player gets hit, they temporarily lose
    // full control, which allows them to be moved a bit
    if *addr_of!(pm_flying) == FLY_HOVER {
        accelerate = pm_vehicleaccelerate;
    } else if (*addr_of!(pml)).groundTrace.surfaceFlags & SURF_SLICK != 0
        || (*(*pmv).ps).pm_flags & PMF_TIME_KNOCKBACK != 0
    {
        accelerate = pm_airaccelerate;
    } else {
        accelerate = pm_accelerate;
    }

    PM_Accelerate(wishdir.as_mut_ptr(), wishspeed, accelerate);
    /*
    if (pm->ps->clientNum >= MAX_CLIENTS) {
    #ifdef QAGAME
        Com_Printf("^1S: %f, %f\n", wishspeed, pm->ps->speed);
    #else
        Com_Printf("^2C: %f, %f\n", wishspeed, pm->ps->speed);
    #endif
    }
    */

    //Com_Printf("velocity = %1.1f %1.1f %1.1f\n", ...);
    //Com_Printf("velocity1 = %1.1f\n", VectorLength(pm->ps->velocity));

    if (*addr_of!(pml)).groundTrace.surfaceFlags & SURF_SLICK != 0
        || (*(*pmv).ps).pm_flags & PMF_TIME_KNOCKBACK != 0
    {
        (*(*pmv).ps).velocity[2] -= (*(*pmv).ps).gravity as f32 * (*addr_of!(pml)).frametime;
    }

    vel = VectorLength(&(*(*pmv).ps).velocity);

    // slide along the ground plane (in==out velocity)
    PM_ClipVelocity(
        (*(*pmv).ps).velocity.as_mut_ptr(),
        (*addr_of!(pml)).groundTrace.plane.normal.as_ptr() as *mut _,
        (*(*pmv).ps).velocity.as_mut_ptr(),
        OVERCLIP,
    );

    // don't decrease velocity when going up or down a slope
    VectorNormalize(&mut (*(*pmv).ps).velocity);
    // VectorScale(velocity, vel, velocity) — distinct src/dst to avoid aliasing.
    {
        let mut scaled: vec3_t = [0.0; 3];
        VectorScale(&(*(*pmv).ps).velocity, vel, &mut scaled);
        (*(*pmv).ps).velocity = scaled;
    }

    // don't do anything if standing still
    if (*(*pmv).ps).velocity[0] == 0.0 && (*(*pmv).ps).velocity[1] == 0.0 {
        return;
    }

    PM_StepSlideMove(QFALSE);

    //Com_Printf("velocity2 = %1.1f\n", VectorLength(pm->ps->velocity));
}

/*
===================
PM_DeadMove
===================
*/
/// `PM_DeadMove` (bg_pmove.c:3360) — apply extra friction to a dead body that's still
/// "walking": shave 20 u/s off the speed, zeroing it once it drops to zero. `static`→`pub`.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_DeadMove() {
    let pmv = *addr_of!(pm);
    let mut forward: f32;

    if (*addr_of!(pml)).walking == 0 {
        return;
    }

    // extra friction

    forward = VectorLength(&(*(*pmv).ps).velocity);
    forward -= 20.0;
    if forward <= 0.0 {
        VectorClear(&mut (*(*pmv).ps).velocity);
    } else {
        VectorNormalize(&mut (*(*pmv).ps).velocity);
        // VectorScale(velocity, forward, velocity) — distinct src/dst to avoid aliasing.
        let mut scaled: vec3_t = [0.0; 3];
        VectorScale(&(*(*pmv).ps).velocity, forward, &mut scaled);
        (*(*pmv).ps).velocity = scaled;
    }
}

/*
===============
PM_NoclipMove
===============
*/
/// `PM_NoclipMove` (bg_pmove.c:3385) — free-form spectator/noclip flight: extra friction,
/// `PM_CmdScale` + optional attack-button turbo boost, build a wish-velocity from the
/// `pml` basis + command, `PM_Accelerate`, then integrate the origin. `static`→`pub`.
///
/// No oracle: a move-integrator composing the already-verified `PM_CmdScale`/
/// `PM_Accelerate` over the engine-set `pml.forward`/`right` basis. The lone original
/// arithmetic (the `pm_friction*1.5` noclip friction) mirrors the oracle-verified
/// `PM_Friction`; the `*1.5` `f64` promotion is replicated.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_NoclipMove() {
    let pmv = *addr_of!(pm);
    let speed: f32;
    let mut drop: f32;
    let friction: f32;
    let control: f32;
    let mut newspeed: f32;
    let fmove: f32;
    let smove: f32;
    let mut wishvel: vec3_t = [0.0; 3];
    let mut wishdir: vec3_t = [0.0; 3];
    let mut wishspeed: f32;
    let mut scale: f32;

    (*(*pmv).ps).viewheight = DEFAULT_VIEWHEIGHT;

    // friction

    speed = VectorLength(&(*(*pmv).ps).velocity);
    if speed < 1.0 {
        VectorCopy(&vec3_origin, &mut (*(*pmv).ps).velocity);
    } else {
        drop = 0.0;

        // C: `pm_friction*1.5` promotes through f64 (the 1.5 double literal).
        friction = (pm_friction as f64 * 1.5) as f32; // extra friction
        control = if speed < pm_stopspeed { pm_stopspeed } else { speed };
        drop += control * friction * (*addr_of!(pml)).frametime;

        // scale the velocity
        newspeed = speed - drop;
        if newspeed < 0.0 {
            newspeed = 0.0;
        }
        newspeed /= speed;

        // VectorScale(velocity, newspeed, velocity) — distinct src/dst.
        let mut scaled: vec3_t = [0.0; 3];
        VectorScale(&(*(*pmv).ps).velocity, newspeed, &mut scaled);
        (*(*pmv).ps).velocity = scaled;
    }

    // accelerate
    scale = PM_CmdScale(addr_of_mut!((*pmv).cmd));
    if (*pmv).cmd.buttons & BUTTON_ATTACK != 0 {
        //turbo boost
        scale *= 10.0;
    }
    if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
        //turbo boost
        scale *= 10.0;
    }

    fmove = (*pmv).cmd.forwardmove as f32;
    smove = (*pmv).cmd.rightmove as f32;

    for i in 0..3 {
        wishvel[i] = (*addr_of!(pml)).forward[i] * fmove + (*addr_of!(pml)).right[i] * smove;
    }
    wishvel[2] += (*pmv).cmd.upmove as f32;

    VectorCopy(&wishvel, &mut wishdir);
    wishspeed = VectorNormalize(&mut wishdir);
    wishspeed *= scale;

    PM_Accelerate(wishdir.as_mut_ptr(), wishspeed, pm_accelerate);

    // move
    {
        // VectorMA(origin, frametime, velocity, origin) — distinct src/dst.
        let mut moved: vec3_t = [0.0; 3];
        VectorMA(
            &(*(*pmv).ps).origin,
            (*addr_of!(pml)).frametime,
            &(*(*pmv).ps).velocity,
            &mut moved,
        );
        (*(*pmv).ps).origin = moved;
    }
}

//============================================================================

/*
================
PM_FootstepForSurface

Returns an event number apropriate for the groundsurface
================
*/
/// `PM_FootstepForSurface` (bg_pmove.c:3455) — the footstep/material event for the
/// current ground surface: 0 when `SURF_NOSTEPS` is set, else the surface's material
/// index (`surfaceFlags & MATERIAL_MASK`). `static int`→`pub`.
pub unsafe fn PM_FootstepForSurface() -> c_int {
    if (*addr_of!(pml)).groundTrace.surfaceFlags & SURF_NOSTEPS != 0 {
        return 0;
    }
    (*addr_of!(pml)).groundTrace.surfaceFlags & MATERIAL_MASK
}

/// `PM_TryRoll` (bg_pmove.c:3465) — attempt a dodge-roll in the command's movement
/// direction: bail if mid-attack/spin (unless soul-cal allows), if not on saber/melee,
/// or if jump force can't be used; otherwise pick the roll anim, trace the destination
/// box, and commit the roll (clearing `saberMove`) if the path is clear. `static int`→`pub`.
///
/// No oracle: gated by the already-verified saber/force predicates and driven by the
/// `pm->trace` engine callback (the `PM_CorrectAllSolid` precedent).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps` + `trace` callback.
pub unsafe fn PM_TryRoll() -> c_int {
    let pmv = *addr_of!(pm);
    let mut trace: trace_t = core::mem::zeroed();
    let mut anim: c_int = -1;
    let mut fwd: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut traceto: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut fwdAngles: vec3_t = [0.0; 3];

    if BG_SaberInAttack((*(*pmv).ps).saberMove) != 0
        || BG_SaberInSpecialAttack((*(*pmv).ps).torsoAnim) != 0
        || BG_SpinningSaberAnim((*(*pmv).ps).legsAnim) != 0
        || PM_SaberInStart((*(*pmv).ps).saberMove) != 0
    {
        //attacking or spinning (or, if player, starting an attack)
        if PM_CanRollFromSoulCal((*pmv).ps) != 0 {
            //hehe
        } else {
            return 0;
        }
    }

    if ((*(*pmv).ps).weapon != WP_SABER && (*(*pmv).ps).weapon != WP_MELEE)
        || PM_IsRocketTrooper() != 0
        || BG_HasYsalamiri((*pmv).gametype, (*pmv).ps) != 0
        || BG_CanUseFPNow((*pmv).gametype, (*pmv).ps, (*pmv).cmd.serverTime, FP_LEVITATION) == 0
    {
        //Not using saber, or can't use jump
        return 0;
    }

    if (*(*pmv).ps).weapon == WP_SABER {
        let mut saber: *mut saberInfo_t = BG_MySaber((*(*pmv).ps).clientNum, 0);
        if !saber.is_null() && (*saber).saberFlags & SFL_NO_ROLLS != 0 {
            return 0;
        }
        saber = BG_MySaber((*(*pmv).ps).clientNum, 1);
        if !saber.is_null() && (*saber).saberFlags & SFL_NO_ROLLS != 0 {
            return 0;
        }
    }

    VectorSet(
        &mut mins,
        (*pmv).mins[0],
        (*pmv).mins[1],
        (*pmv).mins[2] + STEPSIZE as f32,
    );
    VectorSet(
        &mut maxs,
        (*pmv).maxs[0],
        (*pmv).maxs[1],
        (*(*pmv).ps).crouchheight as f32,
    );

    VectorSet(&mut fwdAngles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

    AngleVectors(&fwdAngles, Some(&mut fwd), Some(&mut right), None);

    if (*pmv).cmd.forwardmove != 0 {
        //check forward/backward rolls
        if (*(*pmv).ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
            anim = BOTH_ROLL_B;
            VectorMA(&(*(*pmv).ps).origin, -64.0, &fwd, &mut traceto);
        } else {
            anim = BOTH_ROLL_F;
            VectorMA(&(*(*pmv).ps).origin, 64.0, &fwd, &mut traceto);
        }
    } else if (*pmv).cmd.rightmove > 0 {
        //right
        anim = BOTH_ROLL_R;
        VectorMA(&(*(*pmv).ps).origin, 64.0, &right, &mut traceto);
    } else if (*pmv).cmd.rightmove < 0 {
        //left
        anim = BOTH_ROLL_L;
        VectorMA(&(*(*pmv).ps).origin, -64.0, &right, &mut traceto);
    }

    if anim != -1 {
        //We want to roll. Perform a trace to see if we can, and if so, send us into one.
        ((*pmv).trace.unwrap())(
            &mut trace,
            (*(*pmv).ps).origin.as_ptr(),
            mins.as_ptr(),
            maxs.as_ptr(),
            traceto.as_ptr(),
            (*(*pmv).ps).clientNum,
            CONTENTS_SOLID,
        );
        if trace.fraction >= 1.0 {
            (*(*pmv).ps).saberMove = LS_NONE;
            return anim;
        }
    }
    0
}

/// `PM_CrashLandEffect` (bg_pmove.c:3536) — `#ifdef QAGAME`-only cosmetic landing splash:
/// on a hard, non-water landing, key a material-specific effect off the ground surface
/// and play it at the player's feet. Purely visual — no playerstate/gameplay mutation and
/// no RNG.
///
/// The single `G_PlayEffect` call is wired to its g_utils port (this module IS the QAGAME
/// build); the `bottom`/`effectID` values feed only that call.
///
/// No C oracle: reads global `pm`/`pml` and would drive an engine effect callback.
///
/// Fidelity: C `float delta = fabs(prevVel[2])/10` promotes the `float` to `double` for
/// `fabs` and the `/10` divide, then rounds the `double` quotient back to `float`. So the
/// divide is carried in f64 — `(x as f64).abs() / 10.0` before the `as f32` store — not a
/// single f32 divide (the latter would double-round differently for some inputs).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live playerstate.
pub unsafe fn PM_CrashLandEffect() {
    let pmv = *addr_of!(pm);
    let pmlref = addr_of!(pml);

    let delta: f32;
    if (*pmv).waterlevel != 0 {
        return;
    }
    delta = (((*pmlref).previous_velocity[2] as f64).abs() / 10.0) as f32; //VectorLength( pml.previous_velocity );?
    if delta >= 30.0 {
        let mut bottom: vec3_t = [0.0; 3];
        let mut effectID: c_int = -1;
        let material: c_int = (*pmlref).groundTrace.surfaceFlags & MATERIAL_MASK;
        VectorSet(
            &mut bottom,
            (*(*pmv).ps).origin[0],
            (*(*pmv).ps).origin[1],
            (*(*pmv).ps).origin[2] + (*pmv).mins[2] + 1.0,
        );
        match material {
            MATERIAL_MUD => effectID = EFFECT_LANDING_MUD,
            MATERIAL_SAND => effectID = EFFECT_LANDING_SAND,
            MATERIAL_DIRT => effectID = EFFECT_LANDING_DIRT,
            MATERIAL_SNOW => effectID = EFFECT_LANDING_SNOW,
            MATERIAL_GRAVEL => effectID = EFFECT_LANDING_GRAVEL,
            _ => {}
        }

        if effectID != -1 {
            G_PlayEffect(effectID, &bottom, &(*pmlref).groundTrace.plane.normal);
        }
    }
}

/*
=================
PM_CrashLand

Check for hard landings that generate sound events
=================
*/
/// `PM_CrashLand` (bg_pmove.c:3583) — on touching ground, recover the exact landing
/// velocity (a quadratic in the fall: `½·acc·t² + vel·t − dist = 0`), pick the right
/// land animation, push the weapon back into its ready stance, optionally roll out of
/// a crouched landing, and emit the fall/roll/footstep event with damage-scaled `delta`.
///
/// No C oracle: this mutates global `pm`/`pml`/playerstate and drives the already
/// oracle-pinned `PM_SetAnim` setter cluster (`PM_CorrectAllSolid` trace-driven
/// precedent). The landing-`delta` arithmetic is transcribed verbatim, preserving C's
/// `float`->`double` promotions at `sqrt()` and the `0.0001` literal.
///
/// The cosmetic `PM_CrashLandEffect()` call is now live (ported above); only the
/// `G_PlayEffect` server call *inside* it remains a not-yet-ported `g_*` stub.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live playerstate.
pub unsafe fn PM_CrashLand() {
    let pmv = *addr_of!(pm);
    let pmlref = addr_of_mut!(pml);

    let dist: f32;
    let vel: f32;
    let acc: f32;
    let t: f32;
    let a: f32;
    let b: f32;
    let c: f32;
    let den: f32;
    let mut didRoll: qboolean = QFALSE;

    // calculate the exact velocity on landing
    dist = (*(*pmv).ps).origin[2] - (*pmlref).previous_origin[2];
    vel = (*pmlref).previous_velocity[2];
    acc = -(*(*pmv).ps).gravity as f32;

    a = acc / 2.0;
    b = vel;
    c = -dist;

    den = b * b - 4.0 * a * c;
    if den < 0.0 {
        (*(*pmv).ps).inAirAnim = QFALSE;
        return;
    }
    t = ((-(b as f64) - (den as f64).sqrt()) / (2.0 * a) as f64) as f32;

    let mut delta = vel + t * acc;
    delta = ((delta * delta) as f64 * 0.0001) as f32;

    // #ifdef QAGAME
    PM_CrashLandEffect();
    // #endif

    // ducking while falling doubles damage
    if (*(*pmv).ps).pm_flags & PMF_DUCKED != 0 {
        delta *= 2.0;
    }

    if (*(*pmv).ps).legsAnim == BOTH_A7_KICK_F_AIR
        || (*(*pmv).ps).legsAnim == BOTH_A7_KICK_B_AIR
        || (*(*pmv).ps).legsAnim == BOTH_A7_KICK_R_AIR
        || (*(*pmv).ps).legsAnim == BOTH_A7_KICK_L_AIR
    {
        let mut landAnim: c_int = -1;
        match (*(*pmv).ps).legsAnim {
            BOTH_A7_KICK_F_AIR => landAnim = BOTH_FORCELAND1,
            BOTH_A7_KICK_B_AIR => landAnim = BOTH_FORCELANDBACK1,
            BOTH_A7_KICK_R_AIR => landAnim = BOTH_FORCELANDRIGHT1,
            BOTH_A7_KICK_L_AIR => landAnim = BOTH_FORCELANDLEFT1,
            _ => {}
        }
        if landAnim != -1 {
            if (*(*pmv).ps).torsoAnim == (*(*pmv).ps).legsAnim {
                PM_SetAnim(
                    SETANIM_BOTH,
                    landAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            } else {
                PM_SetAnim(
                    SETANIM_LEGS,
                    landAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            }
        }
    } else if (*(*pmv).ps).legsAnim == BOTH_FORCEJUMPLEFT1
        || (*(*pmv).ps).legsAnim == BOTH_FORCEJUMPRIGHT1
        || (*(*pmv).ps).legsAnim == BOTH_FORCEJUMPBACK1
        || (*(*pmv).ps).legsAnim == BOTH_FORCEJUMP1
    {
        let fjAnim: c_int = match (*(*pmv).ps).legsAnim {
            BOTH_FORCEJUMPLEFT1 => BOTH_LANDLEFT1,
            BOTH_FORCEJUMPRIGHT1 => BOTH_LANDRIGHT1,
            BOTH_FORCEJUMPBACK1 => BOTH_LANDBACK1,
            _ => BOTH_LAND1,
        };
        PM_SetAnim(
            SETANIM_BOTH,
            fjAnim,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            0,
        );
    }
    // decide which landing animation to use
    else if BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) == QFALSE
        && (*(*pmv).ps).inAirAnim != 0
        && (*(*pmv).ps).m_iVehicleNum == 0
    {
        //only play a land animation if we transitioned into an in-air animation while off the ground
        if BG_SaberInSpecial((*(*pmv).ps).saberMove) == QFALSE {
            if (*(*pmv).ps).pm_flags & PMF_BACKWARDS_JUMP != 0 {
                PM_ForceLegsAnim(BOTH_LANDBACK1);
            } else {
                PM_ForceLegsAnim(BOTH_LAND1);
            }
        }
    }

    if (*(*pmv).ps).weapon != WP_SABER
        && (*(*pmv).ps).weapon != WP_MELEE
        && PM_IsRocketTrooper() == QFALSE
    {
        //saber handles its own anims
        //This will push us back into our weaponready stance from the land anim.
        if (*(*pmv).ps).weapon == WP_DISRUPTOR && (*(*pmv).ps).zoomMode == 1 {
            PM_StartTorsoAnim(TORSO_WEAPONREADY4);
        } else if (*(*pmv).ps).weapon == WP_EMPLACED_GUN {
            PM_StartTorsoAnim(BOTH_GUNSIT1);
        } else {
            PM_StartTorsoAnim(WeaponReadyAnim[(*(*pmv).ps).weapon as usize]);
        }
    }

    if BG_InSpecialJump((*(*pmv).ps).legsAnim) == QFALSE
        || (*(*pmv).ps).legsTimer < 1
        || (*(*pmv).ps).legsAnim == BOTH_WALL_RUN_LEFT
        || (*(*pmv).ps).legsAnim == BOTH_WALL_RUN_RIGHT
    {
        //Only set the timer if we're in an anim that can be interrupted (this would not be, say, a flip)
        if BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) == QFALSE && (*(*pmv).ps).inAirAnim != 0 {
            if BG_SaberInSpecial((*(*pmv).ps).saberMove) == QFALSE
                || (*(*pmv).ps).weapon != WP_SABER
            {
                if (*(*pmv).ps).legsAnim != BOTH_FORCELAND1
                    && (*(*pmv).ps).legsAnim != BOTH_FORCELANDBACK1
                    && (*(*pmv).ps).legsAnim != BOTH_FORCELANDRIGHT1
                    && (*(*pmv).ps).legsAnim != BOTH_FORCELANDLEFT1
                {
                    //don't override if we have started a force land
                    (*(*pmv).ps).legsTimer = TIMER_LAND;
                }
            }
        }
    }

    (*(*pmv).ps).inAirAnim = QFALSE;

    if (*(*pmv).ps).m_iVehicleNum != 0 {
        //don't do fall stuff while on a vehicle
        return;
    }

    // never take falling damage if completely underwater
    if (*pmv).waterlevel == 3 {
        return;
    }

    // reduce falling damage if there is standing water
    if (*pmv).waterlevel == 2 {
        delta *= 0.25;
    }
    if (*pmv).waterlevel == 1 {
        delta *= 0.5;
    }

    if delta < 1.0 {
        return;
    }

    if (*(*pmv).ps).pm_flags & PMF_DUCKED != 0 {
        if delta >= 2.0
            && PM_InOnGroundAnim((*(*pmv).ps).legsAnim) == QFALSE
            && PM_InKnockDown((*pmv).ps) == QFALSE
            && BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) == QFALSE
            && (*(*pmv).ps).forceHandExtend == HANDEXTEND_NONE
        {
            //roll!
            let mut anim = PM_TryRoll();

            if PM_InRollComplete((*pmv).ps, (*(*pmv).ps).legsAnim) != QFALSE {
                anim = 0;
                (*(*pmv).ps).legsTimer = 0;
                (*(*pmv).ps).legsAnim = 0;
                PM_SetAnim(
                    SETANIM_BOTH,
                    BOTH_LAND1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    150,
                );
                (*(*pmv).ps).legsTimer = TIMER_LAND;
            }

            if anim != 0 {
                //absorb some impact
                (*(*pmv).ps).legsTimer = 0;
                delta /= 3.0; // /= 2 just cancels out the above delta *= 2 when landing while crouched, the roll itself should absorb a little damage
                (*(*pmv).ps).legsAnim = 0;
                if (*(*pmv).ps).torsoAnim == BOTH_A7_SOULCAL {
                    //get out of it on torso
                    (*(*pmv).ps).torsoTimer = 0;
                }
                PM_SetAnim(
                    SETANIM_BOTH,
                    anim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    150,
                );
                didRoll = QTRUE;
            }
        }
    }

    // SURF_NODAMAGE is used for bounce pads where you don't ever
    // want to take damage or play a crunch sound
    if (*pmlref).groundTrace.surfaceFlags & SURF_NODAMAGE == 0 {
        if delta > 7.0 {
            let mut delta_send = delta as c_int;

            if delta_send > 600 {
                //will never need to know any value above this
                delta_send = 600;
            }

            if (*(*pmv).ps).fd.forceJumpZStart != 0.0 {
                if (*(*pmv).ps).origin[2] as c_int >= (*(*pmv).ps).fd.forceJumpZStart as c_int {
                    //was force jumping, landed on higher or same level as when force jump was started
                    if delta_send > 8 {
                        delta_send = 8;
                    }
                } else if delta_send > 8 {
                    let dif =
                        (*(*pmv).ps).fd.forceJumpZStart as c_int - (*(*pmv).ps).origin[2] as c_int;
                    let mut dmgLess = (forceJumpHeight
                        [(*(*pmv).ps).fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
                        - dif as f32) as c_int;

                    if dmgLess < 0 {
                        dmgLess = 0;
                    }

                    delta_send -= (dmgLess as f32 * 0.3) as c_int;

                    if delta_send < 8 {
                        delta_send = 8;
                    }

                    //Com_Printf("Damage sub: %i\n", (int)((dmgLess*0.1)));
                }
            }

            if didRoll != QFALSE {
                //Add the appropriate event..
                PM_AddEventWithParm(EV_ROLL, delta_send);
            } else {
                PM_AddEventWithParm(EV_FALL, delta_send);
            }
        } else if didRoll != QFALSE {
            PM_AddEventWithParm(EV_ROLL, 0);
        } else {
            PM_AddEventWithParm(EV_FOOTSTEP, PM_FootstepForSurface());
        }
    }

    // make sure velocity resets so we don't bounce back up again in case we miss the clear elsewhere
    (*(*pmv).ps).velocity[2] = 0.0;

    // start footstep cycle over
    (*(*pmv).ps).bobCycle = 0;
}

/*
=============
PM_CorrectAllSolid
=============
*/
/// `PM_CorrectAllSolid` (bg_pmove.c:3861) — when the ground trace starts in solid,
/// jitter the origin ±1 on each axis looking for a non-solid spot; on success re-trace
/// straight down and store the result as `pml.groundTrace`. Returns `qtrue` if it found
/// a way out, `qfalse` (and marks us airborne) otherwise.
///
/// No oracle: driven entirely by the `pm->trace` engine callback (the
/// `PM_pitch_roll_for_slope` precedent).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `trace` callback; `trace` to a
/// valid `trace_t`.
pub unsafe fn PM_CorrectAllSolid(trace: *mut trace_t) -> c_int {
    let pmv = *addr_of!(pm);
    let mut point: vec3_t = [0.0; 3];

    if (*pmv).debugLevel != 0 {
        Com_Printf(&format!("{}:allsolid\n", *addr_of!(c_pmove)));
    }

    // jitter around
    for i in -1..=1 {
        for j in -1..=1 {
            for k in -1..=1 {
                VectorCopy(&(*(*pmv).ps).origin, &mut point);
                point[0] += i as f32;
                point[1] += j as f32;
                point[2] += k as f32;
                ((*pmv).trace.unwrap())(
                    trace,
                    point.as_ptr(),
                    (*pmv).mins.as_ptr(),
                    (*pmv).maxs.as_ptr(),
                    point.as_ptr(),
                    (*(*pmv).ps).clientNum,
                    (*pmv).tracemask,
                );
                if (*trace).allsolid == 0 {
                    point[0] = (*(*pmv).ps).origin[0];
                    point[1] = (*(*pmv).ps).origin[1];
                    point[2] = (*(*pmv).ps).origin[2] - 0.25;

                    ((*pmv).trace.unwrap())(
                        trace,
                        (*(*pmv).ps).origin.as_ptr(),
                        (*pmv).mins.as_ptr(),
                        (*pmv).maxs.as_ptr(),
                        point.as_ptr(),
                        (*(*pmv).ps).clientNum,
                        (*pmv).tracemask,
                    );
                    (*addr_of_mut!(pml)).groundTrace = *trace;
                    return QTRUE;
                }
            }
        }
    }

    (*(*pmv).ps).groundEntityNum = ENTITYNUM_NONE;
    (*addr_of_mut!(pml)).groundPlane = QFALSE;
    (*addr_of_mut!(pml)).walking = QFALSE;

    QFALSE
}

/*
=============
PM_GroundTraceMissed

The ground trace didn't hit a surface, so we are in freefall
=============
*/
/// `PM_GroundTraceMissed` (bg_pmove.c:3905) — the down-trace missed, so we're airborne:
/// pick the right freefall/jump leg animation (choke-float, jet-pack, kick-off into a
/// jump, or just fall) and clear `groundEntityNum`/`groundPlane`/`walking`.
///
/// No C oracle: trace-driven (the `pm->trace` engine callback) and drives the already
/// oracle-pinned `PM_SetAnim` cluster (`PM_CorrectAllSolid` precedent).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `trace` callback and playerstate.
pub unsafe fn PM_GroundTraceMissed() {
    let pmv = *addr_of!(pm);
    let mut trace: trace_t = core::mem::zeroed();
    let mut point: vec3_t = [0.0; 3];

    //rww - don't want to do this when handextend_choke, because you can be standing on the ground
    //while still holding your throat.
    if (*(*pmv).ps).pm_type == PM_FLOAT {
        //we're assuming this is because you're being choked
        let parts: c_int = SETANIM_LEGS;

        //rww - also don't use SETANIM_FLAG_HOLD, it will cause the legs to float around a bit before going into
        //a proper anim even when on the ground.
        PM_SetAnim(parts, BOTH_CHOKE3, SETANIM_FLAG_OVERRIDE, 100);
    } else if (*(*pmv).ps).pm_type == PM_JETPACK {
        //jetpacking
        //rww - also don't use SETANIM_FLAG_HOLD, it will cause the legs to float around a bit before going into
        //a proper anim even when on the ground.
        //PM_SetAnim(SETANIM_LEGS,BOTH_FORCEJUMP1,SETANIM_FLAG_OVERRIDE, 100);
    }
    //If the anim is choke3, act like we just went into the air because we aren't in a float
    else if (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE
        || (*(*pmv).ps).legsAnim == BOTH_CHOKE3
    {
        // we just transitioned into freefall
        if (*pmv).debugLevel != 0 {
            Com_Printf(&format!("{}:lift\n", *addr_of!(c_pmove)));
        }

        // if they aren't in a jumping animation and the ground is a ways away, force into it
        // if we didn't do the trace, the player would be backflipping down staircases
        VectorCopy(&(*(*pmv).ps).origin, &mut point);
        point[2] -= 64.0;

        ((*pmv).trace.unwrap())(
            &mut trace,
            (*(*pmv).ps).origin.as_ptr(),
            (*pmv).mins.as_ptr(),
            (*pmv).maxs.as_ptr(),
            point.as_ptr(),
            (*(*pmv).ps).clientNum,
            (*pmv).tracemask,
        );
        if trace.fraction == 1.0 || (*(*pmv).ps).pm_type == PM_FLOAT {
            if (*(*pmv).ps).velocity[2] <= 0.0 && (*(*pmv).ps).pm_flags & PMF_JUMP_HELD == 0 {
                //PM_SetAnim(SETANIM_LEGS,BOTH_INAIR1,SETANIM_FLAG_OVERRIDE, 100);
                PM_SetAnim(SETANIM_LEGS, BOTH_INAIR1, 0, 100);
                (*(*pmv).ps).pm_flags &= !PMF_BACKWARDS_JUMP;
            } else if (*pmv).cmd.forwardmove >= 0 {
                PM_SetAnim(SETANIM_LEGS, BOTH_JUMP1, SETANIM_FLAG_OVERRIDE, 100);
                (*(*pmv).ps).pm_flags &= !PMF_BACKWARDS_JUMP;
            } else {
                PM_SetAnim(SETANIM_LEGS, BOTH_JUMPBACK1, SETANIM_FLAG_OVERRIDE, 100);
                (*(*pmv).ps).pm_flags |= PMF_BACKWARDS_JUMP;
            }

            (*(*pmv).ps).inAirAnim = QTRUE;
        }
    } else if (*(*pmv).ps).inAirAnim == 0 {
        // if they aren't in a jumping animation and the ground is a ways away, force into it
        // if we didn't do the trace, the player would be backflipping down staircases
        VectorCopy(&(*(*pmv).ps).origin, &mut point);
        point[2] -= 64.0;

        ((*pmv).trace.unwrap())(
            &mut trace,
            (*(*pmv).ps).origin.as_ptr(),
            (*pmv).mins.as_ptr(),
            (*pmv).maxs.as_ptr(),
            point.as_ptr(),
            (*(*pmv).ps).clientNum,
            (*pmv).tracemask,
        );
        if trace.fraction == 1.0 || (*(*pmv).ps).pm_type == PM_FLOAT {
            (*(*pmv).ps).inAirAnim = QTRUE;
        }
    }

    if PM_InRollComplete((*pmv).ps, (*(*pmv).ps).legsAnim) != QFALSE {
        //Client won't catch an animation restart because it only checks frame against incoming frame, so if you roll when you land after rolling
        //off of something it won't replay the roll anim unless we switch it off in the air. This fixes that.
        PM_SetAnim(
            SETANIM_BOTH,
            BOTH_INAIR1,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            150,
        );
        (*(*pmv).ps).inAirAnim = QTRUE;
    }

    (*(*pmv).ps).groundEntityNum = ENTITYNUM_NONE;
    (*addr_of_mut!(pml)).groundPlane = QFALSE;
    (*addr_of_mut!(pml)).walking = QFALSE;
}

/*
=============
PM_GroundTrace
=============
*/
/// `PM_GroundTrace` (bg_pmove.c:3993) — trace straight down a hair (0.25u) to find the
/// floor and classify the result: solid recovery, freefall (`PM_GroundTraceMissed`),
/// kicked off the ground, too-steep slope, or a valid landing (`PM_CrashLand` + the
/// QAGAME land-on-a-vehicle board check). Sets `groundEntityNum`/`groundPlane`/`walking`.
///
/// No C oracle: trace-driven (the `pm->trace` engine callback; `PM_CorrectAllSolid`
/// precedent) and drives the already-pinned land/anim helpers.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `trace` callback and playerstate;
/// `pm_entSelf`/`g_entities` and the vehicle chain must be valid where dereferenced.
pub unsafe fn PM_GroundTrace() {
    let pmv = *addr_of!(pm);
    let mut point: vec3_t = [0.0; 3];
    let mut trace: trace_t = core::mem::zeroed();
    let mut minNormal: f32 = MIN_WALK_NORMAL as f32;

    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int {
        let pEnt: *mut bgEntity_t = *addr_of!(pm_entSelf);

        if !pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_VEHICLE {
            minNormal = (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).maxSlope;
        }
    }

    point[0] = (*(*pmv).ps).origin[0];
    point[1] = (*(*pmv).ps).origin[1];
    point[2] = (*(*pmv).ps).origin[2] - 0.25;

    ((*pmv).trace.unwrap())(
        &mut trace,
        (*(*pmv).ps).origin.as_ptr(),
        (*pmv).mins.as_ptr(),
        (*pmv).maxs.as_ptr(),
        point.as_ptr(),
        (*(*pmv).ps).clientNum,
        (*pmv).tracemask,
    );
    (*addr_of_mut!(pml)).groundTrace = trace;

    // do something corrective if the trace starts in a solid...
    if trace.allsolid != 0 {
        if PM_CorrectAllSolid(&mut trace) == QFALSE {
            return;
        }
    }

    if (*(*pmv).ps).pm_type == PM_FLOAT || (*(*pmv).ps).pm_type == PM_JETPACK {
        PM_GroundTraceMissed();
        (*addr_of_mut!(pml)).groundPlane = QFALSE;
        (*addr_of_mut!(pml)).walking = QFALSE;
        return;
    }

    // if the trace didn't hit anything, we are in free fall
    if trace.fraction == 1.0 {
        PM_GroundTraceMissed();
        (*addr_of_mut!(pml)).groundPlane = QFALSE;
        (*addr_of_mut!(pml)).walking = QFALSE;
        return;
    }

    // check if getting thrown off the ground
    if (*(*pmv).ps).velocity[2] > 0.0
        && DotProduct(&(*(*pmv).ps).velocity, &trace.plane.normal) > 10.0
    {
        if (*pmv).debugLevel != 0 {
            Com_Printf(&format!("{}:kickoff\n", *addr_of!(c_pmove)));
        }
        // go into jump animation
        if (*pmv).cmd.forwardmove >= 0 {
            PM_ForceLegsAnim(BOTH_JUMP1);
            (*(*pmv).ps).pm_flags &= !PMF_BACKWARDS_JUMP;
        } else {
            PM_ForceLegsAnim(BOTH_JUMPBACK1);
            (*(*pmv).ps).pm_flags |= PMF_BACKWARDS_JUMP;
        }

        (*(*pmv).ps).groundEntityNum = ENTITYNUM_NONE;
        (*addr_of_mut!(pml)).groundPlane = QFALSE;
        (*addr_of_mut!(pml)).walking = QFALSE;
        return;
    }

    // slopes that are too steep will not be considered onground
    if trace.plane.normal[2] < minNormal {
        if (*pmv).debugLevel != 0 {
            Com_Printf(&format!("{}:steep\n", *addr_of!(c_pmove)));
        }
        (*(*pmv).ps).groundEntityNum = ENTITYNUM_NONE;
        (*addr_of_mut!(pml)).groundPlane = QTRUE;
        (*addr_of_mut!(pml)).walking = QFALSE;
        return;
    }

    (*addr_of_mut!(pml)).groundPlane = QTRUE;
    (*addr_of_mut!(pml)).walking = QTRUE;

    // hitting solid ground will end a waterjump
    if (*(*pmv).ps).pm_flags & PMF_TIME_WATERJUMP != 0 {
        (*(*pmv).ps).pm_flags &= !(PMF_TIME_WATERJUMP | PMF_TIME_LAND);
        (*(*pmv).ps).pm_time = 0;
    }

    if (*(*pmv).ps).groundEntityNum == ENTITYNUM_NONE {
        // just hit the ground
        if (*pmv).debugLevel != 0 {
            Com_Printf(&format!("{}:Land\n", *addr_of!(c_pmove)));
        }

        PM_CrashLand();

        // #ifdef QAGAME
        if (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int
            && (*(*pmv).ps).m_iVehicleNum == 0
            && (trace.entityNum as c_int) < ENTITYNUM_WORLD
            && (trace.entityNum as c_int) >= MAX_CLIENTS as c_int
            && (*(*pmv).ps).zoomMode == 0
            && !(*addr_of!(pm_entSelf)).is_null()
        {
            //check if we landed on a vehicle
            let trEnt: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(trace.entityNum as usize);
            if (*trEnt).inuse != QFALSE
                && !(*trEnt).client.is_null()
                && (*trEnt).s.eType == ET_NPC
                && (*trEnt).s.NPC_class == CLASS_VEHICLE
                && (*(*trEnt).client).ps.m_iVehicleNum == 0
                && !(*trEnt).m_pVehicle.is_null()
                && (*(*(*trEnt).m_pVehicle).m_pVehicleInfo).r#type != VH_WALKER
                && (*(*(*trEnt).m_pVehicle).m_pVehicleInfo).r#type != VH_FIGHTER
            {
                //it's a vehicle alright, let's board it.. if it's not an atst or ship
                if BG_SaberInSpecial((*(*pmv).ps).saberMove) == QFALSE
                    && (*(*pmv).ps).forceHandExtend == HANDEXTEND_NONE
                    && (*(*pmv).ps).weaponTime <= 0
                {
                    let servEnt: *mut gentity_t = *addr_of!(pm_entSelf) as *mut gentity_t;
                    if (*addr_of!(g_gametype)).integer < GT_TEAM
                        || (*trEnt).alliedTeam == 0
                        || (*trEnt).alliedTeam == (*(*servEnt).client).sess.sessionTeam
                    {
                        //not belonging to a team, or client is on same team
                        ((*(*(*trEnt).m_pVehicle).m_pVehicleInfo).Board.unwrap())(
                            (*trEnt).m_pVehicle,
                            *addr_of!(pm_entSelf),
                        );
                    }
                }
            }
        }
        // #endif

        // don't do landing time if we were just going down a slope
        if (*addr_of_mut!(pml)).previous_velocity[2] < -200.0 {
            // don't allow another jump for a little while
            (*(*pmv).ps).pm_flags |= PMF_TIME_LAND;
            (*(*pmv).ps).pm_time = 250;
        }
    }

    (*(*pmv).ps).groundEntityNum = trace.entityNum as c_int;
    (*(*pmv).ps).lastOnGround = (*pmv).cmd.serverTime;

    PM_AddTouchEnt(trace.entityNum as c_int);
}

/*
=============
PM_SetWaterLevel
=============
*/
/// `PM_SetWaterLevel` (bg_pmove.c:4137) — sample brush contents at up to three heights
/// (feet / mid / view) and set `pm->waterlevel` (0..3) + `pm->watertype` accordingly.
///
/// No oracle: driven by the `pm->pointcontents` engine callback.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `pointcontents` callback.
pub unsafe fn PM_SetWaterLevel() {
    let pmv = *addr_of!(pm);
    let mut point: vec3_t = [0.0; 3];
    let mut cont: c_int;
    let sample1: c_int;
    let sample2: c_int;

    //
    // get waterlevel, accounting for ducking
    //
    (*pmv).waterlevel = 0;
    (*pmv).watertype = 0;

    point[0] = (*(*pmv).ps).origin[0];
    point[1] = (*(*pmv).ps).origin[1];
    point[2] = (*(*pmv).ps).origin[2] + MINS_Z as f32 + 1.0;
    cont = ((*pmv).pointcontents.unwrap())(point.as_ptr(), (*(*pmv).ps).clientNum);

    if cont & MASK_WATER != 0 {
        sample2 = (*(*pmv).ps).viewheight - MINS_Z;
        sample1 = sample2 / 2;

        (*pmv).watertype = cont;
        (*pmv).waterlevel = 1;
        point[2] = (*(*pmv).ps).origin[2] + MINS_Z as f32 + sample1 as f32;
        cont = ((*pmv).pointcontents.unwrap())(point.as_ptr(), (*(*pmv).ps).clientNum);
        if cont & MASK_WATER != 0 {
            (*pmv).waterlevel = 2;
            point[2] = (*(*pmv).ps).origin[2] + MINS_Z as f32 + sample2 as f32;
            cont = ((*pmv).pointcontents.unwrap())(point.as_ptr(), (*(*pmv).ps).clientNum);
            if cont & MASK_WATER != 0 {
                (*pmv).waterlevel = 3;
            }
        }
    }
}

/// `PM_CheckDualForwardJumpDuck` (bg_pmove.c:4174) — while in the dual forward-jump-attack
/// (`BOTH_JUMPATTACK6`), during the sideways portion of the anim dynamically reduce the
/// bounding box (raise `mins[2]` to 0, set `PMF_FIX_MINS`) so the character sails over the
/// heads of enemies. Returns whether the box was resized. `qboolean`→`pub` (the `PM_CheckDuck`
/// caller). No oracle: reads/writes the live `pm` global + `PM_AnimLength`.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_CheckDualForwardJumpDuck() -> qboolean {
    let pmv = *addr_of!(pm);
    let mut resized: qboolean = QFALSE;
    if (*(*pmv).ps).legsAnim == BOTH_JUMPATTACK6 {
        //dynamically reduce bounding box to let character sail over heads of enemies
        if ((*(*pmv).ps).legsTimer >= 1450
            && PM_AnimLength(0, BOTH_JUMPATTACK6) - (*(*pmv).ps).legsTimer >= 400)
            || ((*(*pmv).ps).legsTimer >= 400
                && PM_AnimLength(0, BOTH_JUMPATTACK6) - (*(*pmv).ps).legsTimer >= 1100)
        {
            //in a part of the anim that we're pretty much sideways in, raise up the mins
            (*pmv).mins[2] = 0.0;
            (*(*pmv).ps).pm_flags |= PMF_FIX_MINS;
            resized = QTRUE;
        }
    }
    resized
}

/// `PM_CheckFixMins` (bg_pmove.c:4193) — if `PMF_FIX_MINS` is set, the bbox bottom was
/// previously raised; trace down to drop `mins[2]` back to `MINS_Z` when there's room,
/// else move the player up by the blocked distance, and as a last resort crouch them.
///
/// No oracle: driven by the `pm->trace` engine callback (the `PM_CorrectAllSolid`
/// precedent).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps` + `trace` callback.
pub unsafe fn PM_CheckFixMins() {
    let pmv = *addr_of!(pm);
    if (*(*pmv).ps).pm_flags & PMF_FIX_MINS != 0
    // pm->mins[2] > DEFAULT_MINS_2
    {
        //drop the mins back down
        //do a trace to make sure it's okay
        let mut trace: trace_t = core::mem::zeroed();
        let mut end: vec3_t = [0.0; 3];
        let mut cur_mins: vec3_t = [0.0; 3];
        let mut cur_maxs: vec3_t = [0.0; 3];

        VectorSet(
            &mut end,
            (*(*pmv).ps).origin[0],
            (*(*pmv).ps).origin[1],
            (*(*pmv).ps).origin[2] + MINS_Z as f32,
        );
        VectorSet(&mut cur_mins, (*pmv).mins[0], (*pmv).mins[1], 0.0);
        VectorSet(
            &mut cur_maxs,
            (*pmv).maxs[0],
            (*pmv).maxs[1],
            (*(*pmv).ps).standheight as f32,
        );

        ((*pmv).trace.unwrap())(
            &mut trace,
            (*(*pmv).ps).origin.as_ptr(),
            cur_mins.as_ptr(),
            cur_maxs.as_ptr(),
            end.as_ptr(),
            (*(*pmv).ps).clientNum,
            (*pmv).tracemask,
        );
        if trace.allsolid == 0 && trace.startsolid == 0
        //should never start in solid
        {
            if trace.fraction >= 1.0
            //all clear
            {
                //drop the bottom of my bbox back down
                (*pmv).mins[2] = MINS_Z as f32;
                (*(*pmv).ps).pm_flags &= !PMF_FIX_MINS;
            } else
            //move me up so the bottom of my bbox will be where the trace ended, at least
            {
                //need to trace up, too
                let updist = (1.0 - trace.fraction) * ((-MINS_Z) as f32);
                end[2] = (*(*pmv).ps).origin[2] + updist;
                ((*pmv).trace.unwrap())(
                    &mut trace,
                    (*(*pmv).ps).origin.as_ptr(),
                    cur_mins.as_ptr(),
                    cur_maxs.as_ptr(),
                    end.as_ptr(),
                    (*(*pmv).ps).clientNum,
                    (*pmv).tracemask,
                );
                if trace.allsolid == 0 && trace.startsolid == 0
                //should never start in solid
                {
                    if trace.fraction >= 1.0
                    //all clear
                    {
                        //move me up
                        (*(*pmv).ps).origin[2] += updist;
                        //drop the bottom of my bbox back down
                        (*pmv).mins[2] = MINS_Z as f32;
                        (*(*pmv).ps).pm_flags &= !PMF_FIX_MINS;
                    } else
                    //crap, no room to expand, so just crouch us
                    {
                        if (*(*pmv).ps).legsAnim != BOTH_JUMPATTACK6 || (*(*pmv).ps).legsTimer <= 200
                        {
                            //at the end of the anim, and we can't leave ourselves like this
                            //so drop the maxs, put the mins back and move us up
                            (*pmv).maxs[2] += MINS_Z as f32;
                            (*(*pmv).ps).origin[2] -= MINS_Z as f32;
                            (*pmv).mins[2] = MINS_Z as f32;
                            //this way we'll be in a crouch when we're done
                            if (*(*pmv).ps).legsAnim == BOTH_JUMPATTACK6 {
                                (*(*pmv).ps).legsTimer = 0;
                                (*(*pmv).ps).torsoTimer = 0;
                            }
                            (*(*pmv).ps).pm_flags |= PMF_DUCKED;
                            //FIXME: do we need to set a crouch anim here?
                            (*(*pmv).ps).pm_flags &= !PMF_FIX_MINS;
                        }
                    }
                } //crap, stuck
            }
        } //crap, stuck!
    }
}

/// `PM_CheckDuck` (bg_pmove.c:4262) — set the bbox `mins`/`maxs` and `viewheight` for the
/// frame from the duck/roll/vehicle state. On a vehicle, clear `PMF_DUCKED`/`PMF_ROLLING` and
/// (for speeders/animals) shrink-then-trace the box, dropping it to zero (and arming the
/// server-side `solidHack`) when it won't fit; on foot, run the dual-jump resize /
/// `PM_CheckFixMins`, handle dead/rolling/ducking, and finally derive the final `maxs[2]` +
/// `viewheight` from `PMF_DUCKED`/`PMF_ROLLING`. `static void`→`pub` (the `PmoveSingle`
/// caller). No oracle: driven by the `pm->trace` engine callback.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps` + `trace` callback; on a vehicle,
/// `pm_entVeh` (when non-null) must have valid `m_pVehicle`/`m_pVehicleInfo`.
pub unsafe fn PM_CheckDuck() {
    let pmv = *addr_of!(pm);
    let mut trace: trace_t = core::mem::zeroed();

    if (*(*pmv).ps).m_iVehicleNum > 0 && (*(*pmv).ps).m_iVehicleNum < ENTITYNUM_NONE {
        //riding a vehicle or are a vehicle
        //no ducking or rolling when on a vehicle
        //right?  not even on ones that you just ride on top of?
        (*(*pmv).ps).pm_flags &= !PMF_DUCKED;
        (*(*pmv).ps).pm_flags &= !PMF_ROLLING;
        //NOTE: we don't clear the pm->cmd.upmove here because
        //the vehicle code may need it later... but, for riders,
        //it should have already been copied over to the vehicle, right?

        if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int {
            return;
        }
        let entVeh = *addr_of!(pm_entVeh);
        if !entVeh.is_null()
            && !(*entVeh).m_pVehicle.is_null()
            && ((*(*(*entVeh).m_pVehicle).m_pVehicleInfo).r#type == VH_SPEEDER
                || (*(*(*entVeh).m_pVehicle).m_pVehicleInfo).r#type == VH_ANIMAL)
        {
            let mut solidTr: trace_t = core::mem::zeroed();

            (*pmv).mins[0] = -16.0;
            (*pmv).mins[1] = -16.0;
            (*pmv).mins[2] = MINS_Z as f32;

            (*pmv).maxs[0] = 16.0;
            (*pmv).maxs[1] = 16.0;
            (*pmv).maxs[2] = (*(*pmv).ps).standheight as f32; //DEFAULT_MAXS_2;
            (*(*pmv).ps).viewheight = DEFAULT_VIEWHEIGHT;

            ((*pmv).trace.unwrap())(
                &mut solidTr,
                (*(*pmv).ps).origin.as_ptr(),
                (*pmv).mins.as_ptr(),
                (*pmv).maxs.as_ptr(),
                (*(*pmv).ps).origin.as_ptr(),
                (*(*pmv).ps).m_iVehicleNum,
                (*pmv).tracemask,
            );
            if solidTr.startsolid != 0 || solidTr.allsolid != 0 || solidTr.fraction != 1.0 {
                //whoops, can't fit here. Down to 0!
                VectorClear(&mut (*pmv).mins);
                VectorClear(&mut (*pmv).maxs);
                // #ifdef QAGAME
                {
                    let me: *mut gentity_t =
                        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*pmv).ps).clientNum as usize);
                    if (*me).inuse != QFALSE && !(*me).client.is_null() {
                        //yeah, this is a really terrible hack.
                        (*(*me).client).solidHack =
                            (*addr_of!(crate::codemp::game::g_main::level)).time + 200;
                    }
                }
            }
        }
    } else {
        if (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int {
            (*pmv).mins[0] = -15.0;
            (*pmv).mins[1] = -15.0;

            (*pmv).maxs[0] = 15.0;
            (*pmv).maxs[1] = 15.0;
        }

        if PM_CheckDualForwardJumpDuck() != QFALSE {
            //special anim resizing us
        } else {
            PM_CheckFixMins();

            if (*pmv).mins[2] == 0.0 {
                (*pmv).mins[2] = MINS_Z as f32;
            }
        }

        if (*(*pmv).ps).pm_type == PM_DEAD && (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int {
            (*pmv).maxs[2] = -8.0;
            (*(*pmv).ps).viewheight = DEAD_VIEWHEIGHT;
            return;
        }

        if BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) != QFALSE
            && BG_KickingAnim((*(*pmv).ps).legsAnim) == QFALSE
        {
            (*pmv).maxs[2] = (*(*pmv).ps).crouchheight as f32; //CROUCH_MAXS_2;
            (*(*pmv).ps).viewheight = DEFAULT_VIEWHEIGHT;
            (*(*pmv).ps).pm_flags &= !PMF_DUCKED;
            (*(*pmv).ps).pm_flags |= PMF_ROLLING;
            return;
        } else if (*(*pmv).ps).pm_flags & PMF_ROLLING != 0 {
            // try to stand up
            (*pmv).maxs[2] = (*(*pmv).ps).standheight as f32; //DEFAULT_MAXS_2;
            ((*pmv).trace.unwrap())(
                &mut trace,
                (*(*pmv).ps).origin.as_ptr(),
                (*pmv).mins.as_ptr(),
                (*pmv).maxs.as_ptr(),
                (*(*pmv).ps).origin.as_ptr(),
                (*(*pmv).ps).clientNum,
                (*pmv).tracemask,
            );
            if trace.allsolid == 0 {
                (*(*pmv).ps).pm_flags &= !PMF_ROLLING;
            }
        } else if (*pmv).cmd.upmove < 0
            || (*(*pmv).ps).forceHandExtend == HANDEXTEND_KNOCKDOWN
            || (*(*pmv).ps).forceHandExtend == HANDEXTEND_PRETHROWN
            || (*(*pmv).ps).forceHandExtend == HANDEXTEND_POSTTHROWN
        {
            // duck
            (*(*pmv).ps).pm_flags |= PMF_DUCKED;
        } else {
            // stand up if possible
            if (*(*pmv).ps).pm_flags & PMF_DUCKED != 0 {
                // try to stand up
                (*pmv).maxs[2] = (*(*pmv).ps).standheight as f32; //DEFAULT_MAXS_2;
                ((*pmv).trace.unwrap())(
                    &mut trace,
                    (*(*pmv).ps).origin.as_ptr(),
                    (*pmv).mins.as_ptr(),
                    (*pmv).maxs.as_ptr(),
                    (*(*pmv).ps).origin.as_ptr(),
                    (*(*pmv).ps).clientNum,
                    (*pmv).tracemask,
                );
                if trace.allsolid == 0 {
                    (*(*pmv).ps).pm_flags &= !PMF_DUCKED;
                }
            }
        }
    }

    if (*(*pmv).ps).pm_flags & PMF_DUCKED != 0 {
        (*pmv).maxs[2] = (*(*pmv).ps).crouchheight as f32; //CROUCH_MAXS_2;
        (*(*pmv).ps).viewheight = CROUCH_VIEWHEIGHT;
    } else if (*(*pmv).ps).pm_flags & PMF_ROLLING != 0 {
        (*pmv).maxs[2] = (*(*pmv).ps).crouchheight as f32; //CROUCH_MAXS_2;
        (*(*pmv).ps).viewheight = DEFAULT_VIEWHEIGHT;
    } else {
        (*pmv).maxs[2] = (*(*pmv).ps).standheight as f32; //DEFAULT_MAXS_2;
        (*(*pmv).ps).viewheight = DEFAULT_VIEWHEIGHT;
    }
}

/// `PM_Use` (bg_pmove.c:4411) — generate a `+use` event. Decrements the `useTime`
/// cooldown (100ms/frame), and on a `BUTTON_USE` press past the cooldown fires
/// `EV_USE` and re-arms the `USE_DELAY` (2000ms) timer. `void`→`pub` (the `PmoveSingle`
/// caller). No oracle (event-callback / playerstate-mutation driven).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_Use() {
    const USE_DELAY: c_int = 2000;
    let pmv = *addr_of!(pm);

    if (*(*pmv).ps).useTime > 0 {
        (*(*pmv).ps).useTime -= 100; //pm->cmd.msec;
    }

    if (*(*pmv).ps).useTime > 0 {
        return;
    }

    if (*pmv).cmd.buttons & BUTTON_USE == 0 {
        (*pmv).useEvent = 0;
        (*(*pmv).ps).useTime = 0;
        return;
    }

    (*pmv).useEvent = EV_USE;
    (*(*pmv).ps).useTime = USE_DELAY;
}

/// `PM_WalkingAnim` (bg_pmove.c:4431) — is `anim` one of the walk (or walk-back)
/// loop animations? Pure classifier `switch`→`match` (the `bg_panimate` precedent).
pub fn PM_WalkingAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_WALK1            //# Normal walk
        | BOTH_WALK2          //# Normal walk with saber
        | BOTH_WALK_STAFF     //# Normal walk with staff
        | BOTH_WALK_DUAL      //# Normal walk with staff
        | BOTH_WALK5          //# Tavion taunting Kyle (cin 22)
        | BOTH_WALK6          //# Slow walk for Luke (cin 12)
        | BOTH_WALK7          //# Fast walk
        | BOTH_WALKBACK1      //# Walk1 backwards
        | BOTH_WALKBACK2      //# Walk2 backwards
        | BOTH_WALKBACK_STAFF //# Walk backwards with staff
        | BOTH_WALKBACK_DUAL  //# Walk backwards with dual
        => QTRUE,
        _ => QFALSE,
    }
}

/// `PM_RunningAnim` (bg_pmove.c:4452) — is `anim` one of the run / run-back / run-strafe
/// loop or start/stop animations? Pure classifier `switch`→`match`.
pub fn PM_RunningAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_RUN1
        | BOTH_RUN2
        | BOTH_RUN_STAFF
        | BOTH_RUN_DUAL
        | BOTH_RUNBACK1
        | BOTH_RUNBACK2
        | BOTH_RUNBACK_STAFF
        | BOTH_RUNBACK_DUAL
        | BOTH_RUN1START          //# Start into full run1
        | BOTH_RUN1STOP           //# Stop from full run1
        | BOTH_RUNSTRAFE_LEFT1    //# Sidestep left: should loop
        | BOTH_RUNSTRAFE_RIGHT1   //# Sidestep right: should loop
        => QTRUE,
        _ => QFALSE,
    }
}

/// `PM_SwimmingAnim` (bg_pmove.c:4474) — is `anim` one of the swim idle/forward/backward
/// loop animations? Pure classifier `switch`→`match`.
pub fn PM_SwimmingAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_SWIM_IDLE1    //# Swimming Idle 1
        | BOTH_SWIMFORWARD //# Swim forward loop
        | BOTH_SWIMBACKWARD //# Swim backward loop
        => QTRUE,
        _ => QFALSE,
    }
}

/// `PM_RollingAnim` (bg_pmove.c:4487) — is `anim` one of the four directional roll
/// animations? Pure classifier `switch`→`match`.
pub fn PM_RollingAnim(anim: c_int) -> qboolean {
    match anim {
        BOTH_ROLL_F    //# Roll forward
        | BOTH_ROLL_B  //# Roll backward
        | BOTH_ROLL_L  //# Roll left
        | BOTH_ROLL_R  //# Roll right
        => QTRUE,
        _ => QFALSE,
    }
}

/// `PM_AnglesForSlope` (bg_pmove.c:4501) — given a facing `yaw` and a ground-plane
/// `slope` normal, compute the pitch/roll body angles that lay the model flat on the
/// slope (yaw is zeroed in the output). Pure `vec3_t` math over `AngleVectors`/
/// `vectoangles`/`DotProduct`/`Q_fabs`.
pub fn PM_AnglesForSlope(yaw: f32, slope: &vec3_t, angles: &mut vec3_t) {
    let mut nvf: vec3_t = [0.0; 3];
    let mut ovf: vec3_t = [0.0; 3];
    let mut ovr: vec3_t = [0.0; 3];
    let mut new_angles: vec3_t = [0.0; 3];

    VectorSet(angles, 0.0, yaw, 0.0);
    AngleVectors(angles, Some(&mut ovf), Some(&mut ovr), None);

    vectoangles(slope, &mut new_angles);
    let pitch = new_angles[PITCH] + 90.0;
    new_angles[ROLL] = 0.0;
    new_angles[PITCH] = 0.0;

    AngleVectors(&new_angles, Some(&mut nvf), None, None);

    let mut r#mod = DotProduct(&nvf, &ovr);

    if r#mod < 0.0 {
        r#mod = -1.0;
    } else {
        r#mod = 1.0;
    }

    let dot = DotProduct(&nvf, &ovf);

    angles[YAW] = 0.0;
    angles[PITCH] = dot * pitch;
    angles[ROLL] = (1.0 - Q_fabs(dot)) * pitch * r#mod;
}

/// `PM_FootSlopeTrace` (bg_pmove.c:4529) — measure the ground-height difference between
/// the two foot bolts, used to pick a slope-stand legs anim. For each foot it asks the
/// engine for the bolt's world matrix (`trap_G2API_GetBoltMatrix`), takes the bolt origin
/// (matrix column 3), drops it to the bottom of the bbox+1, traces straight down through
/// `pm->trace`, and returns the L−R endpoint-Z difference in `*pDiff` and the fixed
/// `interval` (4) in `*pInterval`; either out-param may be null. `footLSlope`/`footRSlope`
/// copy each trace's plane normal but, as in the original, are never read afterward
/// (faithful dead stores).
///
/// No oracle: calls the `trap_G2API_GetBoltMatrix` syscall and the `pm->trace` engine
/// callback (the `PM_CorrectAllSolid` precedent) — behaviour depends on engine state, so
/// it is verified by inspection against the C, not bit-exact.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` (live `ps`, `trace` callback, ghoul2 instance);
/// `pDiff`/`pInterval` must be null or valid `f32` out-pointers.
pub unsafe fn PM_FootSlopeTrace(pDiff: *mut f32, pInterval: *mut f32) {
    let pmv = *addr_of!(pm);

    let mut footLOrg: vec3_t = [0.0; 3];
    let mut footROrg: vec3_t = [0.0; 3];
    let mut footLBot: vec3_t = [0.0; 3];
    let mut footRBot: vec3_t = [0.0; 3];
    let mut footLPoint: vec3_t = [0.0; 3];
    let mut footRPoint: vec3_t = [0.0; 3];
    let mut footMins: vec3_t = [0.0; 3];
    let mut footMaxs: vec3_t = [0.0; 3];
    let mut footLSlope: vec3_t = [0.0; 3];
    let mut footRSlope: vec3_t = [0.0; 3];

    let mut trace: trace_t = core::mem::zeroed();
    let diff: f32;
    let interval: f32;

    let mut boltMatrix: mdxaBone_t = mdxaBone_t::default();
    let mut G2Angles: vec3_t = [0.0; 3];

    VectorSet(&mut G2Angles, 0.0, (*(*pmv).ps).viewangles[YAW], 0.0);

    interval = 4.0; //?

    trap::G2API_GetBoltMatrix(
        (*pmv).ghoul2,
        0,
        (*pmv).g2Bolts_LFoot,
        &mut boltMatrix,
        &G2Angles,
        &(*(*pmv).ps).origin,
        (*pmv).cmd.serverTime,
        null_mut(),
        &(*pmv).modelScale,
    );
    footLPoint[0] = boltMatrix.matrix[0][3];
    footLPoint[1] = boltMatrix.matrix[1][3];
    footLPoint[2] = boltMatrix.matrix[2][3];

    trap::G2API_GetBoltMatrix(
        (*pmv).ghoul2,
        0,
        (*pmv).g2Bolts_RFoot,
        &mut boltMatrix,
        &G2Angles,
        &(*(*pmv).ps).origin,
        (*pmv).cmd.serverTime,
        null_mut(),
        &(*pmv).modelScale,
    );
    footRPoint[0] = boltMatrix.matrix[0][3];
    footRPoint[1] = boltMatrix.matrix[1][3];
    footRPoint[2] = boltMatrix.matrix[2][3];

    //get these on the cgame and store it, save ourselves a ghoul2 construct skel call
    VectorCopy(&footLPoint, &mut footLOrg);
    VectorCopy(&footRPoint, &mut footROrg);

    //step 2: adjust foot tag z height to bottom of bbox+1
    footLOrg[2] = (*(*pmv).ps).origin[2] + (*pmv).mins[2] + 1.0;
    footROrg[2] = (*(*pmv).ps).origin[2] + (*pmv).mins[2] + 1.0;
    VectorSet(
        &mut footLBot,
        footLOrg[0],
        footLOrg[1],
        footLOrg[2] - interval * 10.0,
    );
    VectorSet(
        &mut footRBot,
        footROrg[0],
        footROrg[1],
        footROrg[2] - interval * 10.0,
    );

    //step 3: trace down from each, find difference
    VectorSet(&mut footMins, -3.0, -3.0, 0.0);
    VectorSet(&mut footMaxs, 3.0, 3.0, 1.0);

    ((*pmv).trace.unwrap())(
        &mut trace,
        footLOrg.as_ptr(),
        footMins.as_ptr(),
        footMaxs.as_ptr(),
        footLBot.as_ptr(),
        (*(*pmv).ps).clientNum,
        (*pmv).tracemask,
    );
    VectorCopy(&trace.endpos, &mut footLBot);
    VectorCopy(&trace.plane.normal, &mut footLSlope);

    ((*pmv).trace.unwrap())(
        &mut trace,
        footROrg.as_ptr(),
        footMins.as_ptr(),
        footMaxs.as_ptr(),
        footRBot.as_ptr(),
        (*(*pmv).ps).clientNum,
        (*pmv).tracemask,
    );
    VectorCopy(&trace.endpos, &mut footRBot);
    VectorCopy(&trace.plane.normal, &mut footRSlope);

    diff = footLBot[2] - footRBot[2];

    if !pDiff.is_null() {
        *pDiff = diff;
    }
    if !pInterval.is_null() {
        *pInterval = interval;
    }
}

/// `BG_InSlopeAnim` (bg_pmove.c:4594) — is `anim` one of the slope-stand legs
/// animations (the `LEGS_LEFTUP*`/`LEGS_RIGHTUP*` set + the styled `S1`/`S3`/`S4`/`S5`
/// variants — note the source skips the `S2` series)? Pure classifier `switch`→`match`.
pub fn BG_InSlopeAnim(anim: c_int) -> qboolean {
    match anim {
        LEGS_LEFTUP1     //# On a slope with left foot 4 higher than right
        | LEGS_LEFTUP2   //# On a slope with left foot 8 higher than right
        | LEGS_LEFTUP3   //# On a slope with left foot 12 higher than right
        | LEGS_LEFTUP4   //# On a slope with left foot 16 higher than right
        | LEGS_LEFTUP5   //# On a slope with left foot 20 higher than right
        | LEGS_RIGHTUP1  //# On a slope with RIGHT foot 4 higher than left
        | LEGS_RIGHTUP2  //# On a slope with RIGHT foot 8 higher than left
        | LEGS_RIGHTUP3  //# On a slope with RIGHT foot 12 higher than left
        | LEGS_RIGHTUP4  //# On a slope with RIGHT foot 16 higher than left
        | LEGS_RIGHTUP5  //# On a slope with RIGHT foot 20 higher than left
        | LEGS_S1_LUP1
        | LEGS_S1_LUP2
        | LEGS_S1_LUP3
        | LEGS_S1_LUP4
        | LEGS_S1_LUP5
        | LEGS_S1_RUP1
        | LEGS_S1_RUP2
        | LEGS_S1_RUP3
        | LEGS_S1_RUP4
        | LEGS_S1_RUP5
        | LEGS_S3_LUP1
        | LEGS_S3_LUP2
        | LEGS_S3_LUP3
        | LEGS_S3_LUP4
        | LEGS_S3_LUP5
        | LEGS_S3_RUP1
        | LEGS_S3_RUP2
        | LEGS_S3_RUP3
        | LEGS_S3_RUP4
        | LEGS_S3_RUP5
        | LEGS_S4_LUP1
        | LEGS_S4_LUP2
        | LEGS_S4_LUP3
        | LEGS_S4_LUP4
        | LEGS_S4_LUP5
        | LEGS_S4_RUP1
        | LEGS_S4_RUP2
        | LEGS_S4_RUP3
        | LEGS_S4_RUP4
        | LEGS_S4_RUP5
        | LEGS_S5_LUP1
        | LEGS_S5_LUP2
        | LEGS_S5_LUP3
        | LEGS_S5_LUP4
        | LEGS_S5_LUP5
        | LEGS_S5_RUP1
        | LEGS_S5_RUP2
        | LEGS_S5_RUP3
        | LEGS_S5_RUP4
        | LEGS_S5_RUP5
        => QTRUE,
        _ => QFALSE,
    }
}

/// `PM_AdjustStandAnimForSlope` (bg_pmove.c:4656) — when standing still on a slope,
/// pick the matching `LEGS_*UP*` foot-offset stand variant from the two foot bolts.
/// Traces both feet (`PM_FootSlopeTrace`), maps the L−R height difference to one of the
/// 5 left-up / 5 right-up intervals, remaps that into the styled set matching the current
/// stance (`S1`/`S3`/`S4`/`S5`), steps one frame toward it (rate-limited by
/// `slopeRecalcTime`), and commits with `PM_ContinueLegsAnim`. Returns `qtrue` if it set a
/// slope anim, `qfalse` otherwise (no ghoul2 / no foot bolts / off-slope / unhandled
/// stance — the fall-through `PM_Footsteps` already handles). `qboolean`→`qboolean`.
///
/// `SLOPE_RECALC_INT` is the literal `8` (ms), matching the same `//SLOPE_RECALC_INT`
/// usage already in `PM_LegsSlopeBackTransition` below.
///
/// No oracle: reaches the engine through `PM_FootSlopeTrace` (`trap_G2API_GetBoltMatrix`
/// + `pm->trace`), so it is verified by inspection against the C, not bit-exact.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` (with a live `ps`, `trace` callback, ghoul2).
pub unsafe fn PM_AdjustStandAnimForSlope() -> qboolean {
    let pmv = *addr_of!(pm);
    let mut diff: f32 = 0.0;
    let mut interval: f32 = 0.0;
    let mut destAnim: c_int;
    let mut legsAnim: c_int;
    // #define SLOPERECALCVAR pm->ps->slopeRecalcTime //this is purely convenience

    if (*pmv).ghoul2.is_null() {
        //probably just changed models and not quite in sync yet
        return QFALSE;
    }

    if (*pmv).g2Bolts_LFoot == -1 || (*pmv).g2Bolts_RFoot == -1 {
        //need these bolts!
        return QFALSE;
    }

    //step 1: find the 2 foot tags
    PM_FootSlopeTrace(&mut diff, &mut interval);

    //step 4: based on difference, choose one of the left/right slope-match intervals
    if diff >= interval * 5.0 {
        destAnim = LEGS_LEFTUP5;
    } else if diff >= interval * 4.0 {
        destAnim = LEGS_LEFTUP4;
    } else if diff >= interval * 3.0 {
        destAnim = LEGS_LEFTUP3;
    } else if diff >= interval * 2.0 {
        destAnim = LEGS_LEFTUP2;
    } else if diff >= interval {
        destAnim = LEGS_LEFTUP1;
    } else if diff <= interval * -5.0 {
        destAnim = LEGS_RIGHTUP5;
    } else if diff <= interval * -4.0 {
        destAnim = LEGS_RIGHTUP4;
    } else if diff <= interval * -3.0 {
        destAnim = LEGS_RIGHTUP3;
    } else if diff <= interval * -2.0 {
        destAnim = LEGS_RIGHTUP2;
    } else if diff <= interval * -1.0 {
        destAnim = LEGS_RIGHTUP1;
    } else {
        return QFALSE;
    }

    legsAnim = (*(*pmv).ps).legsAnim;
    //adjust for current legs anim
    match legsAnim {
        BOTH_STAND1 | LEGS_S1_LUP1 | LEGS_S1_LUP2 | LEGS_S1_LUP3 | LEGS_S1_LUP4 | LEGS_S1_LUP5
        | LEGS_S1_RUP1 | LEGS_S1_RUP2 | LEGS_S1_RUP3 | LEGS_S1_RUP4 | LEGS_S1_RUP5 => {
            destAnim = LEGS_S1_LUP1 + (destAnim - LEGS_LEFTUP1);
        }
        BOTH_STAND2 | BOTH_SABERFAST_STANCE | BOTH_SABERSLOW_STANCE | BOTH_CROUCH1IDLE
        | BOTH_CROUCH1 | LEGS_LEFTUP1 | LEGS_LEFTUP2 | LEGS_LEFTUP3 | LEGS_LEFTUP4 | LEGS_LEFTUP5
        | LEGS_RIGHTUP1 | LEGS_RIGHTUP2 | LEGS_RIGHTUP3 | LEGS_RIGHTUP4 | LEGS_RIGHTUP5 => {
            //fine
        }
        BOTH_STAND3 | LEGS_S3_LUP1 | LEGS_S3_LUP2 | LEGS_S3_LUP3 | LEGS_S3_LUP4 | LEGS_S3_LUP5
        | LEGS_S3_RUP1 | LEGS_S3_RUP2 | LEGS_S3_RUP3 | LEGS_S3_RUP4 | LEGS_S3_RUP5 => {
            destAnim = LEGS_S3_LUP1 + (destAnim - LEGS_LEFTUP1);
        }
        BOTH_STAND4 | LEGS_S4_LUP1 | LEGS_S4_LUP2 | LEGS_S4_LUP3 | LEGS_S4_LUP4 | LEGS_S4_LUP5
        | LEGS_S4_RUP1 | LEGS_S4_RUP2 | LEGS_S4_RUP3 | LEGS_S4_RUP4 | LEGS_S4_RUP5 => {
            destAnim = LEGS_S4_LUP1 + (destAnim - LEGS_LEFTUP1);
        }
        BOTH_STAND5 | LEGS_S5_LUP1 | LEGS_S5_LUP2 | LEGS_S5_LUP3 | LEGS_S5_LUP4 | LEGS_S5_LUP5
        | LEGS_S5_RUP1 | LEGS_S5_RUP2 | LEGS_S5_RUP3 | LEGS_S5_RUP4 | LEGS_S5_RUP5 => {
            destAnim = LEGS_S5_LUP1 + (destAnim - LEGS_LEFTUP1);
        }
        // case BOTH_STAND6: default:
        _ => return QFALSE,
    }

    //step 5: based on the chosen interval and the current legsAnim, pick the correct anim
    //step 6: increment/decrement to the dest anim, not instant
    if (legsAnim >= LEGS_LEFTUP1 && legsAnim <= LEGS_LEFTUP5)
        || (legsAnim >= LEGS_S1_LUP1 && legsAnim <= LEGS_S1_LUP5)
        || (legsAnim >= LEGS_S3_LUP1 && legsAnim <= LEGS_S3_LUP5)
        || (legsAnim >= LEGS_S4_LUP1 && legsAnim <= LEGS_S4_LUP5)
        || (legsAnim >= LEGS_S5_LUP1 && legsAnim <= LEGS_S5_LUP5)
    {
        //already in left-side up
        if destAnim > legsAnim && (*(*pmv).ps).slopeRecalcTime < (*pmv).cmd.serverTime {
            legsAnim += 1;
            (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
        } else if destAnim < legsAnim && (*(*pmv).ps).slopeRecalcTime < (*pmv).cmd.serverTime {
            legsAnim -= 1;
            (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
        } else
        /*if (SLOPERECALCVAR < pm->cmd.serverTime)*/
        {
            legsAnim = destAnim;
        }

        destAnim = legsAnim;
    } else if (legsAnim >= LEGS_RIGHTUP1 && legsAnim <= LEGS_RIGHTUP5)
        || (legsAnim >= LEGS_S1_RUP1 && legsAnim <= LEGS_S1_RUP5)
        || (legsAnim >= LEGS_S3_RUP1 && legsAnim <= LEGS_S3_RUP5)
        || (legsAnim >= LEGS_S4_RUP1 && legsAnim <= LEGS_S4_RUP5)
        || (legsAnim >= LEGS_S5_RUP1 && legsAnim <= LEGS_S5_RUP5)
    {
        //already in right-side up
        if destAnim > legsAnim && (*(*pmv).ps).slopeRecalcTime < (*pmv).cmd.serverTime {
            legsAnim += 1;
            (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
        } else if destAnim < legsAnim && (*(*pmv).ps).slopeRecalcTime < (*pmv).cmd.serverTime {
            legsAnim -= 1;
            (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
        } else
        /*if (SLOPERECALCVAR < pm->cmd.serverTime)*/
        {
            legsAnim = destAnim;
        }

        destAnim = legsAnim;
    } else {
        //in a stand of some sort?
        match legsAnim {
            BOTH_STAND1 | TORSO_WEAPONREADY1 | TORSO_WEAPONREADY2 | TORSO_WEAPONREADY3
            | TORSO_WEAPONREADY10 => {
                if destAnim >= LEGS_S1_LUP1 && destAnim <= LEGS_S1_LUP5 {
                    //going into left side up
                    destAnim = LEGS_S1_LUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else if destAnim >= LEGS_S1_RUP1 && destAnim <= LEGS_S1_RUP5 {
                    //going into right side up
                    destAnim = LEGS_S1_RUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else {
                    //will never get here
                    return QFALSE;
                }
            }
            BOTH_STAND2 | BOTH_SABERFAST_STANCE | BOTH_SABERSLOW_STANCE | BOTH_CROUCH1IDLE => {
                if destAnim >= LEGS_LEFTUP1 && destAnim <= LEGS_LEFTUP5 {
                    //going into left side up
                    destAnim = LEGS_LEFTUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else if destAnim >= LEGS_RIGHTUP1 && destAnim <= LEGS_RIGHTUP5 {
                    //going into right side up
                    destAnim = LEGS_RIGHTUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else {
                    //will never get here
                    return QFALSE;
                }
            }
            BOTH_STAND3 => {
                if destAnim >= LEGS_S3_LUP1 && destAnim <= LEGS_S3_LUP5 {
                    //going into left side up
                    destAnim = LEGS_S3_LUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else if destAnim >= LEGS_S3_RUP1 && destAnim <= LEGS_S3_RUP5 {
                    //going into right side up
                    destAnim = LEGS_S3_RUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else {
                    //will never get here
                    return QFALSE;
                }
            }
            BOTH_STAND4 => {
                if destAnim >= LEGS_S4_LUP1 && destAnim <= LEGS_S4_LUP5 {
                    //going into left side up
                    destAnim = LEGS_S4_LUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else if destAnim >= LEGS_S4_RUP1 && destAnim <= LEGS_S4_RUP5 {
                    //going into right side up
                    destAnim = LEGS_S4_RUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else {
                    //will never get here
                    return QFALSE;
                }
            }
            BOTH_STAND5 => {
                if destAnim >= LEGS_S5_LUP1 && destAnim <= LEGS_S5_LUP5 {
                    //going into left side up
                    destAnim = LEGS_S5_LUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else if destAnim >= LEGS_S5_RUP1 && destAnim <= LEGS_S5_RUP5 {
                    //going into right side up
                    destAnim = LEGS_S5_RUP1;
                    (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT
                } else {
                    //will never get here
                    return QFALSE;
                }
            }
            // case BOTH_STAND6: default:
            _ => return QFALSE,
        }
    }
    //step 7: set the anim
    //PM_SetAnim( SETANIM_LEGS, destAnim, SETANIM_FLAG_NORMAL, 100 );
    PM_ContinueLegsAnim(destAnim);

    QTRUE
}

/// `PM_LegsSlopeBackTransition` (bg_pmove.c:4959) — when the legs are mid slope-stand
/// (`LEGS_*UP2..5` and the styled `S1`/`S3`/`S4`/`S5` variants), step one anim back toward
/// neutral once per `slopeRecalcTime` tick (8ms) and zero the velocity; otherwise pass the
/// desired anim through unchanged. `int`→`pub`. No oracle (pure anim/timer classifier on
/// `ps->legsAnim` + `cmd.serverTime`, the `BG_InSlopeAnim` precedent).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` (with a live `ps`).
pub unsafe fn PM_LegsSlopeBackTransition(desiredAnim: c_int) -> c_int {
    let pmv = *addr_of!(pm);
    let anim: c_int = (*(*pmv).ps).legsAnim;
    let mut resultingAnim: c_int = desiredAnim;

    match anim {
        LEGS_LEFTUP2     //# On a slope with left foot 8 higher than right
        | LEGS_LEFTUP3   //# On a slope with left foot 12 higher than right
        | LEGS_LEFTUP4   //# On a slope with left foot 16 higher than right
        | LEGS_LEFTUP5   //# On a slope with left foot 20 higher than right
        | LEGS_RIGHTUP2  //# On a slope with RIGHT foot 8 higher than left
        | LEGS_RIGHTUP3  //# On a slope with RIGHT foot 12 higher than left
        | LEGS_RIGHTUP4  //# On a slope with RIGHT foot 16 higher than left
        | LEGS_RIGHTUP5  //# On a slope with RIGHT foot 20 higher than left
        | LEGS_S1_LUP2
        | LEGS_S1_LUP3
        | LEGS_S1_LUP4
        | LEGS_S1_LUP5
        | LEGS_S1_RUP2
        | LEGS_S1_RUP3
        | LEGS_S1_RUP4
        | LEGS_S1_RUP5
        | LEGS_S3_LUP2
        | LEGS_S3_LUP3
        | LEGS_S3_LUP4
        | LEGS_S3_LUP5
        | LEGS_S3_RUP2
        | LEGS_S3_RUP3
        | LEGS_S3_RUP4
        | LEGS_S3_RUP5
        | LEGS_S4_LUP2
        | LEGS_S4_LUP3
        | LEGS_S4_LUP4
        | LEGS_S4_LUP5
        | LEGS_S4_RUP2
        | LEGS_S4_RUP3
        | LEGS_S4_RUP4
        | LEGS_S4_RUP5
        | LEGS_S5_LUP2
        | LEGS_S5_LUP3
        | LEGS_S5_LUP4
        | LEGS_S5_LUP5
        | LEGS_S5_RUP2
        | LEGS_S5_RUP3
        | LEGS_S5_RUP4
        | LEGS_S5_RUP5 => {
            if (*(*pmv).ps).slopeRecalcTime < (*pmv).cmd.serverTime {
                resultingAnim = anim - 1;
                (*(*pmv).ps).slopeRecalcTime = (*pmv).cmd.serverTime + 8; //SLOPE_RECALC_INT;
            } else {
                resultingAnim = anim;
            }
            VectorClear(&mut (*(*pmv).ps).velocity);
        }
        _ => {}
    }

    resultingAnim
}

/*
===============
PM_Footsteps
===============
*/
/// `PM_Footsteps` (bg_pmove.c:5027) — per-frame legs-anim selection + footstep/splash
/// event emission: choose the stand / crouch / walk / run / swim / roll legs animation
/// from movement, weapon/saber stance, NPC class and slope, then advance `bobCycle` and
/// fire `EV_FOOTSPLASH`/`EV_SWIM` on cycle boundaries. `static void`→`pub` (the
/// `PmoveSingle` caller).
///
/// No oracle: a playerstate/command/anim-helper-driven selector over the already-verified
/// `PM_*Anim`/`BG_*` classifiers (the move-mode-callee precedent). The `PM_AdjustStandAnimForSlope`
/// call sites stay live but currently fall through (its ghoul2 body is not yet ported — see there).
/// `footstep` is set but never read in this JKA revision (the legs cycle drives only the water
/// splash here, not `EV_FOOTSTEP`) — kept for fidelity, hence the `unused_assignments` allow.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` (with a live `ps`); `pm_entSelf`, if non-null, valid.
#[allow(unused_assignments)]
pub unsafe fn PM_Footsteps() {
    let pmv = *addr_of!(pm);
    let bobmove: f32;
    let old: c_int;
    let mut footstep: qboolean;
    let mut setAnimFlags: c_int = 0;

    if (PM_InSaberAnim((*(*pmv).ps).legsAnim) != 0 && BG_SpinningSaberAnim((*(*pmv).ps).legsAnim) == 0)
        || (*(*pmv).ps).legsAnim == BOTH_STAND1
        || (*(*pmv).ps).legsAnim == BOTH_STAND1TO2
        || (*(*pmv).ps).legsAnim == BOTH_STAND2TO1
        || (*(*pmv).ps).legsAnim == BOTH_STAND2
        || (*(*pmv).ps).legsAnim == BOTH_SABERFAST_STANCE
        || (*(*pmv).ps).legsAnim == BOTH_SABERSLOW_STANCE
        || (*(*pmv).ps).legsAnim == BOTH_BUTTON_HOLD
        || (*(*pmv).ps).legsAnim == BOTH_BUTTON_RELEASE
        || PM_LandingAnim((*(*pmv).ps).legsAnim) != 0
        || PM_PainAnim((*(*pmv).ps).legsAnim) != 0
    {
        //legs are in a saber anim, and not spinning, be sure to override it
        setAnimFlags |= SETANIM_FLAG_OVERRIDE;
    }

    //
    // calculate speed and cycle to be used for
    // all cyclic walking effects
    //
    (*pmv).xyspeed = (((*(*pmv).ps).velocity[0] * (*(*pmv).ps).velocity[0]
        + (*(*pmv).ps).velocity[1] * (*(*pmv).ps).velocity[1]) as f64)
        .sqrt() as f32;

    if (*(*pmv).ps).saberMove == LS_SPINATTACK {
        PM_ContinueLegsAnim((*(*pmv).ps).torsoAnim);
    } else if (*(*pmv).ps).groundEntityNum == ENTITYNUM_NONE {
        // airborne leaves position in cycle intact, but doesn't advance
        if (*pmv).waterlevel > 1 {
            if (*pmv).xyspeed > 60.0 {
                PM_ContinueLegsAnim(BOTH_SWIMFORWARD);
            } else {
                PM_ContinueLegsAnim(BOTH_SWIM_IDLE1);
            }
        }
        return;
    }
    // if not trying to move
    else if (*pmv).cmd.forwardmove == 0 && (*pmv).cmd.rightmove == 0 {
        if (*pmv).xyspeed < 5.0 {
            (*(*pmv).ps).bobCycle = 0; // start at beginning of cycle again
            if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
                && !(*addr_of!(pm_entSelf)).is_null()
                && (*(*addr_of!(pm_entSelf))).s.NPC_class == CLASS_RANCOR
            {
                if (*(*pmv).ps).eFlags2 & EF2_USE_ALT_ANIM != 0 {
                    //holding someone
                    PM_ContinueLegsAnim(BOTH_STAND4);
                    //PM_SetAnim(pm,SETANIM_LEGS,BOTH_STAND4,SETANIM_FLAG_NORMAL);
                } else if (*(*pmv).ps).eFlags2 & EF2_ALERTED != 0 {
                    //have an enemy or have had one since we spawned
                    PM_ContinueLegsAnim(BOTH_STAND2);
                    //PM_SetAnim(pm,SETANIM_LEGS,BOTH_STAND2,SETANIM_FLAG_NORMAL);
                } else {
                    //just stand there
                    PM_ContinueLegsAnim(BOTH_STAND1);
                    //PM_SetAnim(pm,SETANIM_LEGS,BOTH_STAND1,SETANIM_FLAG_NORMAL);
                }
            } else if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
                && !(*addr_of!(pm_entSelf)).is_null()
                && (*(*addr_of!(pm_entSelf))).s.NPC_class == CLASS_WAMPA
            {
                if (*(*pmv).ps).eFlags2 & EF2_USE_ALT_ANIM != 0 {
                    //holding a victim
                    PM_ContinueLegsAnim(BOTH_STAND2);
                    //PM_SetAnim(pm,SETANIM_LEGS,BOTH_STAND2,SETANIM_FLAG_NORMAL);
                } else {
                    //not holding a victim
                    PM_ContinueLegsAnim(BOTH_STAND1);
                    //PM_SetAnim(pm,SETANIM_LEGS,BOTH_STAND1,SETANIM_FLAG_NORMAL);
                }
            } else if (*(*pmv).ps).pm_flags & PMF_DUCKED != 0
                || (*(*pmv).ps).pm_flags & PMF_ROLLING != 0
            {
                if (*(*pmv).ps).legsAnim != BOTH_CROUCH1IDLE {
                    PM_SetAnim(SETANIM_LEGS, BOTH_CROUCH1IDLE, setAnimFlags, 100);
                } else {
                    PM_ContinueLegsAnim(BOTH_CROUCH1IDLE);
                }
            } else {
                if (*(*pmv).ps).weapon == WP_DISRUPTOR && (*(*pmv).ps).zoomMode == 1 {
                    //????  continue legs anim on a torso anim...??!!!
                    //yeah.. the anim has a valid pose for the legs, it uses it (you can't move while using disruptor)
                    PM_ContinueLegsAnim(TORSO_WEAPONREADY4);
                } else {
                    if (*(*pmv).ps).weapon == WP_SABER && BG_SabersOff((*pmv).ps) != 0 {
                        if PM_AdjustStandAnimForSlope() == 0 {
                            //PM_ContinueLegsAnim( BOTH_STAND1 );
                            PM_ContinueLegsAnim(PM_LegsSlopeBackTransition(BOTH_STAND1));
                        }
                    } else {
                        if (*(*pmv).ps).weapon != WP_SABER || PM_AdjustStandAnimForSlope() == 0 {
                            if (*(*pmv).ps).weapon == WP_SABER {
                                PM_ContinueLegsAnim(PM_LegsSlopeBackTransition(PM_GetSaberStance()));
                            } else {
                                PM_ContinueLegsAnim(PM_LegsSlopeBackTransition(
                                    WeaponReadyLegsAnim[(*(*pmv).ps).weapon as usize],
                                ));
                            }
                        }
                    }
                }
            }
        }
        return;
    }

    footstep = QFALSE;

    if (*(*pmv).ps).saberMove == LS_SPINATTACK {
        bobmove = 0.2;
        PM_ContinueLegsAnim((*(*pmv).ps).torsoAnim);
    } else if (*(*pmv).ps).pm_flags & PMF_DUCKED != 0 {
        let mut rolled: c_int = 0;

        bobmove = 0.5; // ducked characters bob much faster

        if ((PM_RunningAnim((*(*pmv).ps).legsAnim) != 0
            && VectorLengthSquared(&(*(*pmv).ps).velocity) >= 40000.0/*200*200*/)
            || PM_CanRollFromSoulCal((*pmv).ps) != 0)
            && BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) == 0
        {
            //roll!
            rolled = PM_TryRoll();
        }
        if rolled == 0 {
            //if the roll failed or didn't attempt, do standard crouching anim stuff.
            if (*(*pmv).ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                if (*(*pmv).ps).legsAnim != BOTH_CROUCH1WALKBACK {
                    PM_SetAnim(SETANIM_LEGS, BOTH_CROUCH1WALKBACK, setAnimFlags, 100);
                } else {
                    PM_ContinueLegsAnim(BOTH_CROUCH1WALKBACK);
                }
            } else {
                if (*(*pmv).ps).legsAnim != BOTH_CROUCH1WALK {
                    PM_SetAnim(SETANIM_LEGS, BOTH_CROUCH1WALK, setAnimFlags, 100);
                } else {
                    PM_ContinueLegsAnim(BOTH_CROUCH1WALK);
                }
            }
        } else {
            //otherwise send us into the roll
            (*(*pmv).ps).legsTimer = 0;
            (*(*pmv).ps).legsAnim = 0;
            PM_SetAnim(
                SETANIM_BOTH,
                rolled,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                150,
            );
            PM_AddEventWithParm(EV_ROLL, 0);
            (*pmv).maxs[2] = (*(*pmv).ps).crouchheight as f32; //CROUCH_MAXS_2;
            (*(*pmv).ps).viewheight = DEFAULT_VIEWHEIGHT;
            (*(*pmv).ps).pm_flags &= !PMF_DUCKED;
            (*(*pmv).ps).pm_flags |= PMF_ROLLING;
        }
    } else if (*(*pmv).ps).pm_flags & PMF_ROLLING != 0
        && BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) == 0
        && PM_InRollComplete((*pmv).ps, (*(*pmv).ps).legsAnim) == 0
    {
        bobmove = 0.5; // ducked characters bob much faster

        if (*(*pmv).ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
            if (*(*pmv).ps).legsAnim != BOTH_CROUCH1WALKBACK {
                PM_SetAnim(SETANIM_LEGS, BOTH_CROUCH1WALKBACK, setAnimFlags, 100);
            } else {
                PM_ContinueLegsAnim(BOTH_CROUCH1WALKBACK);
            }
        } else {
            if (*(*pmv).ps).legsAnim != BOTH_CROUCH1WALK {
                PM_SetAnim(SETANIM_LEGS, BOTH_CROUCH1WALK, setAnimFlags, 100);
            } else {
                PM_ContinueLegsAnim(BOTH_CROUCH1WALK);
            }
        }
    } else {
        let mut desiredAnim: c_int = -1;

        if ((*(*pmv).ps).legsAnim == BOTH_FORCELAND1
            || (*(*pmv).ps).legsAnim == BOTH_FORCELANDBACK1
            || (*(*pmv).ps).legsAnim == BOTH_FORCELANDRIGHT1
            || (*(*pmv).ps).legsAnim == BOTH_FORCELANDLEFT1)
            && (*(*pmv).ps).legsTimer > 0
        {
            //let it finish first
            bobmove = 0.2;
        } else if (*pmv).cmd.buttons & BUTTON_WALKING == 0 {
            //running
            bobmove = 0.4; // faster speeds bob faster
            if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
                && !(*addr_of!(pm_entSelf)).is_null()
                && (*(*addr_of!(pm_entSelf))).s.NPC_class == CLASS_WAMPA
            {
                if (*(*pmv).ps).eFlags2 & EF2_USE_ALT_ANIM != 0 {
                    //full on run, on all fours
                    desiredAnim = BOTH_RUN1;
                } else {
                    //regular, upright run
                    desiredAnim = BOTH_RUN2;
                }
            } else if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
                && !(*addr_of!(pm_entSelf)).is_null()
                && (*(*addr_of!(pm_entSelf))).s.NPC_class == CLASS_RANCOR
            {
                //no run anims
                if (*(*pmv).ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                    desiredAnim = BOTH_WALKBACK1;
                } else {
                    desiredAnim = BOTH_WALK1;
                }
            } else if (*(*pmv).ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                match (*(*pmv).ps).fd.saberAnimLevel {
                    SS_STAFF => {
                        if (*(*pmv).ps).saberHolstered > 1 {
                            //saber off
                            desiredAnim = BOTH_RUNBACK1;
                        } else {
                            //desiredAnim = BOTH_RUNBACK_STAFF;
                            //hmm.. stuff runback anim is pretty messed up for some reason.
                            desiredAnim = BOTH_RUNBACK2;
                        }
                    }
                    SS_DUAL => {
                        if (*(*pmv).ps).saberHolstered > 1 {
                            //sabers off
                            desiredAnim = BOTH_RUNBACK1;
                        } else {
                            //desiredAnim = BOTH_RUNBACK_DUAL;
                            //and so is the dual
                            desiredAnim = BOTH_RUNBACK2;
                        }
                    }
                    _ => {
                        if (*(*pmv).ps).saberHolstered != 0 {
                            //saber off
                            desiredAnim = BOTH_RUNBACK1;
                        } else {
                            desiredAnim = BOTH_RUNBACK2;
                        }
                    }
                }
            } else {
                match (*(*pmv).ps).fd.saberAnimLevel {
                    SS_STAFF => {
                        if (*(*pmv).ps).saberHolstered > 1 {
                            //blades off
                            desiredAnim = BOTH_RUN1;
                        } else if (*(*pmv).ps).saberHolstered == 1 {
                            //1 blade on
                            desiredAnim = BOTH_RUN2;
                        } else {
                            if (*(*pmv).ps).fd.forcePowersActive & (1 << FP_SPEED) != 0 {
                                desiredAnim = BOTH_RUN1;
                            } else {
                                desiredAnim = BOTH_RUN_STAFF;
                            }
                        }
                    }
                    SS_DUAL => {
                        if (*(*pmv).ps).saberHolstered > 1 {
                            //blades off
                            desiredAnim = BOTH_RUN1;
                        } else if (*(*pmv).ps).saberHolstered == 1 {
                            //1 saber on
                            desiredAnim = BOTH_RUN2;
                        } else {
                            desiredAnim = BOTH_RUN_DUAL;
                        }
                    }
                    _ => {
                        if (*(*pmv).ps).saberHolstered != 0 {
                            //saber off
                            desiredAnim = BOTH_RUN1;
                        } else {
                            desiredAnim = BOTH_RUN2;
                        }
                    }
                }
            }
            footstep = QTRUE;
        } else {
            bobmove = 0.2; // walking bobs slow
            if (*(*pmv).ps).pm_flags & PMF_BACKWARDS_RUN != 0 {
                match (*(*pmv).ps).fd.saberAnimLevel {
                    SS_STAFF => {
                        if (*(*pmv).ps).saberHolstered > 1 {
                            desiredAnim = BOTH_WALKBACK1;
                        } else if (*(*pmv).ps).saberHolstered != 0 {
                            desiredAnim = BOTH_WALKBACK2;
                        } else {
                            desiredAnim = BOTH_WALKBACK_STAFF;
                        }
                    }
                    SS_DUAL => {
                        if (*(*pmv).ps).saberHolstered > 1 {
                            desiredAnim = BOTH_WALKBACK1;
                        } else if (*(*pmv).ps).saberHolstered != 0 {
                            desiredAnim = BOTH_WALKBACK2;
                        } else {
                            desiredAnim = BOTH_WALKBACK_DUAL;
                        }
                    }
                    _ => {
                        if (*(*pmv).ps).saberHolstered != 0 {
                            desiredAnim = BOTH_WALKBACK1;
                        } else {
                            desiredAnim = BOTH_WALKBACK2;
                        }
                    }
                }
            } else {
                if (*(*pmv).ps).weapon == WP_MELEE {
                    desiredAnim = BOTH_WALK1;
                } else if BG_SabersOff((*pmv).ps) != 0 {
                    desiredAnim = BOTH_WALK1;
                } else {
                    match (*(*pmv).ps).fd.saberAnimLevel {
                        SS_STAFF => {
                            if (*(*pmv).ps).saberHolstered > 1 {
                                desiredAnim = BOTH_WALK1;
                            } else if (*(*pmv).ps).saberHolstered != 0 {
                                desiredAnim = BOTH_WALK2;
                            } else {
                                desiredAnim = BOTH_WALK_STAFF;
                            }
                        }
                        SS_DUAL => {
                            if (*(*pmv).ps).saberHolstered > 1 {
                                desiredAnim = BOTH_WALK1;
                            } else if (*(*pmv).ps).saberHolstered != 0 {
                                desiredAnim = BOTH_WALK2;
                            } else {
                                desiredAnim = BOTH_WALK_DUAL;
                            }
                        }
                        _ => {
                            if (*(*pmv).ps).saberHolstered != 0 {
                                desiredAnim = BOTH_WALK1;
                            } else {
                                desiredAnim = BOTH_WALK2;
                            }
                        }
                    }
                }
            }
        }

        if desiredAnim != -1 {
            let ires: c_int = PM_LegsSlopeBackTransition(desiredAnim);

            if (*(*pmv).ps).legsAnim != desiredAnim && ires == desiredAnim {
                PM_SetAnim(SETANIM_LEGS, desiredAnim, setAnimFlags, 100);
            } else {
                PM_ContinueLegsAnim(ires);
            }
        }
    }

    // check for footstep / splash sounds
    old = (*(*pmv).ps).bobCycle;
    (*(*pmv).ps).bobCycle = ((old as f32 + bobmove * (*addr_of!(pml)).msec as f32) as c_int) & 255;

    // if we just crossed a cycle boundary, play an apropriate footstep event
    if ((old + 64) ^ ((*(*pmv).ps).bobCycle + 64)) & 128 != 0 {
        (*(*pmv).ps).footstepTime = (*pmv).cmd.serverTime + 300;
        if (*pmv).waterlevel == 1 {
            // splashing
            PM_AddEvent(EV_FOOTSPLASH);
        } else if (*pmv).waterlevel == 2 {
            // wading / swimming at surface
            PM_AddEvent(EV_SWIM);
        } else if (*pmv).waterlevel == 3 {
            // no sound when completely underwater
        }
    }

    let _ = footstep; // (C: set above but unread in this JKA revision — kept for fidelity)
}

/// `PM_WaterEvents` (bg_pmove.c:5518) — emit the predictable water-transition events
/// (touch/leave/under/clear) as `pm->waterlevel` crosses `pml.previous_waterlevel`, and
/// (QAGAME) trace for an impact splash on fast entry/exit. `static void`→`pub` (the
/// `PmoveSingle` caller). No oracle (event-ring + trace-callback driven). The three
/// cosmetic `G_PlayEffect` splash calls are wired to their g_utils port (this module IS the
/// QAGAME build).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`; `pml` must be set for the frame.
pub unsafe fn PM_WaterEvents() {
    let pmv = *addr_of!(pm);
    // #ifdef QAGAME
    let mut impact_splash: qboolean = QFALSE;
    //
    // if just entered a water volume, play a sound
    //
    if (*addr_of!(pml)).previous_waterlevel == 0 && (*pmv).waterlevel != 0 {
        // #ifdef QAGAME
        if VectorLengthSquared(&(*(*pmv).ps).velocity) > 40000.0 {
            impact_splash = QTRUE;
        }
        // #endif
        PM_AddEvent(EV_WATER_TOUCH);
    }

    //
    // if just completely exited a water volume, play a sound
    //
    if (*addr_of!(pml)).previous_waterlevel != 0 && (*pmv).waterlevel == 0 {
        // #ifdef QAGAME
        if VectorLengthSquared(&(*(*pmv).ps).velocity) > 40000.0 {
            impact_splash = QTRUE;
        }
        // #endif
        PM_AddEvent(EV_WATER_LEAVE);
    }

    // #ifdef QAGAME
    if impact_splash == QTRUE {
        // play the splash effect
        let mut tr: trace_t = core::mem::zeroed();
        let mut start: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];

        VectorCopy(&(*(*pmv).ps).origin, &mut start);
        VectorCopy(&(*(*pmv).ps).origin, &mut end);

        // FIXME: set start and end better
        start[2] += 10.0;
        end[2] -= 40.0;

        ((*pmv).trace.unwrap())(
            &mut tr,
            start.as_ptr(),
            vec3_origin.as_ptr(),
            vec3_origin.as_ptr(),
            end.as_ptr(),
            (*(*pmv).ps).clientNum,
            MASK_WATER,
        );

        if tr.fraction < 1.0 {
            if (tr.contents & CONTENTS_LAVA) != 0 {
                G_PlayEffect(EFFECT_LAVA_SPLASH, &tr.endpos, &tr.plane.normal);
            } else if (tr.contents & CONTENTS_SLIME) != 0 {
                G_PlayEffect(EFFECT_ACID_SPLASH, &tr.endpos, &tr.plane.normal);
            } else {
                // must be water
                G_PlayEffect(EFFECT_WATER_SPLASH, &tr.endpos, &tr.plane.normal);
            }
        }
    }
    // #endif

    //
    // check for head just going under water
    //
    if (*addr_of!(pml)).previous_waterlevel != 3 && (*pmv).waterlevel == 3 {
        PM_AddEvent(EV_WATER_UNDER);
    }

    //
    // check for head just coming out of water
    //
    if (*addr_of!(pml)).previous_waterlevel == 3 && (*pmv).waterlevel != 3 {
        PM_AddEvent(EV_WATER_CLEAR);
    }
}

/// `BG_ClearRocketLock` (bg_pmove.c, PC-new) — reset the rocket-launcher alt-fire lock-on
/// state on a playerstate (no-op when `ps` is null). Carried over from PC JKA; absent from
/// the Xbox source. `void`→`pub`, no oracle (plain field clears).
///
/// # Safety
/// `ps` may be null (the early-out matches the C); otherwise must point to a live
/// `playerState_t`.
pub unsafe fn BG_ClearRocketLock(ps: *mut playerState_t) {
    if !ps.is_null() {
        (*ps).rocketLockIndex = ENTITYNUM_NONE;
        (*ps).rocketLastValidTime = 0.0;
        (*ps).rocketLockTime = -1.0;
        (*ps).rocketTargetTime = 0.0;
    }
}

/// `PM_BeginWeaponChange` (bg_pmove.c:5604) — start switching to `weapon`: validate it
/// (must be a real, owned weapon and not already dropping), cancel any zoom, fire the
/// predicted `EV_CHANGE_WEAPON`, enter `WEAPON_DROPPING` with a 200 ms holster delay, and
/// play the drop-weapon torso anim. No oracle (drives [`PM_AddEventWithParm`]/[`PM_SetAnim`]).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_BeginWeaponChange(weapon: c_int) {
    let pmv = *addr_of!(pm);

    if weapon <= WP_NONE || weapon >= WP_NUM_WEAPONS {
        return;
    }

    if (*(*pmv).ps).stats[STAT_WEAPONS as usize] & (1 << weapon) == 0 {
        return;
    }

    if (*(*pmv).ps).weaponstate == WEAPON_DROPPING {
        return;
    }

    // turn of any kind of zooming when weapon switching.
    if (*(*pmv).ps).zoomMode != 0 {
        (*(*pmv).ps).zoomMode = 0;
        (*(*pmv).ps).zoomTime = (*(*pmv).ps).commandTime;
    }

    PM_AddEventWithParm(EV_CHANGE_WEAPON, weapon);
    (*(*pmv).ps).weaponstate = WEAPON_DROPPING;
    (*(*pmv).ps).weaponTime += 200;
    //PM_StartTorsoAnim( TORSO_DROPWEAP1 );
    PM_SetAnim(SETANIM_TORSO, TORSO_DROPWEAP1, SETANIM_FLAG_OVERRIDE, 0);

    BG_ClearRocketLock((*pmv).ps);
}

/// `PM_FinishWeaponChange` (bg_pmove.c:5637) — complete the switch: clamp the commanded
/// weapon to a real, owned one (else `WP_NONE`), draw the saber via [`PM_SetSaberMove`]
/// (`LS_DRAW`) or play the raise-weapon torso anim, then commit `weapon`, enter
/// `WEAPON_RAISING` and add the 250 ms raise delay. `pm->cmd.weapon` (byte)→`c_int`.
/// No oracle.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_FinishWeaponChange() {
    let pmv = *addr_of!(pm);

    let mut weapon: c_int = (*pmv).cmd.weapon as c_int;
    if weapon < WP_NONE || weapon >= WP_NUM_WEAPONS {
        weapon = WP_NONE;
    }

    if (*(*pmv).ps).stats[STAT_WEAPONS as usize] & (1 << weapon) == 0 {
        weapon = WP_NONE;
    }

    if weapon == WP_SABER {
        PM_SetSaberMove(LS_DRAW as c_short);
    } else {
        //PM_StartTorsoAnim( TORSO_RAISEWEAP1);
        PM_SetAnim(SETANIM_TORSO, TORSO_RAISEWEAP1, SETANIM_FLAG_OVERRIDE, 0);
    }
    (*(*pmv).ps).weapon = weapon;
    (*(*pmv).ps).weaponstate = WEAPON_RAISING;
    (*(*pmv).ps).weaponTime += 250;
}

/// `PM_CanSetWeaponAnims` (bg_pmove.c:6155) — weapon torso anims may be set only when not
/// riding a vehicle. No oracle (trivial predicate).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_CanSetWeaponAnims() -> qboolean {
    let pmv = *addr_of!(pm);

    if (*(*pmv).ps).m_iVehicleNum != 0 {
        return QFALSE;
    }

    QTRUE
}

/// `PM_VehicleWeaponAnimate` (bg_pmove.c:6167) — set the torso/legs anim for a player
/// riding a non-walker, non-fighter vehicle, from vehicle type + weapon + input. Firing
/// picks the saber/blaster left/right/front attack (saber also fires `EV_SABER_ATTACK`
/// and forces an `LS_R_TL2BR` trail); a reversing tauntaun/speeder gets its reverse anim;
/// otherwise the per-weapon idle. The chosen `BOTH_VS_*` anim is then remapped to the
/// tauntaun's `BOTH_VT_*` set when riding a `VH_ANIMAL`, and applied via [`PM_SetAnim`].
///
/// No oracle: drives the `pm`/`pm_entVeh` vehicle/pilot pointer chain + the [`PM_AddEvent`]
/// / [`PM_SetAnim`] callbacks + the `PM_irand_timesync` RNG. Verified by inspection vs JKA
/// C. The C `goto backAgain` (saber alt-attack clears `BUTTON_ALT_ATTACK` then re-tests) is
/// carried as a `loop { … continue / break }` around the button-classify chain. The four
/// retail-disabled `if (0)` branches (VEH_FLYING / VEH_CRASHING anims) are carried verbatim
/// as `if false` so the `m_ulFlags &= ~VEH_CRASHING` side effects stay documented but dead.
///
/// # Safety
/// `pm` (with live `ps`) and, when riding, `pm_entVeh` + its vehicle/pilot chain must be valid.
pub unsafe fn PM_VehicleWeaponAnimate() {
    let pmv = *addr_of!(pm);
    let veh: *mut bgEntity_t = *addr_of!(pm_entVeh);
    let pVeh: *mut Vehicle_t;
    let mut iFlags: c_int = 0;
    let mut iBlend: c_int = 0;
    let mut Anim: c_int = -1;

    if veh.is_null()
        || (*veh).m_pVehicle.is_null()
        || (*(*veh).m_pVehicle).m_pPilot.is_null()
        || (*(*(*veh).m_pVehicle).m_pPilot).playerState.is_null()
        || (*(*pmv).ps).clientNum != (*(*(*(*veh).m_pVehicle).m_pPilot).playerState).clientNum
    {
        //make sure the vehicle exists, and its pilot is this player
        return;
    }

    pVeh = (*veh).m_pVehicle;

    if (*(*pVeh).m_pVehicleInfo).r#type == VH_WALKER
        || (*(*pVeh).m_pVehicleInfo).r#type == VH_FIGHTER
    {
        //slightly hacky I guess, but whatever.
        return;
    }
    // C: `backAgain:` label + `goto backAgain` (saber alt-attack) — modeled as a loop.
    loop {
        // If they're firing, play the right fire animation.
        if (*pmv).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) != 0 {
            iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
            iBlend = 200;

            match (*(*pmv).ps).weapon {
                WP_SABER => {
                    if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                        //don't do anything.. I guess.
                        (*pmv).cmd.buttons &= !BUTTON_ALT_ATTACK;
                        continue; // goto backAgain
                    }
                    // If we're already in an attack animation, leave (let it continue).
                    if (*(*pmv).ps).torsoTimer <= 0 {
                        //we'll be starting a new attack
                        PM_AddEvent(EV_SABER_ATTACK);
                    }

                    //just set it to something so we have a proper trail. This is a stupid
                    //hack (much like the rest of this function)
                    (*(*pmv).ps).saberMove = LS_R_TL2BR;

                    if (*(*pmv).ps).torsoTimer > 0
                        && ((*(*pmv).ps).torsoAnim == BOTH_VS_ATR_S
                            || (*(*pmv).ps).torsoAnim == BOTH_VS_ATL_S)
                    {
                        /*
                        //FIXME: no need to even call the PM_SetAnim at all in this case
                        Anim = (animNumber_t)pm->ps->torsoAnim;
                        iFlags = SETANIM_FLAG_NORMAL;
                        break;
                        */
                        return;
                    }

                    // Start the attack.
                    if (*pmv).cmd.rightmove > 0 {
                        //right side attack
                        Anim = BOTH_VS_ATR_S;
                    } else if (*pmv).cmd.rightmove < 0 {
                        //left-side attack
                        Anim = BOTH_VS_ATL_S;
                    } else
                    //random
                    {
                        //FIXME: alternate back and forth or auto-aim?
                        //if ( !Q_irand( 0, 1 ) )
                        if PM_irand_timesync(0, 1) == 0 {
                            Anim = BOTH_VS_ATR_S;
                        } else {
                            Anim = BOTH_VS_ATL_S;
                        }
                    }

                    if (*(*pmv).ps).torsoTimer <= 0 {
                        //restart the anim if we are already in it (and finished)
                        iFlags |= SETANIM_FLAG_RESTART;
                    }
                }
                WP_BLASTER => {
                    // Override the shoot anim.
                    if (*(*pmv).ps).torsoAnim == BOTH_ATTACK3 {
                        if (*pmv).cmd.rightmove > 0 {
                            //right side attack
                            Anim = BOTH_VS_ATR_G;
                        } else if (*pmv).cmd.rightmove < 0 {
                            //left side
                            Anim = BOTH_VS_ATL_G;
                        } else {
                            //frontal
                            Anim = BOTH_VS_ATF_G;
                        }
                    }
                }
                _ => {
                    Anim = BOTH_VS_IDLE;
                }
            }
        } else if !(*veh).playerState.is_null()
            && (*(*veh).playerState).speed < 0.0
            && (*(*pVeh).m_pVehicleInfo).r#type == VH_ANIMAL
        {
            //tauntaun is going backwards
            Anim = BOTH_VT_WALK_REV;
            iBlend = 600;
        } else if !(*veh).playerState.is_null()
            && (*(*veh).playerState).speed < 0.0
            && (*(*pVeh).m_pVehicleInfo).r#type == VH_SPEEDER
        {
            //speeder is going backwards
            Anim = BOTH_VS_REV;
            iBlend = 600;
        }
        // They're not firing so play the Idle for the weapon.
        else {
            iFlags = SETANIM_FLAG_NORMAL;

            match (*(*pmv).ps).weapon {
                WP_SABER => {
                    if BG_SabersOff((*pmv).ps) != QFALSE {
                        //saber holstered, normal idle
                        Anim = BOTH_VS_IDLE;
                    }
                    // In the Air.
                    //else if ( pVeh->m_ulFlags & VEH_FLYING )
                    else if false {
                        iBlend = 800;
                        Anim = BOTH_VS_AIR_G;
                        iFlags = SETANIM_FLAG_OVERRIDE;
                    }
                    // Crashing.
                    //else if ( pVeh->m_ulFlags & VEH_CRASHING )
                    else if false {
                        (*pVeh).m_ulFlags &= !(VEH_CRASHING as c_ulong); // Remove the flag, we are doing the animation.
                        iBlend = 800;
                        Anim = BOTH_VS_LAND_SR;
                        iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                    } else {
                        Anim = BOTH_VS_IDLE_SR;
                    }
                }
                WP_BLASTER => {
                    // In the Air.
                    //if ( pVeh->m_ulFlags & VEH_FLYING )
                    if false {
                        iBlend = 800;
                        Anim = BOTH_VS_AIR_G;
                        iFlags = SETANIM_FLAG_OVERRIDE;
                    }
                    // Crashing.
                    //else if ( pVeh->m_ulFlags & VEH_CRASHING )
                    else if false {
                        (*pVeh).m_ulFlags &= !(VEH_CRASHING as c_ulong); // Remove the flag, we are doing the animation.
                        iBlend = 800;
                        Anim = BOTH_VS_LAND_G;
                        iFlags = SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD;
                    } else {
                        Anim = BOTH_VS_IDLE_G;
                    }
                }
                _ => {
                    Anim = BOTH_VS_IDLE;
                }
            }
        }
        break;
    }

    if Anim != -1 {
        //override it
        if (*(*pVeh).m_pVehicleInfo).r#type == VH_ANIMAL {
            //agh.. remap anims for the tauntaun
            match Anim {
                BOTH_VS_IDLE => {
                    if !(*veh).playerState.is_null() && (*(*veh).playerState).speed > 0.0 {
                        if (*(*veh).playerState).speed > (*(*pVeh).m_pVehicleInfo).speedMax {
                            //turbo
                            Anim = BOTH_VT_TURBO;
                        } else {
                            Anim = BOTH_VT_RUN_FWD;
                        }
                    } else {
                        Anim = BOTH_VT_IDLE;
                    }
                }
                BOTH_VS_ATR_S => Anim = BOTH_VT_ATR_S,
                BOTH_VS_ATL_S => Anim = BOTH_VT_ATL_S,
                BOTH_VS_ATR_G => Anim = BOTH_VT_ATR_G,
                BOTH_VS_ATL_G => Anim = BOTH_VT_ATL_G,
                BOTH_VS_ATF_G => Anim = BOTH_VT_ATF_G,
                BOTH_VS_IDLE_SL => Anim = BOTH_VT_IDLE_S,
                BOTH_VS_IDLE_SR => Anim = BOTH_VT_IDLE_S,
                BOTH_VS_IDLE_G => Anim = BOTH_VT_IDLE_G,

                //should not happen for tauntaun:
                BOTH_VS_AIR_G | BOTH_VS_LAND_SL | BOTH_VS_LAND_SR | BOTH_VS_LAND_G => return,
                _ => {}
            }
        }

        PM_SetAnim(SETANIM_BOTH, Anim, iFlags, iBlend);
    }
}

/// `MAX_XHAIR_DIST_ACCURACY` (bg_pmove.c) — max crosshair auto-aim trace distance.
const MAX_XHAIR_DIST_ACCURACY: f32 = 20000.0;

/// `BG_VehTraceFromCamPos` (bg_pmove.c, PC-new) — for a vehicle homing-weapon lock, run a
/// second trace from the camera position (not the muzzle) and, if it strikes something
/// closer than the main trace's hit (and past the vehicle's auto-aim minimum), adopt that
/// result. Returns `entityNum+1` (0 = no better hit). PC-only; absent from the Xbox source.
/// This is the `#ifdef QAGAME` build: camera position via [`WP_GetVehicleCamPos`] and the
/// trace via the engine `trap::Trace`. `int`→`pub`, no oracle (engine-trace driven).
///
/// # Safety
/// `camTrace`/`newEnd`/`shotDir` must be valid out-pointers; `bgEnt` must point to a live
/// `bgEntity_t` whose `m_pVehicle`/`m_pVehicleInfo` is set; `entOrg`/`shotStart`/`end` must
/// be valid `vec3_t` pointers.
pub unsafe fn BG_VehTraceFromCamPos(
    camTrace: *mut trace_t,
    bgEnt: *mut bgEntity_t,
    entOrg: *const vec3_t,
    shotStart: *const vec3_t,
    end: *const vec3_t,
    newEnd: *mut vec3_t,
    shotDir: *mut vec3_t,
    bestDist: f32,
) -> c_int {
    //NOTE: this MUST stay up to date with the method used in CG_ScanForCrosshairEntity (where it checks the doExtraVehTraceFromViewPos bool)
    let mut viewDir2End: vec3_t = [0.0; 3];
    let mut extraEnd: vec3_t = [0.0; 3];
    let mut camPos: vec3_t = [0.0; 3];
    let minAutoAimDist: f32;

    WP_GetVehicleCamPos(
        bgEnt as *mut gentity_t,
        (*(*bgEnt).m_pVehicle).m_pPilot as *mut gentity_t,
        &mut camPos,
    );

    minAutoAimDist = Distance(&*entOrg, &camPos)
        + ((*(*(*bgEnt).m_pVehicle).m_pVehicleInfo).length / 2.0)
        + 200.0;

    VectorCopy(&*end, &mut *newEnd);
    VectorSubtract(&*end, &camPos, &mut viewDir2End);
    VectorNormalize(&mut viewDir2End);
    VectorMA(&camPos, MAX_XHAIR_DIST_ACCURACY, &viewDir2End, &mut extraEnd);

    *camTrace = trap::Trace(
        &camPos,
        &vec3_origin,
        &vec3_origin,
        &extraEnd,
        (*bgEnt).s.number,
        CONTENTS_SOLID | CONTENTS_BODY,
    );

    if (*camTrace).allsolid == 0
        && (*camTrace).startsolid == 0
        && (*camTrace).fraction < 1.0
        && ((*camTrace).fraction * MAX_XHAIR_DIST_ACCURACY) > minAutoAimDist
        && (((*camTrace).fraction * MAX_XHAIR_DIST_ACCURACY) - Distance(&*entOrg, &camPos)) < bestDist
    {
        //this trace hit *something* that's closer than the thing the main trace hit, so use this result instead
        VectorCopy(&(*camTrace).endpos, &mut *newEnd);
        VectorSubtract(&*newEnd, &*shotStart, &mut *shotDir);
        VectorNormalize(&mut *shotDir);
        return (*camTrace).entityNum as c_int + 1;
    }
    0
}

/// `PM_RocketLock` (bg_pmove.c:5660) — drive the rocket-launcher (or vehicle homing
/// weapon) alt-fire lock-on. Trace `lockDist` units down the view (offset by the
/// muzzle for the hand weapon), and if it hits a non-cloaked player/NPC, latch
/// `rocketLockIndex`/`rocketLockTime`; otherwise drop or hold the existing lock. The
/// `rocketLock*`/`rocketTargetTime` fields are `f32`, so the integer `serverTime`/`-1`
/// stores narrow to float exactly as the C assignment does. No oracle (geometry over
/// the engine `trace`/[`PM_BGEntForNum`]).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_RocketLock(lockDist: f32, vehicleLock: qboolean) {
    // Not really a charge weapon, but we still want to delay fire until the button comes up so that we can
    //	implement our alt-fire locking stuff
    let mut ang: vec3_t = [0.0; 3];
    let mut tr: trace_t = core::mem::zeroed();

    let mut muzzleOffPoint: vec3_t = [0.0; 3];
    let mut muzzlePoint: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    if vehicleLock != QFALSE {
        AngleVectors(
            &(*ps).viewangles,
            Some(&mut forward),
            Some(&mut right),
            Some(&mut up),
        );
        VectorCopy(&(*ps).origin, &mut muzzlePoint);
        VectorMA(&muzzlePoint, lockDist, &forward, &mut ang);
    } else {
        AngleVectors(
            &(*ps).viewangles,
            Some(&mut forward),
            Some(&mut right),
            Some(&mut up),
        );

        AngleVectors(&(*ps).viewangles, Some(&mut ang), None, None);

        VectorCopy(&(*ps).origin, &mut muzzlePoint);
        VectorCopy(&WP_MuzzlePoint[WP_ROCKET_LAUNCHER as usize], &mut muzzleOffPoint);

        // The C `VectorMA(muzzlePoint, …, forward, muzzlePoint)` accumulates in place;
        // copy the source out first so the read/write don't alias (vec3_t is Copy).
        let prev = muzzlePoint;
        VectorMA(&prev, muzzleOffPoint[0], &forward, &mut muzzlePoint);
        let prev = muzzlePoint;
        VectorMA(&prev, muzzleOffPoint[1], &right, &mut muzzlePoint);
        muzzlePoint[2] += (*ps).viewheight as f32 + muzzleOffPoint[2];
        ang[0] = muzzlePoint[0] + ang[0] * lockDist;
        ang[1] = muzzlePoint[1] + ang[1] * lockDist;
        ang[2] = muzzlePoint[2] + ang[2] * lockDist;
    }

    ((*pmv).trace.unwrap())(
        &mut tr,
        muzzlePoint.as_ptr(),
        core::ptr::null(),
        core::ptr::null(),
        ang.as_ptr(),
        (*ps).clientNum,
        MASK_PLAYERSOLID,
    );

    if vehicleLock != QFALSE {
        //vehicles also do a trace from the camera point if the main one misses
        if tr.fraction >= 1.0 {
            let mut camTrace: trace_t = core::mem::zeroed();
            let mut newEnd: vec3_t = [0.0; 3];
            let mut shotDir: vec3_t = [0.0; 3];
            if BG_VehTraceFromCamPos(
                &mut camTrace,
                PM_BGEntForNum((*ps).clientNum),
                &(*ps).origin,
                &muzzlePoint,
                &tr.endpos,
                &mut newEnd,
                &mut shotDir,
                tr.fraction * lockDist,
            ) != 0
            {
                tr = camTrace;
            }
        }
    }

    if tr.fraction != 1.0
        && (tr.entityNum as c_int) < ENTITYNUM_NONE
        && tr.entityNum as c_int != (*ps).clientNum
    {
        let bgEnt: *mut bgEntity_t = PM_BGEntForNum(tr.entityNum as c_int);
        if !bgEnt.is_null() && (*bgEnt).s.powerups & PW_CLOAKED != 0 {
            (*ps).rocketLockIndex = ENTITYNUM_NONE;
            (*ps).rocketLockTime = 0.0;
        } else if !bgEnt.is_null() && ((*bgEnt).s.eType == ET_PLAYER || (*bgEnt).s.eType == ET_NPC) {
            if (*ps).rocketLockIndex == ENTITYNUM_NONE {
                (*ps).rocketLockIndex = tr.entityNum as c_int;
                (*ps).rocketLockTime = (*pmv).cmd.serverTime as f32;
            } else if (*ps).rocketLockIndex != tr.entityNum as c_int
                && (*ps).rocketTargetTime < (*pmv).cmd.serverTime as f32
            {
                (*ps).rocketLockIndex = tr.entityNum as c_int;
                (*ps).rocketLockTime = (*pmv).cmd.serverTime as f32;
            } else if (*ps).rocketLockIndex == tr.entityNum as c_int {
                if (*ps).rocketLockTime == -1.0 {
                    (*ps).rocketLockTime = (*ps).rocketLastValidTime;
                }
            }

            if (*ps).rocketLockIndex == tr.entityNum as c_int {
                (*ps).rocketTargetTime = ((*pmv).cmd.serverTime + 500) as f32;
            }
        } else if vehicleLock == QFALSE {
            if (*ps).rocketTargetTime < (*pmv).cmd.serverTime as f32 {
                (*ps).rocketLockIndex = ENTITYNUM_NONE;
                (*ps).rocketLockTime = 0.0;
            }
        }
    } else if (*ps).rocketTargetTime < (*pmv).cmd.serverTime as f32 {
        (*ps).rocketLockIndex = ENTITYNUM_NONE;
        (*ps).rocketLockTime = 0.0;
    } else {
        if (*ps).rocketLockTime != -1.0 {
            (*ps).rocketLastValidTime = (*ps).rocketLockTime;
        }
        (*ps).rocketLockTime = -1.0;
    }
}

/// `PM_DoChargedWeapons` (bg_pmove.c:5757) — per-frame charge state machine for the
/// charging weapons (bryar/concussion/bowcaster/rocket/thermal/demp2/disruptor and the
/// vehicle homing lock). While the fire button is held it advances `WEAPON_CHARGING`/
/// `WEAPON_CHARGING_ALT`, draining ammo at the per-weapon cadence, and returns `qtrue`
/// to short-circuit the rest of the weapon code; on button-up it re-presses the button
/// and raises `EF_FIRING`/`EF_ALT_FIRING` so the charged shot goes off, returning
/// `qfalse`. The C `goto rest` is rendered as a labeled block (`break 'rest`). The
/// client-only `FF_Play` force-feedback calls have no effect in the game module and are
/// kept as not-yet-ported stubs (the G_PlayEffect-stub precedent; OpenJK drops them outright).
/// No oracle (button/ammo state machine over the verified `weaponData`/[`PM_RocketLock`]).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`; `veh` may be null (checked).
pub unsafe fn PM_DoChargedWeapons(vehicleRocketLock: qboolean, veh: *mut bgEntity_t) -> qboolean {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;

    let mut charging: qboolean = QFALSE;
    let mut altFire: qboolean = QFALSE;

    if vehicleRocketLock != QFALSE {
        if (*pmv).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) != 0 {
            //actually charging
            if !veh.is_null() && !(*veh).m_pVehicle.is_null() {
                //just make sure we have this veh info
                let vinfo = (*(*veh).m_pVehicle).m_pVehicleInfo;
                if ((*pmv).cmd.buttons & BUTTON_ATTACK != 0
                    && g_vehWeaponInfo[(*vinfo).weapon[0].ID as usize].fHoming != 0.0
                    && (*ps).ammo[0] >= g_vehWeaponInfo[(*vinfo).weapon[0].ID as usize].iAmmoPerShot)
                    || ((*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0
                        && g_vehWeaponInfo[(*vinfo).weapon[1].ID as usize].fHoming != 0.0
                        && (*ps).ammo[1]
                            >= g_vehWeaponInfo[(*vinfo).weapon[1].ID as usize].iAmmoPerShot)
                {
                    //pressing the appropriate fire button for the lock-on/charging weapon
                    PM_RocketLock(16384.0, QTRUE);
                    charging = QTRUE;
                }
                if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                    altFire = QTRUE;
                }
            }
        }
        //else, let go and should fire now
    } else {
        // If you want your weapon to be a charging weapon, just set this bit up
        match (*ps).weapon {
            //------------------
            WP_BRYAR_PISTOL => {
                // alt-fire charges the weapon
                //if ( pm->gametype == GT_SIEGE )
                if true {
                    if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                        charging = QTRUE;
                        altFire = QTRUE;
                    }
                }
            }

            WP_CONCUSSION => {
                if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                    altFire = QTRUE;
                }
            }

            WP_BRYAR_OLD => {
                // alt-fire charges the weapon
                if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                    charging = QTRUE;
                    altFire = QTRUE;
                }
            }

            //------------------
            WP_BOWCASTER => {
                // primary fire charges the weapon
                if (*pmv).cmd.buttons & BUTTON_ATTACK != 0 {
                    charging = QTRUE;
                }
            }

            //------------------
            WP_ROCKET_LAUNCHER => {
                if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0
                    && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                        >= weaponData[(*ps).weapon as usize].altEnergyPerShot
                {
                    PM_RocketLock(2048.0, QFALSE);
                    charging = QTRUE;
                    altFire = QTRUE;
                }
            }

            //------------------
            WP_THERMAL => {
                if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                    altFire = QTRUE; // override default of not being an alt-fire
                    charging = QTRUE;
                } else if (*pmv).cmd.buttons & BUTTON_ATTACK != 0 {
                    charging = QTRUE;
                }
            }

            WP_DEMP2 => {
                if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                    altFire = QTRUE; // override default of not being an alt-fire
                    charging = QTRUE;
                }
            }

            WP_DISRUPTOR => {
                if (*pmv).cmd.buttons & BUTTON_ATTACK != 0
                    && (*ps).zoomMode == 1
                    && (*ps).zoomLocked != QFALSE
                {
                    if (*pmv).cmd.forwardmove == 0
                        && (*pmv).cmd.rightmove == 0
                        && (*pmv).cmd.upmove <= 0
                    {
                        charging = QTRUE;
                        altFire = QTRUE;

                        // FF_Play( fffx_StartConst );  -- NOT YET PORTED: client-only force feedback.
                    } else {
                        charging = QFALSE;
                        altFire = QFALSE;

                        // FF_Play( fffx_StopConst );  -- NOT YET PORTED: client-only force feedback.
                    }
                } else {
                    // FF_Play( fffx_StopConst );  -- NOT YET PORTED: client-only force feedback.
                }

                if (*ps).zoomMode != 1 && (*ps).weaponstate == WEAPON_CHARGING_ALT {
                    (*ps).weaponstate = WEAPON_READY;
                    charging = QFALSE;
                    altFire = QFALSE;
                    // FF_Play( fffx_StopConst );  -- NOT YET PORTED: client-only force feedback.
                }
            }

            _ => {}
        } // end switch
    }

    // set up the appropriate weapon state based on the button that's down.
    //	Note that we ALWAYS return if charging is set ( meaning the buttons are still down )
    // The C `goto rest;` jumps past `return qtrue` to the `rest:` label below — modeled
    // here as `break 'rest` out of this labeled block.
    'rest: {
        if charging != QFALSE {
            if altFire != QFALSE {
                if (*ps).weaponstate != WEAPON_CHARGING_ALT {
                    // charge isn't started, so do it now
                    (*ps).weaponstate = WEAPON_CHARGING_ALT;
                    (*ps).weaponChargeTime = (*pmv).cmd.serverTime;
                    (*ps).weaponChargeSubtractTime =
                        (*pmv).cmd.serverTime + weaponData[(*ps).weapon as usize].altChargeSubTime;

                    //#ifdef _DEBUG  Com_Printf("Starting charge\n");
                    debug_assert!((*ps).weapon > WP_NONE);
                    BG_AddPredictableEventToPlayerstate(EV_WEAPON_CHARGE_ALT, (*ps).weapon, ps);
                }

                if vehicleRocketLock != QFALSE {
                    //check vehicle ammo
                    if !veh.is_null() {
                        let vinfo = (*(*veh).m_pVehicle).m_pVehicleInfo;
                        if (*ps).ammo[1]
                            < g_vehWeaponInfo[(*vinfo).weapon[1].ID as usize].iAmmoPerShot
                        {
                            (*ps).weaponstate = WEAPON_CHARGING_ALT;
                            break 'rest;
                        }
                    }
                } else if (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                    < (weaponData[(*ps).weapon as usize].altChargeSub
                        + weaponData[(*ps).weapon as usize].altEnergyPerShot)
                {
                    (*ps).weaponstate = WEAPON_CHARGING_ALT;

                    break 'rest;
                } else if ((*pmv).cmd.serverTime - (*ps).weaponChargeTime)
                    < weaponData[(*ps).weapon as usize].altMaxCharge
                {
                    if (*ps).weaponChargeSubtractTime < (*pmv).cmd.serverTime {
                        (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] -=
                            weaponData[(*ps).weapon as usize].altChargeSub;
                        (*ps).weaponChargeSubtractTime = (*pmv).cmd.serverTime
                            + weaponData[(*ps).weapon as usize].altChargeSubTime;
                    }
                }
            } else {
                if (*ps).weaponstate != WEAPON_CHARGING {
                    // charge isn't started, so do it now
                    (*ps).weaponstate = WEAPON_CHARGING;
                    (*ps).weaponChargeTime = (*pmv).cmd.serverTime;
                    (*ps).weaponChargeSubtractTime =
                        (*pmv).cmd.serverTime + weaponData[(*ps).weapon as usize].chargeSubTime;

                    //#ifdef _DEBUG  Com_Printf("Starting charge\n");
                    BG_AddPredictableEventToPlayerstate(EV_WEAPON_CHARGE, (*ps).weapon, ps);
                }

                if vehicleRocketLock != QFALSE {
                    if !veh.is_null() {
                        //check vehicle ammo
                        let vinfo = (*(*veh).m_pVehicle).m_pVehicleInfo;
                        if (*ps).ammo[0]
                            < g_vehWeaponInfo[(*vinfo).weapon[0].ID as usize].iAmmoPerShot
                        {
                            (*ps).weaponstate = WEAPON_CHARGING;
                            break 'rest;
                        }
                    }
                } else if (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                    < (weaponData[(*ps).weapon as usize].chargeSub
                        + weaponData[(*ps).weapon as usize].energyPerShot)
                {
                    (*ps).weaponstate = WEAPON_CHARGING;

                    break 'rest;
                } else if ((*pmv).cmd.serverTime - (*ps).weaponChargeTime)
                    < weaponData[(*ps).weapon as usize].maxCharge
                {
                    if (*ps).weaponChargeSubtractTime < (*pmv).cmd.serverTime {
                        (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] -=
                            weaponData[(*ps).weapon as usize].chargeSub;
                        (*ps).weaponChargeSubtractTime = (*pmv).cmd.serverTime
                            + weaponData[(*ps).weapon as usize].chargeSubTime;
                    }
                }
            }

            return QTRUE; // short-circuit rest of weapon code
        }
    }
    // rest:
    // Only charging weapons should be able to set these states...so....
    //	let's see which fire mode we need to set up now that the buttons are up
    if (*ps).weaponstate == WEAPON_CHARGING {
        // weapon has a charge, so let us do an attack
        //#ifdef _DEBUG  Com_Printf("Firing.  Charge time=%d\n", ...);

        // dumb, but since we shoot a charged weapon on button-up, we need to repress this button for now
        (*pmv).cmd.buttons |= BUTTON_ATTACK;
        (*ps).eFlags |= EF_FIRING;
    } else if (*ps).weaponstate == WEAPON_CHARGING_ALT {
        // weapon has a charge, so let us do an alt-attack
        //#ifdef _DEBUG  Com_Printf("Firing.  Charge time=%d\n", ...);

        // dumb, but since we shoot a charged weapon on button-up, we need to repress this button for now
        (*pmv).cmd.buttons |= BUTTON_ALT_ATTACK;
        (*ps).eFlags |= EF_FIRING | EF_ALT_FIRING;
    }

    QFALSE // continue with the rest of the weapon code
}

/// `PM_ItemUsable` (bg_pmove.c:6025) — can the player use the holdable identified by
/// `forcedUse` (an `HI_*`; `0` means "the currently selected `STAT_HOLDABLE_ITEM`")
/// right now? Rejects while on a vehicle, while the use button is still held, or during
/// a private duel; per-item it checks health (medpac), drone-already-deployed (seeker),
/// and traces clear ground/space for the sentry gun and portable shield, emitting an
/// `EV_ITEMUSEFAIL` event carrying the reason on failure. Returns 1 if usable, else 0.
/// Reads the `pm` global for `pm->trace`.
///
/// # Safety
/// `ps` must be valid; `pm` must be live (in-flight `PmoveSingle`) for the trace paths.
pub unsafe fn PM_ItemUsable(ps: *mut playerState_t, mut forcedUse: c_int) -> c_int {
    let pmv = *addr_of!(pm);

    let mut fwd: vec3_t = [0.0; 3];
    let mut fwdorg: vec3_t = [0.0; 3];
    let mut dest: vec3_t = [0.0; 3];
    let mut pos: vec3_t = [0.0; 3];
    let mut yawonly: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut trtest: vec3_t = [0.0; 3];
    let mut tr: trace_t = core::mem::zeroed();

    if (*ps).m_iVehicleNum != 0 {
        return 0;
    }

    if (*ps).pm_flags & PMF_USE_ITEM_HELD != 0 {
        //force to let go first
        return 0;
    }

    if (*ps).duelInProgress != QFALSE {
        //not allowed to use holdables while in a private duel.
        return 0;
    }

    if forcedUse == 0 {
        forcedUse = bg_itemlist[(*ps).stats[STAT_HOLDABLE_ITEM as usize] as usize].giTag;
    }

    if BG_IsItemSelectable(ps, forcedUse) == QFALSE {
        return 0;
    }

    match forcedUse {
        HI_MEDPAC | HI_MEDPAC_BIG => {
            if (*ps).stats[STAT_HEALTH as usize] >= (*ps).stats[STAT_MAX_HEALTH as usize] {
                return 0;
            }
            if (*ps).stats[STAT_HEALTH as usize] <= 0 || (*ps).eFlags & EF_DEAD != 0 {
                return 0;
            }
            1
        }
        HI_SEEKER => {
            if (*ps).eFlags & EF_SEEKERDRONE != 0 {
                PM_AddEventWithParm(EV_ITEMUSEFAIL, SEEKER_ALREADYDEPLOYED);
                return 0;
            }
            1
        }
        HI_SENTRY_GUN => {
            if (*ps).fd.sentryDeployed != QFALSE {
                PM_AddEventWithParm(EV_ITEMUSEFAIL, SENTRY_ALREADYPLACED);
                return 0;
            }

            yawonly[ROLL as usize] = 0.0;
            yawonly[PITCH as usize] = 0.0;
            yawonly[YAW as usize] = (*ps).viewangles[YAW as usize];

            VectorSet(&mut mins, -8.0, -8.0, 0.0);
            VectorSet(&mut maxs, 8.0, 8.0, 24.0);

            AngleVectors(&yawonly, Some(&mut fwd), None, None);

            fwdorg[0] = (*ps).origin[0] + fwd[0] * 64.0;
            fwdorg[1] = (*ps).origin[1] + fwd[1] * 64.0;
            fwdorg[2] = (*ps).origin[2] + fwd[2] * 64.0;

            trtest[0] = fwdorg[0] + fwd[0] * 16.0;
            trtest[1] = fwdorg[1] + fwd[1] * 16.0;
            trtest[2] = fwdorg[2] + fwd[2] * 16.0;

            ((*pmv).trace.unwrap())(
                &mut tr,
                (*ps).origin.as_ptr(),
                mins.as_ptr(),
                maxs.as_ptr(),
                trtest.as_ptr(),
                (*ps).clientNum,
                MASK_PLAYERSOLID,
            );

            if (tr.fraction != 1.0 && tr.entityNum as c_int != (*ps).clientNum)
                || tr.startsolid != 0
                || tr.allsolid != 0
            {
                PM_AddEventWithParm(EV_ITEMUSEFAIL, SENTRY_NOROOM);
                return 0;
            }
            1
        }
        HI_SHIELD => {
            mins[0] = -8.0;
            mins[1] = -8.0;
            mins[2] = 0.0;

            maxs[0] = 8.0;
            maxs[1] = 8.0;
            maxs[2] = 8.0;

            AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
            fwd[2] = 0.0;
            VectorMA(&(*ps).origin, 64.0, &fwd, &mut dest);
            ((*pmv).trace.unwrap())(
                &mut tr,
                (*ps).origin.as_ptr(),
                mins.as_ptr(),
                maxs.as_ptr(),
                dest.as_ptr(),
                (*ps).clientNum,
                MASK_SHOT,
            );
            if tr.fraction > 0.9 && tr.startsolid == 0 && tr.allsolid == 0 {
                VectorCopy(&tr.endpos, &mut pos);
                VectorSet(&mut dest, pos[0], pos[1], pos[2] - 4096.0);
                ((*pmv).trace.unwrap())(
                    &mut tr,
                    pos.as_ptr(),
                    mins.as_ptr(),
                    maxs.as_ptr(),
                    dest.as_ptr(),
                    (*ps).clientNum,
                    MASK_SOLID,
                );
                if tr.startsolid == 0 && tr.allsolid == 0 {
                    return 1;
                }
            }
            PM_AddEventWithParm(EV_ITEMUSEFAIL, SHIELD_NOROOM);
            0
        }
        HI_JETPACK => 1, //check for stuff here?
        HI_HEALTHDISP => 1,
        HI_AMMODISP => 1,
        HI_EWEB => 1,
        HI_CLOAK => 1, //check for stuff here?
        _ => 1,
    }
}

/// `PM_Weapon` (bg_pmove.c:6427) — the per-frame weapon finite-state machine: resolves
/// forced weapon swaps (NPC no-weapon / lost emplaced gun / duel / true-jedi), funnels the
/// saber path through [`PM_WeaponLightsaber`], plays the force-hand-extend gesture anims,
/// runs ammo/charge/weapon-change bookkeeping, then on a held attack button emits the
/// fire event (and the melee punch/kick anims) at the per-weapon cadence.
///
/// `G_CheapWeaponFire` (the QAGAME vehicle-NPC fire-event path, :7186/:7190) and `TryGrapple`
/// (the grapple subsystem, :7269) are wired to their ports (g_active / g_cmds).
///
/// The 4 `#ifdef QAGAME` blocks take the QAGAME branch (this module IS the QAGAME build).
/// No oracle (button/anim/ammo state machine over verified [`weaponData`]/[`WeaponAttackAnim`]/
/// [`saberMoveData`] tables and ported helpers).
///
/// # Safety
/// `pm` and `pm_entSelf` must be valid for the in-flight `PmoveSingle`, with a live `pm->ps`.
unsafe fn PM_Weapon() {
    let pmv = *addr_of!(pm);
    let ps = (*pmv).ps;
    let pm_ent_self = *addr_of!(pm_entSelf);

    let mut addTime: c_int;
    // `int amount;` (bg_pmove.c:6430): the C also does `amount = weaponData[...].energyPerShot`
    // at :6939, but that store is dead — `amount` is unconditionally reassigned (alt/primary
    // energyPerShot) at :7377/:7381 before its only read at :7390 — so it lands at that live use.
    let amount: c_int;
    let mut killAfterItem: c_int = 0;
    let mut veh: *mut bgEntity_t = null_mut();
    let mut vehicleRocketLock: qboolean = QFALSE;

    // #ifdef QAGAME — this module is the QAGAME build.
    if (*ps).clientNum >= MAX_CLIENTS as c_int
        && (*ps).weapon == WP_NONE
        && (*pmv).cmd.weapon as c_int == WP_NONE
        && !pm_ent_self.is_null()
    {
        //npc with no weapon
        let gent: *mut gentity_t = pm_ent_self as *mut gentity_t;
        if (*gent).inuse != QFALSE && !(*gent).client.is_null() && (*gent).localAnimIndex == 0 {
            //humanoid
            (*ps).torsoAnim = (*ps).legsAnim;
            (*ps).torsoTimer = (*ps).legsTimer;
            return;
        }
    }
    // #endif

    if (*ps).emplacedIndex == 0 && (*ps).weapon == WP_EMPLACED_GUN {
        //oh no!
        let mut i: c_int = 0;
        let mut weap: c_int = -1;

        while i < WP_NUM_WEAPONS {
            if (*ps).stats[STAT_WEAPONS as usize] & (1 << i) != 0 && i != WP_NONE {
                //this one's good
                weap = i;
                break;
            }
            i += 1;
        }

        if weap != -1 {
            (*pmv).cmd.weapon = weap as u8;
            (*ps).weapon = weap;
            return;
        }
    }

    if (*pm_ent_self).s.NPC_class != CLASS_VEHICLE && (*ps).m_iVehicleNum != 0 {
        //riding a vehicle
        veh = *addr_of!(pm_entVeh);
        if !veh.is_null()
            && ((!(*veh).m_pVehicle.is_null()
                && (*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER)
                || (!(*veh).m_pVehicle.is_null()
                    && (*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER))
        {
            //riding a walker/fighter
            //keep saber off, do no weapon stuff at all!
            (*ps).saberHolstered = 2;
            // #ifdef QAGAME
            (*pmv).cmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
            // #endif
        }
    }

    if (*ps).weapon != WP_DISRUPTOR //not using disruptor
        && (*ps).weapon != WP_ROCKET_LAUNCHER //not using rocket launcher
        && (*ps).weapon != WP_THERMAL //not using thermals
        && (*ps).m_iVehicleNum == 0
    //not a vehicle or in a vehicle
    {
        //check for exceeding max charge time if not using disruptor or rocket launcher or thermals
        if (*ps).weaponstate == WEAPON_CHARGING_ALT {
            let timeDif = (*pmv).cmd.serverTime - (*ps).weaponChargeTime;

            if timeDif > MAX_WEAPON_CHARGE_TIME {
                (*pmv).cmd.buttons &= !BUTTON_ALT_ATTACK;
            }
        }

        if (*ps).weaponstate == WEAPON_CHARGING {
            let timeDif = (*pmv).cmd.serverTime - (*ps).weaponChargeTime;

            if timeDif > MAX_WEAPON_CHARGE_TIME {
                (*pmv).cmd.buttons &= !BUTTON_ATTACK;
            }
        }
    }

    if (*ps).forceHandExtend == HANDEXTEND_WEAPONREADY && PM_CanSetWeaponAnims() != QFALSE {
        //reset into weapon stance
        if (*ps).weapon != WP_SABER && (*ps).weapon != WP_MELEE && PM_IsRocketTrooper() == QFALSE {
            //saber handles its own anims
            if (*ps).weapon == WP_DISRUPTOR && (*ps).zoomMode == 1 {
                //PM_StartTorsoAnim( TORSO_WEAPONREADY4 );
                PM_StartTorsoAnim(TORSO_RAISEWEAP1);
            } else if (*ps).weapon == WP_EMPLACED_GUN {
                PM_StartTorsoAnim(BOTH_GUNSIT1);
            } else {
                //PM_StartTorsoAnim( WeaponReadyAnim[pm->ps->weapon] );
                PM_StartTorsoAnim(TORSO_RAISEWEAP1);
            }
        }

        //we now go into a weapon raise anim after every force hand extend.
        //this is so that my holster-view-weapon-when-hand-extend stuff works.
        (*ps).weaponstate = WEAPON_RAISING;
        (*ps).weaponTime += 250;

        (*ps).forceHandExtend = HANDEXTEND_NONE;
    } else if (*ps).forceHandExtend != HANDEXTEND_NONE {
        //nothing else should be allowed to happen during this time, including weapon fire
        // (C `int desiredAnim = 0;` — the `= 0` init is dead: every match arm assigns it.)
        let desiredAnim: c_int;
        let mut seperateOnTorso: qboolean = QFALSE;
        let mut playFullBody: qboolean = QFALSE;
        let mut desiredOnTorso: c_int = 0;

        match (*ps).forceHandExtend {
            HANDEXTEND_FORCEPUSH => {
                desiredAnim = BOTH_FORCEPUSH;
            }
            HANDEXTEND_FORCEPULL => {
                desiredAnim = BOTH_FORCEPULL;
            }
            HANDEXTEND_FORCE_HOLD => {
                if (*ps).fd.forcePowersActive & (1 << FP_GRIP) != 0 {
                    //gripping
                    desiredAnim = BOTH_FORCEGRIP_HOLD;
                } else if (*ps).fd.forcePowersActive & (1 << FP_LIGHTNING) != 0 {
                    //lightning
                    if (*ps).weapon == WP_MELEE && (*ps).activeForcePass > FORCE_LEVEL_2 {
                        //2-handed lightning
                        desiredAnim = BOTH_FORCE_2HANDEDLIGHTNING_HOLD;
                    } else {
                        desiredAnim = BOTH_FORCELIGHTNING_HOLD;
                    }
                } else if (*ps).fd.forcePowersActive & (1 << FP_DRAIN) != 0 {
                    //draining
                    desiredAnim = BOTH_FORCEGRIP_HOLD;
                } else {
                    //???
                    desiredAnim = BOTH_FORCEGRIP_HOLD;
                }
            }
            HANDEXTEND_SABERPULL => {
                desiredAnim = BOTH_SABERPULL;
            }
            HANDEXTEND_CHOKE => {
                desiredAnim = BOTH_CHOKE3; //left-handed choke
            }
            HANDEXTEND_DODGE => {
                desiredAnim = (*ps).forceDodgeAnim;
            }
            HANDEXTEND_KNOCKDOWN => {
                if (*ps).forceDodgeAnim != 0 {
                    if (*ps).forceDodgeAnim > 4 {
                        //this means that we want to play a sepereate anim on the torso
                        let originalDAnim = (*ps).forceDodgeAnim - 8; //-8 is the original legs anim
                        if originalDAnim == 2 {
                            desiredAnim = BOTH_FORCE_GETUP_B1;
                        } else if originalDAnim == 3 {
                            desiredAnim = BOTH_FORCE_GETUP_B3;
                        } else {
                            desiredAnim = BOTH_GETUP1;
                        }

                        //now specify the torso anim
                        seperateOnTorso = QTRUE;
                        desiredOnTorso = BOTH_FORCEPUSH;
                    } else if (*ps).forceDodgeAnim == 2 {
                        desiredAnim = BOTH_FORCE_GETUP_B1;
                    } else if (*ps).forceDodgeAnim == 3 {
                        desiredAnim = BOTH_FORCE_GETUP_B3;
                    } else {
                        desiredAnim = BOTH_GETUP1;
                    }
                } else {
                    desiredAnim = BOTH_KNOCKDOWN1;
                }
            }
            HANDEXTEND_DUELCHALLENGE => {
                desiredAnim = BOTH_ENGAGETAUNT;
            }
            HANDEXTEND_TAUNT => {
                desiredAnim = (*ps).forceDodgeAnim;
                if desiredAnim != BOTH_ENGAGETAUNT
                    && VectorCompare(&(*ps).velocity, &vec3_origin) != 0
                    && (*ps).groundEntityNum != ENTITYNUM_NONE
                {
                    playFullBody = QTRUE;
                }
            }
            HANDEXTEND_PRETHROW => {
                desiredAnim = BOTH_A3_TL_BR;
                playFullBody = QTRUE;
            }
            HANDEXTEND_POSTTHROW => {
                desiredAnim = BOTH_D3_TL___;
                playFullBody = QTRUE;
            }
            HANDEXTEND_PRETHROWN => {
                desiredAnim = BOTH_KNEES1;
                playFullBody = QTRUE;
            }
            HANDEXTEND_POSTTHROWN => {
                if (*ps).forceDodgeAnim != 0 {
                    desiredAnim = BOTH_FORCE_GETUP_F2;
                } else {
                    desiredAnim = BOTH_KNOCKDOWN5;
                }
                playFullBody = QTRUE;
            }
            HANDEXTEND_DRAGGING => {
                desiredAnim = BOTH_B1_BL___;
            }
            HANDEXTEND_JEDITAUNT => {
                desiredAnim = BOTH_GESTURE1;
                //playFullBody = qtrue;
            }
            //Hmm... maybe use these, too?
            //BOTH_FORCEHEAL_QUICK //quick heal (SP level 2 & 3)
            //BOTH_MINDTRICK1 // wave (maybe for mind trick 2 & 3 - whole area, and for force seeing)
            //BOTH_MINDTRICK2 // tap (maybe for mind trick 1 - one person)
            //BOTH_FORCEGRIP_START //start grip
            //BOTH_FORCEGRIP_HOLD //hold grip
            //BOTH_FORCEGRIP_RELEASE //release grip
            //BOTH_FORCELIGHTNING //quick lightning burst (level 1)
            //BOTH_FORCELIGHTNING_START //start lightning
            //BOTH_FORCELIGHTNING_HOLD //hold lightning
            //BOTH_FORCELIGHTNING_RELEASE //release lightning
            _ => {
                desiredAnim = BOTH_FORCEPUSH;
            }
        }

        if seperateOnTorso == QFALSE {
            //of seperateOnTorso, handle it after setting the legs
            PM_SetAnim(
                SETANIM_TORSO,
                desiredAnim,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                100,
            );
            (*ps).torsoTimer = 1;
        }

        if playFullBody != QFALSE {
            //sorry if all these exceptions are getting confusing. This one just means play on both legs and torso.
            PM_SetAnim(
                SETANIM_BOTH,
                desiredAnim,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                100,
            );
            (*ps).torsoTimer = 1;
            (*ps).legsTimer = (*ps).torsoTimer;
        } else if (*ps).forceHandExtend == HANDEXTEND_DODGE
            || (*ps).forceHandExtend == HANDEXTEND_KNOCKDOWN
            || ((*ps).forceHandExtend == HANDEXTEND_CHOKE
                && (*ps).groundEntityNum == ENTITYNUM_NONE)
        {
            //special case, play dodge anim on whole body, choke anim too if off ground
            if seperateOnTorso != QFALSE {
                PM_SetAnim(
                    SETANIM_LEGS,
                    desiredAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    100,
                );
                (*ps).legsTimer = 1;

                PM_SetAnim(
                    SETANIM_TORSO,
                    desiredOnTorso,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    100,
                );
                (*ps).torsoTimer = 1;
            } else {
                PM_SetAnim(
                    SETANIM_LEGS,
                    desiredAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    100,
                );
                (*ps).legsTimer = 1;
            }
        }

        return;
    }

    if BG_InSpecialJump((*ps).legsAnim) != QFALSE
        || BG_InRoll(ps, (*ps).legsAnim) != QFALSE
        || PM_InRollComplete(ps, (*ps).legsAnim) != QFALSE
    {
        /*
        if (pm->cmd.weapon != WP_MELEE &&
            pm->ps->weapon != WP_MELEE &&
            (pm->ps->stats[STAT_WEAPONS] & (1<<WP_SABER)))
        { //it's alright also if we are melee
            pm->cmd.weapon = WP_SABER;
            pm->ps->weapon = WP_SABER;
        }
        */
        if (*ps).weaponTime < (*ps).legsTimer {
            (*ps).weaponTime = (*ps).legsTimer;
        }
    }

    if (*ps).duelInProgress != QFALSE {
        (*pmv).cmd.weapon = WP_SABER as u8;
        (*ps).weapon = WP_SABER;

        if (*ps).duelTime >= (*pmv).cmd.serverTime {
            (*pmv).cmd.upmove = 0;
            (*pmv).cmd.forwardmove = 0;
            (*pmv).cmd.rightmove = 0;
        }
    }

    if (*ps).weapon == WP_SABER && (*ps).saberMove != LS_READY && (*ps).saberMove != LS_NONE {
        (*pmv).cmd.weapon = WP_SABER as u8; //don't allow switching out mid-attack
    }

    if (*ps).weapon == WP_SABER {
        //rww - we still need the item stuff, so we won't return immediately
        PM_WeaponLightsaber();
        killAfterItem = 1;
    } else if (*ps).weapon != WP_EMPLACED_GUN {
        (*ps).saberHolstered = 0;
    }

    if PM_CanSetWeaponAnims() != QFALSE
        && ((*ps).weapon == WP_THERMAL
            || (*ps).weapon == WP_TRIP_MINE
            || (*ps).weapon == WP_DET_PACK)
    {
        if (*ps).weapon == WP_THERMAL {
            if (*ps).torsoAnim == WeaponAttackAnim[(*ps).weapon as usize]
                && ((*ps).weaponTime - 200) <= 0
            {
                PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
            }
        } else if (*ps).torsoAnim == WeaponAttackAnim[(*ps).weapon as usize]
            && ((*ps).weaponTime - 700) <= 0
        {
            PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
        }
    }

    // don't allow attack until all buttons are up
    if (*ps).pm_flags & PMF_RESPAWNED != 0 {
        return;
    }

    // ignore if spectator
    if (*ps).clientNum < MAX_CLIENTS as c_int && (*ps).persistant[PERS_TEAM as usize] == TEAM_SPECTATOR {
        return;
    }

    // check for dead player
    if (*ps).stats[STAT_HEALTH as usize] <= 0 {
        (*ps).weapon = WP_NONE;
        return;
    }

    // check for item using
    if (*pmv).cmd.buttons & BUTTON_USE_HOLDABLE != 0 {
        if (*ps).pm_flags & PMF_USE_ITEM_HELD == 0 {
            if (*pm_ent_self).s.NPC_class != CLASS_VEHICLE && (*ps).m_iVehicleNum != 0 {
                //riding a vehicle, can't use holdable items, this button operates as the weapon link/unlink toggle
                return;
            }

            if (*ps).stats[STAT_HOLDABLE_ITEM as usize] == 0 {
                return;
            }

            if PM_ItemUsable(ps, 0) == 0 {
                (*ps).pm_flags |= PMF_USE_ITEM_HELD;
                return;
            } else {
                // C re-reads `bg_itemlist[ps->stats[STAT_HOLDABLE_ITEM]].giTag` inline at every
                // use below; it is constant here (STAT_HOLDABLE_ITEM is untouched until the tail),
                // so it is read once. (PM_ItemUsable does not write stats.)
                let hi_tag = bg_itemlist[(*ps).stats[STAT_HOLDABLE_ITEM as usize] as usize].giTag;

                if (*ps).stats[STAT_HOLDABLE_ITEMS as usize] & (1 << hi_tag) != 0 {
                    if hi_tag != HI_BINOCULARS
                        && hi_tag != HI_JETPACK
                        && hi_tag != HI_HEALTHDISP
                        && hi_tag != HI_AMMODISP
                        && hi_tag != HI_CLOAK
                        && hi_tag != HI_EWEB
                    {
                        //never use up the binoculars or jetpack or dispensers or cloak or ...
                        (*ps).stats[STAT_HOLDABLE_ITEMS as usize] -= 1 << hi_tag;
                    }
                } else {
                    return; //this should not happen...
                }

                (*ps).pm_flags |= PMF_USE_ITEM_HELD;
                PM_AddEvent(EV_USE_ITEM0 + hi_tag);

                if hi_tag != HI_BINOCULARS
                    && hi_tag != HI_JETPACK
                    && hi_tag != HI_HEALTHDISP
                    && hi_tag != HI_AMMODISP
                    && hi_tag != HI_CLOAK
                    && hi_tag != HI_EWEB
                {
                    (*ps).stats[STAT_HOLDABLE_ITEM as usize] = 0;
                    BG_CycleInven(ps, 1);
                }
            }
            return;
        }
    } else {
        (*ps).pm_flags &= !PMF_USE_ITEM_HELD;
    }

    /*
    if (pm->ps->weapon == WP_SABER || pm->ps->weapon == WP_MELEE)
    { //we can't toggle zoom while using saber (for obvious reasons) so make sure it's always off
        pm->ps->zoomMode = 0;
        pm->ps->zoomFov = 0;
        pm->ps->zoomLocked = qfalse;
        pm->ps->zoomLockTime = 0;
    }
    */

    if killAfterItem != 0 {
        return;
    }

    // make weapon function
    if (*ps).weaponTime > 0 {
        (*ps).weaponTime -= (*addr_of!(pml)).msec;
    }
    if (*ps).isJediMaster != QFALSE && (*ps).emplacedIndex != 0 {
        (*ps).emplacedIndex = 0;
        (*ps).saberHolstered = 0;
    }

    if (*ps).duelInProgress != QFALSE && (*ps).emplacedIndex != 0 {
        (*ps).emplacedIndex = 0;
        (*ps).saberHolstered = 0;
    }

    if (*ps).weapon == WP_EMPLACED_GUN && (*ps).emplacedIndex != 0 {
        (*pmv).cmd.weapon = WP_EMPLACED_GUN as u8; //No switch for you!
        PM_StartTorsoAnim(BOTH_GUNSIT1);
    }

    if (*ps).isJediMaster != QFALSE || (*ps).duelInProgress != QFALSE || (*ps).trueJedi != QFALSE {
        (*pmv).cmd.weapon = WP_SABER as u8;
        (*ps).weapon = WP_SABER;

        if (*ps).isJediMaster != QFALSE || (*ps).trueJedi != QFALSE {
            (*ps).stats[STAT_WEAPONS as usize] = 1 << WP_SABER;
        }
    }

    // (bg_pmove.c:6939 `amount = weaponData[pm->ps->weapon].energyPerShot;` omitted — dead store,
    //  see the `let amount` note above; `amount` is assigned at its live use further down.)

    // take an ammo away if not infinite
    if (*ps).weapon != WP_NONE
        && (*ps).weapon == (*pmv).cmd.weapon as c_int
        && ((*ps).weaponTime <= 0 || (*ps).weaponstate != WEAPON_FIRING)
    {
        if (*ps).clientNum < MAX_CLIENTS as c_int
            && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] != -1
        {
            // enough energy to fire this weapon?
            if (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                < weaponData[(*ps).weapon as usize].energyPerShot
                && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize]
                    < weaponData[(*ps).weapon as usize].altEnergyPerShot
            {
                //the weapon is out of ammo essentially because it cannot fire primary or secondary, so do the switch
                //regardless of if the player is attacking or not
                PM_AddEventWithParm(EV_NOAMMO, WP_NUM_WEAPONS + (*ps).weapon);

                if (*ps).weaponTime < 500 {
                    (*ps).weaponTime += 500;
                }
                return;
            }

            if (*ps).weapon == WP_DET_PACK
                && (*ps).hasDetPackPlanted == QFALSE
                && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] < 1
            {
                PM_AddEventWithParm(EV_NOAMMO, WP_NUM_WEAPONS + (*ps).weapon);

                if (*ps).weaponTime < 500 {
                    (*ps).weaponTime += 500;
                }
                return;
            }
        }
    }

    // check for weapon change
    // can't change if weapon is firing, but can change
    // again if lowering or raising
    if (*ps).weaponTime <= 0 || (*ps).weaponstate != WEAPON_FIRING {
        if (*ps).weapon != (*pmv).cmd.weapon as c_int {
            PM_BeginWeaponChange((*pmv).cmd.weapon as c_int);
        }
    }

    if (*ps).weaponTime > 0 {
        return;
    }

    if (*ps).weapon == WP_DISRUPTOR && (*ps).zoomMode == 1 {
        if *addr_of!(pm_cancelOutZoom) != QFALSE {
            (*ps).zoomMode = 0;
            (*ps).zoomFov = 0.0;
            (*ps).zoomLocked = QFALSE;
            (*ps).zoomLockTime = 0;
            PM_AddEvent(EV_DISRUPTOR_ZOOMSOUND);
            return;
        }

        if (*pmv).cmd.forwardmove != 0 || (*pmv).cmd.rightmove != 0 || (*pmv).cmd.upmove > 0 {
            return;
        }
    }

    // change weapon if time
    if (*ps).weaponstate == WEAPON_DROPPING {
        PM_FinishWeaponChange();
        return;
    }

    if (*ps).weaponstate == WEAPON_RAISING {
        (*ps).weaponstate = WEAPON_READY;
        if PM_CanSetWeaponAnims() != QFALSE {
            if (*ps).weapon == WP_SABER {
                PM_StartTorsoAnim(PM_GetSaberStance());
            } else if (*ps).weapon == WP_MELEE || PM_IsRocketTrooper() != QFALSE {
                PM_StartTorsoAnim((*ps).legsAnim);
            } else if (*ps).weapon == WP_DISRUPTOR && (*ps).zoomMode == 1 {
                PM_StartTorsoAnim(TORSO_WEAPONREADY4);
            } else if (*ps).weapon == WP_EMPLACED_GUN {
                PM_StartTorsoAnim(BOTH_GUNSIT1);
            } else {
                PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
            }
        }
        return;
    }

    if PM_CanSetWeaponAnims() != QFALSE
        && PM_IsRocketTrooper() == QFALSE
        && (*ps).weaponstate == WEAPON_READY
        && (*ps).weaponTime <= 0
        && ((*ps).weapon >= WP_BRYAR_PISTOL || (*ps).weapon == WP_STUN_BATON)
        && (*ps).torsoTimer <= 0
        && (*ps).torsoAnim != WeaponReadyAnim[(*ps).weapon as usize]
        && (*ps).torsoAnim != TORSO_WEAPONIDLE3
        && (*ps).weapon != WP_EMPLACED_GUN
    {
        PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
    } else if PM_CanSetWeaponAnims() != QFALSE && (*ps).weapon == WP_MELEE {
        if (*ps).weaponTime <= 0 && (*ps).forceHandExtend == HANDEXTEND_NONE {
            let mut desTAnim = (*ps).legsAnim;

            if desTAnim == BOTH_STAND1 || desTAnim == BOTH_STAND2 {
                //remap the standard standing anims for melee stance
                desTAnim = BOTH_STAND6;
            }

            if (*pmv).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) == 0 {
                //don't do this while holding attack
                if (*ps).torsoAnim != desTAnim {
                    PM_StartTorsoAnim(desTAnim);
                }
            }
        }
    } else if PM_CanSetWeaponAnims() != QFALSE && PM_IsRocketTrooper() != QFALSE {
        let desTAnim = (*ps).legsAnim;

        if (*pmv).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) == 0 {
            //don't do this while holding attack
            if (*ps).torsoAnim != desTAnim {
                PM_StartTorsoAnim(desTAnim);
            }
        }
    }

    if ((*ps).torsoAnim == TORSO_WEAPONREADY4 || (*ps).torsoAnim == BOTH_ATTACK4)
        && ((*ps).weapon != WP_DISRUPTOR || (*ps).zoomMode != 1)
    {
        if (*ps).weapon == WP_EMPLACED_GUN {
            PM_StartTorsoAnim(BOTH_GUNSIT1);
        } else if PM_CanSetWeaponAnims() != QFALSE {
            PM_StartTorsoAnim(WeaponReadyAnim[(*ps).weapon as usize]);
        }
    } else if (*ps).torsoAnim != TORSO_WEAPONREADY4
        && (*ps).torsoAnim != BOTH_ATTACK4
        && PM_CanSetWeaponAnims() != QFALSE
        && ((*ps).weapon == WP_DISRUPTOR && (*ps).zoomMode == 1)
    {
        PM_StartTorsoAnim(TORSO_WEAPONREADY4);
    }

    if (*ps).clientNum >= MAX_CLIENTS as c_int
        && !pm_ent_self.is_null()
        && (*pm_ent_self).s.NPC_class == CLASS_VEHICLE
    {
        //we are a vehicle
        veh = pm_ent_self;
    }
    if !veh.is_null() && !(*veh).m_pVehicle.is_null() {
        let vinfo = (*(*veh).m_pVehicle).m_pVehicleInfo;
        if g_vehWeaponInfo[(*vinfo).weapon[0].ID as usize].fHoming != 0.0
            || g_vehWeaponInfo[(*vinfo).weapon[1].ID as usize].fHoming != 0.0
        {
            //don't clear the rocket locking ever?
            vehicleRocketLock = QTRUE;
        }
    }

    if vehicleRocketLock == QFALSE {
        if (*ps).weapon != WP_ROCKET_LAUNCHER {
            if (*pm_ent_self).s.NPC_class != CLASS_VEHICLE && (*ps).m_iVehicleNum != 0 {
                //riding a vehicle, the vehicle will tell me my rocketlock stuff...
            } else {
                (*ps).rocketLockIndex = ENTITYNUM_NONE;
                (*ps).rocketLockTime = 0.0;
                (*ps).rocketTargetTime = 0.0;
            }
        }
    }

    if PM_DoChargedWeapons(vehicleRocketLock, veh) != QFALSE {
        // In some cases the charged weapon code may want us to short circuit the rest of the firing code
        return;
    }

    // check for fire
    if (*pmv).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK) == 0 {
        (*ps).weaponTime = 0;
        (*ps).weaponstate = WEAPON_READY;
        return;
    }

    if (*ps).weapon == WP_EMPLACED_GUN {
        addTime = weaponData[(*ps).weapon as usize].fireTime;
        (*ps).weaponTime += addTime;
        if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
            PM_AddEvent(EV_ALT_FIRE);
        } else {
            PM_AddEvent(EV_FIRE_WEAPON);
        }
        return;
    } else if (*ps).m_iVehicleNum != 0
        && !pm_ent_self.is_null()
        && (*pm_ent_self).s.NPC_class == CLASS_VEHICLE
    {
        //a vehicle NPC that has a pilot
        (*ps).weaponstate = WEAPON_FIRING;
        (*ps).weaponTime += 100;
        // #ifdef QAGAME //hack, only do it game-side. vehicle weapons don't really need predicting.
        if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
            G_CheapWeaponFire((*ps).clientNum, EV_ALT_FIRE);
        } else {
            G_CheapWeaponFire((*ps).clientNum, EV_FIRE_WEAPON);
        }
        // #endif
        /*
        addTime = weaponData[WP_EMPLACED_GUN].fireTime;
        pm->ps->weaponTime += addTime;
        if ( (pm->cmd.buttons & BUTTON_ALT_ATTACK) )
        {
            PM_AddEvent( EV_ALT_FIRE );
        }
        else
        {
            PM_AddEvent( EV_FIRE_WEAPON );
        }
        */
        return;
    }

    if (*ps).weapon == WP_DISRUPTOR
        && (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0
        && (*ps).zoomLocked == QFALSE
    {
        return;
    }

    if (*ps).weapon == WP_DISRUPTOR
        && (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0
        && (*ps).zoomMode == 2
    {
        //can't use disruptor secondary while zoomed binoculars
        return;
    }

    if (*ps).weapon == WP_DISRUPTOR && (*ps).zoomMode == 1 {
        PM_StartTorsoAnim(BOTH_ATTACK4);
    } else if (*ps).weapon == WP_MELEE {
        //special anims for standard melee attacks
        //Alternate between punches and use the anim length as weapon time.
        if (*ps).m_iVehicleNum == 0 {
            //if riding a vehicle don't do this stuff at all
            if (*pmv).debugMelee != 0
                && (*pmv).cmd.buttons & BUTTON_ATTACK != 0
                && (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0
            {
                //ok, grapple time
                // (JKA #if 0 block — experimental saber-off grapple — kept out as in C.)
                // #ifdef QAGAME
                if !pm_ent_self.is_null() {
                    if TryGrapple(pm_ent_self as *mut gentity_t) != QFALSE {
                        return;
                    }
                }
                // #endif
            } else if (*pmv).debugMelee != 0 && (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                //kicks
                if BG_KickingAnim((*ps).torsoAnim) == QFALSE
                    && BG_KickingAnim((*ps).legsAnim) == QFALSE
                {
                    let mut kickMove = PM_KickMoveForConditions();
                    if kickMove == LS_HILT_BASH {
                        //yeah.. no hilt to bash with!
                        kickMove = LS_KICK_F;
                    }

                    if kickMove != -1 {
                        if (*ps).groundEntityNum == ENTITYNUM_NONE {
                            //if in air, convert kick to an in-air kick
                            let gDist = PM_GroundDistance();
                            //let's only allow air kicks if a certain distance from the ground
                            //it's silly to be able to do them right as you land.
                            //also looks wrong to transition from a non-complete flip anim...
                            if (BG_FlippingAnim((*ps).legsAnim) == QFALSE || (*ps).legsTimer <= 0)
                                && gDist > 64.0 //strict minimum
                                && gDist > (-(*ps).velocity[2]) - 64.0
                            //make sure we are high to ground relative to downward velocity as well
                            {
                                match kickMove {
                                    LS_KICK_F => {
                                        kickMove = LS_KICK_F_AIR;
                                    }
                                    LS_KICK_B => {
                                        kickMove = LS_KICK_B_AIR;
                                    }
                                    LS_KICK_R => {
                                        kickMove = LS_KICK_R_AIR;
                                    }
                                    LS_KICK_L => {
                                        kickMove = LS_KICK_L_AIR;
                                    }
                                    _ => {
                                        //oh well, can't do any other kick move while in-air
                                        kickMove = -1;
                                    }
                                }
                            } else {
                                //off ground, but too close to ground
                                kickMove = -1;
                            }
                        }
                    }

                    if kickMove != -1 {
                        let kickAnim = saberMoveData[kickMove as usize].animToUse;

                        if kickAnim != -1 {
                            PM_SetAnim(
                                SETANIM_BOTH,
                                kickAnim,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                0,
                            );
                            if (*ps).legsAnim == kickAnim {
                                (*ps).weaponTime = (*ps).legsTimer;
                                return;
                            }
                        }
                    }
                }

                //if got here then no move to do so put torso into leg idle or whatever
                if (*ps).torsoAnim != (*ps).legsAnim {
                    PM_SetAnim(
                        SETANIM_BOTH,
                        (*ps).legsAnim,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        0,
                    );
                }
                (*ps).weaponTime = 0;
                return;
            } else {
                //just punch
                let mut desTAnim = BOTH_MELEE1;
                if (*ps).torsoAnim == BOTH_MELEE1 {
                    desTAnim = BOTH_MELEE2;
                }
                PM_StartTorsoAnim(desTAnim);

                if (*ps).torsoAnim == desTAnim {
                    (*ps).weaponTime = (*ps).torsoTimer;
                }
            }
        }
    } else {
        PM_StartTorsoAnim(WeaponAttackAnim[(*ps).weapon as usize]);
    }

    if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
        amount = weaponData[(*ps).weapon as usize].altEnergyPerShot;
    } else {
        amount = weaponData[(*ps).weapon as usize].energyPerShot;
    }

    (*ps).weaponstate = WEAPON_FIRING;

    // take an ammo away if not infinite
    if (*ps).clientNum < MAX_CLIENTS as c_int
        && (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] != -1
    {
        // enough energy to fire this weapon?
        if ((*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] - amount) >= 0 {
            (*ps).ammo[weaponData[(*ps).weapon as usize].ammoIndex as usize] -= amount;
        } else
        // Not enough energy
        {
            // Switch weapons
            if (*ps).weapon != WP_DET_PACK || (*ps).hasDetPackPlanted == QFALSE {
                PM_AddEventWithParm(EV_NOAMMO, WP_NUM_WEAPONS + (*ps).weapon);
                if (*ps).weaponTime < 500 {
                    (*ps).weaponTime += 500;
                }
            }
            return;
        }
    }

    if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
        //if ( pm->ps->weapon == WP_BRYAR_PISTOL && pm->gametype != GT_SIEGE )
        if false {
            //kind of a hack for now
            PM_AddEvent(EV_FIRE_WEAPON);
            addTime = weaponData[(*ps).weapon as usize].fireTime;
        } else if (*ps).weapon == WP_DISRUPTOR && (*ps).zoomMode != 1 {
            PM_AddEvent(EV_FIRE_WEAPON);
            addTime = weaponData[(*ps).weapon as usize].fireTime;
        } else {
            if (*ps).weapon != WP_MELEE || (*ps).m_iVehicleNum == 0 {
                //do not fire melee events at all when on vehicle
                PM_AddEvent(EV_ALT_FIRE);
            }
            addTime = weaponData[(*ps).weapon as usize].altFireTime;
        }
    } else {
        if (*ps).weapon != WP_MELEE || (*ps).m_iVehicleNum == 0 {
            //do not fire melee events at all when on vehicle
            PM_AddEvent(EV_FIRE_WEAPON);
        }
        addTime = weaponData[(*ps).weapon as usize].fireTime;
        if (*pmv).gametype == GT_SIEGE && (*ps).weapon == WP_DET_PACK {
            // were far too spammy before?  So says Rick.
            addTime *= 2;
        }
    }

    /*
    if ( pm->ps->powerups[PW_HASTE] ) {
        addTime /= 1.3;
    }
    */

    if (*ps).fd.forcePowersActive & (1 << FP_RAGE) != 0 {
        addTime = (addTime as f64 * 0.75) as c_int;
    } else if (*ps).fd.forceRageRecoveryTime > (*pmv).cmd.serverTime {
        addTime = (addTime as f64 * 1.5) as c_int;
    }

    (*ps).weaponTime += addTime;
}

/// `PM_Animate` (bg_pmove.c:7468) — on a `BUTTON_GESTURE` press, fire the taunt: when idle
/// (no torso/legs/weapon/saber-lock timers and `HANDEXTEND_NONE`) drive `HANDEXTEND_TAUNT` +
/// `BOTH_ENGAGETAUNT`, arm `forceHandExtendTime` for 1s, and raise `EV_TAUNT`. On a vehicle the
/// stale hand-extend is cleared first. `static void`→`pub` (the `PmoveSingle` caller). The TA
/// bot extra-gesture ladder is a retail `#if 0` block, carried as a comment. No oracle:
/// playerstate-mutation / `PM_AddEvent`-callback driven.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_Animate() {
    let pmv = *addr_of!(pm);
    if (*pmv).cmd.buttons & BUTTON_GESTURE != 0 {
        if (*(*pmv).ps).m_iVehicleNum != 0 {
            //eh, fine, clear it
            if (*(*pmv).ps).forceHandExtendTime < (*pmv).cmd.serverTime {
                (*(*pmv).ps).forceHandExtend = HANDEXTEND_NONE;
            }
        }

        if (*(*pmv).ps).torsoTimer < 1
            && (*(*pmv).ps).forceHandExtend == HANDEXTEND_NONE
            && (*(*pmv).ps).legsTimer < 1
            && (*(*pmv).ps).weaponTime < 1
            && (*(*pmv).ps).saberLockTime < (*pmv).cmd.serverTime
        {
            (*(*pmv).ps).forceHandExtend = HANDEXTEND_TAUNT;

            //FIXME: random taunt anims?
            (*(*pmv).ps).forceDodgeAnim = BOTH_ENGAGETAUNT;

            (*(*pmv).ps).forceHandExtendTime = (*pmv).cmd.serverTime + 1000;

            //pm->ps->weaponTime = 100;

            PM_AddEvent(EV_TAUNT);
        }
        // #if 0
        // Here's an interesting bit.  The bots in TA used buttons to do additional gestures.
        // I ripped them out because I didn't want too many buttons given the fact that I was
        // already adding some for JK2. We can always add some back in if we want though.
        //   } else if ( pm->cmd.buttons & BUTTON_GETFLAG )   -> PM_StartTorsoAnim( TORSO_GETFLAG )
        //   } else if ( pm->cmd.buttons & BUTTON_GUARDBASE )  -> PM_StartTorsoAnim( TORSO_GUARDBASE )
        //   } else if ( pm->cmd.buttons & BUTTON_PATROL )     -> PM_StartTorsoAnim( TORSO_PATROL )
        //   } else if ( pm->cmd.buttons & BUTTON_FOLLOWME )   -> PM_StartTorsoAnim( TORSO_FOLLOWME )
        //   } else if ( pm->cmd.buttons & BUTTON_AFFIRMATIVE )-> PM_StartTorsoAnim( TORSO_AFFIRMATIVE )
        //   } else if ( pm->cmd.buttons & BUTTON_NEGATIVE )   -> PM_StartTorsoAnim( TORSO_NEGATIVE )
        // #endif
    }
}

/// `PM_DropTimers` (bg_pmove.c:7536) — decrement the per-frame timing counters by
/// `pml.msec`: the misc `pm_time` (clearing `PMF_ALL_TIMES` when it expires) and the
/// `legsTimer`/`torsoTimer` animation locks (clamped at 0). `static void`→`pub`.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_DropTimers() {
    let pmv = *addr_of!(pm);
    // drop misc timing counter
    if (*(*pmv).ps).pm_time != 0 {
        if (*addr_of!(pml)).msec >= (*(*pmv).ps).pm_time {
            (*(*pmv).ps).pm_flags &= !PMF_ALL_TIMES;
            (*(*pmv).ps).pm_time = 0;
        } else {
            (*(*pmv).ps).pm_time -= (*addr_of!(pml)).msec;
        }
    }

    // drop animation counter
    if (*(*pmv).ps).legsTimer > 0 {
        (*(*pmv).ps).legsTimer -= (*addr_of!(pml)).msec;
        if (*(*pmv).ps).legsTimer < 0 {
            (*(*pmv).ps).legsTimer = 0;
        }
    }

    if (*(*pmv).ps).torsoTimer > 0 {
        (*(*pmv).ps).torsoTimer -= (*addr_of!(pml)).msec;
        if (*(*pmv).ps).torsoTimer < 0 {
            (*(*pmv).ps).torsoTimer = 0;
        }
    }
}

/// `BG_UnrestrainedPitchRoll` (bg_pmove.c:7572, `#if !defined(_XBOX) || defined(QAGAME)`
/// branch) — may this client roll/pitch its fighter without the usual view clamp? True
/// only for a real client piloting a `VH_FIGHTER` while `bg_fighterAltControl` is set.
///
/// # Safety
/// `ps` must point to a valid `playerState_t`; `pVeh` may be null (checked).
pub unsafe fn BG_UnrestrainedPitchRoll(ps: *mut playerState_t, pVeh: *mut Vehicle_t) -> qboolean {
    if (*addr_of!(bg_fighterAltControl)).integer != 0
        && (*ps).clientNum < MAX_CLIENTS as c_int //real client
        && (*ps).m_iVehicleNum != 0//in a vehicle
        && !pVeh.is_null()//valid vehicle data pointer
        && !(*pVeh).m_pVehicleInfo.is_null()//valid vehicle info
        && (*(*pVeh).m_pVehicleInfo).r#type == VH_FIGHTER
    //fighter
    //FIXME: specify per vehicle instead of assuming true for all fighters
    //FIXME: map/server setting?
    {
        // can roll and pitch without limitation!
        return QTRUE;
    }
    QFALSE
}

/// `PM_UpdateViewAngles` (bg_pmove.c:7601) — circularly clamp the command angles with the
/// player's delta angles into `ps->viewangles`. Pitch is normally clamped to ±90° (±16000
/// in short units); in a non-hyperspacing fighter both pitch and yaw are clamped to ±10°
/// around the previous rider view angles. `static` in C → `pub` here (the `PmoveSingle`
/// caller). No oracle (reads `pm_entVeh` vehicle state + the cvar). The active QAGAME body
/// is ported; the large commented-out `/* ... */` alternate definition is dropped.
///
/// # Safety
/// `pm` must point to a valid `pmove_t`; `ps`/`cmd` to valid structs.
pub unsafe fn PM_UpdateViewAngles(ps: *mut playerState_t, cmd: *const usercmd_t) {
    let mut temp: i16;

    if (*ps).pm_type == PM_INTERMISSION || (*ps).pm_type == PM_SPINTERMISSION {
        return; // no view changes at all
    }

    if (*ps).pm_type != PM_SPECTATOR && (*ps).stats[STAT_HEALTH as usize] <= 0 {
        return; // no view changes at all
    }

    let veh = *addr_of!(pm_entVeh);

    // circularly clamp the angles with deltas
    for i in 0..3 {
        temp = (*cmd).angles[i].wrapping_add((*ps).delta_angles[i]) as i16;
        // #ifdef VEH_CONTROL_SCHEME_4 (defined nowhere) — the in-fighter 22.5-degree
        // PITCH/YAW clamp block is the dead branch; the active #else build below uses the
        // BG_UnrestrainedPitchRoll-gated (near-empty) block.
        if !veh.is_null() && BG_UnrestrainedPitchRoll(ps, (*veh).m_pVehicle) != 0 {
            //in a fighter
            /*
            if ( i == ROLL )
            {//get roll from vehicle
                ps->viewangles[ROLL] = pm_entVeh->playerState->viewangles[ROLL];//->m_pVehicle->m_vOrientation[ROLL];
                continue;

            }
            */
        } else {
            if i == PITCH {
                // don't let the player look up or down more than 90 degrees
                if temp as c_int > 16000 {
                    (*ps).delta_angles[i] = 16000 - (*cmd).angles[i];
                    temp = 16000;
                } else if (temp as c_int) < -16000 {
                    (*ps).delta_angles[i] = -16000 - (*cmd).angles[i];
                    temp = -16000;
                }
            }
        }
        (*ps).viewangles[i] = SHORT2ANGLE(temp as c_int);
    }
}

// (bg_pmove.c 7671-7804 is the `#else` CGAME copy of PM_UpdateViewAngles — dropped, not
//  held for later: this is the QAGAME build, the `#ifdef QAGAME` branch above is the port.)

/// `PM_AdjustAttackStates` (bg_pmove.c:7805) — fold the fire buttons into the playerstate:
/// suppress firing while riding a walker/fighter, compute ammo `amount` for the pending
/// shot, drive the disruptor zoom state machine (`zoomMode`/`zoomLocked`/`zoomFov`), set/
/// clear `EF_FIRING`/`EF_ALT_FIRING`, and convert a zoomed-and-locked disruptor main fire
/// into an alt fire. `void`→`pub`, no oracle (playerstate / event-callback driven).
///
/// # Safety
/// `pmptr` must be valid; `pm_entSelf`/`pm_entVeh` must be set for the frame; `ps`/`cmd`
/// must be live.
pub unsafe fn PM_AdjustAttackStates(pmptr: *mut pmove_t) {
    let amount: c_int;

    if (*(*addr_of!(pm_entSelf))).s.NPC_class != CLASS_VEHICLE && (*(*pmptr).ps).m_iVehicleNum != 0
    {
        //riding a vehicle
        let veh: *mut bgEntity_t = *addr_of!(pm_entVeh);
        if !veh.is_null()
            && (!(*veh).m_pVehicle.is_null()
                && ((*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER
                    || !(*veh).m_pVehicle.is_null()
                        && (*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER))
        {
            //riding a walker/fighter
            //not firing, ever
            (*(*pmptr).ps).eFlags &= !(EF_FIRING | EF_ALT_FIRING);
            return;
        }
    }
    // get ammo usage
    if (*pmptr).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
        amount = (*(*pmptr).ps).ammo
            [weaponData[(*(*pmptr).ps).weapon as usize].ammoIndex as usize]
            - weaponData[(*(*pmptr).ps).weapon as usize].altEnergyPerShot;
    } else {
        amount = (*(*pmptr).ps).ammo
            [weaponData[(*(*pmptr).ps).weapon as usize].ammoIndex as usize]
            - weaponData[(*(*pmptr).ps).weapon as usize].energyPerShot;
    }
    let mut amount = amount;

    // disruptor alt-fire should toggle the zoom mode, but only bother doing this for the player?
    if (*(*pmptr).ps).weapon == WP_DISRUPTOR && (*(*pmptr).ps).weaponstate == WEAPON_READY {
        if (*(*pmptr).ps).eFlags & EF_ALT_FIRING == 0 && (*pmptr).cmd.buttons & BUTTON_ALT_ATTACK != 0
        /*&& pm->cmd.upmove <= 0 && !pm->cmd.forwardmove && !pm->cmd.rightmove*/
        {
            // We just pressed the alt-fire key
            if (*(*pmptr).ps).zoomMode == 0 && (*(*pmptr).ps).pm_type != PM_DEAD {
                // not already zooming, so do it now
                (*(*pmptr).ps).zoomMode = 1;
                (*(*pmptr).ps).zoomLocked = QFALSE;
                (*(*pmptr).ps).zoomFov = 80.0; //cg_fov.value;
                (*(*pmptr).ps).zoomLockTime = (*pmptr).cmd.serverTime + 50;
                PM_AddEvent(EV_DISRUPTOR_ZOOMSOUND);
            } else if (*(*pmptr).ps).zoomMode == 1
                && (*(*pmptr).ps).zoomLockTime < (*pmptr).cmd.serverTime
            {
                //check for == 1 so we can't turn binoculars off with disruptor alt fire
                // already zooming, so must be wanting to turn it off
                (*(*pmptr).ps).zoomMode = 0;
                (*(*pmptr).ps).zoomTime = (*(*pmptr).ps).commandTime;
                (*(*pmptr).ps).zoomLocked = QFALSE;
                PM_AddEvent(EV_DISRUPTOR_ZOOMSOUND);
                (*(*pmptr).ps).weaponTime = 1000;
            }
        } else if (*pmptr).cmd.buttons & BUTTON_ALT_ATTACK == 0
            && (*(*pmptr).ps).zoomLockTime < (*pmptr).cmd.serverTime
        {
            // Not pressing zoom any more
            if (*(*pmptr).ps).zoomMode != 0 {
                if (*(*pmptr).ps).zoomMode == 1 && (*(*pmptr).ps).zoomLocked == QFALSE {
                    //approximate what level the client should be zoomed at based on how long zoom was held
                    (*(*pmptr).ps).zoomFov =
                        (((*pmptr).cmd.serverTime + 50) - (*(*pmptr).ps).zoomLockTime) as f32 * 0.035;
                    if (*(*pmptr).ps).zoomFov > 50.0 {
                        (*(*pmptr).ps).zoomFov = 50.0;
                    }
                    if (*(*pmptr).ps).zoomFov < 1.0 {
                        (*(*pmptr).ps).zoomFov = 1.0;
                    }
                }
                // were zooming in, so now lock the zoom
                (*(*pmptr).ps).zoomLocked = QTRUE;
            }
        }
        //This seemed like a good idea, but apparently it confuses people. So disabled for now.
        // [commented-out alt-zoom-while-moving and movement-cancels-zoom blocks elided — see C source]

        if (*pmptr).cmd.buttons & BUTTON_ATTACK != 0 {
            // If we are zoomed, we should switch the ammo usage to the alt-fire, otherwise, we'll
            //	just use whatever ammo was selected from above
            if (*(*pmptr).ps).zoomMode != 0 {
                amount = (*(*pmptr).ps).ammo
                    [weaponData[(*(*pmptr).ps).weapon as usize].ammoIndex as usize]
                    - weaponData[(*(*pmptr).ps).weapon as usize].altEnergyPerShot;
            }
        } else {
            // alt-fire button pressing doesn't use any ammo
            amount = 0;
        }
    }
    // [commented-out non-ready-disruptor movement-cancels-zoom block elided — see C source]

    // set the firing flag for continuous beam weapons, saber will fire even if out of ammo
    if (*(*pmptr).ps).pm_flags & PMF_RESPAWNED == 0
        && (*(*pmptr).ps).pm_type != PM_INTERMISSION
        && ((*pmptr).cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) != 0
        && (amount >= 0 || (*(*pmptr).ps).weapon == WP_SABER)
    {
        if (*pmptr).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
            (*(*pmptr).ps).eFlags |= EF_ALT_FIRING;
        } else {
            (*(*pmptr).ps).eFlags &= !EF_ALT_FIRING;
        }

        // This flag should always get set, even when alt-firing
        (*(*pmptr).ps).eFlags |= EF_FIRING;
    } else {
        // Clear 'em out
        (*(*pmptr).ps).eFlags &= !(EF_FIRING | EF_ALT_FIRING);
    }

    // disruptor should convert a main fire to an alt-fire if the gun is currently zoomed
    if (*(*pmptr).ps).weapon == WP_DISRUPTOR {
        if (*pmptr).cmd.buttons & BUTTON_ATTACK != 0
            && (*(*pmptr).ps).zoomMode == 1
            && (*(*pmptr).ps).zoomLocked != QFALSE
        {
            // converting the main fire to an alt-fire
            (*pmptr).cmd.buttons |= BUTTON_ALT_ATTACK;
            (*(*pmptr).ps).eFlags |= EF_ALT_FIRING;
        } else if (*pmptr).cmd.buttons & BUTTON_ALT_ATTACK != 0
            && (*(*pmptr).ps).zoomMode == 1
            && (*(*pmptr).ps).zoomLocked != QFALSE
        {
            (*pmptr).cmd.buttons &= !BUTTON_ALT_ATTACK;
            (*(*pmptr).ps).eFlags &= !EF_ALT_FIRING;
        }
    }
}

/// `BG_CmdForRoll` (bg_pmove.c:7975) — override the move command to drive a roll / get-up-roll
/// animation: the four directional rolls push full forward/back/strafe, and the eight
/// `BOTH_GETUP_*ROLL_*` recovery rolls ramp the move based on the `legsTimer`/`torsoTimer`
/// position within the animation. `upmove` is always zeroed. `void`→`pub`, no oracle
/// (command-mutation driven; `PM_AnimLength` is the only callee).
///
/// # Safety
/// `ps` and `pCmd` must be valid pointers.
pub unsafe fn BG_CmdForRoll(ps: *mut playerState_t, anim: c_int, pCmd: *mut usercmd_t) {
    match anim {
        BOTH_ROLL_F => {
            (*pCmd).forwardmove = 127;
            (*pCmd).rightmove = 0;
        }
        BOTH_ROLL_B => {
            (*pCmd).forwardmove = -127;
            (*pCmd).rightmove = 0;
        }
        BOTH_ROLL_R => {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = 127;
        }
        BOTH_ROLL_L => {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = -127;
        }
        BOTH_GETUP_BROLL_R => {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = 48;
            //NOTE: speed is 400
        }
        BOTH_GETUP_FROLL_R => {
            if (*ps).legsTimer <= 250 {
                //end of anim
                (*pCmd).rightmove = 0;
                (*pCmd).forwardmove = 0;
            } else {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 48;
                //NOTE: speed is 400
            }
        }
        BOTH_GETUP_BROLL_L => {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = -48;
            //NOTE: speed is 400
        }
        BOTH_GETUP_FROLL_L => {
            if (*ps).legsTimer <= 250 {
                //end of anim
                (*pCmd).rightmove = 0;
                (*pCmd).forwardmove = 0;
            } else {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = -48;
                //NOTE: speed is 400
            }
        }
        BOTH_GETUP_BROLL_B => {
            if (*ps).torsoTimer <= 250 {
                //end of anim
                (*pCmd).rightmove = 0;
                (*pCmd).forwardmove = 0;
            } else if PM_AnimLength(0, (*ps).legsAnim) - (*ps).torsoTimer < 350 {
                //beginning of anim
                (*pCmd).rightmove = 0;
                (*pCmd).forwardmove = 0;
            } else {
                //FIXME: ramp down over length of anim
                (*pCmd).forwardmove = -64;
                (*pCmd).rightmove = 0;
                //NOTE: speed is 400
            }
        }
        BOTH_GETUP_FROLL_B => {
            if (*ps).torsoTimer <= 100 {
                //end of anim
                (*pCmd).rightmove = 0;
                (*pCmd).forwardmove = 0;
            } else if PM_AnimLength(0, (*ps).legsAnim) - (*ps).torsoTimer < 200 {
                //beginning of anim
                (*pCmd).rightmove = 0;
                (*pCmd).forwardmove = 0;
            } else {
                //FIXME: ramp down over length of anim
                (*pCmd).forwardmove = -64;
                (*pCmd).rightmove = 0;
                //NOTE: speed is 400
            }
        }
        BOTH_GETUP_BROLL_F => {
            if (*ps).torsoTimer <= 550 {
                //end of anim
                (*pCmd).rightmove = 0;
                (*pCmd).forwardmove = 0;
            } else if PM_AnimLength(0, (*ps).legsAnim) - (*ps).torsoTimer < 150 {
                //beginning of anim
                (*pCmd).rightmove = 0;
                (*pCmd).forwardmove = 0;
            } else {
                (*pCmd).forwardmove = 64;
                (*pCmd).rightmove = 0;
                //NOTE: speed is 400
            }
        }
        BOTH_GETUP_FROLL_F => {
            if (*ps).torsoTimer <= 100 {
                //end of anim
                (*pCmd).rightmove = 0;
                (*pCmd).forwardmove = 0;
            } else {
                //FIXME: ramp down over length of anim
                (*pCmd).forwardmove = 64;
                (*pCmd).rightmove = 0;
                //NOTE: speed is 400
            }
        }
        _ => {}
    }
    (*pCmd).upmove = 0;
}

/// `BG_AdjustClientSpeed` (bg_pmove.c:8105) — reset `ps->speed` to the server base each
/// frame (for prediction) then apply every speed modifier: dodge/knockdown stops, the
/// running-backwards penalty, force grip/speed/rage scaling, disruptor-zoom slowdown, the
/// grip-cripple penalties, the saber attack/spin/transition speed drops, and the roll
/// speed ramp keyed off `legsTimer`. `void`→`pub`, no oracle (playerstate-mutation driven;
/// `BG_SaberInAttack`/`BG_SpinningSaberAnim`/`PM_SaberInTransition`/`BG_InRoll` are ported).
///
/// The deliberate `float` vs `double` literal split in the C (`0.75` vs `0.75f`) is carried
/// faithfully: bare doubles promote `speed` to `f64` before the multiply, `f`-suffixed
/// literals stay `f32`.
///
/// # Safety
/// `ps` and `cmd` must be valid; the global `pm` must be installed for the frame (the C
/// reads `pm->ps`/`pm->cmd` directly in a few of the gates).
pub unsafe fn BG_AdjustClientSpeed(ps: *mut playerState_t, cmd: *mut usercmd_t, svTime: c_int) {
    let pmv = *addr_of!(pm);

    if (*ps).clientNum >= MAX_CLIENTS as c_int {
        let bgEnt: *mut bgEntity_t = *addr_of!(pm_entSelf);

        if !bgEnt.is_null() && (*bgEnt).s.NPC_class == CLASS_VEHICLE {
            //vehicles manage their own speed
            return;
        }
    }

    //For prediction, always reset speed back to the last known server base speed
    //If we didn't do this, under lag we'd eventually dwindle speed down to 0 even though
    //that would not be the correct predicted value.
    (*ps).speed = (*ps).basespeed as f32;

    if (*ps).forceHandExtend == HANDEXTEND_DODGE {
        (*ps).speed = 0.0;
    }

    if (*ps).forceHandExtend == HANDEXTEND_KNOCKDOWN
        || (*ps).forceHandExtend == HANDEXTEND_PRETHROWN
        || (*ps).forceHandExtend == HANDEXTEND_POSTTHROWN
    {
        (*ps).speed = 0.0;
    }

    if (*cmd).forwardmove < 0
        && (*cmd).buttons & BUTTON_WALKING == 0
        && (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE
    {
        //running backwards is slower than running forwards (like SP)
        (*ps).speed = ((*ps).speed as f64 * 0.75) as f32;
    }

    if (*ps).fd.forcePowersActive & (1 << FP_GRIP) != 0 {
        (*ps).speed = ((*ps).speed as f64 * 0.4) as f32;
    }

    if (*ps).fd.forcePowersActive & (1 << FP_SPEED) != 0 {
        (*ps).speed = ((*ps).speed as f64 * 1.7) as f32;
    } else if (*ps).fd.forcePowersActive & (1 << FP_RAGE) != 0 {
        (*ps).speed = ((*ps).speed as f64 * 1.3) as f32;
    } else if (*ps).fd.forceRageRecoveryTime > svTime {
        (*ps).speed = ((*ps).speed as f64 * 0.75) as f32;
    }

    if (*(*pmv).ps).weapon == WP_DISRUPTOR
        && (*(*pmv).ps).zoomMode == 1
        && (*(*pmv).ps).zoomLockTime < (*pmv).cmd.serverTime
    {
        (*ps).speed *= 0.5;
    }

    if (*ps).fd.forceGripCripple != 0 {
        if (*ps).fd.forcePowersActive & (1 << FP_RAGE) != 0 {
            (*ps).speed = ((*ps).speed as f64 * 0.9) as f32;
        } else if (*ps).fd.forcePowersActive & (1 << FP_SPEED) != 0 {
            //force speed will help us escape
            (*ps).speed = ((*ps).speed as f64 * 0.8) as f32;
        } else {
            (*ps).speed = ((*ps).speed as f64 * 0.2) as f32;
        }
    }

    if BG_SaberInAttack((*ps).saberMove) != QFALSE && (*cmd).forwardmove < 0 {
        //if running backwards while attacking, don't run as fast.
        match (*ps).fd.saberAnimLevel {
            FORCE_LEVEL_1 => {
                (*ps).speed *= 0.75;
            }
            FORCE_LEVEL_2 | SS_DUAL | SS_STAFF => {
                (*ps).speed *= 0.60;
            }
            FORCE_LEVEL_3 => {
                (*ps).speed *= 0.45;
            }
            _ => {}
        }
    } else if BG_SpinningSaberAnim((*ps).legsAnim) != QFALSE {
        if (*ps).fd.saberAnimLevel == FORCE_LEVEL_3 {
            (*ps).speed *= 0.3;
        } else {
            (*ps).speed *= 0.5;
        }
    } else if (*ps).weapon == WP_SABER && BG_SaberInAttack((*ps).saberMove) != QFALSE {
        //if attacking with saber while running, drop your speed
        match (*ps).fd.saberAnimLevel {
            FORCE_LEVEL_2 | SS_DUAL | SS_STAFF => {
                (*ps).speed *= 0.85;
            }
            FORCE_LEVEL_3 => {
                (*ps).speed *= 0.55;
            }
            _ => {}
        }
    } else if (*ps).weapon == WP_SABER
        && (*ps).fd.saberAnimLevel == FORCE_LEVEL_3
        && PM_SaberInTransition((*ps).saberMove) != QFALSE
    {
        //Now, we want to even slow down in transitions for level 3 (since it has chains and stuff now)
        if (*cmd).forwardmove < 0 {
            (*ps).speed *= 0.4;
        } else {
            (*ps).speed *= 0.6;
        }
    }

    if BG_InRoll(ps, (*ps).legsAnim) != QFALSE && (*ps).speed > 50.0 {
        //can't roll unless you're able to move normally
        if (*ps).legsAnim == BOTH_ROLL_B {
            //backwards roll is pretty fast, should also be slower
            if (*ps).legsTimer > 800 {
                (*ps).speed = ((*ps).legsTimer as f64 / 2.5) as f32;
            } else {
                (*ps).speed = ((*ps).legsTimer as f64 / 6.0) as f32; //450;
            }
        } else if (*ps).legsTimer > 800 {
            (*ps).speed = ((*ps).legsTimer as f64 / 1.5) as f32; //450;
        } else {
            (*ps).speed = ((*ps).legsTimer as f64 / 5.0) as f32; //450;
        }
        if (*ps).speed > 600.0 {
            (*ps).speed = 600.0;
        }
        //Automatically slow down as the roll ends.
    }

    let mut saber: *mut saberInfo_t = BG_MySaber((*ps).clientNum, 0);
    if !saber.is_null() && (*saber).moveSpeedScale != 1.0 {
        (*ps).speed *= (*saber).moveSpeedScale;
    }
    saber = BG_MySaber((*ps).clientNum, 1);
    if !saber.is_null() && (*saber).moveSpeedScale != 1.0 {
        (*ps).speed *= (*saber).moveSpeedScale;
    }
}

/// `BG_InRollAnim` (bg_pmove.c:8272) — is this entity's `legsAnim` one of the four
/// directional roll animations? Pure classifier `switch`→`match` over `cent->legsAnim`.
///
/// # Safety
/// `cent` must point to a valid `entityState_t`.
pub unsafe fn BG_InRollAnim(cent: *mut entityState_t) -> qboolean {
    match (*cent).legsAnim {
        BOTH_ROLL_F | BOTH_ROLL_B | BOTH_ROLL_R | BOTH_ROLL_L => QTRUE,
        _ => QFALSE,
    }
}

/// `BG_InKnockDown` (bg_pmove.c:8285) — is `anim` a knockdown or get-up (incl.
/// force-getup and getup-roll) animation? Pure classifier `switch`→`match`.
pub fn BG_InKnockDown(anim: c_int) -> qboolean {
    match anim {
        BOTH_KNOCKDOWN1 | BOTH_KNOCKDOWN2 | BOTH_KNOCKDOWN3 | BOTH_KNOCKDOWN4 | BOTH_KNOCKDOWN5 => {
            QTRUE
        }
        BOTH_GETUP1
        | BOTH_GETUP2
        | BOTH_GETUP3
        | BOTH_GETUP4
        | BOTH_GETUP5
        | BOTH_FORCE_GETUP_F1
        | BOTH_FORCE_GETUP_F2
        | BOTH_FORCE_GETUP_B1
        | BOTH_FORCE_GETUP_B2
        | BOTH_FORCE_GETUP_B3
        | BOTH_FORCE_GETUP_B4
        | BOTH_FORCE_GETUP_B5
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

/// `BG_InRollES` (bg_pmove.c:8322) — is `anim` one of the four directional roll
/// animations? Pure classifier `switch`→`match` (the `ps` arg is unused, as in C).
///
/// # Safety
/// `ps` must point to a valid `entityState_t` (unused, but the C signature takes it).
pub unsafe fn BG_InRollES(_ps: *mut entityState_t, anim: c_int) -> qboolean {
    match anim {
        BOTH_ROLL_F | BOTH_ROLL_B | BOTH_ROLL_R | BOTH_ROLL_L => QTRUE,
        _ => QFALSE,
    }
}

/// `BG_IK_MoveArm` (bg_pmove.c:8336) — drive the left arm of a ghoul2 instance toward
/// `desiredPos` with inverse kinematics (used to fling a victim around in a saber
/// throw). On the first active frame it initialises the IK/ragdoll effector state and
/// turns the `lhumerus`/`lradius` joints into physics-controlled joints (unrestricted
/// shoulder, restricted elbow); each subsequent frame it nudges the hand bolt toward
/// the target (speed scaled by remaining distance) and re-animates the model; once
/// `forceHalt` (or `IKMove` failing) it tears the IK state down and snaps the arm bones
/// back onto the pelvis animation. `ikInProgress` is the caller-owned latch.
///
/// The `vec3_t` params are C arrays decaying to `float*`, so they are `*mut vec3_t`
/// here (read-only in practice). The three engine scratch structs are zero-initialised
/// (C leaves them on the stack with every read field overwritten before use; only the
/// untouched `boneName` tail / `settleFrame` differ, and the engine ignores those).
///
/// # Safety
/// `ghoul2` is an opaque engine handle; `ent`, `ikInProgress`, and the four `vec3_t`
/// pointers must be valid. Issues ghoul2 engine syscalls (no oracle).
#[allow(clippy::too_many_arguments)]
pub unsafe fn BG_IK_MoveArm(
    ghoul2: *mut c_void,
    lHandBolt: c_int,
    time: c_int,
    ent: *mut entityState_t,
    basePose: c_int,
    desiredPos: *mut vec3_t,
    ikInProgress: *mut qboolean,
    origin: *mut vec3_t,
    angles: *mut vec3_t,
    scale: *mut vec3_t,
    blendTime: c_int,
    forceHalt: qboolean,
) {
    if ghoul2.is_null() {
        return;
    }

    debug_assert!((*addr_of!(bgHumanoidAnimations))[basePose as usize].firstFrame > 0);

    if *ikInProgress == QFALSE && forceHalt == QFALSE {
        let baseposeAnim = basePose;
        let mut ikP: sharedSetBoneIKStateParams_t = core::mem::MaybeUninit::zeroed().assume_init();

        //restrict the shoulder joint
        //VectorSet(ikP.pcjMins,-50.0f,-80.0f,-15.0f);
        //VectorSet(ikP.pcjMaxs,15.0f,40.0f,15.0f);

        //for now, leaving it unrestricted, but restricting elbow joint.
        //This lets us break the arm however we want in order to fling people
        //in throws, and doesn't look bad.
        VectorSet(&mut ikP.pcjMins, 0.0, 0.0, 0.0);
        VectorSet(&mut ikP.pcjMaxs, 0.0, 0.0, 0.0);

        //give the info on our entity.
        ikP.blendTime = blendTime;
        VectorCopy(&*origin, &mut ikP.origin);
        VectorCopy(&*angles, &mut ikP.angles);
        ikP.angles[PITCH] = 0.0;
        ikP.pcjOverrides = 0;
        ikP.radius = 10.0;
        VectorCopy(&*scale, &mut ikP.scale);

        //base pose frames for the limb
        ikP.startFrame = (*addr_of!(bgHumanoidAnimations))[baseposeAnim as usize].firstFrame as c_int
            + (*addr_of!(bgHumanoidAnimations))[baseposeAnim as usize].numFrames as c_int;
        ikP.endFrame = (*addr_of!(bgHumanoidAnimations))[baseposeAnim as usize].firstFrame as c_int
            + (*addr_of!(bgHumanoidAnimations))[baseposeAnim as usize].numFrames as c_int;

        ikP.forceAnimOnBone = QFALSE; //let it use existing anim if it's the same as this one.

        //we want to call with a null bone name first. This will init all of the
        //ik system stuff on the g2 instance, because we need ragdoll effectors
        //in order for our pcj's to know how to angle properly.
        if trap::G2API_SetBoneIKState(ghoul2, time, None, IKS_DYNAMIC, Some(&ikP)) == QFALSE {
            debug_assert!(false, "Failed to init IK system for g2 instance!");
        }

        //Now, create our IK bone state.
        if trap::G2API_SetBoneIKState(ghoul2, time, Some("lhumerus"), IKS_DYNAMIC, Some(&ikP))
            != QFALSE
        {
            //restrict the elbow joint
            VectorSet(&mut ikP.pcjMins, -90.0, -20.0, -20.0);
            VectorSet(&mut ikP.pcjMaxs, 30.0, 20.0, -20.0);

            if trap::G2API_SetBoneIKState(ghoul2, time, Some("lradius"), IKS_DYNAMIC, Some(&ikP))
                != QFALSE
            {
                //everything went alright.
                *ikInProgress = QTRUE;
            }
        }
    }

    if *ikInProgress != QFALSE && forceHalt == QFALSE {
        //actively update our ik state.
        let mut lHandMatrix: mdxaBone_t = mdxaBone_t::default();
        let mut ikM: sharedIKMoveParams_t = core::mem::MaybeUninit::zeroed().assume_init();
        let mut tuParms: sharedRagDollUpdateParams_t =
            core::mem::MaybeUninit::zeroed().assume_init();
        let mut tAngles: vec3_t = [0.0; 3];

        //set the argument struct up
        VectorCopy(&*desiredPos, &mut ikM.desiredOrigin); //we want the bone to move here.. if possible

        VectorCopy(&*angles, &mut tAngles);
        tAngles[PITCH] = 0.0;
        tAngles[ROLL] = 0.0;

        trap::G2API_GetBoltMatrix(
            ghoul2,
            0,
            lHandBolt,
            &mut lHandMatrix,
            &tAngles,
            &*origin,
            time,
            core::ptr::null_mut(),
            &*scale,
        );
        //Get the point position from the matrix.
        let mut lHand: vec3_t = [0.0; 3];
        lHand[0] = lHandMatrix.matrix[0][3];
        lHand[1] = lHandMatrix.matrix[1][3];
        lHand[2] = lHandMatrix.matrix[2][3];

        let mut torg: vec3_t = [0.0; 3];
        VectorSubtract(&lHand, &*desiredPos, &mut torg);
        let distToDest = VectorLength(&torg);

        //closer we are, more we want to keep updated.
        //if we're far away we don't want to be too fast or we'll start twitching all over.
        if distToDest < 2.0 {
            //however if we're this close we want very precise movement
            ikM.movementSpeed = 0.4;
        } else if distToDest < 16.0 {
            ikM.movementSpeed = 0.9; //8.0f;
        } else if distToDest < 32.0 {
            ikM.movementSpeed = 0.8; //4.0f;
        } else if distToDest < 64.0 {
            ikM.movementSpeed = 0.7; //2.0f;
        } else {
            ikM.movementSpeed = 0.6;
        }
        VectorCopy(&*origin, &mut ikM.origin); //our position in the world.

        ikM.boneName[0] = 0;
        if trap::G2API_IKMove(ghoul2, time, &ikM) != QFALSE {
            //now do the standard model animate stuff with ragdoll update params.
            VectorCopy(&*angles, &mut tuParms.angles);
            tuParms.angles[PITCH] = 0.0;

            VectorCopy(&*origin, &mut tuParms.position);
            VectorCopy(&*scale, &mut tuParms.scale);

            tuParms.me = (*ent).number;
            VectorClear(&mut tuParms.velocity);

            trap::G2API_AnimateG2Models(ghoul2, time, &tuParms);
        } else {
            *ikInProgress = QFALSE;
        }
    } else if *ikInProgress != QFALSE {
        //kill it
        let mut cFrame: f32 = 0.0;
        let mut animSpeed: f32 = 0.0;
        let mut sFrame: c_int = 0;
        let mut eFrame: c_int = 0;
        let mut flags: c_int = 0;

        trap::G2API_SetBoneIKState(ghoul2, time, Some("lhumerus"), IKS_NONE, None);
        trap::G2API_SetBoneIKState(ghoul2, time, Some("lradius"), IKS_NONE, None);

        //then reset the angles/anims on these PCJs
        trap::G2API_SetBoneAngles(
            ghoul2, 0, "lhumerus", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y,
            NEGATIVE_Z, core::ptr::null_mut(), 0, time,
        );
        trap::G2API_SetBoneAngles(
            ghoul2, 0, "lradius", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y,
            NEGATIVE_Z, core::ptr::null_mut(), 0, time,
        );

        //Get the anim/frames that the pelvis is on exactly, and match the left arm back up with them again.
        trap::G2API_GetBoneAnim(
            ghoul2,
            "pelvis",
            time,
            &mut cFrame,
            &mut sFrame,
            &mut eFrame,
            &mut flags,
            &mut animSpeed,
            core::ptr::null_mut(),
            0,
        );
        trap::G2API_SetBoneAnim(
            ghoul2, 0, "lhumerus", sFrame, eFrame, flags, animSpeed, time, sFrame as f32, 300,
        );
        trap::G2API_SetBoneAnim(
            ghoul2, 0, "lradius", sFrame, eFrame, flags, animSpeed, time, sFrame as f32, 300,
        );

        //And finally, get rid of all the ik state effector data by calling with null bone name (similar to how we init it).
        trap::G2API_SetBoneIKState(ghoul2, time, None, IKS_NONE, None);

        *ikInProgress = QFALSE;
    }
}

/// `BG_UpdateLookAngles` (bg_pmove.c:8493) — while the look-debounce is active, clamp
/// the desired `lookAngles` to the pitch/yaw/roll limits then slowly lerp toward them
/// (0.1 frame-interp × `lookSpeed`) from `lastHeadAngles`; always store the result back
/// into `lastHeadAngles`. The C `static` scratch (`oldLookAngles`/`lookAnglesDiff`/`ang`)
/// is written-before-read every call, so faithful local vars are behaviorally identical.
///
/// # Safety
/// `lastHeadAngles` and `lookAngles` must point to valid `vec3_t`s.
#[allow(clippy::too_many_arguments)]
pub unsafe fn BG_UpdateLookAngles(
    lookingDebounceTime: c_int,
    lastHeadAngles: *mut vec3_t,
    time: c_int,
    lookAngles: *mut vec3_t,
    lookSpeed: f32,
    minPitch: f32,
    maxPitch: f32,
    minYaw: f32,
    maxYaw: f32,
    minRoll: f32,
    maxRoll: f32,
) {
    const F_FRAME_INTER: f32 = 0.1; // static const float fFrameInter
    let mut old_look_angles: vec3_t = [0.0; 3];
    let mut look_angles_diff: vec3_t = [0.0; 3];

    if lookingDebounceTime > time {
        //clamp so don't get "Exorcist" effect
        if (*lookAngles)[PITCH] > maxPitch {
            (*lookAngles)[PITCH] = maxPitch;
        } else if (*lookAngles)[PITCH] < minPitch {
            (*lookAngles)[PITCH] = minPitch;
        }
        if (*lookAngles)[YAW] > maxYaw {
            (*lookAngles)[YAW] = maxYaw;
        } else if (*lookAngles)[YAW] < minYaw {
            (*lookAngles)[YAW] = minYaw;
        }
        if (*lookAngles)[ROLL] > maxRoll {
            (*lookAngles)[ROLL] = maxRoll;
        } else if (*lookAngles)[ROLL] < minRoll {
            (*lookAngles)[ROLL] = minRoll;
        }

        //slowly lerp to this new value
        //Remember last headAngles
        VectorCopy(&*lastHeadAngles, &mut old_look_angles);
        VectorSubtract(&*lookAngles, &old_look_angles, &mut look_angles_diff);

        for ang in 0..3 {
            look_angles_diff[ang] = AngleNormalize180(look_angles_diff[ang]);
        }

        if VectorLengthSquared(&look_angles_diff) != 0.0 {
            (*lookAngles)[PITCH] = AngleNormalize180(
                old_look_angles[PITCH] + (look_angles_diff[PITCH] * F_FRAME_INTER * lookSpeed),
            );
            (*lookAngles)[YAW] = AngleNormalize180(
                old_look_angles[YAW] + (look_angles_diff[YAW] * F_FRAME_INTER * lookSpeed),
            );
            (*lookAngles)[ROLL] = AngleNormalize180(
                old_look_angles[ROLL] + (look_angles_diff[ROLL] * F_FRAME_INTER * lookSpeed),
            );
        }
    }
    //Remember current lookAngles next time
    VectorCopy(&*lookAngles, &mut *lastHeadAngles);
}

/// `BG_G2ClientNeckAngles` (bg_pmove.c:8550) — split the clamped look angles across the
/// `cranium`/`cervical`/`thoracic` bones so the head and neck track `lookAngles` (first
/// clamped between `headClampMin/MaxAngles`), blending the thoracic share with any value
/// the spine pass already wrote. Three `strap_G2API_SetBoneAngles` calls, dissolved to
/// direct `trap::` (GetBoltMatrix precedent). `static void`→`pub`. No oracle (syscall).
///
/// Fidelity: the thoracic and head blend factors (`0.4`/`0.1`/`0.6`) are C `double`
/// literals, so those products promote f32→f64 then round back to f32 on store; the neck
/// share uses `0.2f`/`0.3f` (`float`) literals and stays in f32. Carried per-line so the
/// rounding matches the original bit-for-bit. (`* 0.5f` on an already-`double` operand
/// promotes 0.5f→0.5, exact, so it is written `* 0.5`.)
///
/// # Safety
/// All pointers must reference valid `vec3_t`s; `thoracicAngles` is both read and written.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(clippy::too_many_arguments)]
pub unsafe fn BG_G2ClientNeckAngles(
    ghoul2: *mut c_void,
    time: c_int,
    lookAngles: *const vec3_t,
    headAngles: *mut vec3_t,
    neckAngles: *mut vec3_t,
    thoracicAngles: *mut vec3_t,
    headClampMinAngles: *const vec3_t,
    headClampMaxAngles: *const vec3_t,
) {
    let mut lA: vec3_t = [0.0; 3];
    VectorCopy(&*lookAngles, &mut lA);
    // clamp the headangles (which should now be relative to the cervical (neck) angles
    if lA[PITCH] < (*headClampMinAngles)[PITCH] {
        lA[PITCH] = (*headClampMinAngles)[PITCH];
    } else if lA[PITCH] > (*headClampMaxAngles)[PITCH] {
        lA[PITCH] = (*headClampMaxAngles)[PITCH];
    }

    if lA[YAW] < (*headClampMinAngles)[YAW] {
        lA[YAW] = (*headClampMinAngles)[YAW];
    } else if lA[YAW] > (*headClampMaxAngles)[YAW] {
        lA[YAW] = (*headClampMaxAngles)[YAW];
    }

    if lA[ROLL] < (*headClampMinAngles)[ROLL] {
        lA[ROLL] = (*headClampMinAngles)[ROLL];
    } else if lA[ROLL] > (*headClampMaxAngles)[ROLL] {
        lA[ROLL] = (*headClampMaxAngles)[ROLL];
    }

    // split it up between the neck and cranium
    if (*thoracicAngles)[PITCH] != 0.0 {
        // already been set above, blend them
        (*thoracicAngles)[PITCH] =
            (((*thoracicAngles)[PITCH] as f64 + (lA[PITCH] as f64 * 0.4)) * 0.5) as f32;
    } else {
        (*thoracicAngles)[PITCH] = (lA[PITCH] as f64 * 0.4) as f32;
    }
    if (*thoracicAngles)[YAW] != 0.0 {
        // already been set above, blend them
        (*thoracicAngles)[YAW] =
            (((*thoracicAngles)[YAW] as f64 + (lA[YAW] as f64 * 0.1)) * 0.5) as f32;
    } else {
        (*thoracicAngles)[YAW] = (lA[YAW] as f64 * 0.1) as f32;
    }
    if (*thoracicAngles)[ROLL] != 0.0 {
        // already been set above, blend them
        (*thoracicAngles)[ROLL] =
            (((*thoracicAngles)[ROLL] as f64 + (lA[ROLL] as f64 * 0.1)) * 0.5) as f32;
    } else {
        (*thoracicAngles)[ROLL] = (lA[ROLL] as f64 * 0.1) as f32;
    }

    (*neckAngles)[PITCH] = lA[PITCH] * 0.2f32;
    (*neckAngles)[YAW] = lA[YAW] * 0.3f32;
    (*neckAngles)[ROLL] = lA[ROLL] * 0.3f32;

    (*headAngles)[PITCH] = (lA[PITCH] as f64 * 0.4) as f32;
    (*headAngles)[YAW] = (lA[YAW] as f64 * 0.6) as f32;
    (*headAngles)[ROLL] = (lA[ROLL] as f64 * 0.6) as f32;

    /* //non-applicable SP code
    if ( G_RidingVehicle( cent->gent ) )// && type == VH_SPEEDER ?
    {//aim torso forward too
        headAngles[YAW] = neckAngles[YAW] = thoracicAngles[YAW] = 0;
    }
    */

    trap::G2API_SetBoneAngles(
        ghoul2, 0, "cranium", &*headAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y,
        NEGATIVE_Z, core::ptr::null_mut(), 0, time,
    );
    trap::G2API_SetBoneAngles(
        ghoul2, 0, "cervical", &*neckAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y,
        NEGATIVE_Z, core::ptr::null_mut(), 0, time,
    );
    trap::G2API_SetBoneAngles(
        ghoul2, 0, "thoracic", &*thoracicAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y,
        NEGATIVE_Z, core::ptr::null_mut(), 0, time,
    );
}

/// `BG_G2ClientSpineAngles` (bg_pmove.c:8629) — turn the torso toward `viewAngles` by
/// distributing the residual yaw/pitch/roll up the spine (`thoracic`/`upper`/`lower lumbar`
/// shares that sum to 1.0). When legs and torso are on different anims (and the player is
/// not flipping / spinning / rolling / knocked-down / dead / in a vehicle), it first
/// subtracts the motion-bolt's own rotation (read via `GetBoltMatrix_NoRecNoRot`) so the
/// correction is relative to the animating pelvis. `static void`→`pub`. No oracle (syscall).
///
/// The original wraps the gate in `#if 1` / `#else` / `#endif`; the `#else` branch (a
/// `corrTime` smoothing variant) is compiled out, so it is dropped here and the
/// `tPitchAngle`/`tYawAngle`/`corrTime` out-params it would use are unused in this active
/// path (kept `_`-prefixed for signature parity, as the two `*tPitchAngle`/`*tYawAngle`
/// writes are also commented out in-source). All `*0.20f`..`*0.45f` shares are `float`
/// literals — no f64 promotion.
///
/// # Safety
/// All pointers must reference valid objects of their types; `viewAngles`/`thoracicAngles`/
/// `ulAngles`/`llAngles` are written.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(clippy::too_many_arguments)]
pub unsafe fn BG_G2ClientSpineAngles(
    ghoul2: *mut c_void,
    motionBolt: c_int,
    cent_lerpOrigin: *mut vec3_t,
    cent_lerpAngles: *mut vec3_t,
    cent: *mut entityState_t,
    time: c_int,
    viewAngles: *mut vec3_t,
    ciLegs: c_int,
    ciTorso: c_int,
    angles: *const vec3_t,
    thoracicAngles: *mut vec3_t,
    ulAngles: *mut vec3_t,
    llAngles: *mut vec3_t,
    modelScale: *mut vec3_t,
    _tPitchAngle: *mut f32,
    _tYawAngle: *mut f32,
    _corrTime: *mut c_int,
) {
    let mut doCorr: qboolean = QFALSE;

    //*tPitchAngle = viewAngles[PITCH];
    (*viewAngles)[YAW] = AngleDelta((*cent_lerpAngles)[YAW], (*angles)[YAW]);
    //*tYawAngle = viewAngles[YAW];

    // #if 1 (the #else corrTime-smoothing branch is compiled out in-source)
    if BG_FlippingAnim((*cent).legsAnim) == QFALSE
        && BG_SpinningSaberAnim((*cent).legsAnim) == QFALSE
        && BG_SpinningSaberAnim((*cent).torsoAnim) == QFALSE
        && BG_InSpecialJump((*cent).legsAnim) == QFALSE
        && BG_InSpecialJump((*cent).torsoAnim) == QFALSE
        && BG_InDeathAnim((*cent).legsAnim) == QFALSE
        && BG_InDeathAnim((*cent).torsoAnim) == QFALSE
        && BG_InRollES(cent, (*cent).legsAnim) == QFALSE
        && BG_InRollAnim(cent) == QFALSE
        && BG_SaberInSpecial((*cent).saberMove) == QFALSE
        && BG_SaberInSpecialAttack((*cent).torsoAnim) == QFALSE
        && BG_SaberInSpecialAttack((*cent).legsAnim) == QFALSE
        && BG_InKnockDown((*cent).torsoAnim) == QFALSE
        && BG_InKnockDown((*cent).legsAnim) == QFALSE
        && BG_InKnockDown(ciTorso) == QFALSE
        && BG_InKnockDown(ciLegs) == QFALSE
        && BG_FlippingAnim(ciLegs) == QFALSE
        && BG_SpinningSaberAnim(ciLegs) == QFALSE
        && BG_SpinningSaberAnim(ciTorso) == QFALSE
        && BG_InSpecialJump(ciLegs) == QFALSE
        && BG_InSpecialJump(ciTorso) == QFALSE
        && BG_InDeathAnim(ciLegs) == QFALSE
        && BG_InDeathAnim(ciTorso) == QFALSE
        && BG_SaberInSpecialAttack(ciTorso) == QFALSE
        && BG_SaberInSpecialAttack(ciLegs) == QFALSE
        && ((*cent).eFlags & EF_DEAD) == 0
        && (*cent).legsAnim != (*cent).torsoAnim
        && ciLegs != ciTorso
        && (*cent).m_iVehicleNum == 0
    {
        doCorr = QTRUE;
    }

    if doCorr != QFALSE {
        // FIXME: no need to do this if legs and torso on are same frame
        // adjust for motion offset
        let mut boltMatrix: mdxaBone_t = mdxaBone_t::default();
        let mut motionFwd: vec3_t = [0.0; 3];
        let mut motionAngles: vec3_t = [0.0; 3];
        let mut motionRt: vec3_t = [0.0; 3];
        let mut tempAng: vec3_t = [0.0; 3];

        trap::G2API_GetBoltMatrix_NoRecNoRot(
            ghoul2,
            0,
            motionBolt,
            &mut boltMatrix,
            &vec3_origin,
            &*cent_lerpOrigin,
            time,
            core::ptr::null_mut(),
            &*modelScale,
        );
        //BG_GiveMeVectorFromMatrix( &boltMatrix, NEGATIVE_Y, motionFwd );
        motionFwd[0] = -boltMatrix.matrix[0][1];
        motionFwd[1] = -boltMatrix.matrix[1][1];
        motionFwd[2] = -boltMatrix.matrix[2][1];

        vectoangles(&motionFwd, &mut motionAngles);

        //BG_GiveMeVectorFromMatrix( &boltMatrix, NEGATIVE_X, motionRt );
        motionRt[0] = -boltMatrix.matrix[0][0];
        motionRt[1] = -boltMatrix.matrix[1][0];
        motionRt[2] = -boltMatrix.matrix[2][0];

        vectoangles(&motionRt, &mut tempAng);
        motionAngles[ROLL] = -tempAng[PITCH];

        for ang in 0..3 {
            (*viewAngles)[ang] =
                AngleNormalize180((*viewAngles)[ang] - AngleNormalize180(motionAngles[ang]));
        }
    }

    // distribute the angles differently up the spine
    // NOTE: each of these distributions must add up to 1.0f
    (*thoracicAngles)[PITCH] = (*viewAngles)[PITCH] * 0.20f32;
    (*llAngles)[PITCH] = (*viewAngles)[PITCH] * 0.40f32;
    (*ulAngles)[PITCH] = (*viewAngles)[PITCH] * 0.40f32;

    (*thoracicAngles)[YAW] = (*viewAngles)[YAW] * 0.20f32;
    (*ulAngles)[YAW] = (*viewAngles)[YAW] * 0.35f32;
    (*llAngles)[YAW] = (*viewAngles)[YAW] * 0.45f32;

    (*thoracicAngles)[ROLL] = (*viewAngles)[ROLL] * 0.20f32;
    (*ulAngles)[ROLL] = (*viewAngles)[ROLL] * 0.35f32;
    (*llAngles)[ROLL] = (*viewAngles)[ROLL] * 0.45f32;
}

/// `BG_SwingAngles` (bg_pmove.c:8757) — ease a single body angle toward `destination`:
/// start swinging once outside `swingTolerance`, move at a delta-scaled speed, stop when
/// the destination is reached, and clamp to within `clampTolerance`. Returns the residual
/// swing. `static float`→`pub`. (`fabs` is exact sign-bit clear; the `*0.5` threshold is
/// exact in `f32`, so no `f64` promotion is needed.)
///
/// # Safety
/// `angle` and `swinging` must point to valid `f32`/`qboolean`.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe fn BG_SwingAngles(
    destination: f32,
    swingTolerance: f32,
    clampTolerance: f32,
    speed: f32,
    angle: *mut f32,
    swinging: *mut qboolean,
    frametime: c_int,
) -> f32 {
    let mut swing: f32;
    let mut r#move: f32;
    let mut scale: f32;

    if *swinging == QFALSE {
        // see if a swing should be started
        swing = AngleSubtract(*angle, destination);
        if swing > swingTolerance || swing < -swingTolerance {
            *swinging = QTRUE;
        }
    }

    if *swinging == QFALSE {
        return 0.0;
    }

    // modify the speed depending on the delta
    // so it doesn't seem so linear
    swing = AngleSubtract(destination, *angle);
    scale = swing.abs(); // fabs(swing)
    if scale < swingTolerance * 0.5 {
        scale = 0.5;
    } else if scale < swingTolerance {
        scale = 1.0;
    } else {
        scale = 2.0;
    }

    // swing towards the destination angle
    if swing >= 0.0 {
        r#move = frametime as f32 * scale * speed;
        if r#move >= swing {
            r#move = swing;
            *swinging = QFALSE;
        }
        *angle = AngleMod(*angle + r#move);
    } else if swing < 0.0 {
        r#move = frametime as f32 * scale * -speed;
        if r#move <= swing {
            r#move = swing;
            *swinging = QFALSE;
        }
        *angle = AngleMod(*angle + r#move);
    }

    // clamp to no more than tolerance
    swing = AngleSubtract(destination, *angle);
    if swing > clampTolerance {
        *angle = AngleMod(destination - (clampTolerance - 1.0));
    } else if swing < -clampTolerance {
        *angle = AngleMod(destination + (clampTolerance - 1.0));
    }

    swing
}

/// `BG_InRoll2` (bg_pmove.c:8818) — is this entity's `legsAnim` a roll *or* a get-up-roll
/// animation? (Superset of `BG_InRollAnim`: also the `BOTH_GETUP_[BF]ROLL_*` set.) Pure
/// classifier `switch`→`match` over `es->legsAnim`.
///
/// # Safety
/// `es` must point to a valid `entityState_t`.
pub unsafe fn BG_InRoll2(es: *mut entityState_t) -> qboolean {
    match (*es).legsAnim {
        BOTH_GETUP_BROLL_B
        | BOTH_GETUP_BROLL_F
        | BOTH_GETUP_BROLL_L
        | BOTH_GETUP_BROLL_R
        | BOTH_GETUP_FROLL_B
        | BOTH_GETUP_FROLL_F
        | BOTH_GETUP_FROLL_L
        | BOTH_GETUP_FROLL_R
        | BOTH_ROLL_F
        | BOTH_ROLL_B
        | BOTH_ROLL_R
        | BOTH_ROLL_L => QTRUE,
        _ => QFALSE,
    }
}

/// `BG_G2PlayerAngles` (bg_pmove.c:8842) — the player/NPC torso-twist solver. Builds the
/// leg axis (`legs`) and feeds the spine/neck/head bone overrides so the upper body tracks
/// `cent_lerpAngles`/`lookAngles` while the legs swing to follow: drift the yaw, fold a
/// fraction of the pitch into the torso (swung via `BG_SwingAngles`), bank a roll from
/// lateral velocity, derive a velocity-based leg yaw, pull the angles back out of the
/// hierarchical chain, then push them into ghoul2 via `BG_G2ClientSpineAngles` /
/// `BG_G2ClientNeckAngles` + `strap_G2API_SetBoneAngles`. Special-cases vehicles /
/// saber-lock (early bone-clear + return) and emplaced guns. `strap_*` dissolved to direct
/// `trap::`. No oracle (engine syscalls).
///
/// Fidelity notes: the original's `static` locals are all written-before-read each call, so
/// they are carried as plain locals (the `BG_UpdateLookAngles` precedent). `movementOffsets`
/// has only a commented-out consumer (`_`-prefixed, kept for fidelity). `BONE_BASED_LEG_ANGLES`
/// is undefined upstream, so its three blocks (incl. `legBoneYaw`) are dropped. Blend factors
/// that are C `double` literals (`*0.75`, `*0.05`, `*0.5`) promote f32→f64 then round to f32;
/// the leading `-360 + pitch` / `headAngles[PITCH]` operands stay f32 (matching C's int→float
/// then float→double order). Self-aliased `vectoangles`/`AnglesSubtract`/`VectorScale`/
/// `VectorAdd` calls (out==in1) use a temp copy — bit-identical for these element-wise ops.
///
/// # Safety
/// All pointers must reference valid objects; `emplaced` and `crazySmoothFactor` may be null.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(clippy::too_many_arguments)]
pub unsafe fn BG_G2PlayerAngles(
    ghoul2: *mut c_void,
    motionBolt: c_int,
    cent: *mut entityState_t,
    time: c_int,
    cent_lerpOrigin: *mut vec3_t,
    cent_lerpAngles: *mut vec3_t,
    legs: *mut [vec3_t; 3],
    legsAngles: *mut vec3_t,
    tYawing: *mut qboolean,
    tPitching: *mut qboolean,
    lYawing: *mut qboolean,
    tYawAngle: *mut f32,
    tPitchAngle: *mut f32,
    lYawAngle: *mut f32,
    frametime: c_int,
    turAngles: *mut vec3_t,
    modelScale: *mut vec3_t,
    ciLegs: c_int,
    ciTorso: c_int,
    corrTime: *mut c_int,
    lookAngles: *mut vec3_t,
    lastHeadAngles: *mut vec3_t,
    lookTime: c_int,
    emplaced: *mut entityState_t,
    crazySmoothFactor: *mut c_int,
) {
    // adddir / dir / degrees_* are written-once on each branch (C inits them to 0, but that
    // value is always overwritten before any read), so they are deferred-init `let`s here.
    let adddir: c_int;
    let dir: c_int;
    // movementOffsets' only reference is a commented-out leg-angle line; kept for fidelity.
    let _movementOffsets: [c_int; 8] = [0, 22, 45, -22, 0, 22, -45, -22];
    let degrees_negative: f32;
    let degrees_positive: f32;
    let mut dif: f32;
    let dest: f32;
    let mut speed: f32;
    let lookSpeed: f32 = 1.5;
    let mut eyeAngles: vec3_t = [0.0; 3];
    let mut neckAngles: vec3_t = [0.0; 3];
    let mut velocity: vec3_t = [0.0; 3];
    let mut torsoAngles: vec3_t = [0.0; 3];
    let mut headAngles: vec3_t = [0.0; 3];
    let mut velPos: vec3_t = [0.0; 3];
    let mut velAng: vec3_t = [0.0; 3];
    let mut ulAngles: vec3_t = [0.0; 3];
    let mut llAngles: vec3_t = [0.0; 3];
    let mut viewAngles: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut thoracicAngles: vec3_t = [0.0, 0.0, 0.0];
    let headClampMinAngles: vec3_t = [-25.0, -55.0, -10.0];
    let headClampMaxAngles: vec3_t = [50.0, 50.0, 10.0];

    if (*cent).m_iVehicleNum != 0
        || (*cent).forceFrame != 0
        || BG_SaberLockBreakAnim((*cent).legsAnim) != QFALSE
        || BG_SaberLockBreakAnim((*cent).torsoAnim) != QFALSE
    {
        // a vehicle or riding a vehicle - in either case we don't need to be in here
        let mut forcedAngles: vec3_t = [0.0; 3];

        VectorClear(&mut forcedAngles);
        forcedAngles[YAW] = (*cent_lerpAngles)[YAW];
        forcedAngles[ROLL] = (*cent_lerpAngles)[ROLL];
        AnglesToAxis(&forcedAngles, &mut *legs);
        VectorCopy(&forcedAngles, &mut *legsAngles);

        if (*cent).number < MAX_CLIENTS as c_int {
            trap::G2API_SetBoneAngles(ghoul2, 0, "lower_lumbar", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
            trap::G2API_SetBoneAngles(ghoul2, 0, "upper_lumbar", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
            trap::G2API_SetBoneAngles(ghoul2, 0, "cranium", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
            trap::G2API_SetBoneAngles(ghoul2, 0, "thoracic", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
            trap::G2API_SetBoneAngles(ghoul2, 0, "cervical", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
        }
        return;
    }

    if (time + 2000) < *corrTime {
        *corrTime = 0;
    }

    VectorCopy(&*cent_lerpAngles, &mut headAngles);
    headAngles[YAW] = AngleMod(headAngles[YAW]);
    VectorClear(&mut *legsAngles);
    VectorClear(&mut torsoAngles);
    // --------- yaw -------------

    // allow yaw to drift a bit
    if (*cent).legsAnim != BOTH_STAND1
        || (*cent).torsoAnim != WeaponReadyAnim[(*cent).weapon as usize]
    {
        // if not standing still, always point all in the same direction
        *tYawing = QTRUE;
        *tPitching = QTRUE;
        *lYawing = QTRUE;
    }

    // adjust legs for movement dir
    if (*cent).eFlags & EF_DEAD != 0 {
        // don't let dead bodies twitch
        dir = 0;
    } else {
        dir = (*cent).angles2[YAW] as c_int;
        if dir < 0 || dir > 7 {
            Com_Error(ERR_DROP, &format!("Bad player movement angle ({})", dir));
        }
    }

    torsoAngles[YAW] = headAngles[YAW];

    // for now, turn torso instantly and let the legs swing to follow
    *tYawAngle = torsoAngles[YAW];

    // --------- pitch -------------

    VectorCopy(&(*cent).pos.trDelta, &mut velocity);

    if BG_InRoll2(cent) != QFALSE {
        // don't affect angles based on vel then
        VectorClear(&mut velocity);
    } else if (*cent).weapon == WP_SABER && BG_SaberInSpecial((*cent).saberMove) != QFALSE {
        VectorClear(&mut velocity);
    }

    speed = VectorNormalize(&mut velocity);

    if speed == 0.0 {
        torsoAngles[YAW] = headAngles[YAW];
    }

    // only show a fraction of the pitch angle in the torso
    if headAngles[PITCH] > 180.0 {
        dest = ((-360.0f32 + headAngles[PITCH]) as f64 * 0.75) as f32;
    } else {
        dest = (headAngles[PITCH] as f64 * 0.75) as f32;
    }

    if (*cent).m_iVehicleNum != 0 {
        // swing instantly on vehicles
        *tPitchAngle = dest;
    } else {
        BG_SwingAngles(dest, 15.0, 30.0, 0.1, tPitchAngle, tPitching, frametime);
    }
    torsoAngles[PITCH] = *tPitchAngle;

    // --------- roll -------------

    if speed != 0.0 {
        let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
        let mut side: f32;

        speed = (speed as f64 * 0.05) as f32;

        AnglesToAxis(&*legsAngles, &mut axis);
        side = speed * DotProduct(&velocity, &axis[1]);
        (*legsAngles)[ROLL] -= side;

        side = speed * DotProduct(&velocity, &axis[0]);
        (*legsAngles)[PITCH] += side;
    }

    //legsAngles[YAW] = headAngles[YAW] + (movementOffsets[ dir ]*speed_dif);

    // rww - crazy velocity-based leg angle calculation
    (*legsAngles)[YAW] = headAngles[YAW];
    velPos[0] = (*cent_lerpOrigin)[0] + velocity[0];
    velPos[1] = (*cent_lerpOrigin)[1] + velocity[1];
    velPos[2] = (*cent_lerpOrigin)[2]; // + velocity[2];

    if (*cent).groundEntityNum == ENTITYNUM_NONE
        || (*cent).forceFrame != 0
        || ((*cent).weapon == WP_EMPLACED_GUN && !emplaced.is_null())
    {
        // off the ground, no direction-based leg angles (same if in saberlock)
        VectorCopy(&*cent_lerpOrigin, &mut velPos);
    }

    VectorSubtract(&*cent_lerpOrigin, &velPos, &mut velAng);

    if VectorCompare(&velAng, &vec3_origin) == 0 {
        {
            let tmp = velAng;
            vectoangles(&tmp, &mut velAng);
        }

        if velAng[YAW] <= (*legsAngles)[YAW] {
            degrees_negative = (*legsAngles)[YAW] - velAng[YAW];
            degrees_positive = (360.0 - (*legsAngles)[YAW]) + velAng[YAW];
        } else {
            degrees_negative = (*legsAngles)[YAW] + (360.0 - velAng[YAW]);
            degrees_positive = velAng[YAW] - (*legsAngles)[YAW];
        }

        if degrees_negative < degrees_positive {
            dif = degrees_negative;
            adddir = 0;
        } else {
            dif = degrees_positive;
            adddir = 1;
        }

        if dif > 90.0 {
            dif = 180.0 - dif;
        }

        if dif > 60.0 {
            dif = 60.0;
        }

        // Slight hack for when playing is running backward
        if dir == 3 || dir == 5 {
            dif = -dif;
        }

        if adddir != 0 {
            (*legsAngles)[YAW] -= dif;
        } else {
            (*legsAngles)[YAW] += dif;
        }
    }

    if (*cent).m_iVehicleNum != 0 {
        // swing instantly on vehicles
        *lYawAngle = (*legsAngles)[YAW];
    } else {
        BG_SwingAngles((*legsAngles)[YAW], /*40*/ 0.0, 90.0, 0.65, lYawAngle, lYawing, frametime);
    }
    (*legsAngles)[YAW] = *lYawAngle;

    /*
    // pain twitch
    CG_AddPainTwitch( cent, torsoAngles );
    */

    (*legsAngles)[ROLL] = 0.0;
    torsoAngles[ROLL] = 0.0;

    //	VectorCopy(legsAngles, turAngles);

    // pull the angles back out of the hierarchial chain
    {
        let tmp = headAngles;
        AnglesSubtract(&tmp, &torsoAngles, &mut headAngles);
    }
    {
        let tmp = torsoAngles;
        AnglesSubtract(&tmp, &*legsAngles, &mut torsoAngles);
    }

    (*legsAngles)[PITCH] = 0.0;

    if (*cent).heldByClient != 0 {
        // keep the base angles clear when doing the IK stuff, it doesn't compensate for it.
        // rwwFIXMEFIXME: Store leg angles off and add them to all the fed in angles for G2 functions?
        VectorClear(&mut *legsAngles);
        (*legsAngles)[YAW] = (*cent_lerpAngles)[YAW];
    }

    // BONE_BASED_LEG_ANGLES is undefined upstream — the legBoneYaw block is dropped.

    VectorCopy(&*legsAngles, &mut *turAngles);

    AnglesToAxis(&*legsAngles, &mut *legs);

    VectorCopy(&*cent_lerpAngles, &mut viewAngles);
    viewAngles[YAW] = 0.0;
    viewAngles[ROLL] = 0.0;
    viewAngles[PITCH] = (viewAngles[PITCH] as f64 * 0.5) as f32;

    VectorSet(&mut angles, 0.0, (*legsAngles)[1], 0.0);

    angles[0] = (*legsAngles)[0];
    if angles[0] > 30.0 {
        angles[0] = 30.0;
    } else if angles[0] < -30.0 {
        angles[0] = -30.0;
    }

    if (*cent).weapon == WP_EMPLACED_GUN && !emplaced.is_null() {
        // if using an emplaced gun, then we want to make sure we're angled to "hold" it right
        let mut facingAngles: vec3_t = [0.0; 3];

        VectorSubtract(&(*emplaced).pos.trBase, &*cent_lerpOrigin, &mut facingAngles);
        {
            let tmp = facingAngles;
            vectoangles(&tmp, &mut facingAngles);
        }

        if (*emplaced).weapon == WP_NONE {
            // e-web
            VectorCopy(&facingAngles, &mut *legsAngles);
            AnglesToAxis(&*legsAngles, &mut *legs);
        } else {
            // misc emplaced
            let dif = AngleSubtract((*cent_lerpAngles)[YAW], facingAngles[YAW]);

            /*
            if (emplaced->weapon == WP_NONE)
            { //offset is a little bit different for the e-web
                dif -= 16.0f;
            }
            */

            VectorSet(&mut facingAngles, -16.0, -dif, 0.0);

            if (*cent).legsAnim == BOTH_STRAFE_LEFT1 || (*cent).legsAnim == BOTH_STRAFE_RIGHT1 {
                // try to adjust so it doesn't look wrong
                if !crazySmoothFactor.is_null() {
                    // want to smooth a lot during this because it chops around and looks like ass
                    *crazySmoothFactor = time + 1000;
                }

                BG_G2ClientSpineAngles(
                    ghoul2, motionBolt, cent_lerpOrigin, cent_lerpAngles, cent, time,
                    &mut viewAngles, ciLegs, ciTorso, &angles, &mut thoracicAngles,
                    &mut ulAngles, &mut llAngles, modelScale, tPitchAngle, tYawAngle, corrTime,
                );
                trap::G2API_SetBoneAngles(ghoul2, 0, "lower_lumbar", &llAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
                trap::G2API_SetBoneAngles(ghoul2, 0, "upper_lumbar", &ulAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
                trap::G2API_SetBoneAngles(ghoul2, 0, "cranium", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);

                {
                    let tmp = facingAngles;
                    VectorAdd(&tmp, &thoracicAngles, &mut facingAngles);
                }

                if (*cent).legsAnim == BOTH_STRAFE_LEFT1 {
                    // this one needs some further correction
                    facingAngles[YAW] -= 32.0;
                }
            } else {
                //strap_G2API_SetBoneAngles(ghoul2, 0, "lower_lumbar", vec3_origin, ...);
                //strap_G2API_SetBoneAngles(ghoul2, 0, "upper_lumbar", vec3_origin, ...);
                trap::G2API_SetBoneAngles(ghoul2, 0, "cranium", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
            }

            {
                let tmp = facingAngles;
                VectorScale(&tmp, 0.6, &mut facingAngles);
            }
            trap::G2API_SetBoneAngles(ghoul2, 0, "lower_lumbar", &vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
            {
                let tmp = facingAngles;
                VectorScale(&tmp, 0.8, &mut facingAngles);
            }
            trap::G2API_SetBoneAngles(ghoul2, 0, "upper_lumbar", &facingAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
            {
                let tmp = facingAngles;
                VectorScale(&tmp, 0.8, &mut facingAngles);
            }
            trap::G2API_SetBoneAngles(ghoul2, 0, "thoracic", &facingAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);

            // Now we want the head angled toward where we are facing
            VectorSet(&mut facingAngles, 0.0, dif, 0.0);
            {
                let tmp = facingAngles;
                VectorScale(&tmp, 0.6, &mut facingAngles);
            }
            trap::G2API_SetBoneAngles(ghoul2, 0, "cervical", &facingAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);

            return; // don't have to bother with the rest then
        }
    }

    BG_G2ClientSpineAngles(
        ghoul2, motionBolt, cent_lerpOrigin, cent_lerpAngles, cent, time, &mut viewAngles,
        ciLegs, ciTorso, &angles, &mut thoracicAngles, &mut ulAngles, &mut llAngles, modelScale,
        tPitchAngle, tYawAngle, corrTime,
    );

    VectorCopy(&*cent_lerpAngles, &mut eyeAngles);

    for i in 0..3 {
        (*lookAngles)[i] = AngleNormalize180((*lookAngles)[i]);
        eyeAngles[i] = AngleNormalize180(eyeAngles[i]);
    }
    {
        let tmp = *lookAngles;
        AnglesSubtract(&tmp, &eyeAngles, &mut *lookAngles);
    }

    BG_UpdateLookAngles(
        lookTime, lastHeadAngles, time, lookAngles, lookSpeed, -50.0, 50.0, -70.0, 70.0, -30.0,
        30.0,
    );

    BG_G2ClientNeckAngles(
        ghoul2, time, lookAngles, &mut headAngles, &mut neckAngles, &mut thoracicAngles,
        &headClampMinAngles, &headClampMaxAngles,
    );

    // BONE_BASED_LEG_ANGLES is undefined upstream — the bLAngles model_root block is dropped.
    trap::G2API_SetBoneAngles(ghoul2, 0, "lower_lumbar", &llAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
    trap::G2API_SetBoneAngles(ghoul2, 0, "upper_lumbar", &ulAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
    trap::G2API_SetBoneAngles(ghoul2, 0, "thoracic", &thoracicAngles, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, core::ptr::null_mut(), 0, time);
    //strap_G2API_SetBoneAngles(ghoul2, 0, "cervical", vec3_origin, ...);
}

/// `BG_G2ATSTAngles` (bg_pmove.c:9219) — override the ATST walker's `thoracic` bone so its
/// upper body tracks `cent_lerpAngles`. A single `strap_G2API_SetBoneAngles` call; the
/// `strap_*` indirection is dissolved — call `trap::` directly (the GetBoltMatrix
/// precedent). No oracle (engine syscall, observably inert in the test harness).
///
/// # Safety
/// `ghoul2` is an opaque engine handle passed straight to the syscall; `cent_lerpAngles`
/// must point to a valid `vec3_t`.
pub unsafe fn BG_G2ATSTAngles(ghoul2: *mut c_void, time: c_int, cent_lerpAngles: *mut vec3_t) {
    //                                                                 up          right       fwd
    trap::G2API_SetBoneAngles(
        ghoul2,
        0,
        "thoracic",
        &*cent_lerpAngles,
        BONE_ANGLES_POSTMULT,
        POSITIVE_X,
        NEGATIVE_Y,
        NEGATIVE_Z,
        core::ptr::null_mut(),
        0,
        time,
    );
}

/// `PM_AdjustAnglesForDualJumpAttack` (bg_pmove.c:9224) — hook to re-aim the dual-jump
/// attack; the body is commented out in the original, so it unconditionally returns
/// `qtrue`. `static qboolean`→`pub`, no oracle (trivial).
///
/// # Safety
/// `ps`/`ucmd` are unused but kept for signature parity; any value is accepted.
pub unsafe fn PM_AdjustAnglesForDualJumpAttack(
    _ps: *mut playerState_t,
    _ucmd: *mut usercmd_t,
) -> qboolean {
    //ucmd->angles[PITCH] = ANGLE2SHORT( ps->viewangles[PITCH] ) - ps->delta_angles[PITCH];
    //ucmd->angles[YAW] = ANGLE2SHORT( ps->viewangles[YAW] ) - ps->delta_angles[YAW];
    QTRUE
}

/// `PM_CmdForSaberMoves` (bg_pmove.c:9232) — override the move command to drive the scripted
/// jumping saber attacks: the dual/staff forward-jump-attacks (`BOTH_JUMPATTACK6` /
/// `BOTH_BUTTERFLY_*`) push forward (or strafe) and launch one or two mid-animation jumps,
/// the staff back-flip attack (`BOTH_JUMPATTACK7`) hurls the player backwards, and the
/// dual/staff spin attacks lock the view. `static`→`pub`, no oracle (playerstate / command /
/// event-callback driven; `PM_AnimLength`/`PM_AdjustAnglesForDualJumpAttack`/`PM_SetPMViewAngle`
/// are ported).
///
/// # Safety
/// `pm` must be installed for the frame with a live `ps`; `ucmd` must be valid (the caller
/// passes `&pm->cmd`).
pub unsafe fn PM_CmdForSaberMoves(ucmd: *mut usercmd_t) {
    let pmv = *addr_of!(pm);

    //DUAL FORWARD+JUMP+ATTACK
    if ((*(*pmv).ps).legsAnim == BOTH_JUMPATTACK6
        && (*(*pmv).ps).saberMove == LS_JUMPATTACK_DUAL)
        || ((*(*pmv).ps).legsAnim == BOTH_BUTTERFLY_FL1
            && (*(*pmv).ps).saberMove == LS_JUMPATTACK_STAFF_LEFT)
        || ((*(*pmv).ps).legsAnim == BOTH_BUTTERFLY_FR1
            && (*(*pmv).ps).saberMove == LS_JUMPATTACK_STAFF_RIGHT)
        || ((*(*pmv).ps).legsAnim == BOTH_BUTTERFLY_RIGHT
            && (*(*pmv).ps).saberMove == LS_BUTTERFLY_RIGHT)
        || ((*(*pmv).ps).legsAnim == BOTH_BUTTERFLY_LEFT
            && (*(*pmv).ps).saberMove == LS_BUTTERFLY_LEFT)
    {
        let aLen: c_int = PM_AnimLength(0, BOTH_JUMPATTACK6);

        (*ucmd).upmove = 0;
        (*ucmd).rightmove = 0;
        (*ucmd).forwardmove = 0;

        if (*(*pmv).ps).legsAnim == BOTH_JUMPATTACK6 {
            //dual stance attack
            if (*(*pmv).ps).legsTimer >= 100 //not at end
                && (aLen - (*(*pmv).ps).legsTimer) >= 250
            //not in beginning
            {
                //middle of anim
                //push forward
                (*ucmd).forwardmove = 127;
            }

            if ((*(*pmv).ps).legsTimer >= 900 //not at end
                && aLen - (*(*pmv).ps).legsTimer >= 950) //not in beginning
                || ((*(*pmv).ps).legsTimer >= 1600
                    && aLen - (*(*pmv).ps).legsTimer >= 400)
            //not in beginning
            {
                //one of the two jumps
                if (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE {
                    //still on ground?
                    if (*(*pmv).ps).groundEntityNum >= MAX_CLIENTS as c_int {
                        //jump!
                        (*(*pmv).ps).velocity[2] = 250.0; //400;
                        (*(*pmv).ps).fd.forceJumpZStart = (*(*pmv).ps).origin[2]; //so we don't take damage if we land at same height
                                                                                  //pm->ps->pm_flags |= PMF_JUMPING;
                                                                                  //FIXME: NPCs yell?
                        PM_AddEvent(EV_JUMP);
                        //G_SoundOnEnt( ent, CHAN_BODY, "sound/weapons/force/jump.wav" );
                    }
                } else {
                    //FIXME: if this is the second jump, maybe we should just stop the anim?
                }
            }
        } else {
            //saberstaff attacks
            let aLen: c_int = PM_AnimLength(0, (*(*pmv).ps).legsAnim);
            let mut lenMin: f32 = 1700.0;
            let mut lenMax: f32 = 1800.0;

            if (*(*pmv).ps).legsAnim == BOTH_BUTTERFLY_LEFT {
                lenMin = 1200.0;
                lenMax = 1400.0;
            }

            //FIXME: don't slide off people/obstacles?
            if (*(*pmv).ps).legsAnim == BOTH_BUTTERFLY_RIGHT
                || (*(*pmv).ps).legsAnim == BOTH_BUTTERFLY_LEFT
            {
                if (*(*pmv).ps).legsTimer > 450 {
                    match (*(*pmv).ps).legsAnim {
                        BOTH_BUTTERFLY_LEFT => {
                            (*ucmd).rightmove = -127;
                        }
                        BOTH_BUTTERFLY_RIGHT => {
                            (*ucmd).rightmove = 127;
                        }
                        _ => {}
                    }
                }
            } else if (*(*pmv).ps).legsTimer >= 100 //not at end
                && aLen - (*(*pmv).ps).legsTimer >= 250
            //not in beginning
            {
                //middle of anim
                //push forward
                (*ucmd).forwardmove = 127;
            }

            if (*(*pmv).ps).legsTimer >= lenMin as c_int && (*(*pmv).ps).legsTimer < lenMax as c_int
            {
                //one of the two jumps
                if (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE {
                    //still on ground?
                    //jump!
                    if (*(*pmv).ps).legsAnim == BOTH_BUTTERFLY_LEFT {
                        (*(*pmv).ps).velocity[2] = 350.0;
                    } else {
                        (*(*pmv).ps).velocity[2] = 250.0;
                    }
                    (*(*pmv).ps).fd.forceJumpZStart = (*(*pmv).ps).origin[2]; //so we don't take damage if we land at same height
                                                                              //pm->ps->pm_flags |= PMF_JUMPING;//|PMF_SLOW_MO_FALL;
                                                                              //FIXME: NPCs yell?
                    PM_AddEvent(EV_JUMP);
                    //G_SoundOnEnt( ent, CHAN_BODY, "sound/weapons/force/jump.wav" );
                } else {
                    //FIXME: if this is the second jump, maybe we should just stop the anim?
                }
            }
        }

        if (*(*pmv).ps).groundEntityNum == ENTITYNUM_NONE {
            //can only turn when your feet hit the ground
            if PM_AdjustAnglesForDualJumpAttack((*pmv).ps, ucmd) != QFALSE {
                PM_SetPMViewAngle((*pmv).ps, (*(*pmv).ps).viewangles.as_mut_ptr(), ucmd);
            }
        }
        //rwwFIXMEFIXME: Bother with bbox resizing like sp?
    }
    //STAFF BACK+JUMP+ATTACK
    else if (*(*pmv).ps).saberMove == LS_A_BACKFLIP_ATK
        && (*(*pmv).ps).legsAnim == BOTH_JUMPATTACK7
    {
        let aLen: c_int = PM_AnimLength(0, BOTH_JUMPATTACK7);

        if (*(*pmv).ps).legsTimer > 800 //not at end
            && aLen - (*(*pmv).ps).legsTimer >= 400
        //not in beginning
        {
            //middle of anim
            if (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE {
                //still on ground?
                let mut yawAngles: vec3_t = [0.0; 3];
                let mut backDir: vec3_t = [0.0; 3];

                //push backwards some?
                VectorSet(&mut yawAngles, 0.0, (*(*pmv).ps).viewangles[YAW] + 180.0, 0.0);
                AngleVectors(&yawAngles, Some(&mut backDir), None, None);
                VectorScale(&backDir, 100.0, &mut (*(*pmv).ps).velocity);

                //jump!
                (*(*pmv).ps).velocity[2] = 300.0;
                (*(*pmv).ps).fd.forceJumpZStart = (*(*pmv).ps).origin[2]; //so we don't take damage if we land at same height
                                                                          //pm->ps->pm_flags |= PMF_JUMPING;//|PMF_SLOW_MO_FALL;

                //FIXME: NPCs yell?
                PM_AddEvent(EV_JUMP);
                //G_SoundOnEnt( ent, CHAN_BODY, "sound/weapons/force/jump.wav" );
                (*ucmd).upmove = 0; //clear any actual jump command
            }
        }
        (*ucmd).upmove = 0;
        (*ucmd).rightmove = 0;
        (*ucmd).forwardmove = 0;
    }
    //STAFF/DUAL SPIN ATTACK
    else if (*(*pmv).ps).saberMove == LS_SPINATTACK
        || (*(*pmv).ps).saberMove == LS_SPINATTACK_DUAL
    {
        (*ucmd).upmove = 0;
        (*ucmd).rightmove = 0;
        (*ucmd).forwardmove = 0;
        //lock their viewangles during these attacks.
        PM_SetPMViewAngle((*pmv).ps, (*(*pmv).ps).viewangles.as_mut_ptr(), ucmd);
    }
}

/// `PM_VehicleViewAngles` (bg_pmove.c:9402) — clamp the rider's view to what the vehicle
/// allows: the pilot is held to the vehicle's `lookPitch` (yaw/roll locked), a passenger
/// manning a turret is held to that turret's pitch/yaw clamps, and a free passenger is left
/// alone. A clamp bound of `-1` means "no clamp", `0/0` means "no allowance". `void`→`pub`,
/// no oracle (playerstate/command driven; only callee is the ported `PM_SetPMViewAngle`).
///
/// # Safety
/// `ps`/`ucmd` must be valid; `veh` must point to a live `bgEntity_t` whose `m_pVehicle`
/// (and its `m_pVehicleInfo`) is set.
pub unsafe fn PM_VehicleViewAngles(ps: *mut playerState_t, veh: *mut bgEntity_t, ucmd: *mut usercmd_t) {
    let pVeh: *mut Vehicle_t = (*veh).m_pVehicle;
    let mut setAngles: qboolean = QTRUE;
    let mut clampMin: vec3_t = [0.0; 3];
    let mut clampMax: vec3_t = [0.0; 3];
    let mut i: c_int;

    if !(*(*veh).m_pVehicle).m_pPilot.is_null()
        && (*(*(*veh).m_pVehicle).m_pPilot).s.number == (*ps).clientNum
    {
        //set the pilot's viewangles to the vehicle's viewangles
        // #ifdef VEH_CONTROL_SCHEME_4 was `if ( 1 )`; active #else build below:
        if BG_UnrestrainedPitchRoll(ps, (*veh).m_pVehicle) == 0 {
            //only if not if doing special free-roll/pitch control
            setAngles = QTRUE;
            clampMin[PITCH as usize] = -(*(*pVeh).m_pVehicleInfo).lookPitch;
            clampMax[PITCH as usize] = (*(*pVeh).m_pVehicleInfo).lookPitch;
            clampMin[YAW as usize] = 0.0;
            clampMax[YAW as usize] = 0.0;
            clampMin[ROLL as usize] = -1.0;
            clampMax[ROLL as usize] = -1.0;
        }
    } else {
        //NOTE: passengers can look around freely, UNLESS they're controlling a turret!
        i = 0;
        while i < MAX_VEHICLE_TURRETS as c_int {
            if (*(*(*veh).m_pVehicle).m_pVehicleInfo).turret[i as usize].passengerNum
                == (*ps).generic1
            {
                //this turret is my station
                //nevermind, don't clamp
                return;
                /*
                setAngles = qtrue;
                clampMin[PITCH] = veh->m_pVehicle->m_pVehicleInfo->turret[i].pitchClampUp;
                clampMax[PITCH] = veh->m_pVehicle->m_pVehicleInfo->turret[i].pitchClampDown;
                clampMin[YAW] = veh->m_pVehicle->m_pVehicleInfo->turret[i].yawClampRight;
                clampMax[YAW] = veh->m_pVehicle->m_pVehicleInfo->turret[i].yawClampLeft;
                clampMin[ROLL] = clampMax[ROLL] = 0;
                break;
                */
            }
            i += 1;
        }
    }
    if setAngles != QFALSE {
        i = 0;
        while i < 3 {
            //clamp viewangles
            if clampMin[i as usize] == -1.0 || clampMax[i as usize] == -1.0 {
                //no clamp
            } else if clampMin[i as usize] == 0.0 && clampMax[i as usize] == 0.0 {
                //no allowance
                //ps->viewangles[i] = veh->playerState->viewangles[i];
            } else {
                //allowance
                if (*ps).viewangles[i as usize] > clampMax[i as usize] {
                    (*ps).viewangles[i as usize] = clampMax[i as usize];
                } else if (*ps).viewangles[i as usize] < clampMin[i as usize] {
                    (*ps).viewangles[i as usize] = clampMin[i as usize];
                }
            }
            i += 1;
        }

        PM_SetPMViewAngle(ps, (*ps).viewangles.as_mut_ptr(), ucmd);
    }
}

// (bg_pmove.c 9467-9494 is a commented-out earlier PM_VehicleViewAngles variant — elided.)

/// `PM_WeaponOkOnVehicle` (bg_pmove.c:9497) — may this weapon be used while riding a
/// vehicle? (melee / saber / blaster only.) Pure classifier `switch`→`match`.
pub fn PM_WeaponOkOnVehicle(weapon: c_int) -> qboolean {
    //FIXME: check g_vehicleInfo for our vehicle?
    match weapon {
        //WP_NONE
        WP_MELEE | WP_SABER | WP_BLASTER => QTRUE,
        //WP_THERMAL
        _ => QFALSE,
    }
}

/// `PM_GetOkWeaponForVehicle` (bg_pmove.c:9514) — the lowest-indexed owned weapon that's
/// vehicle-legal (scanning `stats[STAT_WEAPONS]` bits), or -1 if none. `int`→`pub`.
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`.
pub unsafe fn PM_GetOkWeaponForVehicle() -> c_int {
    let pmv = *addr_of!(pm);
    let mut i: c_int = 0;

    while i < WP_NUM_WEAPONS {
        if ((*(*pmv).ps).stats[STAT_WEAPONS as usize] & (1 << i)) != 0 && PM_WeaponOkOnVehicle(i) != 0
        {
            //this one's good
            return i;
        }

        i += 1;
    }

    //oh dear!
    //assert(!"No valid veh weaps");
    -1
}

/// `PM_VehForcedTurning` (bg_pmove.c:9535) — force the vehicle to turn toward and travel to
/// its scripted turnaround destination entity (`vehTurnaroundIndex`): zero the strafe/forward
/// command (full up), ease the rider/vehicle view yaw+pitch toward the direction to the
/// destination (`0.6 * frametime` per frame), and re-seed `m_vPrevRiderViewAngles`. `void`→
/// `pub`, no oracle (playerstate/command driven; `PM_BGEntForNum`/`PM_SetPMViewAngle` ported).
///
/// # Safety
/// `pm`/`pml` must be installed; `veh` must be a live `bgEntity_t` (the early-out tolerates
/// a null `veh`/`m_pVehicle`, matching the C).
pub unsafe fn PM_VehForcedTurning(veh: *mut bgEntity_t) {
    let pmv = *addr_of!(pm);
    let dst: *mut bgEntity_t = PM_BGEntForNum((*(*veh).playerState).vehTurnaroundIndex);
    let pitchD: f32;
    let yawD: f32;
    let mut dir: vec3_t = [0.0; 3];

    if veh.is_null() || (*veh).m_pVehicle.is_null() {
        return;
    }

    if dst.is_null() {
        //can't find dest ent?
        return;
    }

    (*(*veh).m_pVehicle).m_ucmd.upmove = 127;
    (*pmv).cmd.upmove = 127;
    (*(*veh).m_pVehicle).m_ucmd.forwardmove = 0;
    (*pmv).cmd.forwardmove = 0;
    (*(*veh).m_pVehicle).m_ucmd.rightmove = 0;
    (*pmv).cmd.rightmove = 0;

    VectorSubtract(&(*dst).s.origin, &(*(*veh).playerState).origin, &mut dir);
    let dirIn = dir;
    vectoangles(&dirIn, &mut dir);

    yawD = AngleSubtract((*(*pmv).ps).viewangles[YAW as usize], dir[YAW as usize]);
    pitchD = AngleSubtract((*(*pmv).ps).viewangles[PITCH as usize], dir[PITCH as usize]);

    let yawD = yawD * (0.6 * (*addr_of!(pml)).frametime);
    let pitchD = pitchD * (0.6 * (*addr_of!(pml)).frametime);

    // #ifdef VEH_CONTROL_SCHEME_4 (defined nowhere) — the m_pVehicle->playerState write +
    // PITCH-zero + double PM_SetPMViewAngle + m_vPrevRiderViewAngles dead branch dropped; the
    // active #else build below simply turns pm->ps->viewangles directly.
    (*(*pmv).ps).viewangles[YAW as usize] =
        AngleSubtract((*(*pmv).ps).viewangles[YAW as usize], yawD);
    (*(*pmv).ps).viewangles[PITCH as usize] =
        AngleSubtract((*(*pmv).ps).viewangles[PITCH as usize], pitchD);

    PM_SetPMViewAngle((*pmv).ps, (*(*pmv).ps).viewangles.as_mut_ptr(), addr_of_mut!((*pmv).cmd));
}

/// `PM_VehFaceHyperspacePoint` (bg_pmove.c:9575) — rotate the vehicle to face the stored
/// hyperspace angles before the jump: zero strafe/forward (full up), turn each axis toward
/// `hyperSpaceAngles` at `90*frametime` per frame (yaw wraps 360, pitch/roll 180), and once
/// all three axes match (and we're past the teleport fraction) flag `EF2_HYPERSPACE`; until
/// then keep `hyperSpaceTime` advancing. `void`→`pub`, no oracle.
///
/// # Safety
/// `pm`/`pml` installed; `veh`/`m_pVehicle` may be null (early-out matches the C).
pub unsafe fn PM_VehFaceHyperspacePoint(veh: *mut bgEntity_t) {
    let pmv = *addr_of!(pm);

    if veh.is_null() || (*veh).m_pVehicle.is_null() {
    } else {
        let timeFrac: f32 = ((*pmv).cmd.serverTime - (*(*veh).playerState).hyperSpaceTime) as f32
            / HYPERSPACE_TIME as f32;
        let turnRate: f32;
        let mut aDelta: f32;
        let mut i: c_int;
        let mut matchedAxes: c_int = 0;

        (*(*veh).m_pVehicle).m_ucmd.upmove = 127;
        (*pmv).cmd.upmove = 127;
        (*(*veh).m_pVehicle).m_ucmd.forwardmove = 0;
        (*pmv).cmd.forwardmove = 0;
        (*(*veh).m_pVehicle).m_ucmd.rightmove = 0;
        (*pmv).cmd.rightmove = 0;

        turnRate = 90.0 * (*addr_of!(pml)).frametime;
        i = 0;
        while i < 3 {
            aDelta = AngleSubtract(
                (*(*veh).playerState).hyperSpaceAngles[i as usize],
                *(*(*veh).m_pVehicle).m_vOrientation.add(i as usize),
            );
            if (aDelta as f64).abs() < turnRate as f64 {
                //all is good
                (*(*pmv).ps).viewangles[i as usize] =
                    (*(*veh).playerState).hyperSpaceAngles[i as usize];
                matchedAxes += 1;
            } else {
                aDelta = AngleSubtract(
                    (*(*veh).playerState).hyperSpaceAngles[i as usize],
                    (*(*pmv).ps).viewangles[i as usize],
                );
                if (aDelta as f64).abs() < turnRate as f64 {
                    (*(*pmv).ps).viewangles[i as usize] =
                        (*(*veh).playerState).hyperSpaceAngles[i as usize];
                } else if aDelta > 0.0 {
                    if i == YAW as c_int {
                        (*(*pmv).ps).viewangles[i as usize] =
                            AngleNormalize360((*(*pmv).ps).viewangles[i as usize] + turnRate);
                    } else {
                        (*(*pmv).ps).viewangles[i as usize] =
                            AngleNormalize180((*(*pmv).ps).viewangles[i as usize] + turnRate);
                    }
                } else if i == YAW as c_int {
                    (*(*pmv).ps).viewangles[i as usize] =
                        AngleNormalize360((*(*pmv).ps).viewangles[i as usize] - turnRate);
                } else {
                    (*(*pmv).ps).viewangles[i as usize] =
                        AngleNormalize180((*(*pmv).ps).viewangles[i as usize] - turnRate);
                }
            }
            i += 1;
        }

        // #ifdef VEH_CONTROL_SCHEME_4 (defined nowhere) — the playerState write + PITCH-zero +
        // double PM_SetPMViewAngle + m_vPrevRiderViewAngles dead branch dropped; active #else:
        PM_SetPMViewAngle((*pmv).ps, (*(*pmv).ps).viewangles.as_mut_ptr(), addr_of_mut!((*pmv).cmd));

        if timeFrac < HYPERSPACE_TELEPORT_FRAC {
            //haven't gone through yet
            if matchedAxes < 3 {
                //not facing the right dir yet
                //keep hyperspace time up to date
                (*(*veh).playerState).hyperSpaceTime += (*addr_of!(pml)).msec;
            } else if (*(*veh).playerState).eFlags2 & EF2_HYPERSPACE == 0 {
                //flag us as ready to hyperspace!
                (*(*veh).playerState).eFlags2 |= EF2_HYPERSPACE;
            }
        }
    }
}

/// `BG_VehicleAdjustBBoxForOrientation` (bg_pmove.c:9656) — recompute a vehicle's bounding
/// box to fit its current `m_vOrientation`: fighters/fliers get a dynamic box built from the
/// 8 extreme points of the length/width/height extents rotated into the vehicle's axes (and
/// only adopted if the resulting box isn't start/all-solid via the supplied trace callback);
/// every other vehicle type uses a static width/height box. `void`→`pub`, no oracle (trace-
/// callback driven; only callees are the ported `AnglesToAxis`/`Vector*` math).
///
/// # Safety
/// `veh` may be null (early-out matches the C); `origin`/`mins`/`maxs` must be valid `vec3_t`
/// pointers; `localTrace`, when `Some`, must be a valid world-trace callback.
pub unsafe fn BG_VehicleAdjustBBoxForOrientation(
    veh: *mut Vehicle_t,
    origin: *mut vec3_t,
    mins: *mut vec3_t,
    maxs: *mut vec3_t,
    clientNum: c_int,
    tracemask: c_int,
    localTrace: Option<
        unsafe extern "C" fn(
            results: *mut trace_t,
            start: *const vec_t,
            mins: *const vec_t,
            maxs: *const vec_t,
            end: *const vec_t,
            passEntityNum: c_int,
            contentMask: c_int,
        ),
    >,
) {
    if veh.is_null()
        || (*(*veh).m_pVehicleInfo).length == 0.0
        || (*(*veh).m_pVehicleInfo).width == 0.0
        || (*(*veh).m_pVehicleInfo).height == 0.0
    //|| veh->m_LandTrace.fraction < 1.0f )
    {
        return;
    } else if (*(*veh).m_pVehicleInfo).r#type != VH_FIGHTER
        //&& veh->m_pVehicleInfo->type != VH_SPEEDER
        && (*(*veh).m_pVehicleInfo).r#type != VH_FLIER
    {
        //only those types of vehicles have dynamic bboxes, the rest just use a static bbox
        VectorSet(
            &mut *maxs,
            (*(*veh).m_pVehicleInfo).width / 2.0,
            (*(*veh).m_pVehicleInfo).width / 2.0,
            (*(*veh).m_pVehicleInfo).height + DEFAULT_MINS_2 as f32,
        );
        VectorSet(
            &mut *mins,
            (*(*veh).m_pVehicleInfo).width / -2.0,
            (*(*veh).m_pVehicleInfo).width / -2.0,
            DEFAULT_MINS_2 as f32,
        );
        return;
    } else {
        let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];
        let mut point: [vec3_t; 8] = [[0.0; 3]; 8];
        let mut newMins: vec3_t = [0.0; 3];
        let mut newMaxs: vec3_t = [0.0; 3];
        let mut curAxis: c_int;
        let mut i: c_int;
        let mut trace: trace_t = core::mem::zeroed();

        let vinfo = (*veh).m_pVehicleInfo;
        AnglesToAxis(&*((*veh).m_vOrientation as *const vec3_t), &mut axis);
        VectorMA(&*origin, (*vinfo).length / 2.0, &axis[0], &mut point[0]);
        VectorMA(&*origin, -(*vinfo).length / 2.0, &axis[0], &mut point[1]);
        //extrapolate each side up and down
        let t = point[0];
        VectorMA(&t, (*vinfo).height / 2.0, &axis[2], &mut point[0]);
        let t = point[0];
        VectorMA(&t, -(*vinfo).height, &axis[2], &mut point[2]);
        let t = point[1];
        VectorMA(&t, (*vinfo).height / 2.0, &axis[2], &mut point[1]);
        let t = point[1];
        VectorMA(&t, -(*vinfo).height, &axis[2], &mut point[3]);

        VectorMA(&*origin, (*vinfo).width / 2.0, &axis[1], &mut point[4]);
        VectorMA(&*origin, -(*vinfo).width / 2.0, &axis[1], &mut point[5]);
        //extrapolate each side up and down
        let t = point[4];
        VectorMA(&t, (*vinfo).height / 2.0, &axis[2], &mut point[4]);
        let t = point[4];
        VectorMA(&t, -(*vinfo).height, &axis[2], &mut point[6]);
        let t = point[5];
        VectorMA(&t, (*vinfo).height / 2.0, &axis[2], &mut point[5]);
        let t = point[5];
        VectorMA(&t, -(*vinfo).height, &axis[2], &mut point[7]);
        // (C: a commented-out alternate up/down extrapolation of point[4]/point[5] — elided.)
        //Now inflate a bbox around these points
        VectorCopy(&*origin, &mut newMins);
        VectorCopy(&*origin, &mut newMaxs);
        curAxis = 0;
        while curAxis < 3 {
            i = 0;
            while i < 8 {
                if point[i as usize][curAxis as usize] > newMaxs[curAxis as usize] {
                    newMaxs[curAxis as usize] = point[i as usize][curAxis as usize];
                } else if point[i as usize][curAxis as usize] < newMins[curAxis as usize] {
                    newMins[curAxis as usize] = point[i as usize][curAxis as usize];
                }
                i += 1;
            }
            curAxis += 1;
        }
        let newMinsIn = newMins;
        VectorSubtract(&newMinsIn, &*origin, &mut newMins);
        let newMaxsIn = newMaxs;
        VectorSubtract(&newMaxsIn, &*origin, &mut newMaxs);
        //now see if that's a valid way to be
        if let Some(lt) = localTrace {
            lt(
                &mut trace,
                (*origin).as_ptr(),
                newMins.as_ptr(),
                newMaxs.as_ptr(),
                (*origin).as_ptr(),
                clientNum,
                tracemask,
            );
        } else {
            //don't care about solid stuff then
            trace.allsolid = 0;
            trace.startsolid = 0;
        }
        if trace.startsolid == 0 && trace.allsolid == 0 {
            //let's use it!
            VectorCopy(&newMins, &mut *mins);
            VectorCopy(&newMaxs, &mut *maxs);
        }
        //else: just use the last one, I guess...?
        //FIXME: make it as close as possible?  Or actually prevent the change in m_vOrientation?  Or push away from anything we hit?
    }
}

/// `JETPACK_HOVER_HEIGHT` (bg_pmove.c:9751) — the file-local `#define` for the height the
/// jetpack tries to maintain off the ground; used only by `PmoveSingle`'s PM_JETPACK block.
const JETPACK_HOVER_HEIGHT: c_int = 64;

/// `PM_MoveForKata` (bg_pmove.c:9755) — override the move command for the multi-frame
/// kata / soulcal special attacks: the forward-spinning staff soulcal can roll out or
/// lurch forward (and jumps mid-animation), and the medium/strong katas (`BOTH_A2/A3_SPECIAL`)
/// drive scripted forward lunges at fixed `legsTimer` windows. `void`→`pub`, no oracle
/// (playerstate / event-callback driven).
///
/// # Safety
/// `pm` must point to a valid `pmove_t` with a live `ps`; `ucmd` must be valid (the caller
/// passes `&pm->cmd`).
pub unsafe fn PM_MoveForKata(ucmd: *mut usercmd_t) {
    let pmv = *addr_of!(pm);

    if (*(*pmv).ps).legsAnim == BOTH_A7_SOULCAL && (*(*pmv).ps).saberMove == LS_STAFF_SOULCAL {
        //forward spinning staff attack
        (*ucmd).upmove = 0;

        if PM_CanRollFromSoulCal((*pmv).ps) != QFALSE {
            (*ucmd).upmove = -127;
            (*ucmd).rightmove = 0;
            if (*ucmd).forwardmove < 0 {
                (*ucmd).forwardmove = 0;
            }
        } else {
            (*ucmd).rightmove = 0;
            //FIXME: don't slide off people/obstacles?
            if (*(*pmv).ps).legsTimer >= 2750 {
                //not at end
                //push forward
                (*ucmd).forwardmove = 64;
            } else {
                (*ucmd).forwardmove = 0;
            }
        }
        if (*(*pmv).ps).legsTimer >= 2650 && (*(*pmv).ps).legsTimer < 2850 {
            //the jump
            if (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE {
                //still on ground?
                //jump!
                (*(*pmv).ps).velocity[2] = 250.0;
                (*(*pmv).ps).fd.forceJumpZStart = (*(*pmv).ps).origin[2]; //so we don't take damage if we land at same height
                                                                          //	pm->ps->pm_flags |= PMF_JUMPING;//|PMF_SLOW_MO_FALL;
                                                                          //FIXME: NPCs yell?
                PM_AddEvent(EV_JUMP);
            }
        }
    } else if (*(*pmv).ps).legsAnim == BOTH_A2_SPECIAL {
        //medium kata
        (*pmv).cmd.rightmove = 0;
        (*pmv).cmd.upmove = 0;
        if (*(*pmv).ps).legsTimer < 2700 && (*(*pmv).ps).legsTimer > 2300 {
            (*pmv).cmd.forwardmove = 127;
        } else if (*(*pmv).ps).legsTimer < 900 && (*(*pmv).ps).legsTimer > 500 {
            (*pmv).cmd.forwardmove = 127;
        } else {
            (*pmv).cmd.forwardmove = 0;
        }
    } else if (*(*pmv).ps).legsAnim == BOTH_A3_SPECIAL {
        //strong kata
        (*pmv).cmd.rightmove = 0;
        (*pmv).cmd.upmove = 0;
        if (*(*pmv).ps).legsTimer < 1700 && (*(*pmv).ps).legsTimer > 1000 {
            (*pmv).cmd.forwardmove = 127;
        } else {
            (*pmv).cmd.forwardmove = 0;
        }
    } else {
        (*pmv).cmd.forwardmove = 0;
        (*pmv).cmd.rightmove = 0;
        (*pmv).cmd.upmove = 0;
    }
}

/// `PmoveSingle` (bg_pmove.c:9837) — one fixed-step slice of the player move: install the
/// per-frame `pm`/`pml` context, fold every "you can't move/turn right now" special-case
/// (saber locks, kata, taunts, lunges, emplaced guns, disruptor charge ...) into the command,
/// update the view basis, then run the appropriate move integrator (`PM_WalkMove`/`PM_AirMove`/
/// `PM_WaterMove`/`PM_FlyMove`/...) and the trailing weapon/animation/event passes. `static`→
/// `pub`, **no oracle** (the trace/callback-driven `pm` keystone — its move-fn callees are
/// verified transitively here).
///
/// The callee set is ported and wired; the only remaining faithful-stub gaps are the
/// client-only `FF_Play` force-feedback calls (not in the server module) and the ghoul2
/// tail of `PM_AdjustStandAnimForSlope`. The surrounding control flow, field accesses, and
/// ported-fn / vehicle fn-ptr calls are transcribed verbatim. **QAGAME build**: the
/// `#ifdef QAGAME` branches are taken and the
/// `#else` (cgame) branches dropped — incl. `BG_FighterUpdate` and the cgame view/orient
/// handling in the vehicle block (the `PM_GroundTrace` board-check precedent). `#if 0` blocks
/// and the cgame `_TESTING_VEH_PREDICTION` debug draw are dropped/noted.
#[allow(non_snake_case)]
pub unsafe fn PmoveSingle(pmove: *mut pmove_t) {
    let mut stiffenedUp: qboolean = QFALSE;
    let mut gDist: f32 = 0.0;
    let mut noAnimate: qboolean = QFALSE;
    let mut savedGravity: c_int = 0;

    *addr_of_mut!(pm) = pmove;
    let pmv = *addr_of!(pm);

    if (*(*pmv).ps).emplacedIndex != 0 {
        if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
            //hackerrific.
            (*pmv).cmd.buttons &= !BUTTON_ALT_ATTACK;
            (*pmv).cmd.buttons |= BUTTON_ATTACK;
        }
    }

    //set up these "global" bg ents
    *addr_of_mut!(pm_entSelf) = PM_BGEntForNum((*(*pmv).ps).clientNum);
    if (*(*pmv).ps).m_iVehicleNum != 0 {
        if (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int {
            //player riding vehicle
            *addr_of_mut!(pm_entVeh) = PM_BGEntForNum((*(*pmv).ps).m_iVehicleNum);
        } else {
            //vehicle with player pilot
            *addr_of_mut!(pm_entVeh) = PM_BGEntForNum((*(*pmv).ps).m_iVehicleNum - 1);
        }
    } else {
        //no vehicle ent
        *addr_of_mut!(pm_entVeh) = null_mut();
    }

    *addr_of_mut!(gPMDoSlowFall) = PM_DoSlowFall();

    // this counter lets us debug movement problems with a journal
    // by setting a conditional breakpoint fot the previous frame
    *addr_of_mut!(c_pmove) += 1;

    // clear results
    (*pmv).numtouch = 0;
    (*pmv).watertype = 0;
    (*pmv).waterlevel = 0;

    if PM_IsRocketTrooper() != QFALSE {
        //kind of nasty, don't let them crouch or anything if nonhumanoid (probably a rockettrooper)
        if (*pmv).cmd.upmove < 0 {
            (*pmv).cmd.upmove = 0;
        }
    }

    if (*(*pmv).ps).pm_type == PM_FLOAT {
        //You get no control over where you go in grip movement
        stiffenedUp = QTRUE;
    } else if (*(*pmv).ps).eFlags & EF_DISINTEGRATION != 0 {
        stiffenedUp = QTRUE;
    } else if BG_SaberLockBreakAnim((*(*pmv).ps).legsAnim) != QFALSE
        || BG_SaberLockBreakAnim((*(*pmv).ps).torsoAnim) != QFALSE
        || (*(*pmv).ps).saberLockTime >= (*pmv).cmd.serverTime
    {
        //can't move or turn
        stiffenedUp = QTRUE;
        PM_SetPMViewAngle(
            (*pmv).ps,
            (*(*pmv).ps).viewangles.as_mut_ptr(),
            addr_of_mut!((*pmv).cmd),
        );
    } else if (*(*pmv).ps).saberMove == LS_A_BACK
        || (*(*pmv).ps).saberMove == LS_A_BACK_CR
        || (*(*pmv).ps).saberMove == LS_A_BACKSTAB
        || (*(*pmv).ps).saberMove == LS_A_FLIP_STAB
        || (*(*pmv).ps).saberMove == LS_A_FLIP_SLASH
        || (*(*pmv).ps).saberMove == LS_A_JUMP_T__B_
        || (*(*pmv).ps).saberMove == LS_DUAL_LR
        || (*(*pmv).ps).saberMove == LS_DUAL_FB
    {
        if (*(*pmv).ps).legsAnim == BOTH_JUMPFLIPSTABDOWN
            || (*(*pmv).ps).legsAnim == BOTH_JUMPFLIPSLASHDOWN1
        {
            //flipover medium stance attack
            if (*(*pmv).ps).legsTimer < 1600 && (*(*pmv).ps).legsTimer > 900 {
                (*(*pmv).ps).viewangles[YAW as usize] += (*addr_of!(pml)).frametime * 240.0;
                PM_SetPMViewAngle(
                    (*pmv).ps,
                    (*(*pmv).ps).viewangles.as_mut_ptr(),
                    addr_of_mut!((*pmv).cmd),
                );
            }
        }
        stiffenedUp = QTRUE;
    } else if (*(*pmv).ps).legsAnim == BOTH_A2_STABBACK1
        || (*(*pmv).ps).legsAnim == BOTH_ATTACK_BACK
        || (*(*pmv).ps).legsAnim == BOTH_CROUCHATTACKBACK1
        || (*(*pmv).ps).legsAnim == BOTH_FORCELEAP2_T__B_
        || (*(*pmv).ps).legsAnim == BOTH_JUMPFLIPSTABDOWN
        || (*(*pmv).ps).legsAnim == BOTH_JUMPFLIPSLASHDOWN1
    {
        stiffenedUp = QTRUE;
    } else if (*(*pmv).ps).legsAnim == BOTH_ROLL_STAB {
        stiffenedUp = QTRUE;
        PM_SetPMViewAngle(
            (*pmv).ps,
            (*(*pmv).ps).viewangles.as_mut_ptr(),
            addr_of_mut!((*pmv).cmd),
        );
    } else if (*(*pmv).ps).heldByClient != 0 {
        stiffenedUp = QTRUE;
    } else if BG_KickMove((*(*pmv).ps).saberMove) != QFALSE
        || BG_KickingAnim((*(*pmv).ps).legsAnim) != QFALSE
    {
        stiffenedUp = QTRUE;
    } else if BG_InGrappleMove((*(*pmv).ps).torsoAnim) != 0 {
        stiffenedUp = QTRUE;
        PM_SetPMViewAngle(
            (*pmv).ps,
            (*(*pmv).ps).viewangles.as_mut_ptr(),
            addr_of_mut!((*pmv).cmd),
        );
    } else if (*(*pmv).ps).saberMove == LS_STABDOWN_DUAL
        || (*(*pmv).ps).saberMove == LS_STABDOWN_STAFF
        || (*(*pmv).ps).saberMove == LS_STABDOWN
    {
        //FIXME: need to only move forward until we bump into our target...?
        if (*(*pmv).ps).legsTimer < 800 {
            //freeze movement near end of anim
            stiffenedUp = QTRUE;
            PM_SetPMViewAngle(
                (*pmv).ps,
                (*(*pmv).ps).viewangles.as_mut_ptr(),
                addr_of_mut!((*pmv).cmd),
            );
        } else {
            //force forward til then
            (*pmv).cmd.rightmove = 0;
            (*pmv).cmd.upmove = 0;
            (*pmv).cmd.forwardmove = 64;
        }
    } else if (*(*pmv).ps).saberMove == LS_PULL_ATTACK_STAB
        || (*(*pmv).ps).saberMove == LS_PULL_ATTACK_SWING
    {
        stiffenedUp = QTRUE;
    } else if BG_SaberInKata((*(*pmv).ps).saberMove) != QFALSE
        || BG_InKataAnim((*(*pmv).ps).torsoAnim) != QFALSE
        || BG_InKataAnim((*(*pmv).ps).legsAnim) != QFALSE
    {
        PM_MoveForKata(addr_of_mut!((*pmv).cmd));
    } else if BG_FullBodyTauntAnim((*(*pmv).ps).legsAnim) != QFALSE
        && BG_FullBodyTauntAnim((*(*pmv).ps).torsoAnim) != QFALSE
    {
        if (*pmv).cmd.buttons & BUTTON_ATTACK != 0
            || (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0
            || (*pmv).cmd.buttons & BUTTON_FORCEPOWER != 0
            || (*pmv).cmd.buttons & BUTTON_FORCEGRIP != 0
            || (*pmv).cmd.buttons & BUTTON_FORCE_LIGHTNING != 0
            || (*pmv).cmd.buttons & BUTTON_FORCE_DRAIN != 0
            || (*pmv).cmd.upmove != 0
        {
            //stop the anim
            if (*(*pmv).ps).legsAnim == BOTH_MEDITATE && (*(*pmv).ps).torsoAnim == BOTH_MEDITATE {
                PM_SetAnim(
                    SETANIM_BOTH,
                    BOTH_MEDITATE_END,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    0,
                );
            } else {
                (*(*pmv).ps).torsoTimer = 0;
                (*(*pmv).ps).legsTimer = 0;
            }
            if (*(*pmv).ps).forceHandExtend == HANDEXTEND_TAUNT {
                (*(*pmv).ps).forceHandExtend = 0;
            }
        } else {
            if (*(*pmv).ps).legsAnim == BOTH_MEDITATE {
                if (*(*pmv).ps).legsTimer < 100 {
                    (*(*pmv).ps).legsTimer = 100;
                }
            }
            if (*(*pmv).ps).torsoAnim == BOTH_MEDITATE {
                if (*(*pmv).ps).torsoTimer < 100 {
                    (*(*pmv).ps).legsTimer = 100;
                }
                (*(*pmv).ps).forceHandExtend = HANDEXTEND_TAUNT;
                (*(*pmv).ps).forceHandExtendTime = (*pmv).cmd.serverTime + 100;
            }
            if (*(*pmv).ps).legsTimer > 0 || (*(*pmv).ps).torsoTimer > 0 {
                stiffenedUp = QTRUE;
                PM_SetPMViewAngle(
                    (*pmv).ps,
                    (*(*pmv).ps).viewangles.as_mut_ptr(),
                    addr_of_mut!((*pmv).cmd),
                );
                (*pmv).cmd.rightmove = 0;
                (*pmv).cmd.upmove = 0;
                (*pmv).cmd.forwardmove = 0;
                (*pmv).cmd.buttons = 0;
            }
        }
    } else if (*(*pmv).ps).legsAnim == BOTH_MEDITATE_END && (*(*pmv).ps).legsTimer > 0 {
        stiffenedUp = QTRUE;
        PM_SetPMViewAngle(
            (*pmv).ps,
            (*(*pmv).ps).viewangles.as_mut_ptr(),
            addr_of_mut!((*pmv).cmd),
        );
        (*pmv).cmd.rightmove = 0;
        (*pmv).cmd.upmove = 0;
        (*pmv).cmd.forwardmove = 0;
        (*pmv).cmd.buttons = 0;
    } else if (*(*pmv).ps).legsAnim == BOTH_FORCELAND1
        || (*(*pmv).ps).legsAnim == BOTH_FORCELANDBACK1
        || (*(*pmv).ps).legsAnim == BOTH_FORCELANDRIGHT1
        || (*(*pmv).ps).legsAnim == BOTH_FORCELANDLEFT1
    {
        //can't move while in a force land
        stiffenedUp = QTRUE;
    }

    if (*(*pmv).ps).saberMove == LS_A_LUNGE {
        //can't move during lunge
        (*pmv).cmd.upmove = 0;
        (*pmv).cmd.rightmove = 0;
        if (*(*pmv).ps).legsTimer > 500 {
            (*pmv).cmd.forwardmove = 127;
        } else {
            (*pmv).cmd.forwardmove = 0;
        }
    }

    if (*(*pmv).ps).saberMove == LS_A_JUMP_T__B_ {
        //can't move during leap
        if (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE {
            //hit the ground
            (*pmv).cmd.forwardmove = 0;
        }
        (*pmv).cmd.upmove = 0;
        (*pmv).cmd.rightmove = 0;
    }

    // (the #if 0 BOTH_KISSER1LOOP/BOTH_KISSEE1LOOP stiffenedUp block is dead and dropped.)

    if (*(*pmv).ps).emplacedIndex != 0 {
        if (*pmv).cmd.forwardmove < 0 || PM_GroundDistance() > 32.0 {
            (*(*pmv).ps).emplacedIndex = 0;
            (*(*pmv).ps).saberHolstered = 0;
        } else {
            stiffenedUp = QTRUE;
        }
    }

    // (the commented-out "no move while charging disruptor" block above this in C is dropped.)

    if (*(*pmv).ps).weapon == WP_DISRUPTOR && (*(*pmv).ps).weaponstate == WEAPON_CHARGING_ALT {
        //not allowed to move while charging the disruptor
        if (*pmv).cmd.forwardmove != 0 || (*pmv).cmd.rightmove != 0 || (*pmv).cmd.upmove > 0 {
            //get out
            (*(*pmv).ps).weaponstate = WEAPON_READY;
            (*(*pmv).ps).weaponTime = 1000;
            PM_AddEventWithParm(EV_WEAPON_CHARGE, WP_DISRUPTOR); //cut the weapon charge sound
            (*pmv).cmd.upmove = 0;
        }
    } else if (*(*pmv).ps).weapon == WP_DISRUPTOR && (*(*pmv).ps).zoomMode == 1 {
        //can't jump
        if (*pmv).cmd.upmove > 0 {
            (*pmv).cmd.upmove = 0;
        }
    }

    if stiffenedUp != QFALSE {
        (*pmv).cmd.forwardmove = 0;
        (*pmv).cmd.rightmove = 0;
        (*pmv).cmd.upmove = 0;
    }

    if (*(*pmv).ps).fd.forceGripCripple != 0 {
        //don't let attack or alt attack if being gripped I guess
        (*pmv).cmd.buttons &= !BUTTON_ATTACK;
        (*pmv).cmd.buttons &= !BUTTON_ALT_ATTACK;
    }

    if BG_InRoll((*pmv).ps, (*(*pmv).ps).legsAnim) != QFALSE {
        //can't roll unless you're able to move normally
        BG_CmdForRoll((*pmv).ps, (*(*pmv).ps).legsAnim, addr_of_mut!((*pmv).cmd));
    }

    PM_CmdForSaberMoves(addr_of_mut!((*pmv).cmd));

    BG_AdjustClientSpeed((*pmv).ps, addr_of_mut!((*pmv).cmd), (*pmv).cmd.serverTime);

    if (*(*pmv).ps).stats[STAT_HEALTH as usize] <= 0 {
        (*pmv).tracemask &= !CONTENTS_BODY; // corpses can fly through bodies
    }

    // make sure walking button is clear if they are running, to avoid
    // proxy no-footsteps cheats
    if ((*pmv).cmd.forwardmove as c_int).abs() > 64 || ((*pmv).cmd.rightmove as c_int).abs() > 64 {
        (*pmv).cmd.buttons &= !BUTTON_WALKING;
    }

    // set the talk balloon flag
    if (*pmv).cmd.buttons & BUTTON_TALK != 0 {
        (*(*pmv).ps).eFlags |= EF_TALK;
    } else {
        (*(*pmv).ps).eFlags &= !EF_TALK;
    }

    *addr_of_mut!(pm_cancelOutZoom) = QFALSE;
    if (*(*pmv).ps).weapon == WP_DISRUPTOR && (*(*pmv).ps).zoomMode == 1 {
        if (*pmv).cmd.buttons & BUTTON_ALT_ATTACK != 0
            && (*pmv).cmd.buttons & BUTTON_ATTACK == 0
            && (*(*pmv).ps).zoomLocked != 0
        {
            *addr_of_mut!(pm_cancelOutZoom) = QTRUE;
        }
    }
    // In certain situations, we may want to control which attack buttons are pressed and what kind of functionality
    //	is attached to them
    PM_AdjustAttackStates(pmv);

    // clear the respawned flag if attack and use are cleared
    if (*(*pmv).ps).stats[STAT_HEALTH as usize] > 0
        && (*pmv).cmd.buttons & (BUTTON_ATTACK | BUTTON_USE_HOLDABLE) == 0
    {
        (*(*pmv).ps).pm_flags &= !PMF_RESPAWNED;
    }

    // if talk button is down, dissallow all other input
    // this is to prevent any possible intercept proxy from
    // adding fake talk balloons
    if (*pmove).cmd.buttons & BUTTON_TALK != 0 {
        // keep the talk button set tho for when the cmd.serverTime > 66 msec
        // and the same cmd is used multiple times in Pmove
        (*pmove).cmd.buttons = BUTTON_TALK;
        (*pmove).cmd.forwardmove = 0;
        (*pmove).cmd.rightmove = 0;
        (*pmove).cmd.upmove = 0;
    }

    // clear all pmove local vars
    core::ptr::write_bytes(addr_of_mut!(pml) as *mut u8, 0, core::mem::size_of::<pml_t>());

    // determine the time
    (*addr_of_mut!(pml)).msec = (*pmove).cmd.serverTime - (*(*pmv).ps).commandTime;
    if (*addr_of!(pml)).msec < 1 {
        (*addr_of_mut!(pml)).msec = 1;
    } else if (*addr_of!(pml)).msec > 200 {
        (*addr_of_mut!(pml)).msec = 200;
    }

    (*(*pmv).ps).commandTime = (*pmove).cmd.serverTime;

    // save old org in case we get stuck
    VectorCopy(&(*(*pmv).ps).origin, &mut (*addr_of_mut!(pml)).previous_origin);

    // save old velocity for crashlanding
    VectorCopy(&(*(*pmv).ps).velocity, &mut (*addr_of_mut!(pml)).previous_velocity);

    // C: pml.frametime = pml.msec * 0.001; — `0.001` is a double, so the product is f64
    // before narrowing to the f32 field (the bg_pmove.rs f64-promote-where-C-does convention).
    (*addr_of_mut!(pml)).frametime = ((*addr_of!(pml)).msec as f64 * 0.001) as f32;

    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
        && !(*addr_of!(pm_entSelf)).is_null()
        && (*(*addr_of!(pm_entSelf))).s.NPC_class == CLASS_VEHICLE
    {
        //we are a vehicle
        let veh: *mut bgEntity_t = *addr_of!(pm_entSelf);
        debug_assert!(!veh.is_null() && !(*veh).m_pVehicle.is_null());
        if !veh.is_null() && !(*veh).m_pVehicle.is_null() {
            (*(*veh).m_pVehicle).m_fTimeModifier = (*addr_of!(pml)).frametime * 60.0;
        }
    } else if (*(*addr_of!(pm_entSelf))).s.NPC_class != CLASS_VEHICLE
        && (*(*pmv).ps).m_iVehicleNum != 0
    {
        let veh: *mut bgEntity_t = *addr_of!(pm_entVeh);

        if !veh.is_null()
            && !(*veh).playerState.is_null()
            && ((*pmv).cmd.serverTime - (*(*veh).playerState).hyperSpaceTime) < HYPERSPACE_TIME
        {
            //going into hyperspace, turn to face the right angles
            PM_VehFaceHyperspacePoint(veh);
        } else if !veh.is_null()
            && !(*veh).playerState.is_null()
            && (*(*veh).playerState).vehTurnaroundIndex != 0
            && (*(*veh).playerState).vehTurnaroundTime > (*pmv).cmd.serverTime
        {
            //riding this vehicle, turn my view too
            PM_VehForcedTurning(veh);
        }
    }

    if (*(*pmv).ps).legsAnim == BOTH_FORCEWALLRUNFLIP_ALT && (*(*pmv).ps).legsTimer > 0 {
        let mut vFwd: vec3_t = [0.0; 3];
        let mut fwdAng: vec3_t = [0.0; 3];
        VectorSet(&mut fwdAng, 0.0, (*(*pmv).ps).viewangles[YAW as usize], 0.0);

        AngleVectors(&fwdAng, Some(&mut vFwd), None, None);
        if (*(*pmv).ps).groundEntityNum == ENTITYNUM_NONE {
            let savZ: f32 = (*(*pmv).ps).velocity[2];
            VectorScale(&vFwd, 100.0, &mut (*(*pmv).ps).velocity);
            (*(*pmv).ps).velocity[2] = savZ;
        }
        (*pmv).cmd.forwardmove = 0;
        (*pmv).cmd.rightmove = 0;
        (*pmv).cmd.upmove = 0;
        // PM_AdjustAnglesForWallRunUpFlipAlt is ported — call it directly.
        PM_AdjustAnglesForWallRunUpFlipAlt(addr_of_mut!((*pmv).cmd));
    }

    //	PM_AdjustAngleForWallRun(pm->ps, &pm->cmd, qtrue);
    //	PM_AdjustAnglesForStabDown( pm->ps, &pm->cmd );
    PM_AdjustAngleForWallJump((*pmv).ps, addr_of_mut!((*pmv).cmd), QTRUE);
    PM_AdjustAngleForWallRunUp((*pmv).ps, addr_of_mut!((*pmv).cmd), QTRUE);
    PM_AdjustAngleForWallRun((*pmv).ps, addr_of_mut!((*pmv).cmd), QTRUE);

    if (*(*pmv).ps).saberMove == LS_A_JUMP_T__B_
        || (*(*pmv).ps).saberMove == LS_A_LUNGE
        || (*(*pmv).ps).saberMove == LS_A_BACK_CR
        || (*(*pmv).ps).saberMove == LS_A_BACK
        || (*(*pmv).ps).saberMove == LS_A_BACKSTAB
    {
        PM_SetPMViewAngle(
            (*pmv).ps,
            (*(*pmv).ps).viewangles.as_mut_ptr(),
            addr_of_mut!((*pmv).cmd),
        );
    }

    // (the #if 0 BOTH_KISSER1LOOP/BOTH_KISSEE1LOOP pitch-zero block is dead and dropped.)

    PM_SetSpecialMoveValues();

    // update the viewangles
    PM_UpdateViewAngles((*pmv).ps, addr_of!((*pmv).cmd));

    AngleVectors(
        &(*(*pmv).ps).viewangles,
        Some(&mut (*addr_of_mut!(pml)).forward),
        Some(&mut (*addr_of_mut!(pml)).right),
        Some(&mut (*addr_of_mut!(pml)).up),
    );

    if (*pmv).cmd.upmove < 10 && (*(*pmv).ps).pm_flags & PMF_STUCK_TO_WALL == 0 {
        // not holding jump
        (*(*pmv).ps).pm_flags &= !PMF_JUMP_HELD;
    }

    // decide if backpedaling animations should be used
    if (*pmv).cmd.forwardmove < 0 {
        (*(*pmv).ps).pm_flags |= PMF_BACKWARDS_RUN;
    } else if (*pmv).cmd.forwardmove > 0
        || ((*pmv).cmd.forwardmove == 0 && (*pmv).cmd.rightmove != 0)
    {
        (*(*pmv).ps).pm_flags &= !PMF_BACKWARDS_RUN;
    }

    if (*(*pmv).ps).pm_type >= PM_DEAD {
        (*pmv).cmd.forwardmove = 0;
        (*pmv).cmd.rightmove = 0;
        (*pmv).cmd.upmove = 0;
    }

    // (the commented-out SS_STAFF kick-for-condition block in C is dropped.)

    if (*(*pmv).ps).saberLockTime >= (*pmv).cmd.serverTime {
        (*pmv).cmd.upmove = 0;
        (*pmv).cmd.forwardmove = 0; //50;
        (*pmv).cmd.rightmove = 0; //*= 0.1;
    }

    if (*(*pmv).ps).pm_type == PM_SPECTATOR {
        PM_CheckDuck();
        if (*pmv).noSpecMove == 0 {
            PM_FlyMove();
        }
        PM_DropTimers();
        return;
    }

    if (*(*pmv).ps).pm_type == PM_NOCLIP {
        if (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int {
            PM_NoclipMove();
            PM_DropTimers();
            return;
        }
    }

    if (*(*pmv).ps).pm_type == PM_FREEZE {
        return; // no movement at all
    }

    if (*(*pmv).ps).pm_type == PM_INTERMISSION || (*(*pmv).ps).pm_type == PM_SPINTERMISSION {
        return; // no movement at all
    }

    // set watertype, and waterlevel
    PM_SetWaterLevel();
    (*addr_of_mut!(pml)).previous_waterlevel = (*pmove).waterlevel;

    // set mins, maxs, and viewheight
    PM_CheckDuck();

    if (*(*pmv).ps).pm_type == PM_JETPACK {
        gDist = PM_GroundDistance();
        savedGravity = (*(*pmv).ps).gravity;

        if gDist < (JETPACK_HOVER_HEIGHT + 64) as f32 {
            (*(*pmv).ps).gravity = ((*(*pmv).ps).gravity as f32 * 0.1) as c_int;
        } else {
            (*(*pmv).ps).gravity = ((*(*pmv).ps).gravity as f32 * 0.25) as c_int;
        }
    } else if *addr_of!(gPMDoSlowFall) != QFALSE {
        savedGravity = (*(*pmv).ps).gravity;
        // C: `pm->ps->gravity *= 0.5;` — `0.5` is a double, so the int gravity promotes to
        // f64 before truncating back (the bg_pmove.rs f64-promote-where-C-does convention).
        (*(*pmv).ps).gravity = ((*(*pmv).ps).gravity as f64 * 0.5) as c_int;
    }

    //if we're in jetpack mode then see if we should be jetting around
    if (*(*pmv).ps).pm_type == PM_JETPACK {
        if (*pmv).cmd.rightmove > 0 {
            PM_ContinueLegsAnim(BOTH_INAIRRIGHT1);
        } else if (*pmv).cmd.rightmove < 0 {
            PM_ContinueLegsAnim(BOTH_INAIRLEFT1);
        } else if (*pmv).cmd.forwardmove > 0 {
            PM_ContinueLegsAnim(BOTH_INAIR1);
        } else if (*pmv).cmd.forwardmove < 0 {
            PM_ContinueLegsAnim(BOTH_INAIRBACK1);
        } else {
            PM_ContinueLegsAnim(BOTH_INAIR1);
        }

        if (*(*pmv).ps).weapon == WP_SABER && BG_SpinningSaberAnim((*(*pmv).ps).legsAnim) != QFALSE
        {
            //make him stir around since he shouldn't have any real control when spinning
            (*(*pmv).ps).velocity[0] += Q_irand(-100, 100) as f32;
            (*(*pmv).ps).velocity[1] += Q_irand(-100, 100) as f32;
        }

        if (*pmv).cmd.upmove > 0 && (*(*pmv).ps).velocity[2] < 256.0 {
            //cap upward velocity off at 256. Seems reasonable.
            let mut addIn: f32 = 12.0;

            // (the commented-out distance-scaled addIn loop in C is dropped.)
            if (*(*pmv).ps).velocity[2] > 0.0 {
                addIn = 12.0 - (gDist / 64.0);
            }

            if addIn > 0.0 {
                (*(*pmv).ps).velocity[2] += addIn;
            }

            (*(*pmv).ps).eFlags |= EF_JETPACK_FLAMING; //going up
        } else {
            (*(*pmv).ps).eFlags &= !EF_JETPACK_FLAMING; //idling

            if (*(*pmv).ps).velocity[2] < 256.0 {
                if (*(*pmv).ps).velocity[2] < -100.0 {
                    (*(*pmv).ps).velocity[2] = -100.0;
                }
                if gDist < JETPACK_HOVER_HEIGHT as f32 {
                    //make sure we're always hovering off the ground somewhat while jetpack is active
                    (*(*pmv).ps).velocity[2] += 2.0;
                }
            }
        }
    }

    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
        && !(*addr_of!(pm_entSelf)).is_null()
        && !(*(*addr_of!(pm_entSelf))).m_pVehicle.is_null()
    {
        //Now update our mins/maxs to match our m_vOrientation based on our length, width & height
        BG_VehicleAdjustBBoxForOrientation(
            (*(*addr_of!(pm_entSelf))).m_pVehicle,
            addr_of_mut!((*(*pmv).ps).origin),
            addr_of_mut!((*pmv).mins),
            addr_of_mut!((*pmv).maxs),
            (*(*pmv).ps).clientNum,
            (*pmv).tracemask,
            (*pmv).trace,
        );
    }

    // set groundentity
    PM_GroundTrace();
    if *addr_of!(pm_flying) == FLY_HOVER {
        //never stick to the ground
        PM_HoverTrace();
    }

    if (*(*pmv).ps).groundEntityNum != ENTITYNUM_NONE {
        //on ground
        (*(*pmv).ps).fd.forceJumpZStart = 0.0;
    }

    if (*(*pmv).ps).pm_type == PM_DEAD {
        if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
            && !(*addr_of!(pm_entSelf)).is_null()
            && (*(*addr_of!(pm_entSelf))).s.NPC_class == CLASS_VEHICLE
            && (*(*(*(*addr_of!(pm_entSelf))).m_pVehicle).m_pVehicleInfo).r#type != VH_ANIMAL
        {
            //vehicles don't use deadmove
        } else {
            PM_DeadMove();
        }
    }

    PM_DropTimers();

    // (the #ifndef QAGAME _TESTING_VEH_PREDICTION CG_TestLine debug-draw block is cgame-only,
    //  dropped.)

    if (*(*addr_of!(pm_entSelf))).s.NPC_class != CLASS_VEHICLE && (*(*pmv).ps).m_iVehicleNum != 0 {
        //a player riding a vehicle
        let veh: *mut bgEntity_t = *addr_of!(pm_entVeh);

        if !veh.is_null()
            && !(*veh).m_pVehicle.is_null()
            && ((*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER
                || (*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER)
        {
            //*sigh*, until we get forced weapon-switching working?
            (*pmv).cmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
            (*(*pmv).ps).eFlags &= !(EF_FIRING | EF_ALT_FIRING);
            //pm->cmd.weapon = pm->ps->weapon;
        }
    }

    if (*(*pmv).ps).m_iVehicleNum == 0
        && (*(*addr_of!(pm_entSelf))).s.NPC_class != CLASS_VEHICLE
        && (*(*addr_of!(pm_entSelf))).s.NPC_class != CLASS_RANCOR
        && (*(*pmv).ps).groundEntityNum < ENTITYNUM_WORLD
        && (*(*pmv).ps).groundEntityNum >= MAX_CLIENTS as c_int
    {
        //I am a player client, not riding on a vehicle, and potentially standing on an NPC
        let pEnt: *mut bgEntity_t = PM_BGEntForNum((*(*pmv).ps).groundEntityNum);

        if !pEnt.is_null()
            && (*pEnt).s.eType == ET_NPC
            && (*pEnt).s.NPC_class != CLASS_VEHICLE
        {
            //don't bounce on vehicles
            //this is actually an NPC, let's try to bounce of its head to make sure we can't just stand around on top of it.
            if (*(*pmv).ps).velocity[2] < 270.0 {
                //try forcing velocity up and also force him to jump
                (*(*pmv).ps).velocity[2] = 270.0; //seems reasonable
                (*pmv).cmd.upmove = 127;
            }
        }
        //#ifdef QAGAME
        else if (*(*pmv).ps).zoomMode == 0
            && !(*addr_of!(pm_entSelf)).is_null() //I exist
            && !(*pEnt).m_pVehicle.is_null()
        {
            //ent has a vehicle
            let gEnt: *mut gentity_t = pEnt as *mut gentity_t;
            if !(*gEnt).client.is_null()
                && (*(*gEnt).client).ps.m_iVehicleNum == 0 //vehicle is empty
                && (*gEnt).spawnflags & 2 != 0
            {
                //SUSPENDED
                //it's a vehicle, see if we should get in it
                //if land on an empty, suspended vehicle, get in it
                ((*(*(*pEnt).m_pVehicle).m_pVehicleInfo).Board.unwrap())(
                    (*pEnt).m_pVehicle,
                    *addr_of!(pm_entSelf) as *mut bgEntity_t,
                );
            }
        }
    }

    if (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
        && !(*addr_of!(pm_entSelf)).is_null()
        && (*(*addr_of!(pm_entSelf))).s.NPC_class == CLASS_VEHICLE
    {
        //we are a vehicle
        let veh: *mut bgEntity_t = *addr_of!(pm_entSelf);

        debug_assert!(
            !veh.is_null()
                && !(*veh).playerState.is_null()
                && !(*veh).m_pVehicle.is_null()
                && (*veh).s.number >= MAX_CLIENTS as c_int
        );

        if (*(*(*veh).m_pVehicle).m_pVehicleInfo).r#type != VH_FIGHTER {
            //kind of hacky, don't want to do this for flying vehicles
            *(*(*veh).m_pVehicle).m_vOrientation.add(PITCH as usize) =
                (*(*pmv).ps).viewangles[PITCH as usize];
        }

        if (*(*pmv).ps).m_iVehicleNum == 0 {
            //no one is driving, just update and get out
            //#ifdef QAGAME
            ((*(*(*veh).m_pVehicle).m_pVehicleInfo).Update.unwrap())(
                (*veh).m_pVehicle,
                addr_of!((*pmv).cmd),
            );
            ((*(*(*veh).m_pVehicle).m_pVehicleInfo).Animate.unwrap())((*veh).m_pVehicle);
        } else {
            let self_ent: *mut bgEntity_t = *addr_of!(pm_entVeh);
            //#ifdef QAGAME
            let mut i: c_int = 0;

            debug_assert!(
                !self_ent.is_null()
                    && !(*self_ent).playerState.is_null()
                    && (*self_ent).s.number < MAX_CLIENTS as c_int
            );

            if (*(*pmv).ps).pm_type == PM_DEAD
                && (*(*veh).m_pVehicle).m_ulFlags & VEH_CRASHING as c_ulong != 0
            {
                (*(*veh).m_pVehicle).m_ulFlags &= !(VEH_CRASHING as c_ulong);
            }

            if (*(*self_ent).playerState).m_iVehicleNum != 0 {
                //only do it if they still have a vehicle (didn't get ejected this update or something)
                PM_VehicleViewAngles(
                    (*self_ent).playerState,
                    veh,
                    addr_of_mut!((*(*veh).m_pVehicle).m_ucmd),
                );
            }

            //#ifdef QAGAME
            ((*(*(*veh).m_pVehicle).m_pVehicleInfo).Update.unwrap())(
                (*veh).m_pVehicle,
                addr_of!((*(*veh).m_pVehicle).m_ucmd),
            );
            ((*(*(*veh).m_pVehicle).m_pVehicleInfo).Animate.unwrap())((*veh).m_pVehicle);

            ((*(*(*veh).m_pVehicle).m_pVehicleInfo).UpdateRider.unwrap())(
                (*veh).m_pVehicle,
                self_ent,
                addr_of_mut!((*(*veh).m_pVehicle).m_ucmd),
            );
            //update the passengers
            while i < (*(*veh).m_pVehicle).m_iNumPassengers {
                if !(*(*veh).m_pVehicle).m_ppPassengers[i as usize].is_null() {
                    let thePassenger: *mut gentity_t =
                        (*(*veh).m_pVehicle).m_ppPassengers[i as usize] as *mut gentity_t; //yes, this is, in fact, ass.
                    if (*thePassenger).inuse != QFALSE && !(*thePassenger).client.is_null() {
                        ((*(*(*veh).m_pVehicle).m_pVehicleInfo).UpdateRider.unwrap())(
                            (*veh).m_pVehicle,
                            (*(*veh).m_pVehicle).m_ppPassengers[i as usize],
                            addr_of_mut!((*(*thePassenger).client).pers.cmd),
                        );
                    }
                }
                i += 1;
            }
            // (the #else cgame branch — BG_FighterUpdate / ProcessOrient+MoveCommands /
            //  PM_SetPMViewAngle view handling — is dropped in this QAGAME build.)
        }
        noAnimate = QTRUE;
    }

    if (*(*addr_of!(pm_entSelf))).s.NPC_class != CLASS_VEHICLE && (*(*pmv).ps).m_iVehicleNum != 0 {
        //don't even run physics on a player if he's on a vehicle - he goes where the vehicle goes
    } else {
        //don't even run physics on a player if he's on a vehicle - he goes where the vehicle goes
        if (*(*pmv).ps).pm_type == PM_FLOAT || *addr_of!(pm_flying) == FLY_NORMAL {
            PM_FlyMove();
        } else if *addr_of!(pm_flying) == FLY_VEHICLE {
            PM_FlyVehicleMove();
        } else if (*(*pmv).ps).pm_flags & PMF_TIME_WATERJUMP != 0 {
            PM_WaterJumpMove();
        } else if (*pmv).waterlevel > 1 {
            // swimming
            PM_WaterMove();
        } else if (*addr_of!(pml)).walking != QFALSE {
            // walking on ground
            PM_WalkMove();
        } else {
            // airborne
            PM_AirMove();
        }
    }

    if noAnimate == QFALSE {
        PM_Animate();
    }

    // set groundentity, watertype, and waterlevel
    PM_GroundTrace();
    if *addr_of!(pm_flying) == FLY_HOVER {
        //never stick to the ground
        PM_HoverTrace();
    }
    PM_SetWaterLevel();
    // C: `pm->cmd.forcesel != -1`. `forcesel`/`invensel` are `byte` (unsigned), so the
    // promoted compare is always true and the "none" sentinel (-1 → 255) reaches the
    // `1 << forcesel` shift — UB in C, but on the x86 target the hardware `shl` masks the
    // count to 5 bits (`1 << (255 & 31)` == `1 << 31`), which `forcePowersKnown` never sets,
    // so the branch is skipped. `wrapping_shl` reproduces that masked shift exactly (a bare
    // `1 << 255u8` would instead panic in a debug build). Carried faithfully — see DEVIATIONS.
    if (*pmv).cmd.forcesel as c_int != -1
        && (*(*pmv).ps).fd.forcePowersKnown & 1i32.wrapping_shl((*pmv).cmd.forcesel as u32) != 0
    {
        (*(*pmv).ps).fd.forcePowerSelected = (*pmv).cmd.forcesel as c_int;
    }
    if (*pmv).cmd.invensel as c_int != -1
        && (*(*pmv).ps).stats[STAT_HOLDABLE_ITEMS as usize]
            & 1i32.wrapping_shl((*pmv).cmd.invensel as u32)
            != 0
    {
        (*(*pmv).ps).stats[STAT_HOLDABLE_ITEM as usize] =
            BG_GetItemIndexByTag((*pmv).cmd.invensel as c_int, IT_HOLDABLE);
    }

    if (*(*pmv).ps).m_iVehicleNum != 0
        /*&& pm_entSelf->s.NPC_class != CLASS_VEHICLE*/
        && (*(*pmv).ps).clientNum < MAX_CLIENTS as c_int
    {
        //a client riding a vehicle
        if (*(*pmv).ps).eFlags & EF_NODRAW != 0 {
            //inside the vehicle, do nothing
        } else if PM_WeaponOkOnVehicle((*pmv).cmd.weapon as c_int) == QFALSE
            || PM_WeaponOkOnVehicle((*(*pmv).ps).weapon) == QFALSE
        {
            //this weapon is not legal for the vehicle, force to our current one
            if PM_WeaponOkOnVehicle((*(*pmv).ps).weapon) == QFALSE {
                //uh-oh!
                let weap: c_int = PM_GetOkWeaponForVehicle();

                if weap != -1 {
                    (*pmv).cmd.weapon = weap as u8;
                    (*(*pmv).ps).weapon = weap;
                }
            } else {
                (*pmv).cmd.weapon = (*(*pmv).ps).weapon as u8;
            }
        }
    }

    if (*(*pmv).ps).m_iVehicleNum == 0 //not a vehicle and not riding one
        || (*(*addr_of!(pm_entSelf))).s.NPC_class == CLASS_VEHICLE //you are a vehicle NPC
        || ((*(*pmv).ps).eFlags & EF_NODRAW == 0 && PM_WeaponOkOnVehicle((*pmv).cmd.weapon as c_int) != QFALSE)
    {
        //you're not inside the vehicle and the weapon you're holding can be used when riding this vehicle
        //only run weapons if a valid weapon is selected
        // weapons
        PM_Weapon();
    }

    PM_Use();

    if (*(*pmv).ps).m_iVehicleNum == 0
        && ((*(*pmv).ps).clientNum < MAX_CLIENTS as c_int
            || (*addr_of!(pm_entSelf)).is_null()
            || (*(*addr_of!(pm_entSelf))).s.NPC_class != CLASS_VEHICLE)
    {
        //don't do this if we're on a vehicle, or we are one
        // footstep events / legs animations
        PM_Footsteps();
    }

    // entering / leaving water splashes
    PM_WaterEvents();

    // snap some parts of playerstate to save network bandwidth
    trap::SnapVector(&mut (*(*pmv).ps).velocity);

    if (*(*pmv).ps).pm_type == PM_JETPACK || *addr_of!(gPMDoSlowFall) != QFALSE {
        (*(*pmv).ps).gravity = savedGravity;
    }

    if //pm->ps->m_iVehicleNum &&
        (*(*pmv).ps).clientNum >= MAX_CLIENTS as c_int
        && !(*addr_of!(pm_entSelf)).is_null()
        && (*(*addr_of!(pm_entSelf))).s.NPC_class == CLASS_VEHICLE
    {
        //a vehicle with passengers
        let veh: *mut bgEntity_t = *addr_of!(pm_entSelf);

        debug_assert!(!(*veh).m_pVehicle.is_null());

        //this could be kind of "inefficient" because it's called after every passenger pmove too.
        //Maybe instead of AttachRiders we should have each rider call attach for himself?
        if !(*veh).m_pVehicle.is_null() && !(*veh).ghoul2.is_null() {
            ((*(*(*veh).m_pVehicle).m_pVehicleInfo).AttachRiders.unwrap())((*veh).m_pVehicle);
        }
    }

    if (*(*addr_of!(pm_entSelf))).s.NPC_class != CLASS_VEHICLE && (*(*pmv).ps).m_iVehicleNum != 0 {
        //riding a vehicle, see if we should do some anim overrides
        PM_VehicleWeaponAnimate();
    }
}

/*
================
Pmove

Can be called by either the server or the client
================
*/
/// `Pmove` (bg_pmove.c:10830) — the public entry: clamp the command time, optionally zero
/// the move while falling to death, bump the wrapped `pmove_framecount`, then chop the move
/// into ≤66 ms (or `pmove_msec` when `pmove_fixed`) sub-steps and run [`PmoveSingle`] for
/// each, re-asserting a held jump between steps. `static`→`pub`, no oracle (drives the
/// keystone).
#[allow(non_snake_case)]
pub unsafe fn Pmove(pmove: *mut pmove_t) {
    let finalTime: c_int = (*pmove).cmd.serverTime;

    if finalTime < (*(*pmove).ps).commandTime {
        return; // should not happen
    }

    if finalTime > (*(*pmove).ps).commandTime + 1000 {
        (*(*pmove).ps).commandTime = finalTime - 1000;
    }

    if (*(*pmove).ps).fallingToDeath != 0 {
        (*pmove).cmd.forwardmove = 0;
        (*pmove).cmd.rightmove = 0;
        (*pmove).cmd.upmove = 0;
        (*pmove).cmd.buttons = 0;
    }

    (*(*pmove).ps).pmove_framecount =
        ((*(*pmove).ps).pmove_framecount + 1) & ((1 << PS_PMOVEFRAMECOUNTBITS) - 1);

    // chop the move up if it is too long, to prevent framerate
    // dependent behavior
    while (*(*pmove).ps).commandTime != finalTime {
        let mut msec: c_int = finalTime - (*(*pmove).ps).commandTime;

        if (*pmove).pmove_fixed != 0 {
            if msec > (*pmove).pmove_msec as c_int {
                msec = (*pmove).pmove_msec as c_int;
            }
        } else if msec > 66 {
            msec = 66;
        }
        (*pmove).cmd.serverTime = (*(*pmove).ps).commandTime + msec;

        PmoveSingle(pmove);

        if (*(*pmove).ps).pm_flags & PMF_JUMP_HELD != 0 {
            (*pmove).cmd.upmove = 20;
        }
    }
}

/// Serializes the tests that drive the file-scope `pm`/`pml` globals — `cargo test`
/// otherwise runs them on parallel threads, and they would clobber each other's `pm`
/// pointer. Mirrors `rand_lock`/`parse_lock`; poison ignored so one failure does not
/// cascade.
/// Also serializes the bg_panimate setter tests, which share the `pm` / `g_entities`
/// globals with the keystone tests here.
#[cfg(all(test, feature = "oracle"))]
pub(crate) fn pm_lock() -> std::sync::MutexGuard<'static, ()> {
    static PM_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    PM_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;

    /// Value parity: every `pm_*` movement parameter and every element of the five
    /// force tables matches the authentic C data (independently transcribed in the
    /// oracle TU). All values are exactly representable in `f32`/`i32`, so `==` is
    /// exact. The scalar `pm_*` floats are compared in declaration order against the
    /// `jka_pm_params()` array (same order documented on both sides).
    #[test]
    fn bg_pmove_data_matches_c() {
        unsafe {
            // pm_* movement params, in bg_pmove.c declaration order (lines 40-54).
            let rust_params: [f32; 13] = [
                pm_stopspeed,
                pm_duckScale,
                pm_swimScale,
                pm_wadeScale,
                pm_vehicleaccelerate,
                pm_accelerate,
                pm_airaccelerate,
                pm_wateraccelerate,
                pm_flyaccelerate,
                pm_friction,
                pm_waterfriction,
                pm_flightfriction,
                pm_spectatorfriction,
            ];
            let c_params = core::slice::from_raw_parts(jka_pm_params(), 13);
            assert_eq!(&rust_params[..], c_params, "pm_* movement params");

            let c_speed = core::slice::from_raw_parts(jka_pm_forceSpeedLevels(), 4);
            assert_eq!(&forceSpeedLevels[..], c_speed, "forceSpeedLevels");

            // forcePowerNeeded is [4][18]; compare flattened (row-major, matching C).
            let c_fpn = core::slice::from_raw_parts(
                jka_pm_forcePowerNeeded(),
                NUM_FORCE_POWER_LEVELS * NUM_FORCE_POWERS,
            );
            let rust_fpn: &[c_int] = core::slice::from_raw_parts(
                forcePowerNeeded.as_ptr() as *const c_int,
                NUM_FORCE_POWER_LEVELS * NUM_FORCE_POWERS,
            );
            assert_eq!(rust_fpn, c_fpn, "forcePowerNeeded");

            let c_fjh = core::slice::from_raw_parts(jka_pm_forceJumpHeight(), NUM_FORCE_POWER_LEVELS);
            assert_eq!(&forceJumpHeight[..], c_fjh, "forceJumpHeight");

            let c_fjs =
                core::slice::from_raw_parts(jka_pm_forceJumpStrength(), NUM_FORCE_POWER_LEVELS);
            assert_eq!(&forceJumpStrength[..], c_fjs, "forceJumpStrength");

            let c_fjhm =
                core::slice::from_raw_parts(jka_pm_forceJumpHeightMax(), NUM_FORCE_POWER_LEVELS);
            assert_eq!(&forceJumpHeightMax[..], c_fjhm, "forceJumpHeightMax");
        }
    }

    /// Logic parity for `PM_AddTouchEnt`: point the file-scope `pm` at a zeroed
    /// `pmove_t`, drive one call, and compare the resulting `(numtouch, touchents)`
    /// against the verbatim C body. Covers all four control paths — the world-entity
    /// skip, the full-list skip, the already-present dedup, and a normal add. Takes
    /// `pm_lock()` since it shares the `pm` global with the other keystone tests.
    #[test]
    fn pm_addtouchent_matches_oracle() {
        use core::ptr::addr_of_mut;
        let _g = pm_lock();

        // (initial numtouch, initial touchents prefix, entityNum to add)
        let scenarios: &[(c_int, &[c_int], c_int)] = &[
            (0, &[], 5),                   // add to empty list
            (3, &[5, 6, 7], 9),            // add a distinct ent
            (3, &[5, 6, 7], 6),            // already present -> no-op
            (1, &[42], 42),                // already present (single) -> no-op
            (2, &[1, 2], ENTITYNUM_WORLD), // world ent -> no-op
            (MAXTOUCH as c_int, &[], 99),  // full list -> no-op
        ];

        for &(numtouch, prefix, ent) in scenarios {
            let mut touchents = [0 as c_int; MAXTOUCH];
            for (i, &v) in prefix.iter().enumerate() {
                touchents[i] = v;
            }

            // Rust: drive PM_AddTouchEnt over a zeroed pmove_t pointed to by `pm`.
            let mut pmv: pmove_t = unsafe { core::mem::zeroed() };
            pmv.numtouch = numtouch;
            pmv.touchents = touchents;
            unsafe {
                *addr_of_mut!(pm) = &mut pmv;
                PM_AddTouchEnt(ent);
                *addr_of_mut!(pm) = null_mut(); // leave the global clean
            }

            // C oracle: identical starting state through the verbatim body.
            let mut c_numtouch = numtouch;
            let mut c_touchents = touchents;
            unsafe {
                jka_pm_addtouchent(ent, &mut c_numtouch, c_touchents.as_mut_ptr());
            }

            assert_eq!(pmv.numtouch, c_numtouch, "numtouch (ent={ent}, n={numtouch})");
            assert_eq!(pmv.touchents, c_touchents, "touchents (ent={ent}, n={numtouch})");
        }
    }

    /// Logic parity for `PM_BGEntForNum`: the happy-path pointer arithmetic
    /// `(byte*)pm->baseEnt + pm->entSize*num`. Drives several `(base, entSize, num)`
    /// triples (the NULL/zero-stride guard paths can't be exercised against the C
    /// oracle without tripping its `assert`s, so only valid inputs are swept). The base
    /// address is a fake non-NULL value — the arithmetic never dereferences it.
    #[test]
    fn pm_bgentfornum_matches_oracle() {
        use core::ptr::addr_of_mut;
        let _g = pm_lock();

        // (base address, entSize, num)
        let cases: &[(usize, c_int, c_int)] = &[
            (0x10_0000, 1800, 0),    // gentity_t stride, first
            (0x10_0000, 1800, 1),    // second
            (0x10_0000, 1800, 100),  // mid-array
            (0x10_0000, 1800, 1023), // last valid index (MAX_GENTITIES-1)
            (0x4000, 944, 7),        // centity_t-ish stride
            (0x1, 1, 0),             // minimal stride
        ];

        for &(base, ent_size, num) in cases {
            let mut pmv: pmove_t = unsafe { core::mem::zeroed() };
            pmv.baseEnt = base as *mut bgEntity_t;
            pmv.entSize = ent_size;

            let rust = unsafe {
                *addr_of_mut!(pm) = &mut pmv;
                let r = PM_BGEntForNum(num) as usize;
                *addr_of_mut!(pm) = null_mut();
                r
            };
            let c = unsafe {
                jka_PM_BGEntForNum(base as core::ffi::c_ulong, ent_size, num) as usize
            };
            assert_eq!(rust, c, "base={base:#x} entSize={ent_size} num={num}");
        }
    }

    /// Logic parity for `BG_SabersOff` (param-only, no `pm` global): sweep
    /// `saberHolstered` 0..=3 × `saberAnimLevelBase` -1..=8 (covering SS_NONE..SS_STAFF
    /// and out-of-range) against the verbatim C body.
    #[test]
    fn bg_sabersoff_matches_oracle() {
        for holstered in 0..=3 {
            for base in -1..=8 {
                let r = unsafe {
                    let mut ps: playerState_t = core::mem::zeroed();
                    ps.saberHolstered = holstered;
                    ps.fd.saberAnimLevelBase = base;
                    BG_SabersOff(&mut ps)
                };
                let c = unsafe { jka_BG_SabersOff(holstered, base) };
                assert_eq!(r, c, "holstered={holstered} base={base}");
            }
        }
    }

    /// Logic parity for `BG_KnockDownable`: the NULL guard plus the vehicle / emplaced
    /// immunity checks. Sweeps is_null × m_iVehicleNum {0,1} × emplacedIndex {0,1}.
    #[test]
    fn bg_knockdownable_matches_oracle() {
        for is_null in [false, true] {
            for veh in 0..=1 {
                for emp in 0..=1 {
                    let r = unsafe {
                        if is_null {
                            BG_KnockDownable(null_mut())
                        } else {
                            let mut ps: playerState_t = core::mem::zeroed();
                            ps.m_iVehicleNum = veh;
                            ps.emplacedIndex = emp;
                            BG_KnockDownable(&mut ps)
                        }
                    };
                    let c = unsafe { jka_BG_KnockDownable(is_null as c_int, veh, emp) };
                    assert_eq!(r, c, "is_null={is_null} veh={veh} emp={emp}");
                }
            }
        }
    }

    /// Logic parity for `PM_GetSaberStance` (reads the `pm` global + calls
    /// `BG_SabersOff`): sweep `saberEntityNum` {0,1} × `saberHolstered` {0,1,2} ×
    /// `saberAnimLevelBase` {0,6,7} × `saberAnimLevel` -1..=8 (covering every
    /// switch case incl. SS_DUAL/SS_STAFF, FORCE_LEVEL_0..5 and the default).
    #[test]
    fn pm_getsaberstance_matches_oracle() {
        let _g = pm_lock();
        use core::ptr::addr_of_mut;

        // PC PM_GetSaberStance calls BG_MySaber(clientNum,{0,1}) which reads g_entities;
        // point it at a zeroed array so both sabers resolve to NULL (matching the oracle's
        // null BG_MySaber stub) instead of dereferencing the null global.
        let mut gents: Vec<gentity_t> = (0..64)
            .map(|_| unsafe { core::mem::MaybeUninit::zeroed().assume_init() })
            .collect();
        unsafe {
            core::ptr::copy_nonoverlapping(gents.as_mut_ptr(), core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), gents.len());
        }

        for entnum in 0..=1 {
            for holstered in 0..=2 {
                for base in [0, 6, 7] {
                    for level in -1..=8 {
                        let rust = unsafe {
                            let mut ps: playerState_t = core::mem::zeroed();
                            ps.saberEntityNum = entnum;
                            ps.saberHolstered = holstered;
                            ps.fd.saberAnimLevelBase = base;
                            ps.fd.saberAnimLevel = level;
                            let mut pmv: pmove_t = core::mem::zeroed();
                            pmv.ps = &mut ps;
                            *addr_of_mut!(pm) = &mut pmv;
                            let r = PM_GetSaberStance();
                            *addr_of_mut!(pm) = null_mut();
                            r
                        };
                        let c = unsafe { jka_PM_GetSaberStance(entnum, holstered, base, level) };
                        assert_eq!(
                            rust, c,
                            "entnum={entnum} holstered={holstered} base={base} level={level}"
                        );
                    }
                }
            }
        }

        unsafe {
            core::ptr::write_bytes(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>(), 0, gents.len());
        }
    }

    /// Logic parity for `PM_DoSlowFall` (reads the `pm` global): sweep `legsAnim` over
    /// the two wall-run anims plus a control value × `legsTimer` across the 500
    /// threshold.
    #[test]
    fn pm_doslowfall_matches_oracle() {
        let _g = pm_lock();
        use core::ptr::addr_of_mut;

        let anims = [BOTH_WALL_RUN_RIGHT, BOTH_WALL_RUN_LEFT, BOTH_STAND1];
        let timers = [-1, 0, 499, 500, 501, 1000];
        for &a in &anims {
            for &t in &timers {
                let rust = unsafe {
                    let mut ps: playerState_t = core::mem::zeroed();
                    ps.legsAnim = a;
                    ps.legsTimer = t;
                    let mut pmv: pmove_t = core::mem::zeroed();
                    pmv.ps = &mut ps;
                    *addr_of_mut!(pm) = &mut pmv;
                    let r = PM_DoSlowFall();
                    *addr_of_mut!(pm) = null_mut();
                    r
                };
                let c = unsafe { jka_PM_DoSlowFall(a, t) };
                assert_eq!(rust, c, "legsAnim={a} legsTimer={t}");
            }
        }
    }

    /// `PM_IsRocketTrooper` is `#if 0`'d in JKA — it unconditionally returns `qfalse`.
    /// No C oracle needed for a constant; assert the value directly.
    #[test]
    fn pm_isrockettrooper_is_false() {
        assert_eq!(PM_IsRocketTrooper(), QFALSE);
    }

    /// Bit-exact parity for `PM_ClipVelocity`: sweep a set of `in`/`normal` vectors ×
    /// overbounce × the stuck-to-wall flag × stepSlideFix × clientNum × groundEntityNum,
    /// covering the early-out, the negative/positive overbounce split, and the steep-slope
    /// clamp.
    #[test]
    fn pm_clipvelocity_matches_oracle() {
        let _g = pm_lock();
        let scenarios: [([f32; 3], [f32; 3]); 6] = [
            ([100.0, 50.0, -200.0], [0.0, 0.0, 1.0]),
            ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            ([-30.0, 40.0, 10.0], [0.6, 0.0, 0.5]),
            ([300.0, -100.0, -50.0], [0.5, 0.5, 0.7071]),
            ([12.5, -7.25, 3.0], [-0.3, 0.4, 0.866]),
            ([1.0, 2.0, 3.0], [0.0, 0.0, -1.0]),
        ];
        for (inv, normal) in scenarios.iter() {
            for &ob in &[1.0f32, 1.001, 2.0] {
                for &flags in &[0, PMF_STUCK_TO_WALL] {
                    for &ssf in &[0, 1] {
                        for &cn in &[0, 50] {
                            for &gen in &[ENTITYNUM_NONE, 5] {
                                let mut rout = [0.0f32; 3];
                                unsafe {
                                    let mut ps: playerState_t = core::mem::zeroed();
                                    ps.pm_flags = flags;
                                    ps.clientNum = cn;
                                    ps.groundEntityNum = gen;
                                    let mut pmv: pmove_t = core::mem::zeroed();
                                    pmv.ps = &mut ps;
                                    pmv.stepSlideFix = ssf;
                                    *addr_of_mut!(pm) = &mut pmv;
                                    let mut inv2 = *inv;
                                    let mut nrm2 = *normal;
                                    PM_ClipVelocity(
                                        inv2.as_mut_ptr(),
                                        nrm2.as_mut_ptr(),
                                        rout.as_mut_ptr(),
                                        ob,
                                    );
                                    *addr_of_mut!(pm) = null_mut();
                                }
                                let mut cout = [0.0f32; 3];
                                unsafe {
                                    jka_PM_ClipVelocity(
                                        inv.as_ptr(),
                                        normal.as_ptr(),
                                        ob,
                                        flags,
                                        ssf,
                                        cn,
                                        gen,
                                        cout.as_mut_ptr(),
                                    );
                                }
                                for k in 0..3 {
                                    assert_eq!(
                                        rout[k].to_bits(),
                                        cout[k].to_bits(),
                                        "clipvel out[{k}] in={inv:?} n={normal:?} ob={ob} flags={flags} ssf={ssf} cn={cn} gen={gen}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Bit-exact parity for `PM_CmdScale`: sweep forward/right move over the axial range
    /// (incl. the all-zero early-out and the ±127 extremes) × a few `ps->speed` values.
    #[test]
    fn pm_cmdscale_matches_oracle() {
        let _g = pm_lock();
        let moves = [-127i32, -100, -64, -1, 0, 1, 63, 100, 127];
        let speeds = [0.0f32, 50.0, 100.0, 250.0, 320.5];
        for &f in &moves {
            for &r in &moves {
                for &sp in &speeds {
                    let rust = unsafe {
                        let mut ps: playerState_t = core::mem::zeroed();
                        ps.speed = sp;
                        let mut pmv: pmove_t = core::mem::zeroed();
                        pmv.ps = &mut ps;
                        *addr_of_mut!(pm) = &mut pmv;
                        let mut cmd: usercmd_t = core::mem::zeroed();
                        cmd.forwardmove = f as i8;
                        cmd.rightmove = r as i8;
                        let v = PM_CmdScale(&mut cmd);
                        *addr_of_mut!(pm) = null_mut();
                        v
                    };
                    let c = unsafe { jka_PM_CmdScale(f, r, sp) };
                    assert_eq!(rust.to_bits(), c.to_bits(), "cmdscale f={f} r={r} sp={sp}");
                }
            }
        }
    }

    /// Bit-exact parity for `PM_Accelerate`: sweep velocity/wishdir/wishspeed/accel/
    /// frametime over both the standard ("bunnyhopping") branch and the siege push branch
    /// (selected by the gametype/vehicle/clientNum/pm_type mode tuples).
    #[test]
    fn pm_accelerate_matches_oracle() {
        use crate::codemp::game::bg_public::GT_FFA;
        let _g = pm_lock();
        let vels: [[f32; 3]; 3] = [
            [0.0, 0.0, 0.0],
            [120.0, -40.0, 10.0],
            [-200.0, 300.0, -50.0],
        ];
        let dirs: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.707, 0.707, 0.0]];
        // (gametype, m_iVehicleNum, clientNum, pm_type) — first hits the siege branch,
        // the rest each trip one of the standard-branch disjuncts.
        let modes = [
            (GT_SIEGE, 0, 0, PM_NORMAL),
            (GT_SIEGE, 1, 0, PM_NORMAL),
            (GT_SIEGE, 0, 50, PM_NORMAL),
            (GT_SIEGE, 0, 0, PM_SPECTATOR),
            (GT_FFA, 0, 0, PM_NORMAL),
        ];
        for vel in &vels {
            for dir in &dirs {
                for &ws in &[0.0f32, 100.0, 250.0] {
                    for &ac in &[1.0f32, 10.0, 36.0] {
                        for &ft in &[0.008f32, 0.05] {
                            for &(gt, vn, cn, pt) in &modes {
                                let rvel = unsafe {
                                    let mut ps: playerState_t = core::mem::zeroed();
                                    ps.velocity = *vel;
                                    ps.m_iVehicleNum = vn;
                                    ps.clientNum = cn;
                                    ps.pm_type = pt;
                                    let mut pmv: pmove_t = core::mem::zeroed();
                                    pmv.ps = &mut ps;
                                    pmv.gametype = gt;
                                    (*addr_of_mut!(pml)).frametime = ft;
                                    *addr_of_mut!(pm) = &mut pmv;
                                    let mut dir2 = *dir;
                                    PM_Accelerate(dir2.as_mut_ptr(), ws, ac);
                                    let out = ps.velocity;
                                    *addr_of_mut!(pm) = null_mut();
                                    out
                                };
                                let mut cvel = *vel;
                                unsafe {
                                    jka_PM_Accelerate(
                                        cvel.as_mut_ptr(),
                                        dir.as_ptr(),
                                        ws,
                                        ac,
                                        ft,
                                        gt,
                                        vn,
                                        cn,
                                        pt,
                                    );
                                }
                                for k in 0..3 {
                                    assert_eq!(
                                        rvel[k].to_bits(),
                                        cvel[k].to_bits(),
                                        "accel v[{k}] vel={vel:?} dir={dir:?} ws={ws} ac={ac} ft={ft} mode=({gt},{vn},{cn},{pt})"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Bit-exact parity for `PM_Friction`: a curated scenario list covering the speed<1
    /// early-out, player ground friction (slick/knockback gates), the vehicle-friction
    /// branch (clientNum≥MAX_CLIENTS + the CLASS_VEHICLE pointer chain), the FLY_VEHICLE
    /// branch, water friction, the on-a-client drop=0 case, and spectator/float friction.
    #[test]
    fn pm_friction_matches_oracle() {
        use crate::codemp::game::bg_vehicles_h::{vehicleInfo_t, VH_FIGHTER};
        let _g = pm_lock();

        // (vel, walking, pm_type, clientNum, pm_flags, groundEntityNum, waterlevel,
        //  surfaceFlags, frametime, pm_flying, hasEnt, NPC_class, hasVehicle, vehType,
        //  vehFriction)
        struct S {
            vel: [f32; 3],
            walking: c_int,
            pm_type: c_int,
            client_num: c_int,
            pm_flags: c_int,
            ground_ent: c_int,
            waterlevel: c_int,
            surface_flags: c_int,
            frametime: f32,
            flying: c_int,
            has_ent: bool,
            npc_class: c_int,
            has_vehicle: bool,
            veh_type: c_int,
            veh_friction: f32,
        }
        let s = |vel, walking, pm_type, client_num, pm_flags, ground_ent, waterlevel, surface_flags, frametime, flying, has_ent, npc_class, has_vehicle, veh_type, veh_friction| S {
            vel, walking, pm_type, client_num, pm_flags, ground_ent, waterlevel, surface_flags,
            frametime, flying, has_ent, npc_class, has_vehicle, veh_type, veh_friction,
        };
        let no_fly = FLY_NONE;
        let scenarios = [
            // speed < 1 early-out, spectator (zeros vel[2] too)
            s([0.5, -0.4, 0.3], 0, PM_SPECTATOR, 0, 0, ENTITYNUM_NONE, 0, 0, 0.05, no_fly, false, 0, false, 0, 0.0),
            // speed < 1 early-out, normal (leaves vel[2])
            s([0.2, 0.1, 5.0], 0, PM_NORMAL, 0, 0, ENTITYNUM_NONE, 0, 0, 0.05, no_fly, false, 0, false, 0, 0.0),
            // player ground friction: walking, not slick, no knockback
            s([300.0, -120.0, 0.0], 1, PM_NORMAL, 0, 0, ENTITYNUM_NONE, 0, 0, 0.008, no_fly, false, 0, false, 0, 0.0),
            // player ground friction: slick surface → no ground friction
            s([300.0, -120.0, 0.0], 1, PM_NORMAL, 0, 0, ENTITYNUM_NONE, 0, SURF_SLICK, 0.008, no_fly, false, 0, false, 0, 0.0),
            // player ground friction: knockback → no ground friction
            s([300.0, -120.0, 0.0], 1, PM_NORMAL, PMF_TIME_KNOCKBACK, 0, ENTITYNUM_NONE, 0, 0, 0.008, no_fly, false, 0, false, 0, 0.0),
            // slow speed (below stopspeed) hits the pm_stopspeed clamp
            s([40.0, 0.0, 0.0], 1, PM_NORMAL, 0, 0, ENTITYNUM_NONE, 0, 0, 0.05, no_fly, false, 0, false, 0, 0.0),
            // water friction (waterlevel 2)
            s([200.0, 80.0, -30.0], 0, PM_NORMAL, 0, 0, ENTITYNUM_NONE, 2, 0, 0.05, no_fly, false, 0, false, 0, 0.0),
            // groundEntityNum < MAX_CLIENTS → drop forced to 0
            s([200.0, 80.0, -30.0], 0, PM_NORMAL, 0, 5, ENTITYNUM_NONE, 0, 0, 0.05, FLY_NORMAL, false, 0, false, 0, 0.0),
            // spectator friction
            s([260.0, -90.0, 40.0], 0, PM_SPECTATOR, 0, 0, ENTITYNUM_NONE, 0, 0, 0.05, no_fly, false, 0, false, 0, 0.0),
            // float friction (the f64-promoted 0.1 path)
            s([260.0, -90.0, 40.0], 0, PM_FLOAT, 0, 0, ENTITYNUM_NONE, 0, 0, 0.05, no_fly, false, 0, false, 0, 0.0),
            // vehicle branch: clientNum ≥ MAX_CLIENTS, CLASS_VEHICLE, fighter, friction, no knockback
            s([280.0, 100.0, -20.0], 0, PM_NORMAL, 40, 0, ENTITYNUM_NONE, 0, 0, 0.008, no_fly, true, CLASS_VEHICLE, true, VH_FIGHTER, 5.0),
            // vehicle branch but knockback set → skip vehicle friction
            s([280.0, 100.0, -20.0], 0, PM_NORMAL, 40, PMF_TIME_KNOCKBACK, ENTITYNUM_NONE, 0, 0, 0.008, no_fly, true, CLASS_VEHICLE, true, VH_FIGHTER, 5.0),
            // vehicle branch but friction 0 → falls through to else-if
            s([280.0, 100.0, -20.0], 1, PM_NORMAL, 40, 0, ENTITYNUM_NONE, 0, 0, 0.008, no_fly, true, CLASS_VEHICLE, true, VH_FIGHTER, 0.0),
            // FLY_VEHICLE branch
            s([280.0, 100.0, -20.0], 0, PM_NORMAL, 40, 0, ENTITYNUM_NONE, 0, 0, 0.008, FLY_VEHICLE, false, 0, false, 0, 0.0),
        ];

        for (i, sc) in scenarios.iter().enumerate() {
            let rvel = unsafe {
                let mut ps: playerState_t = core::mem::zeroed();
                ps.velocity = sc.vel;
                ps.pm_type = sc.pm_type;
                ps.clientNum = sc.client_num;
                ps.pm_flags = sc.pm_flags;
                ps.groundEntityNum = sc.ground_ent;
                let mut pmv: pmove_t = core::mem::zeroed();
                pmv.ps = &mut ps;
                pmv.waterlevel = sc.waterlevel;
                (*addr_of_mut!(pml)).walking = sc.walking;
                (*addr_of_mut!(pml)).frametime = sc.frametime;
                (*addr_of_mut!(pml)).groundTrace.surfaceFlags = sc.surface_flags;
                *addr_of_mut!(pm_flying) = sc.flying;

                let mut vi: vehicleInfo_t = core::mem::zeroed();
                vi.r#type = sc.veh_type;
                vi.friction = sc.veh_friction;
                let mut veh: Vehicle_t = core::mem::zeroed();
                veh.m_pVehicleInfo = &mut vi;
                let mut ent: bgEntity_t = core::mem::zeroed();
                ent.s.NPC_class = sc.npc_class;
                ent.m_pVehicle = if sc.has_vehicle { &mut veh } else { null_mut() };
                *addr_of_mut!(pm_entSelf) = if sc.has_ent { &mut ent } else { null_mut() };

                *addr_of_mut!(pm) = &mut pmv;
                PM_Friction();
                let out = ps.velocity;
                *addr_of_mut!(pm) = null_mut();
                *addr_of_mut!(pm_entSelf) = null_mut();
                out
            };
            let mut cvel = sc.vel;
            unsafe {
                jka_PM_Friction(
                    cvel.as_mut_ptr(),
                    sc.walking,
                    sc.pm_type,
                    sc.client_num,
                    sc.pm_flags,
                    sc.ground_ent,
                    sc.waterlevel,
                    sc.surface_flags,
                    sc.frametime,
                    sc.flying,
                    sc.has_ent as c_int,
                    sc.npc_class,
                    sc.has_vehicle as c_int,
                    sc.veh_type,
                    sc.veh_friction,
                );
            }
            for k in 0..3 {
                assert_eq!(
                    rvel[k].to_bits(),
                    cvel[k].to_bits(),
                    "friction scenario {i} vel[{k}]"
                );
            }
        }
    }

    /// Parity for `PM_SetMovementDir`: the active branch is exhaustively driven by the
    /// sign of (forwardmove, rightmove); the idle branch (both zero) is driven by the
    /// prior `movementDir`. Sweep both: forward/right over {-1,0,1} (the only thing the
    /// classification keys on) × every starting `movementDir` in 0..8.
    #[test]
    fn pm_setmovementdir_matches_oracle() {
        let _g = pm_lock();
        for f in -1i32..=1 {
            for r in -1i32..=1 {
                for start in 0i32..8 {
                    let rust = unsafe {
                        let mut ps: playerState_t = core::mem::zeroed();
                        ps.movementDir = start;
                        let mut pmv: pmove_t = core::mem::zeroed();
                        pmv.ps = &mut ps;
                        pmv.cmd.forwardmove = f as i8;
                        pmv.cmd.rightmove = r as i8;
                        *addr_of_mut!(pm) = &mut pmv;
                        PM_SetMovementDir();
                        let out = ps.movementDir;
                        *addr_of_mut!(pm) = null_mut();
                        out
                    };
                    let c = unsafe { jka_PM_SetMovementDir(f, r, start) };
                    assert_eq!(rust, c, "setmovementdir f={f} r={r} start={start}");
                }
            }
        }
    }

    /// Bit-exact parity for `PM_SetPMViewAngle`: `delta_angles[i]` comes from the
    /// float→int `ANGLE2SHORT` truncation minus the command angle, and `viewangles` is a
    /// straight copy. Sweep angles spanning the wrap range and a few command angles.
    #[test]
    fn pm_setpmviewangle_matches_oracle() {
        let angles = [-540.0f32, -180.0, -0.5, 0.0, 45.3, 179.9, 360.0, 721.7];
        let cmds = [0i32, 1, -1, 16384, -32768, 32767];
        for &a0 in &angles {
            for &a1 in &angles {
                for &c0 in &cmds {
                    let angle: vec3_t = [a0, a1, a0 + a1];
                    let ucmd_angles: [c_int; 3] = [c0, c0 / 2, -c0];
                    let (r_delta, r_view) = unsafe {
                        let mut ps: playerState_t = core::mem::zeroed();
                        let mut ucmd: usercmd_t = core::mem::zeroed();
                        ucmd.angles = ucmd_angles;
                        let mut ang = angle;
                        PM_SetPMViewAngle(&mut ps, ang.as_mut_ptr(), &mut ucmd);
                        (ps.delta_angles, ps.viewangles)
                    };
                    let mut c_delta = [0i32; 3];
                    let mut c_view = [0.0f32; 3];
                    unsafe {
                        jka_PM_SetPMViewAngle(
                            angle.as_ptr(),
                            ucmd_angles.as_ptr(),
                            c_delta.as_mut_ptr(),
                            c_view.as_mut_ptr(),
                        );
                    }
                    assert_eq!(r_delta, c_delta, "viewangle delta a={angle:?} c0={c0}");
                    for k in 0..3 {
                        assert_eq!(
                            r_view[k].to_bits(),
                            c_view[k].to_bits(),
                            "viewangle view[{k}] a={angle:?}"
                        );
                    }
                }
            }
        }
    }

    /// `BG_ForceWallJumpStrength` parity: the single value `forceJumpStrength[3]/2.5`
    /// matches the verbatim C computation (over the already-verified force table).
    #[test]
    fn bg_force_walljump_strength_matches_oracle() {
        let got = BG_ForceWallJumpStrength();
        let want = unsafe { jka_BG_ForceWallJumpStrength() };
        assert_eq!(got.to_bits(), want.to_bits(), "BG_ForceWallJumpStrength");
    }

    /// `PM_SetForceJumpZStart` parity: sweep representative `value`s — crucially `0.0`
    /// (the `!value` nudge to `-0.1`, exercising the `-= 0.1` f64 promotion) plus
    /// nonzero positives/negatives that take the store-and-keep path. Drives the Rust
    /// port over a `pm`-pointed `playerState_t` and compares the resulting
    /// `fd.forceJumpZStart` bit-exact against the flattened C body.
    #[test]
    fn pm_setforcejumpzstart_matches_oracle() {
        use core::ptr::addr_of_mut;
        let _g = pm_lock();
        for &value in &[0.0f32, -0.0, 1.0, -1.0, 123.5, -987.25, 0.1] {
            let r = unsafe {
                let mut ps: playerState_t = core::mem::zeroed();
                let mut pmv: pmove_t = core::mem::zeroed();
                pmv.ps = &mut ps;
                *addr_of_mut!(pm) = &mut pmv;
                PM_SetForceJumpZStart(value);
                *addr_of_mut!(pm) = null_mut();
                ps.fd.forceJumpZStart
            };
            let c = unsafe { jka_PM_SetForceJumpZStart(value) };
            assert_eq!(r.to_bits(), c.to_bits(), "PM_SetForceJumpZStart value={value}");
        }
    }

    /// `PM_DeadMove` parity: sweep `pml.walking` × representative velocities — including
    /// the not-walking early-out, the `<=0` clear path (length ≤ 20), and the normalize+
    /// rescale path — and compare the resulting `ps->velocity` bit-exact against the
    /// verbatim C body (driven over the same `pm`/`pml` globals).
    #[test]
    fn pm_deadmove_matches_oracle() {
        use core::ptr::addr_of_mut;
        let _g = pm_lock();
        let vels: &[[f32; 3]] = &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [3.0, 4.0, 0.0],   // length 5 -> -15 -> clear
            [30.0, 40.0, 0.0], // length 50 -> 30
            [100.0, 0.0, -50.0],
            [-12.5, 7.25, 3.0],
            [0.5, 0.5, 0.5], // length < 20 -> clear
        ];
        for &walking in &[0i32, 1] {
            for &v in vels {
                let r = unsafe {
                    let mut ps: playerState_t = core::mem::zeroed();
                    let mut pmv: pmove_t = core::mem::zeroed();
                    ps.velocity = v;
                    pmv.ps = &mut ps;
                    *addr_of_mut!(pm) = &mut pmv;
                    (*addr_of_mut!(pml)).walking = walking;
                    PM_DeadMove();
                    *addr_of_mut!(pm) = null_mut();
                    (*addr_of_mut!(pml)).walking = 0;
                    ps.velocity
                };
                let mut c = v;
                unsafe {
                    jka_PM_DeadMove(walking, c.as_mut_ptr());
                }
                for k in 0..3 {
                    assert_eq!(
                        r[k].to_bits(),
                        c[k].to_bits(),
                        "PM_DeadMove walking={walking} v={v:?} k={k}"
                    );
                }
            }
        }
    }

    /// `PM_FootstepForSurface` parity: sweep `groundTrace.surfaceFlags` over the
    /// `SURF_NOSTEPS` short-circuit and a spread of material-mask values; compare against
    /// the verbatim C body (driven over the shared `pml` global).
    #[test]
    fn pm_footstepforsurface_matches_oracle() {
        use core::ptr::addr_of_mut;
        let _g = pm_lock();
        let flags: &[c_int] = &[
            0,
            SURF_NOSTEPS,
            SURF_NOSTEPS | 5,
            1,
            2,
            0x1f,
            0x20,
            0x3f,
            MATERIAL_MASK,
            0x0040_0005,
            -1,
        ];
        for &sf in flags {
            let r = unsafe {
                (*addr_of_mut!(pml)).groundTrace.surfaceFlags = sf;
                let got = PM_FootstepForSurface();
                (*addr_of_mut!(pml)).groundTrace.surfaceFlags = 0;
                got
            };
            let c = unsafe { jka_PM_FootstepForSurface(sf) };
            assert_eq!(r, c, "PM_FootstepForSurface sf={sf:#x}");
        }
    }

    /// `PM_WalkingAnim`/`PM_RunningAnim`/`PM_SwimmingAnim`/`PM_RollingAnim` parity: the
    /// four pure-switch anim classifiers each take a plain int, so sweep the full
    /// `animNumber_t` domain (`-8..=2100`, the `bg_panimate` range) against the verbatim
    /// C bodies — this transitively verifies every `case` constant on both sides.
    #[test]
    fn anim_classifiers_match_oracle() {
        for i in -8..=2100 {
            assert_eq!(PM_WalkingAnim(i), unsafe { jka_PM_WalkingAnim(i) }, "PM_WalkingAnim {i}");
            assert_eq!(PM_RunningAnim(i), unsafe { jka_PM_RunningAnim(i) }, "PM_RunningAnim {i}");
            assert_eq!(PM_SwimmingAnim(i), unsafe { jka_PM_SwimmingAnim(i) }, "PM_SwimmingAnim {i}");
            assert_eq!(PM_RollingAnim(i), unsafe { jka_PM_RollingAnim(i) }, "PM_RollingAnim {i}");
        }
    }

    /// `PM_AnglesForSlope` parity: sweep representative yaws × slope normals (axis-
    /// aligned, tilted, and arbitrary) and compare the resulting `angles` bit-exact
    /// against the verbatim C body. The oracle reuses the q_math C trig symbols, so any
    /// divergence is in the surrounding f32 pitch/mod/dot arithmetic.
    #[test]
    fn pm_anglesforslope_matches_oracle() {
        let yaws = [0.0f32, 45.0, 90.0, -135.0, 180.0, 270.0, 359.9];
        let slopes: &[[f32; 3]] = &[
            [0.0, 0.0, 1.0],   // flat
            [0.0, 0.0, -1.0],  // inverted
            [0.5, 0.0, 0.866], // tilt forward
            [0.0, 0.5, 0.866], // tilt right
            [0.3, -0.4, 0.866],
            [-0.7, 0.2, 0.68],
            [1.0, 0.0, 0.0], // vertical
        ];
        for &yaw in &yaws {
            for &slope in slopes {
                let mut r_ang: vec3_t = [0.0; 3];
                PM_AnglesForSlope(yaw, &slope, &mut r_ang);
                let mut c_ang = [0.0f32; 3];
                unsafe { jka_PM_AnglesForSlope(yaw, slope.as_ptr(), c_ang.as_mut_ptr()) };
                for k in 0..3 {
                    assert_eq!(
                        r_ang[k].to_bits(),
                        c_ang[k].to_bits(),
                        "PM_AnglesForSlope yaw={yaw} slope={slope:?} angles[{k}]"
                    );
                }
            }
        }
    }

    /// `BG_InSlopeAnim` parity: sweep the full `animNumber_t` domain (`-8..=2100`)
    /// against the verbatim C body, transitively verifying all 50 `LEGS_*` case
    /// constants (and the skipped `S2` series) on both sides.
    #[test]
    fn bg_inslopeanim_matches_oracle() {
        for i in -8..=2100 {
            assert_eq!(BG_InSlopeAnim(i), unsafe { jka_BG_InSlopeAnim(i) }, "BG_InSlopeAnim {i}");
        }
    }

    /// `PM_DropTimers` parity: sweep `pml.msec` × the four timing fields — covering the
    /// expire path (`msec >= pm_time`, clearing `PMF_ALL_TIMES`), the decrement path, and
    /// the legs/torso clamp-at-0 paths — and compare every output field against the
    /// verbatim C body (driven over the same `pm`/`pml` globals).
    #[test]
    fn pm_droptimers_matches_oracle() {
        use core::ptr::addr_of_mut;
        let _g = pm_lock();
        let mscs = [0i32, 1, 8, 50, 1000];
        let times = [0i32, 5, 50, 1000];
        let timers = [-10i32, 0, 1, 8, 200, 1000];
        // PMF_ALL_TIMES bits plus an unrelated bit to confirm only the timing bits clear.
        let flagsets = [0i32, PMF_ALL_TIMES, PMF_ALL_TIMES | PMF_DUCKED, PMF_DUCKED];
        for &msec in &mscs {
            for &pm_time in &times {
                for &legs in &timers {
                    for &torso in &timers {
                        for &flags in &flagsets {
                            let (r_t, r_f, r_l, r_to) = unsafe {
                                let mut ps: playerState_t = core::mem::zeroed();
                                let mut pmv: pmove_t = core::mem::zeroed();
                                ps.pm_time = pm_time;
                                ps.pm_flags = flags;
                                ps.legsTimer = legs;
                                ps.torsoTimer = torso;
                                pmv.ps = &mut ps;
                                *addr_of_mut!(pm) = &mut pmv;
                                (*addr_of_mut!(pml)).msec = msec;
                                PM_DropTimers();
                                *addr_of_mut!(pm) = null_mut();
                                (ps.pm_time, ps.pm_flags, ps.legsTimer, ps.torsoTimer)
                            };
                            let (mut c_t, mut c_f, mut c_l, mut c_to) =
                                (pm_time, flags, legs, torso);
                            unsafe {
                                jka_PM_DropTimers(
                                    msec, &mut c_t, &mut c_f, &mut c_l, &mut c_to,
                                );
                            }
                            assert_eq!(
                                (r_t, r_f, r_l, r_to),
                                (c_t, c_f, c_l, c_to),
                                "PM_DropTimers msec={msec} pm_time={pm_time} legs={legs} torso={torso} flags={flags:#x}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// `BG_InRollAnim`/`BG_InKnockDown`/`BG_InRollES` parity: each reduces to a switch on
    /// one int, so sweep the full `-8..=2100` domain against the verbatim C bodies,
    /// transitively verifying every roll/knockdown/getup case constant on both sides.
    #[test]
    fn roll_knockdown_predicates_match_oracle() {
        for i in -8..=2100 {
            let r_roll = unsafe {
                let mut es: entityState_t = core::mem::zeroed();
                es.legsAnim = i;
                BG_InRollAnim(&mut es)
            };
            assert_eq!(r_roll, unsafe { jka_BG_InRollAnim(i) }, "BG_InRollAnim {i}");
            assert_eq!(BG_InKnockDown(i), unsafe { jka_BG_InKnockDown(i) }, "BG_InKnockDown {i}");
            let r_es = unsafe {
                let mut es: entityState_t = core::mem::zeroed();
                BG_InRollES(&mut es, i)
            };
            assert_eq!(r_es, unsafe { jka_BG_InRollES(i) }, "BG_InRollES {i}");
        }
    }

    /// `BG_UpdateLookAngles` parity: sweep debounce-active vs -inactive × look angles that
    /// straddle the clamp limits × a couple of `lookSpeed`s, comparing both the mutated
    /// `lookAngles` and `lastHeadAngles` bit-exact against the verbatim C body (which
    /// reuses the q_math C trig). The C `static` scratch is write-before-read each call,
    /// so a single isolated call per input is a faithful comparison.
    #[test]
    fn bg_updatelookangles_matches_oracle() {
        let look_inputs: &[[f32; 3]] = &[
            [0.0, 0.0, 0.0],
            [50.0, -80.0, 10.0],   // out of typical limits
            [-90.0, 120.0, -30.0], // out the other way
            [5.0, -5.0, 2.0],      // within limits
        ];
        let last_inputs: &[[f32; 3]] = &[[0.0, 0.0, 0.0], [10.0, -20.0, 5.0]];
        // (lookingDebounceTime, time): active when debounce > time.
        let debtime = [(100i32, 50i32), (50, 100)];
        let speeds = [1.0f32, 0.5, 4.0];
        // pitch/yaw/roll min/max limits.
        let (minp, maxp, miny, maxy, minr, maxr) = (-40.0f32, 40.0, -60.0, 60.0, -20.0, 20.0);
        for &look in look_inputs {
            for &last in last_inputs {
                for &(deb, time) in &debtime {
                    for &spd in &speeds {
                        let (r_look, r_last) = unsafe {
                            let mut la = look;
                            let mut lh = last;
                            BG_UpdateLookAngles(
                                deb, &mut lh, time, &mut la, spd, minp, maxp, miny, maxy, minr,
                                maxr,
                            );
                            (la, lh)
                        };
                        let (mut c_look, mut c_last) = (look, last);
                        unsafe {
                            jka_BG_UpdateLookAngles(
                                deb,
                                c_last.as_mut_ptr(),
                                time,
                                c_look.as_mut_ptr(),
                                spd,
                                minp,
                                maxp,
                                miny,
                                maxy,
                                minr,
                                maxr,
                            );
                        }
                        for k in 0..3 {
                            assert_eq!(
                                r_look[k].to_bits(),
                                c_look[k].to_bits(),
                                "lookAngles[{k}] look={look:?} last={last:?} deb={deb} t={time} spd={spd}"
                            );
                            assert_eq!(
                                r_last[k].to_bits(),
                                c_last[k].to_bits(),
                                "lastHeadAngles[{k}] look={look:?} last={last:?} deb={deb} t={time} spd={spd}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// `BG_SwingAngles` parity: sweep destination/angle pairs (small & large deltas, both
    /// signs), tolerances, speed, the `swinging` flag, and frametime — exercising the
    /// start/stop/clamp branches — comparing the return, the mutated `*angle`, and
    /// `*swinging` bit-exact against the verbatim C body.
    #[test]
    fn bg_swingangles_matches_oracle() {
        let angles = [0.0f32, 10.0, -10.0, 170.0, -175.0, 350.0];
        let dests = [0.0f32, 5.0, 45.0, -45.0, 180.0];
        let tols = [(10.0f32, 30.0f32), (2.0, 8.0)];
        let speeds = [1.0f32, 0.5, 3.0];
        let frametimes = [0i32, 8, 50];
        for &a in &angles {
            for &d in &dests {
                for &(swingtol, clamptol) in &tols {
                    for &spd in &speeds {
                        for &ft in &frametimes {
                            for &sw in &[QFALSE, QTRUE] {
                                let (r_ret, r_ang, r_sw) = unsafe {
                                    let mut ang = a;
                                    let mut swinging = sw;
                                    let ret = BG_SwingAngles(
                                        d, swingtol, clamptol, spd, &mut ang, &mut swinging, ft,
                                    );
                                    (ret, ang, swinging)
                                };
                                let (mut c_ang, mut c_sw) = (a, sw);
                                let c_ret = unsafe {
                                    jka_BG_SwingAngles(
                                        d, swingtol, clamptol, spd, &mut c_ang, &mut c_sw, ft,
                                    )
                                };
                                let ctx = format!(
                                    "a={a} d={d} swingtol={swingtol} clamptol={clamptol} spd={spd} ft={ft} sw={sw}"
                                );
                                assert_eq!(r_ret.to_bits(), c_ret.to_bits(), "ret {ctx}");
                                assert_eq!(r_ang.to_bits(), c_ang.to_bits(), "angle {ctx}");
                                assert_eq!(r_sw, c_sw, "swinging {ctx}");
                            }
                        }
                    }
                }
            }
        }
    }

    /// `BG_InRoll2` parity: sweep the full `-8..=2100` anim domain against the verbatim C
    /// body, verifying the roll + get-up-roll case constants.
    #[test]
    fn bg_inroll2_matches_oracle() {
        for i in -8..=2100 {
            let r = unsafe {
                let mut es: entityState_t = core::mem::zeroed();
                es.legsAnim = i;
                BG_InRoll2(&mut es)
            };
            assert_eq!(r, unsafe { jka_BG_InRoll2(i) }, "BG_InRoll2 {i}");
        }
    }

    /// `PM_WeaponOkOnVehicle`/`PM_GetOkWeaponForVehicle` parity: sweep the weapon index
    /// for the predicate, and a range of `stats[STAT_WEAPONS]` bit patterns for the scan
    /// (no owned weapons, only-melee, mixed, all-set), against the verbatim C bodies.
    #[test]
    fn vehicle_weapon_predicates_match_oracle() {
        let _g = pm_lock();
        for w in -2..=25 {
            assert_eq!(
                PM_WeaponOkOnVehicle(w),
                unsafe { jka_PM_WeaponOkOnVehicle(w) },
                "PM_WeaponOkOnVehicle {w}"
            );
        }
        let bits = [
            0i32,
            1 << 2,            // only melee
            1 << 5,            // only blaster
            1 << 0 | 1 << 1,   // owned but none vehicle-legal
            1 << 3 | 1 << 5,   // saber + blaster
            0x7FFFF,           // all 19 weapons
            !0,                // every bit
        ];
        for &b in &bits {
            let r = unsafe {
                let mut ps: playerState_t = core::mem::zeroed();
                let mut pmv: pmove_t = core::mem::zeroed();
                ps.stats[STAT_WEAPONS as usize] = b;
                pmv.ps = &mut ps;
                *addr_of_mut!(pm) = &mut pmv;
                let got = PM_GetOkWeaponForVehicle();
                *addr_of_mut!(pm) = null_mut();
                got
            };
            assert_eq!(r, unsafe { jka_PM_GetOkWeaponForVehicle(b) }, "PM_GetOkWeaponForVehicle {b:#x}");
        }
    }
}
