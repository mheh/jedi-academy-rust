#![allow(non_snake_case)]

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

	$Id: msis.c,v 1.4 1999/10/19 07:13:09 elrod Exp $
____________________________________________________________________________*/

/****  msis.c  ***************************************************
  Layer III
 antialias, ms and is stereo precessing

**** is_process assumes never switch
      from short to long in is region *****

is_process does ms or stereo in "forbidded sf regions"
    //ms_mode = 0
    lr[0][i][0] = 1.0f;
    lr[0][i][1] = 0.0f;
    // ms_mode = 1, in is bands is routine does ms processing
    lr[1][i][0] = 1.0f;
    lr[1][i][1] = 1.0f;

******************************************************************/

use core::ffi::c_int;

type ARRAY2 = [f32; 2];
type ARRAY8_2 = [[f32; 2]; 8];
type ARRAY2_64_2 = [[[f32; 2]; 64]; 2];
type ARRAY64_2 = [[f32; 2]; 64];

// Stub type definitions from L3.h and mp3struct.h
#[repr(C)]
pub struct SCALEFACT {
	pub l: [c_int; 21],
	pub s: [[c_int; 12]; 3],
}

#[repr(C)]
pub struct CB_INFO {
	pub cbmax: c_int,
	pub cbtype: c_int,
	pub cbmax_s: [c_int; 3],
}

#[repr(C)]
pub struct IS_SF_INFO {
	pub intensity_scale: c_int,
	pub slen: [c_int; 3],
	pub nr: [c_int; 3],
}

// Stub type for pMP3Stream
#[repr(C)]
pub struct MP3Stream {
	pub nBand: [[c_int; 22]; 2],		/* [long/short][cb] */
	pub sfBandIndex: [[c_int; 22]; 2],	/* [long/short][cb] */
}

// pMP3Stream global pointer (extern, defined elsewhere)
static mut pMP3Stream: *mut MP3Stream = std::ptr::null_mut();

/* antialias */		// effectively constant
static mut csa: ARRAY8_2 = [[0.0f32; 2]; 8];

/* pMP3Stream->nBand[0] = long, pMP3Stream->nBand[1] = short */
////@@@@extern int pMP3Stream->nBand[2][22];
////@@@@extern int pMP3Stream->sfBandIndex[2][22];	/* [long/short][cb] */

/* intensity stereo */
/* if ms mode quant pre-scales all values by 1.0/sqrt(2.0) ms_mode in table
   compensates   */
static mut lr: [[[f32; 2]; 8]; 2] = [[[0.0f32; 2]; 8]; 2];	/* [ms_mode 0/1][sf][left/right]  */	// effectively constant


/* intensity stereo MPEG2 */
/* lr2[intensity_scale][ms_mode][sflen_offset+sf][left/right] */
static mut lr2: [[[[f32; 2]; 64]; 2]; 2] = [[[[0.0f32; 2]; 64]; 2]; 2];		// effectively constant


/*===============================================================*/
#[inline]
fn alias_init_addr() -> *mut ARRAY2 {
    unsafe { std::ptr::addr_of_mut!(csa) as *mut ARRAY2 }
}
/*-----------------------------------------------------------*/
#[inline]
fn msis_init_addr() -> *mut ARRAY8_2 {
/*-------
pi = 4.0*atan(1.0);
t = pi/12.0;
for(i=0;i<7;i++) {
    s = sin(i*t);
    c = cos(i*t);
    // ms_mode = 0
    lr[0][i][0] = (float)(s/(s+c));
    lr[0][i][1] = (float)(c/(s+c));
    // ms_mode = 1
    lr[1][i][0] = (float)(sqrt(2.0)*(s/(s+c)));
    lr[1][i][1] = (float)(sqrt(2.0)*(c/(s+c)));
}
//sf = 7
//ms_mode = 0
lr[0][i][0] = 1.0f;
lr[0][i][1] = 0.0f;
// ms_mode = 1, in is bands is routine does ms processing
lr[1][i][0] = 1.0f;
lr[1][i][1] = 1.0f;
------------*/

    unsafe { std::ptr::addr_of_mut!(lr) as *mut ARRAY8_2 }
}
/*-------------------------------------------------------------*/
#[inline]
fn msis_init_addr_MPEG2() -> *mut ARRAY2_64_2 {
    unsafe { std::ptr::addr_of_mut!(lr2) as *mut ARRAY2_64_2 }
}
/*===============================================================*/
pub fn antialias(mut x: *mut f32, n: i32) {
    let mut i: i32;
    let mut k: i32;
    let mut a: f32;
    let mut b: f32;

    k = 0;
    while k < n {
        i = 0;
        while i < 8 {
            unsafe {
                a = *x.offset((17 - i) as isize);
                b = *x.offset((18 + i) as isize);
                *x.offset((17 - i) as isize) = a * csa[i as usize][0] - b * csa[i as usize][1];
                *x.offset((18 + i) as isize) = b * csa[i as usize][0] + a * csa[i as usize][1];
            }
            i += 1;
        }
        x = unsafe { x.offset(18) };
        k += 1;
    }
}
/*===============================================================*/
pub fn ms_process(x: *mut [f32; 1152], n: i32) {
    let mut i: i32;
    let mut xl: f32;
    let mut xr: f32;

/*-- note: sqrt(2) done scaling by dequant ---*/
    i = 0;
    while i < n {
        unsafe {
            xl = (*x)[0][i as usize] + (*x)[1][i as usize];
            xr = (*x)[0][i as usize] - (*x)[1][i as usize];
            (*x)[0][i as usize] = xl;
            (*x)[1][i as usize] = xr;
        }
        i += 1;
    }
}
/*===============================================================*/
pub fn is_process_MPEG1(
    x: *mut [f32; 1152],
    sf: *mut SCALEFACT,
    cb_info: &[CB_INFO; 2],
    nsamp: i32,
    ms_mode: i32,
) {
    let mut i: i32;
    let mut j: i32;
    let mut n: i32;
    let mut cb: i32;
    let mut w: i32;
    let mut fl: f32;
    let mut fr: f32;
    let mut m: i32;
    let mut isf: i32;
    let mut fls: [f32; 3] = [0.0f32; 3];
    let mut frs: [f32; 3] = [0.0f32; 3];
    let mut cb0: i32;

    unsafe {
        cb0 = cb_info[1].cbmax;	/* start at end of right */
        i = (*pMP3Stream).sfBandIndex[cb_info[1].cbtype as usize][cb0 as usize];
        cb0 += 1;
        m = nsamp - i;		/* process to len of left */

        if cb_info[1].cbtype == 0 {
            /*------------------------*/
            /* long_blocks: */
            cb = cb0;
            while cb < 21 {
                isf = (*sf).l[cb as usize];
                n = (*pMP3Stream).nBand[0][cb as usize];
                fl = lr[ms_mode as usize][isf as usize][0];
                fr = lr[ms_mode as usize][isf as usize][1];
                j = 0;
                while j < n {
                    m -= 1;
                    if m < 0 {
                        return;
                    }
                    (*x)[1][i as usize] = fr * (*x)[0][i as usize];
                    (*x)[0][i as usize] = fl * (*x)[0][i as usize];
                    i += 1;
                    j += 1;
                }
                cb += 1;
            }
            return;
        }

        /*------------------------*/
        /* short_blocks: */
        cb = cb0;
        while cb < 12 {
            w = 0;
            while w < 3 {
                isf = (*sf).s[w as usize][cb as usize];
                fls[w as usize] = lr[ms_mode as usize][isf as usize][0];
                frs[w as usize] = lr[ms_mode as usize][isf as usize][1];
                w += 1;
            }
            n = (*pMP3Stream).nBand[1][cb as usize];
            j = 0;
            while j < n {
                m -= 3;
                if m < 0 {
                    return;
                }
                (*x)[1][i as usize] = frs[0] * (*x)[0][i as usize];
                (*x)[0][i as usize] = fls[0] * (*x)[0][i as usize];
                (*x)[1][(1 + i) as usize] = frs[1] * (*x)[0][(1 + i) as usize];
                (*x)[0][(1 + i) as usize] = fls[1] * (*x)[0][(1 + i) as usize];
                (*x)[1][(2 + i) as usize] = frs[2] * (*x)[0][(2 + i) as usize];
                (*x)[0][(2 + i) as usize] = fls[2] * (*x)[0][(2 + i) as usize];
                i += 3;
                j += 1;
            }
            cb += 1;
        }
    }
}
/*===============================================================*/
pub fn is_process_MPEG2(
    x: *mut [f32; 1152],
    sf: *mut SCALEFACT,
    cb_info: &[CB_INFO; 2],
    is_sf_info: *mut IS_SF_INFO,
    nsamp: i32,
    ms_mode: i32,
) {
    let mut i: i32;
    let mut j: i32;
    let mut k: i32;
    let mut n: i32;
    let mut cb: i32;
    let mut w: i32;
    let mut fl: f32;
    let mut fr: f32;
    let mut m: i32;
    let mut isf: i32;
    let mut il: [i32; 21];
    let mut tmp: i32;
    let mut r: i32;
    let mut lr_ptr: *const ARRAY64_2;
    let mut cb0: i32;
    let mut cb1: i32;

    unsafe {
        lr_ptr = std::ptr::addr_of!(lr2[(*is_sf_info).intensity_scale as usize][ms_mode as usize]) as *const ARRAY64_2;

        if cb_info[1].cbtype != 0 {
            /* short_blocks */
            k = 0;
            r = 0;
            while r < 3 {
                tmp = (1 << (*is_sf_info).slen[r as usize]) - 1;
                j = 0;
                while j < (*is_sf_info).nr[r as usize] {
                    il[k as usize] = tmp;
                    k += 1;
                    j += 1;
                }
                r += 1;
            }

            w = 0;
            while w < 3 {
                cb0 = cb_info[1].cbmax_s[w as usize];	/* start at end of right */
                i = (*pMP3Stream).sfBandIndex[1][cb0 as usize] + w;
                cb1 = cb_info[0].cbmax_s[w as usize];	/* process to end of left */

                cb = cb0 + 1;
                while cb <= cb1 {
                    isf = il[cb as usize] + (*sf).s[w as usize][cb as usize];
                    fl = (*lr_ptr)[isf as usize][0];
                    fr = (*lr_ptr)[isf as usize][1];
                    n = (*pMP3Stream).nBand[1][cb as usize];
                    j = 0;
                    while j < n {
                        (*x)[1][i as usize] = fr * (*x)[0][i as usize];
                        (*x)[0][i as usize] = fl * (*x)[0][i as usize];
                        i += 3;
                        j += 1;
                    }
                    cb += 1;
                }

                w += 1;
            }
            return;
        }

        /*------------------------*/
        /* long_blocks: */
        cb0 = cb_info[1].cbmax;	/* start at end of right */
        i = (*pMP3Stream).sfBandIndex[0][cb0 as usize];
        m = nsamp - i;		/* process to len of left */
        /* gen sf info */
        k = 0;
        r = 0;
        while r < 3 {
            tmp = (1 << (*is_sf_info).slen[r as usize]) - 1;
            j = 0;
            while j < (*is_sf_info).nr[r as usize] {
                il[k as usize] = tmp;
                k += 1;
                j += 1;
            }
            r += 1;
        }
        cb = cb0 + 1;
        while cb < 21 {
            isf = il[cb as usize] + (*sf).l[cb as usize];
            fl = (*lr_ptr)[isf as usize][0];
            fr = (*lr_ptr)[isf as usize][1];
            n = (*pMP3Stream).nBand[0][cb as usize];
            j = 0;
            while j < n {
                m -= 1;
                if m < 0 {
                    return;
                }
                (*x)[1][i as usize] = fr * (*x)[0][i as usize];
                (*x)[0][i as usize] = fl * (*x)[0][i as usize];
                i += 1;
                j += 1;
            }
            cb += 1;
        }
    }
}
/*===============================================================*/
