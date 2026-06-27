/* Extracted g_timer.c functions, compiled as a parity oracle. See the header in
   q_math_oracle.c for the method. Function *bodies* are the authentic Raven source
   from refs/raven-jediacademy/codemp/game/g_timer.c, verbatim except for the documented
   ORACLE DEVIATIONs below.

   ORACLE DEVIATIONS:
   - The pool/struct/globals are reproduced here verbatim (g_timer.c:8-19) under a
     `jka_` prefix so they do not collide with any host symbol.
   - g_timer.c reads two pieces of host context that don't exist in this isolated TU:
     `ent->s.number` (the entity index key) and `level.time` (the clock). Rather than
     drag in g_local.h, this oracle uses a minimal `jka_gentity_t` whose only field is
     `s.number`, and a file-local `jka_level_time` int set by `jka_TIMER_oracle_reset`.
     TIMER_Set/TIMER_Done bodies are otherwise verbatim.
   - Q_stricmp is reproduced as a tiny `jka_Q_stricmp` (case-insensitive compare) — the
     only string op the timer lookup needs; its result-equality with q_shared.c's
     Q_stricmp on the ASCII keys used by tests is all that matters here. */

typedef int qboolean;
#define qfalse 0
#define qtrue  1

#define MAX_GTIMERS	16384
#define MAX_GENTITIES	1024

typedef struct jka_gtimer_s
{
	const char *name;
	int time;
	struct jka_gtimer_s *next;	// In either free list or current list
} jka_gtimer_t;

jka_gtimer_t jka_g_timerPool[ MAX_GTIMERS ];
jka_gtimer_t *jka_g_timers[ MAX_GENTITIES ];
jka_gtimer_t *jka_g_timerFreeList;

/* minimal host-context proxy (see ORACLE DEVIATIONS) */
typedef struct { struct { int number; } s; } jka_gentity_t;
static int jka_level_time;

/* case-insensitive compare, sufficient for the ASCII timer keys (see ORACLE DEVIATIONS) */
static int jka_tolower(int c) { return (c >= 'A' && c <= 'Z') ? c + ('a' - 'A') : c; }
static int jka_Q_stricmp(const char *s1, const char *s2)
{
	int c1, c2;
	do {
		c1 = jka_tolower(*s1++);
		c2 = jka_tolower(*s2++);
		if (c1 != c2)
			return c1 < c2 ? -1 : 1;
	} while (c1);
	return 0;
}

/* g_timer.c:27 — TIMER_Clear (verbatim). */
void jka_TIMER_Clear( void )
{
	int i;
	for (i = 0; i < MAX_GENTITIES; i++)
	{
		jka_g_timers[i] = 0;
	}

	for (i = 0; i < MAX_GTIMERS - 1; i++)
	{
		jka_g_timerPool[i].next = &jka_g_timerPool[i+1];
	}
	jka_g_timerPool[MAX_GTIMERS-1].next = 0;
	jka_g_timerFreeList = &jka_g_timerPool[0];
}

/* g_timer.c:79 — TIMER_GetNew (verbatim, Q_stricmp routed to jka copy). */
jka_gtimer_t *jka_TIMER_GetNew(int num, const char *identifier)
{
	jka_gtimer_t *p = jka_g_timers[num];

	// Search for an existing timer with this name
	while (p)
	{
		if (!jka_Q_stricmp(p->name, identifier))
		{ // Found it
			return p;
		}

		p = p->next;
	}

	// No existing timer with this name was found, so grab one from the free list
	if (!jka_g_timerFreeList)
		return 0;

	p = jka_g_timerFreeList;
	jka_g_timerFreeList = jka_g_timerFreeList->next;
	p->next = jka_g_timers[num];
	jka_g_timers[num] = p;
	return p;
}

/* g_timer.c:106 — TIMER_GetExisting (verbatim, Q_stricmp routed to jka copy). */
jka_gtimer_t *jka_TIMER_GetExisting(int num, const char *identifier)
{
	jka_gtimer_t *p = jka_g_timers[num];

	while (p)
	{
		if (!jka_Q_stricmp(p->name, identifier))
		{ // Found it
			return p;
		}

		p = p->next;
	}

	return 0;
}

/* g_timer.c:129 — TIMER_Set (verbatim; level.time -> jka_level_time). */
void jka_TIMER_Set( jka_gentity_t *ent, const char *identifier, int duration )
{
	jka_gtimer_t *timer = jka_TIMER_GetNew(ent->s.number, identifier);

	if (!timer)
	{
		return;
	}
	timer->name = identifier;
	timer->time = jka_level_time + duration;
}

/* g_timer.c:165 — TIMER_Done (verbatim; level.time -> jka_level_time). */
qboolean jka_TIMER_Done( jka_gentity_t *ent, const char *identifier )
{
	jka_gtimer_t *timer = jka_TIMER_GetExisting(ent->s.number, identifier);

	if (!timer)
	{
		return qtrue;
	}

	return (timer->time < jka_level_time);
}

/* ---- test harness shims (not from g_timer.c) ---- */

/* Reset the pool to a clean state and set the clock. Tests call this before each
   scenario so the oracle and the Rust port start from identical state. */
void jka_TIMER_oracle_reset( int level_time )
{
	jka_level_time = level_time;
	jka_TIMER_Clear();
}

void jka_TIMER_oracle_set_time( int level_time )
{
	jka_level_time = level_time;
}

/* Thin wrappers taking the entity index directly, so the Rust test need not build a
   gentity_t mirror just for the oracle side. */
void jka_TIMER_Set_idx( int num, const char *identifier, int duration )
{
	jka_gentity_t e; e.s.number = num;
	jka_TIMER_Set( &e, identifier, duration );
}

qboolean jka_TIMER_Done_idx( int num, const char *identifier )
{
	jka_gentity_t e; e.s.number = num;
	return jka_TIMER_Done( &e, identifier );
}
