
/*****************************************************************************
 * name:		l_script.c
 *
 * desc:		lexicographical parser
 *
 * $Archive: /MissionPack/code/botlib/l_script.c $
 * $Author: Ttimo $
 * $Revision: 9 $
 * $Modtime: 4/13/01 4:45p $
 * $Date: 4/13/01 4:45p $
 *
 *****************************************************************************/

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_void, c_ulong};
use core::ptr;

// External declarations for libc and engine functions
extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    pub fn GetMemory(size: usize) -> *mut c_void;
    pub fn GetClearedMemory(size: usize) -> *mut c_void;
    pub fn FreeMemory(ptr: *mut c_void);
    pub fn Com_Memset(ptr: *mut c_void, value: c_int, size: usize) -> *mut c_void;
    pub fn Com_Memcpy(dst: *mut c_void, src: *const c_void, count: usize) -> *mut c_void;
    pub fn COM_Compress(data_p: *mut c_char) -> c_int;
}

// Script flags
const SCFL_NOERRORS: c_int = 0x0001;
const SCFL_NOWARNINGS: c_int = 0x0002;
const SCFL_NOSTRINGWHITESPACES: c_int = 0x0004;
const SCFL_NOSTRINGESCAPECHARS: c_int = 0x0008;
const SCFL_PRIMITIVE: c_int = 0x0010;
const SCFL_NOBINARYNUMBERS: c_int = 0x0020;
const SCFL_NONUMBERVALUES: c_int = 0x0040;

// Token types
const TT_STRING: c_int = 1;			// string
const TT_LITERAL: c_int = 2;			// literal
const TT_NUMBER: c_int = 3;			// number
const TT_NAME: c_int = 4;			// name
const TT_PUNCTUATION: c_int = 5;			// punctuation

// String sub type
// ---------------
// 		the length of the string
// Literal sub type
// ----------------
// 		the ASCII code of the literal
// Number sub type
// ---------------
const TT_DECIMAL: c_int = 0x0008;	// decimal number
const TT_HEX: c_int = 0x0100;	// hexadecimal number
const TT_OCTAL: c_int = 0x0200;	// octal number
const TT_BINARY: c_int = 0x0400;	// binary number
const TT_FLOAT: c_int = 0x0800;	// floating point number
const TT_INTEGER: c_int = 0x1000;	// integer number
const TT_LONG: c_int = 0x2000;	// long number
const TT_UNSIGNED: c_int = 0x4000;	// unsigned number

// Punctuation sub type
// --------------------
const P_RSHIFT_ASSIGN: c_int = 1;
const P_LSHIFT_ASSIGN: c_int = 2;
const P_PARMS: c_int = 3;
const P_PRECOMPMERGE: c_int = 4;

const P_LOGIC_AND: c_int = 5;
const P_LOGIC_OR: c_int = 6;
const P_LOGIC_GEQ: c_int = 7;
const P_LOGIC_LEQ: c_int = 8;
const P_LOGIC_EQ: c_int = 9;
const P_LOGIC_UNEQ: c_int = 10;

const P_MUL_ASSIGN: c_int = 11;
const P_DIV_ASSIGN: c_int = 12;
const P_MOD_ASSIGN: c_int = 13;
const P_ADD_ASSIGN: c_int = 14;
const P_SUB_ASSIGN: c_int = 15;
const P_INC: c_int = 16;
const P_DEC: c_int = 17;

const P_BIN_AND_ASSIGN: c_int = 18;
const P_BIN_OR_ASSIGN: c_int = 19;
const P_BIN_XOR_ASSIGN: c_int = 20;
const P_RSHIFT: c_int = 21;
const P_LSHIFT: c_int = 22;

const P_POINTERREF: c_int = 23;
const P_CPP1: c_int = 24;
const P_CPP2: c_int = 25;
const P_MUL: c_int = 26;
const P_DIV: c_int = 27;
const P_MOD: c_int = 28;
const P_ADD: c_int = 29;
const P_SUB: c_int = 30;
const P_ASSIGN: c_int = 31;

const P_BIN_AND: c_int = 32;
const P_BIN_OR: c_int = 33;
const P_BIN_XOR: c_int = 34;
const P_BIN_NOT: c_int = 35;

const P_LOGIC_NOT: c_int = 36;
const P_LOGIC_GREATER: c_int = 37;
const P_LOGIC_LESS: c_int = 38;

const P_REF: c_int = 39;
const P_COMMA: c_int = 40;
const P_SEMICOLON: c_int = 41;
const P_COLON: c_int = 42;
const P_QUESTIONMARK: c_int = 43;

const P_PARENTHESESOPEN: c_int = 44;
const P_PARENTHESESCLOSE: c_int = 45;
const P_BRACEOPEN: c_int = 46;
const P_BRACECLOSE: c_int = 47;
const P_SQBRACKETOPEN: c_int = 48;
const P_SQBRACKETCLOSE: c_int = 49;
const P_BACKSLASH: c_int = 50;

const P_PRECOMP: c_int = 51;
const P_DOLLAR: c_int = 52;
const P_ATSIGN: c_int = 53;

// Name sub type
// --------
// 		the length of the name

// Maximum token length
const MAX_TOKEN: usize = 1024;
const MAX_QPATH: usize = 64;

// Punctuation
#[repr(C)]
pub struct punctuation_s
{
	pub p: *mut c_char,						//punctuation character(s)
	pub n: c_int,							//punctuation indication
	pub next: *mut punctuation_s,		//next punctuation
}
pub type punctuation_t = punctuation_s;

// Token
#[repr(C)]
pub struct token_s
{
	pub string: [c_char; MAX_TOKEN],			//available token
	pub type_: c_int,						//last read token type
	pub subtype: c_int,					//last read token sub type
	pub intvalue: c_ulong,	//integer value
	pub floatvalue: f64,			//floating point value
	pub whitespace_p: *mut c_char,				//start of white space before token
	pub endwhitespace_p: *mut c_char,			//start of white space before token
	pub line: c_int,						//line the token was on
	pub linescrossed: c_int,				//lines crossed in white space
	pub next: *mut token_s,			//next token in chain
}
pub type token_t = token_s;

// Script file
#[repr(C)]
pub struct script_s
{
	pub filename: [c_char; 1024],			//file name of the script
	pub buffer: *mut c_char,					//buffer containing the script
	pub script_p: *mut c_char,					//current pointer in the script
	pub end_p: *mut c_char,					//pointer to the end of the script
	pub lastscript_p: *mut c_char,				//script pointer before reading token
	pub whitespace_p: *mut c_char,				//begin of the white space
	pub endwhitespace_p: *mut c_char,			//end of the white space
	pub length: c_int,						//length of the script in bytes
	pub line: c_int,						//current line in script
	pub lastline: c_int,					//line before reading token
	pub tokenavailable: c_int,				//set by UnreadLastToken
	pub flags: c_int,						//several script flags
	pub punctuations: *mut punctuation_t,	//the punctuations used in the script
	pub punctuationtable: *mut *mut punctuation_t,
	pub token: token_t,					//available token
	pub next: *mut script_s,			//next script in a chain
}
pub type script_t = script_s;


//longer punctuations first
pub static default_punctuations: [punctuation_t; 40] =
[
	//binary operators
	punctuation_t { p: b">>=\x00".as_ptr() as *mut c_char, n: P_RSHIFT_ASSIGN, next: ptr::null_mut() },
	punctuation_t { p: b"<<=\x00".as_ptr() as *mut c_char, n: P_LSHIFT_ASSIGN, next: ptr::null_mut() },
	//
	punctuation_t { p: b"...\x00".as_ptr() as *mut c_char, n: P_PARMS, next: ptr::null_mut() },
	//define merge operator
	punctuation_t { p: b"##\x00".as_ptr() as *mut c_char, n: P_PRECOMPMERGE, next: ptr::null_mut() },
	//logic operators
	punctuation_t { p: b"&&\x00".as_ptr() as *mut c_char, n: P_LOGIC_AND, next: ptr::null_mut() },
	punctuation_t { p: b"||\x00".as_ptr() as *mut c_char, n: P_LOGIC_OR, next: ptr::null_mut() },
	punctuation_t { p: b">=\x00".as_ptr() as *mut c_char, n: P_LOGIC_GEQ, next: ptr::null_mut() },
	punctuation_t { p: b"<=\x00".as_ptr() as *mut c_char, n: P_LOGIC_LEQ, next: ptr::null_mut() },
	punctuation_t { p: b"==\x00".as_ptr() as *mut c_char, n: P_LOGIC_EQ, next: ptr::null_mut() },
	punctuation_t { p: b"!=\x00".as_ptr() as *mut c_char, n: P_LOGIC_UNEQ, next: ptr::null_mut() },
	//arithmatic operators
	punctuation_t { p: b"*=\x00".as_ptr() as *mut c_char, n: P_MUL_ASSIGN, next: ptr::null_mut() },
	punctuation_t { p: b"/=\x00".as_ptr() as *mut c_char, n: P_DIV_ASSIGN, next: ptr::null_mut() },
	punctuation_t { p: b"%=\x00".as_ptr() as *mut c_char, n: P_MOD_ASSIGN, next: ptr::null_mut() },
	punctuation_t { p: b"+=\x00".as_ptr() as *mut c_char, n: P_ADD_ASSIGN, next: ptr::null_mut() },
	punctuation_t { p: b"-=\x00".as_ptr() as *mut c_char, n: P_SUB_ASSIGN, next: ptr::null_mut() },
	punctuation_t { p: b"++\x00".as_ptr() as *mut c_char, n: P_INC, next: ptr::null_mut() },
	punctuation_t { p: b"--\x00".as_ptr() as *mut c_char, n: P_DEC, next: ptr::null_mut() },
	//binary operators
	punctuation_t { p: b"&=\x00".as_ptr() as *mut c_char, n: P_BIN_AND_ASSIGN, next: ptr::null_mut() },
	punctuation_t { p: b"|=\x00".as_ptr() as *mut c_char, n: P_BIN_OR_ASSIGN, next: ptr::null_mut() },
	punctuation_t { p: b"^=\x00".as_ptr() as *mut c_char, n: P_BIN_XOR_ASSIGN, next: ptr::null_mut() },
	punctuation_t { p: b">>\x00".as_ptr() as *mut c_char, n: P_RSHIFT, next: ptr::null_mut() },
	punctuation_t { p: b"<<\x00".as_ptr() as *mut c_char, n: P_LSHIFT, next: ptr::null_mut() },
	//reference operators
	punctuation_t { p: b"->\x00".as_ptr() as *mut c_char, n: P_POINTERREF, next: ptr::null_mut() },
	//C++
	punctuation_t { p: b"::\x00".as_ptr() as *mut c_char, n: P_CPP1, next: ptr::null_mut() },
	punctuation_t { p: b".*\x00".as_ptr() as *mut c_char, n: P_CPP2, next: ptr::null_mut() },
	//arithmatic operators
	punctuation_t { p: b"*\x00".as_ptr() as *mut c_char, n: P_MUL, next: ptr::null_mut() },
	punctuation_t { p: b"/\x00".as_ptr() as *mut c_char, n: P_DIV, next: ptr::null_mut() },
	punctuation_t { p: b"%\x00".as_ptr() as *mut c_char, n: P_MOD, next: ptr::null_mut() },
	punctuation_t { p: b"+\x00".as_ptr() as *mut c_char, n: P_ADD, next: ptr::null_mut() },
	punctuation_t { p: b"-\x00".as_ptr() as *mut c_char, n: P_SUB, next: ptr::null_mut() },
	punctuation_t { p: b"=\x00".as_ptr() as *mut c_char, n: P_ASSIGN, next: ptr::null_mut() },
	//binary operators
	punctuation_t { p: b"&\x00".as_ptr() as *mut c_char, n: P_BIN_AND, next: ptr::null_mut() },
	punctuation_t { p: b"|\x00".as_ptr() as *mut c_char, n: P_BIN_OR, next: ptr::null_mut() },
	punctuation_t { p: b"^\x00".as_ptr() as *mut c_char, n: P_BIN_XOR, next: ptr::null_mut() },
	punctuation_t { p: b"~\x00".as_ptr() as *mut c_char, n: P_BIN_NOT, next: ptr::null_mut() },
	//logic operators
	punctuation_t { p: b"!\x00".as_ptr() as *mut c_char, n: P_LOGIC_NOT, next: ptr::null_mut() },
	punctuation_t { p: b">\x00".as_ptr() as *mut c_char, n: P_LOGIC_GREATER, next: ptr::null_mut() },
	punctuation_t { p: b"<\x00".as_ptr() as *mut c_char, n: P_LOGIC_LESS, next: ptr::null_mut() },
	//reference operator
	punctuation_t { p: b".\x00".as_ptr() as *mut c_char, n: P_REF, next: ptr::null_mut() },
	//seperators
	punctuation_t { p: b",\x00".as_ptr() as *mut c_char, n: P_COMMA, next: ptr::null_mut() },
	punctuation_t { p: b";\x00".as_ptr() as *mut c_char, n: P_SEMICOLON, next: ptr::null_mut() },
	//label indication
	punctuation_t { p: b":\x00".as_ptr() as *mut c_char, n: P_COLON, next: ptr::null_mut() },
];

pub static mut basefolder: [c_char; MAX_QPATH] = [0; MAX_QPATH];

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn PS_CreatePunctuationTable(script: *mut script_t, punctuations: *mut punctuation_t)
{
	let mut i: c_int = 0;
	let mut p: *mut punctuation_t;
	let mut lastp: *mut punctuation_t;
	let mut newp: *mut punctuation_t;

	//get memory for the table
	if (*script).punctuationtable.is_null() {
		(*script).punctuationtable = GetMemory(256 * core::mem::size_of::<*mut punctuation_t>()) as *mut *mut punctuation_t;
	}
	Com_Memset((*script).punctuationtable as *mut c_void, 0, 256 * core::mem::size_of::<*mut punctuation_t>());
	//add the punctuations in the list to the punctuation table
	loop {
		if (*punctuations.add(i as usize)).p.is_null() { break; }
		newp = &mut *punctuations.add(i as usize);
		lastp = ptr::null_mut();
		//sort the punctuations in this table entry on length (longer punctuations first)
		p = *(*script).punctuationtable.add(*(*newp).p as usize);
		loop {
			if p.is_null() { break; }
			if strlen((*p).p) < strlen((*newp).p)
			{
				(*newp).next = p;
				if !lastp.is_null() {
					(*lastp).next = newp;
				} else {
					*(*script).punctuationtable.add(*(*newp).p as usize) = newp;
				}
				break;
			} //end if
			lastp = p;
			p = (*p).next;
		} //end for
		if p.is_null()
		{
			(*newp).next = ptr::null_mut();
			if !lastp.is_null() {
				(*lastp).next = newp;
			} else {
				*(*script).punctuationtable.add(*(*newp).p as usize) = newp;
			}
		} //end if
		i += 1;
	} //end for
} //end of the function PS_CreatePunctuationTable

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn PunctuationFromNum(script: *mut script_t, num: c_int) -> *mut c_char
{
	let mut i: c_int = 0;

	loop {
		if (*(*script).punctuations.add(i as usize)).p.is_null() { break; }
		if (*(*script).punctuations.add(i as usize)).n == num {
			return (*(*script).punctuations.add(i as usize)).p;
		}
		i += 1;
	} //end for
	return b"unkown punctuation\x00".as_ptr() as *mut c_char;
} //end of the function PunctuationFromNum

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
// Note: ScriptError is a variadic function in C. In Rust, this is difficult to
// implement for variadic args. The structure is preserved but variadic args
// are not fully supported. This should be implemented in C or via FFI wrapper.
pub unsafe fn ScriptError(script: *mut script_t, str_: *mut c_char)
{
	if ((*script).flags & SCFL_NOERRORS) != 0 { return; }
	// Variadic function handling would require C variadic argument forwarding
	// This is a structural placeholder for the faithful port
} //end of the function ScriptError

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
// Note: ScriptWarning is a variadic function in C. In Rust, this is difficult to
// implement for variadic args. The structure is preserved but variadic args
// are not fully supported. This should be implemented in C or via FFI wrapper.
pub unsafe fn ScriptWarning(script: *mut script_t, str_: *mut c_char)
{
	if ((*script).flags & SCFL_NOWARNINGS) != 0 { return; }
	// Variadic function handling would require C variadic argument forwarding
	// This is a structural placeholder for the faithful port
} //end of the function ScriptWarning

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn SetScriptPunctuations(script: *mut script_t, p: *mut punctuation_t)
{
	if !p.is_null() {
		PS_CreatePunctuationTable(script, p);
	} else {
		PS_CreatePunctuationTable(script, default_punctuations.as_ptr() as *mut punctuation_t);
	}
	if !p.is_null() {
		(*script).punctuations = p;
	} else {
		(*script).punctuations = default_punctuations.as_ptr() as *mut punctuation_t;
	}
} //end of the function SetScriptPunctuations

//============================================================================
// Reads spaces, tabs, C-like comments etc.
// When a newline character is found the scripts line counter is increased.
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ReadWhiteSpace(script: *mut script_t) -> c_int
{
	loop {
		//skip white space
		loop {
			if *(*script).script_p <= ' ' as c_char {
				if *(*script).script_p == 0 as c_char { return 0; }
				if *(*script).script_p == '\n' as c_char { (*script).line += 1; }
				(*script).script_p = (*script).script_p.add(1);
			} else {
				break;
			}
		} //end while
		//skip comments
		if *(*script).script_p == '/' as c_char
		{
			//comments //
			if *(*script).script_p.add(1) == '/' as c_char
			{
				(*script).script_p = (*script).script_p.add(1);
				loop {
					(*script).script_p = (*script).script_p.add(1);
					if *(*script).script_p == 0 as c_char { return 0; }
					if *(*script).script_p == '\n' as c_char { break; }
				} //end do
				(*script).line += 1;
				(*script).script_p = (*script).script_p.add(1);
				if *(*script).script_p == 0 as c_char { return 0; }
				continue;
			} //end if
			//comments /* */
			else if *(*script).script_p.add(1) == '*' as c_char
			{
				(*script).script_p = (*script).script_p.add(1);
				loop {
					(*script).script_p = (*script).script_p.add(1);
					if *(*script).script_p == 0 as c_char { return 0; }
					if *(*script).script_p == '\n' as c_char { (*script).line += 1; }
					if *(*script).script_p == '*' as c_char && *(*script).script_p.add(1) == '/' as c_char { break; }
				} //end do
				(*script).script_p = (*script).script_p.add(1);
				if *(*script).script_p == 0 as c_char { return 0; }
				(*script).script_p = (*script).script_p.add(1);
				if *(*script).script_p == 0 as c_char { return 0; }
				continue;
			} //end if
		} //end if
		break;
	} //end while
	return 1;
} //end of the function PS_ReadWhiteSpace

//============================================================================
// Reads an escape character.
//
// Parameter:				script		: script to read from
//								ch				: place to store the read escape character
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ReadEscapeCharacter(script: *mut script_t, ch: *mut c_char) -> c_int
{
	let mut c: c_int;
	let mut val: c_int;
	let mut i: c_int;

	//step over the leading '\\'
	(*script).script_p = (*script).script_p.add(1);
	//determine the escape character
	match *(*script).script_p as c_int {
		92 => { c = 92; }, // '\\'
		110 => { c = 10; }, // 'n' -> '\n'
		114 => { c = 13; }, // 'r' -> '\r'
		116 => { c = 9; }, // 't' -> '\t'
		118 => { c = 11; }, // 'v' -> '\v'
		98 => { c = 8; }, // 'b' -> '\b'
		102 => { c = 12; }, // 'f' -> '\f'
		97 => { c = 7; }, // 'a' -> '\a'
		39 => { c = 39; }, // '\''
		34 => { c = 34; }, // '"'
		63 => { c = 63; }, // '\?'
		120 => { // 'x'
			(*script).script_p = (*script).script_p.add(1);
			i = 0;
			val = 0;
			loop {
				c = *(*script).script_p as c_int;
				if c >= '0' as c_int && c <= '9' as c_int {
					c = c - '0' as c_int;
				} else if c >= 'A' as c_int && c <= 'Z' as c_int {
					c = c - 'A' as c_int + 10;
				} else if c >= 'a' as c_int && c <= 'z' as c_int {
					c = c - 'a' as c_int + 10;
				} else {
					break;
				}
				val = (val << 4) + c;
				i += 1;
				(*script).script_p = (*script).script_p.add(1);
			} //end for
			(*script).script_p = (*script).script_p.sub(1);
			if val > 0xFF {
				ScriptWarning(script, b"too large value in escape character\x00".as_ptr() as *mut c_char);
				val = 0xFF;
			} //end if
			c = val;
		}, // end case 'x'
		_ => { //NOTE: decimal ASCII code, NOT octal
			if *(*script).script_p < '0' as c_char || *(*script).script_p > '9' as c_char {
				ScriptError(script, b"unknown escape char\x00".as_ptr() as *mut c_char);
			}
			i = 0;
			val = 0;
			loop {
				c = *(*script).script_p as c_int;
				if c >= '0' as c_int && c <= '9' as c_int {
					c = c - '0' as c_int;
				} else {
					break;
				}
				val = val * 10 + c;
				i += 1;
				(*script).script_p = (*script).script_p.add(1);
			} //end for
			(*script).script_p = (*script).script_p.sub(1);
			if val > 0xFF {
				ScriptWarning(script, b"too large value in escape character\x00".as_ptr() as *mut c_char);
				val = 0xFF;
			} //end if
			c = val;
		}, //end default
	} //end switch
	//step over the escape character or the last digit of the number
	(*script).script_p = (*script).script_p.add(1);
	//store the escape character
	*ch = c as c_char;
	//succesfully read escape character
	return 1;
} //end of the function PS_ReadEscapeCharacter

//============================================================================
// Reads C-like string. Escape characters are interpretted.
// Quotes are included with the string.
// Reads two strings with a white space between them as one string.
//
// Parameter:				script		: script to read from
//								token			: buffer to store the string
// Returns:					qtrue when a string was read succesfully
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ReadString(script: *mut script_t, token: *mut token_t, quote: c_char) -> c_int
{
	let mut len: c_int = 0;
	let mut tmpline: c_int;
	let mut tmpscript_p: *mut c_char;

	if quote as c_int == '"' as c_int {
		(*token).type_ = TT_STRING;
	} else {
		(*token).type_ = TT_LITERAL;
	}

	len = 0;
	//leading quote
	(*token).string[len as usize] = *(*script).script_p;
	len += 1;
	(*script).script_p = (*script).script_p.add(1);
	//
	loop {
		//minus 2 because trailing double quote and zero have to be appended
		if len >= MAX_TOKEN as c_int - 2 {
			ScriptError(script, b"string longer than MAX_TOKEN = %d\x00".as_ptr() as *mut c_char, MAX_TOKEN as c_int);
			return 0;
		} //end if
		//if there is an escape character and
		//if escape characters inside a string are allowed
		if *(*script).script_p == '\\' as c_char && ((*script).flags & SCFL_NOSTRINGESCAPECHARS) == 0 {
			if PS_ReadEscapeCharacter(script, &mut (*token).string[len as usize]) == 0 {
				(*token).string[len as usize] = 0 as c_char;
				return 0;
			} //end if
			len += 1;
		} //end if
		//if a trailing quote
		else if *(*script).script_p == quote {
			//step over the double quote
			(*script).script_p = (*script).script_p.add(1);
			//if white spaces in a string are not allowed
			if ((*script).flags & SCFL_NOSTRINGWHITESPACES) != 0 { break; }
			//
			tmpscript_p = (*script).script_p;
			tmpline = (*script).line;
			//read unusefull stuff between possible two following strings
			if PS_ReadWhiteSpace(script) == 0 {
				(*script).script_p = tmpscript_p;
				(*script).line = tmpline;
				break;
			} //end if
			//if there's no leading double qoute
			if *(*script).script_p != quote {
				(*script).script_p = tmpscript_p;
				(*script).line = tmpline;
				break;
			} //end if
			//step over the new leading double quote
			(*script).script_p = (*script).script_p.add(1);
		} //end if
		else {
			if *(*script).script_p == 0 as c_char {
				(*token).string[len as usize] = 0 as c_char;
				ScriptError(script, b"missing trailing quote\x00".as_ptr() as *mut c_char);
				return 0;
			} //end if
			if *(*script).script_p == '\n' as c_char {
				(*token).string[len as usize] = 0 as c_char;
				ScriptError(script, b"newline inside string %s\x00".as_ptr() as *mut c_char, (*token).string.as_ptr());
				return 0;
			} //end if
			(*token).string[len as usize] = *(*script).script_p;
			len += 1;
			(*script).script_p = (*script).script_p.add(1);
		} //end else
	} //end while
	//trailing quote
	(*token).string[len as usize] = quote;
	len += 1;
	//end string with a zero
	(*token).string[len as usize] = 0 as c_char;
	//the sub type is the length of the string
	(*token).subtype = len;
	return 1;
} //end of the function PS_ReadString

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ReadName(script: *mut script_t, token: *mut token_t) -> c_int
{
	let mut len: c_int = 0;
	let mut c: c_char;

	(*token).type_ = TT_NAME;
	loop {
		(*token).string[len as usize] = *(*script).script_p;
		len += 1;
		(*script).script_p = (*script).script_p.add(1);
		if len >= MAX_TOKEN as c_int {
			ScriptError(script, b"name longer than MAX_TOKEN = %d\x00".as_ptr() as *mut c_char, MAX_TOKEN as c_int);
			return 0;
		} //end if
		c = *(*script).script_p;
		if !((c as c_int >= 'a' as c_int && c as c_int <= 'z' as c_int) ||
				(c as c_int >= 'A' as c_int && c as c_int <= 'Z' as c_int) ||
				(c as c_int >= '0' as c_int && c as c_int <= '9' as c_int) ||
				c as c_int == '_' as c_int) {
			break;
		}
	}
	(*token).string[len as usize] = 0 as c_char;
	//the sub type is the length of the name
	(*token).subtype = len;
	return 1;
} //end of the function PS_ReadName

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn NumberValue(string: *mut c_char, subtype: c_int, intvalue: *mut c_ulong, floatvalue: *mut f64)
{
	let mut dotfound: c_ulong = 0;

	*intvalue = 0;
	*floatvalue = 0.0;
	//floating point number
	if (subtype & TT_FLOAT) != 0 {
		loop {
			if *string == 0 as c_char { break; }
			if *string == '.' as c_char {
				if dotfound != 0 { return; }
				dotfound = 10;
				string = string.add(1);
			} //end if
			if dotfound != 0 {
				*floatvalue = *floatvalue + ((*string as c_int - '0' as c_int) as f64) / (dotfound as f64);
				dotfound *= 10;
			} //end if
			else {
				*floatvalue = *floatvalue * 10.0 + ((*string as c_int - '0' as c_int) as f64);
			} //end else
			string = string.add(1);
		} //end while
		*intvalue = *floatvalue as c_ulong;
	} //end if
	else if (subtype & TT_DECIMAL) != 0 {
		loop {
			if *string == 0 as c_char { break; }
			*intvalue = *intvalue * 10 + ((*string as c_int - '0' as c_int) as c_ulong);
			string = string.add(1);
		}
		*floatvalue = *intvalue as f64;
	} //end else if
	else if (subtype & TT_HEX) != 0 {
		//step over the leading 0x or 0X
		string = string.add(2);
		loop {
			if *string == 0 as c_char { break; }
			*intvalue <<= 4;
			if *string as c_int >= 'a' as c_int && *string as c_int <= 'f' as c_int {
				*intvalue += (*string as c_int - 'a' as c_int + 10) as c_ulong;
			} else if *string as c_int >= 'A' as c_int && *string as c_int <= 'F' as c_int {
				*intvalue += (*string as c_int - 'A' as c_int + 10) as c_ulong;
			} else {
				*intvalue += (*string as c_int - '0' as c_int) as c_ulong;
			}
			string = string.add(1);
		} //end while
		*floatvalue = *intvalue as f64;
	} //end else if
	else if (subtype & TT_OCTAL) != 0 {
		//step over the first zero
		string = string.add(1);
		loop {
			if *string == 0 as c_char { break; }
			*intvalue = (*intvalue << 3) + ((*string as c_int - '0' as c_int) as c_ulong);
			string = string.add(1);
		}
		*floatvalue = *intvalue as f64;
	} //end else if
	else if (subtype & TT_BINARY) != 0 {
		//step over the leading 0b or 0B
		string = string.add(2);
		loop {
			if *string == 0 as c_char { break; }
			*intvalue = (*intvalue << 1) + ((*string as c_int - '0' as c_int) as c_ulong);
			string = string.add(1);
		}
		*floatvalue = *intvalue as f64;
	} //end else if
} //end of the function NumberValue

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ReadNumber(script: *mut script_t, token: *mut token_t) -> c_int
{
	let mut len: c_int = 0;
	let mut i: c_int;
	let mut octal: c_int;
	let mut dot: c_int;
	let mut c: c_char;
	//	unsigned long int intvalue = 0;
	//	long double floatvalue = 0;

	(*token).type_ = TT_NUMBER;
	//check for a hexadecimal number
	if *(*script).script_p == '0' as c_char &&
		(*(*script).script_p.add(1) == 'x' as c_char ||
		*(*script).script_p.add(1) == 'X' as c_char) {
		(*token).string[len as usize] = *(*script).script_p;
		len += 1;
		(*script).script_p = (*script).script_p.add(1);
		(*token).string[len as usize] = *(*script).script_p;
		len += 1;
		(*script).script_p = (*script).script_p.add(1);
		c = *(*script).script_p;
		//hexadecimal
		loop {
			if !((c as c_int >= '0' as c_int && c as c_int <= '9' as c_int) ||
					(c as c_int >= 'a' as c_int && c as c_int <= 'f' as c_int) ||
					(c as c_int >= 'A' as c_int && c as c_int <= 'A' as c_int)) {
				break;
			}
			(*token).string[len as usize] = *(*script).script_p;
			len += 1;
			(*script).script_p = (*script).script_p.add(1);
			if len >= MAX_TOKEN as c_int {
				ScriptError(script, b"hexadecimal number longer than MAX_TOKEN = %d\x00".as_ptr() as *mut c_char, MAX_TOKEN as c_int);
				return 0;
			} //end if
			c = *(*script).script_p;
		} //end while
		(*token).subtype |= TT_HEX;
	} //end if

	//check for a binary number
	else if *(*script).script_p == '0' as c_char &&
		(*(*script).script_p.add(1) == 'b' as c_char ||
		*(*script).script_p.add(1) == 'B' as c_char) {
		(*token).string[len as usize] = *(*script).script_p;
		len += 1;
		(*script).script_p = (*script).script_p.add(1);
		(*token).string[len as usize] = *(*script).script_p;
		len += 1;
		(*script).script_p = (*script).script_p.add(1);
		c = *(*script).script_p;
		//binary
		loop {
			if !(c as c_int == '0' as c_int || c as c_int == '1' as c_int) {
				break;
			}
			(*token).string[len as usize] = *(*script).script_p;
			len += 1;
			(*script).script_p = (*script).script_p.add(1);
			if len >= MAX_TOKEN as c_int {
				ScriptError(script, b"binary number longer than MAX_TOKEN = %d\x00".as_ptr() as *mut c_char, MAX_TOKEN as c_int);
				return 0;
			} //end if
			c = *(*script).script_p;
		} //end while
		(*token).subtype |= TT_BINARY;
	} //end if

	else { //decimal or octal integer or floating point number
		octal = 0;
		dot = 0;
		if *(*script).script_p == '0' as c_char { octal = 1; }
		loop {
			c = *(*script).script_p;
			if c as c_int == '.' as c_int { dot = 1; }
			else if c as c_int == '8' as c_int || c as c_int == '9' as c_int { octal = 0; }
			else if c as c_int < '0' as c_int || c as c_int > '9' as c_int { break; }
			(*token).string[len as usize] = *(*script).script_p;
			len += 1;
			(*script).script_p = (*script).script_p.add(1);
			if len >= MAX_TOKEN as c_int - 1 {
				ScriptError(script, b"number longer than MAX_TOKEN = %d\x00".as_ptr() as *mut c_char, MAX_TOKEN as c_int);
				return 0;
			} //end if
		} //end while
		if octal != 0 { (*token).subtype |= TT_OCTAL; }
		else { (*token).subtype |= TT_DECIMAL; }
		if dot != 0 { (*token).subtype |= TT_FLOAT; }
	} //end else
	i = 0;
	loop {
		if !(i < 2) { break; }
		c = *(*script).script_p;
		//check for a LONG number
		if (c as c_int == 'l' as c_int || c as c_int == 'L' as c_int) // bk001204 - brackets
		    && ((*token).subtype & TT_LONG) == 0 {
			(*script).script_p = (*script).script_p.add(1);
			(*token).subtype |= TT_LONG;
		} //end if
		//check for an UNSIGNED number
		else if (c as c_int == 'u' as c_int || c as c_int == 'U' as c_int) // bk001204 - brackets
			  && ((*token).subtype & (TT_UNSIGNED | TT_FLOAT)) == 0 {
			(*script).script_p = (*script).script_p.add(1);
			(*token).subtype |= TT_UNSIGNED;
		} //end if
		i += 1;
	} //end for
	(*token).string[len as usize] = 0 as c_char;

	NumberValue((*token).string.as_mut_ptr(), (*token).subtype, &mut (*token).intvalue, &mut (*token).floatvalue);

	if ((*token).subtype & TT_FLOAT) == 0 { (*token).subtype |= TT_INTEGER; }
	return 1;
} //end of the function PS_ReadNumber

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ReadLiteral(script: *mut script_t, token: *mut token_t) -> c_int
{
	(*token).type_ = TT_LITERAL;
	//first quote
	(*token).string[0] = *(*script).script_p;
	(*script).script_p = (*script).script_p.add(1);
	//check for end of file
	if *(*script).script_p == 0 as c_char {
		ScriptError(script, b"end of file before trailing \'\x00".as_ptr() as *mut c_char);
		return 0;
	} //end if
	//if it is an escape character
	if *(*script).script_p == '\\' as c_char {
		if PS_ReadEscapeCharacter(script, &mut (*token).string[1]) == 0 { return 0; }
	} //end if
	else {
		(*token).string[1] = *(*script).script_p;
		(*script).script_p = (*script).script_p.add(1);
	} //end else
	//check for trailing quote
	if *(*script).script_p != '\'' as c_char {
		ScriptWarning(script, b"too many characters in literal, ignored\x00".as_ptr() as *mut c_char);
		loop {
			if !(*(*script).script_p != 0 as c_char &&
					*(*script).script_p != '\'' as c_char &&
					*(*script).script_p != '\n' as c_char) {
				break;
			}
			(*script).script_p = (*script).script_p.add(1);
		} //end while
		if *(*script).script_p == '\'' as c_char { (*script).script_p = (*script).script_p.add(1); }
	} //end if
	//store the trailing quote
	(*token).string[2] = *(*script).script_p;
	(*script).script_p = (*script).script_p.add(1);
	//store trailing zero to end the string
	(*token).string[3] = 0 as c_char;
	//the sub type is the integer literal value
	(*token).subtype = (*token).string[1] as c_int;
	//
	return 1;
} //end of the function PS_ReadLiteral

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ReadPunctuation(script: *mut script_t, token: *mut token_t) -> c_int
{
	let mut len: usize;
	let mut p: *mut c_char;
	let mut punc: *mut punctuation_t;

	punc = *(*script).punctuationtable.add(*(*script).script_p as usize);
	loop {
		if punc.is_null() { break; }
		p = (*punc).p;
		len = strlen(p);
		//if the script contains at least as much characters as the punctuation
		if (*script).script_p.add(len) <= (*script).end_p {
			//if the script contains the punctuation
			if strncmp((*script).script_p, p, len) == 0 {
				strncpy((*token).string.as_mut_ptr(), p, MAX_TOKEN);
				(*script).script_p = (*script).script_p.add(len);
				(*token).type_ = TT_PUNCTUATION;
				//sub type is the number of the punctuation
				(*token).subtype = (*punc).n;
				return 1;
			} //end if
		} //end if
		punc = (*punc).next;
	} //end for
	return 0;
} //end of the function PS_ReadPunctuation

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ReadPrimitive(script: *mut script_t, token: *mut token_t) -> c_int
{
	let mut len: c_int = 0;

	len = 0;
	loop {
		if !(*(*script).script_p > ' ' as c_char && *(*script).script_p != ';' as c_char) {
			break;
		}
		if len >= MAX_TOKEN as c_int {
			ScriptError(script, b"primitive token longer than MAX_TOKEN = %d\x00".as_ptr() as *mut c_char, MAX_TOKEN as c_int);
			return 0;
		} //end if
		(*token).string[len as usize] = *(*script).script_p;
		len += 1;
		(*script).script_p = (*script).script_p.add(1);
	} //end while
	(*token).string[len as usize] = 0 as c_char;
	//copy the token into the script structure
	Com_Memcpy(&mut (*script).token as *mut token_t as *mut c_void, token as *const c_void, core::mem::size_of::<token_t>());
	//primitive reading successfull
	return 1;
} //end of the function PS_ReadPrimitive

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ReadToken(script: *mut script_t, token: *mut token_t) -> c_int
{
	//if there is a token available (from UnreadToken)
	if (*script).tokenavailable != 0 {
		(*script).tokenavailable = 0;
		Com_Memcpy(token as *mut c_void, &(*script).token as *const token_t as *const c_void, core::mem::size_of::<token_t>());
		return 1;
	} //end if
	//save script pointer
	(*script).lastscript_p = (*script).script_p;
	//save line counter
	(*script).lastline = (*script).line;
	//clear the token stuff
	Com_Memset(token as *mut c_void, 0, core::mem::size_of::<token_t>());
	//start of the white space
	(*script).whitespace_p = (*script).script_p;
	(*token).whitespace_p = (*script).script_p;
	//read unusefull stuff
	if PS_ReadWhiteSpace(script) == 0 { return 0; }
	//end of the white space
	(*script).endwhitespace_p = (*script).script_p;
	(*token).endwhitespace_p = (*script).script_p;
	//line the token is on
	(*token).line = (*script).line;
	//number of lines crossed before token
	(*token).linescrossed = (*script).line - (*script).lastline;
	//if there is a leading double quote
	if *(*script).script_p == '\"' as c_char {
		if PS_ReadString(script, token, '\"' as c_char) == 0 { return 0; }
	} //end if
	//if an literal
	else if *(*script).script_p == '\'' as c_char {
		//if (!PS_ReadLiteral(script, token)) return 0;
		if PS_ReadString(script, token, '\'' as c_char) == 0 { return 0; }
	} //end if
	//if there is a number
	else if (*(*script).script_p >= '0' as c_char && *(*script).script_p <= '9' as c_char) ||
				(*(*script).script_p == '.' as c_char &&
				(*(*script).script_p.add(1) >= '0' as c_char && *(*script).script_p.add(1) <= '9' as c_char)) {
		if PS_ReadNumber(script, token) == 0 { return 0; }
	} //end if
	//if this is a primitive script
	else if ((*script).flags & SCFL_PRIMITIVE) != 0 {
		return PS_ReadPrimitive(script, token);
	} //end else if
	//if there is a name
	else if (*(*script).script_p >= 'a' as c_char && *(*script).script_p <= 'z' as c_char) ||
		(*(*script).script_p >= 'A' as c_char && *(*script).script_p <= 'Z' as c_char) ||
		*(*script).script_p == '_' as c_char || *(*script).script_p == '@' as c_char {
		if PS_ReadName(script, token) == 0 { return 0; }
	} //end if
	//check for punctuations
	else if PS_ReadPunctuation(script, token) == 0 {
		ScriptError(script, b"can\'t read token\x00".as_ptr() as *mut c_char);
		return 0;
	} //end if
	//copy the token into the script structure
	Com_Memcpy(&mut (*script).token as *mut token_t as *mut c_void, token as *const c_void, core::mem::size_of::<token_t>());
	//succesfully read a token
	return 1;
} //end of the function PS_ReadToken

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ExpectTokenString(script: *mut script_t, string: *mut c_char) -> c_int
{
	let mut token: token_t = core::mem::zeroed();

	if PS_ReadToken(script, &mut token) == 0 {
		ScriptError(script, b"couldn\'t find expected %s\x00".as_ptr() as *mut c_char, string);
		return 0;
	} //end if

	if strcmp(token.string.as_ptr(), string) != 0 {
		ScriptError(script, b"expected %s, found %s\x00".as_ptr() as *mut c_char, string, token.string.as_ptr());
		return 0;
	} //end if
	return 1;
} //end of the function PS_ExpectToken

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ExpectTokenType(script: *mut script_t, type_: c_int, subtype: c_int, token: *mut token_t) -> c_int
{
	let mut str: [c_char; MAX_TOKEN] = [0; MAX_TOKEN];

	if PS_ReadToken(script, token) == 0 {
		ScriptError(script, b"couldn\'t read expected token\x00".as_ptr() as *mut c_char);
		return 0;
	} //end if

	if (*token).type_ != type_ {
		if type_ == TT_STRING { strcpy(str.as_mut_ptr(), b"string\x00".as_ptr() as *const c_char); }
		if type_ == TT_LITERAL { strcpy(str.as_mut_ptr(), b"literal\x00".as_ptr() as *const c_char); }
		if type_ == TT_NUMBER { strcpy(str.as_mut_ptr(), b"number\x00".as_ptr() as *const c_char); }
		if type_ == TT_NAME { strcpy(str.as_mut_ptr(), b"name\x00".as_ptr() as *const c_char); }
		if type_ == TT_PUNCTUATION { strcpy(str.as_mut_ptr(), b"punctuation\x00".as_ptr() as *const c_char); }
		ScriptError(script, b"expected a %s, found %s\x00".as_ptr() as *mut c_char, str.as_ptr(), (*token).string.as_ptr());
		return 0;
	} //end if
	if (*token).type_ == TT_NUMBER {
		if ((*token).subtype & subtype) != subtype {
			if (subtype & TT_DECIMAL) != 0 { strcpy(str.as_mut_ptr(), b"decimal\x00".as_ptr() as *const c_char); }
			if (subtype & TT_HEX) != 0 { strcpy(str.as_mut_ptr(), b"hex\x00".as_ptr() as *const c_char); }
			if (subtype & TT_OCTAL) != 0 { strcpy(str.as_mut_ptr(), b"octal\x00".as_ptr() as *const c_char); }
			if (subtype & TT_BINARY) != 0 { strcpy(str.as_mut_ptr(), b"binary\x00".as_ptr() as *const c_char); }
			if (subtype & TT_LONG) != 0 { strcat(str.as_mut_ptr(), b" long\x00".as_ptr() as *const c_char); }
			if (subtype & TT_UNSIGNED) != 0 { strcat(str.as_mut_ptr(), b" unsigned\x00".as_ptr() as *const c_char); }
			if (subtype & TT_FLOAT) != 0 { strcat(str.as_mut_ptr(), b" float\x00".as_ptr() as *const c_char); }
			if (subtype & TT_INTEGER) != 0 { strcat(str.as_mut_ptr(), b" integer\x00".as_ptr() as *const c_char); }
			ScriptError(script, b"expected %s, found %s\x00".as_ptr() as *mut c_char, str.as_ptr(), (*token).string.as_ptr());
			return 0;
		} //end if
	} //end if
	else if (*token).type_ == TT_PUNCTUATION {
		if subtype < 0 {
			ScriptError(script, b"BUG: wrong punctuation subtype\x00".as_ptr() as *mut c_char);
			return 0;
		} //end if
		if (*token).subtype != subtype {
			ScriptError(script, b"expected %s, found %s\x00".as_ptr() as *mut c_char,
							(*script).punctuations.add(subtype as usize).read().p, (*token).string.as_ptr());
			return 0;
		} //end if
	} //end else if
	return 1;
} //end of the function PS_ExpectTokenType

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_ExpectAnyToken(script: *mut script_t, token: *mut token_t) -> c_int
{
	if PS_ReadToken(script, token) == 0 {
		ScriptError(script, b"couldn\'t read expected token\x00".as_ptr() as *mut c_char);
		return 0;
	} //end if
	else {
		return 1;
	} //end else
} //end of the function PS_ExpectAnyToken

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_CheckTokenString(script: *mut script_t, string: *mut c_char) -> c_int
{
	let mut tok: token_t = core::mem::zeroed();

	if PS_ReadToken(script, &mut tok) == 0 { return 0; }
	//if the token is available
	if strcmp(tok.string.as_ptr(), string) == 0 { return 1; }
	//token not available
	(*script).script_p = (*script).lastscript_p;
	return 0;
} //end of the function PS_CheckTokenString

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_CheckTokenType(script: *mut script_t, type_: c_int, subtype: c_int, token: *mut token_t) -> c_int
{
	let mut tok: token_t = core::mem::zeroed();

	if PS_ReadToken(script, &mut tok) == 0 { return 0; }
	//if the type matches
	if tok.type_ == type_ &&
			(tok.subtype & subtype) == subtype {
		Com_Memcpy(token as *mut c_void, &tok as *const token_t as *const c_void, core::mem::size_of::<token_t>());
		return 1;
	} //end if
	//token is not available
	(*script).script_p = (*script).lastscript_p;
	return 0;
} //end of the function PS_CheckTokenType

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_SkipUntilString(script: *mut script_t, string: *mut c_char) -> c_int
{
	let mut token: token_t = core::mem::zeroed();

	loop {
		if PS_ReadToken(script, &mut token) == 0 { break; }
		if strcmp(token.string.as_ptr(), string) == 0 { return 1; }
	} //end while
	return 0;
} //end of the function PS_SkipUntilString

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_UnreadLastToken(script: *mut script_t)
{
	(*script).tokenavailable = 1;
} //end of the function UnreadLastToken

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_UnreadToken(script: *mut script_t, token: *mut token_t)
{
	Com_Memcpy(&mut (*script).token as *mut token_t as *mut c_void, token as *const c_void, core::mem::size_of::<token_t>());
	(*script).tokenavailable = 1;
} //end of the function UnreadToken

//============================================================================
// returns the next character of the read white space, returns NULL if none
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_NextWhiteSpaceChar(script: *mut script_t) -> c_char
{
	if (*script).whitespace_p != (*script).endwhitespace_p {
		let ch = *(*script).whitespace_p;
		(*script).whitespace_p = (*script).whitespace_p.add(1);
		return ch;
	} //end if
	else {
		return 0 as c_char;
	} //end else
} //end of the function PS_NextWhiteSpaceChar

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn StripDoubleQuotes(string: *mut c_char)
{
	if *string == '\"' as c_char {
		strcpy(string, string.add(1));
	} //end if
	if *string.add(strlen(string) - 1) == '\"' as c_char {
		*string.add(strlen(string) - 1) = 0 as c_char;
	} //end if
} //end of the function StripDoubleQuotes

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn StripSingleQuotes(string: *mut c_char)
{
	if *string == '\'' as c_char {
		strcpy(string, string.add(1));
	} //end if
	if *string.add(strlen(string) - 1) == '\'' as c_char {
		*string.add(strlen(string) - 1) = 0 as c_char;
	} //end if
} //end of the function StripSingleQuotes

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn ReadSignedFloat(script: *mut script_t) -> f64
{
	let mut token: token_t = core::mem::zeroed();
	let mut sign: f64 = 1.0;

	PS_ExpectAnyToken(script, &mut token);
	if strcmp(token.string.as_ptr(), b"-\x00".as_ptr() as *const c_char) == 0 {
		sign = -1.0;
		PS_ExpectTokenType(script, TT_NUMBER, 0, &mut token);
	} //end if
	else if (*token).type_ != TT_NUMBER {
		ScriptError(script, b"expected float value, found %s\n\x00".as_ptr() as *mut c_char, (*token).string.as_ptr());
	} //end else if
	return sign * (*token).floatvalue;
} //end of the function ReadSignedFloat

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn ReadSignedInt(script: *mut script_t) -> c_int
{
	let mut token: token_t = core::mem::zeroed();
	let mut sign: c_int = 1;

	PS_ExpectAnyToken(script, &mut token);
	if strcmp(token.string.as_ptr(), b"-\x00".as_ptr() as *const c_char) == 0 {
		sign = -1;
		PS_ExpectTokenType(script, TT_NUMBER, TT_INTEGER, &mut token);
	} //end if
	else if (*token).type_ != TT_NUMBER || (*token).subtype == TT_FLOAT {
		ScriptError(script, b"expected integer value, found %s\n\x00".as_ptr() as *mut c_char, (*token).string.as_ptr());
	} //end else if
	return sign * (*token).intvalue as c_int;
} //end of the function ReadSignedInt

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn SetScriptFlags(script: *mut script_t, flags: c_int)
{
	(*script).flags = flags;
} //end of the function SetScriptFlags

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn GetScriptFlags(script: *mut script_t) -> c_int
{
	return (*script).flags;
} //end of the function GetScriptFlags

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn ResetScript(script: *mut script_t)
{
	//pointer in script buffer
	(*script).script_p = (*script).buffer;
	//pointer in script buffer before reading token
	(*script).lastscript_p = (*script).buffer;
	//begin of white space
	(*script).whitespace_p = ptr::null_mut();
	//end of white space
	(*script).endwhitespace_p = ptr::null_mut();
	//set if there's a token available in script->token
	(*script).tokenavailable = 0;
	//
	(*script).line = 1;
	(*script).lastline = 1;
	//clear the saved token
	Com_Memset(&mut (*script).token as *mut token_t as *mut c_void, 0, core::mem::size_of::<token_t>());
} //end of the function ResetScript

//============================================================================
// returns true if at the end of the script
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn EndOfScript(script: *mut script_t) -> c_int
{
	if (*script).script_p >= (*script).end_p {
		return 1;
	}
	return 0;
} //end of the function EndOfScript

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn NumLinesCrossed(script: *mut script_t) -> c_int
{
	return (*script).line - (*script).lastline;
} //end of the function NumLinesCrossed

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn ScriptSkipTo(script: *mut script_t, value: *mut c_char) -> c_int
{
	let mut len: c_int;
	let mut firstchar: c_char;

	firstchar = *value;
	len = strlen(value) as c_int;
	loop {
		if PS_ReadWhiteSpace(script) == 0 { return 0; }
		if *(*script).script_p == firstchar {
			if strncmp((*script).script_p, value, len as usize) == 0 {
				return 1;
			} //end if
		} //end if
		(*script).script_p = (*script).script_p.add(1);
	} //end do
} //end of the function ScriptSkipTo

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn LoadScriptFile(filename: *const c_char) -> *mut script_t
{
	// Note: This function requires botimport.FS_FOpenFile and botimport.FS_Read
	// which are not directly available in Rust. This is a structural placeholder.
	// In BOTLIB mode, this function should be implemented via external C functions.
	// The C code logic:
	// 	fileHandle_t fp;
	// 	char pathname[MAX_QPATH];
	// 	int length;
	// 	void *buffer;
	// 	script_t *script;
	//
	// 	if (strlen(basefolder))
	// 		Com_sprintf(pathname, sizeof(pathname), "%s/%s", basefolder, filename);
	// 	else
	// 		Com_sprintf(pathname, sizeof(pathname), "%s", filename);
	// 	length = botimport.FS_FOpenFile( pathname, &fp, FS_READ );
	// 	if (!fp) return NULL;
	//
	// 	buffer = GetClearedMemory(sizeof(script_t) + length + 1);
	// 	script = (script_t *) buffer;
	// 	Com_Memset(script, 0, sizeof(script_t));
	// 	strcpy(script->filename, filename);
	// 	script->buffer = (char *) buffer + sizeof(script_t);
	// 	script->buffer[length] = 0;
	// 	script->length = length;
	// 	script->script_p = script->buffer;
	// 	script->lastscript_p = script->buffer;
	// 	script->end_p = &script->buffer[length];
	// 	script->tokenavailable = 0;
	// 	script->line = 1;
	// 	script->lastline = 1;
	// 	SetScriptPunctuations(script, NULL);
	// 	botimport.FS_Read(script->buffer, length, fp);
	// 	botimport.FS_FCloseFile(fp);
	// 	script->length = COM_Compress(script->buffer);
	// 	return script;

	return ptr::null_mut();
} //end of the function LoadScriptFile

//============================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//============================================================================
pub unsafe fn LoadScriptMemory(ptr_: *mut c_char, length: c_int, name: *mut c_char) -> *mut script_t
{
	let mut buffer: *mut c_void;
	let mut script: *mut script_t;

	buffer = GetClearedMemory(core::mem::size_of::<script_t>() + length as usize + 1);
	script = buffer as *mut script_t;
	Com_Memset(script as *mut c_void, 0, core::mem::size_of::<script_t>());
	strcpy((*script).filename.as_mut_ptr(), name);
	(*script).buffer = buffer.add(core::mem::size_of::<script_t>()) as *mut c_char;
	*(*script).buffer.add(length as usize) = 0 as c_char;
	(*script).length = length;
	//pointer in script buffer
	(*script).script_p = (*script).buffer;
	//pointer in script buffer before reading token
	(*script).lastscript_p = (*script).buffer;
	//pointer to end of script buffer
	(*script).end_p = &mut *(*script).buffer.add(length as usize);
	//set if there's a token available in script->token
	(*script).tokenavailable = 0;
	//
	(*script).line = 1;
	(*script).lastline = 1;
	//
	SetScriptPunctuations(script, ptr::null_mut());
	//
	Com_Memcpy((*script).buffer as *mut c_void, ptr_ as *const c_void, length as usize);
	//
	return script;
} //end of the function LoadScriptMemory

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn FreeScript(script: *mut script_t)
{
	if !(*script).punctuationtable.is_null() {
		FreeMemory((*script).punctuationtable as *mut c_void);
	}
	FreeMemory(script as *mut c_void);
} //end of the function FreeScript

//============================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//============================================================================
pub unsafe fn PS_SetBaseFolder(path: *mut c_char)
{
	sprintf(core::ptr::addr_of_mut!(basefolder).as_mut_ptr(), b"%s\x00".as_ptr() as *const c_char, path);
} //end of the function PS_SetBaseFolder
