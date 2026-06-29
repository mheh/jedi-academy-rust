// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// (original includes: g_headers.h, b_local.h, g_nav.h, anims.h, g_navigator.h)

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_uint, c_float, c_char, c_void};

// ---------------------------------------------------------------------------
// Fundamental C type aliases
// ---------------------------------------------------------------------------
pub type qboolean = c_int;
pub type vec_t    = c_float;
pub type vec3_t   = [vec_t; 3];
pub type taskID_t = c_int;

pub const qtrue:  qboolean = 1;
pub const qfalse: qboolean = 0;

// ---------------------------------------------------------------------------
// Opaque #[repr(C)] stubs for types defined in included headers
// ---------------------------------------------------------------------------
/// gentity_t — game entity (g_local.h)
#[repr(C)] pub struct gentity_t      { pub _p: [u8; 0] }
/// gNPC_t — NPC info block (NPC.h / b_local.h)
#[repr(C)] pub struct gNPC_t         { pub _p: [u8; 0] }
/// gclient_t — player/NPC client state (g_local.h)
#[repr(C)] pub struct gclient_t      { pub _p: [u8; 0] }
/// AIGroupInfo_t — squad group data (b_local.h)
#[repr(C)] pub struct AIGroupInfo_t  { pub _p: [u8; 0] }
/// cvar_t — console variable (q_shared.h)
#[repr(C)] pub struct cvar_t         { pub _p: [u8; 0] }
/// trace_t — collision/clip trace result (q_shared.h)
#[repr(C)] pub struct trace_t        { pub _p: [u8; 0] }
/// mdxaBone_t — Ghoul2 bone matrix (ghoul2_public.h)
#[repr(C)] pub struct mdxaBone_t     { pub _p: [u8; 0] }
/// usercmd_t — accumulated NPC user command for the frame (q_shared.h)
#[repr(C)] pub struct usercmd_t      { pub _p: [u8; 0] }
/// level_locals_t — level/game-state singleton (g_local.h)
#[repr(C)] pub struct level_locals_t { pub _p: [u8; 0] }
/// cg_t — client-game state singleton (cg_local.h); used for cg.time
#[repr(C)] pub struct cg_t           { pub _p: [u8; 0] }

// ---------------------------------------------------------------------------
// External game-module globals (defined elsewhere in the engine / game module)
// ---------------------------------------------------------------------------
extern "C" {
    /// Current NPC entity being processed this think tick
    pub static mut NPC:    *mut gentity_t;
    /// NPCInfo for the current NPC (pointer into NPC->NPC)
    pub static mut NPCInfo: *mut gNPC_t;
    /// Accumulated usercmd for the current NPC's frame
    pub static mut ucmd:   usercmd_t;
    /// Level/game-state singleton
    pub static mut level:  level_locals_t;
    /// All game entities; zero-length Rust marker — index via raw pointer arithmetic
    pub static g_entities: [gentity_t; 0];
    /// Zero vector constant
    pub static vec3_origin: vec3_t;
    /// Skill-level cvar (g_spskill)
    pub static mut g_spskill: *mut cvar_t;
    /// Saber-combat debug verbosity cvar (d_saberCombat)
    pub static mut d_saberCombat: *mut cvar_t;
    /// Asynchronous group-AI toggle (declared `extern` at top of this file)
    pub static mut d_asynchronousGroupAI: *mut cvar_t;
    /// Client-game state singleton; only cg.time is read in Noghri melee trace
    pub static mut cg: cg_t;
}

// ---------------------------------------------------------------------------
// External functions — explicitly `extern`-declared at the top of
// AI_Stormtrooper.cpp
// ---------------------------------------------------------------------------
extern "C" {
    pub fn CG_DrawAlert(origin: *mut vec3_t, rating: c_float);
    pub fn G_AddVoiceEvent(self_: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    pub fn AI_GroupUpdateSquadstates(group: *mut AIGroupInfo_t, member: *mut gentity_t, newSquadState: c_int);
    pub fn AI_GroupContainsEntNum(group: *mut AIGroupInfo_t, entNum: c_int) -> qboolean;
    pub fn AI_GroupUpdateEnemyLastSeen(group: *mut AIGroupInfo_t, spot: *mut vec3_t);
    pub fn AI_GroupUpdateClearShotTime(group: *mut AIGroupInfo_t);
    pub fn NPC_TempLookTarget(self_: *mut gentity_t, lookEntNum: c_int, minLookTime: c_int, maxLookTime: c_int);
    pub fn G_ExpandPointToBBox(point: *mut vec3_t, mins: *const vec3_t, maxs: *const vec3_t, ignore: c_int, clipmask: c_int) -> qboolean;
    pub fn ChangeWeapon(ent: *mut gentity_t, newWeapon: c_int);
    pub fn NPC_CheckGetNewWeapon();
    pub fn Q3_TaskIDPending(ent: *mut gentity_t, taskType: taskID_t) -> qboolean;
    pub fn GetTime(lastTime: c_int) -> c_int;
    pub fn NPC_AimAdjust(change: c_int);
    pub fn FlyingCreature(ent: *mut gentity_t) -> qboolean;
    pub fn NPC_EvasionSaber();
    pub fn RT_Flying(self_: *mut gentity_t) -> qboolean;
    // Declared with a local `extern` immediately before first use in this file
    pub fn G_Knockdown(self_: *mut gentity_t, attacker: *mut gentity_t, pushDir: *const vec3_t, strength: c_float, breakSaberLock: qboolean);
    pub fn G_TuskenAttackAnimDamage(self_: *mut gentity_t) -> qboolean;
}

// ---------------------------------------------------------------------------
// External functions — brought in via included headers
// (b_local.h, g_local.h, g_nav.h, anims.h, g_navigator.h, q_shared.h, …)
// ---------------------------------------------------------------------------
extern "C" {
    // Sound
    pub fn G_SoundIndex(sound: *const c_char) -> c_int;
    pub fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, sound: *const c_char);
    pub fn G_Sound(ent: *mut gentity_t, soundIndex: c_int);
    // Flee / pain
    pub fn G_StartFlee(self_: *mut gentity_t, enemy: *mut gentity_t, dangerPoint: *mut vec3_t, dangerLevel: c_int, minTime: c_int, maxTime: c_int);
    pub fn NPC_StartFlee(enemy: *mut gentity_t, point: *mut vec3_t, dangerLevel: c_int, minTime: c_int, maxTime: c_int);
    pub fn NPC_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    // Combat points
    pub fn NPC_FreeCombatPoint(combatPoint: c_int, tryClear: qboolean);
    pub fn NPC_SetCombatPoint(cp: c_int);
    pub fn NPC_FindCombatPoint(origin: *const vec3_t, avoidPoint: *const vec3_t, enemyPos: *const vec3_t, flags: c_int, avoidDist: c_float) -> c_int;
    pub fn NPC_FindCombatPointRetry(origin: *const vec3_t, avoidPoint: *const vec3_t, enemyPos: *const vec3_t, cpFlags: *mut c_int, avoidDist: c_float, lastFailedCP: c_int) -> c_int;
    pub fn NPC_ReachedGoal();
    // Group AI
    pub fn AI_GetGroup(self_: *mut gentity_t);
    // Movement / navigation
    pub fn NPC_MoveToGoal(tryStraight: qboolean) -> qboolean;
    /// NPC_SetMoveGoal — C++ default arg `cp = -1`; always pass explicitly here
    pub fn NPC_SetMoveGoal(self_: *mut gentity_t, point: *mut vec3_t, dist: c_float, isNavGoal: qboolean, cp: c_int) -> c_int;
    pub fn NPC_BSSearchStart(waypoint: c_int, behavior: c_int);
    pub fn NPC_BSPatrol();
    // Alert / danger
    pub fn NPC_CheckForDanger(alertEvent: c_int) -> qboolean;
    /// NPC_CheckAlertEvents — C++ default args (checkSight, checkSound [, lastAlertID=-1 [, fullCheck=qtrue [, minLevel=AEL_MINOR]]])
    pub fn NPC_CheckAlertEvents(checkSight: qboolean, checkSound: qboolean, lastAlertID: c_int, fullCheck: qboolean, minLevel: c_int) -> c_int;
    // Enemy validity / checks
    pub fn NPC_CheckEnemyExt() -> qboolean;
    /// NPC_ClearLOS is overloaded in C++; two Rust-level wrappers cover the two forms used here.
    pub fn NPC_ClearLOS4(ent: *mut gentity_t) -> qboolean;    // NPC_ClearLOS(entity*)
    pub fn NPC_ClearLOS5(point: *const vec3_t) -> qboolean;   // NPC_ClearLOS(vec3_t)
    pub fn NPC_GetHFOVPercentage(spot: *const vec3_t, from: *const vec3_t, angles: *const vec3_t, hfov: c_float) -> c_float;
    pub fn NPC_GetVFOVPercentage(spot: *const vec3_t, from: *const vec3_t, angles: *const vec3_t, vfov: c_float) -> c_float;
    pub fn NPC_FaceEnemy(doPitch: qboolean);
    pub fn NPC_FacePosition(position: *mut vec3_t, doPitch: qboolean);
    pub fn NPC_UpdateAngles(doPitch: qboolean, doYaw: qboolean);
    pub fn NPC_ChangeWeapon(weapon: c_int);
    pub fn NPC_ValidEnemy(ent: *mut gentity_t) -> qboolean;
    pub fn NPC_ShotEntity(ent: *mut gentity_t, impactPos: *mut vec3_t) -> c_int;
    pub fn NPC_SetAnim(ent: *mut gentity_t, setAnimParts: c_int, anim: c_int, setAnimFlags: c_int);
    // Entity / enemy management
    pub fn G_SetEnemy(self_: *mut gentity_t, enemy: *mut gentity_t);
    pub fn G_ClearEnemy(self_: *mut gentity_t);
    pub fn G_ActivateBehavior(self_: *mut gentity_t, bset: c_int) -> qboolean;
    pub fn G_ClearLOS(self_: *mut gentity_t, enemy: *mut gentity_t) -> qboolean;
    pub fn G_Damage(target: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, dir: *mut vec3_t, point: *const vec3_t, damage: c_int, dflags: c_int, mod_: c_int);
    pub fn G_RemoveWeaponModels(ent: *mut gentity_t);
    pub fn G_DebugLine(start: *const vec3_t, end: *const vec3_t, time: c_int, color: c_uint, force: qboolean);
    /// InFOV — overloaded in C++; two Rust wrappers for the two call patterns used.
    pub fn InFOV3(origin: *const vec3_t, from: *const vec3_t, angles: *const vec3_t, hfov: c_int, vfov: c_int) -> qboolean;
    pub fn InFOV2(target: *mut gentity_t, npc: *mut gentity_t, hfov: c_int, vfov: c_int) -> qboolean;
    // Math helpers (implemented as macros/inlines in q_math.c / q_shared.h)
    pub fn DistanceSquared(p1: *const vec3_t, p2: *const vec3_t) -> c_float;
    pub fn Distance(p1: *const vec3_t, p2: *const vec3_t) -> c_float;
    pub fn VectorLength(v: *const vec3_t) -> c_float;
    pub fn AngleDelta(a1: c_float, a2: c_float) -> c_float;
    pub fn VectorNormalize(v: *mut vec3_t) -> c_float;
    pub fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
    pub fn vectoangles(vec: *const vec3_t, angles: *mut vec3_t);
    pub fn GetAnglesForDirection(p1: *const vec3_t, p2: *const vec3_t, out: *mut vec3_t);
    pub fn CalcEntitySpot(ent: *const gentity_t, spot: c_int, out: *mut vec3_t);
    // Goal / NPC globals
    pub fn UpdateGoal() -> *mut gentity_t;
    pub fn SaveNPCGlobals();
    pub fn RestoreNPCGlobals();
    pub fn SetNPCGlobals(ent: *mut gentity_t);
    // Weapon / combat actions
    pub fn WeaponThink(inCombat: qboolean);
    // Misc
    pub fn OnSameTeam(ent1: *mut gentity_t, ent2: *mut gentity_t) -> qboolean;
    pub fn PInUse(entityNum: c_int) -> qboolean;
    // Timer system
    pub fn TIMER_Done(self_: *mut gentity_t, name: *const c_char) -> qboolean;
    pub fn TIMER_Done2(self_: *mut gentity_t, name: *const c_char, remove: qboolean) -> qboolean;
    pub fn TIMER_Set(self_: *mut gentity_t, name: *const c_char, time: c_int);
    pub fn TIMER_Get(self_: *mut gentity_t, name: *const c_char) -> c_int;
    pub fn TIMER_Exists(self_: *mut gentity_t, name: *const c_char) -> qboolean;
    // RNG
    pub fn Q_irand(value1: c_int, value2: c_int) -> c_int;
    pub fn Q_flrand(value1: c_float, value2: c_float) -> c_float;
    pub fn random() -> c_float;
    // Formatted string (varargs — use c_str! at each call site)
    pub fn va(format: *const c_char, ...) -> *const c_char;
    // Navigation — C++ NAV:: namespace; porting note: mangled C++ names must be
    // resolved at link time; these stubs use flat Rust names.
    pub fn NAV_GetNearestNode(ent: *mut gentity_t) -> c_int;
    pub fn NAV_EstimateCostToGoal(startID: c_int, endID: c_int) -> c_float;
    pub fn NAV_InSameRegion(ent: *mut gentity_t, pos: *const vec3_t) -> bool;
    // Steering — C++ STEER:: namespace
    pub fn STEER_Reached(ent: *mut gentity_t, goal: *mut gentity_t, dist: c_float, flying: bool) -> bool;
    // Engine game-interface (gi.*) — gi is a C++ struct of function pointers;
    // these flat extern "C" stubs forward-declare only the members used in this file.
    pub fn gi_pointcontents(point: *const vec3_t, passEntityNum: c_int) -> c_int;
    pub fn gi_EntitiesInBox(mins: *const vec3_t, maxs: *const vec3_t, list: *mut *mut gentity_t, maxcount: c_int) -> c_int;
    pub fn gi_trace(results: *mut trace_t, start: *const vec3_t, mins: *const vec3_t, maxs: *const vec3_t, end: *const vec3_t, passEntityNum: c_int, contentmask: c_int, g2Check: c_int, g2TraceRadius: c_int);
    pub fn gi_inPVS(p1: *const vec3_t, p2: *const vec3_t) -> qboolean;
    pub fn gi_Printf(fmt: *const c_char, ...);
    pub fn gi_G2API_AddBolt(ghoul2: *mut c_void, boneName: *const c_char) -> c_int;
    pub fn gi_G2API_GetBoltMatrix(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut mdxaBone_t, angles: *const vec3_t, origin: *const vec3_t, frameNum: c_int, modelList: *mut c_void, scale: *const vec3_t) -> qboolean;
    pub fn gi_G2API_GiveMeVectorFromMatrix(boltMatrix: *mut mdxaBone_t, flags: c_int, vec: *mut vec3_t);
}
// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// #include "g_headers.h"


// #include "b_local.h"
// #include "g_nav.h"
// #include "anims.h"
// #include "g_navigator.h"

// extern void CG_DrawAlert( vec3_t origin, float rating );
// extern void G_AddVoiceEvent( gentity_t *self, int event, int speakDebounceTime );
// extern void AI_GroupUpdateSquadstates( AIGroupInfo_t *group, gentity_t *member, int newSquadState );
// extern qboolean AI_GroupContainsEntNum( AIGroupInfo_t *group, int entNum );
// extern void AI_GroupUpdateEnemyLastSeen( AIGroupInfo_t *group, vec3_t spot );
// extern void AI_GroupUpdateClearShotTime( AIGroupInfo_t *group );
// extern void NPC_TempLookTarget( gentity_t *self, int lookEntNum, int minLookTime, int maxLookTime );
// extern qboolean G_ExpandPointToBBox( vec3_t point, const vec3_t mins, const vec3_t maxs, int ignore, int clipmask );
// extern void ChangeWeapon( gentity_t *ent, int newWeapon );
// extern void NPC_CheckGetNewWeapon( void );
// extern qboolean Q3_TaskIDPending( gentity_t *ent, taskID_t taskType );
// extern int GetTime ( int lastTime );
// extern void NPC_AimAdjust( int change );
// extern qboolean FlyingCreature( gentity_t *ent );
// extern void NPC_EvasionSaber( void );
// extern qboolean RT_Flying( gentity_t *self );

// //extern	CNavigator	navigator;
// extern	cvar_t		*d_asynchronousGroupAI;

const MAX_VIEW_DIST: c_int = 1024;
const MAX_VIEW_SPEED: c_int = 250;
const MAX_LIGHT_INTENSITY: c_int = 255;
const MIN_LIGHT_THRESHOLD: f64 = 0.1; // C: double literal (no f suffix)
const ST_MIN_LIGHT_THRESHOLD: c_int = 30;
const ST_MAX_LIGHT_THRESHOLD: c_int = 180;
const DISTANCE_THRESHOLD: f32 = 0.075_f32;
const MIN_TURN_AROUND_DIST_SQ: c_int = 10000; //(100 squared) don't stop running backwards if your goal is less than 100 away
const SABER_AVOID_DIST: f32 = 128.0_f32; //256.0f
const SABER_AVOID_DIST_SQ: f32 = SABER_AVOID_DIST * SABER_AVOID_DIST;

const DISTANCE_SCALE: f32 = 0.35_f32; //These first three get your base detection rating, ideally add up to 1
const FOV_SCALE: f32 = 0.40_f32; //
const LIGHT_SCALE: f32 = 0.25_f32; //

const SPEED_SCALE: f32 = 0.25_f32; //These next two are bonuses
const TURNING_SCALE: f32 = 0.25_f32; //

const REALIZE_THRESHOLD: f32 = 0.6_f32;
// Porting note: C computes `REALIZE_THRESHOLD * 0.75` as double (0.75 has no f suffix); ported as f32
const CAUTIOUS_THRESHOLD: f32 = REALIZE_THRESHOLD * 0.75_f32;

// qboolean NPC_CheckPlayerTeamStealth( void );
// (forward declaration — defined in a later chunk of this file)

static mut enemyLOS: qboolean = 0;
static mut enemyCS: qboolean = 0;
static mut enemyInFOV: qboolean = 0;
static mut hitAlly: qboolean = 0;
static mut faceEnemy: qboolean = 0;
static mut r#move: qboolean = 0;
static mut shoot: qboolean = 0;
static mut enemyDist: f32 = 0.0;
static mut impactPos: vec3_t = [0.0_f32; 3];

pub static mut groupSpeechDebounceTime: [c_int; team_t::TEAM_NUM_TEAMS as usize] =
    [0; team_t::TEAM_NUM_TEAMS as usize]; //used to stop several group AI from speaking all at once

pub unsafe fn NPC_Saboteur_Precache() {
    G_SoundIndex(b"sound/chars/shadowtrooper/cloak.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/shadowtrooper/decloak.wav\0".as_ptr() as *const c_char);
}

pub unsafe fn Saboteur_Decloak(self_: *mut gentity_t, uncloakTime: c_int) {
    if !self_.is_null() && !(*self_).client.is_null() {
        if (*(*self_).client).ps.powerups[PW_CLOAKED as usize] != 0
            && TIMER_Done(self_, b"decloakwait\0".as_ptr() as *const c_char) != qfalse
        {
            //Uncloak
            (*(*self_).client).ps.powerups[PW_CLOAKED as usize] = 0;
            (*(*self_).client).ps.powerups[PW_UNCLOAKING as usize] =
                (*core::ptr::addr_of!(level)).time + 2000;
            //FIXME: temp sound
            G_SoundOnEnt(
                self_,
                CHAN_ITEM,
                b"sound/chars/shadowtrooper/decloak.wav\0".as_ptr() as *const c_char,
            );
            TIMER_Set(self_, b"nocloak\0".as_ptr() as *const c_char, uncloakTime);

            // Can't Recloak
            //(*(*self_).NPC).aiFlags	&= !NPCAI_SHIELDS;
        }
    }
}

pub unsafe fn Saboteur_Cloak(self_: *mut gentity_t) {
    if !self_.is_null() && !(*self_).client.is_null() && !(*self_).NPC.is_null() {
        //FIXME: need to have this timer set once first?
        if TIMER_Done(self_, b"nocloak\0".as_ptr() as *const c_char) != qfalse {
            //not sitting around waiting to cloak again
            if ((*(*self_).NPC).aiFlags & NPCAI_SHIELDS) == 0 {
                //not allowed to cloak, actually
                // Porting note: original C++ calls Saboteur_Decloak( self ) with 1 arg;
                // uncloakTime default assumed 0 (likely a default parameter in original header)
                Saboteur_Decloak(self_, 0);
            } else if (*(*self_).client).ps.powerups[PW_CLOAKED as usize] == 0 {
                //cloak
                (*(*self_).client).ps.powerups[PW_CLOAKED as usize] = Q3_INFINITE;
                (*(*self_).client).ps.powerups[PW_UNCLOAKING as usize] =
                    (*core::ptr::addr_of!(level)).time + 2000;
                //FIXME: debounce attacks?
                //FIXME: temp sound
                G_SoundOnEnt(
                    self_,
                    CHAN_ITEM,
                    b"sound/chars/shadowtrooper/cloak.wav\0".as_ptr() as *const c_char,
                );
            }
        }
    }
}



//Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_UNDERFIRE: c_int = 1;
const LSTATE_INVESTIGATE: c_int = 2;

pub unsafe fn ST_AggressionAdjust(self_: *mut gentity_t, change: c_int) {
    let upper_threshold: c_int;
    let lower_threshold: c_int;

    (*(*self_).NPC).stats.aggression += change;

    //FIXME: base this on initial NPC stats
    if (*(*self_).client).playerTeam == team_t::TEAM_PLAYER {
        //good guys are less aggressive
        upper_threshold = 7;
        lower_threshold = 1;
    } else {
        //bad guys are more aggressive
        upper_threshold = 10;
        lower_threshold = 3;
    }

    if (*(*self_).NPC).stats.aggression > upper_threshold {
        (*(*self_).NPC).stats.aggression = upper_threshold;
    } else if (*(*self_).NPC).stats.aggression < lower_threshold {
        (*(*self_).NPC).stats.aggression = lower_threshold;
    }
}

pub unsafe fn ST_ClearTimers(ent: *mut gentity_t) {
    TIMER_Set(ent, b"chatter\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"duck\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"stand\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"shuffleTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"sleepTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"enemyLastVisible\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"roamTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"hideTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"attackDelay\0".as_ptr() as *const c_char, 0); //FIXME: Slant for difficulty levels
    TIMER_Set(ent, b"stick\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"scoutTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"flee\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"interrogating\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"verifyCP\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"strafeRight\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"strafeLeft\0".as_ptr() as *const c_char, 0);
}

const SPEECH_CHASE: c_int = 0;
const SPEECH_CONFUSED: c_int = 1;
const SPEECH_COVER: c_int = 2;
const SPEECH_DETECTED: c_int = 3;
const SPEECH_GIVEUP: c_int = 4;
const SPEECH_LOOK: c_int = 5;
const SPEECH_LOST: c_int = 6;
const SPEECH_OUTFLANK: c_int = 7;
const SPEECH_ESCAPING: c_int = 8;
const SPEECH_SIGHT: c_int = 9;
const SPEECH_SOUND: c_int = 10;
const SPEECH_SUSPICIOUS: c_int = 11;
const SPEECH_YELL: c_int = 12;
const SPEECH_PUSHED: c_int = 13;

unsafe fn ST_Speech(self_: *mut gentity_t, speechType: c_int, failChance: f32) {
    if random() < failChance {
        return;
    }

    if failChance >= 0.0_f32 {
        //a negative failChance makes it always talk
        if !(*(*self_).NPC).group.is_null() {
            //group AI speech debounce timer
            if (*(*(*self_).NPC).group).speechDebounceTime
                > (*core::ptr::addr_of!(level)).time
            {
                return;
            }
            /*
            else if ( !self->NPC->group->enemy )
            {
                if ( groupSpeechDebounceTime[self->client->playerTeam] > level.time )
                {
                    return;
                }
            }
            */
        } else if TIMER_Done(self_, b"chatter\0".as_ptr() as *const c_char) == qfalse {
            //personal timer
            return;
        } else if *core::ptr::addr_of!(
            groupSpeechDebounceTime[(*(*self_).client).playerTeam as usize]
        ) > (*core::ptr::addr_of!(level)).time
        {
            //for those not in group AI
            //FIXME: let certain speech types interrupt others?  Let closer NPCs interrupt farther away ones?
            return;
        }
    }

    if !(*(*self_).NPC).group.is_null() {
        //So they don't all speak at once...
        //FIXME: if they're not yet mad, they have no group, so distracting a group of them makes them all speak!
        (*(*(*self_).NPC).group).speechDebounceTime =
            (*core::ptr::addr_of!(level)).time + Q_irand(2000, 4000);
    } else {
        TIMER_Set(
            self_,
            b"chatter\0".as_ptr() as *const c_char,
            Q_irand(2000, 4000),
        );
    }
    *core::ptr::addr_of_mut!(
        groupSpeechDebounceTime[(*(*self_).client).playerTeam as usize]
    ) = (*core::ptr::addr_of!(level)).time + Q_irand(2000, 4000);

    if (*(*self_).NPC).blockedSpeechDebounceTime > (*core::ptr::addr_of!(level)).time {
        return;
    }

    match speechType {
        SPEECH_CHASE => {
            G_AddVoiceEvent(self_, Q_irand(EV_CHASE1, EV_CHASE3), 2000);
        }
        SPEECH_CONFUSED => {
            G_AddVoiceEvent(self_, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000);
        }
        SPEECH_COVER => {
            G_AddVoiceEvent(self_, Q_irand(EV_COVER1, EV_COVER5), 2000);
        }
        SPEECH_DETECTED => {
            G_AddVoiceEvent(self_, Q_irand(EV_DETECTED1, EV_DETECTED5), 2000);
        }
        SPEECH_GIVEUP => {
            G_AddVoiceEvent(self_, Q_irand(EV_GIVEUP1, EV_GIVEUP4), 2000);
        }
        SPEECH_LOOK => {
            G_AddVoiceEvent(self_, Q_irand(EV_LOOK1, EV_LOOK2), 2000);
        }
        SPEECH_LOST => {
            G_AddVoiceEvent(self_, EV_LOST1, 2000);
        }
        SPEECH_OUTFLANK => {
            G_AddVoiceEvent(self_, Q_irand(EV_OUTFLANK1, EV_OUTFLANK2), 2000);
        }
        SPEECH_ESCAPING => {
            G_AddVoiceEvent(self_, Q_irand(EV_ESCAPING1, EV_ESCAPING3), 2000);
        }
        SPEECH_SIGHT => {
            G_AddVoiceEvent(self_, Q_irand(EV_SIGHT1, EV_SIGHT3), 2000);
        }
        SPEECH_SOUND => {
            G_AddVoiceEvent(self_, Q_irand(EV_SOUND1, EV_SOUND3), 2000);
        }
        SPEECH_SUSPICIOUS => {
            G_AddVoiceEvent(self_, Q_irand(EV_SUSPICIOUS1, EV_SUSPICIOUS5), 2000);
        }
        SPEECH_YELL => {
            G_AddVoiceEvent(self_, Q_irand(EV_ANGER1, EV_ANGER3), 2000);
        }
        SPEECH_PUSHED => {
            G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1, EV_PUSHED3), 2000);
        }
        _ => {}
    }

    (*(*self_).NPC).blockedSpeechDebounceTime = (*core::ptr::addr_of!(level)).time + 2000;
}

pub unsafe fn ST_MarkToCover(self_: *mut gentity_t) {
    if self_.is_null() || (*self_).NPC.is_null() {
        return;
    }
    (*(*self_).NPC).localState = LSTATE_UNDERFIRE;
    TIMER_Set(
        self_,
        b"attackDelay\0".as_ptr() as *const c_char,
        Q_irand(500, 2500),
    );
    ST_AggressionAdjust(self_, -3);
    if !(*(*self_).NPC).group.is_null() && (*(*(*self_).NPC).group).numGroup > 1 {
        ST_Speech(self_, SPEECH_COVER, 0.0_f32); //FIXME: flee sound?
    }
}

pub unsafe fn ST_StartFlee(
    self_: *mut gentity_t,
    enemy: *mut gentity_t,
    dangerPoint: vec3_t,
    dangerLevel: c_int,
    minTime: c_int,
    maxTime: c_int,
) {
    if self_.is_null() || (*self_).NPC.is_null() {
        return;
    }
    G_StartFlee(self_, enemy, dangerPoint, dangerLevel, minTime, maxTime);
    if !(*(*self_).NPC).group.is_null() && (*(*(*self_).NPC).group).numGroup > 1 {
        ST_Speech(self_, SPEECH_COVER, 0.0_f32); //FIXME: flee sound?
    }
}

/*
-------------------------
NPC_ST_Pain
-------------------------
*/

pub unsafe fn NPC_ST_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: vec3_t,
    damage: c_int,
    mod_: c_int, // C param name: mod (Rust keyword, escaped to mod_)
    hitLoc: c_int,
) {
    (*(*self_).NPC).localState = LSTATE_UNDERFIRE;

    TIMER_Set(self_, b"duck\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"hideTime\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"stand\0".as_ptr() as *const c_char, 2000);

    NPC_Pain(self_, inflictor, other, point, damage, mod_, hitLoc);

    if damage == 0 && (*self_).health > 0 {
        //FIXME: better way to know I was pushed
        G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1, EV_PUSHED3), 2000);
    }
}

/*
-------------------------
ST_HoldPosition
-------------------------
*/

unsafe fn ST_HoldPosition() {
    if (*NPCInfo).squadState == SQUAD_RETREAT {
        TIMER_Set(
            NPC,
            b"flee\0".as_ptr() as *const c_char,
            -(*core::ptr::addr_of!(level)).time,
        );
    }
    TIMER_Set(
        NPC,
        b"verifyCP\0".as_ptr() as *const c_char,
        Q_irand(1000, 3000),
    ); //don't look for another one for a few seconds
    NPC_FreeCombatPoint((*NPCInfo).combatPoint, qtrue);
    //NPCInfo->combatPoint = -1;//???
    if Q3_TaskIDPending(NPC, TID_MOVE_NAV) == qfalse {
        //don't have a script waiting for me to get to my point, okay to stop trying and stand
        AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, SQUAD_STAND_AND_SHOOT);
        (*NPCInfo).goalEntity = core::ptr::null_mut();
    }
}

pub unsafe fn NPC_ST_SayMovementSpeech() {
    if (*NPCInfo).movementSpeech == 0 {
        return;
    }
    if !(*NPCInfo).group.is_null()
        && !(*(*NPCInfo).group).commander.is_null()
        && !(*(*(*NPCInfo).group).commander).client.is_null()
        && (*(*(*(*NPCInfo).group).commander).client).NPC_class == class_t::CLASS_IMPERIAL
        && Q_irand(0, 3) == 0
    {
        //imperial (commander) gives the order
        ST_Speech(
            (*(*NPCInfo).group).commander,
            (*NPCInfo).movementSpeech,
            (*NPCInfo).movementSpeechChance,
        );
    } else {
        //really don't want to say this unless we can actually get there...
        ST_Speech(NPC, (*NPCInfo).movementSpeech, (*NPCInfo).movementSpeechChance);
    }

    (*NPCInfo).movementSpeech = 0;
    (*NPCInfo).movementSpeechChance = 0.0_f32;
}

pub unsafe fn NPC_ST_StoreMovementSpeech(speech: c_int, chance: f32) {
    (*NPCInfo).movementSpeech = speech;
    (*NPCInfo).movementSpeechChance = chance;
}

/*
-------------------------
ST_Move
-------------------------
*/
// void ST_TransferMoveGoal( gentity_t *self, gentity_t *other );
// (forward declaration — defined in a later chunk of this file)
unsafe fn ST_Move() -> qboolean {
    (*NPCInfo).combatMove = qtrue; //always move straight toward our goal

    let moved: qboolean = NPC_MoveToGoal(qtrue);
    if moved == qfalse {
        ST_HoldPosition();
    }

    NPC_ST_SayMovementSpeech();

    moved
}
/*
-------------------------
NPC_ST_SleepShuffle
-------------------------
*/

unsafe fn NPC_ST_SleepShuffle() {
    //Play an awake script if we have one
    if G_ActivateBehavior( NPC, BSET_AWAKE) != qfalse {
        return;
    }

    //Automate some movement and noise
    if TIMER_Done( NPC, b"shuffleTime\0".as_ptr() as *const core::ffi::c_char ) != 0 {

        //TODO: Play sleeping shuffle animation

        //int	soundIndex = Q_irand( 0, 1 );

        /*
        switch ( soundIndex )
        {
        case 0:
            G_Sound( NPC, G_SoundIndex("sound/chars/imperialsleeper1/scav4/hunh.mp3") );
            break;

        case 1:
            G_Sound( NPC, G_SoundIndex("sound/chars/imperialsleeper3/scav4/tryingtosleep.wav") );
            break;
        }
        */

        TIMER_Set( NPC, b"shuffleTime\0".as_ptr() as *const core::ffi::c_char, 4000 );
        TIMER_Set( NPC, b"sleepTime\0".as_ptr() as *const core::ffi::c_char, 2000 );
        return;
    }

    //They made another noise while we were stirring, see if we can see them
    if TIMER_Done( NPC, b"sleepTime\0".as_ptr() as *const core::ffi::c_char ) != 0 {
        NPC_CheckPlayerTeamStealth();
        TIMER_Set( NPC, b"sleepTime\0".as_ptr() as *const core::ffi::c_char, 2000 );
    }
}

/*
-------------------------
NPC_ST_Sleep
-------------------------
*/

pub unsafe fn NPC_BSST_Sleep() {
    let alertEvent: c_int = NPC_CheckAlertEvents( qfalse, qtrue );//only check sounds since we're alseep!

    //There is an event we heard
    if alertEvent >= 0 {
        //See if it was enough to wake us up
        if (*level).alertEvents[alertEvent as usize].level == AEL_DISCOVERED
            && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
        {
            let ent_0: *mut gentity_t = core::ptr::addr_of_mut!(g_entities[0]);
            if !ent_0.is_null() && (*ent_0).health > 0 {
                G_SetEnemy( NPC, ent_0 );
                return;
            }
        }

        //Otherwise just stir a bit
        NPC_ST_SleepShuffle();
        return;
    }
}

/*
-------------------------
NPC_CheckEnemyStealth
-------------------------
*/

pub unsafe fn NPC_CheckEnemyStealth( target: *mut gentity_t ) -> qboolean {
    let mut target_dist: f32;
    let mut minDist: f32 = 40.0;//any closer than 40 and we definitely notice

    //In case we aquired one some other way
    if !(*NPC).enemy.is_null() {
        return qtrue;
    }

    //Ignore notarget
    if ((*target).flags & FL_NOTARGET) != 0 {
        return qfalse;
    }

    if (*target).health <= 0 {
        return qfalse;
    }

    if (*(*target).client).ps.weapon == WP_SABER
        && (*(*target).client).ps.SaberActive() != qfalse
        && (*(*target).client).ps.saberInFlight == qfalse
    {//if target has saber in hand and activated, we wake up even sooner even if not facing him
        minDist = 100.0;
    }

    target_dist = DistanceSquared(
        core::ptr::addr_of!((*target).currentOrigin) as *const vec_t,
        core::ptr::addr_of!((*NPC).currentOrigin) as *const vec_t,
    );
    //If the target is this close, then wake up regardless
    if ((*(*target).client).ps.pm_flags & PMF_DUCKED) == 0 //not ducking
        && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 //looking for enemies
        && target_dist < (minDist * minDist) //closer than minDist
    {
        G_SetEnemy( NPC, target );
        (*NPCInfo).enemyLastSeenTime = (*level).time;
        TIMER_Set( NPC, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand( 500, 2500 ) );
        return qtrue;
    }

    let mut maxViewDist: f32 = MAX_VIEW_DIST as f32;

//	if ( NPCInfo->stats.visrange > maxViewDist )
    {//FIXME: should we always just set maxViewDist to this?
        maxViewDist = (*NPCInfo).stats.visrange;
    }

    if target_dist > (maxViewDist * maxViewDist) {//out of possible visRange
        return qfalse;
    }

    //Check FOV first
    if InFOV2( target, NPC, (*NPCInfo).stats.hfov, (*NPCInfo).stats.vfov ) == qfalse {
        return qfalse;
    }

    let clearLOS: qboolean = if (*(*target).client).ps.leanofs != 0 {
        NPC_ClearLOS5( core::ptr::addr_of!((*(*target).client).renderInfo.eyePoint) as *const vec_t )
    } else {
        NPC_ClearLOS4( target )
    };

    //Now check for clear line of vision
    if clearLOS != qfalse {
        if (*(*target).client).NPC_class == CLASS_ATST {//can't miss 'em!
            G_SetEnemy( NPC, target );
            TIMER_Set( NPC, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand( 500, 2500 ) );
            return qtrue;
        }
        let targ_org: vec3_t = [
            (*target).currentOrigin[0],
            (*target).currentOrigin[1],
            (*target).currentOrigin[2] + (*target).maxs[2] - 4.0,
        ];
        let mut hAngle_perc: f32 = NPC_GetHFOVPercentage(
            targ_org.as_ptr() as *const vec_t,
            core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const vec_t,
            core::ptr::addr_of!((*(*NPC).client).renderInfo.eyeAngles) as *const vec_t,
            (*NPCInfo).stats.hfov as f32,
        );
        let mut vAngle_perc: f32 = NPC_GetVFOVPercentage(
            targ_org.as_ptr() as *const vec_t,
            core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const vec_t,
            core::ptr::addr_of!((*(*NPC).client).renderInfo.eyeAngles) as *const vec_t,
            (*NPCInfo).stats.vfov as f32,
        );

        //Scale them vertically some, and horizontally pretty harshly
        vAngle_perc *= vAngle_perc;//( vAngle_perc * vAngle_perc );
        hAngle_perc *= hAngle_perc * hAngle_perc;

        //Cap our vertical vision severely
        //if ( vAngle_perc <= 0.3f ) // was 0.5f
        //	return qfalse;

        //Assess the player's current status
        target_dist = Distance(
            core::ptr::addr_of!((*target).currentOrigin) as *const vec_t,
            core::ptr::addr_of!((*NPC).currentOrigin) as *const vec_t,
        );

        let target_speed: f32 = VectorLength(
            core::ptr::addr_of!((*(*target).client).ps.velocity) as *const vec_t,
        );
        let target_crouching: c_int = ((*(*target).client).usercmd.upmove < 0) as c_int;
        let dist_rating: f32 = target_dist / maxViewDist;
        let mut speed_rating: f32 = target_speed / MAX_VIEW_SPEED as f32;
        let turning_rating: f32 = AngleDelta(
            (*(*target).client).ps.viewangles[PITCH as usize],
            (*target).lastAngles[PITCH as usize],
        ) / 180.0_f32
            + AngleDelta(
                (*(*target).client).ps.viewangles[YAW as usize],
                (*target).lastAngles[YAW as usize],
            ) / 180.0_f32;
        let light_level: f32 = (*target).lightLevel / MAX_LIGHT_INTENSITY as f32;
        let FOV_perc: f32 = 1.0_f32 - (hAngle_perc + vAngle_perc) * 0.5_f32; //FIXME: Dunno about the average...
        let mut vis_rating: f32 = 0.0_f32;

        //Too dark
        if light_level < MIN_LIGHT_THRESHOLD {
            return qfalse;
        }

        //Too close?
        if dist_rating < DISTANCE_THRESHOLD {
            G_SetEnemy( NPC, target );
            TIMER_Set( NPC, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand( 500, 2500 ) );
            return qtrue;
        }

        //Out of range
        if dist_rating > 1.0_f32 {
            return qfalse;
        }

        //Cap our speed checks
        if speed_rating > 1.0_f32 {
            speed_rating = 1.0_f32;
        }


        //Calculate the distance, fov and light influences
        //...Visibilty linearly wanes over distance
        let dist_influence: f32 = DISTANCE_SCALE * (1.0_f32 - dist_rating);
        //...As the percentage out of the FOV increases, straight perception suffers on an exponential scale
        let fov_influence: f32 = FOV_SCALE * (1.0_f32 - FOV_perc);
        //...Lack of light hides, abundance of light exposes
        let light_influence: f32 = (light_level - 0.5_f32) * LIGHT_SCALE;

        //Calculate our base rating
        let mut target_rating: f32 = dist_influence + fov_influence + light_influence;

        //Now award any final bonuses to this number
        let contents: c_int = gi_pointcontents(
            targ_org.as_ptr() as *const [f32; 3],
            (*target).s.number,
        );
        if (contents & CONTENTS_WATER as c_int) != 0 {
            let myContents: c_int = gi_pointcontents(
                core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const [f32; 3],
                (*NPC).s.number,
            );
            if (myContents & CONTENTS_WATER as c_int) == 0 {//I'm not in water
                if (*(*NPC).client).NPC_class == CLASS_SWAMPTROOPER {//these guys can see in in/through water pretty well
                    vis_rating = 0.10_f32;//10% bonus
                } else {
                    vis_rating = 0.35_f32;//35% bonus
                }
            } else {//else, if we're both in water
                if (*(*NPC).client).NPC_class == CLASS_SWAMPTROOPER {//I can see him just fine
                } else {
                    vis_rating = 0.15_f32;//15% bonus
                }
            }
        } else {//not in water
            if (contents & CONTENTS_FOG as c_int) != 0 {
                vis_rating = 0.15_f32;//15% bonus
            }
        }

        target_rating *= 1.0_f32 - vis_rating;

        //...Motion draws the eye quickly
        target_rating += speed_rating * SPEED_SCALE;
        target_rating += turning_rating * TURNING_SCALE;
        //FIXME: check to see if they're animating, too?  But can we do something as simple as frame != oldframe?

        //...Smaller targets are harder to indentify
        if target_crouching != 0 {
            target_rating *= 0.9_f32; //10% bonus
        }

        //If he's violated the threshold, then realize him
        //float difficulty_scale = 1.0f + (2.0f-g_spskill->value);//if playing on easy, 20% harder to be seen...?
        let realize: f32;
        let cautious: f32;
        if (*(*NPC).client).NPC_class == CLASS_SWAMPTROOPER {//swamptroopers can see much better
            realize = CAUTIOUS_THRESHOLD as f32;/* difficulty_scale*/
            cautious = CAUTIOUS_THRESHOLD as f32 * 0.75_f32;/* difficulty_scale*/
        } else {
            realize = REALIZE_THRESHOLD as f32;/* difficulty_scale*/
            cautious = CAUTIOUS_THRESHOLD as f32 * 0.75_f32;/* difficulty_scale*/
        }

        if target_rating > realize && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            G_SetEnemy( NPC, target );
            (*NPCInfo).enemyLastSeenTime = (*level).time;
            TIMER_Set( NPC, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand( 500, 2500 ) );
            return qtrue;
        }

        //If he's above the caution threshold, then realize him in a few seconds unless he moves to cover
        if target_rating > cautious && ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {//FIXME: ambushing guys should never talk
            if TIMER_Done( NPC, b"enemyLastVisible\0".as_ptr() as *const core::ffi::c_char ) != 0 {//If we haven't already, start the counter
                let lookTime: c_int = Q_irand( 4500, 8500 );
                //NPCInfo->timeEnemyLastVisible = level.time + 2000;
                TIMER_Set( NPC, b"enemyLastVisible\0".as_ptr() as *const core::ffi::c_char, lookTime );
                //TODO: Play a sound along the lines of, "Huh?  What was that?"
                ST_Speech( NPC, SPEECH_SIGHT, 0 );
                NPC_TempLookTarget( NPC, (*target).s.number, lookTime, lookTime );
                //FIXME: set desired yaw and pitch towards this guy?
            } else if TIMER_Get( NPC, b"enemyLastVisible\0".as_ptr() as *const core::ffi::c_char ) <= (*level).time + 500
                && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 //FIXME: Is this reliable?
            {
                if (*NPCInfo).rank < RANK_LT && Q_irand( 0, 2 ) == 0 {
                    let interrogateTime: c_int = Q_irand( 2000, 4000 );
                    ST_Speech( NPC, SPEECH_SUSPICIOUS, 0 );
                    TIMER_Set( NPC, b"interrogating\0".as_ptr() as *const core::ffi::c_char, interrogateTime );
                    G_SetEnemy( NPC, target );
                    (*NPCInfo).enemyLastSeenTime = (*level).time;
                    TIMER_Set( NPC, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, interrogateTime );
                    TIMER_Set( NPC, b"stand\0".as_ptr() as *const core::ffi::c_char, interrogateTime );
                } else {
                    G_SetEnemy( NPC, target );
                    (*NPCInfo).enemyLastSeenTime = (*level).time;
                    //FIXME: ambush guys (like those popping out of water) shouldn't delay...
                    TIMER_Set( NPC, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand( 500, 2500 ) );
                    TIMER_Set( NPC, b"stand\0".as_ptr() as *const core::ffi::c_char, Q_irand( 500, 2500 ) );
                }
                return qtrue;
            }

            return qfalse;
        }
    }

    return qfalse;
}

pub unsafe fn NPC_CheckPlayerTeamStealth() -> qboolean {
    /*
    //NOTENOTE: For now, all stealh checks go against the player, since
    //			he is the main focus.  Squad members and rivals do not
    //			fall into this category and will be ignored.

    NPC_CheckEnemyStealth( &g_entities[0] );	//Change this pointer to assess other entities
    */
    let mut i: c_int = 0;
    while i < ENTITYNUM_WORLD {
        if PInUse(i) == qfalse {
            i += 1;
            continue;
        }
        let enemy: *mut gentity_t = core::ptr::addr_of_mut!(g_entities[i as usize]);
        if !enemy.is_null()
            && !(*enemy).client.is_null()
            && NPC_ValidEnemy( enemy ) != qfalse
        {
            if NPC_CheckEnemyStealth( enemy ) != qfalse { //Change this pointer to assess other entities
                return qtrue;
            }
        }
        i += 1;
    }
    return qfalse;
}

pub unsafe fn NPC_CheckEnemiesInSpotlight() -> qboolean {
    let mut entityList: [*mut gentity_t; MAX_GENTITIES] = [core::ptr::null_mut(); MAX_GENTITIES];
    let mut enemy: *mut gentity_t;
    let mut suspect: *mut gentity_t = core::ptr::null_mut();
    let mut i: c_int;
    let mut numListedEntities: c_int;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    i = 0;
    while i < 3 {
        mins[i as usize] = (*(*NPC).client).renderInfo.eyePoint[i as usize] - (*NPC).speed;
        maxs[i as usize] = (*(*NPC).client).renderInfo.eyePoint[i as usize] + (*NPC).speed;
        i += 1;
    }

    numListedEntities = gi_EntitiesInBox( core::ptr::addr_of!(mins), core::ptr::addr_of!(maxs), entityList.as_mut_ptr(), MAX_GENTITIES as c_int );

    i = 0;
    while i < numListedEntities {
        if PInUse(i) == qfalse {
            i += 1;
            continue;
        }

        enemy = entityList[i as usize];

        if !enemy.is_null()
            && !(*enemy).client.is_null()
            && NPC_ValidEnemy( enemy ) != qfalse
            && (*(*enemy).client).playerTeam == (*(*NPC).client).enemyTeam
        {//valid ent & client, valid enemy, on the target team
            //check to see if they're in my FOV
            if InFOV3(
                (*enemy).currentOrigin.as_ptr() as *const vec_t,
                core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const vec_t,
                core::ptr::addr_of!((*(*NPC).client).renderInfo.eyeAngles) as *const vec_t,
                (*NPCInfo).stats.hfov,
                (*NPCInfo).stats.vfov,
            ) != qfalse {//in my cone
                //check to see that they're close enough
                if DistanceSquared(
                    core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const vec_t,
                    (*enemy).currentOrigin.as_ptr() as *const vec_t,
                ) - 256.0/*fudge factor: 16 squared*/ <= (*NPC).speed * (*NPC).speed {//within range
                    //check to see if we have a clear trace to them
                    if NPC_ClearLOS4( enemy ) != qfalse {//clear LOS
                        //make sure their light level is at least my beam's brightness
                        //FIXME: HOW?
                        //enemy->lightLevel / MAX_LIGHT_INTENSITY

                        //good enough, take him!
                        //FIXME: pick closest one?
                        //FIXME: have the graduated noticing like other NPCs? (based on distance, FOV dot, etc...)
                        G_SetEnemy( NPC, enemy );
                        TIMER_Set( NPC, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand( 500, 2500 ) );
                        return qtrue;
                    }
                }
            }
            if InFOV3(
                (*enemy).currentOrigin.as_ptr() as *const vec_t,
                core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const vec_t,
                core::ptr::addr_of!((*(*NPC).client).renderInfo.eyeAngles) as *const vec_t,
                90,
                (*NPCInfo).stats.vfov * 3,
            ) != qfalse {//one to look at if we don't get an enemy
                if NPC_ClearLOS4( enemy ) != qfalse {//clear LOS
                    if suspect.is_null()
                        || DistanceSquared(
                            core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const vec_t,
                            (*enemy).currentOrigin.as_ptr() as *const vec_t,
                        ) < DistanceSquared(
                            core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const vec_t,
                            (*suspect).currentOrigin.as_ptr() as *const vec_t,
                        )
                    {//remember him
                        suspect = enemy;
                    }
                }
            }
        }
        i += 1;
    }
    if !suspect.is_null()
        && Q_flrand(
            0.0,
            (*NPCInfo).stats.visrange * (*NPCInfo).stats.visrange,
        ) > DistanceSquared(
            core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const vec_t,
            (*suspect).currentOrigin.as_ptr() as *const vec_t,
        )
    {//hey!  who's that?
        if TIMER_Done( NPC, b"enemyLastVisible\0".as_ptr() as *const core::ffi::c_char ) != 0 {//If we haven't already, start the counter
            let lookTime: c_int = Q_irand( 4500, 8500 );
            //NPCInfo->timeEnemyLastVisible = level.time + 2000;
            TIMER_Set( NPC, b"enemyLastVisible\0".as_ptr() as *const core::ffi::c_char, lookTime );
            //TODO: Play a sound along the lines of, "Huh?  What was that?"
            ST_Speech( NPC, SPEECH_SIGHT, 0 );
            //set desired yaw and pitch towards this guy?
            //FIXME: this is permanent, they will never look away... *sigh*
            NPC_FacePosition( (*suspect).currentOrigin.as_mut_ptr() );
            //FIXME: they still need some sort of eye/head tag/bone that can turn?
            //NPC_TempLookTarget( NPC, suspect->s.number, lookTime, lookTime );
        } else if TIMER_Get( NPC, b"enemyLastVisible\0".as_ptr() as *const core::ffi::c_char ) <= (*level).time + 500
            && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 //FIXME: Is this reliable?
        {
            if Q_irand( 0, 2 ) == 0 {
                let interrogateTime: c_int = Q_irand( 2000, 4000 );
                ST_Speech( NPC, SPEECH_SUSPICIOUS, 0 );
                TIMER_Set( NPC, b"interrogating\0".as_ptr() as *const core::ffi::c_char, interrogateTime );
                //G_SetEnemy( NPC, target );
                //NPCInfo->enemyLastSeenTime = level.time;
                //TIMER_Set( NPC, "attackDelay", interrogateTime );
                //TIMER_Set( NPC, "stand", interrogateTime );
                //set desired yaw and pitch towards this guy?
                //FIXME: this is permanent, they will never look away... *sigh*
                NPC_FacePosition( (*suspect).currentOrigin.as_mut_ptr() );
                //FIXME: they still need some sort of eye/head tag/bone that can turn?
                //NPC_TempLookTarget( NPC, suspect->s.number, interrogateTime, interrogateTime );
            }
        }
    }
    return qfalse;
}
/*
-------------------------
NPC_ST_InvestigateEvent
-------------------------
*/

pub const MAX_CHECK_THRESHOLD: i32 = 1;

#[allow(non_snake_case)]
unsafe fn NPC_ST_InvestigateEvent(eventID: core::ffi::c_int, extraSuspicious: bool) -> qboolean {
    //If they've given themselves away, just take them as an enemy
    if (*NPCInfo).confusionTime < (*level).time {
        if (*level).alertEvents[eventID as usize].level == AEL_DISCOVERED
            && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
        {
            //NPCInfo->lastAlertID = level.alertEvents[eventID].ID;
            if (*level).alertEvents[eventID as usize].owner.is_null()
                || (*(*level).alertEvents[eventID as usize].owner).client.is_null()
                || (*(*level).alertEvents[eventID as usize].owner).health <= 0
                || (*(*(*level).alertEvents[eventID as usize].owner).client).playerTeam
                    != (*(*NPC).client).enemyTeam
            {
                //not an enemy
                return qfalse;
            }
            //FIXME: what if can't actually see enemy, don't know where he is... should we make them just become very alert and start looking for him?  Or just let combat AI handle this... (act as if you lost him)
            //ST_Speech( NPC, SPEECH_CHARGE, 0 );
            G_SetEnemy(NPC, (*level).alertEvents[eventID as usize].owner);
            (*NPCInfo).enemyLastSeenTime = (*level).time;
            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const core::ffi::c_char, Q_irand(500, 2500));
            if (*level).alertEvents[eventID as usize].r#type == AET_SOUND {
                //heard him, didn't see him, stick for a bit
                TIMER_Set(NPC, b"roamTime\0".as_ptr() as *const core::ffi::c_char, Q_irand(500, 2500));
            }
            return qtrue;
        }
    }

    //don't look at the same alert twice
    /*
    if ( level.alertEvents[eventID].ID == NPCInfo->lastAlertID )
    {
        return qfalse;
    }
    NPCInfo->lastAlertID = level.alertEvents[eventID].ID;
    */

    //Must be ready to take another sound event
    /*
    if ( NPCInfo->investigateSoundDebounceTime > level.time )
    {
        return qfalse;
    }
    */

    if (*level).alertEvents[eventID as usize].r#type == AET_SIGHT {
        //sight alert, check the light level
        if (*level).alertEvents[eventID as usize].light
            < Q_irand(ST_MIN_LIGHT_THRESHOLD, ST_MAX_LIGHT_THRESHOLD)
        {
            //below my threshhold of potentially seeing
            return qfalse;
        }
    }

    //Save the position for movement (if necessary)
    VectorCopy(
        (*level).alertEvents[eventID as usize].position.as_ptr(),
        (*NPCInfo).investigateGoal.as_mut_ptr(),
    );

    //First awareness of it
    (*NPCInfo).investigateCount += if extraSuspicious { 2 } else { 1 };

    //Clamp the value
    if (*NPCInfo).investigateCount > 4 {
        (*NPCInfo).investigateCount = 4;
    }

    //See if we should walk over and investigate
    if (*level).alertEvents[eventID as usize].level > AEL_MINOR
        && (*NPCInfo).investigateCount > 1
        && ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0
    {
        //make it so they can walk right to this point and look at it rather than having to use combatPoints
        if G_ExpandPointToBBox(
            (*NPCInfo).investigateGoal.as_mut_ptr(),
            (*NPC).mins.as_ptr(),
            (*NPC).maxs.as_ptr(),
            (*NPC).s.number,
            (((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP),
        ) != qfalse
        {
            //we were able to move the investigateGoal to a point in which our bbox would fit
            //drop the goal to the ground so we can get at it
            let mut end: vec3_t = [0.0; 3];
            let mut trace: trace_t = core::mem::zeroed();
            VectorCopy((*NPCInfo).investigateGoal.as_ptr(), end.as_mut_ptr());
            end[2] -= 512.0; //FIXME: not always right?  What if it's even higher, somehow?
            gi_trace(
                &mut trace,
                (*NPCInfo).investigateGoal.as_ptr(),
                (*NPC).mins.as_ptr(),
                (*NPC).maxs.as_ptr(),
                end.as_ptr(),
                ENTITYNUM_NONE,
                (((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP),
            );
            if trace.fraction >= 1.0_f32 {
                //too high to even bother
                //FIXME: look at them???
            } else {
                VectorCopy(trace.endpos.as_ptr(), (*NPCInfo).investigateGoal.as_mut_ptr());
                NPC_SetMoveGoal(NPC, (*NPCInfo).investigateGoal.as_mut_ptr(), 16, qtrue);
                (*NPCInfo).localState = LSTATE_INVESTIGATE;
            }
        } else {
            let id = NPC_FindCombatPoint(
                (*NPCInfo).investigateGoal.as_mut_ptr(),
                (*NPCInfo).investigateGoal.as_mut_ptr(),
                (*NPCInfo).investigateGoal.as_mut_ptr(),
                CP_INVESTIGATE | CP_HAS_ROUTE,
                0,
            );

            if id != -1 {
                NPC_SetMoveGoal(
                    NPC,
                    (*level).combatPoints[id as usize].origin.as_mut_ptr(),
                    16,
                    qtrue,
                    id,
                );
                (*NPCInfo).localState = LSTATE_INVESTIGATE;
            }
        }
        //Say something
        //FIXME: only if have others in group... these should be responses?
        if (*NPCInfo).investigateDebounceTime + (*NPCInfo).pauseTime > (*level).time {
            //was already investigating
            if !(*NPCInfo).group.is_null()
                && !(*(*NPCInfo).group).commander.is_null()
                && !(*(*(*NPCInfo).group).commander).client.is_null()
                && (*(*(*(*NPCInfo).group).commander).client).NPC_class == CLASS_IMPERIAL
                && Q_irand(0, 3) == 0
            {
                ST_Speech((*(*NPCInfo).group).commander, SPEECH_LOOK, 0); //FIXME: "I'll go check it out" type sounds
            } else {
                ST_Speech(NPC, SPEECH_LOOK, 0); //FIXME: "I'll go check it out" type sounds
            }
        } else {
            if (*level).alertEvents[eventID as usize].r#type == AET_SIGHT {
                ST_Speech(NPC, SPEECH_SIGHT, 0);
            } else if (*level).alertEvents[eventID as usize].r#type == AET_SOUND {
                ST_Speech(NPC, SPEECH_SOUND, 0);
            }
        }
        //Setup the debounce info
        (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 5000;
        (*NPCInfo).investigateSoundDebounceTime = (*level).time + 2000;
        (*NPCInfo).pauseTime = (*level).time;
    } else {
        //just look?
        //Say something
        if (*level).alertEvents[eventID as usize].r#type == AET_SIGHT {
            ST_Speech(NPC, SPEECH_SIGHT, 0);
        } else if (*level).alertEvents[eventID as usize].r#type == AET_SOUND {
            ST_Speech(NPC, SPEECH_SOUND, 0);
        }
        //Setup the debounce info
        (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 1000;
        (*NPCInfo).investigateSoundDebounceTime = (*level).time + 1000;
        (*NPCInfo).pauseTime = (*level).time;
        VectorCopy(
            (*level).alertEvents[eventID as usize].position.as_ptr(),
            (*NPCInfo).investigateGoal.as_mut_ptr(),
        );
        if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER && RT_Flying(NPC) == qfalse {
            //if ( !Q_irand( 0, 2 ) )
            {
                //look around
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_GUARD_LOOKAROUND1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            }
        }
    }

    if (*level).alertEvents[eventID as usize].level >= AEL_DANGER {
        (*NPCInfo).investigateDebounceTime = Q_irand(500, 2500);
    }

    //Start investigating
    (*NPCInfo).tempBehavior = BS_INVESTIGATE;
    qtrue
}

/*
-------------------------
ST_OffsetLook
-------------------------
*/

#[allow(non_snake_case)]
unsafe fn ST_OffsetLook(offset: f32, out: *mut vec3_t) {
    let mut angles: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut temp: vec3_t = [0.0; 3];

    GetAnglesForDirection(
        (*NPC).currentOrigin.as_ptr(),
        (*NPCInfo).investigateGoal.as_ptr(),
        angles.as_mut_ptr(),
    );
    angles[YAW as usize] += offset;
    AngleVectors(
        angles.as_ptr(),
        forward.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    VectorMA((*NPC).currentOrigin.as_ptr(), 64.0, forward.as_ptr(), (*out).as_mut_ptr());

    CalcEntitySpot(NPC, SPOT_HEAD, temp.as_mut_ptr());
    (*out)[2] = temp[2];
}

/*
-------------------------
ST_LookAround
-------------------------
*/

#[allow(non_snake_case)]
unsafe fn ST_LookAround() {
    let mut lookPos: vec3_t = [0.0; 3];
    let perc: f32 = ((*level).time - (*NPCInfo).pauseTime) as f32
        / (*NPCInfo).investigateDebounceTime as f32;

    //Keep looking at the spot
    if perc < 0.25 {
        VectorCopy((*NPCInfo).investigateGoal.as_ptr(), lookPos.as_mut_ptr());
    } else if perc < 0.5_f32 {
        //Look up but straight ahead
        ST_OffsetLook(0.0_f32, &mut lookPos);
    } else if perc < 0.75_f32 {
        //Look right
        ST_OffsetLook(45.0_f32, &mut lookPos);
    } else {
        //Look left
        ST_OffsetLook(-45.0_f32, &mut lookPos);
    }

    NPC_FacePosition(lookPos.as_mut_ptr());
}

/*
-------------------------
NPC_BSST_Investigate
-------------------------
*/

#[allow(non_snake_case)]
pub unsafe fn NPC_BSST_Investigate() {
    //get group- mainly for group speech debouncing, but may use for group scouting/investigating AI, too
    AI_GetGroup(NPC);

    if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) != 0 {
        WeaponThink(qtrue);
    }

    if (*NPCInfo).confusionTime < (*level).time {
        if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            //Look for an enemy
            if NPC_CheckPlayerTeamStealth() != qfalse {
                //NPCInfo->behaviorState	= BS_HUNT_AND_KILL;//should be auto now
                ST_Speech(NPC, SPEECH_DETECTED, 0);
                (*NPCInfo).tempBehavior = BS_DEFAULT;
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }
    }

    if ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
        let alertEvent = NPC_CheckAlertEvents(qtrue, qtrue, (*NPCInfo).lastAlertID);

        //There is an event to look at
        if alertEvent >= 0 {
            if (*NPCInfo).confusionTime < (*level).time {
                if NPC_CheckForDanger(alertEvent) != qfalse {
                    //running like hell
                    ST_Speech(NPC, SPEECH_COVER, 0); //FIXME: flee sound?
                    return;
                }
            }

            //if ( level.alertEvents[alertEvent].ID != NPCInfo->lastAlertID )
            {
                NPC_ST_InvestigateEvent(alertEvent, true);
            }
        }
    }

    //If we're done looking, then just return to what we were doing
    if ((*NPCInfo).investigateDebounceTime + (*NPCInfo).pauseTime) < (*level).time {
        (*NPCInfo).tempBehavior = BS_DEFAULT;
        (*NPCInfo).goalEntity = UpdateGoal();

        NPC_UpdateAngles(qtrue, qtrue);
        //Say something
        ST_Speech(NPC, SPEECH_GIVEUP, 0);
        return;
    }

    //FIXME: else, look for new alerts

    //See if we're searching for the noise's origin
    if (*NPCInfo).localState == LSTATE_INVESTIGATE && !(*NPCInfo).goalEntity.is_null() {
        //See if we're there
        if STEER_Reached(NPC, (*NPCInfo).goalEntity, 32.0, (FlyingCreature(NPC) != 0) as i32) == 0
        {
            (*ucmd).buttons |= BUTTON_WALKING;

            //Try and move there
            if NPC_MoveToGoal(qtrue) != qfalse {
                //Bump our times
                (*NPCInfo).investigateDebounceTime = (*NPCInfo).investigateCount * 5000;
                (*NPCInfo).pauseTime = (*level).time;

                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }

        //Otherwise we're done or have given up
        //Say something
        //ST_Speech( NPC, SPEECH_LOOK, 0.33f );
        (*NPCInfo).localState = LSTATE_NONE;
    }

    //Look around
    ST_LookAround();
}
/*
-------------------------
NPC_BSST_Patrol
-------------------------
*/

pub unsafe fn NPC_BSST_Patrol() {
    //FIXME: pick up on bodies of dead buddies?

    //Not a scriptflag, but...
    if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
        && ((*(*NPC).client).ps.eFlags & EF_SPOTLIGHT) != 0
    {
        //using spotlight search mode
        let mut eyeFwd: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mins: vec3_t = [-2.0, -2.0, -2.0];
        let maxs: vec3_t = [2.0, 2.0, 2.0];
        let mut trace: trace_t = core::mem::zeroed();
        AngleVectors(
            core::ptr::addr_of!((*(*NPC).client).renderInfo.eyeAngles) as *const vec_t,
            eyeFwd.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        // VectorMA( NPC->client->renderInfo.eyePoint, NPCInfo->stats.visrange, eyeFwd, end );
        {
            let ep = core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint);
            let s = (*NPCInfo).stats.visrange;
            end[0] = (*ep)[0] + eyeFwd[0] * s;
            end[1] = (*ep)[1] + eyeFwd[1] * s;
            end[2] = (*ep)[2] + eyeFwd[2] * s;
        }
        //get server-side trace impact point
        gi_trace(
            core::ptr::addr_of_mut!(trace),
            core::ptr::addr_of!((*(*NPC).client).renderInfo.eyePoint) as *const vec_t,
            mins.as_ptr(),
            maxs.as_ptr(),
            end.as_ptr(),
            (*NPC).s.number,
            MASK_OPAQUE | CONTENTS_BODY | CONTENTS_CORPSE,
        );
        (*NPC).speed = trace.fraction * (*NPCInfo).stats.visrange;
        if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            //FIXME: do a FOV cone check, then a trace
            if trace.entityNum < ENTITYNUM_WORLD {
                //hit something
                //try cheap check first
                let enemy: *mut gentity_t =
                    core::ptr::addr_of_mut!(g_entities[trace.entityNum as usize]);
                if !enemy.is_null()
                    && !(*enemy).client.is_null()
                    && NPC_ValidEnemy(enemy) != qfalse
                    && (*(*enemy).client).playerTeam == (*(*NPC).client).enemyTeam
                {
                    G_SetEnemy(NPC, enemy);
                    TIMER_Set(
                        NPC,
                        b"attackDelay\0".as_ptr() as *const c_char,
                        Q_irand(500, 2500),
                    );
                    //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be auto now
                    //NPC_AngerSound();
                    NPC_UpdateAngles(qtrue, qtrue);
                    return;
                }
            }
            //FIXME: maybe do a quick check of ents within the spotlight's radius?
            //hmmm, look around
            if NPC_CheckEnemiesInSpotlight() != qfalse {
                //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be auto now
                //NPC_AngerSound();
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }
    } else {
        //get group- mainly for group speech debouncing, but may use for group scouting/investigating AI, too
        AI_GetGroup(NPC);

        if (*NPCInfo).confusionTime < (*core::ptr::addr_of!(level)).time {
            //Look for any enemies
            if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
                if NPC_CheckPlayerTeamStealth() != qfalse {
                    //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be auto now
                    //NPC_AngerSound();
                    NPC_UpdateAngles(qtrue, qtrue);
                    return;
                }
            }
        }
    }

    if ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
        let alertEvent: c_int = NPC_CheckAlertEvents(qtrue, qtrue);

        //There is an event to look at
        if alertEvent >= 0 {
            if NPC_CheckForDanger(alertEvent) != qfalse {
                //going to run?
                ST_Speech(NPC, SPEECH_COVER, 0.0);
                return;
            } else if (*(*NPC).client).NPC_class == CLASS_BOBAFETT {
                //NPCInfo->lastAlertID = level.alertEvents[eventID].ID;
                if (*core::ptr::addr_of!(level)).alertEvents[alertEvent as usize]
                    .owner
                    .is_null()
                    || (*(*core::ptr::addr_of!(level)).alertEvents[alertEvent as usize].owner)
                        .client
                        .is_null()
                    || (*(*core::ptr::addr_of!(level)).alertEvents[alertEvent as usize].owner)
                        .health
                        <= 0
                    || (*(*(*core::ptr::addr_of!(level)).alertEvents[alertEvent as usize].owner)
                        .client)
                        .playerTeam
                        != (*(*NPC).client).enemyTeam
                {
                    //not an enemy
                    return;
                }
                //FIXME: what if can't actually see enemy, don't know where he is... should we make them just become very alert and start looking for him?  Or just let combat AI handle this... (act as if you lost him)
                //ST_Speech( NPC, SPEECH_CHARGE, 0 );
                G_SetEnemy(
                    NPC,
                    (*core::ptr::addr_of!(level)).alertEvents[alertEvent as usize].owner,
                );
                (*NPCInfo).enemyLastSeenTime = (*core::ptr::addr_of!(level)).time;
                TIMER_Set(
                    NPC,
                    b"attackDelay\0".as_ptr() as *const c_char,
                    Q_irand(500, 2500),
                );
                return;
            } else if NPC_ST_InvestigateEvent(alertEvent, qfalse) != qfalse {
                //actually going to investigate it
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }
    }

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        (*core::ptr::addr_of_mut!(ucmd)).buttons |= BUTTON_WALKING;
        //ST_Move( NPCInfo->goalEntity );
        NPC_MoveToGoal(qtrue);
    } else// if ( !(NPCInfo->scriptFlags&SCF_IGNORE_ALERTS) )
    {
        if (*(*NPC).client).NPC_class != CLASS_IMPERIAL
            && (*(*NPC).client).NPC_class != CLASS_IMPWORKER
        {
            //imperials do not look around
            if TIMER_Done(NPC, b"enemyLastVisible\0".as_ptr() as *const c_char) != qfalse {
                //nothing suspicious, look around
                if Q_irand(0, 30) == 0 {
                    (*NPCInfo).desiredYaw =
                        (*NPC).s.angles[1] + Q_irand(-90, 90) as f32;
                }
                if Q_irand(0, 30) == 0 {
                    (*NPCInfo).desiredPitch = Q_irand(-20, 20) as f32;
                }
            }
        }
    }

    NPC_UpdateAngles(qtrue, qtrue);
    //TEMP hack for Imperial stand anim
    if (*(*NPC).client).NPC_class == CLASS_IMPERIAL
        || (*(*NPC).client).NPC_class == CLASS_IMPWORKER
    {
        //hack
        if (*(*NPC).client).ps.weapon != WP_CONCUSSION {
            //not Rax
            if (*core::ptr::addr_of!(ucmd)).forwardmove != 0
                || (*core::ptr::addr_of!(ucmd)).rightmove != 0
                || (*core::ptr::addr_of!(ucmd)).upmove != 0
            {
                //moving

                if (*(*NPC).client).ps.torsoAnimTimer == 0
                    || (*(*NPC).client).ps.torsoAnim == BOTH_STAND4
                {
                    if ((*core::ptr::addr_of!(ucmd)).buttons & BUTTON_WALKING) != 0
                        && ((*NPCInfo).scriptFlags & SCF_RUNNING) == 0
                    {
                        //not running, only set upper anim
                        //  No longer overrides scripted anims
                        NPC_SetAnim(
                            NPC,
                            SETANIM_TORSO,
                            BOTH_STAND4,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                        (*(*NPC).client).ps.torsoAnimTimer = 200;
                    }
                }
            } else {
                //standing still, set both torso and legs anim
                //  No longer overrides scripted anims
                if ((*(*NPC).client).ps.torsoAnimTimer == 0
                    || (*(*NPC).client).ps.torsoAnim == BOTH_STAND4)
                    && ((*(*NPC).client).ps.legsAnimTimer == 0
                        || (*(*NPC).client).ps.legsAnim == BOTH_STAND4)
                {
                    NPC_SetAnim(
                        NPC,
                        SETANIM_BOTH,
                        BOTH_STAND4,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    (*(*NPC).client).ps.torsoAnimTimer = 200;
                    (*(*NPC).client).ps.legsAnimTimer = 200;
                }
            }
            //FIXME: this is a disgusting hack that is supposed to make the Imperials start with their weapon holstered- need a better way
            if (*(*NPC).client).ps.weapon != WP_NONE {
                ChangeWeapon(NPC, WP_NONE);
                (*(*NPC).client).ps.weapon = WP_NONE;
                (*(*NPC).client).ps.weaponstate = WEAPON_READY;
                G_RemoveWeaponModels(NPC);
            }
        }
    }
}

/*
-------------------------
NPC_BSST_Idle
-------------------------
*/
/*
void NPC_BSST_Idle( void )
{
	int alertEvent = NPC_CheckAlertEvents( qtrue, qtrue );

	//There is an event to look at
	if ( alertEvent >= 0 )
	{
		NPC_ST_InvestigateEvent( alertEvent, qfalse );
		NPC_UpdateAngles( qtrue, qtrue );
		return;
	}

	TIMER_Set( NPC, "roamTime", 2000 + Q_irand( 1000, 2000 ) );

	NPC_UpdateAngles( qtrue, qtrue );
}
*/
/*
-------------------------
ST_CheckMoveState
-------------------------
*/

unsafe fn ST_CheckMoveState() {
    if Q3_TaskIDPending(NPC, TID_MOVE_NAV) != qfalse {
        //moving toward a goal that a script is waiting on, so don't stop for anything!
        r#move = qtrue;
    } else if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
        && (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE
    {
        //no squad stuff
        return;
    }
    //	else if ( NPC->NPC->scriptFlags&SCF_NO_GROUPS )
    {
        r#move = qtrue;
    }
    //See if we're a scout

    //See if we're moving towards a goal, not the enemy
    if (*NPCInfo).goalEntity != (*NPC).enemy && !(*NPCInfo).goalEntity.is_null() {
        //Did we make it?
        if STEER_Reached(
            NPC,
            (*NPCInfo).goalEntity,
            16.0,
            (FlyingCreature(NPC) != 0) as c_int,
        ) != qfalse
            || (enemyLOS != qfalse
                && ((*NPCInfo).aiFlags & NPCAI_STOP_AT_LOS) != 0
                && Q3_TaskIDPending(NPC, TID_MOVE_NAV) == qfalse)
        {
            //either hit our navgoal or our navgoal was not a crucial (scripted) one (maybe a combat point) and we're scouting and found our enemy
            let mut newSquadState: c_int = SQUAD_STAND_AND_SHOOT;
            //we got where we wanted to go, set timers based on why we were running
            match (*NPCInfo).squadState {
                SQUAD_RETREAT => {
                    //was running away
                    //done fleeing, obviously
                    TIMER_Set(
                        NPC,
                        b"duck\0".as_ptr() as *const c_char,
                        ((*NPC).max_health - (*NPC).health) * 100,
                    );
                    TIMER_Set(
                        NPC,
                        b"hideTime\0".as_ptr() as *const c_char,
                        Q_irand(3000, 7000),
                    );
                    TIMER_Set(
                        NPC,
                        b"flee\0".as_ptr() as *const c_char,
                        (*core::ptr::addr_of!(level)).time.wrapping_neg(),
                    );
                    newSquadState = SQUAD_COVER;
                }
                SQUAD_TRANSITION => {
                    //was heading for a combat point
                    TIMER_Set(
                        NPC,
                        b"hideTime\0".as_ptr() as *const c_char,
                        Q_irand(2000, 4000),
                    );
                }
                SQUAD_SCOUT => {
                    //was running after player
                }
                _ => {}
            }
            AI_GroupUpdateSquadstates((*NPCInfo).group, NPC, newSquadState);
            NPC_ReachedGoal();
            //don't attack right away
            TIMER_Set(
                NPC,
                b"attackDelay\0".as_ptr() as *const c_char,
                Q_irand(250, 500),
            ); //FIXME: Slant for difficulty levels
            //don't do something else just yet

            // THIS IS THE ONE TRUE PLACE WHERE ROAM TIME IS SET
            TIMER_Set(
                NPC,
                b"roamTime\0".as_ptr() as *const c_char,
                Q_irand(8000, 15000),
            ); //Q_irand( 1000, 4000 ) );
            if Q_irand(0, 3) == 0 {
                TIMER_Set(
                    NPC,
                    b"duck\0".as_ptr() as *const c_char,
                    Q_irand(5000, 10000),
                ); // just reached our goal, chance of ducking now
            }
            return;
        }

        //keep going, hold of roamTimer until we get there
        TIMER_Set(
            NPC,
            b"roamTime\0".as_ptr() as *const c_char,
            Q_irand(8000, 9000),
        );
    }
}
pub unsafe fn ST_ResolveBlockedShot(hit: c_int) {
    let stuckTime: c_int;
    //figure out how long we intend to stand here, max
    if TIMER_Get(NPC, c"roamTime".as_ptr()) > TIMER_Get(NPC, c"stick".as_ptr()) {
        stuckTime = TIMER_Get(NPC, c"roamTime".as_ptr()) - (*core::ptr::addr_of!(level)).time;
    } else {
        stuckTime = TIMER_Get(NPC, c"stick".as_ptr()) - (*core::ptr::addr_of!(level)).time;
    }

    if TIMER_Done(NPC, c"duck".as_ptr()) != qfalse {
        //we're not ducking
        if AI_GroupContainsEntNum((*NPCInfo).group, hit) != qfalse {
            let member: *mut gentity_t = core::ptr::addr_of_mut!(g_entities[hit as usize]);
            if TIMER_Done(member, c"duck".as_ptr()) != qfalse {
                //they aren't ducking
                if TIMER_Done(member, c"stand".as_ptr()) != qfalse {
                    //they're not being forced to stand
                    //tell them to duck at least as long as I'm not moving
                    TIMER_Set(member, c"duck".as_ptr(), stuckTime); // tell my friend to duck so I can shoot over his head
                    return;
                }
            }
        }
    } else {
        //maybe we should stand
        if TIMER_Done(NPC, c"stand".as_ptr()) != qfalse {
            //stand for as long as we'll be here
            TIMER_Set(NPC, c"stand".as_ptr(), stuckTime);
            return;
        }
    }
    //Hmm, can't resolve this by telling them to duck or telling me to stand
    //We need to move!
    TIMER_Set(NPC, c"roamTime".as_ptr(), -1);
    TIMER_Set(NPC, c"stick".as_ptr(), -1);
    TIMER_Set(NPC, c"duck".as_ptr(), -1);
    TIMER_Set(NPC, c"attakDelay".as_ptr(), Q_irand(1000, 3000));
}

/*
-------------------------
ST_CheckFireState
-------------------------
*/

unsafe fn ST_CheckFireState() {
    if enemyCS != qfalse {
        //if have a clear shot, always try
        return;
    }

    if (*NPCInfo).squadState == SQUAD_RETREAT
        || (*NPCInfo).squadState == SQUAD_TRANSITION
        || (*NPCInfo).squadState == SQUAD_SCOUT
    {
        //runners never try to fire at the last pos
        return;
    }

    if VectorCompare(
        (*(*NPC).client).ps.velocity.as_ptr(),
        core::ptr::addr_of!(vec3_origin) as *const f32,
    ) == 0
    {
        //if moving at all, don't do this
        return;
    }

    //See if we should continue to fire on their last position
    // TIMER_Done( NPC, "stick" ) ||
    if hitAlly == qfalse //we're not going to hit an ally
        && enemyInFOV != qfalse //enemy is in our FOV //FIXME: or we don't have a clear LOS?
        && (*NPCInfo).enemyLastSeenTime > 0 //we've seen the enemy
        && !(*NPCInfo).group.is_null() //have a group
        && ((*(*NPCInfo).group).numState[SQUAD_RETREAT as usize] > 0
            || (*(*NPCInfo).group).numState[SQUAD_TRANSITION as usize] > 0
            || (*(*NPCInfo).group).numState[SQUAD_SCOUT as usize] > 0)
    //laying down covering fire
    {
        if (*core::ptr::addr_of!(level)).time - (*NPCInfo).enemyLastSeenTime < 10000 //we have seem the enemy in the last 10 seconds
            && ((*NPCInfo).group.is_null()
                || (*core::ptr::addr_of!(level)).time - (*(*NPCInfo).group).lastSeenEnemyTime < 10000)
        //we are not in a group or the group has seen the enemy in the last 10 seconds
        {
            if Q_irand(0, 10) == 0 {
                //Fire on the last known position
                let mut muzzle: vec3_t = [0.0; 3];
                let mut dir: vec3_t = [0.0; 3];
                let mut angles: vec3_t = [0.0; 3];
                let mut tooClose: qboolean = qfalse;
                let mut tooFar: qboolean = qfalse;

                CalcEntitySpot(NPC, SPOT_HEAD, muzzle.as_mut_ptr());
                if VectorCompare(
                    core::ptr::addr_of!(impactPos) as *const f32,
                    core::ptr::addr_of!(vec3_origin) as *const f32,
                ) != 0
                {
                    //never checked ShotEntity this frame, so must do a trace...
                    let mut tr: trace_t = core::mem::zeroed();
                    //vec3_t	mins = {-2,-2,-2}, maxs = {2,2,2};
                    let mut forward: vec3_t = [0.0; 3];
                    let mut end: vec3_t = [0.0; 3];
                    AngleVectors(
                        (*(*NPC).client).ps.viewangles.as_ptr(),
                        forward.as_mut_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    VectorMA(muzzle.as_ptr(), 8192.0, forward.as_ptr(), end.as_mut_ptr());
                    gi_trace(
                        &mut tr,
                        muzzle.as_ptr(),
                        core::ptr::addr_of!(vec3_origin) as *const f32,
                        core::ptr::addr_of!(vec3_origin) as *const f32,
                        end.as_ptr(),
                        (*NPC).s.number,
                        MASK_SHOT,
                    );
                    VectorCopy(
                        tr.endpos.as_ptr(),
                        core::ptr::addr_of_mut!(impactPos) as *mut f32,
                    );
                }

                //see if impact would be too close to me
                let mut distThreshold: f32 = 16384.0 /*128*128*/; //default
                match (*NPC).s.weapon {
                    WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                    | WP_DET_PACK => {
                        distThreshold = 65536.0 /*256*256*/;
                    }
                    WP_REPEATER => {
                        if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                            distThreshold = 65536.0 /*256*256*/;
                        }
                    }
                    WP_CONCUSSION => {
                        if (*NPCInfo).scriptFlags & SCF_ALT_FIRE == 0 {
                            distThreshold = 65536.0 /*256*256*/;
                        }
                    }
                    _ => {}
                }

                let mut dist: f32 = DistanceSquared(
                    core::ptr::addr_of!(impactPos) as *const f32,
                    muzzle.as_ptr(),
                );

                if dist < distThreshold {
                    //impact would be too close to me
                    tooClose = qtrue;
                } else if (*core::ptr::addr_of!(level)).time - (*NPCInfo).enemyLastSeenTime > 5000
                    || (!(*NPCInfo).group.is_null()
                        && (*core::ptr::addr_of!(level)).time
                            - (*(*NPCInfo).group).lastSeenEnemyTime
                            > 5000)
                {
                    //we've haven't seen them in the last 5 seconds
                    //see if it's too far from where he is
                    distThreshold = 65536.0 /*256*256*/; //default
                    match (*NPC).s.weapon {
                        WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE
                        | WP_DET_PACK => {
                            distThreshold = 262144.0 /*512*512*/;
                        }
                        WP_REPEATER => {
                            if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                                distThreshold = 262144.0 /*512*512*/;
                            }
                        }
                        WP_CONCUSSION => {
                            if (*NPCInfo).scriptFlags & SCF_ALT_FIRE == 0 {
                                distThreshold = 262144.0 /*512*512*/;
                            }
                        }
                        _ => {}
                    }
                    dist = DistanceSquared(
                        core::ptr::addr_of!(impactPos) as *const f32,
                        (*NPCInfo).enemyLastSeenLocation.as_ptr(),
                    );
                    if dist > distThreshold {
                        //impact would be too far from enemy
                        tooFar = qtrue;
                    }
                }

                if tooClose == qfalse && tooFar == qfalse {
                    //okay too shoot at last pos
                    VectorSubtract(
                        (*NPCInfo).enemyLastSeenLocation.as_ptr(),
                        muzzle.as_ptr(),
                        dir.as_mut_ptr(),
                    );
                    VectorNormalize(dir.as_mut_ptr());
                    vectoangles(dir.as_ptr(), angles.as_mut_ptr());

                    (*NPCInfo).desiredYaw = angles[YAW as usize];
                    (*NPCInfo).desiredPitch = angles[PITCH as usize];

                    shoot = qtrue;
                    faceEnemy = qfalse;
                    //AI_GroupUpdateSquadstates( NPCInfo->group, NPC, SQUAD_STAND_AND_SHOOT );
                    return;
                }
            }
        }
    }
}

pub unsafe fn ST_TrackEnemy(self_: *mut gentity_t, enemyPos: *const f32) {
    //clear timers
    TIMER_Set(self_, c"attackDelay".as_ptr(), Q_irand(1000, 2000));
    //TIMER_Set( self, "duck", -1 );
    TIMER_Set(self_, c"stick".as_ptr(), Q_irand(500, 1500));
    TIMER_Set(self_, c"stand".as_ptr(), -1);
    TIMER_Set(
        self_,
        c"scoutTime".as_ptr(),
        TIMER_Get(self_, c"stick".as_ptr()) - (*core::ptr::addr_of!(level)).time
            + Q_irand(5000, 10000),
    );
    //leave my combat point
    NPC_FreeCombatPoint((*(*self_).NPC).combatPoint);
    //go after his last seen pos
    NPC_SetMoveGoal(self_, enemyPos, 100.0, qfalse);
    if Q_irand(0, 3) == 0 {
        (*NPCInfo).aiFlags |= NPCAI_STOP_AT_LOS;
    }
}

pub unsafe fn ST_ApproachEnemy(self_: *mut gentity_t) -> c_int {
    TIMER_Set(self_, c"attackDelay".as_ptr(), Q_irand(250, 500));
    //TIMER_Set( self, "duck", -1 );
    TIMER_Set(self_, c"stick".as_ptr(), Q_irand(1000, 2000));
    TIMER_Set(self_, c"stand".as_ptr(), -1);
    TIMER_Set(
        self_,
        c"scoutTime".as_ptr(),
        TIMER_Get(self_, c"stick".as_ptr()) - (*core::ptr::addr_of!(level)).time
            + Q_irand(5000, 10000),
    );
    //leave my combat point
    NPC_FreeCombatPoint((*(*self_).NPC).combatPoint);
    //return the relevant combat point flags
    (CP_CLEAR | CP_CLOSEST)
}

pub unsafe fn ST_HuntEnemy(self_: *mut gentity_t) {
    //TIMER_Set( NPC, "attackDelay", Q_irand( 250, 500 ) );//Disabled this for now, guys who couldn't hunt would never attack
    //TIMER_Set( NPC, "duck", -1 );
    TIMER_Set(NPC, c"stick".as_ptr(), Q_irand(250, 1000));
    TIMER_Set(NPC, c"stand".as_ptr(), -1);
    TIMER_Set(
        NPC,
        c"scoutTime".as_ptr(),
        TIMER_Get(NPC, c"stick".as_ptr()) - (*core::ptr::addr_of!(level)).time
            + Q_irand(5000, 10000),
    );
    //leave my combat point
    NPC_FreeCombatPoint((*NPCInfo).combatPoint);
    //go directly after the enemy
    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        (*(*self_).NPC).goalEntity = (*NPC).enemy;
    }
}

pub unsafe fn ST_TransferTimers(self_: *mut gentity_t, other: *mut gentity_t) {
    TIMER_Set(
        other,
        c"attackDelay".as_ptr(),
        TIMER_Get(self_, c"attackDelay".as_ptr()) - (*core::ptr::addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"duck".as_ptr(),
        TIMER_Get(self_, c"duck".as_ptr()) - (*core::ptr::addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"stick".as_ptr(),
        TIMER_Get(self_, c"stick".as_ptr()) - (*core::ptr::addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"scoutTime".as_ptr(),
        TIMER_Get(self_, c"scoutTime".as_ptr()) - (*core::ptr::addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"roamTime".as_ptr(),
        TIMER_Get(self_, c"roamTime".as_ptr()) - (*core::ptr::addr_of!(level)).time,
    );
    TIMER_Set(
        other,
        c"stand".as_ptr(),
        TIMER_Get(self_, c"stand".as_ptr()) - (*core::ptr::addr_of!(level)).time,
    );
    TIMER_Set(self_, c"attackDelay".as_ptr(), -1);
    TIMER_Set(self_, c"duck".as_ptr(), -1);
    TIMER_Set(self_, c"stick".as_ptr(), -1);
    TIMER_Set(self_, c"scoutTime".as_ptr(), -1);
    TIMER_Set(self_, c"roamTime".as_ptr(), -1);
    TIMER_Set(self_, c"stand".as_ptr(), -1);
}

pub unsafe fn ST_TransferMoveGoal(self_: *mut gentity_t, other: *mut gentity_t) {
    if Q3_TaskIDPending(self_, TID_MOVE_NAV) != qfalse {
        //can't transfer movegoal when a script we're running is waiting to complete
        return;
    }
    if (*(*self_).NPC).combatPoint != -1 {
        //I've got a combatPoint I'm going to, give it to him
        (*(*other).NPC).combatPoint = (*(*self_).NPC).combatPoint;
        (*(*self_).NPC).lastFailedCombatPoint = (*(*other).NPC).combatPoint;
        (*(*self_).NPC).combatPoint = -1;
    } else {
        //I must be going for a goal, give that to him instead
        if (*(*self_).NPC).goalEntity == (*(*self_).NPC).tempGoal {
            NPC_SetMoveGoal(
                other,
                (*(*(*self_).NPC).tempGoal).currentOrigin.as_ptr(),
                (*(*self_).NPC).goalRadius,
                if (*(*(*self_).NPC).tempGoal).svFlags & SVF_NAVGOAL != 0 {
                    qtrue
                } else {
                    qfalse
                },
            );
        } else {
            (*(*other).NPC).goalEntity = (*(*self_).NPC).goalEntity;
        }
    }
    //give him my squadstate
    AI_GroupUpdateSquadstates((*(*self_).NPC).group, other, (*NPCInfo).squadState);

    //give him my timers and clear mine
    ST_TransferTimers(self_, other);

    //now make me stand around for a second or two at least
    AI_GroupUpdateSquadstates((*(*self_).NPC).group, self_, SQUAD_STAND_AND_SHOOT);
    TIMER_Set(self_, c"stand".as_ptr(), Q_irand(1000, 3000));
}

pub unsafe fn ST_GetCPFlags() -> c_int {
    let mut cpFlags: c_int = 0;
    if !NPC.is_null() && !(*NPCInfo).group.is_null() {
        if NPC == (*(*NPCInfo).group).commander
            && (*(*NPC).client).NPC_class == CLASS_IMPERIAL
        {
            //imperials hang back and give orders
            if (*(*NPCInfo).group).numGroup > 1
                && Q_irand(-3, (*(*NPCInfo).group).numGroup) > 1
            {
                //FIXME: make sure he;s giving orders with these lines
                if Q_irand(0, 1) != 0 {
                    ST_Speech(NPC, SPEECH_CHASE, 0.5);
                } else {
                    ST_Speech(NPC, SPEECH_YELL, 0.5);
                }
            }
            cpFlags = CP_CLEAR | CP_COVER | CP_AVOID | CP_SAFE | CP_RETREAT;
        } else if (*(*NPCInfo).group).morale < 0 {
            //hide
            cpFlags = CP_COVER | CP_AVOID | CP_SAFE | CP_RETREAT;
            /*
            if ( NPC->client->NPC_class == CLASS_SABOTEUR && !Q_irand( 0, 3 ) )
            {
                Saboteur_Cloak( NPC );
            }
            */
        }
/*		else if ( NPCInfo->group->morale < NPCInfo->group->numGroup )
        {//morale is low for our size
            int moraleDrop = NPCInfo->group->numGroup - NPCInfo->group->morale;
            if ( moraleDrop < -6 )
            {//flee (no clear shot needed)
                cpFlags = (CP_FLEE|CP_RETREAT|CP_COVER|CP_AVOID|CP_SAFE);
            }
            else if ( moraleDrop < -3 )
            {//retreat (no clear shot needed)
                cpFlags = (CP_RETREAT|CP_COVER|CP_AVOID|CP_SAFE);
            }
            else if ( moraleDrop < 0 )
            {//cover (no clear shot needed)
                cpFlags = (CP_COVER|CP_AVOID|CP_SAFE);
            }
        }*/
        else {
            let moraleBoost: c_int =
                (*(*NPCInfo).group).morale - (*(*NPCInfo).group).numGroup;
            if moraleBoost > 20 {
                //charge to any one and outflank (no cover needed)
                cpFlags = CP_CLEAR | CP_FLANK | CP_APPROACH_ENEMY;
                //Saboteur_Decloak( NPC );
            } else if moraleBoost > 15 {
                //charge to closest one (no cover needed)
                cpFlags = CP_CLEAR | CP_CLOSEST | CP_APPROACH_ENEMY;
                /*
                if ( NPC->client->NPC_class == CLASS_SABOTEUR && !Q_irand( 0, 3 ) )
                {
                    Saboteur_Decloak( NPC );
                }
                */
            } else if moraleBoost > 10 {
                //charge closer (no cover needed)
                cpFlags = CP_CLEAR | CP_APPROACH_ENEMY;
                /*
                if ( NPC->client->NPC_class == CLASS_SABOTEUR && !Q_irand( 0, 6 ) )
                {
                    Saboteur_Decloak( NPC );
                }
                */
            }
        }
    }
    if cpFlags == 0 {
        //at some medium level of morale
        match Q_irand(0, 3) {
            0 => {
                //just take the nearest one
                cpFlags = CP_CLEAR | CP_COVER | CP_NEAREST;
            }
            1 => {
                //take one closer to the enemy
                cpFlags = CP_CLEAR | CP_COVER | CP_APPROACH_ENEMY;
            }
            2 => {
                //take the one closest to the enemy
                cpFlags = CP_CLEAR | CP_COVER | CP_CLOSEST | CP_APPROACH_ENEMY;
            }
            3 => {
                //take the one on the other side of the enemy
                cpFlags = CP_CLEAR | CP_COVER | CP_FLANK | CP_APPROACH_ENEMY;
            }
            _ => {}
        }
    }
    if !NPC.is_null() && (*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST != 0 {
        cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
        cpFlags |= CP_NEAREST;
    }
    cpFlags
}
/*
-------------------------
ST_Commander

  Make decisions about who should go where, etc.

FIXME: leader (group-decision-making) AI?
FIXME: need alternate routes!
FIXME: more group voice interaction
FIXME: work in pairs?

-------------------------
*/
pub unsafe fn ST_Commander() {
    let mut i: c_int; //, j;
    let mut cp: c_int = 0;
    let mut cpFlags_org: c_int = 0;
    let mut cpFlags: c_int = 0;
    let group: *mut AIGroupInfo_t = (*NPCInfo).group;
    let mut member: *mut gentity_t; //, *buddy;
    let mut runner: qboolean = qfalse;
    let mut enemyLost: qboolean = qfalse;
    let mut scouting: qboolean = qfalse;
    let mut squadState: c_int = 0;
    let mut avoidDist: f32 = 0.0f32;

    (*group).processed = qtrue;

    if (*group).enemy.is_null() || (*(*group).enemy).client.is_null() {
        //hmm, no enemy...?!
        return;
    }

    //FIXME: have this group commander check the enemy group (if any) and see if they have
    //		superior numbers.  If they do, fall back rather than advance.  If you have
    //		superior numbers, advance on them.
    //FIXME: find the group commander and have him occasionally give orders when there is speech
    //FIXME: start fleeing when only a couple of you vs. a lightsaber, possibly give up if the only one left

    SaveNPCGlobals();

    if (*group).lastSeenEnemyTime < (*level).time - 180000 {
        //dissolve the group
        ST_Speech(NPC, SPEECH_LOST, 0.0f32);
        (*(*group).enemy).waypoint = NAV_GetNearestNode((*group).enemy);
        i = 0;
        while i < (*group).numGroup {
            member = addr_of_mut!(g_entities[(*group).member[i as usize].number as usize]);
            SetNPCGlobals(member);
            if Q3_TaskIDPending(NPC, TID_MOVE_NAV) != 0 {
                //running somewhere that a script requires us to go, don't break from that
                i += 1;
                continue;
            }
            if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0 {
                //not allowed to move on my own
                i += 1;
                continue;
            }
            //Lost enemy for three minutes?  go into search mode?
            G_ClearEnemy(NPC);
            (*NPC).waypoint = NAV_GetNearestNode((*group).enemy);
            if (*NPC).waypoint == WAYPOINT_NONE {
                (*NPCInfo).behaviorState = BS_DEFAULT; //BS_PATROL;
            } else if (*(*group).enemy).waypoint == WAYPOINT_NONE
                || NAV_EstimateCostToGoal((*NPC).waypoint, (*(*group).enemy).waypoint) >= Q3_INFINITE
            {
                NPC_BSSearchStart((*NPC).waypoint, BS_SEARCH);
            } else {
                NPC_BSSearchStart((*(*group).enemy).waypoint, BS_SEARCH);
            }
            i += 1;
        }
        (*group).enemy = core::ptr::null_mut();
        RestoreNPCGlobals();
        return;
    }

    //see if anyone is running
    if (*group).numState[SQUAD_SCOUT as usize] > 0
        || (*group).numState[SQUAD_TRANSITION as usize] > 0
        || (*group).numState[SQUAD_RETREAT as usize] > 0
    {
        //someone is running
        runner = qtrue;
    }

    if /* !runner && */ (*group).lastSeenEnemyTime > (*level).time - 32000
        && (*group).lastSeenEnemyTime < (*level).time - 30000
    {
        //no-one has seen the enemy for 30 seconds// and no-one is running after him
        if !(*group).commander.is_null() && Q_irand(0, 1) == 0 {
            ST_Speech((*group).commander, SPEECH_ESCAPING, 0.0f32);
        } else {
            ST_Speech(NPC, SPEECH_ESCAPING, 0.0f32);
        }
        //don't say this again
        (*NPCInfo).blockedSpeechDebounceTime = (*level).time + 3000;
    }

    if (*group).lastSeenEnemyTime < (*level).time - 7000 {
        //no-one has seen the enemy for at least 10 seconds!  Should send a scout
        enemyLost = qtrue;
    }

    //Go through the list:

    //Everyone should try to get to a combat point if possible
    let curMemberNum: c_int;
    let lastMemberNum: c_int;
    if (*d_asynchronousGroupAI).integer != 0 {
        //do one member a turn
        (*group).activeMemberNum += 1;
        if (*group).activeMemberNum >= (*group).numGroup {
            (*group).activeMemberNum = 0;
        }
        curMemberNum = (*group).activeMemberNum;
        lastMemberNum = curMemberNum + 1;
    } else {
        curMemberNum = 0;
        lastMemberNum = (*group).numGroup;
    }
    i = curMemberNum;
    while i < lastMemberNum {
        //reset combat point flags
        cp = -1;
        cpFlags = 0;
        squadState = SQUAD_IDLE;
        avoidDist = 0.0f32;
        scouting = qfalse;

        //get the next guy
        member = addr_of_mut!(g_entities[(*group).member[i as usize].number as usize]);
        if (*member).enemy.is_null() {
            //don't include guys that aren't angry
            i += 1;
            continue;
        }
        SetNPCGlobals(member);

        if TIMER_Done(NPC, b"flee\0".as_ptr() as *const c_char) == 0 {
            //running away
            i += 1;
            continue;
        }

        if Q3_TaskIDPending(NPC, TID_MOVE_NAV) != 0 {
            //running somewhere that a script requires us to go
            i += 1;
            continue;
        }

        if (*NPC).s.weapon == WP_NONE
            && !(*NPCInfo).goalEntity.is_null()
            && (*NPCInfo).goalEntity == (*NPCInfo).tempGoal
            && (*(*NPCInfo).goalEntity).s.eType == ET_ITEM
        {
            //running to pick up a gun, don't do other logic
            i += 1;
            continue;
        }

        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES == 0 {
            //not allowed to do combat-movement
            i += 1;
            continue;
        }

        if (*(*NPC).client).ps.weapon == WP_NONE {
            //weaponless, should be hiding
            if (*NPCInfo).goalEntity.is_null()
                || (*(*NPCInfo).goalEntity).enemy.is_null()
                || (*(*(*NPCInfo).goalEntity).enemy).s.eType != ET_ITEM
            {
                //not running after a pickup
                if TIMER_Done(NPC, b"hideTime\0".as_ptr() as *const c_char) != 0
                    || (DistanceSquared(
                        (*(*group).enemy).currentOrigin.as_ptr(),
                        (*NPC).currentOrigin.as_ptr(),
                    ) < 65536.0f32
                        && NPC_ClearLOS4((*NPC).enemy) != 0)
                {
                    //done hiding or enemy near and can see us
                    //er, start another flee I guess?
                    NPC_StartFlee(
                        (*NPC).enemy,
                        (*(*NPC).enemy).currentOrigin.as_mut_ptr(),
                        AEL_DANGER_GREAT,
                        5000,
                        10000,
                    );
                } //else, just hang here
            }
            i += 1;
            continue;
        }

        if enemyLost != 0
            && NAV_InSameRegion(NPC, (*(*NPC).enemy).currentOrigin.as_mut_ptr()) != 0
        {
            ST_TrackEnemy(NPC, (*(*NPC).enemy).currentOrigin.as_mut_ptr());
            i += 1;
            continue;
        }

        if (*NPC).enemy.is_null() {
            i += 1;
            continue;
        }

        // Check To See We Have A Clear Shot To The Enemy Every Couple Seconds
        //---------------------------------------------------------------------
        if TIMER_Done(NPC, b"checkGrenadeTooCloseDebouncer\0".as_ptr() as *const c_char) != 0 {
            TIMER_Set(
                NPC,
                b"checkGrenadeTooCloseDebouncer\0".as_ptr() as *const c_char,
                Q_irand(300, 600),
            );

            let mut mins: vec3_t = [0.0f32; 3];
            let mut maxs: vec3_t = [0.0f32; 3];
            let mut fled: bool = false;
            let mut ent: *mut gentity_t = core::ptr::null_mut();

            let mut entityList: [*mut gentity_t; MAX_GENTITIES as usize] =
                [core::ptr::null_mut(); MAX_GENTITIES as usize];

            let mut idx: c_int = 0;
            while idx < 3 {
                mins[idx as usize] = (*NPC).currentOrigin[idx as usize] - 200.0f32;
                maxs[idx as usize] = (*NPC).currentOrigin[idx as usize] + 200.0f32;
                idx += 1;
            }

            let numListedEntities: c_int = gi_EntitiesInBox(
                mins.as_mut_ptr(),
                maxs.as_mut_ptr(),
                entityList.as_mut_ptr(),
                MAX_GENTITIES,
            );

            let mut e: c_int = 0;
            while e < numListedEntities {
                ent = entityList[e as usize];

                if ent == NPC {
                    e += 1;
                    continue;
                }
                if (*ent).owner == NPC {
                    e += 1;
                    continue;
                }
                if (*ent).inuse == 0 {
                    e += 1;
                    continue;
                }
                if (*ent).s.eType == ET_MISSILE {
                    if (*ent).s.weapon == WP_THERMAL {
                        //a thermal
                        if (*ent).has_bounced != 0
                            && ((*ent).owner.is_null() || OnSameTeam((*ent).owner, NPC) == 0)
                        {
                            //bounced and an enemy thermal
                            ST_Speech(NPC, SPEECH_COVER, 0); //FIXME: flee sound?
                            NPC_StartFlee(
                                (*NPC).enemy,
                                (*ent).currentOrigin.as_mut_ptr(),
                                AEL_DANGER_GREAT,
                                1000,
                                2000,
                            );
                            fled = true;
                            //							cpFlags |= (CP_CLEAR|CP_COVER);	// NOPE, Can't See The Enemy, So Find A New Combat Point
                            TIMER_Set(
                                NPC,
                                b"checkGrenadeTooCloseDebouncer\0".as_ptr() as *const c_char,
                                Q_irand(2000, 4000),
                            );
                            break;
                        }
                    }
                }
                e += 1;
            }
            if fled {
                i += 1;
                continue;
            }
        }

        // Check To See We Have A Clear Shot To The Enemy Every Couple Seconds
        //---------------------------------------------------------------------
        if TIMER_Done(NPC, b"checkEnemyVisDebouncer\0".as_ptr() as *const c_char) != 0 {
            TIMER_Set(
                NPC,
                b"checkEnemyVisDebouncer\0".as_ptr() as *const c_char,
                Q_irand(3000, 7000),
            );
            if NPC_ClearLOS4((*NPC).enemy) == 0 {
                cpFlags |= CP_CLEAR | CP_COVER; // NOPE, Can't See The Enemy, So Find A New Combat Point
            }
        }

        // Check To See If The Enemy Is Too Close For Comfort
        //----------------------------------------------------
        if (*(*NPC).client).NPC_class != CLASS_ASSASSIN_DROID {
            if TIMER_Done(NPC, b"checkEnemyTooCloseDebouncer\0".as_ptr() as *const c_char) != 0 {
                TIMER_Set(
                    NPC,
                    b"checkEnemyTooCloseDebouncer\0".as_ptr() as *const c_char,
                    Q_irand(1000, 6000),
                );

                let mut distThreshold: f32 = 16384.0f32; /*128*128*/ //default
                match (*NPC).s.weapon {
                    WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE | WP_DET_PACK => {
                        distThreshold = 65536.0f32; /*256*256*/
                    }
                    WP_REPEATER => {
                        if (*NPCInfo).scriptFlags & SCF_ALT_FIRE != 0 {
                            distThreshold = 65536.0f32; /*256*256*/
                        }
                    }
                    WP_CONCUSSION => {
                        if (*NPCInfo).scriptFlags & SCF_ALT_FIRE == 0 {
                            distThreshold = 65536.0f32; /*256*256*/
                        }
                    }
                    _ => {}
                }

                if DistanceSquared(
                    (*(*group).enemy).currentOrigin.as_ptr(),
                    (*NPC).currentOrigin.as_ptr(),
                ) < distThreshold
                {
                    cpFlags |= CP_CLEAR | CP_COVER;
                }
            }
        }

        //clear the local state
        (*NPCInfo).localState = LSTATE_NONE;

        cpFlags &= !CP_NEAREST;
        //Assign combat points
        if cpFlags != 0 {
            //we want to run to a combat point
            //always avoid enemy when picking combat points, and we always want to be able to get there
            cpFlags |= CP_AVOID_ENEMY | CP_HAS_ROUTE | CP_TRYFAR;
            avoidDist = 200.0f32;
            cpFlags_org = cpFlags; //remember what we *wanted* to do...

            //now get a combat point
            if cp == -1 {
                //may have had sone set above
                cp = NPC_FindCombatPointRetry(
                    (*NPC).currentOrigin.as_mut_ptr(),
                    (*NPC).currentOrigin.as_mut_ptr(),
                    (*NPC).currentOrigin.as_mut_ptr(),
                    addr_of_mut!(cpFlags),
                    avoidDist,
                    (*NPCInfo).lastFailedCombatPoint,
                );
            }

            //see if we got a valid one
            if cp != -1 {
                //found a combat point
                //let others know that someone is now running
                runner = qtrue;
                //don't change course again until we get to where we're going
                TIMER_Set(NPC, b"roamTime\0".as_ptr() as *const c_char, Q3_INFINITE);

                NPC_SetCombatPoint(cp);
                NPC_SetMoveGoal(
                    NPC,
                    (*level).combatPoints[cp as usize].origin.as_mut_ptr(),
                    8,
                    qtrue,
                    cp,
                );

                // If Successfully
                if (cpFlags & CP_FLANK != 0)
                    || ((cpFlags & CP_COVER != 0) && (cpFlags & CP_CLEAR != 0))
                {
                } else if Q_irand(0, 3) == 0 {
                    (*NPCInfo).aiFlags |= NPCAI_STOP_AT_LOS;
                }

                //okay, try a move right now to see if we can even get there
                if cpFlags & CP_FLANK != 0 {
                    if (*group).numGroup > 1 {
                        NPC_ST_StoreMovementSpeech(SPEECH_OUTFLANK, -1);
                    }
                } else if (cpFlags & CP_COVER != 0) && (cpFlags & CP_CLEAR == 0) {
                    //going into hiding
                    NPC_ST_StoreMovementSpeech(SPEECH_COVER, -1);
                } else {
                    if Q_irand(0, 20) == 0 {
                        //hell, we're loading the sounds, use them every now and then!
                        if Q_irand(0, 1) != 0 {
                            NPC_ST_StoreMovementSpeech(SPEECH_OUTFLANK, -1);
                        } else {
                            NPC_ST_StoreMovementSpeech(SPEECH_ESCAPING, -1);
                        }
                    }
                }
            }
        }
        i += 1;
    }

    RestoreNPCGlobals();
}

// extern void G_Knockdown( gentity_t *self, gentity_t *attacker, const vec3_t pushDir, float strength, qboolean breakSaberLock );
// (declared in shared preamble)
pub unsafe fn Noghri_StickTrace() {
    if (*NPC).ghoul2.size() == 0 || (*NPC).weaponModel[0] <= 0 {
        return;
    }

    let boltIndex: c_int = gi_G2API_AddBolt(
        addr_of_mut!((*NPC).ghoul2[(*NPC).weaponModel[0] as usize]),
        b"*weapon\0".as_ptr() as *const c_char,
    );
    if boltIndex != -1 {
        let curTime: c_int = if (*cg).time != 0 { (*cg).time } else { (*level).time };
        let mut hit: qboolean = qfalse;
        let mut lastHit: c_int = ENTITYNUM_NONE;
        let mut time: c_int = curTime - 25;
        while time <= curTime + 25 && hit == 0 {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut tip: vec3_t = [0.0f32; 3];
            let mut dir: vec3_t = [0.0f32; 3];
            let mut base: vec3_t = [0.0f32; 3];
            let mut angles: vec3_t = [0.0f32, (*NPC).currentAngles[YAW as usize], 0.0f32];
            let mut mins: vec3_t = [-2.0f32, -2.0f32, -2.0f32];
            let mut maxs: vec3_t = [2.0f32, 2.0f32, 2.0f32];
            let mut trace: trace_t = core::mem::zeroed();

            gi_G2API_GetBoltMatrix(
                addr_of_mut!((*NPC).ghoul2),
                (*NPC).weaponModel[0],
                boltIndex,
                addr_of_mut!(boltMatrix),
                angles.as_mut_ptr(),
                (*NPC).currentOrigin.as_mut_ptr(),
                time,
                core::ptr::null_mut(),
                (*NPC).s.modelScale.as_mut_ptr(),
            );
            gi_G2API_GiveMeVectorFromMatrix(boltMatrix, ORIGIN, base.as_mut_ptr());
            gi_G2API_GiveMeVectorFromMatrix(boltMatrix, POSITIVE_Y, dir.as_mut_ptr());
            // VectorMA( base, 48, dir, tip )
            tip[0] = base[0] + 48.0f32 * dir[0];
            tip[1] = base[1] + 48.0f32 * dir[1];
            tip[2] = base[2] + 48.0f32 * dir[2];
    #[cfg(not(feature = "final_build"))]
            {
                if (*d_saberCombat).integer > 1 {
                    G_DebugLine(
                        base.as_mut_ptr(),
                        tip.as_mut_ptr(),
                        FRAMETIME,
                        0x000000ff_i32,
                        qtrue,
                    );
                }
            }
            gi_trace(
                addr_of_mut!(trace),
                base.as_mut_ptr(),
                mins.as_mut_ptr(),
                maxs.as_mut_ptr(),
                tip.as_mut_ptr(),
                (*NPC).s.number,
                MASK_SHOT,
                G2_RETURNONHIT,
                10,
            );
            if trace.fraction < 1.0f32 && trace.entityNum != lastHit {
                //hit something
                let traceEnt: *mut gentity_t =
                    addr_of_mut!(g_entities[trace.entityNum as usize]);
                if (*traceEnt).takedamage != 0
                    && ((*traceEnt).client.is_null()
                        || traceEnt == (*NPC).enemy
                        || (*(*traceEnt).client).NPC_class != (*(*NPC).client).NPC_class)
                {
                    //smack
                    let dmg: c_int = Q_irand(12, 20); //FIXME: base on skill!
                    //FIXME: debounce?
                    G_Sound(
                        traceEnt,
                        G_SoundIndex(va(
                            b"sound/weapons/tusken_staff/stickhit%d.wav\0".as_ptr()
                                as *const c_char,
                            Q_irand(1, 4),
                        )),
                    );
                    G_Damage(
                        traceEnt,
                        NPC,
                        NPC,
                        addr_of_mut!(vec3_origin).cast::<f32>(),
                        trace.endpos.as_mut_ptr(),
                        dmg,
                        DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE,
                    );
                    if (*traceEnt).health > 0 && dmg > 17 {
                        //do pain on enemy
                        G_Knockdown(traceEnt, NPC, dir.as_ptr(), 300.0f32, qtrue);
                    }
                    lastHit = trace.entityNum;
                    hit = qtrue;
                }
            }
            time += 25;
        }
    }
}
extern "C" {
    // NPC_ClearLOS 1-argument entity-target variant; the shared preamble provides
    // the NPC_ClearLOS4/NPC_ClearLOS5 variants (other arg-count forms).
    fn NPC_ClearLOS(ent: *mut gentity_t) -> qboolean;
}

/*
-------------------------
NPC_BSST_Attack
-------------------------
*/

pub unsafe fn NPC_BSST_Attack() {
    //Don't do anything if we're hurt
    if (*NPC).painDebounceTime > (*core::ptr::addr_of!(level)).time {
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    //NPC_CheckEnemy( qtrue, qfalse );
    //If we don't have an enemy, just idle
    if NPC_CheckEnemyExt() == qfalse // NPC->enemy )//
    {
        if (*(*NPC).client).playerTeam == TEAM_PLAYER {
            NPC_BSPatrol();
        } else {
            NPC_BSST_Patrol(); //FIXME: or patrol?
        }
        return;
    }

    //FIXME: put some sort of delay into the guys depending on how they saw you...?

    //Get our group info
    if TIMER_Done(NPC, b"interrogating\0".as_ptr() as *const c_char) != qfalse {
        AI_GetGroup(NPC); //, 45, 512, NPC->enemy );
    } else {
        //FIXME: when done interrogating, I should send out a team alert!
    }

    if !(*NPCInfo).group.is_null() {
        //I belong to a squad of guys - we should *always* have a group
        if (*(*NPCInfo).group).processed == qfalse {
            //I'm the first ent in my group, I'll make the command decisions
            #[cfg(feature = "AI_TIMERS")]
            let start_time: c_int = GetTime(0);
            ST_Commander();
            // Port note: S_COLOR_RED/YELLOW/GREEN macros ("^1"/"^3"/"^2") inlined into
            // the byte strings below; C did string-literal concatenation at compile time.
            #[cfg(feature = "AI_TIMERS")]
            {
                let comm_time: c_int = GetTime(start_time);
                if comm_time > 20 {
                    gi_Printf(
                        b"^1ERROR: Commander time: %d\n\0".as_ptr() as *const c_char,
                        comm_time,
                    );
                } else if comm_time > 10 {
                    gi_Printf(
                        b"^3WARNING: Commander time: %d\n\0".as_ptr() as *const c_char,
                        comm_time,
                    );
                } else if comm_time > 2 {
                    gi_Printf(
                        b"^2Commander time: %d\n\0".as_ptr() as *const c_char,
                        comm_time,
                    );
                }
            }
        }
    } else if TIMER_Done(NPC, b"flee\0".as_ptr() as *const c_char) != qfalse
        && NPC_CheckForDanger(NPC_CheckAlertEvents(qtrue, qtrue, -1, qfalse, AEL_DANGER))
            != qfalse
    {
        //not already fleeing, and going to run
        ST_Speech(NPC, SPEECH_COVER, 0);
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    if (*NPC).enemy.is_null() {
        //WTF?  somehow we lost our enemy?
        NPC_BSST_Patrol(); //FIXME: or patrol?
        return;
    }

    if !(*NPCInfo).goalEntity.is_null() && (*NPCInfo).goalEntity != (*NPC).enemy {
        (*NPCInfo).goalEntity = UpdateGoal();
    }

    enemyLOS = qfalse;
    enemyCS = qfalse;
    enemyInFOV = qfalse;
    r#move = qtrue;
    faceEnemy = qfalse;
    shoot = qfalse;
    hitAlly = qfalse;
    // VectorClear( impactPos )
    impactPos[0] = 0.0;
    impactPos[1] = 0.0;
    impactPos[2] = 0.0;
    enemyDist = DistanceSquared(
        (*NPC).currentOrigin.as_ptr(),
        (*(*NPC).enemy).currentOrigin.as_ptr(),
    );

    let mut enemyDir: vec3_t = [0.0; 3];
    let mut shootDir: vec3_t = [0.0; 3];
    // VectorSubtract( NPC->enemy->currentOrigin, NPC->currentOrigin, enemyDir )
    enemyDir[0] = (*(*NPC).enemy).currentOrigin[0] - (*NPC).currentOrigin[0];
    enemyDir[1] = (*(*NPC).enemy).currentOrigin[1] - (*NPC).currentOrigin[1];
    enemyDir[2] = (*(*NPC).enemy).currentOrigin[2] - (*NPC).currentOrigin[2];
    VectorNormalize(enemyDir.as_mut_ptr());
    AngleVectors(
        (*(*NPC).client).ps.viewangles.as_ptr(),
        shootDir.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    // DotProduct( enemyDir, shootDir )
    let dot: vec_t = enemyDir[0] * shootDir[0]
        + enemyDir[1] * shootDir[1]
        + enemyDir[2] * shootDir[2];
    if dot > 0.5 || (enemyDist * (1.0 - dot)) < 10000.0 {
        //enemy is in front of me or they're very close and not behind me
        enemyInFOV = qtrue;
    }

    if enemyDist < MIN_ROCKET_DIST_SQUARED //128
    {
        //enemy within 128
        if ((*(*NPC).client).ps.weapon == WP_FLECHETTE
            || (*(*NPC).client).ps.weapon == WP_REPEATER)
            && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0
        {
            //shooting an explosive, but enemy too close, switch to primary fire
            (*NPCInfo).scriptFlags &= !SCF_ALT_FIRE;
            //FIXME: we can never go back to alt-fire this way since, after this, we don't know if we were initially supposed to use alt-fire or not...
        }
    } else if enemyDist > 65536.0 //256 squared
    {
        if (*(*NPC).client).ps.weapon == WP_DISRUPTOR {
            //sniping...
            if ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0 {
                //use primary fire
                (*NPCInfo).scriptFlags |= SCF_ALT_FIRE;
                //reset fire-timing variables
                NPC_ChangeWeapon((*(*NPC).client).ps.weapon);
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }
    }

    //can we see our target?
    if NPC_ClearLOS((*NPC).enemy) != qfalse {
        AI_GroupUpdateEnemyLastSeen(
            (*NPCInfo).group,
            (*(*NPC).enemy).currentOrigin.as_mut_ptr(),
        );
        (*NPCInfo).enemyLastSeenTime = (*core::ptr::addr_of!(level)).time;
        enemyLOS = qtrue;

        if (*(*NPC).client).ps.weapon == WP_NONE {
            enemyCS = qfalse; //not true, but should stop us from firing
            NPC_AimAdjust(-1); //adjust aim worse longer we have no weapon
        } else {
            //can we shoot our target?
            if (enemyDist < MIN_ROCKET_DIST_SQUARED)
                && (((*core::ptr::addr_of!(level)).time - (*NPC).lastMoveTime) < 5000)
                && ((*(*NPC).client).ps.weapon == WP_ROCKET_LAUNCHER
                    || ((*(*NPC).client).ps.weapon == WP_CONCUSSION
                        && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0)
                    || ((*(*NPC).client).ps.weapon == WP_FLECHETTE
                        && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) != 0))
            {
                enemyCS = qfalse; //not true, but should stop us from firing
                hitAlly = qtrue; //us!
                              //FIXME: if too close, run away!
            } else if enemyInFOV != qfalse {
                //if enemy is FOV, go ahead and check for shooting
                let hit: c_int = NPC_ShotEntity((*NPC).enemy, impactPos.as_mut_ptr());
                let hit_ent: *mut gentity_t =
                    core::ptr::addr_of_mut!(g_entities[hit as usize]);

                if hit == (*(*NPC).enemy).s.number
                    || (!hit_ent.is_null()
                        && !(*hit_ent).client.is_null()
                        && (*(*hit_ent).client).playerTeam == (*(*NPC).client).enemyTeam)
                    || (!hit_ent.is_null()
                        && (*hit_ent).takedamage != qfalse
                        && (((*hit_ent).svFlags & SVF_GLASS_BRUSH) != 0
                            || (*hit_ent).health < 40
                            || (*NPC).s.weapon == WP_EMPLACED_GUN))
                {
                    //can hit enemy or enemy ally or will hit glass or other minor breakable (or in emplaced gun), so shoot anyway
                    AI_GroupUpdateClearShotTime((*NPCInfo).group);
                    enemyCS = qtrue;
                    NPC_AimAdjust(2); //adjust aim better longer we have clear shot at enemy
                    // VectorCopy( NPC->enemy->currentOrigin, NPCInfo->enemyLastSeenLocation )
                    (*NPCInfo).enemyLastSeenLocation[0] = (*(*NPC).enemy).currentOrigin[0];
                    (*NPCInfo).enemyLastSeenLocation[1] = (*(*NPC).enemy).currentOrigin[1];
                    (*NPCInfo).enemyLastSeenLocation[2] = (*(*NPC).enemy).currentOrigin[2];
                } else {
                    //Hmm, have to get around this bastard
                    NPC_AimAdjust(1); //adjust aim better longer we can see enemy
                    ST_ResolveBlockedShot(hit);
                    if !hit_ent.is_null()
                        && !(*hit_ent).client.is_null()
                        && (*(*hit_ent).client).playerTeam == (*(*NPC).client).playerTeam
                    {
                        //would hit an ally, don't fire!!!
                        hitAlly = qtrue;
                    } else {
                        //Check and see where our shot *would* hit... if it's not close to the enemy (within 256?), then don't fire
                    }
                }
            } else {
                enemyCS = qfalse; //not true, but should stop us from firing
            }
        }
    } else if gi_inPVS(
        (*(*NPC).enemy).currentOrigin.as_ptr(),
        (*NPC).currentOrigin.as_ptr(),
    ) != qfalse
    {
        (*NPCInfo).enemyLastSeenTime = (*core::ptr::addr_of!(level)).time;
        faceEnemy = qtrue;
        NPC_AimAdjust(-1); //adjust aim worse longer we cannot see enemy
    }

    if (*(*NPC).client).ps.weapon == WP_NONE {
        faceEnemy = qfalse;
        shoot = qfalse;
    } else {
        if enemyLOS != qfalse {
            //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
            faceEnemy = qtrue;
        }
        if enemyCS != qfalse {
            shoot = qtrue;
        }
    }

    //Check for movement to take care of
    ST_CheckMoveState();

    //See if we should override shooting decision with any special considerations
    ST_CheckFireState();

    if faceEnemy != qfalse {
        //face the enemy
        NPC_FaceEnemy(qtrue);
    }

    if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
        //not supposed to chase my enemies
        if (*NPCInfo).goalEntity == (*NPC).enemy {
            //goal is my entity, so don't move
            r#move = qfalse;
        }
    } else if ((*(*(*NPC).NPC).scriptFlags) & SCF_NO_GROUPS) != 0 {
        //	NPCInfo->goalEntity = UpdateGoal();

        (*NPCInfo).goalEntity = if enemyLOS != qfalse {
            core::ptr::null_mut()
        } else {
            (*NPC).enemy
        };
    }

    if (*(*NPC).client).fireDelay != 0 && (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
        r#move = qfalse;
    }

    if (*core::ptr::addr_of!(ucmd)).rightmove == 0 {
        //only if not already strafing for some strange reason...?
        //NOTE: these are never set here, but can be set in AI_Jedi.cpp for those NPCs who are sort of Stormtrooper/Jedi hybrids
        //NOTE: this stomps navigation movement entirely!
        //FIXME: if enemy behind me and turning to face enemy, don't strafe in that direction, too
        if TIMER_Done(NPC, b"strafeLeft\0".as_ptr() as *const c_char) == qfalse {
            /*
            if ( NPCInfo->desiredYaw > NPC->client->ps.viewangles[YAW] + 60 )
            {//we want to turn left, don't apply the strafing
            }
            else
            */
            {
                //go ahead and strafe left
                (*core::ptr::addr_of_mut!(ucmd)).rightmove = -127;
                //re-check the duck as we might want to be rolling
                // VectorClear( NPC->client->ps.moveDir )
                (*(*NPC).client).ps.moveDir[0] = 0.0;
                (*(*NPC).client).ps.moveDir[1] = 0.0;
                (*(*NPC).client).ps.moveDir[2] = 0.0;
                r#move = qfalse;
            }
        } else if TIMER_Done(NPC, b"strafeRight\0".as_ptr() as *const c_char) == qfalse {
            /*if ( NPCInfo->desiredYaw < NPC->client->ps.viewangles[YAW] - 60 )
            {//we want to turn right, don't apply the strafing
            }
            else
            */
            {
                //go ahead and strafe left
                (*core::ptr::addr_of_mut!(ucmd)).rightmove = 127;
                // VectorClear( NPC->client->ps.moveDir )
                (*(*NPC).client).ps.moveDir[0] = 0.0;
                (*(*NPC).client).ps.moveDir[1] = 0.0;
                (*(*NPC).client).ps.moveDir[2] = 0.0;
                r#move = qfalse;
            }
        }
    }

    if (*(*NPC).client).ps.legsAnim == BOTH_GUARD_LOOKAROUND1 {
        //don't move when doing silly look around thing
        r#move = qfalse;
    }
    if r#move != qfalse {
        //move toward goal
        if !(*NPCInfo).goalEntity.is_null()
        //&& ( NPCInfo->goalEntity != NPC->enemy || enemyDist > 10000 ) )//100 squared
        {
            r#move = ST_Move();
            if ((*(*NPC).client).NPC_class != CLASS_ROCKETTROOPER
                || (*NPC).s.weapon != WP_ROCKET_LAUNCHER
                || enemyDist < MIN_ROCKET_DIST_SQUARED)
                //rockettroopers who use rocket launchers turn around and run if you get too close (closer than 128)
                && (*core::ptr::addr_of!(ucmd)).forwardmove <= -32
            {
                //moving backwards at least 45 degrees
                if !(*NPCInfo).goalEntity.is_null()
                    && DistanceSquared(
                        (*(*NPCInfo).goalEntity).currentOrigin.as_ptr(),
                        (*NPC).currentOrigin.as_ptr(),
                    ) > MIN_TURN_AROUND_DIST_SQ
                {
                    //don't stop running backwards if your goal is less than 100 away
                    if TIMER_Done(NPC, b"runBackwardsDebounce\0".as_ptr() as *const c_char)
                        != qfalse
                    {
                        //not already waiting for next run backwards
                        if TIMER_Exists(NPC, b"runningBackwards\0".as_ptr() as *const c_char)
                            == qfalse
                        {
                            //start running backwards
                            TIMER_Set(
                                NPC,
                                b"runningBackwards\0".as_ptr() as *const c_char,
                                Q_irand(500, 1000),
                            ); //Q_irand( 2000, 3500 ) );
                        } else if TIMER_Done2(
                            NPC,
                            b"runningBackwards\0".as_ptr() as *const c_char,
                            qtrue,
                        ) != qfalse
                        {
                            //done running backwards
                            TIMER_Set(
                                NPC,
                                b"runBackwardsDebounce\0".as_ptr() as *const c_char,
                                Q_irand(3000, 5000),
                            );
                        }
                    }
                }
            } else {
                //not running backwards
                //TIMER_Remove( NPC, "runningBackwards" );
            }
        } else {
            r#move = qfalse;
        }
    }

    if r#move == qfalse {
        if (*(*NPC).client).NPC_class != CLASS_ASSASSIN_DROID {
            if TIMER_Done(NPC, b"duck\0".as_ptr() as *const c_char) == qfalse {
                (*core::ptr::addr_of_mut!(ucmd)).upmove = -127;
            }
        }
        //FIXME: what about leaning?
    } else {
        //stop ducking!
        TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, -1);
    }

    if (*(*NPC).client).NPC_class == CLASS_REBORN //cultist using a gun
        && (*NPCInfo).rank >= RANK_LT_COMM //commando or better
        && (*(*NPC).enemy).s.weapon == WP_SABER
    //fighting a saber-user
    {
        //commando saboteur vs. jedi/reborn
        //see if we need to avoid their saber
        NPC_EvasionSaber();
    }

    if // TIMER_Done( NPC, "flee" ) ||
        (r#move != qfalse
            && TIMER_Done(NPC, b"runBackwardsDebounce\0".as_ptr() as *const c_char) == qfalse)
    {
        //running away
        faceEnemy = qfalse;
    }

    //FIXME: check scf_face_move_dir here?

    if faceEnemy == qfalse {
        //we want to face in the dir we're running
        if r#move == qfalse {
            //if we haven't moved, we should look in the direction we last looked?
            // VectorCopy( NPC->client->ps.viewangles, NPCInfo->lastPathAngles )
            (*NPCInfo).lastPathAngles[0] = (*(*NPC).client).ps.viewangles[0];
            (*NPCInfo).lastPathAngles[1] = (*(*NPC).client).ps.viewangles[1];
            (*NPCInfo).lastPathAngles[2] = (*(*NPC).client).ps.viewangles[2];
        }
        (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[YAW as usize];
        (*NPCInfo).desiredPitch = 0.0;
        NPC_UpdateAngles(qtrue, qtrue);
        if r#move != qfalse {
            //don't run away and shoot
            shoot = qfalse;
        }
    }

    if ((*NPCInfo).scriptFlags & SCF_DONT_FIRE) != 0 {
        shoot = qfalse;
    }

    if !(*NPC).enemy.is_null() && !(*(*NPC).enemy).enemy.is_null() {
        if (*(*NPC).enemy).s.weapon == WP_SABER
            && (*(*(*NPC).enemy).enemy).s.weapon == WP_SABER
        {
            //don't shoot at an enemy jedi who is fighting another jedi, for fear of injuring one or causing rogue blaster deflections (a la Obi Wan/Vader duel at end of ANH)
            shoot = qfalse;
        }
    }
    //FIXME: don't shoot right away!
    if (*(*NPC).client).fireDelay != 0 {
        if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
            Saboteur_Decloak(NPC);
        }
        if (*NPC).s.weapon == WP_ROCKET_LAUNCHER
            || ((*NPC).s.weapon == WP_CONCUSSION
                && ((*NPCInfo).scriptFlags & SCF_ALT_FIRE) == 0)
        {
            if enemyLOS == qfalse || enemyCS == qfalse {
                //cancel it
                (*(*NPC).client).fireDelay = 0;
            } else {
                //delay our next attempt
                TIMER_Set(
                    NPC,
                    b"attackDelay\0".as_ptr() as *const c_char,
                    Q_irand(3000, 5000),
                );
            }
        }
    } else if shoot != qfalse {
        //try to shoot if it's time
        if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
            Saboteur_Decloak(NPC);
        }
        if TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) != qfalse {
            if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) == 0
            // we've already fired, no need to do it again here
            {
                WeaponThink(qtrue);
            }
            //NASTY
            if (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
                if ((*core::ptr::addr_of!(ucmd)).buttons & BUTTON_ATTACK) != 0
                    && r#move == qfalse
                    && (*g_spskill).integer > 1
                    && Q_irand(0, 3) == 0
                {
                    //every now and then, shoot a homing rocket
                    (*core::ptr::addr_of_mut!(ucmd)).buttons &= !BUTTON_ATTACK;
                    (*core::ptr::addr_of_mut!(ucmd)).buttons |= BUTTON_ALT_ATTACK;
                    (*(*NPC).client).fireDelay = Q_irand(1000, 2500);
                }
            } else if (*NPC).s.weapon == WP_NOGHRI_STICK
                && enemyDist < (48 * 48) as vec_t //?
            {
                (*core::ptr::addr_of_mut!(ucmd)).buttons &= !BUTTON_ATTACK;
                (*core::ptr::addr_of_mut!(ucmd)).buttons |= BUTTON_ALT_ATTACK;
                (*(*NPC).client).fireDelay = Q_irand(1500, 2000);
            }
        }
    } else {
        if (*NPC).attackDebounceTime < (*core::ptr::addr_of!(level)).time {
            if (*(*NPC).client).NPC_class == CLASS_SABOTEUR {
                Saboteur_Cloak(NPC);
            }
        }
    }
}

// extern qboolean G_TuskenAttackAnimDamage( gentity_t *self ); -- already declared in shared preamble
pub unsafe fn NPC_BSST_Default() {
    if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) != 0 {
        WeaponThink(qtrue);
    }

    if (*NPC).s.weapon == WP_NOGHRI_STICK {
        if G_TuskenAttackAnimDamage(NPC) != qfalse {
            Noghri_StickTrace();
        }
    }

    if (*NPC).enemy.is_null() {
        //don't have an enemy, look for one
        NPC_BSST_Patrol();
    } else
    //if ( NPC->enemy )
    {
        //have an enemy
        if !(*(*NPC).enemy).client.is_null() //enemy is a client
            && ((*(*(*NPC).enemy).client).NPC_class == CLASS_UGNAUGHT
                || (*(*(*NPC).enemy).client).NPC_class == CLASS_JAWA) //enemy is a lowly jawa or ugnaught
            && (*(*NPC).enemy).enemy != NPC //enemy's enemy is not me
            && ((*(*NPC).enemy).enemy.is_null()
                || (*(*(*NPC).enemy).enemy).client.is_null()
                || ((*(*(*(*NPC).enemy).enemy).client).NPC_class != CLASS_RANCOR
                    && (*(*(*(*NPC).enemy).enemy).client).NPC_class != CLASS_WAMPA))
        //enemy's enemy is not a client or is not a wampa or rancor (which is scarier than me)
        {
            //they should be scared of ME and no-one else
            G_SetEnemy((*NPC).enemy, NPC);
        }
        NPC_CheckGetNewWeapon();
        NPC_BSST_Attack();
    }
}
