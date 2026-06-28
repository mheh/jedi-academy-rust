/*
 * jdcoefct.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains the coefficient buffer controller for decompression.
 * This controller is the top level of the JPEG decompressor proper.
 * The coefficient buffer lies between entropy decoding and inverse-DCT steps.
 *
 * In buffered-image mode, this controller is the interface between
 * input-oriented processing and output-oriented processing.
 * Also, the input side (only) is used when reading a file for transcoding.
 */

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::c_int;
use core::mem;

/* -- JPEG Type Stubs -- */
/* These are placeholder declarations for JPEG library types.
 * In a complete port, these would come from the ported jpeglib.h headers.
 */

pub type JDIMENSION = u32;
pub type JBLOCK = [i16; 64];
pub type JCOEF = i16;
pub type JCOEFPTR = *mut JCOEF;
pub type JBLOCKROW = *mut JBLOCK;
pub type JBLOCKARRAY = *mut JBLOCKROW;
pub type JSAMPARRAY = *mut *mut u8;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type INT32 = i32;
pub type boolean = c_int;

/* Forward struct declarations */
#[repr(C)]
pub struct jpeg_d_coef_controller {
    pub start_input_pass: extern "C" fn(*mut j_decompress_struct),
    pub start_output_pass: extern "C" fn(*mut j_decompress_struct),
    pub consume_data: extern "C" fn(*mut j_decompress_struct) -> c_int,
    pub decompress_data: extern "C" fn(*mut j_decompress_struct, JSAMPIMAGE) -> c_int,
    pub coef_arrays: *mut c_int,
}

#[repr(C)]
pub struct jpeg_component_info {
    pub component_index: c_int,
    pub component_needed: c_int,
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
    pub width_in_blocks: JDIMENSION,
    pub height_in_blocks: JDIMENSION,
    pub MCU_blocks: c_int,
    pub MCU_width: c_int,
    pub MCU_height: c_int,
    pub MCU_sample_width: JDIMENSION,
    pub DCT_scaled_size: c_int,
    pub last_col_width: c_int,
    pub last_row_height: c_int,
    pub quant_table: *mut JQUANT_TBL,
}

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; 64],
}

#[repr(C)]
pub struct jpeg_entropy_decoder {
    pub decode_mcu: extern "C" fn(*mut j_decompress_struct, *mut JBLOCKROW) -> boolean,
}

#[repr(C)]
pub struct jpeg_inverse_dct {
    pub inverse_DCT: [extern "C" fn(*mut j_decompress_struct, *mut jpeg_component_info, JCOEFPTR, JSAMPARRAY, JDIMENSION); 10],
}

#[repr(C)]
pub struct jpeg_input_controller {
    pub consume_input: extern "C" fn(*mut j_decompress_struct) -> c_int,
    pub finish_input_pass: extern "C" fn(*mut j_decompress_struct),
    pub eoi_reached: boolean,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: extern "C" fn(*mut j_common_struct, c_int, usize) -> *mut c_int,
    pub alloc_large: extern "C" fn(*mut j_common_struct, c_int, usize) -> *mut c_int,
    pub access_virt_barray: extern "C" fn(*mut j_common_struct, *mut c_int, JDIMENSION, JDIMENSION, boolean) -> JBLOCKARRAY,
    pub request_virt_barray: extern "C" fn(*mut j_common_struct, c_int, boolean, JDIMENSION, JDIMENSION, JDIMENSION) -> *mut c_int,
}

#[repr(C)]
pub struct j_common_struct {
    pub err: *mut c_int,
    pub mem: *mut jpeg_memory_mgr,
}

#[repr(C)]
pub struct j_decompress_struct {
    pub common: j_common_struct,
    pub coef: *mut jpeg_d_coef_controller,
    pub entropy: *mut jpeg_entropy_decoder,
    pub idct: *mut jpeg_inverse_dct,
    pub inputctl: *mut jpeg_input_controller,
    pub mem: *mut jpeg_memory_mgr,
    pub cur_comp_info: [*mut jpeg_component_info; 10],
    pub comp_info: *mut jpeg_component_info,
    pub num_components: c_int,
    pub comps_in_scan: c_int,
    pub blocks_in_MCU: c_int,
    pub MCUs_per_row: JDIMENSION,
    pub total_iMCU_rows: JDIMENSION,
    pub input_iMCU_row: JDIMENSION,
    pub output_iMCU_row: JDIMENSION,
    pub input_scan_number: c_int,
    pub output_scan_number: c_int,
    pub progressive_mode: boolean,
    pub do_block_smoothing: boolean,
    pub Ss: c_int,
    pub coef_bits: *mut *mut c_int,
}

pub type j_decompress_ptr = *mut j_decompress_struct;
pub type j_common_ptr = *mut j_common_struct;
pub type inverse_DCT_method_ptr = extern "C" fn(*mut j_decompress_struct, *mut jpeg_component_info, JCOEFPTR, JSAMPARRAY, JDIMENSION);
pub type jvirt_barray_ptr = *mut c_int;

/* Constants and macros */
const D_MAX_BLOCKS_IN_MCU: usize = 10;
const MAX_COMPONENTS: usize = 10;
const MAX_COMPS_IN_SCAN: usize = 4;
const JPOOL_IMAGE: c_int = 1;
const JPEG_SUSPENDED: c_int = 0;
const JPEG_ROW_COMPLETED: c_int = 1;
const JPEG_SCAN_COMPLETED: c_int = 2;
const SAVED_COEFS: usize = 6;  /* we save coef_bits[0..5] */

/* External function stubs */
extern "C" {
    fn jzero_far(ptr: *mut c_int, size: usize);
    fn jcopy_block_row(src: JBLOCKROW, dst: JBLOCKROW, blocks: JDIMENSION);
    fn jround_up(a: i64, b: i64) -> i64;
    fn ERREXIT(cinfo: j_decompress_ptr, code: c_int);
}

/* Private buffer controller object */

#[repr(C)]
pub struct my_coef_controller {
    pub pub_: jpeg_d_coef_controller, /* public fields */

    /* These variables keep track of the current location of the input side. */
    /* cinfo->input_iMCU_row is also used for this. */
    pub MCU_ctr: JDIMENSION,        /* counts MCUs processed in current row */
    pub MCU_vert_offset: c_int,     /* counts MCU rows within iMCU row */
    pub MCU_rows_per_iMCU_row: c_int, /* number of such rows needed */

    /* The output side's location is represented by cinfo->output_iMCU_row. */

    /* In single-pass modes, it's sufficient to buffer just one MCU.
     * We allocate a workspace of D_MAX_BLOCKS_IN_MCU coefficient blocks,
     * and let the entropy decoder write into that workspace each time.
     * (On 80x86, the workspace is FAR even though it's not really very big;
     * this is to keep the module interfaces unchanged when a large coefficient
     * buffer is necessary.)
     * In multi-pass modes, this array points to the current MCU's blocks
     * within the virtual arrays; it is used only by the input side.
     */
    pub MCU_buffer: [JBLOCKROW; D_MAX_BLOCKS_IN_MCU],

    #[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
    /* In multi-pass modes, we need a virtual block array for each component. */
    pub whole_image: [jvirt_barray_ptr; MAX_COMPONENTS],

    #[cfg(feature = "BLOCK_SMOOTHING_SUPPORTED")]
    /* When doing block smoothing, we latch coefficient Al values here */
    pub coef_bits_latch: *mut c_int,
}

pub type my_coef_ptr = *mut my_coef_controller;

/* Forward declarations */
extern "C" {
    fn decompress_onepass(cinfo: j_decompress_ptr, output_buf: JSAMPIMAGE) -> c_int;
    #[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
    fn decompress_data(cinfo: j_decompress_ptr, output_buf: JSAMPIMAGE) -> c_int;
    #[cfg(feature = "BLOCK_SMOOTHING_SUPPORTED")]
    fn smoothing_ok(cinfo: j_decompress_ptr) -> boolean;
    #[cfg(feature = "BLOCK_SMOOTHING_SUPPORTED")]
    fn decompress_smooth_data(cinfo: j_decompress_ptr, output_buf: JSAMPIMAGE) -> c_int;
}


fn start_iMCU_row(cinfo: j_decompress_ptr)
/* Reset within-iMCU-row counters for a new row (input side) */
{
    unsafe {
        let coef = (*cinfo).coef as my_coef_ptr;

        /* In an interleaved scan, an MCU row is the same as an iMCU row.
         * In a noninterleaved scan, an iMCU row has v_samp_factor MCU rows.
         * But at the bottom of the image, process only what's left.
         */
        if (*cinfo).comps_in_scan > 1 {
            (*coef).MCU_rows_per_iMCU_row = 1;
        } else {
            if (*cinfo).input_iMCU_row < ((*cinfo).total_iMCU_rows - 1) {
                (*coef).MCU_rows_per_iMCU_row = (*(*(*cinfo).cur_comp_info[0])).v_samp_factor;
            } else {
                (*coef).MCU_rows_per_iMCU_row = (*(*(*cinfo).cur_comp_info[0])).last_row_height;
            }
        }

        (*coef).MCU_ctr = 0;
        (*coef).MCU_vert_offset = 0;
    }
}


/*
 * Initialize for an input processing pass.
 */

fn start_input_pass(cinfo: j_decompress_ptr)
{
    unsafe {
        (*cinfo).input_iMCU_row = 0;
        start_iMCU_row(cinfo);
    }
}


/*
 * Initialize for an output processing pass.
 */

fn start_output_pass(cinfo: j_decompress_ptr)
{
    #[cfg(feature = "BLOCK_SMOOTHING_SUPPORTED")]
    {
        unsafe {
            let coef = (*cinfo).coef as my_coef_ptr;

            /* If multipass, check to see whether to use block smoothing on this pass */
            if (*coef).pub_.coef_arrays != core::ptr::null_mut() {
                if (*cinfo).do_block_smoothing != 0 && smoothing_ok(cinfo) != 0 {
                    (*coef).pub_.decompress_data = decompress_smooth_data;
                } else {
                    (*coef).pub_.decompress_data = decompress_data;
                }
            }
            (*cinfo).output_iMCU_row = 0;
        }
    }

    #[cfg(not(feature = "BLOCK_SMOOTHING_SUPPORTED"))]
    {
        unsafe {
            (*cinfo).output_iMCU_row = 0;
        }
    }
}


/*
 * Decompress and return some data in the single-pass case.
 * Always attempts to emit one fully interleaved MCU row ("iMCU" row).
 * Input and output must run in lockstep since we have only a one-MCU buffer.
 * Return value is JPEG_ROW_COMPLETED, JPEG_SCAN_COMPLETED, or JPEG_SUSPENDED.
 *
 * NB: output_buf contains a plane for each component in image.
 * For single pass, this is the same as the components in the scan.
 */

fn decompress_onepass(cinfo: j_decompress_ptr, output_buf: JSAMPIMAGE) -> c_int
{
    unsafe {
        let coef = (*cinfo).coef as my_coef_ptr;
        let mut MCU_col_num: JDIMENSION;  /* index of current MCU within row */
        let last_MCU_col = (*cinfo).MCUs_per_row - 1;
        let last_iMCU_row = (*cinfo).total_iMCU_rows - 1;
        let mut blkn: c_int;
        let mut ci: c_int;
        let mut xindex: c_int;
        let mut yindex: c_int;
        let mut yoffset: c_int;
        let mut useful_width: c_int;
        let mut output_ptr: JSAMPARRAY;
        let mut output_col: JDIMENSION;
        let mut compptr: *mut jpeg_component_info;
        let mut inverse_DCT: inverse_DCT_method_ptr;

        /* Loop to process as much as one whole iMCU row */
        let mut yoffset_iter = (*coef).MCU_vert_offset;
        while yoffset_iter < (*coef).MCU_rows_per_iMCU_row {
            yoffset = yoffset_iter;
            MCU_col_num = (*coef).MCU_ctr;
            while MCU_col_num <= last_MCU_col {
                /* Try to fetch an MCU.  Entropy decoder expects buffer to be zeroed. */
                jzero_far(
                    (*coef).MCU_buffer[0] as *mut c_int,
                    ((*cinfo).blocks_in_MCU * mem::size_of::<JBLOCK>() as c_int) as usize,
                );
                if ((*(*cinfo).entropy).decode_mcu)(cinfo, (*coef).MCU_buffer.as_mut_ptr()) == 0 {
                    /* Suspension forced; update state counters and exit */
                    (*coef).MCU_vert_offset = yoffset;
                    (*coef).MCU_ctr = MCU_col_num;
                    return JPEG_SUSPENDED;
                }
                /* Determine where data should go in output_buf and do the IDCT thing.
                 * We skip dummy blocks at the right and bottom edges (but blkn gets
                 * incremented past them!).  Note the inner loop relies on having
                 * allocated the MCU_buffer[] blocks sequentially.
                 */
                blkn = 0;   /* index of current DCT block within MCU */
                ci = 0;
                while ci < (*cinfo).comps_in_scan {
                    compptr = (*(*cinfo).cur_comp_info[ci as usize]);
                    /* Don't bother to IDCT an uninteresting component. */
                    if (*compptr).component_needed == 0 {
                        blkn += (*compptr).MCU_blocks;
                        ci += 1;
                        continue;
                    }
                    inverse_DCT = (*(*(*cinfo).idct).inverse_DCT[(*compptr).component_index as usize]);
                    useful_width = if MCU_col_num < last_MCU_col {
                        (*compptr).MCU_width
                    } else {
                        (*compptr).last_col_width
                    };
                    output_ptr = (*output_buf.add(ci as usize));
                    output_ptr = output_ptr.add((yoffset as usize) * (*compptr).DCT_scaled_size as usize);
                    let start_col = (MCU_col_num as c_int) * (*compptr).MCU_sample_width as c_int;
                    yindex = 0;
                    while yindex < (*compptr).MCU_height {
                        if (*cinfo).input_iMCU_row < last_iMCU_row ||
                            (yoffset + yindex) < (*compptr).last_row_height
                        {
                            output_col = start_col as JDIMENSION;
                            xindex = 0;
                            while xindex < useful_width {
                                inverse_DCT(
                                    cinfo,
                                    compptr,
                                    (*coef).MCU_buffer[(blkn + xindex) as usize] as JCOEFPTR,
                                    output_ptr,
                                    output_col,
                                );
                                output_col = (output_col as c_int + (*compptr).DCT_scaled_size) as JDIMENSION;
                                xindex += 1;
                            }
                        }
                        blkn += (*compptr).MCU_width;
                        output_ptr = output_ptr.add((*compptr).DCT_scaled_size as usize);
                        yindex += 1;
                    }
                    ci += 1;
                }
                MCU_col_num += 1;
            }
            /* Completed an MCU row, but perhaps not an iMCU row */
            (*coef).MCU_ctr = 0;
            yoffset_iter += 1;
        }
        /* Completed the iMCU row, advance counters for next one */
        (*cinfo).output_iMCU_row += 1;
        (*cinfo).input_iMCU_row += 1;
        if (*cinfo).input_iMCU_row < (*cinfo).total_iMCU_rows {
            start_iMCU_row(cinfo);
            return JPEG_ROW_COMPLETED;
        }
        /* Completed the scan */
        ((*(*cinfo).inputctl).finish_input_pass)(cinfo);
        return JPEG_SCAN_COMPLETED;
    }
}


/*
 * Dummy consume-input routine for single-pass operation.
 */

fn dummy_consume_data(cinfo: j_decompress_ptr) -> c_int
{
    return JPEG_SUSPENDED;  /* Always indicate nothing was done */
}


#[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
{
    /*
     * Consume input data and store it in the full-image coefficient buffer.
     * We read as much as one fully interleaved MCU row ("iMCU" row) per call,
     * ie, v_samp_factor block rows for each component in the scan.
     * Return value is JPEG_ROW_COMPLETED, JPEG_SCAN_COMPLETED, or JPEG_SUSPENDED.
     */

    fn consume_data(cinfo: j_decompress_ptr) -> c_int
    {
        unsafe {
            let coef = (*cinfo).coef as my_coef_ptr;
            let mut MCU_col_num: JDIMENSION;  /* index of current MCU within row */
            let mut blkn: c_int;
            let mut ci: c_int;
            let mut xindex: c_int;
            let mut yindex: c_int;
            let mut yoffset: c_int;
            let mut start_col: JDIMENSION;
            let mut buffer: [JBLOCKARRAY; MAX_COMPS_IN_SCAN] = [core::ptr::null_mut(); MAX_COMPS_IN_SCAN];
            let mut buffer_ptr: JBLOCKROW;
            let mut compptr: *mut jpeg_component_info;

            /* Align the virtual buffers for the components used in this scan. */
            ci = 0;
            while ci < (*cinfo).comps_in_scan {
                compptr = (*(*cinfo).cur_comp_info[ci as usize]);
                buffer[ci as usize] = ((*(*(*cinfo).mem).access_virt_barray)(
                    cinfo as j_common_ptr,
                    (*coef).whole_image[(*compptr).component_index as usize],
                    (*cinfo).input_iMCU_row * ((*compptr).v_samp_factor as u32),
                    (*compptr).v_samp_factor as u32,
                    1,
                ));
                /* Note: entropy decoder expects buffer to be zeroed,
                 * but this is handled automatically by the memory manager
                 * because we requested a pre-zeroed array.
                 */
                ci += 1;
            }

            /* Loop to process one whole iMCU row */
            let mut yoffset_iter = (*coef).MCU_vert_offset;
            while yoffset_iter < (*coef).MCU_rows_per_iMCU_row {
                yoffset = yoffset_iter;
                MCU_col_num = (*coef).MCU_ctr;
                while MCU_col_num < (*cinfo).MCUs_per_row {
                    /* Construct list of pointers to DCT blocks belonging to this MCU */
                    blkn = 0;   /* index of current DCT block within MCU */
                    ci = 0;
                    while ci < (*cinfo).comps_in_scan {
                        compptr = (*(*cinfo).cur_comp_info[ci as usize]);
                        start_col = (MCU_col_num as c_int * (*compptr).MCU_width) as JDIMENSION;
                        yindex = 0;
                        while yindex < (*compptr).MCU_height {
                            buffer_ptr = *buffer[ci as usize].add((yindex as usize + yoffset as usize));
                            buffer_ptr = buffer_ptr.add(start_col as usize);
                            xindex = 0;
                            while xindex < (*compptr).MCU_width {
                                (*coef).MCU_buffer[blkn as usize] = buffer_ptr;
                                blkn += 1;
                                buffer_ptr = buffer_ptr.add(1);
                                xindex += 1;
                            }
                            yindex += 1;
                        }
                        ci += 1;
                    }
                    /* Try to fetch the MCU. */
                    if ((*(*cinfo).entropy).decode_mcu)(cinfo, (*coef).MCU_buffer.as_mut_ptr()) == 0 {
                        /* Suspension forced; update state counters and exit */
                        (*coef).MCU_vert_offset = yoffset;
                        (*coef).MCU_ctr = MCU_col_num;
                        return JPEG_SUSPENDED;
                    }
                    MCU_col_num += 1;
                }
                /* Completed an MCU row, but perhaps not an iMCU row */
                (*coef).MCU_ctr = 0;
                yoffset_iter += 1;
            }
            /* Completed the iMCU row, advance counters for next one */
            (*cinfo).input_iMCU_row += 1;
            if (*cinfo).input_iMCU_row < (*cinfo).total_iMCU_rows {
                start_iMCU_row(cinfo);
                return JPEG_ROW_COMPLETED;
            }
            /* Completed the scan */
            ((*(*cinfo).inputctl).finish_input_pass)(cinfo);
            return JPEG_SCAN_COMPLETED;
        }
    }


    /*
     * Decompress and return some data in the multi-pass case.
     * Always attempts to emit one fully interleaved MCU row ("iMCU" row).
     * Return value is JPEG_ROW_COMPLETED, JPEG_SCAN_COMPLETED, or JPEG_SUSPENDED.
     *
     * NB: output_buf contains a plane for each component in image.
     */

    fn decompress_data(cinfo: j_decompress_ptr, output_buf: JSAMPIMAGE) -> c_int
    {
        unsafe {
            let coef = (*cinfo).coef as my_coef_ptr;
            let last_iMCU_row = (*cinfo).total_iMCU_rows - 1;
            let mut block_num: JDIMENSION;
            let mut ci: c_int;
            let mut block_row: c_int;
            let mut block_rows: c_int;
            let mut buffer: JBLOCKARRAY;
            let mut buffer_ptr: JBLOCKROW;
            let mut output_ptr: JSAMPARRAY;
            let mut output_col: JDIMENSION;
            let mut compptr: *mut jpeg_component_info;
            let mut inverse_DCT: inverse_DCT_method_ptr;

            /* Force some input to be done if we are getting ahead of the input. */
            while (*cinfo).input_scan_number < (*cinfo).output_scan_number ||
                  ((*cinfo).input_scan_number == (*cinfo).output_scan_number &&
                   (*cinfo).input_iMCU_row <= (*cinfo).output_iMCU_row)
            {
                if ((*(*cinfo).inputctl).consume_input)(cinfo) == JPEG_SUSPENDED {
                    return JPEG_SUSPENDED;
                }
            }

            /* OK, output from the virtual arrays. */
            ci = 0;
            compptr = (*cinfo).comp_info;
            while ci < (*cinfo).num_components {
                /* Don't bother to IDCT an uninteresting component. */
                if (*compptr).component_needed == 0 {
                    ci += 1;
                    compptr = compptr.add(1);
                    continue;
                }
                /* Align the virtual buffer for this component. */
                buffer = ((*(*(*cinfo).mem).access_virt_barray)(
                    cinfo as j_common_ptr,
                    (*coef).whole_image[ci as usize],
                    (*cinfo).output_iMCU_row * ((*compptr).v_samp_factor as u32),
                    (*compptr).v_samp_factor as u32,
                    0,
                ));
                /* Count non-dummy DCT block rows in this iMCU row. */
                if (*cinfo).output_iMCU_row < last_iMCU_row {
                    block_rows = (*compptr).v_samp_factor;
                } else {
                    /* NB: can't use last_row_height here; it is input-side-dependent! */
                    block_rows = ((*compptr).height_in_blocks % ((*compptr).v_samp_factor as u32)) as c_int;
                    if block_rows == 0 {
                        block_rows = (*compptr).v_samp_factor;
                    }
                }
                inverse_DCT = (*(*(*cinfo).idct).inverse_DCT[ci as usize]);
                output_ptr = *output_buf.add(ci as usize);
                /* Loop over all DCT blocks to be processed. */
                block_row = 0;
                while block_row < block_rows {
                    buffer_ptr = *buffer.add(block_row as usize);
                    output_col = 0;
                    block_num = 0;
                    while block_num < (*compptr).width_in_blocks {
                        inverse_DCT(
                            cinfo,
                            compptr,
                            buffer_ptr as JCOEFPTR,
                            output_ptr,
                            output_col,
                        );
                        buffer_ptr = buffer_ptr.add(1);
                        output_col = (output_col as c_int + (*compptr).DCT_scaled_size) as JDIMENSION;
                        block_num += 1;
                    }
                    output_ptr = output_ptr.add((*compptr).DCT_scaled_size as usize);
                    block_row += 1;
                }
                ci += 1;
                compptr = compptr.add(1);
            }

            (*cinfo).output_iMCU_row += 1;
            if (*cinfo).output_iMCU_row < (*cinfo).total_iMCU_rows {
                return JPEG_ROW_COMPLETED;
            }
            return JPEG_SCAN_COMPLETED;
        }
    }
}

/* D_MULTISCAN_FILES_SUPPORTED */


#[cfg(feature = "BLOCK_SMOOTHING_SUPPORTED")]
{
    /*
     * This code applies interblock smoothing as described by section K.8
     * of the JPEG standard: the first 5 AC coefficients are estimated from
     * the DC values of a DCT block and its 8 neighboring blocks.
     * We apply smoothing only for progressive JPEG decoding, and only if
     * the coefficients it can estimate are not yet known to full precision.
     */

    /*
     * Determine whether block smoothing is applicable and safe.
     * We also latch the current states of the coef_bits[] entries for the
     * AC coefficients; otherwise, if the input side of the decompressor
     * advances into a new scan, we might think the coefficients are known
     * more accurately than they really are.
     */

    fn smoothing_ok(cinfo: j_decompress_ptr) -> boolean
    {
        unsafe {
            let coef = (*cinfo).coef as my_coef_ptr;
            let mut smoothing_useful: boolean = 0;
            let mut ci: c_int;
            let mut coefi: c_int;
            let mut compptr: *mut jpeg_component_info;
            let mut qtable: *mut JQUANT_TBL;
            let mut coef_bits: *mut c_int;
            let mut coef_bits_latch: *mut c_int;

            if (*cinfo).progressive_mode == 0 || (*cinfo).coef_bits == core::ptr::null_mut() {
                return 0; /* FALSE */
            }

            /* Allocate latch area if not already done */
            if (*coef).coef_bits_latch == core::ptr::null_mut() {
                (*coef).coef_bits_latch = ((*(*(*cinfo).mem).alloc_small)(
                    cinfo as j_common_ptr,
                    JPOOL_IMAGE,
                    ((*cinfo).num_components as usize * (SAVED_COEFS * mem::size_of::<c_int>())),
                )) as *mut c_int;
            }
            coef_bits_latch = (*coef).coef_bits_latch;

            ci = 0;
            compptr = (*cinfo).comp_info;
            while ci < (*cinfo).num_components {
                /* All components' quantization values must already be latched. */
                qtable = (*compptr).quant_table;
                if qtable == core::ptr::null_mut() {
                    return 0; /* FALSE */
                }
                /* Verify DC & first 5 AC quantizers are nonzero to avoid zero-divide. */
                coefi = 0;
                while coefi <= 5 {
                    if (*qtable).quantval[coefi as usize] == 0 {
                        return 0; /* FALSE */
                    }
                    coefi += 1;
                }
                /* DC values must be at least partly known for all components. */
                coef_bits = *(*cinfo).coef_bits.add(ci as usize);
                if *coef_bits < 0 {
                    return 0; /* FALSE */
                }
                /* Block smoothing is helpful if some AC coefficients remain inaccurate. */
                coefi = 1;
                while coefi <= 5 {
                    *coef_bits_latch.add((coefi) as usize) = *coef_bits.add(coefi as usize);
                    if *coef_bits.add(coefi as usize) != 0 {
                        smoothing_useful = 1; /* TRUE */
                    }
                    coefi += 1;
                }
                coef_bits_latch = coef_bits_latch.add(SAVED_COEFS);
                ci += 1;
                compptr = compptr.add(1);
            }

            return smoothing_useful;
        }
    }


    /*
     * Variant of decompress_data for use when doing block smoothing.
     */

    fn decompress_smooth_data(cinfo: j_decompress_ptr, output_buf: JSAMPIMAGE) -> c_int
    {
        unsafe {
            let coef = (*cinfo).coef as my_coef_ptr;
            let last_iMCU_row = (*cinfo).total_iMCU_rows - 1;
            let mut block_num: JDIMENSION;
            let mut last_block_column: JDIMENSION;
            let mut ci: c_int;
            let mut block_row: c_int;
            let mut block_rows: c_int;
            let mut access_rows: c_int;
            let mut buffer: JBLOCKARRAY;
            let mut buffer_ptr: JBLOCKROW;
            let mut prev_block_row: JBLOCKROW;
            let mut next_block_row: JBLOCKROW;
            let mut output_ptr: JSAMPARRAY;
            let mut output_col: JDIMENSION;
            let mut compptr: *mut jpeg_component_info;
            let mut inverse_DCT: inverse_DCT_method_ptr;
            let mut first_row: boolean;
            let mut last_row: boolean;
            let mut workspace: JBLOCK = [0; 64];
            let mut coef_bits: *mut c_int;
            let mut quanttbl: *mut JQUANT_TBL;
            let mut Q00: INT32;
            let mut Q01: INT32;
            let mut Q02: INT32;
            let mut Q10: INT32;
            let mut Q11: INT32;
            let mut Q20: INT32;
            let mut num: INT32;
            let mut DC1: c_int;
            let mut DC2: c_int;
            let mut DC3: c_int;
            let mut DC4: c_int;
            let mut DC5: c_int;
            let mut DC6: c_int;
            let mut DC7: c_int;
            let mut DC8: c_int;
            let mut DC9: c_int;
            let mut Al: c_int;
            let mut pred: c_int;

            /* Force some input to be done if we are getting ahead of the input. */
            while (*cinfo).input_scan_number <= (*cinfo).output_scan_number &&
                  (*(*cinfo).inputctl).eoi_reached == 0
            {
                if (*cinfo).input_scan_number == (*cinfo).output_scan_number {
                    /* If input is working on current scan, we ordinarily want it to
                     * have completed the current row.  But if input scan is DC,
                     * we want it to keep one row ahead so that next block row's DC
                     * values are up to date.
                     */
                    let delta: JDIMENSION = if (*cinfo).Ss == 0 { 1 } else { 0 };
                    if (*cinfo).input_iMCU_row > (*cinfo).output_iMCU_row + delta {
                        break;
                    }
                }
                if ((*(*cinfo).inputctl).consume_input)(cinfo) == JPEG_SUSPENDED {
                    return JPEG_SUSPENDED;
                }
            }

            /* OK, output from the virtual arrays. */
            ci = 0;
            compptr = (*cinfo).comp_info;
            while ci < (*cinfo).num_components {
                /* Don't bother to IDCT an uninteresting component. */
                if (*compptr).component_needed == 0 {
                    ci += 1;
                    compptr = compptr.add(1);
                    continue;
                }
                /* Count non-dummy DCT block rows in this iMCU row. */
                if (*cinfo).output_iMCU_row < last_iMCU_row {
                    block_rows = (*compptr).v_samp_factor;
                    access_rows = block_rows * 2;  /* this and next iMCU row */
                    last_row = 0; /* FALSE */
                } else {
                    /* NB: can't use last_row_height here; it is input-side-dependent! */
                    block_rows = ((*compptr).height_in_blocks % ((*compptr).v_samp_factor as u32)) as c_int;
                    if block_rows == 0 {
                        block_rows = (*compptr).v_samp_factor;
                    }
                    access_rows = block_rows;  /* this iMCU row only */
                    last_row = 1; /* TRUE */
                }
                /* Align the virtual buffer for this component. */
                if (*cinfo).output_iMCU_row > 0 {
                    access_rows += (*compptr).v_samp_factor;  /* prior iMCU row too */
                    buffer = ((*(*(*cinfo).mem).access_virt_barray)(
                        cinfo as j_common_ptr,
                        (*coef).whole_image[ci as usize],
                        (((*cinfo).output_iMCU_row as i32 - 1) as u32 * ((*compptr).v_samp_factor as u32)),
                        access_rows as u32,
                        0,
                    ));
                    buffer = buffer.add((*compptr).v_samp_factor as usize);  /* point to current iMCU row */
                    first_row = 0;  /* FALSE */
                } else {
                    buffer = ((*(*(*cinfo).mem).access_virt_barray)(
                        cinfo as j_common_ptr,
                        (*coef).whole_image[ci as usize],
                        0,
                        access_rows as u32,
                        0,
                    ));
                    first_row = 1;  /* TRUE */
                }
                /* Fetch component-dependent info */
                coef_bits = (*coef).coef_bits_latch.add((ci as usize * SAVED_COEFS));
                quanttbl = (*compptr).quant_table;
                Q00 = (*quanttbl).quantval[0] as INT32;
                Q01 = (*quanttbl).quantval[1] as INT32;
                Q10 = (*quanttbl).quantval[2] as INT32;
                Q20 = (*quanttbl).quantval[3] as INT32;
                Q11 = (*quanttbl).quantval[4] as INT32;
                Q02 = (*quanttbl).quantval[5] as INT32;
                inverse_DCT = (*(*(*cinfo).idct).inverse_DCT[ci as usize]);
                output_ptr = *output_buf.add(ci as usize);
                /* Loop over all DCT blocks to be processed. */
                block_row = 0;
                while block_row < block_rows {
                    buffer_ptr = *buffer.add(block_row as usize);
                    if first_row != 0 && block_row == 0 {
                        prev_block_row = buffer_ptr;
                    } else {
                        prev_block_row = *buffer.add((block_row - 1) as usize);
                    }
                    if last_row != 0 && block_row == block_rows - 1 {
                        next_block_row = buffer_ptr;
                    } else {
                        next_block_row = *buffer.add((block_row + 1) as usize);
                    }
                    /* We fetch the surrounding DC values using a sliding-register approach.
                     * Initialize all nine here so as to do the right thing on narrow pics.
                     */
                    DC1 = (*(*prev_block_row)[0]) as c_int;
                    DC2 = DC1;
                    DC3 = DC1;
                    DC4 = (*(*buffer_ptr)[0]) as c_int;
                    DC5 = DC4;
                    DC6 = DC4;
                    DC7 = (*(*next_block_row)[0]) as c_int;
                    DC8 = DC7;
                    DC9 = DC7;
                    output_col = 0;
                    last_block_column = (*compptr).width_in_blocks - 1;
                    block_num = 0;
                    while block_num <= last_block_column {
                        /* Fetch current DCT block into workspace so we can modify it. */
                        jcopy_block_row(buffer_ptr, workspace.as_mut_ptr(), 1);
                        /* Update DC values */
                        if block_num < last_block_column {
                            DC3 = (*(*prev_block_row.add(1))[0]) as c_int;
                            DC6 = (*(*buffer_ptr.add(1))[0]) as c_int;
                            DC9 = (*(*next_block_row.add(1))[0]) as c_int;
                        }
                        /* Compute coefficient estimates per K.8.
                         * An estimate is applied only if coefficient is still zero,
                         * and is not known to be fully accurate.
                         */
                        /* AC01 */
                        if {
                            Al = *coef_bits.add(1);
                            Al != 0 && workspace[1] == 0
                        } {
                            num = 36 * Q00 * ((DC4 - DC6) as INT32);
                            if num >= 0 {
                                pred = (((Q01 << 7) + num) / (Q01 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                            } else {
                                pred = (((Q01 << 7) - num) / (Q01 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                                pred = -pred;
                            }
                            workspace[1] = pred as JCOEF;
                        }
                        /* AC10 */
                        if {
                            Al = *coef_bits.add(2);
                            Al != 0 && workspace[8] == 0
                        } {
                            num = 36 * Q00 * ((DC2 - DC8) as INT32);
                            if num >= 0 {
                                pred = (((Q10 << 7) + num) / (Q10 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                            } else {
                                pred = (((Q10 << 7) - num) / (Q10 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                                pred = -pred;
                            }
                            workspace[8] = pred as JCOEF;
                        }
                        /* AC20 */
                        if {
                            Al = *coef_bits.add(3);
                            Al != 0 && workspace[16] == 0
                        } {
                            num = 9 * Q00 * ((DC2 + DC8 - 2 * DC5) as INT32);
                            if num >= 0 {
                                pred = (((Q20 << 7) + num) / (Q20 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                            } else {
                                pred = (((Q20 << 7) - num) / (Q20 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                                pred = -pred;
                            }
                            workspace[16] = pred as JCOEF;
                        }
                        /* AC11 */
                        if {
                            Al = *coef_bits.add(4);
                            Al != 0 && workspace[9] == 0
                        } {
                            num = 5 * Q00 * ((DC1 - DC3 - DC7 + DC9) as INT32);
                            if num >= 0 {
                                pred = (((Q11 << 7) + num) / (Q11 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                            } else {
                                pred = (((Q11 << 7) - num) / (Q11 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                                pred = -pred;
                            }
                            workspace[9] = pred as JCOEF;
                        }
                        /* AC02 */
                        if {
                            Al = *coef_bits.add(5);
                            Al != 0 && workspace[2] == 0
                        } {
                            num = 9 * Q00 * ((DC4 + DC6 - 2 * DC5) as INT32);
                            if num >= 0 {
                                pred = (((Q02 << 7) + num) / (Q02 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                            } else {
                                pred = (((Q02 << 7) - num) / (Q02 << 8)) as c_int;
                                if Al > 0 && pred >= (1 << Al) {
                                    pred = (1 << Al) - 1;
                                }
                                pred = -pred;
                            }
                            workspace[2] = pred as JCOEF;
                        }
                        /* OK, do the IDCT */
                        inverse_DCT(
                            cinfo,
                            compptr,
                            workspace.as_mut_ptr(),
                            output_ptr,
                            output_col,
                        );
                        /* Advance for next column */
                        DC1 = DC2;
                        DC2 = DC3;
                        DC4 = DC5;
                        DC5 = DC6;
                        DC7 = DC8;
                        DC8 = DC9;
                        buffer_ptr = buffer_ptr.add(1);
                        prev_block_row = prev_block_row.add(1);
                        next_block_row = next_block_row.add(1);
                        output_col = (output_col as c_int + (*compptr).DCT_scaled_size) as JDIMENSION;
                        block_num += 1;
                    }
                    output_ptr = output_ptr.add((*compptr).DCT_scaled_size as usize);
                    block_row += 1;
                }
                ci += 1;
                compptr = compptr.add(1);
            }

            (*cinfo).output_iMCU_row += 1;
            if (*cinfo).output_iMCU_row < (*cinfo).total_iMCU_rows {
                return JPEG_ROW_COMPLETED;
            }
            return JPEG_SCAN_COMPLETED;
        }
    }
}

/* BLOCK_SMOOTHING_SUPPORTED */


/*
 * Initialize coefficient buffer controller.
 */

pub extern "C" fn jinit_d_coef_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean)
{
    unsafe {
        let mut coef: my_coef_ptr;

        coef = ((*(*(*cinfo).mem).alloc_small)(
            cinfo as j_common_ptr,
            JPOOL_IMAGE,
            mem::size_of::<my_coef_controller>(),
        )) as my_coef_ptr;
        (*cinfo).coef = coef as *mut jpeg_d_coef_controller;
        (*coef).pub_.start_input_pass = start_input_pass;
        (*coef).pub_.start_output_pass = start_output_pass;
        #[cfg(feature = "BLOCK_SMOOTHING_SUPPORTED")]
        {
            (*coef).coef_bits_latch = core::ptr::null_mut();
        }

        /* Create the coefficient buffer. */
        if need_full_buffer != 0 {
            #[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
            {
                /* Allocate a full-image virtual array for each component, */
                /* padded to a multiple of samp_factor DCT blocks in each direction. */
                /* Note we ask for a pre-zeroed array. */
                let mut ci: c_int;
                let mut access_rows: c_int;
                let mut compptr: *mut jpeg_component_info;

                ci = 0;
                compptr = (*cinfo).comp_info;
                while ci < (*cinfo).num_components {
                    access_rows = (*compptr).v_samp_factor;
                    #[cfg(feature = "BLOCK_SMOOTHING_SUPPORTED")]
                    {
                        /* If block smoothing could be used, need a bigger window */
                        if (*cinfo).progressive_mode != 0 {
                            access_rows *= 3;
                        }
                    }
                    (*coef).whole_image[ci as usize] = ((*(*(*cinfo).mem).request_virt_barray)(
                        cinfo as j_common_ptr,
                        JPOOL_IMAGE,
                        1,  /* TRUE */
                        jround_up((*compptr).width_in_blocks as i64, (*compptr).h_samp_factor as i64) as u32,
                        jround_up((*compptr).height_in_blocks as i64, (*compptr).v_samp_factor as i64) as u32,
                        access_rows as u32,
                    ));
                    ci += 1;
                    compptr = compptr.add(1);
                }
                (*coef).pub_.consume_data = consume_data;
                (*coef).pub_.decompress_data = decompress_data;
                (*coef).pub_.coef_arrays = (*coef).whole_image.as_mut_ptr();  /* link to virtual arrays */
            }

            #[cfg(not(feature = "D_MULTISCAN_FILES_SUPPORTED"))]
            {
                ERREXIT(cinfo, 0); /* JERR_NOT_COMPILED */
            }
        } else {
            /* We only need a single-MCU buffer. */
            let mut buffer: JBLOCKROW;
            let mut i: c_int;

            buffer = ((*(*(*cinfo).mem).alloc_large)(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                (D_MAX_BLOCKS_IN_MCU * mem::size_of::<JBLOCK>()),
            )) as JBLOCKROW;
            i = 0;
            while i < (D_MAX_BLOCKS_IN_MCU as c_int) {
                (*coef).MCU_buffer[i as usize] = buffer.add(i as usize);
                i += 1;
            }
            (*coef).pub_.consume_data = dummy_consume_data;
            (*coef).pub_.decompress_data = decompress_onepass;
            (*coef).pub_.coef_arrays = core::ptr::null_mut();  /* flag for no virtual arrays */
        }
    }
}
