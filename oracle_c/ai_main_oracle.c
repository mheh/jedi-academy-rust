/* Extracted ai_main.c functions, compiled as a parity oracle. See the header in
   q_math_oracle.c for the method. Function *bodies* are the authentic Raven source
   from raven-jediacademy/codemp/game/ai_main.c, verbatim except for documented ORACLE
   DEVIATIONs. */

typedef float vec_t;
typedef vec_t vec3_t[3];

/* AngleMod (q_math.c:697), copied `static` so this TU is self-contained and does not
   collide with the `AngleMod` symbol already exported by q_math_oracle.c. Body verbatim;
   InFieldOfVision's only callee. */
static float jka_im_AngleMod(float a) {
	a = (360.0/65536) * ((int)(a*(65536/360.0)) & 65535);
	return a;
}

/* ai_main.c:2009. Renamed jka_InFieldOfVision to dodge any host symbol; control flow and
   FP math verbatim, with the one AngleMod call routed to the static copy above. */
//check if said angles are within our fov
int jka_InFieldOfVision(vec3_t viewangles, float fov, vec3_t angles)
{
	int i;
	float diff, angle;

	for (i = 0; i < 2; i++)
	{
		angle = jka_im_AngleMod(viewangles[i]);
		angles[i] = jka_im_AngleMod(angles[i]);
		diff = angles[i] - angle;
		if (angles[i] > angle)
		{
			if (diff > 180.0)
			{
				diff -= 360.0;
			}
		}
		else
		{
			if (diff < -180.0)
			{
				diff += 360.0;
			}
		}
		if (diff > 0)
		{
			if (diff > fov * 0.5)
			{
				return 0;
			}
		}
		else
		{
			if (diff < -fov * 0.5)
			{
				return 0;
			}
		}
	}
	return 1;
}

/* ai_main.c:421. Pure float math, no callees. Renamed to dodge any host symbol; body
   verbatim. */
//==============
//AngleDifference
//==============
float jka_AngleDifference(float ang1, float ang2)
{
	float diff;

	diff = ang1 - ang2;
	if (ang1 > ang2)
	{
		if (diff > 180.0) diff -= 360.0;
	}
	else
	{
		if (diff < -180.0) diff += 360.0;
	}
	return diff;
}

/* ai_main.c:439. Renamed to dodge any host symbol; the two AngleMod calls routed to the
   static copy above. Control flow and FP math verbatim. */
//==============
//BotChangeViewAngle
//==============
float jka_BotChangeViewAngle(float angle, float ideal_angle, float speed)
{
	float move;

	angle = jka_im_AngleMod(angle);
	ideal_angle = jka_im_AngleMod(ideal_angle);
	if (angle == ideal_angle) return angle;
	move = ideal_angle - angle;
	if (ideal_angle > angle)
	{
		if (move > 180.0) move -= 360.0;
	}
	else
	{
		if (move < -180.0) move += 360.0;
	}
	if (move > 0)
	{
		if (move > speed) move = speed;
	}
	else
	{
		if (move < -speed) move = -speed;
	}
	return jka_im_AngleMod(angle + move);
}

/* weapon_t enum (bg_weapons.h:8) copied verbatim so this TU is self-contained — the only
   ids BotWeaponBlockable switches on. */
typedef enum {
	WP_NONE,

	WP_STUN_BATON,
	WP_MELEE,
	WP_SABER,
	WP_BRYAR_PISTOL,
	WP_BLASTER,
	WP_DISRUPTOR,
	WP_BOWCASTER,
	WP_REPEATER,
	WP_DEMP2,
	WP_FLECHETTE,
	WP_ROCKET_LAUNCHER,
	WP_THERMAL,
	WP_TRIP_MINE,
	WP_DET_PACK,
	WP_CONCUSSION,
	WP_BRYAR_OLD,
	WP_EMPLACED_GUN,
	WP_TURRET,

	WP_NUM_WEAPONS
} jka_weapon_t;

/* ai_main.c:5839. Renamed to dodge any host symbol; body verbatim. */
//could we block projectiles from the weapon potentially with a light saber?
int jka_BotWeaponBlockable(int weapon)
{
	switch (weapon)
	{
	case WP_STUN_BATON:
	case WP_MELEE:
		return 0;
	case WP_DISRUPTOR:
		return 0;
	case WP_DEMP2:
		return 0;
	case WP_ROCKET_LAUNCHER:
		return 0;
	case WP_THERMAL:
		return 0;
	case WP_TRIP_MINE:
		return 0;
	case WP_DET_PACK:
		return 0;
	default:
		return 1;
	}
}
