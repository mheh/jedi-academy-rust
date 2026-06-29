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

use core::ffi::{c_int, c_void};

/* Forward declarations of JPEG library types - defined in other modules */
pub type j_decompress_ptr = *mut c_void;
pub type JSAMPARRAY = *mut *mut u8;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type JSAMPROW = *mut u8;
pub type JDIMENSION = u32;
pub type boolean = c_int;

#[repr(C)]
pub struct jpeg_d_main_controller {
    /* Routines are filled in by the module which creates a main controller.
     * These provide the external interface to jpeg_d_main_controller.
     */
    pub process_data: Option<unsafe extern "C" fn(
        j_decompress_ptr,
        JSAMPARRAY,
        *mut JDIMENSION,
        JDIMENSION,
    )>,
    pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr, c_int)>,
}

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
pub struct my_main_controller {
    pub pub_: jpeg_d_main_controller, /* public fields */

    /* Pointer to allocated workspace (M or M+2 row groups). */
    pub buffer: [JSAMPARRAY; 10], /* MAX_COMPONENTS is typically 10 */

    pub buffer_full: boolean, /* Have we gotten an iMCU row from decoder? */
    pub rowgroup_ctr: JDIMENSION, /* counts row groups output to postprocessor */

    /* Remaining fields are only used in the context case. */

    /* These are the master pointers to the funny-order pointer lists. */
    pub xbuffer: [JSAMPIMAGE; 2], /* pointers to weird pointer lists */

    pub whichptr: c_int,         /* indicates which pointer set is now in use */
    pub context_state: c_int,    /* process_data state machine status */
    pub rowgroups_avail: JDIMENSION, /* row groups available to postprocessor */
    pub iMCU_row_ctr: JDIMENSION, /* counts iMCU rows to detect image top/bot */
}

pub type my_main_ptr = *mut my_main_controller;

/* context_state values: */
pub const CTX_PREPARE_FOR_IMCU: c_int = 0; /* need to prepare for MCU row */
pub const CTX_PROCESS_IMCU: c_int = 1;     /* feeding iMCU to postprocessor */
pub const CTX_POSTPONED_ROW: c_int = 2;    /* feeding postponed row group */

/* Forward declarations */
pub unsafe extern "C" fn process_data_simple_main(
    cinfo: j_decompress_ptr,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
);

pub unsafe extern "C" fn process_data_context_main(
    cinfo: j_decompress_ptr,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
);

#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
pub unsafe extern "C" fn process_data_crank_post(
    cinfo: j_decompress_ptr,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
);

/* Stub structure types from JPEG library - enough for this module */
#[repr(C)]
pub struct jpeg_component_info {
    pub v_samp_factor: c_int,
    pub DCT_scaled_size: c_int,
    pub width_in_blocks: JDIMENSION,
    pub downsampled_height: JDIMENSION,
    /* Additional fields omitted - only these are used in this file */
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<
        unsafe extern "C" fn(
            *mut c_void,
            c_int,
            usize,
        ) -> *mut c_void,
    >,
    pub alloc_sarray: Option<
        unsafe extern "C" fn(
            *mut c_void,
            c_int,
            JDIMENSION,
            JDIMENSION,
        ) -> JSAMPARRAY,
    >,
}

#[repr(C)]
pub struct jpeg_upsampler {
    pub need_context_rows: boolean,
    pub post_process_data: Option<
        unsafe extern "C" fn(
            j_decompress_ptr,
            JSAMPIMAGE,
            *mut JDIMENSION,
            JDIMENSION,
            JSAMPARRAY,
            *mut JDIMENSION,
            JDIMENSION,
        )
    >,
}

#[repr(C)]
pub struct jpeg_coef_controller {
    pub decompress_data: Option<
        unsafe extern "C" fn(
            j_decompress_ptr,
            JSAMPIMAGE,
        ) -> boolean
    >,
}

#[repr(C)]
pub struct jpeg_postprocessor {
    pub post_process_data: Option<
        unsafe extern "C" fn(
            j_decompress_ptr,
            JSAMPIMAGE,
            *mut JDIMENSION,
            JDIMENSION,
            JSAMPARRAY,
            *mut JDIMENSION,
            JDIMENSION,
        )
    >,
}

#[repr(C)]
pub struct j_decompress_struct {
    pub mem: *mut jpeg_memory_mgr,
    pub main: *mut jpeg_d_main_controller,
    pub coef: *mut jpeg_coef_controller,
    pub post: *mut jpeg_postprocessor,
    pub upsample: *mut jpeg_upsampler,
    pub min_DCT_scaled_size: c_int,
    pub num_components: c_int,
    pub comp_info: *mut jpeg_component_info,
    pub total_iMCU_rows: JDIMENSION,
}

pub const JPOOL_IMAGE: c_int = 0;
pub const MAX_COMPONENTS: usize = 10;

/* Constants for buffer mode */
pub const JBUF_PASS_THRU: c_int = 0;
#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
pub const JBUF_CRANK_DEST: c_int = 1;

/* Error codes */
pub const JERR_BAD_BUFFER_MODE: c_int = 1;
pub const JERR_NOTIMPL: c_int = 2;

/* Macros - stub for error reporting */
macro_rules! ERREXIT {
    ($cinfo:expr, $code:expr) => {
        return
    };
}

macro_rules! SIZEOF {
    (JSAMPARRAY) => {
        ::core::mem::size_of::<JSAMPARRAY>()
    };
    (JSAMPROW) => {
        ::core::mem::size_of::<JSAMPROW>()
    };
}

unsafe fn alloc_funny_pointers(cinfo: j_decompress_ptr)
/* Allocate space for the funny pointer lists.
 * This is done only once, not once per pass.
 */
{
    // bk001204 - no use main
    let cinfo_struct = &mut *(cinfo as *mut j_decompress_struct);
    let jmain = cinfo_struct.main as my_main_ptr;
    let mut ci = 0;
    let M = cinfo_struct.min_DCT_scaled_size;
    let mut compptr: *mut jpeg_component_info;
    let mut xbuf: JSAMPARRAY;

    /* Get top-level space for component array pointers.
     * We alloc both arrays with one call to save a few cycles.
     */
    (*jmain).xbuffer[0] = ((*(*cinfo_struct).mem).alloc_small.unwrap())(
        cinfo as *mut c_void,
        JPOOL_IMAGE,
        cinfo_struct.num_components as usize * 2 * SIZEOF!(JSAMPARRAY),
    ) as JSAMPIMAGE;
    (*jmain).xbuffer[1] = ((*jmain).xbuffer[0] as usize
        + cinfo_struct.num_components as usize * SIZEOF!(JSAMPARRAY))
        as JSAMPIMAGE;

    ci = 0;
    compptr = cinfo_struct.comp_info;
    while ci < cinfo_struct.num_components {
        let rgroup = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size)
            / cinfo_struct.min_DCT_scaled_size; /* height of a row group of component */
        /* Get space for pointer lists --- M+4 row groups in each list.
         * We alloc both pointer lists with one call to save a few cycles.
         */
        xbuf = ((*(*cinfo_struct).mem).alloc_small.unwrap())(
            cinfo as *mut c_void,
            JPOOL_IMAGE,
            2 * (rgroup as usize * (M as usize + 4)) * SIZEOF!(JSAMPROW),
        ) as JSAMPARRAY;
        xbuf = (xbuf as usize + rgroup as usize * SIZEOF!(JSAMPROW)) as JSAMPARRAY; /* want one row group at negative offsets */
        (*jmain).xbuffer[0][ci as usize] = xbuf;
        xbuf = (xbuf as usize + rgroup as usize * (M as usize + 4) * SIZEOF!(JSAMPROW)) as JSAMPARRAY;
        (*jmain).xbuffer[1][ci as usize] = xbuf;

        ci += 1;
        compptr = compptr.offset(1);
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
    let cinfo_struct = &mut *(cinfo as *mut j_decompress_struct);
    let jmain = cinfo_struct.main as my_main_ptr;
    let mut ci = 0;
    let mut i = 0;
    let M = cinfo_struct.min_DCT_scaled_size;
    let mut compptr: *mut jpeg_component_info;
    let mut buf: JSAMPARRAY;
    let mut xbuf0: JSAMPARRAY;
    let mut xbuf1: JSAMPARRAY;

    ci = 0;
    compptr = cinfo_struct.comp_info;
    while ci < cinfo_struct.num_components {
        let rgroup = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size)
            / cinfo_struct.min_DCT_scaled_size; /* height of a row group of component */
        xbuf0 = (*jmain).xbuffer[0][ci as usize];
        xbuf1 = (*jmain).xbuffer[1][ci as usize];
        /* First copy the workspace pointers as-is */
        buf = (*jmain).buffer[ci as usize];
        i = 0;
        while i < rgroup * (M + 2) {
            *xbuf0.offset(i as isize) = *buf.offset(i as isize);
            *xbuf1.offset(i as isize) = *buf.offset(i as isize);
            i += 1;
        }
        /* In the second list, put the last four row groups in swapped order */
        i = 0;
        while i < rgroup * 2 {
            *xbuf1.offset((rgroup * (M - 2) + i) as isize) =
                *buf.offset((rgroup * M + i) as isize);
            *xbuf1.offset((rgroup * M + i) as isize) =
                *buf.offset((rgroup * (M - 2) + i) as isize);
            i += 1;
        }
        /* The wraparound pointers at top and bottom will be filled later
         * (see set_wraparound_pointers, below).  Initially we want the "above"
         * pointers to duplicate the first actual data line.  This only needs
         * to happen in xbuffer[0].
         */
        i = 0;
        while i < rgroup {
            *xbuf0.offset((i - rgroup) as isize) = *xbuf0.offset(0);
            i += 1;
        }

        ci += 1;
        compptr = compptr.offset(1);
    }
}


unsafe fn set_wraparound_pointers(cinfo: j_decompress_ptr)
/* Set up the "wraparound" pointers at top and bottom of the pointer lists.
 * This changes the pointer list state from top-of-image to the normal state.
 */
{
    // bk001204 - no use main
    let cinfo_struct = &mut *(cinfo as *mut j_decompress_struct);
    let jmain = cinfo_struct.main as my_main_ptr;
    let mut ci = 0;
    let mut i = 0;
    let M = cinfo_struct.min_DCT_scaled_size;
    let mut compptr: *mut jpeg_component_info;
    let mut xbuf0: JSAMPARRAY;
    let mut xbuf1: JSAMPARRAY;

    ci = 0;
    compptr = cinfo_struct.comp_info;
    while ci < cinfo_struct.num_components {
        let rgroup = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size)
            / cinfo_struct.min_DCT_scaled_size; /* height of a row group of component */
        xbuf0 = (*jmain).xbuffer[0][ci as usize];
        xbuf1 = (*jmain).xbuffer[1][ci as usize];
        i = 0;
        while i < rgroup {
            *xbuf0.offset((i - rgroup) as isize) =
                *xbuf0.offset((rgroup * (M + 1) + i) as isize);
            *xbuf1.offset((i - rgroup) as isize) =
                *xbuf1.offset((rgroup * (M + 1) + i) as isize);
            *xbuf0.offset((rgroup * (M + 2) + i) as isize) = *xbuf0.offset(i as isize);
            *xbuf1.offset((rgroup * (M + 2) + i) as isize) = *xbuf1.offset(i as isize);
            i += 1;
        }

        ci += 1;
        compptr = compptr.offset(1);
    }
}


unsafe fn set_bottom_pointers(cinfo: j_decompress_ptr)
/* Change the pointer lists to duplicate the last sample row at the bottom
 * of the image.  whichptr indicates which xbuffer holds the final iMCU row.
 * Also sets rowgroups_avail to indicate number of nondummy row groups in row.
 */
{
    // bk001204 - no use main
    let cinfo_struct = &mut *(cinfo as *mut j_decompress_struct);
    let jmain = cinfo_struct.main as my_main_ptr;
    let mut ci = 0;
    let mut i = 0;
    let mut rgroup = 0;
    let mut iMCUheight = 0;
    let mut rows_left = 0;
    let mut compptr: *mut jpeg_component_info;
    let mut xbuf: JSAMPARRAY;

    ci = 0;
    compptr = cinfo_struct.comp_info;
    while ci < cinfo_struct.num_components {
        /* Count sample rows in one iMCU row and in one row group */
        iMCUheight = (*compptr).v_samp_factor * (*compptr).DCT_scaled_size;
        rgroup = iMCUheight / cinfo_struct.min_DCT_scaled_size;
        /* Count nondummy sample rows remaining for this component */
        rows_left = ((*compptr).downsampled_height % iMCUheight as JDIMENSION) as c_int;
        if rows_left == 0 {
            rows_left = iMCUheight;
        }
        /* Count nondummy row groups.  Should get same answer for each component,
         * so we need only do it once.
         */
        if ci == 0 {
            (*jmain).rowgroups_avail =
                ((rows_left - 1) / rgroup + 1) as JDIMENSION;
        }
        /* Duplicate the last real sample row rgroup*2 times; this pads out the
         * last partial rowgroup and ensures at least one full rowgroup of context.
         */
        xbuf = (*jmain).xbuffer[(*jmain).whichptr as usize][ci as usize];
        i = 0;
        while i < rgroup * 2 {
            *xbuf.offset((rows_left + i) as isize) =
                *xbuf.offset((rows_left - 1) as isize);
            i += 1;
        }

        ci += 1;
        compptr = compptr.offset(1);
    }
}


/*
 * Initialize for a processing pass.
 */

pub unsafe extern "C" fn start_pass_main(cinfo: j_decompress_ptr, pass_mode: c_int) {
    // bk001204 - no use main
    let cinfo_struct = &mut *(cinfo as *mut j_decompress_struct);
    let jmain = cinfo_struct.main as my_main_ptr;

    match pass_mode {
        JBUF_PASS_THRU => {
            if (*(*cinfo_struct.upsample).need_context_rows) != 0 {
                (*jmain).pub_.process_data = Some(process_data_context_main);
                make_funny_pointers(cinfo); /* Create the xbuffer[] lists */
                (*jmain).whichptr = 0; /* Read first iMCU row into xbuffer[0] */
                (*jmain).context_state = CTX_PREPARE_FOR_IMCU;
                (*jmain).iMCU_row_ctr = 0;
            } else {
                /* Simple case with no context needed */
                (*jmain).pub_.process_data = Some(process_data_simple_main);
            }
            (*jmain).buffer_full = 0; /* Mark buffer empty */
            (*jmain).rowgroup_ctr = 0;
        }
        #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
        JBUF_CRANK_DEST => {
            /* For last pass of 2-pass quantization, just crank the postprocessor */
            (*jmain).pub_.process_data = Some(process_data_crank_post);
        }
        _ => {
            ERREXIT!(cinfo, JERR_BAD_BUFFER_MODE);
        }
    }
}


/*
 * Process some data.
 * This handles the simple case where no context is required.
 */

pub unsafe extern "C" fn process_data_simple_main(
    cinfo: j_decompress_ptr,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    // bk001204 - no use main
    let cinfo_struct = &mut *(cinfo as *mut j_decompress_struct);
    let jmain = cinfo_struct.main as my_main_ptr;
    let mut rowgroups_avail: JDIMENSION;

    /* Read input data if we haven't filled the main buffer yet */
    if (*jmain).buffer_full == 0 {
        if ((*(*cinfo_struct.coef).decompress_data.unwrap())(
            cinfo,
            (*jmain).buffer.as_mut_ptr() as JSAMPIMAGE,
        )) == 0
        {
            return; /* suspension forced, can do nothing more */
        }
        (*jmain).buffer_full = 1; /* OK, we have an iMCU row to work with */
    }

    /* There are always min_DCT_scaled_size row groups in an iMCU row. */
    rowgroups_avail = cinfo_struct.min_DCT_scaled_size as JDIMENSION;
    /* Note: at the bottom of the image, we may pass extra garbage row groups
     * to the postprocessor.  The postprocessor has to check for bottom
     * of image anyway (at row resolution), so no point in us doing it too.
     */

    /* Feed the postprocessor */
    ((*(*cinfo_struct.post).post_process_data.unwrap()))(
        cinfo,
        (*jmain).buffer.as_mut_ptr() as JSAMPIMAGE,
        &mut (*jmain).rowgroup_ctr,
        rowgroups_avail,
        output_buf,
        out_row_ctr,
        out_rows_avail,
    );

    /* Has postprocessor consumed all the data yet? If so, mark buffer empty */
    if (*jmain).rowgroup_ctr >= rowgroups_avail {
        (*jmain).buffer_full = 0;
        (*jmain).rowgroup_ctr = 0;
    }
}


/*
 * Process some data.
 * This handles the case where context rows must be provided.
 */

pub unsafe extern "C" fn process_data_context_main(
    cinfo: j_decompress_ptr,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    // bk001204 - no use main
    let cinfo_struct = &mut *(cinfo as *mut j_decompress_struct);
    let jmain = cinfo_struct.main as my_main_ptr;

    /* Read input data if we haven't filled the main buffer yet */
    if (*jmain).buffer_full == 0 {
        if ((*(*cinfo_struct.coef).decompress_data.unwrap())(
            cinfo,
            (*jmain).xbuffer[(*jmain).whichptr as usize],
        )) == 0
        {
            return; /* suspension forced, can do nothing more */
        }
        (*jmain).buffer_full = 1; /* OK, we have an iMCU row to work with */
        (*jmain).iMCU_row_ctr += 1; /* count rows received */
    }

    /* Postprocessor typically will not swallow all the input data it is handed
     * in one call (due to filling the output buffer first).  Must be prepared
     * to exit and restart.  This switch lets us keep track of how far we got.
     * Note that each case falls through to the next on successful completion.
     */
    match (*jmain).context_state {
        CTX_POSTPONED_ROW => {
            /* Call postprocessor using previously set pointers for postponed row */
            ((*(*cinfo_struct.post).post_process_data.unwrap()))(
                cinfo,
                (*jmain).xbuffer[(*jmain).whichptr as usize],
                &mut (*jmain).rowgroup_ctr,
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
            // FALLTHROUGH
            {
                /* Prepare to process first M-1 row groups of this iMCU row */
                (*jmain).rowgroup_ctr = 0;
                (*jmain).rowgroups_avail =
                    (cinfo_struct.min_DCT_scaled_size - 1) as JDIMENSION;
                /* Check for bottom of image: if so, tweak pointers to "duplicate"
                 * the last sample row, and adjust rowgroups_avail to ignore padding rows.
                 */
                if (*jmain).iMCU_row_ctr == cinfo_struct.total_iMCU_rows {
                    set_bottom_pointers(cinfo);
                }
                (*jmain).context_state = CTX_PROCESS_IMCU;
                // FALLTHROUGH
                {
                    /* Call postprocessor using previously set pointers */
                    ((*(*cinfo_struct.post).post_process_data.unwrap()))(
                        cinfo,
                        (*jmain).xbuffer[(*jmain).whichptr as usize],
                        &mut (*jmain).rowgroup_ctr,
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
                    (*jmain).whichptr ^= 1; /* 0=>1 or 1=>0 */
                    (*jmain).buffer_full = 0;
                    /* Still need to process last row group of this iMCU row, */
                    /* which is saved at index M+1 of the other xbuffer */
                    (*jmain).rowgroup_ctr = (cinfo_struct.min_DCT_scaled_size + 1) as JDIMENSION;
                    (*jmain).rowgroups_avail =
                        (cinfo_struct.min_DCT_scaled_size + 2) as JDIMENSION;
                    (*jmain).context_state = CTX_POSTPONED_ROW;
                }
            }
        }
        CTX_PREPARE_FOR_IMCU => {
            /* Prepare to process first M-1 row groups of this iMCU row */
            (*jmain).rowgroup_ctr = 0;
            (*jmain).rowgroups_avail =
                (cinfo_struct.min_DCT_scaled_size - 1) as JDIMENSION;
            /* Check for bottom of image: if so, tweak pointers to "duplicate"
             * the last sample row, and adjust rowgroups_avail to ignore padding rows.
             */
            if (*jmain).iMCU_row_ctr == cinfo_struct.total_iMCU_rows {
                set_bottom_pointers(cinfo);
            }
            (*jmain).context_state = CTX_PROCESS_IMCU;
            // FALLTHROUGH
            {
                /* Call postprocessor using previously set pointers */
                ((*(*cinfo_struct.post).post_process_data.unwrap()))(
                    cinfo,
                    (*jmain).xbuffer[(*jmain).whichptr as usize],
                    &mut (*jmain).rowgroup_ctr,
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
                (*jmain).whichptr ^= 1; /* 0=>1 or 1=>0 */
                (*jmain).buffer_full = 0;
                /* Still need to process last row group of this iMCU row, */
                /* which is saved at index M+1 of the other xbuffer */
                (*jmain).rowgroup_ctr = (cinfo_struct.min_DCT_scaled_size + 1) as JDIMENSION;
                (*jmain).rowgroups_avail =
                    (cinfo_struct.min_DCT_scaled_size + 2) as JDIMENSION;
                (*jmain).context_state = CTX_POSTPONED_ROW;
            }
        }
        CTX_PROCESS_IMCU => {
            /* Call postprocessor using previously set pointers */
            ((*(*cinfo_struct.post).post_process_data.unwrap()))(
                cinfo,
                (*jmain).xbuffer[(*jmain).whichptr as usize],
                &mut (*jmain).rowgroup_ctr,
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
            (*jmain).whichptr ^= 1; /* 0=>1 or 1=>0 */
            (*jmain).buffer_full = 0;
            /* Still need to process last row group of this iMCU row, */
            /* which is saved at index M+1 of the other xbuffer */
            (*jmain).rowgroup_ctr = (cinfo_struct.min_DCT_scaled_size + 1) as JDIMENSION;
            (*jmain).rowgroups_avail =
                (cinfo_struct.min_DCT_scaled_size + 2) as JDIMENSION;
            (*jmain).context_state = CTX_POSTPONED_ROW;
        }
        _ => {}
    }
}


/*
 * Process some data.
 * Final pass of two-pass quantization: just call the postprocessor.
 * Source data will be the postprocessor controller's internal buffer.
 */

#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
pub unsafe extern "C" fn process_data_crank_post(
    cinfo: j_decompress_ptr,
    output_buf: JSAMPARRAY,
    out_row_ctr: *mut JDIMENSION,
    out_rows_avail: JDIMENSION,
) {
    let cinfo_struct = &mut *(cinfo as *mut j_decompress_struct);
    ((*(*cinfo_struct.post).post_process_data.unwrap()))(
        cinfo,
        ::core::ptr::null_mut(),
        ::core::ptr::null_mut(),
        0,
        output_buf,
        out_row_ctr,
        out_rows_avail,
    );
}


/*
 * Initialize main buffer controller.
 */

pub unsafe extern "C" fn jinit_d_main_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean) {
    // bk001204 - no use main
    let cinfo_struct = &mut *(cinfo as *mut j_decompress_struct);
    let jmain: my_main_ptr;
    let mut ci = 0;
    let mut rgroup = 0;
    let mut ngroups = 0;
    let mut compptr: *mut jpeg_component_info;

    jmain = ((*(*cinfo_struct).mem).alloc_small.unwrap())(
        cinfo as *mut c_void,
        JPOOL_IMAGE,
        ::core::mem::size_of::<my_main_controller>(),
    ) as my_main_ptr;
    cinfo_struct.main = jmain as *mut jpeg_d_main_controller;
    (*jmain).pub_.start_pass = Some(start_pass_main);

    if need_full_buffer != 0 {
        /* shouldn't happen */
        ERREXIT!(cinfo, JERR_BAD_BUFFER_MODE);
    }

    /* Allocate the workspace.
     * ngroups is the number of row groups we need.
     */
    if (*(*cinfo_struct.upsample).need_context_rows) != 0 {
        if cinfo_struct.min_DCT_scaled_size < 2 {
            /* unsupported, see comments above */
            ERREXIT!(cinfo, JERR_NOTIMPL);
        }
        alloc_funny_pointers(cinfo); /* Alloc space for xbuffer[] lists */
        ngroups = cinfo_struct.min_DCT_scaled_size + 2;
    } else {
        ngroups = cinfo_struct.min_DCT_scaled_size;
    }

    ci = 0;
    compptr = cinfo_struct.comp_info;
    while ci < cinfo_struct.num_components {
        rgroup = ((*compptr).v_samp_factor * (*compptr).DCT_scaled_size)
            / cinfo_struct.min_DCT_scaled_size; /* height of a row group of component */
        (*jmain).buffer[ci as usize] = ((*(*cinfo_struct.mem).alloc_sarray.unwrap())(
            cinfo as *mut c_void,
            JPOOL_IMAGE,
            (*compptr).width_in_blocks * (*compptr).DCT_scaled_size as JDIMENSION,
            (rgroup * ngroups) as JDIMENSION,
        ));

        ci += 1;
        compptr = compptr.offset(1);
    }
}
