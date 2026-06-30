#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_mut,
    unused_variables,
    dead_code,
    unused_imports
)]

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
use crate::code::server::exe_headers_h::*;

// #define JPEG_INTERNALS
use crate::code::jpeg_6::jinclude_h::*;
use crate::code::jpeg_6::jpeglib_h::*;
use crate::code::jpeg_6::jdct_h::*; /* Private declarations for DCT subsystem */
use crate::code::jpeg_6::jpegint_h::*;
use crate::code::jpeg_6::jmorecfg_h::*;


/* Private subobject for this module */

#[repr(C)]
pub struct my_fdct_controller {
    pub pub_: jpeg_forward_dct,   /* public fields */
                                  /* "pub" renamed to pub_ — Rust keyword */

    /* Pointer to the DCT routine actually in use */
    pub do_dct: forward_DCT_method_ptr,

    /* The actual post-DCT divisors --- not identical to the quant table
     * entries, because of scaling (especially for an unnormalized DCT).
     * Each table is given in normal array order; note that this must
     * be converted from the zigzag order of the quantization tables.
     */
    pub divisors: [*mut DCTELEM; NUM_QUANT_TBLS as usize],

    /* Same as above for the floating-point case. */
    #[cfg(feature = "dct_float_supported")]
    pub do_float_dct: float_DCT_method_ptr,
    #[cfg(feature = "dct_float_supported")]
    pub float_divisors: [*mut FAST_FLOAT; NUM_QUANT_TBLS as usize],
}

pub type my_fdct_ptr = *mut my_fdct_controller;


/*
 * Initialize for a processing pass.
 * Verify that all referenced Q-tables are present, and set up
 * the divisor table for each one.
 * In the current implementation, DCT of all components is done during
 * the first pass, even if only some components will be output in the
 * first scan.  Hence all components should be examined here.
 */

unsafe fn start_pass_fdctmgr(cinfo: j_compress_ptr) {
    let fdct: my_fdct_ptr = (*cinfo).fdct as my_fdct_ptr;
    let mut ci: i32;
    let mut qtblno: i32;
    let mut i: i32;
    let mut compptr: *mut jpeg_component_info;
    let mut qtbl: *mut JQUANT_TBL;
    /* Note: C declares `dtbl` at function scope under #ifdef DCT_ISLOW_SUPPORTED;
     * in Rust it is declared locally in each match arm that uses it. */

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        qtblno = (*compptr).quant_tbl_no;
        /* Make sure specified quantization table is present */
        if qtblno < 0
            || qtblno >= NUM_QUANT_TBLS as i32
            || (*cinfo).quant_tbl_ptrs[qtblno as usize].is_null()
        {
            ERREXIT1!(cinfo, JERR_NO_QUANT_TABLE, qtblno);
        }
        qtbl = (*cinfo).quant_tbl_ptrs[qtblno as usize];
        /* Compute divisors for this quant table */
        /* We may do this more than once for same table, but it's not a big deal */
        match (*cinfo).dct_method {
            #[cfg(feature = "dct_islow_supported")]
            JDCT_ISLOW => {
                /* For LL&M IDCT method, divisors are equal to raw quantization
                 * coefficients multiplied by 8 (to counteract scaling).
                 */
                if (*fdct).divisors[qtblno as usize].is_null() {
                    (*fdct).divisors[qtblno as usize] = ((*(*cinfo).mem).alloc_small)(
                        cinfo as j_common_ptr,
                        JPOOL_IMAGE,
                        DCTSIZE2 as usize * SIZEOF!(DCTELEM),
                    ) as *mut DCTELEM;
                }
                let dtbl: *mut DCTELEM = (*fdct).divisors[qtblno as usize];
                i = 0;
                while i < DCTSIZE2 as i32 {
                    *dtbl.offset(i as isize) =
                        ((*qtbl).quantval[jpeg_zigzag_order[i as usize] as usize] as DCTELEM) << 3;
                    i += 1;
                }
            }
            #[cfg(feature = "dct_ifast_supported")]
            JDCT_IFAST => {
                {
                    /* For AA&N IDCT method, divisors are equal to quantization
                     * coefficients scaled by scalefactor[row]*scalefactor[col], where
                     *   scalefactor[0] = 1
                     *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                     * We apply a further scale factor of 8.
                     */
                    const CONST_BITS: i32 = 14;
                    static aanscales: [INT16; DCTSIZE2 as usize] = [
                        /* precomputed values scaled up by 14 bits: in natural order */
                        16384, 22725, 21407, 19266, 16384, 12873,  8867,  4520,
                        22725, 31521, 29692, 26722, 22725, 17855, 12299,  6270,
                        21407, 29692, 27969, 25172, 21407, 16819, 11585,  5906,
                        19266, 26722, 25172, 22654, 19266, 15137, 10426,  5315,
                        16384, 22725, 21407, 19266, 16384, 12873,  8867,  4520,
                        12873, 17855, 16819, 15137, 12873, 10114,  6967,  3552,
                         8867, 12299, 11585, 10426,  8867,  6967,  4799,  2446,
                         4520,  6270,  5906,  5315,  4520,  3552,  2446,  1247
                    ];
                    SHIFT_TEMPS!();

                    if (*fdct).divisors[qtblno as usize].is_null() {
                        (*fdct).divisors[qtblno as usize] = ((*(*cinfo).mem).alloc_small)(
                            cinfo as j_common_ptr,
                            JPOOL_IMAGE,
                            DCTSIZE2 as usize * SIZEOF!(DCTELEM),
                        ) as *mut DCTELEM;
                    }
                    let dtbl: *mut DCTELEM = (*fdct).divisors[qtblno as usize];
                    i = 0;
                    while i < DCTSIZE2 as i32 {
                        *dtbl.offset(i as isize) = DESCALE!(
                            MULTIPLY16V16!(
                                (*qtbl).quantval[jpeg_zigzag_order[i as usize] as usize] as INT32,
                                aanscales[i as usize] as INT32
                            ),
                            CONST_BITS - 3
                        ) as DCTELEM;
                        i += 1;
                    }
                }
            }
            #[cfg(feature = "dct_float_supported")]
            JDCT_FLOAT => {
                {
                    /* For float AA&N IDCT method, divisors are equal to quantization
                     * coefficients scaled by scalefactor[row]*scalefactor[col], where
                     *   scalefactor[0] = 1
                     *   scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
                     * We apply a further scale factor of 8.
                     * What's actually stored is 1/divisor so that the inner loop can
                     * use a multiplication rather than a division.
                     */
                    let mut fdtbl: *mut FAST_FLOAT;
                    let mut row: i32;
                    let mut col: i32;
                    static aanscalefactor: [f64; DCTSIZE as usize] = [
                        1.0, 1.387039845, 1.306562965, 1.175875602,
                        1.0, 0.785694958, 0.541196100, 0.275899379
                    ];

                    if (*fdct).float_divisors[qtblno as usize].is_null() {
                        (*fdct).float_divisors[qtblno as usize] = ((*(*cinfo).mem).alloc_small)(
                            cinfo as j_common_ptr,
                            JPOOL_IMAGE,
                            DCTSIZE2 as usize * SIZEOF!(FAST_FLOAT),
                        ) as *mut FAST_FLOAT;
                    }
                    fdtbl = (*fdct).float_divisors[qtblno as usize];
                    i = 0;
                    row = 0;
                    while row < DCTSIZE as i32 {
                        col = 0;
                        while col < DCTSIZE as i32 {
                            *fdtbl.offset(i as isize) = (1.0
                                / (((*qtbl).quantval[jpeg_zigzag_order[i as usize] as usize]
                                    as f64)
                                    * aanscalefactor[row as usize]
                                    * aanscalefactor[col as usize]
                                    * 8.0)) as FAST_FLOAT;
                            i += 1;
                            col += 1;
                        }
                        row += 1;
                    }
                }
            }
            _ => {
                ERREXIT!(cinfo, JERR_NOT_COMPILED);
            }
        }
        ci += 1;
        compptr = compptr.offset(1);
    }
}


/*
 * Perform forward DCT on one or more blocks of a component.
 *
 * The input samples are taken from the sample_data[] array starting at
 * position start_row/start_col, and moving to the right for any additional
 * blocks. The quantized coefficients are returned in coef_blocks[].
 */

/* #if 0 // bk001204
METHODDEF void
forward_DCT (j_compress_ptr cinfo, jpeg_component_info * compptr,
	     JSAMPARRAY sample_data, JBLOCKROW coef_blocks,
	     JDIMENSION start_row, JDIMENSION start_col,
	     JDIMENSION num_blocks)
/* This version is used for integer DCT implementations. */
{
  /* This routine is heavily used, so it's worth coding it tightly. */
  my_fdct_ptr fdct = (my_fdct_ptr) cinfo->fdct;
  forward_DCT_method_ptr do_dct = fdct->do_dct;
  DCTELEM * divisors = fdct->divisors[compptr->quant_tbl_no];
  DCTELEM workspace[DCTSIZE2];	/* work area for FDCT subroutine */
  JDIMENSION bi;

  sample_data += start_row;	/* fold in the vertical offset once */

  for (bi = 0; bi < num_blocks; bi++, start_col += DCTSIZE) {
    /* Load data into workspace, applying unsigned->signed conversion */
    { register DCTELEM *workspaceptr;
      register JSAMPROW elemptr;
      register int elemr;

      workspaceptr = workspace;
      for (elemr = 0; elemr < DCTSIZE; elemr++) {
	elemptr = sample_data[elemr] + start_col;
/* #if DCTSIZE == 8		unroll the inner loop
	*workspaceptr++ = GETJSAMPLE(*elemptr++) - CENTERJSAMPLE;
	*workspaceptr++ = GETJSAMPLE(*elemptr++) - CENTERJSAMPLE;
	*workspaceptr++ = GETJSAMPLE(*elemptr++) - CENTERJSAMPLE;
	*workspaceptr++ = GETJSAMPLE(*elemptr++) - CENTERJSAMPLE;
	*workspaceptr++ = GETJSAMPLE(*elemptr++) - CENTERJSAMPLE;
	*workspaceptr++ = GETJSAMPLE(*elemptr++) - CENTERJSAMPLE;
	*workspaceptr++ = GETJSAMPLE(*elemptr++) - CENTERJSAMPLE;
	*workspaceptr++ = GETJSAMPLE(*elemptr++) - CENTERJSAMPLE;
#else
	{ register int elemc;
	  for (elemc = DCTSIZE; elemc > 0; elemc--) {
	    *workspaceptr++ = GETJSAMPLE(*elemptr++) - CENTERJSAMPLE;
	  }
	}
#endif */
      }
    }

    /* Perform the DCT */
    (*do_dct) (workspace);

    /* Quantize/descale the coefficients, and store into coef_blocks[] */
    { register DCTELEM temp, qval;
      register int i;
      register JCOEFPTR output_ptr = coef_blocks[bi];

      for (i = 0; i < DCTSIZE2; i++) {
	qval = divisors[i];
	temp = workspace[i];
	/* Divide the coefficient value by qval, ensuring proper rounding.
	 * Since C does not specify the direction of rounding for negative
	 * quotients, we have to force the dividend positive for portability.
	 *
	 * In most files, at least half of the output values will be zero
	 * (at default quantization settings, more like three-quarters...)
	 * so we should ensure that this case is fast.  On many machines,
	 * a comparison is enough cheaper than a divide to make a special test
	 * a win.  Since both inputs will be nonnegative, we need only test
	 * for a < b to discover whether a/b is 0.
	 * If your machine's division is fast enough, define FAST_DIVIDE.
	 */
/* ifdef FAST_DIVIDE
define DIVIDE_BY(a,b)	a /= b
else
define DIVIDE_BY(a,b)	if (a >= b) a /= b; else a = 0
endif */
	if (temp < 0) {
	  temp = -temp;
	  temp += qval>>1;	/* for rounding */
	  DIVIDE_BY(temp, qval);
	  temp = -temp;
	} else {
	  temp += qval>>1;	/* for rounding */
	  DIVIDE_BY(temp, qval);
	}
	output_ptr[i] = (JCOEF) temp;
      }
    }
  }
}
#endif // 0 */


#[cfg(feature = "dct_float_supported")]
unsafe fn forward_DCT_float(
    cinfo: j_compress_ptr,
    compptr: *mut jpeg_component_info,
    mut sample_data: JSAMPARRAY,
    coef_blocks: JBLOCKROW,
    start_row: JDIMENSION,
    mut start_col: JDIMENSION,
    num_blocks: JDIMENSION,
)
/* This version is used for floating-point DCT implementations. */
{
    /* This routine is heavily used, so it's worth coding it tightly. */
    let fdct: my_fdct_ptr = (*cinfo).fdct as my_fdct_ptr;
    let do_dct: float_DCT_method_ptr = (*fdct).do_float_dct;
    let divisors: *mut FAST_FLOAT =
        (*fdct).float_divisors[(*compptr).quant_tbl_no as usize];
    let mut workspace: [FAST_FLOAT; DCTSIZE2 as usize] =
        [0 as FAST_FLOAT; DCTSIZE2 as usize]; /* work area for FDCT subroutine */
    let mut bi: JDIMENSION;

    sample_data = sample_data.offset(start_row as isize); /* fold in the vertical offset once */

    bi = 0;
    while bi < num_blocks {
        /* Load data into workspace, applying unsigned->signed conversion */
        {
            let mut workspaceptr: *mut FAST_FLOAT;
            let mut elemptr: JSAMPROW;
            let mut elemr: i32;

            workspaceptr = workspace.as_mut_ptr();
            elemr = 0;
            while elemr < DCTSIZE as i32 {
                elemptr = (*sample_data.offset(elemr as isize)).offset(start_col as isize);
                if DCTSIZE == 8 {
                    /* unroll the inner loop */
                    *workspaceptr = (GETJSAMPLE!(*elemptr) - CENTERJSAMPLE) as FAST_FLOAT;
                    workspaceptr = workspaceptr.offset(1); elemptr = elemptr.offset(1);
                    *workspaceptr = (GETJSAMPLE!(*elemptr) - CENTERJSAMPLE) as FAST_FLOAT;
                    workspaceptr = workspaceptr.offset(1); elemptr = elemptr.offset(1);
                    *workspaceptr = (GETJSAMPLE!(*elemptr) - CENTERJSAMPLE) as FAST_FLOAT;
                    workspaceptr = workspaceptr.offset(1); elemptr = elemptr.offset(1);
                    *workspaceptr = (GETJSAMPLE!(*elemptr) - CENTERJSAMPLE) as FAST_FLOAT;
                    workspaceptr = workspaceptr.offset(1); elemptr = elemptr.offset(1);
                    *workspaceptr = (GETJSAMPLE!(*elemptr) - CENTERJSAMPLE) as FAST_FLOAT;
                    workspaceptr = workspaceptr.offset(1); elemptr = elemptr.offset(1);
                    *workspaceptr = (GETJSAMPLE!(*elemptr) - CENTERJSAMPLE) as FAST_FLOAT;
                    workspaceptr = workspaceptr.offset(1); elemptr = elemptr.offset(1);
                    *workspaceptr = (GETJSAMPLE!(*elemptr) - CENTERJSAMPLE) as FAST_FLOAT;
                    workspaceptr = workspaceptr.offset(1); elemptr = elemptr.offset(1);
                    *workspaceptr = (GETJSAMPLE!(*elemptr) - CENTERJSAMPLE) as FAST_FLOAT;
                    workspaceptr = workspaceptr.offset(1);
                    /* elemptr not advanced after last store; matches C source */
                } else {
                    let mut elemc: i32;
                    elemc = DCTSIZE as i32;
                    while elemc > 0 {
                        *workspaceptr = (GETJSAMPLE!(*elemptr) - CENTERJSAMPLE) as FAST_FLOAT;
                        workspaceptr = workspaceptr.offset(1);
                        elemptr = elemptr.offset(1);
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
            let mut i: i32;
            let output_ptr: JCOEFPTR =
                coef_blocks.offset(bi as isize) as JCOEFPTR;

            i = 0;
            while i < DCTSIZE2 as i32 {
                /* Apply the quantization and scaling factor */
                temp = *workspace.as_ptr().offset(i as isize)
                    * *divisors.offset(i as isize);
                /* Round to nearest integer.
                 * Since C does not specify the direction of rounding for negative
                 * quotients, we have to force the dividend positive for portability.
                 * The maximum coefficient size is +-16K (for 12-bit data), so this
                 * code should work for either 16-bit or 32-bit ints.
                 */
                *output_ptr.offset(i as isize) =
                    ((temp + 16384.5f64 as FAST_FLOAT) as i32 - 16384) as JCOEF;
                i += 1;
            }
        }
        bi = bi.wrapping_add(1);
        start_col = start_col.wrapping_add(DCTSIZE as JDIMENSION);
    }
}


/*
 * Initialize FDCT manager.
 */

pub unsafe fn jinit_forward_dct(cinfo: j_compress_ptr) {
    let mut fdct: my_fdct_ptr;
    let mut i: i32;

    fdct = ((*(*cinfo).mem).alloc_small)(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        SIZEOF!(my_fdct_controller),
    ) as my_fdct_ptr;
    (*cinfo).fdct = fdct as *mut jpeg_forward_dct;
    (*fdct).pub_.start_pass = start_pass_fdctmgr;

    match (*cinfo).dct_method {
        #[cfg(feature = "dct_islow_supported")]
        JDCT_ISLOW => {
            (*fdct).pub_.forward_DCT = forward_DCT;
            (*fdct).do_dct = jpeg_fdct_islow;
        }
        #[cfg(feature = "dct_ifast_supported")]
        JDCT_IFAST => {
            (*fdct).pub_.forward_DCT = forward_DCT;
            (*fdct).do_dct = jpeg_fdct_ifast;
        }
        #[cfg(feature = "dct_float_supported")]
        JDCT_FLOAT => {
            (*fdct).pub_.forward_DCT = forward_DCT_float;
            (*fdct).do_float_dct = jpeg_fdct_float;
        }
        _ => {
            ERREXIT!(cinfo, JERR_NOT_COMPILED);
        }
    }

    /* Mark divisor tables unallocated */
    i = 0;
    while i < NUM_QUANT_TBLS as i32 {
        (*fdct).divisors[i as usize] = core::ptr::null_mut();
        #[cfg(feature = "dct_float_supported")]
        {
            (*fdct).float_divisors[i as usize] = core::ptr::null_mut();
        }
        i += 1;
    }
}
