
/*****************************************************************************
 * name:		l_struct.c
 *
 * desc:		structure reading / writing
 *
 * $Archive: /MissionPack/CODE/botlib/l_struct.c $
 * $Author: Raduffy $
 * $Revision: 1 $
 * $Modtime: 12/20/99 8:43p $
 * $Date: 3/08/00 11:28a $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr;
use crate::codemp::botlib::l_struct_h::*;

// External declarations for libc functions
extern "C" {
	pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
	pub fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
	pub fn strlen(s: *const c_char) -> usize;
	pub fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
	pub fn fprintf(stream: *mut FILE, format: *const c_char, ...) -> c_int;
}

// External botlib functions
extern "C" {
	pub fn SourceError(source: *mut source_t, format: *const c_char, ...);
	pub fn StripSingleQuotes(string: *mut c_char);
	pub fn StripDoubleQuotes(string: *mut c_char);
	pub fn PC_ExpectAnyToken(source: *mut source_t, token: *mut token_t) -> c_int;
	pub fn PC_ExpectTokenType(source: *mut source_t, ttype: c_int, subtype: c_int, token: *mut token_t) -> c_int;
	pub fn PC_ExpectTokenString(source: *mut source_t, string: *const c_char) -> c_int;
	pub fn PC_CheckTokenString(source: *mut source_t, string: *const c_char) -> c_int;
	pub fn PC_UnreadLastToken(source: *mut source_t);
}

// Token type constants
const TT_STRING: c_int = 1;			// string
const TT_LITERAL: c_int = 2;			// literal
const TT_NUMBER: c_int = 3;			// number
const TT_PUNCTUATION: c_int = 5;		// punctuation

// Token subtype constants
const TT_FLOAT: c_int = 0x0800;	// floating point number

// Constants
const qfalse: c_int = 0;
const qtrue: c_int = 1;

// Stub types for external dependencies (opaque)
#[repr(C)]
pub struct token_t {
	pub string: [c_char; 1024usize],
	pub type_: c_int,
	pub subtype: c_int,
	pub intvalue: c_int,
	pub floatvalue: f32,
}

// Utility function declarations
fn Maximum<T: PartialOrd + Copy>(x: T, y: T) -> T {
	if x > y { x } else { y }
}

fn Minimum<T: PartialOrd + Copy>(x: T, y: T) -> T {
	if x < y { x } else { y }
}

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn FindField(defs: *mut fielddef_t, name: *mut c_char) -> *mut fielddef_t {
	let mut i: c_int = 0;

	loop {
		if (*defs.add(i as usize)).name.is_null() { break; }
		if strcmp((*defs.add(i as usize)).name, name) == 0 {
			return defs.add(i as usize);
		}
		i += 1;
	} //end for
	return ptr::null_mut();
} //end of the function FindField
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn ReadNumber(source: *mut source_t, fd: *mut fielddef_t, p: *mut c_void) -> c_int {
	let mut token: token_t = core::mem::zeroed();
	let mut negative: c_int = qfalse;
	let mut intval: i64;
	let mut intmin: i64 = 0;
	let mut intmax: i64 = 0;
	let mut floatval: f64;

	if PC_ExpectAnyToken(source, &mut token) == 0 { return qfalse; }

	//check for minus sign
	if token.type_ == TT_PUNCTUATION {
		if ((*fd).type_ & FT_UNSIGNED) != 0 {
			SourceError(source, b"expected unsigned value, found %s\0".as_ptr() as *const c_char, token.string.as_ptr());
			return qfalse;
		} //end if
		//if not a minus sign
		if strcmp(token.string.as_ptr(), b"-\0".as_ptr() as *const c_char) != 0 {
			SourceError(source, b"unexpected punctuation %s\0".as_ptr() as *const c_char, token.string.as_ptr());
			return qfalse;
		} //end if
		negative = qtrue;
		//read the number
		if PC_ExpectAnyToken(source, &mut token) == 0 { return qfalse; }
	} //end if
	//check if it is a number
	if token.type_ != TT_NUMBER {
		SourceError(source, b"expected number, found %s\0".as_ptr() as *const c_char, token.string.as_ptr());
		return qfalse;
	} //end if
	//check for a float value
	if (token.subtype & TT_FLOAT) != 0 {
		if ((*fd).type_ & FT_TYPE) != FT_FLOAT {
			SourceError(source, b"unexpected float\0".as_ptr() as *const c_char);
			return qfalse;
		} //end if
		floatval = token.floatvalue as f64;
		if negative != qfalse { floatval = -floatval; }
		if ((*fd).type_ & FT_BOUNDED) != 0 {
			if floatval < (*fd).floatmin as f64 || floatval > (*fd).floatmax as f64 {
				SourceError(source, b"float out of range [%f, %f]\0".as_ptr() as *const c_char, (*fd).floatmin, (*fd).floatmax);
				return qfalse;
			} //end if
		} //end if
		*(p as *mut f32) = floatval as f32;
		return qtrue;
	} //end if
	//
	intval = token.intvalue as i64;
	if negative != qfalse { intval = -intval; }
	//check bounds
	if ((*fd).type_ & FT_TYPE) == FT_CHAR {
		if ((*fd).type_ & FT_UNSIGNED) != 0 { intmin = 0; intmax = 255; }
		else { intmin = -128; intmax = 127; }
	} //end if
	if ((*fd).type_ & FT_TYPE) == FT_INT {
		if ((*fd).type_ & FT_UNSIGNED) != 0 { intmin = 0; intmax = 65535; }
		else { intmin = -32768; intmax = 32767; }
	} //end else if
	if ((*fd).type_ & FT_TYPE) == FT_CHAR || ((*fd).type_ & FT_TYPE) == FT_INT {
		if ((*fd).type_ & FT_BOUNDED) != 0 {
			intmin = Maximum(intmin, (*fd).floatmin as i64);
			intmax = Minimum(intmax, (*fd).floatmax as i64);
		} //end if
		if intval < intmin || intval > intmax {
			SourceError(source, b"value %d out of range [%d, %d]\0".as_ptr() as *const c_char, intval, intmin, intmax);
			return qfalse;
		} //end if
	} //end if
	else if ((*fd).type_ & FT_TYPE) == FT_FLOAT {
		if ((*fd).type_ & FT_BOUNDED) != 0 {
			if intval as f64 < (*fd).floatmin as f64 || intval as f64 > (*fd).floatmax as f64 {
				SourceError(source, b"value %d out of range [%f, %f]\0".as_ptr() as *const c_char, intval, (*fd).floatmin, (*fd).floatmax);
				return qfalse;
			} //end if
		} //end if
	} //end else if
	//store the value
	if ((*fd).type_ & FT_TYPE) == FT_CHAR {
		if ((*fd).type_ & FT_UNSIGNED) != 0 { *(p as *mut u8) = intval as u8; }
		else { *(p as *mut c_char) = intval as c_char; }
	} //end if
	else if ((*fd).type_ & FT_TYPE) == FT_INT {
		if ((*fd).type_ & FT_UNSIGNED) != 0 { *(p as *mut u32) = intval as u32; }
		else { *(p as *mut c_int) = intval as c_int; }
	} //end else
	else if ((*fd).type_ & FT_TYPE) == FT_FLOAT {
		*(p as *mut f32) = intval as f32;
	} //end else
	return qtrue;
} //end of the function ReadNumber
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn ReadChar(source: *mut source_t, fd: *mut fielddef_t, p: *mut c_void) -> c_int {
	let mut token: token_t = core::mem::zeroed();

	if PC_ExpectAnyToken(source, &mut token) == 0 { return qfalse; }

	//take literals into account
	if token.type_ == TT_LITERAL {
		StripSingleQuotes(token.string.as_mut_ptr());
		*(p as *mut c_char) = token.string[0];
	} //end if
	else {
		PC_UnreadLastToken(source);
		if ReadNumber(source, fd, p) == 0 { return qfalse; }
	} //end if
	return qtrue;
} //end of the function ReadChar
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn ReadString(source: *mut source_t, fd: *mut fielddef_t, p: *mut c_void) -> c_int {
	let mut token: token_t = core::mem::zeroed();

	if PC_ExpectTokenType(source, TT_STRING, 0, &mut token) == 0 { return 0; }
	//remove the double quotes
	StripDoubleQuotes(token.string.as_mut_ptr());
	//copy the string
	strncpy(p as *mut c_char, token.string.as_ptr(), MAX_STRINGFIELD as usize);
	//make sure the string is closed with a zero
	*((p as *mut c_char).add(MAX_STRINGFIELD as usize - 1)) = '\0' as c_char;
	//
	return 1;
} //end of the function ReadString
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn ReadStructure(source: *mut source_t, def: *mut structdef_t, structure: *mut c_char) -> c_int {
	let mut token: token_t = core::mem::zeroed();
	let mut fd: *mut fielddef_t;
	let mut p: *mut c_void;
	let mut num: c_int;

	if PC_ExpectTokenString(source, b"{\0".as_ptr() as *const c_char) == 0 { return 0; }
	loop {
		if PC_ExpectAnyToken(source, &mut token) == 0 { return qfalse; }
		//if end of structure
		if strcmp(token.string.as_ptr(), b"}\0".as_ptr() as *const c_char) == 0 { break; }
		//find the field with the name
		fd = FindField((*def).fields, token.string.as_mut_ptr());
		if fd.is_null() {
			SourceError(source, b"unknown structure field %s\0".as_ptr() as *const c_char, token.string.as_ptr());
			return qfalse;
		} //end if
		if ((*fd).type_ & FT_ARRAY) != 0 {
			num = (*fd).maxarray;
			if PC_ExpectTokenString(source, b"{\0".as_ptr() as *const c_char) == 0 { return qfalse; }
		} //end if
		else {
			num = 1;
		} //end else
		p = (structure as *mut c_void).add(((*fd).offset) as usize);
		while num > 0 {
			if ((*fd).type_ & FT_ARRAY) != 0 {
				if PC_CheckTokenString(source, b"}\0".as_ptr() as *const c_char) != 0 { break; }
			} //end if
			match (*fd).type_ & FT_TYPE {
				x if x == FT_CHAR => {
					if ReadChar(source, fd, p) == 0 { return qfalse; }
					p = (p as *mut c_char).add(core::mem::size_of::<c_char>()) as *mut c_void;
				} //end case
				x if x == FT_INT => {
					if ReadNumber(source, fd, p) == 0 { return qfalse; }
					p = (p as *mut c_char).add(core::mem::size_of::<c_int>()) as *mut c_void;
				} //end case
				x if x == FT_FLOAT => {
					if ReadNumber(source, fd, p) == 0 { return qfalse; }
					p = (p as *mut c_char).add(core::mem::size_of::<f32>()) as *mut c_void;
				} //end case
				x if x == FT_STRING => {
					if ReadString(source, fd, p) == 0 { return qfalse; }
					p = (p as *mut c_char).add(MAX_STRINGFIELD as usize) as *mut c_void;
				} //end case
				x if x == FT_STRUCT => {
					if (*fd).substruct.is_null() {
						SourceError(source, b"BUG: no sub structure defined\0".as_ptr() as *const c_char);
						return qfalse;
					} //end if
					ReadStructure(source, (*fd).substruct, p as *mut c_char);
					p = (p as *mut c_char).add((*(*fd).substruct).size as usize) as *mut c_void;
				} //end case
				_ => {}
			} //end switch
			if ((*fd).type_ & FT_ARRAY) != 0 {
				if PC_ExpectAnyToken(source, &mut token) == 0 { return qfalse; }
				if strcmp(token.string.as_ptr(), b"}\0".as_ptr() as *const c_char) == 0 { break; }
				if strcmp(token.string.as_ptr(), b",\0".as_ptr() as *const c_char) != 0 {
					SourceError(source, b"expected a comma, found %s\0".as_ptr() as *const c_char, token.string.as_ptr());
					return qfalse;
				} //end if
			} //end if
			num -= 1;
		} //end while
	} //end while
	return qtrue;
} //end of the function ReadStructure
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn WriteIndent(fp: *mut FILE, indent: c_int) -> c_int {
	let mut i: c_int = indent;
	while i > 0 {
		if fprintf(fp, b"\t\0".as_ptr() as *const c_char) < 0 { return qfalse; }
		i -= 1;
	} //end while
	return qtrue;
} //end of the function WriteIndent
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn WriteFloat(fp: *mut FILE, value: f32) -> c_int {
	let mut buf: [c_char; 128] = [0; 128];
	let mut l: c_int;

	sprintf(buf.as_mut_ptr(), b"%f\0".as_ptr() as *const c_char, value);
	l = strlen(buf.as_ptr()) as c_int;
	//strip any trailing zeros
	while l > 1 {
		l -= 1;
		if buf[l as usize] != '0' as c_char && buf[l as usize] != '.' as c_char { break; }
		if buf[l as usize] == '.' as c_char {
			buf[l as usize] = 0 as c_char;
			break;
		} //end if
		buf[l as usize] = 0 as c_char;
	} //end while
	//write the float to file
	if fprintf(fp, b"%s\0".as_ptr() as *const c_char, buf.as_ptr()) < 0 { return 0; }
	return 1;
} //end of the function WriteFloat
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
unsafe fn WriteStructWithIndent(fp: *mut FILE, def: *mut structdef_t, structure: *mut c_char, indent: c_int) -> c_int {
	let mut i: c_int = 0;
	let mut num: c_int;
	let mut p: *mut c_void;
	let mut fd: *mut fielddef_t;
	let mut indent_mut = indent;

	if WriteIndent(fp, indent_mut) == qfalse { return qfalse; }
	if fprintf(fp, b"{\r\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }

	indent_mut += 1;
	loop {
		if (*(*def).fields.add(i as usize)).name.is_null() { break; }
		fd = (*def).fields.add(i as usize);
		if WriteIndent(fp, indent_mut) == qfalse { return qfalse; }
		if fprintf(fp, b"%s\t\0".as_ptr() as *const c_char, (*fd).name) < 0 { return qfalse; }
		p = (structure as *mut c_void).add(((*fd).offset) as usize);
		if ((*fd).type_ & FT_ARRAY) != 0 {
			num = (*fd).maxarray;
			if fprintf(fp, b"{\0".as_ptr() as *const c_char) < 0 { return qfalse; }
		} //end if
		else {
			num = 1;
		} //end else
		while num > 0 {
			match (*fd).type_ & FT_TYPE {
				x if x == FT_CHAR => {
					if fprintf(fp, b"%d\0".as_ptr() as *const c_char, *(p as *mut c_char)) < 0 { return qfalse; }
					p = (p as *mut c_char).add(core::mem::size_of::<c_char>()) as *mut c_void;
				} //end case
				x if x == FT_INT => {
					if fprintf(fp, b"%d\0".as_ptr() as *const c_char, *(p as *mut c_int)) < 0 { return qfalse; }
					p = (p as *mut c_char).add(core::mem::size_of::<c_int>()) as *mut c_void;
				} //end case
				x if x == FT_FLOAT => {
					if WriteFloat(fp, *(p as *mut f32)) == qfalse { return qfalse; }
					p = (p as *mut c_char).add(core::mem::size_of::<f32>()) as *mut c_void;
				} //end case
				x if x == FT_STRING => {
					if fprintf(fp, b"\"%s\"\0".as_ptr() as *const c_char, p as *mut c_char) < 0 { return qfalse; }
					p = (p as *mut c_char).add(MAX_STRINGFIELD as usize) as *mut c_void;
				} //end case
				x if x == FT_STRUCT => {
					if WriteStructWithIndent(fp, (*fd).substruct, structure, indent_mut) == qfalse { return qfalse; }
					p = (p as *mut c_char).add((*(*fd).substruct).size as usize) as *mut c_void;
				} //end case
				_ => {}
			} //end switch
			if ((*fd).type_ & FT_ARRAY) != 0 {
				if num > 1 {
					if fprintf(fp, b",\0".as_ptr() as *const c_char) < 0 { return qfalse; }
				} //end if
				else {
					if fprintf(fp, b"}\0".as_ptr() as *const c_char) < 0 { return qfalse; }
				} //end else
			} //end if
			num -= 1;
		} //end while
		if fprintf(fp, b"\r\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
		i += 1;
	} //end for
	indent_mut -= 1;

	if WriteIndent(fp, indent_mut) == qfalse { return qfalse; }
	if fprintf(fp, b"}\r\n\0".as_ptr() as *const c_char) < 0 { return qfalse; }
	return qtrue;
} //end of the function WriteStructWithIndent
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn WriteStructure(fp: *mut FILE, def: *mut structdef_t, structure: *mut c_char) -> c_int {
	return WriteStructWithIndent(fp, def, structure, 0);
} //end of the function WriteStructure
