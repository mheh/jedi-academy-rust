/* Oracle extraction of g_active.c's pure bitmask/enum classifiers
 * G_StandingAnim (refs/raven-jediacademy/codemp/game/g_active.c:1251) and
 * G_ActionButtonPressed (g_active.c:1265). Both take a plain int and return a
 * qboolean (int), reading no globals/structs, so this TU is fully self-contained.
 *
 * The BOTH_STAND* anim enumerators come from the AUTHENTIC, unmodified Raven
 * header (`#include "anims.h"`, the same -I as anims_oracle.c — supplied per-file
 * in build.rs), so the C computing them is independent of the Rust port. The
 * function bodies are reproduced exactly. `jka_` prefix avoids colliding with the
 * test binary.
 *
 * Built only under the `oracle` cargo feature. */

#include "anims.h"

typedef int qboolean;
#define qfalse 0
#define qtrue 1

#define BUTTON_ATTACK         1
#define BUTTON_USE_HOLDABLE   4
#define BUTTON_GESTURE        8
#define BUTTON_USE            32
#define BUTTON_FORCEGRIP      64
#define BUTTON_ALT_ATTACK     128
#define BUTTON_FORCEPOWER     512
#define BUTTON_FORCE_LIGHTNING 1024
#define BUTTON_FORCE_DRAIN    2048

qboolean jka_G_StandingAnim( int anim )
{//NOTE: does not check idles or special (cinematic) stands
	switch ( anim )
	{
	case BOTH_STAND1:
	case BOTH_STAND2:
	case BOTH_STAND3:
	case BOTH_STAND4:
		return qtrue;
		break;
	}
	return qfalse;
}

qboolean jka_G_ActionButtonPressed(int buttons)
{
	if (buttons & BUTTON_ATTACK)
	{
		return qtrue;
	}
	else if (buttons & BUTTON_USE_HOLDABLE)
	{
		return qtrue;
	}
	else if (buttons & BUTTON_GESTURE)
	{
		return qtrue;
	}
	else if (buttons & BUTTON_USE)
	{
		return qtrue;
	}
	else if (buttons & BUTTON_FORCEGRIP)
	{
		return qtrue;
	}
	else if (buttons & BUTTON_ALT_ATTACK)
	{
		return qtrue;
	}
	else if (buttons & BUTTON_FORCEPOWER)
	{
		return qtrue;
	}
	else if (buttons & BUTTON_FORCE_LIGHTNING)
	{
		return qtrue;
	}
	else if (buttons & BUTTON_FORCE_DRAIN)
	{
		return qtrue;
	}

	return qfalse;
}

/* ----------------------------------------------------------------------------
 * G_AddPushVecToUcmd (g_active.c:1216) — pure vector math, extracted to take the
 * exact scalar/vector inputs it reads off self->client / ucmd / level rather than
 * mirroring the whole gentity_t/usercmd_t. The body is reproduced verbatim with the
 * authentic non-VM q_shared.h vector macros (inline-arithmetic forms) plus the real
 * AngleVectors/VectorNormalize/VectorLengthSquared linked from q_math_oracle.c.
 *
 * `ucmd->forwardmove`/`rightmove` are `signed char`, so the in/out params are
 * `signed char *` to reproduce the floor()->char truncation bit-exact. `pushVec` is
 * in/out (may be VectorClear'd). `speed` is in/out.
 * -------------------------------------------------------------------------- */

typedef float vec_t;
typedef vec_t vec3_t[3];

extern void AngleVectors( const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up );
extern vec_t VectorNormalize( vec3_t v );
extern vec_t VectorLengthSquared( const vec3_t v );

#define DotProduct(x,y)			((x)[0]*(y)[0]+(x)[1]*(y)[1]+(x)[2]*(y)[2])
#define VectorAdd(a,b,c)		((c)[0]=(a)[0]+(b)[0],(c)[1]=(a)[1]+(b)[1],(c)[2]=(a)[2]+(b)[2])
#define VectorScale(v, s, o)	((o)[0]=(v)[0]*(s),(o)[1]=(v)[1]*(s),(o)[2]=(v)[2]*(s))
#define VectorMA(v, s, b, o)	((o)[0]=(v)[0]+(b)[0]*(s),(o)[1]=(v)[1]+(b)[1]*(s),(o)[2]=(v)[2]+(b)[2]*(s))
#define VectorClear(a)			((a)[0]=(a)[1]=(a)[2]=0)

#include <math.h>
#include <stddef.h>

void jka_G_AddPushVecToUcmd( float *pushVec, const float *viewangles, float *speed,
							 signed char *forwardmove, signed char *rightmove,
							 int pushVecTime, int levelTime )
{
	vec3_t	forward, right, moveDir;
	float	pushSpeed, fMove, rMove;

	pushSpeed = VectorLengthSquared(pushVec);
	if(!pushSpeed)
	{//not being pushed
		return;
	}

	AngleVectors(viewangles, forward, right, NULL);
	VectorScale(forward, *forwardmove/127.0f * *speed, moveDir);
	VectorMA(moveDir, *rightmove/127.0f * *speed, right, moveDir);
	//moveDir is now our intended move velocity

	VectorAdd(moveDir, pushVec, moveDir);
	*speed = VectorNormalize(moveDir);
	//moveDir is now our intended move velocity plus our push Vector

	fMove = 127.0 * DotProduct(forward, moveDir);
	rMove = 127.0 * DotProduct(right, moveDir);
	*forwardmove = floor(fMove);//If in the same dir , will be positive
	*rightmove = floor(rMove);//If in the same dir , will be positive

	if ( pushVecTime < levelTime )
	{
		VectorClear( pushVec );
	}
}

/* ----------------------------------------------------------------------------
 * ClientTimerActions (g_active.c:787) — pure field arithmetic, extracted to the
 * scalar fields it touches (client->timeResidual, ent->health, the two STAT_* slots
 * of client->ps.stats). Reproduced verbatim. No external callees.
 * -------------------------------------------------------------------------- */
void jka_ClientTimerActions( int msec, int *timeResidual, int *health,
							 int stat_max_health, int *stat_armor )
{
	*timeResidual += msec;

	while ( *timeResidual >= 1000 )
	{
		*timeResidual -= 1000;

		// count down health when over max
		if ( *health > stat_max_health ) {
			(*health)--;
		}

		// count down armor when over max
		if ( *stat_armor > stat_max_health ) {
			(*stat_armor)--;
		}
	}
}
