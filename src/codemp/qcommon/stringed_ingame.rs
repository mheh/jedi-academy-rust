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

use core::ffi::{c_char, c_int, c_void};
use std::collections::BTreeMap;
use std::ffi::CStr;

// copy constructor could not be generated
// assignment operator could not be generated
// C++ language change: blah blah template crap blah blah
// include "stringed_ingame.h"
// include "stringed_interface.h"

///////////////////////////////////////////////
//
// some STL stuff...
// disable the usual stupid and pointless STL warning
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

pub static mut se_language: *mut cvar_t = core::ptr::null_mut();
pub static mut se_debug: *mut cvar_t = core::ptr::null_mut();
pub static mut sp_leet: *mut cvar_t = core::ptr::null_mut(); // kept as 'sp_' for JK2 compat.

// Stub external types and functions (would be defined in other modules)
#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub string: *const c_char,
    pub integer: c_int,
    pub modified: c_int,
    // ... other fields omitted for brevity
}

extern "C" {
    // Declarations for external C functions
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn COM_DefaultExtension(path: *mut c_char, maxlen: usize, ext: *const c_char);
    fn va(fmt: *const c_char, ...) -> *const c_char;

    fn Q_strupr(string: *mut c_char);
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize);

    fn SE_LoadFileData(filename: *const c_char) -> *mut c_uchar;
    fn SE_FreeFileDataAfterLoad(data: *mut c_uchar);
    fn SE_BuildFileList(dir: *const c_char, results: *mut String) -> c_int;
    fn SE_LoadLanguage(language: *const c_char, load_debug: c_int) -> *const c_char;
}

pub type c_uchar = u8;
pub type SE_BOOL = c_int;

const SE_TRUE: SE_BOOL = 1;
const SE_FALSE: SE_BOOL = 0;

// Macro equivalent constants (from stringed_interface.h, assumed)
static sSE_KEYWORD_VERSION: &[u8] = b"VERSION";
static sSE_KEYWORD_CONFIG: &[u8] = b"CONFIG";
static sSE_KEYWORD_FILENOTES: &[u8] = b"FILENOTES";
static sSE_KEYWORD_NOTES: &[u8] = b"NOTES";
static sSE_KEYWORD_REFERENCE: &[u8] = b"REFERENCE";
static sSE_KEYWORD_FLAGS: &[u8] = b"FLAGS";
static sSE_KEYWORD_ENDMARKER: &[u8] = b"ENDMARKER";
static sSE_KEYWORD_LANG: &[u8] = b"LANG_";
static sSE_STRINGS_DIR: &[u8] = b"strings";
static sSE_INGAME_FILE_EXTENSION: &[u8] = b".str";
static sSE_EXPORT_FILE_EXTENSION: &[u8] = b".ste";
static sSE_EXPORT_SAME: &[u8] = b"#same";
static sSE_DEBUGSTR_PREFIX: &[u8] = b"SE:";
static sSE_DEBUGSTR_SUFFIX: &[u8] = b"";
static iSE_VERSION: c_int = 1;
static iSE_MAX_FILENAME_LENGTH: usize = 260;

const ERR_DROP: c_int = 1;
const CVAR_ARCHIVE: c_int = 1;
const CVAR_NORESTART: c_int = 2;
const CVAR_ROM: c_int = 4;
const TAG_TEMP_WORKSPACE: c_int = 3;

#[repr(C)]
#[derive(Clone)]
pub struct SE_Entry_s {
    pub m_strString: String,
    pub m_strDebug: String, // english and/or "#same", used for debugging only. Also prefixed by "SE:" to show which strings go through StringEd (ie aren't hardwired)
    pub m_iFlags: c_int,
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

// Map from string to SE_Entry_t
type mapStringEntries_t = BTreeMap<String, SE_Entry_s>;

pub struct CStringEdPackage {
    // Private members
    m_bEndMarkerFound_ParseOnly: SE_BOOL,
    m_strCurrentEntryRef_ParseOnly: String,
    m_strCurrentEntryEnglish_ParseOnly: String,
    m_strCurrentFileRef_ParseOnly: String,
    m_strLoadingLanguage_ParseOnly: String, // eg "german"
    m_bLoadingEnglish_ParseOnly: SE_BOOL,

    // Public members
    pub m_StringEntries: mapStringEntries_t,
    pub m_bLoadDebug: SE_BOOL,

    // flag stuff...
    pub m_vstrFlagNames: Vec<String>,
    pub m_mapFlagMasks: BTreeMap<String, c_int>,
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
            m_StringEntries: BTreeMap::new(),
            m_bLoadDebug: SE_FALSE,
            m_vstrFlagNames: Vec::new(),
            m_mapFlagMasks: BTreeMap::new(),
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
            (Some(a), Some(b)) => Some(if a > b { a } else { b }),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
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
        if let Some(dot_pos) = p {
            let valid = (p2.is_none() || (p2.is_some() && dot_pos > p2.unwrap())) &&
                        (p3.is_none() || (p3.is_some() && dot_pos > p3.unwrap()));
            if valid {
                sString.truncate(dot_pos);
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
        let mut psCopyPos = 0usize;

        for (i, ch) in psFilename.char_indices() {
            if ch == '/' || ch == '\\' {
                psCopyPos = i + 1;
            }
        }

        if psCopyPos < psFilename.len() {
            psFilename[psCopyPos..].to_string()
        } else {
            String::new()
        }
    }

    fn ExtractLanguageFromPath(&self, psFileName: &str) -> String {
        self.Filename_WithoutPath(&self.Filename_PathOnly(psFileName))
    }

    fn SetupNewFileParse(&mut self, psFileName: &str, bLoadDebug: SE_BOOL) {
        let mut sString = self.Filename_WithoutPath(&self.Filename_WithoutExt(psFileName));
        sString.make_ascii_uppercase();

        self.m_strCurrentFileRef_ParseOnly = sString; // eg "OBJECTIVES"
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
            str.replace_range(iLoc..iLoc + 1, "\n");
        }

        str
    }

    // kill off any "//" onwards part in the line, but NOT if it's inside a quoted string...
    //
    fn REMKill(&self, psBuffer: &mut String) {
        let mut psScanPos = 0usize;
        let mut iDoubleQuotesSoFar = 0;

        // scan forwards in case there are more than one (and the first is inside quotes)...
        //
        while let Some(p) = psBuffer[psScanPos..].find("//") {
            let p = psScanPos + p;

            // count the number of double quotes before this point, if odd number, then we're inside quotes...
            //
            let mut iDoubleQuoteCount = iDoubleQuotesSoFar;

            for i in psScanPos..p {
                if psBuffer.chars().nth(i).unwrap_or('\0') == '"' {
                    iDoubleQuoteCount += 1;
                }
            }

            if (iDoubleQuoteCount & 1) == 0 {
                // not inside quotes, so kill line here...
                //
                psBuffer.truncate(p);
                //
                // and remove any trailing whitespace...
                //
                if !psBuffer.is_empty() {
                    let mut iWhiteSpaceScanPos = psBuffer.len() as i32 - 1;
                    while iWhiteSpaceScanPos >= 0 && psBuffer.chars().nth(iWhiteSpaceScanPos as usize).unwrap_or('\0').is_whitespace() {
                        psBuffer.pop();
                        iWhiteSpaceScanPos -= 1;
                    }
                }

                return;
            } else {
                // inside quotes (blast), oh well, skip past and keep scanning...
                //
                psScanPos = p + 1;
                iDoubleQuotesSoFar = iDoubleQuoteCount;
            }
        }
    }

    // returns true while new lines available to be read...
    //
    fn ReadLine(&self, psParsePos: &mut &str, psDest: &mut String) -> SE_BOOL {
        if !psParsePos.is_empty() {
            if let Some(psLineEnd_pos) = psParsePos.find('\n') {
                let iCharsToCopy = psLineEnd_pos;
                psDest.clear();
                psDest.push_str(&psParsePos[..iCharsToCopy]);
                *psParsePos = &psParsePos[iCharsToCopy..];

                while !psParsePos.is_empty() && ("\r\n".contains(psParsePos.chars().next().unwrap())) {
                    *psParsePos = &psParsePos[1..]; // skip over CR or CR/LF pairs
                }
            } else {
                // last line...
                //
                psDest.clear();
                psDest.push_str(psParsePos);
                *psParsePos = "";
            }

            // clean up the line...
            //
            if !psDest.is_empty() {
                let mut iWhiteSpaceScanPos = psDest.len() as i32 - 1;
                while iWhiteSpaceScanPos >= 0 {
                    let ch = psDest.chars().nth(iWhiteSpaceScanPos as usize).unwrap_or('\0');
                    if ch.is_whitespace() {
                        psDest.pop();
                        iWhiteSpaceScanPos -= 1;
                    } else {
                        break;
                    }
                }

                self.REMKill(psDest);
            }
            return SE_TRUE;
        }

        SE_FALSE
    }

    // remove any outside quotes from this supplied line, plus any leading or trailing whitespace...
    //
    fn InsideQuotes(&self, psLine: &str) -> String {
        // I *could* replace this string object with a declared array, but wasn't sure how big to leave it, and it'd have to
        //	be static as well, hence permanent. (problem on consoles?)
        //
        let mut str = String::new();
        let mut line = psLine;

        // skip any leading whitespace...
        //
        while !line.is_empty() && (line.chars().next().unwrap() == ' ' || line.chars().next().unwrap() == '\t') {
            line = &line[1..];
        }

        // skip any leading quote...
        //
        if !line.is_empty() && line.chars().next().unwrap() == '"' {
            line = &line[1..];
        }

        // assign it...
        //
        str = line.to_string();

        if !line.is_empty() {
            // lose any trailing whitespace...
            //
            while !str.is_empty() {
                let last_ch = str.chars().last().unwrap();
                if last_ch == ' ' || last_ch == '\t' {
                    str.pop();
                } else {
                    break;
                }
            }

            // lose any trailing quote...
            //
            if !str.is_empty() && str.chars().last().unwrap() == '"' {
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
        if let Some(Entry) = self.m_StringEntries.get_mut(&full_ref) {
            Entry.m_iFlags |= iMask;
        }
    }

    fn GetCurrentReference_ParseOnly(&self) -> String {
        self.m_strCurrentEntryRef_ParseOnly.clone()
    }

    // add new string entry (during parse)
    //
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

    fn SetString(&mut self, psLocalReference: &str, psNewString: &str, bEnglishDebug: SE_BOOL) {
        let full_ref = format!("{}_{}", self.m_strCurrentFileRef_ParseOnly, psLocalReference);
        if let Some(Entry) = self.m_StringEntries.get_mut(&full_ref) {
            if bEnglishDebug != 0 || self.m_bLoadingEnglish_ParseOnly != 0 {
                // then this is the leading english text of a foreign sentence pair (so it's the debug-key text),
                //	or it's the only text when it's english being loaded...
                //
                Entry.m_strString = Leetify(psNewString);
                if self.m_bLoadDebug != 0 {
                    Entry.m_strDebug = String::from_utf8_lossy(sSE_DEBUGSTR_PREFIX).to_string();
                    Entry.m_strDebug.push_str(psNewString);
                    Entry.m_strDebug.push_str(&String::from_utf8_lossy(sSE_DEBUGSTR_SUFFIX));
                }
                self.m_strCurrentEntryEnglish_ParseOnly = psNewString.to_string(); // for possible "#same" resolving in foreign later
            } else {
                // then this is foreign text (so check for "#same" resolving)...
                //
                if psNewString.to_lowercase() == String::from_utf8_lossy(sSE_EXPORT_SAME).to_lowercase() {
                    Entry.m_strString = self.m_strCurrentEntryEnglish_ParseOnly.clone(); // foreign "#same" is now english
                    if self.m_bLoadDebug != 0 {
                        Entry.m_strDebug = String::from_utf8_lossy(sSE_DEBUGSTR_PREFIX).to_string();
                        Entry.m_strDebug.push_str(&String::from_utf8_lossy(sSE_EXPORT_SAME)); // english (debug) is now "#same"
                        Entry.m_strDebug.push_str(&String::from_utf8_lossy(sSE_DEBUGSTR_SUFFIX));
                    }
                } else {
                    Entry.m_strString = psNewString.to_string(); // foreign is just foreign
                }
            }
        } else {
            // __ASSERT(0);	// should never happen
        }
    }

    fn ParseLine(&mut self, psLine: &str) -> Option<String> {
        let mut psErrorMessage: Option<String> = None;

        if !psLine.is_empty() {
            let mut line = psLine;

            if self.CheckLineForKeyword(std::str::from_utf8(sSE_KEYWORD_VERSION).unwrap(), &mut line) == SE_TRUE {
                // VERSION 	"1"
                //
                let psVersionNumber = self.InsideQuotes(&line);
                let iVersionNumber: c_int = psVersionNumber.parse().unwrap_or(0);

                if iVersionNumber != iSE_VERSION {
                    psErrorMessage = Some(format!("Unexpected version number {}, expecting {}!\n", iVersionNumber, iSE_VERSION));
                }
            } else if self.CheckLineForKeyword(std::str::from_utf8(sSE_KEYWORD_CONFIG).unwrap(), &mut line) == SE_TRUE
                || self.CheckLineForKeyword(std::str::from_utf8(sSE_KEYWORD_FILENOTES).unwrap(), &mut line) == SE_TRUE
                || self.CheckLineForKeyword(std::str::from_utf8(sSE_KEYWORD_NOTES).unwrap(), &mut line) == SE_TRUE
            {
                // not used ingame, but need to absorb the token
            } else if self.CheckLineForKeyword(std::str::from_utf8(sSE_KEYWORD_REFERENCE).unwrap(), &mut line) == SE_TRUE {
                // REFERENCE	GUARD_GOOD_TO_SEE_YOU
                //
                let psLocalReference = self.InsideQuotes(&line);
                self.AddEntry(&psLocalReference);
            } else if self.CheckLineForKeyword(std::str::from_utf8(sSE_KEYWORD_FLAGS).unwrap(), &mut line) == SE_TRUE {
                // FLAGS 	FLAG_CAPTION FLAG_TYPEMATIC
                //
                let psReference = self.GetCurrentReference_ParseOnly();
                if !psReference.is_empty() {
                    let sSeperators = " \t";
                    let mut sFlags = line.to_string();

                    for psToken in sFlags.split_whitespace() {
                        let mut token_upper = psToken.to_string();
                        token_upper.make_ascii_uppercase();
                        self.AddFlagReference(&psReference, &token_upper);
                    }
                } else {
                    psErrorMessage = Some(format!("Error parsing file: Unexpected \"{}\"\n", std::str::from_utf8(sSE_KEYWORD_FLAGS).unwrap_or("")));
                }
            } else if self.CheckLineForKeyword(std::str::from_utf8(sSE_KEYWORD_ENDMARKER).unwrap(), &mut line) == SE_TRUE {
                // ENDMARKER
                //
                self.m_bEndMarkerFound_ParseOnly = SE_TRUE; // the only major error checking I bother to do (for file truncation)
            } else if line.starts_with(std::str::from_utf8(sSE_KEYWORD_LANG).unwrap()) {
                // LANG_ENGLISH 	"GUARD:  Good to see you, sir.  Taylor is waiting for you in the clean tent.  We need to get you suited up.  "
                //
                let psReference = self.GetCurrentReference_ParseOnly();
                if !psReference.is_empty() {
                    line = &line[std::str::from_utf8(sSE_KEYWORD_LANG).unwrap().len()..];

                    // what language is this?...
                    //
                    let mut psWordEnd = 0usize;
                    for (i, ch) in line.char_indices() {
                        if ch == ' ' || ch == '\t' {
                            psWordEnd = i;
                            break;
                        }
                    }
                    if psWordEnd == 0 {
                        psWordEnd = line.len();
                    }

                    let sThisLanguage = &line[..psWordEnd];

                    let line_rest = &line[psWordEnd..];
                    let _psSentence = self.ConvertCRLiterals_Read(&self.InsideQuotes(line_rest));

                    // Dammit, I hate having to do crap like this just because other people mess up and put
                    //	stupid data in their text, so I have to cope with it.
                    //
                    // note hackery with _psSentence and psSentence because of const-ness. bleurgh. Just don't ask.
                    //
                    let psSentence = CopeWithDumbStringData(&_psSentence, sThisLanguage);

                    if self.m_bLoadingEnglish_ParseOnly != 0 {
                        // if loading just "english", then go ahead and store it...
                        //
                        self.SetString(&psReference, &psSentence, SE_FALSE);
                    } else {
                        // if loading a foreign language...
                        //
                        let bSentenceIsEnglish = if sThisLanguage.to_lowercase() == "english" { SE_TRUE } else { SE_FALSE }; // see whether this is the english master or not

                        // this check can be omitted, I'm just being extra careful here...
                        //
                        if bSentenceIsEnglish == 0 {
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
                    psErrorMessage = Some(format!("Error parsing file: Unexpected \"{}\"\n", std::str::from_utf8(sSE_KEYWORD_LANG).unwrap_or("")));
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

fn get_the_string_package() -> &'static mut CStringEdPackage {
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
    let iBufferSize = psSentence.len() * 3; // *3 to allow for expansion of anything even stupid string consisting entirely of elipsis chars
    let mut psNewString = psSentence.to_string();

    // this is annoying, I have to just guess at which languages to do it for (ie NOT ASIAN/MBCS!!!) since the
    //	string system was deliberately (and correctly) designed to not know or care whether it was doing SBCS
    //	or MBCS languages, because it was never envisioned that I'd have to clean up other people's mess.
    //
    // Ok, bollocks to it, this will have to do. Any other languages that come later and have bugs in their text can
    //	get fixed by them typing it in properly in the first place...
    //
    if psThisLanguage.to_uppercase() == "ENGLISH" ||
        psThisLanguage.to_uppercase() == "FRENCH" ||
        psThisLanguage.to_uppercase() == "GERMAN" ||
        psThisLanguage.to_uppercase() == "ITALIAN" ||
        psThisLanguage.to_uppercase() == "SPANISH" ||
        psThisLanguage.to_uppercase() == "POLISH" ||
        psThisLanguage.to_uppercase() == "RUSSIAN"
    {
        //	strXLS_Speech.Replace(va("%c",0x92),va("%c",0x27));	// "'"
        while let Some(pos) = psNewString.find('\u{0092}') {
            psNewString.replace_range(pos..pos+1, "'");
        }

        //	strXLS_Speech.Replace(va("%c",0x93),"\"");			// smart quotes -> '"'
        while let Some(pos) = psNewString.find('\u{0093}') {
            psNewString.replace_range(pos..pos+1, "\"");
        }

        //	strXLS_Speech.Replace(va("%c",0x94),"\"");			// smart quotes -> '"'
        while let Some(pos) = psNewString.find('\u{0094}') {
            psNewString.replace_range(pos..pos+1, "\"");
        }

        //	strXLS_Speech.Replace(va("%c",0x0B),".");			// full stop
        while let Some(pos) = psNewString.find('\u{000B}') {
            psNewString.replace_range(pos..pos+1, ".");
        }

        //	strXLS_Speech.Replace(va("%c",0x85),"...");			// "..."-char ->  3-char "..."
        while let Some(pos) = psNewString.find('\u{0085}') {
            let suffix = &psNewString[pos+1..].to_string();
            psNewString.truncate(pos);
            psNewString.push_str("...");
            psNewString.push_str(suffix);
        }

        //	strXLS_Speech.Replace(va("%c",0x91),va("%c",0x27));	// "'"
        while let Some(pos) = psNewString.find('\u{0091}') {
            psNewString.replace_range(pos..pos+1, "'");
        }

        //	strXLS_Speech.Replace(va("%c",0x96),va("%c",0x2D));	// "-"
        while let Some(pos) = psNewString.find('\u{0096}') {
            psNewString.replace_range(pos..pos+1, "-");
        }

        //	strXLS_Speech.Replace(va("%c",0x97),va("%c",0x2D));	// "-"
        while let Some(pos) = psNewString.find('\u{0097}') {
            psNewString.replace_range(pos..pos+1, "-");
        }

        // bug fix for picky grammatical errors, replace "?." with "? "
        //
        while let Some(pos) = psNewString.find("?.") {
            psNewString.replace_range(pos+1..pos+2, " ");
        }

        // StripEd and our print code don't support tabs...
        //
        while let Some(pos) = psNewString.find('\t') {
            psNewString.replace_range(pos..pos+1, " ");
        }
    }

    psNewString
}

fn Leetify(psString: &str) -> String {
    let mut str = psString.to_string();

    unsafe {
        if !sp_leet.is_null() && (*sp_leet).integer == 42 {
            // very specific test, so you won't hit it accidentally
            static cReplace: &[u8] = b"o0l1e3a4s5t7i!h#O0L1E3A4S5T7I!H#";

            for i in (0..cReplace.len()).step_by(2) {
                let from_ch = cReplace[i] as char;
                let to_ch = cReplace[i + 1] as char;
                str = str.replace(from_ch, &to_ch.to_string());
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
    unsafe {
        let pkg = get_the_string_package();

        let psLoadedData_ptr = SE_LoadFileData(psFileName.as_ptr() as *const c_char);
        if !psLoadedData_ptr.is_null() {
            // now parse the data...
            //
            let mut psParsePos = std::str::from_utf8_unchecked(std::slice::from_raw_parts(psLoadedData_ptr, 1000000)); // large upper bound

            pkg.SetupNewFileParse(psFileName, bLoadDebug);

            let mut sLineBuffer = String::new();
            while pkg.ReadLine(&mut psParsePos, &mut sLineBuffer) == SE_TRUE {
                if !sLineBuffer.is_empty() {
                    //				__DEBUGOUT( sLineBuffer );
                    //				__DEBUGOUT( "\n" );

                    if let Some(err) = pkg.ParseLine(&sLineBuffer) {
                        SE_FreeFileDataAfterLoad(psLoadedData_ptr);
                        return Some(err);
                    }
                }
            }

            SE_FreeFileDataAfterLoad(psLoadedData_ptr);

            if !pkg.EndMarkerFoundDuringParse() == 0 {
                return Some(format!("Truncated file, failed to find \"{}\" at file end!", std::str::from_utf8(sSE_KEYWORD_ENDMARKER).unwrap()));
            }

            return None;
        } else {
            if bSpeculativeLoad == 0 {
                // then it's ok to not find the file, so do nothing...
            } else {
                return Some(format!("Unable to load \"{}\"!", psFileName));
            }
        }
    }

    None
}

fn SE_GetFoundFile(strResult: &mut String) -> Option<String> {
    static mut sTemp: [u8; 1024] = [0; 1024];

    if strResult.is_empty() {
        return None;
    }

    unsafe {
        let bytes = strResult.as_bytes();
        for (i, &b) in bytes.iter().enumerate().take(1023) {
            sTemp[i] = b;
        }
        sTemp[1023] = 0;

        if let Some(semi_pos) = sTemp.iter().position(|&b| b == b';') {
            strResult.drain(..semi_pos + 1);

            Some(std::str::from_utf8_unchecked(&sTemp[..semi_pos]).to_string())
        } else {
            // no semicolon found, probably last entry? (though i think even those have them on, oh well)
            //
            strResult.clear();
            Some(std::str::from_utf8_unchecked(&sTemp).to_string())
        }
    }
}

//////////// API entry points from rest of game.... //////////////////////////////

// filename is local here, eg:	"strings/german/obj.str"
//
// return is either NULL for good else error message to display...
//
pub extern "C" fn SE_Load(psFileName: *const c_char, bLoadDebug: SE_BOOL, bFailIsCritical: SE_BOOL) -> *const c_char {
    let psFileName_str = unsafe { CStr::from_ptr(psFileName).to_str().unwrap_or("") };

    ////////////////////////////////////////////////////
    //
    // ingame here tends to pass in names without paths, but I expect them when doing a language load, so...
    //
    let mut sTemp = String::new();
    if !psFileName_str.contains('/') {
        sTemp.push_str(std::str::from_utf8(sSE_STRINGS_DIR).unwrap());
        sTemp.push('/');

        unsafe {
            if !se_language.is_null() {
                let lang_str = CStr::from_ptr((*se_language).string).to_str().unwrap_or("");
                sTemp.push_str(lang_str);
                sTemp.push('/');
            }
        }
    }
    sTemp.push_str(psFileName_str);

    unsafe {
        let mut temp_cstr = sTemp.as_bytes().to_vec();
        temp_cstr.resize(256, 0);
        COM_DefaultExtension(temp_cstr.as_mut_ptr() as *mut c_char, temp_cstr.len(), sSE_INGAME_FILE_EXTENSION.as_ptr() as *const c_char);
    }

    //
    ////////////////////////////////////////////////////

    if let Some(err) = SE_Load_Actual(&sTemp, bLoadDebug, SE_FALSE) {
        if bFailIsCritical != 0 {
            unsafe {
                let err_c = std::ffi::CString::new(err.as_str()).unwrap();
                Com_Error(ERR_DROP, b"SE_Load(): Couldn't load \"%s\"!\n\nError: \"%s\"\n\0".as_ptr() as *const c_char, sTemp.as_ptr(), err_c.as_ptr());
            }
        } else {
            unsafe {
                Com_DPrintf(b"SE_Load(): Couldn't load \"%s\"!\n\0".as_ptr() as *const c_char, sTemp.as_ptr());
            }
        }

        let err_c = std::ffi::CString::new(err).unwrap();
        return err_c.as_ptr();
    } else {
        // check for any corresponding / overriding .STE files and load them afterwards...
        //
        let mut sFileName = sTemp.clone();
        if let Some(dot_pos) = sFileName.rfind('.') {
            if sFileName[dot_pos..].len() == std::str::from_utf8(sSE_EXPORT_FILE_EXTENSION).unwrap().len() {
                sFileName.truncate(dot_pos);
                sFileName.push_str(std::str::from_utf8(sSE_EXPORT_FILE_EXTENSION).unwrap());

                if let Some(err) = SE_Load_Actual(&sFileName, bLoadDebug, SE_TRUE) {
                    if bFailIsCritical != 0 {
                        unsafe {
                            let err_c = std::ffi::CString::new(err.as_str()).unwrap();
                            Com_Error(ERR_DROP, b"SE_Load(): Couldn't load \"%s\"!\n\nError: \"%s\"\n\0".as_ptr() as *const c_char, sFileName.as_ptr(), err_c.as_ptr());
                        }
                    }
                    let err_c = std::ffi::CString::new(err).unwrap();
                    return err_c.as_ptr();
                }
            }
        }
    }

    std::ptr::null()
}

// convenience-function for the main GetString call...
//
pub extern "C" fn SE_GetString_2(psPackageReference: *const c_char, psStringReference: *const c_char) -> *const c_char {
    let pkg_ref = unsafe { CStr::from_ptr(psPackageReference).to_str().unwrap_or("") };
    let str_ref = unsafe { CStr::from_ptr(psStringReference).to_str().unwrap_or("") };

    let mut sReference = format!("{}_{}", pkg_ref, str_ref);
    sReference.make_ascii_uppercase();

    SE_GetString(sReference.as_ptr() as *const c_char)
}

pub extern "C" fn SE_GetString(psPackageAndStringReference: *const c_char) -> *const c_char {
    let sReference_str = unsafe { CStr::from_ptr(psPackageAndStringReference).to_str().unwrap_or("") };
    let mut sReference = sReference_str.to_string();
    sReference.make_ascii_uppercase();

    let pkg = get_the_string_package();

    if let Some(Entry) = pkg.m_StringEntries.get(&sReference) {
        unsafe {
            if !se_debug.is_null() && (*se_debug).integer != 0 && pkg.m_bLoadDebug != 0 {
                return Entry.m_strDebug.as_ptr() as *const c_char;
            } else {
                return Entry.m_strString.as_ptr() as *const c_char;
            }
        }
    }

    // should never get here, but fall back anyway... (except we DO use this to see if there's a debug-friendly key bind, which may not exist)
    //
    //	__ASSERT(0);
    b"\0".as_ptr() as *const c_char // you may want to replace this with something based on _DEBUG or not?
}

// convenience-function for the main GetFlags call...
//
pub extern "C" fn SE_GetFlags_2(psPackageReference: *const c_char, psStringReference: *const c_char) -> c_int {
    let pkg_ref = unsafe { CStr::from_ptr(psPackageReference).to_str().unwrap_or("") };
    let str_ref = unsafe { CStr::from_ptr(psStringReference).to_str().unwrap_or("") };

    let sReference = format!("{}_{}", pkg_ref, str_ref);

    SE_GetFlags(sReference.as_ptr() as *const c_char)
}

pub extern "C" fn SE_GetFlags(psPackageAndStringReference: *const c_char) -> c_int {
    let sReference_str = unsafe { CStr::from_ptr(psPackageAndStringReference).to_str().unwrap_or("") };

    let pkg = get_the_string_package();
    if let Some(Entry) = pkg.m_StringEntries.get(sReference_str) {
        return Entry.m_iFlags;
    }

    // should never get here, but fall back anyway...
    //
    // __ASSERT(0);

    0
}

pub extern "C" fn SE_GetNumFlags() -> c_int {
    let pkg = get_the_string_package();
    pkg.m_vstrFlagNames.len() as c_int
}

pub extern "C" fn SE_GetFlagName(iFlagIndex: c_int) -> *const c_char {
    let pkg = get_the_string_package();
    if (iFlagIndex as usize) < pkg.m_vstrFlagNames.len() {
        return pkg.m_vstrFlagNames[iFlagIndex as usize].as_ptr() as *const c_char;
    }

    // __ASSERT(0);
    b"\0".as_ptr() as *const c_char
}

// returns flag bitmask (eg 00000010b), else 0 for not found
//
pub extern "C" fn SE_GetFlagMask(psFlagName: *const c_char) -> c_int {
    let flag_name = unsafe { CStr::from_ptr(psFlagName).to_str().unwrap_or("") };
    let pkg = get_the_string_package();
    pkg.GetFlagMask(flag_name)
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

pub extern "C" fn SE_GetNumLanguages() -> c_int {
    unsafe {
        if gvLanguagesAvailable.is_none() {
            gvLanguagesAvailable = Some(Vec::new());
            let languages = gvLanguagesAvailable.as_mut().unwrap();

            let mut strResults = String::new();
            /*int iFilesFound = */
            #[cfg(feature = "_STRINGED")]
            let dir = b"C:\\Source\\Tools\\StringEd\\test_data\\strings\0".as_ptr() as *const c_char;
            #[cfg(not(feature = "_STRINGED"))]
            let dir = sSE_STRINGS_DIR.as_ptr() as *const c_char;

            SE_BuildFileList(dir, &mut strResults);

            use std::collections::HashSet;
            let mut strUniqueStrings: HashSet<String> = HashSet::new();

            let pkg = get_the_string_package();
            while let Some(p) = SE_GetFoundFile(&mut strResults) {
                let psLanguage = pkg.ExtractLanguageFromPath(&p);

                if !strUniqueStrings.contains(&psLanguage) {
                    strUniqueStrings.insert(psLanguage.clone());

                    // if english is available, it should always be first... ( I suppose )
                    //
                    if psLanguage.to_lowercase() == "english" {
                        languages.insert(0, psLanguage);
                    } else {
                        languages.push(psLanguage);
                    }
                }
            }
        }

        gvLanguagesAvailable.as_ref().map(|v| v.len() as c_int).unwrap_or(0)
    }
}

// SE_GetNumLanguages() must have been called before this...
//
pub extern "C" fn SE_GetLanguageName(iLangIndex: c_int) -> *const c_char {
    unsafe {
        if let Some(ref languages) = gvLanguagesAvailable {
            if (iLangIndex as usize) < languages.len() {
                return languages[iLangIndex as usize].as_ptr() as *const c_char;
            }
        }
    }

    // __ASSERT(0);
    b"\0".as_ptr() as *const c_char
}

// SE_GetNumLanguages() must have been called before this...
//
pub extern "C" fn SE_GetLanguageDir(iLangIndex: c_int) -> *const c_char {
    unsafe {
        if let Some(ref languages) = gvLanguagesAvailable {
            if (iLangIndex as usize) < languages.len() {
                let dir_str = format!("{}/{}",
                    std::str::from_utf8(sSE_STRINGS_DIR).unwrap(),
                    languages[iLangIndex as usize]);
                // Note: this is problematic - we're returning a pointer to a local string
                // In the original C code this would be handled by va()
                // This needs proper handling in a real implementation
                return dir_str.as_ptr() as *const c_char;
            }
        }
    }

    // __ASSERT(0);
    b"\0".as_ptr() as *const c_char
}

pub extern "C" fn SE_NewLanguage() {
    let pkg = get_the_string_package();
    pkg.Clear(SE_TRUE);
}

// these two functions aren't needed other than to make Quake-type games happy and/or stop memory managers
//	complaining about leaks if they report them before the global StringEd package object calls it's own dtor.
//
// but here they are for completeness's sake I guess...
//
pub extern "C" fn SE_Init() {
    // #ifdef _XBOX	// VVFIXME?
    // //	extern void Z_SetNewDeleteTemporary(bool);
    // //	Z_SetNewDeleteTemporary(true);
    // #endif

    let pkg = get_the_string_package();
    pkg.Clear(SE_FALSE);

    // #ifdef _DEBUG
    // //	int iNumLanguages = SE_GetNumLanguages();
    // #endif

    unsafe {
        se_language = Cvar_Get(b"se_language\0".as_ptr() as *const c_char, b"english\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_NORESTART);
        se_debug = Cvar_Get(b"se_debug\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        sp_leet = Cvar_Get(b"sp_leet\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ROM);

        // if doing a buildscript, load all languages...
        //
        extern "C" {
            static com_buildScript: *mut cvar_t;
        }

        if !com_buildScript.is_null() && (*com_buildScript).integer == 2 {
            let iLanguages = SE_GetNumLanguages();
            for iLang in 0..iLanguages {
                let psLanguage = SE_GetLanguageName(iLang);
                let lang_str = CStr::from_ptr(psLanguage).to_str().unwrap_or(""); // eg "german"
                Com_Printf(b"com_buildScript(2): Loading language \"%s\"...\n\0".as_ptr() as *const c_char, lang_str.as_ptr());
                SE_LoadLanguage_Internal(lang_str);
            }
        }

        let lang_cstr = CStr::from_ptr((*se_language).string).to_str().unwrap_or("");
        let psErrorMessage = SE_LoadLanguage_Internal(lang_cstr);
        if psErrorMessage.is_some() {
            let err = psErrorMessage.unwrap();
            Com_Error(ERR_DROP, b"SE_Init() Unable to load language: \"%s\"!\nError: \"%s\"\n\0".as_ptr() as *const c_char, lang_cstr.as_ptr(), err.as_ptr());
        }
    }

    // #ifdef _XBOX
    // //	Z_SetNewDeleteTemporary(false);
    // #endif
}

pub extern "C" fn SE_ShutDown() {
    let pkg = get_the_string_package();
    pkg.Clear(SE_FALSE);
}

// returns error message else NULL for ok.
//
// Any errors that result from this should probably be treated as game-fatal, since an asset file is fuxored.
//
fn SE_LoadLanguage_Internal(psLanguage: &str) -> Option<String> {
    if psLanguage.is_empty() {
        // __ASSERT( 0 && "SE_LoadLanguage(): Bad language name!" );
        return None;
    }

    SE_NewLanguage();

    let mut strResults = String::new();
    /*int iFilesFound = */
    #[cfg(feature = "_STRINGED")]
    let dir = b"C:\\Source\\Tools\\StringEd\\test_data\\strings\0".as_ptr() as *const c_char;
    #[cfg(not(feature = "_STRINGED"))]
    let dir = sSE_STRINGS_DIR.as_ptr() as *const c_char;

    unsafe {
        SE_BuildFileList(dir, &mut strResults);
    }

    let pkg = get_the_string_package();
    while let Some(p) = SE_GetFoundFile(&mut strResults) {
        let psThisLang = pkg.ExtractLanguageFromPath(&p);

        if psLanguage.to_lowercase() == psThisLang.to_lowercase() {
            if let Some(err) = SE_Load_Actual(&p, SE_TRUE, SE_FALSE) {
                return Some(err);
            }
        }
    }

    None
}

pub extern "C" fn SE_LoadLanguage(psLanguage: *const c_char, bLoadDebug: SE_BOOL) -> *const c_char {
    let lang_str = unsafe { CStr::from_ptr(psLanguage).to_str().unwrap_or("") };

    if let Some(err) = SE_LoadLanguage_Internal(lang_str) {
        let err_c = std::ffi::CString::new(err).unwrap();
        return err_c.as_ptr();
    }

    std::ptr::null()
}

// called in Com_Frame, so don't take up any time! (can also be called during dedicated)
//
// instead of re-loading just the files we've already loaded I'm going to load the whole language (simpler)
//
pub extern "C" fn SE_CheckForLanguageUpdates() {
    unsafe {
        if !se_language.is_null() && (*se_language).modified != 0 {
            let lang_str = CStr::from_ptr((*se_language).string).to_str().unwrap_or("");
            let psErrorMessage = SE_LoadLanguage_Internal(lang_str);
            if psErrorMessage.is_some() {
                let err = psErrorMessage.unwrap();
                Com_Error(ERR_DROP, std::ffi::CString::new(err).unwrap().as_ptr());
            }
            (*se_language).modified = SE_FALSE;
        }
    }
}

///////////////////////// eof //////////////////////////
