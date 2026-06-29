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

use core::ffi::c_int;
use core::mem;
use core::ptr;

// ============================================================================
// External JPEG library types and structures (from jpeglib.h, jinclude.h)
// ============================================================================

// Opaque types from JPEG library
#[repr(C)]
pub struct jpeg_d_post_controller {
    // public fields
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct j_decompress_struct {
    // Opaque structure - full definition is in JPEG library
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    // Opaque structure
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_upsampler {
    // Opaque structure
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_color_quantizer {
    // Opaque structure
    _opaque: [u8; 0],
}

pub type j_decompress_ptr = *mut j_decompress_struct;
pub type j_common_ptr = *mut core::ffi::c_void;
pub type jvirt_sarray_ptr = *mut core::ffi::c_void;
pub type JSAMPARRAY = *mut *mut u8;
pub type JSAMPIMAGE = *mut *mut u8;
pub type JDIMENSION = u32;
pub type boolean = i32;

// J_BUF_MODE enum values
pub const JBUF_PASS_THRU: i32 = 0;
pub const JBUF_SAVE_AND_PASS: i32 = 1;
pub const JBUF_CRANK_DEST: i32 = 2;

// Error codes
pub const JERR_BAD_BUFFER_MODE: i32 = 130;

// Memory pool constants
pub const JPOOL_IMAGE: i32 = 1;

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
    pub buffer: JSAMPARRAY, /* strip buffer, or current strip of virtual */
    pub strip_height: JDIMENSION, /* buffer size in rows */
    /* for two-pass mode only: */
    pub starting_row: JDIMENSION, /* row # of first row in current strip */
    pub next_row: JDIMENSION, /* index of next row to fill/empty in strip */
}

pub type my_post_ptr = *mut my_post_controller;

/* Forward declarations */
extern "C" {
    fn post_process_1pass(
        cinfo: j_decompress_ptr,
        input_buf: JSAMPIMAGE,
        in_row_group_ctr: *mut JDIMENSION,
        in_row_groups_avail: JDIMENSION,
        output_buf: JSAMPARRAY,
        out_row_ctr: *mut JDIMENSION,
        out_rows_avail: JDIMENSION,
    );

    #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
    fn post_process_prepass(
        cinfo: j_decompress_ptr,
        input_buf: JSAMPIMAGE,
        in_row_group_ctr: *mut JDIMENSION,
        in_row_groups_avail: JDIMENSION,
        output_buf: JSAMPARRAY,
        out_row_ctr: *mut JDIMENSION,
        out_rows_avail: JDIMENSION,
    );

    #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
    fn post_process_2pass(
        cinfo: j_decompress_ptr,
        input_buf: JSAMPIMAGE,
        in_row_group_ctr: *mut JDIMENSION,
        in_row_groups_avail: JDIMENSION,
        output_buf: JSAMPARRAY,
        out_row_ctr: *mut JDIMENSION,
        out_rows_avail: JDIMENSION,
    );

    fn jround_up(a: i32, b: i32) -> i32;
    fn ERREXIT(cinfo: j_decompress_ptr, code: i32) -> !;
}

/*
 * Initialize for a processing pass.
 */

unsafe extern "C" fn start_pass_dpost(cinfo: j_decompress_ptr, pass_mode: i32) {
    let post = (*cinfo).post as my_post_ptr;

    match pass_mode {
        JBUF_PASS_THRU => {
            if (*cinfo).quantize_colors != 0 {
                /* Single-pass processing with color quantization. */
                (*post).pub_.post_process_data = post_process_1pass;
                /* We could be doing buffered-image output before starting a 2-pass
                 * color quantization; in that case, jinit_d_post_controller did not
                 * allocate a strip buffer.  Use the virtual-array buffer as workspace.
                 */
                if (*post).buffer == ptr::null_mut() {
                    (*post).buffer = ((*(*cinfo).mem).access_virt_sarray)(
                        cinfo as j_common_ptr,
                        (*post).whole_image,
                        0 as JDIMENSION,
                        (*post).strip_height,
                        1 as i32,
                    );
                }
            } else {
                /* For single-pass processing without color quantization,
                 * I have no work to do; just call the upsampler directly.
                 */
                (*post).pub_.post_process_data = (*(*cinfo).upsample).upsample;
            }
        }
        #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
        JBUF_SAVE_AND_PASS => {
            /* First pass of 2-pass quantization */
            if (*post).whole_image == ptr::null_mut() {
                ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
            (*post).pub_.post_process_data = post_process_prepass;
        }
        #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
        JBUF_CRANK_DEST => {
            /* Second pass of 2-pass quantization */
            if (*post).whole_image == ptr::null_mut() {
                ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
            (*post).pub_.post_process_data = post_process_2pass;
        }
        _ => {
            ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
        }
    }
    (*post).starting_row = 0 as JDIMENSION;
    (*post).next_row = 0 as JDIMENSION;
}

/*
 * Process some data in the one-pass (strip buffer) case.
 * This is used for color precision reduction as well as one-pass quantization.
 */

unsafe extern "C" fn post_process_1pass(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    let post = (*cinfo).post as my_post_ptr;
    let mut num_rows: JDIMENSION;
    let mut max_rows: JDIMENSION;

    /* Fill the buffer, but not more than what we can dump out in one go. */
    /* Note we rely on the upsampler to detect bottom of image. */
    max_rows = out_rows_avail - *out_row_ctr;
    if max_rows > (*post).strip_height {
        max_rows = (*post).strip_height;
    }
    num_rows = 0 as JDIMENSION;
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
        (*post).buffer.add(*out_row_ctr as usize),
        output_buf,
        num_rows as c_int,
    );
    *out_row_ctr += num_rows;
}

#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
unsafe extern "C" fn post_process_prepass(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    let post = (*cinfo).post as my_post_ptr;
    let mut old_next_row: JDIMENSION;
    let mut num_rows: JDIMENSION;

    /* Reposition virtual buffer if at start of strip. */
    if (*post).next_row == 0 as JDIMENSION {
        (*post).buffer = ((*(*cinfo).mem).access_virt_sarray)(
            cinfo as j_common_ptr,
            (*post).whole_image,
            (*post).starting_row,
            (*post).strip_height,
            1 as i32,
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
            ptr::null_mut(),
            num_rows as c_int,
        );
        *out_row_ctr += num_rows;
    }

    /* Advance if we filled the strip. */
    if (*post).next_row >= (*post).strip_height {
        (*post).starting_row += (*post).strip_height;
        (*post).next_row = 0 as JDIMENSION;
    }
}

#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
unsafe extern "C" fn post_process_2pass(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    let post = (*cinfo).post as my_post_ptr;
    let mut num_rows: JDIMENSION;
    let mut max_rows: JDIMENSION;

    /* Reposition virtual buffer if at start of strip. */
    if (*post).next_row == 0 as JDIMENSION {
        (*post).buffer = ((*(*cinfo).mem).access_virt_sarray)(
            cinfo as j_common_ptr,
            (*post).whole_image,
            (*post).starting_row,
            (*post).strip_height,
            0 as i32,
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
        num_rows as c_int,
    );
    *out_row_ctr += num_rows;

    /* Advance if we filled the strip. */
    (*post).next_row += num_rows;
    if (*post).next_row >= (*post).strip_height {
        (*post).starting_row += (*post).strip_height;
        (*post).next_row = 0 as JDIMENSION;
    }
}

/*
 * Initialize postprocessing controller.
 */

pub unsafe extern "C" fn jinit_d_post_controller(
    cinfo: j_decompress_ptr,
    need_full_buffer: boolean,
) {
    let post: my_post_ptr;

    post = ((*(*cinfo).mem).alloc_small)(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        mem::size_of::<my_post_controller>(),
    ) as my_post_ptr;
    (*cinfo).post =
        &mut (*post).pub_ as *mut jpeg_d_post_controller as *mut core::ffi::c_void;
    (*post).pub_.start_pass = start_pass_dpost;
    (*post).whole_image = ptr::null_mut(); /* flag for no virtual arrays */
    (*post).buffer = ptr::null_mut(); /* flag for no strip buffer */

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
            #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
            {
                (*post).whole_image = ((*(*cinfo).mem).request_virt_sarray)(
                    cinfo as j_common_ptr,
                    JPOOL_IMAGE,
                    0 as i32,
                    ((*cinfo).output_width * (*cinfo).out_color_components) as u32,
                    jround_up(
                        (*cinfo).output_height as i32,
                        (*post).strip_height as i32,
                    ) as JDIMENSION,
                    (*post).strip_height,
                );
            }
            #[cfg(not(feature = "QUANT_2PASS_SUPPORTED"))]
            {
                ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
        } else {
            /* One-pass color quantization: just make a strip buffer. */
            (*post).buffer = ((*(*cinfo).mem).alloc_sarray)(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                ((*cinfo).output_width * (*cinfo).out_color_components) as u32,
                (*post).strip_height,
            );
        }
    }
}
