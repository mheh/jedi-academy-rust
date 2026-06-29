// Filename:-	snd_music.cpp
//
//  Stuff to parse in special x-fade music format and handle blending etc

//Anything above this #include will be ignored by the compiler
// #include "../server/exe_headers.h"

// #include "../qcommon/sstring.h"
// #include <algorithm>

// #ifdef _XBOX
// #include "snd_local_console.h"
// #include <xtl.h>
// #else
// #include "snd_local.h"
// #include "cl_mp3.h"
// #endif

// #include "snd_music.h"

use core::ffi::{c_int, c_char};
use std::collections::HashMap;
use std::ptr;

// LOCAL stubs for unported engine types
type sstring_t = String;
type qboolean = i32;
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;
type LPCSTR = *const c_char;
const MAX_QPATH_VALUE: usize = 256;

// LOCAL stubs for parse-related types
#[repr(C)]
struct CGPValue {
    _unused: u8,
}

#[repr(C)]
struct CGPGroup {
    _unused: u8,
}

#[repr(C)]
struct CGenericParser2 {
    _unused: u8,
}

// Extern declarations for engine functions
extern "C" {
    fn S_FileExists(psFilename: LPCSTR) -> qboolean;
    fn Com_Printf(format: LPCSTR, ...) -> ();
    fn FS_ReadFile(filename: LPCSTR, buffer: *mut *mut c_char) -> i32;
    fn FS_FreeFile(buffer: *mut c_char) -> ();
    fn Z_Malloc(size: usize, tag: i32, clear: qboolean) -> *mut c_char;
    fn Z_Free(ptr: *mut c_char) -> ();
    fn Q_strncpyz(dst: *mut c_char, src: LPCSTR, dstsize: usize) -> ();
    fn COM_SkipPath(path: *mut c_char) -> LPCSTR;
    fn va(format: LPCSTR, ...) -> LPCSTR;
    fn Q_fabs(f: f32) -> f32;
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn Q_stricmp(s1: LPCSTR, s2: LPCSTR) -> c_int;
    fn strlwr(str: *mut c_char) -> *mut c_char;
    fn rand() -> c_int;
}

#[cfg(target_os = "windows")]
extern "C" {
    fn Z_SetNewDeleteTemporary(bTemp: bool) -> ();
}

// Cvar definition
#[repr(C)]
struct cvar_t {
    name: LPCSTR,
    string: LPCSTR,
    resetString: LPCSTR,
    latched_string: LPCSTR,
    flags: c_int,
    modified: qboolean,
    modificationCount: c_int,
    value: f32,
    integer: c_int,
    min: f32,
    max: f32,
    next: *const cvar_t,
}

#[allow(non_upper_case_globals)]
static mut s_debugdynamic: *const cvar_t = ptr::null();

// String constants
const sKEY_MUSICFILES: &str = "musicfiles";
const sKEY_ENTRY: &str = "entry";
const sKEY_EXIT: &str = "exit";
const sKEY_MARKER: &str = "marker";
const sKEY_TIME: &str = "time";
const sKEY_NEXTFILE: &str = "nextfile";
const sKEY_NEXTMARK: &str = "nextmark";
const sKEY_LEVELMUSIC: &str = "levelmusic";
const sKEY_EXPLORE: &str = "explore";
const sKEY_ACTION: &str = "action";
const sKEY_BOSS: &str = "boss";
const sKEY_DEATH: &str = "death";
const sKEY_USES: &str = "uses";
const sKEY_USEBOSS: &str = "useboss";
const sKEY_PLACEHOLDER: &str = "placeholder";
const sFILENAME_DMS: &str = "ext_data/dms.dat";

#[repr(C)]
struct MusicExitPoint_t {
    sNextFile: sstring_t,
    sNextMark: sstring_t,  // blank if used for an explore piece, name of marker point to enter new file at
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MusicExitTime_t {
    fTime: f32,
    iExitPoint: c_int,
    // I'm defining this '<' operator so STL's sort algorithm will work
}

impl PartialEq for MusicExitTime_t {
    fn eq(&self, other: &Self) -> bool {
        (self.fTime - other.fTime).abs() < 1e-6
    }
}

impl PartialOrd for MusicExitTime_t {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fTime.partial_cmp(&other.fTime)
    }
}

impl Eq for MusicExitTime_t {}

impl Ord for MusicExitTime_t {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

// it's possible for all 3 of these to be empty if it's boss or death music
//
type MusicExitPoints_t = Vec<MusicExitPoint_t>;
type MusicExitTimes_t = Vec<MusicExitTime_t>;
type MusicEntryTimes_t = HashMap<sstring_t, f32>;  // key eg "marker1"

#[repr(C)]
struct MusicFile_t {
    sFileNameBase: sstring_t,
    MusicEntryTimes: MusicEntryTimes_t,
    MusicExitPoints: MusicExitPoints_t,
    MusicExitTimes: MusicExitTimes_t,
}

type MusicData_t = HashMap<sstring_t, MusicFile_t>;  // string is "explore", "action", "boss" etc

static mut MusicData: *mut MusicData_t = ptr::null_mut();

// there are now 2 of these, because of the new "uses" keyword...
//
// eg "kejim_base", formed from literal BSP name, but also used as dir name for music paths
static mut gsLevelNameForLoad: [u8; MAX_QPATH_VALUE] = [0; MAX_QPATH_VALUE];
// eg "kejim_base", formed from literal BSP name, but also used as dir name for music paths
static mut gsLevelNameForCompare: [u8; MAX_QPATH_VALUE] = [0; MAX_QPATH_VALUE];
// eg "kejim_base', special case for enabling boss music to come from a different dir - sigh....
static mut gsLevelNameForBossLoad: [u8; MAX_QPATH_VALUE] = [0; MAX_QPATH_VALUE];

// called from SV_SpawnServer, but before map load and music start etc.
//
// This just initialises the Lucas music structs so the background music player can interrogate them...
//
static mut gsLevelNameFromServer: [u8; MAX_QPATH_VALUE] = [0; MAX_QPATH_VALUE];

fn Music_Free() {
    #[cfg(target_os = "windows")]
    {
        // Prevents pending state changes from crashing the game after
        // level loads, but before new music data has been parsed.
        extern "C" {
            fn S_AvertMusicDisaster() -> ();
        }
        unsafe {
            S_AvertMusicDisaster();
        }
    }

    unsafe {
        if !MusicData.is_null() {
            #[cfg(target_os = "windows")]
            {
                drop(Box::from_raw(MusicData));
            }
            #[cfg(not(target_os = "windows"))]
            {
                (*MusicData).clear();
            }
        }
        MusicData = ptr::null_mut();
    }
}

// some sort of error in the music data...
//
fn Music_Parse_Error(psError: &str) {
    unsafe {
        if !MusicData.is_null() {
            (*MusicData).clear();
        }
    }
}

// something to just mention if interested...
//
fn Music_Parse_Warning(_psError: &str) {
    unsafe {
        let s_debugdynamic = s_debugdynamic;
        if !s_debugdynamic.is_null() && (*s_debugdynamic).integer != 0 {
            // output warning if debug flag is set
        }
    }
}

// the 2nd param here is pretty kludgy (sigh), and only used for testing for the "boss" type.
// Unfortunately two of the places that calls this doesn't have much other access to the state other than
//	a string, not an enum, so for those cases they only pass in BOSS or EXPLORE, so don't rely on it totally.
//
fn Music_BuildFileName(psFileNameBase: &str, eMusicState: MusicState_e) -> String {
    //HACK!
    if eMusicState == MusicState_e::eBGRNDTRACK_DEATH {
        return "music/death_music.mp3".to_string();
    }

    unsafe {
        let psDirName = if eMusicState == MusicState_e::eBGRNDTRACK_BOSS {
            std::ffi::CStr::from_ptr(gsLevelNameForBossLoad.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned()
        } else {
            std::ffi::CStr::from_ptr(gsLevelNameForLoad.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned()
        };

        format!("music/{}/{}.mp3", psDirName, psFileNameBase)
    }
}

// Music state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
enum MusicState_e {
    eBGRNDTRACK_EXPLORE = 0,
    eBGRNDTRACK_ACTION = 1,
    eBGRNDTRACK_BOSS = 2,
    eBGRNDTRACK_DEATH = 3,
    eBGRNDTRACK_SILENCE = 4,
    eBGRNDTRACK_ACTIONTRANS0 = 5,
    eBGRNDTRACK_ACTIONTRANS1 = 6,
    eBGRNDTRACK_ACTIONTRANS2 = 7,
    eBGRNDTRACK_ACTIONTRANS3 = 8,
    eBGRNDTRACK_EXPLORETRANS0 = 9,
    eBGRNDTRACK_EXPLORETRANS1 = 10,
    eBGRNDTRACK_EXPLORETRANS2 = 11,
    eBGRNDTRACK_EXPLORETRANS3 = 12,
    eBGRNDTRACK_FADE = 13,
}

const iMAX_EXPLORE_TRANSITIONS: usize = 4;
const iMAX_ACTION_TRANSITIONS: usize = 4;

const eBGRNDTRACK_FIRSTTRANSITION: MusicState_e = MusicState_e::eBGRNDTRACK_ACTIONTRANS0;
const eBGRNDTRACK_LASTTRANSITION: MusicState_e = MusicState_e::eBGRNDTRACK_EXPLORETRANS3;

// this MUST return NULL for non-base states unless doing debug-query
fn Music_BaseStateToString(eMusicState: MusicState_e, bDebugPrintQuery: qboolean) -> Option<&'static str> {
    match eMusicState {
        MusicState_e::eBGRNDTRACK_EXPLORE => Some("explore"),
        MusicState_e::eBGRNDTRACK_ACTION => Some("action"),
        MusicState_e::eBGRNDTRACK_BOSS => Some("boss"),
        MusicState_e::eBGRNDTRACK_SILENCE => Some("silence"),  // not used in this module, but snd_dma uses it now it's de-static'd
        MusicState_e::eBGRNDTRACK_DEATH => Some("death"),

        // info only, not map<> lookup keys (unlike above)...
        //
        MusicState_e::eBGRNDTRACK_ACTIONTRANS0 => if bDebugPrintQuery != 0 { Some("action_tr0") } else { None },
        MusicState_e::eBGRNDTRACK_ACTIONTRANS1 => if bDebugPrintQuery != 0 { Some("action_tr1") } else { None },
        MusicState_e::eBGRNDTRACK_ACTIONTRANS2 => if bDebugPrintQuery != 0 { Some("action_tr2") } else { None },
        MusicState_e::eBGRNDTRACK_ACTIONTRANS3 => if bDebugPrintQuery != 0 { Some("action_tr3") } else { None },
        MusicState_e::eBGRNDTRACK_EXPLORETRANS0 => if bDebugPrintQuery != 0 { Some("explore_tr0") } else { None },
        MusicState_e::eBGRNDTRACK_EXPLORETRANS1 => if bDebugPrintQuery != 0 { Some("explore_tr1") } else { None },
        MusicState_e::eBGRNDTRACK_EXPLORETRANS2 => if bDebugPrintQuery != 0 { Some("explore_tr2") } else { None },
        MusicState_e::eBGRNDTRACK_EXPLORETRANS3 => if bDebugPrintQuery != 0 { Some("explore_tr3") } else { None },
        MusicState_e::eBGRNDTRACK_FADE => if bDebugPrintQuery != 0 { Some("fade") } else { None },
    }
}

fn Music_ParseMusic(
    _Parser: &mut CGenericParser2,
    MusicData: *mut MusicData_t,
    _pgMusicFiles: *mut CGPGroup,
    psMusicName: &str,
    psMusicNameKey: &str,
    _eMusicState: MusicState_e,
) -> qboolean {
    let mut bReturn = qfalse;

    #[cfg(target_os = "windows")]
    unsafe {
        Z_SetNewDeleteTemporary(true);
    }

    let MusicFile = MusicFile_t {
        sFileNameBase: psMusicName.to_string(),
        MusicEntryTimes: HashMap::new(),
        MusicExitPoints: Vec::new(),
        MusicExitTimes: Vec::new(),
    };

    #[cfg(target_os = "windows")]
    unsafe {
        Z_SetNewDeleteTemporary(false);
    }

    // LOCAL: stub for parser methods - parser implementation would go here
    bReturn = qfalse;

    unsafe {
        if bReturn != qfalse {
            (*MusicData).insert(psMusicNameKey.to_string(), MusicFile);
        }
    }

    bReturn
}

// I only need this because GP2 can't cope with trailing whitespace (for !@#$%^'s sake!!!!)...
//
// (output buffer will always be just '\n' seperated, regardless of possible "\r\n" pairs)
//
// (remember to Z_Free() the returned char * when done with it!!!)
//
fn StripTrailingWhiteSpaceOnEveryLine(pText: *mut c_char) -> *mut c_char {
    #[cfg(target_os = "windows")]
    unsafe {
        Z_SetNewDeleteTemporary(true);
    }

    let mut strNewText = String::new();

    unsafe {
        let mut pText = pText;
        while *pText != 0 {
            let mut sOneLine = vec![0u8; 1024];  // BTO: was 16k

            // find end of line...
            //
            let mut pThisLineEnd = pText;
            while *pThisLineEnd != 0 && *pThisLineEnd != b'\r' as c_char && ((pThisLineEnd as usize - pText as usize) < 1023) {
                pThisLineEnd = pThisLineEnd.offset(1);
            }

            let iCharsToCopy = pThisLineEnd as usize - pText as usize;
            ptr::copy_nonoverlapping(pText as *const u8, sOneLine.as_mut_ptr(), iCharsToCopy);
            sOneLine[iCharsToCopy] = 0;
            pText = pText.offset(iCharsToCopy as isize);

            while *pText == b'\n' as c_char || *pText == b'\r' as c_char {
                pText = pText.offset(1);
            }

            // trim trailing...
            //
            loop {
                // Find string length
                let mut len = 0;
                while sOneLine[len] != 0 {
                    len += 1;
                }

                if len > 0 {
                    if sOneLine[len - 1] == b'\t' || sOneLine[len - 1] == b' ' {
                        sOneLine[len - 1] = 0;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let line_str = std::ffi::CStr::from_ptr(sOneLine.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned();
            strNewText.push_str(&line_str);
            strNewText.push('\n');
        }

        let newtext_cstr = std::ffi::CString::new(strNewText.as_str()).unwrap();
        let pNewText = Z_Malloc(newtext_cstr.as_bytes().len() + 1, 0, qfalse);
        ptr::copy_nonoverlapping(
            newtext_cstr.as_ptr() as *const u8,
            pNewText as *mut u8,
            newtext_cstr.as_bytes().len() + 1,
        );

        #[cfg(target_os = "windows")]
        {
            Z_SetNewDeleteTemporary(false);
        }

        pNewText
    }
}

fn Music_SetLevelName(psLevelName: *const c_char) {
    unsafe {
        let level_str = std::ffi::CStr::from_ptr(psLevelName).to_string_lossy();
        let bytes = level_str.as_bytes();
        let len = bytes.len().min(MAX_QPATH_VALUE - 1);
        gsLevelNameFromServer[..len].copy_from_slice(&bytes[..len]);
        gsLevelNameFromServer[len] = 0;
    }
}

fn Music_ParseLeveldata(_psLevelName: &str) -> qboolean {
    // Stub: full parser implementation would go here
    // This requires CGenericParser2 to be fully ported
    qfalse
}

// returns ptr to music file, or NULL for error/missing...
//
fn Music_GetBaseMusicFile(psMusicState: &str) -> Option<*mut MusicFile_t> {
    unsafe {
        if let Some(entry) = (*MusicData).get_mut(psMusicState) {
            return Some(entry as *mut MusicFile_t);
        }
    }
    None
}

fn Music_GetBaseMusicFile_Enum(eMusicState: MusicState_e) -> Option<*mut MusicFile_t> {
    if let Some(psMusicStateString) = Music_BaseStateToString(eMusicState, qfalse) {
        return Music_GetBaseMusicFile(psMusicStateString);
    }
    None
}

// where label is (eg) "kejim_base"...
//
fn Music_DynamicDataAvailable(psDynamicMusicLabel: *const c_char) -> qboolean {
    unsafe {
        let label_str = if !psDynamicMusicLabel.is_null() && *psDynamicMusicLabel != 0 {
            std::ffi::CStr::from_ptr(psDynamicMusicLabel)
                .to_string_lossy()
                .into_owned()
        } else {
            std::ffi::CStr::from_ptr(gsLevelNameFromServer.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned()
        };

        if !label_str.is_empty() {
            if Music_ParseLeveldata(&label_str) != qfalse {
                return if Music_GetBaseMusicFile_Enum(MusicState_e::eBGRNDTRACK_EXPLORE).is_some()
                    && Music_GetBaseMusicFile_Enum(MusicState_e::eBGRNDTRACK_ACTION).is_some()
                {
                    qtrue
                } else {
                    qfalse
                };
            }
        }

        qfalse
    }
}

fn Music_GetFileNameForState(eMusicState: MusicState_e) -> Option<String> {
    match eMusicState {
        MusicState_e::eBGRNDTRACK_EXPLORE
        | MusicState_e::eBGRNDTRACK_ACTION
        | MusicState_e::eBGRNDTRACK_BOSS
        | MusicState_e::eBGRNDTRACK_DEATH => {
            if let Some(pMusicFile) = Music_GetBaseMusicFile_Enum(eMusicState) {
                unsafe {
                    return Some(Music_BuildFileName(&(*pMusicFile).sFileNameBase, eMusicState));
                }
            }
        }

        MusicState_e::eBGRNDTRACK_ACTIONTRANS0
        | MusicState_e::eBGRNDTRACK_ACTIONTRANS1
        | MusicState_e::eBGRNDTRACK_ACTIONTRANS2
        | MusicState_e::eBGRNDTRACK_ACTIONTRANS3 => {
            if let Some(pMusicFile) = Music_GetBaseMusicFile_Enum(MusicState_e::eBGRNDTRACK_ACTION) {
                unsafe {
                    let iTransNum = (eMusicState as usize) - (MusicState_e::eBGRNDTRACK_ACTIONTRANS0 as usize);
                    if iTransNum < (*pMusicFile).MusicExitPoints.len() {
                        return Some(Music_BuildFileName(
                            &(*pMusicFile).MusicExitPoints[iTransNum].sNextFile,
                            eMusicState,
                        ));
                    }
                }
            }
        }

        MusicState_e::eBGRNDTRACK_EXPLORETRANS0
        | MusicState_e::eBGRNDTRACK_EXPLORETRANS1
        | MusicState_e::eBGRNDTRACK_EXPLORETRANS2
        | MusicState_e::eBGRNDTRACK_EXPLORETRANS3 => {
            if let Some(pMusicFile) = Music_GetBaseMusicFile_Enum(MusicState_e::eBGRNDTRACK_EXPLORE) {
                unsafe {
                    let iTransNum = (eMusicState as usize) - (MusicState_e::eBGRNDTRACK_EXPLORETRANS0 as usize);
                    if iTransNum < (*pMusicFile).MusicExitPoints.len() {
                        return Some(Music_BuildFileName(
                            &(*pMusicFile).MusicExitPoints[iTransNum].sNextFile,
                            eMusicState,
                        ));
                    }
                }
            }
        }

        _ => {
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                // debug output for unhandled case
            }
        }
    }

    None
}

fn Music_StateIsTransition(eMusicState: MusicState_e) -> qboolean {
    if (eMusicState as i32) >= (eBGRNDTRACK_FIRSTTRANSITION as i32)
        && (eMusicState as i32) <= (eBGRNDTRACK_LASTTRANSITION as i32)
    {
        qtrue
    } else {
        qfalse
    }
}

fn Music_StateCanBeInterrupted(eMusicState: MusicState_e, eProposedMusicState: MusicState_e) -> qboolean {
    // death music can interrupt anything...
    //
    if eProposedMusicState == MusicState_e::eBGRNDTRACK_DEATH {
        return qtrue;
    }
    //
    // ... and can't be interrupted once started...(though it will internally-switch to silence at the end, rather than loop)
    //
    if eMusicState == MusicState_e::eBGRNDTRACK_DEATH {
        return qfalse;
    }

    // boss music can interrupt anything (other than death, but that's already handled above)...
    //
    if eProposedMusicState == MusicState_e::eBGRNDTRACK_BOSS {
        return qtrue;
    }
    //
    // ... and can't be interrupted once started...
    //
    if eMusicState == MusicState_e::eBGRNDTRACK_BOSS {
        // ...except by silence (or death, but again, that's already handled above)
        //
        if eProposedMusicState == MusicState_e::eBGRNDTRACK_SILENCE {
            return qtrue;
        }

        return qfalse;
    }

    // action music can interrupt anything (after boss & death filters above)...
    //
    if eProposedMusicState == MusicState_e::eBGRNDTRACK_ACTION {
        return qtrue;
    }

    // nothing can interrupt a transition (after above filters)...
    //
    if Music_StateIsTransition(eMusicState) != qfalse {
        return qfalse;
    }

    // current state is therefore interruptable...
    //
    qtrue
}

// returns qtrue if music is allowed to transition out of current state, based on current play position...
// (doesn't bother returning final state after transition (eg action->transition->explore) becuase it's fairly obvious)
//
// supply:
//
// playing point in float seconds
// enum of track being queried
//
// get:
//
// enum of transition track to switch to
// float time of entry point of new track *after* transition
//
fn Music_AllowedToTransition(
    fPlayingTimeElapsed: f32,
    eMusicState: MusicState_e,
    peTransition: Option<&mut MusicState_e>,
    pfNewTrackEntryTime: Option<&mut f32>,
) -> qboolean {
    const fTimeEpsilon: f32 = 0.3;  // arb., how close we have to be to an exit point to take it.
                                     //		if set too high then music change is sloppy
                                     //		if set too low[/precise] then we might miss an exit if client fps is poor

    unsafe {
        if let Some(pMusicFile) = Music_GetBaseMusicFile_Enum(eMusicState) {
            if !(*pMusicFile).MusicExitTimes.is_empty() {
                // LOCAL: implement binary search equivalent to equal_range
                // For simplicity, iterate through the sorted times
                for exit_time in (*pMusicFile).MusicExitTimes.iter() {
                    if (exit_time.fTime - fPlayingTimeElapsed).abs() <= fTimeEpsilon {
                        // got an exit point!, work out feedback params...
                        //
                        let iExitPoint = exit_time.iExitPoint as usize;
                        //
                        // the two params to give back...
                        //
                        let mut eFeedBackTransition = MusicState_e::eBGRNDTRACK_EXPLORETRANS0;  // any old default
                        let mut fFeedBackNewTrackEntryTime = 0.0f32;
                        //
                        // check legality in case of crap data...
                        //
                        if iExitPoint < (*pMusicFile).MusicExitPoints.len() {
                            let ExitPoint = &(*pMusicFile).MusicExitPoints[iExitPoint];

                            match eMusicState {
                                MusicState_e::eBGRNDTRACK_EXPLORE => {
                                    // assert(iExitPoint < iMAX_EXPLORE_TRANSITIONS);  // already been checked, but sanity
                                    // assert(!ExitPoint.sNextMark.c_str()[0]);  // simple error checking, but harmless if tripped. explore transitions go to silence, hence no entry time for [silence] state after transition

                                    eFeedBackTransition = match iExitPoint {
                                        0 => MusicState_e::eBGRNDTRACK_EXPLORETRANS0,
                                        1 => MusicState_e::eBGRNDTRACK_EXPLORETRANS1,
                                        2 => MusicState_e::eBGRNDTRACK_EXPLORETRANS2,
                                        3 => MusicState_e::eBGRNDTRACK_EXPLORETRANS3,
                                        _ => MusicState_e::eBGRNDTRACK_EXPLORETRANS0,
                                    };
                                }

                                MusicState_e::eBGRNDTRACK_ACTION => {
                                    // assert(iExitPoint < iMAX_ACTION_TRANSITIONS);  // already been checked, but sanity

                                    // if there's an entry marker point defined...
                                    //
                                    if !ExitPoint.sNextMark.is_empty() {
                                        let explore_key = match Music_BaseStateToString(
                                            MusicState_e::eBGRNDTRACK_EXPLORE,
                                            qfalse,
                                        ) {
                                            Some(s) => s.to_string(),
                                            None => String::new(),
                                        };

                                        if let Some(MusicFile_Explore) = (*MusicData).get(&explore_key) {
                                            if let Some(entry_time) =
                                                MusicFile_Explore.MusicEntryTimes.get(&ExitPoint.sNextMark)
                                            {
                                                fFeedBackNewTrackEntryTime = *entry_time;
                                                eFeedBackTransition = match iExitPoint {
                                                    0 => MusicState_e::eBGRNDTRACK_ACTIONTRANS0,
                                                    1 => MusicState_e::eBGRNDTRACK_ACTIONTRANS1,
                                                    2 => MusicState_e::eBGRNDTRACK_ACTIONTRANS2,
                                                    3 => MusicState_e::eBGRNDTRACK_ACTIONTRANS3,
                                                    _ => MusicState_e::eBGRNDTRACK_ACTIONTRANS0,
                                                };
                                            } else {
                                                #[cfg(not(feature = "FINAL_BUILD"))]
                                                {
                                                    // unable to find entry marker - debug output would go here
                                                }
                                                return qfalse;
                                            }
                                        } else {
                                            #[cfg(not(feature = "FINAL_BUILD"))]
                                            {
                                                // unable to find explore version - debug output would go here
                                            }
                                            return qfalse;
                                        }
                                    } else {
                                        eFeedBackTransition = MusicState_e::eBGRNDTRACK_ACTIONTRANS0;
                                        fFeedBackNewTrackEntryTime = 0.0;  // already set to this, but FYI
                                    }
                                }

                                _ => {
                                    #[cfg(not(feature = "FINAL_BUILD"))]
                                    {
                                        // No code to transition from this music type - debug output
                                    }
                                    return qfalse;
                                }
                            }
                        } else {
                            #[cfg(not(feature = "FINAL_BUILD"))]
                            {
                                // Illegal exit point - debug output
                            }
                            return qfalse;
                        }

                        // feed back answers...
                        //
                        if let Some(trans) = peTransition {
                            *trans = eFeedBackTransition;
                        }

                        if let Some(time) = pfNewTrackEntryTime {
                            *time = fFeedBackNewTrackEntryTime;
                        }

                        return qtrue;
                    }
                }
            }
        }
    }

    qfalse
}

// typically used to get a (predefined) random entry point for the action music, but will work on any defined type with entry points,
//	defaults safely to 0.0f if no info available...
//
fn Music_GetRandomEntryTime(eMusicState: MusicState_e) -> f32 {
    unsafe {
        if let Some(state_str) = Music_BaseStateToString(eMusicState, qfalse) {
            if let Some(MusicFile) = (*MusicData).get(state_str) {
                if !MusicFile.MusicEntryTimes.is_empty() {
                    // Quake's random number generator isn't very good, so instead of this:
                    //
                    // int iRandomEntryNum = Q_irand(0, (MusicFile.MusicEntryTimes.size()-1) );
                    //
                    // ... I'll do this (ensuring we don't get the same result on two consecutive calls, but without while-loop)...
                    //
                    static mut iPrevRandomNumber: c_int = -1;
                    static mut iCallCount: c_int = 0;
                    iCallCount += 1;
                    let mut iRandomEntryNum =
                        ((rand() + iCallCount) % (MusicFile.MusicEntryTimes.len() as c_int)) as usize;
                    if iRandomEntryNum == iPrevRandomNumber as usize && MusicFile.MusicEntryTimes.len() > 1 {
                        iRandomEntryNum += 1;
                        iRandomEntryNum %= MusicFile.MusicEntryTimes.len();
                    }
                    iPrevRandomNumber = iRandomEntryNum as c_int;

                    for (idx, (_key, value)) in MusicFile.MusicEntryTimes.iter().enumerate() {
                        if idx == iRandomEntryNum {
                            return *value;
                        }
                    }
                }
            }
        }
    }

    0.0
}

// info only, used in "soundinfo" command...
//
fn Music_GetLevelSetName() -> String {
    unsafe {
        let level_load_str = std::ffi::CStr::from_ptr(gsLevelNameForLoad.as_ptr() as *const c_char)
            .to_string_lossy()
            .into_owned();
        let level_compare_str = std::ffi::CStr::from_ptr(gsLevelNameForCompare.as_ptr() as *const c_char)
            .to_string_lossy()
            .into_owned();

        if level_compare_str != level_load_str {
            // music remap via USES command...
            //
            return format!("{} -> {}", level_compare_str, level_load_str);
        }

        level_load_str
    }
}

///////////////// eof /////////////////////
