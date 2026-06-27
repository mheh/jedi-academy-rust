/*
 * Oracle TU for bg_public.h. Like q_shared_h_oracle.c, it carries a minimal
 * prelude of the real external limit values (from q_shared.h) plus verbatim
 * copies of the bg_public.h definitions under test, and exposes the C compiler's
 * computed results so the Rust port can be asserted bit-for-bit. We copy rather
 * than #include the real header because bg_public.h drags in the whole
 * clang-hostile include tree (bg_weapons.h, bg_vehicles.h, q_shared.h, ...).
 *
 * This first slice covers the computed CS_* config-string chain, where the
 * arithmetic threads through several external MAX_* limits -- exactly where a
 * transcription error would hide. Grown per logical group as bg_public.rs lands.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

/* --- prelude: real config-string limit values, verbatim from q_shared.h --- */
#define MAX_CLIENTS 32
#define MAX_LOCATIONS 64
#define MAX_MODELS 512
#define MAX_SOUNDS 256
#define MAX_ICONS 64
#define MAX_FX 64
#define MAX_G2BONES 64
#define MAX_AMBIENT_SETS 256
#define MAX_LIGHT_STYLES 64
#define MAX_TERRAINS 1
#define MAX_SUB_BSP 32

/* --- CS_* chain from bg_public.h (only the computed tail matters, but the
 * leading literals are kept so the copy is a faithful transcription).
 * These are the retail PC (raven-jediacademy) values: CS_AMBIENT_SET 37 (the
 * Xbox/grayj tree had 38) and CS_MAX = (CS_BSP_MODELS + MAX_SUB_BSP). This also
 * matches the OpenJK/TaystJK runtime client; the lone OpenJK-only addition kept
 * is CS_LEGACY_FIXES (36), in the unused gap so it does not shift the tail. See
 * bg_public.rs's config-string note and crate/DEVIATIONS.md. --- */
#define CS_GLOBAL_AMBIENT_SET 32
#define CS_LEGACY_FIXES 36
#define CS_AMBIENT_SET 37
#define CS_SIEGE_STATE (CS_AMBIENT_SET + MAX_AMBIENT_SETS)
#define CS_SIEGE_OBJECTIVES (CS_SIEGE_STATE + 1)
#define CS_SIEGE_TIMEOVERRIDE (CS_SIEGE_OBJECTIVES + 1)
#define CS_SIEGE_WINTEAM (CS_SIEGE_TIMEOVERRIDE + 1)
#define CS_SIEGE_ICONS (CS_SIEGE_WINTEAM + 1)
#define CS_MODELS (CS_SIEGE_ICONS + 1)
#define CS_SKYBOXORG (CS_MODELS + MAX_MODELS)
#define CS_SOUNDS (CS_SKYBOXORG + 1)
#define CS_ICONS (CS_SOUNDS + MAX_SOUNDS)
#define CS_PLAYERS (CS_ICONS + MAX_ICONS)
#define CS_G2BONES (CS_PLAYERS + MAX_CLIENTS)
#define CS_LOCATIONS (CS_G2BONES + MAX_G2BONES)
#define CS_PARTICLES (CS_LOCATIONS + MAX_LOCATIONS)
#define CS_EFFECTS (CS_PARTICLES + MAX_LOCATIONS)
#define CS_LIGHT_STYLES (CS_EFFECTS + MAX_FX)
#define CS_TERRAINS (CS_LIGHT_STYLES + (MAX_LIGHT_STYLES * 3))
#define CS_BSP_MODELS (CS_TERRAINS + MAX_TERRAINS)
#define CS_MAX (CS_BSP_MODELS + MAX_SUB_BSP)

int jka_bgp_CS_SIEGE_STATE(void) { return CS_SIEGE_STATE; }
int jka_bgp_CS_MODELS(void) { return CS_MODELS; }
int jka_bgp_CS_ICONS(void) { return CS_ICONS; }
int jka_bgp_CS_LIGHT_STYLES(void) { return CS_LIGHT_STYLES; }
int jka_bgp_CS_TERRAINS(void) { return CS_TERRAINS; }
int jka_bgp_CS_BSP_MODELS(void) { return CS_BSP_MODELS; }
int jka_bgp_CS_MAX(void) { return CS_MAX; }

/* --- BG_GiveMeVectorFromMatrix (bg_public.h, the native `static ID_INLINE`
 * branch; the `#ifdef __LCC__` VM build uses an identical out-of-line copy at
 * bg_misc.c:736). Verbatim copy of the mdxaBone_t bolt transform, the
 * Eorientations selectors (q_shared.h:2994), and the inline body. The matrix is
 * handed in as 12 row-major floats so the Rust side can drive it without
 * reproducing the struct ABI in this TU. --- */
typedef struct { float matrix[3][4]; } mdxaBone_oracle_t;

enum Eorientations_oracle {
	ORIGIN = 0,
	POSITIVE_X,
	POSITIVE_Z,
	POSITIVE_Y,
	NEGATIVE_X,
	NEGATIVE_Z,
	NEGATIVE_Y
};

static void BG_GiveMeVectorFromMatrix_oracle(mdxaBone_oracle_t *boltMatrix, int flags, float *vec) {
	switch (flags)
	{
	case ORIGIN:
		vec[0] = boltMatrix->matrix[0][3];
		vec[1] = boltMatrix->matrix[1][3];
		vec[2] = boltMatrix->matrix[2][3];
		break;
	case POSITIVE_Y:
		vec[0] = boltMatrix->matrix[0][1];
		vec[1] = boltMatrix->matrix[1][1];
		vec[2] = boltMatrix->matrix[2][1];
 		break;
	case POSITIVE_X:
		vec[0] = boltMatrix->matrix[0][0];
		vec[1] = boltMatrix->matrix[1][0];
		vec[2] = boltMatrix->matrix[2][0];
		break;
	case POSITIVE_Z:
		vec[0] = boltMatrix->matrix[0][2];
		vec[1] = boltMatrix->matrix[1][2];
		vec[2] = boltMatrix->matrix[2][2];
		break;
	case NEGATIVE_Y:
		vec[0] = -boltMatrix->matrix[0][1];
		vec[1] = -boltMatrix->matrix[1][1];
		vec[2] = -boltMatrix->matrix[2][1];
		break;
	case NEGATIVE_X:
		vec[0] = -boltMatrix->matrix[0][0];
		vec[1] = -boltMatrix->matrix[1][0];
		vec[2] = -boltMatrix->matrix[2][0];
		break;
	case NEGATIVE_Z:
		vec[0] = -boltMatrix->matrix[0][2];
		vec[1] = -boltMatrix->matrix[1][2];
		vec[2] = -boltMatrix->matrix[2][2];
		break;
	}
}

void jka_bgp_GiveMeVectorFromMatrix(const float *m12, int flags, float *out) {
	mdxaBone_oracle_t b;
	int i;
	for (i = 0; i < 12; i++) {
		((float *)b.matrix)[i] = m12[i];
	}
	BG_GiveMeVectorFromMatrix_oracle(&b, flags, out);
}
