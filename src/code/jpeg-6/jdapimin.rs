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

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_void};
use core::mem;
use core::ptr::{addr_of, addr_of_mut};

/* Stub definitions for JPEG library types and constants from jpeglib.h and jinclude.h */

pub const NUM_QUANT_TBLS: usize = 4;
pub const NUM_HUFF_TBLS: usize = 4;

pub const DSTATE_START: c_int = 0;
pub const DSTATE_INHEADER: c_int = 1;
pub const DSTATE_READY: c_int = 2;
pub const DSTATE_PRELOAD: c_int = 3;
pub const DSTATE_PRESCAN: c_int = 4;
pub const DSTATE_SCANNING: c_int = 5;
pub const DSTATE_RAW_OK: c_int = 6;
pub const DSTATE_BUFIMAGE: c_int = 7;
pub const DSTATE_BUFPOST: c_int = 8;
pub const DSTATE_STOPPING: c_int = 9;

pub const JCS_UNKNOWN: c_int = 0;
pub const JCS_GRAYSCALE: c_int = 1;
pub const JCS_RGB: c_int = 2;
pub const JCS_YCbCr: c_int = 3;
pub const JCS_CMYK: c_int = 4;
pub const JCS_YCCK: c_int = 5;

pub const JDCT_DEFAULT: c_int = 0;

pub const JDITHER_FS: c_int = 0;

pub const JPEG_SUSPENDED: c_int = 0;
pub const JPEG_REACHED_SOS: c_int = 1;
pub const JPEG_REACHED_EOI: c_int = 2;
pub const JPEG_HEADER_OK: c_int = 0;
pub const JPEG_HEADER_TABLES_ONLY: c_int = 1;

pub const JPEG_COM: c_int = 0xFE;
pub const JPEG_APP0: c_int = 0xE0;

pub const TRUE: c_int = 1;
pub const FALSE: c_int = 0;

pub const JERR_BAD_STATE: c_int = 1;
pub const JERR_UNKNOWN_MARKER: c_int = 2;
pub const JERR_NO_IMAGE: c_int = 3;
pub const JERR_TOO_LITTLE_DATA: c_int = 4;

pub const JWRN_ADOBE_XFORM: c_int = 1;

pub const JTRC_UNKNOWN_IDS: c_int = 1;

pub type boolean = c_int;
pub type jpeg_marker_parser_method = unsafe extern "C" fn(*mut c_void) -> boolean;

#[repr(C)]
pub struct jpeg_error_mgr {
    _private: c_int,
}

#[repr(C)]
pub struct jpeg_input_controller {
    pub reset_input_controller: Option<unsafe extern "C" fn(*mut c_void)>,
    pub consume_input: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub eoi_reached: boolean,
    pub has_multiple_scans: boolean,
}

#[repr(C)]
pub struct jpeg_marker_processor {
    pub process_COM: Option<jpeg_marker_parser_method>,
    pub process_APPn: [Option<jpeg_marker_parser_method>; 16],
}

#[repr(C)]
pub struct jpeg_source_mgr {
    pub init_source: Option<unsafe extern "C" fn(*mut c_void)>,
    pub term_source: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct jpeg_master {
    pub finish_output_pass: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct jpeg_component_info {
    pub component_id: c_int,
}

#[repr(C)]
pub struct jpeg_decompress_struct {
    pub err: *mut jpeg_error_mgr,
    pub is_decompressor: boolean,
    pub progress: *mut c_void,
    pub src: *mut jpeg_source_mgr,
    pub quant_tbl_ptrs: [*mut c_void; NUM_QUANT_TBLS],
    pub dc_huff_tbl_ptrs: [*mut c_void; NUM_HUFF_TBLS],
    pub ac_huff_tbl_ptrs: [*mut c_void; NUM_HUFF_TBLS],
    pub marker: *mut jpeg_marker_processor,
    pub inputctl: *mut jpeg_input_controller,
    pub master: *mut jpeg_master,
    pub num_components: c_int,
    pub jpeg_color_space: c_int,
    pub out_color_space: c_int,
    pub scale_num: c_int,
    pub scale_denom: c_int,
    pub output_gamma: f64,
    pub buffered_image: boolean,
    pub raw_data_out: boolean,
    pub dct_method: c_int,
    pub do_fancy_upsampling: boolean,
    pub do_block_smoothing: boolean,
    pub quantize_colors: boolean,
    pub dither_mode: c_int,
    pub two_pass_quantize: boolean,
    pub desired_number_of_colors: c_int,
    pub colormap: *mut c_void,
    pub enable_1pass_quant: boolean,
    pub enable_external_quant: boolean,
    pub enable_2pass_quant: boolean,
    pub global_state: c_int,
    pub output_scanline: c_int,
    pub output_height: c_int,
    pub saw_JFIF_marker: boolean,
    pub saw_Adobe_marker: boolean,
    pub Adobe_transform: c_int,
    pub comp_info: [jpeg_component_info; 256],
}

pub type j_decompress_ptr = *mut jpeg_decompress_struct;
pub type j_common_ptr = *mut c_void;

/* External functions from JPEG library */
extern "C" {
    pub fn jinit_memory_mgr(cinfo: j_common_ptr);
    pub fn jinit_marker_reader(cinfo: j_decompress_ptr);
    pub fn jinit_input_controller(cinfo: j_decompress_ptr);
    pub fn jpeg_destroy(cinfo: j_common_ptr);
    pub fn jpeg_abort(cinfo: j_common_ptr);
    pub fn jpeg_consume_input(cinfo: j_decompress_ptr) -> c_int;
}

/*
 * Initialization of a JPEG decompression object.
 * The error manager must already be set up (in case memory manager fails).
 */

pub unsafe extern "C" fn jpeg_create_decompress(cinfo: j_decompress_ptr) {
    let mut i: c_int;

    /* For debugging purposes, zero the whole master structure.
     * But error manager pointer is already there, so save and restore it.
     */
    {
        let err = (*cinfo).err;
        core::ptr::write_bytes(cinfo as *mut u8, 0, mem::size_of::<jpeg_decompress_struct>());
        (*cinfo).err = err;
    }
    (*cinfo).is_decompressor = TRUE;

    /* Initialize a memory manager instance for this object */
    jinit_memory_mgr(cinfo as j_common_ptr);

    /* Zero out pointers to permanent structures. */
    (*cinfo).progress = core::ptr::null_mut();
    (*cinfo).src = core::ptr::null_mut();

    i = 0;
    while i < NUM_QUANT_TBLS as c_int {
        (*cinfo).quant_tbl_ptrs[i as usize] = core::ptr::null_mut();
        i += 1;
    }

    i = 0;
    while i < NUM_HUFF_TBLS as c_int {
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

pub unsafe extern "C" fn jpeg_destroy_decompress(cinfo: j_decompress_ptr) {
    jpeg_destroy(cinfo as j_common_ptr); /* use common routine */
}


/*
 * Abort processing of a JPEG decompression operation,
 * but don't destroy the object itself.
 */

pub unsafe extern "C" fn jpeg_abort_decompress(cinfo: j_decompress_ptr) {
    jpeg_abort(cinfo as j_common_ptr); /* use common routine */
}


/*
 * Install a special processing method for COM or APPn markers.
 */

pub unsafe extern "C" fn jpeg_set_marker_processor(
    cinfo: j_decompress_ptr,
    marker_code: c_int,
    routine: jpeg_marker_parser_method,
) {
    if marker_code == JPEG_COM {
        (*(*cinfo).marker).process_COM = Some(routine);
    } else if marker_code >= JPEG_APP0 && marker_code <= JPEG_APP0 + 15 {
        (*(*cinfo).marker).process_APPn[(marker_code - JPEG_APP0) as usize] = Some(routine);
    } else {
        ERREXIT1(cinfo, JERR_UNKNOWN_MARKER, marker_code);
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
            if (*cinfo).saw_JFIF_marker != 0 {
                (*cinfo).jpeg_color_space = JCS_YCbCr; /* JFIF implies YCbCr */
            } else if (*cinfo).saw_Adobe_marker != 0 {
                match (*cinfo).Adobe_transform {
                    0 => {
                        (*cinfo).jpeg_color_space = JCS_RGB;
                    }
                    1 => {
                        (*cinfo).jpeg_color_space = JCS_YCbCr;
                    }
                    _ => {
                        WARNMS1(cinfo, JWRN_ADOBE_XFORM, (*cinfo).Adobe_transform);
                        (*cinfo).jpeg_color_space = JCS_YCbCr; /* assume it's YCbCr */
                    }
                }
            } else {
                /* Saw no special markers, try to guess from the component IDs */
                let cid0 = (*cinfo).comp_info[0].component_id;
                let cid1 = (*cinfo).comp_info[1].component_id;
                let cid2 = (*cinfo).comp_info[2].component_id;

                if cid0 == 1 && cid1 == 2 && cid2 == 3 {
                    (*cinfo).jpeg_color_space = JCS_YCbCr; /* assume JFIF w/out marker */
                } else if cid0 == 82 && cid1 == 71 && cid2 == 66 {
                    (*cinfo).jpeg_color_space = JCS_RGB; /* ASCII 'R', 'G', 'B' */
                } else {
                    TRACEMS3(cinfo, 1, JTRC_UNKNOWN_IDS, cid0, cid1, cid2);
                    (*cinfo).jpeg_color_space = JCS_YCbCr; /* assume it's YCbCr */
                }
            }
            /* Always guess RGB is proper output colorspace. */
            (*cinfo).out_color_space = JCS_RGB;
        }

        4 => {
            if (*cinfo).saw_Adobe_marker != 0 {
                match (*cinfo).Adobe_transform {
                    0 => {
                        (*cinfo).jpeg_color_space = JCS_CMYK;
                    }
                    2 => {
                        (*cinfo).jpeg_color_space = JCS_YCCK;
                    }
                    _ => {
                        WARNMS1(cinfo, JWRN_ADOBE_XFORM, (*cinfo).Adobe_transform);
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
    #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
    {
        (*cinfo).two_pass_quantize = TRUE;
    }
    #[cfg(not(feature = "QUANT_2PASS_SUPPORTED"))]
    {
        (*cinfo).two_pass_quantize = FALSE;
    }
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

pub unsafe extern "C" fn jpeg_read_header(cinfo: j_decompress_ptr, require_image: boolean) -> c_int {
    let mut retcode: c_int;

    if (*cinfo).global_state != DSTATE_START && (*cinfo).global_state != DSTATE_INHEADER {
        ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }

    retcode = jpeg_consume_input(cinfo);

    match retcode {
        JPEG_REACHED_SOS => {
            retcode = JPEG_HEADER_OK;
        }
        JPEG_REACHED_EOI => {
            if require_image != 0 {		/* Complain if application wanted an image */
                ERREXIT(cinfo, JERR_NO_IMAGE);
            }
            /* Reset to start state; it would be safer to require the application to
             * call jpeg_abort, but we can't change it now for compatibility reasons.
             * A side effect is to free any temporary memory (there shouldn't be any).
             */
            jpeg_abort(cinfo as j_common_ptr); /* sets state = DSTATE_START */
            retcode = JPEG_HEADER_TABLES_ONLY;
        }
        JPEG_SUSPENDED => {
            /* no work */
        }
        _ => {}
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

pub unsafe extern "C" fn jpeg_consume_input(cinfo: j_decompress_ptr) -> c_int {
    let mut retcode: c_int = JPEG_SUSPENDED;

    /* NB: every possible DSTATE value should be listed in this switch */
    match (*cinfo).global_state {
        DSTATE_START => {
            /* Start-of-datastream actions: reset appropriate modules */
            if let Some(reset_input_controller) = (*(*cinfo).inputctl).reset_input_controller {
                reset_input_controller(cinfo as j_common_ptr);
            }
            /* Initialize application's data source module */
            if let Some(init_source) = (*(*cinfo).src).init_source {
                init_source(cinfo as j_common_ptr);
            }
            (*cinfo).global_state = DSTATE_INHEADER;
            // FALLTHROUGH
            if let Some(consume_input) = (*(*cinfo).inputctl).consume_input {
                retcode = consume_input(cinfo as j_common_ptr);
            }
            if retcode == JPEG_REACHED_SOS { /* Found SOS, prepare to decompress */
                /* Set up default parameters based on header data */
                default_decompress_parms(cinfo);
                /* Set global state: ready for start_decompress */
                (*cinfo).global_state = DSTATE_READY;
            }
        }
        DSTATE_INHEADER => {
            if let Some(consume_input) = (*(*cinfo).inputctl).consume_input {
                retcode = consume_input(cinfo as j_common_ptr);
            }
            if retcode == JPEG_REACHED_SOS { /* Found SOS, prepare to decompress */
                /* Set up default parameters based on header data */
                default_decompress_parms(cinfo);
                /* Set global state: ready for start_decompress */
                (*cinfo).global_state = DSTATE_READY;
            }
        }
        DSTATE_READY => {
            /* Can't advance past first SOS until start_decompress is called */
            retcode = JPEG_REACHED_SOS;
        }
        DSTATE_PRELOAD | DSTATE_PRESCAN | DSTATE_SCANNING | DSTATE_RAW_OK | DSTATE_BUFIMAGE
        | DSTATE_BUFPOST | DSTATE_STOPPING => {
            if let Some(consume_input) = (*(*cinfo).inputctl).consume_input {
                retcode = consume_input(cinfo as j_common_ptr);
            }
        }
        _ => {
            ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
        }
    }
    retcode
}


/*
 * Have we finished reading the input file?
 */

pub unsafe extern "C" fn jpeg_input_complete(cinfo: j_decompress_ptr) -> boolean {
    /* Check for valid jpeg object */
    if (*cinfo).global_state < DSTATE_START || (*cinfo).global_state > DSTATE_STOPPING {
        ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    (*(*cinfo).inputctl).eoi_reached
}


/*
 * Is there more than one scan?
 */

pub unsafe extern "C" fn jpeg_has_multiple_scans(cinfo: j_decompress_ptr) -> boolean {
    /* Only valid after jpeg_read_header completes */
    if (*cinfo).global_state < DSTATE_READY || (*cinfo).global_state > DSTATE_STOPPING {
        ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
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

pub unsafe extern "C" fn jpeg_finish_decompress(cinfo: j_decompress_ptr) -> boolean {
    if ((*cinfo).global_state == DSTATE_SCANNING || (*cinfo).global_state == DSTATE_RAW_OK)
        && (*cinfo).buffered_image == 0
    {
        /* Terminate final pass of non-buffered mode */
        if (*cinfo).output_scanline < (*cinfo).output_height {
            ERREXIT(cinfo, JERR_TOO_LITTLE_DATA);
        }
        if let Some(finish_output_pass) = (*(*cinfo).master).finish_output_pass {
            finish_output_pass(cinfo as j_common_ptr);
        }
        (*cinfo).global_state = DSTATE_STOPPING;
    } else if (*cinfo).global_state == DSTATE_BUFIMAGE {
        /* Finishing after a buffered-image operation */
        (*cinfo).global_state = DSTATE_STOPPING;
    } else if (*cinfo).global_state != DSTATE_STOPPING {
        /* STOPPING = repeat call after a suspension, anything else is error */
        ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Read until EOI */
    while (*(*cinfo).inputctl).eoi_reached == 0 {
        if let Some(consume_input) = (*(*cinfo).inputctl).consume_input {
            if consume_input(cinfo as j_common_ptr) == JPEG_SUSPENDED {
                return FALSE;		/* Suspend, come back later */
            }
        }
    }
    /* Do final cleanup */
    if let Some(term_source) = (*(*cinfo).src).term_source {
        term_source(cinfo as j_common_ptr);
    }
    /* We can use jpeg_abort to release memory and reset global_state */
    jpeg_abort(cinfo as j_common_ptr);
    TRUE
}

/* Stub macros for error handling - these would normally call into the error manager */
#[inline]
unsafe fn ERREXIT(cinfo: j_decompress_ptr, code: c_int) {
    let _ = (cinfo, code);
    // Placeholder: in real JPEG library, this would call error handler
}

#[inline]
unsafe fn ERREXIT1(cinfo: j_decompress_ptr, code: c_int, p1: c_int) {
    let _ = (cinfo, code, p1);
    // Placeholder: in real JPEG library, this would call error handler with parameter
}

#[inline]
unsafe fn WARNMS1(cinfo: j_decompress_ptr, code: c_int, p1: c_int) {
    let _ = (cinfo, code, p1);
    // Placeholder: in real JPEG library, this would call warning handler
}

#[inline]
unsafe fn TRACEMS3(cinfo: j_decompress_ptr, level: c_int, code: c_int, p1: c_int, p2: c_int, p3: c_int) {
    let _ = (cinfo, level, code, p1, p2, p3);
    // Placeholder: in real JPEG library, this would call trace handler
}
