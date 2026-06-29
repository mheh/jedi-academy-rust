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

#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::c_int;
use core::ffi::c_void;

// ============================================================================
// Type definitions
// ============================================================================

pub type JDIMENSION = u32;
pub type boolean = u8;

const FALSE: boolean = 0;
const TRUE: boolean = 1;

// Opaque JPEG types
pub struct j_decompress_info;
pub struct j_common_info;
pub struct jpeg_input_controller;
pub struct jpeg_marker_reader;
pub struct jpeg_entropy_decoder;
pub struct jpeg_d_coef_controller;
pub struct jpeg_error_mgr;
pub struct jpeg_memory_mgr;

pub type j_decompress_ptr = *mut j_decompress_info;
pub type j_common_ptr = *mut j_common_info;

// ============================================================================
// JPEG quantization table structure
// ============================================================================

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; 64],   /* quantization step for each coefficient */
    pub sent_table: boolean,   /* TRUE when table has been output */
}

// ============================================================================
// JPEG component information structure
// ============================================================================

#[repr(C)]
pub struct jpeg_component_info {
    pub component_id: c_int,        /* identifier for this component (0..255) */
    pub component_index: c_int,     /* its index in SOF or cinfo->comp_info[] */
    pub h_samp_factor: c_int,       /* horizontal sampling factor (1..4) */
    pub v_samp_factor: c_int,       /* vertical sampling factor (1..4) */
    pub quant_tbl_no: c_int,        /* quantization table selector (0..3) */
    pub dc_tbl_no: c_int,           /* DC entropy table selector (0..3) */
    pub ac_tbl_no: c_int,           /* AC entropy table selector (0..3) */
    pub width_in_blocks: JDIMENSION,     /* component's width in DCT blocks */
    pub height_in_blocks: JDIMENSION,    /* component's height in DCT blocks */
    pub DCT_scaled_size: c_int,     /* size of output from IDCT block */
    pub downsampled_width: JDIMENSION,   /* actual width in samples */
    pub downsampled_height: JDIMENSION,  /* actual height in samples */
    pub component_needed: boolean,  /* do we need the value of this component? */
    pub MCU_width: c_int,           /* number of blocks per MCU, horizontally */
    pub MCU_height: c_int,          /* number of blocks per MCU, vertically */
    pub MCU_blocks: c_int,          /* MCU_width * MCU_height */
    pub MCU_sample_width: c_int,    /* MCU width in samples, MCU_width*DCT_scaled_size */
    pub last_col_width: c_int,      /* # of non-dummy blocks across in last MCU */
    pub last_row_height: c_int,     /* # of non-dummy blocks down in last MCU */
    pub quant_table: *mut JQUANT_TBL,   /* saved quantization table for component */
    pub dct_table: *mut c_void,     /* private per-component storage */
}

// ============================================================================
// Error codes used in this module
// ============================================================================

const JERR_IMAGE_TOO_BIG: c_int = 38;
const JERR_BAD_PRECISION: c_int = 12;
const JERR_COMPONENT_COUNT: c_int = 22;
const JERR_BAD_SAMPLING: c_int = 15;
const JERR_BAD_MCU_SIZE: c_int = 10;
const JERR_NO_QUANT_TABLE: c_int = 48;
const JERR_SOF_NO_SOS: c_int = 54;
const JERR_EOI_EXPECTED: c_int = 32;

// ============================================================================
// Configuration constants
// ============================================================================

const JPEG_MAX_DIMENSION: JDIMENSION = 65500u32;
const BITS_IN_JSAMPLE: c_int = 8;
const MAX_COMPONENTS: c_int = 10;
const MAX_SAMP_FACTOR: c_int = 4;
const DCTSIZE: c_int = 8;
const MAX_COMPS_IN_SCAN: c_int = 4;
const D_MAX_BLOCKS_IN_MCU: c_int = 10;
const NUM_QUANT_TBLS: usize = 4;

// JPEG return codes
const JPEG_SUSPENDED: c_int = 0;
const JPEG_REACHED_SOS: c_int = 1;
const JPEG_REACHED_EOI: c_int = 2;

// Memory pool codes
const JPOOL_PERMANENT: c_int = 0;
const JPOOL_IMAGE: c_int = 1;

// ============================================================================
// Macro implementations
// ============================================================================

/// MAX(a, b) - Maximum of two values
#[inline]
fn MAX(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
}

/// SIZEOF(T) - Get size of type T
#[inline]
const fn SIZEOF<T>() -> usize {
    core::mem::size_of::<T>()
}

/// MEMCOPY(dest, src, size) - Copy memory region
#[inline]
unsafe fn MEMCOPY(dest: *mut c_void, src: *const c_void, size: usize) {
    core::ptr::copy_nonoverlapping(src, dest, size);
}

/// Error exit handler stub
#[inline]
fn ERREXIT(_cinfo: j_decompress_ptr, _code: c_int) {
    /* Stub - error handler called through err->error_exit */
}

/// Error exit handler with one parameter
#[inline]
fn ERREXIT1(_cinfo: j_decompress_ptr, _code: c_int, _p1: c_int) {
    /* Stub - error handler called through err->error_exit */
}

/// Error exit handler with two parameters
#[inline]
fn ERREXIT2(_cinfo: j_decompress_ptr, _code: c_int, _p1: c_int, _p2: c_int) {
    /* Stub - error handler called through err->error_exit */
}

/// jdiv_round_up(a, b) - Divide a by b with rounding up
/// Translates to: (((a) + (b) - 1) / (b))
#[inline]
fn jdiv_round_up(a: i64, b: i64) -> i64 {
    (a + b - 1) / b
}

// ============================================================================
// Private input controller state
// ============================================================================

/// Private state for input controller
#[repr(C)]
struct my_input_controller {
    pub r#pub: jpeg_input_controller,  /* public fields */
    inheaders: boolean,               /* TRUE until first SOS is reached */
}

type my_inputctl_ptr = *mut my_input_controller;

// ============================================================================
// Stub implementations of JPEG library controller structures
// ============================================================================

/// Input controller - public fields accessible from decompressor
#[repr(C)]
pub struct jpeg_input_controller {
    pub consume_input: Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>,
    pub reset_input_controller: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    pub start_input_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    pub finish_input_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    pub has_multiple_scans: boolean,
    pub eoi_reached: boolean,
}

/// Marker reader
#[repr(C)]
pub struct jpeg_marker_reader {
    pub read_markers: Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>,
    pub reset_marker_reader: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    pub saw_SOF: boolean,
    _rest: [u8; 0],
}

/// Entropy decoder
#[repr(C)]
pub struct jpeg_entropy_decoder {
    pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    _rest: [u8; 0],
}

/// Coefficient controller
#[repr(C)]
pub struct jpeg_d_coef_controller {
    pub start_input_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    pub consume_data: Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>,
    pub start_output_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    pub decompress_data: Option<unsafe extern "C" fn(j_decompress_ptr, *mut c_void) -> c_int>,
    pub coef_arrays: *mut *mut c_void,
}

/// Error manager
#[repr(C)]
pub struct jpeg_error_mgr {
    pub reset_error_mgr: Option<unsafe extern "C" fn(j_common_ptr)>,
    _rest: [u8; 0],
}

/// Memory manager
#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void>,
    _rest: [u8; 0],
}

/// Partial definition of decompressor struct for this module
#[repr(C)]
pub struct j_decompress_info {
    pub image_height: JDIMENSION,
    pub image_width: JDIMENSION,
    pub data_precision: c_int,
    pub num_components: c_int,
    pub comp_info: *mut jpeg_component_info,
    pub max_h_samp_factor: c_int,
    pub max_v_samp_factor: c_int,
    pub min_DCT_scaled_size: JDIMENSION,
    pub total_iMCU_rows: JDIMENSION,
    pub comps_in_scan: c_int,
    pub cur_comp_info: [*mut jpeg_component_info; MAX_COMPONENTS as usize],
    pub MCUs_per_row: JDIMENSION,
    pub MCU_rows_in_scan: JDIMENSION,
    pub blocks_in_MCU: c_int,
    pub MCU_membership: [c_int; D_MAX_BLOCKS_IN_MCU as usize],
    pub output_scan_number: c_int,
    pub input_scan_number: c_int,
    pub progressive_mode: boolean,
    pub inputctl: *mut jpeg_input_controller,
    pub marker: *mut jpeg_marker_reader,
    pub entropy: *mut jpeg_entropy_decoder,
    pub coef: *mut jpeg_d_coef_controller,
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub quant_tbl_ptrs: [*mut JQUANT_TBL; NUM_QUANT_TBLS],
    pub coef_bits: *mut c_int,
    _rest: [u8; 0],
}

pub type j_common_info = j_decompress_info;

// ============================================================================
// Routines to calculate various quantities related to the size of the image.
// ============================================================================

/// Called once, when first SOS marker is reached
fn initial_setup(cinfo: j_decompress_ptr) {
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    unsafe {
        /* Make sure image isn't bigger than I can handle */
        if ((*cinfo).image_height as i64) > (JPEG_MAX_DIMENSION as i64) ||
            ((*cinfo).image_width as i64) > (JPEG_MAX_DIMENSION as i64)
        {
            ERREXIT1(cinfo, JERR_IMAGE_TOO_BIG, JPEG_MAX_DIMENSION as c_int);
        }

        /* For now, precision must match compiled-in value... */
        if (*cinfo).data_precision != BITS_IN_JSAMPLE {
            ERREXIT1(cinfo, JERR_BAD_PRECISION, (*cinfo).data_precision);
        }

        /* Check that number of components won't exceed internal array sizes */
        if (*cinfo).num_components > MAX_COMPONENTS {
            ERREXIT2(cinfo, JERR_COMPONENT_COUNT, (*cinfo).num_components, MAX_COMPONENTS);
        }

        /* Compute maximum sampling factors; check factor validity */
        (*cinfo).max_h_samp_factor = 1;
        (*cinfo).max_v_samp_factor = 1;
        ci = 0;
        compptr = (*cinfo).comp_info;
        while ci < (*cinfo).num_components {
            if (*compptr).h_samp_factor <= 0 || (*compptr).h_samp_factor > MAX_SAMP_FACTOR ||
                (*compptr).v_samp_factor <= 0 || (*compptr).v_samp_factor > MAX_SAMP_FACTOR
            {
                ERREXIT(cinfo, JERR_BAD_SAMPLING);
            }
            (*cinfo).max_h_samp_factor = MAX((*cinfo).max_h_samp_factor, (*compptr).h_samp_factor);
            (*cinfo).max_v_samp_factor = MAX((*cinfo).max_v_samp_factor, (*compptr).v_samp_factor);
            ci += 1;
            compptr = compptr.add(1);
        }

        /* We initialize DCT_scaled_size and min_DCT_scaled_size to DCTSIZE.
         * In the full decompressor, this will be overridden by jdmaster.c;
         * but in the transcoder, jdmaster.c is not used, so we must do it here.
         */
        (*cinfo).min_DCT_scaled_size = DCTSIZE as JDIMENSION;

        /* Compute dimensions of components */
        ci = 0;
        compptr = (*cinfo).comp_info;
        while ci < (*cinfo).num_components {
            (*compptr).DCT_scaled_size = DCTSIZE;
            /* Size in DCT blocks */
            (*compptr).width_in_blocks = jdiv_round_up(
                ((*cinfo).image_width as i64) * ((*compptr).h_samp_factor as i64),
                (((*cinfo).max_h_samp_factor as i64) * (DCTSIZE as i64))
            ) as JDIMENSION;
            (*compptr).height_in_blocks = jdiv_round_up(
                ((*cinfo).image_height as i64) * ((*compptr).v_samp_factor as i64),
                (((*cinfo).max_v_samp_factor as i64) * (DCTSIZE as i64))
            ) as JDIMENSION;
            /* downsampled_width and downsampled_height will also be overridden by
             * jdmaster.c if we are doing full decompression.  The transcoder library
             * doesn't use these values, but the calling application might.
             */
            /* Size in samples */
            (*compptr).downsampled_width = jdiv_round_up(
                ((*cinfo).image_width as i64) * ((*compptr).h_samp_factor as i64),
                ((*cinfo).max_h_samp_factor as i64)
            ) as JDIMENSION;
            (*compptr).downsampled_height = jdiv_round_up(
                ((*cinfo).image_height as i64) * ((*compptr).v_samp_factor as i64),
                ((*cinfo).max_v_samp_factor as i64)
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
            ((*cinfo).image_height as i64),
            (((*cinfo).max_v_samp_factor as i64) * (DCTSIZE as i64))
        ) as JDIMENSION;

        /* Decide whether file contains multiple scans */
        if (*cinfo).comps_in_scan < (*cinfo).num_components || (*cinfo).progressive_mode != FALSE {
            (*(*cinfo).inputctl).has_multiple_scans = TRUE;
        } else {
            (*(*cinfo).inputctl).has_multiple_scans = FALSE;
        }
    }
}

/// Do computations that are needed before processing a JPEG scan
/// cinfo->comps_in_scan and cinfo->cur_comp_info[] were set from SOS marker
fn per_scan_setup(cinfo: j_decompress_ptr) {
    let mut ci: c_int;
    let mut mcublks: c_int;
    let mut tmp: c_int;
    let mut compptr: *mut jpeg_component_info;

    unsafe {
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
            tmp = ((*compptr).height_in_blocks % ((*compptr).v_samp_factor as JDIMENSION)) as c_int;
            if tmp == 0 { tmp = (*compptr).v_samp_factor; }
            (*compptr).last_row_height = tmp;

            /* Prepare array describing MCU composition */
            (*cinfo).blocks_in_MCU = 1;
            (*cinfo).MCU_membership[0] = 0;
        } else {
            /* Interleaved (multi-component) scan */
            if (*cinfo).comps_in_scan <= 0 || (*cinfo).comps_in_scan > MAX_COMPS_IN_SCAN {
                ERREXIT2(cinfo, JERR_COMPONENT_COUNT, (*cinfo).comps_in_scan, MAX_COMPS_IN_SCAN);
            }

            /* Overall image size in MCUs */
            (*cinfo).MCUs_per_row = jdiv_round_up(
                ((*cinfo).image_width as i64),
                (((*cinfo).max_h_samp_factor as i64) * (DCTSIZE as i64))
            ) as JDIMENSION;
            (*cinfo).MCU_rows_in_scan = jdiv_round_up(
                ((*cinfo).image_height as i64),
                (((*cinfo).max_v_samp_factor as i64) * (DCTSIZE as i64))
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
                tmp = ((*compptr).width_in_blocks % ((*compptr).MCU_width as JDIMENSION)) as c_int;
                if tmp == 0 { tmp = (*compptr).MCU_width; }
                (*compptr).last_col_width = tmp;
                tmp = ((*compptr).height_in_blocks % ((*compptr).MCU_height as JDIMENSION)) as c_int;
                if tmp == 0 { tmp = (*compptr).MCU_height; }
                (*compptr).last_row_height = tmp;
                /* Prepare array describing MCU composition */
                mcublks = (*compptr).MCU_blocks;
                if (*cinfo).blocks_in_MCU + mcublks > D_MAX_BLOCKS_IN_MCU {
                    ERREXIT(cinfo, JERR_BAD_MCU_SIZE);
                }
                while mcublks > 0 {
                    mcublks -= 1;
                    (*cinfo).MCU_membership[(*cinfo).blocks_in_MCU as usize] = ci;
                    (*cinfo).blocks_in_MCU += 1;
                }

                ci += 1;
            }
        }
    }
}

// ============================================================================
// Save away a copy of the Q-table referenced by each component present
// in the current scan, unless already saved during a prior scan.
//
// In a multiple-scan JPEG file, the encoder could assign different components
// the same Q-table slot number, but change table definitions between scans
// so that each component uses a different Q-table.  (The IJG encoder is not
// currently capable of doing this, but other encoders might.)  Since we want
// to be able to dequantize all the components at the end of the file, this
// means that we have to save away the table actually used for each component.
// We do this by copying the table at the start of the first scan containing
// the component.
// The JPEG spec prohibits the encoder from changing the contents of a Q-table
// slot between scans of a component using that slot.  If the encoder does so
// anyway, this decoder will simply use the Q-table values that were current
// at the start of the first scan for the component.
//
// The decompressor output side looks only at the saved quant tables,
// not at the current Q-table slots.
// ============================================================================

fn latch_quant_tables(cinfo: j_decompress_ptr) {
    let mut ci: c_int;
    let mut qtblno: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut qtbl: *mut JQUANT_TBL;

    unsafe {
        ci = 0;
        while ci < (*cinfo).comps_in_scan {
            compptr = (*cinfo).cur_comp_info[ci as usize];
            /* No work if we already saved Q-table for this component */
            if (*compptr).quant_table != core::ptr::null_mut() {
                ci += 1;
                continue;
            }
            /* Make sure specified quantization table is present */
            qtblno = (*compptr).quant_tbl_no;
            if qtblno < 0 || qtblno >= NUM_QUANT_TBLS as c_int ||
                (*cinfo).quant_tbl_ptrs[qtblno as usize] == core::ptr::null_mut()
            {
                ERREXIT1(cinfo, JERR_NO_QUANT_TABLE, qtblno);
            }
            /* OK, save away the quantization table */
            let mem_fn = (*(*cinfo).mem).alloc_small.unwrap();
            qtbl = mem_fn(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                SIZEOF::<JQUANT_TBL>()
            ) as *mut JQUANT_TBL;
            MEMCOPY(qtbl as *mut c_void, (*cinfo).quant_tbl_ptrs[qtblno as usize] as *const c_void, SIZEOF::<JQUANT_TBL>());
            (*compptr).quant_table = qtbl;

            ci += 1;
        }
    }
}

// ============================================================================
// Initialize the input modules to read a scan of compressed data.
// The first call to this is done by jdmaster.c after initializing
// the entire decompressor (during jpeg_start_decompress).
// Subsequent calls come from consume_markers, below.
// ============================================================================

fn start_input_pass(cinfo: j_decompress_ptr) {
    unsafe {
        per_scan_setup(cinfo);
        latch_quant_tables(cinfo);
        let entropy_fn = (*(*cinfo).entropy).start_pass.unwrap();
        entropy_fn(cinfo);
        let coef_fn = (*(*cinfo).coef).start_input_pass.unwrap();
        coef_fn(cinfo);
        (*(*cinfo).inputctl).consume_input = (*(*cinfo).coef).consume_data;
    }
}

// ============================================================================
// Finish up after inputting a compressed-data scan.
// This is called by the coefficient controller after it's read all
// the expected data of the scan.
// ============================================================================

fn finish_input_pass(cinfo: j_decompress_ptr) {
    unsafe {
        (*(*cinfo).inputctl).consume_input = Some(consume_markers);
    }
}

// ============================================================================
// Read JPEG markers before, between, or after compressed-data scans.
// Change state as necessary when a new scan is reached.
// Return value is JPEG_SUSPENDED, JPEG_REACHED_SOS, or JPEG_REACHED_EOI.
//
// The consume_input method pointer points either here or to the
// coefficient controller's consume_data routine, depending on whether
// we are reading a compressed data segment or inter-segment markers.
// ============================================================================

fn consume_markers(cinfo: j_decompress_ptr) -> c_int {
    let mut inputctl: my_inputctl_ptr;
    let mut val: c_int;

    unsafe {
        inputctl = (*cinfo).inputctl as my_inputctl_ptr;

        if (*inputctl).r#pub.eoi_reached != FALSE {
            /* After hitting EOI, read no further */
            return JPEG_REACHED_EOI;
        }

        let read_markers_fn = (*(*cinfo).marker).read_markers.unwrap();
        val = read_markers_fn(cinfo);

        match val {
            JPEG_REACHED_SOS => {
                /* Found SOS */
                if (*inputctl).inheaders != FALSE {
                    /* 1st SOS */
                    initial_setup(cinfo);
                    (*inputctl).inheaders = FALSE;
                    /* Note: start_input_pass must be called by jdmaster.c
                     * before any more input can be consumed.  jdapi.c is
                     * responsible for enforcing this sequencing.
                     */
                } else {
                    /* 2nd or later SOS marker */
                    if (*inputctl).r#pub.has_multiple_scans == FALSE {
                        ERREXIT(cinfo, JERR_EOI_EXPECTED); /* Oops, I wasn't expecting this! */
                    }
                    start_input_pass(cinfo);
                }
            }
            JPEG_REACHED_EOI => {
                /* Found EOI */
                (*inputctl).r#pub.eoi_reached = TRUE;
                if (*inputctl).inheaders != FALSE {
                    /* Tables-only datastream, apparently */
                    if (*(*cinfo).marker).saw_SOF != FALSE {
                        ERREXIT(cinfo, JERR_SOF_NO_SOS);
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
            JPEG_SUSPENDED => {
                /* Suspended due to lack of input data */
            }
            _ => {}
        }

        val
    }
}

// ============================================================================
// Reset state to begin a fresh datastream.
// ============================================================================

fn reset_input_controller(cinfo: j_decompress_ptr) {
    let mut inputctl: my_inputctl_ptr;

    unsafe {
        inputctl = (*cinfo).inputctl as my_inputctl_ptr;

        (*inputctl).r#pub.consume_input = Some(consume_markers);
        (*inputctl).r#pub.has_multiple_scans = FALSE; /* "unknown" would be better */
        (*inputctl).r#pub.eoi_reached = FALSE;
        (*inputctl).inheaders = TRUE;
        /* Reset other modules */
        let reset_error_fn = (*(*cinfo).err).reset_error_mgr.unwrap();
        reset_error_fn(cinfo as j_common_ptr);
        let reset_marker_fn = (*(*cinfo).marker).reset_marker_reader.unwrap();
        reset_marker_fn(cinfo);
        /* Reset progression state -- would be cleaner if entropy decoder did this */
        (*cinfo).coef_bits = core::ptr::null_mut();
    }
}

// ============================================================================
// Initialize the input controller module.
// This is called only once, when the decompression object is created.
// ============================================================================

pub fn jinit_input_controller(cinfo: j_decompress_ptr) {
    let mut inputctl: my_inputctl_ptr;

    unsafe {
        /* Create subobject in permanent pool */
        let mem_fn = (*(*cinfo).mem).alloc_small.unwrap();
        inputctl = mem_fn(
            cinfo as j_common_ptr,
            JPOOL_PERMANENT,
            SIZEOF::<my_input_controller>()
        ) as my_inputctl_ptr;

        (*cinfo).inputctl = &mut (*inputctl).r#pub;
        /* Initialize method pointers */
        (*inputctl).r#pub.consume_input = Some(consume_markers);
        (*inputctl).r#pub.reset_input_controller = Some(reset_input_controller);
        (*inputctl).r#pub.start_input_pass = Some(start_input_pass);
        (*inputctl).r#pub.finish_input_pass = Some(finish_input_pass);
        /* Initialize state: can't use reset_input_controller since we don't
         * want to try to reset other modules yet.
         */
        (*inputctl).r#pub.has_multiple_scans = FALSE; /* "unknown" would be better */
        (*inputctl).r#pub.eoi_reached = FALSE;
        (*inputctl).inheaders = TRUE;
    }
}
