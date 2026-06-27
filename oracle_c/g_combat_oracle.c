/* Oracle extraction of g_combat.c's pure hit-location classifier
 * G_GetHitLocation (refs/raven-jediacademy/codemp/game/g_combat.c:45). The function
 * reads only target->client (presence), target->r's currentAngles[YAW] / absmin /
 * absmax / mins / maxs, and the impact point, so the prelude carries a minimal
 * gentity_t / entityShared_t with just those fields. The vector ops are q_shared.h
 * macros, reproduced verbatim; AngleVectors / VectorCompare / VectorNormalize are
 * the real symbols from q_math_oracle.c in the same liblmd_oracle.a (declared extern,
 * resolved at link time, already bit-exact verified against q_math.rs). A thin
 * `jka_` wrapper marshals scalar/array test inputs into the struct so the Rust test
 * needs no cross-FFI struct-layout match. Renamed `jka_` to avoid colliding with the
 * test binary. */

#include <stddef.h> /* NULL */
#include <string.h> /* memset */
#include <math.h>   /* fabs */

typedef float vec_t;
typedef vec_t vec3_t[3];
typedef int qboolean;

#define YAW 1

#define HL_NONE     0
#define HL_FOOT_RT  1
#define HL_FOOT_LT  2
#define HL_LEG_RT   3
#define HL_LEG_LT   4
#define HL_WAIST    5
#define HL_BACK_RT  6
#define HL_BACK_LT  7
#define HL_BACK     8
#define HL_CHEST_RT 9
#define HL_CHEST_LT 10
#define HL_CHEST    11
#define HL_ARM_RT   12
#define HL_ARM_LT   13
#define HL_HAND_RT  14
#define HL_HAND_LT  15
#define HL_HEAD     16

#define VectorCopy(a, b)     ((b)[0] = (a)[0], (b)[1] = (a)[1], (b)[2] = (a)[2])
#define VectorAdd(a, b, c)   ((c)[0] = (a)[0] + (b)[0], (c)[1] = (a)[1] + (b)[1], (c)[2] = (a)[2] + (b)[2])
#define VectorSubtract(a, b, c) ((c)[0] = (a)[0] - (b)[0], (c)[1] = (a)[1] - (b)[1], (c)[2] = (a)[2] - (b)[2])
#define VectorScale(v, s, o) ((o)[0] = (v)[0] * (s), (o)[1] = (v)[1] * (s), (o)[2] = (v)[2] * (s))
#define VectorSet(v, x, y, z) ((v)[0] = (x), (v)[1] = (y), (v)[2] = (z))
#define DotProduct(a, b)     ((a)[0] * (b)[0] + (a)[1] * (b)[1] + (a)[2] * (b)[2])

extern void AngleVectors(const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up);
extern int VectorCompare(const vec3_t v1, const vec3_t v2);
extern float VectorNormalize(vec3_t v);

static vec3_t vec3_origin = {0, 0, 0};

typedef struct {
	vec3_t currentAngles;
	vec3_t currentOrigin;
	vec3_t mins, maxs;
	vec3_t absmin, absmax;
} entityShared_t;

typedef struct gentity_s {
	void *client;
	entityShared_t r;
} gentity_t;

/* ---- verbatim g_combat.c:45 body (struct accesses unchanged) ---- */
int G_GetHitLocation(gentity_t *target, vec3_t ppoint)
{
	vec3_t			point, point_dir;
	vec3_t			forward, right, up;
	vec3_t			tangles, tcenter;
	float			tradius;
	float			udot, fdot, rdot;
	int				Vertical, Forward, Lateral;
	int				HitLoc;

	// Get target forward, right and up.
	if(target->client)
	{
		// Ignore player's pitch and roll.
		VectorSet(tangles, 0, target->r.currentAngles[YAW], 0);
	}

	AngleVectors(tangles, forward, right, up);

	// Get center of target.
	VectorAdd(target->r.absmin, target->r.absmax, tcenter);
	VectorScale(tcenter, 0.5, tcenter);

	// Get radius width of target.
	tradius = (fabs(target->r.maxs[0]) + fabs(target->r.maxs[1]) + fabs(target->r.mins[0]) + fabs(target->r.mins[1]))/4;
	(void)tradius; /* dead in the original too (cylinder-projection block is commented out) */

	// Get impact point.
	if(ppoint && !VectorCompare(ppoint, vec3_origin))
	{
		VectorCopy(ppoint, point);
	}
	else
	{
		return HL_NONE;
	}

	VectorSubtract(point, tcenter, point_dir);
	VectorNormalize(point_dir);

	// Get bottom to top (vertical) position index
	udot = DotProduct(up, point_dir);
	if(udot>.800)
	{
		Vertical = 4;
	}
	else if(udot>.400)
	{
		Vertical = 3;
	}
	else if(udot>-.333)
	{
		Vertical = 2;
	}
	else if(udot>-.666)
	{
		Vertical = 1;
	}
	else
	{
		Vertical = 0;
	}

	// Get back to front (forward) position index.
	fdot = DotProduct(forward, point_dir);
	if(fdot>.666)
	{
		Forward = 4;
	}
	else if(fdot>.333)
	{
		Forward = 3;
	}
	else if(fdot>-.333)
	{
		Forward = 2;
	}
	else if(fdot>-.666)
	{
		Forward = 1;
	}
	else
	{
		Forward = 0;
	}

	// Get left to right (lateral) position index.
	rdot = DotProduct(right, point_dir);
	if(rdot>.666)
	{
		Lateral = 4;
	}
	else if(rdot>.333)
	{
		Lateral = 3;
	}
	else if(rdot>-.333)
	{
		Lateral = 2;
	}
	else if(rdot>-.666)
	{
		Lateral = 1;
	}
	else
	{
		Lateral = 0;
	}

	HitLoc = Vertical * 25 + Forward * 5 + Lateral;

	if(HitLoc <= 10)
	{
		// Feet.
		if ( rdot > 0 )
		{
			return HL_FOOT_RT;
		}
		else
		{
			return HL_FOOT_LT;
		}
	}
	else if(HitLoc <= 50)
	{
		// Legs.
		if ( rdot > 0 )
		{
			return HL_LEG_RT;
		}
		else
		{
			return HL_LEG_LT;
		}
	}
	else if(HitLoc == 56||HitLoc == 60||HitLoc == 61||HitLoc == 65||HitLoc == 66||HitLoc == 70)
	{
		// Hands.
		if ( rdot > 0 )
		{
			return HL_HAND_RT;
		}
		else
		{
			return HL_HAND_LT;
		}
	}
	else if(HitLoc == 83||HitLoc == 87||HitLoc == 88||HitLoc == 92||HitLoc == 93||HitLoc == 97)
	{
		// Arms.
		if ( rdot > 0 )
		{
			return HL_ARM_RT;
		}
		else
		{
			return HL_ARM_LT;
		}
	}
	else if((HitLoc >= 107 && HitLoc <= 109)||(HitLoc >= 112 && HitLoc <= 114)||(HitLoc >= 117 && HitLoc <= 119))
	{
		// Head.
		return HL_HEAD;
	}
	else
	{
		if(udot < 0.3)
		{
			return HL_WAIST;
		}
		else if(fdot < 0)
		{
			if(rdot > 0.4)
			{
				return HL_BACK_RT;
			}
			else if(rdot < -0.4)
			{
				return HL_BACK_LT;
			}
			else if(fdot < 0)
			{
				return HL_BACK;
			}
		}
		else
		{
			if(rdot > 0.3)
			{
				return HL_CHEST_RT;
			}
			else if(rdot < -0.3)
			{
				return HL_CHEST_LT;
			}
			else if(fdot < 0)
			{
				return HL_CHEST;
			}
		}
	}
	return HL_NONE;
}

/* test glue: marshal scalar/array inputs into the struct and call the real body */
int jka_G_GetHitLocation(int hasClient, float yaw,
	const float *absmin, const float *absmax,
	const float *mins, const float *maxs, const float *ppoint)
{
	gentity_t target;
	vec3_t pp;
	memset(&target, 0, sizeof(target));
	target.client = hasClient ? (void *)&target : NULL;
	target.r.currentAngles[YAW] = yaw;
	VectorCopy(absmin, target.r.absmin);
	VectorCopy(absmax, target.r.absmax);
	VectorCopy(mins, target.r.mins);
	VectorCopy(maxs, target.r.maxs);
	VectorCopy(ppoint, pp);
	return G_GetHitLocation(&target, pp);
}

/* ---- verbatim g_combat.c:3069 body (NPC limb-position approximation) ---- */
#define G2_MODELPART_HEAD  10
#define G2_MODELPART_WAIST 11
#define G2_MODELPART_LARM  12
#define G2_MODELPART_RARM  13
#define G2_MODELPART_RHAND 14
#define G2_MODELPART_LLEG  15
#define G2_MODELPART_RLEG  16

void G_GetDismemberLoc(gentity_t *self, vec3_t boltPoint, int limbType)
{ //Just get the general area without using server-side ghoul2
	vec3_t fwd, right, up;

	AngleVectors(self->r.currentAngles, fwd, right, up);

	VectorCopy(self->r.currentOrigin, boltPoint);

	switch (limbType)
	{
	case G2_MODELPART_HEAD:
		boltPoint[0] += up[0]*24;
		boltPoint[1] += up[1]*24;
		boltPoint[2] += up[2]*24;
		break;
	case G2_MODELPART_WAIST:
		boltPoint[0] += up[0]*4;
		boltPoint[1] += up[1]*4;
		boltPoint[2] += up[2]*4;
		break;
	case G2_MODELPART_LARM:
		boltPoint[0] += up[0]*18;
		boltPoint[1] += up[1]*18;
		boltPoint[2] += up[2]*18;

		boltPoint[0] -= right[0]*10;
		boltPoint[1] -= right[1]*10;
		boltPoint[2] -= right[2]*10;
		break;
	case G2_MODELPART_RARM:
		boltPoint[0] += up[0]*18;
		boltPoint[1] += up[1]*18;
		boltPoint[2] += up[2]*18;

		boltPoint[0] += right[0]*10;
		boltPoint[1] += right[1]*10;
		boltPoint[2] += right[2]*10;
		break;
	case G2_MODELPART_RHAND:
		boltPoint[0] += up[0]*8;
		boltPoint[1] += up[1]*8;
		boltPoint[2] += up[2]*8;

		boltPoint[0] += right[0]*10;
		boltPoint[1] += right[1]*10;
		boltPoint[2] += right[2]*10;
		break;
	case G2_MODELPART_LLEG:
		boltPoint[0] -= up[0]*4;
		boltPoint[1] -= up[1]*4;
		boltPoint[2] -= up[2]*4;

		boltPoint[0] -= right[0]*10;
		boltPoint[1] -= right[1]*10;
		boltPoint[2] -= right[2]*10;
		break;
	case G2_MODELPART_RLEG:
		boltPoint[0] -= up[0]*4;
		boltPoint[1] -= up[1]*4;
		boltPoint[2] -= up[2]*4;

		boltPoint[0] += right[0]*10;
		boltPoint[1] += right[1]*10;
		boltPoint[2] += right[2]*10;
		break;
	default:
		break;
	}

	return;
}

/* test glue: marshal angles/origin into the struct and return the written boltPoint */
void jka_G_GetDismemberLoc(const float *angles, const float *origin, int limbType, float *out)
{
	gentity_t self;
	memset(&self, 0, sizeof(self));
	VectorCopy(angles, self.r.currentAngles);
	VectorCopy(origin, self.r.currentOrigin);
	G_GetDismemberLoc(&self, out, limbType);
}

/* ---- verbatim g_combat.c:2951 body (G_ApplyKnockback) ----
 * The minimal gentity_t above carries only client/r, so this transcribes the body
 * onto the marshalled scalars/arrays rather than a struct: the arithmetic, branch
 * order, and float/double promotion are line-for-line identical to the original
 * (the `* 0.8`/`* 1.5` double literals promote through `double` exactly as the C
 * VectorScale macro / kvel[2] statement do). g_knockback.value/g_gravity.value and
 * level.time arrive as the gravityValue/knockbackValue/levelTime inputs. All
 * mutable target fields are passed in AND back out so fields a given branch leaves
 * untouched match the Rust side trivially. */
#define TR_STATIONARY      0
#define TR_LINEAR_STOP     3
#define TR_NONLINEAR_STOP  4
#define PMF_TIME_KNOCKBACK 64

void jka_G_ApplyKnockback(
	float physicsBounce, int hasClient, int trType,
	const float *newDir, float knockback,
	float gravityValue, float knockbackValue, int levelTime,
	const float *inVelocity, const float *inTrDelta,
	const float *inTrBase, const float *inCurrentOrigin,
	int inTrTime, int inPmTime, int inPmFlags,
	float *outVelocity, float *outTrDelta, float *outTrBase,
	int *outTrTime, int *outPmTime, int *outPmFlags)
{
	vec3_t kvel;
	float mass;

	vec3_t velocity, trDelta, trBase;
	int trTimeOut, pm_time, pm_flags;
	VectorCopy(inVelocity, velocity);
	VectorCopy(inTrDelta, trDelta);
	VectorCopy(inTrBase, trBase);
	trTimeOut = inTrTime;
	pm_time = inPmTime;
	pm_flags = inPmFlags;

	if ( physicsBounce > 0 )	/* overide the mass */
		mass = physicsBounce;
	else
		mass = 200;

	if ( gravityValue > 0 )
	{
		VectorScale( newDir, knockbackValue * (float)knockback / mass * 0.8, kvel );
		kvel[2] = newDir[2] * knockbackValue * (float)knockback / mass * 1.5;
	}
	else
	{
		VectorScale( newDir, knockbackValue * (float)knockback / mass, kvel );
	}

	if ( hasClient )
	{
		VectorAdd( velocity, kvel, velocity );
	}
	else if ( trType != TR_STATIONARY && trType != TR_LINEAR_STOP && trType != TR_NONLINEAR_STOP )
	{
		VectorAdd( trDelta, kvel, trDelta );
		VectorCopy( inCurrentOrigin, trBase );
		trTimeOut = levelTime;
	}

	if ( hasClient && !pm_time )
	{
		int		t;

		t = knockback * 2;
		if ( t < 50 ) {
			t = 50;
		}
		if ( t > 200 ) {
			t = 200;
		}
		pm_time = t;
		pm_flags |= PMF_TIME_KNOCKBACK;
	}

	VectorCopy(velocity, outVelocity);
	VectorCopy(trDelta, outTrDelta);
	VectorCopy(trBase, outTrBase);
	*outTrTime = trTimeOut;
	*outPmTime = pm_time;
	*outPmFlags = pm_flags;
}

/* ---- verbatim g_combat.c:3005 body (RaySphereIntersections) ----
 * Pure math (no struct access): normalizes dir in place via the real VectorNormalize
 * (extern, q_math_oracle.c), solves the quadratic and writes up to two hit points.
 * sqrt() is libm's double routine (math.h, included above), matching the Rust f64
 * round-trip. VectorMA is the q_shared.h macro, reproduced verbatim. */
#define VectorMA(v, s, b, o) \
	((o)[0] = (v)[0] + (b)[0] * (s), (o)[1] = (v)[1] + (b)[1] * (s), (o)[2] = (v)[2] + (b)[2] * (s))

int RaySphereIntersections( vec3_t origin, float radius, vec3_t point, vec3_t dir, vec3_t intersections[2] ) {
	float b, c, d, t;

	// normalize dir so a = 1
	VectorNormalize(dir);
	b = 2 * (dir[0] * (point[0] - origin[0]) + dir[1] * (point[1] - origin[1]) + dir[2] * (point[2] - origin[2]));
	c = (point[0] - origin[0]) * (point[0] - origin[0]) +
		(point[1] - origin[1]) * (point[1] - origin[1]) +
		(point[2] - origin[2]) * (point[2] - origin[2]) -
		radius * radius;

	d = b * b - 4 * c;
	if (d > 0) {
		t = (- b + sqrt(d)) / 2;
		VectorMA(point, t, dir, intersections[0]);
		t = (- b - sqrt(d)) / 2;
		VectorMA(point, t, dir, intersections[1]);
		return 2;
	}
	else if (d == 0) {
		t = (- b ) / 2;
		VectorMA(point, t, dir, intersections[0]);
		return 1;
	}
	return 0;
}

/* Marshals flat float inputs; copies the (mutated) normalized dir and both
 * intersection slots back out so the Rust test compares the full effect. */
int jka_RaySphereIntersections(const float *origin, float radius, const float *point,
                               float *dir, float *outDir, float *out0, float *out1) {
	vec3_t o, p, dd, inter[2];
	int n;
	VectorCopy(origin, o);
	VectorCopy(point, p);
	VectorCopy(dir, dd);
	n = RaySphereIntersections(o, radius, p, dd, inter);
	VectorCopy(dd, outDir);
	VectorCopy(inter[0], out0);
	VectorCopy(inter[1], out1);
	return n;
}

/* ---- verbatim g_combat.c:852 body (G_InKnockDown) ----
 * Reads only ps->legsAnim, so the prelude carries a minimal playerState_t with
 * just that field, and the BOTH_* anim numbers are #define'd to their anims.h
 * values (KNOCKDOWN1-5 = 1219-1223, GETUP1-5 = 1224-1228, FORCE_GETUP_F1/F2 =
 * 1231/1232, FORCE_GETUP_B1-B5 = 1233-1237). The two crouch get-ups (1229/1230)
 * are intentionally absent from the switch, matching the original. */
#define BOTH_KNOCKDOWN1    1219
#define BOTH_KNOCKDOWN2    1220
#define BOTH_KNOCKDOWN3    1221
#define BOTH_KNOCKDOWN4    1222
#define BOTH_KNOCKDOWN5    1223
#define BOTH_GETUP1        1224
#define BOTH_GETUP2        1225
#define BOTH_GETUP3        1226
#define BOTH_GETUP4        1227
#define BOTH_GETUP5        1228
#define BOTH_FORCE_GETUP_F1 1231
#define BOTH_FORCE_GETUP_F2 1232
#define BOTH_FORCE_GETUP_B1 1233
#define BOTH_FORCE_GETUP_B2 1234
#define BOTH_FORCE_GETUP_B3 1235
#define BOTH_FORCE_GETUP_B4 1236
#define BOTH_FORCE_GETUP_B5 1237

#define qtrue 1
#define qfalse 0

typedef struct {
	int legsAnim;
} ik_playerState_t;

qboolean G_InKnockDown( ik_playerState_t *ps )
{
	switch ( (ps->legsAnim) )
	{
	case BOTH_KNOCKDOWN1:
	case BOTH_KNOCKDOWN2:
	case BOTH_KNOCKDOWN3:
	case BOTH_KNOCKDOWN4:
	case BOTH_KNOCKDOWN5:
		return qtrue;
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
		return qtrue;
		break;
	}
	return qfalse;
}

/* Wraps the bare legsAnim int into the minimal playerState for the test. */
int jka_G_InKnockDown(int legsAnim) {
	ik_playerState_t ps;
	ps.legsAnim = legsAnim;
	return G_InKnockDown(&ps);
}

/* ---- verbatim g_combat.c:4328 body (G_Knockdown) + its BG_KnockDownable gate ----
 * The function touches only victim->client->ps and level.time. BG_KnockDownable
 * (bg_pmove.c) reads ps->m_iVehicleNum and ps->emplacedIndex; the body writes
 * forceHandExtend/forceDodgeAnim/forceHandExtendTime/quickerGetup. HANDEXTEND_KNOCKDOWN
 * is forceHandAnims_t == 8 (bg_public.h). The prelude carries a minimal playerState
 * with just those six fields plus a file-local level.time. qtrue/qfalse are already
 * #define'd above (G_InKnockDown section). */
#define HANDEXTEND_KNOCKDOWN 8

typedef struct {
	int m_iVehicleNum;
	int emplacedIndex;
	int forceHandExtend;
	int forceDodgeAnim;
	int forceHandExtendTime;
	int quickerGetup;
} kd_playerState_t;

static struct { int time; } kd_level;

static qboolean BG_KnockDownable_kd(kd_playerState_t *ps)
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

static void G_Knockdown_kd( kd_playerState_t *ps )
{
	if ( ps && BG_KnockDownable_kd(ps) )
	{
		ps->forceHandExtend = HANDEXTEND_KNOCKDOWN;
		ps->forceDodgeAnim = 0;
		ps->forceHandExtendTime = kd_level.time + 1100;
		ps->quickerGetup = qfalse;
	}
}

/* Marshals the two BG_KnockDownable gates + level.time and the four mutable fields
 * in/out so the Rust test needs no struct-layout match. The four out-fields are
 * seeded by the caller so the not-knockdownable branch (fields left untouched) is
 * observable. */
void jka_G_Knockdown(int m_iVehicleNum, int emplacedIndex, int levelTime,
                     int *forceHandExtend, int *forceDodgeAnim,
                     int *forceHandExtendTime, int *quickerGetup)
{
	kd_playerState_t ps;
	ps.m_iVehicleNum = m_iVehicleNum;
	ps.emplacedIndex = emplacedIndex;
	ps.forceHandExtend = *forceHandExtend;
	ps.forceDodgeAnim = *forceDodgeAnim;
	ps.forceHandExtendTime = *forceHandExtendTime;
	ps.quickerGetup = *quickerGetup;
	kd_level.time = levelTime;
	G_Knockdown_kd(&ps);
	*forceHandExtend = ps.forceHandExtend;
	*forceDodgeAnim = ps.forceDodgeAnim;
	*forceHandExtendTime = ps.forceHandExtendTime;
	*quickerGetup = ps.quickerGetup;
}

/* ---- verbatim g_combat.c:881 body (G_CheckSpecialDeathAnim) ----
 * The function reads only self->client->ps (legsAnim/legsTimer/viewangles/velocity) and
 * self->localAnimIndex; point/damage/mod/hitLoc are unused. The first two branches gate on
 * BG_InRoll / BG_FlippingAnim (passed in as flags), then a knockdown switch keyed on
 * G_InKnockDown (reused above) and animLength. animLength is
 * bgAllAnims[localAnimIndex].anims[legsAnim].numFrames * fabs((float)
 * bgHumanoidAnimations[legsAnim].frameLerp) -- the two scalars (numFrames/frameLerp) are
 * marshalled in so no anim-table is needed; the multiply/cast is reproduced verbatim.
 * AngleVectors / DotProduct are the real q_math symbols (linked) / q_shared macro. The
 * result BOTH_* anims and the extra get-up anims not already #define'd above are added here
 * with their anims.rs values. */
#define BOTH_DEATH14         22
#define BOTH_DEATHFORWARD3   36
#define BOTH_DEATHBACKWARD1  37
#define BOTH_DEATHBACKWARD2  38
#define BOTH_DEATH_ROLL      45
#define BOTH_DEATH_FLIP      46
#define BOTH_DEATH_SPIN_180  49
#define BOTH_DEATH_LYING_UP  50
#define BOTH_DEATH_LYING_DN  51
#define BOTH_DEATH_FALLING_DN 52
#define BOTH_DEATH_FALLING_UP 53
#define BOTH_DEATH_CROUCHED  54
#define BOTH_GETUP_CROUCH_F1 1229
#define BOTH_GETUP_CROUCH_B1 1230
#define BOTH_FORCE_GETUP_B6  1238

typedef struct {
	int legsAnim;
	int legsTimer;
	vec3_t viewangles;
	vec3_t velocity;
} sda_playerState_t;

int jka_G_CheckSpecialDeathAnim(int legsAnim, int legsTimer,
                                const float *viewangles, const float *velocity,
                                int inRoll, int flipping,
                                unsigned short numFrames, short frameLerp)
{
	sda_playerState_t ps;
	ik_playerState_t kps;
	int deathAnim = -1;

	ps.legsAnim = legsAnim;
	ps.legsTimer = legsTimer;
	VectorCopy(viewangles, ps.viewangles);
	VectorCopy(velocity, ps.velocity);
	kps.legsAnim = legsAnim;

	if ( inRoll )
	{
		deathAnim = BOTH_DEATH_ROLL;		//# Death anim from a roll
	}
	else if ( flipping )
	{
		deathAnim = BOTH_DEATH_FLIP;		//# Death anim from a flip
	}
	else if ( G_InKnockDown( &kps ) )
	{//since these happen a lot, let's handle them case by case
		int animLength = numFrames * fabs((float)(frameLerp));
		switch ( ps.legsAnim )
		{
		case BOTH_KNOCKDOWN1:
			if ( animLength - ps.legsTimer > 100 )
			{//on our way down
				if ( ps.legsTimer > 600 )
				{//still partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_KNOCKDOWN2:
			if ( animLength - ps.legsTimer > 700 )
			{//on our way down
				if ( ps.legsTimer > 600 )
				{//still partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_KNOCKDOWN3:
			if ( animLength - ps.legsTimer > 100 )
			{//on our way down
				if ( ps.legsTimer > 1300 )
				{//still partially up
					deathAnim = BOTH_DEATH_FALLING_DN;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_DN;
				}
			}
			break;
		case BOTH_KNOCKDOWN4:
			if ( animLength - ps.legsTimer > 300 )
			{//on our way down
				if ( ps.legsTimer > 350 )
				{//still partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			else
			{//crouch death
				vec3_t fwd;
				float thrown = 0;

				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			break;
		case BOTH_KNOCKDOWN5:
			if ( ps.legsTimer < 750 )
			{//flat
				deathAnim = BOTH_DEATH_LYING_DN;
			}
			break;
		case BOTH_GETUP1:
			if ( ps.legsTimer < 350 )
			{//standing up
			}
			else if ( ps.legsTimer < 800 )
			{//crouching
				vec3_t fwd;
				float thrown = 0;

				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 450 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_GETUP2:
			if ( ps.legsTimer < 150 )
			{//standing up
			}
			else if ( ps.legsTimer < 850 )
			{//crouching
				vec3_t fwd;
				float thrown = 0;

				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 500 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_GETUP3:
			if ( ps.legsTimer < 250 )
			{//standing up
			}
			else if ( ps.legsTimer < 600 )
			{//crouching
				vec3_t fwd;
				float thrown = 0;
				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 150 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_DN;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_DN;
				}
			}
			break;
		case BOTH_GETUP4:
			if ( ps.legsTimer < 250 )
			{//standing up
			}
			else if ( ps.legsTimer < 600 )
			{//crouching
				vec3_t fwd;
				float thrown = 0;

				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 850 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_DN;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_GETUP5:
			if ( ps.legsTimer > 850 )
			{//lying down
				if ( animLength - ps.legsTimer > 1500 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_DN;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_DN;
				}
			}
			break;
		case BOTH_GETUP_CROUCH_B1:
			if ( ps.legsTimer < 800 )
			{//crouching
				vec3_t fwd;
				float thrown = 0;

				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 400 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_GETUP_CROUCH_F1:
			if ( ps.legsTimer < 800 )
			{//crouching
				vec3_t fwd;
				float thrown = 0;

				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 150 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_DN;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_DN;
				}
			}
			break;
		case BOTH_FORCE_GETUP_B1:
			if ( ps.legsTimer < 325 )
			{//standing up
			}
			else if ( ps.legsTimer < 725 )
			{//spinning up
				deathAnim = BOTH_DEATH_SPIN_180;	//# Death anim when facing backwards
			}
			else if ( ps.legsTimer < 900 )
			{//crouching
				vec3_t fwd;
				float thrown = 0;

				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 50 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_FORCE_GETUP_B2:
			if ( ps.legsTimer < 575 )
			{//standing up
			}
			else if ( ps.legsTimer < 875 )
			{//spinning up
				deathAnim = BOTH_DEATH_SPIN_180;	//# Death anim when facing backwards
			}
			else if ( ps.legsTimer < 900 )
			{//crouching
				vec3_t fwd;
				float thrown = 0;

				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			else
			{//lying down
				//partially up
				deathAnim = BOTH_DEATH_FALLING_UP;
			}
			break;
		case BOTH_FORCE_GETUP_B3:
			if ( ps.legsTimer < 150 )
			{//standing up
			}
			else if ( ps.legsTimer < 775 )
			{//flipping
				deathAnim = BOTH_DEATHBACKWARD2; //backflip
			}
			else
			{//lying down
				//partially up
				deathAnim = BOTH_DEATH_FALLING_UP;
			}
			break;
		case BOTH_FORCE_GETUP_B4:
			if ( ps.legsTimer < 325 )
			{//standing up
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 150 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_FORCE_GETUP_B5:
			if ( ps.legsTimer < 550 )
			{//standing up
			}
			else if ( ps.legsTimer < 1025 )
			{//kicking up
				deathAnim = BOTH_DEATHBACKWARD2; //backflip
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 50 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_FORCE_GETUP_B6:
			if ( ps.legsTimer < 225 )
			{//standing up
			}
			else if ( ps.legsTimer < 425 )
			{//crouching up
				vec3_t fwd;
				float thrown = 0;

				AngleVectors( ps.viewangles, fwd, NULL, NULL );
				thrown = DotProduct( fwd, ps.velocity );

				if ( thrown < -150 )
				{
					deathAnim = BOTH_DEATHBACKWARD1;	//# Death anim when crouched and thrown back
				}
				else
				{
					deathAnim = BOTH_DEATH_CROUCHED;	//# Death anim when crouched
				}
			}
			else if ( ps.legsTimer < 825 )
			{//flipping up
				deathAnim = BOTH_DEATHFORWARD3; //backflip
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 225 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_UP;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_UP;
				}
			}
			break;
		case BOTH_FORCE_GETUP_F1:
			if ( ps.legsTimer < 275 )
			{//standing up
			}
			else if ( ps.legsTimer < 750 )
			{//flipping
				deathAnim = BOTH_DEATH14;
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 100 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_DN;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_DN;
				}
			}
			break;
		case BOTH_FORCE_GETUP_F2:
			if ( ps.legsTimer < 1200 )
			{//standing
			}
			else
			{//lying down
				if ( animLength - ps.legsTimer > 225 )
				{//partially up
					deathAnim = BOTH_DEATH_FALLING_DN;
				}
				else
				{//down
					deathAnim = BOTH_DEATH_LYING_DN;
				}
			}
			break;
		}
	}

	return deathAnim;
}

/* ---- verbatim g_combat.c:1373 hit-location selector (G_PickDeathAnim) ----
 * The Rust test exercises the meat of the function -- the hitLoc switch plus its Q_irand
 * variety -- so this wrapper assumes the early returns are not taken (isClient, not
 * inSpace, gGAvoidDismember==0, legAnim not a dead-flop so the dead-flop switch leaves
 * deathAnim==-1, hitLoc already resolved so G_GetHitLocation is skipped) and that the
 * validation tail's BG_HasAnimation passes (the test stocks the anim table full so
 * BG_PickAnim is never reached). specialDeathAnim is the marshalled G_CheckSpecialDeathAnim
 * result (-1 -> run the switch). Q_irand is the PC wrapper over irand (the holdrand MSVC
 * LCG, in q_math_oracle.c); the test seeds Rust's Rand_Init and the oracle's Rand_Init
 * identically so the draws agree in lockstep. The trivial early returns are asserted
 * directly in the test. */
#define BOTH_DEATH1   9
#define BOTH_DEATH2   10
#define BOTH_DEATH3   11
#define BOTH_DEATH4   12
#define BOTH_DEATH5   13
#define BOTH_DEATH6   14
#define BOTH_DEATH7   15
#define BOTH_DEATH8   16
#define BOTH_DEATH9   17
#define BOTH_DEATH10  18
#define BOTH_DEATH11  19
#define BOTH_DEATH12  20
#define BOTH_DEATH13  21
#define BOTH_DEATH15  23
#define BOTH_DEATH16  24
#define BOTH_DEATH17  25
#define BOTH_DEATH18  26
#define BOTH_DEATH19  27
#define MOD_SABER     3

#define VLS(v) ((v)[0]*(v)[0] + (v)[1]*(v)[1] + (v)[2]*(v)[2])

extern int irand(int min, int max);

static int pda_Q_irand(int value1, int value2)
{
	return irand(value1, value2);
}

int jka_G_PickDeathAnim(int hitLoc, int damage, int mod, int max_health,
                        const float *objVelocityIn, int specialDeathAnim)
{
	int deathAnim = -1;
	vec3_t objVelocity;
	VectorCopy(objVelocityIn, objVelocity);

	deathAnim = specialDeathAnim;

	if ( deathAnim == -1 )
	{
		//death anims
		switch( hitLoc )
		{
		case HL_FOOT_RT:
		case HL_FOOT_LT:
			if ( mod == MOD_SABER && !pda_Q_irand( 0, 2 ) )
			{
				return BOTH_DEATH10;//chest: back flip
			}
			else if ( !pda_Q_irand( 0, 2 ) )
			{
				deathAnim = BOTH_DEATH4;//back: forward
			}
			else if ( !pda_Q_irand( 0, 1 ) )
			{
				deathAnim = BOTH_DEATH5;//same as 4
			}
			else
			{
				deathAnim = BOTH_DEATH15;//back: forward
			}
			break;
		case HL_LEG_RT:
			if ( !pda_Q_irand( 0, 2 ) )
			{
				deathAnim = BOTH_DEATH4;//back: forward
			}
			else if ( !pda_Q_irand( 0, 1 ) )
			{
				deathAnim = BOTH_DEATH5;//same as 4
			}
			else
			{
				deathAnim = BOTH_DEATH15;//back: forward
			}
			break;
		case HL_LEG_LT:
			if ( !pda_Q_irand( 0, 2 ) )
			{
				deathAnim = BOTH_DEATH4;//back: forward
			}
			else if ( !pda_Q_irand( 0, 1 ) )
			{
				deathAnim = BOTH_DEATH5;//same as 4
			}
			else
			{
				deathAnim = BOTH_DEATH15;//back: forward
			}
			break;
		case HL_BACK:
			if ( !VLS( objVelocity ) )
			{
				deathAnim = BOTH_DEATH17;//head/back: croak
			}
			else
			{
				if ( !pda_Q_irand( 0, 2 ) )
				{
					deathAnim = BOTH_DEATH4;//back: forward
				}
				else if ( !pda_Q_irand( 0, 1 ) )
				{
					deathAnim = BOTH_DEATH5;//same as 4
				}
				else
				{
					deathAnim = BOTH_DEATH15;//back: forward
				}
			}
			break;
		case HL_CHEST_RT:
		case HL_ARM_RT:
		case HL_HAND_RT:
		case HL_BACK_RT:
			if ( damage <= max_health*0.25 )
			{
				deathAnim = BOTH_DEATH9;//chest right: snap, fall forward
			}
			else if ( damage <= max_health*0.5 )
			{
				deathAnim = BOTH_DEATH3;//chest right: back
			}
			else if ( damage <= max_health*0.75 )
			{
				deathAnim = BOTH_DEATH6;//chest right: spin
			}
			else
			{
				//TEMP HACK: play spinny deaths less often
				if ( pda_Q_irand( 0, 1 ) )
				{
					deathAnim = BOTH_DEATH8;//chest right: spin high
				}
				else
				{
					switch ( pda_Q_irand( 0, 2 ) )
					{
					default:
					case 0:
						deathAnim = BOTH_DEATH9;//chest right: snap, fall forward
						break;
					case 1:
						deathAnim = BOTH_DEATH3;//chest right: back
						break;
					case 2:
						deathAnim = BOTH_DEATH6;//chest right: spin
						break;
					}
				}
			}
			break;
		case HL_CHEST_LT:
		case HL_ARM_LT:
		case HL_HAND_LT:
		case HL_BACK_LT:
			if ( damage <= max_health*0.25 )
			{
				deathAnim = BOTH_DEATH11;//chest left: snap, fall forward
			}
			else if ( damage <= max_health*0.5 )
			{
				deathAnim = BOTH_DEATH7;//chest left: back
			}
			else if ( damage <= max_health*0.75 )
			{
				deathAnim = BOTH_DEATH12;//chest left: spin
			}
			else
			{
				//TEMP HACK: play spinny deaths less often
				if ( pda_Q_irand( 0, 1 ) )
				{
					deathAnim = BOTH_DEATH14;//chest left: spin high
				}
				else
				{
					switch ( pda_Q_irand( 0, 2 ) )
					{
					default:
					case 0:
						deathAnim = BOTH_DEATH11;//chest left: snap, fall forward
						break;
					case 1:
						deathAnim = BOTH_DEATH7;//chest left: back
						break;
					case 2:
						deathAnim = BOTH_DEATH12;//chest left: spin
						break;
					}
				}
			}
			break;
		case HL_CHEST:
		case HL_WAIST:
			if ( damage <= max_health*0.25 || !VLS( objVelocity ) )
			{
				if ( !pda_Q_irand( 0, 1 ) )
				{
					deathAnim = BOTH_DEATH18;//gut: fall right
				}
				else
				{
					deathAnim = BOTH_DEATH19;//gut: fall left
				}
			}
			else if ( damage <= max_health*0.5 )
			{
				deathAnim = BOTH_DEATH2;//chest: backward short
			}
			else if ( damage <= max_health*0.75 )
			{
				if ( !pda_Q_irand( 0, 1 ) )
				{
					deathAnim = BOTH_DEATH1;//chest: backward med
				}
				else
				{
					deathAnim = BOTH_DEATH16;//same as 1
				}
			}
			else
			{
				deathAnim = BOTH_DEATH10;//chest: back flip
			}
			break;
		case HL_HEAD:
			if ( damage <= max_health*0.5 )
			{
				deathAnim = BOTH_DEATH17;//head/back: croak
			}
			else
			{
				deathAnim = BOTH_DEATH13;//head: stumble, fall back
			}
			break;
		default:
			break;
		}
	}

	return deathAnim;
}
