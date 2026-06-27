/* Oracle: StringToFilter extracted verbatim from the original
   raven-jediacademy/codemp/game/g_svcmds.c (the dotted-IP address parser for the
   packet / IP-ban filter list), compared against OpenJK/codemp/game/g_svcmds.c --
   which is byte-identical here. Compiled and linked under the `oracle` cargo
   feature (see build.rs) so Rust tests can call the real C and assert the port in
   src/codemp/game/g_svcmds.rs matches bit-for-bit.

   StringToFilter's only dependencies are libc `atoi` (real, via <stdlib.h>) and
   `G_Printf` on the bad-address path -- stubbed to a no-op here, since parity is on
   the parser's outputs (mask / compare / return value), not the log side effect.
   The jka_StringToFilter wrapper runs it and returns those outputs. `ipFilter_t` /
   `byte` / `qboolean` are copied verbatim (they never cross an ABI). */
#include <stdlib.h>

typedef unsigned char byte;
typedef int qboolean;
#define qfalse 0
#define qtrue  1

typedef struct ipFilter_s
{
	unsigned	mask;
	unsigned	compare;
} ipFilter_t;

/* StringToFilter only logs through this on a bad address; the no-op keeps the
   oracle self-contained (no engine / trap layer to link). */
static void G_Printf( const char *fmt, ... ) { (void)fmt; }

/* --- verbatim from g_svcmds.c --- */
static qboolean StringToFilter (char *s, ipFilter_t *f)
{
	char	num[128];
	int		i, j;
	byte	b[4];
	byte	m[4];

	for (i=0 ; i<4 ; i++)
	{
		b[i] = 0;
		m[i] = 0;
	}

	for (i=0 ; i<4 ; i++)
	{
		if (*s < '0' || *s > '9')
		{
			G_Printf( "Bad filter address: %s\n", s );
			return qfalse;
		}

		j = 0;
		while (*s >= '0' && *s <= '9')
		{
			num[j++] = *s++;
		}
		num[j] = 0;
		b[i] = atoi(num);
		if (b[i] != 0)
			m[i] = 255;

		if (!*s)
			break;
		s++;
	}

	f->mask = *(unsigned *)m;
	f->compare = *(unsigned *)b;

	return qtrue;
}

/* Test shim: run StringToFilter on `s` (it only reads its argument, so the const
   cast is safe) and hand back its mask / compare / return value. `f` is pre-zeroed
   so the qfalse path (which leaves `f` untouched) reports the same 0/0 the Rust
   port's caller-initialized struct does. */
qboolean jka_StringToFilter( const char *s, unsigned *mask, unsigned *compare )
{
	ipFilter_t	f;
	qboolean	ret;

	f.mask = 0;
	f.compare = 0;
	ret = StringToFilter( (char *)s, &f );
	*mask = f.mask;
	*compare = f.compare;
	return ret;
}

/* --- G_FilterPacket oracle ---------------------------------------------------

   G_FilterPacket reads three module globals: the in-memory filter list
   (`ipFilters` / `numIPFilters`) and the `g_filterBan` vmCvar. The verbatim body
   below keeps its own copies of those statics; the jka_G_FilterPacket wrapper
   seeds them from the test's parameters before each call, so the Rust port (which
   reads its own module statics, seeded identically) can be compared bit-for-bit on
   the same (address-string, filter-list, filterBan) triple.

   The IP-string parse path is the interesting computation: dotted octets folded
   into a little-endian u32 with `byte` wraparound, stopping at NUL or ':' (the
   `addr:port` form). The filter-list scan and the g_filterBan allow/deny polarity
   are the rest. */

#define MAX_ORACLE_IPFILTERS 1024
static ipFilter_t	oracleIpFilters[MAX_ORACLE_IPFILTERS];
static int			oracleNumIPFilters;
static int			oracleFilterBan;

/* verbatim from g_svcmds.c, with the three globals renamed to the oracle copies */
static qboolean G_FilterPacket (char *from)
{
	int				i;
	unsigned int	in;
	byte			m[4];
	char			*p;

	i = 0;
	while (i < 4)
	{
		m[i] = 0;
		i++;
	}

	i = 0;
	p = from;
	while (*p && i < 4) {
		while (*p >= '0' && *p <= '9') {
			m[i] = m[i]*10 + (*p - '0');
			p++;
		}
		if (!*p || *p == ':')
			break;
		i++, p++;
	}

	in = *(unsigned int *)m;

	for (i=0 ; i<oracleNumIPFilters ; i++)
		if ( (in & oracleIpFilters[i].mask) == oracleIpFilters[i].compare)
			return oracleFilterBan != 0;

	return oracleFilterBan == 0;
}

/* Test shim: seed the filter list / filterBan, then run G_FilterPacket on `from`.
   `masks` / `compares` are parallel arrays of length `count`. */
qboolean jka_G_FilterPacket( const char *from, int count,
	const unsigned *masks, const unsigned *compares, int filterBan )
{
	int i;

	if ( count > MAX_ORACLE_IPFILTERS )
		count = MAX_ORACLE_IPFILTERS;
	oracleNumIPFilters = count;
	oracleFilterBan = filterBan;
	for ( i = 0 ; i < count ; i++ )
	{
		oracleIpFilters[i].mask = masks[i];
		oracleIpFilters[i].compare = compares[i];
	}

	return G_FilterPacket( (char *)from );
}
