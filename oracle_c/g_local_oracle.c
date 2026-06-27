/*
 * Oracle TU for the g_local.h master structs (gentity_t / gclient_t; level_locals_t
 * follows in a later slice). These embed many other structs *by value*, so the
 * prelude carries verbatim copies of every by-value dependency (entityState_t,
 * playerState_t, entityShared_t, saberInfo_t, usercmd_t + the g_local-local
 * structs) and forward-declares the pointer-only types. The C compiler then yields
 * the real `sizeof`/`offsetof`, which the Rust port asserts against.
 *
 * gentity_t/gclient_t carry pointers => the layout is arch-dependent; this oracle
 * is compiled at the host (64-bit) word size, matching the
 * `#[cfg(target_pointer_width = "64")]` asserts in the Rust port (see DEVIATIONS.md).
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

#include <stddef.h>

/* ---- base types ---- */
typedef int qboolean;
typedef unsigned char byte;
typedef float vec_t;
typedef vec_t vec3_t[3];

/* ---- limits ---- */
#define MAX_QPATH 64
#define MAX_NETNAME 36
#define MAX_SABERS 2
#define MAX_BLADES 8
#define NUM_TIDS 10
#define NUM_BSETS 17
#define MAX_FAILED_NODES 8
#define MAX_PS_EVENTS 2
#define MAX_STATS 16
#define MAX_PERSISTANT 16
#define MAX_POWERUPS 16
#define MAX_WEAPONS 19
#define NUM_FORCE_POWERS 18
#define TRACK_CHANNEL_MAX 6

/* ---- int-width enums (only width matters where embedded) ---- */
typedef int material_t;
typedef int team_t;
typedef int npcteam_t;
typedef int class_t;
typedef int lookMode_t;
typedef int saber_colors_t;
typedef int saberType_t;
typedef int saber_styles_t;

typedef enum { MOVER_POS1, MOVER_POS2, MOVER_1TO2, MOVER_2TO1 } moverState_t;
enum { HL_NONE = 0, HL_FOOT_RT, HL_FOOT_LT, HL_LEG_RT, HL_LEG_LT, HL_WAIST,
	HL_BACK_RT, HL_BACK_LT, HL_BACK, HL_CHEST_RT, HL_CHEST_LT, HL_CHEST,
	HL_ARM_RT, HL_ARM_LT, HL_HAND_RT, HL_HAND_LT, HL_HEAD,
	HL_GENERIC1, HL_GENERIC2, HL_GENERIC3, HL_GENERIC4, HL_GENERIC5, HL_GENERIC6,
	HL_MAX };
typedef enum { CON_DISCONNECTED, CON_CONNECTING, CON_CONNECTED } clientConnected_e;
typedef int clientConnected_t;
typedef enum { SPECTATOR_NOT, SPECTATOR_FREE, SPECTATOR_FOLLOW, SPECTATOR_SCOREBOARD } spectatorState_t;
typedef enum { TEAM_BEGIN, TEAM_ACTIVE } playerTeamStateState_t;

/* ---- by-value embedded structs (verbatim) ---- */
typedef struct cplane_s { vec3_t normal; float dist; byte type; byte signbits; byte pad[2]; } cplane_t;

typedef struct {
	byte allsolid; byte startsolid; short entityNum; float fraction;
	vec3_t endpos; cplane_t plane; int surfaceFlags; int contents;
} trace_t;

typedef struct usercmd_s {
	int serverTime; int angles[3]; int buttons;
	byte weapon; byte forcesel; byte invensel; byte generic_cmd;
	signed char forwardmove, rightmove, upmove;
} usercmd_t;

typedef enum { TR_STATIONARY, TR_INTERPOLATE, TR_LINEAR, TR_LINEAR_STOP, TR_NONLINEAR_STOP, TR_SINE, TR_GRAVITY } trType_t;
typedef struct { trType_t trType; int trTime; int trDuration; vec3_t trBase; vec3_t trDelta; } trajectory_t;

typedef struct forcedata_s {
	int forcePowerDebounce[NUM_FORCE_POWERS]; int forcePowersKnown; int forcePowersActive;
	int forcePowerSelected; int forceButtonNeedRelease; int forcePowerDuration[NUM_FORCE_POWERS];
	int forcePower; int forcePowerMax; int forcePowerRegenDebounceTime;
	int forcePowerLevel[NUM_FORCE_POWERS]; int forcePowerBaseLevel[NUM_FORCE_POWERS];
	int forceUsingAdded; float forceJumpZStart; float forceJumpCharge; int forceJumpSound;
	int forceJumpAddTime; int forceGripEntityNum; int forceGripDamageDebounceTime;
	float forceGripBeingGripped; int forceGripCripple; int forceGripUseTime; float forceGripSoundTime;
	float forceGripStarted; int forceHealTime; int forceHealAmount; int forceMindtrickTargetIndex;
	int forceMindtrickTargetIndex2; int forceMindtrickTargetIndex3; int forceMindtrickTargetIndex4;
	int forceRageRecoveryTime; int forceDrainEntNum; float forceDrainTime; int forceDoInit;
	int forceSide; int forceRank; int forceDeactivateAll; int killSoundEntIndex[TRACK_CHANNEL_MAX];
	qboolean sentryDeployed; int saberAnimLevelBase; int saberAnimLevel; int saberDrawAnimLevel;
	int suicides; int privateDuelTime;
} forcedata_t;

typedef struct entityState_s {
	int number; int eType; int eFlags; int eFlags2; trajectory_t pos; trajectory_t apos;
	int time; int time2; vec3_t origin; vec3_t origin2; vec3_t angles; vec3_t angles2;
	int bolt1; int bolt2; int trickedentindex; int trickedentindex2; int trickedentindex3;
	int trickedentindex4; float speed; int fireflag; int genericenemyindex; int activeForcePass;
	int emplacedOwner; int otherEntityNum; int otherEntityNum2; int groundEntityNum; int constantLight;
	int loopSound; qboolean loopIsSoundset; int soundSetIndex; int modelGhoul2; int g2radius;
	int modelindex; int modelindex2; int clientNum; int frame; qboolean saberInFlight; int saberEntityNum;
	int saberMove; int forcePowersActive; int saberHolstered; qboolean isJediMaster; qboolean isPortalEnt; int solid;
	int event; int eventParm; int owner; int teamowner; qboolean shouldtarget; int powerups; int weapon;
	int legsAnim; int torsoAnim; qboolean legsFlip; qboolean torsoFlip; int forceFrame; int generic1;
	int heldByClient; int ragAttach; int iModelScale; int brokenLimbs; int boltToPlayer;
	qboolean hasLookTarget; int lookTarget; int customRGBA[4]; int health; int maxhealth;
	int npcSaber1; int npcSaber2; int csSounds_Std; int csSounds_Combat; int csSounds_Extra;
	int csSounds_Jedi; int surfacesOn; int surfacesOff; int boneIndex1; int boneIndex2; int boneIndex3;
	int boneIndex4; int boneOrient; vec3_t boneAngles1; vec3_t boneAngles2; vec3_t boneAngles3;
	vec3_t boneAngles4; int NPC_class; int m_iVehicleNum; int userInt1; int userInt2; int userInt3;
	float userFloat1; float userFloat2; float userFloat3; vec3_t userVec1; vec3_t userVec2;
} entityState_t;

typedef struct playerState_s {
	int commandTime; int pm_type; int bobCycle; int pm_flags; int pm_time; vec3_t origin; vec3_t velocity;
	vec3_t moveDir; int weaponTime; int weaponChargeTime; int weaponChargeSubtractTime; int gravity;
	float speed; int basespeed; int delta_angles[3]; int slopeRecalcTime; int useTime; int groundEntityNum;
	int legsTimer; int legsAnim; int torsoTimer; int torsoAnim; qboolean legsFlip; qboolean torsoFlip;
	int movementDir; int eFlags; int eFlags2; int eventSequence; int events[MAX_PS_EVENTS];
	int eventParms[MAX_PS_EVENTS]; int externalEvent; int externalEventParm; int externalEventTime;
	int clientNum; int weapon; int weaponstate; vec3_t viewangles; int viewheight; int damageEvent;
	int damageYaw; int damagePitch; int damageCount; int damageType; int painTime; int painDirection;
	float yawAngle; qboolean yawing; float pitchAngle; qboolean pitching; int stats[MAX_STATS];
	int persistant[MAX_PERSISTANT]; int powerups[MAX_POWERUPS]; int ammo[MAX_WEAPONS]; int generic1;
	int loopSound; int jumppad_ent; int ping; int pmove_framecount; int jumppad_frame;
	int entityEventSequence; int lastOnGround; qboolean saberInFlight; int saberMove; int saberBlocking;
	int saberBlocked; int saberLockTime; int saberLockEnemy; int saberLockFrame; int saberLockHits;
	int saberLockHitCheckTime; int saberLockHitIncrementTime; qboolean saberLockAdvance; int saberEntityNum;
	float saberEntityDist; int saberEntityState; int saberThrowDelay; qboolean saberCanThrow;
	int saberDidThrowTime; int saberDamageDebounceTime; int saberHitWallSoundDebounceTime; int saberEventFlags;
	int rocketLockIndex; float rocketLastValidTime; float rocketLockTime; float rocketTargetTime;
	int emplacedIndex; float emplacedTime; qboolean isJediMaster; qboolean forceRestricted; qboolean trueJedi; qboolean trueNonJedi;
	int saberIndex; int genericEnemyIndex; float droneFireTime; float droneExistTime; int activeForcePass;
	qboolean hasDetPackPlanted; float holocronsCarried[NUM_FORCE_POWERS]; int holocronCantTouch; float holocronCantTouchTime; int holocronBits;
	int electrifyTime; int saberAttackSequence; int saberIdleWound;
	int saberAttackWound; int saberBlockTime; int otherKiller; int otherKillerTime; int otherKillerDebounceTime;
	forcedata_t fd; qboolean forceJumpFlip; int forceHandExtend; int forceHandExtendTime; int forceRageDrainTime;
	int forceDodgeAnim; qboolean quickerGetup; int groundTime; int footstepTime; int otherSoundTime;
	float otherSoundLen; int forceGripMoveInterval; int forceGripChangeMovetype; int forceKickFlip;
	int duelIndex; int duelTime; qboolean duelInProgress; int saberAttackChainCount; int saberHolstered;
	int forceAllowDeactivateTime; int zoomMode; int zoomTime; qboolean zoomLocked; float zoomFov;
	int zoomLockTime; int fallingToDeath; int useDelay; qboolean inAirAnim; vec3_t lastHitLoc;
	int heldByClient; int ragAttach; int iModelScale; int brokenLimbs; qboolean hasLookTarget; int lookTarget;
	int customRGBA[4]; int standheight; int crouchheight; int m_iVehicleNum; vec3_t vehOrientation;
	qboolean vehBoarding; int vehSurfaces; int vehTurnaroundIndex; int vehTurnaroundTime; qboolean vehWeaponsLinked;
	int hyperSpaceTime; vec3_t hyperSpaceAngles; int hackingTime; int hackingBaseTime; int jetpackFuel;
	int cloakFuel; int userInt1; int userInt2; int userInt3; float userFloat1; float userFloat2; float userFloat3;
	vec3_t userVec1; vec3_t userVec2;
} playerState_t;

typedef struct { qboolean linked; int linkcount; int svFlags; int singleClient; qboolean bmodel;
	vec3_t mins, maxs; int contents; vec3_t absmin, absmax; vec3_t currentOrigin; vec3_t currentAngles;
	qboolean mIsRoffing; int ownerNum; int broadcastClients[2]; } entityShared_t;

typedef struct {
	int inAction; int duration; int lastTime; vec3_t base; vec3_t tip; vec3_t dualbase; vec3_t dualtip;
	qboolean haveOldPos[2]; vec3_t oldPos[2]; vec3_t oldNormal[2];
} saberTrail_t;
typedef struct {
	qboolean active; saber_colors_t color; float radius; float length; float lengthMax; float lengthOld;
	float desiredLength; vec3_t muzzlePoint; vec3_t muzzlePointOld; vec3_t muzzleDir; vec3_t muzzleDirOld;
	saberTrail_t trail; int hitWallDebounceTime; int storageTime; int extendDebounce;
} bladeInfo_t;
typedef struct {
	char name[64]; char fullName[64]; saberType_t type; char model[MAX_QPATH]; int skin; int soundOn;
	int soundLoop; int soundOff; int numBlades; bladeInfo_t blade[MAX_BLADES]; int stylesLearned;
	int stylesForbidden; int maxChain; int forceRestrictions; int lockBonus; int parryBonus;
	int breakParryBonus; int breakParryBonus2; int disarmBonus; int disarmBonus2;
	saber_styles_t singleBladeStyle; int saberFlags; int saberFlags2; int spinSound; int swingSound[3];
	float moveSpeedScale; float animSpeedScale; int kataMove; int lungeAtkMove; int jumpAtkUpMove;
	int jumpAtkFwdMove; int jumpAtkBackMove; int jumpAtkRightMove; int jumpAtkLeftMove; int readyAnim;
	int drawAnim; int putawayAnim; int tauntAnim; int bowAnim; int meditateAnim; int flourishAnim;
	int gloatAnim; int bladeStyle2Start; int trailStyle; int g2MarksShader; int g2WeaponMarkShader;
	int hitSound[3]; int blockSound[3]; int bounceSound[3]; int blockEffect; int hitPersonEffect;
	int hitOtherEffect; int bladeEffect; float knockbackScale; float damageScale; float splashRadius;
	int splashDamage; float splashKnockback; int trailStyle2; int g2MarksShader2; int g2WeaponMarkShader2;
	int hit2Sound[3]; int block2Sound[3]; int bounce2Sound[3]; int blockEffect2; int hitPersonEffect2;
	int hitOtherEffect2; int bladeEffect2; float knockbackScale2; float damageScale2; float splashRadius2;
	int splashDamage2; float splashKnockback2;
} saberInfo_t;

/* ---- g_local.h-local by-value structs ---- */
typedef struct {
	playerTeamStateState_t state; int location; int captures; int basedefense; int carrierdefense;
	int flagrecovery; int fragcarrier; int assists; float lasthurtcarrier; float lastreturnedflag;
	float flagsince; float lastfraggedcarrier;
} playerTeamState_t;

typedef struct {
	team_t sessionTeam; int spectatorTime; spectatorState_t spectatorState; int spectatorClient;
	int wins, losses; int selectedFP; int saberLevel; qboolean setForce; int updateUITime;
	qboolean teamLeader; char siegeClass[64]; char saberType[64]; char saber2Type[64];
	int duelTeam; int siegeDesiredTeam;
	int killCount; int TKCount; char IPstring[32];
} clientSession_t;

typedef struct {
	clientConnected_t connected; usercmd_t cmd; qboolean localClient; qboolean initialSpawn;
	qboolean predictItemPickup; qboolean pmoveFixed; char netname[MAX_NETNAME]; int netnameTime; int maxHealth;
	int enterTime; playerTeamState_t teamState; int voteCount; int teamVoteCount; qboolean teamInfo;
} clientPersistant_t;

typedef struct renderInfo_s {
	int headYawRangeLeft; int headYawRangeRight; int headPitchRangeUp; int headPitchRangeDown;
	int torsoYawRangeLeft; int torsoYawRangeRight; int torsoPitchRangeUp; int torsoPitchRangeDown;
	int legsFrame; int torsoFrame; float legsFpsMod; float torsoFpsMod; vec3_t customRGB; int customAlpha;
	int renderFlags; vec3_t muzzlePoint; vec3_t muzzleDir; vec3_t muzzlePointOld; vec3_t muzzleDirOld;
	int mPCalcTime; float lockYaw; vec3_t headPoint; vec3_t headAngles; vec3_t handRPoint; vec3_t handLPoint;
	vec3_t crotchPoint; vec3_t footRPoint; vec3_t footLPoint; vec3_t torsoPoint; vec3_t torsoAngles;
	vec3_t eyePoint; vec3_t eyeAngles; int lookTarget; lookMode_t lookMode; int lookTargetClearTime;
	int lastVoiceVolume; vec3_t lastHeadAngles; vec3_t headBobAngles; vec3_t targetHeadBobAngles;
	int lookingDebounceTime; float legsYaw; void *lastG2; int headBolt; int handRBolt; int handLBolt;
	int torsoBolt; int crotchBolt; int footRBolt; int footLBolt; int motionBolt; int boltValidityTime;
} renderInfo_t;

/* ---- forward decls for pointer-only types ---- */
typedef struct gentity_s gentity_t;
typedef struct gclient_s gclient_t;
typedef struct Vehicle_s Vehicle_t;
typedef struct parms_s parms_t;
typedef struct gNPCstats_s gNPC_t;
typedef struct gitem_s gitem_t;

/* ---- the masters (verbatim) ---- */
struct gentity_s {
	entityState_t	s;
	playerState_t	*playerState;
	Vehicle_t		*m_pVehicle;
	void			*ghoul2;
	int				localAnimIndex;
	vec3_t			modelScale;
	entityShared_t	r;
	int				taskID[NUM_TIDS];
	parms_t			*parms;
	char			*behaviorSet[NUM_BSETS];
	char			*script_targetname;
	int				delayScriptTime;
	char			*fullName;
	char			*targetname;
	char			*classname;
	int				waypoint;
	int				lastWaypoint;
	int				lastValidWaypoint;
	int				noWaypointTime;
	int				combatPoint;
	int				failedWaypoints[MAX_FAILED_NODES];
	int				failedWaypointCheckTime;
	int				next_roff_time;
	struct gclient_s	*client;
	gNPC_t		*NPC;
	int			cantHitEnemyCounter;
	qboolean	noLumbar;
	qboolean	inuse;
	int			lockCount;
	int			spawnflags;
	int			teamnodmg;
	char		*roffname;
	char		*rofftarget;
	char		*healingclass;
	char		*healingsound;
	int			healingrate;
	int			healingDebounce;
	char		*ownername;
	int			objective;
	int			side;
	int			passThroughNum;
	int			aimDebounceTime;
	int			painDebounceTime;
	int			attackDebounceTime;
	int			alliedTeam;
	int			roffid;
	qboolean	neverFree;
	int			flags;
	char		*model;
	char		*model2;
	int			freetime;
	int			eventTime;
	qboolean	freeAfterEvent;
	qboolean	unlinkAfterEvent;
	qboolean	physicsObject;
	float		physicsBounce;
	int			clipmask;
	char		*NPC_type;
	char		*NPC_targetname;
	char		*NPC_target;
	moverState_t moverState;
	int			soundPos1;
	int			sound1to2;
	int			sound2to1;
	int			soundPos2;
	int			soundLoop;
	gentity_t	*parent;
	gentity_t	*nextTrain;
	gentity_t	*prevTrain;
	vec3_t		pos1, pos2;
	vec3_t		pos3;
	char		*message;
	int			timestamp;
	float		angle;
	char		*target;
	char		*target2;
	char		*target3;
	char		*target4;
	char		*target5;
	char		*target6;
	char		*team;
	char		*targetShaderName;
	char		*targetShaderNewName;
	gentity_t	*target_ent;
	char		*closetarget;
	char		*opentarget;
	char		*paintarget;
	char		*goaltarget;
	char		*idealclass;
	float		radius;
	int			maxHealth;
	float		speed;
	vec3_t		movedir;
	float		mass;
	int			setTime;
	int			nextthink;
	void		(*think)(gentity_t *self);
	void		(*reached)(gentity_t *self);
	void		(*blocked)(gentity_t *self, gentity_t *other);
	void		(*touch)(gentity_t *self, gentity_t *other, trace_t *trace);
	void		(*use)(gentity_t *self, gentity_t *other, gentity_t *activator);
	void		(*pain)(gentity_t *self, gentity_t *attacker, int damage);
	void		(*die)(gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int mod);
	int			pain_debounce_time;
	int			fly_sound_debounce_time;
	int			last_move_time;
	int			health;
	qboolean	takedamage;
	material_t	material;
	int			damage;
	int			dflags;
	int			splashDamage;
	int			splashRadius;
	int			methodOfDeath;
	int			splashMethodOfDeath;
	int			locationDamage[HL_MAX];
	int			count;
	int			bounceCount;
	qboolean	alt_fire;
	gentity_t	*chain;
	gentity_t	*enemy;
	gentity_t	*lastEnemy;
	gentity_t	*activator;
	gentity_t	*teamchain;
	gentity_t	*teammaster;
	int			watertype;
	int			waterlevel;
	int			noise_index;
	float		wait;
	float		random;
	int			delay;
	int			genericValue1;
	int			genericValue2;
	int			genericValue3;
	int			genericValue4;
	int			genericValue5;
	int			genericValue6;
	int			genericValue7;
	int			genericValue8;
	int			genericValue9;
	int			genericValue10;
	int			genericValue11;
	int			genericValue12;
	int			genericValue13;
	int			genericValue14;
	int			genericValue15;
	char		*soundSet;
	qboolean	isSaberEntity;
	int			damageRedirect;
	int			damageRedirectTo;
	vec3_t		epVelocity;
	float		epGravFactor;
	gitem_t		*item;
};

struct gclient_s {
	playerState_t	ps;
	clientPersistant_t	pers;
	clientSession_t		sess;
	saberInfo_t	saber[MAX_SABERS];
	void		*weaponGhoul2[MAX_SABERS];
	int			tossableItemDebounce;
	int			bodyGrabTime;
	int			bodyGrabIndex;
	int			pushEffectTime;
	int			invulnerableTimer;
	int			saberCycleQueue;
	int			legsAnimExecute;
	int			torsoAnimExecute;
	qboolean	legsLastFlip;
	qboolean	torsoLastFlip;
	qboolean	readyToExit;
	qboolean	noclip;
	int			lastCmdTime;
	int			buttons;
	int			oldbuttons;
	int			latched_buttons;
	vec3_t		oldOrigin;
	int			damage_armor;
	int			damage_blood;
	int			damage_knockback;
	vec3_t		damage_from;
	qboolean	damage_fromWorld;
	int			damageBoxHandle_Head;
	int			damageBoxHandle_RLeg;
	int			damageBoxHandle_LLeg;
	int			accurateCount;
	int			accuracy_shots;
	int			accuracy_hits;
	int			lastkilled_client;
	int			lasthurt_client;
	int			lasthurt_mod;
	int			respawnTime;
	int			inactivityTime;
	qboolean	inactivityWarning;
	int			rewardTime;
	int			airOutTime;
	int			lastKillTime;
	qboolean	fireHeld;
	gentity_t	*hook;
	int			switchTeamTime;
	int			switchDuelTeamTime;
	int			switchClassTime;
	int			timeResidual;
	char		*areabits;
	int			g2LastSurfaceHit;
	int			g2LastSurfaceTime;
	int			corrTime;
	vec3_t		lastHeadAngles;
	int			lookTime;
	int			brokenLimbs;
	qboolean	noCorpse;
	int			jetPackTime;
	qboolean	jetPackOn;
	int			jetPackToggleTime;
	int			jetPackDebRecharge;
	int			jetPackDebReduce;
	int			cloakToggleTime;
	int			cloakDebRecharge;
	int			cloakDebReduce;
	int			saberStoredIndex;
	int			saberKnockedTime;
	vec3_t		olderSaberBase;
	qboolean	olderIsValid;
	vec3_t		lastSaberDir_Always;
	vec3_t		lastSaberBase_Always;
	int			lastSaberStorageTime;
	qboolean	hasCurrentPosition;
	int			dangerTime;
	int			idleTime;
	int			idleHealth;
	vec3_t		idleViewAngles;
	int			forcePowerSoundDebounce;
	char		modelname[MAX_QPATH];
	qboolean	fjDidJump;
	qboolean	ikStatus;
	int			throwingIndex;
	int			beingThrown;
	int			doingThrow;
	float		hiddenDist;
	vec3_t		hiddenDir;
	renderInfo_t	renderInfo;
	npcteam_t	playerTeam;
	npcteam_t	enemyTeam;
	char		*squadname;
	gentity_t	*team_leader;
	gentity_t	*leader;
	gentity_t	*follower;
	int			numFollowers;
	gentity_t	*formationGoal;
	int			nextFormGoal;
	class_t		NPC_class;
	vec3_t		pushVec;
	int			pushVecTime;
	int			siegeClass;
	int			holdingObjectiveItem;
	int			isMedHealed;
	int			isMedSupplied;
	int			medSupplyDebounce;
	int			isHacking;
	vec3_t		hackingAngles;
	int			siegeEDataSend;
	int			ewebIndex;
	int			ewebTime;
	int			ewebHealth;
	int			inSpaceIndex;
	int			inSpaceSuffocation;
	int			tempSpectate;
	int			jediKickIndex;
	int			jediKickTime;
	int			grappleIndex;
	int			grappleState;
	int			solidHack;
	int			noLightningTime;
	unsigned	mGameFlags;
	qboolean	iAmALoser;
	int			lastGenCmd;
	int			lastGenCmdTime;
	int			otherKillerMOD;
	int			otherKillerVehWeapon;
	int			otherKillerWeaponType;
};

/* ---- accessors ---- */
int    jka_gl_HL_MAX(void) { return HL_MAX; }

size_t jka_gl_sizeof_playerTeamState_t(void) { return sizeof(playerTeamState_t); }
size_t jka_gl_sizeof_clientSession_t(void) { return sizeof(clientSession_t); }
size_t jka_gl_sizeof_clientPersistant_t(void) { return sizeof(clientPersistant_t); }
size_t jka_gl_off_cp_netname(void) { return offsetof(clientPersistant_t, netname); }
size_t jka_gl_off_cp_teamState(void) { return offsetof(clientPersistant_t, teamState); }
size_t jka_gl_sizeof_renderInfo_t(void) { return sizeof(renderInfo_t); }
size_t jka_gl_off_ri_lookMode(void) { return offsetof(renderInfo_t, lookMode); }
size_t jka_gl_off_ri_lastG2(void) { return offsetof(renderInfo_t, lastG2); }

size_t jka_gl_sizeof_gentity_t(void) { return sizeof(gentity_t); }
size_t jka_gl_off_ent_r(void) { return offsetof(gentity_t, r); }
size_t jka_gl_off_ent_taskID(void) { return offsetof(gentity_t, taskID); }
size_t jka_gl_off_ent_client(void) { return offsetof(gentity_t, client); }
size_t jka_gl_off_ent_moverState(void) { return offsetof(gentity_t, moverState); }
size_t jka_gl_off_ent_think(void) { return offsetof(gentity_t, think); }
size_t jka_gl_off_ent_material(void) { return offsetof(gentity_t, material); }
size_t jka_gl_off_ent_locationDamage(void) { return offsetof(gentity_t, locationDamage); }
size_t jka_gl_off_ent_item(void) { return offsetof(gentity_t, item); }

size_t jka_gl_sizeof_gclient_t(void) { return sizeof(gclient_t); }
size_t jka_gl_off_cl_pers(void) { return offsetof(gclient_t, pers); }
size_t jka_gl_off_cl_sess(void) { return offsetof(gclient_t, sess); }
size_t jka_gl_off_cl_saber(void) { return offsetof(gclient_t, saber); }
size_t jka_gl_off_cl_renderInfo(void) { return offsetof(gclient_t, renderInfo); }
size_t jka_gl_off_cl_NPC_class(void) { return offsetof(gclient_t, NPC_class); }
size_t jka_gl_off_cl_lastGenCmdTime(void) { return offsetof(gclient_t, lastGenCmdTime); }

/* ============================================================================
 * level_locals_t and its support types (gentity_t/gclient_t appear only as
 * pointers here, so the full defs above serve as their forward decls).
 * ==========================================================================*/
typedef int fileHandle_t;
#define TEAM_NUM_TEAMS 4
#define MAX_CLIENTS 32
#define MAX_STRING_CHARS 1024
#define BODY_QUEUE_SIZE 8
#define MAX_SPAWN_VARS 64
#define MAX_SPAWN_VARS_CHARS 4096
#define MAX_ALERT_EVENTS 32
#define MAX_FRAME_GROUPS 32
#define MAX_INTEREST_POINTS 64
#define MAX_COMBAT_POINTS 512
#define MAX_GROUP_MEMBERS 32
#define MAX_REFNAME 32
#define MAX_FILEPATH 144

enum { SQUAD_IDLE, SQUAD_STAND_AND_SHOOT, SQUAD_RETREAT, SQUAD_COVER,
	SQUAD_TRANSITION, SQUAD_POINT, SQUAD_SCOUT, NUM_SQUAD_STATES };

typedef struct { vec3_t origin; char *target; } interestPoint_t;
typedef struct { vec3_t origin; int flags; qboolean occupied; int waypoint; int dangerTime; } combatPoint_t;
typedef enum { AET_SIGHT, AET_SOUND, } alertEventType_e;
typedef enum { AEL_MINOR, AEL_SUSPICIOUS, AEL_DISCOVERED, AEL_DANGER, AEL_DANGER_GREAT, } alertEventLevel_e;
typedef struct alertEvent_s {
	vec3_t position; float radius; alertEventLevel_e level; alertEventType_e type;
	gentity_t *owner; float light; float addLight; int ID; int timestamp;
} alertEvent_t;
typedef struct {
	char targetname[MAX_QPATH]; char target[MAX_QPATH]; char target2[MAX_QPATH];
	char target3[MAX_QPATH]; char target4[MAX_QPATH]; int nodeID;
} waypointData_t;
typedef struct AIGroupMember_s { int number; int waypoint; int pathCostToEnemy; int closestBuddy; } AIGroupMember_t;
typedef struct AIGroupInfo_s {
	int numGroup; qboolean processed; team_t team; gentity_t *enemy; int enemyWP;
	int speechDebounceTime; int lastClearShotTime; int lastSeenEnemyTime; int morale;
	int moraleAdjust; int moraleDebounce; int memberValidateTime; int activeMemberNum;
	gentity_t *commander; vec3_t enemyLastSeenPos; int numState[NUM_SQUAD_STATES];
	AIGroupMember_t member[MAX_GROUP_MEMBERS];
} AIGroupInfo_t;
typedef struct reference_tag_s {
	char name[MAX_REFNAME]; vec3_t origin; vec3_t angles; int flags; int radius; qboolean inuse;
} reference_tag_t;
typedef struct bot_settings_s {
	char personalityfile[MAX_FILEPATH]; float skill; char team[MAX_FILEPATH];
} bot_settings_t;

typedef struct {
	struct gclient_s	*clients;
	struct gentity_s	*gentities;
	int			gentitySize;
	int			num_entities;
	int			warmupTime;
	fileHandle_t	logFile;
	int			maxclients;
	int			framenum;
	int			time;
	int			previousTime;
	int			startTime;
	int			teamScores[TEAM_NUM_TEAMS];
	int			lastTeamLocationTime;
	qboolean	newSession;
	qboolean	restarted;
	int			numConnectedClients;
	int			numNonSpectatorClients;
	int			numPlayingClients;
	int			sortedClients[MAX_CLIENTS];
	int			follow1, follow2;
	int			snd_fry;
	int			snd_hack;
	int			snd_medHealed;
	int			snd_medSupplied;
	int			warmupModificationCount;
	char		voteString[MAX_STRING_CHARS];
	char		voteDisplayString[MAX_STRING_CHARS];
	int			voteTime;
	int			voteExecuteTime;
	int			voteYes;
	int			voteNo;
	int			numVotingClients;
	qboolean	votingGametype;
	int			votingGametypeTo;
	char		teamVoteString[2][MAX_STRING_CHARS];
	int			teamVoteTime[2];
	int			teamVoteYes[2];
	int			teamVoteNo[2];
	int			numteamVotingClients[2];
	qboolean	spawning;
	int			numSpawnVars;
	char		*spawnVars[MAX_SPAWN_VARS][2];
	int			numSpawnVarChars;
	char		spawnVarChars[MAX_SPAWN_VARS_CHARS];
	int			intermissionQueued;
	int			intermissiontime;
	char		*changemap;
	qboolean	readyToExit;
	int			exitTime;
	vec3_t		intermission_origin;
	vec3_t		intermission_angle;
	qboolean	locationLinked;
	gentity_t	*locationHead;
	int			bodyQueIndex;
	gentity_t	*bodyQue[BODY_QUEUE_SIZE];
	int			portalSequence;
	alertEvent_t	alertEvents[ MAX_ALERT_EVENTS ];
	int				numAlertEvents;
	int				curAlertID;
	AIGroupInfo_t	groups[MAX_FRAME_GROUPS];
	interestPoint_t	interestPoints[MAX_INTEREST_POINTS];
	int			numInterestPoints;
	combatPoint_t	combatPoints[MAX_COMBAT_POINTS];
	int			numCombatPoints;
	int			mNumBSPInstances;
	int			mBSPInstanceDepth;
	vec3_t		mOriginAdjust;
	float		mRotationAdjust;
	char		*mTargetAdjust;
	char		mTeamFilter[MAX_QPATH];
} level_locals_t;

size_t jka_gl_sizeof_combatPoint_t(void) { return sizeof(combatPoint_t); }
size_t jka_gl_sizeof_waypointData_t(void) { return sizeof(waypointData_t); }
size_t jka_gl_sizeof_reference_tag_t(void) { return sizeof(reference_tag_t); }
size_t jka_gl_sizeof_bot_settings_t(void) { return sizeof(bot_settings_t); }
size_t jka_gl_sizeof_interestPoint_t(void) { return sizeof(interestPoint_t); }
size_t jka_gl_sizeof_alertEvent_t(void) { return sizeof(alertEvent_t); }
size_t jka_gl_off_ae_owner(void) { return offsetof(alertEvent_t, owner); }
size_t jka_gl_sizeof_level_locals_t(void) { return sizeof(level_locals_t); }
size_t jka_gl_off_ll_groups(void) { return offsetof(level_locals_t, groups); }
size_t jka_gl_off_ll_combatPoints(void) { return offsetof(level_locals_t, combatPoints); }
size_t jka_gl_off_ll_mTeamFilter(void) { return offsetof(level_locals_t, mTeamFilter); }
