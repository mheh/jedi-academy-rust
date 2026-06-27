/*
 * Value oracle for the bg_pmove.c data-layer slice: the 13 `pm_*` movement
 * parameters and the five numeric force tables (forceSpeedLevels, forcePowerNeeded,
 * forceJumpHeight, forceJumpStrength, forceJumpHeightMax). The real bg_pmove.c
 * cannot be `#include`d cleanly (its quoted `#include`s resolve to the heavy real
 * headers, dragging in the clang-hostile reference tree), so the table/param DATA is
 * transcribed here independently from the Rust port in src/codemp/game/bg_pmove.rs
 * -- the element-wise compare in the Rust test then catches any single-value typo on
 * either side. The three sizing constants (NUM_FORCE_POWERS, NUM_FORCE_POWER_LEVELS,
 * JUMP_VELOCITY) are transcribed verbatim from q_shared.h / bg_public.h (their full
 * headers are not includable here for the same reason). Built only under `oracle`.
 */

#include <assert.h>
#include <stddef.h> /* NULL */
#include "anims.h" /* authentic Raven BOTH_* enum (-I supplied in build.rs) */

#define NUM_FORCE_POWERS 18
#define NUM_FORCE_POWER_LEVELS 4
#define JUMP_VELOCITY 225 // bg_public.h

/* movement parameters, in bg_pmove.c declaration order (lines 40-54) */
static float oracle_pm_params[13] = {
	100.0f, /* pm_stopspeed */
	0.50f,  /* pm_duckScale */
	0.50f,  /* pm_swimScale */
	0.70f,  /* pm_wadeScale */
	36.0f,  /* pm_vehicleaccelerate */
	10.0f,  /* pm_accelerate */
	1.0f,   /* pm_airaccelerate */
	4.0f,   /* pm_wateraccelerate */
	8.0f,   /* pm_flyaccelerate */
	6.0f,   /* pm_friction */
	1.0f,   /* pm_waterfriction */
	3.0f,   /* pm_flightfriction */
	5.0f,   /* pm_spectatorfriction */
};

static float oracle_forceSpeedLevels[4] = {
	1, //rank 0?
	1.25,
	1.5,
	1.75
};

static int oracle_forcePowerNeeded[NUM_FORCE_POWER_LEVELS][NUM_FORCE_POWERS] = {
	{ //nothing should be usable at rank 0..
		999,//FP_HEAL,//instant
		999,//FP_LEVITATION,//hold/duration
		999,//FP_SPEED,//duration
		999,//FP_PUSH,//hold/duration
		999,//FP_PULL,//hold/duration
		999,//FP_TELEPATHY,//instant
		999,//FP_GRIP,//hold/duration
		999,//FP_LIGHTNING,//hold/duration
		999,//FP_RAGE,//duration
		999,//FP_PROTECT,//duration
		999,//FP_ABSORB,//duration
		999,//FP_TEAM_HEAL,//instant
		999,//FP_TEAM_FORCE,//instant
		999,//FP_DRAIN,//hold/duration
		999,//FP_SEE,//duration
		999,//FP_SABER_OFFENSE,
		999,//FP_SABER_DEFENSE,
		999//FP_SABERTHROW,
	},
	{
		65,//FP_HEAL,//instant //was 25, but that was way too little
		10,//FP_LEVITATION,//hold/duration
		50,//FP_SPEED,//duration
		20,//FP_PUSH,//hold/duration
		20,//FP_PULL,//hold/duration
		20,//FP_TELEPATHY,//instant
		30,//FP_GRIP,//hold/duration
		1,//FP_LIGHTNING,//hold/duration
		50,//FP_RAGE,//duration
		50,//FP_PROTECT,//duration
		50,//FP_ABSORB,//duration
		50,//FP_TEAM_HEAL,//instant
		50,//FP_TEAM_FORCE,//instant
		20,//FP_DRAIN,//hold/duration
		20,//FP_SEE,//duration
		0,//FP_SABER_OFFENSE,
		2,//FP_SABER_DEFENSE,
		20//FP_SABERTHROW,
	},
	{
		60,//FP_HEAL,//instant
		10,//FP_LEVITATION,//hold/duration
		50,//FP_SPEED,//duration
		20,//FP_PUSH,//hold/duration
		20,//FP_PULL,//hold/duration
		20,//FP_TELEPATHY,//instant
		30,//FP_GRIP,//hold/duration
		1,//FP_LIGHTNING,//hold/duration
		50,//FP_RAGE,//duration
		25,//FP_PROTECT,//duration
		25,//FP_ABSORB,//duration
		33,//FP_TEAM_HEAL,//instant
		33,//FP_TEAM_FORCE,//instant
		20,//FP_DRAIN,//hold/duration
		20,//FP_SEE,//duration
		0,//FP_SABER_OFFENSE,
		1,//FP_SABER_DEFENSE,
		20//FP_SABERTHROW,
	},
	{
		50,//FP_HEAL,//instant //You get 5 points of health.. for 50 force points!
		10,//FP_LEVITATION,//hold/duration
		50,//FP_SPEED,//duration
		20,//FP_PUSH,//hold/duration
		20,//FP_PULL,//hold/duration
		20,//FP_TELEPATHY,//instant
		60,//FP_GRIP,//hold/duration
		1,//FP_LIGHTNING,//hold/duration
		50,//FP_RAGE,//duration
		10,//FP_PROTECT,//duration
		10,//FP_ABSORB,//duration
		25,//FP_TEAM_HEAL,//instant
		25,//FP_TEAM_FORCE,//instant
		20,//FP_DRAIN,//hold/duration
		20,//FP_SEE,//duration
		0,//FP_SABER_OFFENSE,
		0,//FP_SABER_DEFENSE,
		20//FP_SABERTHROW,
	}
};

static float oracle_forceJumpHeight[NUM_FORCE_POWER_LEVELS] = {
	32,//normal jump (+stepheight+crouchdiff = 66)
	96,//(+stepheight+crouchdiff = 130)
	192,//(+stepheight+crouchdiff = 226)
	384//(+stepheight+crouchdiff = 418)
};

static float oracle_forceJumpStrength[NUM_FORCE_POWER_LEVELS] = {
	JUMP_VELOCITY,//normal jump
	420,
	590,
	840
};

static float oracle_forceJumpHeightMax[NUM_FORCE_POWER_LEVELS] = {
	66,//normal jump (32+stepheight(18)+crouchdiff(24) = 74)
	130,//(96+stepheight(18)+crouchdiff(24) = 138)
	226,//(192+stepheight(18)+crouchdiff(24) = 234)
	418//(384+stepheight(18)+crouchdiff(24) = 426)
};

const float *jka_pm_params(void) { return oracle_pm_params; }
const float *jka_pm_forceSpeedLevels(void) { return oracle_forceSpeedLevels; }
const int *jka_pm_forcePowerNeeded(void) { return (const int *)oracle_forcePowerNeeded; }
const float *jka_pm_forceJumpHeight(void) { return oracle_forceJumpHeight; }
const float *jka_pm_forceJumpStrength(void) { return oracle_forceJumpStrength; }
const float *jka_pm_forceJumpHeightMax(void) { return oracle_forceJumpHeightMax; }

/*
 * Logic oracles for the bg_pmove.c stateless saber/anim/entity-index helpers
 * (lines 171-323): PM_BGEntForNum, BG_SabersOff, BG_KnockDownable, PM_GetSaberStance,
 * PM_DoSlowFall -- plus PM_AddTouchEnt (line 902). Each body reads either a
 * `playerState_t` PARAMETER or the file-scope `pm` context; here both are MINIMAL
 * structs holding only the fields each body touches, and every body is transcribed
 * VERBATIM. The int-marshalling wrappers (jka_*) load state in, point `pm`/`ps` at it,
 * run one call, and read the result back -- no struct crosses FFI. (Full
 * playerState_t / pmove_t layouts are verified separately in q_shared_h_oracle.c /
 * bg_vehicles_h_oracle.c.) BOTH_* come from the #include'd authentic anims.h; the
 * saber_styles_t and FORCE_LEVEL_* values are transcribed verbatim from q_shared.h.
 */
#define MAXTOUCH 32
#define ENTITYNUM_WORLD 1022 /* MAX_GENTITIES (1<<10) - 2 */
#define MAX_GENTITIES 1024

typedef int qboolean;
#define qtrue 1
#define qfalse 0

typedef unsigned char byte;

/* saber_styles_t (q_shared.h), verbatim */
typedef enum {
	SS_NONE = 0,
	SS_FAST,
	SS_MEDIUM,
	SS_STRONG,
	SS_DESANN,
	SS_TAVION,
	SS_DUAL,
	SS_STAFF,
	SS_NUM_SABER_STYLES
} saber_styles_t;

/* FORCE_LEVEL_* (q_shared.h), verbatim. The trailing NUM_FORCE_POWER_LEVELS enumerator
 * is omitted: that name is already #defined to 4 above for the force tables. */
enum {
	FORCE_LEVEL_0,
	FORCE_LEVEL_1,
	FORCE_LEVEL_2,
	FORCE_LEVEL_3
};
#define FORCE_LEVEL_4 (FORCE_LEVEL_3+1)
#define FORCE_LEVEL_5 (FORCE_LEVEL_4+1)

/* Minimal playerState_t: only the fields these helpers read. The movement-primitive
 * slice (PM_ClipVelocity/PM_Friction/PM_Accelerate/PM_CmdScale) adds velocity, the
 * pm_type/pm_flags/clientNum/groundEntityNum state and speed. */
typedef struct { int saberAnimLevel; int saberAnimLevelBase; } pm_fd_min_t;
typedef struct {
	int legsTimer;
	int legsAnim;
	int saberEntityNum;
	int saberHolstered;
	int m_iVehicleNum;
	int emplacedIndex;
	pm_fd_min_t fd;
	float velocity[3];
	int pm_type;
	int pm_flags;
	int clientNum;
	int groundEntityNum;
	float speed;
	int movementDir;     /* PM_SetMovementDir */
	int delta_angles[3]; /* PM_SetPMViewAngle */
	float viewangles[3]; /* PM_SetPMViewAngle */
	int pm_time;         /* PM_DropTimers */
	int torsoTimer;      /* PM_DropTimers */
	int stats[16];       /* PM_GetOkWeaponForVehicle (MAX_STATS) */
} ps_min_t;
#define playerState_t ps_min_t

/* minimal usercmd carrying the fields PM_SetMovementDir/PM_SetPMViewAngle read */
typedef struct { signed char forwardmove; signed char rightmove; int angles[3]; } usercmd_sm_t;

/* Minimal pmove_t: the touch list (PM_AddTouchEnt), the ps pointer
 * (PM_GetSaberStance/PM_DoSlowFall) and the entity-array base/stride
 * (PM_BGEntForNum), plus the stepSlideFix/waterlevel/gametype the movement
 * primitives read. */
typedef struct {
	int numtouch;
	int touchents[MAXTOUCH];
	playerState_t *ps;
	void *baseEnt;
	int entSize;
	int stepSlideFix;
	int waterlevel;
	int gametype;
	usercmd_sm_t cmd; /* PM_SetMovementDir reads pm->cmd */
} pmove_min_t;

static pmove_min_t *pm;

/* rww - Get a pointer to the bgEntity by the index (returns void* here) */
void *PM_BGEntForNum( int num )
{
	void *ent;

	if (!pm)
	{
		assert(!"You cannot call PM_BGEntForNum outside of pm functions!");
		return NULL;
	}

	if (!pm->baseEnt)
	{
		assert(!"Base entity address not set");
		return NULL;
	}

	if (!pm->entSize)
	{
		assert(!"sizeof(ent) is 0, impossible (not set?)");
		return NULL;
	}

	assert(num >= 0 && num < MAX_GENTITIES);

    ent = (void *)((byte *)pm->baseEnt + pm->entSize*(num));

	return ent;
}

qboolean BG_SabersOff( playerState_t *ps )
{
	if ( !ps->saberHolstered )
	{
		return qfalse;
	}
	if ( ps->fd.saberAnimLevelBase == SS_DUAL
		|| ps->fd.saberAnimLevelBase == SS_STAFF )
	{
		if ( ps->saberHolstered < 2 )
		{
			return qfalse;
		}
	}
	return qtrue;
}

qboolean BG_KnockDownable(playerState_t *ps)
{
	if (!ps)
	{ //just for safety
		return qfalse;
	}

	if (ps->m_iVehicleNum)
	{ //riding a vehicle, don't knock me down
		return qfalse;
	}

	if (ps->emplacedIndex)
	{ //using emplaced gun or eweb, can't be knocked down
		return qfalse;
	}

	//ok, I guess?
	return qtrue;
}

/* Minimal saberInfo_t + BG_MySaber stub: in this minimal harness there is no entity
 * array, so BG_MySaber returns NULL (matching the Rust unit test, which runs against a
 * zeroed g_entities so its real BG_MySaber also returns NULL). With both sabers NULL the
 * PC readyAnim / dual-saber branches are skipped and the saberAnimLevel switch runs --
 * identical coverage to the prior Xbox transcription, now bit-for-bit the PC body. */
typedef struct { int readyAnim; float moveSpeedScale; } saberInfo_t;
static saberInfo_t *BG_MySaber( int clientNum, int saberNum ) { (void)clientNum; (void)saberNum; return 0; }

int PM_GetSaberStance(void)
{
	int anim = BOTH_STAND2;
	saberInfo_t *saber1 = BG_MySaber( pm->ps->clientNum, 0 );
	saberInfo_t *saber2 = BG_MySaber( pm->ps->clientNum, 1 );

	if (!pm->ps->saberEntityNum)
	{ //lost it
		return BOTH_STAND1;
	}

	if ( BG_SabersOff( pm->ps ) )
	{
		return BOTH_STAND1;
	}

	if ( saber1
		&& saber1->readyAnim != -1 )
	{
		return saber1->readyAnim;
	}

	if ( saber2
		&& saber2->readyAnim != -1 )
	{
		return saber2->readyAnim;
	}

	if ( saber1
		&& saber2
		&& !pm->ps->saberHolstered )
	{//dual sabers, both on
		return BOTH_SABERDUAL_STANCE;
	}

	switch ( pm->ps->fd.saberAnimLevel )
	{
	case SS_DUAL:
		anim = BOTH_SABERDUAL_STANCE;
		break;
	case SS_STAFF:
		anim = BOTH_SABERSTAFF_STANCE;
		break;
	case SS_FAST:
	case SS_TAVION:
		anim = BOTH_SABERFAST_STANCE;
		break;
	case SS_STRONG:
		anim = BOTH_SABERSLOW_STANCE;
		break;
	case SS_NONE:
	case SS_MEDIUM:
	case SS_DESANN:
	default:
		anim = BOTH_STAND2;
		break;
	}
	return anim;
}

qboolean PM_DoSlowFall(void)
{
	if ( ( (pm->ps->legsAnim) == BOTH_WALL_RUN_RIGHT || (pm->ps->legsAnim) == BOTH_WALL_RUN_LEFT ) && pm->ps->legsTimer > 500 )
	{
		return qtrue;
	}

	return qfalse;
}

void PM_AddTouchEnt( int entityNum ) {
	int		i;

	if ( entityNum == ENTITYNUM_WORLD ) {
		return;
	}
	if ( pm->numtouch == MAXTOUCH ) {
		return;
	}

	// see if it is already added
	for ( i = 0 ; i < pm->numtouch ; i++ ) {
		if ( pm->touchents[ i ] == entityNum ) {
			return;
		}
	}

	// add it
	pm->touchents[pm->numtouch] = entityNum;
	pm->numtouch++;
}

/* --- int-marshalling wrappers (no struct crosses FFI) --- */

unsigned long jka_PM_BGEntForNum(unsigned long baseEnt, int entSize, int num) {
	pmove_min_t local;
	local.baseEnt = (void *)baseEnt;
	local.entSize = entSize;
	pm = &local;
	return (unsigned long)PM_BGEntForNum(num);
}

int jka_BG_SabersOff(int saberHolstered, int saberAnimLevelBase) {
	ps_min_t ps;
	ps.saberHolstered = saberHolstered;
	ps.fd.saberAnimLevelBase = saberAnimLevelBase;
	return BG_SabersOff(&ps);
}

int jka_BG_KnockDownable(int is_null, int m_iVehicleNum, int emplacedIndex) {
	ps_min_t ps;
	if (is_null) {
		return BG_KnockDownable((ps_min_t *)0);
	}
	ps.m_iVehicleNum = m_iVehicleNum;
	ps.emplacedIndex = emplacedIndex;
	return BG_KnockDownable(&ps);
}

int jka_PM_GetSaberStance(int saberEntityNum, int saberHolstered,
                          int saberAnimLevelBase, int saberAnimLevel) {
	ps_min_t ps;
	pmove_min_t local;
	ps.saberEntityNum = saberEntityNum;
	ps.saberHolstered = saberHolstered;
	ps.fd.saberAnimLevelBase = saberAnimLevelBase;
	ps.fd.saberAnimLevel = saberAnimLevel;
	local.ps = &ps;
	pm = &local;
	return PM_GetSaberStance();
}

int jka_PM_DoSlowFall(int legsAnim, int legsTimer) {
	ps_min_t ps;
	pmove_min_t local;
	ps.legsAnim = legsAnim;
	ps.legsTimer = legsTimer;
	local.ps = &ps;
	pm = &local;
	return PM_DoSlowFall();
}

void jka_pm_addtouchent(int entityNum, int *io_numtouch, int *io_touchents) {
	pmove_min_t local;
	int i;
	local.numtouch = *io_numtouch;
	for (i = 0; i < MAXTOUCH; i++) local.touchents[i] = io_touchents[i];

	pm = &local;
	PM_AddTouchEnt(entityNum);

	*io_numtouch = local.numtouch;
	for (i = 0; i < MAXTOUCH; i++) io_touchents[i] = local.touchents[i];
}

/*
 * Logic oracles for the bg_pmove.c movement primitives (lines 932-1200):
 * PM_ClipVelocity, PM_Friction, PM_Accelerate, PM_CmdScale. Each body is transcribed
 * VERBATIM and reads the `pm`/`pml` file-scope context plus (PM_Friction only) the
 * `pm_entSelf`/`pm_flying` vehicle path -- all modelled here as minimal structs/globals
 * holding just the fields each body touches. The vector helpers below are the verbatim
 * q_shared.h macros / q_math.c functions. The `pm_*` friction params and the FLY_*,
 * VH_*, CLASS_VEHICLE, PM_*, PMF_*, GT_SIEGE, MIN_WALK_NORMAL, SURF_SLICK, MAX_CLIENTS,
 * ENTITYNUM_NONE constants are transcribed verbatim from their JKA headers. The
 * int/float-marshalling wrappers (jka_*) load state in, run one call, and read results
 * back -- no struct crosses FFI. */

#include <math.h> /* sqrt */
#include <stdlib.h> /* abs */

typedef float vec3_t[3];

#define DotProduct(x,y)			((x)[0]*(y)[0]+(x)[1]*(y)[1]+(x)[2]*(y)[2])
#define VectorCopy(a,b)			((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define VectorScale(v,s,o)		((o)[0]=(v)[0]*(s),(o)[1]=(v)[1]*(s),(o)[2]=(v)[2]*(s))
#define VectorSubtract(a,b,c)	((c)[0]=(a)[0]-(b)[0],(c)[1]=(a)[1]-(b)[1],(c)[2]=(a)[2]-(b)[2])
#define VectorMA(v,s,b,o)		((o)[0]=(v)[0]+(b)[0]*(s),(o)[1]=(v)[1]+(b)[1]*(s),(o)[2]=(v)[2]+(b)[2]*(s))

/* q_math.c VectorLength / VectorNormalize, verbatim */
static float VectorLength( const vec3_t v ) {
	return (float)sqrt (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]);
}
static float VectorNormalize( vec3_t v ) {
	float	length, ilength;
	length = v[0]*v[0] + v[1]*v[1] + v[2]*v[2];
	length = sqrt (length);
	if ( length ) {
		ilength = 1/length;
		v[0] *= ilength;
		v[1] *= ilength;
		v[2] *= ilength;
	}
	return length;
}

/* constants from q_shared.h / bg_public.h / bg_local.h / surfaceflags.h / teams.h */
#define MAX_CLIENTS 32
#define ENTITYNUM_NONE (MAX_GENTITIES-1)
#define MIN_WALK_NORMAL 0.7f
#define SURF_SLICK 0x00004000
#define PMF_STUCK_TO_WALL 16384
#define PMF_TIME_KNOCKBACK 64
#define GT_SIEGE 7
#define PM_NORMAL 0
#define PM_FLOAT 2
#define PM_SPECTATOR 4
#define CLASS_VEHICLE 53
#define VH_WALKER 1
#define VH_ANIMAL 4
#define FLY_NONE 0
#define FLY_NORMAL 1
#define FLY_VEHICLE 2

/* pm_* friction parameters (bg_pmove.c), verbatim */
static float pm_stopspeed = 100.0f;
static float pm_friction = 6.0f;
static float pm_waterfriction = 1.0f;
static float pm_spectatorfriction = 5.0f;

/* minimal pml + vehicle chain for PM_Friction */
typedef struct { int surfaceFlags; } trace_min_t;
typedef struct { int walking; float frametime; int msec; trace_min_t groundTrace; } pml_min_t;
static pml_min_t pml;

typedef struct { int type; float friction; } vehicleInfo_min_t;
typedef struct { vehicleInfo_min_t *m_pVehicleInfo; } Vehicle_min_t;
typedef struct { int NPC_class; } es_min_t;
typedef struct { es_min_t s; Vehicle_min_t *m_pVehicle; } bgEntity_min_t;
#define bgEntity_t bgEntity_min_t
static bgEntity_t *pm_entSelf;
static int pm_flying;

/* minimal usercmd_t for PM_CmdScale */
typedef struct { signed char forwardmove; signed char rightmove; signed char upmove; } usercmd_min_t;
#define usercmd_t usercmd_min_t

void PM_ClipVelocity( vec3_t in, vec3_t normal, vec3_t out, float overbounce ) {
	float	backoff;
	float	change;
	float	oldInZ;
	int		i;

	if ( (pm->ps->pm_flags&PMF_STUCK_TO_WALL) )
	{//no sliding!
		VectorCopy( in, out );
		return;
	}
	oldInZ = in[2];

	backoff = DotProduct (in, normal);

	if ( backoff < 0 ) {
		backoff *= overbounce;
	} else {
		backoff /= overbounce;
	}

	for ( i=0 ; i<3 ; i++ ) {
		change = normal[i]*backoff;
		out[i] = in[i] - change;
	}
	if ( pm->stepSlideFix )
	{
		if ( pm->ps->clientNum < MAX_CLIENTS//normal player
			&& pm->ps->groundEntityNum != ENTITYNUM_NONE//on the ground
			&& normal[2] < MIN_WALK_NORMAL )//sliding against a steep slope
		{//if walking on the ground, don't slide up slopes that are too steep to walk on
			out[2] = oldInZ;
		}
	}
}

static void PM_Friction( void ) {
	vec3_t	vec;
	float	*vel;
	float	speed, newspeed, control;
	float	drop;
	bgEntity_t *pEnt = NULL;

	vel = pm->ps->velocity;

	VectorCopy( vel, vec );
	if ( pml.walking ) {
		vec[2] = 0;	// ignore slope movement
	}

	speed = VectorLength(vec);
	if (speed < 1) {
		vel[0] = 0;
		vel[1] = 0;		// allow sinking underwater
		if (pm->ps->pm_type == PM_SPECTATOR)
		{
			vel[2] = 0;
		}
		// FIXME: still have z friction underwater?
		return;
	}

	drop = 0;

	if (pm->ps->clientNum >= MAX_CLIENTS)
	{
		pEnt = pm_entSelf;
	}

	// apply ground friction, even if on ladder
	if (pm_flying != FLY_VEHICLE &&
		pEnt &&
		pEnt->s.NPC_class == CLASS_VEHICLE &&
		pEnt->m_pVehicle &&
		pEnt->m_pVehicle->m_pVehicleInfo->type != VH_ANIMAL &&
		pEnt->m_pVehicle->m_pVehicleInfo->type != VH_WALKER &&
		pEnt->m_pVehicle->m_pVehicleInfo->friction )
	{
		float friction = pEnt->m_pVehicle->m_pVehicleInfo->friction;
		if ( !(pm->ps->pm_flags & PMF_TIME_KNOCKBACK) /*&& !(pm->ps->pm_flags & PMF_TIME_NOFRICTION)*/ )
		{
			control = speed < pm_stopspeed ? pm_stopspeed : speed;
			drop += control*friction*pml.frametime;
		}
	}
	else if ( pm_flying != FLY_NORMAL && pm_flying != FLY_VEHICLE )
	{
		// apply ground friction
		if ( pm->waterlevel <= 1 ) {
			if ( pml.walking && !(pml.groundTrace.surfaceFlags & SURF_SLICK) ) {
				// if getting knocked back, no friction
				if ( ! (pm->ps->pm_flags & PMF_TIME_KNOCKBACK) ) {
					control = speed < pm_stopspeed ? pm_stopspeed : speed;
					drop += control*pm_friction*pml.frametime;
				}
			}
		}
	}

	if ( pm_flying == FLY_VEHICLE )
	{
		if ( !(pm->ps->pm_flags & PMF_TIME_KNOCKBACK) )
		{
			control = speed;// < pm_stopspeed ? pm_stopspeed : speed;
			drop += control*pm_friction*pml.frametime;
		}
	}

	// apply water friction even if just wading
	if ( pm->waterlevel ) {
		drop += speed*pm_waterfriction*pm->waterlevel*pml.frametime;
	}
	// If on a client then there is no friction
	else if ( pm->ps->groundEntityNum < MAX_CLIENTS )
	{
		drop = 0;
	}

	if ( pm->ps->pm_type == PM_SPECTATOR || pm->ps->pm_type == PM_FLOAT )
	{
		if (pm->ps->pm_type == PM_FLOAT)
		{ //almost no friction while floating
			drop += speed*0.1*pml.frametime;
		}
		else
		{
			drop += speed*pm_spectatorfriction*pml.frametime;
		}
	}

	// scale the velocity
	newspeed = speed - drop;
	if (newspeed < 0) {
		newspeed = 0;
	}
	newspeed /= speed;

	vel[0] = vel[0] * newspeed;
	vel[1] = vel[1] * newspeed;
	vel[2] = vel[2] * newspeed;
}

static void PM_Accelerate( vec3_t wishdir, float wishspeed, float accel )
{
	if (pm->gametype != GT_SIEGE
		|| pm->ps->m_iVehicleNum
		|| pm->ps->clientNum >= MAX_CLIENTS
		|| pm->ps->pm_type != PM_NORMAL)
	{ //standard method, allows "bunnyhopping" and whatnot
		int			i;
		float		addspeed, accelspeed, currentspeed;

		currentspeed = DotProduct (pm->ps->velocity, wishdir);
		addspeed = wishspeed - currentspeed;
		if (addspeed <= 0 && pm->ps->clientNum < MAX_CLIENTS) {
			return;
		}

		if (addspeed < 0)
		{
			accelspeed = (-accel)*pml.frametime*wishspeed;
			if (accelspeed < addspeed) {
				accelspeed = addspeed;
			}
		}
		else
		{
			accelspeed = accel*pml.frametime*wishspeed;
			if (accelspeed > addspeed) {
				accelspeed = addspeed;
			}
		}

		for (i=0 ; i<3 ; i++) {
			pm->ps->velocity[i] += accelspeed*wishdir[i];
		}
	}
	else
	{ //use the proper way for siege
		vec3_t		wishVelocity;
		vec3_t		pushDir;
		float		pushLen;
		float		canPush;

		VectorScale( wishdir, wishspeed, wishVelocity );
		VectorSubtract( wishVelocity, pm->ps->velocity, pushDir );
		pushLen = VectorNormalize( pushDir );

		canPush = accel*pml.frametime*wishspeed;
		if (canPush > pushLen) {
			canPush = pushLen;
		}

		VectorMA( pm->ps->velocity, canPush, pushDir, pm->ps->velocity );
	}
}

static float PM_CmdScale( usercmd_t *cmd ) {
	int		max;
	float	total;
	float	scale;
	int		umove = 0; //cmd->upmove;
			//don't factor upmove into scaling speed

	max = abs( cmd->forwardmove );
	if ( abs( cmd->rightmove ) > max ) {
		max = abs( cmd->rightmove );
	}
	if ( abs( umove ) > max ) {
		max = abs( umove );
	}
	if ( !max ) {
		return 0;
	}

	total = sqrt( (float)(cmd->forwardmove * cmd->forwardmove
		+ cmd->rightmove * cmd->rightmove + umove * umove) );
	scale = (float)pm->ps->speed * max / ( 127.0 * total );

	return scale;
}

/* --- int/float-marshalling wrappers (no struct crosses FFI) --- */

void jka_PM_ClipVelocity(const float *in_v, const float *normal_v, float overbounce,
                         int pm_flags, int stepSlideFix, int clientNum, int groundEntityNum,
                         float *out_v) {
	ps_min_t ps;
	pmove_min_t local;
	vec3_t in, normal, out;
	int i;
	for (i = 0; i < 3; i++) { in[i] = in_v[i]; normal[i] = normal_v[i]; out[i] = 0; }
	ps.pm_flags = pm_flags;
	ps.clientNum = clientNum;
	ps.groundEntityNum = groundEntityNum;
	local.ps = &ps;
	local.stepSlideFix = stepSlideFix;
	pm = &local;
	PM_ClipVelocity(in, normal, out, overbounce);
	for (i = 0; i < 3; i++) out_v[i] = out[i];
}

void jka_PM_Friction(float *io_velocity, int walking, int pm_type, int clientNum,
                     int pm_flags, int groundEntityNum, int waterlevel, int surfaceFlags,
                     float frametime, int pm_flying_in,
                     int hasEnt, int NPC_class, int hasVehicle, int vehType, float vehFriction) {
	ps_min_t ps;
	pmove_min_t local;
	bgEntity_t ent;
	Vehicle_min_t veh;
	vehicleInfo_min_t vi;
	int i;
	for (i = 0; i < 3; i++) ps.velocity[i] = io_velocity[i];
	ps.pm_type = pm_type;
	ps.clientNum = clientNum;
	ps.pm_flags = pm_flags;
	ps.groundEntityNum = groundEntityNum;
	local.ps = &ps;
	local.waterlevel = waterlevel;
	pml.walking = walking;
	pml.frametime = frametime;
	pml.groundTrace.surfaceFlags = surfaceFlags;
	pm_flying = pm_flying_in;
	if (hasEnt) {
		ent.s.NPC_class = NPC_class;
		if (hasVehicle) {
			vi.type = vehType;
			vi.friction = vehFriction;
			veh.m_pVehicleInfo = &vi;
			ent.m_pVehicle = &veh;
		} else {
			ent.m_pVehicle = NULL;
		}
		pm_entSelf = &ent;
	} else {
		pm_entSelf = NULL;
	}
	pm = &local;
	PM_Friction();
	for (i = 0; i < 3; i++) io_velocity[i] = ps.velocity[i];
}

void jka_PM_Accelerate(float *io_velocity, const float *wishdir_v, float wishspeed, float accel,
                       float frametime, int gametype, int m_iVehicleNum, int clientNum,
                       int pm_type) {
	ps_min_t ps;
	pmove_min_t local;
	vec3_t wishdir;
	int i;
	for (i = 0; i < 3; i++) { ps.velocity[i] = io_velocity[i]; wishdir[i] = wishdir_v[i]; }
	ps.m_iVehicleNum = m_iVehicleNum;
	ps.clientNum = clientNum;
	ps.pm_type = pm_type;
	local.ps = &ps;
	local.gametype = gametype;
	pml.frametime = frametime;
	pm = &local;
	PM_Accelerate(wishdir, wishspeed, accel);
	for (i = 0; i < 3; i++) io_velocity[i] = ps.velocity[i];
}

float jka_PM_CmdScale(int forwardmove, int rightmove, float speed) {
	ps_min_t ps;
	pmove_min_t local;
	usercmd_t cmd;
	ps.speed = speed;
	local.ps = &ps;
	cmd.forwardmove = (signed char)forwardmove;
	cmd.rightmove = (signed char)rightmove;
	cmd.upmove = 0;
	pm = &local;
	return PM_CmdScale(&cmd);
}

/* ---- PM_SetMovementDir / PM_SetPMViewAngle (bg_pmove.c 1211 / 1320) ---- */

#define ANGLE2SHORT(x)	((int)((x)*65536/360) & 65535)

/* verbatim from bg_pmove.c:1211 */
static void PM_SetMovementDir( void ) {
	if ( pm->cmd.forwardmove || pm->cmd.rightmove ) {
		if ( pm->cmd.rightmove == 0 && pm->cmd.forwardmove > 0 ) {
			pm->ps->movementDir = 0;
		} else if ( pm->cmd.rightmove < 0 && pm->cmd.forwardmove > 0 ) {
			pm->ps->movementDir = 1;
		} else if ( pm->cmd.rightmove < 0 && pm->cmd.forwardmove == 0 ) {
			pm->ps->movementDir = 2;
		} else if ( pm->cmd.rightmove < 0 && pm->cmd.forwardmove < 0 ) {
			pm->ps->movementDir = 3;
		} else if ( pm->cmd.rightmove == 0 && pm->cmd.forwardmove < 0 ) {
			pm->ps->movementDir = 4;
		} else if ( pm->cmd.rightmove > 0 && pm->cmd.forwardmove < 0 ) {
			pm->ps->movementDir = 5;
		} else if ( pm->cmd.rightmove > 0 && pm->cmd.forwardmove == 0 ) {
			pm->ps->movementDir = 6;
		} else if ( pm->cmd.rightmove > 0 && pm->cmd.forwardmove > 0 ) {
			pm->ps->movementDir = 7;
		}
	} else {
		// if they aren't actively going directly sideways,
		// change the animation to the diagonal so they
		// don't stop too crooked
		if ( pm->ps->movementDir == 2 ) {
			pm->ps->movementDir = 1;
		} else if ( pm->ps->movementDir == 6 ) {
			pm->ps->movementDir = 7;
		}
	}
}

/* verbatim from bg_pmove.c:1320 (usercmd_t -> usercmd_sm_t minimal model) */
static void PM_SetPMViewAngle(playerState_t *ps, vec3_t angle, usercmd_sm_t *ucmd)
{
	int			i;

	for (i=0 ; i<3 ; i++)
	{ // set the delta angle
		int		cmdAngle;

		cmdAngle = ANGLE2SHORT(angle[i]);
		ps->delta_angles[i] = cmdAngle - ucmd->angles[i];
	}
	VectorCopy (angle, ps->viewangles);
}

int jka_PM_SetMovementDir(int forwardmove, int rightmove, int movementDir_in) {
	ps_min_t ps;
	pmove_min_t local;
	ps.movementDir = movementDir_in;
	local.ps = &ps;
	local.cmd.forwardmove = (signed char)forwardmove;
	local.cmd.rightmove = (signed char)rightmove;
	pm = &local;
	PM_SetMovementDir();
	pm = 0;
	return ps.movementDir;
}

void jka_PM_SetPMViewAngle(const float *angle, const int *ucmd_angles,
		int *out_delta_angles, float *out_viewangles) {
	ps_min_t ps;
	usercmd_sm_t cmd;
	vec3_t a;
	int i;
	for (i=0 ; i<3 ; i++) { a[i] = angle[i]; cmd.angles[i] = ucmd_angles[i]; }
	PM_SetPMViewAngle(&ps, a, &cmd);
	for (i=0 ; i<3 ; i++) { out_delta_angles[i] = ps.delta_angles[i]; out_viewangles[i] = ps.viewangles[i]; }
}

/* ------------------------------------------------------------------------- *
 * The 1442-3465 self-contained slice (BG_ForceWallJumpStrength,
 * PM_SetForceJumpZStart, PM_DeadMove, PM_FootstepForSurface). The other three
 * fns in the slice get no oracle: PM_AdjustAnglesForWallRunUpFlipAlt (pure
 * delegation to the already-verified PM_SetPMViewAngle), PM_CheckWaterJump
 * and PM_TryRoll (driven by the pm->pointcontents / pm->trace engine
 * callbacks, the PM_SetWaterLevel/PM_CorrectAllSolid precedent), and
 * PM_NoclipMove (a move-integrator composing the verified PM_CmdScale/
 * PM_Accelerate over engine-set pml.forward/right basis vectors).
 * ------------------------------------------------------------------------- */

#define VectorClear(a)		((a)[0]=(a)[1]=(a)[2]=0)
#define SURF_NOSTEPS 0x00400000 /* surfaceflags.h */
#define MATERIAL_MASK 0x1f      /* surfaceflags.h: mask to get the material type */

/* verbatim body of bg_pmove.c:1580 (forceJumpStrength -> the verified oracle table) */
#define forceJumpStrength oracle_forceJumpStrength
static float BG_ForceWallJumpStrength( void )
{
	return (forceJumpStrength[FORCE_LEVEL_3]/2.5f);
}
#undef forceJumpStrength
float jka_BG_ForceWallJumpStrength(void) { return BG_ForceWallJumpStrength(); }

/* verbatim arithmetic of PM_SetForceJumpZStart (bg_pmove.c:1737); the
 * pm->ps->fd.forceJumpZStart indirection is flattened to a single float (all it
 * touches) so the `-= 0.1` double-promotion is what gets checked. */
float jka_PM_SetForceJumpZStart(float value) {
	float forceJumpZStart;
	forceJumpZStart = value;
	if (!forceJumpZStart) {
		forceJumpZStart -= 0.1;
	}
	return forceJumpZStart;
}

/* verbatim body of bg_pmove.c:3360, reusing the global pml.walking + pm->ps->velocity */
static void PM_DeadMove( void ) {
	float	forward;

	if ( !pml.walking ) {
		return;
	}

	// extra friction

	forward = VectorLength (pm->ps->velocity);
	forward -= 20;
	if ( forward <= 0 ) {
		VectorClear (pm->ps->velocity);
	} else {
		VectorNormalize (pm->ps->velocity);
		VectorScale (pm->ps->velocity, forward, pm->ps->velocity);
	}
}
void jka_PM_DeadMove(int walking, float *io_velocity) {
	ps_min_t ps;
	pmove_min_t local;
	int i;
	for (i=0 ; i<3 ; i++) ps.velocity[i] = io_velocity[i];
	local.ps = &ps;
	pm = &local;
	pml.walking = walking;
	PM_DeadMove();
	pm = 0;
	for (i=0 ; i<3 ; i++) io_velocity[i] = ps.velocity[i];
}

/* verbatim body of bg_pmove.c:3455, reusing global pml.groundTrace.surfaceFlags */
static int PM_FootstepForSurface( void )
{
	if ( pml.groundTrace.surfaceFlags & SURF_NOSTEPS )
	{
		return 0;
	}
	return ( pml.groundTrace.surfaceFlags & MATERIAL_MASK );
}
int jka_PM_FootstepForSurface(int surfaceFlags) {
	pml.groundTrace.surfaceFlags = surfaceFlags;
	return PM_FootstepForSurface();
}

/* verbatim bodies of the pure-switch anim classifiers (bg_pmove.c:4431/4452/4474/4487).
 * BOTH_* come from the #include'd authentic anims.h; each takes/returns an int (qboolean),
 * so no struct crosses FFI -- the Rust test sweeps the full anim domain. */
int jka_PM_WalkingAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_WALK1:				//# Normal walk
	case BOTH_WALK2:				//# Normal walk with saber
	case BOTH_WALK_STAFF:			//# Normal walk with staff
	case BOTH_WALK_DUAL:			//# Normal walk with staff
	case BOTH_WALK5:				//# Tavion taunting Kyle (cin 22)
	case BOTH_WALK6:				//# Slow walk for Luke (cin 12)
	case BOTH_WALK7:				//# Fast walk
	case BOTH_WALKBACK1:			//# Walk1 backwards
	case BOTH_WALKBACK2:			//# Walk2 backwards
	case BOTH_WALKBACK_STAFF:		//# Walk backwards with staff
	case BOTH_WALKBACK_DUAL:		//# Walk backwards with dual
		return 1/*qtrue*/;
		break;
	}
	return 0/*qfalse*/;
}

int jka_PM_RunningAnim( int anim )
{
	switch ( (anim) )
	{
	case BOTH_RUN1:			
	case BOTH_RUN2:			
	case BOTH_RUN_STAFF:
	case BOTH_RUN_DUAL:
	case BOTH_RUNBACK1:			
	case BOTH_RUNBACK2:			
	case BOTH_RUNBACK_STAFF:			
	case BOTH_RUNBACK_DUAL:
	case BOTH_RUN1START:			//# Start into full run1
	case BOTH_RUN1STOP:			//# Stop from full run1
	case BOTH_RUNSTRAFE_LEFT1:	//# Sidestep left: should loop
	case BOTH_RUNSTRAFE_RIGHT1:	//# Sidestep right: should loop
		return 1/*qtrue*/;
		break;
	}
	return 0/*qfalse*/;
}

int jka_PM_SwimmingAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_SWIM_IDLE1:		//# Swimming Idle 1
	case BOTH_SWIMFORWARD:		//# Swim forward loop
	case BOTH_SWIMBACKWARD:		//# Swim backward loop
		return 1/*qtrue*/;
		break;
	}
	return 0/*qfalse*/;
}

int jka_PM_RollingAnim( int anim )
{
	switch ( anim )
	{
	case BOTH_ROLL_F:			//# Roll forward
	case BOTH_ROLL_B:			//# Roll backward
	case BOTH_ROLL_L:			//# Roll left
	case BOTH_ROLL_R:			//# Roll right
		return 1/*qtrue*/;
		break;
	}
	return 0/*qfalse*/;
}

/* PM_AnglesForSlope (bg_pmove.c:4501) — bit-exact oracle. Reuses the q_math_oracle.c
 * AngleVectors/vectoangles/Q_fabs symbols (same archive) — the exact C trig the Rust
 * q_math port was verified bit-exact against — so the only new arithmetic is the plain
 * f32 pitch/mod/dot math, identical on both sides. */
extern void AngleVectors( const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up );
extern void vectoangles( const vec3_t value1, vec3_t angles );
extern float Q_fabs( float f );
#ifndef PITCH
#define PITCH 0
#define YAW   1
#define ROLL  2
#endif
#ifndef VectorSet
#define VectorSet(v,x,y,z)	((v)[0]=(x),(v)[1]=(y),(v)[2]=(z))
#endif

void PM_AnglesForSlope( const float yaw, const vec3_t slope, vec3_t angles )
{
	vec3_t	nvf, ovf, ovr, new_angles;
	float	pitch, mod, dot;

	VectorSet( angles, 0, yaw, 0 );
	AngleVectors( angles, ovf, ovr, NULL );

	vectoangles( slope, new_angles );
	pitch = new_angles[PITCH] + 90;
	new_angles[ROLL] = new_angles[PITCH] = 0;

	AngleVectors( new_angles, nvf, NULL, NULL );

	mod = DotProduct( nvf, ovr );

	if ( mod < 0 )
		mod = -1;
	else
		mod = 1;

	dot = DotProduct( nvf, ovf );

	angles[YAW] = 0;
	angles[PITCH] = dot * pitch;
	angles[ROLL] = ((1-Q_fabs(dot)) * pitch * mod);
}
void jka_PM_AnglesForSlope(float yaw, const float *slope_in, float *angles_out) {
	vec3_t slope, angles;
	int i;
	for (i=0;i<3;i++) slope[i]=slope_in[i];
	PM_AnglesForSlope(yaw, slope, angles);
	for (i=0;i<3;i++) angles_out[i]=angles[i];
}

/* BG_InSlopeAnim (bg_pmove.c:4594) — verbatim pure-switch over the LEGS_* slope-stand
 * anims (LEGS_* come from the #include'd authentic anims.h; the source skips the S2
 * series). int in, qboolean out — the Rust test sweeps the full anim domain. */
int jka_BG_InSlopeAnim( int anim )
{
	switch ( anim )
	{
	case LEGS_LEFTUP1:			//# On a slope with left foot 4 higher than right
	case LEGS_LEFTUP2:			//# On a slope with left foot 8 higher than right
	case LEGS_LEFTUP3:			//# On a slope with left foot 12 higher than right
	case LEGS_LEFTUP4:			//# On a slope with left foot 16 higher than right
	case LEGS_LEFTUP5:			//# On a slope with left foot 20 higher than right
	case LEGS_RIGHTUP1:			//# On a slope with RIGHT foot 4 higher than left
	case LEGS_RIGHTUP2:			//# On a slope with RIGHT foot 8 higher than left
	case LEGS_RIGHTUP3:			//# On a slope with RIGHT foot 12 higher than left
	case LEGS_RIGHTUP4:			//# On a slope with RIGHT foot 16 higher than left
	case LEGS_RIGHTUP5:			//# On a slope with RIGHT foot 20 higher than left
	case LEGS_S1_LUP1:
	case LEGS_S1_LUP2:
	case LEGS_S1_LUP3:
	case LEGS_S1_LUP4:
	case LEGS_S1_LUP5:
	case LEGS_S1_RUP1:
	case LEGS_S1_RUP2:
	case LEGS_S1_RUP3:
	case LEGS_S1_RUP4:
	case LEGS_S1_RUP5:
	case LEGS_S3_LUP1:
	case LEGS_S3_LUP2:
	case LEGS_S3_LUP3:
	case LEGS_S3_LUP4:
	case LEGS_S3_LUP5:
	case LEGS_S3_RUP1:
	case LEGS_S3_RUP2:
	case LEGS_S3_RUP3:
	case LEGS_S3_RUP4:
	case LEGS_S3_RUP5:
	case LEGS_S4_LUP1:
	case LEGS_S4_LUP2:
	case LEGS_S4_LUP3:
	case LEGS_S4_LUP4:
	case LEGS_S4_LUP5:
	case LEGS_S4_RUP1:
	case LEGS_S4_RUP2:
	case LEGS_S4_RUP3:
	case LEGS_S4_RUP4:
	case LEGS_S4_RUP5:
	case LEGS_S5_LUP1:
	case LEGS_S5_LUP2:
	case LEGS_S5_LUP3:
	case LEGS_S5_LUP4:
	case LEGS_S5_LUP5:
	case LEGS_S5_RUP1:
	case LEGS_S5_RUP2:
	case LEGS_S5_RUP3:
	case LEGS_S5_RUP4:
	case LEGS_S5_RUP5:
		return 1/*qtrue*/;
		break;
	}
	return 0/*qfalse*/;
}

#ifndef PMF_ALL_TIMES
#define PMF_ALL_TIMES (256|32|64) /* PMF_TIME_WATERJUMP|PMF_TIME_LAND|PMF_TIME_KNOCKBACK */
#endif
/* verbatim body of PM_DropTimers (bg_pmove.c:7536) over the global pm->ps + pml.msec */
static void PM_DropTimers( void ) {
	// drop misc timing counter
	if ( pm->ps->pm_time ) {
		if ( pml.msec >= pm->ps->pm_time ) {
			pm->ps->pm_flags &= ~PMF_ALL_TIMES;
			pm->ps->pm_time = 0;
		} else {
			pm->ps->pm_time -= pml.msec;
		}
	}

	// drop animation counter
	if ( pm->ps->legsTimer > 0 ) {
		pm->ps->legsTimer -= pml.msec;
		if ( pm->ps->legsTimer < 0 ) {
			pm->ps->legsTimer = 0;
		}
	}

	if ( pm->ps->torsoTimer > 0 ) {
		pm->ps->torsoTimer -= pml.msec;
		if ( pm->ps->torsoTimer < 0 ) {
			pm->ps->torsoTimer = 0;
		}
	}
}
/* marshal the 4 ps timing fields + pml.msec by value (no struct crosses FFI) */
void jka_PM_DropTimers(int msec, int *io_pm_time, int *io_pm_flags,
		int *io_legsTimer, int *io_torsoTimer) {
	ps_min_t ps;
	pmove_min_t local;
	ps.pm_time = *io_pm_time;
	ps.pm_flags = *io_pm_flags;
	ps.legsTimer = *io_legsTimer;
	ps.torsoTimer = *io_torsoTimer;
	local.ps = &ps;
	pm = &local;
	pml.msec = msec;
	PM_DropTimers();
	pm = 0;
	*io_pm_time = ps.pm_time;
	*io_pm_flags = ps.pm_flags;
	*io_legsTimer = ps.legsTimer;
	*io_torsoTimer = ps.torsoTimer;
}

/* BG_InRollAnim/BG_InKnockDown/BG_InRollES (bg_pmove.c:8272/8285/8322) — pure-switch
 * roll/knockdown classifiers. BG_InRollAnim reads cent->legsAnim, BG_InRollES reads its
 * `anim` arg (ps unused); both reduce to one int, so the jka_ wrappers take an int. */
int jka_BG_InRollAnim( int legsAnim )
{
	switch ( (legsAnim) )
	{
	case BOTH_ROLL_F:
	case BOTH_ROLL_B:
	case BOTH_ROLL_R:
	case BOTH_ROLL_L:
		return 1/*qtrue*/;
	}
	return 0/*qfalse*/;
}

int jka_BG_InKnockDown( int anim )
{
	switch ( (anim) )
	{
	case BOTH_KNOCKDOWN1:
	case BOTH_KNOCKDOWN2:
	case BOTH_KNOCKDOWN3:
	case BOTH_KNOCKDOWN4:
	case BOTH_KNOCKDOWN5:
		return 1/*qtrue*/;
		break;
	case BOTH_GETUP1:
	case BOTH_GETUP2:
	case BOTH_GETUP3:
	case BOTH_GETUP4:
	case BOTH_GETUP5:
	case BOTH_FORCE_GETUP_F1:
	case BOTH_FORCE_GETUP_F2:
	case BOTH_FORCE_GETUP_B1:
	case BOTH_FORCE_GETUP_B2:
	case BOTH_FORCE_GETUP_B3:
	case BOTH_FORCE_GETUP_B4:
	case BOTH_FORCE_GETUP_B5:
	case BOTH_GETUP_BROLL_B:
	case BOTH_GETUP_BROLL_F:
	case BOTH_GETUP_BROLL_L:
	case BOTH_GETUP_BROLL_R:
	case BOTH_GETUP_FROLL_B:
	case BOTH_GETUP_FROLL_F:
	case BOTH_GETUP_FROLL_L:
	case BOTH_GETUP_FROLL_R:
		return 1/*qtrue*/;
		break;
	}
	return 0/*qfalse*/;
}

int jka_BG_InRollES( int anim )
{
	switch ( (anim) )
	{
	case BOTH_ROLL_F:
	case BOTH_ROLL_B:
	case BOTH_ROLL_R:
	case BOTH_ROLL_L:
		return 1/*qtrue*/;
		break;
	}
	return 0/*qfalse*/;
}

/* BG_UpdateLookAngles (bg_pmove.c:8493) / BG_SwingAngles (8757) — angle easing helpers.
 * Reuse the q_math_oracle.c AngleNormalize180/VectorLengthSquared/AngleSubtract/AngleMod
 * symbols (same archive, bit-exact vs the Rust q_math port); VectorCopy/VectorSubtract
 * are the macros above; fabs is libc. The jka_ wrappers marshal vec3s by float pointer. */
extern float AngleNormalize180( float angle );
extern float VectorLengthSquared( const vec3_t v );
extern float AngleSubtract( float a1, float a2 );
extern float AngleMod( float a );
extern double fabs( double );

void BG_UpdateLookAngles( int lookingDebounceTime, vec3_t lastHeadAngles, int time, vec3_t lookAngles, float lookSpeed, float minPitch, float maxPitch, float minYaw, float maxYaw, float minRoll, float maxRoll )
{
	static const float fFrameInter = 0.1f;
	static vec3_t oldLookAngles;
	static vec3_t lookAnglesDiff;
	static int ang;

	if ( lookingDebounceTime > time )
	{
		//clamp so don't get "Exorcist" effect
		if ( lookAngles[PITCH] > maxPitch )
		{
			lookAngles[PITCH] = maxPitch;
		}
		else if ( lookAngles[PITCH] < minPitch )
		{
			lookAngles[PITCH] = minPitch;
		}
		if ( lookAngles[YAW] > maxYaw )
		{
			lookAngles[YAW] = maxYaw;
		}
		else if ( lookAngles[YAW] < minYaw )
		{
			lookAngles[YAW] = minYaw;
		}
		if ( lookAngles[ROLL] > maxRoll )
		{
			lookAngles[ROLL] = maxRoll;
		}
		else if ( lookAngles[ROLL] < minRoll )
		{
			lookAngles[ROLL] = minRoll;
		}

		//slowly lerp to this new value
		//Remember last headAngles
		VectorCopy( lastHeadAngles, oldLookAngles );
		VectorSubtract( lookAngles, oldLookAngles, lookAnglesDiff );

		for ( ang = 0; ang < 3; ang++ )
		{
			lookAnglesDiff[ang] = AngleNormalize180( lookAnglesDiff[ang] );
		}

		if( VectorLengthSquared( lookAnglesDiff ) )
		{
			lookAngles[PITCH] = AngleNormalize180( oldLookAngles[PITCH]+(lookAnglesDiff[PITCH]*fFrameInter*lookSpeed) );
			lookAngles[YAW] = AngleNormalize180( oldLookAngles[YAW]+(lookAnglesDiff[YAW]*fFrameInter*lookSpeed) );
			lookAngles[ROLL] = AngleNormalize180( oldLookAngles[ROLL]+(lookAnglesDiff[ROLL]*fFrameInter*lookSpeed) );
		}
	}
	//Remember current lookAngles next time
	VectorCopy( lookAngles, lastHeadAngles );
}
void jka_BG_UpdateLookAngles(int lookingDebounceTime, float *lastHeadAngles, int time,
		float *lookAngles, float lookSpeed, float minPitch, float maxPitch, float minYaw,
		float maxYaw, float minRoll, float maxRoll) {
	BG_UpdateLookAngles(lookingDebounceTime, lastHeadAngles, time, lookAngles, lookSpeed,
		minPitch, maxPitch, minYaw, maxYaw, minRoll, maxRoll);
}

static float BG_SwingAngles( float destination, float swingTolerance, float clampTolerance,
					float speed, float *angle, qboolean *swinging, int frametime ) {
	float	swing;
	float	move;
	float	scale;

	if ( !*swinging ) {
		// see if a swing should be started
		swing = AngleSubtract( *angle, destination );
		if ( swing > swingTolerance || swing < -swingTolerance ) {
			*swinging = qtrue;
		}
	}

	if ( !*swinging ) {
		return 0;
	}

	// modify the speed depending on the delta
	// so it doesn't seem so linear
	swing = AngleSubtract( destination, *angle );
	scale = fabs( swing );
	if ( scale < swingTolerance * 0.5 ) {
		scale = 0.5;
	} else if ( scale < swingTolerance ) {
		scale = 1.0;
	} else {
		scale = 2.0;
	}

	// swing towards the destination angle
	if ( swing >= 0 ) {
		move = frametime * scale * speed;
		if ( move >= swing ) {
			move = swing;
			*swinging = qfalse;
		}
		*angle = AngleMod( *angle + move );
	} else if ( swing < 0 ) {
		move = frametime * scale * -speed;
		if ( move <= swing ) {
			move = swing;
			*swinging = qfalse;
		}
		*angle = AngleMod( *angle + move );
	}

	// clamp to no more than tolerance
	swing = AngleSubtract( destination, *angle );
	if ( swing > clampTolerance ) {
		*angle = AngleMod( destination - (clampTolerance - 1) );
	} else if ( swing < -clampTolerance ) {
		*angle = AngleMod( destination + (clampTolerance - 1) );
	}

	return swing;
}
float jka_BG_SwingAngles(float destination, float swingTolerance, float clampTolerance,
		float speed, float *angle, int *swinging, int frametime) {
	return BG_SwingAngles(destination, swingTolerance, clampTolerance, speed, angle,
		swinging, frametime);
}

/* BG_InRoll2 (bg_pmove.c:8818) — verbatim pure-switch over es->legsAnim (reduces to one
 * int). PM_WeaponOkOnVehicle (9497) / PM_GetOkWeaponForVehicle (9514) — the vehicle-legal
 * weapon predicate + scan. The WP_ and STAT_WEAPONS values are transcribed verbatim from
 * bg_weapons.h / bg_public.h; the scan drives the global pm->ps->stats[] by value. */
int jka_BG_InRoll2( int legsAnim )
{
	switch ( (legsAnim) )
	{
	case BOTH_GETUP_BROLL_B:
	case BOTH_GETUP_BROLL_F:
	case BOTH_GETUP_BROLL_L:
	case BOTH_GETUP_BROLL_R:
	case BOTH_GETUP_FROLL_B:
	case BOTH_GETUP_FROLL_F:
	case BOTH_GETUP_FROLL_L:
	case BOTH_GETUP_FROLL_R:
	case BOTH_ROLL_F:
	case BOTH_ROLL_B:
	case BOTH_ROLL_R:
	case BOTH_ROLL_L:
		return 1/*qtrue*/;
		break;
	}
	return 0/*qfalse*/;
}

#define WP_MELEE 2
#define WP_SABER 3
#define WP_BLASTER 5
#define WP_NUM_WEAPONS 19
#define STAT_WEAPONS 4
static int PM_WeaponOkOnVehicle( int weapon )
{
	//FIXME: check g_vehicleInfo for our vehicle?
	switch ( weapon )
	{
	//case WP_NONE:
	case WP_MELEE:
	case WP_SABER:
	case WP_BLASTER:
	//case WP_THERMAL:
		return 1/*qtrue*/;
		break;
	}
	return 0/*qfalse*/;
}
int jka_PM_WeaponOkOnVehicle( int weapon ) { return PM_WeaponOkOnVehicle(weapon); }

static int PM_GetOkWeaponForVehicle(void)
{
	int i = 0;

	while (i < WP_NUM_WEAPONS)
	{
		if ((pm->ps->stats[STAT_WEAPONS] & (1 << i)) &&
			PM_WeaponOkOnVehicle(i))
		{ //this one's good
			return i;
		}

		i++;
	}

	//oh dear!
	//assert(!"No valid veh weaps");
	return -1;
}
int jka_PM_GetOkWeaponForVehicle(int statWeapons) {
	ps_min_t ps;
	pmove_min_t local;
	int r;
	ps.stats[STAT_WEAPONS] = statWeapons;
	local.ps = &ps;
	pm = &local;
	r = PM_GetOkWeaponForVehicle();
	pm = 0;
	return r;
}
