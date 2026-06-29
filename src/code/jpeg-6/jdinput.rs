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

// leave this as first line for PCH reasons...
//

use core::ffi::c_int;
use core::ptr::{self, addr_of, addr_of_mut};

/* JPEG library external types - these are declared as opaque since the full
   definitions are in the JPEG library headers (jinclude.h, jpeglib.h, jerror.h) */

pub type JDIMENSION = core::ffi::c_uint;
pub type boolean = c_int;
pub type UINT8 = u8;
pub type UINT16 = u16;

#[repr(C)]
pub struct j_decompress_struct {
    _opaque: [u8; 0],
}

pub type j_decompress_ptr = *mut j_decompress_struct;
pub type j_common_ptr = *mut j_decompress_struct;

/* JPEG component info structure */
#[repr(C)]
pub struct jpeg_component_info {
    pub component_index: c_int,
    pub component_id: UINT8,
    pub h_samp_factor: UINT8,
    pub v_samp_factor: UINT8,
    pub quant_tbl_no: UINT8,
    pub dc_tbl_no: UINT8,
    pub ac_tbl_no: UINT8,
    // Additional fields from full definition
    pub DCT_scaled_size: c_int,
    pub width_in_blocks: JDIMENSION,
    pub height_in_blocks: JDIMENSION,
    pub downsampled_width: JDIMENSION,
    pub downsampled_height: JDIMENSION,
    pub component_needed: boolean,
    pub MCU_width: c_int,
    pub MCU_height: c_int,
    pub MCU_blocks: c_int,
    pub MCU_sample_width: c_int,
    pub last_col_width: c_int,
    pub last_row_height: c_int,
    pub quant_table: *mut JQUANT_TBL,
}

/* Quantization table */
const DCTSIZE2: usize = 64;
#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [UINT16; DCTSIZE2],
}

/* Huffman table */
#[repr(C)]
pub struct JHUFF_TBL {
    pub bits: [UINT8; 17],
    pub huffval: [UINT8; 256],
}

/* Input control module */
#[repr(C)]
pub struct jpeg_input_controller {
    pub consume_input: Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>,
    pub reset_input_controller: Option<unsafe extern "C" fn(j_decompress_ptr) -> ()>,
    pub start_input_pass: Option<unsafe extern "C" fn(j_decompress_ptr) -> ()>,
    pub finish_input_pass: Option<unsafe extern "C" fn(j_decompress_ptr) -> ()>,
    pub has_multiple_scans: boolean,	/* True if file has multiple scans */
    pub eoi_reached: boolean,		/* True when EOI has been consumed */
}

/* Entropy decoding module */
#[repr(C)]
pub struct jpeg_entropy_decoder {
    pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr) -> ()>,
    pub decode_mcu: Option<unsafe extern "C" fn(j_decompress_ptr, *mut *mut i16) -> boolean>,
}

/* Coefficient buffer control module */
#[repr(C)]
pub struct jpeg_d_coef_controller {
    pub start_input_pass: Option<unsafe extern "C" fn(j_decompress_ptr) -> ()>,
    pub consume_data: Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>,
    pub start_output_pass: Option<unsafe extern "C" fn(j_decompress_ptr) -> ()>,
    pub decompress_data: Option<unsafe extern "C" fn(j_decompress_ptr, *mut *mut *mut u8) -> c_int>,
    pub coef_arrays: *mut *mut core::ffi::c_void,
}

/* Marker reader module */
#[repr(C)]
pub struct jpeg_marker_reader {
    pub reset_marker_reader: Option<unsafe extern "C" fn(j_decompress_ptr) -> ()>,
    pub read_markers: Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>,
    pub read_restart_marker: Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>,
    pub process_COM: Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>,
    pub process_APPn: [Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>; 16],
    pub saw_SOI: boolean,		/* found SOI? */
    pub saw_SOF: boolean,		/* found SOF? */
    pub next_restart_num: c_int,		/* next restart number expected (0-7) */
    pub discarded_bytes: c_int,	/* # of bytes skipped looking for a marker */
}

/* Memory manager (for allocation) */
#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(j_common_ptr, i32, usize) -> *mut core::ffi::c_void>,
}

/* Error handler */
#[repr(C)]
pub struct jpeg_error_mgr {
    pub reset_error_mgr: Option<unsafe extern "C" fn(j_common_ptr) -> ()>,
}

/* Main decompress structure (opaque, but we need to know field offsets) */
#[repr(C)]
pub struct j_decompress_info {
    pub image_height: JDIMENSION,
    pub image_width: JDIMENSION,
    pub data_precision: c_int,
    pub num_components: c_int,
    pub comp_info: *mut jpeg_component_info,
    pub max_h_samp_factor: c_int,
    pub max_v_samp_factor: c_int,
    pub min_DCT_scaled_size: c_int,
    pub total_iMCU_rows: JDIMENSION,
    pub comps_in_scan: c_int,
    pub cur_comp_info: [*mut jpeg_component_info; 4], /* MAX_COMPS_IN_SCAN */
    pub MCUs_per_row: JDIMENSION,
    pub MCU_rows_in_scan: JDIMENSION,
    pub blocks_in_MCU: c_int,
    pub MCU_membership: [c_int; 10], /* D_MAX_BLOCKS_IN_MCU */
    pub progressive_mode: boolean,
    pub output_scan_number: c_int,
    pub input_scan_number: c_int,
    pub coef_bits: *mut c_int,
    pub inputctl: *mut jpeg_input_controller,
    pub entropy: *mut jpeg_entropy_decoder,
    pub coef: *mut jpeg_d_coef_controller,
    pub marker: *mut jpeg_marker_reader,
    pub mem: *mut jpeg_memory_mgr,
    pub err: *mut jpeg_error_mgr,
    pub quant_tbl_ptrs: [*mut JQUANT_TBL; 4], /* NUM_QUANT_TBLS */
}

/* Private state */
#[repr(C)]
pub struct my_input_controller {
    pub pub_: jpeg_input_controller,	/* public fields */
    pub inheaders: boolean,		/* TRUE until first SOS is reached */
}

pub type my_inputctl_ptr = *mut my_input_controller;


/* Forward declarations */
unsafe extern "C" fn consume_markers(cinfo: j_decompress_ptr) -> c_int;

/* JPEG constants */
const JPEG_MAX_DIMENSION: c_int = 65500;
const BITS_IN_JSAMPLE: c_int = 8;
const MAX_COMPONENTS: c_int = 10;
const MAX_SAMP_FACTOR: c_int = 4;
const DCTSIZE: c_int = 8;
const MAX_COMPS_IN_SCAN: c_int = 4;
const D_MAX_BLOCKS_IN_MCU: c_int = 10;
const NUM_QUANT_TBLS: c_int = 4;

/* JPEG pool constants */
const JPOOL_IMAGE: i32 = 1;
const JPOOL_PERMANENT: i32 = 0;

/* JPEG return codes */
const JPEG_SUSPENDED: c_int = 0;
const JPEG_REACHED_SOS: c_int = 1;
const JPEG_REACHED_EOI: c_int = 2;

/* Macro replacements */
#[inline]
fn MAX(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
}

#[inline]
unsafe fn MEMCOPY(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, size: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, size);
}

#[inline]
const fn SIZEOF<T>() -> usize {
    core::mem::size_of::<T>()
}

/* Stub for error macros - these would call error handlers in the actual JPEG library */
#[inline]
unsafe fn ERREXIT(_cinfo: j_decompress_ptr, _err_code: c_int) {
    /* In actual implementation, this would trigger error handling */
}

#[inline]
unsafe fn ERREXIT1(_cinfo: j_decompress_ptr, _err_code: c_int, _param1: c_int) {
    /* In actual implementation, this would trigger error handling with parameter */
}

#[inline]
unsafe fn ERREXIT2(_cinfo: j_decompress_ptr, _err_code: c_int, _param1: c_int, _param2: c_int) {
    /* In actual implementation, this would trigger error handling with 2 parameters */
}

/* External function stubs */
extern "C" {
    pub fn jdiv_round_up(a: i32, b: i32) -> c_int;
}

/*
 * Routines to calculate various quantities related to the size of the image.
 */

unsafe fn initial_setup(cinfo: j_decompress_ptr)
/* Called once, when first SOS marker is reached */
{
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    /* Make sure image isn't bigger than I can handle */
    if ((*cinfo).image_height as i32) > JPEG_MAX_DIMENSION ||
        ((*cinfo).image_width as i32) > JPEG_MAX_DIMENSION
    {
        ERREXIT1(cinfo, 1, JPEG_MAX_DIMENSION as c_int);
    }

    /* For now, precision must match compiled-in value... */
    if (*cinfo).data_precision != BITS_IN_JSAMPLE {
        ERREXIT1(cinfo, 2, (*cinfo).data_precision);
    }

    /* Check that number of components won't exceed internal array sizes */
    if (*cinfo).num_components > MAX_COMPONENTS {
        ERREXIT2(cinfo, 3, (*cinfo).num_components, MAX_COMPONENTS);
    }

    /* Compute maximum sampling factors; check factor validity */
    (*cinfo).max_h_samp_factor = 1;
    (*cinfo).max_v_samp_factor = 1;
    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        if (*compptr).h_samp_factor as c_int <= 0 ||
            (*compptr).h_samp_factor as c_int > MAX_SAMP_FACTOR ||
            (*compptr).v_samp_factor as c_int <= 0 ||
            (*compptr).v_samp_factor as c_int > MAX_SAMP_FACTOR
        {
            ERREXIT(cinfo, 4);
        }
        (*cinfo).max_h_samp_factor =
            MAX((*cinfo).max_h_samp_factor, (*compptr).h_samp_factor as c_int);
        (*cinfo).max_v_samp_factor =
            MAX((*cinfo).max_v_samp_factor, (*compptr).v_samp_factor as c_int);
        ci += 1;
        compptr = compptr.offset(1);
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
            ((*cinfo).image_width as i32) * ((*compptr).h_samp_factor as i32),
            ((*cinfo).max_h_samp_factor * DCTSIZE),
        ) as JDIMENSION;
        (*compptr).height_in_blocks = jdiv_round_up(
            ((*cinfo).image_height as i32) * ((*compptr).v_samp_factor as i32),
            ((*cinfo).max_v_samp_factor * DCTSIZE),
        ) as JDIMENSION;
        /* downsampled_width and downsampled_height will also be overridden by
         * jdmaster.c if we are doing full decompression.  The transcoder library
         * doesn't use these values, but the calling application might.
         */
        /* Size in samples */
        (*compptr).downsampled_width = jdiv_round_up(
            ((*cinfo).image_width as i32) * ((*compptr).h_samp_factor as i32),
            (*cinfo).max_h_samp_factor,
        ) as JDIMENSION;
        (*compptr).downsampled_height = jdiv_round_up(
            ((*cinfo).image_height as i32) * ((*compptr).v_samp_factor as i32),
            (*cinfo).max_v_samp_factor,
        ) as JDIMENSION;
        /* Mark component needed, until color conversion says otherwise */
        (*compptr).component_needed = 1; /* TRUE */
        /* Mark no quantization table yet saved for component */
        (*compptr).quant_table = ptr::null_mut();
        ci += 1;
        compptr = compptr.offset(1);
    }

    /* Compute number of fully interleaved MCU rows. */
    (*cinfo).total_iMCU_rows = jdiv_round_up(
        (*cinfo).image_height as i32,
        (*cinfo).max_v_samp_factor * DCTSIZE,
    ) as JDIMENSION;

    /* Decide whether file contains multiple scans */
    if (*cinfo).comps_in_scan < (*cinfo).num_components || (*cinfo).progressive_mode != 0 {
        (*(*cinfo).inputctl).has_multiple_scans = 1; /* TRUE */
    } else {
        (*(*cinfo).inputctl).has_multiple_scans = 0; /* FALSE */
    }
}


unsafe fn per_scan_setup(cinfo: j_decompress_ptr)
/* Do computations that are needed before processing a JPEG scan */
/* cinfo->comps_in_scan and cinfo->cur_comp_info[] were set from SOS marker */
{
    let mut ci: c_int;
    let mut mcublks: c_int;
    let mut tmp: c_int;
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
        tmp = ((*compptr).height_in_blocks % (*compptr).v_samp_factor as JDIMENSION) as c_int;
        if tmp == 0 {
            tmp = (*compptr).v_samp_factor as c_int;
        }
        (*compptr).last_row_height = tmp;

        /* Prepare array describing MCU composition */
        (*cinfo).blocks_in_MCU = 1;
        (*cinfo).MCU_membership[0] = 0;
    } else {
        /* Interleaved (multi-component) scan */
        if (*cinfo).comps_in_scan <= 0 || (*cinfo).comps_in_scan > MAX_COMPS_IN_SCAN {
            ERREXIT2(cinfo, 5, (*cinfo).comps_in_scan, MAX_COMPS_IN_SCAN);
        }

        /* Overall image size in MCUs */
        (*cinfo).MCUs_per_row = jdiv_round_up(
            (*cinfo).image_width as i32,
            (*cinfo).max_h_samp_factor * DCTSIZE,
        ) as JDIMENSION;
        (*cinfo).MCU_rows_in_scan = jdiv_round_up(
            (*cinfo).image_height as i32,
            (*cinfo).max_v_samp_factor * DCTSIZE,
        ) as JDIMENSION;

        (*cinfo).blocks_in_MCU = 0;

        ci = 0;
        while ci < (*cinfo).comps_in_scan {
            compptr = (*cinfo).cur_comp_info[ci as usize];
            /* Sampling factors give # of blocks of component in each MCU */
            (*compptr).MCU_width = (*compptr).h_samp_factor as c_int;
            (*compptr).MCU_height = (*compptr).v_samp_factor as c_int;
            (*compptr).MCU_blocks = (*compptr).MCU_width * (*compptr).MCU_height;
            (*compptr).MCU_sample_width = (*compptr).MCU_width * (*compptr).DCT_scaled_size;
            /* Figure number of non-dummy blocks in last MCU column & row */
            tmp = ((*compptr).width_in_blocks % (*compptr).MCU_width as JDIMENSION) as c_int;
            if tmp == 0 {
                tmp = (*compptr).MCU_width;
            }
            (*compptr).last_col_width = tmp;
            tmp = ((*compptr).height_in_blocks % (*compptr).MCU_height as JDIMENSION) as c_int;
            if tmp == 0 {
                tmp = (*compptr).MCU_height;
            }
            (*compptr).last_row_height = tmp;
            /* Prepare array describing MCU composition */
            mcublks = (*compptr).MCU_blocks;
            if (*cinfo).blocks_in_MCU + mcublks > D_MAX_BLOCKS_IN_MCU {
                ERREXIT(cinfo, 6);
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

unsafe fn latch_quant_tables(cinfo: j_decompress_ptr)
{
    let mut ci: c_int;
    let mut qtblno: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut qtbl: *mut JQUANT_TBL;

    ci = 0;
    while ci < (*cinfo).comps_in_scan {
        compptr = (*cinfo).cur_comp_info[ci as usize];
        /* No work if we already saved Q-table for this component */
        if (*compptr).quant_table != ptr::null_mut() {
            ci += 1;
            continue;
        }
        /* Make sure specified quantization table is present */
        qtblno = (*compptr).quant_tbl_no as c_int;
        if qtblno < 0
            || qtblno >= NUM_QUANT_TBLS
            || (*cinfo).quant_tbl_ptrs[qtblno as usize] == ptr::null_mut()
        {
            ERREXIT1(cinfo, 7, qtblno);
        }
        /* OK, save away the quantization table */
        qtbl = (*(*cinfo).mem).alloc_small
            .unwrap()(cinfo as j_common_ptr, JPOOL_IMAGE, SIZEOF::<JQUANT_TBL>())
            as *mut JQUANT_TBL;
        MEMCOPY(
            qtbl as *mut core::ffi::c_void,
            (*cinfo).quant_tbl_ptrs[qtblno as usize] as *const core::ffi::c_void,
            SIZEOF::<JQUANT_TBL>(),
        );
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

unsafe extern "C" fn start_input_pass(cinfo: j_decompress_ptr)
{
    per_scan_setup(cinfo);
    latch_quant_tables(cinfo);
    (*(*cinfo).entropy).start_pass.unwrap()(cinfo);
    (*(*cinfo).coef).start_input_pass.unwrap()(cinfo);
    (*(*cinfo).inputctl).consume_input = (*(*cinfo).coef).consume_data;
}


/*
 * Finish up after inputting a compressed-data scan.
 * This is called by the coefficient controller after it's read all
 * the expected data of the scan.
 */

unsafe extern "C" fn finish_input_pass(cinfo: j_decompress_ptr)
{
    (*(*cinfo).inputctl).consume_input = Some(consume_markers);
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

unsafe extern "C" fn consume_markers(cinfo: j_decompress_ptr) -> c_int
{
    let inputctl: my_inputctl_ptr = (*cinfo).inputctl as my_inputctl_ptr;
    let mut val: c_int;

    if (*inputctl).pub_.eoi_reached != 0 {
        /* After hitting EOI, read no further */
        return JPEG_REACHED_EOI;
    }

    val = (*(*cinfo).marker).read_markers.unwrap()(cinfo);

    match val {
        JPEG_REACHED_SOS => {
            /* Found SOS */
            if (*inputctl).inheaders != 0 {
                /* 1st SOS */
                initial_setup(cinfo);
                (*inputctl).inheaders = 0; /* FALSE */
                /* Note: start_input_pass must be called by jdmaster.c
                 * before any more input can be consumed.  jdapi.c is
                 * responsible for enforcing this sequencing.
                 */
            } else {
                /* 2nd or later SOS marker */
                if (*(*cinfo).inputctl).has_multiple_scans == 0 {
                    ERREXIT(cinfo, 8); /* Oops, I wasn't expecting this! */
                }
                start_input_pass(cinfo);
            }
        }
        JPEG_REACHED_EOI => {
            /* Found EOI */
            (*inputctl).pub_.eoi_reached = 1; /* TRUE */
            if (*inputctl).inheaders != 0 {
                /* Tables-only datastream, apparently */
                if (*(*cinfo).marker).saw_SOF != 0 {
                    ERREXIT(cinfo, 9);
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
            /* Do nothing */
        }
        _ => {
            /* Other cases not handled */
        }
    }

    val
}


/*
 * Reset state to begin a fresh datastream.
 */

unsafe extern "C" fn reset_input_controller(cinfo: j_decompress_ptr)
{
    let inputctl: my_inputctl_ptr = (*cinfo).inputctl as my_inputctl_ptr;

    (*inputctl).pub_.consume_input = Some(consume_markers);
    (*inputctl).pub_.has_multiple_scans = 0; /* FALSE - "unknown" would be better */
    (*inputctl).pub_.eoi_reached = 0;        /* FALSE */
    (*inputctl).inheaders = 1;               /* TRUE */
    /* Reset other modules */
    (*(*cinfo).err).reset_error_mgr.unwrap()(cinfo as j_common_ptr);
    (*(*cinfo).marker).reset_marker_reader.unwrap()(cinfo);
    /* Reset progression state -- would be cleaner if entropy decoder did this */
    (*cinfo).coef_bits = ptr::null_mut();
}


/*
 * Initialize the input controller module.
 * This is called only once, when the decompression object is created.
 */

pub unsafe extern "C" fn jinit_input_controller(cinfo: j_decompress_ptr)
{
    let mut inputctl: my_inputctl_ptr;

    /* Create subobject in permanent pool */
    inputctl = (*(*cinfo).mem).alloc_small
        .unwrap()(
            cinfo as j_common_ptr,
            JPOOL_PERMANENT,
            SIZEOF::<my_input_controller>(),
        ) as my_inputctl_ptr;
    (*cinfo).inputctl = addr_of_mut!((*inputctl).pub_);
    /* Initialize method pointers */
    (*inputctl).pub_.consume_input = Some(consume_markers);
    (*inputctl).pub_.reset_input_controller = Some(reset_input_controller);
    (*inputctl).pub_.start_input_pass = Some(start_input_pass);
    (*inputctl).pub_.finish_input_pass = Some(finish_input_pass);
    /* Initialize state: can't use reset_input_controller since we don't
     * want to try to reset other modules yet.
     */
    (*inputctl).pub_.has_multiple_scans = 0; /* FALSE - "unknown" would be better */
    (*inputctl).pub_.eoi_reached = 0;        /* FALSE */
    (*inputctl).inheaders = 1;               /* TRUE */
}
