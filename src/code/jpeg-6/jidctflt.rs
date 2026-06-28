#![allow(non_snake_case)]

use core::ffi::{c_int, c_short, c_uint, c_void};

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

const DCTSIZE: c_int = 8;
const DCTSIZE2: c_int = 64;
const CENTERJSAMPLE: c_int = 128;
const MAXJSAMPLE: c_int = 255;
const RANGE_MASK: c_int = 1023; /* 2 bits wider than legal samples */

/* Dequantize a coefficient by multiplying it by the multiplier-table
 * entry; produce a float result.
 */
#[inline(always)]
fn DEQUANTIZE(coef: c_short, quantval: f32) -> f32 {
    (coef as f32) * quantval
}

/* Descale and correctly round an INT32 value that's scaled by N bits.
 * We assume RIGHT_SHIFT rounds towards minus infinity, so adding
 * the fudge factor is correct for either sign of X.
 */
#[inline(always)]
fn DESCALE(x: i32, n: c_int) -> i32 {
    ((x + (1 << ((n) - 1))) >> (n))
}

/*
 * Perform dequantization and inverse DCT on one block of coefficients.
 */
#[no_mangle]
pub unsafe extern "C" fn jpeg_idct_float(
    cinfo: *mut c_void,
    compptr: *mut c_void,
    coef_block: *mut c_short,
    output_buf: *mut *mut u8,
    output_col: c_uint,
) {
    let mut tmp0: f32;
    let mut tmp1: f32;
    let mut tmp2: f32;
    let mut tmp3: f32;
    let mut tmp4: f32;
    let mut tmp5: f32;
    let mut tmp6: f32;
    let mut tmp7: f32;
    let mut tmp10: f32;
    let mut tmp11: f32;
    let mut tmp12: f32;
    let mut tmp13: f32;
    let mut z5: f32;
    let mut z10: f32;
    let mut z11: f32;
    let mut z12: f32;
    let mut z13: f32;
    let mut inptr: *mut c_short;
    let mut quantptr: *mut f32;
    let mut wsptr: *mut f32;
    let mut outptr: *mut u8;
    let mut workspace: [f32; 64] = [0.0; 64]; /* buffers data between passes */
    let mut ctr: c_int;

    /* Cast cinfo to access sample_range_limit */
    let range_limit = if !cinfo.is_null() {
        /* The sample_range_limit pointer offset from cinfo struct */
        /* IDCT_range_limit(cinfo) = ((cinfo)->sample_range_limit + CENTERJSAMPLE) */
        let cinfo_stub = cinfo as *mut j_decompress_struct_stub;
        (*cinfo_stub).sample_range_limit.add(CENTERJSAMPLE as usize)
    } else {
        core::ptr::null_mut()
    };

    /* Get dct_table pointer from compptr */
    let compptr_stub = compptr as *mut jpeg_component_info_stub;
    quantptr = (*compptr_stub).dct_table as *mut f32;

    /* Pass 1: process columns from input, store into work array. */

    inptr = coef_block;
    wsptr = workspace.as_mut_ptr();
    ctr = DCTSIZE;
    while ctr > 0 {
        /* Due to quantization, we will usually find that many of the input
         * coefficients are zero, especially the AC terms.  We can exploit this
         * by short-circuiting the IDCT calculation for any column in which all
         * the AC terms are zero.  In that case each output is equal to the
         * DC coefficient (with scale factor as needed).
         * With typical images and quantization tables, half or more of the
         * column DCT calculations can be simplified this way.
         */

        if ((*inptr.offset((DCTSIZE * 1) as isize))
            | (*inptr.offset((DCTSIZE * 2) as isize))
            | (*inptr.offset((DCTSIZE * 3) as isize))
            | (*inptr.offset((DCTSIZE * 4) as isize))
            | (*inptr.offset((DCTSIZE * 5) as isize))
            | (*inptr.offset((DCTSIZE * 6) as isize))
            | (*inptr.offset((DCTSIZE * 7) as isize)))
            == 0
        {
            /* AC terms all zero */
            let dcval = DEQUANTIZE(
                *inptr.offset((DCTSIZE * 0) as isize),
                *quantptr.offset((DCTSIZE * 0) as isize),
            );

            *wsptr.offset((DCTSIZE * 0) as isize) = dcval;
            *wsptr.offset((DCTSIZE * 1) as isize) = dcval;
            *wsptr.offset((DCTSIZE * 2) as isize) = dcval;
            *wsptr.offset((DCTSIZE * 3) as isize) = dcval;
            *wsptr.offset((DCTSIZE * 4) as isize) = dcval;
            *wsptr.offset((DCTSIZE * 5) as isize) = dcval;
            *wsptr.offset((DCTSIZE * 6) as isize) = dcval;
            *wsptr.offset((DCTSIZE * 7) as isize) = dcval;

            inptr = inptr.offset(1); /* advance pointers to next column */
            quantptr = quantptr.offset(1);
            wsptr = wsptr.offset(1);
            ctr -= 1;
            continue;
        }

        /* Even part */

        tmp0 = DEQUANTIZE(
            *inptr.offset((DCTSIZE * 0) as isize),
            *quantptr.offset((DCTSIZE * 0) as isize),
        );
        tmp1 = DEQUANTIZE(
            *inptr.offset((DCTSIZE * 2) as isize),
            *quantptr.offset((DCTSIZE * 2) as isize),
        );
        tmp2 = DEQUANTIZE(
            *inptr.offset((DCTSIZE * 4) as isize),
            *quantptr.offset((DCTSIZE * 4) as isize),
        );
        tmp3 = DEQUANTIZE(
            *inptr.offset((DCTSIZE * 6) as isize),
            *quantptr.offset((DCTSIZE * 6) as isize),
        );

        tmp10 = tmp0 + tmp2; /* phase 3 */
        tmp11 = tmp0 - tmp2;

        tmp13 = tmp1 + tmp3; /* phases 5-3 */
        tmp12 = (tmp1 - tmp3) * (1.414213562 as f32) - tmp13; /* 2*c4 */

        tmp0 = tmp10 + tmp13; /* phase 2 */
        tmp3 = tmp10 - tmp13;
        tmp1 = tmp11 + tmp12;
        tmp2 = tmp11 - tmp12;

        /* Odd part */

        tmp4 = DEQUANTIZE(
            *inptr.offset((DCTSIZE * 1) as isize),
            *quantptr.offset((DCTSIZE * 1) as isize),
        );
        tmp5 = DEQUANTIZE(
            *inptr.offset((DCTSIZE * 3) as isize),
            *quantptr.offset((DCTSIZE * 3) as isize),
        );
        tmp6 = DEQUANTIZE(
            *inptr.offset((DCTSIZE * 5) as isize),
            *quantptr.offset((DCTSIZE * 5) as isize),
        );
        tmp7 = DEQUANTIZE(
            *inptr.offset((DCTSIZE * 7) as isize),
            *quantptr.offset((DCTSIZE * 7) as isize),
        );

        z13 = tmp6 + tmp5; /* phase 6 */
        z10 = tmp6 - tmp5;
        z11 = tmp4 + tmp7;
        z12 = tmp4 - tmp7;

        tmp7 = z11 + z13; /* phase 5 */
        tmp11 = (z11 - z13) * (1.414213562 as f32); /* 2*c4 */

        z5 = (z10 + z12) * (1.847759065 as f32); /* 2*c2 */
        tmp10 = (1.082392200 as f32) * z12 - z5; /* 2*(c2-c6) */
        tmp12 = (-2.613125930 as f32) * z10 + z5; /* -2*(c2+c6) */

        tmp6 = tmp12 - tmp7; /* phase 2 */
        tmp5 = tmp11 - tmp6;
        tmp4 = tmp10 + tmp5;

        *wsptr.offset((DCTSIZE * 0) as isize) = tmp0 + tmp7;
        *wsptr.offset((DCTSIZE * 7) as isize) = tmp0 - tmp7;
        *wsptr.offset((DCTSIZE * 1) as isize) = tmp1 + tmp6;
        *wsptr.offset((DCTSIZE * 6) as isize) = tmp1 - tmp6;
        *wsptr.offset((DCTSIZE * 2) as isize) = tmp2 + tmp5;
        *wsptr.offset((DCTSIZE * 5) as isize) = tmp2 - tmp5;
        *wsptr.offset((DCTSIZE * 4) as isize) = tmp3 + tmp4;
        *wsptr.offset((DCTSIZE * 3) as isize) = tmp3 - tmp4;

        inptr = inptr.offset(1); /* advance pointers to next column */
        quantptr = quantptr.offset(1);
        wsptr = wsptr.offset(1);
        ctr -= 1;
    }

    /* Pass 2: process rows from work array, store into output array. */
    /* Note that we must descale the results by a factor of 8 == 2**3. */

    wsptr = workspace.as_mut_ptr();
    ctr = 0;
    while ctr < DCTSIZE {
        outptr = *output_buf.offset(ctr as isize);
        outptr = outptr.offset(output_col as isize);
        /* Rows of zeroes can be exploited in the same way as we did with columns.
         * However, the column calculation has created many nonzero AC terms, so
         * the simplification applies less often (typically 5% to 10% of the time).
         * And testing floats for zero is relatively expensive, so we don't bother.
         */

        /* Even part */

        tmp10 = *wsptr.offset(0) + *wsptr.offset(4);
        tmp11 = *wsptr.offset(0) - *wsptr.offset(4);

        tmp13 = *wsptr.offset(2) + *wsptr.offset(6);
        tmp12 = (*wsptr.offset(2) - *wsptr.offset(6)) * (1.414213562 as f32) - tmp13;

        tmp0 = tmp10 + tmp13;
        tmp3 = tmp10 - tmp13;
        tmp1 = tmp11 + tmp12;
        tmp2 = tmp11 - tmp12;

        /* Odd part */

        z13 = *wsptr.offset(5) + *wsptr.offset(3);
        z10 = *wsptr.offset(5) - *wsptr.offset(3);
        z11 = *wsptr.offset(1) + *wsptr.offset(7);
        z12 = *wsptr.offset(1) - *wsptr.offset(7);

        tmp7 = z11 + z13;
        tmp11 = (z11 - z13) * (1.414213562 as f32);

        z5 = (z10 + z12) * (1.847759065 as f32); /* 2*c2 */
        tmp10 = (1.082392200 as f32) * z12 - z5; /* 2*(c2-c6) */
        tmp12 = (-2.613125930 as f32) * z10 + z5; /* -2*(c2+c6) */

        tmp6 = tmp12 - tmp7;
        tmp5 = tmp11 - tmp6;
        tmp4 = tmp10 + tmp5;

        /* Final output stage: scale down by a factor of 8 and range-limit */

        *outptr.offset(0) = *range_limit.offset(
            ((DESCALE((tmp0 + tmp7) as i32, 3) as c_int) & RANGE_MASK) as isize,
        );
        *outptr.offset(7) = *range_limit.offset(
            ((DESCALE((tmp0 - tmp7) as i32, 3) as c_int) & RANGE_MASK) as isize,
        );
        *outptr.offset(1) = *range_limit.offset(
            ((DESCALE((tmp1 + tmp6) as i32, 3) as c_int) & RANGE_MASK) as isize,
        );
        *outptr.offset(6) = *range_limit.offset(
            ((DESCALE((tmp1 - tmp6) as i32, 3) as c_int) & RANGE_MASK) as isize,
        );
        *outptr.offset(2) = *range_limit.offset(
            ((DESCALE((tmp2 + tmp5) as i32, 3) as c_int) & RANGE_MASK) as isize,
        );
        *outptr.offset(5) = *range_limit.offset(
            ((DESCALE((tmp2 - tmp5) as i32, 3) as c_int) & RANGE_MASK) as isize,
        );
        *outptr.offset(4) = *range_limit.offset(
            ((DESCALE((tmp3 + tmp4) as i32, 3) as c_int) & RANGE_MASK) as isize,
        );
        *outptr.offset(3) = *range_limit.offset(
            ((DESCALE((tmp3 - tmp4) as i32, 3) as c_int) & RANGE_MASK) as isize,
        );

        wsptr = wsptr.offset(DCTSIZE as isize); /* advance pointer to next row */
        ctr += 1;
    }
}

/* Local stub types for cinfo and compptr field access.
 * These mirror the essential C struct layouts needed to access
 * sample_range_limit from the j_decompress_struct and dct_table from
 * the jpeg_component_info struct.
 *
 * SAFETY: These stubs only contain the fields we actually access via
 * casting. The actual JPEG library structs are larger; we rely on the
 * caller to provide correctly initialized pointers that match the
 * C struct layout from jpeglib.h.
 */

#[repr(C)]
struct j_decompress_struct_stub {
    err: *mut c_void,
    mem: *mut c_void,
    progress: *mut c_void,
    is_decompressor: c_int,
    global_state: c_int,
    source: *mut c_void,
    image_width: c_uint,
    image_height: c_uint,
    input_components: c_int,
    in_color_space: c_int,
    input_gamma: f64,
    data_precision: c_int,
    num_components: c_int,
    jpeg_color_space: c_int,
    comp_info: *mut c_void,
    quant_tbl_ptrs: [*mut c_void; 4],
    dc_huff_tbl_ptrs: [*mut c_void; 4],
    ac_huff_tbl_ptrs: [*mut c_void; 4],
    arith_dc_L: [u8; 16],
    arith_dc_U: [u8; 16],
    arith_ac_K: [u8; 16],
    num_scans: c_int,
    scan_info: *mut c_void,
    raw_data_in: c_int,
    arith_code: c_int,
    optimize_coding: c_int,
    CCIR601_sampling: c_int,
    smoothing_factor: c_int,
    dct_method: c_int,
    restart_interval: c_uint,
    restart_in_rows: c_int,
    write_JFIF_header: c_int,
    density_unit: u8,
    X_density: u16,
    Y_density: u16,
    write_Adobe_marker: c_int,
    next_scanline: c_uint,
    progressive_mode: c_int,
    max_h_samp_factor: c_int,
    max_v_samp_factor: c_int,
    total_iMCU_rows: c_uint,
    comps_in_scan: c_int,
    cur_comp_info: [*mut c_void; 4],
    MCUs_per_row: c_uint,
    MCU_rows_in_scan: c_uint,
    blocks_in_MCU: c_int,
    MCU_membership: [c_int; 10],
    Ss: c_int,
    Se: c_int,
    Ah: c_int,
    Al: c_int,
    /* Additional decompressor-specific fields follow, including sample_range_limit */
    sample_range_limit: *mut u8,
}

pub type j_decompress_ptr_stub = *mut j_decompress_struct_stub;

#[repr(C)]
struct jpeg_component_info_stub {
    component_id: c_int,
    component_index: c_int,
    h_samp_factor: c_int,
    v_samp_factor: c_int,
    quant_tbl_no: c_int,
    dc_tbl_no: c_int,
    ac_tbl_no: c_int,
    width_in_blocks: c_uint,
    height_in_blocks: c_uint,
    DCT_scaled_size: c_int,
    downsampled_width: c_uint,
    downsampled_height: c_uint,
    component_needed: c_int,
    MCU_width: c_int,
    MCU_height: c_int,
    MCU_blocks: c_int,
    MCU_sample_width: c_int,
    last_col_width: c_int,
    last_row_height: c_int,
    quant_table: *mut c_void,
    dct_table: *mut c_void,
}

pub type jpeg_component_info_ptr_stub = *mut jpeg_component_info_stub;
