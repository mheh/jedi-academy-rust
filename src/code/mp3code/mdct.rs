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

use core::ffi::c_void;
use core::ptr::addr_of_mut;

//------ 18 point xform -------
pub static mut mdct18w: [f32; 18] = [0.0; 18];		// effectively constant
pub static mut mdct18w2: [f32; 9] = [0.0; 9];		//  "  "
pub static mut coef: [[f32; 4]; 9] = [[0.0; 4]; 9];		//  "  "

pub static mut mdct6_3v: [f32; 6] = [0.0; 6];		//  "  "
pub static mut mdct6_3v2: [f32; 3] = [0.0; 3];		//  "  "
pub static mut coef87: f32 = 0.0;			//  "  "

#[repr(C)]
pub struct IMDCT_INIT_BLOCK {
    pub w: *mut f32,
    pub w2: *mut f32,
    pub coef: *mut c_void,
}

static IMDCT_INFO_18: IMDCT_INIT_BLOCK = IMDCT_INIT_BLOCK {
    w: unsafe { &mdct18w as *const [f32; 18] as *mut [f32; 18] as *mut f32 },
    w2: unsafe { &mdct18w2 as *const [f32; 9] as *mut [f32; 9] as *mut f32 },
    coef: unsafe { &coef as *const [[f32; 4]; 9] as *mut [[f32; 4]; 9] as *mut c_void },
};

static IMDCT_INFO_6: IMDCT_INIT_BLOCK = IMDCT_INIT_BLOCK {
    w: unsafe { &mdct6_3v as *const [f32; 6] as *mut [f32; 6] as *mut f32 },
    w2: unsafe { &mdct6_3v2 as *const [f32; 3] as *mut [f32; 3] as *mut f32 },
    coef: unsafe { &coef87 as *const f32 as *mut f32 as *mut c_void },
};

/*====================================================================*/
pub fn imdct_init_addr_18() -> *const IMDCT_INIT_BLOCK {
    &IMDCT_INFO_18
}

pub fn imdct_init_addr_6() -> *const IMDCT_INIT_BLOCK {
    &IMDCT_INFO_6
}

/*--------------------------------------------------------------------*/
pub unsafe fn imdct18(f: *mut f32) {
    /* 18 point */
    let mut a: [f32; 9] = [0.0; 9];
    let mut b: [f32; 9] = [0.0; 9];
    let mut ap: f32;
    let mut bp: f32;
    let mut a8p: f32;
    let mut b8p: f32;
    let mut g1: f32;
    let mut g2: f32;

    for p in 0..4 {
        g1 = mdct18w[p] * *f.add(p);
        g2 = mdct18w[17 - p] * *f.add(17 - p);
        ap = g1 + g2;		// a[p]

        bp = mdct18w2[p] * (g1 - g2);	// b[p]

        g1 = mdct18w[8 - p] * *f.add(8 - p);
        g2 = mdct18w[9 + p] * *f.add(9 + p);
        a8p = g1 + g2;		// a[8-p]

        b8p = mdct18w2[8 - p] * (g1 - g2);	// b[8-p]

        a[p] = ap + a8p;
        a[5 + p] = ap - a8p;
        b[p] = bp + b8p;
        b[5 + p] = bp - b8p;
    }
    g1 = mdct18w[4] * *f.add(4);
    g2 = mdct18w[13] * *f.add(13);
    a[4] = g1 + g2;
    b[4] = mdct18w2[4] * (g1 - g2);


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

    return;
}

/*--------------------------------------------------------------------*/
/* does 3, 6 pt dct.  changes order from f[i][window] c[window][i] */
pub unsafe fn imdct6_3(mut f: *mut f32) {
    /* 6 point */
    let mut buf: [f32; 18] = [0.0; 18];
    let mut a: *mut f32;
    let mut c: *mut f32;		// b[i] = a[3+i]

    let mut g1: f32;
    let mut g2: f32;
    let mut a02: f32;
    let mut b02: f32;

    c = f;
    a = buf.as_mut_ptr();
    for w in 0..3 {
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

    a = buf.as_mut_ptr();
    for w in 0..3 {
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

    return;
}

/*--------------------------------------------------------------------*/
