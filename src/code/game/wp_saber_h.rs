#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

pub const ARMOR_EFFECT_TIME: c_int = 500;

pub const JSF_AMBUSH: c_int = 16; //ambusher Jedi

//saberEventFlags
pub const SEF_HITENEMY: c_int = 0x1; //Hit the enemy
pub const SEF_HITOBJECT: c_int = 0x2; //Hit some other object
pub const SEF_HITWALL: c_int = 0x4; //Hit a wall
pub const SEF_PARRIED: c_int = 0x8; //Parried a saber swipe
pub const SEF_DEFLECTED: c_int = 0x10; //Deflected a missile or saberInFlight
pub const SEF_BLOCKED: c_int = 0x20; //Was blocked by a parry
pub const SEF_EVENTS: c_int = (SEF_HITENEMY | SEF_HITOBJECT | SEF_HITWALL | SEF_PARRIED | SEF_DEFLECTED | SEF_BLOCKED);
pub const SEF_LOCKED: c_int = 0x40; //Sabers locked with someone else
pub const SEF_INWATER: c_int = 0x80; //Saber is in water
pub const SEF_LOCK_WON: c_int = 0x100; //Won a saberLock
//saberEntityState
pub const SES_LEAVING: c_int = 1;
pub const SES_HOVERING: c_int = 2;
pub const SES_RETURNING: c_int = 3;

pub const SABER_EXTRAPOLATE_DIST: f32 = 16.0f32;

pub const SABER_MAX_DIST: f32 = 400.0f32;
pub const SABER_MAX_DIST_SQUARED: f32 = (SABER_MAX_DIST * SABER_MAX_DIST);

pub const FORCE_POWER_MAX: c_int = 100;

pub const SABER_REFLECT_MISSILE_CONE: f32 = 0.2f32;

pub const SABER_RADIUS_STANDARD: f32 = 3.0f32;

pub const SABER_LOCK_TIME: c_int = 10000;
pub const SABER_LOCK_DELAYED_TIME: c_int = 9500;

#[repr(C)]
pub enum saberLockResult_t {
    LOCK_VICTORY = 0, //one side won
    LOCK_STALEMATE,   //neither side won
    LOCK_DRAW,        //both people fall back
}

#[repr(C)]
pub enum sabersLockMode_t {
    LOCK_FIRST = 0,
    LOCK_TOP = 0, // LOCK_FIRST
    LOCK_DIAG_TR,
    LOCK_DIAG_TL,
    LOCK_DIAG_BR,
    LOCK_DIAG_BL,
    LOCK_R,
    LOCK_L,
    LOCK_RANDOM,
    LOCK_KYLE_GRAB1,
    LOCK_KYLE_GRAB2,
    LOCK_KYLE_GRAB3,
    LOCK_FORCE_DRAIN,
}

#[repr(C)]
pub enum {
    SABERLOCK_TOP,
    SABERLOCK_SIDE,
    SABERLOCK_LOCK,
    SABERLOCK_BREAK,
    SABERLOCK_SUPERBREAK,
    SABERLOCK_WIN,
    SABERLOCK_LOSE,
}

#[repr(C)]
pub enum {
    DIR_RIGHT,
    DIR_LEFT,
    DIR_FRONT,
    DIR_BACK,
}

pub const FORCE_LIGHTSIDE: c_int = 1;
pub const FORCE_DARKSIDE: c_int = 2;

pub const MAX_FORCE_HEAL_HARD: c_int = 25;
pub const MAX_FORCE_HEAL_MEDIUM: c_int = 50;
pub const MAX_FORCE_HEAL_EASY: c_int = 75;
pub const FORCE_HEAL_INTERVAL: c_int = 200; //FIXME: maybe level 1 is slower or level 2 is faster?

pub const FORCE_GRIP_3_MIN_DIST: f32 = 128.0f32;
pub const FORCE_GRIP_3_MAX_DIST: f32 = 256.0f32;
pub const FORCE_GRIP_DIST: f32 = 512.0f32; //FIXME: vary by power level?
pub const FORCE_GRIP_DIST_SQUARED: f32 = (FORCE_GRIP_DIST * FORCE_GRIP_DIST);

pub const FORCE_DRAIN_DIST: f32 = 64.0f32; //FIXME: vary by power level?
pub const FORCE_DRAIN_DIST_SQUARED: f32 = (FORCE_DRAIN_DIST * FORCE_DRAIN_DIST);

pub const MAX_DRAIN_DISTANCE: c_int = 512;

pub const MIN_SABERBLADE_DRAW_LENGTH: f32 = 0.5f32;

pub const STAFF_KICK_RANGE: c_int = 16;

pub const JUMP_OFF_WALL_SPEED: f32 = 200.0f32;

pub const FORCE_LONG_LEAP_SPEED: f32 = 475.0f32; //300

//#define DUAL_SPIN_PROTECT_POWER	50	//power required to do the dual spin attack
//#define SINGLE_SPECIAL_POWER	20	//power required to do the single saber special attacks

pub const SABER_ALT_ATTACK_POWER: c_int = 50; //75?
pub const SABER_ALT_ATTACK_POWER_LR: c_int = 10; //30?
pub const SABER_ALT_ATTACK_POWER_FB: c_int = 25; //30/50?

pub const FORCE_LONGJUMP_POWER: c_int = 20;

pub const WALL_RUN_UP_BACKFLIP_SPEED: f32 = -150.0f32; //was -300.0f
pub const MAX_WALL_RUN_Z_NORMAL: f32 = 0.4f32; //was 0.0f

pub const PLAYER_KNOCKDOWN_HOLD_EXTRA_TIME: c_int = 4000; //player stays down after a knockdown for 4 whole seconds before automatically doing one of the slow get-ups

pub const MAX_WALL_GRAB_SLOPE: f32 = 0.2f32;

//"Matrix" effect flags
pub const MEF_NO_TIMESCALE: c_int = 0x000001; //no timescale
pub const MEF_NO_VERTBOB: c_int = 0x000002; //no vert bob
pub const MEF_NO_SPIN: c_int = 0x000004; //no spin
pub const MEF_NO_RANGEVAR: c_int = 0x000008; //no camera range variation
pub const MEF_HIT_GROUND_STOP: c_int = 0x000010; //stop effect when subject hits the ground
pub const MEF_REVERSE_SPIN: c_int = 0x000020; //spin counter-clockwise instead of clockwise
pub const MEF_MULTI_SPIN: c_int = 0x000040; //spin once every second, until the effect stops
pub const MEF_LOOK_AT_ENEMY: c_int = 0x000200;

pub const SABER_PITCH_HACK: c_int = 90;

extern "C" {
    pub static forceJumpStrength: [f32; 0];
    pub static forceJumpHeight: [f32; 0];
    pub static forceJumpHeightMax: [f32; 0];

    pub static forcePushPullRadius: [f32; 0];

    pub fn ForceSpeed(self_: *mut gentity_t, duration: c_int);
    pub static forceSpeedValue: [f32; 0];
    pub static forceSpeedRangeMod: [f32; 0];
    pub static forceSpeedFOVMod: [f32; 0];
    pub static saberColorStringForColor: [*mut c_char; 0];
}

pub const FORCE_SPEED_DURATION: f32 = 10000.0f32;
pub const FORCE_RAGE_DURATION: f32 = 10000.0f32;

pub const MASK_FORCE_PUSH: c_int = (MASK_OPAQUE | CONTENTS_SOLID);

#[repr(C)]
pub enum forcePower_t {
    FORCE_LEVEL_0 = 0,
    FORCE_LEVEL_1,
    FORCE_LEVEL_2,
    FORCE_LEVEL_3,
    NUM_FORCE_POWER_LEVELS,
}

pub const FORCE_LEVEL_4: c_int = (4); // FORCE_LEVEL_3+1
pub const FORCE_LEVEL_5: c_int = (5); // FORCE_LEVEL_4+1

#[repr(C)]
pub enum forceJump_t {
    FJ_FORWARD = 0,
    FJ_BACKWARD,
    FJ_RIGHT,
    FJ_LEFT,
    FJ_UP,
}

pub const FORCE_JUMP_CHARGE_TIME: f32 = 1000.0f32; //Force jump reaches maximum power in one second

pub const FORCE_POWERS_ROSH_FROM_TWINS: c_int =
    ((1 << 3) | (1 << 6) | (1 << 10) | (1 << 12)); // FP_SPEED, FP_GRIP, FP_RAGE, FP_SABERTHROW

extern "C" {
    pub fn WP_InitForcePowers(ent: *mut gentity_t);
    pub fn WP_GetVelocityForForceJump(self_: *mut gentity_t, jumpVel: *mut f32, ucmd: *mut usercmd_t) -> c_int;
    pub fn WP_SaberInitBladeData(ent: *mut gentity_t) -> c_int;
    pub fn G_CreateG2AttachedWeaponModel(
        ent: *mut gentity_t,
        weaponModel: *const c_char,
        boltNum: c_int,
        weaponNum: c_int,
    );
    pub fn WP_SaberAddG2SaberModels(ent: *mut gentity_t, specificSaberNum: c_int);
    pub fn WP_SaberParseParms(
        SaberName: *const c_char,
        saber: *mut saberInfo_t,
        setColors: c_int,
    ) -> c_int;
    pub fn WP_BreakSaber(
        ent: *mut gentity_t,
        surfName: *const c_char,
        saberType: saberType_t,
    ) -> c_int;
    pub fn ForceThrow(self_: *mut gentity_t, pull: c_int, fake: c_int);
    pub fn G_GetHitLocFromSurfName(
        ent: *mut gentity_t,
        surfName: *const c_char,
        hitLoc: *mut c_int,
        point: *mut f32,
        dir: *mut f32,
        bladeDir: *mut f32,
        mod_: c_int,
        saberType: saberType_t,
    ) -> c_int;
    pub fn G_CheckEnemyPresence(
        ent: *mut gentity_t,
        dir: c_int,
        radius: f32,
        tolerance: f32,
    ) -> c_int;
    pub fn WP_SaberFreeStrings(saber: *mut saberInfo_t);
    pub fn G_EnoughPowerForSpecialMove(forcePower: c_int, cost: c_int, kataMove: c_int) -> c_int;
    pub fn G_DrainPowerForSpecialMove(
        self_: *mut gentity_t,
        fp: forcePowers_t,
        cost: c_int,
        kataMove: c_int,
    );
    pub fn G_CostForSpecialMove(cost: c_int, kataMove: c_int) -> c_int;
    pub fn G_DropSaberItem(
        saberType: *const c_char,
        saberColor: saber_colors_t,
        saberPos: *mut f32,
        saberVel: *mut f32,
        saberAngles: *mut f32,
        copySaber: *mut gentity_t,
    ) -> *mut gentity_t;
}

#[repr(C)]
pub enum evasionType_t {
    EVASION_NONE = 0,
    EVASION_PARRY,
    EVASION_DUCK_PARRY,
    EVASION_JUMP_PARRY,
    EVASION_DODGE,
    EVASION_JUMP,
    EVASION_DUCK,
    EVASION_FJUMP,
    EVASION_CARTWHEEL,
    EVASION_OTHER,
    NUM_EVASION_TYPES,
}

#[repr(C)]
pub enum swingType_t {
    SWING_FAST = 0,
    SWING_MEDIUM,
    SWING_STRONG,
}

// Okay, here lies the much-dreaded Pat-created FSM movement chart...  Heretic II strikes again!
// Why am I inflicting this on you?  Well, it's better than hardcoded states.
// Ideally this will be replaced with an external file or more sophisticated move-picker
// once the game gets out of prototype stage. <- HAHA!

#[repr(C)]
pub enum saberMoveName_t {
    // Invalid, or saber not armed
    LS_INVALID = -1,
    LS_NONE = 0,

    // General movements with saber
    LS_READY,
    LS_DRAW,
    LS_PUTAWAY,

    // Attacks
    LS_A_TL2BR, //4
    LS_A_L2R,
    LS_A_BL2TR,
    LS_A_BR2TL,
    LS_A_R2L,
    LS_A_TR2BL,
    LS_A_T2B,
    LS_A_BACKSTAB,
    LS_A_BACK,
    LS_A_BACK_CR,
    LS_ROLL_STAB,
    LS_A_LUNGE,
    LS_A_JUMP_T__B_,
    LS_A_FLIP_STAB,
    LS_A_FLIP_SLASH,
    LS_JUMPATTACK_DUAL,
    LS_JUMPATTACK_ARIAL_LEFT,
    LS_JUMPATTACK_ARIAL_RIGHT,
    LS_JUMPATTACK_CART_LEFT,
    LS_JUMPATTACK_CART_RIGHT,
    LS_JUMPATTACK_STAFF_LEFT,
    LS_JUMPATTACK_STAFF_RIGHT,
    LS_BUTTERFLY_LEFT,
    LS_BUTTERFLY_RIGHT,
    LS_A_BACKFLIP_ATK,
    LS_SPINATTACK_DUAL,
    LS_SPINATTACK,
    LS_LEAP_ATTACK,
    LS_SWOOP_ATTACK_RIGHT,
    LS_SWOOP_ATTACK_LEFT,
    LS_TAUNTAUN_ATTACK_RIGHT,
    LS_TAUNTAUN_ATTACK_LEFT,
    LS_KICK_F,
    LS_KICK_B,
    LS_KICK_R,
    LS_KICK_L,
    LS_KICK_S,
    LS_KICK_BF,
    LS_KICK_RL,
    LS_KICK_F_AIR,
    LS_KICK_B_AIR,
    LS_KICK_R_AIR,
    LS_KICK_L_AIR,
    LS_STABDOWN,
    LS_STABDOWN_STAFF,
    LS_STABDOWN_DUAL,
    LS_DUAL_SPIN_PROTECT,
    LS_STAFF_SOULCAL,
    LS_A1_SPECIAL,
    LS_A2_SPECIAL,
    LS_A3_SPECIAL,
    LS_UPSIDE_DOWN_ATTACK,
    LS_PULL_ATTACK_STAB,
    LS_PULL_ATTACK_SWING,
    LS_SPINATTACK_ALORA,
    LS_DUAL_FB,
    LS_DUAL_LR,
    LS_HILT_BASH,

    //starts
    LS_S_TL2BR, //26
    LS_S_L2R,
    LS_S_BL2TR, //# Start of attack chaining to SLASH LR2UL
    LS_S_BR2TL, //# Start of attack chaining to SLASH LR2UL
    LS_S_R2L,
    LS_S_TR2BL,
    LS_S_T2B,

    //returns
    LS_R_TL2BR, //33
    LS_R_L2R,
    LS_R_BL2TR,
    LS_R_BR2TL,
    LS_R_R2L,
    LS_R_TR2BL,
    LS_R_T2B,

    //transitions
    LS_T1_BR__R, //40
    LS_T1_BR_TR,
    LS_T1_BR_T_,
    LS_T1_BR_TL,
    LS_T1_BR__L,
    LS_T1_BR_BL,
    LS_T1__R_BR, //46
    LS_T1__R_TR,
    LS_T1__R_T_,
    LS_T1__R_TL,
    LS_T1__R__L,
    LS_T1__R_BL,
    LS_T1_TR_BR, //52
    LS_T1_TR__R,
    LS_T1_TR_T_,
    LS_T1_TR_TL,
    LS_T1_TR__L,
    LS_T1_TR_BL,
    LS_T1_T__BR, //58
    LS_T1_T___R,
    LS_T1_T__TR,
    LS_T1_T__TL,
    LS_T1_T___L,
    LS_T1_T__BL,
    LS_T1_TL_BR, //64
    LS_T1_TL__R,
    LS_T1_TL_TR,
    LS_T1_TL_T_,
    LS_T1_TL__L,
    LS_T1_TL_BL,
    LS_T1__L_BR, //70
    LS_T1__L__R,
    LS_T1__L_TR,
    LS_T1__L_T_,
    LS_T1__L_TL,
    LS_T1__L_BL,
    LS_T1_BL_BR, //76
    LS_T1_BL__R,
    LS_T1_BL_TR,
    LS_T1_BL_T_,
    LS_T1_BL_TL,
    LS_T1_BL__L,

    //Bounces
    LS_B1_BR,
    LS_B1__R,
    LS_B1_TR,
    LS_B1_T_,
    LS_B1_TL,
    LS_B1__L,
    LS_B1_BL,

    //Deflected attacks
    LS_D1_BR,
    LS_D1__R,
    LS_D1_TR,
    LS_D1_T_,
    LS_D1_TL,
    LS_D1__L,
    LS_D1_BL,
    LS_D1_B_,

    //Reflected attacks
    LS_V1_BR,
    LS_V1__R,
    LS_V1_TR,
    LS_V1_T_,
    LS_V1_TL,
    LS_V1__L,
    LS_V1_BL,
    LS_V1_B_,

    // Broken parries
    LS_H1_T_, //
    LS_H1_TR,
    LS_H1_TL,
    LS_H1_BR,
    LS_H1_B_,
    LS_H1_BL,

    // Knockaways
    LS_K1_T_, //
    LS_K1_TR,
    LS_K1_TL,
    LS_K1_BR,
    LS_K1_BL,

    // Parries
    LS_PARRY_UP, //
    LS_PARRY_UR,
    LS_PARRY_UL,
    LS_PARRY_LR,
    LS_PARRY_LL,

    // Projectile Reflections
    LS_REFLECT_UP, //
    LS_REFLECT_UR,
    LS_REFLECT_UL,
    LS_REFLECT_LR,
    LS_REFLECT_LL,

    LS_MOVE_MAX, //
}

extern "C" {
    pub fn PM_SetSaberMove(newMove: saberMoveName_t);
}

#[repr(C)]
pub enum saberQuadrant_t {
    Q_BR = 0,
    Q_R,
    Q_TR,
    Q_T,
    Q_TL,
    Q_L,
    Q_BL,
    Q_B,
    Q_NUM_QUADS,
}

#[repr(C)]
pub struct saberMoveData_t {
    pub name: *mut c_char,
    pub animToUse: c_int,
    pub startQuad: c_int,
    pub endQuad: c_int,
    pub animSetFlags: c_int,
    pub blendTime: c_int,
    pub blocking: c_int,
    pub chain_idle: saberMoveName_t,      // What move to call if the attack button is not pressed at the end of this anim
    pub chain_attack: saberMoveName_t,    // What move to call if the attack button (and nothing else) is pressed
    pub trailLength: c_int,
}

extern "C" {
    pub static saberMoveData: [saberMoveData_t; 0];
}

// Forward declarations of types used in function signatures
// These are unresolved stubs; actual definitions are elsewhere in the codebase
pub enum gentity_t {}
pub enum usercmd_t {}
pub enum saberInfo_t {}
pub enum saberType_t {}
pub enum forcePowers_t {}
pub enum saber_colors_t {}

// Stub for MASK_OPAQUE and CONTENTS_SOLID constants
// These would be defined in a broader qcommon module
const MASK_OPAQUE: c_int = 0;
const CONTENTS_SOLID: c_int = 0;
