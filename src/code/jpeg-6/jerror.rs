/*
 * jerror.c
 *
 * Copyright (C) 1991-1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains simple error-reporting and trace-message routines.
 * These are suitable for Unix-like systems and others where writing to
 * stderr is the right thing to do.  Many applications will want to replace
 * some or all of these routines.
 *
 * These routines are used by both the compression and decompression code.
 */

use core::ffi::{c_int, c_char};

// Local stubs for types not yet fully ported
pub type boolean = c_int;
pub type UINT8 = u8;

const JMSG_LENGTH_MAX: usize = 200;
const JMSG_STR_PARM_MAX: usize = 80;

/// Opaque type stubs for JPEG error management
#[repr(C)]
pub struct jpeg_error_mgr {
	pub error_exit: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
	pub emit_message: Option<unsafe extern "C" fn(*mut core::ffi::c_void, c_int)>,
	pub output_message: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
	pub format_message: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut c_char)>,
	pub reset_error_mgr: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
	pub trace_level: c_int,
	pub num_warnings: c_int,
	pub msg_code: c_int,
	pub msg_parm: msg_parm_union,
	pub jpeg_message_table: *const *const c_char,
	pub last_jpeg_message: c_int,
	pub addon_message_table: *const *const c_char,
	pub first_addon_message: c_int,
	pub last_addon_message: c_int,
}

#[repr(C)]
pub union msg_parm_union {
	pub i: [c_int; 8],
	pub s: [c_char; JMSG_STR_PARM_MAX],
}

#[repr(C)]
pub struct jpeg_common_struct {
	pub err: *mut jpeg_error_mgr,
	// Additional fields omitted for brevity
}

pub type j_common_ptr = *mut jpeg_common_struct;

extern "C" {
	pub fn jpeg_destroy(cinfo: j_common_ptr);
	pub fn Com_Error(error_code: c_int, format: *const c_char, ...);
	pub fn VID_Printf(print_type: c_int, format: *const c_char, ...);
}

/*
 * Create the message string table.
 * We do this from the master message list in jerror.h by re-reading
 * jerror.h with a suitable definition for macro JMESSAGE.
 * The message table is made an external symbol just in case any applications
 * want to refer to it directly.
 */

/* Message string table - array of C strings */
static MSG_0: &[u8] = b"Bogus message code %d\0";  /* JMSG_NOMESSAGE - Must be first entry! */
static MSG_1: &[u8] = b"Sorry, there are legal restrictions on arithmetic coding\0";  /* JERR_ARITH_NOTIMPL */
static MSG_2: &[u8] = b"ALIGN_TYPE is wrong, please fix\0";  /* JERR_BAD_ALIGN_TYPE */
static MSG_3: &[u8] = b"MAX_ALLOC_CHUNK is wrong, please fix\0";  /* JERR_BAD_ALLOC_CHUNK */
static MSG_4: &[u8] = b"Bogus buffer control mode\0";  /* JERR_BAD_BUFFER_MODE */
static MSG_5: &[u8] = b"Invalid component ID %d in SOS\0";  /* JERR_BAD_COMPONENT_ID */
static MSG_6: &[u8] = b"IDCT output block size %d not supported\0";  /* JERR_BAD_DCTSIZE */
static MSG_7: &[u8] = b"Bogus input colorspace\0";  /* JERR_BAD_IN_COLORSPACE */
static MSG_8: &[u8] = b"Bogus JPEG colorspace\0";  /* JERR_BAD_J_COLORSPACE */
static MSG_9: &[u8] = b"Bogus marker length\0";  /* JERR_BAD_LENGTH */
static MSG_10: &[u8] = b"Sampling factors too large for interleaved scan\0";  /* JERR_BAD_MCU_SIZE */
static MSG_11: &[u8] = b"Invalid memory pool code %d\0";  /* JERR_BAD_POOL_ID */
static MSG_12: &[u8] = b"Unsupported JPEG data precision %d\0";  /* JERR_BAD_PRECISION */
static MSG_13: &[u8] = b"Invalid progressive parameters Ss=%d Se=%d Ah=%d Al=%d\0";  /* JERR_BAD_PROGRESSION */
static MSG_14: &[u8] = b"Invalid progressive parameters at scan script entry %d\0";  /* JERR_BAD_PROG_SCRIPT */
static MSG_15: &[u8] = b"Bogus sampling factors\0";  /* JERR_BAD_SAMPLING */
static MSG_16: &[u8] = b"Invalid scan script at entry %d\0";  /* JERR_BAD_SCAN_SCRIPT */
static MSG_17: &[u8] = b"Improper call to JPEG library in state %d\0";  /* JERR_BAD_STATE */
static MSG_18: &[u8] = b"Bogus virtual array access\0";  /* JERR_BAD_VIRTUAL_ACCESS */
static MSG_19: &[u8] = b"Buffer passed to JPEG library is too small\0";  /* JERR_BUFFER_SIZE */
static MSG_20: &[u8] = b"Suspension not allowed here\0";  /* JERR_CANT_SUSPEND */
static MSG_21: &[u8] = b"CCIR601 sampling not implemented yet\0";  /* JERR_CCIR601_NOTIMPL */
static MSG_22: &[u8] = b"Too many color components: %d, max %d\0";  /* JERR_COMPONENT_COUNT */
static MSG_23: &[u8] = b"Unsupported color conversion request\0";  /* JERR_CONVERSION_NOTIMPL */
static MSG_24: &[u8] = b"Bogus DAC index %d\0";  /* JERR_DAC_INDEX */
static MSG_25: &[u8] = b"Bogus DAC value 0x%x\0";  /* JERR_DAC_VALUE */
static MSG_26: &[u8] = b"Bogus DHT counts\0";  /* JERR_DHT_COUNTS */
static MSG_27: &[u8] = b"Bogus DHT index %d\0";  /* JERR_DHT_INDEX */
static MSG_28: &[u8] = b"Bogus DQT index %d\0";  /* JERR_DQT_INDEX */
static MSG_29: &[u8] = b"Empty JPEG image (DNL not supported)\0";  /* JERR_EMPTY_IMAGE */
static MSG_30: &[u8] = b"Read from EMS failed\0";  /* JERR_EMS_READ */
static MSG_31: &[u8] = b"Write to EMS failed\0";  /* JERR_EMS_WRITE */
static MSG_32: &[u8] = b"Didn't expect more than one scan\0";  /* JERR_EOI_EXPECTED */
static MSG_33: &[u8] = b"Input file read error\0";  /* JERR_FILE_READ */
static MSG_34: &[u8] = b"Output file write error --- out of disk space?\0";  /* JERR_FILE_WRITE */
static MSG_35: &[u8] = b"Fractional sampling not implemented yet\0";  /* JERR_FRACT_SAMPLE_NOTIMPL */
static MSG_36: &[u8] = b"Huffman code size table overflow\0";  /* JERR_HUFF_CLEN_OVERFLOW */
static MSG_37: &[u8] = b"Missing Huffman code table entry\0";  /* JERR_HUFF_MISSING_CODE */
static MSG_38: &[u8] = b"Maximum supported image dimension is %u pixels\0";  /* JERR_IMAGE_TOO_BIG */
static MSG_39: &[u8] = b"Empty input file\0";  /* JERR_INPUT_EMPTY */
static MSG_40: &[u8] = b"Premature end of input file\0";  /* JERR_INPUT_EOF */
static MSG_41: &[u8] = b"Cannot transcode due to multiple use of quantization table %d\0";  /* JERR_MISMATCHED_QUANT_TABLE */
static MSG_42: &[u8] = b"Scan script does not transmit all data\0";  /* JERR_MISSING_DATA */
static MSG_43: &[u8] = b"Invalid color quantization mode change\0";  /* JERR_MODE_CHANGE */
static MSG_44: &[u8] = b"Not implemented yet\0";  /* JERR_NOTIMPL */
static MSG_45: &[u8] = b"Requested feature was omitted at compile time\0";  /* JERR_NOT_COMPILED */
static MSG_46: &[u8] = b"Backing store not supported\0";  /* JERR_NO_BACKING_STORE */
static MSG_47: &[u8] = b"Huffman table 0x%02x was not defined\0";  /* JERR_NO_HUFF_TABLE */
static MSG_48: &[u8] = b"JPEG datastream contains no image\0";  /* JERR_NO_IMAGE */
static MSG_49: &[u8] = b"Quantization table 0x%02x was not defined\0";  /* JERR_NO_QUANT_TABLE */
static MSG_50: &[u8] = b"Not a JPEG file: starts with 0x%02x 0x%02x\0";  /* JERR_NO_SOI */
static MSG_51: &[u8] = b"Insufficient memory (case %d)\0";  /* JERR_OUT_OF_MEMORY */
static MSG_52: &[u8] = b"Cannot quantize more than %d color components\0";  /* JERR_QUANT_COMPONENTS */
static MSG_53: &[u8] = b"Cannot quantize to fewer than %d colors\0";  /* JERR_QUANT_FEW_COLORS */
static MSG_54: &[u8] = b"Cannot quantize to more than %d colors\0";  /* JERR_QUANT_MANY_COLORS */
static MSG_55: &[u8] = b"Invalid JPEG file structure: two SOF markers\0";  /* JERR_SOF_DUPLICATE */
static MSG_56: &[u8] = b"Invalid JPEG file structure: missing SOS marker\0";  /* JERR_SOF_NO_SOS */
static MSG_57: &[u8] = b"Unsupported JPEG process: SOF type 0x%02x\0";  /* JERR_SOF_UNSUPPORTED */
static MSG_58: &[u8] = b"Invalid JPEG file structure: two SOI markers\0";  /* JERR_SOI_DUPLICATE */
static MSG_59: &[u8] = b"Invalid JPEG file structure: SOS before SOF\0";  /* JERR_SOS_NO_SOF */
static MSG_60: &[u8] = b"Failed to create temporary file %s\0";  /* JERR_TFILE_CREATE */
static MSG_61: &[u8] = b"Read failed on temporary file\0";  /* JERR_TFILE_READ */
static MSG_62: &[u8] = b"Seek failed on temporary file\0";  /* JERR_TFILE_SEEK */
static MSG_63: &[u8] = b"Write failed on temporary file --- out of disk space?\0";  /* JERR_TFILE_WRITE */
static MSG_64: &[u8] = b"Application transferred too few scanlines\0";  /* JERR_TOO_LITTLE_DATA */
static MSG_65: &[u8] = b"Unsupported marker type 0x%02x\0";  /* JERR_UNKNOWN_MARKER */
static MSG_66: &[u8] = b"Virtual array controller messed up\0";  /* JERR_VIRTUAL_BUG */
static MSG_67: &[u8] = b"Image too wide for this implementation\0";  /* JERR_WIDTH_OVERFLOW */
static MSG_68: &[u8] = b"Read from XMS failed\0";  /* JERR_XMS_READ */
static MSG_69: &[u8] = b"Write to XMS failed\0";  /* JERR_XMS_WRITE */
static MSG_70: &[u8] = b"JCOPYRIGHT placeholder\0";  /* JMSG_COPYRIGHT */
static MSG_71: &[u8] = b"JVERSION placeholder\0";  /* JMSG_VERSION */
static MSG_72: &[u8] = b"Caution: quantization tables are too coarse for baseline JPEG\0";  /* JTRC_16BIT_TABLES */
static MSG_73: &[u8] = b"Adobe APP14 marker: version %d, flags 0x%04x 0x%04x, transform %d\0";  /* JTRC_ADOBE */
static MSG_74: &[u8] = b"Unknown APP0 marker (not JFIF), length %u\0";  /* JTRC_APP0 */
static MSG_75: &[u8] = b"Unknown APP14 marker (not Adobe), length %u\0";  /* JTRC_APP14 */
static MSG_76: &[u8] = b"Define Arithmetic Table 0x%02x: 0x%02x\0";  /* JTRC_DAC */
static MSG_77: &[u8] = b"Define Huffman Table 0x%02x\0";  /* JTRC_DHT */
static MSG_78: &[u8] = b"Define Quantization Table %d  precision %d\0";  /* JTRC_DQT */
static MSG_79: &[u8] = b"Define Restart Interval %u\0";  /* JTRC_DRI */
static MSG_80: &[u8] = b"Freed EMS handle %u\0";  /* JTRC_EMS_CLOSE */
static MSG_81: &[u8] = b"Obtained EMS handle %u\0";  /* JTRC_EMS_OPEN */
static MSG_82: &[u8] = b"End Of Image\0";  /* JTRC_EOI */
static MSG_83: &[u8] = b"        %3d %3d %3d %3d %3d %3d %3d %3d\0";  /* JTRC_HUFFBITS */
static MSG_84: &[u8] = b"JFIF APP0 marker, density %dx%d  %d\0";  /* JTRC_JFIF */
static MSG_85: &[u8] = b"Warning: thumbnail image size does not match data length %u\0";  /* JTRC_JFIF_BADTHUMBNAILSIZE */
static MSG_86: &[u8] = b"Unknown JFIF minor revision number %d.%02d\0";  /* JTRC_JFIF_MINOR */
static MSG_87: &[u8] = b"    with %d x %d thumbnail image\0";  /* JTRC_JFIF_THUMBNAIL */
static MSG_88: &[u8] = b"Skipping marker 0x%02x, length %u\0";  /* JTRC_MISC_MARKER */
static MSG_89: &[u8] = b"Unexpected marker 0x%02x\0";  /* JTRC_PARMLESS_MARKER */
static MSG_90: &[u8] = b"        %4u %4u %4u %4u %4u %4u %4u %4u\0";  /* JTRC_QUANTVALS */
static MSG_91: &[u8] = b"Quantizing to %d = %d*%d*%d colors\0";  /* JTRC_QUANT_3_NCOLORS */
static MSG_92: &[u8] = b"Quantizing to %d colors\0";  /* JTRC_QUANT_NCOLORS */
static MSG_93: &[u8] = b"Selected %d colors for quantization\0";  /* JTRC_QUANT_SELECTED */
static MSG_94: &[u8] = b"At marker 0x%02x, recovery action %d\0";  /* JTRC_RECOVERY_ACTION */
static MSG_95: &[u8] = b"RST%d\0";  /* JTRC_RST */
static MSG_96: &[u8] = b"Smoothing not supported with nonstandard sampling ratios\0";  /* JTRC_SMOOTH_NOTIMPL */
static MSG_97: &[u8] = b"Start Of Frame 0x%02x: width=%u, height=%u, components=%d\0";  /* JTRC_SOF */
static MSG_98: &[u8] = b"    Component %d: %dhx%dv q=%d\0";  /* JTRC_SOF_COMPONENT */
static MSG_99: &[u8] = b"Start of Image\0";  /* JTRC_SOI */
static MSG_100: &[u8] = b"Start Of Scan: %d components\0";  /* JTRC_SOS */
static MSG_101: &[u8] = b"    Component %d: dc=%d ac=%d\0";  /* JTRC_SOS_COMPONENT */
static MSG_102: &[u8] = b"  Ss=%d, Se=%d, Ah=%d, Al=%d\0";  /* JTRC_SOS_PARAMS */
static MSG_103: &[u8] = b"Closed temporary file %s\0";  /* JTRC_TFILE_CLOSE */
static MSG_104: &[u8] = b"Opened temporary file %s\0";  /* JTRC_TFILE_OPEN */
static MSG_105: &[u8] = b"Unrecognized component IDs %d %d %d, assuming YCbCr\0";  /* JTRC_UNKNOWN_IDS */
static MSG_106: &[u8] = b"Freed XMS handle %u\0";  /* JTRC_XMS_CLOSE */
static MSG_107: &[u8] = b"Obtained XMS handle %u\0";  /* JTRC_XMS_OPEN */
static MSG_108: &[u8] = b"Unknown Adobe color transform code %d\0";  /* JWRN_ADOBE_XFORM */
static MSG_109: &[u8] = b"Inconsistent progression sequence for component %d coefficient %d\0";  /* JWRN_BOGUS_PROGRESSION */
static MSG_110: &[u8] = b"Corrupt JPEG data: %u extraneous bytes before marker 0x%02x\0";  /* JWRN_EXTRANEOUS_DATA */
static MSG_111: &[u8] = b"Corrupt JPEG data: premature end of data segment\0";  /* JWRN_HIT_MARKER */
static MSG_112: &[u8] = b"Corrupt JPEG data: bad Huffman code\0";  /* JWRN_HUFF_BAD_CODE */
static MSG_113: &[u8] = b"Warning: unknown JFIF revision number %d.%02d\0";  /* JWRN_JFIF_MAJOR */
static MSG_114: &[u8] = b"Premature end of JPEG file\0";  /* JWRN_JPEG_EOF */
static MSG_115: &[u8] = b"Corrupt JPEG data: found marker 0x%02x instead of RST%d\0";  /* JWRN_MUST_RESYNC */
static MSG_116: &[u8] = b"Invalid SOS parameters for sequential JPEG\0";  /* JWRN_NOT_SEQUENTIAL */
static MSG_117: &[u8] = b"Application transferred too many scanlines\0";  /* JWRN_TOO_MUCH_DATA */

pub static jpeg_std_message_table: &[*const c_char] = &[
	MSG_0.as_ptr() as *const c_char,
	MSG_1.as_ptr() as *const c_char,
	MSG_2.as_ptr() as *const c_char,
	MSG_3.as_ptr() as *const c_char,
	MSG_4.as_ptr() as *const c_char,
	MSG_5.as_ptr() as *const c_char,
	MSG_6.as_ptr() as *const c_char,
	MSG_7.as_ptr() as *const c_char,
	MSG_8.as_ptr() as *const c_char,
	MSG_9.as_ptr() as *const c_char,
	MSG_10.as_ptr() as *const c_char,
	MSG_11.as_ptr() as *const c_char,
	MSG_12.as_ptr() as *const c_char,
	MSG_13.as_ptr() as *const c_char,
	MSG_14.as_ptr() as *const c_char,
	MSG_15.as_ptr() as *const c_char,
	MSG_16.as_ptr() as *const c_char,
	MSG_17.as_ptr() as *const c_char,
	MSG_18.as_ptr() as *const c_char,
	MSG_19.as_ptr() as *const c_char,
	MSG_20.as_ptr() as *const c_char,
	MSG_21.as_ptr() as *const c_char,
	MSG_22.as_ptr() as *const c_char,
	MSG_23.as_ptr() as *const c_char,
	MSG_24.as_ptr() as *const c_char,
	MSG_25.as_ptr() as *const c_char,
	MSG_26.as_ptr() as *const c_char,
	MSG_27.as_ptr() as *const c_char,
	MSG_28.as_ptr() as *const c_char,
	MSG_29.as_ptr() as *const c_char,
	MSG_30.as_ptr() as *const c_char,
	MSG_31.as_ptr() as *const c_char,
	MSG_32.as_ptr() as *const c_char,
	MSG_33.as_ptr() as *const c_char,
	MSG_34.as_ptr() as *const c_char,
	MSG_35.as_ptr() as *const c_char,
	MSG_36.as_ptr() as *const c_char,
	MSG_37.as_ptr() as *const c_char,
	MSG_38.as_ptr() as *const c_char,
	MSG_39.as_ptr() as *const c_char,
	MSG_40.as_ptr() as *const c_char,
	MSG_41.as_ptr() as *const c_char,
	MSG_42.as_ptr() as *const c_char,
	MSG_43.as_ptr() as *const c_char,
	MSG_44.as_ptr() as *const c_char,
	MSG_45.as_ptr() as *const c_char,
	MSG_46.as_ptr() as *const c_char,
	MSG_47.as_ptr() as *const c_char,
	MSG_48.as_ptr() as *const c_char,
	MSG_49.as_ptr() as *const c_char,
	MSG_50.as_ptr() as *const c_char,
	MSG_51.as_ptr() as *const c_char,
	MSG_52.as_ptr() as *const c_char,
	MSG_53.as_ptr() as *const c_char,
	MSG_54.as_ptr() as *const c_char,
	MSG_55.as_ptr() as *const c_char,
	MSG_56.as_ptr() as *const c_char,
	MSG_57.as_ptr() as *const c_char,
	MSG_58.as_ptr() as *const c_char,
	MSG_59.as_ptr() as *const c_char,
	MSG_60.as_ptr() as *const c_char,
	MSG_61.as_ptr() as *const c_char,
	MSG_62.as_ptr() as *const c_char,
	MSG_63.as_ptr() as *const c_char,
	MSG_64.as_ptr() as *const c_char,
	MSG_65.as_ptr() as *const c_char,
	MSG_66.as_ptr() as *const c_char,
	MSG_67.as_ptr() as *const c_char,
	MSG_68.as_ptr() as *const c_char,
	MSG_69.as_ptr() as *const c_char,
	MSG_70.as_ptr() as *const c_char,
	MSG_71.as_ptr() as *const c_char,
	MSG_72.as_ptr() as *const c_char,
	MSG_73.as_ptr() as *const c_char,
	MSG_74.as_ptr() as *const c_char,
	MSG_75.as_ptr() as *const c_char,
	MSG_76.as_ptr() as *const c_char,
	MSG_77.as_ptr() as *const c_char,
	MSG_78.as_ptr() as *const c_char,
	MSG_79.as_ptr() as *const c_char,
	MSG_80.as_ptr() as *const c_char,
	MSG_81.as_ptr() as *const c_char,
	MSG_82.as_ptr() as *const c_char,
	MSG_83.as_ptr() as *const c_char,
	MSG_84.as_ptr() as *const c_char,
	MSG_85.as_ptr() as *const c_char,
	MSG_86.as_ptr() as *const c_char,
	MSG_87.as_ptr() as *const c_char,
	MSG_88.as_ptr() as *const c_char,
	MSG_89.as_ptr() as *const c_char,
	MSG_90.as_ptr() as *const c_char,
	MSG_91.as_ptr() as *const c_char,
	MSG_92.as_ptr() as *const c_char,
	MSG_93.as_ptr() as *const c_char,
	MSG_94.as_ptr() as *const c_char,
	MSG_95.as_ptr() as *const c_char,
	MSG_96.as_ptr() as *const c_char,
	MSG_97.as_ptr() as *const c_char,
	MSG_98.as_ptr() as *const c_char,
	MSG_99.as_ptr() as *const c_char,
	MSG_100.as_ptr() as *const c_char,
	MSG_101.as_ptr() as *const c_char,
	MSG_102.as_ptr() as *const c_char,
	MSG_103.as_ptr() as *const c_char,
	MSG_104.as_ptr() as *const c_char,
	MSG_105.as_ptr() as *const c_char,
	MSG_106.as_ptr() as *const c_char,
	MSG_107.as_ptr() as *const c_char,
	MSG_108.as_ptr() as *const c_char,
	MSG_109.as_ptr() as *const c_char,
	MSG_110.as_ptr() as *const c_char,
	MSG_111.as_ptr() as *const c_char,
	MSG_112.as_ptr() as *const c_char,
	MSG_113.as_ptr() as *const c_char,
	MSG_114.as_ptr() as *const c_char,
	MSG_115.as_ptr() as *const c_char,
	MSG_116.as_ptr() as *const c_char,
	MSG_117.as_ptr() as *const c_char,
];

/*
 * Error exit handler: must not return to caller.
 *
 * Applications may override this if they want to get control back after
 * an error.  Typically one would longjmp somewhere instead of exiting.
 * The setjmp buffer can be made a private field within an expanded error
 * handler object.  Note that the info needed to generate an error message
 * is stored in the error object, so you can generate the message now or
 * later, at your convenience.
 * You should make sure that the JPEG object is cleaned up (with jpeg_abort
 * or jpeg_destroy) at some point.
 */

unsafe extern "C" fn error_exit(cinfo: j_common_ptr) {
	let mut buffer: [c_char; JMSG_LENGTH_MAX] = [0; JMSG_LENGTH_MAX];

	/* Create the message */
	if let Some(format_message_fn) = (*(*cinfo).err).format_message {
		format_message_fn(cinfo, buffer.as_mut_ptr());
	}

	/* Let the memory manager delete any temp files before we die */
	jpeg_destroy(cinfo);

	Com_Error(3, b"%s\n\0".as_ptr() as *const c_char, buffer.as_ptr());
}


/*
 * Actual output of an error or trace message.
 * Applications may override this method to send JPEG messages somewhere
 * other than stderr.
 */

unsafe extern "C" fn output_message(cinfo: j_common_ptr) {
	let mut buffer: [c_char; JMSG_LENGTH_MAX] = [0; JMSG_LENGTH_MAX];

	/* Create the message */
	if let Some(format_message_fn) = (*(*cinfo).err).format_message {
		format_message_fn(cinfo, buffer.as_mut_ptr());
	}

	/* Send it to stderr, adding a newline */
	VID_Printf(1, b"%s\n\0".as_ptr() as *const c_char, buffer.as_ptr());
}


/*
 * Decide whether to emit a trace or warning message.
 * msg_level is one of:
 *   -1: recoverable corrupt-data warning, may want to abort.
 *    0: important advisory messages (always display to user).
 *    1: first level of tracing detail.
 *    2,3,...: successively more detailed tracing messages.
 * An application might override this method if it wanted to abort on warnings
 * or change the policy about which messages to display.
 */

unsafe extern "C" fn emit_message(cinfo: j_common_ptr, msg_level: c_int) {
	let err = (*cinfo).err;

	if msg_level < 0 {
		/* It's a warning message.  Since corrupt files may generate many warnings,
		 * the policy implemented here is to show only the first warning,
		 * unless trace_level >= 3.
		 */
		if (*err).num_warnings == 0 || (*err).trace_level >= 3 {
			if let Some(output_message) = (*err).output_message {
				output_message(cinfo);
			}
		}
		/* Always count warnings in num_warnings. */
		(*err).num_warnings += 1;
	} else {
		/* It's a trace message.  Show it if trace_level >= msg_level. */
		if (*err).trace_level >= msg_level {
			if let Some(output_message) = (*err).output_message {
				output_message(cinfo);
			}
		}
	}
}


/*
 * Format a message string for the most recent JPEG error or message.
 * The message is stored into buffer, which should be at least JMSG_LENGTH_MAX
 * characters.  Note that no '\n' character is added to the string.
 * Few applications should need to override this method.
 */

unsafe extern "C" fn format_message(cinfo: j_common_ptr, buffer: *mut c_char) {
	let err = (*cinfo).err;
	let msg_code = (*err).msg_code;
	let mut msgtext: *const c_char = core::ptr::null();
	let mut msgptr: *const c_char;
	let mut ch: c_char;
	let mut isstring: boolean = 0;

	/* Look up message string in proper table */
	if msg_code > 0 && msg_code <= (*err).last_jpeg_message {
		if !(*err).jpeg_message_table.is_null() {
			msgtext = *(*err).jpeg_message_table.offset(msg_code as isize);
		}
	} else if !(*err).addon_message_table.is_null() &&
		msg_code >= (*err).first_addon_message &&
		msg_code <= (*err).last_addon_message {
		msgtext = *(*err).addon_message_table.offset((msg_code - (*err).first_addon_message) as isize);
	}

	/* Defend against bogus message number */
	if msgtext.is_null() {
		(*err).msg_parm.i[0] = msg_code;
		if !(*err).jpeg_message_table.is_null() {
			msgtext = *(*err).jpeg_message_table.offset(0);
		}
	}

	/* Check for string parameter, as indicated by %s in the message text */
	isstring = 0;
	msgptr = msgtext;
	if !msgptr.is_null() {
		loop {
			ch = *msgptr;
			msgptr = msgptr.offset(1);
			if ch == 0 {
				break;
			}
			if ch == b'%' as c_char {
				if *msgptr == b's' as c_char {
					isstring = 1;
				}
				break;
			}
		}
	}

	/* Format the message into the passed buffer */
	if isstring != 0 {
		sprintf(buffer, msgtext, (*err).msg_parm.s.as_ptr());
	} else {
		sprintf(
			buffer,
			msgtext,
			(*err).msg_parm.i[0],
			(*err).msg_parm.i[1],
			(*err).msg_parm.i[2],
			(*err).msg_parm.i[3],
			(*err).msg_parm.i[4],
			(*err).msg_parm.i[5],
			(*err).msg_parm.i[6],
			(*err).msg_parm.i[7],
		);
	}
}

extern "C" {
	pub fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
}

/*
 * Reset error state variables at start of a new image.
 * This is called during compression startup to reset trace/error
 * processing to default state, without losing any application-specific
 * method pointers.  An application might possibly want to override
 * this method if it has additional error processing state.
 */

unsafe extern "C" fn reset_error_mgr(cinfo: j_common_ptr) {
	(*(*cinfo).err).num_warnings = 0;
	/* trace_level is not reset since it is an application-supplied parameter */
	(*(*cinfo).err).msg_code = 0;	/* may be useful as a flag for "no error" */
}


/*
 * Fill in the standard error-handling methods in a jpeg_error_mgr object.
 * Typical call is:
 *	struct jpeg_compress_struct cinfo;
 *	struct jpeg_error_mgr err;
 *
 *	cinfo.err = jpeg_std_error(&err);
 * after which the application may override some of the methods.
 */

pub unsafe extern "C" fn jpeg_std_error(err: *mut jpeg_error_mgr) -> *mut jpeg_error_mgr {
	(*err).error_exit = Some(error_exit);
	(*err).emit_message = Some(emit_message);
	(*err).output_message = Some(output_message);
	(*err).format_message = Some(format_message);
	(*err).reset_error_mgr = Some(reset_error_mgr);

	(*err).trace_level = 0;		/* default = no tracing */
	(*err).num_warnings = 0;	/* no warnings emitted yet */
	(*err).msg_code = 0;		/* may be useful as a flag for "no error" */

	/* Initialize message table pointers */
	(*err).jpeg_message_table = jpeg_std_message_table.as_ptr();
	(*err).last_jpeg_message = 117;	/* JMSG_LASTMSGCODE - 1 */

	(*err).addon_message_table = core::ptr::null();
	(*err).first_addon_message = 0;	/* for safety */
	(*err).last_addon_message = 0;

	err
}
