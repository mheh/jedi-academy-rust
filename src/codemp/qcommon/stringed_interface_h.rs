//! `stringed_interface.h` — StringEd file access interface declarations.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_uchar};

#[repr(C)]
pub struct std_string {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn SE_LoadFileData(psFileName: *const c_char, piLoadedLength: *mut c_int) -> *mut c_uchar;
    pub fn SE_FreeFileDataAfterLoad(psLoadedFile: *mut c_uchar);
    pub fn SE_BuildFileList(psStartDir: *const c_char, strResults: *mut std_string) -> c_int;
}
