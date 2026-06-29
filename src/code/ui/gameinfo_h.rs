#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use crate::code::game::q_shared_h::{fileHandle_t, fsMode_t};

#[repr(C)]
pub struct gameinfo_import_t {
	pub FS_FOpenFile: unsafe extern "C" fn(*const c_char, *mut fileHandle_t, fsMode_t) -> c_int,
	pub FS_Read: unsafe extern "C" fn(*mut c_void, c_int, fileHandle_t) -> c_int,
	pub FS_FCloseFile: unsafe extern "C" fn(fileHandle_t) -> (),
	pub Cvar_Set: unsafe extern "C" fn(*const c_char, *const c_char) -> (),
	pub Cvar_VariableStringBuffer: unsafe extern "C" fn(*const c_char, *mut c_char, c_int) -> (),
	pub Cvar_Create: unsafe extern "C" fn(*const c_char, *const c_char, c_int) -> (),
	pub FS_ReadFile: unsafe extern "C" fn(*const c_char, *mut *mut c_void) -> c_int,
	pub FS_FreeFile: unsafe extern "C" fn(*mut c_void) -> (),
	pub Printf: unsafe extern "C" fn(*const c_char, ...) -> (),
}

extern "C" {
	pub fn GI_Init(import: *mut gameinfo_import_t);
}
