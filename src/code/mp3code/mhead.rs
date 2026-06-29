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

	$Id: mhead.c,v 1.7 1999/10/19 07:13:09 elrod Exp $
____________________________________________________________________________*/

/*------------ mhead.c ----------------------------------------------
  mpeg audio
  extract info from mpeg header
  portable version (adapted from c:\eco\mhead.c

  add Layer III

  mods 6/18/97 re mux restart, 32 bit ints

  mod 5/7/98 parse mpeg 2.5

---------------------------------------------------------------------*/

use core::ffi::c_int;
use super::mhead_h::MPEG_HEAD;

static MP_BR_TABLE: [[c_int; 16]; 2] =
[[0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],
 [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 0]];
static MP_SR20_TABLE: [[c_int; 4]; 2] =
[[441, 480, 320, -999], [882, 960, 640, -999]];

static MP_BR_TABLEL1: [[c_int; 16]; 2] =
[[0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256, 0],/* mpeg2 */
 [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448, 0]];

static MP_BR_TABLEL3: [[c_int; 16]; 2] =
[[0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],      /* mpeg 2 */
 [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0]];



fn find_sync(buf: &[u8], n: usize) -> c_int;
fn sync_scan(buf: &[u8], n: usize, i0: usize) -> c_int;
fn sync_test(buf: &[u8], n: usize, isync: c_int, padbytes: c_int) -> c_int;


/*--------------------------------------------------------------*/
pub fn head_info(buf: &[u8], n: u32, h: &mut MPEG_HEAD) -> c_int
{
   let mut n = n;
   let mut framebytes: c_int;
   let mut mpeg25_flag: c_int;

   if n > 10000 {
      n = 10000;		/* limit scan for free format */
   }



   h.sync = 0;
   //if ((buf[0] == 0xFF) && ((buf[1] & 0xF0) == 0xF0))
   if (buf[0] == 0xFF) && ((buf[0+1] & 0xF0) == 0xF0)
   {
      mpeg25_flag = 0;		// mpeg 1 & 2

   }
   else if (buf[0] == 0xFF) && ((buf[0+1] & 0xF0) == 0xE0)
   {
      mpeg25_flag = 1;		// mpeg 2.5

   }
   else {
      return 0;			// sync fail
   }

   h.sync = 1;
   if mpeg25_flag != 0 {
      h.sync = 2;		//low bit clear signals mpeg25 (as in 0xFFE)
   }

   h.id = ((buf[0+1] & 0x08) >> 3) as c_int;
   h.option = ((buf[0+1] & 0x06) >> 1) as c_int;
   h.prot = (buf[0+1] & 0x01) as c_int;

   h.br_index = ((buf[0+2] & 0xf0) >> 4) as c_int;
   h.sr_index = ((buf[0+2] & 0x0c) >> 2) as c_int;
   h.pad = ((buf[0+2] & 0x02) >> 1) as c_int;
   h.private_bit = (buf[0+2] & 0x01) as c_int;
   h.mode = ((buf[0+3] & 0xc0) >> 6) as c_int;
   h.mode_ext = ((buf[0+3] & 0x30) >> 4) as c_int;
   h.cr = ((buf[0+3] & 0x08) >> 3) as c_int;
   h.original = ((buf[0+3] & 0x04) >> 2) as c_int;
   h.emphasis = (buf[0+3] & 0x03) as c_int;


// if( mpeg25_flag ) {
 //    if( h->sr_index == 2 ) return 0;   // fail 8khz
 //}


/* compute framebytes for Layer I, II, III */
   if h.option < 1 {
      return 0;
   }
   if h.option > 3 {
      return 0;
   }

   framebytes = 0;

   if h.br_index > 0
   {
      if h.option == 3
      {				/* layer I */
	 framebytes =
	    240 * MP_BR_TABLEL1[h.id as usize][h.br_index as usize]
	    / MP_SR20_TABLE[h.id as usize][h.sr_index as usize];
	 framebytes = 4 * framebytes;
      }
      else if h.option == 2
      {				/* layer II */
	 framebytes =
	    2880 * MP_BR_TABLE[h.id as usize][h.br_index as usize]
	    / MP_SR20_TABLE[h.id as usize][h.sr_index as usize];
      }
      else if h.option == 1
      {				/* layer III */
	 if h.id != 0
	 {			// mpeg1

	    framebytes =
	       2880 * MP_BR_TABLEL3[h.id as usize][h.br_index as usize]
	       / MP_SR20_TABLE[h.id as usize][h.sr_index as usize];
	 }
	 else
	 {			// mpeg2

	    if mpeg25_flag != 0
	    {			// mpeg2.2

	       framebytes =
		  2880 * MP_BR_TABLEL3[h.id as usize][h.br_index as usize]
		  / MP_SR20_TABLE[h.id as usize][h.sr_index as usize];
	    }
	    else
	    {
	       framebytes =
		  1440 * MP_BR_TABLEL3[h.id as usize][h.br_index as usize]
		  / MP_SR20_TABLE[h.id as usize][h.sr_index as usize];
	    }
	 }
      }
   }
   else {
      framebytes = find_sync(buf, n as usize);	/* free format */
   }

  return framebytes;
}

pub fn head_info3(buf: &[u8], n: u32, h: &mut MPEG_HEAD, br: &mut c_int, searchForward: &mut u32) -> c_int {
	let mut pBuf: u32 = 0;

	// jdw insertion...
   while (pBuf < n) && !((buf[pBuf as usize] == 0xFF) &&
          ((buf[(pBuf+1) as usize] & 0xF0) == 0xF0 || (buf[(pBuf+1) as usize] & 0xF0) == 0xE0))
   {
		pBuf += 1;
   }

   if pBuf == n {
       return 0;
   }

   *searchForward = pBuf;
   return head_info2(&buf[pBuf as usize..],n,h,br);
}

/*--------------------------------------------------------------*/
pub fn head_info2(buf: &[u8], n: u32, h: &mut MPEG_HEAD, br: &mut c_int) -> c_int
{
	let mut framebytes: c_int;

	/*---  return br (in bits/sec) in addition to frame bytes ---*/

	*br = 0;
	/*-- assume fail --*/
	framebytes = head_info(buf, n, h);

	if framebytes == 0 {
		return 0;
	}

	match h.option
	{
		1 => {	/* layer III */
			{
				if h.br_index > 0 {
					*br = 1000 * MP_BR_TABLEL3[h.id as usize][h.br_index as usize];
				} else {
					if h.id != 0 {		// mpeg1

						*br = 1000 * framebytes * MP_SR20_TABLE[h.id as usize][h.sr_index as usize] / (144 * 20);
					} else {			// mpeg2

						if (h.sync & 1) == 0 {	//  flags mpeg25

							*br = 500 * framebytes * MP_SR20_TABLE[h.id as usize][h.sr_index as usize] / (72 * 20);
						} else {
							*br = 1000 * framebytes * MP_SR20_TABLE[h.id as usize][h.sr_index as usize] / (72 * 20);
						}
					}
				}
			}
		},

		2 => {	/* layer II */
			{
				if h.br_index > 0 {
					*br = 1000 * MP_BR_TABLE[h.id as usize][h.br_index as usize];
				} else {
					*br = 1000 * framebytes * MP_SR20_TABLE[h.id as usize][h.sr_index as usize]	/ (144 * 20);
				}
			}
		},

		3 => {	/* layer I */
			{
				if h.br_index > 0 {
					*br = 1000 * MP_BR_TABLEL1[h.id as usize][h.br_index as usize];
				} else {
					*br = 1000 * framebytes * MP_SR20_TABLE[h.id as usize][h.sr_index as usize]	/ (48 * 20);
				}
			}
		},

		_ => {

			return 0;	// fuck knows what this is, but it ain't one of ours...
		}
	}


	return framebytes;
}
/*--------------------------------------------------------------*/
fn compare(buf: &[u8], buf2: &[u8]) -> c_int
{
   if buf[0] != buf2[0] {
      return 0;
   }
   if buf[1] != buf2[1] {
      return 0;
   }
   return 1;
}
/*----------------------------------------------------------*/
/*-- does not scan for initial sync, initial sync assumed --*/
fn find_sync(buf: &[u8], n: usize) -> c_int
{
   let mut i0: c_int;
   let mut isync: c_int;
   let mut nmatch: c_int;
   let mut pad: c_int;
   let mut padbytes: c_int;
   let mut option: c_int;

/* mod 4/12/95 i0 change from 72, allows as low as 8kbits for mpeg1 */
   i0 = 24;
   padbytes = 1;
   option = ((buf[1] & 0x06) >> 1) as c_int;
   if option == 3
   {
      padbytes = 4;
      i0 = 24;			/* for shorter layer I frames */
   }

   pad = ((buf[2] & 0x02) >> 1) as c_int;

   let mut n = (n - 3) as c_int;			/*  need 3 bytes of header  */

   while i0 < 2000
   {
      isync = sync_scan(buf, (n + 3) as usize, i0 as usize);
      i0 = isync + 1;
      isync -= pad;
      if isync <= 0 {
	 return 0;
      }
      nmatch = sync_test(buf, (n + 3) as usize, isync, padbytes);
      if nmatch > 0 {
	 return isync;
      }
   }

   return 0;
}
/*------------------------------------------------------*/
/*---- scan for next sync, assume start is valid -------*/
/*---- return number bytes to next sync ----------------*/
fn sync_scan(buf: &[u8], n: usize, i0: usize) -> c_int
{
   let mut i: usize;

   i = i0;
   while i < n {
      if compare(&buf[0..], &buf[i..]) != 0 {
	 return i as c_int;
      }
      i += 1;
   }

   return 0;
}
/*------------------------------------------------------*/
/*- test consecutative syncs, input isync without pad --*/
fn sync_test(buf: &[u8], n: usize, isync: c_int, padbytes: c_int) -> c_int
{
   let mut i: c_int;
   let mut nmatch: c_int;
   let mut pad: c_int;

   nmatch = 0;
   i = 0;
   loop {
      pad = padbytes * (((buf[(i + 2) as usize] & 0x02) >> 1) as c_int);
      i += (pad + isync);
      if i > (n as c_int) {
	 break;
      }
      if compare(&buf[0..], &buf[i as usize..]) == 0 {
	 return -nmatch;
      }
      nmatch += 1;
   }
   return nmatch;
}
