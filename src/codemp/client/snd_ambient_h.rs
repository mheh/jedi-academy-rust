//! Mechanical port of `codemp/client/snd_ambient.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};
use core::marker::PhantomData;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::codemp::qcommon::sstring_h::sstring_t;

// Includes

// #pragma warning ( disable : 4786 )
// #pragma warning ( disable : 4511 )	//copy constructor could not be gen
// #pragma warning ( disable : 4512 )	//assign constructor could not be gen

//these don't work because stl re-sets them
//#pragma warning ( disable : 4663 )	//spcialize class
//#pragma warning ( disable : 4018 )	//signed/unsigned
// #pragma warning (disable:4503)	// decorated name length xceeded, name was truncated
// #pragma warning (push, 3)	//go back down to 3 for the stl include

// #include "../qcommon/sstring.h"	// #include <string>
// #include <vector>
// #include <map>
// #pragma warning (pop)
// #pragma warning (disable:4503)	// decorated name length xceeded, name was truncated

// using namespace std;

// Defines

pub const AMBIENT_SET_FILENAME: &[u8; 16] = b"sound/sound.txt\0";

pub const MAX_WAVES_PER_GROUP: c_int = 8;
pub const MAX_SET_NAME_LENGTH: c_int = 64;

// Enums

/// `set_e` (`snd_ambient.h`) — C `enum`, stored/passed as `int`.
pub type set_e = c_int;

pub const AS_SET_GENERAL: set_e = 0; //General sets
pub const AS_SET_LOCAL: set_e = 1; //Local sets (regional)
pub const AS_SET_BMODEL: set_e = 2; //Brush model sets (doors, plats, etc.)

pub const NUM_AS_SETS: set_e = 3;

/// `setKeyword_e` (`snd_ambient.h`) — C `enum`, stored/passed as `int`.
pub type setKeyword_e = c_int;

pub const SET_KEYWORD_TIMEBETWEENWAVES: setKeyword_e = 0;
pub const SET_KEYWORD_SUBWAVES: setKeyword_e = 1;
pub const SET_KEYWORD_LOOPEDWAVE: setKeyword_e = 2;
pub const SET_KEYWORD_VOLRANGE: setKeyword_e = 3;
pub const SET_KEYWORD_RADIUS: setKeyword_e = 4;
pub const SET_KEYWORD_TYPE: setKeyword_e = 5;
pub const SET_KEYWORD_AMSDIR: setKeyword_e = 6;
pub const SET_KEYWORD_OUTDIR: setKeyword_e = 7;
pub const SET_KEYWORD_BASEDIR: setKeyword_e = 8;

pub const NUM_AS_KEYWORDS: setKeyword_e = 9;

// Structures

//NOTENOTE: Was going to make this a class, but don't want to muck around
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ambientSet_t {
    pub name: [c_char; MAX_SET_NAME_LENGTH as usize],
    pub loopedVolume: c_uchar,
    pub time_start: c_uint,
    pub time_end: c_uint,
    pub volRange_start: c_uint,
    pub volRange_end: c_uint,
    pub numSubWaves: c_uchar,
    pub subWaves: [c_int; MAX_WAVES_PER_GROUP as usize],
    pub loopedWave: c_int,
    pub radius: c_int,       //NOTENOTE: -1 is global
    pub masterVolume: c_int, //Used for fading ambient sets (not a byte to prevent wrapping)
    pub id: c_int,           //Used for easier referencing of sets
    pub fadeTime: c_int,     //When the fade was started on this set
}

const _: () = assert!(core::mem::size_of::<ambientSet_t>() == 140);
const _: () = assert!(core::mem::align_of::<ambientSet_t>() == 4);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, loopedVolume) == 64);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, time_start) == 68);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, numSubWaves) == 84);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, subWaves) == 88);
const _: () = assert!(core::mem::offset_of!(ambientSet_t, fadeTime) == 136);

pub type parseFunc_t = Option<unsafe extern "C" fn(*mut ambientSet_t)>;

// Classes

#[repr(C)]
pub struct std_vector_ambientSet_ptr {
    _opaque: [usize; 3],
}

#[repr(C)]
pub struct std_map_sstring_t_ambientSet_ptr {
    _opaque: *mut c_void,
    _key: PhantomData<sstring_t>,
    _value: PhantomData<*mut ambientSet_t>,
}

//NOTENOTE: But this one should be a class because of all the mapping and internal data handling
#[repr(C)]
pub struct CSetGroup {
    pub m_numSets: c_int,
    pub m_ambientSets: *mut std_vector_ambientSet_ptr,
    pub m_setMap: *mut std_map_sstring_t_ambientSet_ptr,
}

impl CSetGroup {
    pub unsafe fn Init(&mut self) {
        unsafe {
            self.Free();
        }
    }

    pub unsafe fn Free(&mut self) {
        unsafe {
            CSetGroup_Free(self as *mut CSetGroup);
        }
    }

    pub unsafe fn AddSet(&mut self, name: *const c_char) -> *mut ambientSet_t {
        unsafe { CSetGroup_AddSet(self as *mut CSetGroup, name) }
    }

    pub unsafe fn GetSet(&mut self, name: *const c_char) -> *mut ambientSet_t {
        unsafe { CSetGroup_GetSet_name(self as *mut CSetGroup, name) }
    }

    pub unsafe fn GetSet_ID(&mut self, ID: c_int) -> *mut ambientSet_t {
        unsafe { CSetGroup_GetSet_ID(self as *mut CSetGroup, ID) }
    }
}

// Unported shared typedef dependency from the original sound headers.
pub type sfxHandle_t = c_int;

unsafe extern "C" {
    pub fn CSetGroup_CSetGroup(self_: *mut CSetGroup);
    pub fn CSetGroup_destructor(self_: *mut CSetGroup);

    pub fn CSetGroup_Free(self_: *mut CSetGroup);

    pub fn CSetGroup_AddSet(self_: *mut CSetGroup, name: *const c_char) -> *mut ambientSet_t;

    pub fn CSetGroup_GetSet_name(self_: *mut CSetGroup, name: *const c_char) -> *mut ambientSet_t;
    pub fn CSetGroup_GetSet_ID(self_: *mut CSetGroup, ID: c_int) -> *mut ambientSet_t;
}

// Prototypes

unsafe extern "C" {
    pub fn AS_Init();
    pub fn AS_Free();
    pub fn AS_ParseSets();
    pub fn AS_AddPrecacheEntry(name: *const c_char);

    pub fn S_UpdateAmbientSet(name: *const c_char, origin: vec3_t);
    pub fn S_AddLocalSet(
        name: *const c_char,
        listener_origin: vec3_t,
        origin: vec3_t,
        entID: c_int,
        time: c_int,
    ) -> c_int;

    pub fn AS_GetBModelSound(name: *const c_char, stage: c_int) -> sfxHandle_t;
}
