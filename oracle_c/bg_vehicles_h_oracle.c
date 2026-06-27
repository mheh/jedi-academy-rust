/*
 * Oracle TU for bg_vehicles.h structs (the vehicle data layer) plus bg_public.h's
 * bgEntity_t and pmove_t, which share the Vehicle_t cluster's by-value dependencies
 * (entityState_t/usercmd_t/trace_t) -- bgEntity_t forms a mutually-recursive pointer
 * cluster with Vehicle_t, and pmove_t references bgEntity_t -- so they live in this
 * single TU rather than duplicating those deps across files.
 *
 * Several of these carry pointers (and vehicleInfo_t carries C function pointers), so
 * the layout is arch-dependent; this oracle is compiled at the host (64-bit) word
 * size, matching the `#[cfg(target_pointer_width = "64")]` asserts in the Rust port
 * (see DEVIATIONS.md). The C compiler yields the real `sizeof`/`offsetof`, which the
 * Rust port asserts against.
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
#define MAX_VEHICLE_MUZZLES 12
#define MAX_VEHICLE_EXHAUSTS 12
#define MAX_VEHICLE_WEAPONS 2
#define MAX_VEHICLE_TURRETS 2
#define MAX_VEHICLE_TURRET_MUZZLES 2
#define VEH_MAX_PASSENGERS 10
#define MAXTOUCH 32

/* ---- forward decls (pointer-only across the cluster) ---- */
typedef struct Vehicle_s Vehicle_t;
typedef struct bgEntity_s bgEntity_t;
typedef struct playerState_s playerState_t;
typedef struct animation_s animation_t;

/* ---- by-value deps (verbatim, same as g_local_oracle.c) ---- */
typedef enum { TR_STATIONARY, TR_INTERPOLATE, TR_LINEAR, TR_LINEAR_STOP, TR_NONLINEAR_STOP, TR_SINE, TR_GRAVITY } trType_t;
typedef struct { trType_t trType; int trTime; int trDuration; vec3_t trBase; vec3_t trDelta; } trajectory_t;
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

typedef int vehicleType_t;

/* ---- the standalone structs (verbatim) ---- */
typedef struct {
	char	*name;
	qboolean	bIsProjectile;
	qboolean	bHasGravity;
	qboolean	bIonWeapon;
	qboolean	bSaberBlockable;
	int		iMuzzleFX;
	int		iModel;
	int		iShotFX;
	int		iImpactFX;
	int		iG2MarkShaderHandle;
	float	fG2MarkSize;
	int		iLoopSound;
	float	fSpeed;
	float	fHoming;
	float	fHomingFOV;
	int		iLockOnTime;
	int		iDamage;
	int		iSplashDamage;
	float	fSplashRadius;
	int		iAmmoPerShot;
	int		iHealth;
	float	fWidth;
	float	fHeight;
	int		iLifeTime;
	qboolean	bExplodeOnExpire;
} vehWeaponInfo_t;

typedef struct {
	int			iWeapon;
	int			iDelay;
	int			iAmmoMax;
	int			iAmmoRechargeMS;
	char		*yawBone;
	char		*pitchBone;
	int			yawAxis;
	int			pitchAxis;
	float		yawClampLeft;
	float		yawClampRight;
	float		pitchClampUp;
	float		pitchClampDown;
	int			iMuzzle[MAX_VEHICLE_TURRET_MUZZLES];
	char		*gunnerViewTag;
	float		fTurnSpeed;
	qboolean	bAI;
	qboolean	bAILead;
	float		fAIRange;
	int			passengerNum;
} turretStats_t;

typedef struct {
	int			ID;
	int			delay;
	int			linkable;
	qboolean	aimCorrect;
	int			ammoMax;
	int			ammoRechargeMS;
	int			soundNoAmmo;
} vehWeaponStats_t;

typedef struct {
	qboolean	linked;
	int			ammo;
	int			lastAmmoInc;
	int			nextMuzzle;
} vehWeaponStatus_t;

typedef struct {
	int			ammo;
	int			lastAmmoInc;
	int			nextMuzzle;
	int			enemyEntNum;
	int			enemyHoldTime;
} vehTurretStatus_t;

/* ---- vehicleInfo_t (verbatim) ---- */
typedef struct {
	char		*name;
	vehicleType_t	type;
	int			numHands;
	float		lookPitch;
	float		lookYaw;
	float		length;
	float		width;
	float		height;
	vec3_t		centerOfGravity;
	float		speedMax;
	float		turboSpeed;
	float		speedMin;
	float		speedIdle;
	float		accelIdle;
	float		acceleration;
	float		decelIdle;
	float		throttleSticks;
	float		strafePerc;
	float		bankingSpeed;
	float		rollLimit;
	float		pitchLimit;
	float		braking;
	float		mouseYaw;
	float		mousePitch;
	float		turningSpeed;
	qboolean	turnWhenStopped;
	float		traction;
	float		friction;
	float		maxSlope;
	qboolean	speedDependantTurning;
	int			mass;
	int			armor;
	int			shields;
	int			shieldRechargeMS;
	float		toughness;
	int			malfunctionArmorLevel;
	int			surfDestruction;
	int			health_front;
	int			health_back;
	int			health_right;
	int			health_left;
	char		*model;
	char		*skin;
	int			g2radius;
	int			riderAnim;
	int			radarIconHandle;
	int			dmgIndicFrameHandle;
	int			dmgIndicShieldHandle;
	int			dmgIndicBackgroundHandle;
	int			iconFrontHandle;
	int			iconBackHandle;
	int			iconRightHandle;
	int			iconLeftHandle;
	int			crosshairShaderHandle;
	int			shieldShaderHandle;
	char		*droidNPC;
	int			soundOn;
	int			soundTakeOff;
	int			soundEngineStart;
	int			soundLoop;
	int			soundSpin;
	int			soundTurbo;
	int			soundHyper;
	int			soundLand;
	int			soundOff;
	int			soundFlyBy;
	int			soundFlyBy2;
	int			soundShift1;
	int			soundShift2;
	int			soundShift3;
	int			soundShift4;
	int			iExhaustFX;
	int			iTurboFX;
	int			iTurboStartFX;
	int			iTrailFX;
	int			iImpactFX;
	int			iExplodeFX;
	int			iWakeFX;
	int			iDmgFX;
	int			iInjureFX;
	int			iNoseFX;
	int			iLWingFX;
	int			iRWingFX;
	vehWeaponStats_t	weapon[MAX_VEHICLE_WEAPONS];
	int			weapMuzzle[MAX_VEHICLE_MUZZLES];
	turretStats_t	turret[MAX_VEHICLE_TURRETS];
	float		landingHeight;
	int			gravity;
	float		hoverHeight;
	float		hoverStrength;
	qboolean	waterProof;
	float		bouyancy;
	int			fuelMax;
	int			fuelRate;
	int			turboDuration;
	int			turboRecharge;
	int			visibility;
	int			loudness;
	float		explosionRadius;
	int			explosionDamage;
	int			maxPassengers;
	qboolean	hideRider;
	qboolean	killRiderOnDeath;
	qboolean	flammable;
	int			explosionDelay;
	qboolean	cameraOverride;
	float		cameraRange;
	float		cameraVertOffset;
	float		cameraHorzOffset;
	float		cameraPitchOffset;
	float		cameraFOV;
	float		cameraAlpha;
	qboolean	cameraPitchDependantVertOffset;
	int			modelIndex;
	void (*AnimateVehicle)( Vehicle_t *pVeh );
	void (*AnimateRiders)( Vehicle_t *pVeh );
	qboolean (*ValidateBoard)( Vehicle_t *pVeh, bgEntity_t *pEnt );
	void (*SetParent)( Vehicle_t *pVeh, bgEntity_t *pParentEntity );
	void (*SetPilot)( Vehicle_t *pVeh, bgEntity_t *pPilot );
	qboolean (*AddPassenger)( Vehicle_t *pVeh );
	void (*Animate)( Vehicle_t *pVeh );
	qboolean (*Board)( Vehicle_t *pVeh, bgEntity_t *pEnt );
	qboolean (*Eject)( Vehicle_t *pVeh, bgEntity_t *pEnt, qboolean forceEject );
	qboolean (*EjectAll)( Vehicle_t *pVeh );
	void (*StartDeathDelay)( Vehicle_t *pVeh, int iDelayTime );
	void (*DeathUpdate)( Vehicle_t *pVeh );
	void (*RegisterAssets)( Vehicle_t *pVeh );
	qboolean (*Initialize)( Vehicle_t *pVeh );
	qboolean (*Update)( Vehicle_t *pVeh, const usercmd_t *pUcmd );
	qboolean (*UpdateRider)( Vehicle_t *pVeh, bgEntity_t *pRider, usercmd_t *pUcmd );
	void (*ProcessMoveCommands)( Vehicle_t *pVeh );
	void (*ProcessOrientCommands)( Vehicle_t *pVeh );
	void (*AttachRiders)( Vehicle_t *pVeh );
	void (*Ghost)( Vehicle_t *pVeh, bgEntity_t *pEnt );
	void (*UnGhost)( Vehicle_t *pVeh, bgEntity_t *pEnt );
	const bgEntity_t *(*GetPilot)( Vehicle_t *pVeh );
	qboolean (*Inhabited)( Vehicle_t *pVeh );
} vehicleInfo_t;

/* ---- bgEntity_s (bg_public.h, verbatim) ---- */
struct bgEntity_s {
	entityState_t	s;
	playerState_t	*playerState;
	Vehicle_t		*m_pVehicle;
	void			*ghoul2;
	int				localAnimIndex;
	vec3_t			modelScale;
};

/* ---- Vehicle_s (verbatim) ---- */
struct Vehicle_s {
	bgEntity_t *m_pPilot;
	int m_iPilotTime;
	int m_iPilotLastIndex;
	qboolean m_bHasHadPilot;
	bgEntity_t *m_ppPassengers[VEH_MAX_PASSENGERS];
	bgEntity_t *m_pDroidUnit;
	int m_iNumPassengers;
	bgEntity_t *m_pParentEntity;
	int		m_iBoarding;
	qboolean	m_bWasBoarding;
	vec3_t	m_vBoardingVelocity;
	float m_fTimeModifier;
	int m_iLeftExhaustTag;
	int m_iRightExhaustTag;
	int m_iGun1Tag;
	int m_iGun1Bone;
	int m_iLeftWingBone;
	int m_iRightWingBone;
	int m_iExhaustTag[MAX_VEHICLE_EXHAUSTS];
	int m_iMuzzleTag[MAX_VEHICLE_MUZZLES];
	int m_iDroidUnitTag;
	int	m_iGunnerViewTag[MAX_VEHICLE_TURRETS];
	int m_iMuzzleTime[MAX_VEHICLE_MUZZLES];
	vec3_t m_vMuzzlePos[MAX_VEHICLE_MUZZLES], m_vMuzzleDir[MAX_VEHICLE_MUZZLES];
	int m_iMuzzleWait[MAX_VEHICLE_MUZZLES];
	usercmd_t m_ucmd;
	int m_EjectDir;
	unsigned long m_ulFlags;
	int m_iVehicleTypeID;
	float		*m_vOrientation;
	int			m_fStrafeTime;
	vec3_t		m_vPrevOrientation;
	vec3_t		m_vPrevRiderViewAngles;
	float		m_vAngularVelocity;
	vec3_t		m_vFullAngleVelocity;
	int			m_iArmor;
	int			m_iShields;
	int			m_iHitDebounce;
	int			m_iLastFXTime;
	int			m_iDieTime;
	vehicleInfo_t *m_pVehicleInfo;
	trace_t m_LandTrace;
	vec3_t m_vWingAngles;
	int			m_iLastImpactDmg;
	int			m_iRemovedSurfaces;
	int			m_iDmgEffectTime;
	int			m_iTurboTime;
	int			m_iDropTime;
	int			m_iSoundDebounceTimer;
	int			lastShieldInc;
	qboolean	linkWeaponToggleHeld;
	vehWeaponStatus_t	weaponStatus[MAX_VEHICLE_WEAPONS];
	vehTurretStatus_t	turretStatus[MAX_VEHICLE_TURRETS];
	bgEntity_t *	m_pOldPilot;
};

/* ---- pmove_t (bg_public.h, verbatim) -- references bgEntity_t + shares usercmd_t ---- */
typedef struct {
	playerState_t	*ps;
	void		*ghoul2;
	int			g2Bolts_LFoot;
	int			g2Bolts_RFoot;
	vec3_t		modelScale;
	qboolean	nonHumanoid;
	usercmd_t	cmd;
	int			tracemask;
	int			debugLevel;
	qboolean	noFootsteps;
	qboolean	gauntletHit;
	int			framecount;
	int			numtouch;
	int			touchents[MAXTOUCH];
	int			useEvent;
	vec3_t		mins, maxs;
	int			watertype;
	int			waterlevel;
	int			gametype;
	int			debugMelee;
	int			stepSlideFix;
	int			noSpecMove;
	animation_t	*animations;
	float		xyspeed;
	int			pmove_fixed;
	int			pmove_msec;
	void		(*trace)( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentMask );
	int			(*pointcontents)( const vec3_t point, int passEntityNum );
	int			checkDuelLoss;
	bgEntity_t	*baseEnt;
	int			entSize;
} pmove_t;

/* ---- accessors ---- */
size_t jka_bv_sizeof_vehWeaponInfo_t(void) { return sizeof(vehWeaponInfo_t); }
size_t jka_bv_off_vwi_bIsProjectile(void) { return offsetof(vehWeaponInfo_t, bIsProjectile); }

size_t jka_bv_sizeof_turretStats_t(void) { return sizeof(turretStats_t); }
size_t jka_bv_off_ts_yawBone(void) { return offsetof(turretStats_t, yawBone); }
size_t jka_bv_off_ts_iMuzzle(void) { return offsetof(turretStats_t, iMuzzle); }
size_t jka_bv_off_ts_gunnerViewTag(void) { return offsetof(turretStats_t, gunnerViewTag); }

size_t jka_bv_sizeof_vehWeaponStats_t(void) { return sizeof(vehWeaponStats_t); }
size_t jka_bv_sizeof_vehWeaponStatus_t(void) { return sizeof(vehWeaponStatus_t); }
size_t jka_bv_sizeof_vehTurretStatus_t(void) { return sizeof(vehTurretStatus_t); }

size_t jka_bv_sizeof_vehicleInfo_t(void) { return sizeof(vehicleInfo_t); }
size_t jka_bv_off_vi_type(void) { return offsetof(vehicleInfo_t, type); }
size_t jka_bv_off_vi_model(void) { return offsetof(vehicleInfo_t, model); }
size_t jka_bv_off_vi_weapon(void) { return offsetof(vehicleInfo_t, weapon); }
size_t jka_bv_off_vi_turret(void) { return offsetof(vehicleInfo_t, turret); }
size_t jka_bv_off_vi_modelIndex(void) { return offsetof(vehicleInfo_t, modelIndex); }
size_t jka_bv_off_vi_AnimateVehicle(void) { return offsetof(vehicleInfo_t, AnimateVehicle); }

size_t jka_bv_sizeof_bgEntity_t(void) { return sizeof(bgEntity_t); }
size_t jka_bv_off_be_playerState(void) { return offsetof(bgEntity_t, playerState); }
size_t jka_bv_off_be_m_pVehicle(void) { return offsetof(bgEntity_t, m_pVehicle); }
size_t jka_bv_off_be_modelScale(void) { return offsetof(bgEntity_t, modelScale); }

size_t jka_bv_sizeof_Vehicle_t(void) { return sizeof(Vehicle_t); }
size_t jka_bv_off_veh_m_ucmd(void) { return offsetof(Vehicle_t, m_ucmd); }
size_t jka_bv_off_veh_m_ulFlags(void) { return offsetof(Vehicle_t, m_ulFlags); }
size_t jka_bv_off_veh_m_vOrientation(void) { return offsetof(Vehicle_t, m_vOrientation); }
size_t jka_bv_off_veh_m_pVehicleInfo(void) { return offsetof(Vehicle_t, m_pVehicleInfo); }
size_t jka_bv_off_veh_m_LandTrace(void) { return offsetof(Vehicle_t, m_LandTrace); }
size_t jka_bv_off_veh_weaponStatus(void) { return offsetof(Vehicle_t, weaponStatus); }
size_t jka_bv_off_veh_turretStatus(void) { return offsetof(Vehicle_t, turretStatus); }
size_t jka_bv_off_veh_m_pOldPilot(void) { return offsetof(Vehicle_t, m_pOldPilot); }

size_t jka_bv_sizeof_pmove_t(void) { return sizeof(pmove_t); }
size_t jka_bv_off_pm_cmd(void) { return offsetof(pmove_t, cmd); }
size_t jka_bv_off_pm_mins(void) { return offsetof(pmove_t, mins); }
size_t jka_bv_off_pm_animations(void) { return offsetof(pmove_t, animations); }
size_t jka_bv_off_pm_trace(void) { return offsetof(pmove_t, trace); }
size_t jka_bv_off_pm_pointcontents(void) { return offsetof(pmove_t, pointcontents); }
size_t jka_bv_off_pm_baseEnt(void) { return offsetof(pmove_t, baseEnt); }
size_t jka_bv_off_pm_entSize(void) { return offsetof(pmove_t, entSize); }

/* ---- vehicleFields[] offset column (bg_vehicleLoad.c:454, _JK2MP config) ----
 * An INDEPENDENT transcription of the table's offset column, built with the real
 * VFOFS() macro over the (layout-verified) vehicleInfo_t above. The Rust port builds
 * the same offsets via offset_of! + nested-array arithmetic (weap_ofs/turret_ofs/...);
 * the Rust test compares this array element-wise, catching an index/field/arithmetic
 * slip on either side. The terminating {0,-1,VF_INT} row contributes -1. */
#define VFOFS(x) ((int)offsetof(vehicleInfo_t, x))
static const int jka_vehicleFields_ofs[] = {
	VFOFS(name),
	VFOFS(type), VFOFS(numHands), VFOFS(lookPitch), VFOFS(lookYaw),
	VFOFS(length), VFOFS(width), VFOFS(height), VFOFS(centerOfGravity),
	VFOFS(speedMax), VFOFS(turboSpeed), VFOFS(speedMin), VFOFS(speedIdle),
	VFOFS(accelIdle), VFOFS(acceleration), VFOFS(decelIdle), VFOFS(throttleSticks), VFOFS(strafePerc),
	VFOFS(bankingSpeed), VFOFS(pitchLimit), VFOFS(rollLimit), VFOFS(braking),
	VFOFS(mouseYaw), VFOFS(mousePitch), VFOFS(turningSpeed), VFOFS(turnWhenStopped),
	VFOFS(traction), VFOFS(friction), VFOFS(maxSlope), VFOFS(speedDependantTurning),
	VFOFS(mass), VFOFS(armor), VFOFS(shields), VFOFS(shieldRechargeMS), VFOFS(toughness),
	VFOFS(malfunctionArmorLevel), VFOFS(surfDestruction),
	VFOFS(model), VFOFS(skin), VFOFS(g2radius), VFOFS(riderAnim), VFOFS(droidNPC),
	VFOFS(radarIconHandle), VFOFS(dmgIndicFrameHandle), VFOFS(dmgIndicShieldHandle),
	VFOFS(dmgIndicBackgroundHandle), VFOFS(iconFrontHandle), VFOFS(iconBackHandle),
	VFOFS(iconRightHandle), VFOFS(iconLeftHandle), VFOFS(crosshairShaderHandle), VFOFS(shieldShaderHandle),
	VFOFS(health_front), VFOFS(health_back), VFOFS(health_right), VFOFS(health_left),
	VFOFS(soundOn), VFOFS(soundOff), VFOFS(soundLoop), VFOFS(soundTakeOff), VFOFS(soundEngineStart),
	VFOFS(soundSpin), VFOFS(soundTurbo), VFOFS(soundHyper), VFOFS(soundLand), VFOFS(soundFlyBy),
	VFOFS(soundFlyBy2), VFOFS(soundShift1), VFOFS(soundShift2), VFOFS(soundShift3), VFOFS(soundShift4),
	VFOFS(iExhaustFX), VFOFS(iTurboFX), VFOFS(iTurboStartFX), VFOFS(iTrailFX), VFOFS(iImpactFX),
	VFOFS(iExplodeFX), VFOFS(iWakeFX), VFOFS(iDmgFX),
	VFOFS(iInjureFX), VFOFS(iNoseFX), VFOFS(iLWingFX), VFOFS(iRWingFX),
	VFOFS(weapon[0].ID), VFOFS(weapon[1].ID),
	VFOFS(weapon[0].delay), VFOFS(weapon[1].delay),
	VFOFS(weapon[0].linkable), VFOFS(weapon[1].linkable),
	VFOFS(weapon[0].aimCorrect), VFOFS(weapon[1].aimCorrect),
	VFOFS(weapon[0].ammoMax), VFOFS(weapon[1].ammoMax),
	VFOFS(weapon[0].ammoRechargeMS), VFOFS(weapon[1].ammoRechargeMS),
	VFOFS(weapon[0].soundNoAmmo), VFOFS(weapon[1].soundNoAmmo),
	VFOFS(weapMuzzle[0]), VFOFS(weapMuzzle[1]), VFOFS(weapMuzzle[2]), VFOFS(weapMuzzle[3]),
	VFOFS(weapMuzzle[4]), VFOFS(weapMuzzle[5]), VFOFS(weapMuzzle[6]), VFOFS(weapMuzzle[7]),
	VFOFS(weapMuzzle[8]), VFOFS(weapMuzzle[9]),
	VFOFS(landingHeight),
	VFOFS(gravity), VFOFS(hoverHeight), VFOFS(hoverStrength), VFOFS(waterProof), VFOFS(bouyancy),
	VFOFS(fuelMax), VFOFS(fuelRate), VFOFS(turboDuration), VFOFS(turboRecharge),
	VFOFS(visibility), VFOFS(loudness), VFOFS(explosionRadius), VFOFS(explosionDamage),
	VFOFS(maxPassengers), VFOFS(hideRider), VFOFS(killRiderOnDeath), VFOFS(flammable), VFOFS(explosionDelay),
	VFOFS(cameraOverride), VFOFS(cameraRange), VFOFS(cameraVertOffset), VFOFS(cameraHorzOffset),
	VFOFS(cameraPitchOffset), VFOFS(cameraFOV), VFOFS(cameraAlpha), VFOFS(cameraPitchDependantVertOffset),
	VFOFS(turret[0].iWeapon), VFOFS(turret[0].iDelay), VFOFS(turret[0].iAmmoMax), VFOFS(turret[0].iAmmoRechargeMS),
	VFOFS(turret[0].yawBone), VFOFS(turret[0].pitchBone), VFOFS(turret[0].yawAxis), VFOFS(turret[0].pitchAxis),
	VFOFS(turret[0].yawClampLeft), VFOFS(turret[0].yawClampRight), VFOFS(turret[0].pitchClampUp), VFOFS(turret[0].pitchClampDown),
	VFOFS(turret[0].iMuzzle[0]), VFOFS(turret[0].iMuzzle[1]), VFOFS(turret[0].fTurnSpeed),
	VFOFS(turret[0].bAI), VFOFS(turret[0].bAILead), VFOFS(turret[0].fAIRange), VFOFS(turret[0].passengerNum),
	VFOFS(turret[0].gunnerViewTag),
	VFOFS(turret[1].iWeapon), VFOFS(turret[1].iDelay), VFOFS(turret[1].iAmmoMax), VFOFS(turret[1].iAmmoRechargeMS),
	VFOFS(turret[1].yawBone), VFOFS(turret[1].pitchBone), VFOFS(turret[1].yawAxis), VFOFS(turret[1].pitchAxis),
	VFOFS(turret[1].yawClampLeft), VFOFS(turret[1].yawClampRight), VFOFS(turret[1].pitchClampUp), VFOFS(turret[1].pitchClampDown),
	VFOFS(turret[1].iMuzzle[0]), VFOFS(turret[1].iMuzzle[1]), VFOFS(turret[1].fTurnSpeed),
	VFOFS(turret[1].bAI), VFOFS(turret[1].bAILead), VFOFS(turret[1].fAIRange), VFOFS(turret[1].passengerNum),
	VFOFS(turret[1].gunnerViewTag),
	-1, /* terminating {0, -1, VF_INT} */
};
int jka_vehicleFields_count(void) { return (int)(sizeof(jka_vehicleFields_ofs)/sizeof(jka_vehicleFields_ofs[0])); }
const int *jka_vehicleFields_offsets(void) { return jka_vehicleFields_ofs; }

/* ---- BG_VehicleClampData (bg_vehicleLoad.c:821) ----
 * Verbatim transcription of the clamp body, operating on the real (layout-verified)
 * vehicleInfo_t above; the Rust test runs both sides on identical struct bytes and
 * compares the whole region. Self-contained -- no engine deps. */
void jka_BG_VehicleClampData( vehicleInfo_t *vehicle )
{/*sanity check and clamp the vehicle's data*/
	int		i;

	for ( i = 0; i < 3; i++ )
	{
		if ( vehicle->centerOfGravity[i] > 1.0f )
		{
			vehicle->centerOfGravity[i] = 1.0f;
		}
		else if ( vehicle->centerOfGravity[i] < -1.0f )
		{
			vehicle->centerOfGravity[i] = -1.0f;
		}
	}

	/* Validate passenger max. */
	if ( vehicle->maxPassengers > VEH_MAX_PASSENGERS )
	{
		vehicle->maxPassengers = VEH_MAX_PASSENGERS;
	}
	else if ( vehicle->maxPassengers < 0 )
	{
		vehicle->maxPassengers = 0;
	}
}
