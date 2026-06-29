// Anything above this #include will be ignored by the compiler

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use crate::codemp::qcommon::tags_h::TAG_ICARUS5;

// leave these two as standard mallocs for the moment, there's something weird happening in ICARUS...
//

extern "C" {
    pub fn Z_Malloc(
        iSize: c_int,
        eTag: c_int,
        bZeroit: c_int,
        iAlign: c_int,
    ) -> *mut c_void;

    pub fn Z_Free(pMem: *mut c_void);
}

const qfalse: c_int = 0;

pub fn ICARUS_Malloc(iSize: c_int) -> *mut c_void {
    //return gi.Malloc(iSize, TAG_ICARUS);
    //return malloc(iSize);
    unsafe { Z_Malloc(iSize, TAG_ICARUS5, qfalse, 4) }
}

pub fn ICARUS_Free(pMem: *mut c_void) {
    //gi.Free(pMem);
    //free(pMem);
    unsafe { Z_Free(pMem) }
}
