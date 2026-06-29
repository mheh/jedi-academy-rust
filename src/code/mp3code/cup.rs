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

	$Id: cup.c,v 1.3 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  cup.c  ***************************************************

MPEG audio decoder Layer I/II  mpeg1 and mpeg2
should be portable ANSI C, should be endian independent


mod  2/21/95 2/21/95  add bit skip, sb limiting

mods 11/15/95 for Layer I

******************************************************************/
/******************************************************************

       MPEG audio software decoder portable ANSI c.
       Decodes all Layer I/II to 16 bit linear pcm.
       Optional stereo to mono conversion.  Optional
       output sample rate conversion to half or quarter of
       native mpeg rate. dec8.c adds oupuut conversion features.

-------------------------------------
int audio_decode_init(MPEG_HEAD *h, int framebytes_arg,
         int reduction_code, int transform_code, int convert_code,
         int freq_limit)

initilize decoder:
       return 0 = fail, not 0 = success

MPEG_HEAD *h    input, mpeg header info (returned by call to head_info)
pMP3Stream->framebytes      input, mpeg frame size (returned by call to head_info)
reduction_code  input, sample rate reduction code
                    0 = full rate
                    1 = half rate
                    2 = quarter rate

transform_code  input, ignored
convert_code    input, channel conversion
                  convert_code:  0 = two chan output
                                 1 = convert two chan to mono
                                 2 = convert two chan to left chan
                                 3 = convert two chan to right chan
freq_limit      input, limits bandwidth of pcm output to specified
                frequency.  Special use. Set to 24000 for normal use.


---------------------------------
void audio_decode_info( DEC_INFO *info)

information return:
          Call after audio_decode_init.  See mhead.h for
          information returned in DEC_INFO structure.


---------------------------------
IN_OUT audio_decode(unsigned char *bs, void *pcmbuf)

decode one mpeg audio frame:
bs        input, mpeg bitstream, must start with
          sync word.  Caution: may read up to 3 bytes
          beyond end of frame.
pcmbuf    output, pcm samples.

IN_OUT structure returns:
          Number bytes conceptually removed from mpeg bitstream.
          Returns 0 if sync loss.
          Number bytes of pcm output.

*******************************************************************/

use core::ffi::c_int;

// Stubs for external types from mhead.h and mp3struct.h
#[repr(C)]
pub struct MPEG_HEAD {
    // Placeholder structure from mhead.h
}

#[repr(C)]
pub struct DEC_INFO {
    // Placeholder structure from mp3struct.h
}

#[repr(C)]
pub struct IN_OUT {
    pub in_bytes: c_int,
    pub out_bytes: c_int,
}

// MP3_STREAM structure containing decoder state
#[repr(C)]
pub struct MP3_STREAM {
    pub framebytes: c_int,
    pub outbytes: c_int,
    pub outvalues: c_int,
    pub bit_skip: c_int,
    pub nsb_limit: c_int,
    pub max_sb: c_int,
    pub stereo_sb: c_int,
    pub pad: c_int,
    pub nbat: [c_int; 4],
    pub bat: [[c_int; 16]; 4],
    pub cs_factor: [[f32; 64]; 3],
    pub sbt: *mut core::ffi::c_void,
}

/*-------------------------------------------------------
NOTE:  Decoder may read up to three bytes beyond end of
frame.  Calling application must ensure that this does
not cause a memory access violation (protection fault)
---------------------------------------------------------*/

/*====================================================================*/
/*----------------*/
//@@@@ This next one (decinfo) is ok:
pub static mut decinfo: DEC_INFO = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };		/* global for Layer III */	// only written into by decode init funcs, then copied to stack struct higher up

/*----------------*/
static mut look_c_value: [f32; 18] = [0.0; 18];	/* built by init */	// effectively constant

/*----------------*/
////@@@@static int pMP3Stream->outbytes;		// !!!!!!!!!!!!!!?
////@@@@static int pMP3Stream->framebytes;		// !!!!!!!!!!!!!!!!
////@@@@static int pMP3Stream->outvalues;		// !!!!!!!!!!!!?
////@@@@static int pad;
static look_joint: [c_int; 16] =
[				/* lookup stereo sb's by mode+ext */
   64, 64, 64, 64,		/* stereo */
   2 * 4, 2 * 8, 2 * 12, 2 * 16,	/* joint */
   64, 64, 64, 64,		/* dual */
   32, 32, 32, 32,		/* mono */
];

/*----------------*/
////@@@@static int max_sb;		// !!!!!!!!! L1, 2 3
////@@@@static int stereo_sb;

/*----------------*/
////@@@@static int pMP3Stream->nsb_limit = 6;
////@@@@static int bit_skip;
static bat_bit_master: [c_int; 18] =
[
   0, 5, 7, 9, 10, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 48
];

/*----------------*/
////@@@@static int nbat[4] = {3, 8, 12, 7};	// !!!!!!!!!! not constant!!!!
////@@@@static int bat[4][16];	// built as constant, but built according to header type (sigh)
static mut ballo: [c_int; 64] = [0; 64];		/* set by unpack_ba */					// scratchpad
static mut samp_dispatch: [u32; 66] = [0; 66];	/* set by unpack_ba */		// scratchpad?
static mut c_value: [f32; 64] = [0.0; 64];	/* set by unpack_ba */					// scratchpad

/*----------------*/
static mut sf_dispatch: [u32; 66] = [0; 66];	/* set by unpack_ba */		// scratchpad?
static mut sf_table: [f32; 64] = [0.0; 64];		// effectively constant
////@@@@ static float cs_factor[3][64];

/*----------------*/
////@@@@FINDME - groan....  (I shoved a *2 in just in case it needed it for stereo. This whole thing is crap now
pub static mut sample: [f32; 2304 * 2] = [0.0; 2304 * 2];		/* global for use by Later 3 */	// !!!!!!!!!!!!!!!!!!!!!! // scratchpad?
static mut group3_table: [[i8; 3]; 32] = [[0; 3]; 32];		// effectively constant
static mut group5_table: [[i8; 3]; 128] = [[0; 3]; 128];	// effectively constant
static mut group9_table: [[i16; 3]; 1024] = [[0; 3]; 1024];	// effectively constant

/*----------------*/

////@@@@typedef void (*SBT_FUNCTION) (float *sample, short *pcm, int n);
extern "C" {
    pub fn sbt_mono(sample: *mut f32, pcm: *mut i16, n: c_int);
    pub fn sbt_dual(sample: *mut f32, pcm: *mut i16, n: c_int);
}
////@@@@static SBT_FUNCTION sbt = sbt_mono;

pub type AUDIO_DECODE_ROUTINE = unsafe extern "C" fn(*mut u8, *mut i16) -> IN_OUT;
extern "C" {
    pub fn L2audio_decode(bs: *mut u8, pcm: *mut i16) -> IN_OUT;
}
static mut audio_decode_routine: AUDIO_DECODE_ROUTINE = L2audio_decode;

pub static mut pMP3Stream: *mut MP3_STREAM = core::ptr::null_mut();

/*======================================================================*/
/*======================================================================*/
/* get bits from bitstream in endian independent way */
////@@@@ FINDME - this stuff doesn't appear to be used by any of our samples (phew)
static mut bs_ptr: *mut u8 = core::ptr::null_mut();
static mut bitbuf: u64 = 0;
static mut bits: c_int = 0;
static mut bitval: i64 = 0;

/*------------- initialize bit getter -------------*/
unsafe fn load_init(buf: *mut u8)
{
   bs_ptr = buf;
   bits = 0;
   bitbuf = 0;
}
/*------------- get n bits from bitstream -------------*/
unsafe fn load(n: c_int) -> i64
{
   let mut x: u64;

   if bits < n
   {				/* refill bit buf if necessary */
      while bits <= 24
      {
	 bitbuf = (bitbuf << 8) | *bs_ptr as u64;
	 bs_ptr = bs_ptr.offset(1);
	 bits += 8;
      }
   }
   bits -= n;
   x = bitbuf >> (bits as u32);
   bitbuf -= x << (bits as u32);
   return x as i64;
}
/*------------- skip over n bits in bitstream -------------*/
unsafe fn skip(mut n: c_int)
{
   let mut k: c_int;

   if bits < n
   {
      n -= bits;
      k = n >> 3;
/*--- bytes = n/8 --*/
      bs_ptr = bs_ptr.offset(k as isize);
      n -= k << 3;
      bitbuf = *bs_ptr as u64;
      bs_ptr = bs_ptr.offset(1);
      bits = 8;
   }
   bits -= n;
   bitbuf -= (bitbuf >> (bits as u32)) << (bits as u32);
}
/*--------------------------------------------------------------*/
macro_rules! mac_load_check {
    ($n:expr) => {
        if bits < ($n) {
            while bits <= 24 {
                bitbuf = (bitbuf << 8) | (*bs_ptr as u64);
                bs_ptr = bs_ptr.offset(1);
                bits += 8;
            }
        }
    };
}
/*--------------------------------------------------------------*/
macro_rules! mac_load {
    ($n:expr) => {{
        bits -= $n;
        bitval = (bitbuf >> (bits as u32)) as i64;
        bitbuf -= (bitval as u64) << (bits as u32);
        bitval
    }};
}
/*======================================================================*/
unsafe fn unpack_ba()
{
   let mut i: c_int;
   let mut j: c_int;
   let mut k: c_int = 0;
   let nbit: [c_int; 4] = [4, 4, 3, 2];
   let mut nstereo: c_int;

   (*pMP3Stream).bit_skip = 0;
   nstereo = (*pMP3Stream).stereo_sb;
   for i in 0..4
   {
      for j in 0..(*pMP3Stream).nbat[i as usize]
      {
	 mac_load_check!(4);
	 ballo[k as usize] = samp_dispatch[k as usize] = (*pMP3Stream).bat[i as usize][mac_load!(nbit[i as usize]) as usize] as u32;
	 if k >= (*pMP3Stream).nsb_limit
	 {
	    (*pMP3Stream).bit_skip += bat_bit_master[samp_dispatch[k as usize] as usize];
	 }
	 c_value[k as usize] = look_c_value[samp_dispatch[k as usize] as usize];
	 nstereo -= 1;
	 if nstereo < 0
	 {
	    ballo[(k + 1) as usize] = ballo[k as usize];
	    samp_dispatch[k as usize] += 18;	/* flag as joint */
	    samp_dispatch[(k + 1) as usize] = samp_dispatch[k as usize];	/* flag for sf */
	    c_value[(k + 1) as usize] = c_value[k as usize];
	    k += 1;
	 }
	 k += 1;
      }
   }
   samp_dispatch[(*pMP3Stream).nsb_limit as usize] = 37;	/* terminate the dispatcher with skip */
   samp_dispatch[k as usize] = 36;	/* terminate the dispatcher */

}
/*-------------------------------------------------------------------------*/
unsafe fn unpack_sfs()	/* unpack scale factor selectors */
{
   let mut i: c_int;

   for i in 0..(*pMP3Stream).max_sb
   {
      mac_load_check!(2);
      if ballo[i as usize] != 0
      {
	 sf_dispatch[i as usize] = mac_load!(2) as u32;
      }
      else
      {
	 sf_dispatch[i as usize] = 4;	/* no allo */
      }
   }
   sf_dispatch[i as usize] = 5;		/* terminate dispatcher */
}
/*-------------------------------------------------------------------------*/
unsafe fn unpack_sf()		/* unpack scale factor */
{				/* combine dequant and scale factors */
   let mut i: c_int = -1;

 'dispatch: loop {
   i += 1;
   match sf_dispatch[i as usize]
   {
      0 => {			/* 3 factors 012 */
	 mac_load_check!(18);
	 (*pMP3Stream).cs_factor[0][i as usize] = c_value[i as usize] * sf_table[mac_load!(6) as usize];
	 (*pMP3Stream).cs_factor[1][i as usize] = c_value[i as usize] * sf_table[mac_load!(6) as usize];
	 (*pMP3Stream).cs_factor[2][i as usize] = c_value[i as usize] * sf_table[mac_load!(6) as usize];
	 continue 'dispatch;
      }
      1 => {			/* 2 factors 002 */
	 mac_load_check!(12);
	 (*pMP3Stream).cs_factor[1][i as usize] = c_value[i as usize] * sf_table[mac_load!(6) as usize];
	 (*pMP3Stream).cs_factor[0][i as usize] = (*pMP3Stream).cs_factor[1][i as usize];
	 (*pMP3Stream).cs_factor[2][i as usize] = c_value[i as usize] * sf_table[mac_load!(6) as usize];
	 continue 'dispatch;
      }
      2 => {			/* 1 factor 000 */
	 mac_load_check!(6);
	 (*pMP3Stream).cs_factor[2][i as usize] = c_value[i as usize] * sf_table[mac_load!(6) as usize];
	 (*pMP3Stream).cs_factor[1][i as usize] = (*pMP3Stream).cs_factor[2][i as usize];
	 (*pMP3Stream).cs_factor[0][i as usize] = (*pMP3Stream).cs_factor[2][i as usize];
	 continue 'dispatch;
      }
      3 => {			/* 2 factors 022 */
	 mac_load_check!(12);
	 (*pMP3Stream).cs_factor[0][i as usize] = c_value[i as usize] * sf_table[mac_load!(6) as usize];
	 (*pMP3Stream).cs_factor[2][i as usize] = c_value[i as usize] * sf_table[mac_load!(6) as usize];
	 (*pMP3Stream).cs_factor[1][i as usize] = (*pMP3Stream).cs_factor[2][i as usize];
	 continue 'dispatch;
      }
      4 => {			/* no allo */
	/*-- (*pMP3Stream).cs_factor[2][i as usize] = (*pMP3Stream).cs_factor[1][i as usize] = (*pMP3Stream).cs_factor[0][i as usize] = 0.0;  --*/
	 continue 'dispatch;
      }
      5 => {			/* all done */
	 break 'dispatch;
      }
      _ => {}
   }				/* end switch */
}
}
/*-------------------------------------------------------------------------*/
macro_rules! UNPACK_N {
    ($n:expr) => {{
        s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * ((load($n) as i32 - ((1 << ($n - 1)) - 1)) as f32));
        s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * ((load($n) as i32 - ((1 << ($n - 1)) - 1)) as f32));
        s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * ((load($n) as i32 - ((1 << ($n - 1)) - 1)) as f32));
        continue 'dispatch;
    }};
}
macro_rules! UNPACK_N2 {
    ($n:expr) => {{
        mac_load_check!(3 * $n);
        s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * ((mac_load!($n) as i32 - ((1 << ($n - 1)) - 1)) as f32));
        s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * ((mac_load!($n) as i32 - ((1 << ($n - 1)) - 1)) as f32));
        s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * ((mac_load!($n) as i32 - ((1 << ($n - 1)) - 1)) as f32));
        continue 'dispatch;
    }};
}
macro_rules! UNPACK_N3 {
    ($n:expr) => {{
        mac_load_check!(2 * $n);
        s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * ((mac_load!($n) as i32 - ((1 << ($n - 1)) - 1)) as f32));
        s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * ((mac_load!($n) as i32 - ((1 << ($n - 1)) - 1)) as f32));
        mac_load_check!($n);
        s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * ((mac_load!($n) as i32 - ((1 << ($n - 1)) - 1)) as f32));
        continue 'dispatch;
    }};
}
macro_rules! UNPACKJ_N {
    ($n:expr) => {{
        let tmp = load($n) as i32 - ((1 << ($n - 1)) - 1);
        s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * (tmp as f32));
        s.offset(k as isize + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * (tmp as f32));
        let tmp = load($n) as i32 - ((1 << ($n - 1)) - 1);
        s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * (tmp as f32));
        s.offset(k as isize + 64 + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * (tmp as f32));
        let tmp = load($n) as i32 - ((1 << ($n - 1)) - 1);
        s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * (tmp as f32));
        s.offset(k as isize + 128 + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * (tmp as f32));
        k += 1;		/* skip right chan dispatch */
        continue 'dispatch;
    }};
}
/*-------------------------------------------------------------------------*/
unsafe fn unpack_samp()	/* unpack samples */
{
   let mut i: c_int;
   let mut j: c_int;
   let mut k: c_int;
   let s: *mut f32;
   let mut n: c_int;
   let tmp: i64;

   s = sample.as_mut_ptr();
   for i in 0..3
   {				/* 3 groups of scale factors */
      for j in 0..4
      {
	 k = -1;
       'dispatch: loop {
	   k += 1;
	   match samp_dispatch[k as usize]
	   {
	      0 => {
		 s.offset(k as isize + 128).write(0.0);
		 s.offset(k as isize + 64).write(0.0);
		 s.offset(k as isize).write(0.0);
		 continue 'dispatch;
	      }
	      1 => {		/* 3 levels grouped 5 bits */
		 mac_load_check!(5);
		 n = mac_load!(5) as c_int;
		 s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group3_table[n as usize][0] as f32);
		 s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group3_table[n as usize][1] as f32);
		 s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group3_table[n as usize][2] as f32);
		 continue 'dispatch;
	      }
	      2 => {		/* 5 levels grouped 7 bits */
		 mac_load_check!(7);
		 n = mac_load!(7) as c_int;
		 s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group5_table[n as usize][0] as f32);
		 s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group5_table[n as usize][1] as f32);
		 s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group5_table[n as usize][2] as f32);
		 continue 'dispatch;
	      }
	      3 => {
		 UNPACK_N2!(3)	/* 7 levels */
	      }
	      4 => {		/* 9 levels grouped 10 bits */
		 mac_load_check!(10);
		 n = mac_load!(10) as c_int;
		 s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group9_table[n as usize][0] as f32);
		 s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group9_table[n as usize][1] as f32);
		 s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group9_table[n as usize][2] as f32);
		 continue 'dispatch;
	      }
	      5 => {
		 UNPACK_N2!(4)	/* 15 levels */
	      }
	      6 => {
		 UNPACK_N2!(5)	/* 31 levels */
	      }
	      7 => {
		 UNPACK_N2!(6)	/* 63 levels */
	      }
	      8 => {
		 UNPACK_N2!(7)	/* 127 levels */
	      }
	      9 => {
		 UNPACK_N2!(8)	/* 255 levels */
	      }
	      10 => {
		 UNPACK_N3!(9)	/* 511 levels */
	      }
	      11 => {
		 UNPACK_N3!(10)	/* 1023 levels */
	      }
	      12 => {
		 UNPACK_N3!(11)	/* 2047 levels */
	      }
	      13 => {
		 UNPACK_N3!(12)	/* 4095 levels */
	      }
	      14 => {
		 UNPACK_N(13)	/* 8191 levels */
	      }
	      15 => {
		 UNPACK_N(14)	/* 16383 levels */
	      }
	      16 => {
		 UNPACK_N(15)	/* 32767 levels */
	      }
	      17 => {
		 UNPACK_N(16)	/* 65535 levels */
	      }
/* -- joint ---- */
	      18 => {
		 s.offset(k as isize + 128 + 1).write(0.0);
		 s.offset(k as isize + 128).write(0.0);
		 s.offset(k as isize + 64 + 1).write(0.0);
		 s.offset(k as isize + 64).write(0.0);
		 s.offset(k as isize + 1).write(0.0);
		 s.offset(k as isize).write(0.0);
		 k += 1;		/* skip right chan dispatch */
		 continue 'dispatch;
	      }
	      19 => {	/* 3 levels grouped 5 bits */
		 n = load(5) as c_int;
		 s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group3_table[n as usize][0] as f32);
		 s.offset(k as isize + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * group3_table[n as usize][0] as f32);
		 s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group3_table[n as usize][1] as f32);
		 s.offset(k as isize + 64 + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * group3_table[n as usize][1] as f32);
		 s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group3_table[n as usize][2] as f32);
		 s.offset(k as isize + 128 + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * group3_table[n as usize][2] as f32);
		 k += 1;		/* skip right chan dispatch */
		 continue 'dispatch;
	      }
	      20 => {	/* 5 levels grouped 7 bits */
		 n = load(7) as c_int;
		 s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group5_table[n as usize][0] as f32);
		 s.offset(k as isize + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * group5_table[n as usize][0] as f32);
		 s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group5_table[n as usize][1] as f32);
		 s.offset(k as isize + 64 + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * group5_table[n as usize][1] as f32);
		 s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group5_table[n as usize][2] as f32);
		 s.offset(k as isize + 128 + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * group5_table[n as usize][2] as f32);
		 k += 1;		/* skip right chan dispatch */
		 continue 'dispatch;
	      }
	      21 => {
		 UNPACKJ_N!(3)	/* 7 levels */
	      }
	      22 => {	/* 9 levels grouped 10 bits */
		 n = load(10) as c_int;
		 s.offset(k as isize).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group9_table[n as usize][0] as f32);
		 s.offset(k as isize + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * group9_table[n as usize][0] as f32);
		 s.offset(k as isize + 64).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group9_table[n as usize][1] as f32);
		 s.offset(k as isize + 64 + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * group9_table[n as usize][1] as f32);
		 s.offset(k as isize + 128).write((*pMP3Stream).cs_factor[i as usize][k as usize] * group9_table[n as usize][2] as f32);
		 s.offset(k as isize + 128 + 1).write((*pMP3Stream).cs_factor[i as usize][(k + 1) as usize] * group9_table[n as usize][2] as f32);
		 k += 1;		/* skip right chan dispatch */
		 continue 'dispatch;
	      }
	      23 => {
		 UNPACKJ_N!(4)	/* 15 levels */
	      }
	      24 => {
		 UNPACKJ_N!(5)	/* 31 levels */
	      }
	      25 => {
		 UNPACKJ_N!(6)	/* 63 levels */
	      }
	      26 => {
		 UNPACKJ_N!(7)	/* 127 levels */
	      }
	      27 => {
		 UNPACKJ_N!(8)	/* 255 levels */
	      }
	      28 => {
		 UNPACKJ_N!(9)	/* 511 levels */
	      }
	      29 => {
		 UNPACKJ_N!(10)	/* 1023 levels */
	      }
	      30 => {
		 UNPACKJ_N!(11)	/* 2047 levels */
	      }
	      31 => {
		 UNPACKJ_N!(12)	/* 4095 levels */
	      }
	      32 => {
		 UNPACKJ_N!(13)	/* 8191 levels */
	      }
	      33 => {
		 UNPACKJ_N!(14)	/* 16383 levels */
	      }
	      34 => {
		 UNPACKJ_N!(15)	/* 32767 levels */
	      }
	      35 => {
		 UNPACKJ_N!(16)	/* 65535 levels */
	      }
/* -- end of dispatch -- */
	      37 => {
		 skip((*pMP3Stream).bit_skip);
		 break 'dispatch;
	      }
	      36 => {
		 s.offset(3 * 64).read();
		 break 'dispatch;
	      }
	      _ => {}
	   }			/* end switch */
	 }			/* end dispatch loop */
      }				/* end j loop */
   }				/* end i loop */


}
/*-------------------------------------------------------------------------*/
pub static mut gpNextByteAfterData: *mut u8 = core::ptr::null_mut();
pub unsafe fn audio_decode(bs: *mut u8, pcm: *mut i16, pNextByteAfterData: *mut u8) -> IN_OUT
{
	gpNextByteAfterData = pNextByteAfterData;	// sigh....
   L2audio_decode(bs, pcm)
}
/*-------------------------------------------------------------------------*/
pub unsafe fn L2audio_decode(bs: *mut u8, pcm: *mut i16) -> IN_OUT
{
   let mut sync: c_int;
   let mut prot: c_int;
   let mut in_out: IN_OUT;

   load_init(bs);		/* initialize bit getter */
/* test sync */
   in_out.in_bytes = 0;		/* assume fail */
   in_out.out_bytes = 0;
   sync = load(12) as c_int;
   if sync != 0xFFF
   {
      return in_out;		/* sync fail */
   }

   load(3);			/* skip id and option (checked by init) */
   prot = load(1) as c_int;		/* load prot bit */
   load(6);			/* skip to pad */
   (*pMP3Stream).pad = load(1) as c_int;
   load(1);			/* skip to mode */
   (*pMP3Stream).stereo_sb = look_joint[load(4) as usize];
   if prot != 0
   {
      load(4);			/* skip to data */
   }
   else
   {
      load(20);			/* skip crc */
   }

   unpack_ba();			/* unpack bit allocation */
   unpack_sfs();		/* unpack scale factor selectors */
   unpack_sf();			/* unpack scale factor */
   unpack_samp();		/* unpack samples */

   let sbt_fn: extern "C" fn(*mut f32, *mut i16, c_int) = core::mem::transmute((*pMP3Stream).sbt);
   sbt_fn(sample.as_mut_ptr(), pcm, 36);
/*-----------*/
   in_out.in_bytes = (*pMP3Stream).framebytes + (*pMP3Stream).pad;
   in_out.out_bytes = (*pMP3Stream).outbytes;

   return in_out;
}
/*-------------------------------------------------------------------------*/
#[cfg(feature = "compile_me")]
mod cupini {
    // #include "cupini.c"		/* initialization */
}
#[cfg(feature = "compile_me")]
mod cupL1 {
    // #include "cupL1.c"		/* Layer I */
}
/*-------------------------------------------------------------------------*/
