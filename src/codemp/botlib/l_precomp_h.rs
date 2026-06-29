/*****************************************************************************
 * name:		l_precomp.h
 *
 * desc:		pre compiler
 *
 * $Archive: /source/code/botlib/l_precomp.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

use core::ffi::{c_char, c_int};

// Stub declarations for types defined elsewhere in the codebase
pub enum token_t {}
pub enum script_t {}
pub enum punctuation_t {}

// Platform-dependent path separator definitions
#[cfg(any(target_os = "windows"))]
pub const PATHSEPERATOR_STR: &str = "\\";
#[cfg(not(any(target_os = "windows")))]
pub const PATHSEPERATOR_STR: &str = "/";

#[cfg(any(target_os = "windows"))]
pub const PATHSEPERATOR_CHAR: u8 = b'\\';
#[cfg(not(any(target_os = "windows")))]
pub const PATHSEPERATOR_CHAR: u8 = b'/';

pub const DEFINE_FIXED: c_int = 0x0001;
pub const DEFINE_GLOBAL: c_int = 0x0002;

pub const BUILTIN_LINE: c_int = 1;
pub const BUILTIN_FILE: c_int = 2;
pub const BUILTIN_DATE: c_int = 3;
pub const BUILTIN_TIME: c_int = 4;
pub const BUILTIN_STDC: c_int = 5;

pub const INDENT_IF: c_int = 0x0001;
pub const INDENT_ELSE: c_int = 0x0002;
pub const INDENT_ELIF: c_int = 0x0004;
pub const INDENT_IFDEF: c_int = 0x0008;
pub const INDENT_IFNDEF: c_int = 0x0010;

// macro definitions
#[repr(C)]
pub struct define_s {
    pub name: *mut c_char,                      // define name
    pub flags: c_int,                           // define flags
    pub builtin: c_int,                         // > 0 if builtin define
    pub numparms: c_int,                        // number of define parameters
    pub parms: *mut token_t,                    // define parameters
    pub tokens: *mut token_t,                   // macro tokens (possibly containing parm tokens)
    pub next: *mut define_s,                    // next defined macro in a list
    pub hashnext: *mut define_s,                // next define in the hash chain
    pub globalnext: *mut define_s,              // used to link up the globald defines
}
pub type define_t = define_s;

// indents
// used for conditional compilation directives:
// #if, #else, #elif, #ifdef, #ifndef
#[repr(C)]
pub struct indent_s {
    pub r#type: c_int,                          // indent type
    pub skip: c_int,                            // true if skipping current indent
    pub script: *mut script_t,                  // script the indent was in
    pub next: *mut indent_s,                    // next indent on the indent stack
}
pub type indent_t = indent_s;

// source file
#[repr(C)]
pub struct source_s {
    pub filename: [c_char; 1024],               // file name of the script
    pub includepath: [c_char; 1024],            // path to include files
    pub punctuations: *mut punctuation_t,       // punctuations to use
    pub scriptstack: *mut script_t,             // stack with scripts of the source
    pub tokens: *mut token_t,                   // tokens to read first
    pub defines: *mut define_t,                 // list with macro definitions
    pub definehash: *mut *mut define_t,         // hash chain with defines
    pub indentstack: *mut indent_t,             // stack with indents
    pub skip: c_int,                            // > 0 if skipping conditional code
    pub token: token_t,                         // last read token
}
pub type source_t = source_s;

extern "C" {
    // read a token from the source
    pub fn PC_ReadToken(source: *mut source_t, token: *mut token_t) -> c_int;
    // expect a certain token
    pub fn PC_ExpectTokenString(source: *mut source_t, string: *mut c_char) -> c_int;
    // expect a certain token type
    pub fn PC_ExpectTokenType(
        source: *mut source_t,
        r#type: c_int,
        subtype: c_int,
        token: *mut token_t,
    ) -> c_int;
    // expect a token
    pub fn PC_ExpectAnyToken(source: *mut source_t, token: *mut token_t) -> c_int;
    // returns true when the token is available
    pub fn PC_CheckTokenString(source: *mut source_t, string: *mut c_char) -> c_int;
    // returns true an reads the token when a token with the given type is available
    pub fn PC_CheckTokenType(
        source: *mut source_t,
        r#type: c_int,
        subtype: c_int,
        token: *mut token_t,
    ) -> c_int;
    // skip tokens until the given token string is read
    pub fn PC_SkipUntilString(source: *mut source_t, string: *mut c_char) -> c_int;
    // unread the last token read from the script
    pub fn PC_UnreadLastToken(source: *mut source_t);
    // unread the given token
    pub fn PC_UnreadToken(source: *mut source_t, token: *mut token_t);
    // read a token only if on the same line, lines are concatenated with a slash
    pub fn PC_ReadLine(source: *mut source_t, token: *mut token_t) -> c_int;
    // returns true if there was a white space in front of the token
    pub fn PC_WhiteSpaceBeforeToken(token: *mut token_t) -> c_int;
    // add a define to the source
    pub fn PC_AddDefine(source: *mut source_t, string: *mut c_char) -> c_int;
    // add a globals define that will be added to all opened sources
    pub fn PC_AddGlobalDefine(string: *mut c_char) -> c_int;
    // remove the given global define
    pub fn PC_RemoveGlobalDefine(name: *mut c_char) -> c_int;
    // remove all globals defines
    pub fn PC_RemoveAllGlobalDefines();
    // add builtin defines
    pub fn PC_AddBuiltinDefines(source: *mut source_t);
    // set the source include path
    pub fn PC_SetIncludePath(source: *mut source_t, path: *mut c_char);
    // set the punction set
    pub fn PC_SetPunctuations(source: *mut source_t, p: *mut punctuation_t);
    // set the base folder to load files from
    pub fn PC_SetBaseFolder(path: *mut c_char);
    // load a source file
    pub fn LoadSourceFile(filename: *const c_char) -> *mut source_t;
    // load a source from memory
    pub fn LoadSourceMemory(ptr: *mut c_char, length: c_int, name: *mut c_char) -> *mut source_t;
    // free the given source
    pub fn FreeSource(source: *mut source_t);
    // print a source error
    pub fn SourceError(source: *mut source_t, str: *mut c_char, ...);
    // print a source warning
    pub fn SourceWarning(source: *mut source_t, str: *mut c_char, ...);
}

#[cfg(feature = "bspc")]
pub const MAX_TOKENLENGTH: c_int = 1024;

// some of BSPC source does include game/q_shared.h and some does not
// we define pc_token_s pc_token_t if needed (yes, it's ugly)
#[cfg(feature = "bspc")]
#[repr(C)]
pub struct pc_token_s {
    pub r#type: c_int,
    pub subtype: c_int,
    pub intvalue: c_int,
    pub floatvalue: f32,
    pub string: [c_char; 1024],
}

#[cfg(feature = "bspc")]
pub type pc_token_t = pc_token_s;

extern "C" {
    //
    pub fn PC_LoadSourceHandle(filename: *const c_char) -> c_int;
    pub fn PC_FreeSourceHandle(handle: c_int) -> c_int;
    pub fn PC_ReadTokenHandle(handle: c_int, pc_token: *mut pc_token_t) -> c_int;
    pub fn PC_SourceFileAndLine(handle: c_int, filename: *mut c_char, line: *mut c_int) -> c_int;
    pub fn PC_CheckOpenSourceHandles();
    pub fn PC_LoadGlobalDefines(filename: *const c_char) -> c_int;
    pub fn PC_RemoveAllGlobalDefines();
}
