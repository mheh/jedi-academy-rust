/*
 * jchuff.h
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains declarations for Huffman entropy encoding routines
 * that are shared between the sequential encoder (jchuff.c) and the
 * progressive encoder (jcphuff.c).  No other modules need to see these.
 */

/* Trusted import — JHUFF_TBL, jpeg_compress_struct, j_compress_ptr are defined in jpeglib_h */
use crate::codemp::jpeg_6::jpeglib_h::*;

#[allow(unused_imports)]
use core::ffi::{c_char, c_long, c_uint};

/* Derived data constructed for each Huffman table */

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct c_derived_tbl {
    pub ehufco: [c_uint; 256],  /* code for each symbol */
    pub ehufsi: [c_char; 256],  /* length of code for each symbol */
    /* If no code has been allocated for a symbol S, ehufsi[S] contains 0 */
}

/* Short forms of external names for systems with brain-damaged linkers. */

// #ifdef NEED_SHORT_EXTERNAL_NAMES
// #define jpeg_make_c_derived_tbl  jMkCDerived
// #define jpeg_gen_optimal_table   jGenOptTbl
// #endif /* NEED_SHORT_EXTERNAL_NAMES */

extern "C" {
    /* Expand a Huffman table definition into the derived format */
    pub fn jpeg_make_c_derived_tbl(cinfo: j_compress_ptr,
                    htbl: *mut JHUFF_TBL, pdtbl: *mut *mut c_derived_tbl);

    /* Generate an optimal table definition given the specified counts */
    pub fn jpeg_gen_optimal_table(cinfo: j_compress_ptr,
                    htbl: *mut JHUFF_TBL, freq: *mut c_long);
}
