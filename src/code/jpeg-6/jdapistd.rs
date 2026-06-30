/*
 * jdapistd.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains application interface code for the decompression half
 * of the JPEG library.  These are the "standard" API routines that are
 * used in the normal full-decompression case.  They are not used by a
 * transcoding-only application.  Note that if an application links in
 * jpeg_start_decompress, it will end up linking in the entire decompressor.
 * We thus must separate this file from jdapimin.c to avoid linking the
 * whole decompression library into a transcoder.
 */

// leave this as first line for PCH reasons...
//

// Porting note: allow non_snake_case for the local variable lines_per_iMCU_row (iMCU).
#![allow(non_snake_case)]

#[allow(unused_imports)]
use crate::code::server::exe_headers_h::*;

#[allow(unused_imports)]
use crate::code::jpeg_6::jinclude_h::*;
#[allow(unused_imports)]
use crate::code::jpeg_6::jpeglib_h::*;
// JPEG_INTERNALS is defined in the C source before including jpeglib.h, which causes jpegint.h to
// be pulled in; import the internal-interface module explicitly.
#[allow(unused_imports)]
use crate::code::jpeg_6::jpegint_h::*;

/* Forward declarations */
/* output_pass_setup is defined below; no forward declaration is needed in Rust. */


/*
 * Decompression initialization.
 * jpeg_read_header must be completed before calling this.
 *
 * If a multipass operating mode was selected, this will do all but the
 * last pass, and thus may take a great deal of time.
 *
 * Returns FALSE if suspended.  The return value need be inspected only if
 * a suspending data source is used.
 */

pub unsafe fn jpeg_start_decompress(cinfo: j_decompress_ptr) -> boolean {
    if (*cinfo).global_state == DSTATE_READY {
        /* First call: initialize master control, select active modules */
        jinit_master_decompress(cinfo);
        if (*cinfo).buffered_image != 0 {
            /* No more work here; expecting jpeg_start_output next */
            (*cinfo).global_state = DSTATE_BUFIMAGE;
            return TRUE;
        }
        (*cinfo).global_state = DSTATE_PRELOAD;
    }
    if (*cinfo).global_state == DSTATE_PRELOAD {
        /* If file has multiple scans, absorb them all into the coef buffer */
        if (*(*cinfo).inputctl).has_multiple_scans != 0 {
            #[cfg(feature = "d_multiscan_files_supported")]
            {
                loop {
                    let retcode: core::ffi::c_int;
                    /* Call progress monitor hook if present */
                    if !(*cinfo).progress.is_null() {
                        ((*(*cinfo).progress).progress_monitor)(cinfo as j_common_ptr);
                    }
                    /* Absorb some more input */
                    retcode = ((*(*cinfo).inputctl).consume_input)(cinfo);
                    if retcode == JPEG_SUSPENDED {
                        return FALSE;
                    }
                    if retcode == JPEG_REACHED_EOI {
                        break;
                    }
                    /* Advance progress counter if appropriate */
                    if !(*cinfo).progress.is_null()
                        && (retcode == JPEG_ROW_COMPLETED || retcode == JPEG_REACHED_SOS)
                    {
                        (*(*cinfo).progress).pass_counter += 1;
                        if (*(*cinfo).progress).pass_counter >= (*(*cinfo).progress).pass_limit {
                            /* jdmaster underestimated number of scans; ratchet up one scan */
                            (*(*cinfo).progress).pass_limit +=
                                (*cinfo).total_iMCU_rows as core::ffi::c_long;
                        }
                    }
                }
            }
            #[cfg(not(feature = "d_multiscan_files_supported"))]
            {
                ERREXIT!(cinfo, JERR_NOT_COMPILED);
            }
        }
        (*cinfo).output_scan_number = (*cinfo).input_scan_number;
    } else if (*cinfo).global_state != DSTATE_PRESCAN {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Perform any dummy output passes, and set up for the final pass */
    return output_pass_setup(cinfo);
}


/*
 * Set up for an output pass, and perform any dummy pass(es) needed.
 * Common subroutine for jpeg_start_decompress and jpeg_start_output.
 * Entry: global_state = DSTATE_PRESCAN only if previously suspended.
 * Exit: If done, returns TRUE and sets global_state for proper output mode.
 *       If suspended, returns FALSE and sets global_state = DSTATE_PRESCAN.
 */

unsafe fn output_pass_setup(cinfo: j_decompress_ptr) -> boolean {
    if (*cinfo).global_state != DSTATE_PRESCAN {
        /* First call: do pass setup */
        ((*(*cinfo).master).prepare_for_output_pass)(cinfo);
        (*cinfo).output_scanline = 0;
        (*cinfo).global_state = DSTATE_PRESCAN;
    }
    /* Loop over any required dummy passes */
    while (*(*cinfo).master).is_dummy_pass != 0 {
        #[cfg(feature = "quant_2pass_supported")]
        {
            /* Crank through the dummy pass */
            while (*cinfo).output_scanline < (*cinfo).output_height {
                let last_scanline: JDIMENSION;
                /* Call progress monitor hook if present */
                if !(*cinfo).progress.is_null() {
                    (*(*cinfo).progress).pass_counter =
                        (*cinfo).output_scanline as core::ffi::c_long;
                    (*(*cinfo).progress).pass_limit =
                        (*cinfo).output_height as core::ffi::c_long;
                    ((*(*cinfo).progress).progress_monitor)(cinfo as j_common_ptr);
                }
                /* Process some data */
                last_scanline = (*cinfo).output_scanline;
                ((*(*cinfo).main).process_data)(
                    cinfo,
                    core::ptr::null_mut() as JSAMPARRAY,
                    core::ptr::addr_of_mut!((*cinfo).output_scanline),
                    0 as JDIMENSION,
                );
                if (*cinfo).output_scanline == last_scanline {
                    return FALSE; /* No progress made, must suspend */
                }
            }
            /* Finish up dummy pass, and set up for another one */
            ((*(*cinfo).master).finish_output_pass)(cinfo);
            ((*(*cinfo).master).prepare_for_output_pass)(cinfo);
            (*cinfo).output_scanline = 0;
        }
        #[cfg(not(feature = "quant_2pass_supported"))]
        {
            ERREXIT!(cinfo, JERR_NOT_COMPILED);
        }
    }
    /* Ready for application to drive output pass through
     * jpeg_read_scanlines or jpeg_read_raw_data.
     */
    (*cinfo).global_state = if (*cinfo).raw_data_out != 0 {
        DSTATE_RAW_OK
    } else {
        DSTATE_SCANNING
    };
    return TRUE;
}


/*
 * Read some scanlines of data from the JPEG decompressor.
 *
 * The return value will be the number of lines actually read.
 * This may be less than the number requested in several cases,
 * including bottom of image, data source suspension, and operating
 * modes that emit multiple scanlines at a time.
 *
 * Note: we warn about excess calls to jpeg_read_scanlines() since
 * this likely signals an application programmer error.  However,
 * an oversize buffer (max_lines > scanlines remaining) is not an error.
 */

pub unsafe fn jpeg_read_scanlines(
    cinfo: j_decompress_ptr,
    scanlines: JSAMPARRAY,
    max_lines: JDIMENSION,
) -> JDIMENSION {
    let mut row_ctr: JDIMENSION;

    if (*cinfo).global_state != DSTATE_SCANNING {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    if (*cinfo).output_scanline >= (*cinfo).output_height {
        WARNMS!(cinfo, JWRN_TOO_MUCH_DATA);
        return 0;
    }

    /* Call progress monitor hook if present */
    if !(*cinfo).progress.is_null() {
        (*(*cinfo).progress).pass_counter = (*cinfo).output_scanline as core::ffi::c_long;
        (*(*cinfo).progress).pass_limit = (*cinfo).output_height as core::ffi::c_long;
        ((*(*cinfo).progress).progress_monitor)(cinfo as j_common_ptr);
    }

    /* Process some data */
    row_ctr = 0;
    ((*(*cinfo).main).process_data)(
        cinfo,
        scanlines,
        core::ptr::addr_of_mut!(row_ctr),
        max_lines,
    );
    (*cinfo).output_scanline += row_ctr;
    return row_ctr;
}


/*
 * Alternate entry point to read raw data.
 * Processes exactly one iMCU row per call, unless suspended.
 */

pub unsafe fn jpeg_read_raw_data(
    cinfo: j_decompress_ptr,
    data: JSAMPIMAGE,
    max_lines: JDIMENSION,
) -> JDIMENSION {
    let lines_per_iMCU_row: JDIMENSION;

    if (*cinfo).global_state != DSTATE_RAW_OK {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    if (*cinfo).output_scanline >= (*cinfo).output_height {
        WARNMS!(cinfo, JWRN_TOO_MUCH_DATA);
        return 0;
    }

    /* Call progress monitor hook if present */
    if !(*cinfo).progress.is_null() {
        (*(*cinfo).progress).pass_counter = (*cinfo).output_scanline as core::ffi::c_long;
        (*(*cinfo).progress).pass_limit = (*cinfo).output_height as core::ffi::c_long;
        ((*(*cinfo).progress).progress_monitor)(cinfo as j_common_ptr);
    }

    /* Verify that at least one iMCU row can be returned. */
    lines_per_iMCU_row =
        ((*cinfo).max_v_samp_factor * (*cinfo).min_DCT_scaled_size) as JDIMENSION;
    if max_lines < lines_per_iMCU_row {
        ERREXIT!(cinfo, JERR_BUFFER_SIZE);
    }

    /* Decompress directly into user's buffer. */
    if ((*(*cinfo).coef).decompress_data)(cinfo, data) == 0 {
        return 0; /* suspension forced, can do nothing more */
    }

    /* OK, we processed one iMCU row. */
    (*cinfo).output_scanline += lines_per_iMCU_row;
    return lines_per_iMCU_row;
}


/* Additional entry points for buffered-image mode. */

/*
 * Initialize for an output pass in buffered-image mode.
 */

#[cfg(feature = "d_multiscan_files_supported")]
pub unsafe fn jpeg_start_output(
    cinfo: j_decompress_ptr,
    mut scan_number: core::ffi::c_int,
) -> boolean {
    if (*cinfo).global_state != DSTATE_BUFIMAGE && (*cinfo).global_state != DSTATE_PRESCAN {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Limit scan number to valid range */
    if scan_number <= 0 {
        scan_number = 1;
    }
    if (*(*cinfo).inputctl).eoi_reached != 0 && scan_number > (*cinfo).input_scan_number {
        scan_number = (*cinfo).input_scan_number;
    }
    (*cinfo).output_scan_number = scan_number;
    /* Perform any dummy output passes, and set up for the real pass */
    return output_pass_setup(cinfo);
}


/*
 * Finish up after an output pass in buffered-image mode.
 *
 * Returns FALSE if suspended.  The return value need be inspected only if
 * a suspending data source is used.
 */

#[cfg(feature = "d_multiscan_files_supported")]
pub unsafe fn jpeg_finish_output(cinfo: j_decompress_ptr) -> boolean {
    if ((*cinfo).global_state == DSTATE_SCANNING
        || (*cinfo).global_state == DSTATE_RAW_OK)
        && (*cinfo).buffered_image != 0
    {
        /* Terminate this pass. */
        /* We do not require the whole pass to have been completed. */
        ((*(*cinfo).master).finish_output_pass)(cinfo);
        (*cinfo).global_state = DSTATE_BUFPOST;
    } else if (*cinfo).global_state != DSTATE_BUFPOST {
        /* BUFPOST = repeat call after a suspension, anything else is error */
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Read markers looking for SOS or EOI */
    while (*cinfo).input_scan_number <= (*cinfo).output_scan_number
        && (*(*cinfo).inputctl).eoi_reached == 0
    {
        if ((*(*cinfo).inputctl).consume_input)(cinfo) == JPEG_SUSPENDED {
            return FALSE; /* Suspend, come back later */
        }
    }
    (*cinfo).global_state = DSTATE_BUFIMAGE;
    return TRUE;
}

/* #endif D_MULTISCAN_FILES_SUPPORTED */
