//! /*****************************************************************************
//!  * name:		l_struct.h
//!  *
//!  * desc:		structure reading/writing
//!  *
//!  * $Archive: /source/code/botlib/l_struct.h $
//!  * $Author: Mrelusive $
//!  * $Revision: 2 $
//!  * $Modtime: 10/05/99 3:32p $
//!  * $Date: 10/05/99 3:42p $
//!  *
//!  *****************************************************************************/

use core::ffi::{c_char, c_int, c_void};

pub const MAX_STRINGFIELD: c_int = 80;
//field types
pub const FT_CHAR: c_int = 1;			// char
pub const FT_INT: c_int = 2;			// int
pub const FT_FLOAT: c_int = 3;			// float
pub const FT_STRING: c_int = 4;			// char [MAX_STRINGFIELD]
pub const FT_STRUCT: c_int = 6;			// struct (sub structure)
//type only mask
pub const FT_TYPE: c_int = 0x00FF;		// only type, clear subtype
//sub types
pub const FT_ARRAY: c_int = 0x0100;		// array of type
pub const FT_BOUNDED: c_int = 0x0200;	// bounded value
pub const FT_UNSIGNED: c_int = 0x0400;

//structure field definition
#[repr(C)]
pub struct fielddef_s
{
	pub name: *mut c_char,					//name of the field
	pub offset: c_int,						//offset in the structure
	pub type_: c_int,						//type of the field
	//type specific fields
	pub maxarray: c_int,					//maximum array size
	pub floatmin: f32,
	pub floatmax: f32,						//float min and max
	pub substruct: *mut structdef_s,		//sub structure
}

pub type fielddef_t = fielddef_s;

//structure definition
#[repr(C)]
pub struct structdef_s
{
	pub size: c_int,
	pub fields: *mut fielddef_t,
}

pub type structdef_t = structdef_s;

// Opaque type for FILE (from libc)
#[repr(C)]
pub struct FILE {
	_private: [c_void; 0],
}

// Opaque type for source_t (from botlib)
#[repr(C)]
pub struct source_t {
	_private: [c_void; 0],
}

//read a structure from a script
extern "C" {
	pub fn ReadStructure(source: *mut source_t, def: *mut structdef_t, structure: *mut c_char) -> c_int;
	//write a structure to a file
	pub fn WriteStructure(fp: *mut FILE, def: *mut structdef_t, structure: *mut c_char) -> c_int;
	//writes indents
	pub fn WriteIndent(fp: *mut FILE, indent: c_int) -> c_int;
	//writes a float without traling zeros
	pub fn WriteFloat(fp: *mut FILE, value: f32) -> c_int;
}
