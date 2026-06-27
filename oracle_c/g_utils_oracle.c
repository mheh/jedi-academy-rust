/* Oracle extraction of g_utils.c's pure vector/bounds helpers:
 *   G_SetMovedir    (refs/raven-jediacademy/codemp/game/g_utils.c:687)
 *   G_PointInBounds (g_utils.c:1940)
 *   G_BoxInBounds   (g_utils.c:1959)
 *
 * These are self-contained float logic. G_SetMovedir's generic case delegates to
 * AngleVectors and selects its branch with VectorCompare -- both are real-named
 * symbols defined by q_math_oracle.c in the same liblmd_oracle.a, so they are simply
 * declared extern here and resolved at link time (and are already bit-exact verified
 * against q_math.rs). VectorCopy / VectorClear / VectorAdd are macros in q_shared.h,
 * reproduced verbatim. Renamed `jka_` to avoid colliding with the test binary. */

#include <stddef.h> /* NULL */

typedef float vec_t;
typedef vec_t vec3_t[3];
typedef int qboolean;
#define qfalse 0
#define qtrue 1

#define VectorCopy(a, b)    ((b)[0] = (a)[0], (b)[1] = (a)[1], (b)[2] = (a)[2])
#define VectorClear(a)      ((a)[0] = 0, (a)[1] = 0, (a)[2] = 0)
#define VectorAdd(a, b, c)  ((c)[0] = (a)[0] + (b)[0], (c)[1] = (a)[1] + (b)[1], (c)[2] = (a)[2] + (b)[2])

extern int VectorCompare(const vec3_t v1, const vec3_t v2);
extern void AngleVectors(const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up);

void jka_G_SetMovedir(vec3_t angles, vec3_t movedir) {
	static vec3_t VEC_UP		= {0, -1, 0};
	static vec3_t MOVEDIR_UP	= {0, 0, 1};
	static vec3_t VEC_DOWN		= {0, -2, 0};
	static vec3_t MOVEDIR_DOWN	= {0, 0, -1};

	if ( VectorCompare (angles, VEC_UP) ) {
		VectorCopy (MOVEDIR_UP, movedir);
	} else if ( VectorCompare (angles, VEC_DOWN) ) {
		VectorCopy (MOVEDIR_DOWN, movedir);
	} else {
		AngleVectors (angles, movedir, NULL, NULL);
	}
	VectorClear( angles );
}

qboolean jka_G_PointInBounds(vec3_t point, vec3_t mins, vec3_t maxs) {
	int i;

	for(i = 0; i < 3; i++ )
	{
		if ( point[i] < mins[i] )
		{
			return qfalse;
		}
		if ( point[i] > maxs[i] )
		{
			return qfalse;
		}
	}

	return qtrue;
}

qboolean jka_G_BoxInBounds(vec3_t point, vec3_t mins, vec3_t maxs, vec3_t boundsMins, vec3_t boundsMaxs) {
	vec3_t boxMins;
	vec3_t boxMaxs;

	VectorAdd( point, mins, boxMins );
	VectorAdd( point, maxs, boxMaxs );

	if(boxMaxs[0]>boundsMaxs[0])
		return qfalse;

	if(boxMaxs[1]>boundsMaxs[1])
		return qfalse;

	if(boxMaxs[2]>boundsMaxs[2])
		return qfalse;

	if(boxMins[0]<boundsMins[0])
		return qfalse;

	if(boxMins[1]<boundsMins[1])
		return qfalse;

	if(boxMins[2]<boundsMins[2])
		return qfalse;

	//box is completely contained within bounds
	return qtrue;
}

/* ---- shader-remap subsystem (g_utils.c:9-50) ----
 * Verbatim, with its own copy of the file-scope globals. Com_sprintf / Q_strcat /
 * Q_stricmp are the real-named symbols from q_shared_oracle.c (linked in the same
 * liblmd_oracle.a), so BuildShaderStateConfig's "%5.2f" rendering is the genuine C.
 * jka_ResetRemaps lets the parity test drive a deterministic sequence from empty. */

#include <string.h> /* strcpy, memset */

#define MAX_QPATH 64
#define MAX_STRING_CHARS 1024

extern void Com_sprintf(char *dest, int size, const char *fmt, ...);
extern void Q_strcat(char *dest, int size, const char *src);
extern int Q_stricmp(const char *s1, const char *s2);

typedef struct {
  char oldShader[MAX_QPATH];
  char newShader[MAX_QPATH];
  float timeOffset;
} shaderRemap_t;

#define MAX_SHADER_REMAPS 128

static int remapCount = 0;
static shaderRemap_t remappedShaders[MAX_SHADER_REMAPS];

void jka_ResetRemaps(void) {
	remapCount = 0;
	memset(remappedShaders, 0, sizeof(remappedShaders));
}

void jka_AddRemap(const char *oldShader, const char *newShader, float timeOffset) {
	int i;

	for (i = 0; i < remapCount; i++) {
		if (Q_stricmp(oldShader, remappedShaders[i].oldShader) == 0) {
			// found it, just update this one
			strcpy(remappedShaders[i].newShader,newShader);
			remappedShaders[i].timeOffset = timeOffset;
			return;
		}
	}
	if (remapCount < MAX_SHADER_REMAPS) {
		strcpy(remappedShaders[remapCount].newShader,newShader);
		strcpy(remappedShaders[remapCount].oldShader,oldShader);
		remappedShaders[remapCount].timeOffset = timeOffset;
		remapCount++;
	}
}

const char *jka_BuildShaderStateConfig(void) {
	static char	buff[MAX_STRING_CHARS*4];
	char out[(MAX_QPATH * 2) + 5];
	int i;

	memset(buff, 0, MAX_STRING_CHARS);
	for (i = 0; i < remapCount; i++) {
		Com_sprintf(out, (MAX_QPATH * 2) + 5, "%s=%s:%5.2f@", remappedShaders[i].oldShader, remappedShaders[i].newShader, remappedShaders[i].timeOffset);
		Q_strcat( buff, sizeof( buff ), out);
	}
	return buff;
}

/* ---- GetAnglesForDirection (g_utils.c:2372) ----
 * Pure: subtract then convert to Euler angles. vectoangles is the real-named symbol
 * from q_math_oracle.c (same liblmd_oracle.a), already bit-exact verified against
 * q_math.rs; VectorSubtract is a q_shared.h macro, reproduced verbatim. */
extern void vectoangles(const vec3_t value1, vec3_t angles);
#define VectorSubtract(a, b, c)  ((c)[0] = (a)[0] - (b)[0], (c)[1] = (a)[1] - (b)[1], (c)[2] = (a)[2] - (b)[2])

void jka_GetAnglesForDirection(const vec3_t p1, const vec3_t p2, vec3_t out) {
	vec3_t v;

	VectorSubtract( p2, p1, v );
	vectoangles( v, out );
}

/* ---- ShortestLineSegBewteen2LineSegs (g_utils.c:2194) ----
 * Pure float geometry. Distance and G_FindClosestPointOnLineSegment are real-named
 * symbols from q_math_oracle.c (same liblmd_oracle.a), already bit-exact verified against
 * q_math.rs. DotProduct/VectorMA are q_shared.h macros (VectorSubtract/VectorCopy are
 * already #defined above); fabs is libc. */
#include <math.h> /* fabs */

#define Q3_INFINITE 16777216
#define DotProduct(x, y)      ((x)[0]*(y)[0] + (x)[1]*(y)[1] + (x)[2]*(y)[2])
#define VectorMA(v, s, b, o)  ((o)[0] = (v)[0] + (s)*(b)[0], (o)[1] = (v)[1] + (s)*(b)[1], (o)[2] = (v)[2] + (s)*(b)[2])

extern vec_t Distance(const vec3_t p1, const vec3_t p2);
extern qboolean G_FindClosestPointOnLineSegment(const vec3_t start, const vec3_t end, const vec3_t from, vec3_t result);

float jka_ShortestLineSegBewteen2LineSegs(vec3_t start1, vec3_t end1, vec3_t start2, vec3_t end2, vec3_t close_pnt1, vec3_t close_pnt2) {
	float	current_dist, new_dist;
	vec3_t	new_pnt;
	vec3_t	start_dif;
	vec3_t	v1;
	vec3_t	v2;
	float v1v1, v2v2, v1v2;
	float denom;

	VectorSubtract( start2, start1, start_dif );
	VectorSubtract( end1, start1, v1 );
	VectorSubtract( end2, start2, v2 );
	v1v1 = DotProduct( v1, v1 );
	v2v2 = DotProduct( v2, v2 );
	v1v2 = DotProduct( v1, v2 );

	denom = (v1v2 * v1v2) - (v1v1 * v2v2);

	if ( fabs(denom) > 0.001f )
	{
		float s = -( (v2v2*DotProduct( v1, start_dif )) - (v1v2*DotProduct( v2, start_dif )) ) / denom;
		float t = ( (v1v1*DotProduct( v2, start_dif )) - (v1v2*DotProduct( v1, start_dif )) ) / denom;
		qboolean done = qtrue;

		if ( s < 0 ) { done = qfalse; s = 0; }
		if ( s > 1 ) { done = qfalse; s = 1; }
		if ( t < 0 ) { done = qfalse; t = 0; }
		if ( t > 1 ) { done = qfalse; t = 1; }

		VectorMA( start1, s, v1, close_pnt1 );
		VectorMA( start2, t, v2, close_pnt2 );

		current_dist = Distance( close_pnt1, close_pnt2 );
		if ( done )
		{
			return current_dist;
		}
	}
	else
	{
		current_dist = Q3_INFINITE;
	}

	new_dist = Distance( start1, start2 );
	if ( new_dist < current_dist ) { VectorCopy( start1, close_pnt1 ); VectorCopy( start2, close_pnt2 ); current_dist = new_dist; }

	new_dist = Distance( start1, end2 );
	if ( new_dist < current_dist ) { VectorCopy( start1, close_pnt1 ); VectorCopy( end2, close_pnt2 ); current_dist = new_dist; }

	new_dist = Distance( end1, start2 );
	if ( new_dist < current_dist ) { VectorCopy( end1, close_pnt1 ); VectorCopy( start2, close_pnt2 ); current_dist = new_dist; }

	new_dist = Distance( end1, end2 );
	if ( new_dist < current_dist ) { VectorCopy( end1, close_pnt1 ); VectorCopy( end2, close_pnt2 ); current_dist = new_dist; }

	G_FindClosestPointOnLineSegment( start2, end2, start1, new_pnt );
	new_dist = Distance( start1, new_pnt );
	if ( new_dist < current_dist ) { VectorCopy( start1, close_pnt1 ); VectorCopy( new_pnt, close_pnt2 ); current_dist = new_dist; }

	G_FindClosestPointOnLineSegment( start2, end2, end1, new_pnt );
	new_dist = Distance( end1, new_pnt );
	if ( new_dist < current_dist ) { VectorCopy( end1, close_pnt1 ); VectorCopy( new_pnt, close_pnt2 ); current_dist = new_dist; }

	G_FindClosestPointOnLineSegment( start1, end1, start2, new_pnt );
	new_dist = Distance( start2, new_pnt );
	if ( new_dist < current_dist ) { VectorCopy( new_pnt, close_pnt1 ); VectorCopy( start2, close_pnt2 ); current_dist = new_dist; }

	G_FindClosestPointOnLineSegment( start1, end1, end2, new_pnt );
	new_dist = Distance( end2, new_pnt );
	if ( new_dist < current_dist ) { VectorCopy( new_pnt, close_pnt1 ); VectorCopy( end2, close_pnt2 ); current_dist = new_dist; }

	return current_dist;
}
