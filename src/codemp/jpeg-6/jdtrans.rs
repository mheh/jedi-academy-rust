/*
 * jdtrans.c
 *
 * Copyright (C) 1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains library routines for transcoding decompression,
 * that is, reading raw DCT coefficient arrays from an input JPEG file.
 * The routines in jdapimin.c will also be needed by a transcoder.
 */
// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
//
// #define JPEG_INTERNALS
// #include "jinclude.h"
// #include "jpeglib.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_long, c_void};

/* ============================================================================
 * Stubs for JPEG-6 types and structures needed for structural coherence
 * ============================================================================ */

pub type JDIMENSION = u32;
pub type boolean = u8;

const TRUE: boolean = 1;

/* DSTATE_* constants from jpegint.h */
const DSTATE_READY: c_int = 202;       /* found SOS, ready for start_decompress */
const DSTATE_RDCOEFS: c_int = 209;     /* reading file in jpeg_read_coefficients */
const DSTATE_STOPPING: c_int = 210;    /* looking for EOI in jpeg_finish_decompress */

/* JPEG return codes from jpeglib.h */
const JPEG_SUSPENDED: c_int = 0;       /* Suspended due to lack of input data */
const JPEG_REACHED_SOS: c_int = 1;     /* Reached start of new scan */
const JPEG_REACHED_EOI: c_int = 2;     /* Reached end of image */
const JPEG_ROW_COMPLETED: c_int = 3;   /* Completed one iMCU row */

/* Error codes from jerror.h */
const JERR_ARITH_NOTIMPL: c_int = 4;
const JERR_BAD_STATE: c_int = 5;
const JERR_NOT_COMPILED: c_int = 6;

#[repr(C)]
pub struct jvirt_barray_control {
    _opaque: [u8; 0],
}
pub type jvirt_barray_ptr = *mut jvirt_barray_control;

#[repr(C)]
pub struct jpeg_progress_mgr {
    pub pass_counter: c_long,
    pub pass_limit: c_long,
    pub completed_passes: c_int,
    pub total_passes: c_int,
    pub progress_monitor: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct jpeg_input_controller {
    pub consume_input: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub start_input_pass: Option<unsafe extern "C" fn(*mut c_void)>,
    pub has_multiple_scans: boolean,
}

#[repr(C)]
pub struct jpeg_d_coef_controller {
    pub coef_arrays: *mut jvirt_barray_ptr,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub realize_virt_arrays: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct jpeg_error_mgr {
    pub msg_code: c_int,
    pub msg_parm: msg_parm_union,
    pub error_exit: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub union msg_parm_union {
    pub i: [c_int; 8],
    pub s: [u8; 80],
}

#[repr(C)]
pub struct j_decompress_struct {
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub progress: *mut jpeg_progress_mgr,
    pub is_decompressor: boolean,
    pub global_state: c_int,
    pub inputctl: *mut jpeg_input_controller,
    pub coef: *mut jpeg_d_coef_controller,
    pub arith_code: boolean,
    pub progressive_mode: boolean,
    pub num_components: c_int,
    pub total_iMCU_rows: JDIMENSION,
}

pub type j_decompress_ptr = *mut j_decompress_struct;

#[repr(C)]
pub struct j_common_struct {
    _opaque: [u8; 0],
}
pub type j_common_ptr = *mut j_common_struct;

/* Forward declarations */
unsafe fn transdecode_master_selection(cinfo: j_decompress_ptr);

/* External function declarations */
extern "C" {
    pub fn jinit_phuff_decoder(cinfo: j_decompress_ptr);
    pub fn jinit_huff_decoder(cinfo: j_decompress_ptr);
    pub fn jinit_d_coef_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean);
}

/* Macro: ERREXIT(cinfo, code)
 * Sets error code and calls error exit handler
 * Translates: ((cinfo)->err->msg_code = (code),
 *            (*(cinfo)->err->error_exit) ((j_common_ptr) (cinfo))) */
#[inline]
unsafe fn ERREXIT(cinfo: j_decompress_ptr, code: c_int) {
    (*(*cinfo).err).msg_code = code;
    if let Some(error_exit) = (*(*cinfo).err).error_exit {
        error_exit(cinfo as *mut c_void);
    }
}

/* Macro: ERREXIT1(cinfo, code, p1)
 * Sets error code with one parameter and calls error exit handler
 * Translates: ((cinfo)->err->msg_code = (code),
 *            (cinfo)->err->msg_parm.i[0] = (p1),
 *            (*(cinfo)->err->error_exit) ((j_common_ptr) (cinfo))) */
#[inline]
unsafe fn ERREXIT1(cinfo: j_decompress_ptr, code: c_int, p1: c_int) {
    (*(*cinfo).err).msg_code = code;
    (*(*cinfo).err).msg_parm.i[0] = p1;
    if let Some(error_exit) = (*(*cinfo).err).error_exit {
        error_exit(cinfo as *mut c_void);
    }
}

/*
 * Read the coefficient arrays from a JPEG file.
 * jpeg_read_header must be completed before calling this.
 *
 * The entire image is read into a set of virtual coefficient-block arrays,
 * one per component.  The return value is a pointer to the array of
 * virtual-array descriptors.  These can be manipulated directly via the
 * JPEG memory manager, or handed off to jpeg_write_coefficients().
 * To release the memory occupied by the virtual arrays, call
 * jpeg_finish_decompress() when done with the data.
 *
 * Returns NULL if suspended.  This case need be checked only if
 * a suspending data source is used.
 */

pub unsafe fn jpeg_read_coefficients(cinfo: j_decompress_ptr) -> *mut jvirt_barray_ptr {
    let mut retcode: c_int;

    if (*cinfo).global_state == DSTATE_READY {
        /* First call: initialize active modules */
        transdecode_master_selection(cinfo);
        (*cinfo).global_state = DSTATE_RDCOEFS;
    } else if (*cinfo).global_state != DSTATE_RDCOEFS {
        ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }

    /* Absorb whole file into the coef buffer */
    loop {
        /* Call progress monitor hook if present */
        if !(*cinfo).progress.is_null() {
            if let Some(progress_monitor) = (*(*cinfo).progress).progress_monitor {
                progress_monitor(cinfo as *mut c_void);
            }
        }

        /* Absorb some more input */
        retcode = if let Some(consume_input) = (*(*cinfo).inputctl).consume_input {
            consume_input(cinfo as *mut c_void)
        } else {
            JPEG_SUSPENDED
        };

        if retcode == JPEG_SUSPENDED {
            return core::ptr::null_mut();
        }
        if retcode == JPEG_REACHED_EOI {
            break;
        }

        /* Advance progress counter if appropriate */
        if !(*cinfo).progress.is_null() &&
            (retcode == JPEG_ROW_COMPLETED || retcode == JPEG_REACHED_SOS)
        {
            (*(*cinfo).progress).pass_counter += 1;
            if (*(*cinfo).progress).pass_counter >= (*(*cinfo).progress).pass_limit {
                /* startup underestimated number of scans; ratchet up one scan */
                (*(*cinfo).progress).pass_limit +=
                    (*cinfo).total_iMCU_rows as c_long;
            }
        }
    }

    /* Set state so that jpeg_finish_decompress does the right thing */
    (*cinfo).global_state = DSTATE_STOPPING;
    return (*(*cinfo).coef).coef_arrays;
}


/*
 * Master selection of decompression modules for transcoding.
 * This substitutes for jdmaster.c's initialization of the full decompressor.
 */

unsafe fn transdecode_master_selection(cinfo: j_decompress_ptr) {
    let nscans: c_int;

    /* Entropy decoding: either Huffman or arithmetic coding. */
    if (*cinfo).arith_code != 0 {
        ERREXIT(cinfo, JERR_ARITH_NOTIMPL);
    } else {
        if (*cinfo).progressive_mode != 0 {
            #[cfg(feature = "D_PROGRESSIVE_SUPPORTED")]
            {
                jinit_phuff_decoder(cinfo);
            }
            #[cfg(not(feature = "D_PROGRESSIVE_SUPPORTED"))]
            {
                ERREXIT(cinfo, JERR_NOT_COMPILED);
            }
        } else {
            jinit_huff_decoder(cinfo);
        }
    }

    /* Always get a full-image coefficient buffer. */
    jinit_d_coef_controller(cinfo, TRUE);

    /* We can now tell the memory manager to allocate virtual arrays. */
    if let Some(realize_virt_arrays) = (*(*cinfo).mem).realize_virt_arrays {
        realize_virt_arrays(cinfo as *mut c_void);
    }

    /* Initialize input side of decompressor to consume first scan. */
    if let Some(start_input_pass) = (*(*cinfo).inputctl).start_input_pass {
        start_input_pass(cinfo as *mut c_void);
    }

    /* Initialize progress monitoring. */
    if !(*cinfo).progress.is_null() {
        /* Estimate number of scans to set pass_limit. */
        if (*cinfo).progressive_mode != 0 {
            /* Arbitrarily estimate 2 interleaved DC scans + 3 AC scans/component. */
            nscans = 2 + 3 * (*cinfo).num_components;
        } else if (*(*cinfo).inputctl).has_multiple_scans != 0 {
            /* For a nonprogressive multiscan file, estimate 1 scan per component. */
            nscans = (*cinfo).num_components;
        } else {
            nscans = 1;
        }
        (*(*cinfo).progress).pass_counter = 0;
        (*(*cinfo).progress).pass_limit =
            ((*cinfo).total_iMCU_rows as c_long) * (nscans as c_long);
        (*(*cinfo).progress).completed_passes = 0;
        (*(*cinfo).progress).total_passes = 1;
    }
}
