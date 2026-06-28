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

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_int;

use super::l3_h::{CB_INFO, IS_SF_INFO, SCALEFACT};
use super::mp3struct_h::pMP3Stream;

pub type ARRAY2 = [f32; 2];
pub type ARRAY8_2 = [[f32; 2]; 8];
pub type ARRAY2_64_2 = [[[f32; 2]; 64]; 2];
pub type ARRAY64_2 = [[f32; 2]; 64];

pub static mut csa: ARRAY8_2 = [[0.0; 2]; 8]; /* antialias */ // effectively constant

/* pMP3Stream->nBand[0] = long, pMP3Stream->nBand[1] = short */
////@@@@extern int pMP3Stream->nBand[2][22];
////@@@@extern int pMP3Stream->sfBandIndex[2][22];	/* [long/short][cb] */

/* intensity stereo */
/* if ms mode quant pre-scales all values by 1.0/sqrt(2.0) ms_mode in table
   compensates   */
static mut lr: [ARRAY8_2; 2] = [[[0.0; 2]; 8]; 2]; /* [ms_mode 0/1][sf][left/right]  */ // effectively constant

/* intensity stereo MPEG2 */
/* lr2[intensity_scale][ms_mode][sflen_offset+sf][left/right] */
static mut lr2: [[ARRAY64_2; 2]; 2] = [[[[0.0; 2]; 64]; 2]; 2]; // effectively constant

/*===============================================================*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn alias_init_addr() -> *mut ARRAY2 {
    core::ptr::addr_of_mut!(csa).cast::<ARRAY2>()
}

/*-----------------------------------------------------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msis_init_addr() -> *mut ARRAY8_2 {
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

    core::ptr::addr_of_mut!(lr).cast::<ARRAY8_2>()
}

/*-------------------------------------------------------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msis_init_addr_MPEG2() -> *mut ARRAY2_64_2 {
    core::ptr::addr_of_mut!(lr2).cast::<ARRAY2_64_2>()
}

/*===============================================================*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn antialias(mut x: *mut f32, n: c_int) {
    let mut i: c_int;
    let mut k: c_int;
    let mut a: f32;
    let mut b: f32;

    k = 0;
    while k < n {
        i = 0;
        while i < 8 {
            let csa_i = core::ptr::addr_of!(csa).cast::<ARRAY2>().add(i as usize);
            a = *x.add((17 - i) as usize);
            b = *x.add((18 + i) as usize);
            *x.add((17 - i) as usize) = a * (*csa_i)[0] - b * (*csa_i)[1];
            *x.add((18 + i) as usize) = b * (*csa_i)[0] + a * (*csa_i)[1];
            i += 1;
        }
        x = x.add(18);
        k += 1;
    }
}

/*===============================================================*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ms_process(x: *mut [f32; 1152], n: c_int) {
    /* sum-difference stereo */
    let mut i: c_int;
    let mut xl: f32;
    let mut xr: f32;
    let x = x.cast::<f32>();

    /*-- note: sqrt(2) done scaling by dequant ---*/
    i = 0;
    while i < n {
        xl = *x.add(i as usize) + *x.add(1152 + i as usize);
        xr = *x.add(i as usize) - *x.add(1152 + i as usize);
        *x.add(i as usize) = xl;
        *x.add(1152 + i as usize) = xr;
        i += 1;
    }
    return;
}

/*===============================================================*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn is_process_MPEG1(
    x: *mut [f32; 1152], /* intensity stereo */
    sf: *mut SCALEFACT,
    cb_info: *mut CB_INFO, /* [ch] */
    nsamp: c_int,
    ms_mode: c_int,
) {
    let mut i: c_int;
    let mut j: c_int;
    let mut n: c_int;
    let mut cb: c_int;
    let mut w: c_int;
    let mut fl: f32;
    let mut fr: f32;
    let mut m: c_int;
    let mut isf: c_int;
    let mut fls: [f32; 3] = [0.0; 3];
    let mut frs: [f32; 3] = [0.0; 3];
    let mut cb0: c_int;
    let x = x.cast::<f32>();

    cb0 = (*cb_info.add(1)).cbmax; /* start at end of right */
    i = (*pMP3Stream).u.L3.sfBandIndex[(*cb_info.add(1)).cbtype as usize][cb0 as usize];
    cb0 += 1;
    m = nsamp - i; /* process to len of left */

    if (*cb_info.add(1)).cbtype != 0 {
        /*------------------------*/
        /* short_blocks: */
        cb = cb0;
        while cb < 12 {
            w = 0;
            while w < 3 {
                isf = (*sf).s[w as usize][cb as usize];
                let lr_i = core::ptr::addr_of!(lr).cast::<ARRAY8_2>().add(ms_mode as usize);
                fls[w as usize] = (*lr_i)[isf as usize][0];
                frs[w as usize] = (*lr_i)[isf as usize][1];
                w += 1;
            }
            n = (*pMP3Stream).u.L3.nBand[1][cb as usize];
            j = 0;
            while j < n {
                m -= 3;
                if m < 0 {
                    return;
                }
                *x.add(1152 + i as usize) = frs[0] * *x.add(i as usize);
                *x.add(i as usize) = fls[0] * *x.add(i as usize);
                *x.add(1152 + (1 + i) as usize) = frs[1] * *x.add((1 + i) as usize);
                *x.add((1 + i) as usize) = fls[1] * *x.add((1 + i) as usize);
                *x.add(1152 + (2 + i) as usize) = frs[2] * *x.add((2 + i) as usize);
                *x.add((2 + i) as usize) = fls[2] * *x.add((2 + i) as usize);
                i += 3;
                j += 1;
            }
            cb += 1;
        }

        /* exit: */
        return;
    }

    /*------------------------*/
    /* long_blocks: */
    cb = cb0;
    while cb < 21 {
        isf = (*sf).l[cb as usize];
        n = (*pMP3Stream).u.L3.nBand[0][cb as usize];
        let lr_i = core::ptr::addr_of!(lr).cast::<ARRAY8_2>().add(ms_mode as usize);
        fl = (*lr_i)[isf as usize][0];
        fr = (*lr_i)[isf as usize][1];
        j = 0;
        while j < n {
            m -= 1;
            if m < 0 {
                return;
            }
            *x.add(1152 + i as usize) = fr * *x.add(i as usize);
            *x.add(i as usize) = fl * *x.add(i as usize);
            i += 1;
            j += 1;
        }
        cb += 1;
    }
    return;
}

/*===============================================================*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn is_process_MPEG2(
    x: *mut [f32; 1152], /* intensity stereo */
    sf: *mut SCALEFACT,
    cb_info: *mut CB_INFO, /* [ch] */
    is_sf_info: *mut IS_SF_INFO,
    nsamp: c_int,
    ms_mode: c_int,
) {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut n: c_int;
    let mut cb: c_int;
    let mut w: c_int;
    let mut fl: f32;
    let mut fr: f32;
    let mut m: c_int;
    let mut isf: c_int;
    let mut il: [c_int; 21] = [0; 21];
    let mut tmp: c_int;
    let mut r: c_int;
    let mut lr: *mut ARRAY2;
    let mut cb0: c_int;
    let mut cb1: c_int;
    let x = x.cast::<f32>();

    lr = core::ptr::addr_of_mut!(lr2)
        .cast::<ARRAY2_64_2>()
        .add((*is_sf_info).intensity_scale as usize)
        .cast::<ARRAY64_2>()
        .add(ms_mode as usize)
        .cast::<ARRAY2>();

    if (*cb_info.add(1)).cbtype != 0 {
        /*------------------------*/
        /* short_blocks: */

        k = 0;
        r = 0;
        while r < 3 {
            tmp = (1 << (*is_sf_info).slen[r as usize]) - 1;
            j = 0;
            while j < (*is_sf_info).nr[r as usize] {
                il[k as usize] = tmp;
                j += 1;
                k += 1;
            }
            r += 1;
        }

        w = 0;
        while w < 3 {
            cb0 = (*cb_info.add(1)).cbmax_s[w as usize]; /* start at end of right */
            i = (*pMP3Stream).u.L3.sfBandIndex[1][cb0 as usize] + w;
            cb1 = (*cb_info.add(0)).cbmax_s[w as usize]; /* process to end of left */

            cb = cb0 + 1;
            while cb <= cb1 {
                isf = il[cb as usize] + (*sf).s[w as usize][cb as usize];
                fl = (*lr.add(isf as usize))[0];
                fr = (*lr.add(isf as usize))[1];
                n = (*pMP3Stream).u.L3.nBand[1][cb as usize];
                j = 0;
                while j < n {
                    *x.add(1152 + i as usize) = fr * *x.add(i as usize);
                    *x.add(i as usize) = fl * *x.add(i as usize);
                    i += 3;
                    j += 1;
                }
                cb += 1;
            }

            w += 1;
        }

        /* exit: */
        return;
    }

    /*------------------------*/
    /* long_blocks: */
    cb0 = (*cb_info.add(1)).cbmax; /* start at end of right */
    i = (*pMP3Stream).u.L3.sfBandIndex[0][cb0 as usize];
    m = nsamp - i; /* process to len of left */
    /* gen sf info */
    k = 0;
    r = 0;
    while r < 3 {
        tmp = (1 << (*is_sf_info).slen[r as usize]) - 1;
        j = 0;
        while j < (*is_sf_info).nr[r as usize] {
            il[k as usize] = tmp;
            j += 1;
            k += 1;
        }
        r += 1;
    }
    cb = cb0 + 1;
    while cb < 21 {
        isf = il[cb as usize] + (*sf).l[cb as usize];
        fl = (*lr.add(isf as usize))[0];
        fr = (*lr.add(isf as usize))[1];
        n = (*pMP3Stream).u.L3.nBand[0][cb as usize];
        j = 0;
        while j < n {
            m -= 1;
            if m < 0 {
                return;
            }
            *x.add(1152 + i as usize) = fr * *x.add(i as usize);
            *x.add(i as usize) = fl * *x.add(i as usize);
            i += 1;
            j += 1;
        }
        cb += 1;
    }
    return;
}
