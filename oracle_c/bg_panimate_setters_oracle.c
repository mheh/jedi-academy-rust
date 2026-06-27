/*
 * Oracle TU for the bg_panimate.c animation-SETTER cluster (lines 1633-2965):
 * BG_FlipPart, the legs/torso anim starters and timer setters,
 * BG_SaberStartTransAnim, BG_SetAnimFinal and BG_SetAnim. Each function body is
 * copied VERBATIM from raven-jediacademy/codemp/game/bg_panimate.c (and the two anim
 * classifiers PM_RunningAnim / PM_WalkingAnim from bg_pmove.c) so the C compiler
 * evaluates the real control flow + float timer math independently of the Rust
 * port in src/codemp/game/bg_panimate.rs.
 *
 * Build config mirrors the QAGAME game module:
 *   - QAGAME defined -> the `g_entities[ps->clientNum].s.{legs,torso}Anim`
 *     restart-detect branches in BG_Start{Legs,Torso}Anim are compiled in.
 *   - NDEBUG defined -> assert() is a no-op, so the verbatim BG_StartLegsAnim /
 *     BG_StartTorsoAnim do not pull in BG_InDeathAnim. The Rust port mirrors
 *     those asserts as debug_assert!(...); the parity tests keep them satisfied
 *     (no death anim is fed while pm_type >= PM_DEAD).
 *
 * The authentic Raven anims.h is #include'd directly for the BOTH_* constants
 * (clang-clean pure enum; -I supplied per-file in build.rs). All non-anim
 * constants (SETANIM_*, PM_DEAD, MAX_CLIENTS, FORCE_LEVEL_*, FP_*, BROKENLIMB_*)
 * are transcribed below from bg_public.h / q_shared.h -- the same literal values
 * the Rust port uses (each independently oracle-verified in its home module).
 *
 * Minimal struct models stand in for playerState_t / animation_t / the gentity
 * array: the bodies only touch the named scalar fields, and no struct crosses
 * FFI -- the wrappers marshal flat int/float arrays. animation_t keeps the
 * authentic #pragma pack(1) field types (unsigned short / short / signed char)
 * so the firstFrame/numFrames/frameLerp arithmetic is bit-identical.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

#define QAGAME
#define NDEBUG
#include <assert.h>
#include <math.h>
#include <string.h>

#include "anims.h"

typedef int qboolean;
#define qtrue 1
#define qfalse 0

/* --- constants (bg_public.h / q_shared.h), same literal values as the port --- */
#define SETANIM_TORSO 1
#define SETANIM_LEGS 2
#define SETANIM_FLAG_OVERRIDE 1
#define SETANIM_FLAG_HOLD 2
#define SETANIM_FLAG_RESTART 4
#define SETANIM_FLAG_HOLDLESS 8
#define PM_DEAD 5
#define MAX_CLIENTS 32
#define FORCE_LEVEL_1 1
#define FORCE_LEVEL_3 3
#define FP_SPEED 2
#define FP_RAGE 8
#define BROKENLIMB_LARM 1
#define BROKENLIMB_RARM 2
#define WP_NONE 0
#define WP_SABER 3

/* --- minimal struct models --- */
#pragma pack(push, 1)
typedef struct {
	unsigned short firstFrame;
	unsigned short numFrames;
	short frameLerp;
	signed char loopFrames;
} anim_pan_t;
#pragma pack(pop)

typedef struct {
	int saberAnimLevel;
	int forcePowersActive;
} fd_pan_t;

typedef struct {
	int pm_type;
	int clientNum;
	int legsTimer;
	int torsoTimer;
	int legsAnim;
	int torsoAnim;
	int legsFlip;
	int torsoFlip;
	int brokenLimbs;
	int weapon;
	fd_pan_t fd;
} ps_pan_t;

/* Minimal saberInfo_t model for BG_SaberStartTransAnim's PC `weapon==WP_SABER`
 * per-saber animSpeedScale path. The parity tests drive weapon=WP_NONE (and
 * load_ps leaves ps->weapon==0), so this stub is never entered at runtime; the
 * live BG_MySaber/animSpeedScale logic is verified by bg_saberLoad's own tests.
 * Returning NULL keeps the verbatim PC body compilable+linkable here. */
typedef struct {
	float animSpeedScale;
} saberInfo_t;

static saberInfo_t *BG_MySaber(int clientNum, int saberNum)
{
	(void)clientNum;
	(void)saberNum;
	return 0;
}

typedef struct {
	int legsAnim;
	int torsoAnim;
} es_pan_t;

typedef struct {
	es_pan_t s;
} gent_pan_t;

#define playerState_t ps_pan_t
#define animation_t anim_pan_t

/* QAGAME branch reads g_entities[clientNum].s.{legs,torso}Anim */
static gent_pan_t g_entities[64];

/* ===== verbatim bodies ===== */

void BG_FlipPart(playerState_t *ps, int part)
{
	if (part == SETANIM_TORSO)
	{
		if (ps->torsoFlip)
		{
			ps->torsoFlip = qfalse;
		}
		else
		{
			ps->torsoFlip = qtrue;
		}
	}
	else if (part == SETANIM_LEGS)
	{
		if (ps->legsFlip)
		{
			ps->legsFlip = qfalse;
		}
		else
		{
			ps->legsFlip = qtrue;
		}
	}
}

static void BG_StartLegsAnim( playerState_t *ps, int anim )
{
	if ( ps->pm_type >= PM_DEAD )
	{
		assert(!BG_InDeathAnim(anim));
		if (ps->clientNum < MAX_CLIENTS || anim != BOTH_VT_DEATH1)
		{
			return;
		}
	}
	if ( ps->legsTimer > 0 )
	{
		return;		// a high priority animation is running
	}

	if (ps->legsAnim == anim)
	{
		BG_FlipPart(ps, SETANIM_LEGS);
	}
#ifdef QAGAME
	else if (g_entities[ps->clientNum].s.legsAnim == anim)
	{
		BG_FlipPart(ps, SETANIM_LEGS);
	}
#endif
	ps->legsAnim = anim;
}

void BG_StartTorsoAnim( playerState_t *ps, int anim )
{
	if ( ps->pm_type >= PM_DEAD )
	{
		assert(!BG_InDeathAnim(anim));
		return;
	}

	if (ps->torsoAnim == anim)
	{
		BG_FlipPart(ps, SETANIM_TORSO);
	}
#ifdef QAGAME
	else if (g_entities[ps->clientNum].s.torsoAnim == anim)
	{
		BG_FlipPart(ps, SETANIM_TORSO);
	}
#endif
	ps->torsoAnim = anim;
}

void BG_SetLegsAnimTimer(playerState_t *ps, int time)
{
	ps->legsTimer = time;

	if (ps->legsTimer < 0 && time != -1 )
	{
		ps->legsTimer = 0;
	}
}

void BG_SetTorsoAnimTimer(playerState_t *ps, int time )
{
	ps->torsoTimer = time;

	if (ps->torsoTimer < 0 && time != -1 )
	{
		ps->torsoTimer = 0;
	}
}

qboolean PM_InSaberAnim( int anim )
{
	if ( (anim) >= BOTH_A1_T__B_ && (anim) <= BOTH_H1_S1_BR )
	{
		return qtrue;
	}
	return qfalse;
}

void BG_SaberStartTransAnim( int clientNum, int saberAnimLevel, int weapon, int anim, float *animSpeed, int broken )
{
	if ( anim >= BOTH_A1_T__B_ && anim <= BOTH_ROLL_STAB )
	{
		if ( weapon == WP_SABER )
		{
			saberInfo_t *saber = BG_MySaber( clientNum, 0 );
			if ( saber
				&& saber->animSpeedScale != 1.0f )
			{
				*animSpeed *= saber->animSpeedScale;
			}
			saber = BG_MySaber( clientNum, 1 );
			if ( saber
				&& saber->animSpeedScale != 1.0f )
			{
				*animSpeed *= saber->animSpeedScale;
			}
		}
	}

	if ( ( (anim) >= BOTH_T1_BR__R &&
		(anim) <= BOTH_T1_BL_TL ) ||
		( (anim) >= BOTH_T2_BR__R &&
		(anim) <= BOTH_T2_BL_TL ) ||
		( (anim) >= BOTH_T3_BR__R &&
		(anim) <= BOTH_T3_BL_TL ) )
	{
		if ( saberAnimLevel == FORCE_LEVEL_1 )
		{
			*animSpeed *= 1.5f;
		}
		else if ( saberAnimLevel == FORCE_LEVEL_3 )
		{
			*animSpeed *= 0.75f;
		}

		if (broken & (1<<BROKENLIMB_RARM))
		{
			*animSpeed *= 0.5f;
		}
		else if (broken & (1<<BROKENLIMB_LARM))
		{
			*animSpeed *= 0.65f;
		}
	}
	else if (broken && PM_InSaberAnim(anim))
	{
		if (broken & (1<<BROKENLIMB_RARM))
		{
			*animSpeed *= 0.5f;
		}
		else if (broken & (1<<BROKENLIMB_LARM))
		{
			*animSpeed *= 0.65f;
		}
	}
}

qboolean PM_RunningAnim( int anim )
{
	switch ( (anim) )
	{
	case BOTH_RUN1:
	case BOTH_RUN2:
	case BOTH_RUN_STAFF:
	case BOTH_RUN_DUAL:
	case BOTH_RUNBACK1:
	case BOTH_RUNBACK2:
	case BOTH_RUNBACK_STAFF:
	case BOTH_RUNBACK_DUAL:
	case BOTH_RUN1START:
	case BOTH_RUN1STOP:
	case BOTH_RUNSTRAFE_LEFT1:
	case BOTH_RUNSTRAFE_RIGHT1:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean PM_WalkingAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_WALK1:
	case BOTH_WALK2:
	case BOTH_WALK_STAFF:
	case BOTH_WALK_DUAL:
	case BOTH_WALK5:
	case BOTH_WALK6:
	case BOTH_WALK7:
	case BOTH_WALKBACK1:
	case BOTH_WALKBACK2:
	case BOTH_WALKBACK_STAFF:
	case BOTH_WALKBACK_DUAL:
		return qtrue;
		break;
	}
	return qfalse;
}

void BG_SetAnimFinal(playerState_t *ps, animation_t *animations,
					 int setAnimParts,int anim,int setAnimFlags,
					 int blendTime)
{
	float editAnimSpeed = 1;

	if (!animations)
	{
		return;
	}

	assert(anim > -1);
	assert(animations[anim].firstFrame > 0 || animations[anim].numFrames > 0);

	blendTime = 0;

	BG_SaberStartTransAnim(ps->clientNum, ps->fd.saberAnimLevel, ps->weapon, anim, &editAnimSpeed, ps->brokenLimbs);

	if (setAnimParts & SETANIM_TORSO)
	{
		if( !(setAnimFlags & SETANIM_FLAG_RESTART) && (ps->torsoAnim) == anim )
		{
			goto setAnimLegs;
		}
		if( !(setAnimFlags & SETANIM_FLAG_OVERRIDE) && ((ps->torsoTimer > 0)||(ps->torsoTimer == -1)) )
		{
			goto setAnimLegs;
		}

		BG_StartTorsoAnim(ps, anim);

		if (setAnimFlags & SETANIM_FLAG_HOLD)
		{
			if (setAnimFlags & SETANIM_FLAG_HOLDLESS)
			{
				int dur;
				int speedDif;

				dur = (animations[anim].numFrames-1) * fabs((float)(animations[anim].frameLerp));
				speedDif = dur - (dur * editAnimSpeed);
				dur += speedDif;
				if (dur > 1)
				{
					ps->torsoTimer = dur-1;
				}
				else
				{
					ps->torsoTimer = fabs((float)(animations[anim].frameLerp));
				}
			}
			else
			{
				ps->torsoTimer = ((animations[anim].numFrames ) * fabs((float)(animations[anim].frameLerp)));
			}

			if (ps->fd.forcePowersActive & (1 << FP_RAGE))
			{
				ps->torsoTimer /= 1.7;
			}
		}
	}

setAnimLegs:
	if (setAnimParts & SETANIM_LEGS)
	{
		if( !(setAnimFlags & SETANIM_FLAG_RESTART) && (ps->legsAnim) == anim )
		{
			goto setAnimDone;
		}
		if( !(setAnimFlags & SETANIM_FLAG_OVERRIDE) && ((ps->legsTimer > 0)||(ps->legsTimer == -1)) )
		{
			goto setAnimDone;
		}

		BG_StartLegsAnim(ps, anim);

		if (setAnimFlags & SETANIM_FLAG_HOLD)
		{
			if (setAnimFlags & SETANIM_FLAG_HOLDLESS)
			{
				int dur;
				int speedDif;

				dur = (animations[anim].numFrames-1) * fabs((float)(animations[anim].frameLerp));
				speedDif = dur - (dur * editAnimSpeed);
				dur += speedDif;
				if (dur > 1)
				{
					ps->legsTimer = dur-1;
				}
				else
				{
					ps->legsTimer = fabs((float)(animations[anim].frameLerp));
				}
			}
			else
			{
				ps->legsTimer = ((animations[anim].numFrames ) * fabs((float)(animations[anim].frameLerp)));
			}

			if (PM_RunningAnim(anim) ||
				PM_WalkingAnim(anim))
			{
				if (ps->fd.forcePowersActive & (1 << FP_RAGE))
				{
					ps->legsTimer /= 1.3;
				}
				else if (ps->fd.forcePowersActive & (1 << FP_SPEED))
				{
					ps->legsTimer /= 1.7;
				}
			}
		}
	}

setAnimDone:
	return;
}

void BG_SetAnim(playerState_t *ps, animation_t *animations, int setAnimParts,int anim,int setAnimFlags, int blendTime)
{
	if (!animations)
	{
		return; /* (port reads bgAllAnims[0]; the parity tests always pass non-NULL) */
	}

	if (animations[anim].firstFrame == 0 && animations[anim].numFrames == 0)
	{
		if (anim == BOTH_RUNBACK1 ||
			anim == BOTH_WALKBACK1 ||
			anim == BOTH_RUN1)
		{
			anim = BOTH_WALK2;
		}

		if (animations[anim].firstFrame == 0 && animations[anim].numFrames == 0)
		{
			return;
		}
	}

	if (setAnimFlags&SETANIM_FLAG_OVERRIDE)
	{
		if (setAnimParts & SETANIM_TORSO)
		{
			if( (setAnimFlags & SETANIM_FLAG_RESTART) || (ps->torsoAnim) != anim )
			{
				BG_SetTorsoAnimTimer(ps, 0);
			}
		}
		if (setAnimParts & SETANIM_LEGS)
		{
			if( (setAnimFlags & SETANIM_FLAG_RESTART) || (ps->legsAnim) != anim )
			{
				BG_SetLegsAnimTimer(ps, 0);
			}
		}
	}

	BG_SetAnimFinal(ps, animations, setAnimParts, anim, setAnimFlags, blendTime);
}

/* ===== int/float-marshalling wrappers for the Rust parity tests ===== */

/* in[] layout (22 ints):
 *  0 pm_type            1 clientNum          2 legsTimer          3 torsoTimer
 *  4 legsAnim           5 torsoAnim          6 legsFlip           7 torsoFlip
 *  8 brokenLimbs        9 saberAnimLevel    10 forcePowersActive 11 anim
 * 12 firstFrame        13 numFrames         14 frameLerp         15 ent_legsAnim
 * 16 ent_torsoAnim     17 setAnimParts      18 setAnimFlags      19 anim2 index
 * 20 anim2 firstFrame  21 anim2 numFrames
 * The optional second slot (anim2, index 19; skipped when < 0) lets BG_SetAnim's
 * droid-hack redirect target (BOTH_WALK2) carry valid frames.
 * out[] (6 ints): legsTimer torsoTimer legsAnim torsoAnim legsFlip torsoFlip */
static void load_ps(ps_pan_t *ps, anim_pan_t *anims, const int *in)
{
	memset(ps, 0, sizeof(*ps));
	ps->pm_type = in[0];
	ps->clientNum = in[1];
	ps->legsTimer = in[2];
	ps->torsoTimer = in[3];
	ps->legsAnim = in[4];
	ps->torsoAnim = in[5];
	ps->legsFlip = in[6];
	ps->torsoFlip = in[7];
	ps->brokenLimbs = in[8];
	ps->fd.saberAnimLevel = in[9];
	ps->fd.forcePowersActive = in[10];

	anims[in[11]].firstFrame = (unsigned short)in[12];
	anims[in[11]].numFrames = (unsigned short)in[13];
	anims[in[11]].frameLerp = (short)in[14];

	if (in[19] >= 0)
	{
		anims[in[19]].firstFrame = (unsigned short)in[20];
		anims[in[19]].numFrames = (unsigned short)in[21];
	}

	memset(g_entities, 0, sizeof(g_entities));
	g_entities[in[1]].s.legsAnim = in[15];
	g_entities[in[1]].s.torsoAnim = in[16];
}

static void store_ps(const ps_pan_t *ps, int *out)
{
	out[0] = ps->legsTimer;
	out[1] = ps->torsoTimer;
	out[2] = ps->legsAnim;
	out[3] = ps->torsoAnim;
	out[4] = ps->legsFlip;
	out[5] = ps->torsoFlip;
}

static anim_pan_t g_anims[2048];

void jka_BG_SetAnimFinal(const int *in, int *out)
{
	ps_pan_t ps;
	memset(g_anims, 0, sizeof(g_anims));
	load_ps(&ps, g_anims, in);
	BG_SetAnimFinal(&ps, g_anims, in[17], in[11], in[18], 350);
	store_ps(&ps, out);
}

void jka_BG_SetAnim(const int *in, int *out)
{
	ps_pan_t ps;
	memset(g_anims, 0, sizeof(g_anims));
	load_ps(&ps, g_anims, in);
	BG_SetAnim(&ps, g_anims, in[17], in[11], in[18], 350);
	store_ps(&ps, out);
}

float jka_BG_SaberStartTransAnim(int clientNum, int saberAnimLevel, int weapon, int anim, float animSpeed, int broken)
{
	BG_SaberStartTransAnim(clientNum, saberAnimLevel, weapon, anim, &animSpeed, broken);
	return animSpeed;
}

/* BG_HasAnimation (bg_panimate.c:2858), verbatim if-chain over a single-file
 * bgAllAnims model. `numFramesAtSlot` is bgAllAnims[animIndex].anims[animation]
 * .numFrames; numAllAnims is the bgNumAllAnims global (keep < 8 in tests).
 * MAX_ANIMATIONS comes from the #include'd anims.h enum. */
static anim_pan_t hasanim_slots[MAX_TOTALANIMATIONS];
static struct { anim_pan_t *anims; } bgAllAnims_model[8];

int jka_BG_HasAnimation(int animIndex, int animation, int numAllAnims, int numFramesAtSlot)
{
	int bgNumAllAnims = numAllAnims;
	int i;
	anim_pan_t *animations;

	memset(hasanim_slots, 0, sizeof(hasanim_slots));
	if (animation >= 0 && animation < MAX_TOTALANIMATIONS)
	{
		hasanim_slots[animation].numFrames = (unsigned short)numFramesAtSlot;
	}
	for (i = 0; i < 8; i++)
	{
		bgAllAnims_model[i].anims = hasanim_slots;
	}

	if ( animation < 0 || animation >= MAX_ANIMATIONS )
	{
		return qfalse;
	}

	if( animIndex < 0 || animIndex > bgNumAllAnims )
	{
		return qfalse;
	}

	animations = bgAllAnims_model[animIndex].anims;

	if ( animations[animation].numFrames == 0 )
	{
		return qfalse;
	}

	return qtrue;
}

/* BG_AnimLength (bg_panimate.c:1564) / PM_AnimLength (1575): both reduce to the
 * same numFrames * fabs((float)frameLerp) -> int arithmetic over a single
 * animation_t slot (BG reads bgAllAnims[index].anims[anim], PM reads
 * pm->animations[anim]). One verbatim body covers the shared math + the
 * `anim >= MAX_ANIMATIONS` guard; PM's extra `!pm->animations` / `anim < 0`
 * guards are exercised behaviorally on the Rust side. animation_t is anim_pan_t
 * (authentic #pragma pack(1) field types) so the unsigned-short -> float ->
 * double -> int chain is bit-identical. */
int jka_BG_AnimLength(int anim, int numFrames, int frameLerp)
{
	static anim_pan_t alen_slots[MAX_TOTALANIMATIONS];
	static struct { anim_pan_t *anims; } alen_model[1];
	memset(alen_slots, 0, sizeof(alen_slots));
	if (anim >= 0 && anim < MAX_TOTALANIMATIONS)
	{
		alen_slots[anim].numFrames = (unsigned short)numFrames;
		alen_slots[anim].frameLerp = (short)frameLerp;
	}
	alen_model[0].anims = alen_slots;

	if (anim >= MAX_ANIMATIONS)
	{
		return -1;
	}
	return alen_model[0].anims[anim].numFrames * fabs((float)(alen_model[0].anims[anim].frameLerp));
}

#undef playerState_t
#undef animation_t
