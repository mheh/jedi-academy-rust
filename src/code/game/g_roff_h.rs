// Translation of oracle/code/game/g_roff.h

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_void};

// ROFF Defines
//-------------------
pub const ROFF_VERSION: c_int = 1;	// ver # for the (R)otation (O)bject (F)ile (F)ormat
pub const ROFF_VERSION2: c_int = 2;	// ver # for the (R)otation (O)bject (F)ile (F)ormat
pub const MAX_ROFFS: c_int = 32;	// hard coded number of max roffs per level, sigh..
pub const ROFF_SAMPLE_RATE: c_int = 20;	// 10hz


// Type definitions
pub type vec3_t = [f32; 3];

// Forward declaration stub for gentity_t
#[repr(C)]
pub struct gentity_t;


// ROFF Header file definition
//-------------------------------
#[repr(C)]
pub struct roff_hdr_s
{
	pub mHeader: [c_char; 4],		// should be "ROFF" (Rotation, Origin File Format)
	pub mVersion: c_long,
	pub mCount: f32,			// There isn't any reason for this to be anything other than an int, sigh...
		//
		//		Move - Rotate data follows....vec3_t delta_origin, vec3_t delta_rotation
		//
}

pub type roff_hdr_t = roff_hdr_s;


// ROFF move rotate data element
//--------------------------------
#[repr(C)]
pub struct move_rotate_s
{
	pub origin_delta: vec3_t,
	pub rotate_delta: vec3_t,

}

pub type move_rotate_t = move_rotate_s;

#[repr(C)]
pub struct roff_hdr2_s
//-------------------------------
{
	pub mHeader: [c_char; 4],				// should match roff_string defined above
	pub mVersion: c_long,				// version num, supported version defined above
	pub mCount: c_int,					// I think this is a float because of a limitation of the roff exporter
	pub mFrameRate: c_int,				// Frame rate the roff should be played at
	pub mNumNotes: c_int,				// number of notes (null terminated strings) after the roff data

}

pub type roff_hdr2_t = roff_hdr2_s;


#[repr(C)]
pub struct move_rotate2_s
//-------------------------------
{
	pub origin_delta: vec3_t,
	pub rotate_delta: vec3_t,
	pub mStartNote: c_int, pub mNumNotes: c_int,		// note track info

}

pub type move_rotate2_t = move_rotate2_s;


// a precached ROFF list
//-------------------------
#[repr(C)]
pub struct roff_list_s
{
	pub r#type: c_int,			// roff type number, 1-old, 2-new
	pub fileName: *mut c_char,		// roff filename
	pub frames: c_int,			// number of roff entries
	pub data: *mut c_void,			// delta move and rotate vector list
	pub mFrameTime: c_int,		// frame rate
	pub mLerp: c_int,			// Lerp rate (FPS)
	pub mNumNoteTracks: c_int,
	pub mNoteTrackIndexes: *mut *mut c_char,

}

pub type roff_list_t = roff_list_s;



extern "C" {
	pub static mut roffs: [roff_list_t; 32];
	pub static mut num_roffs: c_int;


	// Function prototypes
	//-------------------------
	pub fn G_LoadRoff( fileName: *const c_char ) -> c_int;
	pub fn G_Roff( ent: *mut gentity_t );
	pub fn G_SaveCachedRoffs();
	pub fn G_LoadCachedRoffs();
}
