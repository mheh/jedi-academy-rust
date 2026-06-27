/*
 * Value oracle for the bg_weapons.c data tables. The real bg_weapons.c cannot be
 * `#include`d cleanly (its quoted `#include "q_shared.h"` resolves to the heavy real
 * header in its own directory), so the table DATA is transcribed here independently
 * from the Rust port -- the element-wise compare in the Rust test then catches any
 * single-value typo on either side. The TYPES and enum values (weaponData_t,
 * AMMO_*, WP_NUM_WEAPONS, ...) come from the AUTHENTIC bg_weapons.h (`#include`d via
 * the reference -I), so only the numeric data is re-typed. Built only under `oracle`.
 */

#include <stddef.h>
typedef float vec_t;
typedef vec_t vec3_t[3];
#include "bg_weapons.h"

/* The C source lists only 17 rows for this [WP_NUM_WEAPONS] (19) array; C zero-fills
 * the last two (WP_EMPLACED_GUN, WP_TURRET) -- transcribed exactly so. */
static vec3_t oracle_WP_MuzzlePoint[WP_NUM_WEAPONS] = {
	{0,		0,		0	},	// WP_NONE,
	{0	,	8,		0	},	// WP_STUN_BATON,
	{0	,	8,		0	},	// WP_MELEE,
	{8	,	16,		0	},	// WP_SABER,
	{12,	6,		-6	},	// WP_BRYAR_PISTOL,
	{12,	6,		-6	},	// WP_BLASTER,
	{12,	6,		-6	},	// WP_DISRUPTOR,
	{12,	2,		-6	},	// WP_BOWCASTER,
	{12,	4.5,	-6	},	// WP_REPEATER,
	{12,	6,		-6	},	// WP_DEMP2,
	{12,	6,		-6	},	// WP_FLECHETTE,
	{12,	8,		-4	},	// WP_ROCKET_LAUNCHER,
	{12,	0,		-4	},	// WP_THERMAL,
	{12,	0,		-10	},	// WP_TRIP_MINE,
	{12,	0,		-4	},	// WP_DET_PACK,
	{12,	6,		-6	},	// WP_CONCUSSION
	{12,	6,		-6	},	// WP_BRYAR_OLD,
};

static weaponData_t oracle_weaponData[WP_NUM_WEAPONS] = {
	{ AMMO_NONE,        0,  0,  0,    0,    0,  0,    0,    0,   0,   0, 0,    0, 0 },    // WP_NONE
	{ AMMO_NONE,        5,  0,  400,  8192, 0,  400,  8192, 0,   0,   0, 0,    0, 0 },    // WP_STUN_BATON
	{ AMMO_NONE,        5,  0,  400,  8192, 0,  400,  8192, 0,   0,   0, 0,    0, 0 },    // WP_MELEE
	{ AMMO_NONE,        5,  0,  100,  8192, 0,  100,  8192, 0,   0,   0, 0,    0, 0 },    // WP_SABER
	{ AMMO_BLASTER,     0,  0,  800,  8192, 0,  800,  8192, 0,   0,   0, 0,    0, 0 },    // WP_BRYAR_PISTOL
	{ AMMO_BLASTER,     5,  2,  350,  8192, 3,  150,  8192, 0,   0,   0, 0,    0, 0 },    // WP_BLASTER
	{ AMMO_POWERCELL,   5,  5,  600,  8192, 6,  1300, 8192, 0,   200, 0, 3,    0, 1700 }, // WP_DISRUPTOR
	{ AMMO_POWERCELL,   5,  5,  1000, 8192, 5,  750,  8192, 400, 0,   5, 0, 1700, 0 },    // WP_BOWCASTER
	{ AMMO_METAL_BOLTS, 5,  1,  100,  8192, 15, 800,  8192, 0,   0,   0, 0,    0, 0 },    // WP_REPEATER
	{ AMMO_POWERCELL,   5,  8,  500,  8192, 6,  900,  8192, 0,   250, 0, 3,    0, 2100 }, // WP_DEMP2
	{ AMMO_METAL_BOLTS, 5,  10, 700,  8192, 15, 800,  8192, 0,   0,   0, 0,    0, 0 },    // WP_FLECHETTE
	{ AMMO_ROCKETS,     5,  1,  900,  8192, 2,  1200, 8192, 0,   0,   0, 0,    0, 0 },    // WP_ROCKET_LAUNCHER
	{ AMMO_THERMAL,     0,  1,  800,  8192, 1,  400,  8192, 0,   0,   0, 0,    0, 0 },    // WP_THERMAL
	{ AMMO_TRIPMINE,    0,  1,  800,  8192, 1,  400,  8192, 0,   0,   0, 0,    0, 0 },    // WP_TRIP_MINE
	{ AMMO_DETPACK,     0,  1,  800,  8192, 0,  400,  8192, 0,   0,   0, 0,    0, 0 },    // WP_DET_PACK
	{ AMMO_METAL_BOLTS, 40, 40, 800,  8192, 50, 1200, 8192, 0,   0,   0, 0,    0, 0 },    // WP_CONCUSSION
	{ AMMO_BLASTER,     15, 2,  400,  8192, 2,  400,  8192, 0,   200, 0, 1,    0, 1500 }, // WP_BRYAR_OLD
	{ 0,                0,  0,  100,  8192, 0,  100,  8192, 0,   0,   0, 0,    0, 0 },    // WP_EMPLACED_GUN
	{ 0,                0,  0,  0,    0,    0,  0,    0,    0,   0,   0, 0,    0, 0 },    // WP_TURRET
};

static ammoData_t oracle_ammoData[AMMO_MAX] = {
	{ 0 },   // AMMO_NONE
	{ 100 }, // AMMO_FORCE
	{ 300 }, // AMMO_BLASTER
	{ 300 }, // AMMO_POWERCELL
	{ 300 }, // AMMO_METAL_BOLTS
	{ 25 },  // AMMO_ROCKETS
	{ 800 }, // AMMO_EMPLACED
	{ 10 },  // AMMO_THERMAL
	{ 10 },  // AMMO_TRIPMINE
	{ 10 },  // AMMO_DETPACK
};

const vec3_t *jka_bw_muzzle_ptr(void) { return oracle_WP_MuzzlePoint; }
const weaponData_t *jka_bw_weaponData_ptr(void) { return oracle_weaponData; }
const ammoData_t *jka_bw_ammoData_ptr(void) { return oracle_ammoData; }
