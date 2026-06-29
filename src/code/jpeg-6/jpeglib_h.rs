/*
 * jpeglib.h
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file defines the application interface for the JPEG library.
 * Most applications using the library need only include this file,
 * and perhaps jerror.h if they want to know the exact error codes.
 */

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_void, c_char, c_int};
use std::os::raw::c_ulong;

/* Local type stubs for jconfig.h and jmorecfg.h dependencies.
 * These types would normally come from the JPEG library configuration headers.
 */
pub type JSAMPLE = u8;
pub type JCOEF = i16;
pub type UINT8 = u8;
pub type UINT16 = u16;
pub type JDIMENSION = u32;
pub type JOCTET = u8;
pub type boolean = u8;

/*
 * First we include the configuration files that record how this
 * installation of the JPEG library is set up.  jconfig.h can be
 * generated automatically for many systems.  jmorecfg.h contains
 * manual configuration options that most people need not worry about.
 */

/* Version ID for the JPEG library.
 * Might be useful for tests like "#if JPEG_LIB_VERSION >= 60".
 */

pub const JPEG_LIB_VERSION: c_int = 60;  /* Version 6 */


/* Various constants determining the sizes of things.
 * All of these are specified by the JPEG standard, so don't change them
 * if you want to be compatible.
 */

pub const DCTSIZE: c_int = 8;           /* The basic DCT block is 8x8 samples */
pub const DCTSIZE2: c_int = 64;         /* DCTSIZE squared; # of elements in a block */
pub const NUM_QUANT_TBLS: c_int = 4;    /* Quantization tables are numbered 0..3 */
pub const NUM_HUFF_TBLS: c_int = 4;     /* Huffman tables are numbered 0..3 */
pub const NUM_ARITH_TBLS: c_int = 16;   /* Arith-coding tables are numbered 0..15 */
pub const MAX_COMPS_IN_SCAN: c_int = 4; /* JPEG limit on # of components in one scan */
pub const MAX_SAMP_FACTOR: c_int = 4;   /* JPEG limit on sampling factors */
/* Unfortunately, some bozo at Adobe saw no reason to be bound by the standard;
 * the PostScript DCT filter can emit files with many more than 10 blocks/MCU.
 * If you happen to run across such a file, you can up D_MAX_BLOCKS_IN_MCU
 * to handle it.  We even let you do this from the jconfig.h file.  However,
 * we strongly discourage changing C_MAX_BLOCKS_IN_MCU; just because Adobe
 * sometimes emits noncompliant files doesn't mean you should too.
 */
pub const C_MAX_BLOCKS_IN_MCU: c_int = 10;  /* compressor's limit on blocks per MCU */
pub const D_MAX_BLOCKS_IN_MCU: c_int = 10;  /* decompressor's limit on blocks per MCU */


/* Data structures for images (arrays of samples and of DCT coefficients).
 * On 80x86 machines, the image arrays are too big for near pointers,
 * but the pointer arrays can fit in near memory.
 */

pub type JSAMPROW = *mut JSAMPLE;       /* ptr to one image row of pixel samples. */
pub type JSAMPARRAY = *mut *mut JSAMPLE; /* ptr to some rows (a 2-D sample array) */
pub type JSAMPIMAGE = *mut *mut *mut JSAMPLE; /* a 3-D sample array: top index is color */

pub type JBLOCK = [JCOEF; 64];          /* one block of coefficients */
pub type JBLOCKROW = *mut JBLOCK;       /* pointer to one row of coefficient blocks */
pub type JBLOCKARRAY = *mut *mut JBLOCK; /* a 2-D array of coefficient blocks */
pub type JBLOCKIMAGE = *mut *mut *mut JBLOCK; /* a 3-D array of coefficient blocks */

pub type JCOEFPTR = *mut JCOEF;         /* useful in a couple of places */


/* Types for JPEG compression parameters and working tables. */


/* DCT coefficient quantization tables. */

#[repr(C)]
pub struct JQUANT_TBL {
  /* This field directly represents the contents of a JPEG DQT marker.
   * Note: the values are always given in zigzag order.
   */
  pub quantval: [UINT16; 64],           /* quantization step for each coefficient */
  /* This field is used only during compression.  It's initialized FALSE when
   * the table is created, and set TRUE when it's been output to the file.
   * You could suppress output of a table by setting this to TRUE.
   * (See jpeg_suppress_tables for an example.)
   */
  pub sent_table: boolean,              /* TRUE when table has been output */
}


/* Huffman coding tables. */

#[repr(C)]
pub struct JHUFF_TBL {
  /* These two fields directly represent the contents of a JPEG DHT marker */
  pub bits: [UINT8; 17],                /* bits[k] = # of symbols with codes of */
                                        /* length k bits; bits[0] is unused */
  pub huffval: [UINT8; 256],            /* The symbols, in order of incr code length */
  /* This field is used only during compression.  It's initialized FALSE when
   * the table is created, and set TRUE when it's been output to the file.
   * You could suppress output of a table by setting this to TRUE.
   * (See jpeg_suppress_tables for an example.)
   */
  pub sent_table: boolean,              /* TRUE when table has been output */
}


/* Basic info about one component (color channel). */

#[repr(C)]
pub struct jpeg_component_info {
  /* These values are fixed over the whole image. */
  /* For compression, they must be supplied by parameter setup; */
  /* for decompression, they are read from the SOF marker. */
  pub component_id: c_int,              /* identifier for this component (0..255) */
  pub component_index: c_int,           /* its index in SOF or cinfo->comp_info[] */
  pub h_samp_factor: c_int,             /* horizontal sampling factor (1..4) */
  pub v_samp_factor: c_int,             /* vertical sampling factor (1..4) */
  pub quant_tbl_no: c_int,              /* quantization table selector (0..3) */
  /* These values may vary between scans. */
  /* For compression, they must be supplied by parameter setup; */
  /* for decompression, they are read from the SOS marker. */
  /* The decompressor output side may not use these variables. */
  pub dc_tbl_no: c_int,                 /* DC entropy table selector (0..3) */
  pub ac_tbl_no: c_int,                 /* AC entropy table selector (0..3) */

  /* Remaining fields should be treated as private by applications. */

  /* These values are computed during compression or decompression startup: */
  /* Component's size in DCT blocks.
   * Any dummy blocks added to complete an MCU are not counted; therefore
   * these values do not depend on whether a scan is interleaved or not.
   */
  pub width_in_blocks: JDIMENSION,
  pub height_in_blocks: JDIMENSION,
  /* Size of a DCT block in samples.  Always DCTSIZE for compression.
   * For decompression this is the size of the output from one DCT block,
   * reflecting any scaling we choose to apply during the IDCT step.
   * Values of 1,2,4,8 are likely to be supported.  Note that different
   * components may receive different IDCT scalings.
   */
  pub DCT_scaled_size: c_int,
  /* The downsampled dimensions are the component's actual, unpadded number
   * of samples at the main buffer (preprocessing/compression interface), thus
   * downsampled_width = ceil(image_width * Hi/Hmax)
   * and similarly for height.  For decompression, IDCT scaling is included, so
   * downsampled_width = ceil(image_width * Hi/Hmax * DCT_scaled_size/DCTSIZE)
   */
  pub downsampled_width: JDIMENSION,   /* actual width in samples */
  pub downsampled_height: JDIMENSION,  /* actual height in samples */
  /* This flag is used only for decompression.  In cases where some of the
   * components will be ignored (eg grayscale output from YCbCr image),
   * we can skip most computations for the unused components.
   */
  pub component_needed: boolean,        /* do we need the value of this component? */

  /* These values are computed before starting a scan of the component. */
  /* The decompressor output side may not use these variables. */
  pub MCU_width: c_int,                 /* number of blocks per MCU, horizontally */
  pub MCU_height: c_int,                /* number of blocks per MCU, vertically */
  pub MCU_blocks: c_int,                /* MCU_width * MCU_height */
  pub MCU_sample_width: c_int,          /* MCU width in samples, MCU_width*DCT_scaled_size */
  pub last_col_width: c_int,            /* # of non-dummy blocks across in last MCU */
  pub last_row_height: c_int,           /* # of non-dummy blocks down in last MCU */

  /* Saved quantization table for component; NULL if none yet saved.
   * See jdinput.c comments about the need for this information.
   * This field is not currently used by the compressor.
   */
  pub quant_table: *mut JQUANT_TBL,

  /* Private per-component storage for DCT or IDCT subsystem. */
  pub dct_table: *mut c_void,
}


/* The script for encoding a multiple-scan file is an array of these: */

#[repr(C)]
pub struct jpeg_scan_info {
  pub comps_in_scan: c_int,             /* number of components encoded in this scan */
  pub component_index: [c_int; 4],      /* their SOF/comp_info[] indexes */
  pub Ss: c_int,
  pub Se: c_int,                        /* progressive JPEG spectral selection parms */
  pub Ah: c_int,
  pub Al: c_int,                        /* progressive JPEG successive approx. parms */
}


/* Known color spaces. */

#[repr(C)]
#[derive(Clone, Copy)]
pub enum J_COLOR_SPACE {
    JCS_UNKNOWN,                        /* error/unspecified */
    JCS_GRAYSCALE,                      /* monochrome */
    JCS_RGB,                            /* red/green/blue */
    JCS_YCbCr,                          /* Y/Cb/Cr (also known as YUV) */
    JCS_CMYK,                           /* C/M/Y/K */
    JCS_YCCK,                           /* Y/Cb/Cr/K */
}

/* DCT/IDCT algorithm options. */

#[repr(C)]
#[derive(Clone, Copy)]
pub enum J_DCT_METHOD {
    JDCT_ISLOW,                         /* slow but accurate integer algorithm */
    JDCT_IFAST,                         /* faster, less accurate integer method */
    JDCT_FLOAT,                         /* floating-point: accurate, fast on fast HW */
}

pub const JDCT_DEFAULT: J_DCT_METHOD = J_DCT_METHOD::JDCT_ISLOW;
pub const JDCT_FASTEST: J_DCT_METHOD = J_DCT_METHOD::JDCT_IFAST;

/* Dithering options for decompression. */

#[repr(C)]
#[derive(Clone, Copy)]
pub enum J_DITHER_MODE {
    JDITHER_NONE,                       /* no dithering */
    JDITHER_ORDERED,                    /* simple ordered dither */
    JDITHER_FS,                         /* Floyd-Steinberg error diffusion dither */
}


/* Common fields between JPEG compression and decompression master structs. */

#[repr(C)]
pub struct jpeg_common_struct {
  pub err: *mut jpeg_error_mgr,         /* Error handler module */
  pub mem: *mut jpeg_memory_mgr,        /* Memory manager module */
  pub progress: *mut jpeg_progress_mgr, /* Progress monitor, or NULL if none */
  pub is_decompressor: boolean,         /* so common code can tell which is which */
  pub global_state: c_int,              /* for checking call sequence validity */
  /* Additional fields follow in an actual jpeg_compress_struct or
   * jpeg_decompress_struct.  All three structs must agree on these
   * initial fields!  (This would be a lot cleaner in C++.)
   */
}

pub type j_common_ptr = *mut jpeg_common_struct;
pub type j_compress_ptr = *mut jpeg_compress_struct;
pub type j_decompress_ptr = *mut jpeg_decompress_struct;


/* Master record for a compression instance */

#[repr(C)]
pub struct jpeg_compress_struct {
  pub err: *mut jpeg_error_mgr,
  pub mem: *mut jpeg_memory_mgr,
  pub progress: *mut jpeg_progress_mgr,
  pub is_decompressor: boolean,
  pub global_state: c_int,

  /* Destination for compressed data */
  pub dest: *mut jpeg_destination_mgr,

  /* Description of source image --- these fields must be filled in by
   * outer application before starting compression.  in_color_space must
   * be correct before you can even call jpeg_set_defaults().
   */

  pub image_width: JDIMENSION,          /* input image width */
  pub image_height: JDIMENSION,         /* input image height */
  pub input_components: c_int,          /* # of color components in input image */
  pub in_color_space: J_COLOR_SPACE,    /* colorspace of input image */

  pub input_gamma: f64,                 /* image gamma of input image */

  /* Compression parameters --- these fields must be set before calling
   * jpeg_start_compress().  We recommend calling jpeg_set_defaults() to
   * initialize everything to reasonable defaults, then changing anything
   * the application specifically wants to change.  That way you won't get
   * burnt when new parameters are added.  Also note that there are several
   * helper routines to simplify changing parameters.
   */

  pub data_precision: c_int,            /* bits of precision in image data */

  pub num_components: c_int,            /* # of color components in JPEG image */
  pub jpeg_color_space: J_COLOR_SPACE,  /* colorspace of JPEG image */

  pub comp_info: *mut jpeg_component_info,
  /* comp_info[i] describes component that appears i'th in SOF */

  pub quant_tbl_ptrs: [*mut JQUANT_TBL; 4],
  /* ptrs to coefficient quantization tables, or NULL if not defined */

  pub dc_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
  pub ac_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
  /* ptrs to Huffman coding tables, or NULL if not defined */

  pub arith_dc_L: [UINT8; 16],          /* L values for DC arith-coding tables */
  pub arith_dc_U: [UINT8; 16],          /* U values for DC arith-coding tables */
  pub arith_ac_K: [UINT8; 16],          /* Kx values for AC arith-coding tables */

  pub num_scans: c_int,                 /* # of entries in scan_info array */
  pub scan_info: *const jpeg_scan_info, /* script for multi-scan file, or NULL */
  /* The default value of scan_info is NULL, which causes a single-scan
   * sequential JPEG file to be emitted.  To create a multi-scan file,
   * set num_scans and scan_info to point to an array of scan definitions.
   */

  pub raw_data_in: boolean,             /* TRUE=caller supplies downsampled data */
  pub arith_code: boolean,              /* TRUE=arithmetic coding, FALSE=Huffman */
  pub optimize_coding: boolean,         /* TRUE=optimize entropy encoding parms */
  pub CCIR601_sampling: boolean,        /* TRUE=first samples are cosited */
  pub smoothing_factor: c_int,          /* 1..100, or 0 for no input smoothing */
  pub dct_method: J_DCT_METHOD,         /* DCT algorithm selector */

  /* The restart interval can be specified in absolute MCUs by setting
   * restart_interval, or in MCU rows by setting restart_in_rows
   * (in which case the correct restart_interval will be figured
   * for each scan).
   */
  pub restart_interval: u32,            /* MCUs per restart, or 0 for no restart */
  pub restart_in_rows: c_int,           /* if > 0, MCU rows per restart interval */

  /* Parameters controlling emission of special markers. */

  pub write_JFIF_header: boolean,       /* should a JFIF marker be written? */
  /* These three values are not used by the JPEG code, merely copied */
  /* into the JFIF APP0 marker.  density_unit can be 0 for unknown, */
  /* 1 for dots/inch, or 2 for dots/cm.  Note that the pixel aspect */
  /* ratio is defined by X_density/Y_density even when density_unit=0. */
  pub density_unit: UINT8,              /* JFIF code for pixel size units */
  pub X_density: UINT16,                /* Horizontal pixel density */
  pub Y_density: UINT16,                /* Vertical pixel density */
  pub write_Adobe_marker: boolean,      /* should an Adobe marker be written? */

  /* State variable: index of next scanline to be written to
   * jpeg_write_scanlines().  Application may use this to control its
   * processing loop, e.g., "while (next_scanline < image_height)".
   */

  pub next_scanline: JDIMENSION,        /* 0 .. image_height-1  */

  /* Remaining fields are known throughout compressor, but generally
   * should not be touched by a surrounding application.
   */

  /*
   * These fields are computed during compression startup
   */
  pub progressive_mode: boolean,        /* TRUE if scan script uses progressive mode */
  pub max_h_samp_factor: c_int,         /* largest h_samp_factor */
  pub max_v_samp_factor: c_int,         /* largest v_samp_factor */

  pub total_iMCU_rows: JDIMENSION,      /* # of iMCU rows to be input to coef ctlr */
  /* The coefficient controller receives data in units of MCU rows as defined
   * for fully interleaved scans (whether the JPEG file is interleaved or not).
   * There are v_samp_factor * DCTSIZE sample rows of each component in an
   * "iMCU" (interleaved MCU) row.
   */

  /*
   * These fields are valid during any one scan.
   * They describe the components and MCUs actually appearing in the scan.
   */
  pub comps_in_scan: c_int,             /* # of JPEG components in this scan */
  pub cur_comp_info: [*mut jpeg_component_info; 4],
  /* *cur_comp_info[i] describes component that appears i'th in SOS */

  pub MCUs_per_row: JDIMENSION,         /* # of MCUs across the image */
  pub MCU_rows_in_scan: JDIMENSION,     /* # of MCU rows in the image */

  pub blocks_in_MCU: c_int,             /* # of DCT blocks per MCU */
  pub MCU_membership: [c_int; 10],
  /* MCU_membership[i] is index in cur_comp_info of component owning */
  /* i'th block in an MCU */

  pub Ss: c_int,
  pub Se: c_int,
  pub Ah: c_int,
  pub Al: c_int,                        /* progressive JPEG parameters for scan */

  /*
   * Links to compression subobjects (methods and private variables of modules)
   */
  pub master: *mut jpeg_comp_master,
  pub main: *mut jpeg_c_main_controller,
  pub prep: *mut jpeg_c_prep_controller,
  pub coef: *mut jpeg_c_coef_controller,
  pub marker: *mut jpeg_marker_writer,
  pub cconvert: *mut jpeg_color_converter,
  pub downsample: *mut jpeg_downsampler,
  pub fdct: *mut jpeg_forward_dct,
  pub entropy: *mut jpeg_entropy_encoder,
}


/* Master record for a decompression instance */

#[repr(C)]
pub struct jpeg_decompress_struct {
  pub err: *mut jpeg_error_mgr,
  pub mem: *mut jpeg_memory_mgr,
  pub progress: *mut jpeg_progress_mgr,
  pub is_decompressor: boolean,
  pub global_state: c_int,

  /* Source of compressed data */
  pub src: *mut jpeg_source_mgr,

  /* Basic description of image --- filled in by jpeg_read_header(). */
  /* Application may inspect these values to decide how to process image. */

  pub image_width: JDIMENSION,          /* nominal image width (from SOF marker) */
  pub image_height: JDIMENSION,         /* nominal image height */
  pub num_components: c_int,            /* # of color components in JPEG image */
  pub jpeg_color_space: J_COLOR_SPACE,  /* colorspace of JPEG image */

  /* Decompression processing parameters --- these fields must be set before
   * calling jpeg_start_decompress().  Note that jpeg_read_header() initializes
   * them to default values.
   */

  pub out_color_space: J_COLOR_SPACE,   /* colorspace for output */

  pub scale_num: u32,
  pub scale_denom: u32,                 /* fraction by which to scale image */

  pub output_gamma: f64,                /* image gamma wanted in output */

  pub buffered_image: boolean,          /* TRUE=multiple output passes */
  pub raw_data_out: boolean,            /* TRUE=downsampled data wanted */

  pub dct_method: J_DCT_METHOD,         /* IDCT algorithm selector */
  pub do_fancy_upsampling: boolean,     /* TRUE=apply fancy upsampling */
  pub do_block_smoothing: boolean,      /* TRUE=apply interblock smoothing */

  pub quantize_colors: boolean,         /* TRUE=colormapped output wanted */
  /* the following are ignored if not quantize_colors: */
  pub dither_mode: J_DITHER_MODE,       /* type of color dithering to use */
  pub two_pass_quantize: boolean,       /* TRUE=use two-pass color quantization */
  pub desired_number_of_colors: c_int,  /* max # colors to use in created colormap */
  /* these are significant only in buffered-image mode: */
  pub enable_1pass_quant: boolean,      /* enable future use of 1-pass quantizer */
  pub enable_external_quant: boolean,   /* enable future use of external colormap */
  pub enable_2pass_quant: boolean,      /* enable future use of 2-pass quantizer */

  /* Description of actual output image that will be returned to application.
   * These fields are computed by jpeg_start_decompress().
   * You can also use jpeg_calc_output_dimensions() to determine these values
   * in advance of calling jpeg_start_decompress().
   */

  pub output_width: JDIMENSION,         /* scaled image width */
  pub output_height: JDIMENSION,        /* scaled image height */
  pub out_color_components: c_int,      /* # of color components in out_color_space */
  pub output_components: c_int,         /* # of color components returned */
  /* output_components is 1 (a colormap index) when quantizing colors;
   * otherwise it equals out_color_components.
   */
  pub rec_outbuf_height: c_int,         /* min recommended height of scanline buffer */
  /* If the buffer passed to jpeg_read_scanlines() is less than this many rows
   * high, space and time will be wasted due to unnecessary data copying.
   * Usually rec_outbuf_height will be 1 or 2, at most 4.
   */

  /* When quantizing colors, the output colormap is described by these fields.
   * The application can supply a colormap by setting colormap non-NULL before
   * calling jpeg_start_decompress; otherwise a colormap is created during
   * jpeg_start_decompress or jpeg_start_output.
   * The map has out_color_components rows and actual_number_of_colors columns.
   */
  pub actual_number_of_colors: c_int,   /* number of entries in use */
  pub colormap: JSAMPARRAY,             /* The color map as a 2-D pixel array */

  /* State variables: these variables indicate the progress of decompression.
   * The application may examine these but must not modify them.
   */

  /* Row index of next scanline to be read from jpeg_read_scanlines().
   * Application may use this to control its processing loop, e.g.,
   * "while (output_scanline < output_height)".
   */
  pub output_scanline: JDIMENSION,      /* 0 .. output_height-1  */

  /* Current input scan number and number of iMCU rows completed in scan.
   * These indicate the progress of the decompressor input side.
   */
  pub input_scan_number: c_int,         /* Number of SOS markers seen so far */
  pub input_iMCU_row: JDIMENSION,       /* Number of iMCU rows completed */

  /* The "output scan number" is the notional scan being displayed by the
   * output side.  The decompressor will not allow output scan/row number
   * to get ahead of input scan/row, but it can fall arbitrarily far behind.
   */
  pub output_scan_number: c_int,        /* Nominal scan number being displayed */
  pub output_iMCU_row: JDIMENSION,      /* Number of iMCU rows read */

  /* Current progression status.  coef_bits[c][i] indicates the precision
   * with which component c's DCT coefficient i (in zigzag order) is known.
   * It is -1 when no data has yet been received, otherwise it is the point
   * transform (shift) value for the most recent scan of the coefficient
   * (thus, 0 at completion of the progression).
   * This pointer is NULL when reading a non-progressive file.
   */
  pub coef_bits: *mut [c_int; 64],      /* -1 or current Al value for each coef */

  /* Internal JPEG parameters --- the application usually need not look at
   * these fields.  Note that the decompressor output side may not use
   * any parameters that can change between scans.
   */

  /* Quantization and Huffman tables are carried forward across input
   * datastreams when processing abbreviated JPEG datastreams.
   */

  pub quant_tbl_ptrs: [*mut JQUANT_TBL; 4],
  /* ptrs to coefficient quantization tables, or NULL if not defined */

  pub dc_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
  pub ac_huff_tbl_ptrs: [*mut JHUFF_TBL; 4],
  /* ptrs to Huffman coding tables, or NULL if not defined */

  /* These parameters are never carried across datastreams, since they
   * are given in SOF/SOS markers or defined to be reset by SOI.
   */

  pub data_precision: c_int,            /* bits of precision in image data */

  pub comp_info: *mut jpeg_component_info,
  /* comp_info[i] describes component that appears i'th in SOF */

  pub progressive_mode: boolean,        /* TRUE if SOFn specifies progressive mode */
  pub arith_code: boolean,              /* TRUE=arithmetic coding, FALSE=Huffman */

  pub arith_dc_L: [UINT8; 16],          /* L values for DC arith-coding tables */
  pub arith_dc_U: [UINT8; 16],          /* U values for DC arith-coding tables */
  pub arith_ac_K: [UINT8; 16],          /* Kx values for AC arith-coding tables */

  pub restart_interval: u32,            /* MCUs per restart interval, or 0 for no restart */

  /* These fields record data obtained from optional markers recognized by
   * the JPEG library.
   */
  pub saw_JFIF_marker: boolean,         /* TRUE iff a JFIF APP0 marker was found */
  /* Data copied from JFIF marker: */
  pub density_unit: UINT8,              /* JFIF code for pixel size units */
  pub X_density: UINT16,                /* Horizontal pixel density */
  pub Y_density: UINT16,                /* Vertical pixel density */
  pub saw_Adobe_marker: boolean,        /* TRUE iff an Adobe APP14 marker was found */
  pub Adobe_transform: UINT8,           /* Color transform code from Adobe marker */

  pub CCIR601_sampling: boolean,        /* TRUE=first samples are cosited */

  /* Remaining fields are known throughout decompressor, but generally
   * should not be touched by a surrounding application.
   */

  /*
   * These fields are computed during decompression startup
   */
  pub max_h_samp_factor: c_int,         /* largest h_samp_factor */
  pub max_v_samp_factor: c_int,         /* largest v_samp_factor */

  pub min_DCT_scaled_size: c_int,       /* smallest DCT_scaled_size of any component */

  pub total_iMCU_rows: JDIMENSION,      /* # of iMCU rows in image */
  /* The coefficient controller's input and output progress is measured in
   * units of "iMCU" (interleaved MCU) rows.  These are the same as MCU rows
   * in fully interleaved JPEG scans, but are used whether the scan is
   * interleaved or not.  We define an iMCU row as v_samp_factor DCT block
   * rows of each component.  Therefore, the IDCT output contains
   * v_samp_factor*DCT_scaled_size sample rows of a component per iMCU row.
   */

  pub sample_range_limit: *mut JSAMPLE, /* table for fast range-limiting */

  /*
   * These fields are valid during any one scan.
   * They describe the components and MCUs actually appearing in the scan.
   * Note that the decompressor output side must not use these fields.
   */
  pub comps_in_scan: c_int,             /* # of JPEG components in this scan */
  pub cur_comp_info: [*mut jpeg_component_info; 4],
  /* *cur_comp_info[i] describes component that appears i'th in SOS */

  pub MCUs_per_row: JDIMENSION,         /* # of MCUs across the image */
  pub MCU_rows_in_scan: JDIMENSION,     /* # of MCU rows in the image */

  pub blocks_in_MCU: c_int,             /* # of DCT blocks per MCU */
  pub MCU_membership: [c_int; 10],
  /* MCU_membership[i] is index in cur_comp_info of component owning */
  /* i'th block in an MCU */

  pub Ss: c_int,
  pub Se: c_int,
  pub Ah: c_int,
  pub Al: c_int,                        /* progressive JPEG parameters for scan */

  /* This field is shared between entropy decoder and marker parser.
   * It is either zero or the code of a JPEG marker that has been
   * read from the data source, but has not yet been processed.
   */
  pub unread_marker: c_int,

  /*
   * Links to decompression subobjects (methods, private variables of modules)
   */
  pub master: *mut jpeg_decomp_master,
  pub main: *mut jpeg_d_main_controller,
  pub coef: *mut jpeg_d_coef_controller,
  pub post: *mut jpeg_d_post_controller,
  pub inputctl: *mut jpeg_input_controller,
  pub marker: *mut jpeg_marker_reader,
  pub entropy: *mut jpeg_entropy_decoder,
  pub idct: *mut jpeg_inverse_dct,
  pub upsample: *mut jpeg_upsampler,
  pub cconvert: *mut jpeg_color_deconverter,
  pub cquantize: *mut jpeg_color_quantizer,
}


/* "Object" declarations for JPEG modules that may be supplied or called
 * directly by the surrounding application.
 * As with all objects in the JPEG library, these structs only define the
 * publicly visible methods and state variables of a module.  Additional
 * private fields may exist after the public ones.
 */


/* Error handler object */

pub type jpeg_error_exit_method = extern "C" fn(cinfo: j_common_ptr);
pub type jpeg_emit_message_method = extern "C" fn(cinfo: j_common_ptr, msg_level: c_int);
pub type jpeg_output_message_method = extern "C" fn(cinfo: j_common_ptr);
pub type jpeg_format_message_method = extern "C" fn(cinfo: j_common_ptr, buffer: *mut c_char);
pub type jpeg_reset_error_mgr_method = extern "C" fn(cinfo: j_common_ptr);

#[repr(C)]
pub struct jpeg_error_mgr {
  /* Error exit handler: does not return to caller */
  pub error_exit: Option<jpeg_error_exit_method>,
  /* Conditionally emit a trace or warning message */
  pub emit_message: Option<jpeg_emit_message_method>,
  /* Routine that actually outputs a trace or error message */
  pub output_message: Option<jpeg_output_message_method>,
  /* Format a message string for the most recent JPEG error or message */
  pub format_message: Option<jpeg_format_message_method>,
  /* Reset error state variables at start of a new image */
  pub reset_error_mgr: Option<jpeg_reset_error_mgr_method>,

  /* The message ID code and any parameters are saved here.
   * A message can have one string parameter or up to 8 int parameters.
   */
  pub msg_code: c_int,
  pub msg_parm: jpeg_error_mgr_msg_parm,

  /* Standard state variables for error facility */

  pub trace_level: c_int,               /* max msg_level that will be displayed */

  /* For recoverable corrupt-data errors, we emit a warning message,
   * but keep going unless emit_message chooses to abort.  emit_message
   * should count warnings in num_warnings.  The surrounding application
   * can check for bad data by seeing if num_warnings is nonzero at the
   * end of processing.
   */
  pub num_warnings: c_ulong,            /* number of corrupt-data warnings */

  /* These fields point to the table(s) of error message strings.
   * An application can change the table pointer to switch to a different
   * message list (typically, to change the language in which errors are
   * reported).  Some applications may wish to add additional error codes
   * that will be handled by the JPEG library error mechanism; the second
   * table pointer is used for this purpose.
   *
   * First table includes all errors generated by JPEG library itself.
   * Error code 0 is reserved for a "no such error string" message.
   */
  pub jpeg_message_table: *const *const c_char, /* Library errors */
  pub last_jpeg_message: c_int,         /* Table contains strings 0..last_jpeg_message */
  /* Second table can be added by application (see cjpeg/djpeg for example).
   * It contains strings numbered first_addon_message..last_addon_message.
   */
  pub addon_message_table: *const *const c_char, /* Non-library errors */
  pub first_addon_message: c_int,       /* code for first string in addon table */
  pub last_addon_message: c_int,        /* code for last string in addon table */
}

#[repr(C)]
pub union jpeg_error_mgr_msg_parm {
  pub i: [c_int; 8],
  pub s: [c_char; 80],
}

pub const JMSG_LENGTH_MAX: c_int = 200;   /* recommended size of format_message buffer */
pub const JMSG_STR_PARM_MAX: c_int = 80;


/* Progress monitor object */

pub type jpeg_progress_monitor_method = extern "C" fn(cinfo: j_common_ptr);

#[repr(C)]
pub struct jpeg_progress_mgr {
  pub progress_monitor: Option<jpeg_progress_monitor_method>,

  pub pass_counter: c_ulong,            /* work units completed in this pass */
  pub pass_limit: c_ulong,              /* total number of work units in this pass */
  pub completed_passes: c_int,          /* passes completed so far */
  pub total_passes: c_int,              /* total number of passes expected */
}


/* Data destination object for compression */

pub type jpeg_init_destination_method = extern "C" fn(cinfo: j_compress_ptr);
pub type jpeg_empty_output_buffer_method = extern "C" fn(cinfo: j_compress_ptr) -> boolean;
pub type jpeg_term_destination_method = extern "C" fn(cinfo: j_compress_ptr);

#[repr(C)]
pub struct jpeg_destination_mgr {
  pub next_output_byte: *mut JOCTET,    /* => next byte to write in buffer */
  pub free_in_buffer: usize,            /* # of byte spaces remaining in buffer */

  pub init_destination: Option<jpeg_init_destination_method>,
  pub empty_output_buffer: Option<jpeg_empty_output_buffer_method>,
  pub term_destination: Option<jpeg_term_destination_method>,
}


/* Data source object for decompression */

pub type jpeg_init_source_method = extern "C" fn(cinfo: j_decompress_ptr);
pub type jpeg_fill_input_buffer_method = extern "C" fn(cinfo: j_decompress_ptr) -> boolean;
pub type jpeg_skip_input_data_method = extern "C" fn(cinfo: j_decompress_ptr, num_bytes: c_ulong);
pub type jpeg_resync_to_restart_method = extern "C" fn(cinfo: j_decompress_ptr, desired: c_int) -> boolean;
pub type jpeg_term_source_method = extern "C" fn(cinfo: j_decompress_ptr);

#[repr(C)]
pub struct jpeg_source_mgr {
  pub next_input_byte: *const JOCTET,   /* => next byte to read from buffer */
  pub bytes_in_buffer: usize,           /* # of bytes remaining in buffer */

  pub init_source: Option<jpeg_init_source_method>,
  pub fill_input_buffer: Option<jpeg_fill_input_buffer_method>,
  pub skip_input_data: Option<jpeg_skip_input_data_method>,
  pub resync_to_restart: Option<jpeg_resync_to_restart_method>,
  pub term_source: Option<jpeg_term_source_method>,
}


/* Memory manager object.
 * Allocates "small" objects (a few K total), "large" objects (tens of K),
 * and "really big" objects (virtual arrays with backing store if needed).
 * The memory manager does not allow individual objects to be freed; rather,
 * each created object is assigned to a pool, and whole pools can be freed
 * at once.  This is faster and more convenient than remembering exactly what
 * to free, especially where malloc()/free() are not too speedy.
 * NB: alloc routines never return NULL.  They exit to error_exit if not
 * successful.
 */

pub const JPOOL_PERMANENT: c_int = 0;   /* lasts until master record is destroyed */
pub const JPOOL_IMAGE: c_int = 1;       /* lasts until done with image/datastream */
pub const JPOOL_NUMPOOLS: c_int = 2;

pub type jvirt_sarray_ptr = *mut jvirt_sarray_control;
pub type jvirt_barray_ptr = *mut jvirt_barray_control;

pub type jpeg_alloc_small_method = extern "C" fn(cinfo: j_common_ptr, pool_id: c_int, sizeofobject: usize) -> *mut c_void;
pub type jpeg_alloc_large_method = extern "C" fn(cinfo: j_common_ptr, pool_id: c_int, sizeofobject: usize) -> *mut c_void;
pub type jpeg_alloc_sarray_method = extern "C" fn(cinfo: j_common_ptr, pool_id: c_int, samplesperrow: JDIMENSION, numrows: JDIMENSION) -> JSAMPARRAY;
pub type jpeg_alloc_barray_method = extern "C" fn(cinfo: j_common_ptr, pool_id: c_int, blocksperrow: JDIMENSION, numrows: JDIMENSION) -> JBLOCKARRAY;
pub type jpeg_request_virt_sarray_method = extern "C" fn(cinfo: j_common_ptr, pool_id: c_int, pre_zero: boolean, samplesperrow: JDIMENSION, numrows: JDIMENSION, maxaccess: JDIMENSION) -> jvirt_sarray_ptr;
pub type jpeg_request_virt_barray_method = extern "C" fn(cinfo: j_common_ptr, pool_id: c_int, pre_zero: boolean, blocksperrow: JDIMENSION, numrows: JDIMENSION, maxaccess: JDIMENSION) -> jvirt_barray_ptr;
pub type jpeg_realize_virt_arrays_method = extern "C" fn(cinfo: j_common_ptr);
pub type jpeg_access_virt_sarray_method = extern "C" fn(cinfo: j_common_ptr, ptr: jvirt_sarray_ptr, start_row: JDIMENSION, num_rows: JDIMENSION, writable: boolean) -> JSAMPARRAY;
pub type jpeg_access_virt_barray_method = extern "C" fn(cinfo: j_common_ptr, ptr: jvirt_barray_ptr, start_row: JDIMENSION, num_rows: JDIMENSION, writable: boolean) -> JBLOCKARRAY;
pub type jpeg_free_pool_method = extern "C" fn(cinfo: j_common_ptr, pool_id: c_int);
pub type jpeg_self_destruct_method = extern "C" fn(cinfo: j_common_ptr);

#[repr(C)]
pub struct jpeg_memory_mgr {
  /* Method pointers */
  pub alloc_small: Option<jpeg_alloc_small_method>,
  pub alloc_large: Option<jpeg_alloc_large_method>,
  pub alloc_sarray: Option<jpeg_alloc_sarray_method>,
  pub alloc_barray: Option<jpeg_alloc_barray_method>,
  pub request_virt_sarray: Option<jpeg_request_virt_sarray_method>,
  pub request_virt_barray: Option<jpeg_request_virt_barray_method>,
  pub realize_virt_arrays: Option<jpeg_realize_virt_arrays_method>,
  pub access_virt_sarray: Option<jpeg_access_virt_sarray_method>,
  pub access_virt_barray: Option<jpeg_access_virt_barray_method>,
  pub free_pool: Option<jpeg_free_pool_method>,
  pub self_destruct: Option<jpeg_self_destruct_method>,

  /* Limit on memory allocation for this JPEG object.  (Note that this is
   * merely advisory, not a guaranteed maximum; it only affects the space
   * used for virtual-array buffers.)  May be changed by outer application
   * after creating the JPEG object.
   */
  pub max_memory_to_use: c_ulong,
}


/* Routine signature for application-supplied marker processing methods.
 * Need not pass marker code since it is stored in cinfo->unread_marker.
 */
pub type jpeg_marker_parser_method = extern "C" fn(cinfo: j_decompress_ptr) -> boolean;


/* Stub forward declarations for internal JPEG structures. */
#[repr(C)]
pub struct jvirt_sarray_control {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jvirt_barray_control {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_comp_master {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_c_main_controller {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_c_prep_controller {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_c_coef_controller {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_marker_writer {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_color_converter {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_downsampler {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_forward_dct {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_entropy_encoder {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_decomp_master {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_d_main_controller {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_d_coef_controller {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_d_post_controller {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_input_controller {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_marker_reader {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_entropy_decoder {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_inverse_dct {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_upsampler {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_color_deconverter {
  pub dummy: c_ulong,
}

#[repr(C)]
pub struct jpeg_color_quantizer {
  pub dummy: c_ulong,
}


/* Routine signatures for external library functions */

/* Default error-management setup */
extern "C" {
  pub fn jpeg_std_error(err: *mut jpeg_error_mgr) -> *mut jpeg_error_mgr;

  /* Initialization and destruction of JPEG compression objects */
  /* NB: you must set up the error-manager BEFORE calling jpeg_create_xxx */
  pub fn jpeg_create_compress(cinfo: j_compress_ptr);
  pub fn jpeg_create_decompress(cinfo: j_decompress_ptr);
  pub fn jpeg_destroy_compress(cinfo: j_compress_ptr);
  pub fn jpeg_destroy_decompress(cinfo: j_decompress_ptr);

  /* Standard data source and destination managers: stdio streams. */
  /* Caller is responsible for opening the file before and closing after. */
  pub fn jpeg_stdio_dest(cinfo: j_compress_ptr, outfile: *mut core::ffi::c_void);
  pub fn jpeg_stdio_src(cinfo: j_decompress_ptr, infile: *mut u8);

  /* Default parameter setup for compression */
  pub fn jpeg_set_defaults(cinfo: j_compress_ptr);
  /* Compression parameter setup aids */
  pub fn jpeg_set_colorspace(cinfo: j_compress_ptr, colorspace: J_COLOR_SPACE);
  pub fn jpeg_default_colorspace(cinfo: j_compress_ptr);
  pub fn jpeg_set_quality(cinfo: j_compress_ptr, quality: c_int, force_baseline: boolean);
  pub fn jpeg_set_linear_quality(cinfo: j_compress_ptr, scale_factor: c_int, force_baseline: boolean);
  pub fn jpeg_add_quant_table(cinfo: j_compress_ptr, which_tbl: c_int, basic_table: *const u32, scale_factor: c_int, force_baseline: boolean);
  pub fn jpeg_quality_scaling(quality: c_int) -> c_int;
  pub fn jpeg_simple_progression(cinfo: j_compress_ptr);
  pub fn jpeg_suppress_tables(cinfo: j_compress_ptr, suppress: boolean);
  pub fn jpeg_alloc_quant_table(cinfo: j_common_ptr) -> *mut JQUANT_TBL;
  pub fn jpeg_alloc_huff_table(cinfo: j_common_ptr) -> *mut JHUFF_TBL;

  /* Main entry points for compression */
  pub fn jpeg_start_compress(cinfo: j_compress_ptr, write_all_tables: boolean);
  pub fn jpeg_write_scanlines(cinfo: j_compress_ptr, scanlines: JSAMPARRAY, num_lines: JDIMENSION) -> JDIMENSION;
  pub fn jpeg_finish_compress(cinfo: j_compress_ptr);

  /* Replaces jpeg_write_scanlines when writing raw downsampled data. */
  pub fn jpeg_write_raw_data(cinfo: j_compress_ptr, data: JSAMPIMAGE, num_lines: JDIMENSION) -> JDIMENSION;

  /* Write a special marker.  See libjpeg.doc concerning safe usage. */
  pub fn jpeg_write_marker(cinfo: j_compress_ptr, marker: c_int, dataptr: *const JOCTET, datalen: u32);

  /* Alternate compression function: just write an abbreviated table file */
  pub fn jpeg_write_tables(cinfo: j_compress_ptr);

  /* Decompression startup: read start of JPEG datastream to see what's there */
  pub fn jpeg_read_header(cinfo: j_decompress_ptr, require_image: boolean) -> c_int;
  /* Return value is one of: */

  /* Main entry points for decompression */
  pub fn jpeg_start_decompress(cinfo: j_decompress_ptr) -> boolean;
  pub fn jpeg_read_scanlines(cinfo: j_decompress_ptr, scanlines: JSAMPARRAY, max_lines: JDIMENSION) -> JDIMENSION;
  pub fn jpeg_finish_decompress(cinfo: j_decompress_ptr) -> boolean;

  /* Replaces jpeg_read_scanlines when reading raw downsampled data. */
  pub fn jpeg_read_raw_data(cinfo: j_decompress_ptr, data: JSAMPIMAGE, max_lines: JDIMENSION) -> JDIMENSION;

  /* Additional entry points for buffered-image mode. */
  pub fn jpeg_has_multiple_scans(cinfo: j_decompress_ptr) -> boolean;
  pub fn jpeg_start_output(cinfo: j_decompress_ptr, scan_number: c_int) -> boolean;
  pub fn jpeg_finish_output(cinfo: j_decompress_ptr) -> boolean;
  pub fn jpeg_input_complete(cinfo: j_decompress_ptr) -> boolean;
  pub fn jpeg_new_colormap(cinfo: j_decompress_ptr);
  pub fn jpeg_consume_input(cinfo: j_decompress_ptr) -> c_int;

  /* Precalculate output dimensions for current decompression parameters. */
  pub fn jpeg_calc_output_dimensions(cinfo: j_decompress_ptr);

  /* Install a special processing method for COM or APPn markers. */
  pub fn jpeg_set_marker_processor(cinfo: j_decompress_ptr, marker_code: c_int, routine: Option<jpeg_marker_parser_method>);

  /* Read or write raw DCT coefficients --- useful for lossless transcoding. */
  pub fn jpeg_read_coefficients(cinfo: j_decompress_ptr) -> *mut jvirt_barray_ptr;
  pub fn jpeg_write_coefficients(cinfo: j_compress_ptr, coef_arrays: *mut jvirt_barray_ptr);
  pub fn jpeg_copy_critical_parameters(srcinfo: j_decompress_ptr, dstinfo: j_compress_ptr);

  /* If you choose to abort compression or decompression before completing
   * jpeg_finish_(de)compress, then you need to clean up to release memory,
   * temporary files, etc.  You can just call jpeg_destroy_(de)compress
   * if you're done with the JPEG object, but if you want to clean it up and
   * reuse it, call this:
   */
  pub fn jpeg_abort_compress(cinfo: j_compress_ptr);
  pub fn jpeg_abort_decompress(cinfo: j_decompress_ptr);

  /* Generic versions of jpeg_abort and jpeg_destroy that work on either
   * flavor of JPEG object.  These may be more convenient in some places.
   */
  pub fn jpeg_abort(cinfo: j_common_ptr);
  pub fn jpeg_destroy(cinfo: j_common_ptr);

  /* Default restart-marker-resync procedure for use by data source modules */
  pub fn jpeg_resync_to_restart(cinfo: j_decompress_ptr, desired: c_int) -> boolean;
}

/* Return value codes for jpeg_read_header() */
pub const JPEG_SUSPENDED: c_int = 0;        /* Suspended due to lack of input data */
pub const JPEG_HEADER_OK: c_int = 1;        /* Found valid image datastream */
pub const JPEG_HEADER_TABLES_ONLY: c_int = 2; /* Found valid table-specs-only datastream */
/* If you pass require_image = TRUE (normal case), you need not check for
 * a TABLES_ONLY return code; an abbreviated file will cause an error exit.
 * JPEG_SUSPENDED is only possible if you use a data source module that can
 * give a suspension return (the stdio source module doesn't).
 */

/* Return value codes for jpeg_consume_input() */
pub const JPEG_REACHED_SOS: c_int = 1;    /* Reached start of new scan */
pub const JPEG_REACHED_EOI: c_int = 2;    /* Reached end of image */
pub const JPEG_ROW_COMPLETED: c_int = 3;  /* Completed one iMCU row */
pub const JPEG_SCAN_COMPLETED: c_int = 4; /* Completed last iMCU row of a scan */

/* These marker codes are exported since applications and data source modules
 * are likely to want to use them.
 */

pub const JPEG_RST0: c_int = 0xD0;         /* RST0 marker code */
pub const JPEG_EOI: c_int = 0xD9;          /* EOI marker code */
pub const JPEG_APP0: c_int = 0xE0;         /* APP0 marker code */
pub const JPEG_COM: c_int = 0xFE;          /* COM marker code */
