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

use core::ffi::c_int;
use crate::codemp::jpeg_6::jpegint_h::*;
use crate::codemp::jpeg_6::jmemmgr::{jvirt_sarray_ptr};

// Note: currently, there is no operating mode in which a full-image buffer
// is needed at this step.  If there were, that mode could not be used with
// "raw data" input, since this module is bypassed in that case.  However,
// we've left the code here for possible use in special applications.

// FULL_MAIN_BUFFER_SUPPORTED is disabled - see #undef in original

// Private buffer controller object

#[repr(C)]
pub struct my_main_controller {
    pub pub_: jpeg_c_main_controller, /* public fields */
    pub cur_iMCU_row: JDIMENSION,     /* number of current iMCU row */
    pub rowgroup_ctr: JDIMENSION,     /* counts row groups received in iMCU row */
    pub suspended: boolean,            /* remember if we suspended output */
    pub pass_mode: J_BUF_MODE,         /* current operating mode */

    // If using just a strip buffer, this points to the entire set of buffers
    // (we allocate one for each component).  In the full-image case, this
    // points to the currently accessible strips of the virtual arrays.
    pub buffer: [JSAMPARRAY; MAX_COMPONENTS as usize],

    // ifdef FULL_MAIN_BUFFER_SUPPORTED
    // If using full-image storage, this array holds pointers to virtual-array
    // control blocks for each component.  Unused if not full-image storage.
    // pub whole_image: [jvirt_sarray_ptr; MAX_COMPONENTS as usize],
}

type my_main_ptr = *mut my_main_controller;

// Forward declarations
// (The following functions are defined in this file but used via function pointers)

/*
 * Initialize for a processing pass.
 */

pub unsafe extern "C" fn start_pass_main(cinfo: j_compress_ptr, pass_mode: J_BUF_MODE) {
    // bk001204 - don't use main...
    let jmain = (*cinfo).main as my_main_ptr;

    /* Do nothing in raw-data mode. */
    if (*cinfo).raw_data_in != 0 {
        return;
    }

    (*jmain).cur_iMCU_row = 0; /* initialize counters */
    (*jmain).rowgroup_ctr = 0;
    (*jmain).suspended = 0; /* FALSE */
    (*jmain).pass_mode = pass_mode; /* save mode for use by process_data */

    match pass_mode {
        J_BUF_MODE::JBUF_PASS_THRU => {
            // ifdef FULL_MAIN_BUFFER_SUPPORTED
            // if (*jmain).whole_image[0] != core::ptr::null_mut() {
            //     ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
            // }
            // endif
            (*(*jmain).pub_).process_data = Some(process_data_simple_main);
        }
        // ifdef FULL_MAIN_BUFFER_SUPPORTED
        // J_BUF_MODE::JBUF_SAVE_SOURCE
        // | J_BUF_MODE::JBUF_CRANK_DEST
        // | J_BUF_MODE::JBUF_SAVE_AND_PASS => {
        //     if (*jmain).whole_image[0] == core::ptr::null_mut() {
        //         ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
        //     }
        //     (*(*jmain).pub_).process_data = Some(process_data_buffer_main);
        // }
        // endif
        _ => {
            // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
        }
    }
}

/*
 * Process some data.
 * This routine handles the simple pass-through mode,
 * where we have only a strip buffer.
 */

pub unsafe extern "C" fn process_data_simple_main(
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
            if let Some(pre_process_data) = (*(*cinfo).prep).pre_process_data {
                pre_process_data(
                    cinfo,
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
        let compress_result = if let Some(compress_data) = (*(*cinfo).coef).compress_data {
            compress_data(cinfo, (*jmain).buffer.as_mut_ptr() as JSAMPIMAGE)
        } else {
            0
        };

        if compress_result == 0 {
            /* If compressor did not consume the whole row, then we must need to
             * suspend processing and return to the application.  In this situation
             * we pretend we didn't yet consume the last input row; otherwise, if
             * it happened to be the last row of the image, the application would
             * think we were done.
             */
            if (*jmain).suspended == 0 {
                *in_row_ctr = (*in_row_ctr).wrapping_sub(1);
                (*jmain).suspended = 1; /* TRUE */
            }
            return;
        }
        /* We did finish the row.  Undo our little suspension hack if a previous
         * call suspended; then mark the main buffer empty.
         */
        if (*jmain).suspended != 0 {
            *in_row_ctr = (*in_row_ctr).wrapping_add(1);
            (*jmain).suspended = 0; /* FALSE */
        }
        (*jmain).rowgroup_ctr = 0;
        (*jmain).cur_iMCU_row = (*jmain).cur_iMCU_row.wrapping_add(1);
    }
}

// ifdef FULL_MAIN_BUFFER_SUPPORTED

/*
 * Process some data.
 * This routine handles all of the modes that use a full-size buffer.
 */

// pub unsafe extern "C" fn process_data_buffer_main(
//     cinfo: j_compress_ptr,
//     input_buf: JSAMPARRAY,
//     in_row_ctr: *mut JDIMENSION,
//     in_rows_avail: JDIMENSION,
// ) {
//     let main = (*cinfo).main as my_main_ptr;
//     let mut ci: c_int;
//     let mut compptr: *mut jpeg_component_info;
//     let writing = ((*main).pass_mode as c_int) != (J_BUF_MODE::JBUF_CRANK_DEST as c_int);
//
//     while (*main).cur_iMCU_row < (*cinfo).total_iMCU_rows {
//         /* Realign the virtual buffers if at the start of an iMCU row. */
//         if (*main).rowgroup_ctr == 0 {
//             ci = 0;
//             compptr = (*cinfo).comp_info;
//             while ci < (*cinfo).num_components {
//                 if let Some(access_virt_sarray) = (*(*cinfo).mem).access_virt_sarray {
//                     (*main).buffer[ci as usize] = access_virt_sarray(
//                         cinfo as j_common_ptr,
//                         (*main).whole_image[ci as usize],
//                         (*main).cur_iMCU_row
//                             * ((*compptr).v_samp_factor as JDIMENSION * (DCTSIZE as JDIMENSION)),
//                         ((*compptr).v_samp_factor as JDIMENSION * (DCTSIZE as JDIMENSION)),
//                         if writing { 1 } else { 0 },
//                     );
//                 }
//                 ci += 1;
//                 compptr = compptr.add(1);
//             }
//             /* In a read pass, pretend we just read some source data. */
//             if writing == 0 {
//                 *in_row_ctr +=
//                     ((*cinfo).max_v_samp_factor as JDIMENSION) * (DCTSIZE as JDIMENSION);
//                 (*main).rowgroup_ctr = DCTSIZE as JDIMENSION;
//             }
//         }
//
//         /* If a write pass, read input data until the current iMCU row is full. */
//         /* Note: preprocessor will pad if necessary to fill the last iMCU row. */
//         if writing != 0 {
//             if let Some(pre_process_data) = (*(*cinfo).prep).pre_process_data {
//                 pre_process_data(
//                     cinfo,
//                     input_buf,
//                     in_row_ctr,
//                     in_rows_avail,
//                     (*main).buffer.as_mut_ptr() as JSAMPIMAGE,
//                     &mut (*main).rowgroup_ctr,
//                     DCTSIZE as JDIMENSION,
//                 );
//             }
//             /* Return to application if we need more data to fill the iMCU row. */
//             if (*main).rowgroup_ctr < DCTSIZE as JDIMENSION {
//                 return;
//             }
//         }
//
//         /* Emit data, unless this is a sink-only pass. */
//         if ((*main).pass_mode as c_int) != (J_BUF_MODE::JBUF_SAVE_SOURCE as c_int) {
//             let compress_result = if let Some(compress_data) = (*(*cinfo).coef).compress_data {
//                 compress_data(cinfo, (*main).buffer.as_mut_ptr() as JSAMPIMAGE)
//             } else {
//                 0
//             };
//
//             if compress_result == 0 {
//                 /* If compressor did not consume the whole row, then we must need to
//                  * suspend processing and return to the application.  In this situation
//                  * we pretend we didn't yet consume the last input row; otherwise, if
//                  * it happened to be the last row of the image, the application would
//                  * think we were done.
//                  */
//                 if (*main).suspended == 0 {
//                     *in_row_ctr = (*in_row_ctr).wrapping_sub(1);
//                     (*main).suspended = 1; /* TRUE */
//                 }
//                 return;
//             }
//             /* We did finish the row.  Undo our little suspension hack if a previous
//              * call suspended; then mark the main buffer empty.
//              */
//             if (*main).suspended != 0 {
//                 *in_row_ctr = (*in_row_ctr).wrapping_add(1);
//                 (*main).suspended = 0; /* FALSE */
//             }
//         }
//
//         /* If get here, we are done with this iMCU row.  Mark buffer empty. */
//         (*main).rowgroup_ctr = 0;
//         (*main).cur_iMCU_row = (*main).cur_iMCU_row.wrapping_add(1);
//     }
// }

// endif /* FULL_MAIN_BUFFER_SUPPORTED */

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

    jmain = (if let Some(alloc_small) = (*(*cinfo).mem).alloc_small {
        alloc_small(
            cinfo as j_common_ptr,
            1i32, /* JPOOL_IMAGE */
            core::mem::size_of::<my_main_controller>(),
        )
    } else {
        core::ptr::null_mut()
    }) as my_main_ptr;

    (*cinfo).main = jmain as *mut jpeg_c_main_controller;
    (*(*jmain).pub_).start_pass = Some(start_pass_main);

    /* We don't need to create a buffer in raw-data mode. */
    if (*cinfo).raw_data_in != 0 {
        return;
    }

    /* Create the buffer.  It holds downsampled data, so each component
     * may be of a different size.
     */
    if need_full_buffer != 0 {
        // ifdef FULL_MAIN_BUFFER_SUPPORTED
        /* Allocate a full-image virtual array for each component */
        /* Note we pad the bottom to a multiple of the iMCU height */
        // ci = 0;
        // compptr = (*cinfo).comp_info;
        // while ci < (*cinfo).num_components {
        //     if let Some(request_virt_sarray) = (*(*cinfo).mem).request_virt_sarray {
        //         (*jmain).whole_image[ci as usize] = request_virt_sarray(
        //             cinfo as j_common_ptr,
        //             1, /* JPOOL_IMAGE */
        //             0, /* FALSE */
        //             ((*compptr).width_in_blocks as JDIMENSION) * (DCTSIZE as JDIMENSION),
        //             jround_up(
        //                 (*compptr).height_in_blocks as i32,
        //                 (*compptr).v_samp_factor as i32,
        //             ) as JDIMENSION
        //                 * (DCTSIZE as JDIMENSION),
        //             ((*compptr).v_samp_factor as JDIMENSION) * (DCTSIZE as JDIMENSION),
        //         );
        //     }
        //     ci += 1;
        //     compptr = compptr.add(1);
        // }
        // else
        // ERREXIT(cinfo, JERR_BAD_BUFFER_MODE);
        // endif
    } else {
        // ifdef FULL_MAIN_BUFFER_SUPPORTED
        // (*jmain).whole_image[0] = core::ptr::null_mut(); /* flag for no virtual arrays */
        // endif
        /* Allocate a strip buffer for each component */
        ci = 0;
        compptr = (*cinfo).comp_info;
        while ci < (*cinfo).num_components {
            if let Some(alloc_sarray) = (*(*cinfo).mem).alloc_sarray {
                (*jmain).buffer[ci as usize] = alloc_sarray(
                    cinfo as j_common_ptr,
                    1i32, /* JPOOL_IMAGE */
                    ((*compptr).width_in_blocks as JDIMENSION) * (DCTSIZE as JDIMENSION),
                    ((*compptr).v_samp_factor as JDIMENSION) * (DCTSIZE as JDIMENSION),
                );
            }
            ci += 1;
            compptr = compptr.add(1);
        }
    }
}
