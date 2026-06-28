/*
 * jdtrans.c
 *
 * Copyright (C) 1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains library routines for transcoding decompression,
 * that is, reading raw DCT coefficient arrays from an input JPEG file.
 * The routines in jdapimin.c will also be needed by a transcoder.
 */

// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_void};
use core::ptr::addr_of_mut;

// JPEG error codes (from jerror.h)
const JERR_ARITH_NOTIMPL: c_int = 60;
const JERR_NOT_COMPILED: c_int = 62;
const JERR_BAD_STATE: c_int = 10;

// JPEG state codes
const DSTATE_READY: c_int = 200;
const DSTATE_RDCOEFS: c_int = 205;
const DSTATE_STOPPING: c_int = 210;

// Input controller return codes
const JPEG_SUSPENDED: c_int = 0;
const JPEG_REACHED_EOI: c_int = 1;
const JPEG_ROW_COMPLETED: c_int = 2;
const JPEG_REACHED_SOS: c_int = 3;

// Opaque types from jpeglib.h - these are undefined here,
// but used as pointers to external structures
#[repr(C)]
pub struct jvirt_barray_control {
    _private: [u8; 0],
}
pub type jvirt_barray_ptr = *mut jvirt_barray_control;

#[repr(C)]
pub struct j_common_struct {
    _private: [u8; 0],
}
pub type j_common_ptr = *mut j_common_struct;

#[repr(C)]
pub struct jpeg_error_mgr {
    pub error_exit: unsafe extern "C" fn(j_common_ptr),
    pub emit_message: unsafe extern "C" fn(j_common_ptr, c_int),
    pub output_message: unsafe extern "C" fn(j_common_ptr),
    pub format_message: unsafe extern "C" fn(j_common_ptr, *mut i8),
    pub reset_error_mgr: unsafe extern "C" fn(j_common_ptr),
    pub msg_code: c_int,
    pub msg_parm: MsgParm,
}

#[repr(C)]
pub union MsgParm {
    pub i: [c_int; 8],
    pub s: [i8; 80],
}

#[repr(C)]
pub struct jpeg_progress_mgr {
    pub progress_monitor: unsafe extern "C" fn(j_common_ptr),
    pub pass_counter: i64,
    pub pass_limit: i64,
    pub completed_passes: c_int,
    pub total_passes: c_int,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void,
    pub alloc_large: unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void,
    pub alloc_sarray: unsafe extern "C" fn(j_common_ptr, c_int, c_int, c_int) -> *mut *mut i8,
    pub alloc_barray: unsafe extern "C" fn(j_common_ptr, c_int, c_int, c_int) -> *mut *mut i16,
    pub request_virt_sarray: unsafe extern "C" fn(j_common_ptr, c_int, c_int, c_int, c_int, c_int) -> *mut c_void,
    pub request_virt_barray: unsafe extern "C" fn(j_common_ptr, c_int, c_int, c_int, c_int, c_int) -> jvirt_barray_ptr,
    pub realize_virt_arrays: unsafe extern "C" fn(j_common_ptr),
    pub access_virt_sarray: unsafe extern "C" fn(j_common_ptr, *mut c_void, c_int, c_int, c_int) -> *mut *mut i8,
    pub access_virt_barray: unsafe extern "C" fn(j_common_ptr, jvirt_barray_ptr, c_int, c_int, c_int) -> *mut *mut i16,
    pub free_pool: unsafe extern "C" fn(j_common_ptr, c_int),
    pub self_destruct: unsafe extern "C" fn(j_common_ptr),
    pub max_memory_to_use: i64,
    pub max_alloc_chunk: i64,
}

#[repr(C)]
pub struct jpeg_input_controller {
    pub consume_input: unsafe extern "C" fn(*mut jpeg_decompress_struct) -> c_int,
    pub reset_input_controller: unsafe extern "C" fn(*mut jpeg_decompress_struct),
    pub start_input_pass: unsafe extern "C" fn(*mut jpeg_decompress_struct),
    pub finish_input_pass: unsafe extern "C" fn(*mut jpeg_decompress_struct),
    pub has_multiple_scans: c_int,
    pub eoi_reached: c_int,
}

#[repr(C)]
pub struct jpeg_d_coef_controller {
    pub coef_arrays: *mut jvirt_barray_ptr,
    _private: [u8; 0],
}

#[repr(C)]
pub struct jpeg_decompress_struct {
    /* Common fields */
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub progress: *mut jpeg_progress_mgr,
    pub is_decompressor: c_int,
    pub global_state: c_int,

    /* Source of compressed data */
    pub src: *mut c_void,

    /* Basic image description */
    pub image_width: c_int,
    pub image_height: c_int,
    pub num_components: c_int,
    pub jpeg_color_space: c_int,

    /* Decompression parameters */
    pub out_color_space: c_int,
    pub scale_num: c_int,
    pub scale_denom: c_int,
    pub output_gamma: f64,

    pub buffered_image: c_int,
    pub raw_data_out: c_int,
    pub dct_method: c_int,
    pub do_fancy_upsampling: c_int,
    pub do_block_smoothing: c_int,

    pub quantize_colors: c_int,
    pub dither_mode: c_int,
    pub two_pass_quantize: c_int,
    pub desired_number_of_colors: c_int,
    pub enable_1pass_quant: c_int,
    pub enable_external_quant: c_int,
    pub enable_2pass_quant: c_int,

    /* Output image description */
    pub output_width: c_int,
    pub output_height: c_int,
    pub out_color_components: c_int,
    pub output_components: c_int,
    pub rec_outbuf_height: c_int,

    /* Color quantization tables */
    pub actual_number_of_colors: c_int,
    pub colormap: *mut *mut i8,

    /* State variables */
    pub output_scanline: c_int,
    pub input_scan_number: c_int,
    pub input_iMCU_row: c_int,
    pub output_scan_number: c_int,
    pub output_iMCU_row: c_int,

    pub coef_bits: *mut *mut c_int,

    /* Quantization and Huffman tables */
    pub quant_tbl_ptrs: [*mut c_void; 4],
    pub dc_huff_tbl_ptrs: [*mut c_void; 4],
    pub ac_huff_tbl_ptrs: [*mut c_void; 4],

    /* These parameters never change across datastreams */
    pub data_precision: c_int,
    pub comp_info: *mut c_void,
    pub progressive_mode: c_int,
    pub arith_code: c_int,

    pub arith_dc_L: [u8; 16],
    pub arith_dc_U: [u8; 16],
    pub arith_ac_K: [u8; 16],

    pub restart_interval: c_int,
    pub saw_JFIF_marker: c_int,
    pub density_unit: u8,
    pub X_density: u16,
    pub Y_density: u16,
    pub saw_Adobe_marker: c_int,
    pub Adobe_transform: u8,
    pub CCIR601_sampling: c_int,

    /* Fields computed during decompression startup */
    pub max_h_samp_factor: c_int,
    pub max_v_samp_factor: c_int,
    pub min_DCT_scaled_size: c_int,
    pub total_iMCU_rows: c_int,

    pub sample_range_limit: *mut i8,

    /* Fields valid during any one scan */
    pub comps_in_scan: c_int,
    pub cur_comp_info: [*mut c_void; 10],
    pub MCUs_per_row: c_int,
    pub MCU_rows_in_scan: c_int,
    pub blocks_in_MCU: c_int,
    pub MCU_membership: [c_int; 10],

    pub Ss: c_int,
    pub Se: c_int,
    pub Ah: c_int,
    pub Al: c_int,

    pub unread_marker: c_int,

    /* Subobject links */
    pub master: *mut c_void,
    pub main: *mut c_void,
    pub coef: *mut jpeg_d_coef_controller,
    pub post: *mut c_void,
    pub inputctl: *mut jpeg_input_controller,
    pub marker: *mut c_void,
    pub entropy: *mut c_void,
    pub idct: *mut c_void,
    pub upsample: *mut c_void,
    pub cconvert: *mut c_void,
    pub cquantize: *mut c_void,
}

pub type j_decompress_ptr = *mut jpeg_decompress_struct;

// Forward declarations of external JPEG functions
extern "C" {
    pub fn jinit_phuff_decoder(cinfo: j_decompress_ptr);
    pub fn jinit_huff_decoder(cinfo: j_decompress_ptr);
    pub fn jinit_d_coef_controller(cinfo: j_decompress_ptr, need_full_buffer: c_int);
}

// Macro-like helper for ERREXIT - preserves the C behavior of calling error_exit
#[inline]
fn errexit(cinfo: j_decompress_ptr, code: c_int) -> ! {
    unsafe {
        (*(*cinfo).err).msg_code = code;
        let error_exit = (*(*cinfo).err).error_exit;
        error_exit(cinfo as j_common_ptr);
        core::hint::unreachable_unchecked()
    }
}

// Macro-like helper for ERREXIT1
#[inline]
fn errexit1(cinfo: j_decompress_ptr, code: c_int, p1: c_int) -> ! {
    unsafe {
        (*(*cinfo).err).msg_code = code;
        (*(*cinfo).err).msg_parm.i[0] = p1;
        let error_exit = (*(*cinfo).err).error_exit;
        error_exit(cinfo as j_common_ptr);
        core::hint::unreachable_unchecked()
    }
}

/*
 * Read the coefficient arrays from a JPEG file.
 * jpeg_read_header must be completed before calling this.
 *
 * The entire image is read into a set of virtual coefficient-block arrays,
 * one per component.  The return value is a pointer to the array of
 * virtual-array descriptors.  These can be manipulated directly via the
 * JPEG memory manager, or handed off to jpeg_write_coefficients().
 * To release the memory occupied by the virtual arrays, call
 * jpeg_finish_decompress() when done with the data.
 *
 * Returns NULL if suspended.  This case need be checked only if
 * a suspending data source is used.
 */

pub unsafe fn jpeg_read_coefficients(cinfo: j_decompress_ptr) -> *mut jvirt_barray_ptr {
    if (*cinfo).global_state == DSTATE_READY {
        /* First call: initialize active modules */
        transdecode_master_selection(cinfo);
        (*cinfo).global_state = DSTATE_RDCOEFS;
    } else if (*cinfo).global_state != DSTATE_RDCOEFS {
        errexit1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Absorb whole file into the coef buffer */
    loop {
        let retcode: c_int;
        /* Call progress monitor hook if present */
        if !(*cinfo).progress.is_null() {
            let progress_monitor = (*(*cinfo).progress).progress_monitor;
            progress_monitor(cinfo as j_common_ptr);
        }
        /* Absorb some more input */
        let consume_input = (*(*cinfo).inputctl).consume_input;
        retcode = consume_input(cinfo);
        if retcode == JPEG_SUSPENDED {
            return core::ptr::null_mut();
        }
        if retcode == JPEG_REACHED_EOI {
            break;
        }
        /* Advance progress counter if appropriate */
        if !(*cinfo).progress.is_null()
            && (retcode == JPEG_ROW_COMPLETED || retcode == JPEG_REACHED_SOS)
        {
            (*(*cinfo).progress).pass_counter += 1;
            if (*(*cinfo).progress).pass_counter >= (*(*cinfo).progress).pass_limit {
                /* startup underestimated number of scans; ratchet up one scan */
                (*(*cinfo).progress).pass_limit +=
                    (*cinfo).total_iMCU_rows as i64;
            }
        }
    }
    /* Set state so that jpeg_finish_decompress does the right thing */
    (*cinfo).global_state = DSTATE_STOPPING;
    (*(*cinfo).coef).coef_arrays
}

/*
 * Master selection of decompression modules for transcoding.
 * This substitutes for jdmaster.c's initialization of the full decompressor.
 */

fn transdecode_master_selection(cinfo: j_decompress_ptr) {
    /* Entropy decoding: either Huffman or arithmetic coding. */
    unsafe {
        if (*cinfo).arith_code != 0 {
            errexit(cinfo, JERR_ARITH_NOTIMPL);
        } else {
            if (*cinfo).progressive_mode != 0 {
                #[cfg(feature = "D_PROGRESSIVE_SUPPORTED")]
                {
                    jinit_phuff_decoder(cinfo);
                }
                #[cfg(not(feature = "D_PROGRESSIVE_SUPPORTED"))]
                {
                    errexit(cinfo, JERR_NOT_COMPILED);
                }
            } else {
                jinit_huff_decoder(cinfo);
            }
        }

        /* Always get a full-image coefficient buffer. */
        jinit_d_coef_controller(cinfo, 1);

        /* We can now tell the memory manager to allocate virtual arrays. */
        let realize_virt_arrays = (*(*cinfo).mem).realize_virt_arrays;
        realize_virt_arrays(cinfo as j_common_ptr);

        /* Initialize input side of decompressor to consume first scan. */
        let start_input_pass = (*(*cinfo).inputctl).start_input_pass;
        start_input_pass(cinfo);

        /* Initialize progress monitoring. */
        if !(*cinfo).progress.is_null() {
            let nscans: c_int;
            /* Estimate number of scans to set pass_limit. */
            if (*cinfo).progressive_mode != 0 {
                /* Arbitrarily estimate 2 interleaved DC scans + 3 AC scans/component. */
                nscans = 2 + 3 * (*cinfo).num_components;
            } else if (*(*cinfo).inputctl).has_multiple_scans != 0 {
                /* For a nonprogressive multiscan file, estimate 1 scan per component. */
                nscans = (*cinfo).num_components;
            } else {
                nscans = 1;
            }
            (*(*cinfo).progress).pass_counter = 0;
            (*(*cinfo).progress).pass_limit =
                (*cinfo).total_iMCU_rows as i64 * nscans as i64;
            (*(*cinfo).progress).completed_passes = 0;
            (*(*cinfo).progress).total_passes = 1;
        }
    }
}
