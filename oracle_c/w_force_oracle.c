/* Extracted w_force.c functions, compiled as a parity oracle. See the header in
   q_math_oracle.c for the method. Function *bodies* are the authentic Raven source
   from raven-jediacademy/codemp/game/w_force.c, verbatim except for documented ORACLE
   DEVIATIONs. */

/* Mirror of the engine-side enum constants the bodies branch on
   (codemp/game/q_shared.h). */
#define NUM_FORCE_POWERS 18
#define FP_LEVITATION    1
#define FORCE_LEVEL_0    0
#define FORCE_LEVEL_1    1

/* w_force.c:5015. ORACLE DEVIATION: the body reads only `ps->fd.forcePowerLevel[]`, so the
   oracle takes that array directly (the `jka_HasSetSaberOnly` pass-the-read-fields precedent);
   the `if (ps)` NULL-guard is reproduced via `has_ps` (0 = NULL ps). Renamed to
   jka_WP_HasForcePowers to dodge any host symbol; qboolean -> int, const playerState_t* -> the
   forcePowerLevel array. Control flow otherwise verbatim. */
int jka_WP_HasForcePowers(int has_ps, const int forcePowerLevel[NUM_FORCE_POWERS]) {
	int i;
	if ( has_ps )
	{
		for ( i = 0; i < NUM_FORCE_POWERS; i++ )
		{
			if ( i == FP_LEVITATION )
			{
				if ( forcePowerLevel[i] > FORCE_LEVEL_1 )
				{
					return 1;
				}
			}
			else if ( forcePowerLevel[i] > FORCE_LEVEL_0 )
			{
				return 1;
			}
		}
	}
	return 0;
}

/* w_force.c:4162. ORACLE DEVIATION: the body reads only the four `fd->forceMindtrickTargetIndex*`
   scalars, so the oracle takes them directly; the `if (!fd)` NULL-guard is reproduced via
   `has_fd` (0 = NULL fd). Renamed to jka_G_IsMindTricked to dodge any host symbol; qboolean ->
   int, forcedata_t* -> the four trick indices. Control flow otherwise verbatim. */
int jka_G_IsMindTricked(int has_fd, int trickIndex1, int trickIndex2, int trickIndex3,
                        int trickIndex4, int client) {
	int checkIn;
	int sub = 0;

	if (!has_fd)
	{
		return 0;
	}

	if (client > 47)
	{
		checkIn = trickIndex4;
		sub = 48;
	}
	else if (client > 31)
	{
		checkIn = trickIndex3;
		sub = 32;
	}
	else if (client > 15)
	{
		checkIn = trickIndex2;
		sub = 16;
	}
	else
	{
		checkIn = trickIndex1;
	}

	if (checkIn & (1 << (client-sub)))
	{
		return 1;
	}

	return 0;
}

/* w_force.c:1301. ORACLE DEVIATION: the body mutates only the four `ent->s.trickedentindex*`
   scalars, so the oracle takes pointers to them (in/out); the `if (!ent)` NULL-guard is
   reproduced via `has_ent` (0 = NULL ent). Renamed to jka_WP_AddToClientBitflags to dodge any
   host symbol; gentity_t* -> the four trick-index pointers. Control flow otherwise verbatim. */
void jka_WP_AddToClientBitflags(int has_ent, int *trickIndex1, int *trickIndex2,
                                int *trickIndex3, int *trickIndex4, int entNum) {
	if (!has_ent)
	{
		return;
	}

	if (entNum > 47)
	{
		*trickIndex4 |= (1 << (entNum-48));
	}
	else if (entNum > 31)
	{
		*trickIndex3 |= (1 << (entNum-32));
	}
	else if (entNum > 15)
	{
		*trickIndex2 |= (1 << (entNum-16));
	}
	else
	{
		*trickIndex1 |= (1 << entNum);
	}
}

/* w_force.c:2509. ORACLE DEVIATION: the body mutates only the four
   `fd->forceMindtrickTargetIndex*` scalars, so the oracle takes pointers to them (in/out); the
   `if (!fd)` NULL-guard is reproduced via `has_fd` (0 = NULL fd). Renamed to
   jka_WP_AddAsMindtricked to dodge any host symbol; forcedata_t* -> the four target-index
   pointers. Control flow otherwise verbatim. */
void jka_WP_AddAsMindtricked(int has_fd, int *targetIndex1, int *targetIndex2,
                             int *targetIndex3, int *targetIndex4, int entNum) {
	if (!has_fd)
	{
		return;
	}

	if (entNum > 47)
	{
		*targetIndex4 |= (1 << (entNum-48));
	}
	else if (entNum > 31)
	{
		*targetIndex3 |= (1 << (entNum-32));
	}
	else if (entNum > 15)
	{
		*targetIndex2 |= (1 << (entNum-16));
	}
	else
	{
		*targetIndex1 |= (1 << entNum);
	}
}

/* w_force.c:4206. ORACLE DEVIATION: the body mutates only the four
   `fd->forceMindtrickTargetIndex*` scalars, so the oracle takes pointers to them (in/out); the
   `if (!fd)` NULL-guard is reproduced via `has_fd` (0 = NULL fd). Renamed to jka_RemoveTrickedEnt
   to dodge any host symbol; forcedata_t* -> the four target-index pointers. The clear-bit mirror
   of jka_WP_AddAsMindtricked. Control flow otherwise verbatim. */
void jka_RemoveTrickedEnt(int has_fd, int *targetIndex1, int *targetIndex2,
                          int *targetIndex3, int *targetIndex4, int client) {
	if (!has_fd)
	{
		return;
	}

	if (client > 47)
	{
		*targetIndex4 &= ~(1 << (client-48));
	}
	else if (client > 31)
	{
		*targetIndex3 &= ~(1 << (client-32));
	}
	else if (client > 15)
	{
		*targetIndex2 &= ~(1 << (client-16));
	}
	else
	{
		*targetIndex1 &= ~(1 << client);
	}
}

/* Getup-anim constants the body branches on (codemp/game/anims.h). Values mirror anims.rs. */
#define BOTH_GETUP1          1224
#define BOTH_GETUP2          1225
#define BOTH_GETUP3          1226
#define BOTH_GETUP4          1227
#define BOTH_GETUP5          1228
#define BOTH_FORCE_GETUP_F1  1231
#define BOTH_FORCE_GETUP_F2  1232
#define BOTH_FORCE_GETUP_B1  1233
#define BOTH_FORCE_GETUP_B2  1234
#define BOTH_FORCE_GETUP_B3  1235
#define BOTH_FORCE_GETUP_B4  1236
#define BOTH_FORCE_GETUP_B5  1237
#define BOTH_GETUP_BROLL_B   1239
#define BOTH_GETUP_BROLL_F   1240
#define BOTH_GETUP_BROLL_L   1241
#define BOTH_GETUP_BROLL_R   1242
#define BOTH_GETUP_FROLL_B   1243
#define BOTH_GETUP_FROLL_F   1244
#define BOTH_GETUP_FROLL_L   1245
#define BOTH_GETUP_FROLL_R   1246

/* w_force.c:2977. ORACLE DEVIATION: the body reads only `ps->legsAnim` and `ps->torsoAnim`, so
   the oracle takes those two ints directly (the pass-the-read-fields precedent). Renamed to
   jka_G_InGetUpAnim to dodge any host symbol; qboolean -> int, playerState_t* -> the two anim
   ints. Control flow (the two switches, legs first) otherwise verbatim. */
int jka_G_InGetUpAnim(int legsAnim, int torsoAnim) {
	switch( (legsAnim) )
	{
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
		return 1;
	}

	switch( (torsoAnim) )
	{
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
		return 1;
	}

	return 0;
}
