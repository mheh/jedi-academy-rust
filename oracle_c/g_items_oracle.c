/* Oracle extraction of g_items.c's pure respawn-time scaler:
 *
 *   adjustRespawnTime (refs/raven-jediacademy/codemp/game/g_items.c:47)
 *
 * The function is plain arithmetic over two process globals — g_adaptRespawn.integer
 * and level.numPlayingClients — plus the item type/tag enums. Rather than reproduce
 * the whole cvar/level layout, the two globals are lifted to `jka_` wrapper params;
 * the body is otherwise transcribed verbatim (including the IT_WEAPON / WP_THERMAL /
 * WP_TRIP_MINE / WP_DET_PACK special-case and the RESPAWN_AMMO define). Exposed under
 * a `jka_` prefix so it does not collide with the test binary's own symbols. */

/* g_items.c:24 — RESPAWN_AMMO. */
#define RESPAWN_AMMO 40

/* itemType_t / weapon_t enum values used by the special case (bg_public.h /
 * bg_weapons.h). Only the ones the body compares against are needed. */
#define IT_WEAPON     1
#define WP_THERMAL    12
#define WP_TRIP_MINE  13
#define WP_DET_PACK   14

/* g_items.c:47 — adjustRespawnTime, transcribed verbatim with the two read globals
 * (g_adaptRespawn.integer, level.numPlayingClients) lifted to parameters. */
int jka_adjustRespawnTime(float preRespawnTime, int itemType, int itemTag,
                          int g_adaptRespawn_integer, int numPlayingClients)
{
	float respawnTime = preRespawnTime;

	if (itemType == IT_WEAPON)
	{
		if (itemTag == WP_THERMAL ||
			itemTag == WP_TRIP_MINE ||
			itemTag == WP_DET_PACK)
		{ //special case for these, use ammo respawn rate
			respawnTime = RESPAWN_AMMO;
		}
	}

	if (!g_adaptRespawn_integer)
	{
		return((int)respawnTime);
	}

	if (numPlayingClients > 4)
	{	// Start scaling the respawn times.
		if (numPlayingClients > 32)
		{	// 1/4 time minimum.
			respawnTime *= 0.25;
		}
		else if (numPlayingClients > 12)
		{	// From 12-32, scale from 0.5 to 0.25;
			respawnTime *= 20.0 / (float)(numPlayingClients + 8);
		}
		else
		{	// From 4-12, scale from 1.0 to 0.5;
			respawnTime *= 8.0 / (float)(numPlayingClients + 4);
		}
	}

	if (respawnTime < 1.0)
	{	// No matter what, don't go lower than 1 second, or the pickups become very noisy!
		respawnTime = 1.0;
	}

	return ((int)respawnTime);
}
