/*
 * Oracle TU for bg_weapons.h. Like anims_oracle.c, this `#include`s the AUTHENTIC
 * Raven `bg_weapons.h` directly (it is self-contained -- no other includes), so the
 * C compiler reads the real enums / struct layouts rather than a transcription. The
 * extra -I points at the reference tree (see build.rs).
 *
 * Pointer-free structs => arch-independent; built only under the `oracle` feature.
 */

#include <stddef.h>
#include "bg_weapons.h"

size_t jka_bw_sizeof_weaponData_t(void) { return sizeof(weaponData_t); }
size_t jka_bw_sizeof_ammoData_t(void) { return sizeof(ammoData_t); }

int jka_bw_WP_NONE(void) { return WP_NONE; }
int jka_bw_WP_NUM_WEAPONS(void) { return WP_NUM_WEAPONS; }
int jka_bw_LAST_USEABLE_WEAPON(void) { return LAST_USEABLE_WEAPON; }
int jka_bw_AMMO_NONE(void) { return AMMO_NONE; }
int jka_bw_AMMO_MAX(void) { return AMMO_MAX; }
