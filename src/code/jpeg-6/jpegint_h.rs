/*
 * jpegint.h
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file provides common declarations for the various JPEG modules.
 * These declarations are considered internal to the JPEG library; most
 * applications using the library shouldn't need to include this file.
 */

use core::ffi::{c_int, c_uint, c_void};

// Forward declarations for JPEG library types defined elsewhere
pub type j_compress_ptr = *mut c_void;
pub type j_decompress_ptr = *mut c_void;
pub type j_common_ptr = *mut c_void;
pub type JSAMPARRAY = *mut c_void;
pub type JSAMPIMAGE = *mut c_void;
pub type JBLOCKROW = *mut c_void;
pub type JDIMENSION = c_uint;
pub type JOCTET = u8;
pub type JCOEFPTR = *mut c_void;
pub type jpeg_component_info = c_void;
pub type jvirt_barray_ptr = *mut c_void;
pub type boolean = c_int;

/* Declarations for both compression & decompression */

/* Operating modes for buffer controllers */
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum J_BUF_MODE {
    JBUF_PASS_THRU,     /* Plain stripwise operation */
    /* Remaining modes require a full-image buffer to have been created */
    JBUF_SAVE_SOURCE,   /* Run source subobject only, save output */
    JBUF_CRANK_DEST,    /* Run dest subobject only, using saved data */
    JBUF_SAVE_AND_PASS, /* Run both subobjects, save output */
}

/* Values of global_state field (jdapi.c has some dependencies on ordering!) */
pub const CSTATE_START: c_int = 100;     /* after create_compress */
pub const CSTATE_SCANNING: c_int = 101;  /* start_compress done, write_scanlines OK */
pub const CSTATE_RAW_OK: c_int = 102;    /* start_compress done, write_raw_data OK */
pub const CSTATE_WRCOEFS: c_int = 103;   /* jpeg_write_coefficients done */
pub const DSTATE_START: c_int = 200;     /* after create_decompress */
pub const DSTATE_INHEADER: c_int = 201;  /* reading header markers, no SOS yet */
pub const DSTATE_READY: c_int = 202;     /* found SOS, ready for start_decompress */
pub const DSTATE_PRELOAD: c_int = 203;   /* reading multiscan file in start_decompress*/
pub const DSTATE_PRESCAN: c_int = 204;   /* performing dummy pass for 2-pass quant */
pub const DSTATE_SCANNING: c_int = 205;  /* start_decompress done, read_scanlines OK */
pub const DSTATE_RAW_OK: c_int = 206;    /* start_decompress done, read_raw_data OK */
pub const DSTATE_BUFIMAGE: c_int = 207;  /* expecting jpeg_start_output */
pub const DSTATE_BUFPOST: c_int = 208;   /* looking for SOS/EOI in jpeg_finish_output */
pub const DSTATE_RDCOEFS: c_int = 209;   /* reading file in jpeg_read_coefficients */
pub const DSTATE_STOPPING: c_int = 210;  /* looking for EOI in jpeg_finish_decompress */


/* Declarations for compression modules */

/* Master control module */
#[repr(C)]
pub struct jpeg_comp_master {
    pub prepare_for_pass: Option<extern "C" fn(j_compress_ptr)>,
    pub pass_startup: Option<extern "C" fn(j_compress_ptr)>,
    pub finish_pass: Option<extern "C" fn(j_compress_ptr)>,

    /* State variables made visible to other modules */
    pub call_pass_startup: boolean,  /* True if pass_startup must be called */
    pub is_last_pass: boolean,       /* True during last pass */
}

/* Main buffer control (downsampled-data buffer) */
#[repr(C)]
pub struct jpeg_c_main_controller {
    pub start_pass: Option<extern "C" fn(j_compress_ptr, J_BUF_MODE)>,
    pub process_data: Option<extern "C" fn(j_compress_ptr, JSAMPARRAY, *mut JDIMENSION, JDIMENSION)>,
}

/* Compression preprocessing (downsampling input buffer control) */
#[repr(C)]
pub struct jpeg_c_prep_controller {
    pub start_pass: Option<extern "C" fn(j_compress_ptr, J_BUF_MODE)>,
    pub pre_process_data: Option<extern "C" fn(j_compress_ptr, JSAMPARRAY, *mut JDIMENSION, JDIMENSION, JSAMPIMAGE, *mut JDIMENSION, JDIMENSION)>,
}

/* Coefficient buffer control */
#[repr(C)]
pub struct jpeg_c_coef_controller {
    pub start_pass: Option<extern "C" fn(j_compress_ptr, J_BUF_MODE)>,
    pub compress_data: Option<extern "C" fn(j_compress_ptr, JSAMPIMAGE) -> boolean>,
}

/* Colorspace conversion */
#[repr(C)]
pub struct jpeg_color_converter {
    pub start_pass: Option<extern "C" fn(j_compress_ptr)>,
    pub color_convert: Option<extern "C" fn(j_compress_ptr, JSAMPARRAY, JSAMPIMAGE, JDIMENSION, c_int)>,
}

/* Downsampling */
#[repr(C)]
pub struct jpeg_downsampler {
    pub start_pass: Option<extern "C" fn(j_compress_ptr)>,
    pub downsample: Option<extern "C" fn(j_compress_ptr, JSAMPIMAGE, JDIMENSION, JSAMPIMAGE, JDIMENSION)>,

    pub need_context_rows: boolean,  /* TRUE if need rows above & below */
}

/* Forward DCT (also controls coefficient quantization) */
#[repr(C)]
pub struct jpeg_forward_dct {
    pub start_pass: Option<extern "C" fn(j_compress_ptr)>,
    /* perhaps this should be an array??? */
    pub forward_DCT: Option<extern "C" fn(j_compress_ptr, *mut jpeg_component_info, JSAMPARRAY, JBLOCKROW, JDIMENSION, JDIMENSION, JDIMENSION)>,
}

/* Entropy encoding */
#[repr(C)]
pub struct jpeg_entropy_encoder {
    pub start_pass: Option<extern "C" fn(j_compress_ptr, boolean)>,
    pub encode_mcu: Option<extern "C" fn(j_compress_ptr, *mut JBLOCKROW) -> boolean>,
    pub finish_pass: Option<extern "C" fn(j_compress_ptr)>,
}

/* Marker writing */
#[repr(C)]
pub struct jpeg_marker_writer {
    /* write_any_marker is exported for use by applications */
    /* Probably only COM and APPn markers should be written */
    pub write_any_marker: Option<extern "C" fn(j_compress_ptr, c_int, *const JOCTET, c_uint)>,
    pub write_file_header: Option<extern "C" fn(j_compress_ptr)>,
    pub write_frame_header: Option<extern "C" fn(j_compress_ptr)>,
    pub write_scan_header: Option<extern "C" fn(j_compress_ptr)>,
    pub write_file_trailer: Option<extern "C" fn(j_compress_ptr)>,
    pub write_tables_only: Option<extern "C" fn(j_compress_ptr)>,
}


/* Declarations for decompression modules */

/* Master control module */
#[repr(C)]
pub struct jpeg_decomp_master {
    pub prepare_for_output_pass: Option<extern "C" fn(j_decompress_ptr)>,
    pub finish_output_pass: Option<extern "C" fn(j_decompress_ptr)>,

    /* State variables made visible to other modules */
    pub is_dummy_pass: boolean,  /* True during 1st pass for 2-pass quant */
}

/* Input control module */
#[repr(C)]
pub struct jpeg_input_controller {
    pub consume_input: Option<extern "C" fn(j_decompress_ptr) -> c_int>,
    pub reset_input_controller: Option<extern "C" fn(j_decompress_ptr)>,
    pub start_input_pass: Option<extern "C" fn(j_decompress_ptr)>,
    pub finish_input_pass: Option<extern "C" fn(j_decompress_ptr)>,

    /* State variables made visible to other modules */
    pub has_multiple_scans: boolean,  /* True if file has multiple scans */
    pub eoi_reached: boolean,         /* True when EOI has been consumed */
}

/* Main buffer control (downsampled-data buffer) */
#[repr(C)]
pub struct jpeg_d_main_controller {
    pub start_pass: Option<extern "C" fn(j_decompress_ptr, J_BUF_MODE)>,
    pub process_data: Option<extern "C" fn(j_decompress_ptr, JSAMPARRAY, *mut JDIMENSION, JDIMENSION)>,
}

/* Coefficient buffer control */
#[repr(C)]
pub struct jpeg_d_coef_controller {
    pub start_input_pass: Option<extern "C" fn(j_decompress_ptr)>,
    pub consume_data: Option<extern "C" fn(j_decompress_ptr) -> c_int>,
    pub start_output_pass: Option<extern "C" fn(j_decompress_ptr)>,
    pub decompress_data: Option<extern "C" fn(j_decompress_ptr, JSAMPIMAGE) -> c_int>,
    /* Pointer to array of coefficient virtual arrays, or NULL if none */
    pub coef_arrays: *mut jvirt_barray_ptr,
}

/* Decompression postprocessing (color quantization buffer control) */
#[repr(C)]
pub struct jpeg_d_post_controller {
    pub start_pass: Option<extern "C" fn(j_decompress_ptr, J_BUF_MODE)>,
    pub post_process_data: Option<extern "C" fn(j_decompress_ptr, JSAMPIMAGE, *mut JDIMENSION, JDIMENSION, JSAMPARRAY, *mut JDIMENSION, JDIMENSION)>,
}

/* Marker reading & parsing */
#[repr(C)]
pub struct jpeg_marker_reader {
    pub reset_marker_reader: Option<extern "C" fn(j_decompress_ptr)>,
    /* Read markers until SOS or EOI.
     * Returns same codes as are defined for jpeg_consume_input:
     * JPEG_SUSPENDED, JPEG_REACHED_SOS, or JPEG_REACHED_EOI.
     */
    pub read_markers: Option<extern "C" fn(j_decompress_ptr) -> c_int>,
    /* Read a restart marker --- exported for use by entropy decoder only */
    pub read_restart_marker: jpeg_marker_parser_method,
    /* Application-overridable marker processing methods */
    pub process_COM: jpeg_marker_parser_method,
    pub process_APPn: [jpeg_marker_parser_method; 16],

    /* State of marker reader --- nominally internal, but applications
     * supplying COM or APPn handlers might like to know the state.
     */
    pub saw_SOI: boolean,         /* found SOI? */
    pub saw_SOF: boolean,         /* found SOF? */
    pub next_restart_num: c_int,  /* next restart number expected (0-7) */
    pub discarded_bytes: c_uint,  /* # of bytes skipped looking for a marker */
}

/* Entropy decoding */
#[repr(C)]
pub struct jpeg_entropy_decoder {
    pub start_pass: Option<extern "C" fn(j_decompress_ptr)>,
    pub decode_mcu: Option<extern "C" fn(j_decompress_ptr, *mut JBLOCKROW) -> boolean>,
}

/* Inverse DCT (also performs dequantization) */
pub type inverse_DCT_method_ptr = extern "C" fn(j_decompress_ptr, *mut jpeg_component_info, JCOEFPTR, JSAMPARRAY, JDIMENSION);

#[repr(C)]
pub struct jpeg_inverse_dct {
    pub start_pass: Option<extern "C" fn(j_decompress_ptr)>,
    /* It is useful to allow each component to have a separate IDCT method. */
    pub inverse_DCT: [Option<inverse_DCT_method_ptr>; 10], /* MAX_COMPONENTS = 10 */
}

/* Upsampling (note that upsampler must also call color converter) */
#[repr(C)]
pub struct jpeg_upsampler {
    pub start_pass: Option<extern "C" fn(j_decompress_ptr)>,
    pub upsample: Option<extern "C" fn(j_decompress_ptr, JSAMPIMAGE, *mut JDIMENSION, JDIMENSION, JSAMPARRAY, *mut JDIMENSION, JDIMENSION)>,

    pub need_context_rows: boolean,  /* TRUE if need rows above & below */
}

/* Colorspace conversion */
#[repr(C)]
pub struct jpeg_color_deconverter {
    pub start_pass: Option<extern "C" fn(j_decompress_ptr)>,
    pub color_convert: Option<extern "C" fn(j_decompress_ptr, JSAMPIMAGE, JDIMENSION, JSAMPARRAY, c_int)>,
}

/* Color quantization or color precision reduction */
#[repr(C)]
pub struct jpeg_color_quantizer {
    pub start_pass: Option<extern "C" fn(j_decompress_ptr, boolean)>,
    pub color_quantize: Option<extern "C" fn(j_decompress_ptr, JSAMPARRAY, JSAMPARRAY, c_int)>,
    pub finish_pass: Option<extern "C" fn(j_decompress_ptr)>,
    pub new_color_map: Option<extern "C" fn(j_decompress_ptr)>,
}


/* Miscellaneous useful macros */

#[inline]
pub const fn MAX(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
}

#[inline]
pub const fn MIN(a: c_int, b: c_int) -> c_int {
    if a < b { a } else { b }
}


/* We assume that right shift corresponds to signed division by 2 with
 * rounding towards minus infinity.  This is correct for typical "arithmetic
 * shift" instructions that shift in copies of the sign bit.  But some
 * C compilers implement >> with an unsigned shift.  For these machines you
 * must define RIGHT_SHIFT_IS_UNSIGNED.
 * RIGHT_SHIFT provides a proper signed right shift of an INT32 quantity.
 * It is only applied with constant shift counts.  SHIFT_TEMPS must be
 * included in the variables of any routine using RIGHT_SHIFT.
 */

#[cfg(feature = "RIGHT_SHIFT_IS_UNSIGNED")]
pub fn RIGHT_SHIFT(x: i32, shft: u32) -> i32 {
    let shift_temp = x;
    if shift_temp < 0 {
        (shift_temp >> shft) | (((!0i32) << (32 - shft)))
    } else {
        shift_temp >> shft
    }
}

#[cfg(not(feature = "RIGHT_SHIFT_IS_UNSIGNED"))]
#[inline]
pub const fn RIGHT_SHIFT(x: i32, shft: u32) -> i32 {
    x >> shft
}


/* Short forms of external names for systems with brain-damaged linkers. */

#[cfg(feature = "NEED_SHORT_EXTERNAL_NAMES")]
pub use crate::{
    jinit_compress_master as jICompress,
    jinit_c_master_control as jICMaster,
    jinit_c_main_controller as jICMainC,
    jinit_c_prep_controller as jICPrepC,
    jinit_c_coef_controller as jICCoefC,
    jinit_color_converter as jICColor,
    jinit_downsampler as jIDownsampler,
    jinit_forward_dct as jIFDCT,
    jinit_huff_encoder as jIHEncoder,
    jinit_phuff_encoder as jIPHEncoder,
    jinit_marker_writer as jIMWriter,
    jinit_master_decompress as jIDMaster,
    jinit_d_main_controller as jIDMainC,
    jinit_d_coef_controller as jIDCoefC,
    jinit_d_post_controller as jIDPostC,
    jinit_input_controller as jIInCtlr,
    jinit_marker_reader as jIMReader,
    jinit_huff_decoder as jIHDecoder,
    jinit_phuff_decoder as jIPHDecoder,
    jinit_inverse_dct as jIIDCT,
    jinit_upsampler as jIUpsampler,
    jinit_color_deconverter as jIDColor,
    jinit_1pass_quantizer as jI1Quant,
    jinit_2pass_quantizer as jI2Quant,
    jinit_merged_upsampler as jIMUpsampler,
    jinit_memory_mgr as jIMemMgr,
    jdiv_round_up as jDivRound,
    jround_up as jRound,
    jcopy_sample_rows as jCopySamples,
    jcopy_block_row as jCopyBlocks,
    jzero_far as jZeroFar,
    jpeg_zigzag_order as jZIGTable,
    jpeg_natural_order as jZAGTable,
};


/* Compression module initialization routines */
extern "C" {
    pub fn jinit_compress_master(cinfo: j_compress_ptr);
    pub fn jinit_c_master_control(cinfo: j_compress_ptr, transcode_only: boolean);
    pub fn jinit_c_main_controller(cinfo: j_compress_ptr, need_full_buffer: boolean);
    pub fn jinit_c_prep_controller(cinfo: j_compress_ptr, need_full_buffer: boolean);
    pub fn jinit_c_coef_controller(cinfo: j_compress_ptr, need_full_buffer: boolean);
    pub fn jinit_color_converter(cinfo: j_compress_ptr);
    pub fn jinit_downsampler(cinfo: j_compress_ptr);
    pub fn jinit_forward_dct(cinfo: j_compress_ptr);
    pub fn jinit_huff_encoder(cinfo: j_compress_ptr);
    pub fn jinit_phuff_encoder(cinfo: j_compress_ptr);
    pub fn jinit_marker_writer(cinfo: j_compress_ptr);

    /* Decompression module initialization routines */
    pub fn jinit_master_decompress(cinfo: j_decompress_ptr);
    pub fn jinit_d_main_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean);
    pub fn jinit_d_coef_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean);
    pub fn jinit_d_post_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean);
    pub fn jinit_input_controller(cinfo: j_decompress_ptr);
    pub fn jinit_marker_reader(cinfo: j_decompress_ptr);
    pub fn jinit_huff_decoder(cinfo: j_decompress_ptr);
    pub fn jinit_phuff_decoder(cinfo: j_decompress_ptr);
    pub fn jinit_inverse_dct(cinfo: j_decompress_ptr);
    pub fn jinit_upsampler(cinfo: j_decompress_ptr);
    pub fn jinit_color_deconverter(cinfo: j_decompress_ptr);
    pub fn jinit_1pass_quantizer(cinfo: j_decompress_ptr);
    pub fn jinit_2pass_quantizer(cinfo: j_decompress_ptr);
    pub fn jinit_merged_upsampler(cinfo: j_decompress_ptr);

    /* Memory manager initialization */
    pub fn jinit_memory_mgr(cinfo: j_common_ptr);

    /* Utility routines in jutils.c */
    pub fn jdiv_round_up(a: i32, b: i32) -> i32;
    pub fn jround_up(a: i32, b: i32) -> i32;
    pub fn jcopy_sample_rows(
        input_array: JSAMPARRAY,
        source_row: c_int,
        output_array: JSAMPARRAY,
        dest_row: c_int,
        num_rows: c_int,
        num_cols: JDIMENSION,
    );
    pub fn jcopy_block_row(
        input_row: JBLOCKROW,
        output_row: JBLOCKROW,
        num_blocks: JDIMENSION,
    );
    pub fn jzero_far(target: *mut c_void, bytestozero: usize);

    /* Constant tables in jutils.c */
    pub static jpeg_zigzag_order: [c_int; 64];  /* natural coef order to zigzag order */
    pub static jpeg_natural_order: [c_int; 64]; /* zigzag coef order to natural order */
}

/* Suppress undefined-structure complaints if necessary. */

#[cfg(feature = "INCOMPLETE_TYPES_BROKEN")]
#[cfg(not(feature = "AM_MEMORY_MANAGER"))]
pub struct jvirt_sarray_control {
    dummy: i32,
}

#[cfg(feature = "INCOMPLETE_TYPES_BROKEN")]
#[cfg(not(feature = "AM_MEMORY_MANAGER"))]
pub struct jvirt_barray_control {
    dummy: i32,
}
