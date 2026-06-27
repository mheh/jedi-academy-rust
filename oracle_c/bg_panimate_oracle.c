/*
 * Oracle TU for the bg_panimate.c "Animation utility functions (sequence
 * checking)" block. Each of the 50 stateless predicate functions is copied
 * VERBATIM from bg_panimate.c so the C compiler evaluates the real switch/range
 * logic independently of the Rust port in src/codemp/game/bg_panimate.rs. The
 * Rust tests drive an exhaustive integer input-sweep through both sides and
 * assert identical output, which transitively verifies the BOTH_, LS_, Q_ and
 * BLOCKED_ constant values used in every case label.
 *
 * The BOTH_* constants come from the AUTHENTIC, unmodified Raven anims.h
 * (#include'd directly -- it is a clang-clean pure enum, the anims_oracle.c
 * precedent; -I supplied per-file in build.rs). The LS_* (saberMoveName_t),
 * Q_* (saberQuadrant_t) and BLOCKED_* (saberBlockedType_t) enums are transcribed
 * VERBATIM below from bg_public.h / q_shared.h (those headers drag in the
 * clang-hostile include tree, so they cannot be #include'd; copy is faithful).
 *
 * Q_irand is NOT defined here -- it is declared and resolves at link time to the
 * real LCG in q_shared_oracle.c (both land in the same static lib). That makes
 * BG_BrokenParryForParry's LS_PARRY_UP branch random (separate RNG state from the
 * Rust port), so the Rust test excludes that single input from the equality
 * sweep and asserts set-membership instead.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

#include "anims.h"

typedef int qboolean;
#define qtrue 1
#define qfalse 0

/* --- saberMoveName_t (bg_public.h), verbatim --- */
typedef enum {
	// Invalid, or saber not armed
	LS_NONE		= 0,

	// General movements with saber
	LS_READY,
	LS_DRAW,
	LS_PUTAWAY,

	// Attacks
	LS_A_TL2BR,//4
	LS_A_L2R,
	LS_A_BL2TR,
	LS_A_BR2TL,
	LS_A_R2L,
	LS_A_TR2BL,
	LS_A_T2B,
	LS_A_BACKSTAB,
	LS_A_BACK,
	LS_A_BACK_CR,
	LS_ROLL_STAB,
	LS_A_LUNGE,
	LS_A_JUMP_T__B_,
	LS_A_FLIP_STAB,
	LS_A_FLIP_SLASH,
	LS_JUMPATTACK_DUAL,
	LS_JUMPATTACK_ARIAL_LEFT,
	LS_JUMPATTACK_ARIAL_RIGHT,
	LS_JUMPATTACK_CART_LEFT,
	LS_JUMPATTACK_CART_RIGHT,
	LS_JUMPATTACK_STAFF_LEFT,
	LS_JUMPATTACK_STAFF_RIGHT,
	LS_BUTTERFLY_LEFT,
	LS_BUTTERFLY_RIGHT,
	LS_A_BACKFLIP_ATK,
	LS_SPINATTACK_DUAL,
	LS_SPINATTACK,
	LS_LEAP_ATTACK,
	LS_SWOOP_ATTACK_RIGHT,
	LS_SWOOP_ATTACK_LEFT,
	LS_TAUNTAUN_ATTACK_RIGHT,
	LS_TAUNTAUN_ATTACK_LEFT,
	LS_KICK_F,
	LS_KICK_B,
	LS_KICK_R,
	LS_KICK_L,
	LS_KICK_S,
	LS_KICK_BF,
	LS_KICK_RL,
	LS_KICK_F_AIR,
	LS_KICK_B_AIR,
	LS_KICK_R_AIR,
	LS_KICK_L_AIR,
	LS_STABDOWN,
	LS_STABDOWN_STAFF,
	LS_STABDOWN_DUAL,
	LS_DUAL_SPIN_PROTECT,
	LS_STAFF_SOULCAL,
	LS_A1_SPECIAL,
	LS_A2_SPECIAL,
	LS_A3_SPECIAL,
	LS_UPSIDE_DOWN_ATTACK,
	LS_PULL_ATTACK_STAB,
	LS_PULL_ATTACK_SWING,
	LS_SPINATTACK_ALORA,
	LS_DUAL_FB,
	LS_DUAL_LR,
	LS_HILT_BASH,

	//starts
	LS_S_TL2BR,//26
	LS_S_L2R,
	LS_S_BL2TR,//# Start of attack chaining to SLASH LR2UL
	LS_S_BR2TL,//# Start of attack chaining to SLASH LR2UL
	LS_S_R2L,
	LS_S_TR2BL,
	LS_S_T2B,

	//returns
	LS_R_TL2BR,//33
	LS_R_L2R,
	LS_R_BL2TR,
	LS_R_BR2TL,
	LS_R_R2L,
	LS_R_TR2BL,
	LS_R_T2B,

	//transitions
	LS_T1_BR__R,//40
	LS_T1_BR_TR,
	LS_T1_BR_T_,
	LS_T1_BR_TL,
	LS_T1_BR__L,
	LS_T1_BR_BL,
	LS_T1__R_BR,//46
	LS_T1__R_TR,
	LS_T1__R_T_,
	LS_T1__R_TL,
	LS_T1__R__L,
	LS_T1__R_BL,
	LS_T1_TR_BR,//52
	LS_T1_TR__R,
	LS_T1_TR_T_,
	LS_T1_TR_TL,
	LS_T1_TR__L,
	LS_T1_TR_BL,
	LS_T1_T__BR,//58
	LS_T1_T___R,
	LS_T1_T__TR,
	LS_T1_T__TL,
	LS_T1_T___L,
	LS_T1_T__BL,
	LS_T1_TL_BR,//64
	LS_T1_TL__R,
	LS_T1_TL_TR,
	LS_T1_TL_T_,
	LS_T1_TL__L,
	LS_T1_TL_BL,
	LS_T1__L_BR,//70
	LS_T1__L__R,
	LS_T1__L_TR,
	LS_T1__L_T_,
	LS_T1__L_TL,
	LS_T1__L_BL,
	LS_T1_BL_BR,//76
	LS_T1_BL__R,
	LS_T1_BL_TR,
	LS_T1_BL_T_,
	LS_T1_BL_TL,
	LS_T1_BL__L,

	//Bounces
	LS_B1_BR,
	LS_B1__R,
	LS_B1_TR,
	LS_B1_T_,
	LS_B1_TL,
	LS_B1__L,
	LS_B1_BL,

	//Deflected attacks
	LS_D1_BR,
	LS_D1__R,
	LS_D1_TR,
	LS_D1_T_,
	LS_D1_TL,
	LS_D1__L,
	LS_D1_BL,
	LS_D1_B_,

	//Reflected attacks
	LS_V1_BR,
	LS_V1__R,
	LS_V1_TR,
	LS_V1_T_,
	LS_V1_TL,
	LS_V1__L,
	LS_V1_BL,
	LS_V1_B_,

	// Broken parries
	LS_H1_T_,//
	LS_H1_TR,
	LS_H1_TL,
	LS_H1_BR,
	LS_H1_B_,
	LS_H1_BL,

	// Knockaways
	LS_K1_T_,//
	LS_K1_TR,
	LS_K1_TL,
	LS_K1_BR,
	LS_K1_BL,

	// Parries
	LS_PARRY_UP,//
	LS_PARRY_UR,
	LS_PARRY_UL,
	LS_PARRY_LR,
	LS_PARRY_LL,

	// Projectile Reflections
	LS_REFLECT_UP,//
	LS_REFLECT_UR,
	LS_REFLECT_UL,
	LS_REFLECT_LR,
	LS_REFLECT_LL,

	LS_MOVE_MAX//
};
typedef int saberMoveName_t;

/* --- saberQuadrant_t (bg_public.h), verbatim --- */
typedef enum {
	Q_BR,
	Q_R,
	Q_TR,
	Q_T,
	Q_TL,
	Q_L,
	Q_BL,
	Q_B,
	Q_NUM_QUADS
} saberQuadrant_t;

/* --- saberBlockedType_t (q_shared.h), verbatim --- */
typedef enum {
	BLOCKED_NONE,
	BLOCKED_BOUNCE_MOVE,
	BLOCKED_PARRY_BROKEN,
	BLOCKED_ATK_BOUNCE,
	BLOCKED_UPPER_RIGHT,
	BLOCKED_UPPER_LEFT,
	BLOCKED_LOWER_RIGHT,
	BLOCKED_LOWER_LEFT,
	BLOCKED_TOP,
	BLOCKED_UPPER_RIGHT_PROJ,
	BLOCKED_UPPER_LEFT_PROJ,
	BLOCKED_LOWER_RIGHT_PROJ,
	BLOCKED_LOWER_LEFT_PROJ,
	BLOCKED_TOP_PROJ
} saberBlockedType_t;

/* --- saberMoveData_t (bg_public.h), verbatim --- (for BG_BrokenParryForAttack /
 * PM_SaberBounceForAttack, which index saberMoveData[].startQuad). The table
 * itself is defined once in bg_saber_oracle.c; declare it extern here so both
 * oracle TUs share the single definition at link time. */
typedef struct
{
	char *name;
	int animToUse;
	int	startQuad;
	int	endQuad;
	unsigned animSetFlags;
	int blendTime;
	int blocking;
	saberMoveName_t chain_idle;
	saberMoveName_t chain_attack;
	qboolean trailLength;
} saberMoveData_t;
extern saberMoveData_t saberMoveData[];

/* Resolves to the real LCG in q_shared_oracle.c at link time. */
extern int Q_irand( int value1, int value2 );

/* Forward prototypes -- the originals are declared in bg_local.h; several
 * predicates call siblings defined later in the file (e.g. BG_InSpecialJump
 * calls the rebound/backflip checks), so declare them all up front. */
qboolean BG_SaberStanceAnim( int anim );
qboolean BG_CrouchAnim( int anim );
qboolean BG_InSpecialJump( int anim );
qboolean BG_InSaberStandAnim( int anim );
qboolean BG_InReboundJump( int anim );
qboolean BG_InReboundHold( int anim );
qboolean BG_InReboundRelease( int anim );
qboolean BG_InBackFlip( int anim );
qboolean BG_DirectFlippingAnim( int anim );
qboolean BG_SaberInAttackPure( int move );
qboolean BG_SaberInAttack( int move );
qboolean BG_SaberInKata( int saberMove );
qboolean BG_InKataAnim( int anim );
qboolean BG_SaberInSpecial( int move );
qboolean BG_KickMove( int move );
qboolean BG_SaberInIdle( int move );
qboolean BG_InExtraDefenseSaberMove( int move );
qboolean BG_FlippingAnim( int anim );
qboolean BG_SpinningSaberAnim( int anim );
qboolean BG_SaberInSpecialAttack( int anim );
qboolean BG_KickingAnim( int anim );
int BG_InGrappleMove( int anim );
int BG_BrokenParryForAttack( int move );
int BG_BrokenParryForParry( int move );
int BG_KnockawayForParry( int move );
int PM_SaberBounceForAttack( int move );
qboolean BG_InSpecialDeathAnim( int anim );
qboolean BG_InDeathAnim ( int anim );
qboolean BG_InKnockDownOnly( int anim );
qboolean BG_InSaberLockOld( int anim );
qboolean BG_InSaberLock( int anim );
qboolean PM_InCartwheel( int anim );
qboolean BG_StabDownAnim( int anim );
int PM_SaberDeflectionForQuad( int quad );
qboolean PM_SaberInDeflect( int move );
qboolean PM_SaberInParry( int move );
qboolean PM_SaberInKnockaway( int move );
qboolean PM_SaberInReflect( int move );
qboolean PM_SaberInStart( int move );
qboolean PM_SaberInReturn( int move );
qboolean BG_SaberInReturn( int move );
qboolean PM_InSaberAnim( int anim );
qboolean PM_PainAnim( int anim );
qboolean PM_JumpingAnim( int anim );
qboolean PM_LandingAnim( int anim );
qboolean PM_SpinningAnim( int anim );
qboolean PM_InOnGroundAnim ( int anim );
qboolean BG_SuperBreakLoseAnim( int anim );
qboolean BG_SuperBreakWinAnim( int anim );
qboolean BG_SaberLockBreakAnim( int anim );
qboolean BG_FullBodyTauntAnim( int anim );
qboolean PM_SaberInTransition( int move );
qboolean BG_SaberInTransitionAny( int move );

/* ====================== verbatim function bodies ====================== */

qboolean BG_SaberStanceAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_STAND1://not really a saberstance anim, actually... "saber off" stance
	case BOTH_STAND2://single-saber, medium style
	case BOTH_SABERFAST_STANCE://single-saber, fast style
	case BOTH_SABERSLOW_STANCE://single-saber, strong style
	case BOTH_SABERSTAFF_STANCE://saber staff style
	case BOTH_SABERDUAL_STANCE://dual saber style
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_CrouchAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_SIT1:				//# Normal chair sit.
	case BOTH_SIT2:				//# Lotus position.
	case BOTH_SIT3:				//# Sitting in tired position: elbows on knees
	case BOTH_CROUCH1:			//# Transition from standing to crouch
	case BOTH_CROUCH1IDLE:		//# Crouching idle
	case BOTH_CROUCH1WALK:		//# Walking while crouched
	case BOTH_CROUCH1WALKBACK:	//# Walking while crouched
	case BOTH_CROUCH2TOSTAND1:	//# going from crouch2 to stand1
	case BOTH_CROUCH3:			//# Desann crouching down to Kyle (cin 9)
	case BOTH_KNEES1:			//# Tavion on her knees
	case BOTH_CROUCHATTACKBACK1://FIXME: not if in middle of anim?
	case BOTH_ROLL_STAB:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_InSpecialJump( int anim )
{
	switch ( (anim) )
	{
	case BOTH_WALL_RUN_RIGHT:
	case BOTH_WALL_RUN_RIGHT_STOP:
	case BOTH_WALL_RUN_RIGHT_FLIP:
	case BOTH_WALL_RUN_LEFT:
	case BOTH_WALL_RUN_LEFT_STOP:
	case BOTH_WALL_RUN_LEFT_FLIP:
	case BOTH_WALL_FLIP_RIGHT:
	case BOTH_WALL_FLIP_LEFT:
	case BOTH_FLIP_BACK1:
	case BOTH_FLIP_BACK2:
	case BOTH_FLIP_BACK3:
	case BOTH_WALL_FLIP_BACK1:
	case BOTH_BUTTERFLY_LEFT:
	case BOTH_BUTTERFLY_RIGHT:
	case BOTH_BUTTERFLY_FL1:
	case BOTH_BUTTERFLY_FR1:
	case BOTH_FJSS_TR_BL:
	case BOTH_FJSS_TL_BR:
	case BOTH_FORCELEAP2_T__B_:
	case BOTH_JUMPFLIPSLASHDOWN1://#
	case BOTH_JUMPFLIPSTABDOWN://#
	case BOTH_JUMPATTACK6:
	case BOTH_JUMPATTACK7:
	case BOTH_ARIAL_LEFT:
	case BOTH_ARIAL_RIGHT:
	case BOTH_ARIAL_F1:
	case BOTH_CARTWHEEL_LEFT:
	case BOTH_CARTWHEEL_RIGHT:

	case BOTH_FORCELONGLEAP_START:
	case BOTH_FORCELONGLEAP_ATTACK:
	case BOTH_FORCEWALLRUNFLIP_START:
	case BOTH_FORCEWALLRUNFLIP_END:
	case BOTH_FORCEWALLRUNFLIP_ALT:
	case BOTH_FLIP_ATTACK7:
	case BOTH_FLIP_HOLD7:
	case BOTH_FLIP_LAND:
	case BOTH_A7_SOULCAL:
		return qtrue;
	}
	if ( BG_InReboundJump( anim ) )
	{
		return qtrue;
	}
	if ( BG_InReboundHold( anim ) )
	{
		return qtrue;
	}
	if ( BG_InReboundRelease( anim ) )
	{
		return qtrue;
	}
	if ( BG_InBackFlip( anim ) )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean BG_InSaberStandAnim( int anim )
{
	switch ( (anim) )
	{
	case BOTH_SABERFAST_STANCE:
	case BOTH_STAND2:
	case BOTH_SABERSLOW_STANCE:
	case BOTH_SABERDUAL_STANCE:
	case BOTH_SABERSTAFF_STANCE:
		return qtrue;
	default:
		return qfalse;
	}
}

qboolean BG_InReboundJump( int anim )
{
	switch ( anim )
	{
	case BOTH_FORCEWALLREBOUND_FORWARD:
	case BOTH_FORCEWALLREBOUND_LEFT:
	case BOTH_FORCEWALLREBOUND_BACK:
	case BOTH_FORCEWALLREBOUND_RIGHT:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_InReboundHold( int anim )
{
	switch ( anim )
	{
	case BOTH_FORCEWALLHOLD_FORWARD:
	case BOTH_FORCEWALLHOLD_LEFT:
	case BOTH_FORCEWALLHOLD_BACK:
	case BOTH_FORCEWALLHOLD_RIGHT:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_InReboundRelease( int anim )
{
	switch ( anim )
	{
	case BOTH_FORCEWALLRELEASE_FORWARD:
	case BOTH_FORCEWALLRELEASE_LEFT:
	case BOTH_FORCEWALLRELEASE_BACK:
	case BOTH_FORCEWALLRELEASE_RIGHT:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_InBackFlip( int anim )
{
	switch ( anim )
	{
	case BOTH_FLIP_BACK1:
	case BOTH_FLIP_BACK2:
	case BOTH_FLIP_BACK3:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_DirectFlippingAnim( int anim )
{
	switch ( (anim) )
	{
	case BOTH_FLIP_F:			//# Flip forward
	case BOTH_FLIP_B:			//# Flip backwards
	case BOTH_FLIP_L:			//# Flip left
	case BOTH_FLIP_R:			//# Flip right
		return qtrue;
		break;
	}

	return qfalse;
}

qboolean BG_SaberInAttackPure( int move )
{
	if ( move >= LS_A_TL2BR && move <= LS_A_T2B )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean BG_SaberInAttack( int move )
{
	if ( move >= LS_A_TL2BR && move <= LS_A_T2B )
	{
		return qtrue;
	}
	switch ( move )
	{
	case LS_A_BACK:
	case LS_A_BACK_CR:
	case LS_A_BACKSTAB:
	case LS_ROLL_STAB:
	case LS_A_LUNGE:
	case LS_A_JUMP_T__B_:
	case LS_A_FLIP_STAB:
	case LS_A_FLIP_SLASH:
	case LS_JUMPATTACK_DUAL:
	case LS_JUMPATTACK_ARIAL_LEFT:
	case LS_JUMPATTACK_ARIAL_RIGHT:
	case LS_JUMPATTACK_CART_LEFT:
	case LS_JUMPATTACK_CART_RIGHT:
	case LS_JUMPATTACK_STAFF_LEFT:
	case LS_JUMPATTACK_STAFF_RIGHT:
	case LS_BUTTERFLY_LEFT:
	case LS_BUTTERFLY_RIGHT:
	case LS_A_BACKFLIP_ATK:
	case LS_SPINATTACK_DUAL:
	case LS_SPINATTACK:
	case LS_LEAP_ATTACK:
	case LS_SWOOP_ATTACK_RIGHT:
	case LS_SWOOP_ATTACK_LEFT:
	case LS_TAUNTAUN_ATTACK_RIGHT:
	case LS_TAUNTAUN_ATTACK_LEFT:
	case LS_KICK_F:
	case LS_KICK_B:
	case LS_KICK_R:
	case LS_KICK_L:
	case LS_KICK_S:
	case LS_KICK_BF:
	case LS_KICK_RL:
	case LS_KICK_F_AIR:
	case LS_KICK_B_AIR:
	case LS_KICK_R_AIR:
	case LS_KICK_L_AIR:
	case LS_STABDOWN:
	case LS_STABDOWN_STAFF:
	case LS_STABDOWN_DUAL:
	case LS_DUAL_SPIN_PROTECT:
	case LS_STAFF_SOULCAL:
	case LS_A1_SPECIAL:
	case LS_A2_SPECIAL:
	case LS_A3_SPECIAL:
	case LS_UPSIDE_DOWN_ATTACK:
	case LS_PULL_ATTACK_STAB:
	case LS_PULL_ATTACK_SWING:
	case LS_SPINATTACK_ALORA:
	case LS_DUAL_FB:
	case LS_DUAL_LR:
	case LS_HILT_BASH:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_SaberInKata( int saberMove )
{
	switch ( saberMove )
	{
	case LS_A1_SPECIAL:
	case LS_A2_SPECIAL:
	case LS_A3_SPECIAL:
	case LS_DUAL_SPIN_PROTECT:
	case LS_STAFF_SOULCAL:
		return qtrue;
	}
	return qfalse;
}

qboolean BG_InKataAnim(int anim)
{
	switch (anim)
	{
	case BOTH_A6_SABERPROTECT:
	case BOTH_A7_SOULCAL:
	case BOTH_A1_SPECIAL:
	case BOTH_A2_SPECIAL:
	case BOTH_A3_SPECIAL:
		return qtrue;
	}
	return qfalse;
}

qboolean BG_SaberInSpecial( int move )
{
	switch( move )
	{
	case LS_A_BACK:
	case LS_A_BACK_CR:
	case LS_A_BACKSTAB:
	case LS_ROLL_STAB:
	case LS_A_LUNGE:
	case LS_A_JUMP_T__B_:
	case LS_A_FLIP_STAB:
	case LS_A_FLIP_SLASH:
	case LS_JUMPATTACK_DUAL:
	case LS_JUMPATTACK_ARIAL_LEFT:
	case LS_JUMPATTACK_ARIAL_RIGHT:
	case LS_JUMPATTACK_CART_LEFT:
	case LS_JUMPATTACK_CART_RIGHT:
	case LS_JUMPATTACK_STAFF_LEFT:
	case LS_JUMPATTACK_STAFF_RIGHT:
	case LS_BUTTERFLY_LEFT:
	case LS_BUTTERFLY_RIGHT:
	case LS_A_BACKFLIP_ATK:
	case LS_SPINATTACK_DUAL:
	case LS_SPINATTACK:
	case LS_LEAP_ATTACK:
	case LS_SWOOP_ATTACK_RIGHT:
	case LS_SWOOP_ATTACK_LEFT:
	case LS_TAUNTAUN_ATTACK_RIGHT:
	case LS_TAUNTAUN_ATTACK_LEFT:
	case LS_KICK_F:
	case LS_KICK_B:
	case LS_KICK_R:
	case LS_KICK_L:
	case LS_KICK_S:
	case LS_KICK_BF:
	case LS_KICK_RL:
	case LS_KICK_F_AIR:
	case LS_KICK_B_AIR:
	case LS_KICK_R_AIR:
	case LS_KICK_L_AIR:
	case LS_STABDOWN:
	case LS_STABDOWN_STAFF:
	case LS_STABDOWN_DUAL:
	case LS_DUAL_SPIN_PROTECT:
	case LS_STAFF_SOULCAL:
	case LS_A1_SPECIAL:
	case LS_A2_SPECIAL:
	case LS_A3_SPECIAL:
	case LS_UPSIDE_DOWN_ATTACK:
	case LS_PULL_ATTACK_STAB:
	case LS_PULL_ATTACK_SWING:
	case LS_SPINATTACK_ALORA:
	case LS_DUAL_FB:
	case LS_DUAL_LR:
	case LS_HILT_BASH:
		return qtrue;
	}
	return qfalse;
}

qboolean BG_KickMove( int move )
{
	switch( move )
	{
	case LS_KICK_F:
	case LS_KICK_B:
	case LS_KICK_R:
	case LS_KICK_L:
	case LS_KICK_S:
	case LS_KICK_BF:
	case LS_KICK_RL:
	case LS_KICK_F_AIR:
	case LS_KICK_B_AIR:
	case LS_KICK_R_AIR:
	case LS_KICK_L_AIR:
	case LS_HILT_BASH:
		return qtrue;
	}
	return qfalse;
}

qboolean BG_SaberInIdle( int move )
{
	switch ( move )
	{
	case LS_NONE:
	case LS_READY:
	case LS_DRAW:
	case LS_PUTAWAY:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_InExtraDefenseSaberMove( int move )
{
	switch ( move )
	{
	case LS_SPINATTACK_DUAL:
	case LS_SPINATTACK:
	case LS_DUAL_SPIN_PROTECT:
	case LS_STAFF_SOULCAL:
	case LS_A1_SPECIAL:
	case LS_A2_SPECIAL:
	case LS_A3_SPECIAL:
	case LS_JUMPATTACK_DUAL:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_FlippingAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_FLIP_F:			//# Flip forward
	case BOTH_FLIP_B:			//# Flip backwards
	case BOTH_FLIP_L:			//# Flip left
	case BOTH_FLIP_R:			//# Flip right
	case BOTH_WALL_RUN_RIGHT_FLIP:
	case BOTH_WALL_RUN_LEFT_FLIP:
	case BOTH_WALL_FLIP_RIGHT:
	case BOTH_WALL_FLIP_LEFT:
	case BOTH_FLIP_BACK1:
	case BOTH_FLIP_BACK2:
	case BOTH_FLIP_BACK3:
	case BOTH_WALL_FLIP_BACK1:
	//Not really flips, but...
	case BOTH_WALL_RUN_RIGHT:
	case BOTH_WALL_RUN_LEFT:
	case BOTH_WALL_RUN_RIGHT_STOP:
	case BOTH_WALL_RUN_LEFT_STOP:
	case BOTH_BUTTERFLY_LEFT:
	case BOTH_BUTTERFLY_RIGHT:
	case BOTH_BUTTERFLY_FL1:
	case BOTH_BUTTERFLY_FR1:
	//
	case BOTH_ARIAL_LEFT:
	case BOTH_ARIAL_RIGHT:
	case BOTH_ARIAL_F1:
	case BOTH_CARTWHEEL_LEFT:
	case BOTH_CARTWHEEL_RIGHT:
	case BOTH_JUMPFLIPSLASHDOWN1:
	case BOTH_JUMPFLIPSTABDOWN:
	case BOTH_JUMPATTACK6:
	case BOTH_JUMPATTACK7:
	//JKA
	case BOTH_FORCEWALLRUNFLIP_END:
	case BOTH_FORCEWALLRUNFLIP_ALT:
	case BOTH_FLIP_ATTACK7:
	case BOTH_A7_SOULCAL:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_SpinningSaberAnim( int anim )
{
	switch ( anim )
	{
	//level 1 - FIXME: level 1 will have *no* spins
	case BOTH_T1_BR_BL:
	case BOTH_T1__R__L:
	case BOTH_T1__R_BL:
	case BOTH_T1_TR_BL:
	case BOTH_T1_BR_TL:
	case BOTH_T1_BR__L:
	case BOTH_T1_TL_BR:
	case BOTH_T1__L_BR:
	case BOTH_T1__L__R:
	case BOTH_T1_BL_BR:
	case BOTH_T1_BL__R:
	case BOTH_T1_BL_TR:
	//level 2
	case BOTH_T2_BR__L:
	case BOTH_T2_BR_BL:
	case BOTH_T2__R_BL:
	case BOTH_T2__L_BR:
	case BOTH_T2_BL_BR:
	case BOTH_T2_BL__R:
	//level 3
	case BOTH_T3_BR__L:
	case BOTH_T3_BR_BL:
	case BOTH_T3__R_BL:
	case BOTH_T3__L_BR:
	case BOTH_T3_BL_BR:
	case BOTH_T3_BL__R:
	//level 4
	case BOTH_T4_BR__L:
	case BOTH_T4_BR_BL:
	case BOTH_T4__R_BL:
	case BOTH_T4__L_BR:
	case BOTH_T4_BL_BR:
	case BOTH_T4_BL__R:
	//level 5
	case BOTH_T5_BR_BL:
	case BOTH_T5__R__L:
	case BOTH_T5__R_BL:
	case BOTH_T5_TR_BL:
	case BOTH_T5_BR_TL:
	case BOTH_T5_BR__L:
	case BOTH_T5_TL_BR:
	case BOTH_T5__L_BR:
	case BOTH_T5__L__R:
	case BOTH_T5_BL_BR:
	case BOTH_T5_BL__R:
	case BOTH_T5_BL_TR:
	//level 6
	case BOTH_T6_BR_TL:
	case BOTH_T6__R_TL:
	case BOTH_T6__R__L:
	case BOTH_T6__R_BL:
	case BOTH_T6_TR_TL:
	case BOTH_T6_TR__L:
	case BOTH_T6_TR_BL:
	case BOTH_T6_T__TL:
	case BOTH_T6_T__BL:
	case BOTH_T6_TL_BR:
	case BOTH_T6__L_BR:
	case BOTH_T6__L__R:
	case BOTH_T6_TL__R:
	case BOTH_T6_TL_TR:
	case BOTH_T6__L_TR:
	case BOTH_T6__L_T_:
	case BOTH_T6_BL_T_:
	case BOTH_T6_BR__L:
	case BOTH_T6_BR_BL:
	case BOTH_T6_BL_BR:
	case BOTH_T6_BL__R:
	case BOTH_T6_BL_TR:
	//level 7
	case BOTH_T7_BR_TL:
	case BOTH_T7_BR__L:
	case BOTH_T7_BR_BL:
	case BOTH_T7__R__L:
	case BOTH_T7__R_BL:
	case BOTH_T7_TR__L:
	case BOTH_T7_T___R:
	case BOTH_T7_TL_BR:
	case BOTH_T7__L_BR:
	case BOTH_T7__L__R:
	case BOTH_T7_BL_BR:
	case BOTH_T7_BL__R:
	case BOTH_T7_BL_TR:
	case BOTH_T7_TL_TR:
	case BOTH_T7_T__BR:
	case BOTH_T7__L_TR:
	case BOTH_V7_BL_S7:
	//special
	//case BOTH_A2_STABBACK1:
	case BOTH_ATTACK_BACK:
	case BOTH_CROUCHATTACKBACK1:
	case BOTH_BUTTERFLY_LEFT:
	case BOTH_BUTTERFLY_RIGHT:
	case BOTH_FJSS_TR_BL:
	case BOTH_FJSS_TL_BR:
	case BOTH_JUMPFLIPSLASHDOWN1:
	case BOTH_JUMPFLIPSTABDOWN:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_SaberInSpecialAttack( int anim )
{
	switch ( anim )
	{
	case BOTH_A2_STABBACK1:
	case BOTH_ATTACK_BACK:
	case BOTH_CROUCHATTACKBACK1:
	case BOTH_ROLL_STAB:
	case BOTH_BUTTERFLY_LEFT:
	case BOTH_BUTTERFLY_RIGHT:
	case BOTH_BUTTERFLY_FL1:
	case BOTH_BUTTERFLY_FR1:
	case BOTH_FJSS_TR_BL:
	case BOTH_FJSS_TL_BR:
	case BOTH_LUNGE2_B__T_:
	case BOTH_FORCELEAP2_T__B_:
	case BOTH_JUMPFLIPSLASHDOWN1://#
	case BOTH_JUMPFLIPSTABDOWN://#
	case BOTH_JUMPATTACK6:
	case BOTH_JUMPATTACK7:
	case BOTH_SPINATTACK6:
	case BOTH_SPINATTACK7:
	case BOTH_FORCELONGLEAP_ATTACK:
	case BOTH_VS_ATR_S:
	case BOTH_VS_ATL_S:
	case BOTH_VT_ATR_S:
	case BOTH_VT_ATL_S:
	case BOTH_A7_KICK_F:
	case BOTH_A7_KICK_B:
	case BOTH_A7_KICK_R:
	case BOTH_A7_KICK_L:
	case BOTH_A7_KICK_S:
	case BOTH_A7_KICK_BF:
	case BOTH_A7_KICK_RL:
	case BOTH_A7_KICK_F_AIR:
	case BOTH_A7_KICK_B_AIR:
	case BOTH_A7_KICK_R_AIR:
	case BOTH_A7_KICK_L_AIR:
	case BOTH_STABDOWN:
	case BOTH_STABDOWN_STAFF:
	case BOTH_STABDOWN_DUAL:
	case BOTH_A6_SABERPROTECT:
	case BOTH_A7_SOULCAL:
	case BOTH_A1_SPECIAL:
	case BOTH_A2_SPECIAL:
	case BOTH_A3_SPECIAL:
	case BOTH_FLIP_ATTACK7:
	case BOTH_PULL_IMPALE_STAB:
	case BOTH_PULL_IMPALE_SWING:
	case BOTH_ALORA_SPIN_SLASH:
	case BOTH_A6_FB:
	case BOTH_A6_LR:
	case BOTH_A7_HILT:
		return qtrue;
	}
	return qfalse;
}

qboolean BG_KickingAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_A7_KICK_F:
	case BOTH_A7_KICK_B:
	case BOTH_A7_KICK_R:
	case BOTH_A7_KICK_L:
	case BOTH_A7_KICK_S:
	case BOTH_A7_KICK_BF:
	case BOTH_A7_KICK_RL:
	case BOTH_A7_KICK_F_AIR:
	case BOTH_A7_KICK_B_AIR:
	case BOTH_A7_KICK_R_AIR:
	case BOTH_A7_KICK_L_AIR:
	case BOTH_A7_HILT:
	//NOT kicks, but do kick traces anyway
	case BOTH_GETUP_BROLL_B:
	case BOTH_GETUP_BROLL_F:
	case BOTH_GETUP_FROLL_B:
	case BOTH_GETUP_FROLL_F:
		return qtrue;
		break;
	}
	return qfalse;
}

int BG_InGrappleMove(int anim)
{
	switch (anim)
	{
	case BOTH_KYLE_GRAB:
	case BOTH_KYLE_MISS:
		return 1; //grabbing at someone
	case BOTH_KYLE_PA_1:
	case BOTH_KYLE_PA_2:
		return 2; //beating the shit out of someone
	case BOTH_PLAYER_PA_1:
	case BOTH_PLAYER_PA_2:
	case BOTH_PLAYER_PA_FLY:
		return 3; //getting the shit beaten out of you
		break;
	}

	return 0;
}

int BG_BrokenParryForAttack( int move )
{
	//Our attack was knocked away by a knockaway parry
	//FIXME: need actual anims for this
	//FIXME: need to know which side of the saber was hit!  For now, we presume the saber gets knocked away from the center
	switch ( saberMoveData[move].startQuad )
	{
	case Q_B:
		return LS_V1_B_;
		break;
	case Q_BR:
		return LS_V1_BR;
		break;
	case Q_R:
		return LS_V1__R;
		break;
	case Q_TR:
		return LS_V1_TR;
		break;
	case Q_T:
		return LS_V1_T_;
		break;
	case Q_TL:
		return LS_V1_TL;
		break;
	case Q_L:
		return LS_V1__L;
		break;
	case Q_BL:
		return LS_V1_BL;
		break;
	}
	return LS_NONE;
}

int PM_SaberBounceForAttack( int move )
{
	switch ( saberMoveData[move].startQuad )
	{
	case Q_B:
	case Q_BR:
		return LS_B1_BR;
		break;
	case Q_R:
		return LS_B1__R;
		break;
	case Q_TR:
		return LS_B1_TR;
		break;
	case Q_T:
		return LS_B1_T_;
		break;
	case Q_TL:
		return LS_B1_TL;
		break;
	case Q_L:
		return LS_B1__L;
		break;
	case Q_BL:
		return LS_B1_BL;
		break;
	}
	return LS_NONE;
}

int BG_BrokenParryForParry( int move )
{
	//FIXME: need actual anims for this
	//FIXME: need to know which side of the saber was hit!  For now, we presume the saber gets knocked away from the center
	switch ( move )
	{
	case LS_PARRY_UP:
		//Hmm... since we don't know what dir the hit came from, randomly pick knock down or knock back
		if ( Q_irand( 0, 1 ) )
		{
			return LS_H1_B_;
		}
		else
		{
			return LS_H1_T_;
		}
		break;
	case LS_PARRY_UR:
		return LS_H1_TR;
		break;
	case LS_PARRY_UL:
		return LS_H1_TL;
		break;
	case LS_PARRY_LR:
		return LS_H1_BR;
		break;
	case LS_PARRY_LL:
		return LS_H1_BL;
		break;
	case LS_READY:
		return LS_H1_B_;//???
		break;
	}
	return LS_NONE;
}

int BG_KnockawayForParry( int move )
{
	//FIXME: need actual anims for this
	//FIXME: need to know which side of the saber was hit!  For now, we presume the saber gets knocked away from the center
	switch ( move )
	{
	case BLOCKED_TOP://LS_PARRY_UP:
		return LS_K1_T_;//push up
		break;
	case BLOCKED_UPPER_RIGHT://LS_PARRY_UR:
	default://case LS_READY:
		return LS_K1_TR;//push up, slightly to right
		break;
	case BLOCKED_UPPER_LEFT://LS_PARRY_UL:
		return LS_K1_TL;//push up and to left
		break;
	case BLOCKED_LOWER_RIGHT://LS_PARRY_LR:
		return LS_K1_BR;//push down and to left
		break;
	case BLOCKED_LOWER_LEFT://LS_PARRY_LL:
		return LS_K1_BL;//push down and to right
		break;
	}
	//return LS_NONE;
}

qboolean BG_InSpecialDeathAnim( int anim )
{
	switch( anim )
	{
	case BOTH_DEATH_ROLL:		//# Death anim from a roll
	case BOTH_DEATH_FLIP:		//# Death anim from a flip
	case BOTH_DEATH_SPIN_90_R:	//# Death anim when facing 90 degrees right
	case BOTH_DEATH_SPIN_90_L:	//# Death anim when facing 90 degrees left
	case BOTH_DEATH_SPIN_180:	//# Death anim when facing backwards
	case BOTH_DEATH_LYING_UP:	//# Death anim when lying on back
	case BOTH_DEATH_LYING_DN:	//# Death anim when lying on front
	case BOTH_DEATH_FALLING_DN:	//# Death anim when falling on face
	case BOTH_DEATH_FALLING_UP:	//# Death anim when falling on back
	case BOTH_DEATH_CROUCHED:	//# Death anim when crouched
		return qtrue;
		break;
	default:
		return qfalse;
		break;
	}
}

qboolean BG_InDeathAnim ( int anim )
{//Purposely does not cover stumbledeath and falldeath...
	switch( anim )
	{
	case BOTH_DEATH1:		//# First Death anim
	case BOTH_DEATH2:			//# Second Death anim
	case BOTH_DEATH3:			//# Third Death anim
	case BOTH_DEATH4:			//# Fourth Death anim
	case BOTH_DEATH5:			//# Fifth Death anim
	case BOTH_DEATH6:			//# Sixth Death anim
	case BOTH_DEATH7:			//# Seventh Death anim
	case BOTH_DEATH8:			//#
	case BOTH_DEATH9:			//#
	case BOTH_DEATH10:			//#
	case BOTH_DEATH11:			//#
	case BOTH_DEATH12:			//#
	case BOTH_DEATH13:			//#
	case BOTH_DEATH14:			//#
	case BOTH_DEATH14_UNGRIP:	//# Desann's end death (cin #35)
	case BOTH_DEATH14_SITUP:		//# Tavion sitting up after having been thrown (cin #23)
	case BOTH_DEATH15:			//#
	case BOTH_DEATH16:			//#
	case BOTH_DEATH17:			//#
	case BOTH_DEATH18:			//#
	case BOTH_DEATH19:			//#
	case BOTH_DEATH20:			//#
	case BOTH_DEATH21:			//#
	case BOTH_DEATH22:			//#
	case BOTH_DEATH23:			//#
	case BOTH_DEATH24:			//#
	case BOTH_DEATH25:			//#

	case BOTH_DEATHFORWARD1:		//# First Death in which they get thrown forward
	case BOTH_DEATHFORWARD2:		//# Second Death in which they get thrown forward
	case BOTH_DEATHFORWARD3:		//# Tavion's falling in cin# 23
	case BOTH_DEATHBACKWARD1:	//# First Death in which they get thrown backward
	case BOTH_DEATHBACKWARD2:	//# Second Death in which they get thrown backward

	case BOTH_DEATH1IDLE:		//# Idle while close to death
	case BOTH_LYINGDEATH1:		//# Death to play when killed lying down
	case BOTH_STUMBLEDEATH1:		//# Stumble forward and fall face first death
	case BOTH_FALLDEATH1:		//# Fall forward off a high cliff and splat death - start
	case BOTH_FALLDEATH1INAIR:	//# Fall forward off a high cliff and splat death - loop
	case BOTH_FALLDEATH1LAND:	//# Fall forward off a high cliff and splat death - hit bottom
	//# #sep case BOTH_ DEAD POSES # Should be last frame of corresponding previous anims
	case BOTH_DEAD1:				//# First Death finished pose
	case BOTH_DEAD2:				//# Second Death finished pose
	case BOTH_DEAD3:				//# Third Death finished pose
	case BOTH_DEAD4:				//# Fourth Death finished pose
	case BOTH_DEAD5:				//# Fifth Death finished pose
	case BOTH_DEAD6:				//# Sixth Death finished pose
	case BOTH_DEAD7:				//# Seventh Death finished pose
	case BOTH_DEAD8:				//#
	case BOTH_DEAD9:				//#
	case BOTH_DEAD10:			//#
	case BOTH_DEAD11:			//#
	case BOTH_DEAD12:			//#
	case BOTH_DEAD13:			//#
	case BOTH_DEAD14:			//#
	case BOTH_DEAD15:			//#
	case BOTH_DEAD16:			//#
	case BOTH_DEAD17:			//#
	case BOTH_DEAD18:			//#
	case BOTH_DEAD19:			//#
	case BOTH_DEAD20:			//#
	case BOTH_DEAD21:			//#
	case BOTH_DEAD22:			//#
	case BOTH_DEAD23:			//#
	case BOTH_DEAD24:			//#
	case BOTH_DEAD25:			//#
	case BOTH_DEADFORWARD1:		//# First thrown forward death finished pose
	case BOTH_DEADFORWARD2:		//# Second thrown forward death finished pose
	case BOTH_DEADBACKWARD1:		//# First thrown backward death finished pose
	case BOTH_DEADBACKWARD2:		//# Second thrown backward death finished pose
	case BOTH_LYINGDEAD1:		//# Killed lying down death finished pose
	case BOTH_STUMBLEDEAD1:		//# Stumble forward death finished pose
	case BOTH_FALLDEAD1LAND:		//# Fall forward and splat death finished pose
	//# #sep case BOTH_ DEAD TWITCH/FLOP # React to being shot from death poses
	case BOTH_DEADFLOP1:		//# React to being shot from First Death finished pose
	case BOTH_DEADFLOP2:		//# React to being shot from Second Death finished pose
	case BOTH_DISMEMBER_HEAD1:	//#
	case BOTH_DISMEMBER_TORSO1:	//#
	case BOTH_DISMEMBER_LLEG:	//#
	case BOTH_DISMEMBER_RLEG:	//#
	case BOTH_DISMEMBER_RARM:	//#
	case BOTH_DISMEMBER_LARM:	//#
		return qtrue;
		break;
	default:
		return BG_InSpecialDeathAnim( anim );
		break;
	}
}

qboolean BG_InKnockDownOnly( int anim )
{
	switch ( anim )
	{
	case BOTH_KNOCKDOWN1:
	case BOTH_KNOCKDOWN2:
	case BOTH_KNOCKDOWN3:
	case BOTH_KNOCKDOWN4:
	case BOTH_KNOCKDOWN5:
		return qtrue;
	}
	return qfalse;
}

qboolean BG_InSaberLockOld( int anim )
{
	switch ( anim )
	{
	case BOTH_BF2LOCK:
	case BOTH_BF1LOCK:
	case BOTH_CWCIRCLELOCK:
	case BOTH_CCWCIRCLELOCK:
		return qtrue;
	}
	return qfalse;
}

qboolean BG_InSaberLock( int anim )
{
	switch ( anim )
	{
	case BOTH_LK_S_DL_S_L_1:		//lock if I'm using single vs. a dual
	case BOTH_LK_S_DL_T_L_1:		//lock if I'm using single vs. a dual
	case BOTH_LK_S_ST_S_L_1:		//lock if I'm using single vs. a staff
	case BOTH_LK_S_ST_T_L_1:		//lock if I'm using single vs. a staff
	case BOTH_LK_S_S_S_L_1:		//lock if I'm using single vs. a single and I initiated
	case BOTH_LK_S_S_T_L_1:		//lock if I'm using single vs. a single and I initiated
	case BOTH_LK_DL_DL_S_L_1:	//lock if I'm using dual vs. dual and I initiated
	case BOTH_LK_DL_DL_T_L_1:	//lock if I'm using dual vs. dual and I initiated
	case BOTH_LK_DL_ST_S_L_1:	//lock if I'm using dual vs. a staff
	case BOTH_LK_DL_ST_T_L_1:	//lock if I'm using dual vs. a staff
	case BOTH_LK_DL_S_S_L_1:		//lock if I'm using dual vs. a single
	case BOTH_LK_DL_S_T_L_1:		//lock if I'm using dual vs. a single
	case BOTH_LK_ST_DL_S_L_1:	//lock if I'm using staff vs. dual
	case BOTH_LK_ST_DL_T_L_1:	//lock if I'm using staff vs. dual
	case BOTH_LK_ST_ST_S_L_1:	//lock if I'm using staff vs. a staff and I initiated
	case BOTH_LK_ST_ST_T_L_1:	//lock if I'm using staff vs. a staff and I initiated
	case BOTH_LK_ST_S_S_L_1:		//lock if I'm using staff vs. a single
	case BOTH_LK_ST_S_T_L_1:		//lock if I'm using staff vs. a single
	case BOTH_LK_S_S_S_L_2:
	case BOTH_LK_S_S_T_L_2:
	case BOTH_LK_DL_DL_S_L_2:
	case BOTH_LK_DL_DL_T_L_2:
	case BOTH_LK_ST_ST_S_L_2:
	case BOTH_LK_ST_ST_T_L_2:
		return qtrue;
		break;
	default:
		return BG_InSaberLockOld( anim );
		break;
	}
	//return qfalse;
}

qboolean PM_InCartwheel( int anim )
{
	switch ( anim )
	{
	case BOTH_ARIAL_LEFT:
	case BOTH_ARIAL_RIGHT:
	case BOTH_ARIAL_F1:
	case BOTH_CARTWHEEL_LEFT:
	case BOTH_CARTWHEEL_RIGHT:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_StabDownAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_STABDOWN:
	case BOTH_STABDOWN_STAFF:
	case BOTH_STABDOWN_DUAL:
		return qtrue;
	}
	return qfalse;
}

int PM_SaberDeflectionForQuad( int quad )
{
	switch ( quad )
	{
	case Q_B:
		return LS_D1_B_;
		break;
	case Q_BR:
		return LS_D1_BR;
		break;
	case Q_R:
		return LS_D1__R;
		break;
	case Q_TR:
		return LS_D1_TR;
		break;
	case Q_T:
		return LS_D1_T_;
		break;
	case Q_TL:
		return LS_D1_TL;
		break;
	case Q_L:
		return LS_D1__L;
		break;
	case Q_BL:
		return LS_D1_BL;
		break;
	}
	return LS_NONE;
}

qboolean PM_SaberInDeflect( int move )
{
	if ( move >= LS_D1_BR && move <= LS_D1_B_ )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean PM_SaberInParry( int move )
{
	if ( move >= LS_PARRY_UP && move <= LS_PARRY_LL )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean PM_SaberInKnockaway( int move )
{
	if ( move >= LS_K1_T_ && move <= LS_K1_BL )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean PM_SaberInReflect( int move )
{
	if ( move >= LS_REFLECT_UP && move <= LS_REFLECT_LL )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean PM_SaberInStart( int move )
{
	if ( move >= LS_S_TL2BR && move <= LS_S_T2B )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean PM_SaberInReturn( int move )
{
	if ( move >= LS_R_TL2BR && move <= LS_R_T2B )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean BG_SaberInReturn( int move )
{
	return PM_SaberInReturn( move );
}

qboolean PM_InSaberAnim( int anim )
{
	if ( (anim) >= BOTH_A1_T__B_ && (anim) <= BOTH_H1_S1_BR )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean PM_PainAnim( int anim )
{
	switch ( (anim) )
	{
		case BOTH_PAIN1:				//# First take pain anim
		case BOTH_PAIN2:				//# Second take pain anim
		case BOTH_PAIN3:				//# Third take pain anim
		case BOTH_PAIN4:				//# Fourth take pain anim
		case BOTH_PAIN5:				//# Fifth take pain anim - from behind
		case BOTH_PAIN6:				//# Sixth take pain anim - from behind
		case BOTH_PAIN7:				//# Seventh take pain anim - from behind
		case BOTH_PAIN8:				//# Eigth take pain anim - from behind
		case BOTH_PAIN9:				//#
		case BOTH_PAIN10:			//#
		case BOTH_PAIN11:			//#
		case BOTH_PAIN12:			//#
		case BOTH_PAIN13:			//#
		case BOTH_PAIN14:			//#
		case BOTH_PAIN15:			//#
		case BOTH_PAIN16:			//#
		case BOTH_PAIN17:			//#
		case BOTH_PAIN18:			//#
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean PM_JumpingAnim( int anim )
{
	switch ( (anim) )
	{
		case BOTH_JUMP1:				//# Jump - wind-up and leave ground
		case BOTH_INAIR1:			//# In air loop (from jump)
		case BOTH_LAND1:				//# Landing (from in air loop)
		case BOTH_LAND2:				//# Landing Hard (from a great height)
		case BOTH_JUMPBACK1:			//# Jump backwards - wind-up and leave ground
		case BOTH_INAIRBACK1:		//# In air loop (from jump back)
		case BOTH_LANDBACK1:			//# Landing backwards(from in air loop)
		case BOTH_JUMPLEFT1:			//# Jump left - wind-up and leave ground
		case BOTH_INAIRLEFT1:		//# In air loop (from jump left)
		case BOTH_LANDLEFT1:			//# Landing left(from in air loop)
		case BOTH_JUMPRIGHT1:		//# Jump right - wind-up and leave ground
		case BOTH_INAIRRIGHT1:		//# In air loop (from jump right)
		case BOTH_LANDRIGHT1:		//# Landing right(from in air loop)
		case BOTH_FORCEJUMP1:				//# Jump - wind-up and leave ground
		case BOTH_FORCEINAIR1:			//# In air loop (from jump)
		case BOTH_FORCELAND1:				//# Landing (from in air loop)
		case BOTH_FORCEJUMPBACK1:			//# Jump backwards - wind-up and leave ground
		case BOTH_FORCEINAIRBACK1:		//# In air loop (from jump back)
		case BOTH_FORCELANDBACK1:			//# Landing backwards(from in air loop)
		case BOTH_FORCEJUMPLEFT1:			//# Jump left - wind-up and leave ground
		case BOTH_FORCEINAIRLEFT1:		//# In air loop (from jump left)
		case BOTH_FORCELANDLEFT1:			//# Landing left(from in air loop)
		case BOTH_FORCEJUMPRIGHT1:		//# Jump right - wind-up and leave ground
		case BOTH_FORCEINAIRRIGHT1:		//# In air loop (from jump right)
		case BOTH_FORCELANDRIGHT1:		//# Landing right(from in air loop)
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean PM_LandingAnim( int anim )
{
	switch ( (anim) )
	{
		case BOTH_LAND1:				//# Landing (from in air loop)
		case BOTH_LAND2:				//# Landing Hard (from a great height)
		case BOTH_LANDBACK1:			//# Landing backwards(from in air loop)
		case BOTH_LANDLEFT1:			//# Landing left(from in air loop)
		case BOTH_LANDRIGHT1:		//# Landing right(from in air loop)
		case BOTH_FORCELAND1:		//# Landing (from in air loop)
		case BOTH_FORCELANDBACK1:	//# Landing backwards(from in air loop)
		case BOTH_FORCELANDLEFT1:	//# Landing left(from in air loop)
		case BOTH_FORCELANDRIGHT1:	//# Landing right(from in air loop)
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean PM_SpinningAnim( int anim )
{
	/*
	switch ( anim )
	{
	//FIXME: list any other spinning anims
	default:
		break;
	}
	*/
	return BG_SpinningSaberAnim( anim );
}

qboolean PM_InOnGroundAnim ( int anim )
{
	switch( anim )
	{
	case BOTH_DEAD1:
	case BOTH_DEAD2:
	case BOTH_DEAD3:
	case BOTH_DEAD4:
	case BOTH_DEAD5:
	case BOTH_DEADFORWARD1:
	case BOTH_DEADBACKWARD1:
	case BOTH_DEADFORWARD2:
	case BOTH_DEADBACKWARD2:
	case BOTH_LYINGDEATH1:
	case BOTH_LYINGDEAD1:
	case BOTH_SLEEP1:			//# laying on back-rknee up-rhand on torso
	case BOTH_KNOCKDOWN1:		//#
	case BOTH_KNOCKDOWN2:		//#
	case BOTH_KNOCKDOWN3:		//#
	case BOTH_KNOCKDOWN4:		//#
	case BOTH_KNOCKDOWN5:		//#
	case BOTH_GETUP1:
	case BOTH_GETUP2:
	case BOTH_GETUP3:
	case BOTH_GETUP4:
	case BOTH_GETUP5:
	case BOTH_GETUP_CROUCH_F1:
	case BOTH_GETUP_CROUCH_B1:
	case BOTH_FORCE_GETUP_F1:
	case BOTH_FORCE_GETUP_F2:
	case BOTH_FORCE_GETUP_B1:
	case BOTH_FORCE_GETUP_B2:
	case BOTH_FORCE_GETUP_B3:
	case BOTH_FORCE_GETUP_B4:
	case BOTH_FORCE_GETUP_B5:
	case BOTH_FORCE_GETUP_B6:
	case BOTH_GETUP_BROLL_B:
	case BOTH_GETUP_BROLL_F:
	case BOTH_GETUP_BROLL_L:
	case BOTH_GETUP_BROLL_R:
	case BOTH_GETUP_FROLL_B:
	case BOTH_GETUP_FROLL_F:
	case BOTH_GETUP_FROLL_L:
	case BOTH_GETUP_FROLL_R:
		return qtrue;
		break;
	}

	return qfalse;
}

qboolean BG_SuperBreakLoseAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_LK_S_DL_S_SB_1_L:	//super break I lost
	case BOTH_LK_S_DL_T_SB_1_L:	//super break I lost
	case BOTH_LK_S_ST_S_SB_1_L:	//super break I lost
	case BOTH_LK_S_ST_T_SB_1_L:	//super break I lost
	case BOTH_LK_S_S_S_SB_1_L:	//super break I lost
	case BOTH_LK_S_S_T_SB_1_L:	//super break I lost
	case BOTH_LK_DL_DL_S_SB_1_L:	//super break I lost
	case BOTH_LK_DL_DL_T_SB_1_L:	//super break I lost
	case BOTH_LK_DL_ST_S_SB_1_L:	//super break I lost
	case BOTH_LK_DL_ST_T_SB_1_L:	//super break I lost
	case BOTH_LK_DL_S_S_SB_1_L:	//super break I lost
	case BOTH_LK_DL_S_T_SB_1_L:	//super break I lost
	case BOTH_LK_ST_DL_S_SB_1_L:	//super break I lost
	case BOTH_LK_ST_DL_T_SB_1_L:	//super break I lost
	case BOTH_LK_ST_ST_S_SB_1_L:	//super break I lost
	case BOTH_LK_ST_ST_T_SB_1_L:	//super break I lost
	case BOTH_LK_ST_S_S_SB_1_L:	//super break I lost
	case BOTH_LK_ST_S_T_SB_1_L:	//super break I lost
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_SuperBreakWinAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_LK_S_DL_S_SB_1_W:	//super break I won
	case BOTH_LK_S_DL_T_SB_1_W:	//super break I won
	case BOTH_LK_S_ST_S_SB_1_W:	//super break I won
	case BOTH_LK_S_ST_T_SB_1_W:	//super break I won
	case BOTH_LK_S_S_S_SB_1_W:	//super break I won
	case BOTH_LK_S_S_T_SB_1_W:	//super break I won
	case BOTH_LK_DL_DL_S_SB_1_W:	//super break I won
	case BOTH_LK_DL_DL_T_SB_1_W:	//super break I won
	case BOTH_LK_DL_ST_S_SB_1_W:	//super break I won
	case BOTH_LK_DL_ST_T_SB_1_W:	//super break I won
	case BOTH_LK_DL_S_S_SB_1_W:	//super break I won
	case BOTH_LK_DL_S_T_SB_1_W:	//super break I won
	case BOTH_LK_ST_DL_S_SB_1_W:	//super break I won
	case BOTH_LK_ST_DL_T_SB_1_W:	//super break I won
	case BOTH_LK_ST_ST_S_SB_1_W:	//super break I won
	case BOTH_LK_ST_ST_T_SB_1_W:	//super break I won
	case BOTH_LK_ST_S_S_SB_1_W:	//super break I won
	case BOTH_LK_ST_S_T_SB_1_W:	//super break I won
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean BG_SaberLockBreakAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_BF1BREAK:
	case BOTH_BF2BREAK:
	case BOTH_CWCIRCLEBREAK:
	case BOTH_CCWCIRCLEBREAK:
	case BOTH_LK_S_DL_S_B_1_L:	//normal break I lost
	case BOTH_LK_S_DL_S_B_1_W:	//normal break I won
	case BOTH_LK_S_DL_T_B_1_L:	//normal break I lost
	case BOTH_LK_S_DL_T_B_1_W:	//normal break I won
	case BOTH_LK_S_ST_S_B_1_L:	//normal break I lost
	case BOTH_LK_S_ST_S_B_1_W:	//normal break I won
	case BOTH_LK_S_ST_T_B_1_L:	//normal break I lost
	case BOTH_LK_S_ST_T_B_1_W:	//normal break I won
	case BOTH_LK_S_S_S_B_1_L:	//normal break I lost
	case BOTH_LK_S_S_S_B_1_W:	//normal break I won
	case BOTH_LK_S_S_T_B_1_L:	//normal break I lost
	case BOTH_LK_S_S_T_B_1_W:	//normal break I won
	case BOTH_LK_DL_DL_S_B_1_L:	//normal break I lost
	case BOTH_LK_DL_DL_S_B_1_W:	//normal break I won
	case BOTH_LK_DL_DL_T_B_1_L:	//normal break I lost
	case BOTH_LK_DL_DL_T_B_1_W:	//normal break I won
	case BOTH_LK_DL_ST_S_B_1_L:	//normal break I lost
	case BOTH_LK_DL_ST_S_B_1_W:	//normal break I won
	case BOTH_LK_DL_ST_T_B_1_L:	//normal break I lost
	case BOTH_LK_DL_ST_T_B_1_W:	//normal break I won
	case BOTH_LK_DL_S_S_B_1_L:	//normal break I lost
	case BOTH_LK_DL_S_S_B_1_W:	//normal break I won
	case BOTH_LK_DL_S_T_B_1_L:	//normal break I lost
	case BOTH_LK_DL_S_T_B_1_W:	//normal break I won
	case BOTH_LK_ST_DL_S_B_1_L:	//normal break I lost
	case BOTH_LK_ST_DL_S_B_1_W:	//normal break I won
	case BOTH_LK_ST_DL_T_B_1_L:	//normal break I lost
	case BOTH_LK_ST_DL_T_B_1_W:	//normal break I won
	case BOTH_LK_ST_ST_S_B_1_L:	//normal break I lost
	case BOTH_LK_ST_ST_S_B_1_W:	//normal break I won
	case BOTH_LK_ST_ST_T_B_1_L:	//normal break I lost
	case BOTH_LK_ST_ST_T_B_1_W:	//normal break I won
	case BOTH_LK_ST_S_S_B_1_L:	//normal break I lost
	case BOTH_LK_ST_S_S_B_1_W:	//normal break I won
	case BOTH_LK_ST_S_T_B_1_L:	//normal break I lost
	case BOTH_LK_ST_S_T_B_1_W:	//normal break I won
		return qtrue;
		break;
	}
	return (BG_SuperBreakLoseAnim(anim)||BG_SuperBreakWinAnim(anim));
}

qboolean BG_FullBodyTauntAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_GESTURE1:
	case BOTH_DUAL_TAUNT:
	case BOTH_STAFF_TAUNT:
	case BOTH_BOW:
	case BOTH_MEDITATE:
	case BOTH_SHOWOFF_FAST:
	case BOTH_SHOWOFF_MEDIUM:
	case BOTH_SHOWOFF_STRONG:
	case BOTH_SHOWOFF_DUAL:
	case BOTH_SHOWOFF_STAFF:
	case BOTH_VICTORY_FAST:
	case BOTH_VICTORY_MEDIUM:
	case BOTH_VICTORY_STRONG:
	case BOTH_VICTORY_DUAL:
	case BOTH_VICTORY_STAFF:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean PM_SaberInTransition( int move )
{
	if ( move >= LS_T1_BR__R && move <= LS_T1_BL__L )
	{
		return qtrue;
	}
	return qfalse;
}

qboolean BG_SaberInTransitionAny( int move )
{
	if ( PM_SaberInStart( move ) )
	{
		return qtrue;
	}
	else if ( PM_SaberInTransition( move ) )
	{
		return qtrue;
	}
	else if ( PM_SaberInReturn( move ) )
	{
		return qtrue;
	}
	return qfalse;
}

/* --- playerState_t* anim-state predicates ---
 * BG_InRoll / PM_InKnockDown / PM_InRollComplete / PM_CanRollFromSoulCal read
 * only ps->legsAnim and ps->legsTimer. Rather than transcribe the ~150-field
 * playerState_t (whose layout is verified separately in q_shared_h_oracle.c),
 * use a minimal local struct with just those two fields -- the verbatim fn
 * bodies below behave identically. The jka_* wrappers marshal plain ints from
 * the Rust test into the minimal struct, so no struct layout is shared across
 * the FFI boundary. (BG_InKnockDownOnGround is omitted -- though now ported, it
 * calls BG_AnimLength, which reads the bgAllAnims global animation table rather
 * than plain ints, so it has no standalone oracle here -- same as the other
 * global/trap-bound ports.) */
typedef struct {
	int legsTimer;
	int legsAnim;
} ps_min_t;
#define playerState_t ps_min_t

/* verbatim from bg_panimate.c */
qboolean BG_InRoll( playerState_t *ps, int anim )
{
	switch ( (anim) )
	{
	case BOTH_GETUP_BROLL_B:
	case BOTH_GETUP_BROLL_F:
	case BOTH_GETUP_BROLL_L:
	case BOTH_GETUP_BROLL_R:
	case BOTH_GETUP_FROLL_B:
	case BOTH_GETUP_FROLL_F:
	case BOTH_GETUP_FROLL_L:
	case BOTH_GETUP_FROLL_R:
	case BOTH_ROLL_F:
	case BOTH_ROLL_B:
	case BOTH_ROLL_R:
	case BOTH_ROLL_L:
		if ( ps->legsTimer > 0 )
		{
			return qtrue;
		}
		break;
	}
	return qfalse;
}

qboolean PM_InKnockDown( playerState_t *ps )
{
	switch ( (ps->legsAnim) )
	{
	case BOTH_KNOCKDOWN1:
	case BOTH_KNOCKDOWN2:
	case BOTH_KNOCKDOWN3:
	case BOTH_KNOCKDOWN4:
	case BOTH_KNOCKDOWN5:
		return qtrue;
		break;
	case BOTH_GETUP1:
	case BOTH_GETUP2:
	case BOTH_GETUP3:
	case BOTH_GETUP4:
	case BOTH_GETUP5:
	case BOTH_FORCE_GETUP_F1:
	case BOTH_FORCE_GETUP_F2:
	case BOTH_FORCE_GETUP_B1:
	case BOTH_FORCE_GETUP_B2:
	case BOTH_FORCE_GETUP_B3:
	case BOTH_FORCE_GETUP_B4:
	case BOTH_FORCE_GETUP_B5:
	case BOTH_GETUP_BROLL_B:
	case BOTH_GETUP_BROLL_F:
	case BOTH_GETUP_BROLL_L:
	case BOTH_GETUP_BROLL_R:
	case BOTH_GETUP_FROLL_B:
	case BOTH_GETUP_FROLL_F:
	case BOTH_GETUP_FROLL_L:
	case BOTH_GETUP_FROLL_R:
		if ( ps->legsTimer )
		{
			return qtrue;
		}
		break;
	}
	return qfalse;
}

qboolean PM_InRollComplete( playerState_t *ps, int anim )
{
	switch ( (anim) )
	{
	case BOTH_ROLL_F:
	case BOTH_ROLL_B:
	case BOTH_ROLL_R:
	case BOTH_ROLL_L:
		if ( ps->legsTimer < 1 )
		{
			return qtrue;
		}
		break;
	}
	return qfalse;
}

qboolean PM_CanRollFromSoulCal( playerState_t *ps )
{
	if ( ps->legsAnim == BOTH_A7_SOULCAL
		&& ps->legsTimer < 700
		&& ps->legsTimer > 250 )
	{
		return qtrue;
	}
	return qfalse;
}

#undef playerState_t

/* int-marshalling wrappers for the Rust parity test */
int jka_BG_InRoll( int legsTimer, int legsAnim, int anim )
{
	ps_min_t ps; ps.legsTimer = legsTimer; ps.legsAnim = legsAnim;
	return BG_InRoll( &ps, anim );
}

int jka_PM_InKnockDown( int legsTimer, int legsAnim )
{
	ps_min_t ps; ps.legsTimer = legsTimer; ps.legsAnim = legsAnim;
	return PM_InKnockDown( &ps );
}

int jka_PM_InRollComplete( int legsTimer, int legsAnim, int anim )
{
	ps_min_t ps; ps.legsTimer = legsTimer; ps.legsAnim = legsAnim;
	return PM_InRollComplete( &ps, anim );
}

int jka_PM_CanRollFromSoulCal( int legsTimer, int legsAnim )
{
	ps_min_t ps; ps.legsTimer = legsTimer; ps.legsAnim = legsAnim;
	return PM_CanRollFromSoulCal( &ps );
}
