/*
 * Oracle TU for the q_shared.h master networked structs. Contains the authentic
 * Raven struct definitions (verbatim from raven-jediacademy/codemp/game/q_shared.h,
 * PC / non-_XBOX, _ONEBIT_COMBO undefined) behind a minimal type/const prelude,
 * and exposes the real C `sizeof`/`offsetof` so the Rust port in
 * `src/codemp/game/q_shared_h.rs` can assert its layout matches bit-for-bit on
 * the build arch. Pointer-free structs => 32/64-bit identical, but we still pin
 * every field's offset via a representative set of checkpoints to catch any
 * field-order or type transcription error that happens to preserve total size.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

#include <stddef.h>

/* --- prelude: the types/consts the structs below depend on --- */
typedef int qboolean; /* enum {qfalse,qtrue} -> int, 4 bytes */
typedef unsigned char byte;
typedef float vec_t;
typedef vec_t vec3_t[3];
typedef vec_t vec2_t[2];

#define MAX_PS_EVENTS 2
#define MAX_STATS 16
#define MAX_PERSISTANT 16
#define MAX_POWERUPS 16
#define MAX_WEAPONS 19
#define NUM_FORCE_POWERS 18
#define TRACK_CHANNEL_MAX 6

typedef struct cplane_s {
	vec3_t	normal;
	float	dist;
	byte	type;
	byte	signbits;
	byte	pad[2];
} cplane_t;

/* a trace is returned when a box is swept through the world */
typedef struct {
	byte		allsolid;	/* if true, plane is not valid */
	byte		startsolid;	/* if true, the initial point was in a solid area */
	short		entityNum;	/* entity the contacted sirface is a part of */
	float		fraction;	/* time completed, 1.0 = didn't hit anything */
	vec3_t		endpos;		/* final position */
	cplane_t	plane;		/* surface normal at impact, transformed to world space */
	int			surfaceFlags;	/* surface hit */
	int			contents;	/* contents on other side of surface hit */
} trace_t;

/* usercmd_t is sent to the server each client frame */
typedef struct usercmd_s {
	int				serverTime;
	int				angles[3];
	int 			buttons;
	byte			weapon;
	byte			forcesel;
	byte			invensel;
	byte			generic_cmd;
	signed char	forwardmove, rightmove, upmove;
} usercmd_t;

typedef enum {
	TR_STATIONARY,
	TR_INTERPOLATE,
	TR_LINEAR,
	TR_LINEAR_STOP,
	TR_NONLINEAR_STOP,
	TR_SINE,
	TR_GRAVITY
} trType_t;

typedef struct {
	trType_t	trType;
	int		trTime;
	int		trDuration;
	vec3_t	trBase;
	vec3_t	trDelta;
} trajectory_t;

typedef struct forcedata_s {
	int			forcePowerDebounce[NUM_FORCE_POWERS];
	int			forcePowersKnown;
	int			forcePowersActive;
	int			forcePowerSelected;
	int			forceButtonNeedRelease;
	int			forcePowerDuration[NUM_FORCE_POWERS];
	int			forcePower;
	int			forcePowerMax;
	int			forcePowerRegenDebounceTime;
	int			forcePowerLevel[NUM_FORCE_POWERS];
	int			forcePowerBaseLevel[NUM_FORCE_POWERS];
	int			forceUsingAdded;
	float		forceJumpZStart;
	float		forceJumpCharge;
	int			forceJumpSound;
	int			forceJumpAddTime;
	int			forceGripEntityNum;
	int			forceGripDamageDebounceTime;
	float		forceGripBeingGripped;
	int			forceGripCripple;
	int			forceGripUseTime;
	float		forceGripSoundTime;
	float		forceGripStarted;
	int			forceHealTime;
	int			forceHealAmount;
	int			forceMindtrickTargetIndex;
	int			forceMindtrickTargetIndex2;
	int			forceMindtrickTargetIndex3;
	int			forceMindtrickTargetIndex4;
	int			forceRageRecoveryTime;
	int			forceDrainEntNum;
	float		forceDrainTime;
	int			forceDoInit;
	int			forceSide;
	int			forceRank;
	int			forceDeactivateAll;
	int			killSoundEntIndex[TRACK_CHANNEL_MAX];
	qboolean	sentryDeployed;
	int			saberAnimLevelBase;
	int			saberAnimLevel;
	int			saberDrawAnimLevel;
	int			suicides;
	int			privateDuelTime;
} forcedata_t;

typedef struct entityState_s {
	int		number;
	int		eType;
	int		eFlags;
	int		eFlags2;
	trajectory_t	pos;
	trajectory_t	apos;
	int		time;
	int		time2;
	vec3_t	origin;
	vec3_t	origin2;
	vec3_t	angles;
	vec3_t	angles2;
	int		bolt1;
	int		bolt2;
	int		trickedentindex;
	int		trickedentindex2;
	int		trickedentindex3;
	int		trickedentindex4;
	float	speed;
	int		fireflag;
	int		genericenemyindex;
	int		activeForcePass;
	int		emplacedOwner;
	int		otherEntityNum;
	int		otherEntityNum2;
	int		groundEntityNum;
	int		constantLight;
	int		loopSound;
	qboolean	loopIsSoundset;
	int		soundSetIndex;
	int		modelGhoul2;
	int		g2radius;
	int		modelindex;
	int		modelindex2;
	int		clientNum;
	int		frame;
	qboolean	saberInFlight;
	int			saberEntityNum;
	int			saberMove;
	int			forcePowersActive;
	int			saberHolstered;
	qboolean	isJediMaster;
	qboolean	isPortalEnt;
	int		solid;
	int		event;
	int		eventParm;
	int			owner;
	int			teamowner;
	qboolean	shouldtarget;
	int		powerups;
	int		weapon;
	int		legsAnim;
	int		torsoAnim;
	qboolean	legsFlip;
	qboolean	torsoFlip;
	int		forceFrame;
	int		generic1;
	int		heldByClient;
	int		ragAttach;
	int		iModelScale;
	int		brokenLimbs;
	int		boltToPlayer;
	qboolean	hasLookTarget;
	int			lookTarget;
	int			customRGBA[4];
	int			health;
	int			maxhealth;
	int		npcSaber1;
	int		npcSaber2;
	int		csSounds_Std;
	int		csSounds_Combat;
	int		csSounds_Extra;
	int		csSounds_Jedi;
	int		surfacesOn;
	int		surfacesOff;
	int		boneIndex1;
	int		boneIndex2;
	int		boneIndex3;
	int		boneIndex4;
	int		boneOrient;
	vec3_t	boneAngles1;
	vec3_t	boneAngles2;
	vec3_t	boneAngles3;
	vec3_t	boneAngles4;
	int		NPC_class;
	int		m_iVehicleNum;
	int			userInt1;
	int			userInt2;
	int			userInt3;
	float		userFloat1;
	float		userFloat2;
	float		userFloat3;
	vec3_t		userVec1;
	vec3_t		userVec2;
} entityState_t;

typedef struct playerState_s {
	int			commandTime;
	int			pm_type;
	int			bobCycle;
	int			pm_flags;
	int			pm_time;
	vec3_t		origin;
	vec3_t		velocity;
	vec3_t		moveDir;
	int			weaponTime;
	int			weaponChargeTime;
	int			weaponChargeSubtractTime;
	int			gravity;
	float		speed;
	int			basespeed;
	int			delta_angles[3];
	int			slopeRecalcTime;
	int			useTime;
	int			groundEntityNum;
	int			legsTimer;
	int			legsAnim;
	int			torsoTimer;
	int			torsoAnim;
	qboolean	legsFlip;
	qboolean	torsoFlip;
	int			movementDir;
	int			eFlags;
	int			eFlags2;
	int			eventSequence;
	int			events[MAX_PS_EVENTS];
	int			eventParms[MAX_PS_EVENTS];
	int			externalEvent;
	int			externalEventParm;
	int			externalEventTime;
	int			clientNum;
	int			weapon;
	int			weaponstate;
	vec3_t		viewangles;
	int			viewheight;
	int			damageEvent;
	int			damageYaw;
	int			damagePitch;
	int			damageCount;
	int			damageType;
	int			painTime;
	int			painDirection;
	float		yawAngle;
	qboolean	yawing;
	float		pitchAngle;
	qboolean	pitching;
	int			stats[MAX_STATS];
	int			persistant[MAX_PERSISTANT];
	int			powerups[MAX_POWERUPS];
	int			ammo[MAX_WEAPONS];
	int			generic1;
	int			loopSound;
	int			jumppad_ent;
	int			ping;
	int			pmove_framecount;
	int			jumppad_frame;
	int			entityEventSequence;
	int			lastOnGround;
	qboolean	saberInFlight;
	int			saberMove;
	int			saberBlocking;
	int			saberBlocked;
	int			saberLockTime;
	int			saberLockEnemy;
	int			saberLockFrame;
	int			saberLockHits;
	int			saberLockHitCheckTime;
	int			saberLockHitIncrementTime;
	qboolean	saberLockAdvance;
	int			saberEntityNum;
	float		saberEntityDist;
	int			saberEntityState;
	int			saberThrowDelay;
	qboolean	saberCanThrow;
	int			saberDidThrowTime;
	int			saberDamageDebounceTime;
	int			saberHitWallSoundDebounceTime;
	int			saberEventFlags;
	int			rocketLockIndex;
	float		rocketLastValidTime;
	float		rocketLockTime;
	float		rocketTargetTime;
	int			emplacedIndex;
	float		emplacedTime;
	qboolean	isJediMaster;
	qboolean	forceRestricted;
	qboolean	trueJedi;
	qboolean	trueNonJedi;
	int			saberIndex;
	int			genericEnemyIndex;
	float		droneFireTime;
	float		droneExistTime;
	int			activeForcePass;
	qboolean	hasDetPackPlanted;
	float		holocronsCarried[NUM_FORCE_POWERS];
	int			holocronCantTouch;
	float		holocronCantTouchTime;
	int			holocronBits;
	int			electrifyTime;
	int			saberAttackSequence;
	int			saberIdleWound;
	int			saberAttackWound;
	int			saberBlockTime;
	int			otherKiller;
	int			otherKillerTime;
	int			otherKillerDebounceTime;
	forcedata_t	fd;
	qboolean	forceJumpFlip;
	int			forceHandExtend;
	int			forceHandExtendTime;
	int			forceRageDrainTime;
	int			forceDodgeAnim;
	qboolean	quickerGetup;
	int			groundTime;
	int			footstepTime;
	int			otherSoundTime;
	float		otherSoundLen;
	int			forceGripMoveInterval;
	int			forceGripChangeMovetype;
	int			forceKickFlip;
	int			duelIndex;
	int			duelTime;
	qboolean	duelInProgress;
	int			saberAttackChainCount;
	int			saberHolstered;
	int			forceAllowDeactivateTime;
	int			zoomMode;
	int			zoomTime;
	qboolean	zoomLocked;
	float		zoomFov;
	int			zoomLockTime;
	int			fallingToDeath;
	int			useDelay;
	qboolean	inAirAnim;
	vec3_t		lastHitLoc;
	int			heldByClient;
	int			ragAttach;
	int			iModelScale;
	int			brokenLimbs;
	qboolean	hasLookTarget;
	int			lookTarget;
	int			customRGBA[4];
	int			standheight;
	int			crouchheight;
	int			m_iVehicleNum;
	vec3_t		vehOrientation;
	qboolean	vehBoarding;
	int			vehSurfaces;
	int			vehTurnaroundIndex;
	int			vehTurnaroundTime;
	qboolean	vehWeaponsLinked;
	int			hyperSpaceTime;
	vec3_t		hyperSpaceAngles;
	int			hackingTime;
	int			hackingBaseTime;
	int			jetpackFuel;
	int			cloakFuel;
	int			userInt1;
	int			userInt2;
	int			userInt3;
	float		userFloat1;
	float		userFloat2;
	float		userFloat3;
	vec3_t		userVec1;
	vec3_t		userVec2;
} playerState_t;

/* --- size accessors --- */
size_t jka_sizeof_trajectory_t(void)  { return sizeof(trajectory_t); }
size_t jka_sizeof_usercmd_t(void)      { return sizeof(usercmd_t); }
size_t jka_sizeof_trace_t(void)        { return sizeof(trace_t); }
size_t jka_sizeof_forcedata_t(void)    { return sizeof(forcedata_t); }
size_t jka_sizeof_entityState_t(void)  { return sizeof(entityState_t); }
size_t jka_sizeof_playerState_t(void)  { return sizeof(playerState_t); }

/* --- offsetof checkpoints (interior fields, to catch order/type errors that
 * preserve total size) --- */
size_t jka_off_traj_trBase(void)  { return offsetof(trajectory_t, trBase); }
size_t jka_off_traj_trDelta(void) { return offsetof(trajectory_t, trDelta); }

size_t jka_off_cmd_weapon(void)      { return offsetof(usercmd_t, weapon); }
size_t jka_off_cmd_forwardmove(void) { return offsetof(usercmd_t, forwardmove); }

size_t jka_off_trace_plane(void)    { return offsetof(trace_t, plane); }
size_t jka_off_trace_contents(void) { return offsetof(trace_t, contents); }

size_t jka_off_fd_forcePowerDuration(void) { return offsetof(forcedata_t, forcePowerDuration); }
size_t jka_off_fd_killSoundEntIndex(void)  { return offsetof(forcedata_t, killSoundEntIndex); }
size_t jka_off_fd_privateDuelTime(void)    { return offsetof(forcedata_t, privateDuelTime); }

size_t jka_off_es_pos(void)         { return offsetof(entityState_t, pos); }
size_t jka_off_es_speed(void)       { return offsetof(entityState_t, speed); }
size_t jka_off_es_customRGBA(void)  { return offsetof(entityState_t, customRGBA); }
size_t jka_off_es_boneAngles1(void) { return offsetof(entityState_t, boneAngles1); }
size_t jka_off_es_userVec2(void)    { return offsetof(entityState_t, userVec2); }

size_t jka_off_ps_origin(void)         { return offsetof(playerState_t, origin); }
size_t jka_off_ps_stats(void)          { return offsetof(playerState_t, stats); }
size_t jka_off_ps_fd(void)             { return offsetof(playerState_t, fd); }
size_t jka_off_ps_lastHitLoc(void)     { return offsetof(playerState_t, lastHitLoc); }
size_t jka_off_ps_vehOrientation(void) { return offsetof(playerState_t, vehOrientation); }
size_t jka_off_ps_userVec2(void)       { return offsetof(playerState_t, userVec2); }

/* --- saber data (q_shared.h). saberInfo_t is embedded by value in gclient_t,
 * so its layout is load-bearing. Pointer-free => arch-independent. --- */
typedef int qhandle_t;
typedef int saber_colors_t;
typedef int saberType_t;
typedef int saber_styles_t;
#define MAX_QPATH 64
#define MAX_BLADES 8

typedef struct {
	int		inAction;
	int		duration;
	int		lastTime;
	vec3_t	base;
	vec3_t	tip;
	vec3_t	dualbase;
	vec3_t	dualtip;
	qboolean	haveOldPos[2];
	vec3_t		oldPos[2];
	vec3_t		oldNormal[2];
} saberTrail_t;

typedef struct {
	qboolean	active;
	saber_colors_t	color;
	float		radius;
	float		length;
	float		lengthMax;
	float		lengthOld;
	float		desiredLength;
	vec3_t		muzzlePoint;
	vec3_t		muzzlePointOld;
	vec3_t		muzzleDir;
	vec3_t		muzzleDirOld;
	saberTrail_t	trail;
	int			hitWallDebounceTime;
	int			storageTime;
	int			extendDebounce;
} bladeInfo_t;

typedef struct {
	char		name[64];
	char		fullName[64];
	saberType_t	type;
	char		model[MAX_QPATH];
	qhandle_t	skin;
	int			soundOn;
	int			soundLoop;
	int			soundOff;
	int			numBlades;
	bladeInfo_t	blade[MAX_BLADES];
	int			stylesLearned;
	int			stylesForbidden;
	int			maxChain;
	int			forceRestrictions;
	int			lockBonus;
	int			parryBonus;
	int			breakParryBonus;
	int			breakParryBonus2;
	int			disarmBonus;
	int			disarmBonus2;
	saber_styles_t	singleBladeStyle;
	int			saberFlags;
	int			saberFlags2;
	qhandle_t	spinSound;
	qhandle_t	swingSound[3];
	float		moveSpeedScale;
	float		animSpeedScale;
	int	kataMove;
	int	lungeAtkMove;
	int	jumpAtkUpMove;
	int	jumpAtkFwdMove;
	int	jumpAtkBackMove;
	int	jumpAtkRightMove;
	int	jumpAtkLeftMove;
	int	readyAnim;
	int	drawAnim;
	int	putawayAnim;
	int	tauntAnim;
	int	bowAnim;
	int	meditateAnim;
	int	flourishAnim;
	int	gloatAnim;
	int			bladeStyle2Start;
	int			trailStyle;
	int			g2MarksShader;
	int			g2WeaponMarkShader;
	qhandle_t	hitSound[3];
	qhandle_t	blockSound[3];
	qhandle_t	bounceSound[3];
	int			blockEffect;
	int			hitPersonEffect;
	int			hitOtherEffect;
	int			bladeEffect;
	float		knockbackScale;
	float		damageScale;
	float		splashRadius;
	int			splashDamage;
	float		splashKnockback;
	int			trailStyle2;
	int			g2MarksShader2;
	int			g2WeaponMarkShader2;
	qhandle_t	hit2Sound[3];
	qhandle_t	block2Sound[3];
	qhandle_t	bounce2Sound[3];
	int			blockEffect2;
	int			hitPersonEffect2;
	int			hitOtherEffect2;
	int			bladeEffect2;
	float		knockbackScale2;
	float		damageScale2;
	float		splashRadius2;
	int			splashDamage2;
	float		splashKnockback2;
} saberInfo_t;

/* material_e + typedef int material_t (q_shared.h) */
enum {
	MAT_METAL = 0,
	MAT_GLASS,
	MAT_ELECTRICAL,
	MAT_ELEC_METAL,
	MAT_DRK_STONE,
	MAT_LT_STONE,
	MAT_GLASS_METAL,
	MAT_METAL2,
	MAT_NONE,
	MAT_GREY_STONE,
	MAT_METAL3,
	MAT_CRATE1,
	MAT_GRATE1,
	MAT_ROPE,
	MAT_CRATE2,
	MAT_WHITE_METAL,
	MAT_SNOWY_ROCK,
	NUM_MATERIALS
};

size_t jka_sizeof_saberTrail_t(void) { return sizeof(saberTrail_t); }
size_t jka_off_saberTrail_oldPos(void) { return offsetof(saberTrail_t, oldPos); }
size_t jka_off_saberTrail_oldNormal(void) { return offsetof(saberTrail_t, oldNormal); }

size_t jka_sizeof_bladeInfo_t(void) { return sizeof(bladeInfo_t); }
size_t jka_off_bladeInfo_trail(void) { return offsetof(bladeInfo_t, trail); }
size_t jka_off_bladeInfo_hitWallDebounceTime(void) { return offsetof(bladeInfo_t, hitWallDebounceTime); }

size_t jka_sizeof_saberInfo_t(void) { return sizeof(saberInfo_t); }
size_t jka_off_saberInfo_blade(void) { return offsetof(saberInfo_t, blade); }
size_t jka_off_saberInfo_saberFlags(void) { return offsetof(saberInfo_t, saberFlags); }
size_t jka_off_saberInfo_swingSound(void) { return offsetof(saberInfo_t, swingSound); }
size_t jka_off_saberInfo_knockbackScale(void) { return offsetof(saberInfo_t, knockbackScale); }
size_t jka_off_saberInfo_splashKnockback2(void) { return offsetof(saberInfo_t, splashKnockback2); }

int jka_mat_MAT_METAL(void) { return MAT_METAL; }
int jka_mat_MAT_NONE(void) { return MAT_NONE; }
int jka_mat_MAT_SNOWY_ROCK(void) { return MAT_SNOWY_ROCK; }
int jka_mat_NUM_MATERIALS(void) { return NUM_MATERIALS; }
