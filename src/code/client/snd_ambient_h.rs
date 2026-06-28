#![allow(non_snake_case)]

use core::ffi::c_char;

// Includes

// pragma warning ( disable : 4786 )
// pragma warning ( disable : 4511 )	//copy constructor could not be gen
// pragma warning ( disable : 4512 )	//assign constructor could not be gen

// //these don't work because stl re-sets them
// //#pragma warning ( disable : 4663 )	//spcialize class
// //#pragma warning ( disable : 4018 )	//signed/unsigned
// pragma warning (disable:4503)	// decorated name length xceeded, name was truncated
// pragma warning (push, 3)	//go back down to 3 for the stl include

// #include "../qcommon/sstring.h"	// #include <string>
// #include <vector>
// #include <map>
// pragma warning (pop)
// pragma warning (disable:4503)	// decorated name length xceeded, name was truncated

// using namespace std;

// Defines

pub const AMBIENT_SET_FILENAME: &[u8] = b"sound/sound.txt";

pub const MAX_WAVES_PER_GROUP: usize = 8;
pub const MAX_SET_NAME_LENGTH: usize = 64;

// Enums

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum set_e {
	AS_SET_GENERAL,		//General sets
	AS_SET_LOCAL,		//Local sets (regional)
	AS_SET_BMODEL,		//Brush model sets (doors, plats, etc.)

	NUM_AS_SETS
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum setKeyword_e {
	SET_KEYWORD_TIMEBETWEENWAVES,
	SET_KEYWORD_SUBWAVES,
	SET_KEYWORD_LOOPEDWAVE,
	SET_KEYWORD_VOLRANGE,
	SET_KEYWORD_RADIUS,
	SET_KEYWORD_TYPE,
	SET_KEYWORD_AMSDIR,
	SET_KEYWORD_OUTDIR,
	SET_KEYWORD_BASEDIR,

	NUM_AS_KEYWORDS,
}

// Structures

//NOTENOTE: Was going to make this a class, but don't want to muck around
#[repr(C)]
pub struct ambientSet_s {
	pub name: [c_char; MAX_SET_NAME_LENGTH],
	pub loopedVolume: u8,
	pub time_start: u32,
	pub time_end: u32,
	pub volRange_start: u32,
	pub volRange_end: u32,
	pub numSubWaves: u8,
	pub subWaves: [i32; MAX_WAVES_PER_GROUP],
	pub loopedWave: i32,
	pub radius: i32,							//NOTENOTE: -1 is global
	pub masterVolume: i32,					//Used for fading ambient sets (not a byte to prevent wrapping)
	pub id: i32,								//Used for easier referencing of sets
	pub fadeTime: i32,						//When the fade was started on this set
}

pub type ambientSet_t = ambientSet_s;

pub type parseFunc_t = extern "C" fn(*mut ambientSet_t);

// Classes

//NOTENOTE: But this one should be a class because of all the mapping and internal data handling
pub struct CSetGroup {
	m_numSets: i32,
	m_ambientSets: *mut core::ffi::c_void,		// vector < ambientSet_t * >
	m_setMap: *mut core::ffi::c_void,			// map < sstring_t, ambientSet_t * >
}

impl CSetGroup {
	pub fn Init(&mut self) {
		self.Free();
	}

	pub fn Free(&mut self) {
		// External implementation
	}

	pub fn AddSet(&mut self, name: *const c_char) -> *mut ambientSet_t {
		// External implementation
		core::ptr::null_mut()
	}

	pub fn GetSet(&mut self, name: *const c_char) -> *mut ambientSet_t {
		// External implementation
		core::ptr::null_mut()
	}

	pub fn GetSet_id(&mut self, ID: i32) -> *mut ambientSet_t {
		// External implementation
		core::ptr::null_mut()
	}
}

// Prototypes

extern "C" {
	pub fn AS_Init();
	pub fn AS_Free();
	pub fn AS_ParseSets();
	pub fn AS_AddPrecacheEntry(name: *const c_char);

	pub fn S_UpdateAmbientSet(name: *const c_char, origin: *const f32);
	pub fn S_AddLocalSet(name: *const c_char, origin: *const f32, time: i32) -> i32;

	pub fn AS_GetBModelSound(name: *const c_char, stage: i32) -> i32;
}
