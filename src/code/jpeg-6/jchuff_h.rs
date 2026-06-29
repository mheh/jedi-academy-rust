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

use core::ffi::{c_char, c_long, c_uint, c_void};

/* Derived data constructed for each Huffman table */

#[repr(C)]
pub struct c_derived_tbl {
    pub ehufco: [c_uint; 256],	/* code for each symbol */
    pub ehufsi: [c_char; 256],		/* length of code for each symbol */
    /* If no code has been allocated for a symbol S, ehufsi[S] contains 0 */
}

/* Short forms of external names for systems with brain-damaged linkers. */

#[cfg(feature = "NEED_SHORT_EXTERNAL_NAMES")]
pub use self::jpeg_make_c_derived_tbl as jMkCDerived;
#[cfg(feature = "NEED_SHORT_EXTERNAL_NAMES")]
pub use self::jpeg_gen_optimal_table as jGenOptTbl;

/* Stub types for JPEG structures (defined elsewhere) */
pub type j_compress_ptr = *mut c_void;
pub type JHUFF_TBL = c_void;

/* Expand a Huffman table definition into the derived format */
extern "C" {
    pub fn jpeg_make_c_derived_tbl(
        cinfo: j_compress_ptr,
        htbl: *mut JHUFF_TBL,
        pdtbl: *mut *mut c_derived_tbl,
    );

    /* Generate an optimal table definition given the specified counts */
    pub fn jpeg_gen_optimal_table(
        cinfo: j_compress_ptr,
        htbl: *mut JHUFF_TBL,
        freq: *mut c_long,
    );
}
