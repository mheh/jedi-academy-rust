/*
 * Logic oracle for the g_misc.c ref-tag slice ported into src/codemp/game/g_misc.rs.
 *
 * Covers TAG_Init (g_misc.c:2711) and TAG_FindOwner (g_misc.c:2734): the file-static
 * ref-tag-owner pool zeroer and the case-insensitive owner lookup. The real g_misc.c
 * cannot be `#include`d (its quoted `#include`s drag in the clang-hostile reference
 * tree), so the bodies are transcribed here VERBATIM over a layout-faithful copy of the
 * file-static `refTagOwnerMap` global and the `tagOwner_t`/`reference_tag_t` records
 * (the bg_misc / g_main oracle precedent: replicate the static, transcribe the body,
 * marshal in/out over ints). Q_stricmp/Q_stricmpn are transcribed verbatim too (TAG_FindOwner's
 * compare) so the case-folding is real C. Built only under `oracle`.
 */

#include <string.h>

#define MAX_REFNAME 32   /* g_local.h */
#define MAX_TAGS 256     /* g_misc.c:2650 */
#define MAX_TAG_OWNERS 16 /* g_misc.c:2651 */

typedef enum { qfalse, qtrue } qboolean;
typedef float vec3_t[3];

/* reference_tag_t (g_local.h) — layout-faithful */
typedef struct
{
	char		name[MAX_REFNAME];
	vec3_t		origin;
	vec3_t		angles;
	int			flags;	//Just in case
	int			radius;	//For nav goals
	qboolean	inuse;
} reference_tag_t;

/* tagOwner_t (g_misc.c:2656) — layout-faithful */
typedef struct
{
	char			name[MAX_REFNAME];
	reference_tag_t	tags[MAX_TAGS];
	qboolean		inuse;
} tagOwner_t;

/* tagOwner_t refTagOwnerMap[MAX_TAG_OWNERS]; (g_misc.c:2663) */
static tagOwner_t	refTagOwnerMap[MAX_TAG_OWNERS];

/* Q_stricmpn / Q_stricmp (q_shared.c:842 / :900) — verbatim, for the case-fold compare */
static int Q_stricmpn (const char *s1, const char *s2, int n) {
	int		c1, c2;

        if ( s1 == NULL ) {
           if ( s2 == NULL )
             return 0;
           else
             return -1;
        }
        else if ( s2==NULL )
          return 1;

	do {
		c1 = *s1++;
		c2 = *s2++;

		if (!n--) {
			return 0;		// strings are equal until end point
		}

		if (c1 != c2) {
			if (c1 >= 'a' && c1 <= 'z') {
				c1 -= ('a' - 'A');
			}
			if (c2 >= 'a' && c2 <= 'z') {
				c2 -= ('a' - 'A');
			}
			if (c1 != c2) {
				return c1 < c2 ? -1 : 1;
			}
		}
	} while (c1);

	return 0;		// strings are equal
}

static int Q_stricmp (const char *s1, const char *s2) {
	return (s1 && s2) ? Q_stricmpn (s1, s2, 99999) : -1;
}

/* TAG_Init (g_misc.c:2711) — verbatim */
void TAG_Init( void )
{
	int i = 0;
	int x = 0;

	while (i < MAX_TAG_OWNERS)
	{
		while (x < MAX_TAGS)
		{
			memset(&refTagOwnerMap[i].tags[x], 0, sizeof(refTagOwnerMap[i].tags[x]));
			x++;
		}
		memset(&refTagOwnerMap[i], 0, sizeof(refTagOwnerMap[i]));
		i++;
	}
}

/* TAG_FindOwner (g_misc.c:2734) — verbatim */
tagOwner_t	*TAG_FindOwner( const char *owner )
{
	int i = 0;

	while (i < MAX_TAG_OWNERS)
	{
		if (refTagOwnerMap[i].inuse && !Q_stricmp(refTagOwnerMap[i].name, owner))
		{
			return &refTagOwnerMap[i];
		}
		i++;
	}

	return NULL;
}

/*
 * Marshalling wrappers. The map is opaque to Rust; tests seed/read it through these.
 */

/* Reset every slot to a known dirty value, then run TAG_Init, then report whether
 * the whole map is zero. Returns 1 if fully zeroed, 0 otherwise. */
int jka_TAG_Init_zeroes(void) {
	int i, x;
	unsigned char *p;
	int total = (int)sizeof(refTagOwnerMap);

	/* dirty the entire map */
	p = (unsigned char *)refTagOwnerMap;
	for (i = 0; i < total; i++) {
		p[i] = 0xAB;
	}
	/* also set inuse/names to non-zero sentinels via the typed view */
	for (i = 0; i < MAX_TAG_OWNERS; i++) {
		refTagOwnerMap[i].inuse = qtrue;
		refTagOwnerMap[i].name[0] = 'X';
		for (x = 0; x < MAX_TAGS; x++) {
			refTagOwnerMap[i].tags[x].inuse = qtrue;
		}
	}

	TAG_Init();

	p = (unsigned char *)refTagOwnerMap;
	for (i = 0; i < total; i++) {
		if (p[i] != 0) {
			return 0;
		}
	}
	return 1;
}

/* Seed one owner slot (name + inuse), used to build up a test map. */
void jka_TAG_seed_owner(int idx, const char *name, int inuse) {
	int i;
	for (i = 0; i < MAX_REFNAME && name[i]; i++) {
		refTagOwnerMap[idx].name[i] = name[i];
	}
	refTagOwnerMap[idx].name[i] = '\0';
	refTagOwnerMap[idx].inuse = inuse ? qtrue : qfalse;
}

/* Clear the whole map (so a test starts from a known empty pool). */
void jka_TAG_clear_map(void) {
	memset(refTagOwnerMap, 0, sizeof(refTagOwnerMap));
}

/* Run TAG_FindOwner; return the matched slot index, or -1 if NULL. */
int jka_TAG_FindOwner_index(const char *owner) {
	tagOwner_t *r = TAG_FindOwner(owner);
	if (r == NULL) {
		return -1;
	}
	return (int)(r - refTagOwnerMap);
}

#define TAG_GENERIC_NAME	"__WORLD__"	/* g_misc.c */

/* TAG_Find (g_misc.c:2756) — verbatim */
reference_tag_t	*TAG_Find( const char *owner, const char *name )
{
	tagOwner_t	*tagOwner = NULL;
	int i = 0;

	if (owner && owner[0])
	{
		tagOwner = TAG_FindOwner(owner);
	}
	if (!tagOwner)
	{
		tagOwner = TAG_FindOwner(TAG_GENERIC_NAME);
	}

	//Not found...
	if (!tagOwner)
	{
		tagOwner = TAG_FindOwner( TAG_GENERIC_NAME );

		if (!tagOwner)
		{
			return NULL;
		}
	}

	while (i < MAX_TAGS)
	{
		if (tagOwner->tags[i].inuse && !Q_stricmp(tagOwner->tags[i].name, name))
		{
			return &tagOwner->tags[i];
		}
		i++;
	}

	//Try the generic owner instead
	tagOwner = TAG_FindOwner( TAG_GENERIC_NAME );

	if (!tagOwner)
	{
		return NULL;
	}

	i = 0;
	while (i < MAX_TAGS)
	{
		if (tagOwner->tags[i].inuse && !Q_stricmp(tagOwner->tags[i].name, name))
		{
			return &tagOwner->tags[i];
		}
		i++;
	}

	return NULL;
}

/* Seed one tag (name + inuse) into an owner's tag pool. */
void jka_TAG_seed_tag(int owner_idx, int tag_idx, const char *name, int inuse) {
	int i;
	for (i = 0; i < MAX_REFNAME && name[i]; i++) {
		refTagOwnerMap[owner_idx].tags[tag_idx].name[i] = name[i];
	}
	refTagOwnerMap[owner_idx].tags[tag_idx].name[i] = '\0';
	refTagOwnerMap[owner_idx].tags[tag_idx].inuse = inuse ? qtrue : qfalse;
}

/* Run TAG_Find; return a packed (owner_idx*MAX_TAGS + tag_idx), or -1 if NULL. */
int jka_TAG_Find_index(const char *owner, const char *name) {
	reference_tag_t *r = TAG_Find(owner, name);
	int oi, ti;
	if (r == NULL) {
		return -1;
	}
	/* locate which owner/tag this pointer is */
	for (oi = 0; oi < MAX_TAG_OWNERS; oi++) {
		if (r >= refTagOwnerMap[oi].tags && r < refTagOwnerMap[oi].tags + MAX_TAGS) {
			ti = (int)(r - refTagOwnerMap[oi].tags);
			return oi * MAX_TAGS + ti;
		}
	}
	return -2; /* shouldn't happen */
}

/*
 * TAG_Add + the three tag getters (g_misc.c:2817 / :2893 / :2934 / :2955) — verbatim over
 * the same file-static refTagOwnerMap. Their deps are transcribed: FirstFreeTagOwner /
 * FirstFreeRefTag (g_misc.c:2665/:2684), Q_strncpyz / Q_strlwr (q_shared.c) and the
 * Vector* macros (q_shared.h). Com_Printf / assert are stubbed: the tests only ever exercise
 * the non-error paths (no full pools, no misses on the asserting getters), so the stubs are
 * never reached on a value-bearing branch.
 */

#include <ctype.h>

#define S_COLOR_RED "^1"
#define VectorCopy(a,b)		((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define VectorClear(a)		((a)[0]=(a)[1]=(a)[2]=0)
#define TAG_GENERIC_NAME	"__WORLD__"

/* no-op Com_Printf / assert for the oracle (error paths are never value-tested) */
static void oracle_Com_Printf(const char *fmt, ...) { (void)fmt; }
#define Com_Printf oracle_Com_Printf

/* Q_strncpyz (q_shared.c) — verbatim (NULL/destsize guards dropped; never hit here) */
static void Q_strncpyz( char *dest, const char *src, int destsize ) {
	strncpy( dest, src, destsize-1 );
	dest[destsize-1] = 0;
}

/* Q_strlwr (q_shared.c) — verbatim */
static char *Q_strlwr( char *s1 ) {
	char	*s;
	s = s1;
	while ( *s ) {
		*s = tolower(*s);
		s++;
	}
	return s1;
}

/* FirstFreeTagOwner (g_misc.c:2665) — verbatim */
static tagOwner_t *FirstFreeTagOwner(void)
{
	int i = 0;
	while (i < MAX_TAG_OWNERS)
	{
		if (!refTagOwnerMap[i].inuse)
		{
			return &refTagOwnerMap[i];
		}
		i++;
	}
	Com_Printf("WARNING: MAX_TAG_OWNERS (%i) REF TAG LIMIT HIT\n", MAX_TAG_OWNERS);
	return NULL;
}

/* FirstFreeRefTag (g_misc.c:2684) — verbatim (assert dropped) */
static reference_tag_t *FirstFreeRefTag(tagOwner_t *tagOwner)
{
	int i = 0;
	while (i < MAX_TAGS)
	{
		if (!tagOwner->tags[i].inuse)
		{
			return &tagOwner->tags[i];
		}
		i++;
	}
	Com_Printf("WARNING: MAX_TAGS (%i) REF TAG LIMIT HIT\n", MAX_TAGS);
	return NULL;
}

/* TAG_Add (g_misc.c:2817) — verbatim (assert dropped) */
reference_tag_t	*TAG_Add( const char *name, const char *owner, vec3_t origin, vec3_t angles, int radius, int flags )
{
	reference_tag_t	*tag = NULL;
	tagOwner_t	*tagOwner = NULL;

	//Make sure this tag's name isn't alread in use
	if ( TAG_Find( owner, name ) )
	{
		Com_Printf(S_COLOR_RED"Duplicate tag name \"%s\"\n", name );
		return NULL;
	}

	//Attempt to add this to the owner's list
	if ( !owner || !owner[0] )
	{
		//If the owner isn't found, use the generic world name
		owner = TAG_GENERIC_NAME;
	}

	tagOwner = TAG_FindOwner( owner );

	if (!tagOwner)
	{
		//Create a new owner list
		tagOwner = FirstFreeTagOwner();//new	tagOwner_t;

		if (!tagOwner)
		{
			return 0;
		}
	}

	//This is actually reverse order of how SP does it because of the way we're storing/allocating.
	//Now that we have the owner, we want to get the first free reftag on the owner itself.
	tag = FirstFreeRefTag(tagOwner);

	if (!tag)
	{
		return NULL;
	}

	//Copy the information
	VectorCopy( origin, tag->origin );
	VectorCopy( angles, tag->angles );
	tag->radius = radius;
	tag->flags	= flags;

	if ( !name || !name[0] )
	{
		Com_Printf(S_COLOR_RED"ERROR: Nameless ref_tag found at (%i %i %i)\n", (int)origin[0], (int)origin[1], (int)origin[2]);
		return NULL;
	}

	//Copy the name
	Q_strncpyz( (char *) tagOwner->name, owner, MAX_REFNAME );
	Q_strlwr( (char *) tagOwner->name );	//NOTENOTE: For case insensitive searches on a map

	//Copy the name
	Q_strncpyz( (char *) tag->name, name, MAX_REFNAME );
	Q_strlwr( (char *) tag->name );	//NOTENOTE: For case insensitive searches on a map

	tagOwner->inuse = qtrue;
	tag->inuse = qtrue;

	return tag;
}

/*
 * Marshalling wrappers for TAG_Add + getters.
 */

/* Run TAG_Add; return packed (owner_idx*MAX_TAGS + tag_idx) of the returned tag, or -1. */
int jka_TAG_Add(const char *name, const char *owner,
		float ox, float oy, float oz, float ax, float ay, float az,
		int radius, int flags) {
	vec3_t origin, angles;
	reference_tag_t *r;
	int oi, ti;
	origin[0] = ox; origin[1] = oy; origin[2] = oz;
	angles[0] = ax; angles[1] = ay; angles[2] = az;
	r = TAG_Add(name, owner, origin, angles, radius, flags);
	if (r == NULL) {
		return -1;
	}
	for (oi = 0; oi < MAX_TAG_OWNERS; oi++) {
		if (r >= refTagOwnerMap[oi].tags && r < refTagOwnerMap[oi].tags + MAX_TAGS) {
			ti = (int)(r - refTagOwnerMap[oi].tags);
			return oi * MAX_TAGS + ti;
		}
	}
	return -2;
}

/* Read a tag's fields out by (owner_idx, tag_idx) for cross-checking after an add. */
int  jka_TAG_get_inuse(int oi, int ti)  { return (int)refTagOwnerMap[oi].tags[ti].inuse; }
int  jka_TAG_get_radius(int oi, int ti) { return refTagOwnerMap[oi].tags[ti].radius; }
int  jka_TAG_get_flags(int oi, int ti)  { return refTagOwnerMap[oi].tags[ti].flags; }
float jka_TAG_get_origin(int oi, int ti, int c) { return refTagOwnerMap[oi].tags[ti].origin[c]; }
float jka_TAG_get_angles(int oi, int ti, int c) { return refTagOwnerMap[oi].tags[ti].angles[c]; }
void jka_TAG_get_name(int oi, int ti, char *out) { strcpy(out, refTagOwnerMap[oi].tags[ti].name); }
int  jka_TAG_owner_inuse(int oi) { return (int)refTagOwnerMap[oi].inuse; }
void jka_TAG_owner_name(int oi, char *out) { strcpy(out, refTagOwnerMap[oi].name); }

/* TAG_GetOrigin (g_misc.c:2893) — verbatim */
int	TAG_GetOrigin( const char *owner, const char *name, vec3_t origin )
{
	reference_tag_t	*tag = TAG_Find( owner, name );

	if (!tag)
	{
		VectorClear(origin);
		return 0;
	}

	VectorCopy( tag->origin, origin );

	return 1;
}

/* Run TAG_GetOrigin; write origin into out[0..3], return the flag. */
int jka_TAG_GetOrigin(const char *owner, const char *name, float *out) {
	vec3_t origin;
	int r = TAG_GetOrigin(owner, name, origin);
	out[0] = origin[0]; out[1] = origin[1]; out[2] = origin[2];
	return r;
}

/* TAG_GetAngles (g_misc.c:2934) — verbatim (assert dropped) */
int	TAG_GetAngles( const char *owner, const char *name, vec3_t angles )
{
	reference_tag_t	*tag = TAG_Find( owner, name );

	if (!tag)
	{
		return 0;
	}

	VectorCopy( tag->angles, angles );

	return 1;
}

/* Run TAG_GetAngles; write angles into out[0..3], return the flag. */
int jka_TAG_GetAngles(const char *owner, const char *name, float *out) {
	vec3_t angles;
	int r = TAG_GetAngles(owner, name, angles);
	out[0] = angles[0]; out[1] = angles[1]; out[2] = angles[2];
	return r;
}

/* TAG_GetRadius (g_misc.c:2955) — verbatim (assert dropped) */
int TAG_GetRadius( const char *owner, const char *name )
{
	reference_tag_t	*tag = TAG_Find( owner, name );

	if (!tag)
	{
		return 0;
	}

	return tag->radius;
}

/* Run TAG_GetRadius; return the radius (or 0 on miss). */
int jka_TAG_GetRadius(const char *owner, const char *name) {
	return TAG_GetRadius(owner, name);
}
