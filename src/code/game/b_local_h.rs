// B_local.h
// re-added by MCG

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Includes from original:
// #include "g_local.h"
// #include "say.h"
// #include "AI.h"

// Forward declarations for types defined in included headers
// These types are opaque to this translation but needed for function signatures
#[repr(C)]
pub struct gentity_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct gclient_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct gNPC_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct usercmd_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cvar_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct trace_t {
    _opaque: [u8; 0],
}

pub type vec3_t = [f32; 3];
pub type visibility_t = c_int;
pub type spot_t = c_int;
pub type team_t = c_int;
pub type bState_t = c_int;
pub type qboolean = c_int;

const AI_TIMERS: c_int = 0; // turn on to see print-outs of AI/nav timing
//
// Navigation susbsystem
//

pub const NAVF_DUCK: c_int = 0x00000001;
pub const NAVF_JUMP: c_int = 0x00000002;
pub const NAVF_HOLD: c_int = 0x00000004;
pub const NAVF_SLOW: c_int = 0x00000008;

pub const DEBUG_LEVEL_DETAIL: c_int = 4;
pub const DEBUG_LEVEL_INFO: c_int = 3;
pub const DEBUG_LEVEL_WARNING: c_int = 2;
pub const DEBUG_LEVEL_ERROR: c_int = 1;
pub const DEBUG_LEVEL_NONE: c_int = 0;

pub const MAX_GOAL_REACHED_DIST_SQUARED: c_int = 256; // 16 squared
pub const MIN_ANGLE_ERROR: f32 = 0.01f32;

pub const MIN_ROCKET_DIST_SQUARED: c_int = 16384; // 128*128
//
// NPC.cpp
//
// ai debug cvars
extern "C" {
    pub fn SetNPCGlobals(ent: *mut gentity_t);
    pub fn SaveNPCGlobals();
    pub fn RestoreNPCGlobals();
    pub static mut debugNPCAI: *mut cvar_t; // used to print out debug info about the NPC AI
    pub static mut debugNPCFreeze: *mut cvar_t; // set to disable NPC ai and temporarily freeze them in place
    pub static mut debugNPCName: *mut cvar_t;
    pub static mut d_JediAI: *mut cvar_t;
    pub static mut d_saberCombat: *mut cvar_t;
    pub fn NPC_Think(self_: *mut gentity_t);
    pub fn pitch_roll_for_slope(
        forwhom: *mut gentity_t,
        pass_slope: *mut vec3_t,
        storeAngles: *mut vec3_t,
        keepPitch: qboolean,
    );
}

// NPC_reactions.cpp
extern "C" {
    pub fn NPC_Pain(
        self_: *mut gentity_t,
        inflictor: *mut gentity_t,
        attacker: *mut gentity_t,
        point: *mut vec3_t,
        damage: c_int,
        mod_: c_int,
        hitLoc: c_int,
    );
    pub fn NPC_Touch(
        self_: *mut gentity_t,
        other: *mut gentity_t,
        trace: *mut trace_t,
    );
    pub fn NPC_Use(
        self_: *mut gentity_t,
        other: *mut gentity_t,
        activator: *mut gentity_t,
    );
    pub fn NPC_GetPainChance(self_: *mut gentity_t, damage: c_int) -> f32;
}

//
// NPC_misc.cpp
//
extern "C" {
    pub fn Debug_Printf(cv: *mut cvar_t, level: c_int, fmt: *const c_char, ...);
    pub fn Debug_NPCPrintf(
        printNPC: *mut gentity_t,
        cv: *mut cvar_t,
        debugLevel: c_int,
        fmt: *const c_char,
        ...
    );
}

// MCG - Begin============================================================
// NPC_ai variables - shared by NPC.cpp andf the following modules
extern "C" {
    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut gNPC_t;
    pub static mut client: *mut gclient_t;
    pub static mut ucmd: usercmd_t;
    pub static mut enemyVisibility: visibility_t;
}

// AI_Default
extern "C" {
    pub fn NPC_CheckInvestigate(alertEventNum: c_int) -> qboolean;
    pub fn NPC_StandTrackAndShoot(NPC: *mut gentity_t) -> qboolean;
    pub fn NPC_BSIdle();
    pub fn NPC_BSPointShoot(shoot: qboolean);
    pub fn NPC_BSStandGuard();
    pub fn NPC_BSPatrol();
    pub fn NPC_BSHuntAndKill();
    pub fn NPC_BSStandAndShoot();
    pub fn NPC_BSRunAndShoot();
    pub fn NPC_BSWait();
    pub fn NPC_BSDefault();
}

// NPC_behavior
extern "C" {
    pub fn NPC_BSAdvanceFight();
    pub fn NPC_BSInvestigate();
    pub fn NPC_BSSleep();
    pub fn NPC_BSFollowLeader();
    pub fn NPC_BSJump();
    pub fn NPC_BSRemove();
    pub fn NPC_BSSearch();
    pub fn NPC_BSSearchStart(homeWp: c_int, bState: bState_t);
    pub fn NPC_BSWander();
    pub fn NPC_BSFlee() -> qboolean;
    pub fn NPC_StartFlee(
        enemy: *mut gentity_t,
        dangerPoint: *mut vec3_t,
        dangerLevel: c_int,
        fleeTimeMin: c_int,
        fleeTimeMax: c_int,
    );
    pub fn G_StartFlee(
        self_: *mut gentity_t,
        enemy: *mut gentity_t,
        dangerPoint: *mut vec3_t,
        dangerLevel: c_int,
        fleeTimeMin: c_int,
        fleeTimeMax: c_int,
    );
}

// NPC_combat
extern "C" {
    pub fn ChooseBestWeapon() -> c_int;
    pub fn NPC_ChangeWeapon(newWeapon: c_int);
    pub fn ShootThink();
    pub fn WeaponThink(inCombat: qboolean);
    pub fn HaveWeapon(weapon: c_int) -> qboolean;
    pub fn CanShoot(ent: *mut gentity_t, shooter: *mut gentity_t) -> qboolean;
    pub fn NPC_CheckPossibleEnemy(other: *mut gentity_t, vis: visibility_t);
    pub fn NPC_PickEnemy(
        closestTo: *mut gentity_t,
        enemyTeam: c_int,
        checkVis: qboolean,
        findPlayersFirst: qboolean,
        findClosest: qboolean,
    ) -> *mut gentity_t;
    pub fn NPC_CheckEnemy(
        findNew: qboolean,
        tooFarOk: qboolean,
        setEnemy: qboolean,
    ) -> *mut gentity_t;
    pub fn NPC_CheckAttack(scale: f32) -> qboolean;
    pub fn NPC_CheckDefend(scale: f32) -> qboolean;
    pub fn NPC_CheckCanAttack(attack_scale: f32, stationary: qboolean) -> qboolean;
    pub fn NPC_AttackDebounceForWeapon() -> c_int;
    pub fn EntIsGlass(check: *mut gentity_t) -> qboolean;
    pub fn ShotThroughGlass(
        tr: *mut trace_t,
        target: *mut gentity_t,
        spot: *mut vec3_t,
        mask: c_int,
    ) -> qboolean;
    pub fn G_ClearEnemy(self_: *mut gentity_t);
    pub fn G_SetEnemy(self_: *mut gentity_t, enemy: *mut gentity_t);
    pub fn NPC_PickAlly(
        facingEachOther: qboolean,
        range: f32,
        ignoreGroup: qboolean,
        movingOnly: qboolean,
    ) -> *mut gentity_t;
    pub fn NPC_LostEnemyDecideChase();
    pub fn NPC_MaxDistSquaredForWeapon() -> f32;
    pub fn NPC_EvaluateShot(hit: c_int, glassOK: qboolean) -> qboolean;
    pub fn NPC_ShotEntity(ent: *mut gentity_t, impactPos: *mut vec3_t) -> c_int;
}

// NPC_formation
extern "C" {
    pub fn NPC_SlideMoveToGoal() -> qboolean;
    pub fn NPC_FindClosestTeammate(self_: *mut gentity_t) -> f32;
    pub fn NPC_CalcClosestFormationSpot(self_: *mut gentity_t);
    pub fn G_MaintainFormations(self_: *mut gentity_t);
    pub fn NPC_BSFormation();
    pub fn NPC_CreateFormation(self_: *mut gentity_t);
    pub fn NPC_DropFormation(self_: *mut gentity_t);
    pub fn NPC_ReorderFormation(self_: *mut gentity_t);
    pub fn NPC_InsertIntoFormation(self_: *mut gentity_t);
    pub fn NPC_DeleteFromFormation(self_: *mut gentity_t);
}

pub const COLLISION_RADIUS: c_int = 32;
pub const NUM_POSITIONS: c_int = 30;

// NPC spawnflags
pub const SFB_SMALLHULL: c_int = 1;

pub const SFB_RIFLEMAN: c_int = 2;
pub const SFB_OLDBORG: c_int = 2; // Borg
pub const SFB_PHASER: c_int = 4;
pub const SFB_GUN: c_int = 4; // Borg
pub const SFB_TRICORDER: c_int = 8;
pub const SFB_TASER: c_int = 8; // Borg
pub const SFB_DRILL: c_int = 16; // Borg

pub const SFB_CINEMATIC: c_int = 32;
pub const SFB_NOTSOLID: c_int = 64;
pub const SFB_STARTINSOLID: c_int = 128;

pub const SFB_TROOPERAI: c_int = 1 << 9;

// NPC_goal
extern "C" {
    pub fn SetGoal(goal: *mut gentity_t, rating: f32);
    pub fn NPC_SetGoal(goal: *mut gentity_t, rating: f32);
    pub fn NPC_ClearGoal();
    pub fn NPC_ReachedGoal();
    pub fn ReachedGoal(goal: *mut gentity_t) -> qboolean;
    pub fn UpdateGoal() -> *mut gentity_t;
    pub fn NPC_MoveToGoal(tryStraight: qboolean) -> qboolean;
}

// NPC_move
extern "C" {
    pub fn NPC_Jumping() -> qboolean;
    pub fn NPC_JumpBackingUp() -> qboolean;

    pub fn NPC_TryJump_from_entity(
        goal: *mut gentity_t,
        max_xy_dist: f32,
        max_z_diff: f32,
    ) -> qboolean;
    pub fn NPC_TryJump_from_pos(
        pos: *const vec3_t,
        max_xy_dist: f32,
        max_z_diff: f32,
    ) -> qboolean;
}

// NPC_reactions

// NPC_senses
pub const ALERT_CLEAR_TIME: c_int = 200;
pub const CHECK_PVS: c_int = 1;
pub const CHECK_360: c_int = 2;
pub const CHECK_FOV: c_int = 4;
pub const CHECK_SHOOT: c_int = 8;
pub const CHECK_VISRANGE: c_int = 16;

extern "C" {
    pub fn CanSee(ent: *mut gentity_t) -> qboolean;
    pub fn InFOV_entity(
        ent: *mut gentity_t,
        from: *mut gentity_t,
        hFOV: c_int,
        vFOV: c_int,
    ) -> qboolean;
    pub fn InFOV_pos(
        origin: *const vec3_t,
        from: *mut gentity_t,
        hFOV: c_int,
        vFOV: c_int,
    ) -> qboolean;
    pub fn InFOV_angle(
        spot: *const vec3_t,
        from: *const vec3_t,
        fromAngles: *const vec3_t,
        hFOV: c_int,
        vFOV: c_int,
    ) -> qboolean;
    pub fn NPC_CheckVisibility(ent: *mut gentity_t, flags: c_int) -> visibility_t;
    pub fn InVisrange(ent: *mut gentity_t) -> qboolean;
}

// NPC_sounds
// extern void NPC_AngerSound(void);

// NPC_spawn
extern "C" {
    pub fn NPC_Spawn(self_: *mut gentity_t);
}

// NPC_stats
extern "C" {
    pub fn NPC_ReactionTime() -> c_int;
    pub fn NPC_ParseParms(NPCName: *const c_char, NPC: *mut gentity_t) -> qboolean;
    pub fn NPC_LoadParms();
}

// NPC_utils
extern "C" {
    pub static mut teamNumbers: [c_int; 4]; // TEAM_NUM_TEAMS
    pub static mut teamStrength: [c_int; 4]; // TEAM_NUM_TEAMS
    pub static mut teamCounter: [c_int; 4]; // TEAM_NUM_TEAMS
    pub fn CalcEntitySpot(ent: *const gentity_t, spot: spot_t, point: *mut vec3_t);
    pub fn NPC_UpdateAngles(doPitch: qboolean, doYaw: qboolean) -> qboolean;
    pub fn NPC_UpdateShootAngles(angles: *mut vec3_t, doPitch: qboolean, doYaw: qboolean);
    pub fn NPC_UpdateFiringAngles(doPitch: qboolean, doYaw: qboolean) -> qboolean;
    pub fn SetTeamNumbers();
    pub fn G_ActivateBehavior(self_: *mut gentity_t, bset: c_int) -> qboolean;
    pub fn NPC_AimWiggle(enemy_org: *mut vec3_t);
    pub fn NPC_SetLookTarget(self_: *mut gentity_t, entNum: c_int, clearTime: c_int);
}

// other modules
extern "C" {
    pub fn CalcMuzzlePoint(
        ent: *const gentity_t,
        forward: *mut vec3_t,
        right: *mut vec3_t,
        up: *mut vec3_t,
        muzzlePoint: *mut vec3_t,
        lead_in: f32,
    );
}

// g_combat
extern "C" {
    pub fn ExplodeDeath(self_: *mut gentity_t);
    pub fn ExplodeDeath_Wait(
        self_: *mut gentity_t,
        inflictor: *mut gentity_t,
        attacker: *mut gentity_t,
        damage: c_int,
        meansOfDeath: c_int,
        dFlags: c_int,
        hitLoc: c_int,
    );
    pub fn GoExplodeDeath(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn IdealDistance(self_: *mut gentity_t) -> f32;
}

// g_client
extern "C" {
    pub fn SpotWouldTelefrag(spot: *mut gentity_t, checkteam: team_t) -> qboolean;
}

// g_squad
extern "C" {
    pub fn NPC_SetSayState(self_: *mut gentity_t, to: *mut gentity_t, saying: c_int);
}

// g_utils
extern "C" {
    pub fn G_CheckInSolid(self_: *mut gentity_t, fix: qboolean) -> qboolean;
    pub fn infront(from: *mut gentity_t, to: *mut gentity_t) -> qboolean;
}

// MCG - End============================================================

// NPC.cpp
extern "C" {
    pub fn NPC_SetAnim(
        ent: *mut gentity_t,
        setAnimParts: c_int,
        anim: c_int,
        setAnimFlags: c_int,
        iBlend: c_int,
    );
    pub fn NPC_EnemyTooFar(enemy: *mut gentity_t, dist: f32, toShoot: qboolean) -> qboolean;
}

// ==================================================================

extern "C" {
    pub fn G_ClearLOS(npc: *const gentity_t, start: *const vec3_t, end: *const vec3_t) -> qboolean;
}

// NPC_ClearLOS inline function suite
#[inline]
pub fn NPC_ClearLOS_start_end(start: *const vec3_t, end: *const vec3_t) -> qboolean {
    unsafe { G_ClearLOS(NPC, start, end) }
}

#[inline]
pub fn NPC_ClearLOS_end(end: *const vec3_t) -> qboolean {
    unsafe { G_ClearLOS(NPC, end, end) }
}

#[inline]
pub fn NPC_ClearLOS_ent(ent: *const gentity_t) -> qboolean {
    unsafe { G_ClearLOS(NPC, ent as *const vec3_t, ent as *const vec3_t) }
}

#[inline]
pub fn NPC_ClearLOS_start_ent(start: *const vec3_t, ent: *const gentity_t) -> qboolean {
    unsafe { G_ClearLOS(NPC, start, ent as *const vec3_t) }
}

#[inline]
pub fn NPC_ClearLOS_ent_end(ent: *const gentity_t, end: *const vec3_t) -> qboolean {
    unsafe { G_ClearLOS(NPC, ent as *const vec3_t, end) }
}

extern "C" {
    pub fn NPC_ClearShot(ent: *mut gentity_t) -> qboolean;

    pub fn NPC_FindCombatPoint(
        position: *const vec3_t,
        avoidPosition: *const vec3_t,
        enemyPosition: *mut vec3_t,
        flags: c_int,
        avoidDist: f32,
        ignorePoint: c_int,
    ) -> c_int;

    pub fn NPC_FindCombatPointRetry(
        position: *const vec3_t,
        avoidPosition: *const vec3_t,
        enemyPosition: *mut vec3_t,
        cpFlags: *mut c_int,
        avoidDist: f32,
        ignorePoint: c_int,
    ) -> c_int;

    pub fn NPC_ReserveCombatPoint(combatPointID: c_int) -> qboolean;
    pub fn NPC_FreeCombatPoint(combatPointID: c_int, failed: qboolean) -> qboolean;
    pub fn NPC_SetCombatPoint(combatPointID: c_int) -> qboolean;
}

pub const CP_ANY: c_int = 0; // No flags
pub const CP_COVER: c_int = 0x00000001; // The enemy cannot currently shoot this position
pub const CP_CLEAR: c_int = 0x00000002; // This cover point has a clear shot to the enemy
pub const CP_FLEE: c_int = 0x00000004; // This cover point is marked as a flee point
pub const CP_DUCK: c_int = 0x00000008; // This cover point is marked as a duck point
pub const CP_NEAREST: c_int = 0x00000010; // Find the nearest combat point
pub const CP_AVOID_ENEMY: c_int = 0x00000020; // Avoid our enemy
pub const CP_INVESTIGATE: c_int = 0x00000040; // A special point worth enemy investigation if searching
pub const CP_SQUAD: c_int = 0x00000080; // Squad path
pub const CP_AVOID: c_int = 0x00000100; // Avoid supplied position
pub const CP_APPROACH_ENEMY: c_int = 0x00000200; // Try to get closer to enemy
pub const CP_CLOSEST: c_int = 0x00000400; // Take the closest combatPoint to the enemy that's available
pub const CP_FLANK: c_int = 0x00000800; // Pick a combatPoint behind the enemy
pub const CP_HAS_ROUTE: c_int = 0x00001000; // Pick a combatPoint that we have a route to
pub const CP_SNIPE: c_int = 0x00002000; // Pick a combatPoint that is marked as a sniper spot
pub const CP_SAFE: c_int = 0x00004000; // Pick a combatPoint that is not have dangerTime
pub const CP_HORZ_DIST_COLL: c_int = 0x00008000; // Collect combat points within *horizontal* dist
pub const CP_NO_PVS: c_int = 0x00010000; // A combat point out of the PVS of enemy pos
pub const CP_RETREAT: c_int = 0x00020000; // Try to get farther from enemy
pub const CP_SHORTEST_PATH: c_int = 0x0004000; // Shortest path from me to combat point to enemy
pub const CP_TRYFAR: c_int = 0x00080000;

pub const CPF_NONE: c_int = 0;
pub const CPF_DUCK: c_int = 0x00000001;
pub const CPF_FLEE: c_int = 0x00000002;
pub const CPF_INVESTIGATE: c_int = 0x00000004;
pub const CPF_SQUAD: c_int = 0x00000008;
pub const CPF_LEAN: c_int = 0x00000010;
pub const CPF_SNIPE: c_int = 0x00000020;

pub const MAX_COMBAT_POINT_CHECK: c_int = 32;

extern "C" {
    pub fn NPC_ValidEnemy(ent: *mut gentity_t) -> c_int;
    pub fn NPC_CheckEnemyExt(checkAlerts: qboolean) -> c_int;
    pub fn NPC_FindPlayer() -> qboolean;
    pub fn NPC_CheckCanAttackExt() -> qboolean;

    pub fn NPC_CheckAlertEvents(
        checkSight: qboolean,
        checkSound: qboolean,
        ignoreAlert: c_int,
        mustHaveOwner: qboolean,
        minAlertLevel: c_int,
        onGroundOnly: qboolean,
    ) -> c_int;
    pub fn NPC_CheckForDanger(alertEvent: c_int) -> qboolean;
    pub fn G_AlertTeam(
        victim: *mut gentity_t,
        attacker: *mut gentity_t,
        radius: f32,
        soundDist: f32,
    );

    pub fn NPC_FindSquadPoint(position: *mut vec3_t) -> c_int;

    pub fn ClearPlayerAlertEvents();

    pub fn G_BoundsOverlap(
        mins1: *const vec3_t,
        maxs1: *const vec3_t,
        mins2: *const vec3_t,
        maxs2: *const vec3_t,
    ) -> qboolean;

    pub fn NPC_SetMoveGoal(
        ent: *mut gentity_t,
        point: *mut vec3_t,
        radius: c_int,
        isNavGoal: qboolean,
        combatPoint: c_int,
        targetEnt: *mut gentity_t,
    );

    pub fn NPC_ApplyWeaponFireDelay();

    // NPC_FaceXXX suite
    pub fn NPC_FacePosition(position: *mut vec3_t, doPitch: qboolean) -> qboolean;
    pub fn NPC_FaceEntity(ent: *mut gentity_t, doPitch: qboolean) -> qboolean;
    pub fn NPC_FaceEnemy(doPitch: qboolean) -> qboolean;

    // Skill level cvar
    pub static mut g_spskill: *mut cvar_t;
}

pub const NIF_NONE: c_int = 0x00000000;
pub const NIF_FAILED: c_int = 0x00000001; // failed to find a way to the goal
pub const NIF_MACRO_NAV: c_int = 0x00000002; // using macro navigation
pub const NIF_COLLISION: c_int = 0x00000004; // resolving collision with an entity
pub const NIF_BLOCKED: c_int = 0x00000008; // blocked from moving

/*
-------------------------
struct navInfo_s
-------------------------
*/

#[repr(C)]
pub struct navInfo_s {
    pub blocker: *mut gentity_t,
    pub direction: vec3_t,
    pub pathDirection: vec3_t,
    pub distance: f32,
    pub trace: trace_t,
    pub flags: c_int,
}

pub type navInfo_t = navInfo_s;
