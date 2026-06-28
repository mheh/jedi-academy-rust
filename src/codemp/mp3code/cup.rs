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

    $Id: cup.c,v 1.3 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  cup.c  ***************************************************

MPEG audio decoder Layer I/II  mpeg1 and mpeg2
should be portable ANSI C, should be endian independent


mod  2/21/95 2/21/95  add bit skip, sb limiting

mods 11/15/95 for Layer I

******************************************************************/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_long, c_short, c_uchar, c_uint, c_ulong};
use core::ptr::{addr_of_mut, null_mut};

use super::mhead_h::DEC_INFO;
use super::mp3struct_h::pMP3Stream;
use super::small_header_h::IN_OUT;

/*-------------------------------------------------------
NOTE:  Decoder may read up to three bytes beyond end of
frame.  Calling application must ensure that this does
not cause a memory access violation (protection fault)
---------------------------------------------------------*/

pub type AUDIO_DECODE_ROUTINE =
    Option<unsafe extern "C" fn(bs: *mut c_uchar, pcm: *mut c_short) -> IN_OUT>;

/*====================================================================*/
/*----------------*/
//@@@@ This next one (decinfo) is ok:
#[no_mangle]
pub static mut decinfo: DEC_INFO = DEC_INFO {
    channels: 0,
    outvalues: 0,
    samprate: 0,
    bits: 0,
    framebytes: 0,
    r#type: 0,
}; /* global for Layer III */ // only written into by decode init funcs, then copied to stack struct higher up

/*----------------*/
#[no_mangle]
pub static mut look_c_value: [f32; 18] = [0.0; 18]; /* built by init */ // effectively constant

/*----------------*/
////@@@@static int pMP3Stream->outbytes;		// !!!!!!!!!!!!!!?
////@@@@static int pMP3Stream->framebytes;		// !!!!!!!!!!!!!!!!
////@@@@static int pMP3Stream->outvalues;		// !!!!!!!!!!!!?
////@@@@static int pad;
static look_joint: [c_int; 16] = [
    64, 64, 64, 64, /* stereo */
    2 * 4, 2 * 8, 2 * 12, 2 * 16, /* joint */
    64, 64, 64, 64, /* dual */
    32, 32, 32, 32, /* mono */
];

/*----------------*/
////@@@@static int max_sb;		// !!!!!!!!! L1, 2 3
////@@@@static int stereo_sb;

/*----------------*/
////@@@@static int pMP3Stream->nsb_limit = 6;
////@@@@static int bit_skip;
static bat_bit_master: [c_int; 18] = [
    0, 5, 7, 9, 10, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 48,
];

/*----------------*/
////@@@@static int nbat[4] = {3, 8, 12, 7};	// !!!!!!!!!!!!! not constant!!!!
////@@@@static int bat[4][16];	// built as constant, but built according to header type (sigh)
static mut ballo: [c_int; 64] = [0; 64]; /* set by unpack_ba */ // scratchpad
static mut samp_dispatch: [c_uint; 66] = [0; 66]; /* set by unpack_ba */ // scratchpad?
static mut c_value: [f32; 64] = [0.0; 64]; /* set by unpack_ba */ // scratchpad

/*----------------*/
static mut sf_dispatch: [c_uint; 66] = [0; 66]; /* set by unpack_ba */ // scratchpad?
#[no_mangle]
pub static mut sf_table: [f32; 64] = [0.0; 64]; // effectively constant
////@@@@ static float cs_factor[3][64];

/*----------------*/
////@@@@FINDME - groan....  (I shoved a *2 in just in case it needed it for stereo. This whole thing is crap now
#[no_mangle]
pub static mut sample: [f32; 2304 * 2] = [0.0; 2304 * 2]; /* global for use by Later 3 */ // !!!!!!!!!!!!!!!!!!!!!! // scratchpad?
#[no_mangle]
pub static mut group3_table: [[i8; 3]; 32] = [[0; 3]; 32]; // effectively constant
#[no_mangle]
pub static mut group5_table: [[i8; 3]; 128] = [[0; 3]; 128]; // effectively constant
#[no_mangle]
pub static mut group9_table: [[c_short; 3]; 1024] = [[0; 3]; 1024]; // effectively constant

/*----------------*/

////@@@@typedef void (*SBT_FUNCTION) (float *sample, short *pcm, int n);
unsafe extern "C" {
    pub fn sbt_mono(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt_dual(sample: *mut f32, pcm: *mut c_short, n: c_int);
}
////@@@@static SBT_FUNCTION sbt = sbt_mono;

#[no_mangle]
pub static mut audio_decode_routine: AUDIO_DECODE_ROUTINE = Some(L2audio_decode);

/*======================================================================*/
/*======================================================================*/
/* get bits from bitstream in endian independent way */
////@@@@ FINDME - this stuff doesn't appear to be used by any of our samples (phew)
static mut bs_ptr: *mut c_uchar = null_mut();
static mut bitbuf: c_ulong = 0;
static mut bits: c_int = 0;
static mut bitval: c_long = 0;

/*------------- initialize bit getter -------------*/
unsafe fn load_init(buf: *mut c_uchar) {
    bs_ptr = buf;
    bits = 0;
    bitbuf = 0;
}

/*------------- get n bits from bitstream -------------*/
unsafe fn load(n: c_int) -> c_long {
    let x: c_ulong;

    if bits < n {
        /* refill bit buf if necessary */
        while bits <= 24 {
            bitbuf = (bitbuf << 8) | *bs_ptr as c_ulong;
            bs_ptr = bs_ptr.add(1);
            bits += 8;
        }
    }
    bits -= n;
    x = bitbuf >> bits;
    bitbuf -= x << bits;
    x as c_long
}

/*------------- skip over n bits in bitstream -------------*/
unsafe fn skip(mut n: c_int) {
    let k: c_int;

    if bits < n {
        n -= bits;
        k = n >> 3;
        /*--- bytes = n/8 --*/
        bs_ptr = bs_ptr.offset(k as isize);
        n -= k << 3;
        bitbuf = *bs_ptr as c_ulong;
        bs_ptr = bs_ptr.add(1);
        bits = 8;
    }
    bits -= n;
    bitbuf -= (bitbuf >> bits) << bits;
}

/*--------------------------------------------------------------*/
unsafe fn mac_load_check(n: c_int) {
    if bits < n {
        while bits <= 24 {
            bitbuf = (bitbuf << 8) | *bs_ptr as c_ulong;
            bs_ptr = bs_ptr.add(1);
            bits += 8;
        }
    }
}

/*--------------------------------------------------------------*/
unsafe fn mac_load(n: c_int) -> c_long {
    bits -= n;
    bitval = (bitbuf >> bits) as c_long;
    bitbuf -= (bitval as c_ulong) << bits;
    bitval
}

/*======================================================================*/
unsafe fn unpack_ba() {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    static nbit: [c_int; 4] = [4, 4, 3, 2];
    let mut nstereo: c_int;

    (*pMP3Stream).u.L1_2.bit_skip = 0;
    nstereo = (*pMP3Stream).u.L1_2.stereo_sb;
    k = 0;
    i = 0;
    while i < 4 {
        j = 0;
        while j < (*pMP3Stream).u.L1_2.nbat[i as usize] {
            mac_load_check(4);
            samp_dispatch[k as usize] =
                (*pMP3Stream).u.L1_2.bat[i as usize][mac_load(nbit[i as usize]) as usize] as c_uint;
            ballo[k as usize] = samp_dispatch[k as usize] as c_int;
            if k >= (*pMP3Stream).nsb_limit {
                (*pMP3Stream).u.L1_2.bit_skip += bat_bit_master[samp_dispatch[k as usize] as usize];
            }
            c_value[k as usize] = look_c_value[samp_dispatch[k as usize] as usize];
            nstereo -= 1;
            if nstereo < 0 {
                ballo[(k + 1) as usize] = ballo[k as usize];
                samp_dispatch[k as usize] += 18; /* flag as joint */
                samp_dispatch[(k + 1) as usize] = samp_dispatch[k as usize]; /* flag for sf */
                c_value[(k + 1) as usize] = c_value[k as usize];
                k += 1;
                j += 1;
            }
            j += 1;
            k += 1;
        }
        i += 1;
    }
    samp_dispatch[(*pMP3Stream).nsb_limit as usize] = 37; /* terminate the dispatcher with skip */
    samp_dispatch[k as usize] = 36; /* terminate the dispatcher */
}

/*-------------------------------------------------------------------------*/
unsafe fn unpack_sfs() {
    /* unpack scale factor selectors */
    let mut i: c_int;

    i = 0;
    while i < (*pMP3Stream).u.L1_2.max_sb {
        mac_load_check(2);
        if ballo[i as usize] != 0 {
            sf_dispatch[i as usize] = mac_load(2) as c_uint;
        } else {
            sf_dispatch[i as usize] = 4; /* no allo */
        }
        i += 1;
    }
    sf_dispatch[i as usize] = 5; /* terminate dispatcher */
}

/*-------------------------------------------------------------------------*/
unsafe fn unpack_sf() {
    /* unpack scale factor */
    /* combine dequant and scale factors */
    let mut i: c_int;

    i = -1;
    loop {
        i += 1;
        match sf_dispatch[i as usize] {
            0 => {
                /* 3 factors 012 */
                mac_load_check(18);
                (*pMP3Stream).u.L1_2.cs_factor[0][i as usize] =
                    c_value[i as usize] * sf_table[mac_load(6) as usize];
                (*pMP3Stream).u.L1_2.cs_factor[1][i as usize] =
                    c_value[i as usize] * sf_table[mac_load(6) as usize];
                (*pMP3Stream).u.L1_2.cs_factor[2][i as usize] =
                    c_value[i as usize] * sf_table[mac_load(6) as usize];
            }
            1 => {
                /* 2 factors 002 */
                mac_load_check(12);
                let v = c_value[i as usize] * sf_table[mac_load(6) as usize];
                (*pMP3Stream).u.L1_2.cs_factor[0][i as usize] = v;
                (*pMP3Stream).u.L1_2.cs_factor[1][i as usize] = v;
                (*pMP3Stream).u.L1_2.cs_factor[2][i as usize] =
                    c_value[i as usize] * sf_table[mac_load(6) as usize];
            }
            2 => {
                /* 1 factor 000 */
                mac_load_check(6);
                let v = c_value[i as usize] * sf_table[mac_load(6) as usize];
                (*pMP3Stream).u.L1_2.cs_factor[0][i as usize] = v;
                (*pMP3Stream).u.L1_2.cs_factor[1][i as usize] = v;
                (*pMP3Stream).u.L1_2.cs_factor[2][i as usize] = v;
            }
            3 => {
                /* 2 factors 022 */
                mac_load_check(12);
                (*pMP3Stream).u.L1_2.cs_factor[0][i as usize] =
                    c_value[i as usize] * sf_table[mac_load(6) as usize];
                let v = c_value[i as usize] * sf_table[mac_load(6) as usize];
                (*pMP3Stream).u.L1_2.cs_factor[1][i as usize] = v;
                (*pMP3Stream).u.L1_2.cs_factor[2][i as usize] = v;
            }
            4 => {
                /* no allo */
                /*-- pMP3Stream->cs_factor[2][i] = pMP3Stream->cs_factor[1][i] = pMP3Stream->cs_factor[0][i] = 0.0;  --*/
            }
            5 => {
                /* all done */
                break;
            }
            _ => break,
        }
    } /* end switch */
}

unsafe fn unpack_n(s: *mut f32, i: c_int, k: c_int, n: c_int) {
    *s.add(k as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * (load(n) - ((1 << (n - 1)) - 1)) as f32;
    *s.add((k + 64) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * (load(n) - ((1 << (n - 1)) - 1)) as f32;
    *s.add((k + 128) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * (load(n) - ((1 << (n - 1)) - 1)) as f32;
}

unsafe fn unpack_n2(s: *mut f32, i: c_int, k: c_int, n: c_int) {
    mac_load_check(3 * n);
    *s.add(k as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * (mac_load(n) - ((1 << (n - 1)) - 1)) as f32;
    *s.add((k + 64) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * (mac_load(n) - ((1 << (n - 1)) - 1)) as f32;
    *s.add((k + 128) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * (mac_load(n) - ((1 << (n - 1)) - 1)) as f32;
}

unsafe fn unpack_n3(s: *mut f32, i: c_int, k: c_int, n: c_int) {
    mac_load_check(2 * n);
    *s.add(k as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * (mac_load(n) - ((1 << (n - 1)) - 1)) as f32;
    *s.add((k + 64) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * (mac_load(n) - ((1 << (n - 1)) - 1)) as f32;
    mac_load_check(n);
    *s.add((k + 128) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * (mac_load(n) - ((1 << (n - 1)) - 1)) as f32;
}

unsafe fn unpackj_n(s: *mut f32, i: c_int, k: c_int, n: c_int) {
    let mut tmp: c_long;

    tmp = load(n) - ((1 << (n - 1)) - 1);
    *s.add(k as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * tmp as f32;
    *s.add((k + 1) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize] * tmp as f32;
    tmp = load(n) - ((1 << (n - 1)) - 1);
    *s.add((k + 64) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * tmp as f32;
    *s.add((k + 64 + 1) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize] * tmp as f32;
    tmp = load(n) - ((1 << (n - 1)) - 1);
    *s.add((k + 128) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize] * tmp as f32;
    *s.add((k + 128 + 1) as usize) =
        (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize] * tmp as f32;
}

/*-------------------------------------------------------------------------*/
unsafe fn unpack_samp() {
    /* unpack samples */
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut s: *mut f32;
    let mut n: c_int;

    s = addr_of_mut!(sample).cast::<f32>();
    i = 0;
    while i < 3 {
        /* 3 groups of scale factors */
        j = 0;
        while j < 4 {
            k = -1;
            loop {
                k += 1;
                match samp_dispatch[k as usize] {
                    0 => {
                        *s.add((k + 128) as usize) = 0.0f32;
                        *s.add((k + 64) as usize) = 0.0f32;
                        *s.add(k as usize) = 0.0f32;
                    }
                    1 => {
                        /* 3 levels grouped 5 bits */
                        mac_load_check(5);
                        n = mac_load(5) as c_int;
                        *s.add(k as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group3_table[n as usize][0] as f32;
                        *s.add((k + 64) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group3_table[n as usize][1] as f32;
                        *s.add((k + 128) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group3_table[n as usize][2] as f32;
                    }
                    2 => {
                        /* 5 levels grouped 7 bits */
                        mac_load_check(7);
                        n = mac_load(7) as c_int;
                        *s.add(k as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group5_table[n as usize][0] as f32;
                        *s.add((k + 64) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group5_table[n as usize][1] as f32;
                        *s.add((k + 128) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group5_table[n as usize][2] as f32;
                    }
                    3 => unpack_n2(s, i, k, 3), /* 7 levels */
                    4 => {
                        /* 9 levels grouped 10 bits */
                        mac_load_check(10);
                        n = mac_load(10) as c_int;
                        *s.add(k as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group9_table[n as usize][0] as f32;
                        *s.add((k + 64) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group9_table[n as usize][1] as f32;
                        *s.add((k + 128) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group9_table[n as usize][2] as f32;
                    }
                    5 => unpack_n2(s, i, k, 4),   /* 15 levels */
                    6 => unpack_n2(s, i, k, 5),   /* 31 levels */
                    7 => unpack_n2(s, i, k, 6),   /* 63 levels */
                    8 => unpack_n2(s, i, k, 7),   /* 127 levels */
                    9 => unpack_n2(s, i, k, 8),   /* 255 levels */
                    10 => unpack_n3(s, i, k, 9),  /* 511 levels */
                    11 => unpack_n3(s, i, k, 10), /* 1023 levels */
                    12 => unpack_n3(s, i, k, 11), /* 2047 levels */
                    13 => unpack_n3(s, i, k, 12), /* 4095 levels */
                    14 => unpack_n(s, i, k, 13),  /* 8191 levels */
                    15 => unpack_n(s, i, k, 14),  /* 16383 levels */
                    16 => unpack_n(s, i, k, 15),  /* 32767 levels */
                    17 => unpack_n(s, i, k, 16),  /* 65535 levels */
                    /* -- joint ---- */
                    18 => {
                        *s.add((k + 128 + 1) as usize) = 0.0f32;
                        *s.add((k + 128) as usize) = 0.0f32;
                        *s.add((k + 64 + 1) as usize) = 0.0f32;
                        *s.add((k + 64) as usize) = 0.0f32;
                        *s.add((k + 1) as usize) = 0.0f32;
                        *s.add(k as usize) = 0.0f32;
                        k += 1; /* skip right chan dispatch */
                    }
                    19 => {
                        /* 3 levels grouped 5 bits */
                        n = load(5) as c_int;
                        *s.add(k as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group3_table[n as usize][0] as f32;
                        *s.add((k + 1) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize]
                            * group3_table[n as usize][0] as f32;
                        *s.add((k + 64) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group3_table[n as usize][1] as f32;
                        *s.add((k + 64 + 1) as usize) =
                            (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize]
                                * group3_table[n as usize][1] as f32;
                        *s.add((k + 128) as usize) =
                            (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                                * group3_table[n as usize][2] as f32;
                        *s.add((k + 128 + 1) as usize) =
                            (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize]
                                * group3_table[n as usize][2] as f32;
                        k += 1; /* skip right chan dispatch */
                    }
                    20 => {
                        /* 5 levels grouped 7 bits */
                        n = load(7) as c_int;
                        *s.add(k as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group5_table[n as usize][0] as f32;
                        *s.add((k + 1) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize]
                            * group5_table[n as usize][0] as f32;
                        *s.add((k + 64) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group5_table[n as usize][1] as f32;
                        *s.add((k + 64 + 1) as usize) =
                            (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize]
                                * group5_table[n as usize][1] as f32;
                        *s.add((k + 128) as usize) =
                            (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                                * group5_table[n as usize][2] as f32;
                        *s.add((k + 128 + 1) as usize) =
                            (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize]
                                * group5_table[n as usize][2] as f32;
                        k += 1; /* skip right chan dispatch */
                    }
                    21 => {
                        unpackj_n(s, i, k, 3); /* 7 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    22 => {
                        /* 9 levels grouped 10 bits */
                        n = load(10) as c_int;
                        *s.add(k as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group9_table[n as usize][0] as f32;
                        *s.add((k + 1) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize]
                            * group9_table[n as usize][0] as f32;
                        *s.add((k + 64) as usize) = (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                            * group9_table[n as usize][1] as f32;
                        *s.add((k + 64 + 1) as usize) =
                            (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize]
                                * group9_table[n as usize][1] as f32;
                        *s.add((k + 128) as usize) =
                            (*pMP3Stream).u.L1_2.cs_factor[i as usize][k as usize]
                                * group9_table[n as usize][2] as f32;
                        *s.add((k + 128 + 1) as usize) =
                            (*pMP3Stream).u.L1_2.cs_factor[i as usize][(k + 1) as usize]
                                * group9_table[n as usize][2] as f32;
                        k += 1; /* skip right chan dispatch */
                    }
                    23 => {
                        unpackj_n(s, i, k, 4); /* 15 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    24 => {
                        unpackj_n(s, i, k, 5); /* 31 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    25 => {
                        unpackj_n(s, i, k, 6); /* 63 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    26 => {
                        unpackj_n(s, i, k, 7); /* 127 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    27 => {
                        unpackj_n(s, i, k, 8); /* 255 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    28 => {
                        unpackj_n(s, i, k, 9); /* 511 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    29 => {
                        unpackj_n(s, i, k, 10); /* 1023 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    30 => {
                        unpackj_n(s, i, k, 11); /* 2047 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    31 => {
                        unpackj_n(s, i, k, 12); /* 4095 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    32 => {
                        unpackj_n(s, i, k, 13); /* 8191 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    33 => {
                        unpackj_n(s, i, k, 14); /* 16383 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    34 => {
                        unpackj_n(s, i, k, 15); /* 32767 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    35 => {
                        unpackj_n(s, i, k, 16); /* 65535 levels */
                        k += 1; /* skip right chan dispatch */
                    }
                    /* -- end of dispatch -- */
                    37 => {
                        skip((*pMP3Stream).u.L1_2.bit_skip);
                        s = s.add(3 * 64);
                        break;
                    }
                    36 => {
                        s = s.add(3 * 64);
                        break;
                    }
                    _ => break,
                } /* end switch */
            }
            j += 1;
        } /* end j loop */
        i += 1;
    } /* end i loop */
}

/*-------------------------------------------------------------------------*/
#[no_mangle]
pub static mut gpNextByteAfterData: *mut c_uchar = null_mut();

#[no_mangle]
pub unsafe extern "C" fn audio_decode(
    bs: *mut c_uchar,
    pcm: *mut c_short,
    pNextByteAfterData: *mut c_uchar,
) -> IN_OUT {
    gpNextByteAfterData = pNextByteAfterData; // sigh....
    match audio_decode_routine {
        Some(f) => f(bs, pcm),
        None => IN_OUT {
            in_bytes: 0,
            out_bytes: 0,
        },
    }
}

/*-------------------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn L2audio_decode(bs: *mut c_uchar, pcm: *mut c_short) -> IN_OUT {
    let sync: c_int;
    let prot: c_int;
    let mut in_out: IN_OUT;

    load_init(bs); /* initialize bit getter */
    /* test sync */
    in_out = IN_OUT {
        in_bytes: 0,
        out_bytes: 0,
    }; /* assume fail */
    sync = load(12) as c_int;
    if sync != 0xFFF {
        return in_out; /* sync fail */
    }

    load(3); /* skip id and option (checked by init) */
    prot = load(1) as c_int; /* load prot bit */
    load(6); /* skip to pad */
    (*pMP3Stream).pad = load(1) as c_int;
    load(1); /* skip to mode */
    (*pMP3Stream).u.L1_2.stereo_sb = look_joint[load(4) as usize];
    if prot != 0 {
        load(4); /* skip to data */
    } else {
        load(20); /* skip crc */
    }

    unpack_ba(); /* unpack bit allocation */
    unpack_sfs(); /* unpack scale factor selectors */
    unpack_sf(); /* unpack scale factor */
    unpack_samp(); /* unpack samples */

    if let Some(sbt) = (*pMP3Stream).u.L1_2.sbt {
        sbt(addr_of_mut!(sample).cast::<f32>(), pcm, 36);
    }
    /*-----------*/
    in_out.in_bytes = (*pMP3Stream).framebytes + (*pMP3Stream).pad;
    in_out.out_bytes = (*pMP3Stream).outbytes;

    in_out
}

/*-------------------------------------------------------------------------*/
