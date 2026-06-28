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

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// ============================================================================
// Constants (from jdct.h, jpeglib.h)
// ============================================================================
pub const NUM_QUANT_TBLS: usize = 4;
pub const DCTSIZE: usize = 8;
pub const DCTSIZE2: usize = 64;
pub const JPOOL_IMAGE: c_int = 0;
pub const CONST_BITS: c_int = 14;

// ============================================================================
// External JPEG Library Stubs
// ============================================================================
// These declarations represent JPEG library types and functions that are
// defined in the JPEG headers (jinclude.h, jpeglib.h, jdct.h).
// For the faithful port, we declare these as opaque types and extern functions.

// Type aliases for JPEG types (must be defined early)
pub type DCTELEM = core::ffi::c_short;
pub type FAST_FLOAT = f32;
pub type JDIMENSION = u32;
pub type JSAMPROW = *mut u8;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JCOEF = core::ffi::c_short;
pub type JCOEFPTR = *mut JCOEF;
pub type JBLOCKROW = *mut JCOEF;

// Type stubs for JPEG library types
// Defined in jpeglib.h
#[repr(C)]
pub struct jpeg_compress_struct {
    // Minimal field definitions needed by this module
    pub fdct: *mut c_void,
    pub comp_info: *mut c_void,
    pub num_components: c_int,
    pub quant_tbl_ptrs: *mut *mut JQUANT_TBL,
    pub dct_method: c_int,
    pub mem: *mut c_void,
}

pub type j_compress_ptr = *mut jpeg_compress_struct;

#[repr(C)]
pub struct jpeg_component_info {
    // Minimal field definitions needed by this module
    pub quant_tbl_no: c_int,
    // ... other fields omitted
    _unused: [u8; 0],
}

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; DCTSIZE2],
    // ... other fields omitted
}

// Function pointer types (must be defined before use in structs)
pub type forward_DCT_method_ptr = Option<unsafe extern "C" fn(*mut DCTELEM)>;
pub type float_DCT_method_ptr = Option<unsafe extern "C" fn(*mut FAST_FLOAT)>;
pub type start_pass_method_ptr = Option<unsafe extern "C" fn(j_compress_ptr)>;
pub type DCT_method_ptr = Option<unsafe extern "C" fn(j_compress_ptr, *mut jpeg_component_info, JSAMPARRAY, JBLOCKROW, JDIMENSION, JDIMENSION, JDIMENSION)>;

#[repr(C)]
pub struct jpeg_forward_dct {
    pub start_pass: start_pass_method_ptr,
    pub forward_DCT: DCT_method_ptr,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    // Opaque for this module
    _unused: [u8; 0],
}

// External function declarations
extern "C" {
    // Memory manager functions
    pub fn jpeg_alloc_small(cinfo: j_compress_ptr, pool_id: c_int, size: usize) -> *mut c_void;

    // Error handling macros (simplified as extern stubs)
    pub fn jpeg_errexit1(cinfo: j_compress_ptr, code: c_int, p1: c_int) -> !;
    pub fn jpeg_errexit(cinfo: j_compress_ptr, code: c_int) -> !;

    // DCT method functions
    pub fn jpeg_fdct_islow(data: *mut DCTELEM);
    pub fn jpeg_fdct_ifast(data: *mut DCTELEM);
    pub fn jpeg_fdct_float(data: *mut FAST_FLOAT);

    // Zigzag ordering table
    pub static jpeg_zigzag_order: [usize; DCTSIZE2];
}

// ============================================================================
// Private subobject for this module
// ============================================================================

#[repr(C)]
pub struct my_fdct_controller {
    // public fields
    pub pub_: jpeg_forward_dct,

    // Pointer to the DCT routine actually in use
    pub do_dct: forward_DCT_method_ptr,

    // The actual post-DCT divisors --- not identical to the quant table
    // entries, because of scaling (especially for an unnormalized DCT).
    // Each table is given in normal array order; note that this must
    // be converted from the zigzag order of the quantization tables.
    pub divisors: [*mut DCTELEM; NUM_QUANT_TBLS],

    // Same as above for the floating-point case.
    pub do_float_dct: float_DCT_method_ptr,
    pub float_divisors: [*mut FAST_FLOAT; NUM_QUANT_TBLS],
}

pub type my_fdct_ptr = *mut my_fdct_controller;

// ============================================================================
// Function implementations
// ============================================================================

/*
 * Initialize for a processing pass.
 * Verify that all referenced Q-tables are present, and set up
 * the divisor table for each one.
 * In the current implementation, DCT of all components is done during
 * the first pass, even if only some components will be output in the
 * first scan.  Hence all components should be examined here.
 */
pub unsafe extern "C" fn start_pass_fdctmgr(cinfo: j_compress_ptr) {
    let fdct = (*cinfo).fdct as my_fdct_ptr;
    let mut ci: c_int = 0;
    let mut qtblno: c_int;
    let mut i: c_int;

    let mut compptr = (*cinfo).comp_info as *mut jpeg_component_info;
    let num_components = (*cinfo).num_components;
    let quant_tbl_ptrs = (*cinfo).quant_tbl_ptrs;
    let dct_method = (*cinfo).dct_method;

    ci = 0;
    while ci < num_components {
        qtblno = (*compptr).quant_tbl_no;
        // Make sure specified quantization table is present
        if qtblno < 0 || qtblno >= NUM_QUANT_TBLS as c_int ||
           (*quant_tbl_ptrs.add(qtblno as usize)).is_null() {
            jpeg_errexit1(cinfo, 0, qtblno); // JERR_NO_QUANT_TABLE
        }
        let qtbl = *quant_tbl_ptrs.add(qtblno as usize);
        // Compute divisors for this quant table
        // We may do this more than once for same table, but it's not a big deal
        match dct_method {
            // JDCT_ISLOW case
            0 => {
                // For LL&M IDCT method, divisors are equal to raw quantization
                // coefficients multiplied by 8 (to counteract scaling).
                if (*fdct).divisors[qtblno as usize].is_null() {
                    (*fdct).divisors[qtblno as usize] = jpeg_alloc_small(
                        cinfo,
                        JPOOL_IMAGE,
                        DCTSIZE2 * core::mem::size_of::<DCTELEM>(),
                    ) as *mut DCTELEM;
                }
                let dtbl = (*fdct).divisors[qtblno as usize];
                i = 0;
                while i < DCTSIZE2 as c_int {
                    let zigzag_idx = jpeg_zigzag_order[i as usize];
                    let quantval = (*qtbl).quantval[zigzag_idx];
                    *dtbl.add(i as usize) = (quantval as DCTELEM) << 3;
                    i += 1;
                }
            }
            // JDCT_IFAST case
            1 => {
                // For AA&N IDCT method, divisors are equal to quantization
                // coefficients scaled by scalefactor[row]*scalefactor[col], where
                //   scalefactor[0] = 1
                //   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                // We apply a further scale factor of 8.

                // precomputed values scaled up by 14 bits: in natural order
                static AANSCALES: [i16; DCTSIZE2] = [
                    16384, 22725, 21407, 19266, 16384, 12873,  8867,  4520,
                    22725, 31521, 29692, 26722, 22725, 17855, 12299,  6270,
                    21407, 29692, 27969, 25172, 21407, 16819, 11585,  5906,
                    19266, 26722, 25172, 22654, 19266, 15137, 10426,  5315,
                    16384, 22725, 21407, 19266, 16384, 12873,  8867,  4520,
                    12873, 17855, 16819, 15137, 12873, 10114,  6967,  3552,
                     8867, 12299, 11585, 10426,  8867,  6967,  4799,  2446,
                     4520,  6270,  5906,  5315,  4520,  3552,  2446,  1247
                ];

                if (*fdct).divisors[qtblno as usize].is_null() {
                    (*fdct).divisors[qtblno as usize] = jpeg_alloc_small(
                        cinfo,
                        JPOOL_IMAGE,
                        DCTSIZE2 * core::mem::size_of::<DCTELEM>(),
                    ) as *mut DCTELEM;
                }
                let dtbl = (*fdct).divisors[qtblno as usize];
                i = 0;
                while i < DCTSIZE2 as c_int {
                    let zigzag_idx = jpeg_zigzag_order[i as usize];
                    let quantval = (*qtbl).quantval[zigzag_idx];
                    let aanscale = AANSCALES[i as usize] as i32;
                    let qval_i32 = quantval as i32;

                    // DESCALE(MULTIPLY16V16(qval, aanscale), CONST_BITS - 3)
                    // Simplified: multiply and shift right
                    let product = qval_i32 * aanscale;
                    let shift_amount = CONST_BITS - 3;
                    let divisor_val = ((product + (1 << (shift_amount - 1))) >> shift_amount) as DCTELEM;

                    *dtbl.add(i as usize) = divisor_val;
                    i += 1;
                }
            }
            // JDCT_FLOAT case
            2 => {
                // For float AA&N IDCT method, divisors are equal to quantization
                // coefficients scaled by scalefactor[row]*scalefactor[col], where
                //   scalefactor[0] = 1
                //   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                // We apply a further scale factor of 8.
                // What's actually stored is 1/divisor so that the inner loop can
                // use a multiplication rather than a division.

                static AANSCALEFACTOR: [f64; DCTSIZE] = [
                    1.0, 1.387039845, 1.306562965, 1.175875602,
                    1.0, 0.785694958, 0.541196100, 0.275899379
                ];

                if (*fdct).float_divisors[qtblno as usize].is_null() {
                    (*fdct).float_divisors[qtblno as usize] = jpeg_alloc_small(
                        cinfo,
                        JPOOL_IMAGE,
                        DCTSIZE2 * core::mem::size_of::<FAST_FLOAT>(),
                    ) as *mut FAST_FLOAT;
                }
                let fdtbl = (*fdct).float_divisors[qtblno as usize];
                let mut idx = 0;
                for row in 0..DCTSIZE {
                    for col in 0..DCTSIZE {
                        let zigzag_idx = jpeg_zigzag_order[idx];
                        let quantval = (*qtbl).quantval[zigzag_idx];

                        let denom = ((quantval as f64) *
                                     AANSCALEFACTOR[row] *
                                     AANSCALEFACTOR[col] *
                                     8.0);
                        *fdtbl.add(idx) = (1.0 / denom) as FAST_FLOAT;
                        idx += 1;
                    }
                }
            }
            _ => {
                jpeg_errexit(cinfo, 0); // JERR_NOT_COMPILED
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

// This version was disabled in the original code (#if 0 // bk001204)
// The forward_DCT function is defined in the original but not compiled.
// We preserve its commented state.

/*
pub unsafe extern "C" fn forward_DCT(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    sample_data: JSAMPARRAY,
    coef_blocks: JBLOCKROW,
    start_row: JDIMENSION,
    start_col: JDIMENSION,
    num_blocks: JDIMENSION,
) {
    // This version is used for integer DCT implementations.
    // This routine is heavily used, so it's worth coding it tightly.
    let fdct = (*cinfo).fdct as my_fdct_ptr;
    let do_dct = (*fdct).do_dct;
    let divisors = (*fdct).divisors[(*compptr).quant_tbl_no as usize];
    let mut workspace: [DCTELEM; DCTSIZE2] = [0; DCTSIZE2]; // work area for FDCT subroutine
    let mut bi: JDIMENSION;

    let mut sample_data_ptr = sample_data.add(start_row as usize); // fold in the vertical offset once

    bi = 0;
    while bi < num_blocks {
        // Load data into workspace, applying unsigned->signed conversion
        let mut workspaceptr = &mut workspace[0] as *mut DCTELEM;
        let mut elemr = 0;
        while elemr < DCTSIZE {
            let elemptr = *sample_data_ptr.add(elemr).add(start_col as usize) as *mut u8;
            // For DCTSIZE == 8: unroll the inner loop
            for _ in 0..DCTSIZE {
                let sample = (*elemptr as c_int) - 128; // CENTERJSAMPLE
                *workspaceptr = sample as DCTELEM;
                workspaceptr = workspaceptr.add(1);
            }
            elemr += 1;
        }

        // Perform the DCT
        if let Some(dct_fn) = do_dct {
            dct_fn(&mut workspace[0]);
        }

        // Quantize/descale the coefficients, and store into coef_blocks[]
        let output_ptr = *coef_blocks.add(bi as usize);
        let mut i = 0;
        while i < DCTSIZE2 {
            let qval = *divisors.add(i);
            let mut temp = workspace[i];

            // Divide the coefficient value by qval, ensuring proper rounding.
            // Since C does not specify the direction of rounding for negative
            // quotients, we have to force the dividend positive for portability.
            if temp < 0 {
                temp = -temp;
                temp += qval >> 1; // for rounding
                if temp >= qval {
                    temp /= qval;
                } else {
                    temp = 0;
                }
                temp = -temp;
            } else {
                temp += qval >> 1; // for rounding
                if temp >= qval {
                    temp /= qval;
                } else {
                    temp = 0;
                }
            }
            *output_ptr.add(i) = temp as JCOEF;
            i += 1;
        }

        bi += 1;
        // Note: original had start_col += DCTSIZE in the loop increment
    }
}
*/

#[cfg(feature = "DCT_FLOAT_SUPPORTED")]
pub unsafe extern "C" fn forward_DCT_float(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    sample_data: JSAMPARRAY,
    coef_blocks: JBLOCKROW,
    start_row: JDIMENSION,
    start_col: JDIMENSION,
    num_blocks: JDIMENSION,
) {
    // This version is used for floating-point DCT implementations.
    // This routine is heavily used, so it's worth coding it tightly.
    let fdct = (*cinfo).fdct as my_fdct_ptr;
    let do_dct = (*fdct).do_float_dct;
    let divisors = (*fdct).float_divisors[(*compptr).quant_tbl_no as usize];
    let mut workspace: [FAST_FLOAT; DCTSIZE2] = [0.0; DCTSIZE2]; // work area for FDCT subroutine
    let mut bi: JDIMENSION;

    let mut sample_data_ptr = sample_data.add(start_row as usize); // fold in the vertical offset once

    bi = 0;
    while bi < num_blocks {
        // Load data into workspace, applying unsigned->signed conversion
        let mut workspaceptr = &mut workspace[0] as *mut FAST_FLOAT;
        let mut elemr = 0;
        while elemr < DCTSIZE {
            let mut elemptr = (*sample_data_ptr.add(elemr)).add(start_col as usize);
            // For DCTSIZE == 8: unroll the inner loop
            for _ in 0..DCTSIZE {
                let sample = (*elemptr as c_int) - 128; // CENTERJSAMPLE
                *workspaceptr = sample as FAST_FLOAT;
                workspaceptr = workspaceptr.add(1);
                elemptr = elemptr.add(1);
            }
            elemr += 1;
        }

        // Perform the DCT
        if let Some(dct_fn) = do_dct {
            dct_fn(&mut workspace[0]);
        }

        // Quantize/descale the coefficients, and store into coef_blocks[]
        let output_ptr = *coef_blocks.add(bi as usize);
        let mut i = 0;
        while i < DCTSIZE2 {
            // Apply the quantization and scaling factor
            let temp = workspace[i] * *divisors.add(i);
            // Round to nearest integer.
            // Since C does not specify the direction of rounding for negative
            // quotients, we have to force the dividend positive for portability.
            // The maximum coefficient size is +-16K (for 12-bit data), so this
            // code should work for either 16-bit or 32-bit ints.
            *output_ptr.add(i) = ((temp + 16384.5) as c_int - 16384) as JCOEF;
            i += 1;
        }

        bi += 1;
    }
}

/*
 * Initialize FDCT manager.
 */

pub unsafe extern "C" fn jinit_forward_dct(cinfo: j_compress_ptr) {
    let fdct: my_fdct_ptr;
    let mut i: c_int;

    fdct = jpeg_alloc_small(
        cinfo,
        JPOOL_IMAGE,
        core::mem::size_of::<my_fdct_controller>(),
    ) as my_fdct_ptr;

    (*cinfo).fdct = fdct as *mut jpeg_forward_dct;
    (*fdct).pub_.start_pass = Some(start_pass_fdctmgr);

    let dct_method = (*cinfo).dct_method;
    match dct_method {
        // JDCT_ISLOW
        0 => {
            (*fdct).pub_.forward_DCT = None; // forward_DCT - disabled in original
            (*fdct).do_dct = Some(jpeg_fdct_islow);
        }
        // JDCT_IFAST
        1 => {
            (*fdct).pub_.forward_DCT = None; // forward_DCT - disabled in original
            (*fdct).do_dct = Some(jpeg_fdct_ifast);
        }
        // JDCT_FLOAT
        2 => {
            #[cfg(feature = "DCT_FLOAT_SUPPORTED")]
            {
                (*fdct).pub_.forward_DCT = Some(forward_DCT_float);
                (*fdct).do_float_dct = Some(jpeg_fdct_float);
            }
        }
        _ => {
            jpeg_errexit(cinfo, 0); // JERR_NOT_COMPILED
        }
    }

    // Mark divisor tables unallocated
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
