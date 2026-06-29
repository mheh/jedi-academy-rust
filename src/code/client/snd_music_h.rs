// Filename:-	snd_music.h
//
//

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

// Local type stub - qboolean is typically c_int in Quake engines
pub type qboolean = c_int;

// if you change this enum, you MUST update the #defines below
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum MusicState_e {
	//( eBGRNDTRACK_DATABEGIN )			// begin-label for FOR loops
	//
	eBGRNDTRACK_EXPLORE = 0,		// for normal walking around
	eBGRNDTRACK_ACTION,				// for excitement
	eBGRNDTRACK_BOSS,				// (optional) for final encounter
	eBGRNDTRACK_DEATH,				// (optional) death "flourish"
	eBGRNDTRACK_ACTIONTRANS0,		// transition from action to explore
	eBGRNDTRACK_ACTIONTRANS1,		// "
	eBGRNDTRACK_ACTIONTRANS2,		// "
	eBGRNDTRACK_ACTIONTRANS3,		// "
	eBGRNDTRACK_EXPLORETRANS0,		// transition from explore to silence
	eBGRNDTRACK_EXPLORETRANS1,		// "
	eBGRNDTRACK_EXPLORETRANS2,		// "
	eBGRNDTRACK_EXPLORETRANS3,		// "
	//
	//(	eBGRNDTRACK_DATAEND ),			// tracks from this point on are for logic or copies, do NOT free them.
	//
	eBGRNDTRACK_NONDYNAMIC,			// used for when music is just streaming, not part of dynamic stuff (used to be defined as same as explore entry, but this allows playing music in between 2 invokations of the same dynamic music without midleve reload, and also faster level transitioning if two consecutive dynamic sections use same DMS.DAT entries. Are you still reading this far?
	eBGRNDTRACK_SILENCE,			// silence (more of a logic thing than an actual track at the moment)
	eBGRNDTRACK_FADE,				// the xfade channel
	//
	eBGRNDTRACK_NUMBEROF

}

pub const iMAX_ACTION_TRANSITIONS: c_int = 4;	// these can be increased easily enough, I just need to know about them
pub const iMAX_EXPLORE_TRANSITIONS: c_int = 4;	//

pub const eBGRNDTRACK_DATABEGIN: c_int = 0;	// label for FOR() loops (not in enum, else debugger shows in instead of the explore one unless I declare them backwards, which is gay)
pub const eBGRNDTRACK_DATAEND: c_int = 12; // tracks from this point on are for logic or copies, do NOT free them.

pub const eBGRNDTRACK_FIRSTTRANSITION: c_int = 4;	// used for "are we in transition mode" check
pub const eBGRNDTRACK_LASTTRANSITION: c_int = 11;	//


extern "C" {
	pub fn Music_SetLevelName(psLevelName: *const c_char);
	pub fn Music_DynamicDataAvailable(psDynamicMusicLabel: *const c_char) -> qboolean;
	pub fn Music_GetFileNameForState(eMusicState: MusicState_e) -> *const c_char;
	pub fn Music_StateIsTransition(eMusicState: MusicState_e) -> qboolean;
	pub fn Music_StateCanBeInterrupted(eMusicState: MusicState_e, eProposedMusicState: MusicState_e) -> qboolean;
	pub fn Music_GetRandomEntryTime(eMusicState: MusicState_e) -> f32;

	// #ifdef MP3STUFF_KNOWN
	pub fn Music_AllowedToTransition(fPlayingTimeElapsed: f32, eMusicState: MusicState_e, peTransition: *mut MusicState_e, pfNewTrackEntryTime: *mut f32) -> qboolean;
	// #endif

	pub fn Music_BaseStateToString(eMusicState: MusicState_e, bDebugPrintQuery: qboolean) -> *const c_char;
}


//////////////// eof /////////////////

