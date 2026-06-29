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

// leave this as first line for PCH reasons...
//

use core::ffi::c_int;

// JPEG_INTERNALS
// jinclude.h
// jpeglib.h

// External types from JPEG library (opaque)
#[repr(C)]
pub struct jpeg_d_post_controller {
    // This is an opaque type from the JPEG library
    // We declare it but don't expose the full layout here
}

// JPEG library types
pub type JDIMENSION = core::ffi::c_uint;
pub type JSAMPLE = core::ffi::c_uchar;
pub type JSAMPARRAY = *mut *mut JSAMPLE;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type jvirt_sarray_ptr = *mut core::ffi::c_void;
pub type j_common_ptr = *mut core::ffi::c_void;
pub type j_decompress_ptr = *mut core::ffi::c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum J_BUF_MODE {
    JBUF_PASS_THRU = 0,
    JBUF_SAVE_AND_PASS = 1,
    JBUF_CRANK_DEST = 2,
}

pub const JPOOL_IMAGE: c_int = 1;
pub const JBUF_PASS_THRU: J_BUF_MODE = J_BUF_MODE::JBUF_PASS_THRU;
pub const JBUF_SAVE_AND_PASS: J_BUF_MODE = J_BUF_MODE::JBUF_SAVE_AND_PASS;
pub const JBUF_CRANK_DEST: J_BUF_MODE = J_BUF_MODE::JBUF_CRANK_DEST;

pub const JERR_BAD_BUFFER_MODE: c_int = 1;

// Private buffer controller object

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
}

// Extern declarations for JPEG library functions and types
extern "C" {
    // ERREXIT macro implementation
    fn ERREXIT(cinfo: j_decompress_ptr, msg: c_int);

    // jround_up function/macro
    fn jround_up(a: core::ffi::c_long, b: core::ffi::c_long) -> core::ffi::c_long;
}

// Stub for ERREXIT if not available from JPEG library
#[inline]
fn errexit_impl(_cinfo: j_decompress_ptr, _msg: c_int) {
    // Stub: in a real implementation, this would handle errors
}

/*
 * Initialize for a processing pass.
 */

#[no_mangle]
pub extern "C" fn start_pass_dpost(cinfo: j_decompress_ptr, pass_mode: J_BUF_MODE) {
    unsafe {
        let post = cinfo as *mut my_post_ptr as *mut my_post_controller;
        let post_deref = &mut *post;

        match pass_mode {
            J_BUF_MODE::JBUF_PASS_THRU => {
                // Access cinfo fields through offsets - this is unsafe but faithful to C
                let cinfo_ref = &mut *(cinfo as *mut j_decompress_struct);

                if cinfo_ref.quantize_colors != 0 {
                    /* Single-pass processing with color quantization. */
                    post_deref.pub_.post_process_data = Some(post_process_1pass_impl);
                    /* We could be doing buffered-image output before starting a 2-pass
                     * color quantization; in that case, jinit_d_post_controller did not
                     * allocate a strip buffer.  Use the virtual-array buffer as workspace.
                     */
                    if post_deref.buffer.is_null() {
                        // Call access_virt_sarray through mem function pointer
                        let mem = cinfo_ref.mem;
                        if !mem.is_null() {
                            let mem_ref = &*mem;
                            if let Some(access_virt_sarray) = mem_ref.access_virt_sarray {
                                post_deref.buffer = access_virt_sarray(
                                    cinfo as j_common_ptr,
                                    post_deref.whole_image,
                                    0 as JDIMENSION,
                                    post_deref.strip_height,
                                    1, // TRUE
                                );
                            }
                        }
                    }
                } else {
                    /* For single-pass processing without color quantization,
                     * I have no work to do; just call the upsampler directly.
                     */
                    if let Some(upsample) = cinfo_ref.upsample {
                        let upsample_ref = &*upsample;
                        post_deref.pub_.post_process_data = upsample_ref.upsample;
                    }
                }
            }
            #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
            J_BUF_MODE::JBUF_SAVE_AND_PASS => {
                /* First pass of 2-pass quantization */
                if post_deref.whole_image.is_null() {
                    errexit_impl(cinfo, JERR_BAD_BUFFER_MODE);
                }
                post_deref.pub_.post_process_data = Some(post_process_prepass_impl);
            }
            #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
            J_BUF_MODE::JBUF_CRANK_DEST => {
                /* Second pass of 2-pass quantization */
                if post_deref.whole_image.is_null() {
                    errexit_impl(cinfo, JERR_BAD_BUFFER_MODE);
                }
                post_deref.pub_.post_process_data = Some(post_process_2pass_impl);
            }
            _ => {
                errexit_impl(cinfo, JERR_BAD_BUFFER_MODE);
            }
        }

        post_deref.starting_row = 0 as JDIMENSION;
        post_deref.next_row = 0 as JDIMENSION;
    }
}

/*
 * Process some data in the one-pass (strip buffer) case.
 * This is used for color precision reduction as well as one-pass quantization.
 */

#[no_mangle]
pub extern "C" fn post_process_1pass_impl(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    unsafe {
        let post = cinfo as *mut my_post_ptr as *mut my_post_controller;
        let post_deref = &mut *post;

        let cinfo_ref = &mut *(cinfo as *mut j_decompress_struct);

        let mut num_rows: JDIMENSION = 0 as JDIMENSION;

        /* Fill the buffer, but not more than what we can dump out in one go. */
        /* Note we rely on the upsampler to detect bottom of image. */
        let mut max_rows = out_rows_avail - *out_row_ctr;
        if max_rows > post_deref.strip_height {
            max_rows = post_deref.strip_height;
        }
        num_rows = 0 as JDIMENSION;

        if let Some(upsample) = cinfo_ref.upsample {
            let upsample_ref = &*upsample;
            if let Some(upsample_func) = upsample_ref.upsample {
                upsample_func(
                    cinfo,
                    input_buf,
                    in_row_group_ctr,
                    in_row_groups_avail,
                    post_deref.buffer,
                    &mut num_rows,
                    max_rows,
                );
            }
        }

        /* Quantize and emit data. */
        if let Some(cquantize) = cinfo_ref.cquantize {
            let cquantize_ref = &*cquantize;
            if let Some(color_quantize) = cquantize_ref.color_quantize {
                color_quantize(
                    cinfo,
                    post_deref.buffer,
                    output_buf.add(*out_row_ctr as usize),
                    num_rows as c_int,
                );
            }
        }

        *out_row_ctr += num_rows;
    }
}

#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
/*
 * Process some data in the first pass of 2-pass quantization.
 */

#[no_mangle]
pub extern "C" fn post_process_prepass_impl(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    unsafe {
        let post = cinfo as *mut my_post_ptr as *mut my_post_controller;
        let post_deref = &mut *post;

        let cinfo_ref = &mut *(cinfo as *mut j_decompress_struct);

        /* Reposition virtual buffer if at start of strip. */
        if post_deref.next_row == 0 {
            let mem = cinfo_ref.mem;
            if !mem.is_null() {
                let mem_ref = &*mem;
                if let Some(access_virt_sarray) = mem_ref.access_virt_sarray {
                    post_deref.buffer = access_virt_sarray(
                        cinfo as j_common_ptr,
                        post_deref.whole_image,
                        post_deref.starting_row,
                        post_deref.strip_height,
                        1, // TRUE
                    );
                }
            }
        }

        /* Upsample some data (up to a strip height's worth). */
        let old_next_row = post_deref.next_row;
        if let Some(upsample) = cinfo_ref.upsample {
            let upsample_ref = &*upsample;
            if let Some(upsample_func) = upsample_ref.upsample {
                upsample_func(
                    cinfo,
                    input_buf,
                    in_row_group_ctr,
                    in_row_groups_avail,
                    post_deref.buffer,
                    &mut post_deref.next_row,
                    post_deref.strip_height,
                );
            }
        }

        /* Allow quantizer to scan new data.  No data is emitted, */
        /* but we advance out_row_ctr so outer loop can tell when we're done. */
        if post_deref.next_row > old_next_row {
            let num_rows = post_deref.next_row - old_next_row;
            if let Some(cquantize) = cinfo_ref.cquantize {
                let cquantize_ref = &*cquantize;
                if let Some(color_quantize) = cquantize_ref.color_quantize {
                    color_quantize(
                        cinfo,
                        post_deref.buffer.add(old_next_row as usize),
                        core::ptr::null_mut(),
                        num_rows as c_int,
                    );
                }
            }
            *out_row_ctr += num_rows;
        }

        /* Advance if we filled the strip. */
        if post_deref.next_row >= post_deref.strip_height {
            post_deref.starting_row += post_deref.strip_height;
            post_deref.next_row = 0 as JDIMENSION;
        }
    }
}

#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
/*
 * Process some data in the second pass of 2-pass quantization.
 */

#[no_mangle]
pub extern "C" fn post_process_2pass_impl(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    unsafe {
        let post = cinfo as *mut my_post_ptr as *mut my_post_controller;
        let post_deref = &mut *post;

        let cinfo_ref = &mut *(cinfo as *mut j_decompress_struct);

        /* Reposition virtual buffer if at start of strip. */
        if post_deref.next_row == 0 {
            let mem = cinfo_ref.mem;
            if !mem.is_null() {
                let mem_ref = &*mem;
                if let Some(access_virt_sarray) = mem_ref.access_virt_sarray {
                    post_deref.buffer = access_virt_sarray(
                        cinfo as j_common_ptr,
                        post_deref.whole_image,
                        post_deref.starting_row,
                        post_deref.strip_height,
                        0, // FALSE
                    );
                }
            }
        }

        /* Determine number of rows to emit. */
        let mut num_rows = post_deref.strip_height - post_deref.next_row; /* available in strip */
        let max_rows = out_rows_avail - *out_row_ctr; /* available in output area */
        if num_rows > max_rows {
            num_rows = max_rows;
        }
        /* We have to check bottom of image here, can't depend on upsampler. */
        let max_rows = cinfo_ref.output_height - post_deref.starting_row;
        if num_rows > max_rows {
            num_rows = max_rows;
        }

        /* Quantize and emit data. */
        if let Some(cquantize) = cinfo_ref.cquantize {
            let cquantize_ref = &*cquantize;
            if let Some(color_quantize) = cquantize_ref.color_quantize {
                color_quantize(
                    cinfo,
                    post_deref.buffer.add(post_deref.next_row as usize),
                    output_buf.add(*out_row_ctr as usize),
                    num_rows as c_int,
                );
            }
        }
        *out_row_ctr += num_rows;

        /* Advance if we filled the strip. */
        post_deref.next_row += num_rows;
        if post_deref.next_row >= post_deref.strip_height {
            post_deref.starting_row += post_deref.strip_height;
            post_deref.next_row = 0 as JDIMENSION;
        }
    }
}

/*
 * Initialize postprocessing controller.
 */

#[no_mangle]
pub extern "C" fn jinit_d_post_controller(
    cinfo: j_decompress_ptr,
    need_full_buffer: core::ffi::c_int,
) {
    unsafe {
        let cinfo_ref = &mut *(cinfo as *mut j_decompress_struct);

        let post: my_post_ptr = {
            let mem = cinfo_ref.mem;
            if mem.is_null() {
                return;
            }
            let mem_ref = &*mem;
            if let Some(alloc_small) = mem_ref.alloc_small {
                alloc_small(
                    cinfo as j_common_ptr,
                    JPOOL_IMAGE,
                    core::mem::size_of::<my_post_controller>(),
                ) as *mut my_post_controller
            } else {
                return;
            }
        };

        if post.is_null() {
            return;
        }

        let post_deref = &mut *post;

        // Initialize public fields
        post_deref.pub_.post_process_data = None;

        post_deref.whole_image = core::ptr::null_mut(); /* flag for no virtual arrays */
        post_deref.buffer = core::ptr::null_mut();      /* flag for no strip buffer */

        // Set up start_pass function
        post_deref.pub_.start_pass = Some(start_pass_dpost);

        cinfo_ref.post = &mut post_deref.pub_ as *mut jpeg_d_post_controller;

        /* Create the quantization buffer, if needed */
        if cinfo_ref.quantize_colors != 0 {
            /* The buffer strip height is max_v_samp_factor, which is typically
             * an efficient number of rows for upsampling to return.
             * (In the presence of output rescaling, we might want to be smarter?)
             */
            post_deref.strip_height = cinfo_ref.max_v_samp_factor as JDIMENSION;
            if need_full_buffer != 0 {
                /* Two-pass color quantization: need full-image storage. */
                /* We round up the number of rows to a multiple of the strip height. */
                #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
                {
                    let mem = cinfo_ref.mem;
                    if !mem.is_null() {
                        let mem_ref = &*mem;
                        if let Some(request_virt_sarray) = mem_ref.request_virt_sarray {
                            let rounded_height = jround_up(
                                cinfo_ref.output_height as core::ffi::c_long,
                                post_deref.strip_height as core::ffi::c_long,
                            ) as JDIMENSION;

                            post_deref.whole_image = request_virt_sarray(
                                cinfo as j_common_ptr,
                                JPOOL_IMAGE,
                                0, // FALSE
                                (cinfo_ref.output_width as JDIMENSION)
                                    * (cinfo_ref.out_color_components as JDIMENSION),
                                rounded_height,
                                post_deref.strip_height,
                            );
                        }
                    }
                }

                #[cfg(not(feature = "QUANT_2PASS_SUPPORTED"))]
                {
                    errexit_impl(cinfo, JERR_BAD_BUFFER_MODE);
                }
            } else {
                /* One-pass color quantization: just make a strip buffer. */
                let mem = cinfo_ref.mem;
                if !mem.is_null() {
                    let mem_ref = &*mem;
                    if let Some(alloc_sarray) = mem_ref.alloc_sarray {
                        post_deref.buffer = alloc_sarray(
                            cinfo as j_common_ptr,
                            JPOOL_IMAGE,
                            (cinfo_ref.output_width as JDIMENSION)
                                * (cinfo_ref.out_color_components as JDIMENSION),
                            post_deref.strip_height,
                        );
                    }
                }
            }
        }
    }
}

// Opaque JPEG decompressor structure - only used through pointers
#[repr(C)]
pub struct j_decompress_struct {
    // These fields are declared to allow unsafe access
    // In a real implementation, these would match the actual JPEG library struct
    pub post: *mut jpeg_d_post_controller,
    pub quantize_colors: c_int,
    pub upsample: *mut core::ffi::c_void,
    pub cquantize: *mut core::ffi::c_void,
    pub mem: *mut jpeg_memory_mgr,
    pub output_width: core::ffi::c_uint,
    pub out_color_components: core::ffi::c_int,
    pub output_height: JDIMENSION,
    pub max_v_samp_factor: core::ffi::c_int,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<
        extern "C" fn(
            j_common_ptr: j_common_ptr,
            pool_id: c_int,
            sizeofobject: usize,
        ) -> *mut core::ffi::c_void,
    >,
    pub alloc_sarray: Option<
        extern "C" fn(
            j_common_ptr: j_common_ptr,
            pool_id: c_int,
            samplesperrow: JDIMENSION,
            numrows: JDIMENSION,
        ) -> JSAMPARRAY,
    >,
    pub request_virt_sarray: Option<
        extern "C" fn(
            j_common_ptr: j_common_ptr,
            pool_id: c_int,
            pre_zero: c_int,
            samplesperrow: JDIMENSION,
            numrows: JDIMENSION,
            maxaccess: JDIMENSION,
        ) -> jvirt_sarray_ptr,
    >,
    pub access_virt_sarray: Option<
        extern "C" fn(
            j_common_ptr: j_common_ptr,
            virt_array: jvirt_sarray_ptr,
            start_row: JDIMENSION,
            num_rows: JDIMENSION,
            writable: c_int,
        ) -> JSAMPARRAY,
    >,
}

#[repr(C)]
pub struct jpeg_upsampler {
    pub upsample: Option<
        extern "C" fn(
            cinfo: j_decompress_ptr,
            input_buf: JSAMPIMAGE,
            in_row_group_ctr: *mut JDIMENSION,
            in_row_groups_avail: JDIMENSION,
            output_buf: JSAMPARRAY,
            out_row_ctr: *mut JDIMENSION,
            out_rows_avail: JDIMENSION,
        ),
    >,
}

#[repr(C)]
pub struct jpeg_color_quantizer {
    pub color_quantize: Option<
        extern "C" fn(
            cinfo: j_decompress_ptr,
            input_buf: JSAMPARRAY,
            output_buf: JSAMPARRAY,
            num_rows: c_int,
        ),
    >,
}
