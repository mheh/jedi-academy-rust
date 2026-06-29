/*
 * jdmaster.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains master control logic for the JPEG decompressor.
 * These routines are concerned with selecting the modules to be executed
 * and with determining the number of passes and the work to be done in each
 * pass.
 */

#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::{c_int, c_long, c_void};


/* Forward declarations of JPEG library types */
pub type j_decompress_ptr = *mut c_void;
pub type j_common_ptr = *mut c_void;
pub type JSAMPLE = u8;
pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JDIMENSION = core::ffi::c_uint;
pub type boolean = u8;

const TRUE: boolean = 1;
const FALSE: boolean = 0;

/* Private state */

#[repr(C)]
pub struct my_decomp_master {
  pub pub_: jpeg_decomp_master, /* public fields */

  pub pass_number: c_int,		/* # of passes completed */

  pub using_merged_upsample: boolean, /* TRUE if using merged upsample/cconvert */

  /* Saved references to initialized quantizer modules,
   * in case we need to switch modes.
   */
  pub quantizer_1pass: *mut c_void,
  pub quantizer_2pass: *mut c_void,
}

pub type my_master_ptr = *mut my_decomp_master;


#[repr(C)]
pub struct jpeg_decomp_master {
  pub prepare_for_output_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
  pub finish_output_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
  pub is_dummy_pass: boolean,
}

#[repr(C)]
pub struct jpeg_color_converter {
  pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
}

#[repr(C)]
pub struct jpeg_upsampler {
  pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
}

#[repr(C)]
pub struct jpeg_d_post_controller {
  pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr, c_int)>,
}

#[repr(C)]
pub struct jpeg_inverse_dct {
  pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
}

#[repr(C)]
pub struct jpeg_d_coef_controller {
  pub start_output_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
}

#[repr(C)]
pub struct jpeg_d_main_controller {
  pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr, c_int)>,
  pub process_data: Option<unsafe extern "C" fn(j_decompress_ptr, JSAMPARRAY, *mut JDIMENSION, JDIMENSION)>,
}

#[repr(C)]
pub struct jpeg_memory_mgr {
  pub alloc_small: Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void>,
  pub realize_virt_arrays: Option<unsafe extern "C" fn(j_common_ptr)>,
}

#[repr(C)]
pub struct jpeg_input_controller {
  pub has_multiple_scans: boolean,
  pub eoi_reached: boolean,
  pub start_input_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
  pub consume_input: Option<unsafe extern "C" fn(j_decompress_ptr) -> c_int>,
}

#[repr(C)]
pub struct jpeg_progress_mgr {
  pub pass_counter: c_long,
  pub pass_limit: c_long,
  pub completed_passes: c_int,
  pub total_passes: c_int,
}

#[repr(C)]
pub struct jpeg_color_quantizer {
  pub start_pass: Option<unsafe extern "C" fn(j_decompress_ptr, boolean)>,
  pub finish_pass: Option<unsafe extern "C" fn(j_decompress_ptr)>,
  pub new_color_map: Option<unsafe extern "C" fn(j_decompress_ptr)>,
}

#[repr(C)]
pub struct jpeg_component_info {
  pub h_samp_factor: c_int,
  pub v_samp_factor: c_int,
  pub DCT_scaled_size: c_int,
  pub downsampled_width: JDIMENSION,
  pub downsampled_height: JDIMENSION,
}

/* Constants */
const MAXJSAMPLE: c_int = 255;
const CENTERJSAMPLE: c_int = 128;
const DCTSIZE: c_int = 8;
const JPOOL_IMAGE: c_int = 1;
const DSTATE_READY: c_int = 202;
const DSTATE_BUFIMAGE: c_int = 207;
const RGB_PIXELSIZE: c_int = 3;
const JCS_GRAYSCALE: c_int = 1;
const JCS_RGB: c_int = 2;
const JCS_YCbCr: c_int = 3;
const JCS_CMYK: c_int = 4;
const JCS_YCCK: c_int = 5;
const JERR_BAD_STATE: c_int = 203;
const JERR_WIDTH_OVERFLOW: c_int = 214;
const JERR_NOTIMPL: c_int = 240;
const JERR_NOT_COMPILED: c_int = 241;
const JERR_MODE_CHANGE: c_int = 201;
const JERR_ARITH_NOTIMPL: c_int = 205;
const JBUF_PASS_THRU: c_int = 0;
const JBUF_CRANK_DEST: c_int = 2;
const JBUF_SAVE_AND_PASS: c_int = 3;

/* Macro equivalents */
#[inline]
fn SIZEOF<T>() -> usize {
    core::mem::size_of::<T>()
}

#[inline]
fn MEMZERO(ptr: *mut u8, len: usize) {
    unsafe {
        core::ptr::write_bytes(ptr, 0, len);
    }
}

#[inline]
fn MEMCOPY(dest: *mut u8, src: *mut u8, len: usize) {
    unsafe {
        core::ptr::copy_nonoverlapping(src, dest, len);
    }
}

/* Stub error macros - actual implementations would be in error handler */
#[inline]
unsafe fn ERREXIT(_cinfo: j_decompress_ptr, _code: c_int) {
    // Stub
}

#[inline]
unsafe fn ERREXIT1(_cinfo: j_decompress_ptr, _code: c_int, _arg: c_int) {
    // Stub
}

/* Forward declarations */
extern "C" {
  fn jinit_1pass_quantizer(cinfo: j_decompress_ptr);
  fn jinit_2pass_quantizer(cinfo: j_decompress_ptr);
  fn jinit_merged_upsampler(cinfo: j_decompress_ptr);
  fn jinit_color_deconverter(cinfo: j_decompress_ptr);
  fn jinit_upsampler(cinfo: j_decompress_ptr);
  fn jinit_d_post_controller(cinfo: j_decompress_ptr, need_buffer: boolean);
  fn jinit_inverse_dct(cinfo: j_decompress_ptr);
  fn jinit_phuff_decoder(cinfo: j_decompress_ptr);
  fn jinit_huff_decoder(cinfo: j_decompress_ptr);
  fn jinit_d_coef_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean);
  fn jinit_d_main_controller(cinfo: j_decompress_ptr, need_full_buffer: boolean);
}

fn jdiv_round_up(a: c_long, b: i64) -> c_long {
  ((a + (b as c_long) - 1) / (b as c_long))
}


/*
 * Determine whether merged upsample/color conversion should be used.
 * CRUCIAL: this must match the actual capabilities of jdmerge.c!
 */

fn use_merged_upsample (cinfo: j_decompress_ptr) -> boolean
{
#[cfg(feature = "UPSAMPLE_MERGING_SUPPORTED")]
{
  unsafe {
    /* Merging is the equivalent of plain box-filter upsampling */
    let cinfo_ref = &*(cinfo as *mut j_decompress_info);
    if cinfo_ref.do_fancy_upsampling != 0 || cinfo_ref.CCIR601_sampling != 0 {
      return FALSE;
    }
    /* jdmerge.c only supports YCC=>RGB color conversion */
    if cinfo_ref.jpeg_color_space != JCS_YCbCr || cinfo_ref.num_components != 3 ||
        cinfo_ref.out_color_space != JCS_RGB ||
        cinfo_ref.out_color_components != RGB_PIXELSIZE {
      return FALSE;
    }
    /* and it only handles 2h1v or 2h2v sampling ratios */
    let comp_info = cinfo_ref.comp_info;
    if (*comp_info.add(0)).h_samp_factor != 2 ||
        (*comp_info.add(1)).h_samp_factor != 1 ||
        (*comp_info.add(2)).h_samp_factor != 1 ||
        (*comp_info.add(0)).v_samp_factor >  2 ||
        (*comp_info.add(1)).v_samp_factor != 1 ||
        (*comp_info.add(2)).v_samp_factor != 1 {
      return FALSE;
    }
    /* furthermore, it doesn't work if we've scaled the IDCTs differently */
    if (*comp_info.add(0)).DCT_scaled_size != cinfo_ref.min_DCT_scaled_size ||
        (*comp_info.add(1)).DCT_scaled_size != cinfo_ref.min_DCT_scaled_size ||
        (*comp_info.add(2)).DCT_scaled_size != cinfo_ref.min_DCT_scaled_size {
      return FALSE;
    }
    /* ??? also need to test for upsample-time rescaling, when & if supported */
    return TRUE;			/* by golly, it'll work... */
  }
}
#[cfg(not(feature = "UPSAMPLE_MERGING_SUPPORTED"))]
{
  FALSE
}
}


/*
 * Compute output image dimensions and related values.
 * NOTE: this is exported for possible use by application.
 * Hence it mustn't do anything that can't be done twice.
 * Also note that it may be called before the master module is initialized!
 */

pub unsafe fn jpeg_calc_output_dimensions (cinfo: j_decompress_ptr)
/* Do computations that are needed before master selection phase */
{
  // JDC: commented out to remove warning
  // int ci;
  // jpeg_component_info *compptr;

  let cinfo_ref = &mut *(cinfo as *mut j_decompress_info);

  /* Prevent application from calling me at wrong times */
  if cinfo_ref.global_state != DSTATE_READY {
    ERREXIT1(cinfo, JERR_BAD_STATE, cinfo_ref.global_state);
    return;
  }

#[cfg(feature = "IDCT_SCALING_SUPPORTED")]
{
  /* Compute actual output image dimensions and DCT scaling choices. */
  if cinfo_ref.scale_num * 8 <= cinfo_ref.scale_denom {
    /* Provide 1/8 scaling */
    cinfo_ref.output_width = jdiv_round_up(cinfo_ref.image_width as c_long, 8i64) as JDIMENSION;
    cinfo_ref.output_height = jdiv_round_up(cinfo_ref.image_height as c_long, 8i64) as JDIMENSION;
    cinfo_ref.min_DCT_scaled_size = 1;
  } else if cinfo_ref.scale_num * 4 <= cinfo_ref.scale_denom {
    /* Provide 1/4 scaling */
    cinfo_ref.output_width = jdiv_round_up(cinfo_ref.image_width as c_long, 4i64) as JDIMENSION;
    cinfo_ref.output_height = jdiv_round_up(cinfo_ref.image_height as c_long, 4i64) as JDIMENSION;
    cinfo_ref.min_DCT_scaled_size = 2;
  } else if cinfo_ref.scale_num * 2 <= cinfo_ref.scale_denom {
    /* Provide 1/2 scaling */
    cinfo_ref.output_width = jdiv_round_up(cinfo_ref.image_width as c_long, 2i64) as JDIMENSION;
    cinfo_ref.output_height = jdiv_round_up(cinfo_ref.image_height as c_long, 2i64) as JDIMENSION;
    cinfo_ref.min_DCT_scaled_size = 4;
  } else {
    /* Provide 1/1 scaling */
    cinfo_ref.output_width = cinfo_ref.image_width;
    cinfo_ref.output_height = cinfo_ref.image_height;
    cinfo_ref.min_DCT_scaled_size = DCTSIZE;
  }
  /* In selecting the actual DCT scaling for each component, we try to
   * scale up the chroma components via IDCT scaling rather than upsampling.
   * This saves time if the upsampler gets to use 1:1 scaling.
   * Note this code assumes that the supported DCT scalings are powers of 2.
   */
  {
    let mut ci: c_int = 0;
    while ci < cinfo_ref.num_components {
      let compptr = cinfo_ref.comp_info.add(ci as usize);
      let mut ssize = cinfo_ref.min_DCT_scaled_size;
      while ssize < DCTSIZE &&
	   ((*compptr).h_samp_factor * ssize * 2 <=
	    cinfo_ref.max_h_samp_factor * cinfo_ref.min_DCT_scaled_size) &&
	   ((*compptr).v_samp_factor * ssize * 2 <=
	    cinfo_ref.max_v_samp_factor * cinfo_ref.min_DCT_scaled_size) {
        ssize = ssize * 2;
      }
      (*compptr).DCT_scaled_size = ssize;
      ci += 1;
    }
  }

  /* Recompute downsampled dimensions of components;
   * application needs to know these if using raw downsampled data.
   */
  {
    let mut ci: c_int = 0;
    while ci < cinfo_ref.num_components {
      let compptr = cinfo_ref.comp_info.add(ci as usize);
      /* Size in samples, after IDCT scaling */
      (*compptr).downsampled_width = jdiv_round_up(
        (cinfo_ref.image_width as c_long) *
        ((*compptr).h_samp_factor as c_long * (*compptr).DCT_scaled_size as c_long),
        (cinfo_ref.max_h_samp_factor as c_long * DCTSIZE as c_long)
      ) as JDIMENSION;
      (*compptr).downsampled_height = jdiv_round_up(
        (cinfo_ref.image_height as c_long) *
        ((*compptr).v_samp_factor as c_long * (*compptr).DCT_scaled_size as c_long),
        (cinfo_ref.max_v_samp_factor as c_long * DCTSIZE as c_long)
      ) as JDIMENSION;
      ci += 1;
    }
  }
}
#[cfg(not(feature = "IDCT_SCALING_SUPPORTED"))]
{
  let cinfo_ref = &mut *(cinfo as *mut j_decompress_info);
  /* Hardwire it to "no scaling" */
  cinfo_ref.output_width = cinfo_ref.image_width;
  cinfo_ref.output_height = cinfo_ref.image_height;
  /* jdinput.c has already initialized DCT_scaled_size to DCTSIZE,
   * and has computed unscaled downsampled_width and downsampled_height.
   */
}

  let cinfo_ref = &mut *(cinfo as *mut j_decompress_info);

  /* Report number of components in selected colorspace. */
  /* Probably this should be in the color conversion module... */
  if cinfo_ref.out_color_space == JCS_GRAYSCALE {
    cinfo_ref.out_color_components = 1;
  } else if cinfo_ref.out_color_space == JCS_RGB {
    #[cfg(feature = "RGB_PIXELSIZE_NOT_3")]
    {
      cinfo_ref.out_color_components = RGB_PIXELSIZE;
    }
    #[cfg(not(feature = "RGB_PIXELSIZE_NOT_3"))]
    {
      /* else share code with YCbCr */
      cinfo_ref.out_color_components = 3;
    }
  } else if cinfo_ref.out_color_space == JCS_YCbCr {
    cinfo_ref.out_color_components = 3;
  } else if cinfo_ref.out_color_space == JCS_CMYK || cinfo_ref.out_color_space == JCS_YCCK {
    cinfo_ref.out_color_components = 4;
  } else {
    /* else must be same colorspace as in file */
    cinfo_ref.out_color_components = cinfo_ref.num_components;
  }
  cinfo_ref.output_components = if cinfo_ref.quantize_colors != 0 { 1 } else { cinfo_ref.out_color_components };

  /* See if upsampler will want to emit more than one row at a time */
  if use_merged_upsample(cinfo) != 0 {
    cinfo_ref.rec_outbuf_height = cinfo_ref.max_v_samp_factor;
  } else {
    cinfo_ref.rec_outbuf_height = 1;
  }
}


/*
 * Several decompression processes need to range-limit values to the range
 * 0..MAXJSAMPLE; the input value may fall somewhat outside this range
 * due to noise introduced by quantization, roundoff error, etc.  These
 * processes are inner loops and need to be as fast as possible.  On most
 * machines, particularly CPUs with pipelines or instruction prefetch,
 * a (subscript-check-less) C table lookup
 *		x = sample_range_limit[x];
 * is faster than explicit tests
 *		if (x < 0)  x = 0;
 *		else if (x > MAXJSAMPLE)  x = MAXJSAMPLE;
 * These processes all use a common table prepared by the routine below.
 *
 * For most steps we can mathematically guarantee that the initial value
 * of x is within MAXJSAMPLE+1 of the legal range, so a table running from
 * -(MAXJSAMPLE+1) to 2*MAXJSAMPLE+1 is sufficient.  But for the initial
 * limiting step (just after the IDCT), a wildly out-of-range value is
 * possible if the input data is corrupt.  To avoid any chance of indexing
 * off the end of memory and getting a bad-pointer trap, we perform the
 * post-IDCT limiting thus:
 *		x = range_limit[x & MASK];
 * where MASK is 2 bits wider than legal sample data, ie 10 bits for 8-bit
 * samples.  Under normal circumstances this is more than enough range and
 * a correct output will be generated; with bogus input data the mask will
 * cause wraparound, and we will safely generate a bogus-but-in-range output.
 * For the post-IDCT step, we want to convert the data from signed to unsigned
 * representation by adding CENTERJSAMPLE at the same time that we limit it.
 * So the post-IDCT limiting table ends up looking like this:
 *   CENTERJSAMPLE,CENTERJSAMPLE+1,...,MAXJSAMPLE,
 *   MAXJSAMPLE (repeat 2*(MAXJSAMPLE+1)-CENTERJSAMPLE times),
 *   0          (repeat 2*(MAXJSAMPLE+1)-CENTERJSAMPLE times),
 *   0,1,...,CENTERJSAMPLE-1
 * Negative inputs select values from the upper half of the table after
 * masking.
 *
 * We can save some space by overlapping the start of the post-IDCT table
 * with the simpler range limiting table.  The post-IDCT table begins at
 * sample_range_limit + CENTERJSAMPLE.
 *
 * Note that the table is allocated in near data space on PCs; it's small
 * enough and used often enough to justify this.
 */

fn prepare_range_limit_table (cinfo: j_decompress_ptr)
/* Allocate and fill in the sample_range_limit table */
{
  unsafe {
    let cinfo_ref = &mut *(cinfo as *mut j_decompress_info);
    let table: *mut JSAMPLE;
    let mut i: c_int;

    table = ((*cinfo_ref.mem).alloc_small.unwrap())(
      cinfo,
      JPOOL_IMAGE,
      (5 * (MAXJSAMPLE as usize + 1) + CENTERJSAMPLE as usize) * SIZEOF::<JSAMPLE>()
    ) as *mut JSAMPLE;

    let table_offset = table.add((MAXJSAMPLE + 1) as usize);	/* allow negative subscripts of simple table */
    cinfo_ref.sample_range_limit = table_offset;

    /* First segment of "simple" table: limit[x] = 0 for x < 0 */
    MEMZERO(
      table_offset.sub((MAXJSAMPLE as usize + 1)) as *mut u8,
      (MAXJSAMPLE as usize + 1) * SIZEOF::<JSAMPLE>()
    );

    /* Main part of "simple" table: limit[x] = x */
    i = 0;
    while i <= MAXJSAMPLE {
      *table_offset.add(i as usize) = i as JSAMPLE;
      i += 1;
    }

    let table2 = table_offset.add(CENTERJSAMPLE as usize);	/* Point to where post-IDCT table starts */

    /* End of simple table, rest of first half of post-IDCT table */
    i = CENTERJSAMPLE;
    while i < 2 * (MAXJSAMPLE + 1) {
      *table2.add(i as usize) = MAXJSAMPLE as JSAMPLE;
      i += 1;
    }

    /* Second half of post-IDCT table */
    MEMZERO(
      table2.add((2 * (MAXJSAMPLE + 1)) as usize) as *mut u8,
      (2 * (MAXJSAMPLE + 1) - CENTERJSAMPLE) as usize * SIZEOF::<JSAMPLE>()
    );

    MEMCOPY(
      table2.add((4 * (MAXJSAMPLE + 1) - CENTERJSAMPLE) as usize) as *mut u8,
      table_offset as *mut u8,
      CENTERJSAMPLE as usize * SIZEOF::<JSAMPLE>()
    );
  }
}


/*
 * Master selection of decompression modules.
 * This is done once at jpeg_start_decompress time.  We determine
 * which modules will be used and give them appropriate initialization calls.
 * We also initialize the decompressor input side to begin consuming data.
 *
 * Since jpeg_read_header has finished, we know what is in the SOF
 * and (first) SOS markers.  We also have all the application parameter
 * settings.
 */

fn master_selection (cinfo: j_decompress_ptr)
{
  unsafe {
    let cinfo_ref = &mut *(cinfo as *mut j_decompress_info);
    let master: my_master_ptr = cinfo_ref.master as my_master_ptr;
    let use_c_buffer: boolean;
    let samplesperrow: c_long;
    let jd_samplesperrow: JDIMENSION;

    /* Initialize dimensions and other stuff */
    jpeg_calc_output_dimensions(cinfo);
    prepare_range_limit_table(cinfo);

    /* Width of an output scanline must be representable as JDIMENSION. */
    samplesperrow = (cinfo_ref.output_width as c_long) * (cinfo_ref.out_color_components as c_long);
    jd_samplesperrow = samplesperrow as JDIMENSION;
    if (jd_samplesperrow as c_long) != samplesperrow {
      ERREXIT(cinfo, JERR_WIDTH_OVERFLOW);
    }

    /* Initialize my private state */
    (*master).pass_number = 0;
    (*master).using_merged_upsample = use_merged_upsample(cinfo);

    /* Color quantizer selection */
    (*master).quantizer_1pass = core::ptr::null_mut();
    (*master).quantizer_2pass = core::ptr::null_mut();
    /* No mode changes if not using buffered-image mode. */
    if cinfo_ref.quantize_colors == 0 || cinfo_ref.buffered_image == 0 {
      cinfo_ref.enable_1pass_quant = FALSE;
      cinfo_ref.enable_external_quant = FALSE;
      cinfo_ref.enable_2pass_quant = FALSE;
    }
    if cinfo_ref.quantize_colors != 0 {
      if cinfo_ref.raw_data_out != 0 {
        ERREXIT(cinfo, JERR_NOTIMPL);
      }
      /* 2-pass quantizer only works in 3-component color space. */
      if cinfo_ref.out_color_components != 3 {
        cinfo_ref.enable_1pass_quant = TRUE;
        cinfo_ref.enable_external_quant = FALSE;
        cinfo_ref.enable_2pass_quant = FALSE;
        cinfo_ref.colormap = core::ptr::null_mut();
      } else if !cinfo_ref.colormap.is_null() {
        cinfo_ref.enable_external_quant = TRUE;
      } else if cinfo_ref.two_pass_quantize != 0 {
        cinfo_ref.enable_2pass_quant = TRUE;
      } else {
        cinfo_ref.enable_1pass_quant = TRUE;
      }

      if cinfo_ref.enable_1pass_quant != 0 {
#[cfg(feature = "QUANT_1PASS_SUPPORTED")]
{
        jinit_1pass_quantizer(cinfo);
        (*master).quantizer_1pass = cinfo_ref.cquantize;
}
#[cfg(not(feature = "QUANT_1PASS_SUPPORTED"))]
{
        ERREXIT(cinfo, JERR_NOT_COMPILED);
}
      }

      /* We use the 2-pass code to map to external colormaps. */
      if cinfo_ref.enable_2pass_quant != 0 || cinfo_ref.enable_external_quant != 0 {
#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
{
        jinit_2pass_quantizer(cinfo);
        (*master).quantizer_2pass = cinfo_ref.cquantize;
}
#[cfg(not(feature = "QUANT_2PASS_SUPPORTED"))]
{
        ERREXIT(cinfo, JERR_NOT_COMPILED);
}
      }
      /* If both quantizers are initialized, the 2-pass one is left active;
       * this is necessary for starting with quantization to an external map.
       */
    }

    /* Post-processing: in particular, color conversion first */
    if cinfo_ref.raw_data_out == 0 {
      if (*master).using_merged_upsample != 0 {
#[cfg(feature = "UPSAMPLE_MERGING_SUPPORTED")]
{
        jinit_merged_upsampler(cinfo); /* does color conversion too */
}
#[cfg(not(feature = "UPSAMPLE_MERGING_SUPPORTED"))]
{
        ERREXIT(cinfo, JERR_NOT_COMPILED);
}
      } else {
        jinit_color_deconverter(cinfo);
        jinit_upsampler(cinfo);
      }
      jinit_d_post_controller(cinfo, cinfo_ref.enable_2pass_quant);
    }
    /* Inverse DCT */
    jinit_inverse_dct(cinfo);
    /* Entropy decoding: either Huffman or arithmetic coding. */
    if cinfo_ref.arith_code != 0 {
      ERREXIT(cinfo, JERR_ARITH_NOTIMPL);
    } else {
      if cinfo_ref.progressive_mode != 0 {
#[cfg(feature = "D_PROGRESSIVE_SUPPORTED")]
{
        jinit_phuff_decoder(cinfo);
}
#[cfg(not(feature = "D_PROGRESSIVE_SUPPORTED"))]
{
        ERREXIT(cinfo, JERR_NOT_COMPILED);
}
      } else {
        jinit_huff_decoder(cinfo);
      }
    }

    /* Initialize principal buffer controllers. */
    use_c_buffer = if (*cinfo_ref.inputctl).has_multiple_scans != 0 || cinfo_ref.buffered_image != 0 { TRUE } else { FALSE };
    jinit_d_coef_controller(cinfo, use_c_buffer);

    if cinfo_ref.raw_data_out == 0 {
      jinit_d_main_controller(cinfo, FALSE /* never need full buffer here */);
    }

    /* We can now tell the memory manager to allocate virtual arrays. */
    ((*cinfo_ref.mem).realize_virt_arrays.unwrap())(cinfo);

    /* Initialize input side of decompressor to consume first scan. */
    ((*cinfo_ref.inputctl).start_input_pass.unwrap())(cinfo);

#[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
{
    /* If jpeg_start_decompress will read the whole file, initialize
     * progress monitoring appropriately.  The input step is counted
     * as one pass.
     */
    if !cinfo_ref.progress.is_null() && cinfo_ref.buffered_image == 0 &&
        (*cinfo_ref.inputctl).has_multiple_scans != 0 {
      let mut nscans: c_int;
      /* Estimate number of scans to set pass_limit. */
      if cinfo_ref.progressive_mode != 0 {
        /* Arbitrarily estimate 2 interleaved DC scans + 3 AC scans/component. */
        nscans = 2 + 3 * cinfo_ref.num_components;
      } else {
        /* For a nonprogressive multiscan file, estimate 1 scan per component. */
        nscans = cinfo_ref.num_components;
      }
      (*cinfo_ref.progress).pass_counter = 0;
      (*cinfo_ref.progress).pass_limit = (cinfo_ref.total_iMCU_rows as c_long) * (nscans as c_long);
      (*cinfo_ref.progress).completed_passes = 0;
      (*cinfo_ref.progress).total_passes = if cinfo_ref.enable_2pass_quant != 0 { 3 } else { 2 };
      /* Count the input pass as done */
      (*master).pass_number += 1;
    }
}
  }
}


/*
 * Per-pass setup.
 * This is called at the beginning of each output pass.  We determine which
 * modules will be active during this pass and give them appropriate
 * start_pass calls.  We also set is_dummy_pass to indicate whether this
 * is a "real" output pass or a dummy pass for color quantization.
 * (In the latter case, jdapi.c will crank the pass to completion.)
 */

pub unsafe fn prepare_for_output_pass (cinfo: j_decompress_ptr)
{
  let cinfo_ref = &mut *(cinfo as *mut j_decompress_info);
  let master: my_master_ptr = cinfo_ref.master as my_master_ptr;

  if (*(*master).pub_).is_dummy_pass != 0 {
#[cfg(feature = "QUANT_2PASS_SUPPORTED")]
{
    /* Final pass of 2-pass quantization */
    (*(*master).pub_).is_dummy_pass = FALSE;
    ((*cinfo_ref.cquantize).start_pass.unwrap())(cinfo, FALSE);
    ((*cinfo_ref.post).start_pass.unwrap())(cinfo, JBUF_CRANK_DEST);
    ((*cinfo_ref.main).start_pass.unwrap())(cinfo, JBUF_CRANK_DEST);
}
#[cfg(not(feature = "QUANT_2PASS_SUPPORTED"))]
{
    ERREXIT(cinfo, JERR_NOT_COMPILED);
}
  } else {
    if cinfo_ref.quantize_colors != 0 && cinfo_ref.colormap.is_null() {
      /* Select new quantization method */
      if cinfo_ref.two_pass_quantize != 0 && cinfo_ref.enable_2pass_quant != 0 {
	cinfo_ref.cquantize = (*master).quantizer_2pass;
	(*(*master).pub_).is_dummy_pass = TRUE;
      } else if cinfo_ref.enable_1pass_quant != 0 {
	cinfo_ref.cquantize = (*master).quantizer_1pass;
      } else {
	ERREXIT(cinfo, JERR_MODE_CHANGE);
      }
    }
    ((*cinfo_ref.idct).start_pass.unwrap())(cinfo);
    ((*cinfo_ref.coef).start_output_pass.unwrap())(cinfo);
    if cinfo_ref.raw_data_out == 0 {
      if (*master).using_merged_upsample == 0 {
	((*cinfo_ref.cconvert).start_pass.unwrap())(cinfo);
      }
      ((*cinfo_ref.upsample).start_pass.unwrap())(cinfo);
      if cinfo_ref.quantize_colors != 0 {
	((*cinfo_ref.cquantize).start_pass.unwrap())(cinfo, (*(*master).pub_).is_dummy_pass);
      }
      ((*cinfo_ref.post).start_pass.unwrap())(
        cinfo,
	if (*(*master).pub_).is_dummy_pass != 0 { JBUF_SAVE_AND_PASS } else { JBUF_PASS_THRU }
      );
      ((*cinfo_ref.main).start_pass.unwrap())(cinfo, JBUF_PASS_THRU);
    }
  }

  /* Set up progress monitor's pass info if present */
  if !cinfo_ref.progress.is_null() {
    (*cinfo_ref.progress).completed_passes = (*master).pass_number;
    (*cinfo_ref.progress).total_passes = (*master).pass_number +
					    if (*(*master).pub_).is_dummy_pass != 0 { 2 } else { 1 };
    /* In buffered-image mode, we assume one more output pass if EOI not
     * yet reached, but no more passes if EOI has been reached.
     */
    if cinfo_ref.buffered_image != 0 && (*cinfo_ref.inputctl).eoi_reached == 0 {
      (*cinfo_ref.progress).total_passes += if cinfo_ref.enable_2pass_quant != 0 { 2 } else { 1 };
    }
  }
}


/*
 * Finish up at end of an output pass.
 */

pub unsafe fn finish_output_pass (cinfo: j_decompress_ptr)
{
  let cinfo_ref = &mut *(cinfo as *mut j_decompress_info);
  let master: my_master_ptr = cinfo_ref.master as my_master_ptr;

  if cinfo_ref.quantize_colors != 0 {
    ((*cinfo_ref.cquantize).finish_pass.unwrap())(cinfo);
  }
  (*master).pass_number += 1;
}


#[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
/*
 * Switch to a new external colormap between output passes.
 */

pub unsafe fn jpeg_new_colormap (cinfo: j_decompress_ptr)
{
  let cinfo_ref = &mut *(cinfo as *mut j_decompress_info);
  let master: my_master_ptr = cinfo_ref.master as my_master_ptr;

  /* Prevent application from calling me at wrong times */
  if cinfo_ref.global_state != DSTATE_BUFIMAGE {
    ERREXIT1(cinfo, JERR_BAD_STATE, cinfo_ref.global_state);
    return;
  }

  if cinfo_ref.quantize_colors != 0 && cinfo_ref.enable_external_quant != 0 &&
      !cinfo_ref.colormap.is_null() {
    /* Select 2-pass quantizer for external colormap use */
    cinfo_ref.cquantize = (*master).quantizer_2pass;
    /* Notify quantizer of colormap change */
    ((*cinfo_ref.cquantize).new_color_map.unwrap())(cinfo);
    (*(*master).pub_).is_dummy_pass = FALSE; /* just in case */
  } else {
    ERREXIT(cinfo, JERR_MODE_CHANGE);
  }
}


/*
 * Initialize master decompression control and select active modules.
 * This is performed at the start of jpeg_start_decompress.
 */

pub unsafe fn jinit_master_decompress (cinfo: j_decompress_ptr)
{
  let cinfo_ref = &mut *(cinfo as *mut j_decompress_info);
  let master: my_master_ptr;

  master = ((*cinfo_ref.mem).alloc_small.unwrap())(
    cinfo,
    JPOOL_IMAGE,
    SIZEOF::<my_decomp_master>()
  ) as my_master_ptr;

  cinfo_ref.master = &mut (*master).pub_ as *mut jpeg_decomp_master;
  ((*cinfo_ref.master).prepare_for_output_pass) = Some(prepare_for_output_pass);
  ((*cinfo_ref.master).finish_output_pass) = Some(finish_output_pass);

  (*cinfo_ref.master).is_dummy_pass = FALSE;

  master_selection(cinfo);
}


/* Stub forward declaration for j_decompress_info */
pub struct j_decompress_info {
  pub global_state: c_int,
  pub image_width: JDIMENSION,
  pub image_height: JDIMENSION,
  pub output_width: JDIMENSION,
  pub output_height: JDIMENSION,
  pub out_color_space: c_int,
  pub jpeg_color_space: c_int,
  pub num_components: c_int,
  pub out_color_components: c_int,
  pub output_components: c_int,
  pub scale_num: c_int,
  pub scale_denom: c_int,
  pub min_DCT_scaled_size: c_int,
  pub max_h_samp_factor: c_int,
  pub max_v_samp_factor: c_int,
  pub rec_outbuf_height: c_int,
  pub do_fancy_upsampling: boolean,
  pub CCIR601_sampling: boolean,
  pub comp_info: *mut jpeg_component_info,
  pub quantize_colors: boolean,
  pub buffered_image: boolean,
  pub raw_data_out: boolean,
  pub colormap: *mut c_void,
  pub enable_1pass_quant: boolean,
  pub enable_2pass_quant: boolean,
  pub enable_external_quant: boolean,
  pub two_pass_quantize: boolean,
  pub sample_range_limit: *mut JSAMPLE,
  pub mem: *mut jpeg_memory_mgr,
  pub cquantize: *mut jpeg_color_quantizer,
  pub post: *mut jpeg_d_post_controller,
  pub main: *mut jpeg_d_main_controller,
  pub idct: *mut jpeg_inverse_dct,
  pub coef: *mut jpeg_d_coef_controller,
  pub cconvert: *mut jpeg_color_converter,
  pub upsample: *mut jpeg_upsampler,
  pub inputctl: *mut jpeg_input_controller,
  pub master: *mut jpeg_decomp_master,
  pub progress: *mut jpeg_progress_mgr,
  pub total_iMCU_rows: JDIMENSION,
  pub arith_code: boolean,
  pub progressive_mode: boolean,
}
