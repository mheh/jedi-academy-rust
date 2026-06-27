/* Oracle extraction of g_client.c's SetClientViewAngle
 * (refs/raven-jediacademy/codemp/game/g_client.c:1025). The function reads only the
 * passed angle[3] and ent->client->pers.cmd.angles[3], and writes
 * ent->client->ps.delta_angles[3], ent->s.angles[3], and
 * ent->client->ps.viewangles[3]. The jka_ wrapper marshals those flat in/out
 * arrays into a minimal gentity_t/gclient_t so the Rust test needs no cross-FFI
 * struct-layout match. ANGLE2SHORT and VectorCopy are the verbatim q_shared.h
 * macros. Renamed `jka_` to avoid colliding with the test binary. */

#include <string.h> /* memset */

typedef float vec_t;
typedef vec_t vec3_t[3];

#define ANGLE2SHORT(x) ((int)((x) * 65536 / 360) & 65535)
#define VectorCopy(a, b) ((b)[0] = (a)[0], (b)[1] = (a)[1], (b)[2] = (a)[2])

typedef struct {
    int delta_angles[3];
    vec3_t viewangles;
} playerState_t;

typedef struct {
    int angles[3];
} usercmd_t;

typedef struct {
    usercmd_t cmd;
} clientPersistant_t;

typedef struct {
    playerState_t ps;
    clientPersistant_t pers;
} gclient_t;

typedef struct {
    vec3_t angles;
} entityState_t;

typedef struct gentity_s {
    entityState_t s;
    gclient_t *client;
} gentity_t;

/* Verbatim body from g_client.c:1025. */
static void SetClientViewAngle(gentity_t *ent, vec3_t angle) {
    int i;

    // set the delta angle
    for (i = 0; i < 3; i++) {
        int cmdAngle;

        cmdAngle = ANGLE2SHORT(angle[i]);
        ent->client->ps.delta_angles[i] = cmdAngle - ent->client->pers.cmd.angles[i];
    }
    VectorCopy(angle, ent->s.angles);
    VectorCopy(ent->s.angles, ent->client->ps.viewangles);
}

void jka_SetClientViewAngle(const float *angle, const int *cmd_angles,
                            int *out_delta_angles, float *out_s_angles,
                            float *out_viewangles) {
    gentity_t ent;
    gclient_t client;
    vec3_t a;
    int i;

    memset(&ent, 0, sizeof(ent));
    memset(&client, 0, sizeof(client));
    ent.client = &client;

    VectorCopy(angle, a);
    for (i = 0; i < 3; i++) {
        client.pers.cmd.angles[i] = cmd_angles[i];
    }

    SetClientViewAngle(&ent, a);

    for (i = 0; i < 3; i++) {
        out_delta_angles[i] = client.ps.delta_angles[i];
        out_s_angles[i] = ent.s.angles[i];
        out_viewangles[i] = client.ps.viewangles[i];
    }
}

/* ------------------------------------------------------------------------- */

/* Oracle extraction of g_client.c's ClientCleanName
 * (refs/raven-jediacademy/codemp/game/g_client.c:1244). A pure name-sanitiser:
 * strips leading spaces, caps runs of spaces at 3, drops black (^0) color
 * codes, keeps other ^N codes, and substitutes "Padawan" for an empty/colorless
 * result. ColorIndex and Q_COLOR_ESCAPE are the verbatim q_shared.h macro/const;
 * Q_strncpyz is the verbatim safe-strncpy. The jka_ wrapper just forwards the
 * three args so the Rust test can drive flat C strings. */

#define Q_COLOR_ESCAPE '^'
#define ColorIndex(c) (((c) - '0') & 7)

/* Verbatim Q_strncpyz from q_shared.c (NULL-guards dropped — the call site
 * always passes a valid dest/src, matching the Rust port's preconditions). */
static void Q_strncpyz_oracle(char *dest, const char *src, int destsize) {
    strncpy(dest, src, destsize - 1);
    dest[destsize - 1] = 0;
}

/* Verbatim body from g_client.c:1244. */
static void ClientCleanName(const char *in, char *out, int outSize) {
    int len, colorlessLen;
    char ch;
    char *p;
    int spaces;

    //save room for trailing null byte
    outSize--;

    len = 0;
    colorlessLen = 0;
    p = out;
    *p = 0;
    spaces = 0;

    while (1) {
        ch = *in++;
        if (!ch) {
            break;
        }

        // don't allow leading spaces
        if (!*p && ch == ' ') {
            continue;
        }

        // check colors
        if (ch == Q_COLOR_ESCAPE) {
            // solo trailing carat is not a color prefix
            if (!*in) {
                break;
            }

            // don't allow black in a name, period
            if (ColorIndex(*in) == 0) {
                in++;
                continue;
            }

            // make sure room in dest for both chars
            if (len > outSize - 2) {
                break;
            }

            *out++ = ch;
            *out++ = *in++;
            len += 2;
            continue;
        }

        // don't allow too many consecutive spaces
        if (ch == ' ') {
            spaces++;
            if (spaces > 3) {
                continue;
            }
        } else {
            spaces = 0;
        }

        if (len > outSize - 1) {
            break;
        }

        *out++ = ch;
        colorlessLen++;
        len++;
    }
    *out = 0;

    // don't allow empty names
    if (*p == 0 || colorlessLen == 0) {
        Q_strncpyz_oracle(p, "Padawan", outSize);
    }
}

void jka_ClientCleanName(const char *in, char *out, int outSize) {
    ClientCleanName(in, out, outSize);
}
