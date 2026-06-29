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

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::c_int;

// External JPEG library types (defined in jpeglib and jinclude headers)
pub type j_compress_ptr = *mut c_void;
pub type j_common_ptr = *mut c_void;
pub type JDIMENSION = u32;
pub type JQUANT_TBL = [u16; 64];
pub type jvirt_barray_ptr = *mut c_void;
pub type JSAMPIMAGE = *mut c_void;
pub type JBLOCKARRAY = *mut c_void;
pub type JBLOCKROW = *mut c_void;
pub type JBLOCK = [i16; 64];
pub type J_BUF_MODE = c_int;
pub type boolean = c_int;

// JPEG library constants
const NUM_QUANT_TBLS: usize = 4;
const MAX_COMPONENTS: c_int = 10;
const DCTSIZE2: usize = 64;
const C_MAX_BLOCKS_IN_MCU: usize = 10;
const MAX_COMPS_IN_SCAN: usize = 4;
const CSTATE_START: c_int = 0;
const CSTATE_WRCOEFS: c_int = 2;
const JBUF_CRANK_DEST: c_int = 2;
const FALSE: c_int = 0;
const TRUE: c_int = 1;

// External JPEG structures (opaque)
#[repr(C)]
pub struct jpeg_c_coef_controller {
    _private: [u8; 0],
}

#[repr(C)]
pub struct jpeg_component_info {
    component_id: c_int,
    h_samp_factor: c_int,
    v_samp_factor: c_int,
    quant_tbl_no: c_int,
    quant_table: *mut c_void,
    component_index: c_int,
    MCU_width: c_int,
    MCU_height: c_int,
    last_col_width: c_int,
    last_row_height: c_int,
}

// Private buffer controller object
#[repr(C)]
struct my_coef_controller {
    pub: jpeg_c_coef_controller,
    iMCU_row_num: JDIMENSION,
    mcu_ctr: JDIMENSION,
    MCU_vert_offset: c_int,
    MCU_rows_per_iMCU_row: c_int,
    whole_image: *mut jvirt_barray_ptr,
    dummy_buffer: [JBLOCKROW; C_MAX_BLOCKS_IN_MCU],
}

type my_coef_ptr = *mut my_coef_controller;

// Forward declarations
unsafe fn transencode_master_selection(cinfo: j_compress_ptr, coef_arrays: *mut jvirt_barray_ptr);
unsafe fn transencode_coef_controller(cinfo: j_compress_ptr, coef_arrays: *mut jvirt_barray_ptr);

// Placeholder for external JPEG error/function calls
// These would be linked from the JPEG library
extern "C" {
    fn jpeg_suppress_tables(cinfo: j_compress_ptr, suppress: c_int);
    fn jinit_c_master_control(cinfo: j_compress_ptr, transcode_only: c_int);
    fn jinit_phuff_encoder(cinfo: j_compress_ptr);
    fn jinit_huff_encoder(cinfo: j_compress_ptr);
    fn jinit_marker_writer(cinfo: j_compress_ptr);
    fn jpeg_set_defaults(cinfo: j_compress_ptr);
    fn jpeg_set_colorspace(cinfo: j_compress_ptr, colorspace: c_int);
    fn jpeg_alloc_quant_table(common: j_common_ptr) -> *mut c_void;
    fn jzero_far(ptr: *mut c_void, size: usize);
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

pub unsafe fn jpeg_write_coefficients(cinfo: j_compress_ptr, coef_arrays: *mut jvirt_barray_ptr) {
    // Check state - these would be actual checks in real code
    // if (cinfo->global_state != CSTATE_START)
    //     ERREXIT1(cinfo, JERR_BAD_STATE, cinfo->global_state);

    // Mark all tables to be written
    jpeg_suppress_tables(cinfo, FALSE);
    // (Re)initialize error mgr and destination modules
    // (*cinfo->err->reset_error_mgr) ((j_common_ptr) cinfo);
    // (*cinfo->dest->init_destination) (cinfo);

    // Perform master selection of active modules
    transencode_master_selection(cinfo, coef_arrays);

    // Wait for jpeg_finish_compress() call
    // cinfo->next_scanline = 0;	/* so jpeg_write_marker works */
    // cinfo->global_state = CSTATE_WRCOEFS;
}


/*
 * Initialize the compression object with default parameters,
 * then copy from the source object all parameters needed for lossless
 * transcoding.  Parameters that can be varied without loss (such as
 * scan script and Huffman optimization) are left in their default states.
 */

pub unsafe fn jpeg_copy_critical_parameters(srcinfo: j_compress_ptr, dstinfo: j_compress_ptr) {
    let mut qtblptr: *mut *mut c_void;
    let mut incomp: *mut jpeg_component_info;
    let mut outcomp: *mut jpeg_component_info;
    let mut c_quant: *mut c_void;
    let mut slot_quant: *mut c_void;
    let mut tblno: c_int;
    let mut ci: c_int;
    let mut coefi: c_int;

    // Safety check to ensure start_compress not called yet.
    // if (dstinfo->global_state != CSTATE_START)
    //     ERREXIT1(dstinfo, JERR_BAD_STATE, dstinfo->global_state);

    // Copy fundamental image dimensions
    // dstinfo->image_width = srcinfo->image_width;
    // dstinfo->image_height = srcinfo->image_height;
    // dstinfo->input_components = srcinfo->num_components;
    // dstinfo->in_color_space = srcinfo->jpeg_color_space;

    // Initialize all parameters to default values
    jpeg_set_defaults(dstinfo);
    // jpeg_set_defaults may choose wrong colorspace, eg YCbCr if input is RGB.
    // Fix it to get the right header markers for the image colorspace.
    // jpeg_set_colorspace(dstinfo, srcinfo->jpeg_color_space);

    // dstinfo->data_precision = srcinfo->data_precision;
    // dstinfo->CCIR601_sampling = srcinfo->CCIR601_sampling;

    // Copy the source's quantization tables.
    tblno = 0;
    while tblno < NUM_QUANT_TBLS as c_int {
        // if (srcinfo->quant_tbl_ptrs[tblno] != NULL) {
        //     qtblptr = & dstinfo->quant_tbl_ptrs[tblno];
        //     if (*qtblptr == NULL)
        //         *qtblptr = jpeg_alloc_quant_table((j_common_ptr) dstinfo);
        //     MEMCOPY((*qtblptr)->quantval,
        //             srcinfo->quant_tbl_ptrs[tblno]->quantval,
        //             SIZEOF((*qtblptr)->quantval));
        //     (*qtblptr)->sent_table = FALSE;
        // }
        tblno += 1;
    }

    // Copy the source's per-component info.
    // Note we assume jpeg_set_defaults has allocated the dest comp_info array.
    // dstinfo->num_components = srcinfo->num_components;
    // if (dstinfo->num_components < 1 || dstinfo->num_components > MAX_COMPONENTS)
    //     ERREXIT2(dstinfo, JERR_COMPONENT_COUNT, dstinfo->num_components,
    //              MAX_COMPONENTS);

    // ci = 0;
    // loop: ci < dstinfo->num_components; ci++, incomp++, outcomp++
    // {
    //     outcomp->component_id = incomp->component_id;
    //     outcomp->h_samp_factor = incomp->h_samp_factor;
    //     outcomp->v_samp_factor = incomp->v_samp_factor;
    //     outcomp->quant_tbl_no = incomp->quant_tbl_no;
    //     Make sure saved quantization table for component matches the qtable
    //     slot.  If not, the input file re-used this qtable slot.
    //     IJG encoder currently cannot duplicate this.
    //     tblno = outcomp->quant_tbl_no;
    //     if (tblno < 0 || tblno >= NUM_QUANT_TBLS ||
    //         srcinfo->quant_tbl_ptrs[tblno] == NULL)
    //         ERREXIT1(dstinfo, JERR_NO_QUANT_TABLE, tblno);
    //     slot_quant = srcinfo->quant_tbl_ptrs[tblno];
    //     c_quant = incomp->quant_table;
    //     if (c_quant != NULL) {
    //         coefi = 0;
    //         loop: coefi < DCTSIZE2
    //         {
    //             if (c_quant->quantval[coefi] != slot_quant->quantval[coefi])
    //                 ERREXIT1(dstinfo, JERR_MISMATCHED_QUANT_TABLE, tblno);
    //             coefi++;
    //         }
    //     }
    //     Note: we do not copy the source's Huffman table assignments;
    //     instead we rely on jpeg_set_colorspace to have made a suitable choice.
    // }
}


/*
 * Master selection of compression modules for transcoding.
 * This substitutes for jcinit.c's initialization of the full compressor.
 */

unsafe fn transencode_master_selection(cinfo: j_compress_ptr, coef_arrays: *mut jvirt_barray_ptr) {
    // Although we don't actually use input_components for transcoding,
    // jcmaster.c's initial_setup will complain if input_components is 0.
    // cinfo->input_components = 1;

    // Initialize master control (includes parameter checking/processing)
    jinit_c_master_control(cinfo, TRUE);

    // Entropy encoding: either Huffman or arithmetic coding.
    // if (cinfo->arith_code) {
    //     ERREXIT(cinfo, JERR_ARITH_NOTIMPL);
    // } else {
    if false {
        // placeholder for conditional
    } else {
        // if (cinfo->progressive_mode) {
        // #ifdef C_PROGRESSIVE_SUPPORTED
        jinit_phuff_encoder(cinfo);
        // #else
        //     ERREXIT(cinfo, JERR_NOT_COMPILED);
        // #endif
        // } else
        jinit_huff_encoder(cinfo);
    }

    // We need a special coefficient buffer controller.
    transencode_coef_controller(cinfo, coef_arrays);

    jinit_marker_writer(cinfo);

    // We can now tell the memory manager to allocate virtual arrays.
    // (*cinfo->mem->realize_virt_arrays) ((j_common_ptr) cinfo);

    // Write the datastream header (SOI) immediately.
    // Frame and scan headers are postponed till later.
    // This lets application insert special markers after the SOI.
    // (*cinfo->marker->write_file_header) (cinfo);
}


/*
 * The rest of this file is a special implementation of the coefficient
 * buffer controller.  This is similar to jccoefct.c, but it handles only
 * output from presupplied virtual arrays.  Furthermore, we generate any
 * dummy padding blocks on-the-fly rather than expecting them to be present
 * in the arrays.
 */

/* Private buffer controller object */

// typedef struct {
//   struct jpeg_c_coef_controller pub; /* public fields */
//
//   JDIMENSION iMCU_row_num;	/* iMCU row # within image */
//   JDIMENSION mcu_ctr;		/* counts MCUs processed in current row */
//   int MCU_vert_offset;		/* counts MCU rows within iMCU row */
//   int MCU_rows_per_iMCU_row;	/* number of such rows needed */
//
//   /* Virtual block array for each component. */
//   jvirt_barray_ptr * whole_image;
//
//   /* Workspace for constructing dummy blocks at right/bottom edges. */
//   JBLOCKROW dummy_buffer[C_MAX_BLOCKS_IN_MCU];
// } my_coef_controller;


unsafe fn start_iMCU_row(cinfo: j_compress_ptr) {
    // Reset within-iMCU-row counters for a new row
    let mut coef: my_coef_ptr = cinfo as my_coef_ptr;

    // In an interleaved scan, an MCU row is the same as an iMCU row.
    // In a noninterleaved scan, an iMCU row has v_samp_factor MCU rows.
    // But at the bottom of the image, process only what's left.
    // if (cinfo->comps_in_scan > 1) {
    if false {
        (*coef).MCU_rows_per_iMCU_row = 1;
    } else {
        // if (coef->iMCU_row_num < (cinfo->total_iMCU_rows-1))
        if true {
            // (*coef).MCU_rows_per_iMCU_row = cinfo->cur_comp_info[0]->v_samp_factor;
        } else {
            // (*coef).MCU_rows_per_iMCU_row = cinfo->cur_comp_info[0]->last_row_height;
        }
    }

    (*coef).mcu_ctr = 0;
    (*coef).MCU_vert_offset = 0;
}


/*
 * Initialize for a processing pass.
 */

unsafe fn start_pass_coef(cinfo: j_compress_ptr, pass_mode: J_BUF_MODE) {
    let coef: my_coef_ptr = cinfo as my_coef_ptr;

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

unsafe fn compress_output(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean {
    let coef: my_coef_ptr = cinfo as my_coef_ptr;
    let mut MCU_col_num: JDIMENSION;
    let last_MCU_col: JDIMENSION = 0; // cinfo->MCUs_per_row - 1
    let last_iMCU_row: JDIMENSION = 0; // cinfo->total_iMCU_rows - 1
    let mut blkn: c_int;
    let mut ci: c_int;
    let mut xindex: c_int;
    let mut yindex: c_int;
    let mut yoffset: c_int;
    let mut blockcnt: c_int;
    let mut start_col: JDIMENSION;
    let mut buffer: [JBLOCKARRAY; MAX_COMPS_IN_SCAN] = [core::ptr::null_mut(); MAX_COMPS_IN_SCAN];
    let mut MCU_buffer: [JBLOCKROW; C_MAX_BLOCKS_IN_MCU] = [core::ptr::null_mut(); C_MAX_BLOCKS_IN_MCU];
    let mut buffer_ptr: JBLOCKROW;
    let mut compptr: *mut jpeg_component_info;

    // Align the virtual buffers for the components used in this scan.
    ci = 0;
    while ci < 0 { // cinfo->comps_in_scan
        // compptr = cinfo->cur_comp_info[ci];
        // buffer[ci] = (*cinfo->mem->access_virt_barray)
        //   ((j_common_ptr) cinfo, coef->whole_image[compptr->component_index],
        //    coef->iMCU_row_num * compptr->v_samp_factor,
        //    (JDIMENSION) compptr->v_samp_factor, FALSE);
        ci += 1;
    }

    // Loop to process one whole iMCU row
    yoffset = (*coef).MCU_vert_offset;
    while yoffset < (*coef).MCU_rows_per_iMCU_row {
        MCU_col_num = (*coef).mcu_ctr;
        while MCU_col_num < 0 { // cinfo->MCUs_per_row
            // Construct list of pointers to DCT blocks belonging to this MCU
            blkn = 0;
            ci = 0;
            while ci < 0 { // cinfo->comps_in_scan
                // compptr = cinfo->cur_comp_info[ci];
                // start_col = MCU_col_num * compptr->MCU_width;
                // blockcnt = (MCU_col_num < last_MCU_col) ? compptr->MCU_width
                //                         : compptr->last_col_width;
                start_col = 0;
                blockcnt = 0;
                yindex = 0;
                while yindex < 0 { // compptr->MCU_height
                    if (*coef).iMCU_row_num < last_iMCU_row ||
                       yindex + yoffset < 0 { // compptr->last_row_height
                        // Fill in pointers to real blocks in this row
                        // buffer_ptr = buffer[ci][yindex+yoffset] + start_col;
                        xindex = 0;
                        while xindex < blockcnt {
                            // MCU_buffer[blkn++] = buffer_ptr++;
                            blkn += 1;
                            xindex += 1;
                        }
                    } else {
                        // At bottom of image, need a whole row of dummy blocks
                        xindex = 0;
                    }
                    // Fill in any dummy blocks needed in this row.
                    // Dummy blocks are filled the same way as in jccoefct.c:
                    // all zeroes in the AC entries, DC entries equal to previous
                    // block's DC value.  The init routine has already zeroed the
                    // AC entries, so we need only set the DC entries correctly.
                    while xindex < 0 { // compptr->MCU_width
                        // MCU_buffer[blkn] = coef->dummy_buffer[blkn];
                        // MCU_buffer[blkn][0][0] = MCU_buffer[blkn-1][0][0];
                        blkn += 1;
                        xindex += 1;
                    }
                    yindex += 1;
                }
                ci += 1;
            }
            // Try to write the MCU.
            // if (! (*cinfo->entropy->encode_mcu) (cinfo, MCU_buffer)) {
            if false {
                // Suspension forced; update state counters and exit
                (*coef).MCU_vert_offset = yoffset;
                (*coef).mcu_ctr = MCU_col_num;
                return FALSE;
            }
            MCU_col_num += 1;
        }
        // Completed an MCU row, but perhaps not an iMCU row
        (*coef).mcu_ctr = 0;
        yoffset += 1;
    }
    // Completed the iMCU row, advance counters for next one
    (*coef).iMCU_row_num += 1;
    start_iMCU_row(cinfo);
    return TRUE;
}


/*
 * Initialize coefficient buffer controller.
 *
 * Each passed coefficient array must be the right size for that
 * coefficient: width_in_blocks wide and height_in_blocks high,
 * with unitheight at least v_samp_factor.
 */

unsafe fn transencode_coef_controller(cinfo: j_compress_ptr, coef_arrays: *mut jvirt_barray_ptr) {
    let mut coef: my_coef_ptr;
    let mut buffer: JBLOCKROW;
    let mut i: c_int;

    // coef = (my_coef_ptr)
    //   (*cinfo->mem->alloc_small) ((j_common_ptr) cinfo, JPOOL_IMAGE,
    //                 SIZEOF(my_coef_controller));
    // cinfo->coef = (struct jpeg_c_coef_controller *) coef;
    // coef->pub.start_pass = start_pass_coef;
    // coef->pub.compress_data = compress_output;

    // Save pointer to virtual arrays
    // (*coef).whole_image = coef_arrays;

    // Allocate and pre-zero space for dummy DCT blocks.
    // buffer = (JBLOCKROW)
    //   (*cinfo->mem->alloc_large) ((j_common_ptr) cinfo, JPOOL_IMAGE,
    //                 C_MAX_BLOCKS_IN_MCU * SIZEOF(JBLOCK));
    // jzero_far((void FAR *) buffer, C_MAX_BLOCKS_IN_MCU * SIZEOF(JBLOCK));
    i = 0;
    while i < C_MAX_BLOCKS_IN_MCU as c_int {
        // (*coef).dummy_buffer[i] = buffer + i;
        i += 1;
    }
}
