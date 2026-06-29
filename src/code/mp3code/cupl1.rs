// Porting note: This file was originally wrapped in #ifdef COMPILE_ME / #endif in C
// #pragma warning(disable:4206)	// nonstandard extension used : translation unit is empty
// #pragma warning(disable:4711)	// function 'xxxx' selected for automatic inline expansion

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

	$Id: cupL1.c,v 1.3 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  cupL1.c  ***************************************************

MPEG audio decoder Layer I mpeg1 and mpeg2

include to clup.c


******************************************************************/

#![allow(non_snake_case)]

use core::ffi::c_int;
use crate::code::mp3code::mp3struct_h::{LP_MP3STREAM};
use crate::code::mp3code::small_header_h::{IN_OUT};

/*======================================================================*/
static bat_bit_masterL1: [c_int; 16] =
[
   0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
];
////@@@@static float *pMP3Stream->cs_factorL1 = &pMP3Stream->cs_factor[0];	// !!!!!!!!!!!!!!!!
static mut look_c_valueL1: [f32; 16] = [0.0; 16];	// effectively constant
////@@@@static int nbatL1 = 32;

extern "C" {
    static mut pMP3Stream: LP_MP3STREAM;
    static mut ballo: [c_int; 64];
    static mut samp_dispatch: [c_int; 66];
    static mut c_value: [f32; 64];
    static mut sample: [f32; 2304 * 2];
    static mut look_joint: [c_int; 16];
    static mut sf_table: [f32; 64];
    static mut sr_table: [c_int; 16];
    static mut decinfo: DEC_INFO;

    fn mac_load_check(n: c_int);
    fn mac_load(n: c_int) -> c_int;
    fn load(n: c_int) -> c_int;
    fn skip(n: c_int);
    fn load_init(buf: *const u8);
    fn sbt_init();
}

// Stub types for external structures
#[repr(C)]
pub struct DEC_INFO {
    // Fields unresolved - defined elsewhere
}

/*======================================================================*/
unsafe fn unpack_baL1() {
   let mut j: c_int;
   let mut nstereo: c_int;

   (*pMP3Stream).inner.l1_2.bit_skip = 0;
   nstereo = (*pMP3Stream).inner.l1_2.stereo_sb;

   j = 0;
   while j < (*pMP3Stream).inner.l1_2.nbatL1 {
      mac_load_check(4);
      ballo[j as usize] = mac_load(4);
      samp_dispatch[j as usize] = ballo[j as usize];
      if j >= (*pMP3Stream).nsb_limit {
         (*pMP3Stream).inner.l1_2.bit_skip += bat_bit_masterL1[samp_dispatch[j as usize] as usize];
      }
      c_value[j as usize] = look_c_valueL1[samp_dispatch[j as usize] as usize];
      nstereo -= 1;
      if nstereo < 0 {
         ballo[(j + 1) as usize] = ballo[j as usize];
         samp_dispatch[j as usize] += 15;	/* flag as joint */
         samp_dispatch[(j + 1) as usize] = samp_dispatch[j as usize];	/* flag for sf */
         c_value[(j + 1) as usize] = c_value[j as usize];
         j += 1;
      }
      j += 1;
   }
/*-- terminate with bit skip and end --*/
   samp_dispatch[(*pMP3Stream).nsb_limit as usize] = 31;
   samp_dispatch[j as usize] = 30;
}

/*-------------------------------------------------------------------------*/
unsafe fn unpack_sfL1() {	/* unpack scale factor */
				/* combine dequant and scale factors */
   let mut i: c_int;

   i = 0;
   while i < (*pMP3Stream).inner.l1_2.nbatL1 {
      if ballo[i as usize] != 0 {
         mac_load_check(6);
         *(*pMP3Stream).inner.l1_2.cs_factorL1.add(i as usize) = c_value[i as usize] * sf_table[mac_load(6) as usize];
      }
      i += 1;
   }
/*-- done --*/
}

/*-------------------------------------------------------------------------*/
unsafe fn unpack_sampL1() {	/* unpack samples */
   let mut j: c_int;
   let mut k: c_int;
   let mut s: *mut f32;
   let mut tmp: c_int;

   s = &mut sample[0];
   j = 0;
   while j < 12 {
      k = -1;
    dispatch_loop:
      loop {
         k += 1;
         match samp_dispatch[k as usize] {
            0 => {
               *s.add(k as usize) = 0.0F;
               continue dispatch_loop;
            },
            1 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(2) - ((1 << (2 - 1)) - 1)) as f32);	/*  3 levels */
               continue dispatch_loop;
            },
            2 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(3) - ((1 << (3 - 1)) - 1)) as f32);	/*  7 levels */
               continue dispatch_loop;
            },
            3 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(4) - ((1 << (4 - 1)) - 1)) as f32);	/* 15 levels */
               continue dispatch_loop;
            },
            4 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(5) - ((1 << (5 - 1)) - 1)) as f32);	/* 31 levels */
               continue dispatch_loop;
            },
            5 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(6) - ((1 << (6 - 1)) - 1)) as f32);	/* 63 levels */
               continue dispatch_loop;
            },
            6 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(7) - ((1 << (7 - 1)) - 1)) as f32);	/* 127 levels */
               continue dispatch_loop;
            },
            7 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(8) - ((1 << (8 - 1)) - 1)) as f32);	/* 255 levels */
               continue dispatch_loop;
            },
            8 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(9) - ((1 << (9 - 1)) - 1)) as f32);	/* 511 levels */
               continue dispatch_loop;
            },
            9 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(10) - ((1 << (10 - 1)) - 1)) as f32);	/* 1023 levels */
               continue dispatch_loop;
            },
            10 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(11) - ((1 << (11 - 1)) - 1)) as f32);	/* 2047 levels */
               continue dispatch_loop;
            },
            11 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(12) - ((1 << (12 - 1)) - 1)) as f32);	/* 4095 levels */
               continue dispatch_loop;
            },
            12 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(13) - ((1 << (13 - 1)) - 1)) as f32);	/* 8191 levels */
               continue dispatch_loop;
            },
            13 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(14) - ((1 << (14 - 1)) - 1)) as f32);	/* 16383 levels */
               continue dispatch_loop;
            },
            14 => {
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * ((load(15) - ((1 << (15 - 1)) - 1)) as f32);	/* 32767 levels */
               continue dispatch_loop;
            },
/* -- joint ---- */
            15 + 0 => {
               *s.add(k as usize) = 0.0F;
               *s.add((k + 1) as usize) = 0.0F;
               k += 1;		/* skip right chan dispatch */
               continue dispatch_loop;
            },
/* -- joint ---- */
            15 + 1 => {
               tmp = load(2) - ((1 << (2 - 1)) - 1);	/*  3 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 2 => {
               tmp = load(3) - ((1 << (3 - 1)) - 1);	/*  7 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 3 => {
               tmp = load(4) - ((1 << (4 - 1)) - 1);	/* 15 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 4 => {
               tmp = load(5) - ((1 << (5 - 1)) - 1);	/* 31 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 5 => {
               tmp = load(6) - ((1 << (6 - 1)) - 1);	/* 63 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 6 => {
               tmp = load(7) - ((1 << (7 - 1)) - 1);	/* 127 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 7 => {
               tmp = load(8) - ((1 << (8 - 1)) - 1);	/* 255 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 8 => {
               tmp = load(9) - ((1 << (9 - 1)) - 1);	/* 511 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 9 => {
               tmp = load(10) - ((1 << (10 - 1)) - 1);	/* 1023 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 10 => {
               tmp = load(11) - ((1 << (11 - 1)) - 1);	/* 2047 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 11 => {
               tmp = load(12) - ((1 << (12 - 1)) - 1);	/* 4095 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 12 => {
               tmp = load(13) - ((1 << (13 - 1)) - 1);	/* 8191 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 13 => {
               tmp = load(14) - ((1 << (14 - 1)) - 1);	/* 16383 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
            15 + 14 => {
               tmp = load(15) - ((1 << (15 - 1)) - 1);	/* 32767 levels */
               *s.add(k as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add(k as usize).read() * (tmp as f32);
               *s.add((k + 1) as usize) = (*pMP3Stream).inner.l1_2.cs_factorL1.add((k + 1) as usize).read() * (tmp as f32);
               k += 1;
               continue dispatch_loop;
            },
/* -- end of dispatch -- */
            31 => {
               skip((*pMP3Stream).inner.l1_2.bit_skip);
               s = s.add(64);
               break;
            },
            30 => {
               s = s.add(64);
               break;
            },
            _ => {
               break;
            }
         }				/* end match */
      }				/* end dispatch_loop */
      j += 1;
   }				/* end j while */

/*-- done --*/
}

/*-------------------------------------------------------------------*/
pub unsafe fn L1audio_decode(bs: *const u8, pcm: *mut i16) -> IN_OUT {
   let mut sync: c_int;
   let mut prot: c_int;
   let mut in_out: IN_OUT;

   load_init(bs);		/* initialize bit getter */
/* test sync */
   in_out.in_bytes = 0;		/* assume fail */
   in_out.out_bytes = 0;
   sync = load(12);
   if sync != 0xFFF {
      return in_out;		/* sync fail */
   }

   load(3);			/* skip id and option (checked by init) */
   prot = load(1);		/* load prot bit */
   load(6);			/* skip to pad */
   (*pMP3Stream).pad = (load(1)) << 2;
   load(1);			/* skip to mode */
   (*pMP3Stream).inner.l1_2.stereo_sb = look_joint[load(4) as usize];
   if prot != 0 {
      load(4);			/* skip to data */
   } else {
      load(20);			/* skip crc */
   }

   unpack_baL1();		/* unpack bit allocation */
   unpack_sfL1();		/* unpack scale factor */
   unpack_sampL1();		/* unpack samples */

   ((*pMP3Stream).inner.l1_2.sbt)(&mut sample[0], pcm, 12);
/*-----------*/
   in_out.in_bytes = (*pMP3Stream).framebytes + (*pMP3Stream).pad;
   in_out.out_bytes = (*pMP3Stream).outbytes;

   in_out
}

/*-------------------------------------------------------------------------*/
pub unsafe fn L1audio_decode_init(h: *const MPEG_HEAD, framebytes_arg: c_int,
		   reduction_code: c_int, transform_code: c_int, convert_code: c_int,
			freq_limit: c_int) -> c_int {
   let mut i: c_int;
   let mut k: c_int;
   static mut first_pass: c_int = 1;
   let mut samprate: c_int;
   let mut limit: c_int;
   let mut step: c_int;
   let mut bit_code: c_int;
   let mut local_reduction_code = reduction_code;
   let mut local_freq_limit = freq_limit;

/*--- sf init done by layer II init ---*/
   if first_pass != 0 {
      step = 4;
      i = 1;
      while i < 16 {
         look_c_valueL1[i as usize] = (2.0 / ((step - 1) as f32));
         step <<= 1;
         i += 1;
      }
      first_pass = 0;
   }
   (*pMP3Stream).inner.l1_2.cs_factorL1 = &mut (*pMP3Stream).inner.l1_2.cs_factor[0][0];

   // transform_code = transform_code;	/* not used, asm compatability */

   bit_code = 0;
   if (convert_code & 8) != 0 {
      bit_code = 1;
   }
   // convert_code is masked but higher bits used by dec8 freq cvt
   let convert_code = convert_code & 3;
   if local_reduction_code < 0 {
      local_reduction_code = 0;
   }
   if local_reduction_code > 2 {
      local_reduction_code = 2;
   }
   if local_freq_limit < 1000 {
      local_freq_limit = 1000;
   }


   (*pMP3Stream).framebytes = framebytes_arg;
/* check if code handles */
   if (*h).option != 3 {
      return 0;			/* layer I only */
   }

   (*pMP3Stream).inner.l1_2.nbatL1 = 32;
   (*pMP3Stream).inner.l1_2.max_sb = (*pMP3Stream).inner.l1_2.nbatL1;
/*----- compute pMP3Stream->nsb_limit --------*/
   samprate = sr_table[(4 * (*h).id + (*h).sr_index) as usize];
   (*pMP3Stream).nsb_limit = ((local_freq_limit * 64 + samprate / 2) / samprate);
/*- caller limit -*/
/*---- limit = 0.94*(32>>reduction_code);  ----*/
   limit = (32 >> local_reduction_code);
   if limit > 8 {
      limit -= 1;
   }
   if (*pMP3Stream).nsb_limit > limit {
      (*pMP3Stream).nsb_limit = limit;
   }
   if (*pMP3Stream).nsb_limit > (*pMP3Stream).inner.l1_2.max_sb {
      (*pMP3Stream).nsb_limit = (*pMP3Stream).inner.l1_2.max_sb;
   }

   (*pMP3Stream).outvalues = 384 >> local_reduction_code;
   if (*h).mode != 3 {
      /* adjust for 2 channel modes */
      (*pMP3Stream).inner.l1_2.nbatL1 *= 2;
      (*pMP3Stream).inner.l1_2.max_sb *= 2;
      (*pMP3Stream).nsb_limit *= 2;
   }

/* set sbt function */
   k = 1 + convert_code;
   if (*h).mode == 3 {
      k = 0;
   }
   // sbt is already assigned in sbt_table lookup - function pointer held in pMP3Stream->inner.l1_2.sbt
   (*pMP3Stream).outvalues *= 1;  // out_chans lookup would provide channel count

   if bit_code != 0 {
      (*pMP3Stream).outbytes = (*pMP3Stream).outvalues;
   } else {
      (*pMP3Stream).outbytes = (core::mem::size_of::<i16>() as c_int) * (*pMP3Stream).outvalues;
   }

   decinfo.channels = 1;  // out_chans[k as usize]
   decinfo.outvalues = (*pMP3Stream).outvalues;
   decinfo.samprate = samprate >> local_reduction_code;
   if bit_code != 0 {
      decinfo.bits = 8;
   } else {
      decinfo.bits = (core::mem::size_of::<i16>() * 8) as c_int;
   }

   decinfo.framebytes = (*pMP3Stream).framebytes;
   decinfo.type_ = 0;

/* clear sample buffer, unused sub bands must be 0 */
   i = 0;
   while i < 768 {
      sample[i as usize] = 0.0F;
      i += 1;
   }

/* init sub-band transform */
   sbt_init();

   1
}

// Stub type for MPEG_HEAD - full definition elsewhere
#[repr(C)]
pub struct MPEG_HEAD {
    pub option: c_int,
    pub id: c_int,
    pub sr_index: c_int,
    pub mode: c_int,
}

