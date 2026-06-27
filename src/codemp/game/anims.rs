//! `animNumber_t` (`anims.h`) — the master player-animation enumeration.
//!
//! One `pub const` per C enumerator. The C `enum` has no explicit `= value`
//! assignments, so the numbering is implicit 0-based sequential; we carry that
//! verbatim (each value is its position). `animNumber_t` stays a plain `c_int`
//! alias because the game treats these as ints everywhere — the `legsAnim` /
//! `torsoAnim` fields in `entityState_t` / `playerState_t` are `int`, not this
//! enum type, and the values are compared and arithmetic'd (see
//! `SABER_ANIM_GROUP_SIZE`) and used as animation-table indices.
//!
//! The numbering is ABI/asset-load-bearing, so every checkpoint value (plus the
//! two terminal counts) is asserted against the authentic C enum via the oracle
//! TU `oracle/anims_oracle.c`, which `#include`s the real Raven `anims.h`
//! unmodified — making the Rust list (generated from the header) and the oracle
//! (the C compiler reading the header) independently derived.
//!
//! Generated faithfully from `refs/raven-jediacademy/codemp/game/anims.h`; all original
//! section banners and per-anim comments are carried.

// Many saber-anim enumerators carry double / trailing underscores verbatim from the
// C header (e.g. `BOTH_A1_T__B_`, `BOTH_T1_T___R`), which trips the upper-case-globals
// style lint. rustc stays quiet, but rust-analyzer surfaces it in the editor; the names
// are ABI/asset load-bearing and cannot change, so allow it module-wide (per the
// project convention of keeping C names verbatim for diffability).
#![allow(non_camel_case_types, non_upper_case_globals)]

use core::ffi::c_int;

/// `animNumber_t` (`anims.h`) — a C `enum`, transmitted/stored as a plain `int`.
pub type animNumber_t = c_int;

//=================================================
//HEAD ANIMS
//=================================================
//# #sep Head-only anims
pub const FACE_TALK0: animNumber_t = 0; //# silent
pub const FACE_TALK1: animNumber_t = 1; //# quiet
pub const FACE_TALK2: animNumber_t = 2; //# semi-quiet
pub const FACE_TALK3: animNumber_t = 3; //# semi-loud
pub const FACE_TALK4: animNumber_t = 4; //# loud
pub const FACE_ALERT: animNumber_t = 5; //#
pub const FACE_SMILE: animNumber_t = 6; //#
pub const FACE_FROWN: animNumber_t = 7; //#
pub const FACE_DEAD: animNumber_t = 8; //#

//=================================================
//ANIMS IN WHICH UPPER AND LOWER OBJECTS ARE IN MD3
//=================================================
//# #sep BOTH_ DEATHS
pub const BOTH_DEATH1: animNumber_t = 9; //# First Death anim
pub const BOTH_DEATH2: animNumber_t = 10; //# Second Death anim
pub const BOTH_DEATH3: animNumber_t = 11; //# Third Death anim
pub const BOTH_DEATH4: animNumber_t = 12; //# Fourth Death anim
pub const BOTH_DEATH5: animNumber_t = 13; //# Fifth Death anim
pub const BOTH_DEATH6: animNumber_t = 14; //# Sixth Death anim
pub const BOTH_DEATH7: animNumber_t = 15; //# Seventh Death anim
pub const BOTH_DEATH8: animNumber_t = 16; //#
pub const BOTH_DEATH9: animNumber_t = 17; //#
pub const BOTH_DEATH10: animNumber_t = 18; //#
pub const BOTH_DEATH11: animNumber_t = 19; //#
pub const BOTH_DEATH12: animNumber_t = 20; //#
pub const BOTH_DEATH13: animNumber_t = 21; //#
pub const BOTH_DEATH14: animNumber_t = 22; //#
pub const BOTH_DEATH15: animNumber_t = 23; //#
pub const BOTH_DEATH16: animNumber_t = 24; //#
pub const BOTH_DEATH17: animNumber_t = 25; //#
pub const BOTH_DEATH18: animNumber_t = 26; //#
pub const BOTH_DEATH19: animNumber_t = 27; //#
pub const BOTH_DEATH20: animNumber_t = 28; //#
pub const BOTH_DEATH21: animNumber_t = 29; //#
pub const BOTH_DEATH22: animNumber_t = 30; //#
pub const BOTH_DEATH23: animNumber_t = 31; //#
pub const BOTH_DEATH24: animNumber_t = 32; //#
pub const BOTH_DEATH25: animNumber_t = 33; //#

pub const BOTH_DEATHFORWARD1: animNumber_t = 34; //# First Death in which they get thrown forward
pub const BOTH_DEATHFORWARD2: animNumber_t = 35; //# Second Death in which they get thrown forward
pub const BOTH_DEATHFORWARD3: animNumber_t = 36; //# Tavion's falling in cin# 23
pub const BOTH_DEATHBACKWARD1: animNumber_t = 37; //# First Death in which they get thrown backward
pub const BOTH_DEATHBACKWARD2: animNumber_t = 38; //# Second Death in which they get thrown backward

pub const BOTH_DEATH1IDLE: animNumber_t = 39; //# Idle while close to death
pub const BOTH_LYINGDEATH1: animNumber_t = 40; //# Death to play when killed lying down
pub const BOTH_STUMBLEDEATH1: animNumber_t = 41; //# Stumble forward and fall face first death
pub const BOTH_FALLDEATH1: animNumber_t = 42; //# Fall forward off a high cliff and splat death - start
pub const BOTH_FALLDEATH1INAIR: animNumber_t = 43; //# Fall forward off a high cliff and splat death - loop
pub const BOTH_FALLDEATH1LAND: animNumber_t = 44; //# Fall forward off a high cliff and splat death - hit bottom
pub const BOTH_DEATH_ROLL: animNumber_t = 45; //# Death anim from a roll
pub const BOTH_DEATH_FLIP: animNumber_t = 46; //# Death anim from a flip
pub const BOTH_DEATH_SPIN_90_R: animNumber_t = 47; //# Death anim when facing 90 degrees right
pub const BOTH_DEATH_SPIN_90_L: animNumber_t = 48; //# Death anim when facing 90 degrees left
pub const BOTH_DEATH_SPIN_180: animNumber_t = 49; //# Death anim when facing backwards
pub const BOTH_DEATH_LYING_UP: animNumber_t = 50; //# Death anim when lying on back
pub const BOTH_DEATH_LYING_DN: animNumber_t = 51; //# Death anim when lying on front
pub const BOTH_DEATH_FALLING_DN: animNumber_t = 52; //# Death anim when falling on face
pub const BOTH_DEATH_FALLING_UP: animNumber_t = 53; //# Death anim when falling on back
pub const BOTH_DEATH_CROUCHED: animNumber_t = 54; //# Death anim when crouched
//# #sep BOTH_ DEAD POSES # Should be last frame of corresponding previous anims
pub const BOTH_DEAD1: animNumber_t = 55; //# First Death finished pose
pub const BOTH_DEAD2: animNumber_t = 56; //# Second Death finished pose
pub const BOTH_DEAD3: animNumber_t = 57; //# Third Death finished pose
pub const BOTH_DEAD4: animNumber_t = 58; //# Fourth Death finished pose
pub const BOTH_DEAD5: animNumber_t = 59; //# Fifth Death finished pose
pub const BOTH_DEAD6: animNumber_t = 60; //# Sixth Death finished pose
pub const BOTH_DEAD7: animNumber_t = 61; //# Seventh Death finished pose
pub const BOTH_DEAD8: animNumber_t = 62; //#
pub const BOTH_DEAD9: animNumber_t = 63; //#
pub const BOTH_DEAD10: animNumber_t = 64; //#
pub const BOTH_DEAD11: animNumber_t = 65; //#
pub const BOTH_DEAD12: animNumber_t = 66; //#
pub const BOTH_DEAD13: animNumber_t = 67; //#
pub const BOTH_DEAD14: animNumber_t = 68; //#
pub const BOTH_DEAD15: animNumber_t = 69; //#
pub const BOTH_DEAD16: animNumber_t = 70; //#
pub const BOTH_DEAD17: animNumber_t = 71; //#
pub const BOTH_DEAD18: animNumber_t = 72; //#
pub const BOTH_DEAD19: animNumber_t = 73; //#
pub const BOTH_DEAD20: animNumber_t = 74; //#
pub const BOTH_DEAD21: animNumber_t = 75; //#
pub const BOTH_DEAD22: animNumber_t = 76; //#
pub const BOTH_DEAD23: animNumber_t = 77; //#
pub const BOTH_DEAD24: animNumber_t = 78; //#
pub const BOTH_DEAD25: animNumber_t = 79; //#
pub const BOTH_DEADFORWARD1: animNumber_t = 80; //# First thrown forward death finished pose
pub const BOTH_DEADFORWARD2: animNumber_t = 81; //# Second thrown forward death finished pose
pub const BOTH_DEADBACKWARD1: animNumber_t = 82; //# First thrown backward death finished pose
pub const BOTH_DEADBACKWARD2: animNumber_t = 83; //# Second thrown backward death finished pose
pub const BOTH_LYINGDEAD1: animNumber_t = 84; //# Killed lying down death finished pose
pub const BOTH_STUMBLEDEAD1: animNumber_t = 85; //# Stumble forward death finished pose
pub const BOTH_FALLDEAD1LAND: animNumber_t = 86; //# Fall forward and splat death finished pose
//# #sep BOTH_ DEAD TWITCH/FLOP # React to being shot from death poses
pub const BOTH_DEADFLOP1: animNumber_t = 87; //# React to being shot from First Death finished pose
pub const BOTH_DEADFLOP2: animNumber_t = 88; //# React to being shot from Second Death finished pose
pub const BOTH_DISMEMBER_HEAD1: animNumber_t = 89; //#
pub const BOTH_DISMEMBER_TORSO1: animNumber_t = 90; //#
pub const BOTH_DISMEMBER_LLEG: animNumber_t = 91; //#
pub const BOTH_DISMEMBER_RLEG: animNumber_t = 92; //#
pub const BOTH_DISMEMBER_RARM: animNumber_t = 93; //#
pub const BOTH_DISMEMBER_LARM: animNumber_t = 94; //#
//# #sep BOTH_ PAINS
pub const BOTH_PAIN1: animNumber_t = 95; //# First take pain anim
pub const BOTH_PAIN2: animNumber_t = 96; //# Second take pain anim
pub const BOTH_PAIN3: animNumber_t = 97; //# Third take pain anim
pub const BOTH_PAIN4: animNumber_t = 98; //# Fourth take pain anim
pub const BOTH_PAIN5: animNumber_t = 99; //# Fifth take pain anim - from behind
pub const BOTH_PAIN6: animNumber_t = 100; //# Sixth take pain anim - from behind
pub const BOTH_PAIN7: animNumber_t = 101; //# Seventh take pain anim - from behind
pub const BOTH_PAIN8: animNumber_t = 102; //# Eigth take pain anim - from behind
pub const BOTH_PAIN9: animNumber_t = 103; //#
pub const BOTH_PAIN10: animNumber_t = 104; //#
pub const BOTH_PAIN11: animNumber_t = 105; //#
pub const BOTH_PAIN12: animNumber_t = 106; //#
pub const BOTH_PAIN13: animNumber_t = 107; //#
pub const BOTH_PAIN14: animNumber_t = 108; //#
pub const BOTH_PAIN15: animNumber_t = 109; //#
pub const BOTH_PAIN16: animNumber_t = 110; //#
pub const BOTH_PAIN17: animNumber_t = 111; //#
pub const BOTH_PAIN18: animNumber_t = 112; //#

//# #sep BOTH_ ATTACKS
pub const BOTH_ATTACK1: animNumber_t = 113; //# Attack with stun baton
pub const BOTH_ATTACK2: animNumber_t = 114; //# Attack with one-handed pistol
pub const BOTH_ATTACK3: animNumber_t = 115; //# Attack with blaster rifle
pub const BOTH_ATTACK4: animNumber_t = 116; //# Attack with disruptor
pub const BOTH_ATTACK5: animNumber_t = 117; //# Another Rancor Attack
pub const BOTH_ATTACK6: animNumber_t = 118; //# Yet Another Rancor Attack
pub const BOTH_ATTACK7: animNumber_t = 119; //# Yet Another Rancor Attack
pub const BOTH_ATTACK10: animNumber_t = 120; //# Attack with thermal det
pub const BOTH_ATTACK11: animNumber_t = 121; //# "Attack" with tripmine and detpack
pub const BOTH_MELEE1: animNumber_t = 122; //# First melee attack
pub const BOTH_MELEE2: animNumber_t = 123; //# Second melee attack
pub const BOTH_THERMAL_READY: animNumber_t = 124; //# pull back with thermal
pub const BOTH_THERMAL_THROW: animNumber_t = 125; //# throw thermal
//* #sep BOTH_ SABER ANIMS
//Saber attack anims - power level 1
pub const BOTH_A1_T__B_: animNumber_t = 126; //# Fast weak vertical attack top to bottom
pub const BOTH_A1__L__R: animNumber_t = 127; //# Fast weak horizontal attack left to right
pub const BOTH_A1__R__L: animNumber_t = 128; //# Fast weak horizontal attack right to left
pub const BOTH_A1_TL_BR: animNumber_t = 129; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A1_BR_TL: animNumber_t = 130; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A1_BL_TR: animNumber_t = 131; //# Fast weak diagonal attack bottom left to top right
pub const BOTH_A1_TR_BL: animNumber_t = 132; //# Fast weak diagonal attack bottom left to right
//Saber Arc and Spin Transitions
pub const BOTH_T1_BR__R: animNumber_t = 133; //# Fast arc bottom right to right
pub const BOTH_T1_BR_TL: animNumber_t = 134; //# Fast weak spin bottom right to top left
pub const BOTH_T1_BR__L: animNumber_t = 135; //# Fast weak spin bottom right to left
pub const BOTH_T1_BR_BL: animNumber_t = 136; //# Fast weak spin bottom right to bottom left
pub const BOTH_T1__R_TR: animNumber_t = 137; //# Fast arc right to top right
pub const BOTH_T1__R_TL: animNumber_t = 138; //# Fast arc right to top left
pub const BOTH_T1__R__L: animNumber_t = 139; //# Fast weak spin right to left
pub const BOTH_T1__R_BL: animNumber_t = 140; //# Fast weak spin right to bottom left
pub const BOTH_T1_TR_BR: animNumber_t = 141; //# Fast arc top right to bottom right
pub const BOTH_T1_TR_TL: animNumber_t = 142; //# Fast arc top right to top left
pub const BOTH_T1_TR__L: animNumber_t = 143; //# Fast arc top right to left
pub const BOTH_T1_TR_BL: animNumber_t = 144; //# Fast weak spin top right to bottom left
pub const BOTH_T1_T__BR: animNumber_t = 145; //# Fast arc top to bottom right
pub const BOTH_T1_T___R: animNumber_t = 146; //# Fast arc top to right
pub const BOTH_T1_T__TR: animNumber_t = 147; //# Fast arc top to top right
pub const BOTH_T1_T__TL: animNumber_t = 148; //# Fast arc top to top left
pub const BOTH_T1_T___L: animNumber_t = 149; //# Fast arc top to left
pub const BOTH_T1_T__BL: animNumber_t = 150; //# Fast arc top to bottom left
pub const BOTH_T1_TL_BR: animNumber_t = 151; //# Fast weak spin top left to bottom right
pub const BOTH_T1_TL_BL: animNumber_t = 152; //# Fast arc top left to bottom left
pub const BOTH_T1__L_BR: animNumber_t = 153; //# Fast weak spin left to bottom right
pub const BOTH_T1__L__R: animNumber_t = 154; //# Fast weak spin left to right
pub const BOTH_T1__L_TL: animNumber_t = 155; //# Fast arc left to top left
pub const BOTH_T1_BL_BR: animNumber_t = 156; //# Fast weak spin bottom left to bottom right
pub const BOTH_T1_BL__R: animNumber_t = 157; //# Fast weak spin bottom left to right
pub const BOTH_T1_BL_TR: animNumber_t = 158; //# Fast weak spin bottom left to top right
pub const BOTH_T1_BL__L: animNumber_t = 159; //# Fast arc bottom left to left
//Saber Arc Transitions that use existing animations played backwards
pub const BOTH_T1_BR_TR: animNumber_t = 160; //# Fast arc bottom right to top right		(use: BOTH_T1_TR_BR)
pub const BOTH_T1_BR_T_: animNumber_t = 161; //# Fast arc bottom right to top			(use: BOTH_T1_T__BR)
pub const BOTH_T1__R_BR: animNumber_t = 162; //# Fast arc right to bottom right			(use: BOTH_T1_BR__R)
pub const BOTH_T1__R_T_: animNumber_t = 163; //# Fast ar right to top				(use: BOTH_T1_T___R)
pub const BOTH_T1_TR__R: animNumber_t = 164; //# Fast arc top right to right			(use: BOTH_T1__R_TR)
pub const BOTH_T1_TR_T_: animNumber_t = 165; //# Fast arc top right to top				(use: BOTH_T1_T__TR)
pub const BOTH_T1_TL__R: animNumber_t = 166; //# Fast arc top left to right			(use: BOTH_T1__R_TL)
pub const BOTH_T1_TL_TR: animNumber_t = 167; //# Fast arc top left to top right			(use: BOTH_T1_TR_TL)
pub const BOTH_T1_TL_T_: animNumber_t = 168; //# Fast arc top left to top				(use: BOTH_T1_T__TL)
pub const BOTH_T1_TL__L: animNumber_t = 169; //# Fast arc top left to left				(use: BOTH_T1__L_TL)
pub const BOTH_T1__L_TR: animNumber_t = 170; //# Fast arc left to top right			(use: BOTH_T1_TR__L)
pub const BOTH_T1__L_T_: animNumber_t = 171; //# Fast arc left to top				(use: BOTH_T1_T___L)
pub const BOTH_T1__L_BL: animNumber_t = 172; //# Fast arc left to bottom left			(use: BOTH_T1_BL__L)
pub const BOTH_T1_BL_T_: animNumber_t = 173; //# Fast arc bottom left to top			(use: BOTH_T1_T__BL)
pub const BOTH_T1_BL_TL: animNumber_t = 174; //# Fast arc bottom left to top left		(use: BOTH_T1_TL_BL)
//Saber Attack Start Transitions
pub const BOTH_S1_S1_T_: animNumber_t = 175; //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
pub const BOTH_S1_S1__L: animNumber_t = 176; //# Fast plain transition from stance1 to left-to-right Fast weak attack
pub const BOTH_S1_S1__R: animNumber_t = 177; //# Fast plain transition from stance1 to right-to-left Fast weak attack
pub const BOTH_S1_S1_TL: animNumber_t = 178; //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
pub const BOTH_S1_S1_BR: animNumber_t = 179; //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
pub const BOTH_S1_S1_BL: animNumber_t = 180; //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
pub const BOTH_S1_S1_TR: animNumber_t = 181; //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
//Saber Attack Return Transitions
pub const BOTH_R1_B__S1: animNumber_t = 182; //# Fast plain transition from top-to-bottom Fast weak attack to stance1
pub const BOTH_R1__L_S1: animNumber_t = 183; //# Fast plain transition from left-to-right Fast weak attack to stance1
pub const BOTH_R1__R_S1: animNumber_t = 184; //# Fast plain transition from right-to-left Fast weak attack to stance1
pub const BOTH_R1_TL_S1: animNumber_t = 185; //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
pub const BOTH_R1_BR_S1: animNumber_t = 186; //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
pub const BOTH_R1_BL_S1: animNumber_t = 187; //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
pub const BOTH_R1_TR_S1: animNumber_t = 188; //# Fast plain transition from top-right-to-bottom-left Fast weak attack
//Saber Attack Bounces (first 4 frames of an attack, played backwards)
pub const BOTH_B1_BR___: animNumber_t = 189; //# Bounce-back if attack from BR is blocked
pub const BOTH_B1__R___: animNumber_t = 190; //# Bounce-back if attack from R is blocked
pub const BOTH_B1_TR___: animNumber_t = 191; //# Bounce-back if attack from TR is blocked
pub const BOTH_B1_T____: animNumber_t = 192; //# Bounce-back if attack from T is blocked
pub const BOTH_B1_TL___: animNumber_t = 193; //# Bounce-back if attack from TL is blocked
pub const BOTH_B1__L___: animNumber_t = 194; //# Bounce-back if attack from L is blocked
pub const BOTH_B1_BL___: animNumber_t = 195; //# Bounce-back if attack from BL is blocked
//Saber Attack Deflections (last 4 frames of an attack)
pub const BOTH_D1_BR___: animNumber_t = 196; //# Deflection toward BR
pub const BOTH_D1__R___: animNumber_t = 197; //# Deflection toward R
pub const BOTH_D1_TR___: animNumber_t = 198; //# Deflection toward TR
pub const BOTH_D1_TL___: animNumber_t = 199; //# Deflection toward TL
pub const BOTH_D1__L___: animNumber_t = 200; //# Deflection toward L
pub const BOTH_D1_BL___: animNumber_t = 201; //# Deflection toward BL
pub const BOTH_D1_B____: animNumber_t = 202; //# Deflection toward B
//Saber attack anims - power level 2
pub const BOTH_A2_T__B_: animNumber_t = 203; //# Fast weak vertical attack top to bottom
pub const BOTH_A2__L__R: animNumber_t = 204; //# Fast weak horizontal attack left to right
pub const BOTH_A2__R__L: animNumber_t = 205; //# Fast weak horizontal attack right to left
pub const BOTH_A2_TL_BR: animNumber_t = 206; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A2_BR_TL: animNumber_t = 207; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A2_BL_TR: animNumber_t = 208; //# Fast weak diagonal attack bottom left to top right
pub const BOTH_A2_TR_BL: animNumber_t = 209; //# Fast weak diagonal attack bottom left to right
//Saber Arc and Spin Transitions
pub const BOTH_T2_BR__R: animNumber_t = 210; //# Fast arc bottom right to right
pub const BOTH_T2_BR_TL: animNumber_t = 211; //# Fast weak spin bottom right to top left
pub const BOTH_T2_BR__L: animNumber_t = 212; //# Fast weak spin bottom right to left
pub const BOTH_T2_BR_BL: animNumber_t = 213; //# Fast weak spin bottom right to bottom left
pub const BOTH_T2__R_TR: animNumber_t = 214; //# Fast arc right to top right
pub const BOTH_T2__R_TL: animNumber_t = 215; //# Fast arc right to top left
pub const BOTH_T2__R__L: animNumber_t = 216; //# Fast weak spin right to left
pub const BOTH_T2__R_BL: animNumber_t = 217; //# Fast weak spin right to bottom left
pub const BOTH_T2_TR_BR: animNumber_t = 218; //# Fast arc top right to bottom right
pub const BOTH_T2_TR_TL: animNumber_t = 219; //# Fast arc top right to top left
pub const BOTH_T2_TR__L: animNumber_t = 220; //# Fast arc top right to left
pub const BOTH_T2_TR_BL: animNumber_t = 221; //# Fast weak spin top right to bottom left
pub const BOTH_T2_T__BR: animNumber_t = 222; //# Fast arc top to bottom right
pub const BOTH_T2_T___R: animNumber_t = 223; //# Fast arc top to right
pub const BOTH_T2_T__TR: animNumber_t = 224; //# Fast arc top to top right
pub const BOTH_T2_T__TL: animNumber_t = 225; //# Fast arc top to top left
pub const BOTH_T2_T___L: animNumber_t = 226; //# Fast arc top to left
pub const BOTH_T2_T__BL: animNumber_t = 227; //# Fast arc top to bottom left
pub const BOTH_T2_TL_BR: animNumber_t = 228; //# Fast weak spin top left to bottom right
pub const BOTH_T2_TL_BL: animNumber_t = 229; //# Fast arc top left to bottom left
pub const BOTH_T2__L_BR: animNumber_t = 230; //# Fast weak spin left to bottom right
pub const BOTH_T2__L__R: animNumber_t = 231; //# Fast weak spin left to right
pub const BOTH_T2__L_TL: animNumber_t = 232; //# Fast arc left to top left
pub const BOTH_T2_BL_BR: animNumber_t = 233; //# Fast weak spin bottom left to bottom right
pub const BOTH_T2_BL__R: animNumber_t = 234; //# Fast weak spin bottom left to right
pub const BOTH_T2_BL_TR: animNumber_t = 235; //# Fast weak spin bottom left to top right
pub const BOTH_T2_BL__L: animNumber_t = 236; //# Fast arc bottom left to left
//Saber Arc Transitions that use existing animations played backwards
pub const BOTH_T2_BR_TR: animNumber_t = 237; //# Fast arc bottom right to top right		(use: BOTH_T2_TR_BR)
pub const BOTH_T2_BR_T_: animNumber_t = 238; //# Fast arc bottom right to top			(use: BOTH_T2_T__BR)
pub const BOTH_T2__R_BR: animNumber_t = 239; //# Fast arc right to bottom right			(use: BOTH_T2_BR__R)
pub const BOTH_T2__R_T_: animNumber_t = 240; //# Fast ar right to top				(use: BOTH_T2_T___R)
pub const BOTH_T2_TR__R: animNumber_t = 241; //# Fast arc top right to right			(use: BOTH_T2__R_TR)
pub const BOTH_T2_TR_T_: animNumber_t = 242; //# Fast arc top right to top				(use: BOTH_T2_T__TR)
pub const BOTH_T2_TL__R: animNumber_t = 243; //# Fast arc top left to right			(use: BOTH_T2__R_TL)
pub const BOTH_T2_TL_TR: animNumber_t = 244; //# Fast arc top left to top right			(use: BOTH_T2_TR_TL)
pub const BOTH_T2_TL_T_: animNumber_t = 245; //# Fast arc top left to top				(use: BOTH_T2_T__TL)
pub const BOTH_T2_TL__L: animNumber_t = 246; //# Fast arc top left to left				(use: BOTH_T2__L_TL)
pub const BOTH_T2__L_TR: animNumber_t = 247; //# Fast arc left to top right			(use: BOTH_T2_TR__L)
pub const BOTH_T2__L_T_: animNumber_t = 248; //# Fast arc left to top				(use: BOTH_T2_T___L)
pub const BOTH_T2__L_BL: animNumber_t = 249; //# Fast arc left to bottom left			(use: BOTH_T2_BL__L)
pub const BOTH_T2_BL_T_: animNumber_t = 250; //# Fast arc bottom left to top			(use: BOTH_T2_T__BL)
pub const BOTH_T2_BL_TL: animNumber_t = 251; //# Fast arc bottom left to top left		(use: BOTH_T2_TL_BL)
//Saber Attack Start Transitions
pub const BOTH_S2_S1_T_: animNumber_t = 252; //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
pub const BOTH_S2_S1__L: animNumber_t = 253; //# Fast plain transition from stance1 to left-to-right Fast weak attack
pub const BOTH_S2_S1__R: animNumber_t = 254; //# Fast plain transition from stance1 to right-to-left Fast weak attack
pub const BOTH_S2_S1_TL: animNumber_t = 255; //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
pub const BOTH_S2_S1_BR: animNumber_t = 256; //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
pub const BOTH_S2_S1_BL: animNumber_t = 257; //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
pub const BOTH_S2_S1_TR: animNumber_t = 258; //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
//Saber Attack Return Transitions
pub const BOTH_R2_B__S1: animNumber_t = 259; //# Fast plain transition from top-to-bottom Fast weak attack to stance1
pub const BOTH_R2__L_S1: animNumber_t = 260; //# Fast plain transition from left-to-right Fast weak attack to stance1
pub const BOTH_R2__R_S1: animNumber_t = 261; //# Fast plain transition from right-to-left Fast weak attack to stance1
pub const BOTH_R2_TL_S1: animNumber_t = 262; //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
pub const BOTH_R2_BR_S1: animNumber_t = 263; //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
pub const BOTH_R2_BL_S1: animNumber_t = 264; //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
pub const BOTH_R2_TR_S1: animNumber_t = 265; //# Fast plain transition from top-right-to-bottom-left Fast weak attack
//Saber Attack Bounces (first 4 frames of an attack, played backwards)
pub const BOTH_B2_BR___: animNumber_t = 266; //# Bounce-back if attack from BR is blocked
pub const BOTH_B2__R___: animNumber_t = 267; //# Bounce-back if attack from R is blocked
pub const BOTH_B2_TR___: animNumber_t = 268; //# Bounce-back if attack from TR is blocked
pub const BOTH_B2_T____: animNumber_t = 269; //# Bounce-back if attack from T is blocked
pub const BOTH_B2_TL___: animNumber_t = 270; //# Bounce-back if attack from TL is blocked
pub const BOTH_B2__L___: animNumber_t = 271; //# Bounce-back if attack from L is blocked
pub const BOTH_B2_BL___: animNumber_t = 272; //# Bounce-back if attack from BL is blocked
//Saber Attack Deflections (last 4 frames of an attack)
pub const BOTH_D2_BR___: animNumber_t = 273; //# Deflection toward BR
pub const BOTH_D2__R___: animNumber_t = 274; //# Deflection toward R
pub const BOTH_D2_TR___: animNumber_t = 275; //# Deflection toward TR
pub const BOTH_D2_TL___: animNumber_t = 276; //# Deflection toward TL
pub const BOTH_D2__L___: animNumber_t = 277; //# Deflection toward L
pub const BOTH_D2_BL___: animNumber_t = 278; //# Deflection toward BL
pub const BOTH_D2_B____: animNumber_t = 279; //# Deflection toward B
//Saber attack anims - power level 3
pub const BOTH_A3_T__B_: animNumber_t = 280; //# Fast weak vertical attack top to bottom
pub const BOTH_A3__L__R: animNumber_t = 281; //# Fast weak horizontal attack left to right
pub const BOTH_A3__R__L: animNumber_t = 282; //# Fast weak horizontal attack right to left
pub const BOTH_A3_TL_BR: animNumber_t = 283; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A3_BR_TL: animNumber_t = 284; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A3_BL_TR: animNumber_t = 285; //# Fast weak diagonal attack bottom left to top right
pub const BOTH_A3_TR_BL: animNumber_t = 286; //# Fast weak diagonal attack bottom left to right
//Saber Arc and Spin Transitions
pub const BOTH_T3_BR__R: animNumber_t = 287; //# Fast arc bottom right to right
pub const BOTH_T3_BR_TL: animNumber_t = 288; //# Fast weak spin bottom right to top left
pub const BOTH_T3_BR__L: animNumber_t = 289; //# Fast weak spin bottom right to left
pub const BOTH_T3_BR_BL: animNumber_t = 290; //# Fast weak spin bottom right to bottom left
pub const BOTH_T3__R_TR: animNumber_t = 291; //# Fast arc right to top right
pub const BOTH_T3__R_TL: animNumber_t = 292; //# Fast arc right to top left
pub const BOTH_T3__R__L: animNumber_t = 293; //# Fast weak spin right to left
pub const BOTH_T3__R_BL: animNumber_t = 294; //# Fast weak spin right to bottom left
pub const BOTH_T3_TR_BR: animNumber_t = 295; //# Fast arc top right to bottom right
pub const BOTH_T3_TR_TL: animNumber_t = 296; //# Fast arc top right to top left
pub const BOTH_T3_TR__L: animNumber_t = 297; //# Fast arc top right to left
pub const BOTH_T3_TR_BL: animNumber_t = 298; //# Fast weak spin top right to bottom left
pub const BOTH_T3_T__BR: animNumber_t = 299; //# Fast arc top to bottom right
pub const BOTH_T3_T___R: animNumber_t = 300; //# Fast arc top to right
pub const BOTH_T3_T__TR: animNumber_t = 301; //# Fast arc top to top right
pub const BOTH_T3_T__TL: animNumber_t = 302; //# Fast arc top to top left
pub const BOTH_T3_T___L: animNumber_t = 303; //# Fast arc top to left
pub const BOTH_T3_T__BL: animNumber_t = 304; //# Fast arc top to bottom left
pub const BOTH_T3_TL_BR: animNumber_t = 305; //# Fast weak spin top left to bottom right
pub const BOTH_T3_TL_BL: animNumber_t = 306; //# Fast arc top left to bottom left
pub const BOTH_T3__L_BR: animNumber_t = 307; //# Fast weak spin left to bottom right
pub const BOTH_T3__L__R: animNumber_t = 308; //# Fast weak spin left to right
pub const BOTH_T3__L_TL: animNumber_t = 309; //# Fast arc left to top left
pub const BOTH_T3_BL_BR: animNumber_t = 310; //# Fast weak spin bottom left to bottom right
pub const BOTH_T3_BL__R: animNumber_t = 311; //# Fast weak spin bottom left to right
pub const BOTH_T3_BL_TR: animNumber_t = 312; //# Fast weak spin bottom left to top right
pub const BOTH_T3_BL__L: animNumber_t = 313; //# Fast arc bottom left to left
//Saber Arc Transitions that use existing animations played backwards
pub const BOTH_T3_BR_TR: animNumber_t = 314; //# Fast arc bottom right to top right		(use: BOTH_T3_TR_BR)
pub const BOTH_T3_BR_T_: animNumber_t = 315; //# Fast arc bottom right to top			(use: BOTH_T3_T__BR)
pub const BOTH_T3__R_BR: animNumber_t = 316; //# Fast arc right to bottom right			(use: BOTH_T3_BR__R)
pub const BOTH_T3__R_T_: animNumber_t = 317; //# Fast ar right to top				(use: BOTH_T3_T___R)
pub const BOTH_T3_TR__R: animNumber_t = 318; //# Fast arc top right to right			(use: BOTH_T3__R_TR)
pub const BOTH_T3_TR_T_: animNumber_t = 319; //# Fast arc top right to top				(use: BOTH_T3_T__TR)
pub const BOTH_T3_TL__R: animNumber_t = 320; //# Fast arc top left to right			(use: BOTH_T3__R_TL)
pub const BOTH_T3_TL_TR: animNumber_t = 321; //# Fast arc top left to top right			(use: BOTH_T3_TR_TL)
pub const BOTH_T3_TL_T_: animNumber_t = 322; //# Fast arc top left to top				(use: BOTH_T3_T__TL)
pub const BOTH_T3_TL__L: animNumber_t = 323; //# Fast arc top left to left				(use: BOTH_T3__L_TL)
pub const BOTH_T3__L_TR: animNumber_t = 324; //# Fast arc left to top right			(use: BOTH_T3_TR__L)
pub const BOTH_T3__L_T_: animNumber_t = 325; //# Fast arc left to top				(use: BOTH_T3_T___L)
pub const BOTH_T3__L_BL: animNumber_t = 326; //# Fast arc left to bottom left			(use: BOTH_T3_BL__L)
pub const BOTH_T3_BL_T_: animNumber_t = 327; //# Fast arc bottom left to top			(use: BOTH_T3_T__BL)
pub const BOTH_T3_BL_TL: animNumber_t = 328; //# Fast arc bottom left to top left		(use: BOTH_T3_TL_BL)
//Saber Attack Start Transitions
pub const BOTH_S3_S1_T_: animNumber_t = 329; //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
pub const BOTH_S3_S1__L: animNumber_t = 330; //# Fast plain transition from stance1 to left-to-right Fast weak attack
pub const BOTH_S3_S1__R: animNumber_t = 331; //# Fast plain transition from stance1 to right-to-left Fast weak attack
pub const BOTH_S3_S1_TL: animNumber_t = 332; //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
pub const BOTH_S3_S1_BR: animNumber_t = 333; //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
pub const BOTH_S3_S1_BL: animNumber_t = 334; //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
pub const BOTH_S3_S1_TR: animNumber_t = 335; //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
//Saber Attack Return Transitions
pub const BOTH_R3_B__S1: animNumber_t = 336; //# Fast plain transition from top-to-bottom Fast weak attack to stance1
pub const BOTH_R3__L_S1: animNumber_t = 337; //# Fast plain transition from left-to-right Fast weak attack to stance1
pub const BOTH_R3__R_S1: animNumber_t = 338; //# Fast plain transition from right-to-left Fast weak attack to stance1
pub const BOTH_R3_TL_S1: animNumber_t = 339; //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
pub const BOTH_R3_BR_S1: animNumber_t = 340; //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
pub const BOTH_R3_BL_S1: animNumber_t = 341; //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
pub const BOTH_R3_TR_S1: animNumber_t = 342; //# Fast plain transition from top-right-to-bottom-left Fast weak attack
//Saber Attack Bounces (first 4 frames of an attack, played backwards)
pub const BOTH_B3_BR___: animNumber_t = 343; //# Bounce-back if attack from BR is blocked
pub const BOTH_B3__R___: animNumber_t = 344; //# Bounce-back if attack from R is blocked
pub const BOTH_B3_TR___: animNumber_t = 345; //# Bounce-back if attack from TR is blocked
pub const BOTH_B3_T____: animNumber_t = 346; //# Bounce-back if attack from T is blocked
pub const BOTH_B3_TL___: animNumber_t = 347; //# Bounce-back if attack from TL is blocked
pub const BOTH_B3__L___: animNumber_t = 348; //# Bounce-back if attack from L is blocked
pub const BOTH_B3_BL___: animNumber_t = 349; //# Bounce-back if attack from BL is blocked
//Saber Attack Deflections (last 4 frames of an attack)
pub const BOTH_D3_BR___: animNumber_t = 350; //# Deflection toward BR
pub const BOTH_D3__R___: animNumber_t = 351; //# Deflection toward R
pub const BOTH_D3_TR___: animNumber_t = 352; //# Deflection toward TR
pub const BOTH_D3_TL___: animNumber_t = 353; //# Deflection toward TL
pub const BOTH_D3__L___: animNumber_t = 354; //# Deflection toward L
pub const BOTH_D3_BL___: animNumber_t = 355; //# Deflection toward BL
pub const BOTH_D3_B____: animNumber_t = 356; //# Deflection toward B
//Saber attack anims - power level 4  - Desann's
pub const BOTH_A4_T__B_: animNumber_t = 357; //# Fast weak vertical attack top to bottom
pub const BOTH_A4__L__R: animNumber_t = 358; //# Fast weak horizontal attack left to right
pub const BOTH_A4__R__L: animNumber_t = 359; //# Fast weak horizontal attack right to left
pub const BOTH_A4_TL_BR: animNumber_t = 360; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A4_BR_TL: animNumber_t = 361; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A4_BL_TR: animNumber_t = 362; //# Fast weak diagonal attack bottom left to top right
pub const BOTH_A4_TR_BL: animNumber_t = 363; //# Fast weak diagonal attack bottom left to right
//Saber Arc and Spin Transitions
pub const BOTH_T4_BR__R: animNumber_t = 364; //# Fast arc bottom right to right
pub const BOTH_T4_BR_TL: animNumber_t = 365; //# Fast weak spin bottom right to top left
pub const BOTH_T4_BR__L: animNumber_t = 366; //# Fast weak spin bottom right to left
pub const BOTH_T4_BR_BL: animNumber_t = 367; //# Fast weak spin bottom right to bottom left
pub const BOTH_T4__R_TR: animNumber_t = 368; //# Fast arc right to top right
pub const BOTH_T4__R_TL: animNumber_t = 369; //# Fast arc right to top left
pub const BOTH_T4__R__L: animNumber_t = 370; //# Fast weak spin right to left
pub const BOTH_T4__R_BL: animNumber_t = 371; //# Fast weak spin right to bottom left
pub const BOTH_T4_TR_BR: animNumber_t = 372; //# Fast arc top right to bottom right
pub const BOTH_T4_TR_TL: animNumber_t = 373; //# Fast arc top right to top left
pub const BOTH_T4_TR__L: animNumber_t = 374; //# Fast arc top right to left
pub const BOTH_T4_TR_BL: animNumber_t = 375; //# Fast weak spin top right to bottom left
pub const BOTH_T4_T__BR: animNumber_t = 376; //# Fast arc top to bottom right
pub const BOTH_T4_T___R: animNumber_t = 377; //# Fast arc top to right
pub const BOTH_T4_T__TR: animNumber_t = 378; //# Fast arc top to top right
pub const BOTH_T4_T__TL: animNumber_t = 379; //# Fast arc top to top left
pub const BOTH_T4_T___L: animNumber_t = 380; //# Fast arc top to left
pub const BOTH_T4_T__BL: animNumber_t = 381; //# Fast arc top to bottom left
pub const BOTH_T4_TL_BR: animNumber_t = 382; //# Fast weak spin top left to bottom right
pub const BOTH_T4_TL_BL: animNumber_t = 383; //# Fast arc top left to bottom left
pub const BOTH_T4__L_BR: animNumber_t = 384; //# Fast weak spin left to bottom right
pub const BOTH_T4__L__R: animNumber_t = 385; //# Fast weak spin left to right
pub const BOTH_T4__L_TL: animNumber_t = 386; //# Fast arc left to top left
pub const BOTH_T4_BL_BR: animNumber_t = 387; //# Fast weak spin bottom left to bottom right
pub const BOTH_T4_BL__R: animNumber_t = 388; //# Fast weak spin bottom left to right
pub const BOTH_T4_BL_TR: animNumber_t = 389; //# Fast weak spin bottom left to top right
pub const BOTH_T4_BL__L: animNumber_t = 390; //# Fast arc bottom left to left
//Saber Arc Transitions that use existing animations played backwards
pub const BOTH_T4_BR_TR: animNumber_t = 391; //# Fast arc bottom right to top right		(use: BOTH_T4_TR_BR)
pub const BOTH_T4_BR_T_: animNumber_t = 392; //# Fast arc bottom right to top			(use: BOTH_T4_T__BR)
pub const BOTH_T4__R_BR: animNumber_t = 393; //# Fast arc right to bottom right			(use: BOTH_T4_BR__R)
pub const BOTH_T4__R_T_: animNumber_t = 394; //# Fast ar right to top				(use: BOTH_T4_T___R)
pub const BOTH_T4_TR__R: animNumber_t = 395; //# Fast arc top right to right			(use: BOTH_T4__R_TR)
pub const BOTH_T4_TR_T_: animNumber_t = 396; //# Fast arc top right to top				(use: BOTH_T4_T__TR)
pub const BOTH_T4_TL__R: animNumber_t = 397; //# Fast arc top left to right			(use: BOTH_T4__R_TL)
pub const BOTH_T4_TL_TR: animNumber_t = 398; //# Fast arc top left to top right			(use: BOTH_T4_TR_TL)
pub const BOTH_T4_TL_T_: animNumber_t = 399; //# Fast arc top left to top				(use: BOTH_T4_T__TL)
pub const BOTH_T4_TL__L: animNumber_t = 400; //# Fast arc top left to left				(use: BOTH_T4__L_TL)
pub const BOTH_T4__L_TR: animNumber_t = 401; //# Fast arc left to top right			(use: BOTH_T4_TR__L)
pub const BOTH_T4__L_T_: animNumber_t = 402; //# Fast arc left to top				(use: BOTH_T4_T___L)
pub const BOTH_T4__L_BL: animNumber_t = 403; //# Fast arc left to bottom left			(use: BOTH_T4_BL__L)
pub const BOTH_T4_BL_T_: animNumber_t = 404; //# Fast arc bottom left to top			(use: BOTH_T4_T__BL)
pub const BOTH_T4_BL_TL: animNumber_t = 405; //# Fast arc bottom left to top left		(use: BOTH_T4_TL_BL)
//Saber Attack Start Transitions
pub const BOTH_S4_S1_T_: animNumber_t = 406; //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
pub const BOTH_S4_S1__L: animNumber_t = 407; //# Fast plain transition from stance1 to left-to-right Fast weak attack
pub const BOTH_S4_S1__R: animNumber_t = 408; //# Fast plain transition from stance1 to right-to-left Fast weak attack
pub const BOTH_S4_S1_TL: animNumber_t = 409; //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
pub const BOTH_S4_S1_BR: animNumber_t = 410; //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
pub const BOTH_S4_S1_BL: animNumber_t = 411; //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
pub const BOTH_S4_S1_TR: animNumber_t = 412; //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
//Saber Attack Return Transitions
pub const BOTH_R4_B__S1: animNumber_t = 413; //# Fast plain transition from top-to-bottom Fast weak attack to stance1
pub const BOTH_R4__L_S1: animNumber_t = 414; //# Fast plain transition from left-to-right Fast weak attack to stance1
pub const BOTH_R4__R_S1: animNumber_t = 415; //# Fast plain transition from right-to-left Fast weak attack to stance1
pub const BOTH_R4_TL_S1: animNumber_t = 416; //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
pub const BOTH_R4_BR_S1: animNumber_t = 417; //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
pub const BOTH_R4_BL_S1: animNumber_t = 418; //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
pub const BOTH_R4_TR_S1: animNumber_t = 419; //# Fast plain transition from top-right-to-bottom-left Fast weak attack
//Saber Attack Bounces (first 4 frames of an attack, played backwards)
pub const BOTH_B4_BR___: animNumber_t = 420; //# Bounce-back if attack from BR is blocked
pub const BOTH_B4__R___: animNumber_t = 421; //# Bounce-back if attack from R is blocked
pub const BOTH_B4_TR___: animNumber_t = 422; //# Bounce-back if attack from TR is blocked
pub const BOTH_B4_T____: animNumber_t = 423; //# Bounce-back if attack from T is blocked
pub const BOTH_B4_TL___: animNumber_t = 424; //# Bounce-back if attack from TL is blocked
pub const BOTH_B4__L___: animNumber_t = 425; //# Bounce-back if attack from L is blocked
pub const BOTH_B4_BL___: animNumber_t = 426; //# Bounce-back if attack from BL is blocked
//Saber Attack Deflections (last 4 frames of an attack)
pub const BOTH_D4_BR___: animNumber_t = 427; //# Deflection toward BR
pub const BOTH_D4__R___: animNumber_t = 428; //# Deflection toward R
pub const BOTH_D4_TR___: animNumber_t = 429; //# Deflection toward TR
pub const BOTH_D4_TL___: animNumber_t = 430; //# Deflection toward TL
pub const BOTH_D4__L___: animNumber_t = 431; //# Deflection toward L
pub const BOTH_D4_BL___: animNumber_t = 432; //# Deflection toward BL
pub const BOTH_D4_B____: animNumber_t = 433; //# Deflection toward B
//Saber attack anims - power level 5  - Tavion's
pub const BOTH_A5_T__B_: animNumber_t = 434; //# Fast weak vertical attack top to bottom
pub const BOTH_A5__L__R: animNumber_t = 435; //# Fast weak horizontal attack left to right
pub const BOTH_A5__R__L: animNumber_t = 436; //# Fast weak horizontal attack right to left
pub const BOTH_A5_TL_BR: animNumber_t = 437; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A5_BR_TL: animNumber_t = 438; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A5_BL_TR: animNumber_t = 439; //# Fast weak diagonal attack bottom left to top right
pub const BOTH_A5_TR_BL: animNumber_t = 440; //# Fast weak diagonal attack bottom left to right
//Saber Arc and Spin Transitions
pub const BOTH_T5_BR__R: animNumber_t = 441; //# Fast arc bottom right to right
pub const BOTH_T5_BR_TL: animNumber_t = 442; //# Fast weak spin bottom right to top left
pub const BOTH_T5_BR__L: animNumber_t = 443; //# Fast weak spin bottom right to left
pub const BOTH_T5_BR_BL: animNumber_t = 444; //# Fast weak spin bottom right to bottom left
pub const BOTH_T5__R_TR: animNumber_t = 445; //# Fast arc right to top right
pub const BOTH_T5__R_TL: animNumber_t = 446; //# Fast arc right to top left
pub const BOTH_T5__R__L: animNumber_t = 447; //# Fast weak spin right to left
pub const BOTH_T5__R_BL: animNumber_t = 448; //# Fast weak spin right to bottom left
pub const BOTH_T5_TR_BR: animNumber_t = 449; //# Fast arc top right to bottom right
pub const BOTH_T5_TR_TL: animNumber_t = 450; //# Fast arc top right to top left
pub const BOTH_T5_TR__L: animNumber_t = 451; //# Fast arc top right to left
pub const BOTH_T5_TR_BL: animNumber_t = 452; //# Fast weak spin top right to bottom left
pub const BOTH_T5_T__BR: animNumber_t = 453; //# Fast arc top to bottom right
pub const BOTH_T5_T___R: animNumber_t = 454; //# Fast arc top to right
pub const BOTH_T5_T__TR: animNumber_t = 455; //# Fast arc top to top right
pub const BOTH_T5_T__TL: animNumber_t = 456; //# Fast arc top to top left
pub const BOTH_T5_T___L: animNumber_t = 457; //# Fast arc top to left
pub const BOTH_T5_T__BL: animNumber_t = 458; //# Fast arc top to bottom left
pub const BOTH_T5_TL_BR: animNumber_t = 459; //# Fast weak spin top left to bottom right
pub const BOTH_T5_TL_BL: animNumber_t = 460; //# Fast arc top left to bottom left
pub const BOTH_T5__L_BR: animNumber_t = 461; //# Fast weak spin left to bottom right
pub const BOTH_T5__L__R: animNumber_t = 462; //# Fast weak spin left to right
pub const BOTH_T5__L_TL: animNumber_t = 463; //# Fast arc left to top left
pub const BOTH_T5_BL_BR: animNumber_t = 464; //# Fast weak spin bottom left to bottom right
pub const BOTH_T5_BL__R: animNumber_t = 465; //# Fast weak spin bottom left to right
pub const BOTH_T5_BL_TR: animNumber_t = 466; //# Fast weak spin bottom left to top right
pub const BOTH_T5_BL__L: animNumber_t = 467; //# Fast arc bottom left to left
//Saber Arc Transitions that use existing animations played backwards
pub const BOTH_T5_BR_TR: animNumber_t = 468; //# Fast arc bottom right to top right		(use: BOTH_T5_TR_BR)
pub const BOTH_T5_BR_T_: animNumber_t = 469; //# Fast arc bottom right to top			(use: BOTH_T5_T__BR)
pub const BOTH_T5__R_BR: animNumber_t = 470; //# Fast arc right to bottom right			(use: BOTH_T5_BR__R)
pub const BOTH_T5__R_T_: animNumber_t = 471; //# Fast ar right to top				(use: BOTH_T5_T___R)
pub const BOTH_T5_TR__R: animNumber_t = 472; //# Fast arc top right to right			(use: BOTH_T5__R_TR)
pub const BOTH_T5_TR_T_: animNumber_t = 473; //# Fast arc top right to top				(use: BOTH_T5_T__TR)
pub const BOTH_T5_TL__R: animNumber_t = 474; //# Fast arc top left to right			(use: BOTH_T5__R_TL)
pub const BOTH_T5_TL_TR: animNumber_t = 475; //# Fast arc top left to top right			(use: BOTH_T5_TR_TL)
pub const BOTH_T5_TL_T_: animNumber_t = 476; //# Fast arc top left to top				(use: BOTH_T5_T__TL)
pub const BOTH_T5_TL__L: animNumber_t = 477; //# Fast arc top left to left				(use: BOTH_T5__L_TL)
pub const BOTH_T5__L_TR: animNumber_t = 478; //# Fast arc left to top right			(use: BOTH_T5_TR__L)
pub const BOTH_T5__L_T_: animNumber_t = 479; //# Fast arc left to top				(use: BOTH_T5_T___L)
pub const BOTH_T5__L_BL: animNumber_t = 480; //# Fast arc left to bottom left			(use: BOTH_T5_BL__L)
pub const BOTH_T5_BL_T_: animNumber_t = 481; //# Fast arc bottom left to top			(use: BOTH_T5_T__BL)
pub const BOTH_T5_BL_TL: animNumber_t = 482; //# Fast arc bottom left to top left		(use: BOTH_T5_TL_BL)
//Saber Attack Start Transitions
pub const BOTH_S5_S1_T_: animNumber_t = 483; //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
pub const BOTH_S5_S1__L: animNumber_t = 484; //# Fast plain transition from stance1 to left-to-right Fast weak attack
pub const BOTH_S5_S1__R: animNumber_t = 485; //# Fast plain transition from stance1 to right-to-left Fast weak attack
pub const BOTH_S5_S1_TL: animNumber_t = 486; //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
pub const BOTH_S5_S1_BR: animNumber_t = 487; //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
pub const BOTH_S5_S1_BL: animNumber_t = 488; //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
pub const BOTH_S5_S1_TR: animNumber_t = 489; //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
//Saber Attack Return Transitions
pub const BOTH_R5_B__S1: animNumber_t = 490; //# Fast plain transition from top-to-bottom Fast weak attack to stance1
pub const BOTH_R5__L_S1: animNumber_t = 491; //# Fast plain transition from left-to-right Fast weak attack to stance1
pub const BOTH_R5__R_S1: animNumber_t = 492; //# Fast plain transition from right-to-left Fast weak attack to stance1
pub const BOTH_R5_TL_S1: animNumber_t = 493; //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
pub const BOTH_R5_BR_S1: animNumber_t = 494; //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
pub const BOTH_R5_BL_S1: animNumber_t = 495; //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
pub const BOTH_R5_TR_S1: animNumber_t = 496; //# Fast plain transition from top-right-to-bottom-left Fast weak attack
//Saber Attack Bounces (first 4 frames of an attack, played backwards)
pub const BOTH_B5_BR___: animNumber_t = 497; //# Bounce-back if attack from BR is blocked
pub const BOTH_B5__R___: animNumber_t = 498; //# Bounce-back if attack from R is blocked
pub const BOTH_B5_TR___: animNumber_t = 499; //# Bounce-back if attack from TR is blocked
pub const BOTH_B5_T____: animNumber_t = 500; //# Bounce-back if attack from T is blocked
pub const BOTH_B5_TL___: animNumber_t = 501; //# Bounce-back if attack from TL is blocked
pub const BOTH_B5__L___: animNumber_t = 502; //# Bounce-back if attack from L is blocked
pub const BOTH_B5_BL___: animNumber_t = 503; //# Bounce-back if attack from BL is blocked
//Saber Attack Deflections (last 4 frames of an attack)
pub const BOTH_D5_BR___: animNumber_t = 504; //# Deflection toward BR
pub const BOTH_D5__R___: animNumber_t = 505; //# Deflection toward R
pub const BOTH_D5_TR___: animNumber_t = 506; //# Deflection toward TR
pub const BOTH_D5_TL___: animNumber_t = 507; //# Deflection toward TL
pub const BOTH_D5__L___: animNumber_t = 508; //# Deflection toward L
pub const BOTH_D5_BL___: animNumber_t = 509; //# Deflection toward BL
pub const BOTH_D5_B____: animNumber_t = 510; //# Deflection toward B
//Saber attack anims - power level 6
pub const BOTH_A6_T__B_: animNumber_t = 511; //# Fast weak vertical attack top to bottom
pub const BOTH_A6__L__R: animNumber_t = 512; //# Fast weak horizontal attack left to right
pub const BOTH_A6__R__L: animNumber_t = 513; //# Fast weak horizontal attack right to left
pub const BOTH_A6_TL_BR: animNumber_t = 514; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A6_BR_TL: animNumber_t = 515; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A6_BL_TR: animNumber_t = 516; //# Fast weak diagonal attack bottom left to top right
pub const BOTH_A6_TR_BL: animNumber_t = 517; //# Fast weak diagonal attack bottom left to right
//Saber Arc and Spin Transitions
pub const BOTH_T6_BR__R: animNumber_t = 518; //# Fast arc bottom right to right
pub const BOTH_T6_BR_TL: animNumber_t = 519; //# Fast weak spin bottom right to top left
pub const BOTH_T6_BR__L: animNumber_t = 520; //# Fast weak spin bottom right to left
pub const BOTH_T6_BR_BL: animNumber_t = 521; //# Fast weak spin bottom right to bottom left
pub const BOTH_T6__R_TR: animNumber_t = 522; //# Fast arc right to top right
pub const BOTH_T6__R_TL: animNumber_t = 523; //# Fast arc right to top left
pub const BOTH_T6__R__L: animNumber_t = 524; //# Fast weak spin right to left
pub const BOTH_T6__R_BL: animNumber_t = 525; //# Fast weak spin right to bottom left
pub const BOTH_T6_TR_BR: animNumber_t = 526; //# Fast arc top right to bottom right
pub const BOTH_T6_TR_TL: animNumber_t = 527; //# Fast arc top right to top left
pub const BOTH_T6_TR__L: animNumber_t = 528; //# Fast arc top right to left
pub const BOTH_T6_TR_BL: animNumber_t = 529; //# Fast weak spin top right to bottom left
pub const BOTH_T6_T__BR: animNumber_t = 530; //# Fast arc top to bottom right
pub const BOTH_T6_T___R: animNumber_t = 531; //# Fast arc top to right
pub const BOTH_T6_T__TR: animNumber_t = 532; //# Fast arc top to top right
pub const BOTH_T6_T__TL: animNumber_t = 533; //# Fast arc top to top left
pub const BOTH_T6_T___L: animNumber_t = 534; //# Fast arc top to left
pub const BOTH_T6_T__BL: animNumber_t = 535; //# Fast arc top to bottom left
pub const BOTH_T6_TL_BR: animNumber_t = 536; //# Fast weak spin top left to bottom right
pub const BOTH_T6_TL_BL: animNumber_t = 537; //# Fast arc top left to bottom left
pub const BOTH_T6__L_BR: animNumber_t = 538; //# Fast weak spin left to bottom right
pub const BOTH_T6__L__R: animNumber_t = 539; //# Fast weak spin left to right
pub const BOTH_T6__L_TL: animNumber_t = 540; //# Fast arc left to top left
pub const BOTH_T6_BL_BR: animNumber_t = 541; //# Fast weak spin bottom left to bottom right
pub const BOTH_T6_BL__R: animNumber_t = 542; //# Fast weak spin bottom left to right
pub const BOTH_T6_BL_TR: animNumber_t = 543; //# Fast weak spin bottom left to top right
pub const BOTH_T6_BL__L: animNumber_t = 544; //# Fast arc bottom left to left
//Saber Arc Transitions that use existing animations played backwards
pub const BOTH_T6_BR_TR: animNumber_t = 545; //# Fast arc bottom right to top right		(use: BOTH_T6_TR_BR)
pub const BOTH_T6_BR_T_: animNumber_t = 546; //# Fast arc bottom right to top			(use: BOTH_T6_T__BR)
pub const BOTH_T6__R_BR: animNumber_t = 547; //# Fast arc right to bottom right			(use: BOTH_T6_BR__R)
pub const BOTH_T6__R_T_: animNumber_t = 548; //# Fast ar right to top				(use: BOTH_T6_T___R)
pub const BOTH_T6_TR__R: animNumber_t = 549; //# Fast arc top right to right			(use: BOTH_T6__R_TR)
pub const BOTH_T6_TR_T_: animNumber_t = 550; //# Fast arc top right to top				(use: BOTH_T6_T__TR)
pub const BOTH_T6_TL__R: animNumber_t = 551; //# Fast arc top left to right			(use: BOTH_T6__R_TL)
pub const BOTH_T6_TL_TR: animNumber_t = 552; //# Fast arc top left to top right			(use: BOTH_T6_TR_TL)
pub const BOTH_T6_TL_T_: animNumber_t = 553; //# Fast arc top left to top				(use: BOTH_T6_T__TL)
pub const BOTH_T6_TL__L: animNumber_t = 554; //# Fast arc top left to left				(use: BOTH_T6__L_TL)
pub const BOTH_T6__L_TR: animNumber_t = 555; //# Fast arc left to top right			(use: BOTH_T6_TR__L)
pub const BOTH_T6__L_T_: animNumber_t = 556; //# Fast arc left to top				(use: BOTH_T6_T___L)
pub const BOTH_T6__L_BL: animNumber_t = 557; //# Fast arc left to bottom left			(use: BOTH_T6_BL__L)
pub const BOTH_T6_BL_T_: animNumber_t = 558; //# Fast arc bottom left to top			(use: BOTH_T6_T__BL)
pub const BOTH_T6_BL_TL: animNumber_t = 559; //# Fast arc bottom left to top left		(use: BOTH_T6_TL_BL)
//Saber Attack Start Transitions
pub const BOTH_S6_S6_T_: animNumber_t = 560; //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
pub const BOTH_S6_S6__L: animNumber_t = 561; //# Fast plain transition from stance1 to left-to-right Fast weak attack
pub const BOTH_S6_S6__R: animNumber_t = 562; //# Fast plain transition from stance1 to right-to-left Fast weak attack
pub const BOTH_S6_S6_TL: animNumber_t = 563; //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
pub const BOTH_S6_S6_BR: animNumber_t = 564; //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
pub const BOTH_S6_S6_BL: animNumber_t = 565; //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
pub const BOTH_S6_S6_TR: animNumber_t = 566; //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
//Saber Attack Return Transitions
pub const BOTH_R6_B__S6: animNumber_t = 567; //# Fast plain transition from top-to-bottom Fast weak attack to stance1
pub const BOTH_R6__L_S6: animNumber_t = 568; //# Fast plain transition from left-to-right Fast weak attack to stance1
pub const BOTH_R6__R_S6: animNumber_t = 569; //# Fast plain transition from right-to-left Fast weak attack to stance1
pub const BOTH_R6_TL_S6: animNumber_t = 570; //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
pub const BOTH_R6_BR_S6: animNumber_t = 571; //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
pub const BOTH_R6_BL_S6: animNumber_t = 572; //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
pub const BOTH_R6_TR_S6: animNumber_t = 573; //# Fast plain transition from top-right-to-bottom-left Fast weak attack
//Saber Attack Bounces (first 4 frames of an attack, played backwards)
pub const BOTH_B6_BR___: animNumber_t = 574; //# Bounce-back if attack from BR is blocked
pub const BOTH_B6__R___: animNumber_t = 575; //# Bounce-back if attack from R is blocked
pub const BOTH_B6_TR___: animNumber_t = 576; //# Bounce-back if attack from TR is blocked
pub const BOTH_B6_T____: animNumber_t = 577; //# Bounce-back if attack from T is blocked
pub const BOTH_B6_TL___: animNumber_t = 578; //# Bounce-back if attack from TL is blocked
pub const BOTH_B6__L___: animNumber_t = 579; //# Bounce-back if attack from L is blocked
pub const BOTH_B6_BL___: animNumber_t = 580; //# Bounce-back if attack from BL is blocked
//Saber Attack Deflections (last 4 frames of an attack)
pub const BOTH_D6_BR___: animNumber_t = 581; //# Deflection toward BR
pub const BOTH_D6__R___: animNumber_t = 582; //# Deflection toward R
pub const BOTH_D6_TR___: animNumber_t = 583; //# Deflection toward TR
pub const BOTH_D6_TL___: animNumber_t = 584; //# Deflection toward TL
pub const BOTH_D6__L___: animNumber_t = 585; //# Deflection toward L
pub const BOTH_D6_BL___: animNumber_t = 586; //# Deflection toward BL
pub const BOTH_D6_B____: animNumber_t = 587; //# Deflection toward B
//Saber attack anims - power level 7
pub const BOTH_A7_T__B_: animNumber_t = 588; //# Fast weak vertical attack top to bottom
pub const BOTH_A7__L__R: animNumber_t = 589; //# Fast weak horizontal attack left to right
pub const BOTH_A7__R__L: animNumber_t = 590; //# Fast weak horizontal attack right to left
pub const BOTH_A7_TL_BR: animNumber_t = 591; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A7_BR_TL: animNumber_t = 592; //# Fast weak diagonal attack top left to botom right
pub const BOTH_A7_BL_TR: animNumber_t = 593; //# Fast weak diagonal attack bottom left to top right
pub const BOTH_A7_TR_BL: animNumber_t = 594; //# Fast weak diagonal attack bottom left to right
//Saber Arc and Spin Transitions
pub const BOTH_T7_BR__R: animNumber_t = 595; //# Fast arc bottom right to right
pub const BOTH_T7_BR_TL: animNumber_t = 596; //# Fast weak spin bottom right to top left
pub const BOTH_T7_BR__L: animNumber_t = 597; //# Fast weak spin bottom right to left
pub const BOTH_T7_BR_BL: animNumber_t = 598; //# Fast weak spin bottom right to bottom left
pub const BOTH_T7__R_TR: animNumber_t = 599; //# Fast arc right to top right
pub const BOTH_T7__R_TL: animNumber_t = 600; //# Fast arc right to top left
pub const BOTH_T7__R__L: animNumber_t = 601; //# Fast weak spin right to left
pub const BOTH_T7__R_BL: animNumber_t = 602; //# Fast weak spin right to bottom left
pub const BOTH_T7_TR_BR: animNumber_t = 603; //# Fast arc top right to bottom right
pub const BOTH_T7_TR_TL: animNumber_t = 604; //# Fast arc top right to top left
pub const BOTH_T7_TR__L: animNumber_t = 605; //# Fast arc top right to left
pub const BOTH_T7_TR_BL: animNumber_t = 606; //# Fast weak spin top right to bottom left
pub const BOTH_T7_T__BR: animNumber_t = 607; //# Fast arc top to bottom right
pub const BOTH_T7_T___R: animNumber_t = 608; //# Fast arc top to right
pub const BOTH_T7_T__TR: animNumber_t = 609; //# Fast arc top to top right
pub const BOTH_T7_T__TL: animNumber_t = 610; //# Fast arc top to top left
pub const BOTH_T7_T___L: animNumber_t = 611; //# Fast arc top to left
pub const BOTH_T7_T__BL: animNumber_t = 612; //# Fast arc top to bottom left
pub const BOTH_T7_TL_BR: animNumber_t = 613; //# Fast weak spin top left to bottom right
pub const BOTH_T7_TL_BL: animNumber_t = 614; //# Fast arc top left to bottom left
pub const BOTH_T7__L_BR: animNumber_t = 615; //# Fast weak spin left to bottom right
pub const BOTH_T7__L__R: animNumber_t = 616; //# Fast weak spin left to right
pub const BOTH_T7__L_TL: animNumber_t = 617; //# Fast arc left to top left
pub const BOTH_T7_BL_BR: animNumber_t = 618; //# Fast weak spin bottom left to bottom right
pub const BOTH_T7_BL__R: animNumber_t = 619; //# Fast weak spin bottom left to right
pub const BOTH_T7_BL_TR: animNumber_t = 620; //# Fast weak spin bottom left to top right
pub const BOTH_T7_BL__L: animNumber_t = 621; //# Fast arc bottom left to left
//Saber Arc Transitions that use existing animations played backwards
pub const BOTH_T7_BR_TR: animNumber_t = 622; //# Fast arc bottom right to top right		(use: BOTH_T7_TR_BR)
pub const BOTH_T7_BR_T_: animNumber_t = 623; //# Fast arc bottom right to top			(use: BOTH_T7_T__BR)
pub const BOTH_T7__R_BR: animNumber_t = 624; //# Fast arc right to bottom right			(use: BOTH_T7_BR__R)
pub const BOTH_T7__R_T_: animNumber_t = 625; //# Fast ar right to top				(use: BOTH_T7_T___R)
pub const BOTH_T7_TR__R: animNumber_t = 626; //# Fast arc top right to right			(use: BOTH_T7__R_TR)
pub const BOTH_T7_TR_T_: animNumber_t = 627; //# Fast arc top right to top				(use: BOTH_T7_T__TR)
pub const BOTH_T7_TL__R: animNumber_t = 628; //# Fast arc top left to right			(use: BOTH_T7__R_TL)
pub const BOTH_T7_TL_TR: animNumber_t = 629; //# Fast arc top left to top right			(use: BOTH_T7_TR_TL)
pub const BOTH_T7_TL_T_: animNumber_t = 630; //# Fast arc top left to top				(use: BOTH_T7_T__TL)
pub const BOTH_T7_TL__L: animNumber_t = 631; //# Fast arc top left to left				(use: BOTH_T7__L_TL)
pub const BOTH_T7__L_TR: animNumber_t = 632; //# Fast arc left to top right			(use: BOTH_T7_TR__L)
pub const BOTH_T7__L_T_: animNumber_t = 633; //# Fast arc left to top				(use: BOTH_T7_T___L)
pub const BOTH_T7__L_BL: animNumber_t = 634; //# Fast arc left to bottom left			(use: BOTH_T7_BL__L)
pub const BOTH_T7_BL_T_: animNumber_t = 635; //# Fast arc bottom left to top			(use: BOTH_T7_T__BL)
pub const BOTH_T7_BL_TL: animNumber_t = 636; //# Fast arc bottom left to top left		(use: BOTH_T7_TL_BL)
//Saber Attack Start Transitions
pub const BOTH_S7_S7_T_: animNumber_t = 637; //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
pub const BOTH_S7_S7__L: animNumber_t = 638; //# Fast plain transition from stance1 to left-to-right Fast weak attack
pub const BOTH_S7_S7__R: animNumber_t = 639; //# Fast plain transition from stance1 to right-to-left Fast weak attack
pub const BOTH_S7_S7_TL: animNumber_t = 640; //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
pub const BOTH_S7_S7_BR: animNumber_t = 641; //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
pub const BOTH_S7_S7_BL: animNumber_t = 642; //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
pub const BOTH_S7_S7_TR: animNumber_t = 643; //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
//Saber Attack Return Transitions
pub const BOTH_R7_B__S7: animNumber_t = 644; //# Fast plain transition from top-to-bottom Fast weak attack to stance1
pub const BOTH_R7__L_S7: animNumber_t = 645; //# Fast plain transition from left-to-right Fast weak attack to stance1
pub const BOTH_R7__R_S7: animNumber_t = 646; //# Fast plain transition from right-to-left Fast weak attack to stance1
pub const BOTH_R7_TL_S7: animNumber_t = 647; //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
pub const BOTH_R7_BR_S7: animNumber_t = 648; //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
pub const BOTH_R7_BL_S7: animNumber_t = 649; //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
pub const BOTH_R7_TR_S7: animNumber_t = 650; //# Fast plain transition from top-right-to-bottom-left Fast weak attack
//Saber Attack Bounces (first 4 frames of an attack, played backwards)
pub const BOTH_B7_BR___: animNumber_t = 651; //# Bounce-back if attack from BR is blocked
pub const BOTH_B7__R___: animNumber_t = 652; //# Bounce-back if attack from R is blocked
pub const BOTH_B7_TR___: animNumber_t = 653; //# Bounce-back if attack from TR is blocked
pub const BOTH_B7_T____: animNumber_t = 654; //# Bounce-back if attack from T is blocked
pub const BOTH_B7_TL___: animNumber_t = 655; //# Bounce-back if attack from TL is blocked
pub const BOTH_B7__L___: animNumber_t = 656; //# Bounce-back if attack from L is blocked
pub const BOTH_B7_BL___: animNumber_t = 657; //# Bounce-back if attack from BL is blocked
//Saber Attack Deflections (last 4 frames of an attack)
pub const BOTH_D7_BR___: animNumber_t = 658; //# Deflection toward BR
pub const BOTH_D7__R___: animNumber_t = 659; //# Deflection toward R
pub const BOTH_D7_TR___: animNumber_t = 660; //# Deflection toward TR
pub const BOTH_D7_TL___: animNumber_t = 661; //# Deflection toward TL
pub const BOTH_D7__L___: animNumber_t = 662; //# Deflection toward L
pub const BOTH_D7_BL___: animNumber_t = 663; //# Deflection toward BL
pub const BOTH_D7_B____: animNumber_t = 664; //# Deflection toward B
//Saber parry anims
pub const BOTH_P1_S1_T_: animNumber_t = 665; //# Block shot/saber top
pub const BOTH_P1_S1_TR: animNumber_t = 666; //# Block shot/saber top right
pub const BOTH_P1_S1_TL: animNumber_t = 667; //# Block shot/saber top left
pub const BOTH_P1_S1_BL: animNumber_t = 668; //# Block shot/saber bottom left
pub const BOTH_P1_S1_BR: animNumber_t = 669; //# Block shot/saber bottom right
//Saber knockaway
pub const BOTH_K1_S1_T_: animNumber_t = 670; //# knockaway saber top
pub const BOTH_K1_S1_TR: animNumber_t = 671; //# knockaway saber top right
pub const BOTH_K1_S1_TL: animNumber_t = 672; //# knockaway saber top left
pub const BOTH_K1_S1_BL: animNumber_t = 673; //# knockaway saber bottom left
pub const BOTH_K1_S1_B_: animNumber_t = 674; //# knockaway saber bottom
pub const BOTH_K1_S1_BR: animNumber_t = 675; //# knockaway saber bottom right
//Saber attack knocked away
pub const BOTH_V1_BR_S1: animNumber_t = 676; //# BR attack knocked away
pub const BOTH_V1__R_S1: animNumber_t = 677; //# R attack knocked away
pub const BOTH_V1_TR_S1: animNumber_t = 678; //# TR attack knocked away
pub const BOTH_V1_T__S1: animNumber_t = 679; //# T attack knocked away
pub const BOTH_V1_TL_S1: animNumber_t = 680; //# TL attack knocked away
pub const BOTH_V1__L_S1: animNumber_t = 681; //# L attack knocked away
pub const BOTH_V1_BL_S1: animNumber_t = 682; //# BL attack knocked away
pub const BOTH_V1_B__S1: animNumber_t = 683; //# B attack knocked away
//Saber parry broken
pub const BOTH_H1_S1_T_: animNumber_t = 684; //# saber knocked down from top parry
pub const BOTH_H1_S1_TR: animNumber_t = 685; //# saber knocked down-left from TR parry
pub const BOTH_H1_S1_TL: animNumber_t = 686; //# saber knocked down-right from TL parry
pub const BOTH_H1_S1_BL: animNumber_t = 687; //# saber knocked up-right from BL parry
pub const BOTH_H1_S1_B_: animNumber_t = 688; //# saber knocked up over head from ready?
pub const BOTH_H1_S1_BR: animNumber_t = 689; //# saber knocked up-left from BR parry
//Dual Saber parry anims
pub const BOTH_P6_S6_T_: animNumber_t = 690; //# Block shot/saber top
pub const BOTH_P6_S6_TR: animNumber_t = 691; //# Block shot/saber top right
pub const BOTH_P6_S6_TL: animNumber_t = 692; //# Block shot/saber top left
pub const BOTH_P6_S6_BL: animNumber_t = 693; //# Block shot/saber bottom left
pub const BOTH_P6_S6_BR: animNumber_t = 694; //# Block shot/saber bottom right
//Dual Saber knockaway
pub const BOTH_K6_S6_T_: animNumber_t = 695; //# knockaway saber top
pub const BOTH_K6_S6_TR: animNumber_t = 696; //# knockaway saber top right
pub const BOTH_K6_S6_TL: animNumber_t = 697; //# knockaway saber top left
pub const BOTH_K6_S6_BL: animNumber_t = 698; //# knockaway saber bottom left
pub const BOTH_K6_S6_B_: animNumber_t = 699; //# knockaway saber bottom
pub const BOTH_K6_S6_BR: animNumber_t = 700; //# knockaway saber bottom right
//Dual Saber attack knocked away
pub const BOTH_V6_BR_S6: animNumber_t = 701; //# BR attack knocked away
pub const BOTH_V6__R_S6: animNumber_t = 702; //# R attack knocked away
pub const BOTH_V6_TR_S6: animNumber_t = 703; //# TR attack knocked away
pub const BOTH_V6_T__S6: animNumber_t = 704; //# T attack knocked away
pub const BOTH_V6_TL_S6: animNumber_t = 705; //# TL attack knocked away
pub const BOTH_V6__L_S6: animNumber_t = 706; //# L attack knocked away
pub const BOTH_V6_BL_S6: animNumber_t = 707; //# BL attack knocked away
pub const BOTH_V6_B__S6: animNumber_t = 708; //# B attack knocked away
//Dual Saber parry broken
pub const BOTH_H6_S6_T_: animNumber_t = 709; //# saber knocked down from top parry
pub const BOTH_H6_S6_TR: animNumber_t = 710; //# saber knocked down-left from TR parry
pub const BOTH_H6_S6_TL: animNumber_t = 711; //# saber knocked down-right from TL parry
pub const BOTH_H6_S6_BL: animNumber_t = 712; //# saber knocked up-right from BL parry
pub const BOTH_H6_S6_B_: animNumber_t = 713; //# saber knocked up over head from ready?
pub const BOTH_H6_S6_BR: animNumber_t = 714; //# saber knocked up-left from BR parry
//SaberStaff parry anims
pub const BOTH_P7_S7_T_: animNumber_t = 715; //# Block shot/saber top
pub const BOTH_P7_S7_TR: animNumber_t = 716; //# Block shot/saber top right
pub const BOTH_P7_S7_TL: animNumber_t = 717; //# Block shot/saber top left
pub const BOTH_P7_S7_BL: animNumber_t = 718; //# Block shot/saber bottom left
pub const BOTH_P7_S7_BR: animNumber_t = 719; //# Block shot/saber bottom right
//SaberStaff knockaway
pub const BOTH_K7_S7_T_: animNumber_t = 720; //# knockaway saber top
pub const BOTH_K7_S7_TR: animNumber_t = 721; //# knockaway saber top right
pub const BOTH_K7_S7_TL: animNumber_t = 722; //# knockaway saber top left
pub const BOTH_K7_S7_BL: animNumber_t = 723; //# knockaway saber bottom left
pub const BOTH_K7_S7_B_: animNumber_t = 724; //# knockaway saber bottom
pub const BOTH_K7_S7_BR: animNumber_t = 725; //# knockaway saber bottom right
//SaberStaff attack knocked away
pub const BOTH_V7_BR_S7: animNumber_t = 726; //# BR attack knocked away
pub const BOTH_V7__R_S7: animNumber_t = 727; //# R attack knocked away
pub const BOTH_V7_TR_S7: animNumber_t = 728; //# TR attack knocked away
pub const BOTH_V7_T__S7: animNumber_t = 729; //# T attack knocked away
pub const BOTH_V7_TL_S7: animNumber_t = 730; //# TL attack knocked away
pub const BOTH_V7__L_S7: animNumber_t = 731; //# L attack knocked away
pub const BOTH_V7_BL_S7: animNumber_t = 732; //# BL attack knocked away
pub const BOTH_V7_B__S7: animNumber_t = 733; //# B attack knocked away
//SaberStaff parry broken
pub const BOTH_H7_S7_T_: animNumber_t = 734; //# saber knocked down from top parry
pub const BOTH_H7_S7_TR: animNumber_t = 735; //# saber knocked down-left from TR parry
pub const BOTH_H7_S7_TL: animNumber_t = 736; //# saber knocked down-right from TL parry
pub const BOTH_H7_S7_BL: animNumber_t = 737; //# saber knocked up-right from BL parry
pub const BOTH_H7_S7_B_: animNumber_t = 738; //# saber knocked up over head from ready?
pub const BOTH_H7_S7_BR: animNumber_t = 739; //# saber knocked up-left from BR parry
//Sabers locked anims
//* #sep BOTH_ SABER LOCKED ANIMS
//BOTH_(DL, S, ST)_(DL, S, ST)_(T, S)_(L, B, SB)_1(_W, _L)
//===Single locks==================================================================
//SINGLE vs. DUAL
//side locks - I'm using a single and they're using dual
pub const BOTH_LK_S_DL_S_B_1_L: animNumber_t = 740; //normal break I lost
pub const BOTH_LK_S_DL_S_B_1_W: animNumber_t = 741; //normal break I won
pub const BOTH_LK_S_DL_S_L_1: animNumber_t = 742; //lock if I'm using single vs. a dual
pub const BOTH_LK_S_DL_S_SB_1_L: animNumber_t = 743; //super break I lost
pub const BOTH_LK_S_DL_S_SB_1_W: animNumber_t = 744; //super break I won
//top locks
pub const BOTH_LK_S_DL_T_B_1_L: animNumber_t = 745; //normal break I lost
pub const BOTH_LK_S_DL_T_B_1_W: animNumber_t = 746; //normal break I won
pub const BOTH_LK_S_DL_T_L_1: animNumber_t = 747; //lock if I'm using single vs. a dual
pub const BOTH_LK_S_DL_T_SB_1_L: animNumber_t = 748; //super break I lost
pub const BOTH_LK_S_DL_T_SB_1_W: animNumber_t = 749; //super break I won
//SINGLE vs. STAFF
//side locks
pub const BOTH_LK_S_ST_S_B_1_L: animNumber_t = 750; //normal break I lost
pub const BOTH_LK_S_ST_S_B_1_W: animNumber_t = 751; //normal break I won
pub const BOTH_LK_S_ST_S_L_1: animNumber_t = 752; //lock if I'm using single vs. a staff
pub const BOTH_LK_S_ST_S_SB_1_L: animNumber_t = 753; //super break I lost
pub const BOTH_LK_S_ST_S_SB_1_W: animNumber_t = 754; //super break I won
//top locks
pub const BOTH_LK_S_ST_T_B_1_L: animNumber_t = 755; //normal break I lost
pub const BOTH_LK_S_ST_T_B_1_W: animNumber_t = 756; //normal break I won
pub const BOTH_LK_S_ST_T_L_1: animNumber_t = 757; //lock if I'm using single vs. a staff
pub const BOTH_LK_S_ST_T_SB_1_L: animNumber_t = 758; //super break I lost
pub const BOTH_LK_S_ST_T_SB_1_W: animNumber_t = 759; //super break I won
//SINGLE vs. SINGLE
//side locks
pub const BOTH_LK_S_S_S_B_1_L: animNumber_t = 760; //normal break I lost
pub const BOTH_LK_S_S_S_B_1_W: animNumber_t = 761; //normal break I won
pub const BOTH_LK_S_S_S_L_1: animNumber_t = 762; //lock if I'm using single vs. a single and I initiated
pub const BOTH_LK_S_S_S_SB_1_L: animNumber_t = 763; //super break I lost
pub const BOTH_LK_S_S_S_SB_1_W: animNumber_t = 764; //super break I won
//top locks
pub const BOTH_LK_S_S_T_B_1_L: animNumber_t = 765; //normal break I lost
pub const BOTH_LK_S_S_T_B_1_W: animNumber_t = 766; //normal break I won
pub const BOTH_LK_S_S_T_L_1: animNumber_t = 767; //lock if I'm using single vs. a single and I initiated
pub const BOTH_LK_S_S_T_SB_1_L: animNumber_t = 768; //super break I lost
pub const BOTH_LK_S_S_T_SB_1_W: animNumber_t = 769; //super break I won
//===Dual Saber locks==================================================================
//DUAL vs. DUAL
//side locks
pub const BOTH_LK_DL_DL_S_B_1_L: animNumber_t = 770; //normal break I lost
pub const BOTH_LK_DL_DL_S_B_1_W: animNumber_t = 771; //normal break I won
pub const BOTH_LK_DL_DL_S_L_1: animNumber_t = 772; //lock if I'm using dual vs. dual and I initiated
pub const BOTH_LK_DL_DL_S_SB_1_L: animNumber_t = 773; //super break I lost
pub const BOTH_LK_DL_DL_S_SB_1_W: animNumber_t = 774; //super break I won
//top locks
pub const BOTH_LK_DL_DL_T_B_1_L: animNumber_t = 775; //normal break I lost
pub const BOTH_LK_DL_DL_T_B_1_W: animNumber_t = 776; //normal break I won
pub const BOTH_LK_DL_DL_T_L_1: animNumber_t = 777; //lock if I'm using dual vs. dual and I initiated
pub const BOTH_LK_DL_DL_T_SB_1_L: animNumber_t = 778; //super break I lost
pub const BOTH_LK_DL_DL_T_SB_1_W: animNumber_t = 779; //super break I won
//DUAL vs. STAFF
//side locks
pub const BOTH_LK_DL_ST_S_B_1_L: animNumber_t = 780; //normal break I lost
pub const BOTH_LK_DL_ST_S_B_1_W: animNumber_t = 781; //normal break I won
pub const BOTH_LK_DL_ST_S_L_1: animNumber_t = 782; //lock if I'm using dual vs. a staff
pub const BOTH_LK_DL_ST_S_SB_1_L: animNumber_t = 783; //super break I lost
pub const BOTH_LK_DL_ST_S_SB_1_W: animNumber_t = 784; //super break I won
//top locks
pub const BOTH_LK_DL_ST_T_B_1_L: animNumber_t = 785; //normal break I lost
pub const BOTH_LK_DL_ST_T_B_1_W: animNumber_t = 786; //normal break I won
pub const BOTH_LK_DL_ST_T_L_1: animNumber_t = 787; //lock if I'm using dual vs. a staff
pub const BOTH_LK_DL_ST_T_SB_1_L: animNumber_t = 788; //super break I lost
pub const BOTH_LK_DL_ST_T_SB_1_W: animNumber_t = 789; //super break I won
//DUAL vs. SINGLE
//side locks
pub const BOTH_LK_DL_S_S_B_1_L: animNumber_t = 790; //normal break I lost
pub const BOTH_LK_DL_S_S_B_1_W: animNumber_t = 791; //normal break I won
pub const BOTH_LK_DL_S_S_L_1: animNumber_t = 792; //lock if I'm using dual vs. a single
pub const BOTH_LK_DL_S_S_SB_1_L: animNumber_t = 793; //super break I lost
pub const BOTH_LK_DL_S_S_SB_1_W: animNumber_t = 794; //super break I won
//top locks
pub const BOTH_LK_DL_S_T_B_1_L: animNumber_t = 795; //normal break I lost
pub const BOTH_LK_DL_S_T_B_1_W: animNumber_t = 796; //normal break I won
pub const BOTH_LK_DL_S_T_L_1: animNumber_t = 797; //lock if I'm using dual vs. a single
pub const BOTH_LK_DL_S_T_SB_1_L: animNumber_t = 798; //super break I lost
pub const BOTH_LK_DL_S_T_SB_1_W: animNumber_t = 799; //super break I won
//===Saber Staff locks==================================================================
//STAFF vs. DUAL
//side locks
pub const BOTH_LK_ST_DL_S_B_1_L: animNumber_t = 800; //normal break I lost
pub const BOTH_LK_ST_DL_S_B_1_W: animNumber_t = 801; //normal break I won
pub const BOTH_LK_ST_DL_S_L_1: animNumber_t = 802; //lock if I'm using staff vs. dual
pub const BOTH_LK_ST_DL_S_SB_1_L: animNumber_t = 803; //super break I lost
pub const BOTH_LK_ST_DL_S_SB_1_W: animNumber_t = 804; //super break I won
//top locks
pub const BOTH_LK_ST_DL_T_B_1_L: animNumber_t = 805; //normal break I lost
pub const BOTH_LK_ST_DL_T_B_1_W: animNumber_t = 806; //normal break I won
pub const BOTH_LK_ST_DL_T_L_1: animNumber_t = 807; //lock if I'm using staff vs. dual
pub const BOTH_LK_ST_DL_T_SB_1_L: animNumber_t = 808; //super break I lost
pub const BOTH_LK_ST_DL_T_SB_1_W: animNumber_t = 809; //super break I won
//STAFF vs. STAFF
//side locks
pub const BOTH_LK_ST_ST_S_B_1_L: animNumber_t = 810; //normal break I lost
pub const BOTH_LK_ST_ST_S_B_1_W: animNumber_t = 811; //normal break I won
pub const BOTH_LK_ST_ST_S_L_1: animNumber_t = 812; //lock if I'm using staff vs. a staff and I initiated
pub const BOTH_LK_ST_ST_S_SB_1_L: animNumber_t = 813; //super break I lost
pub const BOTH_LK_ST_ST_S_SB_1_W: animNumber_t = 814; //super break I won
//top locks
pub const BOTH_LK_ST_ST_T_B_1_L: animNumber_t = 815; //normal break I lost
pub const BOTH_LK_ST_ST_T_B_1_W: animNumber_t = 816; //normal break I won
pub const BOTH_LK_ST_ST_T_L_1: animNumber_t = 817; //lock if I'm using staff vs. a staff and I initiated
pub const BOTH_LK_ST_ST_T_SB_1_L: animNumber_t = 818; //super break I lost
pub const BOTH_LK_ST_ST_T_SB_1_W: animNumber_t = 819; //super break I won
//STAFF vs. SINGLE
//side locks
pub const BOTH_LK_ST_S_S_B_1_L: animNumber_t = 820; //normal break I lost
pub const BOTH_LK_ST_S_S_B_1_W: animNumber_t = 821; //normal break I won
pub const BOTH_LK_ST_S_S_L_1: animNumber_t = 822; //lock if I'm using staff vs. a single
pub const BOTH_LK_ST_S_S_SB_1_L: animNumber_t = 823; //super break I lost
pub const BOTH_LK_ST_S_S_SB_1_W: animNumber_t = 824; //super break I won
//top locks
pub const BOTH_LK_ST_S_T_B_1_L: animNumber_t = 825; //normal break I lost
pub const BOTH_LK_ST_S_T_B_1_W: animNumber_t = 826; //normal break I won
pub const BOTH_LK_ST_S_T_L_1: animNumber_t = 827; //lock if I'm using staff vs. a single
pub const BOTH_LK_ST_S_T_SB_1_L: animNumber_t = 828; //super break I lost
pub const BOTH_LK_ST_S_T_SB_1_W: animNumber_t = 829; //super break I won
//Special cases for same saber style vs. each other (won't fit in nice 5-anim size lists above)
pub const BOTH_LK_S_S_S_L_2: animNumber_t = 830; //lock if I'm using single vs. a single and other intitiated
pub const BOTH_LK_S_S_T_L_2: animNumber_t = 831; //lock if I'm using single vs. a single and other initiated
pub const BOTH_LK_DL_DL_S_L_2: animNumber_t = 832; //lock if I'm using dual vs. dual and other initiated
pub const BOTH_LK_DL_DL_T_L_2: animNumber_t = 833; //lock if I'm using dual vs. dual and other initiated
pub const BOTH_LK_ST_ST_S_L_2: animNumber_t = 834; //lock if I'm using staff vs. a staff and other initiated
pub const BOTH_LK_ST_ST_T_L_2: animNumber_t = 835; //lock if I'm using staff vs. a staff and other initiated
//===End Saber locks==================================================================
//old locks
pub const BOTH_BF2RETURN: animNumber_t = 836; //#
pub const BOTH_BF2BREAK: animNumber_t = 837; //#
pub const BOTH_BF2LOCK: animNumber_t = 838; //#
pub const BOTH_BF1RETURN: animNumber_t = 839; //#
pub const BOTH_BF1BREAK: animNumber_t = 840; //#
pub const BOTH_BF1LOCK: animNumber_t = 841; //#
pub const BOTH_CWCIRCLE_R2__R_S1: animNumber_t = 842; //#
pub const BOTH_CCWCIRCLE_R2__L_S1: animNumber_t = 843; //#
pub const BOTH_CWCIRCLE_A2__L__R: animNumber_t = 844; //#
pub const BOTH_CCWCIRCLE_A2__R__L: animNumber_t = 845; //#
pub const BOTH_CWCIRCLEBREAK: animNumber_t = 846; //#
pub const BOTH_CCWCIRCLEBREAK: animNumber_t = 847; //#
pub const BOTH_CWCIRCLELOCK: animNumber_t = 848; //#
pub const BOTH_CCWCIRCLELOCK: animNumber_t = 849; //#
//other saber anims
//* #sep BOTH_ SABER MISC ANIMS
pub const BOTH_SABERFAST_STANCE: animNumber_t = 850;
pub const BOTH_SABERSLOW_STANCE: animNumber_t = 851;
pub const BOTH_SABERDUAL_STANCE: animNumber_t = 852;
pub const BOTH_SABERSTAFF_STANCE: animNumber_t = 853;
pub const BOTH_A2_STABBACK1: animNumber_t = 854; //# Stab saber backward
pub const BOTH_ATTACK_BACK: animNumber_t = 855; //# Swing around backwards and attack
pub const BOTH_JUMPFLIPSLASHDOWN1: animNumber_t = 856; //#
pub const BOTH_JUMPFLIPSTABDOWN: animNumber_t = 857; //#
pub const BOTH_FORCELEAP2_T__B_: animNumber_t = 858; //#
pub const BOTH_LUNGE2_B__T_: animNumber_t = 859; //#
pub const BOTH_CROUCHATTACKBACK1: animNumber_t = 860; //#
//New specials for JKA:
pub const BOTH_JUMPATTACK6: animNumber_t = 861; //#
pub const BOTH_JUMPATTACK7: animNumber_t = 862; //#
pub const BOTH_SPINATTACK6: animNumber_t = 863; //#
pub const BOTH_SPINATTACK7: animNumber_t = 864; //#
pub const BOTH_S1_S6: animNumber_t = 865; //#	From stand1 to saberdual stance - turning on your dual sabers
pub const BOTH_S6_S1: animNumber_t = 866; //#	From dualstaff stance to stand1 - turning off your dual sabers
pub const BOTH_S1_S7: animNumber_t = 867; //#	From stand1 to saberstaff stance - turning on your saberstaff
pub const BOTH_S7_S1: animNumber_t = 868; //#	From saberstaff stance to stand1 - turning off your saberstaff
pub const BOTH_FORCELONGLEAP_START: animNumber_t = 869;
pub const BOTH_FORCELONGLEAP_ATTACK: animNumber_t = 870;
pub const BOTH_FORCELONGLEAP_LAND: animNumber_t = 871;
pub const BOTH_FORCEWALLRUNFLIP_START: animNumber_t = 872;
pub const BOTH_FORCEWALLRUNFLIP_END: animNumber_t = 873;
pub const BOTH_FORCEWALLRUNFLIP_ALT: animNumber_t = 874;
pub const BOTH_FORCEWALLREBOUND_FORWARD: animNumber_t = 875;
pub const BOTH_FORCEWALLREBOUND_LEFT: animNumber_t = 876;
pub const BOTH_FORCEWALLREBOUND_BACK: animNumber_t = 877;
pub const BOTH_FORCEWALLREBOUND_RIGHT: animNumber_t = 878;
pub const BOTH_FORCEWALLHOLD_FORWARD: animNumber_t = 879;
pub const BOTH_FORCEWALLHOLD_LEFT: animNumber_t = 880;
pub const BOTH_FORCEWALLHOLD_BACK: animNumber_t = 881;
pub const BOTH_FORCEWALLHOLD_RIGHT: animNumber_t = 882;
pub const BOTH_FORCEWALLRELEASE_FORWARD: animNumber_t = 883;
pub const BOTH_FORCEWALLRELEASE_LEFT: animNumber_t = 884;
pub const BOTH_FORCEWALLRELEASE_BACK: animNumber_t = 885;
pub const BOTH_FORCEWALLRELEASE_RIGHT: animNumber_t = 886;
pub const BOTH_A7_KICK_F: animNumber_t = 887;
pub const BOTH_A7_KICK_B: animNumber_t = 888;
pub const BOTH_A7_KICK_R: animNumber_t = 889;
pub const BOTH_A7_KICK_L: animNumber_t = 890;
pub const BOTH_A7_KICK_S: animNumber_t = 891;
pub const BOTH_A7_KICK_BF: animNumber_t = 892;
pub const BOTH_A7_KICK_BF_STOP: animNumber_t = 893;
pub const BOTH_A7_KICK_RL: animNumber_t = 894;
pub const BOTH_A7_KICK_F_AIR: animNumber_t = 895;
pub const BOTH_A7_KICK_B_AIR: animNumber_t = 896;
pub const BOTH_A7_KICK_R_AIR: animNumber_t = 897;
pub const BOTH_A7_KICK_L_AIR: animNumber_t = 898;
pub const BOTH_FLIP_ATTACK7: animNumber_t = 899;
pub const BOTH_FLIP_HOLD7: animNumber_t = 900;
pub const BOTH_FLIP_LAND: animNumber_t = 901;
pub const BOTH_PULL_IMPALE_STAB: animNumber_t = 902;
pub const BOTH_PULL_IMPALE_SWING: animNumber_t = 903;
pub const BOTH_PULLED_INAIR_B: animNumber_t = 904;
pub const BOTH_PULLED_INAIR_F: animNumber_t = 905;
pub const BOTH_STABDOWN: animNumber_t = 906;
pub const BOTH_STABDOWN_STAFF: animNumber_t = 907;
pub const BOTH_STABDOWN_DUAL: animNumber_t = 908;
pub const BOTH_A6_SABERPROTECT: animNumber_t = 909;
pub const BOTH_A7_SOULCAL: animNumber_t = 910;
pub const BOTH_A1_SPECIAL: animNumber_t = 911;
pub const BOTH_A2_SPECIAL: animNumber_t = 912;
pub const BOTH_A3_SPECIAL: animNumber_t = 913;
pub const BOTH_ROLL_STAB: animNumber_t = 914;

//# #sep BOTH_ STANDING
pub const BOTH_STAND1: animNumber_t = 915; //# Standing idle, no weapon, hands down
pub const BOTH_STAND1IDLE1: animNumber_t = 916; //# Random standing idle
pub const BOTH_STAND2: animNumber_t = 917; //# Standing idle with a saber
pub const BOTH_STAND2IDLE1: animNumber_t = 918; //# Random standing idle
pub const BOTH_STAND2IDLE2: animNumber_t = 919; //# Random standing idle
pub const BOTH_STAND3: animNumber_t = 920; //# Standing idle with 2-handed weapon
pub const BOTH_STAND3IDLE1: animNumber_t = 921; //# Random standing idle
pub const BOTH_STAND4: animNumber_t = 922; //# hands clasp behind back
pub const BOTH_STAND5: animNumber_t = 923; //# standing idle, no weapon, hand down, back straight
pub const BOTH_STAND5IDLE1: animNumber_t = 924; //# Random standing idle
pub const BOTH_STAND6: animNumber_t = 925; //# one handed, gun at side, relaxed stand
pub const BOTH_STAND8: animNumber_t = 926; //# both hands on hips (male)
pub const BOTH_STAND1TO2: animNumber_t = 927; //# Transition from stand1 to stand2
pub const BOTH_STAND2TO1: animNumber_t = 928; //# Transition from stand2 to stand1
pub const BOTH_STAND2TO4: animNumber_t = 929; //# Transition from stand2 to stand4
pub const BOTH_STAND4TO2: animNumber_t = 930; //# Transition from stand4 to stand2
pub const BOTH_STAND4TOATTACK2: animNumber_t = 931; //# relaxed stand to 1-handed pistol ready
pub const BOTH_STANDUP2: animNumber_t = 932; //# Luke standing up from his meditation platform (cin # 37)
pub const BOTH_STAND5TOSIT3: animNumber_t = 933; //# transition from stand 5 to sit 3
pub const BOTH_STAND1TOSTAND5: animNumber_t = 934; //# Transition from stand1 to stand5
pub const BOTH_STAND5TOSTAND1: animNumber_t = 935; //# Transition from stand5 to stand1
pub const BOTH_STAND5TOAIM: animNumber_t = 936; //# Transition of Kye aiming his gun at Desann (cin #9)
pub const BOTH_STAND5STARTLEDLOOKLEFT: animNumber_t = 937; //# Kyle turning to watch the bridge drop (cin #9)
pub const BOTH_STARTLEDLOOKLEFTTOSTAND5: animNumber_t = 938; //# Kyle returning to stand 5 from watching the bridge drop (cin #9)
pub const BOTH_STAND5TOSTAND8: animNumber_t = 939; //# Transition from stand5 to stand8
pub const BOTH_STAND7TOSTAND8: animNumber_t = 940; //# Tavion putting hands on back of chair (cin #11)
pub const BOTH_STAND8TOSTAND5: animNumber_t = 941; //# Transition from stand8 to stand5
pub const BOTH_STAND9: animNumber_t = 942; //# Kyle's standing idle, no weapon, hands down
pub const BOTH_STAND9IDLE1: animNumber_t = 943; //# Kyle's random standing idle
pub const BOTH_STAND5SHIFTWEIGHT: animNumber_t = 944; //# Weightshift from stand5 to side and back to stand5
pub const BOTH_STAND5SHIFTWEIGHTSTART: animNumber_t = 945; //# From stand5 to side
pub const BOTH_STAND5SHIFTWEIGHTSTOP: animNumber_t = 946; //# From side to stand5
pub const BOTH_STAND5TURNLEFTSTART: animNumber_t = 947; //# Start turning left from stand5
pub const BOTH_STAND5TURNLEFTSTOP: animNumber_t = 948; //# Stop turning left from stand5
pub const BOTH_STAND5TURNRIGHTSTART: animNumber_t = 949; //# Start turning right from stand5
pub const BOTH_STAND5TURNRIGHTSTOP: animNumber_t = 950; //# Stop turning right from stand5
pub const BOTH_STAND5LOOK180LEFTSTART: animNumber_t = 951; //# Start looking over left shoulder (cin #17)
pub const BOTH_STAND5LOOK180LEFTSTOP: animNumber_t = 952; //# Stop looking over left shoulder (cin #17)

pub const BOTH_CONSOLE1START: animNumber_t = 953; //# typing at a console
pub const BOTH_CONSOLE1: animNumber_t = 954; //# typing at a console
pub const BOTH_CONSOLE1STOP: animNumber_t = 955; //# typing at a console
pub const BOTH_CONSOLE2START: animNumber_t = 956; //# typing at a console with comm link in hand (cin #5)
pub const BOTH_CONSOLE2: animNumber_t = 957; //# typing at a console with comm link in hand (cin #5)
pub const BOTH_CONSOLE2STOP: animNumber_t = 958; //# typing at a console with comm link in hand (cin #5)
pub const BOTH_CONSOLE2HOLDCOMSTART: animNumber_t = 959; //# lean in to type at console while holding comm link in hand (cin #5)
pub const BOTH_CONSOLE2HOLDCOMSTOP: animNumber_t = 960; //# lean away after typing at console while holding comm link in hand (cin #5)

pub const BOTH_GUARD_LOOKAROUND1: animNumber_t = 961; //# Cradling weapon and looking around
pub const BOTH_GUARD_IDLE1: animNumber_t = 962; //# Cradling weapon and standing
pub const BOTH_GESTURE1: animNumber_t = 963; //# Generic gesture, non-specific
pub const BOTH_GESTURE2: animNumber_t = 964; //# Generic gesture, non-specific
pub const BOTH_WALK1TALKCOMM1: animNumber_t = 965; //# Talking into coom link while walking
pub const BOTH_TALK1: animNumber_t = 966; //# Generic talk anim
pub const BOTH_TALK2: animNumber_t = 967; //# Generic talk anim
pub const BOTH_TALKCOMM1START: animNumber_t = 968; //# Start talking into a comm link
pub const BOTH_TALKCOMM1: animNumber_t = 969; //# Talking into a comm link
pub const BOTH_TALKCOMM1STOP: animNumber_t = 970; //# Stop talking into a comm link
pub const BOTH_TALKGESTURE1: animNumber_t = 971; //# Generic talk anim

pub const BOTH_HEADTILTLSTART: animNumber_t = 972; //# Head tilt to left
pub const BOTH_HEADTILTLSTOP: animNumber_t = 973; //# Head tilt to left
pub const BOTH_HEADTILTRSTART: animNumber_t = 974; //# Head tilt to right
pub const BOTH_HEADTILTRSTOP: animNumber_t = 975; //# Head tilt to right
pub const BOTH_HEADNOD: animNumber_t = 976; //# Head shake YES
pub const BOTH_HEADSHAKE: animNumber_t = 977; //# Head shake NO
pub const BOTH_SIT2HEADTILTLSTART: animNumber_t = 978; //# Head tilt to left from seated position 2
pub const BOTH_SIT2HEADTILTLSTOP: animNumber_t = 979; //# Head tilt to left from seated position 2

pub const BOTH_REACH1START: animNumber_t = 980; //# Monmothma reaching for crystal
pub const BOTH_REACH1STOP: animNumber_t = 981; //# Monmothma reaching for crystal

pub const BOTH_COME_ON1: animNumber_t = 982; //# Jan gesturing to Kyle (cin #32a)
pub const BOTH_STEADYSELF1: animNumber_t = 983; //# Jan trying to keep footing (cin #32a)
pub const BOTH_STEADYSELF1END: animNumber_t = 984; //# Return hands to side from STEADSELF1 Kyle (cin#5)
pub const BOTH_SILENCEGESTURE1: animNumber_t = 985; //# Luke silencing Kyle with a raised hand (cin #37)
pub const BOTH_REACHFORSABER1: animNumber_t = 986; //# Luke holding hand out for Kyle's saber (cin #37)
pub const BOTH_SABERKILLER1: animNumber_t = 987; //# Tavion about to strike Jan with saber (cin #9)
pub const BOTH_SABERKILLEE1: animNumber_t = 988; //# Jan about to be struck by Tavion with saber (cin #9)
pub const BOTH_HUGGER1: animNumber_t = 989; //# Kyle hugging Jan (cin #29)
pub const BOTH_HUGGERSTOP1: animNumber_t = 990; //# Kyle stop hugging Jan but don't let her go (cin #29)
pub const BOTH_HUGGEE1: animNumber_t = 991; //# Jan being hugged (cin #29)
pub const BOTH_HUGGEESTOP1: animNumber_t = 992; //# Jan stop being hugged but don't let go (cin #29)

pub const BOTH_SABERTHROW1START: animNumber_t = 993; //# Desann throwing his light saber (cin #26)
pub const BOTH_SABERTHROW1STOP: animNumber_t = 994; //# Desann throwing his light saber (cin #26)
pub const BOTH_SABERTHROW2START: animNumber_t = 995; //# Kyle throwing his light saber (cin #32)
pub const BOTH_SABERTHROW2STOP: animNumber_t = 996; //# Kyle throwing his light saber (cin #32)

//# #sep BOTH_ SITTING/CROUCHING
pub const BOTH_SIT1: animNumber_t = 997; //# Normal chair sit.
pub const BOTH_SIT2: animNumber_t = 998; //# Lotus position.
pub const BOTH_SIT3: animNumber_t = 999; //# Sitting in tired position, elbows on knees

pub const BOTH_SIT2TOSTAND5: animNumber_t = 1000; //# Transition from sit 2 to stand 5
pub const BOTH_STAND5TOSIT2: animNumber_t = 1001; //# Transition from stand 5 to sit 2
pub const BOTH_SIT2TOSIT4: animNumber_t = 1002; //# Trans from sit2 to sit4 (cin #12) Luke leaning back from lotus position.
pub const BOTH_SIT3TOSTAND5: animNumber_t = 1003; //# transition from sit 3 to stand 5

pub const BOTH_CROUCH1: animNumber_t = 1004; //# Transition from standing to crouch
pub const BOTH_CROUCH1IDLE: animNumber_t = 1005; //# Crouching idle
pub const BOTH_CROUCH1WALK: animNumber_t = 1006; //# Walking while crouched
pub const BOTH_CROUCH1WALKBACK: animNumber_t = 1007; //# Walking while crouched
pub const BOTH_UNCROUCH1: animNumber_t = 1008; //# Transition from crouch to standing
pub const BOTH_CROUCH2TOSTAND1: animNumber_t = 1009; //# going from crouch2 to stand1
pub const BOTH_CROUCH3: animNumber_t = 1010; //# Desann crouching down to Kyle (cin 9)
pub const BOTH_UNCROUCH3: animNumber_t = 1011; //# Desann uncrouching down to Kyle (cin 9)
pub const BOTH_CROUCH4: animNumber_t = 1012; //# Slower version of crouch1 for cinematics
pub const BOTH_UNCROUCH4: animNumber_t = 1013; //# Slower version of uncrouch1 for cinematics

pub const BOTH_GUNSIT1: animNumber_t = 1014; //# sitting on an emplaced gun.

// Swoop Vehicle animations.
//* #sep BOTH_ SWOOP ANIMS
pub const BOTH_VS_MOUNT_L: animNumber_t = 1015; //# Mount from left
pub const BOTH_VS_DISMOUNT_L: animNumber_t = 1016; //# Dismount to left
pub const BOTH_VS_MOUNT_R: animNumber_t = 1017; //# Mount from  right (symmetry)
pub const BOTH_VS_DISMOUNT_R: animNumber_t = 1018; //# DISMOUNT TO  RIGHT (SYMMETRY)

pub const BOTH_VS_MOUNTJUMP_L: animNumber_t = 1019; //#
pub const BOTH_VS_MOUNTTHROW: animNumber_t = 1020; //# Land on an occupied vehicle & throw off current pilot
pub const BOTH_VS_MOUNTTHROW_L: animNumber_t = 1021; //# Land on an occupied vehicle & throw off current pilot
pub const BOTH_VS_MOUNTTHROW_R: animNumber_t = 1022; //# Land on an occupied vehicle & throw off current pilot
pub const BOTH_VS_MOUNTTHROWEE: animNumber_t = 1023; //# Current pilot getting thrown off by another guy

pub const BOTH_VS_LOOKLEFT: animNumber_t = 1024; //# Turn & Look behind and to the left (no weapon)
pub const BOTH_VS_LOOKRIGHT: animNumber_t = 1025; //# Turn & Look behind and to the right (no weapon)

pub const BOTH_VS_TURBO: animNumber_t = 1026; //# Hit The Turbo Button

pub const BOTH_VS_REV: animNumber_t = 1027; //# Player looks back as swoop reverses

pub const BOTH_VS_AIR: animNumber_t = 1028; //# Player stands up when swoop is airborn
pub const BOTH_VS_AIR_G: animNumber_t = 1029; //# "" with Gun
pub const BOTH_VS_AIR_SL: animNumber_t = 1030; //# "" with Saber Left
pub const BOTH_VS_AIR_SR: animNumber_t = 1031; //# "" with Saber Right

pub const BOTH_VS_LAND: animNumber_t = 1032; //# Player bounces down when swoop lands
pub const BOTH_VS_LAND_G: animNumber_t = 1033; //#  "" with Gun
pub const BOTH_VS_LAND_SL: animNumber_t = 1034; //#  "" with Saber Left
pub const BOTH_VS_LAND_SR: animNumber_t = 1035; //#  "" with Saber Right

pub const BOTH_VS_IDLE: animNumber_t = 1036; //# Sit
pub const BOTH_VS_IDLE_G: animNumber_t = 1037; //# Sit (gun)
pub const BOTH_VS_IDLE_SL: animNumber_t = 1038; //# Sit (saber left)
pub const BOTH_VS_IDLE_SR: animNumber_t = 1039; //# Sit (saber right)

pub const BOTH_VS_LEANL: animNumber_t = 1040; //# Lean left
pub const BOTH_VS_LEANL_G: animNumber_t = 1041; //# Lean left (gun)
pub const BOTH_VS_LEANL_SL: animNumber_t = 1042; //# Lean left (saber left)
pub const BOTH_VS_LEANL_SR: animNumber_t = 1043; //# Lean left (saber right)

pub const BOTH_VS_LEANR: animNumber_t = 1044; //# Lean right
pub const BOTH_VS_LEANR_G: animNumber_t = 1045; //# Lean right (gun)
pub const BOTH_VS_LEANR_SL: animNumber_t = 1046; //# Lean right (saber left)
pub const BOTH_VS_LEANR_SR: animNumber_t = 1047; //# Lean right (saber right)

pub const BOTH_VS_ATL_S: animNumber_t = 1048; //# Attack left with saber
pub const BOTH_VS_ATR_S: animNumber_t = 1049; //# Attack right with saber
pub const BOTH_VS_ATR_TO_L_S: animNumber_t = 1050; //# Attack toss saber from right to left hand
pub const BOTH_VS_ATL_TO_R_S: animNumber_t = 1051; //# Attack toss saber from left to right hand
pub const BOTH_VS_ATR_G: animNumber_t = 1052; //# Attack right with gun (90)
pub const BOTH_VS_ATL_G: animNumber_t = 1053; //# Attack left with gun (90)
pub const BOTH_VS_ATF_G: animNumber_t = 1054; //# Attack forward with gun

pub const BOTH_VS_PAIN1: animNumber_t = 1055; //# Pain

// Added 12/04/02 by Aurelio.
//* #sep BOTH_ TAUNTAUN ANIMS
pub const BOTH_VT_MOUNT_L: animNumber_t = 1056; //# Mount from left
pub const BOTH_VT_MOUNT_R: animNumber_t = 1057; //# Mount from right
pub const BOTH_VT_MOUNT_B: animNumber_t = 1058; //# Mount from air, behind
pub const BOTH_VT_DISMOUNT: animNumber_t = 1059; //# Dismount for tauntaun
pub const BOTH_VT_DISMOUNT_L: animNumber_t = 1060; //# Dismount to tauntauns left
pub const BOTH_VT_DISMOUNT_R: animNumber_t = 1061; //# Dismount to tauntauns right (symmetry)

pub const BOTH_VT_WALK_FWD: animNumber_t = 1062; //# Walk forward
pub const BOTH_VT_WALK_REV: animNumber_t = 1063; //# Walk backward
pub const BOTH_VT_WALK_FWD_L: animNumber_t = 1064; //# walk lean left
pub const BOTH_VT_WALK_FWD_R: animNumber_t = 1065; //# Walk lean right
pub const BOTH_VT_RUN_FWD: animNumber_t = 1066; //# Run forward
pub const BOTH_VT_RUN_REV: animNumber_t = 1067; //# Look backwards while running (not weapon specific)
pub const BOTH_VT_RUN_FWD_L: animNumber_t = 1068; //# Run lean left
pub const BOTH_VT_RUN_FWD_R: animNumber_t = 1069; //# Run lean right

pub const BOTH_VT_SLIDEF: animNumber_t = 1070; //# Tauntaun slides forward with abrupt stop
pub const BOTH_VT_AIR: animNumber_t = 1071; //# Tauntaun jump
pub const BOTH_VT_ATB: animNumber_t = 1072; //# Tauntaun tail swipe
pub const BOTH_VT_PAIN1: animNumber_t = 1073; //# Pain
pub const BOTH_VT_DEATH1: animNumber_t = 1074; //# Die
pub const BOTH_VT_STAND: animNumber_t = 1075; //# Stand still and breath
pub const BOTH_VT_BUCK: animNumber_t = 1076; //# Tauntaun bucking loop animation

pub const BOTH_VT_LAND: animNumber_t = 1077; //# Player bounces down when tauntaun lands
pub const BOTH_VT_TURBO: animNumber_t = 1078; //# Hit The Turbo Button
pub const BOTH_VT_IDLE_SL: animNumber_t = 1079; //# Sit (saber left)
pub const BOTH_VT_IDLE_SR: animNumber_t = 1080; //# Sit (saber right)

pub const BOTH_VT_IDLE: animNumber_t = 1081; //# Sit with no weapon selected
pub const BOTH_VT_IDLE1: animNumber_t = 1082; //# Sit with no weapon selected
pub const BOTH_VT_IDLE_S: animNumber_t = 1083; //# Sit with saber selected
pub const BOTH_VT_IDLE_G: animNumber_t = 1084; //# Sit with gun selected
pub const BOTH_VT_IDLE_T: animNumber_t = 1085; //# Sit with thermal grenade selected

pub const BOTH_VT_ATL_S: animNumber_t = 1086; //# Attack left with saber
pub const BOTH_VT_ATR_S: animNumber_t = 1087; //# Attack right with saber
pub const BOTH_VT_ATR_TO_L_S: animNumber_t = 1088; //# Attack toss saber from right to left hand
pub const BOTH_VT_ATL_TO_R_S: animNumber_t = 1089; //# Attack toss saber from left to right hand
pub const BOTH_VT_ATR_G: animNumber_t = 1090; //# Attack right with gun (90)
pub const BOTH_VT_ATL_G: animNumber_t = 1091; //# Attack left with gun (90)
pub const BOTH_VT_ATF_G: animNumber_t = 1092; //# Attack forward with gun


// Added 2/26/02 by Aurelio.
//* #sep BOTH_ FIGHTER ANIMS
pub const BOTH_GEARS_OPEN: animNumber_t = 1093;
pub const BOTH_GEARS_CLOSE: animNumber_t = 1094;
pub const BOTH_WINGS_OPEN: animNumber_t = 1095;
pub const BOTH_WINGS_CLOSE: animNumber_t = 1096;

pub const BOTH_DEATH14_UNGRIP: animNumber_t = 1097; //# Desann's end death (cin #35)
pub const BOTH_DEATH14_SITUP: animNumber_t = 1098; //# Tavion sitting up after having been thrown (cin #23)
pub const BOTH_KNEES1: animNumber_t = 1099; //# Tavion on her knees
pub const BOTH_KNEES2: animNumber_t = 1100; //# Tavion on her knees looking down
pub const BOTH_KNEES2TO1: animNumber_t = 1101; //# Transition of KNEES2 to KNEES1

//# #sep BOTH_ MOVING
pub const BOTH_WALK1: animNumber_t = 1102; //# Normal walk
pub const BOTH_WALK2: animNumber_t = 1103; //# Normal walk
pub const BOTH_WALK_STAFF: animNumber_t = 1104; //# Walk with saberstaff turned on
pub const BOTH_WALKBACK_STAFF: animNumber_t = 1105; //# Walk backwards with saberstaff turned on
pub const BOTH_WALK_DUAL: animNumber_t = 1106; //# Walk with dual turned on
pub const BOTH_WALKBACK_DUAL: animNumber_t = 1107; //# Walk backwards with dual turned on
pub const BOTH_WALK5: animNumber_t = 1108; //# Tavion taunting Kyle (cin 22)
pub const BOTH_WALK6: animNumber_t = 1109; //# Slow walk for Luke (cin 12)
pub const BOTH_WALK7: animNumber_t = 1110; //# Fast walk
pub const BOTH_RUN1: animNumber_t = 1111; //# Full run
pub const BOTH_RUN1START: animNumber_t = 1112; //# Start into full run1
pub const BOTH_RUN1STOP: animNumber_t = 1113; //# Stop from full run1
pub const BOTH_RUN2: animNumber_t = 1114; //# Full run
pub const BOTH_RUN1TORUN2: animNumber_t = 1115; //# Wampa run anim transition
pub const BOTH_RUN2TORUN1: animNumber_t = 1116; //# Wampa run anim transition
pub const BOTH_RUN4: animNumber_t = 1117; //# Jawa Run
pub const BOTH_RUN_STAFF: animNumber_t = 1118; //# Run with saberstaff turned on
pub const BOTH_RUNBACK_STAFF: animNumber_t = 1119; //# Run backwards with saberstaff turned on
pub const BOTH_RUN_DUAL: animNumber_t = 1120; //# Run with dual turned on
pub const BOTH_RUNBACK_DUAL: animNumber_t = 1121; //# Run backwards with dual turned on
pub const BOTH_STRAFE_LEFT1: animNumber_t = 1122; //# Sidestep left, should loop
pub const BOTH_STRAFE_RIGHT1: animNumber_t = 1123; //# Sidestep right, should loop
pub const BOTH_RUNSTRAFE_LEFT1: animNumber_t = 1124; //# Sidestep left, should loop
pub const BOTH_RUNSTRAFE_RIGHT1: animNumber_t = 1125; //# Sidestep right, should loop
pub const BOTH_TURN_LEFT1: animNumber_t = 1126; //# Turn left, should loop
pub const BOTH_TURN_RIGHT1: animNumber_t = 1127; //# Turn right, should loop
pub const BOTH_TURNSTAND1: animNumber_t = 1128; //# Turn from STAND1 position
pub const BOTH_TURNSTAND2: animNumber_t = 1129; //# Turn from STAND2 position
pub const BOTH_TURNSTAND3: animNumber_t = 1130; //# Turn from STAND3 position
pub const BOTH_TURNSTAND4: animNumber_t = 1131; //# Turn from STAND4 position
pub const BOTH_TURNSTAND5: animNumber_t = 1132; //# Turn from STAND5 position
pub const BOTH_TURNCROUCH1: animNumber_t = 1133; //# Turn from CROUCH1 position

pub const BOTH_WALKBACK1: animNumber_t = 1134; //# Walk1 backwards
pub const BOTH_WALKBACK2: animNumber_t = 1135; //# Walk2 backwards
pub const BOTH_RUNBACK1: animNumber_t = 1136; //# Run1 backwards
pub const BOTH_RUNBACK2: animNumber_t = 1137; //# Run1 backwards

//# #sep BOTH_ JUMPING
pub const BOTH_JUMP1: animNumber_t = 1138; //# Jump - wind-up and leave ground
pub const BOTH_INAIR1: animNumber_t = 1139; //# In air loop (from jump)
pub const BOTH_LAND1: animNumber_t = 1140; //# Landing (from in air loop)
pub const BOTH_LAND2: animNumber_t = 1141; //# Landing Hard (from a great height)

pub const BOTH_JUMPBACK1: animNumber_t = 1142; //# Jump backwards - wind-up and leave ground
pub const BOTH_INAIRBACK1: animNumber_t = 1143; //# In air loop (from jump back)
pub const BOTH_LANDBACK1: animNumber_t = 1144; //# Landing backwards(from in air loop)

pub const BOTH_JUMPLEFT1: animNumber_t = 1145; //# Jump left - wind-up and leave ground
pub const BOTH_INAIRLEFT1: animNumber_t = 1146; //# In air loop (from jump left)
pub const BOTH_LANDLEFT1: animNumber_t = 1147; //# Landing left(from in air loop)

pub const BOTH_JUMPRIGHT1: animNumber_t = 1148; //# Jump right - wind-up and leave ground
pub const BOTH_INAIRRIGHT1: animNumber_t = 1149; //# In air loop (from jump right)
pub const BOTH_LANDRIGHT1: animNumber_t = 1150; //# Landing right(from in air loop)

pub const BOTH_FORCEJUMP1: animNumber_t = 1151; //# Jump - wind-up and leave ground
pub const BOTH_FORCEINAIR1: animNumber_t = 1152; //# In air loop (from jump)
pub const BOTH_FORCELAND1: animNumber_t = 1153; //# Landing (from in air loop)

pub const BOTH_FORCEJUMPBACK1: animNumber_t = 1154; //# Jump backwards - wind-up and leave ground
pub const BOTH_FORCEINAIRBACK1: animNumber_t = 1155; //# In air loop (from jump back)
pub const BOTH_FORCELANDBACK1: animNumber_t = 1156; //# Landing backwards(from in air loop)

pub const BOTH_FORCEJUMPLEFT1: animNumber_t = 1157; //# Jump left - wind-up and leave ground
pub const BOTH_FORCEINAIRLEFT1: animNumber_t = 1158; //# In air loop (from jump left)
pub const BOTH_FORCELANDLEFT1: animNumber_t = 1159; //# Landing left(from in air loop)

pub const BOTH_FORCEJUMPRIGHT1: animNumber_t = 1160; //# Jump right - wind-up and leave ground
pub const BOTH_FORCEINAIRRIGHT1: animNumber_t = 1161; //# In air loop (from jump right)
pub const BOTH_FORCELANDRIGHT1: animNumber_t = 1162; //# Landing right(from in air loop)
//# #sep BOTH_ ACROBATICS
pub const BOTH_FLIP_F: animNumber_t = 1163; //# Flip forward
pub const BOTH_FLIP_B: animNumber_t = 1164; //# Flip backwards
pub const BOTH_FLIP_L: animNumber_t = 1165; //# Flip left
pub const BOTH_FLIP_R: animNumber_t = 1166; //# Flip right

pub const BOTH_ROLL_F: animNumber_t = 1167; //# Roll forward
pub const BOTH_ROLL_B: animNumber_t = 1168; //# Roll backward
pub const BOTH_ROLL_L: animNumber_t = 1169; //# Roll left
pub const BOTH_ROLL_R: animNumber_t = 1170; //# Roll right

pub const BOTH_HOP_F: animNumber_t = 1171; //# quickstep forward
pub const BOTH_HOP_B: animNumber_t = 1172; //# quickstep backwards
pub const BOTH_HOP_L: animNumber_t = 1173; //# quickstep left
pub const BOTH_HOP_R: animNumber_t = 1174; //# quickstep right

pub const BOTH_DODGE_FL: animNumber_t = 1175; //# lean-dodge forward left
pub const BOTH_DODGE_FR: animNumber_t = 1176; //# lean-dodge forward right
pub const BOTH_DODGE_BL: animNumber_t = 1177; //# lean-dodge backwards left
pub const BOTH_DODGE_BR: animNumber_t = 1178; //# lean-dodge backwards right
pub const BOTH_DODGE_L: animNumber_t = 1179; //# lean-dodge left
pub const BOTH_DODGE_R: animNumber_t = 1180; //# lean-dodge right
pub const BOTH_DODGE_HOLD_FL: animNumber_t = 1181; //# lean-dodge pose forward left
pub const BOTH_DODGE_HOLD_FR: animNumber_t = 1182; //# lean-dodge pose forward right
pub const BOTH_DODGE_HOLD_BL: animNumber_t = 1183; //# lean-dodge pose backwards left
pub const BOTH_DODGE_HOLD_BR: animNumber_t = 1184; //# lean-dodge pose backwards right
pub const BOTH_DODGE_HOLD_L: animNumber_t = 1185; //# lean-dodge pose left
pub const BOTH_DODGE_HOLD_R: animNumber_t = 1186; //# lean-dodge pose right

//MP taunt anims
pub const BOTH_ENGAGETAUNT: animNumber_t = 1187;
pub const BOTH_BOW: animNumber_t = 1188;
pub const BOTH_MEDITATE: animNumber_t = 1189;
pub const BOTH_MEDITATE_END: animNumber_t = 1190;
pub const BOTH_SHOWOFF_FAST: animNumber_t = 1191;
pub const BOTH_SHOWOFF_MEDIUM: animNumber_t = 1192;
pub const BOTH_SHOWOFF_STRONG: animNumber_t = 1193;
pub const BOTH_SHOWOFF_DUAL: animNumber_t = 1194;
pub const BOTH_SHOWOFF_STAFF: animNumber_t = 1195;
pub const BOTH_VICTORY_FAST: animNumber_t = 1196;
pub const BOTH_VICTORY_MEDIUM: animNumber_t = 1197;
pub const BOTH_VICTORY_STRONG: animNumber_t = 1198;
pub const BOTH_VICTORY_DUAL: animNumber_t = 1199;
pub const BOTH_VICTORY_STAFF: animNumber_t = 1200;
//other saber/acro anims
pub const BOTH_ARIAL_LEFT: animNumber_t = 1201; //#
pub const BOTH_ARIAL_RIGHT: animNumber_t = 1202; //#
pub const BOTH_CARTWHEEL_LEFT: animNumber_t = 1203; //#
pub const BOTH_CARTWHEEL_RIGHT: animNumber_t = 1204; //#
pub const BOTH_FLIP_LEFT: animNumber_t = 1205; //#
pub const BOTH_FLIP_BACK1: animNumber_t = 1206; //#
pub const BOTH_FLIP_BACK2: animNumber_t = 1207; //#
pub const BOTH_FLIP_BACK3: animNumber_t = 1208; //#
pub const BOTH_BUTTERFLY_LEFT: animNumber_t = 1209; //#
pub const BOTH_BUTTERFLY_RIGHT: animNumber_t = 1210; //#
pub const BOTH_WALL_RUN_RIGHT: animNumber_t = 1211; //#
pub const BOTH_WALL_RUN_RIGHT_FLIP: animNumber_t = 1212; //#
pub const BOTH_WALL_RUN_RIGHT_STOP: animNumber_t = 1213; //#
pub const BOTH_WALL_RUN_LEFT: animNumber_t = 1214; //#
pub const BOTH_WALL_RUN_LEFT_FLIP: animNumber_t = 1215; //#
pub const BOTH_WALL_RUN_LEFT_STOP: animNumber_t = 1216; //#
pub const BOTH_WALL_FLIP_RIGHT: animNumber_t = 1217; //#
pub const BOTH_WALL_FLIP_LEFT: animNumber_t = 1218; //#
pub const BOTH_KNOCKDOWN1: animNumber_t = 1219; //# knocked backwards
pub const BOTH_KNOCKDOWN2: animNumber_t = 1220; //# knocked backwards hard
pub const BOTH_KNOCKDOWN3: animNumber_t = 1221; //#	knocked forwards
pub const BOTH_KNOCKDOWN4: animNumber_t = 1222; //# knocked backwards from crouch
pub const BOTH_KNOCKDOWN5: animNumber_t = 1223; //# dupe of 3 - will be removed
pub const BOTH_GETUP1: animNumber_t = 1224; //#
pub const BOTH_GETUP2: animNumber_t = 1225; //#
pub const BOTH_GETUP3: animNumber_t = 1226; //#
pub const BOTH_GETUP4: animNumber_t = 1227; //#
pub const BOTH_GETUP5: animNumber_t = 1228; //#
pub const BOTH_GETUP_CROUCH_F1: animNumber_t = 1229; //#
pub const BOTH_GETUP_CROUCH_B1: animNumber_t = 1230; //#
pub const BOTH_FORCE_GETUP_F1: animNumber_t = 1231; //#
pub const BOTH_FORCE_GETUP_F2: animNumber_t = 1232; //#
pub const BOTH_FORCE_GETUP_B1: animNumber_t = 1233; //#
pub const BOTH_FORCE_GETUP_B2: animNumber_t = 1234; //#
pub const BOTH_FORCE_GETUP_B3: animNumber_t = 1235; //#
pub const BOTH_FORCE_GETUP_B4: animNumber_t = 1236; //#
pub const BOTH_FORCE_GETUP_B5: animNumber_t = 1237; //#
pub const BOTH_FORCE_GETUP_B6: animNumber_t = 1238; //#
pub const BOTH_GETUP_BROLL_B: animNumber_t = 1239; //#
pub const BOTH_GETUP_BROLL_F: animNumber_t = 1240; //#
pub const BOTH_GETUP_BROLL_L: animNumber_t = 1241; //#
pub const BOTH_GETUP_BROLL_R: animNumber_t = 1242; //#
pub const BOTH_GETUP_FROLL_B: animNumber_t = 1243; //#
pub const BOTH_GETUP_FROLL_F: animNumber_t = 1244; //#
pub const BOTH_GETUP_FROLL_L: animNumber_t = 1245; //#
pub const BOTH_GETUP_FROLL_R: animNumber_t = 1246; //#
pub const BOTH_WALL_FLIP_BACK1: animNumber_t = 1247; //#
pub const BOTH_WALL_FLIP_BACK2: animNumber_t = 1248; //#
pub const BOTH_SPIN1: animNumber_t = 1249; //#
pub const BOTH_CEILING_CLING: animNumber_t = 1250; //# clinging to ceiling
pub const BOTH_CEILING_DROP: animNumber_t = 1251; //# dropping from ceiling cling

//TESTING
pub const BOTH_FJSS_TR_BL: animNumber_t = 1252; //# jump spin slash tr to bl
pub const BOTH_FJSS_TL_BR: animNumber_t = 1253; //# jump spin slash bl to tr
pub const BOTH_RIGHTHANDCHOPPEDOFF: animNumber_t = 1254; //#
pub const BOTH_DEFLECTSLASH__R__L_FIN: animNumber_t = 1255; //#
pub const BOTH_BASHED1: animNumber_t = 1256; //#
pub const BOTH_ARIAL_F1: animNumber_t = 1257; //#
pub const BOTH_BUTTERFLY_FR1: animNumber_t = 1258; //#
pub const BOTH_BUTTERFLY_FL1: animNumber_t = 1259; //#

//NEW SABER/JEDI/FORCE ANIMS
pub const BOTH_BACK_FLIP_UP: animNumber_t = 1260; //# back flip up Bonus Animation!!!!
pub const BOTH_LOSE_SABER: animNumber_t = 1261; //# player losing saber (pulled from hand by force pull 4 - Kyle?)
pub const BOTH_STAFF_TAUNT: animNumber_t = 1262; //# taunt saberstaff
pub const BOTH_DUAL_TAUNT: animNumber_t = 1263; //# taunt dual
pub const BOTH_A6_FB: animNumber_t = 1264; //# dual attack front/back
pub const BOTH_A6_LR: animNumber_t = 1265; //# dual attack left/right
pub const BOTH_A7_HILT: animNumber_t = 1266; //# saber knock (alt + stand still)
//Alora
pub const BOTH_ALORA_SPIN: animNumber_t = 1267; //#jump spin attack	death ballet
pub const BOTH_ALORA_FLIP_1: animNumber_t = 1268; //# gymnast move 1
pub const BOTH_ALORA_FLIP_2: animNumber_t = 1269; //# gymnast move 2
pub const BOTH_ALORA_FLIP_3: animNumber_t = 1270; //# gymnast move3
pub const BOTH_ALORA_FLIP_B: animNumber_t = 1271; //# gymnast move back
pub const BOTH_ALORA_SPIN_THROW: animNumber_t = 1272; //# dual saber throw
pub const BOTH_ALORA_SPIN_SLASH: animNumber_t = 1273; //# spin slash	special bonus animation!! :)
pub const BOTH_ALORA_TAUNT: animNumber_t = 1274; //# special taunt
//Rosh (Kothos battle)
pub const BOTH_ROSH_PAIN: animNumber_t = 1275; //# hurt animation (exhausted)
pub const BOTH_ROSH_HEAL: animNumber_t = 1276; //# healed/rejuvenated
//Tavion
pub const BOTH_TAVION_SCEPTERGROUND: animNumber_t = 1277; //# stabbing ground with sith sword shoots electricity everywhere
pub const BOTH_TAVION_SWORDPOWER: animNumber_t = 1278; //# Tavion doing the He-Man(tm) thing
pub const BOTH_SCEPTER_START: animNumber_t = 1279; //#Point scepter and attack start
pub const BOTH_SCEPTER_HOLD: animNumber_t = 1280; //#Point scepter and attack hold
pub const BOTH_SCEPTER_STOP: animNumber_t = 1281; //#Point scepter and attack stop
//Kyle Boss
pub const BOTH_KYLE_GRAB: animNumber_t = 1282; //# grab
pub const BOTH_KYLE_MISS: animNumber_t = 1283; //# miss
pub const BOTH_KYLE_PA_1: animNumber_t = 1284; //# hold 1
pub const BOTH_PLAYER_PA_1: animNumber_t = 1285; //# player getting held 1
pub const BOTH_KYLE_PA_2: animNumber_t = 1286; //# hold 2
pub const BOTH_PLAYER_PA_2: animNumber_t = 1287; //# player getting held 2
pub const BOTH_PLAYER_PA_FLY: animNumber_t = 1288; //# player getting knocked back from punch at end of hold 1
pub const BOTH_KYLE_PA_3: animNumber_t = 1289; //# hold 3
pub const BOTH_PLAYER_PA_3: animNumber_t = 1290; //# player getting held 3
pub const BOTH_PLAYER_PA_3_FLY: animNumber_t = 1291; //# player getting thrown at end of hold 3
//Rancor
pub const BOTH_BUCK_RIDER: animNumber_t = 1292; //# Rancor bucks when someone is on him
//WAMPA Grabbing enemy
pub const BOTH_HOLD_START: animNumber_t = 1293; //#
pub const BOTH_HOLD_MISS: animNumber_t = 1294; //#
pub const BOTH_HOLD_IDLE: animNumber_t = 1295; //#
pub const BOTH_HOLD_END: animNumber_t = 1296; //#
pub const BOTH_HOLD_ATTACK: animNumber_t = 1297; //#
pub const BOTH_HOLD_SNIFF: animNumber_t = 1298; //# Sniff the guy you're holding
pub const BOTH_HOLD_DROP: animNumber_t = 1299; //# just drop 'em
//BEING GRABBED BY WAMPA
pub const BOTH_GRABBED: animNumber_t = 1300; //#
pub const BOTH_RELEASED: animNumber_t = 1301; //#
pub const BOTH_HANG_IDLE: animNumber_t = 1302; //#
pub const BOTH_HANG_ATTACK: animNumber_t = 1303; //#
pub const BOTH_HANG_PAIN: animNumber_t = 1304; //#

//# #sep BOTH_ MISC MOVEMENT
pub const BOTH_HIT1: animNumber_t = 1305; //# Kyle hit by crate in cin #9
pub const BOTH_LADDER_UP1: animNumber_t = 1306; //# Climbing up a ladder with rungs at 16 unit intervals
pub const BOTH_LADDER_DWN1: animNumber_t = 1307; //# Climbing down a ladder with rungs at 16 unit intervals
pub const BOTH_LADDER_IDLE: animNumber_t = 1308; //#	Just sitting on the ladder

//# #sep BOTH_ FLYING IDLE
pub const BOTH_FLY_SHIELDED: animNumber_t = 1309; //# For sentry droid, shields in

//# #sep BOTH_ SWIMMING
pub const BOTH_SWIM_IDLE1: animNumber_t = 1310; //# Swimming Idle 1
pub const BOTH_SWIMFORWARD: animNumber_t = 1311; //# Swim forward loop
pub const BOTH_SWIMBACKWARD: animNumber_t = 1312; //# Swim backward loop

//# #sep BOTH_ LYING
pub const BOTH_SLEEP1: animNumber_t = 1313; //# laying on back-rknee up-rhand on torso
pub const BOTH_SLEEP6START: animNumber_t = 1314; //# Kyle leaning back to sleep (cin 20)
pub const BOTH_SLEEP6STOP: animNumber_t = 1315; //# Kyle waking up and shaking his head (cin 21)
pub const BOTH_SLEEP1GETUP: animNumber_t = 1316; //# alarmed and getting up out of sleep1 pose to stand
pub const BOTH_SLEEP1GETUP2: animNumber_t = 1317; //#

pub const BOTH_CHOKE1START: animNumber_t = 1318; //# tavion in force grip choke
pub const BOTH_CHOKE1STARTHOLD: animNumber_t = 1319; //# loop of tavion in force grip choke
pub const BOTH_CHOKE1: animNumber_t = 1320; //# tavion in force grip choke

pub const BOTH_CHOKE2: animNumber_t = 1321; //# tavion recovering from force grip choke
pub const BOTH_CHOKE3: animNumber_t = 1322; //# left-handed choke (for people still holding a weapon)

//# #sep BOTH_ HUNTER-SEEKER BOT-SPECIFIC
pub const BOTH_POWERUP1: animNumber_t = 1323; //# Wakes up

pub const BOTH_TURNON: animNumber_t = 1324; //# Protocol Droid wakes up
pub const BOTH_TURNOFF: animNumber_t = 1325; //# Protocol Droid shuts off

pub const BOTH_BUTTON1: animNumber_t = 1326; //# Single button push with right hand
pub const BOTH_BUTTON2: animNumber_t = 1327; //# Single button push with left finger
pub const BOTH_BUTTON_HOLD: animNumber_t = 1328; //# Single button hold with left hand
pub const BOTH_BUTTON_RELEASE: animNumber_t = 1329; //# Single button release with left hand

//# JEDI-SPECIFIC
//# #sep BOTH_ FORCE ANIMS
pub const BOTH_RESISTPUSH: animNumber_t = 1330; //# plant yourself to resist force push/pulls.
pub const BOTH_FORCEPUSH: animNumber_t = 1331; //# Use off-hand to do force power.
pub const BOTH_FORCEPULL: animNumber_t = 1332; //# Use off-hand to do force power.
pub const BOTH_MINDTRICK1: animNumber_t = 1333; //# Use off-hand to do mind trick
pub const BOTH_MINDTRICK2: animNumber_t = 1334; //# Use off-hand to do distraction
pub const BOTH_FORCELIGHTNING: animNumber_t = 1335; //# Use off-hand to do lightning
pub const BOTH_FORCELIGHTNING_START: animNumber_t = 1336; //# Use off-hand to do lightning - start
pub const BOTH_FORCELIGHTNING_HOLD: animNumber_t = 1337; //# Use off-hand to do lightning - hold
pub const BOTH_FORCELIGHTNING_RELEASE: animNumber_t = 1338; //# Use off-hand to do lightning - release
pub const BOTH_FORCEHEAL_START: animNumber_t = 1339; //# Healing meditation pose start
pub const BOTH_FORCEHEAL_STOP: animNumber_t = 1340; //# Healing meditation pose end
pub const BOTH_FORCEHEAL_QUICK: animNumber_t = 1341; //# Healing meditation gesture
pub const BOTH_SABERPULL: animNumber_t = 1342; //# Use off-hand to do force power.
pub const BOTH_FORCEGRIP1: animNumber_t = 1343; //# force-gripping (no anim?)
pub const BOTH_FORCEGRIP3: animNumber_t = 1344; //# force-gripping (right hand)
pub const BOTH_FORCEGRIP3THROW: animNumber_t = 1345; //# throwing while force-gripping (right hand)
pub const BOTH_FORCEGRIP_HOLD: animNumber_t = 1346; //# Use off-hand to do grip - hold
pub const BOTH_FORCEGRIP_RELEASE: animNumber_t = 1347; //# Use off-hand to do grip - release
pub const BOTH_TOSS1: animNumber_t = 1348; //# throwing to left after force gripping
pub const BOTH_TOSS2: animNumber_t = 1349; //# throwing to right after force gripping
//NEW force anims for JKA:
pub const BOTH_FORCE_RAGE: animNumber_t = 1350;
pub const BOTH_FORCE_2HANDEDLIGHTNING: animNumber_t = 1351;
pub const BOTH_FORCE_2HANDEDLIGHTNING_START: animNumber_t = 1352;
pub const BOTH_FORCE_2HANDEDLIGHTNING_HOLD: animNumber_t = 1353;
pub const BOTH_FORCE_2HANDEDLIGHTNING_RELEASE: animNumber_t = 1354;
pub const BOTH_FORCE_DRAIN: animNumber_t = 1355;
pub const BOTH_FORCE_DRAIN_START: animNumber_t = 1356;
pub const BOTH_FORCE_DRAIN_HOLD: animNumber_t = 1357;
pub const BOTH_FORCE_DRAIN_RELEASE: animNumber_t = 1358;
pub const BOTH_FORCE_DRAIN_GRAB_START: animNumber_t = 1359;
pub const BOTH_FORCE_DRAIN_GRAB_HOLD: animNumber_t = 1360;
pub const BOTH_FORCE_DRAIN_GRAB_END: animNumber_t = 1361;
pub const BOTH_FORCE_DRAIN_GRABBED: animNumber_t = 1362;
pub const BOTH_FORCE_ABSORB: animNumber_t = 1363;
pub const BOTH_FORCE_ABSORB_START: animNumber_t = 1364;
pub const BOTH_FORCE_ABSORB_END: animNumber_t = 1365;
pub const BOTH_FORCE_PROTECT: animNumber_t = 1366;
pub const BOTH_FORCE_PROTECT_FAST: animNumber_t = 1367;

pub const BOTH_WIND: animNumber_t = 1368;

pub const BOTH_STAND_TO_KNEEL: animNumber_t = 1369;
pub const BOTH_KNEEL_TO_STAND: animNumber_t = 1370;

pub const BOTH_TUSKENATTACK1: animNumber_t = 1371;
pub const BOTH_TUSKENATTACK2: animNumber_t = 1372;
pub const BOTH_TUSKENATTACK3: animNumber_t = 1373;
pub const BOTH_TUSKENLUNGE1: animNumber_t = 1374;
pub const BOTH_TUSKENTAUNT1: animNumber_t = 1375;

pub const BOTH_COWER1_START: animNumber_t = 1376; //# cower start
pub const BOTH_COWER1: animNumber_t = 1377; //# cower loop
pub const BOTH_COWER1_STOP: animNumber_t = 1378; //# cower stop
pub const BOTH_SONICPAIN_START: animNumber_t = 1379;
pub const BOTH_SONICPAIN_HOLD: animNumber_t = 1380;
pub const BOTH_SONICPAIN_END: animNumber_t = 1381;

//new anim slots per Jarrod's request
pub const BOTH_STAND10: animNumber_t = 1382;
pub const BOTH_STAND10_TALK1: animNumber_t = 1383;
pub const BOTH_STAND10_TALK2: animNumber_t = 1384;
pub const BOTH_STAND10TOSTAND1: animNumber_t = 1385;

pub const BOTH_STAND1_TALK1: animNumber_t = 1386;
pub const BOTH_STAND1_TALK2: animNumber_t = 1387;
pub const BOTH_STAND1_TALK3: animNumber_t = 1388;

pub const BOTH_SIT4: animNumber_t = 1389;
pub const BOTH_SIT5: animNumber_t = 1390;
pub const BOTH_SIT5_TALK1: animNumber_t = 1391;
pub const BOTH_SIT5_TALK2: animNumber_t = 1392;
pub const BOTH_SIT5_TALK3: animNumber_t = 1393;

pub const BOTH_SIT6: animNumber_t = 1394;
pub const BOTH_SIT7: animNumber_t = 1395;

//=================================================
//ANIMS IN WHICH ONLY THE UPPER OBJECTS ARE IN MD3
//=================================================
//# #sep TORSO_ WEAPON-RELATED
pub const TORSO_DROPWEAP1: animNumber_t = 1396; //# Put weapon away
pub const TORSO_DROPWEAP4: animNumber_t = 1397; //# Put weapon away
pub const TORSO_RAISEWEAP1: animNumber_t = 1398; //# Draw Weapon
pub const TORSO_RAISEWEAP4: animNumber_t = 1399; //# Draw Weapon
pub const TORSO_WEAPONREADY1: animNumber_t = 1400; //# Ready to fire stun baton
pub const TORSO_WEAPONREADY2: animNumber_t = 1401; //# Ready to fire one-handed blaster pistol
pub const TORSO_WEAPONREADY3: animNumber_t = 1402; //# Ready to fire blaster rifle
pub const TORSO_WEAPONREADY4: animNumber_t = 1403; //# Ready to fire sniper rifle
pub const TORSO_WEAPONREADY10: animNumber_t = 1404; //# Ready to fire thermal det
pub const TORSO_WEAPONIDLE2: animNumber_t = 1405; //# Holding one-handed blaster
pub const TORSO_WEAPONIDLE3: animNumber_t = 1406; //# Holding blaster rifle
pub const TORSO_WEAPONIDLE4: animNumber_t = 1407; //# Holding sniper rifle
pub const TORSO_WEAPONIDLE10: animNumber_t = 1408; //# Holding thermal det

//# #sep TORSO_ MISC
pub const TORSO_SURRENDER_START: animNumber_t = 1409; //# arms up
pub const TORSO_SURRENDER_STOP: animNumber_t = 1410; //# arms back down

pub const TORSO_CHOKING1: animNumber_t = 1411; //# TEMP

pub const TORSO_HANDSIGNAL1: animNumber_t = 1412;
pub const TORSO_HANDSIGNAL2: animNumber_t = 1413;
pub const TORSO_HANDSIGNAL3: animNumber_t = 1414;
pub const TORSO_HANDSIGNAL4: animNumber_t = 1415;
pub const TORSO_HANDSIGNAL5: animNumber_t = 1416;


//=================================================
//ANIMS IN WHICH ONLY THE LOWER OBJECTS ARE IN MD3
//=================================================
//# #sep Legs-only anims
pub const LEGS_TURN1: animNumber_t = 1417; //# What legs do when you turn your lower body to match your upper body facing
pub const LEGS_TURN2: animNumber_t = 1418; //# Leg turning from stand2
pub const LEGS_LEAN_LEFT1: animNumber_t = 1419; //# Lean left
pub const LEGS_LEAN_RIGHT1: animNumber_t = 1420; //# Lean Right
pub const LEGS_CHOKING1: animNumber_t = 1421; //# TEMP
pub const LEGS_LEFTUP1: animNumber_t = 1422; //# On a slope with left foot 4 higher than right
pub const LEGS_LEFTUP2: animNumber_t = 1423; //# On a slope with left foot 8 higher than right
pub const LEGS_LEFTUP3: animNumber_t = 1424; //# On a slope with left foot 12 higher than right
pub const LEGS_LEFTUP4: animNumber_t = 1425; //# On a slope with left foot 16 higher than right
pub const LEGS_LEFTUP5: animNumber_t = 1426; //# On a slope with left foot 20 higher than right
pub const LEGS_RIGHTUP1: animNumber_t = 1427; //# On a slope with RIGHT foot 4 higher than left
pub const LEGS_RIGHTUP2: animNumber_t = 1428; //# On a slope with RIGHT foot 8 higher than left
pub const LEGS_RIGHTUP3: animNumber_t = 1429; //# On a slope with RIGHT foot 12 higher than left
pub const LEGS_RIGHTUP4: animNumber_t = 1430; //# On a slope with RIGHT foot 16 higher than left
pub const LEGS_RIGHTUP5: animNumber_t = 1431; //# On a slope with RIGHT foot 20 higher than left
pub const LEGS_S1_LUP1: animNumber_t = 1432;
pub const LEGS_S1_LUP2: animNumber_t = 1433;
pub const LEGS_S1_LUP3: animNumber_t = 1434;
pub const LEGS_S1_LUP4: animNumber_t = 1435;
pub const LEGS_S1_LUP5: animNumber_t = 1436;
pub const LEGS_S1_RUP1: animNumber_t = 1437;
pub const LEGS_S1_RUP2: animNumber_t = 1438;
pub const LEGS_S1_RUP3: animNumber_t = 1439;
pub const LEGS_S1_RUP4: animNumber_t = 1440;
pub const LEGS_S1_RUP5: animNumber_t = 1441;
pub const LEGS_S3_LUP1: animNumber_t = 1442;
pub const LEGS_S3_LUP2: animNumber_t = 1443;
pub const LEGS_S3_LUP3: animNumber_t = 1444;
pub const LEGS_S3_LUP4: animNumber_t = 1445;
pub const LEGS_S3_LUP5: animNumber_t = 1446;
pub const LEGS_S3_RUP1: animNumber_t = 1447;
pub const LEGS_S3_RUP2: animNumber_t = 1448;
pub const LEGS_S3_RUP3: animNumber_t = 1449;
pub const LEGS_S3_RUP4: animNumber_t = 1450;
pub const LEGS_S3_RUP5: animNumber_t = 1451;
pub const LEGS_S4_LUP1: animNumber_t = 1452;
pub const LEGS_S4_LUP2: animNumber_t = 1453;
pub const LEGS_S4_LUP3: animNumber_t = 1454;
pub const LEGS_S4_LUP4: animNumber_t = 1455;
pub const LEGS_S4_LUP5: animNumber_t = 1456;
pub const LEGS_S4_RUP1: animNumber_t = 1457;
pub const LEGS_S4_RUP2: animNumber_t = 1458;
pub const LEGS_S4_RUP3: animNumber_t = 1459;
pub const LEGS_S4_RUP4: animNumber_t = 1460;
pub const LEGS_S4_RUP5: animNumber_t = 1461;
pub const LEGS_S5_LUP1: animNumber_t = 1462;
pub const LEGS_S5_LUP2: animNumber_t = 1463;
pub const LEGS_S5_LUP3: animNumber_t = 1464;
pub const LEGS_S5_LUP4: animNumber_t = 1465;
pub const LEGS_S5_LUP5: animNumber_t = 1466;
pub const LEGS_S5_RUP1: animNumber_t = 1467;
pub const LEGS_S5_RUP2: animNumber_t = 1468;
pub const LEGS_S5_RUP3: animNumber_t = 1469;
pub const LEGS_S5_RUP4: animNumber_t = 1470;
pub const LEGS_S5_RUP5: animNumber_t = 1471;
pub const LEGS_S6_LUP1: animNumber_t = 1472;
pub const LEGS_S6_LUP2: animNumber_t = 1473;
pub const LEGS_S6_LUP3: animNumber_t = 1474;
pub const LEGS_S6_LUP4: animNumber_t = 1475;
pub const LEGS_S6_LUP5: animNumber_t = 1476;
pub const LEGS_S6_RUP1: animNumber_t = 1477;
pub const LEGS_S6_RUP2: animNumber_t = 1478;
pub const LEGS_S6_RUP3: animNumber_t = 1479;
pub const LEGS_S6_RUP4: animNumber_t = 1480;
pub const LEGS_S6_RUP5: animNumber_t = 1481;
pub const LEGS_S7_LUP1: animNumber_t = 1482;
pub const LEGS_S7_LUP2: animNumber_t = 1483;
pub const LEGS_S7_LUP3: animNumber_t = 1484;
pub const LEGS_S7_LUP4: animNumber_t = 1485;
pub const LEGS_S7_LUP5: animNumber_t = 1486;
pub const LEGS_S7_RUP1: animNumber_t = 1487;
pub const LEGS_S7_RUP2: animNumber_t = 1488;
pub const LEGS_S7_RUP3: animNumber_t = 1489;
pub const LEGS_S7_RUP4: animNumber_t = 1490;
pub const LEGS_S7_RUP5: animNumber_t = 1491;

//New anim as per Jarrod's request
pub const LEGS_TURN180: animNumber_t = 1492;

//======================================================
//cinematic anims
//======================================================
//# #sep BOTH_ CINEMATIC-ONLY
pub const BOTH_CIN_1: animNumber_t = 1493; //# Level specific cinematic 1
pub const BOTH_CIN_2: animNumber_t = 1494; //# Level specific cinematic 2
pub const BOTH_CIN_3: animNumber_t = 1495; //# Level specific cinematic 3
pub const BOTH_CIN_4: animNumber_t = 1496; //# Level specific cinematic 4
pub const BOTH_CIN_5: animNumber_t = 1497; //# Level specific cinematic 5
pub const BOTH_CIN_6: animNumber_t = 1498; //# Level specific cinematic 6
pub const BOTH_CIN_7: animNumber_t = 1499; //# Level specific cinematic 7
pub const BOTH_CIN_8: animNumber_t = 1500; //# Level specific cinematic 8
pub const BOTH_CIN_9: animNumber_t = 1501; //# Level specific cinematic 9
pub const BOTH_CIN_10: animNumber_t = 1502; //# Level specific cinematic 10
pub const BOTH_CIN_11: animNumber_t = 1503; //# Level specific cinematic 11
pub const BOTH_CIN_12: animNumber_t = 1504; //# Level specific cinematic 12
pub const BOTH_CIN_13: animNumber_t = 1505; //# Level specific cinematic 13
pub const BOTH_CIN_14: animNumber_t = 1506; //# Level specific cinematic 14
pub const BOTH_CIN_15: animNumber_t = 1507; //# Level specific cinematic 15
pub const BOTH_CIN_16: animNumber_t = 1508; //# Level specific cinematic 16
pub const BOTH_CIN_17: animNumber_t = 1509; //# Level specific cinematic 17
pub const BOTH_CIN_18: animNumber_t = 1510; //# Level specific cinematic 18
pub const BOTH_CIN_19: animNumber_t = 1511; //# Level specific cinematic 19
pub const BOTH_CIN_20: animNumber_t = 1512; //# Level specific cinematic 20
pub const BOTH_CIN_21: animNumber_t = 1513; //# Level specific cinematic 21
pub const BOTH_CIN_22: animNumber_t = 1514; //# Level specific cinematic 22
pub const BOTH_CIN_23: animNumber_t = 1515; //# Level specific cinematic 23
pub const BOTH_CIN_24: animNumber_t = 1516; //# Level specific cinematic 24
pub const BOTH_CIN_25: animNumber_t = 1517; //# Level specific cinematic 25
pub const BOTH_CIN_26: animNumber_t = 1518; //# Level specific cinematic
pub const BOTH_CIN_27: animNumber_t = 1519; //# Level specific cinematic
pub const BOTH_CIN_28: animNumber_t = 1520; //# Level specific cinematic
pub const BOTH_CIN_29: animNumber_t = 1521; //# Level specific cinematic
pub const BOTH_CIN_30: animNumber_t = 1522; //# Level specific cinematic
pub const BOTH_CIN_31: animNumber_t = 1523; //# Level specific cinematic
pub const BOTH_CIN_32: animNumber_t = 1524; //# Level specific cinematic
pub const BOTH_CIN_33: animNumber_t = 1525; //# Level specific cinematic
pub const BOTH_CIN_34: animNumber_t = 1526; //# Level specific cinematic
pub const BOTH_CIN_35: animNumber_t = 1527; //# Level specific cinematic
pub const BOTH_CIN_36: animNumber_t = 1528; //# Level specific cinematic
pub const BOTH_CIN_37: animNumber_t = 1529; //# Level specific cinematic
pub const BOTH_CIN_38: animNumber_t = 1530; //# Level specific cinematic
pub const BOTH_CIN_39: animNumber_t = 1531; //# Level specific cinematic
pub const BOTH_CIN_40: animNumber_t = 1532; //# Level specific cinematic
pub const BOTH_CIN_41: animNumber_t = 1533; //# Level specific cinematic
pub const BOTH_CIN_42: animNumber_t = 1534; //# Level specific cinematic
pub const BOTH_CIN_43: animNumber_t = 1535; //# Level specific cinematic
pub const BOTH_CIN_44: animNumber_t = 1536; //# Level specific cinematic
pub const BOTH_CIN_45: animNumber_t = 1537; //# Level specific cinematic
pub const BOTH_CIN_46: animNumber_t = 1538; //# Level specific cinematic
pub const BOTH_CIN_47: animNumber_t = 1539; //# Level specific cinematic
pub const BOTH_CIN_48: animNumber_t = 1540; //# Level specific cinematic
pub const BOTH_CIN_49: animNumber_t = 1541; //# Level specific cinematic
pub const BOTH_CIN_50: animNumber_t = 1542; //# Level specific cinematic

//# #eol
pub const MAX_ANIMATIONS: animNumber_t = 1543;
pub const MAX_TOTALANIMATIONS: animNumber_t = 1544;

/// `SABER_ANIM_GROUP_SIZE` (`anims.h`) — the count of saber anims in one
/// power-level group, derived as the span between consecutive `A*_T__B_`
/// group starts (kept as the original arithmetic expression).
pub const SABER_ANIM_GROUP_SIZE: animNumber_t = BOTH_A2_T__B_ - BOTH_A1_T__B_;

// Compile-time pins for the load-bearing values (independently re-checked
// against the real C enum by the oracle test below).
const _: () = assert!(FACE_TALK0 == 0);
const _: () = assert!(BOTH_DEATH1 == 9);
const _: () = assert!(BOTH_A1_T__B_ == 126);
const _: () = assert!(BOTH_A2_T__B_ == 203);
const _: () = assert!(MAX_ANIMATIONS == 1543);
const _: () = assert!(MAX_TOTALANIMATIONS == 1544);
const _: () = assert!(SABER_ANIM_GROUP_SIZE == 77);

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;

    /// Parity: the Rust enumerator values match the authentic C `animNumber_t`
    /// (oracle `#include`s the real `anims.h`). Checkpoints are spread across the
    /// whole enum; with no explicit `= value` jumps in the source, matching the
    /// terminal counts plus this spread pins the entire sequence.
    #[test]
    fn anim_numbers_match_c() {
        unsafe {
            assert_eq!(FACE_TALK0, jka_anim_FACE_TALK0(), "FACE_TALK0");
            assert_eq!(BOTH_ATTACK10, jka_anim_BOTH_ATTACK10(), "BOTH_ATTACK10");
            assert_eq!(BOTH_A1_T__B_, jka_anim_BOTH_A1_T__B_(), "BOTH_A1_T__B_");
            assert_eq!(BOTH_A2_T__B_, jka_anim_BOTH_A2_T__B_(), "BOTH_A2_T__B_");
            assert_eq!(BOTH_T2__R_T_, jka_anim_BOTH_T2__R_T_(), "BOTH_T2__R_T_");
            assert_eq!(BOTH_A4_TL_BR, jka_anim_BOTH_A4_TL_BR(), "BOTH_A4_TL_BR");
            assert_eq!(BOTH_T5__L_BL, jka_anim_BOTH_T5__L_BL(), "BOTH_T5__L_BL");
            assert_eq!(BOTH_T7__R_TL, jka_anim_BOTH_T7__R_TL(), "BOTH_T7__R_TL");
            assert_eq!(BOTH_K7_S7_T_, jka_anim_BOTH_K7_S7_T_(), "BOTH_K7_S7_T_");
            assert_eq!(BOTH_BF1BREAK, jka_anim_BOTH_BF1BREAK(), "BOTH_BF1BREAK");
            assert_eq!(BOTH_CONSOLE2HOLDCOMSTOP, jka_anim_BOTH_CONSOLE2HOLDCOMSTOP(), "BOTH_CONSOLE2HOLDCOMSTOP");
            assert_eq!(BOTH_VT_IDLE_SR, jka_anim_BOTH_VT_IDLE_SR(), "BOTH_VT_IDLE_SR");
            assert_eq!(BOTH_VICTORY_STAFF, jka_anim_BOTH_VICTORY_STAFF(), "BOTH_VICTORY_STAFF");
            assert_eq!(BOTH_CHOKE1, jka_anim_BOTH_CHOKE1(), "BOTH_CHOKE1");
            assert_eq!(LEGS_S1_RUP4, jka_anim_LEGS_S1_RUP4(), "LEGS_S1_RUP4");
            assert_eq!(BOTH_CIN_50, jka_anim_BOTH_CIN_50(), "BOTH_CIN_50");
            assert_eq!(MAX_ANIMATIONS, jka_anim_MAX_ANIMATIONS(), "MAX_ANIMATIONS");
            assert_eq!(MAX_TOTALANIMATIONS, jka_anim_MAX_TOTALANIMATIONS(), "MAX_TOTALANIMATIONS");
            assert_eq!(SABER_ANIM_GROUP_SIZE, jka_SABER_ANIM_GROUP_SIZE());
        }
    }
}
