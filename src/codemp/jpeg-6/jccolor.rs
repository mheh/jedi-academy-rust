/*
 * jccolor.c
 *
 * Copyright (C) 1991-1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains input colorspace conversion routines.
 */

#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::c_int;

// ============================================================================
// Stubs for JPEG-6 types and structures needed for structural coherence
// ============================================================================

pub type INT32 = i32;
pub type JSAMPLE = u8;
pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type JDIMENSION = core::ffi::c_uint;
pub type boolean = u8;

// Forward declaration for jpeg structs
#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut core::ffi::c_void, c_int, usize) -> *mut core::ffi::c_void>,
}

#[repr(C)]
pub struct jpeg_color_converter {
    pub start_pass: Option<unsafe extern "C" fn(*mut j_compress_struct)>,
    pub color_convert: Option<unsafe extern "C" fn(*mut j_compress_struct, JSAMPARRAY, JSAMPIMAGE, JDIMENSION, c_int)>,
}

#[repr(C)]
pub struct j_compress_struct {
    pub mem: *mut jpeg_memory_mgr,
    pub cconvert: *mut jpeg_color_converter,
    pub image_width: JDIMENSION,
    pub num_components: c_int,
    pub input_components: c_int,
    pub in_color_space: c_int,
    pub jpeg_color_space: c_int,
}

pub type j_compress_ptr = *mut j_compress_struct;
pub type j_common_ptr = *mut core::ffi::c_void;

/* Private subobject */

#[repr(C)]
pub struct my_color_converter {
    pub pub_: jpeg_color_converter,  /* public fields */

    /* Private state for RGB->YCC conversion */
    pub rgb_ycc_tab: *mut INT32,  /* => table for RGB to YCbCr conversion */
}

pub type my_cconvert_ptr = *mut my_color_converter;


/*/
// /**************** RGB -> YCbCr conversion: most common case **************/

/*
 * YCbCr is defined per CCIR 601-1, except that Cb and Cr are
 * normalized to the range 0..MAXJSAMPLE rather than -0.5 .. 0.5.
 * The conversion equations to be implemented are therefore
 *	Y  =  0.29900 * R + 0.58700 * G + 0.11400 * B
 *	Cb = -0.16874 * R - 0.33126 * G + 0.50000 * B  + CENTERJSAMPLE
 *	Cr =  0.50000 * R - 0.41869 * G - 0.08131 * B  + CENTERJSAMPLE
 * (These numbers are derived from TIFF 6.0 section 21, dated 3-June-92.)
 * Note: older versions of the IJG code used a zero offset of MAXJSAMPLE/2,
 * rather than CENTERJSAMPLE, for Cb and Cr.  This gave equal positive and
 * negative swings for Cb/Cr, but meant that grayscale values (Cb=Cr=0)
 * were not represented exactly.  Now we sacrifice exact representation of
 * maximum red and maximum blue in order to get exact grayscales.
 *
 * To avoid floating-point arithmetic, we represent the fractional constants
 * as integers scaled up by 2^16 (about 4 digits precision); we have to divide
 * the products by 2^16, with appropriate rounding, to get the correct answer.
 *
 * For even more speed, we avoid doing any multiplications in the inner loop
 * by precalculating the constants times R,G,B for all possible values.
 * For 8-bit JSAMPLEs this is very reasonable (only 256 entries per table);
 * for 12-bit samples it is still acceptable.  It's not very reasonable for
 * 16-bit samples, but if you want lossless storage you shouldn't be changing
 * colorspace anyway.
 * The CENTERJSAMPLE offsets and the rounding fudge-factor of 0.5 are included
 * in the tables to save adding them separately in the inner loop.
 */

const SCALEBITS: INT32 = 16;  /* speediest right-shift on some machines */
const MAXJSAMPLE: INT32 = 255;
const CENTERJSAMPLE: INT32 = 128;
const CBCR_OFFSET: INT32 = ((CENTERJSAMPLE as i64) << SCALEBITS) as INT32;  /* CENTERJSAMPLE << SCALEBITS */
const ONE_HALF: INT32 = 1 << (SCALEBITS - 1);

/* Macro-like function for FIX conversion */
#[inline(always)]
const fn FIX(x: f64) -> INT32 {
    ((x * ((1i64 << SCALEBITS) as f64) + 0.5) as INT32)
}

/* We allocate one big table and divide it up into eight parts, instead of
 * doing eight alloc_small requests.  This lets us use a single table base
 * address, which can be held in a register in the inner loops on many
 * machines (more than can hold all eight addresses, anyway).
 */

const R_Y_OFF: usize = 0;              /* offset to R => Y section */
const G_Y_OFF: usize = 1 * (MAXJSAMPLE as usize + 1);  /* offset to G => Y section */
const B_Y_OFF: usize = 2 * (MAXJSAMPLE as usize + 1);  /* etc. */
const R_CB_OFF: usize = 3 * (MAXJSAMPLE as usize + 1);
const G_CB_OFF: usize = 4 * (MAXJSAMPLE as usize + 1);
const B_CB_OFF: usize = 5 * (MAXJSAMPLE as usize + 1);
const R_CR_OFF: usize = B_CB_OFF;  /* B=>Cb, R=>Cr are the same */
const G_CR_OFF: usize = 6 * (MAXJSAMPLE as usize + 1);
const B_CR_OFF: usize = 7 * (MAXJSAMPLE as usize + 1);
const TABLE_SIZE: usize = 8 * (MAXJSAMPLE as usize + 1);


/*
 * Initialize for RGB->YCC colorspace conversion.
 */

#[no_mangle]
pub unsafe extern "C" fn rgb_ycc_start(cinfo: j_compress_ptr) {
    let cconvert: my_cconvert_ptr = (*cinfo).cconvert as my_cconvert_ptr;
    let mut rgb_ycc_tab: *mut INT32;

    /* Allocate and fill in the conversion tables. */
    rgb_ycc_tab = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        0,  /* JPOOL_IMAGE */
        TABLE_SIZE * core::mem::size_of::<INT32>(),
    ) as *mut INT32;
    (*cconvert).rgb_ycc_tab = rgb_ycc_tab;

    let mut i: INT32 = 0;
    while i <= MAXJSAMPLE {
        let idx = i as usize;
        *rgb_ycc_tab.add(idx + R_Y_OFF) = FIX(0.29900) * i;
        *rgb_ycc_tab.add(idx + G_Y_OFF) = FIX(0.58700) * i;
        *rgb_ycc_tab.add(idx + B_Y_OFF) = FIX(0.11400) * i + ONE_HALF;
        *rgb_ycc_tab.add(idx + R_CB_OFF) = (-FIX(0.16874)) * i;
        *rgb_ycc_tab.add(idx + G_CB_OFF) = (-FIX(0.33126)) * i;
        /* We use a rounding fudge-factor of 0.5-epsilon for Cb and Cr.
         * This ensures that the maximum output will round to MAXJSAMPLE
         * not MAXJSAMPLE+1, and thus that we don't have to range-limit.
         */
        *rgb_ycc_tab.add(idx + B_CB_OFF) = FIX(0.50000) * i + CBCR_OFFSET + ONE_HALF - 1;
        /*  B=>Cb and R=>Cr tables are the same
            rgb_ycc_tab[i+R_CR_OFF] = FIX(0.50000) * i    + CBCR_OFFSET + ONE_HALF-1;
        */
        *rgb_ycc_tab.add(idx + G_CR_OFF) = (-FIX(0.41869)) * i;
        *rgb_ycc_tab.add(idx + B_CR_OFF) = (-FIX(0.08131)) * i;
        i += 1;
    }
}


/*
 * Convert some rows of samples to the JPEG colorspace.
 *
 * Note that we change from the application's interleaved-pixel format
 * to our internal noninterleaved, one-plane-per-component format.
 * The input buffer is therefore three times as wide as the output buffer.
 *
 * A starting row offset is provided only for the output buffer.  The caller
 * can easily adjust the passed input_buf value to accommodate any row
 * offset required on that side.
 */

#[no_mangle]
pub unsafe extern "C" fn rgb_ycc_convert(
    cinfo: j_compress_ptr,
    mut input_buf: JSAMPARRAY,
    output_buf: JSAMPIMAGE,
    mut output_row: JDIMENSION,
    mut num_rows: c_int,
) {
    let cconvert: my_cconvert_ptr = (*cinfo).cconvert as my_cconvert_ptr;
    let mut r: c_int;
    let mut g: c_int;
    let mut b: c_int;
    let ctab: *mut INT32 = (*cconvert).rgb_ycc_tab;
    let mut inptr: JSAMPROW;
    let mut outptr0: JSAMPROW;
    let mut outptr1: JSAMPROW;
    let mut outptr2: JSAMPROW;
    let mut col: JDIMENSION;
    let num_cols: JDIMENSION = (*cinfo).image_width;

    const RGB_RED: usize = 0;
    const RGB_GREEN: usize = 1;
    const RGB_BLUE: usize = 2;
    const RGB_PIXELSIZE: usize = 3;

    while num_rows >= 1 {
        num_rows -= 1;
        inptr = *input_buf;
        input_buf = input_buf.add(1);
        outptr0 = *(*output_buf).add(0).add(output_row as usize);
        outptr1 = *(*output_buf).add(1).add(output_row as usize);
        outptr2 = *(*output_buf).add(2).add(output_row as usize);
        output_row += 1;
        col = 0;
        while col < num_cols {
            r = *inptr.add(RGB_RED) as c_int;
            g = *inptr.add(RGB_GREEN) as c_int;
            b = *inptr.add(RGB_BLUE) as c_int;
            inptr = inptr.add(RGB_PIXELSIZE);
            /* If the inputs are 0..MAXJSAMPLE, the outputs of these equations
             * must be too; we do not need an explicit range-limiting operation.
             * Hence the value being shifted is never negative, and we don't
             * need the general RIGHT_SHIFT macro.
             */
            /* Y */
            *outptr0.add(col as usize) = ((*ctab.add(r as usize + R_Y_OFF)
                + *ctab.add(g as usize + G_Y_OFF)
                + *ctab.add(b as usize + B_Y_OFF))
                >> SCALEBITS) as JSAMPLE;
            /* Cb */
            *outptr1.add(col as usize) = ((*ctab.add(r as usize + R_CB_OFF)
                + *ctab.add(g as usize + G_CB_OFF)
                + *ctab.add(b as usize + B_CB_OFF))
                >> SCALEBITS) as JSAMPLE;
            /* Cr */
            *outptr2.add(col as usize) = ((*ctab.add(r as usize + R_CR_OFF)
                + *ctab.add(g as usize + G_CR_OFF)
                + *ctab.add(b as usize + B_CR_OFF))
                >> SCALEBITS) as JSAMPLE;
            col += 1;
        }
    }
}


/*/
// /**************** Cases other than RGB -> YCbCr **************/


/*
 * Convert some rows of samples to the JPEG colorspace.
 * This version handles RGB->grayscale conversion, which is the same
 * as the RGB->Y portion of RGB->YCbCr.
 * We assume rgb_ycc_start has been called (we only use the Y tables).
 */

#[no_mangle]
pub unsafe extern "C" fn rgb_gray_convert(
    cinfo: j_compress_ptr,
    mut input_buf: JSAMPARRAY,
    output_buf: JSAMPIMAGE,
    mut output_row: JDIMENSION,
    mut num_rows: c_int,
) {
    let cconvert: my_cconvert_ptr = (*cinfo).cconvert as my_cconvert_ptr;
    let mut r: c_int;
    let mut g: c_int;
    let mut b: c_int;
    let ctab: *mut INT32 = (*cconvert).rgb_ycc_tab;
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut col: JDIMENSION;
    let num_cols: JDIMENSION = (*cinfo).image_width;

    const RGB_RED: usize = 0;
    const RGB_GREEN: usize = 1;
    const RGB_BLUE: usize = 2;
    const RGB_PIXELSIZE: usize = 3;

    while num_rows >= 1 {
        num_rows -= 1;
        inptr = *input_buf;
        input_buf = input_buf.add(1);
        outptr = *(*output_buf).add(0).add(output_row as usize);
        output_row += 1;
        col = 0;
        while col < num_cols {
            r = *inptr.add(RGB_RED) as c_int;
            g = *inptr.add(RGB_GREEN) as c_int;
            b = *inptr.add(RGB_BLUE) as c_int;
            inptr = inptr.add(RGB_PIXELSIZE);
            /* Y */
            *outptr.add(col as usize) = ((*ctab.add(r as usize + R_Y_OFF)
                + *ctab.add(g as usize + G_Y_OFF)
                + *ctab.add(b as usize + B_Y_OFF))
                >> SCALEBITS) as JSAMPLE;
            col += 1;
        }
    }
}


/*
 * Convert some rows of samples to the JPEG colorspace.
 * This version handles Adobe-style CMYK->YCCK conversion,
 * where we convert R=1-C, G=1-M, and B=1-Y to YCbCr using the same
 * conversion as above, while passing K (black) unchanged.
 * We assume rgb_ycc_start has been called.
 */

#[no_mangle]
pub unsafe extern "C" fn cmyk_ycck_convert(
    cinfo: j_compress_ptr,
    mut input_buf: JSAMPARRAY,
    output_buf: JSAMPIMAGE,
    mut output_row: JDIMENSION,
    mut num_rows: c_int,
) {
    let cconvert: my_cconvert_ptr = (*cinfo).cconvert as my_cconvert_ptr;
    let mut r: c_int;
    let mut g: c_int;
    let mut b: c_int;
    let ctab: *mut INT32 = (*cconvert).rgb_ycc_tab;
    let mut inptr: JSAMPROW;
    let mut outptr0: JSAMPROW;
    let mut outptr1: JSAMPROW;
    let mut outptr2: JSAMPROW;
    let mut outptr3: JSAMPROW;
    let mut col: JDIMENSION;
    let num_cols: JDIMENSION = (*cinfo).image_width;

    while num_rows >= 1 {
        num_rows -= 1;
        inptr = *input_buf;
        input_buf = input_buf.add(1);
        outptr0 = *(*output_buf).add(0).add(output_row as usize);
        outptr1 = *(*output_buf).add(1).add(output_row as usize);
        outptr2 = *(*output_buf).add(2).add(output_row as usize);
        outptr3 = *(*output_buf).add(3).add(output_row as usize);
        output_row += 1;
        col = 0;
        while col < num_cols {
            r = (MAXJSAMPLE - *inptr.add(0) as INT32) as c_int;
            g = (MAXJSAMPLE - *inptr.add(1) as INT32) as c_int;
            b = (MAXJSAMPLE - *inptr.add(2) as INT32) as c_int;
            /* K passes through as-is */
            *outptr3.add(col as usize) = *inptr.add(3);  /* don't need GETJSAMPLE here */
            inptr = inptr.add(4);
            /* If the inputs are 0..MAXJSAMPLE, the outputs of these equations
             * must be too; we do not need an explicit range-limiting operation.
             * Hence the value being shifted is never negative, and we don't
             * need the general RIGHT_SHIFT macro.
             */
            /* Y */
            *outptr0.add(col as usize) = ((*ctab.add(r as usize + R_Y_OFF)
                + *ctab.add(g as usize + G_Y_OFF)
                + *ctab.add(b as usize + B_Y_OFF))
                >> SCALEBITS) as JSAMPLE;
            /* Cb */
            *outptr1.add(col as usize) = ((*ctab.add(r as usize + R_CB_OFF)
                + *ctab.add(g as usize + G_CB_OFF)
                + *ctab.add(b as usize + B_CB_OFF))
                >> SCALEBITS) as JSAMPLE;
            /* Cr */
            *outptr2.add(col as usize) = ((*ctab.add(r as usize + R_CR_OFF)
                + *ctab.add(g as usize + G_CR_OFF)
                + *ctab.add(b as usize + B_CR_OFF))
                >> SCALEBITS) as JSAMPLE;
            col += 1;
        }
    }
}


/*
 * Convert some rows of samples to the JPEG colorspace.
 * This version handles grayscale output with no conversion.
 * The source can be either plain grayscale or YCbCr (since Y == gray).
 */

#[no_mangle]
pub unsafe extern "C" fn grayscale_convert(
    cinfo: j_compress_ptr,
    mut input_buf: JSAMPARRAY,
    output_buf: JSAMPIMAGE,
    mut output_row: JDIMENSION,
    mut num_rows: c_int,
) {
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut col: JDIMENSION;
    let num_cols: JDIMENSION = (*cinfo).image_width;
    let instride: c_int = (*cinfo).input_components;

    while num_rows >= 1 {
        num_rows -= 1;
        inptr = *input_buf;
        input_buf = input_buf.add(1);
        outptr = *(*output_buf).add(0).add(output_row as usize);
        output_row += 1;
        col = 0;
        while col < num_cols {
            *outptr.add(col as usize) = *inptr.add(0);  /* don't need GETJSAMPLE() here */
            inptr = inptr.add(instride as usize);
            col += 1;
        }
    }
}


/*
 * Convert some rows of samples to the JPEG colorspace.
 * This version handles multi-component colorspaces without conversion.
 * We assume input_components == num_components.
 */

#[no_mangle]
pub unsafe extern "C" fn null_convert(
    cinfo: j_compress_ptr,
    mut input_buf: JSAMPARRAY,
    output_buf: JSAMPIMAGE,
    mut output_row: JDIMENSION,
    mut num_rows: c_int,
) {
    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut col: JDIMENSION;
    let mut ci: c_int;
    let nc: c_int = (*cinfo).num_components;
    let num_cols: JDIMENSION = (*cinfo).image_width;

    while num_rows >= 1 {
        num_rows -= 1;
        /* It seems fastest to make a separate pass for each component. */
        ci = 0;
        while ci < nc {
            inptr = *input_buf;
            outptr = *(*output_buf).add(ci as usize).add(output_row as usize);
            col = 0;
            while col < num_cols {
                *outptr.add(col as usize) = *inptr.add(ci as usize);  /* don't need GETJSAMPLE() here */
                inptr = inptr.add(nc as usize);
                col += 1;
            }
            ci += 1;
        }
        input_buf = input_buf.add(1);
        output_row += 1;
    }
}


/*
 * Empty method for start_pass.
 */

#[no_mangle]
pub unsafe extern "C" fn null_method(cinfo: j_compress_ptr) {
    /* no work needed */
}


/*
 * Module initialization routine for input colorspace conversion.
 */

#[no_mangle]
pub unsafe extern "C" fn jinit_color_converter(cinfo: j_compress_ptr) {
    let mut cconvert: my_cconvert_ptr;

    cconvert = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        0,  /* JPOOL_IMAGE */
        core::mem::size_of::<my_color_converter>(),
    ) as my_cconvert_ptr;
    (*cinfo).cconvert = cconvert as *mut jpeg_color_converter;
    /* set start_pass to null method until we find out differently */
    (*cconvert).pub_.start_pass = Some(null_method);

    // Constants for color space codes
    const JCS_GRAYSCALE: c_int = 1;
    const JCS_RGB: c_int = 2;
    const JCS_YCbCr: c_int = 3;
    const JCS_CMYK: c_int = 4;
    const JCS_YCCK: c_int = 5;
    const JCS_UNKNOWN: c_int = 0;
    const RGB_PIXELSIZE: c_int = 3;
    const JERR_BAD_IN_COLORSPACE: c_int = 1;
    const JERR_BAD_J_COLORSPACE: c_int = 2;
    const JERR_CONVERSION_NOTIMPL: c_int = 3;

    /* Make sure input_components agrees with in_color_space */
    match (*cinfo).in_color_space {
        JCS_GRAYSCALE => {
            if (*cinfo).input_components != 1 {
                // ERREXIT(cinfo, JERR_BAD_IN_COLORSPACE);
                return;
            }
        }

        JCS_RGB => {
            if RGB_PIXELSIZE != 3 {
                if (*cinfo).input_components != RGB_PIXELSIZE {
                    // ERREXIT(cinfo, JERR_BAD_IN_COLORSPACE);
                    return;
                }
            }
            /* else share code with YCbCr */
        }

        JCS_YCbCr => {
            if (*cinfo).input_components != 3 {
                // ERREXIT(cinfo, JERR_BAD_IN_COLORSPACE);
                return;
            }
        }

        JCS_CMYK | JCS_YCCK => {
            if (*cinfo).input_components != 4 {
                // ERREXIT(cinfo, JERR_BAD_IN_COLORSPACE);
                return;
            }
        }

        _ => {
            /* JCS_UNKNOWN can be anything */
            if (*cinfo).input_components < 1 {
                // ERREXIT(cinfo, JERR_BAD_IN_COLORSPACE);
                return;
            }
        }
    }

    /* Check num_components, set conversion method based on requested space */
    match (*cinfo).jpeg_color_space {
        JCS_GRAYSCALE => {
            if (*cinfo).num_components != 1 {
                // ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
                return;
            }
            if (*cinfo).in_color_space == JCS_GRAYSCALE {
                (*cconvert).pub_.color_convert = Some(grayscale_convert);
            } else if (*cinfo).in_color_space == JCS_RGB {
                (*cconvert).pub_.start_pass = Some(rgb_ycc_start);
                (*cconvert).pub_.color_convert = Some(rgb_gray_convert);
            } else if (*cinfo).in_color_space == JCS_YCbCr {
                (*cconvert).pub_.color_convert = Some(grayscale_convert);
            } else {
                // ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
                return;
            }
        }

        JCS_RGB => {
            if (*cinfo).num_components != 3 {
                // ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
                return;
            }
            if (*cinfo).in_color_space == JCS_RGB && RGB_PIXELSIZE == 3 {
                (*cconvert).pub_.color_convert = Some(null_convert);
            } else {
                // ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
                return;
            }
        }

        JCS_YCbCr => {
            if (*cinfo).num_components != 3 {
                // ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
                return;
            }
            if (*cinfo).in_color_space == JCS_RGB {
                (*cconvert).pub_.start_pass = Some(rgb_ycc_start);
                (*cconvert).pub_.color_convert = Some(rgb_ycc_convert);
            } else if (*cinfo).in_color_space == JCS_YCbCr {
                (*cconvert).pub_.color_convert = Some(null_convert);
            } else {
                // ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
                return;
            }
        }

        JCS_CMYK => {
            if (*cinfo).num_components != 4 {
                // ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
                return;
            }
            if (*cinfo).in_color_space == JCS_CMYK {
                (*cconvert).pub_.color_convert = Some(null_convert);
            } else {
                // ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
                return;
            }
        }

        JCS_YCCK => {
            if (*cinfo).num_components != 4 {
                // ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
                return;
            }
            if (*cinfo).in_color_space == JCS_CMYK {
                (*cconvert).pub_.start_pass = Some(rgb_ycc_start);
                (*cconvert).pub_.color_convert = Some(cmyk_ycck_convert);
            } else if (*cinfo).in_color_space == JCS_YCCK {
                (*cconvert).pub_.color_convert = Some(null_convert);
            } else {
                // ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
                return;
            }
        }

        _ => {
            /* allow null conversion of JCS_UNKNOWN */
            if (*cinfo).jpeg_color_space != (*cinfo).in_color_space
                || (*cinfo).num_components != (*cinfo).input_components
            {
                // ERREXIT(cinfo, JERR_CONVERSION_NOTIMPL);
                return;
            }
            (*cconvert).pub_.color_convert = Some(null_convert);
        }
    }
}
