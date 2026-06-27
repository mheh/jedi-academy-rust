/* Oracle: functions extracted verbatim from the original
   raven-jediacademy/codemp/game/bg_lib.c, compiled and linked under the `oracle` cargo
   feature (see build.rs) so Rust tests can call the real C and assert the Rust port
   in src/codemp/game/bg_lib.rs matches it bit-for-bit.

   ORACLE DEVIATION — names: every function here whose name collides with the test
   binary's own libc (rand, srand, qsort, atof, memmove, ...) is prefixed `jka_`
   (cf. jka_powf in q_math_oracle.c). The bodies are the authentic Raven bg_lib.c.

   BUILD NOTE: this TU is compiled `-fwrapv` (see build.rs) so signed-int overflow
   is defined to wrap as two's-complement — matching the Rust port's wrapping_mul/
   wrapping_add. The rand() LCG below relies on that wrap. */

#include <stddef.h> /* size_t (qsort) */
#include <math.h>   /* sin, cos (jka_tan) */
#include <stdarg.h> /* va_list/va_arg (jka_vsprintf harness) */

/* ===== Group A: always-compiled (ungated) — overrides libc on native, so this is
   the game's real behavior on every target. ===== */

static int randSeed = 0;

void jka_srand( unsigned seed ) {
	randSeed = seed;
}

int jka_rand( void ) {
	randSeed = (69069 * randSeed + 1);
	return randSeed & 0x7fff;
}

/* ===== qsort: Bentley & McIlroy, "Engineering a Sort Function" (bg_lib.c). Kept
   verbatim WITH the full swaptype/SWAPINIT/swap/vecswap/swapcode copy-width machinery,
   so the Rust port's byte-wise-swap collapse is parity-checked against the authentic
   width logic. ORACLE DEVIATION (names): qsort -> jka_qsort (libc collision); the
   recursive call is jka_qsort too. The file-local statics med3/swapfunc and the macros
   keep their names (no external linkage -> no collision). ===== */

/* bk001127 - needed for DLL's */
#if !defined( Q3_VM )
typedef int		 cmp_t(const void *, const void *);
#endif

static char* med3(char *, char *, char *, cmp_t *);
static void	 swapfunc(char *, char *, int, int);

#ifndef min
#define min(a, b)	(a) < (b) ? a : b
#endif

/*
 * Qsort routine from Bentley & McIlroy's "Engineering a Sort Function".
 */
#define swapcode(TYPE, parmi, parmj, n) { 		\
	long i = (n) / sizeof (TYPE); 			\
	register TYPE *pi = (TYPE *) (parmi); 		\
	register TYPE *pj = (TYPE *) (parmj); 		\
	do { 						\
		register TYPE	t = *pi;		\
		*pi++ = *pj;				\
		*pj++ = t;				\
        } while (--i > 0);				\
}

#define SWAPINIT(a, es) swaptype = ((char *)a - (char *)0) % sizeof(long) || \
	es % sizeof(long) ? 2 : es == sizeof(long)? 0 : 1;

static void swapfunc( char* a, char* b, int n, int swaptype)
{
	if(swaptype <= 1)
		swapcode(long, a, b, n)
	else
		swapcode(char, a, b, n)
}

#define swap(a, b)					\
	if (swaptype == 0) {				\
		long t = *(long *)(a);			\
		*(long *)(a) = *(long *)(b);		\
		*(long *)(b) = t;			\
	} else						\
		swapfunc(a, b, es, swaptype)

#define vecswap(a, b, n) 	if ((n) > 0) swapfunc(a, b, n, swaptype)

static char *med3(char* a, char* b, char* c, cmp_t* cmp)
{
	return cmp(a, b) < 0 ?
	       (cmp(b, c) < 0 ? b : (cmp(a, c) < 0 ? c : a ))
              :(cmp(b, c) > 0 ? b : (cmp(a, c) < 0 ? a : c ));
}

void jka_qsort( void* a, size_t n, size_t es, cmp_t* cmp)
{
	char *pa, *pb, *pc, *pd, *pl, *pm, *pn;
	int d, r, swaptype, swap_cnt;

loop:	SWAPINIT(a, es);
	swap_cnt = 0;
	if (n < 7) {
		for (pm = (char *)a + es; pm < (char *)a + n * es; pm += es)
			for (pl = pm; pl > (char *)a && cmp(pl - es, pl) > 0;
			     pl -= es)
				swap(pl, pl - es);
		return;
	}
	pm = (char *)a + (n / 2) * es;
	if (n > 7) {
		pl = a;
		pn = (char *)a + (n - 1) * es;
		if (n > 40) {
			d = (n / 8) * es;
			pl = med3(pl, pl + d, pl + 2 * d, cmp);
			pm = med3(pm - d, pm, pm + d, cmp);
			pn = med3(pn - 2 * d, pn - d, pn, cmp);
		}
		pm = med3(pl, pm, pn, cmp);
	}
	swap(a, pm);
	pa = pb = (char *)a + es;

	pc = pd = (char *)a + (n - 1) * es;
	for (;;) {
		while (pb <= pc && (r = cmp(pb, a)) <= 0) {
			if (r == 0) {
				swap_cnt = 1;
				swap(pa, pb);
				pa += es;
			}
			pb += es;
		}
		while (pb <= pc && (r = cmp(pc, a)) >= 0) {
			if (r == 0) {
				swap_cnt = 1;
				swap(pc, pd);
				pd -= es;
			}
			pc -= es;
		}
		if (pb > pc)
			break;
		swap(pb, pc);
		swap_cnt = 1;
		pb += es;
		pc -= es;
	}
	if (swap_cnt == 0) {  /* Switch to insertion sort */
		for (pm = (char *)a + es; pm < (char *)a + n * es; pm += es)
			for (pl = pm; pl > (char *)a && cmp(pl - es, pl) > 0;
			     pl -= es)
				swap(pl, pl - es);
		return;
	}

	pn = (char *)a + n * es;
	r = min(pa - (char *)a, pb - pa);
	vecswap(a, pb - r, r);
	r = min(pd - pc, pn - pd - es);
	vecswap(pb, pn - r, r);
	if ((r = pb - pa) > es)
		jka_qsort(a, r / es, es, cmp);
	if ((r = pd - pc) > es) {
		/* Iterate rather than recurse to save stack space */
		a = pn - r;
		n = r / es;
		goto loop;
	}
/*		qsort(pn - r, r / es, es, cmp);*/
}

/* ===== atof / _atof: JKA's own string->float parser (bg_lib.c ~774-907). NOT libc
   atof -- no 10e10 exponent notation; whitespace is `*string <= ' '` (read with the
   platform char signedness); the int part accumulates in float while the fractional
   part adds through a double `fraction` and stores back to float. Kept verbatim so
   the Rust port's per-step float/double widths are parity-checked. ORACLE DEVIATION
   (names): atof -> jka_atof, _atof -> jka__atof (libc collision). ===== */

double jka_atof( const char *string ) {
	float sign;
	float value;
	int		c;


	// skip whitespace
	while ( *string <= ' ' ) {
		if ( !*string ) {
			return 0;
		}
		string++;
	}

	// check sign
	switch ( *string ) {
	case '+':
		string++;
		sign = 1;
		break;
	case '-':
		string++;
		sign = -1;
		break;
	default:
		sign = 1;
		break;
	}

	// read digits
	value = 0;
	c = string[0];
	if ( c != '.' ) {
		do {
			c = *string++;
			if ( c < '0' || c > '9' ) {
				break;
			}
			c -= '0';
			value = value * 10 + c;
		} while ( 1 );
	} else {
		string++;
	}

	// check for decimal point
	if ( c == '.' ) {
		double fraction;

		fraction = 0.1;
		do {
			c = *string++;
			if ( c < '0' || c > '9' ) {
				break;
			}
			c -= '0';
			value += c * fraction;
			fraction *= 0.1;
		} while ( 1 );

	}

	// not handling 10e10 notation...

	return value * sign;
}

double jka__atof( const char **stringPtr ) {
	const char	*string;
	float sign;
	float value;
	int		c = '0'; // bk001211 - uninitialized use possible

	string = *stringPtr;

	// skip whitespace
	while ( *string <= ' ' ) {
		if ( !*string ) {
			*stringPtr = string;
			return 0;
		}
		string++;
	}

	// check sign
	switch ( *string ) {
	case '+':
		string++;
		sign = 1;
		break;
	case '-':
		string++;
		sign = -1;
		break;
	default:
		sign = 1;
		break;
	}

	// read digits
	value = 0;
	if ( string[0] != '.' ) {
		do {
			c = *string++;
			if ( c < '0' || c > '9' ) {
				break;
			}
			c -= '0';
			value = value * 10 + c;
		} while ( 1 );
	}

	// check for decimal point
	if ( c == '.' ) {
		double fraction;

		fraction = 0.1;
		do {
			c = *string++;
			if ( c < '0' || c > '9' ) {
				break;
			}
			c -= '0';
			value += c * fraction;
			fraction *= 0.1;
		} while ( 1 );

	}

	// not handling 10e10 notation...
	*stringPtr = string;

	return value * sign;
}

/* ===== memmove: overlap-safe byte copy (bg_lib.c ~287-300), ungated so it overrides
   libc on native. Verbatim. ORACLE DEVIATION (names): memmove -> jka_memmove. ===== */

void *jka_memmove( void *dest, const void *src, size_t count ) {
	int		i;

	if ( dest > src ) {
		for ( i = count-1 ; i >= 0 ; i-- ) {
			((char *)dest)[i] = ((char *)src)[i];
		}
	} else {
		for ( i = 0 ; i < count ; i++ ) {
			((char *)dest)[i] = ((char *)src)[i];
		}
	}
	return dest;
}

/* ===== Group B: `#if defined( Q3_VM )` in bg_lib.c -- the no-libc shims. The Rust
   side gates these behind the `vm` cargo feature; here in the oracle they are extracted
   UNGATED (we never define Q3_VM) so `cargo test --features "oracle vm"` can compare the
   Rust shim against the authentic C. ORACLE DEVIATION (names): every libc-colliding name
   (strlen, strcat, ...) is `jka_`-prefixed; non-colliding JKA originals (AddInt, ...) are
   prefixed too, for uniformity. (`jka_vsprintf` lives at the file tail -- a variadic test
   harness around the verbatim format-state machine; see DEVIATIONS.md.) ===== */

/* ----- string routines (bg_lib.c ~191-262) ----- */

size_t jka_strlen( const char *string ) {
	const char	*s;

	s = string;
	while ( *s ) {
		s++;
	}
	return s - string;
}

char *jka_strcat( char *strDestination, const char *strSource ) {
	char	*s;

	s = strDestination;
	while ( *s ) {
		s++;
	}
	while ( *strSource ) {
		*s++ = *strSource++;
	}
	*s = 0;
	return strDestination;
}

char *jka_strcpy( char *strDestination, const char *strSource ) {
	char *s;

	s = strDestination;
	while ( *strSource ) {
		*s++ = *strSource++;
	}
	*s = 0;
	return strDestination;
}

int jka_strcmp( const char *string1, const char *string2 ) {
	while ( *string1 == *string2 && *string1 && *string2 ) {
		string1++;
		string2++;
	}
	return *string1 - *string2;
}

char *jka_strchr( const char *string, int c ) {
	while ( *string ) {
		if ( *string == c ) {
			return ( char * )string;
		}
		string++;
	}
	return (char *)0;
}

char *jka_strstr( const char *string, const char *strCharSet ) {
	while ( *string ) {
		int		i;

		for ( i = 0 ; strCharSet[i] ; i++ ) {
			if ( string[i] != strCharSet[i] ) {
				break;
			}
		}
		if ( !strCharSet[i] ) {
			return (char *)string;
		}
		string++;
	}
	return (char *)0;
}

/* ----- ctype (bg_lib.c ~269-282) ----- */

int jka_tolower( int c ) {
	if ( c >= 'A' && c <= 'Z' ) {
		c += 'a' - 'A';
	}
	return c;
}

int jka_toupper( int c ) {
	if ( c >= 'a' && c <= 'z' ) {
		c += 'A' - 'a';
	}
	return c;
}

/* ----- abs / fabs (bg_lib.c ~1010-1016). -fwrapv (build.rs) defines abs(INT_MIN). ----- */

int jka_abs( int n ) {
	return n < 0 ? -n : n;
}

double jka_fabs( double x ) {
	return x < 0 ? -x : x;
}

/* ----- tan (bg_lib.c ~754-760). sin/cos are runtime-provided in a VM build; here they
   resolve to libm, matching the Rust shim's extern sin/cos. ----- */

double jka_tan( double x ) {
	return sin(x) / cos(x);
}

/* ----- atoi / _atoi (bg_lib.c ~915-1008). -fwrapv defines the value*10+c overflow. ----- */

int jka_atoi( const char *string ) {
	int		sign;
	int		value;
	int		c;

	// skip whitespace
	while ( *string <= ' ' ) {
		if ( !*string ) {
			return 0;
		}
		string++;
	}

	// check sign
	switch ( *string ) {
	case '+':
		string++;
		sign = 1;
		break;
	case '-':
		string++;
		sign = -1;
		break;
	default:
		sign = 1;
		break;
	}

	// read digits
	value = 0;
	do {
		c = *string++;
		if ( c < '0' || c > '9' ) {
			break;
		}
		c -= '0';
		value = value * 10 + c;
	} while ( 1 );

	// not handling 10e10 notation...

	return value * sign;
}

int jka__atoi( const char **stringPtr ) {
	int		sign;
	int		value;
	int		c;
	const char	*string;

	string = *stringPtr;

	// skip whitespace
	while ( *string <= ' ' ) {
		if ( !*string ) {
			return 0;
		}
		string++;
	}

	// check sign
	switch ( *string ) {
	case '+':
		string++;
		sign = 1;
		break;
	case '-':
		string++;
		sign = -1;
		break;
	default:
		sign = 1;
		break;
	}

	// read digits
	value = 0;
	do {
		c = *string++;
		if ( c < '0' || c > '9' ) {
			break;
		}
		c -= '0';
		value = value * 10 + c;
	} while ( 1 );

	// not handling 10e10 notation...

	*stringPtr = string;

	return value * sign;
}

/* ----- printf helpers (bg_lib.c ~1023-1173). The format-flag #defines are the verbatim
   source block; only LADJUST/ZEROPAD are consulted by jka_AddInt and jka_vsprintf. (sscanf
   uses none of the flag set; the rest of the bits go unused -- see DEVIATIONS.md.) ----- */

#define ALT			0x00000001		/* alternate form */
#define HEXPREFIX	0x00000002		/* add 0x or 0X prefix */
#define LADJUST		0x00000004		/* left adjustment */
#define LONGDBL		0x00000008		/* long double */
#define LONGINT		0x00000010		/* long integer */
#define QUADINT		0x00000020		/* quad integer */
#define SHORTINT	0x00000040		/* short integer */
#define ZEROPAD		0x00000080		/* zero (as opposed to blank) pad */
#define FPT			0x00000100		/* floating point number */

void jka_AddInt( char **buf_p, int val, int width, int flags ) {
	char	text[32];
	int		digits;
	int		signedVal;
	char	*buf;

	digits = 0;
	signedVal = val;
	if ( val < 0 ) {
		val = -val;
	}
	do {
		text[digits++] = '0' + val % 10;
		val /= 10;
	} while ( val );

	if ( signedVal < 0 ) {
		text[digits++] = '-';
	}

	buf = *buf_p;

	if( !( flags & LADJUST ) ) {
		while ( digits < width ) {
			*buf++ = ( flags & ZEROPAD ) ? '0' : ' ';
			width--;
		}
	}

	while ( digits-- ) {
		*buf++ = text[digits];
		width--;
	}

	if( flags & LADJUST ) {
		while ( width-- ) {
			*buf++ = ( flags & ZEROPAD ) ? '0' : ' ';
		}
	}

	*buf_p = buf;
}

void jka_AddFloat( char **buf_p, float fval, int width, int prec ) {
	char	text[32];
	int		digits;
	float	signedVal;
	char	*buf;
	int		val;

	// get the sign
	signedVal = fval;
	if ( fval < 0 ) {
		fval = -fval;
	}

	// write the float number
	digits = 0;
	val = (int)fval;
	do {
		text[digits++] = '0' + val % 10;
		val /= 10;
	} while ( val );

	if ( signedVal < 0 ) {
		text[digits++] = '-';
	}

	buf = *buf_p;

	while ( digits < width ) {
		*buf++ = ' ';
		width--;
	}

	while ( digits-- ) {
		*buf++ = text[digits];
	}

	*buf_p = buf;

	if (prec < 0)
		prec = 6;
	// write the fraction
	digits = 0;
	while (digits < prec) {
		fval -= (int) fval;
		fval *= 10.0;
		val = (int) fval;
		text[digits++] = '0' + val % 10;
	}

	if (digits > 0) {
		buf = *buf_p;
		*buf++ = '.';
		for (prec = 0; prec < digits; prec++) {
			*buf++ = text[prec];
		}
		*buf_p = buf;
	}
}

void jka_AddString( char **buf_p, char *string, int width, int prec ) {
	int		size;
	char	*buf;

	buf = *buf_p;

	if ( string == NULL ) {
		string = "(null)";
		prec = -1;
	}

	if ( prec >= 0 ) {
		for( size = 0; size < prec; size++ ) {
			if( string[size] == '\0' ) {
				break;
			}
		}
	}
	else {
		size = jka_strlen( string );	/* ORACLE DEVIATION (name): strlen -> jka_strlen */
	}

	width -= size;

	while( size-- ) {
		*buf++ = *string++;
	}

	while( width-- > 0 ) {
		*buf++ = ' ';
	}

	*buf_p = buf;
}

/* ----- vsprintf (bg_lib.c ~1183). The format-state machine + the jka_AddInt/jka_AddFloat/
   jka_AddString dispatch are VERBATIM. ORACLE DEVIATION (mechanism): the authentic body does
   `arg = (int *)argptr; ... *arg ...; arg++`, walking the va_list as a packed int array -- a
   32-bit-VM trick that is *broken* on this 64-bit native oracle (varargs are not packed after
   `fmt`). So this harness is variadic and fetches each argument with `va_arg`, the
   64-bit-correct equivalent -- exactly the deviation the Rust port documents (it takes a typed
   `VsArg` slice). `to_digit`/`is_digit` are the verbatim bg_lib.c ~1033 macros; LADJUST/ZEROPAD
   are #defined with the printf helpers above. Mirrors `#if defined(Q3_VM)` source. ----- */
#define to_digit(c)		((c) - '0')
#define is_digit(c)		((unsigned)to_digit(c) <= 9)

int jka_vsprintf( char *buffer, const char *fmt, ... ) {
	va_list	argptr;
	char	*buf_p;
	char	ch;
	int		flags;
	int		width;
	int		prec;
	int		n;
	char	sign;

	va_start( argptr, fmt );
	buf_p = buffer;

	while( 1 ) {
		/* run through the format string until we hit a '%' or '\0' */
		for ( ch = *fmt; (ch = *fmt) != '\0' && ch != '%'; fmt++ ) {
			*buf_p++ = ch;
		}
		if ( ch == '\0' ) {
			goto done;
		}

		/* skip over the '%' */
		fmt++;

		/* reset formatting state */
		flags = 0;
		width = 0;
		prec = -1;
		sign = '\0';
		(void)sign; /* set-but-unused in the authentic source too */

rflag:
		ch = *fmt++;
reswitch:
		switch( ch ) {
		case '-':
			flags |= LADJUST;
			goto rflag;
		case '.':
			n = 0;
			while( is_digit( ( ch = *fmt++ ) ) ) {
				n = 10 * n + ( ch - '0' );
			}
			prec = n < 0 ? -1 : n;
			goto reswitch;
		case '0':
			flags |= ZEROPAD;
			goto rflag;
		case '1':
		case '2':
		case '3':
		case '4':
		case '5':
		case '6':
		case '7':
		case '8':
		case '9':
			n = 0;
			do {
				n = 10 * n + ( ch - '0' );
				ch = *fmt++;
			} while( is_digit( ch ) );
			width = n;
			goto reswitch;
		case 'c':
			*buf_p++ = (char)va_arg( argptr, int );
			break;
		case 'd':
		case 'i':
			jka_AddInt( &buf_p, va_arg( argptr, int ), width, flags );
			break;
		case 'f':
			jka_AddFloat( &buf_p, (float)va_arg( argptr, double ), width, prec );
			break;
		case 's':
			jka_AddString( &buf_p, va_arg( argptr, char * ), width, prec );
			break;
		case '%':
			*buf_p++ = ch;
			break;
		default:
			*buf_p++ = (char)va_arg( argptr, int );
			break;
		}
	}

done:
	*buf_p = 0;
	va_end( argptr );
	return buf_p - buffer;
}
