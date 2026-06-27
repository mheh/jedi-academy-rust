/*
 * Logic oracle for BG_GiveMeVectorFromMatrix (bg_misc.c:736) ported into
 * src/codemp/game/bg_misc.rs.
 *
 * In the real source this function lives inside an `#ifdef __LCC__` (VM/LCC-only build
 * path); the Rust port lands it unconditionally. The real bg_misc.c cannot be
 * `#include`d (its quoted includes drag in the clang-hostile reference tree), so the
 * body is transcribed here VERBATIM, run over a MINIMAL mdxaBone_t holding only the
 * 3x4 float matrix it touches. The full mdxaBone_t layout is verified separately in
 * q_shared_h_oracle.c. A wrapper marshals the matrix + flag in and writes the resulting
 * vec3 back out through a pointer. Built only under `oracle`.
 */

typedef struct {
	float matrix[3][4];
} mdxaBone_t;

/* Eorientations flag values (q_shared.h) — note the non-obvious X, Z, Y order. */
#define ORIGIN     0
#define POSITIVE_X 1
#define POSITIVE_Z 2
#define POSITIVE_Y 3
#define NEGATIVE_X 4
#define NEGATIVE_Z 5
#define NEGATIVE_Y 6

/* verbatim body of bg_misc.c:736 (the __LCC__ branch) */
void BG_GiveMeVectorFromMatrix(mdxaBone_t *boltMatrix, int flags, float vec[3])
{
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

/*
 * Drive one BG_GiveMeVectorFromMatrix call: load the 12 matrix floats (row-major,
 * matrix[3][4]) + the flag in, run the C body, write the resulting vec3 out. `out` is
 * pre-seeded by the caller so the no-match (untouched) path is observable.
 */
void jka_bg_give_me_vector_from_matrix(const float *m12, int flags, float *out)
{
	mdxaBone_t boltMatrix;
	int r, c;
	for (r = 0; r < 3; r++)
		for (c = 0; c < 4; c++)
			boltMatrix.matrix[r][c] = m12[r * 4 + c];

	BG_GiveMeVectorFromMatrix(&boltMatrix, flags, out);
}
