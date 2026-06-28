// g_local.h -- local definitions for game module

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// define GAME_INCLUDE so that g_public.h does not define the
// short, server-visible gclient_t and gentity_t structures,
// because we define the full size ones in this file
const GAME_INCLUDE: c_int = 1;

// Stub imports for external types from included headers
// #include "../ui/gameinfo.h"
// #include "g_shared.h"
// #include "anims.h"
// #include "dmstates.h"

// External type stubs - actual definitions in their respective modules
#[repr(C)]
pub struct gclient_t {
    // Defined elsewhere
}

#[repr(C)]
pub struct gentity_t {
    // Defined elsewhere
}

#[repr(C)]
pub struct game_export_t {
    // Defined elsewhere
}

#[repr(C)]
pub struct AIGroupInfo_t {
    // Defined elsewhere
}

#[repr(C)]
pub struct animation_t {
    // Defined elsewhere
}

#[repr(C)]
pub struct animevent_t {
    // Defined elsewhere
}

#[repr(C)]
pub struct stringID_table_t {
    // Defined elsewhere
}

// Type aliases for common types
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;
pub type fileHandle_t = c_int;
pub type soundChannel_t = c_int;
pub type team_t = c_int;
pub type usercmd_t = c_void; // Placeholder
pub type gitem_t = c_void; // Placeholder
pub type trace_t = c_void; // Placeholder
pub type SavedGameJustLoaded_e = c_int;

// Constants for MAX values
pub const MAX_QPATH: usize = 64;
pub const MAX_ANIMATIONS: usize = 1024;
pub const MAX_ANIM_EVENTS: usize = 30;
pub const MAX_ANIM_FILES: usize = 32;
pub const MAX_INTEREST_POINTS: usize = 64;
pub const MAX_COMBAT_POINTS: usize = 512;
pub const MAX_ALERT_EVENTS: usize = 32;
pub const MAX_SPAWN_VARS: usize = 64;
pub const MAX_SPAWN_VARS_CHARS: usize = 2048;
pub const MAX_FRAME_GROUPS: usize = 32; // Approximate - actual value from dmstates.h
pub const MAX_GENTITIES: usize = 1024; // Approximate
pub const MAX_REFTAGS: usize = 128;
pub const MAX_REFNAME: usize = 32;

//==================================================================

// the "gameversion" client command will print this plus compile date
pub const GAMEVERSION: &[u8] = b"base";

pub const BODY_QUEUE_SIZE: c_int = 8;

pub const Q3_INFINITE: c_int = 16777216;

pub const FRAMETIME: c_int = 100; // msec
pub const EVENT_VALID_MSEC: c_int = 300;
pub const CARNAGE_REWARD_TIME: c_int = 3000;

pub const INTERMISSION_DELAY_TIME: c_int = 1000;

pub const START_TIME_LINK_ENTS: c_int = FRAMETIME * 1; // time-delay after map start at which all ents have been spawned, so can link them
pub const START_TIME_FIND_LINKS: c_int = FRAMETIME * 2; // time-delay after map start at which you can find linked entities
pub const START_TIME_MOVERS_SPAWNED: c_int = FRAMETIME * 2; // time-delay after map start at which all movers should be spawned
pub const START_TIME_REMOVE_ENTS: c_int = FRAMETIME * 3; // time-delay after map start to remove temporary ents
pub const START_TIME_NAV_CALC: c_int = FRAMETIME * 4; // time-delay after map start to connect waypoints and calc routes
pub const START_TIME_FIND_WAYPOINT: c_int = FRAMETIME * 5; // time-delay after map start after which it's okay to try to find your best waypoint

// gentity->flags
pub const FL_SHIELDED: c_int = 0x00000001; // protected from all damage except lightsabers
pub const FL_DMG_BY_HEAVY_WEAP_ONLY: c_int = 0x00000002; // protected from all damage except heavy weap class missiles
pub const FL_DMG_BY_SABER_ONLY: c_int = 0x00000004; //protected from all damage except saber damage
pub const FL_GODMODE: c_int = 0x00000010;
pub const FL_NOTARGET: c_int = 0x00000020;
pub const FL_TEAMSLAVE: c_int = 0x00000400; // not the first on the team
pub const FL_NO_KNOCKBACK: c_int = 0x00000800;
pub const FL_DROPPED_ITEM: c_int = 0x00001000;
pub const FL_DONT_SHOOT: c_int = 0x00002000; // Can target him, but not shoot him
pub const FL_UNDYING: c_int = 0x00004000; // Takes damage down to 1 point, but does not die
// #define FL_OVERCHARGED			0x00008000	// weapon shot is an overcharged version....probably a lame place to be putting this flag...
pub const FL_LOCK_PLAYER_WEAPONS: c_int = 0x00010000; // player can't switch weapons... ask james if there's a better spot for this
pub const FL_DISINTEGRATED: c_int = 0x00020000; // marks that the corpse has already been disintegrated
pub const FL_FORCE_PULLABLE_ONLY: c_int = 0x00040000; // cannot be force pushed
pub const FL_NO_IMPACT_DMG: c_int = 0x00080000; // Will not take impact damage
pub const FL_OVERCHARGED_HEALTH: c_int = 0x00100000; // Reduce health back to max
pub const FL_NO_ANGLES: c_int = 0x00200000; // No bone angle overrides, no pitch or roll in full angles
pub const FL_RED_CROSSHAIR: c_int = 0x00400000; // Crosshair red on me


//Pointer safety utilities
#[inline]
pub fn VALID(a: *const c_void) -> bool {
    !a.is_null()
}

#[inline]
pub fn VALIDATE(a: *const c_void) {
    assert!(!a.is_null());
}

#[inline]
pub fn VALIDATEV(a: *const c_void) {
    if a.is_null() {
        assert!(false);
    }
}

#[inline]
pub fn VALIDATEB(a: *const c_void) -> bool {
    if a.is_null() {
        assert!(false);
        return false;
    }
    true
}

#[inline]
pub fn VALIDATEP(a: *const c_void) -> *const c_void {
    if a.is_null() {
        assert!(false);
        return core::ptr::null();
    }
    a
}

#[inline]
pub fn VALIDSTRING(a: *const c_char) -> bool {
    if a.is_null() {
        return false;
    }
    unsafe { *a != 0 }
}

//animations
#[repr(C)]
pub struct animFileSet_t {
    pub filename: [c_char; MAX_QPATH],
    pub animations: [animation_t; MAX_ANIMATIONS],
    pub torsoAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub legsAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub torsoAnimEventCount: u8,
    pub legsAnimEventCount: u8,
}

extern "C" {
    pub static mut animTable: [stringID_table_t; MAX_ANIMATIONS + 1];
}

//Interest points

#[repr(C)]
pub struct interestPoint_t {
    pub origin: vec3_t,
    pub target: *mut c_char,
}

//Combat points

#[repr(C)]
pub struct combatPoint_t {
    pub origin: vec3_t,
    pub flags: c_int,
    // 	pub NPC_targetname: *mut c_char,
    // 	pub team: team_t,
    pub occupied: qboolean,
    pub waypoint: c_int,
    pub dangerTime: c_int,
}

// Alert events

#[repr(C)]
#[derive(Clone, Copy)]
pub enum alertEventType_e {
    AET_SIGHT = 0,
    AET_SOUND = 1,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum alertEventLevel_e {
    AEL_MINOR = 0,          //Enemy responds to the sound, but only by looking
    AEL_SUSPICIOUS = 1,     //Enemy looks at the sound, and will also investigate it
    AEL_DISCOVERED = 2,     //Enemy knows the player is around, and will actively hunt
    AEL_DANGER = 3,         //Enemy should try to find cover
    AEL_DANGER_GREAT = 4,   //Enemy should run like hell!
}

// !!!!!!!!! LOADSAVE-affecting struct !!!!!!!!!!
#[repr(C)]
pub struct alertEvent_s {
    pub position: vec3_t,           //Where the event is located
    pub radius: f32,                //Consideration radius
    pub level: alertEventLevel_e,   //Priority level of the event
    pub r#type: alertEventType_e,   //Event type (sound,sight)
    pub owner: *mut gentity_t,      //Who made the sound
    pub light: f32,                 //ambient light level at point
    pub addLight: f32,              //additional light- makes it more noticable, even in darkness
    pub ID: c_int,                  //unique... if get a ridiculous number, this will repeat, but should not be a problem as it's just comparing it to your lastAlertID
    pub timestamp: c_int,           //when it was created
    pub onGround: qboolean,         //alert is on the ground (only used for sounds)
}

//
// this structure is cleared as each map is entered
//

#[repr(C)]
pub struct waypointData_t {
    pub targetname: [c_char; MAX_QPATH],
    pub target: [c_char; MAX_QPATH],
    pub target2: [c_char; MAX_QPATH],
    pub target3: [c_char; MAX_QPATH],
    pub target4: [c_char; MAX_QPATH],
    pub nodeID: c_int,
}

pub const WF_RAINING: c_int = 0x00000001; // raining
pub const WF_SNOWING: c_int = 0x00000002; // snowing
pub const WF_PUFFING: c_int = 0x00000004; // puffing something

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct level_locals_t {
    pub clients: *mut gclient_t,     // [maxclients]

    // store latched cvars here that we want to get at often
    pub maxclients: c_int,

    pub framenum: c_int,
    pub time: c_int,                 // in msec
    pub previousTime: c_int,         // so movers can back up when blocked

    pub globalTime: c_int,           // global time at level initialization

    pub mapname: [c_char; MAX_QPATH], // the server name (base1, etc)

    pub locationLinked: qboolean,    // target_locations get linked
    pub locationHead: *mut gentity_t, // head of the location list

    pub alertEvents: [alertEvent_s; MAX_ALERT_EVENTS],
    pub numAlertEvents: c_int,
    pub curAlertID: c_int,

    pub groups: [AIGroupInfo_t; MAX_FRAME_GROUPS],

    pub knownAnimFileSets: [animFileSet_t; MAX_ANIM_FILES],
    pub numKnownAnimFileSets: c_int,

    pub worldFlags: c_int,

    pub dmState: c_int,             //actually, we do want save/load the dynamic music state
    // =====================================
    //
    // NOTE!!!!!!   The only things beyond this point in the structure should be the ones you do NOT wish to be
    //					affected by loading saved-games. Since loading a game first starts the map and then loads
    //					over things like entities etc then these fields are usually the ones setup by the map loader.
    //					If they ever get modified in-game let me know and I'll include them in the save. -Ste
    //
    // #define LEVEL_LOCALS_T_SAVESTOP logFile	// name of whichever field is next below this in the source

    pub logFile: fileHandle_t,

    //Interest points- squadmates automatically look at these if standing around and close to them
    pub interestPoints: [interestPoint_t; MAX_INTEREST_POINTS],
    pub numInterestPoints: c_int,

    //Combat points- NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
    pub combatPoints: [combatPoint_t; MAX_COMBAT_POINTS],
    pub numCombatPoints: c_int,
    pub spawntarget: [c_char; MAX_QPATH], // the targetname of the spawnpoint you want the player to start at

    pub dmDebounceTime: c_int,
    pub dmBeatTime: c_int,

    pub mNumBSPInstances: c_int,
    pub mBSPInstanceDepth: c_int,
    pub mOriginAdjust: vec3_t,
    pub mRotationAdjust: f32,
    pub mTargetAdjust: *mut c_char,
    pub hasBspInstances: qboolean,
}

extern "C" {
    pub static mut level: level_locals_t;
    pub static mut globals: game_export_t;

    pub static mut g_gravity: *mut cvar_t;
    pub static mut g_speed: *mut cvar_t;
    pub static mut g_cheats: *mut cvar_t;
    pub static mut g_developer: *mut cvar_t;
    pub static mut g_knockback: *mut cvar_t;
    pub static mut g_inactivity: *mut cvar_t;
    pub static mut g_debugMove: *mut cvar_t;
    pub static mut g_subtitles: *mut cvar_t;
    pub static mut g_removeDoors: *mut cvar_t;

    pub static mut g_ICARUSDebug: *mut cvar_t;

    pub static mut g_npcdebug: *mut cvar_t;

    pub static mut player: *mut gentity_t;
}

// Stub for cvar_t - actual definition elsewhere
#[repr(C)]
pub struct cvar_t {
    // Defined elsewhere
}

//
// g_spawn.c
//
extern "C" {
    pub fn G_SpawnField(uiField: c_int, ppKey: *mut *mut c_char, ppValue: *mut *mut c_char) -> qboolean;
    pub fn G_SpawnString(key: *const c_char, defaultString: *const c_char, out: *mut *mut c_char) -> qboolean;
    // spawn string returns a temporary reference, you must CopyString() if you want to keep it
    pub fn G_SpawnFloat(key: *const c_char, defaultString: *const c_char, out: *mut f32) -> qboolean;
    pub fn G_SpawnInt(key: *const c_char, defaultString: *const c_char, out: *mut c_int) -> qboolean;
    pub fn G_SpawnVector(key: *const c_char, defaultString: *const c_char, out: *mut f32) -> qboolean;
    pub fn G_SpawnVector4(key: *const c_char, defaultString: *const c_char, out: *mut f32) -> qboolean;
    pub fn G_SpawnAngleHack(key: *const c_char, defaultString: *const c_char, out: *mut f32) -> qboolean;
    pub fn G_SpawnEntitiesFromString(entities: *const c_char);
}

//
// g_cmds.c
//
extern "C" {
    pub fn Cmd_Score_f(ent: *mut gentity_t);
}

//
// g_items.c
//
extern "C" {
    pub fn G_RunItem(ent: *mut gentity_t);
    pub fn RespawnItem(ent: *mut gentity_t);

    pub fn UseHoldableItem(ent: *mut gentity_t);
    pub fn PrecacheItem(it: *mut gitem_t);
    pub fn Drop_Item(ent: *mut gentity_t, item: *mut gitem_t, angle: f32, copytarget: qboolean) -> *mut gentity_t;
    pub fn SetRespawn(ent: *mut gentity_t, delay: f32);
    pub fn G_SpawnItem(ent: *mut gentity_t, item: *mut gitem_t);
    pub fn FinishSpawningItem(ent: *mut gentity_t);
    pub fn Think_Weapon(ent: *mut gentity_t);
    pub fn ArmorIndex(ent: *mut gentity_t) -> c_int;
    pub fn Add_Ammo(ent: *mut gentity_t, weapon: c_int, count: c_int);
    pub fn Touch_Item(ent: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);

    pub fn ClearRegisteredItems();
    pub fn RegisterItem(item: *mut gitem_t);
    pub fn SaveRegisteredItems();
}

//
// g_utils.c
//
extern "C" {
    pub fn G_ModelIndex(name: *const c_char) -> c_int;
    pub fn G_SoundIndex(name: *const c_char) -> c_int;
    /*
    Ghoul2 Insert Start
    */
    pub fn G_SkinIndex(name: *const c_char) -> c_int;
    pub fn G_SetBoltSurfaceRemoval(entNum: c_int, modelIndex: c_int, boltIndex: c_int, surfaceIndex: c_int, duration: f32);
    /*
    Ghoul2 Insert End
    */

    pub fn G_EffectIndex(name: *const c_char) -> c_int;
    pub fn G_PlayEffect(name: *const c_char, origin: *const vec3_t);
    pub fn G_PlayEffect_clientNum(name: *const c_char, clientNum: c_int);
    pub fn G_PlayEffect_origin_fwd(name: *const c_char, origin: *const vec3_t, fwd: *const vec3_t);
    pub fn G_PlayEffect_origin_axis(name: *const c_char, origin: *const vec3_t, axis: *const vec3_t);
    pub fn G_PlayEffect_fxID(fxID: c_int, origin: *const vec3_t);
    pub fn G_PlayEffect_fxID_fwd(fxID: c_int, origin: *const vec3_t, fwd: *const vec3_t);
    pub fn G_PlayEffect_fxID_axis(fxID: c_int, origin: *const vec3_t, axis: *const vec3_t);
    pub fn G_PlayEffect_fxID_bolt(fxID: c_int, modelIndex: c_int, boltIndex: c_int, entNum: c_int, origin: *const vec3_t, iLoopTime: c_int, isRelative: qboolean);
    pub fn G_PlayEffect_fxID_entNum(fxID: c_int, entNum: c_int, fwd: *const vec3_t);
    #[cfg(feature = "_IMMERSION")]
    pub fn G_PlayEffect_clientNum_fwd(name: *const c_char, clientNum: c_int, origin: *const vec3_t, fwd: *const vec3_t);
    #[cfg(feature = "_IMMERSION")]
    pub fn G_PlayEffect_fxID_clientNum_fwd(fxID: c_int, clientNum: c_int, origin: *const vec3_t, fwd: *const vec3_t);
    pub fn G_StopEffect(fxID: c_int, modelIndex: c_int, boltIndex: c_int, entNum: c_int);
    pub fn G_StopEffect_name(name: *const c_char, modelIndex: c_int, boltIndex: c_int, entNum: c_int);

    pub fn G_BSPIndex(name: *mut c_char) -> c_int;

    pub fn G_KillBox(ent: *mut gentity_t);
    pub fn G_Find(from: *mut gentity_t, fieldofs: c_int, match_: *const c_char) -> *mut gentity_t;
    pub fn G_RadiusList(origin: vec3_t, radius: f32, ignore: *mut gentity_t, takeDamage: qboolean, ent_list: *mut *mut gentity_t) -> c_int;
    pub fn G_PickTarget(targetname: *mut c_char) -> *mut gentity_t;
    pub fn G_UseTargets(ent: *mut gentity_t, activator: *mut gentity_t);
    pub fn G_UseTargets2(ent: *mut gentity_t, activator: *mut gentity_t, string: *const c_char);
    pub fn G_SetMovedir(angles: vec3_t, movedir: vec3_t);

    pub fn G_InitGentity(e: *mut gentity_t, bFreeG2: qboolean);
    pub fn G_Spawn() -> *mut gentity_t;
    pub fn G_TempEntity(origin: *const vec3_t, event: c_int) -> *mut gentity_t;
    pub fn G_Sound(ent: *mut gentity_t, soundIndex: c_int);
    pub fn G_FreeEntity(e: *mut gentity_t);

    #[cfg(feature = "_IMMERSION")]
    pub fn G_ForceIndex(name: *const c_char, channel: c_int) -> c_int;
    #[cfg(feature = "_IMMERSION")]
    pub fn G_Force(ent: *mut gentity_t, forceIndex: c_int);
    #[cfg(feature = "_IMMERSION")]
    pub fn G_ForceArea(ent: *mut gentity_t, forceIndex: c_int);
    #[cfg(feature = "_IMMERSION")]
    pub fn G_ForceBroadcast(ent: *mut gentity_t, forceIndex: c_int);
    #[cfg(feature = "_IMMERSION")]
    pub fn G_ForceStop(ent: *mut gentity_t, forceIndex: c_int);
    pub fn G_TouchTriggers(ent: *mut gentity_t);
    pub fn G_TouchTeamClients(ent: *mut gentity_t);
    pub fn G_TouchSolids(ent: *mut gentity_t);

    pub fn vtos(v: *const vec3_t) -> *mut c_char;

    pub fn vectoyaw(vec: *const vec3_t) -> f32;

    pub fn G_AddEvent(ent: *mut gentity_t, event: c_int, eventParm: c_int);
    pub fn G_SetOrigin(ent: *mut gentity_t, origin: *const vec3_t);
    pub fn G_SetAngles(ent: *mut gentity_t, angles: *const vec3_t);

    pub fn G_DebugLine(A: vec3_t, B: vec3_t, duration: c_int, color: c_int, deleteornot: qboolean);
}

//
// g_combat.c
//
extern "C" {
    pub fn CanDamage(targ: *mut gentity_t, origin: *const vec3_t) -> qboolean;
    pub fn G_Damage(targ: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, dir: *const vec3_t, point: *const vec3_t, damage: c_int, dflags: c_int, mod_: c_int, hitLoc: c_int);
    pub fn G_RadiusDamage(origin: *const vec3_t, attacker: *mut gentity_t, damage: f32, radius: f32, ignore: *mut gentity_t, mod_: c_int);
    pub fn TossClientItems(self_: *mut gentity_t) -> *mut gentity_t;
    pub fn ExplodeDeath_Wait(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, meansOfDeath: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn ExplodeDeath(self_: *mut gentity_t);
    pub fn GoExplodeDeath(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn G_ApplyKnockback(targ: *mut gentity_t, newDir: *const vec3_t, knockback: f32);
    pub fn G_Throw(targ: *mut gentity_t, newDir: *const vec3_t, push: f32);
}

// damage flags
pub const DAMAGE_RADIUS: c_int = 0x00000001; // damage was indirect
pub const DAMAGE_NO_ARMOR: c_int = 0x00000002; // armour does not protect from this damage
pub const DAMAGE_NO_KNOCKBACK: c_int = 0x00000008; // do not affect velocity, just view angles
pub const DAMAGE_NO_HIT_LOC: c_int = 0x00000010; // do not modify damage by hit loc
pub const DAMAGE_NO_PROTECTION: c_int = 0x00000020; // armor, shields, invulnerability, and godmode have no effect
pub const DAMAGE_EXTRA_KNOCKBACK: c_int = 0x00000040; // add extra knockback to this damage
pub const DAMAGE_DEATH_KNOCKBACK: c_int = 0x00000080; // only does knockback on death of target
pub const DAMAGE_IGNORE_TEAM: c_int = 0x00000100; // damage is always done, regardless of teams
pub const DAMAGE_NO_DAMAGE: c_int = 0x00000200; // do no actual damage but react as if damage was taken
pub const DAMAGE_DISMEMBER: c_int = 0x00000400; // do dismemberment
pub const DAMAGE_NO_KILL: c_int = 0x00000800; // do damage, but don't kill them
pub const DAMAGE_HEAVY_WEAP_CLASS: c_int = 0x00001000; // doing heavy weapon type damage, certain objects may only take damage by missiles containing this flag
pub const DAMAGE_CUSTOM_HUD: c_int = 0x00002000; // really dumb, but....
pub const DAMAGE_IMPACT_DIE: c_int = 0x00004000; // if a vehicle hits a wall it should instantly die
pub const DAMAGE_DIE_ON_IMPACT: c_int = 0x00008000; // ignores force-power based protection
pub const DAMAGE_SABER_KNOCKBACK1: c_int = 0x00010000; // scale knockback based on saber1's knockbackScale value
pub const DAMAGE_SABER_KNOCKBACK2: c_int = 0x00020000; // scale knockback based on saber2's knockbackScale value
pub const DAMAGE_SABER_KNOCKBACK1_B2: c_int = 0x00040000; // scale knockback based on saber1's knockbackScale2 value
pub const DAMAGE_SABER_KNOCKBACK2_B2: c_int = 0x00080000; // scale knockback based on saber2's knockbackScale2 value

//
// g_missile.c
//
extern "C" {
    pub fn G_RunMissile(ent: *mut gentity_t);

    pub fn fire_blaster(self_: *mut gentity_t, start: vec3_t, aimdir: vec3_t) -> *mut gentity_t;
    pub fn fire_plasma(self_: *mut gentity_t, start: vec3_t, aimdir: vec3_t) -> *mut gentity_t;
    pub fn fire_grenade(self_: *mut gentity_t, start: vec3_t, aimdir: vec3_t) -> *mut gentity_t;
    pub fn fire_rocket(self_: *mut gentity_t, start: vec3_t, dir: vec3_t) -> *mut gentity_t;
}


//
// g_mover.c
//
pub const MOVER_START_ON: c_int = 1;
pub const MOVER_FORCE_ACTIVATE: c_int = 2;
pub const MOVER_CRUSHER: c_int = 4;
pub const MOVER_TOGGLE: c_int = 8;
pub const MOVER_LOCKED: c_int = 16;
pub const MOVER_GOODIE: c_int = 32;
pub const MOVER_PLAYER_USE: c_int = 64;
pub const MOVER_INACTIVE: c_int = 128;

extern "C" {
    pub fn G_RunMover(ent: *mut gentity_t);
}


//
// g_misc.c
//
extern "C" {
    pub fn TeleportPlayer(player: *mut gentity_t, origin: vec3_t, angles: vec3_t);
}


//
// g_weapon.c
//
//void CalcMuzzlePoint ( gentity_t *ent, vec3_t forward, vec3_t right, vec3_t up, vec3_t muzzlePoint );
//void SnapVectorTowards( vec3_t v, vec3_t to );
//qboolean CheckGauntletAttack( gentity_t *ent );
extern "C" {
    pub fn WP_LoadWeaponParms();

    pub fn IT_LoadItemParms();
}

//
// g_client.c
//
extern "C" {
    pub fn PickTeam(ignoreClientNum: c_int) -> team_t;
    pub fn SetClientViewAngle(ent: *mut gentity_t, angle: vec3_t);
    pub fn SelectSpawnPoint(avoidPoint: vec3_t, team: team_t, origin: vec3_t, angles: vec3_t) -> *mut gentity_t;
    pub fn respawn(ent: *mut gentity_t);
    pub fn InitClientPersistant(client: *mut gclient_t);
    pub fn InitClientResp(client: *mut gclient_t);
    pub fn ClientSpawn(ent: *mut gentity_t, eSavedGameJustLoaded: SavedGameJustLoaded_e) -> qboolean;
    pub fn player_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn AddScore(ent: *mut gentity_t, score: c_int);
    pub fn SpotWouldTelefrag(spot: *mut gentity_t, checkteam: team_t) -> qboolean;
    pub fn G_RemoveWeaponModels(ent: *mut gentity_t);
}

//
// g_svcmds.c
//
extern "C" {
    pub fn ConsoleCommand() -> qboolean;
}

//
// g_weapon.c
//
extern "C" {
    pub fn FireWeapon(ent: *mut gentity_t, alt_fire: qboolean);
}

//
// p_hud.c
//
extern "C" {
    pub fn MoveClientToIntermission(client: *mut gentity_t);
    pub fn G_SetStats(ent: *mut gentity_t);
    pub fn DeathmatchScoreboardMessage(client: *mut gentity_t);
}

//
// g_cmds.c
//
extern "C" {
    pub fn G_SayTo(ent: *mut gentity_t, other: *mut gentity_t, mode: c_int, color: c_int, name: *const c_char, message: *const c_char);
}

//
// g_pweapon.c
//


//
// g_main.c
//
extern "C" {
    pub fn G_RunThink(ent: *mut gentity_t);
    pub fn G_Error(fmt: *const c_char, ...);
    pub fn SetInUse(ent: *mut gentity_t);
    pub fn ClearInUse(ent: *mut gentity_t);
    pub fn PInUse(entNum: c_int) -> qboolean;
    pub fn PInUse2(ent: *mut gentity_t) -> qboolean;
    pub fn WriteInUseBits();
    pub fn ReadInUseBits();
}

//
// g_nav.cpp
//
extern "C" {
    pub fn Svcmd_Nav_f();
}

//
// g_squad.cpp
//
extern "C" {
    pub fn Svcmd_Comm_f();
    pub fn Svcmd_Hail_f();
    pub fn Svcmd_Form_f();
}


//
// g_utils.cpp
//
extern "C" {
    pub fn Svcmd_Use_f();
    pub fn G_SoundOnEnt(ent: *mut gentity_t, channel: soundChannel_t, soundPath: *const c_char);
    pub fn G_SoundIndexOnEnt(ent: *mut gentity_t, channel: soundChannel_t, index: c_int);
}

//
// g_weapons.cpp
//

//
// g_client.c
//
extern "C" {
    pub fn ClientConnect(clientNum: c_int, firstTime: qboolean, eSavedGameJustLoaded: SavedGameJustLoaded_e) -> *mut c_char;
    pub fn ClientUserinfoChanged(clientNum: c_int);
    pub fn ClientDisconnect(clientNum: c_int);
    pub fn ClientBegin(clientNum: c_int, cmd: *mut usercmd_t, eSavedGameJustLoaded: SavedGameJustLoaded_e);
    pub fn ClientCommand(clientNum: c_int);
}

//
// g_active.c
//
extern "C" {
    pub fn ClientThink(clientNum: c_int, cmd: *mut usercmd_t);
    pub fn ClientEndFrame(ent: *mut gentity_t);
}

//
// g_inventory.cpp
//
extern "C" {
    pub fn INV_GoodieKeyGive(target: *mut gentity_t) -> qboolean;
    pub fn INV_GoodieKeyTake(target: *mut gentity_t) -> qboolean;
    pub fn INV_GoodieKeyCheck(target: *mut gentity_t) -> c_int;
    pub fn INV_SecurityKeyGive(target: *mut gentity_t, keyname: *const c_char) -> qboolean;
    pub fn INV_SecurityKeyTake(target: *mut gentity_t, keyname: *mut c_char);
    pub fn INV_SecurityKeyCheck(target: *mut gentity_t, keyname: *mut c_char) -> qboolean;
}

//
// g_team.c
//
extern "C" {
    pub fn OnSameTeam(ent1: *mut gentity_t, ent2: *mut gentity_t) -> qboolean;
}


//
// g_mem.c
//
extern "C" {
    pub fn G_Alloc(size: c_int) -> *mut c_void;
    pub fn G_InitMemory();
    pub fn Svcmd_GameMem_f();
}

//
// g_session.c
//
extern "C" {
    pub fn G_ReadSessionData(client: *mut gclient_t);
    pub fn G_InitSessionData(client: *mut gclient_t, userinfo: *mut c_char);

    pub fn G_InitWorldSession();
    pub fn G_WriteSessionData();
}


//
// NPC_senses.cpp
//
extern "C" {
    pub fn AddSightEvent(owner: *mut gentity_t, position: vec3_t, radius: f32, alertLevel: alertEventLevel_e, addLight: f32);
    pub fn AddSoundEvent(owner: *mut gentity_t, position: vec3_t, radius: f32, alertLevel: alertEventLevel_e, needLOS: qboolean, onGround: qboolean);
    pub fn G_CheckForDanger(self_: *mut gentity_t, alertEvent: c_int) -> qboolean;
    pub fn G_CheckAlertEvents(self_: *mut gentity_t, checkSight: qboolean, checkSound: qboolean, maxSeeDist: f32, maxHearDist: f32, ignoreAlert: c_int, mustHaveOwner: qboolean, minAlertLevel: alertEventLevel_e, onGroundOnly: qboolean) -> c_int;
    pub fn G_ClearLOS_start_end(self_: *mut gentity_t, start: *const vec3_t, end: *const vec3_t) -> qboolean;
    pub fn G_ClearLOS_ent_end(self_: *mut gentity_t, ent: *mut gentity_t, end: *const vec3_t) -> qboolean;
    pub fn G_ClearLOS_start_ent(self_: *mut gentity_t, start: *const vec3_t, ent: *mut gentity_t) -> qboolean;
    pub fn G_ClearLOS_ent(self_: *mut gentity_t, ent: *mut gentity_t) -> qboolean;
    pub fn G_ClearLOS_end(self_: *mut gentity_t, end: *const vec3_t) -> qboolean;
}

//============================================================================

//Tags

// Reference tags

pub const RTF_NONE: c_int = 0;
pub const RTF_NAVGOAL: c_int = 0x00000001;

#[repr(C)]
pub struct reference_tag_s {
    pub name: [c_char; MAX_REFNAME],
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub flags: c_int,   //Just in case
    pub radius: c_int,  //For nav goals
}

extern "C" {
    pub fn TAG_Init();
    pub fn TAG_Add(name: *const c_char, owner: *const c_char, origin: vec3_t, angles: vec3_t, radius: c_int, flags: c_int) -> *mut reference_tag_s;

    pub fn TAG_GetOrigin(owner: *const c_char, name: *const c_char, origin: vec3_t) -> c_int;
    pub fn TAG_GetAngles(owner: *const c_char, name: *const c_char, angles: vec3_t) -> c_int;
    pub fn TAG_GetRadius(owner: *const c_char, name: *const c_char) -> c_int;
    pub fn TAG_GetFlags(owner: *const c_char, name: *const c_char) -> c_int;

    pub fn TAG_ShowTags(flags: c_int);
}

// Reference tags END

extern "C" {
    pub fn G_NewString(string: *const c_char) -> *mut c_char;
}

// some stuff for savegames...
//
extern "C" {
    pub fn WriteLevel(qbAutosave: qboolean);
    pub fn ReadLevel(qbAutosave: qboolean, qbLoadTransition: qboolean);
    pub fn GameAllowedToSaveHere() -> qboolean;

    pub fn G_ActivateBehavior(ent: *mut gentity_t, bset: c_int) -> qboolean;
}

//Timing information
extern "C" {
    pub fn TIMER_Clear();
    pub fn TIMER_Clear_idx(idx: c_int);
    pub fn TIMER_Save();
    pub fn TIMER_Load();
    pub fn TIMER_Set(ent: *mut gentity_t, identifier: *const c_char, duration: c_int);
    pub fn TIMER_Get(ent: *mut gentity_t, identifier: *const c_char) -> c_int;
    pub fn TIMER_Done(ent: *mut gentity_t, identifier: *const c_char) -> qboolean;
    pub fn TIMER_Start(self_: *mut gentity_t, identifier: *const c_char, duration: c_int) -> qboolean;
    pub fn TIMER_Done2(ent: *mut gentity_t, identifier: *const c_char, remove: qboolean) -> qboolean;
    pub fn TIMER_Exists(ent: *mut gentity_t, identifier: *const c_char) -> qboolean;
    pub fn TIMER_Remove(ent: *mut gentity_t, identifier: *const c_char);
}

extern "C" {
    pub fn NPC_GetHFOVPercentage(spot: vec3_t, from: vec3_t, facing: vec3_t, hFOV: f32) -> f32;
    pub fn NPC_GetVFOVPercentage(spot: vec3_t, from: vec3_t, facing: vec3_t, vFOV: f32) -> f32;
}

#[cfg(feature = "_XBOX")]
extern "C" {
    // data used for NPC water detection
    pub static mut npcsToUpdate: [i16; 64];    // queue of npcs
    pub static mut npcsToUpdateTop: i16;       // top of the queue
    pub static mut npcsToUpdateCount: i16;     // number of npcs in the queue
}
