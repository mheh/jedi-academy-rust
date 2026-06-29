// Translated from oracle/code/game/ai.h

use core::ffi::{c_int, c_void};

// Distance ratings
#[repr(C)]
pub enum distance_e {
    DIST_MELEE = 0,
    DIST_LONG = 1,
}

// Attack types
#[repr(C)]
pub enum attack_e {
    ATTACK_MELEE = 0,
    ATTACK_RANGE = 1,
}

// Squad state constants
pub const SQUAD_IDLE: c_int = 0;               // No target found, waiting
pub const SQUAD_STAND_AND_SHOOT: c_int = 1;   // Standing in position and shoot (no cover)
pub const SQUAD_RETREAT: c_int = 2;            // Running away from combat
pub const SQUAD_COVER: c_int = 3;              // Under protective cover
pub const SQUAD_TRANSITION: c_int = 4;         // Moving between points, not firing
pub const SQUAD_POINT: c_int = 5;              // On point, laying down suppressive fire
pub const SQUAD_SCOUT: c_int = 6;              // Poking out to draw enemy
pub const NUM_SQUAD_STATES: c_int = 7;

// sigh... had to move in here for groupInfo
#[repr(C)]
pub enum rank_t {
    RANK_CIVILIAN = 0,
    RANK_CREWMAN = 1,
    RANK_ENSIGN = 2,
    RANK_LT_JG = 3,
    RANK_LT = 4,
    RANK_LT_COMM = 5,
    RANK_COMMANDER = 6,
    RANK_CAPTAIN = 7,
}

// Type stubs for external types needed for structural coherence
pub type qboolean = c_int;
pub type team_t = c_int;
pub type gentity_t = c_void;
pub type vec3_t = [f32; 3];

// Constants
pub const MAX_FRAME_GROUPS: c_int = 32;
pub const MAX_GROUP_MEMBERS: c_int = 32;

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct AIGroupMember_s {
    pub number: c_int,
    pub waypoint: c_int,
    pub pathCostToEnemy: c_int,
    pub closestBuddy: c_int,
}

pub type AIGroupMember_t = AIGroupMember_s;

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct AIGroupInfo_s {
    pub numGroup: c_int,
    pub processed: qboolean,
    pub team: team_t,
    pub enemy: *mut gentity_t,
    pub enemyWP: c_int,
    pub speechDebounceTime: c_int,
    pub lastClearShotTime: c_int,
    pub lastSeenEnemyTime: c_int,
    pub morale: c_int,
    pub moraleAdjust: c_int,
    pub moraleDebounce: c_int,
    pub memberValidateTime: c_int,
    pub activeMemberNum: c_int,
    pub commander: *mut gentity_t,
    pub enemyLastSeenPos: vec3_t,
    pub numState: [c_int; 7],
    pub member: [AIGroupMember_t; 32],
}

pub type AIGroupInfo_t = AIGroupInfo_s;

extern "C" {
    pub fn NPC_CheckPlayerTeamStealth() -> qboolean;

    // AI_GRENADIER
    pub fn NPC_BSGrenadier_Default();

    // AI_TUSKEN
    pub fn NPC_BSTusken_Default();

    // AI_SNIPER
    pub fn NPC_BSSniper_Default();

    // AI_STORMTROOPER
    pub fn Saboteur_Decloak(self_: *mut gentity_t, uncloakTime: c_int);
    pub fn NPC_BSST_Investigate();
    pub fn NPC_BSST_Default();
    pub fn NPC_BSST_Sleep();

    // AI_JEDI
    pub fn NPC_BSJedi_Investigate();
    pub fn NPC_BSJedi_Default();
    pub fn NPC_BSJedi_FollowLeader();

    // AI_DROID
    pub fn NPC_BSDroid_Default();

    // AI_ImperialProbe
    pub fn NPC_BSImperialProbe_Default();

    // AI_atst
    pub fn NPC_BSATST_Default();

    pub fn NPC_BSInterrogator_Default();

    // AI Mark 1
    pub fn NPC_BSMark1_Default();

    // AI Mark 2
    pub fn NPC_BSMark2_Default();

    // monsters
    pub fn NPC_BSMineMonster_Default();
    pub fn NPC_BSHowler_Default();
    pub fn NPC_BSRancor_Default();
    pub fn NPC_BSWampa_Default();
    pub fn NPC_BSSandCreature_Default();

    // animals
    pub fn NPC_BSAnimal_Default();

    // Utilities
    // Group AI
    pub fn AI_GetGroupSize(origin: vec3_t, radius: c_int, playerTeam: team_t, avoid: *mut gentity_t) -> c_int;
    pub fn AI_GetGroupSize_ent(ent: *mut gentity_t, radius: c_int) -> c_int;

    pub fn AI_GetGroup(self_: *mut gentity_t);

    pub fn AI_DistributeAttack(attacker: *mut gentity_t, enemy: *mut gentity_t, team: team_t, threshold: c_int) -> *mut gentity_t;
}
