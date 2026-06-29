// Translated from oracle/code/game/characters.h

use core::ffi::{c_char, c_int};

// Stub for sfxHandle_t - audio handle type from engine
type sfxHandle_t = c_int;

#[repr(C)]
pub enum characters_e { // # characters_e
    //HazTeam Alpha
    //CHARACTER_MUNRO = 0,
    CHARACTER_FOSTER = 0,
    CHARACTER_TELSIA,
    CHARACTER_BIESSMAN,
    CHARACTER_CHANG,
    CHARACTER_CHELL,
    CHARACTER_JUROT,
    //HazTeam Beta
    CHARACTER_LAIRD,
    CHARACTER_KENN,
    CHARACTER_OVIEDO,
    CHARACTER_ODELL,
    CHARACTER_NELSON,
    CHARACTER_JAWORSKI,
    CHARACTER_CSATLOS,
    //Senior Crew
    CHARACTER_JANEWAY,
    CHARACTER_CHAKOTAY,
    CHARACTER_TUVOK,
    CHARACTER_TUVOKHAZ,
    CHARACTER_TORRES,
    CHARACTER_PARIS,
    CHARACTER_KIM,
    CHARACTER_DOCTOR,
    CHARACTER_SEVEN,
    CHARACTER_SEVENHAZ,
    CHARACTER_NEELIX,
    //Other Crew
    CHARACTER_PELLETIER,
    //Generic Crew
    CHARACTER_CREWMAN,
    //CHARACTER_ENSIGN,
    CHARACTER_LT,
    CHARACTER_COMM,
    CHARACTER_CAPT,
    CHARACTER_GENERIC1,
    CHARACTER_GENERIC2,
    CHARACTER_GENERIC3,
    CHARACTER_GENERIC4,
    //# #eol
    CHARACTER_NUM_CHARS,
}

pub type characters_t = characters_e;

#[repr(C)]
#[allow(non_snake_case)]
pub struct characterName_t {
    pub name: *mut c_char,
    pub sound: *mut c_char,
    pub soundIndex: sfxHandle_t,
}
