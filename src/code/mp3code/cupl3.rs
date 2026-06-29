/*____________________________________________________________________________

	FreeAmp - The Free MP3 Player

        MP3 Decoder originally Copyright (C) 1995-1997 Xing Technology
        Corp.  http://www.xingtech.com

	Portions Copyright (C) 1998-1999 EMusic.com

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

	$Id: cupl3.c,v 1.8 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  cupL3.c  ***************************************************
unpack Layer III


mod 8/18/97  bugfix crc problem

mod 10/9/97  add pMP3Stream->band_limit12 for short blocks

mod 10/22/97  zero buf_ptrs in init

mod 5/15/98 mpeg 2.5

mod 8/19/98 decode 22 sf bands

******************************************************************/

/*---------------------------------------
TO DO: Test mixed blocks (mixed long/short)
  No mixed blocks in mpeg-1 test stream being used for development

-----------------------------------------*/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::mem;

// extern declarations for types and functions from other modules
// (These would be declared in the actual headers)

#[repr(C)]
pub struct MPEG_HEAD {
    // Structure fields would be defined based on mhead.h
    // Placeholder for now
    pub option: c_int,
    pub id: c_int,
    pub sr_index: c_int,
    pub sync: c_int,
    pub mode: c_int,
}

#[repr(C)]
pub struct SIDE_INFO {
    pub gr: [[GR; 2]; 2],
    pub mode: c_int,
    pub mode_ext: c_int,
    pub main_data_begin: c_int,
    pub private_bits: c_int,
    pub scfsi: [c_int; 2],
}

#[repr(C)]
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
pub struct SCALEFACT {
    // Placeholder structure
    pub data: [c_int; 40],
}

#[repr(C)]
pub struct CB_INFO {
    pub ncbl: c_int,
    pub cbtype: c_int,
    pub cbmax: c_int,
}

#[repr(C)]
pub struct IS_SF_INFO {
    // Placeholder structure
    pub data: [c_int; 40],
}

#[repr(C)]
pub struct DEC_INFO {
    pub channels: c_int,
    pub outvalues: c_int,
    pub samprate: c_int,
    pub bits: c_int,
    pub framebytes: c_int,
    pub type_: c_int,
}

#[repr(C)]
pub struct SAMPLE {
    pub x: f32,
    pub s: c_int,
}

#[repr(C)]
pub struct IN_OUT {
    pub in_bytes: c_int,
    pub out_bytes: c_int,
}

#[repr(C)]
pub struct BITDAT {
    pub bs_ptr0: *mut u8,
    pub bs_ptr: *mut u8,
    pub bs_ptr_end: *mut u8,
    pub bits: c_int,
    pub bitbuf: c_int,
}

#[repr(C)]
pub struct MP3_STREAM {
    // Placeholder for pMP3Stream structure
    // All the fields accessed in the code would be defined here
}

type SBT_FUNCTION = unsafe extern "C" fn(*mut f32, *mut i16, c_int);
type XFORM_FUNCTION = unsafe extern "C" fn(*mut c_void, c_int);
type DECODE_FUNCTION = unsafe extern "C" fn(*mut u8, *mut u8) -> IN_OUT;

/*====================================================================*/
static mp_sr20_table: [[c_int; 4]; 2] = [
    [441, 480, 320, -999],
    [882, 960, 640, -999],
];

static mp_br_tableL3: [[c_int; 16]; 2] = [
    [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],	/* mpeg 2 */
    [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0],
];

/*====================================================================*/

/*-- global band tables */
/*-- short portion is 3*x !! --*/
////@@@@int nBand[2][22];		/* [long/short][cb] */
////@@@@int sfBandIndex[2][22];		/* [long/short][cb] */

/*====================================================================*/

/*----------------*/
extern "C" {
    pub static mut decinfo: DEC_INFO;	////@@@@ this is ok, only written to during init, then chucked.
}

/*----------------*/
////@@@@static int pMP3Stream->mpeg25_flag;		// L3 only

//int iframe;

/*-------*/
////@@@@static int pMP3Stream->band_limit = (576);		// L3 only
////@@@@static int pMP3Stream->band_limit21 = (576);	// limit for sf band 21	// L3 only
////@@@@static int pMP3Stream->band_limit12 = (576);	// limit for sf band 12 short	//L3 only

////@@@@int band_limit_nsb = 32;	/* global for hybrid */
////@@@@static int pMP3Stream->nsb_limit = 32;
////@@@@static int pMP3Stream->gain_adjust = 0;	/* adjust gain e.g. cvt to mono */	// L3 only
////@@@@static int id; // L3 only
////@@@@static int pMP3Stream->ncbl_mixed;		/* number of long cb's in mixed block 8 or 6 */		// L3 only
////@@@@static int pMP3Stream->sr_index;	// L3 only (99%)

//@@@@
////@@@@static int pMP3Stream->outvalues;		//
////@@@@static int pMP3Stream->outbytes;		//
////@@@@static int pMP3Stream->half_outbytes;	// L3 only
////@@@@static int pMP3Stream->framebytes;		//

//static int padframebytes;
////@@@@static int pMP3Stream->crcbytes;		// L3 only
////@@@@static int pMP3Stream->pad;				//
//static int stereo_flag;	// only written to
////@@@@static int pMP3Stream->nchan;			// L3 only
////@@@@static int pMP3Stream->ms_mode;			// L3 only (99%)
////@@@@static int pMP3Stream->is_mode;			// L3 only
////@@@@static unsigned int pMP3Stream->zero_level_pcm = 0;	// L3 only

/* cb_info[igr][ch], compute by dequant, used by joint */
static mut cb_info: [[CB_INFO; 2]; 2] = [
    [CB_INFO { ncbl: 0, cbtype: 0, cbmax: 0 }, CB_INFO { ncbl: 0, cbtype: 0, cbmax: 0 }],
    [CB_INFO { ncbl: 0, cbtype: 0, cbmax: 0 }, CB_INFO { ncbl: 0, cbtype: 0, cbmax: 0 }],
];	// L3 only	############ I think this is ok, only a scratchpad?
static mut is_sf_info: IS_SF_INFO = IS_SF_INFO { data: [0; 40] };	/* MPEG-2 intensity stereo */	// L3 only	 ############## scratchpad?

/*---------------------------------*/
//!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
/* main data bit buffer */
/*@@@@
#define NBUF (8*1024)
#define BUF_TRIGGER (NBUF-1500)
static unsigned char buf[NBUF];
static int buf_ptr0 = 0;	// !!!!!!!!!!!
static int buf_ptr1 = 0;	// !!!!!!!!!!!
static int main_pos_bit;
*/
/*---------------------------------*/
static mut side_info: SIDE_INFO = SIDE_INFO {
    gr: [
        [GR { part2_3_length: 0, big_values: 0, global_gain: 0, scalefac_compress: 0,
              window_switching_flag: 0, block_type: 0, mixed_block_flag: 0, table_select: [0; 3],
              subblock_gain: [0; 3], region0_count: 0, region1_count: 0, preflag: 0,
              scalefac_scale: 0, count1table_select: 0 },
         GR { part2_3_length: 0, big_values: 0, global_gain: 0, scalefac_compress: 0,
              window_switching_flag: 0, block_type: 0, mixed_block_flag: 0, table_select: [0; 3],
              subblock_gain: [0; 3], region0_count: 0, region1_count: 0, preflag: 0,
              scalefac_scale: 0, count1table_select: 0 }],
        [GR { part2_3_length: 0, big_values: 0, global_gain: 0, scalefac_compress: 0,
              window_switching_flag: 0, block_type: 0, mixed_block_flag: 0, table_select: [0; 3],
              subblock_gain: [0; 3], region0_count: 0, region1_count: 0, preflag: 0,
              scalefac_scale: 0, count1table_select: 0 },
         GR { part2_3_length: 0, big_values: 0, global_gain: 0, scalefac_compress: 0,
              window_switching_flag: 0, block_type: 0, mixed_block_flag: 0, table_select: [0; 3],
              subblock_gain: [0; 3], region0_count: 0, region1_count: 0, preflag: 0,
              scalefac_scale: 0, count1table_select: 0 }],
    ],
    mode: 0,
    mode_ext: 0,
    main_data_begin: 0,
    private_bits: 0,
    scfsi: [0; 2],
};	// ####### scratchpad?

static mut sf: [[SCALEFACT; 2]; 2] = [
    [SCALEFACT { data: [0; 40] }, SCALEFACT { data: [0; 40] }],
    [SCALEFACT { data: [0; 40] }, SCALEFACT { data: [0; 40] }],
];	/* [gr][ch] */	 // ########## scratchpad?

static mut nsamp: [[c_int; 2]; 2] = [[0, 0], [0, 0]];	/* must start = 0, for nsamp[igr_prev] */	// ########## scratchpad?

/*- sample union of int/float  sample[ch][gr][576] */
/* static SAMPLE sample[2][2][576]; */
// @@@@FINDME
////@@@@extern SAMPLE sample[2][2][576];						////////////????? suspicious, mainly used in decode loop, but zeroed init code
static mut yout: [f32; 576] = [0.0; 576];	/* hybrid out, sbt in */	//////////// scratchpad

////@@@@typedef void (*SBT_FUNCTION) (float *sample, short *pcm, int ch);
extern "C" {
    pub fn sbt_dual_L3(sample: *mut f32, pcm: *mut i16, n: c_int);
}
////@@@@static SBT_FUNCTION sbt_L3 = sbt_dual_L3;	// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

////@@@@typedef void (*XFORM_FUNCTION) (void *pcm, int igr);
// Note: Xform_dual is defined below as a static function
////@@@@static XFORM_FUNCTION Xform = Xform_dual;	// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

// Note: L3audio_decode_MPEG1 and L3audio_decode_MPEG2 are defined below
////@@@@typedef IN_OUT(*DECODE_FUNCTION) (unsigned char *bs, unsigned char *pcm);
////@@@@static DECODE_FUNCTION decode_function = L3audio_decode_MPEG1;		<------------------ needs streaming, ditto above!!!


/*====================================================================*/
extern "C" {
    pub fn hybrid(xin: *mut c_void, xprev: *mut c_void, y: *mut f32,
        btype: c_int, nlong: c_int, ntot: c_int, nprev: c_int) -> c_int;
    pub fn hybrid_sum(xin: *mut c_void, xin_left: *mut c_void, y: *mut f32,
        btype: c_int, nlong: c_int, ntot: c_int) -> c_int;
    pub fn sum_f_bands(a: *mut c_void, b: *mut c_void, n: c_int);
    pub fn FreqInvert(y: *mut f32, n: c_int);
    pub fn antialias(x: *mut c_void, n: c_int);
    pub fn ms_process(x: *mut c_void, n: c_int);	/* sum-difference stereo */
    pub fn is_process_MPEG1(x: *mut c_void,	/* intensity stereo */
        sf: *mut SCALEFACT,
        cb_info: *mut [CB_INFO; 2],	/* [ch] */
        nsamp: c_int, ms_mode: c_int);
    pub fn is_process_MPEG2(x: *mut c_void,	/* intensity stereo */
        sf: *mut SCALEFACT,
        cb_info: *mut [CB_INFO; 2],	/* [ch] */
        is_sf_info: *mut IS_SF_INFO,
        nsamp: c_int, ms_mode: c_int);

    pub fn unpack_huff(xy: *mut c_void, n: c_int, ntable: c_int);
    pub fn unpack_huff_quad(vwxy: *mut c_void, n: c_int, nbits: c_int, ntable: c_int) -> c_int;
    pub fn dequant(sample: *mut SAMPLE, nsamp: *mut c_int,
        sf: *mut SCALEFACT,
        gr: *mut GR,
        cb_info: *mut CB_INFO, ncbl_mixed: c_int);
    pub fn unpack_sf_sub_MPEG1(scalefac: *mut SCALEFACT, gr: *mut GR,
        scfsi: c_int,	/* bit flag */
        igr: c_int);
    pub fn unpack_sf_sub_MPEG2(sf: *mut SCALEFACT,	/* return intensity scale */
        grdat: *mut GR,
        is_and_ch: c_int, is_sf_info: *mut IS_SF_INFO);
}

/*====================================================================*/
/* get bits from bitstream in endian independent way */

static mut bitdat: BITDAT = BITDAT {
    bs_ptr0: core::ptr::null_mut(),
    bs_ptr: core::ptr::null_mut(),
    bs_ptr_end: core::ptr::null_mut(),
    bits: 0,
    bitbuf: 0,
};			/* global for inline use by Huff */		// !!!!!!!!!!!!!!!!!!!

/*------------- initialize bit getter -------------*/
unsafe fn bitget_init(buf: *mut u8) {
    bitdat.bs_ptr0 = buf;
    bitdat.bs_ptr = buf;
    bitdat.bits = 0;
    bitdat.bitbuf = 0;
}
/*------------- initialize bit getter -------------*/
unsafe fn bitget_init_end(buf_end: *mut u8) {
    bitdat.bs_ptr_end = buf_end;
}
/*------------- get n bits from bitstream -------------*/
unsafe fn bitget_bits_used() -> c_int {
    let n: c_int;			/* compute bits used from last init call */

    n = (((bitdat.bs_ptr as usize - bitdat.bs_ptr0 as usize) << 3) as c_int) - bitdat.bits;
    return n;
}
/*------------- check for n bits in bitbuf -------------*/
unsafe fn bitget_check(n: c_int) {
    if bitdat.bits < n {
        while bitdat.bits <= 24 {
            bitdat.bitbuf = (bitdat.bitbuf << 8) | (*bitdat.bs_ptr as c_int);
            bitdat.bs_ptr = bitdat.bs_ptr.add(1);
            bitdat.bits += 8;
        }
    }
}
/*------------- get n bits from bitstream -------------*/
unsafe fn bitget(n: c_int) -> c_int {
    let x: c_int;

    if bitdat.bits < n {
        /* refill bit buf if necessary */
        while bitdat.bits <= 24 {
            bitdat.bitbuf = (bitdat.bitbuf << 8) | (*bitdat.bs_ptr as c_int);
            bitdat.bs_ptr = bitdat.bs_ptr.add(1);
            bitdat.bits += 8;
        }
    }
    bitdat.bits -= n;
    x = bitdat.bitbuf >> bitdat.bits;
    bitdat.bitbuf -= x << bitdat.bits;
    return x;
}
/*------------- get 1 bit from bitstream -------------*/
unsafe fn bitget_1bit() -> c_int {
    let x: c_int;

    if bitdat.bits <= 0 {
        /* refill bit buf if necessary */
        while bitdat.bits <= 24 {
            bitdat.bitbuf = (bitdat.bitbuf << 8) | (*bitdat.bs_ptr as c_int);
            bitdat.bs_ptr = bitdat.bs_ptr.add(1);
            bitdat.bits += 8;
        }
    }
    bitdat.bits -= 1;
    x = bitdat.bitbuf >> bitdat.bits;
    bitdat.bitbuf -= x << bitdat.bits;
    return x;
}

// Global pMP3Stream pointer placeholder
extern "C" {
    pub static mut pMP3Stream: *mut MP3_STREAM;
}

/*====================================================================*/
unsafe fn Xform_mono(pcm: *mut c_void, igr: c_int) {
    let igr_prev: c_int;
    let n1: c_int;
    let n2: c_int;

/*--- hybrid + sbt ---*/
    // Note: Access to pMP3Stream fields would go through pointer dereference
    // This is a placeholder; actual implementation requires proper struct definition
}

/*--------------------------------------------------------------------*/
unsafe fn Xform_dual_right(pcm: *mut c_void, igr: c_int) {
    let igr_prev: c_int;
    let n1: c_int;
    let n2: c_int;

/*--- hybrid + sbt ---*/
}

/*--------------------------------------------------------------------*/
unsafe fn Xform_dual(pcm: *mut c_void, igr: c_int) {
    let ch: c_int;
    let igr_prev: c_int;
    let n1: c_int;
    let n2: c_int;

/*--- hybrid + sbt ---*/
    igr_prev = igr ^ 1;
}

/*--------------------------------------------------------------------*/
unsafe fn Xform_dual_mono(pcm: *mut c_void, igr: c_int) {
    let igr_prev: c_int;
    let n1: c_int;
    let n2: c_int;
    let n3: c_int;

/*--- hybrid + sbt ---*/
    igr_prev = igr ^ 1;
}

/*--------------------------------------------------------------------*/
/*====================================================================*/
unsafe fn unpack_side_MPEG1() -> c_int {
    let prot: c_int;
    let br_index: c_int;
    let mut igr: c_int;
    let mut ch: c_int;
    let side_bytes: c_int;

/* decode partial header plus initial side info */
/* at entry bit getter points at id, sync skipped by caller */

    // pMP3Stream->id = bitget(1);		/* id */
    // bitget(2);			/* skip layer */
    // prot = bitget(1);		/* bitget prot bit */
    // br_index = bitget(4);
    // pMP3Stream->sr_index = bitget(2);
    // pMP3Stream->pad = bitget(1);
    // bitget(1);			/* skip to mode */
    // side_info.mode = bitget(2);	/* mode */
    // side_info.mode_ext = bitget(2);	/* mode ext */

    // if (side_info.mode != 1)
    //    side_info.mode_ext = 0;

    // /* adjust global gain in ms mode to avoid having to mult by 1/sqrt(2) */
    // pMP3Stream->ms_mode = side_info.mode_ext >> 1;
    // pMP3Stream->is_mode = side_info.mode_ext & 1;

    // pMP3Stream->crcbytes = 0;
    // if (prot)
    //    bitget(4);		/* skip to data */
    // else
    // {
    //    bitget(20);		/* skip crc */
    //    pMP3Stream->crcbytes = 2;
    // }

    // if (br_index > 0)		/* pMP3Stream->framebytes fixed for free format */
    // {
    //   pMP3Stream->framebytes =
    //      2880 * mp_br_tableL3[pMP3Stream->id][br_index] / mp_sr20_table[pMP3Stream->id][pMP3Stream->sr_index];
    // }

    // side_info.main_data_begin = bitget(9);
    // if (side_info.mode == 3)
    // {
    //    side_info.private_bits = bitget(5);
    //    pMP3Stream->nchan = 1;
    // //      stereo_flag = 0;
    //    side_bytes = (4 + 17);
    // /*-- with header --*/
    // }
    // else
    // {
    //    side_info.private_bits = bitget(3);
    //    pMP3Stream->nchan = 2;
    // //      stereo_flag = 1;
    //    side_bytes = (4 + 32);
    // /*-- with header --*/
    // }
    // for (ch = 0; ch < pMP3Stream->nchan; ch++)
    //    side_info.scfsi[ch] = bitget(4);
    // /* this always 0 (both igr) for short blocks */

    // for (igr = 0; igr < 2; igr++)
    // {
    //    for (ch = 0; ch < pMP3Stream->nchan; ch++)
    //    {
    //       side_info.gr[igr][ch].part2_3_length = bitget(12);
    //       side_info.gr[igr][ch].big_values = bitget(9);
    //       side_info.gr[igr][ch].global_gain = bitget(8) + pMP3Stream->gain_adjust;
    //       if (pMP3Stream->ms_mode)
    //          side_info.gr[igr][ch].global_gain -= 2;
    //       side_info.gr[igr][ch].scalefac_compress = bitget(4);
    //       side_info.gr[igr][ch].window_switching_flag = bitget(1);
    //       if (side_info.gr[igr][ch].window_switching_flag)
    //       {
    //          side_info.gr[igr][ch].block_type = bitget(2);
    //          side_info.gr[igr][ch].mixed_block_flag = bitget(1);
    //          side_info.gr[igr][ch].table_select[0] = bitget(5);
    //          side_info.gr[igr][ch].table_select[1] = bitget(5);
    //          side_info.gr[igr][ch].subblock_gain[0] = bitget(3);
    //          side_info.gr[igr][ch].subblock_gain[1] = bitget(3);
    //          side_info.gr[igr][ch].subblock_gain[2] = bitget(3);
    //        /* region count set in terms of long block cb's/bands */
    //        /* r1 set so r0+r1+1 = 21 (lookup produces 576 bands ) */
    //        /* if(window_switching_flag) always 36 samples in region0 */
    //          side_info.gr[igr][ch].region0_count = (8 - 1);	/* 36 samples */
    //          side_info.gr[igr][ch].region1_count = 20 - (8 - 1);
    //       }
    //       else
    //       {
    //          side_info.gr[igr][ch].mixed_block_flag = 0;
    //          side_info.gr[igr][ch].block_type = 0;
    //          side_info.gr[igr][ch].table_select[0] = bitget(5);
    //          side_info.gr[igr][ch].table_select[1] = bitget(5);
    //          side_info.gr[igr][ch].table_select[2] = bitget(5);
    //          side_info.gr[igr][ch].region0_count = bitget(4);
    //          side_info.gr[igr][ch].region1_count = bitget(3);
    //       }
    //       side_info.gr[igr][ch].preflag = bitget(1);
    //       side_info.gr[igr][ch].scalefac_scale = bitget(1);
    //       side_info.gr[igr][ch].count1table_select = bitget(1);
    //    }
    // }

    // /* return  bytes in header + side info */
    return 0;
}

/*====================================================================*/
unsafe fn unpack_side_MPEG2(igr: c_int) -> c_int {
    let prot: c_int;
    let br_index: c_int;
    let mut ch: c_int;
    let side_bytes: c_int;

/* decode partial header plus initial side info */
/* at entry bit getter points at id, sync skipped by caller */

    // pMP3Stream->id = bitget(1);		/* id */
    // bitget(2);			/* skip layer */
    // prot = bitget(1);		/* bitget prot bit */
    // br_index = bitget(4);
    // pMP3Stream->sr_index = bitget(2);
    // pMP3Stream->pad = bitget(1);
    // bitget(1);			/* skip to mode */
    // side_info.mode = bitget(2);	/* mode */
    // side_info.mode_ext = bitget(2);	/* mode ext */

    // if (side_info.mode != 1)
    //    side_info.mode_ext = 0;

    // /* adjust global gain in ms mode to avoid having to mult by 1/sqrt(2) */
    // pMP3Stream->ms_mode = side_info.mode_ext >> 1;
    // pMP3Stream->is_mode = side_info.mode_ext & 1;

    // pMP3Stream->crcbytes = 0;
    // if (prot)
    //    bitget(4);		/* skip to data */
    // else
    // {
    //    bitget(20);		/* skip crc */
    //    pMP3Stream->crcbytes = 2;
    // }

    // if (br_index > 0)
    // {				/* pMP3Stream->framebytes fixed for free format */
    //    if (pMP3Stream->mpeg25_flag == 0)
    //    {
    //       pMP3Stream->framebytes =
    //          1440 * mp_br_tableL3[pMP3Stream->id][br_index] / mp_sr20_table[pMP3Stream->id][pMP3Stream->sr_index];
    //    }
    //    else
    //    {
    //       pMP3Stream->framebytes =
    //          2880 * mp_br_tableL3[pMP3Stream->id][br_index] / mp_sr20_table[pMP3Stream->id][pMP3Stream->sr_index];
    //     //if( pMP3Stream->sr_index == 2 ) return 0;  // fail mpeg25 8khz
    //    }
    // }
    // side_info.main_data_begin = bitget(8);
    // if (side_info.mode == 3)
    // {
    //    side_info.private_bits = bitget(1);
    //    pMP3Stream->nchan = 1;
    // //      stereo_flag = 0;
    //    side_bytes = (4 + 9);
    // /*-- with header --*/
    // }
    // else
    // {
    //    side_info.private_bits = bitget(2);
    //    pMP3Stream->nchan = 2;
    // //      stereo_flag = 1;
    //    side_bytes = (4 + 17);
    // /*-- with header --*/
    // }
    // side_info.scfsi[1] = side_info.scfsi[0] = 0;


    // for (ch = 0; ch < pMP3Stream->nchan; ch++)
    // {
    //    side_info.gr[igr][ch].part2_3_length = bitget(12);
    //    side_info.gr[igr][ch].big_values = bitget(9);
    //    side_info.gr[igr][ch].global_gain = bitget(8) + pMP3Stream->gain_adjust;
    //    if (pMP3Stream->ms_mode)
    //       side_info.gr[igr][ch].global_gain -= 2;
    //    side_info.gr[igr][ch].scalefac_compress = bitget(9);
    //    side_info.gr[igr][ch].window_switching_flag = bitget(1);
    //    if (side_info.gr[igr][ch].window_switching_flag)
    //    {
    //       side_info.gr[igr][ch].block_type = bitget(2);
    //       side_info.gr[igr][ch].mixed_block_flag = bitget(1);
    //       side_info.gr[igr][ch].table_select[0] = bitget(5);
    //       side_info.gr[igr][ch].table_select[1] = bitget(5);
    //       side_info.gr[igr][ch].subblock_gain[0] = bitget(3);
    //       side_info.gr[igr][ch].subblock_gain[1] = bitget(3);
    //       side_info.gr[igr][ch].subblock_gain[2] = bitget(3);
    //     /* region count set in terms of long block cb's/bands  */
    //     /* r1 set so r0+r1+1 = 21 (lookup produces 576 bands ) */
    //     /* bt=1 or 3       54 samples */
    //     /* bt=2 mixed=0    36 samples */
    //     /* bt=2 mixed=1    54 (8 long sf) samples? or maybe 36 */
    //     /* region0 discussion says 54 but this would mix long */
    //     /* and short in region0 if scale factors switch */
    //     /* at band 36 (6 long scale factors) */
    //       if ((side_info.gr[igr][ch].block_type == 2))
    //       {
    //          side_info.gr[igr][ch].region0_count = (6 - 1);	/* 36 samples */
    //          side_info.gr[igr][ch].region1_count = 20 - (6 - 1);
    //       }
    //       else
    //       {			/* long block type 1 or 3 */
    //          side_info.gr[igr][ch].region0_count = (8 - 1);	/* 54 samples */
    //          side_info.gr[igr][ch].region1_count = 20 - (8 - 1);
    //       }
    //    }
    //    else
    //    {
    //       side_info.gr[igr][ch].mixed_block_flag = 0;
    //       side_info.gr[igr][ch].block_type = 0;
    //       side_info.gr[igr][ch].table_select[0] = bitget(5);
    //       side_info.gr[igr][ch].table_select[1] = bitget(5);
    //       side_info.gr[igr][ch].table_select[2] = bitget(5);
    //       side_info.gr[igr][ch].region0_count = bitget(4);
    //       side_info.gr[igr][ch].region1_count = bitget(3);
    //    }
    //    side_info.gr[igr][ch].preflag = 0;
    //    side_info.gr[igr][ch].scalefac_scale = bitget(1);
    //    side_info.gr[igr][ch].count1table_select = bitget(1);
    // }

    // /* return  bytes in header + side info */
    return 0;
}

/*-----------------------------------------------------------------*/
unsafe fn unpack_main(pcm: *mut u8, igr: c_int) {
    let mut ch: c_int;
    let bit0: c_int;
    let mut n1: c_int;
    let mut n2: c_int;
    let mut n3: c_int;
    let mut n4: c_int;
    let nn2: c_int;
    let nn3: c_int;
    let nn4: c_int;
    let qbits: c_int;
    let m0: c_int;

    // for (ch = 0; ch < pMP3Stream->nchan; ch++)
    // {
    //    bitget_init(pMP3Stream->buf + (pMP3Stream->main_pos_bit >> 3));
    //    bit0 = (pMP3Stream->main_pos_bit & 7);
    //    if (bit0)
    //       bitget(bit0);
    //    pMP3Stream->main_pos_bit += side_info.gr[igr][ch].part2_3_length;
    //    bitget_init_end(pMP3Stream->buf + ((pMP3Stream->main_pos_bit + 39) >> 3));
    // /*-- scale factors --*/
    //    if (pMP3Stream->id)
    //       unpack_sf_sub_MPEG1(&sf[igr][ch],
    //                        &side_info.gr[igr][ch], side_info.scfsi[ch], igr);
    //    else
    //       unpack_sf_sub_MPEG2(&sf[igr][ch],
    //                   &side_info.gr[igr][ch], pMP3Stream->is_mode & ch, &is_sf_info);
    // /*--- huff data ---*/
    //    n1 = pMP3Stream->sfBandIndex[0][side_info.gr[igr][ch].region0_count];
    //    n2 = pMP3Stream->sfBandIndex[0][side_info.gr[igr][ch].region0_count
    //                     + side_info.gr[igr][ch].region1_count + 1];
    //    n3 = side_info.gr[igr][ch].big_values;
    //    n3 = n3 + n3;

    //    if (n3 > pMP3Stream->band_limit)
    //       n3 = pMP3Stream->band_limit;
    //    if (n2 > n3)
    //       n2 = n3;
    //    if (n1 > n3)
    //       n1 = n3;
    //    nn3 = n3 - n2;
    //    nn2 = n2 - n1;
    //    unpack_huff(pMP3Stream->sample[ch][igr], n1, side_info.gr[igr][ch].table_select[0]);
    //    unpack_huff(pMP3Stream->sample[ch][igr] + n1, nn2, side_info.gr[igr][ch].table_select[1]);
    //    unpack_huff(pMP3Stream->sample[ch][igr] + n2, nn3, side_info.gr[igr][ch].table_select[2]);
    //    qbits = side_info.gr[igr][ch].part2_3_length - (bitget_bits_used() - bit0);
    //    nn4 = unpack_huff_quad(pMP3Stream->sample[ch][igr] + n3, pMP3Stream->band_limit - n3, qbits,
    //                           side_info.gr[igr][ch].count1table_select);
    //    n4 = n3 + nn4;
    //    nsamp[igr][ch] = n4;
    //  //limit n4 or allow deqaunt to sf band 22
    //    if (side_info.gr[igr][ch].block_type == 2)
    //       n4 = min(n4, pMP3Stream->band_limit12);
    //    else
    //       n4 = min(n4, pMP3Stream->band_limit21);
    //    if (n4 < 576)
    //       memset(pMP3Stream->sample[ch][igr] + n4, 0, sizeof(SAMPLE) * (576 - n4));
    //    if (bitdat.bs_ptr > bitdat.bs_ptr_end)
    //    {				// bad data overrun
    //       memset(pMP3Stream->sample[ch][igr], 0, sizeof(SAMPLE) * (576));
    //    }
    // }

    // /*--- dequant ---*/
    // for (ch = 0; ch < pMP3Stream->nchan; ch++)
    // {
    //    dequant(pMP3Stream->sample[ch][igr],
    //            &nsamp[igr][ch],	/* nsamp updated for shorts */
    //            &sf[igr][ch], &side_info.gr[igr][ch],
    //            &cb_info[igr][ch], pMP3Stream->ncbl_mixed);
    // }

    // /*--- ms stereo processing  ---*/
    // if (pMP3Stream->ms_mode)
    // {
    //    if (pMP3Stream->is_mode == 0)
    //    {
    //       m0 = nsamp[igr][0];	/* process to longer of left/right */
    //       if (m0 < nsamp[igr][1])
    //          m0 = nsamp[igr][1];
    //    }
    //    else
    //    {				/* process to last cb in right */
    //       m0 = pMP3Stream->sfBandIndex[cb_info[igr][1].cbtype][cb_info[igr][1].cbmax];
    //    }
    //    ms_process(pMP3Stream->sample[0][igr], m0);
    // }

    // /*--- is stereo processing  ---*/
    // if (pMP3Stream->is_mode)
    // {
    //    if (pMP3Stream->id)
    //       is_process_MPEG1(pMP3Stream->sample[0][igr], &sf[igr][1],
    //                      cb_info[igr], nsamp[igr][0], pMP3Stream->ms_mode);
    //    else
    //       is_process_MPEG2(pMP3Stream->sample[0][igr], &sf[igr][1],
    //                      cb_info[igr], &is_sf_info,
    //                      nsamp[igr][0], pMP3Stream->ms_mode);
    // }

    // /*-- adjust ms and is modes to max of left/right */
    // if (side_info.mode_ext)
    // {
    //    if (nsamp[igr][0] < nsamp[igr][1])
    //       nsamp[igr][0] = nsamp[igr][1];
    //    else
    //       nsamp[igr][1] = nsamp[igr][0];
    // }

    // /*--- antialias ---*/
    // for (ch = 0; ch < pMP3Stream->nchan; ch++)
    // {
    //    if (cb_info[igr][ch].ncbl == 0)
    //       continue;		/* have no long blocks */
    //    if (side_info.gr[igr][ch].mixed_block_flag)
    //       n1 = 1;		/* 1 -> 36 samples */
    //    else
    //       n1 = (nsamp[igr][ch] + 7) / 18;
    //    if (n1 > 31)
    //       n1 = 31;
    //    antialias(pMP3Stream->sample[ch][igr], n1);
    //    n1 = 18 * n1 + 8;		/* update number of samples */
    //    if (n1 > nsamp[igr][ch])
    //       nsamp[igr][ch] = n1;
    // }

    // /*--- hybrid + sbt ---*/
    // pMP3Stream->Xform(pcm, igr);

    // /*-- done --*/
}

/*--------------------------------------------------------------------*/
/*-----------------------------------------------------------------*/
pub unsafe extern "C" fn L3audio_decode(bs: *mut u8, pcm: *mut u8) -> IN_OUT {
    // return pMP3Stream->decode_function(bs, pcm);
    IN_OUT { in_bytes: 0, out_bytes: 0 }
}

/*--------------------------------------------------------------------*/
extern "C" {
    pub static mut gpNextByteAfterData: *mut u8;
}

pub unsafe extern "C" fn L3audio_decode_MPEG1(bs: *mut u8, pcm: *mut u8) -> IN_OUT {
    let sync: c_int;
    let mut in_out: IN_OUT;
    let side_bytes: c_int;
    let nbytes: c_int;

    let padframebytes: c_int;	////@@@@

//   iframe++;

    bitget_init(bs);		/* initialize bit getter */
/* test sync */
    in_out.in_bytes = 0;		/* assume fail */
    in_out.out_bytes = 0;
    // sync = bitget(12);

    // if (sync != 0xFFF)
    //    return in_out;		/* sync fail */
    // /*-----------*/

    // /*-- unpack side info --*/
    // side_bytes = unpack_side_MPEG1();
    // padframebytes = pMP3Stream->framebytes + pMP3Stream->pad;

    // if (bs + padframebytes > gpNextByteAfterData)
    //    return in_out;	// error check if we're about to read off the end of the legal memory (caused by certain MP3 writers' goofy comment formats) -ste.
    // in_out.in_bytes = padframebytes;

    // /*-- load main data and update buf pointer --*/
    // /*-------------------------------------------
    //    if start point < 0, must just cycle decoder
    //    if jumping into middle of stream,
    // w---------------------------------------------*/
    // pMP3Stream->buf_ptr0 = pMP3Stream->buf_ptr1 - side_info.main_data_begin;	/* decode start point */
    // if (pMP3Stream->buf_ptr1 > BUF_TRIGGER)
    // {				/* shift buffer */
    //    memmove(pMP3Stream->buf, pMP3Stream->buf + pMP3Stream->buf_ptr0, side_info.main_data_begin);
    //    pMP3Stream->buf_ptr0 = 0;
    //    pMP3Stream->buf_ptr1 = side_info.main_data_begin;
    // }
    // nbytes = padframebytes - side_bytes - pMP3Stream->crcbytes;

    // // RAK: This is no bueno. :-(
    // if (nbytes < 0 || nbytes > NBUF)
    // {
    //     in_out.in_bytes = 0;
    //      return in_out;
    // }

    // if (bFastEstimateOnly)
    // {
    //    in_out.out_bytes = pMP3Stream->outbytes;
    //    return in_out;
    // }

    // memmove(pMP3Stream->buf + pMP3Stream->buf_ptr1, bs + side_bytes + pMP3Stream->crcbytes, nbytes);
    // pMP3Stream->buf_ptr1 += nbytes;
    // /*-----------------------*/

    // if (pMP3Stream->buf_ptr0 >= 0)
    // {
    // // dump_frame(buf+buf_ptr0, 64);
    //    pMP3Stream->main_pos_bit = pMP3Stream->buf_ptr0 << 3;
    //    unpack_main(pcm, 0);
    //    unpack_main(pcm + pMP3Stream->half_outbytes, 1);
    //    in_out.out_bytes = pMP3Stream->outbytes;
    // }
    // else
    // {
    //    memset(pcm, pMP3Stream->zero_level_pcm, pMP3Stream->outbytes);	/* fill out skipped frames */
    //    in_out.out_bytes = pMP3Stream->outbytes;
    // /* iframe--;  in_out.out_bytes = 0;  // test test */
    // }

    return in_out;
}

/*--------------------------------------------------------------------*/
/*--------------------------------------------------------------------*/
pub unsafe extern "C" fn L3audio_decode_MPEG2(bs: *mut u8, pcm: *mut u8) -> IN_OUT {
    let sync: c_int;
    let mut in_out: IN_OUT;
    let side_bytes: c_int;
    let nbytes: c_int;
    static mut igr: c_int = 0;

    let padframebytes: c_int;	////@@@@

//   iframe++;


    bitget_init(bs);		/* initialize bit getter */
/* test sync */
    in_out.in_bytes = 0;		/* assume fail */
    in_out.out_bytes = 0;
    // sync = bitget(12);

    // // if( sync != 0xFFF ) return in_out;       /* sync fail */

    // pMP3Stream->mpeg25_flag = 0;
    // if (sync != 0xFFF)
    // {
    //    pMP3Stream->mpeg25_flag = 1;		/* mpeg 2.5 sync */
    //    if (sync != 0xFFE)
    //       return in_out;		/* sync fail */
    // }
    // /*-----------*/

    // /*-- unpack side info --*/
    // side_bytes = unpack_side_MPEG2(igr);
    // padframebytes = pMP3Stream->framebytes + pMP3Stream->pad;
    // in_out.in_bytes = padframebytes;

    // pMP3Stream->buf_ptr0 = pMP3Stream->buf_ptr1 - side_info.main_data_begin;	/* decode start point */
    // if (pMP3Stream->buf_ptr1 > BUF_TRIGGER)
    // {				/* shift buffer */
    //    memmove(pMP3Stream->buf, pMP3Stream->buf + pMP3Stream->buf_ptr0, side_info.main_data_begin);
    //    pMP3Stream->buf_ptr0 = 0;
    //    pMP3Stream->buf_ptr1 = side_info.main_data_begin;
    // }
    // nbytes = padframebytes - side_bytes - pMP3Stream->crcbytes;
    // // RAK: This is no bueno. :-(
    // if (nbytes < 0 || nbytes > NBUF)
    // {
    //     in_out.in_bytes = 0;
    //      return in_out;
    // }

    // if (bFastEstimateOnly)
    // {
    //    in_out.out_bytes = pMP3Stream->outbytes;
    //    return in_out;
    // }

    // memmove(pMP3Stream->buf + pMP3Stream->buf_ptr1, bs + side_bytes + pMP3Stream->crcbytes, nbytes);
    // pMP3Stream->buf_ptr1 += nbytes;
    // /*-----------------------*/

    // if (pMP3Stream->buf_ptr0 >= 0)
    // {
    //    pMP3Stream->main_pos_bit = pMP3Stream->buf_ptr0 << 3;
    //    unpack_main(pcm, igr);
    //    in_out.out_bytes = pMP3Stream->outbytes;
    // }
    // else
    // {
    //    memset(pcm, pMP3Stream->zero_level_pcm, pMP3Stream->outbytes);	/* fill out skipped frames */
    //    in_out.out_bytes = pMP3Stream->outbytes;
    // // iframe--;  in_out.out_bytes = 0; return in_out;// test test */
    // }

    // igr = igr ^ 1;
    return in_out;
}

/*--------------------------------------------------------------------*/
/*--------------------------------------------------------------------*/
/*--------------------------------------------------------------------*/
static sr_table: [c_int; 8] =
[22050, 24000, 16000, 1,
 44100, 48000, 32000, 1];

#[repr(C)]
struct SfBandIndexEntry {
    l: [c_int; 23],
    s: [c_int; 14],
}

static sfBandIndexTable: [[[SfBandIndexEntry; 3]; 3]; 3] =
[
/* mpeg-2 */
   [
      [
         SfBandIndexEntry {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
            s: [0, 4, 8, 12, 18, 24, 32, 42, 56, 74, 100, 132, 174, 192],
         },
         SfBandIndexEntry {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 114, 136, 162, 194, 232, 278, 332, 394, 464, 540, 576],
            s: [0, 4, 8, 12, 18, 26, 36, 48, 62, 80, 104, 136, 180, 192],
         },
         SfBandIndexEntry {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
            s: [0, 4, 8, 12, 18, 26, 36, 48, 62, 80, 104, 134, 174, 192],
         },
      ],
/* mpeg-1 */
      [
         SfBandIndexEntry {
            l: [0, 4, 8, 12, 16, 20, 24, 30, 36, 44, 52, 62, 74, 90, 110, 134, 162, 196, 238, 288, 342, 418, 576],
            s: [0, 4, 8, 12, 16, 22, 30, 40, 52, 66, 84, 106, 136, 192],
         },
         SfBandIndexEntry {
            l: [0, 4, 8, 12, 16, 20, 24, 30, 36, 42, 50, 60, 72, 88, 106, 128, 156, 190, 230, 276, 330, 384, 576],
            s: [0, 4, 8, 12, 16, 22, 28, 38, 50, 64, 80, 100, 126, 192],
         },
         SfBandIndexEntry {
            l: [0, 4, 8, 12, 16, 20, 24, 30, 36, 44, 54, 66, 82, 102, 126, 156, 194, 240, 296, 364, 448, 550, 576],
            s: [0, 4, 8, 12, 16, 22, 30, 42, 58, 78, 104, 138, 180, 192],
         },
      ],

/* mpeg-2.5, 11 & 12 KHz seem ok, 8 ok */
      [
         SfBandIndexEntry {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
            s: [0, 4, 8, 12, 18, 26, 36, 48, 62, 80, 104, 134, 174, 192],
         },
         SfBandIndexEntry {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
            s: [0, 4, 8, 12, 18, 26, 36, 48, 62, 80, 104, 134, 174, 192],
         },
// this 8khz table, and only 8khz, from mpeg123)
         SfBandIndexEntry {
            l: [0, 12, 24, 36, 48, 60, 72, 88, 108, 132, 160, 192, 232, 280, 336, 400, 476, 566, 568, 570, 572, 574, 576],
            s: [0, 8, 16, 24, 36, 52, 72, 96, 124, 160, 162, 164, 166, 192],
         },
      ],
   ],
];

extern "C" {
    pub fn sbt_mono_L3(sample: *mut f32, pcm: *mut i16, ch: c_int);
    pub fn sbt_dual_L3_func(sample: *mut f32, pcm: *mut i16, ch: c_int);
    pub fn sbt16_mono_L3(sample: *mut f32, pcm: *mut i16, ch: c_int);
    pub fn sbt16_dual_L3(sample: *mut f32, pcm: *mut i16, ch: c_int);
    pub fn sbt8_mono_L3(sample: *mut f32, pcm: *mut i16, ch: c_int);
    pub fn sbt8_dual_L3(sample: *mut f32, pcm: *mut i16, ch: c_int);

    pub fn sbtB_mono_L3(sample: *mut f32, pcm: *mut u8, ch: c_int);
    pub fn sbtB_dual_L3(sample: *mut f32, pcm: *mut u8, ch: c_int);
    pub fn sbtB16_mono_L3(sample: *mut f32, pcm: *mut u8, ch: c_int);
    pub fn sbtB16_dual_L3(sample: *mut f32, pcm: *mut u8, ch: c_int);
    pub fn sbtB8_mono_L3(sample: *mut f32, pcm: *mut u8, ch: c_int);
    pub fn sbtB8_dual_L3(sample: *mut f32, pcm: *mut u8, ch: c_int);
}

extern "C" {
    pub fn Xform_mono_func(pcm: *mut c_void, igr: c_int);
    pub fn Xform_dual_func(pcm: *mut c_void, igr: c_int);
    pub fn Xform_dual_mono(pcm: *mut c_void, igr: c_int);
    pub fn Xform_dual_right_func(pcm: *mut c_void, igr: c_int);
}

// xform_table static initialization would require function pointers
// static xform_table: [XFORM_FUNCTION; 5] = [
//     Xform_mono_func as XFORM_FUNCTION,
//     Xform_dual_func as XFORM_FUNCTION,
//     Xform_dual_mono as XFORM_FUNCTION,
//     Xform_mono_func as XFORM_FUNCTION,			/* left */
//     Xform_dual_right_func as XFORM_FUNCTION,
// ];

extern "C" {
    pub fn L3table_init() -> c_int;
    pub fn msis_init();
    pub fn sbt_init();
    pub fn quant_init_band_addr() -> *mut [c_int; 22];
    pub fn msis_init_band_addr() -> *mut [c_int; 22];
}

const NBUF: c_int = 8 * 1024;
const BUF_TRIGGER: c_int = NBUF - 1500;

extern "C" {
    pub static mut bFastEstimateOnly: bool;
}

/*---------------------------------------------------------*/
/* mpeg_head defined in mhead.h  frame bytes is without pMP3Stream->pad */
////@@@@INIT
pub unsafe extern "C" fn L3audio_decode_init(h: *mut MPEG_HEAD, framebytes_arg: c_int,
    reduction_code: c_int, transform_code: c_int, convert_code: c_int,
    freq_limit: c_int) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    // static int first_pass = 1;
    let samprate: c_int;
    let mut limit: c_int;
    let bit_code: c_int;
    let out_chans: c_int;

    // pMP3Stream->buf_ptr0 = 0;
    // pMP3Stream->buf_ptr1 = 0;

    // /* check if code handles */
    // if (h->option != 1)
    //    return 0;			/* layer III only */

    // if (h->id)
    //    pMP3Stream->ncbl_mixed = 8;		/* mpeg-1 */
    // else
    //    pMP3Stream->ncbl_mixed = 6;		/* mpeg-2 */

    // pMP3Stream->framebytes = framebytes_arg;

    // transform_code = transform_code;	/* not used, asm compatability */
    // bit_code = 0;
    // if (convert_code & 8)
    //    bit_code = 1;
    // convert_code = convert_code & 3;	/* higher bits used by dec8 freq cvt */
    // if (reduction_code < 0)
    //    reduction_code = 0;
    // if (reduction_code > 2)
    //    reduction_code = 2;
    // if (freq_limit < 1000)
    //    freq_limit = 1000;

    // samprate = sr_table[4 * h->id + h->sr_index];
    // if ((h->sync & 1) == 0)
    //    samprate = samprate / 2;	// mpeg 2.5

    // /*----- compute pMP3Stream->nsb_limit --------*/
    // pMP3Stream->nsb_limit = (freq_limit * 64L + samprate / 2) / samprate;
    // /*- caller limit -*/
    // limit = (32 >> reduction_code);
    // if (limit > 8)
    //    limit--;
    // if (pMP3Stream->nsb_limit > limit)
    //    pMP3Stream->nsb_limit = limit;
    // limit = 18 * pMP3Stream->nsb_limit;

    // k = h->id;
    // if ((h->sync & 1) == 0)
    //    k = 2;			// mpeg 2.5

    // if (k == 1)
    // {
    //    pMP3Stream->band_limit12 = 3 * sfBandIndexTable[k][h->sr_index].s[13];
    //    pMP3Stream->band_limit = pMP3Stream->band_limit21 = sfBandIndexTable[k][h->sr_index].l[22];
    // }
    // else
    // {
    //    pMP3Stream->band_limit12 = 3 * sfBandIndexTable[k][h->sr_index].s[12];
    //    pMP3Stream->band_limit = pMP3Stream->band_limit21 = sfBandIndexTable[k][h->sr_index].l[21];
    // }
    // pMP3Stream->band_limit += 8;		/* allow for antialias */
    // if (pMP3Stream->band_limit > limit)
    //    pMP3Stream->band_limit = limit;

    // if (pMP3Stream->band_limit21 > pMP3Stream->band_limit)
    //    pMP3Stream->band_limit21 = pMP3Stream->band_limit;
    // if (pMP3Stream->band_limit12 > pMP3Stream->band_limit)
    //    pMP3Stream->band_limit12 = pMP3Stream->band_limit;

    // pMP3Stream->band_limit_nsb = (pMP3Stream->band_limit + 17) / 18;	/* limit nsb's rounded up */
    // /*----------------------------------------------*/
    // pMP3Stream->gain_adjust = 0;		/* adjust gain e.g. cvt to mono sum channel */
    // if ((h->mode != 3) && (convert_code == 1))
    //    pMP3Stream->gain_adjust = -4;

    // pMP3Stream->outvalues = 1152 >> reduction_code;
    // if (h->id == 0)
    //    pMP3Stream->outvalues /= 2;

    // out_chans = 2;
    // if (h->mode == 3)
    //    out_chans = 1;
    // if (convert_code)
    //    out_chans = 1;

    // pMP3Stream->sbt_L3 = sbt_table[bit_code][reduction_code][out_chans - 1];
    // k = 1 + convert_code;
    // if (h->mode == 3)
    //    k = 0;
    // pMP3Stream->Xform = xform_table[k];

    // pMP3Stream->outvalues *= out_chans;

    // if (bit_code)
    //    pMP3Stream->outbytes = pMP3Stream->outvalues;
    // else
    //    pMP3Stream->outbytes = sizeof(short) * pMP3Stream->outvalues;

    // if (bit_code)
    //    pMP3Stream->zero_level_pcm = 128;	/* 8 bit output */
    // else
    //    pMP3Stream->zero_level_pcm = 0;

    // decinfo.channels = out_chans;
    // decinfo.outvalues = pMP3Stream->outvalues;
    // decinfo.samprate = samprate >> reduction_code;
    // if (bit_code)
    //    decinfo.bits = 8;
    // else
    //    decinfo.bits = sizeof(short) * 8;

    // decinfo.framebytes = pMP3Stream->framebytes;
    // decinfo.type = 0;

    // pMP3Stream->half_outbytes = pMP3Stream->outbytes / 2;
    // /*------------------------------------------*/

    // /*- init band tables --*/

    // k = h->id;
    // if ((h->sync & 1) == 0)
    //    k = 2;			// mpeg 2.5

    // for (i = 0; i < 22; i++)
    //    pMP3Stream->sfBandIndex[0][i] = sfBandIndexTable[k][h->sr_index].l[i + 1];
    // for (i = 0; i < 13; i++)
    //    pMP3Stream->sfBandIndex[1][i] = 3 * sfBandIndexTable[k][h->sr_index].s[i + 1];
    // for (i = 0; i < 22; i++)
    //    pMP3Stream->nBand[0][i] = sfBandIndexTable[k][h->sr_index].l[i + 1] - sfBandIndexTable[k][h->sr_index].l[i];
    // for (i = 0; i < 13; i++)
    //    pMP3Stream->nBand[1][i] = sfBandIndexTable[k][h->sr_index].s[i + 1] - sfBandIndexTable[k][h->sr_index].s[i];

    // /* init tables */
    // L3table_init();
    // /* init ms and is stereo modes */
    // msis_init();

    // /*----- init sbt ---*/
    // sbt_init();

    // /*--- clear buffers --*/
    // for (i = 0; i < 576; i++)
    //    yout[i] = 0.0f;
    // for (j = 0; j < 2; j++)
    // {
    //    for (k = 0; k < 2; k++)
    //    {
    //       for (i = 0; i < 576; i++)
    //       {
    //          pMP3Stream->sample[j][k][i].x = 0.0f;
    //          pMP3Stream->sample[j][k][i].s = 0;
    //       }
    //    }
    // }

    // if (h->id == 1)
    //    pMP3Stream->decode_function = L3audio_decode_MPEG1;
    // else
    //    pMP3Stream->decode_function = L3audio_decode_MPEG2;

    return 1;
}
/*---------------------------------------------------------*/
/*==========================================================*/
