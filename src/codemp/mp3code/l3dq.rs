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

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_int;
use core::ptr;

use super::l3_h::{CB_INFO, GR, SCALEFACT};
use super::mp3struct_h::pMP3Stream;
use super::small_header_h::SAMPLE;

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
static pretab: [[c_int; 22]; 2] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 3, 3, 2, 0],
];

////@@@@extern int nBand[2][22];	/* long = nBand[0][i], short = nBand[1][i] */

/* 8 bit plus 2 lookup x = pow(2.0, 0.25*(global_gain-210)) */
/* two extra slots to do 1/sqrt(2) scaling for ms */
/* 4 extra slots to do 1/2 scaling for cvt to mono */
static mut look_global: [f32; 256 + 2 + 4] = [0.0; 256 + 2 + 4]; /* effectively constant */

/*-------- scaling lookup
x = pow(2.0, -0.5*(1+scalefact_scale)*scalefac + preemp)
look_scale[scalefact_scale][preemp][scalefac]
-----------------------*/
static mut look_scale: [[[f32; 32]; 4]; 2] = [[[0.0; 32]; 4]; 2]; /* effectively constant */
pub type LS = [[f32; 32]; 4];

/*--- iSample**(4/3) lookup, -32<=i<=31 ---*/
pub const ISMAX: c_int = 32;
static mut look_pow: [f32; 2 * ISMAX as usize] = [0.0; 2 * ISMAX as usize]; /* effectively constant */

/*-- pow(2.0, -0.25*8.0*subblock_gain) --*/
static mut look_subblock: [f32; 8] = [0.0; 8]; /* effectively constant */

/*-- reorder buffer ---*/
static mut re_buf: [[f32; 3]; 192] = [[0.0; 3]; 192]; /* used by dequant() below, but only during func (as workspace) */
pub type ARRAY3 = [f32; 3];

/*=============================================================*/
pub unsafe extern "C" fn quant_init_global_addr() -> *mut f32 {
    ptr::addr_of_mut!(look_global).cast::<f32>()
}

/*-------------------------------------------------------------*/
pub unsafe extern "C" fn quant_init_scale_addr() -> *mut LS {
    ptr::addr_of_mut!(look_scale).cast::<LS>()
}

/*-------------------------------------------------------------*/
pub unsafe extern "C" fn quant_init_pow_addr() -> *mut f32 {
    ptr::addr_of_mut!(look_pow).cast::<f32>()
}

/*-------------------------------------------------------------*/
pub unsafe extern "C" fn quant_init_subblock_addr() -> *mut f32 {
    ptr::addr_of_mut!(look_subblock).cast::<f32>()
}

/*=============================================================*/

pub unsafe extern "C" fn dequant(
    Sample: *mut SAMPLE,
    nsamp: *mut c_int,
    sf: *mut SCALEFACT,
    gr: *mut GR,
    cb_info: *mut CB_INFO,
    ncbl_mixed: c_int,
) {
    let mut i: c_int;
    let mut j: c_int;
    let mut cb: c_int;
    let mut n: c_int;
    let mut w: c_int;
    let mut x0: f32;
    let mut xs: f32;
    let mut xsb: [f32; 3] = [0.0; 3];
    let mut tmp: f64;
    let mut ncbl: c_int;
    let mut cbs0: c_int;
    let mut buf: *mut ARRAY3; /* short block reorder */
    let nbands: c_int;
    let i0: c_int;
    let mut non_zero: c_int;
    let mut cbmax: [c_int; 3] = [0; 3];

    nbands = *nsamp;

    ncbl = 22; /* long block cb end */
    cbs0 = 12; /* short block cb start */
    /* ncbl_mixed = 8 or 6  mpeg1 or 2 */
    if (*gr).block_type == 2 {
        ncbl = 0;
        cbs0 = 0;
        if (*gr).mixed_block_flag != 0 {
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
    cbmax[1] = cbmax[2];
    cbmax[0] = cbmax[1];
    /* global gain pre-adjusted by 2 if ms_mode, 0 otherwise */
    x0 = *ptr::addr_of!(look_global).cast::<f32>().add(((2 + 4) + (*gr).global_gain) as usize);
    i = 0;
    /*----- long blocks ---*/
    cb = 0;
    while cb < ncbl {
        non_zero = 0;
        xs = x0
            * (*ptr::addr_of!(look_scale)
                .cast::<f32>()
                .add((((*gr).scalefac_scale * 4 * 32)
                    + (pretab[(*gr).preflag as usize][cb as usize] * 32)
                    + (*sf).l[cb as usize]) as usize));
        n = (*pMP3Stream).u.L3.nBand[0][cb as usize];
        j = 0;
        while j < n {
            if (*Sample.add(i as usize)).s == 0 {
                (*Sample.add(i as usize)).x = 0.0f32;
            } else {
                non_zero = 1;
                if ((*Sample.add(i as usize)).s >= -ISMAX) && ((*Sample.add(i as usize)).s < ISMAX) {
                    (*Sample.add(i as usize)).x = xs
                        * *ptr::addr_of!(look_pow)
                            .cast::<f32>()
                            .add((ISMAX + (*Sample.add(i as usize)).s) as usize);
                } else {
                    let tmpConst: f32 = (1.0f64 / 3.0f64) as f32;
                    tmp = (*Sample.add(i as usize)).s as f64;
                    (*Sample.add(i as usize)).x =
                        (xs as f64 * tmp * tmp.abs().powf(tmpConst as f64)) as f32;
                }
            }
            j += 1;
            i += 1;
        }
        if non_zero != 0 {
            cbmax[0] = cb;
        }
        if i >= nbands {
            break;
        }
        cb += 1;
    }

    (*cb_info).cbmax = cbmax[0];
    (*cb_info).cbtype = 0; /* type = long */

    if cbs0 >= 12 {
        return;
    }
    /*---------------------------
    block type = 2  short blocks
    ----------------------------*/
    cbmax[2] = cbs0;
    cbmax[1] = cbmax[2];
    cbmax[0] = cbmax[1];
    i0 = i; /* save for reorder */
    buf = ptr::addr_of_mut!(re_buf).cast::<ARRAY3>();
    w = 0;
    while w < 3 {
        xsb[w as usize] = x0
            * *ptr::addr_of!(look_subblock)
                .cast::<f32>()
                .add((*gr).subblock_gain[w as usize] as usize);
        w += 1;
    }
    cb = cbs0;
    while cb < 13 {
        n = (*pMP3Stream).u.L3.nBand[1][cb as usize];
        w = 0;
        while w < 3 {
            non_zero = 0;
            xs = xsb[w as usize]
                * (*ptr::addr_of!(look_scale)
                    .cast::<f32>()
                    .add((((*gr).scalefac_scale * 4 * 32)
                        + (*sf).s[w as usize][cb as usize]) as usize));
            j = 0;
            while j < n {
                if (*Sample.add(i as usize)).s == 0 {
                    (*buf.add(j as usize))[w as usize] = 0.0f32;
                } else {
                    non_zero = 1;
                    if ((*Sample.add(i as usize)).s >= -ISMAX)
                        && ((*Sample.add(i as usize)).s < ISMAX)
                    {
                        (*buf.add(j as usize))[w as usize] = xs
                            * *ptr::addr_of!(look_pow)
                                .cast::<f32>()
                                .add((ISMAX + (*Sample.add(i as usize)).s) as usize);
                    } else {
                        let tmpConst: f32 = (1.0f64 / 3.0f64) as f32;
                        tmp = (*Sample.add(i as usize)).s as f64;
                        (*buf.add(j as usize))[w as usize] =
                            (xs as f64 * tmp * tmp.abs().powf(tmpConst as f64)) as f32;
                    }
                }
                j += 1;
                i += 1;
            }
            if non_zero != 0 {
                cbmax[w as usize] = cb;
            }
            w += 1;
        }
        if i >= nbands {
            break;
        }
        buf = buf.add(n as usize);
        cb += 1;
    }

    ptr::copy(
        ptr::addr_of!(re_buf).cast::<f32>(),
        Sample.add(i0 as usize).cast::<f32>(),
        (i - i0) as usize,
    );

    *nsamp = i; /* update nsamp */
    (*cb_info).cbmax_s[0] = cbmax[0];
    (*cb_info).cbmax_s[1] = cbmax[1];
    (*cb_info).cbmax_s[2] = cbmax[2];
    if cbmax[1] > cbmax[0] {
        cbmax[0] = cbmax[1];
    }
    if cbmax[2] > cbmax[0] {
        cbmax[0] = cbmax[2];
    }

    (*cb_info).cbmax = cbmax[0];
    (*cb_info).cbtype = 1; /* type = short */

    return;
}
