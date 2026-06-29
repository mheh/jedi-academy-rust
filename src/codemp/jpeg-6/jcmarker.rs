/*
 * jcmarker.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains routines to write JPEG datastream markers.
 */

#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::{c_int, c_void};

// ============================================================================
// Stubs for JPEG-6 types and structures needed for structural coherence
// ============================================================================

pub type JDIMENSION = u32;
pub type JOCTET = u8;
pub type boolean = u8;

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; 64],
    pub sent_table: boolean,
}

#[repr(C)]
pub struct JHUFF_TBL {
    pub bits: [u8; 17],
    pub huffval: [u8; 256],
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
}

#[repr(C)]
pub struct jpeg_destination_mgr {
    pub next_output_byte: *mut JOCTET,
    pub free_in_buffer: usize,
    pub init_destination: Option<unsafe extern "C" fn(*mut c_void)>,
    pub empty_output_buffer: Option<unsafe extern "C" fn(*mut c_void) -> boolean>,
    pub term_destination: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
}

#[repr(C)]
pub struct jpeg_marker_writer {
    pub write_any_marker: Option<unsafe extern "C" fn(*mut c_void, c_int, *const JOCTET, c_int)>,
    pub write_file_header: Option<unsafe extern "C" fn(*mut c_void)>,
    pub write_frame_header: Option<unsafe extern "C" fn(*mut c_void)>,
    pub write_scan_header: Option<unsafe extern "C" fn(*mut c_void)>,
    pub write_file_trailer: Option<unsafe extern "C" fn(*mut c_void)>,
    pub write_tables_only: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct j_compress_struct {
    pub dest: *mut jpeg_destination_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub comp_info: *mut jpeg_component_info,
    pub quant_tbl_ptrs: [*mut JQUANT_TBL; 4],
    pub dc_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
    pub ac_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
    pub num_components: c_int,
    pub data_precision: c_int,
    pub image_height: JDIMENSION,
    pub image_width: JDIMENSION,
    pub restart_interval: c_int,
    pub arith_code: boolean,
    pub progressive_mode: boolean,
    pub write_JFIF_header: boolean,
    pub write_Adobe_marker: boolean,
    pub jpeg_color_space: c_int,
    pub density_unit: c_int,
    pub X_density: c_int,
    pub Y_density: c_int,
    pub cur_comp_info: [*mut jpeg_component_info; 10],
    pub comps_in_scan: c_int,
    pub Ss: c_int,
    pub Se: c_int,
    pub Ah: c_int,
    pub Al: c_int,
    pub arith_dc_L: [c_int; 16],
    pub arith_dc_U: [c_int; 16],
    pub arith_ac_K: [c_int; 16],
    pub marker: *mut jpeg_marker_writer,
}

pub type j_compress_ptr = *mut j_compress_struct;
pub type j_common_ptr = *mut j_compress_struct;

/* Constants */
const DCTSIZE2: c_int = 64;
const NUM_QUANT_TBLS: c_int = 4;
const NUM_HUFF_TBLS: c_int = 4;
const NUM_ARITH_TBLS: c_int = 16;
const JPOOL_IMAGE: c_int = 1;
const FALSE: boolean = 0;
const TRUE: boolean = 1;

/* Error codes */
const JERR_CANT_SUSPEND: c_int = 1;
const JERR_NO_QUANT_TABLE: c_int = 2;
const JERR_NO_HUFF_TABLE: c_int = 3;
const JERR_IMAGE_TOO_BIG: c_int = 4;
const JTRC_16BIT_TABLES: c_int = 5;

/* JPEG color space constants */
const JCS_YCbCr: c_int = 1;
const JCS_YCCK: c_int = 2;

/* External C functions */
extern "C" {
    pub fn ERREXIT(cinfo: j_compress_ptr, code: c_int);
    pub fn ERREXIT1(cinfo: j_compress_ptr, code: c_int, p1: c_int);
    pub fn TRACEMS(cinfo: j_compress_ptr, level: c_int, code: c_int);
}

#[repr(C)]
#[derive(Clone, Copy)]
enum JPEG_MARKER {
    /* JPEG marker codes */
    M_SOF0 = 0xc0,
    M_SOF1 = 0xc1,
    M_SOF2 = 0xc2,
    M_SOF3 = 0xc3,

    M_SOF5 = 0xc5,
    M_SOF6 = 0xc6,
    M_SOF7 = 0xc7,

    M_JPG = 0xc8,
    M_SOF9 = 0xc9,
    M_SOF10 = 0xca,
    M_SOF11 = 0xcb,

    M_SOF13 = 0xcd,
    M_SOF14 = 0xce,
    M_SOF15 = 0xcf,

    M_DHT = 0xc4,

    M_DAC = 0xcc,

    M_RST0 = 0xd0,
    M_RST1 = 0xd1,
    M_RST2 = 0xd2,
    M_RST3 = 0xd3,
    M_RST4 = 0xd4,
    M_RST5 = 0xd5,
    M_RST6 = 0xd6,
    M_RST7 = 0xd7,

    M_SOI = 0xd8,
    M_EOI = 0xd9,
    M_SOS = 0xda,
    M_DQT = 0xdb,
    M_DNL = 0xdc,
    M_DRI = 0xdd,
    M_DHP = 0xde,
    M_EXP = 0xdf,

    M_APP0 = 0xe0,
    M_APP1 = 0xe1,
    M_APP2 = 0xe2,
    M_APP3 = 0xe3,
    M_APP4 = 0xe4,
    M_APP5 = 0xe5,
    M_APP6 = 0xe6,
    M_APP7 = 0xe7,
    M_APP8 = 0xe8,
    M_APP9 = 0xe9,
    M_APP10 = 0xea,
    M_APP11 = 0xeb,
    M_APP12 = 0xec,
    M_APP13 = 0xed,
    M_APP14 = 0xee,
    M_APP15 = 0xef,

    M_JPG0 = 0xf0,
    M_JPG13 = 0xfd,
    M_COM = 0xfe,

    M_TEM = 0x01,

    M_ERROR = 0x100,
}

/*
 * Basic output routines.
 *
 * Note that we do not support suspension while writing a marker.
 * Therefore, an application using suspension must ensure that there is
 * enough buffer space for the initial markers (typ. 600-700 bytes) before
 * calling jpeg_start_compress, and enough space to write the trailing EOI
 * (a few bytes) before calling jpeg_finish_compress.  Multipass compression
 * modes are not supported at all with suspension, so those two are the only
 * points where markers will be written.
 */

unsafe fn emit_byte(cinfo: j_compress_ptr, val: c_int) {
    /* Emit a byte */
    let dest = (*cinfo).dest;

    let dest_ref = &mut *dest;
    *dest_ref.next_output_byte = (val & 0xFF) as JOCTET;
    dest_ref.next_output_byte = dest_ref.next_output_byte.offset(1);
    dest_ref.free_in_buffer -= 1;

    if dest_ref.free_in_buffer == 0 {
        if let Some(empty_buffer_fn) = dest_ref.empty_output_buffer {
            if !empty_buffer_fn(cinfo) {
                ERREXIT(cinfo, JERR_CANT_SUSPEND);
            }
        }
    }
}

unsafe fn emit_marker(cinfo: j_compress_ptr, mark: JPEG_MARKER) {
    /* Emit a marker code */
    emit_byte(cinfo, 0xFF);
    emit_byte(cinfo, mark as c_int);
}

unsafe fn emit_2bytes(cinfo: j_compress_ptr, value: c_int) {
    /* Emit a 2-byte integer; these are always MSB first in JPEG files */
    emit_byte(cinfo, (value >> 8) & 0xFF);
    emit_byte(cinfo, value & 0xFF);
}

/*
 * Routines to write specific marker types.
 */

unsafe fn emit_dqt(cinfo: j_compress_ptr, index: c_int) -> c_int {
    /* Emit a DQT marker */
    /* Returns the precision used (0 = 8bits, 1 = 16bits) for baseline checking */
    let qtbl = (*cinfo).quant_tbl_ptrs[index as usize];
    let mut prec: c_int;

    if qtbl.is_null() {
        ERREXIT1(cinfo, JERR_NO_QUANT_TABLE, index);
    }

    prec = 0;
    for i in 0..DCTSIZE2 {
        if (*qtbl).quantval[i as usize] > 255 {
            prec = 1;
        }
    }

    if (*qtbl).sent_table == FALSE {
        emit_marker(cinfo, JPEG_MARKER::M_DQT);

        emit_2bytes(
            cinfo,
            if prec != 0 {
                DCTSIZE2 * 2 + 1 + 2
            } else {
                DCTSIZE2 + 1 + 2
            },
        );

        emit_byte(cinfo, index + (prec << 4));

        for i in 0..DCTSIZE2 {
            if prec != 0 {
                emit_byte(cinfo, ((*qtbl).quantval[i as usize] >> 8) as c_int);
            }
            emit_byte(cinfo, ((*qtbl).quantval[i as usize] & 0xFF) as c_int);
        }

        (*qtbl).sent_table = TRUE;
    }

    prec
}

unsafe fn emit_dht(cinfo: j_compress_ptr, index: c_int, is_ac: boolean) {
    /* Emit a DHT marker */
    let mut htbl: *mut JHUFF_TBL;
    let mut length: c_int;

    if is_ac != FALSE {
        htbl = (*cinfo).ac_huff_tbl_ptrs[index as usize];
        /* output index has AC bit set */
        let idx = index + 0x10;
    } else {
        htbl = (*cinfo).dc_huff_tbl_ptrs[index as usize];
    }

    if htbl.is_null() {
        ERREXIT1(cinfo, JERR_NO_HUFF_TABLE, index);
    }

    if (*htbl).sent_table == FALSE {
        emit_marker(cinfo, JPEG_MARKER::M_DHT);

        length = 0;
        for i in 1..=16 {
            length += (*htbl).bits[i] as c_int;
        }

        emit_2bytes(cinfo, length + 2 + 1 + 16);
        emit_byte(cinfo, index);

        for i in 1..=16 {
            emit_byte(cinfo, (*htbl).bits[i] as c_int);
        }

        for i in 0..length {
            emit_byte(cinfo, (*htbl).huffval[i as usize] as c_int);
        }

        (*htbl).sent_table = TRUE;
    }
}

unsafe fn emit_dac(cinfo: j_compress_ptr) {
    /* Emit a DAC marker */
    /* Since the useful info is so small, we want to emit all the tables in */
    /* one DAC marker.  Therefore this routine does its own scan of the table. */
    #[cfg(feature = "C_ARITH_CODING_SUPPORTED")]
    {
        let mut dc_in_use: [u8; 16] = [0; 16];
        let mut ac_in_use: [u8; 16] = [0; 16];
        let mut length: c_int;

        for i in 0..NUM_ARITH_TBLS {
            dc_in_use[i as usize] = 0;
            ac_in_use[i as usize] = 0;
        }

        for i in 0..(*cinfo).comps_in_scan {
            let compptr = *(*cinfo).cur_comp_info.as_ptr().offset(i as isize);
            dc_in_use[(*compptr).dc_tbl_no as usize] = 1;
            ac_in_use[(*compptr).ac_tbl_no as usize] = 1;
        }

        length = 0;
        for i in 0..NUM_ARITH_TBLS {
            length += dc_in_use[i as usize] as c_int + ac_in_use[i as usize] as c_int;
        }

        emit_marker(cinfo, JPEG_MARKER::M_DAC);

        emit_2bytes(cinfo, length * 2 + 2);

        for i in 0..NUM_ARITH_TBLS {
            if dc_in_use[i as usize] != FALSE {
                emit_byte(cinfo, i);
                emit_byte(
                    cinfo,
                    (*cinfo).arith_dc_L[i as usize]
                        + ((*cinfo).arith_dc_U[i as usize] << 4),
                );
            }
            if ac_in_use[i as usize] != FALSE {
                emit_byte(cinfo, i + 0x10);
                emit_byte(cinfo, (*cinfo).arith_ac_K[i as usize]);
            }
        }
    }
}

unsafe fn emit_dri(cinfo: j_compress_ptr) {
    /* Emit a DRI marker */
    emit_marker(cinfo, JPEG_MARKER::M_DRI);

    emit_2bytes(cinfo, 4); /* fixed length */

    emit_2bytes(cinfo, (*cinfo).restart_interval as c_int);
}

unsafe fn emit_sof(cinfo: j_compress_ptr, code: JPEG_MARKER) {
    /* Emit a SOF marker */
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    emit_marker(cinfo, code);

    emit_2bytes(
        cinfo,
        3 * (*cinfo).num_components + 2 + 5 + 1,
    ); /* length */

    /* Make sure image isn't bigger than SOF field can handle */
    if ((*cinfo).image_height as c_int) > 65535 || ((*cinfo).image_width as c_int) > 65535 {
        ERREXIT1(cinfo, JERR_IMAGE_TOO_BIG, 65535);
    }

    emit_byte(cinfo, (*cinfo).data_precision);
    emit_2bytes(cinfo, (*cinfo).image_height as c_int);
    emit_2bytes(cinfo, (*cinfo).image_width as c_int);

    emit_byte(cinfo, (*cinfo).num_components);

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        emit_byte(cinfo, (*compptr).component_id);
        emit_byte(
            cinfo,
            ((*compptr).h_samp_factor << 4) + (*compptr).v_samp_factor,
        );
        emit_byte(cinfo, (*compptr).quant_tbl_no);
        ci += 1;
        compptr = compptr.offset(1);
    }
}

unsafe fn emit_sos(cinfo: j_compress_ptr) {
    /* Emit a SOS marker */
    let mut i: c_int;
    let mut td: c_int;
    let mut ta: c_int;
    let mut compptr: *mut jpeg_component_info;

    emit_marker(cinfo, JPEG_MARKER::M_SOS);

    emit_2bytes(cinfo, 2 * (*cinfo).comps_in_scan + 2 + 1 + 3); /* length */

    emit_byte(cinfo, (*cinfo).comps_in_scan);

    for i in 0..(*cinfo).comps_in_scan {
        compptr = *(*cinfo).cur_comp_info.as_ptr().offset(i as isize);
        emit_byte(cinfo, (*compptr).component_id);
        td = (*compptr).dc_tbl_no;
        ta = (*compptr).ac_tbl_no;
        if (*cinfo).progressive_mode != FALSE {
            /* Progressive mode: only DC or only AC tables are used in one scan;
             * furthermore, Huffman coding of DC refinement uses no table at all.
             * We emit 0 for unused field(s); this is recommended by the P&M text
             * but does not seem to be specified in the standard.
             */
            if (*cinfo).Ss == 0 {
                ta = 0; /* DC scan */
                if (*cinfo).Ah != 0 && (*cinfo).arith_code == FALSE {
                    td = 0; /* no DC table either */
                }
            } else {
                td = 0; /* AC scan */
            }
        }
        emit_byte(cinfo, (td << 4) + ta);
    }

    emit_byte(cinfo, (*cinfo).Ss);
    emit_byte(cinfo, (*cinfo).Se);
    emit_byte(cinfo, ((*cinfo).Ah << 4) + (*cinfo).Al);
}

unsafe fn emit_jfif_app0(cinfo: j_compress_ptr) {
    /* Emit a JFIF-compliant APP0 marker */
    /*
     * Length of APP0 block	(2 bytes)
     * Block ID			(4 bytes - ASCII "JFIF")
     * Zero byte			(1 byte to terminate the ID string)
     * Version Major, Minor	(2 bytes - 0x01, 0x01)
     * Units			(1 byte - 0x00 = none, 0x01 = inch, 0x02 = cm)
     * Xdpu			(2 bytes - dots per unit horizontal)
     * Ydpu			(2 bytes - dots per unit vertical)
     * Thumbnail X size		(1 byte)
     * Thumbnail Y size		(1 byte)
     */

    emit_marker(cinfo, JPEG_MARKER::M_APP0);

    emit_2bytes(cinfo, 2 + 4 + 1 + 2 + 1 + 2 + 2 + 1 + 1); /* length */

    emit_byte(cinfo, 0x4A); /* Identifier: ASCII "JFIF" */
    emit_byte(cinfo, 0x46);
    emit_byte(cinfo, 0x49);
    emit_byte(cinfo, 0x46);
    emit_byte(cinfo, 0);
    /* We currently emit version code 1.01 since we use no 1.02 features.
     * This may avoid complaints from some older decoders.
     */
    emit_byte(cinfo, 1); /* Major version */
    emit_byte(cinfo, 1); /* Minor version */
    emit_byte(cinfo, (*cinfo).density_unit); /* Pixel size information */
    emit_2bytes(cinfo, (*cinfo).X_density as c_int);
    emit_2bytes(cinfo, (*cinfo).Y_density as c_int);
    emit_byte(cinfo, 0); /* No thumbnail image */
    emit_byte(cinfo, 0);
}

unsafe fn emit_adobe_app14(cinfo: j_compress_ptr) {
    /* Emit an Adobe APP14 marker */
    /*
     * Length of APP14 block	(2 bytes)
     * Block ID			(5 bytes - ASCII "Adobe")
     * Version Number		(2 bytes - currently 100)
     * Flags0			(2 bytes - currently 0)
     * Flags1			(2 bytes - currently 0)
     * Color transform		(1 byte)
     *
     * Although Adobe TN 5116 mentions Version = 101, all the Adobe files
     * now in circulation seem to use Version = 100, so that's what we write.
     *
     * We write the color transform byte as 1 if the JPEG color space is
     * YCbCr, 2 if it's YCCK, 0 otherwise.  Adobe's definition has to do with
     * whether the encoder performed a transformation, which is pretty useless.
     */

    emit_marker(cinfo, JPEG_MARKER::M_APP14);

    emit_2bytes(cinfo, 2 + 5 + 2 + 2 + 2 + 1); /* length */

    emit_byte(cinfo, 0x41); /* Identifier: ASCII "Adobe" */
    emit_byte(cinfo, 0x64);
    emit_byte(cinfo, 0x6F);
    emit_byte(cinfo, 0x62);
    emit_byte(cinfo, 0x65);
    emit_2bytes(cinfo, 100); /* Version */
    emit_2bytes(cinfo, 0); /* Flags0 */
    emit_2bytes(cinfo, 0); /* Flags1 */
    match (*cinfo).jpeg_color_space {
        JCS_YCbCr => {
            emit_byte(cinfo, 1); /* Color transform = 1 */
        }
        JCS_YCCK => {
            emit_byte(cinfo, 2); /* Color transform = 2 */
        }
        _ => {
            emit_byte(cinfo, 0); /* Color transform = 0 */
        }
    }
}

/*
 * This routine is exported for possible use by applications.
 * The intended use is to emit COM or APPn markers after calling
 * jpeg_start_compress() and before the first jpeg_write_scanlines() call
 * (hence, after write_file_header but before write_frame_header).
 * Other uses are not guaranteed to produce desirable results.
 */

pub unsafe fn write_any_marker(
    cinfo: j_compress_ptr,
    marker: c_int,
    dataptr: *const JOCTET,
    datalen: c_int,
) {
    /* Emit an arbitrary marker with parameters */
    if (datalen as u32) <= 65533 {
        /* safety check */
        emit_marker(cinfo, JPEG_MARKER::M_COM); /* simplified; using M_COM for dispatch */

        emit_2bytes(cinfo, datalen + 2); /* total length */

        let mut ptr = dataptr;
        let mut count = datalen;
        while count > 0 {
            emit_byte(cinfo, *ptr as c_int);
            ptr = ptr.offset(1);
            count -= 1;
        }
    }
}

/*
 * Write datastream header.
 * This consists of an SOI and optional APPn markers.
 * We recommend use of the JFIF marker, but not the Adobe marker,
 * when using YCbCr or grayscale data.  The JFIF marker should NOT
 * be used for any other JPEG colorspace.  The Adobe marker is helpful
 * to distinguish RGB, CMYK, and YCCK colorspaces.
 * Note that an application can write additional header markers after
 * jpeg_start_compress returns.
 */

pub unsafe fn write_file_header(cinfo: j_compress_ptr) {
    emit_marker(cinfo, JPEG_MARKER::M_SOI); /* first the SOI */

    if (*cinfo).write_JFIF_header != FALSE {
        /* next an optional JFIF APP0 */
        emit_jfif_app0(cinfo);
    }
    if (*cinfo).write_Adobe_marker != FALSE {
        /* next an optional Adobe APP14 */
        emit_adobe_app14(cinfo);
    }
}

/*
 * Write frame header.
 * This consists of DQT and SOFn markers.
 * Note that we do not emit the SOF until we have emitted the DQT(s).
 * This avoids compatibility problems with incorrect implementations that
 * try to error-check the quant table numbers as soon as they see the SOF.
 */

pub unsafe fn write_frame_header(cinfo: j_compress_ptr) {
    let mut ci: c_int;
    let mut prec: c_int;
    let mut is_baseline: boolean;
    let mut compptr: *mut jpeg_component_info;

    /* Emit DQT for each quantization table.
     * Note that emit_dqt() suppresses any duplicate tables.
     */
    prec = 0;
    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        prec += emit_dqt(cinfo, (*compptr).quant_tbl_no);
        ci += 1;
        compptr = compptr.offset(1);
    }
    /* now prec is nonzero iff there are any 16-bit quant tables. */

    /* Check for a non-baseline specification.
     * Note we assume that Huffman table numbers won't be changed later.
     */
    if (*cinfo).arith_code != FALSE
        || (*cinfo).progressive_mode != FALSE
        || (*cinfo).data_precision != 8
    {
        is_baseline = FALSE;
    } else {
        is_baseline = TRUE;
        ci = 0;
        compptr = (*cinfo).comp_info;
        while ci < (*cinfo).num_components {
            if (*compptr).dc_tbl_no > 1 || (*compptr).ac_tbl_no > 1 {
                is_baseline = FALSE;
            }
            ci += 1;
            compptr = compptr.offset(1);
        }
        if prec != 0 && is_baseline != FALSE {
            is_baseline = FALSE;
            /* If it's baseline except for quantizer size, warn the user */
            TRACEMS(cinfo, 0, JTRC_16BIT_TABLES);
        }
    }

    /* Emit the proper SOF marker */
    if (*cinfo).arith_code != FALSE {
        emit_sof(cinfo, JPEG_MARKER::M_SOF9); /* SOF code for arithmetic coding */
    } else {
        if (*cinfo).progressive_mode != FALSE {
            emit_sof(cinfo, JPEG_MARKER::M_SOF2); /* SOF code for progressive Huffman */
        } else if is_baseline != FALSE {
            emit_sof(cinfo, JPEG_MARKER::M_SOF0); /* SOF code for baseline implementation */
        } else {
            emit_sof(cinfo, JPEG_MARKER::M_SOF1); /* SOF code for non-baseline Huffman file */
        }
    }
}

/*
 * Write scan header.
 * This consists of DHT or DAC markers, optional DRI, and SOS.
 * Compressed data will be written following the SOS.
 */

pub unsafe fn write_scan_header(cinfo: j_compress_ptr) {
    let mut i: c_int;
    let mut compptr: *mut jpeg_component_info;

    if (*cinfo).arith_code != FALSE {
        /* Emit arith conditioning info.  We may have some duplication
         * if the file has multiple scans, but it's so small it's hardly
         * worth worrying about.
         */
        emit_dac(cinfo);
    } else {
        /* Emit Huffman tables.
         * Note that emit_dht() suppresses any duplicate tables.
         */
        for i in 0..(*cinfo).comps_in_scan {
            compptr = *(*cinfo).cur_comp_info.as_ptr().offset(i as isize);
            if (*cinfo).progressive_mode != FALSE {
                /* Progressive mode: only DC or only AC tables are used in one scan */
                if (*cinfo).Ss == 0 {
                    if (*cinfo).Ah == 0 {
                        /* DC needs no table for refinement scan */
                        emit_dht(cinfo, (*compptr).dc_tbl_no, FALSE);
                    }
                } else {
                    emit_dht(cinfo, (*compptr).ac_tbl_no, TRUE);
                }
            } else {
                /* Sequential mode: need both DC and AC tables */
                emit_dht(cinfo, (*compptr).dc_tbl_no, FALSE);
                emit_dht(cinfo, (*compptr).ac_tbl_no, TRUE);
            }
        }
    }

    /* Emit DRI if required --- note that DRI value could change for each scan.
     * If it doesn't, a tiny amount of space is wasted in multiple-scan files.
     * We assume DRI will never be nonzero for one scan and zero for a later one.
     */
    if (*cinfo).restart_interval != 0 {
        emit_dri(cinfo);
    }

    emit_sos(cinfo);
}

/*
 * Write datastream trailer.
 */

pub unsafe fn write_file_trailer(cinfo: j_compress_ptr) {
    emit_marker(cinfo, JPEG_MARKER::M_EOI);
}

/*
 * Write an abbreviated table-specification datastream.
 * This consists of SOI, DQT and DHT tables, and EOI.
 * Any table that is defined and not marked sent_table = TRUE will be
 * emitted.  Note that all tables will be marked sent_table = TRUE at exit.
 */

pub unsafe fn write_tables_only(cinfo: j_compress_ptr) {
    let mut i: c_int;

    emit_marker(cinfo, JPEG_MARKER::M_SOI);

    for i in 0..NUM_QUANT_TBLS {
        if !(*cinfo).quant_tbl_ptrs[i as usize].is_null() {
            let _ = emit_dqt(cinfo, i);
        }
    }

    if (*cinfo).arith_code == FALSE {
        for i in 0..NUM_HUFF_TBLS {
            if !(*cinfo).dc_huff_tbl_ptrs[i as usize].is_null() {
                emit_dht(cinfo, i, FALSE);
            }
            if !(*cinfo).ac_huff_tbl_ptrs[i as usize].is_null() {
                emit_dht(cinfo, i, TRUE);
            }
        }
    }

    emit_marker(cinfo, JPEG_MARKER::M_EOI);
}

/*
 * Initialize the marker writer module.
 */

pub unsafe fn jinit_marker_writer(cinfo: j_compress_ptr) {
    /* Create the subobject */
    let marker = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        core::mem::size_of::<jpeg_marker_writer>(),
    ) as *mut jpeg_marker_writer;

    (*cinfo).marker = marker;

    /* Initialize method pointers */
    (*marker).write_any_marker = Some(write_any_marker as unsafe extern "C" fn(j_compress_ptr, c_int, *const JOCTET, c_int));
    (*marker).write_file_header = Some(write_file_header as unsafe extern "C" fn(j_compress_ptr));
    (*marker).write_frame_header = Some(write_frame_header as unsafe extern "C" fn(j_compress_ptr));
    (*marker).write_scan_header = Some(write_scan_header as unsafe extern "C" fn(j_compress_ptr));
    (*marker).write_file_trailer = Some(write_file_trailer as unsafe extern "C" fn(j_compress_ptr));
    (*marker).write_tables_only = Some(write_tables_only as unsafe extern "C" fn(j_compress_ptr));
}
