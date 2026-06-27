/* Oracle extraction of NPC_goal.c's pure bounds helper:
 *   G_BoundsOverlap (refs/raven-jediacademy/codemp/game/NPC_goal.c:94)
 *
 * Self-contained float-comparison logic — zero callees. `vec3_t` is `float[3]`,
 * decaying to `float*`. Renamed `jka_` to avoid colliding with the test binary. */

typedef float vec_t;
typedef vec_t vec3_t[3];
typedef int qboolean;
#define qfalse 0
#define qtrue 1

qboolean jka_G_BoundsOverlap(const vec3_t mins1, const vec3_t maxs1, const vec3_t mins2, const vec3_t maxs2)
{//NOTE: flush up against counts as overlapping
	if(mins1[0]>maxs2[0])
		return qfalse;

	if(mins1[1]>maxs2[1])
		return qfalse;

	if(mins1[2]>maxs2[2])
		return qfalse;

	if(maxs1[0]<mins2[0])
		return qfalse;

	if(maxs1[1]<mins2[1])
		return qfalse;

	if(maxs1[2]<mins2[2])
		return qfalse;

	return qtrue;
}
