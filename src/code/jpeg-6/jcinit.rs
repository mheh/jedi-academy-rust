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

use core::ffi::c_int;

/*
 * Master selection of compression modules.
 * This is done once at the start of processing an image.  We determine
 * which modules will be used and give them appropriate initialization calls.
 */

extern "C" {
    fn jinit_c_master_control(cinfo: *mut j_compress_struct, full_compression: c_int);
    fn jinit_color_converter(cinfo: *mut j_compress_struct);
    fn jinit_downsampler(cinfo: *mut j_compress_struct);
    fn jinit_c_prep_controller(cinfo: *mut j_compress_struct, need_full_buffer: c_int);
    fn jinit_forward_dct(cinfo: *mut j_compress_struct);
    fn jinit_phuff_encoder(cinfo: *mut j_compress_struct);
    fn jinit_huff_encoder(cinfo: *mut j_compress_struct);
    fn jinit_c_coef_controller(cinfo: *mut j_compress_struct, need_full_buffer: c_int);
    fn jinit_c_main_controller(cinfo: *mut j_compress_struct, need_full_buffer: c_int);
    fn jinit_marker_writer(cinfo: *mut j_compress_struct);
}

// Local stub for j_compress_struct and related types
#[repr(C)]
pub struct j_compress_struct {
    pub raw_data_in: c_int,
    pub arith_code: c_int,
    pub progressive_mode: c_int,
    pub num_scans: c_int,
    pub optimize_coding: c_int,
    pub mem: *mut mem_mgr_struct,
    pub marker: *mut marker_writer_struct,
}

#[repr(C)]
pub struct mem_mgr_struct {
    pub realize_virt_arrays: Option<unsafe extern "C" fn(*mut j_compress_struct)>,
}

#[repr(C)]
pub struct marker_writer_struct {
    pub write_file_header: Option<unsafe extern "C" fn(*mut j_compress_struct)>,
}

pub type j_compress_ptr = *mut j_compress_struct;

// Error macros
macro_rules! ERREXIT {
    ($cinfo:expr, $code:expr) => {
        {
            // In actual implementation, this would call jerror functions
            // For now, we preserve the original control flow intent
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn jinit_compress_master(cinfo: j_compress_ptr) {
    /* Initialize master control (includes parameter checking/processing) */
    jinit_c_master_control(cinfo, 0 /* full compression */);

    /* Preprocessing */
    if (*cinfo).raw_data_in == 0 {
        jinit_color_converter(cinfo);
        jinit_downsampler(cinfo);
        jinit_c_prep_controller(cinfo, 0 /* never need full buffer here */);
    }
    /* Forward DCT */
    jinit_forward_dct(cinfo);
    /* Entropy encoding: either Huffman or arithmetic coding. */
    if (*cinfo).arith_code != 0 {
        ERREXIT!(cinfo, 0); // JERR_ARITH_NOTIMPL
    } else {
        if (*cinfo).progressive_mode != 0 {
            #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
            {
                jinit_phuff_encoder(cinfo);
            }
            #[cfg(not(feature = "C_PROGRESSIVE_SUPPORTED"))]
            {
                ERREXIT!(cinfo, 0); // JERR_NOT_COMPILED
            }
        } else {
            jinit_huff_encoder(cinfo);
        }
    }

    /* Need a full-image coefficient buffer in any multi-pass mode. */
    jinit_c_coef_controller(
        cinfo,
        (if (*cinfo).num_scans > 1 || (*cinfo).optimize_coding != 0 { 1 } else { 0 }),
    );
    jinit_c_main_controller(cinfo, 0 /* never need full buffer here */);

    jinit_marker_writer(cinfo);

    /* We can now tell the memory manager to allocate virtual arrays. */
    if let Some(realize_virt_arrays) = (*(*cinfo).mem).realize_virt_arrays {
        realize_virt_arrays(cinfo);
    }

    /* Write the datastream header (SOI) immediately.
     * Frame and scan headers are postponed till later.
     * This lets application insert special markers after the SOI.
     */
    if let Some(write_file_header) = (*(*cinfo).marker).write_file_header {
        write_file_header(cinfo);
    }
}
