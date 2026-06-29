// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

// #include "g_local.h"
// #include "g_functions.h"
// #include "..\cgame\cg_media.h"

use core::ffi::{c_int, c_char, c_void};

// Placeholder extern declarations for functions from C
// These will need to be properly linked at build time
#[allow(non_snake_case)]
pub mod externs {
    use core::ffi::{c_int, c_char, c_void};

    #[repr(C)]
    pub struct vec3_t {
        pub data: [f32; 3],
    }

    #[repr(C)]
    pub struct gentity_t {
        // Placeholder - actual structure should be defined elsewhere
        _unused: c_void,
    }

    #[repr(C)]
    pub struct trace_t {
        // Placeholder
        _unused: c_void,
    }

    // client side shortcut hacks from cg_local.h
    extern "C" {
        //pub fn CG_SurfaceExplosion( origin: *mut vec3_t, normal: *mut vec3_t, radius: f32, shake_speed: f32, smoke: crate::qboolean );
        pub fn CG_MiscModelExplosion(mins: *mut vec3_t, maxs: *mut vec3_t, size: c_int, chunkType: i32);
        pub fn CG_Chunks(owner: c_int, origin: *mut vec3_t, normal: *const vec3_t, mins: *const vec3_t, maxs: *const vec3_t,
                                    speed: f32, numChunks: c_int, chunkType: i32, customChunk: c_int, baseScale: f32, customSound: c_int);
        pub fn G_SetEnemy(this: *mut gentity_t, enemy: *mut gentity_t);

        pub fn G_CreateObject(owner: *mut gentity_t, origin: *mut vec3_t, angles: *mut vec3_t, modelIndex: c_int, frame: c_int, trType: i32, effectID: c_int) -> *mut gentity_t;
    }

    pub static mut player_locked: bool = false;
}

// external declarations
extern "C" {
    pub static mut g_entities: [gentity_t; 4096]; // MAX_GENTITIES
    pub static mut level: level_t;
    pub fn G_EffectIndex(effectName: *const c_char) -> c_int;
    pub fn G_SoundIndex(soundName: *const c_char) -> c_int;
    pub fn G_ModelIndex(modelName: *const c_char) -> c_int;
    pub fn G_Damage(target: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t,
                    dir: *const [f32; 3], point: *const [f32; 3], damage: c_int, dflags: c_int, mod_: c_int);
    pub fn G_UseTargets(ent: *mut gentity_t, activator: *mut gentity_t);
    pub fn G_UseTargets2(ent: *mut gentity_t, activator: *mut gentity_t, target: *const c_char);
    pub fn G_ActivateBehavior(ent: *mut gentity_t, bset: c_int);
    pub fn VectorSet(v: *mut [f32; 3], x: f32, y: f32, z: f32);
    pub fn VectorSubtract(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorAdd(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    pub fn VectorScale(in_: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    pub fn VectorMA(veca: *const [f32; 3], scale: f32, vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    pub fn VectorLength(v: *const [f32; 3]) -> f32;
    pub fn VectorClear(v: *mut [f32; 3]);
    pub fn VectorCompare(v1: *const [f32; 3], v2: *const [f32; 3]) -> bool;
    pub fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    pub fn DotProduct(v1: *const [f32; 3], v2: *const [f32; 3]) -> f32;
    pub fn SnapVector(v: *mut [f32; 3]);
    pub fn random() -> f32;
    pub fn Q_irand(min: c_int, max: c_int) -> c_int;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_strncpyz(dst: *mut c_char, src: *const c_char, len: usize);
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    pub fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn COM_DefaultExtension(path: *mut c_char, pathlen: usize, ext: *const c_char);
    pub fn AddSightEvent(owner: *mut gentity_t, pos: *const [f32; 3], radius: c_int, eventType: c_int, extraInfo: c_int);
    pub fn AddSoundEvent(owner: *mut gentity_t, pos: *const [f32; 3], radius: c_int, eventType: c_int, b1: bool, b2: bool);
    pub fn G_RadiusDamage(origin: *const [f32; 3], inflictor: *mut gentity_t, damage: c_int, radius: c_int,
                          attacker: *mut gentity_t, mod_: c_int);
    pub fn G_TempEntity(pos: *const [f32; 3], event: c_int) -> *mut gentity_t;
    pub fn G_FreeEntity(ent: *mut gentity_t);
    pub fn G_Spawn() -> *mut gentity_t;
    pub fn G_SpawnFloat(key: *const c_char, defaultValue: *const c_char, out: *mut f32) -> bool;
    pub fn G_SpawnInt(key: *const c_char, defaultValue: *const c_char, out: *mut c_int) -> bool;
    pub fn G_SpawnVector(key: *const c_char, defaultValue: *const c_char, out: *mut [f32; 3]) -> bool;
    pub fn G_SpawnString(key: *const c_char, defaultValue: *const c_char, out: *mut *const c_char) -> bool;
    pub fn G_SetOrigin(ent: *mut gentity_t, origin: *const [f32; 3]);
    pub fn G_SetAngles(ent: *mut gentity_t, angles: *const [f32; 3]);
    pub fn G_PlayEffect(effectName: *const c_char, origin: *const [f32; 3]);
    pub fn G_PlayEffect(effectIndex: c_int, origin: *const [f32; 3], angles: *const [f32; 3]);
    pub fn G_StopEffect(effectIndex: c_int, model: c_int, bolt: c_int, entnum: c_int);
    pub fn G_Sound(ent: *mut gentity_t, soundIndex: c_int);
    pub fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundName: *const c_char);
    pub fn GetIDForString(table: *const c_void, string: *const c_char) -> c_int;
    pub fn G_Find(from: *mut gentity_t, fieldofs: usize, match_: *const c_char) -> *mut gentity_t;
    pub fn G_Error(format: *const c_char, ...);
    pub fn G_Printf(format: *const c_char, ...);
    pub fn RegisterItem(item: *const c_void);
    pub fn FindItemForWeapon(weapon: c_int) -> *const c_void;

    pub static mut gi: game_import_t;
}

#[repr(C)]
pub struct level_t {
    pub time: c_int,
    _pad: [u8; 1024], // Placeholder for actual fields
}

#[repr(C)]
pub struct game_import_t {
    _unused: c_void,
}

#[repr(C)]
pub struct gentity_t {
    pub s: entity_state_t,
    pub inuse: bool,
    pub linkcount: c_int,
    pub linknode: *mut c_void,
    pub unlinknode: *mut c_void,
    pub svFlags: c_int,
    pub solid: c_int,
    pub contents: c_int,
    pub clipmask: c_int,
    pub owner: *mut gentity_t,
    pub enemy: *mut gentity_t,
    pub activator: *mut gentity_t,
    pub currentOrigin: [f32; 3],
    pub currentAngles: [f32; 3],
    pub absmin: [f32; 3],
    pub absmax: [f32; 3],
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub pos1: [f32; 3],
    pub pos2: [f32; 3],
    pub lastOrigin: [f32; 3],
    pub radius: f32,
    pub mass: f32,
    pub model: *const c_char,
    pub model2: *const c_char,
    pub classname: *const c_char,
    pub target: *const c_char,
    pub target2: *const c_char,
    pub target3: *const c_char,
    pub target4: *const c_char,
    pub paintarget: *const c_char,
    pub team: *const c_char,
    pub targetname: *const c_char,
    pub NPC_targetname: *const c_char,
    pub health: c_int,
    pub max_health: c_int,
    pub takedamage: bool,
    pub damage: c_int,
    pub splashDamage: c_int,
    pub splashRadius: c_int,
    pub spawnflags: c_int,
    pub count: c_int,
    pub noise_index: c_int,
    pub nextthink: c_int,
    pub delay: f32,
    pub wait: f32,
    pub painDebounceTime: c_int,
    pub attackDebounceTime: c_int,
    pub noDamageTeam: c_int,
    pub flags: c_int,
    pub material: i32,
    pub fly_sound_debounce_time: c_int,
    pub forcePushTime: c_int,
    pub forcePuller: c_int,
    pub physics_time: c_int,
    pub playerModel: c_int,
    pub ghoul2: [*mut c_void; 2],
    pub e_DieFunc: *const c_void,
    pub e_UseFunc: *const c_void,
    pub e_TouchFunc: *const c_void,
    pub e_ThinkFunc: *const c_void,
    pub e_PainFunc: *const c_void,
    pub startFrame: c_int,
    pub endFrame: c_int,
    pub loopSound: c_int,
    pub s: entity_state_t,
    pub client: *mut gclient_t,
    pub _pad: [u8; 2048], // Placeholder for additional fields
}

#[repr(C)]
pub struct entity_state_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub modelindex: c_int,
    pub modelindex2: c_int,
    pub modelindex3: c_int,
    pub frame: c_int,
    pub otherEntityNum: c_int,
    pub groundEntityNum: c_int,
    pub solid: c_int,
    pub pos: trajectory_t,
    pub apos: trajectory_t,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub modelScale: [f32; 3],
    pub constantLight: c_int,
    pub loopSound: c_int,
    pub weapon: c_int,
    pub radius: c_int,
    pub eventParm: c_int,
    pub _pad: [u8; 512], // Placeholder
}

#[repr(C)]
pub struct trajectory_t {
    pub trType: c_int,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: [f32; 3],
    pub trDelta: [f32; 3],
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: bool,
    pub startsolid: bool,
    pub fraction: f32,
    pub endpos: [f32; 3],
    pub plane: c_plane_t,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub entityNum: c_int,
}

#[repr(C)]
pub struct c_plane_t {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: u8,
    pub signbits: u8,
    pub pad: [u8; 2],
}

#[repr(C)]
pub struct gclient_t {
    _unused: c_void,
}

// Extern declarations for engine API
extern "C" {
    pub fn cgi_R_GetBModelVerts(bmodelIndex: c_int, verts: *mut [[f32; 3]; 4], normal: *mut [f32; 3]);
    pub fn CG_DoGlass(verts: *const [[f32; 3]; 4], normal: *const [f32; 3], dmgPt: *const [f32; 3], dmgDir: *const [f32; 3], dmgRadius: f32);
    pub static cgs: cgs_t;
    pub static vec3_origin: [f32; 3];
    pub static TeamTable: c_void;
}

#[repr(C)]
pub struct cgs_t {
    pub inlineDrawModel: [c_int; 256], // Placeholder
    _pad: [u8; 1024],
}

// Constants
const MAX_GENTITIES: usize = 4096;
const MAX_QPATH: usize = 256;
const FRAMETIME: c_int = 50;
const MIN_WORLD_COORD: f32 = -131072.0;

// Material constants
const MAT_METAL: i32 = 0;
const MAT_GLASS: i32 = 1;
const MAT_ELECTRICAL: i32 = 2;
const MAT_ELEC_METAL: i32 = 3;
const MAT_DRK_STONE: i32 = 4;
const MAT_LT_STONE: i32 = 5;
const MAT_GLASS_METAL: i32 = 6;
const MAT_METAL2: i32 = 7;
const MAT_NONE: i32 = 8;
const MAT_GREY_STONE: i32 = 9;
const MAT_METAL3: i32 = 10;
const MAT_CRATE1: i32 = 11;
const MAT_GRATE1: i32 = 12;
const MAT_ROPE: i32 = 13;
const MAT_CRATE2: i32 = 14;
const MAT_WHITE_METAL: i32 = 15;

// Spawnflags for func_breakable
const INVINCIBLE: c_int = 1;
const IMPACT: c_int = 2;
const CRUSHER: c_int = 4;
const THIN: c_int = 8;
const SABERONLY: c_int = 16;
const HEAVY_WEAP: c_int = 32;
const USE_NOT_BREAK: c_int = 64;
const PLAYER_USE: c_int = 128;
const NO_EXPLOSION: c_int = 2048;

// Damage flags
const MOD_CRUSH: c_int = 0;
const MOD_UNKNOWN: c_int = 0;
const MOD_ENERGY: c_int = 0;
const MOD_EXPLOSIVE_SPLASH: c_int = 0;

// Entity flags
const FL_DMG_BY_SABER_ONLY: c_int = 1;
const FL_DMG_BY_HEAVY_WEAP_ONLY: c_int = 2;
const FL_RED_CROSSHAIR: c_int = 4;

// SVFlags
const SVF_BBRUSH: c_int = 1;
const SVF_PLAYER_USABLE: c_int = 2;
const SVF_NOCLIENT: c_int = 4;
const SVF_GLASS_BRUSH: c_int = 8;
const SVF_BROADCAST: c_int = 16;
const SVF_BROKEN: c_int = 32;
const SVF_ANIMATING: c_int = 64;

// EFlags
const EF_NODRAW: c_int = 1;
const EF_MISSILE_STICK: c_int = 2;
const EF_ANIM_ALLFAST: c_int = 4;
const EF_LESS_ATTEN: c_int = 8;
const EF_FORCE_VISIBLE: c_int = 16;
const EF_BOUNCE_HALF: c_int = 32;

// Entity types
const ET_MOVER: c_int = 1;
const ET_GENERAL: c_int = 2;
const ET_MISSILE: c_int = 3;

// Trajectory types
const TR_STATIONARY: c_int = 0;
const TR_INTERPOLATE: c_int = 1;
const TR_LINEAR: c_int = 2;
const TR_GRAVITY: c_int = 3;

// Weapon types
const WP_BLASTER: c_int = 1;
const WP_TIE_FIGHTER: c_int = 2;

// Behavior sets
const BSET_USE: c_int = 1;
const BSET_PAIN: c_int = 2;
const BSET_DEATH: c_int = 3;

// Sound channels
const CHAN_VOICE: c_int = 1;

// Contents
const CONTENTS_SOLID: c_int = 1;
const CONTENTS_OPAQUE: c_int = 2;
const CONTENTS_BODY: c_int = 4;
const CONTENTS_MONSTERCLIP: c_int = 8;
const CONTENTS_BOTCLIP: c_int = 16;
const CONTENTS_SHOTCLIP: c_int = 32;

// Masks
const MASK_SHOT: c_int = 1;
const MASK_SOLID: c_int = 2;
const MASK_NPCSOLID: c_int = 4;

// AEL types
const AEL_DISCOVERED: c_int = 1;
const AEL_SUSPICIOUS: c_int = 2;

// Event types
const EV_GENERAL_SOUND: c_int = 1;

// Team types
const TEAM_FREE: c_int = 0;

// misc_model_breakable types
const MDL_OTHER: c_int = 0;
const MDL_ARMOR_HEALTH: c_int = 1;
const MDL_AMMO: c_int = 2;

const MIN_PLAYER_DIST: c_int = 1600;

// Function pointer types
pub type thinkF_t = extern "C" fn(*mut gentity_t);
pub type useF_t = extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t);
pub type touchF_t = extern "C" fn(*mut gentity_t, *mut gentity_t, *mut trace_t);
pub type painF_t = extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, *const [f32; 3], c_int, c_int, c_int);
pub type dieF_t = extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, c_int, c_int, c_int, c_int);

// Placeholder function pointers - to be defined elsewhere
extern "C" {
    pub static dieF_funcBBrushDie: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, c_int, c_int, c_int, c_int);
    pub static dieF_misc_model_breakable_die: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, c_int, c_int, c_int, c_int);
    pub static dieF_funcGlassDie: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, c_int, c_int, c_int, c_int);
    pub static dieF_NULL: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, c_int, c_int, c_int, c_int);
    pub static dieF_G_FreeEntity: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, c_int, c_int, c_int, c_int);

    pub static useF_funcBBrushUse: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t);
    pub static useF_funcGlassUse: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t);
    pub static useF_misc_model_use: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t);
    pub static useF_TieFighterUse: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t);
    pub static useF_health_use: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t);
    pub static useF_ammo_use: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t);
    pub static useF_NULL: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t);

    pub static touchF_funcBBrushTouch: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut trace_t);
    pub static touchF_TouchTieBomb: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut trace_t);
    pub static touchF_NULL: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut trace_t);

    pub static painF_funcBBrushPain: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, *const [f32; 3], c_int, c_int, c_int);
    pub static painF_misc_model_breakable_pain: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, *const [f32; 3], c_int, c_int, c_int);
    pub static painF_NULL: extern "C" fn(*mut gentity_t, *mut gentity_t, *mut gentity_t, *const [f32; 3], c_int, c_int, c_int);

    pub static thinkF_funcBBrushDieGo: extern "C" fn(*mut gentity_t);
    pub static thinkF_G_FreeEntity: extern "C" fn(*mut gentity_t);
    pub static thinkF_TieFighterThink: extern "C" fn(*mut gentity_t);
    pub static thinkF_TieBomberThink: extern "C" fn(*mut gentity_t);
    pub static thinkF_G_RunObject: extern "C" fn(*mut gentity_t);
    pub static thinkF_NULL: extern "C" fn(*mut gentity_t);
}

//---------------------------------------------------
unsafe fn CacheChunkEffects(material: i32) {
    match material {
        MAT_GLASS => {
            G_EffectIndex(b"chunks/glassbreak\0".as_ptr() as *const c_char);
        }
        MAT_GLASS_METAL => {
            G_EffectIndex(b"chunks/glassbreak\0".as_ptr() as *const c_char);
            G_EffectIndex(b"chunks/metalexplode\0".as_ptr() as *const c_char);
        }
        MAT_ELECTRICAL | MAT_ELEC_METAL => {
            G_EffectIndex(b"chunks/sparkexplode\0".as_ptr() as *const c_char);
        }
        MAT_METAL | MAT_METAL2 | MAT_METAL3 | MAT_CRATE1 | MAT_CRATE2 => {
            G_EffectIndex(b"chunks/metalexplode\0".as_ptr() as *const c_char);
        }
        MAT_GRATE1 => {
            G_EffectIndex(b"chunks/grateexplode\0".as_ptr() as *const c_char);
        }
        MAT_DRK_STONE | MAT_LT_STONE | MAT_GREY_STONE | MAT_WHITE_METAL => {
            // what is this crap really supposed to be??
            G_EffectIndex(b"chunks/rockbreaklg\0".as_ptr() as *const c_char);
            G_EffectIndex(b"chunks/rockbreakmed\0".as_ptr() as *const c_char);
        }
        MAT_ROPE => {
            G_EffectIndex(b"chunks/ropebreak\0".as_ptr() as *const c_char);
            // G_SoundIndex(); // FIXME: give it a sound
        }
        _ => {}
    }
}

//--------------------------------------
unsafe fn funcBBrushDieGo(this: *mut gentity_t) {
    let mut org = [0.0; 3];
    let mut dir = [0.0; 3];
    let mut up = [0.0; 3];
    let attacker = (*this).enemy;
    let mut scale: f32;
    let mut numChunks: c_int;
    let mut size: c_int = 0;
    let chunkType = (*this).material;

    // if a missile is stuck to us, blow it up so we don't look dumb
    // FIXME: flag me so I should know to do this check!
    for i in 0..MAX_GENTITIES {
        if g_entities[i].s.groundEntityNum == (*this).s.number && (g_entities[i].s.eFlags & EF_MISSILE_STICK) != 0 {
            G_Damage(&mut g_entities[i], this, this, std::ptr::null(), std::ptr::null(), 99999, 0, MOD_CRUSH); //?? MOD?
        }
    }

    //NOTE: MUST do this BEFORE clearing contents, or you may not open the area portal!!!
    gi.AdjustAreaPortalState(this, true);

    //So chunks don't get stuck inside me
    (*this).s.solid = 0;
    (*this).contents = 0;
    (*this).clipmask = 0;
    gi.linkentity(this);

    VectorSet(&mut up, 0.0, 0.0, 1.0);

    if !(*this).target.is_null() && !attacker.is_null() {
        G_UseTargets(this, attacker);
    }

    VectorSubtract(&(*this).absmax, &(*this).absmin, &mut org); // size

    numChunks = (random() * 6.0 + 18.0) as c_int;

    // This formula really has no logical basis other than the fact that it seemed to be the closest to yielding the results that I wanted.
    // Volume is length * width * height...then break that volume down based on how many chunks we have
    scale = (org[0] * org[1] * org[2]).sqrt().sqrt() * 1.75;

    if scale > 48.0 {
        size = 2;
    } else if scale > 24.0 {
        size = 1;
    }

    scale = scale / numChunks as f32;

    if (*this).radius > 0.0 {
        // designer wants to scale number of chunks, helpful because the above scale code is far from perfect
        //	I do this after the scale calculation because it seems that the chunk size generally seems to be very close, it's just the number of chunks is a bit weak
        numChunks = (numChunks as f32 * (*this).radius) as c_int;
    }

    VectorMA(&(*this).absmin, 0.5, &org, &mut org);
    VectorAdd(&(*this).absmin, &(*this).absmax, &mut org);
    VectorScale(&org, 0.5, &mut org);

    if !attacker.is_null() && !(*attacker).client.is_null() {
        VectorSubtract(&org, &(*attacker).currentOrigin, &mut dir);
        VectorNormalize(&mut dir);
    } else {
        VectorCopy(&up, &mut dir);
    }

    if ((*this).spawnflags & NO_EXPLOSION) == 0 {
        // we are allowed to explode
        CG_MiscModelExplosion(&mut (*this).absmin, &mut (*this).absmax, size, chunkType);
    }

    if (*this).splashDamage > 0 && (*this).splashRadius > 0 {
        //explode
        AddSightEvent(attacker, &org, 256, AEL_DISCOVERED, 100);
        AddSoundEvent(attacker, &org, 128, AEL_DISCOVERED, false, true);//FIXME: am I on ground or not?
        G_RadiusDamage(&org, this, (*this).splashDamage, (*this).splashRadius, this, MOD_UNKNOWN);

        let te = G_TempEntity(&org, EV_GENERAL_SOUND);
        (*te).s.eventParm = G_SoundIndex(b"sound/weapons/explosions/cargoexplode.wav\0".as_ptr() as *const c_char);
    } else {
        //just break
        AddSightEvent(attacker, &org, 128, AEL_DISCOVERED, 0);
        AddSoundEvent(attacker, &org, 64, AEL_SUSPICIOUS, false, true);//FIXME: am I on ground or not?
    }

    //FIXME: base numChunks off size?
    CG_Chunks((*this).s.number, &mut org, &dir, &(*this).absmin, &(*this).absmax, 300.0, numChunks, chunkType, 0, scale, (*this).noise_index);

    (*this).e_ThinkFunc = &thinkF_G_FreeEntity as *const ();
    (*this).nextthink = level.time + 50;
    //G_FreeEntity( self );
}

unsafe fn funcBBrushDie(this: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int) {
    (*this).takedamage = false;//stop chain reaction runaway loops

    G_SetEnemy(this, attacker);

    if (*this).delay != 0.0 {
        (*this).e_ThinkFunc = &thinkF_funcBBrushDieGo as *const ();
        (*this).nextthink = level.time + ((*this).delay * 1000.0).floor() as c_int;
        return;
    }

    funcBBrushDieGo(this);
}

unsafe fn funcBBrushUse(this: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(this, BSET_USE);
    if ((*this).spawnflags & USE_NOT_BREAK) != 0 {
        //Using it doesn't break it, makes it use it's targets
        if !(*this).target.is_null() && *(*this).target as c_int != 0 {
            G_UseTargets(this, activator);
        }
    } else {
        funcBBrushDie(this, other, activator, (*this).health, MOD_UNKNOWN, 0, 0);
    }
}

unsafe fn funcBBrushPain(this: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const [f32; 3], damage: c_int, mod_: c_int, hitLoc: c_int) {
    if (*this).painDebounceTime > level.time {
        return;
    }

    if !(*this).paintarget.is_null() {
        G_UseTargets2(this, (*this).activator, (*this).paintarget);
    }

    G_ActivateBehavior(this, BSET_PAIN);

    if (*this).material == MAT_DRK_STONE
        || (*this).material == MAT_LT_STONE
        || (*this).material == MAT_GREY_STONE {
        let mut org = [0.0; 3];
        let mut dir = [0.0; 3];
        let mut scale: f32;
        VectorSubtract(&(*this).absmax, &(*this).absmin, &mut org); // size
        // This formula really has no logical basis other than the fact that it seemed to be the closest to yielding the results that I wanted.
        // Volume is length * width * height...then break that volume down based on how many chunks we have
        scale = VectorLength(&org) / 100.0;
        VectorMA(&(*this).absmin, 0.5, &org, &mut org);
        VectorAdd(&(*this).absmin, &(*this).absmax, &mut org);
        VectorScale(&org, 0.5, &mut org);
        if !attacker.is_null() && !(*attacker).client.is_null() {
            VectorSubtract(&(*attacker).currentOrigin, &org, &mut dir);
            VectorNormalize(&mut dir);
        } else {
            VectorSet(&mut dir, 0.0, 0.0, 1.0);
        }
        CG_Chunks((*this).s.number, &mut org, &dir, &(*this).absmin, &(*this).absmax, 300.0, Q_irand(1, 3), (*this).material, 0, scale, 0);
    }

    if (*this).wait == -1.0 {
        (*this).e_PainFunc = &painF_NULL as *const ();
        return;
    }

    (*this).painDebounceTime = level.time + ((*this).wait as c_int);
}

unsafe fn InitBBrush(ent: *mut gentity_t) {
    let mut light: f32 = 0.0;
    let mut color = [0.0; 3];
    let mut lightSet: bool;
    let mut colorSet: bool;

    VectorCopy(&(*ent).s.origin, &mut (*ent).pos1);

    gi.SetBrushModel(ent, (*ent).model);

    (*ent).e_DieFunc = &dieF_funcBBrushDie as *const ();

    (*ent).svFlags |= SVF_BBRUSH;

    // if the "model2" key is set, use a seperate model
    // for drawing, but clip against the brushes
    if !(*ent).model2.is_null() {
        (*ent).s.modelindex2 = G_ModelIndex((*ent).model2);
    }

    // if the "color" or "light" keys are set, setup constantLight
    lightSet = G_SpawnFloat(b"light\0".as_ptr() as *const c_char, b"100\0".as_ptr() as *const c_char, &mut light);
    colorSet = G_SpawnVector(b"color\0".as_ptr() as *const c_char, b"1 1 1\0".as_ptr() as *const c_char, &mut color);
    if lightSet || colorSet {
        let mut r: c_int;
        let mut g: c_int;
        let mut b: c_int;
        let mut i: c_int;

        r = (color[0] * 255.0) as c_int;
        if r > 255 {
            r = 255;
        }
        g = (color[1] * 255.0) as c_int;
        if g > 255 {
            g = 255;
        }
        b = (color[2] * 255.0) as c_int;
        if b > 255 {
            b = 255;
        }
        i = (light / 4.0) as c_int;
        if i > 255 {
            i = 255;
        }
        (*ent).s.constantLight = r | (g << 8) | (b << 16) | (i << 24);
    }

    if ((*ent).spawnflags & PLAYER_USE) != 0 {
        //Can be used by the player's BUTTON_USE
        (*ent).svFlags |= SVF_PLAYER_USABLE;
    }

    (*ent).s.eType = ET_MOVER;
    gi.linkentity(ent);

    (*ent).s.pos.trType = TR_STATIONARY;
    VectorCopy(&(*ent).pos1, &mut (*ent).s.pos.trBase);
}

unsafe fn funcBBrushTouch(ent: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    // Empty function
}

/*QUAKED func_breakable (0 .8 .5) ? INVINCIBLE IMPACT CRUSHER THIN SABERONLY HEAVY_WEAP USE_NOT_BREAK PLAYER_USE NO_EXPLOSION
INVINCIBLE - can only be broken by being used
IMPACT - does damage on impact
CRUSHER - won't reverse movement when hit an obstacle
THIN - can be broken by impact damage, like glass
SABERONLY - only takes damage from sabers
HEAVY_WEAP - only takes damage by a heavy weapon, like an emplaced gun or AT-ST gun.
USE_NOT_BREAK - Using it doesn't make it break, still can be destroyed by damage
PLAYER_USE - Player can use it with the use button
NO_EXPLOSION - Does not play an explosion effect, though will still create chunks if specified

When destroyed, fires it's trigger and chunks and plays sound "noise" or sound for type if no noise specified

"targetname" entities with matching target will fire it
"paintarget" target to fire when hit (but not destroyed)
"wait"		how long minimum to wait between firing paintarget each time hit
"delay"		When killed or used, how long (in seconds) to wait before blowing up (none by default)
"model2"	.md3 model to also draw
"modelAngles" md3 model's angles <pitch yaw roll> (in addition to any rotation on the part of the brush entity itself)
"target"	all entities with a matching targetname will be used when this is destoryed
"health"	default is 10
"radius"  Chunk code tries to pick a good volume of chunks, but you can alter this to scale the number of spawned chunks. (default 1)  (.5) is half as many chunks, (2) is twice as many chunks
"NPC_targetname" - Only the NPC with this name can damage this
"forcevisible" - When you turn on force sight (any level), you can see these draw through the entire level...
"redCrosshair" - crosshair turns red when you look at this

Damage: default is none
"splashDamage" - damage to do
"splashRadius" - radius for above damage

"team" - This cannot take damage from members of this team:
	"player"
	"neutral"
	"enemy"

Don't know if these work:
"color"		constantLight color
"light"		constantLight radius

"material" - default is "0 - MAT_METAL" - choose from this list:
0 = MAT_METAL		(basic blue-grey scorched-DEFAULT)
1 = MAT_GLASS
2 = MAT_ELECTRICAL	(sparks only)
3 = MAT_ELEC_METAL	(METAL2 chunks and sparks)
4 =	MAT_DRK_STONE	(brown stone chunks)
5 =	MAT_LT_STONE	(tan stone chunks)
6 =	MAT_GLASS_METAL (glass and METAL2 chunks)
7 = MAT_METAL2		(electronic type of metal)
8 = MAT_NONE		(no chunks)
9 = MAT_GREY_STONE	(grey colored stone)
10 = MAT_METAL3		(METAL and METAL2 chunk combo)
11 = MAT_CRATE1		(yellow multi-colored crate chunks)
12 = MAT_GRATE1		(grate chunks--looks horrible right now)
13 = MAT_ROPE		(for yavin_trial, no chunks, just wispy bits )
14 = MAT_CRATE2		(red multi-colored crate chunks)
15 = MAT_WHITE_METAL (white angular chunks for Stu, NS_hideout )

*/
pub unsafe fn SP_func_breakable(this: *mut gentity_t) {
    if ((*this).spawnflags & INVINCIBLE) == 0 {
        if (*this).health == 0 {
            (*this).health = 10;
        }
    }

    if ((*this).spawnflags & SABERONLY) != 0 {
        // saber only
        (*this).flags |= FL_DMG_BY_SABER_ONLY;
    } else if ((*this).spawnflags & HEAVY_WEAP) != 0 {
        // heavy weap
        (*this).flags |= FL_DMG_BY_HEAVY_WEAP_ONLY;
    }

    if (*this).health != 0 {
        (*this).takedamage = true;
    }

    G_SoundIndex(b"sound/weapons/explosions/cargoexplode.wav\0".as_ptr() as *const c_char);//precaching
    G_SpawnFloat(b"radius\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, &mut (*this).radius); // used to scale chunk code if desired by a designer
    G_SpawnInt(b"material\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut (*this).material as *mut c_int);
    CacheChunkEffects((*this).material);

    (*this).e_UseFunc = &useF_funcBBrushUse as *const ();

    //if ( self->paintarget )
    {
        (*this).e_PainFunc = &painF_funcBBrushPain as *const ();
    }

    (*this).e_TouchFunc = &touchF_funcBBrushTouch as *const ();

    if !(*this).team.is_null() && *(*this).team as c_int != 0 {
        (*this).noDamageTeam = GetIDForString(&TeamTable as *const c_void, (*this).team) as c_int;
        if (*this).noDamageTeam == TEAM_FREE {
            G_Error(b"team name %s not recognized\n\0".as_ptr() as *const c_char, (*this).team);
        }
    }
    (*this).team = std::ptr::null();
    if (*this).model.is_null() {
        G_Error(b"func_breakable with NULL model\n\0".as_ptr() as *const c_char);
    }
    InitBBrush(this);

    let mut buffer: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut s: *const c_char = std::ptr::null();
    if G_SpawnString(b"noise\0".as_ptr() as *const c_char, b"*NOSOUND*\0".as_ptr() as *const c_char, &mut s) {
        Q_strncpyz(&mut buffer[0], s, buffer.len());
        COM_DefaultExtension(&mut buffer[0], buffer.len(), b".wav\0".as_ptr() as *const c_char);
        (*this).noise_index = G_SoundIndex(&buffer[0]);
    }

    let mut forceVisible: c_int = 0;
    G_SpawnInt(b"forcevisible\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut forceVisible);
    if forceVisible != 0 {
        //can see these through walls with force sight, so must be broadcast
        if VectorCompare(&(*this).s.origin, &vec3_origin) {
            //no origin brush
            (*this).svFlags |= SVF_BROADCAST;
        }
        (*this).s.eFlags |= EF_FORCE_VISIBLE;
    }

    let mut redCrosshair: c_int = 0;
    G_SpawnInt(b"redCrosshair\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut redCrosshair);
    if redCrosshair != 0 {
        //can see these through walls with force sight, so must be broadcast
        (*this).flags |= FL_RED_CROSSHAIR;
    }
}


unsafe fn misc_model_breakable_pain(this: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const [f32; 3], damage: c_int, mod_: c_int, hitLoc: c_int) {
    if (*this).health > 0 {
        // still alive, react to the pain
        if !(*this).paintarget.is_null() {
            G_UseTargets2(this, (*this).activator, (*this).paintarget);
        }

        // Don't do script if dead
        G_ActivateBehavior(this, BSET_PAIN);
    }
}

unsafe fn misc_model_breakable_die(this: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, meansOfDeath: c_int, dFlags: c_int, hitLoc: c_int) {
    let mut numChunks: c_int;
    let mut size: f32 = 0.0;
    let mut scale: f32;
    let mut dir = [0.0; 3];
    let mut up = [0.0; 3];
    let mut dis = [0.0; 3];

    if (*this).e_DieFunc == &dieF_NULL as *const c_void {
        //i was probably already killed since my die func was removed
        #[cfg(not(target_env = "final"))]
        {
            G_Printf(b"\x1b[33mRecursive misc_model_breakable_die.  Use targets probably pointing back at self.\n\0".as_ptr() as *const c_char);
        }
        return;	//this happens when you have a cyclic target chain!
    }
    //NOTE: Stop any scripts that are currently running (FLUSH)... ?
    //Turn off animation
    (*this).s.frame = 0;
    (*this).startFrame = 0;
    (*this).endFrame = 0;
    (*this).svFlags &= !SVF_ANIMATING;

    (*this).health = 0;
    //Throw some chunks
    AngleVectors(&(*this).s.apos.trBase, &mut dir, std::ptr::null_mut(), std::ptr::null_mut());
    VectorNormalize(&mut dir);

    numChunks = (random() * 6.0 + 20.0) as c_int;

    VectorSubtract(&(*this).absmax, &(*this).absmin, &mut dis);

    // This formula really has no logical basis other than the fact that it seemed to be the closest to yielding the results that I wanted.
    // Volume is length * width * height...then break that volume down based on how many chunks we have
    scale = (dis[0] * dis[1] * dis[2]).sqrt().sqrt() * 1.75;

    if scale > 48.0 {
        size = 2.0;
    } else if scale > 24.0 {
        size = 1.0;
    }

    scale = scale / numChunks as f32;

    if (*this).radius > 0.0 {
        // designer wants to scale number of chunks, helpful because the above scale code is far from perfect
        //	I do this after the scale calculation because it seems that the chunk size generally seems to be very close, it's just the number of chunks is a bit weak
        numChunks = (numChunks as f32 * (*this).radius) as c_int;
    }

    VectorAdd(&(*this).absmax, &(*this).absmin, &mut dis);
    VectorScale(&dis, 0.5, &mut dis);

    CG_Chunks((*this).s.number, &mut dis, &dir, &(*this).absmin, &(*this).absmax, 300.0, numChunks, (*this).material, (*this).s.modelindex3, scale, 0);

    (*this).e_PainFunc = &painF_NULL as *const ();
    (*this).e_DieFunc = &dieF_NULL as *const ();
    //	self->e_UseFunc  = useF_NULL;

    (*this).takedamage = false;

    if ((*this).spawnflags & CRUSHER) == 0 {
        //We don't want to stay solid
        (*this).s.solid = 0;
        (*this).contents = 0;
        (*this).clipmask = 0;
        if this as *const _ != std::ptr::null() {
            NAV_WayEdgesNowClear(this);
        }
        gi.linkentity(this);
    }

    VectorSet(&mut up, 0.0, 0.0, 1.0);

    if !(*this).target.is_null() {
        G_UseTargets(this, attacker);
    }

    if !inflictor.is_null() && !(*inflictor).client.is_null() {
        VectorSubtract(&(*this).currentOrigin, &(*inflictor).currentOrigin, &mut dir);
        VectorNormalize(&mut dir);
    } else {
        VectorCopy(&up, &mut dir);
    }

    if ((*this).spawnflags & NO_EXPLOSION) == 0 {
        // Ok, we are allowed to explode, so do it now!
        if (*this).splashDamage > 0 && (*this).splashRadius > 0 {
            //explode
            let mut org = [0.0; 3];
            AddSightEvent(attacker, &(*this).currentOrigin, 256, AEL_DISCOVERED, 100);
            AddSoundEvent(attacker, &(*this).currentOrigin, 128, AEL_DISCOVERED, false, true);//FIXME: am I on ground or not?
            //FIXME: specify type of explosion?  (barrel, electrical, etc.)  Also, maybe just use the explosion effect below since it's
            //				a bit better?
            // up the origin a little for the damage check, because several models have their origin on the ground, so they don't alwasy do damage, not the optimal solution...
            VectorCopy(&(*this).currentOrigin, &mut org);
            if (*this).mins[2] > -4.0 {
                //origin is going to be below it or very very low in the model
                //center the origin
                org[2] = (*this).currentOrigin[2] + (*this).mins[2] + ((*this).maxs[2] - (*this).mins[2]) / 2.0;
            }
            G_RadiusDamage(&org, this, (*this).splashDamage, (*this).splashRadius, this, MOD_UNKNOWN);

            if !(*this).model.is_null() && (Q_stricmp(b"models/map_objects/ships/tie_fighter.md3\0".as_ptr() as *const c_char, (*this).model) == 0 ||
                                  Q_stricmp(b"models/map_objects/ships/tie_bomber.md3\0".as_ptr() as *const c_char, (*this).model) == 0) {
                //TEMP HACK for Tie Fighters- they're HUGE
                G_PlayEffect(b"explosions/fighter_explosion2\0".as_ptr() as *const c_char, &(*this).currentOrigin);
                G_Sound(this, G_SoundIndex(b"sound/weapons/tie_fighter/TIEexplode.wav\0".as_ptr() as *const c_char));
                (*this).s.loopSound = 0;
            } else {
                CG_MiscModelExplosion(&mut (*this).absmin, &mut (*this).absmax, size as c_int, (*this).material);
                G_Sound(this, G_SoundIndex(b"sound/weapons/explosions/cargoexplode.wav\0".as_ptr() as *const c_char));
                (*this).s.loopSound = 0;
            }
        } else {
            //just break
            AddSightEvent(attacker, &(*this).currentOrigin, 128, AEL_DISCOVERED, 0);
            AddSoundEvent(attacker, &(*this).currentOrigin, 64, AEL_SUSPICIOUS, false, true);//FIXME: am I on ground or not?
            // This is the default explosion
            CG_MiscModelExplosion(&mut (*this).absmin, &mut (*this).absmax, size as c_int, (*this).material);
            G_Sound(this, G_SoundIndex(b"sound/weapons/explosions/cargoexplode.wav\0".as_ptr() as *const c_char));
        }
    }

    (*this).e_ThinkFunc = &thinkF_NULL as *const ();
    (*this).nextthink = -1;

    if (*this).s.modelindex2 != -1 && ((*this).spawnflags & 8) == 0 {
        //FIXME: modelindex doesn't get set to -1 if the damage model doesn't exist
        (*this).svFlags |= SVF_BROKEN;
        (*this).s.modelindex = (*this).s.modelindex2;
        G_ActivateBehavior(this, BSET_DEATH);
    } else {
        G_FreeEntity(this);
    }
}

unsafe fn misc_model_throw_at_target4(this: *mut gentity_t, activator: *mut gentity_t) {
    let mut pushDir = [0.0; 3];
    let mut kvel = [0.0; 3];
    let mut knockback: f32 = 200.0;
    let mut mass: f32 = (*this).mass;
    let target = G_Find(std::ptr::null_mut(), 0, (*this).target4); // FOFS(targetname)
    if target.is_null() {
        //nothing to throw ourselves at...
        return;
    }
    VectorSubtract(&(*target).currentOrigin, &(*this).currentOrigin, &mut pushDir);
    knockback -= VectorNormalize(&mut pushDir);
    if knockback < 100.0 {
        knockback = 100.0;
    }
    VectorCopy(&(*this).currentOrigin, &mut (*this).s.pos.trBase);
    (*this).s.pos.trTime = level.time;								// move a bit on the very first frame
    if (*this).s.pos.trType != TR_INTERPOLATE {
        //don't do this to rolling missiles
        (*this).s.pos.trType = TR_GRAVITY;
    }

    if mass < 50.0 {
        //???
        mass = 50.0;
    }

    if g_gravity.value > 0.0 {
        VectorScale(&pushDir, g_knockback.value * knockback / mass * 0.8, &mut kvel);
        kvel[2] = pushDir[2] * g_knockback.value * knockback / mass * 1.5;
    } else {
        VectorScale(&pushDir, g_knockback.value * knockback / mass, &mut kvel);
    }

    VectorAdd(&(*this).s.pos.trDelta, &kvel, &mut (*this).s.pos.trDelta);
    if g_gravity.value > 0.0 {
        if (*this).s.pos.trDelta[2] < knockback {
            (*this).s.pos.trDelta[2] = knockback;
        }
    }
    //no trDuration?
    if (*this).e_ThinkFunc != &thinkF_G_RunObject as *const c_void {
        //objects spin themselves?
        //spin it
        //FIXME: messing with roll ruins the rotational center???
        (*this).s.apos.trTime = level.time;
        (*this).s.apos.trType = TR_LINEAR;
        VectorClear(&mut (*this).s.apos.trDelta);
        (*this).s.apos.trDelta[1] = Q_irand(-800, 800) as f32;
    }

    (*this).forcePushTime = level.time + 600; // let the push effect last for 600 ms
    if !activator.is_null() {
        (*this).forcePuller = (*activator).s.number;//remember this regardless
    } else {
        (*this).forcePuller = -1;
    }
}

unsafe fn misc_model_use(this: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if !(*this).target4.is_null() {
        //throw me at my target!
        misc_model_throw_at_target4(this, activator);
        return;
    }

    if (*this).health <= 0 && (*this).max_health > 0 {
        //used while broken fired target3
        G_UseTargets2(this, activator, (*this).target3);
        return;
    }

    // Become solid again.
    if (*this).count == 0 {
        (*this).count = 1;
        (*this).activator = activator;
        (*this).svFlags &= !SVF_NOCLIENT;
        (*this).s.eFlags &= !EF_NODRAW;
    }

    G_ActivateBehavior(this, BSET_USE);
    //Don't explode if they've requested it to not
    if ((*this).spawnflags & 64) != 0 {
        //Usemodels toggling
        if ((*this).spawnflags & 32) != 0 {
            if (*this).s.modelindex == (*this).sound1to2 {
                (*this).s.modelindex = (*this).sound2to1;
            } else {
                (*this).s.modelindex = (*this).sound1to2;
            }
        }

        return;
    }

    (*this).e_DieFunc = &dieF_misc_model_breakable_die as *const ();
    misc_model_breakable_die(this, other, activator, (*this).health, MOD_UNKNOWN, 0, 0);
}

unsafe fn misc_model_breakable_init(ent: *mut gentity_t) {
    let mut type_: c_int;

    type_ = MDL_OTHER;

    if (*ent).model.is_null() {
        G_Error(b"no model set on %s at (%.1f %.1f %.1f)\n\0".as_ptr() as *const c_char, (*ent).classname, (*ent).s.origin[0], (*ent).s.origin[1], (*ent).s.origin[2]);
    }
    //Main model
    (*ent).s.modelindex = (*ent).sound2to1 = G_ModelIndex((*ent).model);

    if ((*ent).spawnflags & 1) != 0 {
        //Blocks movement
        (*ent).contents = CONTENTS_SOLID | CONTENTS_OPAQUE | CONTENTS_BODY | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP;//Was CONTENTS_SOLID, but only architecture should be this
    } else if (*ent).health != 0 {
        //Can only be shot
        (*ent).contents = CONTENTS_SHOTCLIP;
    }

    if type_ == MDL_OTHER {
        (*ent).e_UseFunc = &useF_misc_model_use as *const ();
    } else if type_ == MDL_ARMOR_HEALTH {
        //		G_SoundIndex("sound/player/suithealth.wav");
        (*ent).e_UseFunc = &useF_health_use as *const ();
        if (*ent).count == 0 {
            (*ent).count = 100;
        }
        (*ent).health = 60;
    } else if type_ == MDL_AMMO {
        //		G_SoundIndex("sound/player/suitenergy.wav");
        (*ent).e_UseFunc = &useF_ammo_use as *const ();
        if (*ent).count == 0 {
            (*ent).count = 100;
        }
        (*ent).health = 60;
    }

    if (*ent).health != 0 {
        G_SoundIndex(b"sound/weapons/explosions/cargoexplode.wav\0".as_ptr() as *const c_char);
        (*ent).max_health = (*ent).health;
        (*ent).takedamage = true;
        (*ent).e_PainFunc = &painF_misc_model_breakable_pain as *const ();
        (*ent).e_DieFunc = &dieF_misc_model_breakable_die as *const ();
    }
}

unsafe fn TieFighterThink(this: *mut gentity_t) {
    let player = &mut g_entities[0];

    if (*this).health <= 0 {
        return;
    }

    (*this).nextthink = level.time + FRAMETIME;
    if !player.is_null() {
        let mut playerDir = [0.0; 3];
        let mut fighterDir = [0.0; 3];
        let mut fwd = [0.0; 3];
        let mut rt = [0.0; 3];
        let mut playerDist: f32;
        let mut fighterSpeed: f32;

        //use player eyepoint
        VectorSubtract(&(*player).currentOrigin, &(*this).currentOrigin, &mut playerDir);
        playerDist = VectorNormalize(&mut playerDir);
        VectorSubtract(&(*this).currentOrigin, &(*this).lastOrigin, &mut fighterDir);
        VectorCopy(&(*this).currentOrigin, &mut (*this).lastOrigin);
        fighterSpeed = VectorNormalize(&mut fighterDir) * 1000.0;
        AngleVectors(&(*this).currentAngles, &mut fwd, &mut rt, std::ptr::null_mut());

        if fighterSpeed != 0.0 {
            let mut side: f32;

            // Magic number fun!  Speed is used for banking, so modulate the speed by a sine wave
            fighterSpeed *= (100.0 * 0.003).sin();

            // Clamp to prevent harsh rolling
            if fighterSpeed > 10.0 {
                fighterSpeed = 10.0;
            }

            side = fighterSpeed * DotProduct(&fighterDir, &rt);
            (*this).s.apos.trBase[2] -= side;
        }

        //FIXME: bob up/down, strafe left/right some
        let dot = DotProduct(&playerDir, &fighterDir);
        if dot > 0.0 {
            //heading toward the player
            if playerDist < 1024.0 {
                if DotProduct(&playerDir, &fwd) > 0.7 {
                    //facing the player
                    if (*this).attackDebounceTime < level.time {
                        let mut bolt: *mut gentity_t;

                        bolt = G_Spawn();

                        (*bolt).classname = b"tie_proj\0".as_ptr() as *const c_char;
                        (*bolt).nextthink = level.time + 10000;
                        (*bolt).e_ThinkFunc = &thinkF_G_FreeEntity as *const ();
                        (*bolt).s.eType = ET_MISSILE;
                        (*bolt).s.weapon = WP_BLASTER;
                        (*bolt).owner = this;
                        (*bolt).damage = 30;
                        (*bolt).dflags = 1; // DAMAGE_NO_KNOCKBACK		// Don't push them around, or else we are constantly re-aiming
                        (*bolt).splashDamage = 0;
                        (*bolt).splashRadius = 0;
                        (*bolt).methodOfDeath = MOD_ENERGY;	// ?
                        (*bolt).clipmask = MASK_SHOT;

                        (*bolt).s.pos.trType = TR_LINEAR;
                        (*bolt).s.pos.trTime = level.time;		// move a bit on the very first frame
                        VectorCopy(&(*this).currentOrigin, &mut (*bolt).s.pos.trBase);
                        VectorScale(&fwd, 8000.0, &mut (*bolt).s.pos.trDelta);
                        SnapVector(&mut (*bolt).s.pos.trDelta);		// save net bandwidth
                        VectorCopy(&(*this).currentOrigin, &mut (*bolt).currentOrigin);

                        if Q_irand(0, 2) == 0 {
                            G_SoundOnEnt(bolt, CHAN_VOICE, b"sound/weapons/tie_fighter/tie_fire.wav\0".as_ptr() as *const c_char);
                        } else {
                            G_SoundOnEnt(bolt, CHAN_VOICE, b"sound/weapons/tie_fighter/tie_fire2.wav\0".as_ptr() as *const c_char); // va implementation simplified
                        }
                        (*this).attackDebounceTime = level.time + Q_irand(300, 2000);
                    }
                }
            }
        }

        if playerDist < 1024.0 {
            //within range to start our sound
            if dot > 0.0 {
                if (*this).fly_sound_debounce_time == 0 {
                    //start sound
                    G_SoundOnEnt(this, CHAN_VOICE, b"sound/weapons/tie_fighter/tiepass1.wav\0".as_ptr() as *const c_char);
                    (*this).fly_sound_debounce_time = 2000;
                } else {
                    //sound already started
                    (*this).fly_sound_debounce_time = -1;
                }
            }
        } else if (*this).fly_sound_debounce_time < level.time {
            (*this).fly_sound_debounce_time = 0;
        }
    }
}

unsafe fn TieFighterUse(this: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if this.is_null() || other.is_null() || activator.is_null() {
        return;
    }

    let mut fwd = [0.0; 3];
    let mut rt = [0.0; 3];
    AngleVectors(&(*this).currentAngles, &mut fwd, &mut rt, std::ptr::null_mut());

    let mut bolt: *mut gentity_t;
    bolt = G_Spawn();

    (*bolt).classname = b"tie_proj\0".as_ptr() as *const c_char;
    (*bolt).nextthink = level.time + 10000;
    (*bolt).e_ThinkFunc = &thinkF_G_FreeEntity as *const ();
    (*bolt).s.eType = ET_MISSILE;
    (*bolt).s.weapon = WP_TIE_FIGHTER;
    (*bolt).owner = this;
    (*bolt).damage = 30;
    (*bolt).dflags = 1; // DAMAGE_NO_KNOCKBACK		// Don't push them around, or else we are constantly re-aiming
    (*bolt).splashDamage = 0;
    (*bolt).splashRadius = 0;
    (*bolt).methodOfDeath = MOD_ENERGY;	// ?
    (*bolt).clipmask = MASK_SHOT;

    (*bolt).s.pos.trType = TR_LINEAR;
    (*bolt).s.pos.trTime = level.time;		// move a bit on the very first frame
    VectorCopy(&(*this).currentOrigin, &mut (*bolt).s.pos.trBase);
    rt[2] += 2.0;
    VectorMA(&(*bolt).s.pos.trBase, -15.0, &rt, &mut (*bolt).s.pos.trBase);
    VectorScale(&fwd, 3000.0, &mut (*bolt).s.pos.trDelta);
    SnapVector(&mut (*bolt).s.pos.trDelta);		// save net bandwidth
    VectorCopy(&(*this).currentOrigin, &mut (*bolt).currentOrigin);

    bolt = G_Spawn();

    (*bolt).classname = b"tie_proj\0".as_ptr() as *const c_char;
    (*bolt).nextthink = level.time + 10000;
    (*bolt).e_ThinkFunc = &thinkF_G_FreeEntity as *const ();
    (*bolt).s.eType = ET_MISSILE;
    (*bolt).s.weapon = WP_TIE_FIGHTER;
    (*bolt).owner = this;
    (*bolt).damage = 30;
    (*bolt).dflags = 1; // DAMAGE_NO_KNOCKBACK		// Don't push them around, or else we are constantly re-aiming
    (*bolt).splashDamage = 0;
    (*bolt).splashRadius = 0;
    (*bolt).methodOfDeath = MOD_ENERGY;	// ?
    (*bolt).clipmask = MASK_SHOT;

    (*bolt).s.pos.trType = TR_LINEAR;
    (*bolt).s.pos.trTime = level.time;		// move a bit on the very first frame
    VectorCopy(&(*this).currentOrigin, &mut (*bolt).s.pos.trBase);
    rt[2] -= 4.0;
    VectorMA(&(*bolt).s.pos.trBase, 15.0, &rt, &mut (*bolt).s.pos.trBase);
    VectorScale(&fwd, 3000.0, &mut (*bolt).s.pos.trDelta);
    SnapVector(&mut (*bolt).s.pos.trDelta);		// save net bandwidth
    VectorCopy(&(*this).currentOrigin, &mut (*bolt).currentOrigin);
}

unsafe fn TouchTieBomb(this: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    // Stop the effect.
    G_StopEffect(G_EffectIndex(b"ships/tiebomber_bomb_falling\0".as_ptr() as *const c_char), (*this).playerModel, gi.G2API_AddBolt(&mut (*this).ghoul2[0], b"model_root\0".as_ptr() as *const c_char), (*this).s.number);

    (*this).e_ThinkFunc = &thinkF_G_FreeEntity as *const ();
    (*this).nextthink = level.time + FRAMETIME;
    G_PlayEffect(G_EffectIndex(b"ships/tiebomber_explosion2\0".as_ptr() as *const c_char), &(*this).currentOrigin, &(*this).currentAngles);
    G_RadiusDamage(&(*this).currentOrigin, this, 900, 500, this, MOD_EXPLOSIVE_SPLASH);
}

unsafe fn TieBomberThink(this: *mut gentity_t) {
    // NOTE: Lerp2Angles will think this thinkfunc if the model is a misc_model_breakable. Watchout
    // for that in a script (try to just use ROFF's?).

    // Stop thinking, you're dead.
    if (*this).health <= 0 {
        return;
    }

    // Needed every think...
    (*this).nextthink = level.time + FRAMETIME;

    let player = &mut g_entities[0];
    let mut playerDir = [0.0; 3];
    let mut playerDist: f32;

    //use player eyepoint
    VectorSubtract(&(*player).currentOrigin, &(*this).currentOrigin, &mut playerDir);
    playerDist = VectorNormalize(&mut playerDir);

    // Time to attack?
    if (*player).health > 0 && playerDist < MIN_PLAYER_DIST as f32 && (*this).attackDebounceTime < level.time {
        let name1: &[u8] = b"models/players/remote/model.glm";
        let bomb = G_CreateObject(this, &mut (*this).s.pos.trBase, &mut (*this).s.apos.trBase, 0, 0, TR_GRAVITY, 0);
        (*bomb).s.modelindex = G_ModelIndex(name1.as_ptr() as *const c_char);
        gi.G2API_InitGhoul2Model(&mut (*bomb).ghoul2[0], name1.as_ptr() as *const c_char, (*bomb).s.modelindex);
        (*bomb).s.radius = 50;
        (*bomb).s.eFlags |= EF_NODRAW;

        // Make the bombs go forward in the bombers direction a little.
        let mut fwd = [0.0; 3];
        let mut rt = [0.0; 3];
        AngleVectors(&(*this).currentAngles, &mut fwd, &mut rt, std::ptr::null_mut());
        rt[2] -= 0.5;
        VectorMA(&(*bomb).s.pos.trBase, -30.0, &rt, &mut (*bomb).s.pos.trBase);
        VectorScale(&fwd, 300.0, &mut (*bomb).s.pos.trDelta);
        SnapVector(&mut (*bomb).s.pos.trDelta);		// save net bandwidth

        // Start the effect.
        G_PlayEffect(G_EffectIndex(b"ships/tiebomber_bomb_falling\0".as_ptr() as *const c_char), &(*bomb).playerModel, gi.G2API_AddBolt(&mut (*bomb).ghoul2[0], b"model_root\0".as_ptr() as *const c_char), (*bomb).s.number, &(*bomb).currentOrigin, 1000, true);

        // Set the tie bomb to have a touch function, so when it hits the ground (or whatever), there's a nice 'boom'.
        (*bomb).e_TouchFunc = &touchF_TouchTieBomb as *const ();

        (*this).attackDebounceTime = level.time + 1000;
    }
}

unsafe fn misc_model_breakable_gravity_init(ent: *mut gentity_t, dropToFloor: bool) {
    //G_SoundIndex( "sound/movers/objects/objectHurt.wav" );
    G_EffectIndex(b"melee/kick_impact\0".as_ptr() as *const c_char);
    G_EffectIndex(b"melee/kick_impact_silent\0".as_ptr() as *const c_char);
    //G_SoundIndex( "sound/weapons/melee/punch1.mp3" );
    //G_SoundIndex( "sound/weapons/melee/punch2.mp3" );
    //G_SoundIndex( "sound/weapons/melee/punch3.mp3" );
    //G_SoundIndex( "sound/weapons/melee/punch4.mp3" );
    G_SoundIndex(b"sound/movers/objects/objectHit.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/movers/objects/objectHitHeavy.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/movers/objects/objectBreak.wav\0".as_ptr() as *const c_char);
    //FIXME: dust impact effect when hits ground?
    (*ent).s.eType = ET_GENERAL;
    (*ent).s.eFlags |= EF_BOUNCE_HALF;
    (*ent).clipmask = MASK_SOLID | CONTENTS_BODY | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP;//?
    if (*ent).mass == 0.0 {
        //not overridden by designer
        (*ent).mass = VectorLength(&(*ent).maxs) + VectorLength(&(*ent).mins);
    }
    (*ent).physicsBounce = (*ent).mass;
    //drop to floor
    let mut tr: trace_t = std::mem::zeroed();
    let mut top = [0.0; 3];
    let mut bottom = [0.0; 3];

    if dropToFloor {
        VectorCopy(&(*ent).currentOrigin, &mut top);
        top[2] += 1.0;
        VectorCopy(&(*ent).currentOrigin, &mut bottom);
        bottom[2] = MIN_WORLD_COORD;
        gi.trace(&mut tr, &top, &(*ent).mins, &(*ent).maxs, &bottom, (*ent).s.number, MASK_NPCSOLID);
        if !tr.allsolid && !tr.startsolid && tr.fraction < 1.0 {
            G_SetOrigin(ent, &tr.endpos);
            gi.linkentity(ent);
        }
    } else {
        G_SetOrigin(ent, &(*ent).currentOrigin);
        gi.linkentity(ent);
    }
    //set up for object thinking
    if VectorCompare(&(*ent).s.pos.trDelta, &vec3_origin) {
        //not moving
        (*ent).s.pos.trType = TR_STATIONARY;
    } else {
        (*ent).s.pos.trType = TR_GRAVITY;
    }
    VectorCopy(&(*ent).currentOrigin, &mut (*ent).s.pos.trBase);
    VectorClear(&mut (*ent).s.pos.trDelta);
    (*ent).s.pos.trTime = level.time;
    if VectorCompare(&(*ent).s.apos.trDelta, &vec3_origin) {
        //not moving
        (*ent).s.apos.trType = TR_STATIONARY;
    } else {
        (*ent).s.apos.trType = TR_LINEAR;
    }
    VectorCopy(&(*ent).currentAngles, &mut (*ent).s.apos.trBase);
    VectorClear(&mut (*ent).s.apos.trDelta);
    (*ent).s.apos.trTime = level.time;
    (*ent).nextthink = level.time + FRAMETIME;
    (*ent).e_ThinkFunc = &thinkF_G_RunObject as *const ();
}

/*QUAKED misc_model_breakable (1 0 0) (-16 -16 -16) (16 16 16) SOLID AUTOANIMATE DEADSOLID NO_DMODEL NO_SMOKE USE_MODEL USE_NOT_BREAK PLAYER_USE NO_EXPLOSION START_OFF
SOLID - Movement is blocked by it, if not set, can still be broken by explosions and shots if it has health
AUTOANIMATE - Will cycle it's anim
DEADSOLID - Stay solid even when destroyed (in case damage model is rather large).
NO_DMODEL - Makes it NOT display a damage model when destroyed, even if one exists
USE_MODEL - When used, will toggle to it's usemodel (model + "_u1.md3")... this obviously does nothing if USE_NOT_BREAK is not checked
USE_NOT_BREAK - Using it, doesn't make it break, still can be destroyed by damage
PLAYER_USE - Player can use it with the use button
NO_EXPLOSION - By default, will explode when it dies...this is your override.
START_OFF - Will start off and will not appear until used.

"model"		arbitrary .md3 file to display
"modelscale"	"x" uniform scale
"modelscale_vec" "x y z" scale model in each axis - height, width and length - bbox will scale with it
"health"	how much health to have - default is zero (not breakable)  If you don't set the SOLID flag, but give it health, it can be shot but will not block NPCs or players from moving
"targetname" when used, dies and displays damagemodel (model + "_d1.md3"), if any (if not, removes itself)
"target" What to use when it dies
"target2" What to use when it's repaired
"target3" What to use when it's used while it's broken
"paintarget" target to fire when hit (but not destroyed)
"count"  the amount of armor/health/ammo given (default 50)
"radius"  Chunk code tries to pick a good volume of chunks, but you can alter this to scale the number of spawned chunks. (default 1)  (.5) is half as many chunks, (2) is twice as many chunks
"NPC_targetname" - Only the NPC with this name can damage this
"forcevisible" - When you turn on force sight (any level), you can see these draw through the entire level...
"redCrosshair" - crosshair turns red when you look at this

"gravity"	if set to 1, this will be affected by gravity
"throwtarget" if set (along with gravity), this thing, when used, will throw itself at the entity whose targetname matches this string
"mass"		if gravity is on, this determines how much damage this thing does when it hits someone.  Default is the size of the object from one corner to the other, that works very well.  Only override if this is an object whose mass should be very high or low for it's size (density)

Damage: default is none
"splashDamage" - damage to do (will make it explode on death)
"splashRadius" - radius for above damage

"team" - This cannot take damage from members of this team:
	"player"
	"neutral"
	"enemy"

"material" - default is "8 - MAT_NONE" - choose from this list:
0 = MAT_METAL		(grey metal)
1 = MAT_GLASS
2 = MAT_ELECTRICAL	(sparks only)
3 = MAT_ELEC_METAL	(METAL chunks and sparks)
4 =	MAT_DRK_STONE	(brown stone chunks)
5 =	MAT_LT_STONE	(tan stone chunks)
6 =	MAT_GLASS_METAL (glass and METAL chunks)
7 = MAT_METAL2		(blue/grey metal)
8 = MAT_NONE		(no chunks-DEFAULT)
9 = MAT_GREY_STONE	(grey colored stone)
10 = MAT_METAL3		(METAL and METAL2 chunk combo)
11 = MAT_CRATE1		(yellow multi-colored crate chunks)
12 = MAT_GRATE1		(grate chunks--looks horrible right now)
13 = MAT_ROPE		(for yavin_trial, no chunks, just wispy bits )
14 = MAT_CRATE2		(red multi-colored crate chunks)
15 = MAT_WHITE_METAL (white angular chunks for Stu, NS_hideout )
*/
pub unsafe fn SP_misc_model_breakable(ent: *mut gentity_t) {
    let mut damageModel: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut chunkModel: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut useModel: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut len: usize;

    // Chris F. requested default for misc_model_breakable to be NONE...so don't arbitrarily change this.
    G_SpawnInt(b"material\0".as_ptr() as *const c_char, b"8\0".as_ptr() as *const c_char, &mut (*ent).material as *mut c_int);
    G_SpawnFloat(b"radius\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, &mut (*ent).radius); // used to scale chunk code if desired by a designer
    let mut bHasScale = G_SpawnVector(b"modelscale_vec\0".as_ptr() as *const c_char, b"0 0 0\0".as_ptr() as *const c_char, &mut (*ent).s.modelScale);
    if !bHasScale {
        let mut temp: f32 = 0.0;
        G_SpawnFloat(b"modelscale\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut temp);
        if temp != 0.0 {
            (*ent).s.modelScale[0] = temp;
            (*ent).s.modelScale[1] = temp;
            (*ent).s.modelScale[2] = temp;
            bHasScale = true;
        }
    }

    CacheChunkEffects((*ent).material);
    misc_model_breakable_init(ent);

    len = strlen((*ent).model) - 4;
    assert!(*(*ent).model.add(len) as i32 == b'.' as i32);//we're expecting ".md3"
    strncpy(&mut damageModel[0], (*ent).model, MAX_QPATH);
    damageModel[len] = 0;	//chop extension
    strncpy(&mut chunkModel[0], &damageModel[0], MAX_QPATH);
    strncpy(&mut useModel[0], &damageModel[0], MAX_QPATH);

    if (*ent).takedamage {
        //Dead/damaged model
        if ((*ent).spawnflags & 8) == 0 {	//no dmodel
            strcat(&mut damageModel[0], b"_d1.md3\0".as_ptr() as *const c_char);
            (*ent).s.modelindex2 = G_ModelIndex(&damageModel[0]);
        }

        //Chunk model
        strcat(&mut chunkModel[0], b"_c1.md3\0".as_ptr() as *const c_char);
        (*ent).s.modelindex3 = G_ModelIndex(&chunkModel[0]);
    }

    //Use model
    if ((*ent).spawnflags & 32) != 0 {	//has umodel
        strcat(&mut useModel[0], b"_u1.md3\0".as_ptr() as *const c_char);
        (*ent).sound1to2 = G_ModelIndex(&useModel[0]);
    }
    if (*ent).mins[0] == 0.0 && (*ent).mins[1] == 0.0 && (*ent).mins[2] == 0.0 {
        VectorSet(&mut (*ent).mins, -16.0, -16.0, -16.0);
    }
    if (*ent).maxs[0] == 0.0 && (*ent).maxs[1] == 0.0 && (*ent).maxs[2] == 0.0 {
        VectorSet(&mut (*ent).maxs, 16.0, 16.0, 16.0);
    }

    // Scale up the tie-bomber bbox a little.
    if !(*ent).model.is_null() && Q_stricmp(b"models/map_objects/ships/tie_bomber.md3\0".as_ptr() as *const c_char, (*ent).model) == 0 {
        VectorSet(&mut (*ent).mins, -80.0, -80.0, -80.0);
        VectorSet(&mut (*ent).maxs, 80.0, 80.0, 80.0);

        //ent->s.modelScale[ 0 ] = ent->s.modelScale[ 1 ] = ent->s.modelScale[ 2 ] *= 2.0f;
        //bHasScale = qtrue;
    }

    if bHasScale {
        //scale the x axis of the bbox up.
        (*ent).maxs[0] *= (*ent).s.modelScale[0];//*scaleFactor;
        (*ent).mins[0] *= (*ent).s.modelScale[0];//*scaleFactor;

        //scale the y axis of the bbox up.
        (*ent).maxs[1] *= (*ent).s.modelScale[1];//*scaleFactor;
        (*ent).mins[1] *= (*ent).s.modelScale[1];//*scaleFactor;

        //scale the z axis of the bbox up and adjust origin accordingly
        (*ent).maxs[2] *= (*ent).s.modelScale[2];
        let oldMins2 = (*ent).mins[2];
        (*ent).mins[2] *= (*ent).s.modelScale[2];
        (*ent).s.origin[2] += oldMins2 - (*ent).mins[2];
    }

    if ((*ent).spawnflags & 2) != 0 {
        (*ent).s.eFlags |= EF_ANIM_ALLFAST;
    }

    G_SetOrigin(ent, &(*ent).s.origin);
    G_SetAngles(ent, &(*ent).s.angles);
    gi.linkentity(ent);

    if ((*ent).spawnflags & 128) != 0 {
        //Can be used by the player's BUTTON_USE
        (*ent).svFlags |= SVF_PLAYER_USABLE;
    }

    if !(*ent).team.is_null() && *(*ent).team as c_int != 0 {
        (*ent).noDamageTeam = GetIDForString(&TeamTable as *const c_void, (*ent).team) as c_int;
        if (*ent).noDamageTeam == TEAM_FREE {
            G_Error(b"team name %s not recognized\n\0".as_ptr() as *const c_char, (*ent).team);
        }
    }

    (*ent).team = std::ptr::null();

    //HACK
    if !(*ent).model.is_null() && Q_stricmp(b"models/map_objects/ships/x_wing_nogear.md3\0".as_ptr() as *const c_char, (*ent).model) == 0 {
        if (*ent).splashDamage > 0 && (*ent).splashRadius > 0 {
            (*ent).s.loopSound = G_SoundIndex(b"sound/vehicles/x-wing/loop.wav\0".as_ptr() as *const c_char);
            (*ent).s.eFlags |= EF_LESS_ATTEN;
        }
    } else if !(*ent).model.is_null() && Q_stricmp(b"models/map_objects/ships/tie_fighter.md3\0".as_ptr() as *const c_char, (*ent).model) == 0 {
        //run a think
        G_EffectIndex(b"explosions/fighter_explosion2\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/weapons/tie_fighter/tiepass1.wav\0".as_ptr() as *const c_char);
        /*		G_SoundIndex( "sound/weapons/tie_fighter/tiepass2.wav" );
        G_SoundIndex( "sound/weapons/tie_fighter/tiepass3.wav" );
        G_SoundIndex( "sound/weapons/tie_fighter/tiepass4.wav" );
        G_SoundIndex( "sound/weapons/tie_fighter/tiepass5.wav" );*/
        G_SoundIndex(b"sound/weapons/tie_fighter/tie_fire.wav\0".as_ptr() as *const c_char);
        /*		G_SoundIndex( "sound/weapons/tie_fighter/tie_fire2.wav" );
        G_SoundIndex( "sound/weapons/tie_fighter/tie_fire3.wav" );*/
        G_SoundIndex(b"sound/weapons/tie_fighter/TIEexplode.wav\0".as_ptr() as *const c_char);
        RegisterItem(FindItemForWeapon(WP_TIE_FIGHTER));

        (*ent).s.eFlags |= EF_LESS_ATTEN;

        if (*ent).splashDamage > 0 && (*ent).splashRadius > 0 {
            (*ent).s.loopSound = G_SoundIndex(b"sound/vehicles/tie-bomber/loop.wav\0".as_ptr() as *const c_char);
            //ent->e_ThinkFunc = thinkF_TieFighterThink;
            //ent->e_UseFunc = thinkF_TieFighterThink;
            //ent->nextthink = level.time + FRAMETIME;
            (*ent).e_UseFunc = &useF_TieFighterUse as *const ();

            // Yeah, I could have just made this value changable from the editor, but I
            // need it immediately!
            let mut light: f32 = 0.0;
            let mut color = [0.0; 3];
            let mut lightSet: bool;
            let mut colorSet: bool;

            // if the "color" or "light" keys are set, setup constantLight
            lightSet = true;//G_SpawnFloat( "light", "100", &light );
            light = 255.0;
            //colorSet = "1 1 1"//G_SpawnVector( "color", "1 1 1", color );
            colorSet = true;
            color[0] = 1.0; color[1] = 1.0; color[2] = 1.0;
            if lightSet || colorSet {
                let mut r: c_int;
                let mut g: c_int;
                let mut b: c_int;
                let mut i: c_int;

                r = (color[0] * 255.0) as c_int;
                if r > 255 {
                    r = 255;
                }
                g = (color[1] * 255.0) as c_int;
                if g > 255 {
                    g = 255;
                }
                b = (color[2] * 255.0) as c_int;
                if b > 255 {
                    b = 255;
                }
                i = (light / 4.0) as c_int;
                if i > 255 {
                    i = 255;
                }
                (*ent).s.constantLight = r | (g << 8) | (b << 16) | (i << 24);
            }
        }
    } else if !(*ent).model.is_null() && Q_stricmp(b"models/map_objects/ships/tie_bomber.md3\0".as_ptr() as *const c_char, (*ent).model) == 0 {
        G_EffectIndex(b"ships/tiebomber_bomb_falling\0".as_ptr() as *const c_char);
        G_EffectIndex(b"ships/tiebomber_explosion2\0".as_ptr() as *const c_char);
        G_EffectIndex(b"explosions/fighter_explosion2\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/weapons/tie_fighter/TIEexplode.wav\0".as_ptr() as *const c_char);
        (*ent).e_ThinkFunc = &thinkF_TieBomberThink as *const ();
        (*ent).nextthink = level.time + FRAMETIME;
        (*ent).attackDebounceTime = level.time + 1000;
        // We only take damage from a heavy weapon class missiles.
        (*ent).flags |= FL_DMG_BY_HEAVY_WEAP_ONLY;
        (*ent).s.loopSound = G_SoundIndex(b"sound/vehicles/tie-bomber/loop.wav\0".as_ptr() as *const c_char);
        (*ent).s.eFlags |= EF_LESS_ATTEN;
    }

    let mut grav: f32 = 0.0;
    G_SpawnFloat(b"gravity\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut grav);
    if grav != 0.0 {
        //affected by gravity
        G_SetAngles(ent, &(*ent).s.angles);
        G_SetOrigin(ent, &(*ent).currentOrigin);
        G_SpawnString(b"throwtarget\0".as_ptr() as *const c_char, std::ptr::null(), &mut (*ent).target4); // used to throw itself at something
        misc_model_breakable_gravity_init(ent, true);
    }

    // Start off.
    if ((*ent).spawnflags & 4096) != 0 {
        (*ent).spawnContents = (*ent).contents;	// It Navs can temporarly turn it "on"
        (*ent).s.solid = 0;
        (*ent).contents = 0;
        (*ent).clipmask = 0;
        (*ent).svFlags |= SVF_NOCLIENT;
        (*ent).s.eFlags |= EF_NODRAW;
        (*ent).count = 0;
    }

    let mut forceVisible: c_int = 0;
    G_SpawnInt(b"forcevisible\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut forceVisible);
    if forceVisible != 0 {
        //can see these through walls with force sight, so must be broadcast
        //ent->svFlags |= SVF_BROADCAST;
        (*ent).s.eFlags |= EF_FORCE_VISIBLE;
    }

    let mut redCrosshair: c_int = 0;
    G_SpawnInt(b"redCrosshair\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut redCrosshair);
    if redCrosshair != 0 {
        //can see these through walls with force sight, so must be broadcast
        (*ent).flags |= FL_RED_CROSSHAIR;
    }
}


//----------------------------------
//
// Breaking Glass Technology
//
//----------------------------------

// Really naughty cheating.  Put in an EVENT at some point...

unsafe fn funcGlassDie(this: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int) {
    let mut verts: [[f32; 3]; 4] = [[0.0; 3]; 4];
    let mut normal = [0.0; 3];

    // if a missile is stuck to us, blow it up so we don't look dumb....we could, alternately, just let the missile drop off??
    for i in 0..MAX_GENTITIES {
        if g_entities[i].s.groundEntityNum == (*this).s.number && (g_entities[i].s.eFlags & EF_MISSILE_STICK) != 0 {
            G_Damage(&mut g_entities[i], this, this, std::ptr::null(), std::ptr::null(), 99999, 0, MOD_CRUSH); //?? MOD?
        }
    }

    // Really naughty cheating.  Put in an EVENT at some point...
    cgi_R_GetBModelVerts(cgs.inlineDrawModel[(*this).s.modelindex as usize], &mut verts, &mut normal);
    CG_DoGlass(&verts, &normal, &(*this).pos1, &(*this).pos2, (*this).splashRadius as f32);

    (*this).takedamage = false;//stop chain reaction runaway loops

    G_SetEnemy(this, (*this).enemy);

    //NOTE: MUST do this BEFORE clearing contents, or you may not open the area portal!!!
    gi.AdjustAreaPortalState(this, true);

    //So chunks don't get stuck inside me
    (*this).s.solid = 0;
    (*this).contents = 0;
    (*this).clipmask = 0;
    gi.linkentity(this);

    if !(*this).target.is_null() && !attacker.is_null() {
        G_UseTargets(this, attacker);
    }

    G_FreeEntity(this);
}

//-----------------------------------------------------
unsafe fn funcGlassUse(this: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    let mut temp1 = [0.0; 3];
    let mut temp2 = [0.0; 3];

    // For now, we just break on use
    G_ActivateBehavior(this, BSET_USE);

    VectorAdd(&(*this).mins, &(*this).maxs, &mut temp1);
    VectorScale(&temp1, 0.5, &mut temp1);

    VectorAdd(&(*other).mins, &(*other).maxs, &mut temp2);
    VectorScale(&temp2, 0.5, &mut temp2);

    VectorSubtract(&temp1, &temp2, &mut (*this).pos2);
    VectorCopy(&temp1, &mut (*this).pos1);

    VectorNormalize(&mut (*this).pos2);
    VectorScale(&(*this).pos2, 390.0, &mut (*this).pos2);

    (*this).splashRadius = 40; // ?? some random number, maybe it's ok?

    funcGlassDie(this, other, activator, (*this).health, MOD_UNKNOWN, 0, 0);
}

//-----------------------------------------------------
/*QUAKED func_glass (0 .8 .5) ? INVINCIBLE
When destroyed, fires it's target, breaks into glass chunks and plays glass noise
For now, instantly breaks on impact

INVINCIBLE - can only be broken by being used

"targetname" entities with matching target will fire it
"target"	all entities with a matching targetname will be used when this is destroyed
"health"	default is 1
*/
//-----------------------------------------------------
pub unsafe fn SP_func_glass(this: *mut gentity_t) {
    if ((*this).spawnflags & INVINCIBLE) == 0 {
        if (*this).health == 0 {
            (*this).health = 1;
        }
    }

    if (*this).health != 0 {
        (*this).takedamage = true;
    }

    (*this).e_UseFunc = &useF_funcGlassUse as *const ();
    (*this).e_DieFunc = &dieF_funcGlassDie as *const ();

    VectorCopy(&(*this).s.origin, &mut (*this).pos1);

    gi.SetBrushModel(this, (*this).model);
    (*this).svFlags |= SVF_GLASS_BRUSH | SVF_BBRUSH;
    (*this).material = MAT_GLASS;

    (*this).s.eType = ET_MOVER;

    (*this).s.pos.trType = TR_STATIONARY;
    VectorCopy(&(*this).pos1, &mut (*this).s.pos.trBase);

    G_SoundIndex(b"sound/effects/glassbreak1.wav\0".as_ptr() as *const c_char);
    G_EffectIndex(b"misc/glass_impact\0".as_ptr() as *const c_char);

    gi.linkentity(this);
}

pub unsafe fn G_EntIsBreakable(entityNum: c_int, breaker: *mut gentity_t) -> bool {
    //breakable brush/model that can actually be broken
    if entityNum < 0 || entityNum >= 2048 {
        // ENTITYNUM_WORLD
        return false;
    }

    let ent = &g_entities[entityNum as usize];
    if !ent.takedamage {
        return false;
    }

    if !ent.NPC_targetname.is_null() {
        //only a specific entity can break this!
        if breaker.is_null()
            || (*breaker).targetname.is_null()
            || Q_stricmp(ent.NPC_targetname, (*breaker).targetname) != 0 {
            //I'm not the one who can break it
            return false;
        }
    }

    if (ent.svFlags & SVF_GLASS_BRUSH) != 0 {
        return true;
    }
    if (ent.svFlags & SVF_BBRUSH) != 0 {
        return true;
    }
    if Q_stricmp(b"misc_model_breakable\0".as_ptr() as *const c_char, ent.classname) == 0 {
        return true;
    }
    if Q_stricmp(b"misc_maglock\0".as_ptr() as *const c_char, ent.classname) == 0 {
        return true;
    }

    false
}

// Stub placeholders for external functions/objects not yet defined
unsafe fn NAV_WayEdgesNowClear(_ent: *mut gentity_t) {}

// Stub for g_gravity and g_knockback cvars
#[allow(non_upper_case_globals)]
pub static mut g_gravity: cvar_t = cvar_t { value: 800.0 };
#[allow(non_upper_case_globals)]
pub static mut g_knockback: cvar_t = cvar_t { value: 1000.0 };

#[repr(C)]
pub struct cvar_t {
    pub value: f32,
}
