/*
 * jidctflt.c
 *
 * Copyright (C) 1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains a floating-point implementation of the
 * inverse DCT (Discrete Cosine Transform).  In the IJG code, this routine
 * must also perform dequantization of the input coefficients.
 *
 * This implementation should be more accurate than either of the integer
 * IDCT implementations.  However, it may not give the same results on all
 * machines because of differences in roundoff behavior.  Speed will depend
 * on the hardware's floating point capacity.
 *
 * A 2-D IDCT can be done by 1-D IDCT on each column followed by 1-D IDCT
 * on each row (or vice versa, but it's more convenient to emit a row at
 * a time).  Direct algorithms are also available, but they are much more
 * complex and seem not to be any faster when reduced to code.
 *
 * This implementation is based on Arai, Agui, and Nakajima's algorithm for
 * scaled DCT.  Their original paper (Trans. IEICE E-71(11):1095) is in
 * Japanese, but the algorithm is described in the Pennebaker & Mitchell
 * JPEG textbook (see REFERENCES section in file README).  The following code
 * is based directly on figure 4-8 in P&M.
 * While an 8-point DCT cannot be done in less than 11 multiplies, it is
 * possible to arrange the computation so that many of the multiplies are
 * simple scalings of the final outputs.  These multiplies can then be
 * folded into the multiplications or divisions by the JPEG quantization
 * table entries.  The AA&N method leaves only 5 multiplies and 29 adds
 * to be done in the DCT itself.
 * The primary disadvantage of this method is that with a fixed-point
 * implementation, accuracy is lost due to imprecise representation of the
 * scaled quantization values.  However, that problem does not arise if
 * we use floating point arithmetic.
 */

#![allow(non_snake_case)]

use core::ffi::c_int;

/* ============================================================================
 * Type stubs for structural coherence (JPEG-6 library types)
 * ============================================================================ */

pub type JDIMENSION = u32;
pub type JSAMPLE = u8;
pub type JCOEF = i16;
pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JCOEFPTR = *mut JCOEF;
pub type FAST_FLOAT = f32;
pub type FLOAT_MULT_TYPE = FAST_FLOAT;

#[repr(C)]
pub struct jpeg_component_info {
    pub dct_table: *mut FLOAT_MULT_TYPE,
}

#[repr(C)]
pub struct j_decompress_struct {
    pub sample_range_limit: *mut JSAMPLE,
}

pub type j_decompress_ptr = *mut j_decompress_struct;

/* ============================================================================
 * Constants
 * ============================================================================ */

const DCTSIZE: usize = 8;           /* The basic DCT block is 8x8 samples */
const DCTSIZE2: usize = 64;         /* DCTSIZE squared; # of elements in a block */
const MAXJSAMPLE: c_int = 255;
const CENTERJSAMPLE: c_int = 128;
const RANGE_MASK: c_int = (MAXJSAMPLE * 4 + 3); /* 2 bits wider than legal samples */

/* ============================================================================
 * Macros
 * ============================================================================ */

/* Dequantize a coefficient by multiplying it by the multiplier-table
 * entry; produce a float result.
 */
#[inline]
fn DEQUANTIZE(coef: JCOEF, quantval: FLOAT_MULT_TYPE) -> FAST_FLOAT {
    (coef as FAST_FLOAT) * quantval
}

/* RIGHT_SHIFT provides a proper signed right shift of an INT32 quantity.
 * On this configuration (RIGHT_SHIFT_IS_UNSIGNED is not defined),
 * we use simple arithmetic right shift.
 */
#[inline]
fn RIGHT_SHIFT(x: i32, shft: u32) -> i32 {
    x >> shft
}

/* Descale and correctly round an INT32 value that's scaled by N bits. */
#[inline]
fn DESCALE(x: i32, n: u32) -> i32 {
    RIGHT_SHIFT(x + (1i32 << ((n as i32) - 1)), n)
}

/* IDCT_range_limit extracts the range-limited table from cinfo.
 * This points to a lookup table initialized to map input values
 * (after descaling) into the range 0..MAXJSAMPLE.
 */
#[inline]
unsafe fn IDCT_range_limit(cinfo: j_decompress_ptr) -> *mut JSAMPLE {
    (*cinfo).sample_range_limit.add(CENTERJSAMPLE as usize)
}

/* ============================================================================
 * Perform dequantization and inverse DCT on one block of coefficients.
 * ============================================================================ */

pub unsafe fn jpeg_idct_float(
    cinfo: j_decompress_ptr,
    compptr: *mut jpeg_component_info,
    coef_block: JCOEFPTR,
    output_buf: JSAMPARRAY,
    output_col: JDIMENSION,
) {
    let mut tmp0: FAST_FLOAT;
    let mut tmp1: FAST_FLOAT;
    let mut tmp2: FAST_FLOAT;
    let mut tmp3: FAST_FLOAT;
    let mut tmp4: FAST_FLOAT;
    let mut tmp5: FAST_FLOAT;
    let mut tmp6: FAST_FLOAT;
    let mut tmp7: FAST_FLOAT;
    let mut tmp10: FAST_FLOAT;
    let mut tmp11: FAST_FLOAT;
    let mut tmp12: FAST_FLOAT;
    let mut tmp13: FAST_FLOAT;
    let mut z5: FAST_FLOAT;
    let mut z10: FAST_FLOAT;
    let mut z11: FAST_FLOAT;
    let mut z12: FAST_FLOAT;
    let mut z13: FAST_FLOAT;

    let mut inptr: JCOEFPTR;
    let mut quantptr: *mut FLOAT_MULT_TYPE;
    let mut wsptr: *mut FAST_FLOAT;
    let mut outptr: JSAMPROW;
    let range_limit: *mut JSAMPLE = IDCT_range_limit(cinfo);
    let mut ctr: c_int;
    let mut workspace: [FAST_FLOAT; DCTSIZE2] = [0.0; DCTSIZE2]; /* buffers data between passes */

    /* Pass 1: process columns from input, store into work array. */

    inptr = coef_block;
    quantptr = (*compptr).dct_table as *mut FLOAT_MULT_TYPE;
    wsptr = workspace.as_mut_ptr();
    ctr = DCTSIZE as c_int;
    while ctr > 0 {
        /* Due to quantization, we will usually find that many of the input
         * coefficients are zero, especially the AC terms.  We can exploit this
         * by short-circuiting the IDCT calculation for any column in which all
         * the AC terms are zero.  In that case each output is equal to the
         * DC coefficient (with scale factor as needed).
         * With typical images and quantization tables, half or more of the
         * column DCT calculations can be simplified this way.
         */

        if ((*inptr.add(DCTSIZE * 1) as c_int)
            | (*inptr.add(DCTSIZE * 2) as c_int)
            | (*inptr.add(DCTSIZE * 3) as c_int)
            | (*inptr.add(DCTSIZE * 4) as c_int)
            | (*inptr.add(DCTSIZE * 5) as c_int)
            | (*inptr.add(DCTSIZE * 6) as c_int)
            | (*inptr.add(DCTSIZE * 7) as c_int))
            == 0
        {
            /* AC terms all zero */
            let dcval: FAST_FLOAT = DEQUANTIZE(*inptr.add(DCTSIZE * 0), *quantptr.add(DCTSIZE * 0));

            *wsptr.add(DCTSIZE * 0) = dcval;
            *wsptr.add(DCTSIZE * 1) = dcval;
            *wsptr.add(DCTSIZE * 2) = dcval;
            *wsptr.add(DCTSIZE * 3) = dcval;
            *wsptr.add(DCTSIZE * 4) = dcval;
            *wsptr.add(DCTSIZE * 5) = dcval;
            *wsptr.add(DCTSIZE * 6) = dcval;
            *wsptr.add(DCTSIZE * 7) = dcval;

            inptr = inptr.add(1);         /* advance pointers to next column */
            quantptr = quantptr.add(1);
            wsptr = wsptr.add(1);
            ctr -= 1;
            continue;
        }

        /* Even part */

        tmp0 = DEQUANTIZE(*inptr.add(DCTSIZE * 0), *quantptr.add(DCTSIZE * 0));
        tmp1 = DEQUANTIZE(*inptr.add(DCTSIZE * 2), *quantptr.add(DCTSIZE * 2));
        tmp2 = DEQUANTIZE(*inptr.add(DCTSIZE * 4), *quantptr.add(DCTSIZE * 4));
        tmp3 = DEQUANTIZE(*inptr.add(DCTSIZE * 6), *quantptr.add(DCTSIZE * 6));

        tmp10 = tmp0 + tmp2; /* phase 3 */
        tmp11 = tmp0 - tmp2;

        tmp13 = tmp1 + tmp3; /* phases 5-3 */
        tmp12 = (tmp1 - tmp3) * (1.414213562 as FAST_FLOAT) - tmp13; /* 2*c4 */

        tmp0 = tmp10 + tmp13; /* phase 2 */
        tmp3 = tmp10 - tmp13;
        tmp1 = tmp11 + tmp12;
        tmp2 = tmp11 - tmp12;

        /* Odd part */

        tmp4 = DEQUANTIZE(*inptr.add(DCTSIZE * 1), *quantptr.add(DCTSIZE * 1));
        tmp5 = DEQUANTIZE(*inptr.add(DCTSIZE * 3), *quantptr.add(DCTSIZE * 3));
        tmp6 = DEQUANTIZE(*inptr.add(DCTSIZE * 5), *quantptr.add(DCTSIZE * 5));
        tmp7 = DEQUANTIZE(*inptr.add(DCTSIZE * 7), *quantptr.add(DCTSIZE * 7));

        z13 = tmp6 + tmp5; /* phase 6 */
        z10 = tmp6 - tmp5;
        z11 = tmp4 + tmp7;
        z12 = tmp4 - tmp7;

        tmp7 = z11 + z13; /* phase 5 */
        tmp11 = (z11 - z13) * (1.414213562 as FAST_FLOAT); /* 2*c4 */

        z5 = (z10 + z12) * (1.847759065 as FAST_FLOAT); /* 2*c2 */
        tmp10 = (1.082392200 as FAST_FLOAT) * z12 - z5; /* 2*(c2-c6) */
        tmp12 = (-2.613125930 as FAST_FLOAT) * z10 + z5; /* -2*(c2+c6) */

        tmp6 = tmp12 - tmp7; /* phase 2 */
        tmp5 = tmp11 - tmp6;
        tmp4 = tmp10 + tmp5;

        *wsptr.add(DCTSIZE * 0) = tmp0 + tmp7;
        *wsptr.add(DCTSIZE * 7) = tmp0 - tmp7;
        *wsptr.add(DCTSIZE * 1) = tmp1 + tmp6;
        *wsptr.add(DCTSIZE * 6) = tmp1 - tmp6;
        *wsptr.add(DCTSIZE * 2) = tmp2 + tmp5;
        *wsptr.add(DCTSIZE * 5) = tmp2 - tmp5;
        *wsptr.add(DCTSIZE * 4) = tmp3 + tmp4;
        *wsptr.add(DCTSIZE * 3) = tmp3 - tmp4;

        inptr = inptr.add(1);         /* advance pointers to next column */
        quantptr = quantptr.add(1);
        wsptr = wsptr.add(1);
        ctr -= 1;
    }

    /* Pass 2: process rows from work array, store into output array. */
    /* Note that we must descale the results by a factor of 8 == 2**3. */

    wsptr = workspace.as_mut_ptr();
    ctr = 0;
    while ctr < DCTSIZE as c_int {
        outptr = *output_buf.add(ctr as usize);
        outptr = outptr.add(output_col as usize);
        /* Rows of zeroes can be exploited in the same way as we did with columns.
         * However, the column calculation has created many nonzero AC terms, so
         * the simplification applies less often (typically 5% to 10% of the time).
         * And testing floats for zero is relatively expensive, so we don't bother.
         */

        /* Even part */

        tmp10 = *wsptr.add(0) + *wsptr.add(4);
        tmp11 = *wsptr.add(0) - *wsptr.add(4);

        tmp13 = *wsptr.add(2) + *wsptr.add(6);
        tmp12 = (*wsptr.add(2) - *wsptr.add(6)) * (1.414213562 as FAST_FLOAT) - tmp13;

        tmp0 = tmp10 + tmp13;
        tmp3 = tmp10 - tmp13;
        tmp1 = tmp11 + tmp12;
        tmp2 = tmp11 - tmp12;

        /* Odd part */

        z13 = *wsptr.add(5) + *wsptr.add(3);
        z10 = *wsptr.add(5) - *wsptr.add(3);
        z11 = *wsptr.add(1) + *wsptr.add(7);
        z12 = *wsptr.add(1) - *wsptr.add(7);

        tmp7 = z11 + z13;
        tmp11 = (z11 - z13) * (1.414213562 as FAST_FLOAT);

        z5 = (z10 + z12) * (1.847759065 as FAST_FLOAT); /* 2*c2 */
        tmp10 = (1.082392200 as FAST_FLOAT) * z12 - z5; /* 2*(c2-c6) */
        tmp12 = (-2.613125930 as FAST_FLOAT) * z10 + z5; /* -2*(c2+c6) */

        tmp6 = tmp12 - tmp7;
        tmp5 = tmp11 - tmp6;
        tmp4 = tmp10 + tmp5;

        /* Final output stage: scale down by a factor of 8 and range-limit */

        *outptr.add(0) = *range_limit.add(
            (DESCALE((tmp0 + tmp7) as i32, 3) & RANGE_MASK) as usize,
        );
        *outptr.add(7) = *range_limit.add(
            (DESCALE((tmp0 - tmp7) as i32, 3) & RANGE_MASK) as usize,
        );
        *outptr.add(1) = *range_limit.add(
            (DESCALE((tmp1 + tmp6) as i32, 3) & RANGE_MASK) as usize,
        );
        *outptr.add(6) = *range_limit.add(
            (DESCALE((tmp1 - tmp6) as i32, 3) & RANGE_MASK) as usize,
        );
        *outptr.add(2) = *range_limit.add(
            (DESCALE((tmp2 + tmp5) as i32, 3) & RANGE_MASK) as usize,
        );
        *outptr.add(5) = *range_limit.add(
            (DESCALE((tmp2 - tmp5) as i32, 3) & RANGE_MASK) as usize,
        );
        *outptr.add(4) = *range_limit.add(
            (DESCALE((tmp3 + tmp4) as i32, 3) & RANGE_MASK) as usize,
        );
        *outptr.add(3) = *range_limit.add(
            (DESCALE((tmp3 - tmp4) as i32, 3) & RANGE_MASK) as usize,
        );

        wsptr = wsptr.add(DCTSIZE);    /* advance pointer to next row */
        ctr += 1;
    }
}
