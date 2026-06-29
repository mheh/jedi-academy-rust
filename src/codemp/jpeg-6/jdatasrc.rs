/*
 * jdatasrc.c
 *
 * Copyright (C) 1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains decompression data source routines for the case of
 * reading JPEG data from a file (or any stdio stream).  While these routines
 * are sufficient for most applications, some will want to use a different
 * source manager.
 * IMPORTANT: we assume that fread() will correctly transcribe an array of
 * JOCTETs from 8-bit-wide elements on external storage.  If char is wider
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

pub struct j_decompress_struct {
    pub src: *mut jpeg_source_mgr,
    pub mem: *mut jpeg_memory_mgr,
}
pub type j_decompress_ptr = *mut j_decompress_struct;

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
pub struct jpeg_source_mgr {
    pub next_input_byte: *const JOCTET, /* => next byte to read from buffer */
    pub bytes_in_buffer: usize,         /* # of bytes remaining in buffer */

    pub init_source: Option<unsafe extern "C" fn(j_decompress_ptr)>,
    pub fill_input_buffer: Option<unsafe extern "C" fn(j_decompress_ptr) -> boolean>,
    pub skip_input_data: Option<unsafe extern "C" fn(j_decompress_ptr, c_long)>,
    pub resync_to_restart: Option<unsafe extern "C" fn(j_decompress_ptr, c_int) -> boolean>,
    pub term_source: Option<unsafe extern "C" fn(j_decompress_ptr)>,
}

pub type c_long = core::ffi::c_long;

/* Expanded data source object for stdio input */

#[repr(C)]
pub struct my_source_mgr {
    pub pub_: jpeg_source_mgr, /* public fields */

    pub infile: *mut JOCTET,    /* source stream */
    pub buffer: *mut JOCTET,    /* start of buffer */
    pub start_of_file: boolean, /* have we gotten any data yet? */
}

pub type my_src_ptr = *mut my_source_mgr;

const INPUT_BUF_SIZE: usize = 4096; /* choose an efficiently fread'able size */

// External C functions
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn jpeg_resync_to_restart(cinfo: j_decompress_ptr, desired: c_int) -> boolean;
}

/*
 * Initialize source --- called by jpeg_read_header
 * before any data is actually read.
 */

#[allow(non_snake_case)]
unsafe fn init_source(cinfo: j_decompress_ptr) {
    let src = (*cinfo).src as my_src_ptr;

    /* We reset the empty-input-file flag for each image,
     * but we don't clear the input buffer.
     * This is correct behavior for reading a series of images from one source.
     */
    (*src).start_of_file = 1; /* TRUE */
}

/*
 * Fill the input buffer --- called whenever buffer is emptied.
 *
 * In typical applications, this should read fresh data into the buffer
 * (ignoring the current state of next_input_byte & bytes_in_buffer),
 * reset the pointer & count to the start of the buffer, and return TRUE
 * indicating that the buffer has been reloaded.  It is not necessary to
 * fill the buffer entirely, only to obtain at least one more byte.
 *
 * There is no such thing as an EOF return.  If the end of the file has been
 * reached, the routine has a choice of ERREXIT() or inserting fake data into
 * the buffer.  In most cases, generating a warning message and inserting a
 * fake EOI marker is the best course of action --- this will allow the
 * decompressor to output however much of the image is there.  However,
 * the resulting error message is misleading if the real problem is an empty
 * input file, so we handle that case specially.
 *
 * In applications that need to be able to suspend compression due to input
 * not being available yet, a FALSE return indicates that no more data can be
 * obtained right now, but more may be forthcoming later.  In this situation,
 * the decompressor will return to its caller (with an indication of the
 * number of scanlines it has read, if any).  The application should resume
 * decompression after it has loaded more data into the input buffer.  Note
 * that there are substantial restrictions on the use of suspension --- see
 * the documentation.
 *
 * When suspending, the decompressor will back up to a convenient restart point
 * (typically the start of the current MCU). next_input_byte & bytes_in_buffer
 * indicate where the restart point will be if the current call returns FALSE.
 * Data beyond this point must be rescanned after resumption, so move it to
 * the front of the buffer rather than discarding it.
 */

#[allow(non_snake_case)]
unsafe fn fill_input_buffer(cinfo: j_decompress_ptr) -> boolean {
    let src = (*cinfo).src as my_src_ptr;

    memcpy(
        (*src).buffer as *mut c_void,
        (*src).infile as *const c_void,
        INPUT_BUF_SIZE,
    );

    (*src).infile = (*src).infile.add(INPUT_BUF_SIZE);

    (*src).pub_.next_input_byte = (*src).buffer;
    (*src).pub_.bytes_in_buffer = INPUT_BUF_SIZE;
    (*src).start_of_file = 0; /* FALSE */

    return 1; /* TRUE */
}

/*
 * Skip data --- used to skip over a potentially large amount of
 * uninteresting data (such as an APPn marker).
 *
 * Writers of suspendable-input applications must note that skip_input_data
 * is not granted the right to give a suspension return.  If the skip extends
 * beyond the data currently in the buffer, the buffer can be marked empty so
 * that the next read will cause a fill_input_buffer call that can suspend.
 * Arranging for additional bytes to be discarded before reloading the input
 * buffer is the application writer's problem.
 */

#[allow(non_snake_case)]
unsafe fn skip_input_data(cinfo: j_decompress_ptr, num_bytes: c_long) {
    let src = (*cinfo).src as my_src_ptr;

    /* Just a dumb implementation for now.  Could use fseek() except
     * it doesn't work on pipes.  Not clear that being smart is worth
     * any trouble anyway --- large skips are infrequent.
     */
    if num_bytes > 0 {
        let mut nb = num_bytes;
        loop {
            if !(nb > (*src).pub_.bytes_in_buffer as c_long) {
                break;
            }
            nb -= (*src).pub_.bytes_in_buffer as c_long;
            fill_input_buffer(cinfo);
            /* note we assume that fill_input_buffer will never return FALSE,
             * so suspension need not be handled.
             */
        }
        (*src).pub_.next_input_byte = (*src).pub_.next_input_byte.add(nb as usize);
        (*src).pub_.bytes_in_buffer -= nb as usize;
    }
}

/*
 * An additional method that can be provided by data source modules is the
 * resync_to_restart method for error recovery in the presence of RST markers.
 * For the moment, this source module just uses the default resync method
 * provided by the JPEG library.  That method assumes that no backtracking
 * is possible.
 */

/*
 * Terminate source --- called by jpeg_finish_decompress
 * after all data has been read.  Often a no-op.
 *
 * NB: *not* called by jpeg_abort or jpeg_destroy; surrounding
 * application must deal with any cleanup that should happen even
 * for error exit.
 */

#[allow(non_snake_case)]
unsafe fn term_source(cinfo: j_decompress_ptr) {
    /* no work necessary here */
}

/*
 * Prepare for input from a stdio stream.
 * The caller must have already opened the stream, and is responsible
 * for closing it after finishing decompression.
 */

#[allow(non_snake_case)]
pub unsafe fn jpeg_stdio_src(cinfo: j_decompress_ptr, infile: *mut JOCTET) {
    /* The source object and input buffer are made permanent so that a series
     * of JPEG images can be read from the same file by calling jpeg_stdio_src
     * only before the first one.  (If we discarded the buffer at the end of
     * one image, we'd likely lose the start of the next one.)
     * This makes it unsafe to use this manager and a different source
     * manager serially with the same JPEG object.  Caveat programmer.
     */
    if (*cinfo).src.is_null() {
        /* first time for this JPEG object? */
        (*cinfo).src = ((*(*cinfo).mem).alloc_small.unwrap())(
            cinfo as j_common_ptr,
            JPOOL_PERMANENT,
            core::mem::size_of::<my_source_mgr>(),
        ) as *mut jpeg_source_mgr;
        let src = (*cinfo).src as my_src_ptr;
        (*src).buffer = ((*(*cinfo).mem).alloc_small.unwrap())(
            cinfo as j_common_ptr,
            JPOOL_PERMANENT,
            INPUT_BUF_SIZE * core::mem::size_of::<JOCTET>(),
        ) as *mut JOCTET;
    }

    let src = (*cinfo).src as my_src_ptr;
    (*src).pub_.init_source = Some(init_source);
    (*src).pub_.fill_input_buffer = Some(fill_input_buffer);
    (*src).pub_.skip_input_data = Some(skip_input_data);
    (*src).pub_.resync_to_restart = Some(jpeg_resync_to_restart); /* use default method */
    (*src).pub_.term_source = Some(term_source);
    (*src).infile = infile;
    (*src).pub_.bytes_in_buffer = 0; /* forces fill_input_buffer on first read */
    (*src).pub_.next_input_byte = core::ptr::null(); /* until buffer loaded */
}
