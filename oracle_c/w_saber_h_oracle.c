/*
 * Oracle TU for w_saber.h's #define tunables and the two enums (the anonymous
 * FJ_* force-jump directions and evasionType_t).
 *
 * Unlike bg_local.h, w_saber.h IS cleanly includable: its external deps are
 * NUM_FORCE_POWER_LEVELS / NUM_FORCE_POWERS (used only to size the four `extern`
 * force-tuning array declarations), a forward `vmCvar_t` (named by the PC tree's
 * `extern vmCvar_t g_MaxHolocronCarry;`, see below), plus the
 * `#include "../namespace_begin.h"` / `namespace_end.h` pair, which are _XBOX-only
 * no-ops. So rather than transcribe,
 * we #include the AUTHENTIC, unmodified Raven header (-I supplied per-file in
 * build.rs) after predefining those two force constants. This reads the REAL
 * macro / enumerator values, catching any transcription error in the Rust port
 * (src/codemp/game/w_saber_h.rs). The two predefined macros feed only the extern
 * array decls (which the Rust port carries as comments), so they do not affect any
 * value asserted here.
 *
 * Built only under the `oracle` cargo feature (see build.rs).
 */

/* q_shared.h force counts, provided so w_saber.h's extern table decls compile.
   FORCE_LEVEL_0..3 + NUM_FORCE_POWER_LEVELS => 4; forcePowers_t FP_FIRST..
   FP_SABERTHROW + NUM_FORCE_POWERS => 18. */
#define NUM_FORCE_POWER_LEVELS 4
#define NUM_FORCE_POWERS 18

/* The PC source (raven-jediacademy) w_saber.h activates `extern vmCvar_t
   g_MaxHolocronCarry;` (the grayj/Xbox tree had it commented out). That extern names a
   g_main.c static in a separate TU; no value or layout of it is asserted here, so an
   incomplete forward typedef is all that's needed for the header to parse. */
typedef struct vmCvar_s vmCvar_t;

#include "w_saber.h"

int jka_ws_ARMOR_EFFECT_TIME(void) { return ARMOR_EFFECT_TIME; }

int jka_ws_SEF_HITENEMY(void)   { return SEF_HITENEMY; }
int jka_ws_SEF_HITOBJECT(void)  { return SEF_HITOBJECT; }
int jka_ws_SEF_HITWALL(void)    { return SEF_HITWALL; }
int jka_ws_SEF_PARRIED(void)    { return SEF_PARRIED; }
int jka_ws_SEF_DEFLECTED(void)  { return SEF_DEFLECTED; }
int jka_ws_SEF_BLOCKED(void)    { return SEF_BLOCKED; }
int jka_ws_SEF_EVENTS(void)     { return SEF_EVENTS; }
int jka_ws_SEF_LOCKED(void)     { return SEF_LOCKED; }
int jka_ws_SEF_INWATER(void)    { return SEF_INWATER; }
int jka_ws_SEF_LOCK_WON(void)   { return SEF_LOCK_WON; }

int jka_ws_SES_LEAVING(void)    { return SES_LEAVING; }
int jka_ws_SES_HOVERING(void)   { return SES_HOVERING; }
int jka_ws_SES_RETURNING(void)  { return SES_RETURNING; }

int jka_ws_JSF_AMBUSH(void)     { return JSF_AMBUSH; }

float jka_ws_SABER_RADIUS_STANDARD(void)      { return SABER_RADIUS_STANDARD; }
float jka_ws_SABER_REFLECT_MISSILE_CONE(void) { return SABER_REFLECT_MISSILE_CONE; }

int jka_ws_FORCE_POWER_MAX(void)         { return FORCE_POWER_MAX; }
int jka_ws_MAX_GRIP_DISTANCE(void)       { return MAX_GRIP_DISTANCE; }
int jka_ws_MAX_TRICK_DISTANCE(void)      { return MAX_TRICK_DISTANCE; }
int jka_ws_FORCE_JUMP_CHARGE_TIME(void)  { return FORCE_JUMP_CHARGE_TIME; }
int jka_ws_GRIP_DRAIN_AMOUNT(void)       { return GRIP_DRAIN_AMOUNT; }
int jka_ws_FORCE_LIGHTNING_RADIUS(void)  { return FORCE_LIGHTNING_RADIUS; }
int jka_ws_MAX_DRAIN_DISTANCE(void)      { return MAX_DRAIN_DISTANCE; }

int jka_ws_FJ_FORWARD(void)  { return FJ_FORWARD; }
int jka_ws_FJ_BACKWARD(void) { return FJ_BACKWARD; }
int jka_ws_FJ_RIGHT(void)    { return FJ_RIGHT; }
int jka_ws_FJ_LEFT(void)     { return FJ_LEFT; }
int jka_ws_FJ_UP(void)       { return FJ_UP; }

int jka_ws_EVASION_NONE(void)       { return EVASION_NONE; }
int jka_ws_EVASION_PARRY(void)      { return EVASION_PARRY; }
int jka_ws_EVASION_DUCK_PARRY(void) { return EVASION_DUCK_PARRY; }
int jka_ws_EVASION_JUMP_PARRY(void) { return EVASION_JUMP_PARRY; }
int jka_ws_EVASION_DODGE(void)      { return EVASION_DODGE; }
int jka_ws_EVASION_JUMP(void)       { return EVASION_JUMP; }
int jka_ws_EVASION_DUCK(void)       { return EVASION_DUCK; }
int jka_ws_EVASION_FJUMP(void)      { return EVASION_FJUMP; }
int jka_ws_EVASION_CARTWHEEL(void)  { return EVASION_CARTWHEEL; }
int jka_ws_EVASION_OTHER(void)      { return EVASION_OTHER; }
int jka_ws_NUM_EVASION_TYPES(void)  { return NUM_EVASION_TYPES; }

float jka_ws_SABERMINS_X(void) { return SABERMINS_X; }
float jka_ws_SABERMINS_Y(void) { return SABERMINS_Y; }
float jka_ws_SABERMINS_Z(void) { return SABERMINS_Z; }
float jka_ws_SABERMAXS_X(void) { return SABERMAXS_X; }
float jka_ws_SABERMAXS_Y(void) { return SABERMAXS_Y; }
float jka_ws_SABERMAXS_Z(void) { return SABERMAXS_Z; }
float jka_ws_SABER_MIN_THROW_DIST(void) { return SABER_MIN_THROW_DIST; }
