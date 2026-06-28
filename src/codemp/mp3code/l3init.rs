/*____________________________________________________________________________

    FreeAmp - The Free MP3 Player

        MP3 Decoder originally Copyright (C) 1995-1997 Xing Technology
        Corp.  http://www.xingtech.com

    Portions Copyright (C) 1998 EMusic.com

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

    $Id: l3init.c,v 1.2 1999/10/19 07:13:09 elrod Exp $
____________________________________________________________________________*/

/****  tinit.c  ***************************************************
  Layer III  init tables


******************************************************************/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(static_mut_refs)]

use core::ffi::{c_double, c_float, c_int, c_void};

use super::l3_h::GLOBAL_GAIN_SCALE;

/*---------- quant ---------------------------------*/
/* 8 bit lookup x = pow(2.0, 0.25*(global_gain-210)) */

/* x = pow(2.0, -0.5*(1+scalefact_scale)*scalefac + preemp) */
pub type LS = [[c_float; 32]; 4];

pub type iARRAY22 = [c_int; 22];

/*---------- antialias ---------------------------------*/
pub type PAIR = [c_float; 2];

static Ci: [c_float; 8] = [
    -0.6f32, -0.535f32, -0.33f32, -0.185f32, -0.095f32, -0.041f32, -0.0142f32,
    -0.0037f32,
];

#[repr(C)]
pub struct IMDCT_INIT_BLOCK {
    pub w: *mut c_float,
    pub w2: *mut c_float,
    pub coef: *mut c_void,
}

unsafe extern "C" {
    pub fn quant_init_global_addr() -> *mut c_float;
    pub fn quant_init_scale_addr() -> *mut LS;
    pub fn quant_init_pow_addr() -> *mut c_float;
    pub fn quant_init_subblock_addr() -> *mut c_float;
    pub fn quant_init_band_addr() -> *mut iARRAY22;
    pub fn alias_init_addr() -> *mut PAIR;
    pub fn hwin_init_addr() -> *mut ARRAY36;
    pub fn imdct_init_addr_18() -> *const IMDCT_INIT_BLOCK;
    pub fn imdct_init_addr_6() -> *const IMDCT_INIT_BLOCK;
    pub fn msis_init_addr() -> *mut ARRAY8_2;
    pub fn msis_init_addr_MPEG2() -> *mut ARRAY2_64_2;
}

/*=============================================================*/
pub unsafe extern "C" fn L3table_init() -> c_int {
    let mut i: c_int;
    let mut x: *mut c_float;
    let mut ls: *mut LS;
    let mut scalefact_scale: c_int;
    let mut preemp: c_int;
    let mut scalefac: c_int;
    let mut tmp: c_double;
    let mut csa: *mut PAIR;

    static mut iOnceOnly: c_int = 0;

    {
        let iOnceOnly_ptr = core::ptr::addr_of_mut!(iOnceOnly);
        let iOnceOnly_old = unsafe { *iOnceOnly_ptr };
        unsafe {
            *iOnceOnly_ptr = iOnceOnly_old + 1;
        }
        if iOnceOnly_old == 0 {
            /*================ quant ===============================*/

            /* 8 bit plus 2 lookup x = pow(2.0, 0.25*(global_gain-210)) */
            /* extra 2 for ms scaling by 1/sqrt(2) */
            /* extra 4 for cvt to mono scaling by 1/2 */
            x = unsafe { quant_init_global_addr() };
            i = 0;
            while i < 256 + 2 + 4 {
                unsafe {
                    *x.add(i as usize) = (2.0f64)
                        .powf(0.25f64 * ((i - (2 + 4)) - 210 + GLOBAL_GAIN_SCALE) as c_double)
                        as c_float;
                }
                i += 1;
            }

            /* x = pow(2.0, -0.5*(1+scalefact_scale)*scalefac + preemp) */
            ls = unsafe { quant_init_scale_addr() };
            scalefact_scale = 0;
            while scalefact_scale < 2 {
                preemp = 0;
                while preemp < 4 {
                    scalefac = 0;
                    while scalefac < 32 {
                        unsafe {
                            (*ls.add(scalefact_scale as usize))[preemp as usize]
                                [scalefac as usize] = (2.0f64).powf(
                                -0.5f64
                                    * (1 + scalefact_scale) as c_double
                                    * (scalefac + preemp) as c_double,
                            ) as c_float;
                        }
                        scalefac += 1;
                    }
                    preemp += 1;
                }
                scalefact_scale += 1;
            }

            /*--- iSample**(4/3) lookup, -32<=i<=31 ---*/
            x = unsafe { quant_init_pow_addr() };
            i = 0;
            while i < 64 {
                tmp = (i - 32) as c_double;
                unsafe {
                    *x.add(i as usize) = (tmp * tmp.abs().powf(1.0f64 / 3.0f64)) as c_float;
                }
                i += 1;
            }

            /*-- pow(2.0, -0.25*8.0*subblock_gain)  3 bits --*/
            x = unsafe { quant_init_subblock_addr() };
            i = 0;
            while i < 8 {
                unsafe {
                    *x.add(i as usize) = (2.0f64).powf(0.25f64 * -8.0f64 * i as c_double) as c_float;
                }
                i += 1;
            }

            /*-------------------------*/
            // quant_init_sf_band(sr_index);   replaced by code in sup.c

            /*================ antialias ===============================*/
            // onceonly!!!!!!!!!!!!!!!!!!!!!
            csa = unsafe { alias_init_addr() };
            i = 0;
            while i < 8 {
                unsafe {
                    (*csa.add(i as usize))[0] =
                        (1.0f64 / (1.0f64 + Ci[i as usize] as c_double * Ci[i as usize] as c_double).sqrt())
                            as c_float;
                    (*csa.add(i as usize))[1] = (Ci[i as usize] as c_double
                        / (1.0f64 + Ci[i as usize] as c_double * Ci[i as usize] as c_double).sqrt())
                        as c_float;
                }
                i += 1;
            }
        }
    }

    // these 4 are iOnceOnly-protected inside...

    /*================ msis ===============================*/
    unsafe {
        msis_init();
        msis_init_MPEG2();
    }

    /*================ imdct ===============================*/
    unsafe {
        imdct_init();
    }

    /*--- hybrid windows ------------*/
    unsafe {
        hwin_init();
    }

    return 0;
}

/*====================================================================*/
pub type ARRAY36 = [c_float; 36];

/*--------------------------------------------------------------------*/
pub unsafe extern "C" fn hwin_init() {
    let mut i: c_int;
    let mut j: c_int;
    let mut pi: c_double;
    let mut win: *mut ARRAY36;

    static mut iOnceOnly: c_int = 0;

    let iOnceOnly_ptr = core::ptr::addr_of_mut!(iOnceOnly);
    let iOnceOnly_old = unsafe { *iOnceOnly_ptr };
    unsafe {
        *iOnceOnly_ptr = iOnceOnly_old + 1;
    }
    if iOnceOnly_old == 0 {
        win = unsafe { hwin_init_addr() };

        pi = 4.0f64 * (1.0f64).atan();

        /* type 0 */
        i = 0;
        while i < 36 {
            unsafe {
                (*win.add(0))[i as usize] = (pi / 36.0f64 * (i as c_double + 0.5f64)).sin() as c_float;
            }
            i += 1;
        }

        /* type 1 */
        i = 0;
        while i < 18 {
            unsafe {
                (*win.add(1))[i as usize] = (pi / 36.0f64 * (i as c_double + 0.5f64)).sin() as c_float;
            }
            i += 1;
        }
        i = 18;
        while i < 24 {
            unsafe {
                (*win.add(1))[i as usize] = 1.0f32;
            }
            i += 1;
        }
        i = 24;
        while i < 30 {
            unsafe {
                (*win.add(1))[i as usize] =
                    (pi / 12.0f64 * (i as c_double + 0.5f64 - 18.0f64)).sin() as c_float;
            }
            i += 1;
        }
        i = 30;
        while i < 36 {
            unsafe {
                (*win.add(1))[i as usize] = 0.0f32;
            }
            i += 1;
        }

        /* type 3 */
        i = 0;
        while i < 6 {
            unsafe {
                (*win.add(3))[i as usize] = 0.0f32;
            }
            i += 1;
        }
        i = 6;
        while i < 12 {
            unsafe {
                (*win.add(3))[i as usize] =
                    (pi / 12.0f64 * (i as c_double + 0.5f64 - 6.0f64)).sin() as c_float;
            }
            i += 1;
        }
        i = 12;
        while i < 18 {
            unsafe {
                (*win.add(3))[i as usize] = 1.0f32;
            }
            i += 1;
        }
        i = 18;
        while i < 36 {
            unsafe {
                (*win.add(3))[i as usize] = (pi / 36.0f64 * (i as c_double + 0.5f64)).sin() as c_float;
            }
            i += 1;
        }

        /* type 2 */
        i = 0;
        while i < 12 {
            unsafe {
                (*win.add(2))[i as usize] = (pi / 12.0f64 * (i as c_double + 0.5f64)).sin() as c_float;
            }
            i += 1;
        }
        i = 12;
        while i < 36 {
            unsafe {
                (*win.add(2))[i as usize] = 0.0f32;
            }
            i += 1;
        }

        /*--- invert signs by region to match mdct 18pt --> 36pt mapping */
        j = 0;
        while j < 4 {
            if j == 2 {
                j += 1;
                continue;
            }
            i = 9;
            while i < 36 {
                unsafe {
                    (*win.add(j as usize))[i as usize] = -(*win.add(j as usize))[i as usize];
                }
                i += 1;
            }
            j += 1;
        }

        /*-- invert signs for short blocks --*/
        i = 3;
        while i < 12 {
            unsafe {
                (*win.add(2))[i as usize] = -(*win.add(2))[i as usize];
            }
            i += 1;
        }
    }
}

/*=============================================================*/
pub type ARRAY4 = [c_float; 4];

/*-------------------------------------------------------------*/
pub unsafe extern "C" fn imdct_init() {
    let mut k: c_int;
    let mut p: c_int;
    let mut n: c_int;
    let mut t: c_double;
    let mut pi: c_double;
    let mut addr: *const IMDCT_INIT_BLOCK;
    let mut w: *mut c_float;
    let mut w2: *mut c_float;
    let mut v: *mut c_float;
    let mut v2: *mut c_float;
    let mut coef87: *mut c_float;
    let mut coef: *mut ARRAY4;

    static mut iOnceOnly: c_int = 0;

    let iOnceOnly_ptr = core::ptr::addr_of_mut!(iOnceOnly);
    let iOnceOnly_old = unsafe { *iOnceOnly_ptr };
    unsafe {
        *iOnceOnly_ptr = iOnceOnly_old + 1;
    }
    if iOnceOnly_old == 0 {
        /*--- 18 point --*/
        addr = unsafe { imdct_init_addr_18() };
        unsafe {
            w = (*addr).w;
            w2 = (*addr).w2;
            coef = (*addr).coef as *mut ARRAY4;
        }
        /*----*/
        n = 18;
        pi = 4.0f64 * (1.0f64).atan();
        t = pi / (4 * n) as c_double;
        p = 0;
        while p < n {
            unsafe {
                *w.add(p as usize) = (2.0f64 * (t * (2 * p + 1) as c_double).cos()) as c_float;
            }
            p += 1;
        }
        p = 0;
        while p < 9 {
            unsafe {
                *w2.add(p as usize) = (2.0f64 * (2.0f64 * t * (2 * p + 1) as c_double).cos()) as c_float;
            }
            p += 1;
        }

        t = pi / (2 * n) as c_double;
        k = 0;
        while k < 9 {
            p = 0;
            while p < 4 {
                unsafe {
                    (*coef.add(k as usize))[p as usize] =
                        (t * (2 * k) as c_double * (2 * p + 1) as c_double).cos() as c_float;
                }
                p += 1;
            }
            k += 1;
        }

        /*--- 6 point */
        addr = unsafe { imdct_init_addr_6() };
        unsafe {
            v = (*addr).w;
            v2 = (*addr).w2;
            coef87 = (*addr).coef as *mut c_float;
        }
        /*----*/
        n = 6;
        pi = 4.0f64 * (1.0f64).atan();
        t = pi / (4 * n) as c_double;
        p = 0;
        while p < n {
            unsafe {
                *v.add(p as usize) = (2.0f64 * (t * (2 * p + 1) as c_double).cos()) as c_float;
            }
            p += 1;
        }

        p = 0;
        while p < 3 {
            unsafe {
                *v2.add(p as usize) = (2.0f64 * (2.0f64 * t * (2 * p + 1) as c_double).cos()) as c_float;
            }
            p += 1;
        }

        t = pi / (2 * n) as c_double;
        k = 1;
        p = 0;
        unsafe {
            *coef87 = (t * (2 * k) as c_double * (2 * p + 1) as c_double).cos() as c_float;
        }
        /* adjust scaling to save a few mults */
        p = 0;
        while p < 6 {
            unsafe {
                *v.add(p as usize) = *v.add(p as usize) / 2.0f32;
            }
            p += 1;
        }
        unsafe {
            *coef87 = (2.0f64 * *coef87 as c_double) as c_float;
        }
    }
}

/*===============================================================*/
pub type ARRAY8_2 = [[c_float; 2]; 8];

/*-------------------------------------------------------------*/
pub unsafe extern "C" fn msis_init() {
    let mut i: c_int;
    let mut s: c_double;
    let mut c: c_double;
    let mut pi: c_double;
    let mut t: c_double;
    let mut lr: *mut ARRAY8_2;

    static mut iOnceOnly: c_int = 0;

    let iOnceOnly_ptr = core::ptr::addr_of_mut!(iOnceOnly);
    let iOnceOnly_old = unsafe { *iOnceOnly_ptr };
    unsafe {
        *iOnceOnly_ptr = iOnceOnly_old + 1;
    }
    if iOnceOnly_old == 0 {
        lr = unsafe { msis_init_addr() };

        pi = 4.0f64 * (1.0f64).atan();
        t = pi / 12.0f64;
        i = 0;
        while i < 7 {
            s = (i as c_double * t).sin();
            c = (i as c_double * t).cos();
            /* ms_mode = 0 */
            unsafe {
                (*lr.add(0))[i as usize][0] = (s / (s + c)) as c_float;
                (*lr.add(0))[i as usize][1] = (c / (s + c)) as c_float;
            }
            /* ms_mode = 1 */
            unsafe {
                (*lr.add(1))[i as usize][0] = ((2.0f64).sqrt() * (s / (s + c))) as c_float;
                (*lr.add(1))[i as usize][1] = ((2.0f64).sqrt() * (c / (s + c))) as c_float;
            }
            i += 1;
        }
        /* sf = 7 */
        /* ms_mode = 0 */
        unsafe {
            (*lr.add(0))[i as usize][0] = 1.0f32;
            (*lr.add(0))[i as usize][1] = 0.0f32;
        }
        /* ms_mode = 1, in is bands is routine does ms processing */
        unsafe {
            (*lr.add(1))[i as usize][0] = 1.0f32;
            (*lr.add(1))[i as usize][1] = 1.0f32;
        }

        /*-------
        for(i=0;i<21;i++) nBand[0][i] =
                    sfBandTable[sr_index].l[i+1] - sfBandTable[sr_index].l[i];
        for(i=0;i<12;i++) nBand[1][i] =
                    sfBandTable[sr_index].s[i+1] - sfBandTable[sr_index].s[i];
        -------------*/
    }
}

/*-------------------------------------------------------------*/
/*===============================================================*/
pub type ARRAY2_64_2 = [[[c_float; 2]; 64]; 2];

/*-------------------------------------------------------------*/
pub unsafe extern "C" fn msis_init_MPEG2() {
    let mut k: c_int;
    let mut n: c_int;
    let mut t: c_double;
    let mut lr2: *mut ARRAY2_64_2;
    let mut intensity_scale: c_int;
    let mut ms_mode: c_int;
    let mut sf: c_int;
    let mut sflen: c_int;
    let mut ms_factor: [c_float; 2] = [0.0f32; 2];

    static mut iOnceOnly: c_int = 0;

    let iOnceOnly_ptr = core::ptr::addr_of_mut!(iOnceOnly);
    let iOnceOnly_old = unsafe { *iOnceOnly_ptr };
    unsafe {
        *iOnceOnly_ptr = iOnceOnly_old + 1;
    }
    if iOnceOnly_old == 0 {
        ms_factor[0] = 1.0f32;
        ms_factor[1] = (2.0f64).sqrt() as c_float;

        lr2 = unsafe { msis_init_addr_MPEG2() };

        /* intensity stereo MPEG2 */
        /* lr2[intensity_scale][ms_mode][sflen_offset+sf][left/right] */

        intensity_scale = 0;
        while intensity_scale < 2 {
            t = (2.0f64).powf(-0.25f64 * (1 + intensity_scale) as c_double);
            ms_mode = 0;
            while ms_mode < 2 {
                n = 1;
                k = 0;
                sflen = 0;
                while sflen < 6 {
                    sf = 0;
                    while sf < (n - 1) {
                        if sf == 0 {
                            unsafe {
                                (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][0] =
                                    ms_factor[ms_mode as usize] * 1.0f32;
                                (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][1] =
                                    ms_factor[ms_mode as usize] * 1.0f32;
                            }
                        } else if (sf & 1) != 0 {
                            unsafe {
                                (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][0] =
                                    (ms_factor[ms_mode as usize] as c_double
                                        * t.powf(((sf + 1) / 2) as c_double))
                                        as c_float;
                                (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][1] =
                                    ms_factor[ms_mode as usize] * 1.0f32;
                            }
                        } else {
                            unsafe {
                                (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][0] =
                                    ms_factor[ms_mode as usize] * 1.0f32;
                                (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][1] =
                                    (ms_factor[ms_mode as usize] as c_double
                                        * t.powf((sf / 2) as c_double))
                                        as c_float;
                            }
                        }
                        sf += 1;
                        k += 1;
                    }

                    /* illegal is_pos used to do ms processing */
                    if ms_mode == 0 {
                        /* ms_mode = 0 */
                        unsafe {
                            (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][0] = 1.0f32;
                            (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][1] = 0.0f32;
                        }
                    } else {
                        /* ms_mode = 1, in is bands is routine does ms processing */
                        unsafe {
                            (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][0] = 1.0f32;
                            (*lr2.add(intensity_scale as usize))[ms_mode as usize][k as usize][1] = 1.0f32;
                        }
                    }
                    k += 1;
                    n = n + n;
                    sflen += 1;
                }
                ms_mode += 1;
            }
            intensity_scale += 1;
        }
    }
}

/*-------------------------------------------------------------*/
