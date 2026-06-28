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

// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::mem;

// Type definitions for JPEG library types
pub type JDIMENSION = c_int;
pub type JCOEF = c_int;
pub type JBLOCK = [JCOEF; 64];
pub type JBLOCKROW = *mut JBLOCK;
pub type JBLOCKARRAY = *mut *mut JBLOCK;
pub type JSAMPIMAGE = *mut *mut u8;
pub type jvirt_barray_ptr = *mut c_void;
pub type J_BUF_MODE = c_int;
pub type boolean = c_int;

pub const JBUF_PASS_THRU: c_int = 0;
pub const JBUF_SAVE_AND_PASS: c_int = 1;
pub const JBUF_CRANK_DEST: c_int = 2;

pub const DCTSIZE: usize = 8;
pub const C_MAX_BLOCKS_IN_MCU: usize = 10;
pub const MAX_COMPONENTS: usize = 10;
pub const MAX_COMPS_IN_SCAN: usize = 4;

// Stub types for JPEG library structures - defined in jinclude.h/jpeglib.h
#[repr(C)]
pub struct jpeg_c_coef_controller {
    /* public methods and fields */
}

#[repr(C)]
pub struct jpeg_component_info {
    /* component information */
}

#[repr(C)]
pub struct jpeg_compress_struct {
    /* JPEG compression state */
}

pub type j_compress_ptr = *mut jpeg_compress_struct;

/* Private buffer controller object */

#[repr(C)]
pub struct my_coef_controller {
    pub r#pub: jpeg_c_coef_controller, /* public fields */

    pub iMCU_row_num: JDIMENSION, /* iMCU row # within image */
    pub mcu_ctr: JDIMENSION,      /* counts MCUs processed in current row */
    pub MCU_vert_offset: c_int,   /* counts MCU rows within iMCU row */
    pub MCU_rows_per_iMCU_row: c_int, /* number of such rows needed */

    /* For single-pass compression, it's sufficient to buffer just one MCU
     * (although this may prove a bit slow in practice).  We allocate a
     * workspace of C_MAX_BLOCKS_IN_MCU coefficient blocks, and reuse it for each
     * MCU constructed and sent.  (On 80x86, the workspace is FAR even though
     * it's not really very big; this is to keep the module interfaces unchanged
     * when a large coefficient buffer is necessary.)
     * In multi-pass modes, this array points to the current MCU's blocks
     * within the virtual arrays.
     */
    pub MCU_buffer: [JBLOCKROW; C_MAX_BLOCKS_IN_MCU],

    /* In multi-pass modes, we need a virtual block array for each component. */
    pub whole_image: [jvirt_barray_ptr; MAX_COMPONENTS],
}

pub type my_coef_ptr = *mut my_coef_controller;


/* Forward declarations */
fn compress_data(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean;
#[cfg(feature = "FULL_COEF_BUFFER_SUPPORTED")]
fn compress_first_pass(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean;
#[cfg(feature = "FULL_COEF_BUFFER_SUPPORTED")]
fn compress_output(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean;


fn start_iMCU_row(cinfo: j_compress_ptr)
/* Reset within-iMCU-row counters for a new row */
{
    unsafe {
        let coef = (*cinfo).coef as *mut my_coef_controller;

        /* In an interleaved scan, an MCU row is the same as an iMCU row.
         * In a noninterleaved scan, an iMCU row has v_samp_factor MCU rows.
         * But at the bottom of the image, process only what's left.
         */
        if (*cinfo).comps_in_scan > 1 {
            (*coef).MCU_rows_per_iMCU_row = 1;
        } else {
            if (*coef).iMCU_row_num < ((*cinfo).total_iMCU_rows - 1) {
                (*coef).MCU_rows_per_iMCU_row =
                    (*(*(*cinfo).cur_comp_info.offset(0))).v_samp_factor;
            } else {
                (*coef).MCU_rows_per_iMCU_row =
                    (*(*(*cinfo).cur_comp_info.offset(0))).last_row_height;
            }
        }

        (*coef).mcu_ctr = 0;
        (*coef).MCU_vert_offset = 0;
    }
}


/*
 * Initialize for a processing pass.
 */

fn start_pass_coef(cinfo: j_compress_ptr, pass_mode: J_BUF_MODE) {
    unsafe {
        let coef = (*cinfo).coef as *mut my_coef_controller;

        (*coef).iMCU_row_num = 0;
        start_iMCU_row(cinfo);

        match pass_mode {
            JBUF_PASS_THRU => {
                if (*coef).whole_image[0] != core::ptr::null_mut() {
                    ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
                }
                (*(*coef).r#pub).compress_data = compress_data;
            }
            #[cfg(feature = "FULL_COEF_BUFFER_SUPPORTED")]
            JBUF_SAVE_AND_PASS => {
                if (*coef).whole_image[0] == core::ptr::null_mut() {
                    ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
                }
                (*(*coef).r#pub).compress_data = compress_first_pass;
            }
            #[cfg(feature = "FULL_COEF_BUFFER_SUPPORTED")]
            JBUF_CRANK_DEST => {
                if (*coef).whole_image[0] == core::ptr::null_mut() {
                    ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
                }
                (*(*coef).r#pub).compress_data = compress_output;
            }
            _ => {
                ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
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

fn compress_data(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean {
    unsafe {
        let coef = (*cinfo).coef as *mut my_coef_controller;
        let mut MCU_col_num: JDIMENSION; /* index of current MCU within row */
        let last_MCU_col = (*cinfo).MCUs_per_row - 1;
        let last_iMCU_row = (*cinfo).total_iMCU_rows - 1;
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
                    compptr = (*(*cinfo).cur_comp_info.offset(ci as isize));
                    blockcnt = if MCU_col_num < last_MCU_col {
                        (*compptr).MCU_width
                    } else {
                        (*compptr).last_col_width
                    };
                    xpos = MCU_col_num * (*compptr).MCU_sample_width;
                    ypos = (yoffset * DCTSIZE as c_int) as JDIMENSION; /* ypos == (yoffset+yindex) * DCTSIZE */
                    yindex = 0;
                    while yindex < (*compptr).MCU_height {
                        if (*coef).iMCU_row_num < last_iMCU_row
                            || yoffset + yindex < (*compptr).last_row_height
                        {
                            ((*(*cinfo).fdct).forward_DCT)(
                                cinfo,
                                compptr,
                                *input_buf.offset(ci as isize),
                                *(*coef).MCU_buffer.as_mut_ptr().offset(blkn as isize),
                                ypos,
                                xpos,
                                blockcnt as JDIMENSION,
                            );
                            if blockcnt < (*compptr).MCU_width {
                                /* Create some dummy blocks at the right edge of the image. */
                                jzero_far(
                                    (*(*coef).MCU_buffer.as_mut_ptr()
                                        .offset((blkn + blockcnt) as isize)) as *mut c_void,
                                    (((*compptr).MCU_width - blockcnt) * mem::size_of::<JBLOCK>())
                                        as usize,
                                );
                                bi = blockcnt;
                                while bi < (*compptr).MCU_width {
                                    *(*(*(*coef).MCU_buffer.as_mut_ptr().offset((blkn + bi) as isize)))
                                        .as_mut_ptr()
                                        .offset(0) = *(*(*(*coef)
                                        .MCU_buffer
                                        .as_mut_ptr()
                                        .offset((blkn + bi - 1) as isize)))
                                    .as_mut_ptr()
                                    .offset(0);
                                    bi += 1;
                                }
                            }
                        } else {
                            /* Create a row of dummy blocks at the bottom of the image. */
                            jzero_far(
                                *(*coef).MCU_buffer.as_mut_ptr().offset(blkn as isize) as *mut c_void,
                                ((*compptr).MCU_width * mem::size_of::<JBLOCK>()) as usize,
                            );
                            bi = 0;
                            while bi < (*compptr).MCU_width {
                                *(*(*(*coef)
                                    .MCU_buffer
                                    .as_mut_ptr()
                                    .offset((blkn + bi) as isize)))
                                    .as_mut_ptr()
                                    .offset(0) = *(*(*(*coef)
                                    .MCU_buffer
                                    .as_mut_ptr()
                                    .offset((blkn - 1) as isize)))
                                .as_mut_ptr()
                                .offset(0);
                                bi += 1;
                            }
                        }
                        blkn += (*compptr).MCU_width;
                        ypos = (ypos as c_int + DCTSIZE as c_int) as JDIMENSION;
                        yindex += 1;
                    }
                    ci += 1;
                }
                /* Try to write the MCU.  In event of a suspension failure, we will
                 * re-DCT the MCU on restart (a bit inefficient, could be fixed...)
                 */
                if ((*(*cinfo).entropy).encode_mcu)(cinfo, (*coef).MCU_buffer.as_mut_ptr()) == 0 {
                    /* Suspension forced; update state counters and exit */
                    (*coef).MCU_vert_offset = yoffset;
                    (*coef).mcu_ctr = MCU_col_num;
                    return 0; /* FALSE */
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
        return 1; /* TRUE */
    }
}


#[cfg(feature = "FULL_COEF_BUFFER_SUPPORTED")]
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

#[cfg(feature = "FULL_COEF_BUFFER_SUPPORTED")]
fn compress_first_pass(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean {
    unsafe {
        let coef = (*cinfo).coef as *mut my_coef_controller;
        let last_iMCU_row = (*cinfo).total_iMCU_rows - 1;
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
            buffer = ((*(*cinfo).mem).access_virt_barray)(
                cinfo as *mut c_void,
                (*coef).whole_image[ci as usize],
                (*coef).iMCU_row_num * (*compptr).v_samp_factor,
                (*compptr).v_samp_factor as JDIMENSION,
                1,
            ) as JBLOCKARRAY;
            /* Count non-dummy DCT block rows in this iMCU row. */
            if (*coef).iMCU_row_num < last_iMCU_row {
                block_rows = (*compptr).v_samp_factor;
            } else {
                /* NB: can't use last_row_height here, since may not be set! */
                block_rows =
                    ((*compptr).height_in_blocks % (*compptr).v_samp_factor) as c_int;
                if block_rows == 0 {
                    block_rows = (*compptr).v_samp_factor;
                }
            }
            blocks_across = (*compptr).width_in_blocks as JDIMENSION;
            h_samp_factor = (*compptr).h_samp_factor;
            /* Count number of dummy blocks to be added at the right margin. */
            ndummy = (blocks_across % h_samp_factor as JDIMENSION) as c_int;
            if ndummy > 0 {
                ndummy = h_samp_factor - ndummy;
            }
            /* Perform DCT for all non-dummy blocks in this iMCU row.  Each call
             * on forward_DCT processes a complete horizontal row of DCT blocks.
             */
            block_row = 0;
            while block_row < block_rows {
                thisblockrow = *buffer.offset(block_row as isize);
                ((*(*cinfo).fdct).forward_DCT)(
                    cinfo,
                    compptr,
                    *input_buf.offset(ci as isize),
                    thisblockrow,
                    (block_row as JDIMENSION * DCTSIZE as JDIMENSION),
                    0,
                    blocks_across,
                );
                if ndummy > 0 {
                    /* Create dummy blocks at the right edge of the image. */
                    thisblockrow = thisblockrow.offset(blocks_across as isize); /* => first dummy block */
                    jzero_far(
                        thisblockrow as *mut c_void,
                        (ndummy as usize * mem::size_of::<JBLOCK>()),
                    );
                    lastDC = (*thisblockrow.offset(-1))[0];
                    bi = 0;
                    while bi < ndummy {
                        (*thisblockrow.offset(bi as isize))[0] = lastDC;
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
                blocks_across = (blocks_across as c_int + ndummy) as JDIMENSION;
                MCUs_across = blocks_across / h_samp_factor as JDIMENSION;
                block_row = block_rows;
                while block_row < (*compptr).v_samp_factor {
                    thisblockrow = *buffer.offset(block_row as isize);
                    lastblockrow = *buffer.offset((block_row - 1) as isize);
                    jzero_far(
                        thisblockrow as *mut c_void,
                        (blocks_across as usize * mem::size_of::<JBLOCK>()),
                    );
                    MCUindex = 0;
                    while MCUindex < MCUs_across {
                        lastDC = (*lastblockrow.offset((h_samp_factor - 1) as isize))[0];
                        bi = 0;
                        while bi < h_samp_factor {
                            (*thisblockrow.offset(bi as isize))[0] = lastDC;
                            bi += 1;
                        }
                        thisblockrow = thisblockrow.offset(h_samp_factor as isize); /* advance to next MCU in row */
                        lastblockrow = lastblockrow.offset(h_samp_factor as isize);
                        MCUindex += 1;
                    }
                    block_row += 1;
                }
            }
            ci += 1;
            compptr = compptr.offset(1);
        }
        /* NB: compress_output will increment iMCU_row_num if successful.
         * A suspension return will result in redoing all the work above next time.
         */

        /* Emit data to the entropy encoder, sharing code with subsequent passes */
        return compress_output(cinfo, input_buf);
    }
}


#[cfg(feature = "FULL_COEF_BUFFER_SUPPORTED")]
/*
 * Process some data in subsequent passes of a multi-pass case.
 * We process the equivalent of one fully interleaved MCU row ("iMCU" row)
 * per call, ie, v_samp_factor block rows for each component in the scan.
 * The data is obtained from the virtual arrays and fed to the entropy coder.
 * Returns TRUE if the iMCU row is completed, FALSE if suspended.
 *
 * NB: input_buf is ignored; it is likely to be a NULL pointer.
 */

#[cfg(feature = "FULL_COEF_BUFFER_SUPPORTED")]
fn compress_output(cinfo: j_compress_ptr, input_buf: JSAMPIMAGE) -> boolean {
    unsafe {
        let coef = (*cinfo).coef as *mut my_coef_controller;
        let mut MCU_col_num: JDIMENSION; /* index of current MCU within row */
        let mut blkn: c_int;
        let mut ci: c_int;
        let mut xindex: c_int;
        let mut yindex: c_int;
        let mut yoffset: c_int;
        let mut start_col: JDIMENSION;
        let mut buffer: [JBLOCKARRAY; MAX_COMPS_IN_SCAN];
        let mut buffer_ptr: JBLOCKROW;
        let mut compptr: *mut jpeg_component_info;

        /* Align the virtual buffers for the components used in this scan.
         * NB: during first pass, this is safe only because the buffers will
         * already be aligned properly, so jmemmgr.c won't need to do any I/O.
         */
        ci = 0;
        while ci < (*cinfo).comps_in_scan {
            compptr = (*(*cinfo).cur_comp_info.offset(ci as isize));
            buffer[ci as usize] = ((*(*cinfo).mem).access_virt_barray)(
                cinfo as *mut c_void,
                (*coef).whole_image[(*compptr).component_index as usize],
                (*coef).iMCU_row_num * (*compptr).v_samp_factor,
                (*compptr).v_samp_factor as JDIMENSION,
                0,
            ) as JBLOCKARRAY;
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
                    compptr = (*(*cinfo).cur_comp_info.offset(ci as isize));
                    start_col = MCU_col_num * (*compptr).MCU_width as JDIMENSION;
                    yindex = 0;
                    while yindex < (*compptr).MCU_height {
                        buffer_ptr = (*buffer[ci as usize].offset((yindex + yoffset) as isize))
                            .offset(start_col as isize);
                        xindex = 0;
                        while xindex < (*compptr).MCU_width {
                            let mcu_buf = (*coef).MCU_buffer.as_mut_ptr().offset(blkn as isize);
                            *mcu_buf = buffer_ptr;
                            blkn += 1;
                            buffer_ptr = buffer_ptr.offset(1);
                            xindex += 1;
                        }
                        yindex += 1;
                    }
                    ci += 1;
                }
                /* Try to write the MCU. */
                if ((*(*cinfo).entropy).encode_mcu)(cinfo, (*coef).MCU_buffer.as_mut_ptr()) == 0 {
                    /* Suspension forced; update state counters and exit */
                    (*coef).MCU_vert_offset = yoffset;
                    (*coef).mcu_ctr = MCU_col_num;
                    return 0; /* FALSE */
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
        return 1; /* TRUE */
    }
}

/* #endif FULL_COEF_BUFFER_SUPPORTED */


/*
 * Initialize coefficient buffer controller.
 */

pub fn jinit_c_coef_controller(cinfo: j_compress_ptr, need_full_buffer: boolean) {
    unsafe {
        let coef: *mut my_coef_controller;

        coef = ((*(*cinfo).mem).alloc_small)(
            cinfo as *mut c_void,
            0, /* JPOOL_IMAGE */
            mem::size_of::<my_coef_controller>(),
        ) as *mut my_coef_controller;
        (*cinfo).coef = coef as *mut c_void;
        ((*(*coef).r#pub)).start_pass = Some(start_pass_coef);

        /* Create the coefficient buffer. */
        if need_full_buffer != 0 {
            #[cfg(feature = "FULL_COEF_BUFFER_SUPPORTED")]
            {
                /* Allocate a full-image virtual array for each component, */
                /* padded to a multiple of samp_factor DCT blocks in each direction. */
                let mut ci: c_int;
                let mut compptr: *mut jpeg_component_info;

                ci = 0;
                compptr = (*cinfo).comp_info;
                while ci < (*cinfo).num_components {
                    (*coef).whole_image[ci as usize] = ((*(*cinfo).mem).request_virt_barray)(
                        cinfo as *mut c_void,
                        0,  /* JPOOL_IMAGE */
                        0,  /* FALSE */
                        jround_up(
                            (*compptr).width_in_blocks as c_int as i64,
                            (*compptr).h_samp_factor as i64,
                        ) as JDIMENSION,
                        jround_up(
                            (*compptr).height_in_blocks as c_int as i64,
                            (*compptr).v_samp_factor as i64,
                        ) as JDIMENSION,
                        (*compptr).v_samp_factor as JDIMENSION,
                    );
                    ci += 1;
                    compptr = compptr.offset(1);
                }
            }
            #[cfg(not(feature = "FULL_COEF_BUFFER_SUPPORTED"))]
            {
                ERREXIT(cinfo, 0); /* JERR_BAD_BUFFER_MODE */
            }
        } else {
            /* We only need a single-MCU buffer. */
            let mut buffer: JBLOCKROW;
            let mut i: c_int;

            buffer = ((*(*cinfo).mem).alloc_large)(
                cinfo as *mut c_void,
                0, /* JPOOL_IMAGE */
                (C_MAX_BLOCKS_IN_MCU * mem::size_of::<JBLOCK>()),
            ) as JBLOCKROW;
            i = 0;
            while i < C_MAX_BLOCKS_IN_MCU as c_int {
                let buf_ptr = (*coef).MCU_buffer.as_mut_ptr().offset(i as isize);
                *buf_ptr = buffer.offset(i as isize);
                i += 1;
            }
            (*coef).whole_image[0] = core::ptr::null_mut(); /* flag for no virtual arrays */
        }
    }
}


/* Stub external functions - these would be defined in the JPEG library */

#[inline]
unsafe fn jzero_far(ptr: *mut c_void, size: usize) {
    core::ptr::write_bytes(ptr as *mut u8, 0, size);
}

#[inline]
fn jround_up(a: i64, b: i64) -> i64 {
    ((a + b - 1) / b) * b
}

#[inline]
unsafe fn ERREXIT(cinfo: j_compress_ptr, code: i32) {
    /* Error exit - stub implementation */
    let _ = (cinfo, code);
}

const JERR_BAD_BUFFER_MODE: i32 = 1;
