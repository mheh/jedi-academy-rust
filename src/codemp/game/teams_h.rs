//! NPC team + class enums from `teams.h`.
//!
//! Both are plain `int`-width C enums embedded by value in `gclient_t`
//! (`playerTeam`/`enemyTeam` are `npcteam_t`; `NPC_class` is `class_t`), so the
//! `g_local.h` master needs them. Pure enums (no layout); values oracle-verified
//! in `oracle/teams_h_oracle.c`. Mirrors upstream `codemp/game/teams.h`.

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// `npcteam_t` (teams.h `//# team_e`) — anonymous enum + `typedef int`.
pub type npcteam_t = c_int;
pub const NPCTEAM_FREE: npcteam_t = 0; // also TEAM_FREE - caution, some code checks a team_t via "if (!team_t_varname)" so I guess this should stay as entry 0, great or what? -slc
pub const NPCTEAM_ENEMY: npcteam_t = 1; // also TEAM_RED
pub const NPCTEAM_PLAYER: npcteam_t = 2; // also TEAM_BLUE
pub const NPCTEAM_NEUTRAL: npcteam_t = 3; // also TEAM_SPECTATOR - most droids are team_neutral, there are some exceptions like Probe,Seeker,Interrogator
                                          //# #eol
pub const NPCTEAM_NUM_TEAMS: npcteam_t = 4;

/// `class_t` (teams.h) — NPC class. "This list is made up from the model
/// directories, this MUST be in the same order as the ClassNames array in
/// NPC_stats.cpp."
pub type class_t = c_int;
pub const CLASS_NONE: class_t = 0; // hopefully this will never be used by an npc, just covering all bases
pub const CLASS_ATST: class_t = 1; // technically droid...
pub const CLASS_BARTENDER: class_t = 2;
pub const CLASS_BESPIN_COP: class_t = 3;
pub const CLASS_CLAW: class_t = 4;
pub const CLASS_COMMANDO: class_t = 5;
pub const CLASS_DESANN: class_t = 6;
pub const CLASS_FISH: class_t = 7;
pub const CLASS_FLIER2: class_t = 8;
pub const CLASS_GALAK: class_t = 9;
pub const CLASS_GLIDER: class_t = 10;
pub const CLASS_GONK: class_t = 11; // droid
pub const CLASS_GRAN: class_t = 12;
pub const CLASS_HOWLER: class_t = 13;
pub const CLASS_IMPERIAL: class_t = 14;
pub const CLASS_IMPWORKER: class_t = 15;
pub const CLASS_INTERROGATOR: class_t = 16; // droid
pub const CLASS_JAN: class_t = 17;
pub const CLASS_JEDI: class_t = 18;
pub const CLASS_KYLE: class_t = 19;
pub const CLASS_LANDO: class_t = 20;
pub const CLASS_LIZARD: class_t = 21;
pub const CLASS_LUKE: class_t = 22;
pub const CLASS_MARK1: class_t = 23; // droid
pub const CLASS_MARK2: class_t = 24; // droid
pub const CLASS_GALAKMECH: class_t = 25; // droid
pub const CLASS_MINEMONSTER: class_t = 26;
pub const CLASS_MONMOTHA: class_t = 27;
pub const CLASS_MORGANKATARN: class_t = 28;
pub const CLASS_MOUSE: class_t = 29; // droid
pub const CLASS_MURJJ: class_t = 30;
pub const CLASS_PRISONER: class_t = 31;
pub const CLASS_PROBE: class_t = 32; // droid
pub const CLASS_PROTOCOL: class_t = 33; // droid
pub const CLASS_R2D2: class_t = 34; // droid
pub const CLASS_R5D2: class_t = 35; // droid
pub const CLASS_REBEL: class_t = 36;
pub const CLASS_REBORN: class_t = 37;
pub const CLASS_REELO: class_t = 38;
pub const CLASS_REMOTE: class_t = 39;
pub const CLASS_RODIAN: class_t = 40;
pub const CLASS_SEEKER: class_t = 41; // droid
pub const CLASS_SENTRY: class_t = 42;
pub const CLASS_SHADOWTROOPER: class_t = 43;
pub const CLASS_STORMTROOPER: class_t = 44;
pub const CLASS_SWAMP: class_t = 45;
pub const CLASS_SWAMPTROOPER: class_t = 46;
pub const CLASS_TAVION: class_t = 47;
pub const CLASS_TRANDOSHAN: class_t = 48;
pub const CLASS_UGNAUGHT: class_t = 49;
pub const CLASS_JAWA: class_t = 50;
pub const CLASS_WEEQUAY: class_t = 51;
pub const CLASS_BOBAFETT: class_t = 52;
pub const CLASS_VEHICLE: class_t = 53;
pub const CLASS_RANCOR: class_t = 54;
pub const CLASS_WAMPA: class_t = 55;
pub const CLASS_NUM_CLASSES: class_t = 56;

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;

    /// Parity: the teams.h enumerator values match the authentic C, whose enum
    /// bodies the oracle copies verbatim and lets the C compiler number (anchor /
    /// interior / terminal checkpoints catch any miscount in the long class_t list).
    #[test]
    fn teams_enum_values_match_c() {
        unsafe {
            assert_eq!(NPCTEAM_FREE, jka_teams_NPCTEAM_FREE());
            assert_eq!(NPCTEAM_NUM_TEAMS, jka_teams_NPCTEAM_NUM_TEAMS());

            assert_eq!(CLASS_NONE, jka_teams_CLASS_NONE());
            assert_eq!(CLASS_GONK, jka_teams_CLASS_GONK());
            assert_eq!(CLASS_GALAKMECH, jka_teams_CLASS_GALAKMECH());
            assert_eq!(CLASS_BOBAFETT, jka_teams_CLASS_BOBAFETT());
            assert_eq!(CLASS_WAMPA, jka_teams_CLASS_WAMPA());
            assert_eq!(CLASS_NUM_CLASSES, jka_teams_CLASS_NUM_CLASSES());
        }
    }
}
