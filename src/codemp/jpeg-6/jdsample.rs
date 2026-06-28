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
// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #define JPEG_INTERNALS
// #include "jinclude.h"
// #include "jpeglib.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_void};

/* ============================================================================
 * Stubs for JPEG-6 types and structures needed for structural coherence
 * ============================================================================ */

pub type JSAMPLE = u8;
pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type JDIMENSION = c_uint;
pub type UINT8 = u8;
pub type boolean = u8;

const TRUE: boolean = 1;
const FALSE: boolean = 0;

pub const MAX_COMPONENTS: c_int = 10;

const JERR_CCIR601_NOTIMPL: c_int = 37;
const JERR_FRACT_SAMPLE_NOTIMPL: c_int = 38;

#[repr(C)]
pub struct jpeg_component_info {
    pub component_index: c_int,
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
    pub DCT_scaled_size: c_int,
    pub downsampled_width: JDIMENSION,
    pub component_needed: boolean,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_upsampler {
    pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    pub upsample: Option<unsafe extern "C" fn(j_decompress_ptr, JSAMPIMAGE, *mut JDIMENSION,
                                               JDIMENSION, JSAMPARRAY, *mut JDIMENSION, JDIMENSION)>,
    pub need_context_rows: boolean,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, c_int) -> *mut c_void>,
    pub alloc_sarray: Option<unsafe extern "C" fn(*mut c_void, c_int, JDIMENSION, JDIMENSION) -> JSAMPARRAY>,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_error_mgr {
    pub msg_code: c_int,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_color_converter {
    pub color_convert: Option<unsafe extern "C" fn(
        j_decompress_ptr,
        JSAMPARRAY,
        JDIMENSION,
        JSAMPARRAY,
        c_int,
    )>,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct j_decompress_struct {
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub upsample: *mut jpeg_upsampler,
    pub max_v_samp_factor: c_int,
    pub max_h_samp_factor: c_int,
    pub output_width: JDIMENSION,
    pub output_height: JDIMENSION,
    pub num_components: c_int,
    pub comp_info: *mut jpeg_component_info,
    pub CCIR601_sampling: boolean,
    pub do_fancy_upsampling: boolean,
    pub min_DCT_scaled_size: c_int,
    pub cconvert: *mut jpeg_color_converter,
    _opaque: [u8; 0],
}

pub type j_decompress_ptr = *mut j_decompress_struct;
pub type j_common_ptr = *mut c_void;

pub const JPOOL_IMAGE: c_int = 1;

/* Pointer to routine to upsample a single component */
pub type upsample1_ptr = unsafe extern "C" fn(
    j_decompress_ptr,
    *mut jpeg_component_info,
    JSAMPARRAY,
    *mut JSAMPARRAY,
);

/* Private subobject */

#[repr(C)]
pub struct my_upsampler {
    pub pub_: jpeg_upsampler,	/* public fields */

    /* Color conversion buffer.  When using separate upsampling and color
     * conversion steps, this buffer holds one upsampled row group until it
     * has been color converted and output.
     * Note: we do not allocate any storage for component(s) which are full-size,
     * ie do not need rescaling.  The corresponding entry of color_buf[] is
     * simply set to point to the input data array, thereby avoiding copying.
     */
    pub color_buf: [JSAMPARRAY; MAX_COMPONENTS as usize],

    /* Per-component upsampling method pointers */
    pub methods: [upsample1_ptr; MAX_COMPONENTS as usize],

    pub next_row_out: c_int,		/* counts rows emitted from color_buf */
    pub rows_to_go: JDIMENSION,	/* counts rows remaining in image */

    /* Height of an input row group for each component. */
    pub rowgroup_height: [c_int; MAX_COMPONENTS as usize],

    /* These arrays save pixel expansion factors so that int_expand need not
     * recompute them each time.  They are unused for other upsampling methods.
     */
    pub h_expand: [UINT8; MAX_COMPONENTS as usize],
    pub v_expand: [UINT8; MAX_COMPONENTS as usize],
}

pub type my_upsample_ptr = *mut my_upsampler;

/* External function declarations */
extern "C" {
    pub fn jcopy_sample_rows(input_array: JSAMPARRAY, source_row: c_int,
                             output_array: JSAMPARRAY, dest_row: c_int,
                             num_rows: c_int, num_cols: JDIMENSION);
}

/* Macro: ERREXIT(cinfo, code) */
#[inline]
unsafe fn ERREXIT(cinfo: j_decompress_ptr, code: c_int) {
    (*(*cinfo).err).msg_code = code;
}

/* Macro: SIZEOF - size in bytes for JPEG_INTERNALS mode */
#[inline]
const fn SIZEOF<T>() -> c_int {
    std::mem::size_of::<T>() as c_int
}

/* Macro: GETJSAMPLE - convert JSAMPLE (u8) to signed int for arithmetic */
#[inline]
fn GETJSAMPLE(val: JSAMPLE) -> c_int {
    val as c_int
}

/*
 * Initialize for an upsampling pass.
 */

pub unsafe fn start_pass_upsample(cinfo: j_decompress_ptr) {
    let upsample = (*cinfo).upsample as *mut my_upsampler;

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

pub unsafe fn sep_upsample(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_group_ctr: *mut JDIMENSION,
    _in_row_groups_avail: JDIMENSION,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    let upsample = (*cinfo).upsample as *mut my_upsampler;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut num_rows: JDIMENSION;
    let mut out_rows_avail_mut = out_rows_avail;

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
                    .add(((*in_row_group_ctr as c_int) * (*upsample).rowgroup_height[ci as usize]) as usize),
                &mut (*upsample).color_buf[ci as usize],
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
    out_rows_avail_mut = out_rows_avail_mut.wrapping_sub(*out_row_ctr);
    if num_rows > out_rows_avail_mut {
        num_rows = out_rows_avail_mut;
    }

    /* Call color conversion through cinfo->cconvert->color_convert */
    /* cconvert is a struct pointer with a color_convert function pointer field */
    let cconvert = (*cinfo).cconvert as *mut jpeg_color_converter;
    if !cconvert.is_null() {
        if let Some(color_convert_fn) = (*cconvert).color_convert {
            color_convert_fn(
                cinfo,
                (*upsample).color_buf.as_mut_ptr(),
                (*upsample).next_row_out as JDIMENSION,
                output_buf.add(*out_row_ctr as usize),
                num_rows as c_int,
            );
        }
    }

    /* Adjust counts */
    *out_row_ctr = (*out_row_ctr).wrapping_add(num_rows);
    (*upsample).rows_to_go = (*upsample).rows_to_go.wrapping_sub(num_rows);
    (*upsample).next_row_out = (*upsample).next_row_out.wrapping_add(num_rows as c_int);
    /* When the buffer is emptied, declare this input row group consumed */
    if (*upsample).next_row_out >= (*cinfo).max_v_samp_factor {
        *in_row_group_ctr = (*in_row_group_ctr).wrapping_add(1);
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

pub unsafe fn fullsize_upsample(
    _cinfo: j_decompress_ptr,
    _compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    *output_data_ptr = input_data;
}


/*
 * This is a no-op version used for "uninteresting" components.
 * These components will not be referenced by color conversion.
 */

pub unsafe fn noop_upsample(
    _cinfo: j_decompress_ptr,
    _compptr: *mut jpeg_component_info,
    _input_data: JSAMPARRAY,
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

pub unsafe fn int_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let upsample = cinfo as *mut my_upsampler as my_upsample_ptr;
    let output_data = *output_data_ptr;
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

pub unsafe fn h2v1_upsample(
    cinfo: j_decompress_ptr,
    _compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let output_data = *output_data_ptr;
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

pub unsafe fn h2v2_upsample(
    cinfo: j_decompress_ptr,
    _compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let output_data = *output_data_ptr;
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
        jcopy_sample_rows(output_data, outrow, output_data, outrow + 1, 1, (*cinfo).output_width);
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

pub unsafe fn h2v1_fancy_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let output_data = *output_data_ptr;
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

        colctr = ((*compptr).downsampled_width as c_int - 2) as JDIMENSION;
        while colctr > 0 {
            /* General case: 3/4 * nearer pixel + 1/4 * further pixel */
            invalue = GETJSAMPLE(*inptr) * 3;
            inptr = inptr.add(1);
            *outptr = (((invalue + GETJSAMPLE(*inptr.offset(-2)) + 1) >> 2) as JSAMPLE);
            outptr = outptr.add(1);
            *outptr = (((invalue + GETJSAMPLE(*inptr) + 2) >> 2) as JSAMPLE);
            outptr = outptr.add(1);
            colctr -= 1;
        }

        /* Special case for last column */
        invalue = GETJSAMPLE(*inptr);
        *outptr = (((invalue * 3 + GETJSAMPLE(*inptr.offset(-1)) + 1) >> 2) as JSAMPLE);
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

pub unsafe fn h2v2_fancy_upsample(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
) {
    let output_data = *output_data_ptr;
    let mut inptr0: JSAMPROW;
    let mut inptr1: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut thiscolsum: c_int;
    let mut lastcolsum: c_int;
    let mut nextcolsum: c_int;
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
            inptr0 = *input_data.add(inrow as usize);
            if v == 0 {		/* next nearest is row above */
                inptr1 = *input_data.add((inrow - 1) as usize);
            } else {			/* next nearest is row below */
                inptr1 = *input_data.add((inrow + 1) as usize);
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
            lastcolsum = thiscolsum;
            thiscolsum = nextcolsum;

            colctr = ((*compptr).downsampled_width as c_int - 2) as JDIMENSION;
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
                lastcolsum = thiscolsum;
                thiscolsum = nextcolsum;
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

    upsample = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        SIZEOF::<my_upsampler>(),
    ) as my_upsample_ptr;
    (*cinfo).upsample = &mut (*upsample).pub_ as *mut jpeg_upsampler;
    (*upsample).pub_.start_pass = Some(start_pass_upsample);
    (*upsample).pub_.upsample = Some(sep_upsample);
    (*upsample).pub_.need_context_rows = FALSE; /* until we find out differently */

    if (*cinfo).CCIR601_sampling != FALSE {	/* this isn't supported */
        ERREXIT(cinfo, JERR_CCIR601_NOTIMPL);
    }

    /* jdmainct.c doesn't support context rows when min_DCT_scaled_size = 1,
     * so don't ask for it.
     */
    do_fancy = ((*cinfo).do_fancy_upsampling != FALSE) && ((*cinfo).min_DCT_scaled_size > 1);

    /* Verify we can handle the sampling factors, select per-component methods,
     * and create storage as needed.
     */
    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Compute size of an "input group" after IDCT scaling.  This many samples
         * are to be converted to max_h_samp_factor * max_v_samp_factor pixels.
         */
        h_in_group = ((*compptr).h_samp_factor * (*compptr).DCT_scaled_size) / (*cinfo).min_DCT_scaled_size;
        v_in_group = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size) / (*cinfo).min_DCT_scaled_size;
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
        } else if h_in_group * 2 == h_out_group && v_in_group == v_out_group {
            /* Special cases for 2h1v upsampling */
            if do_fancy != FALSE && (*compptr).downsampled_width > 2 {
                (*upsample).methods[ci as usize] = h2v1_fancy_upsample;
            } else {
                (*upsample).methods[ci as usize] = h2v1_upsample;
            }
        } else if h_in_group * 2 == h_out_group && v_in_group * 2 == v_out_group {
            /* Special cases for 2h2v upsampling */
            if do_fancy != FALSE && (*compptr).downsampled_width > 2 {
                (*upsample).methods[ci as usize] = h2v2_fancy_upsample;
                (*upsample).pub_.need_context_rows = TRUE;
            } else {
                (*upsample).methods[ci as usize] = h2v2_upsample;
            }
        } else if (h_out_group % h_in_group) == 0 && (v_out_group % v_in_group) == 0 {
            /* Generic integral-factors upsampling method */
            (*upsample).methods[ci as usize] = int_upsample;
            (*upsample).h_expand[ci as usize] = (h_out_group / h_in_group) as UINT8;
            (*upsample).v_expand[ci as usize] = (v_out_group / v_in_group) as UINT8;
        } else {
            ERREXIT(cinfo, JERR_FRACT_SAMPLE_NOTIMPL);
        }
        if need_buffer != FALSE {
            (*upsample).color_buf[ci as usize] = ((*(*cinfo).mem).alloc_sarray.unwrap())(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                jround_up((*cinfo).output_width as c_int, (*cinfo).max_h_samp_factor) as JDIMENSION,
                (*cinfo).max_v_samp_factor as JDIMENSION,
            );
        }
        ci += 1;
        compptr = compptr.add(1);
    }
}

/* Utility function to round up to nearest multiple */
#[inline]
fn jround_up(a: c_int, b: c_int) -> c_int {
    ((a + b - 1) / b) * b
}
