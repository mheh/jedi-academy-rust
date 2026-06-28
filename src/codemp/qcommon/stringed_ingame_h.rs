//! `stringed_ingame.h` — in-game StringEd declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared::Q_stricmp;
use crate::codemp::game::q_shared_h::{qboolean, MAX_QPATH, QFALSE, QTRUE};
use core::ffi::{c_char, c_float, c_int};

// alter these to suit your own game...
//
pub type SE_BOOL = qboolean;
pub const SE_TRUE: SE_BOOL = QTRUE;
pub const SE_FALSE: SE_BOOL = QFALSE;
pub const iSE_MAX_FILENAME_LENGTH: usize = MAX_QPATH;
pub const sSE_STRINGS_DIR: &str = "strings";
pub const sSE_DEBUGSTR_PREFIX: &str = "["; // any string you want prefixing onto the debug versions of strings (to spot hardwired english etc)
pub const sSE_DEBUGSTR_SUFFIX: &str = "]"; // ""

// nothing outside the Cvar_*() functions should modify these fields!
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char, // cvar_restart will reset to this value
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int, // incremented each time the cvar is changed
    pub value: c_float,           // atof( string )
    pub integer: c_int,           // atoi( string )
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

unsafe extern "C" {
    pub static mut se_language: *mut cvar_t;
}

// some needed text-equates, do not alter these under any circumstances !!!! (unless you're me. Which you're not)
//
pub const iSE_VERSION: c_int = 1;
pub const sSE_KEYWORD_VERSION: &str = "VERSION";
pub const sSE_KEYWORD_CONFIG: &str = "CONFIG";
pub const sSE_KEYWORD_FILENOTES: &str = "FILENOTES";
pub const sSE_KEYWORD_REFERENCE: &str = "REFERENCE";
pub const sSE_KEYWORD_FLAGS: &str = "FLAGS";
pub const sSE_KEYWORD_NOTES: &str = "NOTES";
pub const sSE_KEYWORD_LANG: &str = "LANG_";
pub const sSE_KEYWORD_ENDMARKER: &str = "ENDMARKER";
pub const sSE_FILE_EXTENSION: &str = ".st"; // editor-only NEVER used ingame, but I wanted all extensions together
pub const sSE_EXPORT_FILE_EXTENSION: &str = ".ste";
pub const sSE_INGAME_FILE_EXTENSION: &str = ".str";
pub const sSE_EXPORT_SAME: &str = "#same";

// available API calls...
//
pub type LPCSTR = *const c_char;

unsafe extern "C" {
    pub fn SE_Init();
    pub fn SE_ShutDown();
    pub fn SE_CheckForLanguageUpdates();
    pub fn SE_GetNumLanguages() -> c_int;
    pub fn SE_GetLanguageName(iLangIndex: c_int) -> LPCSTR; // eg "german"
    pub fn SE_GetLanguageDir(iLangIndex: c_int) -> LPCSTR; // eg "strings/german"
    pub fn SE_LoadLanguage(psLanguage: LPCSTR, bLoadDebug: SE_BOOL) -> LPCSTR; // C++ default: bLoadDebug = SE_TRUE
    pub fn SE_NewLanguage();
    //
    // for convenience, two ways of getting at the same data...
    //
    pub fn SE_GetString(psPackageReference: LPCSTR, psStringReference: LPCSTR) -> LPCSTR;
    #[link_name = "SE_GetString"]
    pub fn SE_GetString_PackageAndStringReference(psPackageAndStringReference: LPCSTR) -> LPCSTR;
    //
    // ditto...
    //
    pub fn SE_GetFlags(psPackageReference: LPCSTR, psStringReference: LPCSTR) -> c_int;
    #[link_name = "SE_GetFlags"]
    pub fn SE_GetFlags_PackageAndStringReference(psPackageAndStringReference: LPCSTR) -> c_int;
    //
    // general flag functions... (SEP_GetFlagMask() return should be used with SEP_GetFlags() return)
    //
    pub fn SE_GetNumFlags() -> c_int;
    pub fn SE_GetFlagName(iFlagIndex: c_int) -> LPCSTR;
    pub fn SE_GetFlagMask(psFlagName: LPCSTR) -> c_int;
}

static RUSSIAN: [c_char; 8] = [
    b'r' as c_char,
    b'u' as c_char,
    b's' as c_char,
    b's' as c_char,
    b'i' as c_char,
    b'a' as c_char,
    b'n' as c_char,
    0,
];
static POLISH: [c_char; 7] = [
    b'p' as c_char,
    b'o' as c_char,
    b'l' as c_char,
    b'i' as c_char,
    b's' as c_char,
    b'h' as c_char,
    0,
];
static KOREAN: [c_char; 7] = [
    b'k' as c_char,
    b'o' as c_char,
    b'r' as c_char,
    b'e' as c_char,
    b'a' as c_char,
    b'n' as c_char,
    0,
];
static TAIWANESE: [c_char; 10] = [
    b't' as c_char,
    b'a' as c_char,
    b'i' as c_char,
    b'w' as c_char,
    b'a' as c_char,
    b'n' as c_char,
    b'e' as c_char,
    b's' as c_char,
    b'e' as c_char,
    0,
];
static JAPANESE: [c_char; 9] = [
    b'j' as c_char,
    b'a' as c_char,
    b'p' as c_char,
    b'a' as c_char,
    b'n' as c_char,
    b'e' as c_char,
    b's' as c_char,
    b'e' as c_char,
    0,
];
static CHINESE: [c_char; 8] = [
    b'c' as c_char,
    b'h' as c_char,
    b'i' as c_char,
    b'n' as c_char,
    b'e' as c_char,
    b's' as c_char,
    b'e' as c_char,
    0,
];
static THAI: [c_char; 5] = [
    b't' as c_char,
    b'h' as c_char,
    b'a' as c_char,
    b'i' as c_char,
    0,
];

// note that so far the only place in the game that needs to know these is the font system so it can know how to
// interpret char codes, for this reason I'm only exposing these simple bool queries...
//
#[inline]
pub unsafe fn Language_IsRussian() -> SE_BOOL {
    if unsafe { !se_language.is_null() && Q_stricmp((*se_language).string, RUSSIAN.as_ptr()) == 0 }
    {
        SE_TRUE
    } else {
        SE_FALSE
    }
}

#[inline]
pub unsafe fn Language_IsPolish() -> SE_BOOL {
    if unsafe { !se_language.is_null() && Q_stricmp((*se_language).string, POLISH.as_ptr()) == 0 }
    {
        SE_TRUE
    } else {
        SE_FALSE
    }
}

#[inline]
pub unsafe fn Language_IsKorean() -> SE_BOOL {
    if unsafe { !se_language.is_null() && Q_stricmp((*se_language).string, KOREAN.as_ptr()) == 0 }
    {
        SE_TRUE
    } else {
        SE_FALSE
    }
}

#[inline]
pub unsafe fn Language_IsTaiwanese() -> SE_BOOL {
    if unsafe { !se_language.is_null() && Q_stricmp((*se_language).string, TAIWANESE.as_ptr()) == 0 }
    {
        SE_TRUE
    } else {
        SE_FALSE
    }
}

#[inline]
pub unsafe fn Language_IsJapanese() -> SE_BOOL {
    if unsafe { !se_language.is_null() && Q_stricmp((*se_language).string, JAPANESE.as_ptr()) == 0 }
    {
        SE_TRUE
    } else {
        SE_FALSE
    }
}

#[inline]
pub unsafe fn Language_IsChinese() -> SE_BOOL {
    if unsafe { !se_language.is_null() && Q_stricmp((*se_language).string, CHINESE.as_ptr()) == 0 }
    {
        SE_TRUE
    } else {
        SE_FALSE
    }
}

#[inline]
pub unsafe fn Language_IsThai() -> SE_BOOL {
    if unsafe { !se_language.is_null() && Q_stricmp((*se_language).string, THAI.as_ptr()) == 0 } {
        SE_TRUE
    } else {
        SE_FALSE
    }
}

/////////////////// eof ////////////////
