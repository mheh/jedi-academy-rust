/*
 * Item-system oracle for the bg_misc.c slice ported into src/codemp/game/bg_misc.rs:
 * the master bg_itemlist table + bg_numItems, the three item selectors
 * (BG_GetItemIndexByTag, BG_IsItemSelectable, BG_CycleInven) and the BG_FindItem
 * lookup family.
 *
 * The real bg_misc.c cannot be included (its quoted includes drag in the clang-hostile
 * reference tree), so -- like bg_saber_oracle.c -- this self-contained TU transcribes
 * the gitem_t struct plus the IT_, HI_, PW_, WP_, AMMO_ and STAT_ constants from
 * bg_public.h / bg_weapons.h, then copies bg_itemlist and the function bodies VERBATIM
 * from bg_misc.c. The Rust port is compared element-by-element against the table
 * (strings via CStr / NULL), and the selectors/finders are driven by int-marshalling
 * wrappers (finders return an INDEX -- pointers cannot cross between the two
 * independent table copies). Built only under the oracle feature. Kept apart from
 * bg_misc_oracle.c to avoid that TU's live PW_/WP_/YAW macro soup.
 */

#include <stdlib.h> /* abort */

#define MAX_ITEM_MODELS 4

/* itemType_t (bg_public.h) */
#define IT_BAD 0
#define IT_WEAPON 1
#define IT_AMMO 2
#define IT_ARMOR 3
#define IT_HEALTH 4
#define IT_POWERUP 5
#define IT_HOLDABLE 6
#define IT_PERSISTANT_POWERUP 7
#define IT_TEAM 8

/* holdable_t (bg_public.h) */
#define HI_NONE 0
#define HI_SEEKER 1
#define HI_SHIELD 2
#define HI_MEDPAC 3
#define HI_MEDPAC_BIG 4
#define HI_BINOCULARS 5
#define HI_SENTRY_GUN 6
#define HI_JETPACK 7
#define HI_HEALTHDISP 8
#define HI_AMMODISP 9
#define HI_EWEB 10
#define HI_CLOAK 11
#define HI_NUM_HOLDABLE 12

/* powerup_t (bg_public.h) -- only the values bg_itemlist uses */
#define PW_REDFLAG 4
#define PW_BLUEFLAG 5
#define PW_NEUTRALFLAG 6
#define PW_FORCE_ENLIGHTENED_LIGHT 12
#define PW_FORCE_ENLIGHTENED_DARK 13
#define PW_FORCE_BOON 14
#define PW_YSALAMIRI 15

/* weapon_t (bg_weapons.h) */
#define WP_STUN_BATON 1
#define WP_MELEE 2
#define WP_SABER 3
#define WP_BRYAR_PISTOL 4
#define WP_BLASTER 5
#define WP_DISRUPTOR 6
#define WP_BOWCASTER 7
#define WP_REPEATER 8
#define WP_DEMP2 9
#define WP_FLECHETTE 10
#define WP_ROCKET_LAUNCHER 11
#define WP_THERMAL 12
#define WP_TRIP_MINE 13
#define WP_DET_PACK 14
#define WP_CONCUSSION 15
#define WP_BRYAR_OLD 16
#define WP_EMPLACED_GUN 17
#define WP_TURRET 18

/* ammo_t (bg_weapons.h) */
#define AMMO_FORCE 1
#define AMMO_BLASTER 2
#define AMMO_POWERCELL 3
#define AMMO_METAL_BOLTS 4
#define AMMO_ROCKETS 5
#define AMMO_THERMAL 7
#define AMMO_TRIPMINE 8
#define AMMO_DETPACK 9

/* statIndex_t (bg_public.h) -- only the holdable slots BG_CycleInven touches */
#define STAT_HOLDABLE_ITEM 1
#define STAT_HOLDABLE_ITEMS 2
#define MAX_STATS 16

#define ERR_DROP 1
typedef int itemType_t;
typedef int qboolean;
#define qtrue 1
#define qfalse 0

/* gitem_t (bg_public.h) -- layout identical to the Rust repr(C) struct. The
 * commented-out pickup_name member is omitted, as in C. */
typedef struct gitem_s {
	char *classname;
	char *pickup_sound;
	char *world_model[MAX_ITEM_MODELS];
	char *view_model;
	char *icon;
	int quantity;
	itemType_t giType;
	int giTag;
	char *precaches;
	char *sounds;
	char *description;
} gitem_t;

/* ===== bg_itemlist[] (bg_misc.c:795) -- copied VERBATIM ===== */
gitem_t bg_itemlist[] =
{
	{
		NULL,				// classname
		NULL,				// pickup_sound
		{	NULL,			// world_model[0]
			NULL,			// world_model[1]
			0, 0} ,			// world_model[2],[3]
		NULL,				// view_model
/* icon */		NULL,		// icon
/* pickup */	//NULL,		// pickup_name
		0,					// quantity
		0,					// giType (IT_*)
		0,					// giTag
/* precache */ "",			// precaches
/* sounds */ "",			// sounds
		""					// description
	},	// leave index 0 alone

	//
	// Pickups
	//
	{
		"item_shield_sm_instant",
		"sound/player/pickupshield.wav",
        { "models/map_objects/mp/psd_sm.md3",
		0, 0, 0},
/* view */		NULL,
/* icon */		"gfx/mp/small_shield",
/* pickup *///	"Shield Small",
		25,
		IT_ARMOR,
		1, //special for shield - max on pickup is maxhealth*tag, thus small shield goes up to 100 shield
/* precache */ "",
/* sounds */ ""
		""					// description
	},
	{
		"item_shield_lrg_instant",
		"sound/player/pickupshield.wav",
        { "models/map_objects/mp/psd.md3",
		0, 0, 0},
/* view */		NULL,
/* icon */		"gfx/mp/large_shield",
/* pickup *///	"Shield Large",
		100,
		IT_ARMOR,
		2, //special for shield - max on pickup is maxhealth*tag, thus large shield goes up to 200 shield
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"item_medpak_instant",
		"sound/player/pickuphealth.wav",
        { "models/map_objects/mp/medpac.md3",
		0, 0, 0 },
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_medkit",
/* pickup *///	"Medpack",
		25,
		IT_HEALTH,
		0,
/* precache */ "",
/* sounds */ "",
		""					// description
	},

	//
	// ITEMS
	//
	{
		"item_seeker",
		"sound/weapons/w_pkup.wav",
		{ "models/items/remote.md3",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_seeker",
/* pickup *///	"Seeker Drone",
		120,
		IT_HOLDABLE,
		HI_SEEKER,
/* precache */ "",
/* sounds */ "",
		"@MENUS_AN_ATTACK_DRONE_SIMILAR"					// description
	},
	{
		"item_shield",
		"sound/weapons/w_pkup.wav",
		{ "models/map_objects/mp/shield.md3",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_shieldwall",
/* pickup *///	"Forcefield",
		120,
		IT_HOLDABLE,
		HI_SHIELD,
/* precache */ "",
/* sounds */ "sound/weapons/detpack/stick.wav sound/movers/doors/forcefield_on.wav sound/movers/doors/forcefield_off.wav sound/movers/doors/forcefield_lp.wav sound/effects/bumpfield.wav",
		"@MENUS_THIS_STATIONARY_ENERGY"					// description
	},
	{
		"item_medpac",	//should be item_bacta
		"sound/weapons/w_pkup.wav",
		{ "models/map_objects/mp/bacta.md3",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_bacta",
/* pickup *///	"Bacta Canister",
		25,
		IT_HOLDABLE,
		HI_MEDPAC,
/* precache */ "",
/* sounds */ "",
		"@SP_INGAME_BACTA_DESC"					// description
	},
	{
		"item_medpac_big",	//should be item_bacta
		"sound/weapons/w_pkup.wav",
		{ "models/items/big_bacta.md3",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_big_bacta",
/* pickup *///	"Bacta Canister",
		25,
		IT_HOLDABLE,
		HI_MEDPAC_BIG,
/* precache */ "",
/* sounds */ "",
		"@SP_INGAME_BACTA_DESC"					// description
	},
	{
		"item_binoculars",
		"sound/weapons/w_pkup.wav",
		{ "models/items/binoculars.md3",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_zoom",
/* pickup *///	"Binoculars",
		60,
		IT_HOLDABLE,
		HI_BINOCULARS,
/* precache */ "",
/* sounds */ "",
		"@SP_INGAME_LA_GOGGLES_DESC"					// description
	},
	{
		"item_sentry_gun",
		"sound/weapons/w_pkup.wav",
		{ "models/items/psgun.glm",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_sentrygun",
/* pickup *///	"Sentry Gun",
		120,
		IT_HOLDABLE,
		HI_SENTRY_GUN,
/* precache */ "",
/* sounds */ "",
		"@MENUS_THIS_DEADLY_WEAPON_IS"					// description
	},
	{
		"item_jetpack",
		"sound/weapons/w_pkup.wav",
		{ "models/items/psgun.glm", //FIXME: no model
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_jetpack",
/* pickup *///	"Sentry Gun",
		120,
		IT_HOLDABLE,
		HI_JETPACK,
/* precache */ "effects/boba/jet.efx",
/* sounds */ "sound/chars/boba/JETON.wav sound/chars/boba/JETHOVER.wav sound/effects/fire_lp.wav",
		"@MENUS_JETPACK_DESC"					// description
	},
	{
		"item_healthdisp",
		"sound/weapons/w_pkup.wav",
		{ "models/map_objects/mp/bacta.md3", //replace me
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_healthdisp",
/* pickup *///	"Sentry Gun",
		120,
		IT_HOLDABLE,
		HI_HEALTHDISP,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"item_ammodisp",
		"sound/weapons/w_pkup.wav",
		{ "models/map_objects/mp/bacta.md3", //replace me
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_ammodisp",
/* pickup *///	"Sentry Gun",
		120,
		IT_HOLDABLE,
		HI_AMMODISP,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"item_eweb_holdable",
		"sound/interface/shieldcon_empty",
		{ "models/map_objects/hoth/eweb_model.glm",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_eweb",
/* pickup *///	"Sentry Gun",
		120,
		IT_HOLDABLE,
		HI_EWEB,
/* precache */ "",
/* sounds */ "",
		"@MENUS_EWEB_DESC"					// description
	},
	{
		"item_cloak",
		"sound/weapons/w_pkup.wav",
		{ "models/items/psgun.glm", //FIXME: no model
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_cloak",
/* pickup *///	"Seeker Drone",
		120,
		IT_HOLDABLE,
		HI_CLOAK,
/* precache */ "",
/* sounds */ "",
		"@MENUS_CLOAK_DESC"					// description
	},
	{
		"item_force_enlighten_light",
		"sound/player/enlightenment.wav",
		{ "models/map_objects/mp/jedi_enlightenment.md3",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/mpi_jlight",
/* pickup *///	"Light Force Enlightenment",
		25,
		IT_POWERUP,
		PW_FORCE_ENLIGHTENED_LIGHT,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"item_force_enlighten_dark",
		"sound/player/enlightenment.wav",
		{ "models/map_objects/mp/dk_enlightenment.md3",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/mpi_dklight",
/* pickup *///	"Dark Force Enlightenment",
		25,
		IT_POWERUP,
		PW_FORCE_ENLIGHTENED_DARK,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"item_force_boon",
		"sound/player/boon.wav",
		{ "models/map_objects/mp/force_boon.md3",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/mpi_fboon",
/* pickup *///	"Force Boon",
		25,
		IT_POWERUP,
		PW_FORCE_BOON,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"item_ysalimari",
		"sound/player/ysalimari.wav",
		{ "models/map_objects/mp/ysalimari.md3",
		0, 0, 0} ,
/* view */		NULL,
/* icon */		"gfx/hud/mpi_ysamari",
/* pickup *///	"Ysalamiri",
		25,
		IT_POWERUP,
		PW_YSALAMIRI,
/* precache */ "",
/* sounds */ "",
		""					// description
	},

	//
	// WEAPONS
	//
	{
		"weapon_stun_baton",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/stun_baton/baton_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/stun_baton/baton.md3",
/* icon */		"gfx/hud/w_icon_stunbaton",
/* pickup *///	"Stun Baton",
		100,
		IT_WEAPON,
		WP_STUN_BATON,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"weapon_melee",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/stun_baton/baton_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/stun_baton/baton.md3",
/* icon */		"gfx/hud/w_icon_melee",
/* pickup *///	"Stun Baton",
		100,
		IT_WEAPON,
		WP_MELEE,
/* precache */ "",
/* sounds */ "",
		"@MENUS_MELEE_DESC"					// description
	},
	{
		"weapon_saber",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/saber/saber_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/saber/saber_w.md3",
/* icon */		"gfx/hud/w_icon_lightsaber",
/* pickup *///	"Lightsaber",
		100,
		IT_WEAPON,
		WP_SABER,
/* precache */ "",
/* sounds */ "",
		"@MENUS_AN_ELEGANT_WEAPON_FOR"				// description
	},
	{
		//"weapon_bryar_pistol",
		"weapon_blaster_pistol",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/blaster_pistol/blaster_pistol_w.glm",//"models/weapons2/briar_pistol/briar_pistol_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/blaster_pistol/blaster_pistol.md3",//"models/weapons2/briar_pistol/briar_pistol.md3",
/* icon */		"gfx/hud/w_icon_blaster_pistol",//"gfx/hud/w_icon_rifle",
/* pickup *///	"Bryar Pistol",
		100,
		IT_WEAPON,
		WP_BRYAR_PISTOL,
/* precache */ "",
/* sounds */ "",
		"@MENUS_BLASTER_PISTOL_DESC"					// description
	},
	{
		"weapon_concussion_rifle",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/concussion/c_rifle_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/concussion/c_rifle.md3",
/* icon */		"gfx/hud/w_icon_c_rifle",//"gfx/hud/w_icon_rifle",
/* pickup *///	"Concussion Rifle",
		50,
		IT_WEAPON,
		WP_CONCUSSION,
/* precache */ "",
/* sounds */ "",
		"@MENUS_CONC_RIFLE_DESC"					// description
	},
	{
		"weapon_bryar_pistol",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/briar_pistol/briar_pistol_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/briar_pistol/briar_pistol.md3",
/* icon */		"gfx/hud/w_icon_briar",//"gfx/hud/w_icon_rifle",
/* pickup *///	"Bryar Pistol",
		100,
		IT_WEAPON,
		WP_BRYAR_OLD,
/* precache */ "",
/* sounds */ "",
		"@SP_INGAME_BLASTER_PISTOL"					// description
	},
	{
		"weapon_blaster",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/blaster_r/blaster_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/blaster_r/blaster.md3",
/* icon */		"gfx/hud/w_icon_blaster",
/* pickup *///	"E11 Blaster Rifle",
		100,
		IT_WEAPON,
		WP_BLASTER,
/* precache */ "",
/* sounds */ "",
		"@MENUS_THE_PRIMARY_WEAPON_OF"				// description
	},
	{
		"weapon_disruptor",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/disruptor/disruptor_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/disruptor/disruptor.md3",
/* icon */		"gfx/hud/w_icon_disruptor",
/* pickup *///	"Tenloss Disruptor Rifle",
		100,
		IT_WEAPON,
		WP_DISRUPTOR,
/* precache */ "",
/* sounds */ "",
		"@MENUS_THIS_NEFARIOUS_WEAPON"					// description
	},
	{
		"weapon_bowcaster",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/bowcaster/bowcaster_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/bowcaster/bowcaster.md3",
/* icon */		"gfx/hud/w_icon_bowcaster",
/* pickup *///	"Wookiee Bowcaster",
		100,
		IT_WEAPON,
		WP_BOWCASTER,
/* precache */ "",
/* sounds */ "",
		"@MENUS_THIS_ARCHAIC_LOOKING"					// description
	},
	{
		"weapon_repeater",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/heavy_repeater/heavy_repeater_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/heavy_repeater/heavy_repeater.md3",
/* icon */		"gfx/hud/w_icon_repeater",
/* pickup *///	"Imperial Heavy Repeater",
		100,
		IT_WEAPON,
		WP_REPEATER,
/* precache */ "",
/* sounds */ "",
		"@MENUS_THIS_DESTRUCTIVE_PROJECTILE"					// description
	},
	{
		"weapon_demp2",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/demp2/demp2_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/demp2/demp2.md3",
/* icon */		"gfx/hud/w_icon_demp2",
/* pickup *///	"DEMP2",
		100,
		IT_WEAPON,
		WP_DEMP2,
/* precache */ "",
/* sounds */ "",
		"@MENUS_COMMONLY_REFERRED_TO"					// description
	},
	{
		"weapon_flechette",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/golan_arms/golan_arms_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/golan_arms/golan_arms.md3",
/* icon */		"gfx/hud/w_icon_flechette",
/* pickup *///	"Golan Arms Flechette",
		100,
		IT_WEAPON,
		WP_FLECHETTE,
/* precache */ "",
/* sounds */ "",
		"@MENUS_WIDELY_USED_BY_THE_CORPORATE"					// description
	},
	{
		"weapon_rocket_launcher",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/merr_sonn/merr_sonn_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/merr_sonn/merr_sonn.md3",
/* icon */		"gfx/hud/w_icon_merrsonn",
/* pickup *///	"Merr-Sonn Missile System",
		3,
		IT_WEAPON,
		WP_ROCKET_LAUNCHER,
/* precache */ "",
/* sounds */ "",
		"@MENUS_THE_PLX_2M_IS_AN_EXTREMELY"					// description
	},
	{
		"ammo_thermal",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/thermal/thermal_pu.md3",
		"models/weapons2/thermal/thermal_w.glm", 0, 0},
/* view */		"models/weapons2/thermal/thermal.md3",
/* icon */		"gfx/hud/w_icon_thermal",
/* pickup *///	"Thermal Detonators",
		4,
		IT_AMMO,
		AMMO_THERMAL,
/* precache */ "",
/* sounds */ "",
		"@MENUS_THE_THERMAL_DETONATOR"					// description
	},
	{
		"ammo_tripmine",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/laser_trap/laser_trap_pu.md3",
		"models/weapons2/laser_trap/laser_trap_w.glm", 0, 0},
/* view */		"models/weapons2/laser_trap/laser_trap.md3",
/* icon */		"gfx/hud/w_icon_tripmine",
/* pickup *///	"Trip Mines",
		3,
		IT_AMMO,
		AMMO_TRIPMINE,
/* precache */ "",
/* sounds */ "",
		"@MENUS_TRIP_MINES_CONSIST_OF"					// description
	},
	{
		"ammo_detpack",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/detpack/det_pack_pu.md3", "models/weapons2/detpack/det_pack_proj.glm", "models/weapons2/detpack/det_pack_w.glm", 0},
/* view */		"models/weapons2/detpack/det_pack.md3",
/* icon */		"gfx/hud/w_icon_detpack",
/* pickup *///	"Det Packs",
		3,
		IT_AMMO,
		AMMO_DETPACK,
/* precache */ "",
/* sounds */ "",
		"@MENUS_A_DETONATION_PACK_IS"					// description
	},
	{
		"weapon_thermal",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/thermal/thermal_w.glm", "models/weapons2/thermal/thermal_pu.md3",
		0, 0 },
/* view */		"models/weapons2/thermal/thermal.md3",
/* icon */		"gfx/hud/w_icon_thermal",
/* pickup *///	"Thermal Detonator",
		4,
		IT_WEAPON,
		WP_THERMAL,
/* precache */ "",
/* sounds */ "",
		"@MENUS_THE_THERMAL_DETONATOR"					// description
	},
	{
		"weapon_trip_mine",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/laser_trap/laser_trap_w.glm", "models/weapons2/laser_trap/laser_trap_pu.md3",
		0, 0},
/* view */		"models/weapons2/laser_trap/laser_trap.md3",
/* icon */		"gfx/hud/w_icon_tripmine",
/* pickup *///	"Trip Mine",
		3,
		IT_WEAPON,
		WP_TRIP_MINE,
/* precache */ "",
/* sounds */ "",
		"@MENUS_TRIP_MINES_CONSIST_OF"					// description
	},
	{
		"weapon_det_pack",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/detpack/det_pack_proj.glm", "models/weapons2/detpack/det_pack_pu.md3", "models/weapons2/detpack/det_pack_w.glm", 0},
/* view */		"models/weapons2/detpack/det_pack.md3",
/* icon */		"gfx/hud/w_icon_detpack",
/* pickup *///	"Det Pack",
		3,
		IT_WEAPON,
		WP_DET_PACK,
/* precache */ "",
/* sounds */ "",
		"@MENUS_A_DETONATION_PACK_IS"					// description
	},
	{
		"weapon_emplaced",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/blaster_r/blaster_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/blaster_r/blaster.md3",
/* icon */		"gfx/hud/w_icon_blaster",
/* pickup *///	"Emplaced Gun",
		50,
		IT_WEAPON,
		WP_EMPLACED_GUN,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
//NOTE: This is to keep things from messing up because the turret weapon type isn't real
	{
		"weapon_turretwp",
		"sound/weapons/w_pkup.wav",
        { "models/weapons2/blaster_r/blaster_w.glm",
		0, 0, 0},
/* view */		"models/weapons2/blaster_r/blaster.md3",
/* icon */		"gfx/hud/w_icon_blaster",
/* pickup *///	"Turret Gun",
		50,
		IT_WEAPON,
		WP_TURRET,
/* precache */ "",
/* sounds */ "",
		""					// description
	},

	//
	// AMMO ITEMS
	//
	{
		"ammo_force",
		"sound/player/pickupenergy.wav",
        { "models/items/energy_cell.md3",
		0, 0, 0},
/* view */		NULL,
/* icon */		"gfx/hud/w_icon_blaster",
/* pickup *///	"Force??",
		100,
		IT_AMMO,
		AMMO_FORCE,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"ammo_blaster",
		"sound/player/pickupenergy.wav",
        { "models/items/energy_cell.md3",
		0, 0, 0},
/* view */		NULL,
/* icon */		"gfx/hud/i_icon_battery",
/* pickup *///	"Blaster Pack",
		100,
		IT_AMMO,
		AMMO_BLASTER,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"ammo_powercell",
		"sound/player/pickupenergy.wav",
        { "models/items/power_cell.md3",
		0, 0, 0},
/* view */		NULL,
/* icon */		"gfx/mp/ammo_power_cell",
/* pickup *///	"Power Cell",
		100,
		IT_AMMO,
		AMMO_POWERCELL,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"ammo_metallic_bolts",
		"sound/player/pickupenergy.wav",
        { "models/items/metallic_bolts.md3",
		0, 0, 0},
/* view */		NULL,
/* icon */		"gfx/mp/ammo_metallic_bolts",
/* pickup *///	"Metallic Bolts",
		100,
		IT_AMMO,
		AMMO_METAL_BOLTS,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"ammo_rockets",
		"sound/player/pickupenergy.wav",
        { "models/items/rockets.md3",
		0, 0, 0},
/* view */		NULL,
/* icon */		"gfx/mp/ammo_rockets",
/* pickup *///	"Rockets",
		3,
		IT_AMMO,
		AMMO_ROCKETS,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"ammo_all",
		"sound/player/pickupenergy.wav",
        { "models/items/battery.md3",  //replace me
		0, 0, 0},
/* view */		NULL,
/* icon */		"gfx/mp/ammo_rockets", //replace me
/* pickup *///	"Rockets",
		0,
		IT_AMMO,
		-1,
/* precache */ "",
/* sounds */ "",
		""					// description
	},

	//
	// POWERUP ITEMS
	//
	{
		"team_CTF_redflag",
		NULL,
        { "models/flags/r_flag.md3",
		"models/flags/r_flag_ysal.md3", 0, 0 },
/* view */		NULL,
/* icon */		"gfx/hud/mpi_rflag",
/* pickup *///	"Red Flag",
		0,
		IT_TEAM,
		PW_REDFLAG,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"team_CTF_blueflag",
		NULL,
        { "models/flags/b_flag.md3",
		"models/flags/b_flag_ysal.md3", 0, 0 },
/* view */		NULL,
/* icon */		"gfx/hud/mpi_bflag",
/* pickup *///	"Blue Flag",
		0,
		IT_TEAM,
		PW_BLUEFLAG,
/* precache */ "",
/* sounds */ "",
		""					// description
	},

	//
	// PERSISTANT POWERUP ITEMS
	//
	{
		"team_CTF_neutralflag",
		NULL,
        { "models/flags/n_flag.md3",
		0, 0, 0 },
/* view */		NULL,
/* icon */		"icons/iconf_neutral1",
/* pickup *///	"Neutral Flag",
		0,
		IT_TEAM,
		PW_NEUTRALFLAG,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"item_redcube",
		"sound/player/pickupenergy.wav",
        { "models/powerups/orb/r_orb.md3",
		0, 0, 0 },
/* view */		NULL,
/* icon */		"icons/iconh_rorb",
/* pickup *///	"Red Cube",
		0,
		IT_TEAM,
		0,
/* precache */ "",
/* sounds */ "",
		""					// description
	},
	{
		"item_bluecube",
		"sound/player/pickupenergy.wav",
        { "models/powerups/orb/b_orb.md3",
		0, 0, 0 },
/* view */		NULL,
/* icon */		"icons/iconh_borb",
/* pickup *///	"Blue Cube",
		0,
		IT_TEAM,
		0,
/* precache */ "",
/* sounds */ "",
		""					// description
	},

	// end of list marker
	{NULL}
};

int bg_numItems = sizeof(bg_itemlist) / sizeof(bg_itemlist[0]) - 1;

/* ===== minimal playerState_t + stubs for the selectors/finders ===== */
typedef struct {
	int stats[MAX_STATS];
} ps_item_min_t;
#define playerState_t ps_item_min_t

/* Com_Error / Q_stricmp the verbatim finder bodies reference. Com_Error aborts (it
 * is `-> noreturn` in JKA); the test only calls finders with valid tags, so it never
 * fires. Q_stricmp matches the JKA semantics used by BG_FindItem (0 == equal). */
static void Com_Error(int level, const char *error, ...) { (void)level; (void)error; abort(); }
static int Q_stricmp(const char *s1, const char *s2) {
	while (*s1 && *s2) {
		int c1 = *s1++, c2 = *s2++;
		if (c1 >= 'A' && c1 <= 'Z') c1 += 'a' - 'A';
		if (c2 >= 'A' && c2 <= 'Z') c2 += 'a' - 'A';
		if (c1 != c2) return c1 < c2 ? -1 : 1;
	}
	if (*s1 == *s2) return 0;
	return *s1 < *s2 ? -1 : 1;
}

/* ===== verbatim BG_* bodies (bg_misc.c) ===== */
int BG_GetItemIndexByTag(int tag, int type)
{ //Get the itemlist index from the tag and type
	int i = 0;

	while (i < bg_numItems)
	{
		if (bg_itemlist[i].giTag == tag &&
			bg_itemlist[i].giType == type)
		{
			return i;
		}

		i++;
	}

	return 0;
}

qboolean BG_IsItemSelectable(playerState_t *ps, int item)
{
	if (item == HI_HEALTHDISP || item == HI_AMMODISP ||
		item == HI_JETPACK)
	{
		return qfalse;
	}
	return qtrue;
}

void BG_CycleInven(playerState_t *ps, int direction)
{
	int i;
	int dontFreeze = 0;
	int original;

	i = bg_itemlist[ps->stats[STAT_HOLDABLE_ITEM]].giTag;
	original = i;

	if (direction == 1)
	{ //next
		i++;
		if (i == HI_NUM_HOLDABLE)
		{
			i = 1;
		}
	}
	else
	{ //previous
		i--;
		if (i == 0)
		{
			i = HI_NUM_HOLDABLE-1;
		}
	}

	while (i != original)
	{ //go in a full loop until hitting something, if hit nothing then select nothing
		if (ps->stats[STAT_HOLDABLE_ITEMS] & (1 << i))
		{ //we have it, select it.
			if (BG_IsItemSelectable(ps, i))
			{
				ps->stats[STAT_HOLDABLE_ITEM] = BG_GetItemIndexByTag(i, IT_HOLDABLE);
				break;
			}
		}

		if (direction == 1)
		{ //next
			i++;
		}
		else
		{ //previous
			i--;
		}

		if (i <= 0)
		{ //wrap around to the last
			i = HI_NUM_HOLDABLE-1;
		}
		else if (i >= HI_NUM_HOLDABLE)
		{ //wrap around to the first
			i = 1;
		}

		dontFreeze++;
		if (dontFreeze >= 32)
		{ //yeah, sure, whatever (it's 2 am and I'm paranoid and can't frickin think)
			break;
		}
	}
}

gitem_t	*BG_FindItemForPowerup( int pw ) {
	int		i;

	for ( i = 0 ; i < bg_numItems ; i++ ) {
		if ( (bg_itemlist[i].giType == IT_POWERUP ||
					bg_itemlist[i].giType == IT_TEAM) &&
			bg_itemlist[i].giTag == pw ) {
			return &bg_itemlist[i];
		}
	}

	return NULL;
}

gitem_t	*BG_FindItemForHoldable( int pw ) {
	int		i;

	for ( i = 0 ; i < bg_numItems ; i++ ) {
		if ( bg_itemlist[i].giType == IT_HOLDABLE && bg_itemlist[i].giTag == pw ) {
			return &bg_itemlist[i];
		}
	}

	Com_Error( ERR_DROP, "HoldableItem not found" );

	return NULL;
}

gitem_t	*BG_FindItemForWeapon( int weapon ) {
	gitem_t	*it;

	for ( it = bg_itemlist + 1 ; it->classname ; it++) {
		if ( it->giType == IT_WEAPON && it->giTag == weapon ) {
			return it;
		}
	}

	Com_Error( ERR_DROP, "Couldn't find item for weapon %i", weapon);
	return NULL;
}

gitem_t	*BG_FindItemForAmmo( int ammo ) {
	gitem_t	*it;

	for ( it = bg_itemlist + 1 ; it->classname ; it++) {
		if ( it->giType == IT_AMMO && it->giTag == ammo ) {
			return it;
		}
	}

	Com_Error( ERR_DROP, "Couldn't find item for ammo %i", ammo);
	return NULL;
}

gitem_t	*BG_FindItem( const char *classname ) {
	gitem_t	*it;

	for ( it = bg_itemlist + 1 ; it->classname ; it++ ) {
		if ( !Q_stricmp( it->classname, classname) )
			return it;
	}

	return NULL;
}

#undef playerState_t

/* ===== marshalling wrappers ===== */

const gitem_t *jka_bgitem_itemlist_ptr(void) { return bg_itemlist; }
int jka_bgitem_numItems(void) { return bg_numItems; }

int jka_bgitem_GetItemIndexByTag(int tag, int type) {
	return BG_GetItemIndexByTag(tag, type);
}

int jka_bgitem_IsItemSelectable(int item) {
	return BG_IsItemSelectable((ps_item_min_t *)0, item);
}

/* Load (STAT_HOLDABLE_ITEM, STAT_HOLDABLE_ITEMS), run BG_CycleInven, read both back. */
void jka_bgitem_CycleInven(int direction, int *io_holdableItem, int *io_holdableItems) {
	ps_item_min_t ps;
	int i;
	for (i = 0; i < MAX_STATS; i++) ps.stats[i] = 0;
	ps.stats[STAT_HOLDABLE_ITEM] = *io_holdableItem;
	ps.stats[STAT_HOLDABLE_ITEMS] = *io_holdableItems;
	BG_CycleInven(&ps, direction);
	*io_holdableItem = ps.stats[STAT_HOLDABLE_ITEM];
	*io_holdableItems = ps.stats[STAT_HOLDABLE_ITEMS];
}

/* Finders return the bg_itemlist INDEX of the hit (-1 if NULL), so the result crosses
 * the FFI boundary as an int rather than a pointer into the C table copy. */
int jka_bgitem_FindItemForPowerup(int pw) {
	gitem_t *it = BG_FindItemForPowerup(pw);
	return it ? (int)(it - bg_itemlist) : -1;
}
int jka_bgitem_FindItemForHoldable(int pw) {
	gitem_t *it = BG_FindItemForHoldable(pw);
	return it ? (int)(it - bg_itemlist) : -1;
}
int jka_bgitem_FindItemForWeapon(int weapon) {
	gitem_t *it = BG_FindItemForWeapon(weapon);
	return it ? (int)(it - bg_itemlist) : -1;
}
int jka_bgitem_FindItemForAmmo(int ammo) {
	gitem_t *it = BG_FindItemForAmmo(ammo);
	return it ? (int)(it - bg_itemlist) : -1;
}
int jka_bgitem_FindItem(const char *classname) {
	gitem_t *it = BG_FindItem(classname);
	return it ? (int)(it - bg_itemlist) : -1;
}

/* ============================================================================
 * BG_CanItemBeGrabbed (bg_misc.c:2192) -- pickup eligibility. Reuses the real
 * bg_itemlist above (the candidate item is bg_itemlist[ent->modelindex]); the
 * ps/ent fields it reads and the weaponData/ammoData ammo-cap columns are kept
 * as minimal locals (no full struct ever crosses the FFI -- the wrapper marshals
 * scalars + the four ps arrays in). The constants below are the JKA values not
 * already #defined above for the item selectors. */

#define STAT_HEALTH 0
#define STAT_WEAPONS 4
#define STAT_ARMOR 5
#define STAT_MAX_HEALTH 8
#define EF_DROPPEDWEAPON (1 << 25)
#define GT_CTF 8
#define GT_CTY 9
#define PERS_TEAM 3
#define TEAM_RED 1
#define TEAM_BLUE 2
#define FP_RAGE 8
#define MAX_WEAPONS 19
#define MAX_POWERUPS 16
#define MAX_PERSISTANT 16

/* Only the ammoIndex and max columns are read; transcribed from bg_weapons.c
 * (these must match the Rust bg_weapons tables for every tested row). Indices
 * 0..18 are WP_NONE..WP_TURRET (ammoIndex = an AMMO_* value), 0..9 are
 * AMMO_NONE..AMMO_DETPACK (max). */
typedef struct { int ammoIndex; } wd_grab_t;
typedef struct { int max; } ad_grab_t;
static wd_grab_t weaponData[MAX_WEAPONS] = {
	{0}, {0}, {0}, {0}, {2}, {2}, {3}, {3}, {4}, {3},
	{4}, {5}, {7}, {8}, {9}, {4}, {2}, {0}, {0},
};
static ad_grab_t ammoData[10] = {
	{0}, {100}, {300}, {300}, {300}, {25}, {800}, {10}, {10}, {10},
};

/* Debug-only diagnostic in the default switch arm (#ifndef Q3_VM / #ifndef
 * NDEBUG). Never reached by the tests (valid in-range items only). */
#define Com_Printf(...) ((void)0)

typedef struct { int forcePowersActive; } fd_grab_t;
typedef struct {
	qboolean trueJedi;
	qboolean trueNonJedi;
	qboolean isJediMaster;
	qboolean duelInProgress;
	int clientNum;
	int stats[MAX_STATS];
	int persistant[MAX_PERSISTANT];
	int ammo[MAX_WEAPONS];
	int powerups[MAX_POWERUPS];
	fd_grab_t fd;
} ps_grab_min_t;
typedef struct {
	int modelindex;
	int modelindex2;
	int generic1;
	int powerups;
	int eFlags;
} es_grab_min_t;

#define playerState_t ps_grab_min_t
#define entityState_t es_grab_min_t

/* ----- verbatim BG_CanItemBeGrabbed body (bg_misc.c:2192) ----- */
qboolean BG_CanItemBeGrabbed( int gametype, const entityState_t *ent, const playerState_t *ps ) {
	gitem_t	*item;

	if ( ent->modelindex < 1 || ent->modelindex >= bg_numItems ) {
		Com_Error( ERR_DROP, "BG_CanItemBeGrabbed: index out of range" );
	}

	item = &bg_itemlist[ent->modelindex];

	if ( ps )
	{
		if ( ps->trueJedi )
		{//force powers and saber only
			if ( item->giType != IT_TEAM //not a flag
				&& item->giType != IT_ARMOR//not shields
				&& (item->giType != IT_WEAPON || item->giTag != WP_SABER)//not a saber
				&& (item->giType != IT_HOLDABLE || item->giTag != HI_SEEKER)//not a seeker
				&& (item->giType != IT_POWERUP || item->giTag == PW_YSALAMIRI) )//not a force pick-up
			{
				return qfalse;
			}
		}
		else if ( ps->trueNonJedi )
		{//can't pick up force powerups
			if ( (item->giType == IT_POWERUP && item->giTag != PW_YSALAMIRI) //if a powerup, can only can pick up ysalamiri
				|| (item->giType == IT_HOLDABLE && item->giTag == HI_SEEKER)//if holdable, cannot pick up seeker
				|| (item->giType == IT_WEAPON && item->giTag == WP_SABER ) )//or if it's a saber
			{
				return qfalse;
			}
		}
		if ( ps->isJediMaster && item && (item->giType == IT_WEAPON || item->giType == IT_AMMO))
		{//jedi master cannot pick up weapons
			return qfalse;
		}
		if ( ps->duelInProgress )
		{ //no picking stuff up while in a duel, no matter what the type is
			return qfalse;
		}
	}
	else
	{//safety return since below code assumes a non-null ps
		return qfalse;
	}

	switch( item->giType ) {
	case IT_WEAPON:
		if (ent->generic1 == ps->clientNum && ent->powerups)
		{
			return qfalse;
		}
		if (!(ent->eFlags & EF_DROPPEDWEAPON) && (ps->stats[STAT_WEAPONS] & (1 << item->giTag)) &&
			item->giTag != WP_THERMAL && item->giTag != WP_TRIP_MINE && item->giTag != WP_DET_PACK)
		{ //weaponstay stuff.. if this isn't dropped, and you already have it, you don't get it.
			return qfalse;
		}
		if (item->giTag == WP_THERMAL || item->giTag == WP_TRIP_MINE || item->giTag == WP_DET_PACK)
		{ //check to see if full on ammo for this, if so, then..
			int ammoIndex = weaponData[item->giTag].ammoIndex;
			if (ps->ammo[ammoIndex] >= ammoData[ammoIndex].max)
			{ //don't need it
				return qfalse;
			}
		}
		return qtrue;	// weapons are always picked up

	case IT_AMMO:
		if (item->giTag == -1)
		{ //special case for "all ammo" packs
			return qtrue;
		}
		if ( ps->ammo[item->giTag] >= ammoData[item->giTag].max) {
			return qfalse;		// can't hold any more
		}
		return qtrue;

	case IT_ARMOR:
		if ( ps->stats[STAT_ARMOR] >= ps->stats[STAT_MAX_HEALTH]/* * item->giTag*/ ) {
			return qfalse;
		}
		return qtrue;

	case IT_HEALTH:
		// small and mega healths will go over the max, otherwise
		// don't pick up if already at max
		if ((ps->fd.forcePowersActive & (1 << FP_RAGE)))
		{
			return qfalse;
		}

		if ( item->quantity == 5 || item->quantity == 100 ) {
			if ( ps->stats[STAT_HEALTH] >= ps->stats[STAT_MAX_HEALTH] * 2 ) {
				return qfalse;
			}
			return qtrue;
		}

		if ( ps->stats[STAT_HEALTH] >= ps->stats[STAT_MAX_HEALTH] ) {
			return qfalse;
		}
		return qtrue;

	case IT_POWERUP:
		if (ps && (ps->powerups[PW_YSALAMIRI]))
		{
			if (item->giTag != PW_YSALAMIRI)
			{
				return qfalse;
			}
		}
		return qtrue;	// powerups are always picked up

	case IT_TEAM: // team items, such as flags
		if( gametype == GT_CTF || gametype == GT_CTY ) {
			// ent->modelindex2 is non-zero on items if they are dropped
			// we need to know this because we can pick up our dropped flag (and return it)
			// but we can't pick up our flag at base
			if (ps->persistant[PERS_TEAM] == TEAM_RED) {
				if (item->giTag == PW_BLUEFLAG ||
					(item->giTag == PW_REDFLAG && ent->modelindex2) ||
					(item->giTag == PW_REDFLAG && ps->powerups[PW_BLUEFLAG]) )
					return qtrue;
			} else if (ps->persistant[PERS_TEAM] == TEAM_BLUE) {
				if (item->giTag == PW_REDFLAG ||
					(item->giTag == PW_BLUEFLAG && ent->modelindex2) ||
					(item->giTag == PW_BLUEFLAG && ps->powerups[PW_REDFLAG]) )
					return qtrue;
			}
		}

		return qfalse;

	case IT_HOLDABLE:
		if ( ps->stats[STAT_HOLDABLE_ITEMS] & (1 << item->giTag))
		{
			return qfalse;
		}
		return qtrue;

        case IT_BAD:
            Com_Error( ERR_DROP, "BG_CanItemBeGrabbed: IT_BAD" );
        default:
#ifndef Q3_VM
#ifndef NDEBUG // bk0001204
          Com_Printf("BG_CanItemBeGrabbed: unknown enum %d\n", item->giType );
#endif
#endif
         break;
	}

	return qfalse;
}

#undef playerState_t
#undef entityState_t

/* Marshal scalars + the four ps arrays into the minimal structs, set ps = NULL
 * when !has_ps, run the verbatim body, return its qboolean. */
int jka_bgitem_can_item_be_grabbed(
	int gametype, int modelindex, int has_ps,
	int trueJedi, int trueNonJedi, int isJediMaster, int duelInProgress, int clientNum,
	int forcePowersActive,
	int generic1, int es_powerups, int eFlags, int modelindex2,
	const int *stats, const int *persistant, const int *ammo, const int *powerups)
{
	ps_grab_min_t ps;
	es_grab_min_t es;
	int i;

	es.modelindex = modelindex;
	es.modelindex2 = modelindex2;
	es.generic1 = generic1;
	es.powerups = es_powerups;
	es.eFlags = eFlags;

	ps.trueJedi = trueJedi;
	ps.trueNonJedi = trueNonJedi;
	ps.isJediMaster = isJediMaster;
	ps.duelInProgress = duelInProgress;
	ps.clientNum = clientNum;
	ps.fd.forcePowersActive = forcePowersActive;
	for (i = 0; i < MAX_STATS; i++) ps.stats[i] = stats[i];
	for (i = 0; i < MAX_PERSISTANT; i++) ps.persistant[i] = persistant[i];
	for (i = 0; i < MAX_WEAPONS; i++) ps.ammo[i] = ammo[i];
	for (i = 0; i < MAX_POWERUPS; i++) ps.powerups[i] = powerups[i];

	return BG_CanItemBeGrabbed(gametype, &es, has_ps ? &ps : (ps_grab_min_t *)0);
}
