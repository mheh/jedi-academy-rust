//! Slice of `b_local.h` — the NPC/AI *private* header (the `extern` globals and
//! combat-point flag macros shared across the NPC track).
//!
//! Carries the two combat-point bit-flag families used by the `NPC_combat.c`
//! combat-point finder:
//! - `CP_*` — the *request* flags passed to `NPC_FindCombatPoint` / collected
//!   against (`level.combatPoints[i].flags`).
//! - `CPF_*` — the *point* flags stored on each `combatPoint_t`.
//!
//! Mirrors upstream `codemp/game/b_local.h:243-269`.

#![allow(dead_code)] // flag set lands ahead of every consumer

use core::ffi::c_int;

// b_local.h:31 — minimum squared distance before a Stormtrooper commander will
// order a rocket (compared against enemyDist, a vec_t/f32 squared distance).
pub const MIN_ROCKET_DIST_SQUARED: f32 = 16384.0; // 128*128

// Combat-point request flags (b_local.h:243-261) — passed to NPC_FindCombatPoint
// and tested while collecting candidate points.
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

// Combat-point stored flags (b_local.h:263-269) — held in combatPoint_t::flags.
pub const CPF_NONE: c_int = 0;
pub const CPF_DUCK: c_int = 0x00000001;
pub const CPF_FLEE: c_int = 0x00000002;
pub const CPF_INVESTIGATE: c_int = 0x00000004;
pub const CPF_SQUAD: c_int = 0x00000008;
pub const CPF_LEAN: c_int = 0x00000010;
pub const CPF_SNIPE: c_int = 0x00000020;
