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
// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// this is not a core library module, so it doesn't define JPEG_INTERNALS
// #include "jinclude.h"
// #include "jpeglib.h"
// #include "jerror.h"

#![allow(non_snake_case)]

use core::ffi::c_void;

// ============================================================================
// Stubs for JPEG-6 types and structures needed for structural coherence
// ============================================================================

pub type JOCTET = u8;
pub type boolean = u8;
pub type JDIMENSION = u32;

pub struct j_common_struct {
    pub mem: *mut jpeg_memory_mgr,
}
pub type j_common_ptr = *mut j_common_struct;

pub struct j_compress_struct {
    pub dest: *mut jpeg_destination_mgr,
    pub mem: *mut jpeg_memory_mgr,
}
pub type j_compress_ptr = *mut j_compress_struct;

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void>,
    pub alloc_large: Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void>,
    // ... other fields omitted for brevity, as they're not used here
}

pub type c_int = core::ffi::c_int;

const JPOOL_IMAGE: c_int = 1;     /* lasts until done with image/datastream */
const JPOOL_PERMANENT: c_int = 0; /* lasts until master record is destroyed */

#[repr(C)]
pub struct jpeg_destination_mgr {
    pub next_output_byte: *mut JOCTET,  /* => next byte to write in buffer */
    pub free_in_buffer: usize,          /* # of byte spaces remaining in buffer */

    pub init_destination: Option<unsafe extern "C" fn(j_compress_ptr)>,
    pub empty_output_buffer: Option<unsafe extern "C" fn(j_compress_ptr) -> boolean>,
    pub term_destination: Option<unsafe extern "C" fn(j_compress_ptr)>,
}

/* Expanded data destination object for stdio output */

#[repr(C)]
pub struct my_destination_mgr {
    pub pub_: jpeg_destination_mgr, /* public fields */

    pub outfile: *mut c_void,        /* target stream */
    pub buffer: *mut JOCTET,         /* start of buffer */
}

pub type my_dest_ptr = *mut my_destination_mgr;

const OUTPUT_BUF_SIZE: usize = 4096; /* choose an efficiently fwrite'able size */

// External C functions
extern "C" {
    pub fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
    pub fn fflush(stream: *mut c_void) -> c_int;
    pub fn ferror(stream: *mut c_void) -> c_int;
}

/*
 * Initialize destination --- called by jpeg_start_compress
 * before any data is actually written.
 */

#[allow(non_snake_case)]
fn init_destination(cinfo: j_compress_ptr) {
    unsafe {
        let dest = (*cinfo).dest as my_dest_ptr;

        /* Allocate the output buffer --- it will be released when done with image */
        (*dest).buffer = ((*(*cinfo).mem).alloc_small.unwrap())(
            cinfo as j_common_ptr,
            JPOOL_IMAGE,
            OUTPUT_BUF_SIZE * core::mem::size_of::<JOCTET>(),
        ) as *mut JOCTET;

        (*dest).pub_.next_output_byte = (*dest).buffer;
        (*dest).pub_.free_in_buffer = OUTPUT_BUF_SIZE;
    }
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

#[allow(non_snake_case)]
fn empty_output_buffer(cinfo: j_compress_ptr) -> boolean {
    unsafe {
        let dest = (*cinfo).dest as my_dest_ptr;

        if fwrite(
            (*dest).buffer as *const c_void,
            1,
            OUTPUT_BUF_SIZE,
            (*dest).outfile,
        ) != OUTPUT_BUF_SIZE
        {
            /* ERREXIT(cinfo, JERR_FILE_WRITE); */
        }

        (*dest).pub_.next_output_byte = (*dest).buffer;
        (*dest).pub_.free_in_buffer = OUTPUT_BUF_SIZE;

        return 1; /* TRUE */
    }
}

/*
 * Terminate destination --- called by jpeg_finish_compress
 * after all data has been written.  Usually needs to flush buffer.
 *
 * NB: *not* called by jpeg_abort or jpeg_destroy; surrounding
 * application must deal with any cleanup that should happen even
 * for error exit.
 */

#[allow(non_snake_case)]
fn term_destination(cinfo: j_compress_ptr) {
    unsafe {
        let dest = (*cinfo).dest as my_dest_ptr;
        let datacount = OUTPUT_BUF_SIZE - (*dest).pub_.free_in_buffer;

        /* Write any data remaining in the buffer */
        if datacount > 0 {
            if fwrite((*dest).buffer as *const c_void, 1, datacount, (*dest).outfile) != datacount
            {
                /* ERREXIT(cinfo, JERR_FILE_WRITE); */
            }
        }
        fflush((*dest).outfile);
        /* Make sure we wrote the output file OK */
        if ferror((*dest).outfile) != 0 {
            /* ERREXIT(cinfo, JERR_FILE_WRITE); */
        }
    }
}

/*
 * Prepare for output to a stdio stream.
 * The caller must have already opened the stream, and is responsible
 * for closing it after finishing compression.
 */

#[allow(non_snake_case)]
pub unsafe fn jpeg_stdio_dest(cinfo: j_compress_ptr, outfile: *mut c_void) {
    /* The destination object is made permanent so that multiple JPEG images
     * can be written to the same file without re-executing jpeg_stdio_dest.
     * This makes it dangerous to use this manager and a different destination
     * manager serially with the same JPEG object, because their private object
     * sizes may be different.  Caveat programmer.
     */
    if (*cinfo).dest.is_null() {
        /* first time for this JPEG object? */
        (*cinfo).dest = ((*(*cinfo).mem).alloc_small.unwrap())(
            cinfo as j_common_ptr,
            JPOOL_PERMANENT,
            core::mem::size_of::<my_destination_mgr>(),
        ) as *mut jpeg_destination_mgr;
    }

    let dest = (*cinfo).dest as my_dest_ptr;
    (*dest).pub_.init_destination = Some(init_destination);
    (*dest).pub_.empty_output_buffer = Some(empty_output_buffer);
    (*dest).pub_.term_destination = Some(term_destination);
    (*dest).outfile = outfile;
}
