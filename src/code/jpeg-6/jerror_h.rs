/*
 * jerror.h
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file defines the error and message codes for the JPEG library.
 * Edit this file to add new codes, or to translate the message strings to
 * some other language.
 * A set of error-reporting macros are defined too.  Some applications using
 * the JPEG library may wish to include this file to get the error codes
 * and/or the macros.
 */

/*
 * To define the enum list of message codes, include this file without
 * defining macro JMESSAGE.  To create a message string table, include it
 * again with a suitable JMESSAGE definition (see jerror.c for an example).
 */

use core::ffi::c_int;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum J_MESSAGE_CODE {
    JMSG_NOMESSAGE = 0, /* Bogus message code %d */ /* Must be first entry! */

    /* For maintenance convenience, list is alphabetical by message code name */
    JERR_ARITH_NOTIMPL = 1,
    /* Sorry, there are legal restrictions on arithmetic coding */
    JERR_BAD_ALIGN_TYPE = 2,
    /* ALIGN_TYPE is wrong, please fix */
    JERR_BAD_ALLOC_CHUNK = 3,
    /* MAX_ALLOC_CHUNK is wrong, please fix */
    JERR_BAD_BUFFER_MODE = 4,
    /* Bogus buffer control mode */
    JERR_BAD_COMPONENT_ID = 5,
    /* Invalid component ID %d in SOS */
    JERR_BAD_DCTSIZE = 6,
    /* IDCT output block size %d not supported */
    JERR_BAD_IN_COLORSPACE = 7,
    /* Bogus input colorspace */
    JERR_BAD_J_COLORSPACE = 8,
    /* Bogus JPEG colorspace */
    JERR_BAD_LENGTH = 9,
    /* Bogus marker length */
    JERR_BAD_MCU_SIZE = 10,
    /* Sampling factors too large for interleaved scan */
    JERR_BAD_POOL_ID = 11,
    /* Invalid memory pool code %d */
    JERR_BAD_PRECISION = 12,
    /* Unsupported JPEG data precision %d */
    JERR_BAD_PROGRESSION = 13,
    /* Invalid progressive parameters Ss=%d Se=%d Ah=%d Al=%d */
    JERR_BAD_PROG_SCRIPT = 14,
    /* Invalid progressive parameters at scan script entry %d */
    JERR_BAD_SAMPLING = 15,
    /* Bogus sampling factors */
    JERR_BAD_SCAN_SCRIPT = 16,
    /* Invalid scan script at entry %d */
    JERR_BAD_STATE = 17,
    /* Improper call to JPEG library in state %d */
    JERR_BAD_VIRTUAL_ACCESS = 18,
    /* Bogus virtual array access */
    JERR_BUFFER_SIZE = 19,
    /* Buffer passed to JPEG library is too small */
    JERR_CANT_SUSPEND = 20,
    /* Suspension not allowed here */
    JERR_CCIR601_NOTIMPL = 21,
    /* CCIR601 sampling not implemented yet */
    JERR_COMPONENT_COUNT = 22,
    /* Too many color components: %d, max %d */
    JERR_CONVERSION_NOTIMPL = 23,
    /* Unsupported color conversion request */
    JERR_DAC_INDEX = 24,
    /* Bogus DAC index %d */
    JERR_DAC_VALUE = 25,
    /* Bogus DAC value 0x%x */
    JERR_DHT_COUNTS = 26,
    /* Bogus DHT counts */
    JERR_DHT_INDEX = 27,
    /* Bogus DHT index %d */
    JERR_DQT_INDEX = 28,
    /* Bogus DQT index %d */
    JERR_EMPTY_IMAGE = 29,
    /* Empty JPEG image (DNL not supported) */
    JERR_EMS_READ = 30,
    /* Read from EMS failed */
    JERR_EMS_WRITE = 31,
    /* Write to EMS failed */
    JERR_EOI_EXPECTED = 32,
    /* Didn't expect more than one scan */
    JERR_FILE_READ = 33,
    /* Input file read error */
    JERR_FILE_WRITE = 34,
    /* Output file write error --- out of disk space? */
    JERR_FRACT_SAMPLE_NOTIMPL = 35,
    /* Fractional sampling not implemented yet */
    JERR_HUFF_CLEN_OVERFLOW = 36,
    /* Huffman code size table overflow */
    JERR_HUFF_MISSING_CODE = 37,
    /* Missing Huffman code table entry */
    JERR_IMAGE_TOO_BIG = 38,
    /* Maximum supported image dimension is %u pixels */
    JERR_INPUT_EMPTY = 39,
    /* Empty input file */
    JERR_INPUT_EOF = 40,
    /* Premature end of input file */
    JERR_MISMATCHED_QUANT_TABLE = 41,
    /* Cannot transcode due to multiple use of quantization table %d */
    JERR_MISSING_DATA = 42,
    /* Scan script does not transmit all data */
    JERR_MODE_CHANGE = 43,
    /* Invalid color quantization mode change */
    JERR_NOTIMPL = 44,
    /* Not implemented yet */
    JERR_NOT_COMPILED = 45,
    /* Requested feature was omitted at compile time */
    JERR_NO_BACKING_STORE = 46,
    /* Backing store not supported */
    JERR_NO_HUFF_TABLE = 47,
    /* Huffman table 0x%02x was not defined */
    JERR_NO_IMAGE = 48,
    /* JPEG datastream contains no image */
    JERR_NO_QUANT_TABLE = 49,
    /* Quantization table 0x%02x was not defined */
    JERR_NO_SOI = 50,
    /* Not a JPEG file: starts with 0x%02x 0x%02x */
    JERR_OUT_OF_MEMORY = 51,
    /* Insufficient memory (case %d) */
    JERR_QUANT_COMPONENTS = 52,
    /* Cannot quantize more than %d color components */
    JERR_QUANT_FEW_COLORS = 53,
    /* Cannot quantize to fewer than %d colors */
    JERR_QUANT_MANY_COLORS = 54,
    /* Cannot quantize to more than %d colors */
    JERR_SOF_DUPLICATE = 55,
    /* Invalid JPEG file structure: two SOF markers */
    JERR_SOF_NO_SOS = 56,
    /* Invalid JPEG file structure: missing SOS marker */
    JERR_SOF_UNSUPPORTED = 57,
    /* Unsupported JPEG process: SOF type 0x%02x */
    JERR_SOI_DUPLICATE = 58,
    /* Invalid JPEG file structure: two SOI markers */
    JERR_SOS_NO_SOF = 59,
    /* Invalid JPEG file structure: SOS before SOF */
    JERR_TFILE_CREATE = 60,
    /* Failed to create temporary file %s */
    JERR_TFILE_READ = 61,
    /* Read failed on temporary file */
    JERR_TFILE_SEEK = 62,
    /* Seek failed on temporary file */
    JERR_TFILE_WRITE = 63,
    /* Write failed on temporary file --- out of disk space? */
    JERR_TOO_LITTLE_DATA = 64,
    /* Application transferred too few scanlines */
    JERR_UNKNOWN_MARKER = 65,
    /* Unsupported marker type 0x%02x */
    JERR_VIRTUAL_BUG = 66,
    /* Virtual array controller messed up */
    JERR_WIDTH_OVERFLOW = 67,
    /* Image too wide for this implementation */
    JERR_XMS_READ = 68,
    /* Read from XMS failed */
    JERR_XMS_WRITE = 69,
    /* Write to XMS failed */
    JMSG_COPYRIGHT = 70,
    JMSG_VERSION = 71,
    JTRC_16BIT_TABLES = 72,
    /* Caution: quantization tables are too coarse for baseline JPEG */
    JTRC_ADOBE = 73,
    /* Adobe APP14 marker: version %d, flags 0x%04x 0x%04x, transform %d */
    JTRC_APP0 = 74,
    /* Unknown APP0 marker (not JFIF), length %u */
    JTRC_APP14 = 75,
    /* Unknown APP14 marker (not Adobe), length %u */
    JTRC_DAC = 76,
    /* Define Arithmetic Table 0x%02x: 0x%02x */
    JTRC_DHT = 77,
    /* Define Huffman Table 0x%02x */
    JTRC_DQT = 78,
    /* Define Quantization Table %d  precision %d */
    JTRC_DRI = 79,
    /* Define Restart Interval %u */
    JTRC_EMS_CLOSE = 80,
    /* Freed EMS handle %u */
    JTRC_EMS_OPEN = 81,
    /* Obtained EMS handle %u */
    JTRC_EOI = 82,
    /* End Of Image */
    JTRC_HUFFBITS = 83,
    /*         %3d %3d %3d %3d %3d %3d %3d %3d */
    JTRC_JFIF = 84,
    /* JFIF APP0 marker, density %dx%d  %d */
    JTRC_JFIF_BADTHUMBNAILSIZE = 85,
    /* Warning: thumbnail image size does not match data length %u */
    JTRC_JFIF_MINOR = 86,
    /* Unknown JFIF minor revision number %d.%02d */
    JTRC_JFIF_THUMBNAIL = 87,
    /*     with %d x %d thumbnail image */
    JTRC_MISC_MARKER = 88,
    /* Skipping marker 0x%02x, length %u */
    JTRC_PARMLESS_MARKER = 89,
    /* Unexpected marker 0x%02x */
    JTRC_QUANTVALS = 90,
    /*         %4u %4u %4u %4u %4u %4u %4u %4u */
    JTRC_QUANT_3_NCOLORS = 91,
    /* Quantizing to %d = %d*%d*%d colors */
    JTRC_QUANT_NCOLORS = 92,
    /* Quantizing to %d colors */
    JTRC_QUANT_SELECTED = 93,
    /* Selected %d colors for quantization */
    JTRC_RECOVERY_ACTION = 94,
    /* At marker 0x%02x, recovery action %d */
    JTRC_RST = 95,
    /* RST%d */
    JTRC_SMOOTH_NOTIMPL = 96,
    /* Smoothing not supported with nonstandard sampling ratios */
    JTRC_SOF = 97,
    /* Start Of Frame 0x%02x: width=%u, height=%u, components=%d */
    JTRC_SOF_COMPONENT = 98,
    /*     Component %d: %dhx%dv q=%d */
    JTRC_SOI = 99,
    /* Start of Image */
    JTRC_SOS = 100,
    /* Start Of Scan: %d components */
    JTRC_SOS_COMPONENT = 101,
    /*     Component %d: dc=%d ac=%d */
    JTRC_SOS_PARAMS = 102,
    /*   Ss=%d, Se=%d, Ah=%d, Al=%d */
    JTRC_TFILE_CLOSE = 103,
    /* Closed temporary file %s */
    JTRC_TFILE_OPEN = 104,
    /* Opened temporary file %s */
    JTRC_UNKNOWN_IDS = 105,
    /* Unrecognized component IDs %d %d %d, assuming YCbCr */
    JTRC_XMS_CLOSE = 106,
    /* Freed XMS handle %u */
    JTRC_XMS_OPEN = 107,
    /* Obtained XMS handle %u */
    JWRN_ADOBE_XFORM = 108,
    /* Unknown Adobe color transform code %d */
    JWRN_BOGUS_PROGRESSION = 109,
    /* Inconsistent progression sequence for component %d coefficient %d */
    JWRN_EXTRANEOUS_DATA = 110,
    /* Corrupt JPEG data: %u extraneous bytes before marker 0x%02x */
    JWRN_HIT_MARKER = 111,
    /* Corrupt JPEG data: premature end of data segment */
    JWRN_HUFF_BAD_CODE = 112,
    /* Corrupt JPEG data: bad Huffman code */
    JWRN_JFIF_MAJOR = 113,
    /* Warning: unknown JFIF revision number %d.%02d */
    JWRN_JPEG_EOF = 114,
    /* Premature end of JPEG file */
    JWRN_MUST_RESYNC = 115,
    /* Corrupt JPEG data: found marker 0x%02x instead of RST%d */
    JWRN_NOT_SEQUENTIAL = 116,
    /* Invalid SOS parameters for sequential JPEG */
    JWRN_TOO_MUCH_DATA = 117,
    /* Application transferred too many scanlines */

    JMSG_LASTMSGCODE = 118,
}

/* Macros to simplify using the error and trace message stuff */
/* The first parameter is either type of cinfo pointer */

/* Fatal errors (print message and exit) */
/// ERREXIT sets msg_code and calls error_exit with the cinfo pointer.
/// Equivalent to C macro ERREXIT(cinfo,code).
/// Usage: ERREXIT!(cinfo, JERR_OUT_OF_MEMORY)
#[macro_export]
macro_rules! ERREXIT {
    ($cinfo:expr, $code:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            ((*(*$cinfo).err).error_exit)($cinfo as *mut core::ffi::c_void as *mut j_common_ptr)
        }
    }};
}

/// ERREXIT1 sets msg_code, msg_parm.i[0], and calls error_exit.
/// Equivalent to C macro ERREXIT1(cinfo,code,p1).
#[macro_export]
macro_rules! ERREXIT1 {
    ($cinfo:expr, $code:expr, $p1:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            (*(*$cinfo).err).msg_parm.i[0] = $p1;
            ((*(*$cinfo).err).error_exit)($cinfo as *mut core::ffi::c_void as *mut j_common_ptr)
        }
    }};
}

/// ERREXIT2 sets msg_code, msg_parm.i[0..2], and calls error_exit.
/// Equivalent to C macro ERREXIT2(cinfo,code,p1,p2).
#[macro_export]
macro_rules! ERREXIT2 {
    ($cinfo:expr, $code:expr, $p1:expr, $p2:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            (*(*$cinfo).err).msg_parm.i[0] = $p1;
            (*(*$cinfo).err).msg_parm.i[1] = $p2;
            ((*(*$cinfo).err).error_exit)($cinfo as *mut core::ffi::c_void as *mut j_common_ptr)
        }
    }};
}

/// ERREXIT3 sets msg_code, msg_parm.i[0..3], and calls error_exit.
/// Equivalent to C macro ERREXIT3(cinfo,code,p1,p2,p3).
#[macro_export]
macro_rules! ERREXIT3 {
    ($cinfo:expr, $code:expr, $p1:expr, $p2:expr, $p3:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            (*(*$cinfo).err).msg_parm.i[0] = $p1;
            (*(*$cinfo).err).msg_parm.i[1] = $p2;
            (*(*$cinfo).err).msg_parm.i[2] = $p3;
            ((*(*$cinfo).err).error_exit)($cinfo as *mut core::ffi::c_void as *mut j_common_ptr)
        }
    }};
}

/// ERREXIT4 sets msg_code, msg_parm.i[0..4], and calls error_exit.
/// Equivalent to C macro ERREXIT4(cinfo,code,p1,p2,p3,p4).
#[macro_export]
macro_rules! ERREXIT4 {
    ($cinfo:expr, $code:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            (*(*$cinfo).err).msg_parm.i[0] = $p1;
            (*(*$cinfo).err).msg_parm.i[1] = $p2;
            (*(*$cinfo).err).msg_parm.i[2] = $p3;
            (*(*$cinfo).err).msg_parm.i[3] = $p4;
            ((*(*$cinfo).err).error_exit)($cinfo as *mut core::ffi::c_void as *mut j_common_ptr)
        }
    }};
}

/// ERREXITS sets msg_code, copies string to msg_parm.s, and calls error_exit.
/// Equivalent to C macro ERREXITS(cinfo,code,str).
/// Note: This requires strncpy to be available in scope.
#[macro_export]
macro_rules! ERREXITS {
    ($cinfo:expr, $code:expr, $str:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            // strncpy((cinfo)->err->msg_parm.s, (str), JMSG_STR_PARM_MAX)
            // Port note: Callers must handle string copy via C interop or unsafe code
            let src = $str as *const core::ffi::c_char;
            let dst = (*(*$cinfo).err).msg_parm.s.as_mut_ptr();
            let mut i = 0usize;
            while i < JMSG_STR_PARM_MAX && *src.add(i) != 0 {
                *dst.add(i) = *src.add(i);
                i += 1;
            }
            if i < JMSG_STR_PARM_MAX {
                *dst.add(i) = 0;
            }
            ((*(*$cinfo).err).error_exit)($cinfo as *mut core::ffi::c_void as *mut j_common_ptr)
        }
    }};
}

/// MAKESTMT wraps multiple statements in a do..while(0) loop.
/// In Rust, this becomes a block expression.
/// Equivalent to C macro MAKESTMT(stuff).
#[macro_export]
macro_rules! MAKESTMT {
    ($stuff:expr) => {{
        $stuff
    }};
}

/* Nonfatal errors (we can keep going, but the data is probably corrupt) */
/// WARNMS sets msg_code and calls emit_message with level -1.
/// Equivalent to C macro WARNMS(cinfo,code).
#[macro_export]
macro_rules! WARNMS {
    ($cinfo:expr, $code:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                -1,
            )
        }
    }};
}

/// WARNMS1 sets msg_code, msg_parm.i[0], and calls emit_message with level -1.
/// Equivalent to C macro WARNMS1(cinfo,code,p1).
#[macro_export]
macro_rules! WARNMS1 {
    ($cinfo:expr, $code:expr, $p1:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            (*(*$cinfo).err).msg_parm.i[0] = $p1;
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                -1,
            )
        }
    }};
}

/// WARNMS2 sets msg_code, msg_parm.i[0..2], and calls emit_message with level -1.
/// Equivalent to C macro WARNMS2(cinfo,code,p1,p2).
#[macro_export]
macro_rules! WARNMS2 {
    ($cinfo:expr, $code:expr, $p1:expr, $p2:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            (*(*$cinfo).err).msg_parm.i[0] = $p1;
            (*(*$cinfo).err).msg_parm.i[1] = $p2;
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                -1,
            )
        }
    }};
}

/* Informational/debugging messages */
/// TRACEMS sets msg_code and calls emit_message with the given level.
/// Equivalent to C macro TRACEMS(cinfo,lvl,code).
#[macro_export]
macro_rules! TRACEMS {
    ($cinfo:expr, $lvl:expr, $code:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                $lvl,
            )
        }
    }};
}

/// TRACEMS1 sets msg_code, msg_parm.i[0], and calls emit_message.
/// Equivalent to C macro TRACEMS1(cinfo,lvl,code,p1).
#[macro_export]
macro_rules! TRACEMS1 {
    ($cinfo:expr, $lvl:expr, $code:expr, $p1:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            (*(*$cinfo).err).msg_parm.i[0] = $p1;
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                $lvl,
            )
        }
    }};
}

/// TRACEMS2 sets msg_code, msg_parm.i[0..2], and calls emit_message.
/// Equivalent to C macro TRACEMS2(cinfo,lvl,code,p1,p2).
#[macro_export]
macro_rules! TRACEMS2 {
    ($cinfo:expr, $lvl:expr, $code:expr, $p1:expr, $p2:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            (*(*$cinfo).err).msg_parm.i[0] = $p1;
            (*(*$cinfo).err).msg_parm.i[1] = $p2;
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                $lvl,
            )
        }
    }};
}

/// TRACEMS3 sets msg_parm.i[0..3], msg_code, and calls emit_message.
/// Equivalent to C macro TRACEMS3(cinfo,lvl,code,p1,p2,p3).
#[macro_export]
macro_rules! TRACEMS3 {
    ($cinfo:expr, $lvl:expr, $code:expr, $p1:expr, $p2:expr, $p3:expr) => {{
        unsafe {
            let _mp = (*(*$cinfo).err).msg_parm.i.as_mut_ptr();
            *_mp.add(0) = $p1;
            *_mp.add(1) = $p2;
            *_mp.add(2) = $p3;
            (*(*$cinfo).err).msg_code = $code;
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                $lvl,
            )
        }
    }};
}

/// TRACEMS4 sets msg_parm.i[0..4], msg_code, and calls emit_message.
/// Equivalent to C macro TRACEMS4(cinfo,lvl,code,p1,p2,p3,p4).
#[macro_export]
macro_rules! TRACEMS4 {
    ($cinfo:expr, $lvl:expr, $code:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr) => {{
        unsafe {
            let _mp = (*(*$cinfo).err).msg_parm.i.as_mut_ptr();
            *_mp.add(0) = $p1;
            *_mp.add(1) = $p2;
            *_mp.add(2) = $p3;
            *_mp.add(3) = $p4;
            (*(*$cinfo).err).msg_code = $code;
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                $lvl,
            )
        }
    }};
}

/// TRACEMS8 sets msg_parm.i[0..8], msg_code, and calls emit_message.
/// Equivalent to C macro TRACEMS8(cinfo,lvl,code,p1,p2,p3,p4,p5,p6,p7,p8).
#[macro_export]
macro_rules! TRACEMS8 {
    ($cinfo:expr, $lvl:expr, $code:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr) => {{
        unsafe {
            let _mp = (*(*$cinfo).err).msg_parm.i.as_mut_ptr();
            *_mp.add(0) = $p1;
            *_mp.add(1) = $p2;
            *_mp.add(2) = $p3;
            *_mp.add(3) = $p4;
            *_mp.add(4) = $p5;
            *_mp.add(5) = $p6;
            *_mp.add(6) = $p7;
            *_mp.add(7) = $p8;
            (*(*$cinfo).err).msg_code = $code;
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                $lvl,
            )
        }
    }};
}

/// TRACEMSS sets msg_code, copies string to msg_parm.s, and calls emit_message.
/// Equivalent to C macro TRACEMSS(cinfo,lvl,code,str).
#[macro_export]
macro_rules! TRACEMSS {
    ($cinfo:expr, $lvl:expr, $code:expr, $str:expr) => {{
        unsafe {
            (*(*$cinfo).err).msg_code = $code;
            // strncpy((cinfo)->err->msg_parm.s, (str), JMSG_STR_PARM_MAX)
            let src = $str as *const core::ffi::c_char;
            let dst = (*(*$cinfo).err).msg_parm.s.as_mut_ptr();
            let mut i = 0usize;
            while i < JMSG_STR_PARM_MAX && *src.add(i) != 0 {
                *dst.add(i) = *src.add(i);
                i += 1;
            }
            if i < JMSG_STR_PARM_MAX {
                *dst.add(i) = 0;
            }
            ((*(*$cinfo).err).emit_message)(
                $cinfo as *mut core::ffi::c_void as *mut j_common_ptr,
                $lvl,
            )
        }
    }};
}

// Port note: JMSG_STR_PARM_MAX and j_common_ptr are defined in jmorecfg.h.
// Declared here as stubs for structural coherence of this module.
pub const JMSG_STR_PARM_MAX: usize = 80; // Placeholder; adjust as needed per jmorecfg.h

#[repr(C)]
pub struct j_common_ptr {
    // Placeholder stub; actual definition comes from j_common in jmorecfg.h
}
