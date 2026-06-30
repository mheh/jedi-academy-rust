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

// leave this as first line for PCH reasons...
//
use crate::code::server::exe_headers_h::*;

use crate::code::jpeg_6::jinclude_h::*;
use crate::code::jpeg_6::jpeglib_h::*;
use crate::code::jpeg_6::jpegint_h::*;
use crate::code::jpeg_6::jmorecfg_h::*;

use core::ffi::c_int;

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
        ((*(*cinfo).mem).free_pool)(cinfo, pool);
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
        ((*(*cinfo).mem).self_destruct)(cinfo);
    }
    (*cinfo).mem = core::ptr::null_mut();	/* be safe if jpeg_destroy is called twice */
    (*cinfo).global_state = 0;	/* mark it destroyed */
}


/*
 * Convenience routines for allocating quantization and Huffman tables.
 * (Would jutils.c be a more reasonable place to put these?)
 */

pub unsafe fn jpeg_alloc_quant_table(cinfo: j_common_ptr) -> *mut JQUANT_TBL {
    let tbl: *mut JQUANT_TBL;

    tbl = ((*(*cinfo).mem).alloc_small)(cinfo, JPOOL_PERMANENT, core::mem::size_of::<JQUANT_TBL>()) as *mut JQUANT_TBL;
    (*tbl).sent_table = FALSE;	/* make sure this is false in any new table */
    return tbl;
}


pub unsafe fn jpeg_alloc_huff_table(cinfo: j_common_ptr) -> *mut JHUFF_TBL {
    let tbl: *mut JHUFF_TBL;

    tbl = ((*(*cinfo).mem).alloc_small)(cinfo, JPOOL_PERMANENT, core::mem::size_of::<JHUFF_TBL>()) as *mut JHUFF_TBL;
    (*tbl).sent_table = FALSE;	/* make sure this is false in any new table */
    return tbl;
}
