/* Minimal q_shared environment for the oracle: just the math types/macros/unions the
   extracted functions need. Definitions copied verbatim from the original
   raven-jediacademy/codemp/game/q_shared.h, plus `byteAlias_t` from OpenJK's
   shared/qcommon/q_platform.h. We use a minimal shim rather than the real header
   because the real one is clang-hostile (MSVC __asm, a powf(float,int) redecl,
   __LCC__ VM hacks). Function *bodies* in the oracle remain the authentic source. */
#ifndef QSHARED_SHIM_H
#define QSHARED_SHIM_H
#include <math.h>
#include <stdint.h>
#include <assert.h>
#include <string.h>   // memcpy/memset for RotatePointAroundVector

// from q_shared.h — calling-convention tag (empty on non-Windows toolchains).
#define QDECL

typedef float vec_t;
typedef vec_t vec3_t[3];
typedef vec_t vec4_t[4];
typedef int qboolean;
typedef unsigned char byte;
#define qfalse 0
#define qtrue  1

// from q_shared.h — 64-bit int as 8 bytes (qvm-portable), used by Long64Swap.
typedef struct
{
	byte	b0;
	byte	b1;
	byte	b2;
	byte	b3;
	byte	b4;
	byte	b5;
	byte	b6;
	byte	b7;
} qint64;

// from q_shared.h — note the `f` suffix: M_PI is a *float* constant here.
#define M_PI    3.14159265358979323846f
#define PITCH   0   // up / down
#define YAW     1   // left / right
#define ROLL    2   // fall over
#define NUMVERTEXNORMALS	162

// from q_shared.h — string-size limits.
#define MAX_STRING_CHARS	1024
#define MAX_TOKEN_CHARS		1024
#define MAX_INFO_STRING		1024
#define MAX_INFO_KEY		1024
#define MAX_INFO_VALUE		1024
#define BIG_INFO_STRING		8192
#define BIG_INFO_KEY		8192
#define BIG_INFO_VALUE		8192
#define MAX_QPATH			64

// from q_shared.h — all-float angle conversions (NOT OpenJK's precomputed-constant form).
#define DEG2RAD( a ) ( ( (a) * M_PI ) / 180.0F )
#define RAD2DEG( a ) ( ( (a) * 180.0f ) / M_PI )

// from q_shared.h — error levels passed to Com_Error.
typedef enum {
	ERR_FATAL,
	ERR_DROP,
	ERR_SERVERDISCONNECT,
	ERR_DISCONNECT,
	ERR_NEED_CD
} errorParm_t;

// from q_shared.h — color-code escape detection used by Q_PrintStrlen/Q_CleanStr.
#define Q_COLOR_ESCAPE	'^'
#define Q_IsColorString(p)	( p && *(p) == Q_COLOR_ESCAPE && *((p)+1) && *((p)+1) != Q_COLOR_ESCAPE && *((p)+1) <= '7' && *((p)+1) >= '0' )

// from q_shared.h — plane struct used by SetPlaneSignbits / BoxOnPlaneSide.
typedef struct cplane_s {
	vec3_t	normal;
	float	dist;
	byte	type;			// for fast side tests: 0,1,2 = axial, 3 = nonaxial
	byte	signbits;		// signx + (signy<<1) + (signz<<2), used as lookup during collision
	byte	pad[2];
} cplane_t;

// from q_shared.h — row type for the GetIDForString/GetStringForID tables.
typedef struct stringID_table_s
{
	char	*name;
	int		id;
} stringID_table_t;

/* OpenJK's type-pun union (q_platform.h) — the portable fix for float<->int bit
   hacks that replaces the original's UB pointer casts / 64-bit-broken `long`. */
typedef union byteAlias_u {
	float f;
	int32_t i;
	uint32_t ui;
	qboolean qb;
	byte b[4];
	char c[4];
} byteAlias_t;

#define VectorClear(a)          ((a)[0]=(a)[1]=(a)[2]=0)
#define DotProduct(x,y)         ((x)[0]*(y)[0]+(x)[1]*(y)[1]+(x)[2]*(y)[2])
#define VectorSubtract(a,b,c)   ((c)[0]=(a)[0]-(b)[0],(c)[1]=(a)[1]-(b)[1],(c)[2]=(a)[2]-(b)[2])
#define VectorCopy(a,b)         ((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define VectorMA(v,s,b,o)       ((o)[0]=(v)[0]+(b)[0]*(s),(o)[1]=(v)[1]+(b)[1]*(s),(o)[2]=(v)[2]+(b)[2]*(s))

#endif
