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
// Anything above this include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #define JPEG_INTERNALS
// #include "jinclude.h"
// #include "jpeglib.h"
// #include "jdct.h"		/* Private declarations for DCT subsystem */

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_void};

// ============================================================================
// Type stubs for JPEG-6 structures needed for structural coherence
// ============================================================================

pub type JDIMENSION = u32;
pub type boolean = u8;
pub type JCOEF = i16;
pub type JCOEFPTR = *mut JCOEF;
pub type JSAMPLE = u8;
pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;

// DCT multiplier types (based on jmorecfg.h and jdct.h)
pub type MULTIPLIER = c_int;
pub type ISLOW_MULT_TYPE = MULTIPLIER;     /* short or int, whichever is faster */
pub type IFAST_MULT_TYPE = MULTIPLIER;     /* 16 bits is OK, use short if faster */
pub type FLOAT_MULT_TYPE = f32;            /* preferred floating type */
pub type INT16 = i16;
pub type INT32 = i32;
pub type FAST_FLOAT = f32;

const DCTSIZE: usize = 8;
const DCTSIZE2: usize = 64;
const MAX_COMPONENTS: usize = 10;
const IFAST_SCALE_BITS: c_int = 2;         /* fractional bits in scale factors */

// Forward declarations
#[repr(C)]
pub struct jpeg_inverse_dct {
    pub start_pass: Option<unsafe extern "C" fn(*mut j_decompress_struct)>,
    pub inverse_DCT: [Option<inverse_DCT_method_ptr>; MAX_COMPONENTS],
}

pub type inverse_DCT_method_ptr = unsafe extern "C" fn(
    cinfo: *mut j_decompress_struct,
    compptr: *mut jpeg_component_info,
    coef_block: JCOEFPTR,
    output_buf: JSAMPARRAY,
    output_col: JDIMENSION,
);

#[repr(C)]
pub struct jpeg_component_info {
    pub DCT_scaled_size: c_int,
    pub component_needed: boolean,
    pub quant_table: *mut JQUANT_TBL,
    pub dct_table: *mut c_void,
}

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; DCTSIZE2],
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
}

#[repr(C)]
pub struct j_decompress_struct {
    pub mem: *mut jpeg_memory_mgr,
    pub idct: *mut jpeg_inverse_dct,
    pub comp_info: *mut jpeg_component_info,
    pub num_components: c_int,
    pub dct_method: c_int,
}

pub type j_decompress_ptr = *mut j_decompress_struct;
pub type j_common_ptr = *mut j_decompress_struct;

// Constants
const JPOOL_IMAGE: c_int = 1;
const JDCT_ISLOW: c_int = 0;
const JDCT_IFAST: c_int = 1;
const JDCT_FLOAT: c_int = 2;
const JERR_NOT_COMPILED: c_int = 0;
const JERR_BAD_DCTSIZE: c_int = 1;

// External functions from JPEG library
extern "C" {
    pub static jpeg_zigzag_order: [c_int; DCTSIZE2];

    pub fn jpeg_idct_1x1(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_2x2(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_4x4(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_islow(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_ifast(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );

    pub fn jpeg_idct_float(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
}

// Stub error handling (ERREXIT and ERREXIT1 macro stubs)
#[inline]
unsafe fn ERREXIT(_cinfo: j_decompress_ptr, _code: c_int) {
    // Stub: error handling not fully ported
}

#[inline]
unsafe fn ERREXIT1(_cinfo: j_decompress_ptr, _code: c_int, _p1: c_int) {
    // Stub: error handling not fully ported
}

// MEMZERO macro equivalent
#[inline]
unsafe fn MEMZERO(target: *mut c_void, size: usize) {
    core::ptr::write_bytes(target as *mut u8, 0, size);
}

// SIZEOF macro equivalent
#[inline]
const fn SIZEOF<T>() -> usize {
    core::mem::size_of::<T>()
}

// Private subobject for this module

#[repr(C)]
struct my_idct_controller {
    pub pub_: jpeg_inverse_dct,  /* public fields */

    /* This array contains the IDCT method code that each multiplier table
     * is currently set up for, or -1 if it's not yet set up.
     * The actual multiplier tables are pointed to by dct_table in the
     * per-component comp_info structures.
     */
    pub cur_method: [c_int; MAX_COMPONENTS],
}

type my_idct_ptr = *mut my_idct_controller;


// Allocated multiplier tables: big enough for any supported variant

#[repr(C)]
union multiplier_table {
    pub islow_array: [ISLOW_MULT_TYPE; DCTSIZE2],
    #[cfg(feature = "DCT_IFAST_SUPPORTED")]
    pub ifast_array: [IFAST_MULT_TYPE; DCTSIZE2],
    #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
    pub float_array: [FLOAT_MULT_TYPE; DCTSIZE2],
}


// The current scaled-IDCT routines require ISLOW-style multiplier tables,
// so be sure to compile that code if either ISLOW or SCALING is requested.

#[cfg(any(feature = "DCT_ISLOW_SUPPORTED", feature = "IDCT_SCALING_SUPPORTED"))]
const PROVIDE_ISLOW_TABLES: bool = true;
#[cfg(not(any(feature = "DCT_ISLOW_SUPPORTED", feature = "IDCT_SCALING_SUPPORTED")))]
const PROVIDE_ISLOW_TABLES: bool = false;


/*
 * Prepare for an output pass.
 * Here we select the proper IDCT routine for each component and build
 * a matching multiplier table.
 */

unsafe fn start_pass(cinfo: j_decompress_ptr) {
    let idct = (*cinfo).idct as my_idct_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut method: c_int = 0;
    let mut method_ptr: Option<inverse_DCT_method_ptr> = None;
    let mut qtbl: *mut JQUANT_TBL;
    let mut i: c_int;

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Select the proper IDCT routine for this component's scaling */
        match (*compptr).DCT_scaled_size {
            #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
            1 => {
                method_ptr = Some(jpeg_idct_1x1);
                method = JDCT_ISLOW;  /* jidctred uses islow-style table */
            }
            #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
            2 => {
                method_ptr = Some(jpeg_idct_2x2);
                method = JDCT_ISLOW;  /* jidctred uses islow-style table */
            }
            #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
            4 => {
                method_ptr = Some(jpeg_idct_4x4);
                method = JDCT_ISLOW;  /* jidctred uses islow-style table */
            }
            DCTSIZE as c_int => {
                match (*cinfo).dct_method {
                    #[cfg(feature = "DCT_ISLOW_SUPPORTED")]
                    JDCT_ISLOW => {
                        method_ptr = Some(jpeg_idct_islow);
                        method = JDCT_ISLOW;
                    }
                    #[cfg(feature = "DCT_IFAST_SUPPORTED")]
                    JDCT_IFAST => {
                        method_ptr = Some(jpeg_idct_ifast);
                        method = JDCT_IFAST;
                    }
                    #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
                    JDCT_FLOAT => {
                        method_ptr = Some(jpeg_idct_float);
                        method = JDCT_FLOAT;
                    }
                    _ => {
                        ERREXIT(cinfo, JERR_NOT_COMPILED);
                    }
                }
            }
            _ => {
                ERREXIT1(cinfo, JERR_BAD_DCTSIZE, (*compptr).DCT_scaled_size);
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
            #[cfg(any(feature = "DCT_ISLOW_SUPPORTED", feature = "IDCT_SCALING_SUPPORTED"))]
            JDCT_ISLOW => {
                /* For LL&M IDCT method, multipliers are equal to raw quantization
                 * coefficients, but are stored in natural order as ints.
                 */
                let ismtbl = (*compptr).dct_table as *mut ISLOW_MULT_TYPE;
                i = 0;
                while i < DCTSIZE2 as c_int {
                    *ismtbl.add(i as usize) =
                        (*qtbl).quantval[jpeg_zigzag_order[i as usize] as usize] as ISLOW_MULT_TYPE;
                    i += 1;
                }
            }
            #[cfg(feature = "DCT_IFAST_SUPPORTED")]
            JDCT_IFAST => {
                /* For AA&N IDCT method, multipliers are equal to quantization
                 * coefficients scaled by scalefactor[row]*scalefactor[col], where
                 *   scalefactor[0] = 1
                 *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                 * For integer operation, the multiplier table is to be scaled by
                 * IFAST_SCALE_BITS.  The multipliers are stored in natural order.
                 */
                let ifmtbl = (*compptr).dct_table as *mut IFAST_MULT_TYPE;
                const CONST_BITS: c_int = 14;
                static AANSCALES: [INT16; DCTSIZE2] = [
                    /* precomputed values scaled up by 14 bits */
                    16384, 22725, 21407, 19266, 16384, 12873, 8867, 4520,
                    22725, 31521, 29692, 26722, 22725, 17855, 12299, 6270,
                    21407, 29692, 27969, 25172, 21407, 16819, 11585, 5906,
                    19266, 26722, 25172, 22654, 19266, 15137, 10426, 5315,
                    16384, 22725, 21407, 19266, 16384, 12873, 8867, 4520,
                    12873, 17855, 16819, 15137, 12873, 10114, 6967, 3552,
                    8867, 12299, 11585, 10426, 8867, 6967, 4799, 2446,
                    4520, 6270, 5906, 5315, 4520, 3552, 2446, 1247,
                ];

                i = 0;
                while i < DCTSIZE2 as c_int {
                    let temp = ((*qtbl).quantval[jpeg_zigzag_order[i as usize] as usize] as INT32
                        * AANSCALES[i as usize] as INT32)
                        + (1i32 << ((CONST_BITS - IFAST_SCALE_BITS) - 1));
                    *ifmtbl.add(i as usize) =
                        (temp >> (CONST_BITS - IFAST_SCALE_BITS)) as IFAST_MULT_TYPE;
                    i += 1;
                }
            }
            #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
            JDCT_FLOAT => {
                /* For float AA&N IDCT method, multipliers are equal to quantization
                 * coefficients scaled by scalefactor[row]*scalefactor[col], where
                 *   scalefactor[0] = 1
                 *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                 * The multipliers are stored in natural order.
                 */
                let fmtbl = (*compptr).dct_table as *mut FLOAT_MULT_TYPE;
                let mut row: c_int;
                let mut col: c_int;
                static AANSCALEFACTOR: [f64; DCTSIZE] = [
                    1.0,
                    1.387039845,
                    1.306562965,
                    1.175875602,
                    1.0,
                    0.785694958,
                    0.541196100,
                    0.275899379,
                ];

                i = 0;
                row = 0;
                while row < DCTSIZE as c_int {
                    col = 0;
                    while col < DCTSIZE as c_int {
                        *fmtbl.add(i as usize) = ((*qtbl).quantval
                            [jpeg_zigzag_order[i as usize] as usize] as f64
                            * AANSCALEFACTOR[row as usize]
                            * AANSCALEFACTOR[col as usize])
                            as FLOAT_MULT_TYPE;
                        i += 1;
                        col += 1;
                    }
                    row += 1;
                }
            }
            _ => {
                ERREXIT(cinfo, JERR_NOT_COMPILED);
            }
        }
        ci += 1;
        compptr = compptr.add(1);
    }
}


/*
 * Initialize IDCT manager.
 */

pub unsafe fn jinit_inverse_dct(cinfo: j_decompress_ptr) {
    let mut idct: my_idct_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    idct = (*(*cinfo).mem).alloc_small.unwrap()(
        cinfo as *mut c_void,
        JPOOL_IMAGE,
        SIZEOF::<my_idct_controller>(),
    ) as my_idct_ptr;
    (*cinfo).idct = idct as *mut jpeg_inverse_dct;
    (*idct).pub_.start_pass = Some(start_pass);

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Allocate and pre-zero a multiplier table for each component */
        (*compptr).dct_table = (*(*cinfo).mem).alloc_small.unwrap()(
            cinfo as *mut c_void,
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
