//! `platform.h` — simple platform API typedef dispatch.

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

use core::ffi::{c_char, c_void};

#[cfg(target_os = "linux")]
pub type LPCTSTR = *const c_char;
#[cfg(target_os = "linux")]
pub type LPCSTR = *const c_char;
#[cfg(target_os = "linux")]
pub type DWORD = c_ulong;
#[cfg(target_os = "linux")]
pub type UINT = c_uint;
#[cfg(target_os = "linux")]
pub type HANDLE = *mut c_void;
#[cfg(target_os = "linux")]
pub type COLORREF = DWORD;
#[cfg(target_os = "linux")]
pub type BYTE = c_uchar;

use core::ffi::{c_uchar, c_uint, c_ulong};
