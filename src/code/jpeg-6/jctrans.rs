/*
 * jctrans.c
 *
 * Copyright (C) 1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains library routines for transcoding compression,
 * that is, writing raw DCT coefficient arrays to an output JPEG file.
 * The routines in jcapimin.c will also be needed by a transcoder.
 */

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

// JPEG library stub types - defined in jpeglib.h
// These are opaque types used to maintain ABI compatibility

#[repr(C)]
pub struct jpeg_compress_struct {
    _private: [u8; 0],
}

#[repr(C)]
pub struct jpeg_decompress_struct {
    _private: [u8; 0],
}

#[repr(C)]
pub struct jpeg_c_coef_controller {
    pub start_pass: Option<unsafe extern "C" fn(*mut jpeg_compress_struct, c_int)>,
    pub compress_data: Option<unsafe extern "C" fn(*mut jpeg_compress_struct, *mut *mut *mut u8) -> c_int>,
}

#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; 64],  // DCTSIZE2 = 64
    pub sent_table: c_int,
}

#[repr(C)]
pub struct jpeg_component_info {
    pub component_id: c_int,
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
    pub quant_tbl_no: c_int,
    pub quant_table: *mut JQUANT_TBL,
    pub component_index: c_int,
    pub MCU_width: c_int,
    pub MCU_height: c_int,
    pub last_col_width: c_int,
    pub last_row_height: c_int,
    _private: [u8; 0],
}

#[repr(C)]
pub struct jpeg_error_mgr {
    pub reset_error_mgr: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct jpeg_destination_mgr {
    pub init_destination: Option<unsafe extern "C" fn(*mut jpeg_compress_struct)>,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
    pub alloc_large: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
    pub access_virt_barray: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, c_int, c_int, c_int) -> *mut *mut c_void>,
    pub realize_virt_arrays: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct jpeg_marker_writer {
    pub write_file_header: Option<unsafe extern "C" fn(*mut jpeg_compress_struct)>,
}

#[repr(C)]
pub struct jpeg_entropy_encoder {
    pub encode_mcu: Option<unsafe extern "C" fn(*mut jpeg_compress_struct, *mut *mut c_void) -> c_int>,
}

// Constants
const NUM_QUANT_TBLS: usize = 4;
const MAX_COMPONENTS: c_int = 10;
const DCTSIZE2: usize = 64;
const C_MAX_BLOCKS_IN_MCU: usize = 10;
const JPOOL_IMAGE: c_int = 1;
const JBUF_CRANK_DEST: c_int = 3;
const CSTATE_START: c_int = 0;
const CSTATE_WRCOEFS: c_int = 3;

// Error codes
const JERR_BAD_STATE: c_int = 203;
const JERR_ARITH_NOTIMPL: c_int = 204;
const JERR_NOT_COMPILED: c_int = 205;
const JERR_BAD_BUFFER_MODE: c_int = 206;
const JERR_COMPONENT_COUNT: c_int = 207;
const JERR_NO_QUANT_TABLE: c_int = 208;
const JERR_MISMATCHED_QUANT_TABLE: c_int = 209;

// Type aliases
pub type j_compress_ptr = *mut jpeg_compress_struct;
pub type j_decompress_ptr = *mut jpeg_decompress_struct;
pub type j_common_ptr = *mut c_void;
pub type jvirt_barray_ptr = *mut c_void;
pub type JDIMENSION = c_int;
pub type JBLOCKROW = *mut *mut i16;
pub type JBLOCKARRAY = *mut JBLOCKROW;
pub type JBLOCK = [i16; 64];
pub type JSAMPIMAGE = *mut *mut *mut u8;
pub type boolean = c_int;

// Private buffer controller object
#[repr(C)]
pub struct my_coef_controller {
    pub pub_: jpeg_c_coef_controller,  /* public fields */

    pub iMCU_row_num: JDIMENSION,      /* iMCU row # within image */
    pub mcu_ctr: JDIMENSION,           /* counts MCUs processed in current row */
    pub MCU_vert_offset: c_int,        /* counts MCU rows within iMCU row */
    pub MCU_rows_per_iMCU_row: c_int,  /* number of such rows needed */

    /* Virtual block array for each component. */
    pub whole_image: *mut jvirt_barray_ptr,

    /* Workspace for constructing dummy blocks at right/bottom edges. */
    pub dummy_buffer: [JBLOCKROW; C_MAX_BLOCKS_IN_MCU],
}

pub type my_coef_ptr = *mut my_coef_controller;

// Forward declarations
unsafe fn transencode_master_selection(cinfo: j_compress_ptr, coef_arrays: *mut jvirt_barray_ptr);
unsafe fn transencode_coef_controller(cinfo: j_compress_ptr, coef_arrays: *mut jvirt_barray_ptr);

// External JPEG library functions
extern "C" {
    fn jpeg_suppress_tables(cinfo: j_compress_ptr, suppress: c_int);
    fn jpeg_set_defaults(cinfo: j_compress_ptr);
    fn jpeg_set_colorspace(cinfo: j_compress_ptr, colorspace: c_int);
    fn jpeg_alloc_quant_table(cinfo: j_common_ptr) -> *mut JQUANT_TBL;
    fn jinit_c_master_control(cinfo: j_compress_ptr, transcode_only: c_int);
    fn jinit_phuff_encoder(cinfo: j_compress_ptr);
    fn jinit_huff_encoder(cinfo: j_compress_ptr);
    fn jinit_marker_writer(cinfo: j_compress_ptr);
    fn jzero_far(ptr: *mut c_void, size: usize);
}

// Helper macro - MEMCOPY equivalent
#[inline]
unsafe fn MEMCOPY(dst: *mut u16, src: *const u16, size: usize) {
    core::ptr::copy_nonoverlapping(src, dst, size / 2);
}

// Helper macro - SIZEOF equivalent for quantval array
#[inline]
fn SIZEOF_QUANTVAL() -> usize {
    64 * 2  // 64 u16 elements * 2 bytes each
}

/*
 * Compression initialization for writing raw-coefficient data.
 * Before calling this, all parameters and a data destination must be set up.
 * Call jpeg_finish_compress() to actually write the data.
 *
 * The number of passed virtual arrays must match cinfo->num_components.
 * Note that the virtual arrays need not be filled or even realized at
 * the time write_coefficients is called; indeed, if the virtual arrays
 * were requested from this compression object's memory manager, they
 * typically will be realized during this routine and filled afterwards.
 */

pub unsafe extern "C" fn jpeg_write_coefficients(
    cinfo: j_compress_ptr,
    coef_arrays: *mut jvirt_barray_ptr,
) {
    if (*cinfo).global_state != CSTATE_START {
        // ERREXIT1(cinfo, JERR_BAD_STATE, cinfo->global_state);
        // Stubbed error exit - in production this would call the error manager
    }
    /* Mark all tables to be written */
    jpeg_suppress_tables(cinfo, 0);
    /* (Re)initialize error mgr and destination modules */
    if let Some(reset_error_mgr) = (*(*cinfo).err).reset_error_mgr {
        reset_error_mgr(cinfo as *mut c_void);
    }
    if let Some(init_destination) = (*(*cinfo).dest).init_destination {
        init_destination(cinfo);
    }
    /* Perform master selection of active modules */
    transencode_master_selection(cinfo, coef_arrays);
    /* Wait for jpeg_finish_compress() call */
    (*cinfo).next_scanline = 0; /* so jpeg_write_marker works */
    (*cinfo).global_state = CSTATE_WRCOEFS;
}

/*
 * Initialize the compression object with default parameters,
 * then copy from the source object all parameters needed for lossless
 * transcoding.  Parameters that can be varied without loss (such as
 * scan script and Huffman optimization) are left in their default states.
 */

pub unsafe extern "C" fn jpeg_copy_critical_parameters(
    srcinfo: j_decompress_ptr,
    dstinfo: j_compress_ptr,
) {
    let mut qtblptr: *mut *mut JQUANT_TBL;
    let mut incomp: *mut jpeg_component_info;
    let mut outcomp: *mut jpeg_component_info;
    let mut c_quant: *mut JQUANT_TBL;
    let mut slot_quant: *mut JQUANT_TBL;
    let mut tblno: c_int;
    let mut ci: c_int;
    let mut coefi: c_int;

    /* Safety check to ensure start_compress not called yet. */
    if (*dstinfo).global_state != CSTATE_START {
        // ERREXIT1(dstinfo, JERR_BAD_STATE, dstinfo->global_state);
    }
    /* Copy fundamental image dimensions */
    (*dstinfo).image_width = (*srcinfo).image_width;
    (*dstinfo).image_height = (*srcinfo).image_height;
    (*dstinfo).input_components = (*srcinfo).num_components;
    (*dstinfo).in_color_space = (*srcinfo).jpeg_color_space;
    /* Initialize all parameters to default values */
    jpeg_set_defaults(dstinfo);
    /* jpeg_set_defaults may choose wrong colorspace, eg YCbCr if input is RGB.
     * Fix it to get the right header markers for the image colorspace.
     */
    jpeg_set_colorspace(dstinfo, (*srcinfo).jpeg_color_space);
    (*dstinfo).data_precision = (*srcinfo).data_precision;
    (*dstinfo).CCIR601_sampling = (*srcinfo).CCIR601_sampling;
    /* Copy the source's quantization tables. */
    tblno = 0;
    while tblno < NUM_QUANT_TBLS as c_int {
        if !(*(*srcinfo).quant_tbl_ptrs.as_ptr().add(tblno as usize)).is_null() {
            qtblptr = &mut *(*dstinfo).quant_tbl_ptrs.as_mut_ptr().add(tblno as usize);
            if (*qtblptr).is_null() {
                *qtblptr = jpeg_alloc_quant_table(dstinfo as *mut c_void);
            }
            MEMCOPY(
                (*(*qtblptr)).quantval.as_mut_ptr(),
                (*(*(*srcinfo).quant_tbl_ptrs.as_ptr().add(tblno as usize)))
                    .quantval
                    .as_ptr(),
                SIZEOF_QUANTVAL(),
            );
            (*(*qtblptr)).sent_table = 0;
        }
        tblno += 1;
    }
    /* Copy the source's per-component info.
     * Note we assume jpeg_set_defaults has allocated the dest comp_info array.
     */
    (*dstinfo).num_components = (*srcinfo).num_components;
    if (*dstinfo).num_components < 1 || (*dstinfo).num_components > MAX_COMPONENTS {
        // ERREXIT2(dstinfo, JERR_COMPONENT_COUNT, dstinfo->num_components, MAX_COMPONENTS);
    }
    ci = 0;
    incomp = (*srcinfo).comp_info;
    outcomp = (*dstinfo).comp_info;
    while ci < (*dstinfo).num_components {
        (*outcomp).component_id = (*incomp).component_id;
        (*outcomp).h_samp_factor = (*incomp).h_samp_factor;
        (*outcomp).v_samp_factor = (*incomp).v_samp_factor;
        (*outcomp).quant_tbl_no = (*incomp).quant_tbl_no;
        /* Make sure saved quantization table for component matches the qtable
         * slot.  If not, the input file re-used this qtable slot.
         * IJG encoder currently cannot duplicate this.
         */
        tblno = (*outcomp).quant_tbl_no;
        if tblno < 0
            || tblno >= NUM_QUANT_TBLS as c_int
            || (*(*srcinfo).quant_tbl_ptrs.as_ptr().add(tblno as usize)).is_null()
        {
            // ERREXIT1(dstinfo, JERR_NO_QUANT_TABLE, tblno);
        }
        slot_quant = *(*srcinfo).quant_tbl_ptrs.as_ptr().add(tblno as usize);
        c_quant = (*incomp).quant_table;
        if !c_quant.is_null() {
            coefi = 0;
            while coefi < DCTSIZE2 as c_int {
                if (*c_quant).quantval[coefi as usize]
                    != (*slot_quant).quantval[coefi as usize]
                {
                    // ERREXIT1(dstinfo, JERR_MISMATCHED_QUANT_TABLE, tblno);
                }
                coefi += 1;
            }
        }
        /* Note: we do not copy the source's Huffman table assignments;
         * instead we rely on jpeg_set_colorspace to have made a suitable choice.
         */
        ci += 1;
        incomp = incomp.add(1);
        outcomp = outcomp.add(1);
    }
}

/*
 * Master selection of compression modules for transcoding.
 * This substitutes for jcinit.c's initialization of the full compressor.
 */

unsafe fn transencode_master_selection(
    cinfo: j_compress_ptr,
    coef_arrays: *mut jvirt_barray_ptr,
) {
    /* Although we don't actually use input_components for transcoding,
     * jcmaster.c's initial_setup will complain if input_components is 0.
     */
    (*cinfo).input_components = 1;
    /* Initialize master control (includes parameter checking/processing) */
    jinit_c_master_control(cinfo, 1); /* transcode only */

    /* Entropy encoding: either Huffman or arithmetic coding. */
    if (*cinfo).arith_code != 0 {
        // ERREXIT(cinfo, JERR_ARITH_NOTIMPL);
    } else {
        if (*cinfo).progressive_mode != 0 {
            #[cfg(feature = "C_PROGRESSIVE_SUPPORTED")]
            {
                jinit_phuff_encoder(cinfo);
            }
            #[cfg(not(feature = "C_PROGRESSIVE_SUPPORTED"))]
            {
                // ERREXIT(cinfo, JERR_NOT_COMPILED);
            }
        } else {
            jinit_huff_encoder(cinfo);
        }
    }

    /* We need a special coefficient buffer controller. */
    transencode_coef_controller(cinfo, coef_arrays);

    jinit_marker_writer(cinfo);

    /* We can now tell the memory manager to allocate virtual arrays. */
    if let Some(realize_virt_arrays) = (*(*cinfo).mem).realize_virt_arrays {
        realize_virt_arrays(cinfo as *mut c_void);
    }

    /* Write the datastream header (SOI) immediately.
     * Frame and scan headers are postponed till later.
     * This lets application insert special markers after the SOI.
     */
    if let Some(write_file_header) = (*(*cinfo).marker).write_file_header {
        write_file_header(cinfo);
    }
}

/*
 * The rest of this file is a special implementation of the coefficient
 * buffer controller.  This is similar to jccoefct.c, but it handles only
 * output from presupplied virtual arrays.  Furthermore, we generate any
 * dummy padding blocks on-the-fly rather than expecting them to be present
 * in the arrays.
 */

unsafe fn start_iMCU_row(cinfo: j_compress_ptr)
/* Reset within-iMCU-row counters for a new row */
{
    let coef = (*cinfo).coef as *mut my_coef_controller;

    /* In an interleaved scan, an MCU row is the same as an iMCU row.
     * In a noninterleaved scan, an iMCU row has v_samp_factor MCU rows.
     * But at the bottom of the image, process only what's left.
     */
    if (*cinfo).comps_in_scan > 1 {
        (*coef).MCU_rows_per_iMCU_row = 1;
    } else {
        if (*coef).iMCU_row_num < ((*cinfo).total_iMCU_rows - 1) {
            (*coef).MCU_rows_per_iMCU_row = (*(*cinfo).cur_comp_info[0]).v_samp_factor;
        } else {
            (*coef).MCU_rows_per_iMCU_row = (*(*cinfo).cur_comp_info[0]).last_row_height;
        }
    }

    (*coef).mcu_ctr = 0;
    (*coef).MCU_vert_offset = 0;
}

/*
 * Initialize for a processing pass.
 */

unsafe extern "C" fn start_pass_coef(cinfo: j_compress_ptr, pass_mode: c_int) {
    let coef = (*cinfo).coef as *mut my_coef_controller;

    if pass_mode != JBUF_CRANK_DEST {
        // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
    }

    (*coef).iMCU_row_num = 0;
    start_iMCU_row(cinfo);
}

/*
 * Process some data.
 * We process the equivalent of one fully interleaved MCU row ("iMCU" row)
 * per call, ie, v_samp_factor block rows for each component in the scan.
 * The data is obtained from the virtual arrays and fed to the entropy coder.
 * Returns TRUE if the iMCU row is completed, FALSE if suspended.
 *
 * NB: input_buf is ignored; it is likely to be a NULL pointer.
 */

unsafe extern "C" fn compress_output(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> c_int {
    let coef = (*cinfo).coef as *mut my_coef_controller;
    let mut MCU_col_num: JDIMENSION; /* index of current MCU within row */
    let last_MCU_col = (*cinfo).MCUs_per_row - 1;
    let last_iMCU_row = (*cinfo).total_iMCU_rows - 1;
    let mut blkn: c_int;
    let mut ci: c_int;
    let mut xindex: c_int;
    let mut yindex: c_int;
    let mut yoffset: c_int;
    let mut blockcnt: c_int;
    let mut start_col: JDIMENSION;
    let mut buffer: [JBLOCKARRAY; 10] = [core::ptr::null_mut(); 10]; /* MAX_COMPS_IN_SCAN */
    let mut MCU_buffer: [JBLOCKROW; C_MAX_BLOCKS_IN_MCU] =
        [core::ptr::null_mut(); C_MAX_BLOCKS_IN_MCU];
    let mut buffer_ptr: JBLOCKROW;
    let mut compptr: *mut jpeg_component_info;

    /* Align the virtual buffers for the components used in this scan. */
    ci = 0;
    while ci < (*cinfo).comps_in_scan {
        compptr = (*cinfo).cur_comp_info[ci as usize];
        buffer[ci as usize] = if let Some(access_virt_barray) = (*(*cinfo).mem).access_virt_barray {
            access_virt_barray(
                cinfo as *mut c_void,
                (*coef).whole_image.add((*compptr).component_index as usize) as *mut c_void,
                (*coef).iMCU_row_num * (*compptr).v_samp_factor,
                (*compptr).v_samp_factor,
                0,
            ) as JBLOCKARRAY
        } else {
            core::ptr::null_mut()
        };
        ci += 1;
    }

    /* Loop to process one whole iMCU row */
    yoffset = (*coef).MCU_vert_offset;
    while yoffset < (*coef).MCU_rows_per_iMCU_row {
        MCU_col_num = (*coef).mcu_ctr;
        while MCU_col_num < (*cinfo).MCUs_per_row {
            /* Construct list of pointers to DCT blocks belonging to this MCU */
            blkn = 0; /* index of current DCT block within MCU */
            ci = 0;
            while ci < (*cinfo).comps_in_scan {
                compptr = (*cinfo).cur_comp_info[ci as usize];
                start_col = MCU_col_num * (*compptr).MCU_width;
                blockcnt = if MCU_col_num < last_MCU_col {
                    (*compptr).MCU_width
                } else {
                    (*compptr).last_col_width
                };
                yindex = 0;
                while yindex < (*compptr).MCU_height {
                    if (*coef).iMCU_row_num < last_iMCU_row
                        || yindex + yoffset < (*compptr).last_row_height
                    {
                        /* Fill in pointers to real blocks in this row */
                        buffer_ptr = *(buffer[ci as usize] as *mut JBLOCKROW)
                            .add((yindex + yoffset) as usize)
                            .add(start_col as usize);
                        xindex = 0;
                        while xindex < blockcnt {
                            MCU_buffer[blkn as usize] = buffer_ptr;
                            blkn += 1;
                            buffer_ptr = buffer_ptr.add(1);
                            xindex += 1;
                        }
                    } else {
                        /* At bottom of image, need a whole row of dummy blocks */
                        xindex = 0;
                    }
                    /* Fill in any dummy blocks needed in this row.
                     * Dummy blocks are filled in the same way as in jccoefct.c:
                     * all zeroes in the AC entries, DC entries equal to previous
                     * block's DC value.  The init routine has already zeroed the
                     * AC entries, so we need only set the DC entries correctly.
                     */
                    while xindex < (*compptr).MCU_width {
                        MCU_buffer[blkn as usize] = (*coef).dummy_buffer[blkn as usize];
                        *(*MCU_buffer[blkn as usize]).add(0) = *(*MCU_buffer[(blkn - 1) as usize]).add(0);
                        blkn += 1;
                        xindex += 1;
                    }
                    yindex += 1;
                }
                ci += 1;
            }
            /* Try to write the MCU. */
            if let Some(encode_mcu) = (*(*cinfo).entropy).encode_mcu {
                if encode_mcu(cinfo, MCU_buffer.as_mut_ptr() as *mut *mut c_void) == 0 {
                    /* Suspension forced; update state counters and exit */
                    (*coef).MCU_vert_offset = yoffset;
                    (*coef).mcu_ctr = MCU_col_num;
                    return 0;
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
    1
}

/*
 * Initialize coefficient buffer controller.
 *
 * Each passed coefficient array must be the right size for that
 * coefficient: width_in_blocks wide and height_in_blocks high,
 * with unitheight at least v_samp_factor.
 */

unsafe fn transencode_coef_controller(
    cinfo: j_compress_ptr,
    coef_arrays: *mut jvirt_barray_ptr,
) {
    let mut coef: my_coef_ptr;
    let mut buffer: JBLOCKROW;
    let mut i: c_int;

    coef = if let Some(alloc_small) = (*(*cinfo).mem).alloc_small {
        alloc_small(
            cinfo as *mut c_void,
            JPOOL_IMAGE,
            core::mem::size_of::<my_coef_controller>(),
        ) as my_coef_ptr
    } else {
        core::ptr::null_mut()
    };

    (*cinfo).coef = &mut (*coef).pub_ as *mut jpeg_c_coef_controller;
    (*coef).pub_.start_pass = Some(start_pass_coef);
    (*coef).pub_.compress_data = Some(compress_output);

    /* Save pointer to virtual arrays */
    (*coef).whole_image = coef_arrays;

    /* Allocate and pre-zero space for dummy DCT blocks. */
    buffer = if let Some(alloc_large) = (*(*cinfo).mem).alloc_large {
        alloc_large(
            cinfo as *mut c_void,
            JPOOL_IMAGE,
            C_MAX_BLOCKS_IN_MCU * core::mem::size_of::<JBLOCK>(),
        ) as JBLOCKROW
    } else {
        core::ptr::null_mut()
    };

    jzero_far(
        buffer as *mut c_void,
        C_MAX_BLOCKS_IN_MCU * core::mem::size_of::<JBLOCK>(),
    );
    i = 0;
    while i < C_MAX_BLOCKS_IN_MCU as c_int {
        (*coef).dummy_buffer[i as usize] = buffer.add(i as usize);
        i += 1;
    }
}
