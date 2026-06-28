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

// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_void};

/* At present, jcsample.c can request context rows only for smoothing.
 * In the future, we might also need context rows for CCIR601 sampling
 * or other more-complex downsampling procedures.  The code to support
 * context rows should be compiled only if needed.
 */
#[cfg(feature = "input_smoothing_supported")]
const CONTEXT_ROWS_SUPPORTED: () = ();

/* External JPEG type stubs and imports */
/* These would normally come from jpeglib.h and jinclude.h */

pub type JDIMENSION = c_int;
pub type JSAMPROW = *mut c_int;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;

pub type j_common_ptr = *mut c_void;
pub type j_compress_ptr = *mut jpeg_compress_struct;

#[repr(C)]
pub struct jpeg_compress_struct {
    /* Placeholder for actual JPEG struct fields */
    /* Real fields would come from jpeglib.h */
    pub dummy: c_int,
}

#[repr(C)]
pub struct jpeg_component_info {
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
    pub width_in_blocks: c_int,
    pub dummy: c_int,
}

#[repr(C)]
pub struct jpeg_c_prep_controller {
    pub start_pass: *mut c_void,
    pub pre_process_data: *mut c_void,
}

pub type J_BUF_MODE = c_int;
const JBUF_PASS_THRU: J_BUF_MODE = 0;
const JPOOL_IMAGE: c_int = 0;
const JERR_BAD_BUFFER_MODE: c_int = 1;
const JERR_NOT_COMPILED: c_int = 2;
const DCTSIZE: c_int = 8;

#[inline]
fn MIN(a: JDIMENSION, b: JDIMENSION) -> JDIMENSION {
    if a < b { a } else { b }
}

#[inline]
fn SIZEOF<T>(_: &T) -> usize {
    std::mem::size_of::<T>()
}

const MAX_COMPONENTS: usize = 10;

/* Private buffer controller object */

#[repr(C)]
pub struct my_prep_controller {
    pub pub_: jpeg_c_prep_controller, /* public fields */

    /* Downsampling input buffer.  This buffer holds color-converted data
     * until we have enough to do a downsample step.
     */
    pub color_buf: [JSAMPARRAY; MAX_COMPONENTS],

    pub rows_to_go: JDIMENSION, /* counts rows remaining in source image */
    pub next_buf_row: c_int, /* index of next row to store in color_buf */

    #[cfg(feature = "input_smoothing_supported")]
    pub this_row_group: c_int, /* starting row index of group to process */
    #[cfg(feature = "input_smoothing_supported")]
    pub next_buf_stop: c_int, /* downsample when we reach this index */
}

pub type my_prep_ptr = *mut my_prep_controller;

/* External JPEG function stubs */
extern "C" {
    fn jcopy_sample_rows(
        input_array: JSAMPARRAY,
        input_row: JDIMENSION,
        output_array: JSAMPARRAY,
        output_row: JDIMENSION,
        num_rows: JDIMENSION,
        num_cols: JDIMENSION,
    );
    fn ERREXIT(cinfo: j_compress_ptr, error_code: c_int);
}

/*
 * Initialize for a processing pass.
 */

#[no_mangle]
pub extern "C" fn start_pass_prep(cinfo: j_compress_ptr, pass_mode: J_BUF_MODE) {
    let prep = unsafe { &mut *((*cinfo).prep as my_prep_ptr) };

    if pass_mode != JBUF_PASS_THRU {
        unsafe { ERREXIT(cinfo, JERR_BAD_BUFFER_MODE) };
    }

    /* Initialize total-height counter for detecting bottom of image */
    prep.rows_to_go = unsafe { (*cinfo).image_height };
    /* Mark the conversion buffer empty */
    prep.next_buf_row = 0;
    #[cfg(feature = "input_smoothing_supported")]
    {
        /* Preset additional state variables for context mode.
         * These aren't used in non-context mode, so we needn't test which mode.
         */
        prep.this_row_group = 0;
        /* Set next_buf_stop to stop after two row groups have been read in. */
        prep.next_buf_stop = 2 * unsafe { (*cinfo).max_v_samp_factor };
    }
}

/*
 * Expand an image vertically from height input_rows to height output_rows,
 * by duplicating the bottom row.
 */

pub extern "C" fn expand_bottom_edge(
    image_data: JSAMPARRAY,
    num_cols: JDIMENSION,
    input_rows: c_int,
    output_rows: c_int,
) {
    let mut row = input_rows;

    while row < output_rows {
        unsafe {
            jcopy_sample_rows(
                image_data,
                (input_rows - 1) as JDIMENSION,
                image_data,
                row as JDIMENSION,
                1,
                num_cols,
            );
        }
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

#[no_mangle]
pub extern "C" fn pre_process_data(
    cinfo: j_compress_ptr,
    input_buf: JSAMPARRAY,
    in_row_ctr: *mut JDIMENSION,
    in_rows_avail: JDIMENSION,
    output_buf: JSAMPIMAGE,
    out_row_group_ctr: *mut JDIMENSION,
    out_row_groups_avail: JDIMENSION,
) {
    let prep = unsafe { &mut *((*cinfo).prep as my_prep_ptr) };
    let mut numrows: c_int;
    let mut ci: c_int;
    let mut inrows: JDIMENSION;
    let mut compptr: *mut jpeg_component_info;

    while unsafe { *in_row_ctr < in_rows_avail && *out_row_group_ctr < out_row_groups_avail } {
        /* Do color conversion to fill the conversion buffer. */
        unsafe { inrows = in_rows_avail - *in_row_ctr };
        numrows = unsafe { (*cinfo).max_v_samp_factor } - prep.next_buf_row;
        numrows = MIN(numrows as JDIMENSION, inrows) as c_int;
        unsafe {
            ((*(*cinfo).cconvert).color_convert)(
                cinfo,
                input_buf.add(*in_row_ctr as usize),
                prep.color_buf.as_mut_ptr(),
                prep.next_buf_row as JDIMENSION,
                numrows as JDIMENSION,
            );
        }
        unsafe { *in_row_ctr += numrows as JDIMENSION };
        prep.next_buf_row += numrows;
        prep.rows_to_go -= numrows as JDIMENSION;
        /* If at bottom of image, pad to fill the conversion buffer. */
        if prep.rows_to_go == 0
            && prep.next_buf_row < unsafe { (*cinfo).max_v_samp_factor }
        {
            ci = 0;
            while ci < unsafe { (*cinfo).num_components } {
                unsafe {
                    expand_bottom_edge(
                        prep.color_buf[ci as usize],
                        (*cinfo).image_width,
                        prep.next_buf_row,
                        (*cinfo).max_v_samp_factor,
                    );
                }
                ci += 1;
            }
            prep.next_buf_row = unsafe { (*cinfo).max_v_samp_factor };
        }
        /* If we've filled the conversion buffer, empty it. */
        if prep.next_buf_row == unsafe { (*cinfo).max_v_samp_factor } {
            unsafe {
                ((*(*cinfo).downsample).downsample)(
                    cinfo,
                    prep.color_buf.as_mut_ptr(),
                    0,
                    output_buf,
                    *out_row_group_ctr,
                );
            }
            prep.next_buf_row = 0;
            unsafe { *out_row_group_ctr += 1 };
        }
        /* If at bottom of image, pad the output to a full iMCU height.
         * Note we assume the caller is providing a one-iMCU-height output buffer!
         */
        if prep.rows_to_go == 0 && unsafe { *out_row_group_ctr < out_row_groups_avail } {
            ci = 0;
            compptr = unsafe { (*cinfo).comp_info };
            while ci < unsafe { (*cinfo).num_components } {
                unsafe {
                    expand_bottom_edge(
                        *output_buf.add(ci as usize),
                        (*compptr).width_in_blocks * DCTSIZE,
                        (*out_row_group_ctr * (*compptr).v_samp_factor as JDIMENSION)
                            as c_int,
                        (out_row_groups_avail * (*compptr).v_samp_factor as JDIMENSION)
                            as c_int,
                    );
                }
                ci += 1;
                compptr = unsafe { compptr.add(1) };
            }
            unsafe { *out_row_group_ctr = out_row_groups_avail };
            break; /* can exit outer loop without test */
        }
    }
}

#[cfg(feature = "input_smoothing_supported")]
{
    /*
     * Process some data in the context case.
     */

    #[no_mangle]
    pub extern "C" fn pre_process_context(
        cinfo: j_compress_ptr,
        input_buf: JSAMPARRAY,
        in_row_ctr: *mut JDIMENSION,
        in_rows_avail: JDIMENSION,
        output_buf: JSAMPIMAGE,
        out_row_group_ctr: *mut JDIMENSION,
        out_row_groups_avail: JDIMENSION,
    ) {
        let prep = unsafe { &mut *((*cinfo).prep as my_prep_ptr) };
        let mut numrows: c_int;
        let mut ci: c_int;
        let buf_height = unsafe { (*cinfo).max_v_samp_factor * 3 };
        let mut inrows: JDIMENSION;
        let mut compptr: *mut jpeg_component_info;

        while unsafe { *out_row_group_ctr < out_row_groups_avail } {
            if unsafe { *in_row_ctr < in_rows_avail } {
                /* Do color conversion to fill the conversion buffer. */
                unsafe { inrows = in_rows_avail - *in_row_ctr };
                numrows = prep.next_buf_stop - prep.next_buf_row;
                numrows = MIN(numrows as JDIMENSION, inrows) as c_int;
                unsafe {
                    ((*(*cinfo).cconvert).color_convert)(
                        cinfo,
                        input_buf.add(*in_row_ctr as usize),
                        prep.color_buf.as_mut_ptr(),
                        prep.next_buf_row as JDIMENSION,
                        numrows as JDIMENSION,
                    );
                }
                /* Pad at top of image, if first time through */
                if prep.rows_to_go == unsafe { (*cinfo).image_height } {
                    ci = 0;
                    while ci < unsafe { (*cinfo).num_components } {
                        let mut row = 1;
                        while row <= unsafe { (*cinfo).max_v_samp_factor } {
                            unsafe {
                                jcopy_sample_rows(
                                    prep.color_buf[ci as usize],
                                    0,
                                    prep.color_buf[ci as usize],
                                    (-row) as JDIMENSION,
                                    1,
                                    (*cinfo).image_width,
                                );
                            }
                            row += 1;
                        }
                        ci += 1;
                    }
                }
                unsafe { *in_row_ctr += numrows as JDIMENSION };
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
                while ci < unsafe { (*cinfo).num_components } {
                    unsafe {
                        expand_bottom_edge(
                            prep.color_buf[ci as usize],
                            (*cinfo).image_width,
                            prep.next_buf_row,
                            prep.next_buf_stop,
                        );
                    }
                    ci += 1;
                }
                prep.next_buf_row = prep.next_buf_stop;
            }
            /* If we've gotten enough data, downsample a row group. */
            if prep.next_buf_row == prep.next_buf_stop {
                unsafe {
                    ((*(*cinfo).downsample).downsample)(
                        cinfo,
                        prep.color_buf.as_mut_ptr(),
                        prep.this_row_group as JDIMENSION,
                        output_buf,
                        *out_row_group_ctr,
                    );
                }
                unsafe { *out_row_group_ctr += 1 };
                /* Advance pointers with wraparound as necessary. */
                prep.this_row_group += unsafe { (*cinfo).max_v_samp_factor };
                if prep.this_row_group >= buf_height {
                    prep.this_row_group = 0;
                }
                if prep.next_buf_row >= buf_height {
                    prep.next_buf_row = 0;
                }
                prep.next_buf_stop = prep.next_buf_row + unsafe { (*cinfo).max_v_samp_factor };
            }
            /* If at bottom of image, pad the output to a full iMCU height.
             * Note we assume the caller is providing a one-iMCU-height output buffer!
             */
            if prep.rows_to_go == 0
                && unsafe { *out_row_group_ctr < out_row_groups_avail }
            {
                ci = 0;
                compptr = unsafe { (*cinfo).comp_info };
                while ci < unsafe { (*cinfo).num_components } {
                    unsafe {
                        expand_bottom_edge(
                            *output_buf.add(ci as usize),
                            (*compptr).width_in_blocks * DCTSIZE,
                            (*out_row_group_ctr * (*compptr).v_samp_factor as JDIMENSION)
                                as c_int,
                            (out_row_groups_avail * (*compptr).v_samp_factor as JDIMENSION)
                                as c_int,
                        );
                    }
                    ci += 1;
                    compptr = unsafe { compptr.add(1) };
                }
                unsafe { *out_row_group_ctr = out_row_groups_avail };
                break; /* can exit outer loop without test */
            }
        }
    }

    /*
     * Create the wrapped-around downsampling input buffer needed for context mode.
     */

    pub extern "C" fn create_context_buffer(cinfo: j_compress_ptr) {
        let prep = unsafe { &mut *((*cinfo).prep as my_prep_ptr) };
        let rgroup_height = unsafe { (*cinfo).max_v_samp_factor };
        let mut ci: c_int;
        let mut i: c_int;
        let mut compptr: *mut jpeg_component_info;
        let mut true_buffer: JSAMPARRAY;
        let mut fake_buffer: JSAMPARRAY;

        /* Grab enough space for fake row pointers for all the components;
         * we need five row groups' worth of pointers for each component.
         */
        unsafe {
            fake_buffer = ((*(*cinfo).mem).alloc_small)(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                (((*cinfo).num_components * 5 * rgroup_height) * SIZEOF(0 as JSAMPROW)) as usize,
            ) as JSAMPARRAY;
        }

        ci = 0;
        compptr = unsafe { (*cinfo).comp_info };
        while ci < unsafe { (*cinfo).num_components } {
            /* Allocate the actual buffer space (3 row groups) for this component.
             * We make the buffer wide enough to allow the downsampler to edge-expand
             * horizontally within the buffer, if it so chooses.
             */
            unsafe {
                true_buffer = ((*(*cinfo).mem).alloc_sarray)(
                    cinfo as j_common_ptr,
                    JPOOL_IMAGE,
                    (((*compptr).width_in_blocks as i64 * DCTSIZE as i64
                        * (*cinfo).max_h_samp_factor as i64)
                        / (*compptr).h_samp_factor as i64) as JDIMENSION,
                    (3 * rgroup_height) as JDIMENSION,
                );
            }
            /* Copy true buffer row pointers into the middle of the fake row array */
            unsafe {
                core::ptr::copy_nonoverlapping(
                    true_buffer,
                    fake_buffer.add(rgroup_height as usize),
                    (3 * rgroup_height) as usize,
                );
            }
            /* Fill in the above and below wraparound pointers */
            i = 0;
            while i < rgroup_height {
                unsafe {
                    *fake_buffer.add(i as usize) =
                        *true_buffer.add((2 * rgroup_height + i) as usize);
                    *fake_buffer.add((4 * rgroup_height + i) as usize) =
                        *true_buffer.add(i as usize);
                }
                i += 1;
            }
            prep.color_buf[ci as usize] = unsafe { fake_buffer.add(rgroup_height as usize) };
            unsafe { fake_buffer = fake_buffer.add((5 * rgroup_height) as usize) }; /* point to space for next component */
            ci += 1;
            compptr = unsafe { compptr.add(1) };
        }
    }
}

/*
 * Initialize preprocessing controller.
 */

#[no_mangle]
pub extern "C" fn jinit_c_prep_controller(
    cinfo: j_compress_ptr,
    need_full_buffer: c_int,
) {
    let mut prep: my_prep_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    if need_full_buffer != 0 {
        /* safety check */
        unsafe { ERREXIT(cinfo, JERR_BAD_BUFFER_MODE) };
    }

    unsafe {
        prep = ((*(*cinfo).mem).alloc_small)(
            cinfo as j_common_ptr,
            JPOOL_IMAGE,
            std::mem::size_of::<my_prep_controller>(),
        ) as my_prep_ptr;
        (*cinfo).prep = &mut (*prep).pub_ as *mut jpeg_c_prep_controller;
        (*prep).pub_.start_pass = start_pass_prep as *mut c_void;
    }

    /* Allocate the color conversion buffer.
     * We make the buffer wide enough to allow the downsampler to edge-expand
     * horizontally within the buffer, if it so chooses.
     */
    if unsafe { (*(*cinfo).downsample).need_context_rows } != 0 {
        /* Set up to provide context rows */
        #[cfg(feature = "input_smoothing_supported")]
        {
            unsafe {
                (*prep).pub_.pre_process_data = pre_process_context as *mut c_void;
            }
            create_context_buffer(cinfo);
        }
        #[cfg(not(feature = "input_smoothing_supported"))]
        {
            unsafe { ERREXIT(cinfo, JERR_NOT_COMPILED) };
        }
    } else {
        /* No context, just make it tall enough for one row group */
        unsafe {
            (*prep).pub_.pre_process_data = pre_process_data as *mut c_void;
        }
        ci = 0;
        compptr = unsafe { (*cinfo).comp_info };
        while ci < unsafe { (*cinfo).num_components } {
            unsafe {
                (*prep).color_buf[ci as usize] = ((*(*cinfo).mem).alloc_sarray)(
                    cinfo as j_common_ptr,
                    JPOOL_IMAGE,
                    (((*compptr).width_in_blocks as i64 * DCTSIZE as i64
                        * (*cinfo).max_h_samp_factor as i64)
                        / (*compptr).h_samp_factor as i64) as JDIMENSION,
                    (*cinfo).max_v_samp_factor as JDIMENSION,
                );
            }
            ci += 1;
            compptr = unsafe { compptr.add(1) };
        }
    }
}
