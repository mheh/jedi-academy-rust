/*
 * jcomapi.c
 *
 * Copyright (C) 1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains application interface routines that are used for both
 * compression and decompression.
 */
// Anything above this include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #define JPEG_INTERNALS
// #include "jinclude.h"
// #include "jpeglib.h"

use core::ffi::c_int;

// ============================================================================
// Stubs for JPEG-6 types and structures needed for structural coherence
// ============================================================================

pub type JDIMENSION = u32;
pub type boolean = u8;

// Forward declarations
#[repr(C)]
pub struct j_common_struct {
    pub err: *mut core::ffi::c_void,           /* Error handler module */
    pub mem: *mut jpeg_memory_mgr,             /* Memory manager module */
    pub progress: *mut core::ffi::c_void,      /* Progress monitor, or NULL if none */
    pub is_decompressor: boolean,              /* so common code can tell which is which */
    pub global_state: c_int,                   /* for checking call sequence validity */
}

pub type j_common_ptr = *mut j_common_struct;

/* DCT coefficient quantization tables. */
#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; 64],  /* quantization step for each coefficient */
    pub sent_table: boolean,  /* TRUE when table has been output */
}

/* Huffman coding tables. */
#[repr(C)]
pub struct JHUFF_TBL {
    pub bits: [u8; 17],       /* bits[k] = # of symbols with codes of length k bits; bits[0] is unused */
    pub huffval: [u8; 256],   /* The symbols, in order of incr code length */
    pub sent_table: boolean,  /* TRUE when table has been output */
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut core::ffi::c_void>,
    pub alloc_large: Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut core::ffi::c_void>,
    pub alloc_sarray: Option<unsafe extern "C" fn(j_common_ptr, c_int, JDIMENSION, JDIMENSION) -> *mut u8>,
    pub alloc_barray: Option<unsafe extern "C" fn(j_common_ptr, c_int, JDIMENSION, JDIMENSION) -> *mut core::ffi::c_void>,
    pub request_virt_sarray: Option<unsafe extern "C" fn(j_common_ptr, c_int, boolean, JDIMENSION, JDIMENSION, JDIMENSION) -> *mut core::ffi::c_void>,
    pub request_virt_barray: Option<unsafe extern "C" fn(j_common_ptr, c_int, boolean, JDIMENSION, JDIMENSION, JDIMENSION) -> *mut core::ffi::c_void>,
    pub realize_virt_arrays: Option<unsafe extern "C" fn(j_common_ptr)>,
    pub access_virt_sarray: Option<unsafe extern "C" fn(j_common_ptr, *mut core::ffi::c_void, JDIMENSION, JDIMENSION, boolean) -> *mut u8>,
    pub access_virt_barray: Option<unsafe extern "C" fn(j_common_ptr, *mut core::ffi::c_void, JDIMENSION, JDIMENSION, boolean) -> *mut core::ffi::c_void>,
    pub free_pool: Option<unsafe extern "C" fn(j_common_ptr, c_int)>,
    pub self_destruct: Option<unsafe extern "C" fn(j_common_ptr)>,
}

// Constants
const JPOOL_PERMANENT: c_int = 0;  /* lasts until master record is destroyed */
const JPOOL_NUMPOOLS: c_int = 2;
const CSTATE_START: c_int = 100;   /* after create_compress */
const DSTATE_START: c_int = 200;   /* after create_decompress */

// Macro equivalent: SIZEOF(object) -> sizeof(object)
#[inline]
fn SIZEOF<T>() -> usize {
    core::mem::size_of::<T>()
}

#[allow(non_snake_case)]
const FALSE: boolean = 0;

/*
 * Abort processing of a JPEG compression or decompression operation,
 * but don't destroy the object itself.
 *
 * For this, we merely clean up all the nonpermanent memory pools.
 * Note that temp files (virtual arrays) are not allowed to belong to
 * the permanent pool, so we will be able to close all temp files here.
 * Closing a data source or destination, if necessary, is the application's
 * responsibility.
 */

pub unsafe fn jpeg_abort(cinfo: j_common_ptr) {
    let mut pool: c_int;

    /* Releasing pools in reverse order might help avoid fragmentation
     * with some (brain-damaged) malloc libraries.
     */
    pool = JPOOL_NUMPOOLS - 1;
    while pool > JPOOL_PERMANENT {
        if let Some(free_pool_fn) = (*(*cinfo).mem).free_pool {
            free_pool_fn(cinfo, pool);
        }
        pool -= 1;
    }

    /* Reset overall state for possible reuse of object */
    (*cinfo).global_state = if (*cinfo).is_decompressor != 0 { DSTATE_START } else { CSTATE_START };
}


/*
 * Destruction of a JPEG object.
 *
 * Everything gets deallocated except the master jpeg_compress_struct itself
 * and the error manager struct.  Both of these are supplied by the application
 * and must be freed, if necessary, by the application.  (Often they are on
 * the stack and so don't need to be freed anyway.)
 * Closing a data source or destination, if necessary, is the application's
 * responsibility.
 */

pub unsafe fn jpeg_destroy(cinfo: j_common_ptr) {
    /* We need only tell the memory manager to release everything. */
    /* NB: mem pointer is NULL if memory mgr failed to initialize. */
    if !(*cinfo).mem.is_null() {
        if let Some(self_destruct_fn) = (*(*cinfo).mem).self_destruct {
            self_destruct_fn(cinfo);
        }
    }
    (*cinfo).mem = core::ptr::null_mut();  /* be safe if jpeg_destroy is called twice */
    (*cinfo).global_state = 0;             /* mark it destroyed */
}


/*
 * Convenience routines for allocating quantization and Huffman tables.
 * (Would jutils.c be a more reasonable place to put these?)
 */

pub unsafe fn jpeg_alloc_quant_table(cinfo: j_common_ptr) -> *mut JQUANT_TBL {
    let tbl: *mut JQUANT_TBL;

    tbl = (*(*cinfo).mem).alloc_small
        .map(|alloc_small_fn| {
            alloc_small_fn(cinfo, JPOOL_PERMANENT, SIZEOF::<JQUANT_TBL>()) as *mut JQUANT_TBL
        })
        .unwrap_or(core::ptr::null_mut());

    if !tbl.is_null() {
        (*tbl).sent_table = FALSE;  /* make sure this is false in any new table */
    }
    tbl
}


pub unsafe fn jpeg_alloc_huff_table(cinfo: j_common_ptr) -> *mut JHUFF_TBL {
    let tbl: *mut JHUFF_TBL;

    tbl = (*(*cinfo).mem).alloc_small
        .map(|alloc_small_fn| {
            alloc_small_fn(cinfo, JPOOL_PERMANENT, SIZEOF::<JHUFF_TBL>()) as *mut JHUFF_TBL
        })
        .unwrap_or(core::ptr::null_mut());

    if !tbl.is_null() {
        (*tbl).sent_table = FALSE;  /* make sure this is false in any new table */
    }
    tbl
}
