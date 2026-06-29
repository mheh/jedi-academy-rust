/*
 * jcparam.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains optional default-setting code for the JPEG compressor.
 * Applications do not have to use this file, but those that don't use it
 * must know a lot more about the innards of the JPEG code.
 */

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// JPEG library type definitions and constants
pub type UINT8 = u8;
pub type UINT16 = u16;
pub type boolean = UINT8;

pub const DCTSIZE2: c_int = 64;
pub const MAX_COMPONENTS: c_int = 10;
pub const NUM_ARITH_TBLS: c_int = 16;
pub const MAX_COMPS_IN_SCAN: c_int = 4;

pub const BITS_IN_JSAMPLE: c_int = 8;

pub const CSTATE_START: c_int = 200;

pub const JERR_BAD_STATE: c_int = 130;
pub const JERR_BAD_IN_COLORSPACE: c_int = 131;
pub const JERR_COMPONENT_COUNT: c_int = 132;
pub const JERR_BAD_J_COLORSPACE: c_int = 133;

pub const JCS_GRAYSCALE: c_int = 1;
pub const JCS_RGB: c_int = 2;
pub const JCS_YCbCr: c_int = 3;
pub const JCS_CMYK: c_int = 4;
pub const JCS_YCCK: c_int = 5;
pub const JCS_UNKNOWN: c_int = 0;

pub const JDCT_DEFAULT: c_int = 0;

pub const JPOOL_PERMANENT: c_int = 0;

pub const TRUE: boolean = 1;
pub const FALSE: boolean = 0;

// External JPEG library types and functions
#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [UINT16; DCTSIZE2 as usize],
    pub sent_table: boolean,
}

#[repr(C)]
pub struct JHUFF_TBL {
    pub bits: [UINT8; 17],
    pub huffval: [UINT8; 256],
    pub sent_table: boolean,
}

#[repr(C)]
pub struct jpeg_component_info {
    pub component_id: c_int,
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
    pub quant_tbl_no: c_int,
    pub dc_tbl_no: c_int,
    pub ac_tbl_no: c_int,
    // Additional fields may exist, but we define what we need
}

#[repr(C)]
pub struct jpeg_scan_info {
    pub comps_in_scan: c_int,
    pub component_index: [c_int; MAX_COMPS_IN_SCAN as usize],
    pub Ss: c_int,
    pub Se: c_int,
    pub Ah: c_int,
    pub Al: c_int,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void,
    // Other fields omitted
}

#[repr(C)]
pub struct j_compress_struct {
    pub mem: *mut jpeg_memory_mgr,
    pub global_state: c_int,
    pub quant_tbl_ptrs: [*mut JQUANT_TBL; 4],
    pub dc_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
    pub ac_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
    pub comp_info: *mut jpeg_component_info,
    pub data_precision: c_int,
    pub num_components: c_int,
    pub jpeg_color_space: c_int,
    pub in_color_space: c_int,
    pub input_components: c_int,
    pub arith_dc_L: [UINT8; NUM_ARITH_TBLS as usize],
    pub arith_dc_U: [UINT8; NUM_ARITH_TBLS as usize],
    pub arith_ac_K: [UINT8; NUM_ARITH_TBLS as usize],
    pub scan_info: *mut jpeg_scan_info,
    pub num_scans: c_int,
    pub raw_data_in: boolean,
    pub arith_code: boolean,
    pub optimize_coding: boolean,
    pub CCIR601_sampling: boolean,
    pub smoothing_factor: c_int,
    pub dct_method: c_int,
    pub restart_interval: c_int,
    pub restart_in_rows: c_int,
    pub density_unit: c_int,
    pub X_density: c_int,
    pub Y_density: c_int,
    pub write_JFIF_header: boolean,
    pub write_Adobe_marker: boolean,
}

pub type j_compress_ptr = *mut j_compress_struct;
pub type j_common_ptr = *mut c_void;

// External JPEG library functions
extern "C" {
    pub fn jpeg_alloc_quant_table(cinfo: j_common_ptr) -> *mut JQUANT_TBL;
    pub fn jpeg_alloc_huff_table(cinfo: j_common_ptr) -> *mut JHUFF_TBL;
}

// Error handling stub - in real implementation, this would call into the error handler
#[inline]
fn ERREXIT(cinfo: j_compress_ptr, _error_code: c_int) {
    // Stub: error handling would be done through cinfo->err
    let _ = cinfo;
}

#[inline]
fn ERREXIT1(cinfo: j_compress_ptr, error_code: c_int, _arg1: c_int) {
    let _ = (cinfo, error_code);
    // Stub: error handling would be done through cinfo->err
}

#[inline]
fn ERREXIT2(cinfo: j_compress_ptr, error_code: c_int, _arg1: c_int, _arg2: c_int) {
    let _ = (cinfo, error_code);
    // Stub: error handling would be done through cinfo->err
}

/*
 * Quantization table setup routines
 */

pub extern "C" fn jpeg_add_quant_table(
    cinfo: j_compress_ptr,
    which_tbl: c_int,
    basic_table: *const c_int,
    scale_factor: c_int,
    force_baseline: boolean,
) {
    /* Define a quantization table equal to the basic_table times
     * a scale factor (given as a percentage).
     * If force_baseline is TRUE, the computed quantization table entries
     * are limited to 1..255 for JPEG baseline compatibility.
     */
    let qtblptr: *mut *mut JQUANT_TBL = unsafe { &mut (*cinfo).quant_tbl_ptrs[which_tbl as usize] };
    let mut i: c_int;
    let mut temp: i64;

    /* Safety check to ensure start_compress not called yet. */
    unsafe {
        if (*cinfo).global_state != CSTATE_START {
            ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
        }

        if *qtblptr == core::ptr::null_mut() {
            *qtblptr = jpeg_alloc_quant_table(cinfo as j_common_ptr);
        }

        i = 0;
        while i < DCTSIZE2 {
            temp = (*(basic_table.add(i as usize)) as i64 * scale_factor as i64 + 50) / 100;
            /* limit the values to the valid range */
            if temp <= 0 {
                temp = 1;
            }
            if temp > 32767 {
                temp = 32767; /* max quantizer needed for 12 bits */
            }
            if force_baseline != 0 && temp > 255 {
                temp = 255; /* limit to baseline range if requested */
            }
            (*(*qtblptr)).quantval[i as usize] = temp as UINT16;
            i += 1;
        }

        /* Initialize sent_table FALSE so table will be written to JPEG file. */
        (*(*qtblptr)).sent_table = FALSE;
    }
}

pub extern "C" fn jpeg_set_linear_quality(
    cinfo: j_compress_ptr,
    scale_factor: c_int,
    force_baseline: boolean,
) {
    /* Set or change the 'quality' (quantization) setting, using default tables
     * and a straight percentage-scaling quality scale.  In most cases it's better
     * to use jpeg_set_quality (below); this entry point is provided for
     * applications that insist on a linear percentage scaling.
     */
    /* This is the sample quantization table given in the JPEG spec section K.1,
     * but expressed in zigzag order (as are all of our quant. tables).
     * The spec says that the values given produce "good" quality, and
     * when divided by 2, "very good" quality.
     */
    #[allow(non_upper_case_globals)]
    const std_luminance_quant_tbl: [c_int; DCTSIZE2 as usize] = [
        16, 11, 12, 14, 12, 10, 16, 14, 13, 14, 18, 17, 16, 19, 24, 40, 26, 24, 22, 22, 24, 49,
        35, 37, 29, 40, 58, 51, 61, 60, 57, 51, 56, 55, 64, 72, 92, 78, 64, 68, 87, 69, 55, 56,
        80, 109, 81, 87, 95, 98, 103, 104, 103, 62, 77, 113, 121, 112, 100, 120, 92, 101, 103,
        99,
    ];
    #[allow(non_upper_case_globals)]
    const std_chrominance_quant_tbl: [c_int; DCTSIZE2 as usize] = [
        17, 18, 18, 24, 21, 24, 47, 26, 26, 47, 99, 66, 56, 66, 99, 99, 99, 99, 99, 99, 99, 99,
        99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
        99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
    ];

    /* Set up two quantization tables using the specified scaling */
    jpeg_add_quant_table(
        cinfo,
        0,
        std_luminance_quant_tbl.as_ptr(),
        scale_factor,
        force_baseline,
    );
    jpeg_add_quant_table(
        cinfo,
        1,
        std_chrominance_quant_tbl.as_ptr(),
        scale_factor,
        force_baseline,
    );
}

pub extern "C" fn jpeg_quality_scaling(mut quality: c_int) -> c_int {
    /* Convert a user-specified quality rating to a percentage scaling factor
     * for an underlying quantization table, using our recommended scaling curve.
     * The input 'quality' factor should be 0 (terrible) to 100 (very good).
     */
    /* Safety limit on quality factor.  Convert 0 to 1 to avoid zero divide. */
    if quality <= 0 {
        quality = 1;
    }
    if quality > 100 {
        quality = 100;
    }

    /* The basic table is used as-is (scaling 100) for a quality of 50.
     * Qualities 50..100 are converted to scaling percentage 200 - 2*Q;
     * note that at Q=100 the scaling is 0, which will cause j_add_quant_table
     * to make all the table entries 1 (hence, no quantization loss).
     * Qualities 1..50 are converted to scaling percentage 5000/Q.
     */
    if quality < 50 {
        quality = 5000 / quality;
    } else {
        quality = 200 - quality * 2;
    }

    quality
}

pub extern "C" fn jpeg_set_quality(
    cinfo: j_compress_ptr,
    mut quality: c_int,
    force_baseline: boolean,
) {
    /* Set or change the 'quality' (quantization) setting, using default tables.
     * This is the standard quality-adjusting entry point for typical user
     * interfaces; only those who want detailed control over quantization tables
     * would use the preceding three routines directly.
     */
    /* Convert user 0-100 rating to percentage scaling */
    quality = jpeg_quality_scaling(quality);

    /* Set up standard quality tables */
    jpeg_set_linear_quality(cinfo, quality, force_baseline);
}

/*
 * Huffman table setup routines
 */

fn add_huff_table(
    cinfo: j_compress_ptr,
    htblptr: *mut *mut JHUFF_TBL,
    bits: *const UINT8,
    val: *const UINT8,
) {
    /* Define a Huffman table */
    unsafe {
        if *htblptr == core::ptr::null_mut() {
            *htblptr = jpeg_alloc_huff_table(cinfo as j_common_ptr);
        }

        // MEMCOPY((*htblptr)->bits, bits, SIZEOF((*htblptr)->bits));
        core::ptr::copy_nonoverlapping(bits, (*(*htblptr)).bits.as_mut_ptr(), 17);
        // MEMCOPY((*htblptr)->huffval, val, SIZEOF((*htblptr)->huffval));
        core::ptr::copy_nonoverlapping(val, (*(*htblptr)).huffval.as_mut_ptr(), 256);

        /* Initialize sent_table FALSE so table will be written to JPEG file. */
        (*(*htblptr)).sent_table = FALSE;
    }
}

fn std_huff_tables(cinfo: j_compress_ptr) {
    /* Set up the standard Huffman tables (cf. JPEG standard section K.3) */
    /* IMPORTANT: these are only valid for 8-bit data precision! */
    #[allow(non_upper_case_globals)]
    const bits_dc_luminance: [UINT8; 17] =
        [/* 0-base */ 0, 0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
    #[allow(non_upper_case_globals)]
    const val_dc_luminance: [UINT8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

    #[allow(non_upper_case_globals)]
    const bits_dc_chrominance: [UINT8; 17] =
        [/* 0-base */ 0, 0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
    #[allow(non_upper_case_globals)]
    const val_dc_chrominance: [UINT8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

    #[allow(non_upper_case_globals)]
    const bits_ac_luminance: [UINT8; 17] = [
        /* 0-base */ 0, 0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d,
    ];
    #[allow(non_upper_case_globals)]
    const val_ac_luminance: [UINT8; 162] = [
        0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61,
        0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52,
        0xd1, 0xf0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x25,
        0x26, 0x27, 0x28, 0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45,
        0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64,
        0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x83,
        0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99,
        0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6,
        0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3,
        0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8,
        0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa,
    ];

    #[allow(non_upper_case_globals)]
    const bits_ac_chrominance: [UINT8; 17] = [
        /* 0-base */ 0, 0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77,
    ];
    #[allow(non_upper_case_globals)]
    const val_ac_chrominance: [UINT8; 162] = [
        0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61,
        0x71, 0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xa1, 0xb1, 0xc1, 0x09, 0x23, 0x33,
        0x52, 0xf0, 0x15, 0x62, 0x72, 0xd1, 0x0a, 0x16, 0x24, 0x34, 0xe1, 0x25, 0xf1, 0x17, 0x18,
        0x19, 0x1a, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44,
        0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63,
        0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a,
        0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
        0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4,
        0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca,
        0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7,
        0xe8, 0xe9, 0xea, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa,
    ];

    unsafe {
        add_huff_table(
            cinfo,
            &mut (*cinfo).dc_huff_tbl_ptrs[0],
            bits_dc_luminance.as_ptr(),
            val_dc_luminance.as_ptr(),
        );
        add_huff_table(
            cinfo,
            &mut (*cinfo).ac_huff_tbl_ptrs[0],
            bits_ac_luminance.as_ptr(),
            val_ac_luminance.as_ptr(),
        );
        add_huff_table(
            cinfo,
            &mut (*cinfo).dc_huff_tbl_ptrs[1],
            bits_dc_chrominance.as_ptr(),
            val_dc_chrominance.as_ptr(),
        );
        add_huff_table(
            cinfo,
            &mut (*cinfo).ac_huff_tbl_ptrs[1],
            bits_ac_chrominance.as_ptr(),
            val_ac_chrominance.as_ptr(),
        );
    }
}

/*
 * Default parameter setup for compression.
 *
 * Applications that don't choose to use this routine must do their
 * own setup of all these parameters.  Alternately, you can call this
 * to establish defaults and then alter parameters selectively.  This
 * is the recommended approach since, if we add any new parameters,
 * your code will still work (they'll be set to reasonable defaults).
 */

pub extern "C" fn jpeg_set_defaults(cinfo: j_compress_ptr) {
    let mut i: c_int;

    unsafe {
        /* Safety check to ensure start_compress not called yet. */
        if (*cinfo).global_state != CSTATE_START {
            ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
        }

        /* Allocate comp_info array large enough for maximum component count.
         * Array is made permanent in case application wants to compress
         * multiple images at same param settings.
         */
        if (*cinfo).comp_info == core::ptr::null_mut() {
            (*cinfo).comp_info = ((*(*cinfo).mem).alloc_small)(
                cinfo as j_common_ptr,
                JPOOL_PERMANENT,
                (MAX_COMPONENTS as usize) * core::mem::size_of::<jpeg_component_info>(),
            ) as *mut jpeg_component_info;
        }

        /* Initialize everything not dependent on the color space */

        (*cinfo).data_precision = BITS_IN_JSAMPLE;
        /* Set up two quantization tables using default quality of 75 */
        jpeg_set_quality(cinfo, 75, TRUE);
        /* Set up two Huffman tables */
        std_huff_tables(cinfo);

        /* Initialize default arithmetic coding conditioning */
        i = 0;
        while i < NUM_ARITH_TBLS {
            (*cinfo).arith_dc_L[i as usize] = 0;
            (*cinfo).arith_dc_U[i as usize] = 1;
            (*cinfo).arith_ac_K[i as usize] = 5;
            i += 1;
        }

        /* Default is no multiple-scan output */
        (*cinfo).scan_info = core::ptr::null_mut();
        (*cinfo).num_scans = 0;

        /* Expect normal source image, not raw downsampled data */
        (*cinfo).raw_data_in = FALSE;

        /* Use Huffman coding, not arithmetic coding, by default */
        (*cinfo).arith_code = FALSE;

        /* By default, do extra passes to optimize entropy coding */
        (*cinfo).optimize_coding = TRUE;
        /* The standard Huffman tables are only valid for 8-bit data precision.
         * If the precision is higher, force optimization on so that usable
         * tables will be computed.  This test can be removed if default tables
         * are supplied that are valid for the desired precision.
         */
        if (*cinfo).data_precision > 8 {
            (*cinfo).optimize_coding = TRUE;
        }

        /* By default, use the simpler non-cosited sampling alignment */
        (*cinfo).CCIR601_sampling = FALSE;

        /* No input smoothing */
        (*cinfo).smoothing_factor = 0;

        /* DCT algorithm preference */
        (*cinfo).dct_method = JDCT_DEFAULT;

        /* No restart markers */
        (*cinfo).restart_interval = 0;
        (*cinfo).restart_in_rows = 0;

        /* Fill in default JFIF marker parameters.  Note that whether the marker
         * will actually be written is determined by jpeg_set_colorspace.
         */
        (*cinfo).density_unit = 0; /* Pixel size is unknown by default */
        (*cinfo).X_density = 1; /* Pixel aspect ratio is square by default */
        (*cinfo).Y_density = 1;

        /* Choose JPEG colorspace based on input space, set defaults accordingly */

        jpeg_default_colorspace(cinfo);
    }
}

/*
 * Select an appropriate JPEG colorspace for in_color_space.
 */

pub extern "C" fn jpeg_default_colorspace(cinfo: j_compress_ptr) {
    unsafe {
        match (*cinfo).in_color_space {
            JCS_GRAYSCALE => {
                jpeg_set_colorspace(cinfo, JCS_GRAYSCALE);
            }
            JCS_RGB => {
                jpeg_set_colorspace(cinfo, JCS_YCbCr);
            }
            JCS_YCbCr => {
                jpeg_set_colorspace(cinfo, JCS_YCbCr);
            }
            JCS_CMYK => {
                jpeg_set_colorspace(cinfo, JCS_CMYK); /* By default, no translation */
            }
            JCS_YCCK => {
                jpeg_set_colorspace(cinfo, JCS_YCCK);
            }
            JCS_UNKNOWN => {
                jpeg_set_colorspace(cinfo, JCS_UNKNOWN);
            }
            _ => {
                ERREXIT(cinfo, JERR_BAD_IN_COLORSPACE);
            }
        }
    }
}

/*
 * Set the JPEG colorspace, and choose colorspace-dependent default values.
 */

pub extern "C" fn jpeg_set_colorspace(cinfo: j_compress_ptr, colorspace: c_int) {
    let mut compptr: *mut jpeg_component_info;
    let mut ci: c_int;

    // SET_COMP macro expanded inline
    macro_rules! SET_COMP {
        ($index:expr, $id:expr, $hsamp:expr, $vsamp:expr, $quant:expr, $dctbl:expr, $actbl:expr) => {{
            compptr = unsafe { &mut (*cinfo).comp_info.add($index as usize) };
            unsafe {
                (*compptr).component_id = $id;
                (*compptr).h_samp_factor = $hsamp;
                (*compptr).v_samp_factor = $vsamp;
                (*compptr).quant_tbl_no = $quant;
                (*compptr).dc_tbl_no = $dctbl;
                (*compptr).ac_tbl_no = $actbl;
            }
        }};
    }

    unsafe {
        /* Safety check to ensure start_compress not called yet. */
        if (*cinfo).global_state != CSTATE_START {
            ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
        }

        /* For all colorspaces, we use Q and Huff tables 0 for luminance components,
         * tables 1 for chrominance components.
         */

        (*cinfo).jpeg_color_space = colorspace;

        (*cinfo).write_JFIF_header = FALSE; /* No marker for non-JFIF colorspaces */
        (*cinfo).write_Adobe_marker = FALSE; /* write no Adobe marker by default */

        match colorspace {
            JCS_GRAYSCALE => {
                (*cinfo).write_JFIF_header = TRUE; /* Write a JFIF marker */
                (*cinfo).num_components = 1;
                /* JFIF specifies component ID 1 */
                SET_COMP!(0, 1, 1, 1, 0, 0, 0);
            }
            JCS_RGB => {
                (*cinfo).write_Adobe_marker = TRUE; /* write Adobe marker to flag RGB */
                (*cinfo).num_components = 3;
                SET_COMP!(0, 0x52 /* 'R' */, 1, 1, 0, 0, 0);
                SET_COMP!(1, 0x47 /* 'G' */, 1, 1, 0, 0, 0);
                SET_COMP!(2, 0x42 /* 'B' */, 1, 1, 0, 0, 0);
            }
            JCS_YCbCr => {
                (*cinfo).write_JFIF_header = TRUE; /* Write a JFIF marker */
                (*cinfo).num_components = 3;
                /* JFIF specifies component IDs 1,2,3 */
                /* We default to 2x2 subsamples of chrominance */
                SET_COMP!(0, 1, 2, 2, 0, 0, 0);
                SET_COMP!(1, 2, 1, 1, 1, 1, 1);
                SET_COMP!(2, 3, 1, 1, 1, 1, 1);
            }
            JCS_CMYK => {
                (*cinfo).write_Adobe_marker = TRUE; /* write Adobe marker to flag CMYK */
                (*cinfo).num_components = 4;
                SET_COMP!(0, 0x43 /* 'C' */, 1, 1, 0, 0, 0);
                SET_COMP!(1, 0x4D /* 'M' */, 1, 1, 0, 0, 0);
                SET_COMP!(2, 0x59 /* 'Y' */, 1, 1, 0, 0, 0);
                SET_COMP!(3, 0x4B /* 'K' */, 1, 1, 0, 0, 0);
            }
            JCS_YCCK => {
                (*cinfo).write_Adobe_marker = TRUE; /* write Adobe marker to flag YCCK */
                (*cinfo).num_components = 4;
                SET_COMP!(0, 1, 2, 2, 0, 0, 0);
                SET_COMP!(1, 2, 1, 1, 1, 1, 1);
                SET_COMP!(2, 3, 1, 1, 1, 1, 1);
                SET_COMP!(3, 4, 2, 2, 0, 0, 0);
            }
            JCS_UNKNOWN => {
                (*cinfo).num_components = (*cinfo).input_components;
                if (*cinfo).num_components < 1 || (*cinfo).num_components > MAX_COMPONENTS {
                    ERREXIT2(
                        cinfo,
                        JERR_COMPONENT_COUNT,
                        (*cinfo).num_components,
                        MAX_COMPONENTS,
                    );
                }
                ci = 0;
                while ci < (*cinfo).num_components {
                    SET_COMP!(ci, ci, 1, 1, 0, 0, 0);
                    ci += 1;
                }
            }
            _ => {
                ERREXIT(cinfo, JERR_BAD_J_COLORSPACE);
            }
        }
    }
}

#[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
mod progressive_support {
    use super::*;

    fn fill_a_scan(
        scanptr: *mut jpeg_scan_info,
        ci: c_int,
        Ss: c_int,
        Se: c_int,
        Ah: c_int,
        Al: c_int,
    ) -> *mut jpeg_scan_info {
        /* Support routine: generate one scan for specified component */
        unsafe {
            (*scanptr).comps_in_scan = 1;
            (*scanptr).component_index[0] = ci;
            (*scanptr).Ss = Ss;
            (*scanptr).Se = Se;
            (*scanptr).Ah = Ah;
            (*scanptr).Al = Al;
            scanptr.add(1)
        }
    }

    fn fill_scans(
        mut scanptr: *mut jpeg_scan_info,
        ncomps: c_int,
        Ss: c_int,
        Se: c_int,
        Ah: c_int,
        Al: c_int,
    ) -> *mut jpeg_scan_info {
        /* Support routine: generate one scan for each component */
        let mut ci: c_int;

        unsafe {
            ci = 0;
            while ci < ncomps {
                (*scanptr).comps_in_scan = 1;
                (*scanptr).component_index[0] = ci;
                (*scanptr).Ss = Ss;
                (*scanptr).Se = Se;
                (*scanptr).Ah = Ah;
                (*scanptr).Al = Al;
                scanptr = scanptr.add(1);
                ci += 1;
            }
            scanptr
        }
    }

    fn fill_dc_scans(
        mut scanptr: *mut jpeg_scan_info,
        ncomps: c_int,
        Ah: c_int,
        Al: c_int,
    ) -> *mut jpeg_scan_info {
        /* Support routine: generate interleaved DC scan if possible, else N scans */
        let mut ci: c_int;

        unsafe {
            if ncomps <= MAX_COMPS_IN_SCAN {
                /* Single interleaved DC scan */
                (*scanptr).comps_in_scan = ncomps;
                ci = 0;
                while ci < ncomps {
                    (*scanptr).component_index[ci as usize] = ci;
                    ci += 1;
                }
                (*scanptr).Ss = 0;
                (*scanptr).Se = 0;
                (*scanptr).Ah = Ah;
                (*scanptr).Al = Al;
                scanptr = scanptr.add(1);
            } else {
                /* Noninterleaved DC scan for each component */
                scanptr = fill_scans(scanptr, ncomps, 0, 0, Ah, Al);
            }
            scanptr
        }
    }

    /*
     * Create a recommended progressive-JPEG script.
     * cinfo->num_components and cinfo->jpeg_color_space must be correct.
     */

    pub extern "C" fn jpeg_simple_progression(cinfo: j_compress_ptr) {
        let ncomps: c_int;
        let mut nscans: c_int;
        let mut scanptr: *mut jpeg_scan_info;

        unsafe {
            ncomps = (*cinfo).num_components;

            /* Safety check to ensure start_compress not called yet. */
            if (*cinfo).global_state != CSTATE_START {
                ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
            }

            /* Figure space needed for script.  Calculation must match code below! */
            if ncomps == 3 && (*cinfo).jpeg_color_space == JCS_YCbCr {
                /* Custom script for YCbCr color images. */
                nscans = 10;
            } else {
                /* All-purpose script for other color spaces. */
                if ncomps > MAX_COMPS_IN_SCAN {
                    nscans = 6 * ncomps; /* 2 DC + 4 AC scans per component */
                } else {
                    nscans = 2 + 4 * ncomps; /* 2 DC scans; 4 AC scans per component */
                }
            }

            /* Allocate space for script. */
            /* We use permanent pool just in case application re-uses script. */
            scanptr = ((*(*cinfo).mem).alloc_small)(
                cinfo as j_common_ptr,
                JPOOL_PERMANENT,
                (nscans as usize) * core::mem::size_of::<jpeg_scan_info>(),
            ) as *mut jpeg_scan_info;
            (*cinfo).scan_info = scanptr;
            (*cinfo).num_scans = nscans;

            if ncomps == 3 && (*cinfo).jpeg_color_space == JCS_YCbCr {
                /* Custom script for YCbCr color images. */
                /* Initial DC scan */
                scanptr = fill_dc_scans(scanptr, ncomps, 0, 1);
                /* Initial AC scan: get some luma data out in a hurry */
                scanptr = fill_a_scan(scanptr, 0, 1, 5, 0, 2);
                /* Chroma data is too small to be worth expending many scans on */
                scanptr = fill_a_scan(scanptr, 2, 1, 63, 0, 1);
                scanptr = fill_a_scan(scanptr, 1, 1, 63, 0, 1);
                /* Complete spectral selection for luma AC */
                scanptr = fill_a_scan(scanptr, 0, 6, 63, 0, 2);
                /* Refine next bit of luma AC */
                scanptr = fill_a_scan(scanptr, 0, 1, 63, 2, 1);
                /* Finish DC successive approximation */
                scanptr = fill_dc_scans(scanptr, ncomps, 1, 0);
                /* Finish AC successive approximation */
                scanptr = fill_a_scan(scanptr, 2, 1, 63, 1, 0);
                scanptr = fill_a_scan(scanptr, 1, 1, 63, 1, 0);
                /* Luma bottom bit comes last since it's usually largest scan */
                scanptr = fill_a_scan(scanptr, 0, 1, 63, 1, 0);
            } else {
                /* All-purpose script for other color spaces. */
                /* Successive approximation first pass */
                scanptr = fill_dc_scans(scanptr, ncomps, 0, 1);
                scanptr = fill_scans(scanptr, ncomps, 1, 5, 0, 2);
                scanptr = fill_scans(scanptr, ncomps, 6, 63, 0, 2);
                /* Successive approximation second pass */
                scanptr = fill_scans(scanptr, ncomps, 1, 63, 2, 1);
                /* Successive approximation final pass */
                scanptr = fill_dc_scans(scanptr, ncomps, 1, 0);
                scanptr = fill_scans(scanptr, ncomps, 1, 63, 1, 0);
            }
        }
    }
}

#[cfg(not(feature = "C_PROGRESSIVE_SUPPORTED"))]
pub extern "C" fn jpeg_simple_progression(cinfo: j_compress_ptr) {
    let _ = cinfo;
    // Stub when C_PROGRESSIVE_SUPPORTED is not enabled
}
