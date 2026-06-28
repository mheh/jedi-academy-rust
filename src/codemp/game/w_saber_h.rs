//! `w_saber.h` — saber / force-combat constants shared across the bg layer.
//!
//! A header of `#define` tunables (saber-event flags, saber-entity states, force-
//! power limits, saber bounding box) plus two enums — the anonymous `FJ_*`
//! force-jump directions and `evasionType_t` — and four `extern` declarations of
//! the force-tuning tables.
//!
//! Those tables (`forcePowerNeeded`/`forceJumpHeight`/`forceJumpStrength`/
//! `forcePushPullRadius`) are *defined* in `bg_pmove.c`, so — per the `bg_local.h`
//! precedent — only their declarations live here; they are carried at the bottom
//! as not-yet-ported source-order comments and wired in when bg_pmove.c lands. (The same
//! four also appear, with the same not-yet-ported status, in [`super::bg_local_h`].)
//!
//! Pure constants + enums (no struct layout). Both `typedef enum {…};` blocks are
//! the JKA no-tag/no-typedef-name quirk, so — like `forcePowers_t` (FP_*) in
//! [`super::q_shared_h`] — the enumerators become plain `c_int` consts (with an
//! `evasionType_t = c_int` alias for the one named type). Values are C-oracle-
//! verified (`oracle/w_saber_h_oracle.c` `#include`s the authentic header).

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// `ARMOR_EFFECT_TIME` (w_saber.h).
pub const ARMOR_EFFECT_TIME: c_int = 500;

// saberEventFlags
pub const SEF_HITENEMY: c_int = 0x1; // Hit the enemy
pub const SEF_HITOBJECT: c_int = 0x2; // Hit some other object
pub const SEF_HITWALL: c_int = 0x4; // Hit a wall
pub const SEF_PARRIED: c_int = 0x8; // Parried a saber swipe
pub const SEF_DEFLECTED: c_int = 0x10; // Deflected a missile or saberInFlight
pub const SEF_BLOCKED: c_int = 0x20; // Was blocked by a parry
pub const SEF_EVENTS: c_int =
    SEF_HITENEMY | SEF_HITOBJECT | SEF_HITWALL | SEF_PARRIED | SEF_DEFLECTED | SEF_BLOCKED;
pub const SEF_LOCKED: c_int = 0x40; // Sabers locked with someone else
pub const SEF_INWATER: c_int = 0x80; // Saber is in water
pub const SEF_LOCK_WON: c_int = 0x100; // Won a saberLock

// saberEntityState
pub const SES_LEAVING: c_int = 1;
pub const SES_HOVERING: c_int = 1; //2
pub const SES_RETURNING: c_int = 1; //3
                                    // This is a hack because ATM the saberEntityState is only non-0 if out or 0 if in, and we
                                    // at least want NPCs knowing when their saber is out regardless.

pub const JSF_AMBUSH: c_int = 16; // ambusher Jedi

pub const SABER_RADIUS_STANDARD: f32 = 3.0;
pub const SABER_REFLECT_MISSILE_CONE: f32 = 0.2;

pub const FORCE_POWER_MAX: c_int = 100;
pub const MAX_GRIP_DISTANCE: c_int = 256;
pub const MAX_TRICK_DISTANCE: c_int = 512;
pub const FORCE_JUMP_CHARGE_TIME: c_int = 6400; //3000.0f
pub const GRIP_DRAIN_AMOUNT: c_int = 30;
pub const FORCE_LIGHTNING_RADIUS: c_int = 300;
pub const MAX_DRAIN_DISTANCE: c_int = 512;

// Anonymous enum (no tag/typedef name) — force-jump directions; plain `int` consts.
pub const FJ_FORWARD: c_int = 0;
pub const FJ_BACKWARD: c_int = 1;
pub const FJ_RIGHT: c_int = 2;
pub const FJ_LEFT: c_int = 3;
pub const FJ_UP: c_int = 4;

/// `evasionType_t` (w_saber.h) — how an attack was evaded.
pub type evasionType_t = c_int;
pub const EVASION_NONE: evasionType_t = 0;
pub const EVASION_PARRY: evasionType_t = 1;
pub const EVASION_DUCK_PARRY: evasionType_t = 2;
pub const EVASION_JUMP_PARRY: evasionType_t = 3;
pub const EVASION_DODGE: evasionType_t = 4;
pub const EVASION_JUMP: evasionType_t = 5;
pub const EVASION_DUCK: evasionType_t = 6;
pub const EVASION_FJUMP: evasionType_t = 7;
pub const EVASION_CARTWHEEL: evasionType_t = 8;
pub const EVASION_OTHER: evasionType_t = 9;
pub const NUM_EVASION_TYPES: evasionType_t = 10;

// PC `w_saber.h` uncomments `extern vmCvar_t g_MaxHolocronCarry;` (the Xbox tree
// kept it commented). The decl points at a cvar *defined* in g_main.c (PC g_main.c
// `vmCvar_t g_MaxHolocronCarry;` + its cvarTable row) — a separate, not-yet-migrated
// TU — so, per the bg_pmove.c force-table precedent, it stays a source-order comment
// here until that static lands; nothing in this header references it:
//   extern vmCvar_t g_MaxHolocronCarry;

pub const SABERMINS_X: f32 = -3.0; //-24.0f
pub const SABERMINS_Y: f32 = -3.0; //-24.0f
pub const SABERMINS_Z: f32 = -3.0; //-8.0f
pub const SABERMAXS_X: f32 = 3.0; //24.0f
pub const SABERMAXS_Y: f32 = 3.0; //24.0f
pub const SABERMAXS_Z: f32 = 3.0; //8.0f
pub const SABER_MIN_THROW_DIST: f32 = 80.0;

// w_saber.h's `extern` declarations of the four force-tuning tables (sized by
// NUM_FORCE_POWER_LEVELS / NUM_FORCE_POWERS from q_shared.h), carried as source-order
// comments per the bg_local.h precedent. Three are *defined* in bg_pmove.c and are
// NOW PORTED in bg_pmove.rs (the data-layer slice); `forcePushPullRadius` is defined
// in another TU (w_force.c, not yet ported) so it stays unported:
//
//   extern int   forcePowerNeeded[NUM_FORCE_POWER_LEVELS][NUM_FORCE_POWERS]; // -> bg_pmove.rs
//   extern float forceJumpHeight[NUM_FORCE_POWER_LEVELS];                    // -> bg_pmove.rs
//   extern float forceJumpStrength[NUM_FORCE_POWER_LEVELS];                  // -> bg_pmove.rs
//   extern float forcePushPullRadius[NUM_FORCE_POWER_LEVELS];               // not yet ported (w_force.c)

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;

    /// Parity: every w_saber.h `#define` and enumerator matches the authentic C
    /// header (`#include`d in `oracle/w_saber_h_oracle.c`). The Rust side is the
    /// `pub const`s above; the C side reads the real macro/enum values, so a
    /// transcription error on either side fails the assert.
    #[test]
    fn w_saber_consts_match_c() {
        unsafe {
            assert_eq!(ARMOR_EFFECT_TIME, jka_ws_ARMOR_EFFECT_TIME());

            // saberEventFlags (incl. the computed SEF_EVENTS composite)
            assert_eq!(SEF_HITENEMY, jka_ws_SEF_HITENEMY());
            assert_eq!(SEF_HITOBJECT, jka_ws_SEF_HITOBJECT());
            assert_eq!(SEF_HITWALL, jka_ws_SEF_HITWALL());
            assert_eq!(SEF_PARRIED, jka_ws_SEF_PARRIED());
            assert_eq!(SEF_DEFLECTED, jka_ws_SEF_DEFLECTED());
            assert_eq!(SEF_BLOCKED, jka_ws_SEF_BLOCKED());
            assert_eq!(SEF_EVENTS, jka_ws_SEF_EVENTS());
            assert_eq!(SEF_LOCKED, jka_ws_SEF_LOCKED());
            assert_eq!(SEF_INWATER, jka_ws_SEF_INWATER());
            assert_eq!(SEF_LOCK_WON, jka_ws_SEF_LOCK_WON());

            // saberEntityState
            assert_eq!(SES_LEAVING, jka_ws_SES_LEAVING());
            assert_eq!(SES_HOVERING, jka_ws_SES_HOVERING());
            assert_eq!(SES_RETURNING, jka_ws_SES_RETURNING());

            assert_eq!(JSF_AMBUSH, jka_ws_JSF_AMBUSH());

            assert_eq!(SABER_RADIUS_STANDARD, jka_ws_SABER_RADIUS_STANDARD());
            assert_eq!(
                SABER_REFLECT_MISSILE_CONE,
                jka_ws_SABER_REFLECT_MISSILE_CONE()
            );

            assert_eq!(FORCE_POWER_MAX, jka_ws_FORCE_POWER_MAX());
            assert_eq!(MAX_GRIP_DISTANCE, jka_ws_MAX_GRIP_DISTANCE());
            assert_eq!(MAX_TRICK_DISTANCE, jka_ws_MAX_TRICK_DISTANCE());
            assert_eq!(FORCE_JUMP_CHARGE_TIME, jka_ws_FORCE_JUMP_CHARGE_TIME());
            assert_eq!(GRIP_DRAIN_AMOUNT, jka_ws_GRIP_DRAIN_AMOUNT());
            assert_eq!(FORCE_LIGHTNING_RADIUS, jka_ws_FORCE_LIGHTNING_RADIUS());
            assert_eq!(MAX_DRAIN_DISTANCE, jka_ws_MAX_DRAIN_DISTANCE());

            // FJ_* force-jump directions (anonymous enum)
            assert_eq!(FJ_FORWARD, jka_ws_FJ_FORWARD());
            assert_eq!(FJ_BACKWARD, jka_ws_FJ_BACKWARD());
            assert_eq!(FJ_RIGHT, jka_ws_FJ_RIGHT());
            assert_eq!(FJ_LEFT, jka_ws_FJ_LEFT());
            assert_eq!(FJ_UP, jka_ws_FJ_UP());

            // evasionType_t
            assert_eq!(EVASION_NONE, jka_ws_EVASION_NONE());
            assert_eq!(EVASION_PARRY, jka_ws_EVASION_PARRY());
            assert_eq!(EVASION_DUCK_PARRY, jka_ws_EVASION_DUCK_PARRY());
            assert_eq!(EVASION_JUMP_PARRY, jka_ws_EVASION_JUMP_PARRY());
            assert_eq!(EVASION_DODGE, jka_ws_EVASION_DODGE());
            assert_eq!(EVASION_JUMP, jka_ws_EVASION_JUMP());
            assert_eq!(EVASION_DUCK, jka_ws_EVASION_DUCK());
            assert_eq!(EVASION_FJUMP, jka_ws_EVASION_FJUMP());
            assert_eq!(EVASION_CARTWHEEL, jka_ws_EVASION_CARTWHEEL());
            assert_eq!(EVASION_OTHER, jka_ws_EVASION_OTHER());
            assert_eq!(NUM_EVASION_TYPES, jka_ws_NUM_EVASION_TYPES());

            // saber bounding box + min throw distance
            assert_eq!(SABERMINS_X, jka_ws_SABERMINS_X());
            assert_eq!(SABERMINS_Y, jka_ws_SABERMINS_Y());
            assert_eq!(SABERMINS_Z, jka_ws_SABERMINS_Z());
            assert_eq!(SABERMAXS_X, jka_ws_SABERMAXS_X());
            assert_eq!(SABERMAXS_Y, jka_ws_SABERMAXS_Y());
            assert_eq!(SABERMAXS_Z, jka_ws_SABERMAXS_Z());
            assert_eq!(SABER_MIN_THROW_DIST, jka_ws_SABER_MIN_THROW_DIST());
        }
    }
}
