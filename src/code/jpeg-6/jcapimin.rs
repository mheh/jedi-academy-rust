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
// #include "../server/exe_headers.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};

// Local stubs for JPEG library types and constants.
// Full definitions would come from jpeglib.h and jinclude.h.
// These are declared to maintain structural coherence with the C code.

#[repr(C)]
pub struct jpeg_error_mgr {
    pub reset_error_mgr: Option<unsafe extern "C" fn(*mut c_void)>,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_progress_mgr {
    pub progress_monitor: Option<unsafe extern "C" fn(*mut c_void)>,
    pub pass_counter: c_int,
    pub pass_limit: c_int,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
    pub free_pool: Option<unsafe extern "C" fn(*mut c_void, c_int)>,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_destination_mgr {
    pub init_destination: Option<unsafe extern "C" fn(*mut jpeg_compress_struct)>,
    pub term_destination: Option<unsafe extern "C" fn(*mut jpeg_compress_struct)>,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_marker_writer {
    pub write_any_marker: Option<unsafe extern "C" fn(*mut jpeg_compress_struct, c_int, *const u8, u32)>,
    pub write_file_trailer: Option<unsafe extern "C" fn(*mut jpeg_compress_struct)>,
    pub write_tables_only: Option<unsafe extern "C" fn(*mut jpeg_compress_struct)>,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_c_master_controller {
    pub finish_pass: Option<unsafe extern "C" fn(*mut jpeg_compress_struct)>,
    pub prepare_for_pass: Option<unsafe extern "C" fn(*mut jpeg_compress_struct)>,
    pub is_last_pass: c_int,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_c_coef_controller {
    pub compress_data: Option<unsafe extern "C" fn(*mut jpeg_compress_struct, *mut u8) -> c_int>,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; 64],
    pub sent_table: c_int,
}

#[repr(C)]
pub struct JHUFF_TBL {
    pub bits: [u8; 17],
    pub huffval: [u8; 256],
    pub sent_table: c_int,
}

#[repr(C)]
pub struct jpeg_compress_struct {
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub progress: *mut jpeg_progress_mgr,
    pub is_decompressor: c_int,
    pub global_state: c_int,
    pub next_scanline: u32,
    pub image_height: u32,
    pub total_iMCU_rows: u32,
    pub dest: *mut jpeg_destination_mgr,
    pub comp_info: *mut c_void,
    pub quant_tbl_ptrs: [*mut JQUANT_TBL; 4],
    pub dc_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
    pub ac_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
    pub input_gamma: f64,
    pub marker: *mut jpeg_marker_writer,
    pub master: *mut jpeg_c_master_controller,
    pub coef: *mut jpeg_c_coef_controller,
    _opaque: [u8; 0],
}

pub type j_compress_ptr = *mut jpeg_compress_struct;

// Constants
const NUM_QUANT_TBLS: usize = 4;
const NUM_HUFF_TBLS: usize = 4;
const FALSE: c_int = 0;
const TRUE: c_int = 1;
const CSTATE_START: c_int = 100;
const CSTATE_SCANNING: c_int = 101;
const CSTATE_RAW_OK: c_int = 102;
const CSTATE_WRCOEFS: c_int = 103;

// Error codes (stubs)
const JERR_TOO_LITTLE_DATA: c_int = 51;
const JERR_BAD_STATE: c_int = 203;
const JERR_CANT_SUSPEND: c_int = 52;

// Extern JPEG functions
extern "C" {
    fn jinit_memory_mgr(cinfo: *mut c_void);
    fn jpeg_destroy(cinfo: *mut c_void);
    fn jpeg_abort(cinfo: *mut c_void);
    fn jinit_marker_writer(cinfo: *mut jpeg_compress_struct);

    // Error handling stubs
    fn ERREXIT(cinfo: *mut c_void, code: c_int);
    fn ERREXIT1(cinfo: *mut c_void, code: c_int, param: c_int);
}

// Helper: MEMZERO - zero out memory (matching jinclude.h)
unsafe fn MEMZERO(target: *mut c_void, size: usize) {
    core::ptr::write_bytes(target as *mut u8, 0, size);
}

// Helper: SIZEOF for jpeg_compress_struct
#[inline]
fn SIZEOF_JPEG_COMPRESS_STRUCT() -> usize {
    core::mem::size_of::<jpeg_compress_struct>()
}

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
        let err = (*cinfo).err;
        MEMZERO(cinfo as *mut c_void, SIZEOF_JPEG_COMPRESS_STRUCT());
        (*cinfo).err = err;
    }
    (*cinfo).is_decompressor = FALSE;

    /* Initialize a memory manager instance for this object */
    jinit_memory_mgr(cinfo as *mut c_void);

    /* Zero out pointers to permanent structures. */
    (*cinfo).progress = null_mut();
    (*cinfo).dest = null_mut();

    (*cinfo).comp_info = null_mut();

    i = 0;
    while i < NUM_QUANT_TBLS as c_int {
        (*cinfo).quant_tbl_ptrs[i as usize] = null_mut();
        i += 1;
    }

    i = 0;
    while i < NUM_HUFF_TBLS as c_int {
        (*cinfo).dc_huff_tbl_ptrs[i as usize] = null_mut();
        (*cinfo).ac_huff_tbl_ptrs[i as usize] = null_mut();
        i += 1;
    }

    (*cinfo).input_gamma = 1.0;	/* in case application forgets */

    /* OK, I'm ready */
    (*cinfo).global_state = CSTATE_START;
}


/*
 * Destruction of a JPEG compression object
 */

pub unsafe fn jpeg_destroy_compress(cinfo: j_compress_ptr) {
    jpeg_destroy(cinfo as *mut c_void); /* use common routine */
}


/*
 * Abort processing of a JPEG compression operation,
 * but don't destroy the object itself.
 */

pub unsafe fn jpeg_abort_compress(cinfo: j_compress_ptr) {
    jpeg_abort(cinfo as *mut c_void); /* use common routine */
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

pub unsafe fn jpeg_suppress_tables(cinfo: j_compress_ptr, suppress: c_int) {
    let mut i: c_int;
    let mut qtbl: *mut JQUANT_TBL;
    let mut htbl: *mut JHUFF_TBL;

    i = 0;
    while i < NUM_QUANT_TBLS as c_int {
        qtbl = (*cinfo).quant_tbl_ptrs[i as usize];
        if !qtbl.is_null() {
            (*qtbl).sent_table = suppress;
        }
        i += 1;
    }

    i = 0;
    while i < NUM_HUFF_TBLS as c_int {
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
    let mut iMCU_row: u32;

    if (*cinfo).global_state == CSTATE_SCANNING ||
        (*cinfo).global_state == CSTATE_RAW_OK {
        /* Terminate first pass */
        if (*cinfo).next_scanline < (*cinfo).image_height {
            ERREXIT(cinfo as *mut c_void, JERR_TOO_LITTLE_DATA);
        }
        if let Some(finish_pass) = (*(*cinfo).master).finish_pass {
            finish_pass(cinfo);
        }
    } else if (*cinfo).global_state != CSTATE_WRCOEFS {
        ERREXIT1(cinfo as *mut c_void, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Perform any remaining passes */
    while (*(*cinfo).master).is_last_pass == 0 {
        if let Some(prepare_for_pass) = (*(*cinfo).master).prepare_for_pass {
            prepare_for_pass(cinfo);
        }
        iMCU_row = 0;
        while iMCU_row < (*cinfo).total_iMCU_rows {
            if !(*cinfo).progress.is_null() {
                (*(*cinfo).progress).pass_counter = iMCU_row as c_int;
                (*(*cinfo).progress).pass_limit = (*cinfo).total_iMCU_rows as c_int;
                if let Some(progress_monitor) = (*(*cinfo).progress).progress_monitor {
                    progress_monitor(cinfo as *mut c_void);
                }
            }
            /* We bypass the main controller and invoke coef controller directly;
             * all work is being done from the coefficient buffer.
             */
            if let Some(compress_data) = (*(*cinfo).coef).compress_data {
                if compress_data(cinfo, null_mut()) == 0 {
                    ERREXIT(cinfo as *mut c_void, JERR_CANT_SUSPEND);
                }
            }
            iMCU_row += 1;
        }
        if let Some(finish_pass) = (*(*cinfo).master).finish_pass {
            finish_pass(cinfo);
        }
    }
    /* Write EOI, do final cleanup */
    if let Some(write_file_trailer) = (*(*cinfo).marker).write_file_trailer {
        write_file_trailer(cinfo);
    }
    if let Some(term_destination) = (*(*cinfo).dest).term_destination {
        term_destination(cinfo);
    }
    /* We can use jpeg_abort to release memory and reset global_state */
    jpeg_abort(cinfo as *mut c_void);
}


/*
 * Write a special marker.
 * This is only recommended for writing COM or APPn markers.
 * Must be called after jpeg_start_compress() and before
 * first call to jpeg_write_scanlines() or jpeg_write_raw_data().
 */

pub unsafe fn jpeg_write_marker(cinfo: j_compress_ptr, marker: c_int,
                                dataptr: *const u8, datalen: u32) {
    if (*cinfo).next_scanline != 0 ||
        ((*cinfo).global_state != CSTATE_SCANNING &&
         (*cinfo).global_state != CSTATE_RAW_OK &&
         (*cinfo).global_state != CSTATE_WRCOEFS) {
        ERREXIT1(cinfo as *mut c_void, JERR_BAD_STATE, (*cinfo).global_state);
    }

    if let Some(write_any_marker) = (*(*cinfo).marker).write_any_marker {
        write_any_marker(cinfo, marker, dataptr, datalen);
    }
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
        ERREXIT1(cinfo as *mut c_void, JERR_BAD_STATE, (*cinfo).global_state);
    }

    /* (Re)initialize error mgr and destination modules */
    if let Some(reset_error_mgr) = (*(*cinfo).err).reset_error_mgr {
        reset_error_mgr(cinfo as *mut c_void);
    }
    if let Some(init_destination) = (*(*cinfo).dest).init_destination {
        init_destination(cinfo);
    }
    /* Initialize the marker writer ... bit of a crock to do it here. */
    jinit_marker_writer(cinfo);
    /* Write them tables! */
    if let Some(write_tables_only) = (*(*cinfo).marker).write_tables_only {
        write_tables_only(cinfo);
    }
    /* And clean up. */
    if let Some(term_destination) = (*(*cinfo).dest).term_destination {
        term_destination(cinfo);
    }
    /* We can use jpeg_abort to release memory. */
    jpeg_abort(cinfo as *mut c_void);
}
