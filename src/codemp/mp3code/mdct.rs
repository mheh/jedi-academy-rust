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

    $Id: mdct.c,v 1.4 1999/10/19 07:13:09 elrod Exp $
____________________________________________________________________________*/

/****  mdct.c  ***************************************************

Layer III

  cos transform for n=18, n=6

computes  c[k] =  Sum( cos((pi/4*n)*(2*k+1)*(2*p+1))*f[p] )
                k = 0, ...n-1,  p = 0...n-1


inplace ok.

******************************************************************/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(static_mut_refs)]

use core::ffi::{c_float, c_int, c_void};

/*------ 18 point xform -------*/
pub static mut mdct18w: [c_float; 18] = [0.0f32; 18]; /* effectively constant */
pub static mut mdct18w2: [c_float; 9] = [0.0f32; 9]; /*  "  " */
pub static mut coef: [[c_float; 4]; 9] = [[0.0f32; 4]; 9]; /*  "  " */

pub static mut mdct6_3v: [c_float; 6] = [0.0f32; 6]; /*  "  " */
pub static mut mdct6_3v2: [c_float; 3] = [0.0f32; 3]; /*  "  " */
pub static mut coef87: c_float = 0.0f32; /*  "  " */

#[repr(C)]
pub struct IMDCT_INIT_BLOCK {
    pub w: *mut c_float,
    pub w2: *mut c_float,
    pub coef: *mut c_void,
}

static mut imdct_info_18: IMDCT_INIT_BLOCK = IMDCT_INIT_BLOCK {
    w: core::ptr::null_mut(),
    w2: core::ptr::null_mut(),
    coef: core::ptr::null_mut(),
};
static mut imdct_info_6: IMDCT_INIT_BLOCK = IMDCT_INIT_BLOCK {
    w: core::ptr::null_mut(),
    w2: core::ptr::null_mut(),
    coef: core::ptr::null_mut(),
};

/*====================================================================*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn imdct_init_addr_18() -> *const IMDCT_INIT_BLOCK {
    unsafe {
        imdct_info_18.w = core::ptr::addr_of_mut!(mdct18w) as *mut c_float;
        imdct_info_18.w2 = core::ptr::addr_of_mut!(mdct18w2) as *mut c_float;
        imdct_info_18.coef = core::ptr::addr_of_mut!(coef) as *mut c_void;
        core::ptr::addr_of!(imdct_info_18)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn imdct_init_addr_6() -> *const IMDCT_INIT_BLOCK {
    unsafe {
        imdct_info_6.w = core::ptr::addr_of_mut!(mdct6_3v) as *mut c_float;
        imdct_info_6.w2 = core::ptr::addr_of_mut!(mdct6_3v2) as *mut c_float;
        imdct_info_6.coef = core::ptr::addr_of_mut!(coef87) as *mut c_void;
        core::ptr::addr_of!(imdct_info_6)
    }
}

/*--------------------------------------------------------------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn imdct18(f: *mut c_float) {
    let mut p: c_int;
    let mut a: [c_float; 9] = [0.0f32; 9];
    let mut b: [c_float; 9] = [0.0f32; 9];
    let mut ap: c_float;
    let mut bp: c_float;
    let mut a8p: c_float;
    let mut b8p: c_float;
    let mut g1: c_float;
    let mut g2: c_float;

    p = 0;
    while p < 4 {
        unsafe {
            g1 = mdct18w[p as usize] * *f.add(p as usize);
            g2 = mdct18w[(17 - p) as usize] * *f.add((17 - p) as usize);
        }
        ap = g1 + g2; /* a[p] */

        unsafe {
            bp = mdct18w2[p as usize] * (g1 - g2); /* b[p] */
        }

        unsafe {
            g1 = mdct18w[(8 - p) as usize] * *f.add((8 - p) as usize);
            g2 = mdct18w[(9 + p) as usize] * *f.add((9 + p) as usize);
        }
        a8p = g1 + g2; /* a[8-p] */

        unsafe {
            b8p = mdct18w2[(8 - p) as usize] * (g1 - g2); /* b[8-p] */
        }

        a[p as usize] = ap + a8p;
        a[(5 + p) as usize] = ap - a8p;
        b[p as usize] = bp + b8p;
        b[(5 + p) as usize] = bp - b8p;
        p += 1;
    }
    unsafe {
        g1 = mdct18w[p as usize] * *f.add(p as usize);
        g2 = mdct18w[(17 - p) as usize] * *f.add((17 - p) as usize);
        a[p as usize] = g1 + g2;
        b[p as usize] = mdct18w2[p as usize] * (g1 - g2);

        *f.add(0) = 0.5f32 * (a[0] + a[1] + a[2] + a[3] + a[4]);
        *f.add(1) = 0.5f32 * (b[0] + b[1] + b[2] + b[3] + b[4]);

        *f.add(2) = coef[1][0] * a[5] + coef[1][1] * a[6] + coef[1][2] * a[7]
            + coef[1][3] * a[8];
        *f.add(3) = coef[1][0] * b[5] + coef[1][1] * b[6] + coef[1][2] * b[7]
            + coef[1][3] * b[8] - *f.add(1);
        *f.add(1) = *f.add(1) - *f.add(0);
        *f.add(2) = *f.add(2) - *f.add(1);

        *f.add(4) = coef[2][0] * a[0] + coef[2][1] * a[1] + coef[2][2] * a[2]
            + coef[2][3] * a[3] - a[4];
        *f.add(5) = coef[2][0] * b[0] + coef[2][1] * b[1] + coef[2][2] * b[2]
            + coef[2][3] * b[3] - b[4] - *f.add(3);
        *f.add(3) = *f.add(3) - *f.add(2);
        *f.add(4) = *f.add(4) - *f.add(3);

        *f.add(6) = coef[3][0] * (a[5] - a[7] - a[8]);
        *f.add(7) = coef[3][0] * (b[5] - b[7] - b[8]) - *f.add(5);
        *f.add(5) = *f.add(5) - *f.add(4);
        *f.add(6) = *f.add(6) - *f.add(5);

        *f.add(8) = coef[4][0] * a[0] + coef[4][1] * a[1] + coef[4][2] * a[2]
            + coef[4][3] * a[3] + a[4];
        *f.add(9) = coef[4][0] * b[0] + coef[4][1] * b[1] + coef[4][2] * b[2]
            + coef[4][3] * b[3] + b[4] - *f.add(7);
        *f.add(7) = *f.add(7) - *f.add(6);
        *f.add(8) = *f.add(8) - *f.add(7);

        *f.add(10) = coef[5][0] * a[5] + coef[5][1] * a[6] + coef[5][2] * a[7]
            + coef[5][3] * a[8];
        *f.add(11) = coef[5][0] * b[5] + coef[5][1] * b[6] + coef[5][2] * b[7]
            + coef[5][3] * b[8] - *f.add(9);
        *f.add(9) = *f.add(9) - *f.add(8);
        *f.add(10) = *f.add(10) - *f.add(9);

        *f.add(12) = 0.5f32 * (a[0] + a[2] + a[3]) - a[1] - a[4];
        *f.add(13) = 0.5f32 * (b[0] + b[2] + b[3]) - b[1] - b[4] - *f.add(11);
        *f.add(11) = *f.add(11) - *f.add(10);
        *f.add(12) = *f.add(12) - *f.add(11);

        *f.add(14) = coef[7][0] * a[5] + coef[7][1] * a[6] + coef[7][2] * a[7]
            + coef[7][3] * a[8];
        *f.add(15) = coef[7][0] * b[5] + coef[7][1] * b[6] + coef[7][2] * b[7]
            + coef[7][3] * b[8] - *f.add(13);
        *f.add(13) = *f.add(13) - *f.add(12);
        *f.add(14) = *f.add(14) - *f.add(13);

        *f.add(16) = coef[8][0] * a[0] + coef[8][1] * a[1] + coef[8][2] * a[2]
            + coef[8][3] * a[3] + a[4];
        *f.add(17) = coef[8][0] * b[0] + coef[8][1] * b[1] + coef[8][2] * b[2]
            + coef[8][3] * b[3] + b[4] - *f.add(15);
        *f.add(15) = *f.add(15) - *f.add(14);
        *f.add(16) = *f.add(16) - *f.add(15);
        *f.add(17) = *f.add(17) - *f.add(16);
    }

    return;
}

/*--------------------------------------------------------------------*/
/* does 3, 6 pt dct.  changes order from f[i][window] c[window][i] */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn imdct6_3(mut f: *mut c_float) {
    let mut w: c_int;
    let mut buf: [c_float; 18] = [0.0f32; 18];
    let mut a: *mut c_float;
    let mut c: *mut c_float; /* b[i] = a[3+i] */

    let mut g1: c_float;
    let mut g2: c_float;
    let mut a02: c_float;
    let mut b02: c_float;

    c = f;
    a = buf.as_mut_ptr();
    w = 0;
    while w < 3 {
        unsafe {
            g1 = mdct6_3v[0] * *f.add(3 * 0);
            g2 = mdct6_3v[5] * *f.add(3 * 5);
            *a.add(0) = g1 + g2;
            *a.add(3 + 0) = mdct6_3v2[0] * (g1 - g2);

            g1 = mdct6_3v[1] * *f.add(3 * 1);
            g2 = mdct6_3v[4] * *f.add(3 * 4);
            *a.add(1) = g1 + g2;
            *a.add(3 + 1) = mdct6_3v2[1] * (g1 - g2);

            g1 = mdct6_3v[2] * *f.add(3 * 2);
            g2 = mdct6_3v[3] * *f.add(3 * 3);
            *a.add(2) = g1 + g2;
            *a.add(3 + 2) = mdct6_3v2[2] * (g1 - g2);

            a = a.add(6);
            f = f.add(1);
        }
        w += 1;
    }

    a = buf.as_mut_ptr();
    w = 0;
    while w < 3 {
        unsafe {
            a02 = *a.add(0) + *a.add(2);
            b02 = *a.add(3 + 0) + *a.add(3 + 2);
            *c.add(0) = a02 + *a.add(1);
            *c.add(1) = b02 + *a.add(3 + 1);
            *c.add(2) = coef87 * (*a.add(0) - *a.add(2));
            *c.add(3) = coef87 * (*a.add(3 + 0) - *a.add(3 + 2)) - *c.add(1);
            *c.add(1) = *c.add(1) - *c.add(0);
            *c.add(2) = *c.add(2) - *c.add(1);
            *c.add(4) = a02 - *a.add(1) - *a.add(1);
            *c.add(5) = b02 - *a.add(3 + 1) - *a.add(3 + 1) - *c.add(3);
            *c.add(3) = *c.add(3) - *c.add(2);
            *c.add(4) = *c.add(4) - *c.add(3);
            *c.add(5) = *c.add(5) - *c.add(4);
            a = a.add(6);
            c = c.add(6);
        }
        w += 1;
    }

    return;
}

/*--------------------------------------------------------------------*/
