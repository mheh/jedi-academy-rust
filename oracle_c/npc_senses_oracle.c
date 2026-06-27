/* Oracle: functions extracted from the original raven-jediacademy/codemp/game/NPC_senses.c,
   compared against OpenJK/codemp/game/NPC_senses.c. Compiled and linked under the
   `oracle` cargo feature (see build.rs) so Rust tests can call the real C and assert
   the Rust port matches. Bodies are authentic Raven source; the only edits are the
   `oracle_` test-export prefixes (to avoid colliding with the Rust extern names) and
   wiring the q_math helpers via extern prototypes resolved against q_math_oracle.c in
   the same archive.

   These functions are pure float math (FOV / in-front position comparators) — no
   gentity/NPC state — so they reproduce bit-exact. VectorSubtract/DotProduct/VectorCopy
   come from the shim macros; vectoangles/AngleDelta/VectorNormalize/AngleVectors are the
   real q_math.c functions (extern-declared here, defined in q_math_oracle.c). */
#include "qshared_shim.h"
#include <math.h>

/* q_math.c functions — defined in q_math_oracle.c, linked in the same archive. */
extern void  vectoangles( const vec3_t value1, vec3_t angles );
extern float AngleDelta( float angle1, float angle2 );
extern vec_t VectorNormalize( vec3_t v );
extern void  AngleVectors( const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up );

qboolean oracle_InFront( vec3_t spot, vec3_t from, vec3_t fromAngles, float threshHold )
{
	vec3_t	dir, forward, angles;
	float	dot;

	VectorSubtract( spot, from, dir );
	dir[2] = 0;
	VectorNormalize( dir );

	VectorCopy( fromAngles, angles );
	angles[0] = 0;
	AngleVectors( angles, forward, NULL, NULL );

	dot = DotProduct( dir, forward );

	return (dot > threshHold);
}

qboolean oracle_InFOV3( vec3_t spot, vec3_t from, vec3_t fromAngles, int hFOV, int vFOV )
{
	vec3_t	deltaVector, angles, deltaAngles;

	VectorSubtract ( spot, from, deltaVector );
	vectoangles ( deltaVector, angles );

	deltaAngles[PITCH]	= AngleDelta ( fromAngles[PITCH], angles[PITCH] );
	deltaAngles[YAW]	= AngleDelta ( fromAngles[YAW], angles[YAW] );

	if ( fabs ( deltaAngles[PITCH] ) <= vFOV && fabs ( deltaAngles[YAW] ) <= hFOV )
	{
		return qtrue;
	}

	return qfalse;
}

float oracle_NPC_GetHFOVPercentage( vec3_t spot, vec3_t from, vec3_t facing, float hFOV )
{
	vec3_t	deltaVector, angles;
	float	delta;

	VectorSubtract ( spot, from, deltaVector );

	vectoangles ( deltaVector, angles );

	delta = fabs( AngleDelta ( facing[YAW], angles[YAW] ) );

	if ( delta > hFOV )
		return 0.0f;

	return ( ( hFOV - delta ) / hFOV );
}

float oracle_NPC_GetVFOVPercentage( vec3_t spot, vec3_t from, vec3_t facing, float vFOV )
{
	vec3_t	deltaVector, angles;
	float	delta;

	VectorSubtract ( spot, from, deltaVector );

	vectoangles ( deltaVector, angles );

	delta = fabs( AngleDelta ( facing[PITCH], angles[PITCH] ) );

	if ( delta > vFOV )
		return 0.0f;

	return ( ( vFOV - delta ) / vFOV );
}
