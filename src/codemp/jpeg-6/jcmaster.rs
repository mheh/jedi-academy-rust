/*
 * jcmaster.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains master control logic for the JPEG compressor.
 * These routines are concerned with parameter validation, initial setup,
 * and inter-pass control (determining the number of passes and the work
 * to be done in each pass).
 */

#![allow(non_snake_case)]

use core::ffi::{c_int, c_long, c_uint};

/* Opaque external types from jpeglib.h */
pub type j_common_ptr = *mut core::ffi::c_void;
pub type j_compress_ptr = *mut jpeg_compress_struct;

#[repr(C)]
pub struct jpeg_compress_struct {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_component_info {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_scan_info {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_comp_master {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_color_converter {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_downsampler {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_prep_controller {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_forward_dct {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_entropy_encoder {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_c_coef_controller {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_c_main_controller {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_marker_writer {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_progress_mgr {
    /* Opaque structure used via pointer */
    _opaque: [u8; 0],
}

/* External function declarations from JPEG library */
extern "C" {
    pub fn ERREXIT(cinfo: j_compress_ptr, code: c_int);
    pub fn ERREXIT1(cinfo: j_compress_ptr, code: c_int, p1: c_int);
    pub fn ERREXIT2(cinfo: j_compress_ptr, code: c_int, p1: c_int, p2: c_int);
    pub fn jdiv_round_up(a: c_long, b: c_long) -> c_int;
}

/* Constants from jpeglib.h */
const JPEG_MAX_DIMENSION: c_int = 65500;
const MAX_COMPONENTS: c_int = 10;
const MAX_SAMP_FACTOR: c_int = 4;
const DCTSIZE: c_int = 8;
const DCTSIZE2: c_int = 64;
const MAX_COMPS_IN_SCAN: c_int = 4;
const C_MAX_BLOCKS_IN_MCU: c_int = 10;
const BITS_IN_JSAMPLE: c_int = 8;
const TRUE: c_int = 1;
const FALSE: c_int = 0;

/* Error codes from jerror.h (stubs) */
const JERR_EMPTY_IMAGE: c_int = 50;
const JERR_IMAGE_TOO_BIG: c_int = 51;
const JERR_WIDTH_OVERFLOW: c_int = 52;
const JERR_BAD_PRECISION: c_int = 53;
const JERR_COMPONENT_COUNT: c_int = 54;
const JERR_BAD_SAMPLING: c_int = 55;
const JERR_NOT_COMPILED: c_int = 56;
const JERR_BAD_SCAN_SCRIPT: c_int = 57;
const JERR_BAD_PROG_SCRIPT: c_int = 58;
const JERR_BAD_MCU_SIZE: c_int = 59;
const JERR_MISSING_DATA: c_int = 60;

/* JBUF mode constants */
const JBUF_PASS_THRU: c_int = 0;
const JBUF_SAVE_AND_PASS: c_int = 1;
const JBUF_CRANK_DEST: c_int = 2;

/* Macro helpers */
#[inline]
fn MAX(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
}

#[inline]
fn MIN(a: c_long, b: c_long) -> c_long {
    if a < b { a } else { b }
}

#[inline]
fn SIZEOF<T>(_: *const T) -> usize {
    core::mem::size_of::<T>()
}

pub type JDIMENSION = c_uint;

/* Private state */

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
enum c_pass_type {
    main_pass = 0,      /* input data, also do first output step */
    huff_opt_pass = 1,  /* Huffman code optimization pass */
    output_pass = 2     /* data output pass */
}

#[repr(C)]
struct my_comp_master {
    pub_field: jpeg_comp_master,  /* public fields */
    pass_type: c_pass_type,       /* the type of the current pass */
    pass_number: c_int,           /* # of passes completed */
    total_passes: c_int,          /* total # of passes needed */
    scan_number: c_int,           /* current index in scan_info[] */
}

type my_master_ptr = *mut my_comp_master;

/*
 * Support routines that do various essential calculations.
 */

unsafe fn initial_setup(cinfo: j_compress_ptr) {
    /* Do computations that are needed before master selection phase */
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut samplesperrow: c_long;
    let mut jd_samplesperrow: JDIMENSION;

    /* Sanity check on image dimensions */
    if (*cinfo).image_height <= 0 || (*cinfo).image_width <= 0
        || (*cinfo).num_components <= 0 || (*cinfo).input_components <= 0 {
        ERREXIT(cinfo, JERR_EMPTY_IMAGE);
    }

    /* Make sure image isn't bigger than I can handle */
    if ((*cinfo).image_height as c_long) > (JPEG_MAX_DIMENSION as c_long) ||
        ((*cinfo).image_width as c_long) > (JPEG_MAX_DIMENSION as c_long) {
        ERREXIT1(cinfo, JERR_IMAGE_TOO_BIG, JPEG_MAX_DIMENSION as c_int);
    }

    /* Width of an input scanline must be representable as JDIMENSION. */
    samplesperrow = ((*cinfo).image_width as c_long) * ((*cinfo).input_components as c_long);
    jd_samplesperrow = samplesperrow as JDIMENSION;
    if (jd_samplesperrow as c_long) != samplesperrow {
        ERREXIT(cinfo, JERR_WIDTH_OVERFLOW);
    }

    /* For now, precision must match compiled-in value... */
    if (*cinfo).data_precision != BITS_IN_JSAMPLE {
        ERREXIT1(cinfo, JERR_BAD_PRECISION, (*cinfo).data_precision);
    }

    /* Check that number of components won't exceed internal array sizes */
    if (*cinfo).num_components > MAX_COMPONENTS {
        ERREXIT2(cinfo, JERR_COMPONENT_COUNT, (*cinfo).num_components,
                 MAX_COMPONENTS);
    }

    /* Compute maximum sampling factors; check factor validity */
    (*cinfo).max_h_samp_factor = 1;
    (*cinfo).max_v_samp_factor = 1;
    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        if (*compptr).h_samp_factor <= 0 || (*compptr).h_samp_factor > MAX_SAMP_FACTOR ||
           (*compptr).v_samp_factor <= 0 || (*compptr).v_samp_factor > MAX_SAMP_FACTOR {
            ERREXIT(cinfo, JERR_BAD_SAMPLING);
        }
        (*cinfo).max_h_samp_factor = MAX((*cinfo).max_h_samp_factor,
                                         (*compptr).h_samp_factor);
        (*cinfo).max_v_samp_factor = MAX((*cinfo).max_v_samp_factor,
                                         (*compptr).v_samp_factor);
        ci += 1;
        compptr = compptr.offset(1);
    }

    /* Compute dimensions of components */
    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Fill in the correct component_index value; don't rely on application */
        (*compptr).component_index = ci;
        /* For compression, we never do DCT scaling. */
        (*compptr).DCT_scaled_size = DCTSIZE;
        /* Size in DCT blocks */
        (*compptr).width_in_blocks = (jdiv_round_up(
            ((*cinfo).image_width as c_long) * ((*compptr).h_samp_factor as c_long),
            (((*cinfo).max_h_samp_factor * DCTSIZE) as c_long)
        )) as JDIMENSION;
        (*compptr).height_in_blocks = (jdiv_round_up(
            ((*cinfo).image_height as c_long) * ((*compptr).v_samp_factor as c_long),
            (((*cinfo).max_v_samp_factor * DCTSIZE) as c_long)
        )) as JDIMENSION;
        /* Size in samples */
        (*compptr).downsampled_width = (jdiv_round_up(
            ((*cinfo).image_width as c_long) * ((*compptr).h_samp_factor as c_long),
            ((*cinfo).max_h_samp_factor as c_long)
        )) as JDIMENSION;
        (*compptr).downsampled_height = (jdiv_round_up(
            ((*cinfo).image_height as c_long) * ((*compptr).v_samp_factor as c_long),
            ((*cinfo).max_v_samp_factor as c_long)
        )) as JDIMENSION;
        /* Mark component needed (this flag isn't actually used for compression) */
        (*compptr).component_needed = TRUE;

        ci += 1;
        compptr = compptr.offset(1);
    }

    /* Compute number of fully interleaved MCU rows (number of times that
     * main controller will call coefficient controller).
     */
    (*cinfo).total_iMCU_rows = (jdiv_round_up(
        ((*cinfo).image_height as c_long),
        (((*cinfo).max_v_samp_factor * DCTSIZE) as c_long)
    )) as JDIMENSION;
}

#[cfg(feature = "C_MULTISCAN_FILES_SUPPORTED")]
unsafe fn validate_script(cinfo: j_compress_ptr) {
    /* Verify that the scan script in cinfo->scan_info[] is valid; also
     * determine whether it uses progressive JPEG, and set cinfo->progressive_mode.
     */
    let mut scanptr: *const jpeg_scan_info;
    let mut scanno: c_int;
    let mut ncomps: c_int;
    let mut ci: c_int;
    let mut coefi: c_int;
    let mut thisi: c_int;
    let mut Ss: c_int;
    let mut Se: c_int;
    let mut Ah: c_int;
    let mut Al: c_int;
    let mut component_sent: [c_int; 10] = [0; 10];
    #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
    let mut last_bitpos_ptr: *mut c_int;
    #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
    let mut last_bitpos: [[c_int; 64]; 10] = [[0; 64]; 10];

    if (*cinfo).num_scans <= 0 {
        ERREXIT1(cinfo, JERR_BAD_SCAN_SCRIPT, 0);
    }

    /* For sequential JPEG, all scans must have Ss=0, Se=DCTSIZE2-1;
     * for progressive JPEG, no scan can have this.
     */
    scanptr = (*cinfo).scan_info;
    if (*scanptr).Ss != 0 || (*scanptr).Se != DCTSIZE2 - 1 {
        #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
        {
            (*cinfo).progressive_mode = TRUE;
            last_bitpos_ptr = &mut last_bitpos[0][0];
            ci = 0;
            while ci < (*cinfo).num_components {
                coefi = 0;
                while coefi < DCTSIZE2 {
                    *last_bitpos_ptr = -1;
                    last_bitpos_ptr = last_bitpos_ptr.offset(1);
                    coefi += 1;
                }
                ci += 1;
            }
        }
        #[cfg(not(feature = "C_PROGRESSIVE_SUPPORTED"))]
        {
            ERREXIT(cinfo, JERR_NOT_COMPILED);
        }
    } else {
        (*cinfo).progressive_mode = FALSE;
        ci = 0;
        while ci < (*cinfo).num_components {
            component_sent[ci as usize] = FALSE;
            ci += 1;
        }
    }

    scanno = 1;
    while scanno <= (*cinfo).num_scans {
        /* Validate component indexes */
        ncomps = (*scanptr).comps_in_scan;
        if ncomps <= 0 || ncomps > MAX_COMPS_IN_SCAN {
            ERREXIT2(cinfo, JERR_COMPONENT_COUNT, ncomps, MAX_COMPS_IN_SCAN);
        }
        ci = 0;
        while ci < ncomps {
            thisi = (*scanptr).component_index[ci as usize];
            if thisi < 0 || thisi >= (*cinfo).num_components {
                ERREXIT1(cinfo, JERR_BAD_SCAN_SCRIPT, scanno);
            }
            /* Components must appear in SOF order within each scan */
            if ci > 0 && thisi <= (*scanptr).component_index[(ci - 1) as usize] {
                ERREXIT1(cinfo, JERR_BAD_SCAN_SCRIPT, scanno);
            }
            ci += 1;
        }
        /* Validate progression parameters */
        Ss = (*scanptr).Ss;
        Se = (*scanptr).Se;
        Ah = (*scanptr).Ah;
        Al = (*scanptr).Al;
        if (*cinfo).progressive_mode != FALSE {
            #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
            {
                if Ss < 0 || Ss >= DCTSIZE2 || Se < Ss || Se >= DCTSIZE2 ||
                   Ah < 0 || Ah > 13 || Al < 0 || Al > 13 {
                    ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
                }
                if Ss == 0 {
                    if Se != 0 {        /* DC and AC together not OK */
                        ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
                    }
                } else {
                    if ncomps != 1 {    /* AC scans must be for only one component */
                        ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
                    }
                }
                ci = 0;
                while ci < ncomps {
                    last_bitpos_ptr = &mut last_bitpos[(*scanptr).component_index[ci as usize] as usize][0];
                    if Ss != 0 && *last_bitpos_ptr < 0 {  /* AC without prior DC scan */
                        ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
                    }
                    coefi = Ss;
                    while coefi <= Se {
                        if *last_bitpos_ptr.offset(coefi as isize) < 0 {
                            /* first scan of this coefficient */
                            if Ah != 0 {
                                ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
                            }
                        } else {
                            /* not first scan */
                            if Ah != *last_bitpos_ptr.offset(coefi as isize) || Al != Ah - 1 {
                                ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
                            }
                        }
                        *last_bitpos_ptr.offset(coefi as isize) = Al;
                        coefi += 1;
                    }
                    ci += 1;
                }
            }
        } else {
            /* For sequential JPEG, all progression parameters must be these: */
            if Ss != 0 || Se != DCTSIZE2 - 1 || Ah != 0 || Al != 0 {
                ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
            }
            /* Make sure components are not sent twice */
            ci = 0;
            while ci < ncomps {
                thisi = (*scanptr).component_index[ci as usize];
                if component_sent[thisi as usize] != FALSE {
                    ERREXIT1(cinfo, JERR_BAD_SCAN_SCRIPT, scanno);
                }
                component_sent[thisi as usize] = TRUE;
                ci += 1;
            }
        }
        scanptr = scanptr.offset(1);
        scanno += 1;
    }

    /* Now verify that everything got sent. */
    if (*cinfo).progressive_mode != FALSE {
        #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
        {
            /* For progressive mode, we only check that at least some DC data
             * got sent for each component; the spec does not require that all bits
             * of all coefficients be transmitted.  Would it be wiser to enforce
             * transmission of all coefficient bits??
             */
            ci = 0;
            while ci < (*cinfo).num_components {
                if last_bitpos[ci as usize][0] < 0 {
                    ERREXIT(cinfo, JERR_MISSING_DATA);
                }
                ci += 1;
            }
        }
    } else {
        ci = 0;
        while ci < (*cinfo).num_components {
            if component_sent[ci as usize] == FALSE {
                ERREXIT(cinfo, JERR_MISSING_DATA);
            }
            ci += 1;
        }
    }
}

unsafe fn select_scan_parameters(cinfo: j_compress_ptr) {
    /* Set up the scan parameters for the current scan */
    let mut ci: c_int;

    #[cfg(feature = "C_MULTISCAN_FILES_SUPPORTED")]
    {
        if (*cinfo).scan_info != core::ptr::null_mut() {
            /* Prepare for current scan --- the script is already validated */
            let master = (*cinfo).master as my_master_ptr;
            let scanptr = (*cinfo).scan_info.offset((*master).scan_number as isize);

            (*cinfo).comps_in_scan = (*scanptr).comps_in_scan;
            ci = 0;
            while ci < (*scanptr).comps_in_scan {
                (*cinfo).cur_comp_info[ci as usize] =
                    &mut *(*cinfo).comp_info.offset((*scanptr).component_index[ci as usize] as isize);
                ci += 1;
            }
            (*cinfo).Ss = (*scanptr).Ss;
            (*cinfo).Se = (*scanptr).Se;
            (*cinfo).Ah = (*scanptr).Ah;
            (*cinfo).Al = (*scanptr).Al;
        } else {
            /* Prepare for single sequential-JPEG scan containing all components */
            if (*cinfo).num_components > MAX_COMPS_IN_SCAN {
                ERREXIT2(cinfo, JERR_COMPONENT_COUNT, (*cinfo).num_components,
                         MAX_COMPS_IN_SCAN);
            }
            (*cinfo).comps_in_scan = (*cinfo).num_components;
            ci = 0;
            while ci < (*cinfo).num_components {
                (*cinfo).cur_comp_info[ci as usize] =
                    &mut *(*cinfo).comp_info.offset(ci as isize);
                ci += 1;
            }
            (*cinfo).Ss = 0;
            (*cinfo).Se = DCTSIZE2 - 1;
            (*cinfo).Ah = 0;
            (*cinfo).Al = 0;
        }
    }

    #[cfg(not(feature = "C_MULTISCAN_FILES_SUPPORTED"))]
    {
        /* Prepare for single sequential-JPEG scan containing all components */
        if (*cinfo).num_components > MAX_COMPS_IN_SCAN {
            ERREXIT2(cinfo, JERR_COMPONENT_COUNT, (*cinfo).num_components,
                     MAX_COMPS_IN_SCAN);
        }
        (*cinfo).comps_in_scan = (*cinfo).num_components;
        ci = 0;
        while ci < (*cinfo).num_components {
            (*cinfo).cur_comp_info[ci as usize] =
                &mut *(*cinfo).comp_info.offset(ci as isize);
            ci += 1;
        }
        (*cinfo).Ss = 0;
        (*cinfo).Se = DCTSIZE2 - 1;
        (*cinfo).Ah = 0;
        (*cinfo).Al = 0;
    }
}

unsafe fn per_scan_setup(cinfo: j_compress_ptr) {
    /* Do computations that are needed before processing a JPEG scan */
    /* cinfo->comps_in_scan and cinfo->cur_comp_info[] are already set */
    let mut ci: c_int;
    let mut mcublks: c_int;
    let mut tmp: c_int;
    let mut compptr: *mut jpeg_component_info;

    if (*cinfo).comps_in_scan == 1 {

        /* Noninterleaved (single-component) scan */
        compptr = *(*cinfo).cur_comp_info[0];

        /* Overall image size in MCUs */
        (*cinfo).MCUs_per_row = (*compptr).width_in_blocks;
        (*cinfo).MCU_rows_in_scan = (*compptr).height_in_blocks;

        /* For noninterleaved scan, always one block per MCU */
        (*compptr).MCU_width = 1;
        (*compptr).MCU_height = 1;
        (*compptr).MCU_blocks = 1;
        (*compptr).MCU_sample_width = DCTSIZE;
        (*compptr).last_col_width = 1;
        /* For noninterleaved scans, it is convenient to define last_row_height
         * as the number of block rows present in the last iMCU row.
         */
        tmp = ((*compptr).height_in_blocks as c_int) % (*compptr).v_samp_factor;
        if tmp == 0 { tmp = (*compptr).v_samp_factor; }
        (*compptr).last_row_height = tmp;

        /* Prepare array describing MCU composition */
        (*cinfo).blocks_in_MCU = 1;
        (*cinfo).MCU_membership[0] = 0;

    } else {

        /* Interleaved (multi-component) scan */
        if (*cinfo).comps_in_scan <= 0 || (*cinfo).comps_in_scan > MAX_COMPS_IN_SCAN {
            ERREXIT2(cinfo, JERR_COMPONENT_COUNT, (*cinfo).comps_in_scan,
                     MAX_COMPS_IN_SCAN);
        }

        /* Overall image size in MCUs */
        (*cinfo).MCUs_per_row = (jdiv_round_up(
            ((*cinfo).image_width as c_long),
            (((*cinfo).max_h_samp_factor * DCTSIZE) as c_long)
        )) as JDIMENSION;
        (*cinfo).MCU_rows_in_scan = (jdiv_round_up(
            ((*cinfo).image_height as c_long),
            (((*cinfo).max_v_samp_factor * DCTSIZE) as c_long)
        )) as JDIMENSION;

        (*cinfo).blocks_in_MCU = 0;

        ci = 0;
        while ci < (*cinfo).comps_in_scan {
            compptr = *(*cinfo).cur_comp_info[ci as usize];
            /* Sampling factors give # of blocks of component in each MCU */
            (*compptr).MCU_width = (*compptr).h_samp_factor;
            (*compptr).MCU_height = (*compptr).v_samp_factor;
            (*compptr).MCU_blocks = (*compptr).MCU_width * (*compptr).MCU_height;
            (*compptr).MCU_sample_width = (*compptr).MCU_width * DCTSIZE;
            /* Figure number of non-dummy blocks in last MCU column & row */
            tmp = ((*compptr).width_in_blocks as c_int) % (*compptr).MCU_width;
            if tmp == 0 { tmp = (*compptr).MCU_width; }
            (*compptr).last_col_width = tmp;
            tmp = ((*compptr).height_in_blocks as c_int) % (*compptr).MCU_height;
            if tmp == 0 { tmp = (*compptr).MCU_height; }
            (*compptr).last_row_height = tmp;
            /* Prepare array describing MCU composition */
            mcublks = (*compptr).MCU_blocks;
            if (*cinfo).blocks_in_MCU + mcublks > C_MAX_BLOCKS_IN_MCU {
                ERREXIT(cinfo, JERR_BAD_MCU_SIZE);
            }
            while mcublks > 0 {
                (*cinfo).MCU_membership[(*cinfo).blocks_in_MCU as usize] = ci;
                (*cinfo).blocks_in_MCU += 1;
                mcublks -= 1;
            }
            ci += 1;
        }

    }

    /* Convert restart specified in rows to actual MCU count. */
    /* Note that count must fit in 16 bits, so we provide limiting. */
    if (*cinfo).restart_in_rows > 0 {
        let nominal = ((*cinfo).restart_in_rows as c_long) * ((*cinfo).MCUs_per_row as c_long);
        (*cinfo).restart_interval = (MIN(nominal, 65535)) as c_uint;
    }
}

/*
 * Per-pass setup.
 * This is called at the beginning of each pass.  We determine which modules
 * will be active during this pass and give them appropriate start_pass calls.
 * We also set is_last_pass to indicate whether any more passes will be
 * required.
 */

#[allow(non_upper_case_globals)]
extern "C" {
    static mut cinfo: j_compress_ptr;
}

extern "C" fn prepare_for_pass(cinfo: j_compress_ptr) {
    unsafe {
        let master = (*cinfo).master as my_master_ptr;

        match (*master).pass_type {
            c_pass_type::main_pass => {
                /* Initial pass: will collect input data, and do either Huffman
                 * optimization or data output for the first scan.
                 */
                select_scan_parameters(cinfo);
                per_scan_setup(cinfo);
                if (*cinfo).raw_data_in == FALSE {
                    ((*(*cinfo).cconvert).start_pass.unwrap())(cinfo);
                    ((*(*cinfo).downsample).start_pass.unwrap())(cinfo);
                    ((*(*cinfo).prep).start_pass.unwrap())(cinfo, JBUF_PASS_THRU);
                }
                ((*(*cinfo).fdct).start_pass.unwrap())(cinfo);
                ((*(*cinfo).entropy).start_pass.unwrap())(cinfo, (*cinfo).optimize_coding);
                ((*(*cinfo).coef).start_pass.unwrap())(cinfo,
                    if (*master).total_passes > 1 {
                        JBUF_SAVE_AND_PASS
                    } else {
                        JBUF_PASS_THRU
                    });
                ((*(*cinfo).main).start_pass.unwrap())(cinfo, JBUF_PASS_THRU);
                if (*cinfo).optimize_coding != FALSE {
                    /* No immediate data output; postpone writing frame/scan headers */
                    (*master).pub_field.call_pass_startup = FALSE;
                } else {
                    /* Will write frame/scan headers at first jpeg_write_scanlines call */
                    (*master).pub_field.call_pass_startup = TRUE;
                }
            }
            #[cfg(feature = "ENTROPY_OPT_SUPPORTED")]
            c_pass_type::huff_opt_pass => {
                /* Do Huffman optimization for a scan after the first one. */
                select_scan_parameters(cinfo);
                per_scan_setup(cinfo);
                if (*cinfo).Ss != 0 || (*cinfo).Ah == 0 || (*cinfo).arith_code != FALSE {
                    ((*(*cinfo).entropy).start_pass.unwrap())(cinfo, TRUE);
                    ((*(*cinfo).coef).start_pass.unwrap())(cinfo, JBUF_CRANK_DEST);
                    (*master).pub_field.call_pass_startup = FALSE;
                } else {
                    /* Special case: Huffman DC refinement scans need no Huffman table
                     * and therefore we can skip the optimization pass for them.
                     */
                    (*master).pass_type = c_pass_type::output_pass;
                    (*master).pass_number += 1;
                    /* FALLTHROUGH */
                    /* Do a data-output pass. */
                    /* We need not repeat per-scan setup if prior optimization pass did it. */
                    if (*cinfo).optimize_coding == FALSE {
                        select_scan_parameters(cinfo);
                        per_scan_setup(cinfo);
                    }
                    ((*(*cinfo).entropy).start_pass.unwrap())(cinfo, FALSE);
                    ((*(*cinfo).coef).start_pass.unwrap())(cinfo, JBUF_CRANK_DEST);
                    /* We emit frame/scan headers now */
                    if (*master).scan_number == 0 {
                        ((*(*cinfo).marker).write_frame_header.unwrap())(cinfo);
                    }
                    ((*(*cinfo).marker).write_scan_header.unwrap())(cinfo);
                    (*master).pub_field.call_pass_startup = FALSE;
                }
            }
            c_pass_type::output_pass => {
                /* Do a data-output pass. */
                /* We need not repeat per-scan setup if prior optimization pass did it. */
                if (*cinfo).optimize_coding == FALSE {
                    select_scan_parameters(cinfo);
                    per_scan_setup(cinfo);
                }
                ((*(*cinfo).entropy).start_pass.unwrap())(cinfo, FALSE);
                ((*(*cinfo).coef).start_pass.unwrap())(cinfo, JBUF_CRANK_DEST);
                /* We emit frame/scan headers now */
                if (*master).scan_number == 0 {
                    ((*(*cinfo).marker).write_frame_header.unwrap())(cinfo);
                }
                ((*(*cinfo).marker).write_scan_header.unwrap())(cinfo);
                (*master).pub_field.call_pass_startup = FALSE;
            }
        }

        (*master).pub_field.is_last_pass = ((*master).pass_number == (*master).total_passes - 1);

        /* Set up progress monitor's pass info if present */
        if (*cinfo).progress != core::ptr::null_mut() {
            (*(*cinfo).progress).completed_passes = (*master).pass_number;
            (*(*cinfo).progress).total_passes = (*master).total_passes;
        }
    }
}

/*
 * Special start-of-pass hook.
 * This is called by jpeg_write_scanlines if call_pass_startup is TRUE.
 * In single-pass processing, we need this hook because we don't want to
 * write frame/scan headers during jpeg_start_compress; we want to let the
 * application write COM markers etc. between jpeg_start_compress and the
 * jpeg_write_scanlines loop.
 * In multi-pass processing, this routine is not used.
 */

extern "C" fn pass_startup(cinfo: j_compress_ptr) {
    unsafe {
        (*(*cinfo).master).call_pass_startup = FALSE; /* reset flag so call only once */

        ((*(*cinfo).marker).write_frame_header.unwrap())(cinfo);
        ((*(*cinfo).marker).write_scan_header.unwrap())(cinfo);
    }
}

/*
 * Finish up at end of pass.
 */

extern "C" fn finish_pass_master(cinfo: j_compress_ptr) {
    unsafe {
        let master = (*cinfo).master as my_master_ptr;

        /* The entropy coder always needs an end-of-pass call,
         * either to analyze statistics or to flush its output buffer.
         */
        ((*(*cinfo).entropy).finish_pass.unwrap())(cinfo);

        /* Update state for next pass */
        match (*master).pass_type {
            c_pass_type::main_pass => {
                /* next pass is either output of scan 0 (after optimization)
                 * or output of scan 1 (if no optimization).
                 */
                (*master).pass_type = c_pass_type::output_pass;
                if (*cinfo).optimize_coding == FALSE {
                    (*master).scan_number += 1;
                }
            }
            #[cfg(feature = "ENTROPY_OPT_SUPPORTED")]
            c_pass_type::huff_opt_pass => {
                /* next pass is always output of current scan */
                (*master).pass_type = c_pass_type::output_pass;
            }
            c_pass_type::output_pass => {
                /* next pass is either optimization or output of next scan */
                if (*cinfo).optimize_coding != FALSE {
                    #[cfg(feature = "ENTROPY_OPT_SUPPORTED")]
                    {
                        (*master).pass_type = c_pass_type::huff_opt_pass;
                    }
                }
                (*master).scan_number += 1;
            }
        }

        (*master).pass_number += 1;
    }
}

/*
 * Initialize master compression control.
 */

pub extern "C" fn jinit_c_master_control(cinfo: j_compress_ptr, transcode_only: c_int) {
    unsafe {
        let mut master: my_master_ptr;

        master = ((*(*cinfo).mem).alloc_small.unwrap())(
            cinfo as j_common_ptr,
            0, /* JPOOL_IMAGE */
            core::mem::size_of::<my_comp_master>()
        ) as my_master_ptr;
        (*cinfo).master = &mut (*master).pub_field;
        (*(*cinfo).master).prepare_for_pass = Some(prepare_for_pass);
        (*(*cinfo).master).pass_startup = Some(pass_startup);
        (*(*cinfo).master).finish_pass = Some(finish_pass_master);
        (*(*cinfo).master).is_last_pass = FALSE;

        /* Validate parameters, determine derived values */
        initial_setup(cinfo);

        if (*cinfo).scan_info != core::ptr::null() {
            #[cfg(feature = "C_MULTISCAN_FILES_SUPPORTED")]
            {
                validate_script(cinfo);
            }
            #[cfg(not(feature = "C_MULTISCAN_FILES_SUPPORTED"))]
            {
                ERREXIT(cinfo, JERR_NOT_COMPILED);
            }
        } else {
            (*cinfo).progressive_mode = FALSE;
            (*cinfo).num_scans = 1;
        }

        if (*cinfo).progressive_mode != FALSE {  /*  TEMPORARY HACK ??? */
            (*cinfo).optimize_coding = TRUE; /* assume default tables no good for progressive mode */
        }

        /* Initialize my private state */
        if transcode_only != FALSE {
            /* no main pass in transcoding */
            if (*cinfo).optimize_coding != FALSE {
                (*master).pass_type = c_pass_type::huff_opt_pass;
            } else {
                (*master).pass_type = c_pass_type::output_pass;
            }
        } else {
            /* for normal compression, first pass is always this type: */
            (*master).pass_type = c_pass_type::main_pass;
        }
        (*master).scan_number = 0;
        (*master).pass_number = 0;
        if (*cinfo).optimize_coding != FALSE {
            (*master).total_passes = (*cinfo).num_scans * 2;
        } else {
            (*master).total_passes = (*cinfo).num_scans;
        }
    }
}
