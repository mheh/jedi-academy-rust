//! `bg_saga.c` — the Siege ("saga") gametype module, shared for game, cgame, and ui.
//!
//! Parses the `.scl` class files and `.team` team files that define each Siege
//! class (weapons, force levels, model/skin, saber, health/armor, portrait) and the
//! teams that own them, into the `bgSiegeClasses` / `bgSiegeTeams` tables. The data
//! model lives in [`bg_saga_h`].
//!
//! This crate compiles as the **QAGAME** server module, so the `#ifdef QAGAME`
//! `bgSiegeClasses` / `bgSiegeTeams` storage is included and the `#ifndef QAGAME`
//! cgame/ui `trap_R_RegisterShaderNoMip` shader-precache paths are excluded.
//!
//! Oracle: the name→id tables below carry raw `char *` (so they mirror the C mutable
//! globals as `static mut`, the `bg_saberLoad::SaberTable` precedent) and are
//! parity-validated by driving them through the real `GetIDForString` in the
//! `BG_SiegeTranslate*` oracle tests, exactly as `SaberTable` is.
//!
//! [`bg_saga_h`]: crate::codemp::game::bg_saga_h

#![allow(non_upper_case_globals, non_snake_case)]

use crate::codemp::game::bg_saga_h::{
    siegeClass_t, siegeClassDesc_t, siegeTeam_t, CFL_CUSTOMSKEL, CFL_EXTRA_AMMO, CFL_FASTFORCEREGEN,
    CFL_HEAVYMELEE, CFL_MORESABERDMG, CFL_SINGLE_ROCKET, CFL_STATVIEWER, CFL_STRONGAGAINSTPHYSICAL,
    MAX_SIEGE_CLASSES, MAX_SIEGE_INFO_SIZE, MAX_SIEGE_TEAMS, SIEGETEAM_TEAM1, SIEGETEAM_TEAM2,
    SIEGE_CLASS_DESC_LEN, SPC_INFANTRY, SPC_MAX,
};
use crate::codemp::game::bg_public::{
    HI_AMMODISP, HI_BINOCULARS, HI_CLOAK, HI_EWEB, HI_HEALTHDISP, HI_JETPACK, HI_MEDPAC,
    HI_MEDPAC_BIG, HI_NONE, HI_SEEKER, HI_SENTRY_GUN, HI_SHIELD, PW_BATTLESUIT, PW_BLUEFLAG,
    PW_CLOAKED, PW_DISINT_4, PW_FORCE_BOON, PW_FORCE_ENLIGHTENED_DARK, PW_FORCE_ENLIGHTENED_LIGHT,
    PW_NEUTRALFLAG, PW_NONE, PW_PULL, PW_QUAD, PW_REDFLAG, PW_SHIELDHIT, PW_SPEED, PW_SPEEDBURST,
    PW_YSALAMIRI,
};
use crate::codemp::game::bg_weapons_h::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_OLD, WP_BRYAR_PISTOL, WP_CONCUSSION, WP_DEMP2, WP_DET_PACK,
    WP_DISRUPTOR, WP_EMPLACED_GUN, WP_FLECHETTE, WP_MELEE, WP_NONE, WP_REPEATER,
    WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use crate::codemp::game::g_main::{Com_Error, Com_Printf};
use crate::codemp::game::q_shared::{va, Q_stricmp, Sz};
use crate::codemp::game::q_shared_h::{
    qboolean, saberInfo_t, stringID_table_t, ERR_DROP, FORCE_LEVEL_3, FORCE_LEVEL_5, FP_ABSORB,
    FP_DRAIN, FP_GRIP, FP_HEAL, FP_LEVITATION, FP_LIGHTNING, FP_PROTECT, FP_PULL, FP_PUSH, FP_RAGE,
    FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE, FP_SEE, FP_SPEED, FP_TEAM_FORCE,
    FP_TEAM_HEAL, FP_TELEPATHY, FS_READ, MAX_QPATH, MAX_SABERS, NUM_FORCE_POWERS, QFALSE, QTRUE,
    SS_DESANN, SS_DUAL, SS_FAST, SS_MEDIUM, SS_NONE, SS_STAFF, SS_STRONG, SS_TAVION,
};
use crate::codemp::game::bg_misc::BG_ModelCache;
use crate::codemp::game::bg_saberLoad::WP_SaberParseParms;
use crate::trap;
use core::ffi::{c_char, c_int, c_short, CStr};
use core::ptr::{addr_of, addr_of_mut};

extern "C" {
    fn atoi(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

/// `SIEGECHAR_TAB` (bg_saga.c:17) — "perhaps a bit hacky, but I don't think there's
/// any define existing for 'tab'".
const SIEGECHAR_TAB: c_char = 9;

/// Builds one [`stringID_table_t`] row from a name + id. Covers both the
/// `ENUM2STRING(x)` rows (`name == "x"`, `id == x`) and the explicit `"name", id`
/// alias/terminator rows in the C tables. (The `bg_saberLoad::enum2string`
/// precedent; widened to take any C-`int` id since these tables mix enum families.)
const fn s(name: &'static CStr, id: c_int) -> stringID_table_t {
    stringID_table_t { name: name.as_ptr(), id }
}

// New - only make one copy of this shit.
// #ifdef QAGAME (this crate is the QAGAME server module)
/// `siegeClass_t bgSiegeClasses[MAX_SIEGE_CLASSES]` (bg_saga.c:21) — the loaded
/// class table. C global → zero-initialized; mirrored as `static mut` zeroed (the
/// `bg_panimate::bgHumanoidAnimations` precedent).
pub static mut bgSiegeClasses: [siegeClass_t; MAX_SIEGE_CLASSES] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
/// `int bgNumSiegeClasses` (bg_saga.c:22).
pub static mut bgNumSiegeClasses: c_int = 0;

/// `siegeTeam_t bgSiegeTeams[MAX_SIEGE_TEAMS]` (bg_saga.c:24).
pub static mut bgSiegeTeams: [siegeTeam_t; MAX_SIEGE_TEAMS] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
/// `int bgNumSiegeTeams` (bg_saga.c:25).
pub static mut bgNumSiegeTeams: c_int = 0;

/// `char siege_info[MAX_SIEGE_INFO_SIZE]` (bg_saga.c:41) — the map's siege config.
pub static mut siege_info: [c_char; MAX_SIEGE_INFO_SIZE] = [0; MAX_SIEGE_INFO_SIZE];
/// `int siege_valid` (bg_saga.c:42).
pub static mut siege_valid: c_int = 0;

/// `siegeTeam_t *team1Theme` (bg_saga.c:44) — the theme chosen for each team.
pub static mut team1Theme: *mut siegeTeam_t = core::ptr::null_mut();
/// `siegeTeam_t *team2Theme` (bg_saga.c:45).
pub static mut team2Theme: *mut siegeTeam_t = core::ptr::null_mut();

//class flags
/// `stringID_table_t bgSiegeClassFlagNames[]` (bg_saga.c:48) — `CFL_*` flag names,
/// scanned by `BG_SiegeTranslateGenericTable`. The `{ "", -1 }` row terminates.
pub static mut bgSiegeClassFlagNames: [stringID_table_t; 9] = [
    s(c"CFL_MORESABERDMG", CFL_MORESABERDMG),
    s(c"CFL_STRONGAGAINSTPHYSICAL", CFL_STRONGAGAINSTPHYSICAL),
    s(c"CFL_FASTFORCEREGEN", CFL_FASTFORCEREGEN),
    s(c"CFL_STATVIEWER", CFL_STATVIEWER),
    s(c"CFL_HEAVYMELEE", CFL_HEAVYMELEE),
    s(c"CFL_SINGLE_ROCKET", CFL_SINGLE_ROCKET),
    s(c"CFL_CUSTOMSKEL", CFL_CUSTOMSKEL),
    s(c"CFL_EXTRA_AMMO", CFL_EXTRA_AMMO),
    s(c"", -1),
];

//saber stances
/// `stringID_table_t StanceTable[]` (bg_saga.c:62) — `SS_*` saber-stance names. The
/// `{ "", 0 }` row terminates.
pub static mut StanceTable: [stringID_table_t; 9] = [
    s(c"SS_NONE", SS_NONE),
    s(c"SS_FAST", SS_FAST),
    s(c"SS_MEDIUM", SS_MEDIUM),
    s(c"SS_STRONG", SS_STRONG),
    s(c"SS_DESANN", SS_DESANN),
    s(c"SS_TAVION", SS_TAVION),
    s(c"SS_DUAL", SS_DUAL),
    s(c"SS_STAFF", SS_STAFF),
    s(c"", 0),
];

//Weapon and force power tables are also used in NPC parsing code and some other places.
/// `stringID_table_t WPTable[]` (bg_saga.c:76) — `WP_*` weapon names. Note the
/// leading `"NULL", WP_NONE` row, the explicit `"WP_NONE", WP_NONE` row, and the
/// `"WP_BLASTER_PISTOL", WP_BRYAR_PISTOL` alias. The `{ "", 0 }` row terminates.
pub static mut WPTable: [stringID_table_t; 22] = [
    s(c"NULL", WP_NONE),
    s(c"WP_NONE", WP_NONE),
    // Player weapons
    s(c"WP_STUN_BATON", WP_STUN_BATON),
    s(c"WP_MELEE", WP_MELEE),
    s(c"WP_SABER", WP_SABER),
    s(c"WP_BRYAR_PISTOL", WP_BRYAR_PISTOL),
    s(c"WP_BLASTER_PISTOL", WP_BRYAR_PISTOL),
    s(c"WP_BLASTER", WP_BLASTER),
    s(c"WP_DISRUPTOR", WP_DISRUPTOR),
    s(c"WP_BOWCASTER", WP_BOWCASTER),
    s(c"WP_REPEATER", WP_REPEATER),
    s(c"WP_DEMP2", WP_DEMP2),
    s(c"WP_FLECHETTE", WP_FLECHETTE),
    s(c"WP_ROCKET_LAUNCHER", WP_ROCKET_LAUNCHER),
    s(c"WP_THERMAL", WP_THERMAL),
    s(c"WP_TRIP_MINE", WP_TRIP_MINE),
    s(c"WP_DET_PACK", WP_DET_PACK),
    s(c"WP_CONCUSSION", WP_CONCUSSION),
    s(c"WP_BRYAR_OLD", WP_BRYAR_OLD),
    s(c"WP_EMPLACED_GUN", WP_EMPLACED_GUN),
    s(c"WP_TURRET", WP_TURRET),
    s(c"", 0),
];

/// `stringID_table_t FPTable[]` (bg_saga.c:102) — `FP_*` force-power names, scanned
/// by `BG_SiegeTranslateForcePowers`. The `{ "", -1 }` row terminates.
pub static mut FPTable: [stringID_table_t; 19] = [
    s(c"FP_HEAL", FP_HEAL),
    s(c"FP_LEVITATION", FP_LEVITATION),
    s(c"FP_SPEED", FP_SPEED),
    s(c"FP_PUSH", FP_PUSH),
    s(c"FP_PULL", FP_PULL),
    s(c"FP_TELEPATHY", FP_TELEPATHY),
    s(c"FP_GRIP", FP_GRIP),
    s(c"FP_LIGHTNING", FP_LIGHTNING),
    s(c"FP_RAGE", FP_RAGE),
    s(c"FP_PROTECT", FP_PROTECT),
    s(c"FP_ABSORB", FP_ABSORB),
    s(c"FP_TEAM_HEAL", FP_TEAM_HEAL),
    s(c"FP_TEAM_FORCE", FP_TEAM_FORCE),
    s(c"FP_DRAIN", FP_DRAIN),
    s(c"FP_SEE", FP_SEE),
    s(c"FP_SABER_OFFENSE", FP_SABER_OFFENSE),
    s(c"FP_SABER_DEFENSE", FP_SABER_DEFENSE),
    s(c"FP_SABERTHROW", FP_SABERTHROW),
    s(c"", -1),
];

/// `stringID_table_t HoldableTable[]` (bg_saga.c:125) — `HI_*` holdable-item names.
/// The `{ "", -1 }` row terminates.
pub static mut HoldableTable: [stringID_table_t; 13] = [
    s(c"HI_NONE", HI_NONE),
    s(c"HI_SEEKER", HI_SEEKER),
    s(c"HI_SHIELD", HI_SHIELD),
    s(c"HI_MEDPAC", HI_MEDPAC),
    s(c"HI_MEDPAC_BIG", HI_MEDPAC_BIG),
    s(c"HI_BINOCULARS", HI_BINOCULARS),
    s(c"HI_SENTRY_GUN", HI_SENTRY_GUN),
    s(c"HI_JETPACK", HI_JETPACK),
    s(c"HI_HEALTHDISP", HI_HEALTHDISP),
    s(c"HI_AMMODISP", HI_AMMODISP),
    s(c"HI_EWEB", HI_EWEB),
    s(c"HI_CLOAK", HI_CLOAK),
    s(c"", -1),
];

/// `stringID_table_t PowerupTable[]` (bg_saga.c:144) — `PW_*` powerup names. The
/// `{ "", -1 }` row terminates.
pub static mut PowerupTable: [stringID_table_t; 17] = [
    s(c"PW_NONE", PW_NONE),
    s(c"PW_QUAD", PW_QUAD),
    s(c"PW_BATTLESUIT", PW_BATTLESUIT),
    s(c"PW_PULL", PW_PULL),
    s(c"PW_REDFLAG", PW_REDFLAG),
    s(c"PW_BLUEFLAG", PW_BLUEFLAG),
    s(c"PW_NEUTRALFLAG", PW_NEUTRALFLAG),
    s(c"PW_SHIELDHIT", PW_SHIELDHIT),
    s(c"PW_SPEEDBURST", PW_SPEEDBURST),
    s(c"PW_DISINT_4", PW_DISINT_4),
    s(c"PW_SPEED", PW_SPEED),
    s(c"PW_CLOAKED", PW_CLOAKED),
    s(c"PW_FORCE_ENLIGHTENED_LIGHT", PW_FORCE_ENLIGHTENED_LIGHT),
    s(c"PW_FORCE_ENLIGHTENED_DARK", PW_FORCE_ENLIGHTENED_DARK),
    s(c"PW_FORCE_BOON", PW_FORCE_BOON),
    s(c"PW_YSALAMIRI", PW_YSALAMIRI),
    s(c"", -1),
];

//======================================
//Parsing functions
//======================================
/// `void BG_SiegeStripTabs(char *buf)` (bg_saga.c:170) — convert every tab in `buf`
/// to a space, in place. (Despite the name it strips nothing: the read and write
/// indices advance together, so `buf` keeps its length.)
///
/// # Safety
/// `buf` must point to a writable NUL-terminated string.
pub unsafe fn BG_SiegeStripTabs(buf: *mut c_char) {
    let mut i: c_int = 0;
    let mut i_r: c_int = 0;

    while *buf.offset(i as isize) != 0 {
        if *buf.offset(i as isize) != SIEGECHAR_TAB {
            //not a tab, just stick it in
            *buf.offset(i_r as isize) = *buf.offset(i as isize);
        } else {
            //If it's a tab, convert it to a space.
            *buf.offset(i_r as isize) = b' ' as c_char;
        }

        i_r += 1;
        i += 1;
    }

    *buf.offset(i_r as isize) = b'\0' as c_char;
}

/// `int BG_SiegeGetValueGroup(char *buf, char *group, char *outbuf)` (bg_saga.c:193)
/// — find the `{ ... }` block named `group` at the top level of `buf` and copy its
/// inner contents (brackets stripped, tabs converted to spaces) into `outbuf`.
/// Returns 1 if found, 0 if not. Nested `{}` are balanced via `parseGroups`; `//`
/// comments are skipped. Malformed input triggers `Com_Error(ERR_DROP, ...)`.
///
/// # Safety
/// `buf`/`group` must be NUL-terminated; `outbuf` must be large enough to hold the
/// extracted group. `buf` must not contain a token longer than 4096 bytes (the C
/// `checkGroup` scratch buffer is fixed-size, faithfully reproduced here).
pub unsafe fn BG_SiegeGetValueGroup(
    buf: *mut c_char,
    group: *mut c_char,
    outbuf: *mut c_char,
) -> c_int {
    const SP: c_char = b' ' as c_char;
    const LB: c_char = b'{' as c_char;
    const RB: c_char = b'}' as c_char;
    const NL: c_char = b'\n' as c_char;
    const CR: c_char = b'\r' as c_char;
    const SL: c_char = b'/' as c_char;

    let mut i: usize = 0;
    let mut j: usize;
    let mut check_group: [c_char; 4096] = [0; 4096];
    let mut is_group: qboolean;
    // C: `int parseGroups = 0;`. The initial 0 is never observed — every read is
    // preceded by a `parse_groups = 0` reset inside its block — so it's left
    // uninitialized here to avoid a dead-store warning.
    let mut parse_groups: c_int;

    while *buf.add(i) != 0 {
        if *buf.add(i) != SP
            && *buf.add(i) != LB
            && *buf.add(i) != RB
            && *buf.add(i) != NL
            && *buf.add(i) != CR
            && *buf.add(i) != SIEGECHAR_TAB
        {
            //we're on a valid character
            if *buf.add(i) == SL && *buf.add(i + 1) == SL {
                //this is a comment, so skip over it
                while *buf.add(i) != 0
                    && *buf.add(i) != NL
                    && *buf.add(i) != CR
                    && *buf.add(i) != SIEGECHAR_TAB
                {
                    i += 1;
                }
            } else {
                //parse to the next space/endline/eos and check this value against our group value.
                j = 0;

                while *buf.add(i) != SP
                    && *buf.add(i) != NL
                    && *buf.add(i) != CR
                    && *buf.add(i) != SIEGECHAR_TAB
                    && *buf.add(i) != LB
                    && *buf.add(i) != 0
                {
                    if *buf.add(i) == SL && *buf.add(i + 1) == SL {
                        //hit a comment, break out.
                        break;
                    }

                    check_group[j] = *buf.add(i);
                    j += 1;
                    i += 1;
                }
                check_group[j] = 0;

                //Make sure this is a group as opposed to a globally defined value.
                if *buf.add(i) == SL && *buf.add(i + 1) == SL {
                    //stopped on a comment, so first parse to the end of it.
                    while *buf.add(i) != 0 && *buf.add(i) != NL && *buf.add(i) != CR {
                        i += 1;
                    }
                    while *buf.add(i) == NL || *buf.add(i) == CR {
                        i += 1;
                    }
                }

                if *buf.add(i) == 0 {
                    Com_Error(
                        ERR_DROP,
                        &format!("Unexpected EOF while looking for group '{}'", Sz(group)),
                    );
                }

                is_group = QFALSE;

                while *buf.add(i) != 0 && *buf.add(i) == SP
                    || *buf.add(i) == SIEGECHAR_TAB
                    || *buf.add(i) == NL
                    || *buf.add(i) == CR
                {
                    //parse to the next valid character
                    i += 1;
                }

                if *buf.add(i) == LB {
                    //if the next valid character is an opening bracket, then this is indeed a group
                    is_group = QTRUE;
                }

                //Is this the one we want?
                if is_group != QFALSE && Q_stricmp(check_group.as_ptr(), group) == 0 {
                    //guess so. Parse until we hit the { indicating the beginning of the group.
                    while *buf.add(i) != LB && *buf.add(i) != 0 {
                        i += 1;
                    }

                    if *buf.add(i) != 0 {
                        //We're at the start of the group now, so parse to the closing bracket.
                        j = 0;

                        parse_groups = 0;

                        while (*buf.add(i) != RB || parse_groups != 0) && *buf.add(i) != 0 {
                            if *buf.add(i) == LB {
                                //increment for the opening bracket.
                                parse_groups += 1;
                            } else if *buf.add(i) == RB {
                                //decrement for the closing bracket
                                parse_groups -= 1;
                            }

                            if parse_groups < 0 {
                                //Syntax error, I guess.
                                Com_Error(ERR_DROP, &format!("Found a closing bracket without an opening bracket while looking for group '{}'", Sz(group)));
                            }

                            if (*buf.add(i) != LB || parse_groups > 1)
                                && (*buf.add(i) != RB || parse_groups > 0)
                            {
                                //don't put the start and end brackets for this group into the output buffer
                                *outbuf.add(j) = *buf.add(i);
                                j += 1;
                            }

                            if *buf.add(i) == RB && parse_groups == 0 {
                                //Alright, we can break out now.
                                break;
                            }

                            i += 1;
                        }
                        *outbuf.add(j) = 0;

                        //Verify that we ended up on the closing bracket.
                        if *buf.add(i) != RB {
                            Com_Error(
                                ERR_DROP,
                                &format!("Group '{}' is missing a closing bracket", Sz(group)),
                            );
                        }

                        //Strip the tabs so we're friendly for value parsing.
                        BG_SiegeStripTabs(outbuf);

                        return 1; //we got it, so return 1.
                    } else {
                        Com_Error(ERR_DROP, &format!("Error parsing group in file, unexpected EOF before opening bracket while looking for group '{}'", Sz(group)));
                    }
                } else if is_group == QFALSE {
                    //if it wasn't a group, parse to the end of the line
                    while *buf.add(i) != 0 && *buf.add(i) != NL && *buf.add(i) != CR {
                        i += 1;
                    }
                } else {
                    //this was a group but we not the one we wanted to find, so parse by it.
                    parse_groups = 0;

                    while *buf.add(i) != 0 && (*buf.add(i) != RB || parse_groups != 0) {
                        if *buf.add(i) == LB {
                            parse_groups += 1;
                        } else if *buf.add(i) == RB {
                            parse_groups -= 1;
                        }

                        if parse_groups < 0 {
                            //Syntax error, I guess.
                            Com_Error(ERR_DROP, &format!("Found a closing bracket without an opening bracket while looking for group '{}'", Sz(group)));
                        }

                        if *buf.add(i) == RB && parse_groups == 0 {
                            //Alright, we can break out now.
                            break;
                        }

                        i += 1;
                    }

                    if *buf.add(i) != RB {
                        Com_Error(ERR_DROP, &format!("Found an opening bracket without a matching closing bracket while looking for group '{}'", Sz(group)));
                    }

                    i += 1;
                }
            }
        } else if *buf.add(i) == LB {
            //we're in a group that isn't the one we want, so parse to the end.
            parse_groups = 0;

            while *buf.add(i) != 0 && (*buf.add(i) != RB || parse_groups != 0) {
                if *buf.add(i) == LB {
                    parse_groups += 1;
                } else if *buf.add(i) == RB {
                    parse_groups -= 1;
                }

                if parse_groups < 0 {
                    //Syntax error, I guess.
                    Com_Error(ERR_DROP, &format!("Found a closing bracket without an opening bracket while looking for group '{}'", Sz(group)));
                }

                if *buf.add(i) == RB && parse_groups == 0 {
                    //Alright, we can break out now.
                    break;
                }

                i += 1;
            }

            if *buf.add(i) != RB {
                Com_Error(ERR_DROP, &format!("Found an opening bracket without a matching closing bracket while looking for group '{}'", Sz(group)));
            }
        }

        if *buf.add(i) == 0 {
            break;
        }
        i += 1;
    }

    0 //guess we never found it.
}

/// `int BG_SiegeGetPairedValue(char *buf, char *key, char *outbuf)` (bg_saga.c:410)
/// — find the top-level `key value` pair in `buf` and copy its value into `outbuf`,
/// returning 1 if found, 0 if not. Sub-`{}`-groups are skipped (not searched);
/// quoted values parse to the closing quote, bare values to the next whitespace.
/// `//` comments are skipped; malformed input triggers `Com_Error(ERR_DROP, ...)`.
///
/// # Safety
/// `buf`/`key` must be NUL-terminated; `outbuf` must be large enough for the value.
/// `buf` must not contain a key token longer than 4096 bytes (the fixed-size C
/// `checkKey` scratch buffer, faithfully reproduced).
pub unsafe fn BG_SiegeGetPairedValue(
    buf: *mut c_char,
    key: *mut c_char,
    outbuf: *mut c_char,
) -> c_int {
    const SP: c_char = b' ' as c_char;
    const LB: c_char = b'{' as c_char;
    const RB: c_char = b'}' as c_char;
    const NL: c_char = b'\n' as c_char;
    const CR: c_char = b'\r' as c_char;
    const SL: c_char = b'/' as c_char;
    const QUOTE: c_char = b'"' as c_char;

    let mut i: usize = 0;
    let mut j: usize;
    let mut k: usize;
    let mut check_key: [c_char; 4096] = [0; 4096];

    while *buf.add(i) != 0 {
        if *buf.add(i) != SP
            && *buf.add(i) != LB
            && *buf.add(i) != RB
            && *buf.add(i) != NL
            && *buf.add(i) != CR
        {
            //we're on a valid character
            if *buf.add(i) == SL && *buf.add(i + 1) == SL {
                //this is a comment, so skip over it
                while *buf.add(i) != 0 && *buf.add(i) != NL && *buf.add(i) != CR {
                    i += 1;
                }
            } else {
                //parse to the next space/endline/eos and check this value against our key value.
                j = 0;

                while *buf.add(i) != SP
                    && *buf.add(i) != NL
                    && *buf.add(i) != CR
                    && *buf.add(i) != SIEGECHAR_TAB
                    && *buf.add(i) != 0
                {
                    if *buf.add(i) == SL && *buf.add(i + 1) == SL {
                        //hit a comment, break out.
                        break;
                    }

                    check_key[j] = *buf.add(i);
                    j += 1;
                    i += 1;
                }
                check_key[j] = 0;

                k = i;

                while *buf.add(k) != 0
                    && (*buf.add(k) == SP || *buf.add(k) == NL || *buf.add(k) == CR)
                {
                    k += 1;
                }

                if *buf.add(k) == LB {
                    //this is not the start of a value but rather of a group. We don't want to look in subgroups so skip over the whole thing.
                    let mut open_b: c_int = 0;

                    while *buf.add(i) != 0 && (*buf.add(i) != RB || open_b != 0) {
                        if *buf.add(i) == LB {
                            open_b += 1;
                        } else if *buf.add(i) == RB {
                            open_b -= 1;
                        }

                        if open_b < 0 {
                            Com_Error(ERR_DROP, &format!("Unexpected closing bracket (too many) while parsing to end of group '{}'", Sz(check_key.as_ptr())));
                        }

                        if *buf.add(i) == RB && open_b == 0 {
                            //this is the end of the group
                            break;
                        }
                        i += 1;
                    }

                    if *buf.add(i) == RB {
                        i += 1;
                    }
                } else {
                    //Is this the one we want?
                    if *buf.add(i) != SL || *buf.add(i + 1) != SL {
                        //make sure we didn't stop on a comment, if we did then this is considered an error in the file.
                        if Q_stricmp(check_key.as_ptr(), key) == 0 {
                            //guess so. Parse along to the next valid character, then put that into the output buffer and return 1.
                            while (*buf.add(i) == SP
                                || *buf.add(i) == NL
                                || *buf.add(i) == CR
                                || *buf.add(i) == SIEGECHAR_TAB)
                                && *buf.add(i) != 0
                            {
                                i += 1;
                            }

                            if *buf.add(i) != 0 {
                                //We're at the start of the value now.
                                let mut parse_to_quote: qboolean = QFALSE;

                                if *buf.add(i) == QUOTE {
                                    //if the value is in quotes, then stop at the next quote instead of ' '
                                    i += 1;
                                    parse_to_quote = QTRUE;
                                }

                                j = 0;
                                while (parse_to_quote == QFALSE
                                    && *buf.add(i) != SP
                                    && *buf.add(i) != NL
                                    && *buf.add(i) != CR)
                                    || (parse_to_quote != QFALSE && *buf.add(i) != QUOTE)
                                {
                                    if *buf.add(i) == SL && *buf.add(i + 1) == SL {
                                        //hit a comment after the value? This isn't an ideal way to be writing things, but we'll support it anyway.
                                        break;
                                    }
                                    *outbuf.add(j) = *buf.add(i);
                                    j += 1;
                                    i += 1;

                                    if *buf.add(i) == 0 {
                                        if parse_to_quote != QFALSE {
                                            Com_Error(ERR_DROP, &format!("Unexpected EOF while looking for endquote, error finding paired value for '{}'", Sz(key)));
                                        } else {
                                            Com_Error(ERR_DROP, &format!("Unexpected EOF while looking for space or endline, error finding paired value for '{}'", Sz(key)));
                                        }
                                    }
                                }
                                *outbuf.add(j) = 0;

                                return 1; //we got it, so return 1.
                            } else {
                                Com_Error(ERR_DROP, &format!("Error parsing file, unexpected EOF while looking for valud '{}'", Sz(key)));
                            }
                        } else {
                            //if that wasn't the desired key, then make sure we parse to the end of the line, so we don't mistake a value for a key
                            while *buf.add(i) != 0 && *buf.add(i) != NL {
                                i += 1;
                            }
                        }
                    } else {
                        Com_Error(ERR_DROP, &format!("Error parsing file, found comment, expected value for '{}'", Sz(key)));
                    }
                }
            }
        }

        if *buf.add(i) == 0 {
            break;
        }
        i += 1;
    }

    0 //guess we never found it.
}

/// `void BG_SiegeTranslateForcePowers(char *buf, siegeClass_t *siegeClass)`
/// (bg_saga.c:573) — parse a `|`-separated force-power list (each optionally
/// `,level`) into `siegeClass->forcePowerLevels`. `"FP_ALL"` grants every power at
/// `FORCE_LEVEL_3`; a bare `"0"` grants none. Unspecified level defaults to 3,
/// clamped to `[0, FORCE_LEVEL_5]`. `"FP_JUMP"` is aliased to `FP_LEVITATION`.
///
/// # Safety
/// `buf` must be NUL-terminated; `siegeClass` must be a valid `siegeClass_t`. No
/// power name may exceed the fixed C scratch buffers (1024/256), faithfully kept.
pub unsafe fn BG_SiegeTranslateForcePowers(buf: *mut c_char, siege_class: *mut siegeClass_t) {
    const SP: c_char = b' ' as c_char;
    const BAR: c_char = b'|' as c_char;
    const COMMA: c_char = b',' as c_char;

    let mut check_power: [c_char; 1024] = [0; 1024];
    let mut check_level: [c_char; 256] = [0; 256];
    let mut l: usize;
    let mut k: usize;
    let mut j: usize;
    let mut i: usize = 0;
    // C: `int parsedLevel = 0;`. The initial 0 is never observed — it is set in both
    // branches (atoi result, clamped, or the default 3) before its only read — so it
    // is left uninitialized here to avoid a dead-store warning.
    let mut parsed_level: c_int;
    let mut all_powers: qboolean = QFALSE;
    let mut no_powers: qboolean = QFALSE;

    if Q_stricmp(buf, c"FP_ALL".as_ptr()) == 0 {
        //this is a special case, just give us all the powers on level 3
        all_powers = QTRUE;
    }

    if *buf.add(0) == b'0' as c_char && *buf.add(1) == 0 {
        //no powers then
        no_powers = QTRUE;
    }

    //First clear out the powers, or in the allPowers case, give us all level 3.
    // (C relies on the `int i = 0` initializer here; no reset needed.)
    while i < NUM_FORCE_POWERS {
        if all_powers != QFALSE {
            (*siege_class).forcePowerLevels[i] = FORCE_LEVEL_3;
        } else {
            (*siege_class).forcePowerLevels[i] = 0;
        }
        i += 1;
    }

    if all_powers != QFALSE || no_powers != QFALSE {
        //we're done now then.
        return;
    }

    i = 0;
    while *buf.add(i) != 0 {
        //parse through the list which is seperated by |, and add all the weapons into a bitflag
        if *buf.add(i) != SP && *buf.add(i) != BAR {
            j = 0;

            while *buf.add(i) != 0
                && *buf.add(i) != SP
                && *buf.add(i) != BAR
                && *buf.add(i) != COMMA
            {
                check_power[j] = *buf.add(i);
                j += 1;
                i += 1;
            }
            check_power[j] = 0;

            if *buf.add(i) == COMMA {
                //parse the power level
                i += 1;
                l = 0;
                while *buf.add(i) != 0 && *buf.add(i) != SP && *buf.add(i) != BAR {
                    check_level[l] = *buf.add(i);
                    l += 1;
                    i += 1;
                }
                check_level[l] = 0;
                parsed_level = atoi(check_level.as_ptr());

                //keep sane limits on the powers
                if parsed_level < 0 {
                    parsed_level = 0;
                }
                if parsed_level > FORCE_LEVEL_5 {
                    parsed_level = FORCE_LEVEL_5;
                }
            } else {
                //if it's not there, assume level 3 I guess.
                parsed_level = 3;
            }

            if check_power[0] != 0 {
                //Got the name, compare it against the weapon table strings.
                k = 0;

                if Q_stricmp(check_power.as_ptr(), c"FP_JUMP".as_ptr()) == 0 {
                    //haqery
                    strcpy(check_power.as_mut_ptr(), c"FP_LEVITATION".as_ptr());
                }

                let fptable = addr_of!(FPTable) as *const stringID_table_t;
                while (*fptable.add(k)).id != -1 && *(*fptable.add(k)).name != 0 {
                    if Q_stricmp(check_power.as_ptr(), (*fptable.add(k)).name) == 0 {
                        //found it, add the weapon into the weapons value
                        (*siege_class).forcePowerLevels[k] = parsed_level;
                        break;
                    }
                    k += 1;
                }
            }
        }

        if *buf.add(i) == 0 {
            break;
        }
        i += 1;
    }
}

//Used for the majority of generic val parsing stuff. buf should be the value string,
//table should be the appropriate string/id table. If bitflag is qtrue then the
//values are accumulated into a bitflag. If bitflag is qfalse then the first value
//is returned as a directly corresponding id and no further parsing is done.
/// `int BG_SiegeTranslateGenericTable(char *buf, stringID_table_t *table, qboolean
/// bitflag)` (bg_saga.c:690).
///
/// # Safety
/// `buf` must be NUL-terminated; `table` must be a `{ NULL, _ }`/`{ "", _ }`-
/// terminated `stringID_table_t` array. No value token may exceed 1024 bytes.
pub unsafe fn BG_SiegeTranslateGenericTable(
    buf: *mut c_char,
    table: *const stringID_table_t,
    bitflag: qboolean,
) -> c_int {
    const SP: c_char = b' ' as c_char;
    const BAR: c_char = b'|' as c_char;

    let mut items: c_int = 0;
    let mut check_item: [c_char; 1024] = [0; 1024];
    let mut i: usize = 0;
    let mut j: usize;
    let mut k: usize;

    if *buf.add(0) == b'0' as c_char && *buf.add(1) == 0 {
        //special case, no items.
        return 0;
    }

    while *buf.add(i) != 0 {
        //Using basically the same parsing method as we do for weapons and forcepowers.
        if *buf.add(i) != SP && *buf.add(i) != BAR {
            j = 0;

            while *buf.add(i) != 0 && *buf.add(i) != SP && *buf.add(i) != BAR {
                check_item[j] = *buf.add(i);
                j += 1;
                i += 1;
            }
            check_item[j] = 0;

            if check_item[0] != 0 {
                k = 0;

                while !(*table.add(k)).name.is_null() && *(*table.add(k)).name != 0 {
                    //go through the list and check the parsed flag name against the hardcoded names
                    if Q_stricmp(check_item.as_ptr(), (*table.add(k)).name) == 0 {
                        //Got it, so add the value into our items value.
                        if bitflag != QFALSE {
                            items |= 1 << (*table.add(k)).id;
                        } else {
                            //return the value directly then.
                            return (*table.add(k)).id;
                        }
                        break;
                    }
                    k += 1;
                }
            }
        }

        if *buf.add(i) == 0 {
            break;
        }

        i += 1;
    }
    items
}
//======================================
//End parsing functions
//======================================

//======================================
//Class loading functions
//======================================

/// `char *classTitles[SPC_MAX]` (bg_saga.c:750) — base-class icon-name suffixes used
/// to derive `playerClass` from a class's `class_shader` name. Mirrors the C mutable
/// `char *[]` global as `static mut` (the `bg_misc` string-table precedent).
pub static mut classTitles: [*const c_char; SPC_MAX as usize] = [
    c"infantry".as_ptr(),      // SPC_INFANTRY
    c"vanguard".as_ptr(),      // SPC_VANGUARD
    c"support".as_ptr(),       // SPC_SUPPORT
    c"jedi_general".as_ptr(),  // SPC_JEDI
    c"demolitionist".as_ptr(), // SPC_DEMOLITIONIST
    c"heavy_weapons".as_ptr(), // SPC_HEAVY_WEAPONS
];

/// `void BG_SiegeParseClassFile(const char *filename, siegeClassDesc_t *descBuffer)`
/// (bg_saga.c:761) — load a `.scl` class file and parse it into the next
/// `bgSiegeClasses` slot (incrementing `bgNumSiegeClasses` on success). `descBuffer`,
/// if non-null, receives the class `description` (or `"DESCRIPTION UNAVAILABLE"`).
/// `name`, `weapons`, and `uishader` are required (else `Com_Error`); everything else
/// has a default. A saber-less class is granted `WP_MELEE`. `playerClass` is derived
/// by suffix-matching the `class_shader` name against [`classTitles`].
///
/// QAGAME build: the `uishader`/`class_shader` `trap_R_RegisterShaderNoMip` paths are
/// `#ifndef QAGAME` (cgame/ui), so here the `uishader` body is empty and the
/// `class_shader` body reduces to the `playerClass` suffix match.
///
/// No oracle — pure engine-trap file I/O (`trap_FS_*`), which the off-engine oracle
/// harness cannot satisfy (the `WP_SaberLoadParms` / `BG_VehWeaponLoadParms`
/// precedent). Its parse logic is delegated to the oracle-tested
/// `BG_SiegeGetPairedValue` / `BG_SiegeGetValueGroup` / `BG_SiegeTranslate*` helpers.
///
/// # Safety
/// `filename` must be NUL-terminated; `descBuffer` may be null or a valid
/// `siegeClassDesc_t`.
pub unsafe fn BG_SiegeParseClassFile(filename: *const c_char, desc_buffer: *mut siegeClassDesc_t) {
    let len: c_int;
    let mut i: c_int;
    let mut class_info: [c_char; 4096] = [0; 4096];
    let mut parse_buf: [c_char; 4096] = [0; 4096];

    let fname = CStr::from_ptr(filename).to_string_lossy();
    let (l, f) = trap::FS_FOpenFile(&fname, FS_READ);
    len = l;

    if f == 0 || len >= 4096 {
        return;
    }

    {
        let buf = core::slice::from_raw_parts_mut(class_info.as_mut_ptr() as *mut u8, len as usize);
        trap::FS_Read(buf, f);
    }

    trap::FS_FCloseFile(f);

    class_info[len as usize] = 0;

    //first get the description if we have a buffer for it
    if !desc_buffer.is_null() {
        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"description".as_ptr() as *mut c_char,
            (*desc_buffer).desc.as_mut_ptr(),
        ) == 0
        {
            strcpy(
                (*desc_buffer).desc.as_mut_ptr(),
                c"DESCRIPTION UNAVAILABLE".as_ptr(),
            );
        }

        //Hit this assert?  Memory has already been trashed.  Increase
        //SIEGE_CLASS_DESC_LEN.
        debug_assert!(strlen((*desc_buffer).desc.as_ptr()) < SIEGE_CLASS_DESC_LEN);
    }

    BG_SiegeGetValueGroup(
        class_info.as_mut_ptr(),
        c"ClassInfo".as_ptr() as *mut c_char,
        class_info.as_mut_ptr(),
    );

    let idx = bgNumSiegeClasses as usize;
    let cls = addr_of_mut!(bgSiegeClasses[idx]);

    //Parse name
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"name".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        strcpy((*cls).name.as_mut_ptr(), parse_buf.as_ptr());
    } else {
        Com_Error(ERR_DROP, "Siege class without name entry");
    }

    //Parse forced model
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"model".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        strcpy((*cls).forcedModel.as_mut_ptr(), parse_buf.as_ptr());
    } else {
        //It's ok if there isn't one, it's optional.
        (*cls).forcedModel[0] = 0;
    }

    //Parse forced skin
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"skin".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        strcpy((*cls).forcedSkin.as_mut_ptr(), parse_buf.as_ptr());
    } else {
        //It's ok if there isn't one, it's optional.
        (*cls).forcedSkin[0] = 0;
    }

    //Parse first saber
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"saber1".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        strcpy((*cls).saber1.as_mut_ptr(), parse_buf.as_ptr());
    } else {
        //It's ok if there isn't one, it's optional.
        (*cls).saber1[0] = 0;
    }

    //Parse second saber
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"saber2".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        strcpy((*cls).saber2.as_mut_ptr(), parse_buf.as_ptr());
    } else {
        //It's ok if there isn't one, it's optional.
        (*cls).saber2[0] = 0;
    }

    //Parse forced saber stance
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"saberstyle".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).saberStance = BG_SiegeTranslateGenericTable(
            parse_buf.as_mut_ptr(),
            addr_of!(StanceTable) as *const stringID_table_t,
            QTRUE,
        );
    } else {
        //It's ok if there isn't one, it's optional.
        (*cls).saberStance = 0;
    }

    //Parse forced saber color
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"sabercolor".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).forcedSaberColor = atoi(parse_buf.as_ptr());
        (*cls).hasForcedSaberColor = QTRUE;
    } else {
        //It's ok if there isn't one, it's optional.
        (*cls).hasForcedSaberColor = QFALSE;
    }

    //Parse forced saber2 color
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"saber2color".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).forcedSaber2Color = atoi(parse_buf.as_ptr());
        (*cls).hasForcedSaber2Color = QTRUE;
    } else {
        //It's ok if there isn't one, it's optional.
        (*cls).hasForcedSaber2Color = QFALSE;
    }

    //Parse weapons
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"weapons".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).weapons = BG_SiegeTranslateGenericTable(
            parse_buf.as_mut_ptr(),
            addr_of!(WPTable) as *const stringID_table_t,
            QTRUE,
        );
    } else {
        Com_Error(ERR_DROP, "Siege class without weapons entry");
    }

    if (*cls).weapons & (1 << WP_SABER) == 0 {
        //make sure it has melee if there's no saber
        (*cls).weapons |= 1 << WP_MELEE;

        //always give them this too if they are not a saber user
        //bgSiegeClasses[bgNumSiegeClasses].weapons |= (1 << WP_BRYAR_PISTOL);
    }

    //Parse forcepowers
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"forcepowers".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        BG_SiegeTranslateForcePowers(parse_buf.as_mut_ptr(), cls);
    } else {
        //fine, clear out the powers.
        i = 0;
        while i < NUM_FORCE_POWERS as c_int {
            (*cls).forcePowerLevels[i as usize] = 0;
            i += 1;
        }
    }

    //Parse classflags
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"classflags".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).classflags = BG_SiegeTranslateGenericTable(
            parse_buf.as_mut_ptr(),
            addr_of!(bgSiegeClassFlagNames) as *const stringID_table_t,
            QTRUE,
        );
    } else {
        //fine, we'll 0 it.
        (*cls).classflags = 0;
    }

    //Parse maxhealth
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"maxhealth".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).maxhealth = atoi(parse_buf.as_ptr());
    } else {
        //It's alright, just default to 100 then.
        (*cls).maxhealth = 100;
    }

    //Parse starthealth
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"starthealth".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).starthealth = atoi(parse_buf.as_ptr());
    } else {
        //It's alright, just default to 100 then.
        (*cls).starthealth = (*cls).maxhealth;
    }

    //Parse startarmor
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"maxarmor".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).maxarmor = atoi(parse_buf.as_ptr());
    } else {
        //It's alright, just default to 0 then.
        (*cls).maxarmor = 0;
    }

    //Parse startarmor
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"startarmor".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).startarmor = atoi(parse_buf.as_ptr());
        if (*cls).maxarmor == 0 {
            //if they didn't specify a damn max armor then use this.
            (*cls).maxarmor = (*cls).startarmor;
        }
    } else {
        //default to maxarmor.
        (*cls).startarmor = (*cls).maxarmor;
    }

    //Parse speed (this is a multiplier value)
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"speed".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).speed = atof(parse_buf.as_ptr()) as f32;
    } else {
        //It's alright, just default to 1 then.
        (*cls).speed = 1.0f32;
    }

    //Parse shader for ui to use
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"uishader".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        // #ifdef QAGAME: the trap_R_RegisterShaderNoMip portrait path is #else (ui).
        (*cls).uiPortraitShader = 0;
        // memset(uiPortrait, 0, sizeof(uiPortrait))
        (*cls).uiPortrait = [0; 256];
    } else {
        //I guess this is an essential.. we don't want to render bad shaders or anything.
        Com_Error(ERR_DROP, "Siege class without uishader entry");
    }

    //Parse shader for ui to use
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"class_shader".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        // #ifdef QAGAME: the classShader trap_R_RegisterShaderNoMip path is #else
        // (cgame/ui); under QAGAME classShader is zeroed and only the
        // base-player-class derivation below remains.
        (*cls).classShader = 0;
        // Find the base player class based on the icon name - very bad, I know.
        let title_length = strlen(parse_buf.as_ptr()) as c_int;
        let class_titles = addr_of!(classTitles) as *const *const c_char;
        let mut ci: c_int = 0;
        while ci < SPC_MAX {
            // Back up
            let array_title_length = strlen(*class_titles.add(ci as usize)) as c_int;
            if array_title_length > title_length {
                // Too long
                break;
            }

            let hold_buf = parse_buf.as_ptr().add((title_length - array_title_length) as usize);
            if strcmp(hold_buf, *class_titles.add(ci as usize)) == 0 {
                (*cls).playerClass = ci as c_short;
                break;
            }
            ci += 1;
        }

        // In case the icon name doesn't match up
        if ci >= SPC_MAX {
            (*cls).playerClass = SPC_INFANTRY as c_short;
        }
    } else {
        //No entry!  Bad bad bad
        Com_Printf(&format!(
            "ERROR: no class_shader defined for class {}\n",
            Sz((*cls).name.as_ptr())
        ));
    }

    //Parse holdable items to use
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"holdables".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).invenItems = BG_SiegeTranslateGenericTable(
            parse_buf.as_mut_ptr(),
            addr_of!(HoldableTable) as *const stringID_table_t,
            QTRUE,
        );
    } else {
        //Just don't start out with any then.
        (*cls).invenItems = 0;
    }

    //Parse powerups to use
    if BG_SiegeGetPairedValue(
        class_info.as_mut_ptr(),
        c"powerups".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        (*cls).powerups = BG_SiegeTranslateGenericTable(
            parse_buf.as_mut_ptr(),
            addr_of!(PowerupTable) as *const stringID_table_t,
            QTRUE,
        );
    } else {
        //Just don't start out with any then.
        (*cls).powerups = 0;
    }

    //A successful read.
    bgNumSiegeClasses += 1;
}

//======================================
//Misc/utility functions
//======================================
/// `siegeTeam_t *BG_SiegeFindThemeForTeam(int team)` (bg_saga.c:1346) — the team's
/// chosen theme (`team1Theme`/`team2Theme`), or null for any other team.
pub unsafe fn BG_SiegeFindThemeForTeam(team: c_int) -> *mut siegeTeam_t {
    if team == SIEGETEAM_TEAM1 {
        return team1Theme;
    } else if team == SIEGETEAM_TEAM2 {
        return team2Theme;
    }

    core::ptr::null_mut()
}

/// `int BG_SiegeCountBaseClass(const int team, const short classIndex)`
/// (bg_saga.c:1073) — count the team's classes whose `playerClass == classIndex`.
pub unsafe fn BG_SiegeCountBaseClass(team: c_int, class_index: c_short) -> c_int {
    let mut count: c_int = 0;
    let mut i: c_int;
    let stm: *mut siegeTeam_t = BG_SiegeFindThemeForTeam(team);
    if stm.is_null() {
        return 0;
    }

    i = 0;
    while i < (*stm).numClasses {
        if (*(*stm).classes[i as usize]).playerClass == class_index {
            count += 1;
        }
        i += 1;
    }
    count
}

/// `char *BG_GetUIPortraitFile(const int team, const short classIndex, const short
/// cntIndex)` (bg_saga.c:1096) — the `uiPortrait` of the `cntIndex`-th class of
/// `classIndex` on the team, or null.
pub unsafe fn BG_GetUIPortraitFile(
    team: c_int,
    class_index: c_short,
    cnt_index: c_short,
) -> *mut c_char {
    let mut count: c_int = 0;
    let mut i: c_int;
    let stm: *mut siegeTeam_t = BG_SiegeFindThemeForTeam(team);
    if stm.is_null() {
        return core::ptr::null_mut();
    }

    // Loop through all the classes for this team
    i = 0;
    while i < (*stm).numClasses {
        // does it match the base class?
        if (*(*stm).classes[i as usize]).playerClass == class_index {
            if count == cnt_index as c_int {
                return (*(*stm).classes[i as usize]).uiPortrait.as_mut_ptr();
            }
            count += 1;
        }
        i += 1;
    }

    core::ptr::null_mut()
}

/// `int BG_GetUIPortrait(const int team, const short classIndex, const short
/// cntIndex)` (bg_saga.c:1125) — the `uiPortraitShader` of the `cntIndex`-th class of
/// `classIndex` on the team, or 0.
pub unsafe fn BG_GetUIPortrait(team: c_int, class_index: c_short, cnt_index: c_short) -> c_int {
    let mut count: c_int = 0;
    let mut i: c_int;
    let stm: *mut siegeTeam_t = BG_SiegeFindThemeForTeam(team);
    if stm.is_null() {
        return 0;
    }

    // Loop through all the classes for this team
    i = 0;
    while i < (*stm).numClasses {
        // does it match the base class?
        if (*(*stm).classes[i as usize]).playerClass == class_index {
            if count == cnt_index as c_int {
                return (*(*stm).classes[i as usize]).uiPortraitShader;
            }
            count += 1;
        }
        i += 1;
    }

    0
}

// This is really getting ugly - looking to get the base class (within a class) based on the index passed in
/// `siegeClass_t *BG_GetClassOnBaseClass(const int team, const short classIndex,
/// const short cntIndex)` (bg_saga.c:1155) — the `cntIndex`-th class of `classIndex`
/// on the team, or null.
pub unsafe fn BG_GetClassOnBaseClass(
    team: c_int,
    class_index: c_short,
    cnt_index: c_short,
) -> *mut siegeClass_t {
    let mut count: c_int = 0;
    let mut i: c_int;
    let stm: *mut siegeTeam_t = BG_SiegeFindThemeForTeam(team);
    if stm.is_null() {
        return core::ptr::null_mut();
    }

    // Loop through all the classes for this team
    i = 0;
    while i < (*stm).numClasses {
        // does it match the base class?
        if (*(*stm).classes[i as usize]).playerClass == class_index {
            if count == cnt_index as c_int {
                return (*stm).classes[i as usize];
            }
            count += 1;
        }
        i += 1;
    }

    core::ptr::null_mut()
}

/// `siegeClass_t *BG_SiegeFindClassByName(const char *classname)` (bg_saga.c:1221) —
/// the loaded class with that name (case-insensitive), or null.
pub unsafe fn BG_SiegeFindClassByName(classname: *const c_char) -> *mut siegeClass_t {
    let base = addr_of_mut!(bgSiegeClasses) as *mut siegeClass_t;
    let mut i: c_int = 0;

    while i < bgNumSiegeClasses {
        if Q_stricmp((*base.add(i as usize)).name.as_ptr(), classname) == 0 {
            //found it
            return base.add(i as usize);
        }
        i += 1;
    }

    core::ptr::null_mut()
}

/// `int BG_SiegeFindClassIndexByName(const char *classname)` (bg_saga.c:1491) — the
/// `bgSiegeClasses` index of that class (case-insensitive), or -1.
pub unsafe fn BG_SiegeFindClassIndexByName(classname: *const c_char) -> c_int {
    let base = addr_of!(bgSiegeClasses) as *const siegeClass_t;
    let mut i: c_int = 0;

    while i < bgNumSiegeClasses {
        if Q_stricmp((*base.add(i as usize)).name.as_ptr(), classname) == 0 {
            //found it
            return i;
        }
        i += 1;
    }

    -1
}

/// `siegeTeam_t *BG_SiegeFindTeamForTheme(char *themeName)` (bg_saga.c:1457) — the
/// loaded team with that theme name (case-insensitive), or null. (The C's
/// `bgSiegeTeams[i].name &&` truthiness test is always true for an array field;
/// kept faithfully.)
pub unsafe fn BG_SiegeFindTeamForTheme(theme_name: *mut c_char) -> *mut siegeTeam_t {
    let base = addr_of_mut!(bgSiegeTeams) as *mut siegeTeam_t;
    let mut i: c_int = 0;

    while i < bgNumSiegeTeams {
        // C: `if (bgSiegeTeams[i].name && !Q_stricmp(...))` — the `name &&` test is
        // vacuous (an array field is never null), so only the name compare remains.
        if Q_stricmp((*base.add(i as usize)).name.as_ptr(), theme_name) == 0 {
            //this is what we're looking for
            return base.add(i as usize);
        }

        i += 1;
    }

    core::ptr::null_mut()
}

/// `void BG_SiegeSetTeamTheme(int team, char *themeName)` (bg_saga.c:1475) — point
/// the team's theme global at the loaded team with that name (the C selects
/// `&team1Theme` for `SIEGETEAM_TEAM1`, else `&team2Theme`).
pub unsafe fn BG_SiegeSetTeamTheme(team: c_int, theme_name: *mut c_char) {
    let team_ptr: *mut *mut siegeTeam_t = if team == SIEGETEAM_TEAM1 {
        addr_of_mut!(team1Theme)
    } else {
        addr_of_mut!(team2Theme)
    };

    *team_ptr = BG_SiegeFindTeamForTheme(theme_name);
}

/// `qboolean BG_SiegeCheckClassLegality(int team, char *classname)` (bg_saga.c:1418)
/// — is `classname` allowed on `team`'s theme? Spectator (any other team) and a
/// theme-less team are always legal. On an illegal class, `classname` is overwritten
/// with the team's first class name and `qfalse` returned.
pub unsafe fn BG_SiegeCheckClassLegality(team: c_int, classname: *mut c_char) -> qboolean {
    let team_ptr: *mut *mut siegeTeam_t;
    let mut i: c_int = 0;

    if team == SIEGETEAM_TEAM1 {
        team_ptr = addr_of_mut!(team1Theme);
    } else if team == SIEGETEAM_TEAM2 {
        team_ptr = addr_of_mut!(team2Theme);
    } else {
        //spectator? Whatever, you're legal then.
        return QTRUE;
    }

    if team_ptr.is_null() || (*team_ptr).is_null() {
        //Well, guess the class is ok, seeing as there is no team theme to begin with.
        return QTRUE;
    }

    let t = *team_ptr;

    //See if the class is listed on the team
    while i < (*t).numClasses {
        if Q_stricmp(classname, (*(*t).classes[i as usize]).name.as_ptr()) == 0 {
            //found it, so it's alright
            return QTRUE;
        }
        i += 1;
    }

    //Didn't find it, so copy the name of the first valid class over it.
    strcpy(classname, (*(*t).classes[0]).name.as_ptr());

    QFALSE
}

//======================================
//Class loading functions (cont.)
//======================================
/// `void BG_SiegeLoadClasses(siegeClassDesc_t *descBuffer)` (bg_saga.c:1183) — reset
/// `bgNumSiegeClasses` and parse every `ext_data/Siege/Classes/*.scl` into
/// `bgSiegeClasses` (descriptions into `descBuffer[i]` if provided).
///
/// No oracle — pure engine-trap file I/O (`trap_FS_GetFileList` + the trap-based
/// `BG_SiegeParseClassFile`).
///
/// # Safety
/// `descBuffer` must be null or point to at least as many `siegeClassDesc_t` as files.
pub unsafe fn BG_SiegeLoadClasses(desc_buffer: *mut siegeClassDesc_t) {
    let mut filelen: c_int; // C: `int filelen;` (uninitialized; set each iteration)
    let mut filelist = [0 as c_char; 4096];
    let mut filename = [0 as c_char; MAX_QPATH];

    bgNumSiegeClasses = 0;

    let num_files = trap::FS_GetFileList("ext_data/Siege/Classes", ".scl", &mut filelist);
    let mut fileptr = filelist.as_mut_ptr();

    let mut i: c_int = 0;
    while i < num_files {
        filelen = strlen(fileptr) as c_int;
        strcpy(filename.as_mut_ptr(), c"ext_data/Siege/Classes/".as_ptr());
        strcat(filename.as_mut_ptr(), fileptr);

        if !desc_buffer.is_null() {
            BG_SiegeParseClassFile(filename.as_ptr(), desc_buffer.add(i as usize));
        } else {
            BG_SiegeParseClassFile(filename.as_ptr(), core::ptr::null_mut());
        }

        i += 1;
        fileptr = fileptr.add((filelen + 1) as usize);
    }
}

//======================================
//Team loading functions
//======================================
/// `void BG_SiegeParseTeamFile(const char *filename)` (bg_saga.c:1237) — parse a
/// `.team` file into the next `bgSiegeTeams` slot: the team `name` (required) and its
/// `class1..classN` membership (resolved via [`BG_SiegeFindClassByName`]).
/// `bgNumSiegeTeams` is incremented on success; malformed input `Com_Error`s.
///
/// QAGAME build: the `FriendlyShader` `trap_R_RegisterShaderNoMip` path is `#ifdef
/// CGAME`, so here `friendlyShader` is just set to 0.
///
/// No oracle — pure engine-trap file I/O (`trap_FS_*`).
///
/// # Safety
/// `filename` must be NUL-terminated.
pub unsafe fn BG_SiegeParseTeamFile(filename: *const c_char) {
    let mut team_info = [0 as c_char; 2048];
    let mut parse_buf = [0 as c_char; 1024];
    let mut look_string = [0 as c_char; 256];
    let mut i: c_int = 1;
    let mut success: qboolean = QTRUE;

    let fname = CStr::from_ptr(filename).to_string_lossy();
    let (len, f) = trap::FS_FOpenFile(&fname, FS_READ);

    if f == 0 || len >= 2048 {
        return;
    }

    {
        let buf = core::slice::from_raw_parts_mut(team_info.as_mut_ptr() as *mut u8, len as usize);
        trap::FS_Read(buf, f);
    }

    trap::FS_FCloseFile(f);

    team_info[len as usize] = 0;

    let idx = bgNumSiegeTeams as usize;
    let tm = addr_of_mut!(bgSiegeTeams[idx]);

    if BG_SiegeGetPairedValue(
        team_info.as_mut_ptr(),
        c"name".as_ptr() as *mut c_char,
        parse_buf.as_mut_ptr(),
    ) != 0
    {
        strcpy((*tm).name.as_mut_ptr(), parse_buf.as_ptr());
    } else {
        Com_Error(ERR_DROP, "Siege team with no name definition");
    }

    // #ifdef CGAME: friendlyShader = trap_R_RegisterShaderNoMip(...). This crate is
    // the QAGAME server module, so it takes the #else path.
    (*tm).friendlyShader = 0;

    (*tm).numClasses = 0;

    if BG_SiegeGetValueGroup(
        team_info.as_mut_ptr(),
        c"Classes".as_ptr() as *mut c_char,
        team_info.as_mut_ptr(),
    ) != 0
    {
        while success != QFALSE && i < MAX_SIEGE_CLASSES as c_int {
            //keep checking for group values named class# up to MAX_SIEGE_CLASSES until we can't find one.
            strcpy(look_string.as_mut_ptr(), va(format_args!("class{}", i)));

            success = BG_SiegeGetPairedValue(
                team_info.as_mut_ptr(),
                look_string.as_mut_ptr(),
                parse_buf.as_mut_ptr(),
            );

            if success == QFALSE {
                break;
            }

            let n = (*tm).numClasses as usize;
            (*tm).classes[n] = BG_SiegeFindClassByName(parse_buf.as_ptr());

            if (*tm).classes[n].is_null() {
                Com_Error(
                    ERR_DROP,
                    &format!("Invalid class specified: '{}'", Sz(parse_buf.as_ptr())),
                );
            }

            (*tm).numClasses += 1;

            i += 1;
        }
    }

    if (*tm).numClasses == 0 {
        Com_Error(ERR_DROP, "Team defined with no allowable classes\n");
    }

    //If we get here then it was a success, so increment the team number
    bgNumSiegeTeams += 1;
}

/// `void BG_SiegeLoadTeams(void)` (bg_saga.c:1316) — reset `bgNumSiegeTeams` and parse
/// every `ext_data/Siege/Teams/*.team` via [`BG_SiegeParseTeamFile`].
///
/// No oracle — pure engine-trap file I/O (`trap_FS_GetFileList`).
pub unsafe fn BG_SiegeLoadTeams() {
    let mut filelen: c_int; // C: `int filelen;` (uninitialized; set each iteration)
    let mut filelist = [0 as c_char; 4096];
    let mut filename = [0 as c_char; MAX_QPATH];

    bgNumSiegeTeams = 0;

    let num_files = trap::FS_GetFileList("ext_data/Siege/Teams", ".team", &mut filelist);
    let mut fileptr = filelist.as_mut_ptr();

    let mut i: c_int = 0;
    while i < num_files {
        filelen = strlen(fileptr) as c_int;
        strcpy(filename.as_mut_ptr(), c"ext_data/Siege/Teams/".as_ptr());
        strcat(filename.as_mut_ptr(), fileptr);
        BG_SiegeParseTeamFile(filename.as_ptr());

        i += 1;
        fileptr = fileptr.add((filelen + 1) as usize);
    }
}

// #ifndef UI_EXPORTS — only for game/cgame (this crate is the QAGAME server module).
/// `void BG_PrecacheSabersForSiegeTeam(int team)` (bg_saga.c:1365) — for every class
/// on the team's theme, parse each of its (up to `MAX_SABERS`) saber names via
/// [`WP_SaberParseParms`] and model-cache the resolved hilt model.
///
/// No oracle — drives [`WP_SaberParseParms`] (saber-file parsing) and
/// [`BG_ModelCache`] (engine model registration), which the off-engine oracle harness
/// cannot satisfy.
pub unsafe fn BG_PrecacheSabersForSiegeTeam(team: c_int) {
    let mut saber: saberInfo_t = core::mem::zeroed();
    let mut saber_name: *const c_char;

    let t = BG_SiegeFindThemeForTeam(team);

    if !t.is_null() {
        let mut i: c_int = 0;

        while i < (*t).numClasses {
            let mut s_num: c_int = 0;

            while s_num < MAX_SABERS as c_int {
                match s_num {
                    0 => saber_name = (*(*t).classes[i as usize]).saber1.as_ptr(),
                    1 => saber_name = (*(*t).classes[i as usize]).saber2.as_ptr(),
                    _ => saber_name = core::ptr::null(),
                }

                if !saber_name.is_null() && *saber_name != 0 {
                    WP_SaberParseParms(saber_name, &mut saber);
                    if Q_stricmp(saber_name, saber.name.as_ptr()) == 0 {
                        //found the matching saber
                        if saber.model[0] != 0 {
                            BG_ModelCache(saber.model.as_ptr(), core::ptr::null());
                        }
                    }
                }

                s_num += 1;
            }

            i += 1;
        }
    }
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::codemp::game::bg_saga_h::{SPC_DEMOLITIONIST, SPC_JEDI, SPC_SUPPORT};
    use crate::oracle::{
        jka_BG_GetClassOnBaseClass, jka_BG_GetUIPortrait, jka_BG_GetUIPortraitFile,
        jka_BG_SiegeCheckClassLegality, jka_BG_SiegeCountBaseClass, jka_BG_SiegeFindClassByName,
        jka_BG_SiegeFindClassIndexByName, jka_BG_SiegeFindTeamForTheme, jka_BG_SiegeFindThemeForTeam,
        jka_BG_SiegeGetPairedValue, jka_BG_SiegeGetValueGroup, jka_BG_SiegeStripTabs,
        jka_BG_SiegeTranslateForcePowers, jka_BG_SiegeTranslateGenericTable,
    };

    /// Serializes every bg_saga test that mutates the module's siege globals
    /// (`team1Theme`/`team2Theme`, `bgSiegeClasses`/`bgNumSiegeClasses`,
    /// `bgSiegeTeams`/`bgNumSiegeTeams`) so the default parallel test runner can't
    /// race them (the `g_mem::POOL_LOCK` precedent).
    static SAGA_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Bytes -> NUL-terminated C buffer.
    fn cbuf(b: &[u8]) -> Vec<c_char> {
        let mut v: Vec<c_char> = b.iter().map(|&x| x as c_char).collect();
        v.push(0);
        v
    }

    /// Bytes -> fixed-`size` zero-padded C buffer (room for an in-place `strcpy`).
    fn padbuf(b: &[u8], size: usize) -> Vec<c_char> {
        let mut v = vec![0 as c_char; size];
        for (i, &x) in b.iter().enumerate() {
            v[i] = x as c_char;
        }
        v
    }

    /// Parity for `BG_SiegeStripTabs`: run a spread of inputs (empty, no tabs, leading/
    /// trailing/consecutive tabs, all tabs) through the Rust port and the verbatim C
    /// oracle on identical buffers and assert byte-equal results.
    #[test]
    fn bg_siege_strip_tabs_matches_oracle() {
        let cases: &[&[u8]] = &[
            b"\0",
            b"no tabs here\0",
            b"\t\0",
            b"\tleading\0",
            b"trailing\t\0",
            b"a\tb\tc\0",
            b"\t\t\tmany\t\t\t\0",
            b"mixed \t spaces\tand\ttabs \0",
        ];

        for case in cases {
            let mut rust_buf: Vec<c_char> = case.iter().map(|&b| b as c_char).collect();
            let mut c_buf: Vec<c_char> = rust_buf.clone();

            unsafe {
                BG_SiegeStripTabs(rust_buf.as_mut_ptr());
                jka_BG_SiegeStripTabs(c_buf.as_mut_ptr());
            }

            assert_eq!(rust_buf, c_buf, "BG_SiegeStripTabs on {case:?}");
        }
    }

    /// Parity for `BG_SiegeGetValueGroup`: drive a spread of well-formed inputs
    /// (group found / not found, nested braces, comments, preceding global values,
    /// a second sibling group, a non-group value sharing the searched name) through
    /// the Rust port and the verbatim C oracle, asserting both the return code and
    /// the full extracted `outbuf` agree.
    #[test]
    fn bg_siege_get_value_group_matches_oracle() {
        let cases: &[(&[u8], &[u8])] = &[
            // simple group found
            (b"GroupA\n{\n\tkey1 value1\n\tkey2 value2\n}\n", b"GroupA"),
            // group not present -> 0
            (b"GroupA\n{\n\tkey1 value1\n}\n", b"GroupZ"),
            // second sibling group, find it
            (
                b"First\n{\n\ta b\n}\nSecond\n{\n\tc d\n}\n",
                b"Second",
            ),
            // nested braces inside the wanted group
            (
                b"Outer\n{\n\tInner\n\t{\n\t\tx y\n\t}\n\tz w\n}\n",
                b"Outer",
            ),
            // a global (non-group) value sharing the name, then the real group
            (
                b"weapons WP_SABER\nGroupA\n{\n\tkey1 value1\n}\n",
                b"GroupA",
            ),
            // a // comment line before the group
            (
                b"//a comment\nGroupA\n{\n\tkey1 value1\n}\n",
                b"GroupA",
            ),
            // a sibling group we skip past to reach the wanted one
            (
                b"Skip\n{\n\tnested\n\t{\n\t\tdeep v\n\t}\n}\nWant\n{\n\tfound it\n}\n",
                b"Want",
            ),
            // group name appears as a plain value first (not followed by a brace)
            (
                b"GroupA somevalue\nGroupA\n{\n\treal stuff\n}\n",
                b"GroupA",
            ),
        ];

        for (input, group) in cases {
            let mut buf_r = cbuf(input);
            let mut buf_c = cbuf(input);
            let mut grp_r = cbuf(group);
            let mut grp_c = cbuf(group);
            let mut out_r = vec![0 as c_char; 8192];
            let mut out_c = vec![0 as c_char; 8192];

            let rr = unsafe {
                BG_SiegeGetValueGroup(buf_r.as_mut_ptr(), grp_r.as_mut_ptr(), out_r.as_mut_ptr())
            };
            let rc = unsafe {
                jka_BG_SiegeGetValueGroup(
                    buf_c.as_mut_ptr(),
                    grp_c.as_mut_ptr(),
                    out_c.as_mut_ptr(),
                )
            };

            assert_eq!(rr, rc, "return for group {group:?} in {input:?}");
            assert_eq!(out_r, out_c, "outbuf for group {group:?} in {input:?}");
        }
    }

    /// Parity for `BG_SiegeGetPairedValue`: drive well-formed inputs (key found /
    /// not found, quoted value, value after a skipped sub-group, key appearing only
    /// inside a sub-group, trailing `//` comment after a value, duplicate-key first
    /// hit wins) through the Rust port and the verbatim C oracle, asserting both the
    /// return code and the full extracted `outbuf` agree.
    #[test]
    fn bg_siege_get_paired_value_matches_oracle() {
        let cases: &[(&[u8], &[u8])] = &[
            // simple pair found
            (b"name infantry\nhealth 100\n", b"health"),
            // key not present -> 0
            (b"name infantry\nhealth 100\n", b"armor"),
            // quoted value (allows embedded spaces)
            (b"name \"Imperial Officer\"\nhealth 100\n", b"name"),
            // key only exists inside a sub-group -> not found at top level
            (b"saberInfo\n{\n\tsaber1 single_1\n}\nhealth 50\n", b"saber1"),
            // a value pair after skipping a sub-group
            (b"group\n{\n\tinner v\n}\nspeed 250\n", b"speed"),
            // trailing // comment after the value
            (b"weapon WP_SABER //the saber\nhealth 100\n", b"weapon"),
            // duplicate key: first hit wins
            (b"health 100\nhealth 200\n", b"health"),
            // tab-separated key/value
            (b"name\tinfantry\nhealth\t75\n", b"name"),
        ];

        for (input, key) in cases {
            let mut buf_r = cbuf(input);
            let mut buf_c = cbuf(input);
            let mut key_r = cbuf(key);
            let mut key_c = cbuf(key);
            let mut out_r = vec![0 as c_char; 8192];
            let mut out_c = vec![0 as c_char; 8192];

            let rr = unsafe {
                BG_SiegeGetPairedValue(buf_r.as_mut_ptr(), key_r.as_mut_ptr(), out_r.as_mut_ptr())
            };
            let rc = unsafe {
                jka_BG_SiegeGetPairedValue(
                    buf_c.as_mut_ptr(),
                    key_c.as_mut_ptr(),
                    out_c.as_mut_ptr(),
                )
            };

            assert_eq!(rr, rc, "return for key {key:?} in {input:?}");
            assert_eq!(out_r, out_c, "outbuf for key {key:?} in {input:?}");
        }
    }

    /// Parity for `BG_SiegeTranslateForcePowers`: drive a spread of power lists
    /// (FP_ALL, "0", explicit/default levels, level clamping, the FP_JUMP alias,
    /// spaces around `|`, unknown names) through the Rust port and the verbatim C
    /// oracle on identical zeroed `siegeClass_t`s, asserting `forcePowerLevels` agree.
    #[test]
    fn bg_siege_translate_force_powers_matches_oracle() {
        let cases: &[&[u8]] = &[
            b"FP_ALL",
            b"0",
            b"",
            b"FP_HEAL,2|FP_PUSH,3|FP_LIGHTNING",
            b"FP_JUMP,1",
            b"FP_SABERTHROW,9",
            b"FP_HEAL,-2",
            b"FP_GRIP | FP_DRAIN,4",
            b"FP_BOGUS,3|FP_SEE,2",
            b"FP_SPEED,2 FP_RAGE,1",
            b"FP_LEVITATION,5|FP_LEVITATION,1",
        ];

        for case in cases {
            let mut buf_r = cbuf(case);
            let mut buf_c = cbuf(case);
            let mut sc_r: siegeClass_t = unsafe { core::mem::zeroed() };
            let mut sc_c: siegeClass_t = unsafe { core::mem::zeroed() };

            unsafe {
                BG_SiegeTranslateForcePowers(buf_r.as_mut_ptr(), &mut sc_r);
                jka_BG_SiegeTranslateForcePowers(buf_c.as_mut_ptr(), &mut sc_c);
            }

            assert_eq!(
                sc_r.forcePowerLevels, sc_c.forcePowerLevels,
                "forcePowerLevels for {case:?}"
            );
        }
    }

    /// Parity for `BG_SiegeTranslateGenericTable`: drive each ported table (the same
    /// `static mut` pointer is handed to both the Rust port and the verbatim C
    /// oracle) over bitflag-accumulate and direct-id inputs — known names, the "0"
    /// special case, the WP_BLASTER_PISTOL alias, spaces around `|`, and unknown
    /// names — asserting the returned ints agree.
    #[test]
    fn bg_siege_translate_generic_table_matches_oracle() {
        unsafe fn check(buf: &[u8], table: *const stringID_table_t, bitflag: c_int) {
            let mut buf_r: Vec<c_char> = buf.iter().map(|&b| b as c_char).collect();
            buf_r.push(0);
            let mut buf_c = buf_r.clone();
            let rr = BG_SiegeTranslateGenericTable(buf_r.as_mut_ptr(), table, bitflag);
            let rc =
                jka_BG_SiegeTranslateGenericTable(buf_c.as_mut_ptr(), table, bitflag);
            assert_eq!(rr, rc, "generic table for {buf:?} bitflag={bitflag}");
        }

        unsafe {
            let wp = addr_of!(WPTable) as *const stringID_table_t;
            let stance = addr_of!(StanceTable) as *const stringID_table_t;
            let flags = addr_of!(bgSiegeClassFlagNames) as *const stringID_table_t;
            let hold = addr_of!(HoldableTable) as *const stringID_table_t;
            let pw = addr_of!(PowerupTable) as *const stringID_table_t;

            // bitflag accumulation
            check(b"WP_SABER|WP_BLASTER|WP_ROCKET_LAUNCHER", wp, QTRUE);
            check(b"WP_SABER | WP_BLASTER", wp, QTRUE);
            check(b"CFL_HEAVYMELEE|CFL_EXTRA_AMMO", flags, QTRUE);
            check(b"HI_SEEKER|HI_EWEB|HI_CLOAK", hold, QTRUE);
            check(b"PW_QUAD|PW_YSALAMIRI", pw, QTRUE);
            // direct-id (bitflag false): first match returned
            check(b"SS_STAFF", stance, QFALSE);
            check(b"WP_ROCKET_LAUNCHER", wp, QFALSE);
            check(b"WP_BLASTER_PISTOL", wp, QFALSE); // alias -> WP_BRYAR_PISTOL
            // "0" special case and unknowns
            check(b"0", wp, QTRUE);
            check(b"0", stance, QFALSE);
            check(b"WP_BOGUS|WP_SABER", wp, QTRUE);
            check(b"NOPE", stance, QFALSE);
        }
    }

    /// Parity for the theme-based class lookups (`BG_SiegeFindThemeForTeam`,
    /// `BG_SiegeCountBaseClass`, `BG_GetUIPortraitFile`, `BG_GetUIPortrait`,
    /// `BG_GetClassOnBaseClass`). Builds a team owning five classes (three sharing
    /// `SPC_INFANTRY`), points the Rust `team1Theme`/`team2Theme` globals at it, and
    /// compares each Rust function (full global-reading path) against the verbatim C
    /// oracle (handed the same team), across every base class and a range of
    /// count-indices plus the null-theme path. Returned pointers are bit-identical
    /// because both sides walk the same class objects.
    #[test]
    fn bg_siege_team_lookups_match_oracle() {
        let _g = SAGA_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        unsafe {
            let pcs = [SPC_INFANTRY, SPC_JEDI, SPC_INFANTRY, SPC_SUPPORT, SPC_INFANTRY];
            let mut classes: Vec<siegeClass_t> = pcs
                .iter()
                .enumerate()
                .map(|(i, &pc)| {
                    let mut c: siegeClass_t = core::mem::zeroed();
                    c.playerClass = pc as c_short;
                    c.uiPortraitShader = 100 + i as c_int;
                    let s = format!("portrait{i}");
                    for (j, b) in s.bytes().enumerate() {
                        c.uiPortrait[j] = b as c_char;
                    }
                    c
                })
                .collect();

            let mut team: siegeTeam_t = core::mem::zeroed();
            team.numClasses = classes.len() as c_int;
            for i in 0..classes.len() {
                team.classes[i] = &mut classes[i];
            }
            let mut team2: siegeTeam_t = core::mem::zeroed();

            team1Theme = &mut team;
            team2Theme = &mut team2;

            // BG_SiegeFindThemeForTeam
            for t in [SIEGETEAM_TEAM1, SIEGETEAM_TEAM2, 3, 0] {
                assert_eq!(
                    BG_SiegeFindThemeForTeam(t),
                    jka_BG_SiegeFindThemeForTeam(t, &mut team, &mut team2),
                    "FindThemeForTeam team={t}"
                );
            }

            let stm: *mut siegeTeam_t = &mut team;
            for ci in [SPC_INFANTRY, SPC_JEDI, SPC_SUPPORT, SPC_DEMOLITIONIST] {
                let ci = ci as c_short;
                assert_eq!(
                    BG_SiegeCountBaseClass(SIEGETEAM_TEAM1, ci),
                    jka_BG_SiegeCountBaseClass(stm, ci),
                    "Count ci={ci}"
                );
                for cnt in 0..4i16 {
                    assert_eq!(
                        BG_GetUIPortraitFile(SIEGETEAM_TEAM1, ci, cnt),
                        jka_BG_GetUIPortraitFile(stm, ci, cnt),
                        "PortraitFile ci={ci} cnt={cnt}"
                    );
                    assert_eq!(
                        BG_GetUIPortrait(SIEGETEAM_TEAM1, ci, cnt),
                        jka_BG_GetUIPortrait(stm, ci, cnt),
                        "Portrait ci={ci} cnt={cnt}"
                    );
                    assert_eq!(
                        BG_GetClassOnBaseClass(SIEGETEAM_TEAM1, ci, cnt),
                        jka_BG_GetClassOnBaseClass(stm, ci, cnt),
                        "ClassOnBase ci={ci} cnt={cnt}"
                    );
                }
            }

            // null-theme path (team 3 -> FindThemeForTeam returns null)
            assert_eq!(
                BG_SiegeCountBaseClass(3, SPC_INFANTRY as c_short),
                jka_BG_SiegeCountBaseClass(core::ptr::null_mut(), SPC_INFANTRY as c_short),
                "Count null theme"
            );

            team1Theme = core::ptr::null_mut();
            team2Theme = core::ptr::null_mut();
        }
    }

    /// Parity for the class/team finders (`BG_SiegeFindClassByName`,
    /// `BG_SiegeFindClassIndexByName`, `BG_SiegeFindTeamForTheme`,
    /// `BG_SiegeSetTeamTheme`, `BG_SiegeCheckClassLegality`). Populates the Rust
    /// `bgSiegeClasses`/`bgSiegeTeams`/theme globals, then compares each Rust
    /// function (real global-reading path) against the verbatim C oracle (handed the
    /// same arrays/themes). `SetTeamTheme` is checked for consistency against
    /// `FindTeamForTheme`; `CheckClassLegality` also compares the in-place `classname`
    /// overwrite on the illegal path.
    #[test]
    fn bg_siege_finders_match_oracle() {
        let _g = SAGA_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        unsafe {
            // FindClassByName / FindClassIndexByName: populate bgSiegeClasses
            let cbase = addr_of_mut!(bgSiegeClasses) as *mut siegeClass_t;
            for (i, nm) in ["alpha", "beta", "gamma"].iter().enumerate() {
                let c = cbase.add(i);
                (*c).name = [0; 512];
                for (j, b) in nm.bytes().enumerate() {
                    (*c).name[j] = b as c_char;
                }
            }
            bgNumSiegeClasses = 3;

            for q in [&b"beta"[..], &b"ALPHA"[..], &b"gamma"[..], &b"zeta"[..], &b""[..]] {
                let qn = cbuf(q);
                assert_eq!(
                    BG_SiegeFindClassByName(qn.as_ptr()),
                    jka_BG_SiegeFindClassByName(qn.as_ptr(), cbase, bgNumSiegeClasses),
                    "FindClassByName {q:?}"
                );
                assert_eq!(
                    BG_SiegeFindClassIndexByName(qn.as_ptr()),
                    jka_BG_SiegeFindClassIndexByName(qn.as_ptr(), cbase, bgNumSiegeClasses),
                    "FindClassIndexByName {q:?}"
                );
            }

            // FindTeamForTheme / SetTeamTheme: populate bgSiegeTeams
            let tbase = addr_of_mut!(bgSiegeTeams) as *mut siegeTeam_t;
            for (i, nm) in ["red", "blue"].iter().enumerate() {
                let t = tbase.add(i);
                (*t).name = [0; 512];
                for (j, b) in nm.bytes().enumerate() {
                    (*t).name[j] = b as c_char;
                }
            }
            bgNumSiegeTeams = 2;

            for q in [&b"blue"[..], &b"RED"[..], &b"green"[..], &b""[..]] {
                let mut qn = cbuf(q);
                assert_eq!(
                    BG_SiegeFindTeamForTheme(qn.as_mut_ptr()),
                    jka_BG_SiegeFindTeamForTheme(qn.as_mut_ptr(), tbase, bgNumSiegeTeams),
                    "FindTeamForTheme {q:?}"
                );
            }

            // SetTeamTheme sets the theme global to FindTeamForTheme(name)
            let mut red = cbuf(b"red");
            BG_SiegeSetTeamTheme(SIEGETEAM_TEAM1, red.as_mut_ptr());
            assert_eq!(*addr_of!(team1Theme), BG_SiegeFindTeamForTheme(red.as_mut_ptr()));
            let mut blue = cbuf(b"blue");
            BG_SiegeSetTeamTheme(SIEGETEAM_TEAM2, blue.as_mut_ptr());
            assert_eq!(*addr_of!(team2Theme), BG_SiegeFindTeamForTheme(blue.as_mut_ptr()));
            let mut green = cbuf(b"green");
            BG_SiegeSetTeamTheme(SIEGETEAM_TEAM1, green.as_mut_ptr());
            assert!(team1Theme.is_null());

            // CheckClassLegality: build a 2-class team
            let mut cls: Vec<siegeClass_t> = ["alpha", "beta"]
                .iter()
                .map(|nm| {
                    let mut c: siegeClass_t = core::mem::zeroed();
                    for (j, b) in nm.bytes().enumerate() {
                        c.name[j] = b as c_char;
                    }
                    c
                })
                .collect();
            let mut team: siegeTeam_t = core::mem::zeroed();
            team.numClasses = cls.len() as c_int;
            for i in 0..cls.len() {
                team.classes[i] = &mut cls[i];
            }
            team1Theme = &mut team;
            team2Theme = core::ptr::null_mut();

            for (team_id, cn) in [
                (SIEGETEAM_TEAM1, &b"beta"[..]),
                (SIEGETEAM_TEAM1, &b"alpha"[..]),
                (SIEGETEAM_TEAM1, &b"illegalclass"[..]),
                (99, &b"whatever"[..]),
            ] {
                let mut buf_r = padbuf(cn, 256);
                let mut buf_c = padbuf(cn, 256);
                let rr = BG_SiegeCheckClassLegality(team_id, buf_r.as_mut_ptr());
                let rc = jka_BG_SiegeCheckClassLegality(
                    team_id,
                    buf_c.as_mut_ptr(),
                    &mut team,
                    core::ptr::null_mut(),
                );
                assert_eq!(rr, rc, "CheckClassLegality return team={team_id} cn={cn:?}");
                assert_eq!(buf_r, buf_c, "CheckClassLegality classname team={team_id} cn={cn:?}");
            }

            // no-theme path: team1Theme null -> always legal
            team1Theme = core::ptr::null_mut();
            {
                let mut buf_r = padbuf(b"x", 256);
                let mut buf_c = padbuf(b"x", 256);
                let rr = BG_SiegeCheckClassLegality(SIEGETEAM_TEAM1, buf_r.as_mut_ptr());
                let rc = jka_BG_SiegeCheckClassLegality(
                    SIEGETEAM_TEAM1,
                    buf_c.as_mut_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                assert_eq!(rr, rc, "CheckClassLegality null-theme return");
                assert_eq!(buf_r, buf_c, "CheckClassLegality null-theme classname");
            }

            // reset globals
            bgNumSiegeClasses = 0;
            bgNumSiegeTeams = 0;
            team1Theme = core::ptr::null_mut();
            team2Theme = core::ptr::null_mut();
        }
    }
}
