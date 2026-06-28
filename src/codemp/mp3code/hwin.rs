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

    $Id: hwin.c,v 1.5 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  hwin.c  ***************************************************

Layer III

hybrid window/filter

******************************************************************/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(static_mut_refs)]

use core::ffi::{c_float, c_int};

use super::mp3struct_h::pMP3Stream;

////@@@@extern int band_limit_nsb;

pub type ARRAY36 = [c_float; 36];

/*-- windows by block type --*/
static mut win: [ARRAY36; 4] = [[0.0f32; 36]; 4]; // effectively a constant

/*====================================================================*/
unsafe extern "C" {
    pub fn imdct18(f: *mut c_float); /* 18 point */
    pub fn imdct6_3(f: *mut c_float); /* 6 point */
}

/*====================================================================*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hwin_init_addr() -> *mut ARRAY36 {
    return core::ptr::addr_of_mut!(win) as *mut ARRAY36;
}

/*====================================================================*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hybrid(
    xin: *mut c_float,
    xprev: *mut c_float,
    y: *mut [[c_float; 32]; 18],
    mut btype: c_int,
    nlong: c_int,
    ntot: c_int,
    nprev: c_int,
) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut x: *mut c_float;
    let mut x0: *mut c_float;
    let mut xa: c_float;
    let mut xb: c_float;
    let mut n: c_int;
    let nout: c_int;

    if btype == 2 {
        btype = 0;
    }
    x = xin;
    x0 = xprev;

    /*-- do long blocks (if any) --*/
    n = (nlong + 17) / 18; /* number of dct's to do */
    i = 0;
    while i < n {
        unsafe {
            imdct18(x);
        }
        j = 0;
        while j < 9 {
            unsafe {
                (*y)[j as usize][i as usize] =
                    *x0.add(j as usize) + win[btype as usize][j as usize] * *x.add((9 + j) as usize);
                (*y)[(9 + j) as usize][i as usize] = *x0.add((9 + j) as usize)
                    + win[btype as usize][(9 + j) as usize] * *x.add((17 - j) as usize);
            }
            j += 1;
        }
        /* window x for next time x0 */
        j = 0;
        while j < 4 {
            unsafe {
                xa = *x.add(j as usize);
                xb = *x.add((8 - j) as usize);
                *x.add(j as usize) = win[btype as usize][(18 + j) as usize] * xb;
                *x.add((8 - j) as usize) = win[btype as usize][((18 + 8) - j) as usize] * xa;
                *x.add((9 + j) as usize) = win[btype as usize][((18 + 9) + j) as usize] * xa;
                *x.add((17 - j) as usize) = win[btype as usize][((18 + 17) - j) as usize] * xb;
            }
            j += 1;
        }
        unsafe {
            xa = *x.add(j as usize);
            *x.add(j as usize) = win[btype as usize][(18 + j) as usize] * xa;
            *x.add((9 + j) as usize) = win[btype as usize][((18 + 9) + j) as usize] * xa;

            x = x.add(18);
            x0 = x0.add(18);
        }
        i += 1;
    }

    /*-- do short blocks (if any) --*/
    n = (ntot + 17) / 18; /* number of 6 pt dct's triples to do */
    while i < n {
        unsafe {
            imdct6_3(x);
        }
        j = 0;
        while j < 3 {
            unsafe {
                (*y)[j as usize][i as usize] = *x0.add(j as usize);
                (*y)[(3 + j) as usize][i as usize] = *x0.add((3 + j) as usize);

                (*y)[(6 + j) as usize][i as usize] =
                    *x0.add((6 + j) as usize) + win[2][j as usize] * *x.add((3 + j) as usize);
                (*y)[(9 + j) as usize][i as usize] =
                    *x0.add((9 + j) as usize) + win[2][(3 + j) as usize] * *x.add((5 - j) as usize);

                (*y)[(12 + j) as usize][i as usize] = *x0.add((12 + j) as usize)
                    + win[2][(6 + j) as usize] * *x.add((2 - j) as usize)
                    + win[2][j as usize] * *x.add(((6 + 3) + j) as usize);
                (*y)[(15 + j) as usize][i as usize] = *x0.add((15 + j) as usize)
                    + win[2][(9 + j) as usize] * *x.add(j as usize)
                    + win[2][(3 + j) as usize] * *x.add(((6 + 5) - j) as usize);
            }
            j += 1;
        }
        /* window x for next time x0 */
        j = 0;
        while j < 3 {
            unsafe {
                *x.add(j as usize) = win[2][(6 + j) as usize] * *x.add(((6 + 2) - j) as usize)
                    + win[2][j as usize] * *x.add(((12 + 3) + j) as usize);
                *x.add((3 + j) as usize) = win[2][(9 + j) as usize] * *x.add((6 + j) as usize)
                    + win[2][(3 + j) as usize] * *x.add(((12 + 5) - j) as usize);
            }
            j += 1;
        }
        j = 0;
        while j < 3 {
            unsafe {
                *x.add((6 + j) as usize) = win[2][(6 + j) as usize] * *x.add(((12 + 2) - j) as usize);
                *x.add((9 + j) as usize) = win[2][(9 + j) as usize] * *x.add((12 + j) as usize);
            }
            j += 1;
        }
        j = 0;
        while j < 3 {
            unsafe {
                *x.add((12 + j) as usize) = 0.0f32;
                *x.add((15 + j) as usize) = 0.0f32;
            }
            j += 1;
        }
        unsafe {
            x = x.add(18);
            x0 = x0.add(18);
        }
        i += 1;
    }

    /*--- overlap prev if prev longer that current --*/
    n = (nprev + 17) / 18;
    while i < n {
        j = 0;
        while j < 18 {
            unsafe {
                (*y)[j as usize][i as usize] = *x0.add(j as usize);
            }
            j += 1;
        }
        unsafe {
            x0 = x0.add(18);
        }
        i += 1;
    }
    nout = 18 * i;

    /*--- clear remaining only to band limit --*/
    while unsafe { i < (*pMP3Stream).u.L3.band_limit_nsb } {
        j = 0;
        while j < 18 {
            unsafe {
                (*y)[j as usize][i as usize] = 0.0f32;
            }
            j += 1;
        }
        i += 1;
    }

    return nout;
}

/*--------------------------------------------------------------------*/
/*--------------------------------------------------------------------*/
/*-- convert to mono, add curr result to y,
    window and add next time to current left */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hybrid_sum(
    xin: *mut c_float,
    xin_left: *mut c_float,
    y: *mut [[c_float; 32]; 18],
    mut btype: c_int,
    nlong: c_int,
    ntot: c_int,
) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut x: *mut c_float;
    let mut x0: *mut c_float;
    let mut xa: c_float;
    let mut xb: c_float;
    let mut n: c_int;
    let nout: c_int;

    if btype == 2 {
        btype = 0;
    }
    x = xin;
    x0 = xin_left;

    /*-- do long blocks (if any) --*/
    n = (nlong + 17) / 18; /* number of dct's to do */
    i = 0;
    while i < n {
        unsafe {
            imdct18(x);
        }
        j = 0;
        while j < 9 {
            unsafe {
                (*y)[j as usize][i as usize] += win[btype as usize][j as usize] * *x.add((9 + j) as usize);
                (*y)[(9 + j) as usize][i as usize] +=
                    win[btype as usize][(9 + j) as usize] * *x.add((17 - j) as usize);
            }
            j += 1;
        }
        /* window x for next time x0 */
        j = 0;
        while j < 4 {
            unsafe {
                xa = *x.add(j as usize);
                xb = *x.add((8 - j) as usize);
                *x0.add(j as usize) += win[btype as usize][(18 + j) as usize] * xb;
                *x0.add((8 - j) as usize) += win[btype as usize][((18 + 8) - j) as usize] * xa;
                *x0.add((9 + j) as usize) += win[btype as usize][((18 + 9) + j) as usize] * xa;
                *x0.add((17 - j) as usize) += win[btype as usize][((18 + 17) - j) as usize] * xb;
            }
            j += 1;
        }
        unsafe {
            xa = *x.add(j as usize);
            *x0.add(j as usize) += win[btype as usize][(18 + j) as usize] * xa;
            *x0.add((9 + j) as usize) += win[btype as usize][((18 + 9) + j) as usize] * xa;

            x = x.add(18);
            x0 = x0.add(18);
        }
        i += 1;
    }

    /*-- do short blocks (if any) --*/
    n = (ntot + 17) / 18; /* number of 6 pt dct's triples to do */
    while i < n {
        unsafe {
            imdct6_3(x);
        }
        j = 0;
        while j < 3 {
            unsafe {
                (*y)[(6 + j) as usize][i as usize] += win[2][j as usize] * *x.add((3 + j) as usize);
                (*y)[(9 + j) as usize][i as usize] += win[2][(3 + j) as usize] * *x.add((5 - j) as usize);

                (*y)[(12 + j) as usize][i as usize] += win[2][(6 + j) as usize] * *x.add((2 - j) as usize)
                    + win[2][j as usize] * *x.add(((6 + 3) + j) as usize);
                (*y)[(15 + j) as usize][i as usize] += win[2][(9 + j) as usize] * *x.add(j as usize)
                    + win[2][(3 + j) as usize] * *x.add(((6 + 5) - j) as usize);
            }
            j += 1;
        }
        /* window x for next time */
        j = 0;
        while j < 3 {
            unsafe {
                *x0.add(j as usize) += win[2][(6 + j) as usize] * *x.add(((6 + 2) - j) as usize)
                    + win[2][j as usize] * *x.add(((12 + 3) + j) as usize);
                *x0.add((3 + j) as usize) += win[2][(9 + j) as usize] * *x.add((6 + j) as usize)
                    + win[2][(3 + j) as usize] * *x.add(((12 + 5) - j) as usize);
            }
            j += 1;
        }
        j = 0;
        while j < 3 {
            unsafe {
                *x0.add((6 + j) as usize) += win[2][(6 + j) as usize] * *x.add(((12 + 2) - j) as usize);
                *x0.add((9 + j) as usize) += win[2][(9 + j) as usize] * *x.add((12 + j) as usize);
            }
            j += 1;
        }
        unsafe {
            x = x.add(18);
            x0 = x0.add(18);
        }
        i += 1;
    }

    nout = 18 * i;

    return nout;
}

/*--------------------------------------------------------------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sum_f_bands(a: *mut c_float, b: *mut c_float, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            *a.add(i as usize) += *b.add(i as usize);
        }
        i += 1;
    }
}

/*--------------------------------------------------------------------*/
/*--------------------------------------------------------------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn FreqInvert(y: *mut [[c_float; 32]; 18], mut n: c_int) {
    let mut i: c_int;
    let mut j: c_int;

    n = (n + 17) / 18;
    j = 0;
    while j < 18 {
        i = 0;
        while i < n {
            unsafe {
                (*y)[(1 + j) as usize][(1 + i) as usize] = -(*y)[(1 + j) as usize][(1 + i) as usize];
            }
            i += 2;
        }
        j += 2;
    }
}

/*--------------------------------------------------------------------*/
