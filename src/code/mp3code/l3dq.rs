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

	$Id: l3dq.c,v 1.6 1999/10/19 07:13:09 elrod Exp $
____________________________________________________________________________*/

/****  quant.c  ***************************************************
  Layer III  dequant

  does reordering of short blocks

  mod 8/19/98 decode 22 sf bands

******************************************************************/

use core::ffi::c_int;
use std::ptr::addr_of_mut;

// External types from L3.h and mp3struct.h
#[repr(C)]
pub struct SAMPLE {
    pub s: c_int,
    pub x: f32,
}

#[repr(C)]
pub struct SCALEFACT {
    pub l: [c_int; 23],
    pub s: [[c_int; 14]; 3],
}

#[repr(C)]
pub struct GR {
    pub block_type: c_int,
    pub mixed_block_flag: c_int,
    pub global_gain: c_int,
    pub scalefac_scale: c_int,
    pub preflag: c_int,
    pub subblock_gain: [c_int; 3],
}

#[repr(C)]
pub struct CB_INFO {
    pub cbs0: c_int,
    pub ncbl: c_int,
    pub cbmax: c_int,
    pub cbtype: c_int,
    pub cbmax_s: [c_int; 3],
}

#[repr(C)]
pub struct MP3Stream {
    pub nBand: [[c_int; 14]; 2],
}

extern "C" {
    pub static mut pMP3Stream: *mut MP3Stream;
}

/*----------
static struct  {
int l[23];
int s[14];} sfBandTable[3] =
{{{0,4,8,12,16,20,24,30,36,44,52,62,74,90,110,134,162,196,238,288,342,418,576},
 {0,4,8,12,16,22,30,40,52,66,84,106,136,192}},
{{0,4,8,12,16,20,24,30,36,42,50,60,72,88,106,128,156,190,230,276,330,384,576},
 {0,4,8,12,16,22,28,38,50,64,80,100,126,192}},
{{0,4,8,12,16,20,24,30,36,44,54,66,82,102,126,156,194,240,296,364,448,550,576},
 {0,4,8,12,16,22,30,42,58,78,104,138,180,192}}};
----------*/

/*--------------------------------*/
#[allow(non_upper_case_globals)]
static pretab: [[c_int; 22]; 2] =
[
   [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
   [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 3, 3, 2, 0],
];


////@@@@extern int nBand[2][22];	/* long = nBand[0][i], short = nBand[1][i] */

/* 8 bit plus 2 lookup x = pow(2.0, 0.25*(global_gain-210)) */
/* two extra slots to do 1/sqrt(2) scaling for ms */
/* 4 extra slots to do 1/2 scaling for cvt to mono */
static mut look_global: [f32; 256 + 2 + 4] = [0.0; 262];		// effectively constant

/*-------- scaling lookup
x = pow(2.0, -0.5*(1+scalefact_scale)*scalefac + preemp)
look_scale[scalefact_scale][preemp][scalefac]
-----------------------*/
static mut look_scale: [[[f32; 32]; 4]; 2] = [[[0.0; 32]; 4]; 2];			// effectively constant
pub type LS = [[f32; 32]; 4];


/*--- iSample**(4/3) lookup, -32<=i<=31 ---*/
const ISMAX: usize = 32;
static mut look_pow: [f32; 2 * ISMAX] = [0.0; 64];			// effectively constant

/*-- pow(2.0, -0.25*8.0*subblock_gain) --*/
static mut look_subblock: [f32; 8] = [0.0; 8];				// effectively constant

/*-- reorder buffer ---*/
static mut re_buf: [[f32; 3]; 192] = [[0.0; 3]; 192];				// used by dequant() below, but only during func (as workspace)
pub type ARRAY3 = [f32; 3];


/*=============================================================*/
pub fn quant_init_global_addr() -> *mut f32
{
   unsafe { addr_of_mut!(look_global) as *mut f32 }
}
/*-------------------------------------------------------------*/
pub fn quant_init_scale_addr() -> *mut LS
{
   unsafe { addr_of_mut!(look_scale) as *mut LS }
}
/*-------------------------------------------------------------*/
pub fn quant_init_pow_addr() -> *mut f32
{
   unsafe { addr_of_mut!(look_pow) as *mut f32 }
}
/*-------------------------------------------------------------*/
pub fn quant_init_subblock_addr() -> *mut f32
{
   unsafe { addr_of_mut!(look_subblock) as *mut f32 }
}
/*=============================================================*/

pub unsafe fn dequant(Sample: *mut SAMPLE, nsamp: *mut c_int,
	     sf: *mut SCALEFACT,
	     gr: *mut GR,
	     cb_info: *mut CB_INFO, ncbl_mixed: c_int)
{
   let mut i: c_int;
   let mut j: c_int;
   let mut cb: c_int;
   let mut n: c_int;
   let mut w: c_int;
   let mut x0: f32;
   let mut xs: f32;
   let mut xsb: [f32; 3];
   let mut tmp: f64;
   let mut ncbl: c_int;
   let mut cbs0: c_int;
   let mut buf: *mut ARRAY3;			/* short block reorder */
   let mut nbands: c_int;
   let mut i0: c_int;
   let mut non_zero: c_int;
   let mut cbmax: [c_int; 3];

   nbands = *nsamp;


   ncbl = 22;			/* long block cb end */
   cbs0 = 12;			/* short block cb start */
/* ncbl_mixed = 8 or 6  mpeg1 or 2 */
   if (*gr).block_type == 2
   {
      ncbl = 0;
      cbs0 = 0;
      if (*gr).mixed_block_flag != 0
      {
	 ncbl = ncbl_mixed;
	 cbs0 = 3;
      }
   }
/* fill in cb_info -- */
   /* This doesn't seem used anywhere...
   cb_info->lb_type = gr->block_type;
   if (gr->block_type == 2)
      cb_info->lb_type;
   */
   (*cb_info).cbs0 = cbs0;
   (*cb_info).ncbl = ncbl;

   cbmax[2] = 0;
   cbmax[1] = 0;
   cbmax[0] = 0;
/* global gain pre-adjusted by 2 if ms_mode, 0 otherwise */
   x0 = look_global[((2 + 4) + (*gr).global_gain) as usize];
   i = 0;
/*----- long blocks ---*/
   cb = 0;
   while cb < ncbl {
      non_zero = 0;
      xs = x0 * look_scale[(*gr).scalefac_scale as usize][pretab[(*gr).preflag as usize][cb as usize] as usize][(*sf).l[cb as usize] as usize];
      n = (*(*pMP3Stream)).nBand[0][cb as usize];
      j = 0;
      while j < n {
	 if (*Sample.offset(i as isize)).s == 0
	 {
	    (*Sample.offset(i as isize)).x = 0.0;
	 }
	 else
	 {
	    non_zero = 1;
	    if (((*Sample.offset(i as isize)).s >= (-ISMAX as c_int)) && ((*Sample.offset(i as isize)).s < ISMAX as c_int))
	       (*Sample.offset(i as isize)).x = xs * look_pow[(ISMAX as c_int + (*Sample.offset(i as isize)).s) as usize];
	    else
	    {
		let tmpConst: f32 = 1.0/3.0;
	       tmp = (*Sample.offset(i as isize)).s as f64;
	       (*Sample.offset(i as isize)).x = (xs * tmp as f32 * (tmp.abs()).powf(tmpConst as f64)) as f32;
	    }
	 }
	 j += 1;
	 i += 1;
      }
      if non_zero != 0
	 cbmax[0] = cb;
      if i >= nbands
	 break;
      cb += 1;
   }

   (*cb_info).cbmax = cbmax[0];
   (*cb_info).cbtype = 0;		// type = long

   if cbs0 >= 12
      return;
/*---------------------------
block type = 2  short blocks
----------------------------*/
   cbmax[2] = cbs0;
   cbmax[1] = cbs0;
   cbmax[0] = cbs0;
   i0 = i;			/* save for reorder */
   buf = re_buf.as_mut_ptr();
   w = 0;
   while w < 3 {
      xsb[w as usize] = x0 * look_subblock[(*gr).subblock_gain[w as usize] as usize];
      w += 1;
   }
   cb = cbs0;
   while cb < 13 {
      n = (*(*pMP3Stream)).nBand[1][cb as usize];
      w = 0;
      while w < 3 {
	 non_zero = 0;
	 xs = xsb[w as usize] * look_scale[(*gr).scalefac_scale as usize][0][(*sf).s[w as usize][cb as usize] as usize];
	 j = 0;
	 while j < n {
	    if (*Sample.offset(i as isize)).s == 0
	       (*buf).offset(j as isize)[w as usize] = 0.0;
	    else
	    {
	       non_zero = 1;
	       if (((*Sample.offset(i as isize)).s >= (-ISMAX as c_int)) && ((*Sample.offset(i as isize)).s < ISMAX as c_int))
		  (*buf).offset(j as isize)[w as usize] = xs * look_pow[(ISMAX as c_int + (*Sample.offset(i as isize)).s) as usize];
	       else
	       {
		  let tmpConst: f32 = 1.0/3.0;
		  tmp = (*Sample.offset(i as isize)).s as f64;
		  (*buf).offset(j as isize)[w as usize] = (xs * tmp as f32 * (tmp.abs()).powf(tmpConst as f64)) as f32;
	       }
	    }
	    j += 1;
	    i += 1;
	 }
	 if non_zero != 0
	    cbmax[w as usize] = cb;
	 w += 1;
      }
      if i >= nbands
	 break;
      cb += 1;
      buf = buf.offset(n as isize);
   }


   let src = addr_of_mut!(re_buf) as *const f32;
   let dst = &mut (*Sample.offset(i0 as isize)).x as *mut f32;
   std::ptr::copy_nonoverlapping(src, dst, (i - i0) as usize);

   *nsamp = i;			/* update nsamp */
   (*cb_info).cbmax_s[0] = cbmax[0];
   (*cb_info).cbmax_s[1] = cbmax[1];
   (*cb_info).cbmax_s[2] = cbmax[2];
   if cbmax[1] > cbmax[0]
      cbmax[0] = cbmax[1];
   if cbmax[2] > cbmax[0]
      cbmax[0] = cbmax[2];

   (*cb_info).cbmax = cbmax[0];
   (*cb_info).cbtype = 1;		/* type = short */


   return;
}
