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

// #define JPEG_INTERNALS
// #include "jinclude.h"
// #include "jpeglib.h"

#![allow(non_snake_case)]

use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::jpeg_6::jinclude_h::*;
use crate::codemp::jpeg_6::jpeglib_h::*;

/* Forward declarations */
// LOCAL void transdecode_master_selection JPP((j_decompress_ptr cinfo));


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
    if (*cinfo).global_state == DSTATE_READY {
        /* First call: initialize active modules */
        transdecode_master_selection(cinfo);
        (*cinfo).global_state = DSTATE_RDCOEFS;
    } else if (*cinfo).global_state != DSTATE_RDCOEFS {
        ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Absorb whole file into the coef buffer */
    loop {
        let retcode: core::ffi::c_int;
        /* Call progress monitor hook if present */
        if !(*cinfo).progress.is_null() {
            ((*(*cinfo).progress).progress_monitor)((cinfo as j_common_ptr));
        }
        /* Absorb some more input */
        retcode = ((*(*cinfo).inputctl).consume_input)(cinfo);
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
                (*(*cinfo).progress).pass_limit += (*cinfo).total_iMCU_rows as core::ffi::c_long;
            }
        }
    }
    /* Set state so that jpeg_finish_decompress does the right thing */
    (*cinfo).global_state = DSTATE_STOPPING;
    (*(*cinfo).coef).coef_arrays
}


/*
 * Master selection of decompression modules for transcoding.
 * This substitutes for jdmaster.c's initialization of the full decompressor.
 */

unsafe fn transdecode_master_selection(cinfo: j_decompress_ptr) {
    /* Entropy decoding: either Huffman or arithmetic coding. */
    if (*cinfo).arith_code != 0 {
        ERREXIT(cinfo, JERR_ARITH_NOTIMPL);
    } else {
        if (*cinfo).progressive_mode != 0 {
            #[cfg(feature = "D_PROGRESSIVE_SUPPORTED")]
            jinit_phuff_decoder(cinfo);
            #[cfg(not(feature = "D_PROGRESSIVE_SUPPORTED"))]
            ERREXIT(cinfo, JERR_NOT_COMPILED);
        } else {
            jinit_huff_decoder(cinfo);
        }
    }

    /* Always get a full-image coefficient buffer. */
    jinit_d_coef_controller(cinfo, TRUE);

    /* We can now tell the memory manager to allocate virtual arrays. */
    ((*(*cinfo).mem).realize_virt_arrays)((cinfo as j_common_ptr));

    /* Initialize input side of decompressor to consume first scan. */
    ((*(*cinfo).inputctl).start_input_pass)(cinfo);

    /* Initialize progress monitoring. */
    if !(*cinfo).progress.is_null() {
        let nscans: core::ffi::c_int;
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
        (*(*cinfo).progress).pass_limit = (*cinfo).total_iMCU_rows as core::ffi::c_long * nscans as core::ffi::c_long;
        (*(*cinfo).progress).completed_passes = 0;
        (*(*cinfo).progress).total_passes = 1;
    }
}
