/*
 * jddctmgr.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains the inverse-DCT management logic.
 * This code selects a particular IDCT implementation to be used,
 * and it performs related housekeeping chores.  No code in this file
 * is executed per IDCT step, only during output pass setup.
 *
 * Note that the IDCT routines are responsible for performing coefficient
 * dequantization as well as the IDCT proper.  This module sets up the
 * dequantization multiplier table needed by the IDCT routine.
 */

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// ============================================================================
// Constants (from jdct.h, jpeglib.h, jmorecfg.h)
// ============================================================================
pub const MAX_COMPONENTS: usize = 10;
pub const DCTSIZE: usize = 8;
pub const DCTSIZE2: usize = 64;
pub const JPOOL_IMAGE: c_int = 0;
pub const CONST_BITS: c_int = 14;

// ============================================================================
// Type aliases for JPEG types
// ============================================================================
pub type DCTELEM = c_int;
pub type FAST_FLOAT = f32;
pub type JDIMENSION = u32;
pub type JCOEF = core::ffi::c_short;
pub type JCOEFPTR = *mut JCOEF;
pub type JSAMPROW = *mut u8;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JBLOCKROW = *mut JCOEF;

// Type stubs for JPEG library types
// Defined in jpeglib.h

#[repr(C)]
pub struct jpeg_decompress_struct {
    // Minimal field definitions needed by this module
    pub idct: *mut c_void,
    pub comp_info: *mut jpeg_component_info,
    pub num_components: c_int,
    pub dct_method: c_int,
    pub mem: *mut jpeg_memory_mgr,
}

pub type j_decompress_ptr = *mut jpeg_decompress_struct;

#[repr(C)]
pub struct jpeg_component_info {
    // Minimal field definitions needed by this module
    pub DCT_scaled_size: c_int,
    pub component_needed: c_int,
    pub quant_table: *mut JQUANT_TBL,
    pub dct_table: *mut c_void,
    // ... other fields omitted
    _unused: [u8; 0],
}

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; DCTSIZE2],
    // ... other fields omitted
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    // Opaque for this module
    _unused: [u8; 0],
}

// Function pointer type for IDCT methods
pub type inverse_DCT_method_ptr = Option<unsafe extern "C" fn(
    j_decompress_ptr,
    *mut jpeg_component_info,
    *mut JCOEF,
    *mut JSAMPARRAY,
    JDIMENSION,
) -> ()>;

#[repr(C)]
pub struct jpeg_inverse_dct {
    pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr) -> ()>,
    pub inverse_DCT: [inverse_DCT_method_ptr; MAX_COMPONENTS],
}

// External function declarations
extern "C" {
    pub fn jpeg_idct_1x1(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: *mut JCOEF,
        output_buf: *mut JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_2x2(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: *mut JCOEF,
        output_buf: *mut JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_4x4(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: *mut JCOEF,
        output_buf: *mut JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_islow(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: *mut JCOEF,
        output_buf: *mut JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_ifast(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: *mut JCOEF,
        output_buf: *mut JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_float(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: *mut JCOEF,
        output_buf: *mut JSAMPARRAY,
        output_col: JDIMENSION,
    );

    // Zigzag ordering table
    pub static jpeg_zigzag_order: [usize; DCTSIZE2];

    // Memory manager functions
    pub fn jpeg_alloc_small(cinfo: j_decompress_ptr, pool_id: c_int, size: usize) -> *mut c_void;

    // Error handling
    pub fn jpeg_errexit(cinfo: j_decompress_ptr, code: c_int) -> !;
    pub fn jpeg_errexit1(cinfo: j_decompress_ptr, code: c_int, p1: c_int) -> !;
}

// Macro stubs from jinclude.h
extern "C" {
    fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void;
}

#[inline]
unsafe fn MEMZERO(target: *mut c_void, size: usize) {
    memset(target, 0, size);
}

#[inline]
pub const fn SIZEOF<T>() -> usize {
    core::mem::size_of::<T>()
}

// ============================================================================
// Private subobject for this module
// ============================================================================

/*
 * The decompressor input side (jdinput.c) saves away the appropriate
 * quantization table for each component at the start of the first scan
 * involving that component.  (This is necessary in order to correctly
 * decode files that reuse Q-table slots.)
 * When we are ready to make an output pass, the saved Q-table is converted
 * to a multiplier table that will actually be used by the IDCT routine.
 * The multiplier table contents are IDCT-method-dependent.  To support
 * application changes in IDCT method between scans, we can remake the
 * multiplier tables if necessary.
 * In buffered-image mode, the first output pass may occur before any data
 * has been seen for some components, and thus before their Q-tables have
 * been saved away.  To handle this case, multiplier tables are preset
 * to zeroes; the result of the IDCT will be a neutral gray level.
 */

/* Private subobject for this module */

#[repr(C)]
pub struct my_idct_controller {
    pub pub_: jpeg_inverse_dct,    /* public fields */

    /* This array contains the IDCT method code that each multiplier table
     * is currently set up for, or -1 if it's not yet set up.
     * The actual multiplier tables are pointed to by dct_table in the
     * per-component comp_info structures.
     */
    pub cur_method: [c_int; MAX_COMPONENTS],
}

pub type my_idct_ptr = *mut my_idct_controller;


/* Allocated multiplier tables: big enough for any supported variant */

#[repr(C)]
pub union multiplier_table {
    pub islow_array: [ISLOW_MULT_TYPE; DCTSIZE2],
    pub ifast_array: [IFAST_MULT_TYPE; DCTSIZE2],
    pub float_array: [FLOAT_MULT_TYPE; DCTSIZE2],
}

// Type definitions for multiplier types
pub type ISLOW_MULT_TYPE = c_int;
pub type IFAST_MULT_TYPE = c_int;
pub type FLOAT_MULT_TYPE = f32;

pub const IFAST_SCALE_BITS: c_int = 2;

/* The current scaled-IDCT routines require ISLOW-style multiplier tables,
 * so be sure to compile that code if either ISLOW or SCALING is requested.
 */
// #define PROVIDE_ISLOW_TABLES is effectively always true for this build

// ============================================================================
// Function implementations
// ============================================================================

/*
 * Prepare for an output pass.
 * Here we select the proper IDCT routine for each component and build
 * a matching multiplier table.
 */

pub unsafe extern "C" fn start_pass(cinfo: j_decompress_ptr) {
    let mut idct = (*cinfo).idct as my_idct_ptr;
    let mut ci: c_int = 0;
    let mut i: c_int;
    let mut compptr = (*cinfo).comp_info;
    let mut method: c_int = 0;
    let mut method_ptr: inverse_DCT_method_ptr = None;
    let mut qtbl: *mut JQUANT_TBL;

    ci = 0;
    while ci < (*cinfo).num_components {
        /* Select the proper IDCT routine for this component's scaling */
        match (*compptr).DCT_scaled_size {
            #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
            1 => {
                method_ptr = Some(jpeg_idct_1x1);
                method = 0;    /* JDCT_ISLOW: jidctred uses islow-style table */
            }
            #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
            2 => {
                method_ptr = Some(jpeg_idct_2x2);
                method = 0;    /* JDCT_ISLOW: jidctred uses islow-style table */
            }
            #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
            4 => {
                method_ptr = Some(jpeg_idct_4x4);
                method = 0;    /* JDCT_ISLOW: jidctred uses islow-style table */
            }
            8 => {
                /* DCTSIZE case */
                match (*cinfo).dct_method {
                    0 => {
                        /* JDCT_ISLOW */
                        method_ptr = Some(jpeg_idct_islow);
                        method = 0; /* JDCT_ISLOW */
                    }
                    1 => {
                        /* JDCT_IFAST */
                        method_ptr = Some(jpeg_idct_ifast);
                        method = 1; /* JDCT_IFAST */
                    }
                    2 => {
                        /* JDCT_FLOAT */
                        method_ptr = Some(jpeg_idct_float);
                        method = 2; /* JDCT_FLOAT */
                    }
                    _ => {
                        jpeg_errexit(cinfo, 0); /* JERR_NOT_COMPILED */
                    }
                }
            }
            _ => {
                jpeg_errexit1(cinfo, 0, (*compptr).DCT_scaled_size); /* JERR_BAD_DCTSIZE */
            }
        }
        (*idct).pub_.inverse_DCT[ci as usize] = method_ptr;
        /* Create multiplier table from quant table.
         * However, we can skip this if the component is uninteresting
         * or if we already built the table.  Also, if no quant table
         * has yet been saved for the component, we leave the
         * multiplier table all-zero; we'll be reading zeroes from the
         * coefficient controller's buffer anyway.
         */
        if (*compptr).component_needed == 0 || (*idct).cur_method[ci as usize] == method {
            ci += 1;
            compptr = compptr.add(1);
            continue;
        }
        qtbl = (*compptr).quant_table;
        if qtbl.is_null() {
            /* happens if no data yet for component */
            ci += 1;
            compptr = compptr.add(1);
            continue;
        }
        (*idct).cur_method[ci as usize] = method;
        match method {
            0 => {
                /* JDCT_ISLOW */
                /* For LL&M IDCT method, multipliers are equal to raw quantization
                 * coefficients, but are stored in natural order as ints.
                 */
                let ismtbl = (*compptr).dct_table as *mut ISLOW_MULT_TYPE;
                i = 0;
                while i < DCTSIZE2 as c_int {
                    *ismtbl.add(i as usize) = (*qtbl).quantval[jpeg_zigzag_order[i as usize]] as ISLOW_MULT_TYPE;
                    i += 1;
                }
            }
            1 => {
                /* JDCT_IFAST */
                /* For AA&N IDCT method, multipliers are equal to quantization
                 * coefficients scaled by scalefactor[row]*scalefactor[col], where
                 *   scalefactor[0] = 1
                 *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                 * For integer operation, the multiplier table is to be scaled by
                 * IFAST_SCALE_BITS.  The multipliers are stored in natural order.
                 */
                let ifmtbl = (*compptr).dct_table as *mut IFAST_MULT_TYPE;
                static AANSCALES: [core::ffi::c_short; DCTSIZE2] = [
                    /* precomputed values scaled up by 14 bits */
                    16384, 22725, 21407, 19266, 16384, 12873,  8867,  4520,
                    22725, 31521, 29692, 26722, 22725, 17855, 12299,  6270,
                    21407, 29692, 27969, 25172, 21407, 16819, 11585,  5906,
                    19266, 26722, 25172, 22654, 19266, 15137, 10426,  5315,
                    16384, 22725, 21407, 19266, 16384, 12873,  8867,  4520,
                    12873, 17855, 16819, 15137, 12873, 10114,  6967,  3552,
                     8867, 12299, 11585, 10426,  8867,  6967,  4799,  2446,
                     4520,  6270,  5906,  5315,  4520,  3552,  2446,  1247
                ];

                i = 0;
                while i < DCTSIZE2 as c_int {
                    let qval = (*qtbl).quantval[jpeg_zigzag_order[i as usize]] as c_int;
                    let scale = AANSCALES[i as usize] as c_int;
                    // DESCALE(MULTIPLY16V16(qval, aanscales[i]), CONST_BITS-IFAST_SCALE_BITS)
                    let product = qval * scale;
                    let shift_amount = CONST_BITS - IFAST_SCALE_BITS;
                    let result = ((product + (1 << (shift_amount - 1))) >> shift_amount) as IFAST_MULT_TYPE;
                    *ifmtbl.add(i as usize) = result;
                    i += 1;
                }
            }
            2 => {
                /* JDCT_FLOAT */
                /* For float AA&N IDCT method, multipliers are equal to quantization
                 * coefficients scaled by scalefactor[row]*scalefactor[col], where
                 *   scalefactor[0] = 1
                 *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                 * The multipliers are stored in natural order.
                 */
                let fmtbl = (*compptr).dct_table as *mut FLOAT_MULT_TYPE;
                static AANSCALEFACTOR: [f64; DCTSIZE] = [
                    1.0, 1.387039845, 1.306562965, 1.175875602,
                    1.0, 0.785694958, 0.541196100, 0.275899379
                ];

                let mut idx = 0;
                let mut row = 0;
                while row < DCTSIZE {
                    let mut col = 0;
                    while col < DCTSIZE {
                        let zigzag_idx = jpeg_zigzag_order[idx];
                        let qval = (*qtbl).quantval[zigzag_idx];
                        *fmtbl.add(idx) = ((qval as f64) *
                                           AANSCALEFACTOR[row] *
                                           AANSCALEFACTOR[col]) as FLOAT_MULT_TYPE;
                        idx += 1;
                        col += 1;
                    }
                    row += 1;
                }
            }
            _ => {
                jpeg_errexit(cinfo, 0); /* JERR_NOT_COMPILED */
            }
        }
        ci += 1;
        compptr = compptr.add(1);
    }
}


/*
 * Initialize IDCT manager.
 */

pub unsafe extern "C" fn jinit_inverse_dct(cinfo: j_decompress_ptr) {
    let mut idct: my_idct_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    idct = jpeg_alloc_small(
        cinfo,
        JPOOL_IMAGE,
        SIZEOF::<my_idct_controller>(),
    ) as my_idct_ptr;
    (*cinfo).idct = idct as *mut c_void;
    (*idct).pub_.start_pass = Some(start_pass);

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Allocate and pre-zero a multiplier table for each component */
        (*compptr).dct_table = jpeg_alloc_small(
            cinfo,
            JPOOL_IMAGE,
            SIZEOF::<multiplier_table>(),
        );
        MEMZERO((*compptr).dct_table, SIZEOF::<multiplier_table>());
        /* Mark multiplier table not yet set up for any method */
        (*idct).cur_method[ci as usize] = -1;
        ci += 1;
        compptr = compptr.add(1);
    }
}
