/*
 * jdct.h
 *
 * Copyright (C) 1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This include file contains common declarations for the forward and
 * inverse DCT modules.  These declarations are private to the DCT managers
 * (jcdctmgr.c, jddctmgr.c) and the individual DCT algorithms.
 * The individual DCT algorithms are kept in separate files to ease
 * machine-dependent tuning (e.g., assembly coding).
 */

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

/* Port note: jdct.h has no #include directives of its own; all external types
 * come from the JPEG library headers that any including file would already have.
 * Glob-importing the three key jpeg-6 headers to bring those types into scope. */
use core::ffi::c_int;
use crate::codemp::jpeg_6::jpeglib_h::*;
use crate::codemp::jpeg_6::jpegint_h::*;
use crate::codemp::jpeg_6::jmorecfg_h::*;


/*
 * A forward DCT routine is given a pointer to a work area of type DCTELEM[];
 * the DCT is to be performed in-place in that buffer.  Type DCTELEM is int
 * for 8-bit samples, INT32 for 12-bit samples.  (NOTE: Floating-point DCT
 * implementations use an array of type FAST_FLOAT, instead.)
 * The DCT inputs are expected to be signed (range +-CENTERJSAMPLE).
 * The DCT outputs are returned scaled up by a factor of 8; they therefore
 * have a range of +-8K for 8-bit data, +-128K for 12-bit data.  This
 * convention improves accuracy in integer implementations and saves some
 * work in floating-point ones.
 * Quantization of the output coefficients is done by jcdctmgr.c.
 */

#[cfg(feature = "BITS_IN_JSAMPLE_8")]
pub type DCTELEM = c_int;		/* 16 or 32 bits is fine */
#[cfg(not(feature = "BITS_IN_JSAMPLE_8"))]
pub type DCTELEM = INT32;		/* must have 32 bits */

pub type forward_DCT_method_ptr = unsafe extern "C" fn(*mut DCTELEM);
pub type float_DCT_method_ptr = unsafe extern "C" fn(*mut FAST_FLOAT);


/*
 * An inverse DCT routine is given a pointer to the input JBLOCK and a pointer
 * to an output sample array.  The routine must dequantize the input data as
 * well as perform the IDCT; for dequantization, it uses the multiplier table
 * pointed to by compptr->dct_table.  The output data is to be placed into the
 * sample array starting at a specified column.  (Any row offset needed will
 * be applied to the array pointer before it is passed to the IDCT code.)
 * Note that the number of samples emitted by the IDCT routine is
 * DCT_scaled_size * DCT_scaled_size.
 */

/* typedef inverse_DCT_method_ptr is declared in jpegint.h */

/*
 * Each IDCT routine has its own ideas about the best dct_table element type.
 */

pub type ISLOW_MULT_TYPE = MULTIPLIER; /* short or int, whichever is faster */
#[cfg(feature = "BITS_IN_JSAMPLE_8")]
pub type IFAST_MULT_TYPE = MULTIPLIER; /* 16 bits is OK, use short if faster */
#[cfg(feature = "BITS_IN_JSAMPLE_8")]
pub const IFAST_SCALE_BITS: i32 = 2;	/* fractional bits in scale factors */
#[cfg(not(feature = "BITS_IN_JSAMPLE_8"))]
pub type IFAST_MULT_TYPE = INT32;	/* need 32 bits for scaled quantizers */
#[cfg(not(feature = "BITS_IN_JSAMPLE_8"))]
pub const IFAST_SCALE_BITS: i32 = 13;	/* fractional bits in scale factors */
pub type FLOAT_MULT_TYPE = FAST_FLOAT; /* preferred floating type */


/*
 * Each IDCT routine is responsible for range-limiting its results and
 * converting them to unsigned form (0..MAXJSAMPLE).  The raw outputs could
 * be quite far out of range if the input data is corrupt, so a bulletproof
 * range-limiting step is required.  We use a mask-and-table-lookup method
 * to do the combined operations quickly.  See the comments with
 * prepare_range_limit_table (in jdmaster.c) for more info.
 */

/* #define IDCT_range_limit(cinfo)  ((cinfo)->sample_range_limit + CENTERJSAMPLE) */
/* Port note: translated as an unsafe inline fn; sample_range_limit field and
 * JSAMPLE type are imported from jpeglib_h / jmorecfg_h. */
#[inline]
pub unsafe fn IDCT_range_limit(cinfo: j_decompress_ptr) -> *mut JSAMPLE {
    /* SAFETY: caller must ensure cinfo is non-null and properly initialised */
    unsafe { (*cinfo).sample_range_limit.offset(CENTERJSAMPLE as isize) }
}

pub const RANGE_MASK: i32 = MAXJSAMPLE as i32 * 4 + 3; /* 2 bits wider than legal samples */


/* Short forms of external names for systems with brain-damaged linkers. */

/* #ifdef NEED_SHORT_EXTERNAL_NAMES */
/* #define jpeg_fdct_islow	jFDislow */
/* #define jpeg_fdct_ifast	jFDifast */
/* #define jpeg_fdct_float	jFDfloat */
/* #define jpeg_idct_islow	jRDislow */
/* #define jpeg_idct_ifast	jRDifast */
/* #define jpeg_idct_float	jRDfloat */
/* #define jpeg_idct_4x4		jRD4x4  */
/* #define jpeg_idct_2x2		jRD2x2  */
/* #define jpeg_idct_1x1		jRD1x1  */
/* #endif */
/* Port note: the short-name #defines are preprocessor renames applied before
 * the extern declarations below.  In Rust this is rendered as two cfg-gated
 * extern "C" blocks — one with #[link_name] aliases, one without. */

/* Extern declarations for the forward and inverse DCT routines. */

#[cfg(feature = "NEED_SHORT_EXTERNAL_NAMES")]
unsafe extern "C" {
    #[link_name = "jFDislow"]
    pub fn jpeg_fdct_islow(data: *mut DCTELEM);
    #[link_name = "jFDifast"]
    pub fn jpeg_fdct_ifast(data: *mut DCTELEM);
    #[link_name = "jFDfloat"]
    pub fn jpeg_fdct_float(data: *mut FAST_FLOAT);

    #[link_name = "jRDislow"]
    pub fn jpeg_idct_islow(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    #[link_name = "jRDifast"]
    pub fn jpeg_idct_ifast(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    #[link_name = "jRDfloat"]
    pub fn jpeg_idct_float(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    #[link_name = "jRD4x4"]
    pub fn jpeg_idct_4x4(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    #[link_name = "jRD2x2"]
    pub fn jpeg_idct_2x2(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    #[link_name = "jRD1x1"]
    pub fn jpeg_idct_1x1(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
}

#[cfg(not(feature = "NEED_SHORT_EXTERNAL_NAMES"))]
unsafe extern "C" {
    pub fn jpeg_fdct_islow(data: *mut DCTELEM);
    pub fn jpeg_fdct_ifast(data: *mut DCTELEM);
    pub fn jpeg_fdct_float(data: *mut FAST_FLOAT);

    pub fn jpeg_idct_islow(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    pub fn jpeg_idct_ifast(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    pub fn jpeg_idct_float(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    pub fn jpeg_idct_4x4(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    pub fn jpeg_idct_2x2(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
    pub fn jpeg_idct_1x1(
        cinfo: j_decompress_ptr,
        compptr: *mut jpeg_component_info,
        coef_block: JCOEFPTR,
        output_buf: JSAMPARRAY,
        output_col: JDIMENSION,
    );
}


/*
 * Macros for handling fixed-point arithmetic; these are used by many
 * but not all of the DCT/IDCT modules.
 *
 * All values are expected to be of type INT32.
 * Fractional constants are scaled left by CONST_BITS bits.
 * CONST_BITS is defined within each module using these macros,
 * and may differ from one module to the next.
 */

pub const ONE: INT32 = 1;

/* #define CONST_SCALE (ONE << CONST_BITS) */
/* Port note: CONST_BITS is undefined here (each module defines it locally), so
 * CONST_SCALE and the macros that depend on it are rendered as macro_rules!
 * so that CONST_BITS is resolved at the call site, matching C preprocessor
 * semantics. */
#[macro_export]
macro_rules! CONST_SCALE {
    () => { ONE << CONST_BITS }
}

/* Convert a positive real constant to an integer scaled by CONST_SCALE.
 * Caution: some C compilers fail to reduce "FIX(constant)" at compile time,
 * thus causing a lot of useless floating-point operations at run time.
 */

/* #define FIX(x)	((INT32) ((x) * CONST_SCALE + 0.5)) */
#[macro_export]
macro_rules! FIX {
    ($x:expr) => {
        (($x * (ONE << CONST_BITS) as f64 + 0.5) as INT32)
    }
}

/* Descale and correctly round an INT32 value that's scaled by N bits.
 * We assume RIGHT_SHIFT rounds towards minus infinity, so adding
 * the fudge factor is correct for either sign of X.
 */

/* #define DESCALE(x,n)  RIGHT_SHIFT((x) + (ONE << ((n)-1)), n) */
#[macro_export]
macro_rules! DESCALE {
    ($x:expr, $n:expr) => {
        RIGHT_SHIFT!(($x) + (ONE << (($n) - 1)), $n)
    }
}

/* Multiply an INT32 variable by an INT32 constant to yield an INT32 result.
 * This macro is used only when the two inputs will actually be no more than
 * 16 bits wide, so that a 16x16->32 bit multiply can be used instead of a
 * full 32x32 multiply.  This provides a useful speedup on many machines.
 * Unfortunately there is no way to specify a 16x16->32 multiply portably
 * in C, but some C compilers will do the right thing if you provide the
 * correct combination of casts.
 */

/* Port note: C logic is — if SHORTxLCONST_32: SHORT*LONG form (wins over
 * SHORTxSHORT_32 when both defined, since the second #define overwrites the
 * first); elif SHORTxSHORT_32 alone: SHORT*SHORT form; else: default.
 * Rendered as three mutually-exclusive cfg-gated macro_rules! items. */

#[cfg(feature = "SHORTxLCONST_32")]		/* known to work with Microsoft C 6.0 */
#[macro_export]
macro_rules! MULTIPLY16C16 {
    ($var:expr, $const:expr) => { ((($var) as INT16) as INT32 * (($const) as INT32)) }
}
#[cfg(all(feature = "SHORTxSHORT_32", not(feature = "SHORTxLCONST_32")))]	/* may work if 'int' is 32 bits */
#[macro_export]
macro_rules! MULTIPLY16C16 {
    ($var:expr, $const:expr) => { ((($var) as INT16) as INT32 * (($const) as INT16) as INT32) }
}
#[cfg(not(any(feature = "SHORTxSHORT_32", feature = "SHORTxLCONST_32")))]	/* default definition */
#[macro_export]
macro_rules! MULTIPLY16C16 {
    ($var:expr, $const:expr) => { ($var) * ($const) }
}

/* Same except both inputs are variables. */

#[cfg(feature = "SHORTxSHORT_32")]		/* may work if 'int' is 32 bits */
#[macro_export]
macro_rules! MULTIPLY16V16 {
    ($var1:expr, $var2:expr) => { ((($var1) as INT16) as INT32 * (($var2) as INT16) as INT32) }
}

#[cfg(not(feature = "SHORTxSHORT_32"))]	/* default definition */
#[macro_export]
macro_rules! MULTIPLY16V16 {
    ($var1:expr, $var2:expr) => { ($var1) * ($var2) }
}
