/*
 * jdsample.c
 *
 * Copyright (C) 1991-1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains upsampling routines.
 *
 * Upsampling input data is counted in "row groups".  A row group
 * is defined to be (v_samp_factor * DCT_scaled_size / min_DCT_scaled_size)
 * sample rows of each component.  Upsampling will normally produce
 * max_v_samp_factor pixel rows from each row group (but this could vary
 * if the upsampler is applying a scale factor of its own).
 *
 * An excellent reference for image resampling is
 *   Digital Image Warping, George Wolberg, 1990.
 *   Pub. by IEEE Computer Society Press, Los Alamitos, CA. ISBN 0-8186-8944-7.
 */
//Anything above this #include will be ignored by the compiler
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code,
         unused_variables, unused_mut, unused_assignments, unsafe_op_in_unsafe_fn)]

// #include "../qcommon/exe_headers.h"
use crate::codemp::qcommon::exe_headers_h::*;
// #define JPEG_INTERNALS
// #include "jinclude.h"
use crate::codemp::jpeg_6::jinclude_h::*;
// #include "jpeglib.h"
use crate::codemp::jpeg_6::jpeglib_h::*;
// porting note: JPEG_INTERNALS defined before jpeglib.h causes jpeglib.h to pull in jpegint.h;
// import it explicitly here to bring those symbols into scope.
use crate::codemp::jpeg_6::jpegint_h::*;

use core::ffi::{c_int, c_long};


/* Pointer to routine to upsample a single component */
// JMETHOD(void, upsample1_ptr, (...)) expands to: typedef void (*upsample1_ptr)(...)
type upsample1_ptr = unsafe extern "C" fn(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
);

/* Private subobject */

#[repr(C)]
struct my_upsampler {
    pub_: jpeg_upsampler,	/* public fields */
    // porting note: C field named `pub` renamed to `pub_`; `pub` is a reserved keyword in Rust.

    /* Color conversion buffer.  When using separate upsampling and color
     * conversion steps, this buffer holds one upsampled row group until it
     * has been color converted and output.
     * Note: we do not allocate any storage for component(s) which are full-size,
     * ie do not need rescaling.  The corresponding entry of color_buf[] is
     * simply set to point to the input data array, thereby avoiding copying.
     */
    color_buf: [JSAMPARRAY; MAX_COMPONENTS as usize],

    /* Per-component upsampling method pointers */
    methods: [upsample1_ptr; MAX_COMPONENTS as usize],

    next_row_out: c_int,		/* counts rows emitted from color_buf */
    rows_to_go: JDIMENSION,	/* counts rows remaining in image */

    /* Height of an input row group for each component. */
    rowgroup_height: [c_int; MAX_COMPONENTS as usize],

    /* These arrays save pixel expansion factors so that int_expand need not
     * recompute them each time.  They are unused for other upsampling methods.
     */
    h_expand: [UINT8; MAX_COMPONENTS as usize],
    v_expand: [UINT8; MAX_COMPONENTS as usize],
}

type my_upsample_ptr = *mut my_upsampler;


/*
 * Initialize for an upsampling pass.
 */

unsafe extern "C" fn start_pass_upsample(cinfo: j_decompress_ptr) {
    let upsample = (*cinfo).upsample as my_upsample_ptr;

    /* Mark the conversion buffer empty */
    (*upsample).next_row_out = (*cinfo).max_v_samp_factor;
    /* Initialize total-height counter for detecting bottom of image */
    (*upsample).rows_to_go = (*cinfo).output_height;
}


/*
 * Control routine to do upsampling (and color conversion).
 *
 * In this version we upsample each component independently.
 * We upsample one row group into the conversion buffer, then apply
 * color conversion a row at a time.
 */

unsafe extern "C" fn sep_upsample(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    let upsample = (*cinfo).upsample as my_upsample_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut num_rows: JDIMENSION;

    /* Fill the conversion buffer, if it's empty */
    if (*upsample).next_row_out >= (*cinfo).max_v_samp_factor {
        ci = 0;
        compptr = (*cinfo).comp_info;
        while ci < (*cinfo).num_components {
            /* Invoke per-component upsample method.  Notice we pass a POINTER
             * to color_buf[ci], so that fullsize_upsample can change it.
             */
            ((*upsample).methods[ci as usize])(
                cinfo,
                compptr,
                (*input_buf.add(ci as usize))
                    .add((*in_row_group_ctr as usize) * (*upsample).rowgroup_height[ci as usize] as usize),
                (*upsample).color_buf.as_mut_ptr().add(ci as usize),
            );
            ci += 1;
            compptr = compptr.add(1);
        }
        (*upsample).next_row_out = 0;
    }

    /* Color-convert and emit rows */

    /* How many we have in the buffer: */
    num_rows = ((*cinfo).max_v_samp_factor - (*upsample).next_row_out) as JDIMENSION;
    /* Not more than the distance to the end of the image.  Need this test
     * in case the image height is not a multiple of max_v_samp_factor:
     */
    if num_rows > (*upsample).rows_to_go {
        num_rows = (*upsample).rows_to_go;
    }
    /* And not more than what the client can accept: */
    let mut out_rows_avail = out_rows_avail - *out_row_ctr;
    if num_rows > out_rows_avail {
        num_rows = out_rows_avail;
    }

    let color_convert = (*(*cinfo).cconvert).color_convert;
    color_convert(
        cinfo,
        (*upsample).color_buf.as_mut_ptr(),
        (*upsample).next_row_out as JDIMENSION,
        output_buf.add(*out_row_ctr as usize),
        num_rows as c_int,
    );

    /* Adjust counts */
    *out_row_ctr += num_rows;
    (*upsample).rows_to_go -= num_rows;
    (*upsample).next_row_out += num_rows as c_int;
    /* When the buffer is emptied, declare this input row group consumed */
    if (*upsample).next_row_out >= (*cinfo).max_v_samp_factor {
        (*in_row_group_ctr) += 1;
    }
}


/*
 * These are the routines invoked by sep_upsample to upsample pixel values
 * of a single component.  One row group is processed per call.
 */


/*
 * For full-size components, we just make color_buf[ci] point at the
 * input buffer, and thus avoid copying any data.  Note that this is
 * safe only because sep_upsample doesn't declare the input row group
 * "consumed" until we are done color converting and emitting it.
 */

unsafe extern "C" fn fullsize_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    *output_data_ptr = input_data;
}


/*
 * This is a no-op version used for "uninteresting" components.
 * These components will not be referenced by color conversion.
 */

unsafe extern "C" fn noop_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    *output_data_ptr = core::ptr::null_mut();	/* safety check */
}


/*
 * This version handles any integral sampling ratios.
 * This is not used for typical JPEG files, so it need not be fast.
 * Nor, for that matter, is it particularly accurate: the algorithm is
 * simple replication of the input pixel onto the corresponding output
 * pixels.  The hi-falutin sampling literature refers to this as a
 * "box filter".  A box filter tends to introduce visible artifacts,
 * so if you are actually going to use 3:1 or 4:1 sampling ratios
 * you would be well advised to improve this code.
 */

unsafe extern "C" fn int_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let upsample = (*cinfo).upsample as my_upsample_ptr;
    let output_data: JSAMPARRAY = *output_data_ptr;
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut invalue: JSAMPLE;
    let mut h: c_int;
    let mut outend: JSAMPROW;
    let mut h_expand: c_int;
    let mut v_expand: c_int;
    let mut inrow: c_int;
    let mut outrow: c_int;

    h_expand = (*upsample).h_expand[(*compptr).component_index as usize] as c_int;
    v_expand = (*upsample).v_expand[(*compptr).component_index as usize] as c_int;

    inrow = 0;
    outrow = 0;
    while outrow < (*cinfo).max_v_samp_factor {
        /* Generate one output row with proper horizontal expansion */
        inptr = *input_data.add(inrow as usize);
        outptr = *output_data.add(outrow as usize);
        outend = outptr.add((*cinfo).output_width as usize);
        while outptr < outend {
            invalue = *inptr;	/* don't need GETJSAMPLE() here */
            inptr = inptr.add(1);
            h = h_expand;
            while h > 0 {
                *outptr = invalue;
                outptr = outptr.add(1);
                h -= 1;
            }
        }
        /* Generate any additional output rows by duplicating the first one */
        if v_expand > 1 {
            jcopy_sample_rows(
                output_data,
                outrow,
                output_data,
                outrow + 1,
                v_expand - 1,
                (*cinfo).output_width,
            );
        }
        inrow += 1;
        outrow += v_expand;
    }
}


/*
 * Fast processing for the common case of 2:1 horizontal and 1:1 vertical.
 * It's still a box filter.
 */

unsafe extern "C" fn h2v1_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let output_data: JSAMPARRAY = *output_data_ptr;
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut invalue: JSAMPLE;
    let mut outend: JSAMPROW;
    let mut inrow: c_int;

    inrow = 0;
    while inrow < (*cinfo).max_v_samp_factor {
        inptr = *input_data.add(inrow as usize);
        outptr = *output_data.add(inrow as usize);
        outend = outptr.add((*cinfo).output_width as usize);
        while outptr < outend {
            invalue = *inptr;	/* don't need GETJSAMPLE() here */
            inptr = inptr.add(1);
            *outptr = invalue;
            outptr = outptr.add(1);
            *outptr = invalue;
            outptr = outptr.add(1);
        }
        inrow += 1;
    }
}


/*
 * Fast processing for the common case of 2:1 horizontal and 2:1 vertical.
 * It's still a box filter.
 */

unsafe extern "C" fn h2v2_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let output_data: JSAMPARRAY = *output_data_ptr;
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut invalue: JSAMPLE;
    let mut outend: JSAMPROW;
    let mut inrow: c_int;
    let mut outrow: c_int;

    inrow = 0;
    outrow = 0;
    while outrow < (*cinfo).max_v_samp_factor {
        inptr = *input_data.add(inrow as usize);
        outptr = *output_data.add(outrow as usize);
        outend = outptr.add((*cinfo).output_width as usize);
        while outptr < outend {
            invalue = *inptr;	/* don't need GETJSAMPLE() here */
            inptr = inptr.add(1);
            *outptr = invalue;
            outptr = outptr.add(1);
            *outptr = invalue;
            outptr = outptr.add(1);
        }
        jcopy_sample_rows(output_data, outrow, output_data, outrow + 1,
                          1, (*cinfo).output_width);
        inrow += 1;
        outrow += 2;
    }
}


/*
 * Fancy processing for the common case of 2:1 horizontal and 1:1 vertical.
 *
 * The upsampling algorithm is linear interpolation between pixel centers,
 * also known as a "triangle filter".  This is a good compromise between
 * speed and visual quality.  The centers of the output pixels are 1/4 and 3/4
 * of the way between input pixel centers.
 *
 * A note about the "bias" calculations: when rounding fractional values to
 * integer, we do not want to always round 0.5 up to the next integer.
 * If we did that, we'd introduce a noticeable bias towards larger values.
 * Instead, this code is arranged so that 0.5 will be rounded up or down at
 * alternate pixel locations (a simple ordered dither pattern).
 */

unsafe extern "C" fn h2v1_fancy_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let output_data: JSAMPARRAY = *output_data_ptr;
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut invalue: c_int;
    let mut colctr: JDIMENSION;
    let mut inrow: c_int;

    inrow = 0;
    while inrow < (*cinfo).max_v_samp_factor {
        inptr = *input_data.add(inrow as usize);
        outptr = *output_data.add(inrow as usize);
        /* Special case for first column */
        invalue = GETJSAMPLE(*inptr);
        inptr = inptr.add(1);
        *outptr = invalue as JSAMPLE;
        outptr = outptr.add(1);
        *outptr = ((invalue * 3 + GETJSAMPLE(*inptr) + 2) >> 2) as JSAMPLE;
        outptr = outptr.add(1);

        colctr = (*compptr).downsampled_width.wrapping_sub(2);
        while colctr > 0 {
            /* General case: 3/4 * nearer pixel + 1/4 * further pixel */
            invalue = GETJSAMPLE(*inptr) * 3;
            inptr = inptr.add(1);
            *outptr = ((invalue + GETJSAMPLE(*inptr.offset(-2)) + 1) >> 2) as JSAMPLE;
            outptr = outptr.add(1);
            *outptr = ((invalue + GETJSAMPLE(*inptr) + 2) >> 2) as JSAMPLE;
            outptr = outptr.add(1);
            colctr -= 1;
        }

        /* Special case for last column */
        invalue = GETJSAMPLE(*inptr);
        *outptr = ((invalue * 3 + GETJSAMPLE(*inptr.offset(-1)) + 1) >> 2) as JSAMPLE;
        outptr = outptr.add(1);
        *outptr = invalue as JSAMPLE;
        outptr = outptr.add(1);
        inrow += 1;
    }
}


/*
 * Fancy processing for the common case of 2:1 horizontal and 2:1 vertical.
 * Again a triangle filter; see comments for h2v1 case, above.
 *
 * It is OK for us to reference the adjacent input rows because we demanded
 * context from the main buffer controller (see initialization code).
 */

unsafe extern "C" fn h2v2_fancy_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let output_data: JSAMPARRAY = *output_data_ptr;
    let mut inptr0: JSAMPROW;
    let mut inptr1: JSAMPROW;
    let mut outptr: JSAMPROW;
    // #if BITS_IN_JSAMPLE == 8
    #[cfg(feature = "bits_in_jsample_8")]
    let (mut thiscolsum, mut lastcolsum, mut nextcolsum): (c_int, c_int, c_int) = (0, 0, 0);
    // #else
    #[cfg(not(feature = "bits_in_jsample_8"))]
    let (mut thiscolsum, mut lastcolsum, mut nextcolsum): (INT32, INT32, INT32) = (0, 0, 0);
    // #endif
    let mut colctr: JDIMENSION;
    let mut inrow: c_int;
    let mut outrow: c_int;
    let mut v: c_int;

    inrow = 0;
    outrow = 0;
    while outrow < (*cinfo).max_v_samp_factor {
        v = 0;
        while v < 2 {
            /* inptr0 points to nearest input row, inptr1 points to next nearest */
            inptr0 = *input_data.offset(inrow as isize);
            if v == 0 {		/* next nearest is row above */
                inptr1 = *input_data.offset((inrow - 1) as isize);
            } else {			/* next nearest is row below */
                inptr1 = *input_data.offset((inrow + 1) as isize);
            }
            outptr = *output_data.add(outrow as usize);
            outrow += 1;

            /* Special case for first column */
            thiscolsum = GETJSAMPLE(*inptr0) * 3 + GETJSAMPLE(*inptr1);
            inptr0 = inptr0.add(1);
            inptr1 = inptr1.add(1);
            nextcolsum = GETJSAMPLE(*inptr0) * 3 + GETJSAMPLE(*inptr1);
            inptr0 = inptr0.add(1);
            inptr1 = inptr1.add(1);
            *outptr = ((thiscolsum * 4 + 8) >> 4) as JSAMPLE;
            outptr = outptr.add(1);
            *outptr = ((thiscolsum * 3 + nextcolsum + 7) >> 4) as JSAMPLE;
            outptr = outptr.add(1);
            lastcolsum = thiscolsum; thiscolsum = nextcolsum;

            colctr = (*compptr).downsampled_width.wrapping_sub(2);
            while colctr > 0 {
                /* General case: 3/4 * nearer pixel + 1/4 * further pixel in each */
                /* dimension, thus 9/16, 3/16, 3/16, 1/16 overall */
                nextcolsum = GETJSAMPLE(*inptr0) * 3 + GETJSAMPLE(*inptr1);
                inptr0 = inptr0.add(1);
                inptr1 = inptr1.add(1);
                *outptr = ((thiscolsum * 3 + lastcolsum + 8) >> 4) as JSAMPLE;
                outptr = outptr.add(1);
                *outptr = ((thiscolsum * 3 + nextcolsum + 7) >> 4) as JSAMPLE;
                outptr = outptr.add(1);
                lastcolsum = thiscolsum; thiscolsum = nextcolsum;
                colctr -= 1;
            }

            /* Special case for last column */
            *outptr = ((thiscolsum * 3 + lastcolsum + 8) >> 4) as JSAMPLE;
            outptr = outptr.add(1);
            *outptr = ((thiscolsum * 4 + 7) >> 4) as JSAMPLE;
            outptr = outptr.add(1);
            v += 1;
        }
        inrow += 1;
    }
}


/*
 * Module initialization routine for upsampling.
 */

pub unsafe fn jinit_upsampler(cinfo: j_decompress_ptr) {
    let mut upsample: my_upsample_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut need_buffer: boolean;
    let mut do_fancy: boolean;
    let mut h_in_group: c_int;
    let mut v_in_group: c_int;
    let mut h_out_group: c_int;
    let mut v_out_group: c_int;

    upsample = ((*(*cinfo).mem).alloc_small)(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        core::mem::size_of::<my_upsampler>(),
    ) as my_upsample_ptr;
    (*cinfo).upsample = upsample as *mut jpeg_upsampler;
    (*upsample).pub_.start_pass = Some(start_pass_upsample);
    (*upsample).pub_.upsample = Some(sep_upsample);
    (*upsample).pub_.need_context_rows = FALSE; /* until we find out differently */

    if (*cinfo).CCIR601_sampling != FALSE {	/* this isn't supported */
        ERREXIT(cinfo, JERR_CCIR601_NOTIMPL);
    }

    /* jdmainct.c doesn't support context rows when min_DCT_scaled_size = 1,
     * so don't ask for it.
     */
    do_fancy = if (*cinfo).do_fancy_upsampling != FALSE && (*cinfo).min_DCT_scaled_size > 1 {
        TRUE
    } else {
        FALSE
    };

    /* Verify we can handle the sampling factors, select per-component methods,
     * and create storage as needed.
     */
    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Compute size of an "input group" after IDCT scaling.  This many samples
         * are to be converted to max_h_samp_factor * max_v_samp_factor pixels.
         */
        h_in_group = ((*compptr).h_samp_factor * (*compptr).DCT_scaled_size) /
                     (*cinfo).min_DCT_scaled_size;
        v_in_group = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size) /
                     (*cinfo).min_DCT_scaled_size;
        h_out_group = (*cinfo).max_h_samp_factor;
        v_out_group = (*cinfo).max_v_samp_factor;
        (*upsample).rowgroup_height[ci as usize] = v_in_group; /* save for use later */
        need_buffer = TRUE;
        if (*compptr).component_needed == FALSE {
            /* Don't bother to upsample an uninteresting component. */
            (*upsample).methods[ci as usize] = noop_upsample;
            need_buffer = FALSE;
        } else if h_in_group == h_out_group && v_in_group == v_out_group {
            /* Fullsize components can be processed without any work. */
            (*upsample).methods[ci as usize] = fullsize_upsample;
            need_buffer = FALSE;
        } else if h_in_group * 2 == h_out_group &&
                  v_in_group == v_out_group {
            /* Special cases for 2h1v upsampling */
            if do_fancy != FALSE && (*compptr).downsampled_width > 2 {
                (*upsample).methods[ci as usize] = h2v1_fancy_upsample;
            } else {
                (*upsample).methods[ci as usize] = h2v1_upsample;
            }
        } else if h_in_group * 2 == h_out_group &&
                  v_in_group * 2 == v_out_group {
            /* Special cases for 2h2v upsampling */
            if do_fancy != FALSE && (*compptr).downsampled_width > 2 {
                (*upsample).methods[ci as usize] = h2v2_fancy_upsample;
                (*upsample).pub_.need_context_rows = TRUE;
            } else {
                (*upsample).methods[ci as usize] = h2v2_upsample;
            }
        } else if (h_out_group % h_in_group) == 0 &&
                  (v_out_group % v_in_group) == 0 {
            /* Generic integral-factors upsampling method */
            (*upsample).methods[ci as usize] = int_upsample;
            (*upsample).h_expand[ci as usize] = (h_out_group / h_in_group) as UINT8;
            (*upsample).v_expand[ci as usize] = (v_out_group / v_in_group) as UINT8;
        } else {
            ERREXIT(cinfo, JERR_FRACT_SAMPLE_NOTIMPL);
        }
        if need_buffer != FALSE {
            (*upsample).color_buf[ci as usize] = ((*(*cinfo).mem).alloc_sarray)(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                jround_up((*cinfo).output_width as c_long,
                          (*cinfo).max_h_samp_factor as c_long) as JDIMENSION,
                (*cinfo).max_v_samp_factor as JDIMENSION,
            );
        }
        ci += 1;
        compptr = compptr.add(1);
    }
}
