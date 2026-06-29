/*
 * jutils.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains tables and miscellaneous utility routines needed
 * for both compression and decompression.
 * Note we prefix all global names with "j" to minimize conflicts with
 * a surrounding application.
 */

// leave this as first line for PCH reasons...
//

use core::ffi::{c_char, c_int, c_long, c_uint, c_void};

// Stub JPEG library type definitions
type JSAMPLE = u8;
type JSAMPROW = *mut JSAMPLE;
type JSAMPARRAY = *mut JSAMPROW;
type JDIMENSION = c_uint;
type JCOEF = i16;
type JCOEFPTR = *mut JCOEF;
type JBLOCK = [JCOEF; 64];
type JBLOCKROW = *mut JBLOCK;

const DCTSIZE: usize = 8;
const DCTSIZE2: usize = 64;

/*
 * jpeg_zigzag_order[i] is the zigzag-order position of the i'th element
 * of a DCT block read in natural order (left to right, top to bottom).
 */

pub const jpeg_zigzag_order: [c_int; DCTSIZE2] = [
   0,  1,  5,  6, 14, 15, 27, 28,
   2,  4,  7, 13, 16, 26, 29, 42,
   3,  8, 12, 17, 25, 30, 41, 43,
   9, 11, 18, 24, 31, 40, 44, 53,
  10, 19, 23, 32, 39, 45, 52, 54,
  20, 22, 33, 38, 46, 51, 55, 60,
  21, 34, 37, 47, 50, 56, 59, 61,
  35, 36, 48, 49, 57, 58, 62, 63
];

/*
 * jpeg_natural_order[i] is the natural-order position of the i'th element
 * of zigzag order.
 *
 * When reading corrupted data, the Huffman decoders could attempt
 * to reference an entry beyond the end of this array (if the decoded
 * zero run length reaches past the end of the block).  To prevent
 * wild stores without adding an inner-loop test, we put some extra
 * "63"s after the real entries.  This will cause the extra coefficient
 * to be stored in location 63 of the block, not somewhere random.
 * The worst case would be a run-length of 15, which means we need 16
 * fake entries.
 */

pub const jpeg_natural_order: [c_int; DCTSIZE2 + 16] = [
  0,  1,  8, 16,  9,  2,  3, 10,
 17, 24, 32, 25, 18, 11,  4,  5,
 12, 19, 26, 33, 40, 48, 41, 34,
 27, 20, 13,  6,  7, 14, 21, 28,
 35, 42, 49, 56, 57, 50, 43, 36,
 29, 22, 15, 23, 30, 37, 44, 51,
 58, 59, 52, 45, 38, 31, 39, 46,
 53, 60, 61, 54, 47, 55, 62, 63,
 63, 63, 63, 63, 63, 63, 63, 63, /* extra entries for safety in decoder */
 63, 63, 63, 63, 63, 63, 63, 63
];


/*
 * Arithmetic utilities
 */

pub extern "C" fn jdiv_round_up(a: c_long, b: c_long) -> c_long {
    /* Compute a/b rounded up to next integer, ie, ceil(a/b) */
    /* Assumes a >= 0, b > 0 */
    (a + b - 1) / b
}


pub extern "C" fn jround_up(mut a: c_long, b: c_long) -> c_long {
    /* Compute a rounded up to next multiple of b, ie, ceil(a/b)*b */
    /* Assumes a >= 0, b > 0 */
    a += b - 1;
    a - (a % b)
}


/* On normal machines we can apply MEMCOPY() and MEMZERO() to sample arrays
 * and coefficient-block arrays.  This won't work on 80x86 because the arrays
 * are FAR and we're assuming a small-pointer memory model.  However, some
 * DOS compilers provide far-pointer versions of memcpy() and memset() even
 * in the small-model libraries.  These will be used if USE_FMEM is defined.
 * Otherwise, the routines below do it the hard way.  (The performance cost
 * is not all that great, because these routines aren't very heavily used.)
 */

pub extern "C" fn jcopy_sample_rows(
    mut input_array: JSAMPARRAY,
    source_row: c_int,
    mut output_array: JSAMPARRAY,
    dest_row: c_int,
    num_rows: c_int,
    num_cols: JDIMENSION,
) {
    /* Copy some rows of samples from one place to another.
     * num_rows rows are copied from input_array[source_row++]
     * to output_array[dest_row++]; these areas may overlap for duplication.
     * The source and destination arrays must be at least as wide as num_cols.
     */

    let mut inptr: JSAMPROW;
    let mut outptr: JSAMPROW;
    let mut count: JDIMENSION;
    let mut row: c_int;

    input_array = unsafe { input_array.offset(source_row as isize) };
    output_array = unsafe { output_array.offset(dest_row as isize) };

    row = num_rows;
    while row > 0 {
        inptr = unsafe { *input_array };
        input_array = unsafe { input_array.offset(1) };
        outptr = unsafe { *output_array };
        output_array = unsafe { output_array.offset(1) };

        count = num_cols;
        while count > 0 {
            unsafe {
                *outptr = *inptr;	/* needn't bother with GETJSAMPLE() here */
                outptr = outptr.offset(1);
                inptr = inptr.offset(1);
            }
            count -= 1;
        }

        row -= 1;
    }
}


pub extern "C" fn jcopy_block_row(
    input_row: JBLOCKROW,
    output_row: JBLOCKROW,
    num_blocks: JDIMENSION,
) {
    /* Copy a row of coefficient blocks from one place to another. */

    let mut inptr: JCOEFPTR = input_row as JCOEFPTR;
    let mut outptr: JCOEFPTR = output_row as JCOEFPTR;
    let mut count: c_long;

    count = (num_blocks as c_long) * (DCTSIZE2 as c_long);
    while count > 0 {
        unsafe {
            *outptr = *inptr;
            outptr = outptr.offset(1);
            inptr = inptr.offset(1);
        }
        count -= 1;
    }
}


pub extern "C" fn jzero_far(target: *mut c_void, bytestozero: usize) {
    /* Zero out a chunk of FAR memory. */
    /* This might be sample-array data, block-array data, or alloc_large data. */

    let mut ptr: *mut c_char = target as *mut c_char;
    let mut count: usize;

    count = bytestozero;
    while count > 0 {
        unsafe {
            *ptr = 0;
            ptr = ptr.offset(1);
        }
        count -= 1;
    }
}
