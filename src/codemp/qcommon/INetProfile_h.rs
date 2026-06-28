//! `INetProfile.h` — optional network profile interface.

#![allow(non_snake_case)]
#![allow(unexpected_cfgs)]

use core::ffi::{c_char, c_int, c_void};

#[cfg(feature = "donetprofile")]
pub unsafe trait INetProfile {
    unsafe fn Reset(this: *mut Self);
    unsafe fn AddField(this: *mut Self, fieldName: *mut c_char, sizeBytes: c_int);
    unsafe fn IncTime(this: *mut Self, msec: c_int);
    unsafe fn ShowTotals(this: *mut Self);
}

#[cfg(feature = "donetprofile")]
unsafe extern "C" {
    // C++ returns `INetProfile &`; keep it opaque until the profiling implementation is ported.
    pub fn ClReadProf() -> *mut c_void;
    pub fn ClSendProf() -> *mut c_void;
}
