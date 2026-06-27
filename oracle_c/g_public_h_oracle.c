/*
 * Oracle TU for the g_public.h data types (the non-ABI half of g_public.h; the
 * GAME_ and G_ ABI enums live in the Rust FFI scaffold, not here). Carries verbatim
 * copies of the structs over a minimal prelude and exposes the real C
 * `sizeof`/`offsetof` plus the ICARUS enum terminals (NUM_TIDS/NUM_BSETS, which
 * size gentity_t arrays), so the Rust port in `src/codemp/game/g_public_h.rs` can
 * assert its layout/values match bit-for-bit. All pointer-free => arch-independent.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

#include <stddef.h>

typedef int qboolean;
typedef float vec_t;
typedef vec_t vec3_t[3];
#define MAX_QPATH 64
#define MAX_PARMS 16
#define MAX_PARM_STRING_LENGTH MAX_QPATH

typedef struct failedEdge_e {
	int	startID;
	int	endID;
	int checkTime;
	int	entID;
} failedEdge_t;

typedef struct {
	qboolean	linked;
	int			linkcount;
	int			svFlags;
	int			singleClient;
	qboolean	bmodel;
	vec3_t		mins, maxs;
	int			contents;
	vec3_t		absmin, absmax;
	vec3_t		currentOrigin;
	vec3_t		currentAngles;
	qboolean	mIsRoffing;
	int			ownerNum;
	int			broadcastClients[2];
} entityShared_t;

typedef enum {
	TID_CHAN_VOICE = 0,
	TID_ANIM_UPPER,
	TID_ANIM_LOWER,
	TID_ANIM_BOTH,
	TID_MOVE_NAV,
	TID_ANGLE_FACE,
	TID_BSTATE,
	TID_LOCATION,
	TID_RESIZE,
	TID_SHOOT,
	NUM_TIDS,
} taskID_t;

typedef enum {
	BSET_INVALID = -1,
	BSET_FIRST = 0,
	BSET_SPAWN = 0,
	BSET_USE,
	BSET_AWAKE,
	BSET_ANGER,
	BSET_ATTACK,
	BSET_VICTORY,
	BSET_LOSTENEMY,
	BSET_PAIN,
	BSET_FLEE,
	BSET_DEATH,
	BSET_DELAYED,
	BSET_BLOCKED,
	BSET_BUMPED,
	BSET_STUCK,
	BSET_FFIRE,
	BSET_FFDEATH,
	BSET_MINDTRICK,
	NUM_BSETS
} bSet_t;

typedef struct {
	char	parm[MAX_PARMS][MAX_PARM_STRING_LENGTH];
} parms_t;

size_t jka_gp_sizeof_failedEdge_t(void) { return sizeof(failedEdge_t); }
size_t jka_gp_sizeof_entityShared_t(void) { return sizeof(entityShared_t); }
size_t jka_gp_off_es_mins(void) { return offsetof(entityShared_t, mins); }
size_t jka_gp_off_es_currentOrigin(void) { return offsetof(entityShared_t, currentOrigin); }
size_t jka_gp_off_es_broadcastClients(void) { return offsetof(entityShared_t, broadcastClients); }
size_t jka_gp_sizeof_parms_t(void) { return sizeof(parms_t); }

int jka_gp_NUM_TIDS(void) { return NUM_TIDS; }
int jka_gp_NUM_BSETS(void) { return NUM_BSETS; }
int jka_gp_BSET_INVALID(void) { return BSET_INVALID; }
int jka_gp_BSET_MINDTRICK(void) { return BSET_MINDTRICK; }
