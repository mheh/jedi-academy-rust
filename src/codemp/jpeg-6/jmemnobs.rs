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

use core::ffi::{c_int, c_void};

/* Forward declarations of types needed for structural coherence */
#[repr(C)]
pub struct j_common_struct {
    _opaque: [u8; 0],
}
pub type j_common_ptr = *mut j_common_struct;

#[repr(C)]
pub struct backing_store_struct {
    _opaque: [u8; 0],
}
pub type backing_store_ptr = *mut backing_store_struct;

/* Error code constant (stub) */
const JERR_NO_BACKING_STORE: c_int = 1;

/* External memory allocation functions from the engine */
extern "C" {
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
}

const TAG_TEMP_WORKSPACE: c_int = 0;
const qfalse: c_int = 0;

/*
 * Memory allocation and ri.Freeing are controlled by the regular library
 * routines Z_Malloc() and Z_Free().
 */

#[no_mangle]
pub extern "C" fn jpeg_get_small(cinfo: j_common_ptr, sizeofobject: usize) -> *mut c_void {
    unsafe { Z_Malloc(sizeofobject, TAG_TEMP_WORKSPACE, qfalse) }
}

#[no_mangle]
pub extern "C" fn jpeg_free_small(cinfo: j_common_ptr, object: *mut c_void, sizeofobject: usize) {
    unsafe { Z_Free(object); }
}


/*
 * "Large" objects are treated the same as "small" ones.
 * NB: although we include FAR keywords in the routine declarations,
 * this file won't actually work in 80x86 small/medium model; at least,
 * you probably won't be able to process useful-size images in only 64KB.
 */

#[no_mangle]
pub extern "C" fn jpeg_get_large(cinfo: j_common_ptr, sizeofobject: usize) -> *mut c_void {
    unsafe { Z_Malloc(sizeofobject, TAG_TEMP_WORKSPACE, qfalse) }
}

#[no_mangle]
pub extern "C" fn jpeg_free_large(cinfo: j_common_ptr, object: *mut c_void, sizeofobject: usize) {
    unsafe { Z_Free(object); }
}


/*
 * This routine computes the total memory space available for allocation.
 * Here we always say, "we got all you want bud!"
 */

#[no_mangle]
pub extern "C" fn jpeg_mem_available(cinfo: j_common_ptr, min_bytes_needed: i32,
                                     max_bytes_needed: i32, already_allocated: i32) -> i32 {
    max_bytes_needed
}


/*
 * Backing store (temporary file) management.
 * Since jpeg_mem_available always promised the moon,
 * this should never be called and we can just error out.
 */

#[no_mangle]
pub extern "C" fn jpeg_open_backing_store(cinfo: j_common_ptr, info: backing_store_ptr,
                                          total_bytes_needed: i32) {
    /* Stub implementation - would call ERREXIT in real JPEG library */
    /* ERREXIT(cinfo, JERR_NO_BACKING_STORE); */
}


/*
 * These routines take care of any system-dependent initialization and
 * cleanup required.  Here, there isn't any.
 */

#[no_mangle]
pub extern "C" fn jpeg_mem_init(cinfo: j_common_ptr) -> i32 {
    0 /* just set max_memory_to_use to 0 */
}

#[no_mangle]
pub extern "C" fn jpeg_mem_term(cinfo: j_common_ptr) {
    /* no work */
}
