/*
 * Logic oracle for the bg_misc.c slice ported into src/codemp/game/bg_misc.rs.
 *
 * Currently covers BG_AddPredictableEventToPlayerstate (bg_misc.c:2644): the
 * playerstate event-ring writer. The real bg_misc.c cannot be `#include`d (its quoted
 * `#include`s drag in the clang-hostile reference tree), so the non-_DEBUG body is
 * transcribed here VERBATIM and run over a MINIMAL playerState_t holding only the
 * three fields it touches (the playerstate-predicate / BG_ForcePowerDrain precedent:
 * never cross a full struct over FFI; the full playerState_t layout is verified
 * separately in q_shared_h_oracle.c). An int-marshalling wrapper drives one call and
 * returns the resulting ring state. Built only under `oracle`.
 */

#define MAX_PS_EVENTS 2 /* q_shared.h */

typedef struct {
	int eventSequence;
	int events[MAX_PS_EVENTS];
	int eventParms[MAX_PS_EVENTS];
} ps_ev_min_t;

#define playerState_t ps_ev_min_t

/* verbatim non-_DEBUG body of bg_misc.c:2644 */
void BG_AddPredictableEventToPlayerstate( int newEvent, int eventParm, playerState_t *ps ) {
	ps->events[ps->eventSequence & (MAX_PS_EVENTS-1)] = newEvent;
	ps->eventParms[ps->eventSequence & (MAX_PS_EVENTS-1)] = eventParm;
	ps->eventSequence++;
}

#undef playerState_t

/*
 * Drive one BG_AddPredictableEventToPlayerstate call: load (eventSequence,
 * events[2], eventParms[2]) in, run the C body, read the mutated state back out.
 */
void jka_bg_add_pred_event(int newEvent, int eventParm,
                           int *io_eventSequence, int *io_events, int *io_eventParms) {
	ps_ev_min_t ps;
	ps.eventSequence = *io_eventSequence;
	ps.events[0] = io_events[0];
	ps.events[1] = io_events[1];
	ps.eventParms[0] = io_eventParms[0];
	ps.eventParms[1] = io_eventParms[1];

	BG_AddPredictableEventToPlayerstate(newEvent, eventParm, &ps);

	*io_eventSequence = ps.eventSequence;
	io_events[0] = ps.events[0];
	io_events[1] = ps.events[1];
	io_eventParms[0] = ps.eventParms[0];
	io_eventParms[1] = ps.eventParms[1];
}

/*
 * BG_HasYsalamiri (bg_misc.c:1794) + BG_CanUseFPNow (bg_misc.c:1810): the two
 * force-power gate predicates. Same minimal-struct discipline — they read only a few
 * playerState_t scalars plus the powerups[] slots they test, so the bodies are copied
 * VERBATIM over a minimal ps holding just those fields. The relevant constants are
 * transcribed from bg_public.h / q_shared.h. Driven by int-marshalling wrappers (no
 * full struct crosses FFI).
 */

#define GT_CTY 9            /* bg_public.h gametype_t */
#define PW_REDFLAG 4        /* bg_public.h powerup_t */
#define PW_BLUEFLAG 5
#define PW_YSALAMIRI 15
#define WP_EMPLACED_GUN 17  /* bg_weapons.h weapon_t */
#define FP_LEVITATION 1     /* q_shared.h forcePowers_t */
#define FP_PUSH 3
#define FP_PULL 4
#define FP_GRIP 6
#define FP_LIGHTNING 7
#define FP_DRAIN 13
#define FP_SABER_OFFENSE 15
#define FP_SABER_DEFENSE 16
#define BROKENLIMB_LARM 1   /* bg_public.h brokenLimb_t */
#define BROKENLIMB_RARM 2

typedef int forcePowers_t;

typedef struct {
	int powerups[16];
	int forceRestricted;
	int trueNonJedi;
	int weapon;
	int m_iVehicleNum;
	int duelInProgress;
	int saberLockFrame;
	int saberLockTime;
	int fallingToDeath;
	int brokenLimbs;
} ps_fp_min_t;

#define playerState_t ps_fp_min_t
#define qboolean int
#define qtrue 1
#define qfalse 0

/* verbatim body of bg_misc.c:1794 */
qboolean BG_HasYsalamiri(int gametype, playerState_t *ps)
{
	if (gametype == GT_CTY &&
		(ps->powerups[PW_REDFLAG] || ps->powerups[PW_BLUEFLAG]))
	{
		return qtrue;
	}

	if (ps->powerups[PW_YSALAMIRI])
	{
		return qtrue;
	}

	return qfalse;
}

/* verbatim body of bg_misc.c:1810 */
qboolean BG_CanUseFPNow(int gametype, playerState_t *ps, int time, forcePowers_t power)
{
	if (BG_HasYsalamiri(gametype, ps))
	{
		return qfalse;
	}

	if ( ps->forceRestricted || ps->trueNonJedi )
	{
		return qfalse;
	}

	if (ps->weapon == WP_EMPLACED_GUN)
	{ //can't use any of your powers while on an emplaced weapon
		return qfalse;
	}

	if (ps->m_iVehicleNum)
	{ //can't use powers while riding a vehicle (this may change, I don't know)
		return qfalse;
	}

	if (ps->duelInProgress)
	{
		if (power != FP_SABER_OFFENSE && power != FP_SABER_DEFENSE && /*power != FP_SABERTHROW &&*/
			power != FP_LEVITATION)
		{
			if (!ps->saberLockFrame || power != FP_PUSH)
			{
				return qfalse;
			}
		}
	}

	if (ps->saberLockFrame || ps->saberLockTime > time)
	{
		if (power != FP_PUSH)
		{
			return qfalse;
		}
	}

	if (ps->fallingToDeath)
	{
		return qfalse;
	}

	if ((ps->brokenLimbs & (1 << BROKENLIMB_RARM)) ||
		(ps->brokenLimbs & (1 << BROKENLIMB_LARM)))
	{ //powers we can't use with a broken arm
        switch (power)
		{
		case FP_PUSH:
		case FP_PULL:
		case FP_GRIP:
		case FP_LIGHTNING:
		case FP_DRAIN:
			return qfalse;
		default:
			break;
		}
	}

	return qtrue;
}

#undef playerState_t
#undef qboolean
#undef qtrue
#undef qfalse

/* Load the three relevant powerups[] slots, run BG_HasYsalamiri, return the result. */
int jka_bg_has_ysalamiri(int gametype, int pw_red, int pw_blue, int pw_ysa) {
	ps_fp_min_t ps;
	int i;
	for (i = 0; i < 16; i++) ps.powerups[i] = 0;
	ps.powerups[PW_REDFLAG] = pw_red;
	ps.powerups[PW_BLUEFLAG] = pw_blue;
	ps.powerups[PW_YSALAMIRI] = pw_ysa;
	return BG_HasYsalamiri(gametype, &ps);
}

/* Load every field BG_CanUseFPNow reads, run it, return the result. */
int jka_bg_can_use_fp_now(int gametype, int time, int power,
                          int pw_red, int pw_blue, int pw_ysa,
                          int forceRestricted, int trueNonJedi, int weapon,
                          int m_iVehicleNum, int duelInProgress, int saberLockFrame,
                          int saberLockTime, int fallingToDeath, int brokenLimbs) {
	ps_fp_min_t ps;
	int i;
	for (i = 0; i < 16; i++) ps.powerups[i] = 0;
	ps.powerups[PW_REDFLAG] = pw_red;
	ps.powerups[PW_BLUEFLAG] = pw_blue;
	ps.powerups[PW_YSALAMIRI] = pw_ysa;
	ps.forceRestricted = forceRestricted;
	ps.trueNonJedi = trueNonJedi;
	ps.weapon = weapon;
	ps.m_iVehicleNum = m_iVehicleNum;
	ps.duelInProgress = duelInProgress;
	ps.saberLockFrame = saberLockFrame;
	ps.saberLockTime = saberLockTime;
	ps.fallingToDeath = fallingToDeath;
	ps.brokenLimbs = brokenLimbs;
	return BG_CanUseFPNow(gametype, &ps, time, power);
}

/*
 * vectoyaw (bg_misc.c:1773): direction → yaw angle. Pure float math (atan2/M_PI), no
 * struct, so the body is transcribed VERBATIM with the YAW/PITCH indices and vec_t/
 * vec3_t typedefs from q_shared.h. Driven by a 3-float-marshalling wrapper. atan2 needs
 * libm — already linked by the q_math oracle TU.
 */
#include <math.h>

#define YAW 1   /* q_shared.h */
#define PITCH 0
/* q_shared.h M_PI carries an `f` suffix — it is a *float* constant, not math.h's
 * double. Override math.h's definition so the division matches real JKA / the f32
 * M_PI on the Rust side (else the result is off by one ULP). */
#undef M_PI
#define M_PI 3.14159265358979323846f

typedef float vy_vec_t;
typedef vy_vec_t vy_vec3_t[3];

float vectoyaw( const vy_vec3_t vec ) {
	float	yaw;

	if (vec[YAW] == 0 && vec[PITCH] == 0) {
		yaw = 0;
	} else {
		if (vec[PITCH]) {
			yaw = ( atan2( vec[YAW], vec[PITCH]) * 180 / M_PI );
		} else if (vec[YAW] > 0) {
			yaw = 90;
		} else {
			yaw = 270;
		}
		if (yaw < 0) {
			yaw += 360;
		}
	}

	return yaw;
}

/* Load a vec3, run vectoyaw, return the result. */
float jka_bg_vectoyaw(float x, float y, float z) {
	vy_vec3_t v;
	v[0] = x; v[1] = y; v[2] = z;
	return vectoyaw(v);
}

/*
 * BG_EvaluateTrajectory / BG_EvaluateTrajectoryDelta (bg_misc.c:2355, 2421):
 * parametric trajectory sampling — a value (position/angles) and its derivative
 * (velocity). trajectory_t is pointer-free, but per the minimal-struct discipline its
 * fields cross FFI as positional scalars and the struct is rebuilt C-side. The switch
 * bodies are transcribed VERBATIM. Reuses this TU's already-`#undef`'d float M_PI and
 * its <math.h> (libm sin/cos). The default case in the real C calls Com_Error(ERR_DROP)
 * — replaced with a no-op here (never exercised; the tests drive only valid trTypes).
 * Note the carried original-JKA quirk faithfully kept in the Delta body's message.
 */

#define DEFAULT_GRAVITY 800          /* bg_public.h */
#define DEG2RAD( a ) (((a) * M_PI) / 180.0f)   /* q_shared.h (M_PI is float here) */

/* trType_t (q_shared.h) */
#define TR_STATIONARY     0
#define TR_INTERPOLATE    1
#define TR_LINEAR         2
#define TR_LINEAR_STOP    3
#define TR_NONLINEAR_STOP 4
#define TR_SINE           5
#define TR_GRAVITY        6

/* vector macros (q_shared.h) */
#define VectorCopy(a,b)      ((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define VectorClear(a)       ((a)[0]=(a)[1]=(a)[2]=0)
#define VectorScale(v,s,o)   ((o)[0]=(v)[0]*(s),(o)[1]=(v)[1]*(s),(o)[2]=(v)[2]*(s))
#define VectorMA(v,s,b,o)    ((o)[0]=(v)[0]+(b)[0]*(s),(o)[1]=(v)[1]+(b)[1]*(s),(o)[2]=(v)[2]+(b)[2]*(s))

typedef float tr_vec3_t[3];
typedef struct {
	int   trType;
	int   trTime;
	int   trDuration;
	float trBase[3];
	float trDelta[3];
} tr_min_t;

#define trajectory_t tr_min_t
#define vec3_t tr_vec3_t

/* verbatim body of bg_misc.c:2355 (default Com_Error replaced with a no-op) */
void BG_EvaluateTrajectory( const trajectory_t *tr, int atTime, vec3_t result ) {
	float		deltaTime;
	float		phase;

	switch( tr->trType ) {
	case TR_STATIONARY:
	case TR_INTERPOLATE:
		VectorCopy( tr->trBase, result );
		break;
	case TR_LINEAR:
		deltaTime = ( atTime - tr->trTime ) * 0.001;	// milliseconds to seconds
		VectorMA( tr->trBase, deltaTime, tr->trDelta, result );
		break;
	case TR_SINE:
		deltaTime = ( atTime - tr->trTime ) / (float) tr->trDuration;
		phase = sin( deltaTime * M_PI * 2 );
		VectorMA( tr->trBase, phase, tr->trDelta, result );
		break;
	case TR_LINEAR_STOP:
		if ( atTime > tr->trTime + tr->trDuration ) {
			atTime = tr->trTime + tr->trDuration;
		}
		deltaTime = ( atTime - tr->trTime ) * 0.001;	// milliseconds to seconds
		if ( deltaTime < 0 ) {
			deltaTime = 0;
		}
		VectorMA( tr->trBase, deltaTime, tr->trDelta, result );
		break;
	case TR_NONLINEAR_STOP:
		if ( atTime > tr->trTime + tr->trDuration )
		{
			atTime = tr->trTime + tr->trDuration;
		}
		//new slow-down at end
		if ( atTime - tr->trTime > tr->trDuration || atTime - tr->trTime <= 0  )
		{
			deltaTime = 0;
		}
		else
		{//FIXME: maybe scale this somehow?  So that it starts out faster and stops faster?
			deltaTime = tr->trDuration*0.001f*((float)cos( DEG2RAD(90.0f - (90.0f*((float)atTime-tr->trTime)/(float)tr->trDuration)) ));
		}
		VectorMA( tr->trBase, deltaTime, tr->trDelta, result );
		break;
	case TR_GRAVITY:
		deltaTime = ( atTime - tr->trTime ) * 0.001;	// milliseconds to seconds
		VectorMA( tr->trBase, deltaTime, tr->trDelta, result );
		result[2] -= 0.5 * DEFAULT_GRAVITY * deltaTime * deltaTime;		// FIXME: local gravity...
		break;
	default:
		/* real C: Com_Error( ERR_DROP, "BG_EvaluateTrajectory: ... unknown trType" ) */
		break;
	}
}

/* verbatim body of bg_misc.c:2421 (default Com_Error replaced with a no-op; note the
 * original quirk that the message prints tr->trTime, not tr->trType) */
void BG_EvaluateTrajectoryDelta( const trajectory_t *tr, int atTime, vec3_t result ) {
	float	deltaTime;
	float	phase;

	switch( tr->trType ) {
	case TR_STATIONARY:
	case TR_INTERPOLATE:
		VectorClear( result );
		break;
	case TR_LINEAR:
		VectorCopy( tr->trDelta, result );
		break;
	case TR_SINE:
		deltaTime = ( atTime - tr->trTime ) / (float) tr->trDuration;
		phase = cos( deltaTime * M_PI * 2 );	// derivative of sin = cos
		phase *= 0.5;
		VectorScale( tr->trDelta, phase, result );
		break;
	case TR_LINEAR_STOP:
		if ( atTime > tr->trTime + tr->trDuration ) {
			VectorClear( result );
			return;
		}
		VectorCopy( tr->trDelta, result );
		break;
	case TR_NONLINEAR_STOP:
		if ( atTime - tr->trTime > tr->trDuration || atTime - tr->trTime <= 0  )
		{
			VectorClear( result );
			return;
		}
		deltaTime = tr->trDuration*0.001f*((float)cos( DEG2RAD(90.0f - (90.0f*((float)atTime-tr->trTime)/(float)tr->trDuration)) ));
		VectorScale( tr->trDelta, deltaTime, result );
		break;
	case TR_GRAVITY:
		deltaTime = ( atTime - tr->trTime ) * 0.001;	// milliseconds to seconds
		VectorCopy( tr->trDelta, result );
		result[2] -= DEFAULT_GRAVITY * deltaTime;		// FIXME: local gravity...
		break;
	default:
		/* real C: Com_Error( ERR_DROP, "...unknown trType: %i", tr->trTime ) */
		break;
	}
}

/* Marshal trajectory fields + atTime in, run the position sampler, read result out. */
void jka_bg_eval_traj(int trType, int trTime, int trDuration,
                      float bx, float by, float bz,
                      float dx, float dy, float dz,
                      int atTime, float *out) {
	tr_min_t tr;
	tr_vec3_t result;
	tr.trType = trType; tr.trTime = trTime; tr.trDuration = trDuration;
	tr.trBase[0] = bx; tr.trBase[1] = by; tr.trBase[2] = bz;
	tr.trDelta[0] = dx; tr.trDelta[1] = dy; tr.trDelta[2] = dz;
	BG_EvaluateTrajectory(&tr, atTime, result);
	out[0] = result[0]; out[1] = result[1]; out[2] = result[2];
}

/* Same marshalling for the velocity (derivative) sampler. */
void jka_bg_eval_traj_delta(int trType, int trTime, int trDuration,
                            float bx, float by, float bz,
                            float dx, float dy, float dz,
                            int atTime, float *out) {
	tr_min_t tr;
	tr_vec3_t result;
	tr.trType = trType; tr.trTime = trTime; tr.trDuration = trDuration;
	tr.trBase[0] = bx; tr.trBase[1] = by; tr.trBase[2] = bz;
	tr.trDelta[0] = dx; tr.trDelta[1] = dy; tr.trDelta[2] = dz;
	BG_EvaluateTrajectoryDelta(&tr, atTime, result);
	out[0] = result[0]; out[1] = result[1]; out[2] = result[2];
}

/* Verbatim body of bg_misc.c:1979. Marshals the player origin + the item's pos
 * trajectory + atTime in (never crossing the full playerState_t/entityState_t), runs
 * the real BG_EvaluateTrajectory, and returns qboolean (1/0). */
int jka_bg_player_touches_item(float px, float py, float pz,
                               int trType, int trTime, int trDuration,
                               float bx, float by, float bz,
                               float dx, float dy, float dz,
                               int atTime) {
	tr_min_t pos;
	tr_vec3_t ps_origin;
	tr_vec3_t origin;
	pos.trType = trType; pos.trTime = trTime; pos.trDuration = trDuration;
	pos.trBase[0] = bx; pos.trBase[1] = by; pos.trBase[2] = bz;
	pos.trDelta[0] = dx; pos.trDelta[1] = dy; pos.trDelta[2] = dz;
	ps_origin[0] = px; ps_origin[1] = py; ps_origin[2] = pz;

	BG_EvaluateTrajectory( &pos, atTime, origin );

	/* we are ignoring ducked differences here */
	if ( ps_origin[0] - origin[0] > 44
		|| ps_origin[0] - origin[0] < -50
		|| ps_origin[1] - origin[1] > 36
		|| ps_origin[1] - origin[1] < -36
		|| ps_origin[2] - origin[2] > 36
		|| ps_origin[2] - origin[2] < -36 ) {
		return 0;	/* qfalse */
	}

	return 1;	/* qtrue */
}

/*
 * ---- Force-power cycle oracle (bg_misc.c:197 table, :1997 / :2014 fns) ----
 *
 * forcePowerSorted[] + BG_ProperForceIndex + BG_CycleForce, transcribed VERBATIM.
 * The FP_* values and NUM_FORCE_POWERS match q_shared.h `forcePowers_t` (verified in
 * q_shared_h_oracle.c). BG_CycleForce touches only ps->fd.{forcePowersKnown,
 * forcePowerSelected}, so it runs over a minimal playerState_t holding just `fd` with
 * those two ints (never crossing the full struct).
 */
#define FP_HEAL 0
#define FP_LEVITATION 1
#define FP_SPEED 2
#define FP_PUSH 3
#define FP_PULL 4
#define FP_TELEPATHY 5
#define FP_GRIP 6
#define FP_LIGHTNING 7
#define FP_RAGE 8
#define FP_PROTECT 9
#define FP_ABSORB 10
#define FP_TEAM_HEAL 11
#define FP_TEAM_FORCE 12
#define FP_DRAIN 13
#define FP_SEE 14
#define FP_SABER_OFFENSE 15
#define FP_SABER_DEFENSE 16
#define FP_SABERTHROW 17
#define NUM_FORCE_POWERS 18

int forcePowerSorted[NUM_FORCE_POWERS] =
{ //rww - always use this order when drawing force powers for any reason
	FP_TELEPATHY,
	FP_HEAL,
	FP_ABSORB,
	FP_PROTECT,
	FP_TEAM_HEAL,
	FP_LEVITATION,
	FP_SPEED,
	FP_PUSH,
	FP_PULL,
	FP_SEE,
	FP_LIGHTNING,
	FP_DRAIN,
	FP_RAGE,
	FP_GRIP,
	FP_TEAM_FORCE,
	FP_SABER_OFFENSE,
	FP_SABER_DEFENSE,
	FP_SABERTHROW
};

/* verbatim bg_misc.c:1997 */
int BG_ProperForceIndex(int power)
{
	int i = 0;

	while (i < NUM_FORCE_POWERS)
	{
		if (forcePowerSorted[i] == power)
		{
			return i;
		}

		i++;
	}

	return -1;
}

typedef struct { int forcePowersKnown; int forcePowerSelected; } fd_force_min_t;
typedef struct { fd_force_min_t fd; } ps_force_min_t;
#define playerState_t ps_force_min_t

/* verbatim bg_misc.c:2014 */
void BG_CycleForce(playerState_t *ps, int direction)
{
	int i = ps->fd.forcePowerSelected;
	int x = i;
	int presel = i;
	int foundnext = -1;

	if (!ps->fd.forcePowersKnown & (1 << x) ||
		x >= NUM_FORCE_POWERS ||
		x == -1)
	{ //apparently we have no valid force powers
		return;
	}

	x = BG_ProperForceIndex(x);
	presel = x;

	if (direction == 1)
	{ //get the next power
		x++;
	}
	else
	{ //get the previous power
		x--;
	}

	if (x >= NUM_FORCE_POWERS)
	{ //cycled off the end.. cycle around to the first
		x = 0;
	}
	if (x < 0)
	{ //cycled off the beginning.. cycle around to the last
		x = NUM_FORCE_POWERS-1;
	}

	i = forcePowerSorted[x]; //the "sorted" value of this power

	while (x != presel)
	{ //loop around to the current force power
		if (ps->fd.forcePowersKnown & (1 << i) && i != ps->fd.forcePowerSelected)
		{ //we have the force power
			if (i != FP_LEVITATION &&
				i != FP_SABER_OFFENSE &&
				i != FP_SABER_DEFENSE &&
				i != FP_SABERTHROW)
			{ //it's selectable
				foundnext = i;
				break;
			}
		}

		if (direction == 1)
		{ //next
			x++;
		}
		else
		{ //previous
			x--;
		}

		if (x >= NUM_FORCE_POWERS)
		{ //loop around
			x = 0;
		}
		if (x < 0)
		{ //loop around
			x = NUM_FORCE_POWERS-1;
		}

		i = forcePowerSorted[x]; //set to the sorted value again
	}

	if (foundnext != -1)
	{ //found one, select it
		ps->fd.forcePowerSelected = foundnext;
	}
}
#undef playerState_t

/* Drive BG_ProperForceIndex. */
int jka_bg_proper_force_index(int power) {
	return BG_ProperForceIndex(power);
}

/* Drive BG_CycleForce: load (forcePowersKnown, forcePowerSelected, direction), run the
 * C body, return the resulting forcePowerSelected. */
int jka_bg_cycle_force(int forcePowersKnown, int forcePowerSelected, int direction) {
	ps_force_min_t ps;
	ps.fd.forcePowersKnown = forcePowersKnown;
	ps.fd.forcePowerSelected = forcePowerSelected;
	BG_CycleForce(&ps, direction);
	return ps.fd.forcePowerSelected;
}

/*
 * BG_EmplacedView (bg_misc.c:2712): emplaced-gun yaw constriction. The function reads
 * only baseAngles[YAW] / angles[YAW] and writes *newYaw, so the wrapper passes just the
 * two yaw scalars (the other vec3 components are never touched). Body transcribed
 * VERBATIM; AngleSubtract is reused from q_math_oracle.c (the bg_pmove_oracle.c
 * precedent), and the file-live `vec3_t`->`tr_vec3_t` (float[3]) macro + `YAW` index are
 * reused as-is (no re-typedef).
 */
extern float AngleSubtract( float a1, float a2 );

/* verbatim bg_misc.c:2712 */
int BG_EmplacedView(vec3_t baseAngles, vec3_t angles, float *newYaw, float constraint)
{
	float dif = AngleSubtract(baseAngles[YAW], angles[YAW]);

	if (dif > constraint ||
		dif < -constraint)
	{
		float amt;

		if (dif > constraint)
		{
			amt = (dif-constraint);
			dif = constraint;
		}
		else if (dif < -constraint)
		{
			amt = (dif+constraint);
			dif = -constraint;
		}
		else
		{
			amt = 0.0f;
		}

		*newYaw = AngleSubtract(angles[YAW], -dif);

		if (amt > 1.0f || amt < -1.0f)
		{ //significant, force the view
			return 2;
		}
		else
		{ //just a little out of range
			return 1;
		}
	}

	return 0;
}

/*
 * Drive BG_EmplacedView: pass the two yaw scalars (the other vec3 components are unread),
 * return the int code; *out_newYaw receives the written yaw (left untouched by the C body
 * when the function returns 0).
 */
int jka_bg_emplaced_view(float baseYaw, float angleYaw, float constraint, float *out_newYaw) {
	vec3_t baseAngles = { 0.0f, baseYaw, 0.0f };
	vec3_t angles = { 0.0f, angleYaw, 0.0f };
	return BG_EmplacedView(baseAngles, angles, out_newYaw, constraint);
}

/*
 * BG_TouchJumpPad (bg_misc.c:2676): jump-pad velocity transfer + pad bookkeeping. Touches
 * only a few ps/es fields, so per minimal-struct discipline the wrapper marshals those
 * scalars in/out (never crossing the full playerState_t/entityState_t). Body transcribed
 * VERBATIM, including the vestigial `effectNum` the original computes but never reads (a
 * `(void)` cast silences -Wunused-but-set without altering behaviour). Reuses this TU's
 * float M_PI / <math.h> (fabs) and the file-live `vec3_t`->float[3] + YAW/PITCH + VectorCopy
 * macros; vectoangles/AngleNormalize180 are reused from q_math_oracle.c (the AngleSubtract
 * precedent above).
 */
extern void vectoangles( const vec3_t value1, vec3_t angles );
extern float AngleNormalize180( float angle );

#define PM_NORMAL  0   /* pmtype_t (q_shared.h) */
#define PM_JETPACK 1
#define PM_FLOAT   2

typedef struct { int pm_type; int jumppad_ent; int jumppad_frame; int pmove_framecount; vec3_t velocity; } jp_ps_t;
typedef struct { int number; vec3_t origin2; } jp_es_t;

#define playerState_t jp_ps_t
#define entityState_t jp_es_t

void BG_TouchJumpPad( playerState_t *ps, entityState_t *jumppad ) {
	vec3_t	angles;
	float p;
	int effectNum;

	// spectators don't use jump pads
	if ( ps->pm_type != PM_NORMAL && ps->pm_type != PM_JETPACK && ps->pm_type != PM_FLOAT ) {
		return;
	}

	// if we didn't hit this same jumppad the previous frame
	// then don't play the event sound again if we are in a fat trigger
	if ( ps->jumppad_ent != jumppad->number ) {

		vectoangles( jumppad->origin2, angles);
		p = fabs( AngleNormalize180( angles[PITCH] ) );
		if( p < 45 ) {
			effectNum = 0;
		} else {
			effectNum = 1;
		}
		(void)effectNum; /* vestigial in the original — silence -Wunused-but-set */
	}
	// remember hitting this jumppad this frame
	ps->jumppad_ent = jumppad->number;
	ps->jumppad_frame = ps->pmove_framecount;
	// give the player the velocity from the jumppad
	VectorCopy( jumppad->origin2, ps->velocity );
}

#undef playerState_t
#undef entityState_t

/* Marshal the read ps/es fields in (jumppad_frame seeded to 0 to match a fresh Rust ps),
 * run the verbatim body, write the three mutated outputs back out. */
void jka_bg_touch_jump_pad(int pm_type, int in_jumppad_ent, int number,
                           int pmove_framecount,
                           float o2x, float o2y, float o2z,
                           int *out_jumppad_ent, int *out_jumppad_frame,
                           float *out_velocity) {
	jp_ps_t ps;
	jp_es_t es;
	ps.pm_type = pm_type;
	ps.jumppad_ent = in_jumppad_ent;
	ps.jumppad_frame = 0;
	ps.pmove_framecount = pmove_framecount;
	ps.velocity[0] = ps.velocity[1] = ps.velocity[2] = 0;
	es.number = number;
	es.origin2[0] = o2x; es.origin2[1] = o2y; es.origin2[2] = o2z;
	BG_TouchJumpPad( &ps, &es );
	*out_jumppad_ent = ps.jumppad_ent;
	*out_jumppad_frame = ps.jumppad_frame;
	out_velocity[0] = ps.velocity[0];
	out_velocity[1] = ps.velocity[1];
	out_velocity[2] = ps.velocity[2];
}

/*
 * BG_IsValidCharacterModel (bg_misc.c:2753): rejects the SP-only "kyle" fpls* skins.
 * Pure string comparison — the wrapper IS the verbatim body (the `char *` args cross FFI
 * directly). Q_stricmp is reused from q_shared_oracle.c. Returns qboolean (1/0).
 */
extern int Q_stricmp( const char *s1, const char *s2 );

int jka_bg_is_valid_character_model(const char *modelName, const char *skinName) {
	if (!Q_stricmp(skinName, "menu")) {
		return 0;
	} else if (!Q_stricmp(modelName, "kyle")) {
		if (!Q_stricmp(skinName, "fpls")) {
			return 0;
		} else if (!Q_stricmp(skinName, "fpls2")) {
			return 0;
		} else if (!Q_stricmp(skinName, "fpls3")) {
			return 0;
		}
	}
	return 1;
}

/*
 * BG_PlayerStateToEntityState (bg_misc.c:2901): collapse a playerState_t into the
 * networked entityState_t. ~60 fields touched, so per minimal-struct discipline the body
 * runs over minimal ps/es structs holding only those fields (never the full 1464/528-byte
 * layouts — those are verified in q_shared_h_oracle.c). Body transcribed VERBATIM. The
 * inline `SnapVector` it calls is the q_shared.h *macro/inline* form (NOT trap_SnapVector):
 * the shipped retail Win32 build uses `__asm fistp`, which rounds to nearest with ties to
 * even under the default x87 rounding mode — reproduced here with rintf (same rounding
 * mode), matching the Rust `f32::round_ties_even`. Reuses this TU's already-live VectorCopy
 * macro + TR_INTERPOLATE / YAW / MAX_PS_EVENTS defines and <math.h>. Inputs/outputs cross
 * FFI as documented int/float arrays (see jka_bg_player_state_to_entity_state).
 */
#include <string.h>               /* memset (zero the marshalling structs) */
#define STAT_HEALTH    0          /* statIndex_t (bg_public.h) */
#define GIB_HEALTH     (-40)      /* bg_public.h */
#define ET_INVISIBLE   12         /* entityType_t (bg_public.h) */
#define ET_PLAYER      1
#define PM_INTERMISSION 7         /* pmtype_t (bg_public.h) */
#define PM_SPECTATOR   4
#define EF_SEEKERDRONE (1<<21)    /* bg_public.h */
#define EF_DEAD        (1<<1)
#define MAX_POWERUPS   16         /* q_shared.h */
/* inline SnapVector (q_shared.h) — retail fistp round-to-nearest-even, via rintf */
#define SnapVector(v) {(v)[0]=rintf((v)[0]);(v)[1]=rintf((v)[1]);(v)[2]=rintf((v)[2]);}

typedef struct {
	int   forceMindtrickTargetIndex;
	int   forceMindtrickTargetIndex2;
	int   forceMindtrickTargetIndex3;
	int   forceMindtrickTargetIndex4;
	int   forcePowersActive;
	int   saberAnimLevel;
} pse_fd_t;

typedef struct {
	int   pm_type;
	int   stats[1];               /* only STAT_HEALTH read */
	int   clientNum;
	float origin[3];
	float velocity[3];
	float viewangles[3];
	pse_fd_t fd;
	int   saberLockFrame;
	int   electrifyTime;
	float speed;
	int   genericEnemyIndex;
	int   activeForcePass;
	int   movementDir;
	int   legsAnim;
	int   torsoAnim;
	int   legsFlip;
	int   torsoFlip;
	int   eFlags;
	int   eFlags2;
	int   saberInFlight;
	int   saberEntityNum;
	int   saberMove;
	int   duelInProgress;
	int   emplacedIndex;
	int   saberHolstered;
	int   externalEvent;
	int   externalEventParm;
	int   entityEventSequence;    /* mutated by the event-latch branch */
	int   eventSequence;
	int   events[2];
	int   eventParms[2];
	int   weapon;
	int   groundEntityNum;
	int   powerups[16];
	int   loopSound;
	int   generic1;
	int   weaponstate;
	int   weaponChargeTime;
	float lastHitLoc[3];
	int   heldByClient;
	int   ragAttach;
	int   iModelScale;
	int   brokenLimbs;
	int   hasLookTarget;
	int   lookTarget;
	int   customRGBA[4];
	int   m_iVehicleNum;
	int   holocronBits;
	int   isJediMaster;
} pse_ps_t;

/* trTime/trDuration are written only by the ExtraPolate variant below; the plain
 * BG_PlayerStateToEntityState leaves them at their memset-0, harmless to its wrapper. */
typedef struct { int trType; float trBase[3]; float trDelta[3]; int trTime; int trDuration; } pse_traj_t;

typedef struct {
	int   eType;
	int   number;
	pse_traj_t pos;
	pse_traj_t apos;
	int   trickedentindex;
	int   trickedentindex2;
	int   trickedentindex3;
	int   trickedentindex4;
	int   forceFrame;
	int   emplacedOwner;
	float speed;
	int   genericenemyindex;
	int   activeForcePass;
	float angles2[3];
	int   legsAnim;
	int   torsoAnim;
	int   legsFlip;
	int   torsoFlip;
	int   clientNum;
	int   eFlags;
	int   eFlags2;
	int   saberInFlight;
	int   saberEntityNum;
	int   saberMove;
	int   forcePowersActive;
	int   bolt1;
	int   otherEntityNum2;
	int   saberHolstered;
	int   event;
	int   eventParm;
	int   weapon;
	int   groundEntityNum;
	int   powerups;
	int   loopSound;
	int   generic1;
	int   modelindex2;
	int   constantLight;
	float origin2[3];
	int   time2;
	int   isJediMaster;
	int   fireflag;
	int   heldByClient;
	int   ragAttach;
	int   iModelScale;
	int   brokenLimbs;
	int   hasLookTarget;
	int   lookTarget;
	int   customRGBA[4];
	int   m_iVehicleNum;
} pse_es_t;

#define playerState_t pse_ps_t
#define entityState_t pse_es_t

/* verbatim body of bg_misc.c:2901 */
void BG_PlayerStateToEntityState( playerState_t *ps, entityState_t *s, int snap ) {
	int		i;

	if ( ps->pm_type == PM_INTERMISSION || ps->pm_type == PM_SPECTATOR ) {
		s->eType = ET_INVISIBLE;
	} else if ( ps->stats[STAT_HEALTH] <= GIB_HEALTH ) {
		s->eType = ET_INVISIBLE;
	} else {
		s->eType = ET_PLAYER;
	}

	s->number = ps->clientNum;

	s->pos.trType = TR_INTERPOLATE;
	VectorCopy( ps->origin, s->pos.trBase );
	if ( snap ) {
		SnapVector( s->pos.trBase );
	}
	// set the trDelta for flag direction
	VectorCopy( ps->velocity, s->pos.trDelta );

	s->apos.trType = TR_INTERPOLATE;
	VectorCopy( ps->viewangles, s->apos.trBase );
	if ( snap ) {
		SnapVector( s->apos.trBase );
	}

	s->trickedentindex = ps->fd.forceMindtrickTargetIndex;
	s->trickedentindex2 = ps->fd.forceMindtrickTargetIndex2;
	s->trickedentindex3 = ps->fd.forceMindtrickTargetIndex3;
	s->trickedentindex4 = ps->fd.forceMindtrickTargetIndex4;

	s->forceFrame = ps->saberLockFrame;

	s->emplacedOwner = ps->electrifyTime;

	s->speed = ps->speed;

	s->genericenemyindex = ps->genericEnemyIndex;

	s->activeForcePass = ps->activeForcePass;

	s->angles2[YAW] = ps->movementDir;
	s->legsAnim = ps->legsAnim;
	s->torsoAnim = ps->torsoAnim;

	s->legsFlip = ps->legsFlip;
	s->torsoFlip = ps->torsoFlip;

	s->clientNum = ps->clientNum;		// ET_PLAYER looks here instead of at number
										// so corpses can also reference the proper config
	s->eFlags = ps->eFlags;
	s->eFlags2 = ps->eFlags2;

	s->saberInFlight = ps->saberInFlight;
	s->saberEntityNum = ps->saberEntityNum;
	s->saberMove = ps->saberMove;
	s->forcePowersActive = ps->fd.forcePowersActive;

	if (ps->duelInProgress)
	{
		s->bolt1 = 1;
	}
	else
	{
		s->bolt1 = 0;
	}

	s->otherEntityNum2 = ps->emplacedIndex;

	s->saberHolstered = ps->saberHolstered;

	if (ps->genericEnemyIndex != -1)
	{
		s->eFlags |= EF_SEEKERDRONE;
	}

	if ( ps->stats[STAT_HEALTH] <= 0 ) {
		s->eFlags |= EF_DEAD;
	} else {
		s->eFlags &= ~EF_DEAD;
	}

	if ( ps->externalEvent ) {
		s->event = ps->externalEvent;
		s->eventParm = ps->externalEventParm;
	} else if ( ps->entityEventSequence < ps->eventSequence ) {
		int		seq;

		if ( ps->entityEventSequence < ps->eventSequence - MAX_PS_EVENTS) {
			ps->entityEventSequence = ps->eventSequence - MAX_PS_EVENTS;
		}
		seq = ps->entityEventSequence & (MAX_PS_EVENTS-1);
		s->event = ps->events[ seq ] | ( ( ps->entityEventSequence & 3 ) << 8 );
		s->eventParm = ps->eventParms[ seq ];
		ps->entityEventSequence++;
	}


	s->weapon = ps->weapon;
	s->groundEntityNum = ps->groundEntityNum;

	s->powerups = 0;
	for ( i = 0 ; i < MAX_POWERUPS ; i++ ) {
		if ( ps->powerups[ i ] ) {
			s->powerups |= 1 << i;
		}
	}

	s->loopSound = ps->loopSound;
	s->generic1 = ps->generic1;

	//NOT INCLUDED IN ENTITYSTATETOPLAYERSTATE:
	s->modelindex2 = ps->weaponstate;
	s->constantLight = ps->weaponChargeTime;

	VectorCopy(ps->lastHitLoc, s->origin2);

	s->isJediMaster = ps->isJediMaster;

	s->time2 = ps->holocronBits;

	s->fireflag = ps->fd.saberAnimLevel;

	s->heldByClient = ps->heldByClient;
	s->ragAttach = ps->ragAttach;

	s->iModelScale = ps->iModelScale;

	s->brokenLimbs = ps->brokenLimbs;

	s->hasLookTarget = ps->hasLookTarget;
	s->lookTarget = ps->lookTarget;

	s->customRGBA[0] = ps->customRGBA[0];
	s->customRGBA[1] = ps->customRGBA[1];
	s->customRGBA[2] = ps->customRGBA[2];
	s->customRGBA[3] = ps->customRGBA[3];

	s->m_iVehicleNum = ps->m_iVehicleNum;
}

#undef playerState_t
#undef entityState_t

/*
 * Drive one BG_PlayerStateToEntityState call. Inputs/outputs cross FFI as flat arrays
 * (never the full structs). Index maps — KEEP IN SYNC WITH THE RUST TEST:
 *
 *  in_i[45] scalar ints:
 *    0 pm_type           1 health(stats[0])  2 clientNum         3 fmti1
 *    4 fmti2             5 fmti3             6 fmti4             7 saberLockFrame
 *    8 electrifyTime     9 genericEnemyIndex 10 activeForcePass  11 movementDir
 *   12 legsAnim         13 torsoAnim        14 legsFlip         15 torsoFlip
 *   16 eFlags           17 eFlags2          18 saberInFlight    19 saberEntityNum
 *   20 saberMove        21 forcePowersActive 22 duelInProgress  23 emplacedIndex
 *   24 saberHolstered   25 externalEvent    26 externalEventParm 27 entityEventSequence
 *   28 eventSequence    29 weapon           30 groundEntityNum  31 loopSound
 *   32 generic1         33 weaponstate      34 weaponChargeTime 35 saberAnimLevel
 *   36 heldByClient     37 ragAttach        38 iModelScale      39 brokenLimbs
 *   40 hasLookTarget    41 lookTarget       42 m_iVehicleNum     43 isJediMaster
 *   44 holocronBits
 *  in_f[12]: origin[0..2], velocity[3..5], viewangles[6..8], lastHitLoc[9..11]
 *  in_speed: ps.speed (float). events[2], eventParms[2], powerups[16], customRGBA[4].
 *
 *  out_i[45] scalar ints:
 *    0 eType            1 number            2 pos.trType        3 apos.trType
 *    4 trickedentindex  5 ti2               6 ti3               7 ti4
 *    8 forceFrame       9 emplacedOwner    10 genericenemyindex 11 activeForcePass
 *   12 legsAnim        13 torsoAnim        14 legsFlip         15 torsoFlip
 *   16 clientNum       17 eFlags          18 eFlags2          19 saberInFlight
 *   20 saberEntityNum  21 saberMove       22 forcePowersActive 23 bolt1
 *   24 otherEntityNum2 25 saberHolstered  26 event            27 eventParm
 *   28 weapon          29 groundEntityNum 30 powerups         31 loopSound
 *   32 generic1        33 modelindex2     34 constantLight    35 time2(=holocronBits)
 *   36 fireflag        37 heldByClient    38 ragAttach        39 iModelScale
 *   40 brokenLimbs     41 hasLookTarget   42 lookTarget       43 m_iVehicleNum
 *   44 isJediMaster
 *  out_f[16]: pos.trBase[0..2], pos.trDelta[3..5], apos.trBase[6..8],
 *             angles2[9..11], origin2[12..14], speed[15]
 *  out_customRGBA[4], *out_entityEventSequence (the mutated ps field).
 */
void jka_bg_player_state_to_entity_state(
		const int *in_i, const float *in_f, float in_speed,
		const int *in_events, const int *in_eventParms,
		const int *in_powerups, const int *in_customRGBA, int snap,
		int *out_i, float *out_f, int *out_customRGBA, int *out_entityEventSequence) {
	pse_ps_t ps;
	pse_es_t s;
	int k;

	/* zero both: the Rust side runs over a freshly zeroed playerState_t/entityState_t, and
	 * the body leaves some compared fields untouched (angles2[0]/[2], and event/eventParm
	 * when neither event branch fires) — match that so unwritten fields read 0 on both sides. */
	memset(&ps, 0, sizeof ps);
	memset(&s, 0, sizeof s);

	ps.pm_type            = in_i[0];
	ps.stats[0]           = in_i[1];
	ps.clientNum          = in_i[2];
	ps.fd.forceMindtrickTargetIndex  = in_i[3];
	ps.fd.forceMindtrickTargetIndex2 = in_i[4];
	ps.fd.forceMindtrickTargetIndex3 = in_i[5];
	ps.fd.forceMindtrickTargetIndex4 = in_i[6];
	ps.saberLockFrame     = in_i[7];
	ps.electrifyTime      = in_i[8];
	ps.genericEnemyIndex  = in_i[9];
	ps.activeForcePass    = in_i[10];
	ps.movementDir        = in_i[11];
	ps.legsAnim           = in_i[12];
	ps.torsoAnim          = in_i[13];
	ps.legsFlip           = in_i[14];
	ps.torsoFlip          = in_i[15];
	ps.eFlags             = in_i[16];
	ps.eFlags2            = in_i[17];
	ps.saberInFlight      = in_i[18];
	ps.saberEntityNum     = in_i[19];
	ps.saberMove          = in_i[20];
	ps.fd.forcePowersActive = in_i[21];
	ps.duelInProgress     = in_i[22];
	ps.emplacedIndex      = in_i[23];
	ps.saberHolstered     = in_i[24];
	ps.externalEvent      = in_i[25];
	ps.externalEventParm  = in_i[26];
	ps.entityEventSequence = in_i[27];
	ps.eventSequence      = in_i[28];
	ps.weapon             = in_i[29];
	ps.groundEntityNum    = in_i[30];
	ps.loopSound          = in_i[31];
	ps.generic1           = in_i[32];
	ps.weaponstate        = in_i[33];
	ps.weaponChargeTime   = in_i[34];
	ps.fd.saberAnimLevel  = in_i[35];
	ps.heldByClient       = in_i[36];
	ps.ragAttach          = in_i[37];
	ps.iModelScale        = in_i[38];
	ps.brokenLimbs        = in_i[39];
	ps.hasLookTarget      = in_i[40];
	ps.lookTarget         = in_i[41];
	ps.m_iVehicleNum      = in_i[42];
	ps.isJediMaster       = in_i[43];
	ps.holocronBits       = in_i[44];

	ps.origin[0] = in_f[0];  ps.origin[1] = in_f[1];  ps.origin[2] = in_f[2];
	ps.velocity[0] = in_f[3]; ps.velocity[1] = in_f[4]; ps.velocity[2] = in_f[5];
	ps.viewangles[0] = in_f[6]; ps.viewangles[1] = in_f[7]; ps.viewangles[2] = in_f[8];
	ps.lastHitLoc[0] = in_f[9]; ps.lastHitLoc[1] = in_f[10]; ps.lastHitLoc[2] = in_f[11];
	ps.speed = in_speed;
	ps.events[0] = in_events[0]; ps.events[1] = in_events[1];
	ps.eventParms[0] = in_eventParms[0]; ps.eventParms[1] = in_eventParms[1];
	for (k = 0; k < 16; k++) ps.powerups[k] = in_powerups[k];
	for (k = 0; k < 4; k++) ps.customRGBA[k] = in_customRGBA[k];

	BG_PlayerStateToEntityState( &ps, &s, snap );

	out_i[0]  = s.eType;
	out_i[1]  = s.number;
	out_i[2]  = s.pos.trType;
	out_i[3]  = s.apos.trType;
	out_i[4]  = s.trickedentindex;
	out_i[5]  = s.trickedentindex2;
	out_i[6]  = s.trickedentindex3;
	out_i[7]  = s.trickedentindex4;
	out_i[8]  = s.forceFrame;
	out_i[9]  = s.emplacedOwner;
	out_i[10] = s.genericenemyindex;
	out_i[11] = s.activeForcePass;
	out_i[12] = s.legsAnim;
	out_i[13] = s.torsoAnim;
	out_i[14] = s.legsFlip;
	out_i[15] = s.torsoFlip;
	out_i[16] = s.clientNum;
	out_i[17] = s.eFlags;
	out_i[18] = s.eFlags2;
	out_i[19] = s.saberInFlight;
	out_i[20] = s.saberEntityNum;
	out_i[21] = s.saberMove;
	out_i[22] = s.forcePowersActive;
	out_i[23] = s.bolt1;
	out_i[24] = s.otherEntityNum2;
	out_i[25] = s.saberHolstered;
	out_i[26] = s.event;
	out_i[27] = s.eventParm;
	out_i[28] = s.weapon;
	out_i[29] = s.groundEntityNum;
	out_i[30] = s.powerups;
	out_i[31] = s.loopSound;
	out_i[32] = s.generic1;
	out_i[33] = s.modelindex2;
	out_i[34] = s.constantLight;
	out_i[35] = s.time2;
	out_i[36] = s.fireflag;
	out_i[37] = s.heldByClient;
	out_i[38] = s.ragAttach;
	out_i[39] = s.iModelScale;
	out_i[40] = s.brokenLimbs;
	out_i[41] = s.hasLookTarget;
	out_i[42] = s.lookTarget;
	out_i[43] = s.m_iVehicleNum;
	out_i[44] = s.isJediMaster;

	out_f[0]  = s.pos.trBase[0];  out_f[1]  = s.pos.trBase[1];  out_f[2]  = s.pos.trBase[2];
	out_f[3]  = s.pos.trDelta[0]; out_f[4]  = s.pos.trDelta[1]; out_f[5]  = s.pos.trDelta[2];
	out_f[6]  = s.apos.trBase[0]; out_f[7]  = s.apos.trBase[1]; out_f[8]  = s.apos.trBase[2];
	out_f[9]  = s.angles2[0];     out_f[10] = s.angles2[1];     out_f[11] = s.angles2[2];
	out_f[12] = s.origin2[0];     out_f[13] = s.origin2[1];     out_f[14] = s.origin2[2];
	out_f[15] = s.speed;

	out_customRGBA[0] = s.customRGBA[0];
	out_customRGBA[1] = s.customRGBA[1];
	out_customRGBA[2] = s.customRGBA[2];
	out_customRGBA[3] = s.customRGBA[3];

	*out_entityEventSequence = ps.entityEventSequence;
}

/*
 * BG_PlayerStateToEntityStateExtraPolate (bg_misc.c:3052): the extrapolating sibling of
 * BG_PlayerStateToEntityState. Body transcribed VERBATIM; it differs from the sibling only
 * in the position trajectory (TR_LINEAR_STOP + trTime/trDuration). Reuses the same minimal
 * pse_ps_t/pse_es_t structs (pse_traj_t already carries trTime/trDuration), so the
 * playerState_t/entityState_t typedefs are re-established for this body. TR_LINEAR_STOP is
 * already defined at TU scope (trType section).
 */
#define playerState_t pse_ps_t
#define entityState_t pse_es_t

/* verbatim body of bg_misc.c:3052 */
void BG_PlayerStateToEntityStateExtraPolate( playerState_t *ps, entityState_t *s, int time, int snap ) {
	int		i;

	if ( ps->pm_type == PM_INTERMISSION || ps->pm_type == PM_SPECTATOR ) {
		s->eType = ET_INVISIBLE;
	} else if ( ps->stats[STAT_HEALTH] <= GIB_HEALTH ) {
		s->eType = ET_INVISIBLE;
	} else {
		s->eType = ET_PLAYER;
	}

	s->number = ps->clientNum;

	s->pos.trType = TR_LINEAR_STOP;
	VectorCopy( ps->origin, s->pos.trBase );
	if ( snap ) {
		SnapVector( s->pos.trBase );
	}
	// set the trDelta for flag direction and linear prediction
	VectorCopy( ps->velocity, s->pos.trDelta );
	// set the time for linear prediction
	s->pos.trTime = time;
	// set maximum extra polation time
	s->pos.trDuration = 50; // 1000 / sv_fps (default = 20)

	s->apos.trType = TR_INTERPOLATE;
	VectorCopy( ps->viewangles, s->apos.trBase );
	if ( snap ) {
		SnapVector( s->apos.trBase );
	}

	s->trickedentindex = ps->fd.forceMindtrickTargetIndex;
	s->trickedentindex2 = ps->fd.forceMindtrickTargetIndex2;
	s->trickedentindex3 = ps->fd.forceMindtrickTargetIndex3;
	s->trickedentindex4 = ps->fd.forceMindtrickTargetIndex4;

	s->forceFrame = ps->saberLockFrame;

	s->emplacedOwner = ps->electrifyTime;

	s->speed = ps->speed;

	s->genericenemyindex = ps->genericEnemyIndex;

	s->activeForcePass = ps->activeForcePass;

	s->angles2[YAW] = ps->movementDir;
	s->legsAnim = ps->legsAnim;
	s->torsoAnim = ps->torsoAnim;

	s->legsFlip = ps->legsFlip;
	s->torsoFlip = ps->torsoFlip;

	s->clientNum = ps->clientNum;		// ET_PLAYER looks here instead of at number
										// so corpses can also reference the proper config
	s->eFlags = ps->eFlags;
	s->eFlags2 = ps->eFlags2;

	s->saberInFlight = ps->saberInFlight;
	s->saberEntityNum = ps->saberEntityNum;
	s->saberMove = ps->saberMove;
	s->forcePowersActive = ps->fd.forcePowersActive;

	if (ps->duelInProgress)
	{
		s->bolt1 = 1;
	}
	else
	{
		s->bolt1 = 0;
	}

	s->otherEntityNum2 = ps->emplacedIndex;

	s->saberHolstered = ps->saberHolstered;

	if (ps->genericEnemyIndex != -1)
	{
		s->eFlags |= EF_SEEKERDRONE;
	}

	if ( ps->stats[STAT_HEALTH] <= 0 ) {
		s->eFlags |= EF_DEAD;
	} else {
		s->eFlags &= ~EF_DEAD;
	}

	if ( ps->externalEvent ) {
		s->event = ps->externalEvent;
		s->eventParm = ps->externalEventParm;
	} else if ( ps->entityEventSequence < ps->eventSequence ) {
		int		seq;

		if ( ps->entityEventSequence < ps->eventSequence - MAX_PS_EVENTS) {
			ps->entityEventSequence = ps->eventSequence - MAX_PS_EVENTS;
		}
		seq = ps->entityEventSequence & (MAX_PS_EVENTS-1);
		s->event = ps->events[ seq ] | ( ( ps->entityEventSequence & 3 ) << 8 );
		s->eventParm = ps->eventParms[ seq ];
		ps->entityEventSequence++;
	}
	s->weapon = ps->weapon;
	s->groundEntityNum = ps->groundEntityNum;

	s->powerups = 0;
	for ( i = 0 ; i < MAX_POWERUPS ; i++ ) {
		if ( ps->powerups[ i ] ) {
			s->powerups |= 1 << i;
		}
	}

	s->loopSound = ps->loopSound;
	s->generic1 = ps->generic1;

	//NOT INCLUDED IN ENTITYSTATETOPLAYERSTATE:
	s->modelindex2 = ps->weaponstate;
	s->constantLight = ps->weaponChargeTime;

	VectorCopy(ps->lastHitLoc, s->origin2);

	s->isJediMaster = ps->isJediMaster;

	s->time2 = ps->holocronBits;

	s->fireflag = ps->fd.saberAnimLevel;

	s->heldByClient = ps->heldByClient;
	s->ragAttach = ps->ragAttach;

	s->iModelScale = ps->iModelScale;

	s->brokenLimbs = ps->brokenLimbs;

	s->hasLookTarget = ps->hasLookTarget;
	s->lookTarget = ps->lookTarget;

	s->customRGBA[0] = ps->customRGBA[0];
	s->customRGBA[1] = ps->customRGBA[1];
	s->customRGBA[2] = ps->customRGBA[2];
	s->customRGBA[3] = ps->customRGBA[3];

	s->m_iVehicleNum = ps->m_iVehicleNum;
}

#undef playerState_t
#undef entityState_t

/*
 * Drive one BG_PlayerStateToEntityStateExtraPolate call. Same flat-array marshalling and
 * index maps as jka_bg_player_state_to_entity_state (KEEP IN SYNC WITH THE RUST TEST),
 * with one extra input `time` and three extra outputs appended to out_i:
 *   out_i[44] = pos.trTime   out_i[45] = pos.trDuration   out_i[46] = isJediMaster
 * out_i[2] (pos.trType) now reports TR_LINEAR_STOP; everything else is identical.
 */
void jka_bg_player_state_to_entity_state_extrapolate(
		const int *in_i, const float *in_f, float in_speed,
		const int *in_events, const int *in_eventParms,
		const int *in_powerups, const int *in_customRGBA, int time, int snap,
		int *out_i, float *out_f, int *out_customRGBA, int *out_entityEventSequence) {
	pse_ps_t ps;
	pse_es_t s;
	int k;

	memset(&ps, 0, sizeof ps);
	memset(&s, 0, sizeof s);

	ps.pm_type            = in_i[0];
	ps.stats[0]           = in_i[1];
	ps.clientNum          = in_i[2];
	ps.fd.forceMindtrickTargetIndex  = in_i[3];
	ps.fd.forceMindtrickTargetIndex2 = in_i[4];
	ps.fd.forceMindtrickTargetIndex3 = in_i[5];
	ps.fd.forceMindtrickTargetIndex4 = in_i[6];
	ps.saberLockFrame     = in_i[7];
	ps.electrifyTime      = in_i[8];
	ps.genericEnemyIndex  = in_i[9];
	ps.activeForcePass    = in_i[10];
	ps.movementDir        = in_i[11];
	ps.legsAnim           = in_i[12];
	ps.torsoAnim          = in_i[13];
	ps.legsFlip           = in_i[14];
	ps.torsoFlip          = in_i[15];
	ps.eFlags             = in_i[16];
	ps.eFlags2            = in_i[17];
	ps.saberInFlight      = in_i[18];
	ps.saberEntityNum     = in_i[19];
	ps.saberMove          = in_i[20];
	ps.fd.forcePowersActive = in_i[21];
	ps.duelInProgress     = in_i[22];
	ps.emplacedIndex      = in_i[23];
	ps.saberHolstered     = in_i[24];
	ps.externalEvent      = in_i[25];
	ps.externalEventParm  = in_i[26];
	ps.entityEventSequence = in_i[27];
	ps.eventSequence      = in_i[28];
	ps.weapon             = in_i[29];
	ps.groundEntityNum    = in_i[30];
	ps.loopSound          = in_i[31];
	ps.generic1           = in_i[32];
	ps.weaponstate        = in_i[33];
	ps.weaponChargeTime   = in_i[34];
	ps.fd.saberAnimLevel  = in_i[35];
	ps.heldByClient       = in_i[36];
	ps.ragAttach          = in_i[37];
	ps.iModelScale        = in_i[38];
	ps.brokenLimbs        = in_i[39];
	ps.hasLookTarget      = in_i[40];
	ps.lookTarget         = in_i[41];
	ps.m_iVehicleNum      = in_i[42];
	ps.isJediMaster       = in_i[43];
	ps.holocronBits       = in_i[44];

	ps.origin[0] = in_f[0];  ps.origin[1] = in_f[1];  ps.origin[2] = in_f[2];
	ps.velocity[0] = in_f[3]; ps.velocity[1] = in_f[4]; ps.velocity[2] = in_f[5];
	ps.viewangles[0] = in_f[6]; ps.viewangles[1] = in_f[7]; ps.viewangles[2] = in_f[8];
	ps.lastHitLoc[0] = in_f[9]; ps.lastHitLoc[1] = in_f[10]; ps.lastHitLoc[2] = in_f[11];
	ps.speed = in_speed;
	ps.events[0] = in_events[0]; ps.events[1] = in_events[1];
	ps.eventParms[0] = in_eventParms[0]; ps.eventParms[1] = in_eventParms[1];
	for (k = 0; k < 16; k++) ps.powerups[k] = in_powerups[k];
	for (k = 0; k < 4; k++) ps.customRGBA[k] = in_customRGBA[k];

	BG_PlayerStateToEntityStateExtraPolate( &ps, &s, time, snap );

	out_i[0]  = s.eType;
	out_i[1]  = s.number;
	out_i[2]  = s.pos.trType;
	out_i[3]  = s.apos.trType;
	out_i[4]  = s.trickedentindex;
	out_i[5]  = s.trickedentindex2;
	out_i[6]  = s.trickedentindex3;
	out_i[7]  = s.trickedentindex4;
	out_i[8]  = s.forceFrame;
	out_i[9]  = s.emplacedOwner;
	out_i[10] = s.genericenemyindex;
	out_i[11] = s.activeForcePass;
	out_i[12] = s.legsAnim;
	out_i[13] = s.torsoAnim;
	out_i[14] = s.legsFlip;
	out_i[15] = s.torsoFlip;
	out_i[16] = s.clientNum;
	out_i[17] = s.eFlags;
	out_i[18] = s.eFlags2;
	out_i[19] = s.saberInFlight;
	out_i[20] = s.saberEntityNum;
	out_i[21] = s.saberMove;
	out_i[22] = s.forcePowersActive;
	out_i[23] = s.bolt1;
	out_i[24] = s.otherEntityNum2;
	out_i[25] = s.saberHolstered;
	out_i[26] = s.event;
	out_i[27] = s.eventParm;
	out_i[28] = s.weapon;
	out_i[29] = s.groundEntityNum;
	out_i[30] = s.powerups;
	out_i[31] = s.loopSound;
	out_i[32] = s.generic1;
	out_i[33] = s.modelindex2;
	out_i[34] = s.constantLight;
	out_i[35] = s.time2;
	out_i[36] = s.fireflag;
	out_i[37] = s.heldByClient;
	out_i[38] = s.ragAttach;
	out_i[39] = s.iModelScale;
	out_i[40] = s.brokenLimbs;
	out_i[41] = s.hasLookTarget;
	out_i[42] = s.lookTarget;
	out_i[43] = s.m_iVehicleNum;
	out_i[44] = s.pos.trTime;
	out_i[45] = s.pos.trDuration;
	out_i[46] = s.isJediMaster;

	out_f[0]  = s.pos.trBase[0];  out_f[1]  = s.pos.trBase[1];  out_f[2]  = s.pos.trBase[2];
	out_f[3]  = s.pos.trDelta[0]; out_f[4]  = s.pos.trDelta[1]; out_f[5]  = s.pos.trDelta[2];
	out_f[6]  = s.apos.trBase[0]; out_f[7]  = s.apos.trBase[1]; out_f[8]  = s.apos.trBase[2];
	out_f[9]  = s.angles2[0];     out_f[10] = s.angles2[1];     out_f[11] = s.angles2[2];
	out_f[12] = s.origin2[0];     out_f[13] = s.origin2[1];     out_f[14] = s.origin2[2];
	out_f[15] = s.speed;

	out_customRGBA[0] = s.customRGBA[0];
	out_customRGBA[1] = s.customRGBA[1];
	out_customRGBA[2] = s.customRGBA[2];
	out_customRGBA[3] = s.customRGBA[3];

	*out_entityEventSequence = ps.entityEventSequence;
}

/*
 * BG_LegalizedForcePowers (bg_misc.c:439): parse + legalize a force-config string
 * ("rank-side-PPPPPPPPPPPPPPPPPP", 18 power-level digits) against a point budget and
 * the supplied rules, writing the result back into powerOut and returning whether it was
 * already legal. Pure string/int logic — no struct crosses FFI, so the body is
 * transcribed VERBATIM here. The three pointer-free tables it reads (forceMasteryPoints,
 * bgForcePowerCost, forcePowerDarkLight) are transcribed verbatim above the body;
 * forcePowerSorted already lives in this TU (BG_CycleForce section) and is not reused
 * here. The FP_* / NUM_FORCE_POWERS macros from that section are reused as-is; the
 * remaining constants are transcribed from q_shared.h / bg_public.h. The libc string
 * helpers come from <string.h>/<stdlib.h>; va()/Q_strcat() are the authentic Raven
 * implementations linked from q_shared_oracle.c (declared here as prototypes).
 */
#include <string.h>   /* strlen, strcpy */
#include <stdlib.h>   /* atoi */

#define NUM_FORCE_POWER_LEVELS   4    /* q_shared.h forcePower_t */
#define NUM_FORCE_MASTERY_LEVELS 8    /* bg_public.h forceMasteryLevels_t */
#define FORCE_LIGHTSIDE 1            /* q_shared.h */
#define FORCE_DARKSIDE  2
#define FORCE_LEVEL_3   3            /* q_shared.h forcePowerLevels_t */
#define GT_TEAM 6                    /* bg_public.h gametype_t */
#define qboolean int                 /* q_shared.h */
#define qtrue  1
#define qfalse 0

/* authentic va()/Q_strcat() from q_shared_oracle.c (resolved at link time) */
char *va(const char *format, ...);
void Q_strcat(char *dest, int size, const char *src);

int forceMasteryPoints[NUM_FORCE_MASTERY_LEVELS] =
{
	0,		// FORCE_MASTERY_UNINITIATED,
	5,		// FORCE_MASTERY_INITIATE,
	10,		// FORCE_MASTERY_PADAWAN,
	20,		// FORCE_MASTERY_JEDI,
	30,		// FORCE_MASTERY_JEDI_GUARDIAN,
	50,		// FORCE_MASTERY_JEDI_ADEPT,
	75,		// FORCE_MASTERY_JEDI_KNIGHT,
	100		// FORCE_MASTERY_JEDI_MASTER,
};

int bgForcePowerCost[NUM_FORCE_POWERS][NUM_FORCE_POWER_LEVELS] = //0 == neutral
{
	{	0,	2,	4,	6	},	// Heal			// FP_HEAL
	{	0,	0,	2,	6	},	// Jump			//FP_LEVITATION,//hold/duration
	{	0,	2,	4,	6	},	// Speed		//FP_SPEED,//duration
	{	0,	1,	3,	6	},	// Push			//FP_PUSH,//hold/duration
	{	0,	1,	3,	6	},	// Pull			//FP_PULL,//hold/duration
	{	0,	4,	6,	8	},	// Mind Trick	//FP_TELEPATHY,//instant
	{	0,	1,	3,	6	},	// Grip			//FP_GRIP,//hold/duration
	{	0,	2,	5,	8	},	// Lightning	//FP_LIGHTNING,//hold/duration
	{	0,	4,	6,	8	},	// Dark Rage	//FP_RAGE,//duration
	{	0,	2,	5,	8	},	// Protection	//FP_PROTECT,//duration
	{	0,	1,	3,	6	},	// Absorb		//FP_ABSORB,//duration
	{	0,	1,	3,	6	},	// Team Heal	//FP_TEAM_HEAL,//instant
	{	0,	1,	3,	6	},	// Team Force	//FP_TEAM_FORCE,//instant
	{	0,	2,	4,	6	},	// Drain		//FP_DRAIN,//hold/duration
	{	0,	2,	5,	8	},	// Sight		//FP_SEE,//duration
	{	0,	1,	5,	8	},	// Saber Attack	//FP_SABER_OFFENSE,
	{	0,	1,	5,	8	},	// Saber Defend	//FP_SABER_DEFENSE,
	{	0,	4,	6,	8	}	// Saber Throw	//FP_SABERTHROW,
	//NUM_FORCE_POWERS
};

int forcePowerDarkLight[NUM_FORCE_POWERS] = //0 == neutral
{ //nothing should be usable at rank 0..
	FORCE_LIGHTSIDE,//FP_HEAL,//instant
	0,//FP_LEVITATION,//hold/duration
	0,//FP_SPEED,//duration
	0,//FP_PUSH,//hold/duration
	0,//FP_PULL,//hold/duration
	FORCE_LIGHTSIDE,//FP_TELEPATHY,//instant
	FORCE_DARKSIDE,//FP_GRIP,//hold/duration
	FORCE_DARKSIDE,//FP_LIGHTNING,//hold/duration
	FORCE_DARKSIDE,//FP_RAGE,//duration
	FORCE_LIGHTSIDE,//FP_PROTECT,//duration
	FORCE_LIGHTSIDE,//FP_ABSORB,//duration
	FORCE_LIGHTSIDE,//FP_TEAM_HEAL,//instant
	FORCE_DARKSIDE,//FP_TEAM_FORCE,//instant
	FORCE_DARKSIDE,//FP_DRAIN,//hold/duration
	0,//FP_SEE,//duration
	0,//FP_SABER_OFFENSE,
	0,//FP_SABER_DEFENSE,
	0//FP_SABERTHROW,
		//NUM_FORCE_POWERS
};

/* verbatim body of bg_misc.c:439 */
qboolean BG_LegalizedForcePowers(char *powerOut, int maxRank, qboolean freeSaber, int teamForce, int gametype, int fpDisabled)
{
	char powerBuf[128];
	char readBuf[128];
	qboolean maintainsValidity = qtrue;
	int powerLen = strlen(powerOut);
	int i = 0;
	int c = 0;
	int allowedPoints = 0;
	int usedPoints = 0;
	int countDown = 0;

	int final_Side;
	int final_Powers[NUM_FORCE_POWERS];

	if (powerLen >= 128)
	{ //This should not happen. If it does, this is obviously a bogus string.
		//They can have this string. Because I said so.
		strcpy(powerBuf, "7-1-032330000000001333");
		maintainsValidity = qfalse;
	}
	else
	{
		strcpy(powerBuf, powerOut); //copy it as the original
	}

	//first of all, print the max rank into the string as the rank
	strcpy(powerOut, va("%i-", maxRank));

	while (i < 128 && powerBuf[i] && powerBuf[i] != '-')
	{
		i++;
	}
	i++;
	while (i < 128 && powerBuf[i] && powerBuf[i] != '-')
	{
		readBuf[c] = powerBuf[i];
		c++;
		i++;
	}
	readBuf[c] = 0;
	i++;
	//at this point, readBuf contains the intended side
	final_Side = atoi(readBuf);

	if (final_Side != FORCE_LIGHTSIDE &&
		final_Side != FORCE_DARKSIDE)
	{ //Not a valid side. You will be dark. Because I said so. (this is something that should never actually happen unless you purposely feed in an invalid config)
		final_Side = FORCE_DARKSIDE;
		maintainsValidity = qfalse;
	}

	if (teamForce)
	{ //If we are under force-aligned teams, make sure we're on the right side.
		if (final_Side != teamForce)
		{
			final_Side = teamForce;
			//maintainsValidity = qfalse;
			//Not doing this, for now. Let them join the team with their filtered powers.
		}
	}

	//Now we have established a valid rank, and a valid side.
	//Read the force powers in, and cut them down based on the various rules supplied.
	c = 0;
	while (i < 128 && powerBuf[i] && powerBuf[i] != '\n' && c < NUM_FORCE_POWERS)
	{
		readBuf[0] = powerBuf[i];
		readBuf[1] = 0;
		final_Powers[c] = atoi(readBuf);
		c++;
		i++;
	}

	//final_Powers now contains all the stuff from the string
	//Set the maximum allowed points used based on the max rank level, and count the points actually used.
	allowedPoints = forceMasteryPoints[maxRank];

	i = 0;
	while (i < NUM_FORCE_POWERS)
	{ //if this power doesn't match the side we're on, then 0 it now.
		if (final_Powers[i] &&
			forcePowerDarkLight[i] &&
			forcePowerDarkLight[i] != final_Side)
		{
			final_Powers[i] = 0;
			//This is only likely to happen with g_forceBasedTeams. Let it slide.
		}

		if ( final_Powers[i] &&
			(fpDisabled & (1 << i)) )
		{ //if this power is disabled on the server via said server option, then we don't get it.
			final_Powers[i] = 0;
		}

		i++;
	}

	if (gametype < GT_TEAM)
	{ //don't bother with team powers then
		final_Powers[FP_TEAM_HEAL] = 0;
		final_Powers[FP_TEAM_FORCE] = 0;
	}

	usedPoints = 0;
	i = 0;
	while (i < NUM_FORCE_POWERS)
	{
		countDown = 0;

		countDown = final_Powers[i];

		while (countDown > 0)
		{
			usedPoints += bgForcePowerCost[i][countDown]; //[fp index][fp level]
			//if this is jump, or we have a free saber and it's offense or defense, take the level back down on level 1
			if ( countDown == 1 &&
				((i == FP_LEVITATION) ||
				 (i == FP_SABER_OFFENSE && freeSaber) ||
				 (i == FP_SABER_DEFENSE && freeSaber)) )
			{
				usedPoints -= bgForcePowerCost[i][countDown];
			}
			countDown--;
		}

		i++;
	}

	if (usedPoints > allowedPoints)
	{ //Time to do the fancy stuff. (meaning, slowly cut parts off while taking a guess at what is most or least important in the config)
		int attemptedCycles = 0;
		int powerCycle = 2;
		int minPow = 0;

		if (freeSaber)
		{
			minPow = 1;
		}

		maintainsValidity = qfalse;

		while (usedPoints > allowedPoints)
		{
			c = 0;

			while (c < NUM_FORCE_POWERS && usedPoints > allowedPoints)
			{
				if (final_Powers[c] && final_Powers[c] < powerCycle)
				{ //kill in order of lowest powers, because the higher powers are probably more important
					if (c == FP_SABER_OFFENSE &&
						(final_Powers[FP_SABER_DEFENSE] > minPow || final_Powers[FP_SABERTHROW] > 0))
					{ //if we're on saber attack, only suck it down if we have no def or throw either
						int whichOne = FP_SABERTHROW; //first try throw

						if (!final_Powers[whichOne])
						{
							whichOne = FP_SABER_DEFENSE; //if no throw, drain defense
						}

						while (final_Powers[whichOne] > 0 && usedPoints > allowedPoints)
						{
							if ( final_Powers[whichOne] > 1 ||
								( (whichOne != FP_SABER_OFFENSE || !freeSaber) &&
								  (whichOne != FP_SABER_DEFENSE || !freeSaber) ) )
							{ //don't take attack or defend down on level 1 still, if it's free
								usedPoints -= bgForcePowerCost[whichOne][final_Powers[whichOne]];
								final_Powers[whichOne]--;
							}
							else
							{
								break;
							}
						}
					}
					else
					{
						while (final_Powers[c] > 0 && usedPoints > allowedPoints)
						{
							if ( final_Powers[c] > 1 ||
								((c != FP_LEVITATION) &&
								(c != FP_SABER_OFFENSE || !freeSaber) &&
								(c != FP_SABER_DEFENSE || !freeSaber)) )
							{
								usedPoints -= bgForcePowerCost[c][final_Powers[c]];
								final_Powers[c]--;
							}
							else
							{
								break;
							}
						}
					}
				}

				c++;
			}

			powerCycle++;
			attemptedCycles++;

			if (attemptedCycles > NUM_FORCE_POWERS)
			{ //I think this should be impossible. But just in case.
				break;
			}
		}

		if (usedPoints > allowedPoints)
		{ //Still? Fine then.. we will kill all of your powers, except the freebies.
			i = 0;

			while (i < NUM_FORCE_POWERS)
			{
				final_Powers[i] = 0;
				if (i == FP_LEVITATION ||
					(i == FP_SABER_OFFENSE && freeSaber) ||
					(i == FP_SABER_DEFENSE && freeSaber))
				{
					final_Powers[i] = 1;
				}
				i++;
			}
			usedPoints = 0;
		}
	}

	if (freeSaber)
	{
		if (final_Powers[FP_SABER_OFFENSE] < 1)
		{
			final_Powers[FP_SABER_OFFENSE] = 1;
		}
		if (final_Powers[FP_SABER_DEFENSE] < 1)
		{
			final_Powers[FP_SABER_DEFENSE] = 1;
		}
	}
	if (final_Powers[FP_LEVITATION] < 1)
	{
		final_Powers[FP_LEVITATION] = 1;
	}

	i = 0;
	while (i < NUM_FORCE_POWERS)
	{
		if (final_Powers[i] > FORCE_LEVEL_3)
		{
			final_Powers[i] = FORCE_LEVEL_3;
		}
		i++;
	}

	if (fpDisabled)
	{ //If we specifically have attack or def disabled, force them up to level 3. It's the way
	  //things work for the case of all powers disabled.
	  //If jump is disabled, down-cap it to level 1. Otherwise don't do a thing.
		if (fpDisabled & (1 << FP_LEVITATION))
		{
			final_Powers[FP_LEVITATION] = 1;
		}
		if (fpDisabled & (1 << FP_SABER_OFFENSE))
		{
			final_Powers[FP_SABER_OFFENSE] = 3;
		}
		if (fpDisabled & (1 << FP_SABER_DEFENSE))
		{
			final_Powers[FP_SABER_DEFENSE] = 3;
		}
	}

	if (final_Powers[FP_SABER_OFFENSE] < 1)
	{
		final_Powers[FP_SABER_DEFENSE] = 0;
		final_Powers[FP_SABERTHROW] = 0;
	}

	//We finally have all the force powers legalized and stored locally.
	//Put them all into the string and return the result. We already have
	//the rank there, so print the side and the powers now.
	Q_strcat(powerOut, 128, va("%i-", final_Side));

	i = strlen(powerOut);
	c = 0;
	while (c < NUM_FORCE_POWERS)
	{
		strcpy(readBuf, va("%i", final_Powers[c]));
		powerOut[i] = readBuf[0];
		c++;
		i++;
	}
	powerOut[i] = 0;

	return maintainsValidity;
}

#undef qboolean
#undef qtrue
#undef qfalse

/*
 * Drive one BG_LegalizedForcePowers call: io_powerOut is the in/out config buffer (the
 * Rust test passes a 128-byte buffer holding the input string; C rewrites it in place).
 * Returns the qboolean (1/0) verbatim.
 */
int jka_bg_legalize_force_powers(char *io_powerOut, int maxRank, int freeSaber,
                                 int teamForce, int gametype, int fpDisabled) {
	return BG_LegalizedForcePowers(io_powerOut, maxRank, freeSaber, teamForce, gametype, fpDisabled);
}

/*
 * ---------------------------------------------------------------------------
 * bg_misc.c file-scope DATA TABLES (bg_misc.c:34/78/113/150) — verbatim copies
 * for element-wise parity against the Rust statics in bg_misc.rs. The size
 * macros are transcribed from bg_public.h / q_shared.h (independent-table-copy
 * discipline, the bg_itemlist precedent). The accessors hand the base pointer
 * (and a count) across FFI; the Rust side walks them index-by-index.
 * ---------------------------------------------------------------------------
 */
#define BG_NUM_TOGGLEABLE_SURFACES 31 /* bg_public.h */
#define MAX_CUSTOM_SIEGE_SOUNDS    30 /* bg_public.h */
#define NUM_FORCE_MASTERY_LEVELS    8 /* bg_public.h */

/* verbatim bg_misc.c:34 */
const char *bgToggleableSurfaces[BG_NUM_TOGGLEABLE_SURFACES] =
{
	"l_arm_key",					//0
	"torso_canister1",
	"torso_canister2",
	"torso_canister3",
	"torso_tube1",
	"torso_tube2",					//5
	"torso_tube3",
	"torso_tube4",
	"torso_tube5",
	"torso_tube6",
	"r_arm",						//10
	"l_arm",
	"torso_shield",
	"torso_galaktorso",
	"torso_collar",
	"r_wing1",						//15
	"r_wing2",
	"l_wing1",
	"l_wing2",
	"r_gear",
	"l_gear",						//20
	"nose",
	"blah4",
	"blah5",
	"l_hand",
	"r_hand",						//25
	"helmet",
	"head",
	"head_concussion_charger",
	"head_light_blaster_cann",		//29
	0
};

/* verbatim bg_misc.c:78 */
const int bgToggleableSurfaceDebris[BG_NUM_TOGGLEABLE_SURFACES] =
{
	0,0,0,0,0,					//0..4
	0,0,0,0,0,					//5..9
	0,0,0,0,0,					//10..14
	3,5,4,6,0,					//15..19
	0,7,0,0,0,					//20..24
	0,0,0,0,0,					//25..29
	-1
};

/* verbatim bg_misc.c:113 */
const char *bg_customSiegeSoundNames[MAX_CUSTOM_SIEGE_SOUNDS] =
{
	"*att_attack",
	"*att_primary",
	"*att_second",
	"*def_guns",
	"*def_position",
	"*def_primary",
	"*def_second",
	"*reply_coming",
	"*reply_go",
	"*reply_no",
	"*reply_stay",
	"*reply_yes",
	"*req_assist",
	"*req_demo",
	"*req_hvy",
	"*req_medic",
	"*req_sup",
	"*req_tech",
	"*spot_air",
	"*spot_defenses",
	"*spot_emplaced",
	"*spot_sniper",
	"*spot_troops",
	"*tac_cover",
	"*tac_fallback",
	"*tac_follow",
	"*tac_hold",
	"*tac_split",
	"*tac_together",
	0
};

/* verbatim bg_misc.c:150 */
char *forceMasteryLevels[NUM_FORCE_MASTERY_LEVELS] =
{
	"MASTERY0",
	"MASTERY1",
	"MASTERY2",
	"MASTERY3",
	"MASTERY4",
	"MASTERY5",
	"MASTERY6",
	"MASTERY7",
};

const char *const *jka_bg_toggleable_surfaces_ptr(void) { return bgToggleableSurfaces; }
const int        *jka_bg_toggleable_surface_debris_ptr(void) { return bgToggleableSurfaceDebris; }
const char *const *jka_bg_custom_siege_sound_names_ptr(void) { return bg_customSiegeSoundNames; }
const char *const *jka_bg_force_mastery_levels_ptr(void) { return (const char *const *)forceMasteryLevels; }
