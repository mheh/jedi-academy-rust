/* Oracle extraction of bg_vehicleLoad.c's BG_ParseVehWeaponParm
 * (refs/raven-jediacademy/codemp/game/bg_vehicleLoad.c:176), the retail QAGAME build.
 *
 * BG_ParseVehWeaponParm reads the *global* vehWeaponFields[] table (it is NOT passed
 * a table like BG_ParseField), so the oracle transcribes both the real vehWeaponInfo_t
 * struct and the real vehWeaponFields[] table verbatim, then both sides parse the same
 * parmName/value stream into their own vehWeaponInfo_t and compare bytewise (the name
 * F_LSTRING pointer compared by pointed-to content).
 *
 * Deviations, all output-byte-preserving:
 *   - BG_Alloc (the VF_LSTRING allocator) -> malloc-backed copy. The OUTPUT BYTES depend
 *     only on the strcpy of `value`, not the bump-pool address (cf. bg_misc_parsefield).
 *   - The QAGAME no-op arms (VF_WEAPON, the _CLIENT variants, VF_SHADER/VF_SHADER_NOMIP)
 *     do nothing on both sides; the VF_MODEL/VF_EFFECT/VF_SOUND/VF_VEHTYPE/VF_ANIM arms
 *     (engine configstrings / GetIDForString) are kept in the switch but never reached by
 *     the vehWeaponFields[] table, so they need no engine stand-in.
 * Q_stricmp / Q_strncpyz are faithful static copies of the q_shared.c originals. atoi/atof/
 * sscanf are the platform libc, exactly what the native game build links (and what the Rust
 * port calls). Renamed `jka_` to avoid colliding with anything in the test binary. */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stddef.h>

typedef unsigned char byte;
typedef int qboolean;
#define qfalse 0
#define qtrue 1

/* --- verbatim vehWeaponInfo_t (bg_vehicles.h) --------------------------------------- */
typedef struct vehWeaponInfo_s {
	char		*name;
	qboolean	bIsProjectile;
	qboolean	bHasGravity;
	qboolean	bIonWeapon;
	qboolean	bSaberBlockable;
	int			iMuzzleFX;
	int			iModel;
	int			iShotFX;
	int			iImpactFX;
	int			iG2MarkShaderHandle;
	float		fG2MarkSize;
	int			iLoopSound;
	float		fSpeed;
	float		fHoming;
	float		fHomingFOV;
	int			iLockOnTime;
	int			iDamage;
	int			iSplashDamage;
	float		fSplashRadius;
	int			iAmmoPerShot;
	int			iHealth;
	float		fWidth;
	float		fHeight;
	int			iLifeTime;
	qboolean	bExplodeOnExpire;
} vehWeaponInfo_t;

#define NUM_VWEAP_PARMS 25
#define VWFOFS(x) offsetof(vehWeaponInfo_t, x)

typedef enum {
	VF_IGNORE,
	VF_INT,
	VF_FLOAT,
	VF_LSTRING,
	VF_VECTOR,
	VF_BOOL,
	VF_VEHTYPE,
	VF_ANIM,
	VF_WEAPON,
	VF_MODEL,
	VF_MODEL_CLIENT,
	VF_EFFECT,
	VF_EFFECT_CLIENT,
	VF_SHADER,
	VF_SHADER_NOMIP,
	VF_SOUND,
	VF_SOUND_CLIENT
} vehFieldType_t;

typedef struct {
	char			*name;
	int				ofs;
	vehFieldType_t	type;
} vehField_t;

static vehField_t jka_vehWeaponFields[NUM_VWEAP_PARMS] =
{
	{"name", VWFOFS(name), VF_LSTRING},
	{"projectile", VWFOFS(bIsProjectile), VF_BOOL},
	{"hasGravity", VWFOFS(bHasGravity), VF_BOOL},
	{"ionWeapon", VWFOFS(bIonWeapon), VF_BOOL},
	{"saberBlockable", VWFOFS(bSaberBlockable), VF_BOOL},
	{"muzzleFX", VWFOFS(iMuzzleFX), VF_EFFECT_CLIENT},
	{"model", VWFOFS(iModel), VF_MODEL_CLIENT},
	{"shotFX", VWFOFS(iShotFX), VF_EFFECT_CLIENT},
	{"impactFX", VWFOFS(iImpactFX), VF_EFFECT_CLIENT},
	{"g2MarkShader", VWFOFS(iG2MarkShaderHandle), VF_SHADER},
	{"g2MarkSize", VWFOFS(fG2MarkSize), VF_FLOAT},
	{"loopSound", VWFOFS(iLoopSound), VF_SOUND_CLIENT},
	{"speed", VWFOFS(fSpeed), VF_FLOAT},
	{"homing", VWFOFS(fHoming), VF_FLOAT},
	{"homingFOV", VWFOFS(fHomingFOV), VF_FLOAT},
	{"lockOnTime", VWFOFS(iLockOnTime), VF_INT},
	{"damage", VWFOFS(iDamage), VF_INT},
	{"splashDamage", VWFOFS(iSplashDamage), VF_INT},
	{"splashRadius", VWFOFS(fSplashRadius), VF_FLOAT},
	{"ammoPerShot", VWFOFS(iAmmoPerShot), VF_INT},
	{"health", VWFOFS(iHealth), VF_INT},
	{"width", VWFOFS(fWidth), VF_FLOAT},
	{"height", VWFOFS(fHeight), VF_FLOAT},
	{"lifetime", VWFOFS(iLifeTime), VF_INT},
	{"explodeOnExpire", VWFOFS(bExplodeOnExpire), VF_BOOL},
};

/* faithful static copy of q_shared.c Q_stricmpn / Q_stricmp */
static int jka_Q_stricmpn (const char *s1, const char *s2, int n) {
	int		c1, c2;
	if ( s1 == NULL ) { return ( s2 == NULL ) ? 0 : -1; }
	else if ( s2==NULL ) return 1;
	do {
		c1 = *s1++; c2 = *s2++;
		if (!n--) return 0;
		if (c1 != c2) {
			if (c1 >= 'a' && c1 <= 'z') c1 -= ('a' - 'A');
			if (c2 >= 'a' && c2 <= 'z') c2 -= ('a' - 'A');
			if (c1 != c2) return c1 < c2 ? -1 : 1;
		}
	} while (c1);
	return 0;
}
static int jka_Q_stricmp (const char *s1, const char *s2) {
	return (s1 && s2) ? jka_Q_stricmpn (s1, s2, 99999) : -1;
}

/* faithful static copy of q_shared.c Q_strncpyz */
static void jka_Q_strncpyz( char *dest, const char *src, int destsize ) {
	if ( !dest || !src || destsize < 1 ) return;
	strncpy( dest, src, destsize-1 );
	dest[destsize-1] = 0;
}

/* BG_Alloc stand-in (see header comment) */
static void *jka_BG_Alloc( int size ) { return malloc( size ); }

qboolean jka_BG_ParseVehWeaponParm( vehWeaponInfo_t *vehWeapon, char *parmName, char *pValue )
{
	int		i;
	float	vec[3];
	byte	*b = (byte *)vehWeapon;
	int		_iFieldsRead = 0;
	char	value[1024];

	jka_Q_strncpyz( value, pValue, sizeof(value) );

	for ( i = 0; i < NUM_VWEAP_PARMS; i++ )
	{
		if ( jka_vehWeaponFields[i].name && !jka_Q_stricmp( jka_vehWeaponFields[i].name, parmName ) )
		{
			switch( jka_vehWeaponFields[i].type )
			{
			case VF_INT:
				*(int *)(b+jka_vehWeaponFields[i].ofs) = atoi(value);
				break;
			case VF_FLOAT:
				*(float *)(b+jka_vehWeaponFields[i].ofs) = atof(value);
				break;
			case VF_LSTRING:
				if (!*(char **)(b+jka_vehWeaponFields[i].ofs))
				{
					*(char **)(b+jka_vehWeaponFields[i].ofs) = (char *)jka_BG_Alloc(1024);
					strcpy(*(char **)(b+jka_vehWeaponFields[i].ofs), value);
				}
				break;
			case VF_VECTOR:
				_iFieldsRead = sscanf (value, "%f %f %f", &vec[0], &vec[1], &vec[2]);
				if (_iFieldsRead!=3)
				{
					/* Com_Printf warning omitted -- no output bytes */
				}
				((float *)(b+jka_vehWeaponFields[i].ofs))[0] = vec[0];
				((float *)(b+jka_vehWeaponFields[i].ofs))[1] = vec[1];
				((float *)(b+jka_vehWeaponFields[i].ofs))[2] = vec[2];
				break;
			case VF_BOOL:
				*(qboolean *)(b+jka_vehWeaponFields[i].ofs) = (qboolean)(atof(value)!=0);
				break;
			/* VF_VEHTYPE / VF_ANIM / VF_MODEL / VF_EFFECT / VF_SOUND: engine configstrings
			 * or GetIDForString -- never reached by vehWeaponFields[], so omitted here. */
			case VF_WEAPON:
			case VF_MODEL_CLIENT:
			case VF_EFFECT_CLIENT:
			case VF_SHADER:
			case VF_SHADER_NOMIP:
			case VF_SOUND_CLIENT:
				break; /* QAGAME no-ops */
			default:
				return qfalse;
			}
			break;
		}
	}
	if ( i == NUM_VWEAP_PARMS )
		return qfalse;
	else
		return qtrue;
}
