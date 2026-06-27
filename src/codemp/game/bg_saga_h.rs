//! `bg_saga.h` — the Siege ("saga") gametype data layer shared by the server game,
//! client game, and UI modules (the "BG" = both-games layer).
//!
//! This header defines the Siege class/team data model: the `siegeClass_t` (a single
//! playable class: weapons, force levels, model, saber, health/armor, portrait) and
//! `siegeTeam_t` (a named team owning up to `MAX_SIEGE_CLASSES_PER_TEAM` class
//! pointers), plus the `siegeClassDesc_t` raw-text description buffer and the
//! `SPC_*` / `CFL_*` enums. The defining `.c` is [`bg_saga`].
//!
//! Layout: `siegeClassDesc_t` and `siegeClass_t` are pure POD (char arrays + `int`/
//! `float`/`short`), so their size/offset asserts are arch-independent. `siegeTeam_t`
//! holds an array of `siegeClass_t *`, so its layout is arch-dependent and its size
//! assert is gated `#[cfg(target_pointer_width = "64")]` (the `bg_vehicles_h` pointer-
//! prefix convention), validated against a host-64-bit C oracle.
//!
//! Not ported here, per the header-port convention: the `extern` data globals
//! (`bgSiegeClasses`/`bgNumSiegeClasses`, `bgSiegeTeams`/`bgNumSiegeTeams`,
//! `siege_info`/`siege_valid`) and the `BG_Siege*` prototypes — all land with their
//! defining `.c`, [`bg_saga`].
//!
//! [`bg_saga`]: crate::codemp::game::bg_saga

#![allow(non_camel_case_types, non_snake_case)]

use crate::codemp::game::q_shared_h::{qboolean, MAX_CLIENTS, NUM_FORCE_POWERS};
use core::ffi::{c_char, c_int, c_short};

/// `MAX_SIEGE_INFO_SIZE` (bg_saga.h:1) — size of the `siege_info` map-config buffer.
pub const MAX_SIEGE_INFO_SIZE: usize = 16384;

/// `SIEGETEAM_TEAM1` / `SIEGETEAM_TEAM2` (bg_saga.h:3-4) — the two siege teams
/// (e.g. `TEAM_RED` / `TEAM_BLUE`).
pub const SIEGETEAM_TEAM1: c_int = 1;
pub const SIEGETEAM_TEAM2: c_int = 2;

/// Siege scoring point awards (bg_saga.h:6-8).
pub const SIEGE_POINTS_OBJECTIVECOMPLETED: c_int = 20;
pub const SIEGE_POINTS_FINALOBJECTIVECOMPLETED: c_int = 30;
pub const SIEGE_POINTS_TEAMWONROUND: c_int = 10;

/// `SIEGE_ROUND_BEGIN_TIME` (bg_saga.h:10) — delay 5 secs after players are in game.
pub const SIEGE_ROUND_BEGIN_TIME: c_int = 5000;

/// `MAX_SIEGE_CLASSES` (bg_saga.h:13) — up to 128 classes.
pub const MAX_SIEGE_CLASSES: usize = 128;
/// `MAX_SIEGE_CLASSES_PER_TEAM` (bg_saga.h:14).
pub const MAX_SIEGE_CLASSES_PER_TEAM: usize = 16;
/// `MAX_SIEGE_TEAMS` (bg_saga.h:16) — up to 16 different teams.
pub const MAX_SIEGE_TEAMS: usize = 16;

/// `MAX_EXDATA_ENTS_TO_SEND` (bg_saga.h:18) — max number of extended data for ents
/// to send. Defined as `MAX_CLIENTS` in the C.
pub const MAX_EXDATA_ENTS_TO_SEND: usize = MAX_CLIENTS;

// The basic siege player classes.
/// `siegePlayerClassFlags_t` (bg_saga.h:21) — ported as a C-`int`-width alias +
/// consts (the `g_public_h` enum convention).
pub type siegePlayerClassFlags_t = c_int;
pub const SPC_INFANTRY: siegePlayerClassFlags_t = 0;
pub const SPC_VANGUARD: siegePlayerClassFlags_t = 1;
pub const SPC_SUPPORT: siegePlayerClassFlags_t = 2;
pub const SPC_JEDI: siegePlayerClassFlags_t = 3;
pub const SPC_DEMOLITIONIST: siegePlayerClassFlags_t = 4;
pub const SPC_HEAVY_WEAPONS: siegePlayerClassFlags_t = 5;
pub const SPC_MAX: siegePlayerClassFlags_t = 6;

/// `siegeClassFlags_t` (bg_saga.h:32) — the `CFL_*` class-ability flags. The C
/// enum is plain (`0,1,2,…`); the values are used as bit positions via `(1<<CFL_x)`
/// in the parsing code.
pub type siegeClassFlags_t = c_int;
pub const CFL_MORESABERDMG: siegeClassFlags_t = 0;
pub const CFL_STRONGAGAINSTPHYSICAL: siegeClassFlags_t = 1;
pub const CFL_FASTFORCEREGEN: siegeClassFlags_t = 2;
pub const CFL_STATVIEWER: siegeClassFlags_t = 3;
pub const CFL_HEAVYMELEE: siegeClassFlags_t = 4;
pub const CFL_SINGLE_ROCKET: siegeClassFlags_t = 5; //has only 1 rocket to use with rocketlauncher
pub const CFL_CUSTOMSKEL: siegeClassFlags_t = 6; //class uses a custom skeleton, be sure to load on server etc
pub const CFL_EXTRA_AMMO: siegeClassFlags_t = 7;

/// `SIEGE_CLASS_DESC_LEN` (bg_saga.h:48) — non-`_XBOX` value (the `_XBOX` 512 form
/// is excluded, matching the rest of the port).
pub const SIEGE_CLASS_DESC_LEN: usize = 4096;

/// `siegeClassDesc_t` (bg_saga.h:50) — the raw class-file text buffer handed to
/// `BG_SiegeParseClassFile`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct siegeClassDesc_t {
    pub desc: [c_char; SIEGE_CLASS_DESC_LEN],
}
const _: () = assert!(core::mem::size_of::<siegeClassDesc_t>() == 4096);

/// `siegeClass_t` (bg_saga.h:55) — a single playable Siege class. Pure POD (no
/// pointers), so the layout is arch-independent.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct siegeClass_t {
    pub name: [c_char; 512],
    pub forcedModel: [c_char; 256],
    pub forcedSkin: [c_char; 256],
    pub saber1: [c_char; 64],
    pub saber2: [c_char; 64],
    pub saberStance: c_int,
    pub weapons: c_int,
    pub forcePowerLevels: [c_int; NUM_FORCE_POWERS],
    pub classflags: c_int,
    pub maxhealth: c_int,
    pub starthealth: c_int,
    pub maxarmor: c_int,
    pub startarmor: c_int,
    pub speed: f32,
    pub hasForcedSaberColor: qboolean,
    pub forcedSaberColor: c_int,
    pub hasForcedSaber2Color: qboolean,
    pub forcedSaber2Color: c_int,
    pub invenItems: c_int,
    pub powerups: c_int,
    pub uiPortraitShader: c_int,
    pub uiPortrait: [c_char; 256],
    pub classShader: c_int,
    pub playerClass: c_short, // SPC_INFANTRY . ..
}
const _: () = assert!(core::mem::offset_of!(siegeClass_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, forcePowerLevels) == 1160);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, uiPortrait) == 1284);
const _: () = assert!(core::mem::offset_of!(siegeClass_t, playerClass) == 1544);
const _: () = assert!(core::mem::size_of::<siegeClass_t>() == 1548);

/// `siegeTeam_t` (bg_saga.h:83) — a named team owning class pointers. Pointer-
/// bearing (`classes`), so the layout is arch-dependent (asserts gated to 64-bit).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct siegeTeam_t {
    pub name: [c_char; 512],
    pub classes: [*mut siegeClass_t; MAX_SIEGE_CLASSES_PER_TEAM],
    pub numClasses: c_int,
    pub friendlyShader: c_int,
}
const _: () = assert!(core::mem::offset_of!(siegeTeam_t, name) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(siegeTeam_t, numClasses) == 640);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<siegeTeam_t>() == 648);
