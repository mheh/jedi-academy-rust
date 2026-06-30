#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_mut,
    unused_assignments
)]

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

use crate::code::server::exe_headers_h::*;
use crate::code::jpeg_6::jinclude_h::*;
use crate::code::jpeg_6::jpeglib_h::*;

/* Forward declarations */
// LOCAL void transencode_master_selection
//     JPP((j_compress_ptr cinfo, jvirt_barray_ptr * coef_arrays));
// LOCAL void transencode_coef_controller
//     JPP((j_compress_ptr cinfo, jvirt_barray_ptr * coef_arrays));

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

pub unsafe fn jpeg_write_coefficients(
    cinfo: j_compress_ptr,
    coef_arrays: *mut jvirt_barray_ptr,
) {
    if (*cinfo).global_state != CSTATE_START {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Mark all tables to be written */
    jpeg_suppress_tables(cinfo, FALSE);
    /* (Re)initialize error mgr and destination modules */
    ((*(*cinfo).err).reset_error_mgr.unwrap())(cinfo as j_common_ptr);
    ((*(*cinfo).dest).init_destination.unwrap())(cinfo);
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

pub unsafe fn jpeg_copy_critical_parameters(
    srcinfo: j_decompress_ptr,
    dstinfo: j_compress_ptr,
) {
    let mut qtblptr: *mut *mut JQUANT_TBL = core::ptr::null_mut();
    let mut incomp: *mut jpeg_component_info = core::ptr::null_mut();
    let mut outcomp: *mut jpeg_component_info = core::ptr::null_mut();
    let mut c_quant: *mut JQUANT_TBL = core::ptr::null_mut();
    let mut slot_quant: *mut JQUANT_TBL = core::ptr::null_mut();
    let mut tblno: core::ffi::c_int = 0;
    let mut ci: core::ffi::c_int = 0;
    let mut coefi: core::ffi::c_int = 0;

    /* Safety check to ensure start_compress not called yet. */
    if (*dstinfo).global_state != CSTATE_START {
        ERREXIT1!(dstinfo, JERR_BAD_STATE, (*dstinfo).global_state);
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
    while tblno < NUM_QUANT_TBLS as core::ffi::c_int {
        if !(*srcinfo).quant_tbl_ptrs[tblno as usize].is_null() {
            qtblptr = core::ptr::addr_of_mut!((*dstinfo).quant_tbl_ptrs[tblno as usize]);
            if (*qtblptr).is_null() {
                *qtblptr = jpeg_alloc_quant_table(dstinfo as j_common_ptr);
            }
            MEMCOPY!(
                (*(*qtblptr)).quantval.as_mut_ptr(),
                (*(*srcinfo).quant_tbl_ptrs[tblno as usize]).quantval.as_ptr(),
                SIZEOF!((*(*qtblptr)).quantval)
            );
            (**qtblptr).sent_table = FALSE;
        }
        tblno += 1;
    }
    /* Copy the source's per-component info.
     * Note we assume jpeg_set_defaults has allocated the dest comp_info array.
     */
    (*dstinfo).num_components = (*srcinfo).num_components;
    if (*dstinfo).num_components < 1
        || (*dstinfo).num_components > MAX_COMPONENTS as core::ffi::c_int
    {
        ERREXIT2!(
            dstinfo,
            JERR_COMPONENT_COUNT,
            (*dstinfo).num_components,
            MAX_COMPONENTS
        );
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
            || tblno >= NUM_QUANT_TBLS as core::ffi::c_int
            || (*srcinfo).quant_tbl_ptrs[tblno as usize].is_null()
        {
            ERREXIT1!(dstinfo, JERR_NO_QUANT_TABLE, tblno);
        }
        slot_quant = (*srcinfo).quant_tbl_ptrs[tblno as usize];
        c_quant = (*incomp).quant_table;
        if !c_quant.is_null() {
            coefi = 0;
            while coefi < DCTSIZE2 as core::ffi::c_int {
                if (*c_quant).quantval[coefi as usize]
                    != (*slot_quant).quantval[coefi as usize]
                {
                    ERREXIT1!(dstinfo, JERR_MISMATCHED_QUANT_TABLE, tblno);
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
    jinit_c_master_control(cinfo, TRUE); /* transcode only */

    /* Entropy encoding: either Huffman or arithmetic coding. */
    if (*cinfo).arith_code != 0 {
        ERREXIT!(cinfo, JERR_ARITH_NOTIMPL);
    } else {
        if (*cinfo).progressive_mode != 0 {
            #[cfg(feature = "c_progressive_supported")]
            jinit_phuff_encoder(cinfo);
            #[cfg(not(feature = "c_progressive_supported"))]
            ERREXIT!(cinfo, JERR_NOT_COMPILED);
        } else {
            jinit_huff_encoder(cinfo);
        }
    }

    /* We need a special coefficient buffer controller. */
    transencode_coef_controller(cinfo, coef_arrays);

    jinit_marker_writer(cinfo);

    /* We can now tell the memory manager to allocate virtual arrays. */
    ((*(*cinfo).mem).realize_virt_arrays.unwrap())(cinfo as j_common_ptr);

    /* Write the datastream header (SOI) immediately.
     * Frame and scan headers are postponed till later.
     * This lets application insert special markers after the SOI.
     */
    ((*(*cinfo).marker).write_file_header.unwrap())(cinfo);
}

/*
 * The rest of this file is a special implementation of the coefficient
 * buffer controller.  This is similar to jccoefct.c, but it handles only
 * output from presupplied virtual arrays.  Furthermore, we generate any
 * dummy padding blocks on-the-fly rather than expecting them to be present
 * in the arrays.
 */

/* Private buffer controller object */

#[repr(C)]
pub struct my_coef_controller {
    pub r#pub: jpeg_c_coef_controller, /* public fields */

    pub iMCU_row_num: JDIMENSION,               /* iMCU row # within image */
    pub mcu_ctr: JDIMENSION,                    /* counts MCUs processed in current row */
    pub MCU_vert_offset: core::ffi::c_int,      /* counts MCU rows within iMCU row */
    pub MCU_rows_per_iMCU_row: core::ffi::c_int, /* number of such rows needed */

    /* Virtual block array for each component. */
    pub whole_image: *mut jvirt_barray_ptr,

    /* Workspace for constructing dummy blocks at right/bottom edges. */
    pub dummy_buffer: [JBLOCKROW; C_MAX_BLOCKS_IN_MCU as usize],
}

pub type my_coef_ptr = *mut my_coef_controller;

unsafe fn start_iMCU_row(cinfo: j_compress_ptr)
/* Reset within-iMCU-row counters for a new row */
{
    let coef: my_coef_ptr = (*cinfo).coef as my_coef_ptr;

    /* In an interleaved scan, an MCU row is the same as an iMCU row.
     * In a noninterleaved scan, an iMCU row has v_samp_factor MCU rows.
     * But at the bottom of the image, process only what's left.
     */
    if (*cinfo).comps_in_scan > 1 {
        (*coef).MCU_rows_per_iMCU_row = 1;
    } else {
        if (*coef).iMCU_row_num < (*cinfo).total_iMCU_rows.wrapping_sub(1) {
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

unsafe fn start_pass_coef(cinfo: j_compress_ptr, pass_mode: J_BUF_MODE) {
    let coef: my_coef_ptr = (*cinfo).coef as my_coef_ptr;

    if pass_mode != JBUF_CRANK_DEST {
        ERREXIT!(cinfo, JERR_BAD_BUFFER_MODE);
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
    let coef: my_coef_ptr = (*cinfo).coef as my_coef_ptr;
    let mut MCU_col_num: JDIMENSION; /* index of current MCU within row */
    let last_MCU_col: JDIMENSION = (*cinfo).MCUs_per_row.wrapping_sub(1);
    let last_iMCU_row: JDIMENSION = (*cinfo).total_iMCU_rows.wrapping_sub(1);
    let mut blkn: core::ffi::c_int;
    let mut ci: core::ffi::c_int;
    let mut xindex: core::ffi::c_int;
    let mut yindex: core::ffi::c_int;
    let mut yoffset: core::ffi::c_int;
    let mut blockcnt: core::ffi::c_int;
    let mut start_col: JDIMENSION;
    let mut buffer: [JBLOCKARRAY; MAX_COMPS_IN_SCAN as usize] =
        [core::ptr::null_mut(); MAX_COMPS_IN_SCAN as usize];
    let mut MCU_buffer: [JBLOCKROW; C_MAX_BLOCKS_IN_MCU as usize] =
        [core::ptr::null_mut(); C_MAX_BLOCKS_IN_MCU as usize];
    let mut buffer_ptr: JBLOCKROW = core::ptr::null_mut();
    let mut compptr: *mut jpeg_component_info;

    /* Align the virtual buffers for the components used in this scan. */
    ci = 0;
    while ci < (*cinfo).comps_in_scan {
        compptr = (*cinfo).cur_comp_info[ci as usize];
        buffer[ci as usize] = ((*(*cinfo).mem).access_virt_barray.unwrap())(
            cinfo as j_common_ptr,
            *(*coef).whole_image.add((*compptr).component_index as usize),
            (*coef)
                .iMCU_row_num
                .wrapping_mul((*compptr).v_samp_factor as JDIMENSION),
            (*compptr).v_samp_factor as JDIMENSION,
            FALSE,
        );
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
                start_col =
                    MCU_col_num.wrapping_mul((*compptr).MCU_width as JDIMENSION);
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
                        buffer_ptr = (*buffer[ci as usize]
                            .add((yindex + yoffset) as usize))
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
                        MCU_buffer[blkn as usize] =
                            (*coef).dummy_buffer[blkn as usize];
                        (*MCU_buffer[blkn as usize])[0] =
                            (*MCU_buffer[(blkn - 1) as usize])[0];
                        blkn += 1;
                        xindex += 1;
                    }
                    yindex += 1;
                }
                ci += 1;
            }
            /* Try to write the MCU. */
            if ((*(*cinfo).entropy).encode_mcu.unwrap())(
                cinfo,
                MCU_buffer.as_mut_ptr(),
            ) == FALSE
            {
                /* Suspension forced; update state counters and exit */
                (*coef).MCU_vert_offset = yoffset;
                (*coef).mcu_ctr = MCU_col_num;
                return FALSE;
            }
            MCU_col_num = MCU_col_num.wrapping_add(1);
        }
        /* Completed an MCU row, but perhaps not an iMCU row */
        (*coef).mcu_ctr = 0;
        yoffset += 1;
    }
    /* Completed the iMCU row, advance counters for next one */
    (*coef).iMCU_row_num = (*coef).iMCU_row_num.wrapping_add(1);
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

unsafe fn transencode_coef_controller(
    cinfo: j_compress_ptr,
    coef_arrays: *mut jvirt_barray_ptr,
) {
    let mut coef: my_coef_ptr;
    let mut buffer: JBLOCKROW;
    let mut i: core::ffi::c_int;

    coef = ((*(*cinfo).mem).alloc_small.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        core::mem::size_of::<my_coef_controller>(),
    ) as my_coef_ptr;
    (*cinfo).coef = coef as *mut jpeg_c_coef_controller;
    (*coef).r#pub.start_pass = Some(start_pass_coef);
    (*coef).r#pub.compress_data = Some(compress_output);

    /* Save pointer to virtual arrays */
    (*coef).whole_image = coef_arrays;

    /* Allocate and pre-zero space for dummy DCT blocks. */
    buffer = ((*(*cinfo).mem).alloc_large.unwrap())(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        (C_MAX_BLOCKS_IN_MCU as usize)
            .wrapping_mul(core::mem::size_of::<JBLOCK>()),
    ) as JBLOCKROW;
    jzero_far(
        buffer as *mut core::ffi::c_void,
        (C_MAX_BLOCKS_IN_MCU as usize)
            .wrapping_mul(core::mem::size_of::<JBLOCK>()),
    );
    i = 0;
    while i < C_MAX_BLOCKS_IN_MCU as core::ffi::c_int {
        (*coef).dummy_buffer[i as usize] = buffer.add(i as usize);
        i += 1;
    }
}
