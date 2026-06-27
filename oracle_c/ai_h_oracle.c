/*
 * Oracle TU for the ai.h group-AI types the g_local.h level_locals_t embeds:
 * AIGroupMember_t (pointer-free) and AIGroupInfo_t (carries gentity_t* => arch-
 * dependent, validated at the host 64-bit layout). gentity_t is forward-declared
 * (only ever a pointer here); the C compiler yields sizeof/offsetof and the squad-
 * state count NUM_SQUAD_STATES that sizes AIGroupInfo_t::numState.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

#include <stddef.h>

typedef int qboolean;
typedef int team_t;
typedef float vec_t;
typedef vec_t vec3_t[3];
typedef struct gentity_s gentity_t;

enum {
	SQUAD_IDLE,
	SQUAD_STAND_AND_SHOOT,
	SQUAD_RETREAT,
	SQUAD_COVER,
	SQUAD_TRANSITION,
	SQUAD_POINT,
	SQUAD_SCOUT,
	NUM_SQUAD_STATES,
};

#define MAX_GROUP_MEMBERS 32

typedef struct AIGroupMember_s {
	int	number;
	int waypoint;
	int pathCostToEnemy;
	int	closestBuddy;
} AIGroupMember_t;

typedef struct AIGroupInfo_s {
	int			numGroup;
	qboolean	processed;
	team_t		team;
	gentity_t	*enemy;
	int			enemyWP;
	int			speechDebounceTime;
	int			lastClearShotTime;
	int			lastSeenEnemyTime;
	int			morale;
	int			moraleAdjust;
	int			moraleDebounce;
	int			memberValidateTime;
	int			activeMemberNum;
	gentity_t	*commander;
	vec3_t		enemyLastSeenPos;
	int			numState[ NUM_SQUAD_STATES ];
	AIGroupMember_t member[ MAX_GROUP_MEMBERS ];
} AIGroupInfo_t;

int    jka_ai_NUM_SQUAD_STATES(void) { return NUM_SQUAD_STATES; }
size_t jka_ai_sizeof_AIGroupMember_t(void) { return sizeof(AIGroupMember_t); }
size_t jka_ai_sizeof_AIGroupInfo_t(void) { return sizeof(AIGroupInfo_t); }
size_t jka_ai_off_enemy(void) { return offsetof(AIGroupInfo_t, enemy); }
size_t jka_ai_off_commander(void) { return offsetof(AIGroupInfo_t, commander); }
size_t jka_ai_off_numState(void) { return offsetof(AIGroupInfo_t, numState); }
size_t jka_ai_off_member(void) { return offsetof(AIGroupInfo_t, member); }
