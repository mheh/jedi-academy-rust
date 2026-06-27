/*
 * Oracle TU for g_cmds.c pure helpers. Holds functions transcribed VERBATIM from
 * raven-jediacademy/codemp/game/g_cmds.c that are computable in isolation (no traps,
 * no game globals), so `src/codemp/game/g_cmds.rs` can parity-check its port
 * bit-for-bit.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

#include <ctype.h> /* tolower */

/* ---- SanitizeString (g_cmds.c:161) ---------------------------------------
 * Remove case and control characters. Verbatim. */
void jka_SanitizeString(char *in, char *out) {
	while (*in) {
		if (*in == 27) {
			in += 2; /* skip color code */
			continue;
		}
		if (*in < 32) {
			in++;
			continue;
		}
		*out++ = tolower((unsigned char)*in++);
	}

	*out = 0;
}

/* ---- SanitizeString2 (g_cmds.c:1846) -------------------------------------
 * Rich's revised version of SanitizeString. Verbatim (MAX_NAME_LENGTH from
 * q_shared.h:404). */
#define MAX_NAME_LENGTH 32 /* max length of a client name (q_shared.h:404) */

void jka_SanitizeString2(char *in, char *out) {
	int i = 0;
	int r = 0;

	while (in[i]) {
		if (i >= MAX_NAME_LENGTH - 1) { /* the ui truncates the name here.. */
			break;
		}

		if (in[i] == '^') {
			if (in[i + 1] >= 48 && /* '0' */
			    in[i + 1] <= 57) { /* '9' */
				/* only skip it if there's a number after it for the color */
				i += 2;
				continue;
			} else { /* just skip the ^ */
				i++;
				continue;
			}
		}

		if (in[i] < 32) {
			i++;
			continue;
		}

		out[r] = in[i];
		r++;
		i++;
	}
	out[r] = 0;
}
