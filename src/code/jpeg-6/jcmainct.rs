/*
 * jcmainct.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains the main buffer controller for compression.
 * The main buffer lies between the pre-processor and the JPEG
 * compressor proper; it holds downsampled data in the JPEG colorspace.
 */

#![allow(non_snake_case)]

use core::ffi::{c_int, c_long, c_void};
use core::mem::size_of;

// ============================================================================
// Stubs for JPEG-6 types and structures needed for structural coherence
// ============================================================================

pub type JDIMENSION = u32;
pub type JOCTET = u8;
pub type JSAMPROW = *mut u8;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type jvirt_sarray_ptr = *mut c_void;
pub type boolean = u8;
pub type J_BUF_MODE = c_int;

// Constants from jpeglib.h and jpegint.h
const MAX_COMPONENTS: usize = 10;
const DCTSIZE: c_int = 8;
const JPOOL_IMAGE: c_int = 0;

const FALSE: boolean = 0;
const TRUE: boolean = 1;

const JBUF_PASS_THRU: J_BUF_MODE = 0;
#[cfg(feature = "full_main_buffer_supported")]
const JBUF_SAVE_SOURCE: J_BUF_MODE = 1;
#[cfg(feature = "full_main_buffer_supported")]
const JBUF_CRANK_DEST: J_BUF_MODE = 2;
#[cfg(feature = "full_main_buffer_supported")]
const JBUF_SAVE_AND_PASS: J_BUF_MODE = 3;

const JERR_BAD_BUFFER_MODE: c_int = 1;

// Forward declarations of structures
#[repr(C)]
pub struct jpeg_component_info {
    pub width_in_blocks: c_int,
    pub height_in_blocks: c_int,
    pub v_samp_factor: c_int,
    pub h_samp_factor: c_int,
}

#[repr(C)]
pub struct jpeg_c_main_controller {
    pub start_pass: Option<unsafe extern "C" fn(*mut c_void, J_BUF_MODE)>,
    pub process_data: Option<
        unsafe extern "C" fn(*mut c_void, JSAMPARRAY, *mut JDIMENSION, JDIMENSION),
    >,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small:
        Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
    pub alloc_sarray: Option<
        unsafe extern "C" fn(*mut c_void, c_int, JDIMENSION, JDIMENSION) -> JSAMPARRAY,
    >,
    pub access_virt_sarray: Option<
        unsafe extern "C" fn(
            *mut c_void,
            jvirt_sarray_ptr,
            JDIMENSION,
            JDIMENSION,
            boolean,
        ) -> JSAMPARRAY,
    >,
    pub request_virt_sarray: Option<
        unsafe extern "C" fn(*mut c_void, c_int, boolean, JDIMENSION, JDIMENSION, JDIMENSION)
            -> jvirt_sarray_ptr,
    >,
}

#[repr(C)]
pub struct jpeg_c_prep_controller {
    pub start_pass: Option<unsafe extern "C" fn(*mut c_void, J_BUF_MODE)>,
    pub pre_process_data: Option<
        unsafe extern "C" fn(
            *mut c_void,
            JSAMPARRAY,
            *mut JDIMENSION,
            JDIMENSION,
            JSAMPIMAGE,
            *mut JDIMENSION,
            JDIMENSION,
        ),
    >,
}

#[repr(C)]
pub struct jpeg_coef_controller {
    pub compress_data: Option<unsafe extern "C" fn(*mut c_void, JSAMPIMAGE) -> boolean>,
}

#[repr(C)]
pub struct j_compress_struct {
    pub main: *mut c_void,
    pub prep: *mut jpeg_c_prep_controller,
    pub coef: *mut jpeg_coef_controller,
    pub mem: *mut jpeg_memory_mgr,
    pub comp_info: *mut jpeg_component_info,
    pub raw_data_in: boolean,
    pub num_components: c_int,
    pub total_iMCU_rows: JDIMENSION,
    pub max_v_samp_factor: c_int,
}

pub type j_compress_ptr = *mut j_compress_struct;
pub type j_common_ptr = *mut j_compress_struct;

/* Note: currently, there is no operating mode in which a full-image buffer
 * is needed at this step.  If there were, that mode could not be used with
 * "raw data" input, since this module is bypassed in that case.  However,
 * we've left the code here for possible use in special applications.
 */
// #undef FULL_MAIN_BUFFER_SUPPORTED

/* Private buffer controller object */

#[repr(C)]
struct my_main_controller {
    pub_: jpeg_c_main_controller, /* public fields */

    cur_iMCU_row: JDIMENSION,         /* number of current iMCU row */
    rowgroup_ctr: JDIMENSION,         /* counts row groups received in iMCU row */
    suspended: boolean,               /* remember if we suspended output */
    pass_mode: J_BUF_MODE,            /* current operating mode */

    /* If using just a strip buffer, this points to the entire set of buffers
     * (we allocate one for each component).  In the full-image case, this
     * points to the currently accessible strips of the virtual arrays.
     */
    buffer: [JSAMPARRAY; MAX_COMPONENTS],

    #[cfg(feature = "full_main_buffer_supported")]
    /* If using full-image storage, this array holds pointers to virtual-array
     * control blocks for each component.  Unused if not full-image storage.
     */
    whole_image: [jvirt_sarray_ptr; MAX_COMPONENTS],
}

type my_main_ptr = *mut my_main_controller;

/* External functions */
extern "C" {
    pub fn jround_up(a: c_long, b: c_long) -> c_long;
}

// SIZEOF macro equivalent
#[inline]
fn SIZEOF<T>() -> usize {
    size_of::<T>()
}

// Stub for ERREXIT - this would normally call error handling
#[inline]
#[allow(non_snake_case)]
unsafe fn ERREXIT(_cinfo: j_compress_ptr, _error_code: c_int) {
    // Stub: original code would exit with error
}

/*
 * Initialize for a processing pass.
 */

unsafe extern "C" fn start_pass_main(cinfo: j_compress_ptr, pass_mode: J_BUF_MODE) {
    // bk001204 - don't use main...
    let jmain = (*cinfo).main as my_main_ptr;

    /* Do nothing in raw-data mode. */
    if (*cinfo).raw_data_in != FALSE {
        return;
    }

    (*jmain).cur_iMCU_row = 0;       /* initialize counters */
    (*jmain).rowgroup_ctr = 0;
    (*jmain).suspended = FALSE;
    (*jmain).pass_mode = pass_mode; /* save mode for use by process_data */

    match pass_mode {
        JBUF_PASS_THRU => {
            #[cfg(feature = "full_main_buffer_supported")]
            {
                if !(*jmain).whole_image[0].is_null() {
                    ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
                }
            }
            (*jmain).pub_.process_data = Some(process_data_simple_main);
        }
        #[cfg(feature = "full_main_buffer_supported")]
        JBUF_SAVE_SOURCE | JBUF_CRANK_DEST | JBUF_SAVE_AND_PASS => {
            if (*jmain).whole_image[0].is_null() {
                ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            }
            (*jmain).pub_.process_data = Some(process_data_buffer_main);
        }
        _ => {
            ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
        }
    }
}

/*
 * Process some data.
 * This routine handles the simple pass-through mode,
 * where we have only a strip buffer.
 */

unsafe extern "C" fn process_data_simple_main(
    cinfo: j_compress_ptr,
    input_buf: JSAMPARRAY,
    in_row_ctr: *mut JDIMENSION,
    in_rows_avail: JDIMENSION,
) {
    // bk001204 - don't use main
    let jmain = (*cinfo).main as my_main_ptr;

    while (*jmain).cur_iMCU_row < (*cinfo).total_iMCU_rows {
        /* Read input data if we haven't filled the main buffer yet */
        if (*jmain).rowgroup_ctr < DCTSIZE as JDIMENSION {
            if let Some(pre_process_data_fn) = (*(*cinfo).prep).pre_process_data {
                pre_process_data_fn(
                    cinfo as *mut c_void,
                    input_buf,
                    in_row_ctr,
                    in_rows_avail,
                    (*jmain).buffer.as_mut_ptr() as JSAMPIMAGE,
                    core::ptr::addr_of_mut!((*jmain).rowgroup_ctr),
                    DCTSIZE as JDIMENSION,
                );
            }
        }

        /* If we don't have a full iMCU row buffered, return to application for
         * more data.  Note that preprocessor will always pad to fill the iMCU row
         * at the bottom of the image.
         */
        if (*jmain).rowgroup_ctr != DCTSIZE as JDIMENSION {
            return;
        }

        /* Send the completed row to the compressor */
        if let Some(compress_data_fn) = (*(*cinfo).coef).compress_data {
            if compress_data_fn(
                cinfo as *mut c_void,
                (*jmain).buffer.as_mut_ptr() as JSAMPIMAGE,
            ) == FALSE
            {
                /* If compressor did not consume the whole row, then we must need to
                 * suspend processing and return to the application.  In this situation
                 * we pretend we didn't yet consume the last input row; otherwise, if
                 * it happened to be the last row of the image, the application would
                 * think we were done.
                 */
                if (*jmain).suspended == FALSE {
                    *in_row_ctr = (*in_row_ctr).wrapping_sub(1);
                    (*jmain).suspended = TRUE;
                }
                return;
            }
        }
        /* We did finish the row.  Undo our little suspension hack if a previous
         * call suspended; then mark the main buffer empty.
         */
        if (*jmain).suspended != FALSE {
            *in_row_ctr = (*in_row_ctr).wrapping_add(1);
            (*jmain).suspended = FALSE;
        }
        (*jmain).rowgroup_ctr = 0;
        (*jmain).cur_iMCU_row += 1;
    }
}

#[cfg(feature = "full_main_buffer_supported")]
/*
 * Process some data.
 * This routine handles all of the modes that use a full-size buffer.
 */

#[cfg(feature = "full_main_buffer_supported")]
unsafe extern "C" fn process_data_buffer_main(
    cinfo: j_compress_ptr,
    input_buf: JSAMPARRAY,
    in_row_ctr: *mut JDIMENSION,
    in_rows_avail: JDIMENSION,
) {
    let main = (*cinfo).main as my_main_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;
    let writing = ((*main).pass_mode != JBUF_CRANK_DEST);

    while (*main).cur_iMCU_row < (*cinfo).total_iMCU_rows {
        /* Realign the virtual buffers if at the start of an iMCU row. */
        if (*main).rowgroup_ctr == 0 {
            ci = 0;
            compptr = (*cinfo).comp_info;
            while ci < (*cinfo).num_components {
                if let Some(access_virt_sarray_fn) = (*(*cinfo).mem).access_virt_sarray {
                    (*main).buffer[ci as usize] = access_virt_sarray_fn(
                        cinfo as *mut c_void,
                        (*main).whole_image[ci as usize],
                        (*main).cur_iMCU_row
                            * ((*compptr).v_samp_factor * DCTSIZE) as JDIMENSION,
                        ((*compptr).v_samp_factor * DCTSIZE) as JDIMENSION,
                        writing,
                    );
                }
                ci += 1;
                compptr = compptr.offset(1);
            }
            /* In a read pass, pretend we just read some source data. */
            if writing == FALSE {
                *in_row_ctr += ((*cinfo).max_v_samp_factor * DCTSIZE) as JDIMENSION;
                (*main).rowgroup_ctr = DCTSIZE as JDIMENSION;
            }
        }

        /* If a write pass, read input data until the current iMCU row is full. */
        /* Note: preprocessor will pad if necessary to fill the last iMCU row. */
        if writing != FALSE {
            if let Some(pre_process_data_fn) = (*(*cinfo).prep).pre_process_data {
                pre_process_data_fn(
                    cinfo as *mut c_void,
                    input_buf,
                    in_row_ctr,
                    in_rows_avail,
                    (*main).buffer.as_mut_ptr() as JSAMPIMAGE,
                    core::ptr::addr_of_mut!((*main).rowgroup_ctr),
                    DCTSIZE as JDIMENSION,
                );
            }
            /* Return to application if we need more data to fill the iMCU row. */
            if (*main).rowgroup_ctr < DCTSIZE as JDIMENSION {
                return;
            }
        }

        /* Emit data, unless this is a sink-only pass. */
        if (*main).pass_mode != JBUF_SAVE_SOURCE {
            if let Some(compress_data_fn) = (*(*cinfo).coef).compress_data {
                if compress_data_fn(
                    cinfo as *mut c_void,
                    (*main).buffer.as_mut_ptr() as JSAMPIMAGE,
                ) == FALSE
                {
                    /* If compressor did not consume the whole row, then we must need to
                     * suspend processing and return to the application.  In this situation
                     * we pretend we didn't yet consume the last input row; otherwise, if
                     * it happened to be the last row of the image, the application would
                     * think we were done.
                     */
                    if (*main).suspended == FALSE {
                        *in_row_ctr = (*in_row_ctr).wrapping_sub(1);
                        (*main).suspended = TRUE;
                    }
                    return;
                }
            }
            /* We did finish the row.  Undo our little suspension hack if a previous
             * call suspended; then mark the main buffer empty.
             */
            if (*main).suspended != FALSE {
                *in_row_ctr = (*in_row_ctr).wrapping_add(1);
                (*main).suspended = FALSE;
            }
        }

        /* If get here, we are done with this iMCU row.  Mark buffer empty. */
        (*main).rowgroup_ctr = 0;
        (*main).cur_iMCU_row += 1;
    }
}

#[cfg(not(feature = "full_main_buffer_supported"))]
unsafe extern "C" fn process_data_buffer_main(
    _cinfo: j_compress_ptr,
    _input_buf: JSAMPARRAY,
    _in_row_ctr: *mut JDIMENSION,
    _in_rows_avail: JDIMENSION,
) {
    // Stub: FULL_MAIN_BUFFER_SUPPORTED is not compiled
}

/*
 * Initialize main buffer controller.
 */

pub unsafe extern "C" fn jinit_c_main_controller(
    cinfo: j_compress_ptr,
    need_full_buffer: boolean,
) {
    // bk001204 - don't use main
    let mut jmain: my_main_ptr;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    jmain = if let Some(alloc_small_fn) = (*(*cinfo).mem).alloc_small {
        alloc_small_fn(cinfo as j_common_ptr, JPOOL_IMAGE, SIZEOF::<my_main_controller>())
            as my_main_ptr
    } else {
        core::ptr::null_mut()
    };

    (*cinfo).main = jmain as *mut c_void;
    (*jmain).pub_.start_pass = Some(start_pass_main);

    /* We don't need to create a buffer in raw-data mode. */
    if (*cinfo).raw_data_in != FALSE {
        return;
    }

    /* Create the buffer.  It holds downsampled data, so each component
     * may be of a different size.
     */
    if need_full_buffer != FALSE {
        #[cfg(feature = "full_main_buffer_supported")]
        {
            /* Allocate a full-image virtual array for each component */
            /* Note we pad the bottom to a multiple of the iMCU height */
            ci = 0;
            compptr = (*cinfo).comp_info;
            while ci < (*cinfo).num_components {
                if let Some(request_virt_sarray_fn) = (*(*cinfo).mem).request_virt_sarray {
                    (*jmain).whole_image[ci as usize] = request_virt_sarray_fn(
                        cinfo as j_common_ptr,
                        JPOOL_IMAGE,
                        FALSE,
                        ((*compptr).width_in_blocks * DCTSIZE) as JDIMENSION,
                        (jround_up(
                            (*compptr).height_in_blocks as c_long,
                            (*compptr).v_samp_factor as c_long,
                        ) * DCTSIZE as c_long) as JDIMENSION,
                        ((*compptr).v_samp_factor * DCTSIZE) as JDIMENSION,
                    );
                }
                ci += 1;
                compptr = compptr.offset(1);
            }
        }
        #[cfg(not(feature = "full_main_buffer_supported"))]
        {
            ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
        }
    } else {
        #[cfg(feature = "full_main_buffer_supported")]
        {
            (*jmain).whole_image[0] = core::ptr::null_mut(); /* flag for no virtual arrays */
        }
        /* Allocate a strip buffer for each component */
        ci = 0;
        compptr = (*cinfo).comp_info;
        while ci < (*cinfo).num_components {
            if let Some(alloc_sarray_fn) = (*(*cinfo).mem).alloc_sarray {
                (*jmain).buffer[ci as usize] = alloc_sarray_fn(
                    cinfo as j_common_ptr,
                    JPOOL_IMAGE,
                    ((*compptr).width_in_blocks * DCTSIZE) as JDIMENSION,
                    ((*compptr).v_samp_factor * DCTSIZE) as JDIMENSION,
                );
            }
            ci += 1;
            compptr = compptr.offset(1);
        }
    }
}
