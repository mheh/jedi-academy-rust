/*
 * Oracle TU for g_vehicles.c leaves. Currently just the pure surface-name predicate
 * G_ShipSurfaceForSurfName (g_vehicles.c:2650): a verbatim chain of Q_strncmp tests
 * mapping a surface name to a SHIPSURF_* index. Self-contained -- the lone callee
 * Q_strncmp (q_shared.c:881) is inlined `static` here and the SHIPSURF_* enum copied,
 * so the TU has no headers/globals.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

/* ---- SHIPSURF_* (bg_vehicles.h: impact damage surface stuff) ---- */
#define SHIPSURF_FRONT 0
#define SHIPSURF_BACK  1
#define SHIPSURF_RIGHT 2
#define SHIPSURF_LEFT  3

/* Q_strncmp (q_shared.c:881) -- verbatim, made static for this TU. */
static int Q_strncmp (const char *s1, const char *s2, int n) {
	int		c1, c2;

	do {
		c1 = *s1++;
		c2 = *s2++;

		if (!n--) {
			return 0;		// strings are equal until end point
		}

		if (c1 != c2) {
			return c1 < c2 ? -1 : 1;
		}
	} while (c1);

	return 0;		// strings are equal
}

int G_ShipSurfaceForSurfName( const char *surfaceName )
{
	if ( !surfaceName )
	{
		return -1;
	}
	if ( !Q_strncmp( "nose", surfaceName, 4 )
		|| !Q_strncmp( "f_gear", surfaceName, 6 )
		|| !Q_strncmp( "glass", surfaceName, 5 ) )
	{
		return SHIPSURF_FRONT;
	}
	if ( !Q_strncmp( "body", surfaceName, 4 ) )
	{
		return SHIPSURF_BACK;
	}
	if ( !Q_strncmp( "r_wing1", surfaceName, 7 )
		|| !Q_strncmp( "r_wing2", surfaceName, 7 )
		|| !Q_strncmp( "r_gear", surfaceName, 6 ) )
	{
		return SHIPSURF_RIGHT;
	}
	if ( !Q_strncmp( "l_wing1", surfaceName, 7 )
		|| !Q_strncmp( "l_wing2", surfaceName, 7 )
		|| !Q_strncmp( "l_gear", surfaceName, 6 ) )
	{
		return SHIPSURF_LEFT;
	}
	return -1;
}
