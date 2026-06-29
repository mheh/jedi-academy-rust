// q_parse.c -- support for parsing text files

use core::ffi::{c_char, c_int};
use core::ptr::{addr_of_mut, null, null_mut};

// Constants from q_shared.h (stub definitions)
const MAX_TOKEN_CHARS: usize = 1024;
const MAX_QPATH: usize = 64;
const MAX_PARSE_INFO: usize = 16;

const ERR_FATAL: c_int = 3;
const ERR_DROP: c_int = 2;

type qboolean = c_int;
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

/*
============================================================================

PARSING

============================================================================
*/

// multiple character punctuation tokens
static PUNCTUATION: &[&[u8]] = &[
	b"+=", b"-=",  b"*=",  b"/=", b"&=", b"|=", b"++", b"--",
		b"&&", b"||",  b"<=",  b">=", b"==", b"!=",
];

#[repr(C)]
pub struct parseInfo_t {
	pub token: [c_char; MAX_TOKEN_CHARS],
	pub lines: c_int,
	pub ungetToken: qboolean,
	pub parseFile: [c_char; MAX_QPATH],
}

#[doc(hidden)]
const PARSE_INFO_INIT: parseInfo_t = parseInfo_t {
	token: [0; MAX_TOKEN_CHARS],
	lines: 0,
	ungetToken: 0,
	parseFile: [0; MAX_QPATH],
};

#[doc(hidden)]
const MAX_PARSE_INFO_CONST: usize = MAX_PARSE_INFO;

static mut PARSE_INFO: [parseInfo_t; MAX_PARSE_INFO] = [PARSE_INFO_INIT; MAX_PARSE_INFO];
static mut PARSE_INFO_NUM: c_int = 0;

// SAFETY: PI is initialized to point to PARSE_INFO[0] and is updated by Com_BeginParseSession/Com_EndParseSession
// to point to a valid element within PARSE_INFO array.
static mut PI: *mut parseInfo_t = unsafe { addr_of_mut!(PARSE_INFO[0]) };

// External C functions
extern "C" {
	fn Com_Error(code: c_int, fmt: *const c_char, ...) -> !;
	fn Com_Printf(fmt: *const c_char, ...);
	fn Q_strncpyz(dst: *mut c_char, src: *const c_char, len: usize);
	fn Q_strcat(dst: *mut c_char, len: usize, src: *const c_char);
}

// Standard C library functions
extern "C" {
	fn strlen(s: *const c_char) -> usize;
	fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
	fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> *mut core::ffi::c_void;
	fn atof(s: *const c_char) -> f64;
	fn atoi(s: *const c_char) -> c_int;
}

/*
===================
Com_BeginParseSession
===================
*/
pub fn Com_BeginParseSession(filename: *const c_char) {
	unsafe {
		if PARSE_INFO_NUM == (MAX_PARSE_INFO as c_int) - 1 {
			Com_Error(ERR_FATAL, c"Com_BeginParseSession: session overflow".as_ptr());
		}
		PARSE_INFO_NUM += 1;
		PI = &mut PARSE_INFO[PARSE_INFO_NUM as usize];

		(*PI).lines = 1;
		Q_strncpyz(
			(*PI).parseFile.as_mut_ptr(),
			filename,
			core::mem::size_of_val(&(*PI).parseFile),
		);
	}
}

/*
===================
Com_EndParseSession
===================
*/
pub fn Com_EndParseSession() {
	unsafe {
		if PARSE_INFO_NUM == 0 {
			Com_Error(ERR_FATAL, c"Com_EndParseSession: session underflow".as_ptr());
		}
		PARSE_INFO_NUM -= 1;
		PI = &mut PARSE_INFO[PARSE_INFO_NUM as usize];
	}
}

/*
===================
Com_GetCurrentParseLine
===================
*/
pub fn Com_GetCurrentParseLine() -> c_int {
	unsafe { (*PI).lines }
}

/*
===================
Com_ScriptError

Prints the script name and line number in the message
===================
*/
pub fn Com_ScriptError(msg: *const c_char) {
	unsafe {
		Com_Error(
			ERR_DROP,
			c"File %s, line %i: %s".as_ptr(),
			(*PI).parseFile.as_ptr(),
			(*PI).lines,
			msg,
		);
	}
}

pub fn Com_ScriptWarning(msg: *const c_char) {
	unsafe {
		Com_Printf(
			c"File %s, line %i: %s".as_ptr(),
			(*PI).parseFile.as_ptr(),
			(*PI).lines,
			msg,
		);
	}
}

/*
===================
Com_UngetToken

Calling this will make the next Com_Parse return
the current token instead of advancing the pointer
===================
*/
pub fn Com_UngetToken() {
	unsafe {
		if (*PI).ungetToken != 0 {
			Com_Error(ERR_FATAL, c"UngetToken called twice".as_ptr());
		}
		(*PI).ungetToken = qtrue;
	}
}

fn SkipWhitespace(data: *const c_char, hasNewLines: *mut qboolean) -> *const c_char {
	let mut c: c_int;
	let mut current_data = data;

	unsafe {
		loop {
			c = *current_data as u8 as c_int;
			if c <= ' ' as c_int {
				if c == 0 {
					return null();
				}
				if c == '\n' as c_int {
					(*PI).lines += 1;
					*hasNewLines = qtrue;
				}
				current_data = current_data.add(1);
			} else {
				break;
			}
		}
	}

	current_data
}

/*
==============
Com_ParseExt

Parse a token out of a string
Will never return NULL, just empty strings.
An empty string will only be returned at end of file.

If "allowLineBreaks" is qtrue then an empty
string will be returned if the next token is
a newline.
==============
*/
fn Com_ParseExt(data_p: *mut *const c_char, allowLineBreaks: qboolean) -> *mut c_char {
	let mut c: c_int = 0;
	let mut len: c_int;
	let mut hasNewLines: qboolean = qfalse;
	let mut data: *const c_char;

	unsafe {
		if data_p.is_null() {
			Com_Error(ERR_FATAL, c"Com_ParseExt: NULL data_p".as_ptr());
		}

		data = *data_p;
		len = 0;
		(*PI).token[0] = 0;

		// make sure incoming data is valid
		if data.is_null() {
			*data_p = null();
			return (*PI).token.as_mut_ptr();
		}

		// skip any leading whitespace
		loop {
			// skip whitespace
			data = SkipWhitespace(data, &mut hasNewLines);
			if data.is_null() {
				*data_p = null();
				return (*PI).token.as_mut_ptr();
			}
			if hasNewLines != 0 && allowLineBreaks == 0 {
				*data_p = data;
				return (*PI).token.as_mut_ptr();
			}

			c = *data as u8 as c_int;

			// skip double slash comments
			if c == '/' as c_int && *data.add(1) as u8 as c_int == '/' as c_int {
				while *data != 0 && *data as u8 as c_int != '\n' as c_int {
					data = data.add(1);
				}
				continue;
			}

			// skip /* */ comments
			if c == '/' as c_int && *data.add(1) as u8 as c_int == '*' as c_int {
				while *data != 0 && !(*data as u8 as c_int == '*' as c_int && *data.add(1) as u8 as c_int == '/' as c_int) {
					if *data as u8 as c_int == '\n' as c_int {
						(*PI).lines += 1;
					}
					data = data.add(1);
				}
				if *data != 0 {
					data = data.add(2);
				}
				continue;
			}

			// a real token to parse
			break;
		}

		// handle quoted strings
		if c == '"' as c_int {
			data = data.add(1);
			loop {
				c = *data as u8 as c_int;
				data = data.add(1);
				if (c as c_char) == '\\' as c_char && (*data as c_char) == '"' as c_char {
					// allow quoted strings to use \" to indicate the " character
					data = data.add(1);
				} else if (c as c_char) == '"' as c_char || c == 0 {
					(*PI).token[len as usize] = 0;
					*data_p = data;
					return (*PI).token.as_mut_ptr();
				} else if *data as u8 as c_int == '\n' as c_int {
					(*PI).lines += 1;
				}
				if len < (MAX_TOKEN_CHARS as c_int) - 1 {
					(*PI).token[len as usize] = c as c_char;
					len += 1;
				}
			}
		}

		// check for a number
		// is this parsing of negative numbers going to cause expression problems
		if (c >= '0' as c_int && c <= '9' as c_int)
			|| (c == '-' as c_int && *data.add(1) as u8 as c_int >= '0' as c_int && *data.add(1) as u8 as c_int <= '9' as c_int)
			|| (c == '.' as c_int && *data.add(1) as u8 as c_int >= '0' as c_int && *data.add(1) as u8 as c_int <= '9' as c_int)
		{
			loop {
				if len < (MAX_TOKEN_CHARS as c_int) - 1 {
					(*PI).token[len as usize] = c as c_char;
					len += 1;
				}
				data = data.add(1);

				c = *data as u8 as c_int;

				if !((c >= '0' as c_int && c <= '9' as c_int) || c == '.' as c_int) {
					break;
				}
			}

			// parse the exponent
			if c == 'e' as c_int || c == 'E' as c_int {
				if len < (MAX_TOKEN_CHARS as c_int) - 1 {
					(*PI).token[len as usize] = c as c_char;
					len += 1;
				}
				data = data.add(1);
				c = *data as u8 as c_int;

				if c == '-' as c_int || c == '+' as c_int {
					if len < (MAX_TOKEN_CHARS as c_int) - 1 {
						(*PI).token[len as usize] = c as c_char;
						len += 1;
					}
					data = data.add(1);
					c = *data as u8 as c_int;
				}

				loop {
					if len < (MAX_TOKEN_CHARS as c_int) - 1 {
						(*PI).token[len as usize] = c as c_char;
						len += 1;
					}
					data = data.add(1);

					c = *data as u8 as c_int;

					if !(c >= '0' as c_int && c <= '9' as c_int) {
						break;
					}
				}
			}

			if len == MAX_TOKEN_CHARS as c_int {
				len = 0;
			}
			(*PI).token[len as usize] = 0;

			*data_p = data;
			return (*PI).token.as_mut_ptr();
		}

		// check for a regular word
		// we still allow forward and back slashes in name tokens for pathnames
		// and also colons for drive letters
		if (c >= 'a' as c_int && c <= 'z' as c_int)
			|| (c >= 'A' as c_int && c <= 'Z' as c_int)
			|| c == '_' as c_int
			|| c == '/' as c_int
			|| c == '\\' as c_int
		{
			loop {
				if len < (MAX_TOKEN_CHARS as c_int) - 1 {
					(*PI).token[len as usize] = c as c_char;
					len += 1;
				}
				data = data.add(1);

				c = *data as u8 as c_int;

				if !((c >= 'a' as c_int && c <= 'z' as c_int)
					|| (c >= 'A' as c_int && c <= 'Z' as c_int)
					|| c == '_' as c_int
					|| (c >= '0' as c_int && c <= '9' as c_int)
					|| c == '/' as c_int
					|| c == '\\' as c_int
					|| c == ':' as c_int
					|| c == '.' as c_int)
				{
					break;
				}
			}

			if len == MAX_TOKEN_CHARS as c_int {
				len = 0;
			}
			(*PI).token[len as usize] = 0;

			*data_p = data;
			return (*PI).token.as_mut_ptr();
		}

		// check for multi-character punctuation token
		let mut punc_idx = 0;
		while punc_idx < PUNCTUATION.len() {
			let punc_bytes = PUNCTUATION[punc_idx];
			let l = punc_bytes.len();
			let mut j = 0;
			while j < l {
				if *data.add(j) as u8 != punc_bytes[j] {
					break;
				}
				j += 1;
			}
			if j == l {
				// a valid multi-character punctuation
				memcpy(
					(*PI).token.as_mut_ptr() as *mut core::ffi::c_void,
					punc_bytes.as_ptr() as *const core::ffi::c_void,
					l,
				);
				(*PI).token[l] = 0;
				data = data.add(l);
				*data_p = data;
				return (*PI).token.as_mut_ptr();
			}
			punc_idx += 1;
		}

		// single character punctuation
		(*PI).token[0] = *data;
		(*PI).token[1] = 0;
		data = data.add(1);
		*data_p = data;

		(*PI).token.as_mut_ptr()
	}
}

/*
===================
Com_Parse
===================
*/
pub fn Com_Parse(data_p: *mut *const c_char) -> *const c_char {
	unsafe {
		if (*PI).ungetToken != 0 {
			(*PI).ungetToken = qfalse;
			return (*PI).token.as_ptr();
		}
		Com_ParseExt(data_p, qtrue) as *const c_char
	}
}

/*
===================
Com_ParseOnLine
===================
*/
pub fn Com_ParseOnLine(data_p: *mut *const c_char) -> *const c_char {
	unsafe {
		if (*PI).ungetToken != 0 {
			(*PI).ungetToken = qfalse;
			return (*PI).token.as_ptr();
		}
		Com_ParseExt(data_p, qfalse) as *const c_char
	}
}

/*
==================
Com_MatchToken
==================
*/
pub fn Com_MatchToken(buf_p: *mut *const c_char, match_: *const c_char, warning: qboolean) {
	unsafe {
		let token = Com_Parse(buf_p);
		if strcmp(token, match_) != 0 {
			if warning != 0 {
				Com_ScriptWarning(c"MatchToken: %s != %s".as_ptr());
			} else {
				Com_ScriptError(c"MatchToken: %s != %s".as_ptr());
			}
		}
	}
}

/*
=================
Com_SkipBracedSection

The next token should be an open brace.
Skips until a matching close brace is found.
Internal brace depths are properly skipped.
=================
*/
pub fn Com_SkipBracedSection(program: *mut *const c_char) {
	let mut depth: c_int = 0;
	unsafe {
		loop {
			let token = Com_Parse(program);
			if *token.add(1) == 0 {
				if *token as u8 as c_int == '{' as c_int {
					depth += 1;
				} else if *token as u8 as c_int == '}' as c_int {
					depth -= 1;
				}
			}
			if !(depth != 0 && !(*program).is_null()) {
				break;
			}
		}
	}
}

/*
=================
Com_SkipRestOfLine
=================
*/
pub fn Com_SkipRestOfLine(data: *mut *const c_char) {
	unsafe {
		let mut p = *data;
		loop {
			let c = *p as u8 as c_int;
			p = p.add(1);
			if c == 0 {
				break;
			}
			if c == '\n' as c_int {
				(*PI).lines += 1;
				break;
			}
		}

		*data = p;
	}
}

/*
====================
Com_ParseRestOfLine
====================
*/
pub fn Com_ParseRestOfLine(data_p: *mut *const c_char) -> *const c_char {
	static mut LINE: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];

	unsafe {
		LINE[0] = 0;
		loop {
			let token = Com_ParseOnLine(data_p);
			if *token == 0 {
				break;
			}
			if LINE[0] != 0 {
				Q_strcat(LINE.as_mut_ptr(), core::mem::size_of_val(&LINE), c" ".as_ptr());
			}
			Q_strcat(LINE.as_mut_ptr(), core::mem::size_of_val(&LINE), token);
		}

		LINE.as_ptr()
	}
}

pub fn Com_ParseFloat(buf_p: *mut *const c_char) -> f64 {
	unsafe {
		let token = Com_Parse(buf_p);
		if *token == 0 {
			return 0.0;
		}
		atof(token)
	}
}

pub fn Com_ParseInt(buf_p: *mut *const c_char) -> c_int {
	unsafe {
		let token = Com_Parse(buf_p);
		if *token == 0 {
			return 0;
		}
		atoi(token)
	}
}

pub fn Com_Parse1DMatrix(buf_p: *mut *const c_char, x: c_int, m: *mut f64) {
	unsafe {
		Com_MatchToken(buf_p, c"(".as_ptr(), qfalse);

		for i in 0..x {
			let token = Com_Parse(buf_p);
			*m.add(i as usize) = atof(token);
		}

		Com_MatchToken(buf_p, c")".as_ptr(), qfalse);
	}
}

pub fn Com_Parse2DMatrix(buf_p: *mut *const c_char, y: c_int, x: c_int, m: *mut f64) {
	unsafe {
		Com_MatchToken(buf_p, c"(".as_ptr(), qfalse);

		for i in 0..y {
			Com_Parse1DMatrix(buf_p, x, m.add((i * x) as usize));
		}

		Com_MatchToken(buf_p, c")".as_ptr(), qfalse);
	}
}

pub fn Com_Parse3DMatrix(buf_p: *mut *const c_char, z: c_int, y: c_int, x: c_int, m: *mut f64) {
	unsafe {
		Com_MatchToken(buf_p, c"(".as_ptr(), qfalse);

		for i in 0..z {
			Com_Parse2DMatrix(buf_p, y, x, m.add((i * x * y) as usize));
		}

		Com_MatchToken(buf_p, c")".as_ptr(), qfalse);
	}
}
