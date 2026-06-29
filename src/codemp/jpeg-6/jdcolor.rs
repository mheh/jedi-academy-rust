/*
 * jdcolor.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains output colorspace conversion routines.
 */

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr;

/* ============================================================================
 * Stubs for JPEG-6 types and structures needed for structural coherence
 * ============================================================================ */

pub type INT32 = i32;
pub type JSAMPLE = u8;
pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type JDIMENSION = c_int;
pub type boolean = u8;

const TRUE: boolean = 1;
const FALSE: boolean = 0;

const MAXJSAMPLE: c_int = 255;
const CENTERJSAMPLE: c_int = 128;

const RGB_RED: c_int = 0;
const RGB_GREEN: c_int = 1;
const RGB_BLUE: c_int = 2;
const RGB_PIXELSIZE: c_int = 4;

const JPOOL_IMAGE: c_int = 1;

const JERR_BAD_J_COLORSPACE: c_int = 20;
const JERR_CONVERSION_NOTIMPL: c_int = 23;

#[repr(C)]
pub struct jpeg_component_info {
    pub component_needed: boolean,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, c_int) -> *mut c_void>,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_error_mgr {
    pub msg_code: c_int,
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_color_deconverter {
    pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    pub color_convert: Option<unsafe extern "C" fn(j_decompress_ptr, JSAMPIMAGE, JDIMENSION, JSAMPARRAY, c_int)>,
}

#[repr(C)]
pub struct j_decompress_struct {
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub cconvert: *mut jpeg_color_deconverter,
    pub sample_range_limit: *mut JSAMPLE,
    pub output_width: JDIMENSION,
    pub num_components: c_int,
    pub jpeg_color_space: c_int,
    pub out_color_space: c_int,
    pub comp_info: *mut jpeg_component_info,
    pub out_color_components: c_int,
    pub output_components: c_int,
    pub quantize_colors: boolean,
    _opaque: [u8; 0],
}

pub type j_decompress_ptr = *mut j_decompress_struct;
pub type j_common_ptr = *mut j_decompress_struct;

/* Color space constants */
const JCS_UNKNOWN: c_int = 0;
const JCS_GRAYSCALE: c_int = 1;
const JCS_RGB: c_int = 2;
const JCS_YCbCr: c_int = 3;
const JCS_CMYK: c_int = 4;
const JCS_YCCK: c_int = 5;

/* Private subobject */

#[repr(C)]
struct my_color_deconverter {
    pub pub_: jpeg_color_deconverter, /* public fields */

    /* Private state for YCC->RGB conversion */
    Cr_r_tab: *mut c_int,       /* => table for Cr to R conversion */
    Cb_b_tab: *mut c_int,       /* => table for Cb to B conversion */
    Cr_g_tab: *mut INT32,       /* => table for Cr to G conversion */
    Cb_g_tab: *mut INT32,       /* => table for Cb to G conversion */
}

type my_cconvert_ptr = *mut my_color_deconverter;

/**************** YCbCr -> RGB conversion: most common case **************/

/*
 * YCbCr is defined per CCIR 601-1, except that Cb and Cr are
 * normalized to the range 0..MAXJSAMPLE rather than -0.5 .. 0.5.
 * The conversion equations to be implemented are therefore
 *	R = Y                + 1.40200 * Cr
 *	G = Y - 0.34414 * Cb - 0.71414 * Cr
 *	B = Y + 1.77200 * Cb
 * where Cb and Cr represent the incoming values less CENTERJSAMPLE.
 * (These numbers are derived from TIFF 6.0 section 21, dated 3-June-92.)
 *
 * To avoid floating-point arithmetic, we represent the fractional constants
 * as integers scaled up by 2^16 (about 4 digits precision); we have to divide
 * the products by 2^16, with appropriate rounding, to get the correct answer.
 * Notice that Y, being an integral input, does not contribute any fraction
 * so it need not participate in the rounding.
 *
 * For even more speed, we avoid doing any multiplications in the inner loop
 * by precalculating the constants times Cb and Cr for all possible values.
 * For 8-bit JSAMPLEs this is very reasonable (only 256 entries per table);
 * for 12-bit samples it is still acceptable.  It's not very reasonable for
 * 16-bit samples, but if you want lossless storage you shouldn't be changing
 * colorspace anyway.
 * The Cr=>R and Cb=>B values can be rounded to integers in advance; the
 * values for the G calculation are left scaled up, since we must add them
 * together before rounding.
 */

const SCALEBITS: u32 = 16;        /* speediest right-shift on some machines */
const ONE_HALF: INT32 = 1i32 << (SCALEBITS - 1);

/// FIX(x) - Translate fractional constant to fixed-point
/// Translates: ((INT32) ((x) * (1L<<SCALEBITS) + 0.5))
#[inline]
fn FIX(x: f64) -> INT32 {
    ((x * (1i64 << SCALEBITS) as f64 + 0.5) as i64) as INT32
}

/// RIGHT_SHIFT(x, n) - Signed right shift with proper rounding
#[inline]
fn RIGHT_SHIFT(x: INT32, shft: u32) -> INT32 {
    x >> shft
}

/// SIZEOF(T) - Get size of type T as c_int
#[inline]
fn SIZEOF_INT() -> c_int {
    core::mem::size_of::<c_int>() as c_int
}

/// GETJSAMPLE(value) - Extract JSAMPLE as i32
#[inline]
fn GETJSAMPLE(value: JSAMPLE) -> c_int {
    value as c_int
}

/*
 * Initialize tables for YCC->RGB colorspace conversion.
 */

unsafe fn build_ycc_rgb_table(cinfo: j_decompress_ptr) {
    let cconvert = (*cinfo).cconvert as my_cconvert_ptr;
    let mut i: c_int;
    let mut x: INT32;

    (*cconvert).Cr_r_tab = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        ((MAXJSAMPLE + 1) * SIZEOF_INT()) as c_int,
    ) as *mut c_int;
    (*cconvert).Cb_b_tab = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        ((MAXJSAMPLE + 1) * SIZEOF_INT()) as c_int,
    ) as *mut c_int;
    (*cconvert).Cr_g_tab = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        ((MAXJSAMPLE + 1) * core::mem::size_of::<INT32>() as c_int) as c_int,
    ) as *mut INT32;
    (*cconvert).Cb_g_tab = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        ((MAXJSAMPLE + 1) * core::mem::size_of::<INT32>() as c_int) as c_int,
    ) as *mut INT32;

    i = 0;
    x = -CENTERJSAMPLE;
    while i <= MAXJSAMPLE {
        /* i is the actual input pixel value, in the range 0..MAXJSAMPLE */
        /* The Cb or Cr value we are thinking of is x = i - CENTERJSAMPLE */
        /* Cr=>R value is nearest int to 1.40200 * x */
        let cr_r_val = RIGHT_SHIFT(FIX(1.40200) * x + ONE_HALF, SCALEBITS);
        ptr::write(
            (*cconvert).Cr_r_tab.offset(i as isize),
            cr_r_val,
        );
        /* Cb=>B value is nearest int to 1.77200 * x */
        let cb_b_val = RIGHT_SHIFT(FIX(1.77200) * x + ONE_HALF, SCALEBITS);
        ptr::write(
            (*cconvert).Cb_b_tab.offset(i as isize),
            cb_b_val,
        );
        /* Cr=>G value is scaled-up -0.71414 * x */
        ptr::write(
            (*cconvert).Cr_g_tab.offset(i as isize),
            (-FIX(0.71414)) * x,
        );
        /* Cb=>G value is scaled-up -0.34414 * x */
        /* We also add in ONE_HALF so that need not do it in inner loop */
        ptr::write(
            (*cconvert).Cb_g_tab.offset(i as isize),
            (-FIX(0.34414)) * x + ONE_HALF,
        );
        i += 1;
        x += 1;
    }
}

/*
 * Convert some rows of samples to the output colorspace.
 *
 * Note that we change from noninterleaved, one-plane-per-component format
 * to interleaved-pixel format.  The output buffer is therefore three times
 * as wide as the input buffer.
 * A starting row offset is provided only for the input buffer.  The caller
 * can easily adjust the passed output_buf value to accommodate any row
 * offset required on that side.
 */

unsafe fn ycc_rgb_convert(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    input_row: JDIMENSION,
    output_buf: JSAMPARRAY,
    num_rows: c_int,
) {
    let cconvert = (*cinfo).cconvert as my_cconvert_ptr;
    let mut y: c_int;
    let mut cb: c_int;
    let mut cr: c_int;
    let mut outptr: JSAMPROW;
    let mut inptr0: JSAMPROW;
    let mut inptr1: JSAMPROW;
    let mut inptr2: JSAMPROW;
    let mut col: JDIMENSION;
    let num_cols = (*cinfo).output_width;
    /* copy these pointers into registers if possible */
    let range_limit = (*cinfo).sample_range_limit;
    let Crrtab = (*cconvert).Cr_r_tab;
    let Cbbtab = (*cconvert).Cb_b_tab;
    let Crgtab = (*cconvert).Cr_g_tab;
    let Cbgtab = (*cconvert).Cb_g_tab;

    let mut num_rows_mut = num_rows;
    let mut input_row_mut = input_row;
    let mut output_buf_mut = output_buf;

    while num_rows_mut > 0 {
        num_rows_mut -= 1;
        inptr0 = *input_buf.offset(0).offset(input_row_mut as isize);
        inptr1 = *input_buf.offset(1).offset(input_row_mut as isize);
        inptr2 = *input_buf.offset(2).offset(input_row_mut as isize);
        input_row_mut += 1;
        outptr = *output_buf_mut;
        output_buf_mut = output_buf_mut.offset(1);
        col = 0;
        while col < num_cols {
            y = GETJSAMPLE(*inptr0.offset(col as isize));
            cb = GETJSAMPLE(*inptr1.offset(col as isize));
            cr = GETJSAMPLE(*inptr2.offset(col as isize));
            /* Range-limiting is essential due to noise introduced by DCT losses. */
            *outptr.offset(RGB_RED as isize) = *range_limit.offset((y + *Crrtab.offset(cr as isize)) as isize);
            *outptr.offset(RGB_GREEN as isize) = *range_limit.offset((y +
                            RIGHT_SHIFT(*Cbgtab.offset(cb as isize) + *Crgtab.offset(cr as isize),
                                        SCALEBITS)) as isize);
            *outptr.offset(RGB_BLUE as isize) = *range_limit.offset((y + *Cbbtab.offset(cb as isize)) as isize);
            outptr = outptr.offset(RGB_PIXELSIZE as isize);
            col += 1;
        }
    }
}

/**************** Cases other than YCbCr -> RGB **************/

/*
 * Color conversion for no colorspace change: just copy the data,
 * converting from separate-planes to interleaved representation.
 */

unsafe fn null_convert(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    input_row: JDIMENSION,
    output_buf: JSAMPARRAY,
    num_rows: c_int,
) {
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut count: JDIMENSION;
    let num_components = (*cinfo).num_components;
    let num_cols = (*cinfo).output_width;
    let mut ci: c_int;

    let mut num_rows_mut = num_rows;
    let mut input_row_mut = input_row;
    let mut output_buf_mut = output_buf;

    while num_rows_mut > 0 {
        num_rows_mut -= 1;
        ci = 0;
        while ci < num_components {
            inptr = *input_buf.offset(ci as isize).offset(input_row_mut as isize);
            outptr = *output_buf_mut.offset(0).offset(ci as isize);
            count = num_cols;
            while count > 0 {
                *outptr = *inptr;                   /* needn't bother with GETJSAMPLE() here */
                inptr = inptr.offset(1);
                outptr = outptr.offset(num_components as isize);
                count -= 1;
            }
            ci += 1;
        }
        input_row_mut += 1;
        output_buf_mut = output_buf_mut.offset(1);
    }
}

/*
 * Color conversion for grayscale: just copy the data.
 * This also works for YCbCr -> grayscale conversion, in which
 * we just copy the Y (luminance) component and ignore chrominance.
 */

extern "C" {
    fn jcopy_sample_rows(
        input_array: JSAMPARRAY,
        source_row: c_int,
        output_array: JSAMPARRAY,
        dest_row: c_int,
        num_rows: c_int,
        num_cols: JDIMENSION,
    );
}

unsafe fn grayscale_convert(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    input_row: JDIMENSION,
    output_buf: JSAMPARRAY,
    num_rows: c_int,
) {
    jcopy_sample_rows(
        *input_buf.offset(0),
        input_row as c_int,
        output_buf,
        0,
        num_rows,
        (*cinfo).output_width,
    );
}

/*
 * Adobe-style YCCK->CMYK conversion.
 * We convert YCbCr to R=1-C, G=1-M, and B=1-Y using the same
 * conversion as above, while passing K (black) unchanged.
 * We assume build_ycc_rgb_table has been called.
 */

unsafe fn ycck_cmyk_convert(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    input_row: JDIMENSION,
    output_buf: JSAMPARRAY,
    num_rows: c_int,
) {
    let cconvert = (*cinfo).cconvert as my_cconvert_ptr;
    let mut y: c_int;
    let mut cb: c_int;
    let mut cr: c_int;
    let mut outptr: JSAMPROW;
    let mut inptr0: JSAMPROW;
    let mut inptr1: JSAMPROW;
    let mut inptr2: JSAMPROW;
    let mut inptr3: JSAMPROW;
    let mut col: JDIMENSION;
    let num_cols = (*cinfo).output_width;
    /* copy these pointers into registers if possible */
    let range_limit = (*cinfo).sample_range_limit;
    let Crrtab = (*cconvert).Cr_r_tab;
    let Cbbtab = (*cconvert).Cb_b_tab;
    let Crgtab = (*cconvert).Cr_g_tab;
    let Cbgtab = (*cconvert).Cb_g_tab;

    let mut num_rows_mut = num_rows;
    let mut input_row_mut = input_row;
    let mut output_buf_mut = output_buf;

    while num_rows_mut > 0 {
        num_rows_mut -= 1;
        inptr0 = *input_buf.offset(0).offset(input_row_mut as isize);
        inptr1 = *input_buf.offset(1).offset(input_row_mut as isize);
        inptr2 = *input_buf.offset(2).offset(input_row_mut as isize);
        inptr3 = *input_buf.offset(3).offset(input_row_mut as isize);
        input_row_mut += 1;
        outptr = *output_buf_mut;
        output_buf_mut = output_buf_mut.offset(1);
        col = 0;
        while col < num_cols {
            y = GETJSAMPLE(*inptr0.offset(col as isize));
            cb = GETJSAMPLE(*inptr1.offset(col as isize));
            cr = GETJSAMPLE(*inptr2.offset(col as isize));
            /* Range-limiting is essential due to noise introduced by DCT losses. */
            *outptr.offset(0) = *range_limit.offset((MAXJSAMPLE - (y + *Crrtab.offset(cr as isize))) as isize);    /* red */
            *outptr.offset(1) = *range_limit.offset((MAXJSAMPLE - (y +                    /* green */
                            RIGHT_SHIFT(*Cbgtab.offset(cb as isize) + *Crgtab.offset(cr as isize),
                                        SCALEBITS))) as isize);
            *outptr.offset(2) = *range_limit.offset((MAXJSAMPLE - (y + *Cbbtab.offset(cb as isize))) as isize);    /* blue */
            /* K passes through unchanged */
            *outptr.offset(3) = *inptr3.offset(col as isize);  /* don't need GETJSAMPLE here */
            outptr = outptr.offset(4);
            col += 1;
        }
    }
}

/*
 * Empty method for start_pass.
 */

unsafe fn start_pass_dcolor(_cinfo: j_decompress_ptr) {
    /* no work needed */
}

/*
 * Module initialization routine for output colorspace conversion.
 */

extern "C" {
    fn ERREXIT(cinfo: j_decompress_ptr, code: c_int) -> !;
}

pub unsafe fn jinit_color_deconverter(cinfo: j_decompress_ptr) {
    let mut cconvert: my_cconvert_ptr;
    let mut ci: c_int;

    cconvert = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        core::mem::size_of::<my_color_deconverter>() as c_int,
    ) as my_cconvert_ptr;
    (*cinfo).cconvert = &mut (*cconvert).pub_ as *mut jpeg_color_deconverter;
    (*(*cinfo).cconvert).start_pass = Some(start_pass_dcolor);

    /* Make sure num_components agrees with jpeg_color_space */
    match (*cinfo).jpeg_color_space {
        JCS_GRAYSCALE => {
            if (*cinfo).num_components != 1 {
                ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
            }
        }

        JCS_RGB | JCS_YCbCr => {
            if (*cinfo).num_components != 3 {
                ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
            }
        }

        JCS_CMYK | JCS_YCCK => {
            if (*cinfo).num_components != 4 {
                ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
            }
        }

        _ => {
            /* JCS_UNKNOWN can be anything */
            if (*cinfo).num_components < 1 {
                ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
            }
        }
    }

    /* Set out_color_components and conversion method based on requested space.
     * Also clear the component_needed flags for any unused components,
     * so that earlier pipeline stages can avoid useless computation.
     */

    match (*cinfo).out_color_space {
        JCS_GRAYSCALE => {
            (*cinfo).out_color_components = 1;
            if (*cinfo).jpeg_color_space == JCS_GRAYSCALE
                || (*cinfo).jpeg_color_space == JCS_YCbCr
            {
                (*(*cinfo).cconvert).color_convert = Some(grayscale_convert);
                /* For color->grayscale conversion, only the Y (0) component is needed */
                ci = 1;
                while ci < (*cinfo).num_components {
                    (*(*(*cinfo).comp_info.offset(ci as isize))).component_needed = FALSE;
                    ci += 1;
                }
            } else {
                ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
            }
        }

        JCS_RGB => {
            (*cinfo).out_color_components = RGB_PIXELSIZE;
            if (*cinfo).jpeg_color_space == JCS_YCbCr {
                (*(*cinfo).cconvert).color_convert = Some(ycc_rgb_convert);
                build_ycc_rgb_table(cinfo);
            } else if (*cinfo).jpeg_color_space == JCS_RGB && RGB_PIXELSIZE == 3 {
                (*(*cinfo).cconvert).color_convert = Some(null_convert);
            } else {
                ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
            }
        }

        JCS_CMYK => {
            (*cinfo).out_color_components = 4;
            if (*cinfo).jpeg_color_space == JCS_YCCK {
                (*(*cinfo).cconvert).color_convert = Some(ycck_cmyk_convert);
                build_ycc_rgb_table(cinfo);
            } else if (*cinfo).jpeg_color_space == JCS_CMYK {
                (*(*cinfo).cconvert).color_convert = Some(null_convert);
            } else {
                ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
            }
        }

        _ => {
            /* Permit null conversion to same output space */
            if (*cinfo).out_color_space == (*cinfo).jpeg_color_space {
                (*cinfo).out_color_components = (*cinfo).num_components;
                (*(*cinfo).cconvert).color_convert = Some(null_convert);
            } else {
                /* unsupported non-null conversion */
                ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
            }
        }
    }

    if (*cinfo).quantize_colors != FALSE {
        (*cinfo).output_components = 1; /* single colormapped output component */
    } else {
        (*cinfo).output_components = (*cinfo).out_color_components;
    }
}
