/*
 * jdcolor.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains output colorspace conversion routines.
 */


// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::code::server::exe_headers_h::*;
use crate::code::jpeg_6::jinclude_h::*;
use crate::code::jpeg_6::jpeglib_h::*;
use core::ffi::c_int;
use core::mem::size_of;


/* Private subobject */

#[repr(C)]
struct my_color_deconverter {
    pub_: jpeg_color_deconverter, /* public fields */

    /* Private state for YCC->RGB conversion */
    Cr_r_tab: *mut c_int,    /* => table for Cr to R conversion */
    Cb_b_tab: *mut c_int,    /* => table for Cb to B conversion */
    Cr_g_tab: *mut INT32,    /* => table for Cr to G conversion */
    Cb_g_tab: *mut INT32,    /* => table for Cb to G conversion */
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

const SCALEBITS: u32 = 16;	/* speediest right-shift on some machines */
const ONE_HALF: INT32 = (1 as INT32) << (SCALEBITS - 1);

/* FIX(x): (INT32)((x) * (1L<<SCALEBITS) + 0.5) — locally defined macro translated as inline fn */
#[inline]
fn FIX(x: f64) -> INT32 {
    ((x) * (1i64 << SCALEBITS) as f64 + 0.5) as INT32
}


/*
 * Initialize tables for YCC->RGB colorspace conversion.
 */

/* LOCAL void -> plain unsafe fn (file-local static function in C) */
unsafe fn build_ycc_rgb_table(cinfo: j_decompress_ptr) {
    let cconvert: my_cconvert_ptr = (*cinfo).cconvert as my_cconvert_ptr;
    let mut i: c_int;
    let mut x: INT32;
    /* SHIFT_TEMPS: C macro that declares temp var for shift; omitted — expands to nothing
     * when RIGHT_SHIFT_IS_UNSIGNED is not defined (the common path). */

    (*cconvert).Cr_r_tab = ((*(*cinfo).mem).alloc_small.unwrap())
        (cinfo as j_common_ptr, JPOOL_IMAGE,
         (MAXJSAMPLE as usize + 1) * size_of::<c_int>()) as *mut c_int;
    /* SIZEOF(int) translated as size_of::<c_int>() — C sizeof macro inlined */
    (*cconvert).Cb_b_tab = ((*(*cinfo).mem).alloc_small.unwrap())
        (cinfo as j_common_ptr, JPOOL_IMAGE,
         (MAXJSAMPLE as usize + 1) * size_of::<c_int>()) as *mut c_int;
    (*cconvert).Cr_g_tab = ((*(*cinfo).mem).alloc_small.unwrap())
        (cinfo as j_common_ptr, JPOOL_IMAGE,
         (MAXJSAMPLE as usize + 1) * size_of::<INT32>()) as *mut INT32;
    (*cconvert).Cb_g_tab = ((*(*cinfo).mem).alloc_small.unwrap())
        (cinfo as j_common_ptr, JPOOL_IMAGE,
         (MAXJSAMPLE as usize + 1) * size_of::<INT32>()) as *mut INT32;

    i = 0;
    x = -(CENTERJSAMPLE as INT32);
    while i <= MAXJSAMPLE as c_int {
        /* i is the actual input pixel value, in the range 0..MAXJSAMPLE */
        /* The Cb or Cr value we are thinking of is x = i - CENTERJSAMPLE */
        /* Cr=>R value is nearest int to 1.40200 * x */
        *(*cconvert).Cr_r_tab.add(i as usize) =
            RIGHT_SHIFT(FIX(1.40200) * x + ONE_HALF, SCALEBITS) as c_int;
        /* Cb=>B value is nearest int to 1.77200 * x */
        *(*cconvert).Cb_b_tab.add(i as usize) =
            RIGHT_SHIFT(FIX(1.77200) * x + ONE_HALF, SCALEBITS) as c_int;
        /* Cr=>G value is scaled-up -0.71414 * x */
        *(*cconvert).Cr_g_tab.add(i as usize) = (- FIX(0.71414)) * x;
        /* Cb=>G value is scaled-up -0.34414 * x */
        /* We also add in ONE_HALF so that need not do it in inner loop */
        *(*cconvert).Cb_g_tab.add(i as usize) = (- FIX(0.34414)) * x + ONE_HALF;
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

/* METHODDEF void -> extern "C" fn (callback assigned through struct function pointer) */
extern "C" fn ycc_rgb_convert(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    input_row: JDIMENSION,
    output_buf: JSAMPARRAY,
    num_rows: c_int,
) {
    unsafe {
        let cconvert: my_cconvert_ptr = (*cinfo).cconvert as my_cconvert_ptr;
        let mut y: c_int;
        let mut cb: c_int;
        let mut cr: c_int;
        let mut outptr: JSAMPROW;
        let mut inptr0: JSAMPROW;
        let mut inptr1: JSAMPROW;
        let mut inptr2: JSAMPROW;
        let mut col: JDIMENSION;
        let num_cols: JDIMENSION = (*cinfo).output_width;
        /* copy these pointers into registers if possible */
        let range_limit: *mut JSAMPLE = (*cinfo).sample_range_limit;
        let Crrtab: *mut c_int = (*cconvert).Cr_r_tab;
        let Cbbtab: *mut c_int = (*cconvert).Cb_b_tab;
        let Crgtab: *mut INT32 = (*cconvert).Cr_g_tab;
        let Cbgtab: *mut INT32 = (*cconvert).Cb_g_tab;
        /* SHIFT_TEMPS omitted — see build_ycc_rgb_table note */

        let mut num_rows = num_rows;
        let mut input_row = input_row;
        let mut output_buf = output_buf;

        while {
            num_rows -= 1;
            num_rows >= 0
        } {
            inptr0 = *(*input_buf.add(0)).add(input_row as usize);
            inptr1 = *(*input_buf.add(1)).add(input_row as usize);
            inptr2 = *(*input_buf.add(2)).add(input_row as usize);
            input_row += 1;
            outptr = *output_buf;
            output_buf = output_buf.add(1);
            col = 0;
            while col < num_cols {
                y  = GETJSAMPLE(*inptr0.add(col as usize));
                cb = GETJSAMPLE(*inptr1.add(col as usize));
                cr = GETJSAMPLE(*inptr2.add(col as usize));
                /* Range-limiting is essential due to noise introduced by DCT losses. */
                *outptr.add(RGB_RED as usize) =
                    *range_limit.add((y + *Crrtab.add(cr as usize)) as usize);
                *outptr.add(RGB_GREEN as usize) = *range_limit.add(
                    (y + RIGHT_SHIFT(
                        *Cbgtab.add(cb as usize) + *Crgtab.add(cr as usize),
                        SCALEBITS,
                    ) as c_int) as usize);
                *outptr.add(RGB_BLUE as usize) =
                    *range_limit.add((y + *Cbbtab.add(cb as usize)) as usize);
                outptr = outptr.add(RGB_PIXELSIZE as usize);
                col += 1;
            }
        }
    }
}


/**************** Cases other than YCbCr -> RGB **************/


/*
 * Color conversion for no colorspace change: just copy the data,
 * converting from separate-planes to interleaved representation.
 */

/* METHODDEF void -> extern "C" fn */
extern "C" fn null_convert(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    input_row: JDIMENSION,
    output_buf: JSAMPARRAY,
    num_rows: c_int,
) {
    unsafe {
        let mut inptr: JSAMPROW;
        let mut outptr: JSAMPROW;
        let mut count: JDIMENSION;
        let num_components: c_int = (*cinfo).num_components;
        let num_cols: JDIMENSION = (*cinfo).output_width;
        let mut ci: c_int;

        let mut num_rows = num_rows;
        let mut input_row = input_row;
        let mut output_buf = output_buf;

        while {
            num_rows -= 1;
            num_rows >= 0
        } {
            ci = 0;
            while ci < num_components {
                inptr = *(*input_buf.add(ci as usize)).add(input_row as usize);
                outptr = (*output_buf).add(ci as usize);
                count = num_cols;
                while count > 0 {
                    *outptr = *inptr;	/* needn't bother with GETJSAMPLE() here */
                    inptr = inptr.add(1);
                    outptr = outptr.add(num_components as usize);
                    count -= 1;
                }
                ci += 1;
            }
            input_row += 1;
            output_buf = output_buf.add(1);
        }
    }
}


/*
 * Color conversion for grayscale: just copy the data.
 * This also works for YCbCr -> grayscale conversion, in which
 * we just copy the Y (luminance) component and ignore chrominance.
 */

/* METHODDEF void -> extern "C" fn */
extern "C" fn grayscale_convert(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    input_row: JDIMENSION,
    output_buf: JSAMPARRAY,
    num_rows: c_int,
) {
    unsafe {
        jcopy_sample_rows(
            *input_buf,
            input_row as c_int,
            output_buf,
            0,
            num_rows,
            (*cinfo).output_width,
        );
    }
}


/*
 * Adobe-style YCCK->CMYK conversion.
 * We convert YCbCr to R=1-C, G=1-M, and B=1-Y using the same
 * conversion as above, while passing K (black) unchanged.
 * We assume build_ycc_rgb_table has been called.
 */

/* METHODDEF void -> extern "C" fn */
extern "C" fn ycck_cmyk_convert(
    cinfo: j_decompress_ptr,
    input_buf: JSAMPIMAGE,
    input_row: JDIMENSION,
    output_buf: JSAMPARRAY,
    num_rows: c_int,
) {
    unsafe {
        let cconvert: my_cconvert_ptr = (*cinfo).cconvert as my_cconvert_ptr;
        let mut y: c_int;
        let mut cb: c_int;
        let mut cr: c_int;
        let mut outptr: JSAMPROW;
        let mut inptr0: JSAMPROW;
        let mut inptr1: JSAMPROW;
        let mut inptr2: JSAMPROW;
        let mut inptr3: JSAMPROW;
        let mut col: JDIMENSION;
        let num_cols: JDIMENSION = (*cinfo).output_width;
        /* copy these pointers into registers if possible */
        let range_limit: *mut JSAMPLE = (*cinfo).sample_range_limit;
        let Crrtab: *mut c_int = (*cconvert).Cr_r_tab;
        let Cbbtab: *mut c_int = (*cconvert).Cb_b_tab;
        let Crgtab: *mut INT32 = (*cconvert).Cr_g_tab;
        let Cbgtab: *mut INT32 = (*cconvert).Cb_g_tab;
        /* SHIFT_TEMPS omitted — see build_ycc_rgb_table note */

        let mut num_rows = num_rows;
        let mut input_row = input_row;
        let mut output_buf = output_buf;

        while {
            num_rows -= 1;
            num_rows >= 0
        } {
            inptr0 = *(*input_buf.add(0)).add(input_row as usize);
            inptr1 = *(*input_buf.add(1)).add(input_row as usize);
            inptr2 = *(*input_buf.add(2)).add(input_row as usize);
            inptr3 = *(*input_buf.add(3)).add(input_row as usize);
            input_row += 1;
            outptr = *output_buf;
            output_buf = output_buf.add(1);
            col = 0;
            while col < num_cols {
                y  = GETJSAMPLE(*inptr0.add(col as usize));
                cb = GETJSAMPLE(*inptr1.add(col as usize));
                cr = GETJSAMPLE(*inptr2.add(col as usize));
                /* Range-limiting is essential due to noise introduced by DCT losses. */
                *outptr.add(0) = *range_limit.add(
                    (MAXJSAMPLE as c_int - (y + *Crrtab.add(cr as usize))) as usize);	/* red */
                *outptr.add(1) = *range_limit.add(
                    (MAXJSAMPLE as c_int - (y +			/* green */
                        RIGHT_SHIFT(
                            *Cbgtab.add(cb as usize) + *Crgtab.add(cr as usize),
                            SCALEBITS,
                        ) as c_int)) as usize);
                *outptr.add(2) = *range_limit.add(
                    (MAXJSAMPLE as c_int - (y + *Cbbtab.add(cb as usize))) as usize);	/* blue */
                /* K passes through unchanged */
                *outptr.add(3) = *inptr3.add(col as usize);	/* don't need GETJSAMPLE here */
                outptr = outptr.add(4);
                col += 1;
            }
        }
    }
}


/*
 * Empty method for start_pass.
 */

/* METHODDEF void -> extern "C" fn */
extern "C" fn start_pass_dcolor(_cinfo: j_decompress_ptr) {
    /* no work needed */
}


/*
 * Module initialization routine for output colorspace conversion.
 */

/* GLOBAL void -> pub unsafe fn */
pub unsafe fn jinit_color_deconverter(cinfo: j_decompress_ptr) {
    let cconvert: my_cconvert_ptr;
    let mut ci: c_int;

    /* SIZEOF(my_color_deconverter) translated as size_of::<my_color_deconverter>() */
    cconvert = ((*(*cinfo).mem).alloc_small.unwrap())
        (cinfo as j_common_ptr, JPOOL_IMAGE,
         size_of::<my_color_deconverter>()) as my_cconvert_ptr;
    (*cinfo).cconvert = cconvert as *mut jpeg_color_deconverter;
    (*cconvert).pub_.start_pass = Some(start_pass_dcolor);

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

        _ => {			/* JCS_UNKNOWN can be anything */
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
            if (*cinfo).jpeg_color_space == JCS_GRAYSCALE ||
                (*cinfo).jpeg_color_space == JCS_YCbCr {
                (*cconvert).pub_.color_convert = Some(grayscale_convert);
                /* For color->grayscale conversion, only the Y (0) component is needed */
                ci = 1;
                while ci < (*cinfo).num_components {
                    (*(*cinfo).comp_info.add(ci as usize)).component_needed = FALSE;
                    ci += 1;
                }
            } else {
                ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
            }
        }

        JCS_RGB => {
            (*cinfo).out_color_components = RGB_PIXELSIZE as c_int;
            if (*cinfo).jpeg_color_space == JCS_YCbCr {
                (*cconvert).pub_.color_convert = Some(ycc_rgb_convert);
                build_ycc_rgb_table(cinfo);
            } else if (*cinfo).jpeg_color_space == JCS_RGB && RGB_PIXELSIZE == 3 {
                (*cconvert).pub_.color_convert = Some(null_convert);
            } else {
                ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
            }
        }

        JCS_CMYK => {
            (*cinfo).out_color_components = 4;
            if (*cinfo).jpeg_color_space == JCS_YCCK {
                (*cconvert).pub_.color_convert = Some(ycck_cmyk_convert);
                build_ycc_rgb_table(cinfo);
            } else if (*cinfo).jpeg_color_space == JCS_CMYK {
                (*cconvert).pub_.color_convert = Some(null_convert);
            } else {
                ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
            }
        }

        _ => {
            /* Permit null conversion to same output space */
            if (*cinfo).out_color_space == (*cinfo).jpeg_color_space {
                (*cinfo).out_color_components = (*cinfo).num_components;
                (*cconvert).pub_.color_convert = Some(null_convert);
            } else {		/* unsupported non-null conversion */
                ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
            }
        }
    }

    if (*cinfo).quantize_colors != 0 {
        (*cinfo).output_components = 1; /* single colormapped output component */
    } else {
        (*cinfo).output_components = (*cinfo).out_color_components;
    }
}
