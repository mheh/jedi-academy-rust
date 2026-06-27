/*
 * Oracle TU for g_trigger.c pure helpers. Holds functions transcribed VERBATIM
 * from raven-jediacademy/codemp/game/g_trigger.c that are computable in isolation (no
 * traps, no game globals), so `src/codemp/game/g_trigger.rs` can parity-check its
 * port bit-for-bit.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

/* Q_stricmp lives in q_shared_oracle.c (same static lib); declare and reuse it
   rather than re-defining (would collide at link time). */
int Q_stricmp(const char *s1, const char *s2);

#define MAX_STRING_CHARS 1024 /* q_shared.h:58 */

typedef int qboolean; /* qfalse=0, qtrue=1 (q_shared.h) */
#define qfalse 0
#define qtrue 1

/* ---- G_NameInTriggerClassList (g_trigger.c:97) ---------------------------
 * Determine if the class given is listed in the string using the | formatting.
 * Verbatim (renamed jka_ to dodge any host symbol). */
qboolean jka_G_NameInTriggerClassList(char *list, char *str)
{
	char cmp[MAX_STRING_CHARS];
	int i = 0;
	int j;

	while (list[i])
	{
        j = 0;
        while (list[i] && list[i] != '|')
		{
			cmp[j] = list[i];
			i++;
			j++;
		}
		cmp[j] = 0;

		if (!Q_stricmp(str, cmp))
		{ /* found it */
			return qtrue;
		}
		if (list[i] != '|')
		{ /* reached the end and never found it */
			return qfalse;
		}
		i++;
	}

	return qfalse;
}
