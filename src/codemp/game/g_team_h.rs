//! `g_team.h` — team gameplay declarations and CTF constants.

#![allow(non_snake_case)]

use crate::codemp::game::bg_public::team_t;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::q_shared_h::{qboolean, vec3_t};
use core::ffi::{c_char, c_int};

pub const CTF_CAPTURE_BONUS: c_int = 100; // what you get for capture
pub const CTF_TEAM_BONUS: c_int = 25; // what your team gets for capture
pub const CTF_RECOVERY_BONUS: c_int = 10; // what you get for recovery
pub const CTF_FLAG_BONUS: c_int = 10; // what you get for picking up enemy flag
pub const CTF_FRAG_CARRIER_BONUS: c_int = 20; // what you get for fragging enemy flag carrier
pub const CTF_FLAG_RETURN_TIME: c_int = 40000; // seconds until auto return

pub const CTF_CARRIER_DANGER_PROTECT_BONUS: c_int = 5; // bonus for fraggin someone who has recently hurt your flag carrier
pub const CTF_CARRIER_PROTECT_BONUS: c_int = 2; // bonus for fraggin someone while either you or your target are near your flag carrier
pub const CTF_FLAG_DEFENSE_BONUS: c_int = 10; // bonus for fraggin someone while either you or your target are near your flag
pub const CTF_RETURN_FLAG_ASSIST_BONUS: c_int = 10; // awarded for returning a flag that causes a capture to happen almost immediately
pub const CTF_FRAG_CARRIER_ASSIST_BONUS: c_int = 10; // award for fragging a flag carrier if a capture happens almost immediately

pub const CTF_TARGET_PROTECT_RADIUS: c_int = 1000; // the radius around an object being defended where a target will be worth extra frags
pub const CTF_ATTACKER_PROTECT_RADIUS: c_int = 1000; // the radius around an object being defended where an attacker will get extra frags when making kills

pub const CTF_CARRIER_DANGER_PROTECT_TIMEOUT: c_int = 8000;
pub const CTF_FRAG_CARRIER_ASSIST_TIMEOUT: c_int = 10000;
pub const CTF_RETURN_FLAG_ASSIST_TIMEOUT: c_int = 10000;

pub const CTF_GRAPPLE_SPEED: c_int = 750; // speed of grapple in flight
pub const CTF_GRAPPLE_PULL_SPEED: c_int = 750; // speed player is pulled at

pub const OVERLOAD_ATTACK_BASE_SOUND_TIME: c_int = 20000;

unsafe extern "C" {
    pub fn OtherTeam(team: c_int) -> c_int;
    pub fn TeamName(team: c_int) -> *const c_char;
    pub fn OtherTeamName(team: c_int) -> *const c_char;
    pub fn TeamColorString(team: c_int) -> *const c_char;
    pub fn AddTeamScore(origin: *mut vec3_t, team: c_int, score: c_int);

    pub fn Team_DroppedFlagThink(ent: *mut gentity_t);
    pub fn Team_FragBonuses(targ: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t);
    pub fn Team_CheckHurtCarrier(targ: *mut gentity_t, attacker: *mut gentity_t);
    pub fn Team_InitGame();
    pub fn Team_ReturnFlag(team: c_int);
    pub fn Team_FreeEntity(ent: *mut gentity_t);
    pub fn SelectCTFSpawnPoint(
        team: team_t,
        teamstate: c_int,
        origin: *mut vec3_t,
        angles: *mut vec3_t,
    ) -> *mut gentity_t;
    pub fn SelectSiegeSpawnPoint(
        siegeClass: c_int,
        team: team_t,
        teamstate: c_int,
        origin: *mut vec3_t,
        angles: *mut vec3_t,
    ) -> *mut gentity_t;
    pub fn Team_GetLocation(ent: *mut gentity_t) -> *mut gentity_t;
    pub fn Team_GetLocationMsg(ent: *mut gentity_t, loc: *mut c_char, loclen: c_int) -> qboolean;
    pub fn TeamplayInfoMessage(ent: *mut gentity_t);
    pub fn CheckTeamStatus();

    pub fn Pickup_Team(ent: *mut gentity_t, other: *mut gentity_t) -> c_int;
}
