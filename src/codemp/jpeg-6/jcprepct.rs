/*
 * jcprepct.c
 *
 * Copyright (C) 1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains the compression preprocessing controller.
 * This controller manages the color conversion, downsampling,
 * and edge expansion steps.
 *
 * Most of the complexity here is associated with buffering input rows
 * as required by the downsampler.  See the comments at the head of
 * jcsample.c for the downsampler's needs.
 */
// Anything above this include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #define JPEG_INTERNALS
// #include "jinclude.h"
// #include "jpeglib.h"

use core::ffi::{c_int, c_void};

/* At present, jcsample.c can request context rows only for smoothing.
 * In the future, we might also need context rows for CCIR601 sampling
 * or other more-complex downsampling procedures.  The code to support
 * context rows should be compiled only if needed.
 */
#[cfg(feature = "input_smoothing_supported")]
const _CONTEXT_ROWS_SUPPORTED: bool = true;

// ============================================================================
// Stubs for JPEG-6 types and structures needed for structural coherence
// ============================================================================

pub type JDIMENSION = u32;
pub type JSAMPROW = *mut u8;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type J_BUF_MODE = c_int;
pub type boolean = u8;

// Forward declarations
#[repr(C)]
pub struct j_compress_struct;

pub type j_compress_ptr = *mut j_compress_struct;
pub type j_common_ptr = *mut j_compress_struct;

#[repr(C)]
pub struct jpeg_component_info {
    pub width_in_blocks: c_int,
    pub v_samp_factor: c_int,
    pub h_samp_factor: c_int,
}

#[repr(C)]
pub struct color_converter {
    pub color_convert: Option<
        unsafe extern "C" fn(j_compress_ptr, JSAMPARRAY, JSAMPARRAY, JDIMENSION, c_int),
    >,
}

#[repr(C)]
pub struct downsample_controller {
    pub need_context_rows: boolean,
    pub downsample:
        Option<unsafe extern "C" fn(j_compress_ptr, JSAMPARRAY, JDIMENSION, JSAMPIMAGE, JDIMENSION)>,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small:
        Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void>,
    pub alloc_sarray: Option<
        unsafe extern "C" fn(j_common_ptr, c_int, JDIMENSION, JDIMENSION) -> JSAMPARRAY,
    >,
}

#[repr(C)]
pub struct jpeg_c_prep_controller {
    pub start_pass: Option<unsafe extern "C" fn(j_compress_ptr, J_BUF_MODE)>,
    pub pre_process_data: Option<
        unsafe extern "C" fn(
            j_compress_ptr,
            JSAMPARRAY,
            *mut JDIMENSION,
            JDIMENSION,
            JSAMPIMAGE,
            *mut JDIMENSION,
            JDIMENSION,
        ),
    >,
}

#[repr(C)]
pub struct j_compress_struct {
    pub prep: *mut jpeg_c_prep_controller,
    pub image_height: JDIMENSION,
    pub max_v_samp_factor: c_int,
    pub max_h_samp_factor: c_int,
    pub num_components: c_int,
    pub comp_info: *mut jpeg_component_info,
    pub downsample: *mut downsample_controller,
    pub cconvert: *mut color_converter,
    pub mem: *mut jpeg_memory_mgr,
}

// Constants
const MAX_COMPONENTS: usize = 10;
const DCTSIZE: c_int = 8;
const JPOOL_IMAGE: c_int = 0;
const JBUF_PASS_THRU: J_BUF_MODE = 0;
const JERR_BAD_BUFFER_MODE: c_int = 1;
const JERR_NOT_COMPILED: c_int = 2;

// Macro equivalent: MIN
#[inline]
fn MIN<T: std::cmp::Ord + Copy>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

// Stub for jcopy_sample_rows - external dependency
#[allow(non_snake_case)]
fn jcopy_sample_rows(
    _input_array: JSAMPARRAY,
    _input_row: c_int,
    _output_array: JSAMPARRAY,
    _output_row: c_int,
    _num_rows: c_int,
    _num_cols: JDIMENSION,
) {
    // Stub: implementation provided by jpeglib
}

/*
 * For the simple (no-context-row) case, we just need to buffer one
 * row group's worth of pixels for the downsampling step.  At the bottom of
 * the image, we pad to a full row group by replicating the last pixel row.
 * The downsampler's last output row is then replicated if needed to pad
 * out to a full iMCU row.
 *
 * When providing context rows, we must buffer three row groups' worth of
 * pixels.  Three row groups are physically allocated, but the row pointer
 * arrays are made five row groups high, with the extra pointers above and
 * below "wrapping around" to point to the last and first real row groups.
 * This allows the downsampler to access the proper context rows.
 * At the top and bottom of the image, we create dummy context rows by
 * copying the first or last real pixel row.  This copying could be avoided
 * by pointer hacking as is done in jdmainct.c, but it doesn't seem worth the
 * trouble on the compression side.
 */

/* Private buffer controller object */

#[repr(C)]
struct my_prep_controller {
    pub pub_: jpeg_c_prep_controller, /* public fields */

    /* Downsampling input buffer.  This buffer holds color-converted data
     * until we have enough to do a downsample step.
     */
    color_buf: [JSAMPARRAY; MAX_COMPONENTS],

    rows_to_go: JDIMENSION, /* counts rows remaining in source image */
    next_buf_row: c_int,    /* index of next row to store in color_buf */

    #[cfg(feature = "input_smoothing_supported")]
    this_row_group: c_int, /* starting row index of group to process */
    #[cfg(feature = "input_smoothing_supported")]
    next_buf_stop: c_int, /* downsample when we reach this index */
}

type my_prep_ptr = *mut my_prep_controller;

/*
 * Initialize for a processing pass.
 */

#[allow(non_snake_case)]
unsafe extern "C" fn start_pass_prep(cinfo: j_compress_ptr, pass_mode: J_BUF_MODE) {
    let prep = &mut *((*cinfo).prep as my_prep_ptr);

    if pass_mode != JBUF_PASS_THRU {
        // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
    }

    /* Initialize total-height counter for detecting bottom of image */
    prep.rows_to_go = (*cinfo).image_height;
    /* Mark the conversion buffer empty */
    prep.next_buf_row = 0;
    #[cfg(feature = "input_smoothing_supported")]
    {
        /* Preset additional state variables for context mode.
         * These aren't used in non-context mode, so we needn't test which mode.
         */
        prep.this_row_group = 0;
        /* Set next_buf_stop to stop after two row groups have been read in. */
        prep.next_buf_stop = 2 * (*cinfo).max_v_samp_factor;
    }
}

/*
 * Expand an image vertically from height input_rows to height output_rows,
 * by duplicating the bottom row.
 */

#[allow(non_snake_case)]
fn expand_bottom_edge(
    image_data: JSAMPARRAY,
    num_cols: JDIMENSION,
    input_rows: c_int,
    output_rows: c_int,
) {
    let mut row = input_rows;

    while row < output_rows {
        jcopy_sample_rows(image_data, input_rows - 1, image_data, row, 1, num_cols);
        row += 1;
    }
}

/*
 * Process some data in the simple no-context case.
 *
 * Preprocessor output data is counted in "row groups".  A row group
 * is defined to be v_samp_factor sample rows of each component.
 * Downsampling will produce this much data from each max_v_samp_factor
 * input rows.
 */

#[allow(non_snake_case)]
unsafe extern "C" fn pre_process_data(
    cinfo: j_compress_ptr,
    input_buf: JSAMPARRAY,
    in_row_ctr: *mut JDIMENSION,
    in_rows_avail: JDIMENSION,
    output_buf: JSAMPIMAGE,
    out_row_group_ctr: *mut JDIMENSION,
    out_row_groups_avail: JDIMENSION,
) {
    let prep = &mut *((*cinfo).prep as my_prep_ptr);
    let mut numrows: c_int;
    let mut ci: c_int;
    let mut inrows: JDIMENSION;
    let mut compptr: *mut jpeg_component_info;

    while *in_row_ctr < in_rows_avail && *out_row_group_ctr < out_row_groups_avail {
        /* Do color conversion to fill the conversion buffer. */
        inrows = in_rows_avail - *in_row_ctr;
        numrows = (*cinfo).max_v_samp_factor - prep.next_buf_row;
        numrows = MIN((numrows as JDIMENSION), inrows) as c_int;
        ((*(*cinfo).cconvert).color_convert.unwrap())(
            cinfo,
            input_buf.add(*in_row_ctr as usize),
            prep.color_buf[0],
            prep.next_buf_row as JDIMENSION,
            numrows,
        );
        *in_row_ctr += numrows as JDIMENSION;
        prep.next_buf_row += numrows;
        prep.rows_to_go -= numrows as JDIMENSION;
        /* If at bottom of image, pad to fill the conversion buffer. */
        if prep.rows_to_go == 0 && prep.next_buf_row < (*cinfo).max_v_samp_factor {
            ci = 0;
            while ci < (*cinfo).num_components {
                expand_bottom_edge(
                    prep.color_buf[ci as usize],
                    (*cinfo).image_width,
                    prep.next_buf_row,
                    (*cinfo).max_v_samp_factor,
                );
                ci += 1;
            }
            prep.next_buf_row = (*cinfo).max_v_samp_factor;
        }
        /* If we've filled the conversion buffer, empty it. */
        if prep.next_buf_row == (*cinfo).max_v_samp_factor {
            ((*(*cinfo).downsample).downsample.unwrap())(
                cinfo,
                prep.color_buf[0],
                0,
                output_buf,
                *out_row_group_ctr,
            );
            prep.next_buf_row = 0;
            *out_row_group_ctr += 1;
        }
        /* If at bottom of image, pad the output to a full iMCU height.
         * Note we assume the caller is providing a one-iMCU-height output buffer!
         */
        if prep.rows_to_go == 0 && *out_row_group_ctr < out_row_groups_avail {
            ci = 0;
            compptr = (*cinfo).comp_info;
            while ci < (*cinfo).num_components {
                expand_bottom_edge(
                    *output_buf.add(ci as usize),
                    ((*compptr).width_in_blocks * DCTSIZE) as JDIMENSION,
                    (*out_row_group_ctr * (*compptr).v_samp_factor as JDIMENSION) as c_int,
                    (out_row_groups_avail * (*compptr).v_samp_factor as JDIMENSION) as c_int,
                );
                ci += 1;
                compptr = compptr.add(1);
            }
            *out_row_group_ctr = out_row_groups_avail;
            break; /* can exit outer loop without test */
        }
    }
}

#[cfg(feature = "input_smoothing_supported")]
/*
 * Process some data in the context case.
 */
#[allow(non_snake_case)]
unsafe extern "C" fn pre_process_context(
    cinfo: j_compress_ptr,
    input_buf: JSAMPARRAY,
    in_row_ctr: *mut JDIMENSION,
    in_rows_avail: JDIMENSION,
    output_buf: JSAMPIMAGE,
    out_row_group_ctr: *mut JDIMENSION,
    out_row_groups_avail: JDIMENSION,
) {
    let prep = &mut *((*cinfo).prep as my_prep_ptr);
    let mut numrows: c_int;
    let mut ci: c_int;
    let buf_height = (*cinfo).max_v_samp_factor * 3;
    let mut inrows: JDIMENSION;
    let mut compptr: *mut jpeg_component_info;

    while *out_row_group_ctr < out_row_groups_avail {
        if *in_row_ctr < in_rows_avail {
            /* Do color conversion to fill the conversion buffer. */
            inrows = in_rows_avail - *in_row_ctr;
            numrows = prep.next_buf_stop - prep.next_buf_row;
            numrows = MIN((numrows as JDIMENSION), inrows) as c_int;
            ((*(*cinfo).cconvert).color_convert.unwrap())(
                cinfo,
                input_buf.add(*in_row_ctr as usize),
                prep.color_buf[0],
                prep.next_buf_row as JDIMENSION,
                numrows,
            );
            /* Pad at top of image, if first time through */
            if prep.rows_to_go == (*cinfo).image_height {
                ci = 0;
                while ci < (*cinfo).num_components {
                    let mut row = 1;
                    while row <= (*cinfo).max_v_samp_factor {
                        jcopy_sample_rows(
                            prep.color_buf[ci as usize],
                            0,
                            prep.color_buf[ci as usize],
                            -row,
                            1,
                            (*cinfo).image_width,
                        );
                        row += 1;
                    }
                    ci += 1;
                }
            }
            *in_row_ctr += numrows as JDIMENSION;
            prep.next_buf_row += numrows;
            prep.rows_to_go -= numrows as JDIMENSION;
        } else {
            /* Return for more data, unless we are at the bottom of the image. */
            if prep.rows_to_go != 0 {
                break;
            }
        }
        /* If at bottom of image, pad to fill the conversion buffer. */
        if prep.rows_to_go == 0 && prep.next_buf_row < prep.next_buf_stop {
            ci = 0;
            while ci < (*cinfo).num_components {
                expand_bottom_edge(
                    prep.color_buf[ci as usize],
                    (*cinfo).image_width,
                    prep.next_buf_row,
                    prep.next_buf_stop,
                );
                ci += 1;
            }
            prep.next_buf_row = prep.next_buf_stop;
        }
        /* If we've gotten enough data, downsample a row group. */
        if prep.next_buf_row == prep.next_buf_stop {
            ((*(*cinfo).downsample).downsample.unwrap())(
                cinfo,
                prep.color_buf[0],
                prep.this_row_group as JDIMENSION,
                output_buf,
                *out_row_group_ctr,
            );
            *out_row_group_ctr += 1;
            /* Advance pointers with wraparound as necessary. */
            prep.this_row_group += (*cinfo).max_v_samp_factor;
            if prep.this_row_group >= buf_height {
                prep.this_row_group = 0;
            }
            if prep.next_buf_row >= buf_height {
                prep.next_buf_row = 0;
            }
            prep.next_buf_stop = prep.next_buf_row + (*cinfo).max_v_samp_factor;
        }
        /* If at bottom of image, pad the output to a full iMCU height.
         * Note we assume the caller is providing a one-iMCU-height output buffer!
         */
        if prep.rows_to_go == 0 && *out_row_group_ctr < out_row_groups_avail {
            ci = 0;
            compptr = (*cinfo).comp_info;
            while ci < (*cinfo).num_components {
                expand_bottom_edge(
                    *output_buf.add(ci as usize),
                    ((*compptr).width_in_blocks * DCTSIZE) as JDIMENSION,
                    (*out_row_group_ctr * (*compptr).v_samp_factor as JDIMENSION) as c_int,
                    (out_row_groups_avail * (*compptr).v_samp_factor as JDIMENSION) as c_int,
                );
                ci += 1;
                compptr = compptr.add(1);
            }
            *out_row_group_ctr = out_row_groups_avail;
            break; /* can exit outer loop without test */
        }
    }
}

#[cfg(feature = "input_smoothing_supported")]
/*
 * Create the wrapped-around downsampling input buffer needed for context mode.
 */
#[allow(non_snake_case)]
unsafe fn create_context_buffer(cinfo: j_compress_ptr) {
    let prep = &mut *((*cinfo).prep as my_prep_ptr);
    let rgroup_height = (*cinfo).max_v_samp_factor;
    let mut ci: c_int;
    let mut i: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut true_buffer: JSAMPARRAY;
    let mut fake_buffer: JSAMPARRAY;

    /* Grab enough space for fake row pointers for all the components;
     * we need five row groups' worth of pointers for each component.
     */
    fake_buffer = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        ((*cinfo).num_components as usize * 5 * rgroup_height as usize)
            * std::mem::size_of::<JSAMPROW>(),
    ) as JSAMPARRAY;

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Allocate the actual buffer space (3 row groups) for this component.
         * We make the buffer wide enough to allow the downsampler to edge-expand
         * horizontally within the buffer, if it so chooses.
         */
        true_buffer = ((*(*cinfo).mem).alloc_sarray.unwrap())(
            cinfo as j_common_ptr,
            JPOOL_IMAGE,
            ((((*compptr).width_in_blocks as i64 * DCTSIZE as i64
                * (*cinfo).max_h_samp_factor as i64)
                / (*compptr).h_samp_factor as i64) as u32),
            (3 * rgroup_height) as u32,
        );
        /* Copy true buffer row pointers into the middle of the fake row array */
        core::ptr::copy_nonoverlapping(
            true_buffer,
            fake_buffer.add((rgroup_height as usize)),
            3 * rgroup_height as usize,
        );
        /* Fill in the above and below wraparound pointers */
        i = 0;
        while i < rgroup_height {
            *fake_buffer.add(i as usize) = *true_buffer.add((2 * rgroup_height + i) as usize);
            *fake_buffer.add((4 * rgroup_height + i) as usize) = *true_buffer.add(i as usize);
            i += 1;
        }
        prep.color_buf[ci as usize] = fake_buffer.add((rgroup_height as usize));
        fake_buffer = fake_buffer.add((5 * rgroup_height as usize)); /* point to space for next component */
        ci += 1;
        compptr = compptr.add(1);
    }
}

/*
 * Initialize preprocessing controller.
 */

#[allow(non_snake_case)]
pub unsafe extern "C" fn jinit_c_prep_controller(cinfo: j_compress_ptr, need_full_buffer: boolean) {
    let mut prep: my_prep_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    if need_full_buffer != 0 {
        /* safety check */
        // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
    }

    prep = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        std::mem::size_of::<my_prep_controller>(),
    ) as my_prep_ptr;
    (*cinfo).prep = prep as *mut jpeg_c_prep_controller;
    (*prep).pub_.start_pass = Some(start_pass_prep);

    /* Allocate the color conversion buffer.
     * We make the buffer wide enough to allow the downsampler to edge-expand
     * horizontally within the buffer, if it so chooses.
     */
    if (*(*cinfo).downsample).need_context_rows != 0 {
        /* Set up to provide context rows */
        #[cfg(feature = "input_smoothing_supported")]
        {
            (*prep).pub_.pre_process_data = Some(pre_process_context);
            create_context_buffer(cinfo);
        }
        #[cfg(not(feature = "input_smoothing_supported"))]
        {
            // ERREXIT(cinfo, JERR_NOT_COMPILED);
        }
    } else {
        /* No context, just make it tall enough for one row group */
        (*prep).pub_.pre_process_data = Some(pre_process_data);
        ci = 0;
        compptr = (*cinfo).comp_info;
        while ci < (*cinfo).num_components {
            (*prep).color_buf[ci as usize] = ((*(*cinfo).mem).alloc_sarray.unwrap())(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                ((((*compptr).width_in_blocks as i64 * DCTSIZE as i64
                    * (*cinfo).max_h_samp_factor as i64)
                    / (*compptr).h_samp_factor as i64) as u32),
                (*cinfo).max_v_samp_factor as u32,
            );
            ci += 1;
            compptr = compptr.add(1);
        }
    }
}
