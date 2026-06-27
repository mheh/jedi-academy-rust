/* Oracle extraction of bg_misc.c's BG_ParseField (refs/raven-jediacademy/codemp/game/bg_misc.c:358).
 *
 * Two deviations, both output-byte-preserving:
 *   - G_NewString (the F_LSTRING allocator) is substituted with a malloc-backed copy that
 *     performs the same \n escape translation -- identical to oracle/g_spawn_oracle.c, and
 *     for the same reason: the game module's G_Alloc bump pool is engine-trap I/O, but the
 *     OUTPUT BYTES depend only on the escape decode, not the allocator.
 *   - The F_PARM1..F_PARM16 -> Q3_SetParm path is omitted (deferred ICARUS in the Rust port,
 *     so those field types are no-ops on both sides).
 * Q_stricmp / Q_stricmpn are faithful static copies of the q_shared.c originals. atoi/atof/
 * sscanf are the platform libc, exactly what the native game build links (and what the Rust
 * port calls). Renamed `jka_` to avoid colliding with anything in the test binary. */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

typedef unsigned char byte;

typedef enum {
	F_INT,
	F_FLOAT,
	F_LSTRING,
	F_GSTRING,
	F_VECTOR,
	F_ANGLEHACK,
	F_ENTITY,
	F_ITEM,
	F_CLIENT,
	F_PARM1, F_PARM2, F_PARM3, F_PARM4, F_PARM5, F_PARM6, F_PARM7, F_PARM8,
	F_PARM9, F_PARM10, F_PARM11, F_PARM12, F_PARM13, F_PARM14, F_PARM15, F_PARM16,
	F_IGNORE
} fieldtype_t;

typedef struct {
	char		*name;
	int			ofs;
	fieldtype_t	type;
	int			flags;
} BG_field_t;

/* faithful static copy of q_shared.c Q_stricmpn / Q_stricmp */
static int jka_Q_stricmpn (const char *s1, const char *s2, int n) {
	int		c1, c2;

	if ( s1 == NULL ) {
		if ( s2 == NULL )
			return 0;
		else
			return -1;
	}
	else if ( s2==NULL )
		return 1;

	do {
		c1 = *s1++;
		c2 = *s2++;

		if (!n--) {
			return 0;		// strings are equal until end point
		}

		if (c1 != c2) {
			if (c1 >= 'a' && c1 <= 'z') {
				c1 -= ('a' - 'A');
			}
			if (c2 >= 'a' && c2 <= 'z') {
				c2 -= ('a' - 'A');
			}
			if (c1 != c2) {
				return c1 < c2 ? -1 : 1;
			}
		}
	} while (c1);

	return 0;		// strings are equal
}

static int jka_Q_stricmp (const char *s1, const char *s2) {
	return (s1 && s2) ? jka_Q_stricmpn (s1, s2, 99999) : -1;
}

/* G_NewString stand-in (see header comment) */
static char *jka_field_NewString( const char *string ) {
	char	*newb, *new_p;
	int		i,l;

	l = strlen(string) + 1;
	newb = (char *) malloc( l );
	new_p = newb;

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

void jka_BG_ParseField( BG_field_t *l_fields, const char *key, const char *value, byte *ent )
{
	BG_field_t	*f;
	byte	*b;
	float	v;
	float	vec[3];

	for ( f=l_fields ; f->name ; f++ ) {
		if ( !jka_Q_stricmp(f->name, key) ) {
			// found it
			b = (byte *)ent;

			switch( f->type ) {
			case F_LSTRING:
				*(char **)(b+f->ofs) = jka_field_NewString (value);
				break;
			case F_VECTOR:
				sscanf (value, "%f %f %f", &vec[0], &vec[1], &vec[2]);
				((float *)(b+f->ofs))[0] = vec[0];
				((float *)(b+f->ofs))[1] = vec[1];
				((float *)(b+f->ofs))[2] = vec[2];
				break;
			case F_INT:
				*(int *)(b+f->ofs) = atoi(value);
				break;
			case F_FLOAT:
				*(float *)(b+f->ofs) = atof(value);
				break;
			case F_ANGLEHACK:
				v = atof(value);
				((float *)(b+f->ofs))[0] = 0;
				((float *)(b+f->ofs))[1] = v;
				((float *)(b+f->ofs))[2] = 0;
				break;
			/* F_PARM1..F_PARM16 -> Q3_SetParm deferred (omitted; no-op both sides) */
			default:
			case F_IGNORE:
				break;
			}
			return;
		}
	}
}
