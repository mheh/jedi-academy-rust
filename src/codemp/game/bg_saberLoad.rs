//! `bg_saberLoad.c` — the shared (both-games) saber **definition loader**: parses
//! `sabers.cfg` / `*.sab` into [`saberInfo_t`], plus the small saber accessor and
//! translate helpers. This slice lands the file's **data-layer + self-contained
//! logic** (data-layer-first per the 1.03 roadmap):
//!
//! - [`SaberTable`] — the `ENUM2STRING` saber-type name table (for `VF_SABER`-style
//!   lookups via `GetIDForString`), the `VehicleTable` precedent.
//! - [`TranslateSaberColor`] / [`TranslateSaberStyle`] — name → enum string maps.
//! - [`BG_ParseLiteral`] — the shared "required keyword" parse step (also used by
//!   NPC code), over the verified [`COM_ParseExt`](crate::codemp::game::q_shared).
//! - the `BG_SI_*` / `BG_BLADE_*` accessor family — pure [`saberInfo_t`] /
//!   [`bladeInfo_t`] field math (activate/length/trail helpers).
//! - [`WP_SaberSetColor`] — sets a blade color from a name.
//! - [`BG_SoundIndex`] / [`WP_SaberSetDefaults`] / [`WP_RemoveSaber`] — these route
//!   through [`G_SoundIndex`](crate::codemp::game::g_utils::G_SoundIndex) (engine
//!   configstring traps), so — like the rest of the trap-facing surface — they are
//!   faithful ports with **no C oracle**.
//! - the `.sab` **parser family** (now landed): [`WP_SaberParseParms`] (the
//!   `{ ... }` block parser), [`WP_SaberParseParm`] (single-value), [`WP_SaberValidForPlayerInMP`],
//!   [`WP_SetSaber`], and [`WP_SaberLoadParms`] (the file loader) — plus the supporting
//!   [`SaberParms`] text block, the shared `bg_saga::FPTable` (forceRestrict; its
//!   canonical home now that `bg_saga.c` has landed), and [`DEFAULT_SABER`]. These
//!   are all engine-facing (`BG_SoundIndex` configstrings, `trap_R_RegisterSkin`,
//!   `trap_FS_*` file I/O, the parser cursor), so they have **no C oracle**. This
//!   completes `bg_saberLoad.c` for the **QAGAME** server build.
//!
//! Oracle: the pointer-free data/logic is verified against the real Raven C in
//! `oracle/bg_saberLoad_oracle.c` — `SaberTable` is driven through the authentic
//! `GetIDForString`/`GetStringForID` (real q_shared.c); the translate fns and the
//! `saberInfo_t` accessors are checked against verbatim C bodies operating on a
//! verbatim struct transcription (the layout itself is verified in
//! `q_shared_h_oracle.c`).

#![allow(non_upper_case_globals, non_snake_case)]

use crate::codemp::cgame::animtable::animTable;
use crate::codemp::game::anims::MAX_ANIMATIONS;
use crate::codemp::game::bg_public::{
    LS_A1_SPECIAL, LS_A2_SPECIAL, LS_A3_SPECIAL, LS_A_BACK, LS_A_BACKFLIP_ATK, LS_A_BACKSTAB,
    LS_A_BACK_CR, LS_A_BL2TR, LS_A_BR2TL, LS_A_FLIP_SLASH, LS_A_FLIP_STAB, LS_A_JUMP_T__B_,
    LS_A_L2R, LS_A_LUNGE, LS_A_R2L, LS_A_T2B, LS_A_TL2BR, LS_A_TR2BL, LS_BUTTERFLY_LEFT,
    LS_BUTTERFLY_RIGHT, LS_DUAL_FB, LS_DUAL_LR, LS_DUAL_SPIN_PROTECT, LS_HILT_BASH, LS_INVALID,
    LS_JUMPATTACK_ARIAL_LEFT, LS_JUMPATTACK_ARIAL_RIGHT, LS_JUMPATTACK_CART_LEFT,
    LS_JUMPATTACK_CART_RIGHT, LS_JUMPATTACK_DUAL, LS_JUMPATTACK_STAFF_LEFT,
    LS_JUMPATTACK_STAFF_RIGHT, LS_KICK_B, LS_KICK_BF, LS_KICK_B_AIR, LS_KICK_F, LS_KICK_F_AIR,
    LS_KICK_L, LS_KICK_L_AIR, LS_KICK_R, LS_KICK_RL, LS_KICK_R_AIR, LS_KICK_S, LS_LEAP_ATTACK,
    LS_MOVE_MAX, LS_NONE, LS_PULL_ATTACK_STAB, LS_PULL_ATTACK_SWING, LS_ROLL_STAB, LS_SPINATTACK,
    LS_SPINATTACK_ALORA, LS_SPINATTACK_DUAL, LS_STABDOWN, LS_STABDOWN_DUAL, LS_STABDOWN_STAFF,
    LS_STAFF_SOULCAL, LS_SWOOP_ATTACK_LEFT, LS_SWOOP_ATTACK_RIGHT, LS_TAUNTAUN_ATTACK_LEFT,
    LS_TAUNTAUN_ATTACK_RIGHT, LS_UPSIDE_DOWN_ATTACK,
};
use crate::codemp::game::bg_saga::FPTable;
use crate::codemp::game::g_main::{Com_Error, Com_Printf};
use crate::codemp::game::g_utils::G_SoundIndex;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared::{
    COM_BeginParseSession, COM_Compress, COM_ParseExt, COM_ParseFloat, COM_ParseInt,
    COM_ParseString, GetIDForString, Q_strcat, Q_stricmp, Q_stricmpn, SkipBracedSection,
    SkipRestOfLine, Sz,
};
use crate::codemp::game::q_shared_h::{
    bladeInfo_t, qboolean, saberInfo_t, saberType_t, saber_colors_t, saber_styles_t,
    stringID_table_t, ERR_DROP, FP_FIRST, FS_READ, MAX_BLADES, MAX_CLIENTS, NUM_FORCE_POWERS,
    NUM_SABERS, QFALSE, QTRUE, SABER_ARC, SABER_BLUE, SABER_BROAD, SABER_CLAW, SABER_DAGGER,
    SABER_GREEN, SABER_LANCE, SABER_NONE, SABER_ORANGE, SABER_PRONG, SABER_PURPLE, SABER_RED,
    SABER_SAI, SABER_SINGLE, SABER_STAFF, SABER_STAR, SABER_TRIDENT, SABER_YELLOW, SS_DESANN,
    SS_DUAL, SS_FAST, SS_MEDIUM, SS_NONE, SS_STAFF, SS_STRONG, SS_TAVION,
};
use crate::codemp::game::q_shared_h::{
    SFL2_ALWAYS_BLOCK, SFL2_ALWAYS_BLOCK2, SFL2_NO_BLADE, SFL2_NO_BLADE2, SFL2_NO_CLASH_FLARE,
    SFL2_NO_CLASH_FLARE2, SFL2_NO_DISMEMBERMENT, SFL2_NO_DISMEMBERMENT2, SFL2_NO_DLIGHT,
    SFL2_NO_DLIGHT2, SFL2_NO_IDLE_EFFECT, SFL2_NO_IDLE_EFFECT2, SFL2_NO_MANUAL_DEACTIVATE,
    SFL2_NO_MANUAL_DEACTIVATE2, SFL2_NO_WALL_MARKS, SFL2_NO_WALL_MARKS2, SFL2_TRANSITION_DAMAGE,
    SFL2_TRANSITION_DAMAGE2, SFL_BOLT_TO_WRIST, SFL_BOUNCE_ON_WALLS, SFL_NOT_ACTIVE_BLOCKING,
    SFL_NOT_DISARMABLE, SFL_NOT_LOCKABLE, SFL_NOT_THROWABLE, SFL_NO_BACK_ATTACK, SFL_NO_CARTWHEELS,
    SFL_NO_FLIPS, SFL_NO_KICKS, SFL_NO_MIRROR_ATTACKS, SFL_NO_PULL_ATTACK, SFL_NO_ROLLS,
    SFL_NO_ROLL_STAB, SFL_NO_STABDOWN, SFL_NO_WALL_FLIPS, SFL_NO_WALL_GRAB, SFL_NO_WALL_RUNS,
    SFL_RETURN_DAMAGE, SFL_SINGLE_BLADE_THROWABLE, SFL_TWO_HANDED, SS_NUM_SABER_STYLES,
};
use crate::codemp::game::w_saber_h::SABER_RADIUS_STANDARD;
use crate::trap;
use core::ffi::{c_char, c_int, CStr};
use core::ptr::{addr_of, addr_of_mut};

extern "C" {
    /// libc `char *strcpy(char *dest, const char *src)` — copies the literal saber
    /// defaults verbatim, as the C source does.
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    /// libc `size_t strlen(const char *s)` — distinguishes the indexed
    /// `saberColorN`/`saberLengthN`/`saberRadiusN` parm forms from the bare keyword.
    fn strlen(s: *const c_char) -> usize;
    /// libc `int atoi(const char *nptr)` — decodes the trailing blade-index digit.
    fn atoi(s: *const c_char) -> c_int;
}

/// `#define MAX_SABER_DATA_SIZE 0x80000` (bg_saberLoad.c:43) — size of the
/// [`SaberParms`] text block holding every concatenated `.sab` file.
const MAX_SABER_DATA_SIZE: usize = 0x80000;

/// `char SaberParms[MAX_SABER_DATA_SIZE]` — the concatenated text of every loaded
/// `.sab` file, tokenized on demand by [`WP_SaberParseParms`]/[`WP_SaberParseParm`].
/// In Raven's tree this is `extern` here with the sole definition in the UI module
/// (ui_saber.c:24); the game module's definition is absent from the public source
/// (the original line-51 `static char SaberParms[]` was externalized). We give the
/// game module its own file-local buffer — the home OpenJK uses (`static char
/// saberParms[]`) and what the commented line 51 originally was.
static mut SaberParms: [c_char; MAX_SABER_DATA_SIZE] = [0; MAX_SABER_DATA_SIZE];

/// `#define DEFAULT_SABER "Kyle"` (bg_saberLoad.c:259) — the fallback hilt loaded
/// when a requested saber name is empty or is not found in [`SaberParms`].
const DEFAULT_SABER: &CStr = c"Kyle";

/// `int BG_SoundIndex(char *sound)` (bg_saberLoad.c:39) — under `QAGAME` (this
/// module), forwards to [`G_SoundIndex`]. Like `G_SoundIndex` it takes a Rust `&str`
/// and is engine-facing (registers a configstring through the engine), so it has no
/// C oracle.
pub fn BG_SoundIndex(sound: &str) -> c_int {
    G_SoundIndex(sound)
}

// `stringID_table_t FPTable[]` — the force-power name table scanned by `GetIDForString`
// for the `.sab` `forceRestrict` parm. Its canonical C home is bg_saga.c:102, declared
// `extern` in bg_saberLoad.c; now that `bg_saga.c` has landed it is the single shared
// global, imported above as `bg_saga::FPTable` (the placeholder copy that lived here
// until then is gone). Same `ENUM2STRING` shape / `{ "", -1 }` terminator as `SaberTable`.

/// Builds one [`stringID_table_t`] row from the C `ENUM2STRING(arg)` macro
/// (`{ "arg", arg }`) — `name` points at a `'static` C-string literal; the table
/// never writes through it, so the field's `*const c_char` is sound. (The
/// `bg_vehicleLoad::VehicleTable` `enum2string` precedent.)
const fn enum2string(name: &'static CStr, id: saberType_t) -> stringID_table_t {
    stringID_table_t {
        name: name.as_ptr(),
        id,
    }
}

/// `stringID_table_t SaberTable[]` (bg_saberLoad.c:53) — the saber-hilt-type name
/// table, scanned by `GetIDForString` / `GetStringForID`. The C global is
/// non-`const` and `stringID_table_t` carries a raw `char *`, so it mirrors the C
/// mutable global as `static mut` (the `VehicleTable` precedent). The `{ "", -1 }`
/// row terminates the scan.
pub static mut SaberTable: [stringID_table_t; 13] = [
    enum2string(c"SABER_NONE", SABER_NONE),
    enum2string(c"SABER_SINGLE", SABER_SINGLE),
    enum2string(c"SABER_STAFF", SABER_STAFF),
    enum2string(c"SABER_BROAD", SABER_BROAD),
    enum2string(c"SABER_PRONG", SABER_PRONG),
    enum2string(c"SABER_DAGGER", SABER_DAGGER),
    enum2string(c"SABER_ARC", SABER_ARC),
    enum2string(c"SABER_SAI", SABER_SAI),
    enum2string(c"SABER_CLAW", SABER_CLAW),
    enum2string(c"SABER_LANCE", SABER_LANCE),
    enum2string(c"SABER_STAR", SABER_STAR),
    enum2string(c"SABER_TRIDENT", SABER_TRIDENT),
    stringID_table_t {
        name: c"".as_ptr(),
        id: -1,
    },
];

/// `stringID_table_t SaberMoveTable[]` (bg_saberLoad.c:63) — the saber-move name table,
/// scanned by `GetIDForString` for the `.sab` `kataMove`/`lungeAtkMove`/`jumpAtk*Move`
/// parms. Same `ENUM2STRING` shape / `{ "", -1 }` terminator as [`SaberTable`]; the
/// id values are the `LS_*` `saberMoveName_t` move ids (also a `c_int` typedef, so the
/// shared [`enum2string`] helper applies). Mirrors the non-`const` C global as
/// `pub static mut`.
pub static mut SaberMoveTable: [stringID_table_t; 60] = [
    enum2string(c"LS_NONE", LS_NONE),
    // Attacks
    enum2string(c"LS_A_TL2BR", LS_A_TL2BR),
    enum2string(c"LS_A_L2R", LS_A_L2R),
    enum2string(c"LS_A_BL2TR", LS_A_BL2TR),
    enum2string(c"LS_A_BR2TL", LS_A_BR2TL),
    enum2string(c"LS_A_R2L", LS_A_R2L),
    enum2string(c"LS_A_TR2BL", LS_A_TR2BL),
    enum2string(c"LS_A_T2B", LS_A_T2B),
    enum2string(c"LS_A_BACKSTAB", LS_A_BACKSTAB),
    enum2string(c"LS_A_BACK", LS_A_BACK),
    enum2string(c"LS_A_BACK_CR", LS_A_BACK_CR),
    enum2string(c"LS_ROLL_STAB", LS_ROLL_STAB),
    enum2string(c"LS_A_LUNGE", LS_A_LUNGE),
    enum2string(c"LS_A_JUMP_T__B_", LS_A_JUMP_T__B_),
    enum2string(c"LS_A_FLIP_STAB", LS_A_FLIP_STAB),
    enum2string(c"LS_A_FLIP_SLASH", LS_A_FLIP_SLASH),
    enum2string(c"LS_JUMPATTACK_DUAL", LS_JUMPATTACK_DUAL),
    enum2string(c"LS_JUMPATTACK_ARIAL_LEFT", LS_JUMPATTACK_ARIAL_LEFT),
    enum2string(c"LS_JUMPATTACK_ARIAL_RIGHT", LS_JUMPATTACK_ARIAL_RIGHT),
    enum2string(c"LS_JUMPATTACK_CART_LEFT", LS_JUMPATTACK_CART_LEFT),
    enum2string(c"LS_JUMPATTACK_CART_RIGHT", LS_JUMPATTACK_CART_RIGHT),
    enum2string(c"LS_JUMPATTACK_STAFF_LEFT", LS_JUMPATTACK_STAFF_LEFT),
    enum2string(c"LS_JUMPATTACK_STAFF_RIGHT", LS_JUMPATTACK_STAFF_RIGHT),
    enum2string(c"LS_BUTTERFLY_LEFT", LS_BUTTERFLY_LEFT),
    enum2string(c"LS_BUTTERFLY_RIGHT", LS_BUTTERFLY_RIGHT),
    enum2string(c"LS_A_BACKFLIP_ATK", LS_A_BACKFLIP_ATK),
    enum2string(c"LS_SPINATTACK_DUAL", LS_SPINATTACK_DUAL),
    enum2string(c"LS_SPINATTACK", LS_SPINATTACK),
    enum2string(c"LS_LEAP_ATTACK", LS_LEAP_ATTACK),
    enum2string(c"LS_SWOOP_ATTACK_RIGHT", LS_SWOOP_ATTACK_RIGHT),
    enum2string(c"LS_SWOOP_ATTACK_LEFT", LS_SWOOP_ATTACK_LEFT),
    enum2string(c"LS_TAUNTAUN_ATTACK_RIGHT", LS_TAUNTAUN_ATTACK_RIGHT),
    enum2string(c"LS_TAUNTAUN_ATTACK_LEFT", LS_TAUNTAUN_ATTACK_LEFT),
    enum2string(c"LS_KICK_F", LS_KICK_F),
    enum2string(c"LS_KICK_B", LS_KICK_B),
    enum2string(c"LS_KICK_R", LS_KICK_R),
    enum2string(c"LS_KICK_L", LS_KICK_L),
    enum2string(c"LS_KICK_S", LS_KICK_S),
    enum2string(c"LS_KICK_BF", LS_KICK_BF),
    enum2string(c"LS_KICK_RL", LS_KICK_RL),
    enum2string(c"LS_KICK_F_AIR", LS_KICK_F_AIR),
    enum2string(c"LS_KICK_B_AIR", LS_KICK_B_AIR),
    enum2string(c"LS_KICK_R_AIR", LS_KICK_R_AIR),
    enum2string(c"LS_KICK_L_AIR", LS_KICK_L_AIR),
    enum2string(c"LS_STABDOWN", LS_STABDOWN),
    enum2string(c"LS_STABDOWN_STAFF", LS_STABDOWN_STAFF),
    enum2string(c"LS_STABDOWN_DUAL", LS_STABDOWN_DUAL),
    enum2string(c"LS_DUAL_SPIN_PROTECT", LS_DUAL_SPIN_PROTECT),
    enum2string(c"LS_STAFF_SOULCAL", LS_STAFF_SOULCAL),
    enum2string(c"LS_A1_SPECIAL", LS_A1_SPECIAL),
    enum2string(c"LS_A2_SPECIAL", LS_A2_SPECIAL),
    enum2string(c"LS_A3_SPECIAL", LS_A3_SPECIAL),
    enum2string(c"LS_UPSIDE_DOWN_ATTACK", LS_UPSIDE_DOWN_ATTACK),
    enum2string(c"LS_PULL_ATTACK_STAB", LS_PULL_ATTACK_STAB),
    enum2string(c"LS_PULL_ATTACK_SWING", LS_PULL_ATTACK_SWING),
    enum2string(c"LS_SPINATTACK_ALORA", LS_SPINATTACK_ALORA),
    enum2string(c"LS_DUAL_FB", LS_DUAL_FB),
    enum2string(c"LS_DUAL_LR", LS_DUAL_LR),
    enum2string(c"LS_HILT_BASH", LS_HILT_BASH),
    stringID_table_t {
        name: c"".as_ptr(),
        id: -1,
    },
];

/// `qboolean BG_ParseLiteral( const char **data, const char *string )`
/// (bg_saberLoad.c:71) — the shared "required keyword" parse step (also used in NPC
/// code): pull the next token and require it to equal `string` (case-insensitively).
/// Returns `qtrue` (an error) on EOF or mismatch, else `qfalse`; prints a diagnostic
/// on failure.
///
/// # Safety
/// `data` must point at a valid parser cursor (`*data` a NUL-terminated C string);
/// `string` must be a valid NUL-terminated C string.
pub unsafe fn BG_ParseLiteral(data: *mut *const c_char, string: *const c_char) -> qboolean {
    let token = COM_ParseExt(data, QTRUE);
    if *token == 0 {
        Com_Printf("unexpected EOF\n");
        return QTRUE;
    }

    if Q_stricmp(token, string) != 0 {
        Com_Printf(&format!("required string '{}' missing\n", Sz(string)));
        return QTRUE;
    }

    QFALSE
}

/// `saber_colors_t TranslateSaberColor( const char *name )` (bg_saberLoad.c:91) —
/// maps a color name to a blade color; "random" picks one of orange..purple, and
/// any unrecognized name falls back to `SABER_BLUE`.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
pub unsafe fn TranslateSaberColor(name: *const c_char) -> saber_colors_t {
    if Q_stricmp(name, c"red".as_ptr()) == 0 {
        return SABER_RED;
    }
    if Q_stricmp(name, c"orange".as_ptr()) == 0 {
        return SABER_ORANGE;
    }
    if Q_stricmp(name, c"yellow".as_ptr()) == 0 {
        return SABER_YELLOW;
    }
    if Q_stricmp(name, c"green".as_ptr()) == 0 {
        return SABER_GREEN;
    }
    if Q_stricmp(name, c"blue".as_ptr()) == 0 {
        return SABER_BLUE;
    }
    if Q_stricmp(name, c"purple".as_ptr()) == 0 {
        return SABER_PURPLE;
    }
    if Q_stricmp(name, c"random".as_ptr()) == 0 {
        return Q_irand(SABER_ORANGE, SABER_PURPLE) as saber_colors_t;
    }
    SABER_BLUE
}

/// `saber_styles_t TranslateSaberStyle( const char *name )` (bg_saberLoad.c:124) —
/// maps a style name to a locked saber style; any unrecognized name → `SS_NONE`.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
pub unsafe fn TranslateSaberStyle(name: *const c_char) -> saber_styles_t {
    if Q_stricmp(name, c"fast".as_ptr()) == 0 {
        return SS_FAST;
    }
    if Q_stricmp(name, c"medium".as_ptr()) == 0 {
        return SS_MEDIUM;
    }
    if Q_stricmp(name, c"strong".as_ptr()) == 0 {
        return SS_STRONG;
    }
    if Q_stricmp(name, c"desann".as_ptr()) == 0 {
        return SS_DESANN;
    }
    if Q_stricmp(name, c"tavion".as_ptr()) == 0 {
        return SS_TAVION;
    }
    if Q_stricmp(name, c"dual".as_ptr()) == 0 {
        return SS_DUAL;
    }
    if Q_stricmp(name, c"staff".as_ptr()) == 0 {
        return SS_STAFF;
    }
    SS_NONE
}

/// `qboolean WP_SaberBladeUseSecondBladeStyle( saberInfo_t *saber, int bladeNum )`
/// (bg_saberLoad.c:215) — true if `bladeNum` falls at/after `bladeStyle2Start` (and a
/// secondary blade style is in use), i.e. this blade uses the `*2` secondary style.
///
/// # Safety
/// `saber` must be null or point at a valid `saberInfo_t`.
pub unsafe fn WP_SaberBladeUseSecondBladeStyle(
    saber: *mut saberInfo_t,
    bladeNum: c_int,
) -> qboolean {
    if !saber.is_null() {
        if (*saber).bladeStyle2Start > 0 {
            if bladeNum >= (*saber).bladeStyle2Start {
                return QTRUE;
            }
        }
    }
    QFALSE
}

/// `qboolean WP_SaberBladeDoTransitionDamage( saberInfo_t *saber, int bladeNum )`
/// (bg_saberLoad.c:230) — true if `bladeNum` should do damage during start/transition/
/// return anims, picking the primary (`SFL2_TRANSITION_DAMAGE`) or secondary
/// (`SFL2_TRANSITION_DAMAGE2`) blade-style flag per [`WP_SaberBladeUseSecondBladeStyle`].
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t`.
pub unsafe fn WP_SaberBladeDoTransitionDamage(
    saber: *mut saberInfo_t,
    bladeNum: c_int,
) -> qboolean {
    if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) == QFALSE
        && ((*saber).saberFlags2 & SFL2_TRANSITION_DAMAGE) != 0
    {
        // use first blade style for this blade
        return QTRUE;
    } else if WP_SaberBladeUseSecondBladeStyle(saber, bladeNum) != QFALSE
        && ((*saber).saberFlags2 & SFL2_TRANSITION_DAMAGE2) != 0
    {
        // use second blade style for this blade
        return QTRUE;
    }
    QFALSE
}

/// `qboolean WP_UseFirstValidSaberStyle( saberInfo_t *saber1, saberInfo_t *saber2, int
/// saberHolstered, int *saberAnimLevel )` (bg_saberLoad.c:245) — if the current
/// `*saberAnimLevel` is forbidden by the active saber(s), pick the first allowed style
/// (from `SS_FAST` up) and write it back through `saberAnimLevel`, returning `qtrue`;
/// returns `qfalse` if the current style is valid or no valid style exists.
///
/// # Safety
/// `saber1`/`saber2` must be null or point at a valid `saberInfo_t`; `saberAnimLevel`
/// must point at a valid, writable `int`.
pub unsafe fn WP_UseFirstValidSaberStyle(
    saber1: *mut saberInfo_t,
    saber2: *mut saberInfo_t,
    saberHolstered: c_int,
    saberAnimLevel: *mut c_int,
) -> qboolean {
    let mut styleInvalid: qboolean = QFALSE;
    let saber1Active: qboolean;
    let saber2Active: qboolean;
    let mut dualSabers: qboolean = QFALSE;
    let mut validStyles: c_int = 0;

    if !saber2.is_null() && (*saber2).model[0] != 0 {
        dualSabers = QTRUE;
    }

    if dualSabers != QFALSE {
        // dual
        if saberHolstered > 1 {
            saber1Active = QFALSE;
            saber2Active = QFALSE;
        } else if saberHolstered > 0 {
            saber1Active = QTRUE;
            saber2Active = QFALSE;
        } else {
            saber1Active = QTRUE;
            saber2Active = QTRUE;
        }
    } else {
        saber2Active = QFALSE;
        if saber1.is_null() || (*saber1).model[0] == 0 {
            saber1Active = QFALSE;
        } else if (*saber1).numBlades > 1 {
            // staff
            if saberHolstered > 1 {
                saber1Active = QFALSE;
            } else {
                saber1Active = QTRUE;
            }
        } else {
            // single
            if saberHolstered != 0 {
                saber1Active = QFALSE;
            } else {
                saber1Active = QTRUE;
            }
        }
    }

    // initially, all styles are valid
    for styleNum in (SS_NONE + 1)..SS_NUM_SABER_STYLES {
        validStyles |= 1 << styleNum;
    }

    if saber1Active != QFALSE
        && !saber1.is_null()
        && (*saber1).model[0] != 0
        && (*saber1).stylesForbidden != 0
    {
        if ((*saber1).stylesForbidden & (1 << *saberAnimLevel)) != 0 {
            // not a valid style for first saber!
            styleInvalid = QTRUE;
            validStyles &= !(*saber1).stylesForbidden;
        }
    }
    if dualSabers != QFALSE {
        // check second saber, too
        if saber2Active != QFALSE && (*saber2).stylesForbidden != 0 {
            if ((*saber2).stylesForbidden & (1 << *saberAnimLevel)) != 0 {
                // not a valid style for second saber!
                styleInvalid = QTRUE;
                // only the ones both sabers allow is valid
                validStyles &= !(*saber2).stylesForbidden;
            }
        }
    }
    if styleInvalid != QFALSE && validStyles != 0 {
        // using an invalid style and have at least one valid style to use, so switch to it
        for styleNum in SS_FAST..SS_NUM_SABER_STYLES {
            if (validStyles & (1 << styleNum)) != 0 {
                *saberAnimLevel = styleNum;
                return QTRUE;
            }
        }
    }
    QFALSE
}

/// `qboolean WP_SaberStyleValidForSaber( saberInfo_t *saber1, saberInfo_t *saber2, int
/// saberHolstered, int saberAnimLevel )` (bg_saberLoad.c:353) — true if `saberAnimLevel`
/// is allowed by the currently-active saber(s): not in either saber's `stylesForbidden`,
/// and (when dual-wielding) only `SS_DUAL` or a `SS_TAVION` granted by one of the sabers.
///
/// # Safety
/// `saber1`/`saber2` must be null or point at a valid `saberInfo_t`.
pub unsafe fn WP_SaberStyleValidForSaber(
    saber1: *mut saberInfo_t,
    saber2: *mut saberInfo_t,
    saberHolstered: c_int,
    saberAnimLevel: c_int,
) -> qboolean {
    let saber1Active: qboolean;
    let saber2Active: qboolean;
    let mut dualSabers: qboolean = QFALSE;

    if !saber2.is_null() && (*saber2).model[0] != 0 {
        dualSabers = QTRUE;
    }

    if dualSabers != QFALSE {
        // dual
        if saberHolstered > 1 {
            saber1Active = QFALSE;
            saber2Active = QFALSE;
        } else if saberHolstered > 0 {
            saber1Active = QTRUE;
            saber2Active = QFALSE;
        } else {
            saber1Active = QTRUE;
            saber2Active = QTRUE;
        }
    } else {
        saber2Active = QFALSE;
        if saber1.is_null() || (*saber1).model[0] == 0 {
            saber1Active = QFALSE;
        } else if (*saber1).numBlades > 1 {
            // staff
            if saberHolstered > 1 {
                saber1Active = QFALSE;
            } else {
                saber1Active = QTRUE;
            }
        } else {
            // single
            if saberHolstered != 0 {
                saber1Active = QFALSE;
            } else {
                saber1Active = QTRUE;
            }
        }
    }

    if saber1Active != QFALSE
        && !saber1.is_null()
        && (*saber1).model[0] != 0
        && (*saber1).stylesForbidden != 0
    {
        if ((*saber1).stylesForbidden & (1 << saberAnimLevel)) != 0 {
            // not a valid style for first saber!
            return QFALSE;
        }
    }
    if dualSabers != QFALSE
        && saber2Active != QFALSE
        && !saber2.is_null()
        && (*saber2).model[0] != 0
    {
        if (*saber2).stylesForbidden != 0 {
            // check second saber, too
            if ((*saber2).stylesForbidden & (1 << saberAnimLevel)) != 0 {
                // not a valid style for second saber!
                return QFALSE;
            }
        }
        // now: if using dual sabers, only dual and tavion (if given with this saber) are allowed
        if saberAnimLevel != SS_DUAL {
            // dual is okay
            if saberAnimLevel != SS_TAVION {
                // tavion might be okay, all others are not
                return QFALSE;
            } else {
                // see if "tavion" style is okay
                if saber1Active != QFALSE
                    && !saber1.is_null()
                    && (*saber1).model[0] != 0
                    && ((*saber1).stylesLearned & (1 << SS_TAVION)) != 0
                {
                    // okay to use tavion style, first saber gave it to us
                } else if ((*saber2).stylesLearned & (1 << SS_TAVION)) != 0 {
                    // okay to use tavion style, second saber gave it to us
                } else {
                    // tavion style is not allowed because neither of the sabers we're using gave it to us (I know, doesn't quite make sense, but...)
                    return QFALSE;
                }
            }
        }
    }
    QTRUE
}

/// `qboolean WP_SaberCanTurnOffSomeBlades( saberInfo_t *saber )` (bg_saberLoad.c:466) —
/// false if every blade is forced always-on (the `SFL2_NO_MANUAL_DEACTIVATE`/`*2` flags
/// cover all blades), true if at least one blade can be manually toggled off.
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t`.
pub unsafe fn WP_SaberCanTurnOffSomeBlades(saber: *mut saberInfo_t) -> qboolean {
    if (*saber).bladeStyle2Start > 0 && (*saber).numBlades > (*saber).bladeStyle2Start {
        if ((*saber).saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE) != 0
            && ((*saber).saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE2) != 0
        {
            // all blades are always on
            return QFALSE;
        }
    } else {
        if ((*saber).saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE) != 0 {
            // all blades are always on
            return QFALSE;
        }
    }
    // you can turn some off
    QTRUE
}

/// `void WP_SaberSetDefaults( saberInfo_t *saber )` (bg_saberLoad.c:157) — initialize
/// a saber to the built-in defaults, so that even if a parse fails there is at least
/// something valid there. The three sound indices route through [`BG_SoundIndex`]
/// (the engine), so — like the rest of the trap-facing surface — this function has no
/// C oracle; every other field is a literal transcription of the C source.
///
/// # Safety
/// `saber` must point at a valid, writable `saberInfo_t`.
pub unsafe fn WP_SaberSetDefaults(saber: *mut saberInfo_t) {
    // Set defaults so that, if it fails, there's at least something there
    for i in 0..MAX_BLADES {
        (*saber).blade[i].color = SABER_RED;
        (*saber).blade[i].radius = SABER_RADIUS_STANDARD;
        (*saber).blade[i].lengthMax = 32.0;
    }

    strcpy((*saber).name.as_mut_ptr(), c"default".as_ptr());
    strcpy((*saber).fullName.as_mut_ptr(), c"lightsaber".as_ptr());
    strcpy(
        (*saber).model.as_mut_ptr(),
        c"models/weapons2/saber_reborn/saber_w.glm".as_ptr(),
    );
    (*saber).skin = 0;
    (*saber).soundOn = BG_SoundIndex("sound/weapons/saber/enemy_saber_on.wav");
    (*saber).soundLoop = BG_SoundIndex("sound/weapons/saber/saberhum3.wav");
    (*saber).soundOff = BG_SoundIndex("sound/weapons/saber/enemy_saber_off.wav");
    (*saber).numBlades = 1;
    (*saber).r#type = SABER_SINGLE;
    (*saber).stylesLearned = 0;
    (*saber).stylesForbidden = 0; // allow all styles
    (*saber).maxChain = 0; // 0 = use default behavior
    (*saber).forceRestrictions = 0;
    (*saber).lockBonus = 0;
    (*saber).parryBonus = 0;
    (*saber).breakParryBonus = 0;
    (*saber).breakParryBonus2 = 0;
    (*saber).disarmBonus = 0;
    (*saber).disarmBonus2 = 0;
    (*saber).singleBladeStyle = SS_NONE; // makes it so that you use a different style if you only have the first blade active
                                         // saber->brokenSaber1 = NULL; // if saber is actually hit by another saber, it can be cut in half/broken and will be replaced with this saber in your right hand
                                         // saber->brokenSaber2 = NULL; // ...left hand
                                         //===NEW========================================================================================
                                         //done in cgame (client-side code)
    (*saber).saberFlags = 0; // see all the SFL_ flags
    (*saber).saberFlags2 = 0; // see all the SFL2_ flags

    (*saber).spinSound = 0; // none - if set, plays this sound as it spins when thrown
    (*saber).swingSound[0] = 0; // none - if set, plays one of these 3 sounds when swung during an attack - NOTE: must provide all 3!!!
    (*saber).swingSound[1] = 0;
    (*saber).swingSound[2] = 0;

    //done in game (server-side code)
    (*saber).moveSpeedScale = 1.0; // 1.0 - you move faster/slower when using this saber
    (*saber).animSpeedScale = 1.0; // 1.0 - plays normal attack animations faster/slower

    (*saber).kataMove = LS_INVALID; // if set, player will execute this move when they press both attack buttons at the same time
    (*saber).lungeAtkMove = LS_INVALID; // crouch+fwd+attack
    (*saber).jumpAtkUpMove = LS_INVALID; // jump+attack
    (*saber).jumpAtkFwdMove = LS_INVALID; // jump+fwd+attack
    (*saber).jumpAtkBackMove = LS_INVALID; // jump+back+attack
    (*saber).jumpAtkRightMove = LS_INVALID; // jump+right+attack
    (*saber).jumpAtkLeftMove = LS_INVALID; // jump+left+attack
    (*saber).readyAnim = -1; // anim to use when standing idle
    (*saber).drawAnim = -1; // anim to use when drawing weapon
    (*saber).putawayAnim = -1; // anim to use when putting weapon away
    (*saber).tauntAnim = -1; // anim to use when hit "taunt"
    (*saber).bowAnim = -1; // anim to use when hit "bow"
    (*saber).meditateAnim = -1; // anim to use when hit "meditate"
    (*saber).flourishAnim = -1; // anim to use when hit "flourish"
    (*saber).gloatAnim = -1; // anim to use when hit "gloat"

    //***NOTE: max of 2 "styles" of blades; bladeStyle2Start is the first blade to use the secondary values***
    (*saber).bladeStyle2Start = 0; // 0 - if set, blades from this number and higher use the secondary values below

    //===PRIMARY BLADES=====================
    //done in cgame (client-side code)
    (*saber).trailStyle = 0; // 0 - default (0) is normal, 1 is a motion blur and 2 is no trail at all
    (*saber).g2MarksShader = 0; // none - if set, the game will use this shader for marks on enemies
    (*saber).g2WeaponMarkShader = 0; // none - if set, projects this shader onto the weapon when it damages a person
                                     //saber->bladeShader = 0; // none - if set, overrides the shader used for the saber blade?
                                     //saber->trailShader = 0; // none - if set, overrides the shader used for the saber trail?
    (*saber).hitSound[0] = 0; // none - if set, plays one of these 3 sounds when saber hits a person - NOTE: must provide all 3!!!
    (*saber).hitSound[1] = 0;
    (*saber).hitSound[2] = 0;
    (*saber).blockSound[0] = 0; // none - if set, plays one of these 3 sounds when saber/sword hits another saber/sword - NOTE: must provide all 3!!!
    (*saber).blockSound[1] = 0;
    (*saber).blockSound[2] = 0;
    (*saber).bounceSound[0] = 0; // none - if set, plays one of these 3 sounds when saber/sword hits a wall and bounces off - NOTE: must provide all 3!!!
    (*saber).bounceSound[1] = 0;
    (*saber).bounceSound[2] = 0;
    (*saber).blockEffect = 0; // none - if set, plays this effect when the saber/sword hits another saber/sword (instead of "saber/saber_block.efx")
    (*saber).hitPersonEffect = 0; // none - if set, plays this effect when the saber/sword hits a person (instead of "saber/blood_sparks_mp.efx")
    (*saber).hitOtherEffect = 0; // none - if set, plays this effect when the saber/sword hits something else damagable (instead of "saber/saber_cut.efx")
    (*saber).bladeEffect = 0; // none - if set, plays this effect at the blade tag

    //done in game (server-side code)
    (*saber).knockbackScale = 0.0; // 0 - if non-zero, uses damage done to calculate an appropriate amount of knockback
    (*saber).damageScale = 1.0; // 1 - scale up or down the damage done by the saber
    (*saber).splashRadius = 0.0; // 0 - radius of splashDamage
    (*saber).splashDamage = 0; // 0 - amount of splashDamage
    (*saber).splashKnockback = 0.0; // 0 - amount of splashKnockback

    //===SECONDARY BLADES===================
    //done in cgame (client-side code)
    (*saber).trailStyle2 = 0; // 0 - default (0) is normal, 1 is a motion blur and 2 is no trail at all
    (*saber).g2MarksShader2 = 0; // none - if set, the game will use this shader for marks on enemies
    (*saber).g2WeaponMarkShader2 = 0; // none - if set, projects this shader onto the weapon when it damages a person
    (*saber).hit2Sound[0] = 0;
    (*saber).hit2Sound[1] = 0;
    (*saber).hit2Sound[2] = 0;
    (*saber).block2Sound[0] = 0;
    (*saber).block2Sound[1] = 0;
    (*saber).block2Sound[2] = 0;
    (*saber).bounce2Sound[0] = 0;
    (*saber).bounce2Sound[1] = 0;
    (*saber).bounce2Sound[2] = 0;
    (*saber).blockEffect2 = 0;
    (*saber).hitPersonEffect2 = 0;
    (*saber).hitOtherEffect2 = 0;
    (*saber).bladeEffect2 = 0;

    //done in game (server-side code)
    (*saber).knockbackScale2 = 0.0;
    (*saber).damageScale2 = 1.0;
    (*saber).splashRadius2 = 0.0;
    (*saber).splashDamage2 = 0;
    (*saber).splashKnockback2 = 0.0;
}

/// `qboolean WP_SaberParseParms( const char *SaberName, saberInfo_t *saber )`
/// (bg_saberLoad.c:232) — find the named saber's `{ ... }` block in [`SaberParms`]
/// and fill `saber` from its keyword/value lines. Resets to [`WP_SaberSetDefaults`]
/// first, falls back to [`DEFAULT_SABER`] when the name is empty or unfound, and
/// dispatches every recognized `.sab` parm onto a `saberInfo_t` field.
///
/// **Build config = retail `QAGAME`:** the `g2MarksShader`/`blockEffect`/
/// `hitPersonEffect`/`hitOtherEffect` parms are `#ifdef QAGAME//cgame-only` — their
/// values are parsed and discarded (the render/effect registration is cgame-only),
/// so `trap_R_RegisterShader`/`trap_FX_RegisterEffect` are never called here. The
/// `brokenSaber1`/`brokenSaber2` `G_NewString` writes are commented out in C. The
/// `#ifndef FINAL_BUILD` bad-parm warnings are present (this is a non-final build);
/// the `#ifdef _DEBUG` unknown-keyword warning is not. Engine-facing
/// (`BG_SoundIndex` → configstrings, `trap_R_RegisterSkin`, the parser cursor), so
/// no C oracle.
///
/// # Safety
/// `saber` must be null or point at a valid [`saberInfo_t`]; `SaberName` must be
/// null or a valid NUL-terminated C string.
pub unsafe fn WP_SaberParseParms(SaberName: *const c_char, saber: *mut saberInfo_t) -> qboolean {
    let mut token: *const c_char;
    let mut value: *const c_char = core::ptr::null();
    let mut p: *const c_char;
    let mut useSaber = [0 as c_char; 1024];
    // C leaves `f`/`n` uninitialized (always written by COM_Parse* / the strlen
    // branch before read); 0-init keeps Rust's flow analysis happy with no behavior change.
    let mut f: f32 = 0.0;
    let mut n: c_int = 0;
    let mut triedDefault: qboolean = QFALSE;
    // C: `int saberMove = LS_INVALID; int anim = -1;` — but each is always written by
    // `GetIDForString` immediately before it is read, so the C initializer is dead in
    // Rust's definite-assignment view (left uninitialized to avoid an unused_assignments
    // warning; behavior is identical).
    let mut saberMove: c_int;
    let mut anim: c_int;

    if saber.is_null() {
        return QFALSE;
    }

    // Set defaults so that, if it fails, there's at least something there
    WP_SaberSetDefaults(saber);

    if SaberName.is_null() || *SaberName == 0 {
        strcpy(useSaber.as_mut_ptr(), DEFAULT_SABER.as_ptr()); //default
        triedDefault = QTRUE;
    } else {
        strcpy(useSaber.as_mut_ptr(), SaberName);
    }

    // try to parse it out
    p = addr_of!(SaberParms) as *const c_char;
    COM_BeginParseSession(c"saberinfo".as_ptr());

    // look for the right saber
    while !p.is_null() {
        token = COM_ParseExt(addr_of_mut!(p), QTRUE);
        if *token == 0 {
            if triedDefault == QFALSE {
                // fall back to default and restart, should always be there
                p = addr_of!(SaberParms) as *const c_char;
                COM_BeginParseSession(c"saberinfo".as_ptr());
                strcpy(useSaber.as_mut_ptr(), DEFAULT_SABER.as_ptr());
                triedDefault = QTRUE;
            } else {
                return QFALSE;
            }
        }

        if Q_stricmp(token, useSaber.as_ptr()) == 0 {
            break;
        }

        SkipBracedSection(addr_of_mut!(p));
    }
    if p.is_null() {
        // even the default saber isn't found?
        return QFALSE;
    }

    // got the name we're using for sure
    strcpy((*saber).name.as_mut_ptr(), useSaber.as_ptr());

    if BG_ParseLiteral(addr_of_mut!(p), c"{".as_ptr()) != QFALSE {
        return QFALSE;
    }

    // parse the saber info block
    loop {
        token = COM_ParseExt(addr_of_mut!(p), QTRUE);
        if *token == 0 {
            Com_Printf(&format!(
                "^1ERROR: unexpected EOF while parsing '{}'\n",
                Sz(useSaber.as_ptr())
            ));
            return QFALSE;
        }

        if Q_stricmp(token, c"}".as_ptr()) == 0 {
            break;
        }

        // saber fullName
        if Q_stricmp(token, c"name".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            strcpy((*saber).fullName.as_mut_ptr(), value);
            continue;
        }

        // saber type
        if Q_stricmp(token, c"saberType".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            let saberType = GetIDForString(addr_of!(SaberTable) as *const stringID_table_t, value);
            if saberType >= SABER_SINGLE && saberType <= NUM_SABERS {
                (*saber).r#type = saberType as saberType_t;
            }
            continue;
        }

        // saber hilt
        if Q_stricmp(token, c"saberModel".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            strcpy((*saber).model.as_mut_ptr(), value);
            continue;
        }

        if Q_stricmp(token, c"customSkin".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).skin = trap::R_RegisterSkin(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // on sound
        if Q_stricmp(token, c"soundOn".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).soundOn = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // loop sound
        if Q_stricmp(token, c"soundLoop".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).soundLoop = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // off sound
        if Q_stricmp(token, c"soundOff".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).soundOff = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        if Q_stricmp(token, c"numBlades".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n < 1 || n > MAX_BLADES as c_int {
                Com_Error(
                    ERR_DROP,
                    &format!(
                        "WP_SaberParseParms: saber {} has illegal number of blades ({}) max: {}",
                        Sz(useSaber.as_ptr()),
                        n,
                        MAX_BLADES
                    ),
                );
                // Com_Error is `-> !`; the C `continue;` after it is unreachable.
            }
            (*saber).numBlades = n;
            continue;
        }

        // saberColor
        if Q_stricmpn(token, c"saberColor".as_ptr(), 10) == 0 {
            if strlen(token) == 10 {
                n = -1;
            } else if strlen(token) == 11 {
                n = atoi(token.add(10)) - 1;
                if n > 7 || n < 1 {
                    // #ifndef FINAL_BUILD
                    Com_Printf(&format!(
                        "^3WARNING: bad saberColor '{}' in {}\n",
                        Sz(token),
                        Sz(useSaber.as_ptr())
                    ));
                    continue;
                }
            } else {
                // #ifndef FINAL_BUILD
                Com_Printf(&format!(
                    "^3WARNING: bad saberColor '{}' in {}\n",
                    Sz(token),
                    Sz(useSaber.as_ptr())
                ));
                continue;
            }

            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE
            //read the color
            {
                continue;
            }

            if n == -1 {
                // NOTE: this fills in the rest of the blades with the same color by default
                let color = TranslateSaberColor(value);
                n = 0;
                while n < MAX_BLADES as c_int {
                    (*saber).blade[n as usize].color = color;
                    n += 1;
                }
            } else {
                (*saber).blade[n as usize].color = TranslateSaberColor(value);
            }
            continue;
        }

        // saber length
        if Q_stricmpn(token, c"saberLength".as_ptr(), 11) == 0 {
            if strlen(token) == 11 {
                n = -1;
            } else if strlen(token) == 12 {
                n = atoi(token.add(11)) - 1;
                if n > 7 || n < 1 {
                    // #ifndef FINAL_BUILD
                    Com_Printf(&format!(
                        "^3WARNING: bad saberLength '{}' in {}\n",
                        Sz(token),
                        Sz(useSaber.as_ptr())
                    ));
                    continue;
                }
            } else {
                // #ifndef FINAL_BUILD
                Com_Printf(&format!(
                    "^3WARNING: bad saberLength '{}' in {}\n",
                    Sz(token),
                    Sz(useSaber.as_ptr())
                ));
                continue;
            }

            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            // cap
            if f < 4.0 {
                f = 4.0;
            }

            if n == -1 {
                // NOTE: this fills in the rest of the blades with the same length by default
                n = 0;
                while n < MAX_BLADES as c_int {
                    (*saber).blade[n as usize].lengthMax = f;
                    n += 1;
                }
            } else {
                (*saber).blade[n as usize].lengthMax = f;
            }
            continue;
        }

        // blade radius
        if Q_stricmpn(token, c"saberRadius".as_ptr(), 11) == 0 {
            if strlen(token) == 11 {
                n = -1;
            } else if strlen(token) == 12 {
                n = atoi(token.add(11)) - 1;
                if n > 7 || n < 1 {
                    // #ifndef FINAL_BUILD
                    Com_Printf(&format!(
                        "^3WARNING: bad saberRadius '{}' in {}\n",
                        Sz(token),
                        Sz(useSaber.as_ptr())
                    ));
                    continue;
                }
            } else {
                // #ifndef FINAL_BUILD
                Com_Printf(&format!(
                    "^3WARNING: bad saberRadius '{}' in {}\n",
                    Sz(token),
                    Sz(useSaber.as_ptr())
                ));
                continue;
            }

            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            // cap
            if f < 0.25 {
                f = 0.25;
            }
            if n == -1 {
                // NOTE: this fills in the rest of the blades with the same length by default
                n = 0;
                while n < MAX_BLADES as c_int {
                    (*saber).blade[n as usize].radius = f;
                    n += 1;
                }
            } else {
                (*saber).blade[n as usize].radius = f;
            }
            continue;
        }

        // locked saber style
        if Q_stricmp(token, c"saberStyle".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // OLD WAY: only allowed ONE style
            let style = TranslateSaberStyle(value);
            // learn only this style
            (*saber).stylesLearned = 1 << style;
            // forbid all other styles
            (*saber).stylesForbidden = 0;
            let mut styleNum = SS_NONE + 1;
            while styleNum < SS_NUM_SABER_STYLES {
                if styleNum != style {
                    (*saber).stylesForbidden |= 1 << styleNum;
                }
                styleNum += 1;
            }
            continue;
        }

        // learned saber style
        if Q_stricmp(token, c"saberStyleLearned".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).stylesLearned |= 1 << TranslateSaberStyle(value);
            continue;
        }

        // forbidden saber style
        if Q_stricmp(token, c"saberStyleForbidden".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).stylesForbidden |= 1 << TranslateSaberStyle(value);
            continue;
        }

        // maxChain
        if Q_stricmp(token, c"maxChain".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).maxChain = n;
            continue;
        }

        // lockable
        if Q_stricmp(token, c"lockable".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n == 0 {
                (*saber).saberFlags |= SFL_NOT_LOCKABLE;
            }
            continue;
        }

        // throwable
        if Q_stricmp(token, c"throwable".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n == 0 {
                (*saber).saberFlags |= SFL_NOT_THROWABLE;
            }
            continue;
        }

        // disarmable
        if Q_stricmp(token, c"disarmable".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n == 0 {
                (*saber).saberFlags |= SFL_NOT_DISARMABLE;
            }
            continue;
        }

        // active blocking
        if Q_stricmp(token, c"blocking".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n == 0 {
                (*saber).saberFlags |= SFL_NOT_ACTIVE_BLOCKING;
            }
            continue;
        }

        // twoHanded
        if Q_stricmp(token, c"twoHanded".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_TWO_HANDED;
            }
            continue;
        }

        // force power restrictions
        if Q_stricmp(token, c"forceRestrict".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            let fp = GetIDForString(addr_of!(FPTable) as *const stringID_table_t, value);
            if fp >= FP_FIRST && fp < NUM_FORCE_POWERS as c_int {
                (*saber).forceRestrictions |= 1 << fp;
            }
            continue;
        }

        // lockBonus
        if Q_stricmp(token, c"lockBonus".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).lockBonus = n;
            continue;
        }

        // parryBonus
        if Q_stricmp(token, c"parryBonus".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).parryBonus = n;
            continue;
        }

        // breakParryBonus
        if Q_stricmp(token, c"breakParryBonus".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).breakParryBonus = n;
            continue;
        }

        // breakParryBonus2
        if Q_stricmp(token, c"breakParryBonus2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).breakParryBonus2 = n;
            continue;
        }

        // disarmBonus
        if Q_stricmp(token, c"disarmBonus".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).disarmBonus = n;
            continue;
        }

        // disarmBonus2
        if Q_stricmp(token, c"disarmBonus2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).disarmBonus2 = n;
            continue;
        }

        // single blade saber style
        if Q_stricmp(token, c"singleBladeStyle".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).singleBladeStyle = TranslateSaberStyle(value);
            continue;
        }

        // single blade throwable
        if Q_stricmp(token, c"singleBladeThrowable".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_SINGLE_BLADE_THROWABLE;
            }
            continue;
        }

        // broken replacement saber1 (right hand)
        if Q_stricmp(token, c"brokenSaber1".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // saber->brokenSaber1 = G_NewString( value ); // commented out in C
            continue;
        }

        // broken replacement saber2 (left hand)
        if Q_stricmp(token, c"brokenSaber2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // saber->brokenSaber2 = G_NewString( value ); // commented out in C
            continue;
        }

        // spins and does damage on return from saberthrow
        if Q_stricmp(token, c"returnDamage".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_RETURN_DAMAGE;
            }
            continue;
        }

        // spin sound (when thrown)
        if Q_stricmp(token, c"spinSound".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).spinSound = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // swing sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"swingSound1".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).swingSound[0] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // swing sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"swingSound2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).swingSound[1] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // swing sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"swingSound3".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).swingSound[2] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // you move faster/slower when using this saber
        if Q_stricmp(token, c"moveSpeedScale".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).moveSpeedScale = f;
            continue;
        }

        // plays normal attack animations faster/slower
        if Q_stricmp(token, c"animSpeedScale".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).animSpeedScale = f;
            continue;
        }

        // if non-zero, the saber will bounce back when it hits solid architecture (good for real-sword type mods)
        if Q_stricmp(token, c"bounceOnWalls".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_BOUNCE_ON_WALLS;
            }
            continue;
        }

        // if set, saber model is bolted to wrist, not in hand... useful for things like claws & shields, etc.
        if Q_stricmp(token, c"boltToWrist".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_BOLT_TO_WRIST;
            }
            continue;
        }

        // kata move
        if Q_stricmp(token, c"kataMove".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            saberMove = GetIDForString(addr_of!(SaberMoveTable) as *const stringID_table_t, value);
            if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                (*saber).kataMove = saberMove; //LS_INVALID - if set, player will execute this move when they press both attack buttons at the same time
            }
            continue;
        }
        // lungeAtkMove move
        if Q_stricmp(token, c"lungeAtkMove".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            saberMove = GetIDForString(addr_of!(SaberMoveTable) as *const stringID_table_t, value);
            if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                (*saber).lungeAtkMove = saberMove;
            }
            continue;
        }
        // jumpAtkUpMove move
        if Q_stricmp(token, c"jumpAtkUpMove".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            saberMove = GetIDForString(addr_of!(SaberMoveTable) as *const stringID_table_t, value);
            if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                (*saber).jumpAtkUpMove = saberMove;
            }
            continue;
        }
        // jumpAtkFwdMove move
        if Q_stricmp(token, c"jumpAtkFwdMove".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            saberMove = GetIDForString(addr_of!(SaberMoveTable) as *const stringID_table_t, value);
            if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                (*saber).jumpAtkFwdMove = saberMove;
            }
            continue;
        }
        // jumpAtkBackMove move
        if Q_stricmp(token, c"jumpAtkBackMove".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            saberMove = GetIDForString(addr_of!(SaberMoveTable) as *const stringID_table_t, value);
            if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                (*saber).jumpAtkBackMove = saberMove;
            }
            continue;
        }
        // jumpAtkRightMove move
        if Q_stricmp(token, c"jumpAtkRightMove".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            saberMove = GetIDForString(addr_of!(SaberMoveTable) as *const stringID_table_t, value);
            if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                (*saber).jumpAtkRightMove = saberMove;
            }
            continue;
        }
        // jumpAtkLeftMove move
        if Q_stricmp(token, c"jumpAtkLeftMove".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            saberMove = GetIDForString(addr_of!(SaberMoveTable) as *const stringID_table_t, value);
            if saberMove >= LS_INVALID && saberMove < LS_MOVE_MAX {
                (*saber).jumpAtkLeftMove = saberMove;
            }
            continue;
        }
        // readyAnim
        if Q_stricmp(token, c"readyAnim".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            anim = GetIDForString(addr_of!(animTable) as *const stringID_table_t, value);
            if anim >= 0 && anim < MAX_ANIMATIONS as c_int {
                (*saber).readyAnim = anim;
            }
            continue;
        }
        // drawAnim
        if Q_stricmp(token, c"drawAnim".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            anim = GetIDForString(addr_of!(animTable) as *const stringID_table_t, value);
            if anim >= 0 && anim < MAX_ANIMATIONS as c_int {
                (*saber).drawAnim = anim;
            }
            continue;
        }
        // putawayAnim
        if Q_stricmp(token, c"putawayAnim".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            anim = GetIDForString(addr_of!(animTable) as *const stringID_table_t, value);
            if anim >= 0 && anim < MAX_ANIMATIONS as c_int {
                (*saber).putawayAnim = anim;
            }
            continue;
        }
        // tauntAnim
        if Q_stricmp(token, c"tauntAnim".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            anim = GetIDForString(addr_of!(animTable) as *const stringID_table_t, value);
            if anim >= 0 && anim < MAX_ANIMATIONS as c_int {
                (*saber).tauntAnim = anim;
            }
            continue;
        }
        // bowAnim
        if Q_stricmp(token, c"bowAnim".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            anim = GetIDForString(addr_of!(animTable) as *const stringID_table_t, value);
            if anim >= 0 && anim < MAX_ANIMATIONS as c_int {
                (*saber).bowAnim = anim;
            }
            continue;
        }
        // meditateAnim
        if Q_stricmp(token, c"meditateAnim".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            anim = GetIDForString(addr_of!(animTable) as *const stringID_table_t, value);
            if anim >= 0 && anim < MAX_ANIMATIONS as c_int {
                (*saber).meditateAnim = anim;
            }
            continue;
        }
        // flourishAnim
        if Q_stricmp(token, c"flourishAnim".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            anim = GetIDForString(addr_of!(animTable) as *const stringID_table_t, value);
            if anim >= 0 && anim < MAX_ANIMATIONS as c_int {
                (*saber).flourishAnim = anim;
            }
            continue;
        }
        // gloatAnim
        if Q_stricmp(token, c"gloatAnim".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            anim = GetIDForString(addr_of!(animTable) as *const stringID_table_t, value);
            if anim >= 0 && anim < MAX_ANIMATIONS as c_int {
                (*saber).gloatAnim = anim;
            }
            continue;
        }

        // if set, cannot do roll-stab move at end of roll
        if Q_stricmp(token, c"noRollStab".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_ROLL_STAB;
            }
            continue;
        }

        // if set, cannot do pull+attack move (move not available in MP anyway)
        if Q_stricmp(token, c"noPullAttack".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_PULL_ATTACK;
            }
            continue;
        }

        // if set, cannot do back-stab moves
        if Q_stricmp(token, c"noBackAttack".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_BACK_ATTACK;
            }
            continue;
        }

        // if set, cannot do stabdown move (when enemy is on ground)
        if Q_stricmp(token, c"noStabDown".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_STABDOWN;
            }
            continue;
        }

        // if set, cannot side-run or forward-run on walls
        if Q_stricmp(token, c"noWallRuns".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_WALL_RUNS;
            }
            continue;
        }

        // if set, cannot do backflip off wall or side-flips off walls
        if Q_stricmp(token, c"noWallFlips".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_WALL_FLIPS;
            }
            continue;
        }

        // if set, cannot grab wall & jump off
        if Q_stricmp(token, c"noWallGrab".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_WALL_GRAB;
            }
            continue;
        }

        // if set, cannot roll
        if Q_stricmp(token, c"noRolls".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_ROLLS;
            }
            continue;
        }

        // if set, cannot do flips
        if Q_stricmp(token, c"noFlips".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_FLIPS;
            }
            continue;
        }

        // if set, cannot do cartwheels
        if Q_stricmp(token, c"noCartwheels".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_CARTWHEELS;
            }
            continue;
        }

        // if set, cannot do kicks (can't do kicks anyway if using a throwable saber/sword)
        if Q_stricmp(token, c"noKicks".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_KICKS;
            }
            continue;
        }

        // if set, cannot do the simultaneous attack left/right moves (only available in Dual Lightsaber Combat Style)
        if Q_stricmp(token, c"noMirrorAttacks".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags |= SFL_NO_MIRROR_ATTACKS;
            }
            continue;
        }

        // stays on in water
        if Q_stricmp(token, c"onInWater".as_ptr()) == 0 {
            // ignore in MP
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        if Q_stricmp(token, c"notInMP".as_ptr()) == 0 {
            // ignore this
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        //===ABOVE THIS, ALL VALUES ARE GLOBAL TO THE SABER========================================================
        // bladeStyle2Start - where to start using the second set of blade data
        if Q_stricmp(token, c"bladeStyle2Start".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).bladeStyle2Start = n;
            continue;
        }
        //===BLADE-SPECIFIC FIELDS=================================================================================

        //===PRIMARY BLADE====================================
        // stops the saber from drawing marks on the world (good for real-sword type mods)
        if Q_stricmp(token, c"noWallMarks".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_WALL_MARKS;
            }
            continue;
        }

        // stops the saber from drawing a dynamic light (good for real-sword type mods)
        if Q_stricmp(token, c"noDlight".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_DLIGHT;
            }
            continue;
        }

        // stops the saber from drawing a blade (good for real-sword type mods)
        if Q_stricmp(token, c"noBlade".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_BLADE;
            }
            continue;
        }

        // default (0) is normal, 1 is a motion blur and 2 is no trail at all (good for real-sword type mods)
        if Q_stricmp(token, c"trailStyle".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).trailStyle = n;
            continue;
        }

        // if set, the game will use this shader for marks on enemies instead of the default "gfx/damage/saberglowmark"
        if Q_stricmp(token, c"g2MarksShader".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->g2MarksShader = trap_R_RegisterShader( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // if set, the game will use this shader for marks on enemies instead of the default "gfx/damage/saberglowmark"
        if Q_stricmp(token, c"g2WeaponMarkShader".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->g2WeaponMarkShader = trap_R_RegisterShader( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // if non-zero, uses damage done to calculate an appropriate amount of knockback
        if Q_stricmp(token, c"knockbackScale".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).knockbackScale = f;
            continue;
        }

        // scale up or down the damage done by the saber
        if Q_stricmp(token, c"damageScale".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).damageScale = f;
            continue;
        }

        // if non-zero, the saber never does dismemberment (good for pointed/blunt melee weapons)
        if Q_stricmp(token, c"noDismemberment".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_DISMEMBERMENT;
            }
            continue;
        }

        // if non-zero, the saber will not do damage or any effects when it is idle (not in an attack anim).  (good for real-sword type mods)
        if Q_stricmp(token, c"noIdleEffect".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_IDLE_EFFECT;
            }
            continue;
        }

        // if set, the blades will always be blocking (good for things like shields that should always block)
        if Q_stricmp(token, c"alwaysBlock".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_ALWAYS_BLOCK;
            }
            continue;
        }

        // if set, the blades cannot manually be toggled on and off
        if Q_stricmp(token, c"noManualDeactivate".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_MANUAL_DEACTIVATE;
            }
            continue;
        }

        // if set, the blade does damage in start, transition and return anims (like strong style does)
        if Q_stricmp(token, c"transitionDamage".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_TRANSITION_DAMAGE;
            }
            continue;
        }

        // splashRadius - radius of splashDamage
        if Q_stricmp(token, c"splashRadius".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).splashRadius = f;
            continue;
        }

        // splashDamage - amount of splashDamage, 100% at a distance of 0, 0% at a distance = splashRadius
        if Q_stricmp(token, c"splashDamage".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).splashDamage = n;
            continue;
        }

        // splashKnockback - amount of splashKnockback, 100% at a distance of 0, 0% at a distance = splashRadius
        if Q_stricmp(token, c"splashKnockback".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).splashKnockback = f;
            continue;
        }

        // hit sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"hitSound1".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).hitSound[0] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // hit sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"hitSound2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).hitSound[1] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // hit sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"hitSound3".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).hitSound[2] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // block sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"blockSound1".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).blockSound[0] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // block sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"blockSound2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).blockSound[1] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // block sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"blockSound3".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).blockSound[2] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // bounce sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"bounceSound1".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).bounceSound[0] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // bounce sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"bounceSound2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).bounceSound[1] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // bounce sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"bounceSound3".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).bounceSound[2] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // block effect - when saber/sword hits another saber/sword
        if Q_stricmp(token, c"blockEffect".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->blockEffect = trap_FX_RegisterEffect( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // hit person effect - when saber/sword hits a person
        if Q_stricmp(token, c"hitPersonEffect".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->hitPersonEffect = trap_FX_RegisterEffect( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // hit other effect - when saber/sword hits something else damagable
        if Q_stricmp(token, c"hitOtherEffect".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->hitOtherEffect = trap_FX_RegisterEffect( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // blade effect
        if Q_stricmp(token, c"bladeEffect".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->bladeEffect = trap_FX_RegisterEffect( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // if non-zero, the saber will not do the big, white clash flare with other sabers
        if Q_stricmp(token, c"noClashFlare".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_CLASH_FLARE;
            }
            continue;
        }

        //===SECONDARY BLADE====================================
        // stops the saber from drawing marks on the world (good for real-sword type mods)
        if Q_stricmp(token, c"noWallMarks2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_WALL_MARKS2;
            }
            continue;
        }

        // stops the saber from drawing a dynamic light (good for real-sword type mods)
        if Q_stricmp(token, c"noDlight2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_DLIGHT2;
            }
            continue;
        }

        // stops the saber from drawing a blade (good for real-sword type mods)
        if Q_stricmp(token, c"noBlade2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_BLADE2;
            }
            continue;
        }

        // default (0) is normal, 1 is a motion blur and 2 is no trail at all (good for real-sword type mods)
        if Q_stricmp(token, c"trailStyle2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).trailStyle2 = n;
            continue;
        }

        // if set, the game will use this shader for marks on enemies instead of the default "gfx/damage/saberglowmark"
        if Q_stricmp(token, c"g2MarksShader2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->g2MarksShader2 = trap_R_RegisterShader( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // if set, the game will use this shader for marks on enemies instead of the default "gfx/damage/saberglowmark"
        if Q_stricmp(token, c"g2WeaponMarkShader2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->g2WeaponMarkShader2 = trap_R_RegisterShader( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // if non-zero, uses damage done to calculate an appropriate amount of knockback
        if Q_stricmp(token, c"knockbackScale2".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).knockbackScale2 = f;
            continue;
        }

        // scale up or down the damage done by the saber
        if Q_stricmp(token, c"damageScale2".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).damageScale2 = f;
            continue;
        }

        // if non-zero, the saber never does dismemberment (good for pointed/blunt melee weapons)
        if Q_stricmp(token, c"noDismemberment2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_DISMEMBERMENT2;
            }
            continue;
        }

        // if non-zero, the saber will not do damage or any effects when it is idle (not in an attack anim).  (good for real-sword type mods)
        if Q_stricmp(token, c"noIdleEffect2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_IDLE_EFFECT2;
            }
            continue;
        }

        // if set, the blades will always be blocking (good for things like shields that should always block)
        if Q_stricmp(token, c"alwaysBlock2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_ALWAYS_BLOCK2;
            }
            continue;
        }

        // if set, the blades cannot manually be toggled on and off
        if Q_stricmp(token, c"noManualDeactivate2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_MANUAL_DEACTIVATE2;
            }
            continue;
        }

        // if set, the blade does damage in start, transition and return anims (like strong style does)
        if Q_stricmp(token, c"transitionDamage2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_TRANSITION_DAMAGE2;
            }
            continue;
        }

        // splashRadius - radius of splashDamage
        if Q_stricmp(token, c"splashRadius2".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).splashRadius2 = f;
            continue;
        }

        // splashDamage - amount of splashDamage, 100% at a distance of 0, 0% at a distance = splashRadius
        if Q_stricmp(token, c"splashDamage2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).splashDamage2 = n;
            continue;
        }

        // splashKnockback - amount of splashKnockback, 100% at a distance of 0, 0% at a distance = splashRadius
        if Q_stricmp(token, c"splashKnockback2".as_ptr()) == 0 {
            if COM_ParseFloat(addr_of_mut!(p), addr_of_mut!(f)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            (*saber).splashKnockback2 = f;
            continue;
        }

        // hit sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"hit2Sound1".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).hit2Sound[0] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // hit sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"hit2Sound2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).hit2Sound[1] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // hit sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"hit2Sound3".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).hit2Sound[2] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // block sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"block2Sound1".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).block2Sound[0] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // block sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"block2Sound2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).block2Sound[1] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // block sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"block2Sound3".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).block2Sound[2] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // bounce sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"bounce2Sound1".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).bounce2Sound[0] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // bounce sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"bounce2Sound2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).bounce2Sound[1] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // bounce sound - NOTE: must provide all 3!!!
        if Q_stricmp(token, c"bounce2Sound3".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            (*saber).bounce2Sound[2] = BG_SoundIndex(&CStr::from_ptr(value).to_string_lossy());
            continue;
        }

        // block effect - when saber/sword hits another saber/sword
        if Q_stricmp(token, c"blockEffect2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->blockEffect2 = trap_FX_RegisterEffect( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // hit person effect - when saber/sword hits a person
        if Q_stricmp(token, c"hitPersonEffect2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->hitPersonEffect2 = trap_FX_RegisterEffect( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // hit other effect - when saber/sword hits something else damagable
        if Q_stricmp(token, c"hitOtherEffect2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->hitOtherEffect2 = trap_FX_RegisterEffect( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // blade effect
        if Q_stricmp(token, c"bladeEffect2".as_ptr()) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            // #ifdef QAGAME — cgame-only cares about this; the rest of the line is
            // skipped. (#elif CGAME: saber->bladeEffect2 = trap_FX_RegisterEffect( value );)
            SkipRestOfLine(addr_of_mut!(p));
            continue;
        }

        // if non-zero, the saber will not do the big, white clash flare with other sabers
        if Q_stricmp(token, c"noClashFlare2".as_ptr()) == 0 {
            if COM_ParseInt(addr_of_mut!(p), addr_of_mut!(n)) != QFALSE {
                SkipRestOfLine(addr_of_mut!(p));
                continue;
            }
            if n != 0 {
                (*saber).saberFlags2 |= SFL2_NO_CLASH_FLARE2;
            }
            continue;
        }
        //===END BLADE-SPECIFIC FIELDS=============================================================================
        // FIXME: saber sounds (on, off, loop)

        // #ifdef _DEBUG — unknown-keyword warning is debug-only (not this build):
        //   Com_Printf( "WARNING: unknown keyword '%s' while parsing '%s'\n", token, useSaber );
        SkipRestOfLine(addr_of_mut!(p));
    }

    // FIXME: precache the saberModel(s)?

    QTRUE
}

/// `qboolean WP_SaberParseParm( const char *saberName, const char *parmname, char *saberData )`
/// (bg_saberLoad.c:1060) — find the named saber's block in [`SaberParms`] and copy a
/// single `parmname` value out into the caller's `saberData` buffer. Returns `qtrue`
/// once the parm is found and copied, `qfalse` on missing saber/parm or EOF. The
/// single-value sibling of [`WP_SaberParseParms`]; engine-facing parser state → no oracle.
///
/// # Safety
/// `saberName`/`parmname` must be null or valid NUL-terminated C strings; `saberData`
/// must point at a buffer large enough for the copied value.
pub unsafe fn WP_SaberParseParm(
    saberName: *const c_char,
    parmname: *const c_char,
    saberData: *mut c_char,
) -> qboolean {
    let mut token: *const c_char;
    let mut value: *const c_char = core::ptr::null();
    let mut p: *const c_char;

    if saberName.is_null() || *saberName == 0 {
        return QFALSE;
    }

    // try to parse it out
    p = addr_of!(SaberParms) as *const c_char;
    COM_BeginParseSession(c"saberinfo".as_ptr());

    // look for the right saber
    while !p.is_null() {
        token = COM_ParseExt(addr_of_mut!(p), QTRUE);
        if *token == 0 {
            return QFALSE;
        }

        if Q_stricmp(token, saberName) == 0 {
            break;
        }

        SkipBracedSection(addr_of_mut!(p));
    }
    if p.is_null() {
        return QFALSE;
    }

    if BG_ParseLiteral(addr_of_mut!(p), c"{".as_ptr()) != QFALSE {
        return QFALSE;
    }

    // parse the saber info block
    loop {
        token = COM_ParseExt(addr_of_mut!(p), QTRUE);
        if *token == 0 {
            Com_Printf(&format!(
                "^1ERROR: unexpected EOF while parsing '{}'\n",
                Sz(saberName)
            ));
            return QFALSE;
        }

        if Q_stricmp(token, c"}".as_ptr()) == 0 {
            break;
        }

        if Q_stricmp(token, parmname) == 0 {
            if COM_ParseString(addr_of_mut!(p), addr_of_mut!(value)) != QFALSE {
                continue;
            }
            strcpy(saberData, value);
            return QTRUE;
        }

        SkipRestOfLine(addr_of_mut!(p));
    }

    QFALSE
}

/// `qboolean WP_SaberValidForPlayerInMP( const char *saberName )` (bg_saberLoad.c:1133) —
/// is this saber legal for a player in MP? Reads the saber's `notInMP` parm via
/// [`WP_SaberParseParm`]; default (undefined / empty) is yes. Engine-facing parser → no oracle.
///
/// # Safety
/// `saberName` must be null or a valid NUL-terminated C string.
pub unsafe fn WP_SaberValidForPlayerInMP(saberName: *const c_char) -> qboolean {
    let mut allowed = [0 as c_char; 8];
    if WP_SaberParseParm(saberName, c"notInMP".as_ptr(), allowed.as_mut_ptr()) == QFALSE {
        // not defined, default is yes
        return QTRUE;
    }
    if allowed[0] == 0 {
        // not defined, default is yes
        QTRUE
    } else {
        // return value
        (atoi(allowed.as_ptr()) == 0) as qboolean
    }
}

/// `void WP_RemoveSaber( saberInfo_t *sabers, int saberNum )` (bg_saberLoad.c:1150) —
/// reset one saber slot back to defaults and turn it off. Calls [`WP_SaberSetDefaults`]
/// (engine-facing via [`BG_SoundIndex`]), so it likewise has no C oracle.
///
/// # Safety
/// `sabers` must be null or point at an array with at least `saberNum + 1` elements.
pub unsafe fn WP_RemoveSaber(sabers: *mut saberInfo_t, saberNum: c_int) {
    if sabers.is_null() {
        return;
    }
    // reset everything for this saber just in case
    let s = sabers.add(saberNum as usize);
    WP_SaberSetDefaults(s);

    strcpy((*s).name.as_mut_ptr(), c"none".as_ptr());
    (*s).model[0] = 0;

    //ent->client->ps.dualSabers = qfalse;
    BG_SI_Deactivate(s);
    BG_SI_SetLength(s, 0.0);
    // (the ghoul2/weaponModel removal + dualSabers reset are SP-only, commented out in C)
}

/// `void WP_SetSaber( int entNum, saberInfo_t *sabers, int saberNum, const char *saberName )`
/// (bg_saberLoad.c:1176) — assign saber `saberNum` from `saberName`. `"none"`/`"remove"`
/// strips the slot (never slot 0); a player (`entNum < MAX_CLIENTS`) requesting an
/// MP-invalid saber gets `"Kyle"` instead; otherwise the named saber is parsed in. Then
/// enforces the two-handed rule: a two-handed saber can't share with a second saber.
/// Routes through [`WP_SaberParseParms`]/[`WP_RemoveSaber`] (engine-facing) → no oracle.
///
/// # Safety
/// `sabers` must be null or point at an array of at least 2 [`saberInfo_t`] (the
/// two-handed checks read slots 0 and 1) indexable by `saberNum`; `saberName` a valid
/// NUL-terminated C string.
pub unsafe fn WP_SetSaber(
    entNum: c_int,
    sabers: *mut saberInfo_t,
    saberNum: c_int,
    saberName: *const c_char,
) {
    if sabers.is_null() {
        return;
    }
    if Q_stricmp(c"none".as_ptr(), saberName) == 0 || Q_stricmp(c"remove".as_ptr(), saberName) == 0
    {
        if saberNum != 0 {
            // can't remove saber 0 ever
            WP_RemoveSaber(sabers, saberNum);
        }
        return;
    }

    if entNum < MAX_CLIENTS as c_int && WP_SaberValidForPlayerInMP(saberName) == QFALSE {
        WP_SaberParseParms(c"Kyle".as_ptr(), sabers.add(saberNum as usize)); //get saber info
    } else {
        WP_SaberParseParms(saberName, sabers.add(saberNum as usize)); //get saber info
    }
    if (*sabers.add(1)).saberFlags & SFL_TWO_HANDED != 0 {
        // not allowed to use a 2-handed saber as second saber
        WP_RemoveSaber(sabers, 1);
        return;
    } else if (*sabers.add(0)).saberFlags & SFL_TWO_HANDED != 0 && (*sabers.add(1)).model[0] != 0 {
        // you can't use a two-handed saber with a second saber, so remove saber 2
        WP_RemoveSaber(sabers, 1);
    }
}

/// `void WP_SaberSetColor( saberInfo_t *sabers, int saberNum, int bladeNum, char *colorName )`
/// (bg_saberLoad.c:1213) — set one blade's color from a color name.
///
/// # Safety
/// `sabers` must be null or point at an array indexable by `saberNum`, whose
/// `blade[bladeNum]` is in range; `colorName` must be a valid NUL-terminated C string.
pub unsafe fn WP_SaberSetColor(
    sabers: *mut saberInfo_t,
    saberNum: c_int,
    bladeNum: c_int,
    colorName: *const c_char,
) {
    if sabers.is_null() {
        return;
    }
    (*sabers.add(saberNum as usize)).blade[bladeNum as usize].color =
        TranslateSaberColor(colorName);
}

/// `void WP_SaberLoadParms( void )` (bg_saberLoad.c:1228) — concatenate every
/// `ext_data/sabers/*.sab` file into the global [`SaberParms`] text block (each chunk
/// separated by a `"\n"` so a `.sab` file missing a trailing newline can't merge tokens
/// with the next). The text is later tokenized by [`WP_SaberParseParms`] /
/// [`WP_SaberParseParm`].
///
/// Faithful to the retail `QAGAME` build: `trap_FS_GetFileList` + the `trap_FS_*` file
/// traps. The C `va( "ext_data/sabers/%s", holdChar )` collapses into a Rust `format!`
/// since [`trap::FS_FOpenFile`] already takes `&str`. `mainBlockLen` is dropped — dead in
/// C (assigned `len`, never read). The temp read buffer is Raven's `Z_Malloc(
/// MAX_SABER_DATA_SIZE, TAG_TEMP_WORKSPACE, qfalse, 4 )` / `Z_Free`; there is no game-VM
/// `Z_Malloc` syscall, so it maps to a heap `Vec<u8>` of the same size, dropped at scope
/// end (= the `Z_Free`) — the engine-zone scratch alloc the source intends (and what the
/// no-zero / align-4 flags are immaterial to for a NUL-terminated parse buffer). See
/// `DEVIATIONS.md`.
///
/// No oracle — pure engine-trap file I/O (`trap_FS_*`), which the off-engine oracle
/// harness cannot satisfy (the `BG_VehWeaponLoadParms` precedent).
pub fn WP_SaberLoadParms() {
    let mut len: c_int;
    let mut totallen: c_int;
    let fileCnt: c_int;
    let mut saberExtensionListBuf = [0 as c_char; 2048]; // The list of file names read in

    len = 0;

    // SAFETY: single-threaded module; the global `SaberParms` text block and the temp
    // read buffer are walked with raw pointers exactly as the C does.
    unsafe {
        // remember where to store the next one
        totallen = len;
        let base = addr_of_mut!(SaberParms) as *mut c_char;
        let mut marker = base.add(totallen as usize);
        *marker = 0;

        // Z_Malloc( MAX_SABER_DATA_SIZE, TAG_TEMP_WORKSPACE, qfalse, 4 ) — heap scratch.
        let mut tbuf = vec![0u8; MAX_SABER_DATA_SIZE];
        let bgSaberParseTBuffer = tbuf.as_mut_ptr() as *mut c_char;

        // now load in the extra .sab extensions
        fileCnt = trap::FS_GetFileList("ext_data/sabers", ".sab", &mut saberExtensionListBuf);

        let mut holdChar = saberExtensionListBuf.as_mut_ptr();

        let mut i = 0;
        while i < fileCnt {
            let saberExtFNLen = strlen(holdChar) as c_int;

            let path = format!(
                "ext_data/sabers/{}",
                CStr::from_ptr(holdChar).to_string_lossy()
            );
            let (l, f) = trap::FS_FOpenFile(&path, FS_READ);
            len = l;

            if len == -1 {
                Com_Printf("error reading file\n");
            } else {
                if totallen + len + 1/*for the endline*/ >= MAX_SABER_DATA_SIZE as c_int {
                    Com_Error(ERR_DROP, "Saber extensions (*.sab) are too large");
                }

                let buf =
                    core::slice::from_raw_parts_mut(bgSaberParseTBuffer as *mut u8, len as usize);
                trap::FS_Read(buf, f);
                *bgSaberParseTBuffer.add(len as usize) = 0;

                len = COM_Compress(bgSaberParseTBuffer);

                Q_strcat(
                    marker,
                    MAX_SABER_DATA_SIZE as c_int - totallen,
                    bgSaberParseTBuffer,
                );
                trap::FS_FCloseFile(f);

                // get around the stupid problem of not having an endline at the bottom
                // of a sab file -rww
                Q_strcat(
                    marker,
                    MAX_SABER_DATA_SIZE as c_int - totallen,
                    c"\n".as_ptr(),
                );
                len += 1;

                totallen += len;
                marker = base.add(totallen as usize);
            }

            i += 1;
            holdChar = holdChar.add((saberExtFNLen + 1) as usize);
        }
        // Z_Free( bgSaberParseTBuffer ) — `tbuf` drops here.
    }
}

// rww — The following were struct functions in SP. Of course we can't have that in
// this codebase so I'm having to externalize them. Which is why this probably seems
// structured a bit oddly. But it's to make porting stuff easier on myself. SI
// indicates it was under saberinfo, and BLADE indicates it was under bladeinfo.

//---------------------------------------
/// `void BG_BLADE_ActivateTrail ( bladeInfo_t *blade, float duration )`
/// (bg_saberLoad.c:1293). The C++ `ClientManager::ActiveClientNum()` trail index is
/// the `_XBOX` split-screen active client; on the non-`_XBOX`/PC build it collapses
/// to the single active client, index `0` (what OpenJK models as an unindexed
/// `trail`). See `DEVIATIONS.md`.
///
/// # Safety
/// `blade` must point at a valid `bladeInfo_t`.
pub unsafe fn BG_BLADE_ActivateTrail(blade: *mut bladeInfo_t, duration: f32) {
    (*blade).trail.inAction = QTRUE;
    (*blade).trail.duration = duration as c_int;
}

/// `void BG_BLADE_DeactivateTrail ( bladeInfo_t *blade, float duration )`
/// (bg_saberLoad.c:1299). Trail index `0` as in [`BG_BLADE_ActivateTrail`].
///
/// # Safety
/// `blade` must point at a valid `bladeInfo_t`.
pub unsafe fn BG_BLADE_DeactivateTrail(blade: *mut bladeInfo_t, duration: f32) {
    (*blade).trail.inAction = QFALSE;
    (*blade).trail.duration = duration as c_int;
}

//---------------------------------------
/// `void BG_SI_Activate( saberInfo_t *saber )` (bg_saberLoad.c:1305).
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_Activate(saber: *mut saberInfo_t) {
    for i in 0..(*saber).numBlades {
        (*saber).blade[i as usize].active = QTRUE;
    }
}

/// `void BG_SI_Deactivate( saberInfo_t *saber )` (bg_saberLoad.c:1315).
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_Deactivate(saber: *mut saberInfo_t) {
    for i in 0..(*saber).numBlades {
        (*saber).blade[i as usize].active = QFALSE;
    }
}

/// `void BG_SI_BladeActivate( saberInfo_t *saber, int iBlade, qboolean bActive )`
/// (bg_saberLoad.c:1330) — activate (or deactivate) one specific blade.
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t`.
pub unsafe fn BG_SI_BladeActivate(saber: *mut saberInfo_t, iBlade: c_int, bActive: qboolean) {
    // Validate blade ID/Index.
    if iBlade < 0 || iBlade >= (*saber).numBlades {
        return;
    }

    (*saber).blade[iBlade as usize].active = bActive;
}

/// `qboolean BG_SI_Active(saberInfo_t *saber)` (bg_saberLoad.c:1339) — true if any
/// blade is active.
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_Active(saber: *mut saberInfo_t) -> qboolean {
    for i in 0..(*saber).numBlades {
        if (*saber).blade[i as usize].active != 0 {
            return QTRUE;
        }
    }
    QFALSE
}

/// `void BG_SI_SetLength( saberInfo_t *saber, float length )` (bg_saberLoad.c:1353).
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_SetLength(saber: *mut saberInfo_t, length: f32) {
    for i in 0..(*saber).numBlades {
        (*saber).blade[i as usize].length = length;
    }
}

/// `void BG_SI_SetDesiredLength(saberInfo_t *saber, float len, int bladeNum )`
/// (bg_saberLoad.c:1364) — not in SP, added for convenience. `bladeNum < 0` (or out
/// of range) sets every blade; otherwise just that blade.
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_SetDesiredLength(saber: *mut saberInfo_t, len: f32, bladeNum: c_int) {
    let mut start_blade: c_int = 0;
    let mut max_blades: c_int = (*saber).numBlades;

    if bladeNum >= 0 && bladeNum < (*saber).numBlades {
        // doing this on a specific blade
        start_blade = bladeNum;
        max_blades = bladeNum + 1;
    }
    for i in start_blade..max_blades {
        (*saber).blade[i as usize].desiredLength = len;
    }
}

/// `void BG_SI_SetLengthGradual(saberInfo_t *saber, int time)` (bg_saberLoad.c:1380)
/// — also not in SP; steps each blade's `length` toward its `desiredLength`
/// (`-1` ⇒ `lengthMax`) by a time-scaled amount.
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_SetLengthGradual(saber: *mut saberInfo_t, time: c_int) {
    for i in 0..(*saber).numBlades {
        let i = i as usize;
        let mut d_len: f32 = (*saber).blade[i].desiredLength;

        if d_len == -1.0 {
            // assume we want max blade len
            d_len = (*saber).blade[i].lengthMax;
        }

        if (*saber).blade[i].length == d_len {
            continue;
        }

        if (*saber).blade[i].length == (*saber).blade[i].lengthMax
            || (*saber).blade[i].length == 0.0
        {
            (*saber).blade[i].extendDebounce = time;
            if (*saber).blade[i].length == 0.0 {
                (*saber).blade[i].length += 1.0;
            } else {
                (*saber).blade[i].length -= 1.0;
            }
        }

        // C: amt = (time - extendDebounce)*0.01 — int diff promoted to double by the
        // `0.01` literal, then narrowed to the float `amt`. Replicate with an f64
        // intermediate so the rounding matches bit-for-bit.
        let mut amt: f32 = ((time - (*saber).blade[i].extendDebounce) as f64 * 0.01) as f32;

        if amt < 0.2 {
            amt = 0.2;
        }

        if (*saber).blade[i].length < d_len {
            (*saber).blade[i].length += amt;

            if (*saber).blade[i].length > d_len {
                (*saber).blade[i].length = d_len;
            }
            if (*saber).blade[i].length > (*saber).blade[i].lengthMax {
                (*saber).blade[i].length = (*saber).blade[i].lengthMax;
            }
        } else if (*saber).blade[i].length > d_len {
            (*saber).blade[i].length -= amt;

            if (*saber).blade[i].length < d_len {
                (*saber).blade[i].length = d_len;
            }
            if (*saber).blade[i].length < 0.0 {
                (*saber).blade[i].length = 0.0;
            }
        }
    }
}

/// `float BG_SI_Length(saberInfo_t *saber)` (bg_saberLoad.c:1449) — the largest blade
/// length. Note the C accumulator is `int`, so the result is truncated toward zero.
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_Length(saber: *mut saberInfo_t) -> f32 {
    // return largest length
    let mut len1: c_int = 0;
    for i in 0..(*saber).numBlades {
        if (*saber).blade[i as usize].length > len1 as f32 {
            len1 = (*saber).blade[i as usize].length as c_int;
        }
    }
    len1 as f32
}

/// `float BG_SI_LengthMax(saberInfo_t *saber)` (bg_saberLoad.c:1464) — the largest
/// blade `lengthMax`. As with [`BG_SI_Length`] the `int` accumulator truncates.
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_LengthMax(saber: *mut saberInfo_t) -> f32 {
    let mut len1: c_int = 0;
    for i in 0..(*saber).numBlades {
        if (*saber).blade[i as usize].lengthMax > len1 as f32 {
            len1 = (*saber).blade[i as usize].lengthMax as c_int;
        }
    }
    len1 as f32
}

/// `void BG_SI_ActivateTrail ( saberInfo_t *saber, float duration )`
/// (bg_saberLoad.c:1479).
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_ActivateTrail(saber: *mut saberInfo_t, duration: f32) {
    for i in 0..(*saber).numBlades {
        BG_BLADE_ActivateTrail(addr_of_mut!((*saber).blade[i as usize]), duration);
    }
}

/// `void BG_SI_DeactivateTrail ( saberInfo_t *saber, float duration )`
/// (bg_saberLoad.c:1490).
///
/// # Safety
/// `saber` must point at a valid `saberInfo_t` with `numBlades <= MAX_BLADES`.
pub unsafe fn BG_SI_DeactivateTrail(saber: *mut saberInfo_t, duration: f32) {
    for i in 0..(*saber).numBlades {
        BG_BLADE_DeactivateTrail(addr_of_mut!((*saber).blade[i as usize]), duration);
    }
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle;
    use core::ptr::addr_of;

    /// Drives the Rust `SaberTable` through the authentic C `GetIDForString` /
    /// `GetStringForID` (oracle, real q_shared.c) — confirms each `ENUM2STRING` row
    /// pairs the right name with the right id, the case-fold path (`Q_stricmp`), a
    /// miss, and that the `{ "", -1 }` terminator bounds the scan as C expects.
    #[test]
    fn saber_table_lookups_match_c() {
        // `addr_of!` takes the address of the `static mut` without forming a reference
        // (no `static_mut_refs`); creating the raw pointer is safe.
        let tbl = addr_of!(SaberTable) as *const stringID_table_t;

        let cases: &[(&CStr, saberType_t)] = &[
            (c"SABER_NONE", SABER_NONE),
            (c"SABER_SINGLE", SABER_SINGLE),
            (c"SABER_STAFF", SABER_STAFF),
            (c"SABER_BROAD", SABER_BROAD),
            (c"SABER_PRONG", SABER_PRONG),
            (c"SABER_DAGGER", SABER_DAGGER),
            (c"SABER_ARC", SABER_ARC),
            (c"SABER_SAI", SABER_SAI),
            (c"SABER_CLAW", SABER_CLAW),
            (c"SABER_LANCE", SABER_LANCE),
            (c"SABER_STAR", SABER_STAR),
            (c"SABER_TRIDENT", SABER_TRIDENT),
            (c"saber_single", SABER_SINGLE), // case-fold
            (c"SABER_MISSING", -1),          // miss
            (c"", -1),                       // empty → terminator
        ];
        for &(name, expect) in cases {
            let o = unsafe { oracle::GetIDForString(tbl, name.as_ptr()) };
            assert_eq!(o, expect, "GetIDForString({name:?})");
        }

        for id in [
            SABER_NONE,
            SABER_SINGLE,
            SABER_STAFF,
            SABER_BROAD,
            SABER_PRONG,
            SABER_DAGGER,
            SABER_ARC,
            SABER_SAI,
            SABER_CLAW,
            SABER_LANCE,
            SABER_STAR,
            SABER_TRIDENT,
        ] {
            let s = unsafe { oracle::GetStringForID(tbl, id) };
            assert!(!s.is_null(), "GetStringForID({id}) null");
            let back = unsafe { oracle::GetIDForString(tbl, s) };
            assert_eq!(back, id, "GetStringForID({id}) round-trip");
        }
    }

    /// `TranslateSaberColor` vs the verbatim C oracle: the deterministic (named +
    /// miss + case-fold) paths must match exactly; the "random" path (independent
    /// RNG states) is checked by set-membership on both sides instead.
    #[test]
    fn translate_saber_color_matches_c() {
        let deterministic: &[&CStr] = &[
            c"red",
            c"orange",
            c"yellow",
            c"green",
            c"blue",
            c"purple", // named
            c"RED",
            c"Orange",
            c"PuRpLe", // case-fold (Q_stricmp)
            c"",
            c"nonsense",
            c"bluish", // misses → SABER_BLUE
        ];
        for &name in deterministic {
            let r = unsafe { TranslateSaberColor(name.as_ptr()) };
            let c = unsafe { oracle::TranslateSaberColor(name.as_ptr()) };
            assert_eq!(r, c, "TranslateSaberColor({name:?})");
        }

        // "random" → Q_irand(SABER_ORANGE, SABER_PURPLE); RNG states differ between
        // Rust and C, so just assert both stay in range. Hold the shared-RNG lock so
        // the Rust call can't perturb the bg_lib rand-parity loops under parallel runs.
        {
            let _g = crate::codemp::game::bg_lib::rand_lock();
            let r = unsafe { TranslateSaberColor(c"random".as_ptr()) };
            assert!(
                (SABER_ORANGE..=SABER_PURPLE).contains(&r),
                "random rust in range"
            );
        }
        let c = unsafe { oracle::TranslateSaberColor(c"random".as_ptr()) };
        assert!(
            (SABER_ORANGE..=SABER_PURPLE).contains(&c),
            "random C in range"
        );
    }

    /// Build a NUL-terminated C buffer (`Vec<c_char>`) from a Rust string.
    fn cbuf(s: &str) -> Vec<c_char> {
        let mut v: Vec<c_char> = s.bytes().map(|b| b as c_char).collect();
        v.push(0);
        v
    }

    /// Install a benign no-op engine syscall pointer so trap-backed calls (here,
    /// `Com_Printf` → `trap_Printf` → `G_PRINT`) don't dereference the unset (null)
    /// syscall pointer in a unit-test process. The stub ignores its args and returns
    /// `0` (the engine's "did nothing" reply). Global + idempotent — harmless to
    /// other tests, none of which depend on the pointer being unset.
    fn install_noop_syscall() {
        unsafe extern "C" fn noop(_arg: isize) -> isize {
            0
        }
        // The real `SyscallFn` is variadic (`fn(isize, ...)`), which stable Rust
        // cannot *define*; a fixed-arity cdecl callee that ignores the extra args is
        // ABI-compatible as the callee, so transmute a 1-arg fn into the slot.
        let f: crate::ffi::syscalls::SyscallFn =
            unsafe { core::mem::transmute(noop as unsafe extern "C" fn(isize) -> isize) };
        crate::ffi::syscalls::set_syscall(f);
    }

    /// `BG_ParseLiteral` vs the verbatim C oracle: drives both with the same input
    /// + required keyword and compares the `qboolean` result AND the advanced parser
    /// cursor offset (`COM_ParseExt` is bit-exact, so both must step identically).
    /// Holds `parse_lock()` — the Rust call mutates the shared `com_token` global.
    #[test]
    fn bg_parse_literal_matches_c() {
        let _g = crate::codemp::game::q_shared::parse_lock();
        install_noop_syscall(); // Com_Printf (error paths) routes through trap_Printf

        // (input stream, required keyword)
        let cases: &[(&str, &str)] = &[
            ("weapon foo", "weapon"),  // match
            ("Weapon foo", "weapon"),  // case-fold match
            ("foo bar", "weapon"),     // mismatch
            ("", "weapon"),            // EOF (empty)
            ("   weapon x", "weapon"), // leading whitespace
            ("{ weapon", "weapon"),    // punctuation token first → mismatch
            ("name", "name"),          // exact single token, then EOF
        ];
        for &(input, req) in cases {
            let reqbuf = cbuf(req);

            // Rust
            let rbuf = cbuf(input);
            let mut rptr = rbuf.as_ptr();
            let rret = unsafe { BG_ParseLiteral(&mut rptr, reqbuf.as_ptr()) };

            // C oracle
            let cbuf2 = cbuf(input);
            let mut cptr = cbuf2.as_ptr();
            let cret = unsafe { oracle::BG_ParseLiteral(&mut cptr, reqbuf.as_ptr()) };

            assert_eq!(rret, cret, "BG_ParseLiteral ret for {input:?}/{req:?}");
            // COM_ParseExt sets the cursor to NULL at EOF; otherwise it points into
            // its (distinct) buffer, so compare null-ness, then the advance offset
            // only when both advanced within their buffers.
            assert_eq!(
                rptr.is_null(),
                cptr.is_null(),
                "cursor null-ness for {input:?}/{req:?}"
            );
            if !rptr.is_null() {
                let roff = unsafe { rptr.offset_from(rbuf.as_ptr()) };
                let coff = unsafe { cptr.offset_from(cbuf2.as_ptr()) };
                assert_eq!(roff, coff, "BG_ParseLiteral cursor for {input:?}/{req:?}");
            }
        }
    }

    /// Zeroed `saberInfo_t` (pointer-free POD, layout verified in q_shared_h_oracle.c).
    fn blank_saber() -> saberInfo_t {
        unsafe { core::mem::zeroed() }
    }

    /// Apply a unit-returning accessor to identical Rust/C copies of `s` and assert
    /// the mutated structs match bit-for-bit (`PartialEq`).
    macro_rules! check_mut {
        ($s:expr, $rust:path, $c:path $(, $arg:expr)*) => {{
            let mut r = $s;
            let mut c = $s;
            unsafe { $rust(&mut r $(, $arg)*); }
            unsafe { $c(&mut c $(, $arg)*); }
            assert_eq!(r, c, concat!(stringify!($rust), " mutated state"));
        }};
    }

    /// As [`check_mut`] but also compares the return value.
    macro_rules! check_ret {
        ($s:expr, $rust:path, $c:path $(, $arg:expr)*) => {{
            let mut r = $s;
            let mut c = $s;
            let rv = unsafe { $rust(&mut r $(, $arg)*) };
            let cv = unsafe { $c(&mut c $(, $arg)*) };
            assert_eq!(rv, cv, concat!(stringify!($rust), " return"));
            assert_eq!(r, c, concat!(stringify!($rust), " state"));
        }};
    }

    /// The `BG_SI_*` / `BG_BLADE_*` accessor family vs the verbatim C oracle (which
    /// operates on a layout-verified struct transcription, so a Rust `*mut saberInfo_t`
    /// passes straight in). Drives a spread of input states — empty, single, mixed,
    /// and full 8-blade sabers hitting every `SetLengthGradual` branch — and compares
    /// the mutated struct (and any return value) for each accessor.
    #[test]
    fn saber_accessors_match_c() {
        // Build the representative input sabers.
        let s_empty = {
            let mut s = blank_saber();
            s.numBlades = 0;
            s
        };

        let s1 = {
            let mut s = blank_saber();
            s.numBlades = 1;
            s.blade[0].length = 16.0;
            s.blade[0].lengthMax = 32.0;
            s.blade[0].desiredLength = 32.0;
            s.blade[0].extendDebounce = 100;
            s
        };

        let s3 = {
            let mut s = blank_saber();
            s.numBlades = 3;
            s.blade[0].length = 0.0; // SetLengthGradual: length==0 -> length++
            s.blade[0].lengthMax = 40.0;
            s.blade[0].desiredLength = 40.0;
            s.blade[1].length = 40.0; // length==lengthMax -> length--
            s.blade[1].lengthMax = 40.0;
            s.blade[1].desiredLength = 10.0;
            s.blade[1].active = QTRUE;
            s.blade[2].length = 25.0; // mid: retract toward desired
            s.blade[2].lengthMax = 32.0;
            s.blade[2].desiredLength = -1.0; // -> lengthMax
            s.blade[2].extendDebounce = 5;
            s
        };

        let s8 = {
            let mut s = blank_saber();
            s.numBlades = 8;
            for b in 0..8usize {
                s.blade[b].length = (b as f32) * 4.0;
                s.blade[b].lengthMax = 28.0 + b as f32;
                s.blade[b].desiredLength = if b % 2 == 0 { -1.0 } else { 12.0 };
                s.blade[b].extendDebounce = (b as c_int) * 13;
                s.blade[b].active = if b % 3 == 0 { QTRUE } else { QFALSE };
            }
            s
        };

        let sabers = [s_empty, s1, s3, s8];

        for &s in &sabers {
            check_mut!(s, BG_SI_Activate, oracle::BG_SI_Activate);
            check_mut!(s, BG_SI_Deactivate, oracle::BG_SI_Deactivate);
            check_ret!(s, BG_SI_Active, oracle::BG_SI_Active);
            check_ret!(s, BG_SI_Length, oracle::BG_SI_Length);
            check_ret!(s, BG_SI_LengthMax, oracle::BG_SI_LengthMax);

            // BladeActivate across in-range, boundary and out-of-range indices.
            for ib in [-1, 0, 1, 2, 7, 8] {
                check_mut!(
                    s,
                    BG_SI_BladeActivate,
                    oracle::BG_SI_BladeActivate,
                    ib,
                    QTRUE
                );
                check_mut!(
                    s,
                    BG_SI_BladeActivate,
                    oracle::BG_SI_BladeActivate,
                    ib,
                    QFALSE
                );
            }

            // Float setters — include a fractional value (trail.duration truncates).
            for len in [0.0f32, 7.5, 24.0, 33.0] {
                check_mut!(s, BG_SI_SetLength, oracle::BG_SI_SetLength, len);
                for bn in [-1, 0, 1, 9] {
                    check_mut!(
                        s,
                        BG_SI_SetDesiredLength,
                        oracle::BG_SI_SetDesiredLength,
                        len,
                        bn
                    );
                }
            }
            for dur in [0.0f32, 9.0, 60.7, 250.9] {
                check_mut!(s, BG_SI_ActivateTrail, oracle::BG_SI_ActivateTrail, dur);
                check_mut!(s, BG_SI_DeactivateTrail, oracle::BG_SI_DeactivateTrail, dur);
            }

            // SetLengthGradual across several time stamps (hits every branch).
            for time in [0, 1, 50, 100, 300, 1000] {
                check_mut!(
                    s,
                    BG_SI_SetLengthGradual,
                    oracle::BG_SI_SetLengthGradual,
                    time
                );
            }
        }

        // BG_BLADE_*Trail directly (single bladeInfo_t), with a fractional duration.
        for dur in [0.0f32, 12.0, 99.9] {
            let mut rb = s8.blade[0];
            let mut cb = s8.blade[0];
            unsafe { BG_BLADE_ActivateTrail(&mut rb, dur) };
            unsafe { oracle::BG_BLADE_ActivateTrail(&mut cb, dur) };
            assert_eq!(rb, cb, "BG_BLADE_ActivateTrail({dur})");

            let mut rb = s8.blade[0];
            let mut cb = s8.blade[0];
            unsafe { BG_BLADE_DeactivateTrail(&mut rb, dur) };
            unsafe { oracle::BG_BLADE_DeactivateTrail(&mut cb, dur) };
            assert_eq!(rb, cb, "BG_BLADE_DeactivateTrail({dur})");
        }
    }

    /// `WP_SaberSetColor` vs the verbatim C oracle — sets `sabers[saberNum].blade
    /// [bladeNum].color = TranslateSaberColor(name)`. Driven over a 2-saber array
    /// (MAX_SABERS), several blade indices and deterministic color names (named +
    /// case-fold + miss); the "random" name is excluded (RNG divergence, covered by
    /// `translate_saber_color_matches_c`). Also exercises the null-`sabers` guard.
    #[test]
    fn wp_saber_set_color_matches_c() {
        let base = {
            let mut a = [blank_saber(); 2];
            a[0].numBlades = 8;
            a[1].numBlades = 8;
            a
        };
        let colors: &[&CStr] = &[
            c"red", c"orange", c"yellow", c"green", c"blue", c"purple", c"RED", c"bogus", c"",
        ];
        for &col in colors {
            for sn in [0, 1] {
                for bn in [0, 1, 7] {
                    let mut r = base;
                    let mut c = base;
                    unsafe { WP_SaberSetColor(r.as_mut_ptr(), sn, bn, col.as_ptr()) };
                    unsafe { oracle::WP_SaberSetColor(c.as_mut_ptr(), sn, bn, col.as_ptr()) };
                    assert_eq!(r, c, "WP_SaberSetColor sn={sn} bn={bn} col={col:?}");
                }
            }
        }
        // null guard: must not touch memory / must not crash.
        unsafe { WP_SaberSetColor(core::ptr::null_mut(), 0, 0, c"red".as_ptr()) };
    }

    /// `TranslateSaberStyle` vs the verbatim C oracle — fully deterministic.
    #[test]
    fn translate_saber_style_matches_c() {
        let names: &[&CStr] = &[
            c"fast", c"medium", c"strong", c"desann", c"tavion", c"dual", c"staff", // named
            c"Fast", c"STAFF", // case-fold
            c"", c"none", c"bogus", // misses → SS_NONE
        ];
        for &name in names {
            let r = unsafe { TranslateSaberStyle(name.as_ptr()) };
            let c = unsafe { oracle::TranslateSaberStyle(name.as_ptr()) };
            assert_eq!(r, c, "TranslateSaberStyle({name:?})");
        }
    }

    /// Build a saberInfo_t with the fields the blade-style predicates read.
    /// `has_model` sets `model[0]` non-zero (a saber "has a model" iff `model[0]`).
    fn mk_saber(
        num_blades: c_int,
        blade_style2_start: c_int,
        saber_flags2: c_int,
        styles_forbidden: c_int,
        styles_learned: c_int,
        has_model: bool,
    ) -> saberInfo_t {
        let mut s = blank_saber();
        s.numBlades = num_blades;
        s.bladeStyle2Start = blade_style2_start;
        s.saberFlags2 = saber_flags2;
        s.stylesForbidden = styles_forbidden;
        s.stylesLearned = styles_learned;
        if has_model {
            s.model[0] = b'a' as c_char;
        }
        s
    }

    /// The per-blade transition-damage / always-on predicates vs the verbatim C oracle:
    /// [`WP_SaberBladeUseSecondBladeStyle`], [`WP_SaberBladeDoTransitionDamage`] (over a
    /// spread of `bladeNum`) and [`WP_SaberCanTurnOffSomeBlades`], across sabers that hit
    /// every `bladeStyle2Start` / `SFL2_TRANSITION_DAMAGE*` / `SFL2_NO_MANUAL_DEACTIVATE*`
    /// branch. Includes the null-`saber` guard on `WP_SaberBladeUseSecondBladeStyle`.
    #[test]
    fn saber_blade_predicates_match_c() {
        let mut variants = [
            mk_saber(1, 0, 0, 0, 0, true),
            mk_saber(3, 1, 0, 0, 0, true),
            mk_saber(3, 2, SFL2_TRANSITION_DAMAGE, 0, 0, true),
            mk_saber(3, 2, SFL2_TRANSITION_DAMAGE2, 0, 0, true),
            mk_saber(
                3,
                2,
                SFL2_TRANSITION_DAMAGE | SFL2_TRANSITION_DAMAGE2,
                0,
                0,
                true,
            ),
            mk_saber(3, 0, SFL2_NO_MANUAL_DEACTIVATE, 0, 0, true),
            mk_saber(3, 2, SFL2_NO_MANUAL_DEACTIVATE, 0, 0, true),
            mk_saber(
                3,
                2,
                SFL2_NO_MANUAL_DEACTIVATE | SFL2_NO_MANUAL_DEACTIVATE2,
                0,
                0,
                true,
            ),
            mk_saber(
                2,
                2,
                SFL2_NO_MANUAL_DEACTIVATE | SFL2_NO_MANUAL_DEACTIVATE2,
                0,
                0,
                true,
            ),
        ];
        for v in variants.iter_mut() {
            let p = v as *mut saberInfo_t;
            for bn in [-1, 0, 1, 2, 3, 5, 8] {
                let r = unsafe { WP_SaberBladeUseSecondBladeStyle(p, bn) };
                let c = unsafe { oracle::WP_SaberBladeUseSecondBladeStyle(p, bn) };
                assert_eq!(r, c, "WP_SaberBladeUseSecondBladeStyle bn={bn}");

                let r = unsafe { WP_SaberBladeDoTransitionDamage(p, bn) };
                let c = unsafe { oracle::WP_SaberBladeDoTransitionDamage(p, bn) };
                assert_eq!(r, c, "WP_SaberBladeDoTransitionDamage bn={bn}");
            }
            let r = unsafe { WP_SaberCanTurnOffSomeBlades(p) };
            let c = unsafe { oracle::WP_SaberCanTurnOffSomeBlades(p) };
            assert_eq!(r, c, "WP_SaberCanTurnOffSomeBlades");
        }

        // null guard on WP_SaberBladeUseSecondBladeStyle.
        for bn in [-1, 0, 1] {
            let r = unsafe { WP_SaberBladeUseSecondBladeStyle(core::ptr::null_mut(), bn) };
            let c = unsafe { oracle::WP_SaberBladeUseSecondBladeStyle(core::ptr::null_mut(), bn) };
            assert_eq!(r, c, "WP_SaberBladeUseSecondBladeStyle null bn={bn}");
        }
    }

    /// The whole-saber valid-style predicates vs the verbatim C oracle:
    /// [`WP_SaberStyleValidForSaber`] and [`WP_UseFirstValidSaberStyle`] (which also
    /// writes back through `saberAnimLevel` — compared too), over the full cross of
    /// {null, no-model, single, staff, forbidden-styles, tavion-learned} saber1×saber2,
    /// every `saberHolstered` activation case, and every saber style.
    #[test]
    fn saber_valid_style_predicates_match_c() {
        let mut variants = [
            mk_saber(1, 0, 0, 0, 0, true),            // single, no restrictions
            mk_saber(1, 0, 0, 1 << SS_FAST, 0, true), // single, forbids fast
            mk_saber(2, 0, 0, 1 << SS_MEDIUM, 1 << SS_TAVION, true), // staff, forbids medium, learned tavion
            mk_saber(
                1,
                0,
                0,
                (1 << SS_FAST) | (1 << SS_MEDIUM) | (1 << SS_STRONG),
                1 << SS_TAVION,
                true,
            ),
            mk_saber(1, 0, 0, 1 << SS_DUAL, 0, true), // forbids dual
            mk_saber(1, 0, 0, 0, 0, false),           // no model
        ];

        // Pointer list: null + one per variant.
        let mut ptrs: Vec<*mut saberInfo_t> = vec![core::ptr::null_mut()];
        for v in variants.iter_mut() {
            ptrs.push(v as *mut saberInfo_t);
        }

        let levels = [
            SS_NONE, SS_FAST, SS_MEDIUM, SS_STRONG, SS_DESANN, SS_TAVION, SS_DUAL, SS_STAFF,
        ];

        for &s1 in &ptrs {
            for &s2 in &ptrs {
                for h in [0, 1, 2, 3] {
                    for lvl in levels {
                        let r = unsafe { WP_SaberStyleValidForSaber(s1, s2, h, lvl) };
                        let c = unsafe { oracle::WP_SaberStyleValidForSaber(s1, s2, h, lvl) };
                        assert_eq!(
                            r, c,
                            "WP_SaberStyleValidForSaber s1={s1:?} s2={s2:?} h={h} lvl={lvl}"
                        );

                        let mut rlvl = lvl;
                        let mut clvl = lvl;
                        let r = unsafe { WP_UseFirstValidSaberStyle(s1, s2, h, &mut rlvl) };
                        let c = unsafe { oracle::WP_UseFirstValidSaberStyle(s1, s2, h, &mut clvl) };
                        assert_eq!(
                            r, c,
                            "WP_UseFirstValidSaberStyle ret s1={s1:?} s2={s2:?} h={h} lvl={lvl}"
                        );
                        assert_eq!(
                            rlvl, clvl,
                            "WP_UseFirstValidSaberStyle level s1={s1:?} s2={s2:?} h={h} lvl={lvl}"
                        );
                    }
                }
            }
        }
    }
}
