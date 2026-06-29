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

use core::ffi::c_int;
use core::f64::consts::PI;

/* get rid of precision loss warnings on conversion */
/* MSVC pragma warning(disable:4244 4056) not needed in Rust */


/*---------- quant ---------------------------------*/
/* 8 bit lookup x = pow(2.0, 0.25*(global_gain-210)) */
extern "C" {
    fn quant_init_global_addr() -> *mut f32;
}

/* x = pow(2.0, -0.5*(1+scalefact_scale)*scalefac + preemp) */
type LS = [[f32; 32]; 4];
extern "C" {
    fn quant_init_scale_addr() -> *mut LS;
}

extern "C" {
    fn quant_init_pow_addr() -> *mut f32;
    fn quant_init_subblock_addr() -> *mut f32;
}

type iARRAY22 = [c_int; 22];
extern "C" {
    fn quant_init_band_addr() -> *mut iARRAY22;
}

/*---------- antialias ---------------------------------*/
type PAIR = [f32; 2];
extern "C" {
    fn alias_init_addr() -> *mut PAIR;
}

const Ci: [f32; 8] = [
   -0.6f32, -0.535f32, -0.33f32, -0.185f32, -0.095f32, -0.041f32, -0.0142f32, -0.0037f32
];

extern "C" {
    fn hwin_init();		/* hybrid windows -- */
    fn imdct_init();
}

#[repr(C)]
struct IMDCT_INIT_BLOCK {
    w: *mut f32,
    w2: *mut f32,
    coef: *mut core::ffi::c_void,
}

extern "C" {
    fn msis_init();
    fn msis_init_MPEG2();
}

/*=============================================================*/
pub extern "C" fn L3table_init() -> c_int
{
    let mut i: c_int;
    let mut x: *mut f32;
    let mut ls: *mut LS;
    let mut scalefact_scale: c_int;
    let mut preemp: c_int;
    let mut scalefac: c_int;
    let mut tmp: f64;
    let mut csa: *mut PAIR;

    static mut iOnceOnly: c_int = 0;

    unsafe {
        if {
            let val = iOnceOnly;
            iOnceOnly += 1;
            val == 0
        }
        {
/*================ quant ===============================*/

            /* 8 bit plus 2 lookup x = pow(2.0, 0.25*(global_gain-210)) */
            /* extra 2 for ms scaling by 1/sqrt(2) */
            /* extra 4 for cvt to mono scaling by 1/2 */
            x = quant_init_global_addr();
            i = 0;
            while i < 256 + 2 + 4 {
                *x.offset(i as isize) = (2.0_f64.powf(0.25 * ((i as f64 - (2 + 4) as f64) - 210.0 + GLOBAL_GAIN_SCALE))) as f32;
                i += 1;
            }



            /* x = pow(2.0, -0.5*(1+scalefact_scale)*scalefac + preemp) */
            ls = quant_init_scale_addr();
            scalefact_scale = 0;
            while scalefact_scale < 2 {
                preemp = 0;
                while preemp < 4 {
                    scalefac = 0;
                    while scalefac < 32 {
                        (*ls)[scalefact_scale as usize][preemp as usize][scalefac as usize] =
                           (2.0_f64.powf(-0.5 * (1.0 + scalefact_scale as f64) * (scalefac as f64 + preemp as f64))) as f32;
                        scalefac += 1;
                    }
                    preemp += 1;
                }
                scalefact_scale += 1;
            }

            /*--- iSample**(4/3) lookup, -32<=i<=31 ---*/
            x = quant_init_pow_addr();
            i = 0;
            while i < 64 {
                tmp = i as f64 - 32.0;
                *x.offset(i as isize) = (tmp * tmp.abs().powf(1.0 / 3.0)) as f32;
                i += 1;
            }


            /*-- pow(2.0, -0.25*8.0*subblock_gain)  3 bits --*/
            x = quant_init_subblock_addr();
            i = 0;
            while i < 8 {
                *x.offset(i as isize) = (2.0_f64.powf(0.25 * -8.0 * i as f64)) as f32;
                i += 1;
            }

            /*-------------------------*/
            // quant_init_sf_band(sr_index);   replaced by code in sup.c


/*================ antialias ===============================*/
            // onceonly!!!!!!!!!!!!!!!!!!!!!
            csa = alias_init_addr();
            i = 0;
            while i < 8 {
                (*csa)[i as usize][0] = (1.0 / (1.0 + Ci[i as usize] as f64 * Ci[i as usize] as f64).sqrt()) as f32;
                (*csa)[i as usize][1] = (Ci[i as usize] as f64 / (1.0 + Ci[i as usize] as f64 * Ci[i as usize] as f64).sqrt()) as f32;
                i += 1;
            }
        }

        // these 4 are iOnceOnly-protected inside...

/*================ msis ===============================*/
        msis_init();
        msis_init_MPEG2();

/*================ imdct ===============================*/
        imdct_init();

/*--- hybrid windows ------------*/
        hwin_init();
    }

    0
}
/*====================================================================*/
type ARRAY36 = [f32; 36];
extern "C" {
    fn hwin_init_addr() -> *mut ARRAY36;
}

/*--------------------------------------------------------------------*/
pub extern "C" fn hwin_init()
{
    let mut i: c_int;
    let mut j: c_int;
    let mut pi: f64;
    let mut win: *mut ARRAY36;

    static mut iOnceOnly: c_int = 0;

    unsafe {
        if {
            let val = iOnceOnly;
            iOnceOnly += 1;
            val == 0
        }
        {
            win = hwin_init_addr();

            pi = 4.0 * (1.0_f64).atan();

            /* type 0 */
            i = 0;
            while i < 36 {
                (*win)[0][i as usize] = (pi / 36.0 * (i as f64 + 0.5)).sin() as f32;
                i += 1;
            }

            /* type 1 */
            i = 0;
            while i < 18 {
                (*win)[1][i as usize] = (pi / 36.0 * (i as f64 + 0.5)).sin() as f32;
                i += 1;
            }
            while i < 24 {
                (*win)[1][i as usize] = 1.0f32;
                i += 1;
            }
            while i < 30 {
                (*win)[1][i as usize] = (pi / 12.0 * (i as f64 + 0.5 - 18.0)).sin() as f32;
                i += 1;
            }
            while i < 36 {
                (*win)[1][i as usize] = 0.0f32;
                i += 1;
            }

            /* type 3 */
            i = 0;
            while i < 6 {
                (*win)[3][i as usize] = 0.0f32;
                i += 1;
            }
            while i < 12 {
                (*win)[3][i as usize] = (pi / 12.0 * (i as f64 + 0.5 - 6.0)).sin() as f32;
                i += 1;
            }
            while i < 18 {
                (*win)[3][i as usize] = 1.0f32;
                i += 1;
            }
            while i < 36 {
                (*win)[3][i as usize] = (pi / 36.0 * (i as f64 + 0.5)).sin() as f32;
                i += 1;
            }

            /* type 2 */
            i = 0;
            while i < 12 {
                (*win)[2][i as usize] = (pi / 12.0 * (i as f64 + 0.5)).sin() as f32;
                i += 1;
            }
            while i < 36 {
                (*win)[2][i as usize] = 0.0f32;
                i += 1;
            }

            /*--- invert signs by region to match mdct 18pt --> 36pt mapping */
            j = 0;
            while j < 4 {
                if j != 2 {
                    i = 9;
                    while i < 36 {
                        (*win)[j as usize][i as usize] = -(*win)[j as usize][i as usize];
                        i += 1;
                    }
                }
                j += 1;
            }

            /*-- invert signs for short blocks --*/
            i = 3;
            while i < 12 {
                (*win)[2][i as usize] = -(*win)[2][i as usize];
                i += 1;
            }
        }
    }
}
/*=============================================================*/
type ARRAY4 = [f32; 4];
extern "C" {
    fn imdct_init_addr_18() -> *const IMDCT_INIT_BLOCK;
    fn imdct_init_addr_6() -> *const IMDCT_INIT_BLOCK;
}

/*-------------------------------------------------------------*/
pub extern "C" fn imdct_init()
{
    let mut k: c_int;
    let mut p: c_int;
    let mut n: c_int;
    let mut t: f64;
    let mut pi: f64;
    let mut addr: *const IMDCT_INIT_BLOCK;
    let mut w: *mut f32;
    let mut w2: *mut f32;
    let mut v: *mut f32;
    let mut v2: *mut f32;
    let mut coef87: *mut f32;
    let mut coef: *mut ARRAY4;

    static mut iOnceOnly: c_int = 0;

    unsafe {
        if {
            let val = iOnceOnly;
            iOnceOnly += 1;
            val == 0
        }
        {
            /*--- 18 point --*/
            addr = imdct_init_addr_18();
            w = (*addr).w;
            w2 = (*addr).w2;
            coef = (*addr).coef as *mut ARRAY4;
            /*----*/
            n = 18;
            pi = 4.0 * (1.0_f64).atan();
            t = pi / (4.0 * n as f64);
            p = 0;
            while p < n {
                *w.offset(p as isize) = (2.0 * (t * (2.0 * p as f64 + 1.0)).cos()) as f32;
                p += 1;
            }
            p = 0;
            while p < 9 {
                *w2.offset(p as isize) = (2.0 * (2.0 * t * (2.0 * p as f64 + 1.0)).cos()) as f32;
                p += 1;
            }

            t = pi / (2.0 * n as f64);
            k = 0;
            while k < 9 {
                p = 0;
                while p < 4 {
                    (*coef)[k as usize][p as usize] = ((t * (2.0 * k as f64) * (2.0 * p as f64 + 1.0)).cos()) as f32;
                    p += 1;
                }
                k += 1;
            }

            /*--- 6 point */
            addr = imdct_init_addr_6();
            v = (*addr).w;
            v2 = (*addr).w2;
            coef87 = (*addr).coef as *mut f32;
            /*----*/
            n = 6;
            pi = 4.0 * (1.0_f64).atan();
            t = pi / (4.0 * n as f64);
            p = 0;
            while p < n {
                *v.offset(p as isize) = (2.0 * (t * (2.0 * p as f64 + 1.0)).cos()) as f32;
                p += 1;
            }

            p = 0;
            while p < 3 {
                *v2.offset(p as isize) = (2.0 * (2.0 * t * (2.0 * p as f64 + 1.0)).cos()) as f32;
                p += 1;
            }

            t = pi / (2.0 * n as f64);
            k = 1;
            p = 0;
            *coef87 = ((t * (2.0 * k as f64) * (2.0 * p as f64 + 1.0)).cos()) as f32;
            /* adjust scaling to save a few mults */
            p = 0;
            while p < 6 {
                *v.offset(p as isize) = *v.offset(p as isize) / 2.0f32;
                p += 1;
            }
            *coef87 = (2.0 * (*coef87 as f64)) as f32;

        }
    }
}
/*===============================================================*/
type ARRAY8_2 = [[f32; 2]; 8];
extern "C" {
    fn msis_init_addr() -> *mut ARRAY8_2;
}

/*-------------------------------------------------------------*/
pub extern "C" fn msis_init()
{
    let mut i: c_int;
    let mut s: f64;
    let mut c: f64;
    let mut pi: f64;
    let mut t: f64;
    let mut lr: *mut ARRAY8_2;

    static mut iOnceOnly: c_int = 0;

    unsafe {
        if {
            let val = iOnceOnly;
            iOnceOnly += 1;
            val == 0
        }
        {
            lr = msis_init_addr();


            pi = 4.0 * (1.0_f64).atan();
            t = pi / 12.0;
            i = 0;
            while i < 7 {
                s = (i as f64 * t).sin();
                c = (i as f64 * t).cos();
                /* ms_mode = 0 */
                (*lr)[0][i as usize][0] = (s / (s + c)) as f32;
                (*lr)[0][i as usize][1] = (c / (s + c)) as f32;
                /* ms_mode = 1 */
                (*lr)[1][i as usize][0] = ((2.0_f64.sqrt()) * (s / (s + c))) as f32;
                (*lr)[1][i as usize][1] = ((2.0_f64.sqrt()) * (c / (s + c))) as f32;
                i += 1;
            }
            /* sf = 7 */
            /* ms_mode = 0 */
            (*lr)[0][i as usize][0] = 1.0f32;
            (*lr)[0][i as usize][1] = 0.0f32;
            /* ms_mode = 1, in is bands is routine does ms processing */
            (*lr)[1][i as usize][0] = 1.0f32;
            (*lr)[1][i as usize][1] = 1.0f32;


            /*-------
            for(i=0;i<21;i++) nBand[0][i] =
                        sfBandTable[sr_index].l[i+1] - sfBandTable[sr_index].l[i];
            for(i=0;i<12;i++) nBand[1][i] =
                        sfBandTable[sr_index].s[i+1] - sfBandTable[sr_index].s[i];
            -------------*/
        }
    }
}
/*-------------------------------------------------------------*/
/*===============================================================*/
type ARRAY2_64_2 = [[[f32; 2]; 64]; 2];
extern "C" {
    fn msis_init_addr_MPEG2() -> *mut ARRAY2_64_2;
}

/*-------------------------------------------------------------*/
pub extern "C" fn msis_init_MPEG2()
{
    let mut k: c_int;
    let mut n: c_int;
    let mut t: f64;
    let mut lr2: *mut ARRAY2_64_2;
    let mut intensity_scale: c_int;
    let mut ms_mode: c_int;
    let mut sf: c_int;
    let mut sflen: c_int;
    let mut ms_factor: [f32; 2];

    static mut iOnceOnly: c_int = 0;

    unsafe {
        if {
            let val = iOnceOnly;
            iOnceOnly += 1;
            val == 0
        }
        {
            ms_factor[0] = 1.0f32;
            ms_factor[1] = (2.0_f64.sqrt()) as f32;

            lr2 = msis_init_addr_MPEG2();

            /* intensity stereo MPEG2 */
            /* lr2[intensity_scale][ms_mode][sflen_offset+sf][left/right] */

            intensity_scale = 0;
            while intensity_scale < 2 {
                t = 2.0_f64.powf(-0.25 * (1.0 + intensity_scale as f64));
                ms_mode = 0;
                while ms_mode < 2 {

                    n = 1;
                    k = 0;
                    sflen = 0;
                    while sflen < 6 {
                        sf = 0;
                        while sf < (n - 1) {
                            if sf == 0
                            {
                                (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][0] = ms_factor[ms_mode as usize] * 1.0f32;
                                (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][1] = ms_factor[ms_mode as usize] * 1.0f32;
                            }
                            else if ((sf & 1) != 0)
                            {
                                (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][0] =
                                   (ms_factor[ms_mode as usize] as f64 * t.powf((sf as f64 + 1.0) / 2.0)) as f32;
                                (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][1] = ms_factor[ms_mode as usize] * 1.0f32;
                            }
                            else
                            {
                                (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][0] = ms_factor[ms_mode as usize] * 1.0f32;
                                (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][1] =
                                   (ms_factor[ms_mode as usize] as f64 * t.powf(sf as f64 / 2.0)) as f32;
                            }
                            sf += 1;
                            k += 1;
                        }

                        /* illegal is_pos used to do ms processing */
                        if ms_mode == 0
                        {			/* ms_mode = 0 */
                            (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][0] = 1.0f32;
                            (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][1] = 0.0f32;
                        }
                        else
                        {
                            /* ms_mode = 1, in is bands is routine does ms processing */
                            (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][0] = 1.0f32;
                            (*lr2)[intensity_scale as usize][ms_mode as usize][k as usize][1] = 1.0f32;
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

}
/*-------------------------------------------------------------*/

// GLOBAL_GAIN_SCALE is an external constant that should be defined elsewhere
// If not defined, this will need an extern declaration or local stub
extern "C" {
    static GLOBAL_GAIN_SCALE: f64;
}
