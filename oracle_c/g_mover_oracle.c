/* Oracle extraction of g_mover.c's three rotation-matrix helpers
 * (refs/raven-jediacademy/codemp/game/g_mover.c:118/128/142):
 *   G_CreateRotationMatrix, G_TransposeMatrix, G_RotatePoint.
 * All three are pure vec3 math over a vec3_t[3] basis. Their only deps are the
 * verbatim q_math.c AngleVectors + VectorInverse and the q_shared.h DotProduct /
 * VectorCopy macros, included inline here so the extract is self-contained (same
 * style as g_client_oracle.c). The jka_ wrappers marshal flat float arrays in/out
 * (a 3x3 matrix is a flat float[9], row-major) so the Rust test needs no struct
 * layout match. Renamed `jka_` to avoid colliding with the test binary. */

#include <math.h> /* sin, cos */

typedef float vec_t;
typedef vec_t vec3_t[3];

#ifndef M_PI
#define M_PI 3.14159265358979323846f
#endif

#define PITCH 0
#define YAW 1
#define ROLL 2

#define DotProduct(x, y) ((x)[0] * (y)[0] + (x)[1] * (y)[1] + (x)[2] * (y)[2])
#define VectorCopy(a, b) ((b)[0] = (a)[0], (b)[1] = (a)[1], (b)[2] = (a)[2])

/* Verbatim q_math.c AngleVectors (oracle/q_math_oracle.c:275). */
static void AngleVectors(const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up) {
    float angle;
    static float sr, sp, sy, cr, cp, cy;
    // static to help MS compiler fp bugs

    angle = angles[YAW] * (M_PI * 2 / 360);
    sy = sin(angle);
    cy = cos(angle);
    angle = angles[PITCH] * (M_PI * 2 / 360);
    sp = sin(angle);
    cp = cos(angle);
    angle = angles[ROLL] * (M_PI * 2 / 360);
    sr = sin(angle);
    cr = cos(angle);

    if (forward) {
        forward[0] = cp * cy;
        forward[1] = cp * sy;
        forward[2] = -sp;
    }
    if (right) {
        right[0] = (-1 * sr * sp * cy + -1 * cr * -sy);
        right[1] = (-1 * sr * sp * sy + -1 * cr * cy);
        right[2] = -1 * sr * cp;
    }
    if (up) {
        up[0] = (cr * sp * cy + -sr * -sy);
        up[1] = (cr * sp * sy + -sr * cy);
        up[2] = cr * cp;
    }
}

/* Verbatim q_math.c VectorInverse (oracle/q_math_oracle.c:133). */
static void VectorInverse(vec3_t v) {
    v[0] = -v[0];
    v[1] = -v[1];
    v[2] = -v[2];
}

/* Verbatim bodies from g_mover.c. */
static void G_CreateRotationMatrix(vec3_t angles, vec3_t matrix[3]) {
    AngleVectors(angles, matrix[0], matrix[1], matrix[2]);
    VectorInverse(matrix[1]);
}

static void G_TransposeMatrix(vec3_t matrix[3], vec3_t transpose[3]) {
    int i, j;
    for (i = 0; i < 3; i++) {
        for (j = 0; j < 3; j++) {
            transpose[i][j] = matrix[j][i];
        }
    }
}

static void G_RotatePoint(vec3_t point, vec3_t matrix[3]) {
    vec3_t tvec;

    VectorCopy(point, tvec);
    point[0] = DotProduct(matrix[0], tvec);
    point[1] = DotProduct(matrix[1], tvec);
    point[2] = DotProduct(matrix[2], tvec);
}

void jka_G_CreateRotationMatrix(const float *angles, float *out_matrix) {
    vec3_t a;
    vec3_t matrix[3];

    VectorCopy(angles, a);
    G_CreateRotationMatrix(a, matrix);

    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            out_matrix[i * 3 + j] = matrix[i][j];
        }
    }
}

void jka_G_TransposeMatrix(const float *in_matrix, float *out_transpose) {
    vec3_t matrix[3];
    vec3_t transpose[3];

    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            matrix[i][j] = in_matrix[i * 3 + j];
        }
    }
    G_TransposeMatrix(matrix, transpose);
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            out_transpose[i * 3 + j] = transpose[i][j];
        }
    }
}

void jka_G_RotatePoint(const float *in_point, const float *in_matrix, float *out_point) {
    vec3_t point;
    vec3_t matrix[3];

    VectorCopy(in_point, point);
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            matrix[i][j] = in_matrix[i * 3 + j];
        }
    }
    G_RotatePoint(point, matrix);
    VectorCopy(point, out_point);
}
