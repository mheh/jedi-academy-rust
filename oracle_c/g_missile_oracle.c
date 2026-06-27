/* Oracle extraction of g_missile.c's pure helpers — the ones whose behavior is
 * pure arithmetic over vec3_t and so can be parity-checked without any gentity /
 * trap / cvar context:
 *
 *   G_BounceProjectile (refs/raven-jediacademy/codemp/game/g_missile.c:277)
 *
 * Reproduced verbatim from the source and exposed under a `jka_` prefix so it does
 * not collide with the test binary's own symbols. The vec3 ops come from
 * qshared_shim.h (DotProduct/VectorSubtract/VectorMA — the q_shared.h macros);
 * VectorNormalize is the verbatim q_math.c function (in q_math_oracle.c, same
 * static lib), declared here. vec3_t in/out cross the FFI as float pointers. */
#include "qshared_shim.h"

/* q_math.c — same static lib (q_math_oracle.c). */
vec_t VectorNormalize( vec3_t v );

/* g_missile.c:277 — verbatim (renamed jka_G_BounceProjectile). */
void jka_G_BounceProjectile( vec3_t start, vec3_t impact, vec3_t dir, vec3_t endout ) {
	vec3_t v, newv;
	float dot;

	VectorSubtract( impact, start, v );
	dot = DotProduct( v, dir );
	VectorMA( v, -2*dot, dir, newv );

	VectorNormalize(newv);
	VectorMA(impact, 8192, newv, endout);
}
