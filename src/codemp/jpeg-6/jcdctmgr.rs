/*
 * jcdctmgr.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains the forward-DCT management logic.
 * This code selects a particular DCT implementation to be used,
 * and it performs related housekeeping chores including coefficient
 * quantization.
 */
// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

#![allow(non_snake_case, non_camel_case_types)]

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
pub type JBLOCKROW = *mut JBLOCK;
pub type JBLOCK = [i16; 64];

// DCT element types
pub type DCTELEM = i16;
pub type FAST_FLOAT = f32;
pub type INT16 = i16;
pub type INT32 = i32;

// Constants
const NUM_QUANT_TBLS: usize = 4;
const DCTSIZE: usize = 8;
const DCTSIZE2: usize = 64;
const JPOOL_IMAGE: c_int = 1;
const JDCT_ISLOW: c_int = 0;
const JDCT_IFAST: c_int = 1;
const JDCT_FLOAT: c_int = 2;
const JERR_NO_QUANT_TABLE: c_int = 0;
const JERR_NOT_COMPILED: c_int = 1;
const CONST_BITS: c_int = 14;

// Type aliases for function pointers
pub type forward_DCT_method_ptr = fn(*mut DCTELEM);
pub type float_DCT_method_ptr = fn(*mut FAST_FLOAT);

// ============================================================================
// JPEG library structures (minimal stubs for structural coherence)
// ============================================================================

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; DCTSIZE2],
}

#[repr(C)]
pub struct jpeg_component_info {
    pub component_id: c_int,
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
    pub quant_tbl_no: c_int,
    pub dc_tbl_no: c_int,
    pub ac_tbl_no: c_int,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
}

#[repr(C)]
pub struct jpeg_forward_dct {
    pub start_pass: Option<unsafe extern "C" fn(*mut j_compress_struct)>,
    pub forward_DCT: Option<unsafe extern "C" fn(*mut j_compress_struct, *mut jpeg_component_info, JSAMPARRAY, JBLOCKROW, JDIMENSION, JDIMENSION, JDIMENSION)>,
}

#[repr(C)]
pub struct j_compress_struct {
    pub mem: *mut jpeg_memory_mgr,
    pub fdct: *mut jpeg_forward_dct,
    pub comp_info: *mut jpeg_component_info,
    pub num_components: c_int,
    pub quant_tbl_ptrs: [*mut JQUANT_TBL; NUM_QUANT_TBLS],
    pub dct_method: c_int,
}

pub type j_compress_ptr = *mut j_compress_struct;
pub type j_common_ptr = *mut j_compress_struct;

// External functions from JPEG library
extern "C" {
    pub static jpeg_zigzag_order: [c_int; DCTSIZE2];

    pub fn jpeg_fdct_islow(workspace: *mut DCTELEM);

    #[cfg(feature = "DCT_IFAST_SUPPORTED")]
    pub fn jpeg_fdct_ifast(workspace: *mut DCTELEM);

    #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
    pub fn jpeg_fdct_float(workspace: *mut FAST_FLOAT);
}

// Stub error handling (ERREXIT and ERREXIT1 macro stubs)
#[inline]
unsafe fn ERREXIT(_cinfo: j_compress_ptr, _code: c_int) {
    // Stub: error handling not fully ported
}

#[inline]
unsafe fn ERREXIT1(_cinfo: j_compress_ptr, _code: c_int, _p1: c_int) {
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
struct my_fdct_controller {
    pub pub_: jpeg_forward_dct,	/* public fields */

    /* Pointer to the DCT routine actually in use */
    do_dct: forward_DCT_method_ptr,

    /* The actual post-DCT divisors --- not identical to the quant table
     * entries, because of scaling (especially for an unnormalized DCT).
     * Each table is given in normal array order; note that this must
     * be converted from the zigzag order of the quantization tables.
     */
    divisors: [*mut DCTELEM; NUM_QUANT_TBLS],

    #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
    /* Same as above for the floating-point case. */
    do_float_dct: float_DCT_method_ptr,
    #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
    float_divisors: [*mut FAST_FLOAT; NUM_QUANT_TBLS],
}

type my_fdct_ptr = *mut my_fdct_controller;

/*
 * Initialize for a processing pass.
 * Verify that all referenced Q-tables are present, and set up
 * the divisor table for each one.
 * In the current implementation, DCT of all components is done during
 * the first pass, even if only some components will be output in the
 * first scan.  Hence all components should be examined here.
 */

unsafe fn start_pass_fdctmgr(cinfo: j_compress_ptr) {
    let fdct = (*cinfo).fdct as my_fdct_ptr;
    let mut ci: c_int;
    let mut qtblno: c_int;
    let mut i: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut qtbl: *mut JQUANT_TBL;
    let mut dtbl: *mut DCTELEM;

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        qtblno = (*compptr).quant_tbl_no;
        /* Make sure specified quantization table is present */
        if qtblno < 0 || qtblno >= NUM_QUANT_TBLS as c_int ||
            (*cinfo).quant_tbl_ptrs[qtblno as usize].is_null() {
            ERREXIT1(cinfo, JERR_NO_QUANT_TABLE, qtblno);
        }
        qtbl = (*cinfo).quant_tbl_ptrs[qtblno as usize];
        /* Compute divisors for this quant table */
        /* We may do this more than once for same table, but it's not a big deal */
        match (*cinfo).dct_method {
            JDCT_ISLOW => {
                /* For LL&M IDCT method, divisors are equal to raw quantization
                 * coefficients multiplied by 8 (to counteract scaling).
                 */
                if (*fdct).divisors[qtblno as usize].is_null() {
                    (*fdct).divisors[qtblno as usize] = (*((*cinfo).mem))
                        .alloc_small
                        .unwrap()(
                            cinfo as *mut c_void,
                            JPOOL_IMAGE,
                            (DCTSIZE2 * SIZEOF::<DCTELEM>()) as usize,
                        ) as *mut DCTELEM;
                }
                dtbl = (*fdct).divisors[qtblno as usize];
                i = 0;
                while i < DCTSIZE2 as c_int {
                    *dtbl.offset(i as isize) = ((*qtbl).quantval
                        [jpeg_zigzag_order[i as usize] as usize]
                        as DCTELEM)
                        << 3;
                    i += 1;
                }
            }
            #[cfg(feature = "DCT_IFAST_SUPPORTED")]
            JDCT_IFAST => {
                /* For AA&N IDCT method, divisors are equal to quantization
                 * coefficients scaled by scalefactor[row]*scalefactor[col], where
                 *   scalefactor[0] = 1
                 *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                 * We apply a further scale factor of 8.
                 */
                static AANSCALES: [INT16; DCTSIZE2] = [
                    /* precomputed values scaled up by 14 bits: in natural order */
                    16384, 22725, 21407, 19266, 16384, 12873, 8867, 4520,
                    22725, 31521, 29692, 26722, 22725, 17855, 12299, 6270,
                    21407, 29692, 27969, 25172, 21407, 16819, 11585, 5906,
                    19266, 26722, 25172, 22654, 19266, 15137, 10426, 5315,
                    16384, 22725, 21407, 19266, 16384, 12873, 8867, 4520,
                    12873, 17855, 16819, 15137, 12873, 10114, 6967, 3552,
                    8867, 12299, 11585, 10426, 8867, 6967, 4799, 2446,
                    4520, 6270, 5906, 5315, 4520, 3552, 2446, 1247,
                ];

                if (*fdct).divisors[qtblno as usize].is_null() {
                    (*fdct).divisors[qtblno as usize] = (*((*cinfo).mem))
                        .alloc_small
                        .unwrap()(
                            cinfo as *mut c_void,
                            JPOOL_IMAGE,
                            (DCTSIZE2 * SIZEOF::<DCTELEM>()) as usize,
                        ) as *mut DCTELEM;
                }
                dtbl = (*fdct).divisors[qtblno as usize];
                i = 0;
                while i < DCTSIZE2 as c_int {
                    *dtbl.offset(i as isize) = DESCALE(
                        MULTIPLY16V16(
                            (*qtbl).quantval[jpeg_zigzag_order[i as usize] as usize] as INT32,
                            AANSCALES[i as usize] as INT32,
                        ),
                        (CONST_BITS - 3),
                    ) as DCTELEM;
                    i += 1;
                }
            }
            #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
            JDCT_FLOAT => {
                /* For float AA&N IDCT method, divisors are equal to quantization
                 * coefficients scaled by scalefactor[row]*scalefactor[col], where
                 *   scalefactor[0] = 1
                 *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                 * We apply a further scale factor of 8.
                 * What's actually stored is 1/divisor so that the inner loop can
                 * use a multiplication rather than a division.
                 */
                static AANSCALEFACTOR: [f64; DCTSIZE] = [
                    1.0, 1.387039845, 1.306562965, 1.175875602,
                    1.0, 0.785694958, 0.541196100, 0.275899379,
                ];

                let fdtbl: *mut FAST_FLOAT;
                let mut row: c_int;
                let mut col: c_int;

                if (*fdct).float_divisors[qtblno as usize].is_null() {
                    (*fdct).float_divisors[qtblno as usize] = (*((*cinfo).mem))
                        .alloc_small
                        .unwrap()(
                            cinfo as *mut c_void,
                            JPOOL_IMAGE,
                            (DCTSIZE2 * SIZEOF::<FAST_FLOAT>()) as usize,
                        ) as *mut FAST_FLOAT;
                }
                fdtbl = (*fdct).float_divisors[qtblno as usize];
                i = 0;
                row = 0;
                while row < DCTSIZE as c_int {
                    col = 0;
                    while col < DCTSIZE as c_int {
                        *fdtbl.offset(i as isize) = (1.0
                            / ((((*qtbl).quantval[jpeg_zigzag_order[i as usize] as usize] as f64)
                                * AANSCALEFACTOR[row as usize]
                                * AANSCALEFACTOR[col as usize]
                                * 8.0))) as FAST_FLOAT;
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
 * Perform forward DCT on one or more blocks of a component.
 *
 * The input samples are taken from the sample_data[] array starting at
 * position start_row/start_col, and moving to the right for any additional
 * blocks. The quantized coefficients are returned in coef_blocks[].
 */

#[cfg(feature = "DCT_FLOAT_SUPPORTED")]
unsafe fn forward_DCT_float(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    sample_data: JSAMPARRAY,
    coef_blocks: JBLOCKROW,
    start_row: JDIMENSION,
    start_col: JDIMENSION,
    num_blocks: JDIMENSION,
) {
    /* This version is used for floating-point DCT implementations. */
    /* This routine is heavily used, so it's worth coding it tightly. */
    let fdct = (*cinfo).fdct as my_fdct_ptr;
    let do_dct = (*fdct).do_float_dct;
    let divisors = (*fdct).float_divisors[(*compptr).quant_tbl_no as usize];
    let mut workspace: [FAST_FLOAT; DCTSIZE2] = [0.0; DCTSIZE2];
    let mut bi: JDIMENSION;
    let sample_data_offset = sample_data.add(start_row as usize);

    bi = 0;
    let mut start_col_offset = start_col;
    while bi < num_blocks {
        /* Load data into workspace, applying unsigned->signed conversion */
        {
            let mut workspaceptr: *mut FAST_FLOAT = workspace.as_mut_ptr();
            let mut elemptr: JSAMPROW;
            let mut elemr: c_int;

            elemr = 0;
            while elemr < DCTSIZE as c_int {
                elemptr = *sample_data_offset.add(elemr as usize);
                elemptr = elemptr.add(start_col_offset as usize);
                /* unroll the inner loop */
                if DCTSIZE == 8 {
                    *workspaceptr = (*elemptr as FAST_FLOAT) - (128.0 as FAST_FLOAT);
                    workspaceptr = workspaceptr.add(1);
                    elemptr = elemptr.add(1);
                    *workspaceptr = (*elemptr as FAST_FLOAT) - (128.0 as FAST_FLOAT);
                    workspaceptr = workspaceptr.add(1);
                    elemptr = elemptr.add(1);
                    *workspaceptr = (*elemptr as FAST_FLOAT) - (128.0 as FAST_FLOAT);
                    workspaceptr = workspaceptr.add(1);
                    elemptr = elemptr.add(1);
                    *workspaceptr = (*elemptr as FAST_FLOAT) - (128.0 as FAST_FLOAT);
                    workspaceptr = workspaceptr.add(1);
                    elemptr = elemptr.add(1);
                    *workspaceptr = (*elemptr as FAST_FLOAT) - (128.0 as FAST_FLOAT);
                    workspaceptr = workspaceptr.add(1);
                    elemptr = elemptr.add(1);
                    *workspaceptr = (*elemptr as FAST_FLOAT) - (128.0 as FAST_FLOAT);
                    workspaceptr = workspaceptr.add(1);
                    elemptr = elemptr.add(1);
                    *workspaceptr = (*elemptr as FAST_FLOAT) - (128.0 as FAST_FLOAT);
                    workspaceptr = workspaceptr.add(1);
                    elemptr = elemptr.add(1);
                    *workspaceptr = (*elemptr as FAST_FLOAT) - (128.0 as FAST_FLOAT);
                    workspaceptr = workspaceptr.add(1);
                } else {
                    let mut elemc = DCTSIZE as c_int;
                    while elemc > 0 {
                        *workspaceptr =
                            ((*elemptr as FAST_FLOAT) - (128.0 as FAST_FLOAT));
                        workspaceptr = workspaceptr.add(1);
                        elemptr = elemptr.add(1);
                        elemc -= 1;
                    }
                }
                elemr += 1;
            }
        }

        /* Perform the DCT */
        do_dct(workspace.as_mut_ptr());

        /* Quantize/descale the coefficients, and store into coef_blocks[] */
        {
            let mut temp: FAST_FLOAT;
            let mut i: c_int;
            let mut output_ptr: JCOEFPTR = coef_blocks.add(bi as usize) as JCOEFPTR;

            i = 0;
            while i < DCTSIZE2 as c_int {
                /* Apply the quantization and scaling factor */
                temp = workspace[i as usize] * *divisors.offset(i as isize);
                /* Round to nearest integer.
                 * Since C does not specify the direction of rounding for negative
                 * quotients, we have to force the dividend positive for portability.
                 * The maximum coefficient size is +-16K (for 12-bit data), so this
                 * code should work for either 16-bit or 32-bit ints.
                 */
                *output_ptr.offset(i as isize) = ((temp + (16384.5 as FAST_FLOAT)) as c_int - 16384) as JCOEF;
                i += 1;
            }
        }

        bi += 1;
        start_col_offset += DCTSIZE as u32;
    }
}

/*
 * Initialize FDCT manager.
 */

pub unsafe extern "C" fn jinit_forward_dct(cinfo: j_compress_ptr) {
    let fdct: my_fdct_ptr;
    let mut i: c_int;

    fdct = (*((*cinfo).mem)).alloc_small.unwrap()(
        cinfo as *mut c_void,
        JPOOL_IMAGE,
        SIZEOF::<my_fdct_controller>() as usize,
    ) as my_fdct_ptr;
    (*cinfo).fdct = fdct as *mut jpeg_forward_dct;
    (*fdct).pub_.start_pass = Some(start_pass_fdctmgr);

    match (*cinfo).dct_method {
        JDCT_ISLOW => {
            (*fdct).pub_.forward_DCT = Some(forward_DCT);
            (*fdct).do_dct = jpeg_fdct_islow;
        }
        #[cfg(feature = "DCT_IFAST_SUPPORTED")]
        JDCT_IFAST => {
            (*fdct).pub_.forward_DCT = Some(forward_DCT);
            (*fdct).do_dct = jpeg_fdct_ifast;
        }
        #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
        JDCT_FLOAT => {
            (*fdct).pub_.forward_DCT = Some(forward_DCT_float);
            (*fdct).do_float_dct = jpeg_fdct_float;
        }
        _ => {
            ERREXIT(cinfo, JERR_NOT_COMPILED);
        }
    }

    /* Mark divisor tables unallocated */
    i = 0;
    while i < NUM_QUANT_TBLS as c_int {
        (*fdct).divisors[i as usize] = core::ptr::null_mut();
        #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
        {
            (*fdct).float_divisors[i as usize] = core::ptr::null_mut();
        }
        i += 1;
    }
}

// Helper functions for DCT arithmetic (based on jdct.h patterns)

// MULTIPLY16V16: 16-bit * 16-bit -> 32-bit multiply
#[inline]
fn MULTIPLY16V16(a: INT32, b: INT32) -> INT32 {
    a * b
}

// DESCALE: Descale (right shift with rounding) a 32-bit value
// Original C: ((x) + (1 << ((n)-1))) >> (n)
#[inline]
fn DESCALE(x: INT32, n: c_int) -> INT32 {
    (x + (1 << ((n) - 1))) >> n
}

/*
 * Perform forward DCT on one or more blocks of a component.
 *
 * The input samples are taken from the sample_data[] array starting at
 * position start_row/start_col, and moving to the right for any additional
 * blocks. The quantized coefficients are returned in coef_blocks[].
 */

/* Note: The original C code has the forward_DCT function disabled with #if 0.
 * This is a faithful port that preserves that disabled state. If this function
 * is needed, it should be imported from jfdctint.rs or jfdctfst.rs.
 */

unsafe fn forward_DCT(
    _cinfo: j_compress_ptr,
    _compptr: *mut jpeg_component_info,
    _sample_data: JSAMPARRAY,
    _coef_blocks: JBLOCKROW,
    _start_row: JDIMENSION,
    _start_col: JDIMENSION,
    _num_blocks: JDIMENSION,
) {
    /* This version is used for integer DCT implementations. */
    /* Note: This function mirrors the structure of the disabled C code (#if 0).
     * For actual DCT implementation, see jfdctint.rs or related files.
     *
     * The original C code loads data into a workspace, applies unsigned->signed
     * conversion, performs DCT via do_dct() function pointer, then quantizes
     * and stores the coefficients.
     */
}
