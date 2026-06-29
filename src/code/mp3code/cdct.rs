#![allow(non_snake_case)]

use core::ptr::addr_of;

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

	$Id: cdct.c,v 1.11 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  cdct.c  ***************************************************

mod 5/16/95 first stage in 8 pt dct does not drop last sb mono


MPEG audio decoder, dct
portable C

******************************************************************/

pub static mut coef32: [f32; 31] = [0.0; 31];	/* 32 pt dct coefs */		// !!!!!!!!!!!!!!!!!! (only generated once (always to same value)

/*------------------------------------------------------------*/
pub fn dct_coef_addr() -> *mut f32
{
   unsafe { (addr_of!(coef32) as *mut f32) }
}
/*------------------------------------------------------------*/
unsafe fn forward_bf(m: i32, n: i32, x: *const f32, f: *mut f32, coef: *const f32)
{
   let mut i: i32;
   let mut j: i32;
   let mut n2: i32;
   let mut p: i32;
   let mut q: i32;
   let mut p0: i32;
   let mut k: i32;

   p0 = 0;
   n2 = n >> 1;
   i = 0;
   while i < m
   {
      k = 0;
      p = p0;
      q = p + n - 1;
      j = 0;
      while j < n2
      {
	 *f.add(p as usize) = *x.add(p as usize) + *x.add(q as usize);
	 *f.add((n2 + p) as usize) = *coef.add(k as usize) * (*x.add(p as usize) - *x.add(q as usize));
	 p += 1;
	 q -= 1;
	 k += 1;
	 j += 1;
      }
      p0 += n;
      i += 1;
   }
}
/*------------------------------------------------------------*/
unsafe fn back_bf(m: i32, n: i32, x: *const f32, f: *mut f32)
{
   let mut i: i32;
   let mut j: i32;
   let mut n2: i32;
   let mut n21: i32;
   let mut p: i32;
   let mut q: i32;
   let mut p0: i32;

   p0 = 0;
   n2 = n >> 1;
   n21 = n2 - 1;
   i = 0;
   while i < m
   {
      p = p0;
      q = p0;
      j = 0;
      while j < n2
      {
	 *f.add(p as usize) = *x.add(q as usize);
	 p += 2;
	 q += 1;
	 j += 1;
      }
      p = p0 + 1;
      j = 0;
      while j < n21
      {
	 *f.add(p as usize) = *x.add(q as usize) + *x.add((q + 1) as usize);
	 p += 2;
	 q += 1;
	 j += 1;
      }
      *f.add(p as usize) = *x.add(q as usize);
      p0 += n;
      i += 1;
   }
}
/*------------------------------------------------------------*/


pub unsafe fn fdct32(x: *const f32, c: *mut f32)
{
   let mut a: [f32; 32] = [0.0; 32];			/* ping pong buffers */
   let mut b: [f32; 32] = [0.0; 32];
   let mut p: i32;
   let mut q: i32;

   let src = x;

/* special first stage */
   p = 0;
   q = 31;
   while p < 16
   {
      a[p as usize] = *src.add(p as usize) + *src.add(q as usize);
      a[(16 + p) as usize] = coef32[p as usize] * (*src.add(p as usize) - *src.add(q as usize));
      p += 1;
      q -= 1;
   }
   forward_bf(2, 16, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16));
   forward_bf(4, 8, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8));
   forward_bf(8, 4, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4));
   forward_bf(16, 2, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4 + 2));
   back_bf(8, 4, a.as_ptr(), b.as_mut_ptr());
   back_bf(4, 8, b.as_ptr(), a.as_mut_ptr());
   back_bf(2, 16, a.as_ptr(), b.as_mut_ptr());
   back_bf(1, 32, b.as_ptr(), c);
}
/*------------------------------------------------------------*/
pub unsafe fn fdct32_dual(x: *const f32, c: *mut f32)
{
   let mut a: [f32; 32] = [0.0; 32];			/* ping pong buffers */
   let mut b: [f32; 32] = [0.0; 32];
   let mut p: i32;
   let mut pp: i32;
   let mut qq: i32;

/* special first stage for dual chan (interleaved x) */
   pp = 0;
   qq = 2 * 31;
   p = 0;
   while p < 16
   {
      a[p as usize] = *x.add(pp as usize) + *x.add(qq as usize);
      a[(16 + p) as usize] = coef32[p as usize] * (*x.add(pp as usize) - *x.add(qq as usize));
      pp += 2;
      qq -= 2;
      p += 1;
   }
   forward_bf(2, 16, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16));
   forward_bf(4, 8, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8));
   forward_bf(8, 4, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4));
   forward_bf(16, 2, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4 + 2));
   back_bf(8, 4, a.as_ptr(), b.as_mut_ptr());
   back_bf(4, 8, b.as_ptr(), a.as_mut_ptr());
   back_bf(2, 16, a.as_ptr(), b.as_mut_ptr());
   back_bf(1, 32, b.as_ptr(), c);
}
/*---------------convert dual to mono------------------------------*/
pub unsafe fn fdct32_dual_mono(x: *const f32, c: *mut f32)
{
   let mut a: [f32; 32] = [0.0; 32];			/* ping pong buffers */
   let mut b: [f32; 32] = [0.0; 32];
   let mut t1: f32;
   let mut t2: f32;
   let mut p: i32;
   let mut pp: i32;
   let mut qq: i32;

/* special first stage  */
   pp = 0;
   qq = 2 * 31;
   p = 0;
   while p < 16
   {
      t1 = 0.5 * (*x.add(pp as usize) + *x.add((pp + 1) as usize));
      t2 = 0.5 * (*x.add(qq as usize) + *x.add((qq + 1) as usize));
      a[p as usize] = t1 + t2;
      a[(16 + p) as usize] = coef32[p as usize] * (t1 - t2);
      pp += 2;
      qq -= 2;
      p += 1;
   }
   forward_bf(2, 16, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16));
   forward_bf(4, 8, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8));
   forward_bf(8, 4, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4));
   forward_bf(16, 2, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4 + 2));
   back_bf(8, 4, a.as_ptr(), b.as_mut_ptr());
   back_bf(4, 8, b.as_ptr(), a.as_mut_ptr());
   back_bf(2, 16, a.as_ptr(), b.as_mut_ptr());
   back_bf(1, 32, b.as_ptr(), c);
}
/*------------------------------------------------------------*/
/*---------------- 16 pt fdct -------------------------------*/
pub unsafe fn fdct16(x: *const f32, c: *mut f32)
{
   let mut a: [f32; 16] = [0.0; 16];			/* ping pong buffers */
   let mut b: [f32; 16] = [0.0; 16];
   let mut p: i32;
   let mut q: i32;

/* special first stage (drop highest sb) */
   a[0] = *x.add(0);
   a[8] = coef32[16] * *x.add(0);
   p = 1;
   q = 14;
   while p < 8
   {
      a[p as usize] = *x.add(p as usize) + *x.add(q as usize);
      a[(8 + p) as usize] = coef32[(16 + p) as usize] * (*x.add(p as usize) - *x.add(q as usize));
      p += 1;
      q -= 1;
   }
   forward_bf(2, 8, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8));
   forward_bf(4, 4, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4));
   forward_bf(8, 2, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4 + 2));
   back_bf(4, 4, b.as_ptr(), a.as_mut_ptr());
   back_bf(2, 8, a.as_ptr(), b.as_mut_ptr());
   back_bf(1, 16, b.as_ptr(), c);
}
/*------------------------------------------------------------*/
/*---------------- 16 pt fdct dual chan---------------------*/
pub unsafe fn fdct16_dual(x: *const f32, c: *mut f32)
{
   let mut a: [f32; 16] = [0.0; 16];			/* ping pong buffers */
   let mut b: [f32; 16] = [0.0; 16];
   let mut p: i32;
   let mut pp: i32;
   let mut qq: i32;

/* special first stage for interleaved input */
   a[0] = *x.add(0);
   a[8] = coef32[16] * *x.add(0);
   pp = 2;
   qq = 2 * 14;
   p = 1;
   while p < 8
   {
      a[p as usize] = *x.add(pp as usize) + *x.add(qq as usize);
      a[(8 + p) as usize] = coef32[(16 + p) as usize] * (*x.add(pp as usize) - *x.add(qq as usize));
      pp += 2;
      qq -= 2;
      p += 1;
   }
   forward_bf(2, 8, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8));
   forward_bf(4, 4, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4));
   forward_bf(8, 2, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4 + 2));
   back_bf(4, 4, b.as_ptr(), a.as_mut_ptr());
   back_bf(2, 8, a.as_ptr(), b.as_mut_ptr());
   back_bf(1, 16, b.as_ptr(), c);
}
/*------------------------------------------------------------*/
/*---------------- 16 pt fdct dual to mono-------------------*/
pub unsafe fn fdct16_dual_mono(x: *const f32, c: *mut f32)
{
   let mut a: [f32; 16] = [0.0; 16];			/* ping pong buffers */
   let mut b: [f32; 16] = [0.0; 16];
   let mut t1: f32;
   let mut t2: f32;
   let mut p: i32;
   let mut pp: i32;
   let mut qq: i32;

/* special first stage  */
   a[0] = 0.5 * (*x.add(0) + *x.add(1));
   a[8] = coef32[16] * a[0];
   pp = 2;
   qq = 2 * 14;
   p = 1;
   while p < 8
   {
      t1 = 0.5 * (*x.add(pp as usize) + *x.add((pp + 1) as usize));
      t2 = 0.5 * (*x.add(qq as usize) + *x.add((qq + 1) as usize));
      a[p as usize] = t1 + t2;
      a[(8 + p) as usize] = coef32[(16 + p) as usize] * (t1 - t2);
      pp += 2;
      qq -= 2;
      p += 1;
   }
   forward_bf(2, 8, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8));
   forward_bf(4, 4, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4));
   forward_bf(8, 2, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4 + 2));
   back_bf(4, 4, b.as_ptr(), a.as_mut_ptr());
   back_bf(2, 8, a.as_ptr(), b.as_mut_ptr());
   back_bf(1, 16, b.as_ptr(), c);
}
/*------------------------------------------------------------*/
/*---------------- 8 pt fdct -------------------------------*/
pub unsafe fn fdct8(x: *const f32, c: *mut f32)
{
   let mut a: [f32; 8] = [0.0; 8];			/* ping pong buffers */
   let mut b: [f32; 8] = [0.0; 8];
   let mut p: i32;
   let mut q: i32;

/* special first stage  */

   b[0] = *x.add(0) + *x.add(7);
   b[4] = coef32[16 + 8] * (*x.add(0) - *x.add(7));
   p = 1;
   q = 6;
   while p < 4
   {
      b[p as usize] = *x.add(p as usize) + *x.add(q as usize);
      b[(4 + p) as usize] = coef32[(16 + 8 + p) as usize] * (*x.add(p as usize) - *x.add(q as usize));
      p += 1;
      q -= 1;
   }

   forward_bf(2, 4, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4));
   forward_bf(4, 2, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4 + 2));
   back_bf(2, 4, b.as_ptr(), a.as_mut_ptr());
   back_bf(1, 8, a.as_ptr(), c);
}
/*------------------------------------------------------------*/
/*---------------- 8 pt fdct dual chan---------------------*/
pub unsafe fn fdct8_dual(x: *const f32, c: *mut f32)
{
   let mut a: [f32; 8] = [0.0; 8];			/* ping pong buffers */
   let mut b: [f32; 8] = [0.0; 8];
   let mut p: i32;
   let mut pp: i32;
   let mut qq: i32;

/* special first stage for interleaved input */
   b[0] = *x.add(0) + *x.add(14);
   b[4] = coef32[16 + 8] * (*x.add(0) - *x.add(14));
   pp = 2;
   qq = 2 * 6;
   p = 1;
   while p < 4
   {
      b[p as usize] = *x.add(pp as usize) + *x.add(qq as usize);
      b[(4 + p) as usize] = coef32[(16 + 8 + p) as usize] * (*x.add(pp as usize) - *x.add(qq as usize));
      pp += 2;
      qq -= 2;
      p += 1;
   }
   forward_bf(2, 4, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4));
   forward_bf(4, 2, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4 + 2));
   back_bf(2, 4, b.as_ptr(), a.as_mut_ptr());
   back_bf(1, 8, a.as_ptr(), c);
}
/*------------------------------------------------------------*/
/*---------------- 8 pt fdct dual to mono---------------------*/
pub unsafe fn fdct8_dual_mono(x: *const f32, c: *mut f32)
{
   let mut a: [f32; 8] = [0.0; 8];			/* ping pong buffers */
   let mut b: [f32; 8] = [0.0; 8];
   let mut t1: f32;
   let mut t2: f32;
   let mut p: i32;
   let mut pp: i32;
   let mut qq: i32;

/* special first stage  */
   t1 = 0.5 * (*x.add(0) + *x.add(1));
   t2 = 0.5 * (*x.add(14) + *x.add(15));
   b[0] = t1 + t2;
   b[4] = coef32[16 + 8] * (t1 - t2);
   pp = 2;
   qq = 2 * 6;
   p = 1;
   while p < 4
   {
      t1 = 0.5 * (*x.add(pp as usize) + *x.add((pp + 1) as usize));
      t2 = 0.5 * (*x.add(qq as usize) + *x.add((qq + 1) as usize));
      b[p as usize] = t1 + t2;
      b[(4 + p) as usize] = coef32[(16 + 8 + p) as usize] * (t1 - t2);
      pp += 2;
      qq -= 2;
      p += 1;
   }
   forward_bf(2, 4, b.as_ptr(), a.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4));
   forward_bf(4, 2, a.as_ptr(), b.as_mut_ptr(), (addr_of!(coef32) as *const f32).add(16 + 8 + 4 + 2));
   back_bf(2, 4, b.as_ptr(), a.as_mut_ptr());
   back_bf(1, 8, a.as_ptr(), c);
}
/*------------------------------------------------------------*/
