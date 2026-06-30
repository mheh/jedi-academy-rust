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
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals,
         unused_variables, dead_code, unused_mut, unused_assignments,
         clippy::all)]

//Anything above this #include will be ignored by the compiler
/* #define JPEG_INTERNALS — triggers jpegint.h inclusion via jpeglib.h */
use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::jpeg_6::jinclude_h::*;
use crate::codemp::jpeg_6::jpeglib_h::*;
use crate::codemp::jpeg_6::jpegint_h::*;

/* Pointer to routine to downsample a single component */
type downsample1_ptr = Option<
    unsafe fn(
        cinfo: j_compress_ptr,
        compptr: *mut jpeg_component_info,
        input_data: JSAMPARRAY,
        output_data: JSAMPARRAY,
    ),
>;

/* Private subobject */

#[repr(C)]
struct my_downsampler {
    pub_: jpeg_downsampler,	/* public fields */

    /* Downsampling method pointers, one per component */
    methods: [downsample1_ptr; MAX_COMPONENTS as usize],
}

type my_downsample_ptr = *mut my_downsampler;


/*
 * Initialize for a downsampling pass.
 */

unsafe fn start_pass_downsample(cinfo: j_compress_ptr) {
    /* no work for now */
}


/*
 * Expand a component horizontally from width input_cols to width output_cols,
 * by duplicating the rightmost samples.
 */

unsafe fn expand_right_edge(
    image_data: JSAMPARRAY,
    num_rows: core::ffi::c_int,
    input_cols: JDIMENSION,
    output_cols: JDIMENSION,
) {
    let mut ptr: JSAMPROW;
    let mut pixval: JSAMPLE;
    let mut count: core::ffi::c_int;
    let mut row: core::ffi::c_int;
    let numcols: core::ffi::c_int =
        output_cols.wrapping_sub(input_cols) as core::ffi::c_int;

    if numcols > 0 {
        row = 0;
        while row < num_rows {
            ptr = (*image_data.add(row as usize)).add(input_cols as usize);
            pixval = *ptr.offset(-1);		/* don't need GETJSAMPLE() here */
            count = numcols;
            while count > 0 {
                *ptr = pixval;
                ptr = ptr.add(1);
                count -= 1;
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

unsafe fn sep_downsample(
    cinfo: j_compress_ptr,
    input_buf: JSAMPIMAGE,
    in_row_index: JDIMENSION,
    output_buf: JSAMPIMAGE,
    out_row_group_index: JDIMENSION,
) {
    let downsample = (*cinfo).downsample as my_downsample_ptr;
    let mut ci: core::ffi::c_int;
    let mut compptr: *mut jpeg_component_info;
    let in_ptr: JSAMPARRAY;
    let out_ptr: JSAMPARRAY;

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        let in_ptr = (*input_buf.add(ci as usize)).add(in_row_index as usize);
        let out_ptr = (*output_buf.add(ci as usize))
            .add(out_row_group_index as usize * (*compptr).v_samp_factor as usize);
        (*downsample).methods[ci as usize].unwrap()(cinfo, compptr, in_ptr, out_ptr);
        ci += 1;
        compptr = compptr.add(1);
    }
}


/*
 * Downsample pixel values of a single component.
 * One row group is processed per call.
 * This version handles arbitrary integral sampling ratios, without smoothing.
 * Note that this version is not actually used for customary sampling ratios.
 */

unsafe fn int_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    let mut inrow: core::ffi::c_int;
    let mut outrow: core::ffi::c_int;
    let mut h_expand: core::ffi::c_int;
    let mut v_expand: core::ffi::c_int;
    let mut numpix: core::ffi::c_int;
    let mut numpix2: core::ffi::c_int;
    let mut h: core::ffi::c_int;
    let mut v: core::ffi::c_int;
    let mut outcol: JDIMENSION;
    let mut outcol_h: JDIMENSION;	/* outcol_h == outcol*h_expand */
    let output_cols: JDIMENSION = (*compptr).width_in_blocks * DCTSIZE as JDIMENSION;
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut outvalue: INT32;

    h_expand = (*cinfo).max_h_samp_factor / (*compptr).h_samp_factor;
    v_expand = (*cinfo).max_v_samp_factor / (*compptr).v_samp_factor;
    numpix = h_expand * v_expand;
    numpix2 = numpix / 2;

    /* Expand input data enough to let all the output samples be generated
     * by the standard loop.  Special-casing padded output would be more
     * efficient.
     */
    expand_right_edge(
        input_data,
        (*cinfo).max_v_samp_factor,
        (*cinfo).image_width,
        output_cols * h_expand as JDIMENSION,
    );

    inrow = 0;
    outrow = 0;
    while outrow < (*compptr).v_samp_factor {
        outptr = *output_data.add(outrow as usize);
        outcol = 0;
        outcol_h = 0;
        while outcol < output_cols {
            outvalue = 0;
            v = 0;
            while v < v_expand {
                inptr = (*input_data.add((inrow + v) as usize)).add(outcol_h as usize);
                h = 0;
                while h < h_expand {
                    outvalue += GETJSAMPLE(*inptr) as INT32;
                    inptr = inptr.add(1);
                    h += 1;
                }
                v += 1;
            }
            *outptr = ((outvalue + numpix2 as INT32) / numpix as INT32) as JSAMPLE;
            outptr = outptr.add(1);
            outcol += 1;
            outcol_h += h_expand as JDIMENSION;
        }
        inrow += v_expand;
        outrow += 1;
    }
}


/*
 * Downsample pixel values of a single component.
 * This version handles the special case of a full-size component,
 * without smoothing.
 */

unsafe fn fullsize_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
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
        (*compptr).width_in_blocks * DCTSIZE as JDIMENSION,
    );
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

unsafe fn h2v1_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    let mut outrow: core::ffi::c_int;
    let mut outcol: JDIMENSION;
    let output_cols: JDIMENSION = (*compptr).width_in_blocks * DCTSIZE as JDIMENSION;
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut bias: core::ffi::c_int;

    /* Expand input data enough to let all the output samples be generated
     * by the standard loop.  Special-casing padded output would be more
     * efficient.
     */
    expand_right_edge(
        input_data,
        (*cinfo).max_v_samp_factor,
        (*cinfo).image_width,
        output_cols * 2,
    );

    outrow = 0;
    while outrow < (*compptr).v_samp_factor {
        outptr = *output_data.add(outrow as usize);
        inptr = *input_data.add(outrow as usize);
        bias = 0;			/* bias = 0,1,0,1,... for successive samples */
        outcol = 0;
        while outcol < output_cols {
            *outptr = ((GETJSAMPLE(*inptr) + GETJSAMPLE(*inptr.add(1))
                        + bias) >> 1) as JSAMPLE;
            outptr = outptr.add(1);
            bias ^= 1;		/* 0=>1, 1=>0 */
            inptr = inptr.add(2);
            outcol += 1;
        }
        outrow += 1;
    }
}


/*
 * Downsample pixel values of a single component.
 * This version handles the standard case of 2:1 horizontal and 2:1 vertical,
 * without smoothing.
 */

unsafe fn h2v2_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    let mut inrow: core::ffi::c_int;
    let mut outrow: core::ffi::c_int;
    let mut outcol: JDIMENSION;
    let output_cols: JDIMENSION = (*compptr).width_in_blocks * DCTSIZE as JDIMENSION;
    let mut inptr0: JSAMPROW;
    let mut inptr1: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut bias: core::ffi::c_int;

    /* Expand input data enough to let all the output samples be generated
     * by the standard loop.  Special-casing padded output would be more
     * efficient.
     */
    expand_right_edge(
        input_data,
        (*cinfo).max_v_samp_factor,
        (*cinfo).image_width,
        output_cols * 2,
    );

    inrow = 0;
    outrow = 0;
    while outrow < (*compptr).v_samp_factor {
        outptr = *output_data.add(outrow as usize);
        inptr0 = *input_data.add(inrow as usize);
        inptr1 = *input_data.add((inrow + 1) as usize);
        bias = 1;			/* bias = 1,2,1,2,... for successive samples */
        outcol = 0;
        while outcol < output_cols {
            *outptr = ((GETJSAMPLE(*inptr0) + GETJSAMPLE(*inptr0.add(1)) +
                        GETJSAMPLE(*inptr1) + GETJSAMPLE(*inptr1.add(1))
                        + bias) >> 2) as JSAMPLE;
            outptr = outptr.add(1);
            bias ^= 3;		/* 1=>2, 2=>1 */
            inptr0 = inptr0.add(2);
            inptr1 = inptr1.add(2);
            outcol += 1;
        }
        inrow += 2;
        outrow += 1;
    }
}


/* #ifdef INPUT_SMOOTHING_SUPPORTED */

/*
 * Downsample pixel values of a single component.
 * This version handles the standard case of 2:1 horizontal and 2:1 vertical,
 * with smoothing.  One row of context is required.
 */

#[cfg(feature = "input_smoothing_supported")]
unsafe fn h2v2_smooth_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    let mut inrow: core::ffi::c_int;
    let mut outrow: core::ffi::c_int;
    let mut colctr: JDIMENSION;
    let output_cols: JDIMENSION = (*compptr).width_in_blocks * DCTSIZE as JDIMENSION;
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
        input_data.offset(-1),
        (*cinfo).max_v_samp_factor + 2,
        (*cinfo).image_width,
        output_cols * 2,
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

    let memberscale = 16384 - (*cinfo).smoothing_factor as INT32 * 80; /* scaled (1-5*SF)/4 */
    let neighscale = (*cinfo).smoothing_factor as INT32 * 16; /* scaled SF/4 */

    inrow = 0;
    outrow = 0;
    while outrow < (*compptr).v_samp_factor {
        outptr = *output_data.add(outrow as usize);
        inptr0 = *input_data.add(inrow as usize);
        inptr1 = *input_data.add((inrow + 1) as usize);
        above_ptr = *input_data.offset(inrow as isize - 1);
        below_ptr = *input_data.add((inrow + 2) as usize);

        /* Special case for first column: pretend column -1 is same as column 0 */
        membersum = GETJSAMPLE(*inptr0) as INT32 + GETJSAMPLE(*inptr0.add(1)) as INT32
            + GETJSAMPLE(*inptr1) as INT32 + GETJSAMPLE(*inptr1.add(1)) as INT32;
        neighsum = GETJSAMPLE(*above_ptr) as INT32 + GETJSAMPLE(*above_ptr.add(1)) as INT32
            + GETJSAMPLE(*below_ptr) as INT32 + GETJSAMPLE(*below_ptr.add(1)) as INT32
            + GETJSAMPLE(*inptr0) as INT32 + GETJSAMPLE(*inptr0.add(2)) as INT32
            + GETJSAMPLE(*inptr1) as INT32 + GETJSAMPLE(*inptr1.add(2)) as INT32;
        neighsum += neighsum;
        neighsum += GETJSAMPLE(*above_ptr) as INT32 + GETJSAMPLE(*above_ptr.add(2)) as INT32
            + GETJSAMPLE(*below_ptr) as INT32 + GETJSAMPLE(*below_ptr.add(2)) as INT32;
        membersum = membersum * memberscale + neighsum * neighscale;
        *outptr = ((membersum + 32768) >> 16) as JSAMPLE;
        outptr = outptr.add(1);
        inptr0 = inptr0.add(2);
        inptr1 = inptr1.add(2);
        above_ptr = above_ptr.add(2);
        below_ptr = below_ptr.add(2);

        colctr = output_cols.wrapping_sub(2);
        while colctr > 0 {
            /* sum of pixels directly mapped to this output element */
            membersum = GETJSAMPLE(*inptr0) as INT32 + GETJSAMPLE(*inptr0.add(1)) as INT32
                + GETJSAMPLE(*inptr1) as INT32 + GETJSAMPLE(*inptr1.add(1)) as INT32;
            /* sum of edge-neighbor pixels */
            neighsum = GETJSAMPLE(*above_ptr) as INT32 + GETJSAMPLE(*above_ptr.add(1)) as INT32
                + GETJSAMPLE(*below_ptr) as INT32 + GETJSAMPLE(*below_ptr.add(1)) as INT32
                + GETJSAMPLE(*inptr0.offset(-1)) as INT32 + GETJSAMPLE(*inptr0.add(2)) as INT32
                + GETJSAMPLE(*inptr1.offset(-1)) as INT32 + GETJSAMPLE(*inptr1.add(2)) as INT32;
            /* The edge-neighbors count twice as much as corner-neighbors */
            neighsum += neighsum;
            /* Add in the corner-neighbors */
            neighsum += GETJSAMPLE(*above_ptr.offset(-1)) as INT32
                + GETJSAMPLE(*above_ptr.add(2)) as INT32
                + GETJSAMPLE(*below_ptr.offset(-1)) as INT32
                + GETJSAMPLE(*below_ptr.add(2)) as INT32;
            /* form final output scaled up by 2^16 */
            membersum = membersum * memberscale + neighsum * neighscale;
            /* round, descale and output it */
            *outptr = ((membersum + 32768) >> 16) as JSAMPLE;
            outptr = outptr.add(1);
            inptr0 = inptr0.add(2);
            inptr1 = inptr1.add(2);
            above_ptr = above_ptr.add(2);
            below_ptr = below_ptr.add(2);
            colctr -= 1;
        }

        /* Special case for last column */
        membersum = GETJSAMPLE(*inptr0) as INT32 + GETJSAMPLE(*inptr0.add(1)) as INT32
            + GETJSAMPLE(*inptr1) as INT32 + GETJSAMPLE(*inptr1.add(1)) as INT32;
        neighsum = GETJSAMPLE(*above_ptr) as INT32 + GETJSAMPLE(*above_ptr.add(1)) as INT32
            + GETJSAMPLE(*below_ptr) as INT32 + GETJSAMPLE(*below_ptr.add(1)) as INT32
            + GETJSAMPLE(*inptr0.offset(-1)) as INT32 + GETJSAMPLE(*inptr0.add(1)) as INT32
            + GETJSAMPLE(*inptr1.offset(-1)) as INT32 + GETJSAMPLE(*inptr1.add(1)) as INT32;
        neighsum += neighsum;
        neighsum += GETJSAMPLE(*above_ptr.offset(-1)) as INT32
            + GETJSAMPLE(*above_ptr.add(1)) as INT32
            + GETJSAMPLE(*below_ptr.offset(-1)) as INT32
            + GETJSAMPLE(*below_ptr.add(1)) as INT32;
        membersum = membersum * memberscale + neighsum * neighscale;
        *outptr = ((membersum + 32768) >> 16) as JSAMPLE;

        inrow += 2;
        outrow += 1;
    }
}


/*
 * Downsample pixel values of a single component.
 * This version handles the special case of a full-size component,
 * with smoothing.  One row of context is required.
 */

#[cfg(feature = "input_smoothing_supported")]
unsafe fn fullsize_smooth_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    let mut outrow: core::ffi::c_int;
    let mut colctr: JDIMENSION;
    let output_cols: JDIMENSION = (*compptr).width_in_blocks * DCTSIZE as JDIMENSION;
    let mut inptr: JSAMPROW;
    let mut above_ptr: JSAMPROW;
    let mut below_ptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut membersum: INT32;
    let mut neighsum: INT32;
    let memberscale: INT32;
    let neighscale: INT32;
    let mut colsum: core::ffi::c_int;
    let mut lastcolsum: core::ffi::c_int;
    let mut nextcolsum: core::ffi::c_int;

    /* Expand input data enough to let all the output samples be generated
     * by the standard loop.  Special-casing padded output would be more
     * efficient.
     */
    expand_right_edge(
        input_data.offset(-1),
        (*cinfo).max_v_samp_factor + 2,
        (*cinfo).image_width,
        output_cols,
    );

    /* Each of the eight neighbor pixels contributes a fraction SF to the
     * smoothed pixel, while the main pixel contributes (1-8*SF).  In order
     * to use integer arithmetic, these factors are multiplied by 2^16 = 65536.
     * Also recall that SF = smoothing_factor / 1024.
     */

    let memberscale = 65536 - (*cinfo).smoothing_factor as INT32 * 512; /* scaled 1-8*SF */
    let neighscale = (*cinfo).smoothing_factor as INT32 * 64; /* scaled SF */

    outrow = 0;
    while outrow < (*compptr).v_samp_factor {
        outptr = *output_data.add(outrow as usize);
        inptr = *input_data.add(outrow as usize);
        above_ptr = *input_data.offset(outrow as isize - 1);
        below_ptr = *input_data.add((outrow + 1) as usize);

        /* Special case for first column */
        colsum = GETJSAMPLE(*above_ptr) + GETJSAMPLE(*below_ptr)
            + GETJSAMPLE(*inptr);
        above_ptr = above_ptr.add(1);
        below_ptr = below_ptr.add(1);
        membersum = GETJSAMPLE(*inptr) as INT32;
        inptr = inptr.add(1);
        nextcolsum = GETJSAMPLE(*above_ptr) + GETJSAMPLE(*below_ptr)
            + GETJSAMPLE(*inptr);
        neighsum = (colsum + (colsum - membersum as core::ffi::c_int) + nextcolsum) as INT32;
        membersum = membersum * memberscale + neighsum * neighscale;
        *outptr = ((membersum + 32768) >> 16) as JSAMPLE;
        outptr = outptr.add(1);
        lastcolsum = colsum;
        colsum = nextcolsum;

        colctr = output_cols.wrapping_sub(2);
        while colctr > 0 {
            membersum = GETJSAMPLE(*inptr) as INT32;
            inptr = inptr.add(1);
            above_ptr = above_ptr.add(1);
            below_ptr = below_ptr.add(1);
            nextcolsum = GETJSAMPLE(*above_ptr) + GETJSAMPLE(*below_ptr)
                + GETJSAMPLE(*inptr);
            neighsum =
                (lastcolsum + (colsum - membersum as core::ffi::c_int) + nextcolsum) as INT32;
            membersum = membersum * memberscale + neighsum * neighscale;
            *outptr = ((membersum + 32768) >> 16) as JSAMPLE;
            outptr = outptr.add(1);
            lastcolsum = colsum;
            colsum = nextcolsum;
            colctr -= 1;
        }

        /* Special case for last column */
        membersum = GETJSAMPLE(*inptr) as INT32;
        neighsum = (lastcolsum + (colsum - membersum as core::ffi::c_int) + colsum) as INT32;
        membersum = membersum * memberscale + neighsum * neighscale;
        *outptr = ((membersum + 32768) >> 16) as JSAMPLE;

        outrow += 1;
    }
}

/* #endif */ /* INPUT_SMOOTHING_SUPPORTED */


/*
 * Module initialization routine for downsampling.
 * Note that we must select a routine for each component.
 */

pub unsafe fn jinit_downsampler(cinfo: j_compress_ptr) {
    let mut downsample: my_downsample_ptr;
    let mut ci: core::ffi::c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut smoothok: boolean = TRUE;

    downsample = (*(*cinfo).mem).alloc_small.unwrap()(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        core::mem::size_of::<my_downsampler>(),
    ) as my_downsample_ptr;
    (*cinfo).downsample = downsample as *mut jpeg_downsampler;
    (*downsample).pub_.start_pass = Some(start_pass_downsample);
    (*downsample).pub_.downsample = Some(sep_downsample);
    (*downsample).pub_.need_context_rows = FALSE;

    if (*cinfo).CCIR601_sampling != 0 {
        ERREXIT(cinfo, JERR_CCIR601_NOTIMPL);
    }

    /* Verify we can handle the sampling factors, and set up method pointers */
    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        if (*compptr).h_samp_factor == (*cinfo).max_h_samp_factor
            && (*compptr).v_samp_factor == (*cinfo).max_v_samp_factor
        {
            /* Porting note: C uses #ifdef INPUT_SMOOTHING_SUPPORTED to gate the
             * inner if/else (not an outer else-if arm), so we duplicate the
             * assignment under cfg(feature) / cfg(not(feature)) blocks. */
            #[cfg(feature = "input_smoothing_supported")]
            if (*cinfo).smoothing_factor != 0 {
                (*downsample).methods[ci as usize] = Some(fullsize_smooth_downsample);
                (*downsample).pub_.need_context_rows = TRUE;
            } else {
                (*downsample).methods[ci as usize] = Some(fullsize_downsample);
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
            /* Same cfg pattern as the fullsize case above */
            #[cfg(feature = "input_smoothing_supported")]
            if (*cinfo).smoothing_factor != 0 {
                (*downsample).methods[ci as usize] = Some(h2v2_smooth_downsample);
                (*downsample).pub_.need_context_rows = TRUE;
            } else {
                (*downsample).methods[ci as usize] = Some(h2v2_downsample);
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
            ERREXIT(cinfo, JERR_FRACT_SAMPLE_NOTIMPL);
        }
        ci += 1;
        compptr = compptr.add(1);
    }

    /* #ifdef INPUT_SMOOTHING_SUPPORTED */
    #[cfg(feature = "input_smoothing_supported")]
    if (*cinfo).smoothing_factor != 0 && smoothok == 0 {
        TRACEMS(cinfo, 0, JTRC_SMOOTH_NOTIMPL);
    }
    /* #endif */
}
