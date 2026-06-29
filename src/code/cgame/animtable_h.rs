// special file included only by cg_players.cpp & ui_players.cpp
//
// moved it from the original header file for PCH reasons...
//

use core::ffi::{c_int, c_char};

// Struct for animation table entries
#[repr(C)]
pub struct StringIDTable {
    pub name: *const c_char,
    pub value: c_int,
}

#[cfg(not(any(target_os = "xbox", feature = "xbox")))]
#[cfg(not(any(feature = "jk2exe", feature = "ui")))]
pub static mut animTable: [StringIDTable; 1540] = [
	//=================================================
	//HEAD ANIMS
	//=================================================
	//# #sep Head-only anims
	StringIDTable { name: b"FACE_TALK0\0".as_ptr() as _, value: 0 }, //# silent
	StringIDTable { name: b"FACE_TALK1\0".as_ptr() as _, value: 1 }, //# quiet
	StringIDTable { name: b"FACE_TALK2\0".as_ptr() as _, value: 2 }, //# semi-quiet
	StringIDTable { name: b"FACE_TALK3\0".as_ptr() as _, value: 3 }, //# semi-loud
	StringIDTable { name: b"FACE_TALK4\0".as_ptr() as _, value: 4 }, //# loud
	StringIDTable { name: b"FACE_ALERT\0".as_ptr() as _, value: 5 }, //# 
	StringIDTable { name: b"FACE_SMILE\0".as_ptr() as _, value: 6 }, //# 
	StringIDTable { name: b"FACE_FROWN\0".as_ptr() as _, value: 7 }, //# 
	StringIDTable { name: b"FACE_DEAD\0".as_ptr() as _, value: 8 }, //# 

	//=================================================
	//ANIMS IN WHICH UPPER AND LOWER OBJECTS ARE IN MD3
	//=================================================
	//# #sep ENUM2STRING(BOTH_ DEATHS
	StringIDTable { name: b"BOTH_DEATH1\0".as_ptr() as _, value: 9 }, //# First Death anim
	StringIDTable { name: b"BOTH_DEATH2\0".as_ptr() as _, value: 10 }, //# Second Death anim
	StringIDTable { name: b"BOTH_DEATH3\0".as_ptr() as _, value: 11 }, //# Third Death anim
	StringIDTable { name: b"BOTH_DEATH4\0".as_ptr() as _, value: 12 }, //# Fourth Death anim
	StringIDTable { name: b"BOTH_DEATH5\0".as_ptr() as _, value: 13 }, //# Fifth Death anim
	StringIDTable { name: b"BOTH_DEATH6\0".as_ptr() as _, value: 14 }, //# Sixth Death anim
	StringIDTable { name: b"BOTH_DEATH7\0".as_ptr() as _, value: 15 }, //# Seventh Death anim
	StringIDTable { name: b"BOTH_DEATH8\0".as_ptr() as _, value: 16 }, //# 
	StringIDTable { name: b"BOTH_DEATH9\0".as_ptr() as _, value: 17 }, //# 
	StringIDTable { name: b"BOTH_DEATH10\0".as_ptr() as _, value: 18 }, //# 
	StringIDTable { name: b"BOTH_DEATH11\0".as_ptr() as _, value: 19 }, //#
	StringIDTable { name: b"BOTH_DEATH12\0".as_ptr() as _, value: 20 }, //# 
	StringIDTable { name: b"BOTH_DEATH13\0".as_ptr() as _, value: 21 }, //# 
	StringIDTable { name: b"BOTH_DEATH14\0".as_ptr() as _, value: 22 }, //# 
	StringIDTable { name: b"BOTH_DEATH15\0".as_ptr() as _, value: 23 }, //# 
	StringIDTable { name: b"BOTH_DEATH16\0".as_ptr() as _, value: 24 }, //# 
	StringIDTable { name: b"BOTH_DEATH17\0".as_ptr() as _, value: 25 }, //# 
	StringIDTable { name: b"BOTH_DEATH18\0".as_ptr() as _, value: 26 }, //# 
	StringIDTable { name: b"BOTH_DEATH19\0".as_ptr() as _, value: 27 }, //# 
	StringIDTable { name: b"BOTH_DEATH20\0".as_ptr() as _, value: 28 }, //# 
	StringIDTable { name: b"BOTH_DEATH21\0".as_ptr() as _, value: 29 }, //# 
	StringIDTable { name: b"BOTH_DEATH22\0".as_ptr() as _, value: 30 }, //# 
	StringIDTable { name: b"BOTH_DEATH23\0".as_ptr() as _, value: 31 }, //# 
	StringIDTable { name: b"BOTH_DEATH24\0".as_ptr() as _, value: 32 }, //# 
	StringIDTable { name: b"BOTH_DEATH25\0".as_ptr() as _, value: 33 }, //# 

	StringIDTable { name: b"BOTH_DEATHFORWARD1\0".as_ptr() as _, value: 34 }, //# First Death in which they get thrown forward
	StringIDTable { name: b"BOTH_DEATHFORWARD2\0".as_ptr() as _, value: 35 }, //# Second Death in which they get thrown forward
	StringIDTable { name: b"BOTH_DEATHFORWARD3\0".as_ptr() as _, value: 36 }, //# Tavion's falling in cin# 23
	StringIDTable { name: b"BOTH_DEATHBACKWARD1\0".as_ptr() as _, value: 37 }, //# First Death in which they get thrown backward
	StringIDTable { name: b"BOTH_DEATHBACKWARD2\0".as_ptr() as _, value: 38 }, //# Second Death in which they get thrown backward

	StringIDTable { name: b"BOTH_DEATH1IDLE\0".as_ptr() as _, value: 39 }, //# Idle while close to death
	StringIDTable { name: b"BOTH_LYINGDEATH1\0".as_ptr() as _, value: 40 }, //# Death to play when killed lying down
	StringIDTable { name: b"BOTH_STUMBLEDEATH1\0".as_ptr() as _, value: 41 }, //# Stumble forward and fall face first death
	StringIDTable { name: b"BOTH_FALLDEATH1\0".as_ptr() as _, value: 42 }, //# Fall forward off a high cliff and splat death - start
	StringIDTable { name: b"BOTH_FALLDEATH1INAIR\0".as_ptr() as _, value: 43 }, //# Fall forward off a high cliff and splat death - loop
	StringIDTable { name: b"BOTH_FALLDEATH1LAND\0".as_ptr() as _, value: 44 }, //# Fall forward off a high cliff and splat death - hit bottom
	StringIDTable { name: b"BOTH_DEATH_ROLL\0".as_ptr() as _, value: 45 }, //# Death anim from a roll
	StringIDTable { name: b"BOTH_DEATH_FLIP\0".as_ptr() as _, value: 46 }, //# Death anim from a flip
	StringIDTable { name: b"BOTH_DEATH_SPIN_90_R\0".as_ptr() as _, value: 47 }, //# Death anim when facing 90 degrees right
	StringIDTable { name: b"BOTH_DEATH_SPIN_90_L\0".as_ptr() as _, value: 48 }, //# Death anim when facing 90 degrees left
	StringIDTable { name: b"BOTH_DEATH_SPIN_180\0".as_ptr() as _, value: 49 }, //# Death anim when facing backwards
	StringIDTable { name: b"BOTH_DEATH_LYING_UP\0".as_ptr() as _, value: 50 }, //# Death anim when lying on back
	StringIDTable { name: b"BOTH_DEATH_LYING_DN\0".as_ptr() as _, value: 51 }, //# Death anim when lying on front
	StringIDTable { name: b"BOTH_DEATH_FALLING_DN\0".as_ptr() as _, value: 52 }, //# Death anim when falling on face
	StringIDTable { name: b"BOTH_DEATH_FALLING_UP\0".as_ptr() as _, value: 53 }, //# Death anim when falling on back
	StringIDTable { name: b"BOTH_DEATH_CROUCHED\0".as_ptr() as _, value: 54 }, //# Death anim when crouched
	//# #sep ENUM2STRING(BOTH_ DEAD POSES # Should be last frame of corresponding previous anims
	StringIDTable { name: b"BOTH_DEAD1\0".as_ptr() as _, value: 55 }, //# First Death finished pose
	StringIDTable { name: b"BOTH_DEAD2\0".as_ptr() as _, value: 56 }, //# Second Death finished pose
	StringIDTable { name: b"BOTH_DEAD3\0".as_ptr() as _, value: 57 }, //# Third Death finished pose
	StringIDTable { name: b"BOTH_DEAD4\0".as_ptr() as _, value: 58 }, //# Fourth Death finished pose
	StringIDTable { name: b"BOTH_DEAD5\0".as_ptr() as _, value: 59 }, //# Fifth Death finished pose
	StringIDTable { name: b"BOTH_DEAD6\0".as_ptr() as _, value: 60 }, //# Sixth Death finished pose
	StringIDTable { name: b"BOTH_DEAD7\0".as_ptr() as _, value: 61 }, //# Seventh Death finished pose
	StringIDTable { name: b"BOTH_DEAD8\0".as_ptr() as _, value: 62 }, //# 
	StringIDTable { name: b"BOTH_DEAD9\0".as_ptr() as _, value: 63 }, //# 
	StringIDTable { name: b"BOTH_DEAD10\0".as_ptr() as _, value: 64 }, //# 
	StringIDTable { name: b"BOTH_DEAD11\0".as_ptr() as _, value: 65 }, //#
	StringIDTable { name: b"BOTH_DEAD12\0".as_ptr() as _, value: 66 }, //# 
	StringIDTable { name: b"BOTH_DEAD13\0".as_ptr() as _, value: 67 }, //# 
	StringIDTable { name: b"BOTH_DEAD14\0".as_ptr() as _, value: 68 }, //# 
	StringIDTable { name: b"BOTH_DEAD15\0".as_ptr() as _, value: 69 }, //# 
	StringIDTable { name: b"BOTH_DEAD16\0".as_ptr() as _, value: 70 }, //# 
	StringIDTable { name: b"BOTH_DEAD17\0".as_ptr() as _, value: 71 }, //# 
	StringIDTable { name: b"BOTH_DEAD18\0".as_ptr() as _, value: 72 }, //# 
	StringIDTable { name: b"BOTH_DEAD19\0".as_ptr() as _, value: 73 }, //# 
	StringIDTable { name: b"BOTH_DEAD20\0".as_ptr() as _, value: 74 }, //# 
	StringIDTable { name: b"BOTH_DEAD21\0".as_ptr() as _, value: 75 }, //# 
	StringIDTable { name: b"BOTH_DEAD22\0".as_ptr() as _, value: 76 }, //# 
	StringIDTable { name: b"BOTH_DEAD23\0".as_ptr() as _, value: 77 }, //# 
	StringIDTable { name: b"BOTH_DEAD24\0".as_ptr() as _, value: 78 }, //# 
	StringIDTable { name: b"BOTH_DEAD25\0".as_ptr() as _, value: 79 }, //# 
	StringIDTable { name: b"BOTH_DEADFORWARD1\0".as_ptr() as _, value: 80 }, //# First thrown forward death finished pose
	StringIDTable { name: b"BOTH_DEADFORWARD2\0".as_ptr() as _, value: 81 }, //# Second thrown forward death finished pose
	StringIDTable { name: b"BOTH_DEADBACKWARD1\0".as_ptr() as _, value: 82 }, //# First thrown backward death finished pose
	StringIDTable { name: b"BOTH_DEADBACKWARD2\0".as_ptr() as _, value: 83 }, //# Second thrown backward death finished pose
	StringIDTable { name: b"BOTH_LYINGDEAD1\0".as_ptr() as _, value: 84 }, //# Killed lying down death finished pose
	StringIDTable { name: b"BOTH_STUMBLEDEAD1\0".as_ptr() as _, value: 85 }, //# Stumble forward death finished pose
	StringIDTable { name: b"BOTH_FALLDEAD1LAND\0".as_ptr() as _, value: 86 }, //# Fall forward and splat death finished pose
	//# #sep ENUM2STRING(BOTH_ DEAD TWITCH/FLOP # React to being shot from death poses
	StringIDTable { name: b"BOTH_DEADFLOP1\0".as_ptr() as _, value: 87 }, //# React to being shot from First Death finished pose
	StringIDTable { name: b"BOTH_DEADFLOP2\0".as_ptr() as _, value: 88 }, //# React to being shot from Second Death finished pose
	StringIDTable { name: b"BOTH_DISMEMBER_HEAD1\0".as_ptr() as _, value: 89 }, //#
	StringIDTable { name: b"BOTH_DISMEMBER_TORSO1\0".as_ptr() as _, value: 90 }, //#
	StringIDTable { name: b"BOTH_DISMEMBER_LLEG\0".as_ptr() as _, value: 91 }, //#
	StringIDTable { name: b"BOTH_DISMEMBER_RLEG\0".as_ptr() as _, value: 92 }, //#
	StringIDTable { name: b"BOTH_DISMEMBER_RARM\0".as_ptr() as _, value: 93 }, //#
	StringIDTable { name: b"BOTH_DISMEMBER_LARM\0".as_ptr() as _, value: 94 }, //#
	//# #sep ENUM2STRING(BOTH_ PAINS
	StringIDTable { name: b"BOTH_PAIN1\0".as_ptr() as _, value: 95 }, //# First take pain anim
	StringIDTable { name: b"BOTH_PAIN2\0".as_ptr() as _, value: 96 }, //# Second take pain anim
	StringIDTable { name: b"BOTH_PAIN3\0".as_ptr() as _, value: 97 }, //# Third take pain anim
	StringIDTable { name: b"BOTH_PAIN4\0".as_ptr() as _, value: 98 }, //# Fourth take pain anim
	StringIDTable { name: b"BOTH_PAIN5\0".as_ptr() as _, value: 99 }, //# Fifth take pain anim - from behind
	StringIDTable { name: b"BOTH_PAIN6\0".as_ptr() as _, value: 100 }, //# Sixth take pain anim - from behind
	StringIDTable { name: b"BOTH_PAIN7\0".as_ptr() as _, value: 101 }, //# Seventh take pain anim - from behind
	StringIDTable { name: b"BOTH_PAIN8\0".as_ptr() as _, value: 102 }, //# Eigth take pain anim - from behind
	StringIDTable { name: b"BOTH_PAIN9\0".as_ptr() as _, value: 103 }, //# 
	StringIDTable { name: b"BOTH_PAIN10\0".as_ptr() as _, value: 104 }, //# 
	StringIDTable { name: b"BOTH_PAIN11\0".as_ptr() as _, value: 105 }, //# 
	StringIDTable { name: b"BOTH_PAIN12\0".as_ptr() as _, value: 106 }, //# 
	StringIDTable { name: b"BOTH_PAIN13\0".as_ptr() as _, value: 107 }, //# 
	StringIDTable { name: b"BOTH_PAIN14\0".as_ptr() as _, value: 108 }, //# 
	StringIDTable { name: b"BOTH_PAIN15\0".as_ptr() as _, value: 109 }, //# 
	StringIDTable { name: b"BOTH_PAIN16\0".as_ptr() as _, value: 110 }, //# 
	StringIDTable { name: b"BOTH_PAIN17\0".as_ptr() as _, value: 111 }, //# 
	StringIDTable { name: b"BOTH_PAIN18\0".as_ptr() as _, value: 112 }, //# 

	//# #sep ENUM2STRING(BOTH_ ATTACKS
	StringIDTable { name: b"BOTH_ATTACK1\0".as_ptr() as _, value: 113 }, //# Attack with stun baton
	StringIDTable { name: b"BOTH_ATTACK2\0".as_ptr() as _, value: 114 }, //# Attack with one-handed pistol
	StringIDTable { name: b"BOTH_ATTACK3\0".as_ptr() as _, value: 115 }, //# Attack with blaster rifle
	StringIDTable { name: b"BOTH_ATTACK4\0".as_ptr() as _, value: 116 }, //# Attack with disruptor
	StringIDTable { name: b"BOTH_ATTACK5\0".as_ptr() as _, value: 117 }, //# Another Rancor Attack
	StringIDTable { name: b"BOTH_ATTACK6\0".as_ptr() as _, value: 118 }, //# Yet Another Rancor Attack
	StringIDTable { name: b"BOTH_ATTACK7\0".as_ptr() as _, value: 119 }, //# Yet Another Rancor Attack
	StringIDTable { name: b"BOTH_ATTACK10\0".as_ptr() as _, value: 120 }, //# Attack with thermal det
	StringIDTable { name: b"BOTH_ATTACK11\0".as_ptr() as _, value: 121 }, //# "Attack" with tripmine and detpack
	StringIDTable { name: b"BOTH_MELEE1\0".as_ptr() as _, value: 122 }, //# First melee attack
	StringIDTable { name: b"BOTH_MELEE2\0".as_ptr() as _, value: 123 }, //# Second melee attack
	StringIDTable { name: b"BOTH_THERMAL_READY\0".as_ptr() as _, value: 124 }, //# pull back with thermal
	StringIDTable { name: b"BOTH_THERMAL_THROW\0".as_ptr() as _, value: 125 }, //# throw thermal
	//* #sep ENUM2STRING(BOTH_ SABER ANIMS
	//Saber attack anims - power level 1
	StringIDTable { name: b"BOTH_A1_T__B_\0".as_ptr() as _, value: 126 }, //# Fast weak vertical attack top to bottom
	StringIDTable { name: b"BOTH_A1__L__R\0".as_ptr() as _, value: 127 }, //# Fast weak horizontal attack left to right
	StringIDTable { name: b"BOTH_A1__R__L\0".as_ptr() as _, value: 128 }, //# Fast weak horizontal attack right to left
	StringIDTable { name: b"BOTH_A1_TL_BR\0".as_ptr() as _, value: 129 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A1_BR_TL\0".as_ptr() as _, value: 130 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A1_BL_TR\0".as_ptr() as _, value: 131 }, //# Fast weak diagonal attack bottom left to top right
	StringIDTable { name: b"BOTH_A1_TR_BL\0".as_ptr() as _, value: 132 }, //# Fast weak diagonal attack bottom left to right
	//Saber Arc and Spin Transitions
	StringIDTable { name: b"BOTH_T1_BR__R\0".as_ptr() as _, value: 133 }, //# Fast arc bottom right to right
	StringIDTable { name: b"BOTH_T1_BR_TL\0".as_ptr() as _, value: 134 }, //# Fast weak spin bottom right to top left
	StringIDTable { name: b"BOTH_T1_BR__L\0".as_ptr() as _, value: 135 }, //# Fast weak spin bottom right to left
	StringIDTable { name: b"BOTH_T1_BR_BL\0".as_ptr() as _, value: 136 }, //# Fast weak spin bottom right to bottom left
	StringIDTable { name: b"BOTH_T1__R_TR\0".as_ptr() as _, value: 137 }, //# Fast arc right to top right
	StringIDTable { name: b"BOTH_T1__R_TL\0".as_ptr() as _, value: 138 }, //# Fast arc right to top left
	StringIDTable { name: b"BOTH_T1__R__L\0".as_ptr() as _, value: 139 }, //# Fast weak spin right to left
	StringIDTable { name: b"BOTH_T1__R_BL\0".as_ptr() as _, value: 140 }, //# Fast weak spin right to bottom left
	StringIDTable { name: b"BOTH_T1_TR_BR\0".as_ptr() as _, value: 141 }, //# Fast arc top right to bottom right
	StringIDTable { name: b"BOTH_T1_TR_TL\0".as_ptr() as _, value: 142 }, //# Fast arc top right to top left
	StringIDTable { name: b"BOTH_T1_TR__L\0".as_ptr() as _, value: 143 }, //# Fast arc top right to left
	StringIDTable { name: b"BOTH_T1_TR_BL\0".as_ptr() as _, value: 144 }, //# Fast weak spin top right to bottom left
	StringIDTable { name: b"BOTH_T1_T__BR\0".as_ptr() as _, value: 145 }, //# Fast arc top to bottom right
	StringIDTable { name: b"BOTH_T1_T___R\0".as_ptr() as _, value: 146 }, //# Fast arc top to right
	StringIDTable { name: b"BOTH_T1_T__TR\0".as_ptr() as _, value: 147 }, //# Fast arc top to top right
	StringIDTable { name: b"BOTH_T1_T__TL\0".as_ptr() as _, value: 148 }, //# Fast arc top to top left
	StringIDTable { name: b"BOTH_T1_T___L\0".as_ptr() as _, value: 149 }, //# Fast arc top to left
	StringIDTable { name: b"BOTH_T1_T__BL\0".as_ptr() as _, value: 150 }, //# Fast arc top to bottom left
	StringIDTable { name: b"BOTH_T1_TL_BR\0".as_ptr() as _, value: 151 }, //# Fast weak spin top left to bottom right
	StringIDTable { name: b"BOTH_T1_TL_BL\0".as_ptr() as _, value: 152 }, //# Fast arc top left to bottom left
	StringIDTable { name: b"BOTH_T1__L_BR\0".as_ptr() as _, value: 153 }, //# Fast weak spin left to bottom right
	StringIDTable { name: b"BOTH_T1__L__R\0".as_ptr() as _, value: 154 }, //# Fast weak spin left to right
	StringIDTable { name: b"BOTH_T1__L_TL\0".as_ptr() as _, value: 155 }, //# Fast arc left to top left
	StringIDTable { name: b"BOTH_T1_BL_BR\0".as_ptr() as _, value: 156 }, //# Fast weak spin bottom left to bottom right
	StringIDTable { name: b"BOTH_T1_BL__R\0".as_ptr() as _, value: 157 }, //# Fast weak spin bottom left to right
	StringIDTable { name: b"BOTH_T1_BL_TR\0".as_ptr() as _, value: 158 }, //# Fast weak spin bottom left to top right
	StringIDTable { name: b"BOTH_T1_BL__L\0".as_ptr() as _, value: 159 }, //# Fast arc bottom left to left
	//Saber Arc Transitions that use existing animations played backwards
	StringIDTable { name: b"BOTH_T1_BR_TR\0".as_ptr() as _, value: 160 }, //# Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T1_TR_BR)
	StringIDTable { name: b"BOTH_T1_BR_T_\0".as_ptr() as _, value: 161 }, //# Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T1_T__BR)
	StringIDTable { name: b"BOTH_T1__R_BR\0".as_ptr() as _, value: 162 }, //# Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T1_BR__R)
	StringIDTable { name: b"BOTH_T1__R_T_\0".as_ptr() as _, value: 163 }, //# Fast ar right to top				(use: ENUM2STRING(BOTH_T1_T___R)
	StringIDTable { name: b"BOTH_T1_TR__R\0".as_ptr() as _, value: 164 }, //# Fast arc top right to right			(use: ENUM2STRING(BOTH_T1__R_TR)
	StringIDTable { name: b"BOTH_T1_TR_T_\0".as_ptr() as _, value: 165 }, //# Fast arc top right to top				(use: ENUM2STRING(BOTH_T1_T__TR)
	StringIDTable { name: b"BOTH_T1_TL__R\0".as_ptr() as _, value: 166 }, //# Fast arc top left to right			(use: ENUM2STRING(BOTH_T1__R_TL)
	StringIDTable { name: b"BOTH_T1_TL_TR\0".as_ptr() as _, value: 167 }, //# Fast arc top left to top right			(use: ENUM2STRING(BOTH_T1_TR_TL)
	StringIDTable { name: b"BOTH_T1_TL_T_\0".as_ptr() as _, value: 168 }, //# Fast arc top left to top				(use: ENUM2STRING(BOTH_T1_T__TL)
	StringIDTable { name: b"BOTH_T1_TL__L\0".as_ptr() as _, value: 169 }, //# Fast arc top left to left				(use: ENUM2STRING(BOTH_T1__L_TL)
	StringIDTable { name: b"BOTH_T1__L_TR\0".as_ptr() as _, value: 170 }, //# Fast arc left to top right			(use: ENUM2STRING(BOTH_T1_TR__L)
	StringIDTable { name: b"BOTH_T1__L_T_\0".as_ptr() as _, value: 171 }, //# Fast arc left to top				(use: ENUM2STRING(BOTH_T1_T___L)
	StringIDTable { name: b"BOTH_T1__L_BL\0".as_ptr() as _, value: 172 }, //# Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T1_BL__L)
	StringIDTable { name: b"BOTH_T1_BL_T_\0".as_ptr() as _, value: 173 }, //# Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T1_T__BL)
	StringIDTable { name: b"BOTH_T1_BL_TL\0".as_ptr() as _, value: 174 }, //# Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T1_TL_BL)
	//Saber Attack Start Transitions
	StringIDTable { name: b"BOTH_S1_S1_T_\0".as_ptr() as _, value: 175 }, //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
	StringIDTable { name: b"BOTH_S1_S1__L\0".as_ptr() as _, value: 176 }, //# Fast plain transition from stance1 to left-to-right Fast weak attack
	StringIDTable { name: b"BOTH_S1_S1__R\0".as_ptr() as _, value: 177 }, //# Fast plain transition from stance1 to right-to-left Fast weak attack
	StringIDTable { name: b"BOTH_S1_S1_TL\0".as_ptr() as _, value: 178 }, //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
	StringIDTable { name: b"BOTH_S1_S1_BR\0".as_ptr() as _, value: 179 }, //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
	StringIDTable { name: b"BOTH_S1_S1_BL\0".as_ptr() as _, value: 180 }, //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
	StringIDTable { name: b"BOTH_S1_S1_TR\0".as_ptr() as _, value: 181 }, //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
	//Saber Attack Return Transitions
	StringIDTable { name: b"BOTH_R1_B__S1\0".as_ptr() as _, value: 182 }, //# Fast plain transition from top-to-bottom Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R1__L_S1\0".as_ptr() as _, value: 183 }, //# Fast plain transition from left-to-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R1__R_S1\0".as_ptr() as _, value: 184 }, //# Fast plain transition from right-to-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R1_TL_S1\0".as_ptr() as _, value: 185 }, //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R1_BR_S1\0".as_ptr() as _, value: 186 }, //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R1_BL_S1\0".as_ptr() as _, value: 187 }, //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R1_TR_S1\0".as_ptr() as _, value: 188 }, //# Fast plain transition from top-right-to-bottom-left Fast weak attack
	//Saber Attack Bounces (first 4 frames of an attack), played backwards)
	StringIDTable { name: b"BOTH_B1_BR___\0".as_ptr() as _, value: 189 }, //# Bounce-back if attack from BR is blocked
	StringIDTable { name: b"BOTH_B1__R___\0".as_ptr() as _, value: 190 }, //# Bounce-back if attack from R is blocked
	StringIDTable { name: b"BOTH_B1_TR___\0".as_ptr() as _, value: 191 }, //# Bounce-back if attack from TR is blocked
	StringIDTable { name: b"BOTH_B1_T____\0".as_ptr() as _, value: 192 }, //# Bounce-back if attack from T is blocked
	StringIDTable { name: b"BOTH_B1_TL___\0".as_ptr() as _, value: 193 }, //# Bounce-back if attack from TL is blocked
	StringIDTable { name: b"BOTH_B1__L___\0".as_ptr() as _, value: 194 }, //# Bounce-back if attack from L is blocked
	StringIDTable { name: b"BOTH_B1_BL___\0".as_ptr() as _, value: 195 }, //# Bounce-back if attack from BL is blocked
	//Saber Attack Deflections (last 4 frames of an attack)
	StringIDTable { name: b"BOTH_D1_BR___\0".as_ptr() as _, value: 196 }, //# Deflection toward BR
	StringIDTable { name: b"BOTH_D1__R___\0".as_ptr() as _, value: 197 }, //# Deflection toward R
	StringIDTable { name: b"BOTH_D1_TR___\0".as_ptr() as _, value: 198 }, //# Deflection toward TR
	StringIDTable { name: b"BOTH_D1_TL___\0".as_ptr() as _, value: 199 }, //# Deflection toward TL
	StringIDTable { name: b"BOTH_D1__L___\0".as_ptr() as _, value: 200 }, //# Deflection toward L
	StringIDTable { name: b"BOTH_D1_BL___\0".as_ptr() as _, value: 201 }, //# Deflection toward BL
	StringIDTable { name: b"BOTH_D1_B____\0".as_ptr() as _, value: 202 }, //# Deflection toward B
	//Saber attack anims - power level 2
	StringIDTable { name: b"BOTH_A2_T__B_\0".as_ptr() as _, value: 203 }, //# Fast weak vertical attack top to bottom
	StringIDTable { name: b"BOTH_A2__L__R\0".as_ptr() as _, value: 204 }, //# Fast weak horizontal attack left to right
	StringIDTable { name: b"BOTH_A2__R__L\0".as_ptr() as _, value: 205 }, //# Fast weak horizontal attack right to left
	StringIDTable { name: b"BOTH_A2_TL_BR\0".as_ptr() as _, value: 206 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A2_BR_TL\0".as_ptr() as _, value: 207 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A2_BL_TR\0".as_ptr() as _, value: 208 }, //# Fast weak diagonal attack bottom left to top right
	StringIDTable { name: b"BOTH_A2_TR_BL\0".as_ptr() as _, value: 209 }, //# Fast weak diagonal attack bottom left to right
	//Saber Arc and Spin Transitions
	StringIDTable { name: b"BOTH_T2_BR__R\0".as_ptr() as _, value: 210 }, //# Fast arc bottom right to right
	StringIDTable { name: b"BOTH_T2_BR_TL\0".as_ptr() as _, value: 211 }, //# Fast weak spin bottom right to top left
	StringIDTable { name: b"BOTH_T2_BR__L\0".as_ptr() as _, value: 212 }, //# Fast weak spin bottom right to left
	StringIDTable { name: b"BOTH_T2_BR_BL\0".as_ptr() as _, value: 213 }, //# Fast weak spin bottom right to bottom left
	StringIDTable { name: b"BOTH_T2__R_TR\0".as_ptr() as _, value: 214 }, //# Fast arc right to top right
	StringIDTable { name: b"BOTH_T2__R_TL\0".as_ptr() as _, value: 215 }, //# Fast arc right to top left
	StringIDTable { name: b"BOTH_T2__R__L\0".as_ptr() as _, value: 216 }, //# Fast weak spin right to left
	StringIDTable { name: b"BOTH_T2__R_BL\0".as_ptr() as _, value: 217 }, //# Fast weak spin right to bottom left
	StringIDTable { name: b"BOTH_T2_TR_BR\0".as_ptr() as _, value: 218 }, //# Fast arc top right to bottom right
	StringIDTable { name: b"BOTH_T2_TR_TL\0".as_ptr() as _, value: 219 }, //# Fast arc top right to top left
	StringIDTable { name: b"BOTH_T2_TR__L\0".as_ptr() as _, value: 220 }, //# Fast arc top right to left
	StringIDTable { name: b"BOTH_T2_TR_BL\0".as_ptr() as _, value: 221 }, //# Fast weak spin top right to bottom left
	StringIDTable { name: b"BOTH_T2_T__BR\0".as_ptr() as _, value: 222 }, //# Fast arc top to bottom right
	StringIDTable { name: b"BOTH_T2_T___R\0".as_ptr() as _, value: 223 }, //# Fast arc top to right
	StringIDTable { name: b"BOTH_T2_T__TR\0".as_ptr() as _, value: 224 }, //# Fast arc top to top right
	StringIDTable { name: b"BOTH_T2_T__TL\0".as_ptr() as _, value: 225 }, //# Fast arc top to top left
	StringIDTable { name: b"BOTH_T2_T___L\0".as_ptr() as _, value: 226 }, //# Fast arc top to left
	StringIDTable { name: b"BOTH_T2_T__BL\0".as_ptr() as _, value: 227 }, //# Fast arc top to bottom left
	StringIDTable { name: b"BOTH_T2_TL_BR\0".as_ptr() as _, value: 228 }, //# Fast weak spin top left to bottom right
	StringIDTable { name: b"BOTH_T2_TL_BL\0".as_ptr() as _, value: 229 }, //# Fast arc top left to bottom left
	StringIDTable { name: b"BOTH_T2__L_BR\0".as_ptr() as _, value: 230 }, //# Fast weak spin left to bottom right
	StringIDTable { name: b"BOTH_T2__L__R\0".as_ptr() as _, value: 231 }, //# Fast weak spin left to right
	StringIDTable { name: b"BOTH_T2__L_TL\0".as_ptr() as _, value: 232 }, //# Fast arc left to top left
	StringIDTable { name: b"BOTH_T2_BL_BR\0".as_ptr() as _, value: 233 }, //# Fast weak spin bottom left to bottom right
	StringIDTable { name: b"BOTH_T2_BL__R\0".as_ptr() as _, value: 234 }, //# Fast weak spin bottom left to right
	StringIDTable { name: b"BOTH_T2_BL_TR\0".as_ptr() as _, value: 235 }, //# Fast weak spin bottom left to top right
	StringIDTable { name: b"BOTH_T2_BL__L\0".as_ptr() as _, value: 236 }, //# Fast arc bottom left to left
	//Saber Arc Transitions that use existing animations played backwards
	StringIDTable { name: b"BOTH_T2_BR_TR\0".as_ptr() as _, value: 237 }, //# Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T2_TR_BR)
	StringIDTable { name: b"BOTH_T2_BR_T_\0".as_ptr() as _, value: 238 }, //# Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T2_T__BR)
	StringIDTable { name: b"BOTH_T2__R_BR\0".as_ptr() as _, value: 239 }, //# Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T2_BR__R)
	StringIDTable { name: b"BOTH_T2__R_T_\0".as_ptr() as _, value: 240 }, //# Fast ar right to top				(use: ENUM2STRING(BOTH_T2_T___R)
	StringIDTable { name: b"BOTH_T2_TR__R\0".as_ptr() as _, value: 241 }, //# Fast arc top right to right			(use: ENUM2STRING(BOTH_T2__R_TR)
	StringIDTable { name: b"BOTH_T2_TR_T_\0".as_ptr() as _, value: 242 }, //# Fast arc top right to top				(use: ENUM2STRING(BOTH_T2_T__TR)
	StringIDTable { name: b"BOTH_T2_TL__R\0".as_ptr() as _, value: 243 }, //# Fast arc top left to right			(use: ENUM2STRING(BOTH_T2__R_TL)
	StringIDTable { name: b"BOTH_T2_TL_TR\0".as_ptr() as _, value: 244 }, //# Fast arc top left to top right			(use: ENUM2STRING(BOTH_T2_TR_TL)
	StringIDTable { name: b"BOTH_T2_TL_T_\0".as_ptr() as _, value: 245 }, //# Fast arc top left to top				(use: ENUM2STRING(BOTH_T2_T__TL)
	StringIDTable { name: b"BOTH_T2_TL__L\0".as_ptr() as _, value: 246 }, //# Fast arc top left to left				(use: ENUM2STRING(BOTH_T2__L_TL)
	StringIDTable { name: b"BOTH_T2__L_TR\0".as_ptr() as _, value: 247 }, //# Fast arc left to top right			(use: ENUM2STRING(BOTH_T2_TR__L)
	StringIDTable { name: b"BOTH_T2__L_T_\0".as_ptr() as _, value: 248 }, //# Fast arc left to top				(use: ENUM2STRING(BOTH_T2_T___L)
	StringIDTable { name: b"BOTH_T2__L_BL\0".as_ptr() as _, value: 249 }, //# Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T2_BL__L)
	StringIDTable { name: b"BOTH_T2_BL_T_\0".as_ptr() as _, value: 250 }, //# Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T2_T__BL)
	StringIDTable { name: b"BOTH_T2_BL_TL\0".as_ptr() as _, value: 251 }, //# Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T2_TL_BL)
	//Saber Attack Start Transitions
	StringIDTable { name: b"BOTH_S2_S1_T_\0".as_ptr() as _, value: 252 }, //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
	StringIDTable { name: b"BOTH_S2_S1__L\0".as_ptr() as _, value: 253 }, //# Fast plain transition from stance1 to left-to-right Fast weak attack
	StringIDTable { name: b"BOTH_S2_S1__R\0".as_ptr() as _, value: 254 }, //# Fast plain transition from stance1 to right-to-left Fast weak attack
	StringIDTable { name: b"BOTH_S2_S1_TL\0".as_ptr() as _, value: 255 }, //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
	StringIDTable { name: b"BOTH_S2_S1_BR\0".as_ptr() as _, value: 256 }, //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
	StringIDTable { name: b"BOTH_S2_S1_BL\0".as_ptr() as _, value: 257 }, //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
	StringIDTable { name: b"BOTH_S2_S1_TR\0".as_ptr() as _, value: 258 }, //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
	//Saber Attack Return Transitions
	StringIDTable { name: b"BOTH_R2_B__S1\0".as_ptr() as _, value: 259 }, //# Fast plain transition from top-to-bottom Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R2__L_S1\0".as_ptr() as _, value: 260 }, //# Fast plain transition from left-to-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R2__R_S1\0".as_ptr() as _, value: 261 }, //# Fast plain transition from right-to-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R2_TL_S1\0".as_ptr() as _, value: 262 }, //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R2_BR_S1\0".as_ptr() as _, value: 263 }, //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R2_BL_S1\0".as_ptr() as _, value: 264 }, //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R2_TR_S1\0".as_ptr() as _, value: 265 }, //# Fast plain transition from top-right-to-bottom-left Fast weak attack
	//Saber Attack Bounces (first 4 frames of an attack), played backwards)
	StringIDTable { name: b"BOTH_B2_BR___\0".as_ptr() as _, value: 266 }, //# Bounce-back if attack from BR is blocked
	StringIDTable { name: b"BOTH_B2__R___\0".as_ptr() as _, value: 267 }, //# Bounce-back if attack from R is blocked
	StringIDTable { name: b"BOTH_B2_TR___\0".as_ptr() as _, value: 268 }, //# Bounce-back if attack from TR is blocked
	StringIDTable { name: b"BOTH_B2_T____\0".as_ptr() as _, value: 269 }, //# Bounce-back if attack from T is blocked
	StringIDTable { name: b"BOTH_B2_TL___\0".as_ptr() as _, value: 270 }, //# Bounce-back if attack from TL is blocked
	StringIDTable { name: b"BOTH_B2__L___\0".as_ptr() as _, value: 271 }, //# Bounce-back if attack from L is blocked
	StringIDTable { name: b"BOTH_B2_BL___\0".as_ptr() as _, value: 272 }, //# Bounce-back if attack from BL is blocked
	//Saber Attack Deflections (last 4 frames of an attack)
	StringIDTable { name: b"BOTH_D2_BR___\0".as_ptr() as _, value: 273 }, //# Deflection toward BR
	StringIDTable { name: b"BOTH_D2__R___\0".as_ptr() as _, value: 274 }, //# Deflection toward R
	StringIDTable { name: b"BOTH_D2_TR___\0".as_ptr() as _, value: 275 }, //# Deflection toward TR
	StringIDTable { name: b"BOTH_D2_TL___\0".as_ptr() as _, value: 276 }, //# Deflection toward TL
	StringIDTable { name: b"BOTH_D2__L___\0".as_ptr() as _, value: 277 }, //# Deflection toward L
	StringIDTable { name: b"BOTH_D2_BL___\0".as_ptr() as _, value: 278 }, //# Deflection toward BL
	StringIDTable { name: b"BOTH_D2_B____\0".as_ptr() as _, value: 279 }, //# Deflection toward B
	//Saber attack anims - power level 3
	StringIDTable { name: b"BOTH_A3_T__B_\0".as_ptr() as _, value: 280 }, //# Fast weak vertical attack top to bottom
	StringIDTable { name: b"BOTH_A3__L__R\0".as_ptr() as _, value: 281 }, //# Fast weak horizontal attack left to right
	StringIDTable { name: b"BOTH_A3__R__L\0".as_ptr() as _, value: 282 }, //# Fast weak horizontal attack right to left
	StringIDTable { name: b"BOTH_A3_TL_BR\0".as_ptr() as _, value: 283 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A3_BR_TL\0".as_ptr() as _, value: 284 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A3_BL_TR\0".as_ptr() as _, value: 285 }, //# Fast weak diagonal attack bottom left to top right
	StringIDTable { name: b"BOTH_A3_TR_BL\0".as_ptr() as _, value: 286 }, //# Fast weak diagonal attack bottom left to right
	//Saber Arc and Spin Transitions
	StringIDTable { name: b"BOTH_T3_BR__R\0".as_ptr() as _, value: 287 }, //# Fast arc bottom right to right
	StringIDTable { name: b"BOTH_T3_BR_TL\0".as_ptr() as _, value: 288 }, //# Fast weak spin bottom right to top left
	StringIDTable { name: b"BOTH_T3_BR__L\0".as_ptr() as _, value: 289 }, //# Fast weak spin bottom right to left
	StringIDTable { name: b"BOTH_T3_BR_BL\0".as_ptr() as _, value: 290 }, //# Fast weak spin bottom right to bottom left
	StringIDTable { name: b"BOTH_T3__R_TR\0".as_ptr() as _, value: 291 }, //# Fast arc right to top right
	StringIDTable { name: b"BOTH_T3__R_TL\0".as_ptr() as _, value: 292 }, //# Fast arc right to top left
	StringIDTable { name: b"BOTH_T3__R__L\0".as_ptr() as _, value: 293 }, //# Fast weak spin right to left
	StringIDTable { name: b"BOTH_T3__R_BL\0".as_ptr() as _, value: 294 }, //# Fast weak spin right to bottom left
	StringIDTable { name: b"BOTH_T3_TR_BR\0".as_ptr() as _, value: 295 }, //# Fast arc top right to bottom right
	StringIDTable { name: b"BOTH_T3_TR_TL\0".as_ptr() as _, value: 296 }, //# Fast arc top right to top left
	StringIDTable { name: b"BOTH_T3_TR__L\0".as_ptr() as _, value: 297 }, //# Fast arc top right to left
	StringIDTable { name: b"BOTH_T3_TR_BL\0".as_ptr() as _, value: 298 }, //# Fast weak spin top right to bottom left
	StringIDTable { name: b"BOTH_T3_T__BR\0".as_ptr() as _, value: 299 }, //# Fast arc top to bottom right
	StringIDTable { name: b"BOTH_T3_T___R\0".as_ptr() as _, value: 300 }, //# Fast arc top to right
	StringIDTable { name: b"BOTH_T3_T__TR\0".as_ptr() as _, value: 301 }, //# Fast arc top to top right
	StringIDTable { name: b"BOTH_T3_T__TL\0".as_ptr() as _, value: 302 }, //# Fast arc top to top left
	StringIDTable { name: b"BOTH_T3_T___L\0".as_ptr() as _, value: 303 }, //# Fast arc top to left
	StringIDTable { name: b"BOTH_T3_T__BL\0".as_ptr() as _, value: 304 }, //# Fast arc top to bottom left
	StringIDTable { name: b"BOTH_T3_TL_BR\0".as_ptr() as _, value: 305 }, //# Fast weak spin top left to bottom right
	StringIDTable { name: b"BOTH_T3_TL_BL\0".as_ptr() as _, value: 306 }, //# Fast arc top left to bottom left
	StringIDTable { name: b"BOTH_T3__L_BR\0".as_ptr() as _, value: 307 }, //# Fast weak spin left to bottom right
	StringIDTable { name: b"BOTH_T3__L__R\0".as_ptr() as _, value: 308 }, //# Fast weak spin left to right
	StringIDTable { name: b"BOTH_T3__L_TL\0".as_ptr() as _, value: 309 }, //# Fast arc left to top left
	StringIDTable { name: b"BOTH_T3_BL_BR\0".as_ptr() as _, value: 310 }, //# Fast weak spin bottom left to bottom right
	StringIDTable { name: b"BOTH_T3_BL__R\0".as_ptr() as _, value: 311 }, //# Fast weak spin bottom left to right
	StringIDTable { name: b"BOTH_T3_BL_TR\0".as_ptr() as _, value: 312 }, //# Fast weak spin bottom left to top right
	StringIDTable { name: b"BOTH_T3_BL__L\0".as_ptr() as _, value: 313 }, //# Fast arc bottom left to left
	//Saber Arc Transitions that use existing animations played backwards
	StringIDTable { name: b"BOTH_T3_BR_TR\0".as_ptr() as _, value: 314 }, //# Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T3_TR_BR)
	StringIDTable { name: b"BOTH_T3_BR_T_\0".as_ptr() as _, value: 315 }, //# Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T3_T__BR)
	StringIDTable { name: b"BOTH_T3__R_BR\0".as_ptr() as _, value: 316 }, //# Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T3_BR__R)
	StringIDTable { name: b"BOTH_T3__R_T_\0".as_ptr() as _, value: 317 }, //# Fast ar right to top				(use: ENUM2STRING(BOTH_T3_T___R)
	StringIDTable { name: b"BOTH_T3_TR__R\0".as_ptr() as _, value: 318 }, //# Fast arc top right to right			(use: ENUM2STRING(BOTH_T3__R_TR)
	StringIDTable { name: b"BOTH_T3_TR_T_\0".as_ptr() as _, value: 319 }, //# Fast arc top right to top				(use: ENUM2STRING(BOTH_T3_T__TR)
	StringIDTable { name: b"BOTH_T3_TL__R\0".as_ptr() as _, value: 320 }, //# Fast arc top left to right			(use: ENUM2STRING(BOTH_T3__R_TL)
	StringIDTable { name: b"BOTH_T3_TL_TR\0".as_ptr() as _, value: 321 }, //# Fast arc top left to top right			(use: ENUM2STRING(BOTH_T3_TR_TL)
	StringIDTable { name: b"BOTH_T3_TL_T_\0".as_ptr() as _, value: 322 }, //# Fast arc top left to top				(use: ENUM2STRING(BOTH_T3_T__TL)
	StringIDTable { name: b"BOTH_T3_TL__L\0".as_ptr() as _, value: 323 }, //# Fast arc top left to left				(use: ENUM2STRING(BOTH_T3__L_TL)
	StringIDTable { name: b"BOTH_T3__L_TR\0".as_ptr() as _, value: 324 }, //# Fast arc left to top right			(use: ENUM2STRING(BOTH_T3_TR__L)
	StringIDTable { name: b"BOTH_T3__L_T_\0".as_ptr() as _, value: 325 }, //# Fast arc left to top				(use: ENUM2STRING(BOTH_T3_T___L)
	StringIDTable { name: b"BOTH_T3__L_BL\0".as_ptr() as _, value: 326 }, //# Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T3_BL__L)
	StringIDTable { name: b"BOTH_T3_BL_T_\0".as_ptr() as _, value: 327 }, //# Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T3_T__BL)
	StringIDTable { name: b"BOTH_T3_BL_TL\0".as_ptr() as _, value: 328 }, //# Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T3_TL_BL)
	//Saber Attack Start Transitions
	StringIDTable { name: b"BOTH_S3_S1_T_\0".as_ptr() as _, value: 329 }, //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
	StringIDTable { name: b"BOTH_S3_S1__L\0".as_ptr() as _, value: 330 }, //# Fast plain transition from stance1 to left-to-right Fast weak attack
	StringIDTable { name: b"BOTH_S3_S1__R\0".as_ptr() as _, value: 331 }, //# Fast plain transition from stance1 to right-to-left Fast weak attack
	StringIDTable { name: b"BOTH_S3_S1_TL\0".as_ptr() as _, value: 332 }, //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
	StringIDTable { name: b"BOTH_S3_S1_BR\0".as_ptr() as _, value: 333 }, //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
	StringIDTable { name: b"BOTH_S3_S1_BL\0".as_ptr() as _, value: 334 }, //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
	StringIDTable { name: b"BOTH_S3_S1_TR\0".as_ptr() as _, value: 335 }, //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
	//Saber Attack Return Transitions
	StringIDTable { name: b"BOTH_R3_B__S1\0".as_ptr() as _, value: 336 }, //# Fast plain transition from top-to-bottom Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R3__L_S1\0".as_ptr() as _, value: 337 }, //# Fast plain transition from left-to-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R3__R_S1\0".as_ptr() as _, value: 338 }, //# Fast plain transition from right-to-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R3_TL_S1\0".as_ptr() as _, value: 339 }, //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R3_BR_S1\0".as_ptr() as _, value: 340 }, //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R3_BL_S1\0".as_ptr() as _, value: 341 }, //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R3_TR_S1\0".as_ptr() as _, value: 342 }, //# Fast plain transition from top-right-to-bottom-left Fast weak attack
	//Saber Attack Bounces (first 4 frames of an attack), played backwards)
	StringIDTable { name: b"BOTH_B3_BR___\0".as_ptr() as _, value: 343 }, //# Bounce-back if attack from BR is blocked
	StringIDTable { name: b"BOTH_B3__R___\0".as_ptr() as _, value: 344 }, //# Bounce-back if attack from R is blocked
	StringIDTable { name: b"BOTH_B3_TR___\0".as_ptr() as _, value: 345 }, //# Bounce-back if attack from TR is blocked
	StringIDTable { name: b"BOTH_B3_T____\0".as_ptr() as _, value: 346 }, //# Bounce-back if attack from T is blocked
	StringIDTable { name: b"BOTH_B3_TL___\0".as_ptr() as _, value: 347 }, //# Bounce-back if attack from TL is blocked
	StringIDTable { name: b"BOTH_B3__L___\0".as_ptr() as _, value: 348 }, //# Bounce-back if attack from L is blocked
	StringIDTable { name: b"BOTH_B3_BL___\0".as_ptr() as _, value: 349 }, //# Bounce-back if attack from BL is blocked
	//Saber Attack Deflections (last 4 frames of an attack)
	StringIDTable { name: b"BOTH_D3_BR___\0".as_ptr() as _, value: 350 }, //# Deflection toward BR
	StringIDTable { name: b"BOTH_D3__R___\0".as_ptr() as _, value: 351 }, //# Deflection toward R
	StringIDTable { name: b"BOTH_D3_TR___\0".as_ptr() as _, value: 352 }, //# Deflection toward TR
	StringIDTable { name: b"BOTH_D3_TL___\0".as_ptr() as _, value: 353 }, //# Deflection toward TL
	StringIDTable { name: b"BOTH_D3__L___\0".as_ptr() as _, value: 354 }, //# Deflection toward L
	StringIDTable { name: b"BOTH_D3_BL___\0".as_ptr() as _, value: 355 }, //# Deflection toward BL
	StringIDTable { name: b"BOTH_D3_B____\0".as_ptr() as _, value: 356 }, //# Deflection toward B
	//Saber attack anims - power level 4 - Desann's
	StringIDTable { name: b"BOTH_A4_T__B_\0".as_ptr() as _, value: 357 }, //# Fast weak vertical attack top to bottom
	StringIDTable { name: b"BOTH_A4__L__R\0".as_ptr() as _, value: 358 }, //# Fast weak horizontal attack left to right
	StringIDTable { name: b"BOTH_A4__R__L\0".as_ptr() as _, value: 359 }, //# Fast weak horizontal attack right to left
	StringIDTable { name: b"BOTH_A4_TL_BR\0".as_ptr() as _, value: 360 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A4_BR_TL\0".as_ptr() as _, value: 361 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A4_BL_TR\0".as_ptr() as _, value: 362 }, //# Fast weak diagonal attack bottom left to top right
	StringIDTable { name: b"BOTH_A4_TR_BL\0".as_ptr() as _, value: 363 }, //# Fast weak diagonal attack bottom left to right
	//Saber Arc and Spin Transitions
	StringIDTable { name: b"BOTH_T4_BR__R\0".as_ptr() as _, value: 364 }, //# Fast arc bottom right to right
	StringIDTable { name: b"BOTH_T4_BR_TL\0".as_ptr() as _, value: 365 }, //# Fast weak spin bottom right to top left
	StringIDTable { name: b"BOTH_T4_BR__L\0".as_ptr() as _, value: 366 }, //# Fast weak spin bottom right to left
	StringIDTable { name: b"BOTH_T4_BR_BL\0".as_ptr() as _, value: 367 }, //# Fast weak spin bottom right to bottom left
	StringIDTable { name: b"BOTH_T4__R_TR\0".as_ptr() as _, value: 368 }, //# Fast arc right to top right
	StringIDTable { name: b"BOTH_T4__R_TL\0".as_ptr() as _, value: 369 }, //# Fast arc right to top left
	StringIDTable { name: b"BOTH_T4__R__L\0".as_ptr() as _, value: 370 }, //# Fast weak spin right to left
	StringIDTable { name: b"BOTH_T4__R_BL\0".as_ptr() as _, value: 371 }, //# Fast weak spin right to bottom left
	StringIDTable { name: b"BOTH_T4_TR_BR\0".as_ptr() as _, value: 372 }, //# Fast arc top right to bottom right
	StringIDTable { name: b"BOTH_T4_TR_TL\0".as_ptr() as _, value: 373 }, //# Fast arc top right to top left
	StringIDTable { name: b"BOTH_T4_TR__L\0".as_ptr() as _, value: 374 }, //# Fast arc top right to left
	StringIDTable { name: b"BOTH_T4_TR_BL\0".as_ptr() as _, value: 375 }, //# Fast weak spin top right to bottom left
	StringIDTable { name: b"BOTH_T4_T__BR\0".as_ptr() as _, value: 376 }, //# Fast arc top to bottom right
	StringIDTable { name: b"BOTH_T4_T___R\0".as_ptr() as _, value: 377 }, //# Fast arc top to right
	StringIDTable { name: b"BOTH_T4_T__TR\0".as_ptr() as _, value: 378 }, //# Fast arc top to top right
	StringIDTable { name: b"BOTH_T4_T__TL\0".as_ptr() as _, value: 379 }, //# Fast arc top to top left
	StringIDTable { name: b"BOTH_T4_T___L\0".as_ptr() as _, value: 380 }, //# Fast arc top to left
	StringIDTable { name: b"BOTH_T4_T__BL\0".as_ptr() as _, value: 381 }, //# Fast arc top to bottom left
	StringIDTable { name: b"BOTH_T4_TL_BR\0".as_ptr() as _, value: 382 }, //# Fast weak spin top left to bottom right
	StringIDTable { name: b"BOTH_T4_TL_BL\0".as_ptr() as _, value: 383 }, //# Fast arc top left to bottom left
	StringIDTable { name: b"BOTH_T4__L_BR\0".as_ptr() as _, value: 384 }, //# Fast weak spin left to bottom right
	StringIDTable { name: b"BOTH_T4__L__R\0".as_ptr() as _, value: 385 }, //# Fast weak spin left to right
	StringIDTable { name: b"BOTH_T4__L_TL\0".as_ptr() as _, value: 386 }, //# Fast arc left to top left
	StringIDTable { name: b"BOTH_T4_BL_BR\0".as_ptr() as _, value: 387 }, //# Fast weak spin bottom left to bottom right
	StringIDTable { name: b"BOTH_T4_BL__R\0".as_ptr() as _, value: 388 }, //# Fast weak spin bottom left to right
	StringIDTable { name: b"BOTH_T4_BL_TR\0".as_ptr() as _, value: 389 }, //# Fast weak spin bottom left to top right
	StringIDTable { name: b"BOTH_T4_BL__L\0".as_ptr() as _, value: 390 }, //# Fast arc bottom left to left
	//Saber Arc Transitions that use existing animations played backwards
	StringIDTable { name: b"BOTH_T4_BR_TR\0".as_ptr() as _, value: 391 }, //# Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T4_TR_BR)
	StringIDTable { name: b"BOTH_T4_BR_T_\0".as_ptr() as _, value: 392 }, //# Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T4_T__BR)
	StringIDTable { name: b"BOTH_T4__R_BR\0".as_ptr() as _, value: 393 }, //# Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T4_BR__R)
	StringIDTable { name: b"BOTH_T4__R_T_\0".as_ptr() as _, value: 394 }, //# Fast ar right to top				(use: ENUM2STRING(BOTH_T4_T___R)
	StringIDTable { name: b"BOTH_T4_TR__R\0".as_ptr() as _, value: 395 }, //# Fast arc top right to right			(use: ENUM2STRING(BOTH_T4__R_TR)
	StringIDTable { name: b"BOTH_T4_TR_T_\0".as_ptr() as _, value: 396 }, //# Fast arc top right to top				(use: ENUM2STRING(BOTH_T4_T__TR)
	StringIDTable { name: b"BOTH_T4_TL__R\0".as_ptr() as _, value: 397 }, //# Fast arc top left to right			(use: ENUM2STRING(BOTH_T4__R_TL)
	StringIDTable { name: b"BOTH_T4_TL_TR\0".as_ptr() as _, value: 398 }, //# Fast arc top left to top right			(use: ENUM2STRING(BOTH_T4_TR_TL)
	StringIDTable { name: b"BOTH_T4_TL_T_\0".as_ptr() as _, value: 399 }, //# Fast arc top left to top				(use: ENUM2STRING(BOTH_T4_T__TL)
	StringIDTable { name: b"BOTH_T4_TL__L\0".as_ptr() as _, value: 400 }, //# Fast arc top left to left				(use: ENUM2STRING(BOTH_T4__L_TL)
	StringIDTable { name: b"BOTH_T4__L_TR\0".as_ptr() as _, value: 401 }, //# Fast arc left to top right			(use: ENUM2STRING(BOTH_T4_TR__L)
	StringIDTable { name: b"BOTH_T4__L_T_\0".as_ptr() as _, value: 402 }, //# Fast arc left to top				(use: ENUM2STRING(BOTH_T4_T___L)
	StringIDTable { name: b"BOTH_T4__L_BL\0".as_ptr() as _, value: 403 }, //# Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T4_BL__L)
	StringIDTable { name: b"BOTH_T4_BL_T_\0".as_ptr() as _, value: 404 }, //# Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T4_T__BL)
	StringIDTable { name: b"BOTH_T4_BL_TL\0".as_ptr() as _, value: 405 }, //# Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T4_TL_BL)
	//Saber Attack Start Transitions
	StringIDTable { name: b"BOTH_S4_S1_T_\0".as_ptr() as _, value: 406 }, //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
	StringIDTable { name: b"BOTH_S4_S1__L\0".as_ptr() as _, value: 407 }, //# Fast plain transition from stance1 to left-to-right Fast weak attack
	StringIDTable { name: b"BOTH_S4_S1__R\0".as_ptr() as _, value: 408 }, //# Fast plain transition from stance1 to right-to-left Fast weak attack
	StringIDTable { name: b"BOTH_S4_S1_TL\0".as_ptr() as _, value: 409 }, //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
	StringIDTable { name: b"BOTH_S4_S1_BR\0".as_ptr() as _, value: 410 }, //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
	StringIDTable { name: b"BOTH_S4_S1_BL\0".as_ptr() as _, value: 411 }, //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
	StringIDTable { name: b"BOTH_S4_S1_TR\0".as_ptr() as _, value: 412 }, //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
	//Saber Attack Return Transitions
	StringIDTable { name: b"BOTH_R4_B__S1\0".as_ptr() as _, value: 413 }, //# Fast plain transition from top-to-bottom Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R4__L_S1\0".as_ptr() as _, value: 414 }, //# Fast plain transition from left-to-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R4__R_S1\0".as_ptr() as _, value: 415 }, //# Fast plain transition from right-to-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R4_TL_S1\0".as_ptr() as _, value: 416 }, //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R4_BR_S1\0".as_ptr() as _, value: 417 }, //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R4_BL_S1\0".as_ptr() as _, value: 418 }, //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R4_TR_S1\0".as_ptr() as _, value: 419 }, //# Fast plain transition from top-right-to-bottom-left Fast weak attack
	//Saber Attack Bounces (first 4 frames of an attack), played backwards)
	StringIDTable { name: b"BOTH_B4_BR___\0".as_ptr() as _, value: 420 }, //# Bounce-back if attack from BR is blocked
	StringIDTable { name: b"BOTH_B4__R___\0".as_ptr() as _, value: 421 }, //# Bounce-back if attack from R is blocked
	StringIDTable { name: b"BOTH_B4_TR___\0".as_ptr() as _, value: 422 }, //# Bounce-back if attack from TR is blocked
	StringIDTable { name: b"BOTH_B4_T____\0".as_ptr() as _, value: 423 }, //# Bounce-back if attack from T is blocked
	StringIDTable { name: b"BOTH_B4_TL___\0".as_ptr() as _, value: 424 }, //# Bounce-back if attack from TL is blocked
	StringIDTable { name: b"BOTH_B4__L___\0".as_ptr() as _, value: 425 }, //# Bounce-back if attack from L is blocked
	StringIDTable { name: b"BOTH_B4_BL___\0".as_ptr() as _, value: 426 }, //# Bounce-back if attack from BL is blocked
	//Saber Attack Deflections (last 4 frames of an attack)
	StringIDTable { name: b"BOTH_D4_BR___\0".as_ptr() as _, value: 427 }, //# Deflection toward BR
	StringIDTable { name: b"BOTH_D4__R___\0".as_ptr() as _, value: 428 }, //# Deflection toward R
	StringIDTable { name: b"BOTH_D4_TR___\0".as_ptr() as _, value: 429 }, //# Deflection toward TR
	StringIDTable { name: b"BOTH_D4_TL___\0".as_ptr() as _, value: 430 }, //# Deflection toward TL
	StringIDTable { name: b"BOTH_D4__L___\0".as_ptr() as _, value: 431 }, //# Deflection toward L
	StringIDTable { name: b"BOTH_D4_BL___\0".as_ptr() as _, value: 432 }, //# Deflection toward BL
	StringIDTable { name: b"BOTH_D4_B____\0".as_ptr() as _, value: 433 }, //# Deflection toward B
	//Saber attack anims - power level 5 - Tavion's
	StringIDTable { name: b"BOTH_A5_T__B_\0".as_ptr() as _, value: 434 }, //# Fast weak vertical attack top to bottom
	StringIDTable { name: b"BOTH_A5__L__R\0".as_ptr() as _, value: 435 }, //# Fast weak horizontal attack left to right
	StringIDTable { name: b"BOTH_A5__R__L\0".as_ptr() as _, value: 436 }, //# Fast weak horizontal attack right to left
	StringIDTable { name: b"BOTH_A5_TL_BR\0".as_ptr() as _, value: 437 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A5_BR_TL\0".as_ptr() as _, value: 438 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A5_BL_TR\0".as_ptr() as _, value: 439 }, //# Fast weak diagonal attack bottom left to top right
	StringIDTable { name: b"BOTH_A5_TR_BL\0".as_ptr() as _, value: 440 }, //# Fast weak diagonal attack bottom left to right
	//Saber Arc and Spin Transitions
	StringIDTable { name: b"BOTH_T5_BR__R\0".as_ptr() as _, value: 441 }, //# Fast arc bottom right to right
	StringIDTable { name: b"BOTH_T5_BR_TL\0".as_ptr() as _, value: 442 }, //# Fast weak spin bottom right to top left
	StringIDTable { name: b"BOTH_T5_BR__L\0".as_ptr() as _, value: 443 }, //# Fast weak spin bottom right to left
	StringIDTable { name: b"BOTH_T5_BR_BL\0".as_ptr() as _, value: 444 }, //# Fast weak spin bottom right to bottom left
	StringIDTable { name: b"BOTH_T5__R_TR\0".as_ptr() as _, value: 445 }, //# Fast arc right to top right
	StringIDTable { name: b"BOTH_T5__R_TL\0".as_ptr() as _, value: 446 }, //# Fast arc right to top left
	StringIDTable { name: b"BOTH_T5__R__L\0".as_ptr() as _, value: 447 }, //# Fast weak spin right to left
	StringIDTable { name: b"BOTH_T5__R_BL\0".as_ptr() as _, value: 448 }, //# Fast weak spin right to bottom left
	StringIDTable { name: b"BOTH_T5_TR_BR\0".as_ptr() as _, value: 449 }, //# Fast arc top right to bottom right
	StringIDTable { name: b"BOTH_T5_TR_TL\0".as_ptr() as _, value: 450 }, //# Fast arc top right to top left
	StringIDTable { name: b"BOTH_T5_TR__L\0".as_ptr() as _, value: 451 }, //# Fast arc top right to left
	StringIDTable { name: b"BOTH_T5_TR_BL\0".as_ptr() as _, value: 452 }, //# Fast weak spin top right to bottom left
	StringIDTable { name: b"BOTH_T5_T__BR\0".as_ptr() as _, value: 453 }, //# Fast arc top to bottom right
	StringIDTable { name: b"BOTH_T5_T___R\0".as_ptr() as _, value: 454 }, //# Fast arc top to right
	StringIDTable { name: b"BOTH_T5_T__TR\0".as_ptr() as _, value: 455 }, //# Fast arc top to top right
	StringIDTable { name: b"BOTH_T5_T__TL\0".as_ptr() as _, value: 456 }, //# Fast arc top to top left
	StringIDTable { name: b"BOTH_T5_T___L\0".as_ptr() as _, value: 457 }, //# Fast arc top to left
	StringIDTable { name: b"BOTH_T5_T__BL\0".as_ptr() as _, value: 458 }, //# Fast arc top to bottom left
	StringIDTable { name: b"BOTH_T5_TL_BR\0".as_ptr() as _, value: 459 }, //# Fast weak spin top left to bottom right
	StringIDTable { name: b"BOTH_T5_TL_BL\0".as_ptr() as _, value: 460 }, //# Fast arc top left to bottom left
	StringIDTable { name: b"BOTH_T5__L_BR\0".as_ptr() as _, value: 461 }, //# Fast weak spin left to bottom right
	StringIDTable { name: b"BOTH_T5__L__R\0".as_ptr() as _, value: 462 }, //# Fast weak spin left to right
	StringIDTable { name: b"BOTH_T5__L_TL\0".as_ptr() as _, value: 463 }, //# Fast arc left to top left
	StringIDTable { name: b"BOTH_T5_BL_BR\0".as_ptr() as _, value: 464 }, //# Fast weak spin bottom left to bottom right
	StringIDTable { name: b"BOTH_T5_BL__R\0".as_ptr() as _, value: 465 }, //# Fast weak spin bottom left to right
	StringIDTable { name: b"BOTH_T5_BL_TR\0".as_ptr() as _, value: 466 }, //# Fast weak spin bottom left to top right
	StringIDTable { name: b"BOTH_T5_BL__L\0".as_ptr() as _, value: 467 }, //# Fast arc bottom left to left
	//Saber Arc Transitions that use existing animations played backwards
	StringIDTable { name: b"BOTH_T5_BR_TR\0".as_ptr() as _, value: 468 }, //# Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T5_TR_BR)
	StringIDTable { name: b"BOTH_T5_BR_T_\0".as_ptr() as _, value: 469 }, //# Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T5_T__BR)
	StringIDTable { name: b"BOTH_T5__R_BR\0".as_ptr() as _, value: 470 }, //# Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T5_BR__R)
	StringIDTable { name: b"BOTH_T5__R_T_\0".as_ptr() as _, value: 471 }, //# Fast ar right to top				(use: ENUM2STRING(BOTH_T5_T___R)
	StringIDTable { name: b"BOTH_T5_TR__R\0".as_ptr() as _, value: 472 }, //# Fast arc top right to right			(use: ENUM2STRING(BOTH_T5__R_TR)
	StringIDTable { name: b"BOTH_T5_TR_T_\0".as_ptr() as _, value: 473 }, //# Fast arc top right to top				(use: ENUM2STRING(BOTH_T5_T__TR)
	StringIDTable { name: b"BOTH_T5_TL__R\0".as_ptr() as _, value: 474 }, //# Fast arc top left to right			(use: ENUM2STRING(BOTH_T5__R_TL)
	StringIDTable { name: b"BOTH_T5_TL_TR\0".as_ptr() as _, value: 475 }, //# Fast arc top left to top right			(use: ENUM2STRING(BOTH_T5_TR_TL)
	StringIDTable { name: b"BOTH_T5_TL_T_\0".as_ptr() as _, value: 476 }, //# Fast arc top left to top				(use: ENUM2STRING(BOTH_T5_T__TL)
	StringIDTable { name: b"BOTH_T5_TL__L\0".as_ptr() as _, value: 477 }, //# Fast arc top left to left				(use: ENUM2STRING(BOTH_T5__L_TL)
	StringIDTable { name: b"BOTH_T5__L_TR\0".as_ptr() as _, value: 478 }, //# Fast arc left to top right			(use: ENUM2STRING(BOTH_T5_TR__L)
	StringIDTable { name: b"BOTH_T5__L_T_\0".as_ptr() as _, value: 479 }, //# Fast arc left to top				(use: ENUM2STRING(BOTH_T5_T___L)
	StringIDTable { name: b"BOTH_T5__L_BL\0".as_ptr() as _, value: 480 }, //# Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T5_BL__L)
	StringIDTable { name: b"BOTH_T5_BL_T_\0".as_ptr() as _, value: 481 }, //# Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T5_T__BL)
	StringIDTable { name: b"BOTH_T5_BL_TL\0".as_ptr() as _, value: 482 }, //# Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T5_TL_BL)
	//Saber Attack Start Transitions
	StringIDTable { name: b"BOTH_S5_S1_T_\0".as_ptr() as _, value: 483 }, //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
	StringIDTable { name: b"BOTH_S5_S1__L\0".as_ptr() as _, value: 484 }, //# Fast plain transition from stance1 to left-to-right Fast weak attack
	StringIDTable { name: b"BOTH_S5_S1__R\0".as_ptr() as _, value: 485 }, //# Fast plain transition from stance1 to right-to-left Fast weak attack
	StringIDTable { name: b"BOTH_S5_S1_TL\0".as_ptr() as _, value: 486 }, //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
	StringIDTable { name: b"BOTH_S5_S1_BR\0".as_ptr() as _, value: 487 }, //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
	StringIDTable { name: b"BOTH_S5_S1_BL\0".as_ptr() as _, value: 488 }, //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
	StringIDTable { name: b"BOTH_S5_S1_TR\0".as_ptr() as _, value: 489 }, //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
	//Saber Attack Return Transitions
	StringIDTable { name: b"BOTH_R5_B__S1\0".as_ptr() as _, value: 490 }, //# Fast plain transition from top-to-bottom Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R5__L_S1\0".as_ptr() as _, value: 491 }, //# Fast plain transition from left-to-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R5__R_S1\0".as_ptr() as _, value: 492 }, //# Fast plain transition from right-to-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R5_TL_S1\0".as_ptr() as _, value: 493 }, //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R5_BR_S1\0".as_ptr() as _, value: 494 }, //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R5_BL_S1\0".as_ptr() as _, value: 495 }, //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R5_TR_S1\0".as_ptr() as _, value: 496 }, //# Fast plain transition from top-right-to-bottom-left Fast weak attack
	//Saber Attack Bounces (first 4 frames of an attack), played backwards)
	StringIDTable { name: b"BOTH_B5_BR___\0".as_ptr() as _, value: 497 }, //# Bounce-back if attack from BR is blocked
	StringIDTable { name: b"BOTH_B5__R___\0".as_ptr() as _, value: 498 }, //# Bounce-back if attack from R is blocked
	StringIDTable { name: b"BOTH_B5_TR___\0".as_ptr() as _, value: 499 }, //# Bounce-back if attack from TR is blocked
	StringIDTable { name: b"BOTH_B5_T____\0".as_ptr() as _, value: 500 }, //# Bounce-back if attack from T is blocked
	StringIDTable { name: b"BOTH_B5_TL___\0".as_ptr() as _, value: 501 }, //# Bounce-back if attack from TL is blocked
	StringIDTable { name: b"BOTH_B5__L___\0".as_ptr() as _, value: 502 }, //# Bounce-back if attack from L is blocked
	StringIDTable { name: b"BOTH_B5_BL___\0".as_ptr() as _, value: 503 }, //# Bounce-back if attack from BL is blocked
	//Saber Attack Deflections (last 4 frames of an attack)
	StringIDTable { name: b"BOTH_D5_BR___\0".as_ptr() as _, value: 504 }, //# Deflection toward BR
	StringIDTable { name: b"BOTH_D5__R___\0".as_ptr() as _, value: 505 }, //# Deflection toward R
	StringIDTable { name: b"BOTH_D5_TR___\0".as_ptr() as _, value: 506 }, //# Deflection toward TR
	StringIDTable { name: b"BOTH_D5_TL___\0".as_ptr() as _, value: 507 }, //# Deflection toward TL
	StringIDTable { name: b"BOTH_D5__L___\0".as_ptr() as _, value: 508 }, //# Deflection toward L
	StringIDTable { name: b"BOTH_D5_BL___\0".as_ptr() as _, value: 509 }, //# Deflection toward BL
	StringIDTable { name: b"BOTH_D5_B____\0".as_ptr() as _, value: 510 }, //# Deflection toward B
	//Saber attack anims - power level 6
	StringIDTable { name: b"BOTH_A6_T__B_\0".as_ptr() as _, value: 511 }, //# Fast weak vertical attack top to bottom
	StringIDTable { name: b"BOTH_A6__L__R\0".as_ptr() as _, value: 512 }, //# Fast weak horizontal attack left to right
	StringIDTable { name: b"BOTH_A6__R__L\0".as_ptr() as _, value: 513 }, //# Fast weak horizontal attack right to left
	StringIDTable { name: b"BOTH_A6_TL_BR\0".as_ptr() as _, value: 514 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A6_BR_TL\0".as_ptr() as _, value: 515 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A6_BL_TR\0".as_ptr() as _, value: 516 }, //# Fast weak diagonal attack bottom left to top right
	StringIDTable { name: b"BOTH_A6_TR_BL\0".as_ptr() as _, value: 517 }, //# Fast weak diagonal attack bottom left to right
	//Saber Arc and Spin Transitions
	StringIDTable { name: b"BOTH_T6_BR__R\0".as_ptr() as _, value: 518 }, //# Fast arc bottom right to right
	StringIDTable { name: b"BOTH_T6_BR_TL\0".as_ptr() as _, value: 519 }, //# Fast weak spin bottom right to top left
	StringIDTable { name: b"BOTH_T6_BR__L\0".as_ptr() as _, value: 520 }, //# Fast weak spin bottom right to left
	StringIDTable { name: b"BOTH_T6_BR_BL\0".as_ptr() as _, value: 521 }, //# Fast weak spin bottom right to bottom left
	StringIDTable { name: b"BOTH_T6__R_TR\0".as_ptr() as _, value: 522 }, //# Fast arc right to top right
	StringIDTable { name: b"BOTH_T6__R_TL\0".as_ptr() as _, value: 523 }, //# Fast arc right to top left
	StringIDTable { name: b"BOTH_T6__R__L\0".as_ptr() as _, value: 524 }, //# Fast weak spin right to left
	StringIDTable { name: b"BOTH_T6__R_BL\0".as_ptr() as _, value: 525 }, //# Fast weak spin right to bottom left
	StringIDTable { name: b"BOTH_T6_TR_BR\0".as_ptr() as _, value: 526 }, //# Fast arc top right to bottom right
	StringIDTable { name: b"BOTH_T6_TR_TL\0".as_ptr() as _, value: 527 }, //# Fast arc top right to top left
	StringIDTable { name: b"BOTH_T6_TR__L\0".as_ptr() as _, value: 528 }, //# Fast arc top right to left
	StringIDTable { name: b"BOTH_T6_TR_BL\0".as_ptr() as _, value: 529 }, //# Fast weak spin top right to bottom left
	StringIDTable { name: b"BOTH_T6_T__BR\0".as_ptr() as _, value: 530 }, //# Fast arc top to bottom right
	StringIDTable { name: b"BOTH_T6_T___R\0".as_ptr() as _, value: 531 }, //# Fast arc top to right
	StringIDTable { name: b"BOTH_T6_T__TR\0".as_ptr() as _, value: 532 }, //# Fast arc top to top right
	StringIDTable { name: b"BOTH_T6_T__TL\0".as_ptr() as _, value: 533 }, //# Fast arc top to top left
	StringIDTable { name: b"BOTH_T6_T___L\0".as_ptr() as _, value: 534 }, //# Fast arc top to left
	StringIDTable { name: b"BOTH_T6_T__BL\0".as_ptr() as _, value: 535 }, //# Fast arc top to bottom left
	StringIDTable { name: b"BOTH_T6_TL_BR\0".as_ptr() as _, value: 536 }, //# Fast weak spin top left to bottom right
	StringIDTable { name: b"BOTH_T6_TL_BL\0".as_ptr() as _, value: 537 }, //# Fast arc top left to bottom left
	StringIDTable { name: b"BOTH_T6__L_BR\0".as_ptr() as _, value: 538 }, //# Fast weak spin left to bottom right
	StringIDTable { name: b"BOTH_T6__L__R\0".as_ptr() as _, value: 539 }, //# Fast weak spin left to right
	StringIDTable { name: b"BOTH_T6__L_TL\0".as_ptr() as _, value: 540 }, //# Fast arc left to top left
	StringIDTable { name: b"BOTH_T6_BL_BR\0".as_ptr() as _, value: 541 }, //# Fast weak spin bottom left to bottom right
	StringIDTable { name: b"BOTH_T6_BL__R\0".as_ptr() as _, value: 542 }, //# Fast weak spin bottom left to right
	StringIDTable { name: b"BOTH_T6_BL_TR\0".as_ptr() as _, value: 543 }, //# Fast weak spin bottom left to top right
	StringIDTable { name: b"BOTH_T6_BL__L\0".as_ptr() as _, value: 544 }, //# Fast arc bottom left to left
	//Saber Arc Transitions that use existing animations played backwards
	StringIDTable { name: b"BOTH_T6_BR_TR\0".as_ptr() as _, value: 545 }, //# Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T6_TR_BR)
	StringIDTable { name: b"BOTH_T6_BR_T_\0".as_ptr() as _, value: 546 }, //# Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T6_T__BR)
	StringIDTable { name: b"BOTH_T6__R_BR\0".as_ptr() as _, value: 547 }, //# Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T6_BR__R)
	StringIDTable { name: b"BOTH_T6__R_T_\0".as_ptr() as _, value: 548 }, //# Fast ar right to top				(use: ENUM2STRING(BOTH_T6_T___R)
	StringIDTable { name: b"BOTH_T6_TR__R\0".as_ptr() as _, value: 549 }, //# Fast arc top right to right			(use: ENUM2STRING(BOTH_T6__R_TR)
	StringIDTable { name: b"BOTH_T6_TR_T_\0".as_ptr() as _, value: 550 }, //# Fast arc top right to top				(use: ENUM2STRING(BOTH_T6_T__TR)
	StringIDTable { name: b"BOTH_T6_TL__R\0".as_ptr() as _, value: 551 }, //# Fast arc top left to right			(use: ENUM2STRING(BOTH_T6__R_TL)
	StringIDTable { name: b"BOTH_T6_TL_TR\0".as_ptr() as _, value: 552 }, //# Fast arc top left to top right			(use: ENUM2STRING(BOTH_T6_TR_TL)
	StringIDTable { name: b"BOTH_T6_TL_T_\0".as_ptr() as _, value: 553 }, //# Fast arc top left to top				(use: ENUM2STRING(BOTH_T6_T__TL)
	StringIDTable { name: b"BOTH_T6_TL__L\0".as_ptr() as _, value: 554 }, //# Fast arc top left to left				(use: ENUM2STRING(BOTH_T6__L_TL)
	StringIDTable { name: b"BOTH_T6__L_TR\0".as_ptr() as _, value: 555 }, //# Fast arc left to top right			(use: ENUM2STRING(BOTH_T6_TR__L)
	StringIDTable { name: b"BOTH_T6__L_T_\0".as_ptr() as _, value: 556 }, //# Fast arc left to top				(use: ENUM2STRING(BOTH_T6_T___L)
	StringIDTable { name: b"BOTH_T6__L_BL\0".as_ptr() as _, value: 557 }, //# Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T6_BL__L)
	StringIDTable { name: b"BOTH_T6_BL_T_\0".as_ptr() as _, value: 558 }, //# Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T6_T__BL)
	StringIDTable { name: b"BOTH_T6_BL_TL\0".as_ptr() as _, value: 559 }, //# Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T6_TL_BL)
	//Saber Attack Start Transitions
	StringIDTable { name: b"BOTH_S6_S6_T_\0".as_ptr() as _, value: 560 }, //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
	StringIDTable { name: b"BOTH_S6_S6__L\0".as_ptr() as _, value: 561 }, //# Fast plain transition from stance1 to left-to-right Fast weak attack
	StringIDTable { name: b"BOTH_S6_S6__R\0".as_ptr() as _, value: 562 }, //# Fast plain transition from stance1 to right-to-left Fast weak attack
	StringIDTable { name: b"BOTH_S6_S6_TL\0".as_ptr() as _, value: 563 }, //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
	StringIDTable { name: b"BOTH_S6_S6_BR\0".as_ptr() as _, value: 564 }, //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
	StringIDTable { name: b"BOTH_S6_S6_BL\0".as_ptr() as _, value: 565 }, //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
	StringIDTable { name: b"BOTH_S6_S6_TR\0".as_ptr() as _, value: 566 }, //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
	//Saber Attack Return Transitions
	StringIDTable { name: b"BOTH_R6_B__S6\0".as_ptr() as _, value: 567 }, //# Fast plain transition from top-to-bottom Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R6__L_S6\0".as_ptr() as _, value: 568 }, //# Fast plain transition from left-to-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R6__R_S6\0".as_ptr() as _, value: 569 }, //# Fast plain transition from right-to-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R6_TL_S6\0".as_ptr() as _, value: 570 }, //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R6_BR_S6\0".as_ptr() as _, value: 571 }, //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R6_BL_S6\0".as_ptr() as _, value: 572 }, //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R6_TR_S6\0".as_ptr() as _, value: 573 }, //# Fast plain transition from top-right-to-bottom-left Fast weak attack
	//Saber Attack Bounces (first 4 frames of an attack), played backwards)
	StringIDTable { name: b"BOTH_B6_BR___\0".as_ptr() as _, value: 574 }, //# Bounce-back if attack from BR is blocked
	StringIDTable { name: b"BOTH_B6__R___\0".as_ptr() as _, value: 575 }, //# Bounce-back if attack from R is blocked
	StringIDTable { name: b"BOTH_B6_TR___\0".as_ptr() as _, value: 576 }, //# Bounce-back if attack from TR is blocked
	StringIDTable { name: b"BOTH_B6_T____\0".as_ptr() as _, value: 577 }, //# Bounce-back if attack from T is blocked
	StringIDTable { name: b"BOTH_B6_TL___\0".as_ptr() as _, value: 578 }, //# Bounce-back if attack from TL is blocked
	StringIDTable { name: b"BOTH_B6__L___\0".as_ptr() as _, value: 579 }, //# Bounce-back if attack from L is blocked
	StringIDTable { name: b"BOTH_B6_BL___\0".as_ptr() as _, value: 580 }, //# Bounce-back if attack from BL is blocked
	//Saber Attack Deflections (last 4 frames of an attack)
	StringIDTable { name: b"BOTH_D6_BR___\0".as_ptr() as _, value: 581 }, //# Deflection toward BR
	StringIDTable { name: b"BOTH_D6__R___\0".as_ptr() as _, value: 582 }, //# Deflection toward R
	StringIDTable { name: b"BOTH_D6_TR___\0".as_ptr() as _, value: 583 }, //# Deflection toward TR
	StringIDTable { name: b"BOTH_D6_TL___\0".as_ptr() as _, value: 584 }, //# Deflection toward TL
	StringIDTable { name: b"BOTH_D6__L___\0".as_ptr() as _, value: 585 }, //# Deflection toward L
	StringIDTable { name: b"BOTH_D6_BL___\0".as_ptr() as _, value: 586 }, //# Deflection toward BL
	StringIDTable { name: b"BOTH_D6_B____\0".as_ptr() as _, value: 587 }, //# Deflection toward B
	//Saber attack anims - power level 7
	StringIDTable { name: b"BOTH_A7_T__B_\0".as_ptr() as _, value: 588 }, //# Fast weak vertical attack top to bottom
	StringIDTable { name: b"BOTH_A7__L__R\0".as_ptr() as _, value: 589 }, //# Fast weak horizontal attack left to right
	StringIDTable { name: b"BOTH_A7__R__L\0".as_ptr() as _, value: 590 }, //# Fast weak horizontal attack right to left
	StringIDTable { name: b"BOTH_A7_TL_BR\0".as_ptr() as _, value: 591 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A7_BR_TL\0".as_ptr() as _, value: 592 }, //# Fast weak diagonal attack top left to botom right
	StringIDTable { name: b"BOTH_A7_BL_TR\0".as_ptr() as _, value: 593 }, //# Fast weak diagonal attack bottom left to top right
	StringIDTable { name: b"BOTH_A7_TR_BL\0".as_ptr() as _, value: 594 }, //# Fast weak diagonal attack bottom left to right
	//Saber Arc and Spin Transitions
	StringIDTable { name: b"BOTH_T7_BR__R\0".as_ptr() as _, value: 595 }, //# Fast arc bottom right to right
	StringIDTable { name: b"BOTH_T7_BR_TL\0".as_ptr() as _, value: 596 }, //# Fast weak spin bottom right to top left
	StringIDTable { name: b"BOTH_T7_BR__L\0".as_ptr() as _, value: 597 }, //# Fast weak spin bottom right to left
	StringIDTable { name: b"BOTH_T7_BR_BL\0".as_ptr() as _, value: 598 }, //# Fast weak spin bottom right to bottom left
	StringIDTable { name: b"BOTH_T7__R_TR\0".as_ptr() as _, value: 599 }, //# Fast arc right to top right
	StringIDTable { name: b"BOTH_T7__R_TL\0".as_ptr() as _, value: 600 }, //# Fast arc right to top left
	StringIDTable { name: b"BOTH_T7__R__L\0".as_ptr() as _, value: 601 }, //# Fast weak spin right to left
	StringIDTable { name: b"BOTH_T7__R_BL\0".as_ptr() as _, value: 602 }, //# Fast weak spin right to bottom left
	StringIDTable { name: b"BOTH_T7_TR_BR\0".as_ptr() as _, value: 603 }, //# Fast arc top right to bottom right
	StringIDTable { name: b"BOTH_T7_TR_TL\0".as_ptr() as _, value: 604 }, //# Fast arc top right to top left
	StringIDTable { name: b"BOTH_T7_TR__L\0".as_ptr() as _, value: 605 }, //# Fast arc top right to left
	StringIDTable { name: b"BOTH_T7_TR_BL\0".as_ptr() as _, value: 606 }, //# Fast weak spin top right to bottom left
	StringIDTable { name: b"BOTH_T7_T__BR\0".as_ptr() as _, value: 607 }, //# Fast arc top to bottom right
	StringIDTable { name: b"BOTH_T7_T___R\0".as_ptr() as _, value: 608 }, //# Fast arc top to right
	StringIDTable { name: b"BOTH_T7_T__TR\0".as_ptr() as _, value: 609 }, //# Fast arc top to top right
	StringIDTable { name: b"BOTH_T7_T__TL\0".as_ptr() as _, value: 610 }, //# Fast arc top to top left
	StringIDTable { name: b"BOTH_T7_T___L\0".as_ptr() as _, value: 611 }, //# Fast arc top to left
	StringIDTable { name: b"BOTH_T7_T__BL\0".as_ptr() as _, value: 612 }, //# Fast arc top to bottom left
	StringIDTable { name: b"BOTH_T7_TL_BR\0".as_ptr() as _, value: 613 }, //# Fast weak spin top left to bottom right
	StringIDTable { name: b"BOTH_T7_TL_BL\0".as_ptr() as _, value: 614 }, //# Fast arc top left to bottom left
	StringIDTable { name: b"BOTH_T7__L_BR\0".as_ptr() as _, value: 615 }, //# Fast weak spin left to bottom right
	StringIDTable { name: b"BOTH_T7__L__R\0".as_ptr() as _, value: 616 }, //# Fast weak spin left to right
	StringIDTable { name: b"BOTH_T7__L_TL\0".as_ptr() as _, value: 617 }, //# Fast arc left to top left
	StringIDTable { name: b"BOTH_T7_BL_BR\0".as_ptr() as _, value: 618 }, //# Fast weak spin bottom left to bottom right
	StringIDTable { name: b"BOTH_T7_BL__R\0".as_ptr() as _, value: 619 }, //# Fast weak spin bottom left to right
	StringIDTable { name: b"BOTH_T7_BL_TR\0".as_ptr() as _, value: 620 }, //# Fast weak spin bottom left to top right
	StringIDTable { name: b"BOTH_T7_BL__L\0".as_ptr() as _, value: 621 }, //# Fast arc bottom left to left
	//Saber Arc Transitions that use existing animations played backwards
	StringIDTable { name: b"BOTH_T7_BR_TR\0".as_ptr() as _, value: 622 }, //# Fast arc bottom right to top right		(use: ENUM2STRING(BOTH_T7_TR_BR)
	StringIDTable { name: b"BOTH_T7_BR_T_\0".as_ptr() as _, value: 623 }, //# Fast arc bottom right to top			(use: ENUM2STRING(BOTH_T7_T__BR)
	StringIDTable { name: b"BOTH_T7__R_BR\0".as_ptr() as _, value: 624 }, //# Fast arc right to bottom right			(use: ENUM2STRING(BOTH_T7_BR__R)
	StringIDTable { name: b"BOTH_T7__R_T_\0".as_ptr() as _, value: 625 }, //# Fast ar right to top				(use: ENUM2STRING(BOTH_T7_T___R)
	StringIDTable { name: b"BOTH_T7_TR__R\0".as_ptr() as _, value: 626 }, //# Fast arc top right to right			(use: ENUM2STRING(BOTH_T7__R_TR)
	StringIDTable { name: b"BOTH_T7_TR_T_\0".as_ptr() as _, value: 627 }, //# Fast arc top right to top				(use: ENUM2STRING(BOTH_T7_T__TR)
	StringIDTable { name: b"BOTH_T7_TL__R\0".as_ptr() as _, value: 628 }, //# Fast arc top left to right			(use: ENUM2STRING(BOTH_T7__R_TL)
	StringIDTable { name: b"BOTH_T7_TL_TR\0".as_ptr() as _, value: 629 }, //# Fast arc top left to top right			(use: ENUM2STRING(BOTH_T7_TR_TL)
	StringIDTable { name: b"BOTH_T7_TL_T_\0".as_ptr() as _, value: 630 }, //# Fast arc top left to top				(use: ENUM2STRING(BOTH_T7_T__TL)
	StringIDTable { name: b"BOTH_T7_TL__L\0".as_ptr() as _, value: 631 }, //# Fast arc top left to left				(use: ENUM2STRING(BOTH_T7__L_TL)
	StringIDTable { name: b"BOTH_T7__L_TR\0".as_ptr() as _, value: 632 }, //# Fast arc left to top right			(use: ENUM2STRING(BOTH_T7_TR__L)
	StringIDTable { name: b"BOTH_T7__L_T_\0".as_ptr() as _, value: 633 }, //# Fast arc left to top				(use: ENUM2STRING(BOTH_T7_T___L)
	StringIDTable { name: b"BOTH_T7__L_BL\0".as_ptr() as _, value: 634 }, //# Fast arc left to bottom left			(use: ENUM2STRING(BOTH_T7_BL__L)
	StringIDTable { name: b"BOTH_T7_BL_T_\0".as_ptr() as _, value: 635 }, //# Fast arc bottom left to top			(use: ENUM2STRING(BOTH_T7_T__BL)
	StringIDTable { name: b"BOTH_T7_BL_TL\0".as_ptr() as _, value: 636 }, //# Fast arc bottom left to top left		(use: ENUM2STRING(BOTH_T7_TL_BL)
	//Saber Attack Start Transitions
	StringIDTable { name: b"BOTH_S7_S7_T_\0".as_ptr() as _, value: 637 }, //# Fast plain transition from stance1 to top-to-bottom Fast weak attack
	StringIDTable { name: b"BOTH_S7_S7__L\0".as_ptr() as _, value: 638 }, //# Fast plain transition from stance1 to left-to-right Fast weak attack
	StringIDTable { name: b"BOTH_S7_S7__R\0".as_ptr() as _, value: 639 }, //# Fast plain transition from stance1 to right-to-left Fast weak attack
	StringIDTable { name: b"BOTH_S7_S7_TL\0".as_ptr() as _, value: 640 }, //# Fast plain transition from stance1 to top-left-to-bottom-right Fast weak attack
	StringIDTable { name: b"BOTH_S7_S7_BR\0".as_ptr() as _, value: 641 }, //# Fast plain transition from stance1 to bottom-right-to-top-left Fast weak attack
	StringIDTable { name: b"BOTH_S7_S7_BL\0".as_ptr() as _, value: 642 }, //# Fast plain transition from stance1 to bottom-left-to-top-right Fast weak attack
	StringIDTable { name: b"BOTH_S7_S7_TR\0".as_ptr() as _, value: 643 }, //# Fast plain transition from stance1 to top-right-to-bottom-left Fast weak attack
	//Saber Attack Return Transitions
	StringIDTable { name: b"BOTH_R7_B__S7\0".as_ptr() as _, value: 644 }, //# Fast plain transition from top-to-bottom Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R7__L_S7\0".as_ptr() as _, value: 645 }, //# Fast plain transition from left-to-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R7__R_S7\0".as_ptr() as _, value: 646 }, //# Fast plain transition from right-to-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R7_TL_S7\0".as_ptr() as _, value: 647 }, //# Fast plain transition from top-left-to-bottom-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R7_BR_S7\0".as_ptr() as _, value: 648 }, //# Fast plain transition from bottom-right-to-top-left Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R7_BL_S7\0".as_ptr() as _, value: 649 }, //# Fast plain transition from bottom-left-to-top-right Fast weak attack to stance1
	StringIDTable { name: b"BOTH_R7_TR_S7\0".as_ptr() as _, value: 650 }, //# Fast plain transition from top-right-to-bottom-left Fast weak attack
	//Saber Attack Bounces (first 4 frames of an attack), played backwards)
	StringIDTable { name: b"BOTH_B7_BR___\0".as_ptr() as _, value: 651 }, //# Bounce-back if attack from BR is blocked
	StringIDTable { name: b"BOTH_B7__R___\0".as_ptr() as _, value: 652 }, //# Bounce-back if attack from R is blocked
	StringIDTable { name: b"BOTH_B7_TR___\0".as_ptr() as _, value: 653 }, //# Bounce-back if attack from TR is blocked
	StringIDTable { name: b"BOTH_B7_T____\0".as_ptr() as _, value: 654 }, //# Bounce-back if attack from T is blocked
	StringIDTable { name: b"BOTH_B7_TL___\0".as_ptr() as _, value: 655 }, //# Bounce-back if attack from TL is blocked
	StringIDTable { name: b"BOTH_B7__L___\0".as_ptr() as _, value: 656 }, //# Bounce-back if attack from L is blocked
	StringIDTable { name: b"BOTH_B7_BL___\0".as_ptr() as _, value: 657 }, //# Bounce-back if attack from BL is blocked
	//Saber Attack Deflections (last 4 frames of an attack)
	StringIDTable { name: b"BOTH_D7_BR___\0".as_ptr() as _, value: 658 }, //# Deflection toward BR
	StringIDTable { name: b"BOTH_D7__R___\0".as_ptr() as _, value: 659 }, //# Deflection toward R
	StringIDTable { name: b"BOTH_D7_TR___\0".as_ptr() as _, value: 660 }, //# Deflection toward TR
	StringIDTable { name: b"BOTH_D7_TL___\0".as_ptr() as _, value: 661 }, //# Deflection toward TL
	StringIDTable { name: b"BOTH_D7__L___\0".as_ptr() as _, value: 662 }, //# Deflection toward L
	StringIDTable { name: b"BOTH_D7_BL___\0".as_ptr() as _, value: 663 }, //# Deflection toward BL
	StringIDTable { name: b"BOTH_D7_B____\0".as_ptr() as _, value: 664 }, //# Deflection toward B
	//Saber parry anims
	StringIDTable { name: b"BOTH_P1_S1_T_\0".as_ptr() as _, value: 665 }, //# Block shot/saber top
	StringIDTable { name: b"BOTH_P1_S1_TR\0".as_ptr() as _, value: 666 }, //# Block shot/saber top right
	StringIDTable { name: b"BOTH_P1_S1_TL\0".as_ptr() as _, value: 667 }, //# Block shot/saber top left
	StringIDTable { name: b"BOTH_P1_S1_BL\0".as_ptr() as _, value: 668 }, //# Block shot/saber bottom left
	StringIDTable { name: b"BOTH_P1_S1_BR\0".as_ptr() as _, value: 669 }, //# Block shot/saber bottom right
	//Saber knockaway
	StringIDTable { name: b"BOTH_K1_S1_T_\0".as_ptr() as _, value: 670 }, //# knockaway saber top
	StringIDTable { name: b"BOTH_K1_S1_TR\0".as_ptr() as _, value: 671 }, //# knockaway saber top right
	StringIDTable { name: b"BOTH_K1_S1_TL\0".as_ptr() as _, value: 672 }, //# knockaway saber top left
	StringIDTable { name: b"BOTH_K1_S1_BL\0".as_ptr() as _, value: 673 }, //# knockaway saber bottom left
	StringIDTable { name: b"BOTH_K1_S1_B_\0".as_ptr() as _, value: 674 }, //# knockaway saber bottom
	StringIDTable { name: b"BOTH_K1_S1_BR\0".as_ptr() as _, value: 675 }, //# knockaway saber bottom right
	//Saber attack knocked away
	StringIDTable { name: b"BOTH_V1_BR_S1\0".as_ptr() as _, value: 676 }, //# BR attack knocked away
	StringIDTable { name: b"BOTH_V1__R_S1\0".as_ptr() as _, value: 677 }, //# R attack knocked away
	StringIDTable { name: b"BOTH_V1_TR_S1\0".as_ptr() as _, value: 678 }, //# TR attack knocked away
	StringIDTable { name: b"BOTH_V1_T__S1\0".as_ptr() as _, value: 679 }, //# T attack knocked away
	StringIDTable { name: b"BOTH_V1_TL_S1\0".as_ptr() as _, value: 680 }, //# TL attack knocked away
	StringIDTable { name: b"BOTH_V1__L_S1\0".as_ptr() as _, value: 681 }, //# L attack knocked away
	StringIDTable { name: b"BOTH_V1_BL_S1\0".as_ptr() as _, value: 682 }, //# BL attack knocked away
	StringIDTable { name: b"BOTH_V1_B__S1\0".as_ptr() as _, value: 683 }, //# B attack knocked away
	//Saber parry broken
	StringIDTable { name: b"BOTH_H1_S1_T_\0".as_ptr() as _, value: 684 }, //# saber knocked down from top parry
	StringIDTable { name: b"BOTH_H1_S1_TR\0".as_ptr() as _, value: 685 }, //# saber knocked down-left from TR parry
	StringIDTable { name: b"BOTH_H1_S1_TL\0".as_ptr() as _, value: 686 }, //# saber knocked down-right from TL parry
	StringIDTable { name: b"BOTH_H1_S1_BL\0".as_ptr() as _, value: 687 }, //# saber knocked up-right from BL parry
	StringIDTable { name: b"BOTH_H1_S1_B_\0".as_ptr() as _, value: 688 }, //# saber knocked up over head from ready?
	StringIDTable { name: b"BOTH_H1_S1_BR\0".as_ptr() as _, value: 689 }, //# saber knocked up-left from BR parry
	//Dual Sabers parry anims
	StringIDTable { name: b"BOTH_P6_S6_T_\0".as_ptr() as _, value: 690 }, //# Block shot/saber top
	StringIDTable { name: b"BOTH_P6_S6_TR\0".as_ptr() as _, value: 691 }, //# Block shot/saber top right
	StringIDTable { name: b"BOTH_P6_S6_TL\0".as_ptr() as _, value: 692 }, //# Block shot/saber top left
	StringIDTable { name: b"BOTH_P6_S6_BL\0".as_ptr() as _, value: 693 }, //# Block shot/saber bottom left
	StringIDTable { name: b"BOTH_P6_S6_BR\0".as_ptr() as _, value: 694 }, //# Block shot/saber bottom right
	//Dual Sabers knockaway
	StringIDTable { name: b"BOTH_K6_S6_T_\0".as_ptr() as _, value: 695 }, //# knockaway saber top
	StringIDTable { name: b"BOTH_K6_S6_TR\0".as_ptr() as _, value: 696 }, //# knockaway saber top right
	StringIDTable { name: b"BOTH_K6_S6_TL\0".as_ptr() as _, value: 697 }, //# knockaway saber top left
	StringIDTable { name: b"BOTH_K6_S6_BL\0".as_ptr() as _, value: 698 }, //# knockaway saber bottom left
	StringIDTable { name: b"BOTH_K6_S6_B_\0".as_ptr() as _, value: 699 }, //# knockaway saber bottom
	StringIDTable { name: b"BOTH_K6_S6_BR\0".as_ptr() as _, value: 700 }, //# knockaway saber bottom right
	//Dual Sabers attack knocked away
	StringIDTable { name: b"BOTH_V6_BR_S6\0".as_ptr() as _, value: 701 }, //# BR attack knocked away
	StringIDTable { name: b"BOTH_V6__R_S6\0".as_ptr() as _, value: 702 }, //# R attack knocked away
	StringIDTable { name: b"BOTH_V6_TR_S6\0".as_ptr() as _, value: 703 }, //# TR attack knocked away
	StringIDTable { name: b"BOTH_V6_T__S6\0".as_ptr() as _, value: 704 }, //# T attack knocked away
	StringIDTable { name: b"BOTH_V6_TL_S6\0".as_ptr() as _, value: 705 }, //# TL attack knocked away
	StringIDTable { name: b"BOTH_V6__L_S6\0".as_ptr() as _, value: 706 }, //# L attack knocked away
	StringIDTable { name: b"BOTH_V6_BL_S6\0".as_ptr() as _, value: 707 }, //# BL attack knocked away
	StringIDTable { name: b"BOTH_V6_B__S6\0".as_ptr() as _, value: 708 }, //# B attack knocked away
	//Dual Sabers parry broken
	StringIDTable { name: b"BOTH_H6_S6_T_\0".as_ptr() as _, value: 709 }, //# saber knocked down from top parry
	StringIDTable { name: b"BOTH_H6_S6_TR\0".as_ptr() as _, value: 710 }, //# saber knocked down-left from TR parry
	StringIDTable { name: b"BOTH_H6_S6_TL\0".as_ptr() as _, value: 711 }, //# saber knocked down-right from TL parry
	StringIDTable { name: b"BOTH_H6_S6_BL\0".as_ptr() as _, value: 712 }, //# saber knocked up-right from BL parry
	StringIDTable { name: b"BOTH_H6_S6_B_\0".as_ptr() as _, value: 713 }, //# saber knocked up over head from ready?
	StringIDTable { name: b"BOTH_H6_S6_BR\0".as_ptr() as _, value: 714 }, //# saber knocked up-left from BR parry
	//SaberStaff parry anims
	StringIDTable { name: b"BOTH_P7_S7_T_\0".as_ptr() as _, value: 715 }, //# Block shot/saber top
	StringIDTable { name: b"BOTH_P7_S7_TR\0".as_ptr() as _, value: 716 }, //# Block shot/saber top right
	StringIDTable { name: b"BOTH_P7_S7_TL\0".as_ptr() as _, value: 717 }, //# Block shot/saber top left
	StringIDTable { name: b"BOTH_P7_S7_BL\0".as_ptr() as _, value: 718 }, //# Block shot/saber bottom left
	StringIDTable { name: b"BOTH_P7_S7_BR\0".as_ptr() as _, value: 719 }, //# Block shot/saber bottom right
	//SaberStaff knockaway
	StringIDTable { name: b"BOTH_K7_S7_T_\0".as_ptr() as _, value: 720 }, //# knockaway saber top
	StringIDTable { name: b"BOTH_K7_S7_TR\0".as_ptr() as _, value: 721 }, //# knockaway saber top right
	StringIDTable { name: b"BOTH_K7_S7_TL\0".as_ptr() as _, value: 722 }, //# knockaway saber top left
	StringIDTable { name: b"BOTH_K7_S7_BL\0".as_ptr() as _, value: 723 }, //# knockaway saber bottom left
	StringIDTable { name: b"BOTH_K7_S7_B_\0".as_ptr() as _, value: 724 }, //# knockaway saber bottom
	StringIDTable { name: b"BOTH_K7_S7_BR\0".as_ptr() as _, value: 725 }, //# knockaway saber bottom right
	//SaberStaff attack knocked away
	StringIDTable { name: b"BOTH_V7_BR_S7\0".as_ptr() as _, value: 726 }, //# BR attack knocked away
	StringIDTable { name: b"BOTH_V7__R_S7\0".as_ptr() as _, value: 727 }, //# R attack knocked away
	StringIDTable { name: b"BOTH_V7_TR_S7\0".as_ptr() as _, value: 728 }, //# TR attack knocked away
	StringIDTable { name: b"BOTH_V7_T__S7\0".as_ptr() as _, value: 729 }, //# T attack knocked away
	StringIDTable { name: b"BOTH_V7_TL_S7\0".as_ptr() as _, value: 730 }, //# TL attack knocked away
	StringIDTable { name: b"BOTH_V7__L_S7\0".as_ptr() as _, value: 731 }, //# L attack knocked away
	StringIDTable { name: b"BOTH_V7_BL_S7\0".as_ptr() as _, value: 732 }, //# BL attack knocked away
	StringIDTable { name: b"BOTH_V7_B__S7\0".as_ptr() as _, value: 733 }, //# B attack knocked away
	//SaberStaff parry broken
	StringIDTable { name: b"BOTH_H7_S7_T_\0".as_ptr() as _, value: 734 }, //# saber knocked down from top parry
	StringIDTable { name: b"BOTH_H7_S7_TR\0".as_ptr() as _, value: 735 }, //# saber knocked down-left from TR parry
	StringIDTable { name: b"BOTH_H7_S7_TL\0".as_ptr() as _, value: 736 }, //# saber knocked down-right from TL parry
	StringIDTable { name: b"BOTH_H7_S7_BL\0".as_ptr() as _, value: 737 }, //# saber knocked up-right from BL parry
	StringIDTable { name: b"BOTH_H7_S7_B_\0".as_ptr() as _, value: 738 }, //# saber knocked up over head from ready?
	StringIDTable { name: b"BOTH_H7_S7_BR\0".as_ptr() as _, value: 739 }, //# saber knocked up-left from BR parry
	//Sabers locked anims
	//* #sep BOTH_ SABER LOCKED ANIMS
	//BOTH_(DL, S, ST)_(DL, S, ST)_(T, S)_(L, B, SB)_1(_W, _L)
//===Single locks==================================================================
//SINGLE vs. DUAL
	//side locks - I'm using a single and they're using dual
	StringIDTable { name: b"BOTH_LK_S_DL_S_B_1_L\0".as_ptr() as _, value: 740 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_S_DL_S_B_1_W\0".as_ptr() as _, value: 741 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_S_DL_S_L_1\0".as_ptr() as _, value: 742 }, //lock if I'm using single vs. a dual
	StringIDTable { name: b"BOTH_LK_S_DL_S_SB_1_L\0".as_ptr() as _, value: 743 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_S_DL_S_SB_1_W\0".as_ptr() as _, value: 744 }, //super break I won
	//top locks
	StringIDTable { name: b"BOTH_LK_S_DL_T_B_1_L\0".as_ptr() as _, value: 745 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_S_DL_T_B_1_W\0".as_ptr() as _, value: 746 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_S_DL_T_L_1\0".as_ptr() as _, value: 747 }, //lock if I'm using single vs. a dual
	StringIDTable { name: b"BOTH_LK_S_DL_T_SB_1_L\0".as_ptr() as _, value: 748 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_S_DL_T_SB_1_W\0".as_ptr() as _, value: 749 }, //super break I won
//SINGLE vs. STAFF
	//side locks
	StringIDTable { name: b"BOTH_LK_S_ST_S_B_1_L\0".as_ptr() as _, value: 750 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_S_ST_S_B_1_W\0".as_ptr() as _, value: 751 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_S_ST_S_L_1\0".as_ptr() as _, value: 752 }, //lock if I'm using single vs. a staff
	StringIDTable { name: b"BOTH_LK_S_ST_S_SB_1_L\0".as_ptr() as _, value: 753 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_S_ST_S_SB_1_W\0".as_ptr() as _, value: 754 }, //super break I won
	//top locks
	StringIDTable { name: b"BOTH_LK_S_ST_T_B_1_L\0".as_ptr() as _, value: 755 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_S_ST_T_B_1_W\0".as_ptr() as _, value: 756 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_S_ST_T_L_1\0".as_ptr() as _, value: 757 }, //lock if I'm using single vs. a staff
	StringIDTable { name: b"BOTH_LK_S_ST_T_SB_1_L\0".as_ptr() as _, value: 758 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_S_ST_T_SB_1_W\0".as_ptr() as _, value: 759 }, //super break I won
//SINGLE vs. SINGLE
	//side locks
	StringIDTable { name: b"BOTH_LK_S_S_S_B_1_L\0".as_ptr() as _, value: 760 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_S_S_S_B_1_W\0".as_ptr() as _, value: 761 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_S_S_S_L_1\0".as_ptr() as _, value: 762 }, //lock if I'm using single vs. a single and I initiated
	StringIDTable { name: b"BOTH_LK_S_S_S_SB_1_L\0".as_ptr() as _, value: 763 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_S_S_S_SB_1_W\0".as_ptr() as _, value: 764 }, //super break I won
	//top locks
	StringIDTable { name: b"BOTH_LK_S_S_T_B_1_L\0".as_ptr() as _, value: 765 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_S_S_T_B_1_W\0".as_ptr() as _, value: 766 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_S_S_T_L_1\0".as_ptr() as _, value: 767 }, //lock if I'm using single vs. a single and I initiated
	StringIDTable { name: b"BOTH_LK_S_S_T_SB_1_L\0".as_ptr() as _, value: 768 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_S_S_T_SB_1_W\0".as_ptr() as _, value: 769 }, //super break I won
//===Dual Saber locks==================================================================
//DUAL vs. DUAL	
	//side locks
	StringIDTable { name: b"BOTH_LK_DL_DL_S_B_1_L\0".as_ptr() as _, value: 770 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_DL_DL_S_B_1_W\0".as_ptr() as _, value: 771 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_DL_DL_S_L_1\0".as_ptr() as _, value: 772 }, //lock if I'm using dual vs. dual and I initiated
	StringIDTable { name: b"BOTH_LK_DL_DL_S_SB_1_L\0".as_ptr() as _, value: 773 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_DL_DL_S_SB_1_W\0".as_ptr() as _, value: 774 }, //super break I won
	//top locks
	StringIDTable { name: b"BOTH_LK_DL_DL_T_B_1_L\0".as_ptr() as _, value: 775 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_DL_DL_T_B_1_W\0".as_ptr() as _, value: 776 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_DL_DL_T_L_1\0".as_ptr() as _, value: 777 }, //lock if I'm using dual vs. dual and I initiated
	StringIDTable { name: b"BOTH_LK_DL_DL_T_SB_1_L\0".as_ptr() as _, value: 778 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_DL_DL_T_SB_1_W\0".as_ptr() as _, value: 779 }, //super break I won
//DUAL vs. STAFF
	//side locks
	StringIDTable { name: b"BOTH_LK_DL_ST_S_B_1_L\0".as_ptr() as _, value: 780 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_DL_ST_S_B_1_W\0".as_ptr() as _, value: 781 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_DL_ST_S_L_1\0".as_ptr() as _, value: 782 }, //lock if I'm using dual vs. a staff
	StringIDTable { name: b"BOTH_LK_DL_ST_S_SB_1_L\0".as_ptr() as _, value: 783 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_DL_ST_S_SB_1_W\0".as_ptr() as _, value: 784 }, //super break I won
	//top locks
	StringIDTable { name: b"BOTH_LK_DL_ST_T_B_1_L\0".as_ptr() as _, value: 785 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_DL_ST_T_B_1_W\0".as_ptr() as _, value: 786 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_DL_ST_T_L_1\0".as_ptr() as _, value: 787 }, //lock if I'm using dual vs. a staff
	StringIDTable { name: b"BOTH_LK_DL_ST_T_SB_1_L\0".as_ptr() as _, value: 788 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_DL_ST_T_SB_1_W\0".as_ptr() as _, value: 789 }, //super break I won
//DUAL vs. SINGLE
	//side locks
	StringIDTable { name: b"BOTH_LK_DL_S_S_B_1_L\0".as_ptr() as _, value: 790 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_DL_S_S_B_1_W\0".as_ptr() as _, value: 791 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_DL_S_S_L_1\0".as_ptr() as _, value: 792 }, //lock if I'm using dual vs. a single
	StringIDTable { name: b"BOTH_LK_DL_S_S_SB_1_L\0".as_ptr() as _, value: 793 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_DL_S_S_SB_1_W\0".as_ptr() as _, value: 794 }, //super break I won
	//top locks
	StringIDTable { name: b"BOTH_LK_DL_S_T_B_1_L\0".as_ptr() as _, value: 795 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_DL_S_T_B_1_W\0".as_ptr() as _, value: 796 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_DL_S_T_L_1\0".as_ptr() as _, value: 797 }, //lock if I'm using dual vs. a single
	StringIDTable { name: b"BOTH_LK_DL_S_T_SB_1_L\0".as_ptr() as _, value: 798 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_DL_S_T_SB_1_W\0".as_ptr() as _, value: 799 }, //super break I won
//===Saber Staff locks==================================================================
//STAFF vs. DUAL
	//side locks
	StringIDTable { name: b"BOTH_LK_ST_DL_S_B_1_L\0".as_ptr() as _, value: 800 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_ST_DL_S_B_1_W\0".as_ptr() as _, value: 801 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_ST_DL_S_L_1\0".as_ptr() as _, value: 802 }, //lock if I'm using staff vs. dual
	StringIDTable { name: b"BOTH_LK_ST_DL_S_SB_1_L\0".as_ptr() as _, value: 803 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_ST_DL_S_SB_1_W\0".as_ptr() as _, value: 804 }, //super break I won
	//top locks
	StringIDTable { name: b"BOTH_LK_ST_DL_T_B_1_L\0".as_ptr() as _, value: 805 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_ST_DL_T_B_1_W\0".as_ptr() as _, value: 806 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_ST_DL_T_L_1\0".as_ptr() as _, value: 807 }, //lock if I'm using staff vs. dual
	StringIDTable { name: b"BOTH_LK_ST_DL_T_SB_1_L\0".as_ptr() as _, value: 808 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_ST_DL_T_SB_1_W\0".as_ptr() as _, value: 809 }, //super break I won
//STAFF vs. STAFF
	//side locks
	StringIDTable { name: b"BOTH_LK_ST_ST_S_B_1_L\0".as_ptr() as _, value: 810 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_ST_ST_S_B_1_W\0".as_ptr() as _, value: 811 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_ST_ST_S_L_1\0".as_ptr() as _, value: 812 }, //lock if I'm using staff vs. a staff and I initiated
	StringIDTable { name: b"BOTH_LK_ST_ST_S_SB_1_L\0".as_ptr() as _, value: 813 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_ST_ST_S_SB_1_W\0".as_ptr() as _, value: 814 }, //super break I won
	//top locks
	StringIDTable { name: b"BOTH_LK_ST_ST_T_B_1_L\0".as_ptr() as _, value: 815 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_ST_ST_T_B_1_W\0".as_ptr() as _, value: 816 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_ST_ST_T_L_1\0".as_ptr() as _, value: 817 }, //lock if I'm using staff vs. a staff and I initiated
	StringIDTable { name: b"BOTH_LK_ST_ST_T_SB_1_L\0".as_ptr() as _, value: 818 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_ST_ST_T_SB_1_W\0".as_ptr() as _, value: 819 }, //super break I won
//STAFF vs. SINGLE
	//side locks
	StringIDTable { name: b"BOTH_LK_ST_S_S_B_1_L\0".as_ptr() as _, value: 820 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_ST_S_S_B_1_W\0".as_ptr() as _, value: 821 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_ST_S_S_L_1\0".as_ptr() as _, value: 822 }, //lock if I'm using staff vs. a single
	StringIDTable { name: b"BOTH_LK_ST_S_S_SB_1_L\0".as_ptr() as _, value: 823 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_ST_S_S_SB_1_W\0".as_ptr() as _, value: 824 }, //super break I won
	//top locks
	StringIDTable { name: b"BOTH_LK_ST_S_T_B_1_L\0".as_ptr() as _, value: 825 }, //normal break I lost
	StringIDTable { name: b"BOTH_LK_ST_S_T_B_1_W\0".as_ptr() as _, value: 826 }, //normal break I won
	StringIDTable { name: b"BOTH_LK_ST_S_T_L_1\0".as_ptr() as _, value: 827 }, //lock if I'm using staff vs. a single
	StringIDTable { name: b"BOTH_LK_ST_S_T_SB_1_L\0".as_ptr() as _, value: 828 }, //super break I lost
	StringIDTable { name: b"BOTH_LK_ST_S_T_SB_1_W\0".as_ptr() as _, value: 829 }, //super break I won
//Special cases for same saber style vs. each other (won't fit in nice 5-anim size lists above)
	StringIDTable { name: b"BOTH_LK_S_S_S_L_2\0".as_ptr() as _, value: 830 }, //lock if I'm using single vs. a single and other intitiated
	StringIDTable { name: b"BOTH_LK_S_S_T_L_2\0".as_ptr() as _, value: 831 }, //lock if I'm using single vs. a single and other initiated
	StringIDTable { name: b"BOTH_LK_DL_DL_S_L_2\0".as_ptr() as _, value: 832 }, //lock if I'm using dual vs. dual and other initiated
	StringIDTable { name: b"BOTH_LK_DL_DL_T_L_2\0".as_ptr() as _, value: 833 }, //lock if I'm using dual vs. dual and other initiated
	StringIDTable { name: b"BOTH_LK_ST_ST_S_L_2\0".as_ptr() as _, value: 834 }, //lock if I'm using staff vs. a staff and other initiated
	StringIDTable { name: b"BOTH_LK_ST_ST_T_L_2\0".as_ptr() as _, value: 835 }, //lock if I'm using staff vs. a staff and other initiated
//===End Saber locks==================================================================
	StringIDTable { name: b"BOTH_BF2RETURN\0".as_ptr() as _, value: 836 }, //#
	StringIDTable { name: b"BOTH_BF2BREAK\0".as_ptr() as _, value: 837 }, //#
	StringIDTable { name: b"BOTH_BF2LOCK\0".as_ptr() as _, value: 838 }, //#
	StringIDTable { name: b"BOTH_BF1RETURN\0".as_ptr() as _, value: 839 }, //#
	StringIDTable { name: b"BOTH_BF1BREAK\0".as_ptr() as _, value: 840 }, //#
	StringIDTable { name: b"BOTH_BF1LOCK\0".as_ptr() as _, value: 841 }, //#
	StringIDTable { name: b"BOTH_CWCIRCLE_R2__R_S1\0".as_ptr() as _, value: 842 }, //#
	StringIDTable { name: b"BOTH_CCWCIRCLE_R2__L_S1\0".as_ptr() as _, value: 843 }, //#
	StringIDTable { name: b"BOTH_CWCIRCLE_A2__L__R\0".as_ptr() as _, value: 844 }, //#
	StringIDTable { name: b"BOTH_CCWCIRCLE_A2__R__L\0".as_ptr() as _, value: 845 }, //#
	StringIDTable { name: b"BOTH_CWCIRCLEBREAK\0".as_ptr() as _, value: 846 }, //#
	StringIDTable { name: b"BOTH_CCWCIRCLEBREAK\0".as_ptr() as _, value: 847 }, //#
	StringIDTable { name: b"BOTH_CWCIRCLELOCK\0".as_ptr() as _, value: 848 }, //#
	StringIDTable { name: b"BOTH_CCWCIRCLELOCK\0".as_ptr() as _, value: 849 }, //#
	//other saber anims/attacks
	StringIDTable { name: b"BOTH_SABERFAST_STANCE\0".as_ptr() as _, value: 850 }, 
	StringIDTable { name: b"BOTH_SABERSLOW_STANCE\0".as_ptr() as _, value: 851 }, 
	StringIDTable { name: b"BOTH_SABERDUAL_STANCE\0".as_ptr() as _, value: 852 }, 
	StringIDTable { name: b"BOTH_SABERSTAFF_STANCE\0".as_ptr() as _, value: 853 }, 
	StringIDTable { name: b"BOTH_A2_STABBACK1\0".as_ptr() as _, value: 854 }, //# Stab saber backward
	StringIDTable { name: b"BOTH_ATTACK_BACK\0".as_ptr() as _, value: 855 }, //# Swing around backwards and attack
	StringIDTable { name: b"BOTH_JUMPFLIPSLASHDOWN1\0".as_ptr() as _, value: 856 }, //#
	StringIDTable { name: b"BOTH_JUMPFLIPSTABDOWN\0".as_ptr() as _, value: 857 }, //#
	StringIDTable { name: b"BOTH_FORCELEAP2_T__B_\0".as_ptr() as _, value: 858 }, //#
	StringIDTable { name: b"BOTH_LUNGE2_B__T_\0".as_ptr() as _, value: 859 }, //#
	StringIDTable { name: b"BOTH_CROUCHATTACKBACK1\0".as_ptr() as _, value: 860 }, //#
	//New specials for JKA:
	StringIDTable { name: b"BOTH_JUMPATTACK6\0".as_ptr() as _, value: 861 }, //#
	StringIDTable { name: b"BOTH_JUMPATTACK7\0".as_ptr() as _, value: 862 }, //#
	StringIDTable { name: b"BOTH_SPINATTACK6\0".as_ptr() as _, value: 863 }, //#
	StringIDTable { name: b"BOTH_SPINATTACK7\0".as_ptr() as _, value: 864 }, //#
	StringIDTable { name: b"BOTH_S1_S6\0".as_ptr() as _, value: 865 }, //#	From stand1 to saberdual stance - turning on your dual sabers
	StringIDTable { name: b"BOTH_S6_S1\0".as_ptr() as _, value: 866 }, //#	From dualstaff stance to stand1 - turning off your dual sabers
	StringIDTable { name: b"BOTH_S1_S7\0".as_ptr() as _, value: 867 }, //#	From stand1 to saberstaff stance - turning on your saberstaff
	StringIDTable { name: b"BOTH_S7_S1\0".as_ptr() as _, value: 868 }, //#	From saberstaff stance to stand1 - turning off your saberstaff
	StringIDTable { name: b"BOTH_FORCELONGLEAP_START\0".as_ptr() as _, value: 869 }, 
	StringIDTable { name: b"BOTH_FORCELONGLEAP_ATTACK\0".as_ptr() as _, value: 870 }, 
	StringIDTable { name: b"BOTH_FORCELONGLEAP_LAND\0".as_ptr() as _, value: 871 }, 
	StringIDTable { name: b"BOTH_FORCEWALLRUNFLIP_START\0".as_ptr() as _, value: 872 }, 
	StringIDTable { name: b"BOTH_FORCEWALLRUNFLIP_END\0".as_ptr() as _, value: 873 }, 
	StringIDTable { name: b"BOTH_FORCEWALLRUNFLIP_ALT\0".as_ptr() as _, value: 874 }, 
	StringIDTable { name: b"BOTH_FORCEWALLREBOUND_FORWARD\0".as_ptr() as _, value: 875 }, 
	StringIDTable { name: b"BOTH_FORCEWALLREBOUND_LEFT\0".as_ptr() as _, value: 876 }, 
	StringIDTable { name: b"BOTH_FORCEWALLREBOUND_BACK\0".as_ptr() as _, value: 877 }, 
	StringIDTable { name: b"BOTH_FORCEWALLREBOUND_RIGHT\0".as_ptr() as _, value: 878 }, 
	StringIDTable { name: b"BOTH_FORCEWALLHOLD_FORWARD\0".as_ptr() as _, value: 879 }, 
	StringIDTable { name: b"BOTH_FORCEWALLHOLD_LEFT\0".as_ptr() as _, value: 880 }, 
	StringIDTable { name: b"BOTH_FORCEWALLHOLD_BACK\0".as_ptr() as _, value: 881 }, 
	StringIDTable { name: b"BOTH_FORCEWALLHOLD_RIGHT\0".as_ptr() as _, value: 882 }, 
	StringIDTable { name: b"BOTH_FORCEWALLRELEASE_FORWARD\0".as_ptr() as _, value: 883 }, 
	StringIDTable { name: b"BOTH_FORCEWALLRELEASE_LEFT\0".as_ptr() as _, value: 884 }, 
	StringIDTable { name: b"BOTH_FORCEWALLRELEASE_BACK\0".as_ptr() as _, value: 885 }, 
	StringIDTable { name: b"BOTH_FORCEWALLRELEASE_RIGHT\0".as_ptr() as _, value: 886 }, 
	StringIDTable { name: b"BOTH_A7_KICK_F\0".as_ptr() as _, value: 887 }, 
	StringIDTable { name: b"BOTH_A7_KICK_B\0".as_ptr() as _, value: 888 }, 
	StringIDTable { name: b"BOTH_A7_KICK_R\0".as_ptr() as _, value: 889 }, 
	StringIDTable { name: b"BOTH_A7_KICK_L\0".as_ptr() as _, value: 890 }, 
	StringIDTable { name: b"BOTH_A7_KICK_S\0".as_ptr() as _, value: 891 }, 
	StringIDTable { name: b"BOTH_A7_KICK_BF\0".as_ptr() as _, value: 892 }, 
	StringIDTable { name: b"BOTH_A7_KICK_BF_STOP\0".as_ptr() as _, value: 893 }, 
	StringIDTable { name: b"BOTH_A7_KICK_RL\0".as_ptr() as _, value: 894 }, 
	StringIDTable { name: b"BOTH_A7_KICK_F_AIR\0".as_ptr() as _, value: 895 }, 
	StringIDTable { name: b"BOTH_A7_KICK_B_AIR\0".as_ptr() as _, value: 896 }, 
	StringIDTable { name: b"BOTH_A7_KICK_R_AIR\0".as_ptr() as _, value: 897 }, 
	StringIDTable { name: b"BOTH_A7_KICK_L_AIR\0".as_ptr() as _, value: 898 }, 
	StringIDTable { name: b"BOTH_FLIP_ATTACK7\0".as_ptr() as _, value: 899 }, 
	StringIDTable { name: b"BOTH_FLIP_HOLD7\0".as_ptr() as _, value: 900 }, 
	StringIDTable { name: b"BOTH_FLIP_LAND\0".as_ptr() as _, value: 901 }, 
	StringIDTable { name: b"BOTH_PULL_IMPALE_STAB\0".as_ptr() as _, value: 902 }, 
	StringIDTable { name: b"BOTH_PULL_IMPALE_SWING\0".as_ptr() as _, value: 903 }, 
	StringIDTable { name: b"BOTH_PULLED_INAIR_B\0".as_ptr() as _, value: 904 }, 
	StringIDTable { name: b"BOTH_PULLED_INAIR_F\0".as_ptr() as _, value: 905 }, 
	StringIDTable { name: b"BOTH_STABDOWN\0".as_ptr() as _, value: 906 }, 
	StringIDTable { name: b"BOTH_STABDOWN_STAFF\0".as_ptr() as _, value: 907 }, 
	StringIDTable { name: b"BOTH_STABDOWN_DUAL\0".as_ptr() as _, value: 908 }, 
	StringIDTable { name: b"BOTH_A6_SABERPROTECT\0".as_ptr() as _, value: 909 }, 
	StringIDTable { name: b"BOTH_A7_SOULCAL\0".as_ptr() as _, value: 910 }, 
	StringIDTable { name: b"BOTH_A1_SPECIAL\0".as_ptr() as _, value: 911 }, 
	StringIDTable { name: b"BOTH_A2_SPECIAL\0".as_ptr() as _, value: 912 }, 
	StringIDTable { name: b"BOTH_A3_SPECIAL\0".as_ptr() as _, value: 913 }, 
	StringIDTable { name: b"BOTH_ROLL_STAB\0".as_ptr() as _, value: 914 }, 

	//# #sep ENUM2STRING(BOTH_ STANDING
	StringIDTable { name: b"BOTH_STAND1\0".as_ptr() as _, value: 915 }, //# Standing idle, no weapon, hands down
	StringIDTable { name: b"BOTH_STAND1IDLE1\0".as_ptr() as _, value: 916 }, //# Random standing idle
	StringIDTable { name: b"BOTH_STAND2\0".as_ptr() as _, value: 917 }, //# Standing idle with a saber
	StringIDTable { name: b"BOTH_STAND2IDLE1\0".as_ptr() as _, value: 918 }, //# Random standing idle
	StringIDTable { name: b"BOTH_STAND2IDLE2\0".as_ptr() as _, value: 919 }, 
	StringIDTable { name: b"BOTH_STAND3\0".as_ptr() as _, value: 920 }, //# Standing idle with 2-handed weapon
	StringIDTable { name: b"BOTH_STAND3IDLE1\0".as_ptr() as _, value: 921 }, //# Random standing idle
	StringIDTable { name: b"BOTH_STAND4\0".as_ptr() as _, value: 922 }, //# hands clasp behind back
	StringIDTable { name: b"BOTH_STAND5\0".as_ptr() as _, value: 923 }, //# standing idle, no weapon, hand down, back straight
	StringIDTable { name: b"BOTH_STAND5IDLE1\0".as_ptr() as _, value: 924 }, //# Random standing idle
	StringIDTable { name: b"BOTH_STAND6\0".as_ptr() as _, value: 925 }, //# one handed), gun at side), relaxed stand
	StringIDTable { name: b"BOTH_STAND8\0".as_ptr() as _, value: 926 }, //# both hands on hips (male)
	StringIDTable { name: b"BOTH_STAND1TO2\0".as_ptr() as _, value: 927 }, //# Transition from stand1 to stand2
	StringIDTable { name: b"BOTH_STAND2TO1\0".as_ptr() as _, value: 928 }, //# Transition from stand2 to stand1
	StringIDTable { name: b"BOTH_STAND2TO4\0".as_ptr() as _, value: 929 }, //# Transition from stand2 to stand4
	StringIDTable { name: b"BOTH_STAND4TO2\0".as_ptr() as _, value: 930 }, //# Transition from stand4 to stand2
	StringIDTable { name: b"BOTH_STAND4TOATTACK2\0".as_ptr() as _, value: 931 }, //# relaxed stand to 1-handed pistol ready
	StringIDTable { name: b"BOTH_STANDUP2\0".as_ptr() as _, value: 932 }, //# Luke standing up from his meditation platform (cin # 37)
	StringIDTable { name: b"BOTH_STAND5TOSIT3\0".as_ptr() as _, value: 933 }, //# transition from stand 5 to sit 3
	StringIDTable { name: b"BOTH_STAND1TOSTAND5\0".as_ptr() as _, value: 934 }, //# Transition from stand1 to stand5
	StringIDTable { name: b"BOTH_STAND5TOSTAND1\0".as_ptr() as _, value: 935 }, //# Transition from stand5 to stand1
	StringIDTable { name: b"BOTH_STAND5TOAIM\0".as_ptr() as _, value: 936 }, //# Transition of Kye aiming his gun at Desann (cin #9) 
	StringIDTable { name: b"BOTH_STAND5STARTLEDLOOKLEFT\0".as_ptr() as _, value: 937 }, //# Kyle turning to watch the bridge drop (cin #9) 
	StringIDTable { name: b"BOTH_STARTLEDLOOKLEFTTOSTAND5\0".as_ptr() as _, value: 938 }, //# Kyle returning to stand 5 from watching the bridge drop (cin #9) 
	StringIDTable { name: b"BOTH_STAND5TOSTAND8\0".as_ptr() as _, value: 939 }, //# Transition from stand5 to stand8
	StringIDTable { name: b"BOTH_STAND7TOSTAND8\0".as_ptr() as _, value: 940 }, //# Tavion putting hands on back of chair (cin #11)
	StringIDTable { name: b"BOTH_STAND8TOSTAND5\0".as_ptr() as _, value: 941 }, //# Transition from stand8 to stand5
	StringIDTable { name: b"BOTH_STAND9\0".as_ptr() as _, value: 942 }, //# Kyle's standing idle, no weapon, hands down
	StringIDTable { name: b"BOTH_STAND9IDLE1\0".as_ptr() as _, value: 943 }, //# Kyle's random standing idle
	StringIDTable { name: b"BOTH_STAND5SHIFTWEIGHT\0".as_ptr() as _, value: 944 }, //# Weightshift from stand5 to side and back to stand5
	StringIDTable { name: b"BOTH_STAND5SHIFTWEIGHTSTART\0".as_ptr() as _, value: 945 }, //# From stand5 to side
	StringIDTable { name: b"BOTH_STAND5SHIFTWEIGHTSTOP\0".as_ptr() as _, value: 946 }, //# From side to stand5
	StringIDTable { name: b"BOTH_STAND5TURNLEFTSTART\0".as_ptr() as _, value: 947 }, //# Start turning left from stand5
	StringIDTable { name: b"BOTH_STAND5TURNLEFTSTOP\0".as_ptr() as _, value: 948 }, //# Stop turning left from stand5
	StringIDTable { name: b"BOTH_STAND5TURNRIGHTSTART\0".as_ptr() as _, value: 949 }, //# Start turning right from stand5
	StringIDTable { name: b"BOTH_STAND5TURNRIGHTSTOP\0".as_ptr() as _, value: 950 }, //# Stop turning right from stand5
	StringIDTable { name: b"BOTH_STAND5LOOK180LEFTSTART\0".as_ptr() as _, value: 951 }, //# Start looking over left shoulder (cin #17)
	StringIDTable { name: b"BOTH_STAND5LOOK180LEFTSTOP\0".as_ptr() as _, value: 952 }, //# Stop looking over left shoulder (cin #17)

	StringIDTable { name: b"BOTH_CONSOLE1START\0".as_ptr() as _, value: 953 }, //# typing at a console
	StringIDTable { name: b"BOTH_CONSOLE1\0".as_ptr() as _, value: 954 }, //# typing at a console
	StringIDTable { name: b"BOTH_CONSOLE1STOP\0".as_ptr() as _, value: 955 }, //# typing at a console
	StringIDTable { name: b"BOTH_CONSOLE2START\0".as_ptr() as _, value: 956 }, //# typing at a console with comm link in hand (cin #5) 
	StringIDTable { name: b"BOTH_CONSOLE2\0".as_ptr() as _, value: 957 }, //# typing at a console with comm link in hand (cin #5) 
	StringIDTable { name: b"BOTH_CONSOLE2STOP\0".as_ptr() as _, value: 958 }, //# typing at a console with comm link in hand (cin #5) 
	StringIDTable { name: b"BOTH_CONSOLE2HOLDCOMSTART\0".as_ptr() as _, value: 959 }, //# lean in to type at console while holding comm link in hand (cin #5) 
	StringIDTable { name: b"BOTH_CONSOLE2HOLDCOMSTOP\0".as_ptr() as _, value: 960 }, //# lean away after typing at console while holding comm link in hand (cin #5) 

	StringIDTable { name: b"BOTH_GUARD_LOOKAROUND1\0".as_ptr() as _, value: 961 }, //# Cradling weapon and looking around
	StringIDTable { name: b"BOTH_GUARD_IDLE1\0".as_ptr() as _, value: 962 }, //# Cradling weapon and standing
	StringIDTable { name: b"BOTH_GESTURE1\0".as_ptr() as _, value: 963 }, //# Generic gesture), non-specific
	StringIDTable { name: b"BOTH_GESTURE2\0".as_ptr() as _, value: 964 }, //# Generic gesture), non-specific
	StringIDTable { name: b"BOTH_WALK1TALKCOMM1\0".as_ptr() as _, value: 965 }, //# Talking into coom link while walking
	StringIDTable { name: b"BOTH_TALK1\0".as_ptr() as _, value: 966 }, //# Generic talk anim
	StringIDTable { name: b"BOTH_TALK2\0".as_ptr() as _, value: 967 }, //# Generic talk anim
	StringIDTable { name: b"BOTH_TALKCOMM1START\0".as_ptr() as _, value: 968 }, //# Start talking into a comm link
	StringIDTable { name: b"BOTH_TALKCOMM1\0".as_ptr() as _, value: 969 }, //# Talking into a comm link
	StringIDTable { name: b"BOTH_TALKCOMM1STOP\0".as_ptr() as _, value: 970 }, //# Stop talking into a comm link
	StringIDTable { name: b"BOTH_TALKGESTURE1\0".as_ptr() as _, value: 971 }, //# Generic talk anim

	StringIDTable { name: b"BOTH_HEADTILTLSTART\0".as_ptr() as _, value: 972 }, //# Head tilt to left
	StringIDTable { name: b"BOTH_HEADTILTLSTOP\0".as_ptr() as _, value: 973 }, //# Head tilt to left
	StringIDTable { name: b"BOTH_HEADTILTRSTART\0".as_ptr() as _, value: 974 }, //# Head tilt to right
	StringIDTable { name: b"BOTH_HEADTILTRSTOP\0".as_ptr() as _, value: 975 }, //# Head tilt to right
	StringIDTable { name: b"BOTH_HEADNOD\0".as_ptr() as _, value: 976 }, //# Head shake YES
	StringIDTable { name: b"BOTH_HEADSHAKE\0".as_ptr() as _, value: 977 }, //# Head shake NO
	StringIDTable { name: b"BOTH_SIT2HEADTILTLSTART\0".as_ptr() as _, value: 978 }, //# Head tilt to left from seated position 2
	StringIDTable { name: b"BOTH_SIT2HEADTILTLSTOP\0".as_ptr() as _, value: 979 }, //# Head tilt to left from seated position 2

	StringIDTable { name: b"BOTH_REACH1START\0".as_ptr() as _, value: 980 }, //# Monmothma reaching for crystal
	StringIDTable { name: b"BOTH_REACH1STOP\0".as_ptr() as _, value: 981 }, //# Monmothma reaching for crystal

	StringIDTable { name: b"BOTH_COME_ON1\0".as_ptr() as _, value: 982 }, //# Jan gesturing to Kyle (cin #32a)
	StringIDTable { name: b"BOTH_STEADYSELF1\0".as_ptr() as _, value: 983 }, //# Jan trying to keep footing (cin #32a) Kyle (cin#5)
	StringIDTable { name: b"BOTH_STEADYSELF1END\0".as_ptr() as _, value: 984 }, //# Return hands to side from STEADSELF1 Kyle (cin#5)
	StringIDTable { name: b"BOTH_SILENCEGESTURE1\0".as_ptr() as _, value: 985 }, //# Luke silencing Kyle with a raised hand (cin #37)
	StringIDTable { name: b"BOTH_REACHFORSABER1\0".as_ptr() as _, value: 986 }, //# Luke holding hand out for Kyle's saber (cin #37)
	StringIDTable { name: b"BOTH_SABERKILLER1\0".as_ptr() as _, value: 987 }, //# Tavion about to strike Jan with saber (cin #9)
	StringIDTable { name: b"BOTH_SABERKILLEE1\0".as_ptr() as _, value: 988 }, //# Jan about to be struck by Tavion with saber (cin #9)
	StringIDTable { name: b"BOTH_HUGGER1\0".as_ptr() as _, value: 989 }, //# Kyle hugging Jan (cin #29)
	StringIDTable { name: b"BOTH_HUGGERSTOP1\0".as_ptr() as _, value: 990 }, //# Kyle stop hugging Jan but don't let her go (cin #29)
	StringIDTable { name: b"BOTH_HUGGEE1\0".as_ptr() as _, value: 991 }, //# Jan being hugged (cin #29)
	StringIDTable { name: b"BOTH_HUGGEESTOP1\0".as_ptr() as _, value: 992 }, //# Jan stop being hugged but don't let go (cin #29)

	StringIDTable { name: b"BOTH_SABERTHROW1START\0".as_ptr() as _, value: 993 }, //# Desann throwing his light saber (cin #26)
	StringIDTable { name: b"BOTH_SABERTHROW1STOP\0".as_ptr() as _, value: 994 }, //# Desann throwing his light saber (cin #26)
	StringIDTable { name: b"BOTH_SABERTHROW2START\0".as_ptr() as _, value: 995 }, //# Kyle throwing his light saber (cin #32)
	StringIDTable { name: b"BOTH_SABERTHROW2STOP\0".as_ptr() as _, value: 996 }, //# Kyle throwing his light saber (cin #32)

	//# #sep ENUM2STRING(BOTH_ SITTING/CROUCHING
	StringIDTable { name: b"BOTH_SIT1\0".as_ptr() as _, value: 997 }, //# Normal chair sit.
	StringIDTable { name: b"BOTH_SIT2\0".as_ptr() as _, value: 998 }, //# Lotus position.
	StringIDTable { name: b"BOTH_SIT3\0".as_ptr() as _, value: 999 }, //# Sitting in tired position), elbows on knees

	StringIDTable { name: b"BOTH_SIT2TOSTAND5\0".as_ptr() as _, value: 1000 }, //# Transition from sit 2 to stand 5
	StringIDTable { name: b"BOTH_STAND5TOSIT2\0".as_ptr() as _, value: 1001 }, //# Transition from stand 5 to sit 2
	StringIDTable { name: b"BOTH_SIT2TOSIT4\0".as_ptr() as _, value: 1002 }, //# Trans from sit2 to sit4 (cin #12) Luke leaning back from lotus position.
	StringIDTable { name: b"BOTH_SIT3TOSTAND5\0".as_ptr() as _, value: 1003 }, //# transition from sit 3 to stand 5

	StringIDTable { name: b"BOTH_CROUCH1\0".as_ptr() as _, value: 1004 }, //# Transition from standing to crouch
	StringIDTable { name: b"BOTH_CROUCH1IDLE\0".as_ptr() as _, value: 1005 }, //# Crouching idle
	StringIDTable { name: b"BOTH_CROUCH1WALK\0".as_ptr() as _, value: 1006 }, //# Walking while crouched
	StringIDTable { name: b"BOTH_CROUCH1WALKBACK\0".as_ptr() as _, value: 1007 }, //# Walking while crouched
	StringIDTable { name: b"BOTH_UNCROUCH1\0".as_ptr() as _, value: 1008 }, //# Transition from crouch to standing
	StringIDTable { name: b"BOTH_CROUCH2TOSTAND1\0".as_ptr() as _, value: 1009 }, //# going from crouch2 to stand1
	StringIDTable { name: b"BOTH_CROUCH3\0".as_ptr() as _, value: 1010 }, //# Desann crouching down to Kyle (cin 9)
	StringIDTable { name: b"BOTH_UNCROUCH3\0".as_ptr() as _, value: 1011 }, //# Desann uncrouching down to Kyle (cin 9)
	StringIDTable { name: b"BOTH_CROUCH4\0".as_ptr() as _, value: 1012 }, //# Slower version of crouch1 for cinematics
	StringIDTable { name: b"BOTH_UNCROUCH4\0".as_ptr() as _, value: 1013 }, //# Slower version of uncrouch1 for cinematics

	StringIDTable { name: b"BOTH_GUNSIT1\0".as_ptr() as _, value: 1014 }, //# sitting on an emplaced gun.

	// Swoop Vehicle animations.
	//* #sep BOTH_ SWOOP ANIMS
	StringIDTable { name: b"BOTH_VS_MOUNT_L\0".as_ptr() as _, value: 1015 }, //# Mount from left		
	StringIDTable { name: b"BOTH_VS_DISMOUNT_L\0".as_ptr() as _, value: 1016 }, //# Dismount to left		
	StringIDTable { name: b"BOTH_VS_MOUNT_R\0".as_ptr() as _, value: 1017 }, //# Mount from  right (symmetry)		
	StringIDTable { name: b"BOTH_VS_DISMOUNT_R\0".as_ptr() as _, value: 1018 }, //# Dismount to  right (symmetry)		

	StringIDTable { name: b"BOTH_VS_MOUNTJUMP_L\0".as_ptr() as _, value: 1019 }, //#
	StringIDTable { name: b"BOTH_VS_MOUNTTHROW\0".as_ptr() as _, value: 1020 }, //# Land on an occupied vehicle & throw off current pilot
	StringIDTable { name: b"BOTH_VS_MOUNTTHROW_L\0".as_ptr() as _, value: 1021 }, //# Land on an occupied vehicle & throw off current pilot
	StringIDTable { name: b"BOTH_VS_MOUNTTHROW_R\0".as_ptr() as _, value: 1022 }, //# Land on an occupied vehicle & throw off current pilot
	StringIDTable { name: b"BOTH_VS_MOUNTTHROWEE\0".as_ptr() as _, value: 1023 }, //# Current pilot getting thrown off by another guy
				
	StringIDTable { name: b"BOTH_VS_LOOKLEFT\0".as_ptr() as _, value: 1024 }, //# Turn & Look behind and to the left (no weapon)		
	StringIDTable { name: b"BOTH_VS_LOOKRIGHT\0".as_ptr() as _, value: 1025 }, //# Turn & Look behind and to the right (no weapon)		

	StringIDTable { name: b"BOTH_VS_TURBO\0".as_ptr() as _, value: 1026 }, //# Hit The Turbo Button

	StringIDTable { name: b"BOTH_VS_REV\0".as_ptr() as _, value: 1027 }, //# Player looks back as swoop reverses		

	StringIDTable { name: b"BOTH_VS_AIR\0".as_ptr() as _, value: 1028 }, //# Player stands up when swoop is airborn		
	StringIDTable { name: b"BOTH_VS_AIR_G\0".as_ptr() as _, value: 1029 }, //# "" with Gun
	StringIDTable { name: b"BOTH_VS_AIR_SL\0".as_ptr() as _, value: 1030 }, //# "" with Saber Left
	StringIDTable { name: b"BOTH_VS_AIR_SR\0".as_ptr() as _, value: 1031 }, //# "" with Saber Right

	StringIDTable { name: b"BOTH_VS_LAND\0".as_ptr() as _, value: 1032 }, //# Player bounces down when swoop lands		
	StringIDTable { name: b"BOTH_VS_LAND_G\0".as_ptr() as _, value: 1033 }, //#  "" with Gun
	StringIDTable { name: b"BOTH_VS_LAND_SL\0".as_ptr() as _, value: 1034 }, //#  "" with Saber Left
	StringIDTable { name: b"BOTH_VS_LAND_SR\0".as_ptr() as _, value: 1035 }, //#  "" with Saber Right

	StringIDTable { name: b"BOTH_VS_IDLE\0".as_ptr() as _, value: 1036 }, //# Sit
	StringIDTable { name: b"BOTH_VS_IDLE_G\0".as_ptr() as _, value: 1037 }, //# Sit (gun)
	StringIDTable { name: b"BOTH_VS_IDLE_SL\0".as_ptr() as _, value: 1038 }, //# Sit (saber left)		
	StringIDTable { name: b"BOTH_VS_IDLE_SR\0".as_ptr() as _, value: 1039 }, //# Sit (saber right)		

	StringIDTable { name: b"BOTH_VS_LEANL\0".as_ptr() as _, value: 1040 }, //# Lean left
	StringIDTable { name: b"BOTH_VS_LEANL_G\0".as_ptr() as _, value: 1041 }, //# Lean left (gun)		
	StringIDTable { name: b"BOTH_VS_LEANL_SL\0".as_ptr() as _, value: 1042 }, //# Lean left (saber left)		
	StringIDTable { name: b"BOTH_VS_LEANL_SR\0".as_ptr() as _, value: 1043 }, //# Lean left (saber right)		

	StringIDTable { name: b"BOTH_VS_LEANR\0".as_ptr() as _, value: 1044 }, //# Lean right		
	StringIDTable { name: b"BOTH_VS_LEANR_G\0".as_ptr() as _, value: 1045 }, //# Lean right (gun)		
	StringIDTable { name: b"BOTH_VS_LEANR_SL\0".as_ptr() as _, value: 1046 }, //# Lean right (saber left)		
	StringIDTable { name: b"BOTH_VS_LEANR_SR\0".as_ptr() as _, value: 1047 }, //# Lean right (saber right)		
				
	StringIDTable { name: b"BOTH_VS_ATL_S\0".as_ptr() as _, value: 1048 }, //# Attack left with saber		
	StringIDTable { name: b"BOTH_VS_ATR_S\0".as_ptr() as _, value: 1049 }, //# Attack right with saber		
	StringIDTable { name: b"BOTH_VS_ATR_TO_L_S\0".as_ptr() as _, value: 1050 }, //# Attack toss saber from right to left hand
	StringIDTable { name: b"BOTH_VS_ATL_TO_R_S\0".as_ptr() as _, value: 1051 }, //# Attack toss saber from left to right hand
	StringIDTable { name: b"BOTH_VS_ATR_G\0".as_ptr() as _, value: 1052 }, //# Attack right with gun (90)		
	StringIDTable { name: b"BOTH_VS_ATL_G\0".as_ptr() as _, value: 1053 }, //# Attack left with gun (90)		
	StringIDTable { name: b"BOTH_VS_ATF_G\0".as_ptr() as _, value: 1054 }, //# Attack forward with gun		

	StringIDTable { name: b"BOTH_VS_PAIN1\0".as_ptr() as _, value: 1055 }, //# Pain		

	// Added 12/04/02 by Aurelio.
	//* #sep BOTH_ TAUNTAUN ANIMS
	StringIDTable { name: b"BOTH_VT_MOUNT_L\0".as_ptr() as _, value: 1056 }, //# Mount from left	
	StringIDTable { name: b"BOTH_VT_MOUNT_R\0".as_ptr() as _, value: 1057 }, //# Mount from right	
	StringIDTable { name: b"BOTH_VT_MOUNT_B\0".as_ptr() as _, value: 1058 }, //# Mount from air, behind
	StringIDTable { name: b"BOTH_VT_DISMOUNT\0".as_ptr() as _, value: 1059 }, //# Dismount for tauntaun
	StringIDTable { name: b"BOTH_VT_DISMOUNT_L\0".as_ptr() as _, value: 1060 }, //# Dismount to tauntauns left	
	StringIDTable { name: b"BOTH_VT_DISMOUNT_R\0".as_ptr() as _, value: 1061 }, //# Dismount to tauntauns right (symmetry)	

	StringIDTable { name: b"BOTH_VT_WALK_FWD\0".as_ptr() as _, value: 1062 }, //# Walk forward	
	StringIDTable { name: b"BOTH_VT_WALK_REV\0".as_ptr() as _, value: 1063 }, //# Walk backward	
	StringIDTable { name: b"BOTH_VT_WALK_FWD_L\0".as_ptr() as _, value: 1064 }, //# walk lean left
	StringIDTable { name: b"BOTH_VT_WALK_FWD_R\0".as_ptr() as _, value: 1065 }, //# walk lean right
	StringIDTable { name: b"BOTH_VT_RUN_FWD\0".as_ptr() as _, value: 1066 }, //# Run forward	
	StringIDTable { name: b"BOTH_VT_RUN_REV\0".as_ptr() as _, value: 1067 }, //# Look backwards while running (not weapon specific)	
	StringIDTable { name: b"BOTH_VT_RUN_FWD_L\0".as_ptr() as _, value: 1068 }, //# run lean left
	StringIDTable { name: b"BOTH_VT_RUN_FWD_R\0".as_ptr() as _, value: 1069 }, //# run lean right

	StringIDTable { name: b"BOTH_VT_SLIDEF\0".as_ptr() as _, value: 1070 }, //# Tauntaun slides forward with abrupt stop	
	StringIDTable { name: b"BOTH_VT_AIR\0".as_ptr() as _, value: 1071 }, //# Tauntaun jump	
	StringIDTable { name: b"BOTH_VT_ATB\0".as_ptr() as _, value: 1072 }, //# Tauntaun tail swipe	
	StringIDTable { name: b"BOTH_VT_PAIN1\0".as_ptr() as _, value: 1073 }, //# Pain	
	StringIDTable { name: b"BOTH_VT_DEATH1\0".as_ptr() as _, value: 1074 }, //# Die	
	StringIDTable { name: b"BOTH_VT_STAND\0".as_ptr() as _, value: 1075 }, //# Stand still and breath	
	StringIDTable { name: b"BOTH_VT_BUCK\0".as_ptr() as _, value: 1076 }, //# Tauntaun bucking loop animation	

	StringIDTable { name: b"BOTH_VT_LAND\0".as_ptr() as _, value: 1077 }, //# Player bounces down when tauntaun lands	
	StringIDTable { name: b"BOTH_VT_TURBO\0".as_ptr() as _, value: 1078 }, //# Hit The Turbo Button
	StringIDTable { name: b"BOTH_VT_IDLE_SL\0".as_ptr() as _, value: 1079 }, //# Sit (saber left)		
	StringIDTable { name: b"BOTH_VT_IDLE_SR\0".as_ptr() as _, value: 1080 }, //# Sit (saber right)		
	StringIDTable { name: b"BOTH_VT_IDLE\0".as_ptr() as _, value: 1081 }, //# Sit with no weapon selected	
	StringIDTable { name: b"BOTH_VT_IDLE1\0".as_ptr() as _, value: 1082 }, //# Sit with no weapon selected	
	StringIDTable { name: b"BOTH_VT_IDLE_S\0".as_ptr() as _, value: 1083 }, //# Sit with saber selected	
	StringIDTable { name: b"BOTH_VT_IDLE_G\0".as_ptr() as _, value: 1084 }, //# Sit with gun selected	
	StringIDTable { name: b"BOTH_VT_IDLE_T\0".as_ptr() as _, value: 1085 }, //# Sit with thermal grenade selected

	StringIDTable { name: b"BOTH_VT_ATL_S\0".as_ptr() as _, value: 1086 }, //# Attack left with saber	
	StringIDTable { name: b"BOTH_VT_ATR_S\0".as_ptr() as _, value: 1087 }, //# Attack right with saber	
	StringIDTable { name: b"BOTH_VT_ATR_TO_L_S\0".as_ptr() as _, value: 1088 }, //# Attack toss saber from right to left hand
	StringIDTable { name: b"BOTH_VT_ATL_TO_R_S\0".as_ptr() as _, value: 1089 }, //# Attack toss saber from left to right hand
	StringIDTable { name: b"BOTH_VT_ATR_G\0".as_ptr() as _, value: 1090 }, //# Attack right with gun (90)	
	StringIDTable { name: b"BOTH_VT_ATL_G\0".as_ptr() as _, value: 1091 }, //# Attack left with gun (90)	
	StringIDTable { name: b"BOTH_VT_ATF_G\0".as_ptr() as _, value: 1092 }, //# Attack forward with gun	


	// Added 2/26/02 by Aurelio.
	//* #sep BOTH_ FIGHTER ANIMS

	///////////////////////////////////

	StringIDTable { name: b"BOTH_DEATH14_UNGRIP\0".as_ptr() as _, value: 1093 }, //# Desann's end death (cin #35)
	StringIDTable { name: b"BOTH_DEATH14_SITUP\0".as_ptr() as _, value: 1094 }, //# Tavion sitting up after having been thrown (cin #23)
	StringIDTable { name: b"BOTH_KNEES1\0".as_ptr() as _, value: 1095 }, //# Tavion on her knees
	StringIDTable { name: b"BOTH_KNEES2\0".as_ptr() as _, value: 1096 }, //# Tavion on her knees looking down
	StringIDTable { name: b"BOTH_KNEES2TO1\0".as_ptr() as _, value: 1097 }, //# Transition of KNEES2 to KNEES1

	//# #sep ENUM2STRING(BOTH_ MOVING
	StringIDTable { name: b"BOTH_WALK1\0".as_ptr() as _, value: 1098 }, //# Normal walk
	StringIDTable { name: b"BOTH_WALK2\0".as_ptr() as _, value: 1099 }, //# Normal walk
	StringIDTable { name: b"BOTH_WALK_STAFF\0".as_ptr() as _, value: 1100 }, //# Walk with saberstaff turned on
	StringIDTable { name: b"BOTH_WALKBACK_STAFF\0".as_ptr() as _, value: 1101 }, //# Walk backwards with saberstaff turned on
	StringIDTable { name: b"BOTH_WALK_DUAL\0".as_ptr() as _, value: 1102 }, //# Walk with dual turned on
	StringIDTable { name: b"BOTH_WALKBACK_DUAL\0".as_ptr() as _, value: 1103 }, //# Walk backwards with dual turned on
	StringIDTable { name: b"BOTH_WALK5\0".as_ptr() as _, value: 1104 }, //# Tavion taunting Kyle (cin 22)
	StringIDTable { name: b"BOTH_WALK6\0".as_ptr() as _, value: 1105 }, //# Slow walk for Luke (cin 12)
	StringIDTable { name: b"BOTH_WALK7\0".as_ptr() as _, value: 1106 }, //# Fast walk
	StringIDTable { name: b"BOTH_RUN1\0".as_ptr() as _, value: 1107 }, //# Full run
	StringIDTable { name: b"BOTH_RUN1START\0".as_ptr() as _, value: 1108 }, //# Start into full run1
	StringIDTable { name: b"BOTH_RUN1STOP\0".as_ptr() as _, value: 1109 }, //# Stop from full run1
	StringIDTable { name: b"BOTH_RUN2\0".as_ptr() as _, value: 1110 }, //# Full run
	StringIDTable { name: b"BOTH_RUN1TORUN2\0".as_ptr() as _, value: 1111 }, //# Wampa run anim transition
	StringIDTable { name: b"BOTH_RUN2TORUN1\0".as_ptr() as _, value: 1112 }, //# Wampa run anim transition
	StringIDTable { name: b"BOTH_RUN4\0".as_ptr() as _, value: 1113 }, //# Jawa run
	StringIDTable { name: b"BOTH_RUN_STAFF\0".as_ptr() as _, value: 1114 }, //# Run with saberstaff turned on
	StringIDTable { name: b"BOTH_RUNBACK_STAFF\0".as_ptr() as _, value: 1115 }, //# Run backwards with saberstaff turned on
	StringIDTable { name: b"BOTH_RUN_DUAL\0".as_ptr() as _, value: 1116 }, //# Run with dual turned on
	StringIDTable { name: b"BOTH_RUNBACK_DUAL\0".as_ptr() as _, value: 1117 }, //# Run backwards with dual turned on
	StringIDTable { name: b"BOTH_STRAFE_LEFT1\0".as_ptr() as _, value: 1118 }, //# Sidestep left), should loop
	StringIDTable { name: b"BOTH_STRAFE_RIGHT1\0".as_ptr() as _, value: 1119 }, //# Sidestep right), should loop
	StringIDTable { name: b"BOTH_RUNSTRAFE_LEFT1\0".as_ptr() as _, value: 1120 }, //# Sidestep left), should loop
	StringIDTable { name: b"BOTH_RUNSTRAFE_RIGHT1\0".as_ptr() as _, value: 1121 }, //# Sidestep right), should loop
	StringIDTable { name: b"BOTH_TURN_LEFT1\0".as_ptr() as _, value: 1122 }, //# Turn left), should loop
	StringIDTable { name: b"BOTH_TURN_RIGHT1\0".as_ptr() as _, value: 1123 }, //# Turn right), should loop
	StringIDTable { name: b"BOTH_TURNSTAND1\0".as_ptr() as _, value: 1124 }, //# Turn from STAND1 position
	StringIDTable { name: b"BOTH_TURNSTAND2\0".as_ptr() as _, value: 1125 }, //# Turn from STAND2 position
	StringIDTable { name: b"BOTH_TURNSTAND3\0".as_ptr() as _, value: 1126 }, //# Turn from STAND3 position
	StringIDTable { name: b"BOTH_TURNSTAND4\0".as_ptr() as _, value: 1127 }, //# Turn from STAND4 position
	StringIDTable { name: b"BOTH_TURNSTAND5\0".as_ptr() as _, value: 1128 }, //# Turn from STAND5 position
	StringIDTable { name: b"BOTH_TURNCROUCH1\0".as_ptr() as _, value: 1129 }, //# Turn from CROUCH1 position

	StringIDTable { name: b"BOTH_WALKBACK1\0".as_ptr() as _, value: 1130 }, //# Walk1 backwards
	StringIDTable { name: b"BOTH_WALKBACK2\0".as_ptr() as _, value: 1131 }, //# Walk2 backwards
	StringIDTable { name: b"BOTH_RUNBACK1\0".as_ptr() as _, value: 1132 }, //# Run1 backwards
	StringIDTable { name: b"BOTH_RUNBACK2\0".as_ptr() as _, value: 1133 }, //# Run1 backwards
	
	//# #sep BOTH_ JUMPING
	StringIDTable { name: b"BOTH_JUMP1\0".as_ptr() as _, value: 1134 }, //# Jump - wind-up and leave ground
	StringIDTable { name: b"BOTH_INAIR1\0".as_ptr() as _, value: 1135 }, //# In air loop (from jump)
	StringIDTable { name: b"BOTH_LAND1\0".as_ptr() as _, value: 1136 }, //# Landing (from in air loop)
	StringIDTable { name: b"BOTH_LAND2\0".as_ptr() as _, value: 1137 }, //# Landing Hard (from a great height)

	StringIDTable { name: b"BOTH_JUMPBACK1\0".as_ptr() as _, value: 1138 }, //# Jump backwards - wind-up and leave ground
	StringIDTable { name: b"BOTH_INAIRBACK1\0".as_ptr() as _, value: 1139 }, //# In air loop (from jump back)
	StringIDTable { name: b"BOTH_LANDBACK1\0".as_ptr() as _, value: 1140 }, //# Landing backwards(from in air loop)

	StringIDTable { name: b"BOTH_JUMPLEFT1\0".as_ptr() as _, value: 1141 }, //# Jump left - wind-up and leave ground
	StringIDTable { name: b"BOTH_INAIRLEFT1\0".as_ptr() as _, value: 1142 }, //# In air loop (from jump left)
	StringIDTable { name: b"BOTH_LANDLEFT1\0".as_ptr() as _, value: 1143 }, //# Landing left(from in air loop)

	StringIDTable { name: b"BOTH_JUMPRIGHT1\0".as_ptr() as _, value: 1144 }, //# Jump right - wind-up and leave ground
	StringIDTable { name: b"BOTH_INAIRRIGHT1\0".as_ptr() as _, value: 1145 }, //# In air loop (from jump right)
	StringIDTable { name: b"BOTH_LANDRIGHT1\0".as_ptr() as _, value: 1146 }, //# Landing right(from in air loop)

	StringIDTable { name: b"BOTH_FORCEJUMP1\0".as_ptr() as _, value: 1147 }, //# Jump - wind-up and leave ground
	StringIDTable { name: b"BOTH_FORCEINAIR1\0".as_ptr() as _, value: 1148 }, //# In air loop (from jump)
	StringIDTable { name: b"BOTH_FORCELAND1\0".as_ptr() as _, value: 1149 }, //# Landing (from in air loop)

	StringIDTable { name: b"BOTH_FORCEJUMPBACK1\0".as_ptr() as _, value: 1150 }, //# Jump backwards - wind-up and leave ground
	StringIDTable { name: b"BOTH_FORCEINAIRBACK1\0".as_ptr() as _, value: 1151 }, //# In air loop (from jump back)
	StringIDTable { name: b"BOTH_FORCELANDBACK1\0".as_ptr() as _, value: 1152 }, //# Landing backwards(from in air loop)

	StringIDTable { name: b"BOTH_FORCEJUMPLEFT1\0".as_ptr() as _, value: 1153 }, //# Jump left - wind-up and leave ground
	StringIDTable { name: b"BOTH_FORCEINAIRLEFT1\0".as_ptr() as _, value: 1154 }, //# In air loop (from jump left)
	StringIDTable { name: b"BOTH_FORCELANDLEFT1\0".as_ptr() as _, value: 1155 }, //# Landing left(from in air loop)

	StringIDTable { name: b"BOTH_FORCEJUMPRIGHT1\0".as_ptr() as _, value: 1156 }, //# Jump right - wind-up and leave ground
	StringIDTable { name: b"BOTH_FORCEINAIRRIGHT1\0".as_ptr() as _, value: 1157 }, //# In air loop (from jump right)
	StringIDTable { name: b"BOTH_FORCELANDRIGHT1\0".as_ptr() as _, value: 1158 }, //# Landing right(from in air loop)
	//# #sep BOTH_ ACROBATICS
	StringIDTable { name: b"BOTH_FLIP_F\0".as_ptr() as _, value: 1159 }, //# Flip forward
	StringIDTable { name: b"BOTH_FLIP_B\0".as_ptr() as _, value: 1160 }, //# Flip backwards
	StringIDTable { name: b"BOTH_FLIP_L\0".as_ptr() as _, value: 1161 }, //# Flip left
	StringIDTable { name: b"BOTH_FLIP_R\0".as_ptr() as _, value: 1162 }, //# Flip right

	StringIDTable { name: b"BOTH_ROLL_F\0".as_ptr() as _, value: 1163 }, //# Roll forward
	StringIDTable { name: b"BOTH_ROLL_B\0".as_ptr() as _, value: 1164 }, //# Roll backward
	StringIDTable { name: b"BOTH_ROLL_L\0".as_ptr() as _, value: 1165 }, //# Roll left
	StringIDTable { name: b"BOTH_ROLL_R\0".as_ptr() as _, value: 1166 }, //# Roll right

	StringIDTable { name: b"BOTH_HOP_F\0".as_ptr() as _, value: 1167 }, //# quickstep forward
	StringIDTable { name: b"BOTH_HOP_B\0".as_ptr() as _, value: 1168 }, //# quickstep backwards
	StringIDTable { name: b"BOTH_HOP_L\0".as_ptr() as _, value: 1169 }, //# quickstep left
	StringIDTable { name: b"BOTH_HOP_R\0".as_ptr() as _, value: 1170 }, //# quickstep right

	StringIDTable { name: b"BOTH_DODGE_FL\0".as_ptr() as _, value: 1171 }, //# lean-dodge forward left
	StringIDTable { name: b"BOTH_DODGE_FR\0".as_ptr() as _, value: 1172 }, //# lean-dodge forward right
	StringIDTable { name: b"BOTH_DODGE_BL\0".as_ptr() as _, value: 1173 }, //# lean-dodge backwards left
	StringIDTable { name: b"BOTH_DODGE_BR\0".as_ptr() as _, value: 1174 }, //# lean-dodge backwards right
	StringIDTable { name: b"BOTH_DODGE_L\0".as_ptr() as _, value: 1175 }, //# lean-dodge left
	StringIDTable { name: b"BOTH_DODGE_R\0".as_ptr() as _, value: 1176 }, //# lean-dodge right
	StringIDTable { name: b"BOTH_DODGE_HOLD_FL\0".as_ptr() as _, value: 1177 }, //# lean-dodge pose forward left
	StringIDTable { name: b"BOTH_DODGE_HOLD_FR\0".as_ptr() as _, value: 1178 }, //# lean-dodge pose forward right
	StringIDTable { name: b"BOTH_DODGE_HOLD_BL\0".as_ptr() as _, value: 1179 }, //# lean-dodge pose backwards left
	StringIDTable { name: b"BOTH_DODGE_HOLD_BR\0".as_ptr() as _, value: 1180 }, //# lean-dodge pose backwards right
	StringIDTable { name: b"BOTH_DODGE_HOLD_L\0".as_ptr() as _, value: 1181 }, //# lean-dodge pose left
	StringIDTable { name: b"BOTH_DODGE_HOLD_R\0".as_ptr() as _, value: 1182 }, //# lean-dodge pose right

	//MP taunt anims
	StringIDTable { name: b"BOTH_ENGAGETAUNT\0".as_ptr() as _, value: 1183 }, 
	StringIDTable { name: b"BOTH_BOW\0".as_ptr() as _, value: 1184 }, 
	StringIDTable { name: b"BOTH_MEDITATE\0".as_ptr() as _, value: 1185 }, 
	StringIDTable { name: b"BOTH_MEDITATE_END\0".as_ptr() as _, value: 1186 }, 
	StringIDTable { name: b"BOTH_SHOWOFF_FAST\0".as_ptr() as _, value: 1187 }, 
	StringIDTable { name: b"BOTH_SHOWOFF_MEDIUM\0".as_ptr() as _, value: 1188 }, 
	StringIDTable { name: b"BOTH_SHOWOFF_STRONG\0".as_ptr() as _, value: 1189 }, 
	StringIDTable { name: b"BOTH_SHOWOFF_DUAL\0".as_ptr() as _, value: 1190 }, 
	StringIDTable { name: b"BOTH_SHOWOFF_STAFF\0".as_ptr() as _, value: 1191 }, 
	StringIDTable { name: b"BOTH_VICTORY_FAST\0".as_ptr() as _, value: 1192 }, 
	StringIDTable { name: b"BOTH_VICTORY_MEDIUM\0".as_ptr() as _, value: 1193 }, 
	StringIDTable { name: b"BOTH_VICTORY_STRONG\0".as_ptr() as _, value: 1194 }, 
	StringIDTable { name: b"BOTH_VICTORY_DUAL\0".as_ptr() as _, value: 1195 }, 
	StringIDTable { name: b"BOTH_VICTORY_STAFF\0".as_ptr() as _, value: 1196 }, 
	//other saber/acro anims
	StringIDTable { name: b"BOTH_ARIAL_LEFT\0".as_ptr() as _, value: 1197 }, //# 
	StringIDTable { name: b"BOTH_ARIAL_RIGHT\0".as_ptr() as _, value: 1198 }, //# 
	StringIDTable { name: b"BOTH_CARTWHEEL_LEFT\0".as_ptr() as _, value: 1199 }, //# 
	StringIDTable { name: b"BOTH_CARTWHEEL_RIGHT\0".as_ptr() as _, value: 1200 }, //# 
	StringIDTable { name: b"BOTH_FLIP_LEFT\0".as_ptr() as _, value: 1201 }, //# 
	StringIDTable { name: b"BOTH_FLIP_BACK1\0".as_ptr() as _, value: 1202 }, //# 
	StringIDTable { name: b"BOTH_FLIP_BACK2\0".as_ptr() as _, value: 1203 }, //# 
	StringIDTable { name: b"BOTH_FLIP_BACK3\0".as_ptr() as _, value: 1204 }, //# 
	StringIDTable { name: b"BOTH_BUTTERFLY_LEFT\0".as_ptr() as _, value: 1205 }, //# 
	StringIDTable { name: b"BOTH_BUTTERFLY_RIGHT\0".as_ptr() as _, value: 1206 }, //# 
	StringIDTable { name: b"BOTH_WALL_RUN_RIGHT\0".as_ptr() as _, value: 1207 }, //# 
	StringIDTable { name: b"BOTH_WALL_RUN_RIGHT_FLIP\0".as_ptr() as _, value: 1208 }, //#
	StringIDTable { name: b"BOTH_WALL_RUN_RIGHT_STOP\0".as_ptr() as _, value: 1209 }, //# 
	StringIDTable { name: b"BOTH_WALL_RUN_LEFT\0".as_ptr() as _, value: 1210 }, //# 
	StringIDTable { name: b"BOTH_WALL_RUN_LEFT_FLIP\0".as_ptr() as _, value: 1211 }, //#
	StringIDTable { name: b"BOTH_WALL_RUN_LEFT_STOP\0".as_ptr() as _, value: 1212 }, //# 
	StringIDTable { name: b"BOTH_WALL_FLIP_RIGHT\0".as_ptr() as _, value: 1213 }, //# 
	StringIDTable { name: b"BOTH_WALL_FLIP_LEFT\0".as_ptr() as _, value: 1214 }, //# 
	StringIDTable { name: b"BOTH_KNOCKDOWN1\0".as_ptr() as _, value: 1215 }, //# knocked backwards
	StringIDTable { name: b"BOTH_KNOCKDOWN2\0".as_ptr() as _, value: 1216 }, //# knocked backwards hard
	StringIDTable { name: b"BOTH_KNOCKDOWN3\0".as_ptr() as _, value: 1217 }, //#	knocked forwards
	StringIDTable { name: b"BOTH_KNOCKDOWN4\0".as_ptr() as _, value: 1218 }, //# knocked backwards from crouch
	StringIDTable { name: b"BOTH_KNOCKDOWN5\0".as_ptr() as _, value: 1219 }, //# dupe of 3 - will be removed
	StringIDTable { name: b"BOTH_GETUP1\0".as_ptr() as _, value: 1220 }, //#
	StringIDTable { name: b"BOTH_GETUP2\0".as_ptr() as _, value: 1221 }, //#
	StringIDTable { name: b"BOTH_GETUP3\0".as_ptr() as _, value: 1222 }, //#
	StringIDTable { name: b"BOTH_GETUP4\0".as_ptr() as _, value: 1223 }, //#
	StringIDTable { name: b"BOTH_GETUP5\0".as_ptr() as _, value: 1224 }, //#
	StringIDTable { name: b"BOTH_GETUP_CROUCH_F1\0".as_ptr() as _, value: 1225 }, //#
	StringIDTable { name: b"BOTH_GETUP_CROUCH_B1\0".as_ptr() as _, value: 1226 }, //#
	StringIDTable { name: b"BOTH_FORCE_GETUP_F1\0".as_ptr() as _, value: 1227 }, //#
	StringIDTable { name: b"BOTH_FORCE_GETUP_F2\0".as_ptr() as _, value: 1228 }, //#
	StringIDTable { name: b"BOTH_FORCE_GETUP_B1\0".as_ptr() as _, value: 1229 }, //#
	StringIDTable { name: b"BOTH_FORCE_GETUP_B2\0".as_ptr() as _, value: 1230 }, //#
	StringIDTable { name: b"BOTH_FORCE_GETUP_B3\0".as_ptr() as _, value: 1231 }, //#
	StringIDTable { name: b"BOTH_FORCE_GETUP_B4\0".as_ptr() as _, value: 1232 }, //#
	StringIDTable { name: b"BOTH_FORCE_GETUP_B5\0".as_ptr() as _, value: 1233 }, //#
	StringIDTable { name: b"BOTH_FORCE_GETUP_B6\0".as_ptr() as _, value: 1234 }, //#
	StringIDTable { name: b"BOTH_GETUP_BROLL_B\0".as_ptr() as _, value: 1235 }, //#
	StringIDTable { name: b"BOTH_GETUP_BROLL_F\0".as_ptr() as _, value: 1236 }, //#
	StringIDTable { name: b"BOTH_GETUP_BROLL_L\0".as_ptr() as _, value: 1237 }, //#
	StringIDTable { name: b"BOTH_GETUP_BROLL_R\0".as_ptr() as _, value: 1238 }, //#
	StringIDTable { name: b"BOTH_GETUP_FROLL_B\0".as_ptr() as _, value: 1239 }, //#
	StringIDTable { name: b"BOTH_GETUP_FROLL_F\0".as_ptr() as _, value: 1240 }, //#
	StringIDTable { name: b"BOTH_GETUP_FROLL_L\0".as_ptr() as _, value: 1241 }, //#
	StringIDTable { name: b"BOTH_GETUP_FROLL_R\0".as_ptr() as _, value: 1242 }, //#
	StringIDTable { name: b"BOTH_WALL_FLIP_BACK1\0".as_ptr() as _, value: 1243 }, //#
	StringIDTable { name: b"BOTH_WALL_FLIP_BACK2\0".as_ptr() as _, value: 1244 }, //#
	StringIDTable { name: b"BOTH_SPIN1\0".as_ptr() as _, value: 1245 }, //#
	StringIDTable { name: b"BOTH_CEILING_CLING\0".as_ptr() as _, value: 1246 }, //# clinging to ceiling
	StringIDTable { name: b"BOTH_CEILING_DROP\0".as_ptr() as _, value: 1247 }, //# dropping from ceiling cling

	//TESTING
	StringIDTable { name: b"BOTH_FJSS_TR_BL\0".as_ptr() as _, value: 1248 }, //# jump spin slash tr to bl
	StringIDTable { name: b"BOTH_FJSS_TL_BR\0".as_ptr() as _, value: 1249 }, //# jump spin slash bl to tr
	StringIDTable { name: b"BOTH_RIGHTHANDCHOPPEDOFF\0".as_ptr() as _, value: 1250 }, //#
	StringIDTable { name: b"BOTH_DEFLECTSLASH__R__L_FIN\0".as_ptr() as _, value: 1251 }, //#
	StringIDTable { name: b"BOTH_BASHED1\0".as_ptr() as _, value: 1252 }, //#
	StringIDTable { name: b"BOTH_ARIAL_F1\0".as_ptr() as _, value: 1253 }, //#
	StringIDTable { name: b"BOTH_BUTTERFLY_FR1\0".as_ptr() as _, value: 1254 }, //#
	StringIDTable { name: b"BOTH_BUTTERFLY_FL1\0".as_ptr() as _, value: 1255 }, //#

	//NEW SABER/JEDI/FORCE ANIMS
	StringIDTable { name: b"BOTH_BACK_FLIP_UP\0".as_ptr() as _, value: 1256 }, //# back flip up Bonus Animation!!!!	
	StringIDTable { name: b"BOTH_LOSE_SABER\0".as_ptr() as _, value: 1257 }, //# player losing saber (pulled from hand by force pull 4 - Kyle?)
	StringIDTable { name: b"BOTH_STAFF_TAUNT\0".as_ptr() as _, value: 1258 }, //# taunt saberstaff			
	StringIDTable { name: b"BOTH_DUAL_TAUNT\0".as_ptr() as _, value: 1259 }, //# taunt dual
	StringIDTable { name: b"BOTH_A6_FB\0".as_ptr() as _, value: 1260 }, //# dual attack front/back		
	StringIDTable { name: b"BOTH_A6_LR\0".as_ptr() as _, value: 1261 }, //# dual attack left/right
	StringIDTable { name: b"BOTH_A7_HILT\0".as_ptr() as _, value: 1262 }, //# saber knock (alt + stand still)
	//Alora			
	StringIDTable { name: b"BOTH_ALORA_SPIN\0".as_ptr() as _, value: 1263 }, //#jump spin attack	death ballet	
	StringIDTable { name: b"BOTH_ALORA_FLIP_1\0".as_ptr() as _, value: 1264 }, //# gymnast move 1		
	StringIDTable { name: b"BOTH_ALORA_FLIP_2\0".as_ptr() as _, value: 1265 }, //# gymnast move 2		
	StringIDTable { name: b"BOTH_ALORA_FLIP_3\0".as_ptr() as _, value: 1266 }, //# gymnast move3		
	StringIDTable { name: b"BOTH_ALORA_FLIP_B\0".as_ptr() as _, value: 1267 }, //# gymnast move back		
	StringIDTable { name: b"BOTH_ALORA_SPIN_THROW\0".as_ptr() as _, value: 1268 }, //# dual saber throw		
	StringIDTable { name: b"BOTH_ALORA_SPIN_SLASH\0".as_ptr() as _, value: 1269 }, //# spin slash	special bonus animation!! :)	
	StringIDTable { name: b"BOTH_ALORA_TAUNT\0".as_ptr() as _, value: 1270 }, //# special taunt
	//Rosh (Kothos battle)			
	StringIDTable { name: b"BOTH_ROSH_PAIN\0".as_ptr() as _, value: 1271 }, //# hurt animation (exhausted)		
	StringIDTable { name: b"BOTH_ROSH_HEAL\0".as_ptr() as _, value: 1272 }, //# healed/rejuvenated		
	//Tavion			
	StringIDTable { name: b"BOTH_TAVION_SCEPTERGROUND\0".as_ptr() as _, value: 1273 }, //# stabbing ground with sith sword shoots electricity everywhere
	StringIDTable { name: b"BOTH_TAVION_SWORDPOWER\0".as_ptr() as _, value: 1274 }, //# Tavion doing the He-Man(tm) thing
	StringIDTable { name: b"BOTH_SCEPTER_START\0".as_ptr() as _, value: 1275 }, //#Point scepter and attack start
	StringIDTable { name: b"BOTH_SCEPTER_HOLD\0".as_ptr() as _, value: 1276 }, //#Point scepter and attack hold
	StringIDTable { name: b"BOTH_SCEPTER_STOP\0".as_ptr() as _, value: 1277 }, //#Point scepter and attack stop
	//Kyle Boss			
	StringIDTable { name: b"BOTH_KYLE_GRAB\0".as_ptr() as _, value: 1278 }, //# grab
	StringIDTable { name: b"BOTH_KYLE_MISS\0".as_ptr() as _, value: 1279 }, //# miss
	StringIDTable { name: b"BOTH_KYLE_PA_1\0".as_ptr() as _, value: 1280 }, //# hold 1
	StringIDTable { name: b"BOTH_PLAYER_PA_1\0".as_ptr() as _, value: 1281 }, //# player getting held 1
	StringIDTable { name: b"BOTH_KYLE_PA_2\0".as_ptr() as _, value: 1282 }, //# hold 2
	StringIDTable { name: b"BOTH_PLAYER_PA_2\0".as_ptr() as _, value: 1283 }, //# player getting held 2
	StringIDTable { name: b"BOTH_PLAYER_PA_FLY\0".as_ptr() as _, value: 1284 }, //# player getting knocked back from punch at end of hold 1
	StringIDTable { name: b"BOTH_KYLE_PA_3\0".as_ptr() as _, value: 1285 }, //# hold 3
	StringIDTable { name: b"BOTH_PLAYER_PA_3\0".as_ptr() as _, value: 1286 }, //# player getting held 3
	StringIDTable { name: b"BOTH_PLAYER_PA_3_FLY\0".as_ptr() as _, value: 1287 }, //# player getting thrown at end of hold 3
	//Rancor
	StringIDTable { name: b"BOTH_BUCK_RIDER\0".as_ptr() as _, value: 1288 }, //# Rancor bucks when someone is on him
	//WAMPA Grabbing enemy
	StringIDTable { name: b"BOTH_HOLD_START\0".as_ptr() as _, value: 1289 }, //#
	StringIDTable { name: b"BOTH_HOLD_MISS\0".as_ptr() as _, value: 1290 }, //#
	StringIDTable { name: b"BOTH_HOLD_IDLE\0".as_ptr() as _, value: 1291 }, //#
	StringIDTable { name: b"BOTH_HOLD_END\0".as_ptr() as _, value: 1292 }, //#
	StringIDTable { name: b"BOTH_HOLD_ATTACK\0".as_ptr() as _, value: 1293 }, //#
	StringIDTable { name: b"BOTH_HOLD_SNIFF\0".as_ptr() as _, value: 1294 }, //# Sniff the guy you're holding
	StringIDTable { name: b"BOTH_HOLD_DROP\0".as_ptr() as _, value: 1295 }, //# just drop 'em
	//BEING GRABBED BY WAMPA
	StringIDTable { name: b"BOTH_GRABBED\0".as_ptr() as _, value: 1296 }, //#
	StringIDTable { name: b"BOTH_RELEASED\0".as_ptr() as _, value: 1297 }, //#
	StringIDTable { name: b"BOTH_HANG_IDLE\0".as_ptr() as _, value: 1298 }, //#
	StringIDTable { name: b"BOTH_HANG_ATTACK\0".as_ptr() as _, value: 1299 }, //#
	StringIDTable { name: b"BOTH_HANG_PAIN\0".as_ptr() as _, value: 1300 }, //#

	//# #sep BOTH_ MISC MOVEMENT
	StringIDTable { name: b"BOTH_HIT1\0".as_ptr() as _, value: 1301 }, //# Kyle hit by crate in cin #9
	StringIDTable { name: b"BOTH_LADDER_UP1\0".as_ptr() as _, value: 1302 }, //# Climbing up a ladder with rungs at 16 unit intervals
	StringIDTable { name: b"BOTH_LADDER_DWN1\0".as_ptr() as _, value: 1303 }, //# Climbing down a ladder with rungs at 16 unit intervals
	StringIDTable { name: b"BOTH_LADDER_IDLE\0".as_ptr() as _, value: 1304 }, //#	Just sitting on the ladder

	//# #sep ENUM2STRING(BOTH_ FLYING IDLE
	StringIDTable { name: b"BOTH_FLY_SHIELDED\0".as_ptr() as _, value: 1305 }, //# For sentry droid, shields in

	//# #sep BOTH_ SWIMMING
	StringIDTable { name: b"BOTH_SWIM_IDLE1\0".as_ptr() as _, value: 1306 }, //# Swimming Idle 1
	StringIDTable { name: b"BOTH_SWIMFORWARD\0".as_ptr() as _, value: 1307 }, //# Swim forward loop
	StringIDTable { name: b"BOTH_SWIMBACKWARD\0".as_ptr() as _, value: 1308 }, //# Swim backward loop
	
	//# #sep ENUM2STRING(BOTH_ LYING
	StringIDTable { name: b"BOTH_SLEEP1\0".as_ptr() as _, value: 1309 }, //# laying on back-rknee up-rhand on torso
	StringIDTable { name: b"BOTH_SLEEP6START\0".as_ptr() as _, value: 1310 }, //# Kyle leaning back to sleep (cin 20)
	StringIDTable { name: b"BOTH_SLEEP6STOP\0".as_ptr() as _, value: 1311 }, //# Kyle waking up and shaking his head (cin 21)
	StringIDTable { name: b"BOTH_SLEEP1GETUP\0".as_ptr() as _, value: 1312 }, //# alarmed and getting up out of sleep1 pose to stand
	StringIDTable { name: b"BOTH_SLEEP1GETUP2\0".as_ptr() as _, value: 1313 }, //# 

	StringIDTable { name: b"BOTH_CHOKE1START\0".as_ptr() as _, value: 1314 }, //# tavion in force grip choke
	StringIDTable { name: b"BOTH_CHOKE1STARTHOLD\0".as_ptr() as _, value: 1315 }, //# loop of tavion in force grip choke
	StringIDTable { name: b"BOTH_CHOKE1\0".as_ptr() as _, value: 1316 }, //# tavion in force grip choke

	StringIDTable { name: b"BOTH_CHOKE2\0".as_ptr() as _, value: 1317 }, //# tavion recovering from force grip choke
	StringIDTable { name: b"BOTH_CHOKE3\0".as_ptr() as _, value: 1318 }, //# left-handed choke (for people still holding a weapon)

	//# #sep ENUM2STRING(BOTH_ HUNTER-SEEKER BOT-SPECIFIC
	StringIDTable { name: b"BOTH_POWERUP1\0".as_ptr() as _, value: 1319 }, //# Wakes up

	StringIDTable { name: b"BOTH_TURNON\0".as_ptr() as _, value: 1320 }, //# Protocol Droid wakes up
	StringIDTable { name: b"BOTH_TURNOFF\0".as_ptr() as _, value: 1321 }, //# Protocol Droid shuts off
	StringIDTable { name: b"BOTH_BUTTON1\0".as_ptr() as _, value: 1322 }, //# Single button push with right hand
	StringIDTable { name: b"BOTH_BUTTON2\0".as_ptr() as _, value: 1323 }, //# Single button push with left finger
	StringIDTable { name: b"BOTH_BUTTON_HOLD\0".as_ptr() as _, value: 1324 }, //# Single button hold with left hand
	StringIDTable { name: b"BOTH_BUTTON_RELEASE\0".as_ptr() as _, value: 1325 }, //# Single button release with left hand

	//# JEDI-SPECIFIC
	//# #sep BOTH_ FORCE ANIMS
	StringIDTable { name: b"BOTH_RESISTPUSH\0".as_ptr() as _, value: 1326 }, //# plant yourself to resist force push/pulls.
	StringIDTable { name: b"BOTH_FORCEPUSH\0".as_ptr() as _, value: 1327 }, //# Use off-hand to do force power.
	StringIDTable { name: b"BOTH_FORCEPULL\0".as_ptr() as _, value: 1328 }, //# Use off-hand to do force power.
	StringIDTable { name: b"BOTH_MINDTRICK1\0".as_ptr() as _, value: 1329 }, //# Use off-hand to do mind trick
	StringIDTable { name: b"BOTH_MINDTRICK2\0".as_ptr() as _, value: 1330 }, //# Use off-hand to do distraction
	StringIDTable { name: b"BOTH_FORCELIGHTNING\0".as_ptr() as _, value: 1331 }, //# Use off-hand to do lightning
	StringIDTable { name: b"BOTH_FORCELIGHTNING_START\0".as_ptr() as _, value: 1332 }, //# Use off-hand to do lightning - start
	StringIDTable { name: b"BOTH_FORCELIGHTNING_HOLD\0".as_ptr() as _, value: 1333 }, //# Use off-hand to do lightning - hold
	StringIDTable { name: b"BOTH_FORCELIGHTNING_RELEASE\0".as_ptr() as _, value: 1334 }, //# Use off-hand to do lightning - release
	StringIDTable { name: b"BOTH_FORCEHEAL_START\0".as_ptr() as _, value: 1335 }, //# Healing meditation pose start
	StringIDTable { name: b"BOTH_FORCEHEAL_STOP\0".as_ptr() as _, value: 1336 }, //# Healing meditation pose end
	StringIDTable { name: b"BOTH_FORCEHEAL_QUICK\0".as_ptr() as _, value: 1337 }, //# Healing meditation gesture
	StringIDTable { name: b"BOTH_SABERPULL\0".as_ptr() as _, value: 1338 }, //# Use off-hand to do force power.
	StringIDTable { name: b"BOTH_FORCEGRIP1\0".as_ptr() as _, value: 1339 }, //# force-gripping (no anim?)
	StringIDTable { name: b"BOTH_FORCEGRIP3\0".as_ptr() as _, value: 1340 }, //# force-gripping (right-hand)
	StringIDTable { name: b"BOTH_FORCEGRIP3THROW\0".as_ptr() as _, value: 1341 }, //# throwing while force-gripping (right hand)
	StringIDTable { name: b"BOTH_FORCEGRIP_HOLD\0".as_ptr() as _, value: 1342 }, //# Use off-hand to do grip - hold
	StringIDTable { name: b"BOTH_FORCEGRIP_RELEASE\0".as_ptr() as _, value: 1343 }, //# Use off-hand to do grip - release
	StringIDTable { name: b"BOTH_TOSS1\0".as_ptr() as _, value: 1344 }, //# throwing to left after force gripping
	StringIDTable { name: b"BOTH_TOSS2\0".as_ptr() as _, value: 1345 }, //# throwing to right after force gripping
	//NEW force anims for JKA:
	StringIDTable { name: b"BOTH_FORCE_RAGE\0".as_ptr() as _, value: 1346 }, 
	StringIDTable { name: b"BOTH_FORCE_2HANDEDLIGHTNING\0".as_ptr() as _, value: 1347 }, 
	StringIDTable { name: b"BOTH_FORCE_2HANDEDLIGHTNING_START\0".as_ptr() as _, value: 1348 }, 
	StringIDTable { name: b"BOTH_FORCE_2HANDEDLIGHTNING_HOLD\0".as_ptr() as _, value: 1349 }, 
	StringIDTable { name: b"BOTH_FORCE_2HANDEDLIGHTNING_RELEASE\0".as_ptr() as _, value: 1350 }, 
	StringIDTable { name: b"BOTH_FORCE_DRAIN\0".as_ptr() as _, value: 1351 }, 
	StringIDTable { name: b"BOTH_FORCE_DRAIN_START\0".as_ptr() as _, value: 1352 }, 
	StringIDTable { name: b"BOTH_FORCE_DRAIN_HOLD\0".as_ptr() as _, value: 1353 }, 
	StringIDTable { name: b"BOTH_FORCE_DRAIN_RELEASE\0".as_ptr() as _, value: 1354 }, 
	StringIDTable { name: b"BOTH_FORCE_DRAIN_GRAB_START\0".as_ptr() as _, value: 1355 }, 
	StringIDTable { name: b"BOTH_FORCE_DRAIN_GRAB_HOLD\0".as_ptr() as _, value: 1356 }, 
	StringIDTable { name: b"BOTH_FORCE_DRAIN_GRAB_END\0".as_ptr() as _, value: 1357 }, 
	StringIDTable { name: b"BOTH_FORCE_DRAIN_GRABBED\0".as_ptr() as _, value: 1358 }, 
	StringIDTable { name: b"BOTH_FORCE_ABSORB\0".as_ptr() as _, value: 1359 }, 
	StringIDTable { name: b"BOTH_FORCE_ABSORB_START\0".as_ptr() as _, value: 1360 }, 
	StringIDTable { name: b"BOTH_FORCE_ABSORB_END\0".as_ptr() as _, value: 1361 }, 
	StringIDTable { name: b"BOTH_FORCE_PROTECT\0".as_ptr() as _, value: 1362 }, 
	StringIDTable { name: b"BOTH_FORCE_PROTECT_FAST\0".as_ptr() as _, value: 1363 }, 

	StringIDTable { name: b"BOTH_WIND\0".as_ptr() as _, value: 1364 }, 

	StringIDTable { name: b"BOTH_STAND_TO_KNEEL\0".as_ptr() as _, value: 1365 }, 
	StringIDTable { name: b"BOTH_KNEEL_TO_STAND\0".as_ptr() as _, value: 1366 }, 

	StringIDTable { name: b"BOTH_TUSKENATTACK1\0".as_ptr() as _, value: 1367 }, 
	StringIDTable { name: b"BOTH_TUSKENATTACK2\0".as_ptr() as _, value: 1368 }, 
	StringIDTable { name: b"BOTH_TUSKENATTACK3\0".as_ptr() as _, value: 1369 }, 
	StringIDTable { name: b"BOTH_TUSKENLUNGE1\0".as_ptr() as _, value: 1370 }, 
	StringIDTable { name: b"BOTH_TUSKENTAUNT1\0".as_ptr() as _, value: 1371 }, 

	StringIDTable { name: b"BOTH_COWER1_START\0".as_ptr() as _, value: 1372 }, //# cower start
	StringIDTable { name: b"BOTH_COWER1\0".as_ptr() as _, value: 1373 }, //# cower loop
	StringIDTable { name: b"BOTH_COWER1_STOP\0".as_ptr() as _, value: 1374 }, //# cower stop
	StringIDTable { name: b"BOTH_SONICPAIN_START\0".as_ptr() as _, value: 1375 }, 
	StringIDTable { name: b"BOTH_SONICPAIN_HOLD\0".as_ptr() as _, value: 1376 }, 
	StringIDTable { name: b"BOTH_SONICPAIN_END\0".as_ptr() as _, value: 1377 }, 

	//new anim slots per Jarrod's request
	StringIDTable { name: b"BOTH_STAND10\0".as_ptr() as _, value: 1378 }, 
	StringIDTable { name: b"BOTH_STAND10_TALK1\0".as_ptr() as _, value: 1379 }, 
	StringIDTable { name: b"BOTH_STAND10_TALK2\0".as_ptr() as _, value: 1380 }, 
	StringIDTable { name: b"BOTH_STAND10TOSTAND1\0".as_ptr() as _, value: 1381 }, 

	StringIDTable { name: b"BOTH_STAND1_TALK1\0".as_ptr() as _, value: 1382 }, 
	StringIDTable { name: b"BOTH_STAND1_TALK2\0".as_ptr() as _, value: 1383 }, 
	StringIDTable { name: b"BOTH_STAND1_TALK3\0".as_ptr() as _, value: 1384 }, 

	StringIDTable { name: b"BOTH_SIT4\0".as_ptr() as _, value: 1385 }, 
	StringIDTable { name: b"BOTH_SIT5\0".as_ptr() as _, value: 1386 }, 
	StringIDTable { name: b"BOTH_SIT5_TALK1\0".as_ptr() as _, value: 1387 }, 
	StringIDTable { name: b"BOTH_SIT5_TALK2\0".as_ptr() as _, value: 1388 }, 
	StringIDTable { name: b"BOTH_SIT5_TALK3\0".as_ptr() as _, value: 1389 }, 

	StringIDTable { name: b"BOTH_SIT6\0".as_ptr() as _, value: 1390 }, 
	StringIDTable { name: b"BOTH_SIT7\0".as_ptr() as _, value: 1391 }, 
	//=================================================
	//ANIMS IN WHICH ONLY THE UPPER OBJECTS ARE IN MD3
	//=================================================
	//# #sep ENUM2STRING(TORSO_ WEAPON-RELATED
	StringIDTable { name: b"TORSO_DROPWEAP1\0".as_ptr() as _, value: 1392 }, //# Put weapon away
	StringIDTable { name: b"TORSO_DROPWEAP4\0".as_ptr() as _, value: 1393 }, //# Put weapon away
	StringIDTable { name: b"TORSO_RAISEWEAP1\0".as_ptr() as _, value: 1394 }, //# Draw Weapon
	StringIDTable { name: b"TORSO_RAISEWEAP4\0".as_ptr() as _, value: 1395 }, //# Draw Weapon
	StringIDTable { name: b"TORSO_WEAPONREADY1\0".as_ptr() as _, value: 1396 }, //# Ready to fire stun baton
	StringIDTable { name: b"TORSO_WEAPONREADY2\0".as_ptr() as _, value: 1397 }, //# Ready to fire one-handed blaster pistol
	StringIDTable { name: b"TORSO_WEAPONREADY3\0".as_ptr() as _, value: 1398 }, //# Ready to fire blaster rifle
	StringIDTable { name: b"TORSO_WEAPONREADY4\0".as_ptr() as _, value: 1399 }, //# Ready to fire sniper rifle
	StringIDTable { name: b"TORSO_WEAPONREADY10\0".as_ptr() as _, value: 1400 }, //# Ready to fire thermal det
	StringIDTable { name: b"TORSO_WEAPONIDLE2\0".as_ptr() as _, value: 1401 }, //# Holding one-handed blaster
	StringIDTable { name: b"TORSO_WEAPONIDLE3\0".as_ptr() as _, value: 1402 }, //# Holding blaster rifle
	StringIDTable { name: b"TORSO_WEAPONIDLE4\0".as_ptr() as _, value: 1403 }, //# Holding sniper rifle
	StringIDTable { name: b"TORSO_WEAPONIDLE10\0".as_ptr() as _, value: 1404 }, //# Holding thermal det

	//# #sep ENUM2STRING(TORSO_ USING NON-WEAPON OBJECTS

	//# #sep ENUM2STRING(TORSO_ MISC
	StringIDTable { name: b"TORSO_SURRENDER_START\0".as_ptr() as _, value: 1405 }, //# arms up
	StringIDTable { name: b"TORSO_SURRENDER_STOP\0".as_ptr() as _, value: 1406 }, //# arms back down
	StringIDTable { name: b"TORSO_CHOKING1\0".as_ptr() as _, value: 1407 }, //# TEMP

	StringIDTable { name: b"TORSO_HANDSIGNAL1\0".as_ptr() as _, value: 1408 }, 
	StringIDTable { name: b"TORSO_HANDSIGNAL2\0".as_ptr() as _, value: 1409 }, 
	StringIDTable { name: b"TORSO_HANDSIGNAL3\0".as_ptr() as _, value: 1410 }, 
	StringIDTable { name: b"TORSO_HANDSIGNAL4\0".as_ptr() as _, value: 1411 }, 
	StringIDTable { name: b"TORSO_HANDSIGNAL5\0".as_ptr() as _, value: 1412 }, 

	//=================================================
	//ANIMS IN WHICH ONLY THE LOWER OBJECTS ARE IN MD3
	//=================================================
	//# #sep Legs-only anims
	StringIDTable { name: b"LEGS_TURN1\0".as_ptr() as _, value: 1413 }, //# What legs do when you turn your lower body to match your upper body facing
	StringIDTable { name: b"LEGS_TURN2\0".as_ptr() as _, value: 1414 }, //# Leg turning from stand2
	StringIDTable { name: b"LEGS_LEAN_LEFT1\0".as_ptr() as _, value: 1415 }, //# Lean left
	StringIDTable { name: b"LEGS_LEAN_RIGHT1\0".as_ptr() as _, value: 1416 }, //# Lean Right
	StringIDTable { name: b"LEGS_CHOKING1\0".as_ptr() as _, value: 1417 }, //# TEMP
	StringIDTable { name: b"LEGS_LEFTUP1\0".as_ptr() as _, value: 1418 }, //# On a slope with left foot 4 higher than right
	StringIDTable { name: b"LEGS_LEFTUP2\0".as_ptr() as _, value: 1419 }, //# On a slope with left foot 8 higher than right
	StringIDTable { name: b"LEGS_LEFTUP3\0".as_ptr() as _, value: 1420 }, //# On a slope with left foot 12 higher than right
	StringIDTable { name: b"LEGS_LEFTUP4\0".as_ptr() as _, value: 1421 }, //# On a slope with left foot 16 higher than right
	StringIDTable { name: b"LEGS_LEFTUP5\0".as_ptr() as _, value: 1422 }, //# On a slope with left foot 20 higher than right
	StringIDTable { name: b"LEGS_RIGHTUP1\0".as_ptr() as _, value: 1423 }, //# On a slope with RIGHT foot 4 higher than left
	StringIDTable { name: b"LEGS_RIGHTUP2\0".as_ptr() as _, value: 1424 }, //# On a slope with RIGHT foot 8 higher than left
	StringIDTable { name: b"LEGS_RIGHTUP3\0".as_ptr() as _, value: 1425 }, //# On a slope with RIGHT foot 12 higher than left
	StringIDTable { name: b"LEGS_RIGHTUP4\0".as_ptr() as _, value: 1426 }, //# On a slope with RIGHT foot 16 higher than left
	StringIDTable { name: b"LEGS_RIGHTUP5\0".as_ptr() as _, value: 1427 }, //# On a slope with RIGHT foot 20 higher than left
	StringIDTable { name: b"LEGS_S1_LUP1\0".as_ptr() as _, value: 1428 }, 
	StringIDTable { name: b"LEGS_S1_LUP2\0".as_ptr() as _, value: 1429 }, 
	StringIDTable { name: b"LEGS_S1_LUP3\0".as_ptr() as _, value: 1430 }, 
	StringIDTable { name: b"LEGS_S1_LUP4\0".as_ptr() as _, value: 1431 }, 
	StringIDTable { name: b"LEGS_S1_LUP5\0".as_ptr() as _, value: 1432 }, 
	StringIDTable { name: b"LEGS_S1_RUP1\0".as_ptr() as _, value: 1433 }, 
	StringIDTable { name: b"LEGS_S1_RUP2\0".as_ptr() as _, value: 1434 }, 
	StringIDTable { name: b"LEGS_S1_RUP3\0".as_ptr() as _, value: 1435 }, 
	StringIDTable { name: b"LEGS_S1_RUP4\0".as_ptr() as _, value: 1436 }, 
	StringIDTable { name: b"LEGS_S1_RUP5\0".as_ptr() as _, value: 1437 }, 
	StringIDTable { name: b"LEGS_S3_LUP1\0".as_ptr() as _, value: 1438 }, 
	StringIDTable { name: b"LEGS_S3_LUP2\0".as_ptr() as _, value: 1439 }, 
	StringIDTable { name: b"LEGS_S3_LUP3\0".as_ptr() as _, value: 1440 }, 
	StringIDTable { name: b"LEGS_S3_LUP4\0".as_ptr() as _, value: 1441 }, 
	StringIDTable { name: b"LEGS_S3_LUP5\0".as_ptr() as _, value: 1442 }, 
	StringIDTable { name: b"LEGS_S3_RUP1\0".as_ptr() as _, value: 1443 }, 
	StringIDTable { name: b"LEGS_S3_RUP2\0".as_ptr() as _, value: 1444 }, 
	StringIDTable { name: b"LEGS_S3_RUP3\0".as_ptr() as _, value: 1445 }, 
	StringIDTable { name: b"LEGS_S3_RUP4\0".as_ptr() as _, value: 1446 }, 
	StringIDTable { name: b"LEGS_S3_RUP5\0".as_ptr() as _, value: 1447 }, 
	StringIDTable { name: b"LEGS_S4_LUP1\0".as_ptr() as _, value: 1448 }, 
	StringIDTable { name: b"LEGS_S4_LUP2\0".as_ptr() as _, value: 1449 }, 
	StringIDTable { name: b"LEGS_S4_LUP3\0".as_ptr() as _, value: 1450 }, 
	StringIDTable { name: b"LEGS_S4_LUP4\0".as_ptr() as _, value: 1451 }, 
	StringIDTable { name: b"LEGS_S4_LUP5\0".as_ptr() as _, value: 1452 }, 
	StringIDTable { name: b"LEGS_S4_RUP1\0".as_ptr() as _, value: 1453 }, 
	StringIDTable { name: b"LEGS_S4_RUP2\0".as_ptr() as _, value: 1454 }, 
	StringIDTable { name: b"LEGS_S4_RUP3\0".as_ptr() as _, value: 1455 }, 
	StringIDTable { name: b"LEGS_S4_RUP4\0".as_ptr() as _, value: 1456 }, 
	StringIDTable { name: b"LEGS_S4_RUP5\0".as_ptr() as _, value: 1457 }, 
	StringIDTable { name: b"LEGS_S5_LUP1\0".as_ptr() as _, value: 1458 }, 
	StringIDTable { name: b"LEGS_S5_LUP2\0".as_ptr() as _, value: 1459 }, 
	StringIDTable { name: b"LEGS_S5_LUP3\0".as_ptr() as _, value: 1460 }, 
	StringIDTable { name: b"LEGS_S5_LUP4\0".as_ptr() as _, value: 1461 }, 
	StringIDTable { name: b"LEGS_S5_LUP5\0".as_ptr() as _, value: 1462 }, 
	StringIDTable { name: b"LEGS_S5_RUP1\0".as_ptr() as _, value: 1463 }, 
	StringIDTable { name: b"LEGS_S5_RUP2\0".as_ptr() as _, value: 1464 }, 
	StringIDTable { name: b"LEGS_S5_RUP3\0".as_ptr() as _, value: 1465 }, 
	StringIDTable { name: b"LEGS_S5_RUP4\0".as_ptr() as _, value: 1466 }, 
	StringIDTable { name: b"LEGS_S5_RUP5\0".as_ptr() as _, value: 1467 }, 
	StringIDTable { name: b"LEGS_S6_LUP1\0".as_ptr() as _, value: 1468 }, 
	StringIDTable { name: b"LEGS_S6_LUP2\0".as_ptr() as _, value: 1469 }, 
	StringIDTable { name: b"LEGS_S6_LUP3\0".as_ptr() as _, value: 1470 }, 
	StringIDTable { name: b"LEGS_S6_LUP4\0".as_ptr() as _, value: 1471 }, 
	StringIDTable { name: b"LEGS_S6_LUP5\0".as_ptr() as _, value: 1472 }, 
	StringIDTable { name: b"LEGS_S6_RUP1\0".as_ptr() as _, value: 1473 }, 
	StringIDTable { name: b"LEGS_S6_RUP2\0".as_ptr() as _, value: 1474 }, 
	StringIDTable { name: b"LEGS_S6_RUP3\0".as_ptr() as _, value: 1475 }, 
	StringIDTable { name: b"LEGS_S6_RUP4\0".as_ptr() as _, value: 1476 }, 
	StringIDTable { name: b"LEGS_S6_RUP5\0".as_ptr() as _, value: 1477 }, 
	StringIDTable { name: b"LEGS_S7_LUP1\0".as_ptr() as _, value: 1478 }, 
	StringIDTable { name: b"LEGS_S7_LUP2\0".as_ptr() as _, value: 1479 }, 
	StringIDTable { name: b"LEGS_S7_LUP3\0".as_ptr() as _, value: 1480 }, 
	StringIDTable { name: b"LEGS_S7_LUP4\0".as_ptr() as _, value: 1481 }, 
	StringIDTable { name: b"LEGS_S7_LUP5\0".as_ptr() as _, value: 1482 }, 
	StringIDTable { name: b"LEGS_S7_RUP1\0".as_ptr() as _, value: 1483 }, 
	StringIDTable { name: b"LEGS_S7_RUP2\0".as_ptr() as _, value: 1484 }, 
	StringIDTable { name: b"LEGS_S7_RUP3\0".as_ptr() as _, value: 1485 }, 
	StringIDTable { name: b"LEGS_S7_RUP4\0".as_ptr() as _, value: 1486 }, 
	StringIDTable { name: b"LEGS_S7_RUP5\0".as_ptr() as _, value: 1487 }, 

	//New anim as per Jarrod's request
	StringIDTable { name: b"LEGS_TURN180\0".as_ptr() as _, value: 1488 }, 

	//======================================================
	//cinematic anims
	//======================================================
	//# #sep BOTH_ CINEMATIC-ONLY
	StringIDTable { name: b"BOTH_CIN_1\0".as_ptr() as _, value: 1489 }, //# Level specific cinematic 1
	StringIDTable { name: b"BOTH_CIN_2\0".as_ptr() as _, value: 1490 }, //# Level specific cinematic 2
	StringIDTable { name: b"BOTH_CIN_3\0".as_ptr() as _, value: 1491 }, //# Level specific cinematic 3
	StringIDTable { name: b"BOTH_CIN_4\0".as_ptr() as _, value: 1492 }, //# Level specific cinematic 4
	StringIDTable { name: b"BOTH_CIN_5\0".as_ptr() as _, value: 1493 }, //# Level specific cinematic 5
	StringIDTable { name: b"BOTH_CIN_6\0".as_ptr() as _, value: 1494 }, //# Level specific cinematic 6
	StringIDTable { name: b"BOTH_CIN_7\0".as_ptr() as _, value: 1495 }, //# Level specific cinematic 7
	StringIDTable { name: b"BOTH_CIN_8\0".as_ptr() as _, value: 1496 }, //# Level specific cinematic 8
	StringIDTable { name: b"BOTH_CIN_9\0".as_ptr() as _, value: 1497 }, //# Level specific cinematic 9
	StringIDTable { name: b"BOTH_CIN_10\0".as_ptr() as _, value: 1498 }, //# Level specific cinematic 10
	StringIDTable { name: b"BOTH_CIN_11\0".as_ptr() as _, value: 1499 }, //# Level specific cinematic 11
	StringIDTable { name: b"BOTH_CIN_12\0".as_ptr() as _, value: 1500 }, //# Level specific cinematic 12
	StringIDTable { name: b"BOTH_CIN_13\0".as_ptr() as _, value: 1501 }, //# Level specific cinematic 13
	StringIDTable { name: b"BOTH_CIN_14\0".as_ptr() as _, value: 1502 }, //# Level specific cinematic 14
	StringIDTable { name: b"BOTH_CIN_15\0".as_ptr() as _, value: 1503 }, //# Level specific cinematic 15
	StringIDTable { name: b"BOTH_CIN_16\0".as_ptr() as _, value: 1504 }, //# Level specific cinematic 16
	StringIDTable { name: b"BOTH_CIN_17\0".as_ptr() as _, value: 1505 }, //# Level specific cinematic 17
	StringIDTable { name: b"BOTH_CIN_18\0".as_ptr() as _, value: 1506 }, //# Level specific cinematic 18
	StringIDTable { name: b"BOTH_CIN_19\0".as_ptr() as _, value: 1507 }, //# Level specific cinematic 19
	StringIDTable { name: b"BOTH_CIN_20\0".as_ptr() as _, value: 1508 }, //# Level specific cinematic 20
	StringIDTable { name: b"BOTH_CIN_21\0".as_ptr() as _, value: 1509 }, //# Level specific cinematic 21
	StringIDTable { name: b"BOTH_CIN_22\0".as_ptr() as _, value: 1510 }, //# Level specific cinematic 22
	StringIDTable { name: b"BOTH_CIN_23\0".as_ptr() as _, value: 1511 }, //# Level specific cinematic 23
	StringIDTable { name: b"BOTH_CIN_24\0".as_ptr() as _, value: 1512 }, //# Level specific cinematic 24
	StringIDTable { name: b"BOTH_CIN_25\0".as_ptr() as _, value: 1513 }, //# Level specific cinematic 25

	StringIDTable { name: b"BOTH_CIN_26\0".as_ptr() as _, value: 1514 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_27\0".as_ptr() as _, value: 1515 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_28\0".as_ptr() as _, value: 1516 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_29\0".as_ptr() as _, value: 1517 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_30\0".as_ptr() as _, value: 1518 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_31\0".as_ptr() as _, value: 1519 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_32\0".as_ptr() as _, value: 1520 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_33\0".as_ptr() as _, value: 1521 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_34\0".as_ptr() as _, value: 1522 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_35\0".as_ptr() as _, value: 1523 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_36\0".as_ptr() as _, value: 1524 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_37\0".as_ptr() as _, value: 1525 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_38\0".as_ptr() as _, value: 1526 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_39\0".as_ptr() as _, value: 1527 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_40\0".as_ptr() as _, value: 1528 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_41\0".as_ptr() as _, value: 1529 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_42\0".as_ptr() as _, value: 1530 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_43\0".as_ptr() as _, value: 1531 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_44\0".as_ptr() as _, value: 1532 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_45\0".as_ptr() as _, value: 1533 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_46\0".as_ptr() as _, value: 1534 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_47\0".as_ptr() as _, value: 1535 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_48\0".as_ptr() as _, value: 1536 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_49\0".as_ptr() as _, value: 1537 }, //# Level specific cinematic
	StringIDTable { name: b"BOTH_CIN_50\0".as_ptr() as _, value: 1538 }, //# Level specific cinematic
										
	//must be terminated
	StringIDTable { name: std::ptr::null(), value: -1 }, // array terminator
];

#[cfg(any(target_os = "xbox", feature = "xbox"))]
pub extern "C" {
	pub static animTable: [StringIDTable; 1540];
}
