/*
 * jdmainct.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains the main buffer controller for decompression.
 * The main buffer lies between the JPEG decompressor proper and the
 * post-processor; it holds downsampled data in the JPEG colorspace.
 *
 * Note that this code is bypassed in raw-data mode, since the application
 * supplies the equivalent of the main buffer in that case.
 */

#![allow(non_snake_case, non_camel_case_types, dead_code, unused_mut,
         unused_variables, unused_imports, non_upper_case_globals,
         unused_unsafe, clippy::all)]

// leave this as first line for PCH reasons...
//
use crate::code::server::exe_headers_h::*;

// #define JPEG_INTERNALS
use crate::code::jpeg_6::jinclude_h::*;
use crate::code::jpeg_6::jpeglib_h::*;
use crate::code::jpeg_6::jpegint_h::*;
use crate::code::jpeg_6::jmorecfg_h::*;
use crate::code::jpeg_6::jerror_h::*;

use core::ffi::c_int;
use core::ptr::addr_of_mut;

/*
 * In the current system design, the main buffer need never be a full-image
 * buffer; any full-height buffers will be found inside the coefficient or
 * postprocessing controllers.  Nonetheless, the main controller is not
 * trivial.  Its responsibility is to provide context rows for upsampling/
 * rescaling, and doing this in an efficient fashion is a bit tricky.
 *
 * Postprocessor input data is counted in "row groups".  A row group
 * is defined to be (v_samp_factor * DCT_scaled_size / min_DCT_scaled_size)
 * sample rows of each component.  (We require DCT_scaled_size values to be
 * chosen such that these numbers are integers.  In practice DCT_scaled_size
 * values will likely be powers of two, so we actually have the stronger
 * condition that DCT_scaled_size / min_DCT_scaled_size is an integer.)
 * Upsampling will typically produce max_v_samp_factor pixel rows from each
 * row group (times any additional scale factor that the upsampler is
 * applying).
 *
 * The coefficient controller will deliver data to us one iMCU row at a time;
 * each iMCU row contains v_samp_factor * DCT_scaled_size sample rows, or
 * exactly min_DCT_scaled_size row groups.  (This amount of data corresponds
 * to one row of MCUs when the image is fully interleaved.)  Note that the
 * number of sample rows varies across components, but the number of row
 * groups does not.  Some garbage sample rows may be included in the last iMCU
 * row at the bottom of the image.
 *
 * Depending on the vertical scaling algorithm used, the upsampler may need
 * access to the sample row(s) above and below its current input row group.
 * The upsampler is required to set need_context_rows TRUE at global selection
 * time if so.  When need_context_rows is FALSE, this controller can simply
 * obtain one iMCU row at a time from the coefficient controller and dole it
 * out as row groups to the postprocessor.
 *
 * When need_context_rows is TRUE, this controller guarantees that the buffer
 * passed to postprocessing contains at least one row group's worth of samples
 * above and below the row group(s) being processed.  Note that the context
 * rows "above" the first passed row group appear at negative row offsets in
 * the passed buffer.  At the top and bottom of the image, the required
 * context rows are manufactured by duplicating the first or last real sample
 * row; this avoids having special cases in the upsampling inner loops.
 *
 * The amount of context is fixed at one row group just because that's a
 * convenient number for this controller to work with.  The existing
 * upsamplers really only need one sample row of context.  An upsampler
 * supporting arbitrary output rescaling might wish for more than one row
 * group of context when shrinking the image; tough, we don't handle that.
 * (This is justified by the assumption that downsizing will be handled mostly
 * by adjusting the DCT_scaled_size values, so that the actual scale factor at
 * the upsample step needn't be much less than one.)
 *
 * To provide the desired context, we have to retain the last two row groups
 * of one iMCU row while reading in the next iMCU row.  (The last row group
 * can't be processed until we have another row group for its below-context,
 * and so we have to save the next-to-last group too for its above-context.)
 * We could do this most simply by copying data around in our buffer, but
 * that'd be very slow.  We can avoid copying any data by creating a rather
 * strange pointer structure.  Here's how it works.  We allocate a workspace
 * consisting of M+2 row groups (where M = min_DCT_scaled_size is the number
 * of row groups per iMCU row).  We create two sets of redundant pointers to
 * the workspace.  Labeling the physical row groups 0 to M+1, the synthesized
 * pointer lists look like this:
 *                   M+1                          M-1
 * master pointer --> 0         master pointer --> 0
 *                    1                            1
 *                   ...                          ...
 *                   M-3                          M-3
 *                   M-2                           M
 *                   M-1                          M+1
 *                    M                           M-2
 *                   M+1                          M-1
 *                    0                            0
 * We read alternate iMCU rows using each master pointer; thus the last two
 * row groups of the previous iMCU row remain un-overwritten in the workspace.
 * The pointer lists are set up so that the required context rows appear to
 * be adjacent to the proper places when we pass the pointer lists to the
 * upsampler.
 *
 * The above pictures describe the normal state of the pointer lists.
 * At top and bottom of the image, we diddle the pointer lists to duplicate
 * the first or last sample row as necessary (this is cheaper than copying
 * sample rows around).
 *
 * This scheme breaks down if M < 2, ie, min_DCT_scaled_size is 1.  In that
 * situation each iMCU row provides only one row group so the buffering logic
 * must be different (eg, we must read two iMCU rows before we can emit the
 * first row group).  For now, we simply do not support providing context
 * rows when min_DCT_scaled_size is 1.  That combination seems unlikely to
 * be worth providing --- if someone wants a 1/8th-size preview, they probably
 * want it quick and dirty, so a context-free upsampler is sufficient.
 */


/* Private buffer controller object */

#[repr(C)]
struct my_main_controller {
    pub pub_: jpeg_d_main_controller, /* public fields */

    /* Pointer to allocated workspace (M or M+2 row groups). */
    buffer: [JSAMPARRAY; MAX_COMPONENTS],

    buffer_full: boolean,       /* Have we gotten an iMCU row from decoder? */
    rowgroup_ctr: JDIMENSION,   /* counts row groups output to postprocessor */

    /* Remaining fields are only used in the context case. */

    /* These are the master pointers to the funny-order pointer lists. */
    xbuffer: [JSAMPIMAGE; 2],   /* pointers to weird pointer lists */

    whichptr: c_int,            /* indicates which pointer set is now in use */
    context_state: c_int,       /* process_data state machine status */
    rowgroups_avail: JDIMENSION, /* row groups available to postprocessor */
    iMCU_row_ctr: JDIMENSION,  /* counts iMCU rows to detect image top/bot */
}

type my_main_ptr = *mut my_main_controller;

/* context_state values: */
const CTX_PREPARE_FOR_IMCU: c_int = 0; /* need to prepare for MCU row */
const CTX_PROCESS_IMCU: c_int = 1;     /* feeding iMCU to postprocessor */
const CTX_POSTPONED_ROW: c_int = 2;    /* feeding postponed row group */


/* Forward declarations */
/* (process_data_simple_main, process_data_context_main,
 *  process_data_crank_post -- no forward declarations needed in Rust) */


unsafe fn alloc_funny_pointers(cinfo: j_decompress_ptr)
/* Allocate space for the funny pointer lists.
 * This is done only once, not once per pass.
 */
{
    // bk001204 - no use main
    let jmain: my_main_ptr = (*cinfo).main as my_main_ptr;
    let mut ci: c_int;
    let mut rgroup: c_int;
    let M: c_int = (*cinfo).min_DCT_scaled_size;
    let mut compptr: *mut jpeg_component_info;
    let mut xbuf: JSAMPARRAY;

    /* Get top-level space for component array pointers.
     * We alloc both arrays with one call to save a few cycles.
     */
    (*jmain).xbuffer[0] = ((*(*cinfo).mem).alloc_small)(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        (*cinfo).num_components as usize * 2 * core::mem::size_of::<JSAMPARRAY>(),
    ) as JSAMPIMAGE;
    (*jmain).xbuffer[1] = (*jmain).xbuffer[0].add((*cinfo).num_components as usize);

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        rgroup = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size) /
            (*cinfo).min_DCT_scaled_size; /* height of a row group of component */
        /* Get space for pointer lists --- M+4 row groups in each list.
         * We alloc both pointer lists with one call to save a few cycles.
         */
        xbuf = ((*(*cinfo).mem).alloc_small)(
            cinfo as j_common_ptr,
            JPOOL_IMAGE,
            2 * (rgroup as usize * (M + 4) as usize) * core::mem::size_of::<JSAMPROW>(),
        ) as JSAMPARRAY;
        xbuf = xbuf.add(rgroup as usize); /* want one row group at negative offsets */
        *(*jmain).xbuffer[0].add(ci as usize) = xbuf;
        xbuf = xbuf.add((rgroup * (M + 4)) as usize);
        *(*jmain).xbuffer[1].add(ci as usize) = xbuf;

        ci += 1;
        compptr = compptr.add(1);
    }
}


unsafe fn make_funny_pointers(cinfo: j_decompress_ptr)
/* Create the funny pointer lists discussed in the comments above.
 * The actual workspace is already allocated (in main->buffer),
 * and the space for the pointer lists is allocated too.
 * This routine just fills in the curiously ordered lists.
 * This will be repeated at the beginning of each pass.
 */
{
 // bk001204 - no use main
    let jmain: my_main_ptr = (*cinfo).main as my_main_ptr;
    let mut ci: c_int;
    let mut i: c_int;
    let mut rgroup: c_int;
    let M: c_int = (*cinfo).min_DCT_scaled_size;
    let mut compptr: *mut jpeg_component_info;
    let mut buf: JSAMPARRAY;
    let mut xbuf0: JSAMPARRAY;
    let mut xbuf1: JSAMPARRAY;

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        rgroup = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size) /
            (*cinfo).min_DCT_scaled_size; /* height of a row group of component */
        xbuf0 = *(*jmain).xbuffer[0].add(ci as usize);
        xbuf1 = *(*jmain).xbuffer[1].add(ci as usize);
        /* First copy the workspace pointers as-is */
        buf = (*jmain).buffer[ci as usize];
        i = 0;
        while i < rgroup * (M + 2) {
            *xbuf0.add(i as usize) = *buf.add(i as usize);
            *xbuf1.add(i as usize) = *buf.add(i as usize);
            i += 1;
        }
        /* In the second list, put the last four row groups in swapped order */
        i = 0;
        while i < rgroup * 2 {
            *xbuf1.add((rgroup * (M - 2) + i) as usize) = *buf.add((rgroup * M + i) as usize);
            *xbuf1.add((rgroup * M + i) as usize) = *buf.add((rgroup * (M - 2) + i) as usize);
            i += 1;
        }
        /* The wraparound pointers at top and bottom will be filled later
         * (see set_wraparound_pointers, below).  Initially we want the "above"
         * pointers to duplicate the first actual data line.  This only needs
         * to happen in xbuffer[0].
         */
        i = 0;
        while i < rgroup {
            *xbuf0.offset(i as isize - rgroup as isize) = *xbuf0;
            i += 1;
        }

        ci += 1;
        compptr = compptr.add(1);
    }
}


unsafe fn set_wraparound_pointers(cinfo: j_decompress_ptr)
/* Set up the "wraparound" pointers at top and bottom of the pointer lists.
 * This changes the pointer list state from top-of-image to the normal state.
 */
{
 // bk001204 - no use main
    let jmain: my_main_ptr = (*cinfo).main as my_main_ptr;
    let mut ci: c_int;
    let mut i: c_int;
    let mut rgroup: c_int;
    let M: c_int = (*cinfo).min_DCT_scaled_size;
    let mut compptr: *mut jpeg_component_info;
    let mut xbuf0: JSAMPARRAY;
    let mut xbuf1: JSAMPARRAY;

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        rgroup = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size) /
            (*cinfo).min_DCT_scaled_size; /* height of a row group of component */
        xbuf0 = *(*jmain).xbuffer[0].add(ci as usize);
        xbuf1 = *(*jmain).xbuffer[1].add(ci as usize);
        i = 0;
        while i < rgroup {
            *xbuf0.offset(i as isize - rgroup as isize) =
                *xbuf0.add((rgroup * (M + 1) + i) as usize);
            *xbuf1.offset(i as isize - rgroup as isize) =
                *xbuf1.add((rgroup * (M + 1) + i) as usize);
            *xbuf0.add((rgroup * (M + 2) + i) as usize) = *xbuf0.add(i as usize);
            *xbuf1.add((rgroup * (M + 2) + i) as usize) = *xbuf1.add(i as usize);
            i += 1;
        }

        ci += 1;
        compptr = compptr.add(1);
    }
}


unsafe fn set_bottom_pointers(cinfo: j_decompress_ptr)
/* Change the pointer lists to duplicate the last sample row at the bottom
 * of the image.  whichptr indicates which xbuffer holds the final iMCU row.
 * Also sets rowgroups_avail to indicate number of nondummy row groups in row.
 */
{
 // bk001204 - no use main
    let jmain: my_main_ptr = (*cinfo).main as my_main_ptr;
    let mut ci: c_int;
    let mut i: c_int;
    let mut rgroup: c_int;
    let mut iMCUheight: c_int;
    let mut rows_left: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut xbuf: JSAMPARRAY;

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        /* Count sample rows in one iMCU row and in one row group */
        iMCUheight = (*compptr).v_samp_factor * (*compptr).DCT_scaled_size;
        rgroup = iMCUheight / (*cinfo).min_DCT_scaled_size;
        /* Count nondummy sample rows remaining for this component */
        rows_left = ((*compptr).downsampled_height % iMCUheight as JDIMENSION) as c_int;
        if rows_left == 0 { rows_left = iMCUheight; }
        /* Count nondummy row groups.  Should get same answer for each component,
         * so we need only do it once.
         */
        if ci == 0 {
            (*jmain).rowgroups_avail = ((rows_left - 1) / rgroup + 1) as JDIMENSION;
        }
        /* Duplicate the last real sample row rgroup*2 times; this pads out the
         * last partial rowgroup and ensures at least one full rowgroup of context.
         */
        xbuf = *(*jmain).xbuffer[(*jmain).whichptr as usize].add(ci as usize);
        i = 0;
        while i < rgroup * 2 {
            *xbuf.add((rows_left + i) as usize) = *xbuf.add((rows_left - 1) as usize);
            i += 1;
        }

        ci += 1;
        compptr = compptr.add(1);
    }
}


/*
 * Initialize for a processing pass.
 */

unsafe extern "C" fn start_pass_main(cinfo: j_decompress_ptr, pass_mode: J_BUF_MODE) {
    // bk001204 - no use main
    let jmain: my_main_ptr = (*cinfo).main as my_main_ptr;

    /* Faithful translation of C switch with #ifdef QUANT_2PASS_SUPPORTED arm.
     * Per porting rule: #[cfg] cannot gate an else-if arm directly, so the
     * JBUF_CRANK_DEST case is handled as a separate guarded block below.
     * The default ERREXIT fires for unknown modes; QUANT_2PASS_SUPPORTED mode
     * bypasses it only when ERREXIT itself terminates the process (C behavior). */
    if pass_mode == JBUF_PASS_THRU {
        if (*(*cinfo).upsample).need_context_rows != FALSE {
            (*jmain).pub_.process_data = Some(process_data_context_main);
            make_funny_pointers(cinfo); /* Create the xbuffer[] lists */
            (*jmain).whichptr = 0;    /* Read first iMCU row into xbuffer[0] */
            (*jmain).context_state = CTX_PREPARE_FOR_IMCU;
            (*jmain).iMCU_row_ctr = 0;
        } else {
            /* Simple case with no context needed */
            (*jmain).pub_.process_data = Some(process_data_simple_main);
        }
        (*jmain).buffer_full = FALSE; /* Mark buffer empty */
        (*jmain).rowgroup_ctr = 0;
    }
    /* #ifdef QUANT_2PASS_SUPPORTED */
    #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
    {
        if pass_mode == JBUF_CRANK_DEST {
            /* For last pass of 2-pass quantization, just crank the postprocessor */
            (*jmain).pub_.process_data = Some(process_data_crank_post);
            return;
        }
    }
    /* #endif QUANT_2PASS_SUPPORTED */
    if pass_mode != JBUF_PASS_THRU {
        ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
    }
}


/*
 * Process some data.
 * This handles the simple case where no context is required.
 */

unsafe extern "C" fn process_data_simple_main(
    cinfo: j_decompress_ptr,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    // bk001204 - no use main
    let jmain: my_main_ptr = (*cinfo).main as my_main_ptr;
    let rowgroups_avail: JDIMENSION;

    /* Read input data if we haven't filled the main buffer yet */
    if (*jmain).buffer_full == FALSE {
        if ((*(*cinfo).coef).decompress_data)(cinfo, (*jmain).buffer.as_mut_ptr()) == FALSE {
            return; /* suspension forced, can do nothing more */
        }
        (*jmain).buffer_full = TRUE; /* OK, we have an iMCU row to work with */
    }

    /* There are always min_DCT_scaled_size row groups in an iMCU row. */
    rowgroups_avail = (*cinfo).min_DCT_scaled_size as JDIMENSION;
    /* Note: at the bottom of the image, we may pass extra garbage row groups
     * to the postprocessor.  The postprocessor has to check for bottom
     * of image anyway (at row resolution), so no point in us doing it too.
     */

    /* Feed the postprocessor */
    ((*(*cinfo).post).post_process_data)(
        cinfo,
        (*jmain).buffer.as_mut_ptr(),
        addr_of_mut!((*jmain).rowgroup_ctr),
        rowgroups_avail,
        output_buf,
        out_row_ctr,
        out_rows_avail,
    );

    /* Has postprocessor consumed all the data yet? If so, mark buffer empty */
    if (*jmain).rowgroup_ctr >= rowgroups_avail {
        (*jmain).buffer_full = FALSE;
        (*jmain).rowgroup_ctr = 0;
    }
}


/*
 * Process some data.
 * This handles the case where context rows must be provided.
 */

unsafe extern "C" fn process_data_context_main(
    cinfo: j_decompress_ptr,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    // bk001204 - no use main
    let jmain: my_main_ptr = (*cinfo).main as my_main_ptr;

    /* Read input data if we haven't filled the main buffer yet */
    if (*jmain).buffer_full == FALSE {
        if ((*(*cinfo).coef).decompress_data)(
            cinfo,
            (*jmain).xbuffer[(*jmain).whichptr as usize],
        ) == FALSE {
            return; /* suspension forced, can do nothing more */
        }
        (*jmain).buffer_full = TRUE; /* OK, we have an iMCU row to work with */
        (*jmain).iMCU_row_ctr += 1;  /* count rows received */
    }

    /* Postprocessor typically will not swallow all the input data it is handed
     * in one call (due to filling the output buffer first).  Must be prepared
     * to exit and restart.  This switch lets us keep track of how far we got.
     * Note that each case falls through to the next on successful completion.
     *
     * Porting note: C switch with fallthrough translated as chained if-statements
     * with a local `state` variable carrying the fall-through between cases.
     */
    let mut state: c_int = (*jmain).context_state;

    if state == CTX_POSTPONED_ROW {
        /* Call postprocessor using previously set pointers for postponed row */
        ((*(*cinfo).post).post_process_data)(
            cinfo,
            (*jmain).xbuffer[(*jmain).whichptr as usize],
            addr_of_mut!((*jmain).rowgroup_ctr),
            (*jmain).rowgroups_avail,
            output_buf,
            out_row_ctr,
            out_rows_avail,
        );
        if (*jmain).rowgroup_ctr < (*jmain).rowgroups_avail {
            return; /* Need to suspend */
        }
        (*jmain).context_state = CTX_PREPARE_FOR_IMCU;
        if *out_row_ctr >= out_rows_avail {
            return; /* Postprocessor exactly filled output buf */
        }
        /*FALLTHROUGH*/
        state = CTX_PREPARE_FOR_IMCU;
    }
    if state == CTX_PREPARE_FOR_IMCU {
        /* Prepare to process first M-1 row groups of this iMCU row */
        (*jmain).rowgroup_ctr = 0;
        (*jmain).rowgroups_avail = ((*cinfo).min_DCT_scaled_size - 1) as JDIMENSION;
        /* Check for bottom of image: if so, tweak pointers to "duplicate"
         * the last sample row, and adjust rowgroups_avail to ignore padding rows.
         */
        if (*jmain).iMCU_row_ctr == (*cinfo).total_iMCU_rows {
            set_bottom_pointers(cinfo);
        }
        (*jmain).context_state = CTX_PROCESS_IMCU;
        /*FALLTHROUGH*/
        state = CTX_PROCESS_IMCU;
    }
    if state == CTX_PROCESS_IMCU {
        /* Call postprocessor using previously set pointers */
        ((*(*cinfo).post).post_process_data)(
            cinfo,
            (*jmain).xbuffer[(*jmain).whichptr as usize],
            addr_of_mut!((*jmain).rowgroup_ctr),
            (*jmain).rowgroups_avail,
            output_buf,
            out_row_ctr,
            out_rows_avail,
        );
        if (*jmain).rowgroup_ctr < (*jmain).rowgroups_avail {
            return; /* Need to suspend */
        }
        /* After the first iMCU, change wraparound pointers to normal state */
        if (*jmain).iMCU_row_ctr == 1 {
            set_wraparound_pointers(cinfo);
        }
        /* Prepare to load new iMCU row using other xbuffer list */
        (*jmain).whichptr ^= 1;  /* 0=>1 or 1=>0 */
        (*jmain).buffer_full = FALSE;
        /* Still need to process last row group of this iMCU row, */
        /* which is saved at index M+1 of the other xbuffer */
        (*jmain).rowgroup_ctr = ((*cinfo).min_DCT_scaled_size + 1) as JDIMENSION;
        (*jmain).rowgroups_avail = ((*cinfo).min_DCT_scaled_size + 2) as JDIMENSION;
        (*jmain).context_state = CTX_POSTPONED_ROW;
    }
}


/*
 * Process some data.
 * Final pass of two-pass quantization: just call the postprocessor.
 * Source data will be the postprocessor controller's internal buffer.
 */

/* #ifdef QUANT_2PASS_SUPPORTED */
#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
unsafe extern "C" fn process_data_crank_post(
    cinfo: j_decompress_ptr,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    ((*(*cinfo).post).post_process_data)(
        cinfo,
        core::ptr::null_mut::<JSAMPARRAY>() as JSAMPIMAGE,
        core::ptr::null_mut::<JDIMENSION>(),
        0 as JDIMENSION,
        output_buf,
        out_row_ctr,
        out_rows_avail,
    );
}
/* #endif QUANT_2PASS_SUPPORTED */


/*
 * Initialize main buffer controller.
 */

pub unsafe fn jinit_d_main_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean) {
    // bk001204 - no use main
    let jmain: my_main_ptr;
    let mut ci: c_int;
    let mut rgroup: c_int;
    let mut ngroups: c_int;
    let mut compptr: *mut jpeg_component_info;

    jmain = ((*(*cinfo).mem).alloc_small)(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        core::mem::size_of::<my_main_controller>(),
    ) as my_main_ptr;
    (*cinfo).main = jmain as *mut jpeg_d_main_controller;
    (*jmain).pub_.start_pass = Some(start_pass_main);

    if need_full_buffer != FALSE {  /* shouldn't happen */
        ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
    }

    /* Allocate the workspace.
     * ngroups is the number of row groups we need.
     */
    if (*(*cinfo).upsample).need_context_rows != FALSE {
        if (*cinfo).min_DCT_scaled_size < 2 { /* unsupported, see comments above */
            ERREXIT(cinfo, JERR_NOTIMPL);
        }
        alloc_funny_pointers(cinfo); /* Alloc space for xbuffer[] lists */
        ngroups = (*cinfo).min_DCT_scaled_size + 2;
    } else {
        ngroups = (*cinfo).min_DCT_scaled_size;
    }

    ci = 0;
    compptr = (*cinfo).comp_info;
    while ci < (*cinfo).num_components {
        rgroup = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size) /
            (*cinfo).min_DCT_scaled_size; /* height of a row group of component */
        (*jmain).buffer[ci as usize] = ((*(*cinfo).mem).alloc_sarray)(
            cinfo as j_common_ptr,
            JPOOL_IMAGE,
            (*compptr).width_in_blocks * (*compptr).DCT_scaled_size as JDIMENSION,
            (rgroup * ngroups) as JDIMENSION,
        );

        ci += 1;
        compptr = compptr.add(1);
    }
}
