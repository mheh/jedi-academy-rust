/* Oracle extraction of g_weapon.c's pure helpers — the ones whose behavior is
 * pure arithmetic over vec3_t / scalars and so can be parity-checked without any
 * gentity / trap / cvar context:
 *
 *   WP_SpeedOfMissileForWeapon (refs/raven-jediacademy/codemp/game/g_weapon.c:174)
 *   VectorNPos                 (g_weapon.c:2598)
 *   SnapVectorTowards          (g_weapon.c:3423)
 *
 * Each is reproduced verbatim from the source and exposed under a `jka_` prefix so
 * it does not collide with the test binary's own symbols. vec3_t in/out come across
 * the FFI as float pointers (the C array-parameter decay), matching the Rust test's
 * marshalling. */

typedef float vec_t;
typedef vec_t vec3_t[3];
typedef int qboolean;

/* g_weapon.c:174 — verbatim. */
float jka_WP_SpeedOfMissileForWeapon(int wp, qboolean alt_fire)
{
	return 500;
}

/* g_weapon.c:2598 — verbatim. */
void jka_VectorNPos(vec3_t in, vec3_t out)
{
	if (in[0] < 0) { out[0] = -in[0]; } else { out[0] = in[0]; }
	if (in[1] < 0) { out[1] = -in[1]; } else { out[1] = in[1]; }
	if (in[2] < 0) { out[2] = -in[2]; } else { out[2] = in[2]; }
}

/* g_weapon.c:3423 — verbatim. */
void jka_SnapVectorTowards(vec3_t v, vec3_t to)
{
	int i;

	for (i = 0; i < 3; i++) {
		if (to[i] <= v[i]) {
			v[i] = (int)v[i];
		} else {
			v[i] = (int)v[i] + 1;
		}
	}
}
