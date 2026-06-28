
/*****************************************************************************
 * name:		l_libvar.c
 *
 * desc:		bot library variables
 *
 * $Archive: /MissionPack/code/botlib/l_libvar.c $
 * $Author: Zaphod $
 * $Revision: 2 $
 * $Modtime: 11/21/00 11:33a $
 * $Date: 11/21/00 11:49a $
 *
 *****************************************************************************/

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};

//library variable
#[repr(C)]
pub struct libvar_s
{
	pub name: *mut c_char,
	pub string: *mut c_char,
	pub flags: c_int,
	pub modified: qboolean,	// set each time the cvar is changed
	pub value: f32,
	pub next: *mut libvar_s,
}
pub type libvar_t = libvar_s;

// Extern declarations for libc functions
extern "C" {
	pub fn strlen(s: *const c_char) -> usize;
	pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
	pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
	pub fn GetMemory(size: usize) -> *mut c_void;
	pub fn FreeMemory(ptr: *mut c_void);
	pub fn Com_Memset(ptr: *mut c_void, value: c_int, size: usize) -> *mut c_void;
}

//list with library variables
pub static mut libvarlist: *mut libvar_t = ptr::null_mut();

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarStringValue(mut string: *const c_char) -> f32
{
	let mut dotfound: c_int = 0;
	let mut value: f32 = 0.0;

	unsafe {
		while *string != 0 as c_char
		{
			if *string < ('0' as c_char) || *string > ('9' as c_char)
			{
				if dotfound != 0 || *string != ('.' as c_char)
				{
					return 0.0;
				} //end if
				else
				{
					dotfound = 10;
					string = string.add(1);
				} //end if
			} //end if
			if dotfound != 0
			{
				value = value + ((*string as c_int) - ('0' as c_int)) as f32 / (dotfound as f32);
				dotfound *= 10;
			} //end if
			else
			{
				value = value * 10.0 + ((*string as c_int) - ('0' as c_int)) as f32;
			} //end else
			string = string.add(1);
		} //end while
	}
	return value;
} //end of the function LibVarStringValue
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarAlloc(var_name: *const c_char) -> *mut libvar_t
{
	let mut v: *mut libvar_t;

	unsafe {
		let name_len = strlen(var_name);
		v = GetMemory(core::mem::size_of::<libvar_t>() + name_len + 1) as *mut libvar_t;
		Com_Memset(v as *mut c_void, 0, core::mem::size_of::<libvar_t>());
		(*v).name = (v as *mut c_char).add(core::mem::size_of::<libvar_t>());
		strcpy((*v).name, var_name);
		//add the variable in the list
		(*v).next = libvarlist;
		libvarlist = v;
		return v;
	}
} //end of the function LibVarAlloc
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarDeAlloc(v: *mut libvar_t)
{
	unsafe {
		if !(*v).string.is_null() {
			FreeMemory((*v).string as *mut c_void);
		}
		FreeMemory(v as *mut c_void);
	}
} //end of the function LibVarDeAlloc
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarDeAllocAll()
{
	unsafe {
		let mut v: *mut libvar_t;

		v = libvarlist;
		while !v.is_null()
		{
			libvarlist = (*libvarlist).next;
			LibVarDeAlloc(v);
			v = libvarlist;
		} //end for
		libvarlist = ptr::null_mut();
	}
} //end of the function LibVarDeAllocAll
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarGet(var_name: *const c_char) -> *mut libvar_t
{
	unsafe {
		let mut v: *mut libvar_t;

		v = libvarlist;
		while !v.is_null()
		{
			if Q_stricmp((*v).name, var_name) == 0
			{
				return v;
			} //end if
			v = (*v).next;
		} //end for
		return ptr::null_mut();
	}
} //end of the function LibVarGet
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarGetString(var_name: *const c_char) -> *const c_char
{
	unsafe {
		let v: *mut libvar_t;

		v = LibVarGet(var_name);
		if !v.is_null()
		{
			return (*v).string;
		} //end if
		else
		{
			return b"\0".as_ptr() as *const c_char;
		} //end else
	}
} //end of the function LibVarGetString
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarGetValue(var_name: *const c_char) -> f32
{
	unsafe {
		let v: *mut libvar_t;

		v = LibVarGet(var_name);
		if !v.is_null()
		{
			return (*v).value;
		} //end if
		else
		{
			return 0.0;
		} //end else
	}
} //end of the function LibVarGetValue
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVar(var_name: *const c_char, value: *const c_char) -> *mut libvar_t
{
	unsafe {
		let mut v: *mut libvar_t;
		v = LibVarGet(var_name);
		if !v.is_null() { return v; }
		//create new variable
		v = LibVarAlloc(var_name);
		//variable string
		let val_len = strlen(value);
		(*v).string = GetMemory(val_len + 1) as *mut c_char;
		strcpy((*v).string, value);
		//the value
		(*v).value = LibVarStringValue((*v).string);
		//variable is modified
		(*v).modified = QTRUE;
		//
		return v;
	}
} //end of the function LibVar
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarString(var_name: *const c_char, value: *const c_char) -> *const c_char
{
	unsafe {
		let v: *mut libvar_t;

		v = LibVar(var_name, value);
		return (*v).string;
	}
} //end of the function LibVarString
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarValue(var_name: *const c_char, value: *const c_char) -> f32
{
	unsafe {
		let v: *mut libvar_t;

		v = LibVar(var_name, value);
		return (*v).value;
	}
} //end of the function LibVarValue
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarSet(var_name: *const c_char, value: *const c_char)
{
	unsafe {
		let mut v: *mut libvar_t;

		v = LibVarGet(var_name);
		if !v.is_null()
		{
			FreeMemory((*v).string as *mut c_void);
		} //end if
		else
		{
			v = LibVarAlloc(var_name);
		} //end else
		//variable string
		let val_len = strlen(value);
		(*v).string = GetMemory(val_len + 1) as *mut c_char;
		strcpy((*v).string, value);
		//the value
		(*v).value = LibVarStringValue((*v).string);
		//variable is modified
		(*v).modified = QTRUE;
	}
} //end of the function LibVarSet
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarChanged(var_name: *const c_char) -> qboolean
{
	unsafe {
		let v: *mut libvar_t;

		v = LibVarGet(var_name);
		if !v.is_null()
		{
			return (*v).modified;
		} //end if
		else
		{
			return QFALSE;
		} //end else
	}
} //end of the function LibVarChanged
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub fn LibVarSetNotModified(var_name: *const c_char)
{
	unsafe {
		let v: *mut libvar_t;

		v = LibVarGet(var_name);
		if !v.is_null()
		{
			(*v).modified = QFALSE;
		} //end if
	}
} //end of the function LibVarSetNotModified
