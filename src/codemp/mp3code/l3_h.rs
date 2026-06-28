/*____________________________________________________________________________

    FreeAmp - The Free MP3 Player

        MP3 Decoder originally Copyright (C) 1996-1997 Xing Technology
        Corp.  http://www.xingtech.com

    Portions Copyright (C) 1998-1999 Emusic.com

    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program; if not, write to the Free Software
    Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

    $Id: L3.h,v 1.7 1999/12/10 07:16:42 elrod Exp $
____________________________________________________________________________*/

/****  L3.h  ***************************************************

  Layer III structures

  *** Layer III is 32 bit only          ***
  *** Layer III code assumes 32 bit int ***

******************************************************************/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_uchar, c_uint};

use super::small_header_h::SAMPLE;

pub const GLOBAL_GAIN_SCALE: c_int = 4 * 15;
/* #define GLOBAL_GAIN_SCALE 0 */

/*-----------------------------------------------------------*/
/*---- huffman lookup tables ---*/
/* endian dependent !!! */
#[repr(C)]
#[derive(Clone, Copy)]
pub union HUFF_ELEMENT {
    pub ptr: c_int,
    pub b: HUFF_ELEMENT_b,
}

#[cfg(target_endian = "little")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HUFF_ELEMENT_b {
    pub signbits: c_uchar,
    pub x: c_uchar,
    pub y: c_uchar,
    pub purgebits: c_uchar, /* 0 = esc */
}

#[cfg(target_endian = "big")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HUFF_ELEMENT_b {
    pub purgebits: c_uchar, /* 0 = esc */
    pub y: c_uchar,
    pub x: c_uchar,
    pub signbits: c_uchar,
}

/*--------------------------------------------------------------*/
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BITDAT {
    pub bitbuf: c_uint,
    pub bits: c_int,
    pub bs_ptr: *mut c_uchar,
    pub bs_ptr0: *mut c_uchar,
    pub bs_ptr_end: *mut c_uchar, /* optional for overrun test */
}

/*-- side info ---*/
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GR {
    pub part2_3_length: c_int,
    pub big_values: c_int,
    pub global_gain: c_int,
    pub scalefac_compress: c_int,
    pub window_switching_flag: c_int,
    pub block_type: c_int,
    pub mixed_block_flag: c_int,
    pub table_select: [c_int; 3],
    pub subblock_gain: [c_int; 3],
    pub region0_count: c_int,
    pub region1_count: c_int,
    pub preflag: c_int,
    pub scalefac_scale: c_int,
    pub count1table_select: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SIDE_INFO {
    pub mode: c_int,
    pub mode_ext: c_int,
    /*---------------*/
    pub main_data_begin: c_int, /* beginning, not end, my spec wrong */
    pub private_bits: c_int,
    /*---------------*/
    pub scfsi: [c_int; 2],     /* 4 bit flags [ch] */
    pub gr: [[GR; 2]; 2],      /* [gran][ch] */
}

/*-----------------------------------------------------------*/
/*-- scale factors ---*/
// check dimensions - need 21 long, 3*12 short
// plus extra for implicit sf=0 above highest cb
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SCALEFACT {
    pub l: [c_int; 23],       /* [cb] */
    pub s: [[c_int; 13]; 3],  /* [window][cb] */
}

/*-----------------------------------------------------------*/
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CB_INFO {
    pub cbtype: c_int,     /* long=0 short=1 */
    pub cbmax: c_int,      /* max crit band */
    //   int lb_type;          /* long block type 0 1 3 */
    pub cbs0: c_int,       /* short band start index 0 3 12 (12=no shorts */
    pub ncbl: c_int,       /* number long cb's 0 8 21 */
    pub cbmax_s: [c_int; 3], /* cbmax by individual short blocks */
}

/*-----------------------------------------------------------*/
/* scale factor infor for MPEG2 intensity stereo  */
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IS_SF_INFO {
    pub nr: [c_int; 3],
    pub slen: [c_int; 3],
    pub intensity_scale: c_int,
}

const _: () = assert!(core::mem::size_of::<HUFF_ELEMENT_b>() == 4);
const _: () = assert!(core::mem::align_of::<HUFF_ELEMENT_b>() == 1);
const _: () = assert!(core::mem::size_of::<HUFF_ELEMENT>() == 4);
const _: () = assert!(core::mem::align_of::<HUFF_ELEMENT>() == 4);

const _: () = {
    let expected = 8 + 3 * core::mem::size_of::<*mut c_uchar>();
    assert!(core::mem::size_of::<BITDAT>() == expected);
};
const _: () = assert!(core::mem::align_of::<BITDAT>() == core::mem::align_of::<*mut c_uchar>());

const _: () = assert!(core::mem::size_of::<GR>() == 68);
const _: () = assert!(core::mem::align_of::<GR>() == 4);
const _: () = assert!(core::mem::size_of::<SIDE_INFO>() == 288);
const _: () = assert!(core::mem::align_of::<SIDE_INFO>() == 4);
const _: () = assert!(core::mem::size_of::<SCALEFACT>() == 248);
const _: () = assert!(core::mem::align_of::<SCALEFACT>() == 4);
const _: () = assert!(core::mem::size_of::<CB_INFO>() == 28);
const _: () = assert!(core::mem::align_of::<CB_INFO>() == 4);
const _: () = assert!(core::mem::size_of::<IS_SF_INFO>() == 28);
const _: () = assert!(core::mem::align_of::<IS_SF_INFO>() == 4);
const _: () = assert!(core::mem::size_of::<SAMPLE>() == 4);

