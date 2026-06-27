/*
 * Oracle TU for anims.h. Unlike the struct/function oracles, this one
 * `#include`s the AUTHENTIC, unmodified Raven header directly (it is a pure
 * `enum` with no clang-hostile constructs), so the C compiler computing these
 * enumerator values is fully independent of the Rust port in
 * `src/codemp/game/anims.rs` (which is generated from the same header). The
 * accessors expose a spread of checkpoints plus the terminal counts and the
 * SABER_ANIM_GROUP_SIZE macro for the parity test to assert bit-for-bit.
 *
 * Built only under the `oracle` cargo feature; the -I to the raven-jediacademy
 * reference tree is supplied per-file in build.rs.
 */
#include "anims.h"

int jka_anim_FACE_TALK0(void) { return FACE_TALK0; }
int jka_anim_BOTH_ATTACK10(void) { return BOTH_ATTACK10; }
int jka_anim_BOTH_A1_T__B_(void) { return BOTH_A1_T__B_; }
int jka_anim_BOTH_A2_T__B_(void) { return BOTH_A2_T__B_; }
int jka_anim_BOTH_T2__R_T_(void) { return BOTH_T2__R_T_; }
int jka_anim_BOTH_A4_TL_BR(void) { return BOTH_A4_TL_BR; }
int jka_anim_BOTH_T5__L_BL(void) { return BOTH_T5__L_BL; }
int jka_anim_BOTH_T7__R_TL(void) { return BOTH_T7__R_TL; }
int jka_anim_BOTH_K7_S7_T_(void) { return BOTH_K7_S7_T_; }
int jka_anim_BOTH_BF1BREAK(void) { return BOTH_BF1BREAK; }
int jka_anim_BOTH_CONSOLE2HOLDCOMSTOP(void) { return BOTH_CONSOLE2HOLDCOMSTOP; }
int jka_anim_BOTH_VT_IDLE_SR(void) { return BOTH_VT_IDLE_SR; }
int jka_anim_BOTH_VICTORY_STAFF(void) { return BOTH_VICTORY_STAFF; }
int jka_anim_BOTH_CHOKE1(void) { return BOTH_CHOKE1; }
int jka_anim_LEGS_S1_RUP4(void) { return LEGS_S1_RUP4; }
int jka_anim_BOTH_CIN_50(void) { return BOTH_CIN_50; }
int jka_anim_MAX_ANIMATIONS(void) { return MAX_ANIMATIONS; }
int jka_anim_MAX_TOTALANIMATIONS(void) { return MAX_TOTALANIMATIONS; }

int jka_SABER_ANIM_GROUP_SIZE(void) { return SABER_ANIM_GROUP_SIZE; }
