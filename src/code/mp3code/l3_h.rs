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

use core::ffi::c_uchar;

pub const GLOBAL_GAIN_SCALE: i32 = 4 * 15;
/* #define GLOBAL_GAIN_SCALE 0 */

/*-----------------------------------------------------------*/
/*---- huffman lookup tables ---*/
/* endian dependent !!! */

#[cfg(target_endian = "little")]
#[repr(C)]
pub union HUFF_ELEMENT {
    pub ptr: i32,
    pub b: HUFF_ELEMENT_b,
}

#[cfg(target_endian = "little")]
#[repr(C)]
pub struct HUFF_ELEMENT_b {
    pub signbits: c_uchar,
    pub x: c_uchar,
    pub y: c_uchar,
    pub purgebits: c_uchar, // 0 = esc
}

#[cfg(target_endian = "big")]
#[repr(C)]
pub union HUFF_ELEMENT {
    pub ptr: i32, /* int must be 32 bits or more */
    pub b: HUFF_ELEMENT_b,
}

#[cfg(target_endian = "big")]
#[repr(C)]
pub struct HUFF_ELEMENT_b {
    pub purgebits: c_uchar, // 0 = esc
    pub y: c_uchar,
    pub x: c_uchar,
    pub signbits: c_uchar,
}

/*--------------------------------------------------------------*/
#[repr(C)]
pub struct BITDAT {
    pub bitbuf: u32,
    pub bits: i32,
    pub bs_ptr: *mut c_uchar,
    pub bs_ptr0: *mut c_uchar,
    pub bs_ptr_end: *mut c_uchar, // optional for overrun test
}

/*-- side info ---*/
#[repr(C)]
pub struct GR {
    pub part2_3_length: i32,
    pub big_values: i32,
    pub global_gain: i32,
    pub scalefac_compress: i32,
    pub window_switching_flag: i32,
    pub block_type: i32,
    pub mixed_block_flag: i32,
    pub table_select: [i32; 3],
    pub subblock_gain: [i32; 3],
    pub region0_count: i32,
    pub region1_count: i32,
    pub preflag: i32,
    pub scalefac_scale: i32,
    pub count1table_select: i32,
}

#[repr(C)]
pub struct SIDE_INFO {
    pub mode: i32,
    pub mode_ext: i32,
    /*---------------*/
    pub main_data_begin: i32, /* beginning, not end, my spec wrong */
    pub private_bits: i32,
    /*---------------*/
    pub scfsi: [i32; 2], /* 4 bit flags [ch] */
    pub gr: [[GR; 2]; 2], /* [gran][ch] */
}

/*-----------------------------------------------------------*/
/*-- scale factors ---*/
// check dimensions - need 21 long, 3*12 short
// plus extra for implicit sf=0 above highest cb
#[repr(C)]
pub struct SCALEFACT {
    pub l: [i32; 23], /* [cb] */
    pub s: [[i32; 13]; 3], /* [window][cb] */
}

/*-----------------------------------------------------------*/
#[repr(C)]
pub struct CB_INFO {
    pub cbtype: i32, /* long=0 short=1 */
    pub cbmax: i32, /* max crit band */
    //   int lb_type;			/* long block type 0 1 3 */
    pub cbs0: i32, /* short band start index 0 3 12 (12=no shorts */
    pub ncbl: i32, /* number long cb's 0 8 21 */
    pub cbmax_s: [i32; 3], /* cbmax by individual short blocks */
}

/*-----------------------------------------------------------*/
/* scale factor infor for MPEG2 intensity stereo  */
#[repr(C)]
pub struct IS_SF_INFO {
    pub nr: [i32; 3],
    pub slen: [i32; 3],
    pub intensity_scale: i32,
}
