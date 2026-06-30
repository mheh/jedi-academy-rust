/*
 * jdinput.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains input control logic for the JPEG decompressor.
 * These routines are concerned with controlling the decompressor's input
 * processing (marker reading and coefficient decoding).  The actual input
 * reading is done in jdmarker.c, jdhuff.c, and jdphuff.c.
 */
/* Anything above this #include will be ignored by the compiler */

#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_parens,
    dead_code,
    unused_variables,
    unused_mut,
    clippy::all,
)]

use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::jpeg_6::jinclude_h::*;
use crate::codemp::jpeg_6::jpeglib_h::*;
use crate::codemp::jpeg_6::jpegint_h::*;

/* Private state */

#[repr(C)]
pub struct my_input_controller {
    pub r#pub: jpeg_input_controller, /* public fields */
    pub inheaders: boolean,           /* TRUE until first SOS is reached */
}

pub type my_inputctl_ptr = *mut my_input_controller;


/* Forward declarations */
/* METHODDEF int consume_markers JPP((j_decompress_ptr cinfo)); */


/*
 * Routines to calculate various quantities related to the size of the image.
 */

/* Called once, when first SOS marker is reached */
unsafe fn initial_setup(cinfo: j_decompress_ptr) {
    let mut ci: core::ffi::c_int;
    let mut compptr: *mut jpeg_component_info;

    /* Make sure image isn't bigger than I can handle */
    if ((*cinfo).image_height as core::ffi::c_long) > (JPEG_MAX_DIMENSION as core::ffi::c_long)
        || ((*cinfo).image_width as core::ffi::c_long) > (JPEG_MAX_DIMENSION as core::ffi::c_long)
    {
        ERREXIT1!(cinfo, JERR_IMAGE_TOO_BIG, (JPEG_MAX_DIMENSION as core::ffi::c_uint));
    }

    /* For now, precision must match compiled-in value... */
    if (*cinfo).data_precision != BITS_IN_JSAMPLE {
        ERREXIT1!(cinfo, JERR_BAD_PRECISION, (*cinfo).data_precision);
    }

    /* Check that number of components won't exceed internal array sizes */
    if (*cinfo).num_components > MAX_COMPONENTS {
        ERREXIT2!(cinfo, JERR_COMPONENT_COUNT, (*cinfo).num_components, MAX_COMPONENTS);
    }

    /* Compute maximum sampling factors; check factor validity */
    (*cinfo).max_h_samp_factor = 1;
    (*cinfo).max_v_samp_factor = 1;
    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        if (*compptr).h_samp_factor <= 0
            || (*compptr).h_samp_factor > MAX_SAMP_FACTOR
            || (*compptr).v_samp_factor <= 0
            || (*compptr).v_samp_factor > MAX_SAMP_FACTOR
        {
            ERREXIT!(cinfo, JERR_BAD_SAMPLING);
        }
        (*cinfo).max_h_samp_factor = MAX!((*cinfo).max_h_samp_factor, (*compptr).h_samp_factor);
        (*cinfo).max_v_samp_factor = MAX!((*cinfo).max_v_samp_factor, (*compptr).v_samp_factor);
        ci += 1;
        compptr = compptr.add(1);
    }

    /* We initialize DCT_scaled_size and min_DCT_scaled_size to DCTSIZE.
     * In the full decompressor, this will be overridden by jdmaster.c;
     * but in the transcoder, jdmaster.c is not used, so we must do it here.
     */
    (*cinfo).min_DCT_scaled_size = DCTSIZE;

    /* Compute dimensions of components */
    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        (*compptr).DCT_scaled_size = DCTSIZE;
        /* Size in DCT blocks */
        (*compptr).width_in_blocks = jdiv_round_up(
            ((*cinfo).image_width as core::ffi::c_long)
                * ((*compptr).h_samp_factor as core::ffi::c_long),
            ((*cinfo).max_h_samp_factor * DCTSIZE) as core::ffi::c_long,
        ) as JDIMENSION;
        (*compptr).height_in_blocks = jdiv_round_up(
            ((*cinfo).image_height as core::ffi::c_long)
                * ((*compptr).v_samp_factor as core::ffi::c_long),
            ((*cinfo).max_v_samp_factor * DCTSIZE) as core::ffi::c_long,
        ) as JDIMENSION;
        /* downsampled_width and downsampled_height will also be overridden by
         * jdmaster.c if we are doing full decompression.  The transcoder library
         * doesn't use these values, but the calling application might.
         */
        /* Size in samples */
        (*compptr).downsampled_width = jdiv_round_up(
            ((*cinfo).image_width as core::ffi::c_long)
                * ((*compptr).h_samp_factor as core::ffi::c_long),
            (*cinfo).max_h_samp_factor as core::ffi::c_long,
        ) as JDIMENSION;
        (*compptr).downsampled_height = jdiv_round_up(
            ((*cinfo).image_height as core::ffi::c_long)
                * ((*compptr).v_samp_factor as core::ffi::c_long),
            (*cinfo).max_v_samp_factor as core::ffi::c_long,
        ) as JDIMENSION;
        /* Mark component needed, until color conversion says otherwise */
        (*compptr).component_needed = TRUE;
        /* Mark no quantization table yet saved for component */
        (*compptr).quant_table = core::ptr::null_mut();
        ci += 1;
        compptr = compptr.add(1);
    }

    /* Compute number of fully interleaved MCU rows. */
    (*cinfo).total_iMCU_rows = jdiv_round_up(
        (*cinfo).image_height as core::ffi::c_long,
        ((*cinfo).max_v_samp_factor * DCTSIZE) as core::ffi::c_long,
    ) as JDIMENSION;

    /* Decide whether file contains multiple scans */
    if (*cinfo).comps_in_scan < (*cinfo).num_components || (*cinfo).progressive_mode != 0 {
        (*(*cinfo).inputctl).has_multiple_scans = TRUE;
    } else {
        (*(*cinfo).inputctl).has_multiple_scans = FALSE;
    }
}


/* Do computations that are needed before processing a JPEG scan */
/* cinfo->comps_in_scan and cinfo->cur_comp_info[] were set from SOS marker */
unsafe fn per_scan_setup(cinfo: j_decompress_ptr) {
    let mut ci: core::ffi::c_int;
    let mut mcublks: core::ffi::c_int;
    let mut tmp: core::ffi::c_int;
    let mut compptr: *mut jpeg_component_info;

    if (*cinfo).comps_in_scan == 1 {

        /* Noninterleaved (single-component) scan */
        compptr = (*cinfo).cur_comp_info[0];

        /* Overall image size in MCUs */
        (*cinfo).MCUs_per_row = (*compptr).width_in_blocks;
        (*cinfo).MCU_rows_in_scan = (*compptr).height_in_blocks;

        /* For noninterleaved scan, always one block per MCU */
        (*compptr).MCU_width = 1;
        (*compptr).MCU_height = 1;
        (*compptr).MCU_blocks = 1;
        (*compptr).MCU_sample_width = (*compptr).DCT_scaled_size;
        (*compptr).last_col_width = 1;
        /* For noninterleaved scans, it is convenient to define last_row_height
         * as the number of block rows present in the last iMCU row.
         */
        tmp = ((*compptr).height_in_blocks as core::ffi::c_int) % (*compptr).v_samp_factor;
        if tmp == 0 {
            tmp = (*compptr).v_samp_factor;
        }
        (*compptr).last_row_height = tmp;

        /* Prepare array describing MCU composition */
        (*cinfo).blocks_in_MCU = 1;
        (*cinfo).MCU_membership[0] = 0;

    } else {

        /* Interleaved (multi-component) scan */
        if (*cinfo).comps_in_scan <= 0 || (*cinfo).comps_in_scan > MAX_COMPS_IN_SCAN {
            ERREXIT2!(cinfo, JERR_COMPONENT_COUNT, (*cinfo).comps_in_scan, MAX_COMPS_IN_SCAN);
        }

        /* Overall image size in MCUs */
        (*cinfo).MCUs_per_row = jdiv_round_up(
            (*cinfo).image_width as core::ffi::c_long,
            ((*cinfo).max_h_samp_factor * DCTSIZE) as core::ffi::c_long,
        ) as JDIMENSION;
        (*cinfo).MCU_rows_in_scan = jdiv_round_up(
            (*cinfo).image_height as core::ffi::c_long,
            ((*cinfo).max_v_samp_factor * DCTSIZE) as core::ffi::c_long,
        ) as JDIMENSION;

        (*cinfo).blocks_in_MCU = 0;

        ci = 0;
        while ci < (*cinfo).comps_in_scan {
            compptr = (*cinfo).cur_comp_info[ci as usize];
            /* Sampling factors give # of blocks of component in each MCU */
            (*compptr).MCU_width = (*compptr).h_samp_factor;
            (*compptr).MCU_height = (*compptr).v_samp_factor;
            (*compptr).MCU_blocks = (*compptr).MCU_width * (*compptr).MCU_height;
            (*compptr).MCU_sample_width = (*compptr).MCU_width * (*compptr).DCT_scaled_size;
            /* Figure number of non-dummy blocks in last MCU column & row */
            tmp = ((*compptr).width_in_blocks as core::ffi::c_int) % (*compptr).MCU_width;
            if tmp == 0 {
                tmp = (*compptr).MCU_width;
            }
            (*compptr).last_col_width = tmp;
            tmp = ((*compptr).height_in_blocks as core::ffi::c_int) % (*compptr).MCU_height;
            if tmp == 0 {
                tmp = (*compptr).MCU_height;
            }
            (*compptr).last_row_height = tmp;
            /* Prepare array describing MCU composition */
            mcublks = (*compptr).MCU_blocks;
            if (*cinfo).blocks_in_MCU + mcublks > D_MAX_BLOCKS_IN_MCU {
                ERREXIT!(cinfo, JERR_BAD_MCU_SIZE);
            }
            /* C: while (mcublks-- > 0) { cinfo->MCU_membership[cinfo->blocks_in_MCU++] = ci; } */
            while mcublks > 0 {
                mcublks -= 1;
                let bim = (*cinfo).blocks_in_MCU as usize;
                (*cinfo).MCU_membership[bim] = ci;
                (*cinfo).blocks_in_MCU += 1;
            }
            ci += 1;
        }

    }
}


/*
 * Save away a copy of the Q-table referenced by each component present
 * in the current scan, unless already saved during a prior scan.
 *
 * In a multiple-scan JPEG file, the encoder could assign different components
 * the same Q-table slot number, but change table definitions between scans
 * so that each component uses a different Q-table.  (The IJG encoder is not
 * currently capable of doing this, but other encoders might.)  Since we want
 * to be able to dequantize all the components at the end of the file, this
 * means that we have to save away the table actually used for each component.
 * We do this by copying the table at the start of the first scan containing
 * the component.
 * The JPEG spec prohibits the encoder from changing the contents of a Q-table
 * slot between scans of a component using that slot.  If the encoder does so
 * anyway, this decoder will simply use the Q-table values that were current
 * at the start of the first scan for the component.
 *
 * The decompressor output side looks only at the saved quant tables,
 * not at the current Q-table slots.
 */

unsafe fn latch_quant_tables(cinfo: j_decompress_ptr) {
    let mut ci: core::ffi::c_int;
    let mut qtblno: core::ffi::c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut qtbl: *mut JQUANT_TBL;

    ci = 0;
    while ci < (*cinfo).comps_in_scan {
        compptr = (*cinfo).cur_comp_info[ci as usize];
        /* No work if we already saved Q-table for this component */
        if !(*compptr).quant_table.is_null() {
            ci += 1;
            continue;
        }
        /* Make sure specified quantization table is present */
        qtblno = (*compptr).quant_tbl_no;
        if qtblno < 0
            || qtblno >= NUM_QUANT_TBLS
            || (*cinfo).quant_tbl_ptrs[qtblno as usize].is_null()
        {
            ERREXIT1!(cinfo, JERR_NO_QUANT_TABLE, qtblno);
        }
        /* OK, save away the quantization table */
        qtbl = ((*(*cinfo).mem).alloc_small)(
            cinfo as j_common_ptr,
            JPOOL_IMAGE,
            SIZEOF!(JQUANT_TBL),
        ) as *mut JQUANT_TBL;
        MEMCOPY!(qtbl, (*cinfo).quant_tbl_ptrs[qtblno as usize], SIZEOF!(JQUANT_TBL));
        (*compptr).quant_table = qtbl;
        ci += 1;
    }
}


/*
 * Initialize the input modules to read a scan of compressed data.
 * The first call to this is done by jdmaster.c after initializing
 * the entire decompressor (during jpeg_start_decompress).
 * Subsequent calls come from consume_markers, below.
 */

pub unsafe fn start_input_pass(cinfo: j_decompress_ptr) {
    per_scan_setup(cinfo);
    latch_quant_tables(cinfo);
    ((*(*cinfo).entropy).start_pass)(cinfo);
    ((*(*cinfo).coef).start_input_pass)(cinfo);
    (*(*cinfo).inputctl).consume_input = (*(*cinfo).coef).consume_data;
}


/*
 * Finish up after inputting a compressed-data scan.
 * This is called by the coefficient controller after it's read all
 * the expected data of the scan.
 */

pub unsafe fn finish_input_pass(cinfo: j_decompress_ptr) {
    (*(*cinfo).inputctl).consume_input = consume_markers;
}


/*
 * Read JPEG markers before, between, or after compressed-data scans.
 * Change state as necessary when a new scan is reached.
 * Return value is JPEG_SUSPENDED, JPEG_REACHED_SOS, or JPEG_REACHED_EOI.
 *
 * The consume_input method pointer points either here or to the
 * coefficient controller's consume_data routine, depending on whether
 * we are reading a compressed data segment or inter-segment markers.
 */

pub unsafe fn consume_markers(cinfo: j_decompress_ptr) -> core::ffi::c_int {
    let inputctl: my_inputctl_ptr = (*cinfo).inputctl as my_inputctl_ptr;
    let val: core::ffi::c_int;

    if (*inputctl).r#pub.eoi_reached != 0 {
        /* After hitting EOI, read no further */
        return JPEG_REACHED_EOI;
    }

    val = ((*(*cinfo).marker).read_markers)(cinfo);

    match val {
        JPEG_REACHED_SOS => {
            /* Found SOS */
            if (*inputctl).inheaders != 0 {
                /* 1st SOS */
                initial_setup(cinfo);
                (*inputctl).inheaders = FALSE;
                /* Note: start_input_pass must be called by jdmaster.c
                 * before any more input can be consumed.  jdapi.c is
                 * responsible for enforcing this sequencing.
                 */
            } else {
                /* 2nd or later SOS marker */
                if (*inputctl).r#pub.has_multiple_scans == 0 {
                    ERREXIT!(cinfo, JERR_EOI_EXPECTED); /* Oops, I wasn't expecting this! */
                }
                start_input_pass(cinfo);
            }
        }
        JPEG_REACHED_EOI => {
            /* Found EOI */
            (*inputctl).r#pub.eoi_reached = TRUE;
            if (*inputctl).inheaders != 0 {
                /* Tables-only datastream, apparently */
                if (*(*cinfo).marker).saw_SOF != 0 {
                    ERREXIT!(cinfo, JERR_SOF_NO_SOS);
                }
            } else {
                /* Prevent infinite loop in coef ctlr's decompress_data routine
                 * if user set output_scan_number larger than number of scans.
                 */
                if (*cinfo).output_scan_number > (*cinfo).input_scan_number {
                    (*cinfo).output_scan_number = (*cinfo).input_scan_number;
                }
            }
        }
        JPEG_SUSPENDED => {}
        _ => {}
    }

    val
}


/*
 * Reset state to begin a fresh datastream.
 */

pub unsafe fn reset_input_controller(cinfo: j_decompress_ptr) {
    let inputctl: my_inputctl_ptr = (*cinfo).inputctl as my_inputctl_ptr;

    (*inputctl).r#pub.consume_input = consume_markers;
    (*inputctl).r#pub.has_multiple_scans = FALSE; /* "unknown" would be better */
    (*inputctl).r#pub.eoi_reached = FALSE;
    (*inputctl).inheaders = TRUE;
    /* Reset other modules */
    ((*(*cinfo).err).reset_error_mgr)(cinfo as j_common_ptr);
    ((*(*cinfo).marker).reset_marker_reader)(cinfo);
    /* Reset progression state -- would be cleaner if entropy decoder did this */
    (*cinfo).coef_bits = core::ptr::null_mut();
}


/*
 * Initialize the input controller module.
 * This is called only once, when the decompression object is created.
 */

pub unsafe fn jinit_input_controller(cinfo: j_decompress_ptr) {
    let inputctl: my_inputctl_ptr;

    /* Create subobject in permanent pool */
    inputctl = ((*(*cinfo).mem).alloc_small)(
        cinfo as j_common_ptr,
        JPOOL_PERMANENT,
        SIZEOF!(my_input_controller),
    ) as my_inputctl_ptr;
    (*cinfo).inputctl = inputctl as *mut jpeg_input_controller;
    /* Initialize method pointers */
    (*inputctl).r#pub.consume_input = consume_markers;
    (*inputctl).r#pub.reset_input_controller = reset_input_controller;
    (*inputctl).r#pub.start_input_pass = start_input_pass;
    (*inputctl).r#pub.finish_input_pass = finish_input_pass;
    /* Initialize state: can't use reset_input_controller since we don't
     * want to try to reset other modules yet.
     */
    (*inputctl).r#pub.has_multiple_scans = FALSE; /* "unknown" would be better */
    (*inputctl).r#pub.eoi_reached = FALSE;
    (*inputctl).inheaders = TRUE;
}
