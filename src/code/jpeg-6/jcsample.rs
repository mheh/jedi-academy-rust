/*
 * jcsample.c
 *
 * Copyright (C) 1991-1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains downsampling routines.
 *
 * Downsampling input data is counted in "row groups".  A row group
 * is defined to be max_v_samp_factor pixel rows of each component,
 * from which the downsampler produces v_samp_factor sample rows.
 * A single row group is processed in each call to the downsampler module.
 *
 * The downsampler is responsible for edge-expansion of its output data
 * to fill an integral number of DCT blocks horizontally.  The source buffer
 * may be modified if it is helpful for this purpose (the source buffer is
 * allocated wide enough to correspond to the desired output width).
 * The caller (the prep controller) is responsible for vertical padding.
 *
 * The downsampler may request "context rows" by setting need_context_rows
 * during startup.  In this case, the input arrays will contain at least
 * one row group's worth of pixels above and below the passed-in data;
 * the caller will create dummy rows at image top and bottom by replicating
 * the first or last real pixel row.
 *
 * An excellent reference for image resampling is
 *   Digital Image Warping, George Wolberg, 1990.
 *   Pub. by IEEE Computer Society Press, Los Alamitos, CA. ISBN 0-8186-8944-7.
 *
 * The downsampling algorithm used here is a simple average of the source
 * pixels covered by the output pixel.  The hi-falutin sampling literature
 * refers to this as a "box filter".  In general the characteristics of a box
 * filter are not very good, but for the specific cases we normally use (1:1
 * and 2:1 ratios) the box is equivalent to a "triangle filter" which is not
 * nearly so bad.  If you intend to use other sampling ratios, you'd be well
 * advised to improve this code.
 *
 * A simple input-smoothing capability is provided.  This is mainly intended
 * for cleaning up color-dithered GIF input files (if you find it inadequate,
 * we suggest using an external filtering program such as pnmconvol).  When
 * enabled, each input pixel P is replaced by a weighted sum of itself and its
 * eight neighbors.  P's weight is 1-8*SF and each neighbor's weight is SF,
 * where SF = (smoothing_factor / 1024).
 * Currently, smoothing is only supported for 2h2v sampling factors.
 */

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_uint};

/* JPEG library type stubs and extern declarations */

pub type JSAMPLE = u8;
pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type JDIMENSION = c_uint;
pub type INT32 = i32;
pub type boolean = c_int;

const DCTSIZE: c_int = 8;
const MAX_COMPONENTS: usize = 10;
const JPOOL_IMAGE: c_int = 0;
const FALSE: boolean = 0;
const TRUE: boolean = 1;

/* Error codes and macros - stubs for external JPEG library */
const JERR_CCIR601_NOTIMPL: c_int = 0;
const JERR_FRACT_SAMPLE_NOTIMPL: c_int = 0;
const JTRC_SMOOTH_NOTIMPL: c_int = 0;

#[repr(C)]
pub struct jpeg_downsampler {
    pub start_pass: *mut c_int,
    pub downsample: *mut c_int,
    pub need_context_rows: boolean,
}

#[repr(C)]
pub struct jpeg_component_info {
    pub width_in_blocks: JDIMENSION,
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
}

#[repr(C)]
pub struct j_compress {
    pub downsample: *mut jpeg_downsampler,
    pub mem: *mut c_int,
    pub comp_info: *mut jpeg_component_info,
    pub num_components: c_int,
    pub max_h_samp_factor: c_int,
    pub max_v_samp_factor: c_int,
    pub image_width: JDIMENSION,
    pub CCIR601_sampling: boolean,
    pub smoothing_factor: c_int,
}

pub type j_compress_ptr = *mut j_compress;

/* Pointer to routine to downsample a single component */
pub type downsample1_ptr = extern "C" fn(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
);

/* Private subobject */

#[repr(C)]
pub struct my_downsampler {
    pub pub_: jpeg_downsampler, /* public fields */
    /* Downsampling method pointers, one per component */
    pub methods: [Option<downsample1_ptr>; MAX_COMPONENTS],
}

pub type my_downsample_ptr = *mut my_downsampler;

extern "C" {
    fn jcopy_sample_rows(
        input_data: JSAMPARRAY,
        source_row: c_int,
        output_data: JSAMPARRAY,
        dest_row: c_int,
        num_rows: c_int,
        num_cols: JDIMENSION,
    );
}

/*
 * Initialize for a downsampling pass.
 */

extern "C" fn start_pass_downsample(_cinfo: j_compress_ptr) {
    /* no work for now */
}

/*
 * Expand a component horizontally from width input_cols to width output_cols,
 * by duplicating the rightmost samples.
 */

extern "C" fn expand_right_edge(
    image_data: JSAMPARRAY,
    num_rows: c_int,
    input_cols: JDIMENSION,
    output_cols: JDIMENSION,
) {
    let mut ptr: JSAMPROW;
    let mut pixval: JSAMPLE;
    let mut count: c_int;
    let mut row: c_int;
    let numcols: c_int = (output_cols as c_int) - (input_cols as c_int);

    if numcols > 0 {
        row = 0;
        while row < num_rows {
            unsafe {
                ptr = (*image_data.add(row as usize)).add(input_cols as usize);
                pixval = *ptr.add(-1isize); /* don't need GETJSAMPLE() here */
                count = numcols;
                while count > 0 {
                    *ptr = pixval;
                    ptr = ptr.add(1);
                    count -= 1;
                }
            }
            row += 1;
        }
    }
}

/*
 * Do downsampling for a whole row group (all components).
 *
 * In this version we simply downsample each component independently.
 */

extern "C" fn sep_downsample(
    cinfo: j_compress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_index: JDIMENSION,
    output_buf: JSAMPIMAGE,
    out_row_group_index: JDIMENSION,
) {
    unsafe {
        let downsample: my_downsample_ptr = (*cinfo).downsample as my_downsample_ptr;
        let mut ci: c_int = 0;
        let mut compptr: *mut jpeg_component_info = (*cinfo).comp_info;
        let mut in_ptr: JSAMPARRAY;
        let mut out_ptr: JSAMPARRAY;

        while ci < (*cinfo).num_components {
            in_ptr = (*input_buf.add(ci as usize)).add(in_row_index as usize);
            out_ptr = (*output_buf.add(ci as usize))
                .add((out_row_group_index as c_int * (*compptr).v_samp_factor) as usize);
            if let Some(method) = (*downsample).methods[ci as usize] {
                method(cinfo, compptr, in_ptr, out_ptr);
            }
            ci += 1;
            compptr = compptr.add(1);
        }
    }
}

/*
 * Downsample pixel values of a single component.
 * One row group is processed per call.
 * This version handles arbitrary integral sampling ratios, without smoothing.
 * Note that this version is not actually used for customary sampling ratios.
 */

extern "C" fn int_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    unsafe {
        let mut inrow: c_int;
        let mut outrow: c_int;
        let h_expand: c_int = (*cinfo).max_h_samp_factor / (*compptr).h_samp_factor;
        let v_expand: c_int = (*cinfo).max_v_samp_factor / (*compptr).v_samp_factor;
        let numpix: c_int = h_expand * v_expand;
        let numpix2: c_int = numpix / 2;
        let output_cols: JDIMENSION =
            ((*compptr).width_in_blocks as c_int * DCTSIZE) as JDIMENSION;
        let mut outcol: JDIMENSION;
        let mut outcol_h: JDIMENSION; /* outcol_h == outcol*h_expand */
        let mut inptr: JSAMPROW;
        let mut outptr: JSAMPROW;
        let mut outvalue: INT32;

        /* Expand input data enough to let all the output samples be generated
         * by the standard loop.  Special-casing padded output would be more
         * efficient.
         */
        expand_right_edge(
            input_data,
            (*cinfo).max_v_samp_factor,
            (*cinfo).image_width,
            (output_cols as c_int * h_expand) as JDIMENSION,
        );

        inrow = 0;
        outrow = 0;
        while outrow < (*compptr).v_samp_factor {
            outptr = *input_data.add(outrow as usize);
            outcol = 0;
            outcol_h = 0;
            while outcol < output_cols {
                outvalue = 0;
                let mut v: c_int = 0;
                while v < v_expand {
                    inptr = (*input_data.add((inrow + v) as usize)).add(outcol_h as usize);
                    let mut h: c_int = 0;
                    while h < h_expand {
                        outvalue += *inptr as INT32;
                        inptr = inptr.add(1);
                        h += 1;
                    }
                    v += 1;
                }
                *outptr = (((outvalue + numpix2 as INT32) / numpix as INT32) as JSAMPLE);
                outptr = outptr.add(1);
                outcol += 1;
                outcol_h = (outcol_h as c_int + h_expand) as JDIMENSION;
            }
            inrow += v_expand;
            outrow += 1;
        }
    }
}

/*
 * Downsample pixel values of a single component.
 * This version handles the special case of a full-size component,
 * without smoothing.
 */

extern "C" fn fullsize_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    unsafe {
        /* Copy the data */
        jcopy_sample_rows(
            input_data,
            0,
            output_data,
            0,
            (*cinfo).max_v_samp_factor,
            (*cinfo).image_width,
        );
        /* Edge-expand */
        expand_right_edge(
            output_data,
            (*cinfo).max_v_samp_factor,
            (*cinfo).image_width,
            ((*compptr).width_in_blocks as c_int * DCTSIZE) as JDIMENSION,
        );
    }
}

/*
 * Downsample pixel values of a single component.
 * This version handles the common case of 2:1 horizontal and 1:1 vertical,
 * without smoothing.
 *
 * A note about the "bias" calculations: when rounding fractional values to
 * integer, we do not want to always round 0.5 up to the next integer.
 * If we did that, we'd introduce a noticeable bias towards larger values.
 * Instead, this code is arranged so that 0.5 will be rounded up or down at
 * alternate pixel locations (a simple ordered dither pattern).
 */

extern "C" fn h2v1_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    unsafe {
        let mut outrow: c_int;
        let mut outcol: JDIMENSION;
        let output_cols: JDIMENSION =
            ((*compptr).width_in_blocks as c_int * DCTSIZE) as JDIMENSION;
        let mut inptr: JSAMPROW;
        let mut outptr: JSAMPROW;
        let mut bias: c_int;

        /* Expand input data enough to let all the output samples be generated
         * by the standard loop.  Special-casing padded output would be more
         * efficient.
         */
        expand_right_edge(
            input_data,
            (*cinfo).max_v_samp_factor,
            (*cinfo).image_width,
            (output_cols as c_int * 2) as JDIMENSION,
        );

        outrow = 0;
        while outrow < (*compptr).v_samp_factor {
            outptr = *input_data.add(outrow as usize);
            inptr = *input_data.add(outrow as usize);
            bias = 0; /* bias = 0,1,0,1,... for successive samples */
            outcol = 0;
            while outcol < output_cols {
                *outptr =
                    ((((*inptr as c_int) + (*inptr.add(1) as c_int) + bias) >> 1) as JSAMPLE);
                bias ^= 1; /* 0=>1, 1=>0 */
                inptr = inptr.add(2);
                outptr = outptr.add(1);
                outcol += 1;
            }
            outrow += 1;
        }
    }
}

/*
 * Downsample pixel values of a single component.
 * This version handles the standard case of 2:1 horizontal and 2:1 vertical,
 * without smoothing.
 */

extern "C" fn h2v2_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    unsafe {
        let mut inrow: c_int;
        let mut outrow: c_int;
        let mut outcol: JDIMENSION;
        let output_cols: JDIMENSION =
            ((*compptr).width_in_blocks as c_int * DCTSIZE) as JDIMENSION;
        let mut inptr0: JSAMPROW;
        let mut inptr1: JSAMPROW;
        let mut outptr: JSAMPROW;
        let mut bias: c_int;

        /* Expand input data enough to let all the output samples be generated
         * by the standard loop.  Special-casing padded output would be more
         * efficient.
         */
        expand_right_edge(
            input_data,
            (*cinfo).max_v_samp_factor,
            (*cinfo).image_width,
            (output_cols as c_int * 2) as JDIMENSION,
        );

        inrow = 0;
        outrow = 0;
        while outrow < (*compptr).v_samp_factor {
            outptr = *input_data.add(outrow as usize);
            inptr0 = *input_data.add(inrow as usize);
            inptr1 = *input_data.add((inrow + 1) as usize);
            bias = 1; /* bias = 1,2,1,2,... for successive samples */
            outcol = 0;
            while outcol < output_cols {
                *outptr = (((*inptr0 as c_int)
                    + (*inptr0.add(1) as c_int)
                    + (*inptr1 as c_int)
                    + (*inptr1.add(1) as c_int)
                    + bias)
                    >> 2) as JSAMPLE;
                bias ^= 3; /* 1=>2, 2=>1 */
                inptr0 = inptr0.add(2);
                inptr1 = inptr1.add(2);
                outptr = outptr.add(1);
                outcol += 1;
            }
            inrow += 2;
            outrow += 1;
        }
    }
}

#[cfg(feature = "input_smoothing_supported")]
extern "C" fn h2v2_smooth_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    /*
     * Downsample pixel values of a single component.
     * This version handles the standard case of 2:1 horizontal and 2:1 vertical,
     * with smoothing.  One row of context is required.
     */

    unsafe {
        let mut inrow: c_int;
        let mut outrow: c_int;
        let mut colctr: JDIMENSION;
        let output_cols: JDIMENSION =
            ((*compptr).width_in_blocks as c_int * DCTSIZE) as JDIMENSION;
        let mut inptr0: JSAMPROW;
        let mut inptr1: JSAMPROW;
        let mut above_ptr: JSAMPROW;
        let mut below_ptr: JSAMPROW;
        let mut outptr: JSAMPROW;
        let mut membersum: INT32;
        let mut neighsum: INT32;
        let memberscale: INT32;
        let neighscale: INT32;

        /* Expand input data enough to let all the output samples be generated
         * by the standard loop.  Special-casing padded output would be more
         * efficient.
         */
        expand_right_edge(
            input_data.add(-1isize),
            (*cinfo).max_v_samp_factor + 2,
            (*cinfo).image_width,
            (output_cols as c_int * 2) as JDIMENSION,
        );

        /* We don't bother to form the individual "smoothed" input pixel values;
         * we can directly compute the output which is the average of the four
         * smoothed values.  Each of the four member pixels contributes a fraction
         * (1-8*SF) to its own smoothed image and a fraction SF to each of the three
         * other smoothed pixels, therefore a total fraction (1-5*SF)/4 to the final
         * output.  The four corner-adjacent neighbor pixels contribute a fraction
         * SF to just one smoothed pixel, or SF/4 to the final output; while the
         * eight edge-adjacent neighbors contribute SF to each of two smoothed
         * pixels, or SF/2 overall.  In order to use integer arithmetic, these
         * factors are scaled by 2^16 = 65536.
         * Also recall that SF = smoothing_factor / 1024.
         */

        memberscale = 16384 - (*cinfo).smoothing_factor * 80; /* scaled (1-5*SF)/4 */
        neighscale = (*cinfo).smoothing_factor * 16; /* scaled SF/4 */

        inrow = 0;
        outrow = 0;
        while outrow < (*compptr).v_samp_factor {
            outptr = *output_data.add(outrow as usize);
            inptr0 = *input_data.add(inrow as usize);
            inptr1 = *input_data.add((inrow + 1) as usize);
            above_ptr = *input_data.add((inrow - 1) as usize);
            below_ptr = *input_data.add((inrow + 2) as usize);

            /* Special case for first column: pretend column -1 is same as column 0 */
            membersum = (*inptr0 as INT32)
                + (*inptr0.add(1) as INT32)
                + (*inptr1 as INT32)
                + (*inptr1.add(1) as INT32);
            neighsum = (*above_ptr as INT32)
                + (*above_ptr.add(1) as INT32)
                + (*below_ptr as INT32)
                + (*below_ptr.add(1) as INT32)
                + (*inptr0 as INT32)
                + (*inptr0.add(2) as INT32)
                + (*inptr1 as INT32)
                + (*inptr1.add(2) as INT32);
            neighsum += neighsum;
            neighsum += (*above_ptr as INT32)
                + (*above_ptr.add(2) as INT32)
                + (*below_ptr as INT32)
                + (*below_ptr.add(2) as INT32);
            membersum = membersum * memberscale + neighsum * neighscale;
            *outptr = (((membersum + 32768) >> 16) as JSAMPLE);
            inptr0 = inptr0.add(2);
            inptr1 = inptr1.add(2);
            above_ptr = above_ptr.add(2);
            below_ptr = below_ptr.add(2);
            outptr = outptr.add(1);

            colctr = output_cols - 2;
            while colctr > 0 {
                /* sum of pixels directly mapped to this output element */
                membersum = (*inptr0 as INT32)
                    + (*inptr0.add(1) as INT32)
                    + (*inptr1 as INT32)
                    + (*inptr1.add(1) as INT32);
                /* sum of edge-neighbor pixels */
                neighsum = (*above_ptr as INT32)
                    + (*above_ptr.add(1) as INT32)
                    + (*below_ptr as INT32)
                    + (*below_ptr.add(1) as INT32)
                    + (*inptr0.add(-1isize) as INT32)
                    + (*inptr0.add(2) as INT32)
                    + (*inptr1.add(-1isize) as INT32)
                    + (*inptr1.add(2) as INT32);
                /* The edge-neighbors count twice as much as corner-neighbors */
                neighsum += neighsum;
                /* Add in the corner-neighbors */
                neighsum += (*above_ptr.add(-1isize) as INT32)
                    + (*above_ptr.add(2) as INT32)
                    + (*below_ptr.add(-1isize) as INT32)
                    + (*below_ptr.add(2) as INT32);
                /* form final output scaled up by 2^16 */
                membersum = membersum * memberscale + neighsum * neighscale;
                /* round, descale and output it */
                *outptr = (((membersum + 32768) >> 16) as JSAMPLE);
                inptr0 = inptr0.add(2);
                inptr1 = inptr1.add(2);
                above_ptr = above_ptr.add(2);
                below_ptr = below_ptr.add(2);
                outptr = outptr.add(1);
                colctr -= 1;
            }

            /* Special case for last column */
            membersum = (*inptr0 as INT32)
                + (*inptr0.add(1) as INT32)
                + (*inptr1 as INT32)
                + (*inptr1.add(1) as INT32);
            neighsum = (*above_ptr as INT32)
                + (*above_ptr.add(1) as INT32)
                + (*below_ptr as INT32)
                + (*below_ptr.add(1) as INT32)
                + (*inptr0.add(-1isize) as INT32)
                + (*inptr0.add(1) as INT32)
                + (*inptr1.add(-1isize) as INT32)
                + (*inptr1.add(1) as INT32);
            neighsum += neighsum;
            neighsum += (*above_ptr.add(-1isize) as INT32)
                + (*above_ptr.add(1) as INT32)
                + (*below_ptr.add(-1isize) as INT32)
                + (*below_ptr.add(1) as INT32);
            membersum = membersum * memberscale + neighsum * neighscale;
            *outptr = (((membersum + 32768) >> 16) as JSAMPLE);

            inrow += 2;
            outrow += 1;
        }
    }
}

#[cfg(feature = "input_smoothing_supported")]
extern "C" fn fullsize_smooth_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    /*
     * Downsample pixel values of a single component.
     * This version handles the special case of a full-size component,
     * with smoothing.  One row of context is required.
     */

    unsafe {
        let mut outrow: c_int;
        let mut colctr: JDIMENSION;
        let output_cols: JDIMENSION =
            ((*compptr).width_in_blocks as c_int * DCTSIZE) as JDIMENSION;
        let mut inptr: JSAMPROW;
        let mut above_ptr: JSAMPROW;
        let mut below_ptr: JSAMPROW;
        let mut outptr: JSAMPROW;
        let mut membersum: INT32;
        let mut neighsum: INT32;
        let mut colsum: c_int;
        let mut lastcolsum: c_int;
        let mut nextcolsum: c_int;
        let memberscale: INT32;
        let neighscale: INT32;

        /* Expand input data enough to let all the output samples be generated
         * by the standard loop.  Special-casing padded output would be more
         * efficient.
         */
        expand_right_edge(
            input_data.add(-1isize),
            (*cinfo).max_v_samp_factor + 2,
            (*cinfo).image_width,
            output_cols,
        );

        /* Each of the eight neighbor pixels contributes a fraction SF to the
         * smoothed pixel, while the main pixel contributes (1-8*SF).  In order
         * to use integer arithmetic, these factors are multiplied by 2^16 = 65536.
         * Also recall that SF = smoothing_factor / 1024.
         */

        memberscale = 65536 - (*cinfo).smoothing_factor * 512; /* scaled 1-8*SF */
        neighscale = (*cinfo).smoothing_factor * 64; /* scaled SF */

        outrow = 0;
        while outrow < (*compptr).v_samp_factor {
            outptr = *output_data.add(outrow as usize);
            inptr = *input_data.add(outrow as usize);
            above_ptr = *input_data.add((outrow - 1) as usize);
            below_ptr = *input_data.add((outrow + 1) as usize);

            /* Special case for first column */
            colsum = (*above_ptr as c_int) + (*below_ptr as c_int) + (*inptr as c_int);
            above_ptr = above_ptr.add(1);
            below_ptr = below_ptr.add(1);
            membersum = *inptr as INT32;
            inptr = inptr.add(1);
            nextcolsum = (*above_ptr as c_int) + (*below_ptr as c_int) + (*inptr as c_int);
            neighsum = (colsum + (colsum - membersum as c_int) + nextcolsum) as INT32;
            membersum = membersum * memberscale + neighsum * neighscale;
            *outptr = (((membersum + 32768) >> 16) as JSAMPLE);
            outptr = outptr.add(1);
            lastcolsum = colsum;
            colsum = nextcolsum;

            colctr = output_cols - 2;
            while colctr > 0 {
                membersum = *inptr as INT32;
                inptr = inptr.add(1);
                above_ptr = above_ptr.add(1);
                below_ptr = below_ptr.add(1);
                nextcolsum = (*above_ptr as c_int) + (*below_ptr as c_int) + (*inptr as c_int);
                neighsum = (lastcolsum + (colsum - membersum as c_int) + nextcolsum) as INT32;
                membersum = membersum * memberscale + neighsum * neighscale;
                *outptr = (((membersum + 32768) >> 16) as JSAMPLE);
                outptr = outptr.add(1);
                lastcolsum = colsum;
                colsum = nextcolsum;
                colctr -= 1;
            }

            /* Special case for last column */
            membersum = *inptr as INT32;
            neighsum = (lastcolsum + (colsum - membersum as c_int) + colsum) as INT32;
            membersum = membersum * memberscale + neighsum * neighscale;
            *outptr = (((membersum + 32768) >> 16) as JSAMPLE);

            outrow += 1;
        }
    }
}

/*
 * Module initialization routine for downsampling.
 * Note that we must select a routine for each component.
 */

pub extern "C" fn jinit_downsampler(cinfo: j_compress_ptr) {
    unsafe {
        let mut downsample: my_downsample_ptr;
        let mut ci: c_int;
        let mut compptr: *mut jpeg_component_info;
        let mut smoothok: boolean = TRUE;

        downsample =
            (*(*cinfo).mem).alloc_small(cinfo as *mut c_int, JPOOL_IMAGE, 0) as my_downsample_ptr;
        (*cinfo).downsample = &mut (*downsample).pub_ as *mut jpeg_downsampler;
        (*(*downsample).pub_).start_pass = &start_pass_downsample as *mut c_int;
        (*(*downsample).pub_).downsample = &sep_downsample as *mut c_int;
        (*(*downsample).pub_).need_context_rows = FALSE;

        if (*cinfo).CCIR601_sampling != FALSE {
            // ERREXIT(cinfo, JERR_CCIR601_NOTIMPL);
        }

        /* Verify we can handle the sampling factors, and set up method pointers */
        ci = 0;
        compptr = (*cinfo).comp_info;
        while ci < (*cinfo).num_components {
            if (*compptr).h_samp_factor == (*cinfo).max_h_samp_factor
                && (*compptr).v_samp_factor == (*cinfo).max_v_samp_factor
            {
                #[cfg(feature = "input_smoothing_supported")]
                {
                    if (*cinfo).smoothing_factor != 0 {
                        (*downsample).methods[ci as usize] = Some(fullsize_smooth_downsample);
                        (*(*downsample).pub_).need_context_rows = TRUE;
                    } else {
                        (*downsample).methods[ci as usize] = Some(fullsize_downsample);
                    }
                }
                #[cfg(not(feature = "input_smoothing_supported"))]
                {
                    (*downsample).methods[ci as usize] = Some(fullsize_downsample);
                }
            } else if (*compptr).h_samp_factor * 2 == (*cinfo).max_h_samp_factor
                && (*compptr).v_samp_factor == (*cinfo).max_v_samp_factor
            {
                smoothok = FALSE;
                (*downsample).methods[ci as usize] = Some(h2v1_downsample);
            } else if (*compptr).h_samp_factor * 2 == (*cinfo).max_h_samp_factor
                && (*compptr).v_samp_factor * 2 == (*cinfo).max_v_samp_factor
            {
                #[cfg(feature = "input_smoothing_supported")]
                {
                    if (*cinfo).smoothing_factor != 0 {
                        (*downsample).methods[ci as usize] = Some(h2v2_smooth_downsample);
                        (*(*downsample).pub_).need_context_rows = TRUE;
                    } else {
                        (*downsample).methods[ci as usize] = Some(h2v2_downsample);
                    }
                }
                #[cfg(not(feature = "input_smoothing_supported"))]
                {
                    (*downsample).methods[ci as usize] = Some(h2v2_downsample);
                }
            } else if ((*cinfo).max_h_samp_factor % (*compptr).h_samp_factor) == 0
                && ((*cinfo).max_v_samp_factor % (*compptr).v_samp_factor) == 0
            {
                smoothok = FALSE;
                (*downsample).methods[ci as usize] = Some(int_downsample);
            } else {
                // ERREXIT(cinfo, JERR_FRACT_SAMPLE_NOTIMPL);
            }
            ci += 1;
            compptr = compptr.add(1);
        }

        #[cfg(feature = "input_smoothing_supported")]
        {
            if (*cinfo).smoothing_factor != 0 && smoothok == FALSE {
                // TRACEMS(cinfo, 0, JTRC_SMOOTH_NOTIMPL);
            }
        }
    }
}
