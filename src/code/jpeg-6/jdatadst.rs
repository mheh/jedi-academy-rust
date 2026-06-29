/*
 * jdatadst.c
 *
 * Copyright (C) 1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains compression data destination routines for the case of
 * emitting JPEG data to a file (or any stdio stream).  While these routines
 * are sufficient for most applications, some will want to use a different
 * destination manager.
 * IMPORTANT: we assume that fwrite() will correctly transcribe an array of
 * JOCTETs into 8-bit-wide elements on external storage.  If char is wider
 * than 8 bits on your machine, you may need to do some tweaking.
 */

#![allow(non_snake_case)]

use core::ffi::c_void;

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

/* this is not a core library module, so it doesn't define JPEG_INTERNALS */
// #include "jinclude.h"
// #include "jpeglib.h"
// #include "jerror.h"

// JPEG library type stubs and declarations
pub type JOCTET = u8;
pub type FILE = c_void;

// Forward declaration of j_compress_info for pointer typing
#[repr(C)]
pub struct j_compress_info {
    // Opaque structure from libjpeg
}

pub type j_compress_ptr = *mut j_compress_info;
pub type j_common_ptr = *mut c_void;

// jpeg_destination_mgr from libjpeg
#[repr(C)]
pub struct jpeg_destination_mgr {
    pub next_output_byte: *mut JOCTET,
    pub free_in_buffer: usize,
    pub init_destination: Option<extern "C" fn(j_compress_ptr)>,
    pub empty_output_buffer: Option<extern "C" fn(j_compress_ptr) -> u8>,
    pub term_destination: Option<extern "C" fn(j_compress_ptr)>,
}

/* Expanded data destination object for stdio output */

#[repr(C)]
pub struct my_destination_mgr {
    pub pub_: jpeg_destination_mgr, /* public fields */

    pub outfile: *mut FILE,  /* target stream */
    pub buffer: *mut JOCTET, /* start of buffer */
}

pub type my_dest_ptr = *mut my_destination_mgr;

const OUTPUT_BUF_SIZE: usize = 4096; /* choose an efficiently fwrite'able size */

// External C library functions
extern "C" {
    fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut FILE) -> usize;
    fn fflush(stream: *mut FILE) -> i32;
    fn ferror(stream: *mut FILE) -> i32;
}

/*
 * Initialize destination --- called by jpeg_start_compress
 * before any data is actually written.
 */

unsafe extern "C" fn init_destination(cinfo: j_compress_ptr) {
    let dest = (*cinfo).dest as my_dest_ptr;

    /* Allocate the output buffer --- it will be released when done with image */
    (*dest).buffer = ((*(*cinfo).mem).alloc_small)(
        cinfo as j_common_ptr,
        0, /* JPOOL_IMAGE */
        OUTPUT_BUF_SIZE * std::mem::size_of::<JOCTET>(),
    ) as *mut JOCTET;

    (*dest).pub_.next_output_byte = (*dest).buffer;
    (*dest).pub_.free_in_buffer = OUTPUT_BUF_SIZE;
}

/*
 * Empty the output buffer --- called whenever buffer fills up.
 *
 * In typical applications, this should write the entire output buffer
 * (ignoring the current state of next_output_byte & free_in_buffer),
 * reset the pointer & count to the start of the buffer, and return TRUE
 * indicating that the buffer has been dumped.
 *
 * In applications that need to be able to suspend compression due to output
 * overrun, a FALSE return indicates that the buffer cannot be emptied now.
 * In this situation, the compressor will return to its caller (possibly with
 * an indication that it has not accepted all the supplied scanlines).  The
 * application should resume compression after it has made more room in the
 * output buffer.  Note that there are substantial restrictions on the use of
 * suspension --- see the documentation.
 *
 * When suspending, the compressor will back up to a convenient restart point
 * (typically the start of the current MCU). next_output_byte & free_in_buffer
 * indicate where the restart point will be if the current call returns FALSE.
 * Data beyond this point will be regenerated after resumption, so do not
 * write it out when emptying the buffer externally.
 */

unsafe extern "C" fn empty_output_buffer(cinfo: j_compress_ptr) -> u8 {
    let dest = (*cinfo).dest as my_dest_ptr;

    if fwrite(
        (*dest).buffer as *const c_void,
        1,
        OUTPUT_BUF_SIZE,
        (*dest).outfile,
    ) != OUTPUT_BUF_SIZE
    {
        // ERREXIT(cinfo, JERR_FILE_WRITE);
    }

    (*dest).pub_.next_output_byte = (*dest).buffer;
    (*dest).pub_.free_in_buffer = OUTPUT_BUF_SIZE;

    return 1u8; /* TRUE */
}

/*
 * Terminate destination --- called by jpeg_finish_compress
 * after all data has been written.  Usually needs to flush buffer.
 *
 * NB: *not* called by jpeg_abort or jpeg_destroy; surrounding
 * application must deal with any cleanup that should happen even
 * for error exit.
 */

unsafe extern "C" fn term_destination(cinfo: j_compress_ptr) {
    let dest = (*cinfo).dest as my_dest_ptr;
    let datacount = OUTPUT_BUF_SIZE - (*dest).pub_.free_in_buffer;

    /* Write any data remaining in the buffer */
    if datacount > 0 {
        if fwrite(
            (*dest).buffer as *const c_void,
            1,
            datacount,
            (*dest).outfile,
        ) != datacount
        {
            // ERREXIT(cinfo, JERR_FILE_WRITE);
        }
    }
    fflush((*dest).outfile);
    /* Make sure we wrote the output file OK */
    if ferror((*dest).outfile) != 0 {
        // ERREXIT(cinfo, JERR_FILE_WRITE);
    }
}

/*
 * Prepare for output to a stdio stream.
 * The caller must have already opened the stream, and is responsible
 * for closing it after finishing compression.
 */

pub unsafe extern "C" fn jpeg_stdio_dest(cinfo: j_compress_ptr, outfile: *mut FILE) {
    /* The destination object is made permanent so that multiple JPEG images
     * can be written to the same file without re-executing jpeg_stdio_dest.
     * This makes it dangerous to use this manager and a different destination
     * manager serially with the same JPEG object, because their private object
     * sizes may be different.  Caveat programmer.
     */
    if (*cinfo).dest.is_null() {
        /* first time for this JPEG object? */
        (*cinfo).dest = ((*(*cinfo).mem).alloc_small)(
            cinfo as j_common_ptr,
            1, /* JPOOL_PERMANENT */
            std::mem::size_of::<my_destination_mgr>(),
        ) as *mut jpeg_destination_mgr;
    }

    let dest = (*cinfo).dest as my_dest_ptr;
    (*dest).pub_.init_destination = Some(init_destination);
    (*dest).pub_.empty_output_buffer = Some(empty_output_buffer);
    (*dest).pub_.term_destination = Some(term_destination);
    (*dest).outfile = outfile;
}
