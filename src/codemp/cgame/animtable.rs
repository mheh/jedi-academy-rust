//! `animTable` (`codemp/cgame/animtable.h`) -- the master string table pairing
//! every `animNumber_t` enumerator with its stringized name, used by the
//! `GetIDForString(animTable, value)` / `GetStringForID(animTable, id)` lookups
//! (e.g. the `VF_ANIM` case of the `.veh`/`.vwp` parsers in `bg_vehicleLoad.rs`,
//! and `PM_DebugLegsAnim` in `bg_panimate.rs`).
//!
//! This is the data companion to [`anims.rs`](super::super::game::anims) (the
//! `animNumber_t` enum): the two share the same ABI/asset-load-bearing numbering,
//! so the table rows are kept in the header's exact source order and every row is
//! oracle-verified against the authentic C `animTable[]` (the oracle TU
//! `oracle/animtable_oracle.c` `#include`s the real Raven header unmodified, so the
//! Rust transcription and the C compiler reading the header are independently
//! derived -- the same method as `anims.rs`).
//!
//! The C source file is `codemp/cgame/animtable.h`; in this MP-only port it has no
//! `cgame` module yet, but the `cgame/` directory mirrors the upstream tree so each
//! Rust file still maps 1:1 to its C origin. The build resolves to the non-Xbox,
//! non-UI definition branch (the `#else` of the `_XBOX && !_UI` guard), i.e. the
//! actual array (not the `extern` declaration). Generated faithfully from
//! `refs/raven-jediacademy/codemp/cgame/animtable.h`; all original section banners and
//! per-anim comments are carried.

#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

use crate::codemp::game::anims::{animNumber_t, MAX_ANIMATIONS, *};
use crate::codemp::game::q_shared_h::stringID_table_t;
use core::ffi::CStr;

/// Builds one `stringID_table_t` row from the C `ENUM2STRING(arg)` macro
/// (`#arg,arg`): the stringized enumerator name paired with its value. (The same
/// helper as `bg_vehicleLoad.rs`'s `VehicleTable`, specialized to `animNumber_t`.)
const fn enum2string(name: &'static CStr, id: animNumber_t) -> stringID_table_t {
    stringID_table_t {
        name: name.as_ptr(),
        id,
    }
}

/// `stringID_table_t animTable[MAX_ANIMATIONS+1]` (`cgame/animtable.h:9`). Non-const
/// in C (mirrored as `static mut`), but only ever read. The final row is the
/// `{ NULL, -1 }` terminator that bounds the `GetIDForString`/`GetStringForID` scan.
pub static mut animTable: [stringID_table_t; (MAX_ANIMATIONS + 1) as usize] = [
    //=================================================
    //HEAD ANIMS
    //=================================================
    //# #sep Head-only anims
    enum2string(c"FACE_TALK0", FACE_TALK0), // # silent
    enum2string(c"FACE_TALK1", FACE_TALK1), // # quiet
    enum2string(c"FACE_TALK2", FACE_TALK2), // # semi-quiet
    enum2string(c"FACE_TALK3", FACE_TALK3), // # semi-loud
    enum2string(c"FACE_TALK4", FACE_TALK4), // # loud
    enum2string(c"FACE_ALERT", FACE_ALERT), // #
    enum2string(c"FACE_SMILE", FACE_SMILE), // #
    enum2string(c"FACE_FROWN", FACE_FROWN), // #
    enum2string(c"FACE_DEAD", FACE_DEAD),   // #
    //=================================================
    //ANIMS IN WHICH UPPER AND LOWER OBJECTS ARE IN MD3
    //=================================================
    //# #sep ENUM2STRING(BOTH_ DEATHS
    enum2string(c"BOTH_DEATH1", BOTH_DEATH1), // # First Death anim
    enum2string(c"BOTH_DEATH2", BOTH_DEATH2), // # Second Death anim
    enum2string(c"BOTH_DEATH3", BOTH_DEATH3), // # Third Death anim
    enum2string(c"BOTH_DEATH4", BOTH_DEATH4), // # Fourth Death anim
    enum2string(c"BOTH_DEATH5", BOTH_DEATH5), // # Fifth Death anim
    enum2string(c"BOTH_DEATH6", BOTH_DEATH6), // # Sixth Death anim
    enum2string(c"BOTH_DEATH7", BOTH_DEATH7), // # Seventh Death anim
    enum2string(c"BOTH_DEATH8", BOTH_DEATH8), // #
    enum2string(c"BOTH_DEATH9", BOTH_DEATH9), // #
    enum2string(c"BOTH_DEATH10", BOTH_DEATH10), // #
    enum2string(c"BOTH_DEATH11", BOTH_DEATH11), // #
    enum2string(c"BOTH_DEATH12", BOTH_DEATH12), // #
    enum2string(c"BOTH_DEATH13", BOTH_DEATH13), // #
    enum2string(c"BOTH_DEATH14", BOTH_DEATH14), // #
    enum2string(c"BOTH_DEATH15", BOTH_DEATH15), // #
    enum2string(c"BOTH_DEATH16", BOTH_DEATH16), // #
    enum2string(c"BOTH_DEATH17", BOTH_DEATH17), // #
    enum2string(c"BOTH_DEATH18", BOTH_DEATH18), // #
    enum2string(c"BOTH_DEATH19", BOTH_DEATH19), // #
    enum2string(c"BOTH_DEATH20", BOTH_DEATH20), // #
    enum2string(c"BOTH_DEATH21", BOTH_DEATH21), // #
    enum2string(c"BOTH_DEATH22", BOTH_DEATH22), // #
    enum2string(c"BOTH_DEATH23", BOTH_DEATH23), // #
    enum2string(c"BOTH_DEATH24", BOTH_DEATH24), // #
    enum2string(c"BOTH_DEATH25", BOTH_DEATH25), // #
    enum2string(c"BOTH_DEATHFORWARD1", BOTH_DEATHFORWARD1), // # First Death in which they get thrown forward
    enum2string(c"BOTH_DEATHFORWARD2", BOTH_DEATHFORWARD2), // # Second Death in which they get thrown forward
    enum2string(c"BOTH_DEATHFORWARD3", BOTH_DEATHFORWARD3), // # Tavion's falling in cin# 23
    enum2string(c"BOTH_DEATHBACKWARD1", BOTH_DEATHBACKWARD1), // # First Death in which they get thrown backward
    enum2string(c"BOTH_DEATHBACKWARD2", BOTH_DEATHBACKWARD2), // # Second Death in which they get thrown backward
    enum2string(c"BOTH_DEATH1IDLE", BOTH_DEATH1IDLE),         // # Idle while close to death
    enum2string(c"BOTH_LYINGDEATH1", BOTH_LYINGDEATH1), // # Death to play when killed lying down
    enum2string(c"BOTH_STUMBLEDEATH1", BOTH_STUMBLEDEATH1), // # Stumble forward and fall face first death
    enum2string(c"BOTH_FALLDEATH1", BOTH_FALLDEATH1), // # Fall forward off a high cliff and splat death - start
    enum2string(c"BOTH_FALLDEATH1INAIR", BOTH_FALLDEATH1INAIR), // # Fall forward off a high cliff and splat death - loop
    enum2string(c"BOTH_FALLDEATH1LAND", BOTH_FALLDEATH1LAND), // # Fall forward off a high cliff and splat death - hit bottom
    enum2string(c"BOTH_DEATH_ROLL", BOTH_DEATH_ROLL),         // # Death anim from a roll
    enum2string(c"BOTH_DEATH_FLIP", BOTH_DEATH_FLIP),         // # Death anim from a flip
    enum2string(c"BOTH_DEATH_SPIN_90_R", BOTH_DEATH_SPIN_90_R), // # Death anim when facing 90 degrees right
    enum2string(c"BOTH_DEATH_SPIN_90_L", BOTH_DEATH_SPIN_90_L), // # Death anim when facing 90 degrees left
    enum2string(c"BOTH_DEATH_SPIN_180", BOTH_DEATH_SPIN_180), // # Death anim when facing backwards
    enum2string(c"BOTH_DEATH_LYING_UP", BOTH_DEATH_LYING_UP), // # Death anim when lying on back
    enum2string(c"BOTH_DEATH_LYING_DN", BOTH_DEATH_LYING_DN), // # Death anim when lying on front
    enum2string(c"BOTH_DEATH_FALLING_DN", BOTH_DEATH_FALLING_DN), // # Death anim when falling on face
    enum2string(c"BOTH_DEATH_FALLING_UP", BOTH_DEATH_FALLING_UP), // # Death anim when falling on back
    enum2string(c"BOTH_DEATH_CROUCHED", BOTH_DEATH_CROUCHED),     // # Death anim when crouched
    //# #sep ENUM2STRING(BOTH_ DEAD POSES # Should be last frame of corresponding previous anims
    enum2string(c"BOTH_DEAD1", BOTH_DEAD1), // # First Death finished pose
    enum2string(c"BOTH_DEAD2", BOTH_DEAD2), // # Second Death finished pose
    enum2string(c"BOTH_DEAD3", BOTH_DEAD3), // # Third Death finished pose
    enum2string(c"BOTH_DEAD4", BOTH_DEAD4), // # Fourth Death finished pose
    enum2string(c"BOTH_DEAD5", BOTH_DEAD5), // # Fifth Death finished pose
    enum2string(c"BOTH_DEAD6", BOTH_DEAD6), // # Sixth Death finished pose
    enum2string(c"BOTH_DEAD7", BOTH_DEAD7), // # Seventh Death finished pose
    enum2string(c"BOTH_DEAD8", BOTH_DEAD8), // #
    enum2string(c"BOTH_DEAD9", BOTH_DEAD9), // #
    enum2string(c"BOTH_DEAD10", BOTH_DEAD10), // #
    enum2string(c"BOTH_DEAD11", BOTH_DEAD11), // #
    enum2string(c"BOTH_DEAD12", BOTH_DEAD12), // #
    enum2string(c"BOTH_DEAD13", BOTH_DEAD13), // #
    enum2string(c"BOTH_DEAD14", BOTH_DEAD14), // #
    enum2string(c"BOTH_DEAD15", BOTH_DEAD15), // #
    enum2string(c"BOTH_DEAD16", BOTH_DEAD16), // #
    enum2string(c"BOTH_DEAD17", BOTH_DEAD17), // #
    enum2string(c"BOTH_DEAD18", BOTH_DEAD18), // #
    enum2string(c"BOTH_DEAD19", BOTH_DEAD19), // #
    enum2string(c"BOTH_DEAD20", BOTH_DEAD20), // #
    enum2string(c"BOTH_DEAD21", BOTH_DEAD21), // #
    enum2string(c"BOTH_DEAD22", BOTH_DEAD22), // #
    enum2string(c"BOTH_DEAD23", BOTH_DEAD23), // #
    enum2string(c"BOTH_DEAD24", BOTH_DEAD24), // #
    enum2string(c"BOTH_DEAD25", BOTH_DEAD25), // #
    enum2string(c"BOTH_DEADFORWARD1", BOTH_DEADFORWARD1), // # First thrown forward death finished pose
    enum2string(c"BOTH_DEADFORWARD2", BOTH_DEADFORWARD2), // # Second thrown forward death finished pose
    enum2string(c"BOTH_DEADBACKWARD1", BOTH_DEADBACKWARD1), // # First thrown backward death finished pose
    enum2string(c"BOTH_DEADBACKWARD2", BOTH_DEADBACKWARD2), // # Second thrown backward death finished pose
    enum2string(c"BOTH_LYINGDEAD1", BOTH_LYINGDEAD1), // # Killed lying down death finished pose
    enum2string(c"BOTH_STUMBLEDEAD1", BOTH_STUMBLEDEAD1), // # Stumble forward death finished pose
    enum2string(c"BOTH_FALLDEAD1LAND", BOTH_FALLDEAD1LAND), // # Fall forward and splat death finished pose
    //# #sep ENUM2STRING(BOTH_ DEAD TWITCH/FLOP # React to being shot from death poses
    enum2string(c"BOTH_DEADFLOP1", BOTH_DEADFLOP1), // # React to being shot from First Death finished pose
    enum2string(c"BOTH_DEADFLOP2", BOTH_DEADFLOP2), // # React to being shot from Second Death finished pose
    enum2string(c"BOTH_DISMEMBER_HEAD1", BOTH_DISMEMBER_HEAD1), // #
    enum2string(c"BOTH_DISMEMBER_TORSO1", BOTH_DISMEMBER_TORSO1), // #
    enum2string(c"BOTH_DISMEMBER_LLEG", BOTH_DISMEMBER_LLEG), // #
    enum2string(c"BOTH_DISMEMBER_RLEG", BOTH_DISMEMBER_RLEG), // #
    enum2string(c"BOTH_DISMEMBER_RARM", BOTH_DISMEMBER_RARM), // #
    enum2string(c"BOTH_DISMEMBER_LARM", BOTH_DISMEMBER_LARM), // #
    //# #sep ENUM2STRING(BOTH_ PAINS
    enum2string(c"BOTH_PAIN1", BOTH_PAIN1), // # First take pain anim
    enum2string(c"BOTH_PAIN2", BOTH_PAIN2), // # Second take pain anim
    enum2string(c"BOTH_PAIN3", BOTH_PAIN3), // # Third take pain anim
    enum2string(c"BOTH_PAIN4", BOTH_PAIN4), // # Fourth take pain anim
    enum2string(c"BOTH_PAIN5", BOTH_PAIN5), // # Fifth take pain anim - from behind
    enum2string(c"BOTH_PAIN6", BOTH_PAIN6), // # Sixth take pain anim - from behind
    enum2string(c"BOTH_PAIN7", BOTH_PAIN7), // # Seventh take pain anim - from behind
    enum2string(c"BOTH_PAIN8", BOTH_PAIN8), // # Eigth take pain anim - from behind
    enum2string(c"BOTH_PAIN9", BOTH_PAIN9), // #
    enum2string(c"BOTH_PAIN10", BOTH_PAIN10), // #
    enum2string(c"BOTH_PAIN11", BOTH_PAIN11), // #
    enum2string(c"BOTH_PAIN12", BOTH_PAIN12), // #
    enum2string(c"BOTH_PAIN13", BOTH_PAIN13), // #
    enum2string(c"BOTH_PAIN14", BOTH_PAIN14), // #
    enum2string(c"BOTH_PAIN15", BOTH_PAIN15), // #
    enum2string(c"BOTH_PAIN16", BOTH_PAIN16), // #
    enum2string(c"BOTH_PAIN17", BOTH_PAIN17), // #
    enum2string(c"BOTH_PAIN18", BOTH_PAIN18), // #
    //# #sep ENUM2STRING(BOTH_ ATTACKS
    enum2string(c"BOTH_ATTACK1", BOTH_ATTACK1), // # Attack with stun baton
    enum2string(c"BOTH_ATTACK2", BOTH_ATTACK2), // # Attack with one-handed pistol
    enum2string(c"BOTH_ATTACK3", BOTH_ATTACK3), // # Attack with blaster rifle
    enum2string(c"BOTH_ATTACK4", BOTH_ATTACK4), // # Attack with disruptor
    enum2string(c"BOTH_ATTACK5", BOTH_ATTACK5), // # Another Rancor Attack
    enum2string(c"BOTH_ATTACK6", BOTH_ATTACK6), // # Yet Another Rancor Attack
    enum2string(c"BOTH_ATTACK7", BOTH_ATTACK7), // # Yet Another Rancor Attack
    enum2string(c"BOTH_ATTACK10", BOTH_ATTACK10), // # Attack with thermal det
    enum2string(c"BOTH_ATTACK11", BOTH_ATTACK11), // # "Attack" with tripmine and detpack
    enum2string(c"BOTH_MELEE1", BOTH_MELEE1),   // # First melee attack
    enum2string(c"BOTH_MELEE2", BOTH_MELEE2),   // # Second melee attack
    enum2string(c"BOTH_THERMAL_READY", BOTH_THERMAL_READY), // # pull back with thermal
    enum2string(c"BOTH_THERMAL_THROW", BOTH_THERMAL_THROW), // # throw thermal
    //* #sep ENUM2STRING(BOTH_ SABER ANIMS
    //Saber attack anims - power level 1
    enum2string(c"BOTH_A1_T__B_", BOTH_A1_T__B_), // # Fast weak vertical attack top to bottom
    enum2string(c"BOTH_A1__L__R", BOTH_A1__L__R), // # Fast weak horizontal attack left to right
    enum2string(c"BOTH_A1__R__L", BOTH_A1__R__L), // # Fast weak horizontal attack right to left
    enum2string(c"BOTH_A1_TL_BR", BOTH_A1_TL_BR), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A1_BR_TL", BOTH_A1_BR_TL), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A1_BL_TR", BOTH_A1_BL_TR), // # Fast weak diagonal attack bottom left to top right
    enum2string(c"BOTH_A1_TR_BL", BOTH_A1_TR_BL), // # Fast weak diagonal attack bottom left to right
    //Saber Arc and Spin Transitions
    enum2string(c"BOTH_T1_BR__R", BOTH_T1_BR__R), // # Fast arc bottom right to right
    enum2string(c"BOTH_T1_BR_TL", BOTH_T1_BR_TL), // # Fast weak spin bottom right to top left
    enum2string(c"BOTH_T1_BR__L", BOTH_T1_BR__L), // # Fast weak spin bottom right to left
    enum2string(c"BOTH_T1_BR_BL", BOTH_T1_BR_BL), // # Fast weak spin bottom right to bottom left
    enum2string(c"BOTH_T1__R_TR", BOTH_T1__R_TR), // # Fast arc right to top right
    enum2string(c"BOTH_T1__R_TL", BOTH_T1__R_TL), // # Fast arc right to top left
    enum2string(c"BOTH_T1__R__L", BOTH_T1__R__L), // # Fast weak spin right to left
    enum2string(c"BOTH_T1__R_BL", BOTH_T1__R_BL), // # Fast weak spin right to bottom left
    enum2string(c"BOTH_T1_TR_BR", BOTH_T1_TR_BR), // # Fast arc top right to bottom right
    enum2string(c"BOTH_T1_TR_TL", BOTH_T1_TR_TL), // # Fast arc top right to top left
    enum2string(c"BOTH_T1_TR__L", BOTH_T1_TR__L), // # Fast arc top right to left
    enum2string(c"BOTH_T1_TR_BL", BOTH_T1_TR_BL), // # Fast weak spin top right to bottom left
    enum2string(c"BOTH_T1_T__BR", BOTH_T1_T__BR), // # Fast arc top to bottom right
    enum2string(c"BOTH_T1_T___R", BOTH_T1_T___R), // # Fast arc top to right
    enum2string(c"BOTH_T1_T__TR", BOTH_T1_T__TR), // # Fast arc top to top right
    enum2string(c"BOTH_T1_T__TL", BOTH_T1_T__TL), // # Fast arc top to top left
    enum2string(c"BOTH_T1_T___L", BOTH_T1_T___L), // # Fast arc top to left
    enum2string(c"BOTH_T1_T__BL", BOTH_T1_T__BL), // # Fast arc top to bottom left
    enum2string(c"BOTH_T1_TL_BR", BOTH_T1_TL_BR), // # Fast weak spin top left to bottom right
    enum2string(c"BOTH_T1_TL_BL", BOTH_T1_TL_BL), // # Fast arc top left to bottom left
    enum2string(c"BOTH_T1__L_BR", BOTH_T1__L_BR), // # Fast weak spin left to bottom right
    enum2string(c"BOTH_T1__L__R", BOTH_T1__L__R), // # Fast weak spin left to right
    enum2string(c"BOTH_T1__L_TL", BOTH_T1__L_TL), // # Fast arc left to top left
    enum2string(c"BOTH_T1_BL_BR", BOTH_T1_BL_BR), // # Fast weak spin bottom left to bottom right
    enum2string(c"BOTH_T1_BL__R", BOTH_T1_BL__R), // # Fast weak spin bottom left to right
    enum2string(c"BOTH_T1_BL_TR", BOTH_T1_BL_TR), // # Fast weak spin bottom left to top right
    enum2string(c"BOTH_T1_BL__L", BOTH_T1_BL__L), // # Fast arc bottom left to left
    //Saber Arc Transitions that use existing animations played backwards
    enum2string(c"BOTH_T1_BR_TR", BOTH_T1_BR_TR), // # Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T1_TR_BR)
    enum2string(c"BOTH_T1_BR_T_", BOTH_T1_BR_T_), // # Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T1_T__BR)
    enum2string(c"BOTH_T1__R_BR", BOTH_T1__R_BR), // # Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T1_BR__R)
    enum2string(c"BOTH_T1__R_T_", BOTH_T1__R_T_), // # Fast ar right to top				(use: ENUM2STRING(BOTH_T1_T___R)
    enum2string(c"BOTH_T1_TR__R", BOTH_T1_TR__R), // # Fast arc top right to right			(use: ENUM2STRING(BOTH_T1__R_TR)
    enum2string(c"BOTH_T1_TR_T_", BOTH_T1_TR_T_), // # Fast arc top right to top				(use: ENUM2STRING(BOTH_T1_T__TR)
    enum2string(c"BOTH_T1_TL__R", BOTH_T1_TL__R), // # Fast arc top left to right			(use: ENUM2STRING(BOTH_T1__R_TL)
    enum2string(c"BOTH_T1_TL_TR", BOTH_T1_TL_TR), // # Fast arc top left to top right			(use: ENUM2STRING(BOTH_T1_TR_TL)
    enum2string(c"BOTH_T1_TL_T_", BOTH_T1_TL_T_), // # Fast arc top left to top				(use: ENUM2STRING(BOTH_T1_T__TL)
    enum2string(c"BOTH_T1_TL__L", BOTH_T1_TL__L), // # Fast arc top left to left				(use: ENUM2STRING(BOTH_T1__L_TL)
    enum2string(c"BOTH_T1__L_TR", BOTH_T1__L_TR), // # Fast arc left to top right			(use: ENUM2STRING(BOTH_T1_TR__L)
    enum2string(c"BOTH_T1__L_T_", BOTH_T1__L_T_), // # Fast arc left to top				(use: ENUM2STRING(BOTH_T1_T___L)
    enum2string(c"BOTH_T1__L_BL", BOTH_T1__L_BL), // # Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T1_BL__L)
    enum2string(c"BOTH_T1_BL_T_", BOTH_T1_BL_T_), // # Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T1_T__BL)
    enum2string(c"BOTH_T1_BL_TL", BOTH_T1_BL_TL), // # Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T1_TL_BL)
    //Saber Attack Start Transitions
    enum2string(c"BOTH_S1_S1_T_", BOTH_S1_S1_T_), // # Fast plain transition from stance1 to top-to-bottom Fast weak attack
    enum2string(c"BOTH_S1_S1__L", BOTH_S1_S1__L), // # Fast plain transition from stance1 to left-to-right Fast weak attack
    enum2string(c"BOTH_S1_S1__R", BOTH_S1_S1__R), // # Fast plain transition from stance1 to right-to-left Fast weak attack
    enum2string(c"BOTH_S1_S1_TL", BOTH_S1_S1_TL), // # Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
    enum2string(c"BOTH_S1_S1_BR", BOTH_S1_S1_BR), // # Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
    enum2string(c"BOTH_S1_S1_BL", BOTH_S1_S1_BL), // # Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
    enum2string(c"BOTH_S1_S1_TR", BOTH_S1_S1_TR), // # Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
    //Saber Attack Return Transitions
    enum2string(c"BOTH_R1_B__S1", BOTH_R1_B__S1), // # Fast plain transition from top-to-bottom Fast weak attack to stance1
    enum2string(c"BOTH_R1__L_S1", BOTH_R1__L_S1), // # Fast plain transition from left-to-right Fast weak attack to stance1
    enum2string(c"BOTH_R1__R_S1", BOTH_R1__R_S1), // # Fast plain transition from right-to-left Fast weak attack to stance1
    enum2string(c"BOTH_R1_TL_S1", BOTH_R1_TL_S1), // # Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
    enum2string(c"BOTH_R1_BR_S1", BOTH_R1_BR_S1), // # Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
    enum2string(c"BOTH_R1_BL_S1", BOTH_R1_BL_S1), // # Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
    enum2string(c"BOTH_R1_TR_S1", BOTH_R1_TR_S1), // # Fast plain transition from top-right-to-bottom-left Fast weak attack
    //Saber Attack Bounces (first 4 frames of an attack), played backwards)
    enum2string(c"BOTH_B1_BR___", BOTH_B1_BR___), // # Bounce-back if attack from BR is blocked
    enum2string(c"BOTH_B1__R___", BOTH_B1__R___), // # Bounce-back if attack from R is blocked
    enum2string(c"BOTH_B1_TR___", BOTH_B1_TR___), // # Bounce-back if attack from TR is blocked
    enum2string(c"BOTH_B1_T____", BOTH_B1_T____), // # Bounce-back if attack from T is blocked
    enum2string(c"BOTH_B1_TL___", BOTH_B1_TL___), // # Bounce-back if attack from TL is blocked
    enum2string(c"BOTH_B1__L___", BOTH_B1__L___), // # Bounce-back if attack from L is blocked
    enum2string(c"BOTH_B1_BL___", BOTH_B1_BL___), // # Bounce-back if attack from BL is blocked
    //Saber Attack Deflections (last 4 frames of an attack)
    enum2string(c"BOTH_D1_BR___", BOTH_D1_BR___), // # Deflection toward BR
    enum2string(c"BOTH_D1__R___", BOTH_D1__R___), // # Deflection toward R
    enum2string(c"BOTH_D1_TR___", BOTH_D1_TR___), // # Deflection toward TR
    enum2string(c"BOTH_D1_TL___", BOTH_D1_TL___), // # Deflection toward TL
    enum2string(c"BOTH_D1__L___", BOTH_D1__L___), // # Deflection toward L
    enum2string(c"BOTH_D1_BL___", BOTH_D1_BL___), // # Deflection toward BL
    enum2string(c"BOTH_D1_B____", BOTH_D1_B____), // # Deflection toward B
    //Saber attack anims - power level 2
    enum2string(c"BOTH_A2_T__B_", BOTH_A2_T__B_), // # Fast weak vertical attack top to bottom
    enum2string(c"BOTH_A2__L__R", BOTH_A2__L__R), // # Fast weak horizontal attack left to right
    enum2string(c"BOTH_A2__R__L", BOTH_A2__R__L), // # Fast weak horizontal attack right to left
    enum2string(c"BOTH_A2_TL_BR", BOTH_A2_TL_BR), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A2_BR_TL", BOTH_A2_BR_TL), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A2_BL_TR", BOTH_A2_BL_TR), // # Fast weak diagonal attack bottom left to top right
    enum2string(c"BOTH_A2_TR_BL", BOTH_A2_TR_BL), // # Fast weak diagonal attack bottom left to right
    //Saber Arc and Spin Transitions
    enum2string(c"BOTH_T2_BR__R", BOTH_T2_BR__R), // # Fast arc bottom right to right
    enum2string(c"BOTH_T2_BR_TL", BOTH_T2_BR_TL), // # Fast weak spin bottom right to top left
    enum2string(c"BOTH_T2_BR__L", BOTH_T2_BR__L), // # Fast weak spin bottom right to left
    enum2string(c"BOTH_T2_BR_BL", BOTH_T2_BR_BL), // # Fast weak spin bottom right to bottom left
    enum2string(c"BOTH_T2__R_TR", BOTH_T2__R_TR), // # Fast arc right to top right
    enum2string(c"BOTH_T2__R_TL", BOTH_T2__R_TL), // # Fast arc right to top left
    enum2string(c"BOTH_T2__R__L", BOTH_T2__R__L), // # Fast weak spin right to left
    enum2string(c"BOTH_T2__R_BL", BOTH_T2__R_BL), // # Fast weak spin right to bottom left
    enum2string(c"BOTH_T2_TR_BR", BOTH_T2_TR_BR), // # Fast arc top right to bottom right
    enum2string(c"BOTH_T2_TR_TL", BOTH_T2_TR_TL), // # Fast arc top right to top left
    enum2string(c"BOTH_T2_TR__L", BOTH_T2_TR__L), // # Fast arc top right to left
    enum2string(c"BOTH_T2_TR_BL", BOTH_T2_TR_BL), // # Fast weak spin top right to bottom left
    enum2string(c"BOTH_T2_T__BR", BOTH_T2_T__BR), // # Fast arc top to bottom right
    enum2string(c"BOTH_T2_T___R", BOTH_T2_T___R), // # Fast arc top to right
    enum2string(c"BOTH_T2_T__TR", BOTH_T2_T__TR), // # Fast arc top to top right
    enum2string(c"BOTH_T2_T__TL", BOTH_T2_T__TL), // # Fast arc top to top left
    enum2string(c"BOTH_T2_T___L", BOTH_T2_T___L), // # Fast arc top to left
    enum2string(c"BOTH_T2_T__BL", BOTH_T2_T__BL), // # Fast arc top to bottom left
    enum2string(c"BOTH_T2_TL_BR", BOTH_T2_TL_BR), // # Fast weak spin top left to bottom right
    enum2string(c"BOTH_T2_TL_BL", BOTH_T2_TL_BL), // # Fast arc top left to bottom left
    enum2string(c"BOTH_T2__L_BR", BOTH_T2__L_BR), // # Fast weak spin left to bottom right
    enum2string(c"BOTH_T2__L__R", BOTH_T2__L__R), // # Fast weak spin left to right
    enum2string(c"BOTH_T2__L_TL", BOTH_T2__L_TL), // # Fast arc left to top left
    enum2string(c"BOTH_T2_BL_BR", BOTH_T2_BL_BR), // # Fast weak spin bottom left to bottom right
    enum2string(c"BOTH_T2_BL__R", BOTH_T2_BL__R), // # Fast weak spin bottom left to right
    enum2string(c"BOTH_T2_BL_TR", BOTH_T2_BL_TR), // # Fast weak spin bottom left to top right
    enum2string(c"BOTH_T2_BL__L", BOTH_T2_BL__L), // # Fast arc bottom left to left
    //Saber Arc Transitions that use existing animations played backwards
    enum2string(c"BOTH_T2_BR_TR", BOTH_T2_BR_TR), // # Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T2_TR_BR)
    enum2string(c"BOTH_T2_BR_T_", BOTH_T2_BR_T_), // # Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T2_T__BR)
    enum2string(c"BOTH_T2__R_BR", BOTH_T2__R_BR), // # Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T2_BR__R)
    enum2string(c"BOTH_T2__R_T_", BOTH_T2__R_T_), // # Fast ar right to top				(use: ENUM2STRING(BOTH_T2_T___R)
    enum2string(c"BOTH_T2_TR__R", BOTH_T2_TR__R), // # Fast arc top right to right			(use: ENUM2STRING(BOTH_T2__R_TR)
    enum2string(c"BOTH_T2_TR_T_", BOTH_T2_TR_T_), // # Fast arc top right to top				(use: ENUM2STRING(BOTH_T2_T__TR)
    enum2string(c"BOTH_T2_TL__R", BOTH_T2_TL__R), // # Fast arc top left to right			(use: ENUM2STRING(BOTH_T2__R_TL)
    enum2string(c"BOTH_T2_TL_TR", BOTH_T2_TL_TR), // # Fast arc top left to top right			(use: ENUM2STRING(BOTH_T2_TR_TL)
    enum2string(c"BOTH_T2_TL_T_", BOTH_T2_TL_T_), // # Fast arc top left to top				(use: ENUM2STRING(BOTH_T2_T__TL)
    enum2string(c"BOTH_T2_TL__L", BOTH_T2_TL__L), // # Fast arc top left to left				(use: ENUM2STRING(BOTH_T2__L_TL)
    enum2string(c"BOTH_T2__L_TR", BOTH_T2__L_TR), // # Fast arc left to top right			(use: ENUM2STRING(BOTH_T2_TR__L)
    enum2string(c"BOTH_T2__L_T_", BOTH_T2__L_T_), // # Fast arc left to top				(use: ENUM2STRING(BOTH_T2_T___L)
    enum2string(c"BOTH_T2__L_BL", BOTH_T2__L_BL), // # Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T2_BL__L)
    enum2string(c"BOTH_T2_BL_T_", BOTH_T2_BL_T_), // # Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T2_T__BL)
    enum2string(c"BOTH_T2_BL_TL", BOTH_T2_BL_TL), // # Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T2_TL_BL)
    //Saber Attack Start Transitions
    enum2string(c"BOTH_S2_S1_T_", BOTH_S2_S1_T_), // # Fast plain transition from stance1 to top-to-bottom Fast weak attack
    enum2string(c"BOTH_S2_S1__L", BOTH_S2_S1__L), // # Fast plain transition from stance1 to left-to-right Fast weak attack
    enum2string(c"BOTH_S2_S1__R", BOTH_S2_S1__R), // # Fast plain transition from stance1 to right-to-left Fast weak attack
    enum2string(c"BOTH_S2_S1_TL", BOTH_S2_S1_TL), // # Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
    enum2string(c"BOTH_S2_S1_BR", BOTH_S2_S1_BR), // # Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
    enum2string(c"BOTH_S2_S1_BL", BOTH_S2_S1_BL), // # Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
    enum2string(c"BOTH_S2_S1_TR", BOTH_S2_S1_TR), // # Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
    //Saber Attack Return Transitions
    enum2string(c"BOTH_R2_B__S1", BOTH_R2_B__S1), // # Fast plain transition from top-to-bottom Fast weak attack to stance1
    enum2string(c"BOTH_R2__L_S1", BOTH_R2__L_S1), // # Fast plain transition from left-to-right Fast weak attack to stance1
    enum2string(c"BOTH_R2__R_S1", BOTH_R2__R_S1), // # Fast plain transition from right-to-left Fast weak attack to stance1
    enum2string(c"BOTH_R2_TL_S1", BOTH_R2_TL_S1), // # Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
    enum2string(c"BOTH_R2_BR_S1", BOTH_R2_BR_S1), // # Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
    enum2string(c"BOTH_R2_BL_S1", BOTH_R2_BL_S1), // # Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
    enum2string(c"BOTH_R2_TR_S1", BOTH_R2_TR_S1), // # Fast plain transition from top-right-to-bottom-left Fast weak attack
    //Saber Attack Bounces (first 4 frames of an attack), played backwards)
    enum2string(c"BOTH_B2_BR___", BOTH_B2_BR___), // # Bounce-back if attack from BR is blocked
    enum2string(c"BOTH_B2__R___", BOTH_B2__R___), // # Bounce-back if attack from R is blocked
    enum2string(c"BOTH_B2_TR___", BOTH_B2_TR___), // # Bounce-back if attack from TR is blocked
    enum2string(c"BOTH_B2_T____", BOTH_B2_T____), // # Bounce-back if attack from T is blocked
    enum2string(c"BOTH_B2_TL___", BOTH_B2_TL___), // # Bounce-back if attack from TL is blocked
    enum2string(c"BOTH_B2__L___", BOTH_B2__L___), // # Bounce-back if attack from L is blocked
    enum2string(c"BOTH_B2_BL___", BOTH_B2_BL___), // # Bounce-back if attack from BL is blocked
    //Saber Attack Deflections (last 4 frames of an attack)
    enum2string(c"BOTH_D2_BR___", BOTH_D2_BR___), // # Deflection toward BR
    enum2string(c"BOTH_D2__R___", BOTH_D2__R___), // # Deflection toward R
    enum2string(c"BOTH_D2_TR___", BOTH_D2_TR___), // # Deflection toward TR
    enum2string(c"BOTH_D2_TL___", BOTH_D2_TL___), // # Deflection toward TL
    enum2string(c"BOTH_D2__L___", BOTH_D2__L___), // # Deflection toward L
    enum2string(c"BOTH_D2_BL___", BOTH_D2_BL___), // # Deflection toward BL
    enum2string(c"BOTH_D2_B____", BOTH_D2_B____), // # Deflection toward B
    //Saber attack anims - power level 3
    enum2string(c"BOTH_A3_T__B_", BOTH_A3_T__B_), // # Fast weak vertical attack top to bottom
    enum2string(c"BOTH_A3__L__R", BOTH_A3__L__R), // # Fast weak horizontal attack left to right
    enum2string(c"BOTH_A3__R__L", BOTH_A3__R__L), // # Fast weak horizontal attack right to left
    enum2string(c"BOTH_A3_TL_BR", BOTH_A3_TL_BR), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A3_BR_TL", BOTH_A3_BR_TL), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A3_BL_TR", BOTH_A3_BL_TR), // # Fast weak diagonal attack bottom left to top right
    enum2string(c"BOTH_A3_TR_BL", BOTH_A3_TR_BL), // # Fast weak diagonal attack bottom left to right
    //Saber Arc and Spin Transitions
    enum2string(c"BOTH_T3_BR__R", BOTH_T3_BR__R), // # Fast arc bottom right to right
    enum2string(c"BOTH_T3_BR_TL", BOTH_T3_BR_TL), // # Fast weak spin bottom right to top left
    enum2string(c"BOTH_T3_BR__L", BOTH_T3_BR__L), // # Fast weak spin bottom right to left
    enum2string(c"BOTH_T3_BR_BL", BOTH_T3_BR_BL), // # Fast weak spin bottom right to bottom left
    enum2string(c"BOTH_T3__R_TR", BOTH_T3__R_TR), // # Fast arc right to top right
    enum2string(c"BOTH_T3__R_TL", BOTH_T3__R_TL), // # Fast arc right to top left
    enum2string(c"BOTH_T3__R__L", BOTH_T3__R__L), // # Fast weak spin right to left
    enum2string(c"BOTH_T3__R_BL", BOTH_T3__R_BL), // # Fast weak spin right to bottom left
    enum2string(c"BOTH_T3_TR_BR", BOTH_T3_TR_BR), // # Fast arc top right to bottom right
    enum2string(c"BOTH_T3_TR_TL", BOTH_T3_TR_TL), // # Fast arc top right to top left
    enum2string(c"BOTH_T3_TR__L", BOTH_T3_TR__L), // # Fast arc top right to left
    enum2string(c"BOTH_T3_TR_BL", BOTH_T3_TR_BL), // # Fast weak spin top right to bottom left
    enum2string(c"BOTH_T3_T__BR", BOTH_T3_T__BR), // # Fast arc top to bottom right
    enum2string(c"BOTH_T3_T___R", BOTH_T3_T___R), // # Fast arc top to right
    enum2string(c"BOTH_T3_T__TR", BOTH_T3_T__TR), // # Fast arc top to top right
    enum2string(c"BOTH_T3_T__TL", BOTH_T3_T__TL), // # Fast arc top to top left
    enum2string(c"BOTH_T3_T___L", BOTH_T3_T___L), // # Fast arc top to left
    enum2string(c"BOTH_T3_T__BL", BOTH_T3_T__BL), // # Fast arc top to bottom left
    enum2string(c"BOTH_T3_TL_BR", BOTH_T3_TL_BR), // # Fast weak spin top left to bottom right
    enum2string(c"BOTH_T3_TL_BL", BOTH_T3_TL_BL), // # Fast arc top left to bottom left
    enum2string(c"BOTH_T3__L_BR", BOTH_T3__L_BR), // # Fast weak spin left to bottom right
    enum2string(c"BOTH_T3__L__R", BOTH_T3__L__R), // # Fast weak spin left to right
    enum2string(c"BOTH_T3__L_TL", BOTH_T3__L_TL), // # Fast arc left to top left
    enum2string(c"BOTH_T3_BL_BR", BOTH_T3_BL_BR), // # Fast weak spin bottom left to bottom right
    enum2string(c"BOTH_T3_BL__R", BOTH_T3_BL__R), // # Fast weak spin bottom left to right
    enum2string(c"BOTH_T3_BL_TR", BOTH_T3_BL_TR), // # Fast weak spin bottom left to top right
    enum2string(c"BOTH_T3_BL__L", BOTH_T3_BL__L), // # Fast arc bottom left to left
    //Saber Arc Transitions that use existing animations played backwards
    enum2string(c"BOTH_T3_BR_TR", BOTH_T3_BR_TR), // # Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T3_TR_BR)
    enum2string(c"BOTH_T3_BR_T_", BOTH_T3_BR_T_), // # Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T3_T__BR)
    enum2string(c"BOTH_T3__R_BR", BOTH_T3__R_BR), // # Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T3_BR__R)
    enum2string(c"BOTH_T3__R_T_", BOTH_T3__R_T_), // # Fast ar right to top				(use: ENUM2STRING(BOTH_T3_T___R)
    enum2string(c"BOTH_T3_TR__R", BOTH_T3_TR__R), // # Fast arc top right to right			(use: ENUM2STRING(BOTH_T3__R_TR)
    enum2string(c"BOTH_T3_TR_T_", BOTH_T3_TR_T_), // # Fast arc top right to top				(use: ENUM2STRING(BOTH_T3_T__TR)
    enum2string(c"BOTH_T3_TL__R", BOTH_T3_TL__R), // # Fast arc top left to right			(use: ENUM2STRING(BOTH_T3__R_TL)
    enum2string(c"BOTH_T3_TL_TR", BOTH_T3_TL_TR), // # Fast arc top left to top right			(use: ENUM2STRING(BOTH_T3_TR_TL)
    enum2string(c"BOTH_T3_TL_T_", BOTH_T3_TL_T_), // # Fast arc top left to top				(use: ENUM2STRING(BOTH_T3_T__TL)
    enum2string(c"BOTH_T3_TL__L", BOTH_T3_TL__L), // # Fast arc top left to left				(use: ENUM2STRING(BOTH_T3__L_TL)
    enum2string(c"BOTH_T3__L_TR", BOTH_T3__L_TR), // # Fast arc left to top right			(use: ENUM2STRING(BOTH_T3_TR__L)
    enum2string(c"BOTH_T3__L_T_", BOTH_T3__L_T_), // # Fast arc left to top				(use: ENUM2STRING(BOTH_T3_T___L)
    enum2string(c"BOTH_T3__L_BL", BOTH_T3__L_BL), // # Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T3_BL__L)
    enum2string(c"BOTH_T3_BL_T_", BOTH_T3_BL_T_), // # Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T3_T__BL)
    enum2string(c"BOTH_T3_BL_TL", BOTH_T3_BL_TL), // # Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T3_TL_BL)
    //Saber Attack Start Transitions
    enum2string(c"BOTH_S3_S1_T_", BOTH_S3_S1_T_), // # Fast plain transition from stance1 to top-to-bottom Fast weak attack
    enum2string(c"BOTH_S3_S1__L", BOTH_S3_S1__L), // # Fast plain transition from stance1 to left-to-right Fast weak attack
    enum2string(c"BOTH_S3_S1__R", BOTH_S3_S1__R), // # Fast plain transition from stance1 to right-to-left Fast weak attack
    enum2string(c"BOTH_S3_S1_TL", BOTH_S3_S1_TL), // # Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
    enum2string(c"BOTH_S3_S1_BR", BOTH_S3_S1_BR), // # Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
    enum2string(c"BOTH_S3_S1_BL", BOTH_S3_S1_BL), // # Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
    enum2string(c"BOTH_S3_S1_TR", BOTH_S3_S1_TR), // # Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
    //Saber Attack Return Transitions
    enum2string(c"BOTH_R3_B__S1", BOTH_R3_B__S1), // # Fast plain transition from top-to-bottom Fast weak attack to stance1
    enum2string(c"BOTH_R3__L_S1", BOTH_R3__L_S1), // # Fast plain transition from left-to-right Fast weak attack to stance1
    enum2string(c"BOTH_R3__R_S1", BOTH_R3__R_S1), // # Fast plain transition from right-to-left Fast weak attack to stance1
    enum2string(c"BOTH_R3_TL_S1", BOTH_R3_TL_S1), // # Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
    enum2string(c"BOTH_R3_BR_S1", BOTH_R3_BR_S1), // # Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
    enum2string(c"BOTH_R3_BL_S1", BOTH_R3_BL_S1), // # Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
    enum2string(c"BOTH_R3_TR_S1", BOTH_R3_TR_S1), // # Fast plain transition from top-right-to-bottom-left Fast weak attack
    //Saber Attack Bounces (first 4 frames of an attack), played backwards)
    enum2string(c"BOTH_B3_BR___", BOTH_B3_BR___), // # Bounce-back if attack from BR is blocked
    enum2string(c"BOTH_B3__R___", BOTH_B3__R___), // # Bounce-back if attack from R is blocked
    enum2string(c"BOTH_B3_TR___", BOTH_B3_TR___), // # Bounce-back if attack from TR is blocked
    enum2string(c"BOTH_B3_T____", BOTH_B3_T____), // # Bounce-back if attack from T is blocked
    enum2string(c"BOTH_B3_TL___", BOTH_B3_TL___), // # Bounce-back if attack from TL is blocked
    enum2string(c"BOTH_B3__L___", BOTH_B3__L___), // # Bounce-back if attack from L is blocked
    enum2string(c"BOTH_B3_BL___", BOTH_B3_BL___), // # Bounce-back if attack from BL is blocked
    //Saber Attack Deflections (last 4 frames of an attack)
    enum2string(c"BOTH_D3_BR___", BOTH_D3_BR___), // # Deflection toward BR
    enum2string(c"BOTH_D3__R___", BOTH_D3__R___), // # Deflection toward R
    enum2string(c"BOTH_D3_TR___", BOTH_D3_TR___), // # Deflection toward TR
    enum2string(c"BOTH_D3_TL___", BOTH_D3_TL___), // # Deflection toward TL
    enum2string(c"BOTH_D3__L___", BOTH_D3__L___), // # Deflection toward L
    enum2string(c"BOTH_D3_BL___", BOTH_D3_BL___), // # Deflection toward BL
    enum2string(c"BOTH_D3_B____", BOTH_D3_B____), // # Deflection toward B
    //Saber attack anims - power level 4 - Desann's
    enum2string(c"BOTH_A4_T__B_", BOTH_A4_T__B_), // # Fast weak vertical attack top to bottom
    enum2string(c"BOTH_A4__L__R", BOTH_A4__L__R), // # Fast weak horizontal attack left to right
    enum2string(c"BOTH_A4__R__L", BOTH_A4__R__L), // # Fast weak horizontal attack right to left
    enum2string(c"BOTH_A4_TL_BR", BOTH_A4_TL_BR), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A4_BR_TL", BOTH_A4_BR_TL), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A4_BL_TR", BOTH_A4_BL_TR), // # Fast weak diagonal attack bottom left to top right
    enum2string(c"BOTH_A4_TR_BL", BOTH_A4_TR_BL), // # Fast weak diagonal attack bottom left to right
    //Saber Arc and Spin Transitions
    enum2string(c"BOTH_T4_BR__R", BOTH_T4_BR__R), // # Fast arc bottom right to right
    enum2string(c"BOTH_T4_BR_TL", BOTH_T4_BR_TL), // # Fast weak spin bottom right to top left
    enum2string(c"BOTH_T4_BR__L", BOTH_T4_BR__L), // # Fast weak spin bottom right to left
    enum2string(c"BOTH_T4_BR_BL", BOTH_T4_BR_BL), // # Fast weak spin bottom right to bottom left
    enum2string(c"BOTH_T4__R_TR", BOTH_T4__R_TR), // # Fast arc right to top right
    enum2string(c"BOTH_T4__R_TL", BOTH_T4__R_TL), // # Fast arc right to top left
    enum2string(c"BOTH_T4__R__L", BOTH_T4__R__L), // # Fast weak spin right to left
    enum2string(c"BOTH_T4__R_BL", BOTH_T4__R_BL), // # Fast weak spin right to bottom left
    enum2string(c"BOTH_T4_TR_BR", BOTH_T4_TR_BR), // # Fast arc top right to bottom right
    enum2string(c"BOTH_T4_TR_TL", BOTH_T4_TR_TL), // # Fast arc top right to top left
    enum2string(c"BOTH_T4_TR__L", BOTH_T4_TR__L), // # Fast arc top right to left
    enum2string(c"BOTH_T4_TR_BL", BOTH_T4_TR_BL), // # Fast weak spin top right to bottom left
    enum2string(c"BOTH_T4_T__BR", BOTH_T4_T__BR), // # Fast arc top to bottom right
    enum2string(c"BOTH_T4_T___R", BOTH_T4_T___R), // # Fast arc top to right
    enum2string(c"BOTH_T4_T__TR", BOTH_T4_T__TR), // # Fast arc top to top right
    enum2string(c"BOTH_T4_T__TL", BOTH_T4_T__TL), // # Fast arc top to top left
    enum2string(c"BOTH_T4_T___L", BOTH_T4_T___L), // # Fast arc top to left
    enum2string(c"BOTH_T4_T__BL", BOTH_T4_T__BL), // # Fast arc top to bottom left
    enum2string(c"BOTH_T4_TL_BR", BOTH_T4_TL_BR), // # Fast weak spin top left to bottom right
    enum2string(c"BOTH_T4_TL_BL", BOTH_T4_TL_BL), // # Fast arc top left to bottom left
    enum2string(c"BOTH_T4__L_BR", BOTH_T4__L_BR), // # Fast weak spin left to bottom right
    enum2string(c"BOTH_T4__L__R", BOTH_T4__L__R), // # Fast weak spin left to right
    enum2string(c"BOTH_T4__L_TL", BOTH_T4__L_TL), // # Fast arc left to top left
    enum2string(c"BOTH_T4_BL_BR", BOTH_T4_BL_BR), // # Fast weak spin bottom left to bottom right
    enum2string(c"BOTH_T4_BL__R", BOTH_T4_BL__R), // # Fast weak spin bottom left to right
    enum2string(c"BOTH_T4_BL_TR", BOTH_T4_BL_TR), // # Fast weak spin bottom left to top right
    enum2string(c"BOTH_T4_BL__L", BOTH_T4_BL__L), // # Fast arc bottom left to left
    //Saber Arc Transitions that use existing animations played backwards
    enum2string(c"BOTH_T4_BR_TR", BOTH_T4_BR_TR), // # Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T4_TR_BR)
    enum2string(c"BOTH_T4_BR_T_", BOTH_T4_BR_T_), // # Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T4_T__BR)
    enum2string(c"BOTH_T4__R_BR", BOTH_T4__R_BR), // # Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T4_BR__R)
    enum2string(c"BOTH_T4__R_T_", BOTH_T4__R_T_), // # Fast ar right to top				(use: ENUM2STRING(BOTH_T4_T___R)
    enum2string(c"BOTH_T4_TR__R", BOTH_T4_TR__R), // # Fast arc top right to right			(use: ENUM2STRING(BOTH_T4__R_TR)
    enum2string(c"BOTH_T4_TR_T_", BOTH_T4_TR_T_), // # Fast arc top right to top				(use: ENUM2STRING(BOTH_T4_T__TR)
    enum2string(c"BOTH_T4_TL__R", BOTH_T4_TL__R), // # Fast arc top left to right			(use: ENUM2STRING(BOTH_T4__R_TL)
    enum2string(c"BOTH_T4_TL_TR", BOTH_T4_TL_TR), // # Fast arc top left to top right			(use: ENUM2STRING(BOTH_T4_TR_TL)
    enum2string(c"BOTH_T4_TL_T_", BOTH_T4_TL_T_), // # Fast arc top left to top				(use: ENUM2STRING(BOTH_T4_T__TL)
    enum2string(c"BOTH_T4_TL__L", BOTH_T4_TL__L), // # Fast arc top left to left				(use: ENUM2STRING(BOTH_T4__L_TL)
    enum2string(c"BOTH_T4__L_TR", BOTH_T4__L_TR), // # Fast arc left to top right			(use: ENUM2STRING(BOTH_T4_TR__L)
    enum2string(c"BOTH_T4__L_T_", BOTH_T4__L_T_), // # Fast arc left to top				(use: ENUM2STRING(BOTH_T4_T___L)
    enum2string(c"BOTH_T4__L_BL", BOTH_T4__L_BL), // # Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T4_BL__L)
    enum2string(c"BOTH_T4_BL_T_", BOTH_T4_BL_T_), // # Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T4_T__BL)
    enum2string(c"BOTH_T4_BL_TL", BOTH_T4_BL_TL), // # Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T4_TL_BL)
    //Saber Attack Start Transitions
    enum2string(c"BOTH_S4_S1_T_", BOTH_S4_S1_T_), // # Fast plain transition from stance1 to top-to-bottom Fast weak attack
    enum2string(c"BOTH_S4_S1__L", BOTH_S4_S1__L), // # Fast plain transition from stance1 to left-to-right Fast weak attack
    enum2string(c"BOTH_S4_S1__R", BOTH_S4_S1__R), // # Fast plain transition from stance1 to right-to-left Fast weak attack
    enum2string(c"BOTH_S4_S1_TL", BOTH_S4_S1_TL), // # Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
    enum2string(c"BOTH_S4_S1_BR", BOTH_S4_S1_BR), // # Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
    enum2string(c"BOTH_S4_S1_BL", BOTH_S4_S1_BL), // # Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
    enum2string(c"BOTH_S4_S1_TR", BOTH_S4_S1_TR), // # Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
    //Saber Attack Return Transitions
    enum2string(c"BOTH_R4_B__S1", BOTH_R4_B__S1), // # Fast plain transition from top-to-bottom Fast weak attack to stance1
    enum2string(c"BOTH_R4__L_S1", BOTH_R4__L_S1), // # Fast plain transition from left-to-right Fast weak attack to stance1
    enum2string(c"BOTH_R4__R_S1", BOTH_R4__R_S1), // # Fast plain transition from right-to-left Fast weak attack to stance1
    enum2string(c"BOTH_R4_TL_S1", BOTH_R4_TL_S1), // # Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
    enum2string(c"BOTH_R4_BR_S1", BOTH_R4_BR_S1), // # Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
    enum2string(c"BOTH_R4_BL_S1", BOTH_R4_BL_S1), // # Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
    enum2string(c"BOTH_R4_TR_S1", BOTH_R4_TR_S1), // # Fast plain transition from top-right-to-bottom-left Fast weak attack
    //Saber Attack Bounces (first 4 frames of an attack), played backwards)
    enum2string(c"BOTH_B4_BR___", BOTH_B4_BR___), // # Bounce-back if attack from BR is blocked
    enum2string(c"BOTH_B4__R___", BOTH_B4__R___), // # Bounce-back if attack from R is blocked
    enum2string(c"BOTH_B4_TR___", BOTH_B4_TR___), // # Bounce-back if attack from TR is blocked
    enum2string(c"BOTH_B4_T____", BOTH_B4_T____), // # Bounce-back if attack from T is blocked
    enum2string(c"BOTH_B4_TL___", BOTH_B4_TL___), // # Bounce-back if attack from TL is blocked
    enum2string(c"BOTH_B4__L___", BOTH_B4__L___), // # Bounce-back if attack from L is blocked
    enum2string(c"BOTH_B4_BL___", BOTH_B4_BL___), // # Bounce-back if attack from BL is blocked
    //Saber Attack Deflections (last 4 frames of an attack)
    enum2string(c"BOTH_D4_BR___", BOTH_D4_BR___), // # Deflection toward BR
    enum2string(c"BOTH_D4__R___", BOTH_D4__R___), // # Deflection toward R
    enum2string(c"BOTH_D4_TR___", BOTH_D4_TR___), // # Deflection toward TR
    enum2string(c"BOTH_D4_TL___", BOTH_D4_TL___), // # Deflection toward TL
    enum2string(c"BOTH_D4__L___", BOTH_D4__L___), // # Deflection toward L
    enum2string(c"BOTH_D4_BL___", BOTH_D4_BL___), // # Deflection toward BL
    enum2string(c"BOTH_D4_B____", BOTH_D4_B____), // # Deflection toward B
    //Saber attack anims - power level 5 - Tavion's
    enum2string(c"BOTH_A5_T__B_", BOTH_A5_T__B_), // # Fast weak vertical attack top to bottom
    enum2string(c"BOTH_A5__L__R", BOTH_A5__L__R), // # Fast weak horizontal attack left to right
    enum2string(c"BOTH_A5__R__L", BOTH_A5__R__L), // # Fast weak horizontal attack right to left
    enum2string(c"BOTH_A5_TL_BR", BOTH_A5_TL_BR), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A5_BR_TL", BOTH_A5_BR_TL), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A5_BL_TR", BOTH_A5_BL_TR), // # Fast weak diagonal attack bottom left to top right
    enum2string(c"BOTH_A5_TR_BL", BOTH_A5_TR_BL), // # Fast weak diagonal attack bottom left to right
    //Saber Arc and Spin Transitions
    enum2string(c"BOTH_T5_BR__R", BOTH_T5_BR__R), // # Fast arc bottom right to right
    enum2string(c"BOTH_T5_BR_TL", BOTH_T5_BR_TL), // # Fast weak spin bottom right to top left
    enum2string(c"BOTH_T5_BR__L", BOTH_T5_BR__L), // # Fast weak spin bottom right to left
    enum2string(c"BOTH_T5_BR_BL", BOTH_T5_BR_BL), // # Fast weak spin bottom right to bottom left
    enum2string(c"BOTH_T5__R_TR", BOTH_T5__R_TR), // # Fast arc right to top right
    enum2string(c"BOTH_T5__R_TL", BOTH_T5__R_TL), // # Fast arc right to top left
    enum2string(c"BOTH_T5__R__L", BOTH_T5__R__L), // # Fast weak spin right to left
    enum2string(c"BOTH_T5__R_BL", BOTH_T5__R_BL), // # Fast weak spin right to bottom left
    enum2string(c"BOTH_T5_TR_BR", BOTH_T5_TR_BR), // # Fast arc top right to bottom right
    enum2string(c"BOTH_T5_TR_TL", BOTH_T5_TR_TL), // # Fast arc top right to top left
    enum2string(c"BOTH_T5_TR__L", BOTH_T5_TR__L), // # Fast arc top right to left
    enum2string(c"BOTH_T5_TR_BL", BOTH_T5_TR_BL), // # Fast weak spin top right to bottom left
    enum2string(c"BOTH_T5_T__BR", BOTH_T5_T__BR), // # Fast arc top to bottom right
    enum2string(c"BOTH_T5_T___R", BOTH_T5_T___R), // # Fast arc top to right
    enum2string(c"BOTH_T5_T__TR", BOTH_T5_T__TR), // # Fast arc top to top right
    enum2string(c"BOTH_T5_T__TL", BOTH_T5_T__TL), // # Fast arc top to top left
    enum2string(c"BOTH_T5_T___L", BOTH_T5_T___L), // # Fast arc top to left
    enum2string(c"BOTH_T5_T__BL", BOTH_T5_T__BL), // # Fast arc top to bottom left
    enum2string(c"BOTH_T5_TL_BR", BOTH_T5_TL_BR), // # Fast weak spin top left to bottom right
    enum2string(c"BOTH_T5_TL_BL", BOTH_T5_TL_BL), // # Fast arc top left to bottom left
    enum2string(c"BOTH_T5__L_BR", BOTH_T5__L_BR), // # Fast weak spin left to bottom right
    enum2string(c"BOTH_T5__L__R", BOTH_T5__L__R), // # Fast weak spin left to right
    enum2string(c"BOTH_T5__L_TL", BOTH_T5__L_TL), // # Fast arc left to top left
    enum2string(c"BOTH_T5_BL_BR", BOTH_T5_BL_BR), // # Fast weak spin bottom left to bottom right
    enum2string(c"BOTH_T5_BL__R", BOTH_T5_BL__R), // # Fast weak spin bottom left to right
    enum2string(c"BOTH_T5_BL_TR", BOTH_T5_BL_TR), // # Fast weak spin bottom left to top right
    enum2string(c"BOTH_T5_BL__L", BOTH_T5_BL__L), // # Fast arc bottom left to left
    //Saber Arc Transitions that use existing animations played backwards
    enum2string(c"BOTH_T5_BR_TR", BOTH_T5_BR_TR), // # Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T5_TR_BR)
    enum2string(c"BOTH_T5_BR_T_", BOTH_T5_BR_T_), // # Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T5_T__BR)
    enum2string(c"BOTH_T5__R_BR", BOTH_T5__R_BR), // # Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T5_BR__R)
    enum2string(c"BOTH_T5__R_T_", BOTH_T5__R_T_), // # Fast ar right to top				(use: ENUM2STRING(BOTH_T5_T___R)
    enum2string(c"BOTH_T5_TR__R", BOTH_T5_TR__R), // # Fast arc top right to right			(use: ENUM2STRING(BOTH_T5__R_TR)
    enum2string(c"BOTH_T5_TR_T_", BOTH_T5_TR_T_), // # Fast arc top right to top				(use: ENUM2STRING(BOTH_T5_T__TR)
    enum2string(c"BOTH_T5_TL__R", BOTH_T5_TL__R), // # Fast arc top left to right			(use: ENUM2STRING(BOTH_T5__R_TL)
    enum2string(c"BOTH_T5_TL_TR", BOTH_T5_TL_TR), // # Fast arc top left to top right			(use: ENUM2STRING(BOTH_T5_TR_TL)
    enum2string(c"BOTH_T5_TL_T_", BOTH_T5_TL_T_), // # Fast arc top left to top				(use: ENUM2STRING(BOTH_T5_T__TL)
    enum2string(c"BOTH_T5_TL__L", BOTH_T5_TL__L), // # Fast arc top left to left				(use: ENUM2STRING(BOTH_T5__L_TL)
    enum2string(c"BOTH_T5__L_TR", BOTH_T5__L_TR), // # Fast arc left to top right			(use: ENUM2STRING(BOTH_T5_TR__L)
    enum2string(c"BOTH_T5__L_T_", BOTH_T5__L_T_), // # Fast arc left to top				(use: ENUM2STRING(BOTH_T5_T___L)
    enum2string(c"BOTH_T5__L_BL", BOTH_T5__L_BL), // # Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T5_BL__L)
    enum2string(c"BOTH_T5_BL_T_", BOTH_T5_BL_T_), // # Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T5_T__BL)
    enum2string(c"BOTH_T5_BL_TL", BOTH_T5_BL_TL), // # Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T5_TL_BL)
    //Saber Attack Start Transitions
    enum2string(c"BOTH_S5_S1_T_", BOTH_S5_S1_T_), // # Fast plain transition from stance1 to top-to-bottom Fast weak attack
    enum2string(c"BOTH_S5_S1__L", BOTH_S5_S1__L), // # Fast plain transition from stance1 to left-to-right Fast weak attack
    enum2string(c"BOTH_S5_S1__R", BOTH_S5_S1__R), // # Fast plain transition from stance1 to right-to-left Fast weak attack
    enum2string(c"BOTH_S5_S1_TL", BOTH_S5_S1_TL), // # Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
    enum2string(c"BOTH_S5_S1_BR", BOTH_S5_S1_BR), // # Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
    enum2string(c"BOTH_S5_S1_BL", BOTH_S5_S1_BL), // # Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
    enum2string(c"BOTH_S5_S1_TR", BOTH_S5_S1_TR), // # Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
    //Saber Attack Return Transitions
    enum2string(c"BOTH_R5_B__S1", BOTH_R5_B__S1), // # Fast plain transition from top-to-bottom Fast weak attack to stance1
    enum2string(c"BOTH_R5__L_S1", BOTH_R5__L_S1), // # Fast plain transition from left-to-right Fast weak attack to stance1
    enum2string(c"BOTH_R5__R_S1", BOTH_R5__R_S1), // # Fast plain transition from right-to-left Fast weak attack to stance1
    enum2string(c"BOTH_R5_TL_S1", BOTH_R5_TL_S1), // # Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
    enum2string(c"BOTH_R5_BR_S1", BOTH_R5_BR_S1), // # Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
    enum2string(c"BOTH_R5_BL_S1", BOTH_R5_BL_S1), // # Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
    enum2string(c"BOTH_R5_TR_S1", BOTH_R5_TR_S1), // # Fast plain transition from top-right-to-bottom-left Fast weak attack
    //Saber Attack Bounces (first 4 frames of an attack), played backwards)
    enum2string(c"BOTH_B5_BR___", BOTH_B5_BR___), // # Bounce-back if attack from BR is blocked
    enum2string(c"BOTH_B5__R___", BOTH_B5__R___), // # Bounce-back if attack from R is blocked
    enum2string(c"BOTH_B5_TR___", BOTH_B5_TR___), // # Bounce-back if attack from TR is blocked
    enum2string(c"BOTH_B5_T____", BOTH_B5_T____), // # Bounce-back if attack from T is blocked
    enum2string(c"BOTH_B5_TL___", BOTH_B5_TL___), // # Bounce-back if attack from TL is blocked
    enum2string(c"BOTH_B5__L___", BOTH_B5__L___), // # Bounce-back if attack from L is blocked
    enum2string(c"BOTH_B5_BL___", BOTH_B5_BL___), // # Bounce-back if attack from BL is blocked
    //Saber Attack Deflections (last 4 frames of an attack)
    enum2string(c"BOTH_D5_BR___", BOTH_D5_BR___), // # Deflection toward BR
    enum2string(c"BOTH_D5__R___", BOTH_D5__R___), // # Deflection toward R
    enum2string(c"BOTH_D5_TR___", BOTH_D5_TR___), // # Deflection toward TR
    enum2string(c"BOTH_D5_TL___", BOTH_D5_TL___), // # Deflection toward TL
    enum2string(c"BOTH_D5__L___", BOTH_D5__L___), // # Deflection toward L
    enum2string(c"BOTH_D5_BL___", BOTH_D5_BL___), // # Deflection toward BL
    enum2string(c"BOTH_D5_B____", BOTH_D5_B____), // # Deflection toward B
    //Saber attack anims - power level 6
    enum2string(c"BOTH_A6_T__B_", BOTH_A6_T__B_), // # Fast weak vertical attack top to bottom
    enum2string(c"BOTH_A6__L__R", BOTH_A6__L__R), // # Fast weak horizontal attack left to right
    enum2string(c"BOTH_A6__R__L", BOTH_A6__R__L), // # Fast weak horizontal attack right to left
    enum2string(c"BOTH_A6_TL_BR", BOTH_A6_TL_BR), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A6_BR_TL", BOTH_A6_BR_TL), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A6_BL_TR", BOTH_A6_BL_TR), // # Fast weak diagonal attack bottom left to top right
    enum2string(c"BOTH_A6_TR_BL", BOTH_A6_TR_BL), // # Fast weak diagonal attack bottom left to right
    //Saber Arc and Spin Transitions
    enum2string(c"BOTH_T6_BR__R", BOTH_T6_BR__R), // # Fast arc bottom right to right
    enum2string(c"BOTH_T6_BR_TL", BOTH_T6_BR_TL), // # Fast weak spin bottom right to top left
    enum2string(c"BOTH_T6_BR__L", BOTH_T6_BR__L), // # Fast weak spin bottom right to left
    enum2string(c"BOTH_T6_BR_BL", BOTH_T6_BR_BL), // # Fast weak spin bottom right to bottom left
    enum2string(c"BOTH_T6__R_TR", BOTH_T6__R_TR), // # Fast arc right to top right
    enum2string(c"BOTH_T6__R_TL", BOTH_T6__R_TL), // # Fast arc right to top left
    enum2string(c"BOTH_T6__R__L", BOTH_T6__R__L), // # Fast weak spin right to left
    enum2string(c"BOTH_T6__R_BL", BOTH_T6__R_BL), // # Fast weak spin right to bottom left
    enum2string(c"BOTH_T6_TR_BR", BOTH_T6_TR_BR), // # Fast arc top right to bottom right
    enum2string(c"BOTH_T6_TR_TL", BOTH_T6_TR_TL), // # Fast arc top right to top left
    enum2string(c"BOTH_T6_TR__L", BOTH_T6_TR__L), // # Fast arc top right to left
    enum2string(c"BOTH_T6_TR_BL", BOTH_T6_TR_BL), // # Fast weak spin top right to bottom left
    enum2string(c"BOTH_T6_T__BR", BOTH_T6_T__BR), // # Fast arc top to bottom right
    enum2string(c"BOTH_T6_T___R", BOTH_T6_T___R), // # Fast arc top to right
    enum2string(c"BOTH_T6_T__TR", BOTH_T6_T__TR), // # Fast arc top to top right
    enum2string(c"BOTH_T6_T__TL", BOTH_T6_T__TL), // # Fast arc top to top left
    enum2string(c"BOTH_T6_T___L", BOTH_T6_T___L), // # Fast arc top to left
    enum2string(c"BOTH_T6_T__BL", BOTH_T6_T__BL), // # Fast arc top to bottom left
    enum2string(c"BOTH_T6_TL_BR", BOTH_T6_TL_BR), // # Fast weak spin top left to bottom right
    enum2string(c"BOTH_T6_TL_BL", BOTH_T6_TL_BL), // # Fast arc top left to bottom left
    enum2string(c"BOTH_T6__L_BR", BOTH_T6__L_BR), // # Fast weak spin left to bottom right
    enum2string(c"BOTH_T6__L__R", BOTH_T6__L__R), // # Fast weak spin left to right
    enum2string(c"BOTH_T6__L_TL", BOTH_T6__L_TL), // # Fast arc left to top left
    enum2string(c"BOTH_T6_BL_BR", BOTH_T6_BL_BR), // # Fast weak spin bottom left to bottom right
    enum2string(c"BOTH_T6_BL__R", BOTH_T6_BL__R), // # Fast weak spin bottom left to right
    enum2string(c"BOTH_T6_BL_TR", BOTH_T6_BL_TR), // # Fast weak spin bottom left to top right
    enum2string(c"BOTH_T6_BL__L", BOTH_T6_BL__L), // # Fast arc bottom left to left
    //Saber Arc Transitions that use existing animations played backwards
    enum2string(c"BOTH_T6_BR_TR", BOTH_T6_BR_TR), // # Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T6_TR_BR)
    enum2string(c"BOTH_T6_BR_T_", BOTH_T6_BR_T_), // # Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T6_T__BR)
    enum2string(c"BOTH_T6__R_BR", BOTH_T6__R_BR), // # Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T6_BR__R)
    enum2string(c"BOTH_T6__R_T_", BOTH_T6__R_T_), // # Fast ar right to top				(use: ENUM2STRING(BOTH_T6_T___R)
    enum2string(c"BOTH_T6_TR__R", BOTH_T6_TR__R), // # Fast arc top right to right			(use: ENUM2STRING(BOTH_T6__R_TR)
    enum2string(c"BOTH_T6_TR_T_", BOTH_T6_TR_T_), // # Fast arc top right to top				(use: ENUM2STRING(BOTH_T6_T__TR)
    enum2string(c"BOTH_T6_TL__R", BOTH_T6_TL__R), // # Fast arc top left to right			(use: ENUM2STRING(BOTH_T6__R_TL)
    enum2string(c"BOTH_T6_TL_TR", BOTH_T6_TL_TR), // # Fast arc top left to top right			(use: ENUM2STRING(BOTH_T6_TR_TL)
    enum2string(c"BOTH_T6_TL_T_", BOTH_T6_TL_T_), // # Fast arc top left to top				(use: ENUM2STRING(BOTH_T6_T__TL)
    enum2string(c"BOTH_T6_TL__L", BOTH_T6_TL__L), // # Fast arc top left to left				(use: ENUM2STRING(BOTH_T6__L_TL)
    enum2string(c"BOTH_T6__L_TR", BOTH_T6__L_TR), // # Fast arc left to top right			(use: ENUM2STRING(BOTH_T6_TR__L)
    enum2string(c"BOTH_T6__L_T_", BOTH_T6__L_T_), // # Fast arc left to top				(use: ENUM2STRING(BOTH_T6_T___L)
    enum2string(c"BOTH_T6__L_BL", BOTH_T6__L_BL), // # Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T6_BL__L)
    enum2string(c"BOTH_T6_BL_T_", BOTH_T6_BL_T_), // # Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T6_T__BL)
    enum2string(c"BOTH_T6_BL_TL", BOTH_T6_BL_TL), // # Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T6_TL_BL)
    //Saber Attack Start Transitions
    enum2string(c"BOTH_S6_S6_T_", BOTH_S6_S6_T_), // # Fast plain transition from stance1 to top-to-bottom Fast weak attack
    enum2string(c"BOTH_S6_S6__L", BOTH_S6_S6__L), // # Fast plain transition from stance1 to left-to-right Fast weak attack
    enum2string(c"BOTH_S6_S6__R", BOTH_S6_S6__R), // # Fast plain transition from stance1 to right-to-left Fast weak attack
    enum2string(c"BOTH_S6_S6_TL", BOTH_S6_S6_TL), // # Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
    enum2string(c"BOTH_S6_S6_BR", BOTH_S6_S6_BR), // # Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
    enum2string(c"BOTH_S6_S6_BL", BOTH_S6_S6_BL), // # Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
    enum2string(c"BOTH_S6_S6_TR", BOTH_S6_S6_TR), // # Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
    //Saber Attack Return Transitions
    enum2string(c"BOTH_R6_B__S6", BOTH_R6_B__S6), // # Fast plain transition from top-to-bottom Fast weak attack to stance1
    enum2string(c"BOTH_R6__L_S6", BOTH_R6__L_S6), // # Fast plain transition from left-to-right Fast weak attack to stance1
    enum2string(c"BOTH_R6__R_S6", BOTH_R6__R_S6), // # Fast plain transition from right-to-left Fast weak attack to stance1
    enum2string(c"BOTH_R6_TL_S6", BOTH_R6_TL_S6), // # Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
    enum2string(c"BOTH_R6_BR_S6", BOTH_R6_BR_S6), // # Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
    enum2string(c"BOTH_R6_BL_S6", BOTH_R6_BL_S6), // # Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
    enum2string(c"BOTH_R6_TR_S6", BOTH_R6_TR_S6), // # Fast plain transition from top-right-to-bottom-left Fast weak attack
    //Saber Attack Bounces (first 4 frames of an attack), played backwards)
    enum2string(c"BOTH_B6_BR___", BOTH_B6_BR___), // # Bounce-back if attack from BR is blocked
    enum2string(c"BOTH_B6__R___", BOTH_B6__R___), // # Bounce-back if attack from R is blocked
    enum2string(c"BOTH_B6_TR___", BOTH_B6_TR___), // # Bounce-back if attack from TR is blocked
    enum2string(c"BOTH_B6_T____", BOTH_B6_T____), // # Bounce-back if attack from T is blocked
    enum2string(c"BOTH_B6_TL___", BOTH_B6_TL___), // # Bounce-back if attack from TL is blocked
    enum2string(c"BOTH_B6__L___", BOTH_B6__L___), // # Bounce-back if attack from L is blocked
    enum2string(c"BOTH_B6_BL___", BOTH_B6_BL___), // # Bounce-back if attack from BL is blocked
    //Saber Attack Deflections (last 4 frames of an attack)
    enum2string(c"BOTH_D6_BR___", BOTH_D6_BR___), // # Deflection toward BR
    enum2string(c"BOTH_D6__R___", BOTH_D6__R___), // # Deflection toward R
    enum2string(c"BOTH_D6_TR___", BOTH_D6_TR___), // # Deflection toward TR
    enum2string(c"BOTH_D6_TL___", BOTH_D6_TL___), // # Deflection toward TL
    enum2string(c"BOTH_D6__L___", BOTH_D6__L___), // # Deflection toward L
    enum2string(c"BOTH_D6_BL___", BOTH_D6_BL___), // # Deflection toward BL
    enum2string(c"BOTH_D6_B____", BOTH_D6_B____), // # Deflection toward B
    //Saber attack anims - power level 7
    enum2string(c"BOTH_A7_T__B_", BOTH_A7_T__B_), // # Fast weak vertical attack top to bottom
    enum2string(c"BOTH_A7__L__R", BOTH_A7__L__R), // # Fast weak horizontal attack left to right
    enum2string(c"BOTH_A7__R__L", BOTH_A7__R__L), // # Fast weak horizontal attack right to left
    enum2string(c"BOTH_A7_TL_BR", BOTH_A7_TL_BR), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A7_BR_TL", BOTH_A7_BR_TL), // # Fast weak diagonal attack top left to botom right
    enum2string(c"BOTH_A7_BL_TR", BOTH_A7_BL_TR), // # Fast weak diagonal attack bottom left to top right
    enum2string(c"BOTH_A7_TR_BL", BOTH_A7_TR_BL), // # Fast weak diagonal attack bottom left to right
    //Saber Arc and Spin Transitions
    enum2string(c"BOTH_T7_BR__R", BOTH_T7_BR__R), // # Fast arc bottom right to right
    enum2string(c"BOTH_T7_BR_TL", BOTH_T7_BR_TL), // # Fast weak spin bottom right to top left
    enum2string(c"BOTH_T7_BR__L", BOTH_T7_BR__L), // # Fast weak spin bottom right to left
    enum2string(c"BOTH_T7_BR_BL", BOTH_T7_BR_BL), // # Fast weak spin bottom right to bottom left
    enum2string(c"BOTH_T7__R_TR", BOTH_T7__R_TR), // # Fast arc right to top right
    enum2string(c"BOTH_T7__R_TL", BOTH_T7__R_TL), // # Fast arc right to top left
    enum2string(c"BOTH_T7__R__L", BOTH_T7__R__L), // # Fast weak spin right to left
    enum2string(c"BOTH_T7__R_BL", BOTH_T7__R_BL), // # Fast weak spin right to bottom left
    enum2string(c"BOTH_T7_TR_BR", BOTH_T7_TR_BR), // # Fast arc top right to bottom right
    enum2string(c"BOTH_T7_TR_TL", BOTH_T7_TR_TL), // # Fast arc top right to top left
    enum2string(c"BOTH_T7_TR__L", BOTH_T7_TR__L), // # Fast arc top right to left
    enum2string(c"BOTH_T7_TR_BL", BOTH_T7_TR_BL), // # Fast weak spin top right to bottom left
    enum2string(c"BOTH_T7_T__BR", BOTH_T7_T__BR), // # Fast arc top to bottom right
    enum2string(c"BOTH_T7_T___R", BOTH_T7_T___R), // # Fast arc top to right
    enum2string(c"BOTH_T7_T__TR", BOTH_T7_T__TR), // # Fast arc top to top right
    enum2string(c"BOTH_T7_T__TL", BOTH_T7_T__TL), // # Fast arc top to top left
    enum2string(c"BOTH_T7_T___L", BOTH_T7_T___L), // # Fast arc top to left
    enum2string(c"BOTH_T7_T__BL", BOTH_T7_T__BL), // # Fast arc top to bottom left
    enum2string(c"BOTH_T7_TL_BR", BOTH_T7_TL_BR), // # Fast weak spin top left to bottom right
    enum2string(c"BOTH_T7_TL_BL", BOTH_T7_TL_BL), // # Fast arc top left to bottom left
    enum2string(c"BOTH_T7__L_BR", BOTH_T7__L_BR), // # Fast weak spin left to bottom right
    enum2string(c"BOTH_T7__L__R", BOTH_T7__L__R), // # Fast weak spin left to right
    enum2string(c"BOTH_T7__L_TL", BOTH_T7__L_TL), // # Fast arc left to top left
    enum2string(c"BOTH_T7_BL_BR", BOTH_T7_BL_BR), // # Fast weak spin bottom left to bottom right
    enum2string(c"BOTH_T7_BL__R", BOTH_T7_BL__R), // # Fast weak spin bottom left to right
    enum2string(c"BOTH_T7_BL_TR", BOTH_T7_BL_TR), // # Fast weak spin bottom left to top right
    enum2string(c"BOTH_T7_BL__L", BOTH_T7_BL__L), // # Fast arc bottom left to left
    //Saber Arc Transitions that use existing animations played backwards
    enum2string(c"BOTH_T7_BR_TR", BOTH_T7_BR_TR), // # Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T7_TR_BR)
    enum2string(c"BOTH_T7_BR_T_", BOTH_T7_BR_T_), // # Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T7_T__BR)
    enum2string(c"BOTH_T7__R_BR", BOTH_T7__R_BR), // # Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T7_BR__R)
    enum2string(c"BOTH_T7__R_T_", BOTH_T7__R_T_), // # Fast ar right to top				(use: ENUM2STRING(BOTH_T7_T___R)
    enum2string(c"BOTH_T7_TR__R", BOTH_T7_TR__R), // # Fast arc top right to right			(use: ENUM2STRING(BOTH_T7__R_TR)
    enum2string(c"BOTH_T7_TR_T_", BOTH_T7_TR_T_), // # Fast arc top right to top				(use: ENUM2STRING(BOTH_T7_T__TR)
    enum2string(c"BOTH_T7_TL__R", BOTH_T7_TL__R), // # Fast arc top left to right			(use: ENUM2STRING(BOTH_T7__R_TL)
    enum2string(c"BOTH_T7_TL_TR", BOTH_T7_TL_TR), // # Fast arc top left to top right			(use: ENUM2STRING(BOTH_T7_TR_TL)
    enum2string(c"BOTH_T7_TL_T_", BOTH_T7_TL_T_), // # Fast arc top left to top				(use: ENUM2STRING(BOTH_T7_T__TL)
    enum2string(c"BOTH_T7_TL__L", BOTH_T7_TL__L), // # Fast arc top left to left				(use: ENUM2STRING(BOTH_T7__L_TL)
    enum2string(c"BOTH_T7__L_TR", BOTH_T7__L_TR), // # Fast arc left to top right			(use: ENUM2STRING(BOTH_T7_TR__L)
    enum2string(c"BOTH_T7__L_T_", BOTH_T7__L_T_), // # Fast arc left to top				(use: ENUM2STRING(BOTH_T7_T___L)
    enum2string(c"BOTH_T7__L_BL", BOTH_T7__L_BL), // # Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T7_BL__L)
    enum2string(c"BOTH_T7_BL_T_", BOTH_T7_BL_T_), // # Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T7_T__BL)
    enum2string(c"BOTH_T7_BL_TL", BOTH_T7_BL_TL), // # Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T7_TL_BL)
    //Saber Attack Start Transitions
    enum2string(c"BOTH_S7_S7_T_", BOTH_S7_S7_T_), // # Fast plain transition from stance1 to top-to-bottom Fast weak attack
    enum2string(c"BOTH_S7_S7__L", BOTH_S7_S7__L), // # Fast plain transition from stance1 to left-to-right Fast weak attack
    enum2string(c"BOTH_S7_S7__R", BOTH_S7_S7__R), // # Fast plain transition from stance1 to right-to-left Fast weak attack
    enum2string(c"BOTH_S7_S7_TL", BOTH_S7_S7_TL), // # Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
    enum2string(c"BOTH_S7_S7_BR", BOTH_S7_S7_BR), // # Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
    enum2string(c"BOTH_S7_S7_BL", BOTH_S7_S7_BL), // # Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
    enum2string(c"BOTH_S7_S7_TR", BOTH_S7_S7_TR), // # Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
    //Saber Attack Return Transitions
    enum2string(c"BOTH_R7_B__S7", BOTH_R7_B__S7), // # Fast plain transition from top-to-bottom Fast weak attack to stance1
    enum2string(c"BOTH_R7__L_S7", BOTH_R7__L_S7), // # Fast plain transition from left-to-right Fast weak attack to stance1
    enum2string(c"BOTH_R7__R_S7", BOTH_R7__R_S7), // # Fast plain transition from right-to-left Fast weak attack to stance1
    enum2string(c"BOTH_R7_TL_S7", BOTH_R7_TL_S7), // # Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
    enum2string(c"BOTH_R7_BR_S7", BOTH_R7_BR_S7), // # Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
    enum2string(c"BOTH_R7_BL_S7", BOTH_R7_BL_S7), // # Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
    enum2string(c"BOTH_R7_TR_S7", BOTH_R7_TR_S7), // # Fast plain transition from top-right-to-bottom-left Fast weak attack
    //Saber Attack Bounces (first 4 frames of an attack), played backwards)
    enum2string(c"BOTH_B7_BR___", BOTH_B7_BR___), // # Bounce-back if attack from BR is blocked
    enum2string(c"BOTH_B7__R___", BOTH_B7__R___), // # Bounce-back if attack from R is blocked
    enum2string(c"BOTH_B7_TR___", BOTH_B7_TR___), // # Bounce-back if attack from TR is blocked
    enum2string(c"BOTH_B7_T____", BOTH_B7_T____), // # Bounce-back if attack from T is blocked
    enum2string(c"BOTH_B7_TL___", BOTH_B7_TL___), // # Bounce-back if attack from TL is blocked
    enum2string(c"BOTH_B7__L___", BOTH_B7__L___), // # Bounce-back if attack from L is blocked
    enum2string(c"BOTH_B7_BL___", BOTH_B7_BL___), // # Bounce-back if attack from BL is blocked
    //Saber Attack Deflections (last 4 frames of an attack)
    enum2string(c"BOTH_D7_BR___", BOTH_D7_BR___), // # Deflection toward BR
    enum2string(c"BOTH_D7__R___", BOTH_D7__R___), // # Deflection toward R
    enum2string(c"BOTH_D7_TR___", BOTH_D7_TR___), // # Deflection toward TR
    enum2string(c"BOTH_D7_TL___", BOTH_D7_TL___), // # Deflection toward TL
    enum2string(c"BOTH_D7__L___", BOTH_D7__L___), // # Deflection toward L
    enum2string(c"BOTH_D7_BL___", BOTH_D7_BL___), // # Deflection toward BL
    enum2string(c"BOTH_D7_B____", BOTH_D7_B____), // # Deflection toward B
    //Saber parry anims
    enum2string(c"BOTH_P1_S1_T_", BOTH_P1_S1_T_), // # Block shot/saber top
    enum2string(c"BOTH_P1_S1_TR", BOTH_P1_S1_TR), // # Block shot/saber top right
    enum2string(c"BOTH_P1_S1_TL", BOTH_P1_S1_TL), // # Block shot/saber top left
    enum2string(c"BOTH_P1_S1_BL", BOTH_P1_S1_BL), // # Block shot/saber bottom left
    enum2string(c"BOTH_P1_S1_BR", BOTH_P1_S1_BR), // # Block shot/saber bottom right
    //Saber knockaway
    enum2string(c"BOTH_K1_S1_T_", BOTH_K1_S1_T_), // # knockaway saber top
    enum2string(c"BOTH_K1_S1_TR", BOTH_K1_S1_TR), // # knockaway saber top right
    enum2string(c"BOTH_K1_S1_TL", BOTH_K1_S1_TL), // # knockaway saber top left
    enum2string(c"BOTH_K1_S1_BL", BOTH_K1_S1_BL), // # knockaway saber bottom left
    enum2string(c"BOTH_K1_S1_B_", BOTH_K1_S1_B_), // # knockaway saber bottom
    enum2string(c"BOTH_K1_S1_BR", BOTH_K1_S1_BR), // # knockaway saber bottom right
    //Saber attack knocked away
    enum2string(c"BOTH_V1_BR_S1", BOTH_V1_BR_S1), // # BR attack knocked away
    enum2string(c"BOTH_V1__R_S1", BOTH_V1__R_S1), // # R attack knocked away
    enum2string(c"BOTH_V1_TR_S1", BOTH_V1_TR_S1), // # TR attack knocked away
    enum2string(c"BOTH_V1_T__S1", BOTH_V1_T__S1), // # T attack knocked away
    enum2string(c"BOTH_V1_TL_S1", BOTH_V1_TL_S1), // # TL attack knocked away
    enum2string(c"BOTH_V1__L_S1", BOTH_V1__L_S1), // # L attack knocked away
    enum2string(c"BOTH_V1_BL_S1", BOTH_V1_BL_S1), // # BL attack knocked away
    enum2string(c"BOTH_V1_B__S1", BOTH_V1_B__S1), // # B attack knocked away
    //Saber parry broken
    enum2string(c"BOTH_H1_S1_T_", BOTH_H1_S1_T_), // # saber knocked down from top parry
    enum2string(c"BOTH_H1_S1_TR", BOTH_H1_S1_TR), // # saber knocked down-left from TR parry
    enum2string(c"BOTH_H1_S1_TL", BOTH_H1_S1_TL), // # saber knocked down-right from TL parry
    enum2string(c"BOTH_H1_S1_BL", BOTH_H1_S1_BL), // # saber knocked up-right from BL parry
    enum2string(c"BOTH_H1_S1_B_", BOTH_H1_S1_B_), // # saber knocked up over head from ready?
    enum2string(c"BOTH_H1_S1_BR", BOTH_H1_S1_BR), // # saber knocked up-left from BR parry
    //Dual Sabers parry anims
    enum2string(c"BOTH_P6_S6_T_", BOTH_P6_S6_T_), // # Block shot/saber top
    enum2string(c"BOTH_P6_S6_TR", BOTH_P6_S6_TR), // # Block shot/saber top right
    enum2string(c"BOTH_P6_S6_TL", BOTH_P6_S6_TL), // # Block shot/saber top left
    enum2string(c"BOTH_P6_S6_BL", BOTH_P6_S6_BL), // # Block shot/saber bottom left
    enum2string(c"BOTH_P6_S6_BR", BOTH_P6_S6_BR), // # Block shot/saber bottom right
    //Dual Sabers knockaway
    enum2string(c"BOTH_K6_S6_T_", BOTH_K6_S6_T_), // # knockaway saber top
    enum2string(c"BOTH_K6_S6_TR", BOTH_K6_S6_TR), // # knockaway saber top right
    enum2string(c"BOTH_K6_S6_TL", BOTH_K6_S6_TL), // # knockaway saber top left
    enum2string(c"BOTH_K6_S6_BL", BOTH_K6_S6_BL), // # knockaway saber bottom left
    enum2string(c"BOTH_K6_S6_B_", BOTH_K6_S6_B_), // # knockaway saber bottom
    enum2string(c"BOTH_K6_S6_BR", BOTH_K6_S6_BR), // # knockaway saber bottom right
    //Dual Sabers attack knocked away
    enum2string(c"BOTH_V6_BR_S6", BOTH_V6_BR_S6), // # BR attack knocked away
    enum2string(c"BOTH_V6__R_S6", BOTH_V6__R_S6), // # R attack knocked away
    enum2string(c"BOTH_V6_TR_S6", BOTH_V6_TR_S6), // # TR attack knocked away
    enum2string(c"BOTH_V6_T__S6", BOTH_V6_T__S6), // # T attack knocked away
    enum2string(c"BOTH_V6_TL_S6", BOTH_V6_TL_S6), // # TL attack knocked away
    enum2string(c"BOTH_V6__L_S6", BOTH_V6__L_S6), // # L attack knocked away
    enum2string(c"BOTH_V6_BL_S6", BOTH_V6_BL_S6), // # BL attack knocked away
    enum2string(c"BOTH_V6_B__S6", BOTH_V6_B__S6), // # B attack knocked away
    //Dual Sabers parry broken
    enum2string(c"BOTH_H6_S6_T_", BOTH_H6_S6_T_), // # saber knocked down from top parry
    enum2string(c"BOTH_H6_S6_TR", BOTH_H6_S6_TR), // # saber knocked down-left from TR parry
    enum2string(c"BOTH_H6_S6_TL", BOTH_H6_S6_TL), // # saber knocked down-right from TL parry
    enum2string(c"BOTH_H6_S6_BL", BOTH_H6_S6_BL), // # saber knocked up-right from BL parry
    enum2string(c"BOTH_H6_S6_B_", BOTH_H6_S6_B_), // # saber knocked up over head from ready?
    enum2string(c"BOTH_H6_S6_BR", BOTH_H6_S6_BR), // # saber knocked up-left from BR parry
    //SaberStaff parry anims
    enum2string(c"BOTH_P7_S7_T_", BOTH_P7_S7_T_), // # Block shot/saber top
    enum2string(c"BOTH_P7_S7_TR", BOTH_P7_S7_TR), // # Block shot/saber top right
    enum2string(c"BOTH_P7_S7_TL", BOTH_P7_S7_TL), // # Block shot/saber top left
    enum2string(c"BOTH_P7_S7_BL", BOTH_P7_S7_BL), // # Block shot/saber bottom left
    enum2string(c"BOTH_P7_S7_BR", BOTH_P7_S7_BR), // # Block shot/saber bottom right
    //SaberStaff knockaway
    enum2string(c"BOTH_K7_S7_T_", BOTH_K7_S7_T_), // # knockaway saber top
    enum2string(c"BOTH_K7_S7_TR", BOTH_K7_S7_TR), // # knockaway saber top right
    enum2string(c"BOTH_K7_S7_TL", BOTH_K7_S7_TL), // # knockaway saber top left
    enum2string(c"BOTH_K7_S7_BL", BOTH_K7_S7_BL), // # knockaway saber bottom left
    enum2string(c"BOTH_K7_S7_B_", BOTH_K7_S7_B_), // # knockaway saber bottom
    enum2string(c"BOTH_K7_S7_BR", BOTH_K7_S7_BR), // # knockaway saber bottom right
    //SaberStaff attack knocked away
    enum2string(c"BOTH_V7_BR_S7", BOTH_V7_BR_S7), // # BR attack knocked away
    enum2string(c"BOTH_V7__R_S7", BOTH_V7__R_S7), // # R attack knocked away
    enum2string(c"BOTH_V7_TR_S7", BOTH_V7_TR_S7), // # TR attack knocked away
    enum2string(c"BOTH_V7_T__S7", BOTH_V7_T__S7), // # T attack knocked away
    enum2string(c"BOTH_V7_TL_S7", BOTH_V7_TL_S7), // # TL attack knocked away
    enum2string(c"BOTH_V7__L_S7", BOTH_V7__L_S7), // # L attack knocked away
    enum2string(c"BOTH_V7_BL_S7", BOTH_V7_BL_S7), // # BL attack knocked away
    enum2string(c"BOTH_V7_B__S7", BOTH_V7_B__S7), // # B attack knocked away
    //SaberStaff parry broken
    enum2string(c"BOTH_H7_S7_T_", BOTH_H7_S7_T_), // # saber knocked down from top parry
    enum2string(c"BOTH_H7_S7_TR", BOTH_H7_S7_TR), // # saber knocked down-left from TR parry
    enum2string(c"BOTH_H7_S7_TL", BOTH_H7_S7_TL), // # saber knocked down-right from TL parry
    enum2string(c"BOTH_H7_S7_BL", BOTH_H7_S7_BL), // # saber knocked up-right from BL parry
    enum2string(c"BOTH_H7_S7_B_", BOTH_H7_S7_B_), // # saber knocked up over head from ready?
    enum2string(c"BOTH_H7_S7_BR", BOTH_H7_S7_BR), // # saber knocked up-left from BR parry
    //Sabers locked anims
    //* #sep BOTH_ SABER LOCKED ANIMS
    //BOTH_(DL, S, ST)_(DL, S, ST)_(T, S)_(L, B, SB)_1(_W, _L)
    //===Single locks==================================================================
    //SINGLE vs. DUAL
    //side locks - I'm using a single and they're using dual
    enum2string(c"BOTH_LK_S_DL_S_B_1_L", BOTH_LK_S_DL_S_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_S_DL_S_B_1_W", BOTH_LK_S_DL_S_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_S_DL_S_L_1", BOTH_LK_S_DL_S_L_1), // lock if I'm using single vs. a dual
    enum2string(c"BOTH_LK_S_DL_S_SB_1_L", BOTH_LK_S_DL_S_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_S_DL_S_SB_1_W", BOTH_LK_S_DL_S_SB_1_W), // super break I won
    //top locks
    enum2string(c"BOTH_LK_S_DL_T_B_1_L", BOTH_LK_S_DL_T_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_S_DL_T_B_1_W", BOTH_LK_S_DL_T_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_S_DL_T_L_1", BOTH_LK_S_DL_T_L_1), // lock if I'm using single vs. a dual
    enum2string(c"BOTH_LK_S_DL_T_SB_1_L", BOTH_LK_S_DL_T_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_S_DL_T_SB_1_W", BOTH_LK_S_DL_T_SB_1_W), // super break I won
    //SINGLE vs. STAFF
    //side locks
    enum2string(c"BOTH_LK_S_ST_S_B_1_L", BOTH_LK_S_ST_S_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_S_ST_S_B_1_W", BOTH_LK_S_ST_S_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_S_ST_S_L_1", BOTH_LK_S_ST_S_L_1), // lock if I'm using single vs. a staff
    enum2string(c"BOTH_LK_S_ST_S_SB_1_L", BOTH_LK_S_ST_S_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_S_ST_S_SB_1_W", BOTH_LK_S_ST_S_SB_1_W), // super break I won
    //top locks
    enum2string(c"BOTH_LK_S_ST_T_B_1_L", BOTH_LK_S_ST_T_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_S_ST_T_B_1_W", BOTH_LK_S_ST_T_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_S_ST_T_L_1", BOTH_LK_S_ST_T_L_1), // lock if I'm using single vs. a staff
    enum2string(c"BOTH_LK_S_ST_T_SB_1_L", BOTH_LK_S_ST_T_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_S_ST_T_SB_1_W", BOTH_LK_S_ST_T_SB_1_W), // super break I won
    //SINGLE vs. SINGLE
    //side locks
    enum2string(c"BOTH_LK_S_S_S_B_1_L", BOTH_LK_S_S_S_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_S_S_S_B_1_W", BOTH_LK_S_S_S_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_S_S_S_L_1", BOTH_LK_S_S_S_L_1), // lock if I'm using single vs. a single and I initiated
    enum2string(c"BOTH_LK_S_S_S_SB_1_L", BOTH_LK_S_S_S_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_S_S_S_SB_1_W", BOTH_LK_S_S_S_SB_1_W), // super break I won
    //top locks
    enum2string(c"BOTH_LK_S_S_T_B_1_L", BOTH_LK_S_S_T_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_S_S_T_B_1_W", BOTH_LK_S_S_T_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_S_S_T_L_1", BOTH_LK_S_S_T_L_1), // lock if I'm using single vs. a single and I initiated
    enum2string(c"BOTH_LK_S_S_T_SB_1_L", BOTH_LK_S_S_T_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_S_S_T_SB_1_W", BOTH_LK_S_S_T_SB_1_W), // super break I won
    //===Dual Saber locks==================================================================
    //DUAL vs. DUAL
    //side locks
    enum2string(c"BOTH_LK_DL_DL_S_B_1_L", BOTH_LK_DL_DL_S_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_DL_DL_S_B_1_W", BOTH_LK_DL_DL_S_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_DL_DL_S_L_1", BOTH_LK_DL_DL_S_L_1), // lock if I'm using dual vs. dual and I initiated
    enum2string(c"BOTH_LK_DL_DL_S_SB_1_L", BOTH_LK_DL_DL_S_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_DL_DL_S_SB_1_W", BOTH_LK_DL_DL_S_SB_1_W), // super break I won
    //top locks
    enum2string(c"BOTH_LK_DL_DL_T_B_1_L", BOTH_LK_DL_DL_T_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_DL_DL_T_B_1_W", BOTH_LK_DL_DL_T_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_DL_DL_T_L_1", BOTH_LK_DL_DL_T_L_1), // lock if I'm using dual vs. dual and I initiated
    enum2string(c"BOTH_LK_DL_DL_T_SB_1_L", BOTH_LK_DL_DL_T_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_DL_DL_T_SB_1_W", BOTH_LK_DL_DL_T_SB_1_W), // super break I won
    //DUAL vs. STAFF
    //side locks
    enum2string(c"BOTH_LK_DL_ST_S_B_1_L", BOTH_LK_DL_ST_S_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_DL_ST_S_B_1_W", BOTH_LK_DL_ST_S_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_DL_ST_S_L_1", BOTH_LK_DL_ST_S_L_1), // lock if I'm using dual vs. a staff
    enum2string(c"BOTH_LK_DL_ST_S_SB_1_L", BOTH_LK_DL_ST_S_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_DL_ST_S_SB_1_W", BOTH_LK_DL_ST_S_SB_1_W), // super break I won
    //top locks
    enum2string(c"BOTH_LK_DL_ST_T_B_1_L", BOTH_LK_DL_ST_T_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_DL_ST_T_B_1_W", BOTH_LK_DL_ST_T_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_DL_ST_T_L_1", BOTH_LK_DL_ST_T_L_1), // lock if I'm using dual vs. a staff
    enum2string(c"BOTH_LK_DL_ST_T_SB_1_L", BOTH_LK_DL_ST_T_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_DL_ST_T_SB_1_W", BOTH_LK_DL_ST_T_SB_1_W), // super break I won
    //DUAL vs. SINGLE
    //side locks
    enum2string(c"BOTH_LK_DL_S_S_B_1_L", BOTH_LK_DL_S_S_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_DL_S_S_B_1_W", BOTH_LK_DL_S_S_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_DL_S_S_L_1", BOTH_LK_DL_S_S_L_1), // lock if I'm using dual vs. a single
    enum2string(c"BOTH_LK_DL_S_S_SB_1_L", BOTH_LK_DL_S_S_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_DL_S_S_SB_1_W", BOTH_LK_DL_S_S_SB_1_W), // super break I won
    //top locks
    enum2string(c"BOTH_LK_DL_S_T_B_1_L", BOTH_LK_DL_S_T_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_DL_S_T_B_1_W", BOTH_LK_DL_S_T_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_DL_S_T_L_1", BOTH_LK_DL_S_T_L_1), // lock if I'm using dual vs. a single
    enum2string(c"BOTH_LK_DL_S_T_SB_1_L", BOTH_LK_DL_S_T_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_DL_S_T_SB_1_W", BOTH_LK_DL_S_T_SB_1_W), // super break I won
    //===Saber Staff locks==================================================================
    //STAFF vs. DUAL
    //side locks
    enum2string(c"BOTH_LK_ST_DL_S_B_1_L", BOTH_LK_ST_DL_S_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_ST_DL_S_B_1_W", BOTH_LK_ST_DL_S_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_ST_DL_S_L_1", BOTH_LK_ST_DL_S_L_1), // lock if I'm using staff vs. dual
    enum2string(c"BOTH_LK_ST_DL_S_SB_1_L", BOTH_LK_ST_DL_S_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_ST_DL_S_SB_1_W", BOTH_LK_ST_DL_S_SB_1_W), // super break I won
    //top locks
    enum2string(c"BOTH_LK_ST_DL_T_B_1_L", BOTH_LK_ST_DL_T_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_ST_DL_T_B_1_W", BOTH_LK_ST_DL_T_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_ST_DL_T_L_1", BOTH_LK_ST_DL_T_L_1), // lock if I'm using staff vs. dual
    enum2string(c"BOTH_LK_ST_DL_T_SB_1_L", BOTH_LK_ST_DL_T_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_ST_DL_T_SB_1_W", BOTH_LK_ST_DL_T_SB_1_W), // super break I won
    //STAFF vs. STAFF
    //side locks
    enum2string(c"BOTH_LK_ST_ST_S_B_1_L", BOTH_LK_ST_ST_S_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_ST_ST_S_B_1_W", BOTH_LK_ST_ST_S_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_ST_ST_S_L_1", BOTH_LK_ST_ST_S_L_1), // lock if I'm using staff vs. a staff and I initiated
    enum2string(c"BOTH_LK_ST_ST_S_SB_1_L", BOTH_LK_ST_ST_S_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_ST_ST_S_SB_1_W", BOTH_LK_ST_ST_S_SB_1_W), // super break I won
    //top locks
    enum2string(c"BOTH_LK_ST_ST_T_B_1_L", BOTH_LK_ST_ST_T_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_ST_ST_T_B_1_W", BOTH_LK_ST_ST_T_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_ST_ST_T_L_1", BOTH_LK_ST_ST_T_L_1), // lock if I'm using staff vs. a staff and I initiated
    enum2string(c"BOTH_LK_ST_ST_T_SB_1_L", BOTH_LK_ST_ST_T_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_ST_ST_T_SB_1_W", BOTH_LK_ST_ST_T_SB_1_W), // super break I won
    //STAFF vs. SINGLE
    //side locks
    enum2string(c"BOTH_LK_ST_S_S_B_1_L", BOTH_LK_ST_S_S_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_ST_S_S_B_1_W", BOTH_LK_ST_S_S_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_ST_S_S_L_1", BOTH_LK_ST_S_S_L_1), // lock if I'm using staff vs. a single
    enum2string(c"BOTH_LK_ST_S_S_SB_1_L", BOTH_LK_ST_S_S_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_ST_S_S_SB_1_W", BOTH_LK_ST_S_S_SB_1_W), // super break I won
    //top locks
    enum2string(c"BOTH_LK_ST_S_T_B_1_L", BOTH_LK_ST_S_T_B_1_L), // normal break I lost
    enum2string(c"BOTH_LK_ST_S_T_B_1_W", BOTH_LK_ST_S_T_B_1_W), // normal break I won
    enum2string(c"BOTH_LK_ST_S_T_L_1", BOTH_LK_ST_S_T_L_1), // lock if I'm using staff vs. a single
    enum2string(c"BOTH_LK_ST_S_T_SB_1_L", BOTH_LK_ST_S_T_SB_1_L), // super break I lost
    enum2string(c"BOTH_LK_ST_S_T_SB_1_W", BOTH_LK_ST_S_T_SB_1_W), // super break I won
    //Special cases for same saber style vs. each other (won't fit in nice 5-anim size lists above)
    enum2string(c"BOTH_LK_S_S_S_L_2", BOTH_LK_S_S_S_L_2), // lock if I'm using single vs. a single and other intitiated
    enum2string(c"BOTH_LK_S_S_T_L_2", BOTH_LK_S_S_T_L_2), // lock if I'm using single vs. a single and other initiated
    enum2string(c"BOTH_LK_DL_DL_S_L_2", BOTH_LK_DL_DL_S_L_2), // lock if I'm using dual vs. dual and other initiated
    enum2string(c"BOTH_LK_DL_DL_T_L_2", BOTH_LK_DL_DL_T_L_2), // lock if I'm using dual vs. dual and other initiated
    enum2string(c"BOTH_LK_ST_ST_S_L_2", BOTH_LK_ST_ST_S_L_2), // lock if I'm using staff vs. a staff and other initiated
    enum2string(c"BOTH_LK_ST_ST_T_L_2", BOTH_LK_ST_ST_T_L_2), // lock if I'm using staff vs. a staff and other initiated
    //===End Saber locks==================================================================
    enum2string(c"BOTH_BF2RETURN", BOTH_BF2RETURN), // #
    enum2string(c"BOTH_BF2BREAK", BOTH_BF2BREAK),   // #
    enum2string(c"BOTH_BF2LOCK", BOTH_BF2LOCK),     // #
    enum2string(c"BOTH_BF1RETURN", BOTH_BF1RETURN), // #
    enum2string(c"BOTH_BF1BREAK", BOTH_BF1BREAK),   // #
    enum2string(c"BOTH_BF1LOCK", BOTH_BF1LOCK),     // #
    enum2string(c"BOTH_CWCIRCLE_R2__R_S1", BOTH_CWCIRCLE_R2__R_S1), // #
    enum2string(c"BOTH_CCWCIRCLE_R2__L_S1", BOTH_CCWCIRCLE_R2__L_S1), // #
    enum2string(c"BOTH_CWCIRCLE_A2__L__R", BOTH_CWCIRCLE_A2__L__R), // #
    enum2string(c"BOTH_CCWCIRCLE_A2__R__L", BOTH_CCWCIRCLE_A2__R__L), // #
    enum2string(c"BOTH_CWCIRCLEBREAK", BOTH_CWCIRCLEBREAK), // #
    enum2string(c"BOTH_CCWCIRCLEBREAK", BOTH_CCWCIRCLEBREAK), // #
    enum2string(c"BOTH_CWCIRCLELOCK", BOTH_CWCIRCLELOCK), // #
    enum2string(c"BOTH_CCWCIRCLELOCK", BOTH_CCWCIRCLELOCK), // #
    //other saber anims/attacks
    enum2string(c"BOTH_SABERFAST_STANCE", BOTH_SABERFAST_STANCE),
    enum2string(c"BOTH_SABERSLOW_STANCE", BOTH_SABERSLOW_STANCE),
    enum2string(c"BOTH_SABERDUAL_STANCE", BOTH_SABERDUAL_STANCE),
    enum2string(c"BOTH_SABERSTAFF_STANCE", BOTH_SABERSTAFF_STANCE),
    enum2string(c"BOTH_A2_STABBACK1", BOTH_A2_STABBACK1), // # Stab saber backward
    enum2string(c"BOTH_ATTACK_BACK", BOTH_ATTACK_BACK),   // # Swing around backwards and attack
    enum2string(c"BOTH_JUMPFLIPSLASHDOWN1", BOTH_JUMPFLIPSLASHDOWN1), // #
    enum2string(c"BOTH_JUMPFLIPSTABDOWN", BOTH_JUMPFLIPSTABDOWN), // #
    enum2string(c"BOTH_FORCELEAP2_T__B_", BOTH_FORCELEAP2_T__B_), // #
    enum2string(c"BOTH_LUNGE2_B__T_", BOTH_LUNGE2_B__T_), // #
    enum2string(c"BOTH_CROUCHATTACKBACK1", BOTH_CROUCHATTACKBACK1), // #
    //New specials for JKA:
    enum2string(c"BOTH_JUMPATTACK6", BOTH_JUMPATTACK6), // #
    enum2string(c"BOTH_JUMPATTACK7", BOTH_JUMPATTACK7), // #
    enum2string(c"BOTH_SPINATTACK6", BOTH_SPINATTACK6), // #
    enum2string(c"BOTH_SPINATTACK7", BOTH_SPINATTACK7), // #
    enum2string(c"BOTH_S1_S6", BOTH_S1_S6), // #	From stand1 to saberdual stance - turning on your dual sabers
    enum2string(c"BOTH_S6_S1", BOTH_S6_S1), // #	From dualstaff stance to stand1 - turning off your dual sabers
    enum2string(c"BOTH_S1_S7", BOTH_S1_S7), // #	From stand1 to saberstaff stance - turning on your saberstaff
    enum2string(c"BOTH_S7_S1", BOTH_S7_S1), // #	From saberstaff stance to stand1 - turning off your saberstaff
    enum2string(c"BOTH_FORCELONGLEAP_START", BOTH_FORCELONGLEAP_START),
    enum2string(c"BOTH_FORCELONGLEAP_ATTACK", BOTH_FORCELONGLEAP_ATTACK),
    enum2string(c"BOTH_FORCELONGLEAP_LAND", BOTH_FORCELONGLEAP_LAND),
    enum2string(c"BOTH_FORCEWALLRUNFLIP_START", BOTH_FORCEWALLRUNFLIP_START),
    enum2string(c"BOTH_FORCEWALLRUNFLIP_END", BOTH_FORCEWALLRUNFLIP_END),
    enum2string(c"BOTH_FORCEWALLRUNFLIP_ALT", BOTH_FORCEWALLRUNFLIP_ALT),
    enum2string(
        c"BOTH_FORCEWALLREBOUND_FORWARD",
        BOTH_FORCEWALLREBOUND_FORWARD,
    ),
    enum2string(c"BOTH_FORCEWALLREBOUND_LEFT", BOTH_FORCEWALLREBOUND_LEFT),
    enum2string(c"BOTH_FORCEWALLREBOUND_BACK", BOTH_FORCEWALLREBOUND_BACK),
    enum2string(c"BOTH_FORCEWALLREBOUND_RIGHT", BOTH_FORCEWALLREBOUND_RIGHT),
    enum2string(c"BOTH_FORCEWALLHOLD_FORWARD", BOTH_FORCEWALLHOLD_FORWARD),
    enum2string(c"BOTH_FORCEWALLHOLD_LEFT", BOTH_FORCEWALLHOLD_LEFT),
    enum2string(c"BOTH_FORCEWALLHOLD_BACK", BOTH_FORCEWALLHOLD_BACK),
    enum2string(c"BOTH_FORCEWALLHOLD_RIGHT", BOTH_FORCEWALLHOLD_RIGHT),
    enum2string(
        c"BOTH_FORCEWALLRELEASE_FORWARD",
        BOTH_FORCEWALLRELEASE_FORWARD,
    ),
    enum2string(c"BOTH_FORCEWALLRELEASE_LEFT", BOTH_FORCEWALLRELEASE_LEFT),
    enum2string(c"BOTH_FORCEWALLRELEASE_BACK", BOTH_FORCEWALLRELEASE_BACK),
    enum2string(c"BOTH_FORCEWALLRELEASE_RIGHT", BOTH_FORCEWALLRELEASE_RIGHT),
    enum2string(c"BOTH_A7_KICK_F", BOTH_A7_KICK_F),
    enum2string(c"BOTH_A7_KICK_B", BOTH_A7_KICK_B),
    enum2string(c"BOTH_A7_KICK_R", BOTH_A7_KICK_R),
    enum2string(c"BOTH_A7_KICK_L", BOTH_A7_KICK_L),
    enum2string(c"BOTH_A7_KICK_S", BOTH_A7_KICK_S),
    enum2string(c"BOTH_A7_KICK_BF", BOTH_A7_KICK_BF),
    enum2string(c"BOTH_A7_KICK_BF_STOP", BOTH_A7_KICK_BF_STOP),
    enum2string(c"BOTH_A7_KICK_RL", BOTH_A7_KICK_RL),
    enum2string(c"BOTH_A7_KICK_F_AIR", BOTH_A7_KICK_F_AIR),
    enum2string(c"BOTH_A7_KICK_B_AIR", BOTH_A7_KICK_B_AIR),
    enum2string(c"BOTH_A7_KICK_R_AIR", BOTH_A7_KICK_R_AIR),
    enum2string(c"BOTH_A7_KICK_L_AIR", BOTH_A7_KICK_L_AIR),
    enum2string(c"BOTH_FLIP_ATTACK7", BOTH_FLIP_ATTACK7),
    enum2string(c"BOTH_FLIP_HOLD7", BOTH_FLIP_HOLD7),
    enum2string(c"BOTH_FLIP_LAND", BOTH_FLIP_LAND),
    enum2string(c"BOTH_PULL_IMPALE_STAB", BOTH_PULL_IMPALE_STAB),
    enum2string(c"BOTH_PULL_IMPALE_SWING", BOTH_PULL_IMPALE_SWING),
    enum2string(c"BOTH_PULLED_INAIR_B", BOTH_PULLED_INAIR_B),
    enum2string(c"BOTH_PULLED_INAIR_F", BOTH_PULLED_INAIR_F),
    enum2string(c"BOTH_STABDOWN", BOTH_STABDOWN),
    enum2string(c"BOTH_STABDOWN_STAFF", BOTH_STABDOWN_STAFF),
    enum2string(c"BOTH_STABDOWN_DUAL", BOTH_STABDOWN_DUAL),
    enum2string(c"BOTH_A6_SABERPROTECT", BOTH_A6_SABERPROTECT),
    enum2string(c"BOTH_A7_SOULCAL", BOTH_A7_SOULCAL),
    enum2string(c"BOTH_A1_SPECIAL", BOTH_A1_SPECIAL),
    enum2string(c"BOTH_A2_SPECIAL", BOTH_A2_SPECIAL),
    enum2string(c"BOTH_A3_SPECIAL", BOTH_A3_SPECIAL),
    enum2string(c"BOTH_ROLL_STAB", BOTH_ROLL_STAB),
    //# #sep ENUM2STRING(BOTH_ STANDING
    enum2string(c"BOTH_STAND1", BOTH_STAND1), // # Standing idle, no weapon, hands down
    enum2string(c"BOTH_STAND1IDLE1", BOTH_STAND1IDLE1), // # Random standing idle
    enum2string(c"BOTH_STAND2", BOTH_STAND2), // # Standing idle with a saber
    enum2string(c"BOTH_STAND2IDLE1", BOTH_STAND2IDLE1), // # Random standing idle
    enum2string(c"BOTH_STAND2IDLE2", BOTH_STAND2IDLE2),
    enum2string(c"BOTH_STAND3", BOTH_STAND3), // # Standing idle with 2-handed weapon
    enum2string(c"BOTH_STAND3IDLE1", BOTH_STAND3IDLE1), // # Random standing idle
    enum2string(c"BOTH_STAND4", BOTH_STAND4), // # hands clasp behind back
    enum2string(c"BOTH_STAND5", BOTH_STAND5), // # standing idle, no weapon, hand down, back straight
    enum2string(c"BOTH_STAND5IDLE1", BOTH_STAND5IDLE1), // # Random standing idle
    enum2string(c"BOTH_STAND6", BOTH_STAND6), // # one handed), gun at side), relaxed stand
    enum2string(c"BOTH_STAND8", BOTH_STAND8), // # both hands on hips (male)
    enum2string(c"BOTH_STAND1TO2", BOTH_STAND1TO2), // # Transition from stand1 to stand2
    enum2string(c"BOTH_STAND2TO1", BOTH_STAND2TO1), // # Transition from stand2 to stand1
    enum2string(c"BOTH_STAND2TO4", BOTH_STAND2TO4), // # Transition from stand2 to stand4
    enum2string(c"BOTH_STAND4TO2", BOTH_STAND4TO2), // # Transition from stand4 to stand2
    enum2string(c"BOTH_STAND4TOATTACK2", BOTH_STAND4TOATTACK2), // # relaxed stand to 1-handed pistol ready
    enum2string(c"BOTH_STANDUP2", BOTH_STANDUP2), // # Luke standing up from his meditation platform (cin # 37)
    enum2string(c"BOTH_STAND5TOSIT3", BOTH_STAND5TOSIT3), // # transition from stand 5 to sit 3
    enum2string(c"BOTH_STAND1TOSTAND5", BOTH_STAND1TOSTAND5), // # Transition from stand1 to stand5
    enum2string(c"BOTH_STAND5TOSTAND1", BOTH_STAND5TOSTAND1), // # Transition from stand5 to stand1
    enum2string(c"BOTH_STAND5TOAIM", BOTH_STAND5TOAIM), // # Transition of Kye aiming his gun at Desann (cin #9)
    enum2string(c"BOTH_STAND5STARTLEDLOOKLEFT", BOTH_STAND5STARTLEDLOOKLEFT), // # Kyle turning to watch the bridge drop (cin #9)
    enum2string(
        c"BOTH_STARTLEDLOOKLEFTTOSTAND5",
        BOTH_STARTLEDLOOKLEFTTOSTAND5,
    ), // # Kyle returning to stand 5 from watching the bridge drop (cin #9)
    enum2string(c"BOTH_STAND5TOSTAND8", BOTH_STAND5TOSTAND8), // # Transition from stand5 to stand8
    enum2string(c"BOTH_STAND7TOSTAND8", BOTH_STAND7TOSTAND8), // # Tavion putting hands on back of chair (cin #11)
    enum2string(c"BOTH_STAND8TOSTAND5", BOTH_STAND8TOSTAND5), // # Transition from stand8 to stand5
    enum2string(c"BOTH_STAND9", BOTH_STAND9), // # Kyle's standing idle, no weapon, hands down
    enum2string(c"BOTH_STAND9IDLE1", BOTH_STAND9IDLE1), // # Kyle's random standing idle
    enum2string(c"BOTH_STAND5SHIFTWEIGHT", BOTH_STAND5SHIFTWEIGHT), // # Weightshift from stand5 to side and back to stand5
    enum2string(c"BOTH_STAND5SHIFTWEIGHTSTART", BOTH_STAND5SHIFTWEIGHTSTART), // # From stand5 to side
    enum2string(c"BOTH_STAND5SHIFTWEIGHTSTOP", BOTH_STAND5SHIFTWEIGHTSTOP), // # From side to stand5
    enum2string(c"BOTH_STAND5TURNLEFTSTART", BOTH_STAND5TURNLEFTSTART), // # Start turning left from stand5
    enum2string(c"BOTH_STAND5TURNLEFTSTOP", BOTH_STAND5TURNLEFTSTOP), // # Stop turning left from stand5
    enum2string(c"BOTH_STAND5TURNRIGHTSTART", BOTH_STAND5TURNRIGHTSTART), // # Start turning right from stand5
    enum2string(c"BOTH_STAND5TURNRIGHTSTOP", BOTH_STAND5TURNRIGHTSTOP), // # Stop turning right from stand5
    enum2string(c"BOTH_STAND5LOOK180LEFTSTART", BOTH_STAND5LOOK180LEFTSTART), // # Start looking over left shoulder (cin #17)
    enum2string(c"BOTH_STAND5LOOK180LEFTSTOP", BOTH_STAND5LOOK180LEFTSTOP), // # Stop looking over left shoulder (cin #17)
    enum2string(c"BOTH_CONSOLE1START", BOTH_CONSOLE1START),                 // # typing at a console
    enum2string(c"BOTH_CONSOLE1", BOTH_CONSOLE1),                           // # typing at a console
    enum2string(c"BOTH_CONSOLE1STOP", BOTH_CONSOLE1STOP),                   // # typing at a console
    enum2string(c"BOTH_CONSOLE2START", BOTH_CONSOLE2START), // # typing at a console with comm link in hand (cin #5)
    enum2string(c"BOTH_CONSOLE2", BOTH_CONSOLE2), // # typing at a console with comm link in hand (cin #5)
    enum2string(c"BOTH_CONSOLE2STOP", BOTH_CONSOLE2STOP), // # typing at a console with comm link in hand (cin #5)
    enum2string(c"BOTH_CONSOLE2HOLDCOMSTART", BOTH_CONSOLE2HOLDCOMSTART), // # lean in to type at console while holding comm link in hand (cin #5)
    enum2string(c"BOTH_CONSOLE2HOLDCOMSTOP", BOTH_CONSOLE2HOLDCOMSTOP), // # lean away after typing at console while holding comm link in hand (cin #5)
    enum2string(c"BOTH_GUARD_LOOKAROUND1", BOTH_GUARD_LOOKAROUND1), // # Cradling weapon and looking around
    enum2string(c"BOTH_GUARD_IDLE1", BOTH_GUARD_IDLE1), // # Cradling weapon and standing
    enum2string(c"BOTH_GESTURE1", BOTH_GESTURE1),       // # Generic gesture), non-specific
    enum2string(c"BOTH_GESTURE2", BOTH_GESTURE2),       // # Generic gesture), non-specific
    enum2string(c"BOTH_WALK1TALKCOMM1", BOTH_WALK1TALKCOMM1), // # Talking into coom link while walking
    enum2string(c"BOTH_TALK1", BOTH_TALK1),                   // # Generic talk anim
    enum2string(c"BOTH_TALK2", BOTH_TALK2),                   // # Generic talk anim
    enum2string(c"BOTH_TALKCOMM1START", BOTH_TALKCOMM1START), // # Start talking into a comm link
    enum2string(c"BOTH_TALKCOMM1", BOTH_TALKCOMM1),           // # Talking into a comm link
    enum2string(c"BOTH_TALKCOMM1STOP", BOTH_TALKCOMM1STOP),   // # Stop talking into a comm link
    enum2string(c"BOTH_TALKGESTURE1", BOTH_TALKGESTURE1),     // # Generic talk anim
    enum2string(c"BOTH_HEADTILTLSTART", BOTH_HEADTILTLSTART), // # Head tilt to left
    enum2string(c"BOTH_HEADTILTLSTOP", BOTH_HEADTILTLSTOP),   // # Head tilt to left
    enum2string(c"BOTH_HEADTILTRSTART", BOTH_HEADTILTRSTART), // # Head tilt to right
    enum2string(c"BOTH_HEADTILTRSTOP", BOTH_HEADTILTRSTOP),   // # Head tilt to right
    enum2string(c"BOTH_HEADNOD", BOTH_HEADNOD),               // # Head shake YES
    enum2string(c"BOTH_HEADSHAKE", BOTH_HEADSHAKE),           // # Head shake NO
    enum2string(c"BOTH_SIT2HEADTILTLSTART", BOTH_SIT2HEADTILTLSTART), // # Head tilt to left from seated position 2
    enum2string(c"BOTH_SIT2HEADTILTLSTOP", BOTH_SIT2HEADTILTLSTOP), // # Head tilt to left from seated position 2
    enum2string(c"BOTH_REACH1START", BOTH_REACH1START), // # Monmothma reaching for crystal
    enum2string(c"BOTH_REACH1STOP", BOTH_REACH1STOP),   // # Monmothma reaching for crystal
    enum2string(c"BOTH_COME_ON1", BOTH_COME_ON1),       // # Jan gesturing to Kyle (cin #32a)
    enum2string(c"BOTH_STEADYSELF1", BOTH_STEADYSELF1), // # Jan trying to keep footing (cin #32a) Kyle (cin#5)
    enum2string(c"BOTH_STEADYSELF1END", BOTH_STEADYSELF1END), // # Return hands to side from STEADSELF1 Kyle (cin#5)
    enum2string(c"BOTH_SILENCEGESTURE1", BOTH_SILENCEGESTURE1), // # Luke silencing Kyle with a raised hand (cin #37)
    enum2string(c"BOTH_REACHFORSABER1", BOTH_REACHFORSABER1), // # Luke holding hand out for Kyle's saber (cin #37)
    enum2string(c"BOTH_SABERKILLER1", BOTH_SABERKILLER1), // # Tavion about to strike Jan with saber (cin #9)
    enum2string(c"BOTH_SABERKILLEE1", BOTH_SABERKILLEE1), // # Jan about to be struck by Tavion with saber (cin #9)
    enum2string(c"BOTH_HUGGER1", BOTH_HUGGER1),           // # Kyle hugging Jan (cin #29)
    enum2string(c"BOTH_HUGGERSTOP1", BOTH_HUGGERSTOP1), // # Kyle stop hugging Jan but don't let her go (cin #29)
    enum2string(c"BOTH_HUGGEE1", BOTH_HUGGEE1),         // # Jan being hugged (cin #29)
    enum2string(c"BOTH_HUGGEESTOP1", BOTH_HUGGEESTOP1), // # Jan stop being hugged but don't let go (cin #29)
    enum2string(c"BOTH_SABERTHROW1START", BOTH_SABERTHROW1START), // # Desann throwing his light saber (cin #26)
    enum2string(c"BOTH_SABERTHROW1STOP", BOTH_SABERTHROW1STOP), // # Desann throwing his light saber (cin #26)
    enum2string(c"BOTH_SABERTHROW2START", BOTH_SABERTHROW2START), // # Kyle throwing his light saber (cin #32)
    enum2string(c"BOTH_SABERTHROW2STOP", BOTH_SABERTHROW2STOP), // # Kyle throwing his light saber (cin #32)
    //# #sep ENUM2STRING(BOTH_ SITTING/CROUCHING
    enum2string(c"BOTH_SIT1", BOTH_SIT1), // # Normal chair sit.
    enum2string(c"BOTH_SIT2", BOTH_SIT2), // # Lotus position.
    enum2string(c"BOTH_SIT3", BOTH_SIT3), // # Sitting in tired position), elbows on knees
    enum2string(c"BOTH_SIT2TOSTAND5", BOTH_SIT2TOSTAND5), // # Transition from sit 2 to stand 5
    enum2string(c"BOTH_STAND5TOSIT2", BOTH_STAND5TOSIT2), // # Transition from stand 5 to sit 2
    enum2string(c"BOTH_SIT2TOSIT4", BOTH_SIT2TOSIT4), // # Trans from sit2 to sit4 (cin #12) Luke leaning back from lotus position.
    enum2string(c"BOTH_SIT3TOSTAND5", BOTH_SIT3TOSTAND5), // # transition from sit 3 to stand 5
    enum2string(c"BOTH_CROUCH1", BOTH_CROUCH1),       // # Transition from standing to crouch
    enum2string(c"BOTH_CROUCH1IDLE", BOTH_CROUCH1IDLE), // # Crouching idle
    enum2string(c"BOTH_CROUCH1WALK", BOTH_CROUCH1WALK), // # Walking while crouched
    enum2string(c"BOTH_CROUCH1WALKBACK", BOTH_CROUCH1WALKBACK), // # Walking while crouched
    enum2string(c"BOTH_UNCROUCH1", BOTH_UNCROUCH1),   // # Transition from crouch to standing
    enum2string(c"BOTH_CROUCH2TOSTAND1", BOTH_CROUCH2TOSTAND1), // # going from crouch2 to stand1
    enum2string(c"BOTH_CROUCH3", BOTH_CROUCH3),       // # Desann crouching down to Kyle (cin 9)
    enum2string(c"BOTH_UNCROUCH3", BOTH_UNCROUCH3),   // # Desann uncrouching down to Kyle (cin 9)
    enum2string(c"BOTH_CROUCH4", BOTH_CROUCH4),       // # Slower version of crouch1 for cinematics
    enum2string(c"BOTH_UNCROUCH4", BOTH_UNCROUCH4), // # Slower version of uncrouch1 for cinematics
    enum2string(c"BOTH_GUNSIT1", BOTH_GUNSIT1),     // # sitting on an emplaced gun.
    // Swoop Vehicle animations.
    //* #sep BOTH_ SWOOP ANIMS
    enum2string(c"BOTH_VS_MOUNT_L", BOTH_VS_MOUNT_L), // # Mount from left
    enum2string(c"BOTH_VS_DISMOUNT_L", BOTH_VS_DISMOUNT_L), // # Dismount to left
    enum2string(c"BOTH_VS_MOUNT_R", BOTH_VS_MOUNT_R), // # Mount from  right (symmetry)
    enum2string(c"BOTH_VS_DISMOUNT_R", BOTH_VS_DISMOUNT_R), // # Dismount to  right (symmetry)
    enum2string(c"BOTH_VS_MOUNTJUMP_L", BOTH_VS_MOUNTJUMP_L), // #
    enum2string(c"BOTH_VS_MOUNTTHROW", BOTH_VS_MOUNTTHROW), // # Land on an occupied vehicle & throw off current pilot
    enum2string(c"BOTH_VS_MOUNTTHROW_L", BOTH_VS_MOUNTTHROW_L), // # Land on an occupied vehicle & throw off current pilot
    enum2string(c"BOTH_VS_MOUNTTHROW_R", BOTH_VS_MOUNTTHROW_R), // # Land on an occupied vehicle & throw off current pilot
    enum2string(c"BOTH_VS_MOUNTTHROWEE", BOTH_VS_MOUNTTHROWEE), // # Current pilot getting thrown off by another guy
    enum2string(c"BOTH_VS_LOOKLEFT", BOTH_VS_LOOKLEFT), // # Turn & Look behind and to the left (no weapon)
    enum2string(c"BOTH_VS_LOOKRIGHT", BOTH_VS_LOOKRIGHT), // # Turn & Look behind and to the right (no weapon)
    enum2string(c"BOTH_VS_TURBO", BOTH_VS_TURBO),         // # Hit The Turbo Button
    enum2string(c"BOTH_VS_REV", BOTH_VS_REV),             // # Player looks back as swoop reverses
    enum2string(c"BOTH_VS_AIR", BOTH_VS_AIR), // # Player stands up when swoop is airborn
    enum2string(c"BOTH_VS_AIR_G", BOTH_VS_AIR_G), // # "" with Gun
    enum2string(c"BOTH_VS_AIR_SL", BOTH_VS_AIR_SL), // # "" with Saber Left
    enum2string(c"BOTH_VS_AIR_SR", BOTH_VS_AIR_SR), // # "" with Saber Right
    enum2string(c"BOTH_VS_LAND", BOTH_VS_LAND), // # Player bounces down when swoop lands
    enum2string(c"BOTH_VS_LAND_G", BOTH_VS_LAND_G), // #  "" with Gun
    enum2string(c"BOTH_VS_LAND_SL", BOTH_VS_LAND_SL), // #  "" with Saber Left
    enum2string(c"BOTH_VS_LAND_SR", BOTH_VS_LAND_SR), // #  "" with Saber Right
    enum2string(c"BOTH_VS_IDLE", BOTH_VS_IDLE), // # Sit
    enum2string(c"BOTH_VS_IDLE_G", BOTH_VS_IDLE_G), // # Sit (gun)
    enum2string(c"BOTH_VS_IDLE_SL", BOTH_VS_IDLE_SL), // # Sit (saber left)
    enum2string(c"BOTH_VS_IDLE_SR", BOTH_VS_IDLE_SR), // # Sit (saber right)
    enum2string(c"BOTH_VS_LEANL", BOTH_VS_LEANL), // # Lean left
    enum2string(c"BOTH_VS_LEANL_G", BOTH_VS_LEANL_G), // # Lean left (gun)
    enum2string(c"BOTH_VS_LEANL_SL", BOTH_VS_LEANL_SL), // # Lean left (saber left)
    enum2string(c"BOTH_VS_LEANL_SR", BOTH_VS_LEANL_SR), // # Lean left (saber right)
    enum2string(c"BOTH_VS_LEANR", BOTH_VS_LEANR), // # Lean right
    enum2string(c"BOTH_VS_LEANR_G", BOTH_VS_LEANR_G), // # Lean right (gun)
    enum2string(c"BOTH_VS_LEANR_SL", BOTH_VS_LEANR_SL), // # Lean right (saber left)
    enum2string(c"BOTH_VS_LEANR_SR", BOTH_VS_LEANR_SR), // # Lean right (saber right)
    enum2string(c"BOTH_VS_ATL_S", BOTH_VS_ATL_S), // # Attack left with saber
    enum2string(c"BOTH_VS_ATR_S", BOTH_VS_ATR_S), // # Attack right with saber
    enum2string(c"BOTH_VS_ATR_TO_L_S", BOTH_VS_ATR_TO_L_S), // # Attack toss saber from right to left hand
    enum2string(c"BOTH_VS_ATL_TO_R_S", BOTH_VS_ATL_TO_R_S), // # Attack toss saber from left to right hand
    enum2string(c"BOTH_VS_ATR_G", BOTH_VS_ATR_G),           // # Attack right with gun (90)
    enum2string(c"BOTH_VS_ATL_G", BOTH_VS_ATL_G),           // # Attack left with gun (90)
    enum2string(c"BOTH_VS_ATF_G", BOTH_VS_ATF_G),           // # Attack forward with gun
    enum2string(c"BOTH_VS_PAIN1", BOTH_VS_PAIN1),           // # Pain
    // Added 12/04/02 by Aurelio.
    //* #sep BOTH_ TAUNTAUN ANIMS
    enum2string(c"BOTH_VT_MOUNT_L", BOTH_VT_MOUNT_L), // # Mount from left
    enum2string(c"BOTH_VT_MOUNT_R", BOTH_VT_MOUNT_R), // # Mount from right
    enum2string(c"BOTH_VT_MOUNT_B", BOTH_VT_MOUNT_B), // # Mount from air, behind
    enum2string(c"BOTH_VT_DISMOUNT", BOTH_VT_DISMOUNT), // # Dismount for tauntaun
    enum2string(c"BOTH_VT_DISMOUNT_L", BOTH_VT_DISMOUNT_L), // # Dismount to tauntauns left
    enum2string(c"BOTH_VT_DISMOUNT_R", BOTH_VT_DISMOUNT_R), // # Dismount to tauntauns right (symmetry)
    enum2string(c"BOTH_VT_WALK_FWD", BOTH_VT_WALK_FWD),     // # Walk forward
    enum2string(c"BOTH_VT_WALK_REV", BOTH_VT_WALK_REV),     // # Walk backward
    enum2string(c"BOTH_VT_WALK_FWD_L", BOTH_VT_WALK_FWD_L), // # walk lean left
    enum2string(c"BOTH_VT_WALK_FWD_R", BOTH_VT_WALK_FWD_R), // # walk lean right
    enum2string(c"BOTH_VT_RUN_FWD", BOTH_VT_RUN_FWD),       // # Run forward
    enum2string(c"BOTH_VT_RUN_REV", BOTH_VT_RUN_REV), // # Look backwards while running (not weapon specific)
    enum2string(c"BOTH_VT_RUN_FWD_L", BOTH_VT_RUN_FWD_L), // # run lean left
    enum2string(c"BOTH_VT_RUN_FWD_R", BOTH_VT_RUN_FWD_R), // # run lean right
    enum2string(c"BOTH_VT_SLIDEF", BOTH_VT_SLIDEF),   // # Tauntaun slides forward with abrupt stop
    enum2string(c"BOTH_VT_AIR", BOTH_VT_AIR),         // # Tauntaun jump
    enum2string(c"BOTH_VT_ATB", BOTH_VT_ATB),         // # Tauntaun tail swipe
    enum2string(c"BOTH_VT_PAIN1", BOTH_VT_PAIN1),     // # Pain
    enum2string(c"BOTH_VT_DEATH1", BOTH_VT_DEATH1),   // # Die
    enum2string(c"BOTH_VT_STAND", BOTH_VT_STAND),     // # Stand still and breath
    enum2string(c"BOTH_VT_BUCK", BOTH_VT_BUCK),       // # Tauntaun bucking loop animation
    enum2string(c"BOTH_VT_LAND", BOTH_VT_LAND),       // # Player bounces down when tauntaun lands
    enum2string(c"BOTH_VT_TURBO", BOTH_VT_TURBO),     // # Hit The Turbo Button
    enum2string(c"BOTH_VT_IDLE_SL", BOTH_VT_IDLE_SL), // # Sit (saber left)
    enum2string(c"BOTH_VT_IDLE_SR", BOTH_VT_IDLE_SR), // # Sit (saber right)
    enum2string(c"BOTH_VT_IDLE", BOTH_VT_IDLE),       // # Sit with no weapon selected
    enum2string(c"BOTH_VT_IDLE1", BOTH_VT_IDLE1),     // # Sit with no weapon selected
    enum2string(c"BOTH_VT_IDLE_S", BOTH_VT_IDLE_S),   // # Sit with saber selected
    enum2string(c"BOTH_VT_IDLE_G", BOTH_VT_IDLE_G),   // # Sit with gun selected
    enum2string(c"BOTH_VT_IDLE_T", BOTH_VT_IDLE_T),   // # Sit with thermal grenade selected
    enum2string(c"BOTH_VT_ATL_S", BOTH_VT_ATL_S),     // # Attack left with saber
    enum2string(c"BOTH_VT_ATR_S", BOTH_VT_ATR_S),     // # Attack right with saber
    enum2string(c"BOTH_VT_ATR_TO_L_S", BOTH_VT_ATR_TO_L_S), // # Attack toss saber from right to left hand
    enum2string(c"BOTH_VT_ATL_TO_R_S", BOTH_VT_ATL_TO_R_S), // # Attack toss saber from left to right hand
    enum2string(c"BOTH_VT_ATR_G", BOTH_VT_ATR_G),           // # Attack right with gun (90)
    enum2string(c"BOTH_VT_ATL_G", BOTH_VT_ATL_G),           // # Attack left with gun (90)
    enum2string(c"BOTH_VT_ATF_G", BOTH_VT_ATF_G),           // # Attack forward with gun
    // Added 2/26/02 by Aurelio.
    //* #sep BOTH_ FIGHTER ANIMS
    enum2string(c"BOTH_GEARS_OPEN", BOTH_GEARS_OPEN),
    enum2string(c"BOTH_GEARS_CLOSE", BOTH_GEARS_CLOSE),
    enum2string(c"BOTH_WINGS_OPEN", BOTH_WINGS_OPEN),
    enum2string(c"BOTH_WINGS_CLOSE", BOTH_WINGS_CLOSE),
    ///////////////////////////////////
    enum2string(c"BOTH_DEATH14_UNGRIP", BOTH_DEATH14_UNGRIP), // # Desann's end death (cin #35)
    enum2string(c"BOTH_DEATH14_SITUP", BOTH_DEATH14_SITUP), // # Tavion sitting up after having been thrown (cin #23)
    enum2string(c"BOTH_KNEES1", BOTH_KNEES1),               // # Tavion on her knees
    enum2string(c"BOTH_KNEES2", BOTH_KNEES2),               // # Tavion on her knees looking down
    enum2string(c"BOTH_KNEES2TO1", BOTH_KNEES2TO1),         // # Transition of KNEES2 to KNEES1
    //# #sep ENUM2STRING(BOTH_ MOVING
    enum2string(c"BOTH_WALK1", BOTH_WALK1), // # Normal walk
    enum2string(c"BOTH_WALK2", BOTH_WALK2), // # Normal walk
    enum2string(c"BOTH_WALK_STAFF", BOTH_WALK_STAFF), // # Walk with saberstaff turned on
    enum2string(c"BOTH_WALKBACK_STAFF", BOTH_WALKBACK_STAFF), // # Walk backwards with saberstaff turned on
    enum2string(c"BOTH_WALK_DUAL", BOTH_WALK_DUAL),           // # Walk with dual turned on
    enum2string(c"BOTH_WALKBACK_DUAL", BOTH_WALKBACK_DUAL), // # Walk backwards with dual turned on
    enum2string(c"BOTH_WALK5", BOTH_WALK5),                 // # Tavion taunting Kyle (cin 22)
    enum2string(c"BOTH_WALK6", BOTH_WALK6),                 // # Slow walk for Luke (cin 12)
    enum2string(c"BOTH_WALK7", BOTH_WALK7),                 // # Fast walk
    enum2string(c"BOTH_RUN1", BOTH_RUN1),                   // # Full run
    enum2string(c"BOTH_RUN1START", BOTH_RUN1START),         // # Start into full run1
    enum2string(c"BOTH_RUN1STOP", BOTH_RUN1STOP),           // # Stop from full run1
    enum2string(c"BOTH_RUN2", BOTH_RUN2),                   // # Full run
    enum2string(c"BOTH_RUN1TORUN2", BOTH_RUN1TORUN2),       // # Wampa run anim transition
    enum2string(c"BOTH_RUN2TORUN1", BOTH_RUN2TORUN1),       // # Wampa run anim transition
    enum2string(c"BOTH_RUN4", BOTH_RUN4),                   // # Jawa run
    enum2string(c"BOTH_RUN_STAFF", BOTH_RUN_STAFF),         // # Run with saberstaff turned on
    enum2string(c"BOTH_RUNBACK_STAFF", BOTH_RUNBACK_STAFF), // # Run backwards with saberstaff turned on
    enum2string(c"BOTH_RUN_DUAL", BOTH_RUN_DUAL),           // # Run with dual turned on
    enum2string(c"BOTH_RUNBACK_DUAL", BOTH_RUNBACK_DUAL),   // # Run backwards with dual turned on
    enum2string(c"BOTH_STRAFE_LEFT1", BOTH_STRAFE_LEFT1),   // # Sidestep left), should loop
    enum2string(c"BOTH_STRAFE_RIGHT1", BOTH_STRAFE_RIGHT1), // # Sidestep right), should loop
    enum2string(c"BOTH_RUNSTRAFE_LEFT1", BOTH_RUNSTRAFE_LEFT1), // # Sidestep left), should loop
    enum2string(c"BOTH_RUNSTRAFE_RIGHT1", BOTH_RUNSTRAFE_RIGHT1), // # Sidestep right), should loop
    enum2string(c"BOTH_TURN_LEFT1", BOTH_TURN_LEFT1),       // # Turn left), should loop
    enum2string(c"BOTH_TURN_RIGHT1", BOTH_TURN_RIGHT1),     // # Turn right), should loop
    enum2string(c"BOTH_TURNSTAND1", BOTH_TURNSTAND1),       // # Turn from STAND1 position
    enum2string(c"BOTH_TURNSTAND2", BOTH_TURNSTAND2),       // # Turn from STAND2 position
    enum2string(c"BOTH_TURNSTAND3", BOTH_TURNSTAND3),       // # Turn from STAND3 position
    enum2string(c"BOTH_TURNSTAND4", BOTH_TURNSTAND4),       // # Turn from STAND4 position
    enum2string(c"BOTH_TURNSTAND5", BOTH_TURNSTAND5),       // # Turn from STAND5 position
    enum2string(c"BOTH_TURNCROUCH1", BOTH_TURNCROUCH1),     // # Turn from CROUCH1 position
    enum2string(c"BOTH_WALKBACK1", BOTH_WALKBACK1),         // # Walk1 backwards
    enum2string(c"BOTH_WALKBACK2", BOTH_WALKBACK2),         // # Walk2 backwards
    enum2string(c"BOTH_RUNBACK1", BOTH_RUNBACK1),           // # Run1 backwards
    enum2string(c"BOTH_RUNBACK2", BOTH_RUNBACK2),           // # Run1 backwards
    //# #sep BOTH_ JUMPING
    enum2string(c"BOTH_JUMP1", BOTH_JUMP1), // # Jump - wind-up and leave ground
    enum2string(c"BOTH_INAIR1", BOTH_INAIR1), // # In air loop (from jump)
    enum2string(c"BOTH_LAND1", BOTH_LAND1), // # Landing (from in air loop)
    enum2string(c"BOTH_LAND2", BOTH_LAND2), // # Landing Hard (from a great height)
    enum2string(c"BOTH_JUMPBACK1", BOTH_JUMPBACK1), // # Jump backwards - wind-up and leave ground
    enum2string(c"BOTH_INAIRBACK1", BOTH_INAIRBACK1), // # In air loop (from jump back)
    enum2string(c"BOTH_LANDBACK1", BOTH_LANDBACK1), // # Landing backwards(from in air loop)
    enum2string(c"BOTH_JUMPLEFT1", BOTH_JUMPLEFT1), // # Jump left - wind-up and leave ground
    enum2string(c"BOTH_INAIRLEFT1", BOTH_INAIRLEFT1), // # In air loop (from jump left)
    enum2string(c"BOTH_LANDLEFT1", BOTH_LANDLEFT1), // # Landing left(from in air loop)
    enum2string(c"BOTH_JUMPRIGHT1", BOTH_JUMPRIGHT1), // # Jump right - wind-up and leave ground
    enum2string(c"BOTH_INAIRRIGHT1", BOTH_INAIRRIGHT1), // # In air loop (from jump right)
    enum2string(c"BOTH_LANDRIGHT1", BOTH_LANDRIGHT1), // # Landing right(from in air loop)
    enum2string(c"BOTH_FORCEJUMP1", BOTH_FORCEJUMP1), // # Jump - wind-up and leave ground
    enum2string(c"BOTH_FORCEINAIR1", BOTH_FORCEINAIR1), // # In air loop (from jump)
    enum2string(c"BOTH_FORCELAND1", BOTH_FORCELAND1), // # Landing (from in air loop)
    enum2string(c"BOTH_FORCEJUMPBACK1", BOTH_FORCEJUMPBACK1), // # Jump backwards - wind-up and leave ground
    enum2string(c"BOTH_FORCEINAIRBACK1", BOTH_FORCEINAIRBACK1), // # In air loop (from jump back)
    enum2string(c"BOTH_FORCELANDBACK1", BOTH_FORCELANDBACK1), // # Landing backwards(from in air loop)
    enum2string(c"BOTH_FORCEJUMPLEFT1", BOTH_FORCEJUMPLEFT1), // # Jump left - wind-up and leave ground
    enum2string(c"BOTH_FORCEINAIRLEFT1", BOTH_FORCEINAIRLEFT1), // # In air loop (from jump left)
    enum2string(c"BOTH_FORCELANDLEFT1", BOTH_FORCELANDLEFT1), // # Landing left(from in air loop)
    enum2string(c"BOTH_FORCEJUMPRIGHT1", BOTH_FORCEJUMPRIGHT1), // # Jump right - wind-up and leave ground
    enum2string(c"BOTH_FORCEINAIRRIGHT1", BOTH_FORCEINAIRRIGHT1), // # In air loop (from jump right)
    enum2string(c"BOTH_FORCELANDRIGHT1", BOTH_FORCELANDRIGHT1), // # Landing right(from in air loop)
    //# #sep BOTH_ ACROBATICS
    enum2string(c"BOTH_FLIP_F", BOTH_FLIP_F), // # Flip forward
    enum2string(c"BOTH_FLIP_B", BOTH_FLIP_B), // # Flip backwards
    enum2string(c"BOTH_FLIP_L", BOTH_FLIP_L), // # Flip left
    enum2string(c"BOTH_FLIP_R", BOTH_FLIP_R), // # Flip right
    enum2string(c"BOTH_ROLL_F", BOTH_ROLL_F), // # Roll forward
    enum2string(c"BOTH_ROLL_B", BOTH_ROLL_B), // # Roll backward
    enum2string(c"BOTH_ROLL_L", BOTH_ROLL_L), // # Roll left
    enum2string(c"BOTH_ROLL_R", BOTH_ROLL_R), // # Roll right
    enum2string(c"BOTH_HOP_F", BOTH_HOP_F),   // # quickstep forward
    enum2string(c"BOTH_HOP_B", BOTH_HOP_B),   // # quickstep backwards
    enum2string(c"BOTH_HOP_L", BOTH_HOP_L),   // # quickstep left
    enum2string(c"BOTH_HOP_R", BOTH_HOP_R),   // # quickstep right
    enum2string(c"BOTH_DODGE_FL", BOTH_DODGE_FL), // # lean-dodge forward left
    enum2string(c"BOTH_DODGE_FR", BOTH_DODGE_FR), // # lean-dodge forward right
    enum2string(c"BOTH_DODGE_BL", BOTH_DODGE_BL), // # lean-dodge backwards left
    enum2string(c"BOTH_DODGE_BR", BOTH_DODGE_BR), // # lean-dodge backwards right
    enum2string(c"BOTH_DODGE_L", BOTH_DODGE_L), // # lean-dodge left
    enum2string(c"BOTH_DODGE_R", BOTH_DODGE_R), // # lean-dodge right
    enum2string(c"BOTH_DODGE_HOLD_FL", BOTH_DODGE_HOLD_FL), // # lean-dodge pose forward left
    enum2string(c"BOTH_DODGE_HOLD_FR", BOTH_DODGE_HOLD_FR), // # lean-dodge pose forward right
    enum2string(c"BOTH_DODGE_HOLD_BL", BOTH_DODGE_HOLD_BL), // # lean-dodge pose backwards left
    enum2string(c"BOTH_DODGE_HOLD_BR", BOTH_DODGE_HOLD_BR), // # lean-dodge pose backwards right
    enum2string(c"BOTH_DODGE_HOLD_L", BOTH_DODGE_HOLD_L), // # lean-dodge pose left
    enum2string(c"BOTH_DODGE_HOLD_R", BOTH_DODGE_HOLD_R), // # lean-dodge pose right
    //MP taunt anims
    enum2string(c"BOTH_ENGAGETAUNT", BOTH_ENGAGETAUNT),
    enum2string(c"BOTH_BOW", BOTH_BOW),
    enum2string(c"BOTH_MEDITATE", BOTH_MEDITATE),
    enum2string(c"BOTH_MEDITATE_END", BOTH_MEDITATE_END),
    enum2string(c"BOTH_SHOWOFF_FAST", BOTH_SHOWOFF_FAST),
    enum2string(c"BOTH_SHOWOFF_MEDIUM", BOTH_SHOWOFF_MEDIUM),
    enum2string(c"BOTH_SHOWOFF_STRONG", BOTH_SHOWOFF_STRONG),
    enum2string(c"BOTH_SHOWOFF_DUAL", BOTH_SHOWOFF_DUAL),
    enum2string(c"BOTH_SHOWOFF_STAFF", BOTH_SHOWOFF_STAFF),
    enum2string(c"BOTH_VICTORY_FAST", BOTH_VICTORY_FAST),
    enum2string(c"BOTH_VICTORY_MEDIUM", BOTH_VICTORY_MEDIUM),
    enum2string(c"BOTH_VICTORY_STRONG", BOTH_VICTORY_STRONG),
    enum2string(c"BOTH_VICTORY_DUAL", BOTH_VICTORY_DUAL),
    enum2string(c"BOTH_VICTORY_STAFF", BOTH_VICTORY_STAFF),
    //other saber/acro anims
    enum2string(c"BOTH_ARIAL_LEFT", BOTH_ARIAL_LEFT), // #
    enum2string(c"BOTH_ARIAL_RIGHT", BOTH_ARIAL_RIGHT), // #
    enum2string(c"BOTH_CARTWHEEL_LEFT", BOTH_CARTWHEEL_LEFT), // #
    enum2string(c"BOTH_CARTWHEEL_RIGHT", BOTH_CARTWHEEL_RIGHT), // #
    enum2string(c"BOTH_FLIP_LEFT", BOTH_FLIP_LEFT),   // #
    enum2string(c"BOTH_FLIP_BACK1", BOTH_FLIP_BACK1), // #
    enum2string(c"BOTH_FLIP_BACK2", BOTH_FLIP_BACK2), // #
    enum2string(c"BOTH_FLIP_BACK3", BOTH_FLIP_BACK3), // #
    enum2string(c"BOTH_BUTTERFLY_LEFT", BOTH_BUTTERFLY_LEFT), // #
    enum2string(c"BOTH_BUTTERFLY_RIGHT", BOTH_BUTTERFLY_RIGHT), // #
    enum2string(c"BOTH_WALL_RUN_RIGHT", BOTH_WALL_RUN_RIGHT), // #
    enum2string(c"BOTH_WALL_RUN_RIGHT_FLIP", BOTH_WALL_RUN_RIGHT_FLIP), // #
    enum2string(c"BOTH_WALL_RUN_RIGHT_STOP", BOTH_WALL_RUN_RIGHT_STOP), // #
    enum2string(c"BOTH_WALL_RUN_LEFT", BOTH_WALL_RUN_LEFT), // #
    enum2string(c"BOTH_WALL_RUN_LEFT_FLIP", BOTH_WALL_RUN_LEFT_FLIP), // #
    enum2string(c"BOTH_WALL_RUN_LEFT_STOP", BOTH_WALL_RUN_LEFT_STOP), // #
    enum2string(c"BOTH_WALL_FLIP_RIGHT", BOTH_WALL_FLIP_RIGHT), // #
    enum2string(c"BOTH_WALL_FLIP_LEFT", BOTH_WALL_FLIP_LEFT), // #
    enum2string(c"BOTH_KNOCKDOWN1", BOTH_KNOCKDOWN1), // # knocked backwards
    enum2string(c"BOTH_KNOCKDOWN2", BOTH_KNOCKDOWN2), // # knocked backwards hard
    enum2string(c"BOTH_KNOCKDOWN3", BOTH_KNOCKDOWN3), // #	knocked forwards
    enum2string(c"BOTH_KNOCKDOWN4", BOTH_KNOCKDOWN4), // # knocked backwards from crouch
    enum2string(c"BOTH_KNOCKDOWN5", BOTH_KNOCKDOWN5), // # dupe of 3 - will be removed
    enum2string(c"BOTH_GETUP1", BOTH_GETUP1),         // #
    enum2string(c"BOTH_GETUP2", BOTH_GETUP2),         // #
    enum2string(c"BOTH_GETUP3", BOTH_GETUP3),         // #
    enum2string(c"BOTH_GETUP4", BOTH_GETUP4),         // #
    enum2string(c"BOTH_GETUP5", BOTH_GETUP5),         // #
    enum2string(c"BOTH_GETUP_CROUCH_F1", BOTH_GETUP_CROUCH_F1), // #
    enum2string(c"BOTH_GETUP_CROUCH_B1", BOTH_GETUP_CROUCH_B1), // #
    enum2string(c"BOTH_FORCE_GETUP_F1", BOTH_FORCE_GETUP_F1), // #
    enum2string(c"BOTH_FORCE_GETUP_F2", BOTH_FORCE_GETUP_F2), // #
    enum2string(c"BOTH_FORCE_GETUP_B1", BOTH_FORCE_GETUP_B1), // #
    enum2string(c"BOTH_FORCE_GETUP_B2", BOTH_FORCE_GETUP_B2), // #
    enum2string(c"BOTH_FORCE_GETUP_B3", BOTH_FORCE_GETUP_B3), // #
    enum2string(c"BOTH_FORCE_GETUP_B4", BOTH_FORCE_GETUP_B4), // #
    enum2string(c"BOTH_FORCE_GETUP_B5", BOTH_FORCE_GETUP_B5), // #
    enum2string(c"BOTH_FORCE_GETUP_B6", BOTH_FORCE_GETUP_B6), // #
    enum2string(c"BOTH_GETUP_BROLL_B", BOTH_GETUP_BROLL_B), // #
    enum2string(c"BOTH_GETUP_BROLL_F", BOTH_GETUP_BROLL_F), // #
    enum2string(c"BOTH_GETUP_BROLL_L", BOTH_GETUP_BROLL_L), // #
    enum2string(c"BOTH_GETUP_BROLL_R", BOTH_GETUP_BROLL_R), // #
    enum2string(c"BOTH_GETUP_FROLL_B", BOTH_GETUP_FROLL_B), // #
    enum2string(c"BOTH_GETUP_FROLL_F", BOTH_GETUP_FROLL_F), // #
    enum2string(c"BOTH_GETUP_FROLL_L", BOTH_GETUP_FROLL_L), // #
    enum2string(c"BOTH_GETUP_FROLL_R", BOTH_GETUP_FROLL_R), // #
    enum2string(c"BOTH_WALL_FLIP_BACK1", BOTH_WALL_FLIP_BACK1), // #
    enum2string(c"BOTH_WALL_FLIP_BACK2", BOTH_WALL_FLIP_BACK2), // #
    enum2string(c"BOTH_SPIN1", BOTH_SPIN1),           // #
    enum2string(c"BOTH_CEILING_CLING", BOTH_CEILING_CLING), // # clinging to ceiling
    enum2string(c"BOTH_CEILING_DROP", BOTH_CEILING_DROP), // # dropping from ceiling cling
    //TESTING
    enum2string(c"BOTH_FJSS_TR_BL", BOTH_FJSS_TR_BL), // # jump spin slash tr to bl
    enum2string(c"BOTH_FJSS_TL_BR", BOTH_FJSS_TL_BR), // # jump spin slash bl to tr
    enum2string(c"BOTH_RIGHTHANDCHOPPEDOFF", BOTH_RIGHTHANDCHOPPEDOFF), // #
    enum2string(c"BOTH_DEFLECTSLASH__R__L_FIN", BOTH_DEFLECTSLASH__R__L_FIN), // #
    enum2string(c"BOTH_BASHED1", BOTH_BASHED1),       // #
    enum2string(c"BOTH_ARIAL_F1", BOTH_ARIAL_F1),     // #
    enum2string(c"BOTH_BUTTERFLY_FR1", BOTH_BUTTERFLY_FR1), // #
    enum2string(c"BOTH_BUTTERFLY_FL1", BOTH_BUTTERFLY_FL1), // #
    //NEW SABER/JEDI/FORCE ANIMS
    enum2string(c"BOTH_BACK_FLIP_UP", BOTH_BACK_FLIP_UP), // # back flip up Bonus Animation!!!!
    enum2string(c"BOTH_LOSE_SABER", BOTH_LOSE_SABER), // # player losing saber (pulled from hand by force pull 4 - Kyle?)
    enum2string(c"BOTH_STAFF_TAUNT", BOTH_STAFF_TAUNT), // # taunt saberstaff
    enum2string(c"BOTH_DUAL_TAUNT", BOTH_DUAL_TAUNT), // # taunt dual
    enum2string(c"BOTH_A6_FB", BOTH_A6_FB),           // # dual attack front/back
    enum2string(c"BOTH_A6_LR", BOTH_A6_LR),           // # dual attack left/right
    enum2string(c"BOTH_A7_HILT", BOTH_A7_HILT),       // # saber knock (alt + stand still)
    //Alora
    enum2string(c"BOTH_ALORA_SPIN", BOTH_ALORA_SPIN), // #jump spin attack	death ballet
    enum2string(c"BOTH_ALORA_FLIP_1", BOTH_ALORA_FLIP_1), // # gymnast move 1
    enum2string(c"BOTH_ALORA_FLIP_2", BOTH_ALORA_FLIP_2), // # gymnast move 2
    enum2string(c"BOTH_ALORA_FLIP_3", BOTH_ALORA_FLIP_3), // # gymnast move3
    enum2string(c"BOTH_ALORA_FLIP_B", BOTH_ALORA_FLIP_B), // # gymnast move back
    enum2string(c"BOTH_ALORA_SPIN_THROW", BOTH_ALORA_SPIN_THROW), // # dual saber throw
    enum2string(c"BOTH_ALORA_SPIN_SLASH", BOTH_ALORA_SPIN_SLASH), // # spin slash	special bonus animation!! :)
    enum2string(c"BOTH_ALORA_TAUNT", BOTH_ALORA_TAUNT),           // # special taunt
    //Rosh (Kothos battle)
    enum2string(c"BOTH_ROSH_PAIN", BOTH_ROSH_PAIN), // # hurt animation (exhausted)
    enum2string(c"BOTH_ROSH_HEAL", BOTH_ROSH_HEAL), // # healed/rejuvenated
    //Tavion
    enum2string(c"BOTH_TAVION_SCEPTERGROUND", BOTH_TAVION_SCEPTERGROUND), // # stabbing ground with sith sword shoots electricity everywhere
    enum2string(c"BOTH_TAVION_SWORDPOWER", BOTH_TAVION_SWORDPOWER), // # Tavion doing the He-Man(tm) thing
    enum2string(c"BOTH_SCEPTER_START", BOTH_SCEPTER_START), // #Point scepter and attack start
    enum2string(c"BOTH_SCEPTER_HOLD", BOTH_SCEPTER_HOLD),   // #Point scepter and attack hold
    enum2string(c"BOTH_SCEPTER_STOP", BOTH_SCEPTER_STOP),   // #Point scepter and attack stop
    //Kyle Boss
    enum2string(c"BOTH_KYLE_GRAB", BOTH_KYLE_GRAB), // # grab
    enum2string(c"BOTH_KYLE_MISS", BOTH_KYLE_MISS), // # miss
    enum2string(c"BOTH_KYLE_PA_1", BOTH_KYLE_PA_1), // # hold 1
    enum2string(c"BOTH_PLAYER_PA_1", BOTH_PLAYER_PA_1), // # player getting held 1
    enum2string(c"BOTH_KYLE_PA_2", BOTH_KYLE_PA_2), // # hold 2
    enum2string(c"BOTH_PLAYER_PA_2", BOTH_PLAYER_PA_2), // # player getting held 2
    enum2string(c"BOTH_PLAYER_PA_FLY", BOTH_PLAYER_PA_FLY), // # player getting knocked back from punch at end of hold 1
    enum2string(c"BOTH_KYLE_PA_3", BOTH_KYLE_PA_3),         // # hold 3
    enum2string(c"BOTH_PLAYER_PA_3", BOTH_PLAYER_PA_3),     // # player getting held 3
    enum2string(c"BOTH_PLAYER_PA_3_FLY", BOTH_PLAYER_PA_3_FLY), // # player getting thrown at end of hold 3
    //Rancor
    enum2string(c"BOTH_BUCK_RIDER", BOTH_BUCK_RIDER), // # Rancor bucks when someone is on him
    //WAMPA Grabbing enemy
    enum2string(c"BOTH_HOLD_START", BOTH_HOLD_START), // #
    enum2string(c"BOTH_HOLD_MISS", BOTH_HOLD_MISS),   // #
    enum2string(c"BOTH_HOLD_IDLE", BOTH_HOLD_IDLE),   // #
    enum2string(c"BOTH_HOLD_END", BOTH_HOLD_END),     // #
    enum2string(c"BOTH_HOLD_ATTACK", BOTH_HOLD_ATTACK), // #
    enum2string(c"BOTH_HOLD_SNIFF", BOTH_HOLD_SNIFF), // # Sniff the guy you're holding
    enum2string(c"BOTH_HOLD_DROP", BOTH_HOLD_DROP),   // # just drop 'em
    //BEING GRABBED BY WAMPA
    enum2string(c"BOTH_GRABBED", BOTH_GRABBED),     // #
    enum2string(c"BOTH_RELEASED", BOTH_RELEASED),   // #
    enum2string(c"BOTH_HANG_IDLE", BOTH_HANG_IDLE), // #
    enum2string(c"BOTH_HANG_ATTACK", BOTH_HANG_ATTACK), // #
    enum2string(c"BOTH_HANG_PAIN", BOTH_HANG_PAIN), // #
    //# #sep BOTH_ MISC MOVEMENT
    enum2string(c"BOTH_HIT1", BOTH_HIT1), // # Kyle hit by crate in cin #9
    enum2string(c"BOTH_LADDER_UP1", BOTH_LADDER_UP1), // # Climbing up a ladder with rungs at 16 unit intervals
    enum2string(c"BOTH_LADDER_DWN1", BOTH_LADDER_DWN1), // # Climbing down a ladder with rungs at 16 unit intervals
    enum2string(c"BOTH_LADDER_IDLE", BOTH_LADDER_IDLE), // #	Just sitting on the ladder
    //# #sep ENUM2STRING(BOTH_ FLYING IDLE
    enum2string(c"BOTH_FLY_SHIELDED", BOTH_FLY_SHIELDED), // # For sentry droid, shields in
    //# #sep BOTH_ SWIMMING
    enum2string(c"BOTH_SWIM_IDLE1", BOTH_SWIM_IDLE1), // # Swimming Idle 1
    enum2string(c"BOTH_SWIMFORWARD", BOTH_SWIMFORWARD), // # Swim forward loop
    enum2string(c"BOTH_SWIMBACKWARD", BOTH_SWIMBACKWARD), // # Swim backward loop
    //# #sep ENUM2STRING(BOTH_ LYING
    enum2string(c"BOTH_SLEEP1", BOTH_SLEEP1), // # laying on back-rknee up-rhand on torso
    enum2string(c"BOTH_SLEEP6START", BOTH_SLEEP6START), // # Kyle leaning back to sleep (cin 20)
    enum2string(c"BOTH_SLEEP6STOP", BOTH_SLEEP6STOP), // # Kyle waking up and shaking his head (cin 21)
    enum2string(c"BOTH_SLEEP1GETUP", BOTH_SLEEP1GETUP), // # alarmed and getting up out of sleep1 pose to stand
    enum2string(c"BOTH_SLEEP1GETUP2", BOTH_SLEEP1GETUP2), // #
    enum2string(c"BOTH_CHOKE1START", BOTH_CHOKE1START), // # tavion in force grip choke
    enum2string(c"BOTH_CHOKE1STARTHOLD", BOTH_CHOKE1STARTHOLD), // # loop of tavion in force grip choke
    enum2string(c"BOTH_CHOKE1", BOTH_CHOKE1),                   // # tavion in force grip choke
    enum2string(c"BOTH_CHOKE2", BOTH_CHOKE2), // # tavion recovering from force grip choke
    enum2string(c"BOTH_CHOKE3", BOTH_CHOKE3), // # left-handed choke (for people still holding a weapon)
    //# #sep ENUM2STRING(BOTH_ HUNTER-SEEKER BOT-SPECIFIC
    enum2string(c"BOTH_POWERUP1", BOTH_POWERUP1), // # Wakes up
    enum2string(c"BOTH_TURNON", BOTH_TURNON),     // # Protocol Droid wakes up
    enum2string(c"BOTH_TURNOFF", BOTH_TURNOFF),   // # Protocol Droid shuts off
    enum2string(c"BOTH_BUTTON1", BOTH_BUTTON1),   // # Single button push with right hand
    enum2string(c"BOTH_BUTTON2", BOTH_BUTTON2),   // # Single button push with left finger
    enum2string(c"BOTH_BUTTON_HOLD", BOTH_BUTTON_HOLD), // # Single button hold with left hand
    enum2string(c"BOTH_BUTTON_RELEASE", BOTH_BUTTON_RELEASE), // # Single button release with left hand
    //# JEDI-SPECIFIC
    //# #sep BOTH_ FORCE ANIMS
    enum2string(c"BOTH_RESISTPUSH", BOTH_RESISTPUSH), // # plant yourself to resist force push/pulls.
    enum2string(c"BOTH_FORCEPUSH", BOTH_FORCEPUSH),   // # Use off-hand to do force power.
    enum2string(c"BOTH_FORCEPULL", BOTH_FORCEPULL),   // # Use off-hand to do force power.
    enum2string(c"BOTH_MINDTRICK1", BOTH_MINDTRICK1), // # Use off-hand to do mind trick
    enum2string(c"BOTH_MINDTRICK2", BOTH_MINDTRICK2), // # Use off-hand to do distraction
    enum2string(c"BOTH_FORCELIGHTNING", BOTH_FORCELIGHTNING), // # Use off-hand to do lightning
    enum2string(c"BOTH_FORCELIGHTNING_START", BOTH_FORCELIGHTNING_START), // # Use off-hand to do lightning - start
    enum2string(c"BOTH_FORCELIGHTNING_HOLD", BOTH_FORCELIGHTNING_HOLD), // # Use off-hand to do lightning - hold
    enum2string(c"BOTH_FORCELIGHTNING_RELEASE", BOTH_FORCELIGHTNING_RELEASE), // # Use off-hand to do lightning - release
    enum2string(c"BOTH_FORCEHEAL_START", BOTH_FORCEHEAL_START), // # Healing meditation pose start
    enum2string(c"BOTH_FORCEHEAL_STOP", BOTH_FORCEHEAL_STOP),   // # Healing meditation pose end
    enum2string(c"BOTH_FORCEHEAL_QUICK", BOTH_FORCEHEAL_QUICK), // # Healing meditation gesture
    enum2string(c"BOTH_SABERPULL", BOTH_SABERPULL),             // # Use off-hand to do force power.
    enum2string(c"BOTH_FORCEGRIP1", BOTH_FORCEGRIP1),           // # force-gripping (no anim?)
    enum2string(c"BOTH_FORCEGRIP3", BOTH_FORCEGRIP3),           // # force-gripping (right-hand)
    enum2string(c"BOTH_FORCEGRIP3THROW", BOTH_FORCEGRIP3THROW), // # throwing while force-gripping (right hand)
    enum2string(c"BOTH_FORCEGRIP_HOLD", BOTH_FORCEGRIP_HOLD),   // # Use off-hand to do grip - hold
    enum2string(c"BOTH_FORCEGRIP_RELEASE", BOTH_FORCEGRIP_RELEASE), // # Use off-hand to do grip - release
    enum2string(c"BOTH_TOSS1", BOTH_TOSS1), // # throwing to left after force gripping
    enum2string(c"BOTH_TOSS2", BOTH_TOSS2), // # throwing to right after force gripping
    //NEW force anims for JKA:
    enum2string(c"BOTH_FORCE_RAGE", BOTH_FORCE_RAGE),
    enum2string(c"BOTH_FORCE_2HANDEDLIGHTNING", BOTH_FORCE_2HANDEDLIGHTNING),
    enum2string(
        c"BOTH_FORCE_2HANDEDLIGHTNING_START",
        BOTH_FORCE_2HANDEDLIGHTNING_START,
    ),
    enum2string(
        c"BOTH_FORCE_2HANDEDLIGHTNING_HOLD",
        BOTH_FORCE_2HANDEDLIGHTNING_HOLD,
    ),
    enum2string(
        c"BOTH_FORCE_2HANDEDLIGHTNING_RELEASE",
        BOTH_FORCE_2HANDEDLIGHTNING_RELEASE,
    ),
    enum2string(c"BOTH_FORCE_DRAIN", BOTH_FORCE_DRAIN),
    enum2string(c"BOTH_FORCE_DRAIN_START", BOTH_FORCE_DRAIN_START),
    enum2string(c"BOTH_FORCE_DRAIN_HOLD", BOTH_FORCE_DRAIN_HOLD),
    enum2string(c"BOTH_FORCE_DRAIN_RELEASE", BOTH_FORCE_DRAIN_RELEASE),
    enum2string(c"BOTH_FORCE_DRAIN_GRAB_START", BOTH_FORCE_DRAIN_GRAB_START),
    enum2string(c"BOTH_FORCE_DRAIN_GRAB_HOLD", BOTH_FORCE_DRAIN_GRAB_HOLD),
    enum2string(c"BOTH_FORCE_DRAIN_GRAB_END", BOTH_FORCE_DRAIN_GRAB_END),
    enum2string(c"BOTH_FORCE_DRAIN_GRABBED", BOTH_FORCE_DRAIN_GRABBED),
    enum2string(c"BOTH_FORCE_ABSORB", BOTH_FORCE_ABSORB),
    enum2string(c"BOTH_FORCE_ABSORB_START", BOTH_FORCE_ABSORB_START),
    enum2string(c"BOTH_FORCE_ABSORB_END", BOTH_FORCE_ABSORB_END),
    enum2string(c"BOTH_FORCE_PROTECT", BOTH_FORCE_PROTECT),
    enum2string(c"BOTH_FORCE_PROTECT_FAST", BOTH_FORCE_PROTECT_FAST),
    enum2string(c"BOTH_WIND", BOTH_WIND),
    enum2string(c"BOTH_STAND_TO_KNEEL", BOTH_STAND_TO_KNEEL),
    enum2string(c"BOTH_KNEEL_TO_STAND", BOTH_KNEEL_TO_STAND),
    enum2string(c"BOTH_TUSKENATTACK1", BOTH_TUSKENATTACK1),
    enum2string(c"BOTH_TUSKENATTACK2", BOTH_TUSKENATTACK2),
    enum2string(c"BOTH_TUSKENATTACK3", BOTH_TUSKENATTACK3),
    enum2string(c"BOTH_TUSKENLUNGE1", BOTH_TUSKENLUNGE1),
    enum2string(c"BOTH_TUSKENTAUNT1", BOTH_TUSKENTAUNT1),
    enum2string(c"BOTH_COWER1_START", BOTH_COWER1_START), // # cower start
    enum2string(c"BOTH_COWER1", BOTH_COWER1),             // # cower loop
    enum2string(c"BOTH_COWER1_STOP", BOTH_COWER1_STOP),   // # cower stop
    enum2string(c"BOTH_SONICPAIN_START", BOTH_SONICPAIN_START),
    enum2string(c"BOTH_SONICPAIN_HOLD", BOTH_SONICPAIN_HOLD),
    enum2string(c"BOTH_SONICPAIN_END", BOTH_SONICPAIN_END),
    //new anim slots per Jarrod's request
    enum2string(c"BOTH_STAND10", BOTH_STAND10),
    enum2string(c"BOTH_STAND10_TALK1", BOTH_STAND10_TALK1),
    enum2string(c"BOTH_STAND10_TALK2", BOTH_STAND10_TALK2),
    enum2string(c"BOTH_STAND10TOSTAND1", BOTH_STAND10TOSTAND1),
    enum2string(c"BOTH_STAND1_TALK1", BOTH_STAND1_TALK1),
    enum2string(c"BOTH_STAND1_TALK2", BOTH_STAND1_TALK2),
    enum2string(c"BOTH_STAND1_TALK3", BOTH_STAND1_TALK3),
    enum2string(c"BOTH_SIT4", BOTH_SIT4),
    enum2string(c"BOTH_SIT5", BOTH_SIT5),
    enum2string(c"BOTH_SIT5_TALK1", BOTH_SIT5_TALK1),
    enum2string(c"BOTH_SIT5_TALK2", BOTH_SIT5_TALK2),
    enum2string(c"BOTH_SIT5_TALK3", BOTH_SIT5_TALK3),
    enum2string(c"BOTH_SIT6", BOTH_SIT6),
    enum2string(c"BOTH_SIT7", BOTH_SIT7),
    //=================================================
    //ANIMS IN WHICH ONLY THE UPPER OBJECTS ARE IN MD3
    //=================================================
    //# #sep ENUM2STRING(TORSO_ WEAPON-RELATED
    enum2string(c"TORSO_DROPWEAP1", TORSO_DROPWEAP1), // # Put weapon away
    enum2string(c"TORSO_DROPWEAP4", TORSO_DROPWEAP4), // # Put weapon away
    enum2string(c"TORSO_RAISEWEAP1", TORSO_RAISEWEAP1), // # Draw Weapon
    enum2string(c"TORSO_RAISEWEAP4", TORSO_RAISEWEAP4), // # Draw Weapon
    enum2string(c"TORSO_WEAPONREADY1", TORSO_WEAPONREADY1), // # Ready to fire stun baton
    enum2string(c"TORSO_WEAPONREADY2", TORSO_WEAPONREADY2), // # Ready to fire one-handed blaster pistol
    enum2string(c"TORSO_WEAPONREADY3", TORSO_WEAPONREADY3), // # Ready to fire blaster rifle
    enum2string(c"TORSO_WEAPONREADY4", TORSO_WEAPONREADY4), // # Ready to fire sniper rifle
    enum2string(c"TORSO_WEAPONREADY10", TORSO_WEAPONREADY10), // # Ready to fire thermal det
    enum2string(c"TORSO_WEAPONIDLE2", TORSO_WEAPONIDLE2),   // # Holding one-handed blaster
    enum2string(c"TORSO_WEAPONIDLE3", TORSO_WEAPONIDLE3),   // # Holding blaster rifle
    enum2string(c"TORSO_WEAPONIDLE4", TORSO_WEAPONIDLE4),   // # Holding sniper rifle
    enum2string(c"TORSO_WEAPONIDLE10", TORSO_WEAPONIDLE10), // # Holding thermal det
    //# #sep ENUM2STRING(TORSO_ USING NON-WEAPON OBJECTS

    //# #sep ENUM2STRING(TORSO_ MISC
    enum2string(c"TORSO_SURRENDER_START", TORSO_SURRENDER_START), // # arms up
    enum2string(c"TORSO_SURRENDER_STOP", TORSO_SURRENDER_STOP),   // # arms back down
    enum2string(c"TORSO_CHOKING1", TORSO_CHOKING1),               // # TEMP
    enum2string(c"TORSO_HANDSIGNAL1", TORSO_HANDSIGNAL1),
    enum2string(c"TORSO_HANDSIGNAL2", TORSO_HANDSIGNAL2),
    enum2string(c"TORSO_HANDSIGNAL3", TORSO_HANDSIGNAL3),
    enum2string(c"TORSO_HANDSIGNAL4", TORSO_HANDSIGNAL4),
    enum2string(c"TORSO_HANDSIGNAL5", TORSO_HANDSIGNAL5),
    //=================================================
    //ANIMS IN WHICH ONLY THE LOWER OBJECTS ARE IN MD3
    //=================================================
    //# #sep Legs-only anims
    enum2string(c"LEGS_TURN1", LEGS_TURN1), // # What legs do when you turn your lower body to match your upper body facing
    enum2string(c"LEGS_TURN2", LEGS_TURN2), // # Leg turning from stand2
    enum2string(c"LEGS_LEAN_LEFT1", LEGS_LEAN_LEFT1), // # Lean left
    enum2string(c"LEGS_LEAN_RIGHT1", LEGS_LEAN_RIGHT1), // # Lean Right
    enum2string(c"LEGS_CHOKING1", LEGS_CHOKING1), // # TEMP
    enum2string(c"LEGS_LEFTUP1", LEGS_LEFTUP1), // # On a slope with left foot 4 higher than right
    enum2string(c"LEGS_LEFTUP2", LEGS_LEFTUP2), // # On a slope with left foot 8 higher than right
    enum2string(c"LEGS_LEFTUP3", LEGS_LEFTUP3), // # On a slope with left foot 12 higher than right
    enum2string(c"LEGS_LEFTUP4", LEGS_LEFTUP4), // # On a slope with left foot 16 higher than right
    enum2string(c"LEGS_LEFTUP5", LEGS_LEFTUP5), // # On a slope with left foot 20 higher than right
    enum2string(c"LEGS_RIGHTUP1", LEGS_RIGHTUP1), // # On a slope with RIGHT foot 4 higher than left
    enum2string(c"LEGS_RIGHTUP2", LEGS_RIGHTUP2), // # On a slope with RIGHT foot 8 higher than left
    enum2string(c"LEGS_RIGHTUP3", LEGS_RIGHTUP3), // # On a slope with RIGHT foot 12 higher than left
    enum2string(c"LEGS_RIGHTUP4", LEGS_RIGHTUP4), // # On a slope with RIGHT foot 16 higher than left
    enum2string(c"LEGS_RIGHTUP5", LEGS_RIGHTUP5), // # On a slope with RIGHT foot 20 higher than left
    enum2string(c"LEGS_S1_LUP1", LEGS_S1_LUP1),
    enum2string(c"LEGS_S1_LUP2", LEGS_S1_LUP2),
    enum2string(c"LEGS_S1_LUP3", LEGS_S1_LUP3),
    enum2string(c"LEGS_S1_LUP4", LEGS_S1_LUP4),
    enum2string(c"LEGS_S1_LUP5", LEGS_S1_LUP5),
    enum2string(c"LEGS_S1_RUP1", LEGS_S1_RUP1),
    enum2string(c"LEGS_S1_RUP2", LEGS_S1_RUP2),
    enum2string(c"LEGS_S1_RUP3", LEGS_S1_RUP3),
    enum2string(c"LEGS_S1_RUP4", LEGS_S1_RUP4),
    enum2string(c"LEGS_S1_RUP5", LEGS_S1_RUP5),
    enum2string(c"LEGS_S3_LUP1", LEGS_S3_LUP1),
    enum2string(c"LEGS_S3_LUP2", LEGS_S3_LUP2),
    enum2string(c"LEGS_S3_LUP3", LEGS_S3_LUP3),
    enum2string(c"LEGS_S3_LUP4", LEGS_S3_LUP4),
    enum2string(c"LEGS_S3_LUP5", LEGS_S3_LUP5),
    enum2string(c"LEGS_S3_RUP1", LEGS_S3_RUP1),
    enum2string(c"LEGS_S3_RUP2", LEGS_S3_RUP2),
    enum2string(c"LEGS_S3_RUP3", LEGS_S3_RUP3),
    enum2string(c"LEGS_S3_RUP4", LEGS_S3_RUP4),
    enum2string(c"LEGS_S3_RUP5", LEGS_S3_RUP5),
    enum2string(c"LEGS_S4_LUP1", LEGS_S4_LUP1),
    enum2string(c"LEGS_S4_LUP2", LEGS_S4_LUP2),
    enum2string(c"LEGS_S4_LUP3", LEGS_S4_LUP3),
    enum2string(c"LEGS_S4_LUP4", LEGS_S4_LUP4),
    enum2string(c"LEGS_S4_LUP5", LEGS_S4_LUP5),
    enum2string(c"LEGS_S4_RUP1", LEGS_S4_RUP1),
    enum2string(c"LEGS_S4_RUP2", LEGS_S4_RUP2),
    enum2string(c"LEGS_S4_RUP3", LEGS_S4_RUP3),
    enum2string(c"LEGS_S4_RUP4", LEGS_S4_RUP4),
    enum2string(c"LEGS_S4_RUP5", LEGS_S4_RUP5),
    enum2string(c"LEGS_S5_LUP1", LEGS_S5_LUP1),
    enum2string(c"LEGS_S5_LUP2", LEGS_S5_LUP2),
    enum2string(c"LEGS_S5_LUP3", LEGS_S5_LUP3),
    enum2string(c"LEGS_S5_LUP4", LEGS_S5_LUP4),
    enum2string(c"LEGS_S5_LUP5", LEGS_S5_LUP5),
    enum2string(c"LEGS_S5_RUP1", LEGS_S5_RUP1),
    enum2string(c"LEGS_S5_RUP2", LEGS_S5_RUP2),
    enum2string(c"LEGS_S5_RUP3", LEGS_S5_RUP3),
    enum2string(c"LEGS_S5_RUP4", LEGS_S5_RUP4),
    enum2string(c"LEGS_S5_RUP5", LEGS_S5_RUP5),
    enum2string(c"LEGS_S6_LUP1", LEGS_S6_LUP1),
    enum2string(c"LEGS_S6_LUP2", LEGS_S6_LUP2),
    enum2string(c"LEGS_S6_LUP3", LEGS_S6_LUP3),
    enum2string(c"LEGS_S6_LUP4", LEGS_S6_LUP4),
    enum2string(c"LEGS_S6_LUP5", LEGS_S6_LUP5),
    enum2string(c"LEGS_S6_RUP1", LEGS_S6_RUP1),
    enum2string(c"LEGS_S6_RUP2", LEGS_S6_RUP2),
    enum2string(c"LEGS_S6_RUP3", LEGS_S6_RUP3),
    enum2string(c"LEGS_S6_RUP4", LEGS_S6_RUP4),
    enum2string(c"LEGS_S6_RUP5", LEGS_S6_RUP5),
    enum2string(c"LEGS_S7_LUP1", LEGS_S7_LUP1),
    enum2string(c"LEGS_S7_LUP2", LEGS_S7_LUP2),
    enum2string(c"LEGS_S7_LUP3", LEGS_S7_LUP3),
    enum2string(c"LEGS_S7_LUP4", LEGS_S7_LUP4),
    enum2string(c"LEGS_S7_LUP5", LEGS_S7_LUP5),
    enum2string(c"LEGS_S7_RUP1", LEGS_S7_RUP1),
    enum2string(c"LEGS_S7_RUP2", LEGS_S7_RUP2),
    enum2string(c"LEGS_S7_RUP3", LEGS_S7_RUP3),
    enum2string(c"LEGS_S7_RUP4", LEGS_S7_RUP4),
    enum2string(c"LEGS_S7_RUP5", LEGS_S7_RUP5),
    //New anim as per Jarrod's request
    enum2string(c"LEGS_TURN180", LEGS_TURN180),
    //======================================================
    //cinematic anims
    //======================================================
    //# #sep BOTH_ CINEMATIC-ONLY
    enum2string(c"BOTH_CIN_1", BOTH_CIN_1), // # Level specific cinematic 1
    enum2string(c"BOTH_CIN_2", BOTH_CIN_2), // # Level specific cinematic 2
    enum2string(c"BOTH_CIN_3", BOTH_CIN_3), // # Level specific cinematic 3
    enum2string(c"BOTH_CIN_4", BOTH_CIN_4), // # Level specific cinematic 4
    enum2string(c"BOTH_CIN_5", BOTH_CIN_5), // # Level specific cinematic 5
    enum2string(c"BOTH_CIN_6", BOTH_CIN_6), // # Level specific cinematic 6
    enum2string(c"BOTH_CIN_7", BOTH_CIN_7), // # Level specific cinematic 7
    enum2string(c"BOTH_CIN_8", BOTH_CIN_8), // # Level specific cinematic 8
    enum2string(c"BOTH_CIN_9", BOTH_CIN_9), // # Level specific cinematic 9
    enum2string(c"BOTH_CIN_10", BOTH_CIN_10), // # Level specific cinematic 10
    enum2string(c"BOTH_CIN_11", BOTH_CIN_11), // # Level specific cinematic 11
    enum2string(c"BOTH_CIN_12", BOTH_CIN_12), // # Level specific cinematic 12
    enum2string(c"BOTH_CIN_13", BOTH_CIN_13), // # Level specific cinematic 13
    enum2string(c"BOTH_CIN_14", BOTH_CIN_14), // # Level specific cinematic 14
    enum2string(c"BOTH_CIN_15", BOTH_CIN_15), // # Level specific cinematic 15
    enum2string(c"BOTH_CIN_16", BOTH_CIN_16), // # Level specific cinematic 16
    enum2string(c"BOTH_CIN_17", BOTH_CIN_17), // # Level specific cinematic 17
    enum2string(c"BOTH_CIN_18", BOTH_CIN_18), // # Level specific cinematic 18
    enum2string(c"BOTH_CIN_19", BOTH_CIN_19), // # Level specific cinematic 19
    enum2string(c"BOTH_CIN_20", BOTH_CIN_20), // # Level specific cinematic 20
    enum2string(c"BOTH_CIN_21", BOTH_CIN_21), // # Level specific cinematic 21
    enum2string(c"BOTH_CIN_22", BOTH_CIN_22), // # Level specific cinematic 22
    enum2string(c"BOTH_CIN_23", BOTH_CIN_23), // # Level specific cinematic 23
    enum2string(c"BOTH_CIN_24", BOTH_CIN_24), // # Level specific cinematic 24
    enum2string(c"BOTH_CIN_25", BOTH_CIN_25), // # Level specific cinematic 25
    enum2string(c"BOTH_CIN_26", BOTH_CIN_26), // # Level specific cinematic
    enum2string(c"BOTH_CIN_27", BOTH_CIN_27), // # Level specific cinematic
    enum2string(c"BOTH_CIN_28", BOTH_CIN_28), // # Level specific cinematic
    enum2string(c"BOTH_CIN_29", BOTH_CIN_29), // # Level specific cinematic
    enum2string(c"BOTH_CIN_30", BOTH_CIN_30), // # Level specific cinematic
    enum2string(c"BOTH_CIN_31", BOTH_CIN_31), // # Level specific cinematic
    enum2string(c"BOTH_CIN_32", BOTH_CIN_32), // # Level specific cinematic
    enum2string(c"BOTH_CIN_33", BOTH_CIN_33), // # Level specific cinematic
    enum2string(c"BOTH_CIN_34", BOTH_CIN_34), // # Level specific cinematic
    enum2string(c"BOTH_CIN_35", BOTH_CIN_35), // # Level specific cinematic
    enum2string(c"BOTH_CIN_36", BOTH_CIN_36), // # Level specific cinematic
    enum2string(c"BOTH_CIN_37", BOTH_CIN_37), // # Level specific cinematic
    enum2string(c"BOTH_CIN_38", BOTH_CIN_38), // # Level specific cinematic
    enum2string(c"BOTH_CIN_39", BOTH_CIN_39), // # Level specific cinematic
    enum2string(c"BOTH_CIN_40", BOTH_CIN_40), // # Level specific cinematic
    enum2string(c"BOTH_CIN_41", BOTH_CIN_41), // # Level specific cinematic
    enum2string(c"BOTH_CIN_42", BOTH_CIN_42), // # Level specific cinematic
    enum2string(c"BOTH_CIN_43", BOTH_CIN_43), // # Level specific cinematic
    enum2string(c"BOTH_CIN_44", BOTH_CIN_44), // # Level specific cinematic
    enum2string(c"BOTH_CIN_45", BOTH_CIN_45), // # Level specific cinematic
    enum2string(c"BOTH_CIN_46", BOTH_CIN_46), // # Level specific cinematic
    enum2string(c"BOTH_CIN_47", BOTH_CIN_47), // # Level specific cinematic
    enum2string(c"BOTH_CIN_48", BOTH_CIN_48), // # Level specific cinematic
    enum2string(c"BOTH_CIN_49", BOTH_CIN_49), // # Level specific cinematic
    enum2string(c"BOTH_CIN_50", BOTH_CIN_50), // # Level specific cinematic
    //must be terminated
    stringID_table_t {
        name: core::ptr::null(),
        id: -1,
    }, // must be terminated
];

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::{GetIDForString, GetStringForID};
    use core::ptr::addr_of;

    extern "C" {
        fn jka_animTable_count() -> core::ffi::c_int;
        fn jka_animTable_name(i: core::ffi::c_int) -> *const core::ffi::c_char;
        fn jka_animTable_id(i: core::ffi::c_int) -> core::ffi::c_int;
    }

    /// Every row of the Rust `animTable` matches the authentic C `animTable[]`
    /// element-wise (name bytes + id), including the count and the `{ NULL, -1 }`
    /// terminator -- the C side comes from `#include`ing the real header, so a
    /// transcription slip (wrong name, wrong id, dropped/added row, mis-order)
    /// fails here.
    #[test]
    fn anim_table_matches_c() {
        let rust = addr_of!(animTable) as *const stringID_table_t;
        unsafe {
            let n = jka_animTable_count();
            assert_eq!(n as usize, (MAX_ANIMATIONS + 1) as usize, "animTable len");
            for i in 0..n {
                let r = &*rust.add(i as usize);
                let cname = jka_animTable_name(i);
                let cid = jka_animTable_id(i);
                assert_eq!(r.id, cid, "animTable[{i}].id");
                if cname.is_null() {
                    assert!(r.name.is_null(), "animTable[{i}].name should be NULL");
                } else {
                    let rn = CStr::from_ptr(r.name);
                    let cn = CStr::from_ptr(cname);
                    assert_eq!(rn, cn, "animTable[{i}].name");
                }
            }
        }
    }

    /// Drives the Rust `animTable` through the authentic C `GetIDForString` /
    /// `GetStringForID` (real q_shared.c) on a spread of checkpoints -- confirms the
    /// table is usable by the exact lookups the parsers call.
    #[test]
    fn anim_table_lookups_match_c() {
        let tbl = addr_of!(animTable) as *const stringID_table_t;
        let cases: &[(&CStr, animNumber_t)] = &[
            (c"FACE_TALK0", FACE_TALK0),
            (c"BOTH_DEATH1", BOTH_DEATH1),
            (c"BOTH_ATTACK1", BOTH_ATTACK1),
            (c"BOTH_CIN_50", BOTH_CIN_50),
            (c"both_death1", BOTH_DEATH1), // GetIDForString uses Q_stricmp (case-fold)
            (c"BOTH_NONEXISTENT_ANIM", -1),
            (c"", -1),
        ];
        for (name, want) in cases {
            let got = unsafe { GetIDForString(tbl, name.as_ptr()) };
            assert_eq!(got, *want, "GetIDForString({name:?})");
        }
        // round-trip a couple ids back to names via GetStringForID
        unsafe {
            let s = GetStringForID(tbl, FACE_TALK0);
            assert_eq!(CStr::from_ptr(s), c"FACE_TALK0");
        }
    }
}
