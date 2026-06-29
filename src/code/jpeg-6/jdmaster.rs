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

// leave this as first line for PCH reasons...
//

use crate::code::jpeg_6::jinclude_h::*;
use crate::code::jpeg_6::jpeglib_h::*;
use core::ffi::{c_int, c_long};
use core::mem;

/* Private state */

#[repr(C)]
struct my_decomp_master {
    pub pub_: jpeg_decomp_master, /* public fields */

    pass_number: c_int,		/* # of passes completed */

    using_merged_upsample: c_int, /* TRUE if using merged upsample/cconvert */

    /* Saved references to initialized quantizer modules,
     * in case we need to switch modes.
     */
    quantizer_1pass: *mut jpeg_color_quantizer,
    quantizer_2pass: *mut jpeg_color_quantizer,
}

type my_master_ptr = *mut my_decomp_master;


/*
 * Determine whether merged upsample/color conversion should be used.
 * CRUCIAL: this must match the actual capabilities of jdmerge.c!
 */

fn use_merged_upsample(cinfo: *mut j_decompress_ptr) -> c_int
{
    #[cfg(feature = "UPSAMPLE_MERGING_SUPPORTED")]
    {
        /* Merging is the equivalent of plain box-filter upsampling */
        if unsafe { (*(*cinfo)).do_fancy_upsampling } != 0 || unsafe { (*(*cinfo)).CCIR601_sampling } != 0 {
            return 0; /* FALSE */
        }
        /* jdmerge.c only supports YCC=>RGB color conversion */
        if unsafe { (*(*cinfo)).jpeg_color_space } != JCS_YCbCr || unsafe { (*(*cinfo)).num_components } != 3 ||
            unsafe { (*(*cinfo)).out_color_space } != JCS_RGB ||
            unsafe { (*(*cinfo)).out_color_components } != RGB_PIXELSIZE as c_int
        {
            return 0; /* FALSE */
        }
        /* and it only handles 2h1v or 2h2v sampling ratios */
        if unsafe { (*(*cinfo)).comp_info[0].h_samp_factor } != 2 ||
            unsafe { (*(*cinfo)).comp_info[1].h_samp_factor } != 1 ||
            unsafe { (*(*cinfo)).comp_info[2].h_samp_factor } != 1 ||
            unsafe { (*(*cinfo)).comp_info[0].v_samp_factor } > 2 ||
            unsafe { (*(*cinfo)).comp_info[1].v_samp_factor } != 1 ||
            unsafe { (*(*cinfo)).comp_info[2].v_samp_factor } != 1
        {
            return 0; /* FALSE */
        }
        /* furthermore, it doesn't work if we've scaled the IDCTs differently */
        if unsafe { (*(*cinfo)).comp_info[0].DCT_scaled_size } != unsafe { (*(*cinfo)).min_DCT_scaled_size } ||
            unsafe { (*(*cinfo)).comp_info[1].DCT_scaled_size } != unsafe { (*(*cinfo)).min_DCT_scaled_size } ||
            unsafe { (*(*cinfo)).comp_info[2].DCT_scaled_size } != unsafe { (*(*cinfo)).min_DCT_scaled_size }
        {
            return 0; /* FALSE */
        }
        /* ??? also need to test for upsample-time rescaling, when & if supported */
        return 1; /* TRUE - by golly, it'll work... */
    }
    #[cfg(not(feature = "UPSAMPLE_MERGING_SUPPORTED"))]
    {
        return 0; /* FALSE */
    }
}


/*
 * Compute output image dimensions and related values.
 * NOTE: this is exported for possible use by application.
 * Hence it mustn't do anything that can't be done twice.
 * Also note that it may be called before the master module is initialized!
 */

pub fn jpeg_calc_output_dimensions(cinfo: *mut j_decompress_ptr)
/* Do computations that are needed before master selection phase */
{
    /* Prevent application from calling me at wrong times */
    if unsafe { (*(*cinfo)).global_state } != DSTATE_READY as c_int {
        ERREXIT1(cinfo, JERR_BAD_STATE, unsafe { (*(*cinfo)).global_state });
    }

    #[cfg(feature = "IDCT_SCALING_SUPPORTED")]
    {
        let mut ci: c_int;
        let mut compptr: *mut jpeg_component_info;

        /* Compute actual output image dimensions and DCT scaling choices. */
        if unsafe { (*(*cinfo)).scale_num } * 8 <= unsafe { (*(*cinfo)).scale_denom } {
            /* Provide 1/8 scaling */
            unsafe {
                (*(*cinfo)).output_width = jdiv_round_up((*(*cinfo)).image_width as c_long, 8) as JDIMENSION;
                (*(*cinfo)).output_height = jdiv_round_up((*(*cinfo)).image_height as c_long, 8) as JDIMENSION;
                (*(*cinfo)).min_DCT_scaled_size = 1;
            }
        } else if unsafe { (*(*cinfo)).scale_num } * 4 <= unsafe { (*(*cinfo)).scale_denom } {
            /* Provide 1/4 scaling */
            unsafe {
                (*(*cinfo)).output_width = jdiv_round_up((*(*cinfo)).image_width as c_long, 4) as JDIMENSION;
                (*(*cinfo)).output_height = jdiv_round_up((*(*cinfo)).image_height as c_long, 4) as JDIMENSION;
                (*(*cinfo)).min_DCT_scaled_size = 2;
            }
        } else if unsafe { (*(*cinfo)).scale_num } * 2 <= unsafe { (*(*cinfo)).scale_denom } {
            /* Provide 1/2 scaling */
            unsafe {
                (*(*cinfo)).output_width = jdiv_round_up((*(*cinfo)).image_width as c_long, 2) as JDIMENSION;
                (*(*cinfo)).output_height = jdiv_round_up((*(*cinfo)).image_height as c_long, 2) as JDIMENSION;
                (*(*cinfo)).min_DCT_scaled_size = 4;
            }
        } else {
            /* Provide 1/1 scaling */
            unsafe {
                (*(*cinfo)).output_width = (*(*cinfo)).image_width;
                (*(*cinfo)).output_height = (*(*cinfo)).image_height;
                (*(*cinfo)).min_DCT_scaled_size = DCTSIZE as c_int;
            }
        }
        /* In selecting the actual DCT scaling for each component, we try to
         * scale up the chroma components via IDCT scaling rather than upsampling.
         * This saves time if the upsampler gets to use 1:1 scaling.
         * Note this code assumes that the supported DCT scalings are powers of 2.
         */
        ci = 0;
        compptr = unsafe { (*(*cinfo)).comp_info.as_mut_ptr() };
        while ci < unsafe { (*(*cinfo)).num_components } {
            let mut ssize: c_int = unsafe { (*(*cinfo)).min_DCT_scaled_size };
            while ssize < DCTSIZE as c_int &&
                   unsafe { ((*compptr).h_samp_factor as c_int * ssize * 2 <=
                    (*(*cinfo)).max_h_samp_factor as c_int * unsafe { (*(*cinfo)).min_DCT_scaled_size }) } &&
                   unsafe { ((*compptr).v_samp_factor as c_int * ssize * 2 <=
                    (*(*cinfo)).max_v_samp_factor as c_int * unsafe { (*(*cinfo)).min_DCT_scaled_size }) }
            {
                ssize = ssize * 2;
            }
            unsafe { (*compptr).DCT_scaled_size = ssize as c_int; }
            ci += 1;
            compptr = unsafe { compptr.add(1) };
        }

        /* Recompute downsampled dimensions of components;
         * application needs to know these if using raw downsampled data.
         */
        ci = 0;
        compptr = unsafe { (*(*cinfo)).comp_info.as_mut_ptr() };
        while ci < unsafe { (*(*cinfo)).num_components } {
            /* Size in samples, after IDCT scaling */
            unsafe {
                (*compptr).downsampled_width = jdiv_round_up(
                    (*(*cinfo)).image_width as c_long *
                    ((*compptr).h_samp_factor as c_long * (*compptr).DCT_scaled_size as c_long),
                    ((*(*cinfo)).max_h_samp_factor as c_long * DCTSIZE as c_long)
                ) as JDIMENSION;
                (*compptr).downsampled_height = jdiv_round_up(
                    (*(*cinfo)).image_height as c_long *
                    ((*compptr).v_samp_factor as c_long * (*compptr).DCT_scaled_size as c_long),
                    ((*(*cinfo)).max_v_samp_factor as c_long * DCTSIZE as c_long)
                ) as JDIMENSION;
            }
            ci += 1;
            compptr = unsafe { compptr.add(1) };
        }
    }

    #[cfg(not(feature = "IDCT_SCALING_SUPPORTED"))]
    {
        /* Hardwire it to "no scaling" */
        unsafe {
            (*(*cinfo)).output_width = (*(*cinfo)).image_width;
            (*(*cinfo)).output_height = (*(*cinfo)).image_height;
        }
        /* jdinput.c has already initialized DCT_scaled_size to DCTSIZE,
         * and has computed unscaled downsampled_width and downsampled_height.
         */
    }

    /* Report number of components in selected colorspace. */
    /* Probably this should be in the color conversion module... */
    match unsafe { (*(*cinfo)).out_color_space } as c_int {
        JCS_GRAYSCALE => {
            unsafe { (*(*cinfo)).out_color_components = 1; }
        },
        JCS_RGB => {
            #[cfg(not(feature = "RGB_PIXELSIZE_3"))]
            {
                unsafe { (*(*cinfo)).out_color_components = RGB_PIXELSIZE as c_int; }
            }
            /* else share code with YCbCr */
        },
        JCS_YCbCr => {
            unsafe { (*(*cinfo)).out_color_components = 3; }
        },
        JCS_CMYK | JCS_YCCK => {
            unsafe { (*(*cinfo)).out_color_components = 4; }
        },
        _ => {
            /* else must be same colorspace as in file */
            unsafe { (*(*cinfo)).out_color_components = (*(*cinfo)).num_components; }
        }
    }
    unsafe {
        (*(*cinfo)).output_components = if (*(*cinfo)).quantize_colors != 0 { 1 }
                                        else { (*(*cinfo)).out_color_components };
    }

    /* See if upsampler will want to emit more than one row at a time */
    if use_merged_upsample(cinfo) != 0 {
        unsafe { (*(*cinfo)).rec_outbuf_height = (*(*cinfo)).max_v_samp_factor as JDIMENSION; }
    } else {
        unsafe { (*(*cinfo)).rec_outbuf_height = 1; }
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

fn prepare_range_limit_table(cinfo: *mut j_decompress_ptr)
/* Allocate and fill in the sample_range_limit table */
{
    let mut table: *mut JSAMPLE;
    let mut i: c_int;

    unsafe {
        table = (*(*cinfo).mem).alloc_small.unwrap()(
            cinfo as *mut c_void,
            JPOOL_IMAGE as c_int,
            ((5 * (MAXJSAMPLE as c_int + 1) + CENTERJSAMPLE as c_int) * mem::size_of::<JSAMPLE>() as c_int) as usize,
        ) as *mut JSAMPLE;
        table = table.add((MAXJSAMPLE as usize) + 1); /* allow negative subscripts of simple table */
        (*(*cinfo)).sample_range_limit = table;
        /* First segment of "simple" table: limit[x] = 0 for x < 0 */
        MEMZERO(
            table.sub((MAXJSAMPLE as usize) + 1),
            ((MAXJSAMPLE as c_int + 1) * mem::size_of::<JSAMPLE>() as c_int) as usize
        );
        /* Main part of "simple" table: limit[x] = x */
        i = 0;
        while i <= MAXJSAMPLE as c_int {
            *table.add(i as usize) = i as JSAMPLE;
            i += 1;
        }
        table = table.add(CENTERJSAMPLE as usize); /* Point to where post-IDCT table starts */
        /* End of simple table, rest of first half of post-IDCT table */
        i = CENTERJSAMPLE as c_int;
        while i < 2 * (MAXJSAMPLE as c_int + 1) {
            *table.add(i as usize) = MAXJSAMPLE as JSAMPLE;
            i += 1;
        }
        /* Second half of post-IDCT table */
        MEMZERO(
            table.add(2 * (MAXJSAMPLE as usize + 1)),
            (2 * (MAXJSAMPLE as c_int + 1) - CENTERJSAMPLE as c_int) as usize * mem::size_of::<JSAMPLE>()
        );
        MEMCOPY(
            table.add((4 * (MAXJSAMPLE as usize + 1)) - CENTERJSAMPLE as usize),
            (*(*cinfo)).sample_range_limit,
            CENTERJSAMPLE as usize * mem::size_of::<JSAMPLE>()
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

fn master_selection(cinfo: *mut j_decompress_ptr)
{
    let master: my_master_ptr = unsafe { (*(*cinfo)).master as my_master_ptr };
    let mut use_c_buffer: c_int;
    let mut samplesperrow: c_long;
    let mut jd_samplesperrow: JDIMENSION;

    /* Initialize dimensions and other stuff */
    jpeg_calc_output_dimensions(cinfo);
    prepare_range_limit_table(cinfo);

    /* Width of an output scanline must be representable as JDIMENSION. */
    unsafe {
        samplesperrow = (*(*cinfo)).output_width as c_long * (*(*cinfo)).out_color_components as c_long;
        jd_samplesperrow = samplesperrow as JDIMENSION;
        if jd_samplesperrow as c_long != samplesperrow {
            ERREXIT(cinfo, JERR_WIDTH_OVERFLOW);
        }
    }

    /* Initialize my private state */
    unsafe {
        (*master).pass_number = 0;
        (*master).using_merged_upsample = use_merged_upsample(cinfo);
    }

    /* Color quantizer selection */
    unsafe {
        (*master).quantizer_1pass = core::ptr::null_mut();
        (*master).quantizer_2pass = core::ptr::null_mut();
        /* No mode changes if not using buffered-image mode. */
        if (*(*cinfo)).quantize_colors == 0 || (*(*cinfo)).buffered_image == 0 {
            (*(*cinfo)).enable_1pass_quant = 0;
            (*(*cinfo)).enable_external_quant = 0;
            (*(*cinfo)).enable_2pass_quant = 0;
        }
        if (*(*cinfo)).quantize_colors != 0 {
            if (*(*cinfo)).raw_data_out != 0 {
                ERREXIT(cinfo, JERR_NOTIMPL);
            }
            /* 2-pass quantizer only works in 3-component color space. */
            if (*(*cinfo)).out_color_components != 3 {
                (*(*cinfo)).enable_1pass_quant = 1;
                (*(*cinfo)).enable_external_quant = 0;
                (*(*cinfo)).enable_2pass_quant = 0;
                (*(*cinfo)).colormap = core::ptr::null_mut();
            } else if (*(*cinfo)).colormap != core::ptr::null_mut() {
                (*(*cinfo)).enable_external_quant = 1;
            } else if (*(*cinfo)).two_pass_quantize != 0 {
                (*(*cinfo)).enable_2pass_quant = 1;
            } else {
                (*(*cinfo)).enable_1pass_quant = 1;
            }

            if (*(*cinfo)).enable_1pass_quant != 0 {
                #[cfg(feature = "QUANT_1PASS_SUPPORTED")]
                {
                    jinit_1pass_quantizer(cinfo);
                    (*master).quantizer_1pass = (*(*cinfo)).cquantize;
                }
                #[cfg(not(feature = "QUANT_1PASS_SUPPORTED"))]
                {
                    ERREXIT(cinfo, JERR_NOT_COMPILED);
                }
            }

            /* We use the 2-pass code to map to external colormaps. */
            if (*(*cinfo)).enable_2pass_quant != 0 || (*(*cinfo)).enable_external_quant != 0 {
                #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
                {
                    jinit_2pass_quantizer(cinfo);
                    (*master).quantizer_2pass = (*(*cinfo)).cquantize;
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
        if (*(*cinfo)).raw_data_out == 0 {
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
            jinit_d_post_controller(cinfo, (*(*cinfo)).enable_2pass_quant);
        }
        /* Inverse DCT */
        jinit_inverse_dct(cinfo);
        /* Entropy decoding: either Huffman or arithmetic coding. */
        if (*(*cinfo)).arith_code != 0 {
            ERREXIT(cinfo, JERR_ARITH_NOTIMPL);
        } else {
            if (*(*cinfo)).progressive_mode != 0 {
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
        use_c_buffer = if (*(*cinfo)).inputctl.is_null() { 0 } else {
            let inputctl = &*(*(*cinfo)).inputctl;
            if inputctl.has_multiple_scans != 0 || (*(*cinfo)).buffered_image != 0 { 1 } else { 0 }
        };
        jinit_d_coef_controller(cinfo, use_c_buffer);

        if (*(*cinfo)).raw_data_out == 0 {
            jinit_d_main_controller(cinfo, 0); /* FALSE - never need full buffer here */
        }

        /* We can now tell the memory manager to allocate virtual arrays. */
        (*(*cinfo).mem).realize_virt_arrays.unwrap()(cinfo as *mut c_void);

        /* Initialize input side of decompressor to consume first scan. */
        (*(*(*cinfo)).inputctl).start_input_pass.unwrap()(cinfo);
    }

    #[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
    {
        /* If jpeg_start_decompress will read the whole file, initialize
         * progress monitoring appropriately.  The input step is counted
         * as one pass.
         */
        unsafe {
            if (*(*cinfo)).progress != core::ptr::null_mut() && (*(*cinfo)).buffered_image == 0 &&
                (*(*(*cinfo)).inputctl).has_multiple_scans != 0 {
                let mut nscans: c_int;
                /* Estimate number of scans to set pass_limit. */
                if (*(*cinfo)).progressive_mode != 0 {
                    /* Arbitrarily estimate 2 interleaved DC scans + 3 AC scans/component. */
                    nscans = 2 + 3 * (*(*cinfo)).num_components;
                } else {
                    /* For a nonprogressive multiscan file, estimate 1 scan per component. */
                    nscans = (*(*cinfo)).num_components;
                }
                (*(*cinfo)).progress.as_mut().unwrap().pass_counter = 0;
                (*(*cinfo)).progress.as_mut().unwrap().pass_limit = (*(*cinfo)).total_iMCU_rows as c_long * nscans as c_long;
                (*(*cinfo)).progress.as_mut().unwrap().completed_passes = 0;
                (*(*cinfo)).progress.as_mut().unwrap().total_passes = if (*(*cinfo)).enable_2pass_quant != 0 { 3 } else { 2 };
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

pub fn prepare_for_output_pass(cinfo: *mut j_decompress_ptr)
{
    let master: my_master_ptr = unsafe { (*(*cinfo)).master as my_master_ptr };

    unsafe {
        if (*master).pub_.is_dummy_pass != 0 {
            #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
            {
                /* Final pass of 2-pass quantization */
                (*master).pub_.is_dummy_pass = 0;
                (*(*(*cinfo)).cquantize).start_pass.unwrap()(cinfo, 0);
                (*(*(*cinfo)).post).start_pass.unwrap()(cinfo, JBUF_CRANK_DEST as c_int);
                (*(*(*cinfo)).main).start_pass.unwrap()(cinfo, JBUF_CRANK_DEST as c_int);
            }
            #[cfg(not(feature = "QUANT_2PASS_SUPPORTED"))]
            {
                ERREXIT(cinfo, JERR_NOT_COMPILED);
            }
        } else {
            if (*(*cinfo)).quantize_colors != 0 && (*(*cinfo)).colormap == core::ptr::null_mut() {
                /* Select new quantization method */
                if (*(*cinfo)).two_pass_quantize != 0 && (*(*cinfo)).enable_2pass_quant != 0 {
                    (*(*cinfo)).cquantize = (*master).quantizer_2pass;
                    (*master).pub_.is_dummy_pass = 1;
                } else if (*(*cinfo)).enable_1pass_quant != 0 {
                    (*(*cinfo)).cquantize = (*master).quantizer_1pass;
                } else {
                    ERREXIT(cinfo, JERR_MODE_CHANGE);
                }
            }
            (*(*(*cinfo)).idct).start_pass.unwrap()(cinfo);
            (*(*(*cinfo)).coef).start_output_pass.unwrap()(cinfo);
            if (*(*cinfo)).raw_data_out == 0 {
                if (*master).using_merged_upsample == 0 {
                    (*(*(*cinfo)).cconvert).start_pass.unwrap()(cinfo);
                }
                (*(*(*cinfo)).upsample).start_pass.unwrap()(cinfo);
                if (*(*cinfo)).quantize_colors != 0 {
                    (*(*(*cinfo)).cquantize).start_pass.unwrap()(cinfo, (*master).pub_.is_dummy_pass);
                }
                (*(*(*cinfo)).post).start_pass.unwrap()(cinfo,
                    if (*master).pub_.is_dummy_pass != 0 { JBUF_SAVE_AND_PASS as c_int } else { JBUF_PASS_THRU as c_int });
                (*(*(*cinfo)).main).start_pass.unwrap()(cinfo, JBUF_PASS_THRU as c_int);
            }
        }

        /* Set up progress monitor's pass info if present */
        if (*(*cinfo)).progress != core::ptr::null_mut() {
            (*(*cinfo)).progress.as_mut().unwrap().completed_passes = (*master).pass_number;
            (*(*cinfo)).progress.as_mut().unwrap().total_passes = (*master).pass_number +
                                if (*master).pub_.is_dummy_pass != 0 { 2 } else { 1 };
            /* In buffered-image mode, we assume one more output pass if EOI not
             * yet reached, but no more passes if EOI has been reached.
             */
            if (*(*cinfo)).buffered_image != 0 && (*(*(*cinfo)).inputctl).eoi_reached == 0 {
                (*(*cinfo)).progress.as_mut().unwrap().total_passes += if (*(*cinfo)).enable_2pass_quant != 0 { 2 } else { 1 };
            }
        }
    }
}


/*
 * Finish up at end of an output pass.
 */

pub fn finish_output_pass(cinfo: *mut j_decompress_ptr)
{
    let master: my_master_ptr = unsafe { (*(*cinfo)).master as my_master_ptr };

    unsafe {
        if (*(*cinfo)).quantize_colors != 0 {
            (*(*(*cinfo)).cquantize).finish_pass.unwrap()(cinfo);
        }
        (*master).pass_number += 1;
    }
}


#[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
/*
 * Switch to a new external colormap between output passes.
 */

pub fn jpeg_new_colormap(cinfo: *mut j_decompress_ptr)
{
    let master: my_master_ptr = unsafe { (*(*cinfo)).master as my_master_ptr };

    unsafe {
        /* Prevent application from calling me at wrong times */
        if (*(*cinfo)).global_state != DSTATE_BUFIMAGE as c_int {
            ERREXIT1(cinfo, JERR_BAD_STATE, (*(*cinfo)).global_state);
        }

        if (*(*cinfo)).quantize_colors != 0 && (*(*cinfo)).enable_external_quant != 0 &&
            (*(*cinfo)).colormap != core::ptr::null_mut() {
            /* Select 2-pass quantizer for external colormap use */
            (*(*cinfo)).cquantize = (*master).quantizer_2pass;
            /* Notify quantizer of colormap change */
            (*(*(*cinfo)).cquantize).new_color_map.unwrap()(cinfo);
            (*master).pub_.is_dummy_pass = 0; /* just in case */
        } else {
            ERREXIT(cinfo, JERR_MODE_CHANGE);
        }
    }
}


/*
 * Initialize master decompression control and select active modules.
 * This is performed at the start of jpeg_start_decompress.
 */

pub fn jinit_master_decompress(cinfo: *mut j_decompress_ptr)
{
    let master: my_master_ptr;

    unsafe {
        master = (*(*cinfo).mem).alloc_small.unwrap()(
            cinfo as *mut c_void,
            JPOOL_IMAGE as c_int,
            mem::size_of::<my_decomp_master>() as usize,
        ) as my_master_ptr;
        (*(*cinfo)).master = master as *mut jpeg_decomp_master;
        (*master).pub_.prepare_for_output_pass = Some(prepare_for_output_pass);
        (*master).pub_.finish_output_pass = Some(finish_output_pass);

        (*master).pub_.is_dummy_pass = 0;

        master_selection(cinfo);
    }
}


/* Stubs for external functions called by this module */

extern "C" {
    fn jdiv_round_up(a: c_long, b: c_long) -> c_long;
    fn MEMZERO(addr: *mut core::ffi::c_void, size: usize);
    fn MEMCOPY(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, size: usize);
    fn ERREXIT(cinfo: *mut j_decompress_ptr, code: c_int);
    fn ERREXIT1(cinfo: *mut j_decompress_ptr, code: c_int, arg: c_int);
    fn jinit_1pass_quantizer(cinfo: *mut j_decompress_ptr);
    fn jinit_2pass_quantizer(cinfo: *mut j_decompress_ptr);
    fn jinit_merged_upsampler(cinfo: *mut j_decompress_ptr);
    fn jinit_color_deconverter(cinfo: *mut j_decompress_ptr);
    fn jinit_upsampler(cinfo: *mut j_decompress_ptr);
    fn jinit_d_post_controller(cinfo: *mut j_decompress_ptr, need_full_buffer: c_int);
    fn jinit_inverse_dct(cinfo: *mut j_decompress_ptr);
    fn jinit_phuff_decoder(cinfo: *mut j_decompress_ptr);
    fn jinit_huff_decoder(cinfo: *mut j_decompress_ptr);
    fn jinit_d_coef_controller(cinfo: *mut j_decompress_ptr, need_full_buffer: c_int);
    fn jinit_d_main_controller(cinfo: *mut j_decompress_ptr, need_full_buffer: c_int);
}
