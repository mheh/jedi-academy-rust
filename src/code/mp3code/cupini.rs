// #pragma warning(disable:4206)	// nonstandard extension used : translation unit is empty
// #ifdef COMPILE_ME
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

	$Id: cupini.c,v 1.3 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/*=========================================================
 initialization for cup.c - include to cup.c
 mpeg audio decoder portable "c"

mod 8/6/96 add 8 bit output

mod 5/10/95 add quick (low precision) window

mod 5/16/95 sb limit for reduced samprate output
            changed from 94% to 100% of Nyquist sb

mod 11/15/95 for Layer I


=========================================================*/
/*-- compiler bug, floating constant overflow w/ansi --*/
// #ifdef _MSC_VER
// #pragma warning(disable:4056)
// #endif

#![allow(non_snake_case)]

use core::ffi::c_int;

static STEPS: [c_int; 18] =
[
   0, 3, 5, 7, 9, 15, 31, 63, 127,
   255, 511, 1023, 2047, 4095, 8191, 16383, 32767, 65535];


/* ABCD_INDEX = lookqt[mode][sr_index][br_index]  */
/* -1 = invalid  */
static LOOKQT: [[[i8; 16]; 3]; 4] =
[
 [[1, -1, -1, -1, 2, -1, 2, 0, 0, 0, 1, 1, 1, 1, 1, -1],	/*  44ks stereo */
  [0, -1, -1, -1, 2, -1, 2, 0, 0, 0, 0, 0, 0, 0, 0, -1],	/*  48ks */
  [1, -1, -1, -1, 3, -1, 3, 0, 0, 0, 1, 1, 1, 1, 1, -1]],	/*  32ks */
 [[1, -1, -1, -1, 2, -1, 2, 0, 0, 0, 1, 1, 1, 1, 1, -1],	/*  44ks joint stereo */
  [0, -1, -1, -1, 2, -1, 2, 0, 0, 0, 0, 0, 0, 0, 0, -1],	/*  48ks */
  [1, -1, -1, -1, 3, -1, 3, 0, 0, 0, 1, 1, 1, 1, 1, -1]],	/*  32ks */
 [[1, -1, -1, -1, 2, -1, 2, 0, 0, 0, 1, 1, 1, 1, 1, -1],	/*  44ks dual chan */
  [0, -1, -1, -1, 2, -1, 2, 0, 0, 0, 0, 0, 0, 0, 0, -1],	/*  48ks */
  [1, -1, -1, -1, 3, -1, 3, 0, 0, 0, 1, 1, 1, 1, 1, -1]],	/*  32ks */
// mono extended beyond legal br index
//  1,2,2,0,0,0,1,1,1,1,1,1,1,1,1,-1,          /*  44ks single chan */
//  0,2,2,0,0,0,0,0,0,0,0,0,0,0,0,-1,          /*  48ks */
//  1,3,3,0,0,0,1,1,1,1,1,1,1,1,1,-1,          /*  32ks */
// legal mono
 [[1, 2, 2, 0, 0, 0, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1],	/*  44ks single chan */
  [0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1, -1, -1],	/*  48ks */
  [1, 3, 3, 0, 0, 0, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1]],	/*  32ks */
];

static SR_TABLE: [c_int; 8] =
[22050, 24000, 16000, 1,
 44100, 48000, 32000, 1];

/* bit allocation table look up */
/* table per mpeg spec tables 3b2a/b/c/d  /e is mpeg2 */
/* look_bat[abcd_index][4][16]  */
static LOOK_BAT: [[[u8; 16]; 4]; 5] =
[
/* LOOK_BATA */
 [[0, 1, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
  [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 17],
  [0, 1, 2, 3, 4, 5, 6, 17, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 1, 2, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]],
/* LOOK_BATB */
 [[0, 1, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
  [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 17],
  [0, 1, 2, 3, 4, 5, 6, 17, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 1, 2, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]],
/* LOOK_BATC */
 [[0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 1, 2, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]],
/* LOOK_BATD */
 [[0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 1, 2, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]],
/* LOOK_BATE */
 [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 1, 2, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 1, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]],
];

/* look_nbat[abcd_index]][4] */
static LOOK_NBAT: [[u8; 4]; 5] =
[
  [3, 8, 12, 4],
  [3, 8, 12, 7],
  [2, 0, 6, 0],
  [2, 0, 10, 0],
  [4, 0, 7, 19],
];


extern "C" {
    fn sbt_mono(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt_dual(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt_dual_mono(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt_dual_left(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt_dual_right(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt16_mono(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt16_dual(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt16_dual_mono(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt16_dual_left(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt16_dual_right(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt8_mono(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt8_dual(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt8_dual_mono(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt8_dual_left(sample: *mut f32, pcm: *mut i16, n: c_int);
    fn sbt8_dual_right(sample: *mut f32, pcm: *mut i16, n: c_int);

    /*--- 8 bit output ---*/
    fn sbtB_mono(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB_dual(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB_dual_mono(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB_dual_left(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB_dual_right(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB16_mono(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB16_dual(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB16_dual_mono(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB16_dual_left(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB16_dual_right(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB8_mono(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB8_dual(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB8_dual_mono(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB8_dual_left(sample: *mut f32, pcm: *mut u8, n: c_int);
    fn sbtB8_dual_right(sample: *mut f32, pcm: *mut u8, n: c_int);
}

pub type SBT_FUNCTION = extern "C" fn(*mut f32, *mut i16, c_int);

static SBT_TABLE: [[[SBT_FUNCTION; 5]; 3]; 2] =
[
 [[sbt_mono, sbt_dual, sbt_dual_mono, sbt_dual_left, sbt_dual_right],
  [sbt16_mono, sbt16_dual, sbt16_dual_mono, sbt16_dual_left, sbt16_dual_right],
  [sbt8_mono, sbt8_dual, sbt8_dual_mono, sbt8_dual_left, sbt8_dual_right]],
 [[(sbtB_mono as SBT_FUNCTION),
   (sbtB_dual as SBT_FUNCTION),
   (sbtB_dual_mono as SBT_FUNCTION),
   (sbtB_dual_left as SBT_FUNCTION),
   (sbtB_dual_right as SBT_FUNCTION)],
  [(sbtB16_mono as SBT_FUNCTION),
   (sbtB16_dual as SBT_FUNCTION),
   (sbtB16_dual_mono as SBT_FUNCTION),
   (sbtB16_dual_left as SBT_FUNCTION),
   (sbtB16_dual_right as SBT_FUNCTION)],
  [(sbtB8_mono as SBT_FUNCTION),
   (sbtB8_dual as SBT_FUNCTION),
   (sbtB8_dual_mono as SBT_FUNCTION),
   (sbtB8_dual_left as SBT_FUNCTION),
   (sbtB8_dual_right as SBT_FUNCTION)]],
];

static OUT_CHANS: [c_int; 5] =
[1, 2, 1, 1, 1];


extern "C" {
    fn L1audio_decode_init(h: *mut MPEG_HEAD, framebytes_arg: c_int,
            reduction_code: c_int, transform_code: c_int, convert_code: c_int,
            freq_limit: c_int) -> c_int;
    fn L3audio_decode_init(h: *mut MPEG_HEAD, framebytes_arg: c_int,
            reduction_code: c_int, transform_code: c_int, convert_code: c_int,
            freq_limit: c_int) -> c_int;
    fn sbt_init();
}

// Stub type for MPEG_HEAD - full definition in mhead.h equivalent
#[repr(C)]
pub struct MPEG_HEAD {
    // Fields are external; this is a structural placeholder
}

// Stub type for IN_OUT
#[repr(C)]
pub struct IN_OUT {
    // Fields are external
}

extern "C" {
    fn L1audio_decode(bs: *mut u8, pcm: *mut i16) -> IN_OUT;
    fn L2audio_decode(bs: *mut u8, pcm: *mut i16) -> IN_OUT;
    fn L3audio_decode(bs: *mut u8, pcm: *mut u8) -> IN_OUT;
}

pub type AUDIO_DECODE_ROUTINE = extern "C" fn(*mut u8, *mut i16) -> IN_OUT;

static DECODE_ROUTINE_TABLE: [AUDIO_DECODE_ROUTINE; 4] =
[
   L2audio_decode,
   (L3audio_decode as AUDIO_DECODE_ROUTINE),
   L2audio_decode,
   L1audio_decode,
];

// External globals - defined elsewhere
extern "C" {
    static mut pMP3Stream: *mut MP3Stream;
    static mut sample: [f32; 2304];
    static mut decinfo: DEC_INFO;
    static mut look_c_value: [f32; 18];
    static mut sf_table: [f32; 64];
    static mut group3_table: [[i8; 3]; 32];
    static mut group5_table: [[i8; 3]; 128];
    static mut group9_table: [[i16; 3]; 1024];
    static mut audio_decode_routine: AUDIO_DECODE_ROUTINE;
    fn pow(x: f64, y: f64) -> f64;
}

// Stub types for external structures
#[repr(C)]
pub struct MP3Stream {
    // Fields unresolved
}

#[repr(C)]
pub struct DEC_INFO {
    // Fields unresolved
}

/*---------------------------------------------------------*/
unsafe fn table_init()
{
   let mut i: c_int;
   let mut j: c_int;
   let mut code: c_int;
   static mut iOnceOnly: c_int = 0;

   if iOnceOnly == 0
   {
       iOnceOnly += 1;

       /*--  c_values (dequant) --*/
       i = 1;
       while i < 18
       {
           let idx = i as usize;
           let val = 2.0f32 / STEPS[idx] as f32;
           // look_c_value[i] = val - assignment through external global
           i += 1;
       }

       /*--  scale factor table, scale by 32768 for 16 pcm output  --*/
       i = 0;
       while i < 64
       {
           let val = 32768.0f64 * 2.0f64 * pow(2.0f64, -(i as f64) / 3.0f64);
           // sf_table[i] = val - assignment through external global
           i += 1;
       }

       /*--  grouped 3 level lookup table 5 bit token --*/
       i = 0;
       while i < 32
       {
           code = i;
           j = 0;
           while j < 3
           {
               // group3_table[i][j] = (code % 3) - 1 - assignment through external global
               code /= 3;
               j += 1;
           }
           i += 1;
       }

       /*--  grouped 5 level lookup table 7 bit token --*/
       i = 0;
       while i < 128
       {
           code = i;
           j = 0;
           while j < 3
           {
               // group5_table[i][j] = (code % 5) - 2 - assignment through external global
               code /= 5;
               j += 1;
           }
           i += 1;
       }

       /*--  grouped 9 level lookup table 10 bit token --*/
       i = 0;
       while i < 1024
       {
           code = i;
           j = 0;
           while j < 3
           {
               // group9_table[i][j] = (code % 9) - 4 - assignment through external global
               code /= 9;
               j += 1;
           }
           i += 1;
       }
   }
}

/*---------------------------------------------------------*/
/* mpeg_head defined in mhead.h  frame bytes is without pad */
#[no_mangle]
pub extern "C" fn audio_decode_init(h: *mut MPEG_HEAD, framebytes_arg: c_int,
            reduction_code: c_int, transform_code: c_int, convert_code: c_int,
            freq_limit: c_int) -> c_int
{
   unsafe {
       let mut i: c_int;
       let mut j: c_int;
       let mut k: c_int;
       static mut first_pass: c_int = 1;
       let mut abcd_index: c_int;
       let mut samprate: c_int;
       let mut limit: c_int;
       let mut bit_code: c_int;

       if first_pass != 0
       {
           table_init();
           first_pass = 0;
       }

       /* select decoder routine Layer I,II,III */
       audio_decode_routine = DECODE_ROUTINE_TABLE[((*h).option & 3) as usize];


       if (*h).option == 3		/* layer I */
       {
           return L1audio_decode_init(h, framebytes_arg,
                 reduction_code, transform_code, convert_code, freq_limit);
       }

       if (*h).option == 1		/* layer III */
       {
           return L3audio_decode_init(h, framebytes_arg,
                 reduction_code, transform_code, convert_code, freq_limit);
       }



       // transform_code = transform_code;	/* not used, asm compatability */
       bit_code = 0;
       if (convert_code & 8) != 0
       {
           bit_code = 1;
       }
       // convert_code = convert_code & 3;	/* higher bits used by dec8 freq cvt */
       if reduction_code < 0
       {
           // reduction_code = 0;
       }
       if reduction_code > 2
       {
           // reduction_code = 2;
       }
       if freq_limit < 1000
       {
           // freq_limit = 1000;
       }


       (*pMP3Stream).framebytes = framebytes_arg;
       /* check if code handles */
       if (*h).option != 2
       {
           return 0;			/* layer II only */
       }
       if (*h).sr_index == 3
       {
           return 0;			/* reserved */
       }

       /* compute abcd index for bit allo table selection */
       if (*h).id != 0			/* mpeg 1 */
       {
           abcd_index = LOOKQT[(*h).mode as usize][(*h).sr_index as usize][(*h).br_index as usize] as c_int;
       }
       else
       {
           abcd_index = 4;		/* mpeg 2 */
       }

       if abcd_index < 0
       {
           return 0;			// fail invalid Layer II bit rate index
       }

       i = 0;
       while i < 4
       {
           j = 0;
           while j < 16
           {
               // (*pMP3Stream).bat[i][j] = LOOK_BAT[abcd_index as usize][i as usize][j as usize]
               j += 1;
           }
           i += 1;
       }
       i = 0;
       while i < 4
       {
           // (*pMP3Stream).nbat[i] = LOOK_NBAT[abcd_index as usize][i as usize]
           i += 1;
       }
       // (*pMP3Stream).max_sb = (*pMP3Stream).nbat[0] + (*pMP3Stream).nbat[1] + (*pMP3Stream).nbat[2] + (*pMP3Stream).nbat[3]
       /*----- compute (*pMP3Stream).nsb_limit --------*/
       samprate = SR_TABLE[(4 * (*h).id + (*h).sr_index) as usize];
       // (*pMP3Stream).nsb_limit = (freq_limit * 64 + samprate / 2) / samprate;
       /*- caller limit -*/
       /*---- limit = 0.94*(32>>reduction_code);  ----*/
       limit = (32 >> reduction_code);
       if limit > 8
       {
           limit -= 1;
       }
       // if (*pMP3Stream).nsb_limit > limit
       //    (*pMP3Stream).nsb_limit = limit;
       // if (*pMP3Stream).nsb_limit > (*pMP3Stream).max_sb
       //    (*pMP3Stream).nsb_limit = (*pMP3Stream).max_sb;

       // (*pMP3Stream).outvalues = 1152 >> reduction_code;
       if (*h).mode != 3
       {				/* adjust for 2 channel modes */
           i = 0;
           while i < 4
           {
               // (*pMP3Stream).nbat[i] *= 2;
               i += 1;
           }
           // (*pMP3Stream).max_sb *= 2;
           // (*pMP3Stream).nsb_limit *= 2;
       }

       /* set sbt function */
       k = 1 + (convert_code & 3);
       if (*h).mode == 3
       {
           k = 0;
       }
       // (*pMP3Stream).sbt = SBT_TABLE[bit_code as usize][reduction_code as usize][k as usize];
       // (*pMP3Stream).outvalues *= OUT_CHANS[k as usize];
       if bit_code != 0
       {
           // (*pMP3Stream).outbytes = (*pMP3Stream).outvalues;
       }
       else
       {
           // (*pMP3Stream).outbytes = (core::mem::size_of::<i16>() as c_int) * (*pMP3Stream).outvalues;
       }

       // decinfo.channels = OUT_CHANS[k as usize];
       // decinfo.outvalues = (*pMP3Stream).outvalues;
       // decinfo.samprate = samprate >> reduction_code;
       if bit_code != 0
       {
           // decinfo.bits = 8;
       }
       else
       {
           // decinfo.bits = (core::mem::size_of::<i16>() as c_int) * 8;
       }

       // decinfo.framebytes = (*pMP3Stream).framebytes;
       // decinfo.type = 0;



       /* clear sample buffer, unused sub bands must be 0 */
       i = 0;
       while i < 2304*2 {	// the *2 here was inserted by me just in case, since the array is now *2, because of stereo files unpacking at 4608 bytes per frame (which may or may not be relevant, but in any case I don't think we use the L1 versions of MP3 now anyway
           sample[i as usize] = 0.0f32;
           i += 1;
       }


       /* init sub-band transform */
       sbt_init();

       return 1;
   }
}

/*---------------------------------------------------------*/
#[no_mangle]
pub extern "C" fn audio_decode_info(info: *mut DEC_INFO)
{
   unsafe {
       *info = decinfo;		/* info return, call after init */
   }
}

/*---------------------------------------------------------*/
#[no_mangle]
pub extern "C" fn decode_table_init()
{
   /* dummy for asm version compatability */
}

/*---------------------------------------------------------*/
// #endif	// #ifdef COMPILE_ME
