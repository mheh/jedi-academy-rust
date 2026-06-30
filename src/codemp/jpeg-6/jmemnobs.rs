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
 * can be obtained from Z_Malloc().
 * This is very portable in the sense that it'll compile on almost anything,
 * but you'd better have lots of main memory (or virtual memory) if you want
 * to process big images.
 * Note that the max_memory_to_use option is ignored by this implementation.
 */
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]

// Anything above this #include will be ignored by the compiler
use crate::codemp::qcommon::exe_headers_h::*;

// #define JPEG_INTERNALS — preprocessor guard to expose internal JPEG types; types come via glob imports
use crate::codemp::jpeg_6::jinclude_h::*;
use crate::codemp::jpeg_6::jpeglib_h::*;
use crate::codemp::jpeg_6::jmemsys_h::*; /* import the system-dependent declarations */

use crate::codemp::renderer::tr_local_h::*;

/*
 * Memory allocation and ri.Freeing are controlled by the regular library
 * routines Z_Malloc() and Z_Free().
 */

#[no_mangle]
pub unsafe extern "C" fn jpeg_get_small(
    cinfo: j_common_ptr,
    sizeofobject: usize,
) -> *mut core::ffi::c_void {
    Z_Malloc(sizeofobject as core::ffi::c_int, TAG_TEMP_WORKSPACE, qfalse)
        as *mut core::ffi::c_void
}

#[no_mangle]
pub unsafe extern "C" fn jpeg_free_small(
    cinfo: j_common_ptr,
    object: *mut core::ffi::c_void,
    sizeofobject: usize,
) {
    Z_Free(object);
}


/*
 * "Large" objects are treated the same as "small" ones.
 * NB: although we include FAR keywords in the routine declarations,
 * this file won't actually work in 80x86 small/medium model; at least,
 * you probably won't be able to process useful-size images in only 64KB.
 */

#[no_mangle]
pub unsafe extern "C" fn jpeg_get_large(
    cinfo: j_common_ptr,
    sizeofobject: usize,
) -> *mut core::ffi::c_void {
    /* FAR expands to nothing on non-segmented platforms */
    Z_Malloc(sizeofobject as core::ffi::c_int, TAG_TEMP_WORKSPACE, qfalse)
        as *mut core::ffi::c_void
}

#[no_mangle]
pub unsafe extern "C" fn jpeg_free_large(
    cinfo: j_common_ptr,
    object: *mut core::ffi::c_void,
    sizeofobject: usize,
) {
    /* FAR expands to nothing on non-segmented platforms */
    Z_Free(object);
}


/*
 * This routine computes the total memory space available for allocation.
 * Here we always say, "we got all you want bud!"
 */

#[no_mangle]
pub unsafe extern "C" fn jpeg_mem_available(
    cinfo: j_common_ptr,
    min_bytes_needed: core::ffi::c_long,
    max_bytes_needed: core::ffi::c_long,
    already_allocated: core::ffi::c_long,
) -> core::ffi::c_long {
    max_bytes_needed
}


/*
 * Backing store (temporary file) management.
 * Since jpeg_mem_available always promised the moon,
 * this should never be called and we can just error out.
 */

#[no_mangle]
pub unsafe extern "C" fn jpeg_open_backing_store(
    cinfo: j_common_ptr,
    info: backing_store_ptr,
    total_bytes_needed: core::ffi::c_long,
) {
    ERREXIT!(cinfo, JERR_NO_BACKING_STORE);
}


/*
 * These routines take care of any system-dependent initialization and
 * cleanup required.  Here, there isn't any.
 */

#[no_mangle]
pub unsafe extern "C" fn jpeg_mem_init(cinfo: j_common_ptr) -> core::ffi::c_long {
    0 /* just set max_memory_to_use to 0 */
}

#[no_mangle]
pub unsafe extern "C" fn jpeg_mem_term(cinfo: j_common_ptr) {
    /* no work */
}
