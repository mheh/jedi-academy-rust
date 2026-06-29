#![allow(non_snake_case)]

use core::ffi::{c_char, c_void, c_int};

// #include "common_headers.h"

// //#include "ff_utils.h"

// LOCAL STUB: cvar_t structure (minimal definition for compatibility)
#[repr(C)]
pub struct cvar_t {
    integer: c_int,
}

extern "C" {
    static mut ff_developer: *mut cvar_t;
    fn strlen(s: *const c_char) -> usize;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn FS_ReadFile(filename: *const c_char, buffer: *mut *mut c_void) -> c_int;
}

#[cfg(feature = "FF_PRINT")]
extern "C" {
    fn Com_Printf(fmt: *const c_char, ...) -> c_int;
}

//
// Didn't know about strrchr. This is slightly different anyway.
//
pub extern "C" fn _rcpos(string: *const c_char, c: c_char, pos: c_int) -> c_int {
    let length = unsafe { strlen(string) } as c_int;
    let length = if pos >= 0 && pos < length { pos } else { length };
    for i in (0..=length - 1).rev() {
        unsafe {
            if *string.add(i as usize) == c {
                return i;
            }
        }
    }
    -1
}

pub extern "C" fn LoadFile(filename: *const c_char) -> *mut c_void {
    let mut buffer: *mut c_void = core::ptr::null_mut();

    let length = unsafe { FS_ReadFile(filename, &mut buffer) };

    if length != -1 { buffer } else { core::ptr::null_mut() }
}

pub extern "C" fn UncommonDirectory(target: *const c_char, comp: *const c_char) -> *const c_char {
    let mut pos = target;

    let mut i = 0;
    loop {
        unsafe {
            if *target.add(i) == 0 || *comp.add(i) == 0 || *target.add(i) != *comp.add(i) {
                break;
            }
            if *target.add(i) as u8 == b'/' {
                pos = target.add(i + 1);
            }
            i += 1;
        }
    }

    unsafe {
        if *comp.add(i) == 0 && *target.add(i) as u8 == b'/' {
            pos = target.add(i + 1);
        } else if *target.add(i) == 0 && *comp.add(i) as u8 == b'/' {
            pos = target.add(i);
        }
    }

    pos
}

////-------
/// RightOf
//-----------
//
//
//	Parameters:
//
//	Returns:
//
pub extern "C" fn RightOf(str: *const c_char, str2: *const c_char) -> *const c_char {
    if str.is_null() || str2.is_null() {
        return core::ptr::null();
    }

    let mut s = str;
    let len1 = unsafe { strlen(str) } as c_int;
    let len2 = unsafe { strlen(str2) } as c_int;

    if (len2 != 0) && (len1 >= len2) {
        s = unsafe { strstr(str, str2) };
        if !s.is_null() {
            unsafe {
                if ((s == str) && (*s.add(len2 as usize) as u8 == b'/'))
                    || ((*s.offset(-1) as u8 == b'/') && (*s.add(len2 as usize) as u8 == b'/'))
                {
                    s = s.add(len2 as usize + 1);
                }
            }
        }
    }

    if !s.is_null() { s } else { str }
}

#[cfg(feature = "FF_PRINT")]
pub extern "C" fn ConsoleParseError(
    message: *const c_char,
    line: *const c_char,
    pos: c_int,
) {
    unsafe {
        if !ff_developer.is_null() && (*ff_developer).integer != 0 {
            Com_Printf(
                b"Parse error: %s\n%s\n%*c\n\0".as_ptr() as *const c_char,
                message,
                line,
                pos + 1,
                b'^' as c_int,
            );
        }
    }
}

#[cfg(feature = "FF_PRINT")]
pub extern "C" fn FS_VerifyName(
    _src: *const c_char,
    _name: *const c_char,
    _out: *mut c_char,
    _maxlen: c_int,
) -> c_int {
    1 // qtrue
}
