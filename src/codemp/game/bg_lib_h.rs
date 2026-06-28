//! `bg_lib.h` -- standard C library replacement routines used by code compiled
//! for the virtual machine.
//!
//! This file is NOT included on native builds.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_double, c_int, c_uint, c_void};

pub type size_t = c_int;

pub type va_list = *mut c_char;

pub const fn _INTSIZEOF(n: usize) -> usize {
    (n + core::mem::size_of::<c_int>() - 1) & !(core::mem::size_of::<c_int>() - 1)
}

// `va_start`, `va_arg`, and `va_end` were C preprocessor macros in the original.
// These helpers preserve the pointer arithmetic shape for ports that need to
// spell the VM va_list walk explicitly.
pub unsafe fn va_arg_ptr(ap: *mut va_list, size: usize) -> *mut c_void {
    unsafe {
        *ap = (*ap).add(_INTSIZEOF(size));
        (*ap).sub(_INTSIZEOF(size)) as *mut c_void
    }
}

pub unsafe fn va_end(ap: *mut va_list) {
    unsafe {
        *ap = core::ptr::null_mut();
    }
}

pub const CHAR_BIT: c_int = 8; // number of bits in a char
pub const SCHAR_MIN: c_int = -128; // minimum signed char value
pub const SCHAR_MAX: c_int = 127; // maximum signed char value
pub const UCHAR_MAX: c_int = 0xff; // maximum unsigned char value

pub const SHRT_MIN: c_int = -32768; // minimum (signed) short value
pub const SHRT_MAX: c_int = 32767; // maximum (signed) short value
pub const USHRT_MAX: c_int = 0xffff; // maximum unsigned short value
pub const INT_MIN: c_int = -2147483647 - 1; // minimum (signed) int value
pub const INT_MAX: c_int = 2147483647; // maximum (signed) int value
pub const UINT_MAX: c_uint = 0xffffffff; // maximum unsigned int value
pub const LONG_MIN: c_int = -2147483647 - 1; // minimum (signed) long value
pub const LONG_MAX: c_int = 2147483647; // maximum (signed) long value
pub const ULONG_MAX: c_uint = 0xffffffff; // maximum unsigned long value

// Misc functions
pub type cmp_t = unsafe extern "C" fn(*const c_void, *const c_void) -> c_int;

unsafe extern "C" {
    pub fn qsort(a: *mut c_void, n: size_t, es: size_t, cmp: cmp_t);
    pub fn srand(seed: c_uint);
    pub fn rand() -> c_int;

    // String functions
    pub fn strlen(string: *const c_char) -> size_t;
    pub fn strcat(strDestination: *mut c_char, strSource: *const c_char) -> *mut c_char;
    pub fn strcpy(strDestination: *mut c_char, strSource: *const c_char) -> *mut c_char;
    pub fn strcmp(string1: *const c_char, string2: *const c_char) -> c_int;
    pub fn strchr(string: *const c_char, c: c_int) -> *mut c_char;
    pub fn strstr(string: *const c_char, strCharSet: *const c_char) -> *mut c_char;
    pub fn strncpy(
        strDest: *mut c_char,
        strSource: *const c_char,
        count: size_t,
    ) -> *mut c_char;
    pub fn tolower(c: c_int) -> c_int;
    pub fn toupper(c: c_int) -> c_int;

    pub fn atof(string: *const c_char) -> c_double;
    pub fn _atof(stringPtr: *mut *const c_char) -> c_double;
    pub fn atoi(string: *const c_char) -> c_int;
    pub fn _atoi(stringPtr: *mut *const c_char) -> c_int;

    pub fn vsprintf(buffer: *mut c_char, fmt: *const c_char, argptr: va_list) -> c_int;
    pub fn sscanf(buffer: *const c_char, fmt: *const c_char, ...) -> c_int;

    // Memory functions
    pub fn memmove(dest: *mut c_void, src: *const c_void, count: size_t) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, count: size_t) -> *mut c_void;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, count: size_t) -> *mut c_void;

    // Math functions
    pub fn ceil(x: c_double) -> c_double;
    pub fn floor(x: c_double) -> c_double;
    pub fn sqrt(x: c_double) -> c_double;
    pub fn sin(x: c_double) -> c_double;
    pub fn cos(x: c_double) -> c_double;
    pub fn atan2(y: c_double, x: c_double) -> c_double;
    pub fn tan(x: c_double) -> c_double;
    pub fn abs(n: c_int) -> c_int;
    pub fn fabs(x: c_double) -> c_double;
    pub fn acos(x: c_double) -> c_double;
}
