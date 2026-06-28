//! `chars.h` — bot characteristic ids.

#![allow(non_upper_case_globals)]

use core::ffi::c_int;

// name
pub const CHARACTERISTIC_NAME: c_int = 0; // string
// gender of the bot
pub const CHARACTERISTIC_GENDER: c_int = 1; // string ("male", "female", "it")
// attack skill
// >  0.0 && <  0.2 = don't move
// >  0.3 && <  1.0 = aim at enemy during retreat
// >  0.0 && <  0.4 = only move forward/backward
// >= 0.4 && <  1.0 = circle strafing
// >  0.7 && <  1.0 = random strafe direction change
pub const CHARACTERISTIC_ATTACK_SKILL: c_int = 2; // float [0, 1]
// weapon weight file
pub const CHARACTERISTIC_WEAPONWEIGHTS: c_int = 3; // string
// view angle difference to angle change factor
pub const CHARACTERISTIC_VIEW_FACTOR: c_int = 4; // float <0, 1]
// maximum view angle change
pub const CHARACTERISTIC_VIEW_MAXCHANGE: c_int = 5; // float [1, 360]
// reaction time in seconds
pub const CHARACTERISTIC_REACTIONTIME: c_int = 6; // float [0, 5]
// accuracy when aiming
pub const CHARACTERISTIC_AIM_ACCURACY: c_int = 7; // float [0, 1]
// weapon specific aim accuracy
pub const CHARACTERISTIC_AIM_ACCURACY_MACHINEGUN: c_int = 8; // float [0, 1]
pub const CHARACTERISTIC_AIM_ACCURACY_SHOTGUN: c_int = 9; // float [0, 1]
pub const CHARACTERISTIC_AIM_ACCURACY_ROCKETLAUNCHER: c_int = 10; // float [0, 1]
pub const CHARACTERISTIC_AIM_ACCURACY_GRENADELAUNCHER: c_int = 11; // float [0, 1]
pub const CHARACTERISTIC_AIM_ACCURACY_LIGHTNING: c_int = 12;
pub const CHARACTERISTIC_AIM_ACCURACY_PLASMAGUN: c_int = 13; // float [0, 1]
pub const CHARACTERISTIC_AIM_ACCURACY_RAILGUN: c_int = 14;
pub const CHARACTERISTIC_AIM_ACCURACY_BFG10K: c_int = 15; // float [0, 1]
// skill when aiming
// >  0.0 && <  0.9 = aim is affected by enemy movement
// >  0.4 && <= 0.8 = enemy linear leading
// >  0.8 && <= 1.0 = enemy exact movement leading
// >  0.5 && <= 1.0 = prediction shots when enemy is not visible
// >  0.6 && <= 1.0 = splash damage by shooting nearby geometry
pub const CHARACTERISTIC_AIM_SKILL: c_int = 16; // float [0, 1]
// weapon specific aim skill
pub const CHARACTERISTIC_AIM_SKILL_ROCKETLAUNCHER: c_int = 17; // float [0, 1]
pub const CHARACTERISTIC_AIM_SKILL_GRENADELAUNCHER: c_int = 18; // float [0, 1]
pub const CHARACTERISTIC_AIM_SKILL_PLASMAGUN: c_int = 19; // float [0, 1]
pub const CHARACTERISTIC_AIM_SKILL_BFG10K: c_int = 20; // float [0, 1]

// chat
pub const CHARACTERISTIC_CHAT_FILE: c_int = 21; // string
pub const CHARACTERISTIC_CHAT_NAME: c_int = 22; // string
pub const CHARACTERISTIC_CHAT_CPM: c_int = 23; // integer [1, 4000]
pub const CHARACTERISTIC_CHAT_INSULT: c_int = 24; // float [0, 1]
pub const CHARACTERISTIC_CHAT_MISC: c_int = 25; // float [0, 1]
pub const CHARACTERISTIC_CHAT_STARTENDLEVEL: c_int = 26; // float [0, 1]
pub const CHARACTERISTIC_CHAT_ENTEREXITGAME: c_int = 27; // float [0, 1]
pub const CHARACTERISTIC_CHAT_KILL: c_int = 28; // float [0, 1]
pub const CHARACTERISTIC_CHAT_DEATH: c_int = 29; // float [0, 1]
pub const CHARACTERISTIC_CHAT_ENEMYSUICIDE: c_int = 30; // float [0, 1]
pub const CHARACTERISTIC_CHAT_HITTALKING: c_int = 31; // float [0, 1]
pub const CHARACTERISTIC_CHAT_HITNODEATH: c_int = 32; // float [0, 1]
pub const CHARACTERISTIC_CHAT_HITNOKILL: c_int = 33; // float [0, 1]
pub const CHARACTERISTIC_CHAT_RANDOM: c_int = 34; // float [0, 1]
pub const CHARACTERISTIC_CHAT_REPLY: c_int = 35; // float [0, 1]

// movement
pub const CHARACTERISTIC_CROUCHER: c_int = 36; // float [0, 1]
pub const CHARACTERISTIC_JUMPER: c_int = 37; // float [0, 1]
pub const CHARACTERISTIC_WALKER: c_int = 48; // float [0, 1]
pub const CHARACTERISTIC_WEAPONJUMPING: c_int = 38; // float [0, 1]
pub const CHARACTERISTIC_GRAPPLE_USER: c_int = 39; // float [0, 1]

// goal
pub const CHARACTERISTIC_ITEMWEIGHTS: c_int = 40; // string
pub const CHARACTERISTIC_AGGRESSION: c_int = 41; // float [0, 1]
pub const CHARACTERISTIC_SELFPRESERVATION: c_int = 42; // float [0, 1]
pub const CHARACTERISTIC_VENGEFULNESS: c_int = 43; // float [0, 1]
pub const CHARACTERISTIC_CAMPER: c_int = 44; // float [0, 1]

pub const CHARACTERISTIC_EASY_FRAGGER: c_int = 45; // float [0, 1]
pub const CHARACTERISTIC_ALERTNESS: c_int = 46; // float [0, 1]
pub const CHARACTERISTIC_FIRETHROTTLE: c_int = 47; // float [0, 1]
