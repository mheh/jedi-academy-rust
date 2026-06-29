/*
 * jcinit.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains initialization logic for the JPEG compressor.
 * This routine is in charge of selecting the modules to be executed and
 * making an initialization call to each one.
 *
 * Logically, this code belongs in jcmaster.c.  It's split out because
 * linking this routine implies linking the entire compression library.
 * For a transcoding-only application, we want to be able to use jcmaster.c
 * without linking in the whole library.
 */

use core::ffi::{c_int, c_void};

// Stub types for JPEG library structures
// Full definitions would come from jinclude.h and jpeglib.h

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub realize_virt_arrays: unsafe extern "C" fn(*mut c_void),
}

#[repr(C)]
pub struct jpeg_marker_writer {
    pub write_file_header: unsafe extern "C" fn(*mut c_void),
}

#[repr(C)]
pub struct j_compress_info {
    pub raw_data_in: c_int,
    pub arith_code: c_int,
    pub progressive_mode: c_int,
    pub num_scans: c_int,
    pub optimize_coding: c_int,
    pub mem: *mut jpeg_memory_mgr,
    pub marker: *mut jpeg_marker_writer,
}

pub type j_compress_ptr = *mut j_compress_info;
pub type j_common_ptr = *mut c_void;

const FALSE: c_int = 0;

// Error codes
const JERR_ARITH_NOTIMPL: c_int = 1;
const JERR_NOT_COMPILED: c_int = 2;

// Macro for ERREXIT - stub that preserves control flow
macro_rules! ERREXIT {
    ($cinfo:expr, $code:expr) => {
        {
            let _ = $code;
            return;
        }
    };
}

// JPEG library initialization functions
extern "C" {
    pub fn jinit_c_master_control(cinfo: j_compress_ptr, need_full_buffer: c_int);
    pub fn jinit_color_converter(cinfo: j_compress_ptr);
    pub fn jinit_downsampler(cinfo: j_compress_ptr);
    pub fn jinit_c_prep_controller(cinfo: j_compress_ptr, need_full_buffer: c_int);
    pub fn jinit_forward_dct(cinfo: j_compress_ptr);
    pub fn jinit_phuff_encoder(cinfo: j_compress_ptr);
    pub fn jinit_huff_encoder(cinfo: j_compress_ptr);
    pub fn jinit_c_coef_controller(cinfo: j_compress_ptr, need_full_buffer: c_int);
    pub fn jinit_c_main_controller(cinfo: j_compress_ptr, need_full_buffer: c_int);
    pub fn jinit_marker_writer(cinfo: j_compress_ptr);
}

/*
 * Master selection of compression modules.
 * This is done once at the start of processing an image.  We determine
 * which modules will be used and give them appropriate initialization calls.
 */
pub unsafe extern "C" fn jinit_compress_master(cinfo: j_compress_ptr) {
    /* Initialize master control (includes parameter checking/processing) */
    jinit_c_master_control(cinfo, FALSE /* full compression */);

    /* Preprocessing */
    if (*cinfo).raw_data_in == 0 {
        jinit_color_converter(cinfo);
        jinit_downsampler(cinfo);
        jinit_c_prep_controller(cinfo, FALSE /* never need full buffer here */);
    }
    /* Forward DCT */
    jinit_forward_dct(cinfo);
    /* Entropy encoding: either Huffman or arithmetic coding. */
    if (*cinfo).arith_code != 0 {
        ERREXIT!(cinfo, JERR_ARITH_NOTIMPL);
    } else {
        if (*cinfo).progressive_mode != 0 {
            #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
            {
                jinit_phuff_encoder(cinfo);
            }
            #[cfg(not(feature = "C_PROGRESSIVE_SUPPORTED"))]
            {
                ERREXIT!(cinfo, JERR_NOT_COMPILED);
            }
        } else {
            jinit_huff_encoder(cinfo);
        }
    }

    /* Need a full-image coefficient buffer in any multi-pass mode. */
    jinit_c_coef_controller(cinfo,
                            if (*cinfo).num_scans > 1 || (*cinfo).optimize_coding != 0 { 1 } else { 0 });
    jinit_c_main_controller(cinfo, FALSE /* never need full buffer here */);

    jinit_marker_writer(cinfo);

    /* We can now tell the memory manager to allocate virtual arrays. */
    ((*(*cinfo).mem).realize_virt_arrays)(cinfo as j_common_ptr);

    /* Write the datastream header (SOI) immediately.
     * Frame and scan headers are postponed till later.
     * This lets application insert special markers after the SOI.
     */
    ((*(*cinfo).marker).write_file_header)(cinfo as j_common_ptr);
}
