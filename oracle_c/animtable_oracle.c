/*
 * Oracle TU for cgame/animtable.h. Like anims_oracle.c, it `#include`s the
 * AUTHENTIC, unmodified Raven header directly so the C compiler building the
 * `animTable[]` array is fully independent of the Rust port in
 * src/codemp/cgame/animtable.rs (both derived from the same header).
 *
 * animtable.h needs three things from its including TU before it expands: the
 * `animNumber_t` enum + `MAX_ANIMATIONS` (from the real anims.h), and the
 * `ENUM2STRING` macro + `stringID_table_t` struct (verbatim from q_shared.h:3061).
 * Providing those minimal prerequisites here lets us pull in the real table body
 * without dragging in the whole clang-hostile q_shared.h. The non-Xbox / non-UI
 * `#else` branch (the actual definition) is selected since none of _XBOX/_UI are
 * defined.
 *
 * Built only under the `oracle` cargo feature; the -I to the raven-jediacademy
 * reference cgame + game trees is supplied per-file in build.rs.
 */
#include <stddef.h> /* NULL */

#include "anims.h" /* animNumber_t enum + MAX_ANIMATIONS */

/* verbatim from refs/raven-jediacademy/codemp/game/q_shared.h:3061-3066 */
#define ENUM2STRING(arg) #arg, arg
typedef struct stringID_table_s {
    char *name;
    int id;
} stringID_table_t;

#include "animtable.h" /* defines stringID_table_t animTable[MAX_ANIMATIONS+1] */

int jka_animTable_count(void) {
    return (int)(sizeof(animTable) / sizeof(animTable[0]));
}
const char *jka_animTable_name(int i) {
    return animTable[i].name;
}
int jka_animTable_id(int i) {
    return animTable[i].id;
}
