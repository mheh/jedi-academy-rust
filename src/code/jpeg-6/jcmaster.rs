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

// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_long, c_uint, c_void};

#[allow(non_snake_case)]

/* Private state */

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum c_pass_type {
    main_pass,      /* input data, also do first output step */
    huff_opt_pass,  /* Huffman code optimization pass */
    output_pass     /* data output pass */
}

#[repr(C)]
struct my_comp_master {
    pub_: jpeg_comp_master,        /* public fields */

    pass_type: c_pass_type,        /* the type of the current pass */

    pass_number: c_int,            /* # of passes completed */
    total_passes: c_int,           /* total # of passes needed */

    scan_number: c_int,            /* current index in scan_info[] */
}

type my_master_ptr = *mut my_comp_master;

/* Stub: these types are defined in the C headers but we model them minimally */

#[repr(C)]
struct jpeg_error_mgr {
    msg_code: c_int,
    msg_parm: c_msg_parm,
    error_exit: Option<extern "C" fn(*mut c_void)>,
}

#[repr(C)]
union c_msg_parm {
    i: [c_int; 8],
}

#[repr(C)]
struct jpeg_memory_mgr {
    alloc_small: Option<extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
}

#[repr(C)]
struct jpeg_comp_master {
    prepare_for_pass: Option<extern "C" fn(*mut jpeg_compress_struct)>,
    pass_startup: Option<extern "C" fn(*mut jpeg_compress_struct)>,
    finish_pass: Option<extern "C" fn(*mut jpeg_compress_struct)>,

    call_pass_startup: c_uint,
    is_last_pass: c_uint,
}

#[repr(C)]
struct jpeg_progress_mgr {
    completed_passes: c_int,
    total_passes: c_int,
}

#[repr(C)]
struct jpeg_compress_struct {
    err: *mut jpeg_error_mgr,
    mem: *mut jpeg_memory_mgr,
    progress: *mut jpeg_progress_mgr,
    is_decompressor: c_uint,
    global_state: c_int,

    dest: *mut c_void,

    image_width: c_uint,
    image_height: c_uint,
    input_components: c_int,
    in_color_space: c_int,

    input_gamma: f64,

    data_precision: c_int,

    num_components: c_int,
    jpeg_color_space: c_int,

    comp_info: *mut jpeg_component_info,
    quant_tbl_ptrs: [*mut c_void; 4],
    dc_huff_tbl_ptrs: [*mut c_void; 4],
    ac_huff_tbl_ptrs: [*mut c_void; 4],

    arith_dc_L: [c_uint; 16],
    arith_dc_U: [c_uint; 16],
    arith_ac_K: [c_uint; 16],

    num_scans: c_int,
    scan_info: *const jpeg_scan_info,

    raw_data_in: c_uint,
    arith_code: c_uint,
    optimize_coding: c_uint,
    CCIR601_sampling: c_uint,
    smoothing_factor: c_int,
    dct_method: c_int,

    restart_interval: c_uint,
    restart_in_rows: c_int,

    write_JFIF_header: c_uint,
    density_unit: c_uint,
    X_density: c_uint,
    Y_density: c_uint,
    write_Adobe_marker: c_uint,

    next_scanline: c_uint,

    progressive_mode: c_uint,
    max_h_samp_factor: c_int,
    max_v_samp_factor: c_int,

    total_iMCU_rows: c_uint,

    comps_in_scan: c_int,
    cur_comp_info: [*mut jpeg_component_info; 4],

    MCUs_per_row: c_uint,
    MCU_rows_in_scan: c_uint,

    blocks_in_MCU: c_int,
    MCU_membership: [c_int; 10],

    Ss: c_int,
    Se: c_int,
    Ah: c_int,
    Al: c_int,

    master: *mut jpeg_comp_master,
    main: *mut c_void,
    prep: *mut jpeg_c_prep_controller,
    coef: *mut jpeg_c_coef_controller,
    marker: *mut jpeg_marker_writer,
    cconvert: *mut jpeg_color_converter,
    downsample: *mut jpeg_downsampler,
    fdct: *mut jpeg_forward_dct,
    entropy: *mut jpeg_entropy_encoder,
}

#[repr(C)]
struct jpeg_component_info {
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

    component_needed: c_uint,

    MCU_width: c_int,
    MCU_height: c_int,
    MCU_blocks: c_int,
    MCU_sample_width: c_int,
    last_col_width: c_int,
    last_row_height: c_int,

    quant_table: *mut c_void,

    dct_table: *mut c_void,
}

#[repr(C)]
struct jpeg_scan_info {
    comps_in_scan: c_int,
    component_index: [c_int; 4],
    Ss: c_int,
    Se: c_int,
    Ah: c_int,
    Al: c_int,
}

/* Forward declarations of module types */
#[repr(C)]
struct jpeg_c_prep_controller {
    start_pass: Option<extern "C" fn(*mut jpeg_compress_struct, c_int)>,
}

#[repr(C)]
struct jpeg_c_coef_controller {
    start_pass: Option<extern "C" fn(*mut jpeg_compress_struct, c_int)>,
}

#[repr(C)]
struct jpeg_marker_writer {
    write_frame_header: Option<extern "C" fn(*mut jpeg_compress_struct)>,
    write_scan_header: Option<extern "C" fn(*mut jpeg_compress_struct)>,
}

#[repr(C)]
struct jpeg_color_converter {
    start_pass: Option<extern "C" fn(*mut jpeg_compress_struct)>,
}

#[repr(C)]
struct jpeg_downsampler {
    start_pass: Option<extern "C" fn(*mut jpeg_compress_struct)>,
}

#[repr(C)]
struct jpeg_forward_dct {
    start_pass: Option<extern "C" fn(*mut jpeg_compress_struct)>,
}

#[repr(C)]
struct jpeg_entropy_encoder {
    start_pass: Option<extern "C" fn(*mut jpeg_compress_struct, c_uint)>,
    finish_pass: Option<extern "C" fn(*mut jpeg_compress_struct)>,
}

#[repr(C)]
struct jpeg_c_main_controller {
    start_pass: Option<extern "C" fn(*mut jpeg_compress_struct, c_int)>,
}

/* Constants */
const DCTSIZE: c_int = 8;
const DCTSIZE2: c_int = 64;
const MAX_COMPONENTS: c_int = 10;
const MAX_COMPS_IN_SCAN: c_int = 4;
const MAX_SAMP_FACTOR: c_int = 4;
const C_MAX_BLOCKS_IN_MCU: c_int = 10;
const BITS_IN_JSAMPLE: c_int = 8;
const JPEG_MAX_DIMENSION: c_uint = 65500;

const TRUE: c_uint = 1;
const FALSE: c_uint = 0;

/* Buffer modes */
const JBUF_PASS_THRU: c_int = 0;
const JBUF_SAVE_AND_PASS: c_int = 3;
const JBUF_CRANK_DEST: c_int = 2;

/* Error codes (stubs) */
const JERR_EMPTY_IMAGE: c_int = 1;
const JERR_IMAGE_TOO_BIG: c_int = 2;
const JERR_WIDTH_OVERFLOW: c_int = 3;
const JERR_BAD_PRECISION: c_int = 4;
const JERR_COMPONENT_COUNT: c_int = 5;
const JERR_BAD_SAMPLING: c_int = 6;
const JERR_BAD_SCAN_SCRIPT: c_int = 7;
const JERR_BAD_PROG_SCRIPT: c_int = 8;
const JERR_NOT_COMPILED: c_int = 9;
const JERR_MISSING_DATA: c_int = 10;
const JERR_BAD_MCU_SIZE: c_int = 11;

const JPOOL_IMAGE: c_int = 1;

/*
 * Support routines that do various essential calculations.
 */

/* Forward declarations */
extern "C" {
    fn jdiv_round_up(a: c_long, b: c_long) -> c_long;
}

fn ERREXIT(cinfo: *mut jpeg_compress_struct, code: c_int) {
    unsafe {
        if !cinfo.is_null() && !(*cinfo).err.is_null() {
            (*(*cinfo).err).msg_code = code;
            if let Some(error_exit) = (*(*cinfo).err).error_exit {
                error_exit(cinfo as *mut c_void);
            }
        }
    }
}

fn ERREXIT1(cinfo: *mut jpeg_compress_struct, code: c_int, p1: c_int) {
    unsafe {
        if !cinfo.is_null() && !(*cinfo).err.is_null() {
            (*(*cinfo).err).msg_code = code;
            (*(*cinfo).err).msg_parm.i[0] = p1;
            if let Some(error_exit) = (*(*cinfo).err).error_exit {
                error_exit(cinfo as *mut c_void);
            }
        }
    }
}

fn ERREXIT2(cinfo: *mut jpeg_compress_struct, code: c_int, p1: c_int, p2: c_int) {
    unsafe {
        if !cinfo.is_null() && !(*cinfo).err.is_null() {
            (*(*cinfo).err).msg_code = code;
            (*(*cinfo).err).msg_parm.i[0] = p1;
            (*(*cinfo).err).msg_parm.i[1] = p2;
            if let Some(error_exit) = (*(*cinfo).err).error_exit {
                error_exit(cinfo as *mut c_void);
            }
        }
    }
}

fn MAX(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
}

fn MIN(nominal: c_long, limit: c_long) -> c_uint {
    if nominal < limit { nominal as c_uint } else { limit as c_uint }
}

fn initial_setup(cinfo: *mut jpeg_compress_struct)
/* Do computations that are needed before master selection phase */
{
  unsafe {
      let mut ci: c_int;
      let mut compptr: *mut jpeg_component_info;
      let mut samplesperrow: c_long;
      let mut jd_samplesperrow: c_uint;

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
      jd_samplesperrow = samplesperrow as c_uint;
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
        (*compptr).width_in_blocks = jdiv_round_up(((*cinfo).image_width as c_long) * ((*compptr).h_samp_factor as c_long),
                                                     (((*cinfo).max_h_samp_factor as c_long) * (DCTSIZE as c_long))) as c_uint;
        (*compptr).height_in_blocks = jdiv_round_up(((*cinfo).image_height as c_long) * ((*compptr).v_samp_factor as c_long),
                                                      (((*cinfo).max_v_samp_factor as c_long) * (DCTSIZE as c_long))) as c_uint;
        /* Size in samples */
        (*compptr).downsampled_width = jdiv_round_up(((*cinfo).image_width as c_long) * ((*compptr).h_samp_factor as c_long),
                                                       ((*cinfo).max_h_samp_factor as c_long)) as c_uint;
        (*compptr).downsampled_height = jdiv_round_up(((*cinfo).image_height as c_long) * ((*compptr).v_samp_factor as c_long),
                                                        ((*cinfo).max_v_samp_factor as c_long)) as c_uint;
        /* Mark component needed (this flag isn't actually used for compression) */
        (*compptr).component_needed = TRUE;
        ci += 1;
        compptr = compptr.offset(1);
      }

      /* Compute number of fully interleaved MCU rows (number of times that
       * main controller will call coefficient controller).
       */
      (*cinfo).total_iMCU_rows = jdiv_round_up((*cinfo).image_height as c_long,
                                                (((*cinfo).max_v_samp_factor as c_long) * (DCTSIZE as c_long))) as c_uint;
  }
}

fn select_scan_parameters(cinfo: *mut jpeg_compress_struct)
/* Set up the scan parameters for the current scan */
{
  unsafe {
      let mut ci: c_int;

      #[cfg(feature = "C_MULTISCAN_FILES_SUPPORTED")]
      {
        if !(*cinfo).scan_info.is_null() {
          /* Prepare for current scan --- the script is already validated */
          let master: my_master_ptr = (*cinfo).master as *mut my_comp_master;
          let scanptr: *const jpeg_scan_info = (*cinfo).scan_info.offset((*master).scan_number as isize);

          (*cinfo).comps_in_scan = (*scanptr).comps_in_scan;
          ci = 0;
          while ci < (*scanptr).comps_in_scan {
            (*cinfo).cur_comp_info[ci as usize] =
              (*cinfo).comp_info.offset((*scanptr).component_index[ci as usize] as isize);
            ci += 1;
          }
          (*cinfo).Ss = (*scanptr).Ss;
          (*cinfo).Se = (*scanptr).Se;
          (*cinfo).Ah = (*scanptr).Ah;
          (*cinfo).Al = (*scanptr).Al;
          return;
        }
      }

      /* Prepare for single sequential-JPEG scan containing all components */
      if (*cinfo).num_components > MAX_COMPS_IN_SCAN {
        ERREXIT2(cinfo, JERR_COMPONENT_COUNT, (*cinfo).num_components,
                 MAX_COMPS_IN_SCAN);
      }
      (*cinfo).comps_in_scan = (*cinfo).num_components;
      ci = 0;
      while ci < (*cinfo).num_components {
        (*cinfo).cur_comp_info[ci as usize] =
          (*cinfo).comp_info.offset(ci as isize);
        ci += 1;
      }
      (*cinfo).Ss = 0;
      (*cinfo).Se = DCTSIZE2-1;
      (*cinfo).Ah = 0;
      (*cinfo).Al = 0;
  }
}

fn per_scan_setup(cinfo: *mut jpeg_compress_struct)
/* Do computations that are needed before processing a JPEG scan */
/* cinfo->comps_in_scan and cinfo->cur_comp_info[] are already set */
{
  unsafe {
      let mut ci: c_int;
      let mut mcublks: c_int;
      let mut tmp: c_int;
      let mut compptr: *mut jpeg_component_info;

      if (*cinfo).comps_in_scan == 1 {

        /* Noninterleaved (single-component) scan */
        compptr = (*cinfo).cur_comp_info[0];

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
        (*cinfo).MCUs_per_row = jdiv_round_up((*cinfo).image_width as c_long,
                                               (((*cinfo).max_h_samp_factor as c_long) * (DCTSIZE as c_long))) as c_uint;
        (*cinfo).MCU_rows_in_scan = jdiv_round_up((*cinfo).image_height as c_long,
                                                   (((*cinfo).max_v_samp_factor as c_long) * (DCTSIZE as c_long))) as c_uint;

        (*cinfo).blocks_in_MCU = 0;

        ci = 0;
        while ci < (*cinfo).comps_in_scan {
          compptr = (*cinfo).cur_comp_info[ci as usize];
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
            mcublks -= 1;
            (*cinfo).MCU_membership[(*cinfo).blocks_in_MCU as usize] = ci;
            (*cinfo).blocks_in_MCU += 1;
          }
          ci += 1;
        }

      }

      /* Convert restart specified in rows to actual MCU count. */
      /* Note that count must fit in 16 bits, so we provide limiting. */
      if (*cinfo).restart_in_rows > 0 {
        let nominal: c_long = ((*cinfo).restart_in_rows as c_long) * ((*cinfo).MCUs_per_row as c_long);
        (*cinfo).restart_interval = MIN(nominal, 65535);
      }
  }
}

/*
 * Per-pass setup.
 * This is called at the beginning of each pass.  We determine which modules
 * will be active during this pass and give them appropriate start_pass calls.
 * We also set is_last_pass to indicate whether any more passes will be
 * required.
 */

fn prepare_for_pass(cinfo: *mut jpeg_compress_struct)
{
  unsafe {
      let master: my_master_ptr = (*cinfo).master as *mut my_comp_master;

      match (*master).pass_type {
          c_pass_type::main_pass => {
            /* Initial pass: will collect input data, and do either Huffman
             * optimization or data output for the first scan.
             */
            select_scan_parameters(cinfo);
            per_scan_setup(cinfo);
            if (*cinfo).raw_data_in == 0 {
              if let Some(f) = (*(*cinfo).cconvert).start_pass {
                  f(cinfo);
              }
              if let Some(f) = (*(*cinfo).downsample).start_pass {
                  f(cinfo);
              }
              if let Some(f) = (*(*cinfo).prep).start_pass {
                  f(cinfo, JBUF_PASS_THRU);
              }
            }
            if let Some(f) = (*(*cinfo).fdct).start_pass {
                f(cinfo);
            }
            if let Some(f) = (*(*cinfo).entropy).start_pass {
                f(cinfo, (*cinfo).optimize_coding);
            }
            if let Some(f) = (*(*cinfo).coef).start_pass {
                f(cinfo, if (*master).total_passes > 1 { JBUF_SAVE_AND_PASS } else { JBUF_PASS_THRU });
            }
            if let Some(f) = (*(*cinfo).main).start_pass {
                (f as extern "C" fn(*mut jpeg_compress_struct, c_int))(cinfo, JBUF_PASS_THRU);
            }
            if (*cinfo).optimize_coding != 0 {
              /* No immediate data output; postpone writing frame/scan headers */
              (*(*cinfo).master).call_pass_startup = FALSE;
            } else {
              /* Will write frame/scan headers at first jpeg_write_scanlines call */
              (*(*cinfo).master).call_pass_startup = TRUE;
            }
          }
          c_pass_type::huff_opt_pass => {
            /* Do Huffman optimization for a scan after the first one. */
            select_scan_parameters(cinfo);
            per_scan_setup(cinfo);
            if (*cinfo).Ss != 0 || (*cinfo).Ah == 0 || (*cinfo).arith_code != 0 {
              if let Some(f) = (*(*cinfo).entropy).start_pass {
                  f(cinfo, TRUE);
              }
              if let Some(f) = (*(*cinfo).coef).start_pass {
                  f(cinfo, JBUF_CRANK_DEST);
              }
              (*(*cinfo).master).call_pass_startup = FALSE;
            } else {
              /* Special case: Huffman DC refinement scans need no Huffman table
               * and therefore we can skip the optimization pass for them.
               */
              (*master).pass_type = c_pass_type::output_pass;
              (*master).pass_number += 1;
              /*FALLTHROUGH*/
              select_scan_parameters(cinfo);
              per_scan_setup(cinfo);
              if let Some(f) = (*(*cinfo).entropy).start_pass {
                  f(cinfo, FALSE);
              }
              if let Some(f) = (*(*cinfo).coef).start_pass {
                  f(cinfo, JBUF_CRANK_DEST);
              }
              /* We emit frame/scan headers now */
              if (*master).scan_number == 0 {
                if let Some(f) = (*(*cinfo).marker).write_frame_header {
                    f(cinfo);
                }
              }
              if let Some(f) = (*(*cinfo).marker).write_scan_header {
                  f(cinfo);
              }
              (*(*cinfo).master).call_pass_startup = FALSE;
            }
          }
          c_pass_type::output_pass => {
            /* Do a data-output pass. */
            /* We need not repeat per-scan setup if prior optimization pass did it. */
            if (*cinfo).optimize_coding == 0 {
              select_scan_parameters(cinfo);
              per_scan_setup(cinfo);
            }
            if let Some(f) = (*(*cinfo).entropy).start_pass {
                f(cinfo, FALSE);
            }
            if let Some(f) = (*(*cinfo).coef).start_pass {
                f(cinfo, JBUF_CRANK_DEST);
            }
            /* We emit frame/scan headers now */
            if (*master).scan_number == 0 {
              if let Some(f) = (*(*cinfo).marker).write_frame_header {
                  f(cinfo);
              }
            }
            if let Some(f) = (*(*cinfo).marker).write_scan_header {
                f(cinfo);
            }
            (*(*cinfo).master).call_pass_startup = FALSE;
          }
      }

      (*(*cinfo).master).is_last_pass = if (*master).pass_number == (*master).total_passes-1 { TRUE } else { FALSE };

      /* Set up progress monitor's pass info if present */
      if !(*cinfo).progress.is_null() {
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

fn pass_startup(cinfo: *mut jpeg_compress_struct)
{
  unsafe {
      (*(*cinfo).master).call_pass_startup = FALSE; /* reset flag so call only once */

      if let Some(f) = (*(*cinfo).marker).write_frame_header {
          f(cinfo);
      }
      if let Some(f) = (*(*cinfo).marker).write_scan_header {
          f(cinfo);
      }
  }
}

/*
 * Finish up at end of pass.
 */

fn finish_pass_master(cinfo: *mut jpeg_compress_struct)
{
  unsafe {
      let master: my_master_ptr = (*cinfo).master as *mut my_comp_master;

      /* The entropy coder always needs an end-of-pass call,
       * either to analyze statistics or to flush its output buffer.
       */
      if let Some(f) = (*(*cinfo).entropy).finish_pass {
          f(cinfo);
      }

      /* Update state for next pass */
      match (*master).pass_type {
          c_pass_type::main_pass => {
            /* next pass is either output of scan 0 (after optimization)
             * or output of scan 1 (if no optimization).
             */
            (*master).pass_type = c_pass_type::output_pass;
            if (*cinfo).optimize_coding == 0 {
              (*master).scan_number += 1;
            }
          }
          c_pass_type::huff_opt_pass => {
            /* next pass is always output of current scan */
            (*master).pass_type = c_pass_type::output_pass;
          }
          c_pass_type::output_pass => {
            /* next pass is either optimization or output of next scan */
            if (*cinfo).optimize_coding != 0 {
              (*master).pass_type = c_pass_type::huff_opt_pass;
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

pub extern "C" fn jinit_c_master_control(cinfo: *mut jpeg_compress_struct, transcode_only: c_uint)
{
  unsafe {
      let master: my_master_ptr;

      master = (*(*cinfo).mem).alloc_small.unwrap()(
          cinfo as *mut c_void,
          JPOOL_IMAGE,
          std::mem::size_of::<my_comp_master>()
      ) as my_master_ptr;

      (*cinfo).master = &mut (*master).pub_ as *mut jpeg_comp_master;
      (*(*cinfo).master).prepare_for_pass = Some(prepare_for_pass);
      (*(*cinfo).master).pass_startup = Some(pass_startup);
      (*(*cinfo).master).finish_pass = Some(finish_pass_master);
      (*(*cinfo).master).is_last_pass = FALSE;

      /* Validate parameters, determine derived values */
      initial_setup(cinfo);

      if !(*cinfo).scan_info.is_null() {
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

      if (*cinfo).progressive_mode != 0 {  /*  TEMPORARY HACK ??? */
        (*cinfo).optimize_coding = TRUE; /* assume default tables no good for progressive mode */
      }

      /* Initialize my private state */
      if transcode_only != 0 {
        /* no main pass in transcoding */
        if (*cinfo).optimize_coding != 0 {
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
      if (*cinfo).optimize_coding != 0 {
        (*master).total_passes = (*cinfo).num_scans * 2;
      } else {
        (*master).total_passes = (*cinfo).num_scans;
      }
  }
}

#[cfg(feature = "C_MULTISCAN_FILES_SUPPORTED")]
fn validate_script(cinfo: *mut jpeg_compress_struct)
/* Verify that the scan script in cinfo->scan_info[] is valid; also
 * determine whether it uses progressive JPEG, and set cinfo->progressive_mode.
 */
{
  unsafe {
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
      let mut component_sent: [c_uint; 10];

      #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
      let mut last_bitpos: [[c_int; 64]; 10];

      if (*cinfo).num_scans <= 0 {
        ERREXIT1(cinfo, JERR_BAD_SCAN_SCRIPT, 0);
      }

      /* For sequential JPEG, all scans must have Ss=0, Se=DCTSIZE2-1;
       * for progressive JPEG, no scan can have this.
       */
      scanptr = (*cinfo).scan_info;
      if (*scanptr).Ss != 0 || (*scanptr).Se != DCTSIZE2-1 {
        #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
        {
          (*cinfo).progressive_mode = TRUE;
          for i in 0..10usize {
              for j in 0..64usize {
                  last_bitpos[i][j] = -1;
              }
          }
          ci = 0;
          while ci < (*cinfo).num_components {
            coefi = 0;
            while coefi < DCTSIZE2 {
              last_bitpos[ci as usize][coefi as usize] = -1;
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
          if ci > 0 && thisi <= (*scanptr).component_index[(ci-1) as usize] {
            ERREXIT1(cinfo, JERR_BAD_SCAN_SCRIPT, scanno);
          }
          ci += 1;
        }
        /* Validate progression parameters */
        Ss = (*scanptr).Ss;
        Se = (*scanptr).Se;
        Ah = (*scanptr).Ah;
        Al = (*scanptr).Al;
        if (*cinfo).progressive_mode != 0 {
          #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
          {
            if Ss < 0 || Ss >= DCTSIZE2 || Se < Ss || Se >= DCTSIZE2 ||
                Ah < 0 || Ah > 13 || Al < 0 || Al > 13 {
              ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
            }
            if Ss == 0 {
              if Se != 0 {       /* DC and AC together not OK */
                ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
              }
            } else {
              if ncomps != 1 {   /* AC scans must be for only one component */
                ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
              }
            }
            ci = 0;
            while ci < ncomps {
              if Ss != 0 && last_bitpos[(*scanptr).component_index[ci as usize] as usize][0] < 0 {
                /* AC without prior DC scan */
                ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
              }
              coefi = Ss;
              while coefi <= Se {
                if last_bitpos[(*scanptr).component_index[ci as usize] as usize][coefi as usize] < 0 {
                  /* first scan of this coefficient */
                  if Ah != 0 {
                    ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
                  }
                } else {
                  /* not first scan */
                  if Ah != last_bitpos[(*scanptr).component_index[ci as usize] as usize][coefi as usize] || Al != Ah-1 {
                    ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
                  }
                }
                last_bitpos[(*scanptr).component_index[ci as usize] as usize][coefi as usize] = Al;
                coefi += 1;
              }
              ci += 1;
            }
          }
        } else {
          /* For sequential JPEG, all progression parameters must be these: */
          if Ss != 0 || Se != DCTSIZE2-1 || Ah != 0 || Al != 0 {
            ERREXIT1(cinfo, JERR_BAD_PROG_SCRIPT, scanno);
          }
          /* Make sure components are not sent twice */
          ci = 0;
          while ci < ncomps {
            thisi = (*scanptr).component_index[ci as usize];
            if component_sent[thisi as usize] != 0 {
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
      if (*cinfo).progressive_mode != 0 {
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
          if component_sent[ci as usize] == 0 {
            ERREXIT(cinfo, JERR_MISSING_DATA);
          }
          ci += 1;
        }
      }
  }
}

#[cfg(not(feature = "C_MULTISCAN_FILES_SUPPORTED"))]
fn validate_script(_cinfo: *mut jpeg_compress_struct) {
    /* Stub for when multiscan is not supported */
}
