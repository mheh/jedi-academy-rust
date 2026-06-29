/*
 * jmemnobs.c
 *
 * Copyright (C) 1992-1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file provides a really simple implementation of the system-
 * dependent portion of the JPEG memory manager.  This implementation
 * assumes that no backing-store files are needed: all required space
 * can be obtained from ri.Malloc().
 * This is very portable in the sense that it'll compile on almost anything,
 * but you'd better have lots of main memory (or virtual memory) if you want
 * to process big images.
 * Note that the max_memory_to_use option is ignored by this implementation.
 */

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_void, c_long};

// #define JPEG_INTERNALS
// #include "jinclude.h"
// #include "jpeglib.h"
// #include "jmemsys.h"		/* import the system-dependent declarations */

// #include "../renderer/tr_local.h"

// Type aliases for JPEG library (from jmemsys.h)
type j_common_ptr = *mut c_void;
type backing_store_ptr = *mut c_void;

// External engine functions
extern "C" {
    fn Z_Malloc(size: usize, tag: c_long, clear: c_long) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn ERREXIT(cinfo: j_common_ptr, code: c_long);
}

// Constants
const TAG_TEMP_JPG: c_long = 0;  // Placeholder; actual value from engine headers
const qfalse: c_long = 0;
const JERR_NO_BACKING_STORE: c_long = 21;  // Placeholder; actual value from jinclude.h

/*
 * Memory allocation and ri.Freeing are controlled by the regular library
 * routines ri.Malloc() and ri.Free().
 */

#[no_mangle]
pub extern "C" fn jpeg_get_small(cinfo: j_common_ptr, sizeofobject: usize) -> *mut c_void {
    unsafe { Z_Malloc(sizeofobject, TAG_TEMP_JPG, qfalse) }
}

#[no_mangle]
pub extern "C" fn jpeg_free_small(cinfo: j_common_ptr, object: *mut c_void, _sizeofobject: usize) {
    unsafe {
        Z_Free(object);
    }
}


/*
 * "Large" objects are treated the same as "small" ones.
 * NB: although we include FAR keywords in the routine declarations,
 * this file won't actually work in 80x86 small/medium model; at least,
 * you probably won't be able to process useful-size images in only 64KB.
 */

#[no_mangle]
pub extern "C" fn jpeg_get_large(cinfo: j_common_ptr, sizeofobject: usize) -> *mut c_void {
    unsafe { Z_Malloc(sizeofobject, TAG_TEMP_JPG, qfalse) }
}

#[no_mangle]
pub extern "C" fn jpeg_free_large(cinfo: j_common_ptr, object: *mut c_void, _sizeofobject: usize) {
    unsafe {
        Z_Free(object);
    }
}


/*
 * This routine computes the total memory space available for allocation.
 * Here we always say, "we got all you want bud!"
 */

#[no_mangle]
pub extern "C" fn jpeg_mem_available(cinfo: j_common_ptr, _min_bytes_needed: c_long,
                                     max_bytes_needed: c_long, _already_allocated: c_long) -> c_long {
    max_bytes_needed
}


/*
 * Backing store (temporary file) management.
 * Since jpeg_mem_available always promised the moon,
 * this should never be called and we can just error out.
 */

#[no_mangle]
pub extern "C" fn jpeg_open_backing_store(cinfo: j_common_ptr, _info: backing_store_ptr,
                                          _total_bytes_needed: c_long) {
    unsafe {
        ERREXIT(cinfo, JERR_NO_BACKING_STORE);
    }
}


/*
 * These routines take care of any system-dependent initialization and
 * cleanup required.  Here, there isn't any.
 */

#[no_mangle]
pub extern "C" fn jpeg_mem_init(cinfo: j_common_ptr) -> c_long {
    0			/* just set max_memory_to_use to 0 */
}

#[no_mangle]
pub extern "C" fn jpeg_mem_term(cinfo: j_common_ptr) {
    /* no work */
}
