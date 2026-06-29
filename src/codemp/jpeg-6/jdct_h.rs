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

use core::ffi::{c_int, c_void};

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
pub type DCTELEM = i32;		/* must have 32 bits */

pub type forward_DCT_method_ptr = extern "C" fn(*mut DCTELEM);
pub type float_DCT_method_ptr = extern "C" fn(*mut FAST_FLOAT);


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
pub const IFAST_SCALE_BITS: c_int = 2;	/* fractional bits in scale factors */
#[cfg(not(feature = "BITS_IN_JSAMPLE_8"))]
pub type IFAST_MULT_TYPE = i32;	/* need 32 bits for scaled quantizers */
#[cfg(not(feature = "BITS_IN_JSAMPLE_8"))]
pub const IFAST_SCALE_BITS: c_int = 13;	/* fractional bits in scale factors */
pub type FLOAT_MULT_TYPE = FAST_FLOAT; /* preferred floating type */


/*
 * Each IDCT routine is responsible for range-limiting its results and
 * converting them to unsigned form (0..MAXJSAMPLE).  The raw outputs could
 * be quite far out of range if the input data is corrupt, so a bulletproof
 * range-limiting step is required.  We use a mask-and-table-lookup method
 * to do the combined operations quickly.  See the comments with
 * prepare_range_limit_table (in jdmaster.c) for more info.
 */

/* IDCT_range_limit(cinfo)  ((cinfo)->sample_range_limit + CENTERJSAMPLE) */

pub const RANGE_MASK: c_int = (MAXJSAMPLE * 4 + 3); /* 2 bits wider than legal samples */


/* Short forms of external names for systems with brain-damaged linkers. */

#[cfg(feature = "NEED_SHORT_EXTERNAL_NAMES")]
extern "C" {
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
extern "C" {
    /* Extern declarations for the forward and inverse DCT routines. */

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

pub const ONE: i32 = 1;
/* CONST_SCALE (ONE << CONST_BITS) */

/* Convert a positive real constant to an integer scaled by CONST_SCALE.
 * Caution: some C compilers fail to reduce "FIX(constant)" at compile time,
 * thus causing a lot of useless floating-point operations at run time.
 */

/* FIX(x)	((INT32) ((x) * CONST_SCALE + 0.5)) */

/* Descale and correctly round an INT32 value that's scaled by N bits.
 * We assume RIGHT_SHIFT rounds towards minus infinity, so adding
 * the fudge factor is correct for either sign of X.
 */

/* DESCALE(x,n)  RIGHT_SHIFT((x) + (ONE << ((n)-1)), n) */

/* Multiply an INT32 variable by an INT32 constant to yield an INT32 result.
 * This macro is used only when the two inputs will actually be no more than
 * 16 bits wide, so that a 16x16->32 bit multiply can be used instead of a
 * full 32x32 multiply.  This provides a useful speedup on many machines.
 * Unfortunately there is no way to specify a 16x16->32 multiply portably
 * in C, but some C compilers will do the right thing if you provide the
 * correct combination of casts.
 */

#[cfg(feature = "SHORTxSHORT_32")]
/* MULTIPLY16C16(var,const)  (((INT16) (var)) * ((INT16) (const))) */

#[cfg(feature = "SHORTxLCONST_32")]
/* MULTIPLY16C16(var,const)  (((INT16) (var)) * ((INT32) (const))) */

#[cfg(not(any(feature = "SHORTxSHORT_32", feature = "SHORTxLCONST_32")))]
/* MULTIPLY16C16(var,const)  ((var) * (const)) */

/* Same except both inputs are variables. */

#[cfg(feature = "SHORTxSHORT_32")]
/* MULTIPLY16V16(var1,var2)  (((INT16) (var1)) * ((INT16) (var2))) */

#[cfg(not(feature = "SHORTxSHORT_32"))]
/* MULTIPLY16V16(var1,var2)  ((var1) * (var2)) */

/* Opaque types from JPEG library — imported from jpeglib/jpegint */
pub type FAST_FLOAT = f32;
pub type MULTIPLIER = c_int;
#[repr(C)]
pub struct j_decompress {
    _private: [u8; 0],
}
pub type j_decompress_ptr = *mut j_decompress;
#[repr(C)]
pub struct jpeg_component_info {
    _private: [u8; 0],
}
pub type JCOEFPTR = *mut c_int;
pub type JSAMPARRAY = *mut *mut u8;
pub type JDIMENSION = c_int;

/* JPEG constants from jmorecfg.h */
pub const MAXJSAMPLE: c_int = 255;
pub const CENTERJSAMPLE: c_int = 128;
