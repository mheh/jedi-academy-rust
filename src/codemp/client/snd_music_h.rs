//! Mechanical port of `codemp/client/snd_music.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int};

// Unported sound-local typedef dependency: `snd_local.h` has `#define sboolean int`.
pub type sboolean = c_int;

// if you change this enum, you MUST update the #defines below
/// `MusicState_e` (`snd_music.h`) — C `typedef enum`, stored/passed as `int`.
pub type MusicState_e = c_int;

//( eBGRNDTRACK_DATABEGIN )			// begin-label for FOR loops
//
pub const eBGRNDTRACK_EXPLORE: MusicState_e = 0; // for normal walking around
pub const eBGRNDTRACK_ACTION: MusicState_e = 1; // for excitement
pub const eBGRNDTRACK_BOSS: MusicState_e = 2; // (optional) for final encounter
pub const eBGRNDTRACK_DEATH: MusicState_e = 3; // (optional) death "flourish"
pub const eBGRNDTRACK_ACTIONTRANS0: MusicState_e = 4; // transition from action to explore
pub const eBGRNDTRACK_ACTIONTRANS1: MusicState_e = 5; // "
pub const eBGRNDTRACK_ACTIONTRANS2: MusicState_e = 6; // "
pub const eBGRNDTRACK_ACTIONTRANS3: MusicState_e = 7; // "
pub const eBGRNDTRACK_EXPLORETRANS0: MusicState_e = 8; // transition from explore to silence
pub const eBGRNDTRACK_EXPLORETRANS1: MusicState_e = 9; // "
pub const eBGRNDTRACK_EXPLORETRANS2: MusicState_e = 10; // "
pub const eBGRNDTRACK_EXPLORETRANS3: MusicState_e = 11; // "
//
//(	eBGRNDTRACK_DATAEND ),			// tracks from this point on are for logic or copies, do NOT free them.
//
pub const eBGRNDTRACK_NONDYNAMIC: MusicState_e = 12; // used for when music is just streaming, not part of dynamic stuff (used to be defined as same as explore entry, but this allows playing music in between 2 invokations of the same dynamic music without midleve reload, and also faster level transitioning if two consecutive dynamic sections use same DMS.DAT entries. Are you still reading this far?
pub const eBGRNDTRACK_SILENCE: MusicState_e = 13; // silence (more of a logic thing than an actual track at the moment)
pub const eBGRNDTRACK_FADE: MusicState_e = 14; // the xfade channel
//
pub const eBGRNDTRACK_NUMBEROF: MusicState_e = 15;

pub const iMAX_ACTION_TRANSITIONS: c_int = 4; // these can be increased easily enough, I just need to know about them
pub const iMAX_EXPLORE_TRANSITIONS: c_int = 4; //

pub const eBGRNDTRACK_DATABEGIN: MusicState_e = eBGRNDTRACK_EXPLORE; // label for FOR() loops (not in enum, else debugger shows in instead of the explore one unless I declare them backwards, which is gay)
pub const eBGRNDTRACK_DATAEND: MusicState_e = eBGRNDTRACK_NONDYNAMIC; // tracks from this point on are for logic or copies, do NOT free them.

pub const eBGRNDTRACK_FIRSTTRANSITION: MusicState_e = eBGRNDTRACK_ACTIONTRANS0; // used for "are we in transition mode" check
pub const eBGRNDTRACK_LASTTRANSITION: MusicState_e = eBGRNDTRACK_EXPLORETRANS3; //

unsafe extern "C" {
    pub fn Music_SetLevelName(psLevelName: *const c_char);
    pub fn Music_DynamicDataAvailable(psDynamicMusicLabel: *const c_char) -> sboolean;
    pub fn Music_GetFileNameForState(eMusicState: MusicState_e) -> *const c_char;
    pub fn Music_StateIsTransition(eMusicState: MusicState_e) -> sboolean;
    pub fn Music_StateCanBeInterrupted(
        eMusicState: MusicState_e,
        eProposedMusicState: MusicState_e,
    ) -> sboolean;
    pub fn Music_GetRandomEntryTime(eMusicState: MusicState_e) -> c_float;

    // C++ default arguments (`peTransition = NULL`, `pfNewTrackEntryTime = NULL`)
    // are not represented in Rust FFI.
    pub fn Music_AllowedToTransition(
        fPlayingTimeElapsed: c_float,
        eMusicState: MusicState_e,
        peTransition: *mut MusicState_e,
        pfNewTrackEntryTime: *mut c_float,
    ) -> sboolean;

    // C++ default argument `bDebugPrintQuery = qfalse` is not represented in Rust FFI.
    pub fn Music_BaseStateToString(
        eMusicState: MusicState_e,
        bDebugPrintQuery: sboolean,
    ) -> *const c_char;
}

//////////////// eof /////////////////
