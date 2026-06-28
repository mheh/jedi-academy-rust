/*
 * jdmarker.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains routines to decode JPEG datastream markers.
 * Most of the complexity arises from our desire to support input
 * suspension: if not all of the data for a marker is available,
 * we must exit back to the application.  On resumption, we reprocess
 * the marker.
 */

use core::ffi::c_int;

/* JPEG marker codes */
#[repr(C)]
pub enum JPEG_MARKER {
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

/* Opaque JPEG library types */
#[repr(C)]
pub struct j_decompress_struct {
    _opaque: [u8; 0],
}
pub type j_decompress_ptr = *mut j_decompress_struct;

#[repr(C)]
pub struct j_common_struct {
    _opaque: [u8; 0],
}
pub type j_common_ptr = *mut j_common_struct;

#[repr(C)]
pub struct jpeg_source_mgr {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_marker_reader {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_component_info {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct JHUFF_TBL {
    pub bits: [u8; 17],
    pub huffval: [u8; 256],
}

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; 64],
}

pub type JOCTET = u8;
pub type INT32 = i32;
pub type UINT8 = u8;
pub type UINT16 = u16;
pub type UINT32 = u32;
pub type boolean = u8;

const TRUE: boolean = 1;
const FALSE: boolean = 0;

const NUM_ARITH_TBLS: usize = 16;
const MAX_COMPS_IN_SCAN: usize = 4;
const NUM_HUFF_TBLS: usize = 4;
const NUM_QUANT_TBLS: usize = 4;
const DCTSIZE2: usize = 64;
const JFIF_LEN: usize = 14;
const ADOBE_LEN: usize = 12;

const JPOOL_IMAGE: c_int = 0;
const JPOOL_PERMANENT: c_int = 1;

const JPEG_SUSPENDED: c_int = 0;
const JPEG_REACHED_SOS: c_int = 1;
const JPEG_REACHED_EOI: c_int = 2;

const JTRC_SOI: c_int = 0;
const JERR_SOI_DUPLICATE: c_int = 1;
const JTRC_SOF: c_int = 2;
const JERR_SOF_DUPLICATE: c_int = 3;
const JERR_EMPTY_IMAGE: c_int = 4;
const JERR_BAD_LENGTH: c_int = 5;
const JTRC_SOF_COMPONENT: c_int = 6;
const JERR_SOS_NO_SOF: c_int = 7;
const JTRC_SOS: c_int = 8;
const JERR_BAD_COMPONENT_ID: c_int = 9;
const JTRC_SOS_COMPONENT: c_int = 10;
const JTRC_SOS_PARAMS: c_int = 11;
const JTRC_APP0: c_int = 12;
const JTRC_JFIF: c_int = 13;
const JTRC_JFIF_MINOR: c_int = 14;
const JWRN_JFIF_MAJOR: c_int = 15;
const JTRC_JFIF_THUMBNAIL: c_int = 16;
const JTRC_JFIF_BADTHUMBNAILSIZE: c_int = 17;
const JTRC_APP14: c_int = 18;
const JTRC_ADOBE: c_int = 19;
const JTRC_DAC: c_int = 20;
const JERR_DAC_INDEX: c_int = 21;
const JERR_DAC_VALUE: c_int = 22;
const JTRC_DHT: c_int = 23;
const JTRC_HUFFBITS: c_int = 24;
const JERR_DHT_COUNTS: c_int = 25;
const JERR_DHT_INDEX: c_int = 26;
const JTRC_DQT: c_int = 27;
const JERR_DQT_INDEX: c_int = 28;
const JTRC_QUANTVALS: c_int = 29;
const JTRC_DRI: c_int = 30;
const JTRC_MISC_MARKER: c_int = 31;
const JTRC_EOI: c_int = 32;
const JTRC_PARMLESS_MARKER: c_int = 33;
const JTRC_RST: c_int = 34;
const JWRN_MUST_RESYNC: c_int = 35;
const JTRC_RECOVERY_ACTION: c_int = 36;
const JERR_SOF_UNSUPPORTED: c_int = 37;
const JERR_UNKNOWN_MARKER: c_int = 38;
const JERR_NO_SOI: c_int = 39;
const JWRN_EXTRANEOUS_DATA: c_int = 40;

/* Stub functions for trace/error/warning macros - these would be defined in the full JPEG library */
#[inline]
fn TRACEMS(_cinfo: j_decompress_ptr, _level: c_int, _code: c_int) {}

#[inline]
fn TRACEMS1(_cinfo: j_decompress_ptr, _level: c_int, _code: c_int, _p1: c_int) {}

#[inline]
fn TRACEMS2(_cinfo: j_decompress_ptr, _level: c_int, _code: c_int, _p1: c_int, _p2: c_int) {}

#[inline]
fn TRACEMS3(_cinfo: j_decompress_ptr, _level: c_int, _code: c_int, _p1: c_int, _p2: c_int, _p3: c_int) {}

#[inline]
fn TRACEMS4(_cinfo: j_decompress_ptr, _level: c_int, _code: c_int, _p1: c_int, _p2: c_int, _p3: c_int, _p4: c_int) {}

#[inline]
fn TRACEMS8(_cinfo: j_decompress_ptr, _level: c_int, _code: c_int, _p1: c_int, _p2: c_int, _p3: c_int, _p4: c_int, _p5: c_int, _p6: c_int, _p7: c_int, _p8: c_int) {}

#[inline]
fn ERREXIT(_cinfo: j_decompress_ptr, _code: c_int) {
    /* Stub - would call error handler in actual JPEG library */
}

#[inline]
fn ERREXIT1(_cinfo: j_decompress_ptr, _code: c_int, _p1: c_int) {
    /* Stub - would call error handler in actual JPEG library */
}

#[inline]
fn ERREXIT2(_cinfo: j_decompress_ptr, _code: c_int, _p1: c_int, _p2: c_int) {
    /* Stub - would call error handler in actual JPEG library */
}

#[inline]
fn WARNMS2(_cinfo: j_decompress_ptr, _code: c_int, _p1: c_int, _p2: c_int) {
    /* Stub - would call warning handler in actual JPEG library */
}

#[inline]
fn GETJOCTET(x: JOCTET) -> JOCTET {
    x
}

#[inline]
fn MEMCOPY(dest: *mut u8, src: *const u8, len: usize) {
    unsafe {
        core::ptr::copy(src, dest, len);
    }
}

extern "C" {
    fn jpeg_alloc_huff_table(cinfo: j_common_ptr) -> *mut JHUFF_TBL;
    fn jpeg_alloc_quant_table(cinfo: j_common_ptr) -> *mut JQUANT_TBL;
}

/*
 * Macros for fetching data from the data source module.
 *
 * At all times, cinfo->src->next_input_byte and ->bytes_in_buffer reflect
 * the current restart point; we update them only when we have reached a
 * suitable place to restart if a suspension occurs.
 */

/* Declare and initialize local copies of input pointer/count */
macro_rules! INPUT_VARS {
    ($cinfo:expr) => {{
        let datasrc = unsafe { (*$cinfo).src };
        let next_input_byte = unsafe { (*datasrc).next_input_byte };
        let bytes_in_buffer = unsafe { (*datasrc).bytes_in_buffer };
        (datasrc, next_input_byte, bytes_in_buffer)
    }};
}

/* Unload the local copies --- do this only at a restart boundary */
macro_rules! INPUT_SYNC {
    ($cinfo:expr, $datasrc:expr, $next_input_byte:expr, $bytes_in_buffer:expr) => {
        unsafe {
            (*$datasrc).next_input_byte = $next_input_byte;
            (*$datasrc).bytes_in_buffer = $bytes_in_buffer;
        }
    };
}

/* Reload the local copies --- seldom used except in MAKE_BYTE_AVAIL */
macro_rules! INPUT_RELOAD {
    ($datasrc:expr) => {{
        let next_input_byte = unsafe { (*$datasrc).next_input_byte };
        let bytes_in_buffer = unsafe { (*$datasrc).bytes_in_buffer };
        (next_input_byte, bytes_in_buffer)
    }};
}

/* Read a byte into variable V.
 * If must suspend, take the specified action (typically "return FALSE").
 */
macro_rules! INPUT_BYTE {
    ($cinfo:expr, $datasrc:expr, $next_input_byte:expr, $bytes_in_buffer:expr, $V:expr) => {{
        if $bytes_in_buffer == 0 {
            let fill_input_buffer = unsafe { (*$datasrc).fill_input_buffer };
            if unsafe { fill_input_buffer($cinfo) } as u8 == 0 {
                return FALSE;
            }
            let reloaded = INPUT_RELOAD!($datasrc);
            $next_input_byte = reloaded.0;
            $bytes_in_buffer = reloaded.1;
        }
        $bytes_in_buffer -= 1;
        $V = unsafe { GETJOCTET(*$next_input_byte) };
        $next_input_byte = unsafe { $next_input_byte.add(1) };
    }};
}

/* As above, but read two bytes interpreted as an unsigned 16-bit integer.
 * V should be declared unsigned int or perhaps INT32.
 */
macro_rules! INPUT_2BYTES {
    ($cinfo:expr, $datasrc:expr, $next_input_byte:expr, $bytes_in_buffer:expr, $V:expr) => {{
        if $bytes_in_buffer == 0 {
            let fill_input_buffer = unsafe { (*$datasrc).fill_input_buffer };
            if unsafe { fill_input_buffer($cinfo) } as u8 == 0 {
                return FALSE;
            }
            let reloaded = INPUT_RELOAD!($datasrc);
            $next_input_byte = reloaded.0;
            $bytes_in_buffer = reloaded.1;
        }
        $bytes_in_buffer -= 1;
        $V = ((unsafe { GETJOCTET(*$next_input_byte) } as UINT32) << 8);
        $next_input_byte = unsafe { $next_input_byte.add(1) };
        if $bytes_in_buffer == 0 {
            let fill_input_buffer = unsafe { (*$datasrc).fill_input_buffer };
            if unsafe { fill_input_buffer($cinfo) } as u8 == 0 {
                return FALSE;
            }
            let reloaded = INPUT_RELOAD!($datasrc);
            $next_input_byte = reloaded.0;
            $bytes_in_buffer = reloaded.1;
        }
        $bytes_in_buffer -= 1;
        $V += unsafe { GETJOCTET(*$next_input_byte) as UINT32 };
        $next_input_byte = unsafe { $next_input_byte.add(1) };
    }};
}

fn get_soi(cinfo: j_decompress_ptr) -> boolean {
    /* Process an SOI marker */
    let mut i: c_int;

    TRACEMS(cinfo, 1, JTRC_SOI);

    unsafe {
        if (*cinfo).marker != core::ptr::null_mut() && (*(*cinfo).marker).saw_SOI as u8 != 0 {
            ERREXIT(cinfo, JERR_SOI_DUPLICATE);
        }

        /* Reset all parameters that are defined to be reset by SOI */

        for i in 0..NUM_ARITH_TBLS {
            (*cinfo).arith_dc_L[i] = 0;
            (*cinfo).arith_dc_U[i] = 1;
            (*cinfo).arith_ac_K[i] = 5;
        }
        (*cinfo).restart_interval = 0;

        /* Set initial assumptions for colorspace etc */

        (*cinfo).jpeg_color_space = 0; /* JCS_UNKNOWN */
        (*cinfo).CCIR601_sampling = FALSE; /* Assume non-CCIR sampling??? */

        (*cinfo).saw_JFIF_marker = FALSE;
        (*cinfo).density_unit = 0; /* set default JFIF APP0 values */
        (*cinfo).X_density = 1;
        (*cinfo).Y_density = 1;
        (*cinfo).saw_Adobe_marker = FALSE;
        (*cinfo).Adobe_transform = 0;

        (*(*cinfo).marker).saw_SOI = TRUE;
    }

    TRUE
}

fn get_sof(cinfo: j_decompress_ptr, is_prog: boolean, is_arith: boolean) -> boolean {
    /* Process a SOFn marker */
    let mut length: UINT32 = 0;
    let mut c: UINT8 = 0;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    unsafe {
        (*cinfo).progressive_mode = is_prog;
        (*cinfo).arith_code = is_arith;
    }

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, length);
    let length = length as INT32;

    INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, unsafe { (*cinfo).data_precision });
    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, unsafe { (*cinfo).image_height } as UINT32);
    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, unsafe { (*cinfo).image_width } as UINT32);
    INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, unsafe { (*cinfo).num_components });

    let mut length = length - 8;

    TRACEMS4(cinfo, 1, JTRC_SOF, unsafe { (*cinfo).unread_marker },
        unsafe { (*cinfo).image_width as c_int },
        unsafe { (*cinfo).image_height as c_int },
        unsafe { (*cinfo).num_components as c_int });

    unsafe {
        if (*(*cinfo).marker).saw_SOF as u8 != 0 {
            ERREXIT(cinfo, JERR_SOF_DUPLICATE);
        }

        /* We don't support files in which the image height is initially specified */
        /* as 0 and is later redefined by DNL.  As long as we have to check that,  */
        /* might as well have a general sanity check. */
        if (*cinfo).image_height <= 0 || (*cinfo).image_width <= 0
            || (*cinfo).num_components <= 0 {
            ERREXIT(cinfo, JERR_EMPTY_IMAGE);
        }

        if length != ((*cinfo).num_components as INT32 * 3) {
            ERREXIT(cinfo, JERR_BAD_LENGTH);
        }

        if (*cinfo).comp_info == core::ptr::null_mut() { /* do only once, even if suspend */
            (*cinfo).comp_info = ((*(*cinfo).mem).alloc_small)(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                ((*cinfo).num_components as usize) * core::mem::size_of::<jpeg_component_info>()
            ) as *mut jpeg_component_info;
        }

        ci = 0;
        compptr = (*cinfo).comp_info;
        while ci < (*cinfo).num_components as c_int {
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, (*compptr).component_id);
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c);
            (*compptr).h_samp_factor = (c >> 4) & 15;
            (*compptr).v_samp_factor = (c) & 15;
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, (*compptr).quant_tbl_no);

            TRACEMS4(cinfo, 1, JTRC_SOF_COMPONENT,
                (*compptr).component_id as c_int,
                (*compptr).h_samp_factor as c_int,
                (*compptr).v_samp_factor as c_int,
                (*compptr).quant_tbl_no as c_int);

            ci += 1;
            compptr = compptr.add(1);
        }

        (*(*cinfo).marker).saw_SOF = TRUE;

        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
    }
    TRUE
}

fn get_sos(cinfo: j_decompress_ptr) -> boolean {
    /* Process a SOS marker */
    let mut length: UINT32 = 0;
    let mut i: c_int;
    let mut ci: c_int;
    let mut n: UINT8 = 0;
    let mut c: UINT8 = 0;
    let mut cc: UINT8 = 0;
    let mut compptr: *mut jpeg_component_info;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    unsafe {
        if (*(*cinfo).marker).saw_SOF as u8 == 0 {
            ERREXIT(cinfo, JERR_SOS_NO_SOF);
        }
    }

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, length);
    let mut length = length as INT32;

    INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, n); /* Number of components */

    unsafe {
        if length != (n as INT32 * 2 + 6) || n < 1 || n > MAX_COMPS_IN_SCAN as UINT8 {
            ERREXIT(cinfo, JERR_BAD_LENGTH);
        }
    }

    TRACEMS1(cinfo, 1, JTRC_SOS, n as c_int);

    unsafe {
        (*cinfo).comps_in_scan = n;

        /* Collect the component-spec parameters */

        i = 0;
        while i < n as c_int {
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, cc);
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c);

            ci = 0;
            compptr = (*cinfo).comp_info;
            while ci < (*cinfo).num_components as c_int {
                if cc == (*compptr).component_id {
                    /* id_found: */
                    break;
                }
                ci += 1;
                compptr = compptr.add(1);
            }

            if ci >= (*cinfo).num_components as c_int {
                ERREXIT1(cinfo, JERR_BAD_COMPONENT_ID, cc as c_int);
            }

            (*cinfo).cur_comp_info[i as usize] = compptr;
            (*compptr).dc_tbl_no = (c >> 4) & 15;
            (*compptr).ac_tbl_no = (c) & 15;

            TRACEMS3(cinfo, 1, JTRC_SOS_COMPONENT, cc as c_int,
                (*compptr).dc_tbl_no as c_int,
                (*compptr).ac_tbl_no as c_int);

            i += 1;
        }

        /* Collect the additional scan parameters Ss, Se, Ah/Al. */
        INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c);
        (*cinfo).Ss = c;
        INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c);
        (*cinfo).Se = c;
        INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c);
        (*cinfo).Ah = (c >> 4) & 15;
        (*cinfo).Al = (c) & 15;

        TRACEMS4(cinfo, 1, JTRC_SOS_PARAMS,
            (*cinfo).Ss as c_int,
            (*cinfo).Se as c_int,
            (*cinfo).Ah as c_int,
            (*cinfo).Al as c_int);

        /* Prepare to scan data & restart markers */
        (*(*cinfo).marker).next_restart_num = 0;

        /* Count another SOS marker */
        (*cinfo).input_scan_number += 1;

        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
    }
    TRUE
}

fn get_app0(cinfo: j_decompress_ptr) -> boolean {
    /* Process an APP0 marker */
    let mut length: UINT32 = 0;
    let mut b: [UINT8; JFIF_LEN] = [0; JFIF_LEN];
    let mut buffp: c_int;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, length);
    let mut length = length as INT32 - 2;

    /* See if a JFIF APP0 marker is present */

    if length >= JFIF_LEN as INT32 {
        buffp = 0;
        while buffp < JFIF_LEN as c_int {
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, b[buffp as usize]);
            buffp += 1;
        }
        length -= JFIF_LEN as INT32;

        if b[0] == 0x4A && b[1] == 0x46 && b[2] == 0x49 && b[3] == 0x46 && b[4] == 0 {
            /* Found JFIF APP0 marker: check version */
            /* Major version must be 1, anything else signals an incompatible change.
             * We used to treat this as an error, but now it's a nonfatal warning,
             * because some bozo at Hijaak couldn't read the spec.
             * Minor version should be 0..2, but process anyway if newer.
             */
            if b[5] != 1 {
                WARNMS2(cinfo, JWRN_JFIF_MAJOR, b[5] as c_int, b[6] as c_int);
            } else if b[6] > 2 {
                TRACEMS2(cinfo, 1, JTRC_JFIF_MINOR, b[5] as c_int, b[6] as c_int);
            }
            /* Save info */
            unsafe {
                (*cinfo).saw_JFIF_marker = TRUE;
                (*cinfo).density_unit = b[7];
                (*cinfo).X_density = ((b[8] as c_int) << 8) + (b[9] as c_int);
                (*cinfo).Y_density = ((b[10] as c_int) << 8) + (b[11] as c_int);
            }
            TRACEMS3(cinfo, 1, JTRC_JFIF,
                unsafe { (*cinfo).X_density },
                unsafe { (*cinfo).Y_density },
                unsafe { (*cinfo).density_unit as c_int });
            if (b[12] | b[13]) != 0 {
                TRACEMS2(cinfo, 1, JTRC_JFIF_THUMBNAIL, b[12] as c_int, b[13] as c_int);
            }
            if length != (((b[12] as INT32) * (b[13] as INT32)) * 3) {
                TRACEMS1(cinfo, 1, JTRC_JFIF_BADTHUMBNAILSIZE, length as c_int);
            }
        } else {
            /* Start of APP0 does not match "JFIF" */
            TRACEMS1(cinfo, 1, JTRC_APP0, (length as c_int) + (JFIF_LEN as c_int));
        }
    } else {
        /* Too short to be JFIF marker */
        TRACEMS1(cinfo, 1, JTRC_APP0, length as c_int);
    }

    unsafe {
        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
        if length > 0 { /* skip any remaining data -- could be lots */
            ((*(*cinfo).src).skip_input_data)(cinfo, length as c_int);
        }
    }

    TRUE
}

fn get_app14(cinfo: j_decompress_ptr) -> boolean {
    /* Process an APP14 marker */
    let mut length: UINT32 = 0;
    let mut b: [UINT8; ADOBE_LEN] = [0; ADOBE_LEN];
    let mut buffp: c_int;
    let mut version: c_int;
    let mut flags0: c_int;
    let mut flags1: c_int;
    let mut transform: c_int;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, length);
    let mut length = length as INT32 - 2;

    /* See if an Adobe APP14 marker is present */

    if length >= ADOBE_LEN as INT32 {
        buffp = 0;
        while buffp < ADOBE_LEN as c_int {
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, b[buffp as usize]);
            buffp += 1;
        }
        length -= ADOBE_LEN as INT32;

        if b[0] == 0x41 && b[1] == 0x64 && b[2] == 0x6F && b[3] == 0x62 && b[4] == 0x65 {
            /* Found Adobe APP14 marker */
            version = ((b[5] as c_int) << 8) + (b[6] as c_int);
            flags0 = ((b[7] as c_int) << 8) + (b[8] as c_int);
            flags1 = ((b[9] as c_int) << 8) + (b[10] as c_int);
            transform = b[11] as c_int;
            TRACEMS4(cinfo, 1, JTRC_ADOBE, version, flags0, flags1, transform);
            unsafe {
                (*cinfo).saw_Adobe_marker = TRUE;
                (*cinfo).Adobe_transform = b[11];
            }
        } else {
            /* Start of APP14 does not match "Adobe" */
            TRACEMS1(cinfo, 1, JTRC_APP14, (length as c_int) + (ADOBE_LEN as c_int));
        }
    } else {
        /* Too short to be Adobe marker */
        TRACEMS1(cinfo, 1, JTRC_APP14, length as c_int);
    }

    unsafe {
        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
        if length > 0 { /* skip any remaining data -- could be lots */
            ((*(*cinfo).src).skip_input_data)(cinfo, length as c_int);
        }
    }

    TRUE
}

fn get_dac(cinfo: j_decompress_ptr) -> boolean {
    /* Process a DAC marker */
    let mut length: UINT32 = 0;
    let mut index: UINT8 = 0;
    let mut val: UINT8 = 0;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, length);
    let mut length = length as INT32 - 2;

    while length > 0 {
        INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, index);
        INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, val);

        length -= 2;

        TRACEMS2(cinfo, 1, JTRC_DAC, index as c_int, val as c_int);

        if index as c_int >= ((2 * NUM_ARITH_TBLS) as c_int) {
            ERREXIT1(cinfo, JERR_DAC_INDEX, index as c_int);
        }

        if index as c_int >= NUM_ARITH_TBLS as c_int { /* define AC table */
            unsafe {
                (*cinfo).arith_ac_K[(index as c_int - NUM_ARITH_TBLS as c_int) as usize] = val;
            }
        } else { /* define DC table */
            unsafe {
                (*cinfo).arith_dc_L[index as usize] = (val) & 0x0F;
                (*cinfo).arith_dc_U[index as usize] = (val) >> 4;
                if (*cinfo).arith_dc_L[index as usize] > (*cinfo).arith_dc_U[index as usize] {
                    ERREXIT1(cinfo, JERR_DAC_VALUE, val as c_int);
                }
            }
        }
    }

    unsafe {
        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
    }
    TRUE
}

fn get_dht(cinfo: j_decompress_ptr) -> boolean {
    /* Process a DHT marker */
    let mut length: UINT32 = 0;
    let mut bits: [UINT8; 17] = [0; 17];
    let mut huffval: [UINT8; 256] = [0; 256];
    let mut i: c_int;
    let mut index: UINT8 = 0;
    let mut count: c_int;
    let mut htblptr: *mut *mut JHUFF_TBL;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, length);
    let mut length = length as INT32 - 2;

    while length > 0 {
        INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, index);

        TRACEMS1(cinfo, 1, JTRC_DHT, index as c_int);

        bits[0] = 0;
        count = 0;
        i = 1;
        while i <= 16 {
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, bits[i as usize]);
            count += bits[i as usize] as c_int;
            i += 1;
        }

        length -= 1 + 16;

        TRACEMS8(cinfo, 2, JTRC_HUFFBITS,
            bits[1] as c_int, bits[2] as c_int, bits[3] as c_int, bits[4] as c_int,
            bits[5] as c_int, bits[6] as c_int, bits[7] as c_int, bits[8] as c_int);
        TRACEMS8(cinfo, 2, JTRC_HUFFBITS,
            bits[9] as c_int, bits[10] as c_int, bits[11] as c_int, bits[12] as c_int,
            bits[13] as c_int, bits[14] as c_int, bits[15] as c_int, bits[16] as c_int);

        if count > 256 || (count as INT32) > length {
            ERREXIT(cinfo, JERR_DHT_COUNTS);
        }

        i = 0;
        while i < count {
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, huffval[i as usize]);
            i += 1;
        }

        length -= count as INT32;

        unsafe {
            if (index & 0x10) != 0 { /* AC table definition */
                index -= 0x10;
                htblptr = &mut (*cinfo).ac_huff_tbl_ptrs[(index as c_int) as usize];
            } else { /* DC table definition */
                htblptr = &mut (*cinfo).dc_huff_tbl_ptrs[(index as c_int) as usize];
            }

            if (index as c_int) >= NUM_HUFF_TBLS as c_int {
                ERREXIT1(cinfo, JERR_DHT_INDEX, index as c_int);
            }

            if *htblptr == core::ptr::null_mut() {
                *htblptr = jpeg_alloc_huff_table(cinfo as j_common_ptr);
            }

            MEMCOPY(
                (*(*htblptr)).bits.as_mut_ptr(),
                bits.as_ptr(),
                core::mem::size_of::<[UINT8; 17]>()
            );
            MEMCOPY(
                (*(*htblptr)).huffval.as_mut_ptr(),
                huffval.as_ptr(),
                core::mem::size_of::<[UINT8; 256]>()
            );
        }
    }

    unsafe {
        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
    }
    TRUE
}

fn get_dqt(cinfo: j_decompress_ptr) -> boolean {
    /* Process a DQT marker */
    let mut length: UINT32 = 0;
    let mut n: UINT8 = 0;
    let mut i: c_int;
    let mut prec: c_int;
    let mut tmp: UINT32 = 0;
    let mut quant_ptr: *mut JQUANT_TBL;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, length);
    let mut length = length as INT32 - 2;

    while length > 0 {
        INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, n);
        prec = (n >> 4) as c_int;
        n = (n) & 0x0F;

        TRACEMS2(cinfo, 1, JTRC_DQT, n as c_int, prec);

        if (n as c_int) >= NUM_QUANT_TBLS as c_int {
            ERREXIT1(cinfo, JERR_DQT_INDEX, n as c_int);
        }

        unsafe {
            if (*cinfo).quant_tbl_ptrs[n as usize] == core::ptr::null_mut() {
                (*cinfo).quant_tbl_ptrs[n as usize] = jpeg_alloc_quant_table(cinfo as j_common_ptr);
            }
            quant_ptr = (*cinfo).quant_tbl_ptrs[n as usize];

            i = 0;
            while i < DCTSIZE2 as c_int {
                if prec != 0 {
                    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, tmp);
                } else {
                    let mut tmp_byte: UINT8 = 0;
                    INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, tmp_byte);
                    tmp = tmp_byte as UINT32;
                }
                (*quant_ptr).quantval[i as usize] = tmp as UINT16;
                i += 1;
            }

            i = 0;
            while i < DCTSIZE2 as c_int {
                TRACEMS8(cinfo, 2, JTRC_QUANTVALS,
                    (*quant_ptr).quantval[i as usize] as c_int,
                    (*quant_ptr).quantval[(i + 1) as usize] as c_int,
                    (*quant_ptr).quantval[(i + 2) as usize] as c_int,
                    (*quant_ptr).quantval[(i + 3) as usize] as c_int,
                    (*quant_ptr).quantval[(i + 4) as usize] as c_int,
                    (*quant_ptr).quantval[(i + 5) as usize] as c_int,
                    (*quant_ptr).quantval[(i + 6) as usize] as c_int,
                    (*quant_ptr).quantval[(i + 7) as usize] as c_int);
                i += 8;
            }

            length -= (DCTSIZE2 + 1) as INT32;
            if prec != 0 {
                length -= DCTSIZE2 as INT32;
            }
        }
    }

    unsafe {
        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
    }
    TRUE
}

fn get_dri(cinfo: j_decompress_ptr) -> boolean {
    /* Process a DRI marker */
    let mut length: UINT32 = 0;
    let mut tmp: UINT32 = 0;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, length);

    if length as INT32 != 4 {
        ERREXIT(cinfo, JERR_BAD_LENGTH);
    }

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, tmp);

    TRACEMS1(cinfo, 1, JTRC_DRI, tmp as c_int);

    unsafe {
        (*cinfo).restart_interval = tmp as UINT16;

        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
    }
    TRUE
}

fn skip_variable(cinfo: j_decompress_ptr) -> boolean {
    /* Skip over an unknown or uninteresting variable-length marker */
    let mut length: UINT32 = 0;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    INPUT_2BYTES!(cinfo, datasrc, next_input_byte, bytes_in_buffer, length);

    TRACEMS2(cinfo, 1, JTRC_MISC_MARKER, unsafe { (*cinfo).unread_marker }, (length as INT32 - 2) as c_int);

    unsafe {
        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer); /* do before skip_input_data */
        ((*(*cinfo).src).skip_input_data)(cinfo, (length as INT32 - 2) as c_int);
    }

    TRUE
}

/*
 * Find the next JPEG marker, save it in cinfo->unread_marker.
 * Returns FALSE if had to suspend before reaching a marker;
 * in that case cinfo->unread_marker is unchanged.
 *
 * Note that the result might not be a valid marker code,
 * but it will never be 0 or FF.
 */

fn next_marker(cinfo: j_decompress_ptr) -> boolean {
    let mut c: UINT8 = 0;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    loop {
        INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c);
        /* Skip any non-FF bytes.
         * This may look a bit inefficient, but it will not occur in a valid file.
         * We sync after each discarded byte so that a suspending data source
         * can discard the byte from its buffer.
         */
        while c != 0xFF {
            unsafe {
                (*(*cinfo).marker).discarded_bytes += 1;
            }
            INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c);
        }
        /* This loop swallows any duplicate FF bytes.  Extra FFs are legal as
         * pad bytes, so don't count them in discarded_bytes.  We assume there
         * will not be so many consecutive FF bytes as to overflow a suspending
         * data source's input buffer.
         */
        loop {
            INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c);
            if c != 0xFF {
                break;
            }
        }
        if c != 0 {
            break; /* found a valid marker, exit loop */
        }
        /* Reach here if we found a stuffed-zero data sequence (FF/00).
         * Discard it and loop back to try again.
         */
        unsafe {
            (*(*cinfo).marker).discarded_bytes += 2;
        }
        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
    }

    unsafe {
        if (*(*cinfo).marker).discarded_bytes != 0 {
            WARNMS2(cinfo, JWRN_EXTRANEOUS_DATA, (*(*cinfo).marker).discarded_bytes, c as c_int);
            (*(*cinfo).marker).discarded_bytes = 0;
        }

        (*cinfo).unread_marker = c as c_int;

        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
    }
    TRUE
}

fn first_marker(cinfo: j_decompress_ptr) -> boolean {
    /* Like next_marker, but used to obtain the initial SOI marker. */
    /* For this marker, we do not allow preceding garbage or fill; otherwise,
     * we might well scan an entire input file before realizing it ain't JPEG.
     * If an application wants to process non-JFIF files, it must seek to the
     * SOI before calling the JPEG library.
     */
    let mut c: UINT8 = 0;
    let mut c2: UINT8 = 0;

    let (datasrc, mut next_input_byte, mut bytes_in_buffer) = INPUT_VARS!(cinfo);

    INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c);
    INPUT_BYTE!(cinfo, datasrc, next_input_byte, bytes_in_buffer, c2);
    if c != 0xFF || c2 != (0xd8 as UINT8) {
        ERREXIT2(cinfo, JERR_NO_SOI, c as c_int, c2 as c_int);
    }

    unsafe {
        (*cinfo).unread_marker = c2 as c_int;

        INPUT_SYNC!(cinfo, datasrc, next_input_byte, bytes_in_buffer);
    }
    TRUE
}

/*
 * Read markers until SOS or EOI.
 *
 * Returns same codes as are defined for jpeg_consume_input:
 * JPEG_SUSPENDED, JPEG_REACHED_SOS, or JPEG_REACHED_EOI.
 */

fn read_markers(cinfo: j_decompress_ptr) -> c_int {
    /* Outer loop repeats once for each marker. */
    loop {
        unsafe {
            /* Collect the marker proper, unless we already did. */
            /* NB: first_marker() enforces the requirement that SOI appear first. */
            if (*cinfo).unread_marker == 0 {
                if (*(*cinfo).marker).saw_SOI as u8 == 0 {
                    if first_marker(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                } else {
                    if next_marker(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }
            }
            /* At this point cinfo->unread_marker contains the marker code and the
             * input point is just past the marker proper, but before any parameters.
             * A suspension will cause us to return with this state still true.
             */
            match (*cinfo).unread_marker {
                0xd8 => { /* M_SOI */
                    if get_soi(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                0xc0 | 0xc1 => { /* M_SOF0 | M_SOF1 - Baseline or Extended sequential, Huffman */
                    if get_sof(cinfo, FALSE, FALSE) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                0xc2 => { /* M_SOF2 - Progressive, Huffman */
                    if get_sof(cinfo, TRUE, FALSE) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                0xc9 => { /* M_SOF9 - Extended sequential, arithmetic */
                    if get_sof(cinfo, FALSE, TRUE) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                0xca => { /* M_SOF10 - Progressive, arithmetic */
                    if get_sof(cinfo, TRUE, TRUE) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                /* Currently unsupported SOFn types */
                0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc8 | 0xcb | 0xcd | 0xce | 0xcf => {
                    /* M_SOF3 - Lossless, Huffman */
                    /* M_SOF5 - Differential sequential, Huffman */
                    /* M_SOF6 - Differential progressive, Huffman */
                    /* M_SOF7 - Differential lossless, Huffman */
                    /* M_JPG - Reserved for JPEG extensions */
                    /* M_SOF11 - Lossless, arithmetic */
                    /* M_SOF13 - Differential sequential, arithmetic */
                    /* M_SOF14 - Differential progressive, arithmetic */
                    /* M_SOF15 - Differential lossless, arithmetic */
                    ERREXIT1(cinfo, JERR_SOF_UNSUPPORTED, (*cinfo).unread_marker);
                }

                0xda => { /* M_SOS */
                    if get_sos(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                    (*cinfo).unread_marker = 0; /* processed the marker */
                    return JPEG_REACHED_SOS;
                }

                0xd9 => { /* M_EOI */
                    TRACEMS(cinfo, 1, JTRC_EOI);
                    (*cinfo).unread_marker = 0; /* processed the marker */
                    return JPEG_REACHED_EOI;
                }

                0xcc => { /* M_DAC */
                    if get_dac(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                0xc4 => { /* M_DHT */
                    if get_dht(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                0xdb => { /* M_DQT */
                    if get_dqt(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                0xdd => { /* M_DRI */
                    if get_dri(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                0xe0 | 0xe1 | 0xe2 | 0xe3 | 0xe4 | 0xe5 | 0xe6 | 0xe7 | 0xe8 | 0xe9 | 0xea | 0xeb | 0xec | 0xed | 0xee | 0xef => {
                    /* M_APP0 through M_APP15 */
                    let appn_index = ((*cinfo).unread_marker - 0xe0) as usize;
                    if appn_index < 16 {
                        let process_fn = (*(*cinfo).marker).process_APPn[appn_index];
                        if process_fn(cinfo) as u8 == 0 {
                            return JPEG_SUSPENDED;
                        }
                    }
                }

                0xfe => { /* M_COM */
                    let process_fn = (*(*cinfo).marker).process_COM;
                    if process_fn(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                0xd0 | 0xd1 | 0xd2 | 0xd3 | 0xd4 | 0xd5 | 0xd6 | 0xd7 | 0x01 => {
                    /* M_RST0 through M_RST7 and M_TEM - all parameterless */
                    TRACEMS1(cinfo, 1, JTRC_PARMLESS_MARKER, (*cinfo).unread_marker);
                }

                0xdc => { /* M_DNL - Ignore DNL ... perhaps the wrong thing */
                    if skip_variable(cinfo) as u8 == 0 {
                        return JPEG_SUSPENDED;
                    }
                }

                _ => { /* must be DHP, EXP, JPGn, or RESn */
                    /* For now, we treat the reserved markers as fatal errors since they are
                     * likely to be used to signal incompatible JPEG Part 3 extensions.
                     * Once the JPEG 3 version-number marker is well defined, this code
                     * ought to change!
                     */
                    ERREXIT1(cinfo, JERR_UNKNOWN_MARKER, (*cinfo).unread_marker);
                }
            }
            /* Successfully processed marker, so reset state variable */
            (*cinfo).unread_marker = 0;
        }
    } /* end loop */
}

/*
 * Read a restart marker, which is expected to appear next in the datastream;
 * if the marker is not there, take appropriate recovery action.
 * Returns FALSE if suspension is required.
 *
 * This is called by the entropy decoder after it has read an appropriate
 * number of MCUs.  cinfo->unread_marker may be nonzero if the entropy decoder
 * has already read a marker from the data source.  Under normal conditions
 * cinfo->unread_marker will be reset to 0 before returning; if not reset,
 * it holds a marker which the decoder will be unable to read past.
 */

fn read_restart_marker(cinfo: j_decompress_ptr) -> boolean {
    /* Obtain a marker unless we already did. */
    /* Note that next_marker will complain if it skips any data. */
    unsafe {
        if (*cinfo).unread_marker == 0 {
            if next_marker(cinfo) as u8 == 0 {
                return FALSE;
            }
        }

        if (*cinfo).unread_marker == ((0xd0 as c_int) + (*(*cinfo).marker).next_restart_num as c_int) {
            /* Normal case --- swallow the marker and let entropy decoder continue */
            TRACEMS1(cinfo, 2, JTRC_RST, (*(*cinfo).marker).next_restart_num as c_int);
            (*cinfo).unread_marker = 0;
        } else {
            /* Uh-oh, the restart markers have been messed up. */
            /* Let the data source manager determine how to resync. */
            if ((*(*cinfo).src).resync_to_restart)(cinfo, (*(*cinfo).marker).next_restart_num as c_int) as u8 == 0 {
                return FALSE;
            }
        }

        /* Update next-restart state */
        (*(*cinfo).marker).next_restart_num = ((*(*cinfo).marker).next_restart_num + 1) & 7;
    }

    TRUE
}

/*
 * This is the default resync_to_restart method for data source managers
 * to use if they don't have any better approach.  Some data source managers
 * may be able to back up, or may have additional knowledge about the data
 * which permits a more intelligent recovery strategy; such managers would
 * presumably supply their own resync method.
 *
 * read_restart_marker calls resync_to_restart if it finds a marker other than
 * the restart marker it was expecting.  (This code is *not* used unless
 * a nonzero restart interval has been declared.)  cinfo->unread_marker is
 * the marker code actually found (might be anything, except 0 or FF).
 * The desired restart marker number (0..7) is passed as a parameter.
 * This routine is supposed to apply whatever error recovery strategy seems
 * appropriate in order to position the input stream to the next data segment.
 * Note that cinfo->unread_marker is treated as a marker appearing before
 * the current data-source input point; usually it should be reset to zero
 * before returning.
 * Returns FALSE if suspension is required.
 *
 * This implementation is substantially constrained by wanting to treat the
 * input as a data stream; this means we can't back up.  Therefore, we have
 * only the following actions to work with:
 *   1. Simply discard the marker and let the entropy decoder resume at next
 *      byte of file.
 *   2. Read forward until we find another marker, discarding intervening
 *      data.  (In theory we could look ahead within the current bufferload,
 *      without having to discard data if we don't find the desired marker.
 *      This idea is not implemented here, in part because it makes behavior
 *      dependent on buffer size and chance buffer-boundary positions.)
 *   3. Leave the marker unread (by failing to zero cinfo->unread_marker).
 *      This will cause the entropy decoder to process an empty data segment,
 *      inserting dummy zeroes, and then we will reprocess the marker.
 *
 * #2 is appropriate if we think the desired marker lies ahead, while #3 is
 * appropriate if the found marker is a future restart marker (indicating
 * that we have missed the desired restart marker, probably because it got
 * corrupted).
 * We apply #2 or #3 if the found marker is a restart marker no more than
 * two counts behind or ahead of the expected one.  We also apply #2 if the
 * found marker is not a legal JPEG marker code (it's certainly bogus data).
 * If the found marker is a restart marker more than 2 counts away, we do #1
 * (too much risk that the marker is erroneous; with luck we will be able to
 * resync at some future point).
 * For any valid non-restart JPEG marker, we apply #3.  This keeps us from
 * overrunning the end of a scan.  An implementation limited to single-scan
 * files might find it better to apply #2 for markers other than EOI, since
 * any other marker would have to be bogus data in that case.
 */

pub fn jpeg_resync_to_restart(cinfo: j_decompress_ptr, desired: c_int) -> boolean {
    let mut marker = unsafe { (*cinfo).unread_marker };
    let mut action = 1;

    /* Always put up a warning. */
    WARNMS2(cinfo, JWRN_MUST_RESYNC, marker, desired);

    /* Outer loop handles repeated decision after scanning forward. */
    loop {
        if marker < (0xc0 as c_int) {
            action = 2; /* invalid marker */
        } else if marker < (0xd0 as c_int) || marker > (0xd7 as c_int) {
            action = 3; /* valid non-restart marker */
        } else {
            if marker == ((0xd0 as c_int) + ((desired + 1) & 7)) ||
                marker == ((0xd0 as c_int) + ((desired + 2) & 7)) {
                action = 3; /* one of the next two expected restarts */
            } else if marker == ((0xd0 as c_int) + ((desired - 1) & 7)) ||
                      marker == ((0xd0 as c_int) + ((desired - 2) & 7)) {
                action = 2; /* a prior restart, so advance */
            } else {
                action = 1; /* desired restart or too far away */
            }
        }
        TRACEMS2(cinfo, 4, JTRC_RECOVERY_ACTION, marker, action);
        match action {
            1 => {
                /* Discard marker and let entropy decoder resume processing. */
                unsafe {
                    (*cinfo).unread_marker = 0;
                }
                return TRUE;
            }
            2 => {
                /* Scan to the next marker, and repeat the decision loop. */
                if next_marker(cinfo) as u8 == 0 {
                    return FALSE;
                }
                unsafe {
                    marker = (*cinfo).unread_marker;
                }
            }
            3 => {
                /* Return without advancing past this marker. */
                /* Entropy decoder will be forced to process an empty segment. */
                return TRUE;
            }
            _ => {}
        }
    } /* end loop */
}

/*
 * Reset marker processing state to begin a fresh datastream.
 */

fn reset_marker_reader(cinfo: j_decompress_ptr) {
    unsafe {
        (*cinfo).comp_info = core::ptr::null_mut(); /* until allocated by get_sof */
        (*cinfo).input_scan_number = 0; /* no SOS seen yet */
        (*cinfo).unread_marker = 0; /* no pending marker */
        (*(*cinfo).marker).saw_SOI = FALSE; /* set internal state too */
        (*(*cinfo).marker).saw_SOF = FALSE;
        (*(*cinfo).marker).discarded_bytes = 0;
    }
}

/*
 * Initialize the marker reader module.
 * This is called only once, when the decompression object is created.
 */

pub fn jinit_marker_reader(cinfo: j_decompress_ptr) {
    unsafe {
        /* Create subobject in permanent pool */
        (*cinfo).marker = ((*(*cinfo).mem).alloc_small)(
            cinfo as j_common_ptr,
            JPOOL_PERMANENT,
            core::mem::size_of::<jpeg_marker_reader>()
        ) as *mut jpeg_marker_reader;

        /* Initialize method pointers */
        (*(*cinfo).marker).reset_marker_reader = reset_marker_reader;
        (*(*cinfo).marker).read_markers = read_markers;
        (*(*cinfo).marker).read_restart_marker = read_restart_marker;
        (*(*cinfo).marker).process_COM = skip_variable;

        for i in 0..16 {
            (*(*cinfo).marker).process_APPn[i] = skip_variable;
        }
        (*(*cinfo).marker).process_APPn[0] = get_app0;
        (*(*cinfo).marker).process_APPn[14] = get_app14;

        /* Reset marker processing state */
        reset_marker_reader(cinfo);
    }
}
