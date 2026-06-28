// Filename:-	stringed_ingame.cpp
//
// This file is designed to be pasted into each game project that uses the StringEd package's files.
//  You can alter the way it does things by (eg) replacing STL with RATL, BUT LEAVE THE OVERALL
//	FUNCTIONALITY THE SAME, or if I ever make any funadamental changes to the way the package works
//	then you're going to be SOOL (shit out of luck ;-)...
//

//////////////////////////////////////////////////
//
// stuff common to all qcommon files...
// #include "../server/server.h"
// #include "../game/q_shared.h"
// #include "qcommon.h"
//
//////////////////////////////////////////////////

use std::collections::{HashMap, HashSet};
use core::ffi::{c_int, c_char, CStr};
use std::ffi::CString;

// Porting notes: extern dependencies are stubbed as needed below
// These would normally come from various game engine headers

// Stub for cvar_t type
#[repr(C)]
pub struct cvar_t {
    pub string: *const c_char,
    pub integer: c_int,
    pub modified: c_int,
    // ... other fields not needed for this file
}

// Stub for constants/functions from stringed_interface.h
const iSE_MAX_FILENAME_LENGTH: usize = 260;
const iSE_VERSION: c_int = 1;
const sSE_KEYWORD_VERSION: &str = "VERSION";
const sSE_KEYWORD_CONFIG: &str = "CONFIG";
const sSE_KEYWORD_FILENOTES: &str = "FILENOTES";
const sSE_KEYWORD_NOTES: &str = "NOTES";
const sSE_KEYWORD_REFERENCE: &str = "REFERENCE";
const sSE_KEYWORD_FLAGS: &str = "FLAGS";
const sSE_KEYWORD_ENDMARKER: &str = "ENDMARKER";
const sSE_KEYWORD_LANG: &str = "LANG_";
const sSE_STRINGS_DIR: &str = "strings";
const sSE_INGAME_FILE_EXTENSION: &str = ".str";
const sSE_EXPORT_FILE_EXTENSION: &str = ".ste";
const sSE_EXPORT_SAME: &str = "#same";
const sSE_DEBUGSTR_PREFIX: &str = "SE:";
const sSE_DEBUGSTR_SUFFIX: &str = "";

type SE_BOOL = c_int;
const SE_TRUE: SE_BOOL = 1;
const SE_FALSE: SE_BOOL = 0;

// Stubs for external dependencies
extern "C" {
    fn Z_Malloc(size: usize, tag: c_int, clear: SE_BOOL) -> *mut c_char;
    fn Z_Free(ptr: *mut c_void);
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn COM_DefaultExtension(path: *mut c_char, maxlen: usize, ext: *const c_char);
    fn Q_strupr(string: *mut c_char);
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize);
    fn SE_LoadFileData(filename: *const c_char) -> *mut c_uchar;
    fn SE_FreeFileDataAfterLoad(data: *mut c_uchar);
    fn SE_BuildFileList(path: *const c_char, results: *mut String) -> c_int;
}

use std::os::raw::c_void;
type c_uchar = u8;

// #pragma warning ( disable : 4511 )			// copy constructor could not be generated
// #pragma warning ( disable : 4512 )			// assignment operator could not be generated
// #pragma warning ( disable : 4663 )			// C++ language change: blah blah template crap blah blah

///////////////////////////////////////////////
//
// some STL stuff...
// #pragma warning ( disable : 4786 )			// disable the usual stupid and pointless STL warning
// #include <list>
// #include <map>
// #include <set>
// #include <string>
// #include <vector>
// using namespace std;

// typedef vector<string>	vStrings_t;
// typedef vector<int>		vInts_t;
//
///////////////////////////////////////////////

pub static mut se_language: *mut cvar_t = std::ptr::null_mut();
pub static mut se_debug: *mut cvar_t = std::ptr::null_mut();
pub static mut sp_leet: *mut cvar_t = std::ptr::null_mut();	// kept as 'sp_' for JK2 compat.

// #define __DEBUGOUT(_string)	OutputDebugString(_string)
// #define __ASSERT(_blah)		assert(_blah)

#[derive(Clone)]
struct SE_Entry_s {
    m_strString: String,
    m_strDebug: String,	// english and/or "#same", used for debugging only. Also prefixed by "SE:" to show which strings go through StringEd (ie aren't hardwired)
    m_iFlags: c_int,
}

impl SE_Entry_s {
    fn new() -> Self {
        SE_Entry_s {
            m_strString: String::new(),
            m_strDebug: String::new(),
            m_iFlags: 0,
        }
    }
}

type mapStringEntries_t = HashMap<String, SE_Entry_s>;

struct CStringEdPackage {
    // private:

    m_bEndMarkerFound_ParseOnly: SE_BOOL,
    m_strCurrentEntryRef_ParseOnly: String,
    m_strCurrentEntryEnglish_ParseOnly: String,
    m_strCurrentFileRef_ParseOnly: String,
    m_strLoadingLanguage_ParseOnly: String,	// eg "german"
    m_bLoadingEnglish_ParseOnly: SE_BOOL,

    // public:

    m_StringEntries: mapStringEntries_t,	// needs to be in public space now
    m_bLoadDebug: SE_BOOL,		// ""
    //
    // flag stuff...
    //
    m_vstrFlagNames: Vec<String>,
    m_mapFlagMasks: HashMap<String, c_int>,
}

impl CStringEdPackage {
    fn new() -> Self {
        CStringEdPackage {
            m_bEndMarkerFound_ParseOnly: SE_FALSE,
            m_strCurrentEntryRef_ParseOnly: String::new(),
            m_strCurrentEntryEnglish_ParseOnly: String::new(),
            m_strCurrentFileRef_ParseOnly: String::new(),
            m_strLoadingLanguage_ParseOnly: String::new(),
            m_bLoadingEnglish_ParseOnly: SE_FALSE,
            m_StringEntries: HashMap::new(),
            m_bLoadDebug: SE_FALSE,
            m_vstrFlagNames: Vec::new(),
            m_mapFlagMasks: HashMap::new(),
        }
    }

    fn Clear(&mut self, bChangingLanguages: SE_BOOL) {
        self.m_StringEntries.clear();

        if bChangingLanguages == SE_FALSE {
            // if we're changing languages, then I'm going to leave these alone. This is to do with any (potentially) cached
            //	flag bitmasks on the game side. It shouldn't matter since all files are written out at once using the build
            //	command in StringEd. But if ever someone changed a file by hand, or added one, or whatever, and it had a
            //	different set of flags declared, or the strings were in a different order, then the flags might also change
            //	the order I see them in, and hence their indexes and masks. This should never happen unless people mess with
            //	the .STR files by hand and delete some, but this way makes sure it'll all work just in case...
            //
            // ie. flags stay defined once they're defined, and only the destructor (at app-end) kills them.
            //
            self.m_vstrFlagNames.clear();
            self.m_mapFlagMasks.clear();
        }

        self.m_bEndMarkerFound_ParseOnly = SE_FALSE;
        self.m_strCurrentEntryRef_ParseOnly = String::new();
        self.m_strCurrentEntryEnglish_ParseOnly = String::new();
        //
        // the other vars are cleared in SetupNewFileParse(), and are ok to not do here.
        //
    }

    // loses anything after the path (if any), (eg) "dir/name.bmp" becomes "dir"
    // (copes with either slash-scheme for names)
    //
    // (normally I'd call another function for this, but this is supposed to be engine-independant,
    //	 so a certain amount of re-invention of the wheel is to be expected...)
    //
    fn Filename_PathOnly(&self, psFilename: &str) -> String {
        let mut sString = psFilename.to_string();

        let p1 = sString.rfind('\\');
        let p2 = sString.rfind('/');
        let p = match (p1, p2) {
            (Some(i1), Some(i2)) => Some(if i1 > i2 { i1 } else { i2 }),
            (Some(i1), None) => Some(i1),
            (None, Some(i2)) => Some(i2),
            (None, None) => None,
        };
        if let Some(pos) = p {
            sString.truncate(pos);
        }

        sString
    }

    // returns (eg) "dir/name" for "dir/name.bmp"
    // (copes with either slash-scheme for names)
    //
    // (normally I'd call another function for this, but this is supposed to be engine-independant,
    //	 so a certain amount of re-invention of the wheel is to be expected...)
    //
    fn Filename_WithoutExt(&self, psFilename: &str) -> String {
        let mut sString = psFilename.to_string();

        let p = sString.rfind('.');
        let p2 = sString.rfind('\\');
        let p3 = sString.rfind('/');

        // special check, make sure the first suffix we found from the end wasn't just a directory suffix (eg on a path'd filename with no extension anyway)
        //
        if let Some(pos) = p {
            let keep = match (p2, p3) {
                (Some(i2), Some(i3)) => pos > i2 && pos > i3,
                (Some(i2), None) => pos > i2,
                (None, Some(i3)) => pos > i3,
                (None, None) => true,
            };
            if keep {
                sString.truncate(pos);
            }
        }

        sString
    }

    // returns actual filename only, no path
    // (copes with either slash-scheme for names)
    //
    // (normally I'd call another function for this, but this is supposed to be engine-independant,
    //	 so a certain amount of re-invention of the wheel is to be expected...)
    //
    fn Filename_WithoutPath(&self, psFilename: &str) -> String {
        let mut psCopyPos: &str = psFilename;

        for (i, c) in psFilename.chars().enumerate() {
            if c == '/' || c == '\\' {
                psCopyPos = &psFilename[i+1..];
            }
        }

        psCopyPos.to_string()
    }

    fn ExtractLanguageFromPath(&self, psFileName: &str) -> String {
        let path_only = self.Filename_PathOnly(psFileName);
        self.Filename_WithoutPath(&path_only)
    }

    fn SetupNewFileParse(&mut self, psFileName: &str, bLoadDebug: SE_BOOL) {
        let mut sString = self.Filename_WithoutExt(&self.Filename_WithoutPath(psFileName));
        sString.make_ascii_uppercase();

        self.m_strCurrentFileRef_ParseOnly = sString;	// eg "OBJECTIVES"
        self.m_strLoadingLanguage_ParseOnly = self.ExtractLanguageFromPath(psFileName);
        self.m_bLoadingEnglish_ParseOnly = if self.m_strLoadingLanguage_ParseOnly.to_lowercase() == "english" { SE_TRUE } else { SE_FALSE };
        self.m_bLoadDebug = bLoadDebug;
    }

    // returns SE_TRUE if supplied keyword found at line start (and advances supplied ptr past any whitespace to next arg (or line end if none),
    //
    //	else returns SE_FALSE...
    //
    fn CheckLineForKeyword(&self, psKeyword: &str, psLine: &mut &str) -> SE_BOOL {
        if psLine.starts_with(psKeyword) {
            *psLine = &psLine[psKeyword.len()..];

            // skip whitespace to arrive at next item...
            //
            while !psLine.is_empty() && (*psLine.chars().next().unwrap() == '\t' || *psLine.chars().next().unwrap() == ' ') {
                *psLine = &psLine[1..];
            }
            return SE_TRUE;
        }

        SE_FALSE
    }

    // change "\n" to '\n' (i.e. 2-byte char-string to 1-byte ctrl-code)...
    //  (or "\r\n" in editor)
    //
    fn ConvertCRLiterals_Read(&self, psString: &str) -> String {
        let mut str = psString.to_string();
        while let Some(iLoc) = str.find("\\n") {
            str.remove(iLoc + 1);
            str.replace_range(iLoc..=iLoc, "\n");
        }

        str
    }

    // kill off any "//" onwards part in the line, but NOT if it's inside a quoted string...
    //
    fn REMKill(&self, psBuffer: &mut String) {
        let mut iDoubleQuotesSoFar = 0;

        // scan forwards in case there are more than one (and the first is inside quotes)...
        //
        loop {
            let psScanPos = psBuffer.clone();
            if let Some(p_pos) = psScanPos.find("//") {
                // count the number of double quotes before this point, if odd number, then we're inside quotes...
                //
                let mut iDoubleQuoteCount = iDoubleQuotesSoFar;

                for i in 0..p_pos {
                    if psScanPos.chars().nth(i) == Some('"') {
                        iDoubleQuoteCount += 1;
                    }
                }
                if (iDoubleQuoteCount & 1) == 0 {
                    // not inside quotes, so kill line here...
                    //
                    psBuffer.truncate(p_pos);
                    //
                    // and remove any trailing whitespace...
                    //
                    if !psBuffer.is_empty() {
                        while !psBuffer.is_empty() && psBuffer.chars().last().unwrap().is_whitespace() {
                            psBuffer.pop();
                        }
                    }

                    return;
                } else {
                    // inside quotes (blast), oh well, skip past and keep scanning...
                    //
                    let _ = psScanPos.get(p_pos+1..);
                    iDoubleQuotesSoFar = iDoubleQuoteCount;
                }
            } else {
                break;
            }
        }
    }

    // returns true while new lines available to be read...
    //
    fn ReadLine(&self, psParsePos: &mut &[u8], psDest: &mut String) -> SE_BOOL {
        if !psParsePos.is_empty() {
            // Find newline
            if let Some(pos) = psParsePos.iter().position(|&b| b == b'\n') {
                let line_bytes = &psParsePos[..pos];
                if let Ok(line_str) = std::str::from_utf8(line_bytes) {
                    *psDest = line_str.to_string();
                    *psParsePos = &psParsePos[pos+1..];
                    // skip over CR or CR/LF pairs
                    while !psParsePos.is_empty() && (psParsePos[0] == b'\r' || psParsePos[0] == b'\n') {
                        *psParsePos = &psParsePos[1..];
                    }
                }
            } else {
                // last line...
                //
                if let Ok(line_str) = std::str::from_utf8(psParsePos) {
                    *psDest = line_str.to_string();
                    *psParsePos = &[];
                }
            }

            // clean up the line...
            //
            if !psDest.is_empty() {
                while !psDest.is_empty() && psDest.chars().last().unwrap().is_whitespace() {
                    psDest.pop();
                }

                let mut line_mut = psDest.clone();
                self.REMKill(&mut line_mut);
                *psDest = line_mut;
            }
            return SE_TRUE;
        }

        SE_FALSE
    }

    // remove any outside quotes from this supplied line, plus any leading or trailing whitespace...
    //
    fn InsideQuotes(&self, psLine: &str) -> String {
        let mut str = String::new();

        // skip any leading whitespace...
        //
        let mut it = psLine;
        while !it.is_empty() && (it.chars().next() == Some(' ') || it.chars().next() == Some('\t')) {
            it = &it[1..];
        }

        // skip any leading quote...
        //
        if !it.is_empty() && it.chars().next() == Some('"') {
            it = &it[1..];
        }

        // assign it...
        //
        str = it.to_string();

        if !str.is_empty() {
            // lose any trailing whitespace...
            //
            while !str.is_empty() && (str.chars().last() == Some(' ') || str.chars().last() == Some('\t')) {
                str.pop();
            }

            // lose any trailing quote...
            //
            if !str.is_empty() && str.chars().last() == Some('"') {
                str.pop();
            }
        }

        // and return it...
        //
        str
    }

    // returns flag bitmask (eg 00000010b), else 0 for not found
    //
    fn GetFlagMask(&self, psFlagName: &str) -> c_int {
        if let Some(iMask) = self.m_mapFlagMasks.get(psFlagName) {
            *iMask
        } else {
            0
        }
    }

    fn AddFlagReference(&mut self, psLocalReference: &str, psFlagName: &str) {
        // add the flag to the list of known ones...
        //
        let mut iMask = self.GetFlagMask(psFlagName);
        if iMask == 0 {
            self.m_vstrFlagNames.push(psFlagName.to_string());
            iMask = 1 << (self.m_vstrFlagNames.len() - 1);
            self.m_mapFlagMasks.insert(psFlagName.to_string(), iMask);
        }
        //
        // then add the reference to this flag to the currently-parsed reference...
        //
        let full_ref = format!("{}_{}", self.m_strCurrentFileRef_ParseOnly, psLocalReference);
        if let Some(entry) = self.m_StringEntries.get_mut(&full_ref) {
            entry.m_iFlags |= iMask;
        }
    }

    fn GetCurrentFileName(&self) -> String {
        String::new() // Stub - not used in this file's public API
    }

    fn GetCurrentReference_ParseOnly(&self) -> String {
        self.m_strCurrentEntryRef_ParseOnly.clone()
    }

    fn AddEntry(&mut self, psLocalReference: &str) {
        // the reason I don't just assign it anyway is because the optional .STE override files don't contain flags,
        //	and therefore would wipe out the parsed flags of the .STR file...
        //
        let full_ref = format!("{}_{}", self.m_strCurrentFileRef_ParseOnly, psLocalReference);
        if !self.m_StringEntries.contains_key(&full_ref) {
            self.m_StringEntries.insert(full_ref, SE_Entry_s::new());
        }
        self.m_strCurrentEntryRef_ParseOnly = psLocalReference.to_string();
    }

    fn GetNumStrings(&self) -> c_int {
        self.m_StringEntries.len() as c_int
    }

    fn SetString(&mut self, psLocalReference: &str, psNewString: &str, bEnglishDebug: SE_BOOL) {
        let full_ref = format!("{}_{}", self.m_strCurrentFileRef_ParseOnly, psLocalReference);
        if let Some(entry) = self.m_StringEntries.get_mut(&full_ref) {
            if bEnglishDebug == SE_TRUE || self.m_bLoadingEnglish_ParseOnly == SE_TRUE {
                // then this is the leading english text of a foreign sentence pair (so it's the debug-key text),
                //	or it's the only text when it's english being loaded...
                //
                entry.m_strString = Leetify(psNewString);
                if self.m_bLoadDebug == SE_TRUE {
                    entry.m_strDebug = format!("{}{}{}", sSE_DEBUGSTR_PREFIX, psNewString, sSE_DEBUGSTR_SUFFIX);
                }
                self.m_strCurrentEntryEnglish_ParseOnly = psNewString.to_string();	// for possible "#same" resolving in foreign later
            } else {
                // then this is foreign text (so check for "#same" resolving)...
                //
                if psNewString.to_lowercase() == sSE_EXPORT_SAME {
                    entry.m_strString = self.m_strCurrentEntryEnglish_ParseOnly.clone();	// foreign "#same" is now english
                    if self.m_bLoadDebug == SE_TRUE {
                        entry.m_strDebug = format!("{}{}{}", sSE_DEBUGSTR_PREFIX, sSE_EXPORT_SAME, sSE_DEBUGSTR_SUFFIX);
                    }
                } else {
                    entry.m_strString = psNewString.to_string();							// foreign is just foreign
                }
            }
        }
    }

    fn SetReference(&mut self, iIndex: c_int, psNewString: &str) -> SE_BOOL {
        SE_FALSE // Stub - not implemented
    }

    fn ParseLine(&mut self, psLine: &str) -> Option<String> {
        let mut psErrorMessage: Option<String> = None;

        if !psLine.is_empty() {
            let mut psLine_mut = psLine;

            if self.CheckLineForKeyword(sSE_KEYWORD_VERSION, &mut psLine_mut) == SE_TRUE {
                // VERSION 	"1"
                //
                let psVersionNumber = self.InsideQuotes(psLine_mut);
                let iVersionNumber = psVersionNumber.parse::<c_int>().unwrap_or(0);

                if iVersionNumber != iSE_VERSION {
                    psErrorMessage = Some(format!("Unexpected version number {}, expecting {}!\n", iVersionNumber, iSE_VERSION));
                }
            } else if self.CheckLineForKeyword(sSE_KEYWORD_CONFIG, &mut psLine_mut) == SE_TRUE
                || self.CheckLineForKeyword(sSE_KEYWORD_FILENOTES, &mut psLine_mut) == SE_TRUE
                || self.CheckLineForKeyword(sSE_KEYWORD_NOTES, &mut psLine_mut) == SE_TRUE {
                // not used ingame, but need to absorb the token
            } else if self.CheckLineForKeyword(sSE_KEYWORD_REFERENCE, &mut psLine_mut) == SE_TRUE {
                // REFERENCE	GUARD_GOOD_TO_SEE_YOU
                //
                let psLocalReference = self.InsideQuotes(psLine_mut);
                self.AddEntry(&psLocalReference);
            } else if self.CheckLineForKeyword(sSE_KEYWORD_FLAGS, &mut psLine_mut) == SE_TRUE {
                // FLAGS 	FLAG_CAPTION FLAG_TYPEMATIC
                //
                let psReference = self.GetCurrentReference_ParseOnly();
                if !psReference.is_empty() {
                    let mut sFlags = psLine_mut.to_string();
                    let sSeperators = " \t";

                    let mut psToken = "";
                    let mut remaining = sFlags.as_str();
                    while let Some(pos) = remaining.find(|c: char| sSeperators.contains(c)) {
                        if pos > 0 {
                            psToken = &remaining[..pos];
                            let mut token_upper = psToken.to_string();
                            token_upper.make_ascii_uppercase();
                            self.AddFlagReference(&psReference, &token_upper);
                        }
                        remaining = &remaining[pos..];
                        while !remaining.is_empty() && sSeperators.contains(remaining.chars().next().unwrap()) {
                            remaining = &remaining[1..];
                        }
                    }
                    if !remaining.is_empty() {
                        let mut token_upper = remaining.to_string();
                        token_upper.make_ascii_uppercase();
                        self.AddFlagReference(&psReference, &token_upper);
                    }
                } else {
                    psErrorMessage = Some(format!("Error parsing file: Unexpected \"{}\"\n", sSE_KEYWORD_FLAGS));
                }
            } else if self.CheckLineForKeyword(sSE_KEYWORD_ENDMARKER, &mut psLine_mut) == SE_TRUE {
                // ENDMARKER
                //
                self.m_bEndMarkerFound_ParseOnly = SE_TRUE;	// the only major error checking I bother to do (for file truncation)
            } else if psLine_mut.starts_with(sSE_KEYWORD_LANG) {
                // LANG_ENGLISH 	"GUARD:  Good to see you, sir.  Taylor is waiting for you in the clean tent.  We need to get you suited up.  "
                //
                let psReference = self.GetCurrentReference_ParseOnly();
                if !psReference.is_empty() {
                    let mut line_rest = &psLine_mut[sSE_KEYWORD_LANG.len()..];

                    // what language is this?...
                    //
                    let mut psWordEnd = line_rest;
                    for (i, c) in line_rest.chars().enumerate() {
                        if c != ' ' && c != '\t' {
                            psWordEnd = &line_rest[i..];
                        } else {
                            break;
                        }
                    }
                    let iCharsToCopy = psWordEnd.len().min(iSE_MAX_FILENAME_LENGTH - 1);
                    let mut sThisLanguage = String::new();
                    if iCharsToCopy > 0 {
                        sThisLanguage = line_rest[..iCharsToCopy].to_string();
                    }

                    line_rest = &line_rest[iCharsToCopy..];
                    while !line_rest.is_empty() && line_rest.chars().next() != Some('"') {
                        line_rest = &line_rest[1..];
                    }
                    let _psSentence = self.ConvertCRLiterals_Read(&self.InsideQuotes(line_rest));

                    // Dammit, I hate having to do crap like this just because other people mess up and put
                    //	stupid data in their text, so I have to cope with it.
                    //
                    // note hackery with _psSentence and psSentence because of const-ness. bleurgh. Just don't ask.
                    //
                    let psSentence = CopeWithDumbStringData(&_psSentence, &sThisLanguage);

                    if self.m_bLoadingEnglish_ParseOnly == SE_TRUE {
                        // if loading just "english", then go ahead and store it...
                        //
                        self.SetString(&psReference, &psSentence, SE_FALSE);
                    } else {
                        // if loading a foreign language...
                        //
                        let bSentenceIsEnglish = if sThisLanguage.to_lowercase() == "english" { SE_TRUE } else { SE_FALSE };	// see whether this is the english master or not

                        // this check can be omitted, I'm just being extra careful here...
                        //
                        if bSentenceIsEnglish == SE_FALSE {
                            // basically this is just checking that an .STE file override is the same language as the .STR...
                            //
                            if self.m_strLoadingLanguage_ParseOnly.to_lowercase() != sThisLanguage.to_lowercase() {
                                psErrorMessage = Some(format!("Language \"{}\" found when expecting \"{}\"!\n", sThisLanguage, self.m_strLoadingLanguage_ParseOnly));
                            }
                        }

                        if psErrorMessage.is_none() {
                            self.SetString(&psReference, &psSentence, bSentenceIsEnglish);
                        }
                    }
                } else {
                    psErrorMessage = Some(format!("Error parsing file: Unexpected \"{}\"\n", sSE_KEYWORD_LANG));
                }
            } else {
                psErrorMessage = Some(format!("Unknown keyword at linestart: \"{}\"\n", psLine));
            }
        }

        psErrorMessage
    }

    fn EndMarkerFoundDuringParse(&self) -> SE_BOOL {
        self.m_bEndMarkerFound_ParseOnly
    }
}

static mut TheStringPackage: Option<CStringEdPackage> = None;

fn GetTheStringPackage() -> &'static mut CStringEdPackage {
    unsafe {
        if TheStringPackage.is_none() {
            TheStringPackage = Some(CStringEdPackage::new());
        }
        TheStringPackage.as_mut().unwrap()
    }
}

// this copes with both foreigners using hi-char values (eg the french using 0x92 instead of 0x27
//  for a "'" char), as well as the fact that our buggy fontgen program writes out zeroed glyph info for
//	some fonts anyway (though not all, just as a gotcha).
//
// New bit, instead of static buffer (since XBox guys are desperately short of mem) I return a malloc'd buffer now,
//	so remember to free it!
//
fn CopeWithDumbStringData(psSentence: &str, psThisLanguage: &str) -> String {
    let mut psNewString = psSentence.to_string();

    // this is annoying, I have to just guess at which languages to do it for (ie NOT ASIAN/MBCS!!!) since the
    //	string system was deliberately (and correctly) designed to not know or care whether it was doing SBCS
    //	or MBCS languages, because it was never envisioned that I'd have to clean up other people's mess.
    //
    // Ok, bollocks to it, this will have to do. Any other languages that come later and have bugs in their text can
    //	get fixed by them typing it in properly in the first place...
    //
    let lang_lower = psThisLanguage.to_uppercase();
    if lang_lower == "ENGLISH"
        || lang_lower == "FRENCH"
        || lang_lower == "GERMAN"
        || lang_lower == "ITALIAN"
        || lang_lower == "SPANISH"
        || lang_lower == "POLISH"
        || lang_lower == "RUSSIAN" {

        //	strXLS_Speech.Replace(va("%c",0x92),va("%c",0x27));	// "'"
        while psNewString.contains('\u{0092}') {  // "rich" (and illegal) apostrophe
            psNewString = psNewString.replace('\u{0092}', "\u{0027}");
        }

        //	strXLS_Speech.Replace(va("%c",0x93),"\"");			// smart quotes -> '"'
        while psNewString.contains('\u{0093}') {
            psNewString = psNewString.replace('\u{0093}', "\"");
        }

        //	strXLS_Speech.Replace(va("%c",0x94),"\"");			// smart quotes -> '"'
        while psNewString.contains('\u{0094}') {
            psNewString = psNewString.replace('\u{0094}', "\"");
        }

        //	strXLS_Speech.Replace(va("%c",0x0B),".");			// full stop
        while psNewString.contains('\u{000B}') {
            psNewString = psNewString.replace('\u{000B}', ".");
        }

        //	strXLS_Speech.Replace(va("%c",0x85),"...");			// "..."-char ->  3-char "..."
        while let Some(pos) = psNewString.find('\u{0085}') {  // "rich" (and illegal) apostrophe
            psNewString.remove(pos);
            psNewString.insert_str(pos, "...");
        }

        //	strXLS_Speech.Replace(va("%c",0x91),va("%c",0x27));	// "'"
        while psNewString.contains('\u{0091}') {
            psNewString = psNewString.replace('\u{0091}', "\u{0027}");
        }

        //	strXLS_Speech.Replace(va("%c",0x96),va("%c",0x2D));	// "-"
        while psNewString.contains('\u{0096}') {
            psNewString = psNewString.replace('\u{0096}', "\u{002D}");
        }

        //	strXLS_Speech.Replace(va("%c",0x97),va("%c",0x2D));	// "-"
        while psNewString.contains('\u{0097}') {
            psNewString = psNewString.replace('\u{0097}', "\u{002D}");
        }

        // bug fix for picky grammatical errors, replace "?." with "? "
        //
        while psNewString.contains("?.") {
            psNewString = psNewString.replace("?.", "? ");
        }

        // StripEd and our print code don't support tabs...
        //
        while psNewString.contains('\u{0009}') {
            psNewString = psNewString.replace('\u{0009}', " ");
        }
    }

    psNewString
}

fn Leetify(psString: &str) -> String {
    let mut str = psString.to_string();
    unsafe {
        if !sp_leet.is_null() && (*sp_leet).integer == 42 {	// very specific test, so you won't hit it accidentally
            let cReplace: &[u8] = &[
                b'o', b'0', b'l', b'1', b'e', b'3', b'a', b'4', b's', b'5', b't', b'7', b'i', b'!', b'h', b'#',
                b'O', b'0', b'L', b'1', b'E', b'3', b'A', b'4', b'S', b'5', b'T', b'7', b'I', b'!', b'H', b'#'	// laziness because of strchr()
            ];

            let mut i = 0;
            while i < cReplace.len() {
                let from_char = cReplace[i] as char;
                let to_char = cReplace[i + 1] as char;
                str = str.replace(from_char, &to_char.to_string());
                i += 2;
            }
        }
    }

    str
}

// filename is local here, eg:	"strings/german/obj.str"
//
// return is either NULL for good else error message to display...
//
fn SE_Load_Actual(psFileName: &str, bLoadDebug: SE_BOOL, bSpeculativeLoad: SE_BOOL) -> Option<String> {
    let psErrorMessage: Option<String> = None;

    unsafe {
        let psLoadedData = SE_LoadFileData(CString::new(psFileName).unwrap().as_ptr());
        if !psLoadedData.is_null() {
            // now parse the data...
            //
            let loaded_slice = std::slice::from_raw_parts(psLoadedData, 1024*1024); // Rough upper bound
            let mut psParsePos = loaded_slice;

            GetTheStringPackage().SetupNewFileParse(psFileName, bLoadDebug);

            let mut sLineBuffer = String::new();
            while psErrorMessage.is_none() && GetTheStringPackage().ReadLine(&mut psParsePos, &mut sLineBuffer) == SE_TRUE {
                if !sLineBuffer.is_empty() {
                    if let Some(err) = GetTheStringPackage().ParseLine(&sLineBuffer) {
                        return Some(err);
                    }
                }
            }

            SE_FreeFileDataAfterLoad(psLoadedData as *mut c_void);

            if psErrorMessage.is_none() && GetTheStringPackage().EndMarkerFoundDuringParse() == SE_FALSE {
                return Some(format!("Truncated file, failed to find \"{}\" at file end!", sSE_KEYWORD_ENDMARKER));
            }
        } else {
            if bSpeculativeLoad == SE_FALSE {
                return Some(format!("Unable to load \"{}\"!", psFileName));
            }
        }
    }

    psErrorMessage
}

fn SE_GetFoundFile(strResult: &mut String) -> Option<String> {
    if strResult.is_empty() {
        return None;
    }

    let psSemiColon = strResult.find(';');
    let sTemp = if let Some(pos) = psSemiColon {
        let result = strResult[..pos].to_string();
        strResult.drain(..=pos);
        Some(result)
    } else {
        // no semicolon found, probably last entry? (though i think even those have them on, oh well)
        //
        let result = strResult.clone();
        strResult.clear();
        Some(result)
    };

    sTemp
}

//////////// API entry points from rest of game.... //////////////////////////////

// filename is local here, eg:	"strings/german/obj.str"
//
// return is either NULL for good else error message to display...
//
pub fn SE_Load(psFileName: &str, bLoadDebug: SE_BOOL, bFailIsCritical: SE_BOOL) -> Option<String> {
    ////////////////////////////////////////////////////
    //
    // ingame here tends to pass in names without paths, but I expect them when doing a language load, so...
    //
    let mut sTemp = String::new();
    if !psFileName.contains('/') {
        sTemp.push_str(sSE_STRINGS_DIR);
        sTemp.push('/');
        unsafe {
            if !se_language.is_null() {
                if let Ok(cstr) = CStr::from_ptr((*se_language).string).to_str() {
                    sTemp.push_str(cstr);
                    sTemp.push('/');
                }
            }
        }
    }
    sTemp.push_str(psFileName);
    // COM_DefaultExtension would be called here but we'll skip it for now
    let psFileName_final = if !sTemp.ends_with(sSE_INGAME_FILE_EXTENSION) {
        format!("{}{}", sTemp, sSE_INGAME_FILE_EXTENSION)
    } else {
        sTemp
    };
    //
    ////////////////////////////////////////////////////

    let psErrorMessage = SE_Load_Actual(&psFileName_final, bLoadDebug, SE_FALSE);

    // check for any corresponding / overriding .STE files and load them afterwards...
    //
    if psErrorMessage.is_none() {
        let mut sFileName = psFileName_final.clone();
        if let Some(pos) = sFileName.rfind('.') {
            if sFileName.len() - pos == sSE_EXPORT_FILE_EXTENSION.len() {
                sFileName.truncate(pos);
                sFileName.push_str(sSE_EXPORT_FILE_EXTENSION);

                if let Some(err) = SE_Load_Actual(&sFileName, bLoadDebug, SE_TRUE) {
                    return Some(err);
                }
            }
        }
    }

    if let Some(err) = psErrorMessage {
        if bFailIsCritical == SE_TRUE {
            unsafe {
                Com_Error(1, CString::new(format!("SE_Load(): Couldn't load \"{}\"!\n\nError: \"{}\"\n", psFileName_final, err)).unwrap().as_ptr());
            }
        } else {
            unsafe {
                Com_DPrintf(CString::new(format!("SE_Load(): Couldn't load \"{}\"!\n", psFileName_final)).unwrap().as_ptr());
            }
        }
    }

    psErrorMessage
}

// convenience-function for the main GetString call...
//
pub fn SE_GetString_package(psPackageReference: &str, psStringReference: &str) -> String {
    let sReference = format!("{}_{}", psPackageReference, psStringReference);
    SE_GetString(&sReference)
}

pub fn SE_GetString(psPackageAndStringReference: &str) -> String {
    let mut sReference = psPackageAndStringReference.to_string();
    sReference.make_ascii_uppercase();

    unsafe {
        let pkg = GetTheStringPackage();
        if let Some(entry) = pkg.m_StringEntries.get(&sReference) {
            if !se_debug.is_null() && (*se_debug).integer != 0 && pkg.m_bLoadDebug == SE_TRUE {
                return entry.m_strDebug.clone();
            } else {
                return entry.m_strString.clone();
            }
        }
    }

    // should never get here, but fall back anyway... (except we DO use this to see if there's a debug-friendly key bind, which may not exist)
    //
    String::new()	// you may want to replace this with something based on _DEBUG or not?
}

// convenience-function for the main GetFlags call...
//
pub fn SE_GetFlags_package(psPackageReference: &str, psStringReference: &str) -> c_int {
    let sReference = format!("{}_{}", psPackageReference, psStringReference);
    SE_GetFlags(&sReference)
}

pub fn SE_GetFlags(psPackageAndStringReference: &str) -> c_int {
    unsafe {
        let pkg = GetTheStringPackage();
        if let Some(entry) = pkg.m_StringEntries.get(psPackageAndStringReference) {
            return entry.m_iFlags;
        }
    }

    // should never get here, but fall back anyway...
    //
    0
}

pub fn SE_GetNumFlags() -> c_int {
    unsafe {
        GetTheStringPackage().m_vstrFlagNames.len() as c_int
    }
}

pub fn SE_GetFlagName(iFlagIndex: c_int) -> String {
    unsafe {
        let pkg = GetTheStringPackage();
        if iFlagIndex >= 0 && (iFlagIndex as usize) < pkg.m_vstrFlagNames.len() {
            return pkg.m_vstrFlagNames[iFlagIndex as usize].clone();
        }
    }

    String::new()
}

// returns flag bitmask (eg 00000010b), else 0 for not found
//
pub fn SE_GetFlagMask(psFlagName: &str) -> c_int {
    unsafe {
        GetTheStringPackage().GetFlagMask(psFlagName)
    }
}

// I could cache the result of this since it won't change during app lifetime unless someone does a build-publish
//	while you're still ingame. Cacheing would make sense since it can take a while to scan, but I'll leave it and
//	let whoever calls it cache the results instead. I'll make it known that it's a slow process to call this, but
//	whenever anyone calls someone else's code they should assign it to an int anyway, since you've no idea what's
//	going on. Basically, don't  use this in a FOR loop as the end-condition. Duh.
//
// Groan, except for Bob. I mentioned that this was slow and only call it once, but he's calling it from
//	every level-load...  Ok, cacheing it is...
//
static mut gvLanguagesAvailable: Option<Vec<String>> = None;

pub fn SE_GetNumLanguages() -> c_int {
    unsafe {
        if gvLanguagesAvailable.is_none() {
            gvLanguagesAvailable = Some(Vec::new());
            let mut strResults = String::new();

            #[cfg(feature = "stringed")]
            let path = format!("C:\\Source\\Tools\\StringEd\\test_data\\{}", sSE_STRINGS_DIR);
            #[cfg(not(feature = "stringed"))]
            let path = sSE_STRINGS_DIR.to_string();

            let _iFilesFound = SE_BuildFileList(CString::new(&path).unwrap().as_ptr(), &mut strResults);

            let mut strUniqueStrings: HashSet<String> = HashSet::new();	// laziness <g>
            while let Some(p) = SE_GetFoundFile(&mut strResults) {
                let psLanguage = GetTheStringPackage().ExtractLanguageFromPath(&p);

                if !strUniqueStrings.contains(&psLanguage) {
                    strUniqueStrings.insert(psLanguage.clone());

                    // if english is available, it should always be first... ( I suppose )
                    //
                    if psLanguage.to_lowercase() == "english" {
                        gvLanguagesAvailable.as_mut().unwrap().insert(0, psLanguage);
                    } else {
                        gvLanguagesAvailable.as_mut().unwrap().push(psLanguage);
                    }
                }
            }
        }

        gvLanguagesAvailable.as_ref().unwrap().len() as c_int
    }
}

// SE_GetNumLanguages() must have been called before this...
//
pub fn SE_GetLanguageName(iLangIndex: c_int) -> String {
    unsafe {
        if let Some(ref langs) = gvLanguagesAvailable {
            if iLangIndex >= 0 && (iLangIndex as usize) < langs.len() {
                return langs[iLangIndex as usize].clone();
            }
        }
    }

    String::new()
}

// SE_GetNumLanguages() must have been called before this...
//
pub fn SE_GetLanguageDir(iLangIndex: c_int) -> String {
    unsafe {
        if let Some(ref langs) = gvLanguagesAvailable {
            if iLangIndex >= 0 && (iLangIndex as usize) < langs.len() {
                return format!("{}/{}", sSE_STRINGS_DIR, langs[iLangIndex as usize]);
            }
        }
    }

    String::new()
}

pub fn SE_NewLanguage() {
    unsafe {
        GetTheStringPackage().Clear(SE_TRUE);
    }
}

// these two functions aren't needed other than to make Quake-type games happy and/or stop memory managers
//	complaining about leaks if they report them before the global StringEd package object calls it's own dtor.
//
// but here they are for completeness's sake I guess...
//
pub fn SE_Init() {
    // #ifdef _XBOX	// VVFIXME?
    // //	extern void Z_SetNewDeleteTemporary(bool);
    // //	Z_SetNewDeleteTemporary(true);
    // #endif

    unsafe {
        GetTheStringPackage().Clear(SE_FALSE);

        // #ifdef _DEBUG
        // //	int iNumLanguages = SE_GetNumLanguages();
        // #endif

        se_language = Cvar_Get(
            CString::new("se_language").unwrap().as_ptr(),
            CString::new("english").unwrap().as_ptr(),
            0x0001 | 0x0004 // CVAR_ARCHIVE | CVAR_NORESTART
        );
        se_debug = Cvar_Get(
            CString::new("se_debug").unwrap().as_ptr(),
            CString::new("0").unwrap().as_ptr(),
            0
        );
        sp_leet = Cvar_Get(
            CString::new("sp_leet").unwrap().as_ptr(),
            CString::new("0").unwrap().as_ptr(),
            0x2000 // CVAR_ROM
        );

        // if doing a buildscript, load all languages...
        //
        // extern cvar_t *com_buildScript;
        // if (com_buildScript->integer == 2)
        // {
        //     int iLanguages = SE_GetNumLanguages();
        //     for (int iLang = 0; iLang < iLanguages; iLang++)
        //     {
        //         LPCSTR psLanguage = SE_GetLanguageName( iLang );	// eg "german"
        //         Com_Printf( "com_buildScript(2): Loading language \"%s\"...\n", psLanguage );
        //         SE_LoadLanguage( psLanguage );
        //     }
        // }

        if let Ok(lang_str) = CStr::from_ptr((*se_language).string).to_str() {
            if let Some(err) = SE_LoadLanguage(lang_str, SE_TRUE) {
                Com_Error(1, CString::new(format!("SE_Init() Unable to load language: \"{}\"!\nError: \"{}\"\n", lang_str, err)).unwrap().as_ptr());
            }
        }

        // #ifdef _XBOX
        // //	Z_SetNewDeleteTemporary(false);
        // #endif
    }
}

pub fn SE_ShutDown() {
    unsafe {
        GetTheStringPackage().Clear(SE_FALSE);
    }
}

// returns error message else NULL for ok.
//
// Any errors that result from this should probably be treated as game-fatal, since an asset file is fuxored.
//
pub fn SE_LoadLanguage(psLanguage: &str, bLoadDebug: SE_BOOL) -> Option<String> {
    if psLanguage.is_empty() {
        return None;
    }

    unsafe {
        SE_NewLanguage();

        let mut strResults = String::new();

        #[cfg(feature = "stringed")]
        let path = format!("C:\\Source\\Tools\\StringEd\\test_data\\{}", sSE_STRINGS_DIR);
        #[cfg(not(feature = "stringed"))]
        let path = sSE_STRINGS_DIR.to_string();

        let _iFilesFound = SE_BuildFileList(CString::new(&path).unwrap().as_ptr(), &mut strResults);

        let mut psErrorMessage: Option<String> = None;
        while let Some(p) = SE_GetFoundFile(&mut strResults) {
            if psErrorMessage.is_none() {
                let psThisLang = GetTheStringPackage().ExtractLanguageFromPath(&p);

                if psLanguage.to_lowercase() == psThisLang.to_lowercase() {
                    psErrorMessage = SE_Load(&p, bLoadDebug, SE_FALSE);
                }
            }
        }

        psErrorMessage
    }
}

// called in Com_Frame, so don't take up any time! (can also be called during dedicated)
//
// instead of re-loading just the files we've already loaded I'm going to load the whole language (simpler)
//
pub fn SE_CheckForLanguageUpdates() {
    unsafe {
        if !se_language.is_null() && (*se_language).modified != 0 {
            if let Ok(lang_str) = CStr::from_ptr((*se_language).string).to_str() {
                if let Some(err) = SE_LoadLanguage(lang_str, SE_TRUE) {
                    Com_Error(1, CString::new(err).unwrap().as_ptr());
                }
            }
            (*se_language).modified = SE_FALSE;
        }
    }
}

///////////////////////// eof //////////////////////////
