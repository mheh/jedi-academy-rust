/*
 * jdpostct.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains the decompression postprocessing controller.
 * This controller manages the upsampling, color conversion, and color
 * quantization/reduction steps; specifically, it controls the buffering
 * between upsample/color conversion and color quantization/reduction.
 *
 * If no color quantization/reduction is required, then this module has no
 * work to do, and it just hands off to the upsample/color conversion code.
 * An integrated upsample/convert/quantize process would replace this module
 * entirely.
 */
// Anything above this #include will be ignored by the compiler
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_variables)]

use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::jpeg_6::jinclude_h::*;
use crate::codemp::jpeg_6::jpeglib_h::*;
use crate::codemp::jpeg_6::jpegint_h::*;

/* Private buffer controller object */

#[repr(C)]
pub struct my_post_controller {
    pub pub_: jpeg_d_post_controller, /* public fields */

    /* Color quantization source buffer: this holds output data from
     * the upsample/color conversion step to be passed to the quantizer.
     * For two-pass color quantization, we need a full-image buffer;
     * for one-pass operation, a strip buffer is sufficient.
     */
    pub whole_image: jvirt_sarray_ptr, /* virtual array, or NULL if one-pass */
    pub buffer: JSAMPARRAY,            /* strip buffer, or current strip of virtual */
    pub strip_height: JDIMENSION,      /* buffer size in rows */
    /* for two-pass mode only: */
    pub starting_row: JDIMENSION, /* row # of first row in current strip */
    pub next_row: JDIMENSION,     /* index of next row to fill/empty in strip */
}

pub type my_post_ptr = *mut my_post_controller;


/* Forward declarations */
// post_process_1pass (and post_process_prepass, post_process_2pass under
// QUANT_2PASS_SUPPORTED) are defined below; Rust does not require forward declarations.


/*
 * Initialize for a processing pass.
 */

unsafe fn start_pass_dpost(cinfo: j_decompress_ptr, pass_mode: J_BUF_MODE) {
    let post: my_post_ptr = (*cinfo).post as my_post_ptr;

    match pass_mode {
        JBUF_PASS_THRU => {
            if (*cinfo).quantize_colors != 0 {
                /* Single-pass processing with color quantization. */
                (*post).pub_.post_process_data = post_process_1pass;
                /* We could be doing buffered-image output before starting a 2-pass
                 * color quantization; in that case, jinit_d_post_controller did not
                 * allocate a strip buffer.  Use the virtual-array buffer as workspace.
                 */
                if (*post).buffer.is_null() {
                    (*post).buffer = ((*(*cinfo).mem).access_virt_sarray)(
                        cinfo as j_common_ptr,
                        (*post).whole_image,
                        0 as JDIMENSION,
                        (*post).strip_height,
                        TRUE,
                    );
                }
            } else {
                /* For single-pass processing without color quantization,
                 * I have no work to do; just call the upsampler directly.
                 */
                (*post).pub_.post_process_data = (*(*cinfo).upsample).upsample;
            }
        }
        #[cfg(feature = "quant_2pass_supported")]
        JBUF_SAVE_AND_PASS => {
            /* First pass of 2-pass quantization */
            if (*post).whole_image.is_null() {
                ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
            (*post).pub_.post_process_data = post_process_prepass;
        }
        #[cfg(feature = "quant_2pass_supported")]
        JBUF_CRANK_DEST => {
            /* Second pass of 2-pass quantization */
            if (*post).whole_image.is_null() {
                ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
            (*post).pub_.post_process_data = post_process_2pass;
        }
        /* QUANT_2PASS_SUPPORTED */
        _ => {
            ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
        }
    }
    (*post).starting_row = 0;
    (*post).next_row = 0;
}


/*
 * Process some data in the one-pass (strip buffer) case.
 * This is used for color precision reduction as well as one-pass quantization.
 */

unsafe fn post_process_1pass(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    let post: my_post_ptr = (*cinfo).post as my_post_ptr;
    let mut num_rows: JDIMENSION;
    let mut max_rows: JDIMENSION;

    /* Fill the buffer, but not more than what we can dump out in one go. */
    /* Note we rely on the upsampler to detect bottom of image. */
    max_rows = out_rows_avail - *out_row_ctr;
    if max_rows > (*post).strip_height {
        max_rows = (*post).strip_height;
    }
    num_rows = 0;
    ((*(*cinfo).upsample).upsample)(
        cinfo,
        input_buf,
        in_row_group_ctr,
        in_row_groups_avail,
        (*post).buffer,
        &mut num_rows,
        max_rows,
    );
    /* Quantize and emit data. */
    ((*(*cinfo).cquantize).color_quantize)(
        cinfo,
        (*post).buffer,
        output_buf.add(*out_row_ctr as usize),
        num_rows as core::ffi::c_int,
    );
    *out_row_ctr += num_rows;
}


/*
 * Process some data in the first pass of 2-pass quantization.
 */

#[cfg(feature = "quant_2pass_supported")]
unsafe fn post_process_prepass(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    let post: my_post_ptr = (*cinfo).post as my_post_ptr;
    let old_next_row: JDIMENSION;
    let num_rows: JDIMENSION;

    /* Reposition virtual buffer if at start of strip. */
    if (*post).next_row == 0 {
        (*post).buffer = ((*(*cinfo).mem).access_virt_sarray)(
            cinfo as j_common_ptr,
            (*post).whole_image,
            (*post).starting_row,
            (*post).strip_height,
            TRUE,
        );
    }

    /* Upsample some data (up to a strip height's worth). */
    old_next_row = (*post).next_row;
    ((*(*cinfo).upsample).upsample)(
        cinfo,
        input_buf,
        in_row_group_ctr,
        in_row_groups_avail,
        (*post).buffer,
        &mut (*post).next_row,
        (*post).strip_height,
    );

    /* Allow quantizer to scan new data.  No data is emitted, */
    /* but we advance out_row_ctr so outer loop can tell when we're done. */
    if (*post).next_row > old_next_row {
        num_rows = (*post).next_row - old_next_row;
        ((*(*cinfo).cquantize).color_quantize)(
            cinfo,
            (*post).buffer.add(old_next_row as usize),
            core::ptr::null_mut() as JSAMPARRAY,
            num_rows as core::ffi::c_int,
        );
        *out_row_ctr += num_rows;
    }

    /* Advance if we filled the strip. */
    if (*post).next_row >= (*post).strip_height {
        (*post).starting_row += (*post).strip_height;
        (*post).next_row = 0;
    }
}


/*
 * Process some data in the second pass of 2-pass quantization.
 */

#[cfg(feature = "quant_2pass_supported")]
unsafe fn post_process_2pass(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    let post: my_post_ptr = (*cinfo).post as my_post_ptr;
    let mut num_rows: JDIMENSION;
    let mut max_rows: JDIMENSION;

    /* Reposition virtual buffer if at start of strip. */
    if (*post).next_row == 0 {
        (*post).buffer = ((*(*cinfo).mem).access_virt_sarray)(
            cinfo as j_common_ptr,
            (*post).whole_image,
            (*post).starting_row,
            (*post).strip_height,
            FALSE,
        );
    }

    /* Determine number of rows to emit. */
    num_rows = (*post).strip_height - (*post).next_row; /* available in strip */
    max_rows = out_rows_avail - *out_row_ctr; /* available in output area */
    if num_rows > max_rows {
        num_rows = max_rows;
    }
    /* We have to check bottom of image here, can't depend on upsampler. */
    max_rows = (*cinfo).output_height - (*post).starting_row;
    if num_rows > max_rows {
        num_rows = max_rows;
    }

    /* Quantize and emit data. */
    ((*(*cinfo).cquantize).color_quantize)(
        cinfo,
        (*post).buffer.add((*post).next_row as usize),
        output_buf.add(*out_row_ctr as usize),
        num_rows as core::ffi::c_int,
    );
    *out_row_ctr += num_rows;

    /* Advance if we filled the strip. */
    (*post).next_row += num_rows;
    if (*post).next_row >= (*post).strip_height {
        (*post).starting_row += (*post).strip_height;
        (*post).next_row = 0;
    }
}

/* QUANT_2PASS_SUPPORTED */


/*
 * Initialize postprocessing controller.
 */

pub unsafe fn jinit_d_post_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean) {
    let post: my_post_ptr;

    post = ((*(*cinfo).mem).alloc_small)(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        core::mem::size_of::<my_post_controller>(),
    ) as my_post_ptr;
    // Port note: C casts post (my_post_ptr) to (struct jpeg_d_post_controller *) via
    // first-member aliasing; in Rust take the address of the first field directly.
    (*cinfo).post = core::ptr::addr_of_mut!((*post).pub_);
    (*post).pub_.start_pass = start_pass_dpost;
    (*post).whole_image = core::ptr::null_mut(); /* flag for no virtual arrays */
    (*post).buffer = core::ptr::null_mut();      /* flag for no strip buffer */

    /* Create the quantization buffer, if needed */
    if (*cinfo).quantize_colors != 0 {
        /* The buffer strip height is max_v_samp_factor, which is typically
         * an efficient number of rows for upsampling to return.
         * (In the presence of output rescaling, we might want to be smarter?)
         */
        (*post).strip_height = (*cinfo).max_v_samp_factor as JDIMENSION;
        if need_full_buffer != 0 {
            /* Two-pass color quantization: need full-image storage. */
            /* We round up the number of rows to a multiple of the strip height. */
            #[cfg(feature = "quant_2pass_supported")]
            {
                (*post).whole_image = ((*(*cinfo).mem).request_virt_sarray)(
                    cinfo as j_common_ptr,
                    JPOOL_IMAGE,
                    FALSE,
                    (*cinfo).output_width * (*cinfo).out_color_components,
                    jround_up(
                        (*cinfo).output_height as core::ffi::c_long,
                        (*post).strip_height as core::ffi::c_long,
                    ) as JDIMENSION,
                    (*post).strip_height,
                );
            }
            #[cfg(not(feature = "quant_2pass_supported"))]
            {
                ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
            /* QUANT_2PASS_SUPPORTED */
        } else {
            /* One-pass color quantization: just make a strip buffer. */
            (*post).buffer = ((*(*cinfo).mem).alloc_sarray)(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                (*cinfo).output_width * (*cinfo).out_color_components,
                (*post).strip_height,
            );
        }
    }
}
