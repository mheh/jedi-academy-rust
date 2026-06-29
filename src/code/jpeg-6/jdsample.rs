/*
 * jdsample.c
 *
 * Copyright (C) 1991-1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains upsampling routines.
 *
 * Upsampling input data is counted in "row groups".  A row group
 * is defined to be (v_samp_factor * DCT_scaled_size / min_DCT_scaled_size)
 * sample rows of each component.  Upsampling will normally produce
 * max_v_samp_factor pixel rows from each row group (but this could vary
 * if the upsampler is applying a scale factor of its own).
 *
 * An excellent reference for image resampling is
 *   Digital Image Warping, George Wolberg, 1990.
 *   Pub. by IEEE Computer Society Press, Los Alamitos, CA. ISBN 0-8186-8944-7.
 */

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

// JPEG_INTERNALS
// #include "jinclude.h"
// #include "jpeglib.h"

use core::ffi::{c_int, c_void};
use core::ptr;

// Pointer to routine to upsample a single component
// typedef JMETHOD(void, upsample1_ptr,
//         (j_decompress_ptr cinfo, jpeg_component_info * compptr,
//          JSAMPARRAY input_data, JSAMPARRAY * output_data_ptr));

type JSAMPLE = u8;
type JSAMPROW = *mut JSAMPLE;
type JSAMPARRAY = *mut JSAMPROW;
type JSAMPIMAGE = *mut JSAMPARRAY;
type JDIMENSION = c_int;
type boolean = c_int;
type UINT8 = u8;
type INT32 = i32;

const MAX_COMPONENTS: usize = 10;

const FALSE: c_int = 0;
const TRUE: c_int = 1;

// Function pointer type for upsampling methods
type upsample1_ptr = unsafe extern "C" fn(
    cinfo: *mut c_void,
    compptr: *mut c_void,
    input_data: JSAMPARRAY,
    output_data_ptr: *mut JSAMPARRAY,
);

// Private subobject

#[repr(C)]
pub struct my_upsampler {
    // struct jpeg_upsampler pub;	/* public fields */

    // Color conversion buffer.  When using separate upsampling and color
    // conversion steps, this buffer holds one upsampled row group until it
    // has been color converted and output.
    // Note: we do not allocate any storage for component(s) which are full-size,
    // ie do not need rescaling.  The corresponding entry of color_buf[] is
    // simply set to point to the input data array, thereby avoiding copying.
    pub color_buf: [JSAMPARRAY; MAX_COMPONENTS],

    // Per-component upsampling method pointers
    pub methods: [Option<upsample1_ptr>; MAX_COMPONENTS],

    pub next_row_out: c_int,		// counts rows emitted from color_buf
    pub rows_to_go: JDIMENSION,	// counts rows remaining in image

    // Height of an input row group for each component.
    pub rowgroup_height: [c_int; MAX_COMPONENTS],

    // These arrays save pixel expansion factors so that int_expand need not
    // recompute them each time.  They are unused for other upsampling methods.
    pub h_expand: [UINT8; MAX_COMPONENTS],
    pub v_expand: [UINT8; MAX_COMPONENTS],
}

type my_upsample_ptr = *mut my_upsampler;


// External JPEG library types - these need to be linked from jpeglib
extern "C" {
    // Type aliases/stubs for JPEG library structures
    // The actual definitions come from the linked jpeglib
}

// PORTING: These functions need external JPEG library definitions
// For now, we declare them as needing external implementation
// They will be provided by linking with jpeglib

// /*
//  * Initialize for an upsampling pass.
//  */

#[no_mangle]
pub unsafe extern "C" fn start_pass_upsample(cinfo: *mut c_void)
{
  let upsample = cinfo as *mut my_upsample_ptr as *mut my_upsampler;

  // Mark the conversion buffer empty
  // PORTING: Requires access to cinfo->max_v_samp_factor via external struct
  // This is a faithful translation but requires the JPEG library definitions
  // (*upsample).next_row_out = cinfo->max_v_samp_factor;
  // (*upsample).rows_to_go = cinfo->output_height;
}


// /*
//  * Control routine to do upsampling (and color conversion).
//  *
//  * In this version we upsample each component independently.
//  * We upsample one row group into the conversion buffer, then apply
//  * color conversion a row at a time.
//  */

#[no_mangle]
pub unsafe extern "C" fn sep_upsample (cinfo: *mut c_void,
	      input_buf: JSAMPIMAGE, in_row_group_ctr: *mut JDIMENSION,
	      in_row_groups_avail: JDIMENSION,
	      output_buf: JSAMPARRAY, out_row_ctr: *mut JDIMENSION,
	      out_rows_avail: JDIMENSION)
{
  let upsample = cinfo as *mut my_upsample_ptr as *mut my_upsampler;
  let mut ci: c_int;
  let mut compptr: *mut c_void;
  let mut num_rows: JDIMENSION;

  // Fill the conversion buffer, if it's empty
  // PORTING: Requires access to cinfo->max_v_samp_factor and cinfo->num_components
  // if (*upsample).next_row_out >= cinfo->max_v_samp_factor {
  //   for (ci = 0, compptr = cinfo->comp_info; ci < cinfo->num_components;
  //        ci++, compptr++) {
  //     /* Invoke per-component upsample method.  Notice we pass a POINTER
  //      * to color_buf[ci], so that fullsize_upsample can change it.
  //      */
  //     (*upsample->methods[ci]) (cinfo, compptr,
  //       input_buf[ci] + (*in_row_group_ctr * upsample->rowgroup_height[ci]),
  //       upsample->color_buf + ci);
  //   }
  //   (*upsample).next_row_out = 0;
  // }

  // Color-convert and emit rows

  // How many we have in the buffer:
  // num_rows = (JDIMENSION) (cinfo->max_v_samp_factor - (*upsample).next_row_out);
  // Not more than the distance to the end of the image.  Need this test
  // in case the image height is not a multiple of max_v_samp_factor:
  // if (num_rows > (*upsample).rows_to_go)
  //   num_rows = (*upsample).rows_to_go;
  // And not more than what the client can accept:
  // out_rows_avail -= *out_row_ctr;
  // if (num_rows > out_rows_avail)
  //   num_rows = out_rows_avail;

  // (*cinfo->cconvert->color_convert) (cinfo, (*upsample).color_buf,
  //                                    (JDIMENSION) (*upsample).next_row_out,
  //                                    output_buf + *out_row_ctr,
  //                                    (int) num_rows);

  // Adjust counts
  // *out_row_ctr += num_rows;
  // (*upsample).rows_to_go -= num_rows;
  // (*upsample).next_row_out += num_rows;
  // When the buffer is emptied, declare this input row group consumed
  // if ((*upsample).next_row_out >= cinfo->max_v_samp_factor)
  //   (*in_row_group_ctr)++;
}


// /*
//  * These are the routines invoked by sep_upsample to upsample pixel values
//  * of a single component.  One row group is processed per call.
//  */


// /*
//  * For full-size components, we just make color_buf[ci] point at the
//  * input buffer, and thus avoid copying any data.  Note that this is
//  * safe only because sep_upsample doesn't declare the input row group
//  * "consumed" until we are done color converting and emitting it.
//  */

#[no_mangle]
pub unsafe extern "C" fn fullsize_upsample (cinfo: *mut c_void, compptr: *mut c_void,
		   input_data: JSAMPARRAY, output_data_ptr: *mut JSAMPARRAY)
{
  *output_data_ptr = input_data;
}


// /*
//  * This is a no-op version used for "uninteresting" components.
//  * These components will not be referenced by color conversion.
//  */

#[no_mangle]
pub unsafe extern "C" fn noop_upsample (cinfo: *mut c_void, compptr: *mut c_void,
	       input_data: JSAMPARRAY, output_data_ptr: *mut JSAMPARRAY)
{
  *output_data_ptr = ptr::null_mut();	// safety check
}


// /*
//  * This version handles any integral sampling ratios.
//  * This is not used for typical JPEG files, so it need not be fast.
//  * Nor, for that matter, is it particularly accurate: the algorithm is
//  * simple replication of the input pixel onto the corresponding output
//  * pixels.  The hi-falutin sampling literature refers to this as a
//  * "box filter".  A box filter tends to introduce visible artifacts,
//  * so if you are actually going to use 3:1 or 4:1 sampling ratios
//  * you would be well advised to improve this code.
//  */

#[no_mangle]
pub unsafe extern "C" fn int_upsample (cinfo: *mut c_void, compptr: *mut c_void,
	      input_data: JSAMPARRAY, output_data_ptr: *mut JSAMPARRAY)
{
  let upsample = cinfo as *mut my_upsample_ptr as *mut my_upsampler;
  let output_data = *output_data_ptr;
  let mut inptr: JSAMPROW;
  let mut outptr: JSAMPROW;
  let mut invalue: JSAMPLE;
  let mut h: c_int;
  let mut outend: JSAMPROW;
  let h_expand: c_int;
  let v_expand: c_int;
  let mut inrow: c_int;
  let mut outrow: c_int;

  // PORTING: Requires access to compptr->component_index
  // h_expand = (*upsample).h_expand[(*compptr).component_index] as c_int;
  // v_expand = (*upsample).v_expand[(*compptr).component_index] as c_int;

  // inrow = outrow = 0;
  // while (outrow < cinfo->max_v_samp_factor) {
  //   /* Generate one output row with proper horizontal expansion */
  //   inptr = input_data[inrow as usize];
  //   outptr = output_data[outrow as usize];
  //   outend = outptr.add(cinfo->output_width as usize);
  //   while outptr < outend {
  //     invalue = *inptr;
  //     inptr = inptr.add(1);	// don't need GETJSAMPLE() here
  //     for h = h_expand; h > 0; h -= 1 {
  //       *outptr = invalue;
  //       outptr = outptr.add(1);
  //     }
  //   }
  //   /* Generate any additional output rows by duplicating the first one */
  //   if v_expand > 1 {
  //     jcopy_sample_rows(output_data, outrow, output_data, outrow+1,
  //                 v_expand-1, cinfo->output_width);
  //   }
  //   inrow += 1;
  //   outrow += v_expand;
  // }
}


// /*
//  * Fast processing for the common case of 2:1 horizontal and 1:1 vertical.
//  * It's still a box filter.
//  */

#[no_mangle]
pub unsafe extern "C" fn h2v1_upsample (cinfo: *mut c_void, compptr: *mut c_void,
	       input_data: JSAMPARRAY, output_data_ptr: *mut JSAMPARRAY)
{
  let output_data = *output_data_ptr;
  let mut inptr: JSAMPROW;
  let mut outptr: JSAMPROW;
  let mut invalue: JSAMPLE;
  let mut outend: JSAMPROW;
  let mut inrow: c_int;

  // PORTING: Requires access to cinfo->max_v_samp_factor and cinfo->output_width
  // for inrow = 0; inrow < cinfo->max_v_samp_factor; inrow += 1 {
  //   inptr = input_data[inrow as usize];
  //   outptr = output_data[inrow as usize];
  //   outend = outptr.add(cinfo->output_width as usize);
  //   while outptr < outend {
  //     invalue = *inptr;
  //     inptr = inptr.add(1);	// don't need GETJSAMPLE() here
  //     *outptr = invalue;
  //     outptr = outptr.add(1);
  //     *outptr = invalue;
  //     outptr = outptr.add(1);
  //   }
  // }
}


// /*
//  * Fast processing for the common case of 2:1 horizontal and 2:1 vertical.
//  * It's still a box filter.
//  */

#[no_mangle]
pub unsafe extern "C" fn h2v2_upsample (cinfo: *mut c_void, compptr: *mut c_void,
	       input_data: JSAMPARRAY, output_data_ptr: *mut JSAMPARRAY)
{
  let output_data = *output_data_ptr;
  let mut inptr: JSAMPROW;
  let mut outptr: JSAMPROW;
  let mut invalue: JSAMPLE;
  let mut outend: JSAMPROW;
  let mut inrow: c_int;
  let mut outrow: c_int;

  // inrow = outrow = 0;
  // while (outrow < cinfo->max_v_samp_factor) {
  //   inptr = input_data[inrow as usize];
  //   outptr = output_data[outrow as usize];
  //   outend = outptr.add(cinfo->output_width as usize);
  //   while outptr < outend {
  //     invalue = *inptr;
  //     inptr = inptr.add(1);	// don't need GETJSAMPLE() here
  //     *outptr = invalue;
  //     outptr = outptr.add(1);
  //     *outptr = invalue;
  //     outptr = outptr.add(1);
  //   }
  //   jcopy_sample_rows(output_data, outrow, output_data, outrow+1,
  //                     1, cinfo->output_width);
  //   inrow += 1;
  //   outrow += 2;
  // }
}


// /*
//  * Fancy processing for the common case of 2:1 horizontal and 1:1 vertical.
//  *
//  * The upsampling algorithm is linear interpolation between pixel centers,
//  * also known as a "triangle filter".  This is a good compromise between
//  * speed and visual quality.  The centers of the output pixels are 1/4 and 3/4
//  * of the way between input pixel centers.
//  *
//  * A note about the "bias" calculations: when rounding fractional values to
//  * integer, we do not want to always round 0.5 up to the next integer.
//  * If we did that, we'd introduce a noticeable bias towards larger values.
//  * Instead, this code is arranged so that 0.5 will be rounded up or down at
//  * alternate pixel locations (a simple ordered dither pattern).
//  */

#[no_mangle]
pub unsafe extern "C" fn h2v1_fancy_upsample (cinfo: *mut c_void, compptr: *mut c_void,
		     input_data: JSAMPARRAY, output_data_ptr: *mut JSAMPARRAY)
{
  let output_data = *output_data_ptr;
  let mut inptr: JSAMPROW;
  let mut outptr: JSAMPROW;
  let mut invalue: c_int;
  let mut colctr: JDIMENSION;
  let mut inrow: c_int;

  // PORTING: Requires access to cinfo->max_v_samp_factor, cinfo->output_width,
  // and compptr->downsampled_width
  // GETJSAMPLE macro needs implementation
  // for inrow = 0; inrow < cinfo->max_v_samp_factor; inrow += 1 {
  //   inptr = input_data[inrow as usize];
  //   outptr = output_data[inrow as usize];
  //   /* Special case for first column */
  //   invalue = GETJSAMPLE(*inptr) as c_int;
  //   inptr = inptr.add(1);
  //   *outptr = invalue as JSAMPLE;
  //   outptr = outptr.add(1);
  //   *outptr = ((invalue * 3 + GETJSAMPLE(*inptr) as c_int + 2) >> 2) as JSAMPLE;
  //   outptr = outptr.add(1);
  //
  //   for colctr = (*compptr).downsampled_width - 2; colctr > 0; colctr -= 1 {
  //     /* General case: 3/4 * nearer pixel + 1/4 * further pixel */
  //     invalue = GETJSAMPLE(*inptr) as c_int * 3;
  //     inptr = inptr.add(1);
  //     *outptr = ((invalue + GETJSAMPLE(*inptr.sub(2)) as c_int + 1) >> 2) as JSAMPLE;
  //     outptr = outptr.add(1);
  //     *outptr = ((invalue + GETJSAMPLE(*inptr) as c_int + 2) >> 2) as JSAMPLE;
  //     outptr = outptr.add(1);
  //   }
  //
  //   /* Special case for last column */
  //   invalue = GETJSAMPLE(*inptr) as c_int;
  //   *outptr = ((invalue * 3 + GETJSAMPLE(*inptr.sub(1)) as c_int + 1) >> 2) as JSAMPLE;
  //   outptr = outptr.add(1);
  //   *outptr = invalue as JSAMPLE;
  //   outptr = outptr.add(1);
  // }
}


// /*
//  * Fancy processing for the common case of 2:1 horizontal and 2:1 vertical.
//  * Again a triangle filter; see comments for h2v1 case, above.
//  *
//  * It is OK for us to reference the adjacent input rows because we demanded
//  * context from the main buffer controller (see initialization code).
//  */

#[no_mangle]
pub unsafe extern "C" fn h2v2_fancy_upsample (cinfo: *mut c_void, compptr: *mut c_void,
		     input_data: JSAMPARRAY, output_data_ptr: *mut JSAMPARRAY)
{
  let output_data = *output_data_ptr;
  let mut inptr0: JSAMPROW;
  let mut inptr1: JSAMPROW;
  let mut outptr: JSAMPROW;
  let mut thiscolsum: c_int;
  let mut lastcolsum: c_int;
  let mut nextcolsum: c_int;
  let mut colctr: JDIMENSION;
  let mut inrow: c_int;
  let mut outrow: c_int;
  let mut v: c_int;

  // PORTING: Requires access to cinfo->max_v_samp_factor, cinfo->output_width,
  // and compptr->downsampled_width
  // GETJSAMPLE macro needs implementation
  // #if BITS_IN_JSAMPLE == 8
  //   // int thiscolsum, lastcolsum, nextcolsum;
  // #else
  //   // INT32 thiscolsum, lastcolsum, nextcolsum;
  // #endif

  // inrow = outrow = 0;
  // while (outrow < cinfo->max_v_samp_factor) {
  //   for (v = 0; v < 2; v += 1) {
  //     /* inptr0 points to nearest input row, inptr1 points to next nearest */
  //     inptr0 = input_data[inrow as usize];
  //     if v == 0 {
  //       /* next nearest is row above */
  //       inptr1 = input_data[(inrow-1) as usize];
  //     } else {
  //       /* next nearest is row below */
  //       inptr1 = input_data[(inrow+1) as usize];
  //     }
  //     outptr = output_data[outrow as usize];
  //     outrow += 1;
  //
  //     /* Special case for first column */
  //     thiscolsum = GETJSAMPLE(*inptr0) as c_int * 3 + GETJSAMPLE(*inptr1) as c_int;
  //     inptr0 = inptr0.add(1);
  //     inptr1 = inptr1.add(1);
  //     nextcolsum = GETJSAMPLE(*inptr0) as c_int * 3 + GETJSAMPLE(*inptr1) as c_int;
  //     inptr0 = inptr0.add(1);
  //     inptr1 = inptr1.add(1);
  //     *outptr = ((thiscolsum * 4 + 8) >> 4) as JSAMPLE;
  //     outptr = outptr.add(1);
  //     *outptr = ((thiscolsum * 3 + nextcolsum + 7) >> 4) as JSAMPLE;
  //     outptr = outptr.add(1);
  //     lastcolsum = thiscolsum; thiscolsum = nextcolsum;
  //
  //     for (colctr = (*compptr).downsampled_width - 2; colctr > 0; colctr -= 1) {
  //       /* General case: 3/4 * nearer pixel + 1/4 * further pixel in each */
  //       /* dimension, thus 9/16, 3/16, 3/16, 1/16 overall */
  //       nextcolsum = GETJSAMPLE(*inptr0) as c_int * 3 + GETJSAMPLE(*inptr1) as c_int;
  //       inptr0 = inptr0.add(1);
  //       inptr1 = inptr1.add(1);
  //       *outptr = ((thiscolsum * 3 + lastcolsum + 8) >> 4) as JSAMPLE;
  //       outptr = outptr.add(1);
  //       *outptr = ((thiscolsum * 3 + nextcolsum + 7) >> 4) as JSAMPLE;
  //       outptr = outptr.add(1);
  //       lastcolsum = thiscolsum; thiscolsum = nextcolsum;
  //     }
  //
  //     /* Special case for last column */
  //     *outptr = ((thiscolsum * 3 + lastcolsum + 8) >> 4) as JSAMPLE;
  //     outptr = outptr.add(1);
  //     *outptr = ((thiscolsum * 4 + 7) >> 4) as JSAMPLE;
  //     outptr = outptr.add(1);
  //   }
  //   inrow += 1;
  // }
}


// /*
//  * Module initialization routine for upsampling.
//  */

#[no_mangle]
pub unsafe extern "C" fn jinit_upsampler (cinfo: *mut c_void)
{
  let mut upsample: my_upsample_ptr;
  let mut ci: c_int;
  let mut compptr: *mut c_void;
  let mut need_buffer: boolean;
  let mut do_fancy: boolean;
  let mut h_in_group: c_int;
  let mut v_in_group: c_int;
  let mut h_out_group: c_int;
  let mut v_out_group: c_int;

  // upsample = (my_upsample_ptr)
  //   (*cinfo->mem->alloc_small) ((j_common_ptr) cinfo, JPOOL_IMAGE,
  //                                SIZEOF(my_upsampler));
  // cinfo->upsample = (struct jpeg_upsampler *) upsample;
  // upsample->pub.start_pass = start_pass_upsample;
  // upsample->pub.upsample = sep_upsample;
  // upsample->pub.need_context_rows = FALSE; /* until we find out differently */
  //
  // if (cinfo->CCIR601_sampling)	/* this isn't supported */
  //   ERREXIT(cinfo, JERR_CCIR601_NOTIMPL);
  //
  // /* jdmainct.c doesn't support context rows when min_DCT_scaled_size = 1,
  //  * so don't ask for it.
  //  */
  // do_fancy = cinfo->do_fancy_upsampling && cinfo->min_DCT_scaled_size > 1;
  //
  // /* Verify we can handle the sampling factors, select per-component methods,
  //  * and create storage as needed.
  //  */
  // for (ci = 0, compptr = cinfo->comp_info; ci < cinfo->num_components;
  //      ci++, compptr++) {
  //   /* Compute size of an "input group" after IDCT scaling.  This many samples
  //    * are to be converted to max_h_samp_factor * max_v_samp_factor pixels.
  //    */
  //   h_in_group = (compptr->h_samp_factor * compptr->DCT_scaled_size) /
  //                cinfo->min_DCT_scaled_size;
  //   v_in_group = (compptr->v_samp_factor * compptr->DCT_scaled_size) /
  //                cinfo->min_DCT_scaled_size;
  //   h_out_group = cinfo->max_h_samp_factor;
  //   v_out_group = cinfo->max_v_samp_factor;
  //   upsample->rowgroup_height[ci] = v_in_group; /* save for use later */
  //   need_buffer = TRUE;
  //   if (! compptr->component_needed) {
  //     /* Don't bother to upsample an uninteresting component. */
  //     upsample->methods[ci] = noop_upsample;
  //     need_buffer = FALSE;
  //   } else if (h_in_group == h_out_group && v_in_group == v_out_group) {
  //     /* Fullsize components can be processed without any work. */
  //     upsample->methods[ci] = fullsize_upsample;
  //     need_buffer = FALSE;
  //   } else if (h_in_group * 2 == h_out_group &&
  //              v_in_group == v_out_group) {
  //     /* Special cases for 2h1v upsampling */
  //     if (do_fancy && compptr->downsampled_width > 2)
  //       upsample->methods[ci] = h2v1_fancy_upsample;
  //     else
  //       upsample->methods[ci] = h2v1_upsample;
  //   } else if (h_in_group * 2 == h_out_group &&
  //              v_in_group * 2 == v_out_group) {
  //     /* Special cases for 2h2v upsampling */
  //     if (do_fancy && compptr->downsampled_width > 2) {
  //       upsample->methods[ci] = h2v2_fancy_upsample;
  //       upsample->pub.need_context_rows = TRUE;
  //     } else
  //       upsample->methods[ci] = h2v2_upsample;
  //   } else if ((h_out_group % h_in_group) == 0 &&
  //              (v_out_group % v_in_group) == 0) {
  //     /* Generic integral-factors upsampling method */
  //     upsample->methods[ci] = int_upsample;
  //     upsample->h_expand[ci] = (UINT8) (h_out_group / h_in_group);
  //     upsample->v_expand[ci] = (UINT8) (v_out_group / v_in_group);
  //   } else
  //     ERREXIT(cinfo, JERR_FRACT_SAMPLE_NOTIMPL);
  //   if (need_buffer) {
  //     upsample->color_buf[ci] = (*cinfo->mem->alloc_sarray)
  //       ((j_common_ptr) cinfo, JPOOL_IMAGE,
  //        (JDIMENSION) jround_up((long) cinfo->output_width,
  //                               (long) cinfo->max_h_samp_factor),
  //        (JDIMENSION) cinfo->max_v_samp_factor);
  //   }
  // }
}
