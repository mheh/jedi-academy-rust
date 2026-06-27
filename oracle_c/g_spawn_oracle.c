/* Oracle extraction of g_spawn.c's G_NewString (refs/raven-jediacademy/codemp/game/g_spawn.c:716).
 *
 * The only deviation from the original is the backing allocator: the game module's
 * G_Alloc is a fixed-pool bump allocator (g_mem.c) that prints/errors through engine
 * traps and is itself a "no oracle" function. The OUTPUT BYTES are allocator-independent
 * -- they depend only on the \n escape translation -- so this oracle substitutes malloc
 * for G_Alloc and the Rust port's output is compared byte-for-byte. Renamed `jka_` to
 * avoid colliding with anything in the test binary. */

#include <stdlib.h>
#include <string.h>

char *jka_G_NewString( const char *string ) {
	char	*newb, *new_p;
	int		i,l;

	l = strlen(string) + 1;

	newb = (char *) malloc( l );

	new_p = newb;

	// turn \n into a real linefeed
	for ( i=0 ; i< l ; i++ ) {
		if (string[i] == '\\' && i < l-1) {
			i++;
			if (string[i] == 'n') {
				*new_p++ = '\n';
			} else {
				*new_p++ = '\\';
			}
		} else {
			*new_p++ = string[i];
		}
	}

	return newb;
}
