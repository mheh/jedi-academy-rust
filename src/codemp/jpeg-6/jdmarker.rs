/*
 * jdmarker.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains routines to decode JPEG datastream markers.
 * Most of the complexity arises from our desire to support input
 * suspension: if not all of the data for a marker is available,
 * we must exit back to the application.  On resumption, we reprocess
 * the marker.
 */

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals,
         unused_mut, unused_variables, dead_code, unused_unsafe,
         clippy::needless_return)]

use core::ffi::{c_int, c_uint, c_void};

// Porting note: exe_headers.h is a game-engine include; skipped (no Rust equivalent).
// Porting note: jinclude.h defines MEMCOPY/SIZEOF/GETJOCTET; jpeglib.h defines all
//   JPEG types; with JPEG_INTERNALS defined, jpeglib.h also includes jpegint.h and
//   jerror.h. All four are imported via glob below.
use crate::codemp::jpeg_6::jinclude_h::*;
use crate::codemp::jpeg_6::jmorecfg_h::*;
use crate::codemp::jpeg_6::jpeglib_h::*;
use crate::codemp::jpeg_6::jpegint_h::*;
// Porting note: jerror_h covers JERR_* and JTRC_* codes; trusted to exist.
use crate::codemp::jpeg_6::jerror_h::*;

// Porting deviation: INT32 is defined in C as `typedef long INT32` (in jmorecfg.h),
// but jmorecfg_h.rs does not export it (the typedef is commented out there).
// Defined locally as i32, matching the project-wide convention.
type INT32 = i32;

// Porting deviation: jmorecfg_h exports FALSE/TRUE as i32.  Shadow them here as
// `boolean`-typed constants so `return FALSE`/`return TRUE` match the boolean (u8)
// return type without per-site casts.
const FALSE: boolean = 0;
const TRUE: boolean = 1;


/* JPEG marker codes.
 * Porting deviation: the C `typedef enum { ... } JPEG_MARKER` is translated as a
 * plain c_int type alias + const values, because the enum values are used as c_int
 * match arms against cinfo->unread_marker (also c_int).
 */
#[allow(dead_code)]
type JPEG_MARKER = c_int;

const M_SOF0:  JPEG_MARKER = 0xc0;
const M_SOF1:  JPEG_MARKER = 0xc1;
const M_SOF2:  JPEG_MARKER = 0xc2;
const M_SOF3:  JPEG_MARKER = 0xc3;

const M_SOF5:  JPEG_MARKER = 0xc5;
const M_SOF6:  JPEG_MARKER = 0xc6;
const M_SOF7:  JPEG_MARKER = 0xc7;

const M_JPG:   JPEG_MARKER = 0xc8;
const M_SOF9:  JPEG_MARKER = 0xc9;
const M_SOF10: JPEG_MARKER = 0xca;
const M_SOF11: JPEG_MARKER = 0xcb;

const M_SOF13: JPEG_MARKER = 0xcd;
const M_SOF14: JPEG_MARKER = 0xce;
const M_SOF15: JPEG_MARKER = 0xcf;

const M_DHT:   JPEG_MARKER = 0xc4;

const M_DAC:   JPEG_MARKER = 0xcc;

const M_RST0:  JPEG_MARKER = 0xd0;
const M_RST1:  JPEG_MARKER = 0xd1;
const M_RST2:  JPEG_MARKER = 0xd2;
const M_RST3:  JPEG_MARKER = 0xd3;
const M_RST4:  JPEG_MARKER = 0xd4;
const M_RST5:  JPEG_MARKER = 0xd5;
const M_RST6:  JPEG_MARKER = 0xd6;
const M_RST7:  JPEG_MARKER = 0xd7;

const M_SOI:   JPEG_MARKER = 0xd8;
const M_EOI:   JPEG_MARKER = 0xd9;
const M_SOS:   JPEG_MARKER = 0xda;
const M_DQT:   JPEG_MARKER = 0xdb;
const M_DNL:   JPEG_MARKER = 0xdc;
const M_DRI:   JPEG_MARKER = 0xdd;
const M_DHP:   JPEG_MARKER = 0xde;
const M_EXP:   JPEG_MARKER = 0xdf;

const M_APP0:  JPEG_MARKER = 0xe0;
const M_APP1:  JPEG_MARKER = 0xe1;
const M_APP2:  JPEG_MARKER = 0xe2;
const M_APP3:  JPEG_MARKER = 0xe3;
const M_APP4:  JPEG_MARKER = 0xe4;
const M_APP5:  JPEG_MARKER = 0xe5;
const M_APP6:  JPEG_MARKER = 0xe6;
const M_APP7:  JPEG_MARKER = 0xe7;
const M_APP8:  JPEG_MARKER = 0xe8;
const M_APP9:  JPEG_MARKER = 0xe9;
const M_APP10: JPEG_MARKER = 0xea;
const M_APP11: JPEG_MARKER = 0xeb;
const M_APP12: JPEG_MARKER = 0xec;
const M_APP13: JPEG_MARKER = 0xed;
const M_APP14: JPEG_MARKER = 0xee;
const M_APP15: JPEG_MARKER = 0xef;

const M_JPG0:  JPEG_MARKER = 0xf0;
const M_JPG13: JPEG_MARKER = 0xfd;
const M_COM:   JPEG_MARKER = 0xfe;

const M_TEM:   JPEG_MARKER = 0x01;

const M_ERROR: JPEG_MARKER = 0x100;


/*
 * Macros for fetching data from the data source module.
 *
 * At all times, cinfo->src->next_input_byte and ->bytes_in_buffer reflect
 * the current restart point; we update them only when we have reached a
 * suitable place to restart if a suspension occurs.
 */

/* Declare and initialize local copies of input pointer/count */
macro_rules! INPUT_VARS {
    ($cinfo:expr) => {
        let datasrc: *mut jpeg_source_mgr = unsafe { (*$cinfo).src };
        let mut next_input_byte: *const JOCTET = unsafe { (*datasrc).next_input_byte };
        let mut bytes_in_buffer: usize = unsafe { (*datasrc).bytes_in_buffer };
    };
}

/* Unload the local copies --- do this only at a restart boundary */
macro_rules! INPUT_SYNC {
    ($cinfo:expr) => {
        unsafe {
            (*datasrc).next_input_byte = next_input_byte;
            (*datasrc).bytes_in_buffer = bytes_in_buffer;
        }
    };
}

/* Reload the local copies --- seldom used except in MAKE_BYTE_AVAIL */
macro_rules! INPUT_RELOAD {
    ($cinfo:expr) => {
        unsafe {
            next_input_byte = (*datasrc).next_input_byte;
            bytes_in_buffer = (*datasrc).bytes_in_buffer;
        }
    };
}

/* Internal macro for INPUT_BYTE and INPUT_2BYTES: make a byte available.
 * Note we do *not* do INPUT_SYNC before calling fill_input_buffer,
 * but we must reload the local copies after a successful fill.
 */
macro_rules! MAKE_BYTE_AVAIL {
    ($cinfo:expr, $action:expr) => {
        if bytes_in_buffer == 0 {
            if unsafe { ((*datasrc).fill_input_buffer.unwrap())($cinfo) } == 0 {
                $action;
            }
            INPUT_RELOAD!($cinfo);
        }
        bytes_in_buffer -= 1;
    };
}

/* Read a byte into variable V.
 * If must suspend, take the specified action (typically "return FALSE").
 */
macro_rules! INPUT_BYTE {
    ($cinfo:expr, $V:expr, $action:expr) => {
        MAKE_BYTE_AVAIL!($cinfo, $action);
        $V = unsafe { GETJOCTET(*next_input_byte) } as _;
        next_input_byte = unsafe { next_input_byte.add(1) };
    };
}

/* As above, but read two bytes interpreted as an unsigned 16-bit integer.
 * V should be declared unsigned int or perhaps INT32.
 */
macro_rules! INPUT_2BYTES {
    ($cinfo:expr, $V:expr, $action:expr) => {
        MAKE_BYTE_AVAIL!($cinfo, $action);
        $V = ((unsafe { GETJOCTET(*next_input_byte) } as c_uint) << 8) as _;
        next_input_byte = unsafe { next_input_byte.add(1) };
        MAKE_BYTE_AVAIL!($cinfo, $action);
        $V = ($V as c_uint + unsafe { GETJOCTET(*next_input_byte) } as c_uint) as _;
        next_input_byte = unsafe { next_input_byte.add(1) };
    };
}


/*
 * Routines to process JPEG markers.
 *
 * Entry condition: JPEG marker itself has been read and its code saved
 *   in cinfo->unread_marker; input restart point is just after the marker.
 *
 * Exit: if return TRUE, have read and processed any parameters, and have
 *   updated the restart point to point after the parameters.
 *   If return FALSE, was forced to suspend before reaching end of
 *   marker parameters; restart point has not been moved.  Same routine
 *   will be called again after application supplies more input data.
 *
 * This approach to suspension assumes that all of a marker's parameters can
 * fit into a single input bufferload.  This should hold for "normal"
 * markers.  Some COM/APPn markers might have large parameter segments,
 * but we use skip_input_data to get past those, and thereby put the problem
 * on the source manager's shoulders.
 *
 * Note that we don't bother to avoid duplicate trace messages if a
 * suspension occurs within marker parameters.  Other side effects
 * require more care.
 */


/* Porting note: TRACEMS/WARNMS/ERREXIT are C macros from jerror.h (included via
 * jpeglib.h when JPEG_INTERNALS is defined).  They are translated here as
 * macro_rules! expanding to the same field assignments and function-pointer calls
 * that the C macros expand to.  All expansions stay within unsafe fn bodies.
 * j_common_ptr = *mut jpeg_common_struct; cinfo is j_decompress_ptr = *mut jpeg_decompress_struct;
 * casting via `as j_common_ptr` reinterprets the pointer (allowed for raw ptrs in Rust).
 */

macro_rules! TRACEMS {
    ($cinfo:expr, $lvl:expr, $code:expr) => {
        unsafe {
            (*(*$cinfo).err).msg_code = $code as c_int;
            ((*(*$cinfo).err).emit_message.unwrap())($cinfo as j_common_ptr, $lvl as c_int);
        }
    };
}

macro_rules! TRACEMS1 {
    ($cinfo:expr, $lvl:expr, $code:expr, $p1:expr) => {
        unsafe {
            (*(*$cinfo).err).msg_code = $code as c_int;
            (*(*$cinfo).err).msg_parm.i[0] = $p1 as c_int;
            ((*(*$cinfo).err).emit_message.unwrap())($cinfo as j_common_ptr, $lvl as c_int);
        }
    };
}

macro_rules! TRACEMS2 {
    ($cinfo:expr, $lvl:expr, $code:expr, $p1:expr, $p2:expr) => {
        unsafe {
            (*(*$cinfo).err).msg_code = $code as c_int;
            (*(*$cinfo).err).msg_parm.i[0] = $p1 as c_int;
            (*(*$cinfo).err).msg_parm.i[1] = $p2 as c_int;
            ((*(*$cinfo).err).emit_message.unwrap())($cinfo as j_common_ptr, $lvl as c_int);
        }
    };
}

macro_rules! TRACEMS3 {
    ($cinfo:expr, $lvl:expr, $code:expr, $p1:expr, $p2:expr, $p3:expr) => {
        unsafe {
            let _mp: *mut c_int = (*(*$cinfo).err).msg_parm.i.as_mut_ptr();
            *_mp.add(0) = $p1 as c_int;
            *_mp.add(1) = $p2 as c_int;
            *_mp.add(2) = $p3 as c_int;
            (*(*$cinfo).err).msg_code = $code as c_int;
            ((*(*$cinfo).err).emit_message.unwrap())($cinfo as j_common_ptr, $lvl as c_int);
        }
    };
}

macro_rules! TRACEMS4 {
    ($cinfo:expr, $lvl:expr, $code:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr) => {
        unsafe {
            let _mp: *mut c_int = (*(*$cinfo).err).msg_parm.i.as_mut_ptr();
            *_mp.add(0) = $p1 as c_int;
            *_mp.add(1) = $p2 as c_int;
            *_mp.add(2) = $p3 as c_int;
            *_mp.add(3) = $p4 as c_int;
            (*(*$cinfo).err).msg_code = $code as c_int;
            ((*(*$cinfo).err).emit_message.unwrap())($cinfo as j_common_ptr, $lvl as c_int);
        }
    };
}

macro_rules! TRACEMS8 {
    ($cinfo:expr, $lvl:expr, $code:expr,
     $p1:expr, $p2:expr, $p3:expr, $p4:expr,
     $p5:expr, $p6:expr, $p7:expr, $p8:expr) => {
        unsafe {
            let _mp: *mut c_int = (*(*$cinfo).err).msg_parm.i.as_mut_ptr();
            *_mp.add(0) = $p1 as c_int;
            *_mp.add(1) = $p2 as c_int;
            *_mp.add(2) = $p3 as c_int;
            *_mp.add(3) = $p4 as c_int;
            *_mp.add(4) = $p5 as c_int;
            *_mp.add(5) = $p6 as c_int;
            *_mp.add(6) = $p7 as c_int;
            *_mp.add(7) = $p8 as c_int;
            (*(*$cinfo).err).msg_code = $code as c_int;
            ((*(*$cinfo).err).emit_message.unwrap())($cinfo as j_common_ptr, $lvl as c_int);
        }
    };
}

macro_rules! WARNMS2 {
    ($cinfo:expr, $code:expr, $p1:expr, $p2:expr) => {
        unsafe {
            (*(*$cinfo).err).msg_code = $code as c_int;
            (*(*$cinfo).err).msg_parm.i[0] = $p1 as c_int;
            (*(*$cinfo).err).msg_parm.i[1] = $p2 as c_int;
            ((*(*$cinfo).err).emit_message.unwrap())($cinfo as j_common_ptr, -1);
        }
    };
}

macro_rules! ERREXIT {
    ($cinfo:expr, $code:expr) => {
        unsafe {
            (*(*$cinfo).err).msg_code = $code as c_int;
            ((*(*$cinfo).err).error_exit.unwrap())($cinfo as j_common_ptr);
        }
    };
}

macro_rules! ERREXIT1 {
    ($cinfo:expr, $code:expr, $p1:expr) => {
        unsafe {
            (*(*$cinfo).err).msg_code = $code as c_int;
            (*(*$cinfo).err).msg_parm.i[0] = $p1 as c_int;
            ((*(*$cinfo).err).error_exit.unwrap())($cinfo as j_common_ptr);
        }
    };
}

macro_rules! ERREXIT2 {
    ($cinfo:expr, $code:expr, $p1:expr, $p2:expr) => {
        unsafe {
            (*(*$cinfo).err).msg_code = $code as c_int;
            (*(*$cinfo).err).msg_parm.i[0] = $p1 as c_int;
            (*(*$cinfo).err).msg_parm.i[1] = $p2 as c_int;
            ((*(*$cinfo).err).error_exit.unwrap())($cinfo as j_common_ptr);
        }
    };
}


unsafe fn get_soi(cinfo: j_decompress_ptr) -> boolean
/* Process an SOI marker */
{
  let mut i: c_int;

  TRACEMS!(cinfo, 1, JTRC_SOI);

  if (*(*cinfo).marker).saw_SOI != 0 {
    ERREXIT!(cinfo, JERR_SOI_DUPLICATE);
  }

  /* Reset all parameters that are defined to be reset by SOI */

  i = 0;
  while i < NUM_ARITH_TBLS {
    (*cinfo).arith_dc_L[i as usize] = 0;
    (*cinfo).arith_dc_U[i as usize] = 1;
    (*cinfo).arith_ac_K[i as usize] = 5;
    i += 1;
  }
  (*cinfo).restart_interval = 0;

  /* Set initial assumptions for colorspace etc */

  (*cinfo).jpeg_color_space = JCS_UNKNOWN;
  (*cinfo).CCIR601_sampling = FALSE; /* Assume non-CCIR sampling??? */

  (*cinfo).saw_JFIF_marker = FALSE;
  (*cinfo).density_unit = 0;	/* set default JFIF APP0 values */
  (*cinfo).X_density = 1;
  (*cinfo).Y_density = 1;
  (*cinfo).saw_Adobe_marker = FALSE;
  (*cinfo).Adobe_transform = 0;

  (*(*cinfo).marker).saw_SOI = TRUE;

  return TRUE;
}


unsafe fn get_sof(cinfo: j_decompress_ptr, is_prog: boolean, is_arith: boolean) -> boolean
/* Process a SOFn marker */
{
  let mut length: INT32 = 0;
  let mut c: c_int = 0;
  let mut ci: c_int;
  let mut compptr: *mut jpeg_component_info;
  INPUT_VARS!(cinfo);

  (*cinfo).progressive_mode = is_prog;
  (*cinfo).arith_code = is_arith;

  INPUT_2BYTES!(cinfo, length, return FALSE);

  INPUT_BYTE!(cinfo, (*cinfo).data_precision, return FALSE);
  INPUT_2BYTES!(cinfo, (*cinfo).image_height, return FALSE);
  INPUT_2BYTES!(cinfo, (*cinfo).image_width, return FALSE);
  INPUT_BYTE!(cinfo, (*cinfo).num_components, return FALSE);

  length -= 8;

  TRACEMS4!(cinfo, 1, JTRC_SOF, (*cinfo).unread_marker,
	   (*cinfo).image_width as c_int, (*cinfo).image_height as c_int,
	   (*cinfo).num_components);

  if (*(*cinfo).marker).saw_SOF != 0 {
    ERREXIT!(cinfo, JERR_SOF_DUPLICATE);
  }

  /* We don't support files in which the image height is initially specified */
  /* as 0 and is later redefined by DNL.  As long as we have to check that,  */
  /* might as well have a general sanity check. */
  if (*cinfo).image_height <= 0 || (*cinfo).image_width <= 0
      || (*cinfo).num_components <= 0 {
    ERREXIT!(cinfo, JERR_EMPTY_IMAGE);
  }

  if length != ((*cinfo).num_components * 3) {
    ERREXIT!(cinfo, JERR_BAD_LENGTH);
  }

  if (*cinfo).comp_info.is_null() {	/* do only once, even if suspend */
    (*cinfo).comp_info = ((*(*cinfo).mem).alloc_small.unwrap())(
			(cinfo as j_common_ptr), JPOOL_IMAGE,
			(*cinfo).num_components as usize * core::mem::size_of::<jpeg_component_info>())
			as *mut jpeg_component_info;
  }

  ci = 0;
  compptr = (*cinfo).comp_info;
  while ci < (*cinfo).num_components {
    (*compptr).component_index = ci;
    INPUT_BYTE!(cinfo, (*compptr).component_id, return FALSE);
    INPUT_BYTE!(cinfo, c, return FALSE);
    (*compptr).h_samp_factor = (c >> 4) & 15;
    (*compptr).v_samp_factor = (c     ) & 15;
    INPUT_BYTE!(cinfo, (*compptr).quant_tbl_no, return FALSE);

    TRACEMS4!(cinfo, 1, JTRC_SOF_COMPONENT,
	     (*compptr).component_id, (*compptr).h_samp_factor,
	     (*compptr).v_samp_factor, (*compptr).quant_tbl_no);
    ci += 1;
    compptr = compptr.add(1);
  }

  (*(*cinfo).marker).saw_SOF = TRUE;

  INPUT_SYNC!(cinfo);
  return TRUE;
}


unsafe fn get_sos(cinfo: j_decompress_ptr) -> boolean
/* Process a SOS marker */
{
  let mut length: INT32 = 0;
  let mut i: c_int;
  let mut ci: c_int;
  let mut n: c_int = 0;
  let mut c: c_int = 0;
  let mut cc: c_int = 0;
  let mut compptr: *mut jpeg_component_info;
  INPUT_VARS!(cinfo);

  if (*(*cinfo).marker).saw_SOF == 0 {
    ERREXIT!(cinfo, JERR_SOS_NO_SOF);
  }

  INPUT_2BYTES!(cinfo, length, return FALSE);

  INPUT_BYTE!(cinfo, n, return FALSE); /* Number of components */

  if length != (n * 2 + 6) || n < 1 || n > MAX_COMPS_IN_SCAN {
    ERREXIT!(cinfo, JERR_BAD_LENGTH);
  }

  TRACEMS1!(cinfo, 1, JTRC_SOS, n);

  (*cinfo).comps_in_scan = n;

  /* Collect the component-spec parameters */

  i = 0;
  while i < n {
    INPUT_BYTE!(cinfo, cc, return FALSE);
    INPUT_BYTE!(cinfo, c, return FALSE);

    ci = 0;
    compptr = (*cinfo).comp_info;
    let mut id_found = false;
    while ci < (*cinfo).num_components {
      if cc == (*compptr).component_id {
        id_found = true;
        break;
      }
      ci += 1;
      compptr = compptr.add(1);
    }

    if !id_found {
      ERREXIT1!(cinfo, JERR_BAD_COMPONENT_ID, cc);
    }

    /* id_found: */
    (*cinfo).cur_comp_info[i as usize] = compptr;
    (*compptr).dc_tbl_no = (c >> 4) & 15;
    (*compptr).ac_tbl_no = (c     ) & 15;

    TRACEMS3!(cinfo, 1, JTRC_SOS_COMPONENT, cc,
	     (*compptr).dc_tbl_no, (*compptr).ac_tbl_no);
    i += 1;
  }

  /* Collect the additional scan parameters Ss, Se, Ah/Al. */
  INPUT_BYTE!(cinfo, c, return FALSE);
  (*cinfo).Ss = c;
  INPUT_BYTE!(cinfo, c, return FALSE);
  (*cinfo).Se = c;
  INPUT_BYTE!(cinfo, c, return FALSE);
  (*cinfo).Ah = (c >> 4) & 15;
  (*cinfo).Al = (c     ) & 15;

  TRACEMS4!(cinfo, 1, JTRC_SOS_PARAMS, (*cinfo).Ss, (*cinfo).Se,
	   (*cinfo).Ah, (*cinfo).Al);

  /* Prepare to scan data & restart markers */
  (*(*cinfo).marker).next_restart_num = 0;

  /* Count another SOS marker */
  (*cinfo).input_scan_number += 1;

  INPUT_SYNC!(cinfo);
  return TRUE;
}


unsafe extern "C" fn get_app0(cinfo: j_decompress_ptr) -> boolean
/* Process an APP0 marker */
{
  const JFIF_LEN: c_int = 14;
  let mut length: INT32 = 0;
  let mut b: [UINT8; 14] = [0; 14];
  let mut buffp: c_int;
  INPUT_VARS!(cinfo);

  INPUT_2BYTES!(cinfo, length, return FALSE);
  length -= 2;

  /* See if a JFIF APP0 marker is present */

  if length >= JFIF_LEN as INT32 {
    buffp = 0;
    while buffp < JFIF_LEN {
      INPUT_BYTE!(cinfo, b[buffp as usize], return FALSE);
      buffp += 1;
    }
    length -= JFIF_LEN as INT32;

    if b[0]==0x4A && b[1]==0x46 && b[2]==0x49 && b[3]==0x46 && b[4]==0 {
      /* Found JFIF APP0 marker: check version */
      /* Major version must be 1, anything else signals an incompatible change.
       * We used to treat this as an error, but now it's a nonfatal warning,
       * because some bozo at Hijaak couldn't read the spec.
       * Minor version should be 0..2, but process anyway if newer.
       */
      if b[5] != 1 {
	WARNMS2!(cinfo, JWRN_JFIF_MAJOR, b[5], b[6]);
      } else if b[6] > 2 {
	TRACEMS2!(cinfo, 1, JTRC_JFIF_MINOR, b[5], b[6]);
      }
      /* Save info */
      (*cinfo).saw_JFIF_marker = TRUE;
      (*cinfo).density_unit = b[7];
      (*cinfo).X_density = (((b[8] as c_int) << 8) + b[9] as c_int) as u16;
      (*cinfo).Y_density = (((b[10] as c_int) << 8) + b[11] as c_int) as u16;
      TRACEMS3!(cinfo, 1, JTRC_JFIF,
	       (*cinfo).X_density, (*cinfo).Y_density, (*cinfo).density_unit);
      if (b[12] | b[13]) != 0 {
	TRACEMS2!(cinfo, 1, JTRC_JFIF_THUMBNAIL, b[12], b[13]);
      }
      if length != (b[12] as INT32) * (b[13] as INT32) * 3 {
	TRACEMS1!(cinfo, 1, JTRC_JFIF_BADTHUMBNAILSIZE, length as c_int);
      }
    } else {
      /* Start of APP0 does not match "JFIF" */
      TRACEMS1!(cinfo, 1, JTRC_APP0, (length + JFIF_LEN as INT32) as c_int);
    }
  } else {
    /* Too short to be JFIF marker */
    TRACEMS1!(cinfo, 1, JTRC_APP0, length as c_int);
  }

  INPUT_SYNC!(cinfo);
  if length > 0 {		/* skip any remaining data -- could be lots */
    ((*(*cinfo).src).skip_input_data.unwrap())(cinfo, length as c_int);
  }

  return TRUE;
}


unsafe extern "C" fn get_app14(cinfo: j_decompress_ptr) -> boolean
/* Process an APP14 marker */
{
  const ADOBE_LEN: c_int = 12;
  let mut length: INT32 = 0;
  let mut b: [UINT8; 12] = [0; 12];
  let mut buffp: c_int;
  let mut version: c_uint;
  let mut flags0: c_uint;
  let mut flags1: c_uint;
  let mut transform: c_uint;
  INPUT_VARS!(cinfo);

  INPUT_2BYTES!(cinfo, length, return FALSE);
  length -= 2;

  /* See if an Adobe APP14 marker is present */

  if length >= ADOBE_LEN as INT32 {
    buffp = 0;
    while buffp < ADOBE_LEN {
      INPUT_BYTE!(cinfo, b[buffp as usize], return FALSE);
      buffp += 1;
    }
    length -= ADOBE_LEN as INT32;

    if b[0]==0x41 && b[1]==0x64 && b[2]==0x6F && b[3]==0x62 && b[4]==0x65 {
      /* Found Adobe APP14 marker */
      version = ((b[5] as c_uint) << 8) + b[6] as c_uint;
      flags0 = ((b[7] as c_uint) << 8) + b[8] as c_uint;
      flags1 = ((b[9] as c_uint) << 8) + b[10] as c_uint;
      transform = b[11] as c_uint;
      TRACEMS4!(cinfo, 1, JTRC_ADOBE, version, flags0, flags1, transform);
      (*cinfo).saw_Adobe_marker = TRUE;
      (*cinfo).Adobe_transform = transform as UINT8;
    } else {
      /* Start of APP14 does not match "Adobe" */
      TRACEMS1!(cinfo, 1, JTRC_APP14, (length + ADOBE_LEN as INT32) as c_int);
    }
  } else {
    /* Too short to be Adobe marker */
    TRACEMS1!(cinfo, 1, JTRC_APP14, length as c_int);
  }

  INPUT_SYNC!(cinfo);
  if length > 0 {		/* skip any remaining data -- could be lots */
    ((*(*cinfo).src).skip_input_data.unwrap())(cinfo, length as c_int);
  }

  return TRUE;
}


unsafe fn get_dac(cinfo: j_decompress_ptr) -> boolean
/* Process a DAC marker */
{
  let mut length: INT32 = 0;
  let mut index: c_int = 0;
  let mut val: c_int = 0;
  INPUT_VARS!(cinfo);

  INPUT_2BYTES!(cinfo, length, return FALSE);
  length -= 2;

  while length > 0 {
    INPUT_BYTE!(cinfo, index, return FALSE);
    INPUT_BYTE!(cinfo, val, return FALSE);

    length -= 2;

    TRACEMS2!(cinfo, 1, JTRC_DAC, index, val);

    if index < 0 || index >= (2 * NUM_ARITH_TBLS) {
      ERREXIT1!(cinfo, JERR_DAC_INDEX, index);
    }

    if index >= NUM_ARITH_TBLS { /* define AC table */
      (*cinfo).arith_ac_K[(index - NUM_ARITH_TBLS) as usize] = val as UINT8;
    } else {			/* define DC table */
      (*cinfo).arith_dc_L[index as usize] = (val & 0x0F) as UINT8;
      (*cinfo).arith_dc_U[index as usize] = (val >> 4) as UINT8;
      if (*cinfo).arith_dc_L[index as usize] > (*cinfo).arith_dc_U[index as usize] {
	ERREXIT1!(cinfo, JERR_DAC_VALUE, val);
      }
    }
  }

  INPUT_SYNC!(cinfo);
  return TRUE;
}


unsafe fn get_dht(cinfo: j_decompress_ptr) -> boolean
/* Process a DHT marker */
{
  let mut length: INT32 = 0;
  let mut bits: [UINT8; 17] = [0; 17];
  let mut huffval: [UINT8; 256] = [0; 256];
  let mut i: c_int;
  let mut index: c_int = 0;
  let mut count: c_int;
  let mut htblptr: *mut *mut JHUFF_TBL;
  INPUT_VARS!(cinfo);

  INPUT_2BYTES!(cinfo, length, return FALSE);
  length -= 2;

  while length > 0 {
    INPUT_BYTE!(cinfo, index, return FALSE);

    TRACEMS1!(cinfo, 1, JTRC_DHT, index);

    bits[0] = 0;
    count = 0;
    i = 1;
    while i <= 16 {
      INPUT_BYTE!(cinfo, bits[i as usize], return FALSE);
      count += bits[i as usize] as c_int;
      i += 1;
    }

    length -= 1 + 16;

    TRACEMS8!(cinfo, 2, JTRC_HUFFBITS,
	     bits[1], bits[2], bits[3], bits[4],
	     bits[5], bits[6], bits[7], bits[8]);
    TRACEMS8!(cinfo, 2, JTRC_HUFFBITS,
	     bits[9], bits[10], bits[11], bits[12],
	     bits[13], bits[14], bits[15], bits[16]);

    if count > 256 || (count as INT32) > length {
      ERREXIT!(cinfo, JERR_DHT_COUNTS);
    }

    i = 0;
    while i < count {
      INPUT_BYTE!(cinfo, huffval[i as usize], return FALSE);
      i += 1;
    }

    length -= count as INT32;

    if index & 0x10 != 0 {		/* AC table definition */
      index -= 0x10;
      htblptr = &mut (*cinfo).ac_huff_tbl_ptrs[index as usize];
    } else {			/* DC table definition */
      htblptr = &mut (*cinfo).dc_huff_tbl_ptrs[index as usize];
    }

    if index < 0 || index >= NUM_HUFF_TBLS {
      ERREXIT1!(cinfo, JERR_DHT_INDEX, index);
    }

    if (*htblptr).is_null() {
      *htblptr = jpeg_alloc_huff_table(cinfo as j_common_ptr);
    }

    MEMCOPY(
      (*(*htblptr)).bits.as_mut_ptr() as *mut c_void,
      bits.as_ptr() as *const c_void,
      core::mem::size_of_val(&(*(*htblptr)).bits),
    );
    MEMCOPY(
      (*(*htblptr)).huffval.as_mut_ptr() as *mut c_void,
      huffval.as_ptr() as *const c_void,
      core::mem::size_of_val(&(*(*htblptr)).huffval),
    );
  }

  INPUT_SYNC!(cinfo);
  return TRUE;
}


unsafe fn get_dqt(cinfo: j_decompress_ptr) -> boolean
/* Process a DQT marker */
{
  let mut length: INT32 = 0;
  let mut n: c_int = 0;
  let mut i: c_int;
  let mut prec: c_int;
  let mut tmp: c_uint = 0;
  let mut quant_ptr: *mut JQUANT_TBL;
  INPUT_VARS!(cinfo);

  INPUT_2BYTES!(cinfo, length, return FALSE);
  length -= 2;

  while length > 0 {
    INPUT_BYTE!(cinfo, n, return FALSE);
    prec = n >> 4;
    n &= 0x0F;

    TRACEMS2!(cinfo, 1, JTRC_DQT, n, prec);

    if n >= NUM_QUANT_TBLS {
      ERREXIT1!(cinfo, JERR_DQT_INDEX, n);
    }

    if (*cinfo).quant_tbl_ptrs[n as usize].is_null() {
      (*cinfo).quant_tbl_ptrs[n as usize] =
        jpeg_alloc_quant_table(cinfo as j_common_ptr);
    }
    quant_ptr = (*cinfo).quant_tbl_ptrs[n as usize];

    i = 0;
    while i < DCTSIZE2 {
      if prec != 0 {
	INPUT_2BYTES!(cinfo, tmp, return FALSE);
      } else {
	INPUT_BYTE!(cinfo, tmp, return FALSE);
      }
      (*quant_ptr).quantval[i as usize] = tmp as UINT16;
      i += 1;
    }

    i = 0;
    while i < DCTSIZE2 {
      TRACEMS8!(cinfo, 2, JTRC_QUANTVALS,
	       (*quant_ptr).quantval[i as usize],
	       (*quant_ptr).quantval[(i+1) as usize],
	       (*quant_ptr).quantval[(i+2) as usize],
	       (*quant_ptr).quantval[(i+3) as usize],
	       (*quant_ptr).quantval[(i+4) as usize],
	       (*quant_ptr).quantval[(i+5) as usize],
	       (*quant_ptr).quantval[(i+6) as usize],
	       (*quant_ptr).quantval[(i+7) as usize]);
      i += 8;
    }

    length -= DCTSIZE2 + 1;
    if prec != 0 { length -= DCTSIZE2; }
  }

  INPUT_SYNC!(cinfo);
  return TRUE;
}


unsafe fn get_dri(cinfo: j_decompress_ptr) -> boolean
/* Process a DRI marker */
{
  let mut length: INT32 = 0;
  let mut tmp: c_uint = 0;
  INPUT_VARS!(cinfo);

  INPUT_2BYTES!(cinfo, length, return FALSE);

  if length != 4 {
    ERREXIT!(cinfo, JERR_BAD_LENGTH);
  }

  INPUT_2BYTES!(cinfo, tmp, return FALSE);

  TRACEMS1!(cinfo, 1, JTRC_DRI, tmp);

  (*cinfo).restart_interval = tmp;

  INPUT_SYNC!(cinfo);
  return TRUE;
}


unsafe extern "C" fn skip_variable(cinfo: j_decompress_ptr) -> boolean
/* Skip over an unknown or uninteresting variable-length marker */
{
  let mut length: INT32 = 0;
  INPUT_VARS!(cinfo);

  INPUT_2BYTES!(cinfo, length, return FALSE);

  TRACEMS2!(cinfo, 1, JTRC_MISC_MARKER, (*cinfo).unread_marker, length as c_int);

  INPUT_SYNC!(cinfo);		/* do before skip_input_data */
  ((*(*cinfo).src).skip_input_data.unwrap())(cinfo, (length - 2) as c_int);

  return TRUE;
}


/*
 * Find the next JPEG marker, save it in cinfo->unread_marker.
 * Returns FALSE if had to suspend before reaching a marker;
 * in that case cinfo->unread_marker is unchanged.
 *
 * Note that the result might not be a valid marker code,
 * but it will never be 0 or FF.
 */

unsafe fn next_marker(cinfo: j_decompress_ptr) -> boolean
{
  let mut c: c_int = 0;
  INPUT_VARS!(cinfo);

  loop {
    INPUT_BYTE!(cinfo, c, return FALSE);
    /* Skip any non-FF bytes.
     * This may look a bit inefficient, but it will not occur in a valid file.
     * We sync after each discarded byte so that a suspending data source
     * can discard the byte from its buffer.
     */
    while c != 0xFF {
      (*(*cinfo).marker).discarded_bytes += 1;
      INPUT_SYNC!(cinfo);
      INPUT_BYTE!(cinfo, c, return FALSE);
    }
    /* This loop swallows any duplicate FF bytes.  Extra FFs are legal as
     * pad bytes, so don't count them in discarded_bytes.  We assume there
     * will not be so many consecutive FF bytes as to overflow a suspending
     * data source's input buffer.
     */
    loop {
      INPUT_BYTE!(cinfo, c, return FALSE);
      if c != 0xFF { break; }
    }
    if c != 0 {
      break;			/* found a valid marker, exit loop */
    }
    /* Reach here if we found a stuffed-zero data sequence (FF/00).
     * Discard it and loop back to try again.
     */
    (*(*cinfo).marker).discarded_bytes += 2;
    INPUT_SYNC!(cinfo);
  }

  if (*(*cinfo).marker).discarded_bytes != 0 {
    WARNMS2!(cinfo, JWRN_EXTRANEOUS_DATA, (*(*cinfo).marker).discarded_bytes, c);
    (*(*cinfo).marker).discarded_bytes = 0;
  }

  (*cinfo).unread_marker = c;

  INPUT_SYNC!(cinfo);
  return TRUE;
}


unsafe fn first_marker(cinfo: j_decompress_ptr) -> boolean
/* Like next_marker, but used to obtain the initial SOI marker. */
/* For this marker, we do not allow preceding garbage or fill; otherwise,
 * we might well scan an entire input file before realizing it ain't JPEG.
 * If an application wants to process non-JFIF files, it must seek to the
 * SOI before calling the JPEG library.
 */
{
  let mut c: c_int = 0;
  let mut c2: c_int = 0;
  INPUT_VARS!(cinfo);

  INPUT_BYTE!(cinfo, c, return FALSE);
  INPUT_BYTE!(cinfo, c2, return FALSE);
  if c != 0xFF || c2 != M_SOI {
    ERREXIT2!(cinfo, JERR_NO_SOI, c, c2);
  }

  (*cinfo).unread_marker = c2;

  INPUT_SYNC!(cinfo);
  return TRUE;
}


/*
 * Read markers until SOS or EOI.
 *
 * Returns same codes as are defined for jpeg_consume_input:
 * JPEG_SUSPENDED, JPEG_REACHED_SOS, or JPEG_REACHED_EOI.
 */

unsafe extern "C" fn read_markers(cinfo: j_decompress_ptr) -> c_int
{
  /* Outer loop repeats once for each marker. */
  loop {
    /* Collect the marker proper, unless we already did. */
    /* NB: first_marker() enforces the requirement that SOI appear first. */
    if (*cinfo).unread_marker == 0 {
      if (*(*cinfo).marker).saw_SOI == 0 {
	if first_marker(cinfo) == FALSE {
	  return JPEG_SUSPENDED;
	}
      } else {
	if next_marker(cinfo) == FALSE {
	  return JPEG_SUSPENDED;
	}
      }
    }
    /* At this point cinfo->unread_marker contains the marker code and the
     * input point is just past the marker proper, but before any parameters.
     * A suspension will cause us to return with this state still true.
     */
    match (*cinfo).unread_marker {
    M_SOI => {
      if get_soi(cinfo) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_SOF0 |		/* Baseline */
    M_SOF1 => {		/* Extended sequential, Huffman */
      if get_sof(cinfo, FALSE, FALSE) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_SOF2 => {		/* Progressive, Huffman */
      if get_sof(cinfo, TRUE, FALSE) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_SOF9 => {		/* Extended sequential, arithmetic */
      if get_sof(cinfo, FALSE, TRUE) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_SOF10 => {	/* Progressive, arithmetic */
      if get_sof(cinfo, TRUE, TRUE) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    /* Currently unsupported SOFn types */
    M_SOF3 |		/* Lossless, Huffman */
    M_SOF5 |		/* Differential sequential, Huffman */
    M_SOF6 |		/* Differential progressive, Huffman */
    M_SOF7 |		/* Differential lossless, Huffman */
    M_JPG  |		/* Reserved for JPEG extensions */
    M_SOF11 |		/* Lossless, arithmetic */
    M_SOF13 |		/* Differential sequential, arithmetic */
    M_SOF14 |		/* Differential progressive, arithmetic */
    M_SOF15 => {	/* Differential lossless, arithmetic */
      ERREXIT1!(cinfo, JERR_SOF_UNSUPPORTED, (*cinfo).unread_marker);
    }

    M_SOS => {
      if get_sos(cinfo) == FALSE {
	return JPEG_SUSPENDED;
      }
      (*cinfo).unread_marker = 0;	/* processed the marker */
      return JPEG_REACHED_SOS;
    }

    M_EOI => {
      TRACEMS!(cinfo, 1, JTRC_EOI);
      (*cinfo).unread_marker = 0;	/* processed the marker */
      return JPEG_REACHED_EOI;
    }

    M_DAC => {
      if get_dac(cinfo) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_DHT => {
      if get_dht(cinfo) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_DQT => {
      if get_dqt(cinfo) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_DRI => {
      if get_dri(cinfo) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_APP0  |
    M_APP1  |
    M_APP2  |
    M_APP3  |
    M_APP4  |
    M_APP5  |
    M_APP6  |
    M_APP7  |
    M_APP8  |
    M_APP9  |
    M_APP10 |
    M_APP11 |
    M_APP12 |
    M_APP13 |
    M_APP14 |
    M_APP15 => {
      let idx = ((*cinfo).unread_marker - M_APP0) as usize;
      if ((*(*cinfo).marker).process_APPn[idx])(cinfo) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_COM => {
      if ((*(*cinfo).marker).process_COM)(cinfo) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    M_RST0 |	/* these are all parameterless */
    M_RST1 |
    M_RST2 |
    M_RST3 |
    M_RST4 |
    M_RST5 |
    M_RST6 |
    M_RST7 |
    M_TEM => {
      TRACEMS1!(cinfo, 1, JTRC_PARMLESS_MARKER, (*cinfo).unread_marker);
    }

    M_DNL => {		/* Ignore DNL ... perhaps the wrong thing */
      if skip_variable(cinfo) == FALSE {
	return JPEG_SUSPENDED;
      }
    }

    _ => {		/* must be DHP, EXP, JPGn, or RESn */
      /* For now, we treat the reserved markers as fatal errors since they are
       * likely to be used to signal incompatible JPEG Part 3 extensions.
       * Once the JPEG 3 version-number marker is well defined, this code
       * ought to change!
       */
      ERREXIT1!(cinfo, JERR_UNKNOWN_MARKER, (*cinfo).unread_marker);
    }
    }
    /* Successfully processed marker, so reset state variable */
    (*cinfo).unread_marker = 0;
  } /* end loop */
}


/*
 * Read a restart marker, which is expected to appear next in the datastream;
 * if the marker is not there, take appropriate recovery action.
 * Returns FALSE if suspension is required.
 *
 * This is called by the entropy decoder after it has read an appropriate
 * number of MCUs.  cinfo->unread_marker may be nonzero if the entropy decoder
 * has already read a marker from the data source.  Under normal conditions
 * cinfo->unread_marker will be reset to 0 before returning; if not reset,
 * it holds a marker which the decoder will be unable to read past.
 */

unsafe extern "C" fn read_restart_marker(cinfo: j_decompress_ptr) -> boolean
{
  /* Obtain a marker unless we already did. */
  /* Note that next_marker will complain if it skips any data. */
  if (*cinfo).unread_marker == 0 {
    if next_marker(cinfo) == FALSE {
      return FALSE;
    }
  }

  if (*cinfo).unread_marker ==
      (M_RST0 + (*(*cinfo).marker).next_restart_num) {
    /* Normal case --- swallow the marker and let entropy decoder continue */
    TRACEMS1!(cinfo, 2, JTRC_RST, (*(*cinfo).marker).next_restart_num);
    (*cinfo).unread_marker = 0;
  } else {
    /* Uh-oh, the restart markers have been messed up. */
    /* Let the data source manager determine how to resync. */
    if ((*(*cinfo).src).resync_to_restart.unwrap())(cinfo,
					    (*(*cinfo).marker).next_restart_num) == FALSE {
      return FALSE;
    }
  }

  /* Update next-restart state */
  (*(*cinfo).marker).next_restart_num = ((*(*cinfo).marker).next_restart_num + 1) & 7;

  return TRUE;
}


/*
 * This is the default resync_to_restart method for data source managers
 * to use if they don't have any better approach.  Some data source managers
 * may be able to back up, or may have additional knowledge about the data
 * which permits a more intelligent recovery strategy; such managers would
 * presumably supply their own resync method.
 *
 * read_restart_marker calls resync_to_restart if it finds a marker other than
 * the restart marker it was expecting.  (This code is *not* used unless
 * a nonzero restart interval has been declared.)  cinfo->unread_marker is
 * the marker code actually found (might be anything, except 0 or FF).
 * The desired restart marker number (0..7) is passed as a parameter.
 * This routine is supposed to apply whatever error recovery strategy seems
 * appropriate in order to position the input stream to the next data segment.
 * Note that cinfo->unread_marker is treated as a marker appearing before
 * the current data-source input point; usually it should be reset to zero
 * before returning.
 * Returns FALSE if suspension is required.
 *
 * This implementation is substantially constrained by wanting to treat the
 * input as a data stream; this means we can't back up.  Therefore, we have
 * only the following actions to work with:
 *   1. Simply discard the marker and let the entropy decoder resume at next
 *      byte of file.
 *   2. Read forward until we find another marker, discarding intervening
 *      data.  (In theory we could look ahead within the current bufferload,
 *      without having to discard data if we don't find the desired marker.
 *      This idea is not implemented here, in part because it makes behavior
 *      dependent on buffer size and chance buffer-boundary positions.)
 *   3. Leave the marker unread (by failing to zero cinfo->unread_marker).
 *      This will cause the entropy decoder to process an empty data segment,
 *      inserting dummy zeroes, and then we will reprocess the marker.
 *
 * #2 is appropriate if we think the desired marker lies ahead, while #3 is
 * appropriate if the found marker is a future restart marker (indicating
 * that we have missed the desired restart marker, probably because it got
 * corrupted).
 * We apply #2 or #3 if the found marker is a restart marker no more than
 * two counts behind or ahead of the expected one.  We also apply #2 if the
 * found marker is not a legal JPEG marker code (it's certainly bogus data).
 * If the found marker is a restart marker more than 2 counts away, we do #1
 * (too much risk that the marker is erroneous; with luck we will be able to
 * resync at some future point).
 * For any valid non-restart JPEG marker, we apply #3.  This keeps us from
 * overrunning the end of a scan.  An implementation limited to single-scan
 * files might find it better to apply #2 for markers other than EOI, since
 * any other marker would have to be bogus data in that case.
 */

pub unsafe extern "C" fn jpeg_resync_to_restart(cinfo: j_decompress_ptr, desired: c_int) -> boolean
{
  let mut marker: c_int = (*cinfo).unread_marker;
  let mut action: c_int = 1;

  /* Always put up a warning. */
  WARNMS2!(cinfo, JWRN_MUST_RESYNC, marker, desired);

  /* Outer loop handles repeated decision after scanning forward. */
  loop {
    if marker < M_SOF0 {
      action = 2;		/* invalid marker */
    } else if marker < M_RST0 || marker > M_RST7 {
      action = 3;		/* valid non-restart marker */
    } else {
      if marker == (M_RST0 + ((desired+1) & 7)) ||
	  marker == (M_RST0 + ((desired+2) & 7)) {
	action = 3;		/* one of the next two expected restarts */
      } else if marker == (M_RST0 + ((desired-1) & 7)) ||
	       marker == (M_RST0 + ((desired-2) & 7)) {
	action = 2;		/* a prior restart, so advance */
      } else {
	action = 1;		/* desired restart or too far away */
      }
    }
    TRACEMS2!(cinfo, 4, JTRC_RECOVERY_ACTION, marker, action);
    match action {
    1 => {
      /* Discard marker and let entropy decoder resume processing. */
      (*cinfo).unread_marker = 0;
      return TRUE;
    }
    2 => {
      /* Scan to the next marker, and repeat the decision loop. */
      if next_marker(cinfo) == FALSE {
	return FALSE;
      }
      marker = (*cinfo).unread_marker;
    }
    3 => {
      /* Return without advancing past this marker. */
      /* Entropy decoder will be forced to process an empty segment. */
      return TRUE;
    }
    _ => {}
    }
  } /* end loop */
}


/*
 * Reset marker processing state to begin a fresh datastream.
 */

unsafe extern "C" fn reset_marker_reader(cinfo: j_decompress_ptr)
{
  (*cinfo).comp_info = core::ptr::null_mut();	/* until allocated by get_sof */
  (*cinfo).input_scan_number = 0;		/* no SOS seen yet */
  (*cinfo).unread_marker = 0;			/* no pending marker */
  (*(*cinfo).marker).saw_SOI = FALSE;		/* set internal state too */
  (*(*cinfo).marker).saw_SOF = FALSE;
  (*(*cinfo).marker).discarded_bytes = 0;
}


/*
 * Initialize the marker reader module.
 * This is called only once, when the decompression object is created.
 */

pub unsafe extern "C" fn jinit_marker_reader(cinfo: j_decompress_ptr)
{
  let mut i: c_int;

  /* Create subobject in permanent pool */
  (*cinfo).marker = ((*(*cinfo).mem).alloc_small.unwrap())(
    (cinfo as j_common_ptr), JPOOL_PERMANENT,
    core::mem::size_of::<jpeg_marker_reader>()) as *mut jpeg_marker_reader;
  /* Initialize method pointers */
  (*(*cinfo).marker).reset_marker_reader = Some(reset_marker_reader);
  (*(*cinfo).marker).read_markers = Some(read_markers);
  (*(*cinfo).marker).read_restart_marker = read_restart_marker;
  (*(*cinfo).marker).process_COM = skip_variable;
  i = 0;
  while i < 16 {
    (*(*cinfo).marker).process_APPn[i as usize] = skip_variable;
    i += 1;
  }
  (*(*cinfo).marker).process_APPn[0] = get_app0;
  (*(*cinfo).marker).process_APPn[14] = get_app14;
  /* Reset marker processing state */
  reset_marker_reader(cinfo);
}
