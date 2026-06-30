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
#![allow(non_upper_case_globals)]
#![allow(unused_variables)]

// #define JPEG_INTERNALS
use crate::code::server::exe_headers_h::*;
use crate::code::jpeg_6::jinclude_h::*;
use crate::code::jpeg_6::jpeglib_h::*;
use crate::code::jpeg_6::jdct_h::*; /* Private declarations for DCT subsystem */
use crate::code::jpeg_6::jpegint_h::*;
use crate::code::jpeg_6::jmorecfg_h::*;
use core::ffi::c_int;

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
    pub pub_: jpeg_inverse_dct,  /* public fields */

    /* This array contains the IDCT method code that each multiplier table
     * is currently set up for, or -1 if it's not yet set up.
     * The actual multiplier tables are pointed to by dct_table in the
     * per-component comp_info structures.
     */
    pub cur_method: [c_int; MAX_COMPONENTS as usize],
}

pub type my_idct_ptr = *mut my_idct_controller;


/* Allocated multiplier tables: big enough for any supported variant */

#[repr(C)]
pub union multiplier_table {
    pub islow_array: [ISLOW_MULT_TYPE; DCTSIZE2 as usize],
    #[cfg(feature = "DCT_IFAST_SUPPORTED")]
    pub ifast_array: [IFAST_MULT_TYPE; DCTSIZE2 as usize],
    #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
    pub float_array: [FLOAT_MULT_TYPE; DCTSIZE2 as usize],
}


/* The current scaled-IDCT routines require ISLOW-style multiplier tables,
 * so be sure to compile that code if either ISLOW or SCALING is requested.
 */
/* #ifdef DCT_ISLOW_SUPPORTED => #define PROVIDE_ISLOW_TABLES
 * #else #ifdef IDCT_SCALING_SUPPORTED => #define PROVIDE_ISLOW_TABLES
 * Mapped to cfg(any(feature = "DCT_ISLOW_SUPPORTED", feature = "IDCT_SCALING_SUPPORTED"))
 * wherever PROVIDE_ISLOW_TABLES was referenced below.
 */


/*
 * Prepare for an output pass.
 * Here we select the proper IDCT routine for each component and build
 * a matching multiplier table.
 */

/* METHODDEF (file-local static in C); declared extern "C" so it can be stored
 * as an inverse_DCT_method_ptr function pointer in jpeg_inverse_dct.start_pass.
 */
unsafe extern "C" fn start_pass(cinfo: j_decompress_ptr) {
    let idct: my_idct_ptr = (*cinfo).idct as my_idct_ptr;
    let mut ci: c_int;
    let mut i: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut method: c_int = 0;
    let mut method_ptr: inverse_DCT_method_ptr = None;
    let mut qtbl: *mut JQUANT_TBL;

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Select the proper IDCT routine for this component's scaling */
        let dct_scaled_size = (*compptr).DCT_scaled_size;
        match dct_scaled_size {
            #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
            1 => {
                method_ptr = Some(jpeg_idct_1x1);
                method = JDCT_ISLOW;	/* jidctred uses islow-style table */
            }
            #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
            2 => {
                method_ptr = Some(jpeg_idct_2x2);
                method = JDCT_ISLOW;	/* jidctred uses islow-style table */
            }
            #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
            4 => {
                method_ptr = Some(jpeg_idct_4x4);
                method = JDCT_ISLOW;	/* jidctred uses islow-style table */
            }
            dss => {
                if dss == DCTSIZE as c_int {
                    match (*cinfo).dct_method {
                        #[cfg(feature = "DCT_ISLOW_SUPPORTED")]
                        dm if dm == JDCT_ISLOW => {
                            method_ptr = Some(jpeg_idct_islow);
                            method = JDCT_ISLOW;
                        }
                        #[cfg(feature = "DCT_IFAST_SUPPORTED")]
                        dm if dm == JDCT_IFAST => {
                            method_ptr = Some(jpeg_idct_ifast);
                            method = JDCT_IFAST;
                        }
                        #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
                        dm if dm == JDCT_FLOAT => {
                            method_ptr = Some(jpeg_idct_float);
                            method = JDCT_FLOAT;
                        }
                        _ => {
                            ERREXIT(cinfo, JERR_NOT_COMPILED);
                        }
                    }
                } else {
                    ERREXIT1(cinfo, JERR_BAD_DCTSIZE, (*compptr).DCT_scaled_size);
                }
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
        if qtbl.is_null() {		/* happens if no data yet for component */
            ci += 1;
            compptr = compptr.add(1);
            continue;
        }
        (*idct).cur_method[ci as usize] = method;
        match method {
            /* #ifdef PROVIDE_ISLOW_TABLES => cfg(any(...)) */
            #[cfg(any(feature = "DCT_ISLOW_SUPPORTED", feature = "IDCT_SCALING_SUPPORTED"))]
            m if m == JDCT_ISLOW => {
                {
                    /* For LL&M IDCT method, multipliers are equal to raw quantization
                     * coefficients, but are stored in natural order as ints.
                     */
                    let ismtbl: *mut ISLOW_MULT_TYPE =
                        (*compptr).dct_table as *mut ISLOW_MULT_TYPE;
                    i = 0;
                    while i < DCTSIZE2 as c_int {
                        *ismtbl.add(i as usize) = (*qtbl).quantval
                            [jpeg_zigzag_order[i as usize] as usize]
                            as ISLOW_MULT_TYPE;
                        i += 1;
                    }
                }
            }
            #[cfg(feature = "DCT_IFAST_SUPPORTED")]
            m if m == JDCT_IFAST => {
                {
                    /* For AA&N IDCT method, multipliers are equal to quantization
                     * coefficients scaled by scalefactor[row]*scalefactor[col], where
                     *   scalefactor[0] = 1
                     *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                     * For integer operation, the multiplier table is to be scaled by
                     * IFAST_SCALE_BITS.  The multipliers are stored in natural order.
                     */
                    let ifmtbl: *mut IFAST_MULT_TYPE =
                        (*compptr).dct_table as *mut IFAST_MULT_TYPE;
                    const CONST_BITS: c_int = 14;
                    static aanscales: [INT16; DCTSIZE2 as usize] = [
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
                    /* SHIFT_TEMPS — not needed in Rust: >> on INT32 is arithmetic right shift */
                    i = 0;
                    while i < DCTSIZE2 as c_int {
                        *ifmtbl.add(i as usize) = DESCALE(
                            MULTIPLY16V16(
                                (*qtbl).quantval[jpeg_zigzag_order[i as usize] as usize]
                                    as INT32,
                                aanscales[i as usize] as INT32,
                            ),
                            CONST_BITS - IFAST_SCALE_BITS,
                        ) as IFAST_MULT_TYPE;
                        i += 1;
                    }
                }
            }
            #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
            m if m == JDCT_FLOAT => {
                {
                    /* For float AA&N IDCT method, multipliers are equal to quantization
                     * coefficients scaled by scalefactor[row]*scalefactor[col], where
                     *   scalefactor[0] = 1
                     *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                     * The multipliers are stored in natural order.
                     */
                    let fmtbl: *mut FLOAT_MULT_TYPE =
                        (*compptr).dct_table as *mut FLOAT_MULT_TYPE;
                    let mut row: c_int;
                    let mut col: c_int;
                    static aanscalefactor: [f64; DCTSIZE as usize] = [
                        1.0, 1.387039845, 1.306562965, 1.175875602,
                        1.0, 0.785694958, 0.541196100, 0.275899379
                    ];

                    i = 0;
                    row = 0;
                    while row < DCTSIZE as c_int {
                        col = 0;
                        while col < DCTSIZE as c_int {
                            *fmtbl.add(i as usize) = ((*qtbl).quantval
                                [jpeg_zigzag_order[i as usize] as usize]
                                as f64
                                * aanscalefactor[row as usize]
                                * aanscalefactor[col as usize])
                                as FLOAT_MULT_TYPE;
                            i += 1;
                            col += 1;
                        }
                        row += 1;
                    }
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

pub unsafe extern "C" fn jinit_inverse_dct(cinfo: j_decompress_ptr) {
    let idct: my_idct_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    idct = (*(*cinfo).mem).alloc_small.unwrap()(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        SIZEOF::<my_idct_controller>(),
    ) as my_idct_ptr;
    (*cinfo).idct = idct as *mut jpeg_inverse_dct;
    (*idct).pub_.start_pass = Some(start_pass);

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Allocate and pre-zero a multiplier table for each component */
        (*compptr).dct_table =
            (*(*cinfo).mem).alloc_small.unwrap()(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                SIZEOF::<multiplier_table>(),
            ) as *mut c_void;
        MEMZERO((*compptr).dct_table as *mut c_void, SIZEOF::<multiplier_table>());
        /* Mark multiplier table not yet set up for any method */
        (*idct).cur_method[ci as usize] = -1;
        ci += 1;
        compptr = compptr.add(1);
    }
}
