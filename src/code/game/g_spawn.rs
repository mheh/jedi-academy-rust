// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"
//
// #include "Q3_Interface.h"
// #include "g_local.h"
// #include "g_functions.h"

use core::ffi::{c_char, c_int, c_void};
use core::mem;
use core::ptr;

// extern cvar_t *g_spskill;
// extern cvar_t *g_delayedShutdown;
extern "C" {
    static mut g_spskill: *mut cvar_t;
    static mut g_delayedShutdown: *mut cvar_t;
    static mut com_buildScript: *mut cvar_t;
    static g_eSavedGameJustLoaded: SavedGameJustLoaded_e;
}

// these vars I moved here out of the level_locals_t struct simply because it's pointless to try saving them,
//	and the level_locals_t struct is included in the save process... -slc
//
pub static mut spawning: qboolean = qfalse;	// the G_Spawn*() functions are valid  (only turned on during one function)
pub static mut numSpawnVars: c_int = 0;
pub static mut spawnVars: [[*mut c_char; 2]; MAX_SPAWN_VARS as usize] = [[ptr::null_mut(); 2]; MAX_SPAWN_VARS as usize];	// key / value pairs
pub static mut numSpawnVarChars: c_int = 0;
pub static mut spawnVarChars: [c_char; MAX_SPAWN_VARS_CHARS as usize] = [0; MAX_SPAWN_VARS_CHARS as usize];

pub static mut delayedShutDown: c_int = 0;

// #include "../qcommon/sstring.h"
//
//NOTENOTE: Be sure to change the mirrored code in cgmain.cpp
// typedef	map< sstring_t, unsigned char, less<sstring_t>, allocator< unsigned char >  >	namePrecache_m;
pub static mut as_preCacheMap: *mut namePrecache_m = ptr::null_mut();

// Forward declarations
extern "C" {
    pub fn G_Spawn() -> *mut gentity_t;
    pub fn G_FreeEntity(ent: *mut gentity_t);
    pub fn G_Alloc(size: c_int) -> *mut c_void;
    pub fn G_SpawnItem(ent: *mut gentity_t, item: *mut gitem_t);
    pub fn G_Error(fmt: *const c_char, ...);
    pub fn Quake3Game() -> *mut IGameInterface;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_strncmp(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn atoi(s: *const c_char) -> c_int;
    pub fn atof(s: *const c_char) -> f32;
    pub fn sscanf(s: *const c_char, fmt: *const c_char, ...) -> c_int;
    pub fn GetIDForString(table: *const stringID_table_t, string: *const c_char) -> c_int;
    pub fn COM_Parse(data: *mut *const c_char) -> *const c_char;
    pub fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize);
    pub fn VectorCopy(src: *const f32, dest: *mut f32);
    pub fn VectorAdd(src1: *const f32, src2: *const f32, dest: *mut f32);
    pub fn G_ActivateBehavior(ent: *mut gentity_t, bset: c_int);
    pub fn Q3_SetParm(entID: c_int, parmNum: c_int, parmValue: *const c_char);
}

extern "C" {
    static mut g_entities: [gentity_t; MAX_GENTITIES as usize];
    static mut level: level_locals_t;
    static mut globals: gclient_s;
    static mut bg_itemlist: gitem_t;
}

// Type definitions and constants
pub type qboolean = c_int;
pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

pub const MAX_SPAWN_VARS: c_int = 64;
pub const MAX_SPAWN_VARS_CHARS: c_int = 4096;
pub const MAX_GENTITIES: c_int = 4096;
pub const MAX_STRING_CHARS: c_int = 1024;
pub const LS_NUM_STYLES: c_int = 32;
pub const LS_STYLES_START: c_int = 0;
pub const CS_MUSIC: c_int = 0;		// stub constants
pub const CS_MESSAGE: c_int = 1;
pub const CS_LIGHT_STYLES: c_int = 2;
pub const CS_AMBIENT_SET: c_int = 3;

pub const ENTITYNUM_WORLD: usize = 0;

pub const BSET_SPAWN: c_int = 0;
pub const BSET_USE: c_int = 1;
pub const BSET_AWAKE: c_int = 2;
pub const BSET_ANGER: c_int = 3;
pub const BSET_ATTACK: c_int = 4;
pub const BSET_VICTORY: c_int = 5;
pub const BSET_LOSTENEMY: c_int = 6;
pub const BSET_PAIN: c_int = 7;
pub const BSET_FLEE: c_int = 8;
pub const BSET_DEATH: c_int = 9;
pub const BSET_DELAYED: c_int = 10;
pub const BSET_BLOCKED: c_int = 11;
pub const BSET_FFIRE: c_int = 12;
pub const BSET_FFDEATH: c_int = 13;
pub const BSET_MINDTRICK: c_int = 14;

pub const ERR_DROP: c_int = 1;

pub const S_COLOR_RED: &[u8] = b"^1";
pub const S_COLOR_YELLOW: &[u8] = b"^3";

pub const FINAL_BUILD: bool = false;

// FOFS macro - field offset in structure
// In C: #define FOFS(x) ((int)&(((gentity_t *)0)->x))
#[inline]
pub fn FOFS(offset: usize) -> c_int {
    offset as c_int
}

// Stub types - these would be defined elsewhere in a complete port
#[repr(C)]
pub struct cvar_t {
    // stub
}

#[repr(C)]
pub struct gentity_t {
    // stub - represents the entity structure
    pub s: entityState_t,
    pub classname: *mut c_char,
    pub spawnflags: c_int,
    pub max_health: c_int,
    pub currentOrigin: [f32; 3],
    pub currentAngles: [f32; 3],
    pub soundSet: *const c_char,
    pub behaviorSet: [*mut c_char; 16],
    pub count: c_int,
    pub e_ThinkFunc: Option<unsafe extern "C" fn(*mut c_void)>,
    pub nextthink: c_int,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub model: *mut c_char,
    pub model2: *mut c_char,
    pub target: *mut c_char,
    pub target2: *mut c_char,
    pub target3: *mut c_char,
    pub target4: *mut c_char,
    pub targetJump: *mut c_char,
    pub targetname: *mut c_char,
    pub material: c_int,
    pub message: *mut c_char,
    pub team: *mut c_char,
    pub wait: f32,
    pub random: f32,
    pub delay: c_int,
    pub sounds: c_int,
    pub damage: c_int,
    pub modelAngles: [f32; 3],
    pub cameraGroup: *mut c_char,
    pub radius: f32,
    pub endFrame: c_int,
    pub alt_fire: c_int,
    pub paintarget: *mut c_char,
    pub closetarget: *mut c_char,
    pub opentarget: *mut c_char,
    pub startRGBA: [f32; 4],
    pub finalRGBA: [f32; 4],
    pub mass: f32,
    pub splashDamage: c_int,
    pub splashRadius: c_int,
    pub delayScriptTime: c_int,
    pub script_targetname: *mut c_char,
    pub NPC_targetname: *mut c_char,
    pub NPC_target: *mut c_char,
    pub NPC_type: *mut c_char,
    pub ownername: *mut c_char,
    pub loopAnim: c_int,
    pub fxFile: *mut c_char,
    // ... many more fields would be here in a complete port
}

#[repr(C)]
pub struct entityState_t {
    // stub
    pub number: c_int,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub pos: trType_t,
    pub apos: trType_t,
    pub radius: f32,
    // ... many more fields
}

#[repr(C)]
pub struct trType_t {
    pub trBase: [f32; 3],
    // stub
}

#[repr(C)]
pub struct gitem_t {
    pub classname: *const c_char,
    // stub
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
    // stub - represents level state
}

#[repr(C)]
pub struct gclient_s {
    pub num_entities: c_int,
    // stub
}

#[repr(C)]
pub struct IGameInterface {
    // stub
}

// Methods on IGameInterface accessed via C linkage
extern "C" {
    pub fn IGameInterface_ValidEntity(this: *mut IGameInterface, ent: *mut gentity_t) -> c_int;
    pub fn IGameInterface_InitEntity(this: *mut IGameInterface, ent: *mut gentity_t);
}

#[repr(C)]
pub struct namePrecache_m {
    // stub - STL map type
}

// stringID_table_t definition
#[repr(C)]
pub struct stringID_table_t {
    pub name: *const c_char,
    pub id: *const c_void,
}

//
// fields are needed for spawning from the entity string
//
#[repr(C)]
#[derive(Copy, Clone)]
pub enum fieldtype_t {
    F_INT,
    F_FLOAT,
    F_LSTRING,			// string on disk, pointer in memory, TAG_LEVEL
    F_GSTRING,			// string on disk, pointer in memory, TAG_GAME
    F_VECTOR,
    F_VECTOR4,
    F_ANGLEHACK,
    F_ENTITY,			// index on disk, pointer in memory
    F_ITEM,				// index on disk, pointer in memory
    F_CLIENT,			// index on disk, pointer in memory
    F_PARM1,			// Special case for parms
    F_PARM2,			// Special case for parms
    F_PARM3,			// Special case for parms
    F_PARM4,			// Special case for parms
    F_PARM5,			// Special case for parms
    F_PARM6,			// Special case for parms
    F_PARM7,			// Special case for parms
    F_PARM8,			// Special case for parms
    F_PARM9,			// Special case for parms
    F_PARM10,			// Special case for parms
    F_PARM11,			// Special case for parms
    F_PARM12,			// Special case for parms
    F_PARM13,			// Special case for parms
    F_PARM14,			// Special case for parms
    F_PARM15,			// Special case for parms
    F_PARM16,			// Special case for parms
    F_FLAG,				// special case for flags
    F_IGNORE,
}

#[repr(C)]
pub struct field_t {
    pub name: *const c_char,
    pub ofs: c_int,
    pub field_type: fieldtype_t,
    pub flags: c_int,
}

pub const flagTable: [stringID_table_t; 2] = [
    //"noTED", EF_NO_TED,
    //stringID_table_t Must end with a null entry
    stringID_table_t {
        name: b"\0" as *const u8 as *const c_char,
        id: ptr::null(),
    },
    stringID_table_t {
        name: ptr::null(),
        id: ptr::null(),
    },
];

// Field definitions for spawning - partial representation
// Note: Complete field table from original has ~70+ entries for Radiant/game fields
// The offsets (ofs) are placeholders; actual values depend on gentity_t layout
pub static FIELDS_TABLE: &[field_t] = &[
    //Fields for benefit of Radiant only
    field_t {
        name: b"autobound\0" as *const u8 as *const c_char,
        ofs: 0,
        field_type: fieldtype_t::F_IGNORE,
        flags: 0,
    },
    field_t {
        name: b"groupname\0" as *const u8 as *const c_char,
        ofs: 0,
        field_type: fieldtype_t::F_IGNORE,
        flags: 0,
    },
    field_t {
        name: b"noBasicSounds\0" as *const u8 as *const c_char,
        ofs: 0,
        field_type: fieldtype_t::F_IGNORE,
        flags: 0,
    },
    field_t {
        name: b"noCombatSounds\0" as *const u8 as *const c_char,
        ofs: 0,
        field_type: fieldtype_t::F_IGNORE,
        flags: 0,
    },
    field_t {
        name: b"noExtraSounds\0" as *const u8 as *const c_char,
        ofs: 0,
        field_type: fieldtype_t::F_IGNORE,
        flags: 0,
    },
];

#[repr(C)]
pub struct spawn_t {
    pub name: *const c_char,
    pub spawn: Option<unsafe extern "C" fn(*mut gentity_t)>,
}

// Spawn function declarations
extern "C" {
    pub fn SP_info_player_start(ent: *mut gentity_t);
    pub fn SP_info_player_deathmatch(ent: *mut gentity_t);
    pub fn SP_info_player_intermission(ent: *mut gentity_t);
    pub fn SP_info_firstplace(ent: *mut gentity_t);
    pub fn SP_info_secondplace(ent: *mut gentity_t);
    pub fn SP_info_thirdplace(ent: *mut gentity_t);

    pub fn SP_func_plat(ent: *mut gentity_t);
    pub fn SP_func_static(ent: *mut gentity_t);
    pub fn SP_func_rotating(ent: *mut gentity_t);
    pub fn SP_func_bobbing(ent: *mut gentity_t);
    pub fn SP_func_breakable(self_: *mut gentity_t);
    pub fn SP_func_glass(self_: *mut gentity_t);
    pub fn SP_func_pendulum(ent: *mut gentity_t);
    pub fn SP_func_button(ent: *mut gentity_t);
    pub fn SP_func_door(ent: *mut gentity_t);
    pub fn SP_func_train(ent: *mut gentity_t);
    pub fn SP_func_timer(self_: *mut gentity_t);
    pub fn SP_func_wall(ent: *mut gentity_t);
    pub fn SP_func_usable(self_: *mut gentity_t);
    pub fn SP_rail_mover(self_: *mut gentity_t);
    pub fn SP_rail_track(self_: *mut gentity_t);
    pub fn SP_rail_lane(self_: *mut gentity_t);

    pub fn SP_trigger_always(ent: *mut gentity_t);
    pub fn SP_trigger_multiple(ent: *mut gentity_t);
    pub fn SP_trigger_once(ent: *mut gentity_t);
    pub fn SP_trigger_push(ent: *mut gentity_t);
    pub fn SP_trigger_teleport(ent: *mut gentity_t);
    pub fn SP_trigger_hurt(ent: *mut gentity_t);
    pub fn SP_trigger_bidirectional(ent: *mut gentity_t);
    pub fn SP_trigger_entdist(self_: *mut gentity_t);
    pub fn SP_trigger_location(ent: *mut gentity_t);
    pub fn SP_trigger_visible(self_: *mut gentity_t);
    pub fn SP_trigger_space(self_: *mut gentity_t);
    pub fn SP_trigger_shipboundary(self_: *mut gentity_t);

    pub fn SP_target_give(ent: *mut gentity_t);
    pub fn SP_target_delay(ent: *mut gentity_t);
    pub fn SP_target_speaker(ent: *mut gentity_t);
    pub fn SP_target_print(ent: *mut gentity_t);
    pub fn SP_target_laser(self_: *mut gentity_t);
    pub fn SP_target_character(ent: *mut gentity_t);
    pub fn SP_target_score(ent: *mut gentity_t);
    pub fn SP_target_teleporter(ent: *mut gentity_t);
    pub fn SP_target_relay(ent: *mut gentity_t);
    pub fn SP_target_kill(ent: *mut gentity_t);
    pub fn SP_target_position(ent: *mut gentity_t);
    pub fn SP_target_location(ent: *mut gentity_t);
    pub fn SP_target_push(ent: *mut gentity_t);
    pub fn SP_target_random(self_: *mut gentity_t);
    pub fn SP_target_counter(self_: *mut gentity_t);
    pub fn SP_target_scriptrunner(self_: *mut gentity_t);
    pub fn SP_target_interest(self_: *mut gentity_t);
    pub fn SP_target_activate(self_: *mut gentity_t);
    pub fn SP_target_deactivate(self_: *mut gentity_t);
    pub fn SP_target_gravity_change(self_: *mut gentity_t);
    pub fn SP_target_friction_change(self_: *mut gentity_t);
    pub fn SP_target_level_change(self_: *mut gentity_t);
    pub fn SP_target_change_parm(self_: *mut gentity_t);
    pub fn SP_target_play_music(self_: *mut gentity_t);
    pub fn SP_target_autosave(self_: *mut gentity_t);
    pub fn SP_target_secret(self_: *mut gentity_t);

    pub fn SP_light(self_: *mut gentity_t);
    pub fn SP_info_null(self_: *mut gentity_t);
    pub fn SP_info_notnull(self_: *mut gentity_t);
    pub fn SP_path_corner(self_: *mut gentity_t);

    pub fn SP_misc_teleporter(self_: *mut gentity_t);
    pub fn SP_misc_teleporter_dest(self_: *mut gentity_t);
    pub fn SP_misc_model(ent: *mut gentity_t);
    pub fn SP_misc_model_static(ent: *mut gentity_t);
    pub fn SP_misc_turret(base: *mut gentity_t);
    pub fn SP_misc_ns_turret(base: *mut gentity_t);
    pub fn SP_laser_arm(base: *mut gentity_t);
    pub fn SP_misc_ion_cannon(ent: *mut gentity_t);
    pub fn SP_misc_maglock(ent: *mut gentity_t);
    pub fn SP_misc_panel_turret(ent: *mut gentity_t);
    pub fn SP_misc_model_welder(ent: *mut gentity_t);
    pub fn SP_misc_model_jabba_cam(ent: *mut gentity_t);

    pub fn SP_misc_model_shield_power_converter(ent: *mut gentity_t);
    pub fn SP_misc_model_ammo_power_converter(ent: *mut gentity_t);
    pub fn SP_misc_model_bomb_planted(ent: *mut gentity_t);
    pub fn SP_misc_model_beacon(ent: *mut gentity_t);

    pub fn SP_misc_shield_floor_unit(ent: *mut gentity_t);
    pub fn SP_misc_ammo_floor_unit(ent: *mut gentity_t);

    pub fn SP_misc_model_gun_rack(ent: *mut gentity_t);
    pub fn SP_misc_model_ammo_rack(ent: *mut gentity_t);
    pub fn SP_misc_model_cargo_small(ent: *mut gentity_t);

    pub fn SP_misc_exploding_crate(ent: *mut gentity_t);
    pub fn SP_misc_gas_tank(ent: *mut gentity_t);
    pub fn SP_misc_crystal_crate(ent: *mut gentity_t);
    pub fn SP_misc_atst_drivable(ent: *mut gentity_t);

    pub fn SP_misc_model_breakable(ent: *mut gentity_t);
    pub fn SP_misc_model_ghoul(ent: *mut gentity_t);
    pub fn SP_misc_portal_camera(ent: *mut gentity_t);

    pub fn SP_misc_bsp(ent: *mut gentity_t);
    pub fn SP_terrain(ent: *mut gentity_t);
    pub fn SP_misc_skyportal(ent: *mut gentity_t);

    pub fn SP_misc_portal_surface(ent: *mut gentity_t);
    pub fn SP_misc_camera_focus(self_: *mut gentity_t);
    pub fn SP_misc_camera_track(self_: *mut gentity_t);
    pub fn SP_misc_dlight(ent: *mut gentity_t);
    pub fn SP_misc_security_panel(ent: *mut gentity_t);
    pub fn SP_misc_camera(ent: *mut gentity_t);
    pub fn SP_misc_spotlight(ent: *mut gentity_t);

    pub fn SP_shooter_rocket(ent: *mut gentity_t);
    pub fn SP_shooter_plasma(ent: *mut gentity_t);
    pub fn SP_shooter_grenade(ent: *mut gentity_t);
    pub fn SP_misc_replicator_item(ent: *mut gentity_t);
    pub fn SP_misc_trip_mine(self_: *mut gentity_t);
    pub fn SP_PAS(ent: *mut gentity_t);
    pub fn SP_misc_weapon_shooter(self_: *mut gentity_t);
    pub fn SP_misc_weather_zone(ent: *mut gentity_t);

    //New spawn functions
    pub fn SP_reference_tag(ent: *mut gentity_t);

    pub fn SP_NPC_spawner(self_: *mut gentity_t);

    pub fn SP_NPC_Vehicle(self_: *mut gentity_t);
    pub fn SP_NPC_Player(self_: *mut gentity_t);
    pub fn SP_NPC_Kyle(self_: *mut gentity_t);
    pub fn SP_NPC_Lando(self_: *mut gentity_t);
    pub fn SP_NPC_Jan(self_: *mut gentity_t);
    pub fn SP_NPC_Luke(self_: *mut gentity_t);
    pub fn SP_NPC_MonMothma(self_: *mut gentity_t);
    pub fn SP_NPC_Rosh_Penin(self_: *mut gentity_t);
    pub fn SP_NPC_Tavion(self_: *mut gentity_t);
    pub fn SP_NPC_Tavion_New(self_: *mut gentity_t);
    pub fn SP_NPC_Alora(self_: *mut gentity_t);
    pub fn SP_NPC_Reelo(self_: *mut gentity_t);
    pub fn SP_NPC_Galak(self_: *mut gentity_t);
    pub fn SP_NPC_Desann(self_: *mut gentity_t);
    pub fn SP_NPC_Rax(self_: *mut gentity_t);
    pub fn SP_NPC_BobaFett(self_: *mut gentity_t);
    pub fn SP_NPC_Ragnos(self_: *mut gentity_t);
    pub fn SP_NPC_Lannik_Racto(self_: *mut gentity_t);
    pub fn SP_NPC_Kothos(self_: *mut gentity_t);
    pub fn SP_NPC_Chewbacca(self_: *mut gentity_t);
    pub fn SP_NPC_Bartender(self_: *mut gentity_t);
    pub fn SP_NPC_MorganKatarn(self_: *mut gentity_t);
    pub fn SP_NPC_Jedi(self_: *mut gentity_t);
    pub fn SP_NPC_Prisoner(self_: *mut gentity_t);
    pub fn SP_NPC_Merchant(self_: *mut gentity_t);
    pub fn SP_NPC_Rebel(self_: *mut gentity_t);
    pub fn SP_NPC_Human_Merc(self_: *mut gentity_t);
    pub fn SP_NPC_Stormtrooper(self_: *mut gentity_t);
    pub fn SP_NPC_StormtrooperOfficer(self_: *mut gentity_t);
    pub fn SP_NPC_Tie_Pilot(self_: *mut gentity_t);
    pub fn SP_NPC_Snowtrooper(self_: *mut gentity_t);
    pub fn SP_NPC_RocketTrooper(self_: *mut gentity_t);
    pub fn SP_NPC_HazardTrooper(self_: *mut gentity_t);
    pub fn SP_NPC_Ugnaught(self_: *mut gentity_t);
    pub fn SP_NPC_Jawa(self_: *mut gentity_t);
    pub fn SP_NPC_Gran(self_: *mut gentity_t);
    pub fn SP_NPC_Rodian(self_: *mut gentity_t);
    pub fn SP_NPC_Weequay(self_: *mut gentity_t);
    pub fn SP_NPC_Trandoshan(self_: *mut gentity_t);
    pub fn SP_NPC_Tusken(self_: *mut gentity_t);
    pub fn SP_NPC_Noghri(self_: *mut gentity_t);
    pub fn SP_NPC_SwampTrooper(self_: *mut gentity_t);
    pub fn SP_NPC_Imperial(self_: *mut gentity_t);
    pub fn SP_NPC_ImpWorker(self_: *mut gentity_t);
    pub fn SP_NPC_BespinCop(self_: *mut gentity_t);
    pub fn SP_NPC_Reborn(self_: *mut gentity_t);
    pub fn SP_NPC_Reborn_New(self_: *mut gentity_t);
    pub fn SP_NPC_Cultist(self_: *mut gentity_t);
    pub fn SP_NPC_Cultist_Saber(self_: *mut gentity_t);
    pub fn SP_NPC_Cultist_Saber_Powers(self_: *mut gentity_t);
    pub fn SP_NPC_Cultist_Destroyer(self_: *mut gentity_t);
    pub fn SP_NPC_Cultist_Commando(self_: *mut gentity_t);
    pub fn SP_NPC_ShadowTrooper(self_: *mut gentity_t);
    pub fn SP_NPC_Saboteur(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Murjj(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Swamp(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Howler(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Rancor(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Mutant_Rancor(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Wampa(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Claw(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Glider(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Flier2(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Lizard(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Fish(self_: *mut gentity_t);
    pub fn SP_NPC_Monster_Sand_Creature(self_: *mut gentity_t);
    pub fn SP_NPC_MineMonster(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Interrogator(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Probe(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Mark1(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Mark2(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_ATST(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Seeker(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Remote(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Sentry(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Gonk(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Mouse(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_R2D2(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_R5D2(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Protocol(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Assassin(self_: *mut gentity_t);
    pub fn SP_NPC_Droid_Saber(self_: *mut gentity_t);

    pub fn SP_waypoint(ent: *mut gentity_t);
    pub fn SP_waypoint_small(ent: *mut gentity_t);
    pub fn SP_waypoint_navgoal(ent: *mut gentity_t);

    pub fn SP_fx_runner(ent: *mut gentity_t);
    pub fn SP_fx_explosion_trail(ent: *mut gentity_t);
    pub fn SP_fx_target_beam(ent: *mut gentity_t);
    pub fn SP_fx_cloudlayer(ent: *mut gentity_t);

    pub fn SP_CreateSnow(ent: *mut gentity_t);
    pub fn SP_CreateRain(ent: *mut gentity_t);
    pub fn SP_CreateWind(ent: *mut gentity_t);
    pub fn SP_CreateWindZone(ent: *mut gentity_t);
    // Added 10/20/02 by Aurelio Reis
    pub fn SP_CreatePuffSystem(ent: *mut gentity_t);

    pub fn SP_object_cargo_barrel1(ent: *mut gentity_t);

    pub fn SP_point_combat(self_: *mut gentity_t);

    pub fn SP_emplaced_eweb(self_: *mut gentity_t);
    pub fn SP_emplaced_gun(self_: *mut gentity_t);

    pub fn SP_misc_turbobattery(base: *mut gentity_t);
}

// Spawn function table - maps entity classnames to spawn functions
// Preserved from original with exact ordering as in oracle/code/game/g_spawn.cpp
pub static SPAWNS_TABLE: &[(&[u8], Option<unsafe extern "C" fn(*mut gentity_t)>)] = &[
    (b"info_player_start\0", Some(SP_info_player_start)),
    (b"info_player_deathmatch\0", Some(SP_info_player_deathmatch)),
    (b"func_plat\0", Some(SP_func_plat)),
    (b"func_button\0", Some(SP_func_button)),
    (b"func_door\0", Some(SP_func_door)),
    (b"func_static\0", Some(SP_func_static)),
    (b"func_rotating\0", Some(SP_func_rotating)),
    (b"func_bobbing\0", Some(SP_func_bobbing)),
    (b"func_breakable\0", Some(SP_func_breakable)),
    (b"func_pendulum\0", Some(SP_func_pendulum)),
    (b"func_train\0", Some(SP_func_train)),
    (b"func_timer\0", Some(SP_func_timer)),
    (b"func_wall\0", Some(SP_func_wall)),
    (b"func_usable\0", Some(SP_func_usable)),
    (b"func_glass\0", Some(SP_func_glass)),
    (b"rail_mover\0", Some(SP_rail_mover)),
    (b"rail_track\0", Some(SP_rail_track)),
    (b"rail_lane\0", Some(SP_rail_lane)),
    (b"trigger_always\0", Some(SP_trigger_always)),
    (b"trigger_multiple\0", Some(SP_trigger_multiple)),
    (b"trigger_once\0", Some(SP_trigger_once)),
    (b"trigger_push\0", Some(SP_trigger_push)),
    (b"trigger_teleport\0", Some(SP_trigger_teleport)),
    (b"trigger_hurt\0", Some(SP_trigger_hurt)),
    (b"trigger_bidirectional\0", Some(SP_trigger_bidirectional)),
    (b"trigger_entdist\0", Some(SP_trigger_entdist)),
    (b"trigger_location\0", Some(SP_trigger_location)),
    (b"trigger_visible\0", Some(SP_trigger_visible)),
    (b"trigger_space\0", Some(SP_trigger_space)),
    (b"trigger_shipboundary\0", Some(SP_trigger_shipboundary)),
    (b"target_give\0", Some(SP_target_give)),
    (b"target_delay\0", Some(SP_target_delay)),
    (b"target_speaker\0", Some(SP_target_speaker)),
    (b"target_print\0", Some(SP_target_print)),
    (b"target_laser\0", Some(SP_target_laser)),
    (b"target_score\0", Some(SP_target_score)),
    (b"target_teleporter\0", Some(SP_target_teleporter)),
    (b"target_relay\0", Some(SP_target_relay)),
    (b"target_kill\0", Some(SP_target_kill)),
    (b"target_position\0", Some(SP_target_position)),
    (b"target_location\0", Some(SP_target_location)),
    (b"target_push\0", Some(SP_target_push)),
    (b"target_random\0", Some(SP_target_random)),
    (b"target_counter\0", Some(SP_target_counter)),
    (b"target_scriptrunner\0", Some(SP_target_scriptrunner)),
    (b"target_interest\0", Some(SP_target_interest)),
    (b"target_activate\0", Some(SP_target_activate)),
    (b"target_deactivate\0", Some(SP_target_deactivate)),
    (b"target_gravity_change\0", Some(SP_target_gravity_change)),
    (b"target_friction_change\0", Some(SP_target_friction_change)),
    (b"target_level_change\0", Some(SP_target_level_change)),
    (b"target_change_parm\0", Some(SP_target_change_parm)),
    (b"target_play_music\0", Some(SP_target_play_music)),
    (b"target_autosave\0", Some(SP_target_autosave)),
    (b"target_secret\0", Some(SP_target_secret)),
    (b"light\0", Some(SP_light)),
    (b"info_null\0", Some(SP_info_null)),
    (b"func_group\0", Some(SP_info_null)),
    (b"info_notnull\0", Some(SP_info_notnull)),
    (b"path_corner\0", Some(SP_path_corner)),
    (b"misc_teleporter\0", Some(SP_misc_teleporter)),
    (b"misc_teleporter_dest\0", Some(SP_misc_teleporter_dest)),
    (b"misc_model\0", Some(SP_misc_model)),
    (b"misc_model_static\0", Some(SP_misc_model_static)),
    (b"misc_turret\0", Some(SP_misc_turret)),
    (b"misc_ns_turret\0", Some(SP_misc_ns_turret)),
    (b"misc_laser_arm\0", Some(SP_laser_arm)),
    (b"misc_ion_cannon\0", Some(SP_misc_ion_cannon)),
    (b"misc_sentry_turret\0", Some(SP_PAS)),
    (b"misc_maglock\0", Some(SP_misc_maglock)),
    (b"misc_weapon_shooter\0", Some(SP_misc_weapon_shooter)),
    (b"misc_weather_zone\0", Some(SP_misc_weather_zone)),
    (b"misc_model_ghoul\0", Some(SP_misc_model_ghoul)),
    (b"misc_model_breakable\0", Some(SP_misc_model_breakable)),
    (b"misc_portal_surface\0", Some(SP_misc_portal_surface)),
    (b"misc_portal_camera\0", Some(SP_misc_portal_camera)),
    (b"misc_bsp\0", Some(SP_misc_bsp)),
    (b"terrain\0", Some(SP_terrain)),
    (b"misc_skyportal\0", Some(SP_misc_skyportal)),
    (b"misc_camera_focus\0", Some(SP_misc_camera_focus)),
    (b"misc_camera_track\0", Some(SP_misc_camera_track)),
    (b"misc_dlight\0", Some(SP_misc_dlight)),
    (b"misc_replicator_item\0", Some(SP_misc_replicator_item)),
    (b"misc_trip_mine\0", Some(SP_misc_trip_mine)),
    (b"misc_security_panel\0", Some(SP_misc_security_panel)),
    (b"misc_camera\0", Some(SP_misc_camera)),
    (b"misc_spotlight\0", Some(SP_misc_spotlight)),
    (b"misc_panel_turret\0", Some(SP_misc_panel_turret)),
    (b"misc_model_welder\0", Some(SP_misc_model_welder)),
    (b"misc_model_jabba_cam\0", Some(SP_misc_model_jabba_cam)),
    (b"misc_model_shield_power_converter\0", Some(SP_misc_model_shield_power_converter)),
    (b"misc_model_ammo_power_converter\0", Some(SP_misc_model_ammo_power_converter)),
    (b"misc_model_bomb_planted\0", Some(SP_misc_model_bomb_planted)),
    (b"misc_model_beacon\0", Some(SP_misc_model_beacon)),
    (b"misc_shield_floor_unit\0", Some(SP_misc_shield_floor_unit)),
    (b"misc_ammo_floor_unit\0", Some(SP_misc_ammo_floor_unit)),
    (b"misc_model_gun_rack\0", Some(SP_misc_model_gun_rack)),
    (b"misc_model_ammo_rack\0", Some(SP_misc_model_ammo_rack)),
    (b"misc_model_cargo_small\0", Some(SP_misc_model_cargo_small)),
    (b"misc_exploding_crate\0", Some(SP_misc_exploding_crate)),
    (b"misc_gas_tank\0", Some(SP_misc_gas_tank)),
    (b"misc_crystal_crate\0", Some(SP_misc_crystal_crate)),
    (b"misc_atst_drivable\0", Some(SP_misc_atst_drivable)),
    (b"shooter_rocket\0", Some(SP_shooter_rocket)),
    (b"shooter_grenade\0", Some(SP_shooter_grenade)),
    (b"shooter_plasma\0", Some(SP_shooter_plasma)),
    (b"ref_tag\0", Some(SP_reference_tag)),
    (b"NPC_spawner\0", Some(SP_NPC_spawner)),
    (b"NPC_Vehicle\0", Some(SP_NPC_Vehicle)),
    (b"NPC_Player\0", Some(SP_NPC_Player)),
    (b"NPC_Kyle\0", Some(SP_NPC_Kyle)),
    (b"NPC_Lando\0", Some(SP_NPC_Lando)),
    (b"NPC_Jan\0", Some(SP_NPC_Jan)),
    (b"NPC_Luke\0", Some(SP_NPC_Luke)),
    (b"NPC_MonMothma\0", Some(SP_NPC_MonMothma)),
    (b"NPC_Rosh_Penin\0", Some(SP_NPC_Rosh_Penin)),
    (b"NPC_Tavion\0", Some(SP_NPC_Tavion)),
    (b"NPC_Tavion_New\0", Some(SP_NPC_Tavion_New)),
    (b"NPC_Alora\0", Some(SP_NPC_Alora)),
    (b"NPC_Reelo\0", Some(SP_NPC_Reelo)),
    (b"NPC_Galak\0", Some(SP_NPC_Galak)),
    (b"NPC_Desann\0", Some(SP_NPC_Desann)),
    (b"NPC_Rax\0", Some(SP_NPC_Rax)),
    (b"NPC_BobaFett\0", Some(SP_NPC_BobaFett)),
    (b"NPC_Ragnos\0", Some(SP_NPC_Ragnos)),
    (b"NPC_Lannik_Racto\0", Some(SP_NPC_Lannik_Racto)),
    (b"NPC_Kothos\0", Some(SP_NPC_Kothos)),
    (b"NPC_Chewbacca\0", Some(SP_NPC_Chewbacca)),
    (b"NPC_Bartender\0", Some(SP_NPC_Bartender)),
    (b"NPC_MorganKatarn\0", Some(SP_NPC_MorganKatarn)),
    (b"NPC_Jedi\0", Some(SP_NPC_Jedi)),
    (b"NPC_Prisoner\0", Some(SP_NPC_Prisoner)),
    (b"NPC_Merchant\0", Some(SP_NPC_Merchant)),
    (b"NPC_Rebel\0", Some(SP_NPC_Rebel)),
    (b"NPC_Human_Merc\0", Some(SP_NPC_Human_Merc)),
    (b"NPC_Stormtrooper\0", Some(SP_NPC_Stormtrooper)),
    (b"NPC_StormtrooperOfficer\0", Some(SP_NPC_StormtrooperOfficer)),
    (b"NPC_Tie_Pilot\0", Some(SP_NPC_Tie_Pilot)),
    (b"NPC_Snowtrooper\0", Some(SP_NPC_Snowtrooper)),
    (b"NPC_RocketTrooper\0", Some(SP_NPC_RocketTrooper)),
    (b"NPC_HazardTrooper\0", Some(SP_NPC_HazardTrooper)),
    (b"NPC_Ugnaught\0", Some(SP_NPC_Ugnaught)),
    (b"NPC_Jawa\0", Some(SP_NPC_Jawa)),
    (b"NPC_Gran\0", Some(SP_NPC_Gran)),
    (b"NPC_Rodian\0", Some(SP_NPC_Rodian)),
    (b"NPC_Weequay\0", Some(SP_NPC_Weequay)),
    (b"NPC_Trandoshan\0", Some(SP_NPC_Trandoshan)),
    (b"NPC_Tusken\0", Some(SP_NPC_Tusken)),
    (b"NPC_Noghri\0", Some(SP_NPC_Noghri)),
    (b"NPC_SwampTrooper\0", Some(SP_NPC_SwampTrooper)),
    (b"NPC_Imperial\0", Some(SP_NPC_Imperial)),
    (b"NPC_ImpWorker\0", Some(SP_NPC_ImpWorker)),
    (b"NPC_BespinCop\0", Some(SP_NPC_BespinCop)),
    (b"NPC_Reborn\0", Some(SP_NPC_Reborn)),
    (b"NPC_Reborn_New\0", Some(SP_NPC_Reborn_New)),
    (b"NPC_Cultist\0", Some(SP_NPC_Cultist)),
    (b"NPC_Cultist_Saber\0", Some(SP_NPC_Cultist_Saber)),
    (b"NPC_Cultist_Saber_Powers\0", Some(SP_NPC_Cultist_Saber_Powers)),
    (b"NPC_Cultist_Destroyer\0", Some(SP_NPC_Cultist_Destroyer)),
    (b"NPC_Cultist_Commando\0", Some(SP_NPC_Cultist_Commando)),
    (b"NPC_ShadowTrooper\0", Some(SP_NPC_ShadowTrooper)),
    (b"NPC_Saboteur\0", Some(SP_NPC_Saboteur)),
    (b"NPC_Monster_Murjj\0", Some(SP_NPC_Monster_Murjj)),
    (b"NPC_Monster_Swamp\0", Some(SP_NPC_Monster_Swamp)),
    (b"NPC_Monster_Howler\0", Some(SP_NPC_Monster_Howler)),
    (b"NPC_Monster_Rancor\0", Some(SP_NPC_Monster_Rancor)),
    (b"NPC_Monster_Mutant_Rancor\0", Some(SP_NPC_Monster_Mutant_Rancor)),
    (b"NPC_Monster_Wampa\0", Some(SP_NPC_Monster_Wampa)),
    (b"NPC_MineMonster\0", Some(SP_NPC_MineMonster)),
    (b"NPC_Monster_Claw\0", Some(SP_NPC_Monster_Claw)),
    (b"NPC_Monster_Glider\0", Some(SP_NPC_Monster_Glider)),
    (b"NPC_Monster_Flier2\0", Some(SP_NPC_Monster_Flier2)),
    (b"NPC_Monster_Lizard\0", Some(SP_NPC_Monster_Lizard)),
    (b"NPC_Monster_Fish\0", Some(SP_NPC_Monster_Fish)),
    (b"NPC_Monster_Sand_Creature\0", Some(SP_NPC_Monster_Sand_Creature)),
    (b"NPC_Droid_Interrogator\0", Some(SP_NPC_Droid_Interrogator)),
    (b"NPC_Droid_Probe\0", Some(SP_NPC_Droid_Probe)),
    (b"NPC_Droid_Mark1\0", Some(SP_NPC_Droid_Mark1)),
    (b"NPC_Droid_Mark2\0", Some(SP_NPC_Droid_Mark2)),
    (b"NPC_Droid_ATST\0", Some(SP_NPC_Droid_ATST)),
    (b"NPC_Droid_Seeker\0", Some(SP_NPC_Droid_Seeker)),
    (b"NPC_Droid_Remote\0", Some(SP_NPC_Droid_Remote)),
    (b"NPC_Droid_Sentry\0", Some(SP_NPC_Droid_Sentry)),
    (b"NPC_Droid_Gonk\0", Some(SP_NPC_Droid_Gonk)),
    (b"NPC_Droid_Mouse\0", Some(SP_NPC_Droid_Mouse)),
    (b"NPC_Droid_R2D2\0", Some(SP_NPC_Droid_R2D2)),
    (b"NPC_Droid_R5D2\0", Some(SP_NPC_Droid_R5D2)),
    (b"NPC_Droid_Protocol\0", Some(SP_NPC_Droid_Protocol)),
    (b"NPC_Droid_Assassin\0", Some(SP_NPC_Droid_Assassin)),
    (b"NPC_Droid_Saber\0", Some(SP_NPC_Droid_Saber)),
    (b"NPC_Colombian_Soldier\0", Some(SP_NPC_Reborn)),
    (b"NPC_Colombian_Rebel\0", Some(SP_NPC_Reborn)),
    (b"NPC_Colombian_EmplacedGunner\0", Some(SP_NPC_ShadowTrooper)),
    (b"NPC_Manuel_Vergara_RMG\0", Some(SP_NPC_Desann)),
    (b"waypoint\0", Some(SP_waypoint)),
    (b"waypoint_small\0", Some(SP_waypoint_small)),
    (b"waypoint_navgoal\0", Some(SP_waypoint_navgoal)),
    (b"fx_runner\0", Some(SP_fx_runner)),
    (b"fx_explosion_trail\0", Some(SP_fx_explosion_trail)),
    (b"fx_target_beam\0", Some(SP_fx_target_beam)),
    (b"fx_cloudlayer\0", Some(SP_fx_cloudlayer)),
    (b"fx_rain\0", Some(SP_CreateRain)),
    (b"fx_wind\0", Some(SP_CreateWind)),
    (b"fx_snow\0", Some(SP_CreateSnow)),
    (b"fx_puff\0", Some(SP_CreatePuffSystem)),
    (b"fx_wind_zone\0", Some(SP_CreateWindZone)),
    (b"object_cargo_barrel1\0", Some(SP_object_cargo_barrel1)),
    (b"point_combat\0", Some(SP_point_combat)),
    (b"emplaced_gun\0", Some(SP_emplaced_gun)),
    (b"emplaced_eweb\0", Some(SP_emplaced_eweb)),
];

pub fn AddSpawnField(field: *mut c_char, value: *mut c_char) {
    let mut i: c_int;

    unsafe {
        for i in 0..numSpawnVars {
            if Q_stricmp(*spawnVars.get_unchecked(i as usize).get_unchecked(0), field) == 0 {
                *spawnVars.get_unchecked_mut(i as usize).get_unchecked_mut(1) = G_AddSpawnVarToken(value as *const c_char);
                return;
            }
        }

        *spawnVars.get_unchecked_mut(numSpawnVars as usize).get_unchecked_mut(0) = G_AddSpawnVarToken(field as *const c_char);
        *spawnVars.get_unchecked_mut(numSpawnVars as usize).get_unchecked_mut(1) = G_AddSpawnVarToken(value as *const c_char);
        numSpawnVars += 1;
    }
}

pub extern "C" fn G_SpawnField(uiField: c_int, ppKey: *mut *mut c_char, ppValue: *mut *mut c_char) -> qboolean {
    unsafe {
        if uiField >= numSpawnVars {
            return qfalse;
        }

        *ppKey = *spawnVars.get_unchecked(uiField as usize).get_unchecked(0);
        *ppValue = *spawnVars.get_unchecked(uiField as usize).get_unchecked(1);

        return qtrue;
    }
}

pub extern "C" fn G_SpawnString(key: *const c_char, defaultString: *const c_char, out: *mut *mut c_char) -> qboolean {
    let mut i: c_int;

    unsafe {
        if spawning == qfalse {
            *out = defaultString as *mut c_char;
            //		G_Error( "G_SpawnString() called while not spawning" );
        }

        for i in 0..numSpawnVars {
            if Q_stricmp(key, *spawnVars.get_unchecked(i as usize).get_unchecked(0)) == 0 {
                *out = *spawnVars.get_unchecked(i as usize).get_unchecked(1);
                return qtrue;
            }
        }

        *out = defaultString as *mut c_char;
        return qfalse;
    }
}

pub extern "C" fn G_SpawnFloat(key: *const c_char, defaultString: *const c_char, out: *mut f32) -> qboolean {
    let s: *mut c_char = ptr::null_mut();
    let mut s_mut = s;
    let present: qboolean;

    unsafe {
        present = G_SpawnString(key, defaultString, &mut s_mut);
        *out = atof(s_mut);
        return present;
    }
}

pub extern "C" fn G_SpawnInt(key: *const c_char, defaultString: *const c_char, out: *mut c_int) -> qboolean {
    let s: *mut c_char = ptr::null_mut();
    let mut s_mut = s;
    let present: qboolean;

    unsafe {
        present = G_SpawnString(key, defaultString, &mut s_mut);
        *out = atoi(s_mut);
        return present;
    }
}

pub extern "C" fn G_SpawnVector(key: *const c_char, defaultString: *const c_char, out: *mut f32) -> qboolean {
    let s: *mut c_char = ptr::null_mut();
    let mut s_mut = s;
    let present: qboolean;

    unsafe {
        present = G_SpawnString(key, defaultString, &mut s_mut);
        sscanf(s_mut, b"%f %f %f\0" as *const u8 as *const c_char, out.add(0), out.add(1), out.add(2));
        return present;
    }
}

pub extern "C" fn G_SpawnVector4(key: *const c_char, defaultString: *const c_char, out: *mut f32) -> qboolean {
    let s: *mut c_char = ptr::null_mut();
    let mut s_mut = s;
    let present: qboolean;

    unsafe {
        present = G_SpawnString(key, defaultString, &mut s_mut);
        sscanf(s_mut, b"%f %f %f %f\0" as *const u8 as *const c_char, out.add(0), out.add(1), out.add(2), out.add(3));
        return present;
    }
}

pub extern "C" fn G_SpawnFlag(key: *const c_char, flag: c_int, out: *mut c_int) -> qboolean {
    //find that key
    unsafe {
        for i in 0..numSpawnVars {
            if strcmp(key, *spawnVars.get_unchecked(i as usize).get_unchecked(0)) == 0 {
                //found the key
                if atoi(*spawnVars.get_unchecked(i as usize).get_unchecked(1)) != 0 {
                    //if it's non-zero, and in the flag
                    *out |= flag;
                } else {
                    //if it's zero, or out the flag
                    *out &= !flag;
                }
                return qtrue;
            }
        }
    }

    return qfalse;
}

pub extern "C" fn G_SpawnAngleHack(key: *const c_char, defaultString: *const c_char, out: *mut f32) -> qboolean {
    let s: *mut c_char = ptr::null_mut();
    let mut s_mut = s;
    let present: qboolean;
    let mut temp: f32 = 0.0;

    unsafe {
        present = G_SpawnString(key, defaultString, &mut s_mut);
        sscanf(s_mut, b"%f\0" as *const u8 as *const c_char, &mut temp);

        *out.add(0) = 0.0;
        *out.add(1) = temp;
        *out.add(2) = 0.0;

        return present;
    }
}

/*
===============
G_CallSpawn

Finds the spawn function for the entity and calls it,
returning qfalse if not found
===============
*/
pub extern "C" fn G_CallSpawn(ent: *mut gentity_t) -> qboolean {
    let mut item: *mut gitem_t;

    unsafe {
        if (*ent).classname.is_null() {
            // gi.Printf (S_COLOR_RED"G_CallSpawn: NULL classname\n");
            return qfalse;
        }

        // check item spawn functions
        item = core::mem::addr_of_mut!(bg_itemlist).add(1);
        loop {
            if (*item).classname.is_null() {
                break;
            }
            if strcmp((*item).classname, (*ent).classname) == 0 {
                // found it
                G_SpawnItem(ent, item);
                return qtrue;
            }
            item = item.add(1);
        }

        // check normal spawn functions
        for spawn_entry in SPAWNS_TABLE {
            let name_str = spawn_entry.0;
            let spawn_fn = spawn_entry.1;

            if strcmp(name_str.as_ptr() as *const c_char, (*ent).classname) == 0 {
                // found it
                if let Some(fn_ptr) = spawn_fn {
                    fn_ptr(ent);
                }
                return qtrue;
            }
        }

        let mut str: *mut c_char = ptr::null_mut();
        G_SpawnString(b"origin\0" as *const u8 as *const c_char, b"?\0" as *const u8 as *const c_char, &mut str);
        // gi.Printf (S_COLOR_RED"ERROR: %s is not a spawn function @(%s)\n", ent->classname, str);
        *core::mem::addr_of_mut!(delayedShutDown) = *core::mem::addr_of!(level).time + 100;
        return qfalse;
    }
}

/*
=============
G_NewString

Builds a copy of the string, translating \n to real linefeeds
so message texts can be multi-line
=============
*/
pub extern "C" fn G_NewString(string: *const c_char) -> *mut c_char {
    let mut newb: *mut c_char;
    let mut new_p: *mut c_char;
    let mut i: c_int;
    let mut l: c_int;

    unsafe {
        if string.is_null() || *string == 0 {
            //gi.Printf(S_COLOR_RED"Error: G_NewString called with NULL string!\n");
            return ptr::null_mut();
        }

        l = strlen(string) as c_int + 1;

        newb = G_Alloc(l) as *mut c_char;

        new_p = newb;

        // turn \n into a real linefeed
        for i in 0..l {
            if *string.add(i as usize) as c_int == '\\' as c_int && i < l - 1 {
                i = i + 1;
                if *string.add(i as usize) as c_int == 'n' as c_int {
                    *new_p = '\n' as c_char;
                    new_p = new_p.add(1);
                } else {
                    *new_p = '\\' as c_char;
                    new_p = new_p.add(1);
                }
            } else {
                *new_p = *string.add(i as usize);
                new_p = new_p.add(1);
            }
        }

        return newb;
    }
}

/*
===============
G_ParseField

Takes a key/value pair and sets the binary values
in a gentity
===============
*/
pub extern "C" fn G_ParseField(key: *const c_char, value: *const c_char, ent: *mut gentity_t) {
    let mut b: *mut u8;
    let mut v: f32;
    let mut vec: [f32; 3] = [0.0; 3];
    let mut vec4: [f32; 4] = [0.0; 4];

    unsafe {
        for f in FIELDS_TABLE {
            if f.name.is_null() {
                break;
            }
            if Q_stricmp(f.name, key) == 0 {
                // found it
                b = ent as *mut u8;

                match f.field_type {
                    fieldtype_t::F_LSTRING => {
                        let dest = (b as *mut *mut c_char).add(f.ofs as usize);
                        *dest = G_NewString(value);
                    }
                    fieldtype_t::F_VECTOR => {
                        let _iFieldsRead = sscanf(value, b"%f %f %f\0" as *const u8 as *const c_char, &mut vec[0], &mut vec[1], &mut vec[2]);
                        // assert(_iFieldsRead==3);
                        if _iFieldsRead != 3 {
                            // gi.Printf (S_COLOR_YELLOW"G_ParseField: VEC3 sscanf() failed to read 3 floats ('angle' key bug?)\n");
                            *core::mem::addr_of_mut!(delayedShutDown) = *core::mem::addr_of!(level).time + 100;
                        }
                        let dest = (b as *mut f32).add(f.ofs as usize);
                        *dest = vec[0];
                        *dest.add(1) = vec[1];
                        *dest.add(2) = vec[2];
                    }
                    fieldtype_t::F_VECTOR4 => {
                        let _iFieldsRead = sscanf(value, b"%f %f %f %f\0" as *const u8 as *const c_char, &mut vec4[0], &mut vec4[1], &mut vec4[2], &mut vec4[3]);
                        // assert(_iFieldsRead==4);
                        if _iFieldsRead != 4 {
                            // gi.Printf (S_COLOR_YELLOW"G_ParseField: VEC4 sscanf() failed to read 4 floats\n");
                            *core::mem::addr_of_mut!(delayedShutDown) = *core::mem::addr_of!(level).time + 100;
                        }
                        let dest = (b as *mut f32).add(f.ofs as usize);
                        *dest = vec4[0];
                        *dest.add(1) = vec4[1];
                        *dest.add(2) = vec4[2];
                        *dest.add(3) = vec4[3];
                    }
                    fieldtype_t::F_INT => {
                        let dest = (b as *mut c_int).add(f.ofs as usize);
                        *dest = atoi(value);
                    }
                    fieldtype_t::F_FLOAT => {
                        let dest = (b as *mut f32).add(f.ofs as usize);
                        *dest = atof(value);
                    }
                    fieldtype_t::F_ANGLEHACK => {
                        v = atof(value);
                        let dest = (b as *mut f32).add(f.ofs as usize);
                        *dest = 0.0;
                        *dest.add(1) = v;
                        *dest.add(2) = 0.0;
                    }
                    fieldtype_t::F_PARM1 | fieldtype_t::F_PARM2 | fieldtype_t::F_PARM3 | fieldtype_t::F_PARM4
                    | fieldtype_t::F_PARM5 | fieldtype_t::F_PARM6 | fieldtype_t::F_PARM7 | fieldtype_t::F_PARM8
                    | fieldtype_t::F_PARM9 | fieldtype_t::F_PARM10 | fieldtype_t::F_PARM11 | fieldtype_t::F_PARM12
                    | fieldtype_t::F_PARM13 | fieldtype_t::F_PARM14 | fieldtype_t::F_PARM15 | fieldtype_t::F_PARM16 => {
                        Q3_SetParm((*ent).s.number, (f.field_type as c_int - fieldtype_t::F_PARM1 as c_int), value);
                    }
                    fieldtype_t::F_FLAG => {
                        //try to find the proper flag for that key:
                        let flag = GetIDForString(core::mem::addr_of!(flagTable), key);

                        if flag > 0 {
                            G_SpawnFlag(key, flag, (b as *mut c_int).add(f.ofs as usize));
                        } else {
                            if !FINAL_BUILD {
                                // gi.Printf (S_COLOR_YELLOW"WARNING: G_ParseField: can't find flag for key %s\n", key);
                            }
                        }
                    }
                    fieldtype_t::F_IGNORE | _ => {}
                }
                return;
            }
        }

        if !FINAL_BUILD {
            //didn't find it?
            if *key as c_int != '_' as c_int {
                // gi.Printf ( S_COLOR_YELLOW"WARNING: G_ParseField: no such field: %s\n", key );
            }
        }
    }
}

pub extern "C" fn SpawnForCurrentDifficultySetting(ent: *mut gentity_t) -> qboolean {
    unsafe {
        if (*com_buildScript).integer != 0 {
            //always spawn when building a pak file
            return qtrue;
        }

        if (*ent).spawnflags & (1 << (8 + (*g_spskill).integer)) != 0 {
            // easy -256	medium -512		hard -1024
            return qfalse;
        } else {
            return qtrue;
        }
    }
}

/*
===================
G_SpawnGEntityFromSpawnVars

Spawn an entity and fill in all of the level fields from
level.spawnVars[], then call the class specfic spawn function
===================
*/

pub extern "C" fn G_SpawnGEntityFromSpawnVars() {
    let mut i: c_int;
    let mut ent: *mut gentity_t;

    unsafe {
        // get the next free entity
        ent = G_Spawn();

        for i in 0..numSpawnVars {
            G_ParseField(*spawnVars.get_unchecked(i as usize).get_unchecked(0), *spawnVars.get_unchecked(i as usize).get_unchecked(1) as *const c_char, ent);
        }

        let mut i_out: c_int = 0;
        G_SpawnInt(b"notsingle\0" as *const u8 as *const c_char, b"0\0" as *const u8 as *const c_char, &mut i_out);
        if i_out != 0 || SpawnForCurrentDifficultySetting(ent) == qfalse {
            G_FreeEntity(ent);
            return;
        }

        // move editor origin to pos
        VectorCopy(&(*ent).s.origin[0], &mut (*ent).s.pos.trBase[0]);
        VectorCopy(&(*ent).s.origin[0], &mut (*ent).currentOrigin[0]);

        // if we didn't get a classname, don't bother spawning anything
        if G_CallSpawn(ent) == qfalse {
            G_FreeEntity(ent);
            return;
        }

        //Tag on the ICARUS scripting information only to valid recipients
        if IGameInterface_ValidEntity(Quake3Game(), ent) != 0 {
            IGameInterface_InitEntity(Quake3Game(), ent); //ICARUS_InitEnt( ent );

            if !(*ent).classname.is_null() && *(*ent).classname as c_int != 0 {
                if Q_strncmp(b"NPC_\0" as *const u8 as *const c_char, (*ent).classname, 4) != 0 {
                    //Not an NPC_spawner
                    G_ActivateBehavior(ent, BSET_SPAWN);
                }
            }
        }
    }
}

pub extern "C" fn G_SpawnSubBSPGEntityFromSpawnVars(posOffset: *const f32, angOffset: *const f32) {
    let mut i: c_int;
    let mut ent: *mut gentity_t;

    unsafe {
        // get the next free entity
        ent = G_Spawn();

        for i in 0..numSpawnVars {
            G_ParseField(*spawnVars.get_unchecked(i as usize).get_unchecked(0), *spawnVars.get_unchecked(i as usize).get_unchecked(1) as *const c_char, ent);
        }

        let mut i_out: c_int = 0;
        G_SpawnInt(b"notsingle\0" as *const u8 as *const c_char, b"0\0" as *const u8 as *const c_char, &mut i_out);
        if i_out != 0 || SpawnForCurrentDifficultySetting(ent) == qfalse {
            G_FreeEntity(ent);
            return;
        }

        VectorAdd(&(*ent).s.origin[0], posOffset, &mut (*ent).s.origin[0]);
        VectorAdd(&(*ent).s.angles[0], angOffset, &mut (*ent).s.angles[0]);

        VectorCopy(&(*ent).s.angles[0], &mut (*ent).s.apos.trBase[0]);
        VectorCopy(&(*ent).s.angles[0], &mut (*ent).currentAngles[0]);

        // move editor origin to pos
        VectorCopy(&(*ent).s.origin[0], &mut (*ent).s.pos.trBase[0]);
        VectorCopy(&(*ent).s.origin[0], &mut (*ent).currentOrigin[0]);

        // if we didn't get a classname, don't bother spawning anything
        if G_CallSpawn(ent) == qfalse {
            G_FreeEntity(ent);
            return;
        }

        //Tag on the ICARUS scripting information only to valid recipients
        if (*Quake3Game()).ValidEntity(ent) != 0 {
            (*Quake3Game()).InitEntity(ent); // ICARUS_InitEnt( ent );

            if !(*ent).classname.is_null() && *(*ent).classname as c_int != 0 {
                if Q_strncmp(b"NPC_\0" as *const u8 as *const c_char, (*ent).classname, 4) != 0 {
                    //Not an NPC_spawner
                    G_ActivateBehavior(ent, BSET_SPAWN);
                }
            }
        }
    }
}

/*
====================
G_AddSpawnVarToken
====================
*/
pub extern "C" fn G_AddSpawnVarToken(string: *const c_char) -> *mut c_char {
    let mut l: c_int;
    let mut dest: *mut c_char;

    unsafe {
        l = strlen(string) as c_int;
        if numSpawnVarChars + l + 1 > MAX_SPAWN_VARS_CHARS {
            G_Error(b"G_AddSpawnVarToken: MAX_SPAWN_VARS\0" as *const u8 as *const c_char);
        }

        dest = core::mem::addr_of_mut!(spawnVarChars)[numSpawnVarChars as usize] as *mut c_char;
        memcpy(dest as *mut c_void, string as *const c_void, (l + 1) as usize);

        numSpawnVarChars += l + 1;

        return dest;
    }
}

/*
====================
G_ParseSpawnVars

Parses a brace bounded set of key / value pairs out of the
level's entity strings into level.spawnVars[]

This does not actually spawn an entity.
====================
*/
pub extern "C" fn G_ParseSpawnVars(data: *mut *const c_char) -> qboolean {
    let mut keyname: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];
    let mut com_token: *const c_char;

    unsafe {
        numSpawnVars = 0;
        numSpawnVarChars = 0;

        // parse the opening brace
        com_token = COM_Parse(data);
        if data.is_null() || (*data).is_null() {
            // end of spawn string
            return qfalse;
        }
        if *com_token as c_int != '{' as c_int {
            G_Error(b"G_ParseSpawnVars: found %s when expecting {\0" as *const u8 as *const c_char, com_token);
        }

        // go through all the key / value pairs
        loop {
            // parse key
            com_token = COM_Parse(data);
            if *com_token as c_int == '}' as c_int {
                break;
            }
            if data.is_null() || (*data).is_null() {
                G_Error(b"G_ParseSpawnVars: EOF without closing brace\0" as *const u8 as *const c_char);
            }

            Q_strncpyz(keyname.as_mut_ptr(), com_token, MAX_STRING_CHARS as usize);

            // parse value
            com_token = COM_Parse(data);
            if *com_token as c_int == '}' as c_int {
                G_Error(b"G_ParseSpawnVars: closing brace without data\0" as *const u8 as *const c_char);
            }
            if data.is_null() || (*data).is_null() {
                G_Error(b"G_ParseSpawnVars: EOF without closing brace\0" as *const u8 as *const c_char);
            }
            if numSpawnVars == MAX_SPAWN_VARS {
                G_Error(b"G_ParseSpawnVars: MAX_SPAWN_VARS\0" as *const u8 as *const c_char);
            }
            *spawnVars.get_unchecked_mut(numSpawnVars as usize).get_unchecked_mut(0) = G_AddSpawnVarToken(keyname.as_ptr());
            *spawnVars.get_unchecked_mut(numSpawnVars as usize).get_unchecked_mut(1) = G_AddSpawnVarToken(com_token);
            numSpawnVars += 1;
        }

        return qtrue;
    }
}

pub static defaultStyles: [[&[u8]; 3]; LS_NUM_STYLES as usize] = [
    [	// 0 normal
        b"z",
        b"z",
        b"z"
    ],
    [	// 1 FLICKER (first variety)
        b"mmnmmommommnonmmonqnmmo",
        b"mmnmmommommnonmmonqnmmo",
        b"mmnmmommommnonmmonqnmmo"
    ],
    [	// 2 SLOW STRONG PULSE
        b"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcb",
        b"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcb",
        b"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcb"
    ],
    [	// 3 CANDLE (first variety)
        b"mmmmmaaaaammmmmaaaaaabcdefgabcdefg",
        b"mmmmmaaaaammmmmaaaaaabcdefgabcdefg",
        b"mmmmmaaaaammmmmaaaaaabcdefgabcdefg"
    ],
    [	// 4 FAST STROBE
        b"mamamamamama",
        b"mamamamamama",
        b"mamamamamama"
    ],
    [	// 5 GENTLE PULSE 1
        b"jklmnopqrstuvwxyzyxwvutsrqponmlkj",
        b"jklmnopqrstuvwxyzyxwvutsrqponmlkj",
        b"jklmnopqrstuvwxyzyxwvutsrqponmlkj"
    ],
    [	// 6 FLICKER (second variety)
        b"nmonqnmomnmomomno",
        b"nmonqnmomnmomomno",
        b"nmonqnmomnmomomno"
    ],
    [	// 7 CANDLE (second variety)
        b"mmmaaaabcdefgmmmmaaaammmaamm",
        b"mmmaaaabcdefgmmmmaaaammmaamm",
        b"mmmaaaabcdefgmmmmaaaammmaamm"
    ],
    [	// 8 CANDLE (third variety)
        b"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa",
        b"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa",
        b"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa"
    ],
    [	// 9 SLOW STROBE (fourth variety)
        b"aaaaaaaazzzzzzzz",
        b"aaaaaaaazzzzzzzz",
        b"aaaaaaaazzzzzzzz"
    ],
    [	// 10 FLUORESCENT FLICKER
        b"mmamammmmammamamaaamammma",
        b"mmamammmmammamamaaamammma",
        b"mmamammmmammamamaaamammma"
    ],
    [	// 11 SLOW PULSE NOT FADE TO BLACK
        b"abcdefghijklmnopqrrqponmlkjihgfedcba",
        b"abcdefghijklmnopqrrqponmlkjihgfedcba",
        b"abcdefghijklmnopqrrqponmlkjihgfedcba"
    ],
    [	// 12 FAST PULSE FOR JEREMY
        b"mkigegik",
        b"mkigegik",
        b"mkigegik"
    ],
    [	// 13 Test Blending
        b"abcdefghijklmqrstuvwxyz",
        b"zyxwvutsrqmlkjihgfedcba",
        b"aammbbzzccllcckkffyyggp"
    ],
    [	// 14
        b"",
        b"",
        b""
    ],
    [	// 15
        b"",
        b"",
        b""
    ],
    [	// 16
        b"",
        b"",
        b""
    ],
    [	// 17
        b"",
        b"",
        b""
    ],
    [	// 18
        b"",
        b"",
        b""
    ],
    [	// 19
        b"",
        b"",
        b""
    ],
    [	// 20
        b"",
        b"",
        b""
    ],
    [	// 21
        b"",
        b"",
        b""
    ],
    [	// 22
        b"",
        b"",
        b""
    ],
    [	// 23
        b"",
        b"",
        b""
    ],
    [	// 24
        b"",
        b"",
        b""
    ],
    [	// 25
        b"",
        b"",
        b""
    ],
    [	// 26
        b"",
        b"",
        b""
    ],
    [	// 27
        b"",
        b"",
        b""
    ],
    [	// 28
        b"",
        b"",
        b""
    ],
    [	// 29
        b"",
        b"",
        b""
    ],
    [	// 30
        b"",
        b"",
        b""
    ],
    [	// 31
        b"",
        b"",
        b""
    ]
];

/*QUAKED worldspawn (0 0 0) ?
Every map should have exactly one worldspawn.
"music"     path to WAV or MP3 files (e.g. "music\intro.mp3 music\loopfile.mp3")
"gravity"   800 is default gravity
"message"   Text to print during connection
"soundSet"  Ambient sound set to play
"spawnscript" runs this script

BSP Options
"gridsize"     size of lighting grid to "X Y Z". default="64 64 128"
"ambient"      amount of global light to add to each surf (uses _color)
"chopsize"     value for bsp on the maximum polygon / portal size
"distancecull" value for vis for the maximum viewing distance
"_minlight"   minimum lighting on a surf.  overrides _mingridlight and _minvertexlight

Game Options
"fog"          shader name of the global fog texture - must include the full path, such as "textures/rj/fog1"
"ls_Xr"	override lightstyle X with this pattern for Red.
"ls_Xg"	green (valid patterns are "a-z")
"ls_Xb"	blue (a is OFF, z is ON)
"breath"		Whether the entity's have breath puffs or not (0 = No, 1 = All, 2 = Just cold breath, 3 = Just under water bubbles ).
"clearstats" default 1, if 0 loading this map will not clear the stats for player
"tier_storyinfo" sets 'tier_storyinfo' cvar
*/
pub extern "C" fn SP_worldspawn() {
    let mut s: *mut c_char;
    let mut i: c_int;

    unsafe {
        g_entities[ENTITYNUM_WORLD].max_health = 0;

        for i in 0..numSpawnVars {
            if Q_stricmp(b"spawnscript\0" as *const u8 as *const c_char, *spawnVars.get_unchecked(i as usize).get_unchecked(0)) == 0 {
                //ONly let them set spawnscript, we don't want them setting an angle or something on the world.
                G_ParseField(*spawnVars.get_unchecked(i as usize).get_unchecked(0), *spawnVars.get_unchecked(i as usize).get_unchecked(1) as *const c_char, &mut g_entities[ENTITYNUM_WORLD]);
            }
            if Q_stricmp(b"region\0" as *const u8 as *const c_char, *spawnVars.get_unchecked(i as usize).get_unchecked(0)) == 0 {
                g_entities[ENTITYNUM_WORLD].s.radius = atoi(*spawnVars.get_unchecked(i as usize).get_unchecked(1)) as f32;
            }
            if Q_stricmp(b"distancecull\0" as *const u8 as *const c_char, *spawnVars.get_unchecked(i as usize).get_unchecked(0)) == 0 {
                g_entities[ENTITYNUM_WORLD].max_health = (((atoi(*spawnVars.get_unchecked(i as usize).get_unchecked(1)) as f32) * 0.7f32) as c_int);
            }
        }

        s = ptr::null_mut();
        G_SpawnString(b"classname\0" as *const u8 as *const c_char, b"\0" as *const u8 as *const c_char, &mut s);
        if Q_stricmp(s, b"worldspawn\0" as *const u8 as *const c_char) != 0 {
            G_Error(b"SP_worldspawn: The first entity isn't 'worldspawn'\0" as *const u8 as *const c_char);
        }

        // make some data visible to connecting client
        s = ptr::null_mut();
        G_SpawnString(b"music\0" as *const u8 as *const c_char, b"\0" as *const u8 as *const c_char, &mut s);
        // gi.SetConfigstring( CS_MUSIC, s );

        s = ptr::null_mut();
        G_SpawnString(b"message\0" as *const u8 as *const c_char, b"\0" as *const u8 as *const c_char, &mut s);
        // gi.SetConfigstring( CS_MESSAGE, s );				// map specific message

        s = ptr::null_mut();
        G_SpawnString(b"gravity\0" as *const u8 as *const c_char, b"800\0" as *const u8 as *const c_char, &mut s);
        // if (g_eSavedGameJustLoaded != eFULL)
        // {
        // 	gi.cvar_set( "g_gravity", s );
        // }

        s = ptr::null_mut();
        G_SpawnString(b"soundSet\0" as *const u8 as *const c_char, b"default\0" as *const u8 as *const c_char, &mut s);
        // gi.SetConfigstring( CS_AMBIENT_SET, s );

        //Lightstyles
        // gi.SetConfigstring(CS_LIGHT_STYLES+(LS_STYLES_START*3)+0, defaultStyles[0][0]);
        // gi.SetConfigstring(CS_LIGHT_STYLES+(LS_STYLES_START*3)+1, defaultStyles[0][1]);
        // gi.SetConfigstring(CS_LIGHT_STYLES+(LS_STYLES_START*3)+2, defaultStyles[0][2]);

        for i in 1..LS_NUM_STYLES {
            let mut temp: [c_char; 32] = [0; 32];
            let mut lengthRed: c_int;
            let mut lengthBlue: c_int;
            let mut lengthGreen: c_int;

            Com_sprintf(temp.as_mut_ptr(), 32, b"ls_%dr\0" as *const u8 as *const c_char, i);
            s = ptr::null_mut();
            G_SpawnString(temp.as_ptr(), defaultStyles[i as usize][0].as_ptr() as *const c_char, &mut s);
            lengthRed = strlen(s) as c_int;
            // gi.SetConfigstring(CS_LIGHT_STYLES+((i+LS_STYLES_START)*3)+0, s);

            Com_sprintf(temp.as_mut_ptr(), 32, b"ls_%dg\0" as *const u8 as *const c_char, i);
            s = ptr::null_mut();
            G_SpawnString(temp.as_ptr(), defaultStyles[i as usize][1].as_ptr() as *const c_char, &mut s);
            lengthGreen = strlen(s) as c_int;
            // gi.SetConfigstring(CS_LIGHT_STYLES+((i+LS_STYLES_START)*3)+1, s);

            Com_sprintf(temp.as_mut_ptr(), 32, b"ls_%db\0" as *const u8 as *const c_char, i);
            s = ptr::null_mut();
            G_SpawnString(temp.as_ptr(), defaultStyles[i as usize][2].as_ptr() as *const c_char, &mut s);
            lengthBlue = strlen(s) as c_int;
            // gi.SetConfigstring(CS_LIGHT_STYLES+((i+LS_STYLES_START)*3)+2, s);

            if lengthRed != lengthGreen || lengthGreen != lengthBlue {
                Com_Error(ERR_DROP, b"Style %d has inconsistent lengths: R %d, G %d, B %d\0" as *const u8 as *const c_char, i, lengthRed, lengthGreen, lengthBlue);
            }
        }

        s = ptr::null_mut();
        G_SpawnString(b"breath\0" as *const u8 as *const c_char, b"0\0" as *const u8 as *const c_char, &mut s);
        // gi.cvar_set( "cg_drawBreath", s );

        s = ptr::null_mut();
        G_SpawnString(b"clearstats\0" as *const u8 as *const c_char, b"1\0" as *const u8 as *const c_char, &mut s);
        // gi.cvar_set( "g_clearstats", s );

        if G_SpawnString(b"tier_storyinfo\0" as *const u8 as *const c_char, b"\0" as *const u8 as *const c_char, &mut s) != qfalse {
            // gi.cvar_set( "tier_storyinfo", s );
        }

        g_entities[ENTITYNUM_WORLD].s.number = ENTITYNUM_WORLD as c_int;
        g_entities[ENTITYNUM_WORLD].classname = b"worldspawn\0" as *const u8 as *const c_char as *mut c_char;
    }
}

/*
-------------------------
G_ParsePrecaches
-------------------------
*/

pub extern "C" fn G_ParsePrecaches() {
    let mut ent: *mut gentity_t = ptr::null_mut();

    //Clear any old lists
    unsafe {
        if as_preCacheMap.is_null() {
            // as_preCacheMap = new namePrecache_m;
        }

        // as_preCacheMap->clear();

        for i in 0..globals.num_entities {
            ent = &mut g_entities[i as usize];

            if !(*ent).soundSet.is_null() {
                // (*as_preCacheMap)[ (char *) ent->soundSet ] = 1;
            }
        }
    }
}

pub extern "C" fn G_ASPreCacheFree() {
    unsafe {
        if !as_preCacheMap.is_null() {
            // delete as_preCacheMap;
            as_preCacheMap = ptr::null_mut();
        }
    }
}

/*
==============
G_SpawnEntitiesFromString

Parses textual entity definitions out of an entstring and spawns gentities.
==============
*/
extern "C" {
    pub static mut num_waypoints: c_int;
    pub fn RG_RouteGen();
    pub static mut NPCsPrecached: qboolean;
}

pub extern "C" fn SP_bsp_worldspawn() -> qboolean {
    return qtrue;
}

pub extern "C" fn G_SubBSPSpawnEntitiesFromString(entityString: *const c_char, posOffset: *const f32, angOffset: *const f32) {
    let mut entities: *const c_char;

    unsafe {
        entities = entityString;

        // allow calls to G_Spawn*()
        spawning = qtrue;
        NPCsPrecached = qfalse;
        numSpawnVars = 0;

        // the worldspawn is not an actual entity, but it still
        // has a "spawn" function to perform any global setup
        // needed by a level (setting configstrings or cvars, etc)
        if G_ParseSpawnVars(&mut entities as *mut *const c_char) == qfalse {
            G_Error(b"SpawnEntities: no entities\0" as *const u8 as *const c_char);
        }

        // Skip this guy if its worldspawn fails
        if SP_bsp_worldspawn() == qfalse {
            return;
        }

        // parse ents
        while G_ParseSpawnVars(&mut entities as *mut *const c_char) != qfalse {
            G_SpawnSubBSPGEntityFromSpawnVars(posOffset, angOffset);
        }
    }
}

pub extern "C" fn G_SpawnEntitiesFromString(entityString: *const c_char) {
    let mut entities: *const c_char;

    unsafe {
        entities = entityString;

        // allow calls to G_Spawn*()
        spawning = qtrue;
        NPCsPrecached = qfalse;
        numSpawnVars = 0;

        // the worldspawn is not an actual entity, but it still
        // has a "spawn" function to perform any global setup
        // needed by a level (setting configstrings or cvars, etc)
        if G_ParseSpawnVars(&mut entities as *mut *const c_char) == qfalse {
            G_Error(b"SpawnEntities: no entities\0" as *const u8 as *const c_char);
        }

        SP_worldspawn();

        // parse ents
        while G_ParseSpawnVars(&mut entities as *mut *const c_char) != qfalse {
            G_SpawnGEntityFromSpawnVars();
        }

        //Search the entities for precache information
        G_ParsePrecaches();

        if !g_entities[ENTITYNUM_WORLD].behaviorSet.as_ptr().is_null() && !(*g_entities[ENTITYNUM_WORLD].behaviorSet.as_ptr()).is_null() {
            //World has a spawn script, but we don't want the world in ICARUS and running scripts,
            //so make a scriptrunner and start it going.
            let script_runner: *mut gentity_t = G_Spawn();
            if !script_runner.is_null() {
                (*script_runner).behaviorSet[BSET_USE as usize] = g_entities[ENTITYNUM_WORLD].behaviorSet[BSET_SPAWN as usize];
                (*script_runner).count = 1;
                (*script_runner).e_ThinkFunc = Some(thinkF_scriptrunner_run);
                (*script_runner).nextthink = level.time + 100;

                if IGameInterface_ValidEntity(Quake3Game(), script_runner) != 0 {
                    IGameInterface_InitEntity(Quake3Game(), script_runner); //ICARUS_InitEnt( script_runner );
                }
            }
        }

        //gi.Printf(S_COLOR_YELLOW"Total waypoints: %d\n", num_waypoints);
        //Automatically run routegen
        //RG_RouteGen();

        spawning = qfalse;			// any future calls to G_Spawn*() will be errors

        if (*g_delayedShutdown).integer != 0 && delayedShutDown != 0 {
            // assert(0);
            G_Error(b"Errors loading map, check the console for them.\0" as *const u8 as *const c_char);
        }
    }
}

// Stub types and additional extern declarations
#[repr(C)]
pub struct SavedGameJustLoaded_e {
    // stub
}

extern "C" {
    pub fn thinkF_scriptrunner_run(arg: *mut c_void);
}

// Additional fields for gentity_t that are referenced in this file
// These are stubs representing the full entity structure in g_local.h
pub struct GentityFields {
    // Extended entity fields referenced in this module
    // currentOrigin, currentAngles, soundSet, behaviorSet, count, e_ThinkFunc, nextthink
    // These would be defined in the complete gentity_t structure
}
