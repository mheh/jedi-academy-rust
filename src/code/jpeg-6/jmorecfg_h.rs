/*
 * jmorecfg.h
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains additional configuration options that customize the
 * JPEG software for special applications or support machine-dependent
 * optimizations.  Most users will not need to touch this file.
 */

#![allow(non_snake_case)]

use core::ffi::c_int;

/*
 * Define BITS_IN_JSAMPLE as either
 *   8   for 8-bit sample values (the usual setting)
 *   12  for 12-bit sample values
 * Only 8 and 12 are legal data precisions for lossy JPEG according to the
 * JPEG standard, and the IJG code does not support anything else!
 * We do not support run-time selection of data precision, sorry.
 */

pub const BITS_IN_JSAMPLE: u32 = 8;	/* use 8 or 12 */


/*
 * Maximum number of components (color channels) allowed in JPEG image.
 * To meet the letter of the JPEG spec, set this to 255.  However, darn
 * few applications need more than 4 channels (maybe 5 for CMYK + alpha
 * mask).  We recommend 10 as a reasonable compromise; use 4 if you are
 * really short on memory.  (Each allowed component costs a hundred or so
 * bytes of storage, whether actually used in an image or not.)
 */

pub const MAX_COMPONENTS: u32 = 10;	/* maximum number of image components */


/*
 * Basic data types.
 * You may need to change these if you have a machine with unusual data
 * type sizes; for example, "char" not 8 bits, "short" not 16 bits,
 * or "long" not 32 bits.  We don't care whether "int" is 16 or 32 bits,
 * but it had better be at least 16.
 */

/* Representation of a single sample (pixel element value).
 * We frequently allocate large arrays of these, so it's important to keep
 * them small.  But if you have memory to burn and access to char or short
 * arrays is very slow on your hardware, you might want to change these.
 */

/* BITS_IN_JSAMPLE == 8:
 * JSAMPLE should be the smallest type that will hold the values 0..255.
 * You can use a signed char by having GETJSAMPLE mask it with 0xFF.
 * Assuming HAVE_UNSIGNED_CHAR is defined.
 */

pub type JSAMPLE = u8;

#[inline]
pub fn GETJSAMPLE(value: JSAMPLE) -> c_int {
    value as c_int
}

pub const MAXJSAMPLE: u32 = 255;
pub const CENTERJSAMPLE: u32 = 128;


/* Representation of a DCT frequency coefficient.
 * This should be a signed value of at least 16 bits; "short" is usually OK.
 * Again, we allocate large arrays of these, but you can change to int
 * if you have memory to burn and "short" is really slow.
 */

pub type JCOEF = i16;


/* Compressed datastreams are represented as arrays of JOCTET.
 * These must be EXACTLY 8 bits wide, at least once they are written to
 * external storage.  Note that when using the stdio data source/destination
 * managers, this is also the data type passed to fread/fwrite.
 */

pub type JOCTET = u8;

#[inline]
pub fn GETJOCTET(value: JOCTET) -> JOCTET {
    value
}


/* These typedefs are used for various table entries and so forth.
 * They must be at least as wide as specified; but making them too big
 * won't cost a huge amount of memory, so we don't provide special
 * extraction code like we did for JSAMPLE.  (In other words, these
 * typedefs live at a different point on the speed/space tradeoff curve.)
 */

/* UINT8 must hold at least the values 0..255. */

pub type UINT8 = u8;

/* UINT16 must hold at least the values 0..65535. */

pub type UINT16 = u16;

// compile warning for VC6 with CPP being defined
// typedef long INT32;

/* INT16 must hold at least the values -32768..32767. */

pub type INT16 = i16;

/* INT32 must hold at least signed 32-bit values. */

//#ifndef XMD_H			/* X11/xmd.h correctly defines INT32 */
//typedef long INT32;
//#endif

/* Datatype used for image dimensions.  The JPEG standard only supports
 * images up to 64K*64K due to 16-bit fields in SOF markers.  Therefore
 * "unsigned int" is sufficient on all machines.  However, if you need to
 * handle larger images and you don't mind deviating from the spec, you
 * can change this datatype.
 */

pub type JDIMENSION = u32;

pub const JPEG_MAX_DIMENSION: u32 = 65500;  /* a tad under 64K to prevent overflows */


/* These defines are used in all function definitions and extern declarations.
 * You could modify them if you need to change function linkage conventions.
 * Another application is to make all functions global for use with debuggers
 * or code profilers that require it.
 *
 * In Rust, visibility and calling conventions are handled differently:
 * - METHODDEF (static): Use private functions or pub functions as needed
 * - LOCAL (static): Use private functions
 * - GLOBAL: Use pub functions
 * - EXTERN (extern): Use extern "C" declarations
 *
 * #define METHODDEF static	/* a function called through method pointers */
 * #define LOCAL	  static	/* a function used only in its module */
 * #define GLOBAL			/* a function referenced thru EXTERNs */
 * #define EXTERN	  extern	/* a reference to a GLOBAL function */
 */


/* Here is the pseudo-keyword for declaring pointers that must be "far"
 * on 80x86 machines.  Most of the specialized coding for 80x86 is handled
 * by just saying "FAR *" where such a pointer is needed.  In a few places
 * explicit coding is needed; see uses of the NEED_FAR_POINTERS symbol.
 * FAR pointer semantics are not needed on modern systems, so this is omitted.
 */

/* #ifdef NEED_FAR_POINTERS
 * #undef FAR
 * #define FAR  far
 * #else
 * #undef FAR
 * #define FAR
 * #endif
 */


/*
 * On a few systems, type boolean and/or its values FALSE, TRUE may appear
 * in standard header files.  Or you may have conflicts with application-
 * specific header files that you want to include together with these files.
 * Defining HAVE_BOOLEAN before including jpeglib.h should make it work.
 */

/* #ifndef HAVE_BOOLEAN
 * typedef int boolean;
 * #endif
 */

pub const FALSE: u32 = 0;		/* values of boolean */
pub const TRUE: u32 = 1;


/*
 * The remaining options affect code selection within the JPEG library,
 * but they don't need to be visible to most applications using the library.
 * To minimize application namespace pollution, the symbols won't be
 * defined unless JPEG_INTERNALS or JPEG_INTERNAL_OPTIONS has been defined.
 */

/* #ifdef JPEG_INTERNALS
 * #define JPEG_INTERNAL_OPTIONS
 * #endif
 *
 * #ifdef JPEG_INTERNAL_OPTIONS
 */


/*
 * These defines indicate whether to include various optional functions.
 * Undefining some of these symbols will produce a smaller but less capable
 * library.  Note that you can leave certain source files out of the
 * compilation/linking process if you've #undef'd the corresponding symbols.
 * (You may HAVE to do that if your compiler doesn't like null source files.)
 */

/* Arithmetic coding is unsupported for legal reasons.  Complaints to IBM. */

/* Capability options common to encoder and decoder: */

/* #undef DCT_ISLOW_SUPPORTED	/* slow but accurate integer algorithm */
 * #undef DCT_IFAST_SUPPORTED	/* faster, less accurate integer method */
 * #define DCT_FLOAT_SUPPORTED	/* floating-point: accurate, fast on fast HW */
 */

pub const DCT_FLOAT_SUPPORTED: bool = true;

/* Encoder capability options: */

/* #undef  C_ARITH_CODING_SUPPORTED    /* Arithmetic coding back end? */
 * #define C_MULTISCAN_FILES_SUPPORTED /* Multiple-scan JPEG files? */
 * #define C_PROGRESSIVE_SUPPORTED	    /* Progressive JPEG? (Requires MULTISCAN)*/
 * #define ENTROPY_OPT_SUPPORTED	    /* Optimization of entropy coding parms? */
 */

pub const C_MULTISCAN_FILES_SUPPORTED: bool = true; /* Multiple-scan JPEG files? */
pub const C_PROGRESSIVE_SUPPORTED: bool = true; /* Progressive JPEG? (Requires MULTISCAN)*/
pub const ENTROPY_OPT_SUPPORTED: bool = true; /* Optimization of entropy coding parms? */

/* Note: if you selected 12-bit data precision, it is dangerous to turn off
 * ENTROPY_OPT_SUPPORTED.  The standard Huffman tables are only good for 8-bit
 * precision, so jchuff.c normally uses entropy optimization to compute
 * usable tables for higher precision.  If you don't want to do optimization,
 * you'll have to supply different default Huffman tables.
 * The exact same statements apply for progressive JPEG: the default tables
 * don't work for progressive mode.  (This may get fixed, however.)
 */

pub const INPUT_SMOOTHING_SUPPORTED: bool = true; /* Input image smoothing option? */

/* Decoder capability options: */

/* #undef  D_ARITH_CODING_SUPPORTED    /* Arithmetic coding back end? */
 * #undef D_MULTISCAN_FILES_SUPPORTED /* Multiple-scan JPEG files? */
 * #undef D_PROGRESSIVE_SUPPORTED	    /* Progressive JPEG? (Requires MULTISCAN)*/
 * #undef BLOCK_SMOOTHING_SUPPORTED   /* Block smoothing? (Progressive only) */
 * #undef IDCT_SCALING_SUPPORTED	    /* Output rescaling via IDCT? */
 * #undef  UPSAMPLE_SCALING_SUPPORTED  /* Output rescaling at upsample stage? */
 * #undef UPSAMPLE_MERGING_SUPPORTED  /* Fast path for sloppy upsampling? */
 * #undef QUANT_1PASS_SUPPORTED	    /* 1-pass color quantization? */
 * #undef QUANT_2PASS_SUPPORTED	    /* 2-pass color quantization? */
 */

/* more capability options later, no doubt */


/*
 * Ordering of RGB data in scanlines passed to or from the application.
 * If your application wants to deal with data in the order B,G,R, just
 * change these macros.  You can also deal with formats such as R,G,B,X
 * (one extra byte per pixel) by changing RGB_PIXELSIZE.  Note that changing
 * the offsets will also change the order in which colormap data is organized.
 * RESTRICTIONS:
 * 1. The sample applications cjpeg,djpeg do NOT support modified RGB formats.
 * 2. These macros only affect RGB<=>YCbCr color conversion, so they are not
 *    useful if you are using JPEG color spaces other than YCbCr or grayscale.
 * 3. The color quantizer modules will not behave desirably if RGB_PIXELSIZE
 *    is not 3 (they don't understand about dummy color components!).  So you
 *    can't use color quantization if you change that value.
 */

pub const RGB_RED: u32 = 0;	/* Offset of Red in an RGB scanline element */
pub const RGB_GREEN: u32 = 1;	/* Offset of Green */
pub const RGB_BLUE: u32 = 2;	/* Offset of Blue */
pub const RGB_PIXELSIZE: u32 = 4;	/* JSAMPLEs per RGB scanline element */


/* Definitions for speed-related optimizations. */


/* If your compiler supports inline functions, define INLINE
 * as the inline keyword; otherwise define it as empty.
 * In Rust, #[inline] is used for equivalent effect.
 */

/* On some machines (notably 68000 series) "int" is 32 bits, but multiplying
 * two 16-bit shorts is faster than multiplying two ints.  Define MULTIPLIER
 * as short on such a machine.  MULTIPLIER must be at least 16 bits wide.
 */

pub type MULTIPLIER = c_int;		/* type for fastest integer multiply */


/* FAST_FLOAT should be either float or double, whichever is done faster
 * by your compiler.  (Note that this type is only used in the floating point
 * DCT routines, so it only matters if you've defined DCT_FLOAT_SUPPORTED.)
 * Typically, float is faster in ANSI C compilers, while double is faster in
 * pre-ANSI compilers (because they insist on converting to double anyway).
 * The code below therefore chooses float if we have ANSI-style prototypes.
 */

pub type FAST_FLOAT = f32;

/* #endif /* JPEG_INTERNAL_OPTIONS */
