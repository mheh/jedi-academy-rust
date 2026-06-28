/*
 * jccoefct.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains the coefficient buffer controller for compression.
 * This controller is the top level of the JPEG compressor proper.
 * The coefficient buffer lies between forward-DCT and entropy encoding steps.
 */

use core::ffi::{c_int, c_void};
use core::mem::size_of;
use core::ptr;

// ============================================================================
// Stubs for JPEG-6 types and structures needed for structural coherence
// ============================================================================

pub type JDIMENSION = u32;
pub type JOCTET = u8;
pub type JSAMPROW = *mut u8;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type JCOEF = i16;
pub type JBLOCK = [JCOEF; 64];
pub type JBLOCKROW = *mut JBLOCK;
pub type JBLOCKARRAY = *mut JBLOCKROW;
pub type boolean = u8;
pub type J_BUF_MODE = c_int;
pub type jvirt_barray_ptr = *mut c_void;

// Constants
const DCTSIZE: c_int = 8;
const DCTSIZE2: usize = 64;
const C_MAX_BLOCKS_IN_MCU: usize = 10;
const MAX_COMPONENTS: usize = 10;
const MAX_COMPS_IN_SCAN: usize = 4;

const FALSE: boolean = 0;
const TRUE: boolean = 1;

const JBUF_PASS_THRU: J_BUF_MODE = 0;
const JBUF_SAVE_SOURCE: J_BUF_MODE = 1;
const JBUF_CRANK_DEST: J_BUF_MODE = 3;
const JBUF_SAVE_AND_PASS: J_BUF_MODE = 3;

const JPOOL_IMAGE: c_int = 1;

const JERR_BAD_BUFFER_MODE: c_int = 1;

// Forward declarations of structures
#[repr(C)]
pub struct jpeg_error_mgr {
    pub reset_error_mgr: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
    pub alloc_large: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
    pub access_virt_barray: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, JDIMENSION, JDIMENSION, boolean) -> *mut c_void>,
    pub request_virt_barray: Option<unsafe extern "C" fn(*mut c_void, c_int, boolean, JDIMENSION, JDIMENSION, JDIMENSION) -> *mut c_void>,
}

#[repr(C)]
pub struct jpeg_c_coef_controller {
    pub start_pass: Option<unsafe extern "C" fn(*mut c_void, J_BUF_MODE)>,
    pub compress_data: Option<unsafe extern "C" fn(*mut c_void, JSAMPIMAGE) -> boolean>,
}

#[repr(C)]
pub struct jpeg_component_info {
    pub component_id: c_int,
    pub component_index: c_int,
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
    pub quant_tbl_no: c_int,
    pub dc_tbl_no: c_int,
    pub ac_tbl_no: c_int,
    pub width_in_blocks: JDIMENSION,
    pub height_in_blocks: JDIMENSION,
    pub DCT_scaled_size: c_int,
    pub downsampled_width: JDIMENSION,
    pub downsampled_height: JDIMENSION,
    pub component_needed: boolean,
    pub MCU_width: c_int,
    pub MCU_height: c_int,
    pub MCU_blocks: c_int,
    pub MCU_sample_width: c_int,
    pub last_col_width: c_int,
    pub last_row_height: c_int,
    pub quant_table: *mut c_void,
    pub dct_table: *mut c_void,
}

#[repr(C)]
pub struct jpeg_forward_dct {
    pub start_pass: Option<unsafe extern "C" fn(*mut c_void)>,
    pub forward_DCT: Option<unsafe extern "C" fn(*mut c_void, *mut jpeg_component_info, JSAMPARRAY, JBLOCKROW, JDIMENSION, JDIMENSION, JDIMENSION)>,
}

#[repr(C)]
pub struct jpeg_entropy_encoder {
    pub start_pass: Option<unsafe extern "C" fn(*mut c_void, boolean)>,
    pub encode_mcu: Option<unsafe extern "C" fn(*mut c_void, *mut JBLOCKROW) -> boolean>,
    pub finish_pass: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct j_compress_struct {
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub progress: *mut c_void,
    pub is_decompressor: boolean,
    pub global_state: c_int,
    pub dest: *mut c_void,
    pub comp_info: *mut jpeg_component_info,
    pub image_width: JDIMENSION,
    pub image_height: JDIMENSION,
    pub input_components: c_int,
    pub in_color_space: c_int,
    pub input_gamma: f64,
    pub data_precision: c_int,
    pub num_components: c_int,
    pub jpeg_color_space: c_int,
    pub quant_tbl_ptrs: [*mut c_void; 4],
    pub dc_huff_tbl_ptrs: [*mut c_void; 4],
    pub ac_huff_tbl_ptrs: [*mut c_void; 4],
    pub arith_dc_L: [u8; 16],
    pub arith_dc_U: [u8; 16],
    pub arith_ac_K: [u8; 16],
    pub num_scans: c_int,
    pub scan_info: *mut c_void,
    pub raw_data_in: boolean,
    pub arith_code: boolean,
    pub optimize_coding: boolean,
    pub CCIR601_sampling: boolean,
    pub smoothing_factor: c_int,
    pub dct_method: c_int,
    pub restart_interval: c_int,
    pub restart_in_rows: c_int,
    pub write_JFIF_header: boolean,
    pub density_unit: u8,
    pub X_density: u16,
    pub Y_density: u16,
    pub next_scanline: JDIMENSION,
    pub progressive_mode: boolean,
    pub max_h_samp_factor: c_int,
    pub max_v_samp_factor: c_int,
    pub total_iMCU_rows: JDIMENSION,
    pub comps_in_scan: c_int,
    pub cur_comp_info: [*mut jpeg_component_info; MAX_COMPS_IN_SCAN],
    pub MCUs_per_row: JDIMENSION,
    pub MCU_rows_in_scan: JDIMENSION,
    pub blocks_in_MCU: c_int,
    pub max_blocks_in_MCU: c_int,
    pub natural_order: [u8; DCTSIZE2],
    pub natural_order_start: c_int,
    pub lossless_simple_prediction: c_int,
    pub Al: c_int,
    pub Ah: c_int,
    pub Ss: c_int,
    pub Se: c_int,
    pub master: *mut c_void,
    pub main: *mut c_void,
    pub prep: *mut c_void,
    pub coef: *mut jpeg_c_coef_controller,
    pub marker: *mut c_void,
    pub fdct: *mut jpeg_forward_dct,
    pub entropy: *mut jpeg_entropy_encoder,
    pub script_space: *mut c_void,
    pub script_space_size: c_int,
}

pub type j_compress_ptr = *mut j_compress_struct;
pub type j_common_ptr = *mut j_compress_struct;

/* Private buffer controller object */

#[repr(C)]
struct my_coef_controller {
    pub r#pub: jpeg_c_coef_controller, /* public fields */
    iMCU_row_num: JDIMENSION,          /* iMCU row # within image */
    mcu_ctr: JDIMENSION,               /* counts MCUs processed in current row */
    MCU_vert_offset: c_int,            /* counts MCU rows within iMCU row */
    MCU_rows_per_iMCU_row: c_int,      /* number of such rows needed */
    /* For single-pass compression, it's sufficient to buffer just one MCU
     * (although this may prove a bit slow in practice).  We allocate a
     * workspace of C_MAX_BLOCKS_IN_MCU coefficient blocks, and reuse it for each
     * MCU constructed and sent.  (On 80x86, the workspace is FAR even though
     * it's not really very big; this is to keep the module interfaces unchanged
     * when a large coefficient buffer is necessary.)
     * In multi-pass modes, this array points to the current MCU's blocks
     * within the virtual arrays.
     */
    MCU_buffer: [JBLOCKROW; C_MAX_BLOCKS_IN_MCU],
    /* In multi-pass modes, we need a virtual block array for each component. */
    whole_image: [jvirt_barray_ptr; MAX_COMPONENTS],
}

type my_coef_ptr = *mut my_coef_controller;

/* Reset within-iMCU-row counters for a new row */
unsafe fn start_iMCU_row(cinfo: j_compress_ptr) {
    let coef = (*cinfo).coef as my_coef_ptr;

    /* In an interleaved scan, an MCU row is the same as an iMCU row.
     * In a noninterleaved scan, an iMCU row has v_samp_factor MCU rows.
     * But at the bottom of the image, process only what's left.
     */
    if (*cinfo).comps_in_scan > 1 {
        (*coef).MCU_rows_per_iMCU_row = 1;
    } else {
        if (*coef).iMCU_row_num < ((*cinfo).total_iMCU_rows - 1) {
            (*coef).MCU_rows_per_iMCU_row = (*(*(*cinfo).cur_comp_info[0])).v_samp_factor;
        } else {
            (*coef).MCU_rows_per_iMCU_row = (*(*(*cinfo).cur_comp_info[0])).last_row_height;
        }
    }

    (*coef).mcu_ctr = 0;
    (*coef).MCU_vert_offset = 0;
}

/*
 * Initialize for a processing pass.
 */

unsafe fn start_pass_coef(cinfo: j_compress_ptr, pass_mode: J_BUF_MODE) {
    let coef = (*cinfo).coef as my_coef_ptr;

    (*coef).iMCU_row_num = 0;
    start_iMCU_row(cinfo);

    match pass_mode {
        JBUF_PASS_THRU => {
            if !(*coef).whole_image[0].is_null() {
                // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
            (*coef).r#pub.compress_data = Some(compress_data as unsafe extern "C" fn(*mut c_void, JSAMPIMAGE) -> boolean);
        }
        #[cfg(any(feature = "entropy_opt_supported", feature = "c_multiscan_files_supported"))]
        JBUF_SAVE_AND_PASS => {
            if (*coef).whole_image[0].is_null() {
                // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
            (*coef).r#pub.compress_data = Some(compress_first_pass as unsafe extern "C" fn(*mut c_void, JSAMPIMAGE) -> boolean);
        }
        #[cfg(any(feature = "entropy_opt_supported", feature = "c_multiscan_files_supported"))]
        JBUF_CRANK_DEST => {
            if (*coef).whole_image[0].is_null() {
                // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
            (*coef).r#pub.compress_data = Some(compress_output as unsafe extern "C" fn(*mut c_void, JSAMPIMAGE) -> boolean);
        }
        _ => {
            // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
        }
    }
}

/*
 * Process some data in the single-pass case.
 * We process the equivalent of one fully interleaved MCU row ("iMCU" row)
 * per call, ie, v_samp_factor block rows for each component in the image.
 * Returns TRUE if the iMCU row is completed, FALSE if suspended.
 *
 * NB: input_buf contains a plane for each component in image.
 * For single pass, this is the same as the components in the scan.
 */

unsafe fn compress_data(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean {
    let coef = (*cinfo).coef as my_coef_ptr;
    let mut MCU_col_num: JDIMENSION;    /* index of current MCU within row */
    let last_MCU_col: JDIMENSION = (*cinfo).MCUs_per_row - 1;
    let last_iMCU_row: JDIMENSION = (*cinfo).total_iMCU_rows - 1;
    let mut blkn: c_int;
    let mut bi: c_int;
    let mut ci: c_int;
    let mut yindex: c_int;
    let mut yoffset: c_int;
    let mut blockcnt: c_int;
    let mut ypos: JDIMENSION;
    let mut xpos: JDIMENSION;
    let mut compptr: *mut jpeg_component_info;

    /* Loop to write as much as one whole iMCU row */
    yoffset = (*coef).MCU_vert_offset;
    while yoffset < (*coef).MCU_rows_per_iMCU_row {
        MCU_col_num = (*coef).mcu_ctr;
        while MCU_col_num <= last_MCU_col {
            /* Determine where data comes from in input_buf and do the DCT thing.
             * Each call on forward_DCT processes a horizontal row of DCT blocks
             * as wide as an MCU; we rely on having allocated the MCU_buffer[] blocks
             * sequentially.  Dummy blocks at the right or bottom edge are filled in
             * specially.  The data in them does not matter for image reconstruction,
             * so we fill them with values that will encode to the smallest amount of
             * data, viz: all zeroes in the AC entries, DC entries equal to previous
             * block's DC value.  (Thanks to Thomas Kinsman for this idea.)
             */
            blkn = 0;
            ci = 0;
            while ci < (*cinfo).comps_in_scan {
                compptr = (*cinfo).cur_comp_info[ci as usize];
                blockcnt = if MCU_col_num < last_MCU_col {
                    (*compptr).MCU_width
                } else {
                    (*compptr).last_col_width
                };
                xpos = MCU_col_num * (*compptr).MCU_sample_width as u32;
                ypos = yoffset as u32 * DCTSIZE as u32; /* ypos == (yoffset+yindex) * DCTSIZE */
                yindex = 0;
                while yindex < (*compptr).MCU_height {
                    if (*coef).iMCU_row_num < last_iMCU_row ||
                        yoffset + yindex < (*compptr).last_row_height {
                        if let Some(forward_DCT_fn) = (*(*(*cinfo).fdct)).forward_DCT {
                            forward_DCT_fn(
                                cinfo as *mut c_void,
                                compptr,
                                *input_buf.add(ci as usize),
                                (*coef).MCU_buffer[blkn as usize],
                                ypos,
                                xpos,
                                blockcnt as u32,
                            );
                        }
                        if blockcnt < (*compptr).MCU_width {
                            /* Create some dummy blocks at the right edge of the image. */
                            ptr::write_bytes(
                                (*coef).MCU_buffer[(blkn + blockcnt) as usize] as *mut u8,
                                0,
                                (((*compptr).MCU_width - blockcnt) as usize) * size_of::<JBLOCK>(),
                            );
                            bi = blockcnt;
                            while bi < (*compptr).MCU_width {
                                (*(*coef).MCU_buffer[((blkn + bi) as usize)])[0][0] =
                                    (*(*coef).MCU_buffer[((blkn + bi - 1) as usize)])[0][0];
                                bi += 1;
                            }
                        }
                    } else {
                        /* Create a row of dummy blocks at the bottom of the image. */
                        ptr::write_bytes(
                            (*coef).MCU_buffer[blkn as usize] as *mut u8,
                            0,
                            ((*compptr).MCU_width as usize) * size_of::<JBLOCK>(),
                        );
                        bi = 0;
                        while bi < (*compptr).MCU_width {
                            (*(*coef).MCU_buffer[((blkn + bi) as usize)])[0][0] =
                                (*(*coef).MCU_buffer[((blkn - 1) as usize)])[0][0];
                            bi += 1;
                        }
                    }
                    blkn += (*compptr).MCU_width;
                    ypos = ypos + DCTSIZE as u32;
                    yindex += 1;
                }
                ci += 1;
            }
            /* Try to write the MCU.  In event of a suspension failure, we will
             * re-DCT the MCU on restart (a bit inefficient, could be fixed...)
             */
            if let Some(encode_mcu_fn) = (*(*(*cinfo).entropy)).encode_mcu {
                if encode_mcu_fn(cinfo as *mut c_void, (*coef).MCU_buffer.as_mut_ptr()) == FALSE {
                    /* Suspension forced; update state counters and exit */
                    (*coef).MCU_vert_offset = yoffset;
                    (*coef).mcu_ctr = MCU_col_num;
                    return FALSE;
                }
            }
            MCU_col_num += 1;
        }
        /* Completed an MCU row, but perhaps not an iMCU row */
        (*coef).mcu_ctr = 0;
        yoffset += 1;
    }
    /* Completed the iMCU row, advance counters for next one */
    (*coef).iMCU_row_num += 1;
    start_iMCU_row(cinfo);
    return TRUE;
}

#[cfg(any(feature = "entropy_opt_supported", feature = "c_multiscan_files_supported"))]
/*
 * Process some data in the first pass of a multi-pass case.
 * We process the equivalent of one fully interleaved MCU row ("iMCU" row)
 * per call, ie, v_samp_factor block rows for each component in the image.
 * This amount of data is read from the source buffer, DCT'd and quantized,
 * and saved into the virtual arrays.  We also generate suitable dummy blocks
 * as needed at the right and lower edges.  (The dummy blocks are constructed
 * in the virtual arrays, which have been padded appropriately.)  This makes
 * it possible for subsequent passes not to worry about real vs. dummy blocks.
 *
 * We must also emit the data to the entropy encoder.  This is conveniently
 * done by calling compress_output() after we've loaded the current strip
 * of the virtual arrays.
 *
 * NB: input_buf contains a plane for each component in image.  All
 * components are DCT'd and loaded into the virtual arrays in this pass.
 * However, it may be that only a subset of the components are emitted to
 * the entropy encoder during this first pass; be careful about looking
 * at the scan-dependent variables (MCU dimensions, etc).
 */

#[cfg(any(feature = "entropy_opt_supported", feature = "c_multiscan_files_supported"))]
unsafe fn compress_first_pass(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean {
    let coef = (*cinfo).coef as my_coef_ptr;
    let last_iMCU_row: JDIMENSION = (*cinfo).total_iMCU_rows - 1;
    let mut blocks_across: JDIMENSION;
    let mut MCUs_across: JDIMENSION;
    let mut MCUindex: JDIMENSION;
    let mut bi: c_int;
    let mut ci: c_int;
    let mut h_samp_factor: c_int;
    let mut block_row: c_int;
    let mut block_rows: c_int;
    let mut ndummy: c_int;
    let mut lastDC: JCOEF;
    let mut compptr: *mut jpeg_component_info;
    let mut buffer: JBLOCKARRAY;
    let mut thisblockrow: JBLOCKROW;
    let mut lastblockrow: JBLOCKROW;

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Align the virtual buffer for this component. */
        if let Some(access_virt_barray_fn) = (*(*(*cinfo).mem)).access_virt_barray {
            buffer = access_virt_barray_fn(
                cinfo as *mut c_void,
                (*coef).whole_image[ci as usize],
                (*coef).iMCU_row_num * (*compptr).v_samp_factor as u32,
                (*compptr).v_samp_factor as u32,
                TRUE,
            ) as JBLOCKARRAY;
        } else {
            buffer = ptr::null_mut();
        }
        /* Count non-dummy DCT block rows in this iMCU row. */
        if (*coef).iMCU_row_num < last_iMCU_row {
            block_rows = (*compptr).v_samp_factor;
        } else {
            /* NB: can't use last_row_height here, since may not be set! */
            block_rows = ((*compptr).height_in_blocks % (*compptr).v_samp_factor as u32) as c_int;
            if block_rows == 0 {
                block_rows = (*compptr).v_samp_factor;
            }
        }
        blocks_across = (*compptr).width_in_blocks;
        h_samp_factor = (*compptr).h_samp_factor;
        /* Count number of dummy blocks to be added at the right margin. */
        ndummy = (blocks_across % h_samp_factor as u32) as c_int;
        if ndummy > 0 {
            ndummy = h_samp_factor - ndummy;
        }
        /* Perform DCT for all non-dummy blocks in this iMCU row.  Each call
         * on forward_DCT processes a complete horizontal row of DCT blocks.
         */
        block_row = 0;
        while block_row < block_rows {
            thisblockrow = *buffer.add(block_row as usize);
            if let Some(forward_DCT_fn) = (*(*(*cinfo).fdct)).forward_DCT {
                forward_DCT_fn(
                    cinfo as *mut c_void,
                    compptr,
                    *input_buf.add(ci as usize),
                    thisblockrow,
                    (block_row as u32) * DCTSIZE as u32,
                    0,
                    blocks_across,
                );
            }
            if ndummy > 0 {
                /* Create dummy blocks at the right edge of the image. */
                thisblockrow = thisblockrow.add(blocks_across as usize); /* => first dummy block */
                ptr::write_bytes(
                    thisblockrow as *mut u8,
                    0,
                    (ndummy as usize) * size_of::<JBLOCK>(),
                );
                lastDC = (*thisblockrow.offset(-1))[0];
                bi = 0;
                while bi < ndummy {
                    (*thisblockrow.add(bi as usize))[0] = lastDC;
                    bi += 1;
                }
            }
            block_row += 1;
        }
        /* If at end of image, create dummy block rows as needed.
         * The tricky part here is that within each MCU, we want the DC values
         * of the dummy blocks to match the last real block's DC value.
         * This squeezes a few more bytes out of the resulting file...
         */
        if (*coef).iMCU_row_num == last_iMCU_row {
            blocks_across += ndummy as u32;	/* include lower right corner */
            MCUs_across = blocks_across / h_samp_factor as u32;
            block_row = block_rows;
            while block_row < (*compptr).v_samp_factor {
                thisblockrow = *buffer.add(block_row as usize);
                lastblockrow = *buffer.add((block_row - 1) as usize);
                ptr::write_bytes(
                    thisblockrow as *mut u8,
                    0,
                    (blocks_across as usize) * size_of::<JBLOCK>(),
                );
                MCUindex = 0;
                while MCUindex < MCUs_across {
                    lastDC = (*lastblockrow.add((h_samp_factor - 1) as usize))[0];
                    bi = 0;
                    while bi < h_samp_factor {
                        (*thisblockrow.add(bi as usize))[0] = lastDC;
                        bi += 1;
                    }
                    thisblockrow = thisblockrow.add(h_samp_factor as usize); /* advance to next MCU in row */
                    lastblockrow = lastblockrow.add(h_samp_factor as usize);
                    MCUindex += 1;
                }
                block_row += 1;
            }
        }
        ci += 1;
        compptr = compptr.add(1);
    }
    /* NB: compress_output will increment iMCU_row_num if successful.
     * A suspension return will result in redoing all the work above next time.
     */

    /* Emit data to the entropy encoder, sharing code with subsequent passes */
    return compress_output(cinfo, input_buf);
}

#[cfg(any(feature = "entropy_opt_supported", feature = "c_multiscan_files_supported"))]
/*
 * Process some data in subsequent passes of a multi-pass case.
 * We process the equivalent of one fully interleaved MCU row ("iMCU" row)
 * per call, ie, v_samp_factor block rows for each component in the scan.
 * The data is obtained from the virtual arrays and fed to the entropy coder.
 * Returns TRUE if the iMCU row is completed, FALSE if suspended.
 *
 * NB: input_buf is ignored; it is likely to be a NULL pointer.
 */

#[cfg(any(feature = "entropy_opt_supported", feature = "c_multiscan_files_supported"))]
unsafe fn compress_output(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean {
    let coef = (*cinfo).coef as my_coef_ptr;
    let mut MCU_col_num: JDIMENSION;    /* index of current MCU within row */
    let mut blkn: c_int;
    let mut ci: c_int;
    let mut xindex: c_int;
    let mut yindex: c_int;
    let mut yoffset: c_int;
    let mut start_col: JDIMENSION;
    let mut buffer: [JBLOCKARRAY; MAX_COMPS_IN_SCAN] = [ptr::null_mut(); MAX_COMPS_IN_SCAN];
    let mut buffer_ptr: JBLOCKROW;
    let mut compptr: *mut jpeg_component_info;

    /* Align the virtual buffers for the components used in this scan.
     * NB: during first pass, this is safe only because the buffers will
     * already be aligned properly, so jmemmgr.c won't need to do any I/O.
     */
    ci = 0;
    while ci < (*cinfo).comps_in_scan {
        compptr = (*cinfo).cur_comp_info[ci as usize];
        if let Some(access_virt_barray_fn) = (*(*(*cinfo).mem)).access_virt_barray {
            buffer[ci as usize] = access_virt_barray_fn(
                cinfo as *mut c_void,
                (*coef).whole_image[(*compptr).component_index as usize],
                (*coef).iMCU_row_num * (*compptr).v_samp_factor as u32,
                (*compptr).v_samp_factor as u32,
                FALSE,
            ) as JBLOCKARRAY;
        }
        ci += 1;
    }

    /* Loop to process one whole iMCU row */
    yoffset = (*coef).MCU_vert_offset;
    while yoffset < (*coef).MCU_rows_per_iMCU_row {
        MCU_col_num = (*coef).mcu_ctr;
        while MCU_col_num < (*cinfo).MCUs_per_row {
            /* Construct list of pointers to DCT blocks belonging to this MCU */
            blkn = 0;			/* index of current DCT block within MCU */
            ci = 0;
            while ci < (*cinfo).comps_in_scan {
                compptr = (*cinfo).cur_comp_info[ci as usize];
                start_col = MCU_col_num * (*compptr).MCU_width as u32;
                yindex = 0;
                while yindex < (*compptr).MCU_height {
                    buffer_ptr = *(*buffer[ci as usize]).add((yindex + yoffset) as usize);
                    buffer_ptr = buffer_ptr.add(start_col as usize);
                    xindex = 0;
                    while xindex < (*compptr).MCU_width {
                        (*coef).MCU_buffer[blkn as usize] = buffer_ptr;
                        blkn += 1;
                        buffer_ptr = buffer_ptr.add(1);
                        xindex += 1;
                    }
                    yindex += 1;
                }
                ci += 1;
            }
            /* Try to write the MCU. */
            if let Some(encode_mcu_fn) = (*(*(*cinfo).entropy)).encode_mcu {
                if encode_mcu_fn(cinfo as *mut c_void, (*coef).MCU_buffer.as_mut_ptr()) == FALSE {
                    /* Suspension forced; update state counters and exit */
                    (*coef).MCU_vert_offset = yoffset;
                    (*coef).mcu_ctr = MCU_col_num;
                    return FALSE;
                }
            }
            MCU_col_num += 1;
        }
        /* Completed an MCU row, but perhaps not an iMCU row */
        (*coef).mcu_ctr = 0;
        yoffset += 1;
    }
    /* Completed the iMCU row, advance counters for next one */
    (*coef).iMCU_row_num += 1;
    start_iMCU_row(cinfo);
    return TRUE;
}

/*
 * Initialize coefficient buffer controller.
 */

pub unsafe fn jinit_c_coef_controller(cinfo: j_compress_ptr, need_full_buffer: boolean) {
    let mut coef: my_coef_ptr;

    if let Some(alloc_small_fn) = (*(*(*cinfo).mem)).alloc_small {
        coef = alloc_small_fn(
            cinfo as *mut c_void,
            JPOOL_IMAGE,
            size_of::<my_coef_controller>(),
        ) as my_coef_ptr;
    } else {
        coef = ptr::null_mut();
    }
    (*cinfo).coef = coef as *mut jpeg_c_coef_controller;
    (*coef).r#pub.start_pass = Some(start_pass_coef as unsafe extern "C" fn(*mut c_void, J_BUF_MODE));

    /* Create the coefficient buffer. */
    if need_full_buffer != FALSE {
        #[cfg(any(feature = "entropy_opt_supported", feature = "c_multiscan_files_supported"))]
        {
            /* Allocate a full-image virtual array for each component, */
            /* padded to a multiple of samp_factor DCT blocks in each direction. */
            let mut ci: c_int;
            let mut compptr: *mut jpeg_component_info;

            ci = 0;
            compptr = (*cinfo).comp_info;
            while ci < (*cinfo).num_components {
                if let Some(request_virt_barray_fn) = (*(*(*cinfo).mem)).request_virt_barray {
                    (*coef).whole_image[ci as usize] = request_virt_barray_fn(
                        cinfo as *mut c_void,
                        JPOOL_IMAGE,
                        FALSE,
                        jround_up(
                            (*compptr).width_in_blocks as i32,
                            (*compptr).h_samp_factor,
                        ) as u32,
                        jround_up(
                            (*compptr).height_in_blocks as i32,
                            (*compptr).v_samp_factor,
                        ) as u32,
                        (*compptr).v_samp_factor as u32,
                    );
                }
                ci += 1;
                compptr = compptr.add(1);
            }
        }
        #[cfg(not(any(feature = "entropy_opt_supported", feature = "c_multiscan_files_supported")))]
        {
            // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
        }
    } else {
        /* We only need a single-MCU buffer. */
        let mut buffer: JBLOCKROW;
        let mut i: c_int;

        if let Some(alloc_large_fn) = (*(*(*cinfo).mem)).alloc_large {
            buffer = alloc_large_fn(
                cinfo as *mut c_void,
                JPOOL_IMAGE,
                C_MAX_BLOCKS_IN_MCU * size_of::<JBLOCK>(),
            ) as JBLOCKROW;
        } else {
            buffer = ptr::null_mut();
        }
        i = 0;
        while i < C_MAX_BLOCKS_IN_MCU as c_int {
            (*coef).MCU_buffer[i as usize] = buffer.add(i as usize);
            i += 1;
        }
        (*coef).whole_image[0] = ptr::null_mut(); /* flag for no virtual arrays */
    }
}

/* Helper function for rounding up division (jround_up macro equivalent) */
#[inline]
fn jround_up(a: i32, b: c_int) -> i32 {
    ((a + b - 1) / b) * b
}
