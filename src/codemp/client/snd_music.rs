// Filename:-	snd_music.cpp
//
//  Stuff to parse in special x-fade music format and handle blending etc

// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #include "../game/q_shared.h"
// #include "../qcommon/sstring.h"

// pragma warning ( disable : 4663 )	//spcialize class
// pragma warning( push, 3 )
// #include <algorithm>
// pragma warning (pop)

// #ifdef _XBOX
// #include "snd_local_console.h"
// #include <xtl.h>
// #else
// #include "snd_local.h"
// //#include "snd_mp3.h"
// #endif

// #include "snd_music.h"
// #include "snd_ambient.h"

// #include "../qcommon/GenericParser2.h"

use core::ffi::c_int;
use std::collections::HashMap;

// Local stubs for external dependencies not yet ported
// These are opaque types that will be linked from C
extern "C" {
    fn S_FileExists(psFilename: *const i8) -> i32; // sboolean

    #[cfg(target_os = "xbox")]
    fn Z_SetNewDeleteTemporary(bTemp: bool);

    fn Com_Printf(format: *const i8, ...);
    fn FS_ReadFile(filename: *const i8, buffer: *mut *mut i8) -> c_int;
    fn FS_FreeFile(buffer: *mut i8);
    fn Z_Malloc(size: usize, tag: c_int, bZero: i32) -> *mut i8;
    fn Z_Free(ptr: *mut i8);
    fn Q_fabs(x: f32) -> f32;
    fn Q_stricmp(s1: *const i8, s2: *const i8) -> c_int;
    fn Q_strncpyz(dst: *mut i8, src: *const i8, len: usize);
    fn COM_SkipPath(path: *mut i8) -> *const i8;
    fn strlwr(s: *mut i8) -> *mut i8;
    fn strlen(s: *const i8) -> usize;
    fn strncpy(dst: *mut i8, src: *const i8, len: usize) -> *mut i8;
    fn strcpy(dst: *mut i8, src: *const i8) -> *mut i8;
    fn strcmp(s1: *const i8, s2: *const i8) -> c_int;
    fn strncmp(s1: *const i8, s2: *const i8, len: usize) -> c_int;
}

// Opaque types for external structures
#[repr(C)]
pub struct CGenericParser2 {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CGPGroup {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CGPValue {
    _private: [u8; 0],
}

// sstring_t appears to be a C++ string class; we'll use a simple wrapper
#[repr(C)]
pub struct sstring_t {
    _private: [u8; 0],
}

// Constants for max path and other sizes
const MAX_QPATH: usize = 256;

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

const sKEY_PLACEHOLDER: &str = "placeholder"; // ignore these

const sFILENAME_DMS: &str = "ext_data/dms.dat";

// MUSIC_PARSE_ERROR only use during parse, not run-time use, and bear in mid that data is zapped after error message, so exit any loops immediately
// MUSIC_PARSE_WARNING

// Enum for music states (from snd_music.h, assumed)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MusicState_e {
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
    eBGRNDTRACK_FIRSTTRANSITION = 5, // eBGRNDTRACK_ACTIONTRANS0
    eBGRNDTRACK_LASTTRANSITION = 12,  // eBGRNDTRACK_EXPLORETRANS3
}

const iMAX_EXPLORE_TRANSITIONS: usize = 4;
const iMAX_ACTION_TRANSITIONS: usize = 4;

#[repr(C)]
pub struct MusicExitPoint_t {
    pub sNextFile: String,
    pub sNextMark: String, // blank if used for an explore piece, name of marker point to enter new file at
}

// I'm defining this '<' operator so STL's sort algorithm will work
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MusicExitTime_t {
    pub fTime: f32,
    pub iExitPoint: c_int,
}

impl PartialEq for MusicExitTime_t {
    fn eq(&self, other: &Self) -> bool {
        (self.fTime - other.fTime).abs() < f32::EPSILON
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
        self.fTime.partial_cmp(&other.fTime).unwrap_or(std::cmp::Ordering::Equal)
    }
}

// it's possible for all 3 of these to be empty if it's boss or death music
//
// typedef vector	<MusicExitPoint_t>	MusicExitPoints_t;
// typedef vector	<MusicExitTime_t>	MusicExitTimes_t;
// typedef map		<sstring_t, float>	MusicEntryTimes_t;	// key eg "marker1"

pub type MusicExitPoints_t = Vec<MusicExitPoint_t>;
pub type MusicExitTimes_t = Vec<MusicExitTime_t>;
pub type MusicEntryTimes_t = HashMap<String, f32>; // key eg "marker1"

#[repr(C)]
pub struct MusicFile_t {
    pub sFileNameBase: String,
    pub MusicEntryTimes: MusicEntryTimes_t,
    pub MusicExitPoints: MusicExitPoints_t,
    pub MusicExitTimes: MusicExitTimes_t,
}

pub type MusicData_t = HashMap<String, MusicFile_t>; // string is "explore", "action", "boss" etc

// there are now 2 of these, because of the new "uses" keyword...
//
static mut MusicData: Option<Box<MusicData_t>> = None;

// there are now 2 of these, because of the new "uses" keyword...
//
static mut gsLevelNameForLoad: String = String::new(); // eg "kejim_base", formed from literal BSP name, but also used as dir name for music paths
static mut gsLevelNameForCompare: String = String::new(); // eg "kejim_base", formed from literal BSP name, but also used as dir name for music paths
static mut gsLevelNameForBossLoad: String = String::new(); // eg "kejim_base', special case for enabling boss music to come from a different dir - sigh....

pub fn Music_Free() {
    unsafe {
        MusicData = None;
    }
}

// some sort of error in the music data...
//
unsafe fn Music_Parse_Error(psError: *const i8) {
    #[cfg(not(feature = "FINAL_BUILD"))]
    {
        Com_Printf(
            b"Error parsing music data ( in \"%s\" ):\n%s\n\0".as_ptr() as *const i8,
            sFILENAME_DMS.as_ptr(),
            psError,
        );
    }

    if let Some(ref mut data) = MusicData {
        data.clear();
    }
}

// something to just mention if interested...
//
unsafe fn Music_Parse_Warning(psError: *const i8) {
    #[cfg(not(feature = "FINAL_BUILD"))]
    {
        Com_Printf(
            b"%s\0".as_ptr() as *const i8,
            psError,
        );
    }
}

// the 2nd param here is pretty kludgy (sigh), and only used for testing for the "boss" type.
// Unfortunately two of the places that calls this doesn't have much other access to the state other than
//	a string, not an enum, so for those cases they only pass in BOSS or EXPLORE, so don't rely on it totally.
//
unsafe fn Music_BuildFileName(psFileNameBase: *const i8, eMusicState: MusicState_e) -> String {
    static mut sFileName: String = String::new();

    // HACK!
    if eMusicState == MusicState_e::eBGRNDTRACK_DEATH {
        return "music/death_music.mp3".to_string();
    }

    let psDirName = if eMusicState == MusicState_e::eBGRNDTRACK_BOSS {
        &gsLevelNameForBossLoad
    } else {
        &gsLevelNameForLoad
    };

    // SAFETY: psFileNameBase is expected to be a valid C string
    let file_base = std::ffi::CStr::from_ptr(psFileNameBase)
        .to_string_lossy()
        .into_owned();

    sFileName = format!("music/{}/{}.mp3", psDirName, file_base);
    sFileName
}

// this MUST return NULL for non-base states unless doing debug-query
pub unsafe fn Music_BaseStateToString(eMusicState: MusicState_e, bDebugPrintQuery: bool) -> Option<&'static str> {
    match eMusicState {
        MusicState_e::eBGRNDTRACK_EXPLORE => Some("explore"),
        MusicState_e::eBGRNDTRACK_ACTION => Some("action"),
        MusicState_e::eBGRNDTRACK_BOSS => Some("boss"),
        MusicState_e::eBGRNDTRACK_SILENCE => Some("silence"), // not used in this module, but snd_dma uses it now it's de-static'd
        MusicState_e::eBGRNDTRACK_DEATH => Some("death"),

        // info only, not map<> lookup keys (unlike above)...
        //
        MusicState_e::eBGRNDTRACK_ACTIONTRANS0 => {
            if bDebugPrintQuery {
                Some("action_tr0")
            } else {
                None
            }
        }
        MusicState_e::eBGRNDTRACK_ACTIONTRANS1 => {
            if bDebugPrintQuery {
                Some("action_tr1")
            } else {
                None
            }
        }
        MusicState_e::eBGRNDTRACK_ACTIONTRANS2 => {
            if bDebugPrintQuery {
                Some("action_tr2")
            } else {
                None
            }
        }
        MusicState_e::eBGRNDTRACK_ACTIONTRANS3 => {
            if bDebugPrintQuery {
                Some("action_tr3")
            } else {
                None
            }
        }
        MusicState_e::eBGRNDTRACK_EXPLORETRANS0 => {
            if bDebugPrintQuery {
                Some("explore_tr0")
            } else {
                None
            }
        }
        MusicState_e::eBGRNDTRACK_EXPLORETRANS1 => {
            if bDebugPrintQuery {
                Some("explore_tr1")
            } else {
                None
            }
        }
        MusicState_e::eBGRNDTRACK_EXPLORETRANS2 => {
            if bDebugPrintQuery {
                Some("explore_tr2")
            } else {
                None
            }
        }
        MusicState_e::eBGRNDTRACK_EXPLORETRANS3 => {
            if bDebugPrintQuery {
                Some("explore_tr3")
            } else {
                None
            }
        }
        MusicState_e::eBGRNDTRACK_FADE => {
            if bDebugPrintQuery {
                Some("fade")
            } else {
                None
            }
        }
        _ => None,
    }
}

unsafe fn Music_ParseMusic(
    Parser: *mut CGenericParser2,
    MusicData: *mut MusicData_t,
    pgMusicFiles: *mut CGPGroup,
    psMusicName: *const i8,
    psMusicNameKey: *const i8,
    eMusicState: MusicState_e,
) -> i32 {
    let mut bReturn: i32 = 0; // qfalse

    #[cfg(target_os = "xbox")]
    Z_SetNewDeleteTemporary(true);

    let mut MusicFile = MusicFile_t {
        sFileNameBase: String::new(),
        MusicEntryTimes: HashMap::new(),
        MusicExitPoints: Vec::new(),
        MusicExitTimes: Vec::new(),
    };

    #[cfg(target_os = "xbox")]
    Z_SetNewDeleteTemporary(false);

    let music_name_cstr = std::ffi::CStr::from_ptr(psMusicName).to_string_lossy().into_owned();
    let music_name_key_cstr = std::ffi::CStr::from_ptr(psMusicNameKey).to_string_lossy().into_owned();

    // Note: Cannot actually call FindSubGroup without proper bindings; this is a structural placeholder
    let pgMusicFile: *mut CGPGroup = std::ptr::null_mut(); // Would be pgMusicFiles->FindSubGroup(psMusicName);

    if !pgMusicFile.is_null() {
        // read subgroups...
        //
        let mut bEntryFound: i32 = 0; // qfalse
        let mut bExitFound: i32 = 0; // qfalse
        //
        // (read entry points first, so I can check exit points aren't too close in time)
        //
        // CGPGroup *pEntryGroup = pgMusicFile->FindSubGroup(sKEY_ENTRY);
        let pEntryGroup: *mut CGPGroup = std::ptr::null_mut(); // Placeholder

        if !pEntryGroup.is_null() {
            // read entry points...
            //
            // This would iterate through pEntryGroup->GetPairs()
            // For now, placeholder since we can't call C++ methods directly
        }

        // for (CGPGroup *pGroup = pgMusicFile->GetSubGroups(); pGroup; pGroup = pGroup->GetNext())
        // {
        //     ... processing logic ...
        // }

        // for now, assume everything was ok unless some obvious things are missing...
        //
        bReturn = 1; // qtrue

        if eMusicState != MusicState_e::eBGRNDTRACK_BOSS && eMusicState != MusicState_e::eBGRNDTRACK_DEATH { // boss & death pieces can omit entry/exit stuff
            if bEntryFound == 0 {
                Music_Parse_Error(b"Unable to find subgroup in group\0".as_ptr() as *const i8);
                bReturn = 0; // qfalse
            }
            if bExitFound == 0 {
                Music_Parse_Error(b"Unable to find subgroup in group\0".as_ptr() as *const i8);
                bReturn = 0; // qfalse
            }
        }
    } else {
        Music_Parse_Error(b"Unable to find musicfiles entry\0".as_ptr() as *const i8);
    }

    if bReturn != 0 {
        MusicFile.sFileNameBase = music_name_cstr;
        (*MusicData).insert(music_name_key_cstr, MusicFile);
    }

    bReturn
}

// I only need this because GP2 can't cope with trailing whitespace (for !@#$%^'s sake!!!!)...
//
// (output buffer will always be just '\n' seperated, regardless of possible "\r\n" pairs)
//
// (remember to Z_Free() the returned char * when done with it!!!)
//
unsafe fn StripTrailingWhiteSpaceOnEveryLine(pText: *mut i8) -> *mut i8 {
    #[cfg(target_os = "xbox")]
    Z_SetNewDeleteTemporary(true);

    let mut strNewText = String::new();

    let mut p = pText;
    while *p != 0 {
        const SONE_LINE_SIZE: usize = 1024; // BTO: was 16k
        let mut sOneLine: [u8; 1024] = [0; 1024];

        // find end of line...
        //
        let mut pThisLineEnd = p;
        while *pThisLineEnd != 0 && *pThisLineEnd != b'\r' as i8 && ((pThisLineEnd as usize - p as usize) < SONE_LINE_SIZE - 1) {
            pThisLineEnd = pThisLineEnd.offset(1);
        }

        let iCharsToCopy = (pThisLineEnd as usize - p as usize) as usize;
        strncpy(sOneLine.as_mut_ptr() as *mut i8, p, iCharsToCopy);
        sOneLine[iCharsToCopy] = 0;
        p = p.offset(iCharsToCopy as isize);
        while *p == b'\n' as i8 || *p == b'\r' as i8 {
            p = p.offset(1);
        }

        // trim trailing...
        //
        let mut bTrimmed: i32 = 0; // qfalse
        loop {
            bTrimmed = 0; // qfalse
            let iStrLen = strlen(sOneLine.as_ptr() as *const i8);

            if iStrLen != 0 {
                if sOneLine[iStrLen - 1] == b'\t' || sOneLine[iStrLen - 1] == b' ' {
                    sOneLine[iStrLen - 1] = 0;
                    bTrimmed = 1; // qtrue
                }
            }
            if bTrimmed == 0 {
                break;
            }
        }

        strNewText.push_str(std::ffi::CStr::from_ptr(sOneLine.as_ptr() as *const i8).to_str().unwrap_or(""));
        strNewText.push('\n');
    }

    let pNewText = Z_Malloc(strNewText.len() + 1, 0, 0); // TAG_TEMP_WORKSPACE is tag param
    strcpy(pNewText as *mut i8, strNewText.as_ptr() as *const i8);
    #[cfg(target_os = "xbox")]
    Z_SetNewDeleteTemporary(false);

    pNewText as *mut i8
}

// called from SV_SpawnServer, but before map load and music start etc.
//
// This just initialises the Lucas music structs so the background music player can interrogate them...
//
static mut gsLevelNameFromServer: String = String::new();

pub unsafe fn Music_SetLevelName(psLevelName: *const i8) {
    gsLevelNameFromServer = std::ffi::CStr::from_ptr(psLevelName)
        .to_string_lossy()
        .into_owned();
}

unsafe fn Music_ParseLeveldata(psLevelName: *const i8) -> i32 {
    let mut bReturn: i32 = 0; // qfalse

    if MusicData.is_none() {
        MusicData = Some(Box::new(HashMap::new()));
    }

    // already got this data?
    //
    let psLevelName_str = std::ffi::CStr::from_ptr(psLevelName)
        .to_string_lossy()
        .into_owned();

    if let Some(ref data) = MusicData {
        if !data.is_empty() && Q_stricmp(psLevelName, gsLevelNameForCompare.as_ptr() as *const i8) == 0 {
            return 1; // qtrue
        }
    }

    if let Some(ref mut data) = &mut MusicData {
        data.clear();
    }

    let mut sLevelName: [u8; MAX_QPATH] = [0; MAX_QPATH];
    Q_strncpyz(sLevelName.as_mut_ptr() as *mut i8, psLevelName, MAX_QPATH);

    gsLevelNameForLoad = psLevelName_str.clone(); // harmless to init here even if we fail to parse dms.dat file
    gsLevelNameForCompare = psLevelName_str.clone(); // harmless to init here even if we fail to parse dms.dat file
    gsLevelNameForBossLoad = psLevelName_str.clone(); // harmless to init here even if we fail to parse dms.dat file

    let mut pText: *mut i8 = std::ptr::null_mut();
    /*int iTotalBytesLoaded = */FS_ReadFile(sFILENAME_DMS.as_ptr() as *const i8, &mut pText);
    if !pText.is_null() {
        let psStrippedText = StripTrailingWhiteSpaceOnEveryLine(pText);
        // CGenericParser2 Parser;
        let mut Parser = std::mem::zeroed::<CGenericParser2>();
        let mut psDataPtr = psStrippedText; // because ptr gets advanced, so we supply a clone that GP can alter

        // if (Parser.Parse(&psDataPtr, true))
        // {
        //     ... parsing logic ...
        // }

        // This would need actual C++ bindings to proceed further
        // For structural translation, we show the placeholder

        // if parsing succeeded...
        {
            bReturn = 1; // qtrue

            // Check that music data is valid and files exist
            // This section would iterate through MusicData entries
        }

        Z_Free(psStrippedText);
        FS_FreeFile(pText);
    } else {
        Music_Parse_Error(b"Unable to even read main file\0".as_ptr() as *const i8); // file name specified in error message
    }

    bReturn
}

// returns ptr to music file, or NULL for error/missing...
//
unsafe fn Music_GetBaseMusicFile_str(psMusicState: *const i8) -> Option<*const MusicFile_t> {
    // where psMusicState is (eg) "explore", "action" or "boss"
    if let Some(ref data) = MusicData {
        let state_key = std::ffi::CStr::from_ptr(psMusicState)
            .to_string_lossy()
            .into_owned();
        if let Some(music_file) = data.get(&state_key) {
            return Some(music_file as *const MusicFile_t);
        }
    }

    None
}

unsafe fn Music_GetBaseMusicFile(eMusicState: MusicState_e) -> Option<*const MusicFile_t> {
    if let Some(psMusicStateString) = Music_BaseStateToString(eMusicState, false) {
        let state_str = std::ffi::CString::new(psMusicStateString).unwrap();
        return Music_GetBaseMusicFile_str(state_str.as_ptr());
    }

    None
}

// where label is (eg) "kejim_base"...
//
pub unsafe fn Music_DynamicDataAvailable(psDynamicMusicLabel: *const i8) -> i32 {
    let mut sLevelName: [u8; MAX_QPATH] = [0; MAX_QPATH];

    let effective_label = if !psDynamicMusicLabel.is_null() && *psDynamicMusicLabel != 0 {
        psDynamicMusicLabel
    } else {
        gsLevelNameFromServer.as_ptr() as *const i8
    };

    let skipped = COM_SkipPath(effective_label as *mut i8);
    Q_strncpyz(sLevelName.as_mut_ptr() as *mut i8, skipped, MAX_QPATH);
    strlwr(sLevelName.as_mut_ptr() as *mut i8);

    if strlen(sLevelName.as_ptr() as *const i8) != 0 { // avoid error messages when there's no music waiting to be played and we try and restart it...
        if Music_ParseLeveldata(sLevelName.as_ptr() as *const i8) != 0 {
            let explore = Music_GetBaseMusicFile(MusicState_e::eBGRNDTRACK_EXPLORE);
            let action = Music_GetBaseMusicFile(MusicState_e::eBGRNDTRACK_ACTION);
            return if explore.is_some() && action.is_some() { 1 } else { 0 };
        }
    }

    0 // qfalse
}

pub unsafe fn Music_GetFileNameForState(eMusicState: MusicState_e) -> Option<String> {
    match eMusicState {
        MusicState_e::eBGRNDTRACK_EXPLORE
        | MusicState_e::eBGRNDTRACK_ACTION
        | MusicState_e::eBGRNDTRACK_BOSS
        | MusicState_e::eBGRNDTRACK_DEATH => {
            if let Some(pMusicFile) = Music_GetBaseMusicFile(eMusicState) {
                let music_file = &*pMusicFile;
                let file_name_ptr = music_file.sFileNameBase.as_ptr() as *const i8;
                return Some(Music_BuildFileName(file_name_ptr, eMusicState));
            }
        }

        MusicState_e::eBGRNDTRACK_ACTIONTRANS0
        | MusicState_e::eBGRNDTRACK_ACTIONTRANS1
        | MusicState_e::eBGRNDTRACK_ACTIONTRANS2
        | MusicState_e::eBGRNDTRACK_ACTIONTRANS3 => {
            if let Some(pMusicFile) = Music_GetBaseMusicFile(MusicState_e::eBGRNDTRACK_ACTION) {
                let music_file = &*pMusicFile;
                let iTransNum = (eMusicState as c_int) - (MusicState_e::eBGRNDTRACK_ACTIONTRANS0 as c_int);
                if (iTransNum as usize) < music_file.MusicExitPoints.len() {
                    let next_file_ptr = music_file.MusicExitPoints[iTransNum as usize].sNextFile.as_ptr() as *const i8;
                    return Some(Music_BuildFileName(next_file_ptr, eMusicState));
                }
            }
        }

        MusicState_e::eBGRNDTRACK_EXPLORETRANS0
        | MusicState_e::eBGRNDTRACK_EXPLORETRANS1
        | MusicState_e::eBGRNDTRACK_EXPLORETRANS2
        | MusicState_e::eBGRNDTRACK_EXPLORETRANS3 => {
            if let Some(pMusicFile) = Music_GetBaseMusicFile(MusicState_e::eBGRNDTRACK_EXPLORE) {
                let music_file = &*pMusicFile;
                let iTransNum = (eMusicState as c_int) - (MusicState_e::eBGRNDTRACK_EXPLORETRANS0 as c_int);
                if (iTransNum as usize) < music_file.MusicExitPoints.len() {
                    let next_file_ptr = music_file.MusicExitPoints[iTransNum as usize].sNextFile.as_ptr() as *const i8;
                    return Some(Music_BuildFileName(next_file_ptr, eMusicState));
                }
            }
        }

        _ => {
            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                debug_assert!(false, "Music_GetFileNameForState unhandled case");
                Com_Printf(
                    b"Music_GetFileNameForState( %d ) unhandled case!\n\0".as_ptr() as *const i8,
                    eMusicState as c_int,
                );
            }
        }
    }

    None
}

pub unsafe fn Music_StateIsTransition(eMusicState: MusicState_e) -> i32 {
    if (eMusicState as c_int) >= (MusicState_e::eBGRNDTRACK_FIRSTTRANSITION as c_int)
        && (eMusicState as c_int) <= (MusicState_e::eBGRNDTRACK_LASTTRANSITION as c_int)
    {
        1 // qtrue
    } else {
        0 // qfalse
    }
}

pub unsafe fn Music_StateCanBeInterrupted(
    eMusicState: MusicState_e,
    eProposedMusicState: MusicState_e,
) -> i32 {
    // death music can interrupt anything...
    //
    if eProposedMusicState == MusicState_e::eBGRNDTRACK_DEATH {
        return 1; // qtrue
    }
    // ... and can't be interrupted once started...(though it will internally-switch to silence at the end, rather than loop)
    //
    if eMusicState == MusicState_e::eBGRNDTRACK_DEATH {
        return 0; // qfalse
    }

    // boss music can interrupt anything (other than death, but that's already handled above)...
    //
    if eProposedMusicState == MusicState_e::eBGRNDTRACK_BOSS {
        return 1; // qtrue
    }
    // ... and can't be interrupted once started...
    //
    if eMusicState == MusicState_e::eBGRNDTRACK_BOSS {
        // ...except by silence (or death, but again, that's already handled above)
        //
        if eProposedMusicState == MusicState_e::eBGRNDTRACK_SILENCE {
            return 1; // qtrue
        }

        return 0; // qfalse
    }

    // action music can interrupt anything (after boss & death filters above)...
    //
    if eProposedMusicState == MusicState_e::eBGRNDTRACK_ACTION {
        return 1; // qtrue
    }

    // nothing can interrupt a transition (after above filters)...
    //
    if Music_StateIsTransition(eMusicState) != 0 {
        return 0; // qfalse
    }

    // current state is therefore interruptable...
    //
    1 // qtrue
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
pub unsafe fn Music_AllowedToTransition(
    fPlayingTimeElapsed: f32,
    eMusicState: MusicState_e,
    //
    peTransition: Option<*mut MusicState_e>,
    pfNewTrackEntryTime: Option<*mut f32>,
) -> i32 {
    const fTimeEpsilon: f32 = 0.3; // arb., how close we have to be to an exit point to take it.
                                    //		if set too high then music change is sloppy
                                    //		if set too low[/precise] then we might miss an exit if client fps is poor

    if let Some(pMusicFile) = Music_GetBaseMusicFile(eMusicState) {
        let music_file = &*pMusicFile;
        if !music_file.MusicExitTimes.is_empty() {
            // since a MusicExitTimes_t item is a sorted array, we can use the equal_range algorithm...
            //
            let T = MusicExitTime_t {
                fTime: fPlayingTimeElapsed,
                iExitPoint: 0,
            };

            // Find the range of elements equal to T (within floating point tolerance)
            let mut itp_first = 0;
            let mut itp_second = music_file.MusicExitTimes.len();

            // Find first element >= T
            for (i, elem) in music_file.MusicExitTimes.iter().enumerate() {
                if elem.fTime >= T.fTime {
                    itp_first = i;
                    break;
                }
            }

            // Find first element > T
            for (i, elem) in music_file.MusicExitTimes.iter().enumerate() {
                if elem.fTime > T.fTime {
                    itp_second = i;
                    break;
                }
            }

            if itp_first != 0 {
                itp_first -= 1; // encompass the one before, in case we've just missed an exit point by < fTimeEpsilon
            }
            if itp_second != music_file.MusicExitTimes.len() {
                itp_second += 1; // increase range to one beyond, so we can do normal STL being/end looping below
            }

            for it in itp_first..itp_second {
                let pExitTime = &music_file.MusicExitTimes[it];

                if Q_fabs(pExitTime.fTime - fPlayingTimeElapsed) <= fTimeEpsilon {
                    // got an exit point!, work out feedback params...
                    //
                    let iExitPoint = pExitTime.iExitPoint as usize;
                    //
                    // the two params to give back...
                    //
                    let mut eFeedBackTransition: MusicState_e = MusicState_e::eBGRNDTRACK_EXPLORETRANS0; // any old default
                    let mut fFeedBackNewTrackEntryTime: f32 = 0.0;
                    //
                    // check legality in case of crap data...
                    //
                    if iExitPoint < music_file.MusicExitPoints.len() {
                        let ExitPoint = &music_file.MusicExitPoints[iExitPoint];

                        match eMusicState {
                            MusicState_e::eBGRNDTRACK_EXPLORE => {
                                debug_assert!(iExitPoint < iMAX_EXPLORE_TRANSITIONS); // already been checked, but sanity
                                debug_assert!(ExitPoint.sNextMark.as_ptr()[0] == 0); // simple error checking, but harmless if tripped. explore transitions go to silence, hence no entry time for [silence] state after transition

                                eFeedBackTransition = std::mem::transmute::<c_int, MusicState_e>(
                                    (MusicState_e::eBGRNDTRACK_EXPLORETRANS0 as c_int) + (iExitPoint as c_int),
                                );
                            }

                            MusicState_e::eBGRNDTRACK_ACTION => {
                                debug_assert!(iExitPoint < iMAX_ACTION_TRANSITIONS); // already been checked, but sanity

                                // if there's an entry marker point defined...
                                //
                                if ExitPoint.sNextMark.as_ptr()[0] != 0 {
                                    if let Some(ref data) = MusicData {
                                        if let Some(Music_BaseStateString) = Music_BaseStateToString(MusicState_e::eBGRNDTRACK_EXPLORE, false) {
                                            if let Some(MusicFile_Explore) = data.get(Music_BaseStateString) {
                                                // find the entry marker within the music and read the time there...
                                                //
                                                if let Some(&entry_time) =
                                                    MusicFile_Explore.MusicEntryTimes.get(&ExitPoint.sNextMark)
                                                {
                                                    fFeedBackNewTrackEntryTime = entry_time;
                                                    eFeedBackTransition = std::mem::transmute::<c_int, MusicState_e>(
                                                        (MusicState_e::eBGRNDTRACK_ACTIONTRANS0 as c_int)
                                                            + (iExitPoint as c_int),
                                                    );
                                                } else {
                                                    #[cfg(not(feature = "FINAL_BUILD"))]
                                                    {
                                                        debug_assert!(false); // sanity, should have been caught elsewhere, but harmless to do this
                                                        Com_Printf(
                                                            b"Music_AllowedToTransition() unable to find entry marker \"%s\" in \"%s\"\0"
                                                                .as_ptr()
                                                                as *const i8,
                                                            ExitPoint.sNextMark.as_ptr(),
                                                            MusicFile_Explore.sFileNameBase.as_ptr(),
                                                        );
                                                    }
                                                    return 0; // qfalse
                                                }
                                            } else {
                                                #[cfg(not(feature = "FINAL_BUILD"))]
                                                {
                                                    debug_assert!(false); // sanity, should have been caught elsewhere, but harmless to do this
                                                    Com_Printf(
                                                        b"Music_AllowedToTransition() unable to find %s version of \"%s\"\n\0"
                                                            .as_ptr()
                                                            as *const i8,
                                                        Music_BaseStateToString(MusicState_e::eBGRNDTRACK_EXPLORE, false)
                                                            .unwrap_or("")
                                                            .as_ptr(),
                                                        music_file.sFileNameBase.as_ptr(),
                                                    );
                                                }
                                                return 0; // qfalse
                                            }
                                        }
                                    }
                                } else {
                                    eFeedBackTransition = MusicState_e::eBGRNDTRACK_ACTIONTRANS0;
                                    fFeedBackNewTrackEntryTime = 0.0; // already set to this, but FYI
                                }
                            }

                            _ => {
                                #[cfg(not(feature = "FINAL_BUILD"))]
                                {
                                    debug_assert!(false);
                                    Com_Printf(
                                        b"Music_AllowedToTransition(): No code to transition from music type %d\n\0"
                                            .as_ptr()
                                            as *const i8,
                                        eMusicState as c_int,
                                    );
                                }
                                return 0; // qfalse
                            }
                        }
                    } else {
                        #[cfg(not(feature = "FINAL_BUILD"))]
                        {
                            debug_assert!(false);
                            Com_Printf(
                                b"Music_AllowedToTransition(): Illegal exit point %d, max = %d (music: \"%s\")\n\0"
                                    .as_ptr()
                                    as *const i8,
                                iExitPoint as c_int,
                                (music_file.MusicExitPoints.len() - 1) as c_int,
                                music_file.sFileNameBase.as_ptr(),
                            );
                        }
                        return 0; // qfalse
                    }

                    // feed back answers...
                    //
                    if let Some(pTransition) = peTransition {
                        *pTransition = eFeedBackTransition;
                    }

                    if let Some(pNewTrackEntryTime) = pfNewTrackEntryTime {
                        *pNewTrackEntryTime = fFeedBackNewTrackEntryTime;
                    }

                    return 1; // qtrue
                }
            }
        }
    }

    0 // qfalse
}

// typically used to get a (predefined) random entry point for the action music, but will work on any defined type with entry points,
//	defaults safely to 0.0f if no info available...
//
pub unsafe fn Music_GetRandomEntryTime(eMusicState: MusicState_e) -> f32 {
    if let Some(ref data) = MusicData {
        if let Some(state_str) = Music_BaseStateToString(eMusicState, false) {
            if let Some(MusicFile) = data.get(state_str) {
                if MusicFile.MusicEntryTimes.len() != 0 { // make sure at least one defined, else default to start
                    // Quake's random number generator isn't very good, so instead of this:
                    //
                    // int iRandomEntryNum = Q_irand(0, (MusicFile.MusicEntryTimes.size()-1) );
                    //
                    // ... I'll do this (ensuring we don't get the same result on two consecutive calls, but without while-loop)...
                    //
                    static mut iPrevRandomNumber: c_int = -1;
                    static mut iCallCount: c_int = 0;
                    iCallCount += 1;
                    let mut iRandomEntryNum = ((rand() as c_int) + iCallCount) % (MusicFile.MusicEntryTimes.len() as c_int);
                    if iRandomEntryNum == iPrevRandomNumber && MusicFile.MusicEntryTimes.len() > 1 {
                        iRandomEntryNum += 1;
                        iRandomEntryNum %= MusicFile.MusicEntryTimes.len() as c_int;
                    }
                    iPrevRandomNumber = iRandomEntryNum;

                    let mut counter = 0;
                    for (_key, &entry_time) in &MusicFile.MusicEntryTimes {
                        if counter == iRandomEntryNum {
                            return entry_time;
                        }
                        counter += 1;
                    }
                }
            }
        }
    }

    0.0
}

// info only, used in "soundinfo" command...
//
pub unsafe fn Music_GetLevelSetName() -> String {
    if Q_stricmp(gsLevelNameForCompare.as_ptr() as *const i8, gsLevelNameForLoad.as_ptr() as *const i8) != 0 {
        // music remap via USES command...
        //
        return format!("{} -> {}", gsLevelNameForCompare, gsLevelNameForLoad);
    }

    gsLevelNameForLoad.clone()
}

// stub for external rand() function
extern "C" {
    fn rand() -> c_int;
}

///////////////// eof /////////////////////
