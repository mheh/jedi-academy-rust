/*
 * jdapimin.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains application interface code for the decompression half
 * of the JPEG library.  These are the "minimum" API routines that may be
 * needed in either the normal full-decompression case or the
 * transcoding-only case.
 *
 * Most of the routines intended to be called directly by an application
 * are in this file or in jdapistd.c.  But also see jcomapi.c for routines
 * shared by compression and decompression, and jdtrans.c for the transcoding
 * case.
 */
//Anything above this #include will be ignored by the compiler
// Porting: #define JPEG_INTERNALS enables jpegint.h inclusion via jpeglib.h;
// handled by the jpegint_h glob-import below.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use core::ffi::c_int;
use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::jpeg_6::jinclude_h::*;
use crate::codemp::jpeg_6::jpeglib_h::*;
use crate::codemp::jpeg_6::jpegint_h::*;


/*
 * Initialization of a JPEG decompression object.
 * The error manager must already be set up (in case memory manager fails).
 */

pub unsafe fn jpeg_create_decompress(cinfo: j_decompress_ptr) {
    let mut i: c_int;

    /* For debugging purposes, zero the whole master structure.
     * But error manager pointer is already there, so save and restore it.
     */
    {
        let err: *mut jpeg_error_mgr = (*cinfo).err;
        MEMZERO!(cinfo, SIZEOF!(jpeg_decompress_struct));
        (*cinfo).err = err;
    }
    (*cinfo).is_decompressor = TRUE;

    /* Initialize a memory manager instance for this object */
    jinit_memory_mgr(cinfo as j_common_ptr);

    /* Zero out pointers to permanent structures. */
    (*cinfo).progress = core::ptr::null_mut();
    (*cinfo).src = core::ptr::null_mut();

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

    /* Initialize marker processor so application can override methods
     * for COM, APPn markers before calling jpeg_read_header.
     */
    jinit_marker_reader(cinfo);

    /* And initialize the overall input controller. */
    jinit_input_controller(cinfo);

    /* OK, I'm ready */
    (*cinfo).global_state = DSTATE_START;
}


/*
 * Destruction of a JPEG decompression object
 */

pub unsafe fn jpeg_destroy_decompress(cinfo: j_decompress_ptr) {
    jpeg_destroy(cinfo as j_common_ptr); /* use common routine */
}


/*
 * Abort processing of a JPEG decompression operation,
 * but don't destroy the object itself.
 */

pub unsafe fn jpeg_abort_decompress(cinfo: j_decompress_ptr) {
    jpeg_abort(cinfo as j_common_ptr); /* use common routine */
}


/*
 * Install a special processing method for COM or APPn markers.
 */

pub unsafe fn jpeg_set_marker_processor(
    cinfo: j_decompress_ptr,
    marker_code: c_int,
    routine: jpeg_marker_parser_method,
) {
    if marker_code == JPEG_COM {
        (*(*cinfo).marker).process_COM = routine;
    } else if marker_code >= JPEG_APP0 && marker_code <= JPEG_APP0 + 15 {
        (*(*cinfo).marker).process_APPn[(marker_code - JPEG_APP0) as usize] = routine;
    } else {
        ERREXIT1!(cinfo, JERR_UNKNOWN_MARKER, marker_code);
    }
}


/*
 * Set default decompression parameters.
 */

unsafe fn default_decompress_parms(cinfo: j_decompress_ptr) {
    /* Guess the input colorspace, and set output colorspace accordingly. */
    /* (Wish JPEG committee had provided a real way to specify this...) */
    /* Note application may override our guesses. */
    match (*cinfo).num_components {
        1 => {
            (*cinfo).jpeg_color_space = JCS_GRAYSCALE;
            (*cinfo).out_color_space = JCS_GRAYSCALE;
        }

        3 => {
            if (*cinfo).saw_JFIF_marker != FALSE {
                (*cinfo).jpeg_color_space = JCS_YCbCr; /* JFIF implies YCbCr */
            } else if (*cinfo).saw_Adobe_marker != FALSE {
                match (*cinfo).Adobe_transform {
                    0 => {
                        (*cinfo).jpeg_color_space = JCS_RGB;
                    }
                    1 => {
                        (*cinfo).jpeg_color_space = JCS_YCbCr;
                    }
                    _ => {
                        WARNMS1!(cinfo, JWRN_ADOBE_XFORM, (*cinfo).Adobe_transform);
                        (*cinfo).jpeg_color_space = JCS_YCbCr; /* assume it's YCbCr */
                    }
                }
            } else {
                /* Saw no special markers, try to guess from the component IDs */
                let cid0: c_int = (*(*cinfo).comp_info.add(0)).component_id;
                let cid1: c_int = (*(*cinfo).comp_info.add(1)).component_id;
                let cid2: c_int = (*(*cinfo).comp_info.add(2)).component_id;

                if cid0 == 1 && cid1 == 2 && cid2 == 3 {
                    (*cinfo).jpeg_color_space = JCS_YCbCr; /* assume JFIF w/out marker */
                } else if cid0 == 82 && cid1 == 71 && cid2 == 66 {
                    (*cinfo).jpeg_color_space = JCS_RGB; /* ASCII 'R', 'G', 'B' */
                } else {
                    TRACEMS3!(cinfo, 1, JTRC_UNKNOWN_IDS, cid0, cid1, cid2);
                    (*cinfo).jpeg_color_space = JCS_YCbCr; /* assume it's YCbCr */
                }
            }
            /* Always guess RGB is proper output colorspace. */
            (*cinfo).out_color_space = JCS_RGB;
        }

        4 => {
            if (*cinfo).saw_Adobe_marker != FALSE {
                match (*cinfo).Adobe_transform {
                    0 => {
                        (*cinfo).jpeg_color_space = JCS_CMYK;
                    }
                    2 => {
                        (*cinfo).jpeg_color_space = JCS_YCCK;
                    }
                    _ => {
                        WARNMS1!(cinfo, JWRN_ADOBE_XFORM, (*cinfo).Adobe_transform);
                        (*cinfo).jpeg_color_space = JCS_YCCK; /* assume it's YCCK */
                    }
                }
            } else {
                /* No special markers, assume straight CMYK. */
                (*cinfo).jpeg_color_space = JCS_CMYK;
            }
            (*cinfo).out_color_space = JCS_CMYK;
        }

        _ => {
            (*cinfo).jpeg_color_space = JCS_UNKNOWN;
            (*cinfo).out_color_space = JCS_UNKNOWN;
        }
    }

    /* Set defaults for other decompression parameters. */
    (*cinfo).scale_num = 1;		/* 1:1 scaling */
    (*cinfo).scale_denom = 1;
    (*cinfo).output_gamma = 1.0;
    (*cinfo).buffered_image = FALSE;
    (*cinfo).raw_data_out = FALSE;
    (*cinfo).dct_method = JDCT_DEFAULT;
    (*cinfo).do_fancy_upsampling = TRUE;
    (*cinfo).do_block_smoothing = TRUE;
    (*cinfo).quantize_colors = FALSE;
    /* We set these in case application only sets quantize_colors. */
    (*cinfo).dither_mode = JDITHER_FS;
    #[cfg(feature = "quant_2pass_supported")]
    { (*cinfo).two_pass_quantize = TRUE; }
    #[cfg(not(feature = "quant_2pass_supported"))]
    { (*cinfo).two_pass_quantize = FALSE; }
    (*cinfo).desired_number_of_colors = 256;
    (*cinfo).colormap = core::ptr::null_mut();
    /* Initialize for no mode change in buffered-image mode. */
    (*cinfo).enable_1pass_quant = FALSE;
    (*cinfo).enable_external_quant = FALSE;
    (*cinfo).enable_2pass_quant = FALSE;
}


/*
 * Decompression startup: read start of JPEG datastream to see what's there.
 * Need only initialize JPEG object and supply a data source before calling.
 *
 * This routine will read as far as the first SOS marker (ie, actual start of
 * compressed data), and will save all tables and parameters in the JPEG
 * object.  It will also initialize the decompression parameters to default
 * values, and finally return JPEG_HEADER_OK.  On return, the application may
 * adjust the decompression parameters and then call jpeg_start_decompress.
 * (Or, if the application only wanted to determine the image parameters,
 * the data need not be decompressed.  In that case, call jpeg_abort or
 * jpeg_destroy to release any temporary space.)
 * If an abbreviated (tables only) datastream is presented, the routine will
 * return JPEG_HEADER_TABLES_ONLY upon reaching EOI.  The application may then
 * re-use the JPEG object to read the abbreviated image datastream(s).
 * It is unnecessary (but OK) to call jpeg_abort in this case.
 * The JPEG_SUSPENDED return code only occurs if the data source module
 * requests suspension of the decompressor.  In this case the application
 * should load more source data and then re-call jpeg_read_header to resume
 * processing.
 * If a non-suspending data source is used and require_image is TRUE, then the
 * return code need not be inspected since only JPEG_HEADER_OK is possible.
 *
 * This routine is now just a front end to jpeg_consume_input, with some
 * extra error checking.
 */

pub unsafe fn jpeg_read_header(cinfo: j_decompress_ptr, require_image: boolean) -> c_int {
    let mut retcode: c_int;

    if (*cinfo).global_state != DSTATE_START &&
       (*cinfo).global_state != DSTATE_INHEADER
    {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }

    retcode = jpeg_consume_input(cinfo);

    if retcode == JPEG_REACHED_SOS {
        retcode = JPEG_HEADER_OK;
    } else if retcode == JPEG_REACHED_EOI {
        if require_image != FALSE {	/* Complain if application wanted an image */
            ERREXIT!(cinfo, JERR_NO_IMAGE);
        }
        /* Reset to start state; it would be safer to require the application to
         * call jpeg_abort, but we can't change it now for compatibility reasons.
         * A side effect is to free any temporary memory (there shouldn't be any).
         */
        jpeg_abort(cinfo as j_common_ptr); /* sets state = DSTATE_START */
        retcode = JPEG_HEADER_TABLES_ONLY;
    } else if retcode == JPEG_SUSPENDED {
        /* no work */
    }

    retcode
}


/*
 * Consume data in advance of what the decompressor requires.
 * This can be called at any time once the decompressor object has
 * been created and a data source has been set up.
 *
 * This routine is essentially a state machine that handles a couple
 * of critical state-transition actions, namely initial setup and
 * transition from header scanning to ready-for-start_decompress.
 * All the actual input is done via the input controller's consume_input
 * method.
 */

pub unsafe fn jpeg_consume_input(cinfo: j_decompress_ptr) -> c_int {
    let mut retcode: c_int = JPEG_SUSPENDED;
    let gs: c_int = (*cinfo).global_state;

    /* NB: every possible DSTATE value should be listed in this switch */
    if gs == DSTATE_START {
        /* Start-of-datastream actions: reset appropriate modules */
        ((*(*cinfo).inputctl).reset_input_controller)(cinfo);
        /* Initialize application's data source module */
        ((*(*cinfo).src).init_source)(cinfo);
        (*cinfo).global_state = DSTATE_INHEADER;
        /*FALLTHROUGH*/
        /* Porting: C fallthrough from DSTATE_START into DSTATE_INHEADER body. */
        retcode = ((*(*cinfo).inputctl).consume_input)(cinfo);
        if retcode == JPEG_REACHED_SOS { /* Found SOS, prepare to decompress */
            /* Set up default parameters based on header data */
            default_decompress_parms(cinfo);
            /* Set global state: ready for start_decompress */
            (*cinfo).global_state = DSTATE_READY;
        }
    } else if gs == DSTATE_INHEADER {
        retcode = ((*(*cinfo).inputctl).consume_input)(cinfo);
        if retcode == JPEG_REACHED_SOS { /* Found SOS, prepare to decompress */
            /* Set up default parameters based on header data */
            default_decompress_parms(cinfo);
            /* Set global state: ready for start_decompress */
            (*cinfo).global_state = DSTATE_READY;
        }
    } else if gs == DSTATE_READY {
        /* Can't advance past first SOS until start_decompress is called */
        retcode = JPEG_REACHED_SOS;
    } else if gs == DSTATE_PRELOAD
           || gs == DSTATE_PRESCAN
           || gs == DSTATE_SCANNING
           || gs == DSTATE_RAW_OK
           || gs == DSTATE_BUFIMAGE
           || gs == DSTATE_BUFPOST
           || gs == DSTATE_STOPPING
    {
        retcode = ((*(*cinfo).inputctl).consume_input)(cinfo);
    } else {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    retcode
}


/*
 * Have we finished reading the input file?
 */

pub unsafe fn jpeg_input_complete(cinfo: j_decompress_ptr) -> boolean {
    /* Check for valid jpeg object */
    if (*cinfo).global_state < DSTATE_START ||
       (*cinfo).global_state > DSTATE_STOPPING
    {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    (*(*cinfo).inputctl).eoi_reached
}


/*
 * Is there more than one scan?
 */

pub unsafe fn jpeg_has_multiple_scans(cinfo: j_decompress_ptr) -> boolean {
    /* Only valid after jpeg_read_header completes */
    if (*cinfo).global_state < DSTATE_READY ||
       (*cinfo).global_state > DSTATE_STOPPING
    {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    (*(*cinfo).inputctl).has_multiple_scans
}


/*
 * Finish JPEG decompression.
 *
 * This will normally just verify the file trailer and release temp storage.
 *
 * Returns FALSE if suspended.  The return value need be inspected only if
 * a suspending data source is used.
 */

pub unsafe fn jpeg_finish_decompress(cinfo: j_decompress_ptr) -> boolean {
    if ((*cinfo).global_state == DSTATE_SCANNING ||
        (*cinfo).global_state == DSTATE_RAW_OK) && (*cinfo).buffered_image == FALSE
    {
        /* Terminate final pass of non-buffered mode */
        if (*cinfo).output_scanline < (*cinfo).output_height {
            ERREXIT!(cinfo, JERR_TOO_LITTLE_DATA);
        }
        ((*(*cinfo).master).finish_output_pass)(cinfo);
        (*cinfo).global_state = DSTATE_STOPPING;
    } else if (*cinfo).global_state == DSTATE_BUFIMAGE {
        /* Finishing after a buffered-image operation */
        (*cinfo).global_state = DSTATE_STOPPING;
    } else if (*cinfo).global_state != DSTATE_STOPPING {
        /* STOPPING = repeat call after a suspension, anything else is error */
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Read until EOI */
    while (*(*cinfo).inputctl).eoi_reached == FALSE {
        if ((*(*cinfo).inputctl).consume_input)(cinfo) == JPEG_SUSPENDED {
            return FALSE;		/* Suspend, come back later */
        }
    }
    /* Do final cleanup */
    ((*(*cinfo).src).term_source)(cinfo);
    /* We can use jpeg_abort to release memory and reset global_state */
    jpeg_abort(cinfo as j_common_ptr);
    TRUE
}
