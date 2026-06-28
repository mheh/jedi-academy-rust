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

use core::ffi::{c_int, c_void};

/* ============================================================================
 * Stubs for JPEG-6 types and structures needed for structural coherence
 * ============================================================================ */

pub type JSAMPLE = u8;
pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type JDIMENSION = u32;
pub type INT32 = i32;
pub type boolean = u8;

const TRUE: boolean = 1;
const FALSE: boolean = 0;

pub const MAX_COMPONENTS: c_int = 10;
const DCTSIZE: c_int = 8;

const JPOOL_IMAGE: c_int = 1;

const JERR_CCIR601_NOTIMPL: c_int = 37;
const JERR_FRACT_SAMPLE_NOTIMPL: c_int = 38;
const JTRC_SMOOTH_NOTIMPL: c_int = 1;

#[repr(C)]
pub struct jpeg_component_info {
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
    pub width_in_blocks: c_int,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_downsampler {
    pub start_pass: Option<unsafe extern "C" fn(j_compress_ptr)>,
    pub downsample: Option<unsafe extern "C" fn(j_compress_ptr, JSAMPIMAGE, JDIMENSION, JSAMPIMAGE, JDIMENSION)>,
    pub need_context_rows: boolean,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void>,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_error_mgr {
    pub msg_code: c_int,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct j_compress_struct {
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub downsample: *mut jpeg_downsampler,
    pub max_h_samp_factor: c_int,
    pub max_v_samp_factor: c_int,
    pub image_width: JDIMENSION,
    pub num_components: c_int,
    pub comp_info: *mut jpeg_component_info,
    pub CCIR601_sampling: boolean,
    pub smoothing_factor: c_int,
    _opaque: [u8; 0],
}

pub type j_compress_ptr = *mut j_compress_struct;
pub type j_common_ptr = *mut c_void;

/* External function declarations */
extern "C" {
    pub fn jcopy_sample_rows(input_array: JSAMPARRAY, source_row: c_int,
                             output_array: JSAMPARRAY, dest_row: c_int,
                             num_rows: c_int, num_cols: JDIMENSION);
}

/* Pointer to routine to downsample a single component */
type downsample1_ptr = unsafe extern "C" fn(
    j_compress_ptr,
    *mut jpeg_component_info,
    JSAMPARRAY,
    JSAMPARRAY,
);

/* Private subobject */

#[repr(C)]
struct my_downsampler {
    pub_: jpeg_downsampler,	/* public fields */

    /* Downsampling method pointers, one per component */
    methods: [downsample1_ptr; MAX_COMPONENTS as usize],
}

type my_downsample_ptr = *mut my_downsampler;

/* Macros */

#[inline]
unsafe fn ERREXIT(cinfo: j_compress_ptr, code: c_int) {
    (*(*cinfo).err).msg_code = code;
}

#[inline]
unsafe fn TRACEMS(cinfo: j_compress_ptr, _level: c_int, _code: c_int) {
    (*(*cinfo).err).msg_code = _code;
}

#[inline]
fn SIZEOF<T>() -> usize {
    core::mem::size_of::<T>()
}

/* Macro: GETJSAMPLE - convert JSAMPLE (u8) to signed int for arithmetic */
#[inline]
fn GETJSAMPLE(val: JSAMPLE) -> c_int {
    val as c_int
}


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
    num_rows: c_int,
    input_cols: JDIMENSION,
    output_cols: JDIMENSION,
) {
    let mut ptr: JSAMPROW;
    let mut pixval: JSAMPLE;
    let mut count: c_int;
    let mut row: c_int;
    let numcols: c_int = (output_cols - input_cols) as c_int;

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
    let downsample = (*cinfo).downsample as *mut my_downsampler;
    let mut ci: c_int = 0;
    let mut compptr: *mut jpeg_component_info = (*cinfo).comp_info;

    while ci < (*cinfo).num_components {
        let in_ptr: JSAMPARRAY = (*input_buf.add(ci as usize)).add(in_row_index as usize);
        let out_ptr: JSAMPARRAY = (*output_buf.add(ci as usize))
            .add((out_row_group_index as c_int * (*compptr).v_samp_factor) as usize);
        ((*downsample).methods[ci as usize])(cinfo, compptr, in_ptr, out_ptr);
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
    let mut inrow: c_int;
    let mut outrow: c_int;
    let h_expand: c_int = (*cinfo).max_h_samp_factor / (*compptr).h_samp_factor;
    let v_expand: c_int = (*cinfo).max_v_samp_factor / (*compptr).v_samp_factor;
    let numpix: c_int = h_expand * v_expand;
    let numpix2: c_int = numpix / 2;
    let output_cols: JDIMENSION = ((*compptr).width_in_blocks * DCTSIZE) as u32;
    let mut outcol: JDIMENSION;
    let mut outcol_h: JDIMENSION;
    let mut outptr: JSAMPROW;
    let mut inptr: JSAMPROW;
    let mut outvalue: INT32;
    let mut v: c_int;
    let mut h: c_int;

    /* Expand input data enough to let all the output samples be generated
     * by the standard loop.  Special-casing padded output would be more
     * efficient.
     */
    expand_right_edge(
        input_data,
        (*cinfo).max_v_samp_factor,
        (*cinfo).image_width,
        output_cols * h_expand as u32,
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
            outcol_h = (outcol_h as c_int + h_expand) as JDIMENSION;
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
        ((*compptr).width_in_blocks * DCTSIZE) as JDIMENSION,
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
    let mut outrow: c_int;
    let mut outcol: JDIMENSION;
    let output_cols: JDIMENSION = ((*compptr).width_in_blocks * DCTSIZE) as u32;
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
        output_cols * 2,
    );

    outrow = 0;
    while outrow < (*compptr).v_samp_factor {
        outptr = *output_data.add(outrow as usize);
        inptr = *input_data.add(outrow as usize);
        bias = 0;			/* bias = 0,1,0,1,... for successive samples */
        outcol = 0;
        while outcol < output_cols {
            *outptr = ((GETJSAMPLE(*inptr) + GETJSAMPLE(*inptr.offset(1)) + bias) >> 1) as JSAMPLE;
            bias ^= 1;		/* 0=>1, 1=>0 */
            inptr = inptr.add(2);
            outptr = outptr.add(1);
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
    let mut inrow: c_int;
    let mut outrow: c_int;
    let mut outcol: JDIMENSION;
    let output_cols: JDIMENSION = ((*compptr).width_in_blocks * DCTSIZE) as u32;
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
            *outptr = ((GETJSAMPLE(*inptr0) + GETJSAMPLE(*inptr0.offset(1))
                + GETJSAMPLE(*inptr1) + GETJSAMPLE(*inptr1.offset(1)) + bias) >> 2) as JSAMPLE;
            bias ^= 3;		/* 1=>2, 2=>1 */
            inptr0 = inptr0.add(2);
            inptr1 = inptr1.add(2);
            outptr = outptr.add(1);
            outcol += 1;
        }
        inrow += 2;
        outrow += 1;
    }
}


#[cfg(feature = "input_smoothing_supported")]
/*
 * Downsample pixel values of a single component.
 * This version handles the standard case of 2:1 horizontal and 2:1 vertical,
 * with smoothing.  One row of context is required.
 */

unsafe fn h2v2_smooth_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    let mut inrow: c_int;
    let mut outrow: c_int;
    let mut colctr: JDIMENSION;
    let output_cols: JDIMENSION = ((*compptr).width_in_blocks * DCTSIZE) as u32;
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
        (*input_data).offset(-1),
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

    let memberscale = 16384i32 - ((*cinfo).smoothing_factor as i32) * 80; /* scaled (1-5*SF)/4 */
    let neighscale = ((*cinfo).smoothing_factor as i32) * 16; /* scaled SF/4 */

    inrow = 0;
    outrow = 0;
    while outrow < (*compptr).v_samp_factor {
        outptr = *output_data.add(outrow as usize);
        inptr0 = *input_data.add(inrow as usize);
        inptr1 = *input_data.add((inrow + 1) as usize);
        above_ptr = *input_data.add((inrow - 1) as usize);
        below_ptr = *input_data.add((inrow + 2) as usize);

        /* Special case for first column: pretend column -1 is same as column 0 */
        membersum = GETJSAMPLE(*inptr0) as INT32 + GETJSAMPLE(*inptr0.offset(1)) as INT32
            + GETJSAMPLE(*inptr1) as INT32 + GETJSAMPLE(*inptr1.offset(1)) as INT32;
        neighsum = GETJSAMPLE(*above_ptr) as INT32 + GETJSAMPLE(*above_ptr.offset(1)) as INT32
            + GETJSAMPLE(*below_ptr) as INT32 + GETJSAMPLE(*below_ptr.offset(1)) as INT32
            + GETJSAMPLE(*inptr0) as INT32 + GETJSAMPLE(*inptr0.offset(2)) as INT32
            + GETJSAMPLE(*inptr1) as INT32 + GETJSAMPLE(*inptr1.offset(2)) as INT32;
        neighsum += neighsum;
        neighsum += GETJSAMPLE(*above_ptr) as INT32 + GETJSAMPLE(*above_ptr.offset(2)) as INT32
            + GETJSAMPLE(*below_ptr) as INT32 + GETJSAMPLE(*below_ptr.offset(2)) as INT32;
        membersum = membersum * memberscale + neighsum * neighscale;
        *outptr = ((membersum + 32768) >> 16) as JSAMPLE;
        inptr0 = inptr0.add(2);
        inptr1 = inptr1.add(2);
        above_ptr = above_ptr.add(2);
        below_ptr = below_ptr.add(2);
        outptr = outptr.add(1);

        colctr = output_cols - 2;
        while colctr > 0 {
            /* sum of pixels directly mapped to this output element */
            membersum = GETJSAMPLE(*inptr0) as INT32 + GETJSAMPLE(*inptr0.offset(1)) as INT32
                + GETJSAMPLE(*inptr1) as INT32 + GETJSAMPLE(*inptr1.offset(1)) as INT32;
            /* sum of edge-neighbor pixels */
            neighsum = GETJSAMPLE(*above_ptr) as INT32 + GETJSAMPLE(*above_ptr.offset(1)) as INT32
                + GETJSAMPLE(*below_ptr) as INT32 + GETJSAMPLE(*below_ptr.offset(1)) as INT32
                + GETJSAMPLE(*inptr0.offset(-1)) as INT32 + GETJSAMPLE(*inptr0.offset(2)) as INT32
                + GETJSAMPLE(*inptr1.offset(-1)) as INT32 + GETJSAMPLE(*inptr1.offset(2)) as INT32;
            /* The edge-neighbors count twice as much as corner-neighbors */
            neighsum += neighsum;
            /* Add in the corner-neighbors */
            neighsum += GETJSAMPLE(*above_ptr.offset(-1)) as INT32 + GETJSAMPLE(*above_ptr.offset(2)) as INT32
                + GETJSAMPLE(*below_ptr.offset(-1)) as INT32 + GETJSAMPLE(*below_ptr.offset(2)) as INT32;
            /* form final output scaled up by 2^16 */
            membersum = membersum * memberscale + neighsum * neighscale;
            /* round, descale and output it */
            *outptr = ((membersum + 32768) >> 16) as JSAMPLE;
            inptr0 = inptr0.add(2);
            inptr1 = inptr1.add(2);
            above_ptr = above_ptr.add(2);
            below_ptr = below_ptr.add(2);
            outptr = outptr.add(1);
            colctr -= 1;
        }

        /* Special case for last column */
        membersum = GETJSAMPLE(*inptr0) as INT32 + GETJSAMPLE(*inptr0.offset(1)) as INT32
            + GETJSAMPLE(*inptr1) as INT32 + GETJSAMPLE(*inptr1.offset(1)) as INT32;
        neighsum = GETJSAMPLE(*above_ptr) as INT32 + GETJSAMPLE(*above_ptr.offset(1)) as INT32
            + GETJSAMPLE(*below_ptr) as INT32 + GETJSAMPLE(*below_ptr.offset(1)) as INT32
            + GETJSAMPLE(*inptr0.offset(-1)) as INT32 + GETJSAMPLE(*inptr0.offset(1)) as INT32
            + GETJSAMPLE(*inptr1.offset(-1)) as INT32 + GETJSAMPLE(*inptr1.offset(1)) as INT32;
        neighsum += neighsum;
        neighsum += GETJSAMPLE(*above_ptr.offset(-1)) as INT32 + GETJSAMPLE(*above_ptr.offset(1)) as INT32
            + GETJSAMPLE(*below_ptr.offset(-1)) as INT32 + GETJSAMPLE(*below_ptr.offset(1)) as INT32;
        membersum = membersum * memberscale + neighsum * neighscale;
        *outptr = ((membersum + 32768) >> 16) as JSAMPLE;

        inrow += 2;
        outrow += 1;
    }
}


#[cfg(feature = "input_smoothing_supported")]
/*
 * Downsample pixel values of a single component.
 * This version handles the special case of a full-size component,
 * with smoothing.  One row of context is required.
 */

unsafe fn fullsize_smooth_downsample(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    input_data: JSAMPARRAY,
    output_data: JSAMPARRAY,
) {
    let mut outrow: c_int;
    let mut colctr: JDIMENSION;
    let output_cols: JDIMENSION = ((*compptr).width_in_blocks * DCTSIZE) as u32;
    let mut inptr: JSAMPROW;
    let mut above_ptr: JSAMPROW;
    let mut below_ptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut membersum: INT32;
    let mut neighsum: INT32;
    let memberscale: INT32;
    let neighscale: INT32;
    let mut colsum: c_int;
    let mut lastcolsum: c_int;
    let mut nextcolsum: c_int;

    /* Expand input data enough to let all the output samples be generated
     * by the standard loop.  Special-casing padded output would be more
     * efficient.
     */
    expand_right_edge(
        (*input_data).offset(-1),
        (*cinfo).max_v_samp_factor + 2,
        (*cinfo).image_width,
        output_cols,
    );

    /* Each of the eight neighbor pixels contributes a fraction SF to the
     * smoothed pixel, while the main pixel contributes (1-8*SF).  In order
     * to use integer arithmetic, these factors are multiplied by 2^16 = 65536.
     * Also recall that SF = smoothing_factor / 1024.
     */

    let memberscale = 65536i32 - ((*cinfo).smoothing_factor as i32) * 512; /* scaled 1-8*SF */
    let neighscale = ((*cinfo).smoothing_factor as i32) * 64; /* scaled SF */

    outrow = 0;
    while outrow < (*compptr).v_samp_factor {
        outptr = *output_data.add(outrow as usize);
        inptr = *input_data.add(outrow as usize);
        above_ptr = *input_data.add((outrow - 1) as usize);
        below_ptr = *input_data.add((outrow + 1) as usize);

        /* Special case for first column */
        colsum = GETJSAMPLE(*above_ptr) as c_int + GETJSAMPLE(*below_ptr) as c_int
            + GETJSAMPLE(*inptr) as c_int;
        above_ptr = above_ptr.add(1);
        below_ptr = below_ptr.add(1);
        membersum = GETJSAMPLE(*inptr) as INT32;
        inptr = inptr.add(1);
        nextcolsum = GETJSAMPLE(*above_ptr) as c_int + GETJSAMPLE(*below_ptr) as c_int
            + GETJSAMPLE(*inptr) as c_int;
        neighsum = (colsum + (colsum - membersum as c_int) + nextcolsum) as INT32;
        membersum = membersum * memberscale + neighsum * neighscale;
        *outptr = ((membersum + 32768) >> 16) as JSAMPLE;
        outptr = outptr.add(1);
        lastcolsum = colsum;
        colsum = nextcolsum;

        colctr = output_cols - 2;
        while colctr > 0 {
            membersum = GETJSAMPLE(*inptr) as INT32;
            inptr = inptr.add(1);
            above_ptr = above_ptr.add(1);
            below_ptr = below_ptr.add(1);
            nextcolsum = GETJSAMPLE(*above_ptr) as c_int + GETJSAMPLE(*below_ptr) as c_int
                + GETJSAMPLE(*inptr) as c_int;
            neighsum = (lastcolsum + (colsum - membersum as c_int) + nextcolsum) as INT32;
            membersum = membersum * memberscale + neighsum * neighscale;
            *outptr = ((membersum + 32768) >> 16) as JSAMPLE;
            outptr = outptr.add(1);
            lastcolsum = colsum;
            colsum = nextcolsum;
            colctr -= 1;
        }

        /* Special case for last column */
        membersum = GETJSAMPLE(*inptr) as INT32;
        neighsum = (lastcolsum + (colsum - membersum as c_int) + colsum) as INT32;
        membersum = membersum * memberscale + neighsum * neighscale;
        *outptr = ((membersum + 32768) >> 16) as JSAMPLE;

        outrow += 1;
    }
}


/*
 * Module initialization routine for downsampling.
 * Note that we must select a routine for each component.
 */

pub unsafe fn jinit_downsampler(cinfo: j_compress_ptr) {
    let mut downsample: *mut my_downsampler;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut smoothok: boolean = TRUE;

    downsample = (*(*cinfo).mem).alloc_small.unwrap()(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        SIZEOF::<my_downsampler>(),
    ) as *mut my_downsampler;
    (*cinfo).downsample = &mut (*downsample).pub_ as *mut jpeg_downsampler;
    (*(*cinfo).downsample).start_pass = Some(start_pass_downsample);
    (*(*cinfo).downsample).downsample = Some(sep_downsample);
    (*(*cinfo).downsample).need_context_rows = FALSE;

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
            #[cfg(feature = "input_smoothing_supported")]
            {
                if (*cinfo).smoothing_factor != 0 {
                    (*downsample).methods[ci as usize] = fullsize_smooth_downsample;
                    (*(*cinfo).downsample).need_context_rows = TRUE;
                } else {
                    (*downsample).methods[ci as usize] = fullsize_downsample;
                }
            }
            #[cfg(not(feature = "input_smoothing_supported"))]
            {
                (*downsample).methods[ci as usize] = fullsize_downsample;
            }
        } else if (*compptr).h_samp_factor * 2 == (*cinfo).max_h_samp_factor
            && (*compptr).v_samp_factor == (*cinfo).max_v_samp_factor
        {
            smoothok = FALSE;
            (*downsample).methods[ci as usize] = h2v1_downsample;
        } else if (*compptr).h_samp_factor * 2 == (*cinfo).max_h_samp_factor
            && (*compptr).v_samp_factor * 2 == (*cinfo).max_v_samp_factor
        {
            #[cfg(feature = "input_smoothing_supported")]
            {
                if (*cinfo).smoothing_factor != 0 {
                    (*downsample).methods[ci as usize] = h2v2_smooth_downsample;
                    (*(*cinfo).downsample).need_context_rows = TRUE;
                } else {
                    (*downsample).methods[ci as usize] = h2v2_downsample;
                }
            }
            #[cfg(not(feature = "input_smoothing_supported"))]
            {
                (*downsample).methods[ci as usize] = h2v2_downsample;
            }
        } else if ((*cinfo).max_h_samp_factor % (*compptr).h_samp_factor) == 0
            && ((*cinfo).max_v_samp_factor % (*compptr).v_samp_factor) == 0
        {
            smoothok = FALSE;
            (*downsample).methods[ci as usize] = int_downsample;
        } else {
            ERREXIT(cinfo, JERR_FRACT_SAMPLE_NOTIMPL);
        }
        ci += 1;
        compptr = compptr.add(1);
    }

    #[cfg(feature = "input_smoothing_supported")]
    {
        if (*cinfo).smoothing_factor != 0 && smoothok == FALSE {
            TRACEMS(cinfo, 0, JTRC_SMOOTH_NOTIMPL);
        }
    }
}
