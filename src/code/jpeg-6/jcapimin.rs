/*
 * jcapimin.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains application interface code for the compression half
 * of the JPEG library.  These are the "minimum" API routines that may be
 * needed in either the normal full-compression case or the transcoding-only
 * case.
 *
 * Most of the routines intended to be called directly by an application
 * are in this file or in jcapistd.c.  But also see jcparam.c for
 * parameter-setup helper routines, jcomapi.c for routines shared by
 * compression and decompression, and jctrans.c for the transcoding case.
 */

// leave this as first line for PCH reasons...
//
use crate::code::server::exe_headers_h::*;
use crate::code::jpeg_6::jinclude_h::*;
use crate::code::jpeg_6::jpeglib_h::*;
// JPEG_INTERNALS was defined before jpeglib.h, causing jpegint.h to be pulled in;
// import it explicitly here.
use crate::code::jpeg_6::jpegint_h::*;

use core::ffi::{c_int, c_uint, c_long};

/*
 * Initialization of a JPEG compression object.
 * The error manager must already be set up (in case memory manager fails).
 */

pub unsafe fn jpeg_create_compress(cinfo: j_compress_ptr) {
    let mut i: c_int;

    /* For debugging purposes, zero the whole master structure.
     * But error manager pointer is already there, so save and restore it.
     */
    {
        let err: *mut jpeg_error_mgr = (*cinfo).err;
        MEMZERO!(cinfo, SIZEOF!(jpeg_compress_struct));
        (*cinfo).err = err;
    }
    (*cinfo).is_decompressor = FALSE;

    /* Initialize a memory manager instance for this object */
    jinit_memory_mgr(cinfo as j_common_ptr);

    /* Zero out pointers to permanent structures. */
    (*cinfo).progress = core::ptr::null_mut();
    (*cinfo).dest = core::ptr::null_mut();

    (*cinfo).comp_info = core::ptr::null_mut();

    i = 0;
    while i < NUM_QUANT_TBLS {
        (*cinfo).quant_tbl_ptrs[i as usize] = core::ptr::null_mut();
        i += 1;
    }

    i = 0;
    while i < NUM_HUFF_TBLS {
        (*cinfo).dc_huff_tbl_ptrs[i as usize] = core::ptr::null_mut();
        (*cinfo).ac_huff_tbl_ptrs[i as usize] = core::ptr::null_mut();
        i += 1;
    }

    (*cinfo).input_gamma = 1.0f64; /* in case application forgets */

    /* OK, I'm ready */
    (*cinfo).global_state = CSTATE_START;
}


/*
 * Destruction of a JPEG compression object
 */

pub unsafe fn jpeg_destroy_compress(cinfo: j_compress_ptr) {
    jpeg_destroy(cinfo as j_common_ptr); /* use common routine */
}


/*
 * Abort processing of a JPEG compression operation,
 * but don't destroy the object itself.
 */

pub unsafe fn jpeg_abort_compress(cinfo: j_compress_ptr) {
    jpeg_abort(cinfo as j_common_ptr); /* use common routine */
}


/*
 * Forcibly suppress or un-suppress all quantization and Huffman tables.
 * Marks all currently defined tables as already written (if suppress)
 * or not written (if !suppress).  This will control whether they get emitted
 * by a subsequent jpeg_start_compress call.
 *
 * This routine is exported for use by applications that want to produce
 * abbreviated JPEG datastreams.  It logically belongs in jcparam.c, but
 * since it is called by jpeg_start_compress, we put it here --- otherwise
 * jcparam.o would be linked whether the application used it or not.
 */

pub unsafe fn jpeg_suppress_tables(cinfo: j_compress_ptr, suppress: boolean) {
    let mut i: c_int;
    let mut qtbl: *mut JQUANT_TBL;
    let mut htbl: *mut JHUFF_TBL;

    i = 0;
    while i < NUM_QUANT_TBLS {
        qtbl = (*cinfo).quant_tbl_ptrs[i as usize];
        if !qtbl.is_null() {
            (*qtbl).sent_table = suppress;
        }
        i += 1;
    }

    i = 0;
    while i < NUM_HUFF_TBLS {
        htbl = (*cinfo).dc_huff_tbl_ptrs[i as usize];
        if !htbl.is_null() {
            (*htbl).sent_table = suppress;
        }
        htbl = (*cinfo).ac_huff_tbl_ptrs[i as usize];
        if !htbl.is_null() {
            (*htbl).sent_table = suppress;
        }
        i += 1;
    }
}


/*
 * Finish JPEG compression.
 *
 * If a multipass operating mode was selected, this may do a great deal of
 * work including most of the actual output.
 */

pub unsafe fn jpeg_finish_compress(cinfo: j_compress_ptr) {
    let mut iMCU_row: JDIMENSION;

    if (*cinfo).global_state == CSTATE_SCANNING ||
       (*cinfo).global_state == CSTATE_RAW_OK {
        /* Terminate first pass */
        if (*cinfo).next_scanline < (*cinfo).image_height {
            ERREXIT!(cinfo, JERR_TOO_LITTLE_DATA);
        }
        (*(*cinfo).master).finish_pass.unwrap()(cinfo);
    } else if (*cinfo).global_state != CSTATE_WRCOEFS {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Perform any remaining passes */
    while (*(*cinfo).master).is_last_pass == FALSE {
        (*(*cinfo).master).prepare_for_pass.unwrap()(cinfo);
        iMCU_row = 0;
        while iMCU_row < (*cinfo).total_iMCU_rows {
            if !(*cinfo).progress.is_null() {
                (*(*cinfo).progress).pass_counter = iMCU_row as c_long;
                (*(*cinfo).progress).pass_limit = (*cinfo).total_iMCU_rows as c_long;
                (*(*cinfo).progress).progress_monitor.unwrap()(cinfo as j_common_ptr);
            }
            /* We bypass the main controller and invoke coef controller directly;
             * all work is being done from the coefficient buffer.
             */
            if (*(*cinfo).coef).compress_data.unwrap()(cinfo, core::ptr::null_mut() as JSAMPIMAGE) == FALSE {
                ERREXIT!(cinfo, JERR_CANT_SUSPEND);
            }
            iMCU_row += 1;
        }
        (*(*cinfo).master).finish_pass.unwrap()(cinfo);
    }
    /* Write EOI, do final cleanup */
    (*(*cinfo).marker).write_file_trailer.unwrap()(cinfo);
    (*(*cinfo).dest).term_destination.unwrap()(cinfo);
    /* We can use jpeg_abort to release memory and reset global_state */
    jpeg_abort(cinfo as j_common_ptr);
}


/*
 * Write a special marker.
 * This is only recommended for writing COM or APPn markers.
 * Must be called after jpeg_start_compress() and before
 * first call to jpeg_write_scanlines() or jpeg_write_raw_data().
 */

pub unsafe fn jpeg_write_marker(cinfo: j_compress_ptr, marker: c_int,
                                dataptr: *const JOCTET, datalen: c_uint) {
    if (*cinfo).next_scanline != 0 ||
       ((*cinfo).global_state != CSTATE_SCANNING &&
        (*cinfo).global_state != CSTATE_RAW_OK &&
        (*cinfo).global_state != CSTATE_WRCOEFS) {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }

    (*(*cinfo).marker).write_any_marker.unwrap()(cinfo, marker, dataptr, datalen);
}


/*
 * Alternate compression function: just write an abbreviated table file.
 * Before calling this, all parameters and a data destination must be set up.
 *
 * To produce a pair of files containing abbreviated tables and abbreviated
 * image data, one would proceed as follows:
 *
 *		initialize JPEG object
 *		set JPEG parameters
 *		set destination to table file
 *		jpeg_write_tables(cinfo);
 *		set destination to image file
 *		jpeg_start_compress(cinfo, FALSE);
 *		write data...
 *		jpeg_finish_compress(cinfo);
 *
 * jpeg_write_tables has the side effect of marking all tables written
 * (same as jpeg_suppress_tables(..., TRUE)).  Thus a subsequent start_compress
 * will not re-emit the tables unless it is passed write_all_tables=TRUE.
 */

pub unsafe fn jpeg_write_tables(cinfo: j_compress_ptr) {
    if (*cinfo).global_state != CSTATE_START {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }

    /* (Re)initialize error mgr and destination modules */
    (*(*cinfo).err).reset_error_mgr.unwrap()(cinfo as j_common_ptr);
    (*(*cinfo).dest).init_destination.unwrap()(cinfo);
    /* Initialize the marker writer ... bit of a crock to do it here. */
    jinit_marker_writer(cinfo);
    /* Write them tables! */
    (*(*cinfo).marker).write_tables_only.unwrap()(cinfo);
    /* And clean up. */
    (*(*cinfo).dest).term_destination.unwrap()(cinfo);
    /* We can use jpeg_abort to release memory. */
    jpeg_abort(cinfo as j_common_ptr);
}
