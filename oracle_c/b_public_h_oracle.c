/*
 * Oracle TU for the b_public.h slice the g_local.h master needs: lookMode_t,
 * embedded by value in renderInfo_t. Plus the real per-NPC runtime state struct
 * gNPC_t (gentity_t::NPC points at one) and its pointer-free embedded gNPCstats_t.
 * gNPC_t carries gentity_t* / AIGroupInfo_t* pointers => arch-dependent, validated
 * at the host 64-bit layout. Forward-declared pointer targets (gentity_t,
 * AIGroupInfo_t) yield correct pointer width via the C compiler.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

#include <stddef.h>

typedef int qboolean;
typedef float vec_t;
typedef vec_t vec3_t[3];
typedef struct gentity_s gentity_t;
typedef struct AIGroupInfo_s AIGroupInfo_t;

typedef enum {
	LM_ENT = 0,
	LM_INTEREST
} lookMode_t;

int jka_bp_LM_ENT(void) { return LM_ENT; }
int jka_bp_LM_INTEREST(void) { return LM_INTEREST; }

typedef enum {VIS_UNKNOWN, VIS_NOT, VIS_PVS, VIS_360, VIS_FOV, VIS_SHOOT} visibility_t;

typedef enum //# jumpState_e
{
	JS_WAITING = 0,
	JS_FACING,
	JS_CROUCHING,
	JS_JUMPING,
	JS_LANDING
} jumpState_t;

typedef enum //# rank_e
{
	RANK_CIVILIAN,
	RANK_CREWMAN,
	RANK_ENSIGN,
	RANK_LT_JG,
	RANK_LT,
	RANK_LT_COMM,
	RANK_COMMANDER,
	RANK_CAPTAIN
} rank_t;

typedef enum //# bState_e
{
	BS_DEFAULT = 0,
	BS_ADVANCE_FIGHT,
	BS_SLEEP,
	BS_FOLLOW_LEADER,
	BS_JUMP,
	BS_SEARCH,
	BS_WANDER,
	BS_NOCLIP,
	BS_REMOVE,
	BS_CINEMATIC,
	BS_WAIT,
	BS_STAND_GUARD,
	BS_PATROL,
	BS_INVESTIGATE,
	BS_STAND_AND_SHOOT,
	BS_HUNT_AND_KILL,
	BS_FLEE,
	NUM_BSTATES
} bState_t;

/* usercmd_t (q_shared.h) — pointer-free, embedded by value in gNPC_t::last_ucmd. */
typedef unsigned char byte;
typedef struct usercmd_s {
	int		serverTime;
	int		angles[3];
	int		buttons;
	byte	weapon;
	byte	forcesel;
	byte	invensel;
	byte	generic_cmd;
	signed char	forwardmove, rightmove, upmove;
} usercmd_t;

typedef struct gNPCstats_e
{
	int		aggression;
	int		aim;
	float	earshot;
	int		evasion;
	int		hfov;
	int		intelligence;
	int		move;
	int		reactions;
	float	shootDistance;
	int		vfov;
	float	vigilance;
	float	visrange;
	int		runSpeed;
	int		walkSpeed;
	float	yawSpeed;
	int		health;
	int		acceleration;
} gNPCstats_t;

#define	MAX_ENEMY_POS_LAG	2400
#define	ENEMY_POS_LAG_INTERVAL	100
#define	ENEMY_POS_LAG_STEPS	(MAX_ENEMY_POS_LAG/ENEMY_POS_LAG_INTERVAL)

typedef struct
{
	int			timeOfDeath;
	gentity_t	*touchedByPlayer;

	visibility_t	enemyLastVisibility;

	int			aimTime;
	float		desiredYaw;
	float		desiredPitch;
	float		lockedDesiredYaw;
	float		lockedDesiredPitch;
	gentity_t	*aimingBeam;

	vec3_t		enemyLastSeenLocation;
	int			enemyLastSeenTime;
	vec3_t		enemyLastHeardLocation;
	int			enemyLastHeardTime;
	int			lastAlertID;

	int			eFlags;
	int			aiFlags;

	int			currentAmmo;
	int			shotTime;
	int			burstCount;
	int			burstMin;
	int			burstMean;
	int			burstMax;
	int			burstSpacing;
	int			attackHold;
	int			attackHoldTime;
	vec3_t		shootAngles;

	rank_t		rank;

	bState_t	behaviorState;
	bState_t	defaultBehavior;
	bState_t	tempBehavior;

	qboolean	ignorePain;

	int			duckDebounceTime;
	int			walkDebounceTime;
	int			enemyCheckDebounceTime;
	int			investigateDebounceTime;
	int			investigateCount;
	vec3_t		investigateGoal;
	int			investigateSoundDebounceTime;
	int			greetingDebounceTime;
	gentity_t	*eventOwner;

	gentity_t	*coverTarg;
	jumpState_t	jumpState;
	float		followDist;

	gentity_t	*tempGoal;
	gentity_t	*goalEntity;
	gentity_t	*lastGoalEntity;
	gentity_t	*eventualGoal;
	gentity_t	*captureGoal;
	gentity_t	*defendEnt;
	gentity_t	*greetEnt;
	int			goalTime;
	qboolean	straightToGoal;
	float		distToGoal;
	int			navTime;
	int			blockingEntNum;
	int			blockedSpeechDebounceTime;
	int			lastSideStepSide;
	int			sideStepHoldTime;
	int			homeWp;
	AIGroupInfo_t	*group;

	vec3_t		lastPathAngles;

	gNPCstats_t	stats;
	int			aimErrorDebounceTime;
	float		lastAimErrorYaw;
	float		lastAimErrorPitch;
	vec3_t		aimOfs;
	int			currentAim;
	int			currentAggression;

	int			scriptFlags;

	int			desiredSpeed;
	int			currentSpeed;
	char		last_forwardmove;
	char		last_rightmove;
	vec3_t		lastClearOrigin;
	int			consecutiveBlockedMoves;
	int			blockedDebounceTime;
	int			shoveCount;
	vec3_t		blockedDest;

	int			combatPoint;
	int			lastFailedCombatPoint;
	int			movementSpeech;
	float		movementSpeechChance;

	int			nextBStateThink;
	usercmd_t	last_ucmd;

	qboolean	combatMove;
	int			goalRadius;

	int			pauseTime;
	int			standTime;

	int			localState;
	int			squadState;

	int			confusionTime;
	int			charmedTime;
	int			controlledTime;
	int			surrenderTime;

	vec3_t		enemyLaggedPos[ENEMY_POS_LAG_STEPS];

	gentity_t	*watchTarget;

	int			ffireCount;
	int			ffireDebounce;
	int			ffireFadeDebounce;
} gNPC_t;

size_t jka_bp_sizeof_gNPCstats_t(void) { return sizeof(gNPCstats_t); }
size_t jka_bp_off_stats_aggression(void) { return offsetof(gNPCstats_t, aggression); }
size_t jka_bp_off_stats_runSpeed(void) { return offsetof(gNPCstats_t, runSpeed); }
size_t jka_bp_off_stats_acceleration(void) { return offsetof(gNPCstats_t, acceleration); }

size_t jka_bp_sizeof_gNPC_t(void) { return sizeof(gNPC_t); }
size_t jka_bp_off_npc_timeOfDeath(void) { return offsetof(gNPC_t, timeOfDeath); }
size_t jka_bp_off_npc_stats(void) { return offsetof(gNPC_t, stats); }
size_t jka_bp_off_npc_last_ucmd(void) { return offsetof(gNPC_t, last_ucmd); }
size_t jka_bp_off_npc_ffireFadeDebounce(void) { return offsetof(gNPC_t, ffireFadeDebounce); }
